//! [`DoAttackView`] over the live [`crate::Sim`].
//!
//! SEAMs, each named where it bites: the cargo chain is the units whose
//! `inside_unit` is this one, in index order (the original walks
//! `inside_down`, whose order is the order of entry), and a carrier's strafe
//! has no home to name (a strafe's home is a building here);
//! `find_attack_pos`'s `strict` flag is not modelled, so the barge's first
//! ask is the crate's and its second answers nothing; `get_tregion` is the
//! crate's, without the original's coastal refinement; gun tracking and the
//! cavalry archer's range override write fields this crate does not carry.

use super::{DoAttackView, do_attack_steps};
use crate::combat::Obj;

struct SimDoAttack<'a> {
    s: &'a mut crate::Sim,
    u: usize,
    frame: i64,
    target: Option<Obj>,
    cargo: Vec<usize>,
    at: usize,
}

impl SimDoAttack<'_> {
    fn riders(&self) -> Vec<usize> {
        (0..self.s.units.len())
            .filter(|&i| self.s.units[i].inside_unit == Some(self.u))
            .collect()
    }
}

impl DoAttackView for SimDoAttack<'_> {
    fn is_carrier(&mut self) -> bool {
        self.s.units[self.u]
            .ty
            .is_some_and(|t| self.s.is_aircraft_carrier(t))
    }
    fn target_alive(&mut self) -> bool {
        match self.target {
            Some(Obj::Unit(t)) => self.s.units[t].alive(),
            Some(Obj::Building(b)) => self.s.buildings[b].alive,
            None => false,
        }
    }
    fn frame_off_phase(&mut self) -> bool {
        self.frame & 0xf != 0
    }
    fn target_is_building(&mut self) -> bool {
        matches!(self.target, Some(Obj::Building(_)))
    }
    fn mandatory(&mut self) -> bool {
        self.s.units[self.u].combat.mandatory
    }
    fn cargo_start(&mut self) -> bool {
        self.cargo = self.riders();
        self.at = 0;
        !self.cargo.is_empty()
    }
    fn strafe_cargo(&mut self) {
        let r = self.cargo[self.at];
        let mandatory = self.s.units[self.u].combat.mandatory;
        self.s.add_strafe_order(
            r,
            self.target,
            None,
            mandatory,
            crate::orders::QueuePos::New,
            true,
        );
    }
    fn cargo_next(&mut self) -> bool {
        self.at += 1;
        self.at < self.cargo.len()
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn attack(&mut self) -> i32 {
        self.s.profile(Obj::Unit(self.u)).attack
    }
    fn is_barge(&mut self) -> bool {
        self.s.unit_is(self.u, crate::transport::ty::TRANSPORTBARGE)
    }
    fn passenger_target_valid(&mut self) -> bool {
        let (Some(&p), Some(t)) = (self.riders().first(), self.target) else {
            return false;
        };
        self.s.valid_target(Obj::Unit(p), t)
    }
    fn passenger_attack_pos(&mut self, strict: bool) -> Option<(i32, i32)> {
        if strict {
            return None;
        }
        let (&p, t) = (self.riders().first()?, self.target?);
        let here = self.s.units[self.u].pos;
        self.s
            .find_attack_pos(p, t, here, crate::fight::SITE_ATTACK_POS_FIGHT)
            .map(|p| (p.x, p.y))
    }
    fn target_pos(&mut self) -> (i32, i32) {
        let p = self
            .target
            .map_or(self.s.units[self.u].pos, |t| self.s.pos_of(t));
        (p.x, p.y)
    }
    fn move_first(&mut self, x: i32, y: i32, action: bool) {
        self.s.add_move_order(
            self.u,
            crate::Pos::new(x, y),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            action,
        );
    }
    fn group(&mut self) -> i32 {
        self.s.units[self.u].group_ptr.map_or(-1, i32::from)
    }
    fn set_group(&mut self, group: i32) {
        self.s.units[self.u].group_ptr = u8::try_from(group).ok();
    }
    fn is_worker(&mut self) -> bool {
        self.s.is_worker(self.u)
    }
    fn group_max_range(&mut self, worker: bool) -> i32 {
        // `GroupData::get_max_range@00711080`: the longest `max_range` over
        // the members that are alive, on the map, no aircraft and a worker
        // exactly when asked for one, whose type attacks. The crate has no
        // method of its own (that function's sweep is an inline copy); this
        // is that reading over the pool's live members.
        let Some(slot) = self.s.units[self.u].group_ptr else {
            return 0;
        };
        let who = self.s.units[self.u].owner;
        let mut best = 0;
        for m in self.s.pool_members(who, slot) {
            let unit = &self.s.units[m];
            let attacks = unit
                .ty
                .is_some_and(|t| self.s.unit_types[t].combat.attack != 0);
            if unit.alive()
                && unit.on_map
                && self.s.group_domain(m) != crate::attrition::Domain::Air
                && self.s.is_worker(m) == worker
                && attacks
            {
                best = best.max(self.s.max_range_of(Obj::Unit(m)));
            }
        }
        best
    }
    fn is_supply(&mut self) -> bool {
        self.s.units[self.u]
            .ty
            .is_some_and(|t| self.s.unit_types[t].cols.unit_flags2 & 0x40 != 0)
    }
    fn target_set(&mut self) -> bool {
        self.target.is_some()
    }
    fn target_footprint(&mut self) -> i32 {
        match self.target {
            Some(Obj::Building(b)) => self.s.buildings[b].ty.map_or(0, |t| {
                self.s.build_types[t].x_size + self.s.build_types[t].y_size
            }),
            _ => 0,
        }
    }
    fn pos(&mut self) -> (i32, i32) {
        let p = self.s.units[self.u].pos;
        (p.x, p.y)
    }
    fn find_nearby_spot(&mut self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.s
            .find_nearby_spot(
                self.u,
                crate::Pos::new(x, y),
                0,
                -1,
                0,
                crate::movement::Angle(0x5555_5555),
                None,
            )
            .map(|p| (p.x, p.y))
    }
    fn spot_valid(&mut self, x: i32, y: i32) -> bool {
        let units = |cells: i32| cells * crate::world::TILES_PER_CELL * 0xc0;
        x >= 0 && y >= 0 && x < units(self.s.world.width()) && y < units(self.s.world.height())
    }
    fn div3(&mut self, i: i32) -> i32 {
        i / 3
    }
    fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
        self.s
            .world
            .tregion(crate::Pos::new(tx, ty))
            .map_or(-1, i32::from)
    }
    fn unit_flags(&mut self) -> u32 {
        self.s.units[self.u]
            .ty
            .map_or(0, |t| self.s.unit_types[t].cols.unit_flags)
    }
    fn track_target(&mut self) {}
    fn stance(&mut self) -> i32 {
        if self.s.units[self.u].combat.stance == crate::combat::Stance::StandGround {
            2
        } else {
            0
        }
    }
    /// `do_attack@005f1b80:245`–`251`: a melee-and-ranged type at stance 2
    /// fights at its second range, the bracket [`crate::Sim::cavarch_fight`]
    /// also puts round its own (item 3383).
    fn fight_cavarch_range(&mut self, _mandatory: bool) -> bool {
        let (u, frame) = (self.u, self.frame);
        self.s.with_second_range(u, |s| s.do_attack_fight(u, frame));
        false
    }
    fn fight(&mut self, _mandatory: bool) -> bool {
        self.s.do_attack_fight(self.u, self.frame);
        false
    }
    fn recharge_byte(&mut self) -> u8 {
        self.s.units[self.u].combat.recharging
    }
    fn set_recharge_byte(&mut self, b: u8) {
        self.s.units[self.u].combat.recharging = b;
    }
    fn cavarch_fight(&mut self) {}
}

impl crate::Sim {
    /// `Unit::do_attack@005f1b80` — [`do_attack_steps`] over [`SimDoAttack`];
    /// the `fight` half is [`Self::do_attack_fight`].
    pub(crate) fn do_attack(&mut self, u: usize, frame: i64) {
        if !matches!(
            self.current_order(u).map(|o| &o.body),
            Some(crate::orders::Body::Attack(_))
        ) {
            return;
        }
        let target = self.units[u].combat.target;
        do_attack_steps(&mut SimDoAttack {
            s: self,
            u,
            frame,
            target,
            cargo: Vec::new(),
            at: 0,
        });
    }
}
