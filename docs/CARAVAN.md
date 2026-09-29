# Caravans and trade routes

*Established 2026-09-02 from the decompile (`tools/ghidra/`), the PE
listing (`llvm-objdump`) where the decompiler's locals could not be
trusted, the PDB's own type record for `CaravanData` and its
`FilterIndex` enum, and one oracle: **run64**, which is run54's game with
a `DUMP_ALL` window on `[6164, 6172)` and `docs/ROADS.md` §7.2's three
road proxies over the frames a caravan's road is planned on. Confidence:
**high** for the object, the schedule and the search — every one of the
12,965 nodes the original priced over five frames is asserted against its
own record, and its whole twenty-six-node stack against the `CARAVAN`
block that prints it — and **medium** for the destination choice, which
one capture with two cities cannot separate from several simpler rules,
and for the legs (§7), whose shape the word confirms for nine frames and
whose arithmetic no capture yet reaches.*

The mechanic is `docs/QUEUE.md`'s items 174 and 177. It was booked as a
road and turned out to be a whole object: **a caravan unit owns a
`Caravan`**, the route between two cities, and the road is the route's
first act — and then the route's stack is what the caravan walks, back and
forth, for the rest of the game.

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
                                                 — **in cells**, see below
if d: v = (d + 3)·v / 3
if the two cities have different owners: v = v·3/2
if the Indian tribe bonus:  v = v·(indians_caravan + 100)/100
if the spice rare:          v = v·(spice_caravan_income + 100)/100
```

**Which leader the two bonuses ask** (item 1189, off the listing).
`trade_value` takes one end in `ecx`/`edx`, city and owner, and the
decompiler drops both. The foreign test and both bonuses read
`leaders.list[edx]`. `do_trade` (`005ed65a`) passes the home there when
the caravan is the home owner's, and the partner otherwise.
`compute_trade` (`007396f4`) passes the end whose owner is the computing
city's, so there it is always that city's own owner. The rare test is
`+0x6da4 & 0x40` or `+0x6dcc & 0x40`: `rare` and `rare_conquest` are
`BitMask<44>`s at `+0x6d98` and `+0x6dc0` whose bytes open at `+0xc`, so
this is bit 6, good 12, **Spice**. `SPICE_CARAVAN_INCOME` ships as `20%`
and `INDIANS_CARAVAN` as 15 (`rules.xml`). Neither was read here until
item 1189. Great Sahara's who=1 holds Spice (`rares_collected[6]`), and
from frame 14660 its two Villages' route is worth 22 · 120/100 = 26, so
`trade_val` is 208 at each end where this crate read 176.

`CityData::get_trade_value@007363f0` is the city's building count, plus
two for a Large City (`TOWN`) and four for a Major City or the Forbidden
City. **The count is `CityData::num_buildings@00738190`** — the member
chain walked from the city's own `o`, so the **city building itself is in
it**, and only its *finished* members are. Not the member list's length:
a city of nine buildings scores nine, not eight.

**And the band is measured on the tile grid.** `Caravan::trade_value`
hands `Caravan::distance@0073d300` four `div_3_table[coord >> 8]` values —
`floor(floor(x / 256) / 3)`, one coordinate at a time, the same double
floor `produce_building`'s builder distance takes (`docs/AI.md` §2.20) —
and `distance` compares `vector_dist` of their differences against
`WorldData::xs`, which is that same grid's width. Measuring the
**world-unit** distance against a tile width puts every pair of cities on
every map in band 3 and doubles every route's value. Both errors were
live here until 2026-09-06; together they read `trade_val` 240 on Great
Lakes' AI where the original reads 128, and §8 says what that cost.

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

~~SEAM: the original's arrival facing is `find_angle(1, 0)`, the literal
pair `do_trade` passes where `add_move_order@00616ed0` reads a direction.
Nothing reads it until the unit arrives.~~ **The facing is the bearing to
the near city** (§11.3, item 1115).

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

### 5.3 What the answer becomes, and the stack it leaves behind

The search's answer is not the route. `build_road@0073db10:82` empties the
stack `find_road` filled and rebuilds it, and what it builds is what the
**legs** walk — so the loop is a mechanic and not bookkeeping.

`astar_caravan_road`'s own reconstruction (`+0x986`) walks from the arrived
node's parent up to but not including the root, **skips any tile carrying
`mask & 0x4000`**, and pushes `PathData { to, tolerance: 0x60, flags: 0 }`.
The goal end is therefore `list[0]` and the start end the top.

Then `build_road` pops that stack down to nothing, top first — the *start*
end first — into a scratch stack, and pops the scratch back on, which
restores the orientation. Per node:

- **open water** (`mask & 0x30 == 0x20`) lays nothing and is *sampled*.
  The first of a run is kept with `tolerance = 0x180` and arms a countdown
  of four; the next four are dropped outright. The countdown is bypassed —
  every node kept — while fewer than five have been written or fewer than
  five are left, so both ends of a crossing are dense and its middle is
  every fifth tile.
- **anything else** calls `World::set_road_at` on the tile and takes
  `flags |= 0x20`.
- either way, the first node written and the last take `flags |= 1`.

`Caravans::reset_paths` follows, and `reset_road`/`making_road` are cleared.

**The stack is the diff.** A `DUMP_ALL` block prints the whole of it under
`CARAVAN`/`STACK<TYPE>`, and run64's `FRAME 6171` block — the end of the
sim-frame `build_road` answered 1 on — carries all twenty-six of East
Indies' route: `(35232, 36384)` to `(38496, 39648)`, `tolerance 96`
throughout, `flags 33` at the two ends and `32` between. None of it is
water, so the sampling arm above is still a reading.

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

## 7. The legs — `do_trade`'s tail

Everything above the road is reached on the frame the route is established
and then never again while it walks: §4.1's move and §7.1's both go in with
`QUEUE_FIRST`, so the *move* is the current order and `do_trade` does not
run under it. **`do_trade`'s later frames are its arrivals**, and that is
what makes the head's `set_anim(CHAR_DEFAULT, 0, 1)` — the very first
instruction of the function, ahead of the caravan-slot test and every
return — a mark of arrival rather than of the order: a walking figure
leaves `Guy::set_anim` without a roll, a standing one rolls. East Indies'
word at **6198** is those three draws, one through `Unit::set_anim+0x56`
and two through `+0xb6`, and the same site fires on every arrival
afterwards — 6511, 6766, 7021 — one draw or three depending on what the
figures were playing.

The tail proper is two halves.

**The arrival test** is made against exactly one of the two cities: the
**far** one while the caravan is empty, the **home** one while it is
carrying. `local_38` is that city's own footprint, `max(x_size, y_size) ·
0x60` off `ObjectTypeData`, and the box is `± (local_38 + 0x306)` on each
axis independently — not a radius. Inside it:

- empty at the far city → `loaded = 1`, and `City::new_caravan` on the far
  city naming the home one;
- carrying at the home city → `loaded = 0`, `unit_masks |= 0x200`,
  `caravan_flags |= 4`, `City::compute_trade` on **both** cities, and
  `City::new_caravan` on the home city naming the far one.

A **decoy** (`unit_masks & 1`) turns `loaded` over and does none of the
rest. `unit_masks & 0x200` has no reader anywhere in the export — this is
its only mention — so it is not modelled.

### 7.1 `LAB_005ee01a` — walking the route

With a road planned the caravan does not path at all: it walks the route's
own stack.

```
if road.length:
    if dist(me, road.list[0]) < dist(me, road.top): road.invert()
    path = road                                   ← the whole stack, copied
    for i in 1 .. path.length:                    ← the smoothing, below
        dx, dy = path[i] − path[i−1]              (both read before either is written)
        if |dx| <= 0xc0 and |dy| <= 0xc0:
            path[i].to_x += dy / 3
            path[i].to_y -= dx / 3
    path.pop()                                    ← the node under my feet
    add_move_order(path.list[0], 1, 1, QUEUE_FIRST, 0, …)
    if tregion(path.top) != tregion(me):
        path.push(path.pop() with flags | 4)
    road.invert()
```

Three things in that are easy to get wrong and each is settled off the
listing rather than the decompiler.

**The invert is an orientation, not a reversal of intent.** The two
`vector_dist` calls at `5ee08e` and `5ee0cc` measure the unit against
`list[0]` and against `list[length−1]`; the invert runs when the *bottom*
is nearer, so the invariant afterwards is **the top of the stack is the end
I am standing at**. The pop then throws that end away and the move order's
destination is `list[0]`, the far end. The trailing `road.invert()` leaves
the route pointing the other way for the leg after this one.

**The move is `pathed`.** `Unit::add_move_order`'s fourth argument becomes
`add_move_facing_order`'s fifth, which is the order flag `1` — "the top
segment of the unit's path stack is this move's" — so `do_move` walks what
is already on the stack instead of planning. §4.1's arm passes `0` there
and this one passes `1`. ~~The same two arguments are also what
`add_move_order` hands `find_angle`, so the arrival facing of each is a
literal: `find_angle(1, 0)` for §4.1 and `find_angle(1, 1)` here.~~ The
arrival facing of each is the bearing from the caravan to the move's
target (§11.3, item 1115).

**The smoothing is perpendicular.** `to_x += dy/3` and `to_y −= dx/3` is
the step vector turned a quarter turn, so the route is walked *beside*
itself rather than along it — a third of a tile off the road, on one side.
The decompiler prints the second half as an unfolded multiply; the listing
at `5ee18f` is the magic `0x55555555` with a `sub`/`sar`, which is `x / −3`
and not `x / 3`. Both deltas are measured from the previous node **as the
search left it**: `5ee14e` and `5ee151` save `to_x` and `to_y` before
either is written, and the loop tail reads those, so the offsets do not
compound down the chain.

With no road — `build_road` answered 0 — the arm instead asks
`UnitType::find_nearby_spot` for a point between `local_38` and `local_38 +
0xc0` of the *other* city, swept from ~~`find_angle(1, 0)`~~ a
`find_angle` whose register arguments the decompiler does not name (§11.3;
not re-read) under
`FILTER_NOT_ME`, and moves there only if it came back inside `local_38 +
0xc6`. `local_38` is deliberately the footprint of the city the arrival
test used, not of the one being walked to.

### 7.2 `City::compute_trade@00739640` — what a route is worth

```
trade_val = 0
for link in city.vans:
    van = caravans[link.who][link.cara]
    if van.caravan_flags & 4 and both cities alive:
        mine = the end whose owner is my owner    (ecx/edx, §4)
        trade_val += trade_value(mine, the other) · 16 / 2
if trade_val changed: leader_flags |= 0x2000000     ← the economy is dirty
```

`trade_val` is `CityData +0x52` and it is the **first** line of
`LeaderData::calc_city_resources`, added to wealth ahead of everything a
city gathers — so a trade route's income is a city rate in sixteenths, not
a lump. The `& 4` gate is why it is worth nothing until the caravan has
completed a **round trip**: the bit is written only where a carrying
caravan reaches the home city.

### 7.3 `City::new_caravan@00739750` — the one-off

`traded_with[who]` (`CityData +0x2c`, `int[8]`) is a bit per partner city.
The first time a leader's caravan reaches this city from a given partner
the bit is set and the leader is paid, into **wealth**, `(epoch[2] + 1) ·
10` — times **twenty** instead when the leader is not this city's owner.
~~`epoch[1]` is the Civic library level~~ — **`epoch[2]`, the Commerce
level**: `0073979f` loads `LeaderDataEncrypt +0xf0`, and `epoch` opens at
`+0xe8` (§9, item 573). Nothing is paid the second time.

## 8. Coverage

**The Spice multiplier is diff-backed** (item 1189,
`run418_s_word_frame_is_widened_whole`). On run418's block 14661 both of
Great Sahara's who=1 cities' `trade_val` go 176 → 208 on a caravan's
return home. They agree here now, and who=1's wealth rows agree from
14664. **The Indian arm and the leader choice for a foreign pair are a
reading only.** No capture's route has a foreign end, and none has a
leader with the caravans' power.

**Diff-backed** (`rondata::diff::tests::run64_s_caravan_road_is_the_
original_s_node_for_node`, and its Great Lakes sibling on run73):

- **The whole search, node for node**: every one of the 3,204 + 3,204 +
  3,204 + 3,202 prices of run64's frames 6166 to 6169 — the tile, the
  direction it was reached from and the price — against the original's
  own proxied `calc_road_cost` answers, and the 151 of 6170 by tile and
  direction — **and 6170's prices too since 2026-09-02**, when the two
  draws the stream was out of phase by turned out to be the caravan's own
  crew figures wrapping an empty animation packet (`docs/ANIM.md` §3.6).
- **The five frames' node counts**, which is the budget rule and the
  never-restored `traversed` together.
- **The endpoints**, from the `astar_caravan_road` bracket: `oA 2000`,
  `whoA 1`, `oB 2007`, `caravan 0`.
- **The world the search reads**: all 57,600 tile masks' heights and all
  3,600 cell owners of run64's frame-6166 block. The heights are the
  sharp half — `Wall::init`'s farm gate (`docs/ROADS.md` §7.4) was found
  by 182 of them.
- **The route's own stack, all twenty-six nodes** — position, tolerance
  and flag byte, against run64's `FRAME 6171` `CARAVAN`/`STACK<TYPE>`
  block, which is the only block in the capture that carries a laid road
  (§5.3). It is what pins `build_road`'s loop rather than the search, and
  it found that this crate had been keeping the road as a list of *tiles*:
  `set_road_at` was handed world coordinates thirty thousand tiles off the
  map and **every trade road in the port went unlaid**.
- **A second map's search, node for node**
  (`run73_s_caravan_road_is_the_original_s_node_for_node`): Great Lakes'
  own trade route, `[5566, 5573]`, eight frames and 22,145 prices — seven
  budgets and the arrival. It is the same instrument one map over, and it
  found a road mechanic rather than a search one (`docs/ROADS.md` §9.5).
  A **five**-frame search on one map and an **eight**-frame one on the
  other is what §5.2's "every resumed frame gets a fresh budget" now rests
  on.
- The order and the schedule, indirectly: East Indies' word, which is a
  per-frame draw *sequence*, now passes 6166 to 6352 whole — through
  §7's arrival at 6198, the leg's first turn and a hundred and forty
  frames of the walk that follows.
- **The whole leg, unit for unit** (`run65_s_window_is_the_original_s_
  unit_for_unit`): every dumped unit of run65's eighteen-frame window —
  position, `UnitData::angle`, `orders_x/y`, `tolerance` and every slot
  of every path stack, 5,186 fields — over the frames the caravan
  arrives, is given its road, turns out of its city and walks. It is
  what pins §7.1's leg against the original rather than against the
  reading.
- **The three figures' clocks, frame by frame**
  (`run64_s_window_clocks_are_the_original_s`): every `GUY` block of
  run64's eight-frame `DUMP_ALL` window, 2,061 fields, of which the only
  ones that part are the caravan's own walk below. `CREW_SIZE 2` means
  three figures, the two crew ones are carried (no `trackoffset` on
  `CARAVAN-DEFAULT-AGE0-CREW1`/`-CREW2`), their packets are empty, and
  they pay two idle rolls every third frame from the moment the unit
  starts moving.

- **`trade_val` itself, over 610 frames and three archives** (2026-09-06,
  `rondata::diff::tests::great_lakes_goods_record_and_its_trade_routes_
  are_the_original_s`): run76's, run83's and run79's `CITY` records over
  Great Lakes' 6640–7249, 1,785 city-frames. The AI's two cities carry
  **128** each on every one of them and the human's carries nothing.
  Before 2026-09-06 this crate read 240 — `members.len()` for the count
  and world units for the band, each worth a factor — and the two of them
  put fourteen wealth a minute into the AI's purse that the original does
  not have. The consequence was an **AI decision**: at Great Lakes 6982
  the surplus was what let `make_stuff` afford a University the original
  declined (`docs/AI.md` §30), and the long capture's word had stood at
  6982 on it. The check that found it is the goods record beside this one
  — the same test compares the leader's `bucket`, `leftover`, `resources`,
  `income`, `resource_cap` and `gather_slots`, both players, six goods,
  over run18b's 6374–6589 and run84's 6950–7029, and all 21,312
  good-frames are the original's.

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
- §5.3's **water sampling**: run64's road has no ocean node, so the
  countdown of four, the `0x180` tolerance and the two end exemptions rest
  on the loop alone. A route between two coasts of one region would show
  them; East Indies' caravan has `can_transport` and its search prices
  ocean, so a capture of a longer route on the same map would do it.
- ~~`new_caravan`'s `(epoch[1] + 1) · 10`~~ — **diff-backed since item
  573, and the index was wrong** (§9).
- §7's **arithmetic past the word**. The arrival box, the `loaded`
  turnover and `compute_trade`'s `× 16 / 2` are all reachable in run54 — the caravan loads at
  the far city around frame 6511 and unloads at home around 6766 — but the
  word parts at 6207, so no assertion reaches them. The `LEADERS=9` census
  over `[6760, 6790)` is the capture: it prints `bucket` and `income` per
  good per leader, and both halves of §7.2 and §7.3 land in wealth on the
  frame the caravan comes home.
- `Caravan::restart_trade_route@0073d070`, `Caravans::new_danger@0073e0c0`
  and `Caravan::verify_road@0073d950` — no traced game enters any of them.
- §7.1's **no-road arm**, and `UnitType::find_nearby_spot`'s ring inside
  it: a route whose `build_road` answers 0 has no caravan on it in any
  capture, and this crate walks to the city's own point instead.

**Not established at all:** what a fallen city does to a route mid-trip —
`Caravan::verify_road` and `restart_trade_route` are the functions and
nothing enters them.

~~And the arrival's own **one-frame residue**~~ — closed 2026-09-02 by
run65, and **it was not the caravan's at all**. The window said so in one
reading: both sides push the same detour node `(38508, 40620)` on the same
frame, turn through the same eight bearings at the same rate, and on
sim-frame 6206 compute the same step to the same point. The original
simply does not take it — `move_step` asks `UnitData::invalid_loc` about
any step that changes tile, and that point is a tile inside the caravan's
own city's footprint. The answer is `docs/MOVEMENT.md`'s, under "The unit
step", and it is the last of the four things that section listed as read
and not modelled; the whole eighteen-frame window is now
`run65_s_window_is_the_original_s_unit_for_unit`, 5,186 fields including
every slot of the twenty-six-node stack. East Indies' word: **6207 →
6353**.

~~**And it does not walk there the original's way**~~ — closed 2026-09-02,
and the walk was never the bug. §4.1's move goes in with `QUEUE_FIRST` and
this crate's carried a **waypoint** at `(37752, 41592)`, so from run64's
frame 6167 `1/18` set off south-west while the original heads straight at
`(39288, 40056)` — but the waypoint was `find_wpath`'s answer, and that
answer was wrong because **`calc_cost` was missing the danger map**.
`WorldData::danger[who]` is 65 to 135 negative around a leader's own city,
the world grid prices a step by `danger / 8`, and eight of the caravan's own
steps were 8 to 16 too dear: the search took a seventh expansion the
original does not and came back with a route where the original's stops at
the goal's own neighbour and pushes nothing. All 2,061 fields of run64's
window are the original's now (`docs/DANGER.md` §6, `crate::danger`).

~~What is left of item 177 is its second half~~ — closed 2026-09-02. The
arrival, the legs, `loaded`, the income and the one-off are §7; the word
went **6198 → 6207**, and the two things that carried it were a `set_anim`
at the head of a function and a `.tile()` that was never there.

## 9. The one-off's level is Commerce, and it cost East Indies a Scholar (item 573, 2026-09-23)

**How it was found.** East Indies' long word stood at 9711 for forty
landings. Its widening on run99 (`rondata::diff::tests::run99_s_word_
frame_is_widened_whole`) put one row on the word's block: the original's
sixth seated Scholar, `1/28`, born at University `1/2015`, and none here.
Walked backwards, the only row between it and the window's floor was the
purchase, block 9577: the original queues the Scholar on sim-frame 9576
and this crate queues nothing. The buyer is `economic.bhs`'s opening
script, `if (at_least_type(who, 130, "Wealth")) train_unit_with_cost(who,
1, "Scholar")`, and this crate's player 1 held **114** wealth there.

run98 and run99 print `LEADERS=1`'s stub, so the stockpile on 9576 is on
no disk. run82 is the same game at `LEADERS=9` over [6860, 6930), and its
first block read player 1's `bucket[2]` **226 against 246**, every other
good and `resources`, `income`, `rate` agreeing. run59's [5150, 5400]
agrees to the unit, so the twenty was lost in between. This crate's
wealth moves in lumps on nine frames of that span, and two of them are
the caravan's first arrivals at each end of the London–Norwich route,
+20 on 6512 and +20 on 6767: this section's one-off at a level of 1.

**What it is.** The type record puts `LeaderDataEncrypt::epoch` at
`+0xe8`, so `+0xf0` is `epoch[2]`, and the listing is
`movl 0xf0(%ecx), %eax; xorl $0x63187, %eax; incl %eax` at
`0073979f`–`007397aa`. This crate read `epoch[Line::Civic]`, index 1.
Player 1's Commerce level stood one above its Civic, so each arrival paid
30 in the original against 20 here, and the difference is the whole
twenty.

**What it moved.** With Commerce read, run82's stockpile agrees on all 70
blocks (`run82_s_leader_record_is_the_original_s_wealth`, 420 rows; made
to fail on purpose by reading Civic again). The Scholar is bought on
9576 and born on 9712, and East Indies' word moves **9711 → 9983**. Great
Lakes (12038) and the golden chapters (900, 900, 900, 1277) hold.

**What is not established.**

- The wealth on 9576 itself. The nearest dumped value is run82's 6860,
  2,700 frames earlier. That the twenty is the only difference between
  6860 and 9576 is inferred from the word, not read.
- Which of Civic and Commerce the other `epoch[]` readers in this crate
  mean. Only this one was checked against its listing.
- 9983 itself. It is a `make_stuff` on both sides, and they buy
  different things (the widening's rows on block 9984). The make list
  that decides it is on no disk: run99 prints the stub.

**Coverage.** Diff-backed: the level, by run82's wealth on 70 blocks and
by the word. Reading-backed and pinned by
`sim::caravan::tests::the_arrival_bonus_is_paid_once_and_doubles_for_a_foreigner`,
which now sets Commerce and Civic to different levels: the once-per-bit
rule and the foreign doubling.

## 10. A road-flagged waypoint asks whether the road is still there (2026-09-24, item 695)

*Established from `Unit::do_move@005f7b30`, `Caravan::verify_road@0073d950`
and `WorldData::is_built_at@0046f880`, and diff-backed by Great Lakes
14529 (run178, run189).*

~~`path_flag::ROAD` is written by `build_road` and nothing reads it
back.~~ `Unit::do_move` reads it. The flag's doc comment used to say
otherwise.

### 10.1 The check — `do_move@005f7b30:437`–`478`

When the move takes its next waypoint (`dest == 0`, the top of the path
stack) and the unit's **action** is `TRADE_ROUTE` (type `0xf`, from
`get_action`'s vslot `+0x10`), it asks whether the waypoint carries flag
`0x20`:

```
tile = div3(to >> 6)
if surface(tile) != ocean:  TerrainOut::caravan_step(tile, who)   ← camel steps (ROADS §10.5)
if surface(tile) != road and (is_built_at(tile) or mask & 0x4000):
    caravans[who][this.caravan].verify_road()
    every entry of this unit's path: flags &= ~0x20
```

`is_built_at` is `(mask & 3) == 3 || mask & 0x80`. That covers a finished
footprint, and a placed-but-unstarted one as well. The tile is the
**waypoint's**, which is the offset position §7.1 wrote. The stripped
flag means the walk asks once per leg.

### 10.2 `Caravan::verify_road@0073d950`

It walks the route's own road stack (`+0x10`, `+0x18` its length). The
first tile that is built on, by the same low-byte test, calls
`build_road`. The search is fresh: nothing is parked, so the three-way
gate (§5) throws nothing away and starts again. A budget stop parks the
search, and `Unit::work`'s block (§6) resumes it each frame. A route with
every tile clear is left alone.

### 10.3 What it moved

On Great Lakes, `1/23` takes (42272, 19424) on tick 14529. That is tile
(220, 101), under the Barracks `1/2025` (placed on 14382, not started),
and the stray-road sweep had taken its road (`docs/ROADS.md` §10). So the
route is verified: 3,206 road draws that frame, 3,206 the next, and on
until 14533. On block 14530 all 24 of the walk's waypoints have lost
`0x20`. This crate had neither half. Its tile was still road, and nothing
asked. With both, Great Lakes' word moves **14529 → 14650**
(`docs/AI.md` §68).

### 10.4 What is not established

- `TerrainOut::caravan_step` is not modelled. Its sound and ruts are the
  renderer's, but its reference counts are the mesh's (`docs/ROADS.md`
  §10.5).
- `Caravans::new_danger@0073e0c0` restarts a route whose road passes
  within `0x600` of ~~a hit on a caravan~~ ~~a hit on a land caravan that
  `Object::take_damage` lets through, and it lets through two arms only~~
  **the killing blow on a land caravan on a linked route** (item 965).
  ~~Neither hit a capture made on London's ground entered it: run341's
  hoplites, and run342's Tower, whose arrows took the caravan's damage up
  by 12 a shot from frame 913. So the second arm's attacker is **not**
  simply the object that fired, and what `Object::do_damage` passes there
  for an arrow is not established; the first arm, a hit on unowned or
  enemy ground, is untried.~~ **Established by the listing, the emulator
  and run353:**
  - **The gate is the kill.** The block at `00652df1..00652f41` has one
    entry, `jge 0x652ab5` at `006529eb`, `damage >= share`; a hit the
    caravan survives returns first (`docs/COMBAT.md` §7.2 step 9). On the
    death path, in order: `is_unit` (`+0x18`), `is_captain` (`+0xe8`,
    `o_up < 0`), `is_caravan` (`+0xd0`), the type's domain (`+0x218`) 0;
    then either arm; then the unit's slot (`+0x86`) set and
    `caravan_flags & 3 == 3`. The slots are the type records'
    (`SubObjectData` and `ObjectData`), so the second arm's `+0x1c` is
    `is_wallbuild`: 1 on Build and Wall, 0 on Unit.
  - **Arguments 7 and 8 name the blow's dealer**: `Object::do_damage@0064a480`
    pushes its own `this`'s `+0xa` and `+0x9` (`0064ba18`); `Unit::fight`
    calls it on the attacking unit, and `Ammo::do_damage@00678060` on
    `objects[who][o]`, the ammo's shooter (`AmmoData` `+0x3c`, `+0x40`), so
    a Tower's arrow names the Tower. The same `this` writes the target's
    `damage_o`/`damage_who` (`0064a62e`), which is why run342's caravan
    reads `2009`/`0` after the Tower's hit. `Unit::suffer_attrition` and
    `SpellType::cast_sabotage` pass −1, −1, so only the first arm; a
    decoy's overflow names the captain itself.
  - **run342's kill was a Unit's.** The caravan `1/6` stood at 82 of 90
    on block 1209. The Tower's recharge ran down across 1195–1210 without
    firing, hoplite `0/7`'s went 1 → 32 on block 1210, and `DEATH_OBJS`
    prints `1/6` with `first_frame 1209`. A Unit's kill on the owner's
    ground fails both arms, so 959's reading of the arms stands; what it
    missed is that the gate needs a kill.
  - **Under the emulator** (a scratch probe on `tools/recomp/step4.py`'s
    machine, Great Lakes' packet at logger frame 11186, caravan `1/23`
    with `caravan_flags` 7 on its owner's tile, player 0's Build `0/2000`
    and Unit `0/0`): a hit that does not kill returns 0 before the gate,
    whether a Build's or attrition's shape. A Unit's kill on the owner's
    ground stops at `is_wallbuild`. **A Build's kill enters `new_danger`
    and `Caravan::restart_trade_route`.** Attrition's shape (−1, −1)
    killing on the owner's ground stops at `o < 0`, and enters both on
    a tile patched unowned. A Unit's kill on a tile patched to player
    0's enters both.
  - **run353** (`docs/RUNS.md`): three player-0 Towers beside the road
    and no Unit attacker. Their arrows take the caravan to 82 of 90 by
    981, and it dies on 1007 on the road at (220, 100). `new_danger`,
    `restart_trade_route` and `close_caravan` are first entered on 1007.

  **Not established**: which arm admitted run353's kill. The dump prints
  no tile owner, and the tile lies on the road between two of player 1's
  cities, 16 tiles from the capital and 13 from the Small City. So the
  first arm has the emulator only, on a patched tile. `new_danger`'s body
  past its call of `restart_trade_route` was not run: the probe stopped
  there. The port has neither: `Unit::close`'s caravan arm only gives the
  slot back. It is not modelled.

## 11. A route across water: the region tests, the shore surcharge and the bearing (2026-09-28, item 1115)

*Established from `Unit::do_trade@005ed270`, `PathFinder::calc_road_cost@00686300`
and `Unit::add_move_order@00616ed0`, read off the listing where the
decompiler printed register arguments as stack ones, and backed by
**run413**, a packet at logger frame 5776 on East Indies at Toughest, with
the second pair's word (`docs/AI.md` §85).*

East Indies' AI founds its third city, Newcastle `1/2017`, on a second
island (region 5) on 5518. Its first caravan `1/33`, trained on 5772,
stands at London (region 11), whose one route (to Norwich) is taken. The
only free partner is across the sea. This crate had never had a route
cross water, and three pieces of it were wrong.

### 11.1 The region tests — `do_trade@005ed270`

`do_trade` asks `WorldData::get_tregion` (with its coastal refinement,
this crate's `tregion_alt`) three times, and each time a difference is
forgiven only by `UnitData::can_transport` on the caravan:

| call returns at | compares | on failure |
|---|---|---|
| `005ed483` | the caravan's tile against its home city's | `LAB_005eda11`: the order dies |
| `005ed631` | each candidate's tile against the **home city's** | the candidate is skipped |
| `005ed920` | the caravan off the home's region, or the far city off it | `LAB_005eda11` |

The first and third run on every call, arrivals included. ~~SEAM: the
cross-region arm~~: this crate refused every candidate in another region,
whatever the caravan could do. On East Indies that left `1/33` with no
partner, and the failure arm (`idle = 99`, the order killed) is what it
took where the original started the road. `1/33` has `unit_masks
0x840000`, `can_transport`'s `0x800000`.

**Not modelled**: the failure arm's AI tail. With `unit_masks & 0x40000`
it `go_to_city`s the home owner's next city when one is alive. No capture
reaches it.

### 11.2 The shore surcharge — `calc_road_cost@00686300`

On an ocean tile (`mask & 0x30 == 0x20`) the cost function has two arms on
`pathfinder +0x8c`, "both road ends in one region":

- **set**: `+0xa4` on every sea tile, which is run64's 155 + jitter;
- **clear**: `+0xa8` only when the node's **parent** tile is not ocean.

So a crossing between two regions pays the surcharge once, at the shore,
and every sea tile after it costs the base (55) and the jitter. This crate
added `+0xa8` on every sea tile. The difference is invisible to the draw
stream until the search's order parts: ours kept the original's per-frame
counts for three frames (3,203, 3,206 and 3,205) and parted on the fourth,
5776, with 3,201 against 3,206.

**run413** settles it: the original's parked search is in memory between
frames (`CaravanData +0x28` the open `Tree<PathNode*,int>`, `+0x30` the
closed `BRTree<PathNode*,ulong>`, `caravans` at VA `0xE3A290`). At logger
frame 5776, after three search frames, it holds 158 open and 1,279 closed
nodes. With the surcharge on every sea tile, ours held 1,436 and **171
differed** in length or parent, the lowest-valued ones sea tiles south of
London where a shared parent's step cost 57 there and 155 here. With the
parent test, **1,437 against 1,437, none differing**.

### 11.3 The move faces its bearing — `add_move_order@00616ed0`

The decompiler prints `find_angle(param_3, param_4)`, the order kind and
the pathed flag, and §4.1 and §7.1 read those as a literal facing.
`find_angle@0092d130` takes its two arguments in `ecx` and `edx`, and
`add_move_order` loads them at `616edc`..`616f1f` with the destination less
the unit's own decoded position: **the bearing to the target**. The
pushes the decompiler saw are `add_move_facing_order`'s. This crate's
generic `add_move_order` already took the bearing from the same listing;
only the caravan's calls carried the literal. run357's block 5774 reads
546111488 on `1/33`'s move, the bearing to London's point, where this
crate wrote 1073741824 (due east).

**The trade order's bit 4.** `think_caravan` passes 0 as
`add_trade_order@005e4dc0`'s last argument, which clears the order's
`0x4`. This crate set it. `get_action` skips only a transit move without
the bit, and a trade order is never a transit move, so nothing read it;
run357's `order:flags` and `order:action` rows on `1/33` were all it
changed.

### 11.4 What is not established

- **The failure arm's `go_to_city`** (§11.1).
- **`find_nearby_spot`'s sweep angle** in §7.1's no-road arm, which the
  decompiler prints as registers it does not name. Not modelled.
- **`add_trade_order`'s transport tail**: a caravan in another region than
  its home city gets `unit_masks |= 0x800000` when the leader's transport
  setting allows it and `can_ever_transport`. Not modelled; `1/33` shares
  London's region.

### 11.5 Coverage

- **Packet-backed**: the search, node for node on the packet (run413),
  after three frames.
- **Diff-backed**: run346's word moves 5773 → 5776 on §11.1 and 5776 →
  5975 on §11.2; run357's `1/33` rows (`orders.len`, `order:flags`,
  `order:action`, `order:move.angle`, `dest_angle`) close on §11.1 and
  §11.3.
- **Pinned capture-free**:
  `cities_tests::a_caravan_trades_across_regions_only_when_it_can_transport`
  and `roads::tests::a_sea_crossing_between_regions_pays_once_at_the_shore`.
- **Listing-backed**: the facing (`616edc`..`616f1f`) and the three region
  calls' return addresses.

