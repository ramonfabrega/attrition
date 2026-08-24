# The sync stream, frame by frame

*Established 2026-08-24 from the decompile (`tools/ghidra/`, the export under
`decomp/`) and one oracle: **run12**, the `DUMP_ALL` capture whose
`begin_frame`/`end_frame` dumps print `game_random seed`
(`docs/ORACLE.md`, "What the trace does not cover"). The method is the one
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
   `execute_events` (0), the goods, `Ammo::inc_time`, `DeathObj::inc_time`,
   then **`Farms::inc_time@008d8600`** (§3.3), `Doober::inc_time` (0),
   `Surf::inc_time` (draws — from **`internal_random`**, not the stream).
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
| `Guy::set_anim@005da300` (idle) | `set_anim(CHAR_DEFAULT)` from `do_idle`, `do_gather`'s stand, any order's rest — **but only when a new default animation starts**: a `CHAR_DEFAULT` request while the current default anim still runs (`cur_time < end_time`) returns first (`set_anim:155–224`) | 1 per guy when `UnitData+0x104 == 0`: `p = rand % 100` → variant **0** for `p ≤ 69`, **1** for `70–82`, **2** for `83–95`, **3** for `96–99` (a peasant standing on a tile with `mask & 3` takes 1 for `p ≥ 83`; `+0x9a & 0x20` collapses it to 0/1 at 69) |
| `Guy::set_anim` (`param_2 == 0xc`, the sow) | only with `param_3 != 0` — the farmer's `set_anim(CHAR_SOW)` passes 0 (all six farmers show anim 35) | 0 |
| `Guy::set_anim` (walk, a bird) | who 9, types `0x192–0x194` | 1 |
| `Guy::init_real@005db6b0` | a guy's creation | 1 (`% 100` → a 4-way variant) |
| `Unit::do_move@005f7b30:599` | the first `do_move` of a move whose path is planned, after `find_path` | 1 (`% 5` → the `far` threshold; `orders.rs`) |
| `do_non_flat_gather@005f0170` | the woodcutter/miner machine | `% 200 + 400` on choosing a tile, `% 50 + 100` and `% 100 + 300` on the waits (`docs/ORDERS.md` §6.4; `orders.rs`) |
| `do_gather` at a farm | a **new tile** (`docs/ORDERS.md` §6.5) | 2 (`GameAccess::rnd(3)` twice) |
| `Unit::think_scout@005f6010` | a scout with no orders (`think`, the bottom branch: not supply, not hero, `is(0x3a)` or `type+0xb2 & 0x10`, `get_army < 0`) — the human's too | 1 for the scan start (`% (ceil(count/100) + frame&7)`) + **1 per candidate cell** that is unseen, `invalid_loc`-clean and of the right domain (`& 7`, a score jitter); the `param_1 != 0` entry draws `% count` for a region pick and re-draws while the cell has `& 0x70` |
| `Animal::do_idle@005d7460` | an idle herd animal, every frame | the idle anim above; then, when `guy.cur_time == guy.end_time − 1`, 1 (`% 10 < 3` → wander) and, near its herd's centre (`< 0x181`), 3 more (`& 7`, `& 3`, `& 3` → a step along `move_x/y`); else `find_nearby_spot` (0) |
| `Animal::think_farm_animal@005d7700` | a farm animal, every 128 frames phased by `o · (scale+1)` | 1 (`& 7`) when its farm covers the tile |
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

Landed: the birds' sampling and the herd walk, `crates/sim/src/gaia.rs`.

### 3.3 The farms — `Farms::inc_time@008d8600`

Every frame, every farm whose `+0xbd` byte is not 1 and whose building is
complete (`Build flags & 4`; a site is skipped). Each of its 16 cells holds a
state byte and a `float percent`:

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

## 4. Run12 attributed

Frame 0, draws 0–119 (the LCG from `0x3bd39ae9`):

| draws | what | how it is placed |
|---|---|---|
| 0–1 | the AI's sweep, `compute_sites`' stride | `site_mark 16` (`docs/AI.md` §12.1) |
| 2–19 | the market, six goods × 3 | **pinned**: the flux values match at offset 2 and nowhere else |
| 20 | scout `0/0`'s idle anim | `p = 91` → variant 2; the dump shows both its guys go `0 → 2` |
| 21–35 | 15 draws — the human scout's `think_scout(0)` scan, on the reading in §3.2 | by elimination: the next `(variant 1, variant 0)` pair that the two woodcutters need is unique in the segment |
| 36, 37 | `0/1` (variant 1: `p = 79`), `0/2` (variant 0: `p = 18`) | the dump's `0 → 1`, `1 → 0` |
| 38–45 | 8 draws — the AI scout `1/0`'s explore: `do_move`'s `% 5` and, on the reading, the rest of its first path | by elimination (see §6) |
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

**The counts, 2026-08-24**, run10 with run11/run3/run12 as siblings:

| frame | ours | the original's | the gap |
|---|---|---|---|
| 0 | **48** | 120 | the 52 idle anims, the two scouts' 23, the 4-draw tail — 48 is exactly sweep 2 + market 18 + birds 20 + herd 2 + farms 6 |
| 1 | **54** | **54** | none: the script's eight, the placement, the AI scout's re-plan and the builder's walk, the three woodcutters, the seven farm draws — all of frame 1 is modelled |
| 2 | 6 | 6 | none |
| 3 | 7 | 6 | one: the sim's `do_move` draws its `% 5` for one of the three woodcutters' walks queued that frame and the original's does not — a gate in `do_move`'s planning path read differently (`docs/ORDERS.md` §4.4), and a lead for the run6 `0/2` divergence at frame 4 |

With the words installed, run7's first script call takes the original's
branch (steps 6 → 7 → 9 → 10 → 11, pinned), run6's AI trains its three
citizens on the original's frames, and run10 holds at 268 unlinked
unit-frames with `1/9` now linking — only `1/10` at 1505 is missing.

## 6. What is not established

- **The 4 draws at 116–119 of frame 0.** Nothing after `Farms::inc_time` in
  `do_frame` names `game_random`. The alternative reading — five draws in
  the buildings' `process` before the birds (`Wall::process` reaches
  `Object::disband`, which draws) and only five farms — is consistent with
  every pin too; run12's `BUILDDATA` shows no site or disband at frame 0.
  A frame-0 `GUYS=4` capture (every `set_anim` logs its index at detail 4)
  or a second `DUMP_ALL` on a lobby with a different farm count separates
  them.
- **The human scout's 15 and the AI scout's 8** are placed by elimination,
  not by outcome: `think_scout`'s scan draws once per unseen candidate cell,
  which needs the seen map the sim does not keep, and the AI scout's first
  path is planned by a pathfinder wired to `do_move`'s one draw. A `GUYS=4`
  or `PATHFINDER=…` frame-0 capture would settle both.
- ~~**Frame 1's split** between the script + placement and the two AI
  moves: measured by running the sim on the true stream (§5), not by
  reading.~~ Measured: the sim draws 54 of 54, so the split is whatever the
  sim's script, `produce_building` and `do_move` take — and they add up.
- **Frame 3's extra draw.** The sim's `do_move` rolls its `% 5` grid
  threshold for one of the three woodcutters' walks (`orders.rs`, the
  "first `do_move` of a move `find_path` refuses" reading of
  `do_move@005f7b30:599`); the original's frame 3 is the six farms and
  nothing else. The gate before that draw — `invalid_loc` on the order's
  cell, `path_recursion > 1`, and `find_path`'s return — is read
  differently for at least one of the three walks. The same three units
  are run6's earliest position divergences (`0/2` at frame 4).
- **Diplomacy's cadence** (`Leader::diplomacy`, nine sites; 0 draws on
  frames 0–3).
- **Animals.** The 40 idle-anim draws are pinned, but the sim has no gaia
  units: an animal's next draw falls when its animation ends, and the
  animation lengths are **art data** (the packet's frame counts; run12's
  `end_time`s cycle 101/116/170 with the guy's scale variant, `o % 3`).
  The plan: `rondata` reads the lengths from a dump's `GUY` blocks (each
  `(type, scale, anim) → end_time`) until a BHA reader exists, and the sim
  takes them as an input like the map. Then `animal.rs`: `do_idle`'s wander
  and `think_farm_animal`, with the herds already in `gaia.rs`.
- **Birds after creation** (`think_bird`, `do_air_physics`), and
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
- **Run13's 100 and 101 — the animals' cycle, or the farmers' re-target?**
  Twelve of run12's forty animals (type 411, every third `o`) carry
  `end_time 101`, and `cur_time` is the frame during the unit phase. So
  §3.2's `do_idle` predicts exactly run13's spikes: on sim-frame 100
  `cur_time == end_time − 1` for all twelve → twelve wander checks, plus
  the six farms = **18**; on 101 the anim ends → twelve idle-anim redraws,
  the six farms, and three more from a wanderer near its herd's centre =
  **21**. That leaves **no room for the six farmers' twelve re-target
  draws** on sim-frame 101 (the log's 102), where §3.3 and `docs/ORDERS.md`
  §6.5 put them from run6 — unless they fall on a frame the window did not
  count (94 or earlier is inside the cumulative 1268; 104 is uncounted).
  Run13's `FRAME 102` block holds the farmers' order lists and every
  animal's `cur_anim`/`cur_time`: the first thing to read next session,
  since it either confirms the re-target's frame and finds the animals'
  draws elsewhere, or moves the re-target — and `farms.rs`'s clock — by a
  frame.
