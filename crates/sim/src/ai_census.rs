//! The census — `Leader::plan_strategy@006b9620`'s sweep after the cadence
//! check, `docs/AI.md` §2.3 steps 1–16: the recount of everything a
//! computer leader owns into [`crate::ai::Census`] and the per-city
//! [`crate::ai::CityAi`] records. The tail (step 17) — the orphan check,
//! `compute_sites`, arming the step machine — is the driver's.
//!
//! Oracle: a `LEADERS=9` dump prints every field under the same name
//! (`tools/gamelog/leader.py FRAME WHO`); run8's frame 1 is the AI leader
//! after its frame-0 sweep.
//!
//! # Region numbering
//!
//! The original packs land regions into `0..0x3f` and sea regions into
//! `0x3f..0x7e`, stores the sea ones `% 0x3f` in its 63-entry arrays, and
//! tests "is this a land region" as `r < 0x40`. Ours are one flat list, one
//! slot per region either way, so every `r < 0x40` becomes
//! `world.terrain(r) == Land` and every `% 0x3f` disappears. That is the
//! only deliberate departure from the original's indexing in this module.
//!
//! # Seams
//!
//! The simulation does not carry every input the sweep reads. Each missing
//! one is a private fn below whose name is the original's and whose answer
//! is what an empty map would give; they are listed in one place at
//! [`seams`]. Nothing else here is approximate.

use crate::ai::{Census, CityAi};
use crate::attrition::Domain;
use crate::build;
use crate::city::radius_for_level;
use crate::combat::Obj;
use crate::economy::RESOURCES;
use crate::orders::{Body, Worker, index};
use crate::world::{Cell, Owner, Pos, Terrain, tile, vector_dist};
use crate::{Player, Sim};

/// The rate step 1 writes into every good's `escrow_rate` slot.
const ESCROW_RATE: i32 = 40;

/// `CityData::peasant_dist`'s "no citizen anywhere near" value.
const PEASANT_DIST_NONE: i32 = 100;

/// Position units per cell — the divisor `peasant_dist` is measured in.
const CELL: i32 = 0x300;

/// `TypeIndex` values the sweep hardcodes and the simulation cannot name any
/// other way. Citizens (`0x32`/`0x33`) and scholars (`0x34`/`0x35`) are
/// [`Worker`] instead, which is how `rondata` loads them.
mod ty {
    use crate::tech::TypeId;
    /// `MERCHANT`, `ECHINESEMERCHANT`, `MERCHANT2` — the three base types
    /// the merchant test compares against exactly.
    pub const MERCHANTS: [TypeId; 3] = [0x3d, 0x3e, 0x190];
    /// `CARAVAN` — the lineage test `is(0x3b)`.
    pub const CARAVAN: TypeId = 0x3b;
    /// `missile` splits on these two exact types.
    pub const CRUISE: TypeId = 0x139;
    pub const NUKE: TypeId = 0x13b;
    /// `air` splits on these two.
    pub const FIGHTER: TypeId = 0x11f;
    pub const BOMBER: TypeId = 0x130;
}

/// `UnitTypeData::role & 0x10000` — a military unit.
const ROLE_MILITARY: u32 = 0x1_0000;
/// `role & 0x10` — the bit that, with `0x10000`, keeps a unit out of the
/// invader count (`role & 0x10010 == 0x10000`).
const ROLE_NO_INVADE: u32 = 0x10;
/// `ObjectTypeData::obj_masks & 0x8000000` — a missile rather than a plane.
const OBJ_MASK_MISSILE: u32 = 0x800_0000;

/// The seams, in one list, so a reader does not have to find them.
///
/// | Seam | Answers | Why |
/// | --- | --- | --- |
/// | `village_num` | 0 | `LeaderData::village_num` has no counterpart here. |
/// | `GoodData::ever_seen` | unwritten | step 9 sets the leader's bit on every rare it counts; only `GoodData::is_seen` reads it, and nothing here asks a good whether it is seen. |
/// | `is_seen` / step 6 | skipped whole | step 6 is the `ALLY_LOS`/`reveal_map` path, and no capture sets either; **the live path to the met bit is the fog** — `check_ever_seen` → `Leader::meet` (`docs/VISION.md` §6.2), and [`Sim::has_met`] reads the bit it sets. |
/// | `ScenarioData::ally_mask` | 0 | scenarios are cut from v1. |
/// | `unit_masks & 1` | clear | the flag that excludes an object from the census is unmodelled. |
/// | `unit_masks & 0x80000` | clear | "packed/idle" is unmodelled; every merchant counts in `reg_unpack_merch`, no fisherman is idle. |
/// | order kinds 8 / 0xe | never | build-and-repair-through-a-transport is unmodelled. |
///
/// Two seams closed 2026-08-25 with `docs/TRANSPORT.md`: `Region::num_coasts`
/// is computed from the cells (`World::rebuild_coasts`) and
/// `BuildTypeData::is_dock_tile` is `Sim::is_dock_tile`; `carry` and
/// `unit_flags2 & 4` come from the type's columns.
/// | region flags `& 8` | clear | step 15's two-landmass expand probe never fires. |
/// | `Armies::init_navy` | seeded by `create_units`' sea branch | step 16 is `Sim::census_seed_army` (`docs/ARMY.md`); the sea branch seeds the navy since item 579 (`docs/ORDERS.md` §25.3). |
/// | `check_explore`'s visibility | every region cell counts | no fog: the leader has seen the map. |
///
/// `home_reg`, `pop`, `control` and `scouts` are *not* seams: the sweep
/// reads them and never writes them, so they stay whatever the rest of the
/// simulation last put there.
pub mod seams {}

impl Sim {
    // ------------------------------------------------------------------
    // Seams
    // ------------------------------------------------------------------

    /// `treaties[i] & 1`, "I have met leader `i`" — the **whole** gate steps
    /// 14 and 15 carry, and until item 385 the one thing this crate had no
    /// way to derive.
    ///
    /// There is **no `human` test anywhere in either loop**:
    /// `plan_strategy@006b9620:1511,1557` gates on `leader_flags & 2`,
    /// `i != who` and this bit, and the human's `leader_flags` is 7. The
    /// original sets the bit on *first contact* and never clears it in a
    /// game, and the live path to it is the fog:
    /// [`Sim::check_ever_seen`](crate::Sim::check_ever_seen) →
    /// [`Sim::meet`] → [`Sim::treaty_on`]. `docs/VISION.md` §6.2.
    ///
    /// What this replaced, and why the replacement is the right way round:
    /// the human was skipped here as a stand-in for the missing bit, which
    /// on a one-human-one-AI capture is "never met" — 2,000 frames of Great
    /// Lakes' word closer than answering "yes" from frame 1, which is what
    /// removing the skip did (9182 → 7182, `docs/AI.md` §45). The dumps say
    /// what the right answer is: player 1's `treaties[0]` is **0** on blocks
    /// 6950 and 7514 and **1** on 8174 and 9170, with `diplos[0]` at 0 — at
    /// war — on all four, so the diplomacy never moves and only contact
    /// does, once, in (7616, 8174].
    fn met(&self, who: Player, other: usize) -> bool {
        other != who as usize && !self.defeated[other] && self.has_met(who, other)
    }

    /// `treaties[other] & 1` — the met bit, and the field the leader-record
    /// comparison prints. `defeated` is not part of the original's bit; it
    /// is [`Sim::met`]'s own `leader_flags & 2` folded in here so both
    /// readers agree.
    pub fn has_met(&self, who: Player, other: usize) -> bool {
        other == who as usize
            || (!self.defeated[other]
                && self
                    .treaties
                    .get(who as usize)
                    .and_then(|r| r.get(other))
                    .is_some_and(|t| t & 1 != 0))
    }

    // ------------------------------------------------------------------
    // Small readers the sweep leans on
    // ------------------------------------------------------------------

    /// Whether a region index names land — the original's `r < 0x40`.
    fn region_is_land(&self, r: u16) -> bool {
        (r as usize) < self.world.region_count() && self.world.terrain(r) == Terrain::Land
    }

    /// `diplos[a][b] == 0` — at war. The original tests either side.
    fn diplo_war(&self, a: usize, b: usize) -> bool {
        self.at_war[a][b]
    }

    /// `diplos[a][b] == 2` — allied. The original tests both sides.
    fn diplo_ally(&self, a: usize, b: usize) -> bool {
        self.allied[a][b]
    }

    /// `LeaderData::territory`: the cells this leader owns.
    fn territory_of(&self, who: Player) -> i32 {
        let mut n = 0;
        for y in 0..self.world.height() {
            for x in 0..self.world.width() {
                if self.world.owner(Cell::new(x, y)) == Owner::Player(who) {
                    n += 1;
                }
            }
        }
        n
    }

    /// `LeaderData::get_team_terr@006d62e0`: my territory plus every ally's.
    fn team_terr(&self, who: Player) -> i32 {
        let w = who as usize;
        (0..self.players.len())
            .filter(|&i| i == w || (!self.defeated[i] && self.diplo_ally(w, i)))
            .map(|i| self.territory_of(i as Player))
            .sum()
    }

    /// `reg_buildings[r][0] + [1] + [2] + [0x75]` — the four city types, as
    /// the building lifecycle keeps them: active city buildings of `who`
    /// whose cell is in `r`.
    /// [`Sim::reg_city_buildings`] for the one caller outside the sweep —
    /// `was_seen`'s territory shortcut, which asks about a leader whose
    /// census may never have run.
    pub(crate) fn reg_city_buildings_pub(&self, who: Player, r: u16) -> i32 {
        self.reg_city_buildings(who, r)
    }

    fn reg_city_buildings(&self, who: Player, r: u16) -> i32 {
        self.buildings
            .iter()
            .filter(|b| {
                b.alive
                    && b.active
                    && b.owner == who
                    && b.ty
                        .is_some_and(|t| build::city_level(&self.build_types, t) > 0)
                    && self.world.region_of(b.pos.cell()) == Some(r)
            })
            .count() as i32
    }

    /// `UnitTypeData::domain` on a unit's type.
    fn unit_domain(&self, u: usize) -> Domain {
        self.units[u]
            .ty
            .map_or(Domain::Land, |t| self.unit_types[t].combat.domain)
    }

    /// The unit's `TypeIndex`, when the tree knows it.
    pub(crate) fn unit_tree(&self, u: usize) -> Option<crate::tech::TypeId> {
        self.units[u].ty.and_then(|t| self.unit_types[t].tree)
    }

    /// `ObjectData::is(t, 0)` on a unit — the lineage test.
    pub(crate) fn unit_line_is(&self, u: usize, t: crate::tech::TypeId) -> bool {
        self.unit_tree(u)
            .is_some_and(|ut| self.tech_tree.is(ut, t, false))
    }

    /// `ObjectsData::find_city@0065ba90(x, y, SEARCH_FRIENDLY, who, 0x200)`
    /// — the nearest live city of `who` **in the query point's own region**
    /// (`0x200` is that flag, not a radius: the search has no distance
    /// limit, `find_dist` starts at 99,999,999). Returns the city and
    /// `objects.find_dist`, in position units. Ties go to the later city,
    /// because the original compares `<=`.
    fn census_find_city(&self, who: Player, at: Pos) -> Option<(usize, i32)> {
        let reg = self.world.region_of(at.cell());
        let mut best: Option<(usize, i32)> = None;
        for (i, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who || c.reg != reg {
                continue;
            }
            let d = vector_dist(c.pos.x - at.x, c.pos.y - at.y);
            if best.is_none_or(|(_, bd)| d <= bd) {
                best = Some((i, d));
            }
        }
        best
    }

    /// The region a unit counts in (`docs/AI.md` §2.3 step 10): its own
    /// tile's, with the coastal `region2` refinement; when garrisoned, the
    /// container's — except that a **sea-domain** unit inside a water-domain
    /// building takes the first ocean tile of the container's footprint, so
    /// a ship in a dock counts in the sea region. The result must be a sea
    /// region (`> 0x3e`) or the scan is discarded.
    ///
    /// **A container is a unit too** (item 592): `ObjectData::get_inside`
    /// walks `inside_up` to the outermost container, building or boat, and
    /// a land unit takes that container's tile, `region2` rule and all. A
    /// citizen riding a transport barge counts in the barge's region, not
    /// where it boarded (`docs/AI.md` §58). A sea-domain unit inside a boat
    /// asks the boat a vslot the export folds (`+0x108`); no such rider
    /// exists here, and it takes the boat's tile too.
    fn census_unit_region(&self, u: usize) -> Option<u16> {
        let unit = &self.units[u];
        if let Some(boat) = unit.inside_unit {
            return self.world.tregion_alt(self.units[boat].pos.tile());
        }
        let Some(b) = unit.inside else {
            return self.world.tregion_alt(unit.pos.tile());
        };
        let container = &self.buildings[b];
        let plain = self.world.tregion_alt(container.pos.tile());
        if self.unit_domain(u) != Domain::Sea {
            return plain;
        }
        let Some(bt) = container.ty.map(|t| &self.build_types[t]) else {
            return plain;
        };
        if bt.domain() == build::BuildDomain::Land {
            return plain;
        }
        let centre = container.pos.tile();
        let corner = Pos::new(centre.x - bt.x_size / 2, centre.y - bt.y_size / 2);
        for tx in corner.x..corner.x + bt.x_size {
            for ty in corner.y..corner.y + bt.y_size {
                let t = Pos::new(tx, ty);
                if self.world.tile_mask(t) & tile::SURFACE != tile::SURFACE_OCEAN {
                    continue;
                }
                if let Some(r) = self.world.tregion_alt(t)
                    && !self.region_is_land(r)
                {
                    return Some(r);
                }
                break;
            }
        }
        plain
    }

    /// `City::count_gather_slots@00737dc0(city, 0, 0)`'s return value: the
    /// gather slots of the city's own members, **knowledge excluded from
    /// the total** — the original skips good 3 when accumulating its return
    /// while still writing it into the per-good array it is not given here.
    ///
    /// `ai_units.rs` has the three-output form of the same function under
    /// the original's name; it is private to that module, so this is the
    /// total on its own until one of them is made `pub(crate)`.
    fn city_gather_slot_total(&self, c: usize) -> i32 {
        self.count_gather_slots(c).0
    }

    // ------------------------------------------------------------------
    // The sweep
    // ------------------------------------------------------------------

    /// The sweep, steps 1–16. Writes `self.ai[who].census` and
    /// `self.ai[who].city_ai`, and `invaders[who]` on every other leader.
    pub fn census(&mut self, who: Player) {
        let w = who as usize;
        let regions = self.world.region_count();
        let players = self.players.len();
        self.ai[w].census.resize(regions, players);
        let n = self.cities.len();
        self.ai[w].city_ai.resize(n, CityAi::default());

        self.census_escrow_rate(who);
        self.census_reset_cities(who);
        census_zero_scalars(&mut self.ai[w].census);
        self.census_territory(who);
        self.census_ally_mask(who);
        self.census_clear_invaders(who);
        self.census_zero_regions(who);
        self.census_rares(who);
        self.census_units(who);
        self.census_buildings(who);
        self.census_maxima(who);
        self.census_city_sites(who);
        self.census_wars(who);
        self.census_strategy(who);
        // Step 16, the army seeding (`docs/ARMY.md` §15).
        self.census_seed_army(who);
    }

    /// Step 1. Once the leader holds more than two cities and villages,
    /// every good escrows at 40 — and never goes back.
    fn census_escrow_rate(&mut self, who: Player) {
        if self.city_num(who) + self.village_num(who) > 2 {
            self.ai[who as usize].census.escrow_rate = [ESCROW_RATE; RESOURCES];
        }
    }

    /// Step 2, and the `in_port` half that the original defers until after
    /// the rares.
    fn census_reset_cities(&mut self, who: Player) {
        let w = who as usize;
        for c in self.cities_of(who) {
            let r = &mut self.ai[w].city_ai[c];
            r.free = 0;
            r.busy = 0;
            r.gatherers = 0;
            r.peasant_dist = PEASANT_DIST_NONE;
            r.in_port = 0;
        }
    }

    /// Step 4. My team's territory, and the widest and narrowest of every
    /// other computer leader's team that is not allied with me both ways.
    fn census_territory(&mut self, who: Player) {
        let w = who as usize;
        let mine = self.team_terr(who);
        let mut other = 0;
        let mut min_other = 0;
        for i in 0..self.players.len() {
            // **Every other leader the roster holds, the human included**
            // (item 368). `plan_strategy@006b9620:159-166`'s gate is
            // `leader_flags & 2`, `i != who` and `i >= 0` — and bit 1 is
            // set for *every* leader every capture prints, the human's
            // `leader_flags 7` among them. Skipping humans here read
            // `other_team_terr` as nought on a one-human-one-AI game,
            // which is every capture this crate is diffed against.
            if i == w {
                continue;
            }
            if self.diplo_ally(i, w) && self.diplo_ally(w, i) {
                continue;
            }
            let t = self.team_terr(i as Player);
            other = other.max(t);
            min_other = if min_other == 0 { t } else { min_other.min(t) };
        }
        let cs = &mut self.ai[w].census;
        cs.my_team_terr = mine;
        cs.other_team_terr = other;
        cs.min_other_team_terr = min_other;
    }

    /// Step 5, and step 6's one surviving effect: the mask is my own bit
    /// plus every leader allied with me both ways. The meeting pass itself
    /// is a seam.
    fn census_ally_mask(&mut self, who: Player) {
        let w = who as usize;
        let mut mask = 1u32 << (w & 0x1f);
        for i in 0..self.players.len() {
            if i == w || self.defeated[i] || self.nation[i].human {
                continue;
            }
            if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                mask |= 1 << (i & 0x1f);
            }
        }
        let cs = &mut self.ai[w].census;
        cs.filled_gather_slots = [0; RESOURCES];
        cs.ally_mask = mask;
    }

    /// Step 7. `invaders[who]` is cleared on **every** active leader,
    /// mine included; step 10 refills it.
    fn census_clear_invaders(&mut self, who: Player) {
        let w = who as usize;
        let players = self.players.len();
        for i in 0..players {
            if self.defeated[i] {
                continue;
            }
            let inv = &mut self.ai[i].census.invaders;
            inv.resize(players, 0);
            inv[w] = 0;
        }
    }

    /// Step 8. Every per-region array back to zero, and `reg_cities`
    /// recounted from the four city types. `reg_pop`, `reg_forts`,
    /// `reg_docks`, `reg_terr` and `reg_buildings` are *not* zeroed — they
    /// belong to the building lifecycle.
    fn census_zero_regions(&mut self, who: Player) {
        let w = who as usize;
        let cities: Vec<i32> = (0..self.world.region_count())
            .map(|r| self.reg_city_buildings(who, r as u16))
            .collect();
        let cs = &mut self.ai[w].census;
        for v in [
            &mut cs.reg_active,
            &mut cs.reg_combat,
            &mut cs.reg_attack,
            &mut cs.reg_naval,
            &mut cs.reg_transports,
            &mut cs.reg_defense,
            &mut cs.reg_attacked,
            &mut cs.reg_land,
            &mut cs.reg_peasants,
            &mut cs.reg_free_peasants,
            &mut cs.reg_xport_peasants,
            &mut cs.reg_gatherers,
            &mut cs.reg_gather_slots,
            &mut cs.reg_known_rares,
            &mut cs.reg_unpack_merch,
        ] {
            v.iter_mut().for_each(|s| *s = 0);
        }
        cs.reg_cities.copy_from_slice(&cities);
        // `pop` and `reg_pop` belong to the city lifecycle and not to this
        // sweep — [`Sim::sync_leader_pop`] keeps them, and every writer the
        // original has is one of its call sites. It is repeated here for
        // one reason: the per-region arrays are `resize`d by the sweep, so
        // a `sync_pop_cities` that ran before the first one (`build_sim`
        // standing a dump up) wrote `reg_pop` into an empty vector.
        self.sync_leader_pop();
    }

    /// Step 9, the rares — `plan_strategy@006b9620:314–361`, the only
    /// writer `reg_known_rares` (`LeaderData +0x4d4`) has. Every good in
    /// the leader's `new_rares` list is counted into its cell's region
    /// when two things hold:
    ///
    /// * the leader or an ally has **explored** the half-cell the good
    ///   stands in — `seen2[div_3_table[y >> 7]][div_3_table[x >> 7]] &
    ///   ally_mask`, which is [`Sim::was_really_seen_fog`] exactly, exits
    ///   and all (`who >= 8`, `reveal_map == 3`, and the two leader flags
    ///   it keeps as seams);
    /// * the cell under it is **unowned, mine, or a mutual ally's** — the
    ///   `WData` owner byte is negative, or `who`'s, or `diplos` reads 2
    ///   both ways.
    ///
    /// The list only grows (`Leader::new_rare@006d9e70`), so a rare once
    /// seen is counted on every sweep until someone else's border covers
    /// it. `Leader::calc_gather@006ceee0` sums the array into
    /// `known_rares` under its own cadence ([`Sim::assemble_holdings`]),
    /// and `create_units`' merchant arm reads that sum — `docs/AI.md` §55.
    fn census_rares(&mut self, who: Player) {
        let w = who as usize;
        let list = self.ai[w].new_rares.clone();
        for gi in list {
            let Some(pos) = self.world.goods().get(gi).map(|g| g.pos) else {
                continue;
            };
            // `div_3_table[c >> 7]`: the half-cell, `c / 0x180`.
            let half = |c: i32| c.div_euclid(crate::world::UNITS_PER_CELL / 2);
            if !self.was_really_seen_fog(half(pos.x), half(pos.y), who) {
                continue;
            }
            let cell = pos.cell();
            if let Owner::Player(o) = self.world.owner(cell)
                && o != who
                && !(self.diplo_ally(w, o as usize) && self.diplo_ally(o as usize, w))
            {
                continue;
            }
            let Some(r) = self.world.region_of(cell) else {
                continue;
            };
            if let Some(slot) = self.ai[w].census.reg_known_rares.get_mut(usize::from(r)) {
                *slot += 1;
            }
        }
    }

    /// Step 10, the unit census.
    fn census_units(&mut self, who: Player) {
        let w = who as usize;
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() {
                continue;
            }
            // **Captains only** — `Group::get_num_cap@007145c0`'s `is_captain`
            // through vslot `+0xe8`, the sweep's own iteration (`docs/AI.md`
            // §2.3 step 10). A follower of a squad is an object of mine and
            // is not one of my units for any of these counters: run91's
            // leader 1 stands nine Longbowmen in three squads and three
            // Hoplites in one, and the original's `active` is **32** — the
            // roster less those eight followers, to the unit. Item 295,
            // `docs/AI.md` §35.
            if !self.is_captain(u) {
                continue;
            }
            // A decoy is no unit of mine (`testb $0x1, 0x68(%esi)` at
            // `6b9f57`, beside the captain test): run346's six decoy squads
            // of 11637 stay out of `non_siege`, and `create_units`' military
            // gate reads the count without them (item 1302).
            if unit.decoy {
                continue;
            }
            // `type->control_cost != 0`: a unit that costs no population is
            // not counted at all.
            let control_cost = unit.ty.map_or(0, |t| self.unit_types[t].price.pop);
            if control_cost == 0 {
                continue;
            }
            let Some(reg) = self.census_unit_region(u) else {
                continue;
            };
            let land_reg = self.region_is_land(reg);
            let domain = self.unit_domain(u);
            let profile = unit.ty.map(|t| &self.unit_types[t].combat);
            // **`UnitTypeData::role`, not [`crate::combat::Profile::roles`]**
            // — item 295. The two words share no bit assignment: this
            // crate's own profile bitfield puts `CARAVAN` at `1 << 16` and
            // `V2ROCKET` at `1 << 4`, exactly where the original's role word
            // keeps `combat_role` and the scout bit, so the sweep read a
            // caravan as the leader's only soldier and every Hoplite and
            // Longbowman as a civilian. `docs/AI.md` §35.
            let roles = unit.ty.map_or(0, |rec| self.role_word_of_rec(rec));
            let obj_masks = profile.map_or(0, |p| p.obj_masks);
            let siege = profile.is_some_and(|p| p.siege);
            let military = roles & ROLE_MILITARY != 0;
            // `UnitTypeData::carry` and `unit_flags2 & 4` — a transport and
            // a fisherman.
            let carry = unit.ty.is_some_and(|t| self.unit_types[t].cols.carry != 0);
            let fisherman = unit
                .ty
                .is_some_and(|t| self.unit_types[t].cols.unit_flags2 & 4 != 0);
            // `attack() / 10`, or a flat 10 off the ground.
            let strength = if domain == Domain::Land {
                self.attack_of(Obj::Unit(u)) / 10
            } else {
                10
            };

            self.ai[w].census.active += 1;
            if military {
                let cs = &mut self.ai[w].census;
                cs.attack += strength;
                cs.combat += 1;
                if siege {
                    cs.siege += 1;
                } else {
                    cs.non_siege += 1;
                }
            }
            match domain {
                Domain::Sea => {
                    let cs = &mut self.ai[w].census;
                    cs.naval += strength;
                    if strength != 0 {
                        cs.sea_combat += 1;
                    }
                    if carry {
                        cs.transports += 1;
                    }
                    if fisherman {
                        cs.fishermen += 1;
                    }
                }
                Domain::Air => {
                    let cs = &mut self.ai[w].census;
                    if obj_masks & OBJ_MASK_MISSILE != 0 {
                        cs.missile += 1;
                    } else {
                        cs.air += 1;
                    }
                    if obj_masks & OBJ_MASK_MISSILE != 0 {
                        if self.unit_line_is(u, ty::CRUISE) {
                            self.ai[w].census.cruise += 1;
                        } else if self.unit_line_is(u, ty::NUKE) {
                            self.ai[w].census.nuke += 1;
                        }
                    } else if self.unit_line_is(u, ty::FIGHTER) {
                        self.ai[w].census.fighters += 1;
                    } else if self.unit_line_is(u, ty::BOMBER) {
                        self.ai[w].census.bombers += 1;
                    }
                }
                Domain::Land => {}
            }

            // The invader count: a military unit without `role & 0x10`,
            // standing on a cell another active leader owns.
            if roles & (ROLE_MILITARY | ROLE_NO_INVADE) == ROLE_MILITARY
                && let Some(o) = self.world.owner(self.units[u].pos.cell()).player()
                && o as usize != w
                && !self.defeated[o as usize]
            {
                let players = self.players.len();
                let inv = &mut self.ai[o as usize].census.invaders;
                inv.resize(players, 0);
                inv[w] += 1;
            }

            let mut reg = reg;
            let mut land_reg = land_reg;
            match self.worker_of(u) {
                Worker::Scholar => self.census_scholar(who, u),
                Worker::Citizen => {
                    // **The region is one local, and the rider's arm writes
                    // it** (item 592): `plan_strategy`'s `xport` arm keeps
                    // the move point's region in the variable the unit's
                    // own region lives in, and the `reg_active` family
                    // below reads it after. run143's 10576 prints the rider
                    // `1/31` in `reg_active[5]`, the barge's destination,
                    // and in no region of the barge's own tile
                    // (`docs/AI.md` §58).
                    if let Some(r) = self.census_citizen(who, u, reg, land_reg) {
                        reg = r;
                        land_reg = self.region_is_land(r);
                    }
                }
                Worker::None => self.census_trader(who, u, reg),
            }

            // `reg_active`/`reg_combat`/`reg_attack` count only where the
            // unit's domain matches the region's kind, or the unit flies.
            let matches_region = match domain {
                Domain::Air => true,
                Domain::Land => land_reg,
                Domain::Sea => !land_reg,
            };
            if !matches_region {
                continue;
            }
            let r = reg as usize;
            let cs = &mut self.ai[w].census;
            cs.reg_active[r] += 1;
            if military {
                cs.reg_attack[r] += strength;
                cs.reg_combat[r] += 1;
            }
            if domain == Domain::Sea && !land_reg {
                cs.reg_naval[r] += strength;
                if carry {
                    cs.reg_transports[r] += 1;
                }
            }
        }
    }

    /// A scholar: counted, and its university slot filled when it is inside
    /// one.
    fn census_scholar(&mut self, who: Player, u: usize) {
        let w = who as usize;
        self.ai[w].census.scholars += 1;
        if let Some(b) = self.units[u].inside
            && self.building_ident(b) == build::Ident::University
        {
            self.ai[w].census.filled_gather_slots[3] += 1;
        }
    }

    /// A merchant or a caravan — the sweep's `else` arm, which every unit
    /// that is neither a scholar nor a citizen falls into.
    fn census_trader(&mut self, who: Player, u: usize, reg: u16) {
        let w = who as usize;
        let base = self.unit_tree(u);
        if base.is_some_and(|t| ty::MERCHANTS.contains(&t)) {
            self.ai[w].census.merchants += 1;
            // seam: `unit_masks & 0x80000` (packed) is never set.
            self.ai[w].census.reg_unpack_merch[reg as usize] += 1;
        } else if self.unit_line_is(u, ty::CARAVAN) {
            self.ai[w].census.caras += 1;
        }
    }

    /// A citizen: the nearest friendly city, the per-region peasant counts,
    /// and then the split by the *action*'s kind.
    ///
    /// Returns the region the `xport` arm left in the sweep's region local,
    /// when it ran (see [`Sim::census_citizen_inside`]).
    fn census_citizen(&mut self, who: Player, u: usize, reg: u16, land_reg: bool) -> Option<u16> {
        let w = who as usize;
        let found = self.census_find_city(who, self.units[u].pos);
        self.ai[w].census.peasants += 1;

        // `is_on_map`: the sign of `inside_up`, so a rider on a boat is
        // off the map exactly as a garrison is (item 592).
        if self.units[u].inside.is_some() || self.units[u].inside_unit.is_some() {
            return self.census_citizen_inside(who, u);
        }

        let r = reg as usize;
        let kind = self
            .action_of(u)
            .map_or(index::NONE, |i| self.units[u].orders[i].index());

        if land_reg {
            self.ai[w].census.reg_peasants[r] += 1;
            if self.ai[w].census.reg_cities[r] == 0 && kind != index::BUILD_AT {
                self.ai[w].census.reg_xport_peasants[r] += 1;
                self.ai[w].census.xport_peasants += 1;
            }
        }

        if kind == index::NONE || kind == index::EXPLORE_TO {
            self.ai[w].census.free_peasants += 1;
            if land_reg {
                self.ai[w].census.reg_free_peasants[r] += 1;
            }
            if let Some((c, dist)) = found {
                let rec = &mut self.ai[w].city_ai[c];
                rec.free = rec.free.wrapping_add(1);
                rec.peasant_dist = rec.peasant_dist.min(dist / CELL);
            }
            return None;
        }

        if let Some((c, _)) = found {
            self.ai[w].city_ai[c].busy += 1;
        }
        if kind != index::GATHER {
            // Kinds 8 and 0xe — build and repair from inside a transport —
            // are a seam; nothing else in the original's table counts here.
            return None;
        }

        let target = self
            .action_of(u)
            .and_then(|i| match self.units[u].orders[i].body {
                Body::Gather(g) => Some(g.building),
                _ => None,
            });
        // Knowledge is the scholar's, not a gatherer's: the original's
        // switch has no `0x1a4` arm here.
        if let Some(b) = target
            && let Some(good) = crate::ai_place::gather_good(self.building_ident(b))
            && good != 3
        {
            self.ai[w].census.filled_gather_slots[good] += 1;
        }
        if let Some((c, _)) = found {
            self.ai[w].city_ai[c].busy -= 1;
        }
        // **A gatherer counts in its building's city, not the nearest one**
        // (item 752, `docs/AI.md` §73). The listing keeps the found city in
        // `edi` and overwrites it with the target building's `+0x72` city
        // at `6babfb` (`movswl 0x72(%eax), %edi`), once the building is
        // the leader's own and its city is not negative; `+0x5c`'s
        // increment at `6bac3c` and the `peasant_dist` minimum after it
        // index `edi`. The decompiler prints the same reuse as `iVar25`.
        // The distance stays the found city's, `objects+0x1fc`.
        let mut counted = found.map(|(c, _)| c);
        if let Some(b) = target
            && self.buildings[b].owner == who
            && let Some(tc) = self.buildings[b].city
        {
            self.ai[w].city_ai[tc].busy += 1;
            counted = Some(tc);
        }
        self.ai[w].census.gatherers += 1;
        if let Some(c) = counted {
            let rec = &mut self.ai[w].city_ai[c];
            rec.gatherers += 1;
            // SEAM: a gatherer with no friendly city in its region but a
            // target in one reads the search's untouched `find_dist`
            // (99,999,999) truncated to a short; no capture has one, so
            // its minimum is skipped.
            if let Some((_, dist)) = found {
                rec.peasant_dist = rec.peasant_dist.min(dist / CELL);
            }
        }
        if land_reg {
            self.ai[w].census.reg_gatherers[r] += 1;
        }
        None
    }

    /// A garrisoned citizen: the oil slot when it is inside an oil
    /// platform. The transport half — a citizen riding a ship with a
    /// building-targeting order — is a seam.
    ///
    /// Two arms, on the outermost container (`plan_strategy@006b9620`, the
    /// citizen branch's `!is_on_map` half): an oil platform fills a slot of
    /// oil; and **a sea-domain unit whose current order is a move** — vslot
    /// `+0x14`, `is_move`, which every `MoveOrder` class answers — makes the
    /// rider an `xport` peasant in the region of the move's point, the cell
    /// under `MoveOrder::x/y` (`reg_xport_peasants[r]++`, `xport_peasants++`,
    /// both only when that cell has a region). Neither arm counts it free,
    /// busy or in `reg_peasants`: a rider is not on the map (`docs/AI.md`
    /// §58).
    fn census_citizen_inside(&mut self, who: Player, u: usize) -> Option<u16> {
        let w = who as usize;
        if let Some(b) = self.units[u].inside {
            if self.buildings[b]
                .ty
                .is_some_and(|t| build::is(&self.build_types, t, build::Ident::OilPlatform))
            {
                self.ai[w].census.filled_gather_slots[5] += 1;
            }
            return None;
        }
        let boat = self.units[u].inside_unit?;
        if self.unit_domain(boat) != Domain::Sea {
            return None;
        }
        let Some(crate::orders::Order {
            body: crate::orders::Body::Move(m),
            ..
        }) = self.units[boat].orders.front()
        else {
            return None;
        };
        let r = self.world.region_of(m.dest.cell())?;
        let cs = &mut self.ai[w].census;
        if let Some(x) = cs.reg_xport_peasants.get_mut(usize::from(r)) {
            *x += 1;
        }
        cs.xport_peasants += 1;
        Some(r)
    }

    /// Step 11, the building census: the gather slots and the defensive
    /// score. The original walks the build list and the wall list, which is
    /// one array here.
    fn census_buildings(&mut self, who: Player) {
        let w = who as usize;
        let mut slots = [0i32; RESOURCES];
        let mut defense = 0;
        let mut reg_defense: Vec<(usize, i32)> = Vec::new();
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.owner != who || !bd.active {
                continue;
            }
            let Some(rec) = bd.ty else {
                continue;
            };
            let bt = &self.build_types[rec];
            let reg = self.world.region_of(bd.pos.cell());
            if bt.has(build::flags::GATHER)
                && let Some(good) = crate::ai_place::gather_good(bt.ident)
            {
                slots[good] += bd.gather_max.unwrap_or(0);
            }
            // A dock is exempt: the guard is the *type's* domain, not the
            // region's.
            if bt.domain() == build::BuildDomain::Water || bt.attack == 0 {
                continue;
            }
            let Some(r) = reg.filter(|r| self.region_is_land(*r)) else {
                continue;
            };
            let n = if build::is_tower(&self.build_types, rec) {
                1
            } else if build::is_fort(&self.build_types, rec) {
                2
            } else {
                0
            };
            defense += n;
            reg_defense.push((r as usize, n));
        }
        let cs = &mut self.ai[w].census;
        // `gather_slots[2]` — wealth — is the one slot the original never
        // writes: no building gathers it, and the store keeps its old value.
        for g in [0, 1, 3, 4, 5] {
            cs.gather_slots[g] = slots[g];
        }
        cs.defense += defense;
        for (r, n) in reg_defense {
            cs.reg_defense[r] += n;
        }
    }

    /// Step 12, the maxima and the rare count.
    fn census_maxima(&mut self, who: Player) {
        let w = who as usize;
        let cities = self.city_num(who);
        let villages = self.village_num(who);
        let cs = &mut self.ai[w].census;
        for g in [0, 1, 3, 4, 5] {
            cs.gather_slots_high[g] = cs.gather_slots_high[g].max(cs.gather_slots[g]);
        }
        cs.peasant_high = cs.peasant_high.max(cs.peasants);
        cs.scholar_high = cs.scholar_high.max(cs.scholars);
        cs.caravan_high = cs.caravan_high.max(cs.caras);
        cs.merchant_high = cs.merchant_high.max(cs.merchants);
        cs.army_high = cs.army_high.max(cs.combat);
        cs.city_high = cs.city_high.max(cities);
        cs.village_high = cs.village_high.max(villages);
        cs.population_high = cs.population_high.max(cs.pop);
        // `resources_controlled = bit_count(rare)`; the rares are a seam,
        // so the mask is empty and the count is zero.
        cs.resources_controlled = 0;
    }

    /// Step 13, the per-city site picture: the circle walk over the city's
    /// own radius.
    fn census_city_sites(&mut self, who: Player) {
        let w = who as usize;
        let world_cells = self.world.cell_count();
        let circle = crate::ai_place::circle();
        for c in self.cities_of(who) {
            let level = self.city_level_of(c);
            let indians = self.nation[w].indians;
            let radius = radius_for_level(&self.tuning, level, indians);
            let k = ((radius + 2) / 4) as usize;
            let city_reg = self.cities[c].reg;
            let centre = self.cities[c].pos.cell();

            {
                let pop = self.cities[c].pop;
                let cs = &mut self.ai[w].census;
                cs.city_pop_high = cs.city_pop_high.max(pop);
            }
            {
                let rec = &mut self.ai[w].city_ai[c];
                rec.ocean = 0;
                rec.land = 0;
                rec.filled = 0;
                rec.dock_tile = 0;
                rec.space = [0; 3];
                rec.ter = [0; RESOURCES];
            }
            let slots = self.city_gather_slot_total(c);
            if let Some(r) = city_reg {
                self.ai[w].census.reg_gather_slots[r as usize] += slots;
            }
            if self.cities[c].no_heal {
                self.ai[w].census.attacked += 1;
                if let Some(r) = city_reg.filter(|r| self.region_is_land(*r)) {
                    self.ai[w].census.reg_attacked[r as usize] += 1;
                }
            }

            let outer = circle.radius.get(k + 1).copied().unwrap_or(0);
            let inner = circle.radius.get(k).copied().unwrap_or(0);
            if outer > 1 {
                for i in 0..outer {
                    let cell = Cell::new(centre.x + circle.x[i], centre.y + circle.y[i]);
                    if !self.world.contains(cell) {
                        continue;
                    }
                    self.census_site_cell(who, c, cell, i, inner, city_reg, world_cells);
                }
            }

            let rec = self.ai[w].city_ai[c];
            let open = rec.land - rec.filled;
            if open < 2 {
                self.ai[w].census.full_cities += 1;
            }
            if let Some(r) = city_reg {
                self.ai[w].census.reg_land[r as usize] += open;
            }
        }
    }

    /// One cell of step 13's circle.
    #[allow(clippy::too_many_arguments)]
    fn census_site_cell(
        &mut self,
        who: Player,
        c: usize,
        cell: Cell,
        i: usize,
        inner: usize,
        city_reg: Option<u16>,
        world_cells: i32,
    ) {
        let w = who as usize;
        let d = self.world.cell_data(cell);
        // Mine or nobody's.
        match self.world.owner(cell) {
            Owner::Player(p) if p != who => return,
            _ => {}
        }
        // A water cell — `land` 1 or 2, and not flagged coastal.
        if d.flags & 0x100 == 0 && (d.land == 1 || d.land == 2) {
            self.ai[w].city_ai[c].ocean += 1;
            if let Some(r) = self.world.region_of(cell) {
                let big =
                    self.world.num_coasts(r) > 1 || self.world.region_size(r) >= world_cells / 10;
                if big && self.ai[w].city_ai[c].dock_tile == 0 && self.is_dock_tile(cell) {
                    self.ai[w].city_ai[c].dock_tile += 1;
                }
            }
            return;
        }
        // Land, inside the inner ring, in the city's own region.
        if i + 1 >= inner || self.world.region_of(cell) != city_reg {
            return;
        }
        if d.flags & 0x70 != 0 {
            self.census_site_gather(who, c, cell);
            return;
        }
        self.ai[w].city_ai[c].land += 1;
        let space = self.check_building_wcoord(who, cell, 0, 0, 1, true);
        if space > 1 {
            for n in 2..=space.min(4) {
                self.ai[w].city_ai[c].space[(n - 2) as usize] += 1;
            }
        }
        if space >= 4 {
            // A wide-open cell is *not* counted filled; its gather value is
            // recorded instead, exactly as an occupied one's is.
            self.census_site_gather(who, c, cell);
            return;
        }
        self.ai[w].city_ai[c].filled += 1;
    }

    /// `World::gather_at` into `CityAi::ter`, per good, as the maximum
    /// over the circle's cells. Step 13 is the `centre_only` caller
    /// (`docs/AI.md` §24), so each cell answers out of its own land class
    /// alone.
    fn census_site_gather(&mut self, who: Player, c: usize, cell: Cell) {
        let amounts = self.world.gather_at(cell, true);
        let rec = &mut self.ai[who as usize].city_ai[c];
        for (t, a) in rec.ter.iter_mut().zip(amounts) {
            *t = (*t).max(a);
        }
    }

    /// Step 14: wars and alliances with every other met computer leader.
    fn census_wars(&mut self, who: Player) {
        let w = who as usize;
        let mut wars = 0;
        let mut allies = 0;
        for i in 0..self.players.len() {
            if i == w || !self.met(who, i) {
                continue;
            }
            if self.diplo_war(w, i) || self.diplo_war(i, w) {
                wars += 1;
            } else if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                allies += 1;
            }
        }
        let cs = &mut self.ai[w].census;
        cs.wars = wars;
        cs.allies = allies;
        cs.active_wars = 0;
        cs.active_wars_with = 0;
    }

    /// Step 15, the per-region strategy word.
    ///
    /// Bit 1 is set when the region is *thin* — every leader's cities there
    /// times fifty do not fill it. When it is not thin and I have a city
    /// there, the word starts at **8** rather than 0; the original's `else
    /// if` arm is the only place that value comes from, and `docs/AI.md`
    /// §2.3 step 15 does not mention it.
    fn census_strategy(&mut self, who: Player) {
        let w = who as usize;
        let players = self.players.len();
        // `world+0x34`, the style's sea class (`World::sea_map`).
        let sea_map = self.world.sea_map();
        let difficulty = self.ai_difficulty();
        let start_res_eight = self.lobby.starting_resources == 8;
        for r in 0..self.world.region_count() {
            if !self.region_is_land(r as u16) {
                continue;
            }
            {
                let cs = &mut self.ai[w].census;
                cs.reg_wars[r] = 0;
                cs.reg_allies[r] = 0;
                cs.reg_neutrals[r] = 0;
                cs.strategy[r] = 0;
            }
            let total: i32 = (0..players)
                .filter(|&i| !self.defeated[i])
                .map(|i| self.reg_city_buildings(i as Player, r as u16))
                .sum();
            let thin = total * 50 < self.world.region_size(r as u16);
            let mine = self.ai[w].census.reg_cities[r];
            if thin {
                self.ai[w].census.strategy[r] = 1;
            } else if mine != 0 {
                self.ai[w].census.strategy[r] = 8;
            } else {
                continue;
            }
            if mine == 0 {
                continue;
            }

            let my_attack = self.ai[w].census.attack;
            let home = self.ai[w].census.home_reg;
            for i in 0..players {
                if i == w || self.defeated[i] || !self.met(who, i) {
                    continue;
                }
                let theirs = self.reg_city_buildings(i as Player, r as u16);
                if theirs == 0 {
                    continue;
                }
                if self.diplo_war(w, i) || self.diplo_war(i, w) {
                    {
                        let cs = &mut self.ai[w].census;
                        cs.active_wars += 1;
                        cs.active_wars_with |= 1 << (i & 0x1f);
                        cs.reg_wars[r] += 1;
                    }
                    // Weaker: my attack is under theirs, and either this is
                    // my home region or they hold fewer cities here than I
                    // do. (The decompile's operands are that way round;
                    // §2.3 has them the other way — see the report.)
                    let weaker =
                        my_attack < self.ai[i].census.attack && (home == r as i32 || theirs < mine);
                    let cs = &mut self.ai[w].census;
                    cs.strategy[r] = if weaker {
                        (cs.strategy[r] & 0xfff5) | 4
                    } else {
                        (cs.strategy[r] & 0xfffb) | 2
                    };
                    cs.strategy[r] = if difficulty < 2 {
                        (cs.strategy[r] & 0xfffd) | 4
                    } else if start_res_eight {
                        (cs.strategy[r] & 0xfffb) | 2
                    } else {
                        cs.strategy[r]
                    };
                } else if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                    self.ai[w].census.reg_allies[r] += 1;
                } else {
                    self.ai[w].census.reg_neutrals[r] += 1;
                }
            }

            let cs = &mut self.ai[w].census;
            if sea_map < 1 {
                cs.strategy[r] &= 0xfff7;
            } else if sea_map == 2 {
                // The class-2 probe wants a region flagged `8` with
                // nobody's city in it — a seam, so it never fires.
            } else if sea_map >= 3 && cs.strategy[r] & 6 == 0 {
                cs.strategy[r] |= 8;
            }
        }
    }

    /// `Leader::check_explore@006bc860`: the explored recount
    /// (`docs/AI.md` §2.1). Its cadence lives in the driver
    /// ([`crate::ai::cadence::explore_due`]), so this is the body only.
    ///
    /// With `EXPLORE_MAP_BONUS` the original answers `reg_size`
    /// (`world+0x2c`) outright; otherwise it walks the **region grid**
    /// (`reg_xs × reg_ys`, `world+0x24`/`+0x28`) and counts the cells whose
    /// sample byte in `seen2` (`world+0x160`, stride `fog_xs`) carries the
    /// leader's bit. That grid is two world cells across each way: `seen2`
    /// is indexed `div_3_table[pos >> 7]`, half a world cell, so `fog_xs`
    /// is twice `xs`, and check_explore's `fog_xs * (4·ry + 3) + 4·rx + 3`
    /// steps four fog cells per region cell.
    ///
    /// There is no visibility here, so the seam answers what
    /// `EXPLORE_MAP_BONUS` would: the whole region grid. The rounding when
    /// the world has an odd extent is not established.
    pub fn check_explore(&mut self, who: Player) {
        let w = who as usize;
        self.ai[w].census.explored = (self.world.width() / 2) * (self.world.height() / 2);
    }
}

impl Sim {
    /// `LeaderData::wonder_mark`'s writers, on every leader: the entry
    /// `Wonders::close_wonder@0073c7e0` clears when a wonder closes, and
    /// the one `Wonders::init_wonder@0073c860` writes when one activates
    /// (`Build::activate`'s wonder arm, after `remove_unbuilt_wonder`).
    ///
    /// - **A close** clears its entry and walks the mark down while the
    ///   entry under it is clear, so the mark is one past the highest
    ///   entry in use.
    /// - **An activation** takes the first clear entry under the mark, or
    ///   the one at it, and raises the mark past it.
    ///
    /// The original writes both inside the objects' pass, so this runs at
    /// that pass's end: a wonder that activates on sim-frame N is on the
    /// dump's block N + 1 and is read by the next frame's
    /// `create_buildings` (`docs/AI.md` §75). It is derived from the
    /// buildings rather than called from the activation, in building
    /// order. A wonder closes when it is no longer an alive, activated
    /// wonder of the leader's; a capture is not read.
    pub fn note_wonders(&mut self) {
        for w in 0..self.ai.len() {
            let who = w as Player;
            let standing: Vec<usize> = (0..self.buildings.len())
                .filter(|&b| {
                    let bd = &self.buildings[b];
                    bd.alive
                        && bd.active
                        && bd.owner == who
                        && bd.ty.is_some_and(|t| self.build_types[t].wonder)
                })
                .collect();
            let cs = &mut self.ai[w].census;
            // `close_wonder`: clear, then walk the mark down.
            for e in &mut cs.wonder_slots {
                if e.is_some_and(|b| !standing.contains(&b)) {
                    *e = None;
                }
            }
            while cs.wonder_mark > 0 && cs.wonder_slots[cs.wonder_mark as usize - 1].is_none() {
                cs.wonder_mark -= 1;
            }
            // `init_wonder`: the first clear entry under the mark, else the
            // mark's own.
            for b in standing {
                if cs.wonder_slots.contains(&Some(b)) {
                    continue;
                }
                let mark = cs.wonder_mark as usize;
                let i = cs.wonder_slots[..mark]
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(mark);
                if cs.wonder_slots.len() <= i {
                    cs.wonder_slots.resize(i + 1, None);
                }
                cs.wonder_slots[i] = Some(b);
                cs.wonder_mark = cs.wonder_mark.max(i as i32 + 1);
            }
        }
    }
}

/// Step 3: the scalars the sweep zeroes. `control`, `pop`, `scouts`, the
/// maxima and the `reg_*` arrays the building lifecycle keeps are all
/// deliberately absent.
fn census_zero_scalars(cs: &mut Census) {
    cs.active = 0;
    cs.combat = 0;
    cs.non_siege = 0;
    cs.sea_combat = 0;
    cs.siege = 0;
    cs.defense = 0;
    cs.attack = 0;
    cs.naval = 0;
    cs.air = 0;
    cs.missile = 0;
    cs.transports = 0;
    cs.fishermen = 0;
    cs.idle_fishermen = 0;
    cs.peasants = 0;
    cs.scholars = 0;
    cs.caras = 0;
    cs.merchants = 0;
    cs.fighters = 0;
    cs.bombers = 0;
    cs.cruise = 0;
    cs.nuke = 0;
    cs.free_peasants = 0;
    cs.xport_peasants = 0;
    cs.gatherers = 0;
    cs.attacked = 0;
    cs.full_cities = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BuildType, Ident, flags};
    use crate::garrison::UnitTraits;
    use crate::orders::{GatherOrder, Order};
    use crate::world::{TILES_PER_CELL, UNITS_PER_TILE, World};
    use crate::{Tuning, Unit, UnitType, cost, economy};

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(
            tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        )
    }

    fn bt(ident: Ident, from: Option<usize>, flag: &str, xs: i32, ys: i32) -> BuildType {
        BuildType {
            ident,
            from,
            x_size: xs,
            y_size: ys,
            flags: flags::parse(flag),
            job_time: 100,
            hits: 500,
            garrison_max: 10,
            price: cost::Price {
                kind: cost::Kind::Building,
                ..cost::Price::free().with_base(economy::Resource::Timber, 1)
            },
            ..BuildType::default()
        }
    }

    struct Fix {
        sim: Sim,
        village: usize,
        farm: usize,
        woodcutter: usize,
        university: usize,
        tower: usize,
        fort: usize,
        dock: usize,
        citizen: usize,
        scholar: usize,
        scout: usize,
    }

    /// A 16×16-cell land world with a sea strip on the right, one AI
    /// player and one human, and the handful of types the census reads.
    fn fix() -> Fix {
        let mut w = World::new(16, 16);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(11, 15));
        w.fill_region(Terrain::Sea, Cell::new(12, 0), Cell::new(15, 15));
        // The sea cells' tiles read as ocean, so `tregion_alt` and the dock
        // scan see water where the regions say water.
        for tx in 48..64 {
            for ty in 0..64 {
                w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
            }
        }
        let mut sim = Sim::new(Tuning::RON, w, 2);
        sim.nation[1].human = false;
        // Room for six cities, so the escrow and thin-region tests can found
        // more than one.
        sim.tech[1].epoch[crate::tech::Line::Civic as usize] = 5;
        for l in &mut sim.ledgers {
            l.bucket = [10_000; economy::RESOURCES];
        }
        let village = sim.add_build_type(bt(Ident::Village, None, "ean", 7, 7));
        let mut farm = bt(Ident::Farm, None, "gda", 4, 4);
        farm.flags |= flags::FLAT;
        let farm = sim.add_build_type(farm);
        let woodcutter = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 3, 3));
        let university = sim.add_build_type(bt(Ident::University, None, "ga", 4, 4));
        let mut tower = bt(Ident::Tower, None, "ean", 2, 2);
        tower.attack = 12;
        let tower = sim.add_build_type(tower);
        let mut fort = bt(Ident::Fort, None, "ean", 5, 5);
        fort.attack = 20;
        let fort = sim.add_build_type(fort);
        let mut dock = bt(Ident::Dock, None, "eab", 4, 4);
        dock.attack = 8;
        let dock = sim.add_build_type(dock);

        let base = UnitType {
            price: cost::Price {
                pop: 1,
                ..cost::Price::free()
            },
            hits: 40,
            garrison: UnitTraits {
                trained_at: Some(village),
                ..UnitTraits::default()
            },
            ..UnitType::default()
        };
        let citizen = sim.add_unit_type(UnitType {
            worker: Worker::Citizen,
            ..base.clone()
        });
        let scholar = sim.add_unit_type(UnitType {
            worker: Worker::Scholar,
            ..base.clone()
        });
        let scout = sim.add_unit_type(base);
        Fix {
            sim,
            village,
            farm,
            woodcutter,
            university,
            tower,
            fort,
            dock,
            citizen,
            scholar,
            scout,
        }
    }

    /// Step 9 (`docs/AI.md` §55): every good in `new_rares` counts into its
    /// cell's region unless the cell is another leader's and that leader is
    /// not a mutual ally — and only the census writes the array, while the
    /// leader-level sum waits for `calc_gather`'s recompute.
    #[test]
    fn step_9_counts_the_seen_rares_a_merchant_may_reach() {
        let mut f = fix();
        let s = &mut f.sim;
        let good = |s: &mut Sim, x: i32, y: i32| {
            s.world.add_good(crate::world::Good {
                pos: tile_pos(x * TILES_PER_CELL, y * TILES_PER_CELL),
                ty: 20,
                alive: true,
            })
        };
        let unowned = good(s, 2, 2);
        let mine = good(s, 4, 4);
        let theirs = good(s, 6, 6);
        s.world
            .set_owner(Cell::new(4, 4), Owner::Player(1), Owner::None);
        s.world
            .set_owner(Cell::new(6, 6), Owner::Player(0), Owner::None);
        s.ai[1].new_rares = vec![unowned, mine, theirs];
        let land = s.world.region_of(Cell::new(2, 2)).expect("land") as usize;

        s.census(1);
        assert_eq!(
            s.ai[1].census.reg_known_rares[land], 2,
            "unowned and own ground count; the human's does not"
        );
        assert_eq!(
            s.ai[1].known_rares, 0,
            "the census writes the array, not the sum"
        );
        s.assemble_holdings(1);
        assert_eq!(s.ai[1].known_rares, 2, "calc_gather's recompute sums it");

        // A mutual ally's ground counts; a one-sided alliance does not.
        s.allied[1][0] = true;
        s.census(1);
        assert_eq!(
            s.ai[1].census.reg_known_rares[land], 2,
            "one side is not an ally"
        );
        s.allied[0][1] = true;
        s.census(1);
        assert_eq!(s.ai[1].census.reg_known_rares[land], 3, "both sides are");

        // And the sweep zeroes before it counts, so a list that shrank
        // would not leave a stale count behind.
        s.ai[1].new_rares.clear();
        s.census(1);
        assert_eq!(s.ai[1].census.reg_known_rares[land], 0);
    }

    /// A border fix between the census and `calc_gather`'s recompute
    /// zeroes the array, so the recompute sums 0 and the Merchant arm is
    /// shut until the next sweep counts again — East Indies 18182's make
    /// list (`docs/AI.md` §76). `Region::fix_borders` →
    /// `check_borders` → `compute_reg_territory:76–105`.
    #[test]
    fn a_border_fix_zeroes_the_rares_until_the_next_census() {
        let mut f = fix();
        let s = &mut f.sim;
        let g = s.world.add_good(crate::world::Good {
            pos: tile_pos(2 * TILES_PER_CELL, 2 * TILES_PER_CELL),
            ty: 20,
            alive: true,
        });
        s.ai[1].new_rares = vec![g];
        let land = s.world.region_of(Cell::new(2, 2)).expect("land") as usize;
        s.census(1);
        assert_eq!(s.ai[1].census.reg_known_rares[land], 1, "counted");
        s.sync_territory();
        assert_eq!(
            s.ai[1].census.reg_known_rares[land], 1,
            "the fix only marks the regions"
        );
        s.check_borders();
        assert_eq!(
            s.ai[1].census.reg_known_rares[land], 0,
            "the next check_borders zeroes every region"
        );
        s.assemble_holdings(1);
        assert_eq!(s.ai[1].known_rares, 0, "the recompute sums the zero");
        s.census(1);
        s.assemble_holdings(1);
        assert_eq!(s.ai[1].known_rares, 1, "the next sweep counts it again");
    }

    fn finish(sim: &mut Sim, b: usize) {
        let mut guard = 0;
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
            guard += 1;
            assert!(guard < 10);
        }
        sim.buildings[b].helpers = 0;
    }

    fn build(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
        let b = sim
            .place_building(who, ty, tile_pos(tx, ty_))
            .unwrap_or_else(|e| panic!("placing at ({tx},{ty_}) should work: {e:?}"));
        finish(sim, b);
        b
    }

    /// A finished building placed straight into the world, past the
    /// placement rules — the only way to stand a dock on this test map.
    fn force_build(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
        let b = sim.init_build(who, ty, tile_pos(tx, ty_), false);
        sim.activate(b, false, true);
        b
    }

    fn spawn(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = Unit::new(who, index, tile_pos(tx, ty_), hits);
        u.ty = Some(ty);
        sim.add_unit(u)
    }

    fn gather_order(building: usize) -> Order {
        Order {
            flags: crate::orders::flag::ACTION,
            body: Body::Gather(GatherOrder {
                building,
                tile: None,
                wait: 0,
                goto_build: false,
                dist_mod: 0,
                been_there: true,
            }),
        }
    }

    /// The AI leader's city, five citizens on farms and woodcutters, one
    /// scout — run8's shape. The free/busy split is by the **action**'s
    /// kind: no action or EXPLORE_TO is free, everything else is busy, and
    /// a GATHER moves the count on to the target building's city.
    /// **A citizen on a barge is off the map** (item 592, `docs/AI.md` §58):
    /// neither free nor busy nor in `reg_peasants`, and — while the barge's
    /// order is a move — an `xport` peasant in the region of the move's
    /// point, which is also where the `reg_active` family counts it, because
    /// the original keeps that region in the local the unit's own region
    /// lives in. run143's 10576 is the case: `1/31` rides `1/36` and the
    /// original counts it in `reg_active[5]` and `reg_xport_peasants[5]`.
    #[test]
    fn a_rider_is_an_xport_peasant_where_its_boat_is_going() {
        let mut f = fix();
        // Two land regions: the boarding shore and the far shore.
        let home = f.sim.world.region_of(Cell::new(2, 2)).unwrap();
        let far = f
            .sim
            .world
            .fill_region(Terrain::Land, Cell::new(0, 10), Cell::new(11, 15));
        let sea = f.sim.world.region_of(Cell::new(13, 4)).unwrap();
        // A city at home, so the citizen ashore is not in transport itself.
        build(&mut f.sim, 1, f.village, 20, 20);
        let mut barge = UnitType {
            hits: 50,
            ..UnitType::default()
        };
        barge.combat.domain = Domain::Sea;
        let barge = f.sim.add_unit_type(barge);
        let boat = spawn(
            &mut f.sim,
            1,
            barge,
            13 * TILES_PER_CELL,
            4 * TILES_PER_CELL,
        );
        let rider = spawn(
            &mut f.sim,
            1,
            f.citizen,
            2 * TILES_PER_CELL,
            2 * TILES_PER_CELL,
        );
        spawn(
            &mut f.sim,
            1,
            f.citizen,
            3 * TILES_PER_CELL,
            3 * TILES_PER_CELL,
        );
        f.sim.units[rider].inside_unit = Some(boat);
        f.sim.units[rider].on_map = false;
        let sail = |dest: Pos| Order {
            flags: crate::orders::flag::ACTION,
            body: Body::Move(crate::orders::MoveOrder {
                kind: crate::orders::MoveKind::MoveTo,
                dest,
                angle: crate::movement::Angle(0),
                facing: None,
                has_waypoint: false,
                waypoint: dest,
                last: None,
                pause: 0,
                retry: 0,
                attempts: 0,
                timer: 0,
                coll: None,
                orig: None,
                group: None,
            }),
        };
        f.sim.units[boat]
            .orders
            .push_back(sail(tile_pos(4 * TILES_PER_CELL, 12 * TILES_PER_CELL)));

        f.sim.census(1);
        let cs = &f.sim.ai[1].census;
        let reg = |v: &[i32], r: u16| Census::reg(v, r);
        assert_eq!(cs.peasants, 2, "the rider is a peasant");
        assert_eq!(cs.free_peasants, 1, "only the one ashore is free");
        assert_eq!(cs.xport_peasants, 1, "the rider is in transport");
        assert_eq!(reg(&cs.reg_xport_peasants, far), 1, "where the boat goes");
        assert_eq!(reg(&cs.reg_peasants, home), 1, "a rider is not on the map");
        assert_eq!(reg(&cs.reg_free_peasants, home), 1);
        assert_eq!(
            (reg(&cs.reg_active, home), reg(&cs.reg_active, far)),
            (1, 1),
            "the rider's `reg_active` is the move point's region"
        );

        // A boat at rest carries no `xport`: the rider counts in the boat's
        // own tile's region, the sea, where a land unit is not active.
        f.sim.units[boat].orders.clear();
        f.sim.census(1);
        let cs = &f.sim.ai[1].census;
        assert_eq!((cs.free_peasants, cs.xport_peasants), (1, 0));
        assert_eq!(reg(&cs.reg_active, far), 0);
        assert_eq!(reg(&cs.reg_active, sea), 0);
        assert_eq!(reg(&cs.reg_active, home), 1, "only the one ashore");
    }

    #[test]
    fn the_free_busy_split_follows_the_action_kind() {
        let mut f = fix();
        let city_b = build(&mut f.sim, 1, f.village, 20, 20);
        let c = f.sim.buildings[city_b].city.expect("a city record");
        let farm_b = build(&mut f.sim, 1, f.farm, 26, 20);
        f.sim.buildings[farm_b].gather_max = Some(3);
        f.sim.plant_camp_forest(tile_pos(20, 26));
        let wood_b = build(&mut f.sim, 1, f.woodcutter, 20, 26);
        f.sim.buildings[wood_b].gather_max = Some(5);

        // Three gathering on the farm, two on the woodcutter.
        for k in 0..3 {
            let u = spawn(&mut f.sim, 1, f.citizen, 26 + k, 21);
            f.sim.units[u].orders.push_back(gather_order(farm_b));
        }
        for k in 0..2 {
            let u = spawn(&mut f.sim, 1, f.citizen, 21 + k, 26);
            f.sim.units[u].orders.push_back(gather_order(wood_b));
        }
        // One idle citizen and a scout that is exploring.
        spawn(&mut f.sim, 1, f.citizen, 21, 21);
        let s = spawn(&mut f.sim, 1, f.scout, 22, 22);
        f.sim.units[s].orders.push_back(Order {
            flags: crate::orders::flag::ACTION,
            body: Body::Move(crate::orders::MoveOrder {
                kind: crate::orders::MoveKind::ExploreTo,
                dest: tile_pos(40, 40),
                angle: crate::movement::Angle(0),
                facing: None,
                has_waypoint: false,
                waypoint: tile_pos(40, 40),
                last: None,
                pause: 0,
                retry: 0,
                attempts: 0,
                timer: 0,
                coll: None,
                orig: None,
                group: None,
            }),
        });

        f.sim.census(1);
        let cs = &f.sim.ai[1].census;
        assert_eq!(cs.peasants, 6, "six citizens, the scout is not one");
        assert_eq!(cs.gatherers, 5);
        assert_eq!(cs.free_peasants, 1, "only the idle citizen is free");
        assert_eq!(cs.active, 7);
        // Five gatherers, all working buildings of this city, so the
        // `busy--`/`busy++` pair puts them all back on the same record.
        let rec = f.sim.ai[1].city_ai[c];
        assert_eq!(rec.free, 1);
        assert_eq!(rec.busy, 5);
        assert_eq!(rec.gatherers, 5);
        assert!(rec.peasant_dist < PEASANT_DIST_NONE, "a citizen was found");
    }

    /// **A gatherer counts in its building's city, not its nearest**
    /// (item 752, `docs/AI.md` §73). `plan_strategy` overwrites the found
    /// city with the target building's at `6babfb`, so `gatherers` and
    /// `peasant_dist` follow `busy` there. run227's woodcutters `1/50` and
    /// `1/61` stand nearer London and work a camp of who=1's second city:
    /// the original counts them in the second, 10 and 12 against this
    /// crate's 11 and 11, and the crossing rule in `find_gather_spot` read
    /// the difference on East Indies 16594.
    #[test]
    fn a_gatherer_counts_in_its_building_s_city_not_the_nearest() {
        let mut f = fix();
        let near_b = build(&mut f.sim, 1, f.village, 20, 20);
        let near = f.sim.buildings[near_b].city.expect("a city record");
        let far_b = build(&mut f.sim, 1, f.village, 44, 20);
        let far = f.sim.buildings[far_b].city.expect("a city record");
        f.sim.plant_camp_forest(tile_pos(30, 26));
        let wood_b = build(&mut f.sim, 1, f.woodcutter, 30, 26);
        f.sim.buildings[wood_b].gather_max = Some(5);
        f.sim.buildings[wood_b].city = Some(far);
        // Nearer the first city, working the second's camp.
        let u = spawn(&mut f.sim, 1, f.citizen, 24, 22);
        f.sim.units[u].orders.push_back(gather_order(wood_b));

        f.sim.census(1);
        let (n, fa) = (f.sim.ai[1].city_ai[near], f.sim.ai[1].city_ai[far]);
        assert_eq!((n.busy, n.gatherers), (0, 0), "not the nearest city");
        assert_eq!((fa.busy, fa.gatherers), (1, 1), "the building's city");
        assert!(fa.peasant_dist < PEASANT_DIST_NONE, "its distance goes too");
        assert_eq!(n.peasant_dist, PEASANT_DIST_NONE);
    }

    /// `gather_slots` comes off the buildings, `filled_gather_slots` off
    /// the citizens working them, and each is keyed by the *building's*
    /// type: farm food, woodcutter timber, university knowledge (which only
    /// a garrisoned scholar fills), mine metal, oil.
    #[test]
    fn the_gather_slot_books_are_kept_by_building_type() {
        let mut f = fix();
        let city_b = build(&mut f.sim, 1, f.village, 20, 20);
        let c = f.sim.buildings[city_b].city.unwrap();
        let farm_b = build(&mut f.sim, 1, f.farm, 26, 20);
        f.sim.buildings[farm_b].gather_max = Some(3);
        f.sim.plant_camp_forest(tile_pos(20, 26));
        let wood_b = build(&mut f.sim, 1, f.woodcutter, 20, 26);
        f.sim.buildings[wood_b].gather_max = Some(5);
        let uni_b = build(&mut f.sim, 1, f.university, 26, 26);
        f.sim.buildings[uni_b].gather_max = Some(4);

        for _ in 0..3 {
            let u = spawn(&mut f.sim, 1, f.citizen, 26, 21);
            f.sim.units[u].orders.push_back(gather_order(farm_b));
        }
        for _ in 0..2 {
            let u = spawn(&mut f.sim, 1, f.citizen, 21, 26);
            f.sim.units[u].orders.push_back(gather_order(wood_b));
        }
        // A scholar inside the university fills the knowledge slot.
        let sch = spawn(&mut f.sim, 1, f.scholar, 26, 26);
        f.sim.units[sch].inside = Some(uni_b);
        f.sim.units[sch].on_map = false;

        f.sim.census(1);
        let cs = &f.sim.ai[1].census;
        assert_eq!(cs.gather_slots[0], 3, "the farm");
        assert_eq!(cs.gather_slots[1], 5, "the woodcutter");
        assert_eq!(cs.gather_slots[3], 4, "the university");
        assert_eq!(cs.gather_slots[2], 0, "wealth is never written");
        assert_eq!(cs.filled_gather_slots[0], 3);
        assert_eq!(cs.filled_gather_slots[1], 2);
        assert_eq!(cs.filled_gather_slots[3], 1, "the garrisoned scholar");
        assert_eq!(cs.scholars, 1);
        // `count_gather_slots` leaves knowledge out of its total, so the
        // region sees 3 + 5 and not 3 + 5 + 4 — run8's `reg_gather_slots 8`.
        let r = f.sim.cities[c].reg.unwrap() as usize;
        assert_eq!(cs.reg_gather_slots[r], 8);
    }

    /// A ship garrisoned in a dock counts in the **sea** region behind the
    /// dock, not in the land region the dock stands in; a citizen in the
    /// same dock counts in the land region.
    #[test]
    fn a_garrisoned_unit_takes_its_dock_s_sea_region() {
        let mut f = fix();
        // The dock stands on the land/sea border: cells 11 (land) and 12
        // (sea) meet at tile 48, and its 4×4 footprint reaches tile 48.
        let dock_b = force_build(&mut f.sim, 1, f.dock, 47, 20);
        let land = f.sim.world.region_of(Cell::new(11, 5)).unwrap();
        let sea = f.sim.world.region_of(Cell::new(12, 5)).unwrap();
        assert_ne!(land, sea);

        let ship = spawn(&mut f.sim, 1, f.scout, 47, 20);
        let sty = f.sim.units[ship].ty.unwrap();
        f.sim.unit_types[sty].combat.domain = Domain::Sea;
        f.sim.units[ship].inside = Some(dock_b);
        f.sim.units[ship].on_map = false;

        assert_eq!(f.sim.census_unit_region(ship), Some(sea));

        let cit = spawn(&mut f.sim, 1, f.citizen, 47, 20);
        f.sim.units[cit].inside = Some(dock_b);
        f.sim.units[cit].on_map = false;
        assert_eq!(f.sim.census_unit_region(cit), Some(land));
    }

    /// A tower counts 1 for defence, a fort 2, and a dock — whose type has
    /// a water domain — counts nothing even though it has `ATTACK`.
    #[test]
    fn defence_counts_towers_and_forts_and_exempts_the_dock() {
        let mut f = fix();
        build(&mut f.sim, 1, f.village, 20, 20);
        build(&mut f.sim, 1, f.tower, 28, 20);
        build(&mut f.sim, 1, f.fort, 20, 28);
        force_build(&mut f.sim, 1, f.dock, 47, 20);
        f.sim.census(1);
        assert_eq!(f.sim.ai[1].census.defense, 3);
        let r = f.sim.world.region_of(Cell::new(5, 5)).unwrap() as usize;
        assert_eq!(f.sim.ai[1].census.reg_defense[r], 3);
    }

    /// The maxima only ever climb, and the escrow rate switches on at the
    /// *third* city and stays on.
    #[test]
    fn the_maxima_climb_and_the_escrow_rate_latches() {
        let mut f = fix();
        build(&mut f.sim, 1, f.village, 20, 20);
        for _ in 0..4 {
            spawn(&mut f.sim, 1, f.citizen, 21, 21);
        }
        f.sim.census(1);
        assert_eq!(f.sim.ai[1].census.peasants, 4);
        assert_eq!(f.sim.ai[1].census.peasant_high, 4);
        assert_eq!(f.sim.ai[1].census.escrow_rate, [0; RESOURCES], "one city");

        // Kill three citizens: the count falls, the high-water mark does not.
        for u in 0..f.sim.units.len() {
            if f.sim.units[u].owner == 1 && f.sim.units[u].health > 0 && u < 3 {
                f.sim.units[u].health = 0;
            }
        }
        build(&mut f.sim, 1, f.village, 40, 20);
        build(&mut f.sim, 1, f.village, 20, 40);
        f.sim.census(1);
        assert_eq!(f.sim.ai[1].census.peasants, 1);
        assert_eq!(f.sim.ai[1].census.peasant_high, 4);
        assert_eq!(f.sim.ai[1].census.city_high, 3);
        assert_eq!(
            f.sim.ai[1].census.escrow_rate, [ESCROW_RATE; RESOURCES],
            "three cities is more than two"
        );
    }

    /// A region nobody has filled is *thin* — bit 1. With a city in a
    /// region that is not thin the word starts at 8 instead, which is the
    /// original's `else if` arm and not in `docs/AI.md` §2.3.
    #[test]
    fn the_thin_bit_and_the_eight_that_replaces_it() {
        let mut f = fix();
        // The eight survives step 15's tail only on a map whose sea class
        // is 1 (Great Lakes' — `world+0x34`, `World::sea_map`): class 0,
        // the land maps, clears it; 3 and up set it again for a region
        // with neither `2` nor `4`. Run9's lobby is class 1.
        f.sim.world.set_sea_map(1);
        build(&mut f.sim, 1, f.village, 20, 20);
        f.sim.census(1);
        let r = f.sim.world.region_of(Cell::new(5, 5)).unwrap() as usize;
        // 12 × 16 = 192 cells, one city: 50 < 192, so thin.
        assert_eq!(f.sim.ai[1].census.strategy[r], 1);
        assert_eq!(f.sim.ai[1].census.reg_cities[r], 1);

        // Four cities fill it: 4 × 50 = 200 ≥ 192.
        build(&mut f.sim, 1, f.village, 40, 20);
        build(&mut f.sim, 1, f.village, 20, 40);
        build(&mut f.sim, 1, f.village, 40, 40);
        f.sim.census(1);
        assert_eq!(f.sim.ai[1].census.strategy[r], 8);
    }

    /// A city's circle walk fills `land`, `filled` and `space`, and
    /// `reg_land` takes the open remainder; `full_cities` counts a city
    /// with fewer than two open cells.
    #[test]
    fn the_site_picture_counts_the_open_cells_of_the_circle() {
        let mut f = fix();
        let b = build(&mut f.sim, 1, f.village, 20, 20);
        let c = f.sim.buildings[b].city.unwrap();
        f.sim.census(1);
        let rec = f.sim.ai[1].city_ai[c];
        let r = f.sim.cities[c].reg.unwrap() as usize;
        // The radius is the city's own `get_radius` — 20 tiles for a level
        // one city — so `(20 + 2) / 4 = 5` picks `circle_radius[5]`, and
        // the inner cutoff is `circle_radius[5]` too... one ring in.
        let circle = crate::ai_place::circle();
        assert!(rec.land > 0, "an empty map is all open");
        assert!(
            (rec.land as usize) < circle.radius[5],
            "the walk stops at the inner ring"
        );
        assert_eq!(rec.ocean, 0, "no water in reach");
        assert_eq!(
            f.sim.ai[1].census.reg_land[r],
            rec.land - rec.filled,
            "the region takes the open remainder"
        );
        assert_eq!(f.sim.ai[1].census.full_cities, 0);
    }

    /// `check_explore` answers the whole **region** grid — two world cells
    /// across each way — because there is no fog here.
    #[test]
    fn check_explore_counts_the_whole_region_grid() {
        let mut f = fix();
        f.sim.check_explore(1);
        assert_eq!(
            f.sim.ai[1].census.explored,
            8 * 8,
            "a 16x16 cell world is 8x8 region cells"
        );
        assert_eq!(TILES_PER_CELL, 4);
    }

    /// **The sweep reads `UnitTypeData::role`, over captains** — item
    /// 295, and both halves failed on purpose before they landed.
    ///
    /// The role word the producers read (`ai_load::role`) and this crate's
    /// own profile bitfield (`combat::role`, `docs/DECISIONS.md` entry 18)
    /// collide twice: `1 << 16` is `MILITARY` in one and `CARAVAN` in the
    /// other, `1 << 4` is the scout bit and `V2ROCKET`. The sweep read the
    /// wrong one, so on run91's Great Lakes the leader's caravan was its
    /// only soldier and nine Longbowmen and three Hoplites were civilians.
    /// And it walked every live object where the original walks captains
    /// (`docs/AI.md` §2.3 step 10), so the twelve of those that stand in
    /// four squads counted twelve rather than four.
    ///
    /// Here: a three-figure squad of a military type and a lone caravan.
    /// `active` is 2 and not 4, `combat` is 1 and not 3, and the caravan
    /// is on neither count. **`attack` is the half that separates the two
    /// faults**: under the profile word the caravan took the military
    /// branch and `combat` read 1 for the wrong unit, with a zero attack
    /// where the soldier's is 15.
    #[test]
    fn the_census_counts_captains_under_the_original_s_role_word() {
        let mut f = fix();
        build(&mut f.sim, 1, f.village, 20, 20);
        f.sim.ai[1].census.resize(f.sim.world.region_count(), 2);

        // A military type by the original's word, and a caravan by this
        // crate's — whose `1 << 16` the sweep used to read as military.
        let soldier = f.sim.unit_types[f.scout].clone();
        let soldier = f.sim.add_unit_type(soldier);
        f.sim.unit_types[soldier].cols.role = ROLE_MILITARY;
        f.sim.unit_types[soldier].combat.attack = 150;
        let trader = f.sim.unit_types[f.scout].clone();
        let trader = f.sim.add_unit_type(trader);
        f.sim.unit_types[trader].combat.roles = crate::combat::role::CARAVAN;

        let squad: Vec<usize> = (0..3)
            .map(|i| spawn(&mut f.sim, 1, soldier, 20 + i, 22))
            .collect();
        for w in squad.windows(2) {
            f.sim.units[w[1]].captain = false;
            f.sim.units[w[1]].o_up = Some(w[0]);
            f.sim.units[w[0]].o_down = Some(w[1]);
        }
        spawn(&mut f.sim, 1, trader, 24, 22);

        f.sim.census(1);
        let c = &f.sim.ai[1].census;
        assert_eq!(c.active, 2, "the squad's followers are counted as units");
        assert_eq!(c.combat, 1, "one squad is one soldier");
        assert_eq!(c.non_siege, 1);
        assert_eq!(c.attack, 15, "`attack() / 10`, the captain's alone");
    }

    /// **A decoy is no unit of the census** (item 1302, `docs/AI.md`
    /// §99.14; `plan_strategy`'s `testb $0x1, 0x68(%esi)` at `6b9f57`):
    /// run346's six decoy squads of 11637 stay off `non_siege`, and
    /// `create_units`' military gate on 11780 reads the count without them.
    ///
    /// Made to fail with the decoy test dropped.
    #[test]
    fn a_decoy_is_no_unit_of_the_census() {
        let mut f = fix();
        build(&mut f.sim, 1, f.village, 20, 20);
        f.sim.ai[1].census.resize(f.sim.world.region_count(), 2);
        let soldier = f.sim.unit_types[f.scout].clone();
        let soldier = f.sim.add_unit_type(soldier);
        f.sim.unit_types[soldier].cols.role = ROLE_MILITARY;
        f.sim.unit_types[soldier].combat.attack = 150;
        spawn(&mut f.sim, 1, soldier, 20, 22);
        let copy = spawn(&mut f.sim, 1, soldier, 22, 22);
        f.sim.units[copy].decoy = true;
        f.sim.census(1);
        let c = &f.sim.ai[1].census;
        assert_eq!((c.active, c.combat, c.non_siege), (1, 1, 1));
    }

    /// A military unit of mine standing on another leader's land shows up
    /// in *their* `invaders[me]`, and the count is cleared each sweep.
    #[test]
    fn invaders_land_on_the_owner_s_record() {
        let mut f = fix();
        build(&mut f.sim, 1, f.village, 20, 20);
        // Player 0 owns a patch of cells.
        for x in 8..10 {
            for y in 8..10 {
                f.sim
                    .world
                    .set_owner(Cell::new(x, y), Owner::Player(0), Owner::None);
            }
        }
        let u = spawn(&mut f.sim, 1, f.scout, 34, 34);
        let ty = f.sim.units[u].ty.unwrap();
        f.sim.unit_types[ty].cols.role = ROLE_MILITARY;
        f.sim.ai[0].census.resize(f.sim.world.region_count(), 2);

        f.sim.census(1);
        assert_eq!(f.sim.ai[0].census.invaders[1], 1);
        assert_eq!(f.sim.ai[1].census.combat, 1);

        // Move it home: the next sweep clears the old count.
        f.sim.units[u].pos = tile_pos(20, 22);
        f.sim.census(1);
        assert_eq!(f.sim.ai[0].census.invaders[1], 0);
    }
}
