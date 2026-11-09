//! The per-frame passes: buildings that shoot, and ammo in flight to its landing (`Ammo::inc_time`, `hit_target`) (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // Buildings
    // ------------------------------------------------------------------

    /// `Build::process` → `Build::do_attack` for a building that shoots (§8.6).
    pub(crate) fn process_building_combat(&mut self, b: usize, frame: i64) {
        let me = Obj::Building(b);
        if !self.active(me) || self.attack_of(me) == 0 {
            return;
        }
        // **A site does not shoot** (item 1323, `docs/COMBAT.md` §8.6):
        // `Build::process@0061edf0` returns straight after `Wall::process`
        // unless the building is finished — `61ee1b` reads `flags & 4`
        // inline for `Build::vftable@00b42174`, else vslot `+0x4c`, which
        // the PE holds as `0x472350`, `WallData::is_active` (`flags & 4`
        // again) — and `61ee31` jumps past `do_attack` to the epilogue.
        // Chapter forty-seven's Tower site `0/2008` fired a round on 712
        // here and never did in run514.
        if !self.buildings[b].active {
            return;
        }
        // **An anti-air building's `recharging` is its `Wall::inc_time`
        // cycle's, not this countdown** (item 1112, `docs/COMBAT.md`
        // §84): `Build::do_attack@006228f0`'s head (`6228fe`..`622946`) skips the countdown for
        // an `ANTI_AIR` building that is neither a Lookout nor an
        // Observation Post, and it never reaches `Object::fire_ammo`.
        let cycles = self.wall_cycle_of(b).is_some();
        let bd = &self.buildings[b];
        let phase = bd.phase(frame) & 0x1f;
        // Without a target, `do_attack` runs every 32nd frame.
        // **Or on any frame its nearest sighting is in range** — the head of
        // `Build::process@0061edf0` (`61ee9e`..`61eec4`, `docs/GOLDEN.md`
        // §59): with no target, `do_attack` runs when `(o + frame) & 0x1f`
        // is 0, else when `near_o`/`near_who` (+0x34/+0x36), the last
        // search's nearest sighting inside `0xf00`, is in range.
        if bd.target.is_none() && phase != 0 && !bd.near.is_some_and(|n| self.is_in_range(me, n)) {
            return;
        }
        if !cycles && bd.recharging > 0 {
            self.buildings[b].recharging -= 1;
            return;
        }
        // **The head of `do_attack` past the countdown** (`62295a`..
        // `622978`): on the building's 32-frame phase, with the latch
        // down, `visible` is cleared; the latch drops every call.
        if phase == 0 && !self.buildings[b].attacking {
            self.buildings[b].visible = 0;
        }
        self.buildings[b].attacking = false;
        let bd = &self.buildings[b];
        let p = self.profile(me);
        // The garrison's arrows: the input the earlier mechanic took, plus
        // what the squads actually inside contribute (`docs/CITIES.md` §6).
        let garrison_attack = bd.garrison_attack + self.garrison_attack_sum(b);
        // `BuildData::get_base_garrison_arrows@0062e340`: the Maya's
        // `MAYA_GARRISON_ARROWS` (item 3970).
        let arrows = combat::garrison_arrows(
            self.attack_of(me),
            self.base_garrison_arrows(b),
            p.most_shots,
            garrison_attack,
        );
        if arrows == 0 {
            return;
        }
        // **The target, as `Build::do_attack@006228f0`'s listing takes it**
        // (`622a37`..`622b4c`, item 1131, `docs/COMBAT.md` §8.6):
        // - a building without an explicit order (`build_masks & 4`) calls
        //   `Build::find_target@00622c80` on **every** call (`622a3f`), and
        //   `compare_target`'s current-target ×2 (or /2) is what holds it;
        // - no target: cleared, and return (`622a5a` → `622c1c`);
        // - one `valid_target` refuses: `find_target`, and return without
        //   a shot (`622a75` → `622b25`);
        // - unordered, on `(o + frame + 14) & 0x1f == 0`: `find_target` and
        //   return, unless the target is a unit that is not moving and is
        //   combat-role (`type +0x2c8 & 0x10000`) or casting
        //   (`action_type == CAST_SPELL`, `622b20`);
        // - out of range (`622b45`): the target is cleared (`622c1c`).
        // `find_target` itself clears the order bit (`& 0xfffb`).
        let radius = (p.x_size.max(p.y_size) + 2 * self.max_range_of(me)) * 0x60;
        if !bd.ordered {
            self.buildings[b].target = self.find_nearby_target(me, radius);
        }
        let Some(target) = self.buildings[b].target else {
            self.buildings[b].ordered = false;
            return;
        };
        if !self.valid_target(me, target) {
            self.buildings[b].target = self.find_nearby_target(me, radius);
            self.buildings[b].ordered = false;
            return;
        }
        if !self.buildings[b].ordered && ((self.buildings[b].phase(frame) + 14) & 0x1f) == 0 {
            let keep = match target {
                Obj::Unit(u) => {
                    self.units[u].movement.dest.is_none()
                        && (self.profile(target).combat_role
                            || matches!(
                                self.current_order(u).map(|o| o.body),
                                Some(crate::orders::Body::Cast(_))
                            ))
                }
                Obj::Building(_) => false,
            };
            if !keep {
                self.buildings[b].target = self.find_nearby_target(me, radius);
                return;
            }
        }
        if !self.is_in_range(me, target) {
            self.buildings[b].target = None;
            self.buildings[b].ordered = false;
            return;
        }
        // `do_attack`'s in-range arm returns before `fire_ammo` for the
        // same buildings (`622b52`..`622ba3`, to the return at `622c2e`): their round is the cycle's
        // release event ([`Sim::walls_inc_time`]).
        if cycles {
            return;
        }
        // Fire: `ammo_per_att` ammo at random points in the footprint.
        let centre = self.pos_of(me);
        let tp = self.profile(target);
        let target_pos = self.pos_of(target);
        let acc = combat::accuracy(p.to_hit, p.attenuate, self.attack_dist(me, target));
        let land_unit = matches!(target, Obj::Unit(_)) && matches!(tp.domain, Domain::Land);
        let s = combat::scatter(&self.tables.tuning, acc, land_unit, false, false);
        let xs = p.x_size * 0x60;
        let ys = p.y_size * 0x60;
        for _ in 0..p.ammo_per_att.max(1) {
            let ox = if xs - 1 < 1 {
                0
            } else {
                self.mark(SITE_FIRE_FOOTPRINT_X);
                self.rng.roll() % xs
            };
            let oy = if ys - 1 < 1 {
                0
            } else {
                self.mark(SITE_FIRE_FOOTPRINT_Y);
                self.rng.roll() % ys
            };
            let launch = Pos::new(centre.x - xs / 2 + ox, centre.y - ys / 2 + oy);
            let angle = find_angle(target_pos.x - launch.x, target_pos.y - launch.y);
            let landing =
                self.scatter_landing(target_pos, s, (SITE_AMMO_SCATTER_X, SITE_AMMO_SCATTER_Y));
            // **A building's arrow flies at 90, not at its type's
            // `PROJ_SPEED`** (`Ammo::init@0067bbf0`'s building arm,
            // `0067d1e0`): `get_shot() == 0` divides by
            // `unit_move_speed × 0x5a`. A Tower in the first three ages
            // reads 0 (item 1380, `docs/GOLDEN.md` §57).
            let d = if self.building_shot(b) == 0 {
                i64::from(combat::UNIT_MOVE_SPEED) * 0x5a
            } else {
                i64::from(p.proj_speed) * i64::from(combat::UNIT_MOVE_SPEED)
            };
            let dx = i64::from(landing.x - launch.x);
            let dy = i64::from(landing.y - launch.y);
            let total_time = combat::flight_time(dx * dx + dy * dy, d).max(1);
            let landing = self.lead_landing(Some(target), landing, total_time);
            // `Object::fire_ammo`'s building arm: 250 above the building's
            // own `z` (§46.1).
            let sz = self.world.tile_z(centre.tile()) + 0xfa;
            let ez = self.aim_z(target, land_unit);
            self.add_ammo(combat::Projectile {
                shooter: me,
                owner: self.buildings[b].owner,
                target: Some(target),
                launch,
                landing,
                cur_time: 0,
                total_time,
                accuracy: acc,
                angle,
                splash_area: p.splash_area,
                num_guys: 0,
                // A building has no order, so `Ammo::init`'s `local_38`
                // is zero and the flag turns on the target alone.
                rolling: land_unit,
                missed: false,
                harmless: false,
                air: matches!(tp.domain, Domain::Air),
                sz,
                ez,
                v1z: combat::arc_v1z(sz, ez, total_time),
                slot: 0,
                flame: false,
                // SEAM: the building's ammo piece is not read, so its
                // `BuildingArrow` (`puncture="1"`) never sticks here
                // (item 3679, `docs/COMBAT.md` §46.7).
                punctures: false,
                spent: false,
                crash: None,
                spline: None,
            });
        }
        // `do_attack`'s tail after `fire_ammo` (`622b5b`..`622bd0`): the
        // latch is raised, and the target's player gets the building's
        // `visible` bit; a player that has never seen it has the building
        // light itself into its fog (vslot `+0x164`).
        self.buildings[b].attacking = true;
        let bit = Self::who_bit(self.owner_of(target));
        if self.buildings[b].visible & bit == 0 {
            self.buildings[b].visible |= bit;
            if self.buildings[b].ever_seen & bit == 0 {
                self.update_local_seen_build(b);
            }
        }
        self.buildings[b].recharging = p.recharge / arrows;
    }

    // ------------------------------------------------------------------
    // Ammo
    // ------------------------------------------------------------------

    /// `Ammo::inc_time` for every live projectile (§9.2, §9.3), and the
    /// **rolling** arm at its foot (§42.2).
    ///
    /// `inc_time@0067d380` breaks its own loop at `cur_time >=
    /// total_time` and then, for a shot with flag `4`, tries
    /// `hit_target` and `check_hit(GROUND)` **once**: if neither names an
    /// object it sets flag `8` and the shot travels on along its line —
    /// up to `3 × total_time` — until the terrain rises to meet its
    /// parabola, and only then does `do_damage` run. So a rolling shot
    /// whose target died in flight spends **no draws on the frame it was
    /// due**, where a non-rolling one punctures the ground with two.
    ///
    /// **The terrain test** is §46's: the arc `(v1z × t + sz) + GRAV_Z ×
    /// 0.5 × t × t` ([`combat::arc_z`]) against the ground under the point
    /// ([`Sim::ground_z`], `TerrainOut::find_data_z@00866560`), both in
    /// [`crate::single::Single`] arithmetic, operation for operation as the
    /// listing does them. The shot comes down on the first step its height
    /// is not above the ground, and `do_damage` runs at the new point — on
    /// whatever `check_hit` finds there, or into the ground if nothing.
    pub(crate) fn process_projectiles(&mut self, frame: i64) {
        let mut i = 0;
        while i < self.combat.projectiles.len() {
            self.combat.projectiles[i].cur_time += 1;
            let p = self.combat.projectiles[i];
            // **A spent arrow** (item 3679, `docs/COMBAT.md` §46.7):
            // `inc_time`'s `flags & 2` clear arm returns until `cur_time`
            // reaches 200, and then closes it. Its dead shooter's bump is
            // [`Sim::hold_frames_tick`]'s, as for any round.
            if p.spent {
                if p.cur_time >= SPENT_LIFE {
                    self.combat.projectiles.remove(i);
                } else {
                    i += 1;
                }
                continue;
            }
            if p.crash.is_some() {
                // `inc_time`'s first statement, here rather than in
                // [`Sim::hold_frames_tick`]: a crash is laid inside this walk,
                // and the frame it is laid on bumps its plane's slot too.
                if let Obj::Unit(s) = p.shooter
                    && !self.active(p.shooter)
                {
                    self.units[s].hold_frames += 1;
                }
                if p.in_flight() {
                    if !self.crash_comes_down(i) {
                        i += 1;
                        continue;
                    }
                    // The landing arm jumps back to the loop's head
                    // (`67d726` → `67d39a`): `cur_time` and a dead
                    // shooter's `hold_frames` are bumped a second time,
                    // and `cur_time >= total_time` ends it.
                    self.combat.projectiles[i].cur_time += 1;
                    if let Obj::Unit(s) = p.shooter
                        && !self.active(p.shooter)
                    {
                        self.units[s].hold_frames += 1;
                    }
                }
                // **And it strikes there** (item 2677, `docs/AI.md` §230):
                // `inc_time`'s tail restricts the point and runs
                // `Ammo::do_damage` — `init_crash` cleared flag `0x10` —
                // and only then closes, as an ordinary round does below.
                let p = self.combat.projectiles[i];
                self.land(p, frame);
                if let Some(k) = self
                    .combat
                    .projectiles
                    .iter()
                    .position(|q| q.slot == p.slot)
                {
                    self.combat.projectiles.remove(k);
                }
                i = self
                    .combat
                    .projectiles
                    .partition_point(|q| q.slot <= p.slot);
                continue;
            }
            // **A guided round** (item 3802): `inc_time`'s spline arm, which
            // either closes it on this frame or holds it, re-planned or not.
            if p.in_flight() && !(p.spline.is_some() && self.guided_closes(i)) {
                i += 1;
                continue;
            }
            if p.rolling {
                if !p.missed {
                    // `hit_target` forgets the target on a miss, in
                    // `inc_time` exactly as in `do_damage`: run112's
                    // `0/8` arrow prints `whom -1 ox -1` on the block
                    // after it rolled, with `flags 14` beside it.
                    let hit = self.hit_target(&p);
                    if !hit {
                        self.combat.projectiles[i].target = None;
                        if self.check_hit(&p).is_none() {
                            self.combat.projectiles[i].missed = true;
                        }
                    }
                }
                if self.combat.projectiles[i].missed {
                    // `if ((uint)(total_time * 3) < cur_time) close()` —
                    // and `close` is not `do_damage`, so a shot that
                    // never meets the ground damages nothing.
                    if (p.total_time.wrapping_mul(3) as u32) < p.cur_time as u32 {
                        self.combat.projectiles.remove(i);
                        continue;
                    }
                    // On along the line, and down the arc (§46.1): the
                    // point at `t / T` past the launch, off the world is
                    // a `close`, and the shot comes down on the first step
                    // its height is at or under the ground's.
                    let at = combat::arc_point(p.launch, p.landing, p.cur_time, p.total_time);
                    if at.x < 0
                        || at.y < 0
                        || at.x >= self.world.width() * UNITS_PER_CELL
                        || at.y >= self.world.height() * UNITS_PER_CELL
                    {
                        self.combat.projectiles.remove(i);
                        continue;
                    }
                    let z = combat::arc_z(p.v1z, p.sz, p.cur_time);
                    let ground = crate::single::Single::from_i32(self.ground_z(at));
                    if z.gt(ground) {
                        i += 1;
                        continue;
                    }
                    // Landed: `pos = (x, y, (int)z)` and `total_time =
                    // cur_time`, then `do_damage` at the new point.
                    let q = &mut self.combat.projectiles[i];
                    q.landing = at;
                    q.ez = z.to_i32();
                    q.total_time = q.cur_time;
                }
            }
            // **The round keeps its pool slot while it lands** (item 2609,
            // `docs/AI.md` §228): `Ammo::inc_time` runs `do_damage` and only
            // then closes, so a plane it brings down lays its crash round in
            // the next free slot — run405's `0/6` crash takes index 1 under
            // the Battery's round in 0. The walk then goes on from the next
            // slot past this one, as the pool's own walk does.
            let p = self.combat.projectiles[i];
            // `inc_time:260`: a round with flag `0x10` is closed, and
            // `Ammo::do_damage` is never called for it (§39.5).
            if !p.harmless {
                self.land(p, frame);
            }
            if let Some(k) = self
                .combat
                .projectiles
                .iter()
                .position(|q| q.slot == p.slot)
                && !self.combat.projectiles[k].spent
            {
                self.combat.projectiles.remove(k);
            }
            i = self
                .combat
                .projectiles
                .partition_point(|q| q.slot <= p.slot);
        }
    }

    /// **`Ammo::inc_time@0067d380`'s spline arm** (`67d3e6`–`67d57a`,
    /// `67d72b`–`67d8ca`; item 3802, `docs/GOLDEN.md` §152.2), on a frame
    /// short of `total_time`, for a round with a spline. It answers whether
    /// the round closes now.
    ///
    /// - With no target, or one no longer active, or no point past this
    ///   one (`len ≤ cur + 1`), it holds: the round flies on to
    ///   `total_time` and lands on its end point.
    /// - Point `cur`, truncated, is measured against the target's point:
    ///   `vector_dist(|dx|, |dy|)` under the target type's `target_size`
    ///   (`UnitTypeData +0x300`; `0xc0` for a building), and `|dz| < 0xc0`
    ///   against the target figure's height. Inside both, the end point
    ///   becomes that point and `total_time` `cur − 1`, and the arm jumps
    ///   back to the loop's head (`67d57a` → `67d392`): `cur_time` and a
    ///   dead shooter's `hold_frames` are bumped a second time, and the
    ///   round lands there.
    /// - Otherwise, on every fourth frame of a spline (`cur ≠ 0`, `cur & 3
    ///   == 0`) and a unit target, it **re-plans**: a new spline from the
    ///   point (its integers), through `P + 2·(point[cur+1] − P)`, to the
    ///   target's point now, at `200 × unit_move_speed`; the end point is
    ///   the target's, `total_time` the new point count and `cur_time` 0.
    ///
    /// run1469: the Russian Jet Fighter `2/14`'s round on `1/14` re-plans
    /// on 620 (nine points) and 624 (six), and closes on 628 at `cur` 4,
    /// (12848, 14983) against the target's (12792, 14712), a frame before
    /// the arc ours flew would have landed.
    ///
    /// SEAM: a ground unit's figure height (`guys[0] +0x14`) is read as its
    /// object's, which this crate carries; an aircraft's is its airframe's.
    pub(super) fn guided_closes(&mut self, i: usize) -> bool {
        let p = self.combat.projectiles[i];
        let Some(path) = p.spline else {
            return false;
        };
        let Some(t) = p.target.filter(|&t| self.active(t)) else {
            return false;
        };
        let cur = usize::try_from(p.cur_time).unwrap_or(0);
        if path.len() <= cur + 1 {
            return false;
        }
        let pt = path.point(cur).map(Single::to_i32);
        let at = self.pos_of(t);
        let tz = match t {
            Obj::Unit(u) if self.units[u].kind.domain == Domain::Air => self.units[u].airframe.z,
            _ => self.world.object_z(at.tile()),
        };
        let radius = match t {
            Obj::Unit(_) => self.profile(t).target_size,
            Obj::Building(_) => 0xc0,
        };
        if vector_dist((pt[0] - at.x).abs(), (pt[1] - at.y).abs()) < radius
            && (pt[2] - tz).abs() < 0xc0
        {
            let q = &mut self.combat.projectiles[i];
            q.landing = Pos::new(pt[0], pt[1]);
            q.ez = pt[2];
            q.total_time = q.cur_time - 1;
            q.cur_time += 1;
            if let Obj::Unit(s) = p.shooter
                && !self.active(p.shooter)
            {
                self.units[s].hold_frames += 1;
            }
            return true;
        }
        if cur != 0 && cur % 4 == 0 && matches!(t, Obj::Unit(_)) {
            let from = pt.map(Single::from_i32);
            let ahead = path.point(cur + 1);
            let two = Single::from_bits(0x4000_0000);
            let dir = [0, 1, 2].map(|c| ahead[c].subss(from[c]).mulss(two).addss(from[c]));
            let end = [at.x, at.y, tz].map(Single::from_i32);
            let speed = Single::from_i32(200 * combat::UNIT_MOVE_SPEED);
            let next = combat::Spline::from_dir(from, dir, end, speed);
            let q = &mut self.combat.projectiles[i];
            q.landing = at;
            q.ez = tz;
            q.spline = Some(next);
            q.total_time = i32::try_from(next.len()).expect("at most SPLINE_MAX");
            q.cur_time = 0;
        }
        false
    }

    /// **`Ammo::inc_time@0067d380`'s crash arm** (`+0x50 == 1`,
    /// `67d590`–`67d726`; `docs/AI.md` §228), on a frame short of
    /// `total_time`: the landing drifts by the bank pair and is
    /// `WorldData::restrict`ed, and then the round is tested against the
    /// ground at its point on the line, `(float)find_data_z > arc` —
    /// [`combat::arc_point`] and [`combat::arc_z`], the rolling arm's own
    /// arithmetic. Under the ground it comes down there: the point is its
    /// landing, `ez` the ground, `total_time` one past `cur_time`.
    /// Returns whether it came down.
    ///
    /// SEAM: the arm returns before the test when
    /// `GraphicPieces::get_gpiece_type(+0x8)` is non-zero; this crate loads
    /// no graphic piece and reads it as zero. run405's two crashes end
    /// short of their `total_time` (13 of 18, 12 of 17), which only the
    /// test does.
    pub(super) fn crash_comes_down(&mut self, i: usize) -> bool {
        let p = self.combat.projectiles[i];
        let Some(c) = p.crash else { return false };
        let mut landing = p.landing;
        if c.bank != (0, 0) {
            landing = self.restrict_pos(Pos::new(landing.x + c.bank.0, landing.y + c.bank.1));
            self.combat.projectiles[i].landing = landing;
        }
        let at = combat::arc_point(p.launch, landing, p.cur_time, p.total_time);
        let ground = self.ground_z(at);
        let z = combat::arc_z(p.v1z, p.sz, p.cur_time);
        if !crate::single::Single::from_i32(ground).gt(z) {
            return false;
        }
        let q = &mut self.combat.projectiles[i];
        q.landing = at;
        q.total_time = q.cur_time + 1;
        q.ez = ground;
        true
    }

    /// `Ammo::hit_target@00678f90` (§9.4): the shot's own target, still
    /// active and still inside its `target_size` (or its footprint, for a
    /// building), at the landing point.
    pub(super) fn hit_target(&self, p: &combat::Projectile) -> bool {
        p.target
            .filter(|&t| self.struck(t))
            .is_some_and(|t| match t {
                Obj::Unit(u) => combat::hits_unit(
                    p.landing,
                    self.units[u].pos,
                    self.profile(t).target_size,
                    p.accuracy,
                ),
                Obj::Building(b) => {
                    let tp = self.profile(t);
                    combat::hits_building(p.landing, self.buildings[b].pos, tp.x_size, tp.y_size)
                }
            })
    }

    /// **`do_damage`'s puncture arm, after its two draws** (`678c59`..,
    /// item 3679, `docs/COMBAT.md` §46.7): the point is
    /// `WorldData::restrict`ed and, when the round's piece punctures
    /// (`verify_ammo_flags(…, 1)`) and the tile under the point is not
    /// ocean (`mask & 0x30 != 0x20`; `is_valid` cannot refuse a restricted
    /// point), the round is left in its slot as `flags 1` with `cur_time`
    /// 0 — [`combat::Projectile::spent`] — rather than closed. The point
    /// is the decal's; the record keeps its own `ex`/`ey` and its target.
    pub(super) fn stick(&mut self, p: combat::Projectile, at: Pos) {
        if !p.punctures || p.crash.is_some() {
            return;
        }
        let at = self.restrict_pos(at);
        if self.world.is_tocean(at.tile()) {
            return;
        }
        // `flags = 1` whole, so the rolled and missed bits go with the live
        // one (run1331b: `1/52`'s spent arrow reads `flags 1` from 3333,
        // where ours kept `4` and `8`); the target pair is not written.
        if let Some(q) = self
            .combat
            .projectiles
            .iter_mut()
            .find(|q| q.slot == p.slot)
        {
            q.spent = true;
            q.cur_time = 0;
            q.rolling = false;
            q.missed = false;
        }
    }

    /// `Ammo::do_damage` (§9.3): the hit test at the landing point, then
    /// `Object::do_damage` on what was hit — the target, or whatever stood
    /// there instead — and the splash around it.
    pub(super) fn land(&mut self, p: combat::Projectile, frame: i64) {
        let mut target = p.target.filter(|&t| self.struck(t));
        // **A round whose target is gone lands as a ground round**
        // (`Ammo::do_damage@00678060`'s first test, item 3829, `docs/
        // GOLDEN.md` §152.12): an inactive target object (`+8 & 1` clear)
        // has `whom`/`ox` set to −1 before anything else, and the domain
        // `check_hit` and the splash's unit arm are handed is the target
        // type's `+0x218` only while those are set — `GROUND` otherwise.
        // So an air shooter's round on a plane that died in flight finds
        // and splashes only what is not air: run1476's `0/15` round of 792
        // on the dead Stealth Bomber `1/17` comes down on 797 455 from the
        // Jet Fighter Bomber `1/18` and leaves it whole (205 on blocks
        // 796–799), where ours splashed it for 3½.
        let p = combat::Projectile {
            air: p.air && target.is_some(),
            ..p
        };
        if !self.hit_target(&p) {
            target = self.check_hit(&p);
        }
        if p.splash_area == 0 {
            let Some(t) = target else {
                // A shot into the ground: two draws for where it punctures
                // (§39). Marked per draw, because the original spends them
                // at two addresses.
                self.mark(SITE_PUNCTURE_X);
                let dx = self.rng.roll() % 41 - 20;
                self.mark(SITE_PUNCTURE_Y);
                let dy = self.rng.roll() % 41 - 20;
                self.stick(p, Pos::new(p.landing.x + dx, p.landing.y + dy));
                return;
            };
            if t == p.shooter {
                return;
            }
            let angle = p.bearing();
            self.do_damage_round(
                p.shooter, t, angle, true, p.flame, 0x100, false, false, frame,
            );
            return;
        }
        // **The missile arm's shield** (item 1078, `678337`..`67845d`;
        // `docs/PRODUCTION.md` "The missile's other arms"): a missile's
        // round is closed with no damage and no walk when the landing
        // cell's territory owner holds `MISSILE_DEFENSE_BONUS` and is not
        // the shooter's player. run390's V2b on 3169, in who=1's land ten
        // frames' flight after `tech who=1 missile_shield on`. SEAM: a
        // landing on ocean, whose owner is the first owned cell of a ring.
        if self.profile(p.shooter).has(mask::MISSILE)
            && let Some(w) = self.world.owner_at(p.landing).player()
            && w != p.owner
            && self.missile_defense_held(w)
        {
            return;
        }
        // **The nuke arm** (item 1091, `67846d`..`6785db`): a nuke's round
        // strikes nothing where it lands; it lays the blast (`crate::nuke`).
        if self.is_nuke(p.shooter) {
            self.nuke_land(p, frame);
            return;
        }
        // Splash: everything on the square of cells the spiral table walks,
        // and the one cell past it the inclusive walk reads
        // ([`combat::splash_extra_cell`]), the intended target at full count
        // and everything else as a fringe.
        let k = combat::splash_cells(p.splash_area);
        let extra = combat::splash_extra_cell(k);
        let landing_cell = p.landing.cell();
        let candidates: Vec<Obj> = (0..self.units.len())
            .map(Obj::Unit)
            .chain((0..self.buildings.len()).map(Obj::Building))
            // The round's own target is struck while it stands, hit points
            // or none (`struck`): a second siege shot lands on a city the
            // first has taken to its ceiling (item 1528).
            .filter(|&o| self.struck(o))
            .filter(|&o| {
                let c = self.pos_of(o).cell();
                let (dx, dy) = (c.x - landing_cell.x, c.y - landing_cell.y);
                // The square, and the spiral's next entry the inclusive
                // walk reads (item 3636): run1296b's Bombard round on 5047
                // lands in cell (8, 9) and wounds the Library `0/2005`,
                // chained in (7, 7).
                (dx.abs() <= k && dy.abs() <= k) || extra == Some((dx, dy))
            })
            .filter(|&o| {
                // **The round's own target skips the team test**
                // (`6787d9`..`6787e1`, item 1200): the shooter itself is
                // left out and a player past seven, and then only an
                // object that is not the round's target is asked whether
                // it is the shooter's own or its mutual ally. So a round
                // that `check_hit` put on one of its own side's buildings
                // strikes it: chapter forty-one's Biplane on who=1's
                // `1/2003`, 858 (`docs/GOLDEN.md` §50). SEAM: a neutral
                // object, neither allied nor at war, is struck there and
                // left out here.
                let owner = self.owner_of(o);
                if o == p.shooter || owner >= crate::world::PLAYER_SLOTS {
                    return false;
                }
                target == Some(o)
                    || (owner != p.owner
                        && (self.at_war_with(p.owner, owner) || self.at_war_with(owner, p.owner)))
            })
            .collect();
        for o in candidates {
            let op = self.profile(o);
            let pos = self.pos_of(o);
            let count = match o {
                Obj::Unit(_) => {
                    if matches!(op.domain, Domain::Air) != p.air || op.has(mask::MISSILE) {
                        continue;
                    }
                    let d = vector_dist(p.landing.x - pos.x, p.landing.y - pos.y)
                        - 0xc0
                        - op.guy_radius;
                    let c = combat::splash_count(d, p.splash_area);
                    if c <= 0 {
                        continue;
                    }
                    c
                }
                Obj::Building(_) => {
                    if p.air {
                        continue;
                    }
                    // The x term is explicit in the decompile and the y term
                    // is a lost register; by symmetry both axes less the
                    // footprint, then the hypotenuse.
                    let dx = ((p.landing.x - pos.x).abs() - op.x_size * 0xc0).max(0);
                    let dy = ((p.landing.y - pos.y).abs() - op.y_size * 0xc0).max(0);
                    let d = vector_dist(dx, dy);
                    let c = combat::splash_count(d, p.splash_area);
                    if c < 0 {
                        continue;
                    }
                    c
                }
            };
            let splash = target != Some(o);
            let angle = p.bearing();
            self.do_damage_round(
                p.shooter, o, angle, true, p.flame, count, splash, false, frame,
            );
        }
    }

    /// `Ammo::check_hit` (§9.4): a unit within two tiles of the landing point
    /// in the ammo's domain class — any player's — else a building on the
    /// tile.
    pub(super) fn check_hit(&self, p: &combat::Projectile) -> Option<Obj> {
        // **An aircraft's round passes over its own side** (`678db9`..
        // `678dcc`, item 1200): a shooter of the air domain that is not a
        // missile searches `SEARCH_NON_FRIENDLY` (6), which
        // `Search::valid_search@0067daa0`'s case 6 reads as "not the
        // searcher's own player" — allies are found. Every other shooter
        // searches `SEARCH_ALL`. Unasked while [`Sim::land`] left the
        // shooter's side out of the splash; it is asked since the round's
        // own target skips that test.
        let sp = self.profile(p.shooter);
        let own_passed = matches!(sp.domain, Domain::Air) && !sp.has(mask::MISSILE);
        let mut best: Option<(i32, usize)> = None;
        for (i, u) in self.units.iter().enumerate() {
            if !(u.alive() && u.on_map) || Obj::Unit(i) == p.shooter {
                continue;
            }
            if own_passed && u.owner == p.owner {
                continue;
            }
            // `check_hit` is a `find_unit`, whose leader loop stops at eight:
            // an arrow never lands on gaia (`world::PLAYER_SLOTS`).
            if u.is_gaia() {
                continue;
            }
            let up = self.profile(Obj::Unit(i));
            if matches!(up.domain, Domain::Air) != p.air {
                continue;
            }
            let d = vector_dist(u.pos.x - p.landing.x, u.pos.y - p.landing.y);
            if d > 0x180 {
                continue;
            }
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, i));
            }
        }
        if let Some((d, i)) = best {
            if self.profile(Obj::Unit(i)).target_size < d {
                // Found, but too small to be where it landed.
            } else {
                return Some(Obj::Unit(i));
            }
        }
        self.building_under(p.landing).map(Obj::Building)
    }

    /// **`check_hit`'s second half** (item 1077, `docs/COMBAT.md` §73):
    /// `ObjectsData::find_building_at@0065ab40(landing tile, SEARCH_ALL,
    /// −1, FILTER_ALL)` at `678ece`, reached only for a landing inside the
    /// world (`678e7a`..`678eac`, `world+0x18 × 0xc0` a side) when
    /// `find_unit` found nothing. The tile is `div_3_table[c >> 6]`, the
    /// landing's own tile; the building is the first, on the 3×3 cells
    /// round it, of a player below eight, active (vslots `0xc` and `0x4c`,
    /// `SubObjectData::is_active` and `WallData::is_active`) whose
    /// footprint, `WallData::tile_corner` to `+ x_size`/`+ y_size`, holds
    /// the tile. This crate compared the tile against the building's point
    /// in position units, and no building was ever found: a V2 with no
    /// target that came down on run371's Barracks `1/2006` struck it as a
    /// splash fringe, 400 of its 1,200, where the original struck it as
    /// the round's own target and destroyed it on 2821.
    ///
    /// **Active is the two flag reads and nothing about hit points** (item
    /// 3539, `docs/AI.md` §303): `0065ac7f` calls vslot `0xc` and `0065ac8e`
    /// vslot `0x4c`, flags `& 1` and `& 4`. [`Sim::active`] also asks for
    /// health left, the target search's question, and refused a city at
    /// its ceiling: run1171's Napata `0/2000`, reduced on 4341, whose
    /// footprint holds the tile `1/28`'s missed round comes down on at
    /// 4443. Theirs strikes it (`build_masks` takes `0x4000` on block
    /// 4444) and makes no puncture draws; ours punctured the ground. A
    /// building with no combat profile is still refused, as
    /// [`Sim::struck`] refuses it.
    ///
    /// **The walk's order picks when footprints overlap** (item 3970,
    /// `docs/GOLDEN.md` §176): the nine cells are `move_x`/`move_y`'s
    /// first nine round the tile's cell (`tile >> 2`), the centre first,
    /// and each cell's chain holds the buildings whose point is in it. A
    /// console `add` overlaps: chapter 126's Temple `1/2006` stands on the
    /// Library `1/2005`'s footprint, and the City's arrow missing `0/7` on
    /// 804 comes down on the tile both hold — theirs strikes the Temple,
    /// whose point is in the landing's own cell, where the first building by
    /// index took the Library. Inside one cell the order is the building
    /// index, as the target search's is; the chain's own order there is not
    /// established.
    pub(super) fn building_under(&self, landing: Pos) -> Option<usize> {
        if landing.x < 0
            || landing.y < 0
            || landing.x >= self.world.width() * UNITS_PER_CELL
            || landing.y >= self.world.height() * UNITS_PER_CELL
        {
            return None;
        }
        let tile = landing.tile();
        let at = landing.cell();
        let (mx, my) = (crate::ai_place::MOVE_X, crate::ai_place::MOVE_Y);
        mx.iter().zip(my).find_map(|(&dx, dy)| {
            let cell = crate::Cell::new(at.x + dx, at.y + dy);
            (0..self.buildings.len()).find(|&b| {
                self.buildings[b].pos.cell() == cell
                    && self.buildings[b].owner < crate::world::PLAYER_SLOTS
                    && self.struck(Obj::Building(b))
                    && self.buildings[b].active
                    && self.covers_tile(b, tile)
            })
        })
    }
}
