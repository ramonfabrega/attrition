//! **The City's alarm and a player's gather** — two issuers of the
//! Militia's Civilian that no chapter before forty staged (item 1167,
//! `docs/GOLDEN.md` §49).
//!
//! - `CommandPackage::process_alarm@00947ef0` → `Group::action_alarm@
//!   0070ec30` on a group of the player's own buildings: the bell, and
//!   the all-clear that makes every Militia inside the city's buildings a
//!   Citizen again (`SpellType::cast_civilian`) before it empties them.
//! - `CommandPackage::process_gather@009488b0` → `Group::action_gather@
//!   00700b90(ox, queued)` on a group of units: a Militia among them takes
//!   the Civilian in front of its gather.

use crate::Sim;
use crate::group::Group;
use crate::orders::{QueuePos, index, spell};
use crate::world::{Player, Pos};

impl Sim {
    /// `Group::action_alarm@0070ec30` on the buildings `list` of `who`,
    /// in the group's order.
    ///
    /// ```text
    /// for each member that is a live city building (+8 & 0x20, its
    ///   CityData at +0x72) whose city has city_flags & 1:
    ///     0x40 clear, and no all-clear yet this call: the BELL
    ///     0x40 set,   and no bell yet this call:      the ALL-CLEAR
    /// ```
    ///
    /// Every press sets the leader's `0x2000000` first (`0070ed27`), the
    /// economy's dirty bit ([`Sim::economy_changed`], `crate::holdings`):
    /// run430's income on 889, eight frames after the bell, counts the
    /// Citizens under their GARRISONs out.
    ///
    /// The walk is `crate::alarm_view::action_alarm_steps`, swept against the
    /// original (item 3364); the SEAMs are `SimAlarm`'s: the scenario's
    /// `ignore_orders` sweep; the two sounds; a building on a cell's chain.
    /// A group of units' alarm is [`Sim::action_alarm_peasant`], which the
    /// harness calls directly (item 3916).
    pub fn action_alarm(&mut self, who: Player, list: &[usize]) {
        let mut view = crate::alarm_view::SimAlarm::new(self, who, list);
        crate::alarm_view::action_alarm_steps(&mut view);
    }

    /// `Group::action_alarm_peasant@006fd980`: `Group::action_alarm`'s arm
    /// for a group of units (`+0x49` clear), the key bound to
    /// `OPTION_PEASANT_ALARM` on a selection of Citizens (item 3916,
    /// `docs/GOLDEN.md` §163).
    ///
    /// ```text
    /// for each member of type 0x32 or 0x33 (the TypeIndex, not the lineage):
    ///   b = find_building(pos, SEARCH_FRIENDLY, who, -1, 0x200,
    ///                     FILTER_CAN_GARRISON, o, who)   # the whole map
    ///   b < 0: next
    ///   while the action is a GARRISONORDER (0x1a): repath,
    ///     kill_current_order(0)
    ///   add_garrison_order(b, who, 1, QUEUE_FIRST, 1), inline
    ///   the leader's 0x2000000
    /// ```
    ///
    /// The range is −1 and `0x200` the flags word (the listing's pushes at
    /// `6fda58`..`6fda6e`), so the search is [`Sim::nearest_garrison_for`]'s
    /// list walk over the whole map, gated on the Citizen's region. The
    /// loop's `0x1a` is `GARRISON`, not `GATHER` (7): run1510's `0/1` keeps
    /// its `GATHERORDER` under the new order on 644, and a second press on a
    /// Citizen already walking to garrison replaces its order.
    ///
    /// SEAMS: the group's slot `+0x14` and the scenario's ignore-orders
    /// sweep, as [`Sim::action_alarm`]'s; an ally's building, whose order
    /// the original lays with the group's `who` beside the ally's object
    /// number (`+0xc` is `bVar2`, not `find_who`).
    pub fn action_alarm_peasant(&mut self, g: &Group) {
        let who = g.who;
        for &u in &g.list {
            if !matches!(self.units[u].type_index, 0x32 | 0x33) {
                continue;
            }
            let Some(b) = self.nearest_garrison_for(u, who) else {
                continue;
            };
            while self
                .action_of(u)
                .is_some_and(|a| self.units[u].orders[a].index() == index::GARRISON)
            {
                self.repath(u);
                self.kill_current_order(u);
            }
            self.add_garrison_order(u, b, true, QueuePos::First, true);
            self.economy_changed(who);
        }
    }

    /// **A Citizen struck rings its city's bell** — `Object::take_damage
    /// @00652020`'s arm for an object that is not a city building (vslot
    /// `+0x20` answers 0; the decompile's lines 176–214), after the
    /// accumulate and before the kill arm, on every blow (item 3984,
    /// `docs/AI.md` §416; the swept walk is
    /// [`crate::take_damage_view`]'s `peasant_alarm`).
    ///
    /// ```text
    /// leader_flags & 4 clear (a computer), leader_flags2 & 8 clear,
    /// get_diff > 1, ObjectData::is_peasant(this),
    /// c = find_city_at(this's point, who, -1, 0) >= 0,
    /// c's building finished, and c's flags & 0x40 (the bell) clear:
    ///   a group of c's building alone, pushed on who's groups, and
    ///   Group::action_alarm on it
    /// ```
    ///
    /// run1075's Battleship `0/23` strikes `1/2033` on 2072, and the spill
    /// reaches its four repairers in chain order: the first, `1/37`, rings
    /// the bell, and every Citizen in the city's radius takes a
    /// `GARRISONORDER` in front of its REPAIR, so the spill passes the other
    /// three by. `leader_flags2 & 8` is the combat AI's scenario switch,
    /// never set here (`docs/ARMY.md` §18). SEAMS: the group is not pushed
    /// on the player's pool (`Groups::push_group`'s slot; run1075's
    /// widening parts no pool row on 2073 without it); attrition's blow (`Unit::suffer_attrition`
    /// calls `take_damage` with `param_5` 1, which this arm does not read)
    /// does not come through here, and needs a computer Citizen bleeding
    /// on a cell `find_city_at` accepts for its own city — neutral
    /// attrition, which no shipped scenario sets.
    pub(crate) fn peasant_alarm(&mut self, u: usize) {
        let who = self.units[u].owner;
        if self.cities_state.nation[who as usize].human
            || self.ai_difficulty() < 2
            || !self.is_peasant(u)
        {
            return;
        }
        let Some(c) = self.find_city_at(who, self.units[u].pos, None) else {
            return;
        };
        let b = self.cities_state.cities[c].building;
        let bd = &self.buildings[b];
        if !(bd.alive && bd.active) || self.cities_state.cities[c].alarm {
            return;
        }
        self.action_alarm(who, &[b]);
    }

    /// `ObjectsData::find_building@0065d260(pos, SEARCH_FRIENDLY, who, -1,
    /// 0x200, FILTER_CAN_GARRISON, u, who)`: the list walk (a negative
    /// range), every player that is `who` or an ally in object order, each
    /// building in the unit's cell region that is alive and active, whose
    /// type the unit `can_garrison`, and with room for its population
    /// (`Search::valid_filter`'s arm 20, `0067e44d`: `num_inside(0)` +
    /// `control_cost` `<=` the garrison limit). Nearest by
    /// [`vector_dist`](crate::world::vector_dist) of the tile deltas, ties
    /// to the later (`<=`).
    pub(crate) fn nearest_garrison_for(&self, u: usize, who: Player) -> Option<usize> {
        let at = self.units[u].pos;
        let region = self.world.region_of(at.cell());
        let ut = self.units[u].ty?;
        let pop = self.control_cost(u);
        let mut cands: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive
                    && bd.active
                    && (bd.owner == who || self.is_ally(who, bd.owner))
                    && self.world.region_of(bd.pos.cell()) == region
                    && bd.ty.is_some_and(|bt| self.can_garrison(ut, bt))
                    && self.num_inside(b) + pop <= self.garrison_limit(b)
            })
            .collect();
        cands.sort_by_key(|&b| (self.buildings[b].owner, self.buildings[b].index));
        let (t, mut best, mut found) = (at.tile(), i32::MAX, None);
        for b in cands {
            let bt = self.buildings[b].pos.tile();
            let d = crate::world::vector_dist(bt.x - t.x, bt.y - t.y);
            if d <= best {
                best = d;
                found = Some(b);
            }
        }
        found
    }

    /// `ObjectData::is(ti, 0)` on a unit, `ti` a `TypeIndex`: its lineage
    /// holds the type's tree node.
    pub(crate) fn unit_is_index(&self, u: usize, ti: i32) -> bool {
        self.unit_rec_of_index(ti)
            .and_then(|r| self.tables.unit_types[r].tree)
            .is_some_and(|t| self.unit_line_is(u, t))
    }

    /// `UnitData::control_cost(0)@00609d40`: a decoy 0, else the type's
    /// `POP` (`+0x2f0`).
    pub(crate) fn control_cost(&self, u: usize) -> i32 {
        if self.units[u].decoy {
            return 0;
        }
        self.units[u]
            .ty
            .map_or(0, |t| self.tables.unit_types[t].price.pop)
    }

    /// `Object::eject_contents(0, PEASANTS, 0, 0)@0064cd20` on a building:
    /// the typed arm, which is **immediate** (only a negative type defers
    /// through the building's `0x4000`). Walking the inside chain from the
    /// head: a unit that `is(PEASANTS)` comes out (`repath` first when it
    /// has an action); one that is not, but `is(MILITIA)`, comes out too
    /// with a Civilian at `QUEUE_FIRST` on the building's point; anything
    /// else stays. A `come_out` that fails ends the walk (`param_1` 0).
    pub(crate) fn eject_citizens_now(&mut self, b: usize) {
        let mut i = 0;
        while i < self.buildings[b].garrison.len() {
            let u = self.buildings[b].garrison[i];
            let citizen = self.unit_is_index(u, crate::cast::PEASANTS);
            let militia = !citizen && self.unit_is_index(u, crate::cast::MILITIA);
            if !citizen && !militia {
                i += 1;
                continue;
            }
            if self.action_of(u).is_some() {
                self.repath(u);
            }
            if militia && self.spell_castable(spell::CAST_CIVILIAN, u) {
                let at = self.buildings[b].pos;
                self.add_cast_order_on(u, spell::CAST_CIVILIAN, None, at, QueuePos::First, true);
            }
            if !self.come_out(u) {
                return;
            }
        }
    }

    /// `Group::action_gather@00700b90(b, queued)` on the group `g`: each
    /// member that gathers is given a gather building of its own player's
    /// found on a spiral round `b` (`move_x/move_y`, `world::MOVE_289`),
    /// and a Militia among them the Civilian in front of it.
    ///
    /// ```text
    /// queued QUEUE_FIRST is taken as QUEUE_NEW; the group's form -1
    /// region = b's cell's tregion (the coastal refinement)
    /// idle   = count(CAN_GATHER) - count(GATHER_WORKERS)
    /// for each member, the LAST first, alive and on the map:
    ///     a Scholar (0x34/0x35, or rare): the movers
    ///     a Citizen, or castable Civilian, or (a computer's) a transport
    ///       holding Citizens: a gatherer; anything else: the movers
    ///   a gatherer: skipped when idle != 0 and its action is a GATHER;
    ///     a human's gatherer not in b's region: skipped
    ///     the spiral's next offset k (k carries across members, 0 after
    ///       each assignment; past 0x78 every member left joins the
    ///       movers): a human's probe out of b's region is passed over;
    ///       on the probe cell's objects, the first of the player's that
    ///       is an active gather building: the member's +0x80 = -1, and
    ///       QUEUE_LAST: Civilian (castable) then gather, both QUEUE_LAST
    ///       else:       gather QUEUE_NEW, then Civilian QUEUE_FIRST
    ///       the member leaves the group (Group::kill); next member
    /// the movers: action_move_near(b's point, 0xc0, QUEUE_NEW, MOVE_TO)
    /// ```
    ///
    /// SEAM, read and not staged by chapter forty: a University target and
    /// its Scholars' arm; the good's `type_avail` test; the probe cell's
    /// objects are its buildings in this crate (`Sim::cell_chain` threads
    /// units, and a unit is never a gather building); the non-castable
    /// arm's oil-rig boarding and `Build::check_gatherers`; the movers'
    /// `action_move_near`.
    pub fn group_gather(&mut self, g: &Group, b: usize, queued: QueuePos) {
        let pos = if queued == QueuePos::First {
            QueuePos::New
        } else {
            queued
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let who = g.who;
        let target = self.buildings[b].pos;
        let region = self.world.tregion_alt(target.tile());
        let human = self.cities_state.nation[who as usize].human;
        let idle = g
            .list
            .iter()
            .filter(|&&u| self.worker_of(u) == crate::orders::Worker::Citizen)
            .filter(|&&u| {
                !self
                    .action_of(u)
                    .is_some_and(|a| self.units[u].orders[a].index() == index::GATHER)
            })
            .count();
        let mut members = g.list.clone();
        let mut k = 0usize;
        let mut i = members.len();
        'members: while i > 0 {
            let u = members[i - 1];
            if !self.units[u].alive() || !self.units[u].on_map {
                i -= 1;
                continue;
            }
            let ti = self.units[u].type_index;
            if matches!(ti, 0x34 | 0x35) {
                i -= 1;
                continue;
            }
            let gatherer =
                matches!(ti, 0x32 | 0x33) || self.spell_castable(spell::CAST_CIVILIAN, u);
            if !gatherer {
                i -= 1;
                continue;
            }
            let gathering = self
                .action_of(u)
                .is_some_and(|a| self.units[u].orders[a].index() == index::GATHER);
            if idle != 0 && gathering {
                i -= 1;
                continue;
            }
            if human && self.world.tregion_alt(self.units[u].pos.tile()) != region {
                i -= 1;
                continue;
            }
            loop {
                let Some(&(dx, dy)) = crate::world::MOVE_289.get(k) else {
                    break 'members;
                };
                k += 1;
                if k > 0x78 {
                    break 'members;
                }
                let probe = Pos::new(target.x + dx * 0x300, target.y + dy * 0x300);
                let (w, h) = (self.world.width() * 0x300, self.world.height() * 0x300);
                if probe.x < 0 || probe.y < 0 || probe.x >= w || probe.y >= h {
                    continue;
                }
                if human && self.world.tregion_alt(probe.tile()) != region {
                    continue;
                }
                let cell = probe.cell();
                let found = (0..self.buildings.len()).rev().find(|&x| {
                    let bd = &self.buildings[x];
                    bd.owner == who
                        && bd.alive
                        && bd.active
                        && bd.pos.cell() == cell
                        && self.is_gather_type(x)
                });
                let Some(site) = found else {
                    continue;
                };
                let at = self.units[u].pos;
                let castable = self.spell_castable(spell::CAST_CIVILIAN, u);
                if pos == QueuePos::Last {
                    if castable {
                        self.add_cast_order_on(
                            u,
                            spell::CAST_CIVILIAN,
                            None,
                            at,
                            QueuePos::Last,
                            true,
                        );
                    }
                    self.add_gather_order(u, site, QueuePos::Last, true);
                } else {
                    self.add_gather_order(u, site, QueuePos::New, true);
                    if castable {
                        self.add_cast_order_on(
                            u,
                            spell::CAST_CIVILIAN,
                            None,
                            at,
                            QueuePos::First,
                            true,
                        );
                    }
                }
                if let Some(seat) = Self::seat_of_group(g) {
                    self.seat_kill(seat, u, false, false);
                }
                members.retain(|&m| m != u);
                k = 0;
                i -= 1;
                continue 'members;
            }
        }
    }
}
