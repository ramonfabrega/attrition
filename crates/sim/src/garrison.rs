//! Garrisons — units inside buildings, and coming out. `docs/CITIES.md` §6.
//!
//! The original keeps one doubly-linked chain per container (`inside_down` on
//! the building, `inside_up`/`inside_down` on each unit) and appends at the
//! bottom; here the chain is a `Vec` of squad captains on the building, in
//! the order they entered, which is the same FIFO. A unit inside is off the
//! map: it does not move, fight, gather, bleed or get shot, and the only thing
//! it does is heal.

use crate::attrition::Domain;
use crate::build::{self, Ident, flags};
use crate::combat::Obj;
use crate::orders::Coll;
use crate::world::UNITS_PER_TILE;
use crate::{Player, Sim};

/// What a unit type needs in order to garrison — the columns of
/// `unitrules.xml` `UnitTypeData::can_garrison` reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitTraits {
    /// `unit_flags & 0x1800` — `FLAGS` letters `l` or `m`: may enter a city,
    /// a tower or a fort. The code does not tell the two apart.
    pub fortify: bool,
    /// `TypeData::where` — the building type that trains it, as an index into
    /// [`Sim::build_types`]; the unit may garrison in that lineage and its
    /// sibling military buildings.
    pub trained_at: Option<usize>,
    /// `UnitData::is_gov_hero`: takes one slot whatever its control cost.
    pub gov_hero: bool,
    /// `unit_flags & 0x10` on a sea unit — transports and merchant fleets
    /// never garrison.
    pub transport: bool,
}

/// Why a garrison order was refused — the gates of `Unit::do_garrison`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GarrisonRefused {
    /// The target is dead or not a typed building.
    NoBuilding,
    /// Construction is not finished.
    Inactive,
    /// Not the unit's owner's building, nor an ally's.
    NotOwner,
    /// `get_garrison_limit == 0`.
    NoCapacity,
    /// `can_garrison` says no for this pair of types.
    CantGarrison,
    /// The unit is not next to the building yet.
    NotAdjacent,
    /// A city that is not assimilated admits nobody.
    Unassimilated,
    /// A city under a tenth of its hit points admits nobody.
    CityTooHurt,
    /// No room.
    Full,
    /// The building stands on a tile an enemy owns.
    EnemyTerritory,
    /// The unit is already inside something, or dead.
    Unavailable,
}

impl Sim {
    /// `BuildTypeData::get_garrison_limit(who)`: the column, plus the
    /// tower and fort garrison techs.
    pub fn garrison_limit(&self, b: usize) -> i32 {
        let Some(ty) = self.buildings[b].ty else {
            return 0;
        };
        let level = self.nation[self.buildings[b].owner as usize]
            .fort_garrison_level
            .clamp(1, 4);
        let mut limit = self.build_types[ty].garrison_max;
        if build::is_tower(&self.build_types, ty) {
            limit += (level - 1) * self.tuning.tower_garrison_upgrade;
        }
        if build::is_fort(&self.build_types, ty) {
            limit += (level - 1) * self.tuning.fort_garrison_upgrade;
        }
        limit
    }

    /// `UnitTypeData::can_garrison(unit type, building type)`.
    pub fn can_garrison(&self, unit_ty: usize, build_ty: usize) -> bool {
        let ut = &self.unit_types[unit_ty];
        let g = ut.garrison;
        let types = &self.build_types;
        let sea = matches!(ut.kind.domain, Domain::Sea);
        if sea && g.transport {
            return false;
        }
        if g.fortify
            && (build::is_fort(types, build_ty)
                || build::is_tower(types, build_ty)
                || build::is_city(types, build_ty))
        {
            return true;
        }
        if sea {
            return build::is_dock(types, build_ty);
        }
        let Some(w) = g.trained_at else {
            return false;
        };
        let where_ident = types[w].ident;
        if build::is(types, build_ty, where_ident) && where_ident != Ident::University {
            return true;
        }
        let bt = types[build_ty].ident;
        match where_ident {
            Ident::Barracks => matches!(bt, Ident::Barracks | Ident::Stable | Ident::AutoPlant),
            Ident::Stable | Ident::AutoPlant => matches!(
                bt,
                Ident::Barracks
                    | Ident::Stable
                    | Ident::AutoPlant
                    | Ident::SiegeFactory
                    | Ident::Factory
            ),
            Ident::SiegeFactory | Ident::Factory => {
                matches!(
                    bt,
                    Ident::Barracks
                        | Ident::Stable
                        | Ident::AutoPlant
                        | Ident::SiegeFactory
                        | Ident::Factory
                ) || build::is_fort(types, build_ty)
                    || build::is_city(types, build_ty)
            }
            _ => false,
        }
    }

    /// The garrison slots one squad takes — `num_inside(0)`'s per-captain
    /// term: a government patriot 1, anything else its control cost.
    fn slot_cost(&self, unit: usize) -> i32 {
        match self.units[unit].ty {
            Some(t) => {
                let ut = &self.unit_types[t];
                if ut.garrison.gov_hero {
                    1
                } else {
                    ut.price.pop
                }
            }
            None => 1,
        }
    }

    /// `ObjectData::num_inside(0)`: the slots occupied.
    pub fn num_inside(&self, b: usize) -> i32 {
        self.buildings[b]
            .garrison
            .iter()
            .map(|&u| self.slot_cost(u))
            .sum()
    }

    /// `ObjectData::num_inside(1)`: squads inside.
    pub fn squads_inside(&self, b: usize) -> i32 {
        self.buildings[b].garrison.len() as i32
    }

    /// The figures of a squad — every unit of the same owner sharing a
    /// captain.
    ///
    /// `combat.captain` is an **object number**, not a simulation index
    /// (`UnitData::captain` is `o`, which is what `damage_o` is compared
    /// against), so the owner is half the key. Without it a lone unit
    /// drags every other player's `o`-th unit into its squad — Gaia's
    /// animals included, which is how the first trained citizen to go
    /// through here put a herd inside London.
    fn squad_of(&self, captain: usize) -> Vec<usize> {
        let (c, who) = (
            self.units[captain].combat.captain,
            self.units[captain].owner,
        );
        self.units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.owner == who && u.combat.captain == c && u.alive())
            .map(|(i, _)| i)
            .collect()
    }

    fn captain_of(&self, unit: usize) -> usize {
        let (c, who) = (self.units[unit].combat.captain, self.units[unit].owner);
        self.units
            .iter()
            .position(|u| u.owner == who && u.index == c as i16 && u.alive())
            .unwrap_or(unit)
    }

    /// `Unit::do_garrison`'s gates, then `go_inside`. The unit must already
    /// be adjacent; [`Sim::order_garrison`] walks it there first.
    pub fn garrison(&mut self, unit: usize, b: usize) -> Result<(), GarrisonRefused> {
        let u = &self.units[unit];
        if !u.alive() || u.inside.is_some() {
            return Err(GarrisonRefused::Unavailable);
        }
        let bd = &self.buildings[b];
        let Some(bty) = bd.ty else {
            return Err(GarrisonRefused::NoBuilding);
        };
        if !bd.alive {
            return Err(GarrisonRefused::NoBuilding);
        }
        if !bd.active {
            return Err(GarrisonRefused::Inactive);
        }
        if !(bd.owner == u.owner || self.is_ally(u.owner, bd.owner)) {
            return Err(GarrisonRefused::NotOwner);
        }
        let limit = self.garrison_limit(b);
        if limit == 0 {
            return Err(GarrisonRefused::NoCapacity);
        }
        let Some(uty) = u.ty else {
            return Err(GarrisonRefused::CantGarrison);
        };
        if !self.can_garrison(uty, bty) {
            return Err(GarrisonRefused::CantGarrison);
        }
        let corner = self.tile_corner(bty, bd.pos);
        let (xs, ys) = (self.build_types[bty].x_size, self.build_types[bty].y_size);
        let t = u.pos.tile();
        if !(t.x >= corner.x - 1
            && t.x <= corner.x + xs
            && t.y >= corner.y - 1
            && t.y <= corner.y + ys)
        {
            return Err(GarrisonRefused::NotAdjacent);
        }
        if self.building_is_city(b) {
            if let Some(c) = bd.city
                && self.cities[c].alive
                && self.cities[c].race != Some(bd.owner)
            {
                return Err(GarrisonRefused::Unassimilated);
            }
            if bd.hits_now() / 10 > bd.hits_now() - bd.damage {
                return Err(GarrisonRefused::CityTooHurt);
            }
        }
        let cost = self.slot_cost(unit);
        if self.num_inside(b) + cost > limit {
            return Err(GarrisonRefused::Full);
        }
        if let crate::world::Owner::Player(p) = self.world.owner_at(bd.pos)
            && p != bd.owner
            && !self.is_ally(bd.owner, p)
        {
            return Err(GarrisonRefused::EnemyTerritory);
        }
        self.go_inside(unit, b);
        Ok(())
    }

    /// Gives a unit a garrison order, as the player issues it: it walks to
    /// the building and tries to enter each frame; the order dies on any
    /// refusal but adjacency (`docs/ORDERS.md` §5.7).
    pub fn order_garrison(&mut self, unit: usize, b: usize) {
        self.add_garrison_order(unit, b, false, crate::orders::QueuePos::New, true);
    }

    /// `Unit::go_inside`: the whole squad goes in, appended at the bottom.
    pub fn go_inside(&mut self, unit: usize, b: usize) {
        let captain = self.captain_of(unit);
        for f in self.squad_of(captain) {
            // `Object::remove_from_world`: off the map is out of both
            // collision indices (`docs/COLLISION.md` §2, §3).
            self.coll_remove(f);
            self.chain_remove(f);
            let u = &mut self.units[f];
            u.inside = Some(b);
            u.on_map = false;
            u.movement.dest = None;
            u.combat.target = None;
            u.combat.mandatory = false;
        }
        if !self.buildings[b].garrison.contains(&captain) {
            self.buildings[b].garrison.push(captain);
        }
    }

    /// `Unit::come_out@00617c10`: the squad leaves onto the exit ring.
    ///
    /// The ring is `(x_size + y_size) × 0x30 + UNIT_TRAIN_DISTANCE` out to
    /// `… + UNIT_TRAIN_MAX_DISTANCE` (`618411`; the boat arm's
    /// `BOAT_TRAIN_*` pair and its `+0x244` type term are not modelled), and
    /// the inner radius the search is *given* is that ring only while the
    /// building is alive — a dying one lets its garrison out from zero
    /// (`618437`, `flags & 1`). Then `find_nearby_spot` sweeps it from due
    /// **south**, and its first free quarter-tile centre is the spot.
    /// What the fallbacks are depends on the type's `block_radius`: zero
    /// takes a second pass at twice both radii and then the building's own
    /// position, so it never fails; non-zero — which is every unit any
    /// capture has trained — re-sweeps the *same* ring ignoring collision
    /// and refuses if that finds nothing, keeping the unit inside
    /// (`docs/CITIES.md` §11).
    ///
    /// **The bearing is diff-backed, not read.** The decompile aliases the
    /// angle's stack slot and the listing's own `[esp+0x2c]` is written only
    /// on the sibling arm, so the value was fixed by the dump: all five
    /// citizens run10's AI trains — frames 100, 206, 320, 1297 and 1505 —
    /// come out at `(42360, 17208)`, which is due south of London at exactly
    /// the inner radius and at no other bearing the sweep tries first. A
    /// capture on any map where the ground south of a trainer is blocked
    /// would separate this from a sweep that starts elsewhere.
    pub fn come_out(&mut self, unit: usize) -> bool {
        let captain = self.captain_of(unit);
        let Some(b) = self.units[captain].inside else {
            return false;
        };
        let bd = &self.buildings[b];
        let (xs, ys) = bd.ty.map_or((0, 0), |t| {
            (self.build_types[t].x_size, self.build_types[t].y_size)
        });
        let ring = (xs + ys) * 0x30 + self.tuning.unit_train_distance;
        let max = ring + (self.tuning.unit_train_max_distance - self.tuning.unit_train_distance);
        let min = if bd.alive { ring } else { 0 };
        let pos = bd.pos;
        let south = crate::movement::Angle(i32::MIN);
        // **The two arms are chosen on `block_radius` (`+0x240`), not on
        // `big_radius`**, which is what the first reading of this function
        // said (`618457`/`61852c`). A citizen's `BLOCK_RADIUS` is 1, so
        // every unit run10 trains takes the *second* arm: `FILTER_NOT_ME`
        // over the ring, then the **same** ring with `nocoll 1`, and a
        // refusal — the unit stays inside — if even that finds nothing
        // passable. The `block_radius == 0` arm is the one that doubles
        // the ring and falls back to the building's own position, and its
        // filter is `FILTER_ALL`, whose general test is a seam
        // (`docs/CITIES.md` §11, `docs/ORDERS.md` §10).
        let spot = if self.profile(Obj::Unit(captain)).block_radius == 0 {
            self.find_nearby_spot_coll(captain, pos, min, max, 0, south, None, Coll::None)
                .or_else(|| {
                    self.find_nearby_spot_coll(
                        captain,
                        pos,
                        min * 2,
                        max * 2,
                        0,
                        south,
                        None,
                        Coll::None,
                    )
                })
                .unwrap_or(pos)
        } else {
            let free = self.find_nearby_spot(captain, pos, min, max, 0, south, None);
            match free.or_else(|| {
                self.find_nearby_spot_coll(captain, pos, min, max, 0, south, None, Coll::None)
            }) {
                Some(spot) => spot,
                None => return false,
            }
        };
        self.buildings[b].garrison.retain(|&c| c != captain);
        for f in self.squad_of(captain) {
            let u = &mut self.units[f];
            u.inside = None;
            u.on_map = true;
            u.pos = spot;
            u.movement = crate::Movement {
                speed: u.movement.speed,
                turning: u.movement.turning,
                ..crate::Movement::at(spot)
            };
            // `Object::add_to_world` reaches `update_seen(0)` — the whole
            // disc, not the ring (`docs/VISION.md` §6) — and both
            // collision indices (`docs/COLLISION.md` §2, §3).
            self.coll_add(f);
            self.chain_add(f);
            self.update_seen(f, false);
        }
        // The city alarm clears when the city empties.
        if self.building_is_city(b)
            && self.buildings[b].garrison.is_empty()
            && let Some(c) = self.buildings[b].city
        {
            self.cities[c].alarm = false;
        }
        // The function's own tail: the army coin, thrown once the unit is
        // out and by the scout lines alone ([`Sim::come_out_join_army`],
        // `docs/ARMY.md` §4). A trained unit reaches it through
        // `Build::train`, which is what run54's two `+0x25b0` draws are.
        self.come_out_join_army(captain);
        true
    }

    /// `Object::eject_contents` on a building: deferred — one squad a frame
    /// from the head, through [`Sim::process_ejection`].
    pub fn eject_contents(&mut self, b: usize, _kill_if_stuck: bool) {
        if !self.buildings[b].garrison.is_empty() {
            self.buildings[b].eject_pending = true;
        }
    }

    /// `Build::process_ejection`: the head squad out, FIFO; a dead building
    /// keeps its slot while it empties.
    pub fn process_ejection(&mut self, b: usize) {
        let Some(&head) = self.buildings[b].garrison.first() else {
            self.buildings[b].eject_pending = false;
            return;
        };
        self.come_out(head);
        if !self.buildings[b].alive {
            self.buildings[b].hold_frames += 1;
        }
    }

    /// The `inside_up ≥ 0` branch of `Unit::process_healing`: the garrison
    /// heal. Runs for every unit, every frame; does something only for a
    /// damaged captain inside a building that heals its kind.
    pub(crate) fn garrison_heal(&mut self, i: usize, frame: i64) {
        let u = &self.units[i];
        let Some(b) = u.inside else {
            return;
        };
        if !u.alive() || self.captain_of(i) != i || matches!(u.kind.domain, Domain::Air) {
            return;
        }
        let damaged = self
            .squad_of(i)
            .iter()
            .any(|&f| self.units[f].health < self.units[f].max_health);
        if !damaged {
            return;
        }
        let bd = &self.buildings[b];
        let Some(bty) = bd.ty else {
            return;
        };
        let owner = u.owner as usize;
        let n = &self.nation[owner];
        let level = n.heal_level.clamp(0, 3) as usize;
        let mut period = self.tuning.unit_heal_rate[level];
        if self.build_types[bty].ident == Ident::RedFort || n.red_fort {
            period = period * 100 / (self.tuning.red_fort_heal + 100);
        }
        if matches!(u.kind.domain, Domain::Sea) && build::is_dock(&self.build_types, bty) {
            period = (2 * period + 2) / 3;
        }
        let period = i64::from(period.max(1));
        if (i64::from(u.index) + frame) % period != 0 {
            return;
        }
        let types = &self.build_types;
        let trained =
            u.ty.and_then(|t| self.unit_types[t].garrison.trained_at)
                .is_some_and(|w| build::is(types, bty, types[w].ident));
        let eligible = trained
            || build::is_city(types, bty)
            || build::is_fort(types, bty)
            || build::is_tower(types, bty);
        if !eligible {
            return;
        }
        for f in self.squad_of(i) {
            let u = &mut self.units[f];
            if u.health < u.max_health {
                u.health += 1;
            }
        }
    }

    /// The garrison's contribution to a tower's arrows —
    /// `count_inside(GARRISON_ARROWS)`: `attack / 10` per garrisoned captain
    /// that is FOOT and not siege, halved for a melee one. `docs/COMBAT.md`
    /// §8.6 (the militia third is not modelled).
    pub(crate) fn garrison_attack_sum(&self, b: usize) -> i32 {
        let mut sum = 0;
        for &c in &self.buildings[b].garrison {
            let u = &self.units[c];
            if !u.alive() || u.kind.siege {
                continue;
            }
            let p = self.profile(Obj::Unit(c));
            // FOOT is obj_masks letter `F`, bit 5.
            if p.obj_masks & 0x20 == 0 {
                continue;
            }
            let a = self.attack_of(Obj::Unit(c)) / 10;
            sum += if p.max_range == 0 { (a + 1) / 2 } else { a };
        }
        sum
    }

    /// Whether a building type is a hangar — `can_carry(AIR)`.
    pub fn is_hangar(&self, ty: usize) -> bool {
        matches!(
            self.build_types[ty].ident,
            Ident::Airbase | Ident::MissileSilo
        )
    }

    /// A building's garrison-related flags, for tests: whether it needs a
    /// city and its limit.
    pub fn needs_city(&self, ty: usize) -> bool {
        !self.build_types[ty].has(flags::NO_CITY)
    }
}

/// One tile, in position units — the exit ring is measured in these.
pub const TILE: i32 = UNITS_PER_TILE;

/// A player index, re-exported for the garrison tests.
pub type Who = Player;
