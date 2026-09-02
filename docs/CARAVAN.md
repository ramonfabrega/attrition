# Caravans and trade routes

*Established 2026-09-02 from the decompile (`tools/ghidra/`), the PE
listing (`llvm-objdump`) where the decompiler's locals could not be
trusted, the PDB's own type record for `CaravanData` and its
`FilterIndex` enum, and one oracle: **run64**, which is run54's game with
a `DUMP_ALL` window on `[6164, 6172)` and `docs/ROADS.md` §7.2's three
road proxies over the frames a caravan's road is planned on. Confidence:
**high** for the object, the schedule and the search — every one of the
12,965 nodes the original priced over five frames is asserted against its
own record — and **medium** for the destination choice, which one capture
with two cities cannot separate from several simpler rules. The legs of
the route are not established at all: nothing here moves cargo or pays
wealth.*

The mechanic is `docs/QUEUE.md`'s item 174. It was booked as a road and
turned out to be a whole object: **a caravan unit owns a `Caravan`**, the
route between two cities, and the road is the route's first act.

`docs/ROADS.md` opens by saying its search's name was wrong — that
`astar_caravan_road` runs for a *building* and no caravan is in the game.
On East Indies' frame 6166 one is, and the arm the building never takes
is the one this document is about.

## 1. The object — `CaravanData`, 0x44 bytes

The type record names every field, which is why none of this is inferred
from use:

| offset | name | what it is |
| --- | --- | --- |
| `+0x0`, `+0x2` | `city2`, `whom` | the **near** city and its owner — the search's start |
| `+0x4`, `+0x6` | `city3`, `whose` | the far city and its owner — the goal |
| `+0x8` | `cara` | this route's own slot |
| `+0xa` | `o` | the caravan **unit**'s object number |
| `+0xc` | `caravan_flags` | `1` alive, `2` linked, `4` `restart_trade_route`'s |
| `+0xd` | `who` | the owner |
| `+0x10` | `road` | `Stack<PathData>` — the plan, near-goal end first |
| `+0x20` | `making_road` | a search is parked and wants another frame |
| `+0x24` | `reset_road` | the map changed; throw the parked search away |
| `+0x28`, `+0x2c`, `+0x30` | `openlist`, `openlistrefs`, `closedlist` | the parked search's three containers |
| `+0x34` | `offset` | the wheel preference it stopped on |
| `+0x38`, `+0x3c` | `endx`, `endy` | the goal, in world units |
| `+0x40` | `traversed` | written, and **never read back** (§5.2) |

`Caravans::init@0073e870` gives every leader **twenty** of them, and
`LeaderData +0x438` is the mark — the same shape as the docks
(`docs/TRANSPORT.md`). A `DUMP_ALL` block prints them under `CARAVANS`,
six fields and the road stack; run13's frame 95 is twenty empty ones.

## 2. Birth and death

`Unit::init@00612100:436` calls `Caravans::init_caravan(who, o)` for a
unit that `is_caravan` — `unit_flags2 & 8` — **and whose type domain is
land**, which is `UnitTypeData +0x218 == 0`. That second test is what
keeps the sea-domain Merchant Fleet out: it takes the `Specials` list
instead and never owns a route. The slot goes in the unit's `+0x86`, the
union the same field uses for a hero's slot and an animal's herd.

`init_caravan` takes the first slot below the mark whose `alive` bit is
clear, else the mark itself; `Caravans::close_caravan@0073e350` clears
the record and walks the mark back down past every dead slot at the top.

## 3. The order — `Unit::think_caravan@005f5650`

Step 5 of `Unit::think` (`docs/ORDERS.md` §2.4), and it sits **above**
the tail's own thirty-two-frame gate: `think_caravan` has a cadence of
its own and no other.

```
threshold = (unit_masks & 0x40000) ? 1                 ← AI-driven
          : {1:7, 2:12, 3:17, 4:32, 5:62}[difficulty] or 2
if idle <  threshold                     → nothing
if idle != threshold and (idle−2) % 5    → nothing
city = find_city(pos, SEARCH_ALLIED, who, FILTER_CAN_TRADE)
if city >= 0 and city is alive           → add_trade_order(city, QUEUE_NEW)
```

So an AI caravan asks on its **first** idle frame and a human's on its
second, and both retry every fifth frame after. East Indies' caravan is
born on 6164 and its order is on 6165.

`FILTER_CAN_TRADE` is arm 17 of `Search::valid_filter@0067dbb0`'s jump
table — the enum is `FILTER_TYPE = 1`, `FILTER_CAN_TRADE = 18`, the table
is at `0067e57c` and the arm at `0067e204`, read from the PE because the
decompiler prints the table as an indirect jump. The arm is §4's own loop
with the scoring taken out: *is there any allied city, not this one, that
this city and it can both still pair with*. `ObjectsData::find_city@0065ba90`
takes the nearest that passes, with no region flag and no distance limit,
and ties go to the **later** city because the compare is `<=`.

## 4. The route — `Unit::do_trade@005ed270`

The order's own step, and its head is the reason the retry lives
somewhere else:

```
if caravans[who][this->caravan].making_road != 0:  return
```

While a road is being planned `do_trade` does nothing at all.

With no far city yet (`TradeOrder +0x14` is −1) it runs the selection
loop: for every leader `w` with `is_ally(me, w)` — and, when `w` is not
me, only if I `has_preq(BASE_BONUSTYPES)` and the home city is my own —
and every city `c` of that leader:

- not the home city itself;
- `c` alive, and `CityData::is_seen@00739560(c, me)`;
- `get_empty_trade_routes(home, c, w)` and `get_empty_trade_routes(c,
  home, homeWho)` both non-zero;
- `c`'s region is the caravan's own, or the caravan `can_transport`.

The score is `Caravan::trade_value@0073d9d0`, **quadrupled** when the home
city is the caravan owner's own, and the highest wins:

```
v = get_trade_value(a) + get_trade_value(b)
d = distance band                                (0 under ¼ the map's width,
                                                  1 under ½, 2 under ⅘, 3 beyond)
if d: v = (d + 3)·v / 3
if the two cities have different owners: v = v·3/2
if the Indian tribe bonus:  v = v·(indians_caravan + 100)/100
if the spice rare:          v = v·(spice_caravan_income + 100)/100
```

`CityData::get_trade_value@007363f0` is the city's building count, plus
two for a Large City (`TOWN`) and four for a Major City or the Forbidden
City.

Then the two cities are ordered by **distance from the caravan** — the
nearer becomes `city2`, the search's start — both cities gain a
`CaravanLink`, `caravan_flags |= 3`, both `City::compute_trade`, and
`build_road` runs.

### 4.1 The failure arm, and why it matters to the stream

```
if build_road() < 0:
    add_move_order(city2.x, city2.y, …, QUEUE_FIRST)
    return
```

A plan that stopped at the budget sends the caravan **walking to the near
city** while the search continues. That walk is not decoration: its
figures go into a walking animation, and their clocks are draws
(`docs/ANIM.md`). The move goes in ahead of the `TRADE_ROUTE`, which
stays behind it as the *action* — which is why §6's gate reads
`get_action` and not the current order.

SEAM: the original's arrival facing is `find_angle(1, 0)`, the literal
pair `do_trade` passes where `add_move_order@00616ed0` reads a direction.
Nothing reads it until the unit arrives.

## 5. The road — `Caravan::build_road@0073db10`

`clear_road`, then four refusals (either city index negative, either city
not alive), then a three-way gate:

```
making_road and not reset_road and openlist != 0  → find_road_restore(…)
otherwise                                          → clear_temp_road if
                                                     reset_road or no openlist;
                                                     find_road(…)
```

Both calls carry the same arguments — `(city2.o, whom, city3.o, whose,
unit o, who)` — and differ only in `PathFinder::find_road_restore@00685950`
setting `pathfinder+0x84` around the call. That flag makes `find_road`
skip the endpoint bounds test and the two `Stack<PathData>::push`es, so
the search resumes instead of starting.

The answer is the original's: **1** the road was laid, **0** there was
nothing to plan, **−1** the budget stopped it. On 1 the plan is walked
down from the top, each tile `World::set_road_at`, `reset_road` and
`making_road` cleared, and `Caravans::reset_paths@0073e060` tells every
*other* leader's part-planned route to start again — the map just
changed under it.

`Caravan::process@0073e000` is `build_road` again, COMDAT-adjacent and
gated on `making_road`; no traced game enters it.

### 5.1 `can_transport`, and the ocean arm coming alive

`find_road@00688a40+0x2ac` sets `pathfinder+0x98` from the **caravan
unit** — `UnitData::can_transport@0046f960`, which is `(unit_masks &
0x800000 && !(unit_masks2 & 0x2000)) || (unit_flags & 0x10)` — and leaves
it zero for a building, whose `param_6` is −1.

With it set, `valid_roadcoord` stops refusing ocean tiles and
`calc_road_cost`'s ocean arm — dead for every building road
(`docs/ROADS.md` §5.2) — prices them at `jitter + 55 + 100`, the
`avoid_sea` figure, because both endpoints are in one region. East
Indies' caravan has the bit, and its route walks over the water between
two coasts: run64's frame 6167 is 3,204 nodes and one of them is
`(36384, 43488)`, an ocean tile priced at 231.

### 5.2 The budget, and the search that spans five frames

`astar_caravan_road@00685990`'s budget arm (`traversed >= 0xc80`) has two
halves, and which one runs is `param_6 >= 0`:

- **a building** recycles the popped node and returns −1: the road is
  simply not laid;
- **a caravan** puts the popped node **back on the open list**, moves the
  three containers into the `CaravanData` with `offset`, `endx`, `endy`
  and `traversed`, sets `making_road`, and returns −1.

Next frame `find_road_restore` hands them back and the search carries on.
**`traversed` is written and never read**: `local_2c` is zeroed at the top
of the function and the restore arm reads `+0x34`, `+0x38` and `+0x3c`
and nothing else. So every resumed frame gets a fresh budget of 3,200
nodes, which is why one trade route costs 3,204 + 3,204 + 3,204 + 3,202 +
151 draws on East Indies' frames 6166 to 6170 rather than 3,204 once.

`docs/ROADS.md` §8 has the search's own three differences — the eight
directions, the heuristic, and the diagonal corner rule.

## 6. `Unit::work@0060d180`'s block

Ahead of the order dispatch, for a unit that `is_caravan` whose route has
`caravan_flags & 6` and whose order type is not `CAST_SPELL`:

```
action = get_action()
if action is null or action.type != TRADE_ROUTE  → end_trade_route()
else:
    if making_road: build_road()
    if order_type() != TRADE_ROUTE and the order before the list's last
       is not one either                          → end_trade_route()
```

This is the retry: `do_trade` returns at its head while `making_road`
stands, so the frames after the first are all `Unit::work`'s. The test is
on the **action**, because by then the caravan is walking §4.1's move and
a plain move is current.

SEAM: the tail's second test cannot fire while the only two orders are
that pair, and is not modelled.

## 7. Coverage

**Diff-backed** (`rondata::diff::tests::run64_s_caravan_road_is_the_
original_s_node_for_node`):

- **The whole search, node for node**: every one of the 3,204 + 3,204 +
  3,204 + 3,202 prices of run64's frames 6166 to 6169 — the tile, the
  direction it was reached from and the price — against the original's
  own proxied `calc_road_cost` answers, and the 151 of 6170 by tile and
  direction (its jitters are two draws out of phase; `docs/QUEUE.md`
  item 176).
- **The five frames' node counts**, which is the budget rule and the
  never-restored `traversed` together.
- **The endpoints**, from the `astar_caravan_road` bracket: `oA 2000`,
  `whoA 1`, `oB 2007`, `caravan 0`.
- **The world the search reads**: all 57,600 tile masks' heights and all
  3,600 cell owners of run64's frame-6166 block. The heights are the
  sharp half — `Wall::init`'s farm gate (`docs/ROADS.md` §7.4) was found
  by 182 of them.
- The order and the schedule, indirectly: East Indies' word, which is a
  per-frame draw *sequence*, now passes 6166 to 6168 whole.

**Reading-only, and each names the capture that would falsify it:**

- §3's idle thresholds — the five difficulty arms. A capture with a human
  caravan and `LEADERS=9` would show the option; none has one.
- §4's selection loop past the two-city case. run64 has one route between
  the only two cities that could pair, so the scoring, the ally arm, the
  `BASE_BONUSTYPES` gate and the two bonuses are all unexercised. A
  capture with three cities of one leader would separate them.
- `CityData::is_seen`, taken here as "the owner has seen the tile"; the
  original keeps a per-leader bit on the city.
- §5's `reset_road` path, which needs two routes planning at once.
- `Caravan::restart_trade_route@0073d070`, `Caravans::new_danger@0073e0c0`
  and `Caravan::verify_road@0073d950` — no traced game enters any of them.

**Not established at all:** everything past the road. The legs between the
two cities, `TradeOrder +0x20 loaded`, the wealth a completed trip pays
(`docs/ECONOMY.md`'s `trade_val`), and what happens when a city on the
route falls. A caravan in this crate plans its road, walks to the near
city and stops.
