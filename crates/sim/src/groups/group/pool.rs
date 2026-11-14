//! The pool and its seats: `Groups::push_group`, the open slot, `Group::kill`, `normalize`, `sort` and `get_num`, and the building groups (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // Building a group (§3, §4)
    // ------------------------------------------------------------------

    /// **`CommandPackage::process_group@0094a0c0`'s write of
    /// `UnitData::play`** (`docs/ORDERS.md` §8.1, item 4113): for each
    /// object a player's `group` command lists that is alive, the object is
    /// recorded with its `uid` as the selection a later `num = 0` command
    /// repeats; and when it is a unit, `play` is written on it and down its
    /// `o_down` chain — every figure behind a captain. A unit number at or
    /// past the band's mark is the original's error report, and is skipped.
    /// Returns the recorded `(o, uid)` list (`last_objects_received`,
    /// `last_uids_received`).
    pub(crate) fn process_group_play(
        &mut self,
        who: Player,
        play: i8,
        objects: &[i16],
    ) -> Vec<(i16, u16)> {
        let mut received = Vec::new();
        for &o in objects {
            if o >= crate::units::unit::BUILD_BASE {
                if let Some(b) = self
                    .buildings
                    .iter()
                    .position(|b| b.owner == who && b.index == o && b.alive)
                {
                    received.push((o, self.buildings[b].uid));
                }
                continue;
            }
            if let Some(u) = self.unit_by_o(who, o) {
                received.push((o, self.units[u].uid));
                self.write_play_down(u, play);
            }
        }
        received
    }

    /// `process_group`'s `num = 0` arm: the last list it recorded for this
    /// player, each object still alive **with the uid it had** re-added,
    /// and a unit's `play` written down its chain again.
    pub(crate) fn process_group_play_repeat(
        &mut self,
        who: Player,
        play: i8,
        received: &[(i16, u16)],
    ) {
        for &(o, uid) in received {
            if o >= crate::units::unit::BUILD_BASE {
                continue;
            }
            if let Some(u) = self.unit_by_o(who, o)
                && self.units[u].uid == uid
            {
                self.write_play_down(u, play);
            }
        }
    }

    /// `+0xb6 = play` on `u` and on each unit down its `o_down` chain.
    fn write_play_down(&mut self, u: usize, play: i8) {
        let mut at = Some(u);
        let mut steps = 0;
        while let Some(x) = at {
            self.units[x].play = play;
            at = self.units[x].o_down;
            steps += 1;
            if steps > self.units.len() {
                break;
            }
        }
    }

    /// The army's one group (`docs/ARMY.md` §3.2).
    pub fn army_group(&self, who: Player, slot: usize) -> Group {
        Group {
            who,
            army: Some(slot),
            pushed: None,
            list: self.ai.armies[who as usize].list[slot].units.clone(),
        }
    }

    /// One installed [`Pushed`] slot as a [`Group`].
    pub(crate) fn pushed_group_at(&self, slot: usize) -> Option<Group> {
        let x = self.orders.pushed.get(slot)?;
        Some(Group {
            who: x.who,
            army: None,
            pushed: Some(slot),
            list: x.list.clone(),
        })
    }

    /// The record half of a seated group — the army's, or the pool
    /// slot's. `None` is a group with no seat at all: a stack group that
    /// was never pushed, or a player's selection, which is the original's
    /// `UnitData::group == -1`.
    pub(crate) fn gstate(&self, g: &Group) -> Option<&GroupState> {
        match (g.army, g.pushed) {
            (Some(s), _) => Some(&self.ai.armies[g.who as usize].list[s].group),
            (None, Some(i)) => self.orders.pushed.get(i).map(|x| &x.state),
            (None, None) => None,
        }
    }

    /// [`Self::gstate`], to write.
    pub(crate) fn gstate_mut(&mut self, g: &Group) -> Option<&mut GroupState> {
        match (g.army, g.pushed) {
            (Some(s), _) => Some(&mut self.ai.armies[g.who as usize].list[s].group),
            (None, Some(i)) => self.orders.pushed.get_mut(i).map(|x| &mut x.state),
            (None, None) => None,
        }
    }

    /// `Group::add(o, who, 0, 0)` (§4.1) — the entry point every caller in
    /// the simulation uses. A non-captain is refused and **its captain is
    /// added instead**, so asking for a figure gets you its squad.
    pub fn group_add(&self, g: &mut Group, u: usize) {
        self.group_add_keeping(g, u, false);
    }

    /// `Group::add(o, who, keep_captain, const)` (§4.1) whole.
    ///
    /// `keep_captain` is the original's `param_3`, and it flips the walk:
    /// with 0 a **non-captain** is replaced by its captain, and with 1 a
    /// **captain** is first killed out of the group so that re-adding it
    /// moves it to the end. The one caller that passes 1 is this function
    /// itself, on the subordinate chain — which is why a group built from
    /// captains still holds every figure, in `captain, o_down, o_down's
    /// o_down` order. A player's selection is exactly that, and it is why
    /// run31's twelve squads are **36** members and not 12
    /// (`docs/GROUPS.md` §12.2).
    pub fn group_add_keeping(&self, g: &mut Group, u: usize, keep_captain: bool) {
        if !self.units[u].alive() {
            return;
        }
        if self.units[u].owner != g.who && !g.list.is_empty() {
            return;
        }
        if !keep_captain && !self.units[u].captain {
            // The tail call: `add(unit +0x8e, who, 0, const)`.
            if let Some(cap) = self.units[u].o_up {
                self.group_add_keeping(g, cap, false);
            }
            return;
        }
        if keep_captain && self.units[u].captain {
            // `kill(o, who, 0, 0)` first, so the re-add appends.
            g.list.retain(|&m| m != u);
        }
        if g.list.contains(&u) || g.list.len() >= 128 {
            return;
        }
        g.who = self.units[u].owner;
        g.list.push(u);
        // `+0x90 >= 0` and active: the next figure joins behind its captain,
        // and with `keep_captain = 1` so that it is not swapped back for one.
        if let Some(sub) = self.units[u].o_down
            && self.units[sub].alive()
        {
            self.group_add_keeping(g, sub, true);
        }
    }

    /// `Groups::push_group(who, g, force)` (§3.2), two rules of which are
    /// live without a pool: with `force == 0` a group of fewer than two is
    /// **not** installed and every member is left group-less; and an
    /// installed group **takes its members out of the group they were
    /// in** (§3.3). Returns whether the group took a slot.
    pub fn push_group(&mut self, g: &mut Group, force: bool) -> bool {
        // The original's own steps, [`push_group_steps`] (swept under the
        // emulator, `sweep::groups_push_group`): the gate's arm writes
        // `+0x80 = −1` on every active member and touches no list — the unit
        // stays wherever it was listed, naming nothing (§23); past it the
        // slot is chosen and every active member pointed at it.
        let n = g.num();
        let mut cx = (self, g);
        let s = push_group_steps(
            &mut cx,
            n,
            force,
            |(s, g), k| s.units[g.list[k]].alive(),
            |(s, g)| i32::from(s.push_group_slot(g)),
            |(s, g), k| s.units[g.list[k]].group_ptr.map_or(-1, i16::from),
            // The port asks each member's named group before the copy, in
            // `unseat_group` ([`Sim::kill_from_named`], its whole squad),
            // where the original asks it here, after; so this walk's kill
            // is already done (and on the equal arm, none is owed).
            |_, _, _| {},
            |(s, g), k, slot| {
                s.units[g.list[k]].group_ptr = u8::try_from(slot).ok();
            },
        );
        s >= 0
    }

    /// `push_group`'s slot (`70fa86`..`70fac6`): `equals_group` against the
    /// last pushed slot, else `get_open_slot` and `copy_group` into it —
    /// with the port's own bookkeeping of the record it lands in.
    pub(super) fn push_group_slot(&mut self, g: &mut Group) -> u8 {
        let last = self.orders.last_group[g.who as usize];
        // `Group::equals_group@00708000` opens by normalizing each side
        // whose `id` is not −1 — so the last pushed slot is pruned and its
        // speed reset to its leader's (`normalize`'s tail) on every push,
        // equal or not. French East Indies' go-to push on 10766 reads
        // slot 71's speed 28 on run630's 10767, 0 here before item 1457
        // (`docs/GROUPS.md` §36.2).
        if let Some(seat) = self.pool_seat(g.who, last) {
            self.seat_normalize(seat);
        }
        let pool = self.pool_slot_for(g.who, &g.list);
        // `70fa3b`: `equals_group` against the player's **last pushed**
        // slot — the same owner and the same members in the same order —
        // and an equal group is **not copied**: `copy_group` runs only on
        // the fresh slot `get_open_slot` hands back. So a selection pushed
        // again keeps its record, `(ox, oy)`, `o_angle`, `facing` and the
        // slot bytes, which the next layout reads (`docs/GOLDEN.md` §22:
        // run210's `process_group, repeat` on 701 and 741). The second
        // walk then finds every member already pointing at the slot and
        // kills none.
        if pool == last
            && let Some(i) = self
                .orders
                .pushed
                .iter()
                .position(|x| x.who == g.who && x.state.pool == Some(pool) && x.list == g.list)
        {
            g.army = None;
            g.pushed = Some(i);
            return pool;
        }
        // **Parked, twenty-fourth pass, group 15 row 9 (A9 row 9).** The
        // listing has the equal arm skip `get_open_slot` and `copy_group`
        // whoever holds `last_group`'s slot, so an equal push onto an army's
        // group would be that one record. Built, it parted golden chapter 41
        // at 732 from its 1100 (frame 732: ours 6 draws against 5, index 0
        // `Guy::set_anim+0x97a < Unit::move_step+0x823` against
        // `Farms::inc_time+0x1ae`), so it is not built: the capture's push
        // is one an army-backed record does not take.
        self.unseat_group(g, pool);
        // `Groups::copy_group@006fa690` writes the stack group **into the
        // slot's own record** (§28): `who`, `num`, `ox`/`oy`, `o_dist`,
        // `o_angle`, `buildings`, `speed`, the list and its four offset
        // arrays and the angle bytes, and `stamp = game->frame` (run215's
        // slot 1 prints `stamp 621`, `docs/GOLDEN.md` §23). Everything
        // else — `form`, `order_num`, `facing`, `form_num`, `new_speed`,
        // `march` — is the previous occupant's. So the pool index is the
        // record's identity: an entry of [`Sim::pushed`] is the one slot
        // it names, and a push onto that index reuses it rather than any
        // entry whose members happen to be gone. East Indies' tick 20000
        // pushes `1/64`..`1/66` onto the slot army 1's closed group last
        // held, and the layout reads that group's `facing` (§28).
        let (slot, prev) = self.pool_record(g.who, pool);
        let state = GroupState::copied(prev, pool, self.frame, g.list.len());
        let entry = crate::groups::group::Pushed {
            who: g.who,
            list: g.list.clone(),
            state,
            builds: Vec::new(),
        };
        let slot = match slot {
            Some(i) => {
                self.orders.pushed[i] = entry;
                i
            }
            None => {
                self.orders.pushed.push(entry);
                self.orders.pushed.len() - 1
            }
        };
        g.army = None;
        g.pushed = Some(slot);
        // `copy_group` wrote the **army's own record** when the slot is one a valid
        // army owns and has emptied but not recounted (`num_units` still set,
        // list empty — `get_open_slot` hands such a slot out): the army's
        // list now names the push, so the army's tick commands it. Golden
        // chapter ninety-four's Scout `1/0` joins army 0 this way on 846
        // and gets its `ATTACK_TO` to the muster on 1021 (`docs/GOLDEN.md`
        // §120). Only the empty case: the equal arm above is parked row 9's.
        if let Some(a) = self.ai.armies[g.who as usize].list.iter_mut().find(|a| {
            a.valid && a.num_units != 0 && a.group.pool == Some(pool) && a.units.is_empty()
        }) {
            a.units = g.list.clone();
        }
        pool
    }

    /// `push_group`'s second walk (`0070f9e0`), which the first reading
    /// left out: before a member's `+0x80` is pointed at the new slot,
    /// **the group it was in is asked to kill it** — `(*old->vtbl+0x10)
    /// (unit.o, unit.who, 0, 0)`, `Group::kill@00714110` through the
    /// vtable (`vtables.txt`) — whenever that group is not the slot being
    /// written. `Group::kill(o, who, 0, 0)` walks the squad chain exactly
    /// as `Group::add` does (§4.1): a non-captain is replaced by its
    /// captain, and a captain's `o_down` chain goes with it. So the unit
    /// that leaves is always a **whole squad**.
    ///
    /// An army's one group is its [`Army::units`] list (`docs/ARMY.md`
    /// §3.2), so a unit the simulation pushes into a stack group **leaves
    /// the army** — it stops being `Army::member`, it stops being counted
    /// by `Army::normalize`, and every later `Group::action_*` the army
    /// issues goes out without it. That is how Great Lakes' AI army is
    /// nine units and not fifteen from frame 8186 on: §12's probe pushes
    /// its pair, `Group::add` brings both squads, and the six leave
    /// (`docs/ARMY.md` §3.4). The group each member's `+0x80` names is
    /// asked first, and may be cleared ([`Sim::kill_from_named`]).
    pub(super) fn unseat_group(&mut self, g: &Group, pool: u8) {
        let w = g.who as usize;
        let mut touched = self.kill_from_named(g.who, &g.list, pool);
        // `Group::kill`'s own walk: up to the captain, then down `o_down`.
        let leaving = self.leaving_squads(&g.list);
        // A pool slot is a group too, and `(*old->vtbl+0x10)` is asked of
        // whichever one holds the member — so a unit pushed twice leaves
        // the first slot rather than sitting in two groups at once.
        for i in 0..self.orders.pushed.len() {
            self.seat_drop(Seat::Pushed(i), &leaving);
        }
        for slot in 0..self.ai.armies[w].list.len() {
            if self.seat_drop(Seat::Army(g.who, slot), &leaving) && !touched.contains(&slot) {
                touched.push(slot);
            }
        }
        for slot in touched {
            self.army_normalize(g.who, slot);
        }
    }

    /// Every figure of each listed member's squad, `Group::kill`'s own
    /// walk: up to the captain, then down `o_down`.
    pub(super) fn leaving_squads(&self, list: &[usize]) -> Vec<usize> {
        let mut leaving: Vec<usize> = Vec::new();
        for &m in list {
            if !self.units[m].alive() {
                continue;
            }
            for f in self.squad_of(self.captain_of(m)) {
                if !leaving.contains(&f) {
                    leaving.push(f);
                }
            }
        }
        leaving
    }

    /// `push_group`'s second walk (`0070f9e0`), the kill half, **on the
    /// group each member's `+0x80` names** when that is not `pool`, the
    /// slot being written (`docs/GROUPS.md` §30). `Group::kill@00714110`
    /// takes the member's squad out of that record's list and writes its
    /// `stamp = frame` — or, when the list empties, runs `Group::clear(-1)`
    /// on it, so the slot's next occupant inherits `order_num 0`, `facing
    /// 0` and `form −1` rather than the last one's counts. run45's who=1
    /// slot 65 reads exactly that on block 239, when `think_scout` repushes
    /// `1/0` onto 64, and run233's slot 64 on 17157, when `1/60` leaves
    /// army 0's orphan for army 1's fresh group on 71. Returns the army
    /// slots of `who` it took members from, for their `normalize`.
    pub(crate) fn kill_from_named(&mut self, who: Player, list: &[usize], pool: u8) -> Vec<usize> {
        // **Per member, through the seat's own `Group::kill`** (item 1171,
        // `docs/GROUPS.md` §30). `kill@00714110` shifts all six parallel
        // arrays down over the leaving index — `list +0x8cc`, `angles
        // +0x84c`, and `off_x`/`off_y`/`curr_x`/`curr_y` from `+0x4c` in
        // steps of `0x200` — so a member that stays keeps its own offset
        // at its new index. A retain on the list alone left the offsets
        // where they were: Great Sahara's army group 64 lost `1/28`–`1/30`
        // and `1/65`–`1/67` to group 68 on frame 14586, and `1/52`, now
        // listed fifth, walked to the fifth *old* slot, `1/47`'s.
        let frame = self.frame;
        let mut touched = Vec::new();
        for &m in list {
            let Some(s) = self.units[m].group_ptr else {
                continue;
            };
            if !self.units[m].alive() || s == pool {
                continue;
            }
            let Some(seat) = self.pool_seat(who, s) else {
                continue;
            };
            let before = self.seat_list(seat).len();
            self.seat_kill(seat, m, false, false);
            let (l, st) = self.seat_parts(seat);
            if l.len() < before {
                // `num == 0` runs `clear(-1)` where a survivor stamps.
                if l.is_empty() {
                    st.clear(frame);
                }
                if let Seat::Army(_, a) = seat
                    && !touched.contains(&a)
                {
                    touched.push(a);
                }
            }
        }
        touched
    }

    /// Drop every member `leaving` names from a seat's list, shifting the
    /// per-member arrays with it ([`Self::seat_remove_at`]), last to first.
    pub(super) fn seat_drop(&mut self, seat: Seat, leaving: &[usize]) -> bool {
        let (l, st) = self.seat_parts(seat);
        let mut dropped = false;
        for i in (0..l.len()).rev() {
            if leaving.contains(&l[i]) {
                Self::seat_remove_at(l, st, i);
                dropped = true;
            }
        }
        dropped
    }

    // ------------------------------------------------------------------
    // The pool's numbering and its per-frame pass (§19)
    // ------------------------------------------------------------------

    /// A closed army's group, kept in the pool on its own index
    /// (`Army::close@006f8ea0` writes only its `army` to −1; `docs/ARMY.md`
    /// §22). It takes a [`Pushed`] entry whose members are all gone, as
    /// `push_group` does, else a new one; its record and list are the
    /// army's, and [`Sim::groups_process`] prunes it from then on.
    pub(crate) fn seat_orphan(&mut self, who: Player, list: Vec<usize>, state: GroupState) {
        // The entry is the pool index's own record (§28): the orphan
        // replaces whatever stale entry named its index, and never an entry
        // that names another — that entry is another slot's record, which
        // a later `copy_group` into that slot reads.
        let at = self
            .orders
            .pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == state.pool);
        let entry = crate::groups::group::Pushed {
            who,
            list,
            state,
            builds: Vec::new(),
        };
        match at {
            Some(i) => self.orders.pushed[i] = entry,
            None => self.orders.pushed.push(entry),
        }
    }

    /// The record pool slot `s` of `who` holds now — the army whose group
    /// sits on it, else the [`Pushed`] entry that names it — as what
    /// `copy_group` leaves behind (§28), with the entry to overwrite.
    pub(crate) fn pool_record(&self, who: Player, s: u8) -> (Option<usize>, GroupState) {
        let entry = self
            .orders
            .pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == Some(s));
        let army = self.ai.armies.get(who as usize).and_then(|x| {
            x.list
                .iter()
                .find(|a| a.valid && a.group.pool == Some(s))
                .map(|a| a.group.clone())
        });
        let prev = army
            .or_else(|| entry.map(|i| self.orders.pushed[i].state.clone()))
            .unwrap_or_default();
        (entry, prev)
    }

    /// Whether the pool slot a pushed group sits in still names a live army
    /// in its `army` field (`GroupData +0x8`).
    ///
    /// `copy_group@006fa690` never writes that field (§28): the slot keeps
    /// its previous occupant's. When `get_open_slot` hands back the slot of
    /// an army that has lost every member, the army is still `valid` and its
    /// slot still reads `army ≥ 0`, so `Group::action_move_near@00704990`
    /// (`706178`..`70619f`) sets the pathfinder's `army` mode for whoever
    /// was pushed there. Golden chapter ninety-four's Scout `1/0` on 846:
    /// army 0's group 64 empties with the Artillery (824), the Scout's lone
    /// group is pushed onto 64, and its explore search prices every
    /// `NEARBLOCK` cell at `base << 5` (`docs/GOLDEN.md` §119).
    ///
    /// **Only while the army's `num_units` is still non-zero** — the count
    /// the army's own recount (`army_normalize`) clears. Chapter ninety-four's
    /// army 0 still reads 2 on 846 (the Artillery died on 824, no recount
    /// since); chapter sixty-two's same slot on 846 reads 0 and its Scout
    /// keeps the plain search, which a diff pins (`ch62` word frame). That
    /// the recount is what resets the link in the original is an assumption
    /// this discriminator stands on, not a reading (`docs/GOLDEN.md` §119).
    pub(crate) fn slot_keeps_army_link(&self, g: &Group) -> bool {
        let Some(i) = g.pushed else { return false };
        let Some(pool) = self.orders.pushed.get(i).and_then(|e| e.state.pool) else {
            return false;
        };
        self.ai.armies.get(g.who as usize).is_some_and(|x| {
            x.list
                .iter()
                .any(|a| a.valid && a.num_units != 0 && a.group.pool == Some(pool))
        })
    }

    /// The seat whose record pool slot `s` of `who` is — the army whose
    /// group sits on it, else the [`Pushed`] entry that names it.
    pub(super) fn pool_seat(&self, who: Player, s: u8) -> Option<Seat> {
        let w = who as usize;
        if let Some(a) = self.ai.armies.get(w).and_then(|x| {
            x.list
                .iter()
                .position(|a| a.valid && a.group.pool == Some(s))
        }) {
            return Some(Seat::Army(who, a));
        }
        self.orders
            .pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == Some(s))
            .map(Seat::Pushed)
    }

    /// The live members of whichever seat of `who` holds pool slot `s` —
    /// an army's group or a pushed one — in join order.
    ///
    /// "Live" is what `Group::get_num` answers after the `normalize` it
    /// opens with: alive, and still pointing at this slot. A unit that has
    /// joined an army since it was pushed points at the army's slot
    /// (`Unit::set_group@00605220` writes `+0x80` and leaves the old list
    /// alone), so it no longer counts for the slot it left — and since
    /// item 557 the test is the pointer itself ([`Unit::group_ptr`]), for
    /// an army's list as for a pushed one.
    pub(crate) fn pool_members(&self, who: Player, s: u8) -> Vec<usize> {
        let live = |l: &[usize]| -> Vec<usize> {
            l.iter()
                .copied()
                .filter(|&u| self.units[u].alive() && self.units[u].group_ptr == Some(s))
                .collect()
        };
        let w = who as usize;
        self.ai.armies[w]
            .list
            .iter()
            .filter(|a| a.valid && a.group.pool == Some(s))
            .map(|a| live(&a.units))
            .chain(
                self.orders
                    .pushed
                    .iter()
                    .filter(|p| p.who == who && p.state.pool == Some(s))
                    .map(|p| live(&p.list)),
            )
            .find(|l| !l.is_empty())
            .unwrap_or_default()
    }

    /// The seat `u`'s back-pointer names — what the original reaches as
    /// `groups[unit->+0x80]`, and so what every reader of a unit's group
    /// asks (§23).
    ///
    /// The original has one record per pool index; this crate keeps an
    /// army's record on the army and a pushed one in [`Sim::pushed`], and
    /// may hold a stale seat beside a live one on the same index. So the
    /// seat that **lists** `u` is preferred, army first; a unit its own
    /// seat does not list — `1/62` and `1/63` on Great Lakes between
    /// 11424 and 11512 — is answered by the seat on that index that still
    /// has a live member, which is the one the original's record is.
    pub(crate) fn seat_of(&self, u: usize) -> Option<Seat> {
        let s = self.units[u].group_ptr?;
        let who = self.units[u].owner;
        let w = who as usize;
        let armies = || {
            self.ai
                .armies
                .get(w)
                .into_iter()
                .flat_map(|x| x.list.iter().enumerate())
                .filter(|(_, a)| a.valid && a.group.pool == Some(s))
        };
        let pushed = || {
            self.orders
                .pushed
                .iter()
                .enumerate()
                .filter(|(_, p)| p.who == who && p.state.pool == Some(s))
        };
        if let Some((a, _)) = armies().find(|(_, a)| a.units.contains(&u)) {
            return Some(Seat::Army(who, a));
        }
        if let Some((i, _)) = pushed().find(|(_, p)| p.list.contains(&u)) {
            return Some(Seat::Pushed(i));
        }
        let live = |l: &[usize]| l.iter().any(|&m| self.units[m].alive());
        if let Some((a, _)) = armies().find(|(_, a)| live(&a.units)) {
            return Some(Seat::Army(who, a));
        }
        pushed()
            .find(|(_, p)| live(&p.list))
            .map(|(i, _)| Seat::Pushed(i))
    }

    // ------------------------------------------------------------------
    // A seated group's own list, as `Group::add`/`kill`/`normalize`/`sort`
    // leave it (§4, §23)
    // ------------------------------------------------------------------

    /// The seat a [`Group`] value was built from, if it has one.
    pub(crate) fn seat_of_group(g: &Group) -> Option<Seat> {
        match (g.army, g.pushed) {
            (Some(a), _) => Some(Seat::Army(g.who, a)),
            (None, Some(i)) => Some(Seat::Pushed(i)),
            (None, None) => None,
        }
    }

    pub(crate) fn seat_list(&self, seat: Seat) -> &Vec<usize> {
        match seat {
            Seat::Army(w, a) => &self.ai.armies[w as usize].list[a].units,
            Seat::Pushed(i) => &self.orders.pushed[i].list,
        }
    }

    pub(super) fn seat_who(&self, seat: Seat) -> Player {
        match seat {
            Seat::Army(w, _) => w,
            Seat::Pushed(i) => self.orders.pushed[i].who,
        }
    }

    pub(crate) fn seat_parts(&mut self, seat: Seat) -> (&mut Vec<usize>, &mut GroupState) {
        match seat {
            Seat::Army(w, a) => {
                let x = &mut self.ai.armies[w as usize].list[a];
                (&mut x.units, &mut x.group)
            }
            Seat::Pushed(i) => {
                let x = &mut self.orders.pushed[i];
                (&mut x.list, &mut x.state)
            }
        }
    }

    /// The seat as a [`Group`] value, for the queries that take one.
    pub(super) fn seat_group(&self, seat: Seat) -> Group {
        let who = self.seat_who(seat);
        match seat {
            Seat::Army(_, a) => self.army_group(who, a),
            Seat::Pushed(i) => Group {
                who,
                army: None,
                pushed: Some(i),
                list: self.orders.pushed[i].list.clone(),
            },
        }
    }

    /// Drop list index `i` and the per-member arrays' entry with it — the
    /// shift `Group::kill` and `normalize` both do over all five arrays.
    pub(super) fn seat_remove_at(list: &mut Vec<usize>, st: &mut GroupState, i: usize) {
        list.remove(i);
        if i < st.off.len() {
            st.off.remove(i);
        }
        if i < st.curr.len() {
            st.curr.remove(i);
        }
        if i < st.angles.len() {
            st.angles.remove(i);
        }
    }

    /// `speed = new_speed = UnitData::speed(find_leader)`, or 0 — the tail
    /// `compute_speed`, `kill` and `normalize` share (§18.1).
    pub(crate) fn seat_set_speed(&mut self, seat: Seat) {
        let g = self.seat_group(seat);
        let v = self.group_compute_speed(&g);
        let (_, st) = self.seat_parts(seat);
        st.speed = v;
        st.new_speed = v;
    }

    /// `Group::normalize@00711540` on a seated group (§4.3): last to first,
    /// drop a member that is dead or whose `+0x80` does not name this slot
    /// (`priority` is 0 and `id >= 0` for every seat here), then
    /// `find_role` and the speed tail.
    pub(crate) fn seat_normalize(&mut self, seat: Seat) {
        let (list, st) = self.seat_parts(seat);
        let pool = st.pool;
        let list = list.clone();
        for i in (0..list.len()).rev() {
            let u = list[i];
            if !self.units[u].alive() || self.units[u].group_ptr != pool {
                let (l, st) = self.seat_parts(seat);
                Self::seat_remove_at(l, st, i);
            }
        }
        self.seat_normalize_tail(seat);
    }

    /// `Group::normalize@00711540`'s tail on a seated group, after its
    /// prune: `Group::find_role@007081f0` ([`GroupState::role`]), then the
    /// speed pair ([`Sim::seat_set_speed`]).
    pub(crate) fn seat_normalize_tail(&mut self, seat: Seat) {
        let role = self.group_find_role(self.seat_list(seat));
        let (_, st) = self.seat_parts(seat);
        st.role = role;
        self.seat_set_speed(seat);
    }

    /// `Group::get_num` (vslot `+0x4`): with **fewer than four** members
    /// and an `id`, it is `normalize` whole; otherwise only the inactive
    /// are dropped. It is the first thing `Group::add` does, which is why
    /// a squad joining a small group can lose its own head (§23).
    ///
    /// **An empty group is tested first** (`Group::get_num@00714700`:
    /// `num < 1` → `num = 0; oy = 0; ox = 0; return`, twenty-fourth pass,
    /// group 14; A9 row 12): no `normalize`, so its speed pair stands, and
    /// its origin is zeroed.
    pub(super) fn seat_get_num(&mut self, seat: Seat) -> usize {
        if self.seat_list(seat).is_empty() {
            let (_, st) = self.seat_parts(seat);
            st.o = Pos::new(0, 0);
            return 0;
        }
        if self.seat_list(seat).len() < 4 {
            self.seat_normalize(seat);
        } else {
            let list = self.seat_list(seat).clone();
            for i in (0..list.len()).rev() {
                if !self.units[list[i]].alive() {
                    let (l, st) = self.seat_parts(seat);
                    Self::seat_remove_at(l, st, i);
                }
            }
        }
        self.seat_list(seat).len()
    }

    /// `Group::get_num_cap@007145c0` on a seat: an empty group answers zero
    /// untouched; one of fewer than four members is `normalize`d whole, a
    /// larger one only loses its dead; the answer is the captains standing.
    pub(super) fn seat_get_num_cap(&mut self, seat: Seat) -> usize {
        if self.seat_list(seat).is_empty() {
            return 0;
        }
        self.seat_get_num(seat);
        self.seat_list(seat)
            .iter()
            .filter(|&&u| self.units[u].alive() && self.is_captain(u))
            .count()
    } // twin of Group::get_num_cap

    /// `Group::add(o, who, keep_captain, const)@00714350` on a seated group,
    /// whole (§4.1, §23). Unlike [`Sim::group_add_keeping`], which builds a
    /// stack group (`id −1`, never normalized), every step here opens with
    /// `get_num` unless `konst` — so a figure whose `+0x80` does not name
    /// this group is dropped by the very recursion that adds the next one.
    /// It writes no back-pointer.
    pub(crate) fn seat_add(&mut self, seat: Seat, o: usize, keep_captain: bool, konst: bool) {
        if !konst {
            self.seat_get_num(seat);
        }
        if !self.units[o].alive() {
            return;
        }
        if !keep_captain && !self.units[o].captain {
            // The tail call: `add(unit +0x8e, who, 0, const)`.
            if let Some(up) = self.units[o].o_up {
                self.seat_add(seat, up, false, konst);
            }
            return;
        }
        if keep_captain && self.units[o].captain {
            self.seat_kill(seat, o, false, false);
        }
        let frame = self.frame;
        let (list, st) = self.seat_parts(seat);
        if list.contains(&o) || list.len() >= 128 {
            return;
        }
        // The four offset arrays and the angle byte are zeroed at the new
        // index — where the arrays reach it.
        let n = list.len();
        if st.off.len() == n {
            st.off.push((0, 0));
        }
        if st.curr.len() == n {
            st.curr.push(Pos::new(0, 0));
        }
        if st.angles.len() == n {
            st.angles.push(0);
        }
        list.push(o);
        st.stamp = frame;
        if let Some(d) = self.units[o].o_down
            && self.units[d].alive()
        {
            self.seat_add(seat, d, true, konst);
        }
        self.seat_set_speed(seat);
    }

    /// `Group::kill(o, who, keep_captain, const)@00714110` on a seated group
    /// (§4.2): a non-captain kills its captain instead and returns; a
    /// captain's subordinate goes first; then `o` leaves the list, and its
    /// `+0x80` is cleared **only if it names this group**.
    pub(crate) fn seat_kill(&mut self, seat: Seat, o: usize, keep_captain: bool, konst: bool) {
        if !keep_captain && !self.units[o].captain {
            if let Some(up) = self.units[o].o_up {
                self.seat_kill(seat, up, false, konst);
            }
            return;
        }
        if let Some(d) = self.units[o].o_down
            && (konst || self.units[d].alive())
        {
            self.seat_kill(seat, d, true, konst);
        }
        let frame = self.frame;
        let (list, st) = self.seat_parts(seat);
        let Some(i) = list.iter().position(|&m| m == o) else {
            return;
        };
        let pool = st.pool;
        Self::seat_remove_at(list, st, i);
        st.stamp = frame;
        if list.is_empty() {
            st.emptied(frame);
        }
        if self.units[o].group_ptr == pool {
            self.units[o].group_ptr = None;
        }
        self.seat_set_speed(seat);
    }

    /// `Group::sort@00708090` — `Form::categorize`'s first statement, so it
    /// runs on every formation a seated group lays out (§4.1, §23).
    ///
    /// Walk the list keeping the last captain seen; a follower whose own
    /// top captain (`UnitData::get_captain`, up `o_up` to the root) is not
    /// that one — or that comes before any captain — is `kill`ed (which
    /// takes its whole squad out, clearing each `+0x80` that names this
    /// group), the group is `normalize`d, and the follower is re-`add`ed
    /// with `const = 1`, which brings the squad back whole at the end of
    /// the list **and writes no pointer**. Repeat until a pass is clean.
    ///
    /// On Great Lakes 11512 this is the whole of `1/64`'s exit: the army's
    /// list holds `1/64` without its captain (§23), the sort kills and
    /// re-adds the squad, and `1/64` comes back listed and naming nothing.
    ///
    /// The original loops until clean; a re-add the list refuses would
    /// loop it forever, so this stops after 256 passes, twice the most a
    /// list can hold.
    pub(crate) fn seat_sort(&mut self, seat: Seat) {
        for _ in 0..256 {
            let list = self.seat_list(seat).clone();
            let mut last: Option<usize> = None;
            let mut bad = None;
            for &m in &list {
                if self.units[m].captain {
                    last = Some(m);
                    continue;
                }
                match last {
                    Some(l) if self.top_captain(m) == l => {}
                    _ => {
                        bad = Some(m);
                        break;
                    }
                }
            }
            let Some(m) = bad else { return };
            self.seat_kill(seat, m, false, false);
            self.seat_normalize(seat);
            self.seat_add(seat, m, false, true);
        }
    }

    /// `Group::sort@00708090` on a group **on the stack** (§27): the same
    /// walk as [`Self::seat_sort`], over the value's own list. The `kill`
    /// takes the follower's whole squad out and the `add` brings it back
    /// whole at the end.
    ///
    /// SEAM: the stack group carries its parent's `id`, so the original's
    /// `kill` clears `+0x80` on a squad member that names it, and its
    /// `normalize` prunes by that id. Neither is written here. No capture
    /// has a sub-group that needs sorting: every siege arm on file has one
    /// anchor, and one member sorts to itself.
    pub(crate) fn stack_sort(&self, g: &mut Group) {
        for _ in 0..256 {
            let mut last: Option<usize> = None;
            let mut bad = None;
            for &m in &g.list {
                if self.units[m].captain {
                    last = Some(m);
                    continue;
                }
                match last {
                    Some(l) if self.top_captain(m) == l => {}
                    _ => {
                        bad = Some(m);
                        break;
                    }
                }
            }
            let Some(m) = bad else { return };
            let cap = self.top_captain(m);
            let before = g.list.len();
            g.list.retain(|&x| self.top_captain(x) != cap);
            self.group_add(g, m);
            if g.list.len() < before {
                // A re-add the list refused: stop rather than loop.
                return;
            }
        }
    }

    /// `UnitData::get_captain` (vslot `+0xe4`): up `o_up` (`+0x8e`) to the
    /// figure that has none.
    pub(crate) fn top_captain(&self, u: usize) -> usize {
        let mut f = u;
        for _ in 0..self.units.len() {
            match self.units[f].o_up {
                Some(up) => f = up,
                None => break,
            }
        }
        f
    } // twin of UnitData::get_captain

    /// `Unit::set_group(unit, g, 0)@00605220`: up the squad chain to its
    /// captain, then `+0x80 = g` on the captain and every **active**
    /// figure down its `o_down` chain. `Army::add_unit`'s last act, and
    /// the only writer that points a unit **at** an army's group.
    pub(crate) fn set_group(&mut self, u: usize, s: Option<u8>) {
        let units = &self.units;
        let walked = crate::squad_walk(
            u,
            units.len(),
            |i| units[i].o_up,
            |i| units[i].o_down.filter(|&d| units[d].alive()),
        );
        for f in walked {
            self.units[f].group_ptr = s;
        }
    }

    /// The slot `Groups::push_group@0070f9e0` hands a group of `who`, and
    /// the `last_group` it leaves behind (§3.1, §3.2):
    ///
    /// - the group **equals** the seat `last_group[who]` names — same
    ///   members in the same order — and that slot is reused;
    /// - otherwise `Groups::get_open_slot@006fa460`'s first rule: the
    ///   lowest of `0..46` whose seat is empty and which is not
    ///   `last_group[who]`.
    ///
    /// `last_group` starts at each player's slot 0 (`Groups::clear@00713f20`
    /// writes `who·64`), which is why a player's first push lands in slot 1.
    ///
    /// SEAM: `get_open_slot`'s two fallbacks — the oldest one-captain
    /// group, and past it the "UH OH, NEED MORE GROUPS!" pass — are not
    /// modelled; they need 46 live groups at once, and no capture has more
    /// than four. A full pool answers slot 45.
    pub(crate) fn pool_slot_for(&mut self, who: Player, list: &[usize]) -> u8 {
        let w = who as usize;
        let last = self.orders.last_group[w];
        let s = if !list.is_empty() && self.pool_members(who, last) == list {
            last
        } else {
            self.open_slot(who)
        };
        self.orders.last_group[w] = s;
        s
    }

    /// `Groups::get_open_slot@006fa460` (§3.1, §28): the slot a group that
    /// does not equal `last_group` is copied into, with its tail's
    /// eviction. `last_group` is the caller's to write.
    pub(super) fn open_slot(&mut self, who: Player) -> u8 {
        let last = self.orders.last_group[who as usize];
        // **One pass, as `Groups::get_open_slot@006fa460` walks it**
        // (twenty-fourth pass, group 13; A9 row 15): `get_num` on every slot
        // it reaches — each normalises a slot of one to three members — and
        // an open slot (`get_num == 0`, or a building group) that is not
        // `last_group` ends the walk; past a non-open slot the stamp arm
        // calls `get_num_cap`, which normalises it again, while its stamp
        // is no later than the best so far, and takes it when it holds
        // exactly one captain. The last of equal stamps wins; failing all of
        // them the last slot not `last_group`.
        let mut best: Option<u8> = None;
        let mut stamp = self.frame;
        let mut open = None;
        for s in 0..46u8 {
            let seat = self.pool_seat(who, s);
            let n = seat.map_or(0, |seat| self.seat_get_num(seat));
            if (n == 0 || self.pool_is_building_group(who, s)) && s != last {
                open = Some(s);
                break;
            }
            if let Some(seat) = seat
                && self.pool_record(who, s).1.stamp <= stamp
                && self.seat_get_num_cap(seat) == 1
                && s != last
            {
                stamp = self.pool_record(who, s).1.stamp;
                best = Some(s);
            }
        }
        let s = open.or(best).unwrap_or(if last == 45 { 44 } else { 45 });
        // `get_open_slot`'s tail (§3.1): unless the slot holds a
        // building group, every unit of `who` whose `+0x80` names it
        // is cleared before the new group moves in. What this reaches
        // is a pointer its list dropped — the stale half §23 is about.
        if !self.pool_is_building_group(who, s) {
            for x in &mut self.units {
                if x.owner == who && x.group_ptr == Some(s) {
                    x.group_ptr = None;
                }
            }
        }
        s
    }

    /// Does slot `s` of `who` hold a building group now?
    pub(super) fn pool_is_building_group(&self, who: Player, s: u8) -> bool {
        let army = self
            .ai
            .armies
            .get(who as usize)
            .is_some_and(|x| x.list.iter().any(|a| a.valid && a.group.pool == Some(s)));
        !army
            && self
                .orders
                .pushed
                .iter()
                .any(|x| x.who == who && x.state.pool == Some(s) && !x.builds.is_empty())
    }

    /// `Groups::push_group(who, g, 1)` for the one-building group the AI
    /// scripts' `train_unit*` and `research_tech_with_cost` build before
    /// `Group::action_queue_up` (§28.4): [`Sim::push_buildings_group`] of
    /// one building.
    pub(crate) fn push_building_group(&mut self, who: Player, b: usize) {
        self.push_buildings_group(who, &[b]);
    }

    /// `Groups::push_group@0070f9e0(who, g, 1)` for a group of buildings
    /// (`docs/PRODUCTION.md`, "The command on a selection of buildings"):
    /// `list` in the order `Group::add@00714350` took them. A group equal to `last_group`'s record —
    /// `Group::equals_group@00708000`: the same members **in the same
    /// order** — reuses it and writes nothing; otherwise `get_open_slot`
    /// and `copy_group` seat it, and it takes every later push's slot
    /// first. A building's own `+0x80` is not carried: the member walk
    /// writes a unit's alone (vslot 8).
    pub(crate) fn push_buildings_group(&mut self, who: Player, list: &[usize]) {
        let w = who as usize;
        let last = self.orders.last_group[w];
        // `Group::equals_group@00708000` opens by normalizing the last
        // pushed slot (its `id` is never −1), equal or not, as
        // [`Sim::push_group_slot`] does for a group of units: so a building
        // push prunes the slot and writes its `role` and speed pair. The
        // minted game's `queue_up` on 60 pushes building 2000 while
        // `last_group` is the Scout's slot 1, which reads `role` 0x40110 and
        // speed 34 from 61 in the original (run1580; item 4222), where this
        // crate waited for `Groups::process`' turn on 65.
        if let Some(seat) = self.pool_seat(who, last) {
            self.seat_normalize(seat);
        }
        let equal = self.orders.pushed.iter().any(|x| {
            x.who == who && x.state.pool == Some(last) && x.builds == list && x.list.is_empty()
        }) && !self.ai.armies[w]
            .list
            .iter()
            .any(|a| a.valid && a.group.pool == Some(last));
        if equal {
            return;
        }
        let pool = self.open_slot(who);
        self.orders.last_group[w] = pool;
        let (slot, prev) = self.pool_record(who, pool);
        let state = GroupState::copied(prev, pool, self.frame, list.len());
        let entry = crate::groups::group::Pushed {
            who,
            list: Vec::new(),
            state,
            builds: list.to_vec(),
        };
        match slot {
            Some(i) => self.orders.pushed[i] = entry,
            None => self.orders.pushed.push(entry),
        }
    }

    /// The pool slot `u` points at, as the dump prints it — `who·64 + s`,
    /// or −1 for a unit in no group. For a probe and the diff harness.
    pub fn pool_group_of(&self, u: usize) -> i64 {
        let who = i64::from(self.units[u].owner);
        self.units[u]
            .group_ptr
            .map_or(-1, |s| who * 64 + i64::from(s))
    }

    /// The record half of `who`'s pool slot `s` — an army's or a pushed
    /// group's, whichever [`Sim::pool_list`] would list — for a widening
    /// that compares the dump's `GROUPDATA` whole (`docs/GOLDEN.md` §23).
    pub fn pool_state(&self, who: Player, s: u8) -> Option<&GroupState> {
        let w = who as usize;
        let army = self
            .ai
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .find(|a| a.valid && a.group.pool == Some(s))
            .map(|a| &a.group);
        army.or_else(|| {
            self.orders
                .pushed
                .iter()
                .find(|p| p.who == who && p.state.pool == Some(s) && !p.list.is_empty())
                .map(|p| &p.state)
        })
    }

    /// Every record this crate holds on `who`'s pool slot `s` — an army's
    /// group first, then each [`Pushed`] entry naming it, stale or not —
    /// with its members as object numbers. The original has one record per
    /// index; this crate can hold a closed army's orphan and a later push
    /// on the same one (`docs/GROUPS.md` §28). For a widening.
    pub fn pool_records(&self, who: Player, s: u8) -> Vec<(&GroupState, Vec<i16>)> {
        let names = |l: &[usize]| l.iter().map(|&u| self.units[u].index).collect();
        let w = who as usize;
        let armies = self
            .ai
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .filter(|a| a.valid && a.group.pool == Some(s))
            .map(|a| (&a.group, names(&a.units)));
        let pushed = self
            .orders
            .pushed
            .iter()
            .filter(|p| p.who == who && p.state.pool == Some(s))
            .map(|p| (&p.state, names(&p.list)));
        armies.chain(pushed).collect()
    }

    /// The member list of the seat on `who`'s pool slot `s`, as object
    /// numbers in list order — what the dump's `GROUPDATA` prints under
    /// `id who·64 + s`. Empty for a slot nothing holds. For the diff
    /// harness; nothing in the simulation calls it.
    pub fn pool_list(&self, who: Player, s: u8) -> Vec<i16> {
        let w = who as usize;
        let army = self
            .ai
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .find(|a| a.valid && a.group.pool == Some(s))
            .map(|a| a.units.clone());
        let list = army.or_else(|| {
            self.orders
                .pushed
                .iter()
                .find(|p| p.who == who && p.state.pool == Some(s) && !p.list.is_empty())
                .map(|p| p.list.clone())
        });
        list.unwrap_or_default()
            .into_iter()
            .map(|u| self.units[u].index)
            .collect()
    }
}
