//! Bribe's conversion — `SpellType::cast_bribe@00672d00`'s unit arm and
//! `Unit::swap_team@006118d0` (item 2823, `docs/GOLDEN.md` §107).
//!
//! The target is not handed over: a **copy** is born under the caster's
//! owner (`Objects::init_unit` of `LeaderData::current_upgrade` of the old
//! type, at the old position), the figures' state is copied across one by
//! one, and the old squad is closed through `Unit::close(0, −1, 0)` — no
//! death roll, thirty frames' hold, its population handed back.
//!
//! ```text
//! cast_bribe, unit arm:   leader(caster) +0x8f4 += 1 ; target.swap_team(caster)
//! swap_team(this, a):
//!   not a captain          -> the captain's swap_team(a)
//!   new = init_unit(a, current_upgrade(a, this.type), this.pos)
//!   per figure, old and new walked down `o_down` together:
//!     damage, damage_frac, angle, form, recharging, guy_mark copied;
//!     unit_masks = new & 0x40000 | old & ~0x40000   (the AI bit stays new's)
//!     flags: active/on-map from old; off the map -> remove_from_world,
//!            else set_new_location(old pos)
//!     per guy: stopped, x, y, z, angle, turret angles copied (not
//!              node_flags / des_node_flags); update_gpiece
//!   the old squad closed, captain first, until a closed figure is a captain
//! ```
//!
//! **The two words are copied whole** (item 4735, `docs/COMBAT.md` §91):
//! `unit_masks` (`+0x68`) but the AI bit, and the `flags` byte (`+0x8`) but
//! `0x40`, the detector bit, which stays the new type's. Every bit this
//! crate keeps as its own field rides the copy, so the old squad's attrition
//! mark (`0x80`, without its period), `blown` and `fought` (`0x11000`) and
//! `cant_reach` (`flags & 0x10`) reach the new one. run1797's Iroquois
//! Assault Infantry, bribed on 3738 from `1/86` to `2/122`..`124`, print
//! `unit_masks` 0x15108c and `flags` 0x11 on 3739; the copy's `blown` is
//! what keeps the Iroquois heal (`!(unit_masks & 0x1000)`) off its captain.
//!
//! SEAM: the `inside_down` chain (`+0x28`, units carried inside the target,
//! each re-created and `go_inside`d), which a Land unit that Bribe admits
//! does not hold; `unit_masks & 0x2000`, whose field
//! ([`crate::Unit::ambushing`]) carries a payload the word does not; the
//! cast's messages and sounds (`S_SPY_OURS_DONE_BRIBE`).

use crate::{Player, Sim};

impl Sim {
    /// `SpellType::cast_bribe`'s unit arm, past its refusals: the caster's
    /// leader counts one (`+0x8f4`) and the target is swapped to the
    /// caster's owner. Returns the new captain's index.
    pub(crate) fn cast_bribe_unit(&mut self, caster: usize, target: usize) -> Option<usize> {
        let a = self.units[caster].owner;
        if self.units[target].owner == a {
            return None;
        }
        self.tech[a as usize].bribes += 1;
        self.unit_swap_team(target, a)
    }

    /// `Unit::swap_team@006118d0`: see the module. `None` when the type has
    /// no record for `a` to build.
    pub(crate) fn unit_swap_team(&mut self, u: usize, a: Player) -> Option<usize> {
        // A figure that is not a captain hands the swap to the captain.
        let mut head = u;
        while let Some(up) = self.units[head].o_up {
            head = up;
        }
        let rec = self
            .upgrade_record_of(a, self.units[head].type_index)
            .or(self.units[head].ty)?;
        let pos = self.units[head].pos;
        let n = self.init_unit(a, rec, pos);
        // The squads walked together, figure by figure.
        let (mut old, mut new) = (Some(head), Some(n));
        let mut olds = Vec::new();
        while let (Some(o), Some(m)) = (old, new) {
            olds.push(o);
            self.copy_figure(o, m);
            old = self.units[o].o_down;
            new = self.units[m].o_down;
        }
        // `Unit::close` on the captain, and on whoever leads after it,
        // until the one closed is a captain once its relink has run.
        let mut cur = head;
        for _ in 0..self.units.len() {
            self.units[cur].health = self.units[cur].health.min(0);
            self.close_count(cur);
            self.relink_squad(cur);
            self.close_dead_orders(cur);
            self.units[cur].hold_frames = Self::CLOSE_HOLD;
            self.close_supply(cur);
            self.forget(crate::combat::Obj::Unit(cur));
            if self.units[cur].o_up.is_none() {
                break;
            }
            let mut c = cur;
            while let Some(up) = self.units[c].o_up {
                c = up;
            }
            cur = c;
        }
        debug_assert!(!olds.is_empty());
        Some(n)
    } // twin of Unit::swap_team

    /// One figure's copy from the old unit `o` to the new one `m`
    /// (`swap_team`'s do-while body).
    fn copy_figure(&mut self, o: usize, m: usize) {
        let old = &self.units[o];
        let damage = old.max_health - old.health;
        // `swap_team` copies two angles, each from its own owner: the
        // unit's `angle` (`UnitData +0x50`, this crate's `heading`) and the
        // figure's `angle` (`GuyData +0x18`, this crate's `facing`); the
        // figure's `des_angle` (`+0x64`) is not copied. They differ on a
        // figure still turning, run1554's `1/7` on 1293 (the figure's
        // −420955942 against the unit's −118161408), and the copy then
        // finishes that turn on its birth frame: `Guy::move`'s owed-turn
        // arm puts it on the walk, and the next frame's `do_idle` rolls
        // it back (item 3999, `docs/GOLDEN.md` §189).
        let (frac, facing, heading, form, recharging) = (
            old.damage_frac,
            old.movement.facing,
            old.movement.heading,
            old.form,
            old.combat.recharging,
        );
        // `unit_masks = new & 0x40000 | old & !0x40000`: every bit but the
        // AI's, each a field here (item 4735). `& 2` (the figure's mirror)
        // and `& 0x100000` (the half step) are run1543's bribed Hoplites,
        // 0x14000a old and 0x10000a new on 1294 (item 3985).
        let masks = UnitMasks::of(old);
        // `flags = new & 0x40 | old & 0xbf`: the detector bit is the type's,
        // the rest the old figure's (item 4735).
        let (idle_latch, cant_reach, attacking) = (old.idle_latch, old.cant_reach, old.attacking);
        let (on_map, at, active, body) = (old.on_map, old.pos, old.alive(), old.movement.body.pos);
        let guys: Vec<_> = old.guys.iter().map(|g| (g.stopped, g.turret)).collect();
        let n = &mut self.units[m];
        n.health = n.max_health - damage;
        n.damage_frac = frac;
        n.movement.facing = facing;
        // The copy is no `Guy::do_turn`: `guy_flags` (`+0x9a`) is outside
        // the copied range, so the fresh figure's `& 2` stays clear and its
        // first `Guy::move` turns it ([`crate::Movement::frame_facing`]).
        n.movement.frame_facing = facing;
        n.movement.heading = heading;
        n.form = form;
        n.combat.recharging = recharging;
        masks.onto(n);
        n.idle_latch = idle_latch;
        n.cant_reach = cant_reach;
        n.attacking = attacking;
        // **The turret's angles ride the copy, its node flags do not**
        // (item 4758, `docs/RECGAME.md` §13.5): the per-guy loop
        // (`006118d0:84`–`105`) copies `+0x20..+0x3c` (`turret_angles`,
        // `des_turret_angles`) and `+0x40` (`turret_inc`), and never
        // `+0x96`/`+0x98`, so the copy keeps the `node_flags` and
        // `des_node_flags` its own `Unit::init` left it. run1797's BMP
        // `2/50`, bribed on 4058, prints 0 and 0 on 4059 where the old
        // figure held 15 and 1.
        for (g, (stopped, turret)) in n.guys.iter_mut().zip(guys) {
            g.stopped = stopped;
            g.turret.angles = turret.angles;
            g.turret.des = turret.des;
            g.turret.inc = turret.inc;
        }
        // **A dead figure's copy leaves the world where it was born** (item
        // 4769, `docs/RECGAME.md` §12.3). The branch reads the copy's
        // `flags & 1` (`SubObjectData::is_active`), just taken from the old
        // figure (`006118d0:63`), at `:72`: clear, and the copy takes
        // `Object::remove_from_world@00647970`, never `set_new_location`. Its
        // disc walk is gated on that same bit and does not run; its chain
        // half unthreads the copy, whose unit point stays `init_unit`'s. The
        // per-guy loop still copies the figure's point (`+0xc`..`+0x14`).
        // run1784's 4469: who=2 bribes a Riflemen squad whose third figure
        // is dead, and the copy `2/41` stood threaded on its birth cell above
        // `2/39`, `up` ours 41, theirs −1 on 4470.
        if !active {
            self.chain_remove(m);
            self.units[m].movement.body.pos = body;
        } else if on_map {
            // **`set_new_location(…, 1, 1)`, the turn included** (item 4769):
            // its `param_4` arm snaps guy 0's `angle` and `last_angle` to
            // the unit's `angle`, just copied (`+0x50`). The per-guy loop
            // runs after it and copies the old figure's `angle` (`+0x18`)
            // back, but not `last_angle` (`+0x1c`). run1784's 4470: the
            // copy `2/40`'s `last_angle`, ours 0x55555555 (init's), theirs
            // −1191444480, the unit's `angle`.
            self.set_new_location_turning(m, at, true);
            let mv = &mut self.units[m].movement;
            mv.facing = facing;
            mv.frame_facing = facing;
        } else {
            self.units[m].on_map = false;
        }
        self.update_gpiece(m);
    }

    /// `LeaderData::current_upgrade(a, type)` as a unit record.
    fn upgrade_record_of(&self, a: Player, ti: i32) -> Option<usize> {
        self.upgrade_record(a, ti)
    }
}

/// `UnitData::unit_masks` (`+0x68`) as this crate holds it, bit by field,
/// but the AI bit (`0x40000`, the owner's) and `0x2000` (see the module's
/// `SEAM`): the word `Unit::swap_team@006118d0` copies (item 4735).
struct UnitMasks {
    carry: u32,
    bits: [bool; 23],
    marching: Option<i64>,
}

impl UnitMasks {
    fn of(u: &crate::Unit) -> Self {
        Self {
            carry: u.carry,
            bits: [
                u.decoy,
                u.movement.mirror,
                u.in_danger,
                u.line_ok,
                u.contact,
                u.gather_here,
                u.waiting_on,
                u.attrition_mark,
                u.auto_explore,
                u.trade_unloaded,
                u.was_builder,
                u.cloak,
                u.blown,
                u.slay_marked,
                u.fought,
                u.casting,
                u.combat.packed,
                u.half_step,
                u.ignores_supply,
                u.auto_transport,
                u.combat.entrenched,
                u.explore_queued,
                u.jam_shown,
            ],
            marching: u.marching,
        }
    }

    fn onto(&self, u: &mut crate::Unit) {
        let [
            decoy,
            mirror,
            in_danger,
            line_ok,
            contact,
            gather_here,
            waiting_on,
            attrition_mark,
            auto_explore,
            trade_unloaded,
            was_builder,
            cloak,
            blown,
            slay_marked,
            fought,
            casting,
            packed,
            half_step,
            ignores_supply,
            auto_transport,
            entrenched,
            explore_queued,
            jam_shown,
        ] = self.bits;
        u.carry = self.carry;
        u.decoy = decoy;
        u.movement.mirror = mirror;
        u.in_danger = in_danger;
        u.line_ok = line_ok;
        u.contact = contact;
        u.gather_here = gather_here;
        u.waiting_on = waiting_on;
        u.attrition_mark = attrition_mark;
        u.auto_explore = auto_explore;
        u.trade_unloaded = trade_unloaded;
        u.was_builder = was_builder;
        u.cloak = cloak;
        u.blown = blown;
        u.slay_marked = slay_marked;
        u.fought = fought;
        u.casting = casting;
        u.combat.packed = packed;
        u.half_step = half_step;
        u.ignores_supply = ignores_supply;
        u.auto_transport = auto_transport;
        u.combat.entrenched = entrenched;
        u.explore_queued = explore_queued;
        u.jam_shown = jam_shown;
        u.marching = self.marching;
    }
}
