//! The helpers of `docs/ARMY.md` §14 that move the army's group:
//! `Army::send_here` and `Army::charge`, and the claim `send_here` shares
//! with `do_forming` and `march_to_target` (item 4395). Moved here from
//! `ai/army.rs` by item 4395, whose claim grew that file past the
//! structure pin.

use crate::combat::Obj;
use crate::movement::{Angle, find_angle};
use crate::orders::{MoveKind, QueuePos};
use crate::world::Pos;
use crate::{Player, Sim};

impl Sim {
    /// `Army::send_here(x, y, order)`: the point and the muster cell move,
    /// and every group walks there.
    pub fn send_here(&mut self, who: Player, slot: usize, p: Pos, kind: MoveKind) {
        let w = who as usize;
        let p = self.restrict(p);
        let here = self.ai.armies[w].list[slot].pos;
        if p.x != here.x && p.y != here.y {
            self.ai.armies[w].list[slot].muster_angle = find_angle(p.x - here.x, p.y - here.y);
        }
        {
            let a = &mut self.ai.armies[w].list[slot];
            a.pos = p;
            a.muster = p.cell();
        }
        // `send_here@006f98a0:48`: each populated group's record takes the
        // army's id before the move ([`Army::adopted`]).
        if !self.ai.armies[w].list[slot].units.is_empty() {
            self.army_claim_record(who, slot);
        }
        let ang = self.ai.armies[w].list[slot].muster_angle;
        let g = self.army_group(who, slot);
        self.group_action_move_to(&g, p, QueuePos::New, true, ang, kind, true);
    }

    /// **A group record takes its army's id** (item 4395): `do_forming`,
    /// `march_to_target` and `send_here` each write the army's own id
    /// (`+2`) into a populated group record's `army` (`+8`), so an
    /// adopted record ([`Army::adopted`]) is the army's again and
    /// `Object::get_army@00649d70` answers it.
    pub(crate) fn army_claim_record(&mut self, who: Player, slot: usize) {
        self.ai.armies[who as usize].list[slot].adopted = false;
    }

    /// `Army::charge(o, who)`: a siege unit under attack drags its army
    /// onto the attacker.
    pub fn army_charge(&mut self, who: Player, slot: usize, target: Obj) {
        if !self.ai.armies[who as usize].list[slot].valid {
            return;
        }
        let p = self.pos_of(target);
        let g = self.army_group(who, slot);
        self.group_action_move_to(
            &g,
            p,
            QueuePos::First,
            false,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        self.ai.armies[who as usize].list[slot].target = Some(target);
    }
}
