//! The object-number allocator and the unit store on the `Sim`: adding a unit
//! or a type, `find_free`, the lookups by object number, a figure's init and
//! set-type, and the muster's counts (`docs/STRUCTURE.md` §3, `units/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// `Object::insert_inside@00647e90` (`delta` 1) and
    /// `remove_from_inside@006480f0` (−1): a captain going into or out of a
    /// building moves one of the leader's garrison counts — a peasant, else
    /// a Stable type (`+0x40 == 0x1ac`), else `is_special` (the scout bit),
    /// else a Barracks type (`0x1ab`) — and with `LAKOTA_FOOD` set a Lakota
    /// leader's `0x2000000` (`docs/AI.md` §219), with
    /// `AMERICANS_BARRACKS_GATHER` set an American's for the Barracks arm
    /// (§292).
    ///
    /// The −1 is [`remove_inside_view::remove_from_inside_steps`] itself, the
    /// whole function swept under the emulator (item 3372); the +1 is its
    /// tail alone, [`remove_inside_view::garrison_tail`] behind the captain
    /// test — `insert_inside` is not swept.
    ///
    /// **A decoy moves no count** (item 3964, `docs/AI.md` §408): both
    /// functions test the unit record's `unit_masks & 1` (`+0x68`) ahead of
    /// the captain test — `insert_inside@00647e90`'s `(piVar2[0x1a] & 1) ==
    /// 0`, `remove_from_inside`'s the same — so a French decoy garrisoned
    /// counts as nothing.
    ///
    /// SEAM: the container's slot `0x20` is not modelled, nor
    /// `insert_inside`'s `in_a_building` — every caller here puts the unit in
    /// a building.
    pub(crate) fn track_garrison(&mut self, u: usize, delta: i32) {
        use remove_inside_view::{RemoveInsideView, SimRemoveInside};
        let mut view = SimRemoveInside { sim: self, unit: u };
        if delta < 0 {
            remove_inside_view::remove_from_inside_steps(&mut view);
        } else if !view.masked() && view.is_captain() {
            remove_inside_view::garrison_tail(&mut view, delta);
        }
    }

    /// Adds a unit, registering it as a supply source if its type is one.
    ///
    /// This is `Unit::init`'s half of the supply bookkeeping and the reason to
    /// prefer it over pushing onto `units` directly: a supply wagon that never
    /// registered supplies nobody, silently.
    pub fn add_unit(&mut self, unit: Unit) -> usize {
        // `Unit::init` preserves an existing guy's point until seating it;
        // the collision move uses the new type's radius (COLLISION §23).
        // Only the latest occupant of the object number owns that figure.
        let previous_body = self
            .units
            .iter()
            .rev()
            .find(|old| old.owner == unit.owner && old.index == unit.index)
            .filter(|old| !old.alive() && !old.guys.is_empty())
            .map(|old| old.movement.body.pos);
        // **`path_recursion` (`UnitData +0xaf`) survives the slot**: only
        // `UnitData::UnitData@00606670` zeroes it, and `Unit::init@00612100`
        // never writes it, so a recycled object number keeps its last
        // occupant's count until the next `find_path`. French East Indies'
        // Hoplite captain `1/77`, born on 10765 into the slot a dead `1/77`
        // left, reads 1 on run630's 10766 (item 1458, `docs/GROUPS.md` §37.2).
        let mut unit = unit;
        if let Some(old) = self
            .units
            .iter()
            .rev()
            .find(|old| old.owner == unit.owner && old.index == unit.index && !old.alive())
        {
            unit.path_recursion = old.path_recursion;
        }
        let i = self.units.len();
        let owner = unit.owner as usize;
        let source = unit.kind.supply_unit;
        // A unit handed in with its number already chosen (the harness reads
        // `o` from the dump) moves the mark past it, so a unit allocated next
        // does not collide with one the original numbered higher.
        let mark = &mut self.roster.marks[owner].unit;
        *mark = (*mark).max(unit.index.saturating_add(1));
        self.units.push(unit);
        // `Unit::init`'s transport clause (`docs/TRANSPORT.md` §3.4).
        self.transport_init_unit(i);
        // `Unit::init@00612100:376` — a type that packs is **born packed**:
        // `unit_flags2 & 4` sets `unit_masks |= 0x80000` there and nothing
        // else in the function touches the bit. It is what makes a freshly
        // trained Fisherman skip `think_fish`'s 1,024-frame head and search
        // on its first idle frame (`docs/ORDERS.md` §6.8).
        if self.units[i]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].combat.packs)
        {
            self.units[i].combat.packed = true;
        }
        // **`UnitData::get_combat_stance@00610a30` answers
        // `COMBAT_STAND_GROUND` for a type with no combat stances**
        // ([`crate::stance::get_stance`]): the `+0xb1` byte is the packer's,
        // the worker's or the caster's own, and every reader of vslot
        // `+0xf4` sees 2. This crate keeps the combat half apart as
        // [`combat::Stance`], so it is written here once. run842's packet
        // (item 2265, `docs/GOLDEN.md` §82): the Catapult `0/9`'s idle
        // search on tick 776 runs `find_nearby_target(0xc00, …)` — no
        // AGGRESSIVE `+0x180` — and never reaches the Small City at 2781.
        // The predicate is `UnitTypeData::get_stance_type@0061d350`'s
        // `Combat` arm: `role & MILITARY` and not `unit_flags2 & 4`. The
        // loader fills the profile's `combat_role` and `packs` from those
        // two columns, and a hand-built type may set either spelling.
        if self.units[i].ty.is_some_and(|t| {
            let ty = &self.tables.unit_types[t];
            let military = ty.combat.combat_role || ty.cols.is(crate::ai_load::role::ROLE_MILITARY);
            let packs = ty.combat.packs || ty.cols.flag2(crate::ai_load::uflags2::PACKS);
            !military || packs
        }) {
            self.units[i].combat.stance = crate::combat::Stance::StandGround;
        }
        if source {
            self.units[i].supply_slot = Some(self.territory.supply[owner].list.register(i));
        }
        // `Unit::init@00612100:436` — a **land** caravan takes a slot in its
        // leader's `Caravans` list. `is_caravan` is `unit_flags2 & 8` and
        // the domain test is `UnitTypeData +0x218 == 0`, which is what
        // keeps the sea-domain Merchant Fleet out (`crate::caravan` §2).
        if self.units[i].ty.is_some_and(|ty| {
            self.tables.unit_types[ty]
                .cols
                .flag2(crate::ai_load::uflags2::FLAG2_CARAVAN)
                && self.tables.unit_types[ty].combat.domain == attrition::Domain::Land
        }) {
            let who = self.units[i].owner;
            self.units[i].caravan = self.init_caravan(who, i);
        }
        // `Unit::init@00612100`'s own `update_los` (vtable `+0x160`), on the
        // finished unit — packed bit and all — before `add_to_world` reads
        // it for the disc (`docs/VISION.md` §2).
        self.update_los(i);
        // `Object::add_to_world`: both collision indices
        // (`docs/COLLISION.md` §2, §3) **and the vision disc**.
        if let Some(from) = previous_body {
            self.coll_rebirth(i, from);
        } else {
            self.coll_add(i);
        }
        if self.units[i].alive() && self.units[i].on_map {
            self.chain_add(i);
            // `update_seen(0)`, the whole disc rather than the ring
            // (`docs/VISION.md` §6). This was long credited to
            // `add_to_world`; item 3361 read it as `Unit::init`'s own call,
            // at the trainer's snapped centre — the same disc, so the same
            // effect — and [`Sim::come_out_place`]'s inside arm lights
            // none of its own. A unit *born* on the map once made none, so its owner's line of sight did not exist
            // until the unit first crossed a half-cell or the hundredth-
            // frame resync came round. Nothing read the plane closely
            // enough to notice until [`Sim::target_is_seen`]
            // (`docs/COMBAT.md` §31.4).
            self.update_seen(i, false);
        }
        i
    }

    /// `Objects::find_free(who, base, limit, &mark, −1)`: the object number a
    /// new object of `who` takes. The band `[base, mark)` is scanned first for
    /// a number whose object is dead (`flags & 1` clear) and holds nothing
    /// (`+0x32 == 0`) — the first such is reused; failing that the mark
    /// itself, unless it has reached `limit`, in which case there is no room
    /// and the original's callers give up. A number below the mark that no
    /// object of ours carries at all counts as free too: the only way the sim
    /// gets one is a dump that omitted it, and a dump omits the dead.
    pub fn find_free(&mut self, who: Player, base: i16, limit: i16) -> Option<i16> {
        let w = who as usize;
        let mark = if base == BUILD_BASE {
            self.roster.marks[w].build
        } else {
            self.roster.marks[w].unit
        };
        // A number nothing carries is free; one an object holds is not.
        let mut free = vec![true; (mark - base).max(0) as usize];
        if base == BUILD_BASE {
            for b in &self.buildings {
                let i = b.index - base;
                if b.owner == who && (base..mark).contains(&b.index) {
                    free[i as usize] &=
                        find_free_reusable(b.alive, i16::from(b.hold_frames != 0), false, true);
                }
            }
        } else {
            for u in &self.units {
                let i = u.index - base;
                if u.owner == who && (base..mark).contains(&u.index) {
                    // The reuse test's captain term, `UnitData +0x8e`
                    // (`o_up`) negative (`0065adde`): a dead tail still
                    // linked under a live squad is held. 2400 read it so and
                    // the third pair fell (parked 2441) because the squad's
                    // last death left its dead tail linked here;
                    // [`Sim::relink_squad`]'s whole-squad arm clears it, as
                    // `Unit::close` does (`docs/AI.md` §324.2).
                    free[i as usize] &= find_free_reusable(
                        u.alive(),
                        i16::from(u.hold_frames != 0),
                        true,
                        u.o_up.is_none(),
                    );
                }
            }
        }
        let mut m = i32::from(mark);
        let got = find_free_steps(i32::from(base), i32::from(limit), &mut m, |i| {
            free[(i - i32::from(base)) as usize]
        });
        if base == BUILD_BASE {
            self.roster.marks[w].build = m as i16;
        } else {
            self.roster.marks[w].unit = m as i16;
        }
        (got >= 0).then_some(got as i16)
    }

    /// The live unit of `who` numbered `o`, if any — the object a script or
    /// a producer holds by handle.
    pub fn unit_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.units
            .iter()
            .position(|u| u.owner == who && u.index == o && u.alive())
    }

    /// `who`'s unit slots in the original's list order — `units[who][o]`
    /// walked from 0, which is object number, not this crate's slot order
    /// (a reused number sits at the end of `self.units`). Dead units are
    /// kept; a caller filters on `alive()` as the original's `vtable+8`
    /// does.
    pub(crate) fn units_by_o(&self, who: Player) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].owner == who)
            .collect();
        v.sort_by_key(|&u| self.units[u].index);
        v
    }

    /// The live building of `who` numbered `o`, if any.
    pub fn building_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.buildings
            .iter()
            .position(|b| b.owner == who && b.index == o && b.alive)
    }

    /// Whether building `b`'s object number has gone to a later building —
    /// the only way the original's `uid` test on it can fail, because a
    /// closed building keeps its uid until `Objects::find_free` reuses the
    /// number. A handle that was never valid counts as reused.
    pub(crate) fn building_slot_reused(&self, b: usize) -> bool {
        let Some(bd) = self.buildings.get(b) else {
            return true;
        };
        self.buildings[b + 1..]
            .iter()
            .any(|x| x.owner == bd.owner && x.index == bd.index)
    }

    /// Registers a unit type and returns its id.
    ///
    /// Every player's muster grows with it, because a type nobody has built is
    /// still a type whose count the ramp reads — as zero, which is what makes
    /// the first one cheap.
    pub fn add_unit_type(&mut self, ty: UnitType) -> usize {
        let id = self.tables.unit_types.len();
        let groups = ty.group.map_or(0, |g| g + 1);
        let tree = ty.tree;
        self.tables.unit_types.push(ty);
        // The combat table grows with the type space, at 100 until a
        // builder fills the new row and column.
        if self.combat.table.width() < self.tables.unit_types.len() {
            self.combat.table = self
                .combat
                .table
                .grown(self.tables.unit_types.len(), self.tables.build_types.len());
        }
        for (who, m) in self.roster.muster.iter_mut().enumerate() {
            m.by_type.push(0);
            m.queued_by_type.push(0);
            // A type in the tree starts with the tree's bit — a free unit
            // the starting position already owns is a train job from the
            // first frame.
            m.researched
                .push(tree.is_some_and(|id| self.tech[who].tech[id]));
            if m.by_group.len() < groups {
                m.by_group.resize(groups, 0);
                m.queued_by_group.resize(groups, 0);
            }
        }
        id
    }

    /// Recomputes every player's population cap from scratch —
    /// `Leader::calc_pop_cap`, which the original also calls wholesale
    /// whenever anything that feeds it changes.
    ///
    /// The peacocks term is read off the live rare mask, as the original's
    /// `testb $0x8, 0x6da6(%esi)` reads it on every call ([`economy::PEACOCKS`]),
    /// so a recompute for any other cause sees the mask as it stands. The
    /// Colossus term is `has_wonder(0x20f)`, asked live on the same call
    /// (`006dc490`'s last statement, item 3326, `docs/AI.md` §266).
    pub fn recompute_pop_caps(&mut self) {
        let peacocks = 1u64 << (economy::PEACOCKS - economy::BASE_RARE);
        let colossus: Vec<bool> = (0..self.roster.muster.len())
            .map(|who| {
                u8::try_from(who)
                    .is_ok_and(|p| self.wonders_held(p) & (1 << tech::wonder::COLOSSUS) != 0)
            })
            .collect();
        for (who, m) in self.roster.muster.iter_mut().enumerate() {
            m.bonuses.peacocks = self
                .ledgers
                .get(who)
                .is_some_and(|l| l.rare & peacocks != 0);
            m.bonuses.colossus = colossus[who];
            // `Leader::calc_pop_cap`'s `has_tribe_bonus(3)` (`:68`–`73`): the
            // flag [`cost::pop_cap`]'s Bantu arm reads, which had no writer
            // until the Bantu word's block 1 printed who=1's cap at 50
            // against this crate's 25 (run978, `docs/AI.md` §276).
            m.bonuses.bantu = self.cities_state.nation.get(who).is_some_and(|n| n.bantu);
            m.cap = cost::pop_cap(&self.tables.tuning, m.military_level, m.limit, &m.bonuses);
        }
    }

    /// `Unit::init@00612100` on one object number — the pass of
    /// [`Sim::init_unit`]'s loop, and what `Unit::repair_damage@0060de10`
    /// calls through vslot `+0x8c` when it regrows a figure (item 3993,
    /// `docs/GOLDEN.md` §186): the unit, its stance, its birth masks, and
    /// `Guy::init_real`'s draw. `pos` is the snapped birth point.
    pub(crate) fn init_figure(&mut self, who: Player, ty: usize, index: i16, pos: Pos) -> usize {
        let mut unit = Unit::new(who, index, pos, self.unit_hits(who, ty));
        unit.kind = self.tables.unit_types[ty].kind;
        unit.ty = Some(ty);
        unit.type_index = self.tables.unit_types[ty].type_index;
        unit.form = init_form(unit.type_index);
        // The stance is a switch on the *type's* stance kind and then,
        // when the trainer shares that kind, the trainer's own byte —
        // `Unit::init@00612100:282–309` and `Build::train@0062f9b0:86`
        // (`crate::stance`, `docs/ORDERS.md` §5.10). A trainer is what
        // [`Sim::build_train`] passes; a free-standing spawn has none.
        unit.stance = self.init_stance(who, ty);
        unit.movement.speed = self.type_speed(who, ty);
        unit.movement.turning = self.turning_for(ty);
        let u = self.add_unit(unit);
        // `Unit::init@00612100:592`: a spy (`is(SPY, 0)`, the lineage)
        // is born with half its pool spent.
        if self.unit_is_spy(u) {
            self.units[u].mana_burn = i16::try_from(self.unit_mana(u) / 2).unwrap_or(0);
        }
        // `Unit::init@00612100:598-611` (item 2165, `docs/GOLDEN.md`
        // §79): a unit of the Bark line (`is(BARK)`, the lineage) takes
        // `unit_masks2 |= 4`, and a computer's (`leader_flags & 4`
        // clear, `is_human` inlined) `Unit::set_stance@00605310(3, 0)`,
        // RAID, over the stance the switch above gave it. run835's
        // `1/8` is born `stance 3`, the human `0/6` and `0/7` at 0;
        // every AI Bark on disk carries both.
        if self.unit_line_is(u, crate::army::BARK) {
            self.units[u].unit_masks2 |= crate::combat::umask2::MASK2_BARK;
            if !self
                .cities_state
                .nation
                .get(who as usize)
                .is_some_and(|n| n.human)
            {
                self.units[u].stance = 3;
                self.units[u].combat.stance = crate::combat::Stance::Raid;
            }
        }
        self.init_iroquois_masks(u);
        // `Unit::init@00612100:613-626` (item 2351, `docs/AI.md` §219):
        // a Lakota unit the food term counts — a citizen (`TypeIndex`
        // `0x32`, `0x33`), a scout (`is(SCOUT, 0)`) or a Stable type
        // (`+0x40 == 0x1ac`) — raises the leader's `0x2000000`.
        if self
            .cities_state
            .nation
            .get(who as usize)
            .is_some_and(|n| n.lakota)
            && self.tables.tuning.lakota_food != 0
            && (matches!(self.tables.unit_types[ty].type_index, 0x32 | 0x33)
                || self.unit_line_is(u, 0x45)
                || self.trainer_ident(ty) == Some(build::Ident::Stable))
        {
            self.economy_changed(who);
        }
        // `Unit::init@00612100:629-641` (item 3384, `docs/AI.md` §292):
        // an American Barracks type (`+0x40 == 0x1ab`) that is not a
        // scout, with `AMERICANS_BARRACKS_GATHER` set, raises it too.
        if self.american_barracks_unit(who, u, ty) {
            self.economy_changed(who);
        }
        // `Unit::init` → `Guy::init_real`: the figure's one draw.
        // (The unit's `ty` stays unset here, as it always has; the
        // piece lookup takes the type directly.)
        self.init_guys(u, Some(ty));
        // **`Unit::init@00612100:549`'s `set_new_location(x, y, 1, 1)`**
        // (item 3804, `docs/GOLDEN.md` §152.3): its guy half is
        // `Guy::set_new_location`'s, so a helicopter's figure takes
        // its first climb at birth ([`Sim::helicopter_climb`], read
        // here off the type, which the unit does not carry yet).
        // run1469's Attack Helicopters fly 30 higher than ours did
        // from their first frame, and their rounds aim at that.
        if self.tables.unit_types[ty].cols.unit_flags & crate::ai_load::uflags::FLAG_HELICOPTER != 0
            && self.tables.unit_types[ty].combat.domain == crate::attrition::Domain::Air
        {
            let ground = self.ground_z(self.units[u].pos);
            let af = &mut self.units[u].airframe;
            af.z += (ground - af.z + 1000).clamp(-30, 30);
            af.last_z = af.z;
        }
        // `Unit::init@00612100:467`: a government patriot's birth is
        // stamped on its leader, and that stamp is what keeps a
        // Senate from training a second one.
        if self.tables.unit_types[ty].tree.is_some_and(|t| {
            self.tables
                .tech_tree
                .kind(t)
                .unit()
                .is_some_and(|x| x.patriot)
        }) {
            self.tech[who as usize].gov_hero_frame = self.frame;
        }
        u
    }

    /// `Objects::init_unit@0065e0c0` — **a squad is `uber_size` units, not
    /// one unit with three figures.**
    ///
    /// The loop runs `UnitTypeData::uber_size` times, and each pass is a
    /// whole `Unit::init` with its own `Guy::init_real` draw. The figures
    /// each unit gets are `crew_size + squad_size`, and `squad_size` is
    /// **written 1 by `UnitType::init` and never written again** — so the
    /// three figures of a Bowmen are three *objects*, threaded
    /// `o_up`/`o_down` as a list (the head's `o_up` is −1 and the tail's
    /// `o_down` is; a member's `o_up` is the member before it, not the
    /// captain — run17's frame 1301 has `6 → 7 → 8` exactly so).
    ///
    /// Only the head is counted: every member with an `o_up` is handed
    /// straight back to `track_unit_type(·, −1, ·)`, so `num_units`,
    /// `control` and the two running totals see one unit for the squad.
    ///
    /// SEAM: the original seats each member with `find_nearby_spot` around
    /// the captain before it returns; here they share the captain's
    /// position until [`Sim::come_out`] or a formation moves them. The
    /// search takes no draw, so the stream does not know the difference.
    pub fn init_unit(&mut self, who: Player, ty: usize, pos: Pos) -> usize {
        // **`Unit::init@00612100:68-73` snaps the birth point for every
        // unit**, whatever its owner or domain (twenty-fourth pass, group
        // 17; A3 row 3): a trained unit at its trainer's whole-tile centre
        // is born 24 south and east of it, which a unit kept inside shows.
        let pos = Pos::new(gaia::init_snap(pos.x), gaia::init_snap(pos.y));
        let n = self.tables.unit_types[ty].combat.uber_size.max(1);
        let mut head = None;
        let mut prev = None;
        for _ in 0..n {
            // `find_free(who, 0, 2000, …)`; a full band (2,000 live units
            // of one player) is not modelled as a refusal here, so the
            // number saturates instead.
            let index = self
                .find_free(who, UNIT_BASE, BUILD_BASE)
                .unwrap_or(i16::MAX);
            let u = self.init_figure(who, ty, index, pos);
            match (head, prev) {
                (None, _) => {
                    head = Some(u);
                    self.track_unit_type(who, ty, 1);
                }
                (Some(h), Some(p)) => {
                    let captain = self.units[h].index;
                    self.units[u].captain = false;
                    self.units[u].combat.captain = i32::from(captain);
                    self.units[u].o_up = Some(p);
                    self.units[p].o_down = Some(u);
                    // **The squad is seated around its captain**
                    // (`Objects::init_unit@0065e0c0`, the block ending at
                    // `Unit::set_new_location(spot, 1, 1)`). Every member
                    // past the first is born on the captain's point and
                    // then moved to a free spot near it — the ring
                    // `[size · 0x30, size · 0x60 + 0xc0]`, step −1 (so an
                    // eighth of the span), the bias angle the unit's own
                    // (`UnitData +0x50`, `0x55555555` at birth), and
                    // `FILTER_NOT_ME` with this unit's own `o`/`who`.
                    //
                    // **The search takes no draw**, so the stream never
                    // knew the difference; the positions did, and they
                    // are what the golden record pins. `add hoplite
                    // who=0 4,40` puts the three at `(888, 7800)`,
                    // `(1032, 7800)`, `(936, 7944)` and `who=1 5,40` at
                    // `(1368, 7992)`, `(1512, 7992)`, `(1416, 8136)` —
                    // six coordinates this reproduces exactly, and the
                    // second captain's is the test that matters: with
                    // the squad stacked on one point the near spots stay
                    // free and the second `add` lands 18 tiles from
                    // where the original puts it (`docs/ANIM.md` §6.3,
                    // `docs/INPUT.md` §11.5).
                    //
                    // The original guards the move with a cell test —
                    // the requested point and the captain's own must
                    // share a `0x30` cell — which cannot fail here,
                    // because this crate's caller places the captain on
                    // the requested point itself. And it runs the same
                    // block for **member 0**, whose `get_captain` is its
                    // own index; the dump says the captain does not
                    // move, so the arm is taken as members 1.. only and
                    // the first member's outcome is an open question
                    // (`docs/ANIM.md` §6.4).
                    let size = self.coll_size(u);
                    let centre = self.units[h].pos;
                    let angle = self.units[u].movement.facing;
                    if let Some(spot) = self.find_nearby_spot(
                        u,
                        centre,
                        size * 0x30,
                        size * 0x60 + 0xc0,
                        -1,
                        angle,
                        None,
                    ) {
                        let before = self.units[u].pos;
                        self.set_new_location(u, spot, true);
                        // **`Unit::set_new_location@005f8d20`'s world-cell
                        // change** brackets the write with `remove_from_world`
                        // and `add_to_world`, which ends in `update_seen(0)`:
                        // the seated member lights the disc at its own spot
                        // (`docs/GOLDEN.md` §72). Only this seating is
                        // built: the same call on every cell crossing moved
                        // the successor pair's first draw parting 3188 → 1387.
                        let (a, b) = (before, self.units[u].pos);
                        if (a.x >> 8, a.y >> 8) != (b.x >> 8, b.y >> 8) {
                            self.update_seen(u, false);
                        }
                    }
                }
                (Some(_), None) => unreachable!("a head is set with its own index"),
            }
            prev = Some(u);
        }
        head.expect("uber_size is at least one")
    }

    /// `Unit::set_type@00612fa0(t, SET_TYPE_NORMAL)`, as far as this
    /// simulation carries state for it.
    ///
    /// The order is the original's: the leader's counters come **off** for
    /// the old type before the swap (`6130d1`'s `track_unit_type(·, −1,
    /// ·)` and the two running totals), the record and the derived stats
    /// change, the guys are re-initialised ([`Sim::reinit_guys_begin`]), and the
    /// counters go **on** for the new one. Only a captain is counted, the
    /// same rule `Objects::init_unit` follows for a squad.
    ///
    /// **Damage carries.** `set_type` never writes `myhits`; the object's
    /// `damage` (`ObjectData +0x24`) is untouched by the swap, so a unit
    /// converted at half health is at the *new* type's hits minus the same
    /// damage. This crate stores the complement, so the subtraction is
    /// explicit.
    ///
    /// The tail's `update_gpiece` is [`Sim::reinit_guys_begin`]' piece write, and
    /// each `init_real` re-pieces its guy before it reads the flags, so a
    /// converted guy's `guy_flags & 8` is the new piece's
    /// ([`crate::anim::Guy::flag_piece`]).
    ///
    /// **No same-type guard** (item 3787, `docs/GOLDEN.md` §150.2): the
    /// original's `set_type` re-initialises the guys whatever the old type
    /// was, so a Senate's second government `set_type`s a standing Despot
    /// to The Despot and spends its three `init_real` draws — who=1's `1/30`
    /// on the second melee's frame 828. A caller that must not convert in
    /// place tests for itself, as `gain_tech`'s upgrade walk does.
    ///
    /// SEAM: the `is(0x165, 1)` CEO bit and its `update_ceo_position`, the
    /// `is(0x77, 0)` flag, and the two vslots `+0x15c` and `+0x160` the
    /// tail calls before `update_armor`/`update_speed` — none has state
    /// here.
    pub(crate) fn unit_set_type(&mut self, u: usize, rec: usize) {
        if self.units[u].ty.is_none() {
            return;
        }
        // The steps are `unit_set_type_view::set_type_steps`, the walk the
        // sweep `unit_set_type_normal` holds to the original's bytes; the
        // adapter says what this crate keeps of each.
        let mut view = set_type_adapter::SimSetType::new(self, u, rec);
        unit_set_type_view::set_type_steps(
            &mut view,
            rec as i32,
            unit_set_type_view::SET_TYPE_NORMAL,
        );
    }

    /// **`Object::take_damage@00652020`'s remainder test** (decompile lines 311-329,
    /// the `+0x308` block): a figure of a squad type takes `hits − (uber − 1)
    /// · hits / uber` — the remainder — only when it is a captain (vslot
    /// `+0xe8`) **and** `UnitData::curr_uber_size(this, 1)` answers 1, the
    /// last figure standing; every other figure takes `hits / uber`. The
    /// count is live: this crate read its stored [`Unit::squad_size`], which
    /// nothing outside the tests writes past 1, so every captain of a whole
    /// squad took the remainder and outlived its members by a point (item
    /// 3618, `docs/AI.md` §326.2: Great Sahara's six Elite Javelineers
    /// squads on 4442, `1/200` falling at 37 where theirs falls at 36).
    pub(crate) fn takes_remainder_share(&self, i: usize) -> bool {
        let u = &self.units[i];
        u.combat.captain == i32::from(u.index) && self.curr_uber_size(i) == 1
    }

    /// The squad a captain heads, captain first — `o_down` walked.
    /// `UnitData::curr_uber_size`: the figures in `i`'s squad now — up
    /// the `o_up` links to the captain, then the captain's chain. A dead
    /// figure is **not** relinked out: `Unit::close` re-appends it at the
    /// chain's tail, and the walk stops at it because `0060a760` follows
    /// `o_down` only while the next figure's `flags & 1` holds, the bit
    /// `Object::close@00647160` clears (item 3612: run1254b's `1/46` dies
    /// on 3225 and `1/47` takes the two-figure share on the same block).
    pub fn curr_uber_size(&self, i: usize) -> i32 {
        squad_size(
            i,
            |u| self.units[u].o_up,
            |u| self.units[u].o_down.filter(|&d| self.units[d].alive()),
        )
    }

    pub fn squad_members(&self, captain: usize) -> Vec<usize> {
        let mut out = vec![captain];
        let mut cur = self.units[captain].o_down;
        let mut guard = 0;
        while let Some(u) = cur {
            out.push(u);
            cur = self.units[u].o_down;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        out
    }

    /// **The gate both of the count's per-object writers share**
    /// (`docs/TRANSPORT.md` §16): `Unit::set_type@00612fa0`'s `+1` block
    /// on a birth and `Unit::close@0060ee50`'s `−1` at `0060f3db` on a
    /// close each move `num_units`, `control` and `active` only for a
    /// unit that is no squad follower (`is_captain`, `+0x8e`) and whose
    /// type has population (`+0x2f0 control_cost`) — the `pop == 0` arm
    /// admits `is(0x134)` and a governor-hero alone — with `+0x68 & 1`
    /// clear. So a Merchant Fleet (`POP` 1) is counted and a Transport
    /// Barge (`POP` 0) is not: run462's block 10178 holds the barge `1/78`
    /// and the fleet `1/83`, and its `num_units` has `[268]` 1 and no
    /// `[270]`.
    ///
    /// The governor-hero test is [`Sim::type_is_hero`] (item 3794).
    ///
    /// SEAM: the `pop == 0` arm's `is(0x134)` is not read here, and answers
    /// no; `+0x68 & 1` has no state here.
    pub(crate) fn counts_in_muster(&self, u: usize) -> bool {
        let unit = &self.units[u];
        unit.captain
            && unit
                .ty
                .is_some_and(|t| self.tables.unit_types[t].price.pop != 0 || self.type_is_hero(t))
    }

    /// **`UnitData::is_gov_hero@0046ce70`** on a unit of type `ty`: the
    /// type's `unit_flags & 0x4000000` (`+0x2b4`), the hero digit `1` of its
    /// `OBJ_MASK` — the six government patriots and a scenario's heroes
    /// (`docs/GOLDEN.md` §149.11, §151.1).
    pub(crate) fn type_is_hero(&self, ty: usize) -> bool {
        self.tables.unit_types[ty]
            .tree
            .and_then(|t| self.tables.tech_tree.types.get(t))
            .and_then(|d| d.kind.unit())
            .is_some_and(|x| x.hero)
    }

    /// `Leader::track_unit_type`, and the two lines every one of its call
    /// sites carries beside it: `control` and **`active`**.
    ///
    /// `active` is not a census output. `Unit::set_type@00612fa0:74,284`
    /// moves `num_units`, `control` and `active` in one guarded block in
    /// each direction, and `Objects::init_unit@0065e0c0:92,174` undoes all
    /// three when the new unit turns out to be a squad follower; the
    /// sweep's own recount (`Leader::plan_strategy@006b9620:127,508`) only
    /// re-derives every two hundred frames what these writers have been
    /// keeping live. Modelling the recount alone left `active` a stale
    /// snapshot — 31 against the original's 32 on 62 of run91's 86 blocks,
    /// item 303. `docs/AI.md` §37.
    ///
    /// Keeping the four on one statement is the same reasoning as
    /// [`Sim::track_tree_queued`]: they agree by construction rather than
    /// by inspection.
    pub fn track_unit_type(&mut self, who: Player, ty: usize, delta: i32) {
        let pop = self.tables.unit_types[ty].price.pop;
        let group = self.tables.unit_types[ty].group;
        let muster = &mut self.roster.muster[who as usize];
        muster.by_type[ty] += delta;
        muster.control += pop * delta;
        if let Some(g) = group {
            muster.by_group[g] += delta;
        }
        self.ai[who as usize].census.active += delta;
        self.track_civilian_type(who, ty, delta);
    }

    /// The live half of the census's civilian counts (item 1398,
    /// `docs/AI.md` §103): `Leader::track_unit_type@006e0dd0` moves
    /// `peasants` (`+0x978`) and `scholars` (`+0x97c`) by the type's
    /// predicate, and `Unit::set_type@00612fa0` and `Unit::close@0060ee50`
    /// move `caras` (`+0x980`), so a unit born or closed between two
    /// sweeps is already in the count `create_units` and `make_stuff` read.
    /// The sweep zeroes and recounts them, as the original's does.
    pub(crate) fn track_civilian_type(&mut self, who: Player, ty: usize, delta: i32) {
        let worker = self.tables.unit_types[ty].worker;
        // The live writer uses is_caravan (including Merchant Fleets),
        // while the periodic census counts the land CARA lineage alone.
        // A sweep can drop a fleet from caras before its close subtracts
        // one. docs/AI.md §177 holds this observed asymmetry.
        let caravan = self.tables.unit_types[ty]
            .cols
            .flag2(crate::ai_load::uflags2::FLAG2_CARAVAN);
        // `scouts` (`+0x988`) is the third arm of the same chain —
        // `role & 0x10` on a type neither peasant nor scholar — and no
        // sweep recounts it (`docs/AI.md` §219).
        let scout = self.tables.unit_types[ty]
            .cols
            .is(crate::ai_load::role::ROLE_SCOUT);
        let census = &mut self.ai[who as usize].census;
        match worker {
            crate::orders::Worker::Citizen => census.peasants += delta,
            crate::orders::Worker::Scholar => census.scholars += delta,
            crate::orders::Worker::None if caravan => census.caras += delta,
            crate::orders::Worker::None => {}
        }
        if scout && matches!(worker, crate::orders::Worker::None) {
            census.scouts += delta;
        }
    }
}
