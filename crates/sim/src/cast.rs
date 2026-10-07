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
/// `LeaderData::get_general_upgrade`, which no staged leader raises.
const GENERAL_UPGRADE: i32 = 0;
/// `TypeIndex` rows [`Sim::craft_rate`] asks (`enums/TypeIndex.txt`).
const GENERAL: crate::tech::TypeId = 0x36;
const SPY: crate::tech::TypeId = 0x3a;
const MEMNON: crate::tech::TypeId = 0x16c;
/// A Conquer the World bonus, which no staged leader holds.
const SPIES_GENERALS_RECOVER_CRAFT: crate::tech::TypeId = 0x301;

/// `SpellTypeData::spell_flags` letters the cast reads, as bits `a`..`m`.
pub mod craft {
    /// `b`, `c`, `d`: a unit, a building, an area — the targeted craft.
    pub const TARGETED: u32 = 0xe;
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
    /// SEAM: the other terms — the American marines, Copper and Bananas,
    /// the Iroquois, the Dutch, the Spy and General upgrades —
    /// which no staged nation or holding takes.
    pub fn unit_hits(&self, who: Player, rec: usize) -> i32 {
        let mut hits = self.type_hits(who, rec);
        let t = &self.unit_types[rec];
        let trader = matches!(t.type_index, 0x3d | 0x3e | 400)
            || t.cols.flag2(crate::ai_load::uflags2::CARAVAN);
        if trader
            && self.tuning.nubian_hit_points != 0
            && self.tech.get(who as usize).is_some_and(|p| {
                self.tech_tree
                    .has_tribe_bonus(&self.setup, p, crate::ai::tribe::NUBIANS)
            })
        {
            hits = (self.tuning.nubian_hit_points + 100) * hits / 100;
        }
        // The last term (`0060ecad`): a supply unit adds
        // `supply_hp_upgrade[get_supply_upgrade]`, the count of the three
        // upgrade prerequisites its leader holds (`docs/AI.md` §157).
        if t.cols.flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
            && !t.cols.flag2(crate::ai_load::uflags2::GENERAL)
        {
            let level = self.supply_upgrade_level(who).clamp(0, 3) as usize;
            hits += self.tuning.supply_hp_upgrade[level];
        }
        hits
    }

    /// A gain of a Militia-line bit re-reads every Citizen's `myhits`, as
    /// `update_hits` would: the damage is kept, the pool is the new one
    /// (run422's `0/1`..`0/5` at 50 on block 605, the block after the line).
    pub(crate) fn refresh_citizen_hits(&mut self, who: Player) {
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() || !matches!(unit.type_index, PEASANTS | 0x33) {
                continue;
            }
            let Some(rec) = unit.ty else { continue };
            let hits = self.unit_hits(who, rec);
            let damage = self.units[u].max_health - self.units[u].health;
            self.units[u].max_health = hits;
            self.units[u].health = hits - damage;
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
    /// SEAM: the unit mask arms (`0x100`/`0x200` against objmask
    /// `0x200000`), which none of the Spy's crafts carries; and the whole
    /// of Bribe's, Counterintelligence's, Pilfer's and Sabotage's own
    /// arms, which **refuse** here rather than be guessed — run246's packet
    /// has the first two refuse the staged Barracks.
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
            spell::BRIBE | spell::COUNTERINTEL | 0x276 | 0x280 | 0x281 => false,
            _ => true,
        }
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
    /// caster stands in the target owner's fog; the cloak (`0x11000`) for a
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
        // The spy's animations (`is(SPY, 0)`): Bribe `CHAR_ATTACK3`, the
        // Informer `CHAR_ATTACKWALK`, anything else `CHAR_ATTACK2` on a
        // spy and `CHAR_ATTACK1` on anything else.
        let anim = if !self.unit_is_spy(u) {
            crate::anim::ATTACK1
        } else if s == spell::BRIBE {
            crate::anim::ATTACK3
        } else if s == spell::INFORMER {
            crate::anim::ATTACKWALK
        } else if matches!(t, Obj::Unit(v) if self.unit_is_spy(v)) {
            crate::anim::ATTACK2
        } else {
            crate::anim::ATTACK1
        };
        self.set_anim(u, anim, false, false);
        self.units[u].casting = true;
        self.units[u].spell_time += 1;
        if self.units[u].spell_time < self.spell_job_time(s) {
            return;
        }
        self.units[u].spell_time = 0;
        self.cast_on(u, s, t);
        self.clear_cast_paid(u);
        self.kill_current_order(u);
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
    /// SEAM: `general_upgrade` (0: `get_general_upgrade` counts the three
    /// `GENERALS_UPGRADE_n` prerequisites, none held here); Porus's and
    /// Kutosov's multiples; the leader's other two counters
    /// (`+0x93c`, `+0x808`); `is_caravan` as the two caravan ids; the sound.
    pub(crate) fn cast_create_decoy(&mut self, g: usize) {
        let who = self.units[g].owner;
        let limit = (GENERAL_UPGRADE + 2) * 5;
        let mut made = self
            .units
            .iter()
            .filter(|x| x.alive() && x.owner == who && x.captain && x.decoy)
            .count() as i32;
        // The caster may be a patriot: its own hero radius includes the
        // subtype's bonus (GOLDEN §60, run588's President).
        let radius = self.hero_radius(g) * 0xc0;
        let centre = self.units[g].pos;
        // `unit_masks & 0x40000` (a computer's unit): the General's army,
        // which every copy joins (`Army::add_unit`, the call at `674712`) before its
        // figures are marked. run346's `1/98` on 11637: army 4's group 70
        // lists 37, the six squads after the nineteen (item 1302).
        let army = if self.ai_driven(who) {
            self.army_of(g)
        } else {
            None
        };
        let mut list: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].owner == who)
            .collect();
        list.sort_by_key(|&u| std::cmp::Reverse(self.units[u].index));
        let mut any = false;
        for u in list {
            let unit = &self.units[u];
            if !unit.alive() || !unit.captain || unit.decoy || u == g {
                continue;
            }
            let Some(ty) = unit.ty else { continue };
            if self.unit_domain_of(u) != crate::attrition::Domain::Land
                || self.profile(Obj::Unit(u)).attack == 0
                || (PEASANTS..=0x35).contains(&unit.type_index)
                || matches!(unit.type_index, 0x3b | 0x3c)
                || self.is_merchant(u)
            {
                continue;
            }
            let d = crate::world::vector_dist(unit.pos.x - centre.x, unit.pos.y - centre.y);
            if d >= radius {
                continue;
            }
            let Some(spot) = self.find_nearby_spot_type_for(
                self.decoy_search_type(g, ty),
                g,
                centre,
                0x180,
                -1,
                crate::movement::Angle(0),
            ) else {
                break;
            };
            let head = self.init_unit(who, ty, spot);
            if let Some(slot) = army {
                self.army_add_unit(who, slot, head);
            }
            any = true;
            if self.unit_types[ty].price.pop != 0 {
                self.track_unit_type(who, ty, -1);
            }
            let mut at = Some(head);
            while let Some(x) = at {
                self.units[x].decoy = true;
                self.units[x].mana_burn = 0;
                // `cast_create_decoy@00674370`'s `update_los`: term 12's
                // one tile, on the cache.
                self.update_los(x);
                at = self.units[x].o_down;
            }
            made += 1;
            if made >= limit {
                break;
            }
        }
        if !any && let Some(d) = self.spell(spell::CREATE_DECOY) {
            let back = i16::try_from(d.mana).unwrap_or(i16::MAX);
            self.units[g].mana_burn -= back.min(self.units[g].mana_burn);
        }
    }

    /// The type the decoy's spot search is **asked with**: the caster's
    /// own (`find_nearby_spot@0061de70` reads `this->+0x240/+0x244` for its
    /// block radius and its default span; listing `674666..6746b9` loads
    /// `this` from the caster `param_1`), while the copy itself is made
    /// with the source's type (`6746d1..6746ed`) — twenty-fourth pass,
    /// group 11; A6 row 45. A caster with no type falls back to the source.
    fn decoy_search_type(&self, caster: usize, source: usize) -> usize {
        self.units[caster].ty.unwrap_or(source)
    }

    /// `LeaderData::current_upgrade(who, ti)` as a unit record.
    fn upgrade_record(&self, who: Player, ti: i32) -> Option<usize> {
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
    /// SEAM: the supply wagon's gate, and the mask's `0x2000`, which no
    /// capture holds (scan: `grep -aoE 'unit_masks -?[0-9]+'
    /// gamelog-run*.txt`, every distinct value tested for `0x2000`: none,
    /// over the 306 of 311 dumps that print the field; item 1470).
    pub(crate) fn recover_mana(&mut self, u: usize, frame: i64) {
        if !self.units[u].decoy && self.unit_domain_of(u) == crate::attrition::Domain::Air {
            self.burn_fuel(u);
            return;
        }
        // `Unit::process@00610bc0`'s other arm, `unit_masks & 1`: a decoy's
        // `mana_burn` is its age, one a frame from 0 (run422: 1 on its first
        // block), and it closes once the age reaches `(general_upgrade + 2)
        // × DECOY_TIME / 2` ([`Self::close_decoy`]). SEAM: the attrition it
        // takes every seventh frame on another's ground.
        if self.units[u].decoy {
            self.units[u].mana_burn = self.units[u].mana_burn.saturating_add(1);
            let life = (GENERAL_UPGRADE + 2) * self.tuning.decoy_time / 2;
            if life <= i32::from(self.units[u].mana_burn) {
                self.close_decoy(u);
            }
            return;
        }
        let unit = &self.units[u];
        if unit.mana_burn == 0 || unit.casting || unit.marching.is_some() {
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
            .types
            .get(SPIES_GENERALS_RECOVER_CRAFT)
            .is_some()
            && self
                .tech_tree
                .has_preq(&self.setup, &self.tech[who], SPIES_GENERALS_RECOVER_CRAFT)
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
        let end = self.frame + i64::from(d.duration + GENERAL_UPGRADE * d.duration_upgrade);
        self.units[g].marching = Some(end);
        let who = self.units[g].owner as usize;
        self.forced_march[who] = true;
    }

    /// `Caster::process_spells(o, who, 0)@00739ad0` on a hero, from the
    /// head of `Unit::process`: a spell whose end frame has passed is
    /// removed, and a Forced March's takes `unit_masks & 0x8000` with it;
    /// then `Leader::verify_spell_flags@006ce190` drops the leader's
    /// `0x8000` when none of its heroes still marches.
    pub(crate) fn process_spells(&mut self, u: usize, frame: i64) {
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
    /// SEAM: `get_general_upgrade` is [`GENERAL_UPGRADE`]; the wonder test
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
                upgrades: GENERAL_UPGRADE,
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
        if unit.marching.is_some() && self.is_hero_unit(u) {
            return true;
        }
        let (who, at) = (unit.owner, unit.pos);
        (0..self.units.len()).any(|h| {
            let x = &self.units[h];
            x.owner == who
                && x.alive()
                && x.on_map
                && x.marching.is_some()
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
    /// (`docs/MOVEMENT.md`, "The speed pipeline"): the cached `myspeed`,
    /// and a type not of the land (`+0x218 != 0`) takes nothing more.
    ///
    /// **Forced March** (`0060ad0x`–`0060ae10`): under the leader's
    /// `0x8000` and [`Sim::near_marching_hero`], the speed is
    /// `FORCED_MARCH_SPEED × UNIT_MOVE_SPEED` — 42, no shift on this arm
    /// (`0060add1`–`0060addc`) — or `myspeed` when that is larger
    /// (`cmovle`, `0060ae0e`): a march never slows a unit.
    ///
    /// SEAM: the Iroquois spear bonus in allied ground; Alexander's arm
    /// (the hero or the unit `is(0x166)`/`is(0x167)`, `× 384 >> 8`); and
    /// the hero auras below it — Spitamenes, Blucher, Porus, Charles,
    /// Napoleon — each behind a leader's power (`+0x59cc`..`+0x59e8`);
    /// none is carried.
    pub(crate) fn unit_speed(&self, u: usize) -> i32 {
        let speed = self.units[u].movement.speed;
        if self.units[u].is_gaia()
            || self.unit_domain_of(u) != crate::attrition::Domain::Land
            || !self.leader_marching(u)
            || !self.near_marching_hero(u)
        {
            return speed;
        }
        let march = self.tuning.forced_march_speed * crate::combat::UNIT_MOVE_SPEED;
        march.max(speed)
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

#[cfg(test)]
mod tests {
    use crate::tech::{TypeDef, UnitTraits};
    use crate::world::{Cell, Terrain, World};
    use crate::{Pos, Sim, Tuning, Unit, UnitType};

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

    /// What one frame of each parity takes back from a `mana_burn` of 100.
    fn steps(s: &mut Sim, u: usize) -> [i16; 2] {
        [10, 11].map(|frame| {
            s.units[u].mana_burn = 100;
            s.recover_mana(u, frame);
            100 - s.units[u].mana_burn
        })
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
