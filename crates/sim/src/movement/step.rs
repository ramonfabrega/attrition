//! The movement step: `process_movement`, the crew and the follower
//! (`docs/MOVEMENT.md`; `docs/STRUCTURE.md` §3, `movement/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// [`turning_of`] for a type this simulation holds.
    pub fn turning_for(&self, ty: usize) -> movement::Turning {
        turning_of(&self.unit_types[ty])
    }

    /// The body's frame, `Guy::move` through `Guy::process`, which
    /// `Unit::process` reaches after the order step: the body lands on where
    /// the unit *now* is. The unit step itself is the move order's
    /// (`orders::do_move`), in the same frame and before this.
    pub(crate) fn process_movement(&mut self, i: usize) {
        let (guy_mark, squad, len) = self.guy_walk_counts(i);
        let walk: Vec<usize> = anim::process_walk(guy_mark, squad, len).collect();
        self.process_movement_on(i, &walk);
    }

    /// [`Sim::process_movement`] on the figures `Guy::process` runs on, in
    /// [`anim::process_walk`]'s order.
    pub(crate) fn process_movement_on(&mut self, i: usize, walk: &[usize]) {
        let unit = &self.units[i];
        let m = unit.movement;
        let facing = m.facing;
        let heading = m.heading;
        let pos = unit.pos;
        // `Guy::move`'s animation half runs on the body as it stood before
        // the follow: a body away from its destination starts the walk, one
        // standing on it a frame after arriving goes idle (`anim.rs`).
        let was_at_des = m.body.pos == pos;
        // `Guy::move:55`'s `des_angle == angle` — the heading against the
        // facing, read before this frame's turn. It decides both the
        // arrival's animation arm and whether the crew's destination is
        // rewritten below.
        let facing_settled = facing == heading;
        // The body's rate is mode 1: the base, always. It reads `last_speed`
        // — but **a body already standing on its unit reads a zero**, because
        // `Guy::move@005d9240:53` writes `last_speed = 0` at the head of its
        // at-des branch, ahead of everything else in it and so ahead of
        // `turn_towards`. That is not bookkeeping: `GuyData::turn_speed:29`
        // returns `0x80000000` — instant — for `last_speed == 0` on a foot or
        // mounted type (`guy_flags & 0x10`), so **a standing body swallows
        // whatever turn it is owed in one frame**, however large. The frame it
        // arrives on still reads the step it just took, so the turn is one
        // frame and not two: `docs/SYNC.md` §3.11's arrival pair, whose second
        // draw the third frame would have made a third.
        let turned = facing != m.frame_facing;
        let turning = movement::Turning {
            packed: self.units[i].combat.packed,
            ..m.turning
        };
        let rate = movement::turn_speed(
            &self.tuning,
            &turning,
            if was_at_des { 0 } else { m.body.last_speed },
            m.body.avg_speed,
            movement::TurnMode::Body,
        );
        let mut follow = movement::body_follow(m.body, facing, pos, heading, turned, rate);
        if !self.world.accepts(follow.body.pos) {
            follow.body.pos = m.body.pos;
        }
        // `Guy::move:109`'s `turn_towards(des_angle, …, 1)`, which hands
        // `Guy::do_turn` the same override `move_step`'s two turn-in-place
        // arms do — so a standing guy owed a turn asks for its turn
        // animation here as well, and pays the idle roll if it has none
        // (`docs/ANIM.md` §4.8). The gate is the original's own: `guy_flags
        // & 2` clear, which is `turned`, and the body on its unit. It runs
        // inside guy 0's turn in `guys_follow`, before the crew's own
        // `Guy::move` (item 1429).
        let mut turn =
            (was_at_des && !turned && !facing_settled).then_some((facing, follow.facing, heading));
        // **Where a crew guy is told to be**, written before the figures
        // run because `Guy::move` writes it in guy 0's own step and
        // `Guy::process` runs the figures in slot order: a tracked figure
        // later in the array than guy 0 finds its destination already
        // rewritten (item 1510). The write is the one below, moved up; it
        // reads the unit's position and the frame's new facing, neither of
        // which a figure's animation touches.
        if !was_at_des || !facing_settled {
            let pos = self.units[i].pos;
            self.crew_des(i, pos, follow.facing, false);
        }
        self.guys_follow_on(i, walk, was_at_des, &mut turn);
        // A helicopter's figure height (item 1048): `Guy::move:47`'s
        // `last_z = z` at its head, and the moving arm's
        // `Guy::set_new_location(·, 0)` (`:190`) climbs it at the body's
        // new point ([`Sim::helicopter_climb`]). SEAM: `:46`'s
        // `guy_flags & 0x40` exemption, and the write for other figures.
        if self.is_helicopter(i) {
            self.units[i].airframe.last_z = self.units[i].airframe.z;
            if !was_at_des {
                self.helicopter_climb(i, follow.body.pos);
            }
        }
        // A unit with no figure to walk keeps the turn's mark where it was.
        if let Some((was, to, heading)) = turn {
            self.mark(anim::SITE_TURN_STAND);
            self.do_turn_anim(i, was, to, heading);
        }
        let unit = &mut self.units[i];
        unit.movement.facing = follow.facing;
        // **`guy_flags & 2` is cleared here, after `Guy::move`**
        // (`Guy::process@005e0230:28`), not at the head of the unit's
        // process. `Guy::do_turn@005d97a0` sets it on any turn that moves
        // the angle, whoever calls it, so a push turned by a unit processed
        // earlier in the frame keeps the pushed body from taking its own
        // turn in `Guy::move:108` this frame, and one turned later in the
        // frame keeps it from the next. East Indies' General `1/138` on
        // 4336: the Catapult `1/124` pushes it one half-rate step before
        // its own process, and the body turns no further (item 3571,
        // `docs/AI.md` §310).
        unit.movement.frame_facing = follow.facing;
        // Guy 0's `Guy::move` has turned the crew with it, or there was
        // nothing to turn ([`Movement::crew_des`]).
        unit.movement.crew_des = None;
        unit.movement.body = follow.body;
        // `Guy::move`'s `Guy::set_new_location` is what moves the
        // collision disc (`CollCheck::move_unit`, its only caller): the
        // bits follow guy 0 here, not the unit's own step
        // (`docs/COLLISION.md` §16).
        self.coll_follow(i);
        // **Where a crew guy is told to be**, and it is written twice
        // over — the two arms of `Guy::move` reach it by different
        // functions and both end in the same rotation:
        //
        // - **moving**: `Guy::set_new_location(des, 0)`, whose crew loop
        //   (`005d88da–005d89cc`) rewrites `des_angle` and `des` from guy
        //   0's own **new** position and facing. The snap flag is zero, so
        //   the crew is told where to be and not put there.
        // - **standing but still owed a turn**: `Guy::turn_towards →
        //   do_turn → Guy::set_angle(new facing, 0)`, whose crew loop
        //   (`005d90ad–005d9192`) is the same rotation about guy 0's
        //   position, with the angle it has just turned to. `do_turn` is
        //   called whether or not the facing actually moved.
        //
        // And **not written at all** on the third: a standing guy 0 whose
        // facing has reached the heading takes `Guy::move:55`'s settled
        // arm straight to the average, past the turn — so the crew keeps
        // the point it was last given and walks on toward it. That is the
        // whole of why a scout's dog is still walking four frames after
        // the man has stopped.
        //
        // Guy 0's position after either arm is the unit's own, so the two
        // are one expression here — written above, ahead of the figures
        // ([`Sim::guys_follow_on`] runs the tracked ones in their slots).
        // `Guy::process@005e0230:30–56`, right after `Guy::move`: a pivot
        // figure (`guy_flags & 0x100`, its type has `<RESTRICTION>` rows)
        // turns each turret toward where `set_all_pivots` last told it,
        // 15° a frame, snapping inside 15°, and sets the node's bit once
        // there (`docs/COMBAT.md` §55.3).
        if self.units[i]
            .ty
            .and_then(|t| self.art.pivots.get(&self.unit_types[t].type_index))
            .is_some_and(|n| !n.is_empty())
        {
            for g in &mut self.units[i].guys {
                g.turret.step();
            }
        }
        // `Guy::process`'s own tail: the sixty-fourth-frame repaint of the
        // collision block (`docs/COLLISION.md` §2.2). It reads the body as
        // `Guy::move` has just left it, which for guy 0 is the unit's own
        // position.
        self.coll_repaint(i);
        // And the same block's last line, `Guy::process@005e0230:120`:
        // a standing guy on its sixty-fourth frame forgets the in-danger
        // mark `Unit::set_in_danger` gave it (`docs/ORDERS.md` §22).
        // The flag is guy 0's (`set_in_danger` sets no other), so guy
        // 0's gate is the whole clear: a crew figure has none to drop.
        if collide::repaint_due(
            self.frame,
            self.units[i].index,
            self.units[i].movement.body.avg_speed,
        ) {
            self.units[i].guy_flag_0x20 = false;
        }
    }

    /// **The crew loop**, which is the same eight lines in three places
    /// and belongs to guy 0 in all of them: `Guy::set_angle@005d9010`
    /// (`005d90ad`–`005d9192`), `Guy::set_new_location@005d86f0`
    /// (`005d88da`–`005d89cc`) and `Unit::init`'s seating. Every guy past
    /// `squad_size` is told an angle and a point — `des_angle = angle`,
    /// `des = from + rotate(track, angle)`, each axis clamped into the
    /// world — and nothing else about it is touched.
    ///
    /// `from` is guy 0's own `x` / `y`, which is the unit's position on
    /// every frame the unit's step was taken.
    ///
    /// **`snap` is the callers' `param_3`**, and where it is set the crew
    /// is not told the point but *put* on it: the loop then runs
    /// `set_angle(crew, des_angle, 1)` and `set_new_location(crew, des,
    /// 1)`, which write the figure's facing and its body outright. Only
    /// the two teleporting callers carry it —
    /// `Unit::set_new_location(…, 1, …)`, which is `Unit::init`'s seating
    /// and `resolve_unit_collision`'s cell-centre snap
    /// (`docs/COLLISION.md` §6 step 6) — and `Unit::set_angle(…, 1)`,
    /// which `Unit::do_cast`'s turn to its target is (item 3920,
    /// [`Sim::unit_set_angle_as`]). Every other `Unit::set_angle` call this
    /// crate makes, and `Guy::move`'s, pass zero.
    pub(crate) fn crew_des(&mut self, u: usize, from: Pos, angle: movement::Angle, snap: bool) {
        let bound = Pos::new(
            self.world.width() * world::UNITS_PER_CELL,
            self.world.height() * world::UNITS_PER_CELL,
        );
        for g in 0..self.units[u].guys.len() {
            let Some(f) = &mut self.units[u].guys[g].follow else {
                continue;
            };
            let track = f.track;
            f.des_angle = angle;
            f.des = movement::follower_des(from, angle, track, bound);
            if snap {
                f.facing = angle;
                f.body.pos = f.des;
            }
        }
    }

    /// One crew guy's own frame — `Guy::process → Guy::move` for a guy
    /// with a track offset, which runs after guy 0's and reads the `des`
    /// guy 0 has just written.
    ///
    /// The two arms are `Guy::move`'s own. Standing on its destination it
    /// writes `last_speed = 0` and turns toward `des_angle` — its
    /// `guy_flags & 2` is never set by anything else, because
    /// `Guy::do_turn@005d97a0:37` recurses only into the crew that has
    /// *no* track, so the turn is never skipped. Walking, it is
    /// [`movement::follower_step`]. `docs/MOVEMENT.md`, "The follower's
    /// destination".
    pub(crate) fn process_follower(&mut self, i: usize, g: usize) {
        let Some(f) = self.units[i].guys[g].follow else {
            return;
        };
        // `Guy::move:203` — `GuyData::get_speed`, which is the unit's
        // `get_speed(x, y, 1)` asked at **this figure's** point: the
        // order scale and the slow ground, never the group cap
        // (`docs/MOVEMENT.md`, "The crew's speed"). The type's base
        // speed stepped a guarding computer's crew a quarter slow.
        // Its `+9` for a squad figure on a group order is
        // [`orders::guy_speed_steps`]'s; `g` is this figure's `guy_num`.
        let speed = orders::guy_speed_steps(
            &mut (),
            g as i32,
            |_| self.get_speed_at(i, f.body.pos, 1),
            |_| anim::SQUAD_SIZE as i32,
            |_| !self.units[i].orders.is_empty(),
            |_| self.current_order(i).is_some(),
            |_| {
                matches!(
                    self.order_type(i),
                    orders::index::GROUP_MOVE
                        | orders::index::GROUP_ATTACK_TO
                        | orders::index::GROUP_PATROL
                )
            },
        );
        let at_des = f.body.pos == f.des;
        self.guy_follow_anim(i, g, at_des, f.facing == f.des_angle);
        // A tracked crew guy's rate is [`movement::CREW_TURN_SPEED`] and
        // nothing else: `GuyData::turn_speed` returns it before the
        // instant-from-a-stop test, so neither the type's `TURN_SPEED`
        // nor a zero `last_speed` is read for this guy at all.
        let rate = movement::CREW_TURN_SPEED;
        let mut next = f;
        if at_des {
            next.body.last_speed = 0;
            if next.facing != next.des_angle {
                // `Guy::move:109` again, and for this guy in its own right
                // (`docs/ANIM.md` §4.8): the standing arm's
                // `turn_towards(des_angle, ..., 1)` hands `Guy::do_turn`
                // the same override guy 0's does. The gate `guy_flags & 2`
                // is always open here — the only writer is `do_turn`, and
                // `do_turn@005d97a0:37` recurses into the crew that has
                // **no** track, never this one — so a tracked crew figure
                // of a packing type asks for a turn animation it has not
                // got and pays the idle roll.
                let was = next.facing;
                next.facing = movement::turn_towards(next.facing, next.des_angle, rate).0;
                self.mark(anim::SITE_TURN_STAND);
                self.guy_do_turn_anim(i, g, was, next.facing, next.des_angle);
            }
            next.body.avg_speed = (next.body.avg_speed * 3 + next.body.last_speed) / 4;
        } else {
            let (facing, body) = movement::follower_step(f.body, f.facing, f.des, rate, speed);
            next.facing = facing;
            if let Some(mut body) = body {
                if !self.world.accepts(body.pos) {
                    body.pos = f.body.pos;
                }
                next.body = body;
                self.units[i].guys[g].stopped = false;
            }
        }
        self.units[i].guys[g].follow = Some(next);
    }
}
