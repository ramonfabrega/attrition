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
    /// `UnitData::mana@00609a50` — the type's `MANA`. SEAM: the supply
    /// upgrade's multiple, the French craft bonus and the space-air arm;
    /// none is reached by a Spy of a nation without them.
    pub(crate) fn unit_mana(&self, u: usize) -> i32 {
        self.units[u].ty.map_or(0, |t| self.unit_types[t].mana)
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
    /// SEAM: the tribe and graft substitutions, and every craft's own case
    /// below the head; the Spy's three have none.
    pub(crate) fn spell_castable(&self, s: i32, u: usize) -> bool {
        let Some(d) = self.spell(s) else {
            return false;
        };
        d.from.iter().flatten().any(|&t| self.unit_line_is(u, t))
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
        let Some(t) = target else {
            return 0;
        };
        // `validate_spell`'s `DOUBLE_AGENT` arm: an object already
        // infiltrated by the caster's player refuses the Informer.
        if s == spell::INFORMER && self.infiltrated_of(t) & Self::who_bit(g.who) != 0 {
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
            if !self.spell_valid_target(s, g.who, t) {
                continue;
            }
            // `is_busy@0060a370`: a head cast (group.rs's reading).
            if matches!(self.current_order(u).map(|o| o.body), Some(Body::Cast(_))) {
                continue;
            }
            if self.units[u].cast_target != Some(t) {
                self.units[u].spell_time = 0;
                self.units[u].cast_target = Some(t);
            }
            let pos = if d.flags & craft::FIRST != 0 {
                while matches!(self.current_order(u).map(|o| o.body), Some(Body::Cast(_))) {
                    self.kill_current_order(u);
                }
                QueuePos::First
            } else {
                QueuePos::New
            };
            self.add_cast_order_on(u, s, Some(t), at, pos, true);
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
        self.kill_current_order(u);
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

    /// Is the object still standing?
    fn obj_alive(&self, t: Obj) -> bool {
        match t {
            Obj::Unit(v) => self.units.get(v).is_some_and(|x| x.alive()),
            Obj::Building(b) => self.buildings.get(b).is_some_and(|x| x.alive),
        }
    }

    /// `Unit::process@00610bc0`'s caster arm: one point of craft back a
    /// frame, `(frame & 1 + 2) / 2`, while `unit_masks & 0x2a000` is clear.
    ///
    /// SEAM: the French craft bonus, `SPIES_GENERALS_RECOVER_CRAFT`,
    /// Memnon's rate, the supply wagon's gate, and the two other bits of
    /// the mask (`0x2000`, `0x8000`), none of which a capture holds.
    pub(crate) fn recover_mana(&mut self, u: usize, frame: i64) {
        if !self.units[u].decoy && self.unit_domain_of(u) == crate::attrition::Domain::Air {
            self.burn_fuel(u);
            return;
        }
        let unit = &self.units[u];
        if unit.mana_burn == 0 || unit.decoy || unit.casting {
            return;
        }
        let step = i16::try_from(((frame & 1) + 2) / 2).unwrap_or(1);
        let unit = &mut self.units[u];
        unit.mana_burn -= step.min(unit.mana_burn);
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
