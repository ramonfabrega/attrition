//! Trade routes — the `Caravan` object a caravan unit owns, and the road it
//! plans between two cities.
//!
//! `docs/CARAVAN.md`. The mechanic is one object and four callers:
//!
//! 1. **Birth** (§2). `Unit::init` gives every **land** caravan a slot in its
//!    leader's twenty-slot `Caravans` list; the unit remembers the slot and
//!    the slot remembers the unit.
//! 2. **The order** (§3). `Unit::think_caravan` waits out the idle gate and
//!    queues a `TRADE_ROUTE` against the nearest allied city that could take
//!    another route.
//! 3. **The route** (§4). `Unit::do_trade` picks the far city — the best
//!    `Caravan::trade_value` over every allied city that can pair with the
//!    home one — writes the pair into the `Caravan`, and plans the road.
//! 4. **The road** (§5). `Caravan::build_road` is `crate::roads`' search with
//!    the caravan arm on: eight directions, a heuristic sixteen times
//!    larger, and the right to **stop at the budget and carry on next
//!    frame**. A plan that stops is parked in the `Caravan`; `Unit::work`
//!    calls `build_road` again on every later frame until it finishes.
//!
//! The **legs** — the walk between the two cities, the cargo, the wealth a
//! completed trip pays — are not modelled. What is here is what the stream
//! sees: a trade route between two of East Indies' cities costs 12,965 road
//! draws over five frames, and until 2026-09-02 this crate spent none of
//! them.

use crate::orders::{Body, QueuePos, TradeOrder, flag, index};
use crate::roads::{RoadPlan, RoadSearch};
use crate::world::Pos;
use crate::{Player, Sim};

/// `Caravans::init@0073e870` gives every leader twenty `Caravan`s.
pub const CARAVAN_SLOTS: usize = 20;

/// One trade route — the original's `CaravanData` (0x44 bytes), whose whole
/// layout the type record names.
#[derive(Clone, Debug, Default)]
pub struct Caravan {
    /// `caravan_flags & 1`: the slot is in use.
    pub alive: bool,
    /// `caravan_flags & 2`: `do_trade` has written the pair. `Unit::work`'s
    /// gate is `& 6`, and `& 4` is `restart_trade_route`'s — unset here.
    pub linked: bool,
    /// `city2`/`whom` (+0x0, +0x2) — the **home** city, and the search's
    /// start.
    pub city_a: Option<usize>,
    /// `city3`/`whose` (+0x4, +0x6) — the far city, and the goal.
    pub city_b: Option<usize>,
    /// `o` (+0xa) — the caravan unit.
    pub unit: Option<usize>,
    /// `road` (+0x10): the plan, near-goal end first.
    pub road: Vec<Pos>,
    /// `making_road` (+0x20): a search is parked and wants another frame.
    pub making_road: bool,
    /// `reset_road` (+0x24): somebody else's road changed the map, so throw
    /// the parked search away and start again.
    pub reset_road: bool,
    /// `openlist`/`openlistrefs`/`closedlist` (+0x28, +0x2c, +0x30) with
    /// `offset`, `endx` and `endy` (+0x34, +0x38, +0x3c) — the parked search.
    ///
    /// `traversed` (+0x40) is written beside them and **never read back**,
    /// which is why a resumed frame starts its budget again at zero.
    pub search: Option<RoadSearch>,
}

/// One leader's list — `PtrArray<Caravan>` and `LeaderData::caravan_mark`.
#[derive(Clone, Debug)]
pub struct Caravans {
    pub slots: Vec<Caravan>,
    /// `LeaderData +0x438`: one past the highest slot ever used, walked back
    /// down past trailing free slots by `close_caravan`.
    pub mark: usize,
}

impl Default for Caravans {
    fn default() -> Caravans {
        Caravans {
            slots: vec![Caravan::default(); CARAVAN_SLOTS],
            mark: 0,
        }
    }
}

impl Sim {
    // ------------------------------------------------------------------
    // §2 — birth and death
    // ------------------------------------------------------------------

    /// `Caravans::init_caravan@0073e1f0` — the first free slot below the
    /// mark, else the mark itself. `Unit::init@00612100:436` calls it for a
    /// unit that `is_caravan` (`unit_flags2 & 8`) **and whose type domain is
    /// land** (`UnitTypeData +0x218 == 0`), so a Merchant Fleet takes the
    /// special list instead and never owns a route.
    pub(crate) fn init_caravan(&mut self, who: Player, unit: usize) -> Option<usize> {
        let list = self.caravans.get_mut(who as usize)?;
        let slot = (0..list.mark)
            .find(|&i| !list.slots[i].alive)
            .or_else(|| (list.mark < list.slots.len()).then_some(list.mark))?;
        list.mark = list.mark.max(slot + 1);
        list.slots[slot] = Caravan {
            alive: true,
            unit: Some(unit),
            ..Caravan::default()
        };
        Some(slot)
    }

    /// `Caravans::close_caravan@0073e350` — the slot goes back, the parked
    /// search with it, and the mark walks down past every dead slot at the
    /// top.
    pub(crate) fn close_caravan(&mut self, who: Player, slot: usize) {
        let Some(list) = self.caravans.get_mut(who as usize) else {
            return;
        };
        list.slots[slot] = Caravan::default();
        while list.mark > 0 && !list.slots[list.mark - 1].alive {
            list.mark -= 1;
        }
    }

    // ------------------------------------------------------------------
    // §3 — `Unit::think_caravan@005f5650`
    // ------------------------------------------------------------------

    /// The idle gate, the home city, and the order. `true` ends the think.
    ///
    /// The threshold is **1** for an AI-driven unit and otherwise the
    /// owner's difficulty option — 2 by default — and after it the unit
    /// tries again every fifth frame: `idle == threshold || (idle − 2) % 5
    /// == 0`.
    pub(crate) fn think_caravan(&mut self, u: usize) -> bool {
        let idle = self.units[u].idle;
        let threshold = if self.ai_driven(self.units[u].owner) {
            1
        } else {
            // SEAM: `LeaderOptions +0x8`'s five arms (7, 12, 17, 32, 62) are
            // the human's idle-caravan setting; the default is 2 and no
            // capture has a human caravan at all.
            2
        };
        if idle < threshold {
            return false;
        }
        if idle != threshold && !(idle - 2).is_multiple_of(5) {
            return false;
        }
        let Some(home) = self.find_trade_city(self.units[u].owner, self.units[u].pos) else {
            return false;
        };
        self.add_trade_order(u, home);
        true
    }

    /// `ObjectsData::find_city(SEARCH_ALLIED, who, FILTER_CAN_TRADE)`: the
    /// nearest live city of any ally that could take another route. No
    /// region flag, no distance limit, and ties go to the **later** city —
    /// the original compares `<=`.
    ///
    /// `FILTER_CAN_TRADE` is jump-table arm 17 of
    /// `Search::valid_filter@0067dbb0` (`FILTER_CAN_TRADE = 18`,
    /// `FILTER_TYPE = 1`; the table is at `0067e57c` and the arm at
    /// `0067e204`). It is the same predicate as §4's loop with the scoring
    /// taken out: some allied city, not this one, that this city and it can
    /// both still pair with.
    fn find_trade_city(&self, who: Player, at: Pos) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (i, c) in self.cities.iter().enumerate() {
            if !c.alive || !self.is_ally(who, c.owner) {
                continue;
            }
            let d = crate::world::vector_dist(c.pos.x - at.x, c.pos.y - at.y);
            if best.is_some_and(|(_, bd)| d > bd) {
                continue;
            }
            if !self.city_can_trade(who, i) {
                continue;
            }
            best = Some((i, d));
        }
        best.map(|(i, _)| i)
    }

    /// `Search::valid_filter`'s `FILTER_CAN_TRADE` arm: is there any allied
    /// city this one could still be paired with.
    fn city_can_trade(&self, who: Player, city: usize) -> bool {
        (0..self.cities.len()).any(|o| o != city && self.trade_pair_free(who, city, o))
    }

    /// Both halves of the pairing test, which every caller makes together:
    /// the other city is alive, its owner an ally, this player has seen it,
    /// and **neither** city already runs a route to the other.
    ///
    /// `CityData::get_empty_trade_routes@007395c0` counts the entries of a
    /// city's `vans` list that name the other city and answers `1 − count`,
    /// so a pair of cities carries at most one route.
    fn trade_pair_free(&self, who: Player, city: usize, other: usize) -> bool {
        let b = &self.cities[other];
        if !b.alive || !self.is_ally(who, b.owner) {
            return false;
        }
        if !self.city_is_seen(other, who) {
            return false;
        }
        !self.trade_route_exists(city, other) && !self.trade_route_exists(other, city)
    }

    /// One direction of the `vans` test: does any live route of any leader
    /// already join these two.
    fn trade_route_exists(&self, city: usize, other: usize) -> bool {
        self.caravans.iter().any(|l| {
            l.slots[..l.mark].iter().any(|v| {
                v.alive
                    && v.linked
                    && ((v.city_a == Some(city) && v.city_b == Some(other))
                        || (v.city_a == Some(other) && v.city_b == Some(city)))
            })
        })
    }

    /// `CityData::is_seen@00739560`.
    ///
    /// SEAM: taken as "the owner has ever seen the city's tile"; the
    /// original keeps a per-leader bit on the city and the fog read behind
    /// it is `crate::scout`'s. Every capture's cities are its own or in
    /// sight, so the two have not been told apart.
    fn city_is_seen(&self, city: usize, who: Player) -> bool {
        let c = &self.cities[city];
        c.owner == who || self.was_seen_fog(c.pos.tile().x >> 1, c.pos.tile().y >> 1, who)
    }

    /// `Unit::add_trade_order@005e4dc0` with `QUEUE_NEW` and no far city:
    /// the order list is cleared and one `TRADE_ROUTE` put on it.
    fn add_trade_order(&mut self, u: usize, home: usize) {
        let order = crate::orders::Order {
            flags: flag::ACTION,
            body: Body::Trade(TradeOrder {
                home,
                dest: None,
                started: false,
            }),
        };
        self.enqueue_order(u, order, QueuePos::New);
    }

    // ------------------------------------------------------------------
    // §4 — `Unit::do_trade@005ed270`
    // ------------------------------------------------------------------

    /// The order's own step. While a road is being planned the function
    /// returns at its head, which is why the retry lives in `Unit::work`.
    ///
    /// SEAM: everything past the road — the legs between the two cities, the
    /// cargo, the wealth a completed trip pays — is unmodelled, so a caravan
    /// whose road is laid stands still. `docs/CARAVAN.md` §7.
    pub(crate) fn do_trade(&mut self, u: usize) {
        let Some(v) = self.units[u].caravan else {
            return;
        };
        let who = self.units[u].owner;
        if self.caravans[who as usize].slots[v].making_road {
            return;
        }
        let Some(Body::Trade(mut ord)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        if !self.cities[ord.home].alive {
            self.kill_current_order(u);
            return;
        }
        if ord.dest.is_none() {
            let Some(dest) = self.pick_trade_partner(u, ord.home) else {
                // "No trade route available": the order dies and the unit
                // goes back to thinking.
                self.units[u].idle = 99;
                self.kill_current_order(u);
                return;
            };
            ord.dest = Some(dest);
            self.set_trade_order(u, ord);
        }
        if ord.started {
            return;
        }
        let Some(dest) = ord.dest else { return };
        // The nearer city becomes the route's `city2` — the search's start —
        // and the further its `city3`.
        let here = self.units[u].pos;
        let da = crate::world::vector_dist(
            self.cities[ord.home].pos.x - here.x,
            self.cities[ord.home].pos.y - here.y,
        );
        let db = crate::world::vector_dist(
            self.cities[dest].pos.x - here.x,
            self.cities[dest].pos.y - here.y,
        );
        let (a, b) = if da < db {
            (ord.home, dest)
        } else {
            (dest, ord.home)
        };
        let list = &mut self.caravans[who as usize].slots[v];
        list.city_a = Some(a);
        list.city_b = Some(b);
        list.alive = true;
        list.linked = true;
        ord.started = true;
        self.set_trade_order(u, ord);
        // **A plan that did not finish sends the caravan walking.**
        // `do_trade@005ed270:397`: `build_road` answering −1 queues a
        // `QUEUE_FIRST` move to the **near** city's own point and returns,
        // so the unit walks its first leg while the road is still being
        // searched. That walk is what puts its two crew figures into a
        // walking animation, and their clocks are two of run54's frame
        // 6169 draws (`docs/CARAVAN.md` §4.1).
        //
        // SEAM: the original's arrival facing is `find_angle(1, 0)` — the
        // literal `1, 0` it passes where `add_move_order` reads a
        // direction — where this crate's adder takes the bearing to the
        // destination. Nothing reads it until the unit arrives.
        if self.caravan_build_road(who, v) < 0 {
            let to = self.cities[a].pos;
            self.add_move_order(
                u,
                to,
                crate::orders::MoveKind::MoveTo,
                QueuePos::First,
                false,
            );
        }
    }

    /// Rewrite the current order's body in place.
    fn set_trade_order(&mut self, u: usize, ord: TradeOrder) {
        if let Some(o) = self.units[u].orders.front_mut() {
            o.body = Body::Trade(ord);
        }
    }

    /// §4's selection loop: over every leader that is an ally and every one
    /// of its cities, the best `Caravan::trade_value`, quadrupled when the
    /// home city is the caravan owner's own.
    fn pick_trade_partner(&mut self, u: usize, home: usize) -> Option<usize> {
        let who = self.units[u].owner;
        let mine = self.cities[home].owner == who;
        let mut best: Option<(usize, i32)> = None;
        for other in 0..self.cities.len() {
            if other == home || !self.trade_pair_free(who, home, other) {
                continue;
            }
            // SEAM: the cross-region arm. The original admits a partner in
            // another region only when the caravan `can_transport`; no
            // capture has a route that crosses water (`docs/CARAVAN.md` §6).
            if self.world.tregion(self.cities[other].pos.tile())
                != self.world.tregion(self.cities[home].pos.tile())
            {
                continue;
            }
            let mut v = self.trade_value(home, other);
            if mine {
                v <<= 2;
            }
            if best.is_none_or(|(_, bv)| bv < v) {
                best = Some((other, v));
            }
        }
        best.map(|(c, _)| c)
    }

    /// `Caravan::trade_value@0073d9d0`: the two cities' trade values, scaled
    /// by the distance band, by half again for a foreign partner, and by the
    /// two bonuses.
    fn trade_value(&self, a: usize, b: usize) -> i32 {
        let d = self.trade_distance(a, b);
        let mut v = self.city_trade_value(a) + self.city_trade_value(b);
        if d != 0 {
            v = (d + 3) * v / 3;
        }
        if self.cities[a].owner != self.cities[b].owner {
            v = v * 3 / 2;
        }
        // SEAM: the Indian tribe bonus (`has_tribe_bonus(0x15)`,
        // `indians_caravan`) and the spice rare (`leader_flags & 0x40`,
        // `spice_caravan_income`) both scale the answer; neither is read
        // here, and no capture has either.
        v
    }

    /// `Caravan::distance@0073d300`: a band, not a distance — 0 under a
    /// quarter of the map's width, 1 under a half, 2 under four fifths, 3
    /// beyond.
    fn trade_distance(&self, a: usize, b: usize) -> i32 {
        let (pa, pb) = (self.cities[a].pos, self.cities[b].pos);
        let d = crate::world::vector_dist(pa.x - pb.x, pa.y - pb.y);
        let w = self.world.width();
        if d < w.div_euclid(4) {
            return 0;
        }
        if d < w / 2 {
            return 1;
        }
        2 + i32::from(w * 4 / 5 <= d)
    }

    /// `CityData::get_trade_value@007363f0`: the city's building count, plus
    /// two for a Large City and four for a Major City or better.
    fn city_trade_value(&self, c: usize) -> i32 {
        let n = i32::try_from(self.cities[c].members.len()).unwrap_or(0);
        let Some(ty) = self.buildings[self.cities[c].building].ty else {
            return n;
        };
        // `TOWN` (0x19f) is the Large City; `METROPOLIS` (0x1a0) and
        // `FORBIDDENCITY` (0x213) are the two that score four.
        if self.build_types[ty].ident == crate::build::Ident::Town {
            return n + 2;
        }
        if matches!(
            self.build_types[ty].ident,
            crate::build::Ident::Metropolis | crate::build::Ident::ForbiddenCity
        ) {
            return n + 4;
        }
        n
    }

    // ------------------------------------------------------------------
    // §5 — `Caravan::build_road@0073db10`
    // ------------------------------------------------------------------

    /// One frame's worth of the route's road plan.
    ///
    /// Returns the original's own answer: 1 when the road was laid, 0 when
    /// there was nothing to plan, −1 when the budget stopped the search and
    /// it wants another frame.
    pub(crate) fn caravan_build_road(&mut self, who: Player, v: usize) -> i32 {
        let w = who as usize;
        self.caravans[w].slots[v].road.clear();
        let (Some(a), Some(b)) = (
            self.caravans[w].slots[v].city_a,
            self.caravans[w].slots[v].city_b,
        ) else {
            self.clear_temp_road(who, v);
            return 0;
        };
        if !self.cities[a].alive || !self.cities[b].alive {
            self.clear_temp_road(who, v);
            return 0;
        }
        let ends = (self.cities[a].building, self.cities[b].building);
        // The three-way gate at `build_road`'s head: a parked search that
        // nothing has invalidated is **resumed**; anything else starts over,
        // and `clear_temp_road` throws the parked one away.
        let van = &self.caravans[w].slots[v];
        let resume = van.making_road && !van.reset_road && van.search.is_some();
        let mut st = if resume {
            self.caravans[w].slots[v]
                .search
                .take()
                .expect("the parked search")
        } else {
            if self.caravans[w].slots[v].reset_road || self.caravans[w].slots[v].search.is_none() {
                self.clear_temp_road(who, v);
            }
            let Some(st) = self.start_road(ends.0, ends.1) else {
                return -1;
            };
            st
        };
        // `find_road@00688a40+0x2ac` sets `pathfinder+0x98` from the
        // caravan **unit** — `UnitData::can_transport` — so a route whose
        // caravan may auto-transport prices ocean tiles instead of refusing
        // them (`docs/CARAVAN.md` §5.1).
        let can_transport = self.caravans[w].slots[v]
            .unit
            .is_some_and(|u| self.unit_can_transport(u));
        match self.step_road_caravan(&mut st, ends, can_transport) {
            RoadPlan::Budget => {
                let van = &mut self.caravans[w].slots[v];
                van.search = Some(st);
                van.making_road = true;
                -1
            }
            RoadPlan::None => {
                self.caravans[w].slots[v].road.clear();
                0
            }
            RoadPlan::Road(tiles) => {
                for t in &tiles {
                    if self.world.tile_mask(*t) & crate::world::tile::BLOCKED != 0 {
                        continue;
                    }
                    self.set_road_at(*t);
                }
                let van = &mut self.caravans[w].slots[v];
                van.road = tiles;
                van.reset_road = false;
                van.making_road = false;
                van.search = None;
                // `Caravans::reset_paths@0073e060`: the map just changed, so
                // every other leader's part-planned route starts again.
                self.caravans_reset_paths();
                1
            }
        }
    }

    /// `Caravan::clear_temp_road@0073de80`: the parked search is freed and
    /// `making_road` cleared with it.
    fn clear_temp_road(&mut self, who: Player, v: usize) {
        let van = &mut self.caravans[who as usize].slots[v];
        van.search = None;
        van.making_road = false;
    }

    /// `Caravans::reset_paths@0073e060`: every alive route of every leader
    /// that is mid-search is told to start again.
    fn caravans_reset_paths(&mut self) {
        for l in &mut self.caravans {
            for v in &mut l.slots {
                if v.alive && v.making_road {
                    v.reset_road = true;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // §6 — `Unit::work@0060d180`'s caravan block
    // ------------------------------------------------------------------

    /// The retry, ahead of the order dispatch: a caravan whose route is
    /// linked and whose current action is still a `TRADE_ROUTE` runs another
    /// frame of `build_road` while one is parked; anything else ends the
    /// route.
    pub(crate) fn caravan_work(&mut self, u: usize) {
        let Some(v) = self.units[u].caravan else {
            return;
        };
        let who = self.units[u].owner;
        if !self.caravans[who as usize].slots[v].linked {
            return;
        }
        if self.order_type(u) == index::CAST_SPELL {
            return;
        }
        // The test is on the **action**, not the current order: while the
        // road is unfinished the caravan is walking, so a plain move is
        // current and the `TRADE_ROUTE` sits behind it as the intent.
        let is_trade = self
            .action_of(u)
            .is_some_and(|i| self.units[u].orders[i].index() == index::TRADE_ROUTE);
        if !is_trade {
            self.end_trade_route(u);
            return;
        }
        if self.caravans[who as usize].slots[v].making_road {
            self.caravan_build_road(who, v);
        }
        // SEAM: `work@0060d180:114`'s tail — when the *current* order is
        // not a `TRADE_ROUTE` the original also tests the one before the
        // list's last and ends the route if that is not one either. It
        // cannot fire while the only two orders are this pair.
    }

    /// `Unit::end_trade_route@005ed1c0`: the route is given up — the pair
    /// forgotten, the plan and any parked search with it.
    pub(crate) fn end_trade_route(&mut self, u: usize) {
        let Some(v) = self.units[u].caravan else {
            return;
        };
        let who = self.units[u].owner;
        self.clear_temp_road(who, v);
        let van = &mut self.caravans[who as usize].slots[v];
        van.road.clear();
        van.linked = false;
        van.city_a = None;
        van.city_b = None;
        van.reset_road = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare() -> Sim {
        Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2)
    }

    #[test]
    fn a_slot_is_reused_and_the_mark_walks_back_down() {
        let mut sim = bare();
        let a = sim.init_caravan(1, 7).expect("a free slot");
        let b = sim.init_caravan(1, 8).expect("a free slot");
        assert_eq!((a, b), (0, 1));
        assert_eq!(sim.caravans[1].mark, 2);
        sim.close_caravan(1, 0);
        // The mark stays where the higher slot holds it.
        assert_eq!(sim.caravans[1].mark, 2);
        assert_eq!(sim.init_caravan(1, 9), Some(0), "the hole is reused first");
        sim.close_caravan(1, 1);
        sim.close_caravan(1, 0);
        assert_eq!(sim.caravans[1].mark, 0, "and then it walks back down");
    }

    #[test]
    fn twenty_slots_and_no_more() {
        let mut sim = bare();
        for i in 0..CARAVAN_SLOTS {
            assert_eq!(sim.init_caravan(0, i), Some(i));
        }
        assert_eq!(sim.init_caravan(0, 99), None, "the list is twenty long");
    }
}
