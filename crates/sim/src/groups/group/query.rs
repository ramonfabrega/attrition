//! The queries and the per-frame pass: `Groups::process`, the leader, speed, stance and location, and positions and facing (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    /// `Groups::process@006fa210` — once a frame, from
    /// `GameDaemon::process_all` after the markets and before
    /// `Armies::process_all` and the objects: for **one** slot of each
    /// player, the cursor `proc_group`, the same prune `Group::normalize`
    /// runs and then `speed = new_speed = UnitData::speed(find_leader)`, or
    /// 0 for a group with no leader (§3.3, §19).
    ///
    /// The cursor starts at 0 (`Groups::Groups`, `Groups::clear`) and steps
    /// once a call, wrapping at 64, so on frame `f` it is `f mod 64` — and
    /// every group's cap goes back to its leader's own speed once every 64
    /// frames whatever its members have reported. It is the writer
    /// `docs/GROUPS.md` §18 did not count: Great Lakes' raid walks at its
    /// slow squad's 25 from frame 10241 because 10241 is `65 mod 64`'s
    /// frame, with nobody in the group able to report.
    ///
    /// The prune is the back-pointer's (§23): a member is kept while it is
    /// alive and its `+0x80` names this slot, on an army's list and a
    /// pushed one alike. `find_role` is not modelled — no consumer of
    /// `GroupData::role` is.
    pub(crate) fn groups_process(&mut self, frame: i64) {
        let s = u8::try_from(frame.rem_euclid(64)).expect("under 64");
        for w in 0..self.armies.len() {
            let who = w as Player;
            for a in 0..self.armies[w].list.len() {
                if self.armies[w].list[a].group.pool != Some(s) {
                    continue;
                }
                // `normalize`'s prune, on the army's list too (§23): a
                // member whose `+0x80` no longer names this slot is
                // dropped. The army's own counts are `Army::normalize`'s
                // and wait for it.
                let keep: Vec<usize> = self.armies[w].list[a]
                    .units
                    .iter()
                    .copied()
                    .filter(|&u| self.units[u].alive() && self.units[u].group_ptr == Some(s))
                    .collect();
                self.army_keep_units(w, a, keep);
                let g = self.army_group(who, a);
                let v = self.group_compute_speed(&g);
                let st = &mut self.armies[w].list[a].group;
                st.speed = v;
                st.new_speed = v;
            }
            for i in 0..self.orders.pushed.len() {
                if self.orders.pushed[i].who != who || self.orders.pushed[i].state.pool != Some(s) {
                    continue;
                }
                // The shift is over all five arrays (`normalize@00711540`'s
                // inner loop, item 1776): the member's `off`, `curr` and
                // `angles` entries leave with it.
                for k in (0..self.orders.pushed[i].list.len()).rev() {
                    let u = self.orders.pushed[i].list[k];
                    if self.units[u].alive() && self.units[u].group_ptr == Some(s) {
                        continue;
                    }
                    let p = &mut self.orders.pushed[i];
                    Self::seat_remove_at(&mut p.list, &mut p.state, k);
                }
                // A building group's members take the same test's first
                // half (`(+8 & 1) == 0`, the object no longer in use): a
                // building closed under the group leaves it on the slot's
                // turn, and an emptied slot is the dump's no longer.
                // run1512's `[2007]`, disbanded on 832, leaves slot 1 on
                // 833, `833 mod 64` (item 3918, `docs/GOLDEN.md` §165).
                let buildings = &self.buildings;
                self.orders.pushed[i].builds.retain(|&b| buildings[b].alive);
                let g = crate::group::Group {
                    who,
                    army: None,
                    pushed: Some(i),
                    list: self.orders.pushed[i].list.clone(),
                };
                let v = self.group_compute_speed(&g);
                let st = &mut self.orders.pushed[i].state;
                st.speed = v;
                st.new_speed = v;
            }
        }
    }

    // ------------------------------------------------------------------
    // The queries (§4.3, §4.4)
    // ------------------------------------------------------------------

    /// `GroupData::find_leader` (§4.4): the active captain with the lowest
    /// `FormData::type_cat`, on the map if any is, ties to the first.
    ///
    /// The key is live as of 2026-08-26 — `sim::form::type_cat` was built
    /// for `Form::compute` and this is its second reader. The listing at
    /// `0070ccb0` gives the winner test as `local_8 < 0 || cat < best`, so
    /// the **first** qualifying member always takes the lead and only a
    /// **strictly** lower category displaces it; a tie goes to the first.
    /// Two passes, and only the first tests `is_on_map`.
    ///
    /// A typeless unit — a harness fixture, never an install's — keys at
    /// `NUM_FORM_CAT`, so it leads only when it is the first candidate and
    /// loses to any typed one that follows.
    pub fn group_find_leader(&self, g: &Group) -> Option<usize> {
        self.group_find_leader_slot(g).map(|(u, _)| u)
    }

    /// [`Self::group_find_leader`] with the out-parameter the original also
    /// fills: the winner's **index in `list`**, which `compute_form` uses to
    /// read the leader's own `angles` byte (§6.3).
    pub fn group_find_leader_slot(&self, g: &Group) -> Option<(usize, usize)> {
        let human = self.cities_state.nation[g.who as usize].human;
        for on_map_only in [true, false] {
            let mut best: Option<(usize, usize, usize)> = None;
            for (i, &u) in g.list.iter().enumerate() {
                if !self.units[u].alive() || !self.units[u].captain {
                    continue;
                }
                if on_map_only && !self.units[u].on_map {
                    continue;
                }
                let key = self.units[u]
                    .ty
                    .map_or(crate::form::cat::FORM_CAT_NUM, |t| {
                        crate::form::type_cat(&self.tables.unit_types[t], t, human)
                    });
                if best.is_none_or(|(k, _, _)| key < k) {
                    best = Some((key, u, i));
                }
            }
            if let Some((_, u, i)) = best {
                return Some((u, i));
            }
        }
        None
    }

    /// `GroupData::is_on_map`: at least one active on-map captain.
    pub fn group_is_on_map(&self, g: &Group) -> bool {
        !g.list.is_empty()
            && g.list
                .iter()
                .any(|&u| self.units[u].alive() && self.units[u].captain && self.units[u].on_map)
    }

    /// `GroupData::num_valid`: members whose object is valid.
    pub fn group_num_valid(&self, g: &Group) -> i32 {
        g.list.iter().filter(|&&u| self.units[u].alive()).count() as i32
    }

    /// `Group::normalize@00711540` (§4.3): drop the dead, last to first,
    /// and then **recompute the group's speed from its leader**.
    ///
    /// The tail is the second half of §18: `find_role`, then
    /// [`Sim::group_compute_speed`] written into both halves of the pair.
    /// It is what primes a freshly pushed group — `Army::find_target`'s
    /// probe normalizes at the end of its own build
    /// ([`Sim::find_target_probe`](Self::find_target_probe)) — so the
    /// first frame of the first march is already capped, rather than
    /// running at the leader's own speed until its first report.
    pub fn group_normalize(&mut self, g: &mut Group) {
        g.list
            .retain(|&u| u < self.units.len() && self.units[u].alive());
        self.group_set_speed(g);
    }

    /// `Group::normalize`'s speed tail on its own, for the sites that
    /// normalize a **seated** group without a local [`Group`] to prune:
    /// `Group::kill_group_move@007123f0` and
    /// `Group::refresh_group_order@00713a50` both open with `normalize`.
    ///
    /// SEAM: the prune itself. The seat's list is not walked here, because
    /// membership is maintained elsewhere in this crate and dropping a
    /// dead member from a pool slot mid-frame is a different change.
    pub(crate) fn group_set_speed(&mut self, g: &Group) {
        let s = self.group_compute_speed(g);
        if let Some(st) = self.gstate_mut(g) {
            st.speed = s;
            st.new_speed = s;
        }
    }

    // ------------------------------------------------------------------
    // The speed cap (§14)
    // ------------------------------------------------------------------

    /// The cap `UnitData::get_speed@00608720`'s last arm reads —
    /// `groups[unit->group].speed` — **0 for a unit with no seat**, which
    /// is what the original's `group >= 0` test answers in the same
    /// breath: a zero caps nothing either way.
    ///
    /// Written without building a [`Group`], because this runs once per
    /// moving unit per frame and [`Sim::group_of`] clones the list.
    pub(crate) fn group_speed_of(&self, u: usize) -> i32 {
        match self.seat_of(u) {
            Some(Seat::Army(w, a)) => self.armies[w as usize].list[a].group.speed,
            Some(Seat::Pushed(i)) => self.orders.pushed[i].state.speed,
            None => 0,
        }
    }

    /// The cap and its accumulator — `GroupData::speed` and `new_speed` —
    /// of the seat `u` sits in, army or pool slot, for a probe's line.
    /// `None` for a unit with no seat. Read-only; nothing in the
    /// simulation calls it.
    pub fn group_speed_pair_of(&self, u: usize) -> Option<(i32, i32)> {
        let st = match self.seat_of(u)? {
            Seat::Army(w, a) => &self.armies[w as usize].list[a].group,
            Seat::Pushed(i) => &self.orders.pushed[i].state,
        };
        Some((st.speed, st.new_speed))
    }

    /// `Group::report_speed@00713bb0`, which `Unit::do_group_move` inlines
    /// at `5e8336` — a **follower** in formation drives the group's speed
    /// down to its own, never up.
    ///
    /// The value reported is `UnitData::speed`, layer two, not
    /// `get_speed` — so the ground the follower is standing on and the
    /// order it carries do not enter the cap the rest of the group walks
    /// at.
    pub(crate) fn group_report_speed(&mut self, g: &Group, speed: i32) {
        let Some(st) = self.gstate_mut(g) else { return };
        if speed < st.speed {
            st.speed = speed;
            st.new_speed = speed;
        }
    }

    /// `Group::leader_report_speed@007137f0`, which `Unit::do_group_move`
    /// inlines at `5e7a9a` — the **leader**, on every frame its own step
    /// succeeds, publishes the pass.
    ///
    /// `speed` takes what the accumulator held, and the accumulator
    /// restarts at the leader's own **uncapped** speed
    /// (`get_speed(x, y, 1)`). So the cap a member walks at is the
    /// *previous* pass's minimum, one frame stale, and a group whose slow
    /// members stop reporting drifts back up to the leader's own speed
    /// over two frames rather than at once.
    ///
    /// Neither function has a caller in the 48k-function export: both are
    /// inlined at their one call site, which is why a grep for callers
    /// finds nothing and reads like dead code.
    pub(crate) fn group_leader_report_speed(&mut self, g: &Group, speed: i32) {
        let Some(st) = self.gstate_mut(g) else { return };
        st.speed = st.new_speed;
        st.new_speed = speed;
        st.march = false;
    }

    /// `do_group_move`'s forced-march arm under the leader's report
    /// (`groups[+0x80] +0x4b = 1`): the leader's leader flags carry
    /// `0x8000` and `has_general(0x8000, -1)` found a marching hero.
    pub(crate) fn group_set_march(&mut self, g: &Group) {
        if let Some(st) = self.gstate_mut(g) {
            st.march = true;
        }
    }

    /// `Group::compute_speed@00707f80` — the group's speed as the four
    /// membership sites set it: the **leader's** `UnitData::speed`, or 0
    /// when the group holds buildings, holds nothing, or has no leader.
    ///
    /// The original recomputes at `Group::add`, `Group::kill`,
    /// `Group::normalize` and `Group::clear`, and the group machinery
    /// reaches all four constantly: a dozen `Group::is_*` queries
    /// normalize, `Group::kill_group_move` and
    /// `Group::refresh_group_order` open with one, and `Form::categorize`
    /// calls `Group::sort`, which kills a member and re-adds it.
    /// SEAM: `Group::add` and `Group::kill` recompute too, and neither is
    /// a site this crate has — `Form::categorize`'s call to `Group::sort`,
    /// which kills a member out of category order and re-adds it, is not
    /// modelled. What is modelled is [`Sim::group_normalize`] and the two
    /// sites that open with it, [`Sim::seat_normalize`], and
    /// `Army::normalize`'s own pass over its group
    /// ([`Sim::army_normalize`], item 1586).
    pub(crate) fn group_compute_speed(&self, g: &Group) -> i32 {
        if g.list.is_empty() {
            return 0;
        }
        self.group_find_leader(g).map_or(0, |u| self.unit_speed(u))
    }

    /// `GroupData::get_stance_type` (§4.4): the leader's, or the first
    /// member's that is neither `None` nor `Caster`.
    pub fn group_stance_type(&self, g: &Group) -> StanceType {
        let Some(leader) = self.group_find_leader(g) else {
            return StanceType::None;
        };
        let s = self.unit_stance_type(leader);
        if s != StanceType::None && s != StanceType::Caster {
            return s;
        }
        let mut caster = false;
        for &u in &g.list {
            let s = self.unit_stance_type(u);
            if s == StanceType::None {
                continue;
            }
            if s != StanceType::Caster {
                return s;
            }
            caster = true;
        }
        if caster { StanceType::Caster } else { s }
    }

    /// `ObjectData` vslot `+0x108` → the type's `UnitTypeData::get_stance_type@0061d350`
    /// (the PDB names the slot; `docs/GROUPS.md` §4.4). The order of the
    /// tests is the original's and it is load-bearing:
    ///
    /// 1. `role & MILITARY` → `PACKER` if `unit_flags2 & 4`, else
    ///    `COMBAT` — so a military caster is *combat*;
    /// 2. the four citizen ids (`role::ROLE_CITIZEN`, set for exactly those) →
    ///    `WORKER`;
    /// 3. `(unit_flags2 & 6) == 2` — a caster that does not pack → `CASTER`;
    /// 4. else `NONE`. Having an attack decides nothing on its own.
    ///
    /// `Profile::packs` is `unit_flags2 & 4` for a type from the install.
    pub(crate) fn unit_stance_type(&self, u: usize) -> StanceType {
        let Some(t) = self.units[u].ty else {
            return StanceType::None;
        };
        let c = &self.tables.unit_types[t];
        if c.cols.is(crate::ai_load::role::ROLE_MILITARY) {
            return if c.combat.packs {
                StanceType::Packer
            } else {
                StanceType::Combat
            };
        }
        if c.cols.is(crate::ai_load::role::ROLE_CITIZEN) {
            return StanceType::Worker;
        }
        if c.cols.flag2(uflags2::CASTER) && !c.combat.packs {
            return StanceType::Caster;
        }
        StanceType::None
    }

    /// `GroupData::get_stance_option@0070bab0`: the **modal** stance option
    /// over the members — a histogram of each valid member's `+0xb1` that
    /// answers `has_stance_type` (vslot `+0x104`) for the group's own
    /// stance type, argmax on a **strict `<`** so a tie goes to the lowest
    /// index, and 0 when nothing counts. This, not the leader's own stance,
    /// is what `action_stance`'s negative cycle steps from (§8).
    pub(super) fn group_stance_option(&self, g: &Group, ty: StanceType) -> i32 {
        let n = ty.options();
        if n == 0 {
            return 0;
        }
        // Six is the largest option count (`STANCE_COMBAT`).
        let mut counts = [0i32; 6];
        for &u in &g.list {
            if !self.units[u].alive() || self.unit_stance_type(u) != ty {
                continue;
            }
            let opt = if ty == StanceType::Combat {
                stance_option(self.units[u].combat.stance)
            } else {
                i32::from(self.units[u].stance)
            };
            if let Some(c) = counts.get_mut(opt as usize) {
                *c += 1;
            }
        }
        let mut best = 0usize;
        for i in 0..n as usize {
            if counts[i] > 0 && counts[best] < counts[i] {
                best = i;
            }
        }
        best as i32
    }

    /// `GroupData::get_form`: the `unit +0xaa` shared by every active
    /// on-map member, −1 as soon as two differ.
    pub fn group_get_form(&self, g: &Group) -> i32 {
        let mut form = -1;
        let mut seen = false;
        for &u in &g.list {
            if !self.units[u].alive() || !self.units[u].on_map {
                continue;
            }
            let f = i32::from(self.units[u].form);
            if seen && f != form {
                return -1;
            }
            form = f;
            seen = true;
        }
        form
    }

    /// `GroupData::get_loc@0070e030`: the leader's position, **and the two
    /// `(ox, oy)` substitutions on top of it** (§6.3, §12.5).
    ///
    /// Both gates are a `vector_dist` whose operands the decompiler loses
    /// to `unaff_EDI`/`unaff_ESI`; the listing at `70e109` and `70e190`
    /// supplies them, and `jg 0x180` makes both bounds **inclusive**:
    ///
    /// - the leader within `0x180` of the group's own `(ox, oy)` returns
    ///   `(ox, oy)` outright;
    /// - otherwise, the leader's **current move order's destination**
    ///   within `0x180` of `(ox, oy)` returns the leader's position
    ///   translated by the difference, `pos - order.dest + (ox, oy)` —
    ///   "where the group will be".
    ///
    /// The second arm's pair is `MoveOrder +0x4`/`+0x8`, which the type
    /// record names **`x`/`y`** — the order's destination. It is *not*
    /// `orig_x`/`orig_y`, which sit at `+0x44`/`+0x48`; the surrounding
    /// code reads the same in both spellings and only the layout settles
    /// it.
    ///
    /// This matters because [`Self::group_move`] feeds the answer to both
    /// the formation angle **and** `group_plan_path`'s start, so a
    /// substitution here moves the cell the leader's route is planned
    /// from.
    pub(super) fn group_loc(&self, g: &Group) -> Option<Pos> {
        let u = self.group_find_leader(g)?;
        let pos = self.units[u].pos;
        let o = self.group_o(g);
        // `70e0f8`/`70e103`: either coordinate negative and the record is
        // not a point at all — a group that has never moved.
        if o.x < 0 || o.y < 0 {
            return Some(pos);
        }
        if vector_dist(pos.x - o.x, pos.y - o.y) <= GROUP_LOC_NEAR {
            return Some(o);
        }
        // `70e14b`/`70e162`: the object must be a unit and `is_moving` —
        // which is `orderlist` head, `get_type`, non-zero — and then
        // `get_move_order` must hand back a `MoveOrder`. All three
        // collapse here to "the current order is a move".
        let Some(dest) = self.current_order(u).and_then(Order::move_dest) else {
            return Some(pos);
        };
        if vector_dist(dest.x - o.x, dest.y - o.y) > GROUP_LOC_NEAR {
            return Some(pos);
        }
        Some(Pos::new(pos.x - dest.x + o.x, pos.y - dest.y + o.y))
    }

    /// `GroupData::get_loc_to@0070c5d0` — the group's location for a
    /// **`QUEUE_LAST`** move, which is not where the leader stands but
    /// where it will *end up*: `UnitData::get_final_loc` walks the orders
    /// already on its list and hands back the first point one of them
    /// names.
    ///
    /// The `(ox, oy)` override is the same `0x180` window
    /// [`Self::group_loc`] uses, measured from the **unit** like it — not
    /// from the final point (`0070c6cc`; the decompile's operands are
    /// `unaff_` and the listing settles it, item 1528).
    ///
    /// SEAM: the `buildings` seat (`list[0]` rather than `find_leader`) and
    /// the `get_inside` hop for a garrisoned leader, neither of which any
    /// group this simulation builds reaches.
    pub(crate) fn group_loc_to(&self, g: &Group) -> Option<Pos> {
        let u = self.group_find_leader(g)?;
        let p = self.unit_final_loc(u);
        let o = self.group_o(g);
        // **The window is measured from the leader's own position**
        // (`70c6cc`–`70c702`: `objects[who][leader] +0x10/+0x14` less
        // `(ox, oy)`, `get_final_loc`'s answer already written to the
        // caller's pair), the decompile's `vector_dist(unaff_EBX,
        // unaff_EDI)` hiding the operands. Item 1528: a leader still
        // 3,400 units from the point it was ordered to leaves the window
        // shut, the answer the *final* location, and a formation re-issued
        // at `QUEUE_LAST` to a point behind that location is laid out
        // reversed (`docs/AI.md` §142).
        let pos = self.units[u].pos;
        if o.x >= 0 && o.y >= 0 && vector_dist(pos.x - o.x, pos.y - o.y) <= GROUP_LOC_NEAR {
            return Some(o);
        }
        Some(p)
    }

    /// `UnitData::get_final_loc@00608040` — where this unit's order list
    /// leaves it.
    ///
    /// **The walk starts at the bottom of the ring** — the order issued
    /// last at `QUEUE_LAST`, the one that runs last — and climbs toward the
    /// current order, stopping at the first that names a point: the
    /// *final* location is the deepest point on the list (item 2055,
    /// `docs/AI.md` §206). The ring's anchor (`orderlist +0x14`) is where
    /// `get_final_loc` starts and `OrderList::log_data` prints first, and
    /// the current order is the node before it (`kill_current_order` reads
    /// `+0x14 → +4`), so the dump prints the bottom first and the current
    /// order last. ~~From the head~~ until item 2055: a re-issued charge
    /// read the charge's own point, (6216, 29784), where the original reads
    /// the copy beneath it, (6072, 29832). A move order hands back its
    /// destination (vslot `+0x14`, then `+0xb8`'s `(+4, +8)` — the order's
    /// `x`/`y`, not its live waypoint), and an order carrying a
    /// `TargetOrder` hands back its target **object's** position, provided
    /// that object is still `flags & 1`. An order naming neither is walked
    /// past.
    ///
    /// Then the answer is checked with `invalid_loc(unit, tile, 1, …)` —
    /// buildings ignored, which is why a farm's own cell is an answer at all
    /// — and a point that fails it is replaced by the unit's own position,
    /// as is a list that named nothing.
    ///
    /// **The target lives on the unit here, not on the order**
    /// (`docs/GROUPS.md` §12), so the target arm reads
    /// [`combat::State::target`] for an attack and the order's own building
    /// for the three that carry one. SEAM: a gather order out at a resource
    /// tile, and a queued attack behind another unit's — this crate keeps
    /// one target per unit, so a list holding two attacks answers with the
    /// live one for both.
    ///
    /// **A guard is a `TargetOrder` too**, and it answers the escorted
    /// unit's own position, never its post (item 2055, `docs/AI.md` §206):
    /// Great Sahara's leader `1/17` holds a `GUARDORDER` on the MLRS `1/144`
    /// at its head on block 4265, and the original's `get_loc_to` on run821's
    /// packet answers (14743, 25626), the MLRS's point. Walked past, the
    /// answer was the charge's (6216, 29784), and `compute_form`'s reverse
    /// on the last re-issue never fired. **And so is a follow** (item 2131,
    /// `docs/GOLDEN.md` §78): `FollowOrder`'s block carries `TargetOrder`'s
    /// `is_targeted` and `get_target_order` thunks, and run827's two
    /// shift-moves on following squads lay their formation out from the
    /// leader's point — under a leg, too, since the walk starts at the
    /// bottom.
    pub(crate) fn unit_final_loc(&self, u: usize) -> Pos {
        let here = self.units[u].pos;
        let found = self.units[u]
            .orders
            .iter()
            .rev()
            .find_map(|o| match o.body {
                Body::Move(m) => Some(m.dest),
                Body::Attack(_) => self.units[u]
                    .combat
                    .target
                    .and_then(|t| self.obj_alive_pos(t)),
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    self.obj_alive_pos(crate::combat::Obj::Building(b))
                }
                Body::Gather(gt) => self.obj_alive_pos(crate::combat::Obj::Building(gt.building)),
                Body::Guard(gd) => self.obj_alive_pos(crate::combat::Obj::Unit(gd.target)),
                Body::Follow(f) => self.obj_alive_pos(crate::combat::Obj::Unit(f.target)),
                _ => None,
            });
        let Some(p) = found else { return here };
        if self.invalid_loc(u, p.tile(), true, false, false, false, false) != 0 {
            return here;
        }
        p
    }

    /// `flags & 1` alone — `get_final_loc`'s own test on a target object,
    /// which unlike [`Sim::active`] asks nothing about the map or combat.
    pub(super) fn obj_alive_pos(&self, o: crate::combat::Obj) -> Option<Pos> {
        let alive = match o {
            crate::combat::Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive()),
            crate::combat::Obj::Building(b) => self.buildings.get(b).is_some_and(|b| b.alive),
        };
        alive.then(|| self.pos_of(o))
    }

    /// `GroupData +0x18`/`+0x1c` — the point the last move was ordered
    /// **to**, before any slot offset. A group with no army has no record
    /// to read, and the original's `(-1, -1)` initialiser is what a
    /// never-moved group holds.
    pub(crate) fn group_o(&self, g: &Group) -> Pos {
        self.gstate(g).map_or(Pos::new(-1, -1), |st| st.o)
    }

    // ------------------------------------------------------------------
    // The type tests the actions ask (§6.5, §7, §9, §10)
    // ------------------------------------------------------------------

    pub(crate) fn is_siege_unit(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t].combat.siege
                || self.tables.unit_types[t].cols.flag(uflags::FLAG_SIEGE)
        })
    }

    pub(crate) fn is_hero_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.flag2(uflags2::FLAG2_GENERAL))
    }

    /// `UnitData::is_supply@0046ce80`, the base every unit vtable's `+0xcc`
    /// names: `unit_flags2 & 0x40`, the raw bit (swept by
    /// `sweep::unit_is_supply`). A hero that carries it — The Citizen, TY
    /// 356 — answers yes, so `action_siege_attack_to`'s supply arm takes
    /// it before a later Supply Wagon (item 3682, `docs/AI.md` §346.2).
    pub(crate) fn is_supply_unit(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t]
                .cols
                .flag2(uflags2::SUPPLY_OR_HERO)
        })
    }

    pub(crate) fn is_special_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.flag2(uflags2::FLAG2_SCOUT))
    }

    pub(crate) fn group_domain(&self, u: usize) -> Domain {
        self.units[u]
            .ty
            .map_or(Domain::Land, |t| self.tables.unit_types[t].combat.domain)
    }

    /// `UnitData::is_plane@0046ce40` — `domain == 2 && !(unit_flags & 0x20)`.
    /// There is no altitude test: a plane is skipped whether it is in the
    /// air or on a runway (§7). Bit `f` is `unitrules.xml`'s "flies like a
    /// helicopter", carried by exactly three types — `Helicopter` and the
    /// two `Attack Helicopter`s, all of them `<DOMAIN>Air` — so a
    /// **helicopter is not a plane** and is ordered like a ground unit.
    pub fn is_plane(&self, u: usize) -> bool {
        self.group_domain(u) == Domain::Air
            && !self.units[u]
                .ty
                .is_some_and(|t| self.tables.unit_types[t].cols.flag(uflags::FLAG_HELICOPTER))
    }

    /// `Group::is_attacking_to@0070ea70` (item 1012, `docs/COMBAT.md`
    /// §66): after `normalize`, every member that is alive and on the map
    /// and has a head order is walking an attack-move — `ATTACK_TO` (2)
    /// or `GROUP_ATTACK_TO` (`0x15`). A member with no order passes; one
    /// member fighting, or walking to where it will fight, is a no.
    pub(crate) fn group_is_attacking_to(&mut self, g: &Group) -> bool {
        let mut g = g.clone();
        self.group_normalize(&mut g);
        g.list.iter().all(|&u| {
            !(self.units[u].alive() && self.units[u].on_map)
                || self
                    .current_order(u)
                    .is_none_or(|o| matches!(o.index(), index::ATTACK_TO | index::GROUP_ATTACK_TO))
        })
    }

    /// Whether a member takes an order at all: active, on the map, and not
    /// a plane. Every action's inner loop opens with this.
    pub(super) fn group_member_orderable(&self, u: usize) -> bool {
        self.units[u].alive() && self.units[u].on_map && !self.is_plane(u)
    }

    // ------------------------------------------------------------------
    // The small pieces the actions share
    // ------------------------------------------------------------------

    /// §6.5's `ai_group`: `!(leaders[who].flags & 4) && group.army >= 0`.
    pub(super) fn ai_army_group(&self, g: &Group) -> bool {
        g.army.is_some()
            && self
                .cities_state
                .nation
                .get(g.who as usize)
                .is_some_and(|n| !n.human)
    }

    /// `ObjectsData::find_city(SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)`
    /// around a point.
    pub(super) fn city_near(&self, who: Player, p: Pos) -> Option<usize> {
        let reg = self.world.tregion(p.tile());
        let mut best: Option<(i32, usize)> = None;
        for c in self.cities_of(who) {
            if self.cities_state.cities[c].reg != reg {
                continue;
            }
            let q = self.cities_state.cities[c].pos;
            let d = crate::world::vector_dist((p.x - q.x).abs(), (p.y - q.y).abs());
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// §6.5's stable: `can_garrison` → a garrison order, else a move to a
    /// spot around the city between `0x300` and `0x600`.
    pub(super) fn stable_in_city(&mut self, u: usize, c: usize, angle: Angle) {
        let b = self.cities_state.cities[c].building;
        let p = self.cities_state.cities[c].pos;
        if self.units[u]
            .ty
            .zip(self.buildings[b].ty)
            .is_some_and(|(ut, bt)| self.can_garrison(ut, bt))
        {
            self.add_garrison_order(u, b, false, QueuePos::New, false);
            return;
        }
        let spot = self
            .find_nearby_spot(u, p, 0x300, 0x600, 0, angle, None)
            .unwrap_or(p);
        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::New, false);
    }

    pub(crate) fn group_o_angle(&self, g: &Group) -> Angle {
        self.gstate(g).map_or(Angle(0), |st| st.o_angle)
    }

    /// The group's own id (`GroupData +0x4`), the pool index: 64 slots a
    /// player, so a group's is `who × 64 +` the pool slot its record sits in
    /// (item 1926: an army's `group.pool`, a [`Pushed`] group's `state.pool`,
    /// which `get_open_slot` hands out after the AI scripts' building groups;
    /// item 1863 had the army's own number, which parted by one to four slots).
    /// A record with no slot yet falls back to its own number.
    pub(super) fn group_id(&self, g: &Group) -> i32 {
        let who = i32::from(g.who);
        match (g.army, g.pushed) {
            (Some(s), _) => {
                let slot = self
                    .armies
                    .get(g.who as usize)
                    .and_then(|a| a.list.get(s))
                    .and_then(|a| a.group.pool)
                    .map_or(s as i32, i32::from);
                who * 64 + slot
            }
            (None, Some(i)) => {
                who * 64 + self.orders.pushed[i].state.pool.map_or(i as i32, i32::from)
            }
            (None, None) => -1,
        }
    }

    pub(super) fn group_order_num(&self, g: &Group) -> i32 {
        self.gstate(g).map_or(0, |st| st.order_num)
    }

    /// `Group::update_positions@00713810(o, who)` — the slot table
    /// rotated once a frame from `do_group_move`'s leader arm. It is what
    /// makes a formation bend around a corner.
    ///
    /// **And the angle is not the leader's heading.** `713844` loads
    /// `UnitData::angle` as the default, and then `7138e1`–`71390f`
    /// replaces it with `find_angle(order.waypoint − leader.pos)` whenever
    /// the leader's head order is a move that **has** a waypoint
    /// (`MoveOrder +0x10 dest`). So the block points where the leader is
    /// *going*, not where it is *facing*, and the two come apart on every
    /// frame of a turn — which is exactly where a follower would otherwise
    /// be walking into its leader.
    pub(crate) fn group_update_positions(&mut self, g: &Group, leader: usize) {
        if self.gstate(g).is_none() {
            return;
        }
        let mut v = SimUpdatePositions {
            sim: self,
            g,
            u: leader,
        };
        crate::update_positions_view::update_positions_steps(&mut v);
    }

    /// A follower's target point this frame: the leader's **current**
    /// position plus slot `i`'s rotated offset.
    pub(crate) fn group_slot_point(&self, g: &Group, leader: usize, i: usize) -> Option<Pos> {
        let off = *self.gstate(g)?.curr.get(i)?;
        let p = self.units[leader].pos;
        Some(Pos::new(p.x + off.x, p.y + off.y))
    }

    /// After a refresh, every member's group order names the member that
    /// took the formation over (`refresh_group_order`'s last third —
    /// `modify_group_order` per member).
    pub(crate) fn group_rewrite_leader(&mut self, g: &Group, leader: usize, id: i64) {
        for i in 0..g.list.len() {
            let m = g.list[i];
            for o in &mut self.units[m].orders {
                if let Some(mv) = o.move_mut()
                    && let Some(gm) = mv.group.as_mut()
                    && gm.id == id
                {
                    gm.leader = leader;
                }
            }
        }
    }

    pub(crate) fn bump_order_num(&mut self, g: &Group) {
        if let Some(st) = self.gstate_mut(g) {
            st.order_num += 1;
        }
    }

    /// `action_move_near`'s writes to the record after `compute_form`: the
    /// origin and its angle, only for `QUEUE_NEW` and `QUEUE_LAST` (§6.3) —
    /// so `charge`'s `QUEUE_FIRST` leaves them. `form` (`70535a`) is written
    /// before the layout, in [`Self::group_action_move_to`].
    pub(super) fn record_move(&mut self, g: &Group, to: Pos, angle: Angle, queue: QueuePos) {
        let Some(st) = self.gstate_mut(g) else { return };
        if queue != QueuePos::First {
            st.o = to;
            st.o_angle = angle;
        }
    }

    /// The different-region arm's `+0x80 = this->id`: the member names the
    /// group. A group with no seat has no id to write.
    pub(super) fn repoint_member(&mut self, g: &Group, u: usize) {
        if let Some(pool) = self.gstate(g).and_then(|st| st.pool) {
            self.units[u].group_ptr = Some(pool);
        }
    }

    /// `Group::refresh_group_order@00713a50` (§6.8) — hand the group's
    /// formation over to `member`, which becomes the block's origin and the
    /// object every member's group order names.
    ///
    /// `Unit::do_group_move@005e79a0` calls it when the unit named by the
    /// order's `oxx` is no longer usable — dead, off the map, in another
    /// group, or no longer holding a matching group order — and the mover is
    /// still more than `0x5ff` away. The trigger is order machinery the
    /// simulation stands in for (§12's third seam); the **effect** is this,
    /// and it is what run31's record actually shows: the table it dumps is
    /// `compute_dests`' output re-origined nought or once.
    ///
    /// Returns whether the group had a slot for `member`.
    pub fn group_refresh_order(&mut self, g: &Group, member: usize) -> bool {
        // `00713a5a`: `normalize(this)` opens it, and its tail is the
        // group's speed (§18).
        self.group_set_speed(g);
        let Some(i) = g.list.iter().position(|&u| u == member) else {
            return false;
        };
        let Some(st) = self.gstate_mut(g) else {
            return false;
        };
        st.reorigin(i);
        // `713b94`: the tail is `call 0x713810` — the whole of
        // `update_positions` off the taking member, **waypoint arm
        // included**, not a turn by its heading (Great Sahara 17493, item
        // 1206). `modify_group_order` before it writes only the group
        // order's `oxx` and `whose`, so the waypoint it reads is the
        // member's own.
        self.group_update_positions(g, member);
        true
    }

    /// `Group::compute_form`'s own reverse test (§6.3): is the leader
    /// pointing more than 90° away from the formation's bearing?
    ///
    /// `compute_form@00707c80` flips `facing` when it holds, calls
    /// `Form::compute` with the flipped value, and flips it back — so the
    /// mirror the layout uses is `facing XOR this`, and the record only ever
    /// shows `facing`. The heading is the **leader's** own, offset by its
    /// slot's angle byte, and the compare is unsigned on the wrapped
    /// difference so both bounds are inclusive.
    ///
    /// The byte's **sign** in that sum is the listing's, not the
    /// decompiler's paraphrase: `707e95`–`707ea6` is `movsbl angles[slot]`,
    /// `shll $0x18`, `sub` — so `compute_form` **subtracts** the packed
    /// byte, and `705f42`–`705f4d` **adds** the same product onto the angle
    /// the order carries (§6.6 step 6). No capture separates the two, since
    /// every group any run has dumped lays out in Line and carries `angles`
    /// of all zero; a formation that leans is still what would show it.
    pub(super) fn group_leader_faces_away(&self, g: &Group, angle: Angle) -> bool {
        let Some(heading) = self.group_leader_bearing(g) else {
            return false;
        };
        reversing(Angle(heading.0.wrapping_sub(angle.0)))
    }

    /// The quantity both halves of §6.3 are built on: the leader's own
    /// heading less its packed slot byte, `%esi` at `707ea8`.
    ///
    /// `compute_form` computes it once and uses it twice — the zero-delta
    /// arm **assigns** it as the formation angle (`707e3d`–`707e6d`) and the
    /// mirror test compares it against whatever angle was chosen — so it is
    /// one function here rather than two readings of the same listing.
    pub(super) fn group_leader_bearing(&self, g: &Group) -> Option<Angle> {
        let (u, slot) = self.group_find_leader_slot(g)?;
        let byte = self.group_angles(g).get(slot).copied().unwrap_or(0);
        Some(Angle(
            self.units[u]
                .movement
                .heading
                .0
                .wrapping_sub(i32::from(byte) << 24),
        ))
    }

    /// `GroupData::facing` — the mirror flag the layout reads.
    pub(super) fn group_facing(&self, g: &Group) -> bool {
        self.gstate(g).is_some_and(|st| st.facing)
    }

    /// The group this unit belongs to, if any — the original's
    /// `unit +0x80` read straight into the pool.
    ///
    /// Two seats hold a group here: an army's (`docs/ARMY.md` §3.2) and a
    /// [`Pushed`] slot, and the pointer ([`Unit::group_ptr`]) names one of
    /// them through [`Sim::seat_of`]. Until item 557 this asked the lists,
    /// which cannot tell a unit its group has dropped from one it never
    /// left (§23).
    pub(crate) fn group_of(&self, u: usize) -> Option<Group> {
        match self.seat_of(u)? {
            Seat::Army(w, a) => Some(self.army_group(w, a)),
            Seat::Pushed(i) => self.pushed_group_at(i),
        }
    }

    /// Is this unit the leader of its own group?
    ///
    /// `Unit::set_angle` and `Unit::kill_current_order` both ask it the same
    /// way and both only act when the answer is yes: `find_leader` for a
    /// unit group, `list[0]` for a buildings group, and `-1` — nobody —
    /// while the group holds fewer than one member.
    pub(super) fn is_group_leader(&self, u: usize, g: &Group) -> bool {
        g.num() >= 1 && self.group_find_leader(g) == Some(u)
    }

    /// `Unit::set_angle@00605400`, the second writer of `GroupData::facing`
    /// (§4.1) and the ordinary source of a live group's `facing 1`.
    ///
    /// Setting a unit's heading to something 90° or more from the one it
    /// has **toggles its group's mirror flag**, if that unit is the group's
    /// leader. The original also flips the unit's own `unit_masks & 2`,
    /// which nothing this simulation models reads.
    ///
    /// The field is `UnitData::angle` — [`Movement::heading`], not the
    /// facing. `set_angle` compares against it, writes it, and passes the
    /// same value on to guy 0 as its `des_angle`; guy 0's own `angle`, the
    /// one the step is taken along, is turned by `Guy::do_turn` and is not
    /// touched here.
    ///
    /// `Unit::move_step`'s call is modelled — the one a marching unit
    /// makes every frame — and `Unit::do_build`'s turn to its site (item
    /// 1330). `come_out`'s turn to a boat's heading reaches the group arm
    /// through [`Self::toggle_leader_facing`] (item 2193). The other
    /// callers (`do_gather`, `fight`, …) are turns this simulation makes
    /// through `Movement::set_heading` or not at all, and each is a place a
    /// group's flag would move that this one leaves still.
    ///
    /// **And the tail of it is `Guy::set_angle(guy 0, angle, 0)`**, whose
    /// crew loop rewrites every tracked figure's `des_angle` and `des`
    /// from guy 0's own point and *this* angle — the **heading**, which is
    /// the bearing `find_angle` has just returned and not the facing the
    /// step is taken along ([`Sim::crew_des`],
    /// `docs/MOVEMENT.md`, "Who writes it, and when", third row). It
    /// fires at the **top** of `move_step`, ahead of the collision block,
    /// so a crew figure that walked exactly onto its destination last
    /// frame is one unit off it again before the blocked stand's
    /// `set_anim` asks: `Guy::set_anim`'s walking-guy early return then
    /// takes it, and it does not roll. That was East Indies 6571 — the
    /// crew's second `Unit::move_step+0x823` draw, and the difference is
    /// 720,896 of a turn, one unit on each axis of a (−48, −192) track.
    pub fn unit_set_angle(&mut self, u: usize, angle: Angle) {
        self.unit_set_angle_as(u, angle, false);
    }

    /// `Unit::set_angle(angle, target, param_3)` with its third argument
    /// set — `Unit::do_cast@005ebfe0`'s turn to its target (`5ec619`
    /// pushes 1; item 3920, `docs/GOLDEN.md` §167). `Guy::set_angle@005d9010`
    /// hands `param_3` to its crew loop, which then **puts** each tracked
    /// figure on its point (`set_angle(crew, des_angle, 1)`,
    /// `set_new_location(crew, des, 1)`) rather than telling it the point:
    /// run1514c's Scout crew stands on its rotated track the cast's first
    /// frame, `stopped` 1, and plays the cast's `ATTACK2` with guy 0.
    pub(crate) fn unit_set_angle_as(&mut self, u: usize, angle: Angle, snap: bool) {
        let turned = reversing(Angle(
            angle.0.wrapping_sub(self.units[u].movement.heading.0),
        ));
        self.units[u].movement.heading = angle;
        self.units[u].movement.crew_des = Some(self.frame);
        // `605424`: the unit's own mirror flips before the leader test.
        if turned {
            self.units[u].movement.mirror = !self.units[u].movement.mirror;
        }
        let from = self.units[u].movement.body.pos;
        self.crew_des(u, from, angle, snap);
        if turned {
            self.toggle_leader_facing(u);
        }
    }

    /// `Unit::set_angle@00605400`'s group arm on its own: a turn of 90° or
    /// more by the group's **leader** (`find_leader`, through the unit's
    /// `+0x80`) toggles `GroupData::facing`. Its two callers are
    /// [`Self::unit_set_angle`] and `come_out`'s turn to the host's angle
    /// (`crate::transport`'s `land_tail`, item 2193), which writes the
    /// heading through [`crate::Movement::set_facing`] and so reaches
    /// here only for the flag.
    pub(crate) fn toggle_leader_facing(&mut self, u: usize) {
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(st) = self.facing_record_mut(&g)
        {
            st.facing = !st.facing;
        }
    }

    /// `Unit::kill_current_order@005e2cb0`'s move branch, the third writer
    /// of `GroupData::facing` (§4.1, §6.3) — and the one that makes a
    /// group's *second* click behave.
    ///
    /// A dying move order carries the mirror its formation was laid out
    /// with. When the unit is its group's leader, that mirror is written
    /// back onto the group — **inverted if the unit has since turned
    /// around**, by the same `reversing` window against the order's own
    /// angle. So `compute_form` never sees the flag the leader's marching
    /// left behind; it sees the last layout's own answer.
    ///
    /// The original also writes the result into `unit_masks & 2`
    /// ([`crate::Movement::mirror`]) for every unit, before the leader
    /// test, and skips the whole branch for an order type outside
    /// `{1, 2, 3, 4, 0x12, 0x13, 0x15}` — the move family, which is
    /// [`Body::Move`] here.
    pub(crate) fn hand_back_facing(&mut self, u: usize, order_facing: bool, order_angle: Angle) {
        let turned = reversing(Angle(
            self.units[u].movement.heading.0.wrapping_sub(order_angle.0),
        ));
        let f = order_facing != turned;
        // `5e3087`/`5e308d`: `unit_masks & 2` takes it, leader or not.
        self.units[u].movement.mirror = f;
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(st) = self.facing_record_mut(&g)
        {
            st.facing = f;
        }
    }

    /// The record a member's own write of `GroupData::facing` lands on —
    /// `kill_current_order`'s hand-back and `set_angle`'s toggle, both
    /// through the unit's `+0x80`. That is the seat's record, except while
    /// `action_siege_attack_to`'s stack sub-group holds the seat: then it
    /// is the parent's, set aside in [`Sim::set_aside`] (`docs/AI.md` §212).
    pub(super) fn facing_record_mut(&mut self, g: &Group) -> Option<&mut GroupState> {
        if self
            .orders
            .set_aside
            .as_ref()
            .is_some_and(|(who, army, pushed, _)| {
                (*who, *army, *pushed) == (g.who, g.army, g.pushed)
            })
        {
            return self.orders.set_aside.as_mut().map(|(.., st)| st);
        }
        self.gstate_mut(g)
    }

    /// `GroupData::angles` — carried in because `Form::compute` does not
    /// clear it and Column and Mob never write it (`form::Form::new`).
    pub(super) fn group_angles(&self, g: &Group) -> Vec<i8> {
        self.gstate(g)
            .map(|st| st.angles.clone())
            .unwrap_or_default()
    }

    /// `Form::compute`'s writes to the record: `form_num`, the per-member
    /// offsets quantised by the floor divide by 48 (§6.4), the group's
    /// `o_angle`/`o_dist` from the **leader's** slot rather than the
    /// order's point, and `update_positions`' `curr`.
    ///
    /// The last of those is really `Unit::do_group_move`'s, once a frame
    /// off the leader's own heading; it is written here from the same
    /// heading so the record has a value the moment the move is issued,
    /// which is what run29's window compares against.
    pub(super) fn record_form(&mut self, g: &Group, f: &crate::form::Form, to: Pos) {
        if self.gstate(g).is_none() {
            return;
        }
        // The group's table is quantised from the `Form`'s **before**
        // `compute_form`'s tail negates either (§6.3): the tail negates two
        // tables that have already been written, and `quantise` is a floor
        // divide, so negating the raw value first would round the other
        // way.
        let q = |v: i32| {
            if f.flipped {
                -crate::form::Form::quantise(-v)
            } else {
                crate::form::Form::quantise(v)
            }
        };
        let off: Vec<(i32, i32)> = f.off.iter().map(|&(x, y)| (q(x), q(y))).collect();
        let theta = f.o.map_or(Angle(0), |u| self.units[u].movement.heading);
        let curr = Sim::form_update_positions(&off, theta);
        let (o_angle, o_dist) = Sim::form_leader_offset(f, to);
        let num = g.num();
        let angles = f.angles.clone();
        let Some(st) = self.gstate_mut(g) else { return };
        st.form_num = num;
        st.off = off;
        st.curr = curr;
        st.angles = angles;
        st.o_angle = o_angle;
        st.o_dist = o_dist;
    }

    /// `WorldData::restrict` on a destination (§6.1).
    pub(crate) fn restrict_pos(&self, p: Pos) -> Pos {
        Pos::new(
            p.x.clamp(0, self.world.width() * crate::world::UNITS_PER_CELL - 1),
            p.y.clamp(0, self.world.height() * crate::world::UNITS_PER_CELL - 1),
        )
    }
}

/// The arm of `Group::action_eject_all@00710b40` that empties the group's own
/// buildings: `cmd_who` (the original's `param_2`) is negative, or the command
/// names no other player (`eject_who`, `param_4`, negative) and `cmd_who` is
/// the group's player. Plain data, so `sweep::group_action_eject_all` drives it.
pub(crate) fn eject_all_own_arm(group_player: i32, eject_who: i32, cmd_who: i32) -> bool {
    (eject_who < 0 && cmd_who == group_player) || cmd_who < 0
}

/// One building of that arm: alive, then holding a squad, then no hangar —
/// the original asks `num_inside` and `can_carry(AIR)` only as far as the
/// answer needs, so the two are closures.
pub(crate) fn eject_all_wants(
    alive: bool,
    holds_squad: impl FnOnce() -> bool,
    is_hangar: impl FnOnce() -> bool,
) -> bool {
    alive && holds_squad() && !is_hangar()
}

/// `Group::update_positions`' view over the sim: the leader unit and the
/// group's slot table (`update_positions_view.rs`).
pub(super) struct SimUpdatePositions<'a> {
    pub(super) sim: &'a mut Sim,
    pub(super) g: &'a Group,
    pub(super) u: usize,
}

impl crate::update_positions_view::UpdatePositionsView for SimUpdatePositions<'_> {
    fn heading(&mut self) -> i32 {
        self.sim.units[self.u].movement.heading.0
    }
    fn waypoint(&mut self) -> Option<(i32, i32)> {
        match self.sim.current_move(self.u) {
            Some(m) if m.has_waypoint => Some((m.waypoint.x, m.waypoint.y)),
            _ => None,
        }
    }
    fn leader_point(&mut self) -> (i32, i32) {
        let p = self.sim.units[self.u].pos;
        (p.x, p.y)
    }
    /// `update_positions@00713810` loops over `form_num` slots, not `num`: a
    /// member past it keeps the `curr` it was given — a siege unit's, copied
    /// back from the sub-group (item 1493). The table's own length bounds it.
    fn form_num(&mut self) -> i32 {
        self.sim
            .gstate(self.g)
            .map_or(0, |st| st.form_num.max(0).min(st.off.len() as i32))
    }
    fn slot_offset(&mut self, i: usize) -> (i32, i32) {
        self.sim.gstate(self.g).map_or((0, 0), |st| st.off[i])
    }
    fn set_curr(&mut self, i: usize, p: Pos) {
        if let Some(st) = self.sim.gstate_mut(self.g) {
            if st.curr.len() <= i {
                st.curr.resize(i + 1, Pos::new(0, 0));
            }
            st.curr[i] = p;
        }
    }
}
