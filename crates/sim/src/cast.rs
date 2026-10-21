//! **The targeted cast** — a player's craft on an object: `Group::
//! action_spell@006fe1a0`'s default arm, which lays the `CastOrder`, and
//! `Unit::do_cast@005ebfe0`'s targeted half, which walks the caster into
//! range, holds it for the craft's job time and casts (`docs/GOLDEN.md`
//! §27, `docs/ORDERS.md` §37).
//!
//! The one craft a capture has reached is the Spy's **Informer** (`0x27f`)
//! on an enemy building, run245. Its price is the mana alone: run246's
//! packet has `Type::pay_cost` rewrite every bucket unchanged although the
//! row prints `COST 20g/20w`, so no price is taken here (a stated seam, the
//! reason not established). The other crafts' arms of `is_valid_target`,
//! `SpellType::cast` and the animation are refused or stated rather than
//! guessed.

use crate::combat::Obj;
use crate::group::Group;
use crate::orders::{Body, CastOrder, MoveKind, Order, QueuePos, flag, spell};
use crate::{Player, Pos, Sim};

/// `TypeIndex` rows the three untargeted crafts name (`enums/TypeIndex.txt`).
pub(crate) const PEASANTS: i32 = 0x32;
pub(crate) const MILITIA: i32 = 0x42;
const MINUTEMAN: i32 = 0x43;
const PARTISAN: i32 = 0x44;
/// `TypeIndex` rows [`Sim::craft_rate`] asks (`enums/TypeIndex.txt`).
const GENERAL: crate::tech::TypeId = 0x36;
const SPY: crate::tech::TypeId = 0x3a;
const MEMNON: crate::tech::TypeId = 0x16c;

/// `SpellTypeData::spell_flags` letters the cast reads, as bits `a`..`m`.
pub mod craft {
    /// `b`, `c`, `d`: a unit, a building, an area — the targeted craft.
    pub const TARGETED: u32 = 0xe;
    /// `b`: a unit may be the target.
    pub const UNIT: u32 = 0x2;
    /// `c`: a building may be the target.
    pub const BUILDING: u32 = 0x4;
    /// `d`: the target is a point, not an object.
    pub const AREA: u32 = 0x8;
    /// `e`: one's own object may be the target.
    pub const OWN: u32 = 0x10;
    /// `f`: an enemy's.
    pub const ENEMY: u32 = 0x20;
    /// `g`: `action_spell` kills the caster's head casts and queues first.
    pub const FIRST: u32 = 0x40;
    /// `h`: an ally's.
    pub const ALLY: u32 = 0x80;
    /// `k`: a peace partner's.
    pub const PEACE: u32 = 0x400;
    /// `l`: the cast keeps a cloaked caster cloaked.
    pub const KEEP_CLOAK: u32 = 0x800;
    /// `m`: `action_spell` hands the cast to the member with the most
    /// `mana_left`.
    pub const BEST_MANA: u32 = 0x1000;
}

/// `UnitData::decoy_time@00609d70` — what a decoy has left to live:
/// `(general_upgrade + 2) × DECOY_TIME / 2 − mana_burn`, the halving
/// truncated toward zero (`cdq; sub; sar`) and a negative floored at 0
/// (`cmovs`). `Unit::process` closes the decoy at 0 (item 2072).
#[must_use]
pub(crate) const fn decoy_time_left(general_upgrade: i32, decoy_time: i32, mana_burn: i16) -> i32 {
    let left = (general_upgrade + 2) * decoy_time / 2 - mana_burn as i32;
    if left < 0 { 0 } else { left }
}

impl Sim {
    /// `UnitData::mana@00609a50` — the type's `MANA`, and a French
    /// General's `(FRENCH_SPECIAL_CRAFT + 100) × MANA / 100` (`609ad7`..
    /// `609b1c`, `has_tribe_bonus(10)` and `is(GENERAL, 1)`; the bonus
    /// ships as 0%). SEAM: the supply upgrade's multiple and the space-air
    /// arm; neither is reached by a Spy or a General.
    pub(crate) fn unit_mana(&self, u: usize) -> i32 {
        let mana = self.units[u].ty.map_or(0, |t| self.unit_types[t].mana);
        if mana == 0 {
            return 0;
        }
        let who = self.units[u].owner as usize;
        let french_general = self.tech_tree.has_tribe_bonus(
            &self.setup,
            &self.tech[who],
            crate::nations::power::FRENCH,
        ) && self.tech_tree.types.get(GENERAL).is_some()
            && self
                .unit_tree(u)
                .is_some_and(|t| self.tech_tree.is(t, GENERAL, true));
        if french_general {
            (self.tuning.french_special_craft + 100) * mana / 100
        } else {
            mana
        }
    }

    /// `Unit::process@00610bc0`'s decoy test: on the map, and some bit of
    /// `seen3` (`World +0x164`, the inner disc of a `DETECT` object) at the
    /// decoy's fog cell outside its owner's ally mask (`LeaderData
    /// +0x6929`) — an enemy detector has it in its disc.
    pub(crate) fn decoy_detected(&self, u: usize) -> bool {
        let un = &self.units[u];
        if !un.on_map {
            return false;
        }
        let fog = crate::vision::UNITS_PER_FOG;
        self.world.seen3(un.pos.x / fog, un.pos.y / fog) & !self.seen_ally_mask(un.owner) != 0
    }

    /// `UnitData::mana_left@00609a30`: `max(0, mana − mana_burn)`.
    pub(crate) fn mana_left(&self, u: usize) -> i32 {
        (self.unit_mana(u) - i32::from(self.units[u].mana_burn)).max(0)
    }

    /// `ObjectData::infiltrated` (`+0x3a`) of either kind of object.
    pub(crate) fn infiltrated_of(&self, o: Obj) -> u8 {
        match o {
            Obj::Unit(u) => self.units[u].infiltrated,
            Obj::Building(b) => self.buildings[b].infiltrated,
        }
    }

    /// `SpellTypeData::is_castable(spell, o, who, ·) != 0` for the crafts a
    /// player's unit can be handed — the head's lineage test: the caster
    /// `is` the row's `FROM` or `FROM2`.
    ///
    /// Past the head, `@00675d19`: a unit that is a decoy (`unit_masks & 1`)
    /// casts nothing but a pack or an unpack. Then the craft's own case;
    /// the ones this crate carries:
    ///
    /// ```text
    /// TO_ARMS (0x294): the caster on the map; the player holding MILITIA,
    ///   MINUTEMAN or PARTISAN (`has_tech`: the preq and the type's own
    ///   bit, which `library` does not set — run424); and the cell under
    ///   it no one's, or the caster's, or an ally's
    /// anything else this crate issues:            3
    /// ```
    ///
    /// SEAM: the tribe and graft substitutions (the head's and To Arms'
    /// `tribe_can_type`), and every other craft's own case; the Spy's three
    /// and the General's and the Militia's have none.
    pub(crate) fn spell_castable(&self, s: i32, u: usize) -> bool {
        let Some(d) = self.spell(s) else {
            return false;
        };
        if !d.from.iter().flatten().any(|&t| self.unit_line_is(u, t)) {
            return false;
        }
        if self.units[u].decoy && !spell::is_pack(s) && !spell::is_unpack(s) {
            return false;
        }
        match s {
            spell::TO_ARMS => self.to_arms_castable(u),
            _ => true,
        }
    }

    /// `is_castable`'s `0x294` case (`00675fd1`..`0067607c`).
    fn to_arms_castable(&self, u: usize) -> bool {
        let unit = &self.units[u];
        if !unit.on_map {
            return false;
        }
        let who = unit.owner;
        let had = [MILITIA, MINUTEMAN, PARTISAN].iter().any(|&ti| {
            self.unit_rec_of_index(ti)
                .and_then(|r| self.unit_types[r].tree)
                .is_some_and(|t| {
                    self.tech_tree
                        .has_tech(&self.setup, &self.tech[who as usize], t)
                })
        });
        if !had {
            return false;
        }
        match self.world.owner_at(unit.pos) {
            crate::world::Owner::Player(p) => p == who || self.is_ally(who, p),
            _ => true,
        }
    }

    /// The leader's own bit for the unit type `ti` (`LeaderData +0x6c18`,
    /// the `BitMask` `has_tech` reads for a unit) — without the
    /// prerequisite half, as `Object::update_hits` and `Unit::update_los`
    /// read it (`leader +0x6c20 & 4/8/0x10`, bits 0x42..0x44).
    pub(crate) fn holds_unit_bit(&self, who: Player, ti: i32) -> bool {
        self.unit_rec_of_index(ti)
            .and_then(|r| self.unit_types[r].tree)
            .is_some_and(|t| self.tech[who as usize].tech[t])
    }

    /// `Object::update_hits@00647010`: a unit's base `myhits` — the type's
    /// `HITS`, except that a Citizen (`PEASANTS`, `PEASANTSKOREAN`) takes the
    /// hits of the newest of MILITIA, MINUTEMAN and PARTISAN its leader holds
    /// (`docs/GOLDEN.md` §48: run422's Citizens are 50, the Militia's, from
    /// the block after `tech who=0 militia on`).
    ///
    /// SEAM: `Unit::update_hits`'s national and wonder terms, which no
    /// staged nation takes.
    pub(crate) fn type_hits(&self, who: Player, rec: usize) -> i32 {
        let ti = self.unit_types[rec].type_index;
        if matches!(ti, PEASANTS | 0x33) {
            for line in [PARTISAN, MINUTEMAN, MILITIA] {
                if self.holds_unit_bit(who, line)
                    && let Some(r) = self.unit_rec_of_index(line)
                {
                    return self.unit_types[r].hits;
                }
            }
        }
        self.unit_types[rec].hits
    }

    /// `Unit::update_hits@0060e930`: [`Sim::type_hits`] and then the
    /// unit's own terms, of which one is carried — **the Nubians'**. A
    /// merchant (the three ids, `0x3d`, `0x3e`, 400, by exact type) or a
    /// caravan (`is_caravan`, `unit_flags2 & 8`) of a leader with tribe
    /// bonus 4 takes `(NUBIAN_HIT_POINTS + 100) × hits / 100`, truncated
    /// (`0060ec72`..`0060ecab`). run492's Merchant `0/6` is 135 from its
    /// birth block 603, the type's 90 × 150 % (`docs/GOLDEN.md` §54).
    ///
    /// **The Iroquois'** (item 3107, `0060ea36`..`0060eab4`): a unit of a
    /// leader with tribe bonus 0x12 whose trainer (`UnitTypeData +0x40`) is
    /// the Barracks (0x1ab), by equality, and whose type is armed
    /// (`attack`, `+0x1e8`) or a Scout (`is(0x45)`), takes
    /// `(IROQUOIS_EXTRA_HITS + 100) × hits / 100`, truncated — run897's
    /// Scout `1/0` is 63 on block 1, the type's 60 × 105 %.
    ///
    /// SEAM: the other terms — the American marines, Copper and Bananas,
    /// the Dutch — which no scored nation or holding takes.
    /// The Spy's upgrades are carried (item 1585), and the General's
    /// (item 1614).
    pub fn unit_hits(&self, who: Player, rec: usize) -> i32 {
        let k = crate::unit_update_hits_view::UnitHitsK {
            iroquois: self.tuning.iroquois_extra_hits,
            dutch: self.tuning.dutch_hp_bonus,
            nubian: self.tuning.nubian_hit_points,
            spy: self.tuning.spy_upgrade_hp,
            supply: self.tuning.supply_hp_upgrade,
            ..Default::default()
        };
        let mut v = TypeHits {
            sim: self,
            who,
            rec,
            stored_hits: 0,
        };
        crate::unit_update_hits_view::unit_update_hits(&k, &mut v)
    }

    /// **The Dutch kinds' age count** (item 3329, `docs/COMBAT.md` §4):
    /// the owner's `ages` when it has the Dutch power (`has_tribe_bonus
    /// (0x16)`) and the unit record is one of the three merchant ids
    /// (`0x3d`, `0x3e`, `400`), a caravan (`is_caravan`, `unit_flags2 &
    /// 8`) or a supply unit (`is_supply`, `& 0x40`), and is not a
    /// government hero (`is_gov_hero`, `unit_flags & 0x4000000`, which
    /// carries `& 0x40` too); 0 otherwise. It is the one predicate of
    /// `Unit::update_hits@0060e930` (`0060eab4`), `ObjectData::attack
    /// @006469f0` and `ObjectData::armor@00647db0`, each multiplying it by
    /// its own constant. The patch gate on the hero exemption (`8 <
    /// get_patch_version`) is taken as passed: the shipped build is past it.
    pub(crate) fn dutch_ages(&self, who: Player, rec: usize) -> i32 {
        let Some(t) = self.unit_types.get(rec) else {
            return 0;
        };
        let kind = matches!(t.type_index, 0x3d | 0x3e | 400)
            || t.cols.flag2(crate::ai_load::uflags2::CARAVAN)
            || t.cols.flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO);
        let gov_hero = t
            .tree
            .and_then(|i| self.tech_tree.types.get(i))
            .and_then(|d| d.kind.unit())
            .is_some_and(|u| u.patriot);
        match self.tech.get(who as usize) {
            Some(p)
                if kind
                    && !gov_hero
                    && self
                        .tech_tree
                        .has_tribe_bonus(&self.setup, p, crate::ai::tribe::DUTCH) =>
            {
                p.ages
            }
            _ => 0,
        }
    }

    /// The unit record whose `TypeIndex` is `ti`, the first when a nation's
    /// twin shares it.
    pub(crate) fn unit_rec_of_index(&self, ti: i32) -> Option<usize> {
        self.unit_types.iter().position(|t| t.type_index == ti)
    }

    /// `SpellTypeData::is_valid_target@006763c0(spell, who, o, whom)` for a
    /// player's targeted craft on an object.
    ///
    /// ```text
    /// untargeted or an area craft:                1
    /// a building without `c`:                     0
    /// one's own without `e`; an enemy's without `f`;
    ///   an ally's without `h`; a peace partner's without `k`:   0
    /// the Informer on what `who` has infiltrated: 0
    /// ```
    ///
    /// Bribe on an enemy unit is [`Sim::bribe_valid_unit`] (item 2812).
    ///
    /// SEAM: the unit mask arms (`0x100`/`0x200` against objmask
    /// `0x200000`), which none of the Spy's crafts carries. Pilfer's
    /// (an active city, run246's packet refuses the Barracks) and
    /// Sabotage's (a unit must be a vehicle, item 2994, run911) are read,
    /// as are Counterintelligence's and the Sniper's (item 2971).
    pub(crate) fn spell_valid_target(&self, s: i32, who: Player, t: Obj) -> bool {
        let Some(d) = self.spell(s) else {
            return false;
        };
        if d.flags & craft::TARGETED == 0 || d.flags & craft::AREA != 0 {
            return true;
        }
        if matches!(t, Obj::Building(_)) && d.flags & craft::BUILDING == 0 {
            return false;
        }
        let whom = self.owner_of(t);
        let allowed = if who == whom {
            d.flags & craft::OWN != 0
        } else if self.is_enemy(who, whom) {
            d.flags & craft::ENEMY != 0
        } else if self.is_ally(who, whom) {
            d.flags & craft::ALLY != 0
        } else {
            // `is_peace`: a player neither enemy nor ally in a lobby is at
            // peace, which is `diplos` 1.
            d.flags & craft::PEACE != 0
        };
        if !allowed {
            return false;
        }
        match s {
            spell::INFORMER => self.infiltrated_of(t) & Self::who_bit(who) == 0,
            spell::BRIBE => match t {
                Obj::Unit(v) => self.bribe_valid_unit(who, v),
                Obj::Building(_) => false,
            },
            spell::COUNTERINTEL => self.counterintel_valid_target(who, t),
            0x281 => match t {
                Obj::Unit(v) => self.sniper_valid_unit(v),
                Obj::Building(_) => false,
            },
            // `is_valid_target@006763c0`'s Pilfer arm: an active building
            // with object flag `0x20` (a city).
            0x276 => match t {
                Obj::Building(b) => self.buildings[b].active && self.building_is_city(b),
                Obj::Unit(_) => false,
            },
            // Its Sabotage arm (item 2994): a unit target needs the
            // `VEHICLE` objmask (`has_objmask(0x200000)`) when the craft
            // lacks `b`; a building passes on `c`.
            0x280 => match t {
                Obj::Unit(_) => {
                    d.flags & craft::UNIT != 0
                        || self.profile(t).obj_masks & crate::combat::mask::VEHICLE != 0
                }
                Obj::Building(_) => true,
            },
            _ => true,
        }
    }

    /// `SpellTypeData::is_valid_target@006763c0`'s Sniper arm (`0x281`,
    /// `00676660`ff.) for a unit the craft's flags (`fbm`: an enemy's, no
    /// `c`) let through:
    ///
    /// ```text
    /// has_general(v, 0, 0x165) >= 0, is_gov_hero(v), domain != 0   -> 0
    /// not (attack || hero || supply || is(SCOUT)) -> 0
    /// is_caravan, is_worker, is_merchant                            -> 0
    /// ```
    pub(crate) fn sniper_valid_unit(&self, v: usize) -> bool {
        if self.has_ceo_shield(v)
            || crate::type_index::is_gov_hero_type(self.units[v].type_index)
            || self.unit_domain_of(v) != crate::attrition::Domain::Land
        {
            return false;
        }
        if !(self.attack_of(Obj::Unit(v)) != 0
            || self.is_hero_unit(v)
            || self.is_supply_unit(v)
            || self.unit_line_is(v, 0x45))
        {
            return false;
        }
        let caravan = self.units[v].ty.is_some_and(|t| {
            self.unit_types[t]
                .cols
                .flag2(crate::ai_load::uflags2::CARAVAN)
        });
        !(caravan || self.is_worker(v) || self.is_merchant(v))
    }

    /// `SpellTypeData::is_valid_target@006763c0`'s Counterintelligence arm
    /// (`0x277`, `00676880`ff.): a target `who` sees, and
    ///
    /// ```text
    /// one's own:   infiltrated by anyone
    /// an ally's:   infiltrated by anyone, and not yet by `who`
    /// an enemy's:  is(SPY)
    /// anyone else: 0
    /// ```
    pub(crate) fn counterintel_valid_target(&self, who: Player, t: Obj) -> bool {
        if !self.target_seen_by(who, t) {
            return false;
        }
        let inf = self.infiltrated_of(t);
        let whom = self.owner_of(t);
        if whom == who {
            inf != 0
        } else if self.is_ally(who, whom) {
            inf != 0 && inf & Self::who_bit(who) == 0
        } else if self.is_enemy(who, whom) {
            matches!(t, Obj::Unit(v) if self.unit_is_spy(v))
        } else {
            false
        }
    }

    /// `has_general(v, 0, 0x165) >= 0` — a CEO's reach: the unit is one, or
    /// its owner's active, on-map `0x165` hero is within its radius.
    fn has_ceo_shield(&self, v: usize) -> bool {
        let unit = &self.units[v];
        unit.type_index == 0x165
            || (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == unit.owner
                    && x.type_index == 0x165
                    && x.alive()
                    && x.on_map
                    && crate::world::vector_dist(
                        (x.pos.x - unit.pos.x).abs(),
                        (x.pos.y - unit.pos.y).abs(),
                    ) <= self.hero_radius(h) * 0xc0
            })
    }

    /// `SpellTypeData::is_valid_target@006763c0`'s Bribe arm (`0x275`,
    /// `006767d0`ff.) for an enemy's unit — the one target the craft's
    /// flags (`fcbhm`, no `c`) let through, a building being refused ahead
    /// of it:
    ///
    /// ```text
    /// has_general(v, 0, 0x165) >= 0         -> 0   (a CEO's reach)
    /// is_gov_hero(v), v's owner == who      -> 0
    /// an ally                               -> teams_locked && !(v's leader & 2)
    /// an enemy: is(SPY), not (attack || hero), is_caravan, is_worker,
    ///           is_supply, is_merchant, domain != 0           -> 0
    /// ```
    ///
    /// SEAM: the ally arm (`Game::teams_locked`, which no lobby here locks)
    /// refuses; and `has_general` is the owner's active, on-map `0x165`
    /// hero within [`Sim::hero_radius`] × 0xc0, or the unit itself.
    pub(crate) fn bribe_valid_unit(&self, who: Player, v: usize) -> bool {
        let unit = &self.units[v];
        if self.has_ceo_shield(v) {
            return false;
        }
        if crate::type_index::is_gov_hero_type(unit.type_index)
            || unit.owner == who
            || !self.is_enemy(who, unit.owner)
        {
            return false;
        }
        let armed = self.attack_of(Obj::Unit(v)) != 0;
        let caravan = unit.ty.is_some_and(|t| {
            self.unit_types[t]
                .cols
                .flag2(crate::ai_load::uflags2::CARAVAN)
        });
        !(self.unit_is_spy(v)
            || !(armed || self.is_hero_unit(v))
            || caravan
            || self.is_worker(v)
            || self.is_supply_unit(v)
            || self.is_merchant(v)
            || self.unit_domain_of(v) != crate::attrition::Domain::Land)
    }

    /// `SpellTypeData::get_range@00676a80(spell, who, o, whom)`: the row's
    /// range, and the Informer's halved on a building.
    ///
    /// SEAM: the spy upgrade's steps (`get_spy_upgrade × spy_*_upgrade_
    /// range × 192`) and Terra Cotta's; Bribe's and Counterintelligence's
    /// one-tile range on a hero, a supply wagon or a building; Sabotage's
    /// on an objmask-`0x200000` unit.
    pub(crate) fn spell_range(&self, s: i32, t: Obj) -> i32 {
        let r = self.spell(s).map_or(0, |d| d.range);
        if r == 0 {
            return 0;
        }
        if s == spell::INFORMER && matches!(t, Obj::Building(_)) {
            (r + 1) / 2
        } else {
            r
        }
    }

    /// `Group::action_spell@006fe1a0`, the arm a craft that is neither a
    /// unit type nor `TRANSPORT` takes (`docs/GOLDEN.md` §27): each member
    /// that can cast it, or the one with the most `mana_left` when the
    /// craft carries `m`, with the mana the craft asks, the target valid,
    /// and not busy, is written the target and handed
    /// `add_cast_order(ox, whom, x, y, spell, queue, 1)` — `QUEUE_NEW`, or
    /// `QUEUE_FIRST` behind a kill of its head casts for a `g` craft.
    /// Returns the orders laid.
    ///
    /// SEAM: `validate_spell`'s refusals past the Informer's (a Bribe of a
    /// building, a caster the type cannot hold), `can_pay_cost` (run246: 10,
    /// affordable), the pack and unpack members' tests, a building caster's
    /// queue, the feedback and the sounds; and the unit-type and transport
    /// arms. No capture reaches one.
    pub fn group_action_spell(&mut self, g: &Group, s: i32, target: Option<Obj>, at: Pos) -> usize {
        let Some(d) = self.spell(s) else {
            return 0;
        };
        // An untargeted craft (`spell_flags & 0xe` clear) comes with
        // `(ox, whom)` = `(−1, −1)`, the button's own `target_spell(type,
        // −1, −1, 0, 0)` (`Options::do_spell@0071d7a0:189`), and
        // `is_valid_target` answers 1 for it (`006763c0`'s first test).
        if target.is_none() && d.targeted() {
            return 0;
        }
        // `validate_spell`'s `DOUBLE_AGENT` arm: an object already
        // infiltrated by the caster's player refuses the Informer.
        if s == spell::INFORMER
            && target.is_some_and(|t| self.infiltrated_of(t) & Self::who_bit(g.who) != 0)
        {
            return 0;
        }
        let on_map = |s: &Sim, u: usize| s.units[u].alive() && s.units[u].on_map;
        // The `m` craft's caster: the most `mana_left` among the members
        // that can cast it and are not already casting it. SEAM: the
        // leader option `+0x1c & 1` that turns the choice off.
        let mut chosen = None;
        if d.flags & craft::BEST_MANA != 0 {
            let mut best = 0;
            for (i, &u) in g.list.iter().enumerate() {
                if !on_map(self, u) || !self.spell_castable(s, u) || self.is_casting(u, s) {
                    continue;
                }
                let left = self.mana_left(u);
                if best < left {
                    best = left;
                    chosen = Some(i);
                }
            }
        }
        let mut laid = 0;
        for (i, &u) in g.list.iter().enumerate() {
            if !on_map(self, u) || !self.spell_castable(s, u) {
                continue;
            }
            if chosen.is_some_and(|c| c != i) {
                continue;
            }
            // The mana, with what a cast already at the head has spent
            // counted back in, capped at the pool.
            let pending = match self.current_order(u).map(|o| o.body) {
                Some(Body::Cast(c)) => self.spell(c.spell).map_or(0, |h| h.mana),
                _ => 0,
            };
            let left = (self.mana_left(u) + pending).min(self.unit_mana(u));
            if d.mana != 0 && left < d.mana {
                continue;
            }
            if target.is_some_and(|t| !self.spell_valid_target(s, g.who, t)) {
                continue;
            }
            // `is_busy@0060a370`: a head cast (group.rs's reading).
            if matches!(self.current_order(u).map(|o| o.body), Some(Body::Cast(_))) {
                continue;
            }
            // `(ox, whom)` against `(cavarch_o, cavarch_who)`: a change
            // restarts the clock and writes the pair.
            let whom = target.map_or(-1, |t| self.owner_of(t) as i8);
            if self.units[u].cast_target != target || self.units[u].cavarch_who != whom {
                self.units[u].spell_time = 0;
                self.units[u].cast_target = target;
                self.units[u].cavarch_who = whom;
            }
            let pos = if d.flags & craft::FIRST != 0 {
                while matches!(self.current_order(u).map(|o| o.body), Some(Body::Cast(_))) {
                    self.kill_current_order(u);
                }
                QueuePos::First
            } else {
                QueuePos::New
            };
            self.add_cast_order_on(u, s, target, at, pos, true);
            laid += 1;
        }
        laid
    }

    /// `UnitData::is_casting(spell)`: the unit's head order is a cast of
    /// that craft.
    pub(crate) fn is_casting(&self, u: usize, s: i32) -> bool {
        matches!(self.current_order(u).map(|o| o.body), Some(Body::Cast(c)) if c.spell == s)
    }

    /// `Unit::add_cast_order@005e4a60` with a target: the record, and the
    /// action bit when `param_7` is set. The pack and unpack rewrites are
    /// [`Sim::add_cast_order_at`]'s, and no targeted craft is one.
    pub(crate) fn add_cast_order_on(
        &mut self,
        u: usize,
        s: i32,
        target: Option<Obj>,
        at: Pos,
        pos: QueuePos,
        action: bool,
    ) {
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Cast(CastOrder {
                spell: s,
                paid: false,
                target,
                at,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::do_cast@005ebfe0`'s **targeted half** (`spell_flags & 0xe`),
    /// one frame (`docs/GOLDEN.md` §27):
    ///
    /// 1. `pay_cast_costs` once — `paid`, and the craft's `MANA` onto
    ///    `mana_burn`;
    /// 2. the target re-read by `is_valid_target`, and written onto the
    ///    caster (`cavarch_o`/`who`/`uid`);
    /// 3. out of `get_range` + the target's radius — a building's
    ///    `(x_size + y_size) × 48`, a unit's `big_radius` — a
    ///    `find_nearby_spot` ring on the target at `range + radius − 0xc0`
    ///    to `− 0x30`, bearing toward the caster, and an
    ///    `add_move_order(spot, MOVE_TO, QUEUE_FIRST)` ahead of the cast
    ///    (its seventh argument, the 1,344, is never read);
    /// 4. in range: an enemy target must be `is_seen` by the caster's
    ///    player or the order dies; the caster's `visible` takes the
    ///    target's owner and it faces the target; `set_anim` every frame;
    ///    `unit_masks |= 0x20000` once;
    /// 5. `spell_time += 1`, and on the job time `SpellType::cast`, then
    ///    `kill_current_order`.
    ///
    /// SEAM: the captain and inside re-reads of the target; Bribe's
    /// territory refusal; `flags |= 0x80`; `update_local_seen` when the
    /// caster stands in the target owner's fog; (the cloak write is [`Sim::cast_blow`]) for a
    /// craft without `l`; Bribe's `unit_masks2 |= 0x60`; a non-spy
    /// caster's animations; the messages and sounds.
    pub(crate) fn do_cast_targeted(&mut self, u: usize, order: CastOrder) {
        let s = order.spell;
        let Some(d) = self.spell(s) else {
            self.kill_current_order(u);
            return;
        };
        if !order.paid {
            self.units[u].mana_burn = self.units[u]
                .mana_burn
                .saturating_add(i16::try_from(d.mana).unwrap_or(i16::MAX));
            if let Some(front) = self.units[u].orders.front_mut()
                && let Body::Cast(c) = &mut front.body
            {
                c.paid = true;
            }
        }
        let who = self.units[u].owner;
        let Some(t) = order.target.filter(|&t| self.obj_alive(t)) else {
            self.kill_current_order(u);
            return;
        };
        if !self.spell_valid_target(s, who, t) {
            self.kill_current_order(u);
            return;
        }
        self.units[u].cast_target = Some(t);
        self.units[u].cavarch_who = self.owner_of(t) as i8;
        let (centre, radius) = if d.flags & craft::AREA == 0 {
            let radius = match t {
                Obj::Building(b) => self.buildings[b].ty.map_or(0, |ty| {
                    (self.build_types[ty].x_size + self.build_types[ty].y_size) * 0x30
                }),
                Obj::Unit(_) => self.profile(t).big_radius,
            };
            (self.pos_of(t), radius)
        } else {
            (order.at, 0)
        };
        let range = self.spell_range(s, t);
        let here = self.units[u].pos;
        if range != 0
            && crate::world::vector_dist(centre.x - here.x, centre.y - here.y) > range + radius
        {
            let bearing = crate::movement::find_angle(here.x - centre.x, here.y - centre.y);
            let Some(spot) = self.find_nearby_spot(
                u,
                centre,
                range + radius - 0xc0,
                range + radius - 0x30,
                0,
                bearing,
                None,
            ) else {
                return;
            };
            if crate::world::vector_dist(spot.x - centre.x, spot.y - centre.y) > range + radius {
                return;
            }
            self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
            return;
        }
        let whom = self.owner_of(t);
        if d.flags & craft::TARGETED != 0 && whom != who {
            if !self.target_is_seen(Obj::Unit(u), t) {
                self.kill_current_order(u);
                return;
            }
            self.units[u].visible |= Self::who_bit(whom);
            let there = self.pos_of(t);
            let angle = crate::movement::find_angle(there.x - here.x, there.y - here.y);
            // `Unit::set_angle(angle, target, 1)`: the third argument is
            // `Guy::set_angle@005d9010`'s, and set it writes guy 0's
            // `angle` and `last_angle` outright — the figure faces the
            // target this frame rather than turning to it, so it stands
            // (`stopped` 1) on the first frame in range. run245's 756.
            self.unit_set_angle(u, angle);
            self.units[u].movement.facing = angle;
            self.units[u].movement.frame_facing = angle;
        }
        // `Unit::do_cast@005ebfe0:5ec66f`..`5ec759`: a Spy (`is(0x3a, 0)`)
        // — Bribe `CHAR_ATTACK3`, the Informer `CHAR_ATTACKWALK`, a Spy
        // target `CHAR_ATTACK2`, else `CHAR_ATTACK1`; else a Scout line
        // (`is(0x45, 0)`, the Commando's) — Sabotage `CHAR_ATTACKWALK`, the
        // Sniper `CHAR_ATTACK3`, then as a Spy's; else `CHAR_ATTACK1` with
        // `Unit::set_anim`'s third argument set (item 2971).
        let target_spy = matches!(t, Obj::Unit(v) if self.unit_is_spy(v));
        let (anim, p3) = if self.unit_is_spy(u) {
            let a = if s == spell::BRIBE {
                crate::anim::ATTACK3
            } else if s == spell::INFORMER {
                crate::anim::ATTACKWALK
            } else if target_spy {
                crate::anim::ATTACK2
            } else {
                crate::anim::ATTACK1
            };
            (a, false)
        } else if self.unit_line_is(u, 0x45) {
            let a = if s == 0x280 {
                crate::anim::ATTACKWALK
            } else if s == 0x281 {
                crate::anim::ATTACK3
            } else if target_spy {
                crate::anim::ATTACK2
            } else {
                crate::anim::ATTACK1
            };
            (a, false)
        } else {
            (crate::anim::ATTACK1, true)
        };
        self.set_anim(u, anim, false, p3);
        self.units[u].casting = true;
        self.cast_blow(u, s, true);
        self.units[u].spell_time += 1;
        if self.units[u].spell_time < self.spell_job_time(s) {
            return;
        }
        self.cast_blow(u, s, false);
        self.units[u].spell_time = 0;
        self.cast_on(u, s, t);
        self.clear_cast_paid(u);
        self.kill_current_order(u);
    }

    /// `UnitData::is_cloaked@0060a6a0`, with the order list as it stands:
    /// the type's `CLOAKED` (`0x4000`), `unit_masks2 & 0x8000`, or a
    /// `STEALTH` type with no order. `unit_masks & 0x800` is not carried.
    pub(crate) fn unit_is_cloaked(&self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        let cols = self.unit_types[t].cols;
        if cols.flag(crate::ai_load::uflags::CLOAKED) || self.units[u].unit_masks2 & 0x8000 != 0 {
            return true;
        }
        cols.flag(crate::ai_load::uflags::STEALTH) && self.units[u].orders.is_empty()
    }

    /// `Unit::do_cast@005ebfe0`'s two `unit_masks |= 0x11000` writes
    /// (`005ec932`, `005ec9a7`; `docs/GOLDEN.md` §105): a cloaked caster
    /// whose craft lacks `l` ([`craft::KEEP_CLOAK`]) is blown and fought.
    /// `every_frame` is the first, reached each frame ahead of the clock:
    /// it also skips the Sniper (`0x281`) and a Spy (`is(0x3a, 0)`). The
    /// second is the cast frame's, on the Spy too. A stealth-only unit
    /// holds its cast order, so `is_cloaked` answers no for it.
    pub(crate) fn cast_blow(&mut self, u: usize, s: i32, every_frame: bool) {
        let Some(d) = self.spell(s) else {
            return;
        };
        if !self.unit_is_cloaked(u) || d.flags & craft::KEEP_CLOAK != 0 {
            return;
        }
        if every_frame && (s == 0x281 || self.unit_is_spy(u)) {
            return;
        }
        self.units[u].blown = true;
        self.units[u].fought = true;
    }

    /// `do_cast`'s `paid = 0` before its closing `kill_current_order`
    /// (`005ece96`): the head cast's craft stays spent.
    pub(crate) fn clear_cast_paid(&mut self, u: usize) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Cast(c) = &mut front.body
        {
            c.paid = false;
        }
    }

    /// `SpellType::cast@00676ce0`'s targeted cases this crate carries:
    /// the Informer, `cast_double_agent@00673a80` — the target's
    /// `infiltrated |= 1 << who` and its `update_seen(0)`.
    ///
    /// SEAM: the infiltrator's plane (`update_seen`'s `set_seen2` over the
    /// target's line of sight for every leader in `infiltrated`); the
    /// captain re-read of a squad target; the selection hand-over.
    fn cast_on(&mut self, u: usize, s: i32, t: Obj) {
        if s == spell::BRIBE
            && let Obj::Unit(v) = t
            && self.spell_castable(s, u)
        {
            self.cast_bribe_unit(u, v);
            return;
        }
        if s == 0x281
            && let Obj::Unit(v) = t
            && self.spell_castable(s, u)
        {
            self.cast_sniper_unit(v, 2);
            return;
        }
        if s == 0x280 && self.spell_castable(s, u) {
            match t {
                Obj::Unit(v) => self.cast_sniper_unit(v, 3),
                Obj::Building(b) => self.cast_sabotage_building(b),
            }
            return;
        }
        if s != spell::INFORMER || !self.spell_castable(s, u) {
            return;
        }
        let bit = Self::who_bit(self.units[u].owner);
        match t {
            Obj::Unit(v) => self.units[v].infiltrated |= bit,
            Obj::Building(b) => self.buildings[b].infiltrated |= bit,
        }
    }

    /// `SpellType::cast_to_arms@00670880`: a Citizen made the player's
    /// current Militia. `rare` (`+0x54`) keeps the type it was, the damage
    /// keeps its fraction of the hits, and `form` is 0 (read off the
    /// listing, end to end, and run under the emulator on a damage sweep —
    /// `docs/GOLDEN.md` §48).
    pub(crate) fn cast_to_arms(&mut self, u: usize) {
        let former = self.units[u].type_index;
        let who = self.units[u].owner;
        let Some(rec) = self.upgrade_record(who, MILITIA) else {
            return;
        };
        self.units[u].rare = former;
        self.convert_keeping_fraction(u, rec);
        self.units[u].form = 0;
    }

    /// `SpellType::cast_civilian@006704a0`: a Militia made back into the
    /// player's current upgrade of the type `rare` holds — the Citizen
    /// (`PEASANTS`) when `rare` is 0, which it then keeps; `rare` is never
    /// cleared. `form` is 9, `Unit::init`'s for a civilian.
    ///
    /// SEAM: the Citizen's tribe substitution (`tribe_can_type` and the
    /// type's `+0x68`), which no nation here takes.
    pub(crate) fn cast_civilian(&mut self, u: usize) {
        if self.units[u].rare == 0 {
            self.units[u].rare = PEASANTS;
        }
        let who = self.units[u].owner;
        let Some(rec) = self.upgrade_record(who, self.units[u].rare) else {
            return;
        };
        self.convert_keeping_fraction(u, rec);
        self.units[u].form = 9;
    }

    /// `SpellType::cast_create_decoy@00674370` (`docs/GOLDEN.md` §48):
    /// the General `g` copies the armed land squads standing near it.
    ///
    /// ```text
    /// limit = (general_upgrade + 2) × 5
    /// made  = who's captains already decoys
    /// for o from who's last object down to 0:
    ///     a live captain, not a decoy, of the land (`+0x218` 0), armed
    ///     (`UnitData::attack`), not PEASANTS..SCHOLARSKOREAN, not a
    ///     caravan, not a merchant (`is(MERCHANT)`), not `g` itself, and
    ///     within `get_radius × 0xc0` of `g`:
    ///         spot = find_nearby_spot(its type, g, 0x180, −1, 0, 0,
    ///                                 FILTER_NOT_ME g)      else stop
    ///         init_unit(who, its type, spot)                 else stop
    ///         its population handed back (`track_unit_type −1`)
    ///         every figure: `unit_masks |= 1`, `mana_burn` 0
    ///         made += 1; stop at `limit`
    /// none made: the craft's mana handed back
    /// ```
    ///
    /// run422's block 821: the Slingers' copy `0/16`..`0/18` at (7800,
    /// 34296), 384 due north of the General, then the Hoplites'
    /// `0/19`..`0/21`; six `Guy::init_real` draws, and the new figures'
    /// `do_idle` on the same frame.
    ///
    /// `general_upgrade` is the caster's leader's
    /// [`Sim::general_upgrade_level`] (`docs/AI.md` §172).
    ///
    /// The body is [`crate::cast_decoy::cast_create_decoy_steps`] (item 3320;
    /// Porus's and Kutosov's multiples are modelled there).
    ///
    /// SEAM: the leader's other two counters (`+0x93c`, `+0x808`);
    /// `is_caravan` as the two caravan ids; the sound.
    pub(crate) fn cast_create_decoy(&mut self, g: usize) {
        // The body is `crate::cast_decoy::cast_create_decoy_steps`, the
        // original's flow over a view, which the emulator sweep also runs
        // (item 3320).
        let mut view = crate::cast_decoy::SimDecoy::new(self, g);
        crate::cast_decoy::cast_create_decoy_steps(&mut view, g as i32);
    }

    /// The type the decoy's spot search is **asked with**: the caster's
    /// own (`find_nearby_spot@0061de70` reads `this->+0x240/+0x244` for its
    /// block radius and its default span; listing `674666..6746b9` loads
    /// `this` from the caster `param_1`), while the copy itself is made
    /// with the source's type (`6746d1..6746ed`) — twenty-fourth pass,
    /// group 11; A6 row 45. A caster with no type falls back to the source.
    pub(crate) fn decoy_search_type(&self, caster: usize, source: usize) -> usize {
        self.units[caster].ty.unwrap_or(source)
    }

    /// `LeaderData::current_upgrade(who, ti)` as a unit record.
    pub(crate) fn upgrade_record(&self, who: Player, ti: i32) -> Option<usize> {
        let t = self.unit_types[self.unit_rec_of_index(ti)?].tree?;
        let up = self
            .tech_tree
            .current_upgrade(&self.setup, &self.tech[who as usize], t);
        self.unit_record(up)
    }

    /// The two casts' shared tail: `frac = (damage << 8) / hits` before
    /// `set_type(t, 0)`, then `damage = new_hits × frac / 256` rounded
    /// toward zero (`cltd; andl $0xff; addl; sarl $8`).
    fn convert_keeping_fraction(&mut self, u: usize, rec: usize) {
        let hits = self.units[u].max_health.max(1);
        let frac = ((self.units[u].max_health - self.units[u].health) << 8) / hits;
        self.unit_set_type(u, rec);
        let new_hits = self.units[u].max_health;
        let damage = (new_hits * frac) / 256;
        self.units[u].health = new_hits - damage;
    }

    /// Is the object still standing?
    pub(crate) fn obj_alive(&self, t: Obj) -> bool {
        match t {
            Obj::Unit(v) => self.units.get(v).is_some_and(|x| x.alive()),
            Obj::Building(b) => self.buildings.get(b).is_some_and(|x| x.alive),
        }
    }

    /// `Unit::process@00610bc0`'s caster arm: craft back a frame,
    /// `((frame & 1) + r) / 2`, while `unit_masks & 0x2a000` is clear —
    /// [`Self::craft_rate`] is `r` (`docs/AI.md` §119).
    ///
    /// `0x20000` is [`crate::Unit::casting`] and `0x8000`, Forced March,
    /// is `marching`: run630 holds the original's `1/79` at 1000 from its
    /// march on 10777 through run629's 10815, and ours resumes on 10928.
    ///
    /// `0x2000` is [`crate::Unit::ambushing`]: run943 holds the
    /// original's Despot `1/46` at 1000 under `unit_masks` `0x42008` on
    /// every block of 10689..10945, and run952 has it recovering, 392 on
    /// 11579 under `0x40008` (item 3297; item 1470's scan predates both).
    ///
    /// SEAM: the supply wagon's gate.
    pub(crate) fn recover_mana(&mut self, u: usize, frame: i64) {
        if !self.units[u].decoy && self.unit_domain_of(u) == crate::attrition::Domain::Air {
            self.burn_fuel(u);
            return;
        }
        // `Unit::process@00610bc0`'s other arm, `unit_masks & 1`: a decoy's
        // `mana_burn` is its age, one a frame from 0 (run422: 1 on its first
        // block), and it closes once the age reaches `(general_upgrade + 2)
        // × DECOY_TIME / 2` ([`Self::close_decoy`]).
        if self.units[u].decoy {
            self.units[u].mana_burn = self.units[u].mana_burn.saturating_add(1);
            let left = decoy_time_left(
                self.general_upgrade_level(self.units[u].owner),
                self.tuning.decoy_time,
                self.units[u].mana_burn,
            );
            if left == 0 {
                self.close_decoy(u);
            } else if self.decoy_detected(u) {
                // **A detected decoy bleeds every frame** (`610c8a`..
                // `611158`, item 3361): `suffer_attrition(frame % 7 == 0)`,
                // whose argument is the graphic alone. run989d: who=1's
                // Cataphract decoys `1/72`, `1/74` beside who=0's Scout
                // `0/0` take a point a frame (28 → 29 → … on 17388..), and
                // `1/75`, `1/81` are already gone.
                let _ = self.suffer_attrition(u, frame);
            }
            return;
        }
        let unit = &self.units[u];
        if unit.mana_burn == 0
            || unit.casting
            || unit.marching.is_some()
            || unit.ambushing.is_some()
        {
            return;
        }
        let step = i16::try_from(((frame & 1) + i64::from(self.craft_rate(u))) / 2).unwrap_or(1);
        let unit = &mut self.units[u];
        unit.mana_burn -= step.min(unit.mana_burn);
    }

    /// **The craft rate `r`** of `Unit::process@00610bc0`, read off the
    /// listing at `610f2d`..`61101e` (`docs/AI.md` §119, item 1470):
    ///
    /// ```text
    /// r = 2
    /// has_tribe_bonus(10) and is(GENERAL, 1)            -> r = 4
    /// has_preq(SPIES_GENERALS_RECOVER_CRAFT) and
    ///     (is(GENERAL, 1) or is(SPY, 1))                -> r *= 2
    /// is(MEMNON, 0)                                     -> r = MEMNON_REGEN_RATE × r >> 8
    /// ```
    ///
    /// The French General's is the arm a capture holds: run632's `1/79`
    /// takes back two a frame (102 on 11376, 64 on 11395), where one is
    /// `r = 2`'s `(frame & 1 + 2) / 2` on either parity. `is(x, 1)` is
    /// the vslot `+0xb8` with `push 1; push x` — the strict test, the type
    /// or its graft.
    pub(crate) fn craft_rate(&self, u: usize) -> i32 {
        use crate::nations::power::FRENCH;
        let who = self.units[u].owner as usize;
        let tree = self.unit_tree(u);
        // A row the tree does not carry — a fixture's short table; the
        // original's is always whole — is no lineage of anything.
        let is = |x: crate::tech::TypeId, strict: bool| {
            self.tech_tree.types.get(x).is_some()
                && tree.is_some_and(|t| self.tech_tree.is(t, x, strict))
        };
        let general = is(GENERAL, true);
        let mut r = 2;
        if self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[who], FRENCH)
            && general
        {
            r = 4;
        }
        if self
            .tech_tree
            .roles
            .spy_general_recover_preq
            .is_some_and(|t| self.tech_tree.has_tech(&self.setup, &self.tech[who], t))
            && (general || is(SPY, true))
        {
            r *= 2;
        }
        if is(MEMNON, false) {
            // `imul; cltd; and $0xff, %edx; lea; sar $8`: toward zero.
            r = (self.tuning.memnon_regen_rate * r) / 256;
        }
        r
    }

    /// **A decoy's close** (item 1351, `docs/GOLDEN.md` §48): the listing
    /// at `610c3e`..`610c8a` takes `+0x96` up one, and at `(general_upgrade
    /// + 2) × decoy_time / 2 <= mana_burn` calls vslot `+0x150` — read off
    /// the PE at `Unit::vftable` `0xb417d0`, `Unit::close@0060ee50` — with
    /// `(0, −1, 0.0)`, and returns from `process` before the rest of the
    /// unit's frame. `close` with a zero first argument spends no death
    /// draw and writes no `DEATH_OBJS`, and its population arm is gated on
    /// `unit_masks & 1` clear, so a copy hands back nothing
    /// (`cast_create_decoy` already did). What is left is the squad relink,
    /// the orders,
    /// the supply and collision slots, and `Object::close`'s thirty-frame
    /// hold — not `Object::die`'s, so no shot in flight lengthens it.
    ///
    /// run535: who=1's `1/104`..`1/120` stand at `mana_burn` 2499 on block
    /// 14136 and are gone on 14137; `1/93`, at 2498, a block later.
    pub(crate) fn close_decoy(&mut self, u: usize) {
        self.units[u].health = self.units[u].health.min(0);
        self.relink_squad(u);
        self.close_dead_orders(u);
        self.units[u].hold_frames = Self::CLOSE_HOLD;
        self.close_supply(u);
        self.forget(Obj::Unit(u));
        self.clear_decoy_at_close(u);
    }

    /// **`Unit::close@0060ee50`'s last statement** (`:603`, after
    /// `close_orders` and `Object::close`, on every path through the
    /// function): `unit_masks &= ~1` — a closed unit is no decoy. So a
    /// round a decoy fired is a real blow if it lands after its shooter
    /// closed: `Ammo::do_damage` hands the dead slot to `Object::do_damage`,
    /// whose decoy return reads the bit, and `get_damage` prices it on the
    /// slot's own type (item 3322, `docs/AI.md` §265). run956c: who=1's
    /// Horse Archers decoy `1/110` stands at `mana_burn` 2499 on block
    /// 18366 and is gone on 18367 with a round in flight; on 18368 it
    /// lands on who=0's Village `0/2000` beside `1/53`'s, 2.0 together
    /// (867 → 869). run959's packet holds the closed `1/110` at `flags 0`,
    /// `unit_masks` `0x851006`.
    pub(crate) fn clear_decoy_at_close(&mut self, u: usize) {
        self.units[u].decoy = false;
    }

    /// `SpellType::cast_march@00671500` — Forced March on a hero: an
    /// `ActiveSpell` to `frame + duration + general_upgrade ×
    /// duration_upgrade` (`DURATION` and `DURATION_UPGRADE` × 15 at
    /// `SpellType::init`: 150 and 75 frames), `unit_masks |= 0x8000` and
    /// the leader's `0x8000` (`docs/AI.md` §99.13).
    ///
    /// What the march does is [`Sim::unit_speed`]'s and the group's
    /// `march` flag's (`docs/AI.md` §99.15).
    pub(crate) fn cast_march(&mut self, g: usize) {
        let Some(d) = self.spell(crate::orders::spell::FORCED_MARCH) else {
            return;
        };
        let level = self.general_upgrade_level(self.units[g].owner);
        let end = self.frame + i64::from(march_extent(d.duration, d.duration_upgrade, level));
        self.units[g].marching = Some(end);
        let who = self.units[g].owner as usize;
        self.forced_march[who] = true;
    }

    /// `SpellType::cast_ambush@00671610` — Ambush on a hero: an
    /// `ActiveSpell` (`0x27b`) to `frame + duration + general_upgrade ×
    /// duration_upgrade` (`this +0x1d4`, `+0x1d8`, the fields
    /// [`Sim::cast_march`] reads), and `unit_masks |= 0x2800`
    /// (`docs/AI.md` §261).
    ///
    /// Parmenio's (`0x168`) `PARMENIO_AMBUSH_TIME` 8.8 scale is
    /// [`cast_ambush_steps`]'s.
    ///
    /// SEAM: `0x800`, the cloak `UnitData::is_seen` and `is_cloaked` read,
    /// and `Unit::set_new_location` clears on the hero's next step — the
    /// hero arm casts Ambush only on the move, so it lasts a step; the
    /// leader's `0x4000`, which only `Leader::verify_spell_flags` reads;
    /// the console's sound.
    pub(crate) fn cast_ambush(&mut self, g: usize) {
        let Some(d) = self.spell(crate::orders::spell::AMBUSH) else {
            return;
        };
        let level = self.general_upgrade_level(self.units[g].owner);
        let mut view = SimAmbush {
            sim: self,
            g,
            duration: d.duration,
            duration_upgrade: d.duration_upgrade,
            level,
        };
        cast_ambush_steps(&mut view);
    }

    /// `Caster::process_spells(o, who, 0)@00739ad0` on a hero, from the
    /// head of `Unit::process`: a spell whose end frame has passed is
    /// removed, and a Forced March's takes `unit_masks & 0x8000` with it;
    /// then `Leader::verify_spell_flags@006ce190` drops the leader's
    /// `0x8000` when none of its heroes still marches.
    pub(crate) fn process_spells(&mut self, u: usize, frame: i64) {
        // An Ambush's `0x27b` arm: `unit_masks &= 0xffffd7ff`.
        if self.units[u].ambushing.is_some_and(|end| end < frame) {
            self.units[u].ambushing = None;
        }
        let Some(end) = self.units[u].marching else {
            return;
        };
        if end >= frame {
            return;
        }
        self.units[u].marching = None;
        let who = self.units[u].owner;
        if !self
            .units
            .iter()
            .any(|x| x.owner == who && x.alive() && x.marching.is_some())
        {
            self.forced_march[who as usize] = false;
        }
    }

    /// `HeroData::get_radius@00739e50` for the hero `h`, in tiles: the
    /// general radius scaled by `(general_upgrade + 3) / 2`, Parmenio's
    /// (`0x168`) 8.8 scale, Wellington's (`0x170`) and Kutosov's (`0x176`)
    /// percentages, the Terra Cotta Army's range, then a military patriot's
    /// (`0x160`, `0x162`, `0x164`) or an economic patriot's (`0x161`,
    /// `0x163`, `0x165`) bonus — [`crate::supply::general_radius`] in that
    /// order. run470's Senator `1/80` is type 353, `0x161`: 6 × 3 / 2 + 1.
    ///
    /// `get_general_upgrade` is the hero's leader's
    /// [`Sim::general_upgrade_level`]: with all three held the radius is
    /// 18 tiles, not 9 — the coverage pair's General `1/150` marches
    /// `1/16` sixteen tiles off (`docs/AI.md` §172).
    ///
    /// SEAM: the wonder test
    /// (`has_wonder(0x211)`) is read as absent — `TERRA_COTTA_RANGE` ships 0;
    /// the patriot tests are `is(t, 1)`, taken as the type itself.
    pub(crate) fn hero_radius(&self, h: usize) -> i32 {
        use crate::supply::Patriot;
        let t = self.units[h].type_index;
        let patriot = match t {
            0x160 | 0x162 | 0x164 => Some(Patriot::Military),
            0x161 | 0x163 | 0x165 => Some(Patriot::Economic),
            _ => None,
        };
        crate::supply::general_radius(
            &self.tuning,
            &crate::supply::General {
                upgrades: self.general_upgrade_level(self.units[h].owner),
                parmenio: t == 0x168,
                wellington: t == 0x170,
                kutosov: t == 0x176,
                terra_cotta: false,
                patriot,
            },
        )
    }

    /// `ObjectData::has_general(u, 0x8000, -1) >= 0` (`00646b00`): `u` is
    /// itself a hero (`UnitData::is_hero`, vslot `+0xc4`, `unit_flags2 &
    /// 0x20`) on a Forced March, or `HeroesData::find_hero@0073a1b0` finds
    /// one of its owner's: a hero record active (`hero_flags & 1`, from
    /// `Heroes::init_hero` at `Unit::init` to `close_hero`), its unit
    /// active (vslot `+0x8`) and on the map (vslot `+0xbc`,
    /// `UnitData::is_on_map`, `inside_up < 0`), with `unit_masks & 0x8000`,
    /// and `vector_dist(|dx|, |dy|) − 0 ≤ get_radius × 0xc0` (`0073a2c9`–
    /// `0073a2ea`; the `− 0` is a building's footprint, vslot `+0x20`, 0
    /// for a unit).
    pub(crate) fn near_marching_hero(&self, u: usize) -> bool {
        let unit = &self.units[u];
        (unit.marching.is_some() && self.is_hero_unit(u)) || self.find_marching_hero(u) >= 0
    }

    /// `HeroesData::find_hero@0073a1b0(.., 0x8000, -1, 0)` for `u`'s owner
    /// and point: the first unit (in slot order, the hero list's own) that
    /// qualifies as [`Sim::near_marching_hero`] lists, `-1` for none.
    fn find_marching_hero(&self, u: usize) -> i32 {
        let (who, at) = (self.units[u].owner, self.units[u].pos);
        (0..self.units.len())
            .find(|&h| {
                let x = &self.units[h];
                x.owner == who
                    && x.alive()
                    && x.on_map
                    && x.marching.is_some()
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                        <= self.hero_radius(h) * 0xc0
            })
            .map_or(-1, |h| h as i32)
    }

    /// `ObjectData::has_general(u, 0, 0x162) >= 0` (`00646b00`), the
    /// question `UnitData::get_speed@00608720` asks before it doubles a
    /// siege unit (item 3390): `u` is itself a hero of type THEMONARCH
    /// (`0x162`), or `HeroesData::find_hero@0073a1b0` finds one of its
    /// owner's — the hero record active, its unit active and on the map,
    /// `is(0x162)`, and `vector_dist(|dx|, |dy|) − 0 ≤ get_radius × 0xc0`
    /// from `u`'s own point (`has_general` hands `find_hero` the unit's
    /// `+0x10`/`+0x14`, never the virtual's point; the `− 0` is a
    /// building's footprint). `param_1` is 0, so no mask is asked.
    /// run1013c's Spanish Bombards `1/113` and `1/114` step 49 on 13484
    /// against ours' 24, `myspeed` 23 (the group's cap) both sides.
    ///
    /// SEAM: the leader's per-type count at `+0x56fe` (the early `return
    /// -1` on a zero) is not read, as in
    /// [`Sim::president_near_building`]; `find_hero` finds none either
    /// without a live Monarch. `is(0x162)` is taken as the type itself, as
    /// in [`Sim::hero_radius`].
    pub(crate) fn near_monarch(&self, u: usize) -> bool {
        const MONARCH: i32 = 0x162;
        let unit = &self.units[u];
        if unit.type_index == MONARCH && self.is_hero_unit(u) {
            return true;
        }
        let (who, at) = (unit.owner, unit.pos);
        (0..self.units.len()).any(|h| {
            let x = &self.units[h];
            x.owner == who
                && x.type_index == MONARCH
                && x.alive()
                && x.on_map
                && self.is_hero_unit(h)
                && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                    <= self.hero_radius(h) * 0xc0
        })
    }

    /// Whether `u`'s leader has a hero on a Forced March —
    /// `LeaderData::leader_flags & 0x8000`, [`Sim::forced_march`]. Gaia's
    /// is never set.
    pub(crate) fn leader_marching(&self, u: usize) -> bool {
        self.forced_march
            .get(self.units[u].owner as usize)
            .copied()
            .unwrap_or(false)
    }

    /// `UnitData::speed@0060aae0` — layer two of the speed pipeline
    /// (`docs/MOVEMENT.md`, "The speed pipeline"): the walk of
    /// [`crate::unit_speed_view::unit_speed`] over this unit (item 3399,
    /// swept against the original in `sweep/unit_data_speed.rs`). The cached
    /// `myspeed`, and a type not of the land (`+0x218 != 0`) takes nothing
    /// more.
    ///
    /// **Forced March** (`0060ad0x`–`0060ae10`): under the leader's
    /// `0x8000` and a marching hero ([`Sim::near_marching_hero`]), the speed
    /// is `FORCED_MARCH_SPEED × UNIT_MOVE_SPEED` — 42, no shift on this arm
    /// (`0060add1`–`0060addc`) — or `myspeed` when that is larger
    /// (`cmovle`, `0060ae0e`): a march never slows a unit. **Alexander's
    /// arm** (the hero or the unit `is(0x166)`/`is(0x167)`) multiplies in
    /// `ALEXANDER_FORCED_MARCH_SPEED` and shifts out the 8.8 scale
    /// (`0060ade2`–`0060ae04`).
    ///
    /// **The Iroquois spear** (`0060ab0x`–`0060ad0x`, item 3211): a unit
    /// of the Spearmen line (`is(0xa6, 0)`) standing in a cell whose owner
    /// its leader `is_ally` adds `IROQUOIS_SPEAR_SPEED[k]`, `k` the first
    /// of `is(0xa6..0xa9, 1)` it answers, and nothing when it answers none.
    /// run932's `1/28`..`1/30` (TI 166, born on 5809) step one point past
    /// the squad before them in their own ground. The march arm compares
    /// against `myspeed`, not this sum, and so replaces it.
    ///
    /// SEAM: the hero auras below the march — Spitamenes, Blucher, Porus,
    /// Charles, Napoleon — each behind a leader's power (`+0x59cc`..
    /// `+0x59e8`); none is carried, nor the constants they scale by.
    pub(crate) fn unit_speed(&self, u: usize) -> i32 {
        if self.units[u].is_gaia() {
            return self.units[u].movement.speed;
        }
        crate::unit_speed_view::unit_speed(&mut LiveSpeed { s: self, u })
    }

    /// `ObjectData::is(x, strict)` on unit `u`: the lineage test on its
    /// tree row, false off a tree or a row the tree lacks.
    fn speed_is(&self, u: usize, x: i32, strict: bool) -> bool {
        let Some(tree) = self.unit_tree(u) else {
            return false;
        };
        let Ok(x) = crate::tech::TypeId::try_from(x) else {
            return false;
        };
        self.tech_tree.types.get(x).is_some() && self.tech_tree.is(tree, x, strict)
    }

    /// **The tank** — `Unit::process@00610bc0`'s air arm, taken for a type
    /// of domain 2 in place of the caster's (`docs/ORDERS.md` §38.1). On
    /// the map (`inside_up < 0`) the plane burns one a frame while it has
    /// any left, `mana_left@00609a30 != 0`; inside anything, it refills
    /// [`Tuning::air_unit_mana_recharge`](crate::tuning::Tuning) a frame,
    /// to 0. run223 and run265: the Fighter's 1 on 611, the block after
    /// its `add`, 112 on its landing, −2 a block inside to 0 on 778, and
    /// 400 — its `MANA` — on 1178.
    ///
    /// SEAM: the heal inside, `aircraft_heal_rate[heal level]` frames
    /// apart on the unit's phase (two thirds of it for one type), and the
    /// `+0x38` word it writes; no captured plane is ever damaged.
    fn burn_fuel(&mut self, u: usize) {
        let outside = self.units[u].inside.is_none() && self.units[u].inside_unit.is_none();
        if outside {
            if self.mana_left(u) != 0 {
                self.units[u].mana_burn = self.units[u].mana_burn.saturating_add(1);
            }
        } else {
            let r = i16::try_from(self.tuning.air_unit_mana_recharge).unwrap_or(i16::MAX);
            let unit = &mut self.units[u];
            unit.mana_burn = if unit.mana_burn > r {
                unit.mana_burn - r
            } else {
                0
            };
        }
    }
}

#[cfg(test)]
mod decoy_type_tests {
    /// **The spot is searched with the caster's type** (group 11; A6 row
    /// 45): a Cannon's block radius 2 beside a General's 1 is not the
    /// General's search.
    #[test]
    fn the_decoy_search_is_asked_with_the_casters_type() {
        let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
        let general = s.add_unit_type(crate::UnitType::default());
        let cannon = s.add_unit_type(crate::UnitType::default());
        let mut g = crate::Unit::new(0, 0, crate::Pos::new(500, 500), 10);
        g.ty = Some(general);
        let g = s.add_unit(g);
        assert_eq!(s.decoy_search_type(g, cannon), general);
        s.units[g].ty = None;
        assert_eq!(
            s.decoy_search_type(g, cannon),
            cannon,
            "untyped: the source"
        );
    }
}

/// **`SpellType::cast_march@00671500`'s span** (item 2287): the
/// `ActiveSpell` it appends ends `duration (+0x1d4) + general_upgrade ×
/// duration_upgrade (+0x1d8)` frames past the frame it is cast on, in the
/// original's 32-bit arithmetic.
pub(crate) fn march_extent(duration: i32, duration_upgrade: i32, level: i32) -> i32 {
    duration.wrapping_add(level.wrapping_mul(duration_upgrade))
}

/// What `SpellType::cast_ambush@00671610` reads and writes, in the order it
/// does (item 3320): the sweep (`sweep/spell_type_cast_ambush.rs`) lays the
/// original's records as bytes and implements this over them; [`Sim`]
/// implements it over the crate's state.
pub(crate) trait AmbushView {
    /// `game->frame` (`+0x550`).
    fn frame(&self) -> i32;
    /// `SpellType +0x1d4`, `+0x1d8`.
    fn spell_extent(&self) -> (i32, i32);
    /// `LeaderData::get_general_upgrade`.
    fn general_upgrade(&mut self) -> i32;
    /// The unit's `is(0x168, 0)` (vslot `+0xb8`; Parmenio).
    fn is_parmenio(&mut self) -> bool;
    /// `constants->parmenio_ambush_time` (`+0xb58`), 8.8.
    fn parmenio_ambush_time(&self) -> i32;
    /// The hero's `ActiveSpell { 0x27b, start, end }` appended.
    fn push_active_spell(&mut self, start: i32, end: i32);
    /// The unit's `unit_masks |= 0x2800` and the leader's `leader_flags |=
    /// 0x4000`.
    fn set_flags(&mut self);
    /// The console's `S_GENERAL_AMBUSH` when the caster is its player.
    fn console_sound(&mut self);
}

/// `SpellType::cast_ambush@00671610`'s body over [`AmbushView`]. Parmenio's
/// extent is scaled by `PARMENIO_AMBUSH_TIME` (8.8): the product shifted
/// right by eight, rounding toward zero.
pub(crate) fn cast_ambush_steps<V: AmbushView>(v: &mut V) {
    let start = v.frame();
    let level = v.general_upgrade();
    let (duration, upgrade) = v.spell_extent();
    let mut extent = march_extent(duration, upgrade, level);
    if v.is_parmenio() {
        let p = v.parmenio_ambush_time().wrapping_mul(extent);
        extent = p.wrapping_add((p >> 31) & 0xff) >> 8;
    }
    let end = v.frame().wrapping_add(extent);
    v.push_active_spell(start, end);
    v.set_flags();
    v.console_sound();
}

/// [`Sim`]'s [`UnitSpeedView`](crate::unit_speed_view::UnitSpeedView) over
/// one unit. The unit's vslot `+0x18` is `Buffer::is_pending_load`, a
/// constant 1; its own `units` slot carries `0x8000` when it is marching
/// (`marching`); `find_hero` is [`Sim::find_marching_hero`].
struct LiveSpeed<'a> {
    s: &'a Sim,
    u: usize,
}

impl crate::unit_speed_view::UnitSpeedView for LiveSpeed<'_> {
    fn myspeed(&self) -> i32 {
        self.s.units[self.u].movement.speed
    }
    fn not_land(&self) -> bool {
        self.s.unit_domain_of(self.u) != crate::attrition::Domain::Land
    }
    fn is_this(&mut self, x: i32, strict: bool) -> bool {
        self.s.speed_is(self.u, x, strict)
    }
    fn ground_owner(&mut self) -> Option<i32> {
        self.s
            .world
            .owner_at(self.s.units[self.u].pos)
            .player()
            .map(i32::from)
    }
    fn is_ally(&mut self, other: i32) -> bool {
        u8::try_from(other).is_ok_and(|o| self.s.is_ally(self.s.units[self.u].owner, o))
    }
    fn spear_speed(&self, k: usize) -> i32 {
        self.s.tuning.iroquois_spear_speed[k]
    }
    fn leader_marching(&self) -> bool {
        self.s.leader_marching(self.u)
    }
    fn is_active(&mut self) -> bool {
        true
    }
    fn is_hero(&mut self) -> bool {
        self.s.is_hero_unit(self.u)
    }
    fn own_slot_marching(&mut self) -> bool {
        self.s.units[self.u].marching.is_some()
    }
    fn own_slot(&self) -> i32 {
        self.u as i32
    }
    fn find_hero(&mut self) -> i32 {
        self.s.find_marching_hero(self.u)
    }
    fn hero_is(&mut self, slot: i32, x: i32) -> bool {
        usize::try_from(slot).is_ok_and(|h| self.s.speed_is(h, x, false))
    }
    fn forced_march_speed(&self) -> i32 {
        self.s.tuning.forced_march_speed
    }
    fn alexander_forced_march_speed(&self) -> i32 {
        self.s.tuning.alexander_forced_march_speed
    }
    fn unit_move_speed(&self) -> i32 {
        crate::combat::UNIT_MOVE_SPEED
    }
}

/// [`Sim`]'s [`AmbushView`]: an `ActiveSpell` is `ambushing`; the cloak
/// `0x800`, the leader's `0x4000` and the sound are unmodelled (the SEAM on
/// [`Sim::cast_ambush`]).
struct SimAmbush<'a> {
    sim: &'a mut Sim,
    g: usize,
    duration: i32,
    duration_upgrade: i32,
    level: i32,
}

impl AmbushView for SimAmbush<'_> {
    fn frame(&self) -> i32 {
        self.sim.frame as i32
    }
    fn spell_extent(&self) -> (i32, i32) {
        (self.duration, self.duration_upgrade)
    }
    fn general_upgrade(&mut self) -> i32 {
        self.level
    }
    fn is_parmenio(&mut self) -> bool {
        self.sim.units[self.g].type_index == 0x168
    }
    fn parmenio_ambush_time(&self) -> i32 {
        self.sim.tuning.parmenio_ambush_time
    }
    fn push_active_spell(&mut self, _start: i32, end: i32) {
        self.sim.units[self.g].ambushing = Some(i64::from(end));
    }
    fn set_flags(&mut self) {}
    fn console_sound(&mut self) {}
}

#[cfg(test)]
mod tests {
    use crate::tech::{TypeDef, UnitTraits};

    /// **Parmenio's Ambush** (item 3320; `SpellType::cast_ambush@00671610`):
    /// his `ActiveSpell` runs `PARMENIO_AMBUSH_TIME` (the file's `3/1`, 768
    /// at 8.8) times the usual extent, shifted right by eight and rounded
    /// toward zero. Made to fail with the tuning at 256.
    #[test]
    fn parmenio_s_ambush_is_three_times_as_long() {
        struct V(i32, bool, Option<(i32, i32)>);
        impl super::AmbushView for V {
            fn frame(&self) -> i32 {
                1000
            }
            fn spell_extent(&self) -> (i32, i32) {
                (300, 100)
            }
            fn general_upgrade(&mut self) -> i32 {
                2
            }
            fn is_parmenio(&mut self) -> bool {
                self.1
            }
            fn parmenio_ambush_time(&self) -> i32 {
                self.0
            }
            fn push_active_spell(&mut self, start: i32, end: i32) {
                self.2 = Some((start, end));
            }
            fn set_flags(&mut self) {}
            fn console_sound(&mut self) {}
        }
        assert_eq!(Tuning::RON.parmenio_ambush_time, 768);
        let run = |t: i32, parmenio: bool| {
            let mut v = V(t, parmenio, None);
            super::cast_ambush_steps(&mut v);
            v.2.expect("pushed")
        };
        assert_eq!(run(Tuning::RON.parmenio_ambush_time, false), (1000, 1500));
        assert_eq!(run(Tuning::RON.parmenio_ambush_time, true), (1000, 2500));
        // Toward zero: 500 × -255 / 256 is -498.05.
        assert_eq!(run(-255, true), (1000, 1000 - 498));
    }
    use crate::world::{Cell, Terrain, World};
    use crate::{Pos, Sim, Tuning, Unit, UnitType};

    /// **The Iroquois spear** (item 3211; `UnitData::speed@0060aae0`): a
    /// unit of the line `is(0xa6, 0)` on a cell its leader is allied to
    /// steps `IROQUOIS_SPEAR_SPEED[k]` more, `k` its strict `is(0xa6 +
    /// k, 1)`; off allied ground, or off the line, nothing. run932's
    /// `1/28`..`1/30` (TI 166). Made to fail with the term dropped and
    /// with the ally test dropped.
    #[test]
    fn an_iroquois_spearman_steps_faster_on_allied_ground() {
        use crate::world::Owner;
        let mut world = World::new(8, 8);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
        let mut tuning = Tuning::RON;
        tuning.iroquois_spear_speed = [1, 2, 3, 4];
        let mut s = Sim::new(tuning, world, 2);
        let mut t = crate::tech::TechTree::new();
        while t.types.len() <= 0xaa {
            t.add(TypeDef::unit("filler", UnitTraits::default()));
        }
        t.types[0xa7].from = Some(0xa6);
        s.set_tech_tree(t);
        let unit = |s: &mut Sim, row: usize| {
            let ty = s.add_unit_type(UnitType {
                tree: Some(row),
                type_index: i32::try_from(row).unwrap(),
                ..UnitType::default()
            });
            let mut u = Unit::new(1, 0, Pos::new(3 * 768 + 10, 3 * 768 + 10), 20);
            u.ty = Some(ty);
            u.movement.speed = 25;
            s.add_unit(u)
        };
        let spear = unit(&mut s, 0xa6);
        let pike = unit(&mut s, 0xa7);
        let other = unit(&mut s, 0xaa);
        let cell = Cell::new(3, 3);
        assert_eq!(s.unit_speed(spear), 25, "unowned ground");
        s.world.set_owner(cell, Owner::Player(1), Owner::None);
        assert_eq!(s.unit_speed(spear), 26, "its own ground, entry0");
        assert_eq!(s.unit_speed(pike), 27, "of the line from 0xa6, entry1");
        assert_eq!(s.unit_speed(other), 25, "off the line");
        s.world.set_owner(cell, Owner::Player(0), Owner::None);
        assert_eq!(s.unit_speed(spear), 25, "another leader's, not allied");
    }

    /// A tree whose rows are `TypeIndex`'s, up to Memnon, and one unit of
    /// player 1 for each of the General, the Spy, Memnon and a filler row,
    /// each holding a full pool (`MANA` 1000).
    fn casters(tuning: Tuning) -> (Sim, [usize; 4]) {
        let mut world = World::new(8, 8);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
        let mut s = Sim::new(tuning, world, 2);
        let mut t = crate::tech::TechTree::new();
        while t.types.len() <= super::MEMNON {
            let name = match t.types.len() {
                super::GENERAL => "General",
                super::SPY => "Spy",
                super::MEMNON => "Memnon",
                _ => "filler",
            };
            t.add(TypeDef::unit(name, UnitTraits::default()));
        }
        s.set_tech_tree(t);
        let units = [super::GENERAL, super::SPY, super::MEMNON, 0x40].map(|row| {
            let ty = s.add_unit_type(UnitType {
                mana: 1000,
                tree: Some(row),
                type_index: i32::try_from(row).unwrap(),
                ..UnitType::default()
            });
            let mut u = Unit::new(1, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = Some(ty);
            s.add_unit(u)
        });
        (s, units)
    }

    /// **`do_cast`'s two `0x11000` writes** (`005ec932`, `005ec9a7`; item
    /// 2804, `docs/GOLDEN.md` §105): a cloaked caster of a craft without
    /// `l` is blown and fought — on the first, every-frame site not a Spy
    /// and not the Sniper, on the cast frame's a Spy too. A craft with `l`
    /// keeps the cloak, and a stealth-only unit holds its cast order, so
    /// it is not cloaked at all. Made to fail with the cloak test dropped,
    /// with the `l` test dropped, and with the Spy exemption dropped.
    #[test]
    fn a_cloaked_caster_of_a_craft_without_l_is_blown() {
        use crate::ai_load::{UnitCols, uflags};
        use crate::orders::{SpellType, spell};
        let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
        s.spells =
            vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
        let at = |c: i32| usize::try_from(c - spell::FIRST).unwrap();
        s.spells[at(spell::BRIBE)].flags = 0;
        s.spells[at(spell::INFORMER)].flags = super::craft::KEEP_CLOAK;
        s.spells[at(0x281)].flags = 0;
        let cloak = |s: &mut Sim, u: usize, flags: u32| {
            let t = s.units[u].ty.unwrap();
            s.unit_types[t].cols = UnitCols {
                unit_flags: flags,
                ..UnitCols::default()
            };
        };
        cloak(&mut s, spy, uflags::CLOAKED);
        cloak(&mut s, filler, uflags::CLOAKED);
        let reset = |s: &mut Sim, u: usize| {
            s.units[u].blown = false;
            s.units[u].fought = false;
        };
        let bits = |s: &Sim, u: usize| (s.units[u].blown, s.units[u].fought);
        // The first site: a cloaked non-Spy, the craft without `l`.
        s.cast_blow(filler, spell::BRIBE, true);
        assert_eq!(bits(&s, filler), (true, true));
        reset(&mut s, filler);
        s.cast_blow(filler, spell::INFORMER, true);
        s.cast_blow(filler, 0x281, true);
        assert_eq!(bits(&s, filler), (false, false), "`l`, and the Sniper");
        // A Spy is exempt at the first site and not at the second.
        s.cast_blow(spy, spell::BRIBE, true);
        assert_eq!(bits(&s, spy), (false, false));
        s.cast_blow(spy, spell::INFORMER, false);
        assert_eq!(bits(&s, spy), (false, false), "the Informer keeps it");
        s.cast_blow(spy, spell::BRIBE, false);
        assert_eq!(bits(&s, spy), (true, true));
        // Not cloaked: a stealth-only unit with its cast order held.
        cloak(&mut s, filler, uflags::STEALTH);
        reset(&mut s, filler);
        s.add_cast_order(filler, spell::BRIBE);
        s.cast_blow(filler, spell::BRIBE, false);
        assert_eq!(bits(&s, filler), (false, false));
        s.units[filler].orders.clear();
        assert!(s.unit_is_cloaked(filler), "idle, it is");
    }

    /// **Bribe's target arm** (`SpellTypeData::is_valid_target@006763c0`,
    /// item 2812): an enemy's armed, land, non-spy, non-worker unit; a
    /// building, an unarmed unit, a Spy, a worker, an ally and a peace
    /// partner are refused. Made to fail with the war test and with the
    /// armed test dropped.
    #[test]
    fn bribe_takes_only_an_enemy_s_armed_land_unit() {
        use crate::combat::Obj;
        use crate::orders::{SpellType, spell};
        let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
        s.spells =
            vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
        s.spells[usize::try_from(spell::BRIBE - spell::FIRST).unwrap()].flags = 0x2 | 0x20 | 0x80;
        let target = s.add_unit({
            let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = s.units[filler].ty;
            u
        });
        s.units[target].owner = 0;
        let t = Obj::Unit(target);
        s.units[spy].owner = 1;
        assert!(!s.spell_valid_target(spell::BRIBE, 1, t), "at peace");
        s.declare_war(0, 1);
        assert!(!s.spell_valid_target(spell::BRIBE, 1, t), "unarmed");
        let ty = s.units[target].ty.unwrap();
        s.unit_types[ty].combat.attack = 5;
        assert!(
            s.spell_valid_target(spell::BRIBE, 1, t),
            "an enemy's soldier"
        );
        assert!(
            !s.spell_valid_target(spell::BRIBE, 1, Obj::Unit(spy)),
            "its own"
        );
    }

    /// **The Sniper's target arm** (`SpellTypeData::is_valid_target@006763c0`'s
    /// `0x281`, item 2971): an enemy's armed land unit; refused unarmed, a
    /// worker's line aside, and in the air. Made to fail with the armed
    /// test dropped.
    #[test]
    fn sniper_takes_only_an_enemy_s_armed_land_unit() {
        use crate::combat::Obj;
        use crate::orders::{SpellType, spell};
        let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
        s.spells =
            vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
        s.spells[usize::try_from(0x281 - spell::FIRST).unwrap()].flags = 0x2 | 0x20;
        let target = s.add_unit({
            let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = s.units[filler].ty;
            u
        });
        let t = Obj::Unit(target);
        s.declare_war(0, 1);
        assert!(!s.spell_valid_target(0x281, 1, t), "unarmed");
        let ty = s.units[target].ty.unwrap();
        s.unit_types[ty].combat.attack = 5;
        assert!(s.spell_valid_target(0x281, 1, t), "an enemy's soldier");
        assert!(
            !s.spell_valid_target(0x281, 0, Obj::Unit(spy)),
            "no war arm for its own side"
        );
    }

    /// **Counterintelligence's target arm** (`0x277`, item 2971): one's own
    /// is a target only once infiltrated; an enemy's only if it is a Spy.
    /// Made to fail with the infiltrated test dropped and with the Spy
    /// test dropped.
    #[test]
    fn counterintelligence_takes_an_infiltrated_friend_or_an_enemy_spy() {
        use crate::combat::Obj;
        use crate::orders::{SpellType, spell};
        let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
        s.spells =
            vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
        s.spells[usize::try_from(spell::COUNTERINTEL - spell::FIRST).unwrap()].flags =
            0x2 | 0x10 | 0x20;
        let friend = Obj::Unit(filler);
        assert!(
            !s.spell_valid_target(spell::COUNTERINTEL, 1, friend),
            "not infiltrated"
        );
        s.units[filler].infiltrated = 1;
        assert!(
            s.spell_valid_target(spell::COUNTERINTEL, 1, friend),
            "infiltrated"
        );
        let foe_filler = s.add_unit({
            let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = s.units[filler].ty;
            u
        });
        let foe_spy = s.add_unit({
            let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = s.units[spy].ty;
            u
        });
        s.declare_war(0, 1);
        assert!(
            !s.spell_valid_target(spell::COUNTERINTEL, 1, Obj::Unit(foe_filler)),
            "an enemy soldier"
        );
        assert!(
            s.spell_valid_target(spell::COUNTERINTEL, 1, Obj::Unit(foe_spy)),
            "an enemy Spy"
        );
    }

    /// **Bribe's conversion** (`SpellType::cast_bribe@00672d00`'s unit arm,
    /// `Unit::swap_team@006118d0`, item 2823): a copy is born under the
    /// caster's owner carrying the damage and the facing, the old unit is
    /// closed without a death and held thirty frames, and the caster's
    /// leader counts one. Made to fail with the counter dropped, with the
    /// old unit left standing, and with the damage not carried.
    #[test]
    fn bribe_swaps_a_unit_s_team_and_closes_the_old_one() {
        let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
        let target = s.add_unit({
            let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = s.units[filler].ty;
            u
        });
        s.units[spy].owner = 1;
        s.units[target].health = 15;
        s.units[target].max_health = 20;
        s.units[target].movement.facing = crate::movement::Angle(0x1200_0000);
        let bribes = s.tech[1].bribes;
        let before = s.units.len();
        let n = s.cast_bribe_unit(spy, target).expect("a copy is born");
        assert!(s.units.len() > before || n < before, "a unit is made");
        assert_eq!(s.units[n].owner, 1, "under the caster's owner");
        assert_eq!(
            s.units[n].max_health - s.units[n].health,
            5,
            "damage carried"
        );
        assert_eq!(
            s.units[n].movement.heading, s.units[n].movement.facing,
            "the figure is settled: `dest_angle` equals `angle` (run880)"
        );
        assert_eq!(
            s.units[n].movement.facing,
            crate::movement::Angle(0x1200_0000)
        );
        assert!(!s.units[target].alive(), "the old one is closed");
        assert_eq!(s.units[target].hold_frames, Sim::CLOSE_HOLD);
        assert_eq!(s.tech[1].bribes, bribes + 1, "the leader's +0x8f4");
        assert!(s.cast_bribe_unit(spy, n).is_none(), "its own: nothing");
    }

    /// What one frame of each parity takes back from a `mana_burn` of 100.
    fn steps(s: &mut Sim, u: usize) -> [i16; 2] {
        [10, 11].map(|frame| {
            s.units[u].mana_burn = 100;
            s.recover_mana(u, frame);
            100 - s.units[u].mana_burn
        })
    }

    /// **`SPIES_GENERALS_RECOVER_CRAFT` doubles a General's and a Spy's
    /// rate** (`Unit::process@00610bc0:610f2d`, item 1673): the Persian
    /// General `1/150` of the coverage pair runs `mana_burn` 12, 10, ... 0
    /// on 3838..3844 under All Technologies, two a frame, and so ours' 480
    /// on 3864 kept it from the Create Decoys cast the original laid. The
    /// gate is the bonus's prerequisite tech held; the doubling is `r *=
    /// 2` on a General or a Spy, not on any other caster.
    ///
    /// Made to fail with the arm dropped and with the gate read as held.
    #[test]
    fn the_recover_craft_bonus_doubles_a_generals_and_a_spys_rate() {
        let (mut s, [general, spy, _, filler]) = casters(Tuning::RON);
        s.tech_tree.roles.spy_general_recover_preq = Some(0x40);
        s.tech[1].tech.resize(0x41, false);
        assert_eq!(steps(&mut s, general), [1, 1], "the prerequisite not held");
        s.tech[1].tech[0x40] = true;
        assert_eq!(steps(&mut s, general), [2, 2], "held: the General");
        assert_eq!(steps(&mut s, spy), [2, 2], "held: the Spy");
        assert_eq!(steps(&mut s, filler), [1, 1], "held: a caster of no line");
    }

    /// **A French General takes back two a frame** (`docs/AI.md` §119,
    /// item 1470): `Unit::process@00610bc0`'s `r` is 4 under
    /// `has_tribe_bonus(10)` and `is(GENERAL, 1)`, and `((frame & 1) + r)
    /// / 2` is 2 on either parity; anyone else's `r = 2` is 1 on either.
    /// run632's `1/79`, a French General, 102 on 11376 and 64 on 11395.
    /// Memnon's `MEMNON_REGEN_RATE` 2/1 (512) doubles his `r` to 4, and a
    /// modded 3/2 (384) takes it to 3: one and two by parity. A French
    /// General's pool is `(FRENCH_SPECIAL_CRAFT + 100) × MANA / 100`.
    ///
    /// Made to fail with the craft rate back at 2 for everyone, and with
    /// the bonus on the Spy (`is(GENERAL, 1)` alone gates it).
    #[test]
    fn a_french_general_recovers_craft_twice_as_fast() {
        let (mut s, [general, spy, memnon, filler]) = casters(Tuning::RON);
        assert_eq!(steps(&mut s, general), [1, 1], "no nation power");
        s.tech[1].power = Some(crate::nations::power::FRENCH);
        assert_eq!(steps(&mut s, general), [2, 2], "the French General");
        assert_eq!(steps(&mut s, spy), [1, 1], "a French Spy");
        assert_eq!(steps(&mut s, filler), [1, 1], "a French caster of no line");
        assert_eq!(steps(&mut s, memnon), [2, 2], "Memnon, 512 × 2 >> 8");
        assert_eq!(s.unit_mana(general), 1000, "the bonus ships as 0%");

        let mut modded = Tuning::RON;
        modded.memnon_regen_rate = 384;
        modded.french_special_craft = 20;
        let (mut s, [general, _, memnon, filler]) = casters(modded);
        assert_eq!(steps(&mut s, memnon), [1, 2], "Memnon at 3/2: r = 3");
        assert_eq!(s.unit_mana(general), 1000, "not French");
        s.tech[1].power = Some(crate::nations::power::FRENCH);
        assert_eq!(s.unit_mana(general), 1200, "(20 + 100) × 1000 / 100");
        assert_eq!(s.unit_mana(filler), 1000, "only a General's pool");
    }
}

/// [`crate::unit_update_hits_view::UnitHitsView`] on a unit type of the
/// port: the unit is its type, always a captain; the marines', Copper and
/// Bananas terms are the SEAM (no staged nation takes them) and ask false,
/// and the patch gate is taken as passed (the shipped build is past it).
struct TypeHits<'a> {
    sim: &'a Sim,
    who: Player,
    rec: usize,
    stored_hits: i32,
}

impl TypeHits<'_> {
    fn t(&self) -> &crate::UnitType {
        &self.sim.unit_types[self.rec]
    }
    fn tribe(&self, n: usize) -> bool {
        self.sim
            .tech
            .get(self.who as usize)
            .is_some_and(|p| self.sim.tech_tree.has_tribe_bonus(&self.sim.setup, p, n))
    }
}

impl crate::unit_update_hits_view::UnitHitsView for TypeHits<'_> {
    fn is_captain(&mut self) -> bool {
        true
    }
    fn redirect(&mut self) -> i32 {
        0
    }
    fn base_hits(&mut self) -> i32 {
        self.sim.type_hits(self.who, self.rec)
    }
    fn is_marine(&mut self) -> bool {
        false
    }
    fn copper_bit(&mut self) -> bool {
        false
    }
    fn type_218(&mut self) -> i32 {
        0
    }
    fn trainer(&mut self) -> i32 {
        self.t()
            .tree
            .and_then(|i| self.sim.tech_tree.types.get(i))
            .and_then(|d| d.where_)
            .map_or(0, |w| w as i32)
    }
    fn bananas_bit(&mut self) -> bool {
        false
    }
    fn has_tribe(&mut self, n: i32) -> bool {
        match n {
            0x12 => self.tribe(crate::ai::tribe::IROQUOIS),
            0x16 => self.tribe(crate::ai::tribe::DUTCH),
            4 => self.tribe(crate::ai::tribe::NUBIANS),
            _ => false,
        }
    }
    fn attack_word(&mut self) -> i32 {
        self.t().combat.attack as i32
    }
    fn is_type(&mut self, n: i32) -> bool {
        match n {
            0x45 => self.t().cols.is(crate::ai_load::role::SCOUT),
            0x3a => self.sim.is_spy_type(self.rec),
            _ => false,
        }
    }
    fn type_index(&mut self) -> i32 {
        self.t().type_index as i32
    }
    fn is_caravan(&mut self) -> bool {
        self.t().cols.flag2(crate::ai_load::uflags2::CARAVAN)
    }
    fn is_supply(&mut self) -> bool {
        self.t().cols.flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
    }
    fn patch_version(&mut self) -> i32 {
        9
    }
    fn is_gov_hero(&mut self) -> bool {
        self.t()
            .tree
            .and_then(|i| self.sim.tech_tree.types.get(i))
            .and_then(|d| d.kind.unit())
            .is_some_and(|u| u.patriot)
    }
    fn ages_stored(&mut self) -> i32 {
        self.sim.tech.get(self.who as usize).map_or(0, |p| p.ages) ^ 0x62766
    }
    fn spy_upgrade(&mut self) -> i32 {
        self.sim.spy_upgrade_level(self.who)
    }
    fn is_hero(&mut self) -> bool {
        self.t().cols.flag2(crate::ai_load::uflags2::GENERAL)
    }
    fn general_upgrade(&mut self) -> i32 {
        self.sim.general_upgrade_level(self.who)
    }
    fn supply_upgrade(&mut self) -> i32 {
        self.sim.supply_upgrade_level(self.who).clamp(0, 3)
    }
    fn link_of_self(&mut self) -> i32 {
        -1
    }
    fn link_of(&mut self, _i: i32) -> i32 {
        -1
    }
    fn store(&mut self, _at: Option<i32>, h: i32) {
        self.stored_hits = h;
    }
    fn stored(&mut self) -> i32 {
        self.stored_hits
    }
}
