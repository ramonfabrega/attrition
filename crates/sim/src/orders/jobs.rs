//! The jobs: swarm, build, repair, garrison and gather, with their views (STRUCTURE §5, move 7).

use super::*;

/// `Unit::kill_garrison_order@005e2bd0` once it has its captain, in the
/// original's own call order (item 2400's sweep,
/// `crate::sweep::unit_kill_garrison_order`): at each link `get_action`
/// (`608450`) and its order's type (vslot `+0x10`) against `GARRISON`
/// (`0x1a`) gate `repath` (`5e29b0`) and `kill_current_order(0)`
/// (`5e2cb0`) (`5e2c09`-`5e2c2b`); then `o_down` (`+0x90`, signed) below
/// zero ends the walk (`5e2c3a`), as does a next object whose `+8` bit 0 is
/// clear (`5e2c60`); else the walk moves to it. The original has no bound
/// (a cycle of live links never ends); `limit` is the port's own, and the
/// sweep's chains are acyclic.
pub(crate) fn kill_garrison_order_steps<C, K: Copy>(
    cx: &mut C,
    captain: K,
    limit: usize,
    mut is_garrison: impl FnMut(&mut C, K) -> bool,
    mut repath: impl FnMut(&mut C, K),
    mut kill_current: impl FnMut(&mut C, K),
    mut down: impl FnMut(&mut C, K) -> Option<K>,
) {
    let mut v = captain;
    for _ in 0..limit {
        if is_garrison(cx, v) {
            repath(cx, v);
            kill_current(cx, v);
        }
        match down(cx, v) {
            Some(next) => v = next,
            None => return,
        }
    }
}

/// Which worker kind a unit type is, for the gather chain and `think_peasant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Worker {
    #[default]
    None,
    Citizen,
    Scholar,
}

impl Sim {
    /// The swarm ring (§5.4): `min(x_size, y_size) × 0x60 + 0x30`, halved
    /// for a farm under a build, nudged `0x30` outward.
    ///
    /// Returns the spot **and the move order's angle**, which is
    /// `find_angle(site − spot)` — the bearing from the spot back to the
    /// site, so a builder arrives facing what it is about to build. It is
    /// taken from the ring's own answer, *before* the `BUILD_AT` nudge:
    /// the listing at `7103f3`–`710406` reads the two out-parameters of
    /// `find_nearby_spot` straight into the subtraction and only then, at
    /// `710415`, runs `leal 0x30(%esi)` on the same register. The
    /// decompiler prints both of the function's `find_angle` calls with
    /// the same two locals because the pair travels in `ecx`/`edx`, and
    /// they are not the same pair: the first is `find_angle(unit − site)`
    /// (`71021a`), the ring's sweep bearing, and this is the second.
    pub(super) fn swarm_spot(&self, u: usize, b: usize, building: bool) -> Option<(Pos, Angle)> {
        let bd = &self.buildings[b];
        let (xs, ys) = bd.ty.map_or((1, 1), |t| {
            (
                self.tables.build_types[t].x_size,
                self.tables.build_types[t].y_size,
            )
        });
        let mut r = xs.min(ys) * HALF_TILE + SNAP;
        if building && self.building_ident(b) == Ident::Farm {
            r /= 2;
        }
        let here = self.units[u].pos;
        let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
        let seeker = match self.swarm_seeker_type(u, b) {
            Some(t) => Seeker::TypeFor(t, u),
            None => Seeker::Unit(u),
        };
        let sweep = |centre: Pos, min: i32, max: i32, step: i32| {
            self.spot_sweep(
                seeker,
                centre,
                min,
                max,
                step,
                angle,
                Some(b),
                Coll::Pairwise,
                false,
            )
        };
        let spot = sweep(bd.pos, r, 0, -1)?;
        let facing = find_angle(bd.pos.x - spot.x, bd.pos.y - spot.y);
        if !building {
            return Some((spot, facing));
        }
        // The `+0x30` nudge away from the site on each axis, re-validated
        // as a one-candidate ring.
        let nudge = |v: i32, c: i32| {
            if v > c {
                v + SNAP
            } else if v < c {
                v - SNAP
            } else {
                v
            }
        };
        let nudged = Pos::new(nudge(spot.x, bd.pos.x), nudge(spot.y, bd.pos.y));
        Some((sweep(nudged, 0, 0, 0).unwrap_or(spot), facing))
    }

    /// **The type the swarm ring is searched as** (`Group::action_swarm_around
    /// @0070fbe0`, `00710238`–`007102cb`): a **water** building
    /// (`is_wallbuild`, its type's `domain` 1 — the Oil Platform, the Dock
    /// lineage excepted by `is(DOCK)`) swarmed by a citizen
    /// (`ObjectData::is_peasant`) of a leader that `can_transport` at
    /// `TRANSPORT_CIVILIAN` or above is searched as that leader's
    /// `current_upgrade(TRANSPORTBARGE)` — a sea type, so the ring takes
    /// water, with the member still the pairwise pair's `not_o`. Anything
    /// else is searched as the member's own type (`None`). run678's citizen
    /// `1/26` on 1182, sent to the Oil Platform `1/2031` at sea, walks to
    /// the ring's first water point in the original and stood in ours
    /// (`docs/AI.md` §151).
    ///
    /// SEAM: the arm after it (`007102d6`–`0071039e`) — a member carrying
    /// citizens (`count_inside(PEASANTS)`) swarming a **land** building is
    /// searched as its `inside_down` passenger's type. No member of a
    /// swarm here carries anyone.
    pub(super) fn swarm_seeker_type(&self, u: usize, b: usize) -> Option<usize> {
        let t = self.buildings[b].ty?;
        if self.tables.build_types[t].domain() != crate::cities::build::BuildDomain::Water
            || crate::cities::build::is_dock(&self.tables.build_types, t)
            || !self.is_peasant(u)
            || self.transport_level(self.units[u].owner)
                < crate::ai::transport::TransportType::Civilian
        {
            return None;
        }
        self.barge_type(self.units[u].owner)
    }

    /// One ring query of `Group::action_swarm_around`'s member arm
    /// (`UnitType::find_nearby_spot`, `0070fbe0:7103e6`/`710472`): the
    /// member's own type, or the barge a citizen swarming water is
    /// searched as ([`Self::swarm_seeker_type`]), around `centre`.
    /// [`crate::groups::swarm_around_view::SimSwarm`] asks it once for the first
    /// spot and once for the build's nudged one.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn swarm_ring(
        &self,
        u: usize,
        b: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
    ) -> Option<Pos> {
        let seeker = match self.swarm_seeker_type(u, b) {
            Some(t) => Seeker::TypeFor(t, u),
            None => Seeker::Unit(u),
        };
        self.spot_sweep(
            seeker,
            centre,
            min,
            max,
            step,
            angle,
            Some(b),
            Coll::Pairwise,
            false,
        )
    }

    /// `Group::action_swarm_around` for one unit: the approach move in
    /// front, then the order re-queued with the same action bit.
    ///
    /// The approach's class is `add_move_facing_order`'s switch on
    /// `local_40`, which is 1 (`MOVE_TO`) unless the swarm is `BUILD_AT`
    /// for a computer, when it is 3 (`EXPLORE_TO`): `~(leader_flags >> 1)
    /// & 2 | 1`, `leader_flags & 4` being the human bit. So a human's
    /// builder walks under a `MOVEORDER` — run241's `0/8` taking
    /// `find_build_spot`'s help on 1097 (`docs/GOLDEN.md` §26).
    ///
    /// SEAM: a computer's `REPAIR` swarm is `MOVE_TO` in the original and
    /// an `EXPLORE_TO` here; the AI repairs on both long captures, so the
    /// change is held until both words are measured with it.
    pub(crate) fn swarm_around(&mut self, u: usize, b: usize, body: Body, action: bool) {
        let building = matches!(body, Body::Build(_));
        let kind = if self.cities.nation[self.units[u].owner as usize].human {
            MoveKind::MoveTo
        } else {
            MoveKind::ExploreTo
        };
        // **Nothing is queued when the ring finds no spot** (item 3622,
        // `docs/AI.md` §330): the member arm's move, its order and the
        // builder's mark all sit inside `find_nearby_spot == 0`
        // (`swarm_around_view::swarm_member`). `do_build`'s re-swarm of a
        // Dock site the ring cannot reach therefore drops the build, and the
        // insert dance's `finish_insert` re-swarms what was queued under it
        // (run1275b's `1/17` on 1875); this shape inserted the order with no
        // approach and the builder stood on it.
        let Some((spot, facing)) = self.swarm_spot(u, b, building) else {
            return;
        };
        self.add_move_facing_order(u, spot, kind, QueuePos::First, false, facing, None, false);
        // **The builder's mark.** `action_swarm_around@0070fbe0` has one
        // arm for `QUEUE_NEW` and `QUEUE_LAST` (`QUEUE_FIRST` re-enters
        // it at `QUEUE_NEW`, `70fd40`), and that arm queues the order
        // through `Unit::add_build_order@005e5210` (`710797`) or
        // `add_repair_order@005e4ff0` (`7106fb`), each of which sets
        // `unit_masks |= 0x400` (`5e5247`) — the "has been a builder" bit
        // `think_peasant`'s build arm reads. This shape inserted the order itself and left it
        // unset, so the AI's builder sent through `produce_building`'s
        // `QUEUE_NEW` arm, its site re-placed under it, went idle without
        // it and took a gather where the original took the new site
        // (run921's `1/7` on 1378, `docs/AI.md` §248).
        if matches!(body, Body::Build(_) | Body::Repair(_)) {
            self.units[u].was_builder = true;
        }
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body,
        };
        // The build order goes behind the approach move and ahead of
        // everything that was queued.
        let at = usize::from(self.current_order(u).is_some_and(Order::is_transit));
        self.units[u].orders.insert(at, order);
        if at == 0 {
            self.new_head(u);
        }
        self.update_action(u);
    }

    /// `Group::action_swarm_around`'s member arm at **`QUEUE_LAST`**
    /// (`0070fbe0`, `docs/GROUPS.md` §24): the approach move appended, and
    /// the build or repair order appended behind it, so both run after
    /// whatever the member already holds.
    ///
    /// Three things differ from [`Self::swarm_around`]'s `QUEUE_FIRST`
    /// shape. **Nothing is queued when the ring finds no spot**: the move
    /// and the order are both inside the `find_nearby_spot == 0` arm. The
    /// move's kind is the caller's (`local_40`). And the move is
    /// **`QUEUE_NEW`** when the member's action is a gather and the
    /// order carries the action bit (`00710487`), which clears the list
    /// before the approach goes in.
    pub(crate) fn swarm_around_last(
        &mut self,
        u: usize,
        b: usize,
        body: Body,
        action: bool,
        kind: MoveKind,
        pos: QueuePos,
    ) {
        let building = matches!(body, Body::Build(_));
        let Some((spot, facing)) = self.swarm_spot(u, b, building) else {
            return;
        };
        let gathering = self
            .action_of(u)
            .is_some_and(|i| self.units[u].orders[i].index() == index::GATHER);
        let pos = if pos == QueuePos::Last && gathering && action {
            QueuePos::New
        } else {
            pos
        };
        self.add_move_facing_order(u, spot, kind, pos, false, facing, None, false);
        // `is_castable(0x293)`: a Militia takes the Civilian at
        // `QUEUE_LAST` between the approach and the order, untargeted
        // (`−1, −1`) at the point it stands on at the swarm
        // (`00710520`/`007105a3`, item 1167, `docs/GOLDEN.md` §49).
        // SEAM: `BUILD_AT`'s clear of the site's `+0x60 & 0x2000`.
        if self.spell_castable(spell::CAST_CIVILIAN, u) {
            let at = self.units[u].pos;
            self.add_cast_order_on(u, spell::CAST_CIVILIAN, None, at, QueuePos::Last, action);
        }
        match body {
            Body::Repair(_) => self.add_repair_order(u, b, QueuePos::Last, action),
            _ => self.add_build_order(u, b, QueuePos::Last, action),
        }
    }

    /// `Unit::do_build` (§5.2): [`do_build_steps`] over [`DoBuildView`]
    /// (`SimDoBuild`); `crates/sim/tests/sweep/unit_do_build.rs` runs the
    /// steps against the original under the emulator (item 3295).
    pub(super) fn do_build(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Build(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let _ = frame;
        do_build_steps(&mut SimDoBuild {
            s: self,
            u,
            b,
            action: flags & flag::ACTION != 0,
        });
    }

    /// `Unit::check_build_order` (§5.3): pop the queued build orders, re-add
    /// the surviving sites, least crowded first, all with the action bit.
    ///
    /// The walk is [`crate::orders::check_build_view::check_build_order_steps`] over
    /// [`SimCheckBuild`] (item 3372, swept against the original under
    /// unicorn): a **cursor** walk, not a run of kills (the second reading's
    /// R3 C2). An intervening transit `MOVE_TO` is stepped over, not removed;
    /// the loop's own gate is `get_type() == BUILD_AT || get_type() ==
    /// MOVE_TO`, so a swarm's `EXPLORE_TO` ends the scan; and the site the
    /// scan **stops** on keeps its order (R3 C5): the `break` jumps out
    /// before the `repath; kill`, so that site ends the pass with a
    /// duplicate, which the next `do_build` disposes of through its
    /// "already finished" branch. After a kill the original re-tails the list
    /// and resumes from the front.
    ///
    /// **The crowd** is the units `Objects::find_units(SEARCH_FRIENDLY, owner,
    /// UNIT_BUILD_RESPOND_RANGE * 0xc0, 0x200, FILTER_BUILDREPAIR)` returns
    /// (the sweep forced this: the reading had counted every unit of the
    /// owner, in reach or not), less this unit itself — the count skips the
    /// unit doing the counting (`check_build_order@00603470:132` guards the
    /// whole body on `this_00->o != this->o`; `find_build_spot`'s otherwise
    /// identical count does *not*; see the audit's R3 F2) — whose action is a
    /// `BUILD_AT` on the site, issued under this owner.
    pub(super) fn check_build_order(&mut self, u: usize) {
        crate::orders::check_build_view::check_build_order_steps(&mut SimCheckBuild {
            s: self,
            u,
            i: 0,
            found: Vec::new(),
            site: 0,
        });
    }

    /// `Unit::build_done` (§5.5): what a builder does next.
    ///
    /// **The AI's arm is its own and stops there.** `unit_masks & 0x40000`
    /// takes `find_build_spot` → `find_repair_spot` → `find_gather_spot`
    /// with **no stance gate** and, crucially, **without the tail** that
    /// hands a human builder the site it has just finished. The two object
    /// searches are the unmodelled seams; the gather search is not.
    pub(super) fn build_done(&mut self, u: usize, site: Option<usize>) {
        if self.units[u].orders.len() > 1 {
            return;
        }
        let who = self.units[u].owner;
        if self.ai_driven(who) {
            // **The AI's first question is where to build next**, and it is
            // what an AI citizen that has just finished a site does instead
            // of adopting it (§5.2 step 6). Great Lakes' `1/1` finishes
            // building `2010` on frame 2803 and walks to `2011` on 2804;
            // this crate sent it to gather until [`Self::find_build_spot`]
            // landed. `find_repair_spot` (item 3187) only ever
            // runs when the build search comes back empty.
            if self.find_build_spot(u) || self.find_repair_spot(u) {
                return;
            }
            if !self.ai.lobby.resources_unlimited() {
                self.find_gather_spot(u, self.tables.tuning.unit_gather_respond_range * TILE);
            }
            return;
        }
        let stance = self.units[u].stance;
        if (stance == 1 || stance == 2) && self.find_build_spot(u) {
            return;
        }
        if stance <= 1
            && self.find_gather_spot(u, self.tables.tuning.unit_gather_respond_range * TILE)
        {
            // The human arm's gather clears the unit's `+0x80` (`group`),
            // and only the pointer: the pool keeps it listed (`00603cb5`;
            // run430's `0/9` on 1075, a Barracks finished and a Woodcutter's
            // Camp taken — item 1167).
            self.units[u].group_ptr = None;
            return;
        }
        if let Some(b) = site
            && self
                .buildings
                .get(b)
                .is_some_and(|bd| bd.alive && bd.owner == who)
            && self.is_gather_type(b)
            && !matches!(
                self.building_ident(b),
                Ident::University | Ident::OilPlatform
            )
            && self.gather_room(b)
        {
            self.add_gather_order(u, b, QueuePos::New, false);
        }
    }

    /// `Objects::find_builds`/`Objects::find_units`' ring index:
    /// `(range + 0x2ff) / 0x300`, the range in **cells** rounded up, and a
    /// cell is four tiles. Both searches take their cheap circle walk only
    /// while that ring holds no more points than the thing being counted
    /// (`num_def_builds`, a flat **200** from `Game::init_data@0058dca0`,
    /// and `total_units`, the live unit count); past it they walk the
    /// object lists instead.
    pub(super) fn ring_index(range: i32) -> usize {
        ((range.max(0) + 0x2ff) / 0x300).min(0x40) as usize
    }

    /// `game->total_units`, which picks the unit searches' walk: the count
    /// `Unit::init@00612100:379` raises and `Unit::close@0060ee50:69`
    /// lowers, **for an owner below nine only** — the players and the
    /// animals (owner 8). An owner-9 unit (a flock bird, a farm's beast) is
    /// inited without being counted. run672's block 690 holds 134 (14, 16
    /// and 104) beside nineteen owner-9 units, under `circle_radius[6]`'s
    /// 145, so the original walked the lists where a count of all 153 took
    /// the circle (`docs/AI.md` §148).
    pub(crate) fn total_units(&self) -> usize {
        self.units
            .iter()
            .filter(|x| x.alive() && x.owner < 9)
            .count()
    }

    /// `find_units@0065a620`'s list-walk reach (listing `65aa72..65aaa6`):
    /// a candidate is kept when `vector_dist − R <= range`, R its
    /// `push_size` if it is a sea unit and its `big_radius` otherwise.
    pub(crate) fn find_units_reach(&self, o: usize) -> i32 {
        if self.units[o].kind.domain == crate::territory::attrition::Domain::Sea {
            self.profile(Obj::Unit(o)).push_size
        } else {
            self.units[o]
                .ty
                .map_or(0, |t| self.tables.unit_types[t].combat.big_radius)
        }
    }

    /// `Objects::find_builds(SEARCH_FRIENDLY, who, range, 0x200,
    /// FILTER_CONSTRUCT)` as `find_build_spot` uses it, with that caller's
    /// own two extra tests folded in: **my** sites, and not under attack
    /// (`build_masks & 0x20`).
    ///
    /// `FILTER_CONSTRUCT` is arm 5 of `Search::valid_filter@0067dbb0`'s
    /// jump table — the table is at `0067e57c` and the arm at `0067dd54`,
    /// read from the PE because the decompiler prints the dispatch as an
    /// indirect jump. It is `vtable+0xc` (the object exists) and then a
    /// **negated** `vtable+0x4c` on what `vtable+0x40` hands back: an
    /// object that is not active, which for a building is a site still
    /// under construction. Arm 6 next door is `FILTER_DAMAGED` and is the
    /// same pair un-negated plus `+0x24 damage != 0`, which is how the
    /// polarity is settled.
    ///
    /// The `0x200` flag is the region gate: only cells whose region is the
    /// searcher's own. Order is the circle's, then — because
    /// `Object::add_to_world` pushes onto the head of the cell's chain —
    /// the **newest** object of a cell first. Nothing threads a building
    /// into `chain_heads` here (`docs/QUEUE.md` 48), so the within-cell
    /// half is descending slot order rather than a chain walk.
    ///
    /// The walk is [`crate::cities::find_builds_view::find_builds_steps`] over
    /// [`crate::cities::find_builds_view::SimFindBuilds`]; `sweep::objects_find_builds`
    /// runs it against the original under the emulator (item 3379). Its list
    /// walk (past `num_def_builds`' circle) keeps a site when
    /// `vector_dist(dx - 192 * x_size, dx - 192 * y_size) <= range`, `dx` the
    /// **x** offset twice: the listing never reads the building's `+0x14`.
    pub(crate) fn find_construct_sites(&self, u: usize, range: i32) -> Vec<usize> {
        use crate::cities::find_builds_view::{
            FILTER_CONSTRUCT, FindBuildsArgs, SEARCH_FRIENDLY, SimFindBuilds, find_builds_steps,
        };
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        // The searcher's own cell has no region: nothing shares it, and the walk
        // would match other regionless cells (`cell_region` reads them -1).
        if self.world.region_of(here.cell()).is_none() {
            return Vec::new();
        }
        let mut v = SimFindBuilds::new(self, usize::from(who));
        find_builds_steps(
            &mut v,
            &FindBuildsArgs {
                x: here.x,
                y: here.y,
                search: SEARCH_FRIENDLY,
                who: i32::from(who),
                range,
                flags: 0x200,
                filter: FILTER_CONSTRUCT,
                p8: 0,
                cont: 0,
            },
        );
        v.out
    }

    /// `Objects::find_units(SEARCH_FRIENDLY, who, range, 0x200, …)`'s answer
    /// for unit `u`, in the search's own order: the live units of `u`'s owner
    /// or an ally that the walk reaches. The filter a caller passes
    /// (`FILTER_BUILDREPAIR`) is left to the caller's own test of the
    /// unit's action — subsumed, as [`Self::build_crowd`] says.
    pub(crate) fn find_units_friendly(&self, u: usize, range: i32) -> Vec<usize> {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let mut out = Vec::new();
        let circle = crate::ai::ai_place::circle();
        let ring = Self::ring_index(range);
        let live = self.total_units();
        let keep = |sim: &Self, o: usize, out: &mut Vec<usize>| {
            let unit = &sim.units[o];
            if unit.alive() && (unit.owner == who || sim.is_ally(who, unit.owner)) {
                out.push(o);
            }
        };
        if circle.radius[ring] <= live {
            // The circle walk (`find_units@0065a620`, the decompile's
            // lines 50–125): every cell of the ring is walked, and the `0x200` region gate
            // is **each unit's own tile region** against the searcher's
            // (`get_tregion` at `div_3_table[pos >> 6]`), never the cell's.
            // A builder carried at sea stands on the coastal cell's water
            // and answers the sea region: run679's `1/24` on 1340, bound for
            // the Mine `1/2028` as a barge, is not counted (`docs/AI.md` §155).
            let c0 = here.cell();
            let region = self.world.tregion_alt(here.tile());
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) {
                    continue;
                }
                let slot = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
                let mut next = self.terrain.chain_heads[slot];
                while let Some(o) = next {
                    next = self.units[o].down;
                    if self.world.tregion_alt(self.units[o].pos.tile()) != region {
                        continue;
                    }
                    keep(self, o, &mut out);
                }
            }
        } else {
            // The list walk (`find_units@0065a620`, `65aa72..65aaa6`): the
            // candidate's own tile region against the searcher's, and the
            // reach less its radius.
            let region = self.world.tregion_alt(here.tile());
            for o in 0..self.units.len() {
                let p = self.units[o].pos;
                if !self.units[o].on_map
                    || self.world.tregion_alt(p.tile()) != region
                    || vector_dist(p.x - here.x, p.y - here.y) - self.find_units_reach(o) > range
                {
                    continue;
                }
                keep(self, o, &mut out);
            }
        }
        out
    }

    /// `Unit::find_build_spot@00603e20` (§5.5) — the search an AI builder
    /// takes the moment a site is finished, and the one an idle builder
    /// takes ahead of the gather search.
    ///
    /// The walk is [`crate::orders::build_spot_view::build_spot_steps`] over
    /// [`crate::orders::build_spot_view::SimBuildSpot`]; `sweep::unit_find_build_spot` runs it against the
    /// original under the emulator (item 3372). Range is
    /// `UNIT_BUILD_RESPOND_RANGE × 0xc0`, **doubled** on worker stance 1 or 2.
    /// The candidates are [`Self::find_construct_sites`]; the choice is the
    /// fewest builders already on one, and the first of a tie, because the
    /// original's min-search is a strict `<` walking up from index 0. The
    /// winner is a `swarm_around` with `BUILD_AT` and no action bit.
    ///
    /// The original wraps the swarm in a one-member `Group`; every other
    /// swarm call site here is the same single-unit shape (§10).
    pub(crate) fn find_build_spot(&mut self, u: usize) -> bool {
        crate::orders::build_spot_view::build_spot_steps(
            &mut crate::orders::build_spot_view::SimBuildSpot::new(self, u),
        )
    }

    /// `Unit::find_repair_spot@00604320` (`docs/ORDERS.md` §5.10), as the
    /// AI takes it: a computer below difficulty 2 answers 0 and never
    /// repairs on its own (`docs/NATIONS.md` §6; `get_diff() < 2`).
    ///
    /// The search is `ObjectsData::find_any_building@00659ca0` with
    /// `SEARCH_FRIENDLY`, range `UNIT_BUILD_RESPOND_RANGE × 0xc0` (×0x180
    /// for a human on worker stance 1 or 2), the `0x200` region gate,
    /// `FILTER_DAMAGED` (an active building with `damage != 0`) and
    /// `FILTER_NOT_UNDER_ATTACK`. It keeps the **nearest** candidate,
    /// `vector_dist <= find_dist`, so a tie goes to the later one visited:
    /// the circle's cells in order (`range / 0x300 <= 9`), else the object
    /// arrays. A building on a tile owned by a leader that is not me or a
    /// mutual ally answers 0. Then a one-member `Group` takes
    /// `action_swarm_around(b, QUEUE_LAST, REPAIR, 0)`: **`MOVE_TO`** for a
    /// computer, which is what `swarm_around`'s SEAM is held for.
    pub(crate) fn find_repair_spot(&mut self, u: usize) -> bool {
        let who = self.units[u].owner;
        let human = !self.unit_ai_bit(who) && !self.ai_driven(who);
        let mut range = self.tables.tuning.unit_build_respond_range * TILE;
        if human {
            let stance = self.units[u].stance;
            if stance == 1 || stance == 2 {
                range *= 2;
            }
        } else if self.ai_difficulty() < 2 {
            return false;
        }
        let here = self.units[u].pos;
        let Some(region) = self.world.region_of(here.cell()) else {
            return false;
        };
        let candidate = |sim: &Self, b: usize| {
            let bd = &sim.buildings[b];
            bd.alive
                && bd.active
                && bd.damage != 0
                && !bd.is_under_attack()
                && (bd.owner == who || sim.is_ally(who, bd.owner))
                && sim.world.region_of(bd.pos.cell()) == Some(region)
                && vector_dist(bd.pos.x - here.x, bd.pos.y - here.y) <= range
        };
        let mut order: Vec<usize> = Vec::new();
        if range < 0 || (range + 0x2ff) / 0x300 > 9 {
            order.extend(0..self.buildings.len());
        } else {
            let circle = crate::ai::ai_place::circle();
            let ring = Self::ring_index(range);
            let c0 = here.cell();
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) {
                    continue;
                }
                order.extend(
                    (0..self.buildings.len())
                        .rev()
                        .filter(|&b| self.buildings[b].pos.cell() == c),
                );
            }
        }
        let mut best: Option<(i32, usize)> = None;
        for b in order {
            if !candidate(self, b) {
                continue;
            }
            let p = self.buildings[b].pos;
            let d = vector_dist(p.x - here.x, p.y - here.y);
            if best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, b));
            }
        }
        let Some((_, b)) = best else {
            return false;
        };
        if let crate::Owner::Player(p) = self.world.owner_at(self.buildings[b].pos)
            && p != who
            && !self.is_ally(who, p)
        {
            return false;
        }
        let kind = MoveKind::MoveTo;
        self.swarm_around_last(u, b, Body::Repair(b), false, kind, QueuePos::Last);
        true
    }

    /// `Unit::do_repair` (§5.6): [`do_repair_steps`] over [`DoRepairView`]
    /// (`SimDoRepair`); `crates/sim/tests/sweep/unit_do_repair.rs` runs the
    /// steps against the original under the emulator (item 3346).
    pub(super) fn do_repair(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Repair(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        do_repair_steps(&mut SimDoRepair {
            s: self,
            u,
            b,
            action: flags & flag::ACTION != 0,
            frame: frame as i32,
        });
    }

    /// `Unit::do_garrison`'s order side (§5.7): the gates are the garrison
    /// mechanic's; not adjacent → a move **in front**, the order stays.
    pub(super) fn do_garrison_order(&mut self, u: usize) {
        let Some(Order {
            body:
                Body::Garrison {
                    building: b,
                    search,
                },
            // The redirected order's action bit is a constant 0, so the
            // original order's flags are not read here (R3 G6).
            flags: _,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        // **The helicopter arm** (`do_garrison@005e6b80:63`–`:83`, the
        // listing's `is(0x1bf, 0)` at `5e6bf4`; item 3990, `docs/GOLDEN.md`
        // §183): a type that flies like a helicopter ordered into an
        // Airbase that `can_carry`s it flies in — the order killed, a
        // strafe with no target home there (`returning` from its birth, and
        // `+0x3c = 1` again after `update_order`) put first with the action
        // bit. A base that will not take it gets a `QUEUE_FIRST` move to
        // its point and the red feedback (the console's alone).
        if self.is_helicopter(u) && self.building_ident(b) == crate::cities::build::Ident::Airbase {
            self.kill_current_order(u);
            if self.base_can_carry(b, u) {
                self.add_strafe_order(u, None, Some(b), true, QueuePos::First, true);
                self.update_action(u);
            } else {
                let at = self.buildings[b].pos;
                self.add_move_order(u, at, MoveKind::MoveTo, QueuePos::First, false);
            }
            return;
        }
        match self.garrison(u, b) {
            Ok(()) => self.kill_garrison_order(u),
            Err(GarrisonRefused::NotAdjacent) => {
                let bd = &self.buildings[b];
                let (xs, ys) = bd.ty.map_or((1, 1), |t| {
                    (
                        self.tables.build_types[t].x_size,
                        self.tables.build_types[t].y_size,
                    )
                });
                let r = xs.min(ys) * HALF_TILE + SNAP;
                let here = self.units[u].pos;
                let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
                let bpos = bd.pos;
                match self.find_nearby_spot(u, bpos, r, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
            }
            Err(GarrisonRefused::Full | GarrisonRefused::NoCapacity) if search => {
                let who = self.units[u].owner;
                let city = self.buildings[b].city;
                let next = self.find_garrison_build(u, city, who);
                self.kill_current_order(u);
                if let Some(n) = next {
                    // The action bit of the redirected order is **not** the
                    // original's: `do_garrison@005e6b80:288` reads it through
                    // vtable slot `+0x2c` on the order's `UnitOrder`
                    // sub-object, and `GarrisonOrder`'s entry there
                    // (`0xb48ee8+0x2c` → `0x41bff0`) is the folded
                    // `xor eax,eax; ret`. So it is always 0.
                    // (`docs/audit/2026-08-21-orders.md` R3 G6.)
                    self.add_garrison_order(u, n, true, QueuePos::First, false);
                }
            }
            Err(_) => self.kill_current_order(u),
        }
    }

    /// `Unit::kill_garrison_order(captain, 0)@005e2bd0`: from the squad's
    /// captain down its `o_down` chain, each unit whose action is a
    /// GARRISON is `repath`ed — its leading moves go — and has its head
    /// killed, once. The walk stops at the first link that is not active.
    /// run208's squad goes in on 761 through `0/7`, and `0/8` and `0/9`
    /// go in with their walks and their GARRISONs gone too.
    ///
    /// The walk is [`kill_garrison_order_steps`] (item 2400), which the sweep
    /// `sweep::unit_kill_garrison_order` holds to the original.
    pub(super) fn kill_garrison_order(&mut self, u: usize) {
        let captain = self.captain_of(u);
        let limit = self.units.len();
        kill_garrison_order_steps(
            self,
            captain,
            limit,
            |s, v| {
                s.action_of(v)
                    .is_some_and(|a| matches!(s.units[v].orders[a].body, Body::Garrison { .. }))
            },
            |s, v| s.repath(v),
            |s, v| s.kill_current_order(v),
            |s, v| match s.units[v].o_down {
                Some(next) if s.units[next].alive() => Some(next),
                _ => None,
            },
        );
    }

    /// `Unit::find_garrison_build`: the city's nearest active building with
    /// room that the unit may enter.
    pub(crate) fn find_garrison_build(
        &self,
        u: usize,
        city: Option<usize>,
        who: Player,
    ) -> Option<usize> {
        let city = city?;
        let here = self.units[u].pos;
        let uty = self.units[u].ty?;
        self.buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.active && b.owner == who && b.city == Some(city))
            .filter(|(i, b)| {
                b.ty.is_some_and(|t| self.can_garrison(uty, t))
                    && self.num_inside(*i) < self.garrison_limit(*i)
            })
            .min_by_key(|(_, b)| vector_dist(b.pos.x - here.x, b.pos.y - here.y))
            .map(|(i, _)| i)
    }

    // ------------------------------------------------------------------
    // The gather chain — §6.1
    // ------------------------------------------------------------------

    /// `BuildType` vslot `0x90`: `build_flags & 0x40`.
    pub fn is_gather_type(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.tables.build_types[t].has(bflags::GATHERER))
    }

    /// `build_flags & FLAT`.
    pub fn is_flat(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.tables.build_types[t].has(bflags::FLAT))
    }

    /// The unit type's worker kind.
    pub fn worker_of(&self, u: usize) -> Worker {
        self.units[u]
            .ty
            .map_or(Worker::None, |t| self.tables.unit_types[t].worker)
    }

    /// `BuildData::gather_max` against the chain: room for one more.
    pub(super) fn gather_room(&self, b: usize) -> bool {
        self.buildings[b]
            .gather_max
            .is_none_or(|max| self.num_gatherers(b, false, false) < max)
    }

    /// `Build::add_gatherer` (§6.1): same owner, room, a worker on the map,
    /// not already in the chain → prune, then push-front.
    pub fn add_gatherer(&mut self, b: usize, u: usize) -> bool {
        let unit = &self.units[u];
        if self.buildings[b].owner != unit.owner
            || !self.gather_room(b)
            || !unit.alive()
            || !unit.on_map
            || self.worker_of(u) == Worker::None
            || self.buildings[b].gatherers.contains(&u)
        {
            return false;
        }
        self.check_gatherers(b);
        self.buildings[b].gatherers.insert(0, u);
        true
    }

    /// `Build::remove_gatherer`.
    pub fn remove_gatherer(&mut self, b: usize, u: usize) {
        self.buildings[b].gatherers.retain(|&x| x != u);
    }

    /// `Build::check_gatherers`: prune the dead, the re-tasked, the off-map.
    pub fn check_gatherers(&mut self, b: usize) {
        // The list is the chain, its order the links (`+0x70`, then each
        // member's `+0x92`).
        let list = self.buildings[b].gatherers.clone();
        let mut next: Vec<i32> = (1..=list.len())
            .map(|k| if k < list.len() { k as i32 } else { -1 })
            .collect();
        let mut head = if list.is_empty() { -1 } else { 0 };
        prune_chain(
            &mut next,
            &mut head,
            |n, k| n[k as usize],
            |n, k, v| n[k as usize] = v,
            |_, k| {
                let u = list[k as usize];
                self.units[u].alive() && self.is_gathering_at(u, b, false) && self.units[u].on_map
            },
        );
        let mut keep = Vec::new();
        while head >= 0 {
            keep.push(list[head as usize]);
            head = next[head as usize];
        }
        self.buildings[b].gatherers = keep;
    }

    /// `BuildData::is_gathered_by`.
    pub fn is_gathered_by(&self, b: usize, u: usize) -> bool {
        self.buildings[b].gatherers.contains(&u)
    }

    /// `UnitData::is_gathering_at(o, who, strict)`: a worker whose **action**
    /// is a GATHER on this building — and, strictly, arrived.
    ///
    /// The match is on [`Sim::action_of`], not the front of the list
    /// (`is_gathering_at@00608880`: `pUVar3 = get_action(this)`, then
    /// `get_type() == 7`). That is load-bearing, not pedantry: `do_gather`
    /// and `do_non_flat_gather` insert their walks as `QUEUE_FIRST` moves
    /// *without* the action bit, so on every walk-out and walk-back leg the
    /// front order is a move and only the action walk still finds the
    /// GATHER. Reading the front instead made a woodcutter stop counting as
    /// a gatherer the moment it set off — and be pruned out of its own chain
    /// by `check_gatherers`. (`docs/audit/2026-08-21-orders.md` R4 G16.)
    ///
    /// The inside-the-building arm is **scholar-only**: the original gates it
    /// on `ptype[4] ∈ {SCHOLARS, SCHOLARSKOREAN}` (G17), so a garrisoned
    /// citizen matches nothing. A platform's peasants are counted by
    /// `num_gatherers` through `count_inside` instead.
    pub fn is_gathering_at(&self, u: usize, b: usize, strict: bool) -> bool {
        let unit = &self.units[u];
        let worker = self.worker_of(u);
        gathering_at_steps(
            worker == Worker::Scholar,
            || unit.on_map,
            || unit.inside == Some(b),
            strict,
            worker != Worker::None,
            || match self
                .action_of(u)
                .and_then(|i| unit.orders.get(i))
                .map(|o| o.body)
            {
                Some(Body::Gather(g)) => Some((g.building == b, i32::from(g.been_there))),
                _ => None,
            },
        ) != 0
    }

    /// `BuildData::num_gatherers(arrived, skip_decoys)`: the garrisoned
    /// gatherers of a university or platform, plus the chain members that
    /// are gathering here. `calc_gather` calls it `(1, 1)`. `skip_decoys`
    /// turns the garrison term's `count_inside` mode from `COUNT_TYPE` (17)
    /// to `COUNT_NON_DECOY_TYPE` (19) as well as filtering the chain (A7
    /// row 28, twenty-fourth pass).
    pub fn num_gatherers(&self, b: usize, arrived: bool, skip_decoys: bool) -> i32 {
        let ident = self.building_ident(b);
        let mut n = 0;
        if matches!(ident, Ident::University | Ident::OilPlatform) {
            let want = if ident == Ident::University {
                Worker::Scholar
            } else {
                Worker::Citizen
            };
            n += self.buildings[b]
                .garrison
                .iter()
                .filter(|&&u| self.worker_of(u) == want)
                .filter(|&&u| !(skip_decoys && self.units[u].decoy))
                .count() as i32;
        }
        n + chain_gatherers(
            self.buildings[b].gatherers.iter().copied(),
            skip_decoys,
            |u| self.units[u].decoy,
            |u| self.is_gathering_at(u, b, arrived),
        )
    }

    /// `Build::all_gathering`: every chain member is out at its tile.
    ///
    /// **The chain is pruned first**: `62f573` calls
    /// `Build::check_gatherers@0062f710` before the walk, so a member that
    /// is dead, off the map or no longer gathering here is dropped rather
    /// than read as a gatherer still walking (item 1248, `docs/GOLDEN.md`
    /// §52).
    pub(super) fn all_gathering(&mut self, b: usize) -> bool {
        self.check_gatherers(b);
        all_at_work(
            self.buildings[b].gatherers.iter().copied(),
            |u| match self.units[u].orders.front().map(|o| o.body) {
                Some(Body::Gather(g)) => Some((g.goto_build, g.wait)),
                _ => None,
            },
        )
    }

    // ------------------------------------------------------------------
    // `do_gather` — §6.3
    // ------------------------------------------------------------------

    /// `Unit::do_gather` (§6.3): [`do_gather_steps`] over [`DoGatherView`]
    /// (`SimDoGather`); `crates/sim/tests/sweep/unit_do_gather_005ef2a0.rs`
    /// runs the steps against the original under the emulator (item 3356).
    pub(super) fn do_gather(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Gather(g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        do_gather_steps(&mut SimDoGather {
            b: g.building,
            g,
            s: self,
            u,
            frame: frame as i32,
            drew: false,
        });
    }

    /// Writes a gather order's fields back — **to the gather order, not to
    /// the front of the list**.
    ///
    /// The original holds a pointer to the order object (`go`) for the whole
    /// of `do_gather` and `do_non_flat_gather`, and every walk those
    /// functions issue goes in *front* of it: `add_move_order(…, 1, …)`
    /// allocates a new order and links it at the head, leaving `go` valid.
    /// So a branch that issues a walk and then writes `go->goto_build` is
    /// writing the same object it was handed.
    ///
    /// Writing the front instead dropped every field of the branches that
    /// walk — most visibly the return to camp, which sets `goto_build = 1`
    /// and `wait = 32` *after* `add_move_order` (`docs/ORDERS.md` §6.4).
    /// Without them a woodcutter that had finished its shift never left the
    /// `goto_build == 0, wait < 0` arm: it walked to the camp, arrived, and
    /// re-issued the same walk and the same `CHAR_DEFAULT` stand for the
    /// rest of the game, so it never unloaded and never took another tile.
    pub(super) fn store_gather(&mut self, u: usize, g: GatherOrder) {
        if let Some(x) = self.units[u]
            .orders
            .iter_mut()
            .find_map(|o| match &mut o.body {
                Body::Gather(x) => Some(x),
                _ => None,
            })
        {
            *x = g;
        }
    }

    /// `Unit::do_non_flat_gather` (§6.4): wood and ore, given the building's
    /// resource tiles. Without a tile list the worker stays at the camp with
    /// `been_there` set — an input.
    pub(super) fn do_non_flat_gather(&mut self, u: usize, mut g: GatherOrder) {
        let b = g.building;
        let who = self.units[u].owner;
        let wood = self.building_ident(b) == Ident::Woodcutter;
        // The recharge bump and the group drop are the swept head
        // ([`crate::economy::gather_tile::gather_head_steps`], item 3381).
        // **Every call drops the gatherer's group**, a computer's too
        // (`5f0237` `orl $-1`, `5f023e` `movw %ax, 0x80(%ebx)`: both
        // paths from the entry join there, before the `goto_build`
        // branch). The pool keeps listing the unit; only the back-pointer
        // goes, as in `think_peasant`'s human arm. On East Indies' block
        // 8369 it is `1/11`'s `group` 65 → −1, the first pool parting
        // once the AI's building groups number the pool
        // (`docs/GROUPS.md` §29).
        crate::economy::gather_tile::gather_head_steps(
            &mut crate::economy::gather_tile::SimGatherTile { sim: self, u, b },
            g.been_there,
        );
        let bpos = self.buildings[b].pos;
        let here = self.units[u].pos;
        let (xs, ys) = self.buildings[b].ty.map_or((1, 1), |t| {
            (
                self.tables.build_types[t].x_size,
                self.tables.build_types[t].y_size,
            )
        });
        let d = xs.min(ys) * HALF_TILE + SNAP;
        if !g.goto_build {
            // `do_non_flat_gather:96`, the first statement of this half:
            // the carrying walk is dropped whole, and one of the four is
            // put back by whichever walk this frame issues.
            self.units[u].carry &= !CARRY_ANY;
            if !g.been_there {
                g.been_there = true;
                self.economy.ledgers[who as usize].dirty = true;
            }
            if g.wait < 0 {
                // Return to the camp.
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                match self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                    None => {
                        // No spot: keep chopping (`do_non_flat_gather:130`).
                        let work = if wood {
                            crate::units::anim::CHOP_WOOD
                        } else {
                            crate::units::anim::MINE_ORE
                        };
                        self.set_anim(u, work, false, true);
                        g.wait = 20;
                    }
                    Some(spot) => {
                        self.mark(crate::units::anim::SITE_STAND_RETURN);
                        self.set_default_anim(u);
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                        g.goto_build = true;
                        g.wait = 32;
                        // `:150` / `:154` — the walk back is a *carrying*
                        // one, by the camp's resource.
                        self.units[u].carry |= if wood {
                            CARRY_WITH_WOOD
                        } else {
                            CARRY_WITH_ORE
                        };
                    }
                }
                self.store_gather(u, g);
                return;
            }
            // **The guy's own animation is read before the tile is**
            // (`do_non_flat_gather:255`, `005f0d0f`): a worker already in
            // its resource loop never reaches the tile arithmetic below,
            // and the branch it takes instead is where the steady-state
            // wait lives.
            //
            // - `CHAR_MINE_ORE` — a miner mid-swing does *nothing*: no
            //   decrement, no facing, no animation. Its `wait` is the
            //   1,000,000 the tile choice gave it and it stays out.
            // - `CHAR_CHOP_WOOD` — the decrement, and on zero the reroll,
            //   and that is the whole frame. No facing, no `set_anim`.
            //
            // The reroll here is `% 100 + 300`, **not** the `% 50 + 100`
            // of the arrival frame below. The two used to be one branch
            // and the count could not see it: both sites draw once, so
            // the word stayed matched for six hundred frames while the
            // worker's clock ran at a third of the original's and sent it
            // home two hundred frames early (§6.4, `docs/SYNC.md` §3.18).
            let anim = self.units[u].guys.first().map_or(0, |g| g.anim);
            if anim == crate::units::anim::MINE_ORE {
                self.store_gather(u, g);
                return;
            }
            if anim == crate::units::anim::CHOP_WOOD {
                g.wait -= 1;
                if g.wait != 0 {
                    self.store_gather(u, g);
                    return;
                }
                g.wait = if self.all_gathering(b) {
                    -1
                } else {
                    self.mark(SITE_WORK_WAIT);
                    300 + self.rng.roll() % 100
                };
                self.store_gather(u, g);
                return;
            }
            let Some(t) = g.tile else {
                // A tile was never chosen: nothing to walk to.
                self.store_gather(u, g);
                return;
            };
            let centre = Pos::new(t.x * TILE + HALF_TILE, t.y * TILE + HALF_TILE);
            if vector_dist(centre.x - here.x, centre.y - here.y) < AT_TILE {
                // The **arrival** frame: at the tile and not yet in the
                // loop. The decrement and its reroll come first
                // (`do_non_flat_gather:267`), then the facing, and the
                // work animation is the function's last statement — which
                // is what puts every later frame on the branch above.
                g.wait -= 1;
                if g.wait == 0 {
                    if self.all_gathering(b) {
                        // `LAB_005f0ef1`: the return is immediate, so this
                        // frame sets neither the facing nor the animation.
                        g.wait = -1;
                        self.store_gather(u, g);
                        return;
                    }
                    self.mark(SITE_ARRIVE_WAIT);
                    g.wait = 100 + self.rng.roll() % 50;
                }
                self.units[u]
                    .movement
                    .set_heading(find_angle(centre.x - here.x, centre.y - here.y));
                // The hold doober, between the facing and the animation
                // (`do_non_flat_gather:212`–`:256`, `docs/ORDERS.md` §6.12).
                self.add_hold_doober(u);
                let work = if wood {
                    crate::units::anim::CHOP_WOOD
                } else {
                    crate::units::anim::MINE_ORE
                };
                self.set_anim(u, work, false, true);
                self.store_gather(u, g);
                return;
            }
            // The stand before the approach — `do_non_flat_gather:275`,
            // the call at `005f113f` and so the site `+0xfd4`. §6.4's
            // pseudocode has always carried it and the implementation did
            // not; the whole-frame sequence check is what made the gap
            // countable (`docs/SYNC.md` §5.1).
            self.mark(crate::units::anim::SITE_STAND_TILE);
            self.set_default_anim(u);
            let angle = find_angle(here.x - centre.x, here.y - centre.y);
            // Three answers take the no-spot arm (`005f1191`–`005f1318`):
            // none found, the unit's own position, and **the unit's
            // `avoid_x/avoid_y`** (`cmpl 0x120(%ebx)` at `5f1298`, `0x124`
            // at `5f12a0`) — the point `do_move`'s gather park left there
            // (`docs/COLLISION.md` §15; `docs/AI.md` §329, item 3621).
            match self.find_nearby_spot(u, centre, TILE, 0x100, 2, angle, None) {
                Some(spot) if spot != here && Some(spot) != self.units[u].avoid => {
                    if !self.is_gathered_by(b, u) {
                        g.been_there = false;
                        self.store_gather(u, g);
                        return;
                    }
                    self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                    // `:376` and `LAB_005f13da` — the approach is a walk
                    // *to* the resource.
                    self.units[u].carry |= if wood { CARRY_TO_WOOD } else { CARRY_TO_ORE };
                }
                _ => {
                    g.tile = None;
                    g.goto_build = true;
                    g.wait = -1;
                    if g.dist_mod > 0 {
                        g.dist_mod -= 1;
                    }
                    // `:296`–`:301`, `:319`–`:324`, `:342`–`:347`: a
                    // worker's hold doober goes with the tile.
                    if self.is_worker(u) {
                        self.remove_hold_doober(u);
                    }
                }
            }
            self.store_gather(u, g);
            return;
        }
        // Heading to, or at, the camp.
        if g.wait >= 0 {
            if self.adjacent_to(u, b) {
                // At the camp, and the unload — `do_non_flat_gather:398`
                // through `:416`, in the original's own order: the wait is
                // decremented, `been_there` set, and a wait that has run
                // out returns *before* the facing and the animation. There
                // is **no idle stand here**: the branch is a two-way
                // `CHAR_DUMP_WOOD` / `CHAR_DUMP_ORE` on the camp's
                // resource, `been_there` is written and never read, and
                // the listing at `5f0b5e`–`5f0b89` shows the two `set_anim`
                // calls and no third.
                //
                // A first arrival used to stand idle here, which is where
                // the sim's two extra frame-0 draws came from: run20's
                // trace has no `Unit::do_non_flat_gather` draw at frame 0
                // at all, and the sequence comparison in `rondata::diff`
                // named this call as the divergence (`docs/SYNC.md` §4.2,
                // §6 — the stand half of the stand/wrap swap).
                g.wait -= 1;
                if !g.been_there {
                    g.been_there = true;
                    self.economy.ledgers[who as usize].dirty = true;
                }
                if g.wait >= 0 {
                    self.units[u]
                        .movement
                        .set_heading(find_angle(bpos.x - here.x, bpos.y - here.y));
                    let dump = if wood {
                        crate::units::anim::DUMP_WOOD
                    } else {
                        crate::units::anim::DUMP_ORE
                    };
                    self.set_anim(u, dump, false, true);
                }
                self.store_gather(u, g);
                return;
            }
            if !self.is_gathered_by(b, u) {
                g.been_there = false;
                self.store_gather(u, g);
                return;
            }
            let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
            if let Some(spot) = self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                self.store_gather(u, g);
                return;
            }
            if !g.been_there && vector_dist(bpos.x - here.x, bpos.y - here.y) > 0x600 {
                match self.find_nearby_spot(u, bpos, 0x600, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
                return;
            }
            g.wait = -1;
            self.store_gather(u, g);
            return;
        }
        // Choose a tile: [`crate::economy::gather_tile::gather_tile_steps`], the swept
        // arm (item 3381) — the stand first (`do_non_flat_gather:512`, whose
        // call is at `005f027a` and whose return address is therefore
        // `+0x10f`; the woodcutters' frame-0 draws on run12 are this call,
        // `docs/ANIM.md` §4), the held tile kept while `has_gather_access`
        // holds, the list scanned (a tiny range's weight capped at 3 —
        // East Indies' `1/11` on 10958, item 620), the wait rolled at
        // `+0x54b`. An empty or walled-in list returns before the roll and
        // **leaves a stale held tile in place**.
        let mut o = crate::economy::gather_tile::TileOrder {
            tx: g.tile.map_or(-1, |t| t.x),
            ty: g.tile.map_or(-1, |t| t.y),
            wait: g.wait,
            goto_build: g.goto_build,
            dist_mod: g.dist_mod,
            been_there: g.been_there,
        };
        let held = (o.tx, o.ty);
        crate::economy::gather_tile::gather_tile_steps(
            &mut crate::economy::gather_tile::SimGatherTile { sim: self, u, b },
            &mut o,
        );
        if (o.tx, o.ty) != held {
            g.tile = Some(Pos::new(o.tx, o.ty));
        }
        g.wait = o.wait;
        g.goto_build = o.goto_build;
        g.been_there = o.been_there;
        self.store_gather(u, g);
    }

    /// `Unit::find_gather_spot` (§6.6): the best gather building of the
    /// owner's with room, scored on the **headroom under the commerce cap**
    /// of the goods it gathers, divided by the distance. The search is
    /// [`find_gather_spot_steps`] over [`GatherSpotSim`], the same steps
    /// `sweep::unit_find_gather_spot` drives over the emulated original's
    /// windows (item 3320).
    pub(crate) fn find_gather_spot(&mut self, u: usize, range: i32) -> bool {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let mut v = GatherSpotSim {
            sim: self,
            u,
            who,
            here,
        };
        find_gather_spot_steps(&mut v, range)
    }
}

/// **`Build::all_gathering@0062f570`**'s walk, after its
/// `check_gatherers`: every member of the gather chain has a gather order
/// (`order` answers its `goto_build` and `wait`; `None` — no order at
/// `+0xdc`, a null order, or vslot `+0xe4` answering none — is false)
/// that is not going to the building and whose `wait` is not negative. An
/// empty chain answers true. Item 2276's sweep holds it.
pub(crate) fn all_at_work(
    chain: impl IntoIterator<Item = usize>,
    mut order: impl FnMut(usize) -> Option<(bool, i32)>,
) -> bool {
    chain
        .into_iter()
        .all(|u| order(u).is_some_and(|(goto_build, wait)| !goto_build && wait >= 0))
}

/// **`BuildData::num_gatherers@00630450`**'s chain term: the members of
/// the building's gather chain (`+0x70`, then each unit's `+0x92`) that
/// `gathering` answers for — `UnitData::is_gathering_at` — skipping, when
/// `skip_decoys`, a decoy (`unit_masks & 1`) **before** asking. Item
/// 2276's sweep holds it.
pub(crate) fn chain_gatherers(
    chain: impl IntoIterator<Item = usize>,
    skip_decoys: bool,
    decoy: impl Fn(usize) -> bool,
    mut gathering: impl FnMut(usize) -> bool,
) -> i32 {
    chain
        .into_iter()
        .filter(|&u| !(skip_decoys && decoy(u)) && gathering(u))
        .count() as i32
}

/// What `Unit::do_build@005eebf0` asks of its unit, the order's site and the
/// world, in the order it asks (item 3295). [`SimDoBuild`] answers from this
/// crate's state; the sweep answers from the emulated original's windows.
///
/// The site is the order's `(idx, uid)` pair; the view holds it, so no
/// method takes one. The `is(type)` questions are the object's vslot `+0xb8`
/// (the slow arm of the compare with `ObjectData::is`); the sweep holds that
/// arm only.
pub trait DoBuildView {
    /// The order's body (vslot `+0x60`), `idx` and `uid` both non-negative,
    /// and the object's vslot `+0xc` (in use).
    fn site_alive(&mut self) -> bool;
    /// The object's vslot `+0x4c`: the site is already finished.
    fn site_active(&mut self) -> bool;
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// `UnitData::order_type != NONE`, inlined (the dead-site arm: the
    /// list head, the cursor, vslot `+0x10` of the current order).
    fn order_pending_inline(&mut self) -> bool;
    /// `UnitData::order_type != NONE`, called.
    fn order_pending(&mut self) -> bool;
    /// `Unit::build_done(idx, uid, gather_respond_range * 0xc0)`.
    fn build_done(&mut self);
    /// `Unit::check_build_order`.
    fn check_build_order(&mut self);
    /// The site's `is(0x1a6)`: an oil platform.
    fn is_oil(&mut self) -> bool;
    /// `unit_masks & 0x40000`.
    fn ai_driven(&mut self) -> bool;
    /// The unit's vslot `+0xf8`: the worker stance.
    fn stance(&mut self) -> i32;
    /// The unit's player is the order's `uid`.
    fn owned(&mut self) -> bool;
    /// The site's data record, vslot `+0x90` of the object behind `+0x18`.
    fn is_gather_type(&mut self) -> bool;
    /// The site's `is(0x1a4)`: a University.
    fn is_university(&mut self) -> bool;
    /// `Unit::add_gather_order(idx, QUEUE_NEW, 0)`.
    fn add_gather_order(&mut self);
    /// What follows the gather order at `LAB_005eed5f`: the select-group
    /// drop-out (the player's group with more than one member is told, vslot
    /// `+0x10`) and the unit's `+0x80` cleared. SEAM: the interface's
    /// select groups are not modelled.
    fn gather_adopted(&mut self);
    /// The unit's vslot `+0x170` (`adjacent_to`) of the site.
    fn adjacent(&mut self) -> bool;
    /// `WallData::covers_tile` of the site's footprint at the unit's tile.
    fn covers_tile(&mut self) -> bool;
    /// The footprint object's `is(0x1a1)` (a Farm), vtable-compared.
    fn wall_is_farm(&mut self) -> bool;
    /// The order's flag byte `& 4`, through its virtual base.
    fn order_action(&mut self) -> bool;
    /// `Group::action_swarm_around(idx, player, QUEUE_FIRST, BUILD_AT,
    /// action)` on a one-member stack group.
    fn swarm(&mut self, action: bool);
    /// The site object's own `is(0x1a1)`: the work animation is a sow.
    fn is_farm(&mut self) -> bool;
    /// `Unit::set_anim(CHAR_BUILD or CHAR_SOW, 0, 1)`.
    fn set_work_anim(&mut self, farm: bool);
    /// `find_angle` of the site from the unit.
    fn angle_to_site(&mut self) -> i32;
    /// The unit's `+0x50`.
    fn heading(&mut self) -> i32;
    /// `Unit::set_angle(angle, ..., 0)`.
    fn set_angle(&mut self, angle: i32);
    /// `unit_masks & 1`: a decoy builds nothing.
    fn decoy(&mut self) -> bool;
    /// `constants->accel_construct`.
    fn accel_construct(&mut self) -> i32;
    /// The Koreans' tribe bonus `0x10` **and** `korean_build_under_fire`.
    fn under_fire_exempt(&mut self) -> bool;
    /// The site's vslot `+0x188` (`WallData::is_under_attack`).
    fn site_under_attack(&mut self) -> bool;
    /// `Wall::do_construct(amount)` of the site.
    fn do_construct(&mut self, amount: i32) -> bool;
    /// The unit's `+0x80` names a group (not negative).
    fn has_group(&mut self) -> bool;
    /// `Group::normalize` of the unit's group.
    fn group_normalize(&mut self);
    /// `GroupData::count(COUNT_PEASANTS, 0, 0)` of the unit's group.
    fn group_peasants(&mut self) -> i32;
}

/// **`Unit::do_build@005eebf0`'s steps** (item 3295, `docs/ORDERS.md` §5.2):
/// the dead or finished site, the re-swarm of a builder out of reach or
/// standing in a footprint, the work animation, the turn, the decoy's early
/// return, the construction, and what the builder does once the site is up.
pub fn do_build_steps<V: DoBuildView>(v: &mut V) {
    if !v.site_alive() {
        v.kill_current_order();
        if v.order_pending_inline() {
            return;
        }
        v.build_done();
        return;
    }
    // Whether the worker keeps the finished site: oil platforms always, a
    // player's builder only on a gathering stance (0 or 1).
    fn adopts<V: DoBuildView>(v: &mut V) -> bool {
        let oil = v.is_oil();
        (oil || (!v.ai_driven() && (v.stance() == 1 || v.stance() == 0)))
            && v.owned()
            && v.is_gather_type()
            && !v.is_university()
    }
    if v.site_active() {
        v.kill_current_order();
        if v.order_pending() {
            v.check_build_order();
            return;
        }
        if adopts(v) {
            v.add_gather_order();
            v.gather_adopted();
        } else {
            v.build_done();
        }
        return;
    }
    let reach = v.adjacent() && !(v.covers_tile() && !v.wall_is_farm());
    if !reach {
        // `005ef165`: the order's action bit is read before the kill.
        let action = v.order_action();
        v.kill_current_order();
        v.swarm(action);
        return;
    }
    let farm = v.is_farm();
    v.set_work_anim(farm);
    let angle = v.angle_to_site();
    if v.heading() != angle {
        v.set_angle(angle);
    }
    if v.decoy() {
        return;
    }
    let accel = v.accel_construct();
    let amount = if !v.under_fire_exempt() && v.site_under_attack() {
        accel / 4
    } else {
        accel
    };
    if !v.do_construct(amount) {
        return;
    }
    v.kill_current_order();
    let pending = v.order_pending();
    if pending {
        if !v.is_gather_type() || v.is_university() || !v.has_group() {
            v.check_build_order();
            return;
        }
        v.group_normalize();
        if v.group_peasants() < 2 {
            v.check_build_order();
            return;
        }
    }
    if adopts(v) {
        v.add_gather_order();
        v.gather_adopted();
    } else if pending {
        v.check_build_order();
    } else {
        v.build_done();
    }
}

/// [`DoBuildView`] over the live [`Sim`].
pub(super) struct SimDoBuild<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    pub(super) b: usize,
    pub(super) action: bool,
}

impl DoBuildView for SimDoBuild<'_> {
    fn site_alive(&mut self) -> bool {
        self.s.buildings.get(self.b).is_some_and(|bd| bd.alive)
    }
    fn site_active(&mut self) -> bool {
        self.s.buildings[self.b].active
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn order_pending_inline(&mut self) -> bool {
        self.s.order_type(self.u) != index::NONE
    }
    fn order_pending(&mut self) -> bool {
        self.s.order_type(self.u) != index::NONE
    }
    fn build_done(&mut self) {
        self.s.build_done(self.u, Some(self.b));
    }
    fn check_build_order(&mut self) {
        self.s.check_build_order(self.u);
    }
    fn is_oil(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::OilPlatform
    }
    fn ai_driven(&mut self) -> bool {
        self.s.ai_driven(self.s.units[self.u].owner)
    }
    fn stance(&mut self) -> i32 {
        i32::from(self.s.units[self.u].stance)
    }
    fn owned(&mut self) -> bool {
        self.s.buildings[self.b].owner == self.s.units[self.u].owner
    }
    fn is_gather_type(&mut self) -> bool {
        self.s.is_gather_type(self.b)
    }
    fn is_university(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::University
    }
    fn add_gather_order(&mut self) {
        self.s
            .add_gather_order(self.u, self.b, QueuePos::New, false);
    }
    fn gather_adopted(&mut self) {
        // `LAB_005eed5f`: the gather clears the unit's `+0x80` (`group`),
        // the pointer only (`005eed9c`, item 1167).
        self.s.units[self.u].group_ptr = None;
    }
    fn adjacent(&mut self) -> bool {
        self.s.adjacent_to(self.u, self.b)
    }
    fn covers_tile(&mut self) -> bool {
        self.s.covers_tile(self.b, self.s.units[self.u].pos.tile())
    }
    fn wall_is_farm(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::Farm
    }
    fn order_action(&mut self) -> bool {
        self.action
    }
    fn swarm(&mut self, action: bool) {
        // `Unit::do_build@005eebf0` re-enters through a one-member
        // `Group` at `QUEUE_FIRST` (`005ef165`–`005ef1a0`): the group
        // insert dance (`Group::action_swarm_around@0070fbe0`,
        // `docs/GROUPS.md` §17) — the unit's action-flagged orders
        // copied aside, the member halted, this swarm at `QUEUE_NEW`,
        // and each copy re-issued at `QUEUE_LAST`, so a build queued
        // behind this one is swarmed afresh from where the unit now
        // stands (run767's `1/12` on 2877, `docs/AI.md` §189).
        let (u, b) = (self.u, self.b);
        let who = self.s.units[u].owner;
        let mut g = crate::groups::group::Group::stack(who);
        self.s.group_add(&mut g, u);
        let insert = self.s.group_set_up_insert(&g);
        self.s.group_action_halt(&g, 0);
        self.s.swarm_around(u, b, Body::Build(b), action);
        self.s.group_finish_insert(&g, insert);
    }
    fn is_farm(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::Farm
    }
    fn set_work_anim(&mut self, farm: bool) {
        // §5.2 step 4's first half, and it is a draw the arrival stand
        // would otherwise spend: the builder is put on its **work**
        // animation before it is turned, so `Guy::move`'s arrival test
        // (`cur_anim == CHAR_WALK`) never sees a walk again and the idle
        // roll at `Guy::set_anim+0x97a < Guy::move+0x19f` never fires
        // (`docs/ANIM.md` §4.6). A farm sows; everything else builds.
        let work = if farm { anim::SOW } else { anim::BUILD };
        self.s.set_anim(self.u, work, false, true);
    }
    fn angle_to_site(&mut self) -> i32 {
        let bpos = self.s.buildings[self.b].pos;
        let here = self.s.units[self.u].pos;
        find_angle(bpos.x - here.x, bpos.y - here.y).0
    }
    fn heading(&mut self) -> i32 {
        self.s.units[self.u].movement.heading.0
    }
    fn set_angle(&mut self, angle: i32) {
        // `Unit::do_build@005eebf0:155` turns the builder through
        // `Unit::set_angle`, whose turn of 90° or more toggles the mirror
        // of the group it leads — the `@build`'s own group. That flag is
        // the layout's `facing` on the group's next move (item 1330:
        // chapter forty-seven's builders, `facing` 1 on block 624).
        self.s.unit_set_angle(self.u, Angle(angle));
    }
    fn decoy(&mut self) -> bool {
        self.s.units[self.u].decoy
    }
    fn accel_construct(&mut self) -> i32 {
        self.s.tables.tuning.accel_construct
    }
    fn under_fire_exempt(&mut self) -> bool {
        self.s.cities.nation[self.s.units[self.u].owner as usize].koreans
            && self.s.tables.tuning.korean_build_under_fire != 0
    }
    fn site_under_attack(&mut self) -> bool {
        self.s.buildings[self.b].is_under_attack()
    }
    fn do_construct(&mut self, amount: i32) -> bool {
        self.s.do_construct(self.b, amount)
    }
    fn has_group(&mut self) -> bool {
        self.s.units[self.u].group_ptr.is_some()
    }
    fn group_normalize(&mut self) {
        if let Some(seat) = self.s.seat_of(self.u) {
            self.s.seat_normalize(seat);
        }
    }
    fn group_peasants(&mut self) -> i32 {
        // SEAM: a rider's host (`is_map_unit` with `+0x28` set) is counted
        // too by the original; a peasant never carries, so it adds none.
        let Some(seat) = self.s.seat_of(self.u) else {
            return 0;
        };
        self.s
            .seat_list(seat)
            .iter()
            .filter(|&&m| self.s.units[m].alive() && self.s.is_peasant(m))
            .count() as i32
    }
}

/// What `Unit::do_repair@005ee420` asks of its unit, the order's site, the
/// leaders and the world, in the order it asks (item 3346). [`SimDoRepair`]
/// answers from this crate's state; the sweep answers from the emulated
/// original's windows.
///
/// The site is the order's `(idx, uid)` pair; the view holds it, so no
/// method takes one. `good` is the good's index, 0 to 5.
pub trait DoRepairView {
    /// `Unit::set_anim(CHAR_REPAIR, 0, 1)`, ahead of every gate.
    fn set_repair_anim(&mut self);
    /// The site object's `+0x24`: the damage it carries.
    fn site_damage(&mut self) -> i32;
    /// The order's `uid` (the site's owner).
    fn order_owner(&mut self) -> i32;
    /// The unit's player (the byte at `+0x9`).
    fn unit_player(&mut self) -> i32;
    /// `leaders[who]`'s `+0x8`.
    fn leader_id(&mut self) -> i32;
    /// `leaders[who]`'s `+0x74 + 4 * uid`.
    fn relation(&mut self) -> i32;
    /// `leadersc[uid]`'s `+0x74 + 4 * leader`.
    fn relation_const(&mut self) -> i32;
    /// `UnitData::order_type == NONE`.
    fn order_none(&mut self) -> bool;
    /// `unit_masks & 0x40000`.
    fn ai_driven(&mut self) -> bool;
    /// The unit's vslot `+0xf8`: the worker stance.
    fn stance(&mut self) -> i32;
    /// The site's vslot `+0x1c`.
    fn site_in_use(&mut self) -> bool;
    /// The site's data record, vslot `+0x90` of the object behind `+0x18`.
    fn is_gather_type(&mut self) -> bool;
    /// The site's `is(0x1a4)`: a University.
    fn is_university(&mut self) -> bool;
    /// `Unit::add_gather_order(idx, QUEUE_NEW, 0)`.
    fn add_gather_order(&mut self);
    /// `Unit::find_repair_spot`.
    fn find_repair_spot(&mut self);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// The site record's vslot `+0x4c`: finished.
    fn site_active(&mut self) -> bool;
    /// The site record's vslot `+0x188`.
    fn site_under_attack(&mut self) -> bool;
    /// The order's flag byte `& 4`, through its virtual base.
    fn order_action(&mut self) -> bool;
    /// The owner of the tile under the site (`-1` for none).
    fn tile_owner(&mut self) -> i32;
    /// `LeaderData::is_ally(leaders[uid], owner)`.
    fn leader_ally(&mut self, owner: i32) -> bool;
    /// The unit's vslot `+0x170` (`adjacent_to`) of the site.
    fn adjacent(&mut self) -> bool;
    /// `Group::action_swarm_around(idx, player, QUEUE_FIRST, REPAIR,
    /// action)` on a one-member stack group.
    fn swarm(&mut self, action: bool);
    /// `unit_masks & 1`: a decoy repairs nothing.
    fn decoy(&mut self) -> bool;
    /// The site record's vslot `+0x11c(0)`.
    fn hits(&mut self) -> i32;
    /// The site object's own vslot `+0x11c(0)`, asked once a priced good.
    fn good_hits(&mut self) -> i32;
    /// `constants->accel_construct`.
    fn accel_construct(&mut self) -> i32;
    /// The site record's vslot `+0x18c(1)`.
    fn construct_time(&mut self) -> i32;
    /// The site record's helpers byte (`+0x64`).
    fn helpers(&mut self) -> i32;
    /// The site record's flag `0x20`: a city building.
    fn is_city(&mut self) -> bool;
    /// The city's two bytes (`+0x5e`, `+0x5f`) differ: a captured city.
    fn captured(&mut self) -> bool;
    /// `LeaderData::has_tribe_bonus(0x10)`: the Koreans.
    fn korean(&mut self) -> bool;
    /// `constants->korean_repair`.
    fn korean_repair(&mut self) -> i32;
    /// `constants->korean_build_under_fire`.
    fn korean_under_fire(&mut self) -> i32;
    /// The site record's `+0x60 & 0x10`: freshly hit.
    fn fresh_hit(&mut self) -> bool;
    /// The helpers byte incremented.
    fn bump_helpers(&mut self);
    /// `game->frame`.
    fn frame(&mut self) -> i32;
    /// The site's vslot `+0x114`: the hit points it has.
    fn hits_left(&mut self) -> i32;
    /// `LeaderData::type_avail(good, 1) != 0`.
    fn good_avail(&mut self, good: usize) -> bool;
    /// The site's type's vslot `+0x78(good, uid, ..)`: the price in `good`.
    fn price(&mut self, good: usize) -> i32;
    /// The leader's decrypted bucket of `good`.
    fn bucket(&mut self, good: usize) -> i32;
    /// The leader's last-complaint frame (`+0x208`).
    fn feedback_stamp(&mut self) -> i32;
    fn set_feedback_stamp(&mut self, frame: i32);
    /// The console's player is the unit's.
    fn is_console_player(&mut self) -> bool;
    /// The complaint: `MessageWin::add_feedback` and the invalid-order
    /// sound. SEAM: the interface is not modelled.
    fn feedback(&mut self);
    /// The bucket of `good` down by `d` and its escrow down by `d`, floored
    /// at zero.
    fn charge(&mut self, good: usize, d: i32);
    /// The site's vslot `+0x168(amount, 0, 1)`: `Build::repair_damage`.
    fn repair_damage(&mut self, amount: i32);
}

/// **`Unit::do_repair@005ee420`'s steps** (item 3346, `docs/ORDERS.md`
/// §5.6): the gates, the re-swarm of a repairer out of reach, the period
/// and the amount, the price in whole units of each good (a single-precision
/// quotient, truncated), and the charge.
#[allow(clippy::needless_range_loop)]
pub fn do_repair_steps<V: DoRepairView>(v: &mut V) {
    /// `LAB_005eeb02`: the order dies, and the unit goes on to look for
    /// work — an AI repairer for the next repair, a human one for the site
    /// it has just mended (or the one it was refused).
    fn refused<V: DoRepairView>(v: &mut V) {
        v.kill_current_order();
        if v.ai_driven() {
            v.find_repair_spot();
            return;
        }
        let uid = v.order_owner();
        // The unit's own player, the unit between orders and on a gathering
        // stance, a site that is in use, gathered from and not a University.
        if uid == v.unit_player()
            && v.order_none()
            && v.stance() < 2
            && v.site_in_use()
            && v.is_gather_type()
            && !v.is_university()
        {
            v.add_gather_order();
        }
    }
    v.set_repair_anim();
    if v.site_damage() == 0 {
        refused(v);
        return;
    }
    let uid = v.order_owner();
    if uid != v.leader_id() && !(v.relation() == 2 && v.relation_const() == 2) {
        refused(v);
        return;
    }
    if !v.site_in_use() || !v.site_active() {
        v.kill_current_order();
        return;
    }
    if v.site_under_attack() && !v.order_action() {
        v.kill_current_order();
        return;
    }
    let owner = v.tile_owner();
    if owner >= 0 && owner != uid && !v.leader_ally(owner) {
        v.kill_current_order();
        return;
    }
    if !v.adjacent() {
        // The action bit is read before the kill.
        let action = v.order_action();
        v.kill_current_order();
        v.swarm(action);
        return;
    }
    if v.decoy() {
        return;
    }
    let amount = if !v.site_in_use() {
        frame_amount(v.frame(), 0x100)
    } else {
        let hits = v.hits();
        let accel = v.accel_construct();
        let construct_time = v.construct_time();
        let helpers = v.helpers();
        let city = v.is_city();
        let captured = city && v.captured();
        let korean = v.korean();
        let korean_repair = if korean { v.korean_repair() } else { 0 };
        let exempt = korean && v.korean_under_fire() != 0;
        let under_attack = !exempt && v.site_under_attack();
        let fresh_hit = !exempt && v.fresh_hit();
        let period = crate::cities::build::repair_period(&crate::cities::build::RepairTarget {
            helpers,
            construct_time,
            hits,
            accel,
            city,
            captured,
            korean_repair,
            under_attack,
            fresh_hit,
        });
        v.bump_helpers();
        if period < 1 {
            v.site_damage()
        } else {
            frame_amount(v.frame(), period)
        }
    };
    if amount == 0 {
        return;
    }
    let left = v.hits_left();
    let mut due = [0i32; 6];
    for g in 0..6 {
        if !v.good_avail(g) {
            continue;
        }
        let p = v.price(g);
        if p == 0 {
            continue;
        }
        let hits = v.good_hits();
        let q = Single::from_i32(hits).divss(Single::from_i32(p));
        let before = Single::from_i32(left).divss(q).to_i32();
        let after = Single::from_i32(amount.wrapping_add(left))
            .divss(q)
            .to_i32();
        if before < after {
            due[g] = 1;
        }
        if v.bucket(g) < due[g] {
            v.kill_current_order();
            let frame = v.frame();
            if frame.wrapping_sub(v.feedback_stamp()) < 0x97 {
                return;
            }
            v.set_feedback_stamp(frame);
            if v.is_console_player() {
                v.feedback();
            }
            return;
        }
    }
    for g in 0..6 {
        if v.good_avail(g) {
            v.charge(g, due[g]);
        }
    }
    v.repair_damage(amount);
}

/// `frame * 256 / period - (frame * 256 - 256) / period`, the signed
/// divides of `Unit::do_repair`.
pub(super) fn frame_amount(frame: i32, period: i32) -> i32 {
    let f = frame.wrapping_mul(0x100);
    (f / period).wrapping_sub(f.wrapping_sub(0x100) / period)
}

/// [`DoRepairView`] over the live [`Sim`].
pub(super) struct SimDoRepair<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    pub(super) b: usize,
    pub(super) action: bool,
    pub(super) frame: i32,
}

impl DoRepairView for SimDoRepair<'_> {
    fn set_repair_anim(&mut self) {
        // §5.6's first line, and unlike `do_build`'s it is **ahead of every
        // gate**: a repairer is put on `CHAR_REPAIR` even on the frame the
        // order dies. Same reason as `do_build`'s (`docs/ANIM.md` §4.6).
        self.s.set_anim(self.u, anim::ANIM_REPAIR, false, true);
    }
    fn site_damage(&mut self) -> i32 {
        self.s.buildings[self.b].damage
    }
    fn order_owner(&mut self) -> i32 {
        i32::from(self.s.buildings[self.b].owner)
    }
    fn unit_player(&mut self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn leader_id(&mut self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn relation(&mut self) -> i32 {
        let (a, b) = (self.s.units[self.u].owner, self.s.buildings[self.b].owner);
        if self.s.allied_with(a, b) { 2 } else { 0 }
    }
    fn relation_const(&mut self) -> i32 {
        let (a, b) = (self.s.units[self.u].owner, self.s.buildings[self.b].owner);
        if self.s.allied_with(b, a) { 2 } else { 0 }
    }
    fn order_none(&mut self) -> bool {
        self.s.order_type(self.u) == index::NONE
    }
    fn ai_driven(&mut self) -> bool {
        self.s.ai_driven(self.s.units[self.u].owner)
    }
    fn stance(&mut self) -> i32 {
        i32::from(self.s.units[self.u].stance)
    }
    fn site_in_use(&mut self) -> bool {
        self.s.buildings.get(self.b).is_some_and(|bd| bd.alive)
    }
    fn is_gather_type(&mut self) -> bool {
        self.s.is_gather_type(self.b)
    }
    fn is_university(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::University
    }
    fn add_gather_order(&mut self) {
        self.s
            .add_gather_order(self.u, self.b, QueuePos::New, false);
    }
    fn find_repair_spot(&mut self) {
        self.s.find_repair_spot(self.u);
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn site_active(&mut self) -> bool {
        self.s.buildings[self.b].active
    }
    fn site_under_attack(&mut self) -> bool {
        self.s.buildings[self.b].is_under_attack()
    }
    fn order_action(&mut self) -> bool {
        self.action
    }
    fn tile_owner(&mut self) -> i32 {
        match self.s.world.owner_at(self.s.buildings[self.b].pos) {
            crate::Owner::Player(p) => i32::from(p),
            _ => -1,
        }
    }
    fn leader_ally(&mut self, owner: i32) -> bool {
        self.s.is_ally(
            self.s.buildings[self.b].owner,
            owner as crate::world::Player,
        )
    }
    fn adjacent(&mut self) -> bool {
        self.s.adjacent_to(self.u, self.b)
    }
    fn swarm(&mut self, action: bool) {
        // The same one-member `Group` insert dance as `do_build`'s
        // (`Group::action_swarm_around@0070fbe0`, `docs/GROUPS.md` §17):
        // `Unit::do_repair@005ee420` clears two groups, adds the unit to
        // one and swarms at `QUEUE_FIRST` (item 3346).
        let (u, b) = (self.u, self.b);
        let who = self.s.units[u].owner;
        let mut g = crate::groups::group::Group::stack(who);
        self.s.group_add(&mut g, u);
        let insert = self.s.group_set_up_insert(&g);
        self.s.group_action_halt(&g, 0);
        self.s.swarm_around(u, b, Body::Repair(b), action);
        self.s.group_finish_insert(&g, insert);
    }
    fn decoy(&mut self) -> bool {
        self.s.units[self.u].decoy
    }
    fn hits(&mut self) -> i32 {
        self.s.buildings[self.b].hits_now()
    }
    fn good_hits(&mut self) -> i32 {
        self.s.buildings[self.b].hits_now()
    }
    fn accel_construct(&mut self) -> i32 {
        self.s.tables.tuning.accel_construct
    }
    fn construct_time(&mut self) -> i32 {
        self.s.construct_time_of(self.b)
    }
    fn helpers(&mut self) -> i32 {
        self.s.buildings[self.b].helpers
    }
    fn is_city(&mut self) -> bool {
        let bd = &self.s.buildings[self.b];
        matches!(bd.city, Some(c) if self.s.building_is_city(self.b) && self.s.cities.cities[c].alive)
    }
    fn captured(&mut self) -> bool {
        let bd = &self.s.buildings[self.b];
        bd.city
            .is_some_and(|c| self.s.cities.cities[c].race != Some(bd.owner))
    }
    fn korean(&mut self) -> bool {
        self.s.cities.nation[self.s.units[self.u].owner as usize].koreans
    }
    fn korean_repair(&mut self) -> i32 {
        self.s.tables.tuning.korean_repair
    }
    fn korean_under_fire(&mut self) -> i32 {
        self.s.tables.tuning.korean_build_under_fire
    }
    fn fresh_hit(&mut self) -> bool {
        self.s.buildings[self.b].under_attack & 0x1 != 0
    }
    fn bump_helpers(&mut self) {
        self.s.buildings[self.b].helpers += 1;
    }
    fn frame(&mut self) -> i32 {
        self.frame
    }
    fn hits_left(&mut self) -> i32 {
        let bd = &self.s.buildings[self.b];
        bd.hits_now() - bd.damage
    }
    fn good_avail(&mut self, good: usize) -> bool {
        self.s.economy.holdings[self.s.units[self.u].owner as usize].available[good]
    }
    fn price(&mut self, good: usize) -> i32 {
        // The site's owner is the `uid` the type's price is asked for
        // (`vslot +0x78(good, uid, ..)`), not the repairer: an ally
        // mends at the owner's price (item 3346).
        let bd = &self.s.buildings[self.b];
        bd.ty
            .map_or(0, |t| self.s.building_price(bd.owner, t)[good])
    }
    fn bucket(&mut self, good: usize) -> i32 {
        self.s.economy.ledgers[self.s.units[self.u].owner as usize].bucket[good]
    }
    /// `repair_stamp` (`+0x208`) on the repairer's owner's slot
    /// (`docs/LEADERS.md` §15.2, item 4176): 0 at init, written for every
    /// player before the console test.
    fn feedback_stamp(&mut self) -> i32 {
        self.s.repair_stamp(self.s.units[self.u].owner) as i32
    }
    fn set_feedback_stamp(&mut self, frame: i32) {
        let who = self.s.units[self.u].owner;
        self.s.set_repair_stamp(who, i64::from(frame));
    }
    fn is_console_player(&mut self) -> bool {
        false
    }
    fn feedback(&mut self) {}
    fn charge(&mut self, good: usize, d: i32) {
        // `Unit::do_repair@005ee420`'s charge draws the escrow down by the
        // same amount, floored at zero (`docs/AI.md` §98).
        let ledger = &mut self.s.economy.ledgers[self.s.units[self.u].owner as usize];
        ledger.bucket[good] -= d;
        ledger.escrow[good] = (ledger.escrow[good] - d).max(0);
    }
    fn repair_damage(&mut self, amount: i32) {
        self.s.repair_building(self.b, amount);
    }
}

/// What `Unit::do_gather@005ef2a0` asks of its unit, its gather order, the
/// site and the world, in the order it asks (item 3356). [`SimDoGather`]
/// answers from this crate's state; the sweep answers from the emulated
/// original's windows (`crates/sim/tests/sweep/unit_do_gather_005ef2a0.rs`).
///
/// The site is the order's `(idx, uid)` pair; the view holds it, so no
/// method takes one. Coordinates are the world's own (decoded: the unit
/// stores `x ^ 0x63637`), tiles are `div_3_table[c >> 6]`.
pub trait DoGatherView {
    /// The order's `uid` (`+0xc`): the site's owner.
    fn order_owner(&mut self) -> i32;
    /// The unit's player (the byte at `+0x9`).
    fn unit_player(&mut self) -> i32;
    /// The site's vslot `+0xb0` record, slot `+0xc`: in use.
    fn site_alive(&mut self) -> bool;
    /// The same record's slot `+0x4c`: finished.
    fn site_active(&mut self) -> bool;
    /// The order's `been_there` byte (`+0x27`).
    fn been_there(&mut self) -> bool;
    fn set_been_there(&mut self, on: bool);
    /// The order's `build_type` (`+0x1c`): a `TypeIndex`, `0x1a1` farm,
    /// `0x1a2` woodcutter, `0x1a3` mine, `0x1a6` oil platform.
    fn build_type(&mut self) -> i32;
    /// `BuildData::is_gathered_by(unit)`.
    fn is_gathered_by(&mut self) -> bool;
    /// The site record's gatherer cap, the signed byte at `+0x80`.
    fn max_gatherers(&mut self) -> i32;
    /// `BuildData::num_gatherers(0, 0)`.
    fn num_gatherers(&mut self) -> i32;
    /// The unit's type record, `+0x218`: its domain.
    fn unit_domain(&mut self) -> i32;
    /// The site's type record, `+0x218`.
    fn site_domain(&mut self) -> i32;
    /// `Build::add_gatherer(unit, player)`.
    fn add_gatherer(&mut self);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// `Unit::add_think_order`.
    fn add_think_order(&mut self);
    /// The site's type, slot `+0x94`: flat.
    fn site_flat(&mut self) -> bool;
    /// `WallData::covers_tile(unit's tile)`.
    fn covers_unit_tile(&mut self) -> bool;
    /// The unit's position.
    fn unit_pos(&mut self) -> (i32, i32);
    /// The site object's position.
    fn site_pos(&mut self) -> (i32, i32);
    /// `Unit::add_move_order(x, y, 1, 0, QUEUE_FIRST, 0, y, -1, -1)`.
    fn add_move_order(&mut self, x: i32, y: i32);
    /// The same call from the spot search's tail, whose unused seventh
    /// argument is the caller's leftover `ecx` and not `y`.
    fn add_move_order_spot(&mut self, x: i32, y: i32);
    /// The site record's city (`+0x72`), `-1` for none.
    fn site_city(&mut self) -> i32;
    /// The site record's farm slot (`+0x78`).
    fn farm_index(&mut self) -> i32;
    /// `game->frame`.
    fn frame(&mut self) -> i32;
    /// The unit's number (the short at `+0xa`).
    fn unit_id(&mut self) -> i32;
    /// `Unit::do_non_flat_gather(order)`.
    fn non_flat_gather(&mut self);
    /// The unit's vslot `+0x170` of the site: `adjacent_to`.
    fn adjacent(&mut self) -> bool;
    /// The smaller side of the site record's footprint (`+0x234`/`+0x238` of
    /// its `+0x18`, through slot `+0xb0`).
    fn site_min_size(&mut self) -> i32;
    /// The footprint (`+0x234`, `+0x238`) of the site record's type through
    /// slot `+0xac`.
    fn site_size(&mut self) -> (i32, i32);
    /// `UnitData::can_transport`.
    fn can_transport(&mut self) -> bool;
    /// `UnitType::find_nearby_spot(site, out, dist, -1, 0, angle,
    /// FILTER_NOT_ME, unit, player, 0, 0, -1, 0, -1)`: the spot, or `None`
    /// when the search answers non-zero.
    fn find_spot(&mut self, dist: i32, angle: i32) -> Option<(i32, i32)>;
    /// `unit_flags |= 0x10`.
    fn set_cant_reach(&mut self);
    /// `WorldData::get_tregion` of the site's tile and of the unit's differ.
    fn region_differs(&mut self) -> bool;
    /// `leaders[player].flags |= 0x2000000`.
    fn mark_leader(&mut self);
    /// `ObjectData::is_scholar`.
    fn is_scholar(&mut self) -> bool;
    /// `Unit::go_inside(idx, uid, 0)`.
    fn go_inside(&mut self);
    /// `Build::check_gatherers`.
    fn check_gatherers(&mut self);
    /// The site record `is(OILPLATFORM)`: a `Build` asks its type (slot
    /// `+0x60`), any other record its own slot `+0xb8(0x1a6, 0)`.
    fn is_platform(&mut self) -> bool;
    /// `ObjectData::can_carry(GROUND)`.
    fn can_carry_ground(&mut self) -> bool;
    /// `Unit::same_damage` on the unit's first passenger, when it has one
    /// (the short at `+0x28`, not negative).
    fn carrier_same_damage(&mut self);
    /// The unit's vslot `+0x158` (`Object::die(0, -1, 0.0)`).
    fn die(&mut self);
    /// `unit_masks & 0x40000`.
    fn ai_driven(&mut self) -> bool;
    /// `LeaderData::get_diff`.
    fn leader_diff(&mut self) -> i32;
    /// The site record's damage (`+0x24`).
    fn site_damage(&mut self) -> i32;
    /// The site record under attack: a `Build`'s `+0x60 & 0x20`, any other
    /// record's slot `+0x188`.
    fn site_under_attack(&mut self) -> bool;
    /// `cities[player][city].city_flags & 2`: `CITY_UNDER_ATTACK`.
    fn city_under_attack(&mut self) -> bool;
    /// `Unit::add_repair_order(idx, uid, QUEUE_NEW, 0)`.
    fn add_repair_order(&mut self);
    /// The site record's `+0x60 & 0x800`: a worker has been counted.
    fn site_counted(&mut self) -> bool;
    /// `+0x7a += 1` and `+0x60 |= 0x800`.
    fn bump_site_counter(&mut self);
    /// Guy 0's current animation, the byte at `+0x9c`.
    fn cur_anim(&mut self) -> i32;
    /// `Unit::set_anim(anim, 0, 1)`.
    fn set_anim(&mut self, anim: i32);
    /// `Unit::set_angle(0x40000000, ., 0)`: face east.
    fn face_east(&mut self);
    /// `Unit::set_new_location(x, y, 1, 1)`.
    fn set_new_location(&mut self, x: i32, y: i32);
    /// `FarmsData::get_farm_type(farm)`.
    fn farm_type(&mut self, farm: i32) -> i32;
    /// `WallData::tile_corner`: the footprint's corner tile.
    fn tile_corner(&mut self) -> (i32, i32);
    /// The unit's tile.
    fn unit_tile(&mut self) -> (i32, i32);
    /// The farm's cell `(fx, fy)`: `status[fx][fy]`.
    fn farm_cell(&mut self, farm: i32, fx: i32, fy: i32) -> i32;
    /// `Farms::grow(farm, fy, fx)`.
    fn farm_grow(&mut self, farm: i32, fy: i32, fx: i32);
    /// `Farms::snip(farm, fy, fx)`.
    fn farm_snip(&mut self, farm: i32, fy: i32, fx: i32);
    /// `GameAccess::rnd(n)`.
    fn rnd(&mut self, n: i32) -> i32;
    /// The "Invalid farming location?" report, unless `ignore_always` is
    /// set. SEAM: the dialog is not modelled; the original's `int3` on a
    /// non-zero answer is a debugger break.
    fn invalid_farm_cell(&mut self);
}

/// `TypeIndex` values the gather order names.
pub(super) const GATHER_FARM: i32 = 0x1a1;

pub(super) const GATHER_WOODCUTTER: i32 = 0x1a2;

pub(super) const GATHER_MINE: i32 = 0x1a3;

pub(super) const GATHER_PLATFORM: i32 = 0x1a6;

/// **`Unit::do_gather@005ef2a0`'s steps** (item 3356, `docs/ORDERS.md`
/// §6.3): the gate on the order, the chain, the arrival, the stand, the
/// farm. `docs/ORDERS.md` §6.5 has the farm cell.
pub fn do_gather_steps<V: DoGatherView>(v: &mut V) {
    /// The site walk shared by every give-up and by the retargets:
    /// `LAB_005ef3b1` — a flat site the unit is not on is walked to.
    fn site_walk<V: DoGatherView>(v: &mut V) {
        if !v.site_flat() || v.covers_unit_tile() {
            return;
        }
        let (x, y) = v.site_pos();
        v.add_move_order(x, y);
    }
    /// `LAB_005ef4dc`: the order dies, the unit thinks, and walks back to a
    /// flat site.
    fn fail<V: DoGatherView>(v: &mut V) {
        v.kill_current_order();
        v.add_think_order();
        site_walk(v);
    }
    /// `LAB_005ef63e` / `LAB_005ef68f`: the order dies and the unit is
    /// marked unable to reach.
    fn cant_reach<V: DoGatherView>(v: &mut V) {
        v.kill_current_order();
        v.set_cant_reach();
    }
    /// The unit's phase against a 256-frame period: the original masks
    /// with `0x800000ff` and tests a negative remainder by hand, which is
    /// the low byte either way.
    fn on_phase(sum: i32) -> bool {
        sum & 0xff == 0
    }

    // `5ef2de`: the order is the unit's player's, and its site in use and
    // finished.
    if v.order_owner() != v.unit_player() {
        v.kill_current_order();
        return;
    }
    if !v.site_alive() || !v.site_active() {
        v.kill_current_order();
        return;
    }
    let ty = v.build_type();
    if !v.been_there() && !v.is_gathered_by() {
        // The retry for a unit refused at issue (`5ef338`): room in the
        // chain, and the unit put into it. An oil platform and any unit
        // that is not a land unit skip the chain and go on to the arrival.
        let max = v.max_gatherers();
        if v.num_gatherers() >= max {
            fail(v);
            return;
        }
        if ty != GATHER_PLATFORM && v.unit_domain() == 0 {
            v.add_gatherer();
            if !v.is_gathered_by() {
                fail(v);
                return;
            }
        }
    }
    // `5ef4f5`
    let mut farm = -1;
    if ty == GATHER_FARM {
        if v.site_city() < 0 {
            fail(v);
            return;
        }
        farm = v.farm_index();
    }
    if ty == GATHER_MINE || ty == GATHER_WOODCUTTER {
        // Wood and ore: the over-full check once in 128 frames, and the
        // non-flat machine otherwise.
        if on_phase_128(v.frame().wrapping_add(v.unit_id().wrapping_mul(4))) {
            let max = v.max_gatherers();
            if v.num_gatherers() > max {
                v.kill_current_order();
                v.add_think_order();
                site_walk(v);
                return;
            }
        }
        v.non_flat_gather();
        return;
    }
    if ty != GATHER_FARM {
        if !v.adjacent() {
            // `5ef73b`: walk to a spot beside the site, or to the site
            // itself for a transport that crosses water.
            let dist = v.site_min_size().wrapping_mul(0x60).wrapping_add(0x30);
            let (x, y);
            if v.site_domain() == v.unit_domain() || !v.can_transport() {
                let (ux, uy) = v.unit_pos();
                let (sx, sy) = v.site_pos();
                let angle = crate::movement::find_angle(ux.wrapping_sub(sx), uy.wrapping_sub(sy)).0;
                let Some(spot) = v.find_spot(dist, angle) else {
                    cant_reach(v);
                    return;
                };
                (x, y) = spot;
            } else {
                (x, y) = v.site_pos();
            }
            if !v.is_gathered_by() && !v.is_platform() {
                v.set_been_there(false);
                return;
            }
            v.add_move_order_spot(x, y);
            return;
        }
    } else if !v.covers_unit_tile() {
        // `5ef5a8`: a farm the unit is not on. Across a region boundary
        // without a transport, or over the cap, it cannot reach.
        if v.region_differs() && !v.can_transport() {
            cant_reach(v);
            return;
        }
        let max = v.max_gatherers();
        if v.num_gatherers() > max {
            cant_reach(v);
            return;
        }
        if v.unit_domain() == 0 && !v.is_gathered_by() {
            v.set_been_there(false);
            return;
        }
        let (x, y) = v.site_pos();
        v.add_move_order(x, y);
        return;
    }
    // `5ef939`: at the site.
    if !v.been_there() {
        let max = v.max_gatherers();
        if v.num_gatherers() > max {
            cant_reach(v);
            return;
        }
        v.set_been_there(true);
        v.mark_leader();
        if v.is_scholar() {
            v.go_inside();
            v.check_gatherers();
            v.kill_current_order();
            return;
        }
        if v.is_platform() {
            if v.can_carry_ground() {
                // `5efa4a`: a transport goes in and dies, its passenger
                // the platform's.
                v.carrier_same_damage();
                v.go_inside();
                v.kill_current_order();
                v.die();
                v.check_gatherers();
                return;
            }
            v.go_inside();
            v.check_gatherers();
            v.kill_current_order();
            return;
        }
        v.check_gatherers();
    }
    // `5efb20`: an AI's idle worker mends a damaged site once in 256 frames.
    if v.ai_driven() {
        let sum = v
            .unit_id()
            .wrapping_add(v.frame())
            .wrapping_add(v.unit_player());
        if on_phase(sum) && v.leader_diff() > 1 && v.site_damage() != 0 && !v.site_under_attack() {
            let city = v.site_city();
            if city >= 0 && !v.city_under_attack() {
                v.add_repair_order();
                return;
            }
        }
    }
    if ty != GATHER_FARM {
        // The oil well: counted once, then stood at beside the platform.
        if !v.site_counted() {
            v.bump_site_counter();
        }
        if v.cur_anim() == i32::from(anim::ANIM_FARM) {
            return;
        }
        v.set_anim(i32::from(anim::ANIM_FARM));
        v.face_east();
        let (sx, sy) = v.site_pos();
        v.set_new_location(sx.wrapping_add(OILWELL_OFFSET), sy);
        return;
    }
    // `5efcfb`: the farm.
    let kind = v.farm_type(farm);
    let (xs, ys) = v.site_size();
    let (cx, cy) = v.tile_corner();
    if kind == 1 {
        // A pasture sows nothing: on its own 256-frame phase it walks to a
        // tile of the inner part.
        v.set_anim(i32::from(anim::SOW));
        let sum = v
            .unit_id()
            .wrapping_mul(PASTURE_STRIDE)
            .wrapping_add(v.frame())
            .wrapping_add(v.unit_player());
        if !on_phase(sum) {
            return;
        }
        let rx = v.rnd(xs / 2);
        let ry = v.rnd(ys / 2);
        let x = cx
            .wrapping_add(1)
            .wrapping_add(rx)
            .wrapping_mul(TILE)
            .wrapping_add(HALF_TILE);
        let y = cy
            .wrapping_add(1)
            .wrapping_add(ry)
            .wrapping_mul(TILE)
            .wrapping_add(HALF_TILE);
        v.add_move_order(x, y);
        return;
    }
    let (ux, uy) = v.unit_tile();
    let (mut fx, mut fy) = (ux.wrapping_sub(cx), uy.wrapping_sub(cy));
    if fy < 0 || fx < 0 || fy >= ys || fx >= xs {
        fx = 1;
        fy = 1;
        v.invalid_farm_cell();
    }
    // `status[fx][fy]` — the cell in the farm's own record, the byte at
    // `(farm * 0x30 + fx) * 4 + fy + 0xac`.
    match v.farm_cell(farm, fx, fy) {
        0 if v.cur_anim() == i32::from(anim::REAP) => {}
        0 | 1 => {
            v.set_anim(i32::from(anim::SOW));
            v.farm_grow(farm, fy, fx);
            return;
        }
        2 if v.cur_anim() != i32::from(anim::SOW) => {
            v.set_anim(i32::from(anim::REAP));
            v.farm_snip(farm, fy, fx);
            return;
        }
        3 => {
            v.set_anim(i32::from(anim::REAP));
            return;
        }
        _ => {}
    }
    // Empty under a reaper, ripe under a sower, or a byte the switch does
    // not name: a new tile of the crop.
    let rx = v.rnd(xs);
    let ry = v.rnd(ys);
    let x = cx
        .wrapping_add(rx)
        .wrapping_mul(TILE)
        .wrapping_add(HALF_TILE);
    let y = cy
        .wrapping_add(ry)
        .wrapping_mul(TILE)
        .wrapping_add(HALF_TILE);
    v.add_move_order(x, y);
}

/// `test al, 0x7f` of `frame + id * 4`.
pub(super) fn on_phase_128(sum: i32) -> bool {
    sum & 0x7f == 0
}

/// [`DoGatherView`] over the live [`Sim`].
pub(super) struct SimDoGather<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    pub(super) b: usize,
    /// The gather order, as the original holds a pointer to it for the
    /// whole of the function (see [`Sim::store_gather`]).
    pub(super) g: GatherOrder,
    pub(super) frame: i32,
    /// The farmer's pair of draws has been marked.
    pub(super) drew: bool,
}

impl DoGatherView for SimDoGather<'_> {
    fn order_owner(&mut self) -> i32 {
        i32::from(self.s.buildings[self.b].owner)
    }
    fn unit_player(&mut self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn site_alive(&mut self) -> bool {
        self.s.buildings[self.b].alive
    }
    fn site_active(&mut self) -> bool {
        self.s.buildings[self.b].active
    }
    fn been_there(&mut self) -> bool {
        self.g.been_there
    }
    fn set_been_there(&mut self, on: bool) {
        self.g.been_there = on;
        self.s.store_gather(self.u, self.g);
    }
    fn build_type(&mut self) -> i32 {
        match self.s.building_ident(self.b) {
            Ident::Farm => GATHER_FARM,
            Ident::Woodcutter => GATHER_WOODCUTTER,
            Ident::Mine => GATHER_MINE,
            Ident::OilPlatform => GATHER_PLATFORM,
            _ => 0,
        }
    }
    fn is_gathered_by(&mut self) -> bool {
        self.s.is_gathered_by(self.b, self.u)
    }
    fn max_gatherers(&mut self) -> i32 {
        // No cap is a cap nothing reaches.
        self.s.buildings[self.b].gather_max.unwrap_or(i32::MAX)
    }
    fn num_gatherers(&mut self) -> i32 {
        self.s.num_gatherers(self.b, false, false)
    }
    fn unit_domain(&mut self) -> i32 {
        match self.s.units[self.u].kind.domain {
            crate::territory::attrition::Domain::Land => 0,
            crate::territory::attrition::Domain::Sea => 1,
            crate::territory::attrition::Domain::Air => 2,
        }
    }
    fn site_domain(&mut self) -> i32 {
        self.s.buildings[self.b]
            .ty
            .map_or(0, |t| match self.s.tables.build_types[t].domain() {
                crate::cities::build::BuildDomain::Land => 0,
                crate::cities::build::BuildDomain::Water => 1,
                crate::cities::build::BuildDomain::Both => 2,
            })
    }
    fn add_gatherer(&mut self) {
        self.s.add_gatherer(self.b, self.u);
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn add_think_order(&mut self) {
        self.s.add_think_order(self.u);
    }
    fn site_flat(&mut self) -> bool {
        self.s.is_flat(self.b)
    }
    fn covers_unit_tile(&mut self) -> bool {
        self.s.covers_tile(self.b, self.s.units[self.u].pos.tile())
    }
    fn unit_pos(&mut self) -> (i32, i32) {
        let p = self.s.units[self.u].pos;
        (p.x, p.y)
    }
    fn site_pos(&mut self) -> (i32, i32) {
        let p = self.s.buildings[self.b].pos;
        (p.x, p.y)
    }
    fn add_move_order(&mut self, x: i32, y: i32) {
        self.s.add_move_order(
            self.u,
            Pos::new(x, y),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
    }
    fn add_move_order_spot(&mut self, x: i32, y: i32) {
        self.add_move_order(x, y);
    }
    fn site_city(&mut self) -> i32 {
        self.s.buildings[self.b].city.map_or(-1, |c| c as i32)
    }
    fn farm_index(&mut self) -> i32 {
        // A farm's record is its own building in this crate.
        self.b as i32
    }
    fn frame(&mut self) -> i32 {
        self.frame
    }
    fn unit_id(&mut self) -> i32 {
        i32::from(self.s.units[self.u].index)
    }
    fn non_flat_gather(&mut self) {
        self.s.do_non_flat_gather(self.u, self.g);
    }
    fn adjacent(&mut self) -> bool {
        self.s.adjacent_to(self.u, self.b)
    }
    fn site_min_size(&mut self) -> i32 {
        let (xs, ys) = self.site_size();
        xs.min(ys)
    }
    fn site_size(&mut self) -> (i32, i32) {
        self.s.buildings[self.b].ty.map_or((1, 1), |t| {
            (
                self.s.tables.build_types[t].x_size,
                self.s.tables.build_types[t].y_size,
            )
        })
    }
    fn can_transport(&mut self) -> bool {
        self.s.unit_can_transport(self.u)
    }
    fn find_spot(&mut self, dist: i32, angle: i32) -> Option<(i32, i32)> {
        let at = self.s.buildings[self.b].pos;
        self.s
            .find_nearby_spot(self.u, at, dist, -1, 0, Angle(angle), None)
            .map(|p| (p.x, p.y))
    }
    fn set_cant_reach(&mut self) {
        self.s.units[self.u].cant_reach = true;
    }
    fn region_differs(&mut self) -> bool {
        let site = self
            .s
            .world
            .tregion_alt(self.s.buildings[self.b].pos.tile());
        let unit = self.s.world.tregion_alt(self.s.units[self.u].pos.tile());
        site != unit
    }
    fn mark_leader(&mut self) {
        let who = self.s.units[self.u].owner as usize;
        self.s.economy.ledgers[who].dirty = true;
    }
    fn is_scholar(&mut self) -> bool {
        self.s.worker_of(self.u) == Worker::Scholar
    }
    fn go_inside(&mut self) {
        self.s.go_inside(self.u, self.b);
    }
    fn check_gatherers(&mut self) {
        self.s.check_gatherers(self.b);
    }
    fn is_platform(&mut self) -> bool {
        self.s.building_ident(self.b) == Ident::OilPlatform
    }
    fn can_carry_ground(&mut self) -> bool {
        self.s.can_carry_ground(self.u)
    }
    fn carrier_same_damage(&mut self) {
        self.s.boat_same_damage(self.u);
    }
    fn die(&mut self) {
        self.s.boat_dies_into(self.u, self.b);
    }
    fn ai_driven(&mut self) -> bool {
        self.s.unit_ai_bit(self.s.units[self.u].owner)
    }
    fn leader_diff(&mut self) -> i32 {
        self.s.ai_difficulty()
    }
    fn site_damage(&mut self) -> i32 {
        self.s.buildings[self.b].damage
    }
    fn site_under_attack(&mut self) -> bool {
        self.s.buildings[self.b].is_under_attack()
    }
    fn city_under_attack(&mut self) -> bool {
        self.s.buildings[self.b]
            .city
            .is_some_and(|c| self.s.cities.cities[c].no_heal)
    }
    fn add_repair_order(&mut self) {
        self.s
            .add_repair_order(self.u, self.b, QueuePos::New, false);
    }
    fn site_counted(&mut self) -> bool {
        self.s.buildings[self.b].gather_bumped
    }
    fn bump_site_counter(&mut self) {
        self.s.buildings[self.b].recharging += 1;
        self.s.buildings[self.b].gather_bumped = true;
    }
    fn cur_anim(&mut self) -> i32 {
        i32::from(
            self.s.units[self.u]
                .guys
                .first()
                .map_or(anim::DEFAULT, |g| g.anim),
        )
    }
    fn set_anim(&mut self, a: i32) {
        self.s.set_anim(self.u, a as i8, false, true);
    }
    fn face_east(&mut self) {
        self.s.units[self.u]
            .movement
            .set_heading(Angle(0x4000_0000));
    }
    fn set_new_location(&mut self, x: i32, y: i32) {
        // `world.accepts` is this crate's own guard: the original's
        // `set_new_location` has none, and the oil-well stand is inside
        // the map.
        let stand = Pos::new(x, y);
        if self.s.world.accepts(stand) {
            let from = self.s.units[self.u].pos;
            self.s.set_new_location(self.u, stand, true);
            // `do_gather`'s `set_new_location(…, 1, 1)`: the whole disc,
            // not the ring (`docs/VISION.md` §6).
            self.s.moved_to(self.u, from, false);
        }
    }
    fn farm_type(&mut self, _farm: i32) -> i32 {
        i32::from(self.s.buildings[self.b].farm.farm_type & farms::ANIMAL_FARM)
    }
    fn tile_corner(&mut self) -> (i32, i32) {
        let c = self.s.buildings[self.b].ty.map_or(Pos::new(0, 0), |t| {
            self.s.tile_corner(t, self.s.buildings[self.b].pos)
        });
        (c.x, c.y)
    }
    fn unit_tile(&mut self) -> (i32, i32) {
        let t = self.s.units[self.u].pos.tile();
        (t.x, t.y)
    }
    fn farm_cell(&mut self, _farm: i32, fx: i32, fy: i32) -> i32 {
        // **`dx * 4 + dy`, not `dy * 4 + dx`.** `FarmStruct::status` is a
        // `uchar[4][4]` and the farmer's cell is `status[dx][dy]`: both
        // `do_gather`'s own switch (`005eff54`, `(dx + farm·0x30)·4 + 0xac +
        // dy`) and `Farms::grow@008d91c0`'s write (`(farm·0x30 + param_3)·4
        // + 0xac + param_2`, called `(dy, dx)`) land on the same byte, and
        // it is the transpose of what this had. Invisible for a hundred
        // frames — every starting farmer stands on `(2, 2)`, which is its
        // own transpose — and then wrong the moment one re-picks a cell
        // (`docs/ORDERS.md` §6.5).
        i32::from(self.s.buildings[self.b].farm.state[(fx * 4 + fy) as usize])
    }
    fn farm_grow(&mut self, _farm: i32, fy: i32, fx: i32) {
        self.s.buildings[self.b].farm.grow((fx * 4 + fy) as usize);
    }
    fn farm_snip(&mut self, _farm: i32, fy: i32, fx: i32) {
        self.s.buildings[self.b].farm.snip((fx * 4 + fy) as usize);
    }
    fn rnd(&mut self, n: i32) -> i32 {
        // The farmer's cell re-pick and the herder's: one mark, the pair
        // (`SITE_FARM_CELL`).
        if !self.drew {
            self.drew = true;
            self.s.mark(SITE_FARM_CELL);
        }
        self.s.rnd(n)
    }
    fn invalid_farm_cell(&mut self) {}
}

/// **`UnitData::is_gathering_at@00608880`'s steps** (item 2287): a scholar
/// (`0x34`/`0x35`) off the map is gathering where it is inside
/// (`get_inside`), and, when `strict`, nowhere else. Then a worker type
/// on the map (`is_on_map`, `inside_up`'s sign bit, asked again) whose
/// action is a `GATHER` (`get_type() == 7`) naming this building answers
/// 1, or, `strict`, the order's arrived byte (`+0x27`).
pub(crate) fn gathering_at_steps(
    scholar: bool,
    mut on_map: impl FnMut() -> bool,
    inside_here: impl FnOnce() -> bool,
    strict: bool,
    worker: bool,
    gather: impl FnOnce() -> Option<(bool, i32)>,
) -> i32 {
    if scholar && !on_map() {
        if inside_here() {
            return 1;
        }
        if strict {
            return 0;
        }
    }
    if !on_map() || !worker {
        return 0;
    }
    match gather() {
        Some((true, arrived)) => {
            if strict {
                arrived
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// **`Build::check_gatherers@0062f710`'s prune** (item 2287): an empty
/// chain (`head < 0`) answers `-1` untouched. From the head, a member that
/// is not kept (in use, vslot `+8`; `is_gathering_at(this, 0)`; in use and
/// on the map, `+8` and `+0xbc`, of its unit record) is unlinked — the
/// head takes its link and its own link (`+0x92`) goes to `-1` — until a
/// keeper; past it, a dropped member is stepped over, the keeper before it
/// taking its link and **its own link left as it was**. The answer is the
/// head as it stands.
pub(crate) fn prune_chain<C>(
    cx: &mut C,
    head: &mut i32,
    next: impl Fn(&mut C, i32) -> i32,
    mut set_next: impl FnMut(&mut C, i32, i32),
    mut keep: impl FnMut(&mut C, i32) -> bool,
) -> i32 {
    if *head < 0 {
        return -1;
    }
    while *head >= 0 {
        if keep(cx, *head) {
            break;
        }
        let n = next(cx, *head);
        set_next(cx, *head, -1);
        *head = n;
    }
    if *head < 0 {
        return *head;
    }
    let mut prev = *head;
    loop {
        let n = next(cx, prev);
        if n < 0 {
            return *head;
        }
        if keep(cx, n) {
            prev = n;
        } else {
            let after = next(cx, n);
            set_next(cx, prev, after);
        }
    }
}

/// `Object::adjacent_to@00651f40` in the original's own order. `live` and
/// `target_live` are the two slots' `+8 & 1`; `sea` the type's `+0x218 ==
/// 1`; `is_unit` vslot `+0x18`, asked only of a sea type; `boat` the
/// unit's own type read back through the units table — its `+0x2b4 &
/// 0x10` (a transport) and `+0x248` (`coll_size`); `boat_max` the
/// constants' `+0x94`. `dist(true)` is the two-argument
/// `ObjectData::attack_dist@0064c880` of the boat arm, `dist(false)` the
/// four-argument `006488f0` at the unit's own position — the same
/// distance, called by two names.
pub(crate) fn adjacent_steps(
    live: bool,
    target_live: bool,
    sea: bool,
    is_unit: impl FnOnce() -> bool,
    boat: impl FnOnce() -> (bool, i32),
    boat_max: i32,
    dist: impl FnOnce(bool) -> i32,
) -> bool {
    if !live || !target_live {
        return false;
    }
    if sea && is_unit() {
        let (transport, coll) = boat();
        if !transport {
            let max = if coll < 4 { boat_max + 0x180 } else { boat_max };
            return dist(true) < max;
        }
    }
    dist(false) < ADJACENT
}

/// What `Unit::find_gather_spot@005f5170` asks of the world (item 3320): the
/// owner's object list from `obj_base[1]` to its count, and for each object
/// the vtable questions, the region and city tests, the position and the
/// six-good ledger. Every method is one call (or one read) of the original;
/// the steps ask them in the original's order, which is what lets
/// `sweep::unit_find_gather_spot` answer each from the stub that stands for it.
pub trait GatherSpotView {
    /// `obj_base[1]`, where the loop starts.
    fn first(&mut self) -> i32;
    /// `objects.count[owner]`, where it ends.
    fn count(&mut self) -> i32;
    /// `ObjectData::is_scholar(this)` (`UnitTypeData +0x4` in `0x34`/`0x35`).
    fn scholar(&mut self) -> bool;
    /// `get_tregion` of the unit's own tile.
    fn region_of_unit(&mut self) -> i32;
    /// `find_city_at(unit's position, owner, -1, 0)`, negative for none.
    fn city_of_unit(&mut self) -> i32;
    /// The unit's position, `(x, y)`.
    fn unit_pos(&mut self) -> (i32, i32);
    /// Vtable `+0xc` of object `i`.
    fn alive(&mut self, i: i32) -> bool;
    /// Vtable `+0x4c`.
    fn active(&mut self, i: i32) -> bool;
    /// The type's gather flag (data `+0x18`, vtable `+0x90`).
    fn gather_type(&mut self, i: i32) -> bool;
    /// Data vtable `+0x180` (`WallData::is_neutralized` answers no).
    fn neutralized(&mut self, i: i32) -> bool;
    /// `is(0x1a4)`: a University.
    fn is_university(&mut self, i: i32) -> bool;
    /// `BuildData::num_gatherers(0, 0)`.
    fn num_gatherers(&mut self, i: i32) -> i32;
    /// `BuildData +0x80`, a signed byte.
    fn max_gatherers(&mut self, i: i32) -> i32;
    /// `UnitData::is_gathering_at(i, owner, 0)`.
    fn gathering_at(&mut self, i: i32) -> bool;
    /// `get_tregion` of the object's tile.
    fn region_of(&mut self, i: i32) -> i32;
    /// `BuildData +0x72`, the object's city (a short, negative for none).
    fn city_of(&mut self, i: i32) -> i32;
    /// A city's `free + gatherers` (`CityData +0x5a`, `+0x5c`).
    fn city_pop(&mut self, c: i32) -> i32;
    /// The object's position.
    fn pos_of(&mut self, i: i32) -> (i32, i32);
    /// `LeaderData::type_avail(good, 1)`.
    fn good_avail(&mut self, g: usize) -> bool;
    /// `building.is(best_gather_type(good))`.
    fn gathers(&mut self, i: i32, g: usize) -> bool;
    /// The ledger's per-good state word is the `0x8932` code (under the cap).
    fn under_cap(&mut self, g: usize) -> bool;
    /// `(cap ^ 0x1281) - (income ^ 0x90236)`, wrapping.
    fn headroom(&mut self, g: usize) -> i32;
    /// `LeaderData::has_tribe_bonus(0x16)`.
    fn dutch(&mut self) -> bool;
    /// `Unit::add_gather_order(i, QUEUE_LAST, 0)`.
    fn add_gather_order(&mut self, i: i32);
}

/// `Unit::find_gather_spot@005f5170` — the whole function over a
/// [`GatherSpotView`]; `range` is the caller's first stack argument.
///
/// Answers **false when the best building is one the unit is already
/// gathering at** (`5f5601`, `jne` to the `xor eax, eax` exit): the old
/// body answered true there without adding an order. A scholar searches
/// without a range limit (`bVar2`, the `0x34`/`0x35` test of the header).
pub fn find_gather_spot_steps<V: GatherSpotView>(v: &mut V, range: i32) -> bool {
    /// The knowledge good's place in the six-good table (`cmp ebx, 3`).
    const KNOWLEDGE: usize = 3;
    let header_scholar = v.scholar();
    let range = if header_scholar { -1 } else { range };
    let region = v.region_of_unit();
    let my_city = v.city_of_unit();
    let mut best: i32 = -1;
    let mut best_score = 0;
    let mut i = v.first();
    while i < v.count() {
        let cur = i;
        i += 1;
        if !v.alive(cur) || !v.active(cur) || !v.gather_type(cur) || v.neutralized(cur) {
            continue;
        }
        // Proceed only when the building is a University exactly when the
        // unit is a scholar.
        let univ = v.is_university(cur);
        if univ != v.scholar() {
            continue;
        }
        if v.num_gatherers(cur) >= v.max_gatherers(cur) && !v.gathering_at(cur) {
            continue;
        }
        if v.region_of(cur) != region {
            continue;
        }
        if !v.scholar() && my_city >= 0 {
            let theirs = v.city_of(cur);
            if my_city != theirs {
                let mypop = v.city_pop(my_city);
                if mypop < 2 {
                    continue;
                }
                if theirs >= 0 && mypop <= v.city_pop(theirs) + 2 {
                    continue;
                }
            }
        }
        let (ux, uy) = v.unit_pos();
        let (bx, by) = v.pos_of(cur);
        let dist = crate::world::vector_dist((ux - bx).abs(), (uy - by).abs());
        if range > 0 && dist > range {
            continue;
        }
        let mut sum: i32 = 0;
        for g in 0..6 {
            if !v.good_avail(g) || !v.gathers(cur, g) || !v.under_cap(g) {
                continue;
            }
            let h = v.headroom(g);
            if v.dutch() && g != KNOWLEDGE {
                sum = sum.wrapping_add(DUTCH_INTEREST_HEADROOM);
            }
            sum = sum.wrapping_add(h);
        }
        let score = sum.wrapping_mul(500) / (dist / TILE + 2);
        if best_score < score {
            best = cur;
            best_score = score;
        }
    }
    if best < 0 || v.gathering_at(best) {
        return false;
    }
    v.add_gather_order(best);
    true
}

/// [`GatherSpotView`] over the live [`Sim`]: the buildings are the objects
/// (an index that is not the owner's, or dead, answers `alive` no), the six
/// goods are the ledger's, and the city counters are the census's
/// `free + gatherers` (an absent record is zero, as for a human leader).
pub(super) struct GatherSpotSim<'a> {
    pub(super) sim: &'a mut Sim,
    pub(super) u: usize,
    pub(super) who: Player,
    pub(super) here: Pos,
}

impl GatherSpotView for GatherSpotSim<'_> {
    fn first(&mut self) -> i32 {
        0
    }
    fn count(&mut self) -> i32 {
        self.sim.buildings.len() as i32
    }
    fn scholar(&mut self) -> bool {
        self.sim.worker_of(self.u) == Worker::Scholar
    }
    fn region_of_unit(&mut self) -> i32 {
        // `get_tregion@006b52e0` is the coastal-refined region, here and
        // for each building below (item 3335: run970e's `1/77`, a barge
        // beside the Dock it finished on 14593, stands on a coastal cell's
        // water tile — `tregion` 12, `tregion_alt` 0 — so the original
        // finds no camp in its region and leaves it idle).
        self.sim
            .world
            .tregion_alt(self.here.tile())
            .map_or(-1, i32::from)
    }
    fn city_of_unit(&mut self) -> i32 {
        self.sim
            .find_city_at(self.who, self.here, None)
            .map_or(-1, |c| c as i32)
    }
    fn unit_pos(&mut self) -> (i32, i32) {
        (self.here.x, self.here.y)
    }
    fn alive(&mut self, i: i32) -> bool {
        let b = &self.sim.buildings[i as usize];
        b.alive && b.owner == self.who
    }
    fn active(&mut self, i: i32) -> bool {
        self.sim.buildings[i as usize].active
    }
    fn gather_type(&mut self, i: i32) -> bool {
        self.sim.is_gather_type(i as usize)
    }
    fn neutralized(&mut self, _i: i32) -> bool {
        // SEAM: a wall is not a gather type here, so the slot's `WallData`
        // answer (always no) is the only one reached.
        false
    }
    fn is_university(&mut self, i: i32) -> bool {
        self.sim.building_ident(i as usize) == Ident::University
    }
    fn num_gatherers(&mut self, i: i32) -> i32 {
        self.sim.num_gatherers(i as usize, false, false)
    }
    fn max_gatherers(&mut self, i: i32) -> i32 {
        // `None` is uncapped.
        self.sim.buildings[i as usize]
            .gather_max
            .unwrap_or(i32::MAX)
    }
    fn gathering_at(&mut self, i: i32) -> bool {
        self.sim.is_gathering_at(self.u, i as usize, false)
    }
    fn region_of(&mut self, i: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(self.sim.buildings[i as usize].pos.tile())
            .map_or(-1, i32::from)
    }
    fn city_of(&mut self, i: i32) -> i32 {
        self.sim.buildings[i as usize].city.map_or(-1, |c| c as i32)
    }
    fn city_pop(&mut self, c: i32) -> i32 {
        // `city_ai` only exists as far as the last census reached, and a
        // human leader never runs one — so an absent record is zero, which
        // is what the original's `CityData` holds for the same reason.
        self.sim.ai.ai[self.who as usize]
            .city_ai
            .get(c as usize)
            .map_or(0, |r| i32::from(r.free) + i32::from(r.gatherers))
    }
    fn pos_of(&mut self, i: i32) -> (i32, i32) {
        let p = self.sim.buildings[i as usize].pos;
        (p.x, p.y)
    }
    fn good_avail(&mut self, g: usize) -> bool {
        self.sim.economy.holdings[self.who as usize].available[g]
    }
    fn gathers(&mut self, i: i32, g: usize) -> bool {
        crate::ai::ai_place::gather_good(self.sim.building_ident(i as usize)) == Some(g)
    }
    fn under_cap(&mut self, g: usize) -> bool {
        self.sim.economy.ledgers[self.who as usize].over_cap[g] == economy::OverCap::Under
    }
    fn headroom(&mut self, g: usize) -> i32 {
        let l = &self.sim.economy.ledgers[self.who as usize];
        l.cap[g].wrapping_sub(l.income[g])
    }
    fn dutch(&mut self) -> bool {
        let w = self.who as usize;
        self.sim
            .tables
            .tech_tree
            .has_tribe_bonus(&self.sim.tables.setup, &self.sim.tech[w], 0x16)
    }
    fn add_gather_order(&mut self, i: i32) {
        self.sim
            .add_gather_order(self.u, i as usize, QueuePos::Last, false);
    }
}

/// [`crate::orders::check_build_view::CheckBuildView`] over the live [`Sim`]: the
/// cursor is an index into the unit's order list, the found list the units
/// `find_units` walked.
pub(super) struct SimCheckBuild<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    /// The cursor: the order's position in the list, front first.
    pub(super) i: usize,
    /// The units `find_units` returned.
    pub(super) found: Vec<usize>,
    /// The site [`Self::target_site`] named.
    pub(super) site: usize,
}

impl SimCheckBuild<'_> {
    /// The cursor order's site, when it is a build order.
    pub(super) fn target(&self) -> Option<usize> {
        match self.s.units[self.u].orders.get(self.i)?.body {
            Body::Build(b) => Some(b),
            _ => None,
        }
    }
}

impl crate::orders::check_build_view::CheckBuildView for SimCheckBuild<'_> {
    fn orders_present(&mut self) -> bool {
        !self.s.units[self.u].orders.is_empty()
    }
    fn list_present(&mut self) -> bool {
        !self.s.units[self.u].orders.is_empty()
    }
    fn cursor_to_front(&mut self) {
        self.i = 0;
    }
    fn cursor_advance(&mut self) {
        self.i += 1;
    }
    fn has_order(&mut self) -> bool {
        self.i < self.s.units[self.u].orders.len()
    }
    fn order_type(&mut self) -> i32 {
        i32::from(self.s.units[self.u].orders[self.i].index())
    }
    fn order_is_action(&mut self) -> bool {
        self.s.units[self.u].orders[self.i].has(flag::ACTION)
    }
    fn cursor_at_end(&mut self) -> bool {
        self.i + 1 >= self.s.units[self.u].orders.len()
    }
    fn target_exists(&mut self) -> bool {
        self.target()
            .is_some_and(|b| self.s.buildings.get(b).is_some_and(|bd| bd.alive))
    }
    fn target_finished(&mut self) -> bool {
        self.target().is_some_and(|b| self.s.buildings[b].active)
    }
    fn target_site(&mut self) -> i32 {
        self.site = self.target().expect("a build order");
        self.site as i32
    }
    fn target_blocks(&mut self) -> bool {
        self.s.building_is_city(self.site)
    }
    fn repath(&mut self) {
        self.s.repath(self.u);
    }
    fn kill_current_order(&mut self) {
        self.s.remove_order_at(self.u, self.i);
    }
    fn add_build_order(&mut self, site: i32) {
        self.s
            .add_build_order(self.u, site as usize, QueuePos::First, true);
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn find_units(&mut self) {
        let range = self.s.tables.tuning.unit_build_respond_range * TILE;
        self.found = self.s.find_units_friendly(self.u, range);
    }
    fn found_count(&mut self) -> i32 {
        self.found.len() as i32
    }
    fn found_other(&mut self, j: i32) -> bool {
        self.s.units[self.found[j as usize]].index != self.s.units[self.u].index
    }
    fn found_builds(&mut self, j: i32) -> bool {
        let o = self.found[j as usize];
        self.s
            .action_of(o)
            .is_some_and(|k| matches!(self.s.units[o].orders[k].body, Body::Build(_)))
    }
    fn found_target(&mut self, j: i32) -> (i32, i32) {
        let o = self.found[j as usize];
        let k = self.s.action_of(o).expect("an action");
        let Body::Build(b) = self.s.units[o].orders[k].body else {
            unreachable!("found_builds said a build order")
        };
        // The order's `+0xc` is the owner `add_build_order` was handed: the unit's own.
        (i32::from(self.s.units[o].owner), b as i32)
    }
}
