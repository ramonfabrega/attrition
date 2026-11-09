//! The actions: every `Group::action_*` (`CommandPackage::process_*`), from swarm and build to move, siege, guard, patrol, flight and attack (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // The actions
    // ------------------------------------------------------------------

    /// `Group::action_swarm_around@0070fbe0` at **`QUEUE_NEW`** or
    /// **`QUEUE_LAST`** — the position a player's drop issues a build at
    /// (`Group::action_build@00707510`, `docs/GOLDEN.md` §26) and the one
    /// `finish_insert` re-issues a build or a repair at (§24). The two
    /// are one arm of the original: the member's approach is
    /// `add_move_facing_order` at the swarm's own position (`QVar16 =
    /// param_3`), so `QUEUE_NEW` clears each member's list first, and the
    /// order goes in behind it at `QUEUE_LAST` either way.
    ///
    /// The members are walked twice, land (`domain` 0) first and sea
    /// second; an air member is never taken. A citizen (`0x32`/`0x33`)
    /// gets the approach and the order ([`Sim::swarm_around_last`]).
    /// The approach is an `EXPLORE_TO` for a building under a computer
    /// (`local_40 = ~(leader_flags >> 1) & 2 | 1`) and a `MOVE_TO`
    /// otherwise.
    ///
    /// A member that can cast `0x293` (a Militia) is a builder too, and
    /// takes the Civilian between its approach and its order
    /// ([`Sim::swarm_around_last`], item 1167, `docs/GOLDEN.md` §49).
    ///
    /// SEAM, none reached by a capture on file: the non-builders, which
    /// the original gathers into a scratch group and sends `MOVE_TO` the
    /// site at `QUEUE_NEW`; a builder already inside a building
    /// (`count_inside`), which goes with them;
    /// the gather filter (`local_30`, the group's idle citizens, against a
    /// member whose action is a gather); and `is_busy`, a member mid-cast
    /// or boarding. `finish_insert`'s one reach on file is a goody box's
    /// one-member group whose member was just halted. **`QUEUE_FIRST`**
    /// is the original's re-entry (`set_up_insert`, `action_halt`, this
    /// arm at `QUEUE_NEW`, `finish_insert`), which a group command on file
    /// never passes; it is taken here member by member through
    /// [`Sim::swarm_around`], the shape `do_build`'s re-entry has.
    pub(crate) fn group_action_swarm_around(
        &mut self,
        g: &Group,
        b: usize,
        pos: QueuePos,
        body: Body,
        action: bool,
    ) {
        let Some(who) = self.buildings.get(b).map(|bd| i32::from(bd.owner)) else {
            return;
        };
        let order = if matches!(body, Body::Repair(_)) {
            crate::orders::index::REPAIR as i32
        } else {
            crate::orders::index::BUILD_AT as i32
        };
        let queue = match pos {
            QueuePos::First => 0,
            QueuePos::Last => 1,
            QueuePos::New => 2,
        };
        crate::swarm_around_view::action_swarm_around_steps(
            &mut crate::swarm_around_view::SimSwarm::new(self, g),
            b as i32,
            who,
            queue,
            order,
            i32::from(action),
        );
    }

    /// `Group::action_build@00707510` (`docs/GOLDEN.md` §26): a player's
    /// drop. The group must be on the map; the type must be available
    /// (`LeaderData::type_avail(t, 1) == 4`); then the site, the city
    /// limit and the price are [`Sim::place_building`]'s, in the
    /// original's order of refusal — `validate_build`'s snap,
    /// `blocked_site` and `can_afford`, the city limit, and the loop's
    /// snap, `blocked_site`, price and `Objects::init_build`. The loop
    /// runs once: `action_build` pushes one `PathData`, the first point,
    /// and nothing else pushes. A placed site is swarmed at the command's
    /// position with `BUILD_AT` and the action bit
    /// ([`Self::group_action_swarm_around`]). Returns the site.
    ///
    /// SEAM: the scenario's free-building semaphore, the feedback and
    /// the sounds, and `ignore_orders`; none is reached by a capture on
    /// file.
    pub fn group_action_build(
        &mut self,
        g: &Group,
        at: Pos,
        ty: usize,
        pos: QueuePos,
    ) -> Option<usize> {
        if !self.group_is_on_map(g) || !g.list.iter().any(|&u| self.units[u].alive()) {
            return None;
        }
        let tree = self.tables.build_types[ty].tree?;
        if self.type_avail(g.who, tree) != crate::tech::AVAILABLE {
            return None;
        }
        let b = self.place_building(g.who, ty, at).ok()?;
        self.group_action_swarm_around(g, b, pos, Body::Build(b), true);
        Some(b)
    }

    /// `Group::set_up_insert@0070e520` (§6.2, §17): the **leader's**
    /// action-flagged orders, copied aside ahead of a halt, and the target
    /// an attack copy needs — this crate keeps it on the unit rather than on
    /// the order (`docs/GROUPS.md` §12), and the halt clears it.
    pub(crate) fn group_set_up_insert(&self, g: &Group) -> Insert {
        let leader = self.group_find_leader(g);
        let saved: Vec<Order> = leader
            .map(|l| {
                self.units[l]
                    .orders
                    .iter()
                    .filter(|o| o.has(flag::ACTION))
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        // The target goes with the copies: this crate keeps it on the
        // unit rather than on the attack order (`docs/GROUPS.md` §12),
        // and the halt below clears it.
        let aim = leader.and_then(|l| {
            let unit = &self.units[l];
            unit.combat.target.map(|t| (t, unit.combat.mandatory))
        });
        Insert { saved, aim }
    }

    /// `Group::finish_insert@0070e620` (§6.2, §17): each copy
    /// [`Self::group_set_up_insert`] saved, re-issued as a **group**
    /// action at `QUEUE_LAST`. The dispatch is
    /// [`crate::finish_insert_view::finish_insert`], swept against the
    /// original (item 3372); this crate's view over the copies is
    /// [`SimInsert`].
    pub(crate) fn group_finish_insert(&mut self, g: &Group, insert: Insert) {
        let Insert { saved, aim } = insert;
        let mut v = SimInsert {
            sim: self,
            g,
            saved: saved.into(),
            cur: None,
            aim,
        };
        crate::finish_insert_view::finish_insert(&mut v);
    }

    /// `Group::action_halt(mask)` (§7): the steps of [`crate::halt_view`]
    /// over this simulation.
    pub fn group_action_halt(&mut self, g: &Group, mask: i32) {
        let mut view = crate::halt_view::SimHalt { sim: self, g };
        crate::halt_view::action_halt_steps(&mut view, mask);
    }

    /// `Group::action_stance(s)` (§8). A negative `s` cycles: −1 forward,
    /// −2 back.
    pub fn group_action_stance(&mut self, g: &Group, s: i32) {
        if !self.group_is_on_map(g) {
            return;
        }
        let ty = self.group_stance_type(g);
        let n = ty.options();
        if n == 0 {
            return;
        }
        let cur = self.group_stance_option(g, ty);
        let s = if s >= 0 {
            s
        } else if s == -2 {
            if cur - 1 < 0 { n - 1 } else { cur - 1 }
        } else {
            (cur + 1) % n
        };
        for &u in &g.list {
            // `0070d440:95`: vslot `+0xc0` is `is_plane`, so a plane never
            // receives a stance. (The building arm's gate is `is_build` —
            // 1 for a Build and 0 for a Wall, so a wall never receives one
            // either; the simulation has no buildings group to reach it.)
            if !self.units[u].alive() || self.is_plane(u) || self.unit_stance_type(u) != ty {
                continue;
            }
            // `0070d440:117`: `+0xb1 = option`, one byte for **every**
            // panel, the combat stance included. This crate keeps the
            // combat half again as a [`Stance`] below, and wrote only that
            // half until item 530 — so a hoplite the army set to
            // `action_stance(1)` read `stance 0` against the dump's 1.
            self.units[u].stance = s.clamp(0, 255) as u8;
            // `0070d440:100`: `SubObjectData::flags |= 0x10` on every
            // member set, whatever the panel — the bit `Unit::think`'s
            // idle gate reads, so the member thinks on its next frame
            // (item 3962, `docs/AI.md` §406).
            self.units[u].cant_reach = true;
            if ty != StanceType::Combat {
                // The worker/caster/packer stances live elsewhere in the
                // simulation (`Unit::stance`); only the write is modelled.
                continue;
            }
            let Some(&st) = COMBAT_STANCES.get(s as usize) else {
                // Unreachable for a combat stance: it has six options.
                self.clear_orders(u);
                continue;
            };
            self.units[u].combat.stance = st;
            match s {
                0 | 3 | 4 => self.units[u].combat.mandatory = false,
                // 1, 2, 5 kill the current order — for a **human** leader
                // only (`leaders.list[who] & 4`, `docs/GROUPS.md` §8),
                // when it lacks the action bit and the action order does
                // too: `Unit::repath`, then `kill_current_order(0)`. No
                // army's leader is human; a player's stance button is
                // (item 2165, `docs/GOLDEN.md` §79): run835's `0/7` drops
                // the attack it found on the fishing boat on 681.
                1 | 2 | 5 => {
                    let human = self
                        .cities_state
                        .nation
                        .get(g.who as usize)
                        .is_some_and(|n| n.human);
                    if human
                        && self
                            .current_order(u)
                            .is_some_and(|o| !o.has(crate::orders::flag::ACTION))
                        && self.update_action(u).is_none_or(|a| {
                            !self.units[u].orders[a].has(crate::orders::flag::ACTION)
                        })
                    {
                        self.repath(u);
                        self.kill_current_order(u);
                    }
                }
                _ => {}
            }
        }
    }

    /// `Group::action_move_to` → `action_move_near` (§6). `set_angle` and
    /// `disembark` are the caller's; `form`/`width` are −1 on every army
    /// call and the formation is a seam, so every member is given the
    /// group's own destination.
    #[allow(clippy::too_many_arguments)]
    pub fn group_action_move_to(
        &mut self,
        g: &Group,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    ) {
        if !self.group_is_on_map(g) || g.list.is_empty() {
            return;
        }

        // §17 — **a group's `QUEUE_FIRST` is not a member's.** The arm at
        // `00704bfe` does not hand `QUEUE_FIRST` down to
        // `add_move_facing_order` at all: it copies the **leader's**
        // action-flagged orders aside (`set_up_insert`), `action_halt`s
        // every member, calls itself with `QUEUE_NEW`, and re-issues the
        // copies as group actions at `QUEUE_LAST` (`finish_insert`).
        //
        // So the walk a member was on is *dropped*, not stacked behind —
        // which is what run39's frame 825 shows: the scout's plain
        // `EXPLORE_TO` carries no action bit, so nothing is saved, and its
        // order list comes out of `Unit::get_goody_box` holding the new
        // explore alone (`docs/GOODY.md` §7.3).
        //
        // **And the re-issue reads the group as the move left it** (item
        // 3517): the original's `this` is the army's own record, and the
        // move's `Form::categorize` has just `Group::sort`ed it — a squad
        // listed by its follower killed and re-added whole at the end — so
        // `finish_insert`'s `action_guard` lays the escort out over the
        // sorted list. Great Sahara's Americans, frame 4478: army 1/0 listed
        // `1/21` and `1/24` (the tails of the squads `1/19` and `1/22`)
        // second and third, and the escort on `1/99` took its slots in the
        // stale order — every member's `GUARDORDER` `dx`/`dy` parted.
        if queue == QueuePos::First {
            let stack = Self::seat_of_group(g).is_some_and(|seat| *self.seat_list(seat) != g.list);
            let insert = self.group_set_up_insert(g);
            self.group_action_halt(g, 0);
            self.group_action_move_to(g, to, QueuePos::New, set_angle, angle, kind, action);
            let now = match Self::seat_of_group(g) {
                Some(seat) if !stack => Group {
                    list: self.seat_list(seat).clone(),
                    ..g.clone()
                },
                _ => g.clone(),
            };
            self.group_finish_insert(&now, insert);
            return;
        }

        let to = self.restrict_pos(to);
        // A value that names a seat but holds another list is a group on
        // the stack carrying the seat's record — `action_siege_attack_to`'s
        // sub-group (§26, §27). Read before the clear, which can move the
        // seat's list.
        let stack = Self::seat_of_group(g).is_some_and(|seat| *self.seat_list(seat) != g.list);

        // §6.3: the formation angle. With `set_angle` the caller's stands;
        // without it, the direction from the group's own location to the
        // destination.
        //
        // **And which location depends on the queue position**
        // (`00704990:361`–`364`): a `QUEUE_LAST` move asks
        // [`Self::group_loc_to`] — where the leader will *end up* once the
        // orders already on its list are done — and every other position
        // asks [`Self::group_loc`], where it stands now. The two come apart
        // exactly when something is queued ahead, which is the case §12's
        // probe makes on every game: it queues an attack on a farm at
        // `QUEUE_NEW` and the walk home behind it at `QUEUE_LAST`, to the
        // leader's **own** position — so read from where the leader stands
        // the delta is zero and the formation has no bearing at all, and
        // read from the farm it is the length of the map (§17).
        let from = if queue == QueuePos::Last {
            self.group_loc_to(g)
        } else {
            self.group_loc(g)
        };
        let angle = if set_angle {
            angle
        } else {
            match from {
                Some(p) if p != to => find_angle(to.x - p.x, to.y - p.y),
                // The zero-delta arm, and it has two halves
                // (`707e3d`–`707e6d`): with the location different from the
                // group's own `(ox, oy)` — which is every group that has not
                // already been ordered to this very point — the angle is the
                // **leader's own heading** less its packed slot byte, the
                // same quantity [`Self::group_leader_faces_away`] measures.
                // Only a group standing where it was last sent falls through
                // to the record's `o_angle`.
                Some(p) if p != self.group_o(g) => self
                    .group_leader_bearing(g)
                    .unwrap_or_else(|| self.group_o_angle(g)),
                _ => self.group_o_angle(g),
            }
        };

        // §6.3: the formation index. Every simulation caller passes −1,
        // which takes the group's own (`get_form`), clamped up to 0 — so
        // an army's first move settles on form 0 and writes it to every
        // member, which is what `get_form` reads back next time.
        let form = self.group_get_form(g).max(0);
        // …and the same for the width: `param_10 == −1` takes
        // `get_form_mod_option`, which is 50 for an army and stays 50.
        let width = self.group_form_mod_option(g);
        // `action_move_near` writes `form` to the record here, ahead of
        // `compute_form` (`00704990:354` against `:460`), so a sort that
        // kills the list whole and `Group::clear`s the record leaves it −1
        // (run1555's slot 64 on 765, item 4000); `ox`/`oy` and `o_angle`
        // come after it ([`Self::record_move`]).
        if let Some(st) = self.gstate_mut(g) {
            st.form = form;
        }

        // §6.5: the AI branch. `hurry` is the army's, so a group with no
        // army never takes it — and "hurrying" and "found a city" are two
        // different facts, because the clear loop and the order loop do not
        // gate on the same one.
        let ai = self.ai_army_group(g);
        let hurry = ai && self.armies[g.who as usize].list[g.army.unwrap_or(0)].hurry != 0;
        let hurry_city = if hurry {
            self.city_near(g.who, to)
        } else {
            None
        };

        // What the move does to each member, decided **before** the clear.
        // The original decides the same thing twice — once in the clear
        // loop at `70524f` and once in the order loop at `7054c7` — and the
        // second reading sees state the first has already wiped, which is
        // why a shooting siege unit that was cleared falls through and
        // takes the move like everyone else.
        let decide = |this: &Sim, u: usize| -> Member {
            if !this.group_member_orderable(u) {
                return Member::Skip;
            }
            if !ai {
                return Member::Move;
            }
            let shooting_siege = this.is_siege_unit(u) && this.order_type(u) == index::ATTACK;
            match hurry_city {
                Some(c)
                    if this.is_siege_unit(u) || this.is_supply_unit(u) || this.is_hero_unit(u) =>
                {
                    Member::Stable(c)
                }
                // The `QUEUE_NEW` clear at `70524f` gates on `hurry`
                // **alone**; the order loop at `7054c7` on `hurry && a
                // city was found`. So a siege unit that is already
                // shooting is left entirely alone by a move that is not
                // hurrying. A hurrying army that found no friendly city
                // clears its orders — and the order loop then re-reads
                // `order_type()`, which `Unit::close_orders` has left at
                // `NONE`, so the unit falls through and takes the move
                // like every other member. Only `QUEUE_NEW` clears; any
                // other queue position leaves it shooting and skipped.
                _ if shooting_siege && !(hurry && queue == QueuePos::New) => Member::Skip,
                _ => Member::Move,
            }
        };
        let plan: Vec<Member> = g.list.iter().map(|&u| decide(self, u)).collect();

        // §6.6's `QUEUE_NEW` clear, and **it runs before the layout**: the
        // loop at `70524f` clears every member's orders, and its
        // `Unit::clear_orders` at `70538d` is ahead of `compute_form` at
        // `7053ec` in the listing. That ordering is load-bearing rather
        // than incidental — the leader's dying move hands its own mirror
        // back to the group on the way out (§6.3), so the flag
        // `compute_form` reads a line later is the *last layout's* answer
        // and not whatever the march left behind.
        //
        // **A packer mid-pack keeps its pack** (`70524f`–`7053a0`, item
        // 3336): a member whose type packs (`+0x2b8 & 4`) and whose head is
        // a pack cast (`is_packing`) has that order lifted off the list,
        // the rest cleared, and the cast put back
        // ([`Self::clear_keeping_pack`]).
        if queue == QueuePos::New {
            for (&u, p) in g.list.iter().zip(&plan) {
                if *p == Member::Move && !self.clear_keeping_pack(u) {
                    self.clear_orders(u);
                }
            }
        }

        // §6.4: the slot table. `facing` is the group's own mirror flag, and
        // `compute_form` **toggles it around the call** when the leader is
        // pointing more than 90° from the formation's bearing, toggling it
        // back after — so the mirror the layout uses is not the `facing` the
        // record keeps. run31 shows the two coming apart: two of its moves
        // carry `facing 1` and lay out unmirrored, the third carries
        // `facing 1` and mirrors (§6.3, §12.3).
        //
        // **The leader is found before the sort, the flag read after it**
        // (`compute_form@00707c80:37` finds the leader and its slot;
        // `Form::categorize`'s `Group::sort` runs at `:72`; the mirror test
        // reads `GroupData::facing` at `:103`). A sort that kills a
        // follower-first squad whole empties the list, and `Group::kill`'s
        // `Group::clear(-1)` zeroes the flag before it is read (item 4000,
        // `docs/GOLDEN.md` §190: chapter 128's `1/6..1/8` on 764).
        let away = self.group_leader_faces_away(g, angle);
        // `Form::compute` → `Form::categorize`, whose first statement is
        // `Group::sort@00708090` (§4.1, §23) — after the clear, before the
        // mirror test, the layout and the order loop, which
        // therefore walk the **sorted** list. A member the sort brought in
        // was not in the list the clear walked, so it is decided now, as
        // the order loop's own second reading would.
        //
        // **A stack group sorts its own list** (§27): the siege arm's
        // sub-group carries the parent's `army` so that it reads and writes
        // a record, but `Group::sort` runs on the stack copy and the army's
        // list is never touched. Sorting the seat here handed a one-anchor
        // sub-group the whole army whenever the army's list held a squad
        // follower ahead of its captain — East Indies 17402, army 1's
        // `[60, 69]`.
        let resorted = match Self::seat_of_group(g) {
            Some(_) if stack => {
                let mut ng = g.clone();
                self.stack_sort(&mut ng);
                (ng.list != g.list).then_some(ng)
            }
            Some(seat) => {
                let before = self.seat_list(seat).clone();
                self.seat_sort(seat);
                (*self.seat_list(seat) != before).then(|| self.seat_group(seat))
            }
            None => None,
        };
        let (g, plan) = match &resorted {
            Some(ng) => {
                let p: Vec<Member> = ng
                    .list
                    .iter()
                    .map(|&u| match g.list.iter().position(|&m| m == u) {
                        Some(i) => plan[i],
                        None => decide(self, u),
                    })
                    .collect();
                (ng, p)
            }
            None => (g, plan),
        };
        let facing = self.group_facing(g);
        let reverse = facing != away;
        let mut slots = self.form_compute(
            g,
            to,
            angle,
            form,
            width,
            reverse,
            false,
            &self.group_angles(g),
        );
        // §6.3's **second** test, and it is not a mirror: with the angle
        // supplied — every army call — `compute_form` measures its own
        // `find_angle(dest − group_loc)` against it, and when the two are
        // `90°` or more apart the tail negates every member's offsets on
        // both the `Form` and the group, leaving the destinations alone.
        // A group laid out this way walks to the slots it was given and
        // carries the opposite offsets, so `update_positions` puts every
        // follower on the far side of its leader.
        //
        // SEAM: the scenario-editor bit (`semaphore[1] & 8`) forces this
        // false with `facing`; nothing in this simulation sets it.
        if set_angle
            && let Some(p) = from
            && reversing(Angle(
                find_angle(to.x - p.x, to.y - p.y).0.wrapping_sub(angle.0),
            ))
        {
            slots.flipped = true;
            for off in &mut slots.off {
                *off = (-off.0, -off.1);
            }
        }

        for (i, &u) in g.list.iter().enumerate() {
            match plan[i] {
                Member::Skip => continue,
                Member::Stable(c) => {
                    self.stable_in_city(u, c, angle);
                    continue;
                }
                Member::Move => {}
            }
            // §6.6 step 1: the formation index and its width twin, on
            // every member but the four citizen/scholar ids.
            // The exemption guards `form` (`+0xaa`) alone: the width's
            // store at `00705749` (`+0xab`) is outside the test, so a
            // citizen carries the group's `form_mod` like every other
            // member (run157's `1/1` on 990, §24).
            if !self.units[u].ty.is_some_and(|t| {
                self.tables.unit_types[t]
                    .cols
                    .is(crate::ai_load::role::ROLE_CITIZEN)
            }) {
                self.units[u].form = form as i8;
            }
            self.units[u].form_width = width as i8;
            // §6.6 step 2, and it only means anything now that §6.7
            // plans: a `QUEUE_LAST` move turns the member's existing stack
            // over before the group's new legs are pushed on top, so that
            // the single invert at the end of §6.7 leaves the old segment
            // the right way up underneath the new one.
            if queue == QueuePos::Last {
                self.units[u].path.reverse();
            }
            // §6.6 step 3: the member's **own** slot, clamped into the
            // world — not the group's destination.
            //
            // **The clamp is written back into the table**
            // (`action_move_near@00704990:521`–`536`: the four stores are
            // to `param_9 + 0x514`/`+0x714` themselves), so §6.7's plan
            // — the leader's base, a member's translation and
            // `group_plan_each`'s own goal — reads the clamped slot. Greek
            // East Indies' Heavy Fire Raft `1/70` on 5626: its slot sat
            // at x 46259 on a map 46080 wide, and the raw one went on its
            // stack, so `find_path` pulled back off the map's edge onto
            // its own square where theirs, from (46079, 38109), stopped
            // at (45683, 37689) (item 3711, `docs/AI.md` §363.1).
            let mut slot = self.restrict_pos(slots.to[i]);
            slots.to[i] = slot;
            // §6.6 step 4's **different-region** arm (`705a2a`..`705c9b`):
            // a member past the first whose slot's `get_tregion` — the
            // coastal-water `region2` arm, [`crate::world::World::tregion_alt`]
            // — is not slot 0's is re-placed by `find_nearby_spot` around
            // **slot 0's** point (`min 0, max −1`, step 0, bias
            // `0x55555555`, `FILTER_NOT_ME`), and failing that takes slot
            // 0's point itself. French East Indies' Hoplite `1/81` on
            // run630's 10766: its slot's tile is coastal water (region2 0)
            // and slot 0's is region 12, and its order aims at the group's
            // own point there (item 1458, `docs/GROUPS.md` §37).
            //
            // **The sweep asks a barge where slot 0 will not hold the
            // member** (`705c0f`–`705c9b`, item 3522): when `invalid_loc(1,
            // 1, 0, 0, 0)` refuses slot 0's tile to a land member and its
            // leader `can_transport`, the type swept is the leader's
            // `current_upgrade(TRANSPORTBARGE)`, so the ring's first
            // candidate — slot 0's own ocean point — stands. Russian East
            // Indies' army 1 on 4090: slot 0 is the Spy's, in the sea off
            // (8010, 2873), and a T34's own ring reached the shore at
            // (8664, 1368) where the original's whole army aims at
            // (8568, 984) (run1168's block 4091, `docs/AI.md` §304).
            // SEAM: a loaded boat's passenger type (a sea member with
            // `num_inside`); and the same-region `invalid_loc` arm, which a
            // computer's unit (`unit_masks & 0x40000`) skips.
            if i > 0 {
                let s0 = self.restrict_pos(slots.to[0]);
                if self.world.tregion_alt(slot.tile()) != self.world.tregion_alt(s0.tile()) {
                    // The type's sweep with `not_o`/`not_who` −1: nothing
                    // is exempt, the member itself included.
                    slot = self
                        .region_spot_type(u, s0)
                        .and_then(|t| {
                            self.find_nearby_spot_type(t, s0, 0, -1, 0, Angle(0x5555_5555))
                        })
                        .unwrap_or(s0);
                    // Written back into the table (`param_9 + 0x514`,
                    // `+0x714`), which §6.7's translation reads next.
                    slots.to[i] = slot;
                    // **The member names this group afterwards**: the arm
                    // writes `+0x80 = −1` before the search and
                    // `+0x80 = this->id` after it (`705a2a..705c9b`,
                    // decompile 653-655 and 716-718; twenty-fourth pass,
                    // group 15; A9 row 34), so a stale back-pointer is
                    // re-pointed. A group with no seat has no id to write.
                    self.repoint_member(g, u);
                }
            }
            // §6.6 step 6: the order's angle is the formation's, **plus**
            // this slot's packed byte — an addition of a signed byte
            // shifted into the top of the word (`705f42`–`705f4d`), where
            // `compute_form` subtracts the same product back off the
            // leader's heading. And its `facing` is the mirror this layout
            // used, which is what the order hands back when it dies.
            let order_angle = Angle(angle.0.wrapping_add(i32::from(slots.angles[i]) << 24));
            // **The order loop's own keep** (`705e5d`–`705ed8`, item 3336):
            // under `QUEUE_NEW` a packer still packing keeps its cast at
            // the head once more, and its move is added `QUEUE_LAST`,
            // beneath it — Great Sahara's Cannon `1/125` on tick 20218
            // (run967g's block 20219: the `GROUPATTACKTOORDER` under the
            // `CASTORDER` `spell 651`, `spell_time` running on), so the
            // escort its army posts round it that tick reads it standing.
            let queue = if queue == QueuePos::New && self.clear_keeping_pack(u) {
                QueuePos::Last
            } else {
                queue
            };
            // §6.6 step 6's own gate (`705f00`–`705f61`): a **land
            // formation of two or more** walking a `MOVE_TO` or an
            // `ATTACK_TO` gets a `GroupMoveOrder`, and everything else a
            // plain move. The exemptions are modern infantry, a
            // `role & 0x10` type that is not AI-driven (a scout on land, a
            // bark at sea), a group of fewer than two, `unit_masks & 4`
            // ([`sim::Unit::in_danger`], carried since item 465 because
            // it is what the army line below was standing in for), a sea
            // type, and form 9.
            //
            // A group with **no seat** is exempt too, and that is this
            // crate's own line rather than the original's: a `Group` here
            // is a value a caller builds, where the original's is always
            // a pool slot, so the stand-in for `this->group == -1` —
            // which `do_group_move` ungroups on its first line — is "the
            // record has somewhere to live". A player's selection is
            // still N independent moves (`docs/ORDERS.md` §8.4); an
            // army's group and a **pushed** one are not.
            //
            // ~~`g.army.is_some()`~~ until item 465, which cost Great
            // Lakes' probe its `GroupMoveOrder` for four months: the six
            // raiders it pushes out of the army on 8186 are `group 65` in
            // the dump and held no group at all here (`docs/ORDERS.md`
            // §16).
            let grouped = self.gstate(g).is_some()
                && matches!(kind, MoveKind::MoveTo | MoveKind::AttackTo)
                && !self.is_modern_infantry(u)
                && !(self.units[u].ty.is_some_and(|t| {
                    self.tables.unit_types[t]
                        .cols
                        .is(crate::ai_load::role::ROLE_SCOUT)
                }) && !self.ai_driven(g.who))
                && g.num() >= 2
                && !self.units[u].in_danger
                && self.group_domain(u) != Domain::Sea
                && form != 9;
            if let (true, Some(leader)) = (grouped, self.group_find_leader(g)) {
                let gm = crate::orders::GroupMove {
                    leader,
                    id: group_move_id(self.group_id(g), self.frame, self.group_order_num(g)),
                    form_id: i,
                    group_angle: order_angle,
                    in_group: false,
                    orig: to,
                };
                self.add_move_facing_order_grouped(
                    u,
                    slot,
                    kind,
                    queue,
                    action,
                    order_angle,
                    Some(reverse),
                    true,
                    Some(gm),
                    Some(to),
                );
            } else {
                // `705f61`–`705f9b`: the plain arm hands
                // `add_move_facing_order` the click as its `orig`
                // (`param_1`, `param_2`), which `finish_insert` replays
                // to (`docs/GOLDEN.md` §24).
                self.add_move_facing_order_grouped(
                    u,
                    slot,
                    kind,
                    queue,
                    action,
                    order_angle,
                    Some(reverse),
                    true,
                    None,
                    Some(to),
                );
            }
            // **Either arm takes the builder's mark off** (`705fe3`: `andl
            // $0xfffffbff, 0x68(%eax)`, after the member's order): a citizen
            // an army or a raid moves off its site is no longer "has been a
            // builder", and its next idle think does not search for a
            // build. run1370's `1/95` leaves its `BUILDORDER` for group 80's
            // move between 4001 and 4101 and its `unit_masks` loses 0x400;
            // ours kept it to 4841 and sent it to the unbuilt `1/2052`
            // (`docs/AI.md` §358.4, item 3710).
            self.units[u].was_builder = false;
        }
        // §6.7: the group's own path, planned once off the leader's slot
        // and handed to every member translated. It sits exactly here in
        // the original — after the order loop, before `order_num` — and it
        // is why the orders above are born `PATHED`.
        //
        // `from` is `get_loc`'s answer and is `None` only for a group with
        // no leader at all, which is the same condition that sends the plan
        // down its own no-leader arm; the destination stands in so that the
        // arm has a start to fall back on.
        self.group_plan_path(g, &slots, &plan, kind, form, from.unwrap_or(to));
        self.bump_order_num(g);
        // The order matters and run31 settles it: `Form::compute`'s tail
        // writes `o_angle` and `o_dist` from the leader's slot, and then
        // `action_move_near` writes `o_angle` again — with the **formation
        // angle** — for a `QUEUE_NEW`/`QUEUE_LAST` move (`00704990:464`).
        // So the record's `o_angle` is the formation's bearing and its
        // `o_dist` is the leader's offset within the block, which is how
        // run31 measures §6.4's displacement.
        self.record_form(g, &slots, to);
        self.record_move(g, to, angle, queue);
    }

    /// The type `action_move_near`'s different-region arm sweeps for member
    /// `u` round slot 0's point `s0` (`705c0f`–`705c9b`): the member's own,
    /// unless slot 0's tile is `invalid_loc(1, 1, 0, 0, 0)` for it and its
    /// type is of the land domain (`+0x218 == 0`) under a leader that
    /// `can_transport` — then the leader's `current_upgrade(TRANSPORTBARGE)`.
    pub(crate) fn region_spot_type(&self, u: usize, s0: Pos) -> Option<usize> {
        let own = self.units[u].ty?;
        let who = self.units[u].owner;
        if self.tables.unit_types[own].combat.domain == Domain::Land
            && self.invalid_loc(u, s0.tile(), true, true, false, false, false) != 0
            && self.transport_level(who) != crate::transport::TransportType::None
        {
            return self.barge_type(who).or(Some(own));
        }
        Some(own)
    }

    /// `Group::action_move_near@00704990`'s keep (`:410`–`440`, `:727`–
    /// `752`, item 3336): a member whose type packs and whose head order is
    /// a pack cast (`UnitData::is_packing`) has that order taken off the
    /// list (`LinkListBase::remove_current` — not killed), the rest
    /// cleared (`Unit::clear_orders`), and the cast added back, then
    /// `update_gpiece`. False, and nothing done, for any other member.
    pub(crate) fn clear_keeping_pack(&mut self, u: usize) -> bool {
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].combat.packs);
        if !packs || !self.is_packing(u) {
            return false;
        }
        let Some(cast) = self.units[u].orders.pop_front() else {
            return false;
        };
        self.clear_orders(u);
        self.units[u].orders.push_front(cast);
        self.update_gpiece(u);
        true
    }

    /// `Group::action_siege_attack_to(x, y, ·, ·, angle)` (§9).
    pub fn group_action_siege_attack_to(&mut self, g: &Group, to: Pos, angle: Angle) {
        if g.list.is_empty() {
            return;
        }
        // The sub-group: every siege member, else the first supply wagon,
        // else the first hero.
        let mut sub = Group {
            who: g.who,
            army: g.army,
            pushed: g.pushed,
            list: Vec::new(),
        };
        for &u in &g.list {
            if self.units[u].alive() && self.units[u].on_map && self.is_siege_unit(u) {
                self.group_add(&mut sub, u);
            }
        }
        // A **human** leader skips the widening and the anchor scoring and
        // takes the sub-group's own leader (`0070dabd`). No army's leader
        // is human, so this arm is the player's siege-attack command.
        if self
            .cities_state
            .nation
            .get(g.who as usize)
            .is_some_and(|n| n.human)
        {
            // The anchor is only read by `action_guard`, which is the seam;
            // whether the sub-group has a leader is what chooses the arm.
            let escort = self.group_find_leader(&sub).is_some();
            let it = if escort { &sub } else { g };
            self.group_action_move_to(it, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        }
        if sub.list.is_empty()
            && let Some(&u) = g
                .list
                .iter()
                .find(|&&u| self.units[u].alive() && self.units[u].on_map && self.is_supply_unit(u))
        {
            self.group_add(&mut sub, u);
        }
        if sub.list.is_empty()
            && let Some(&u) = g
                .list
                .iter()
                .find(|&&u| self.units[u].alive() && self.units[u].on_map && self.is_hero_unit(u))
        {
            self.group_add(&mut sub, u);
        }

        // The anchor: the sub-group member with the smallest total
        // Manhattan distance to the whole group, in 1024-unit steps.
        let anchor = self.siege_anchor(g, &sub);
        let Some(anchor) = anchor else {
            self.group_action_move_to(g, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        };
        // The area test: the anchor and the destination must share a
        // region (the sim's stand-in for §9's area id).
        let same =
            self.world.tregion(self.units[anchor].pos.tile()) == self.world.tregion(to.tile());
        if !same {
            self.group_action_move_to(g, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        }
        // **The sub-group's record is its own** (§26): `Group::clear(-1)` on
        // the stack, then the parent's `id` and `army` and `stamp = 0`
        // (`0070d830:60–66`). Its layout reads that record's `facing`, 0,
        // and writes that record, which the original discards on return.
        // This crate keys a group's record by its seat, so the parent's is
        // set aside while the sub-group moves and put back after it.
        let stack = GroupState {
            stamp: 0,
            pool: self.gstate(&sub).and_then(|st| st.pool),
            ..GroupState::default()
        };
        let parent = self.gstate_mut(&sub).map(|st| std::mem::replace(st, stack));
        // The parent is set aside where a member's own writes can find it:
        // the clear inside the layout kills each member's head order, and a
        // dying move's hand-back goes to the unit's real group (`docs/AI.md`
        // §212) — run820's Supply Wagon `1/205`, the army's leader, wrote its
        // 1 into the stack record on 4082 and the layout read it.
        self.orders.set_aside = parent.map(|p| (sub.who, sub.army, sub.pushed, p));
        self.group_action_move_to(
            &sub,
            to,
            QueuePos::New,
            true,
            angle,
            MoveKind::AttackTo,
            true,
        );
        let parent = self.orders.set_aside.take().map(|(.., p)| p);
        // The sub-group's own list as its layout walked it: `Group::sort`
        // runs on the stack record in place, and the record's tables are
        // indexed by the sorted list.
        let mut walked = sub.clone();
        self.stack_sort(&mut walked);
        let mine = match (parent, self.gstate_mut(&sub)) {
            (Some(p), Some(st)) => Some(std::mem::replace(st, p)),
            _ => None,
        };
        // **The copy-back** (`0070de..0070dfa0`): for each member of the
        // parent that the sub-group holds, the sub-group's slot — `off`,
        // `curr`, `angles` at the member's index in the sub-group — is
        // written into the parent's table **at the member's index in the
        // parent**, and `Unit::replace_form_id@005fd420` re-points every
        // group order the member holds at that index. So a siege unit past
        // the parent's `form_num` carries its slot in the parent's tail,
        // where `refresh_group_order` reads it (item 1493, `docs/AI.md`
        // §128: 1/141's `off` (23, 0) at index 62).
        if let Some(mine) = mine {
            for (k, &m) in g.list.iter().enumerate() {
                if !(self.units[m].alive() && self.units[m].on_map) {
                    continue;
                }
                let Some(j) = walked.list.iter().position(|&x| x == m) else {
                    continue;
                };
                if let Some(st) = self.gstate_mut(&sub) {
                    // The arrays are the member count long once a member
                    // has been added through the seat; a record built
                    // before that is brought up to the list's length.
                    let n = g.list.len();
                    if st.off.len() < n {
                        st.off.resize(n, (0, 0));
                    }
                    if st.curr.len() < n {
                        st.curr.resize(n, Pos::new(0, 0));
                    }
                    if st.angles.len() < n {
                        st.angles.resize(n, 0);
                    }
                    if let Some(&o) = mine.off.get(j) {
                        st.off[k] = o;
                    }
                    if let Some(&o) = mine.curr.get(j) {
                        st.curr[k] = o;
                    }
                    if let Some(&o) = mine.angles.get(j) {
                        st.angles[k] = o;
                    }
                }
                for o in &mut self.units[m].orders {
                    if let Some(gm) = o.move_mut().and_then(|mv| mv.group.as_mut()) {
                        gm.form_id = k;
                    }
                }
            }
        }
        // `action_guard(anchor, who, QUEUE_NEW, 1)` on the parent: the
        // rest escort the anchor while it walks in (`docs/ORDERS.md` §24).
        // Golden chapter four's word, 1277: an AI army with a Supply Wagon
        // and no siege, whose three hoplites take a `GUARDORDER` on the
        // wagon on the army's own tick.
        self.group_action_guard(g, anchor, QueuePos::New, true);
    }

    /// `Group::action_guard(o, who, queue, no_siege)@006fcd30`
    /// (`docs/ORDERS.md` §24), for a **unit** target.
    ///
    /// `action_begin`, and the group's `form` goes to −1. The target is
    /// replaced by its squad's **captain** (`get_captain`, vslot `+0xe4`;
    /// the building test at vslot `+0x18` answers 1 for every unit). A
    /// stack group then gathers the members that can escort it: active, on
    /// the map, not a plane, of the target's domain — or a helicopter, or
    /// a sea transport beside a land target — not siege when `no_siege`,
    /// and not the target itself. On patch versions above 3 a target that
    /// is itself guarding one of the members has its orders cleared, so
    /// two units never guard each other. The escort is laid out by
    /// `compute_form` **at the target**, on bearing 0, width `0x32`, with
    /// the guard flag set — which reserves an artillery rank for the target
    /// and keeps the reverse negation off — and each member is given its
    /// slot's `off_x/off_y` as the guard offset: rewritten on a `GUARD`
    /// it already holds on this target, else a fresh `add_guard_order`.
    ///
    /// SEAM, none reached by a capture on file: `QUEUE_FIRST`'s insert
    /// dance (taken as `QUEUE_NEW`); the building-target arm (every member
    /// `add_guard_order(o, who, −1, −1)`; this returns instead); the human
    /// leader's sweep over the player's other units already guarding the
    /// target; and `Group::sort` inside `Form::compute`, a no-op on a list
    /// [`Self::group_add`] has just built captain-first.
    pub fn group_action_guard(
        &mut self,
        g: &Group,
        target: usize,
        queue: QueuePos,
        no_siege: bool,
    ) {
        if g.list.is_empty() {
            return;
        }
        if !(self.units[target].alive() && self.units[target].on_map) {
            return;
        }
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let whom = self.units[target].owner;
        if !self.is_ally(g.who, whom) || self.is_plane(target) {
            return;
        }
        let cap = self.top_captain(target);
        let dom = self.group_domain(cap);
        let mut local = Group {
            who: g.who,
            army: None,
            pushed: None,
            list: Vec::new(),
        };
        for &m in &g.list {
            if !(self.units[m].alive() && self.units[m].on_map) || self.is_plane(m) {
                continue;
            }
            let flag = |f: u32| {
                self.units[m]
                    .ty
                    .is_some_and(|t| self.tables.unit_types[t].cols.flag(f))
            };
            let admitted = flag(uflags::FLAG_HELICOPTER)
                || self.group_domain(m) == dom
                || (flag(uflags::FLAG_TRANSPORT) && dom == Domain::Land);
            if !admitted || (no_siege && self.is_siege_unit(m)) {
                continue;
            }
            if m == cap && g.who == whom {
                continue;
            }
            self.group_add(&mut local, m);
        }
        // `Game::get_patch_version() > 3`: break a guard cycle.
        if self.order_type(cap) != index::NONE
            && g.list
                .iter()
                .any(|&m| self.update_guard_order(cap, m).is_some())
        {
            self.clear_orders(cap);
        }
        // `get_num_cap`: the captains still standing.
        if !local
            .list
            .iter()
            .any(|&m| self.units[m].alive() && self.units[m].captain)
        {
            return;
        }
        if self.group_find_leader(&local).is_none() {
            return;
        }
        // `compute_form(local, target, leader_flags >> 2 & 1, 0x32, ·, 1,
        // &0, loc, 1)`: formation 0 for an AI, the caller's angle of zero
        // standing, and the guard flag. The stack group's `facing` is the
        // `clear`ed zero, so the mirror is the leader toggle alone; the
        // reverse negation is skipped because the guard flag is set.
        let human = self
            .cities_state
            .nation
            .get(g.who as usize)
            .is_some_and(|n| n.human);
        let form = i32::from(human);
        let tp = self.units[cap].pos;
        let reverse = self.group_leader_faces_away(&local, Angle(0));
        let slots = self.form_compute(&local, tp, Angle(0), form, 0x32, reverse, true, &[]);
        for (k, &m) in local.list.iter().enumerate() {
            if m == cap || self.top_captain(m) == cap {
                continue;
            }
            let (dx, dy) = slots.off.get(k).copied().unwrap_or((0, 0));
            match self.update_guard_order(m, cap) {
                Some(i) => {
                    if let Body::Guard(x) = &mut self.units[m].orders[i].body {
                        x.dx = dx;
                        x.dy = dy;
                    }
                }
                None => self.add_guard_order(m, cap, dx, dy, queue),
            }
        }
    }

    /// `Group::action_follow(ox, whom, queue)@006fd510` (`docs/ORDERS.md`
    /// §28), reached from `CommandPackage::process_follow@009479c0` with
    /// the command's leader and queue position.
    ///
    /// `action_begin`, then a group of buildings returns. The group's
    /// `form` goes to −1. The leader must be active, on the map and not a
    /// plane (vslots `+0x8`, `+0xbc`, `+0xc0`). Every member that is the
    /// same, and is not the leader's own captain of the same player
    /// (`get_captain`, vslot `+0xe4`), gets one
    /// [`Sim::add_follow_order`]. **There is no `is_ally` test**: a player
    /// may follow anyone's unit.
    ///
    /// SEAM, none reached by a capture on file: `QUEUE_FIRST`'s insert
    /// dance (`set_up_insert`, `action_halt`, the follow at `QUEUE_NEW`,
    /// `finish_insert`), taken here as `QUEUE_NEW`; the scenario's
    /// `ignore_orders` sweep; a buildings group, which the command's
    /// `process_group` cannot build from units.
    pub fn group_action_follow(&mut self, g: &Group, target: usize, queue: QueuePos) {
        let queue = if queue == QueuePos::First {
            QueuePos::New
        } else {
            queue
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        if !self.group_member_orderable(target) {
            return;
        }
        let whom = self.units[target].owner;
        let cap = self.top_captain(target);
        for &m in &g.list {
            if !self.group_member_orderable(m) || (m == cap && g.who == whom) {
                continue;
            }
            self.add_follow_order(m, target, queue);
        }
    }

    /// `Group::action_garrison(ox, whom, queue, search)@00700490`
    /// (`docs/ORDERS.md` §29), reached from
    /// `CommandPackage::process_garrison@00948760` with the command's
    /// building and queue position, and `search` 0.
    ///
    /// `action_begin`; the group's `form` goes to −1. The building must be
    /// the group's player's or a mutual ally's, alive, with a garrison
    /// limit, and not an unassimilated city. Then every member that is
    /// active, on the map and not a plane gets one
    /// [`Sim::add_garrison_order`] with the action bit — **if its type
    /// `can_garrison` the building's**; a member that cannot gets nothing.
    /// A member whose action is already a GARRISON is `repath`ed and has
    /// its head killed first.
    ///
    /// The walk is [`crate::garrison_view::action_garrison_steps`], swept
    /// against the original (item 3376); the SEAMs are
    /// [`crate::garrison_view::SimGarrison`]'s, none reached by a capture on
    /// file.
    pub fn group_action_garrison(&mut self, g: &Group, b: usize, queue: QueuePos) {
        self.group_action_garrison_search(g, b, queue, false);
    }

    /// [`Self::group_action_garrison`] with its fifth argument, `search`,
    /// which the City's alarm passes as 1 (item 1167, `docs/GOLDEN.md`
    /// §49): each member is sent to `Unit::find_garrison_build@00605040`'s
    /// building of the city when it finds one, the building `b` otherwise,
    /// and the order carries `search`. At `QUEUE_NEW` a **worker**
    /// (`ObjectData::is_worker@0046fa10`: `0x32`..`0x35`) takes its order
    /// at `QUEUE_FIRST`, in front of what it holds, with the same `search`
    /// (`00700a05`..`00700a13`: `edx` is still `param_4` from `007008c4`
    /// or `00700909`).
    pub fn group_action_garrison_search(
        &mut self,
        g: &Group,
        b: usize,
        queue: QueuePos,
        search: bool,
    ) {
        let queue = match queue {
            QueuePos::First => 0,
            QueuePos::Last => 1,
            QueuePos::New => 2,
        };
        let who = i32::from(self.buildings[b].owner);
        let mut view = crate::garrison_view::SimGarrison { sim: self, g };
        crate::garrison_view::action_garrison_steps(
            &mut view,
            b as i32,
            who,
            queue,
            i32::from(search),
            0,
        );
    }

    /// `ObjectData::is_worker@0046fa10`: the type is `0x32`..`0x35` — the
    /// Citizen, its tribe's, and the two Scholars.
    pub(crate) fn is_worker(&self, u: usize) -> bool {
        (0x32..=0x35).contains(&self.units[u].type_index)
    }

    /// `Group::action_eject_all(back_to_work, who, eject_o, eject_who)
    /// @00710b40` (`docs/ORDERS.md` §29), reached from
    /// `CommandPackage::process_eject_all@00947fe0` on a group of
    /// buildings. With `who < 0`, or `eject_who < 0` and `who` the group's,
    /// every building of the group, last first, that is alive, holds a
    /// squad and is no hangar gets `Object::eject_contents(0, −1, 0,
    /// back_to_work == 0)`, which for a building on the map defers:
    /// [`Sim::eject_contents`], one squad a frame from the head.
    ///
    /// SEAMS: `back_to_work`'s type filter (`0x32`, the citizens) and the
    /// immediate path it takes; the `eject_o`/`eject_who` arm that pulls one
    /// player's units out of another's building. A group of one boat, with
    /// the two types (`0x140`, `0x13e`) that die when emptied, is
    /// [`Sim::boat_eject_all`] (item 3521).
    pub fn action_eject_all(
        &mut self,
        who: Player,
        buildings: &[usize],
        eject_who: i32,
        cmd_who: i32,
    ) {
        if !eject_all_own_arm(i32::from(who), eject_who, cmd_who) {
            return;
        }
        for &b in buildings.iter().rev() {
            let bd = &self.buildings[b];
            if !eject_all_wants(
                bd.alive,
                || !bd.garrison.is_empty(),
                || bd.ty.is_some_and(|t| self.is_hangar(t)),
            ) {
                continue;
            }
            self.eject_contents(b, false);
        }
    }

    /// `Group::action_patrol@007030c0` (`docs/ORDERS.md` §27), reached from
    /// `CommandPackage::process_patrol@00949380` with the command's point
    /// and queue position.
    ///
    /// The point is clamped into the world and a `QUEUE_FIRST` taken as
    /// `QUEUE_NEW`; the group's `form` goes to −1. The patrol's first point
    /// is the group's location ([`Self::group_loc`], the leader's position
    /// unless the group stands on its own order point), and every member
    /// that is on the map, not a plane and not busy gets a
    /// `GroupPatrolOrder` from it to the click, with the group's id, its
    /// own index and the leader. `order_num` steps once after the loop.
    ///
    /// SEAMS, none of which a player's command on land units reaches:
    /// a plane leader or any plane member hands the whole call to
    /// `action_air_patrol`, which returns here; the `QUEUE_LAST` arm reads
    /// [`Self::group_loc_to`] and appends the click to each member's
    /// existing patrol (`update_patrol_order`) instead of issuing one —
    /// here it issues; a member off the map with type flag `0x20` gets a
    /// plain move; the scenario's `ignore_orders` filter; a buildings
    /// group. `UnitData::is_busy@0060a370` is [`Self::is_busy`]: a head pack
    /// or unpack, never another cast — and a lone member's alone, since a
    /// group's leader halts every member in `do_patrol`'s `QUEUE_FIRST`
    /// move the same frame (item 3988, `docs/GOLDEN.md` §181).
    pub fn group_action_patrol(&mut self, g: &Group, to: Pos, queue: QueuePos) {
        let to = self.restrict_pos(to);
        let queue = if queue == QueuePos::First {
            QueuePos::New
        } else {
            queue
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let Some(leader) = self.group_find_leader(g) else {
            return;
        };
        // `count(COUNT_DOMAIN, 4)`: param 4 counts vslot `+0xc0`, `is_plane`,
        // over the alive members — a Helicopter is not counted, so a group
        // with one and no plane takes the land arm (item 3967, `docs/GOLDEN.md`
        // §173).
        if self.is_plane(leader)
            || g.list
                .iter()
                .any(|&m| self.units[m].alive() && self.is_plane(m))
        {
            self.group_action_air_patrol(g, to, queue);
            return;
        }
        let from = if queue == QueuePos::Last {
            self.group_loc_to(g)
        } else {
            self.group_loc(g)
        };
        let Some(from) = from else {
            return;
        };
        let id = group_move_id(self.group_id(g), self.frame, self.group_order_num(g));
        for (i, &m) in g.list.iter().enumerate() {
            if !self.group_member_orderable(m) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy {
                continue;
            }
            self.add_patrol_order(m, (from, to), id, i, leader, queue);
        }
        self.bump_order_num(g);
    }

    /// **`Group::action_air_patrol(x, y, queue)@007029d0`** (item 3911,
    /// `docs/GOLDEN.md` §158), which [`Self::group_action_patrol`] hands a
    /// group with a plane leader or an air member. The point is clamped
    /// into the world, and every member that is a plane (`is_plane`: domain
    /// 2, not a helicopter), not busy and not a missile (`type +0x1e4 &
    /// 0x8000000`) takes `add_air_patrol_order(x, y, home, 1)`: the home is
    /// the front air order's (a `STRAFE`, `AIR_PATROL` or
    /// `AIR_ATTACK_GROUND`, `+4`/`+8`), else `get_inside`, and a plane with
    /// neither takes nothing. The adder's queue argument is the constant 1,
    /// not the command's. run1505's Fighter `0/6`, flying home on its strafe:
    /// one `AIRPATROLORDER` over (7680, 11520), home `0/2007`, on 682.
    ///
    /// SEAMS, none of which run1505 reaches: a group with members that are
    /// not planes is split, the planes recurse and the rest take
    /// `action_move_to(MOVE_TO)`, here the others take nothing; a plane
    /// already on an `AIR_PATROL` under `QUEUE_LAST` appends the point to
    /// its arrays (see [`crate::orders::PATROL_POINTS`]), here it is given
    /// a new patrol; the empty-tank feedback is the interface's alone; the
    /// scenario's `ignore_orders` filter.
    pub fn group_action_air_patrol(&mut self, g: &Group, to: Pos, _queue: QueuePos) {
        let to = self.restrict_pos(to);
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy || self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            let home = match self.current_air(m) {
                Some(a) => a.home,
                None => self.units[m].inside,
            };
            if home.is_some() {
                self.add_air_patrol_order(m, to, home, true);
            }
        }
    }

    /// **`GroupData::is_in_air_range(o, whom)@0070b6b0`** (item 3911,
    /// `docs/GOLDEN.md` §158): `Console::execute_at_cursor@007c6630:3116`'s
    /// test for a right-click on an enemy with a selection that carries
    /// planes. Whether the target object is within `mana × get_speed(·, 1)`
    /// of a plane, the listing's arithmetic (`0070b6b0`–`0070b9d0`):
    ///
    /// - **a building selection** (`+0x49`): each live member that
    ///   `can_carry(AIR)` measures `vector_dist(target − member)` once, and
    ///   any plane in its `inside_down` chain whose `UnitData::mana` times
    ///   its speed at its own point (vslot `+0x17c`, flag 1) reaches it
    ///   answers 1;
    /// - **a unit selection**: each live member that is off the map and a
    ///   fixed-wing plane measures from the host `get_inside` names.
    ///
    /// run1505b's Airbase `0/2007` with the Fighter `0/6` inside, on
    /// `1/2006`: 1, and the click issued the launch on 807.
    ///
    /// SEAM: `can_carry(AIR)` is [`Self::is_hangar`] here (an Airbase or a
    /// Missile Silo); a carrier selected is the unit arm's host, not
    /// reached.
    pub fn is_in_air_range(&self, buildings: &[usize], units: &[usize], target: Obj) -> bool {
        let at = self.pos_of(target);
        let reach = |u: usize| self.unit_mana(u) * self.get_speed(u, 1);
        for &b in buildings {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.ty.is_some_and(|t| self.is_hangar(t)) {
                continue;
            }
            let d = crate::world::vector_dist(at.x - bd.pos.x, at.y - bd.pos.y);
            if bd.garrison.iter().any(|&u| d <= reach(u)) {
                return true;
            }
        }
        for &u in units {
            let un = &self.units[u];
            if !un.alive() || un.on_map || !self.is_plane(u) {
                continue;
            }
            let Some(host) = un.inside else {
                continue;
            };
            let h = self.buildings[host].pos;
            if crate::world::vector_dist(at.x - h.x, at.y - h.y) <= reach(u) {
                return true;
            }
        }
        false
    }

    /// **`Group::action_air_attack_ground(x, y, queue)@00703d80`** (item
    /// 3911, `docs/GOLDEN.md` §158), which `Group::action_attack_ground@
    /// 00704520:119-168` hands a group whose leader is a plane or carries
    /// `AIR`. The point is clamped into the world, and each live member that
    /// is a plane and not busy takes `add_air_attack_ground_order(x, y,
    /// home, queue, 1)`: the home is the front air order's (a `STRAFE`,
    /// `AIR_PATROL` or `AIR_ATTACK_GROUND`), else `get_inside`, and a plane
    /// with neither takes nothing. run1505b's Bomber `0/7`, on its patrol
    /// over the Barracks' point: one `AIRATTACKGROUNDORDER` there, home
    /// `0/2007`, on 682.
    ///
    /// SEAMS, none of which run1505b reaches: the missile arms — a missile
    /// still in its silo's `launching` list is skipped, the first missile
    /// ends the search for others (`local_28`), and a missile's strike asks
    /// `war_allowed` and the target tile's ocean; a group holding type
    /// `0x13b` asks each member its type's slot `+0x60`; the empty-tank
    /// feedback is the interface's; the scenario's `ignore_orders` filter.
    pub fn group_action_air_attack_ground(&mut self, g: &Group, at: Pos, queue: QueuePos) {
        let at = self.restrict_pos(at);
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            if self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy {
                continue;
            }
            let home = match self.current_air(m) {
                Some(a) => a.home,
                None => self.units[m].inside,
            };
            if home.is_some() {
                self.add_air_attack_ground_order(m, at, home, queue, true);
            }
        }
    }

    /// **`Group::action_recall@006fa7e0`**, the `recall` command's (byte
    /// 0x23, `CommandPackage::process_recall@00947bd0`) group action: a
    /// leader whose type is domain 2 hands the call to
    /// [`Self::group_action_return`] (item 3911, `docs/GOLDEN.md` §158).
    ///
    /// The building arm is [`Self::group_action_recall_buildings`] (item
    /// 3912), which the harness reaches on a building selection.
    pub fn group_action_recall(&mut self, g: &Group) {
        let Some(leader) = self.group_find_leader(g) else {
            return;
        };
        if self.group_domain(leader) == Domain::Air {
            self.group_action_return(g);
        }
    }

    /// **`Group::action_return@006fad40`** (item 3911, `docs/GOLDEN.md`
    /// §158): each member that is not a missile and stands on the map takes
    /// its front air order's home: the order's `cruising_alt` and
    /// `sharp_turn` are kept, its orders are closed (`unit_masks &=
    /// ~0x4000000`, the path emptied, `close_orders`, `clear_partial_path`,
    /// `update_action`), and `add_strafe_order(-1, -1, home, 0, QUEUE_NEW,
    /// 0)` lays a strafe home, which takes `returning` 1 for having no
    /// target; the two kept fields are written back into it. run1505's
    /// Bomber `0/8`, on its strafe at `1/2006`: on 854 one `STRAFEORDER`,
    /// target −1, `mandatory 0`, home `0/2007`, `cruising_alt` 1600,
    /// `returning 1`, the patrol behind it gone.
    ///
    /// SEAMS, none of which run1505 reaches: a helicopter (`+0x2b4 & 0x20`)
    /// on the map is closed and given a strafe with no home, flying home;
    /// a plane inside a host is closed and cut from the host's
    /// `launching` list (`+0x44`), which this crate does not carry; a front
    /// `SPECIALANIMORDER` (a landing) gives its own air order; the
    /// scenario's filter of units that ignore orders.
    pub fn group_action_return(&mut self, g: &Group) {
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            if self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            if self.units[m].inside.is_some() {
                continue;
            }
            let Some(air) = self.current_air(m) else {
                continue;
            };
            self.units[m].path.clear();
            self.close_orders(m);
            self.clear_partial_path(m);
            self.update_action(m);
            self.add_strafe_order(m, None, air.home, false, QueuePos::New, false);
            self.set_cruising_alt(m, air.cruising_alt);
            self.set_sharp_turn(m, air.sharp_turn);
        }
    }

    /// `Group::redo_patrol_order@00706d90`, `finish_insert`'s case `0x16`
    /// (`docs/ORDERS.md` §27): after a group `QUEUE_FIRST` has halted every
    /// member, each one that is on the map, not a plane and not a missile
    /// (`obj_masks & 0x8000000`) is given the saved patrol again at
    /// `QUEUE_LAST` — its two points, its id, its leader and **its
    /// `form_id`, the leader's** — and then the saved `waypoint` is
    /// written into the new order (`update_patrol_order(id)`). run184's
    /// squad carries `form_id 0` on all three patrols from its first leg.
    ///
    /// SEAM: the copy of points past the second; see
    /// [`crate::orders::PatrolOrder`].
    pub(crate) fn group_redo_patrol_order(&mut self, g: &Group, p: crate::orders::PatrolOrder) {
        let mut v = crate::redo_patrol_view::SimRedoPatrol::new(self, g, p);
        crate::redo_patrol_view::redo_patrol_order_steps(&mut v);
    }

    /// §9's anchor score.
    pub(super) fn siege_anchor(&self, g: &Group, sub: &Group) -> Option<usize> {
        let mut best: Option<(i32, usize)> = None;
        for &c in &sub.list {
            let p = self.units[c].pos;
            let mut total = 0;
            for &u in &g.list {
                // `0070d830:194` gates every summed member on
                // `is_valid_unit() && is_on_map()`, so a garrisoned or
                // carried member does not drag the anchor towards itself.
                if !self.units[u].alive() || !self.units[u].on_map {
                    continue;
                }
                let q = self.units[u].pos;
                total += ((p.x - q.x).abs() + (p.y - q.y).abs()) >> 10;
            }
            if best.is_none_or(|(bt, _)| total < bt) {
                best = Some((total, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// `Group::action_flight(ox, whom, orders, shift, ctrl, alt)@006fb260`
    /// (`docs/ORDERS.md` §32): the flight command's one call, and the
    /// only maker of a player's `StrafeOrder`. `kind` is the command's
    /// `orders`: [`Flight::Home`] for `MOVE_TO`, a right-click on one's
    /// own base, and [`Flight::Strike`] for `ATTACK`, one on an enemy.
    ///
    /// The target must be live (`+8 & 1`); a flight home must name one of
    /// the group's own Airbases (`can_carry(AIR)`). Then, per member
    /// (the loop's bound is `group.num`):
    /// - **a member already on a `STRAFE`** has that order re-pointed and
    ///   gets no new one. A strike writes the target and its point, then
    ///   `returning 0`, `mandatory 1` and the action bit; a flight home
    ///   writes the base as its home, no target, `returning 1`,
    ///   `mandatory 1` and the action bit;
    /// - **any other member**'s "inside" is its `inside_up`. A strike
    ///   needs it; a flight home gives `add_strafe_order(−1, −1, base,
    ///   who, 1, QUEUE_NEW, 1)` unless the member already stands in that
    ///   base.
    ///
    /// So **an aircraft on the ground outside a base takes no strike at
    /// all**, and no feedback beyond the console's: run223's block 622.
    ///
    /// **A strike on a mixed group splits first**: the planes are flown by
    /// this call on a group of their own, and the rest take
    /// `action_attack(o, whom, mandatory 1, QUEUE_NEW)` (item 3991,
    /// `docs/GOLDEN.md` §184). **A flight home asks `can_carry` of each
    /// member**, which counts the base's homed planes in the air too.
    ///
    /// SEAMs, none reached by run223: the `NUCLEARMISSILE` arm, the missile arm of each gate, the empty-tank
    /// arms (this crate carries no fuel, so a tank is never empty), the
    /// `FIGHTERBOMBER` `home_base` gate, a carrier as the base, the
    /// `AIR_PATROL`/`AIR_ATTACK_GROUND` home as the "inside" (no member
    /// holds either), and the war test (always allowed). A strike from
    /// inside a base is [`Sim::strike_from_inside`]: a missile's three
    /// gates, a plane's none (item 3968).
    pub fn group_action_flight(&mut self, g: &Group, target: Obj, kind: Flight) {
        let live = match target {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive()),
            Obj::Building(b) => self.buildings.get(b).is_some_and(|b| b.alive),
        };
        if !live {
            return;
        }
        let base = match (kind, target) {
            (Flight::Home, Obj::Building(b))
                if self.buildings[b].owner == g.who
                    && self.buildings[b].ty.is_some_and(|t| {
                        crate::build::is(&self.tables.build_types, t, crate::build::Ident::Airbase)
                    }) =>
            {
                Some(b)
            }
            (Flight::Home, _) => return,
            (Flight::Strike, _) => None,
        };
        // **The split** (`6fb3f9`..`6fb53f`, item 3991, `docs/GOLDEN.md`
        // §184): a strike whose live members are not all planes
        // (`UnitData::is_plane`, vslot `+0xc0`: a helicopter is no plane)
        // hands the planes to this call again on a group of their own, and
        // the rest to `action_attack(o, whom, 1, QUEUE_NEW, 0)` — or
        // `action_guard` on an ally's target — and returns.
        if kind == Flight::Strike {
            let mut air = Group::stack(g.who);
            let mut rest = Group::stack(g.who);
            for &u in &g.list {
                if !self.units[u].alive() {
                    continue;
                }
                if self.is_plane(u) {
                    self.group_add(&mut air, u);
                } else {
                    self.group_add(&mut rest, u);
                }
            }
            if !rest.list.is_empty() {
                self.group_action_flight(&air, target, Flight::Strike);
                let whom = self.owner_of(target);
                if whom < crate::world::PLAYER_SLOTS && self.is_ally(g.who, whom) {
                    // SEAM: `action_guard` on an ally's building, which
                    // this crate's guard does not take.
                    if let Obj::Unit(t) = target {
                        self.group_action_guard(&rest, t, QueuePos::New, false);
                    }
                } else {
                    self.group_action_attack(&rest, target, true, QueuePos::New, 0);
                }
                return;
            }
        }
        let point = self.pos_of(target);
        for &u in &g.list {
            if !self.units[u].alive() {
                continue;
            }
            // `MOVE_TO`'s gate (`6fb5f0`..`6fb67b`): `ObjectData::can_carry`
            // on each member, whose room counts every plane homed on the
            // base, flying or standing (`num_aircraft_here(0)`, item 3991).
            if let Some(b) = base
                && self.is_air_unit(u)
                && !self.base_can_carry(b, u)
            {
                continue;
            }
            if let Some(o) = self.units[u].orders.front_mut()
                && let Body::Strafe(ref mut sf) = o.body
            {
                match kind {
                    Flight::Strike => {
                        sf.target = Some(target);
                        sf.at = Some(point);
                        sf.returning = false;
                    }
                    Flight::Home => {
                        sf.home = base;
                        sf.target = None;
                        sf.at = None;
                        sf.returning = true;
                    }
                }
                sf.mandatory = true;
                o.flags |= flag::ACTION;
                continue;
            }
            let inside = self.units[u].inside;
            match kind {
                // A strike from inside a base: `crate::air`'s inside arm
                // (`docs/ORDERS.md` §38.2, item 836).
                Flight::Strike => {
                    if let Some(b) = inside {
                        self.strike_from_inside(u, b, target);
                    }
                }
                Flight::Home => {
                    if inside != base {
                        self.add_strafe_order(u, None, base, true, QueuePos::New, true);
                    }
                }
            }
        }
    }

    /// `Group::action_attack(o, whom, mandatory, queue, ignore)` (§10):
    /// three passes by domain, the `ignore` mask, and the `mandatory == 0`
    /// melee retarget — **as the steps of [`crate::attack_view`]** (item
    /// 3362), which the sweep holds to the original call for call; this is
    /// the crate's view over them ([`crate::attack_view::SimAttack`]).
    ///
    /// The one line that is the crate's own is the target's liveness: the
    /// original asks only `whom >= 0` (its caller never hands a dead one).
    pub fn group_action_attack(
        &mut self,
        g: &Group,
        target: Obj,
        mandatory: bool,
        queue: QueuePos,
        ignore: i32,
    ) {
        if !self.active(target) {
            return;
        }
        let (whom, wwho) = crate::attack_view::obj_key(self, target);
        let q = match queue {
            QueuePos::First => crate::attack_view::QUEUE_FIRST,
            QueuePos::Last => crate::attack_view::QUEUE_LAST,
            QueuePos::New => crate::attack_view::QUEUE_NEW,
        };
        let mut v = crate::attack_view::SimAttack {
            sim: self,
            g,
            target,
            insert: None,
        };
        crate::attack_view::action_attack_steps(
            &mut v,
            whom,
            wwho,
            i32::from(mandatory),
            q,
            ignore,
        );
    }
}
