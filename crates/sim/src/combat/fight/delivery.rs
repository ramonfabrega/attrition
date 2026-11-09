//! Damage and death: `Object::do_damage` and `take_damage`, plunder and refunds, `close`, `die` and the death pool (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // Delivery
    // ------------------------------------------------------------------

    /// **The tail of `Object::do_damage@0064c1d8`: a city at its ceiling**
    /// (`docs/AI.md` §142). After `Build::check_capture` has answered no, a
    /// hit by another player's object on an active city building whose
    /// `damage` has reached `hits(0)` (1) has a computer leader's **siege**
    /// attacker, itself carrying `unit_masks & 0x40000` and sitting in an
    /// army, drag that army onto the city (`Army::charge`, an `ATTACK_TO`
    /// at the front of the queue), and (2) stamps the city's
    /// `reduce_stamp` — always on the hit that brought it to the ceiling
    /// (`fresh`, `local_70`), and once in 300 frames after. The
    /// notice, the sound and the text bubble that follow are the
    /// interface's and are not carried.
    pub(super) fn city_reduced(&mut self, attacker: Obj, b: usize, fresh: bool) {
        let who = self.owner_of(attacker);
        let bd = &self.buildings[b];
        if !self.building_is_city(b) || who == bd.owner || !bd.active || bd.damage < bd.hits_now() {
            return;
        }
        let Some(c) = bd.city else {
            return;
        };
        if let Obj::Unit(u) = attacker
            && self.ai_driven(who)
            && self.is_siege_unit(u)
            && self.unit_ai_bit(who)
            && let Some(slot) = self.army_of(u)
        {
            self.army_charge(who, slot, Obj::Building(b));
        }
        if !fresh && self.frame - self.cities[c].reduce_stamp < 300 {
            return;
        }
        self.cities[c].reduce_stamp = self.frame;
    }

    /// `Object::do_damage` (§7.1): computes, scales and delivers one hit from
    /// `attacker` to `target`. `count` is 8.8; `ammo` says a projectile
    /// delivered it; `splash` marks a fringe; `quiet` suppresses the
    /// target's opportunity response. Returns what `take_damage` did.
    #[allow(clippy::too_many_arguments)]
    pub fn do_damage(
        &mut self,
        attacker: Obj,
        target: Obj,
        angle: Angle,
        ammo: bool,
        count: i32,
        splash: bool,
        quiet: bool,
        frame: i64,
    ) -> Option<Taken> {
        self.do_damage_round(
            attacker, target, angle, ammo, false, count, splash, quiet, frame,
        )
    }

    /// [`Self::do_damage`] for a round's blow, which also says whether the
    /// round's ammo piece burns ([`combat::Projectile::flame`]): `ammo` is
    /// `Object::do_damage`'s `param_5 >= 0`, and `flame` its
    /// `verify_ammo_flags(ammo[param_5].gpiece, 0x10)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn do_damage_round(
        &mut self,
        attacker: Obj,
        target: Obj,
        angle: Angle,
        ammo: bool,
        flame: bool,
        count: i32,
        splash: bool,
        quiet: bool,
        frame: i64,
    ) -> Option<Taken> {
        if count < 1 || !self.struck(target) {
            return None;
        }
        // **A decoy's blow is no blow** (`Object::do_damage@0064a480`'s head,
        // item 1522): after the `count < 1` return, an attacker whose
        // vtable `+0x18` answers 1 (a unit) and whose `unit_masks & 1` is
        // set returns at once — before `get_damage`, before the first-wound
        // draw, before any bookkeeping. A General's decoys fire (the launch
        // draws are spent and the round flies its whole arc) and every round
        // lands on nothing: run666's `1/96`, `1/95` and `1/106`, three
        // Bombard decoys (`mana_burn` counting, `unit_masks` 0x5100d, 0x5100d,
        // 0x5100f) struck `0/2000` for nothing where `1/139` and `1/84`
        // (0x5100e) struck it for 135.
        if let Obj::Unit(a) = attacker
            && self.units[a].decoy
        {
            return None;
        }
        // **A hit on another player's object marks both leaders' treaties**
        // (`Object::do_damage@0064a480:327`–`:332`, after the scaling and
        // before `take_damage`): `treaties[victim] |= 2` on the attacker's
        // leader and `treaties[attacker] |= 2` on the victim's — bit 1, the
        // "has struck" bit beside `meet`'s bit 0. Unconditional on what the
        // hit does, so a nuke's ring marks the struck leader's row.
        let (hit_by, hit_who) = (self.owner_of(attacker), self.owner_of(target));
        if hit_by != hit_who {
            self.treaty_on(hit_by, hit_who, 2);
        }
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let at = self.attacker_side(attacker);
        let tt = self.target_side(target, attacker);
        let owner = self.owner_of(attacker) as usize;
        let pct = self.table_pct(attacker, target);
        let mut dmg = combat::get_damage(
            &self.tables.tuning,
            &ap,
            at,
            &tp,
            tt,
            self.attack_of(attacker),
            self.armor_of(target),
            pct,
            angle,
            splash,
            frame,
            &self.mods_of(owner),
        );
        // Step 2: the target's captain reacts, and the overkill record.
        if let Obj::Unit(t) = target
            && !quiet
        {
            self.target_opportunity(t, attacker, frame);
            // `0064a5dd`: the **struck** unit's `unit_masks & 0x10` — which
            // [`Sim::mark_contact`] raises on the responder, the captain the
            // walk reached — doubles the blow.
            if self.units[t].contact {
                dmg = dmg.wrapping_mul(2);
            }
            let u = &mut self.units[t];
            if u.combat.damage_frame == 0
                || frame - u.combat.damage_frame >= i64::from(self.tables.tuning.overkill_frames)
            {
                u.combat.damage_frame = frame;
                u.combat.damage_o = at.captain;
                u.combat.damage_who = i32::try_from(owner).unwrap_or(-1);
            }
        }
        // `do_damage@0064a480:194`–`:198`: a live, non-missile attacker's
        // blow scrambles a base's fighter (item 3990).
        if let Obj::Building(b) = target
            && !ap.has(combat::mask::MISSILE)
            && self.obj_alive(attacker)
        {
            let at = self.pos_of(attacker);
            self.attempt_launch(b, at);
        }
        // Step 5: scale.
        let dealt = combat::scale(
            dmg,
            count,
            matches!(attacker, Obj::Unit(_)),
            ammo,
            ap.ammo_per_att,
            ap.uber_size,
        );
        // Step 6: `dtype`, which is the death animation's gate (§42.1).
        // `get_damage` leaves 2, or 3 for an EXPLOSIVE/BOMBARD attacker;
        // `do_damage` then overrides it to **4** for an ammo whose graphic
        // piece carries flag `0x10`, and to **1** for a Build-proper
        // target — the second test runs after the first, so a burning round
        // on a building is 1. The `0x10` piece is `effects_graphics.xml`'s
        // `flame="1"` (item 3580: run1202b's Fire Lances `1/31` kills the
        // Citizen `0/10` on 3030 with `Unit::close`'s three draws).
        let dtype = if tt.build_proper {
            1
        } else if ammo && flame {
            4
        } else {
            combat::death_type(ap.obj_masks, !matches!(attacker, Obj::Unit(_)))
        };
        // `local_70` (`do_damage@0064a480`, `0064a8ac`): the struck city
        // building was still below its ceiling when the hit began.
        let fresh = matches!(target, Obj::Building(b)
            if self.building_is_city(b)
                && self.buildings[b].active
                && self.buildings[b].hits_now() > self.buildings[b].damage);
        // The city alarm, before the take (`0064a9f8`..`0064b2c4`).
        let alarm = self.city_alarm(attacker, target, ap.siege, dealt.whole, frame);
        // Step 7: take.
        let taken = self.take_damage_typed(target, dealt, attacker, frame, dtype, splash);
        let killed = matches!(taken, Taken::Died { .. });
        // **`Armies::emergency` is the CITY alarm's, and a unit never
        // reaches it** (`docs/ARMY.md` §15.8, item 399). The call at
        // `do_damage@0064a480:952` is
        // `if (local_30 != 0 && (leaders[param_2].leader_flags & 4) == 0)`,
        // after `take_damage`'s non-zero return has left the function (a
        // kill never reaches it), and `local_30` is [`Self::city_alarm`]'s
        // answer: a heavy hit on a city building, once in 300 frames
        // (item 2235, `docs/GOLDEN.md` §81).
        if alarm && !killed {
            let tw = self.owner_of(target);
            if (tw as usize) < self.armies.len() {
                self.armies_emergency(tw);
            }
        }
        // Step 8 and 9 on a building: a kill by a non-air, non-splash,
        // non-allied unit plunders it; a hit that did not kill a capturable
        // building by another player's unit is a capture attempt
        // (`docs/CITIES.md` §7.1, §8.3).
        if let Obj::Building(b) = target {
            let who = self.owner_of(attacker);
            let bowner = self.buildings[b].owner;
            if killed {
                if matches!(attacker, Obj::Unit(_))
                    && !splash
                    && !matches!(ap.domain, Domain::Air)
                    && !self.is_ally(who, bowner)
                {
                    self.plunder_kill(b, who);
                }
            } else {
                let captured = if let Obj::Unit(u) = attacker
                    && who != bowner
                    && self.capture_eligible(b)
                {
                    self.check_capture(b, u)
                } else {
                    false
                };
                if !captured {
                    self.city_reduced(attacker, b, fresh);
                }
            }
        }
        self.hits.push(combat::Hit {
            frame,
            attacker,
            target,
            damage: dmg,
            dealt,
            splash,
            killed,
        });
        // Step 9: a hit that did not kill breaks entrenchment.
        if !killed && let Obj::Unit(t) = target {
            self.units[t].combat.entrenched = false;
        }
        if let Obj::Building(b) = target {
            self.spill_onto_builders(attacker, b, angle, (ammo, flame), count, frame);
        }
        Some(taken)
    }

    /// **The city alarm** — `Object::do_damage@0064a480`'s building branch
    /// (`0064a9f8`..`0064b2c4`, the decompile's lines 428–735), run before
    /// the take, for a hit across owners on an active building that belongs
    /// to a city (`BuildData +0x72 >= 0`). Item 2235, `docs/GOLDEN.md` §81.
    ///
    /// ```text
    /// heavy = the attacker's type is_siege (UnitTypeData: unit_flags & 0x20000)
    ///      or damage + dealt >= hits / 10   (a fort or tower — type vslot +0x104,
    ///                                         ObjectTypeData::is_defense — or a city)
    ///                         hits / 4     (anything else)
    /// city c not the capital (city_flags & 0x10 clear):
    ///   heavy: alarm when frame − c.attack_stamp > 300;
    ///          c.attack_stamp = c.raid_stamp = frame
    ///   light: c.raid_stamp = frame
    /// the capital:
    ///   heavy: alarm when frame − victim.capital_stamp[attacker] > 300, and the
    ///          stamp takes the frame then; the city's own stamps are not written
    ///   light: c.raid_stamp = frame
    /// ```
    ///
    /// Answers the alarm, `local_30`, which gates `Armies::emergency`.
    /// The victim leader's `attack_stamp`/`raid_stamp`/`ally_stamp` rows,
    /// the notices and the sounds are not carried: the dump prints the
    /// three arrays only at `LEADERS=9`, and nothing here reads them.
    pub(crate) fn city_alarm(
        &mut self,
        attacker: Obj,
        target: Obj,
        siege: bool,
        whole: i32,
        frame: i64,
    ) -> bool {
        let (att, vic) = (self.owner_of(attacker), self.owner_of(target));
        if att == vic {
            return false;
        }
        let b = match target {
            Obj::Building(b) if self.buildings[b].active => b,
            // **The worker arm** (`0064a9xx`, the decompile's lines 400–
            // 424): a struck worker (`ObjectData::is_worker`) whose action
            // (`UnitData::get_action@00608450`) answers `get_gather_order`
            // (order vslot `+0xe4`, by the PDB's method record) with an
            // active building of the victim's in a city raids that city —
            // always the light arm, `local_34` still 0. run437's citizens
            // gathering at the Farm `1/2001`, struck on 671, 672 and 690.
            Obj::Unit(t) if self.is_worker(t) => {
                let gather = self
                    .action_of(t)
                    .and_then(|i| match self.units[t].orders[i].body {
                        crate::orders::Body::Gather(g) => Some(g.building),
                        _ => None,
                    });
                if let Some(g) = gather
                    && let Some(bd) = self.buildings.get(g)
                    && bd.active
                    && bd.owner == vic
                    && let Some(c) = bd.city
                {
                    self.cities[c].raid_stamp = frame;
                }
                return false;
            }
            _ => return false,
        };
        let Some(c) = self.buildings[b].city else {
            return false;
        };
        let heavy = (matches!(attacker, Obj::Unit(_)) && siege) || {
            let bd = &self.buildings[b];
            let defense = bd.ty.is_some_and(|t| {
                crate::build::is_fort(&self.tables.build_types, t)
                    || crate::build::is_tower(&self.tables.build_types, t)
            });
            let ceiling = if defense || self.building_is_city(b) {
                bd.hits_now() / 10
            } else {
                bd.hits_now() / 4
            };
            bd.damage + whole >= ceiling
        };
        if !heavy {
            self.cities[c].raid_stamp = frame;
            return false;
        }
        if self.cities[c].capital {
            let stamp = &mut self.ai[vic as usize].capital_stamp[att as usize];
            if frame - *stamp > 300 {
                *stamp = frame;
                return true;
            }
            return false;
        }
        let city = &mut self.cities[c];
        let alarm = frame - city.attack_stamp > 300;
        city.attack_stamp = frame;
        city.raid_stamp = frame;
        alarm
    }

    /// **A hit on a building spills onto the hands at work on it**
    /// (`Object::do_damage@0064a480`'s tail, `64c10c`..`64c4e3`, item
    /// 1182, `docs/GOLDEN.md` §50). Reached for a building target only
    /// (`64bec5`, `is_build`), after `take_damage`, whether it killed or
    /// not. For each cell of `circle_x/y[..circle_radius[1]]` round the
    /// building's cell, its object chain, in chain order:
    ///
    /// ```text
    /// an enemy of the attacker's (Search::valid_search mode 3), active and
    ///   on the map, whose FRONT order is BUILD_AT or REPAIR (valid_filter
    ///   9, UnitData::order_type) and whose action's target is this
    ///   building (valid_filter 10);
    /// not a Korean's under KOREAN_BUILD_UNDER_FIRE (has_tribe_bonus 0x10);
    /// vector_dist(it, building) <= max(x_size, y_size) * 192;
    /// the attacker: air, none; sea and is_siege, none (`64c3b3`,
    ///   ObjectData::is_siege@0046ef90);
    ///   a land siege type: count / 4, no reach test;
    ///   else vector_dist(it, building) <= max(max_range * 192, 0x180):
    ///   count / 8;
    /// do_damage(attacker, it, angle, ·, ammo, count', splash 0, quiet 1)
    /// ```
    ///
    /// Both divisions are the sign-fixed shifts of the listing (`cdq; and
    /// edx, 3|7; add; sar`), toward zero. SEAM: `num_guys` (§7.1), which
    /// this crate's `do_damage` does not carry, is passed through there.
    pub(super) fn spill_onto_builders(
        &mut self,
        attacker: Obj,
        b: usize,
        angle: Angle,
        (ammo, flame): (bool, bool),
        count: i32,
        frame: i64,
    ) {
        let ap = self.profile(attacker);
        let bp = self.profile(Obj::Building(b));
        let at = self.owner_of(attacker);
        let centre = self.buildings[b].pos;
        let home = centre.cell();
        let circ = crate::ai_place::circle();
        for k in 0..circ.radius[1] {
            let cell = crate::Cell::new(home.x + circ.x[k], home.y + circ.y[k]);
            if cell.x < 0
                || cell.y < 0
                || cell.x >= self.world.width()
                || cell.y >= self.world.height()
            {
                continue;
            }
            for u in self.cell_chain(cell) {
                let unit = &self.units[u];
                if !self.is_enemy(unit.owner, at) || !unit.alive() || !unit.on_map {
                    continue;
                }
                let front = unit.orders.front().map(|o| o.body);
                if !matches!(front, Some(crate::orders::Body::Build(x) | crate::orders::Body::Repair(x)) if x == b)
                {
                    continue;
                }
                if self.nation[unit.owner as usize].koreans
                    && self.tables.tuning.korean_build_under_fire != 0
                {
                    continue;
                }
                let d = vector_dist(unit.pos.x - centre.x, unit.pos.y - centre.y);
                if d > bp.x_size.max(bp.y_size) * 192 {
                    continue;
                }
                let share = match ap.domain {
                    Domain::Air => continue,
                    Domain::Sea if ap.siege => continue,
                    Domain::Land if ap.siege => (count + ((count >> 31) & 3)) >> 2,
                    _ => {
                        let reach = (self.max_range_of(attacker) * 192).max(0x180);
                        if d > reach {
                            continue;
                        }
                        (count + ((count >> 31) & 7)) >> 3
                    }
                };
                self.do_damage_round(
                    attacker,
                    Obj::Unit(u),
                    angle,
                    ammo,
                    flame,
                    share,
                    false,
                    true,
                    frame,
                );
            }
        }
    }

    /// `Object::take_damage` (§7.2) on a unit figure or a building.
    #[cfg(test)]
    pub(super) fn take_damage(
        &mut self,
        target: Obj,
        hit: Sixteenths,
        by: Obj,
        frame: i64,
    ) -> Taken {
        // Every caller outside `do_damage` is a test or the attrition
        // path, and `Object::die`'s `dtype` there is 0 — the argument
        // `Sim::attrition_tick` and the scripting paths pass, and the one
        // value `Unit::close`'s arm refuses (§42.1).
        self.take_damage_typed(target, hit, by, frame, 0, false)
    }

    /// [`Self::take_damage`] with `Object::take_damage`'s own `dtype`
    /// argument, which it forwards to `T.die(dtype, gpiece, angle)` and
    /// which is the whole of the death draw's gate (§42.1).
    pub(super) fn take_damage_typed(
        &mut self,
        target: Obj,
        hit: Sixteenths,
        _by: Obj,
        _frame: i64,
        dtype: i32,
        splash: bool,
    ) -> Taken {
        // `Object::take_damage@00652020:67-71`, first after the sixteenth
        // floor: a hit by anything but attrition (`param_5 == 0`), at
        // difficulty below 2, stamps the **struck object's owner's**
        // `frame_attacked` — human or computer, the leader of whatever was
        // hit. `Army::find_target`'s difficulty gate reads it for 7,200
        // frames (`docs/ARMY.md` §12, `docs/AI.md` §71).
        if self.ai_difficulty() < 2 {
            let owner = self.owner_of(target) as usize;
            if let Some(l) = self.ai.get_mut(owner) {
                l.frame_attacked = _frame;
            }
        }
        // `Object::take_damage@00652020:306-311`, ahead of the accumulate:
        // a unit hit by anything but attrition (`param_5 == 0`) marks its
        // whole squad in danger. Attrition never comes through here.
        if let Obj::Unit(i) = target {
            self.set_in_danger(i);
        }
        match target {
            Obj::Unit(i) => {
                // **The threshold is the figure's share of the squad, not
                // the squad's own number** (§7.2 step 8, §7.3, §41.4).
                // `myhits` is written onto every figure of a squad and
                // `take_damage` divides it on the way in, so a hoplite of
                // `UBER_SIZE` 3 and `HITS` 120 falls at 40; this crate
                // passed the whole 120 and its figures absorbed a squad's
                // worth apiece. run112's `1/8` reached `damage` 51 on
                // block 685 still standing where the original's died on
                // 683 at its fortieth point, and `extra 1/8` was that.
                //
                // `health` stays the **complement** of the dump's
                // `damage` against the squad-sized `myhits`, which is what
                // item 484's row pinned on 245,679 field-frames; the
                // accumulated whole points therefore go into `take` as
                // `max_health - health` and the share beside them.
                let u = &self.units[i];
                let uber = self.profile(target).uber_size;
                let alone = self.takes_remainder_share(i);
                let share = combat::share(u.max_health, uber, alone);
                let u = &self.units[i];
                let (taken, _, frac) =
                    combat::take(u.max_health - u.health, u.damage_frac, share, hit);
                let lost = match taken {
                    Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
                };
                let u = &mut self.units[i];
                u.damage_frac = frac;
                u.health -= lost;
                // `take_damage@00652020:619`: the whole points past the
                // figure's share, read before `die` — what a decoy's death
                // hands on to its captain below.
                let overflow = u.max_health - u.health - share;
                let decoy = u.decoy;
                // `take_damage@00652020:176`-`:214`: a Citizen struck rings
                // its city's bell before the kill arm (item 3984).
                self.peasant_alarm(i);
                if matches!(taken, Taken::Alive { .. }) {
                    self.struck_alive(i);
                }
                if matches!(taken, Taken::Died { .. }) {
                    let u = &mut self.units[i];
                    u.health = u.health.min(0);
                    // `take_damage@00652020:607`, before `die` (vslot
                    // `+0x158`): the kill's plunder (`docs/AI.md` §259).
                    if !splash && let Obj::Unit(k) = _by {
                        self.unit_plunder(i, k);
                    }
                    self.inca_refund(i, _by);
                    // `Object::die` is `close()` — the draw arm below —
                    // and **then** the slot hold, which is not gated on
                    // the draw arm's `dtype` at all (§42.3).
                    self.close_count(i);
                    self.relink_squad(i);
                    self.close_unit(i, dtype, _frame);
                    self.kill_contents(i);
                    self.kremlin_spy_closed(i);
                    self.close_dead_orders(i);
                    self.hold_dead_slot(i);
                    let keep = self.keeps_disc(i, dtype);
                    self.close_supply_in(i, keep);
                    self.forget(Obj::Unit(i));
                    self.clear_decoy_at_close(i);
                    // **A decoy's overflow strikes its captain**
                    // (`take_damage@00652020:619`–`631`, item 3393): after
                    // `die`, a figure that was a decoy (`unit_masks & 1`,
                    // read before `close` cleared it) with whole points
                    // past its share hands them to `get_captain` — the
                    // chain as `close` re-linked it — unless that is
                    // itself: `take_damage(overflow, 0, dtype, …, captain,
                    // who, 0)`, the captain its own attacker. A decoy's
                    // blow is `get_damage`'s step 17, so the squad goes
                    // down figure by figure on the one round. run1015b's
                    // packet: the tail `1/92` takes 13370, `1/90` the
                    // 13345 past its 39, `1/91` the 13327 past 1/90's.
                    if decoy && overflow > 0 {
                        let (c, who) = (self.units[i].combat.captain, self.units[i].owner);
                        let captain = (c != i32::from(self.units[i].index))
                            .then(|| {
                                self.units.iter().position(|u| {
                                    u.owner == who && i32::from(u.index) == c && u.alive()
                                })
                            })
                            .flatten();
                        if let Some(k) = captain {
                            let hit = combat::Sixteenths {
                                whole: overflow,
                                frac: 0,
                            };
                            self.take_damage_typed(
                                Obj::Unit(k),
                                hit,
                                Obj::Unit(k),
                                _frame,
                                dtype,
                                false,
                            );
                        }
                    }
                }
                taken
            }
            Obj::Building(b) => self.damage_building(b, hit, Some(_by), _frame, false),
        }
    }

    /// **A struck unit that survives raises `0x10` on its captain**
    /// (`Object::take_damage@00652020:345`–`350`, item 3983, `docs/AI.md`
    /// §415): after the hit is accumulated, a unit whose whole points taken
    /// (`+0x24`) are still under its figure's share sets
    /// `SubObjectData::flags |= 0x10` on `get_captain` (vslot `+0xe4`) —
    /// the bit `Unit::think`'s idle gate reads, carried as `cant_reach`.
    /// Attrition comes through here too (`suffer_attrition`'s slot `0x16c`
    /// is this function), so [`crate::suffer_view::SimSuffer`] calls it as
    /// well. A building's own arm (the same write on its captain, itself)
    /// is not carried: no building flag is compared.
    pub(crate) fn struck_alive(&mut self, i: usize) {
        let c = self.captain_of(i);
        self.units[c].cant_reach = true;
    }

    /// **The Despot's unit plunder** (`docs/AI.md` §259): `Object::take_damage
    /// @00652020`'s kill arm (`:520`–`:609`) and the EE arm of
    /// `Unit::plunder@00605b00` it calls, for a victim `v` killed by the unit
    /// `k`.
    ///
    /// The caller's gate: `v` is its squad's captain, the last figure of it
    /// (`curr_uber_size == 1`) and no decoy; the killer is a unit, the hit no
    /// splash (`param_9`), and the two leaders not allied. A victim that
    /// `is` a General, a caravan or a supply wagon plunders by its own
    /// constants; any other plunders only when `HeroesData::find_hero
    /// @0073a1b0` finds one of the killer's heroes of type THEDESPOT
    /// (`0x160`), else SPITAMENES (`0x16d`), active, on the map and within
    /// `get_radius × 0xc0` of the victim, and `THEDESPOT_UNIT_PLUNDER` is
    /// non-zero — and then `plunder(v, k, who, 0, 1)`.
    ///
    /// In `plunder` (`get_patch_version >= 9`, `00605f78`..): one
    /// `Random::get(0, 0xffff)` per good of the killer's leader available
    /// (`type_avail(g, 1)`), knowledge skipped, food to oil in order; the good
    /// whose `bucket + roll % 25` is least, first on a tie, takes
    /// `THEDESPOT_UNIT_PLUNDER`. `leaders +0x914` (the scenario's plunder
    /// scale, `-1.0` from `Leader::init` and written only by
    /// `ScenarioFuncSet::set_plunder_scale`) is negative and scales nothing.
    /// run943's Citizen `0/3`, killed on 10852 by `1/58` beside The Despot
    /// `1/46`: four draws (oil not yet available) and who=1's metal 44 → 59.
    ///
    /// **The Aztec slay arm** comes first (`:553`–`:555`, item 3657): a
    /// killer whose leader has tribe bonus 0 (the Aztecs) and whose type's
    /// trainer (`+0x40`) is **by equality** the Barracks (`0x1ab`), the
    /// Stable (`0x1ac`) or the Dock (`0x1b0`) calls `plunder(v, k, who, 1,
    /// 0)` whatever the victim, Despot or none. The pay is `(ages + 1) ×
    /// AZTEC_SLAY_BONUS` up to `AZTEC_SLAY_CAP`, in the good the same four
    /// draws pick; run1308b's Heavy Horse Archer `1/29` (a Stable type) kills
    /// the Citizen `0/13` on 2556 at Medieval (`ages` 2) and who=1's timber
    /// goes 12714 → 12759.
    ///
    /// The body is [`crate::plunder::plunder_steps`] over [`crate::plunder::SimPlunder`]
    /// (the original's whole function, swept against it in item 3356).
    ///
    /// A General, caravan or supply-wagon victim (`uflags2::FLAG2_GENERAL`,
    /// `CARAVAN`, `SUPPLY_OR_HERO`) plunders by its own constant, `p3` and
    /// `p4` both 0 (item 3788). The Inca refund is [`Sim::inca_refund`].
    ///
    /// SEAM: the victim redirect of a type with `+0x2b4 & 0x10` is not
    /// carried.
    pub(crate) fn unit_plunder(&mut self, v: usize, k: usize) {
        use crate::ai_load::uflags2;
        const DESPOT: i32 = 0x160;
        const SPITAMENES: i32 = 0x16d;
        let amount = self.tables.tuning.thedespot_unit_plunder;
        let victim = &self.units[v];
        if !self.is_captain(v) || victim.decoy || self.curr_uber_size(v) != 1 {
            return;
        }
        let (vw, at) = (victim.owner, victim.pos);
        let who = self.units[k].owner;
        if who == vw || self.is_ally(vw, who) {
            return;
        }
        let slay = self.nation[who as usize].aztecs
            && self.units[k]
                .ty
                .and_then(|t| self.trainer_ident(t))
                .is_some_and(|i| {
                    use crate::build::Ident;
                    matches!(i, Ident::Barracks | Ident::Stable | Ident::Dock)
                });
        if slay {
            crate::plunder::plunder_steps(
                &mut crate::plunder::SimPlunder { sim: self, v },
                k as i32,
                i32::from(who),
                1,
                0,
            );
            return;
        }
        let kind = self.units[v]
            .ty
            .map_or(0, |t| self.tables.unit_types[t].cols.unit_flags2);
        // `take_damage@00652020`'s `is(GENERAL)`/`is(CARA)`/`is(SUPPLYWAGON)`
        // arm (`LAB_0065332c`): `plunder(v, k, who, 0, 0)` — no Despot search,
        // the base constant in wealth (item 3788; run1461b's who=2 takes
        // `GENERAL_PLUNDER` 100 for who=1's Despot `1/28` on 1463).
        if kind & (uflags2::FLAG2_GENERAL | uflags2::FLAG2_CARAVAN | uflags2::SUPPLY_OR_HERO) != 0 {
            crate::plunder::plunder_steps(
                &mut crate::plunder::SimPlunder { sim: self, v },
                k as i32,
                i32::from(who),
                0,
                0,
            );
            return;
        }
        if amount == 0
            || !(self.hero_near(who, DESPOT, at, 0) || self.hero_near(who, SPITAMENES, at, 0))
        {
            return;
        }
        // `Unit::plunder(victim, k, who, 0, 1)`: the Extended Edition's arm,
        // over the crate's own buckets and draws.
        crate::plunder::plunder_steps(
            &mut crate::plunder::SimPlunder { sim: self, v },
            k as i32,
            i32::from(who),
            0,
            1,
        );
    }

    /// **The Inca refund** (item 3788, `docs/GOLDEN.md` §149.12):
    /// `Object::take_damage@00652020`'s `LAB_00653354` arm, reached by the
    /// kill arm's own gate — the victim `v` its squad's captain, the last
    /// figure (`curr_uber_size == 1`), no decoy, a killer — and run after
    /// the plunder block whether that block ran or not, so a building's
    /// kill and a splash kill refund too. When the victim's leader is not
    /// allied with the killer's (`is_ally` answers itself, so an own kill
    /// pays nothing), `INCA_REFUND` is non-zero and the victim's leader
    /// holds tribe bonus 2, it calls `Unit::plunder(v, v.index, v.who, 0,
    /// 0)`: `INCA_REFUND`% of the unit's price in each available good,
    /// added to the bucket uncapped. run1461b's who=2 (the Inca) takes
    /// `+16` food and `+11` timber on logger frame 860 and `+25` timber
    /// and `+25` metal on 864, the Bombard `2/23`'s death.
    pub(crate) fn inca_refund(&mut self, v: usize, by: Obj) {
        let victim = &self.units[v];
        if !self.is_captain(v) || victim.decoy || self.curr_uber_size(v) != 1 {
            return;
        }
        let vw = victim.owner;
        if self.tables.tuning.inca_refund == 0
            || self.is_ally(vw, self.owner_of(by))
            || !self.holdings[vw as usize].inca
        {
            return;
        }
        crate::plunder::plunder_steps(
            &mut crate::plunder::SimPlunder { sim: self, v },
            v as i32,
            i32::from(vw),
            0,
            0,
        );
    }

    /// The leader's per-type count at `+0x56fe + ti × 2` — the muster's
    /// count of type `ti` — non-zero: `has_general`'s early `return -1`.
    pub(crate) fn hero_counted(&self, who: Player, ti: i32) -> bool {
        self.tables
            .unit_types
            .iter()
            .position(|t| t.type_index == ti)
            .and_then(|r| self.muster.get(who as usize).and_then(|m| m.by_type.get(r)))
            .is_some_and(|&n| n != 0)
    }

    /// **`HeroesData::find_hero@0073a1b0`** behind a leader's per-type count
    /// (`+0x56fe + ti × 2`, the muster here): one of `who`'s heroes of type
    /// `ti`, active and on the map, with `vector_dist(hero, at) − slack ≤
    /// get_radius × 0xc0`. `slack` is the caller's sixth argument: 0 for a
    /// unit's plunder, a building's footprint for its own
    /// ([`Sim::plunder_kill`], `docs/AI.md` §261).
    pub(crate) fn hero_near(&self, who: Player, ti: i32, at: Pos, slack: i32) -> bool {
        self.hero_counted(who, ti)
            && (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == who
                    && x.type_index == ti
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                        - slack
                        <= self.hero_radius(h) * 0xc0
            })
    }

    /// `SpellType::cast_sabotage@006720e0`'s unit arm for the Sniper
    /// (`0x281`, item 2971, `docs/GOLDEN.md` §115): the target figure
    /// `die`s with `dtype` 2 — `Unit::die@0060eda0` through vslot `+0x158`,
    /// the combat death's tail without a wound.
    ///
    /// SEAM: the caster's leader counters (`+0xa58`, `+0xa5a`, `+0x80c`)
    /// and the score `+0x44 += cost / 5` for a squad captain, and the
    /// message and sound.
    pub(crate) fn cast_sniper_unit(&mut self, v: usize, dtype: i32) {
        self.die_with(v, dtype);
    }

    /// The `die o,who` cheat on a unit (`run_cmd`'s `die` arm,
    /// `007dde60`–`007ddf1d`): vslot `+0x158` with `(0, −1, 0)`,
    /// [`Sim::die_with`] at `dtype` 0 — no death draw (item 3989,
    /// `docs/GOLDEN.md` §182).
    pub fn cheat_die_unit(&mut self, v: usize) {
        self.die_with(v, 0);
    }

    /// `Object::die(o, dtype, −1, 0)` on a unit, through `Unit::die`
    /// (vslot `+0x158`): the combat death's tail without a wound — the
    /// Sniper's kill ([`Sim::cast_sniper_unit`]) and a disband
    /// ([`Sim::disband_unit`], item 3914).
    pub(crate) fn die_with(&mut self, v: usize, dtype: i32) {
        let frame = self.frame;
        self.units[v].health = 0;
        self.close_count(v);
        self.relink_squad(v);
        self.close_unit(v, dtype, frame);
        self.kill_contents(v);
        self.kremlin_spy_closed(v);
        self.close_dead_orders(v);
        self.hold_dead_slot(v);
        let keep = self.keeps_disc(v, dtype);
        self.close_supply_in(v, keep);
        self.forget(Obj::Unit(v));
        self.clear_decoy_at_close(v);
    }

    /// A unit's death by attrition: `Unit::suffer_attrition@005e1a10` calls
    /// the unit's `take_damage` (vslot `+0x16c`) with `dtype` **1**, not 0 —
    /// `(whole, sixteenths, 1, -1, 1, 0x55555555, ...)` — so `Object::die`
    /// runs the whole combat tail: the death animation's draw
    /// (`Unit::close+0xcb6`), the sound, and a Tank's disc left standing
    /// (`docs/GOLDEN.md` §135, run936: the original draws on 1890).
    pub(crate) fn die_by_attrition(&mut self, i: usize, frame: i64) {
        const ATTRITION_DTYPE: i32 = 1;
        self.close_count(i);
        self.relink_squad(i);
        self.close_unit(i, ATTRITION_DTYPE, frame);
        self.kill_contents(i);
        self.kremlin_spy_closed(i);
        self.close_dead_orders(i);
        self.hold_dead_slot(i);
        let keep = self.keeps_disc(i, ATTRITION_DTYPE);
        self.close_supply_in(i, keep);
        self.forget(Obj::Unit(i));
        self.clear_decoy_at_close(i);
    }

    /// `Object::close@00647160`'s `blocks_while_dead` arm: any death with a
    /// non-zero `param_1` (`dtype`) of a type carrying `x` leaves its
    /// occupancy disc standing until the death object's cull — a combat
    /// death or a Sabotage's boom alike (item 3228, `docs/GOLDEN.md` §130);
    /// attrition (`dtype` 1, `Unit::suffer_attrition`) keeps it like any other (item 3252, `docs/GOLDEN.md` §135).
    pub(super) fn keeps_disc(&self, i: usize, dtype: i32) -> bool {
        dtype != 0
            && self.units[i].on_map
            && self.units[i].ty.is_some_and(|t| {
                self.tables.unit_types[t].cols.unit_flags
                    & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                    != 0
            })
    }

    /// `SpellType::cast_sabotage@006720e0`'s building arm (item 2994,
    /// `docs/GOLDEN.md` §116): half the building's `hits_left`, floored at
    /// `COMMANDO_MIN_DAMAGE`, taken as `take_damage(dmg, 0, dtype 3, -1,
    /// attrition 1, ..)` — no attacker, so no latch and no first-wound
    /// draws. (The `this->field_4 == 0x4d / 0x4b` tests name a Type index
    /// no craft carries, so the Commando's floor stands.)
    ///
    /// SEAM: the caster's leader counters and score (`+0xa58`, `+0xa5a`,
    /// `+0x80c`; `+0x818` and `+0x44 += cost / 5` on the razed building),
    /// the message and the sound; the dead-target refund arm.
    pub(crate) fn cast_sabotage_building(&mut self, b: usize) {
        let frame = self.frame;
        let dmg = (self.buildings[b].health / 2).max(self.tables.tuning.commando_min_damage);
        self.damage_building(
            b,
            Sixteenths {
                whole: dmg,
                frac: 0,
            },
            None,
            frame,
            true,
        );
    }

    /// The **first wound**'s draws (§7.2 step 3), spent before the
    /// accumulate and only on the combat path.
    ///
    /// `Object::take_damage@00652020` reaches `+0xe1` when `damage == 0`
    /// and the object's vtable slot `+0x1c` answers 1 — a `Build` or a
    /// `Wall`, never a unit — and takes one roll there whatever kind of
    /// building it is. Only if that roll is `% 100 < 5` does the
    /// fort/`TEMPLE`/`TOWN` test run, and only then, with a siege
    /// attacker, the second roll at `+0x18b` that sizes a flock of birds.
    ///
    /// The gate reads `damage`, the whole-hit count — a hit that moves
    /// only `damage_frac` leaves it at zero and the next hit draws again.
    pub(super) fn first_wound_draws(&mut self, b: usize, by: Option<Obj>, attrition: bool) {
        if attrition || self.buildings[b].damage != 0 {
            return;
        }
        self.mark(SITE_FIRST_WOUND);
        if self.rng.roll() % 100 >= 5 {
            return;
        }
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let fortlike = crate::build::is_fort(&self.tables.build_types, ty)
            || crate::build::is(&self.tables.build_types, ty, crate::build::Ident::Temple)
            || crate::build::is(&self.tables.build_types, ty, crate::build::Ident::Town);
        if !fortlike {
            return;
        }
        // `if (param_7 < 0) goto LAB_006522f2` — no attacker, no flock
        // and no war declaration either.
        let Some(by) = by else { return };
        if !self.profile(by).siege {
            return;
        }
        self.mark(SITE_FIRST_WOUND_FLOCK);
        // `Objects::add_flock(x, y, -1, roll % 2 + 3)` — three or four
        // birds over the building, from its tile (`div_3_table[v >> 6]`,
        // `6521ba`–`6521e3`) and with no player's stamp to wait on
        // (`crate::flock`, item 1293).
        let n = self.rng.roll() % 2 + 3;
        let at = self.buildings[b].pos.tile();
        self.add_flock(at, None, n);
    }

    /// `Object::take_damage` on a building (§7.2): the under-attack latch, a
    /// site's lost progress and the building-on-site quadrupling, the city
    /// clamp — a city never dies, it sits at zero — and death through
    /// `Build::close`. `attrition` is the building-side attrition's call
    /// (`docs/CITIES.md` §9.5), which latches nothing.
    pub(crate) fn damage_building(
        &mut self,
        b: usize,
        hit: Sixteenths,
        by: Option<Obj>,
        frame: i64,
        attrition: bool,
    ) -> Taken {
        if !self.buildings[b].alive {
            return Taken::Alive { lost: 0 };
        }
        let mut hit = hit;
        let owner = self.buildings[b].owner;
        if !attrition
            && let Some(by) = by
            && self.owner_of(by) != owner
        {
            let bd = &mut self.buildings[b];
            bd.under_attack |= 0x3;
            bd.hit_frame = Some(frame);
            // **`city_flags |= 0xe`** (`Object::take_damage@00652020`, the
            // `orw $0xe, 0x4(%eax)` at `00652561`; `docs/COMBAT.md` §69):
            // under the same `param_5 == 0 && who != param_8` as the latch,
            // and whatever the latch's peasant arm decided, a building that
            // belongs to a city (`BuildData +0x72 >= 0`) marks its city
            // under attack, attacking and ever attacked. The first of the
            // three is the city heal's veto.
            if let Some(c) = self.buildings[b].city {
                let cd = &mut self.cities[c];
                cd.no_heal = true;
                cd.attacking = true;
                cd.ever_attacked = true;
            }
        }
        self.first_wound_draws(b, by, attrition);
        let site = !self.buildings[b].active && self.buildings[b].ty.is_some();
        if site && matches!(by, Some(Obj::Building(_))) {
            hit.whole *= 4;
            hit.frac *= 4;
        }
        let bd = &self.buildings[b];
        let share = bd.health;
        let (taken, _, frac) = combat::take(0, bd.damage_frac, share, hit);
        let lost = match taken {
            Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
        };
        if site {
            // **The progress a site loses is the points the blow took,
            // carry and all** (item 1350): the listing rewrites `param_1`,
            // the blow's whole hits, as `whole + (frac + hit.frac) / 16`
            // at `0065230b`, and the `imul $0x32` at `0065239b` reads it
            // back. run514's Keep site, a blow of 4 whole and 5 sixteenths:
            // 200 off its `job_counter` on 706, and 250 on 738, where the
            // sixteenths carried. **An aircraft's blow costs nothing**
            // (`0065234e`–`0065237c`): an attacker in use whose type's
            // `domain` (`+0x218`) is 2 sets the skip, and so does
            // attrition (`param_5`, `00652341`).
            let air = by.is_some_and(|o| matches!(self.profile(o).domain, Domain::Air));
            if !attrition && !air {
                let bd = &mut self.buildings[b];
                bd.job_counter = (bd.job_counter - combat_progress_lost(lost)).max(0);
            }
        }
        let city = self.building_is_city(b);
        let bd = &mut self.buildings[b];
        bd.damage_frac = frac;
        bd.damage += lost;
        let hits = bd.hits_now();
        if city && bd.active {
            // The clamp: a city never dies from damage. **It keeps the
            // fraction** (`Object::take_damage@00652020`, the `flags & 0x20`
            // arm: `damage = hits`, `flags |= 0x10`, and `+0x3b` untouched;
            // `docs/AI.md` §230): run866's capital at its ceiling steps
            // `damage_frac` 10, 15, 4, 9 … from 5161.
            if bd.damage >= hits {
                bd.damage = hits;
                if !bd.garrison.is_empty() {
                    bd.eject_pending = true;
                }
            }
            bd.sync_health();
            return Taken::Alive { lost };
        }
        bd.sync_health();
        if matches!(taken, Taken::Died { .. }) {
            bd.damage = hits;
            bd.sync_health();
            self.die_building(b);
        } else if site && bd.construct_hits <= bd.damage && !bd.garrison.is_empty() {
            bd.eject_pending = true;
        }
        taken
    }

    /// **`Unit::close@0060ee50`'s count, at `0060f3db`** — the general
    /// death path's `track_unit_type(type, −1)`, with `control` and
    /// `active` beside it (`docs/AI.md` §232; parked 1092, 1328).
    ///
    /// The listing takes the count off only for the squad's head
    /// (`+0x8e < 0`) when no live figure follows it: a head whose next
    /// figure (`+0x90`) is active hands the squad down at `0060f679`
    /// instead, and the squad is still one unit. Then the gate
    /// [`Sim::counts_in_muster`] shares with `set_type`'s `+1` — a type
    /// with population, and `unit_masks & 1` (the decoy bit) clear. Called
    /// before [`Self::relink_squad`], which is `close`'s relink after it.
    /// run867's who=1 kept three shot-down Jet Fighters in `num_units`
    /// (7 against 4), and its next Fighter's ramp ran twelve frames long.
    ///
    /// **A government hero is counted off with no population** (item 3794,
    /// `docs/GOLDEN.md` §151.1): the gate's `control_cost == 0` arm
    /// (`0060f350`..`0060f3d9`) goes on to `0060f3db` when the unit
    /// `is(0x134)` or `UnitData::is_gov_hero@0046ce70` answers — the type's
    /// `+0x2b4 & 0x4000000`, `OBJ_MASK`'s hero digit, [`Sim::type_is_hero`].
    /// The Despot is `POP 0`: the first melee's who=1 kept its `1/28` in
    /// `num_units[302]` from its death on 1463 to the close.
    ///
    /// SEAM: the `is(0x134)` half of that arm is not read here, and answers
    /// no.
    pub(crate) fn close_count(&mut self, i: usize) {
        self.gov_hero_closed(i);
        // `Unit::close@0060ee50:157-170`, ahead of the count and ungated by
        // the squad: an American Barracks unit that is not a scout raises
        // its leader's `0x2000000` (item 3384, `docs/AI.md` §292).
        if let Some(ty) = self.units[i].ty {
            let who = self.units[i].owner;
            if self.american_barracks_unit(who, i, ty) {
                self.economy_changed(who);
            }
        }
        let u = &self.units[i];
        if u.o_up.is_some() || u.o_down.is_some_and(|d| self.units[d].alive()) {
            return;
        }
        // `Unit::close@0060ee50:287`-`:289`: the hotkey arm sits on the
        // squad gate alone, below the count's decoy and population tests,
        // and reaches `Build::replace_gather` (item 3989).
        if u.owner < 8 {
            self.replace_gather(i);
        }
        let u = &self.units[i];
        if u.decoy {
            return;
        }
        let Some(ty) = u.ty else { return };
        if self.tables.unit_types[ty].price.pop == 0 && !self.type_is_hero(ty) {
            return;
        }
        // Gaia's animals carry population in some rows and hold no muster
        // here: this crate keeps one per player slot.
        let who = u.owner;
        if (who as usize) < self.muster.len() && (who as usize) < self.ai.len() {
            self.track_unit_type(who, ty, -1);
        }
    }

    /// **`Unit::close@0060ee50`'s squad relink** (`docs/COMBAT.md` §47.2),
    /// from the listing, `0060f28e`-`0060f758`. It sits above the death
    /// draw's `dtype` gate and under nothing but the slot's `flags & 1`, so
    /// it runs on every death.
    ///
    /// The dying figure is taken out of its chain and put back **at the
    /// tail**, where `Unit::repair_damage@0060de10` finds its slot to regrow
    /// a figure into:
    ///
    /// ```text
    /// if o_up >= 0:  above.o_down = this.o_down ; edi = 0
    /// if o_down >= 0:
    ///     below.o_up = this.o_up                 ; a dying head's -1
    ///     if below.flags & 1:  goto append
    /// if edi != 0:   the whole-squad arm         ; nothing left to lead
    /// append:
    ///     tail = walk o_down from this.o_down, or this if it has none
    ///     if tail != this:  tail.o_down = this ; this.o_down = -1 ; this.o_up = tail
    ///     else:             above.o_down = this ; this.o_down = -1 ; this.o_up = above
    /// ```
    ///
    /// So a dying tail leaves its captain's `o_down` where it was, and a
    /// dying **head** hands the squad to the figure below it, whose `o_up`
    /// is now the head's own −1: `UnitData::is_captain` is `o_up < 0`, and
    /// the new head is a captain from this frame. run112 prints both. After
    /// `0/11` dies on 729, `0/10` still carries `o_down 11`. After `1/6` dies
    /// on 743, `1/7` carries `o_up -1` and `o_down 8`, a chain of 7 → 8 → 6.
    ///
    /// This crate caches the captain in two places, [`crate::Unit::captain`]
    /// and [`combat::State::captain`], which the original reads live
    /// (`UnitData::get_captain@00610ab0` walks `o_up`). Both are re-pointed
    /// down the whole chain here, dead slots included, because a dead slot's
    /// `get_captain` walks to the new head as well.
    pub(crate) fn relink_squad(&mut self, i: usize) {
        let (up, down) = (self.units[i].o_up, self.units[i].o_down);
        let head = up.is_none();
        if let Some(a) = up {
            self.units[a].o_down = down;
        }
        let mut append = !head;
        if let Some(b) = down {
            self.units[b].o_up = up;
            append |= self.units[b].alive();
        }
        if !append {
            // The whole-squad arm (`Unit::close@0060ee50:292`-`316`): the last of the
            // squad dies, and every figure still linked below it — the dead
            // tail — has both links cleared, then this one. Only then is a
            // dead number a captain (`o_up < 0`) to `Objects::find_free`'s
            // reuse test; a dead tail of a live squad stays linked and its
            // number is held (`docs/AI.md` §324.2).
            let mut at = down;
            for _ in 0..self.units.len() {
                let Some(n) = at else { break };
                at = self.units[n].o_down;
                self.units[n].o_down = None;
                self.units[n].o_up = None;
            }
            self.units[i].o_down = None;
            self.units[i].o_up = None;
            return;
        }
        let mut tail = i;
        let mut at = down;
        // Bounded by the list: a cycle would hang the original, and here it
        // stops rather than spins.
        for _ in 0..self.units.len() {
            let Some(n) = at else { break };
            tail = n;
            at = self.units[n].o_down;
        }
        if tail != i {
            self.units[tail].o_down = Some(i);
            self.units[i].o_up = Some(tail);
        } else if let Some(a) = up {
            self.units[a].o_down = Some(i);
        }
        self.units[i].o_down = None;
        if head && let Some(b) = down {
            self.units[b].captain = true;
            let c = i32::from(self.units[b].index);
            let mut at = Some(b);
            for _ in 0..self.units.len() {
                let Some(n) = at else { break };
                self.units[n].combat.captain = c;
                at = self.units[n].o_down;
            }
        }
    }

    /// **`Unit::set_in_danger(this, 0)@005fcfb0`** — the squad's
    /// in-danger mark (`docs/ORDERS.md` §22).
    ///
    /// It climbs `o_up` to the captain without asking whether a figure is
    /// alive, then walks `o_down`. Each figure it reaches gets
    /// `unit_masks |= 4` ([`crate::Unit::in_danger`]) and
    /// `guy_flags |= 0x20` on every guy ([`crate::Unit::guy_flag_0x20`]).
    /// The walk stops at the first figure whose slot is not active
    /// (`flags & 1`). The captain it starts from is marked unasked.
    pub(crate) fn set_in_danger(&mut self, u: usize) {
        let units = &self.units;
        let marked = crate::squad_walk(
            u,
            units.len(),
            |i| units[i].o_up,
            |i| units[i].o_down.filter(|&d| units[d].alive()),
        );
        for n in marked {
            self.units[n].in_danger = true;
            self.units[n].guy_flag_0x20 = true;
        }
    }

    /// **`Unit::close@0060ee50`'s death-animation arm** (`docs/COMBAT.md`
    /// §42.1) — one `Random::get(0, 0xffff)` per death, and the
    /// `DEATH_OBJS` record the dump then prints.
    ///
    /// The listing's gate, in its own order (`0060fa4c`-`0060fa8f`): the
    /// slot answers `SubObjectData::is_active` and `UnitData::is_on_map`,
    /// and `unit_masks & 1` — the decoy bit — is clear. Above it the
    /// function's own `local_1c`, which is set only on the arm that
    /// unblocks four tiles for a **Merchant**, a Dutch Merchant or a Fur
    /// Trapper (type index `0x3d`, `0x3e`, `400`); those three take no
    /// death draw. And above everything, `dtype != 0`: the argument
    /// `Object::die` was called with, which combat always fills and the
    /// disband and scripting paths leave at zero.
    ///
    /// **`is_active` is the slot's allocation flag, not its health.** The
    /// object is still allocated and still on the map while `close` runs
    /// — it is `Object::close`, at the foot of this function, that takes
    /// it off — so the gate this crate applies is `on_map` and the decoy
    /// bit, and never [`crate::Unit::alive`], which the caller has
    /// already cleared.
    pub(crate) fn close_unit(&mut self, i: usize, dtype: i32, frame: i64) {
        if dtype == 0 {
            return;
        }
        let u = &self.units[i];
        if !u.on_map || u.decoy {
            return;
        }
        if self.profile(Obj::Unit(i)).is(combat::role::IS_MERCHANT) {
            return;
        }
        self.mark(SITE_DEATH_ANIM);
        let roll = self.rng.roll() % 2;
        let mut cur_anim = combat::death_anim(dtype, roll);
        if dtype == 4 {
            // The ammo-graphic death **replaces** the anim the first draw
            // chose with one from a second, and rolls its own facing
            // where every other death takes the attack's angle. The first
            // draw is still spent, and its value discarded. A burning
            // round's kill (`flame="1"`, [`combat::Projectile::flame`]).
            self.mark(SITE_DEATH_ANIM_ALT);
            let alt = self.rng.roll() % 2;
            cur_anim = combat::death_anim(4, alt);
            self.mark(SITE_DEATH_ANIM_FACING);
            let _ = self.rng.roll();
        }
        // **An aircraft shot down falls, and leaves no death object**
        // (item 1102, `docs/COMBAT.md` §81): `Objects::kill_guy`'s arm for
        // a guy of `CAT` Air that is not a missile.
        if self.crashes(i) {
            return;
        }
        // **Every figure's own disc is cleared, the crew's too** (item
        // 3787, `docs/GOLDEN.md` §150.2). `Unit::close`'s two loops — the
        // squad's `0 .. guy_mark` and the crew's `squad_size .. guy_num` —
        // call `Objects::kill_guy@00659410` on each figure, and its last
        // arm clears `radius[coll_size]` around **that figure's** cell
        // unless the type's vslot `+0x120` (`blocks_while_dead`) answers.
        // `remove_from_world` below takes guy 0's again; a crew figure
        // standing a cell off its engine punches its own disc out of any
        // neighbour's block, and nothing puts it back but the neighbour's
        // sixty-fourth-frame repaint.
        if !self.units[i].ty.is_some_and(|t| {
            self.tables.unit_types[t].cols.unit_flags & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                != 0
        }) {
            for k in 0..self.units[i].guys.len() {
                self.coll_kill_guy(i, k);
            }
        }
        let figures = self.death_figures(i, cur_anim);
        let life = self.death_life(i, cur_anim);
        let u = &self.units[i];
        let (who, o) = (i32::from(u.owner), i32::from(u.index));
        self.deaths.push(combat::Death {
            who,
            o,
            first_frame: frame,
            cur_anim,
            life,
            figures,
        });
        // `Objects::kill_guy`'s `add_death`, one call a figure, in the
        // order `Unit::close`'s loops reach them.
        for k in 0..figures.n {
            self.add_death(combat::DeathSlot {
                who,
                o,
                first_frame: frame,
                figure: k,
            });
        }
    }

    /// Whether the record a pool slot names is still live on `self.frame`,
    /// before this frame's `DeathObj::inc_time`: its [`combat::Death`]
    /// stands and that figure's life has not run out.
    pub(super) fn death_slot_live(&self, r: &combat::DeathSlot) -> bool {
        let age = self.frame - r.first_frame;
        self.deaths.iter().any(|d| {
            d.who == r.who
                && d.o == r.o
                && d.first_frame == r.first_frame
                && r.figure < d.figures.n
                && age < d.figures.life[usize::from(r.figure)]
        })
    }

    /// `DeathObjData::blocking@008d4a90`: the type of the object holding
    /// the record's number answers vslot `+0x120`, `blocks_while_dead`.
    pub(super) fn death_slot_blocks(&self, r: &combat::DeathSlot) -> bool {
        self.units
            .iter()
            .rposition(|u| i32::from(u.owner) == r.who && i32::from(u.index) == r.o)
            .and_then(|u| self.units[u].ty)
            .is_some_and(|t| {
                self.tables.unit_types[t].cols.unit_flags
                    & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                    != 0
            })
    }

    /// **`Objects::add_death@00653b60`'s slot** (item 3971, `docs/GOLDEN.md`
    /// §177): the pool is [`combat::DEATH_POOL`] slots and nothing grows
    /// it. The walk takes the first slot whose record is not valid; on the
    /// way it remembers the oldest record whose type does not block while
    /// dead — `first_frame < best`, `best` starting at `frame + 1`, so the
    /// lowest slot wins a tie — and when no slot is free that record is
    /// overwritten, or slot 0 when every record blocks. An overwritten
    /// record that blocks has its disc cleared first
    /// (`DeathObj::clear_blocking`); this crate leaves that to the cull
    /// below, the same frame, since only the all-blocking fallback can
    /// reach it.
    ///
    /// The evicted record ends where it stands: its figure's life is cut
    /// to its age, so it bumps its number no more
    /// ([`Self::death_objs_inc_time`]), and a [`combat::Death`] left with
    /// no live figure is culled this frame and leaves the dump's list.
    pub(crate) fn add_death(&mut self, rec: combat::DeathSlot) -> usize {
        if self.death_pool.len() < combat::DEATH_POOL {
            self.death_pool.resize(combat::DEATH_POOL, None);
        }
        let mut best = (0usize, self.frame + 1);
        let mut free = None;
        for k in 0..combat::DEATH_POOL {
            let Some(r) = self.death_pool[k].filter(|r| self.death_slot_live(r)) else {
                free = Some(k);
                break;
            };
            if !self.death_slot_blocks(&r) && r.first_frame < best.1 {
                best = (k, r.first_frame);
            }
        }
        let k = free.unwrap_or(best.0);
        if let Some(old) = self.death_pool[k].filter(|r| self.death_slot_live(r)) {
            let age = self.frame - old.first_frame;
            if let Some(d) = self
                .deaths
                .iter_mut()
                .find(|d| d.who == old.who && d.o == old.o && d.first_frame == old.first_frame)
            {
                d.figures.life[usize::from(old.figure)] = age;
                let n = usize::from(d.figures.n).min(combat::Figures::MAX);
                let last = d.figures.life[..n].iter().copied().max().unwrap_or(age);
                d.life = (last != i64::MAX).then_some(last);
            }
        }
        self.death_pool[k] = Some(rec);
        k
    }

    /// **`Object::close@00647160`'s contents arm on a unit** (item 3862,
    /// `docs/GOLDEN.md` §153.6): an object on the map (vslot `+0xbc`) that
    /// holds anything (`num_inside(1) != 0`) ejects it only when it cannot
    /// carry air **and** its vslot `+0x18` answers 0 — and a `Unit`'s
    /// `+0x18` is the folded `return 1` (`0041e0e0`), so a unit never
    /// ejects: it takes `Object::kill_contents@00645550`, which for each
    /// head of `inside_down` calls `Unit::come_out(p, 0)` and
    /// `Unit::die_uber(p, 0)` — `Object::die(p, 0, −1, 0)` on the squad,
    /// no death draw. A transport sunk at sea takes its riders with it:
    /// run1483's Freighter `1/54` and the Elite Special Forces `1/0`
    /// aboard it. A boat off the map (inside an oil platform) takes the
    /// `remove_from_inside` arm instead ([`Sim::boat_dies_into`]).
    pub(crate) fn kill_contents(&mut self, boat: usize) {
        if !self.units[boat].on_map {
            return;
        }
        let riders: Vec<usize> = (0..self.units.len())
            .filter(|&r| self.units[r].inside_unit == Some(boat) && self.units[r].alive())
            .collect();
        for r in riders {
            self.units[r].inside_unit = None;
            self.die_quietly(r);
        }
    }

    /// `DeathObj body_fade_end`, in frames: a land unit whose death piece
    /// has no skeleton lives its animation and this (`DeathObj::inc_time
    /// @008d5240`, `.rdata` `0002:378732`; `docs/GOLDEN.md` §127).
    pub(crate) const BODY_FADE_END: i64 = 135;
    /// `corpse_fade_end`, `.rdata` `0002:378740`: the same, for a piece
    /// with a skeleton (a corpse).
    pub(crate) const CORPSE_FADE_END: i64 = 627;

    /// How many frames the death object `close_unit` makes lives
    /// (`DeathObj::inc_time@008d5240`): the death piece's animation packet
    /// length for `cur_anim`, plus [`Self::CORPSE_FADE_END`] when the piece
    /// has a skeleton (`DeathObj +0x44`, `RData::unit_data +0x20`),
    /// [`Self::BODY_FADE_END`] when it has none, and nothing for a sea
    /// unit (`type +0x218 == 1`). `None` when the install does not name
    /// the piece's slot.
    pub(super) fn death_life(&self, i: usize, cur_anim: i32) -> Option<i64> {
        // One `DeathObj` per guy, each on its own `gpiece` (ch94's four-man
        // crew prints four, `gpiece` 219, 12891, 25563, 38235): the unit's
        // number is held by the last to end.
        let mut life = None;
        for k in 0..self.units[i].guys.len() {
            life = life.max(Some(self.figure_life(i, k, cur_anim)?));
        }
        life
    }

    /// One guy's record's life, as [`Self::death_life`] reads it.
    pub(super) fn figure_life(&self, i: usize, k: usize, cur_anim: i32) -> Option<i64> {
        let anim = i8::try_from(cur_anim).ok()?;
        let sea = self.units[i].kind.domain == crate::attrition::Domain::Sea;
        let gpiece = self.units[i].guys[k].gpiece;
        if gpiece < 0 || !self.art.piece_lengths.contains_key(&gpiece) {
            return None;
        }
        let frames = i64::from(self.slot_length(i, gpiece, anim)?);
        let fade = if sea {
            0
        } else if self.art.piece_skeletons.contains(&gpiece) {
            Self::CORPSE_FADE_END
        } else {
            Self::BODY_FADE_END
        };
        Some(frames + fade)
    }

    /// **Every guy's record** (item 3784, `docs/GOLDEN.md` §149.10): the
    /// lives [`Self::hold_frames_tick`] counts, one bump a record a frame.
    /// A record whose piece the install does not describe lives for good.
    pub(super) fn death_figures(&self, i: usize, cur_anim: i32) -> combat::Figures {
        let mut f = combat::Figures::one(None);
        let n = self.units[i].guys.len().clamp(1, combat::Figures::MAX);
        f.n = n as u8;
        for k in 0..n {
            f.life[k] = if k < self.units[i].guys.len() {
                self.figure_life(i, k, cur_anim).unwrap_or(i64::MAX)
            } else {
                i64::MAX
            };
        }
        f
    }

    /// **`Object::die@00647080`'s tail** — `hold_frames = max(hold_frames,
    /// 1, for every live ammo this object fired: nuke_effect[0x108] + 1 +
    /// total_time − cur_time)` (§11, §42.3). The slot is held until the
    /// dead archer's last arrow has landed.
    ///
    /// The ammo is matched on its **shooter**: `+0x3c`/`+0x40`, which
    /// `Ammo::init` fills from the firing object, and the same pair
    /// `Ammo::inc_time` reads for its own per-frame bump. `nuke_effect`'s
    /// term is [`Self::NUKE_SPREAD`], the 30 `Nuke::init` writes — on
    /// every ammo, not a nuke's alone (item 1594, `docs/AI.md` §165): the
    /// listing adds `[0xc0a888]` at `64710b` with no test of the round.
    ///
    /// **And the `hold_frames` it maxes against is already
    /// [`Self::CLOSE_HOLD`]** (§59.3): `die` calls `close` through vslot
    /// `+0x150` first, and `Unit::close` ends in `Object::close`, whose
    /// last write to an active object is `hold_frames = 0x1e`. So every
    /// death holds its number thirty frames, whatever `dtype` it carried —
    /// the transport that dies putting its passenger ashore
    /// ([`Sim::disembark`]), attrition, the upgrade's squad trim.
    pub(crate) fn hold_dead_slot(&mut self, i: usize) {
        // The type's `+0x1fc` gate is a field this crate does not carry (no
        // shipped type with a round in flight is known to hold it zero), so
        // it is read as set; `die_hold_steps` takes it for the sweep.
        let (who, idx) = (self.units[i].owner, self.units[i].index);
        let ammo: Vec<DieAmmo> = self
            .projectiles
            .iter()
            .filter(|p| p.shooter == Obj::Unit(i))
            .map(|p| DieAmmo {
                flags: 1,
                who: i32::from(who),
                idx: i32::from(idx),
                total: p.total_time,
                cur: p.cur_time,
            })
            .collect();
        let hold = die_hold_steps(
            u16::try_from(Self::CLOSE_HOLD).expect("0x1e"),
            who,
            idx,
            true,
            Self::NUKE_SPREAD,
            &ammo,
        );
        self.units[i].hold_frames = i32::from(hold);
    }

    /// **`Object::close@00647160`'s hold**: the last write it makes to an
    /// object that was active, after `Objects::remove`, is
    /// `hold_frames = 0x1e` — on both of its arms, so on every close
    /// (`docs/COMBAT.md` §59.3). `Objects::process_all` takes one off per
    /// frame, and `Objects::find_free` will not hand the number out until
    /// it is zero.
    pub(crate) const CLOSE_HOLD: i32 = 0x1e;

    /// **`nuke_effect +0x108`**, a nuke's spread time in frames:
    /// `Nuke::init@0092c960` writes `0x1e` to it (`92c987`, `0xc0a888`), and
    /// nothing in the simulation writes it again (`Nuke::registration`
    /// exposes it as a tuning variable). `Nuke::do_damage` reads it as the
    /// blast's growth after `+0x104`'s 10 frames, and `Object::die` adds it
    /// to every live round's remaining time when it holds a dead shooter's
    /// number (item 1594, `docs/AI.md` §165).
    pub(crate) const NUKE_SPREAD: i32 = 0x1e;

    /// **`Ammo::inc_time@0067d380`'s first statement**: every frame, a
    /// shot in flight whose shooter is no longer active bumps the
    /// shooter's `hold_frames` (§42.3). On dead slots only, which is why
    /// the dump prints `hold_frames 0` on every living unit of every
    /// capture — the row `crate::diff::compare` now carries. The death
    /// objects' own bump is [`Self::death_objs_inc_time`], after the walk.
    pub(crate) fn hold_frames_tick(&mut self) {
        for k in 0..self.projectiles.len() {
            // A crash round bumps its own plane's slot in its step
            // ([`Sim::process_projectiles`]).
            if self.projectiles[k].crash.is_some() {
                continue;
            }
            let Obj::Unit(s) = self.projectiles[k].shooter else {
                continue;
            };
            if !self.active(Obj::Unit(s)) {
                self.units[s].hold_frames += 1;
            }
        }
    }

    /// **`DeathObj::inc_time@008d5240`'s first statement**: every frame, a
    /// death object bumps its own object's `hold_frames`, and then ends at
    /// its life ([`Self::cull_deaths`]).
    ///
    /// **After the ammo walk, not before it** (item 3873, `docs/GOLDEN.md`
    /// §156.1): `Objects::inc_time@0065db70` runs every `Ammo::inc_time`
    /// (line 74) and then every live record (line 86), so a ship a round
    /// sinks gets its record's first bump on its own frame. The hold it
    /// keeps while the record lives is then 31, not 30 — run1496's packet
    /// at 1539 reads the dead Carrier `1/18`'s `+0x32` at 2 where the
    /// bump ahead of the walk left 1, and the Jet that ours put into 18 on
    /// tick 1539 takes 41 in theirs.
    pub(crate) fn death_objs_inc_time(&mut self) {
        for k in 0..self.deaths.len() {
            let (who, o) = (self.deaths[k].who, self.deaths[k].o);
            // **One bump a record** (item 3784, `docs/GOLDEN.md` §149.10):
            // `DeathObj::inc_time`'s first statement is on each guy's
            // record, and `process_all` takes one off a frame — so a dead
            // Bombard's four records hold its number long past their cull.
            let bumps = self.deaths[k]
                .figures
                .live(self.frame - self.deaths[k].first_frame);
            // The object in the slot, dead or not: `unit_by_o` finds only
            // the living, and this bump is on the dead (§59).
            if let Some(u) = self
                .units
                .iter()
                .rposition(|u| i32::from(u.owner) == who && i32::from(u.index) == o)
            {
                self.units[u].hold_frames = self.units[u].hold_frames.saturating_add(bumps);
            }
        }
        self.cull_deaths();
    }

    /// **The death object's end** (`DeathObj::inc_time@008d5240`, item
    /// 3205, `docs/GOLDEN.md` §127): its counter, one per call and zero at
    /// `init`, reaches `life`, the record is zeroed and — for a type whose
    /// `blocks_while_dead` kept the disc — `clear_blocking` takes the disc
    /// out of the occupancy index. The bump above has already run, as the
    /// listing's first statement does.
    pub(super) fn cull_deaths(&mut self) {
        let frame = self.frame;
        let mut k = 0;
        while k < self.deaths.len() {
            let d = self.deaths[k];
            let ended = d.life.is_some_and(|l| frame - d.first_frame + 1 >= l);
            if !ended {
                k += 1;
                continue;
            }
            self.deaths.remove(k);
            if let Some(u) = self
                .units
                .iter()
                .rposition(|u| i32::from(u.owner) == d.who && i32::from(u.index) == d.o)
            {
                let blocks = !self.units[u].alive()
                    && self.units[u].ty.is_some_and(|t| {
                        self.tables.unit_types[t].cols.unit_flags
                            & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                            != 0
                    });
                if blocks && self.units[u].coll_at.is_none() {
                    self.coll_remove(u);
                }
            }
        }
    }
}

/// One live record of `GameAccess::objects`' array as `Object::die@00647080`
/// reads it: the flag byte (`+4`), the shooter pair (`+0x3c` owner, `+0x40`
/// slot) and the two times (`+0x24` current, `+0x28` total).
#[derive(Clone, Copy)]
pub struct DieAmmo {
    pub flags: u8,
    pub who: i32,
    pub idx: i32,
    pub total: i32,
    pub cur: i32,
}

/// **`Object::die@00647080`'s tail** after the `close` it calls through
/// vslot `+0x150` (which leaves `hold`): when the type's `+0x1fc` is set,
/// every record with `flags & 3 != 0` whose shooter pair is this object
/// raises the hold to `nuke + 1 + (total - cur)` (32-bit wrapping, compared
/// signed); the hold is at least 1, and is stored as a short.
pub fn die_hold_steps(
    hold: u16,
    who: u8,
    idx: i16,
    gate: bool,
    nuke: i32,
    ammo: &[DieAmmo],
) -> u16 {
    let mut v: i32 = 1;
    if gate {
        for a in ammo {
            if a.flags & 3 != 0 && a.idx == i32::from(idx) && a.who == i32::from(who) {
                let t = nuke
                    .wrapping_add(1)
                    .wrapping_add(a.total.wrapping_sub(a.cur));
                v = v.max(t);
            }
        }
    }
    v = v.max(i32::from(hold));
    v as u16
}
