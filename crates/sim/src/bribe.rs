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
//!     per guy: stopped, x, y, z, angle, turrets copied; update_gpiece
//!   the old squad closed, captain first, until a closed figure is a captain
//! ```
//!
//! SEAM: the `inside_down` chain (`+0x28`, units carried inside the target,
//! each re-created and `go_inside`d), which a Land unit that Bribe admits
//! does not hold; the `unit_masks` bits this crate keeps as separate
//! fields are copied only for `in_danger`, `line_ok`, `casting` and
//! `marching`; the cast's messages and sounds (`S_SPY_OURS_DONE_BRIBE`).

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
    }

    /// One figure's copy from the old unit `o` to the new one `m`
    /// (`swap_team`'s do-while body).
    fn copy_figure(&mut self, o: usize, m: usize) {
        let old = &self.units[o];
        let damage = old.max_health - old.health;
        let (frac, facing, form, recharging) = (
            old.damage_frac,
            old.movement.facing,
            old.form,
            old.combat.recharging,
        );
        let (in_danger, line_ok, casting, marching) =
            (old.in_danger, old.line_ok, old.casting, old.marching);
        let (on_map, at) = (old.on_map, old.pos);
        let guys: Vec<_> = old.guys.iter().map(|g| (g.stopped, g.turret)).collect();
        let n = &mut self.units[m];
        n.health = n.max_health - damage;
        n.damage_frac = frac;
        n.movement.facing = facing;
        n.form = form;
        n.combat.recharging = recharging;
        n.in_danger = in_danger;
        n.line_ok = line_ok;
        n.casting = casting;
        n.marching = marching;
        for (g, (stopped, turret)) in n.guys.iter_mut().zip(guys) {
            g.stopped = stopped;
            g.turret = turret;
        }
        if on_map {
            self.set_new_location(m, at, true);
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
