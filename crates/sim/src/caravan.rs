//! Trade routes — the `Caravan` object a caravan unit owns, and the road it
//! plans between two cities.
//!
//! `docs/CARAVAN.md`. The mechanic is one object and five callers:
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
//!    What it leaves behind is a stack of world-unit waypoints, not a list
//!    of tiles (§5.3), and that stack is the route.
//! 5. **The legs** (§7). `do_trade`'s tail copies the route's stack onto the
//!    unit, offsets it a third of a step sideways, and issues a `pathed`
//!    move to its far end — then inverts the route so the next leg reads
//!    the other way. The caravan shuttles: it loads at the far city,
//!    unloads at the home one, and the round trip is what makes the route
//!    worth anything (§7.2).
//!
//! A trade route between two of East Indies' cities costs 12,965 road draws
//! over five frames, and until 2026-09-02 this crate spent none of them.

use crate::orders::{Body, PathData, QueuePos, TradeOrder, flag, index, path_flag};
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
    /// gate is `& 6`.
    pub linked: bool,
    /// `caravan_flags & 4`: the route has completed a round trip — written
    /// where the caravan unloads at the **home** city, and the bit
    /// `City::compute_trade` counts, so a route that has not returned once
    /// is worth nothing (§7). `Caravan::restart_trade_route@0073d070` is
    /// its other writer and no traced game enters it.
    pub delivered: bool,
    /// `city2`/`whom` (+0x0, +0x2) — the **home** city, and the search's
    /// start.
    pub city_a: Option<usize>,
    /// `city3`/`whose` (+0x4, +0x6) — the far city, and the goal.
    pub city_b: Option<usize>,
    /// `o` (+0xa) — the caravan unit.
    pub unit: Option<usize>,
    /// `road` (+0x10): the plan as `Stack<PathData>` — **world** positions,
    /// near-goal end at index 0, which is the stack's own bottom. Its
    /// tolerances and flags are `build_road`'s, not the search's (§5.3), and
    /// it is what the legs walk (§7.1).
    pub road: Vec<PathData>,
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
                loaded: false,
            }),
        };
        self.enqueue_order(u, order, QueuePos::New);
    }

    // ------------------------------------------------------------------
    // §4 — `Unit::do_trade@005ed270`
    // ------------------------------------------------------------------

    /// The order's own step. While a road is being planned the function
    /// returns at its head, which is why the retry lives in `Unit::work`;
    /// once one is laid, every later call is an **arrival** (§7).
    pub(crate) fn do_trade(&mut self, u: usize) {
        // **The first instruction of the step**, ahead of the caravan-slot
        // test and every one of the returns below: `do_trade@005ed270+0x40`
        // is the return address of `Unit::set_anim(CHAR_DEFAULT, 0, 1)`.
        // It draws nothing while the caravan is walking — a walk-category
        // guy whose body has not arrived leaves `Guy::set_anim` without a
        // roll — so its draws are the frames the unit is *standing* at a
        // city, which is why run54's word parted at 6198 and not at 6166.
        self.mark(crate::anim::SITE_TRADE);
        self.set_default_anim(u);
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
        let Some(dest) = ord.dest else { return };
        if ord.started {
            self.trade_legs(u, v, ord);
            return;
        }
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
        // **The facing is the literal pair, not a bearing.**
        // `Unit::add_move_order@00616ed0` opens with
        // `find_angle(param_3, param_4)` — the two arguments that are the
        // *order kind* and the pathed flag, not a direction — so every
        // caller's arrival angle is a constant. `do_trade`'s failure arm
        // passes `(1, 0)`, which is due east; its leg passes `(1, 1)`.
        if self.caravan_build_road(who, v) < 0 {
            let to = self.cities[a].pos;
            self.add_move_facing_order(
                u,
                to,
                crate::orders::MoveKind::MoveTo,
                QueuePos::First,
                false,
                crate::movement::find_angle(1, 0),
                None,
                false,
            );
            return;
        }
        self.trade_legs(u, v, ord);
    }

    // ------------------------------------------------------------------
    // §7 — the legs, `do_trade@005ed270+0xae6` and `+0xdaa`
    // ------------------------------------------------------------------

    /// The tail of `do_trade`, reached on the frame the route is
    /// established and on every later frame the order is current again —
    /// which is every frame the caravan **arrives**, because the move a leg
    /// queues goes in with `QUEUE_FIRST` and `do_trade` does not run under
    /// it.
    ///
    /// Two halves. The first is the arrival test, made against **one** of
    /// the two cities — the far one while the caravan is empty and the home
    /// one while it is carrying — and it is what turns `loaded` over, marks
    /// the route as having delivered, and pays. The second is the leg
    /// itself (§7.1).
    fn trade_legs(&mut self, u: usize, v: usize, mut ord: TradeOrder) {
        let who = self.units[u].owner;
        let home = ord.home;
        let Some(dest) = ord.dest else { return };
        // The preamble's refusals: both ends are still cities, and alive.
        //
        // SEAM: the original re-reads each end through its **object** and
        // checks `is_active_wallbuild` and the vtable's own "am I a city",
        // which is how a route survives its city being replaced in the
        // slot; this crate holds the city index and asks the city.
        if !self.cities[home].alive || !self.cities[dest].alive {
            self.kill_current_order(u);
            if self.current_order(u).is_none() {
                self.think_caravan(u);
            }
            return;
        }
        let target = if ord.loaded { home } else { dest };
        let span = self.city_span(target);
        let here = self.units[u].pos;
        let c = self.cities[target].pos;
        if (here.x - c.x).abs() <= span + 0x306 && (here.y - c.y).abs() <= span + 0x306 {
            ord.loaded = !ord.loaded;
            self.set_trade_order(u, ord);
            // `unit_masks |= 0x200` goes with the unload, and nothing in
            // the export reads the bit back — `do_trade` is its only
            // mention — so it is not modelled.
            //
            // A **decoy** does none of the rest: `unit_masks & 1`.
            if !self.units[u].decoy {
                if ord.loaded {
                    self.city_new_caravan(dest, self.cities[home].owner, home);
                } else {
                    // The home city is where a round trip closes, and the
                    // only place `caravan_flags & 4` is written — which is
                    // the bit `City::compute_trade` counts, so a route pays
                    // nothing at all until its first full return.
                    self.caravans[who as usize].slots[v].delivered = true;
                    self.compute_trade(home);
                    self.compute_trade(dest);
                    self.city_new_caravan(home, self.cities[dest].owner, dest);
                }
            }
        }
        self.trade_walk(u, v, if ord.loaded { dest } else { home }, span);
    }

    /// §7.1 — `LAB_005ee01a`, the leg.
    ///
    /// With a road planned the caravan walks **the route's own stack**: it
    /// is oriented so the end nearest the unit is on top, copied whole onto
    /// the unit's path, smoothed, and then the top — the node under the
    /// unit's feet — is popped off and a `QUEUE_FIRST` move issued to the
    /// stack's *bottom*, which is the far end. The move carries
    /// [`flag::PATHED`], so `do_move` walks what is already there rather
    /// than planning; and the route's stack is inverted on the way out, so
    /// the next leg reads the other way.
    ///
    /// The smoothing is a **perpendicular** offset, not a shortening: each
    /// node past the first takes `+dy/3` on x and `−dx/3` on y, where
    /// `(dx, dy)` is the step from the node before it and both are read
    /// before either is written. It applies only where the step is inside
    /// `0xc0` on both axes — one tile — so a leg that jumps a sampled water
    /// run is left alone. The listing settles the arithmetic
    /// (`5ee174`–`5ee1a1`): the second half's magic is `0x55555555` with a
    /// `sub`/`sar`, which is `x / −3` and not `x / 3`.
    ///
    /// `fallback` is the city the walk aims at when there is no road, and
    /// `span` the footprint of the city the arrival test above used — the
    /// original's `local_38`, which is deliberately the **other** city's.
    fn trade_walk(&mut self, u: usize, v: usize, fallback: usize, span: i32) {
        let who = self.units[u].owner as usize;
        if self.caravans[who].slots[v].road.is_empty() {
            self.trade_walk_no_road(u, fallback, span);
            return;
        }
        let here = self.units[u].pos;
        let road = &self.caravans[who].slots[v].road;
        let (first, last) = (road[0].to, road[road.len() - 1].to);
        let d_bottom =
            crate::world::vector_dist((first.x - here.x).abs(), (first.y - here.y).abs());
        let d_top = crate::world::vector_dist((last.x - here.x).abs(), (last.y - here.y).abs());
        if d_bottom < d_top {
            self.caravans[who].slots[v].road.reverse();
        }
        let mut path = self.caravans[who].slots[v].road.clone();
        offset_road(&mut path);
        self.units[u].path = path;
        self.units[u].path.pop();
        let goal = self.units[u].path[0].to;
        self.add_move_facing_order(
            u,
            goal,
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
            crate::movement::find_angle(1, 1),
            None,
            true,
        );
        // The next waypoint in another region is a **transport** waypoint:
        // the same `path_flag::TRANSPORT` the tile grid writes, popped and
        // pushed back rather than edited in place (`docs/PATHFINDER.md` §7).
        if let Some(top) = self.units[u].path.last().copied() {
            let mine = self.world.tregion(here.tile());
            if self.world.tregion(top.to.tile()) != mine {
                self.units[u].path.pop();
                self.units[u].path.push(PathData {
                    flags: top.flags | path_flag::TRANSPORT,
                    ..top
                });
            }
        }
        self.caravans[who].slots[v].road.reverse();
    }

    /// The no-road arm of §7.1: a spot beside the target city, and a move
    /// to it if one was found close enough.
    ///
    /// SEAM: `UnitType::find_nearby_spot@0067d0d0`'s ring — between `span`
    /// and `span + 0xc0` of the city, swept from `find_angle(1, 0)`,
    /// `FILTER_NOT_ME` — is not modelled; this crate walks to the city's
    /// own point. No traced game reaches the arm: a route whose `build_road`
    /// answered 0 has no caravan on it.
    fn trade_walk_no_road(&mut self, u: usize, city: usize, span: i32) {
        let to = self.cities[city].pos;
        let here = self.units[u].pos;
        if crate::world::vector_dist((to.x - here.x).abs(), (to.y - here.y).abs()) > span + 0xc6 {
            return;
        }
        self.add_move_facing_order(
            u,
            to,
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
            crate::movement::find_angle(1, 0),
            None,
            false,
        );
    }

    /// `max(x_size, y_size) · 0x60` of a city's own building type — the
    /// original's `local_38`, half a footprint in world units, and what the
    /// arrival box is measured out from.
    fn city_span(&self, city: usize) -> i32 {
        let b = self.cities[city].building;
        self.buildings[b].ty.map_or(0, |ty| {
            let t = &self.build_types[ty];
            if t.x_size > t.y_size {
                t.x_size * 0x60
            } else {
                t.y_size * 0x60
            }
        })
    }

    /// `City::compute_trade@00739640`: `trade_val` is the sum, over every
    /// route this city is an end of that has **delivered at least once**
    /// and whose other end is still alive, of `trade_value(other) · 16 / 2`.
    ///
    /// It is what a city contributes to its owner's wealth rate on top of
    /// its gatherers (`crate::economy::city_rates`), and a change marks the
    /// leader's economy dirty.
    pub(crate) fn compute_trade(&mut self, city: usize) {
        let was = self.cities[city].trade_val;
        let mut val = 0;
        for w in 0..self.caravans.len() {
            for s in 0..self.caravans[w].mark {
                let van = &self.caravans[w].slots[s];
                if !van.delivered {
                    continue;
                }
                let (Some(a), Some(b)) = (van.city_a, van.city_b) else {
                    continue;
                };
                if !self.cities[a].alive || !self.cities[b].alive {
                    continue;
                }
                let other = if a == city {
                    b
                } else if b == city {
                    a
                } else {
                    continue;
                };
                val += self.trade_value(other, city) * 16 / 2;
            }
        }
        self.cities[city].trade_val = val;
        if was != val {
            let owner = self.cities[city].owner;
            self.economy_changed(owner);
        }
    }

    /// `City::new_caravan@00739750`: the **one-off** a leader is paid the
    /// first time one of its caravans reaches this city from a given
    /// partner.
    ///
    /// `traded_with[who]` is a bit per partner city, so the bonus is paid
    /// once per (city, leader, partner). The amount is the leader's Civic
    /// level plus one, times ten — times **twenty** when the leader is not
    /// this city's owner — and it lands in wealth.
    ///
    /// SEAM: the feedback line and its sound are the console player's only.
    fn city_new_caravan(&mut self, city: usize, who: Player, partner: usize) {
        let w = who as usize;
        let Some(bit) = u32::try_from(partner).ok().filter(|b| *b < 32) else {
            return;
        };
        let mask = 1u32 << bit;
        if self.cities[city].traded_with[w] & mask != 0 {
            return;
        }
        self.cities[city].traded_with[w] |= mask;
        let level = self.tech[w].epoch[crate::tech::Line::Civic.index()] + 1;
        let pay = if who == self.cities[city].owner {
            level * 10
        } else {
            level * 20
        };
        self.ledgers[w].bucket[crate::economy::Resource::Wealth as usize] += pay;
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
    ///
    /// **The distance is in cells, not world units**, and the width it is
    /// compared against is `WorldData::xs` — the same grid.
    /// `Caravan::trade_value@0073d9d0` hands `distance` four
    /// `div_3_table[coord >> 8]` values, which is `floor(floor(x / 256) / 3)`
    /// — this crate's [`Pos::cell`](crate::world::Pos::cell), one coordinate
    /// at a time, the same double floor `produce_building`'s builder
    /// distance takes (`docs/AI.md` §2.20). Measuring the world-unit
    /// distance instead put every pair in band 3 and doubled every trade
    /// route's value (`docs/CARAVAN.md` §3.1).
    fn trade_distance(&self, a: usize, b: usize) -> i32 {
        let (pa, pb) = (self.cities[a].pos.cell(), self.cities[b].pos.cell());
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
    ///
    /// The count is `CityData::num_buildings@00738190` — the whole chain
    /// **including the city building itself**, and only its *finished*
    /// members — not the member list's length. A city of nine buildings
    /// scored eight (`docs/CARAVAN.md` §3.1).
    fn city_trade_value(&self, c: usize) -> i32 {
        let n = self.num_buildings(c);
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
            RoadPlan::Road(nodes) => {
                let road = self.lay_caravan_road(&nodes);
                let van = &mut self.caravans[w].slots[v];
                van.road = road;
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

    /// `Caravan::build_road@0073db10:82` — the loop that turns the search's
    /// answer into the route's own stack, and the only place a trade road
    /// is written to the map.
    ///
    /// It walks the search's stack **from the top down**, which is the
    /// near-*start* end first, into a scratch stack that is then popped
    /// back on — so the orientation survives and the decisions inside are
    /// made in walking order.
    ///
    /// Two things happen per node. A node on **open water** lays nothing
    /// and is *sampled*: the first of a run is kept with a tolerance of
    /// `0x180` and sets a countdown of four, and the next four are dropped
    /// outright — unless fewer than five nodes have been written or fewer
    /// than five are left, at the two ends, where every one is kept.
    /// Everything else lays a road tile and takes [`path_flag::ROAD`]. Both
    /// ends of what survives take [`path_flag::FINAL`].
    ///
    /// The tolerance is the search's own `0x60` (`astar_caravan_road`
    /// `+0x9a6`) everywhere else.
    fn lay_caravan_road(&mut self, nodes: &[Pos]) -> Vec<PathData> {
        let mut out: Vec<PathData> = Vec::with_capacity(nodes.len());
        let mut run: i32 = 0;
        for i in (0..nodes.len()).rev() {
            let p = nodes[i];
            let t = p.tile();
            let mut e = PathData {
                to: p,
                tolerance: 0x60,
                flags: 0,
            };
            if self.world.tile_mask(t) & crate::world::tile::SURFACE
                == crate::world::tile::SURFACE_OCEAN
            {
                if run != 0 && i >= 5 && out.len() >= 5 {
                    run -= 1;
                    continue;
                }
                run = 4;
                e.tolerance = 0x180;
            } else {
                self.set_road_at(t);
                e.flags |= path_flag::ROAD;
                run = 0;
            }
            if out.is_empty() || i == 0 {
                e.flags |= path_flag::FINAL;
            }
            out.push(e);
        }
        out.reverse();
        out
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

/// §7.1's smoothing pass, in place: every node past the first is displaced
/// a third of the step that reaches it, **turned a quarter turn** — `+dy/3`
/// on x and `−dx/3` on y — so the caravan walks beside its road rather than
/// down the middle of it. A step longer than `0xc0` on either axis, which
/// is what a sampled water run leaves, is left alone.
///
/// Both deltas are measured against the previous node **before** it was
/// displaced, so the offsets do not compound.
fn offset_road(path: &mut [PathData]) {
    let Some(first) = path.first().map(|p| p.to) else {
        return;
    };
    let mut prev = first;
    for e in path.iter_mut().skip(1) {
        let here = e.to;
        let (dx, dy) = (here.x - prev.x, here.y - prev.y);
        prev = here;
        if dx.abs() > 0xc0 || dy.abs() > 0xc0 {
            continue;
        }
        e.to.x += dy / 3;
        e.to.y -= dx / 3;
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

    fn node(x: i32, y: i32) -> PathData {
        PathData {
            to: Pos::new(x, y),
            tolerance: 0x60,
            flags: 0,
        }
    }

    /// §7.1: the displacement is the step turned a quarter turn, and the
    /// step is measured from where the *search* left the node before.
    ///
    /// Written from run64's own road, whose last three nodes are the
    /// straight north run `(38496, 39264)`, `(38496, 39456)`,
    /// `(38496, 39648)`: a step of `(0, 192)` puts `+64` on x and nothing
    /// on y, twice over, and the second `+64` is off `38496` and not off
    /// the `38560` the first one wrote.
    #[test]
    fn the_offset_is_perpendicular_and_does_not_compound() {
        let mut p = vec![node(38496, 39264), node(38496, 39456), node(38496, 39648)];
        offset_road(&mut p);
        assert_eq!(p[0].to, Pos::new(38496, 39264), "the first node is fixed");
        assert_eq!(p[1].to, Pos::new(38560, 39456));
        assert_eq!(p[2].to, Pos::new(38560, 39648));

        // The other axis, and the sign: a step due **east** displaces
        // south by `−dx/3`, which is negative y.
        let mut q = vec![node(0, 0), node(192, 0)];
        offset_road(&mut q);
        assert_eq!(q[1].to, Pos::new(192, -64));

        // Truncation is toward zero, as the original's two divisions are.
        let mut r = vec![node(0, 0), node(-100, -100)];
        offset_road(&mut r);
        assert_eq!(r[1].to, Pos::new(-100 - 33, -100 + 33));
    }

    /// A step wider than one tile on either axis is left where it is —
    /// which is the arm a sampled water run takes, since the sampling
    /// leaves gaps of five tiles.
    #[test]
    fn a_step_past_one_tile_is_not_displaced() {
        let mut p = vec![node(0, 0), node(0, 0xc0), node(0, 0xc0 + 0xc1)];
        offset_road(&mut p);
        assert_eq!(p[1].to, Pos::new(0x40, 0xc0), "0xc0 exactly is inside");
        assert_eq!(p[2].to, Pos::new(0, 0xc0 + 0xc1), "0xc1 is not");
    }

    /// §7.3: the one-off is paid once per (city, leader, partner), and it
    /// is `(epoch[1] + 1) · 10` — twenty when the payee is not the city's
    /// own owner.
    #[test]
    fn the_arrival_bonus_is_paid_once_and_doubles_for_a_foreigner() {
        let mut sim = bare();
        sim.cities.push(crate::city::City {
            alive: true,
            owner: 0,
            race: Some(0),
            founder: 0,
            building: 0,
            members: Vec::new(),
            reg: None,
            pos: Pos::new(0, 0),
            capital: false,
            founding_capital: false,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            no_muster: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 0,
            has_citizen: false,
            source: None,
            trade_val: 0,
            traded_with: [0; 8],
        });
        let wealth = crate::economy::Resource::Wealth as usize;
        for l in &mut sim.ledgers {
            l.bucket[wealth] = 0;
        }
        sim.tech[0].epoch[crate::tech::Line::Civic.index()] = 2;
        sim.tech[1].epoch[crate::tech::Line::Civic.index()] = 2;

        sim.city_new_caravan(0, 0, 5);
        assert_eq!(
            sim.ledgers[0].bucket[wealth], 30,
            "(2 + 1) × 10, the owner's"
        );
        sim.city_new_caravan(0, 0, 5);
        assert_eq!(sim.ledgers[0].bucket[wealth], 30, "the bit is already set");
        sim.city_new_caravan(0, 0, 6);
        assert_eq!(
            sim.ledgers[0].bucket[wealth], 60,
            "a different partner pays"
        );
        sim.city_new_caravan(0, 1, 5);
        assert_eq!(
            sim.ledgers[1].bucket[wealth], 60,
            "(2 + 1) × 20 for a leader who does not own the city"
        );
    }
}
