//! `Unit::process`: one unit's step in [`Sim::tick`]'s object loop
//! (`docs/STRUCTURE.md` §3, `units/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// One unit's turn inside `Objects::process_all`'s first loop —
    /// `Unit::process@00610bc0`, or `Animal::process` for gaia's.
    pub(crate) fn process_unit(&mut self, i: usize, frame: i64, events: &mut Vec<Tick>) {
        // `who/o` — the unit's own [`Unit::index`], not its slot in this
        // vector. Every other `who/o` in the project is the game's `o`,
        // and a label that quietly meant the slot cost a probe run once.
        self.mark(&format!(
            "unit {}/{}",
            self.units[i].owner, self.units[i].index
        ));
        if !self.units[i].alive() {
            return;
        }
        // Gaia's animals: `Animal::process` is `Unit::process` without
        // a player behind it — no heal, no attrition, no reload; the
        // order step (`Animal::do_idle` when idle) and the body follow.
        //
        // `Animal::process@005d72c0` is [`anim::animal_process_steps`]: the
        // `+0x188` think (`Animal::work`), then `Guy::process` on the
        // squad and the crew, the walk handed to the body's frame.
        if self.units[i].is_gaia() {
            if self.units[i].on_map {
                crate::units::anim::animal_process_steps(
                    self,
                    |s| {
                        s.work(i, frame);
                        s.herd_sighting(i, frame);
                    },
                    |s| s.guy_walk_counts(i),
                    |s, walk| {
                        if s.units[i].alive() && s.units[i].on_map {
                            s.process_movement_on(i, walk);
                        }
                    },
                );
            }
            return;
        }
        // `Unit::process@00610bc0`'s head counts the reload down first
        // (`:59`–`:61`, `UnitData +0xae`), for every unit **inside or out**
        // (item 3993, `docs/GOLDEN.md` §186): a unit that garrisons inside
        // its reload comes out with the frames it spent inside spent.
        if self.units[i].combat.recharging > 0 {
            self.units[i].combat.recharging -= 1;
        }
        // `:63`–`65`: `full` and the wait, beside it.
        if self.units[i].full != 0 {
            self.units[i].full -= 1;
        }
        if self.units[i].waiting != 0 {
            self.units[i].waiting -= 1;
        }
        // `:69`–`71`: the heal's clock, beside them (item 3961).
        if self.units[i].healing != 0 {
            self.units[i].healing -= 1;
        }
        // `Unit::process@00610bc0`'s head, a caster's arm: a hero's
        // spells run out, then a point of craft back (`crate::spells::cast`),
        // before the heal and the work.
        self.process_spells(i, frame);
        self.recover_mana(i, frame);
        // A decoy whose age has run out was closed there, and `process`
        // returns before the rest of its frame (item 1351).
        if !self.units[i].alive() {
            return;
        }
        // `process_healing` runs for every unit, inside or out: the
        // garrison branch inside, and on the map the civilian heal
        // (`crate::units::heal`, `docs/COMBAT.md` §72).
        // `Unit::process@00610bc0:288`–`:420`, after the attrition and
        // before the heal: a unit's own queue (item 3981).
        self.carrier_queue_tick(i);
        self.garrison_heal(i, frame);
        self.ship_heal(i, frame);
        self.iroquois_heal(i, frame);
        self.senator_heal(i, frame);
        self.ceo_heal(i, frame);
        self.supply_heal(i, frame);
        self.civilian_heal(i, frame);
        // **The frozen frame's mark is cleared here** —
        // `Unit::process@00610bc0`'s `unit_masks2 & 0xffffffef`,
        // immediately after `process_healing` and under the same
        // `inside_up < 0` guard the `targeted` decay sits in, but
        // **outside** its sixteen-frame cadence. That is what makes
        // [`combat::umask2::NOT_FIRING`] a one-frame mark: the clear runs
        // before this unit's own order step, `Unit::fight` may set it
        // again inside that step, and phase 7's `Objects::inc_time` reads
        // it at the end of the frame. `docs/COMBAT.md` §43.2.
        if self.units[i].inside.is_none() && self.units[i].inside_unit.is_none() {
            self.units[i].unit_masks2 &= !combat::umask2::NOT_FIRING;
            // `Unit::process_cloak@005e1c20`, reached every sixteenth
            // frame by `o` and gated again to every 512th: a unit with
            // `unit_masks & 0x10000` drops that bit, one without drops
            // `0x1000` — so a fight's `0x11000` takes two passes to go
            // (item 2785, `docs/GOLDEN.md` §103). SEAM: the pass's other
            // writes (`unit_masks2 & 0x60`, the Spy cloak particles, the
            // `0x800` tribe and hero arms) have no reader here.
            if (frame + i64::from(self.units[i].index)).rem_euclid(512) == 0 {
                if self.units[i].fought {
                    self.units[i].fought = false;
                } else {
                    self.units[i].blown = false;
                }
            }
        } else {
            // `Unit::process@00610bc0:557`, the `inside_up >= 0` arm:
            // `unit_masks &= 0xfffeefff` — a garrisoned unit loses both
            // the blown and the fought bits on every frame it is inside
            // (item 2798, `docs/GOLDEN.md` §104; run678's ESF `1/0`
            // enters on 979 and reads `0x840008` on 980).
            self.units[i].blown = false;
            self.units[i].fought = false;
            // The same arm's `unit_masks2 &= 0xffff7f9f` and, for a unit
            // without the hero's `0x2000`, `unit_masks &= ~0x800`: inside,
            // no cloak stands but an ambushing hero's (item 3958,
            // `docs/AI.md` §402).
            self.units[i].unit_masks2 &= !(combat::umask2::LAND_CLOAK | 0x60);
            if self.units[i].ambushing.is_none() {
                self.units[i].cloak = false;
            }
        }
        if !self.units[i].on_map {
            // **The inside arm's figure write** — `Unit::process@00610bc0`'s
            // `else`, `guy+0x70 = guy+0x14` for every figure: `last_z` is
            // put onto `z`. This crate carries a figure's altitude for a
            // plane alone ([`air::Airframe`]), which is what the landed
            // Fighter's `last_z` on run223's 723 is (item 759).
            let af = &mut self.units[i].airframe;
            af.last_z = af.z;
            return;
        }
        // **`ObjectData::targeted` decays**, and its slot is the one the
        // attrition refresh is nested inside. `Unit::process@00610bc0`
        // reads, under `inside_up < 0`:
        //
        //     if ((frame + o) % 16 == 0) {
        //         targeted = targeted / 4;          // signed, toward zero
        //         process_cloak();
        //         if ((frame + o) % 32 == 0) { … process_attrition(); … }
        //     }
        //
        // Nothing else in the executable writes `+0x3d` but
        // `Object::init` (zero), `Object::find_nearby_target` (the bump on
        // its winner) and `Wall::process` (the same decay) — so a count
        // that is never decremented when an attacker drops its target is
        // *deliberate*: the field is a decaying crowding penalty, not a
        // reference count. `docs/COMBAT.md` §33.
        //
        // The counter is clamped non-negative here, so the original's
        // round-toward-zero idiom is a plain `/ 4`.
        if self.units[i].phase(frame) % TARGETED_DECAY_FRAMES == 0
            && self.units[i].inside.is_none()
            && self.units[i].inside_unit.is_none()
        {
            self.units[i].combat.targeted /= 4;
            self.process_cloak(i);
        }
        // Attrition first, movement second. That is the order inside
        // `Unit::process`, and it is observable: a unit that steps over a
        // border this frame is not standing there when this frame's
        // attrition looks, so it cannot bleed for the crossing until the
        // next one — and, because the period is only refreshed every 32
        // frames, usually not for a good while after that.
        if let Some(tick) = self.process_attrition(i, frame) {
            events.push(tick);
        }
        if !self.units[i].alive() {
            return;
        }
        // Then the order step — `Unit::work` → `do_job` on the front
        // order (`docs/ORDERS.md` §2.3): a move steps the unit, a build
        // runs the clock, an attack runs `fight`, an idle unit thinks.
        self.work(i, frame);
        if !self.units[i].alive() {
            // **A boat that stepped ashore in its own work still takes its
            // figures' `Guy::move`** (item 1191, `docs/TRANSPORT.md` §6.4).
            // `Unit::process@00610bc0` runs the `+0x188` think and then
            // `Guy::process` over the guy array with no test between them,
            // and the boat's death inside the think —
            // `Unit::set_new_location`'s sea arm, `eject_contents` then
            // `Object::die(0)` → `Unit::close` → `Object::close` — writes
            // neither the array nor its counts. So a boat standing on its
            // `des` on the walk with `stopped` set pays the arrival stand on
            // the frame it dies: East Indies' `1/56` on 8519, whose draw is
            // invisible in every dump because the boat is gone on 8520.
            //
            // **And the figure's step moves the disc after the close has
            // cleared it** (item 2088, `docs/AI.md` §208). `coll_follow`
            // skips a dead unit, and the reading here was that it could:
            // the figure on its `des`, `Guy::set_new_location` never
            // reached. run825's packet says otherwise. On East Indies'
            // tick 4204 the Freighter `1/191` disembarks; `Object::close`
            // → `remove_from_world@00647970` clears its radius-3 disc round
            // the figure at unit cell (384, 620), and then the boat's own
            // `Guy::move+0x4a5` → `Guy::set_new_location` →
            // `CollCheck::move_unit` steps the figure to (384, 621), whose
            // set pass paints the disc's new row (624) on no unit at all.
            // Those four bits walled `1/183` in thirty-four frames later.
            // `move_unit` asks nothing of the unit's life, so the disc moves
            // with the body here as it would on a live one.
            //
            // [`Sim::disembark`] is the only death this crate takes inside
            // a unit's own work; it leaves the boat dead and off the map.
            // SEAM: a unit killed in its own work with a death animation
            // (`Unit::close`'s `param_1 != 0`) has its figures handed to
            // `Objects::kill_guy` first, and this crate kills none that way
            // there. `coll_repaint` takes the boat too (item 1223): the
            // sixty-fourth-frame repaint re-marks the disc the close cleared.
            if self.units[i].kind.domain == crate::territory::attrition::Domain::Sea
                && !self.units[i].on_map
            {
                let from = self.units[i].movement.body.pos;
                self.process_movement(i);
                let to = self.units[i].movement.body.pos;
                self.coll_move(i, from, to);
            }
            return;
        }
        if !self.units[i].on_map {
            // `Unit::process` runs `Guy::process` for each figure after
            // `work` **whether or not the work put the unit inside**
            // (`00610bc0`, the `+0x188` call and the guy loops after it).
            // A plane that landed this frame is standing on its `des`, so
            // its figure takes `Guy::move`'s standing arm once — the
            // `last_speed 0`, `stopped 1` of run223's 722 — and never
            // again, the inside arm writing only `last_z` from then on.
            //
            // **And so is every unit that went inside in its own work**,
            // not an aircraft alone (item 850, `docs/ANIM.md` §15): a
            // walker that casts its barge at the shore is cargo by the
            // work's end, and its figures still take the standing arm —
            // East Indies' Merchant `1/77` on 19413, whose second figure
            // turns to its `des_angle` and pays the turn's draw.
            //
            // SEAM: a squad that garrisons in its own `work` takes the
            // same frame by the reading; no widening names its figures on
            // that frame yet.
            if self.units[i].inside.is_some() || self.units[i].inside_unit.is_some() {
                self.process_movement(i);
            }
            return;
        }
        self.process_movement(i);
    }
}
