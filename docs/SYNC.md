# The sync stream, frame by frame

*Established 2026-08-24 from the decompile (`tools/ghidra/`, the export under
`decomp/`) and two oracles: **run12**, the `DUMP_ALL` capture whose
`begin_frame`/`end_frame` dumps print `game_random seed`
(`docs/ORACLE.md`, "What the trace does not cover"), and **run13**, the
same game's `DUMP_ALL` window over sim-frames 95–103 (§4.1). The method is the one
that settled the setup path: walk the 32-bit LCG forward from a known word
and match each consumer's visible outcome to the draw that produced it. Every
claim below says which of the two it rests on.*

`GameAccess::game_random` is the one stream the lockstep simulation is
synchronised on (`docs/COMBAT.md` §9.5: `seed = seed · 0x19660D + 0x3C6EF35F`,
`Random::get(0, 0xffff) = ((seed & 0xffff) · 0xffff) >> 16`). `docs/ORDERS.md`
§9.2 counts every draw from the lobby seed to frame 0; this document counts
the ones **inside a frame**, which is what the AI's script needs — its
`rand_int`s at frame 1 land where frame 0's draws leave the stream
(`docs/AI.md` §12.1 item 2′).

## 1. What run12 says

Four frames, four counts, and the dumps themselves draw nothing (the records
at `end_frame(n)` and `begin_frame(n+1)` carry the same word):

| frame | draws | word at the end |
|---|---|---|
| 0 | **120** | `0xb6194ba1` |
| 1 | **54** | `0x4554ec0f` |
| 2 | 6 | `0xab3b035d` |
| 3 | 6 | `0xc24206bb` |

The log numbers its frames from 1 (`BEGIN FRAME 1` is the first frame, the
engine's `game->frame == 0`); this document and the harness use the engine's.
Both records of a frame are printed by `GameLog::full_dump`, whose
`say_checksum` runs **before** `dump_all`, so a frame's count is exactly the
draws between the frame's first instruction and its `end_frame`.

## 2. The frame — `Game::do_frame@00591ef0`

In order, keeping only what can reach the stream:

1. `GameLog::begin_frame` (the dump; 0 draws).
2. **`Leaders::process_all`** — income, `calc_unit_stats`, `calc_wall_stats`,
   `gather`, elimination, taunts. **0 draws** (proved at frame 0: the sweep's
   two draws are the frame's first two, §4).
3. **`Leaders::strategy_all@006ed430`**, for every leader with
   `flags & 3 == 3`: `check_explore` (0), **`plan_strategy`** (the census and,
   on a production step, the script or the C++ producers — `docs/AI.md` §11
   is the inventory; the sweep itself draws 2 in `compute_sites` for a
   ≥200-cell region, §2.7 there), `compute_score` (0), **`diplomacy`** (nine
   sites in `Leader::diplomacy@006bc950`; 0 draws on run12's four frames —
   its cadence is not read).
4. **`GameDaemon::process_all` → `calc_markets@00732180`** — §3.1.
5. **`Armies::process_all`** — `Army::find_target@006f69b0` draws at three
   sites (`% 200 + 900`, twice, and a coin); the AI's army exists from frame
   0 (`ARMY who 1, status 1`) and drew nothing on frames 0–3.
6. **`Objects::process_all@0065dce0`** — §3.2: every object's virtual
   `process`, then the birds and the herds.
7. **`Objects::inc_time@0065db70`** — per-object `inc_time` and
   `execute_events`, **in leader order 0–9 with no rotation** (units, then
   that leader's buildings), then the goods, `Ammo::inc_time`,
   `DeathObj::inc_time`, then **`Farms::inc_time@008d8600`** (§3.3),
   `Doober::inc_time` (0), `Surf::inc_time` (draws — from
   **`internal_random`**, not the stream). ~~(0 draws)~~ **This is the
   animation clock, and it draws** (run13, 2026-08-24): `Unit::inc_time@
   00610b40` → `Guy::inc_time@005d9e10` adds the step (1; 2 in an
   `ATTACK2`; 0 while `Unit+0x6c & 0x10`) to every guy's `cur_time`, and
   when it reaches `end_time` restarts the animation — a looping one with
   `set_anim(same, 0, 1)`, anything else with **`set_anim(CHAR_DEFAULT, 0,
   1)`**, which is the idle-anim draw of §3.2. So a standing unit or animal
   costs **one draw each time its idle animation runs out**, at the
   frame where `cur_time == end_time − 1` was read at the previous
   `end_frame`, *after* every object's `process` and *before* the farms.
   Run13's sim-frame 100 is exactly this: twelve fish schools with
   `end_time 101` redraw at draws 0–11, the six farms follow (§4).
8. `GraphicEvents::process`, `Leaders::end_process_all`,
   `Achieve::capture_data`, `Leader::process_event_frame`,
   `OrdersMemManager::cycle`, `Roads::scan_and_kill_stray_roads`: none of
   them, nor anything they call by name, touches `game_random`.
9. `frame += 1`; `GameLog::end_frame` (the dump).

The whole set of functions that read `GameAccess::game_random` is 129 in the
export. Setup and map-making take 60 of them (`docs/ORDERS.md` §9.2); the
per-frame ones are listed by phase below; the rest are event-driven — combat
(`docs/COMBAT.md`), `Object::disband`, `Unit::close`, `come_out`, `plunder`,
`explore_goody`, `SpellType::cast_pilfer`, `Dock::init`, `Farms::add` and
`add_animals` (a farm's creation), `Build::find_gather_tiles` (a camp's
creation), `World::compute_val` (a site value), `PathFinder::calc_road_cost`
(a cost jitter `% 20` per node on the **road** grid — a caravan's), and the
`astar_path` refusals that roll `pause = % 3 + 6`.

## 3. The per-frame sites

### 3.1 The market — `GameDaemon::calc_markets@00732180`

Runs when `frame == 0`, or `market_cycle_rate < 2` (it ships as **1**, so
every frame), or `frame % market_cycle_rate == 0`. For each of the six goods
`i` (the `market[6]`, `market_flux[6]`, `next_flux[6]`, `delta_flux[6]`,
`flux_length[6]` arrays on `Game`), with `t = market_tick`:

```
if t == 0 or (t + i) & 7 == 0:
    if (t + i) & 0xff == 0:                    # the price step, one good a tick
        if market[i] > EQUILIBRIUM:  market[i] -= market[i] / EQUILIBRIUM  (−1 if EQUILIBRIUM == 0)
        elif market[i] < EQUILIBRIUM: market[i] += 1; if market[i] < BASEMENT: market[i] += 1
    if t == 0 or --flux_length[i] < 1:  calc_market(i)
    market_flux[i] += delta_flux[i]
market_tick += 1
```

`calc_market@00732270(i)`: `v = max(market[i] / 2, MARKET_MIN_VARIANCE)`,
`v = (v + 1) / 2`; if `v ≥ 1`, **two draws** `a = rand % (v+1)`,
`b = rand % (v+1)` and `next_flux = b − (v+1) + a`; then, if
`MARKET_TREND_RANGE > 1`, **one draw** `flux_length = MARKET_MIN_TREND +
rand % MARKET_TREND_RANGE`; `delta_flux = (next_flux − market_flux +
(len − 1) · sign) / len`. The shipped constants: `MARKET_EQUILIBRIUM 65`,
`MARKET_BASEMENT 10`, `MARKET_MIN_VARIANCE 2`, `MARKET_MIN_TREND 8`,
`MARKET_TREND_RANGE 16`, `MARKET_CYCLE_RATE 1`; every good starts at
`market 50`, `market_flux 15`, `next_flux 15`, `flux_length 1`.

So **frame 0 costs 18 draws** — all six goods, three each — and afterwards
good `i` is visited once every eight ticks, drawing three when its
`flux_length` runs out. Run12's frame-0 outcomes (`next_flux = −13, −9, −11,
−3, −1, −9`; `flux_length = 15, 8, 21, 22, 11, 20`; good 0's price stepped
50 → 51 first) are reproduced by draws **2–19** of the frame and by no other
offset — the pin that places the market and proves the two phases before it
draw exactly the sweep's two. Landed: `crates/sim/src/market.rs`.

### 3.2 The objects — `Objects::process_all@0065dce0`

**The unit loop rotates.** `for i in 0..10: slot = (frame + i) % 10` — the
owner whose units go first is `frame % 10`, so at frame 0 player 0's units
run first and at frame 1 player 1's, with gaia's animals (who 8) and birds
(who 9) in their slots. Run12 shows it: at frame 1 the AI's woodcutter draws
its `wait` (draw 44) *before* the human's two (45, 46). Buildings follow in a
second, unrotated loop (`Build::process`, then walls). The sim's tick keeps
its buildings-first order (`lib.rs`), which is a known divergence this
document does not close.

Per unit, the sites that run on an ordinary frame:

| site | when | draws |
|---|---|---|
| `Guy::set_anim@005da300` (idle) | `set_anim(CHAR_DEFAULT)` from `do_idle`, `do_gather`'s stand, any order's rest, an arrival — **but only when a new default animation starts**: a `CHAR_DEFAULT` request while the current default anim still runs (`cur_time < end_time`) returns first (`set_anim:155–224`); **and again from `Guy::inc_time` in phase 7 each time the running animation ends** (§2 step 7 — run13: the fish at sim-frame 100, the human scout's `60/61 → 0/61` at 101) | 1 per guy when `UnitData+0x104 == 0`: `p = rand % 100` → variant **0** for `p ≤ 69`, **1** for `70–82`, **2** for `83–95`, **3** for `96–99` (a peasant standing on a tile with `mask & 3` takes 1 for `p ≥ 83`; `+0x9a & 0x20` collapses it to 0/1 at 69) |
| `Guy::set_anim` (`param_2 == 0xc`) | ~~the sow~~ **the attack category** (`UnitAnimCat[11..=14] = 12`, `docs/ANIM.md` §2): with `param_3 != 0`, one draw picks `ATTACK1`/`ATTACK2`/`ATTACK3` at 30/40/30 %. The sow (35) is its own category and never draws; `do_gather:407` passes `(CHAR_SOW, 0, 1)` | 1 per attack start |
| `Guy::set_anim` (walk, a bird) | who 9, types `0x192–0x194` | 1 |
| `Guy::init_real@005db6b0` | a guy's creation | 1 (`% 100` → a 4-way variant) |
| `Unit::do_move@005f7b30:599` | the first `do_move` of a move whose path is planned, after `find_path` | 1 (`% 5` → the `far` threshold; `orders.rs`) |
| `do_non_flat_gather@005f0170` | the woodcutter/miner machine | `% 200 + 400` on choosing a tile, `% 50 + 100` and `% 100 + 300` on the waits (`docs/ORDERS.md` §6.4; `orders.rs`) |
| `do_gather` at a farm | a **new tile** (`docs/ORDERS.md` §6.5) | 2 (`GameAccess::rnd(4)` twice, x then y — **4, measured on run13**: the six farmers' twelve draws on sim-frame 101 give the dump's new tiles under `% 4` and no other modulus, §4; the target is `(corner + r) · 0xc0 + 0x60`, then `add_move_order`'s 48-cell snap puts it at `corner·192 + 120 + 192r`) |
| `Unit::think_scout@005f6010` | a scout with no orders (`think`, the bottom branch: not supply, not hero, `is(0x3a)` or `type+0xb2 & 0x10`, `get_army < 0`) — the human's too | 1 for the scan start (`% (ceil(count/100) + frame&7)`) + **1 per candidate cell** that is unseen, `invalid_loc`-clean and of the right domain (`& 7`, a score jitter); the `param_1 != 0` entry draws `% count` for a region pick and re-draws while the cell has `& 0x70` |
| `Animal::do_idle@005d7460` | an idle animal, every frame: `Unit::set_anim(CHAR_DEFAULT, 0, 1)` first (a draw only on an arrival — run13: sheep 0 at sim-frame 101), then, for a type with `+0x218 == 0`, **a herd member** (`+0x86 ≥ 0`) rolls, a herdless one goes to `think_farm_animal` | when `guy.cur_time == guy.end_time − 1`, 1 (`% 10 < 3` → wander) and, near its herd's centre (`< 0x181`), 3 more (`& 7`, `& 3`, `& 3` → a step along `move_x/y`); else `find_nearby_spot` (0). **This lobby's forty animals are four `HERDSHEEP` (408, herd 0) and thirty-six `HERDFISH` (411) in twelve schools of three at one spot each**; the fish never reach the roll — run13's twelve `end_time 101` fish drew exactly their twelve wraps at sim-frame 100 and nothing else |
| `Animal::think_farm_animal@005d7700` | a **pasture's** animal (owner 9, five per farm — §3.6), every 128 frames phased by `o · (slot + 1)`, where `slot` is `Animal+0x154`, its place in the five — ~~`scale`~~ | 1 (`& 7`) when the farm covers the tile of the object it measures: the farm itself while nobody gathers there, `gather_down`'s first gatherer once somebody does |
| `Animal::think_bird@005d79e0` | who 9, every 8 frames | 2 (`% 0x51`/`% 0xf` offsets), then a landing roll `% n` and a 30-round `% count`, `% 50 + 1` search |
| `resolve_unit_collision@005f9d30:499` | a mutual collision | 1 (`pause = % 9 + 1`) |
| `do_group_move@005e79a0:376`, `do_guard:288`, `do_spec_anim` (2), `think_fish` (2), `think_spellcaster` (2), `do_air_physics` (2) | their situations | as named |

Then, still inside `process_all`:

- **Birds, every 32 frames** (`frame & 0x1f == 0`): `n = min(10, xs·ys/100)`
  minus the live who-9 units of type `0x192`; `n` times: **2 draws**
  (`% xs`, `% ys`), and if that cell's `flags & 0x20` (a **mountain** cell on
  this map — ocean cells carry 0; 344 of run9's 3,600 are `0x20`) a bird is
  created (`init_unit(who 9, BASE_GAIATYPES)`, an air-patrol order; its
  `Guy::init_real` draws once more). Run9's map: **20 draws at frame 0, no
  bird** — all ten cells miss.
- **One herd, every 64 frames** (`frame & 0x3f == 0`): herd
  `(frame >> 6) % max(count, 5)`, if it exists and is alive:
  `Herd::process@00741760` — **2 draws**, `wx' = wx − 1 + rand % 3`,
  `wy' = wy − 1 + rand % 3`, accepted when in bounds, the cell has no
  `flags & 0x70` feature and its `+0x11` byte `< 8`. Run12: herd 0's
  `wy 34 → 35` with `wx 22` kept — the pair `(1, 2) mod 3`.

Landed: the birds' sampling, the bird itself (§3.9) and the herd walk,
`crates/sim/src/gaia.rs`.

### 3.3 The farms — `Farms::inc_time@008d8600`

Every frame, every farm whose **`farm_type` is not 1** (`FarmStruct+0xbd`,
named by the type record — §3.6; that farm is a *pasture* and grows nothing)
and whose building is complete (`Build flags & 4`; a site is skipped). Each
of its 16 cells holds a state byte and a `float percent`:

```
for each cell:                       # column-major: col outer, row inner
    if state == 1: percent += 0.005f
    if state == 3: percent −= 0.01f
    if percent > 0: if percent ≥ 1.0f: state = 2, percent = 1.0f
    else:           state = 0, percent = 0, empty += 1
if empty ≥ 12: chance = 20
elif empty ≥ 5: chance = (empty − 4) · 20 / 8
else: next farm
ONE DRAW: if rand % 1000 < chance:
    ONE DRAW (if empty > 1): k = rand % empty
    the k-th empty cell in column-major order → state = 1   # a sprout
```

So **a complete farm costs one draw a frame** while it has five or more
empty cells, and two on the 2 % of frames it sprouts. Run12's lobby has
six farms: 6 draws on frames 2 and 3 (nothing else drew), 6 at frame 0
(no sprout), and **7 at frame 1** — farm 1 sprouted (`% 1000 = 19`,
`% 15 = 12` → its 13th empty cell, `(row 1, col 3)`), which is exactly the
cell the frame-2 dump shows starting to grow. The AI's seventh farm is a
site and draws nothing until it completes.

**This is the other half of the farm clock, and it closes `docs/ORDERS.md`
§6.5's open item.** The farmer's `Farms::grow@008d91c0` adds `0.005f` and
sets state 1; `inc_time` adds another `0.005f` the same frame; and `0.005f`
summed in single precision reaches `1.0f` on the **201st** add, not the
200th (the pinned crossing, `farms.rs`'s test). Two adds a frame checked
once a frame: `< 1.0f` after 200 adds at frame 99, `≥ 1.0f` after 202 at
frame **100** → state 2 → the farmer's `snip` (2 → 3) on 101 → the "new
tile" branch with its two draws on **102**. The regrowth is `1.0f − 0.01f`
per frame, `≤ 0` on the **101st** subtraction. Landed:
`crates/sim/src/farms.rs`, `FARM_GROWS` retired.

### 3.4 The docks — `Dock::init@00740a80` (2026-08-25)

Not per frame: per dock finished. `Build::activate` → `Docks::init_dock` →
`Dock::init` spawns the dock's gull (`Objects::init_unit(9, GULLBIRD, x −
0xc0, y − 0xc0)` — one creation draw, `Guy::init_real`) and then draws
`Random::get(game_random, 0, 0xffff)` for its heading (`% 7 × 0x0aaaaaaa −
0x40000000`). **Two draws, creation first**, inside the building's own
`process` in the objects phase (§3.2). Run21, frame 3579: draws 0 and 1
are `Guy::init_real+0x52 < Unit::init < Animal::init` and `Dock::init+0x125
< Docks::init_dock+0x128 < Build::activate+0xcbf`. `docs/TRANSPORT.md`
§5.2; landed in `crates/sim/src/transport.rs` (`dock_open`).

### 3.5 The armies — `Army::find_target@006f69b0`, `Unit::come_out@00617c10` (2026-08-25)

`Army::find_target` draws `Random::get(game_random, 0, 0xffff)` **once per
candidate city** that reaches its score (`% 200 + 900`, the base), once
per candidate fort, and once more in the low-difficulty gate's coin
(`& 1`) — so an army's retarget tick spends as many draws as there are
scoreable cities, on the army's own 256-frame phase (`docs/ARMY.md` §5,
§12). `Unit::come_out` draws once per unit leaving a building, to decide
whether it joins an army (`docs/ARMY.md` §4). Neither is reached in
run12/run13's windows; run24–26 are the captures with them
(`docs/ARMY.md` §16). The harness's `find_target` draws on the same
stream in the same order.

### 3.6 The pasture and its five animals — `Farms::add_animals@008d8f30` (2026-08-26)

**A farm's `farm_type` is `FarmStruct+0xbd`**, and `rise.pdb`'s type record
is what names it (`FarmStruct // size 0xc0`: `who`, `o`, `float[4][4]
percent`, `float[5][5] terrain_height`, `uchar[4][4] status`, `uchar valid`,
`uchar farm_type`) — not the surrounding code, which only ever compares it
to 1. **`farm_type == 1` is the pasture**, and the whole of what it changes
is this:

- **`Farms::inc_time` skips it.** So a lobby with six farms spends *five*
  crop draws a frame, not six (§3.3). Run20's list reads `1, 0, 0, 0, 0, 4`
  and its every frame's farm block is five draws long.
- **`Farms::add_animals` gives it five animals of owner 9.** The loop is
  `do { … } while (i < 5)`, guarded by the same `+0xbd == 1`; each pass is
  `Random::get` for the type (`& 1`: even `FARMCHICKEN` 0x196, odd
  `FARMPIG` 0x195), one for the `y` offset and one for the `x`
  (`% 0x180 − 0xc0` from the building, so within a tile either way), then
  `Objects::init_unit(objects, **9**, type, x, y)` — whose `Guy::init_real`
  draws once more. **Four draws an animal, twenty a pasture.** ~~Spent
  where the farm is built~~ — **corrected 2026-08-26 (§3.8): the caller is
  `Build::activate@00623e20` line 1209, not `Farms::add`**, so they are
  spent where the farm *finishes*. For a starting pasture that is the same
  breath inside `Setup::build_empire`, which is why the difference never
  showed; for one a player builds it is the completion frame, and the
  simulation models neither the draws nor the animals there.

Each animal is stamped with the farm's `o` (`Animal+0x150`), the farm's
`who` (`+0x152`) and **its own place in the five** (`+0x154`), and that
last byte is what phases its thinking.

**The animals are herdless, so `Animal::do_idle` hands them straight to
`Animal::think_farm_animal@005d7700`** — the branch §3.2 named and left
unread. `do_idle` calls `Unit::set_anim(CHAR_DEFAULT, 0, 1)` first (the
idle roll, one draw a guy when a new default animation starts), then, for
a type with `+0x218 == 0`, tests `+0x86`: a herd member rolls to wander, a
herdless one goes to `think_farm_animal` **with no clock gate of its own**.
There:

```
if (o · (slot + 1) + game->frame) % 128 != 0: return          # its phase
target = num_gatherers(farm) == 0 ? farm : farm->gather_down  # BuildData+0x70
if not farm->covers_tile(target.tile):        return
ONE DRAW: dir = rand & 7                                      # then a move
```

So a pasture costs, per frame: **five idle rolls in the unit loop, plus one
`think_farm_animal` draw for each animal whose 128-frame phase lands and
whose farm covers the tile it measures** — and **one fewer** crop draw in
`Farms::inc_time`. At frame 0 exactly one animal fires, and it is always
the first: `slot 0` with `o = 0` gives `0 · 1 + 0 = 0`.

**Owner 9 is the reason this hid for so long.** No dump prints a leader-9
object — run20's first `FULL DUMP` has 104 `ANIMALDATA` records and not one
`who 9` field anywhere — so the five animals appear in a capture *only* as
draws. The trace is what found them: run20's frame 0 has **109**
`Guy::set_anim+0x97a < Unit::set_anim+0x56 < Animal::do_idle+0x19` draws
against 104 dumped animals, and the five extra sit at the end of the animal
run with the single `Animal::think_farm_animal+0x142 < Animal::do_idle+0x43`
between the first of them and the rest — draws 138–143, in that order. The
fuzzed map is the same five and the same one.

Landed: `crates/sim/src/farms.rs` (`Farm::farm_type`, `Sim::farm_add_animals`,
`Sim::think_farm_animal`), `Sim::build_covers_tile`, and
`gamelog::Initial::farms` — the `Farms` list read off the dump, which has no
block of its own (`Farms::log_data` writes `who`, `o`, the cells, the corner
heights and then `valid`, `farm_type` as flat fields of the enclosing dump).

**What this leaves open.** The animals' **positions**, and with them where
`think_farm_animal` walks them: `add_animals`' two offset draws are spent in
the setup stream the harness does not replay, and `corner_x`/`corner_y[slot]`
with the thirds-of-a-tile arithmetic is read but not issued — a destination
here would be fiction. So is the coin that picks chicken or pig, and so are
their **animation lengths**: no dump prints an owner-9 `GUY`, so the sim
gives them an unknown length, their clocks never run out and their later
idle re-rolls are missing. None of that touches the frame-0 count; all of it
touches a long run.

### 3.7 The idle scout — `Unit::think_scout@005f6010` (2026-08-26)

The unit loop's largest single site, and the whole of frame 0's remaining
gap on three maps. An idle AI scout walks rings of cells outward from each
city it knows about and takes **two draws at the head of every ring** (a
rotation and, from ring 4 up, a phase) plus **one per cell** that is in its
own region and not really seen. Ten draws on run20 and on the fuzzed map,
twenty-four on the Great Lakes.

`docs/SCOUT.md` is the mechanic, `crates/sim/src/scout.rs` the
implementation. Two things belong here rather than there: the ring count
depends on `frame % 8` (so this site's cost changes every eight frames),
and the cell count depends on the **fog grid**, which only a `DUMP_ALL`
capture's `WORLD` block supplies — a harness world without one sees
everything and spends the ring draws alone.

### 3.8 The new farm — `Farms::add@008d8a40` (2026-08-26)

Not a per-frame site: it fires **once, where a farm is placed**, and it is
what run20's frame 1 spends two draws on that no other capture's frame 0–3
does. `Build::init` calls it at `+0x4ea` — line 249 of the decompile,
after `find_city` (line 116) and before the gather survey — gated on
`is(FARM)` and on `Build::init`'s **`restore` argument being zero**, so a
farm changing hands keeps the record it had.

**The record is created with the site, not with the finished farm.** Its
slot in the `Farms` array is chosen here (the first whose `valid` byte is
clear, else an append), and that slot is the order `Farms::inc_time` walks
in (§3.3). The building's own `BuildData+0x78` is written by `Build::init`
**after** `Farms::add` returns, which is load-bearing: the farm being
placed is invisible to the `count_farms` its own `Farms::add` runs.

**The type.** `FarmType` is `-1` none, `0` wheat, `1` pasture, and the
`+0xbd` byte also carries `4` for the ambience bit below. With `others`
the city's other farm *buildings* (sites included,
`CityData::count_buildings(FARM, exact, all)`) and `crops` the ones among
them carrying a record whose type is not the pasture
(`CityData::count_farms@007368c0`, `farm_type & 1 == 0`):

| condition | type | draws |
|---|---|---|
| no city (`BuildData+0x72 < 0`) | `o & 1` — **an odd object number is a pasture** | none |
| `others != crops` (the city already holds a pasture) | wheat, and on to the ambience | none |
| `others == crops == 4` | pasture | none |
| `get_nearest_farm_type` finds a farm within `0x480` | its `farm_type & 1`; wheat goes on to the ambience | none |
| `others == crops == 3` | pasture | none |
| otherwise | `rand & 3 == 3` → pasture, else wheat | **one**, `+0x128` |

`FarmsData::get_nearest_farm_type@008d73a0` is
`ObjectsData::find_any_building` over **every** owner, `FILTER_TYPE 0x1a1`,
`FILTER_NOT_ME`, radius `0x480` (six tiles) — so farms cluster by type
across a border as readily as inside one, and only the *nearest* answers.

**The ambience, and the two draws.** A wheat farm whose city holds **more
than one** crop farm, and none of whose city's farms already carries the
bit, spends `+0x23f` and then `+0x25b` — `rand % 3` for `x`, then for `y`
— places a `GraphicEvents::add_ambience(1, (corner + 1 + x) · 0xc0,
(corner + 1 + y) · 0xc0, 135.0f, who, o)` and ors **`4`** into its own
`farm_type`. The emitter is art; the bit is not, because it is what stops
every later farm of that city taking another, and it is why run20's sixth
farm reads `4`. The walk that looks for it is the city's building chain and
any hit ends it, so its order cannot matter.

Everything after that is the 5×5 corner-height sample, which is floats and
takes no draw.

**Confirmed on two maps, no capture needed** (run20 and the fuzzed map both
had the pair on disk since 2026-08-25): run20's frame 1 draws 43 and 44 are
`Farms::add+0x23f` and `+0x25b` under `Build::init+0x4ea <
Objects::init_build+0x82`, the fuzzed map's are its 33rd and 34th, and the
sim now spends both on both. The Great Lakes' frame 1 places a *woodcutter*
and spends neither, on both sides. No coin appears on any capture: run20's
new farm joins the city that holds the map's pasture, so `others != crops`
settles the type without one.

`crates/sim/src/farms.rs` (`Sim::farms_add`, `Sim::city_count_farms`,
`Sim::nearest_farm_type`, `Farm::valid`, `farms::AMBIENCE`), called from
`Sim::init_build`. The check is
`diff::tests::run20_s_frame_1_spends_the_farm_s_ambience_pair` and
`farms::tests::farms_add_picks_the_type_and_hands_out_one_ambience_a_city`.

**What this leaves open.**

- **`Farms::remove`'s freed slot.** The original refills the first record
  whose `valid` is clear; `Sim::farms_add` appends. Nothing on any capture
  has demolished a farm, so nothing separates the two yet.
- **A tie in `get_nearest_farm_type`.** `find_any_building` walks the
  cell-circle table and keeps its running minimum with `<=`, so the *last*
  candidate at the winning distance wins; `Sim::nearest_farm_type` walks
  the building list and keeps the first. It parts from the original only
  where two farms of **different types** sit at exactly the same distance.
- **`Farms::add_animals` is called from `Build::activate@00623e20` (line
  1209), not from `Farms::add`** — corrected here, and §3.6's "spent where
  the farm is built" holds only because a *starting* pasture is placed and
  activated in the same breath inside `Setup::build_empire`. A pasture a
  player builds mid-game owes its twenty draws at the frame it **finishes**,
  and the simulation stands its animals up only for a farm read off a dump
  (`Sim::farm_add_animals` has no caller in `Sim::activate`). No capture
  contains one.

### 3.9 The bird — `Animal::think_bird@005d79e0`, `Unit::do_air_patrol@005ea620` (2026-08-28)

A bird is the object `Objects::process_all`'s sampling creates: `init_unit(
who 9, BASE_GAIATYPES)` at the hit cell's centre (`cell·0x300 + 0x180` on
each axis) and `add_air_patrol_order` on the same point. From that frame it
is in the unit loop's who-9 slot every frame, and `do_job` runs
`do_air_patrol`, whose **first act** — the `+0x180` virtual at `+0x28` — is
`think_bird`.

**No dump prints owner 9 at all** (run20's first `FULL DUMP` has 104
`ANIMALDATA` records and not one; run13's window has none across ten
frames). So the draw-site trace is the bird's only oracle, and every claim
here is a site or a count from run14.

`think_bird`, the `0x192` arm, with `spell_time` (`UnitData+0x98`, named by
the type record; `Unit::init` clears it with `mana_burn`, so a bird starts
at 0) as its counter:

```
spell_time < 0                     → run;
else spell_time += 1, run only when frame & 7 == 0     # every frame, the step
mana_burn = 0
the order's patrol point moves:
    spell_time < 1 → x += rnd % 0x51 − 0x28 ; y += rnd % 0x51 − 0x28
    else           → x += rnd % 0xf  − 7    ; y += rnd % 0xf  − 7
    kept when 0 ≤ x < xs·0xc0, 0 ≤ y < ys·0xc0 and the new cell's region is
    the point's own (not the bird's)
spell_time ≥ 0 → the order's target object is cleared
spell_time > 0: spell_time += 1; when it is not 1, one more draw —
    rnd % spell_time; == 100 or > 799 starts the landing search
```

**Three draws every eighth frame per live bird**, and nothing else: the
landing search's thirty rounds (two draws each, over the region's cell
list) cannot fire until the counter has passed 100 — the counter is the
modulus — which is ~90 frames of flight, and no traced frame reaches it.
`0x194` returns after an `order_type` call and draws nothing.

Run14 is the whole of the evidence and it is exact:

| what | run14 |
|---|---|
| hatchings (`Guy::init_real` under `Animal::init`) | frames **96, 192, 256** |
| the sampling's pairs | 10 at 0/32/64/96, **9** at 128/160/192, **8** at 224/256 |
| `think_bird` | **3** a frame at 104…192, **6** at 200…256, **9** at 264…280 |

— which is the count read off the unit list rather than kept: the sampling
is `min(10, xs·ys/100)` less the birds alive *at that moment*, and a bird
born inside the loop is not yet counted by it.

**What the simulation has.** The hatching (a unit of owner 9 with the
type's guy, so `Guy::init_real` is spent where the original spends it), the
live count, `think_bird` whole, and the patrol point as state. `is_air` is
now the domain rather than the seam it was, so a bird does not paint the
occupancy grid a citizen walks on (`docs/COLLISION.md` §2).

**What it does not, and what each is worth.** All of it is
`Guy::set_anim`'s bird branch and the flight it serves:

- **`Guy::set_anim+0x104b`** — `set_anim(CHAR_WALK)` on a guy of owner 9
  and type `0x192`–`0x194` draws once and takes the **second** walk
  animation when `rnd % 100 > 0x31` (`set_anim:620`). Run14 spends it 28
  times: **25 as phase-7 wraps** under `Guy::inc_time+0x271` and 3 under
  `Unit::do_air_physics+0x683`. Six of the 25 fall between the last traced
  word and frame 168, which is most of what still stands between the
  simulation's stream and the original's there.
- **The lengths.** A wrap falls when the animation ends, and the bird's
  two lengths are art data — the `AnimationPacket` frame counts. `Art` is
  read out of a dump's `GUY` blocks, and no dump has a bird, so there is
  no entry: the simulation's bird carries `piece = −1`, every length
  lookup fails and it never wraps. **This is the open item**, and the
  check is run14's own list — 97, 127 (×2), 142, 150 (×3), 181, 193, 212,
  223 (×2), 238, 243 (×8), 257, 261, 274 (×2), 279, 284.
- **`Unit::do_air_physics@005e86d0`** and the flight: unread past its
  `set_anim` site.
- **The landing search**, unreachable on every capture; a bird that
  reaches it is recorded in `Gaia::bird_landings` rather than drifting
  quietly.

## 4. Run12 attributed

Frame 0, draws 0–119 (the LCG from `0x3bd39ae9`):

| draws | what | how it is placed |
|---|---|---|
| 0–1 | the AI's sweep, `compute_sites`' stride | `site_mark 16` (`docs/AI.md` §12.1) |
| 2–19 | the market, six goods × 3 | **pinned**: the flux values match at offset 2 and nowhere else |
| 20 | scout `0/0`'s idle anim | `p = 91` → variant 2; the dump shows both its guys go `0 → 2` |
| 21 | the scout's second guy, the dog, rolling its own (`p = 2`) — overwritten by the mirror in phase 7 | **its `end_time 61` at the frame's end**, which only `set_anim`'s tail writes (`docs/ANIM.md` §5; the second reading) |
| 22–35 | ~~15~~ **14** draws — the human scout's `think_scout(0)` scan, on the reading in §3.2 | by elimination: the next `(variant 1, variant 0)` pair that the two woodcutters need is unique in the segment |
| 36, 37 | `0/1` (variant 1: `p = 79`), `0/2` (variant 0: `p = 18`) | the dump's `0 → 1`, `1 → 0` |
| 38–45 | 8 draws — the AI scout `1/0`'s two guys' idle rolls (`docs/ANIM.md` §5) and its explore: `do_move`'s `% 5` and, on the reading, the rest of its first path (**6**, not 8) | by elimination (see §6) |
| 46, 47 | `1/1`, `1/2` (variant 0, `p = 25`, `p = 8`) | the dump's `1 → 0`, `1 → 0`; the six farmers draw nothing |
| **48–87** | **the 40 animals' idle anims, one each in `o` order** | **pinned**: forty `rand % 100 → variant` outcomes match the dump's `cur_anim` at offset 48 and nowhere else (0 mismatches; the next-best offset has 15) |
| 88–107 | the ten bird attempts | all ten cells miss `0x20` on run9's map |
| 108, 109 | herd 0's walk | `% 3 = 1, 2` → `wy 34 → 35` |
| 110–115 | the six farms | all `% 1000 ≥ 20` — no sprout, and frame 1's dump shows none |
| 116–119 | **4 draws not attributed** | after `Farms::inc_time` on this reading; see §6 |

Frame 1, draws 0–53 (from `0xb6194ba1`): the rotation puts the AI first.

| draws | what |
|---|---|
| 0–43 | the AI's turn — the script's eight `rand_int`s and the farm's placement (`docs/AI.md` §5, §12.1) — then player 1's units: `1/0`'s re-plan (`dest_y 19704 → 18168`, `tolerance 384`), `1/1`'s walk to the site; the split is the simulation's to measure (§5) |
| 44 | `1/2`'s `wait = 400 + 95` |
| 45, 46 | `0/1`'s `400 + 46`, `0/2`'s `400 + 17` — **pinned** (the dump's `446`, `417`, `495`) |
| 47 | farm 0 |
| 48, 49 | farm 1: `% 1000 = 19 < 20`, `% 15 = 12` — **the sprout, pinned** to the cell that grows from frame 2 |
| 50–53 | farms 2–5 |

Frames 2 and 3: the six farms, and nothing else — the animals are mid-anim
(their `end_time`s are 90–250 frames), the scouts are past `idle 1`, no
bird or herd tick falls there.

### 4.1 Run13 attributed (the window, sim-frames 95–103)

Run13 is the same game with `LogStartFrame=95 LogEndFrame=105`
(`docs/ORACLE.md`); each `FRAME n` block's first pass is the end of
sim-frame `n − 1`, and the per-unit state between two passes says what
each draw did. The three frames that mattered, from the words `0x259a53dd`
(end of 98), `0x60032f25` (end of 99) and `0xa45fecaf` (end of 100):

| sim-frame | draws | what |
|---|---|---|
| 99 | 8 | 0–1: the AI's new citizen `1/6` — `Guy::init_real`'s `% 100` and its first idle anim (`% 100 = 74` → variant 1, the dump's `cur_anim 1`); 2–7: the six farms, no sprout |
| 100 | **18** | **0–11: the twelve `end_time 101` fish, `o` 4, 7, …, 37, each `% 100` → variant** — the replay gives `2,0,0,1,0,0,0,0,0,0,0,0` and the dump's new `cur_anim`s are exactly those, in that order, with `cur_time` reset to 0 (the phase-7 wrap, §2 step 7); 12–17: the six farms |
| 101 | **21** | the rotation puts the AI first: **0–5: the AI's farmers `1/3`, `1/4`, `1/5`, two `% 4` each** — `1,0 / 3,2 / 1,0`, and their new `MoveOrder` goals are the farm's centre `− 264 + 192·r` on each axis; **6: sheep 0 arrives** (its walk cut at `12/16`, `set_anim(CHAR_DEFAULT)` from `do_idle`, `% 100 = 27` → variant 0); **7–12: the human's farmers `0/3`, `0/4`, `0/5`** — `2,1 / 0,3 / 2,1`, likewise; **13: the human scout's wrap** (`60/61 → 0/61`, phase 7, leader 0 before anything else); **14–20: the seven farm draws** — farm 1 of the list (the AI's `2003`, twelve empties) sprouts at draw 15 (`% 1000 = 6`) and draw 16 picks `65297 % 12 = 5`, the sixth empty cell in print order, which is the one cell that appears in the next pass |
| 102, 103 | 6, 6 | the farms; the six farmers' walks start on 102 and `1/6`'s on 103 **with no draw** — `find_path` takes them |
| 95–98 | 23, 28, 7, 6 | the AI scout's `think_scout` re-target at 95 (17 beyond the farms), the birds' twenty at 96 (`frame & 0x1f == 0`) plus two, one at 97, the farms alone at 98 |

The farmers' modulus is settled by the `% 4` match alone: under `% 3` the
same twelve draws read `0,2,1,1,2,2 / …`, and a `+312` offset (r = 3)
appears twice among the six goals. The `Farms` list order that the sprout
pins is the **AI's three farms first, then the human's, then the AI's
seventh** — the setup's creation order, not `BUILDDATA`'s. Three
`DUMP_ALL` dumps agree — run3, run13 and run5, a different lobby — and the
loop is `Setup::build_game@005ac190`'s walk over `info.player[k]`'s
start-slot field (unread beyond that). The sim keeps `Sim::farm_order`,
filled at `activate` and walked by `farms_inc_time`; `build_sim` orders
the starting farms higher slot first.

### 4.2 Frame 0 on two other maps, and the "fifteen missing draws" (2026-08-26)

The shortfall booked as "15 missing draws, one fixed block" **is four
blocks, two of which cancel** — which is why it came out the same on both
maps and looked like one thing.

The two captures are **run20** (the islands lobby, `rontrace-run20.log`) and
the fuzzer's control run on a map nobody tuned against
(`gamelog-fuzz-424242-early.txt`, `rontrace-fuzz-424242.log`). Both traces
carry `cover=1` over frame 0, so every draw is placed by site. Folded, and
set beside the harness's own draws folded by phase (`Sim::phase_marks`, §5):

| block | run20: theirs / ours | fuzzed: theirs / ours |
|---|---|---|
| `Leader::compute_sites` (the sweep's stride) | 2 / 2 | 2 / 2 |
| `GameDaemon::calc_market` | 18 / 18 | 18 / 18 |
| `Unit::do_idle` → `Unit::set_anim` (the two scouts, two guys each) | 4 / ~~8~~ **4** | 4 / ~~9~~ **4** |
| **`Unit::think_scout`** (`+0x436` ×4, `+0x458` ×2, `+0x64c` ×4) | 10 / **10** | 10 / ~~11~~ **10** |
| `Animal::do_idle` — the dumped animals | 104 / 104 | 123 / 123 |
| **the pasture** — 5 idle rolls + 1 `think_farm_animal` (§3.6) | **6 / 6** | **6 / 6** |
| `Objects::process_all` — the ten bird attempts | 20 / 20 | 20 / 20 |
| `Herd::process` | 2 / 2 | 2 / 2 |
| **`Guy::inc_time` — the phase-7 wraps** | 4 / ~~0~~ **4** | 5 / ~~0~~ **5** |
| `Farms::inc_time` — the *five* crop farms (§3.6) | 5 / 5 | 5 / 5 |
| **total** | **175 / 175** | ~~196 / 195~~ **195 / 195** |

**The two struck rows closed together on 2026-08-26, and they were one
line.** §6's stand/wrap entry has the finding; the short version is that
the sim's camp-arrival stand in `do_non_flat_gather` was its own invention
— the original's branch there is a two-way `CHAR_DUMP_WOOD` /
`CHAR_DUMP_ORE` — and that stand had been resetting the citizens' clocks,
so the wraps never fell due. Removing it moved four draws, not two. Frame
0 is now compared **draw for draw** rather than by these blocks
(`diff::tests::frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`),
on run20 and on the Great Lakes.

(The "ours" column is after **two** sessions' work. Before the first the
pasture row read `6 / 0` and the farm row `5 / 6`, for 160 and 180; before
the second the `think_scout` row read `10 / 0`, for 165 and 185.)

Read down the two bold rows that are not the pasture:

- **`think_scout` is modelled** (`docs/SCOUT.md`, 2026-08-26), and it is
  the mechanic that turns the next row from a curiosity into a bug with a
  price.
- ~~**The stands and the wraps cancelled, map by map**~~ — **closed
  2026-08-26, and the two halves were one line** (§6). They cancelled
  +4/−4 on run20 and +5/−5 on the fuzzed map, and stopped cancelling when
  `think_scout` landed, because the two spurious stands put the scout's
  ring rotations on the wrong stream: on run20 that cost nothing (ring 5
  gives four cells on either stream), on the fuzzed map one cell, which
  was the whole of `196` against `195`. Both are gone. The original draws
  **no** unit-phase stand for a gathering citizen at frame 0; those
  citizens' guys run their animation out in phase 7 and re-roll there, and
  the sim now does the same. The evidence that made this look half
  explained — run20's end-of-frame-0 dump has `1/1` and `1/2` at
  `cur_anim 1, cur_time 0, end_time 232` (wrapped) and `0/1`, `0/2` at
  `cur_anim 0, cur_time 1, end_time 33` (a `set_anim` that did not draw)
  — is not the whole record: the trace says four wraps and the dump shows
  two, because a wrap whose roll lands on the slot already running leaves
  `cur_time` stepping normally. The **sequence** settled it where the
  dumped clocks could not, and no `GUYS=4` capture was needed.

Frame 1 and frame 2 fall out of the same fold. Every row below is now a
site rather than a block — `--diff`'s `by phase` note prints the labels
`--trace` prints (§5):

| frame | theirs | ours | what is left |
|---|---|---|---|
| 1 (run20) | 53 | **53** | none — **draw for draw, the whole frame, since `go_around_building` landed 2026-08-26** (`docs/ORDERS.md` §4.6.1). The last `Unit::do_move+0xe84` was a unit whose straight line clips a *building* short of its waypoint; the sim now finds the same detour and spends nothing on it. `Leader::produce_building` is 39 + 4 on both sides (`docs/AI.md` §2.20) and the farm it places lands on the original's own tile; `Farms::add`'s pair, the three `do_non_flat_gather+0x54b` and the five farms were already matched |
| 2 (run20) | 5 | **5** | none — the five crop farms and nothing else, on either side |
| 1 (fuzzed) | 45 | **43** | ours is two *short*: one `Leader::produce_building+0xc99` (29 against 30 — a spiral candidate the original scores and the sim does not) and one `do_non_flat_gather+0x54b`. The jitter is **3 on both sides** here, one of the 2×2's four sub-positions being blocked — the second map that makes the inclusive reading a rule |

The unit that pays run20's last `+0xe84` is `1/1`, not the `1/0` §6 once
named; and that frame read 53 against 53 for two days while wrong in three
places that cancelled. Both stories are in `docs/JOURNAL.md`, 2026-08-27;
the arithmetic is `docs/AI.md` §2.20 and the lesson is §5.1's.

The Great Lakes lobby (run10/run12/run13, traced as run14) is the third
map `think_scout` is checked on and the only one that exercises the
**foreign**-city arm: `+0x436` ×6, `+0x458` ×2, `+0x64c` ×16, which is one
city at `max_ring = 12, step = 2` and a second at `max_ring = 3, step = 1`.
Seeded from the trace the harness reproduces all twenty-four, split
4/2/1 then 2/0/15. On the frame's own stream that map's frame 0 went 96 →
128 → **120 of 120** as the scout landed and then the stand was removed;
it is the second map the draw-for-draw check covers, and its four
woodcutters are the four wraps `docs/ANIM.md` §5 could not place.

`rondata::diff`'s
`run20_s_pasture_grows_nothing_and_its_five_animals_draw_six` pins the
run20 column, and
`frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps` pins the
whole of it, in order.

## 5. The harness

`rondata --diff` now reads the per-frame records out of a `DUMP_ALL` dump
(`gamelog::Log::frame_seeds`), borrows them from a sibling for a dump that
lacks them (run12 is run9/10/11's — and only a sibling whose setup trace
ends on the same word qualifies; run3 is the same map from a different
start), and after each traced frame (`diff::Built::tick`) prints the sim's
own draw count against the original's — the number each mechanic below
moves — and **installs the original's word** so that the next frame starts
on the true stream. That is a correction, printed as one, and it is what
lets the script's frame-1 branch be checked on run10 without the whole of
frame 0 modelled: the boom order that the original takes.

**And it folds its own draws by phase** (`Sim::phase_marks`, filled only
while `Sim::trace_phases` is set — the harness sets it, the soak pays
nothing). The mark before each phase of `Sim::tick`, and one before each
unit the loop visits, turn a frame's count into a line like

```
rng: frame 0: ours by phase — strategy_all 2, markets 18, unit 0/0 2,
  unit 0/1..0/2 ×2 1, unit 1/6 2, unit 1/7..8/115 ×106 1, unit 9/116 2,
  unit 9/117..9/120 ×4 1, gaia 22, farms 5
```

which is the harness's answer to `tools/trace/report.py … sites`, and lines
up against it directly. §4.2 is what that comparison found the first time it
was run; a bare total would not have.

**Every phase in that line is now a site** (2026-08-26, §5.1): the same
frame reads `Leader::compute_sites+0x4ac 1, … GameDaemon::calc_market+0x54
1, … Guy::set_anim+0x97a < Unit::do_idle+0x7d 2, … Unit::think_scout+0x436
1, …`, and a phase name left standing in it — `unit 1/9`, `guys_inc_time`
— is a draw *no mechanic has claimed*, which is the useful half of reading
it.

**And it reads the trace itself now** (`rondata::trace`, 2026-08-26). The
`rontrace.log` format is thirty-two-byte records and the Rust side parses
it, so three things stopped being Python's job:

- `rondata --trace <rontrace.log>` prints the original's own per-frame
  fold **by site**, in the same shape `by phase` prints ours — the two
  lines sit one above the other and no longer have to be lined up by hand.
  Addresses are bare; naming them still needs the Ghidra export's
  `INDEX.tsv`, which never enters this repo, so `report.py` is where a name
  comes from.
- `Trace::run_in(frame, lo, hi)` isolates **one function's own draws** from
  the ones its callees took, which is what makes a per-mechanic assertion
  possible at all.
- A mechanic may mark its own draw sites (`Sim::mark`, under the
  original's offsets — `crate::scout`'s three are the first), and
  `diff::mark_sites` expands the marks into one label per draw. Seed the
  sim with the trace's own word, run the mechanic, and compare the two
  **sequences**.

That last one is the point. A count cannot tell four rotations and two
phases from three and three; a sequence can, and it is checkable **while
the frame's stream is still wrong upstream** — which is exactly the
position `docs/SCOUT.md` landed in.

### 5.1 The frame as one sequence (2026-08-26)

The scout's check was seed-anchored and one function wide. Widening it to a
**whole frame** needed one more piece: the trace's addresses and the
harness's marks had to be the same strings.

**`trace::SITES` is that table, and it is deliberately small.** Each row is
`(address, an optional caller, the label)`, and the label is a `pub const`
in the mechanic's own module — `sim::market::SITE_A`,
`sim::anim::SITE_WRAP`, `sim::gaia::SITE_HERD_X` — so the name lives beside
the code that spends the draw and only the address lives in `rondata`. It
is **not** a symbol table: naming a trace in general is still
`report.py`'s job with the Ghidra export, and nothing from that export
enters the repo.

The optional caller is what makes it work at all, because **one address is
several sites**. `Guy::set_anim+0x97a` is the idle roll for an animal
(`< Animal::do_idle+0x19`), for an idle unit (`< Unit::do_idle+0x7d`), for
a gathering one's camp stand (`< Unit::do_non_flat_gather+0x10f`) and for
the phase-7 wrap (`< Guy::inc_time+0x271`), and only the record's `ebp`
chain (`Draw::up`) separates them. A row with a caller matches when that
address is anywhere in the chain, and it must precede a bare row for the
same site.

`Trace::labels(frame)` is then the original's frame as a `Vec<String>`,
`diff::mark_sites` is ours, and the comparison is one `assert_eq!`. Two
things fall out of it that a per-block table did not give:

- **An unmodelled draw reads as a bare hex address**, so a hole in the
  simulation is legible rather than silent. The whole-frame test asserts
  there are none at frame 0.
- **A draw the sim spends under the wrong caller reads as a mismatch at
  the exact index**, with the label on both sides. That is what found the
  stand/wrap swap (§6) on the check's first run: `ours
  Guy::set_anim+0x97a < Unit::do_non_flat_gather+0x10f` against `theirs
  Guy::set_anim+0x97a < Unit::do_idle+0x7d`, at draw 22.

`diff::tests::frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`
is the check, on run20 and on the Great Lakes; the failure message prints
the first parting with three draws either side rather than 175 lines of
`assert_eq!`. Made to fail on purpose by dropping `sim::anim`'s wrap mark,
which reads as four `guys_inc_time` against four `Guy::set_anim+0x97a <
Guy::inc_time+0x271` at draw 166.

**A phase left standing in the fold is worth more than a matching total,
and marking one has now paid twice.** The second was
`Leader::produce_building`, frame 1's last unmarked phase
(2026-08-26): the frame read 53 against 53 and the two marks split it into
`+0xc99` 39 against 41 and `+0x1805` 4 against 1 — **three** defects whose
residues happened to cancel, one of which (`buildings_allowed`) was a map
layer the dump had been carrying unused. `docs/AI.md` §2.20 has all three.
The rule that falls out: *mark the phase before believing the total*, and
mark it even when the total already agrees.

**The counts, 2026-08-24**, run10 with run11/run3/run12 as siblings:

| frame | ours | the original's | the gap |
|---|---|---|---|
| 0 | **48** | 120 | the 52 idle anims, the two scouts' 23, the 4-draw tail — 48 is exactly sweep 2 + market 18 + birds 20 + herd 2 + farms 6 |
| 1 | **54** | **54** | none: the script's eight, the placement, the AI scout's re-plan and the builder's walk, the three woodcutters, the seven farm draws — all of frame 1 is modelled |
| 2 | 6 | 6 | none |
| 3 | 7 | 6 | one — ~~the sim's `do_move` draws its `% 5` for one of the three woodcutters' walks queued that frame~~ **stale: re-measured 2026-08-26 and it is a `Unit::do_non_flat_gather+0x54b`, a citizen picking a tile a frame the original does not.** The `do_move` reading was right for run20 and is fixed there (§6); on this map it had already stopped being the cause. Still a lead for the run6 `0/2` divergence at frame 4 |

With the words installed, run7's first script call takes the original's
branch (steps 6 → 7 → 9 → 10 → 11, pinned), run6's AI trains its three
citizens on the original's frames, and run10 holds at 268 unlinked
unit-frames with `1/9` now linking — only `1/10` at 1505 is missing.

**The window, 2026-08-24**: run10 with run11, run3 and **run13** as the
siblings (`--sibling` takes any dump whose setup trace ends on the same
word; run13's `frame_seeds` are sim-frames 94–103). The harness installs
run13's word at the end of 94 and then counts:

| sim-frame | ours | the original's | the gap |
|---|---|---|---|
| 95 | 6 | 23 | the AI scout's `think_scout` re-target — **modelled since 2026-08-26**, `docs/SCOUT.md`; the window has not been re-run against it |
| 96 | 26 | 28 | the birds' twenty on both sides; the scout's two |
| 97 | 6 | 7 | the scout's one |
| 98 | **6** | **6** | none |
| 99 | 6 | 8 | the new citizen's two creation draws — the sim creates it without them |
| 100 | 6 | 18 | the twelve fish wraps — the sim has no animation clock |
| 101 | 19 | 21 | the sheep's arrival and the scout's wrap; the twelve farmer draws and the seven farm draws are on both sides |
| 102 | **6** | **6** | none — the six walks start without a draw on both sides |
| 103 | **6** | **6** | none |

So the sim's `do_move` gate agrees with the original's on all seven walks
the window starts (§6's frame-3 draw is specific to those three), and the
next gaps are the ones §6 names: the animation clock and the scout.

**With the animation clock, 2026-08-24 (`docs/ANIM.md` §6)**, the same
run: 98 **6/6**, 99 **8/8**, 100 **18/18**, 101 ~~20/21 (the scout's wrap
in, the sheep's arrival out)~~ **21/21 since 2026-08-27**, 102 **6/6**, 103
**6/6**; 97 6/7, 96 26/28, 95 6/23 are the AI scout's. With run12 as a
sibling too, frame 0 is **96 of 120** — the forty animals' first idles, the
two scouts' four and the four woodcutters' stands — and frames 1 and 2 hold
at 54/54 and 6/6.

The sheep closed as a **harness correction, not a model**. An animal
wanders on draws of the sync stream, so between two traced frames it walks
somewhere the sim's own stream sent it and arrives on the wrong frame — and
an arrival costs one draw, *in the middle of the unit loop*, between one
player's units and the next. `Sim::reseat_animal` puts gaia's animals back
on every traced frame's dumped position and goal, beside the clocks and the
word; the drift is reported rather than hidden (run10's is under a tile).
Until the untraced stretch is modelled there is nothing else that can put
that draw in the right place, and it is worth 20 ticks of the headline:
without it the human's three farmers spend the AI's leftovers.

The siblings' traced frames are **pooled** (`borrow_from_siblings`): with
run12 and run13 both on the list, a run of this lobby gets the true word at
the ends of frames 0–3 and 94–103, and the dump's own word wins where two
name a frame. Run6 now takes run13's correction at 94, so its farmers'
first re-target runs on the original's stream — their disagreement count
over 432 frames fell from 662/432 to 588/372 (the run6 pin's ceilings).

## 6. What is not established

**The draw-site trace exists (run14, 2026-08-24; `docs/ORACLE.md`, "The
draw-site trace and function coverage", `tools/trace/`).** Every draw of
frames 0–3 and 95–103 is now placed by its *site* — the return address
into the function that called `Random::get` — on the same game as
run12/run13 (the frame-0 word `0x3bd39ae9`, the counts 120/54/6/6 and
23/28/7/6/8/18/21/6/6 reproduced). The items below that it settles are
struck through and point there.

- ~~**The 4 draws at 116–119 of frame 0.**~~ **Settled by the trace: they
  are at 110–113, not 116–119, and they are phase-7 wraps** —
  `Guy::set_anim+0x97a` < `Guy::inc_time+0x271` < `Unit::inc_time+0x3e`,
  followed by the six farms at 114–119 (the farms are the frame's *last*
  draws). The rest of frame 0, in order: 2 `compute_sites`, 18 market,
  **4 stands from `Unit::do_idle+0x7d`** (20–23), **24 `think_scout`**
  (`+0x436` ×6, `+0x458` ×2, `+0x64c` ×16; 24–47), 40 `Animal::do_idle`
  (48–87), 20 `Objects::process_all+0x2df`/`+0x30b` (88–107), 2
  `Herd::process` (108–109).

  **Closed 2026-08-26, and the `GUYS=4` window was never booked.** The
  whole-frame sequence check (§5.1) named the divergence on its first run
  — draw 22, ours `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0x10f`
  against theirs `< Unit::do_idle+0x7d` — and the answer was one line of
  the harness's own: **`do_non_flat_gather`'s camp-arrival branch has no
  `CHAR_DEFAULT`** (`:398`–`:416`, the listing at `5f0b5e`–`5f0b89`). The
  invented stand had been resetting the citizens' clocks, so the wraps the
  original spends in phase 7 never fell due; removing it moved four draws,
  not two, and both traced maps' frame 0 now matches draw for draw.
  `docs/ANIM.md` §5's "the four woodcutters' draws are unit-phase stands"
  is superseded — they are wraps. The four readings this replaced, and why
  each looked right, are in `docs/JOURNAL.md` (2026-08-24 and 2026-08-26).
- ~~**The human scout's ~~15~~ 14 and the AI scout's ~~8~~ 6** (the dogs' rolls
  are the other three, `docs/ANIM.md` §5) are placed by elimination,
  not by outcome~~ — **placed by site (run14): `Unit::think_scout` draws
  24 times at frame 0**, at three sites — `+0x436` (6), `+0x458` (2) and
  `+0x64c` (16) — all under `Unit::think+0x7da` < `Unit::do_idle+0x94`, in
  the order 436 436 436 458 64c 436 458 436 64c 64c 64c 64c 64c 436 64c ×9;
  and 15 at sim-frame 95 (`+0x436` ×6, `+0x458` ×6, `+0x64c` ×3), none on
  96–103. No pathfinder draw and no `do_move` draw appears anywhere on
  frames 0–3, so the AI scout's first path costs nothing on the stream.
  Which of the three sites is the scan and which the re-target is the next
  reading of `think_scout@…` with the offsets in hand: `think_scout`'s scan draws once per unseen candidate cell,
  which needs the seen map the sim does not keep, and the AI scout's first
  path is planned by a pathfinder wired to `do_move`'s one draw. ~~A `GUYS=4`
  or `PATHFINDER=…` frame-0 capture would settle both.~~ **And it is now
  the whole of frame 0's remaining gap, on two further maps (§4.2): ten
  draws on run20 and ten on the fuzzed one, `+0x436` ×4, `+0x458` ×2,
  `+0x64c` ×4 in both.** That the count is identical on two unrelated maps
  is itself a lead — a scan whose draw count were "one per unseen
  candidate cell" would not be, so at least one of the three sites is
  fixed-count. Reading it is the next step, and it needs no capture.
- ~~**Frame 1's split** between the script + placement and the two AI
  moves: measured by running the sim on the true stream (§5), not by
  reading.~~ Measured: the sim draws 54 of 54, so the split is whatever the
  sim's script, `produce_building` and `do_move` take — and they add up.
- **Frame 3's extra draw.** The original's frames 2 and 3 carry six
  `Farms::inc_time+0x1ae` draws each and no other `game_random` call
  (confirmed by site, run14), so the divergence is entirely the sim's own
  gate — the one before `Unit::do_move+0xe84`'s `% 5`: `invalid_loc` on
  the order's cell, `path_recursion > 1`, and `find_path`'s return.

  **Two thirds of it closed 2026-08-26, and the answer was upstream of the
  gate: `find_path`'s pull-back** — while `invalid_loc(goal tile)`, pull
  the goal back toward us by `(sx, cy)`, rewriting `mo->waypoint` and the
  stack top (`005fbaa6`–`005fbb56`, `docs/ORDERS.md` §4.6). A woodcutter
  sent at a **forest tile** is therefore sent at the last open point short
  of it; the simulation had marched in, found the tile invalid and paid
  the grid draw for a detour the original never needed. The last
  `+0xe84` went with `go_around_building` proper (§4.6.1): run20's frame 1
  is **53 against 53, draw for draw**, and its path-stack disagreements
  over five frames fell 25 → 17.

  **And the Great Lakes' frame-3 extra draw is no longer this one at all**
  — it is a `Unit::do_non_flat_gather+0x54b`, a citizen picking a tile a
  frame the original does not. The `do_move` attribution was only ever
  right for run20. The story, and the residue whose owner this document
  named wrongly twice, is in `docs/JOURNAL.md` (2026-08-26).

- ~~**`Farms::add`'s two draws.**~~ **Settled 2026-08-26 and modelled,
  §3.8**, with no capture: they are the **ambience emitter's** `x` and `y`
  offsets, spent once per *city* by the first wheat farm placed while the
  city already holds more than one crop farm and no emitter. The
  `farm_type` coin at `+0x128` is a third site and fires on none of the
  three captures — run20's new farm joins the city that holds the map's
  pasture, so `others != crops` settles its type without one. Original
  text: Run20's frame 1 has `Farms::add+0x23f` and `+0x25b` under
  `Build::init+0x4ea` < `Objects::init_build+0x82` — the AI's new farm
  being created — and the sim draws neither, which is two of that frame's
  three-way gap — the other is one `Leader::produce_building` draw short
  and two spurious `Unit::do_move+0xe84` (§4.2).
- ~~**`Leader::produce_building`'s draw count.**~~ **Settled and modelled
  2026-08-26** — `docs/AI.md` §2.20. Its two sites are marked
  (`sim::ai_place::SITE_SPIRAL`, `SITE_JITTER`) and run20's frame 1 is
  39 + 4 on both sides, with the farm on the original's own tile. The
  three defects behind the one-draw gap were an exclusive 2×2 jitter, a
  stride test on the wrong index, and an unmodelled
  `WorldData::buildings_allowed`.
- ~~**The world grid's waypoints sit at `cell + off`, not at the cell
  centre**~~ — **settled and implemented 2026-08-26**, no capture:
  `toff` is the current move order's own `off_x/off_y`, read through
  `UnitOrder::is_move` / `update_move_order` (`docs/PATHFINDER.md` §4.1,
  §7). Run20's `1/0` now walks the original's own `+504` lattice and
  three of its five world nodes are the original's exactly
  (`diff::tests::run20_s_world_chain_sits_on_the_move_orders_own_offset`).
  Original text: (2026-08-26, found while closing the item above and
  unread).
  Run20's unit `1/0` walks a `find_wpath` chain the original logs at
  `(42744, 37368)`, `(41976, 38136)`, `(41976, 38904)`, `(41208, 39672)`,
  … — every one of them `cell*0x300 + 504` on both axes, where the
  simulation's `astar_path` emits the cell **centre**, `cell*0x300 + 0x180`.
  It is `toff`, and `docs/PATHFINDER.md` §7 already had it: a reconstructed
  world node is pushed at `node + toff − 0x180`, and the simulation carries
  that as a **stated seam** (`path.rs`: "the target-is-a-unit offsets
  (`toff`) are zero — move orders here have point goals"). The arithmetic
  closes: `504 − 0x180 = 120`, and the order's own `off_x` is `504` —
  `41976 mod 0x300`, from a destination the simulation computes the same
  way the original does. So the seam is wrong for a *plain* move, not only
  for a unit target: `toff` is the order's `+0x4c/+0x4e`, which on a
  `MoveOrder` is `off_x/off_y` (`docs/ORDERS.md` §4.1). That one number is
  most of run20's remaining path-to disagreements and all of `1/0`'s
  position drift, and it needs no capture: the dump on disk has both
  sides.
- ~~**And the goal at the bottom of that stack is `0x18` short of the
  order's own**~~ — **settled 2026-08-26, and it is not the pre-walk.**
  It is `Group::action_move_near`'s own goal push: a group plans one path
  on the global `grouppath` whose `FINAL` entry is the **raw slot
  destination**, un-snapped, and hands it to its members
  (`docs/GROUPS.md` §6.7, and §13 for the three disagreements it costs
  run20). The simulation does not implement §6.7 at all; the queue's item
  31 is that, not a pre-walk. `docs/PATHFINDER.md` §12 records why the
  pre-walk and `do_move` are both ruled out. Original text:
  (2026-08-26, the same comparison). The original logs
  `(41952, 36576)`, both exact multiples of `0x30`, where the order itself
  is at `(41976, 36600)` — `add_move_order`'s `u*0x30 + 0x18`, which the
  `off_x = 504` above confirms for the original too. So the snap is not the
  difference: `find_wpath`'s pre-walk moved the goal before the dump saw it
  (`docs/ORDERS.md` §4.6's table has that walk stepping in `0x180`/`0x30`
  until `get_tregion` matches). `docs/PATHFINDER.md`, and it costs a `grep`
  of the same dump.
- **The pasture's twenty creation draws, and its animals' art** — §3.6's
  own open list: the offsets, the chicken/pig coin and the animation
  lengths all live in streams or dumps that no capture carries.
- **Diplomacy's cadence** (`Leader::diplomacy`, nine sites; 0 draws on
  frames 0–3).
- **The caravan road, frames 10–11** (run16, `docs/ORACLE.md` "The
  attrition run"): 220 and 248 `game_random` draws at
  `PathFinder::calc_road_cost+0x46` < `PathFinder::astar_caravan_road` <
  `PathFinder::find_road` — the game planning a road on the sim's stream,
  two frames after the start, with no caravan in the game. Who calls
  `find_road` at frame 10, and whether the sim's draw count for those
  frames (currently 6) can be made to match, is unread. Run14's 285 frames
  carry the same two spikes; they were outside its reported frames.
- ~~**Animals, and the animation clock.** The 40 idle-anim draws are pinned,
  but the sim has no gaia units and no `cur_time`/`end_time`: a guy's next
  draw falls when its animation ends (§2 step 7), and the animation lengths
  are **art data** (the packet's frame counts; run12's `end_time`s cycle
  101/116/170 for the fish and 90/109/250 for the sheep with the guy's
  scale variant, `o % 3`; the scout's idle is 41 or 61, the citizen's 33,
  the sow 47). The plan: `rondata` reads the lengths from a dump's `GUY`
  blocks (each `(type, scale, anim) → end_time`) until a BHA reader exists,
  and the sim takes them as an input like the map. Then the clock itself —
  every guy's `cur_time` stepped in phase 7, the wrap's draw — and
  `animal.rs`: the sheep's `do_idle` wander and `think_fish`'s cadence
  (unread; the fish drew nothing but wraps on frames 0–3 and 95–103), with
  the herds already in `gaia.rs`. Run13 says the clock is worth 12 of the
  18 at sim-frame 100 and 2 of the 21 at 101 (§5).~~ **Done, 2026-08-24 —
  `docs/ANIM.md`, `crates/sim/src/anim.rs`.** The clock, the wrap's roll,
  `init_real`'s, the arrival, the mirror, the animals as units of owner 8
  with the wander; the lengths and pieces as the `Art` input read out of
  the dumps (the variant is `(seed + o) % 3` for gaia, the gender bit `o &
  1` for a citizen); the harness installs the clocks beside the words. What
  it left open is its §9: the woodcutters' un-stepped frame 0, the dog's
  own roll, the unobserved lengths, `think_farm_animal`.
- ~~**Birds after creation** (`think_bird`, `do_air_physics`)~~ — **read
  and modelled 2026-08-28, §3.9**: `think_bird` is three draws every
  eighth frame per live bird, and its landing search is unreachable.
  `do_air_physics` and `Guy::set_anim`'s bird branch stay open, and the
  25 phase-7 wraps they leave unspent are what still stands between the
  simulation's stream and the original's after frame 103. And
  `Farms::add`/`add_animals` at a farm's creation — event-driven, unread
  past their draw sites.
- ~~**`Checksum Dump` / `Checksum Break`** (`gamelog.ini`; internal strings
  2849/2850, read by `GameLog::init` right after `DUMP_ALL` with default
  −1 into `log_start_frame`/`log_end_frame`, which `begin_frame`/`end_frame`
  gate the dump on) are very likely a **frame window** for `DUMP_ALL` — the
  way to a per-frame trace of frame 100 without a 6 GB file. Untried.~~
  **Settled 2026-08-24, run13** (`docs/ORACLE.md`, "The frame window is
  real, and it is not the keys we guessed"). The window exists but those are
  the wrong keys: `Checksum Dump`/`Checksum Break` land in
  `game_log.checksum_dump`/`checksum_break`, which `say_checksum` compares
  against the **checksum record index** — one extra `dump_all` at record N,
  and an `int 3` at record N (a crash here, no debugger). The window is
  **`LogStartFrame` / `LogEndFrame` in `rise2.ini`** (internal strings
  2866/2867), read through `prefs2_file` with write-back off, default −1,
  inclusive start and exclusive end, gating the whole per-frame `full_dump`.
  **`gamelog-run13-window-95-105.txt`** is the capture: `DUMP_ALL=1` with
  `[95, 105)` gives ten frame blocks at 61 MB each and nothing for frames
  0–94. Because `Game::do_frame` increments `frame` before its second
  `end_frame`, a window `[a, b)` yields the per-frame draw counts of
  sim-frames `a … b−2` plus one cumulative number for everything before `a`.
  Run13's: 1268 draws over sim-frames 0–94, then 23, 28, 7, 6, 8, 18, 21, 6,
  6 for 95–103 — the same floor of six from `Farms::inc_time` as run12, with
  spikes on top.
- ~~**Run13's 100 and 101 — the animals' cycle, or the farmers' re-target?**
  Twelve of run12's forty animals (type 411, every third `o`) carry
  `end_time 101`, and `cur_time` is the frame during the unit phase. So
  §3.2's `do_idle` predicts exactly run13's spikes: on sim-frame 100
  `cur_time == end_time − 1` for all twelve → twelve wander checks, plus
  the six farms = **18**; on 101 the anim ends → twelve idle-anim redraws,
  the six farms, and three more from a wanderer near its herd's centre =
  **21**. That leaves **no room for the six farmers' twelve re-target
  draws** on sim-frame 101 (the log's 102), where §3.3 and `docs/ORDERS.md`
  §6.5 put them from run6 — unless they fall on a frame the window did not
  count.~~ **Read, 2026-08-24 (§4.1): both, and the farm clock stands.**
  The twelve type-411 animals are fish, not herd members, so they never
  reach the wander roll; their twelve draws at sim-frame 100 are the
  **animation wrap** in phase 7, one `% 100` each, matching the dump's new
  variants in `o` order. Sim-frame 101 is the six farmers' twelve `% 4`
  draws exactly where §3.3 put them, with the sheep's arrival, the human
  scout's wrap and a sprouting farm making up the 21. The corrections that
  fell out: the wrap draw lives in `Objects::inc_time`, not `do_idle`; the
  farm's modulus is 4, not 3; and `Guy::inc_time`'s step is gated
  (`Unit+0x6c & 0x10`, `+0x82`, the type's `+4`) in ways the four
  woodcutters' frame 0 shows but this reading has not named.
