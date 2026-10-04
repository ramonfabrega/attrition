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
(a cost jitter `% 20` per node on the **road** grid — a building's road to
its city, `docs/ROADS.md`), and the
`astar_path` refusals that roll `pause = % 3 + 6`.

## 3. The per-frame sites

## 3.1 The market — `GameDaemon::calc_markets@00732180`

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

## 3.2 The objects — `Objects::process_all@0065dce0`

**The unit loop rotates.** `for i in 0..10: slot = (frame + i) % 10` — the
owner whose units go first is `frame % 10`, so at frame 0 player 0's units
run first and at frame 1 player 1's, with gaia's animals (who 8) and birds
(who 9) in their slots. Run12 shows it: at frame 1 the AI's woodcutter draws
its `wait` (draw 44) *before* the human's two (45, 46). Buildings follow in a
second, unrotated loop (`Build::process`, then walls) — **each building
whole, its queue included, before the next** (§3.28). ~~The sim's tick keeps
its buildings-first order (`lib.rs`), which is a known divergence this
document does not close.~~ **Landed 2026-08-30, §3.16** — and the divergence
was worth East Indies' word 219 → 274 and Great Lakes' sequence 99 → 576.

**And the bound is re-read, every iteration.** Both inner loops test
`o < unit_mark[who]` (and `< build_mark[who]`, `< wall_mark[who]`) at the
*bottom*, out of the `ObjectsData` array rather than out of a local, so an
object created inside the loop with a number above the one being walked is
processed on the frame it is born. §3.22 is what that is worth.

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
| `Animal::think_farm_animal@005d7700` | a **pasture's** animal (owner 9, five per farm — §3.6), every 128 frames phased by `o · (slot + 1)`, where `slot` is `Animal+0x154`, its place in the five — ~~`scale`~~ | 1 (`& 7`) when the farm covers the tile of the object it measures: the farm itself while `num_gatherers(1, 0)` is zero — nobody has **arrived** — and `gather_down`'s head once somebody has (§3.6, corrected 2026-09-03) |
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
  `Herd::process@00741760` — **2 draws**, ~~`wx' = wx − 1 + rand % 3`,
  `wy' = wy − 1 + rand % 3`~~ **`wx' = cx − 1 + rand % 3`,
  `wy' = cy − 1 + rand % 3`** — the read is the **home** cell
  (`HerdData +0x0/+0x4`) and only the write is the wander centre
  (`+0x8/+0xc`), so the jitter is about home every time and the centre is
  never more than one cell from it. Accepted when in bounds, the cell has
  no `flags & 0x70` feature and its `+0x11` byte `< 8`. Run12: herd 0's
  `wy 34 → 35` with `wx 22` kept — the pair `(1, 2) mod 3`; a herd's
  first walk cannot tell the two readings apart, because `w == c` until
  it. Corrected 2026-08-31, item 112, and it is worth Great Lakes' word
  986 → 1372 (`docs/ANIM.md` §7).

Landed: the birds' sampling, the bird itself (§3.9) and the herd walk,
`crates/sim/src/gaia.rs`.

## 3.3 The farms — `Farms::inc_time@008d8600`

Every frame, every farm whose **`farm_type` is not 1** (`FarmStruct+0xbd`,
named by the type record — §3.6; that farm is a *pasture* and grows nothing)
and whose building is complete (`Build flags & 4`; a site is skipped). Each
of its 16 cells holds a state byte and a `float percent`:

```
for each cell:                       # `4·dx + dy`, dx inner, dy outer — a row at a time
    if state == 1: percent += 0.005f
    if state == 3: percent −= 0.01f
    if percent > 0: if percent ≥ 1.0f: state = 2, percent = 1.0f
    else:           state = 0, percent = 0, empty += 1
if empty ≥ 12: chance = 20
elif empty ≥ 5: chance = (empty − 4) · 20 / 8
else: next farm
ONE DRAW: if rand % 1000 < chance:
    ONE DRAW (if empty > 1): k = rand % empty
    the k-th empty cell in that same order → state = 1     # a sprout
```

So **a complete farm costs one draw a frame** while it has five or more
empty cells, and two on the 2 % of frames it sprouts. Run12's lobby has
six farms: 6 draws on frames 2 and 3 (nothing else drew), 6 at frame 0
(no sprout), and **7 at frame 1** — farm 1 sprouted (`% 1000 = 19`,
`% 15 = 12` → its 13th empty cell, memory index 7), which is exactly the
cell the frame-2 dump shows starting to grow. The AI's seventh farm is a
site and draws nothing until it completes.

**The record is dumped, and it is diffed** (2026-08-28). `Farms::log_data`
writes each `FarmStruct` as flat fields of the enclosing dump: `who`, `o`,
sixteen `percent[scan][scan2]` / `status[scan][scan2]` pairs **in memory
order** (`[dx][dy]`, index `dx·4 + dy`; `docs/ORDERS.md` §6.5), the
twenty-five corner heights, then `valid` and `farm_type`. The harness read
four of those fields and threw the thirty-two cells away for a month;
`run12_and_run13_s_farm_records_are_the_original_s_cell_for_cell` now
compares the whole record on every frame either `DUMP_ALL` capture of this
game prints one — run12's 1–3 and run13's 95–104 — so the clock and the
sprout's cell are assertions rather than readings. Made to fail twice:
dropping `inc_time`'s add parts frame 1, transposing the sprout's search
parts frame 2.

**The clock.** The farmer's `Farms::grow@008d91c0` adds `0.005f` and sets
state 1; `inc_time` adds another the same frame; and `0.005f` in single
precision reaches `1.0f` on the **201st** add, not the 200th (`farms.rs`'s
pinned crossing). Two adds a frame checked once a frame: `< 1.0f` after 200
adds at frame 99, `≥ 1.0f` after 202 at frame **100** → state 2 → the "new
tile" branch's two draws on **101**. The regrowth is `−0.01f` a frame, `≤ 0`
on the **101st** subtraction.

## 3.4 The docks — `Dock::init@00740a80` (2026-08-25)

Not per frame: per dock finished. `Build::activate` → `Docks::init_dock` →
`Dock::init` spawns the dock's gull (`Objects::init_unit(9, GULLBIRD, x −
0xc0, y − 0xc0)` — one creation draw, `Guy::init_real`) and then draws
`Random::get(game_random, 0, 0xffff)` for its heading (`% 7 × 0x0aaaaaaa −
0x40000000`). **Two draws, creation first**, inside the building's own
`process` in the objects phase (§3.2). Run21, frame 3579: draws 0 and 1
are `Guy::init_real+0x52 < Unit::init < Animal::init` and `Dock::init+0x125
< Docks::init_dock+0x128 < Build::activate+0xcbf`. `docs/TRANSPORT.md`
§5.2; landed in `crates/sim/src/transport.rs` (`dock_open`).

**The dock's own frame is not the end of it, and that was East Indies'
word at 3579** (2026-09-01). Two things stood in the way and both were
this crate's, not the reading's: the heading roll carried **no `mark`**, so
the sequence read the standing label — a second `Guy::init_real+0x52` —
where the original reads `Dock::init+0x125` (queue item 122's shape,
found by the score); and the gull was created with **no `type_index`**, so
nothing recognised it as one of `Guy::set_anim`'s three gaia bird types and
it never flew. A gull is a bird: the frame after its birth
`Unit::do_strafe` runs `Unit::do_air_physics`, whose tail is
`set_anim(CHAR_WALK, 0, 1)` and whose one draw that is — run54's frame
**3580**, `Guy::set_anim+0x104b < Unit::set_anim+0x56 <
Unit::do_air_physics+0x683` — and after that it is on the wing beat like
any other (§3.9). `docs/TRANSPORT.md` §5.2.1 has the whole of it, including
the four `set_anim` and `do_air_physics` arms a gull does *not* take. The
word went 3579 → **3608**, where `SpellType::cast_transport` first fires —
the dock's own shadow, since the level it granted at 3579 is what lets a
unit board (`docs/TRANSPORT.md` §6).

## 3.5 The armies — `Army::find_target@006f69b0`, `Unit::come_out@00617c10` (2026-08-25)

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

## 3.6 The pasture and its five animals — `Farms::add_animals@008d8f30` (2026-08-26)

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
if (o · (slot + 1) + game->frame) % 128 != 0: return           # its phase
target = num_gatherers(farm, 1, 0) == 0 ? farm                 # arrived only
                                        : farm->gather_down    # BuildData+0x70
if not farm->covers_tile(target.tile):        return
ONE DRAW: dir = rand & 7                                       # then a move
```

**The count is `num_gatherers(this, 1, 0)`, and the `1` is the whole of
it** (2026-09-03, item 196). That first argument is `is_gathering_at`'s
third — `arrived` — so the count is chain members whose *action* is a
`GATHER` on this farm **with `been_there` set**, decoys included
(`docs/ORDERS.md` §6.1). A citizen joins the chain the moment
`add_gather_order` issues it and sets `been_there` only on arrival, so
through the whole walk out the count is zero and the measured object is
**the farm**, which covers its own tile — the draw is spent. Read as the
chain's *length* instead, a citizen still five tiles away becomes the
measured object, `covers_tile` fails, and the draw is dropped: that was
Great Lakes' word at run53's **2930**, and with the arrived count in it
runs to **4241**. run69's 3,000 frames went from six units ever off the
original's point to **none**.

Note the asymmetry, which is in the listing and not a simplification: the
*count* is filtered by arrival, and the object it then reads is
`gather_down` — the chain's **head, unfiltered**. Whoever is at the front
of the chain is measured, arrived or not; the arrived count only decides
*whether* the head is consulted at all.

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

**And owner 9 is the reason a *score* hid for a week.** Because no dump
prints the five, the only place a run can get them is the trace —
`Trace::add_animals`, through `diff::borrow_pasture` — and until 2026-08-31
that call sat in the word's own check and nowhere else. `run_traced`, which
is what every tick-and-order score is measured on, borrowed the siblings'
initial state and no pasture: so on a map whose AI builds one, **the run
that scores and the run that matches the word were not the same
simulation**. The five roll an idle a frame, the stream parts within a few
frames of the pasture finishing, and every position after that is a
different game's arithmetic. East Indies read 167/167 for two days for that
reason and nothing else; with the trace passed in it reads **1374/1373** —
its own word's parting. `run_traced` now takes the trace as a source beside
the siblings, and the second map's score asserts the pasture is there
(item 69).

Landed: `crates/sim/src/farms.rs` (`Farm::farm_type`, `Sim::farm_add_animals`,
`Sim::think_farm_animal`), `Sim::build_covers_tile`, and
`gamelog::Initial::farms` — the `Farms` list read off the dump, which has no
block of its own (`Farms::log_data` writes `who`, `o`, the cells, the corner
heights and then `valid`, `farm_type` as flat fields of the enclosing dump).

~~**What this leaves open.** The animals' **positions**, and with them where
`think_farm_animal` walks them … So is the coin that picks chicken or pig,
and so are their **animation lengths**.~~ — **all four are closed by §3.11
(2026-08-30)**: the coin, the two offsets and the species come off the
capture's own trace, the `type_index` that follows from the species is what
reaches the install's gaia table, and the walk is issued. ~~None of that
touches the frame-0 count; all of it touches a long run.~~ — it was the
whole of the second map's word from frame 19 on, and what is left there is
movement's and animation's, not this mechanic's (§3.11's last section).

## 3.7 The idle scout — `Unit::think_scout@005f6010` (2026-08-26)

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

## 3.8 The new farm — `Farms::add@008d8a40` (2026-08-26)

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
| `get_nearest_farm_type` finds a **started** farm within `0x480` | its `farm_type & 1`; wheat goes on to the ambience | none |
| `others == crops == 3` | pasture | none |
| otherwise | `rand & 3 == 3` → pasture, else wheat | **one**, `+0x128` |

`FarmsData::get_nearest_farm_type@008d73a0` is
`ObjectsData::find_any_building` over **every** owner, `FILTER_TYPE 0x1a1`,
`FILTER_NOT_ME`, radius `0x480` (six tiles) — so farms cluster by type
across a border as readily as inside one, and only the *nearest* answers.
**And only a started one**: the search passes `find_who = −1`, so it keeps
a building only when its vtable `+0x50`, `WallData::is_started@00472360`
(`flags & 2`), answers. A site no citizen has reached is invisible to it
(item 919, `docs/AI.md` §79).

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
  ~~and the simulation stands its animals up only for a farm read off a dump
  (`Sim::farm_add_animals` has no caller in `Sim::activate`). No capture
  contains one.~~ **run33 contains one — its frame 1372, which is where
  Great Lakes' word had been parting — and `Sim::activate` makes the call
  since 2026-08-31 (§3.21).**

## 3.9 The bird — `Animal::think_bird@005d79e0`, `Unit::do_air_patrol@005ea620` (2026-08-28)

A bird is what `Objects::process_all`'s sampling creates: `init_unit(who 9,
BASE_GAIATYPES)` at the hit cell's centre (`cell·0x300 + 0x180` on each
axis) and `add_air_patrol_order` on the same point. The console's `bird`
makes the same pair at its raw cursor, so both go through
`Sim::spawn_bird_at` (item 652, `docs/GOLDEN.md` §10). From that frame it is in
the unit loop's who-9 slot, and `do_job` runs `do_air_patrol`: the `+0x180`
virtual (`think_bird`), then `Unit::do_air_physics`.

**No dump prints owner 9 at all** (run20's first `FULL DUMP` has 104
`ANIMALDATA` records and not one). The draw-site trace is the bird's only
behavioural oracle, and every count below is run14's.

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

**Three draws every eighth frame per live bird, and sixty more on the
frame it lands.** The landing search cannot fire until the counter has
passed 100 — the counter is the modulus — which is ~90 frames of flight,
so the short captures never reach it. run39 does, on frame **576**.
`0x194` returns after an `order_type` call and draws nothing.

#### The landing search — `think_bird+0x2aa`, `+0x2d3` (2026-08-31)

```
spell_time = 0
best = the patrol point's own cell;  r = the region that cell is in
thirty rounds:
    n = r.size                                       # Region+0x14
    i = n <= 1 ? 0 : rnd % n                         # +0x2aa
    (x, y) = r.tiles[i]                              # Region+0x7c, 8 bytes a cell
    score = rnd % 0x32 + 1                           # +0x2d3
    if cell(x, y).flags & 0x20 (FOREST):   score *= 3
    if cell(x, y).flags & 0x10 (MOUNTAIN): score *= 2
    if score > −1:  best = (x, y)                    # always
the patrol point = best · 0x300 + 0x180 on each axis
```

**The score decides nothing.** The "is this one better" test the loop
carries is `-1 < score`, and a `% 0x32 + 1` product of 1..300 can never
fail it, so `best` is simply the **thirtieth** cell sampled and the two
terrain multipliers are dead arithmetic. Only the sixty draws are
observable — no dump prints owner 9 (above) — and they are what the word
sees: run39's frame 576 spends 118 draws to this simulation's 56 without
them, and 118 to 118 with them
(`rondata::diff`, `run39_s_bird_lands_on_576_and_spends_the_search_s_sixty`).

**What is not established.** Which cell the search settles on, and
therefore where the bird then flies, rests on the reading alone: the score
being inert makes the answer "the thirtieth sample", which no capture can
confirm because nothing dumps a bird. A region of one cell would spend
thirty draws rather than sixty; run39's does not, so the `n <= 1` skip is
reading-only too.

#### The counter's extra step — `do_air_patrol`'s tail (2026-08-31)

`Unit::do_air_patrol` is `think_bird`, then `do_air_physics`, and then one
branch this section had not read:

```text
if (do_air_physics(...) != 0) {
    if (vtable+0x30 () == 0)   … the military plane's target search …
    else if (spell_time == 0)  spell_time = 1
}
```

`vtable+0x30` is `SubObjectData::is_animal` (§3.14; the map folds all
three `Animal` vtables' slot onto `Buffer::is_pending_load@0041e0e0`,
`return 1`), so a bird always takes the second arm; and
`do_air_physics` returns 1 on every path a bird carrying its single
air-patrol order takes through it (the three `return 0`s are `check_fuel`,
behind the same `vtable+0x30` test; `land_plane`, behind an order field
only the `0x193` arm sets; and a `kill_current_order` behind
`1 < UnitData+0xd8`, the queued-order count). `think_bird` steps the
counter on **every** frame it runs, so `spell_time` is 0 there only on a
frame the landing search has just zeroed it: **the branch is one extra
step per landing, and nothing else.**

One step, and it is the whole of East Indies' word at 1256. run39's second
bird had landed on 944; 311 frames and 38 think-frames later its counter
reads 349, the two steps inside `think_bird` make the modulus 351, and
`27127 % 351` is exactly **100** — the single value in the counter's range
the `== 100` arm tests for. With the extra step the modulus is 352, the
remainder 23, and the bird flies on as the original's does.

**The landing frames are the assertion.** Nothing dumps owner 9, so the
counter itself is unobservable — but the frame a search fires on is not:
thirty `+0x2aa` draws at a time, and run39's frame 944 spends sixty
because two birds land on it. Up to the word this simulation lands on 576,
944, 944, 1016, 1144 and **1368**, which is the trace's own list
(`rondata::diff`, `a_bird_s_landing_frames_are_the_trace_s_own`); without
the branch it lands a seventh time, on 1256, where the original does not.

| what | run14 |
|---|---|
| hatchings (`Guy::init_real` under `Animal::init`) | frames **96, 192, 256** |
| the sampling's pairs | 10 at 0/32/64/96, **9** at 128/160/192, **8** at 224/256 |
| `think_bird` | **3** a frame at 104…192, **6** at 200…256, **9** at 264…280 |

The sampling's count is read off the unit list rather than kept: it is
`min(10, xs·ys/100)` less the birds alive *at that moment*, and a bird born
inside the loop is not yet counted by it.

#### The wing beat — `Guy::set_anim+0x104b`

`set_anim` on a guy of owner 9 and type `0x192`–`0x194` whose *category* is
`CHAR_WALK` draws once and takes the **second** walk animation when
`rnd % 100 > 0x31` (`set_anim:620`). Run14 spends it 28 times: 25 as
phase-7 wraps under `Guy::inc_time+0x271` and 3 under
`Unit::do_air_physics+0x683`. Four rules place all of them.

- **The two lengths.** `unit_graphics.xml` names exactly two animations for
  `WILDBIRD` — `CHAR_WALK` is *Bird Soar* and `CHAR_JOG` is *Bird Flap* —
  and the `.bha` files make them **31 frames and 23** (`docs/FORMATS.md`,
  "The animation file"; `rondata::artdata`). Every idle slot is absent, so
  `AnimationPacket::get_game_frames` gives them its fallback of **3**.
- **The coin re-throws.** `set_anim` dispatches on `UnitAnimCat[anim]` and
  the walk arm opens with that *category* as its answer (`set_anim:613`),
  so the slot the caller named never reaches it: a wrap's
  `set_anim(CHAR_JOG)` starts again from `CHAR_WALK`. A flip keeps
  `cur_time` (the rescale is `t/t`), and 31 still overruns 23, so one wrap
  can spend several coins — two at frame 127, nine at 243.
- **The hatch frame wraps.** `Guy::init_real` leaves `end_time` at **zero**
  and a bird is created on open ground mid-`Objects::process_all`, so the
  same frame's `Objects::inc_time` reaches it and overflows it at once.
  That is frame 96's twenty-second draw. The "created this frame" skip in
  the clock only ever meant *created inside a building*.
- **The birth coin.** `do_air_physics` ends in `set_anim(CHAR_WALK, 0, 1)`
  on every frame (`field_0xae` gates it and is only ever set for
  `FLOCKBIRD`); it draws once per bird, at birth, because from the frame
  after the guy is walk-category and inside its length and
  `set_anim:169`'s gaia early return takes it.

**A dock's gull is the third type on the same coin** (2026-09-01).
`set_anim:620` names `0x192`, `0x193` and `0x194`, so a `GULLBIRD` beats
its wings exactly as a `WILDBIRD` does — and reaches `do_air_physics` the
same way, by a different order: `Unit::do_strafe` rather than
`Unit::do_air_patrol`. Everything in this section that is about *the bird*
rather than about `think_bird` applies to it. run54's birth coin is frame
3580; the chain `Guy::set_anim+0x104b < Unit::set_anim+0x56 <
Unit::do_air_physics+0x683` occurs **12 times in 24,000 frames** — ten wild
birds, the gull, and one at 15458 — which is the count that says the coin
is a birth and not a per-frame draw. §3.4 and `docs/TRANSPORT.md` §5.2.1.

Run14's coin frames: 97, 127 (×2), 142, 150 (×3), 181, 193, 212, 223 (×2),
238, 243 (×9), 257, 261, 274 (×2), 279, 284. The first four are reproduced
exactly (96 + 31 = 127, then the flip's 15 to 142); from 143 the stream has
drifted for other reasons (§3.1) and the later coins are its own.

**What the simulation has**: the hatching, the live count, `think_bird`
whole, the landing search and the counter's extra step after it, the
patrol point as state, the wing beat, the flight itself (2026-09-02, "The
flight", "The birth") and the figure's own arrival stand (2026-09-07,
below). `is_air` is the loaded domain, so a bird does not paint the
occupancy grid a citizen walks on (`docs/COLLISION.md` §2).
~~`Guy::move`'s arrival half is skipped for a bird, there being no ground
body to follow~~ — **wrong, and it was Great Lakes' word**: §3.27. `think_bird`'s own draw (`+0x3b`, `rnd % 7 + 0xd`) sits
behind a vtable test the wild bird fails on every traced frame — run14
confirms it: three `think_bird` draws an eighth frame per bird and no
fourth.

#### The flight — `Unit::do_air_physics@005e86d0` (2026-09-02)

A wild bird takes a narrow path through a function written for the game's
aircraft. Everything below is `crates/sim/src/air.rs`; the addresses are the
whole of the argument, because **nothing dumps owner 9 and the flight has
exactly one observable** — the coin at `+0x639`.

```text
speed  = AnimalData::get_speed(dx, dy, 1)      # air animal: UnitData::speed,
         × ai_speed when that exceeds one      #   which for domain 2 is MOVES
goal   = the air order's one waypoint, clamped into [0, xs·0x300)
des    = find_angle(goal − pos)
         owed more than 45° and vector_dist < min_range·0xc0 + 0x300
             → des = the heading it has                       # fly past
         AirOrder::sharp_turn ≠ 0 → des = heading + turn·0x40000000
bank_aircraft(des, &speed, 0)                  # below
Guy::set_angle(heading, 1)                     # the figure snaps
(nx, ny) = (x + sin(heading)·speed, y − cos(heading)·speed)
UnitData::invalid_loc(tile of nx, ny, 0,0,0,0,0)
    valid   → sharp_turn = 0
    invalid → WorldData::restrict clamps (nx, ny), and when sharp_turn is 0:
                  rnd = get(0, 0xffff)                        # +0x639
                  sharp_turn = (rnd & 0x80000001) ? +1 : −1
set_new_location(nx, ny, 0, 1)
```

**`invalid_loc` on an air type is the world's rectangle and nothing else.**
Its domain arm — `type +0x218 == 2` — returns valid before every terrain
test, and `do_air_physics` passes zero for all five flags that would reach
the rest. So the only step a bird is refused is one that leaves the map, and
the coin is thrown once per departure rather than once per frame out:
`sharp_turn` stands until a step lands inside again, which is why run53's
coins come in clusters a hundred frames apart. The field's name is the type
record's (`AirOrder +0x10`, beside `cruising_alt` at `+0xc` and `returning`
at `+0x18`), and `returning` being zero for a patrol is what excuses
`land_plane`, the landing approach and the bank's doubling arm.

`Unit::bank_aircraft@005e9520`, with `roll` the negation of `GuyData +0x44`:

```text
roll = 0, spell_time ≥ 0 and frame & 7 == 0 → return, before the store
d      = des − heading;  a = d > 0x80000000 ? ~d : d
sign   = d > 0x80000000 ? −1 : +1                    # the caller's, when it has one
rate   = (TURN_SPEED >> 8) · UNIT_TURN_SPEED, floored at 0x5b05b0
want   = (float)(unsigned)a / (float)rate × 55 × 0.33 ; halved when under 5
         ; then min(want, 55)                        # owner ≥ 8 skips the rest
step   = sign·want − roll
    |step| ≥ 2 → roll moves toward want by min(|step|, 10), inside ±55
    |step| < 2 and |roll| < 2 → roll = 0
roll > 0 → heading += air_turn_speed(+1)
roll < 0 → heading −= air_turn_speed(−1)
roll = 0 → heading = des, but only when a < 5°
guy.roll = −roll
```

and `Unit::air_turn_speed@005ea390` is `(rate / 55) · |trunc(guy.roll)|`,
floored at `0x5b05b0` and **read before this frame's bank is stored**, so a
frame that reverses the bank turns at the floor. Half a degree is that floor
and the `WILDBIRD`'s own `TURN_SPEED 5` is the ceiling.

**The bank is the state and the heading is downstream of it**, which is the
finding: a bird cannot turn until it has banked into the turn, and the bank
takes six frames to reach 55. So a bird overshoots its patrol point, flies
straight past it while more than 45° is owed inside `0x300`, releases at
`0x300`, and comes round on a radius of about 400 position units. That is
the orbit, and it is a limit cycle — perturbing a bird's initial heading by
one unit leaves its position 1,700 frames later unchanged to the unit.

**The arithmetic is single-precision, and is done in integers.**
`crates/sim/src/single.rs` reproduces `addss`/`subss`/`mulss`/`divss`,
`cvtdq2ps`, `cvtdq2pd`+`cvtpd2ps` and `cvttss2si` exactly, each checked
against the host's own float over random bit patterns. The constants are
`0xb69680` = 55, `0xb69490` = 0.33, `0xb694c0` = 0.5, `0xb695c0` = 2,
`0xb69628` = 10, `0xb697c4` = −55, read from the PE.

#### The birth — `Unit::init@00612100` (2026-09-02, run61)

**The flight was never the residue; the birth was.** run61 proxies
`do_air_physics`, `air_turn_speed` and `set_new_location` over all 5,400
frames of run54's game and folds them per bird per frame — goal, turn rate,
landing position (`docs/RUNS.md`, "run61"). Seeded with nothing but the
birth state and fed the original's own goal, `air.rs` reproduces **every
one of the ten wild birds exactly, to the last frame** — 45,712 air frames,
every position and every one of the bank's zero-crossings, runs up to 5,272
long (`diff::tests::run61_s_birds_fly_where_the_original_s_do`). The arms,
the constants, the bank and the coin were all right.

What was wrong was twenty-four position units, in the constructor.
`Unit::init@00612100`'s first two lines snap the requested position onto the
centre of its 48-unit tile — `div_3_table[p >> 4] · 0x30 + 0x18`, i.e.
`(p / 48) · 48 + 24` — and *that* is what `Object::init` is handed, while
`add_air_patrol_order` keeps the unsnapped point. So a bird asked for at a
cell centre is put down **twenty-four units into its tile** on each axis;
run61 has all ten born at exactly `goal + (24, 24)`, and `Gaia::spawn_bird`
had handed the cell centre through to both.

The same constructor writes `UnitData::angle = 0x55555555` — a third of a
turn, `movement::Angle::INITIAL` — and a bird is the one unit whose first
frames never overwrite it, so the fill pattern *is* its initial heading.
The dock's gull is the exception that shows it: born on a `StrafeOrder` at
3579, `Unit::do_strafe` points it due west first.

With the snap the flight is called from `Sim::do_idle` and both long words
move: East Indies **5437 → 5466**, Great Lakes **1802 → 2419**.

**Still not established**: the landing search's *outcome* (the sixty draws
agree, no capture puts the chosen cell beside the original's — a `callwin`
over `think_bird`'s tail would), and `Unit::do_strafe`, so the gull is
unmodelled.

## 3.10 The residue's other names (2026-08-28)

Five draws this simulation was already taking, each of them under a coarse
phase mark (`strategy_all`, `unit w/o`) rather than a site of its own. A
frame counts on the ledger only when the two label sequences are *equal*
(§5.1), so a coarse mark fails a frame the counts agree on; naming these
changed no mechanic and moved run14 **192 → 198 of 284**.

| the original's site | what draws | our label | `SITES` `via` |
|---|---|---|---|
| `MathUtilFuncSet::rand_int@009e1890+0x18` | the script VM's only draw | `ai_host::SITE_RAND_INT` | `ScriptFuncSet::call_func+0x401` |
| `Animal::do_idle@005d7460+0x83` | a herd animal's wander coin | `gaia::SITE_WANDER_ROLL` | — |
| `Animal::do_idle+0x1a4`, `+0x1d4`, `+0x212` | its direction and two step counts | `gaia::SITE_WANDER_{DIR,X,Y}` | — |
| `GameAccess::rnd@0043cca0+0x20` | the farmer's cell re-pick, **two** draws | `orders::SITE_FARM_CELL` | `Unit::do_job+0x67` |
| `Guy::set_anim+0x97a` | the arrival stand | `anim::SITE_ARRIVE` | `Guy::move+0x19f` |
| `Guy::set_anim+0x97a` | the blocked stand | `anim::SITE_BLOCKED` | `Unit::move_step+0x823` |
| `Guy::set_anim+0x97a` | the **snap** arm's blocked stand | `anim::SITE_SNAP_BLOCKED` | `Unit::move_step+0x4e2` |

**`GameAccess::rnd` is frameless, and its address alone names nothing** —
it is the helper `Random::get(0, 0xffff) % ecx`, whose call returns to
`+0x20` for every caller in the executable, and the `ebp` walk skips it
*and* `Unit::do_gather`, so the chain reads `< Unit::do_job+0x67`. A `via`
is mandatory here and needless for the four unique `Animal::do_idle`
addresses. Both draws of the pair share the address, so one mark carries
them (`docs/ORDERS.md` §6.5).

**`Guy::move+0x19f` sits one frame higher than the other `+0x97a` chains**,
being `Guy::move`'s own call rather than `Unit::set_anim+0x56`'s; the
disambiguator is at `up[0]`, and `Trace::label` tests both slots. It is the
arrival stand (`docs/ANIM.md` §4). **`Unit::move_step+0x823` is named from
the original's side only**: the stand a blocked unit plays before the three
give-up tests (`docs/COLLISION.md` §5, §7), which this simulation does not
take. **And `+0x4e2` is its sibling, added 2026-09-18 (item 360)**:
`move_step` has a collision block on *each* side of its `param_2 <
local_28` split, and this crate spent `+0x823` for both. Great Lakes 9134
was one draw on each side against a bare `5dac7a`, which is a comparison
that cannot fail (`docs/COLLISION.md` §5.4, §8.9). Making the call **costs** both scores as things stand — 43340 → 42755
agreeing unit-frames, 198 → 196 traced frames — because these collisions do
not yet fall on the original's frames. The row stays regardless: without it
frames 122, 184 and 256 read as a bare `5dac7a`.

~~**One site is left unnamed on run14**~~ — **named 2026-08-28,
`docs/ROADS.md`**: `PathFinder::calc_road_cost+0x46`, 657 draws on frames
10, 11 and 171, one per node a **building's** road to its city centre
costs, `sim::roads::SITE_COST`. The block desynchronises the stream from
frame 10 on and with it every value-driven label after, so the remaining
eighty-six frames are mostly not naming faults. The site is taken under
`Sim::plan_roads`, **off** while the search's count is short.

## 3.11 The pasture is East Indies' word, at 19 (2026-08-30)

§3.6 left three things open and a long run has now priced all three.
`crates/rondata`'s `run39_s_long_trace_says_where_the_second_map_s_word_parts`
reads `rontrace-run39.log` against the harness frame for frame: **the second
map's word parts at 19**, 148 frames before the order-list divergence at 168
that had been called its first, and all three of the open items are in it.
East Indies' AI starts with a pasture and Great Lakes has none, which is why
run33's word never saw any of this.

**The coin is settled, and a pasture is one species.** The four draws an
animal are a fixed stride, so every coin of a call lands on the same parity
of the stream; and `Random::get(0, 0xffff)` returns `((seed & 0xffff) ·
0xffff) >> 16`, whose low bit is the complement of the seed's, which the LCG
flips on every step. **Five even coins or five odd ones, never a mix.**
Which of the two is still a setup draw — but the trace carries the seed
*before* each step, so a capture's own coins are readable: run39's five, at
`add_animals+0x92`, are even. **Chickens**, whose `CHAR_DEFAULT` is 30
frames where a pig's is 90; both name `CHAR_IDLE1..3` at 90.

**The lengths were missing twice over.** A pasture animal here carries no
`type_index`, so `Sim::slot_length` cannot reach the gaia table at all; and
the table itself had no row, because `rondata::artdata`'s `GAIA_UNITS` left
`FARMPIG` and `FARMCHICKEN` out of its twelve. The second is fixed
(2026-08-30) and is inert until the first is. Together they are **three of
the original's six `Guy::inc_time` wraps on run39's frame 29**, and they
take the early window from 49 frames on the count to 60 and 47 draw-for-draw
to 53 — while **costing the ticks score 167 → 102**, because the stream
after 19 is nobody's either way and 167 was luck on it. So they wait for the
walk rather than landing alone.

**The walk is the fourth animal's nineteen frames.** The one whose phase
hits frame 0 — `o` 0 of 0–4, slot 0, and the trace's own phases
`{0, 108, 116, 122, 126}` are `(o·(slot+1)) % 128` for exactly that
assignment — is handed a `MOVE_TO` this crate does not add. It walks, and
its **arrival spends two `Animal::do_idle` set_anim draws on consecutive
frames**, 19 and 20 (both roll `≤ 69`, so both take `CHAR_DEFAULT`), after
which its clock is nineteen frames behind the other four and wraps at 49
rather than 29. The signature repeats all game: every `think_farm_animal`
draw is followed nine to twenty-five frames later by such a pair.

**Where it walks to, in full.** `think_farm_animal`'s destination needs
nothing but the one draw it already spends. Let `T` be the tile of the
reference object (the farm, or `gather_down`'s first gatherer) plus
`move_x`/`move_y[dir + 1]`, and `C` the slot's corner; then the axis is

```
idx  = div_3_table[((T · 3 + C) · 0x40 + 0x60) >> 4]   =  4·T + (4·C + 6)/3
dest = idx · 0x30 + 0x18                               =  192·T + {24, 120, 168}
```

for `C` of `−1`, `0`, `+1` — the low edge, the middle and the high side of
a tile, which is 192 position units wide. **`corner_y@00adc3c0` and
`corner_x@00adc3e0`** (`compass.obj`, `const int[]`; read out of the PE
against `rise_z.map`) are

| slot | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| `corner_x` | 0 | −1 | +1 | +1 | −1 |
| `corner_y` | 0 | −1 | −1 | +1 | +1 |

— a centre and its four corners, one per animal, which is what
`Animal+0x154` is for.

**The positions are borrowed, not derived.** They are the last thing the
harness cannot produce: `Farms::add_animals` spends them at **`+0x92`**
(the coin), **`+0x134`** (`y`) and **`+0x182`** (`x`), under
`Build::activate+0x1c25 < Leader::produce_building+0x1a10` — which is
§3.8's correction confirmed by the chain rather than by reading — and the
whole call sits inside `Setup::build_empire`. run39's five are `(−143,
40)`, `(−187, −148)`, `(−39, −144)`, `(−83, −76)`, `(−63, 56)` as
`(dy, dx)`. **A draw record carries the seed before the step, so its
outcome is recoverable without the game**, and both readers now do that
arithmetic (`tools/trace/report.py <log> draws setup`;
`rondata::trace::Trace::add_animals`, and `diff::borrow_pasture` puts them
on `Initial` — the one field of it no dump fills). run20's trace and the
fuzzed map's carry their own fifteen, so this is not one capture's trick.

### The pasture, landed (2026-08-30)

`crates/sim/src/farms.rs` now creates each animal with its species and so
with a **`type_index`** — which is the whole of why the clock never wrapped,
since `Sim::slot_length` keys the install's gaia table by it — at its
borrowed position, with `Objects::init_unit`'s speed and turn rate, and
`Sim::think_farm_animal` issues the `MOVE_TO` above after
`close_orders`/`clear_partial_path`/`update_action`, at
`add_move_facing_order`'s snap of the unsnapped point with the angle taken
to that point rather than to the snapped one. run39's early window goes
**49/47 → 62/55** of its first 64 frames; ticks and orders hold at 167.

### The pair, read (2026-08-30)

The word parted at **19**, on the *first* of the arrival's two draws, and the
three things that stood between are now all read out of the original. **Two
of them are unlanded**, together, because the third of them costs the other
map more than the pair is worth; the numbers below are what they are worth,
measured.

**1. The arrival is a frame early, and the reason is where the animal is
born.** `Unit::init@00612100:69` **snaps every unit's starting position** —
`div_3_table[v >> 4] · 0x30 + 0x18`, the same 48-unit snap a move order's
destination takes — before it hands the pair to `Object::init` and to
`set_new_location`. `Farms::add_animals` computes `building ± (rnd % 0x180 −
0xc0)` and `Objects::init_unit` passes that straight through, so run39's
first animal is born at `(40536, 40392)` and not at the `(40552, 40369)` the
two draws name. That is **432 units of `y` at 25 a frame, not 455** —
eighteen steps rather than nineteen. Nothing else moves: `myspeed` is
`MOVES × unit_move_speed` = 25 (`Unit::update_speed@006055c0`, whose four
multipliers are all `is(ARQUEBUSIERS|RIFLEMAN|INFANTRY|MECHINFANTRY)`), and
the arrival tolerance really is zero. **A pasture animal is the only object
this crate places from a raw, unrounded point**; every other one comes from a
dump, which prints where an object *is* rather than where it was born, so the
snap belongs at the pasture and not in `Sim::add_unit`.

**2. The second draw is `Guy::move`'s turn arm**, exactly as item 93
predicted. Standing on its unit with `des_angle != angle`, the guy is put
**back on `CHAR_WALK`** and marked unstopped (`Guy::move:73–89`,
`docs/MOVEMENT.md`), so the next frame's `Animal::do_idle` sees the walk
category again and rolls again. The guard the arm carries is a **sea** unit
(`type+0x218 == 1`) or a `SPECIAL_ANIM` order, neither of which is a chicken.
The `+0x9e`/`+0xa0` stashes an earlier draft named here are `hold_attack` and
`queued_attack` and belong to `set_anim`'s **attack** category, not to this.

**3. And the turn is one frame, not two, because the standing body turns
instantly.** `Guy::move` writes `last_speed = 0` at the head of the at-des
branch, *ahead* of the `turn_towards` at its foot, and `GuyData::turn_speed`
answers a zero `last_speed` on a foot or mounted guy with `0x80000000`. The
residual here is 8.8° against a chicken's 5° a frame; with the zero it is one
frame, so the pair is 19/20 and not 19/20/21.

**(3) landed first, alone.** It holds every score and closed queue item 37 —
run10's AI scout no longer turns a frame late at 96, 362 or 721, and the angle
disagreements went 9,156 of 33,992 → **8,969 of 35,868**.

**(1) and (2) landed together, once §3.12 had put the farmers' angles right.**
Either alone is worse than neither — the snap alone moves the word to 20 and
the window to 60/53, the arm alone to 58/57 — and until the other map's
facings were the original's, both together cost run33's word 780 → 584,
because the arm reads `des_angle != angle` on *every* standing unit. With
§3.12 in first they cost nothing: run39's early window is **64 of 64 on the
count and 64 draw for draw**, the word goes 19 → **69**, and Great Lakes holds
at 780 and improves to 954/843. `crates/sim/src/farms.rs` snaps the animal at
birth; `anim.rs`'s `guys_follow` carries the arm and its sea guard.

**The named suspect for the other map was a dead end, and that is now an
assertion.** `Guy::do_turn@005d97a0:15` overrides the arm's walk with
`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` when `guy_flags & 8`, which
`Guy::init_real@005db6b0:179` sets only for a guy whose piece names a turn
animation. 273 of the install's 1,359 unit pieces do — but **none of the
eight a `DUMP_ALL` run's guys carry**: pieces 0, 19, 352, 371, 6336, 6688,
12691 and 13043, the two scouts, their dogs and the six citizens. Gaia's
60063–60074 are not `<UNIT>` entries at all. So the override cannot fire on
any capture there is, and `rondata::diff`'s
`the_install_s_piece_lengths_match_the_dumps` says so on every commit.

## 3.12 The farmers' angles — two predicates, not seventeen callers (2026-08-30)

Item 36 was booked as "`Unit::set_angle`'s seventeen other callers", the
name the residue had carried since 2026-08-27: 8,866 of run10's 8,969 angle
disagreements were farmers — `0/3`–`0/5`, `1/3`–`1/5`, `1/8` — standing
where the original stands them and pointing somewhere else. It is **not**
seventeen callers. It is two predicates in the code this crate already had,
and both are visible in the same capture on frames six apart.

**1. The order's angle is the bearing to the caller's point, not to the
snap of it.** `Unit::add_move_order@00616ed0` computes `find_angle`'s two
arguments in registers from its own *arguments* — `ecx = x − (this->
field_0x10 ^ 0x63637)`, `edx = y − (field_0x14 ^ 0x63637)`, the obfuscated
position pair — and only then indexes `div_3_table` for the coordinates it
pushes to `add_move_facing_order`. So a caller that hands over an unsnapped
point gets a heading to *that* point and a destination up to 24 units an
axis away from it. `Unit::do_gather@005ef2a0`'s wheat branch is such a
caller: its re-picked cell is `(corner + rnd % 4) · 0xc0 + 0x60`, the centre
of a **192**-unit tile, which is never the centre of a 48-unit cell.

run10's farmer `0/3` walks from `(2712, 32136)` to the cell whose snapped
centre is `(2808, 31992)`; the dump's `MOVEORDER angle` is `0x10890000`
(23.25°), and `find_angle` over the *raw* `(2784, 31968)` gives exactly
that, where the snapped pair gives 33.72°. `0/4`'s `−124.74°` is
`find_angle(−312, 216)` on the same rule. Two farmers, two exact matches,
no free parameters — and this crate had `let dest = snapped(to)` one line
above the `find_angle` it fed. **8,969 → 1,227 of 35,868.**

The pasture already knew this: §3.11's `think_farm_animal` takes "the angle
to that point before the snap", and its note says the walk "cannot go
through `Sim::add_move_order`". It can now; the special case was the bug
report.

**2. The gather clause belongs to the snap arm alone.** `move_step@005faf30`
ends a final leg two ways, and `docs/ORDERS.md` §4.5 has had both since it
was written: the **Manhattan snap** (`manh <= step`) faces the order's angle
when `orderlist.length == 1` **or** the action beneath is a `GATHER`
(`005fb562`); the **partial step** — the unit walked its whole step and
happened to land on the destination — faces it only when the move was the
only order (`005fb4a8`). The code applied the gather clause to both.

The two arms are six frames apart in one capture. `0/3` arrives on 110 with
`manh 7` at speed 25: the snap, a gather beneath, and the heading becomes
the order's 23.25°. `1/3` arrives on 116 from `(41789, 17040)` with
`manh 29` — a full 25-unit step whose sine and cosine are −5 and 24 lands it
exactly — so it is the partial arm, and its heading stays
`find_angle(−5, −24) = −11.42°` while its order's angle is −18.58°. A
farmer's last leg is routinely the second kind, which is why every farmer
was in the residue and nothing else was.

**What it cost, and what it unblocked.** Both together: run10's angle
disagreements 8,969 → **1,227 of 35,868**, the earliest surviving row 110 →
820, and `1/1`, `1/5` and `1/9` out of the residue entirely. Ticks and
orders hold at 572/776; East Indies holds at 19/62/55. That is the shape of
a dependency rather than a score: item 93's arm reads `des_angle != angle`
on every standing unit, and with the facings wrong it cost the other map
196 frames of word. With them right it costs nothing, and §3.11's pair
lands the same session — **East Indies' word 19 → 69, 64 of 64 draw for
draw**, Great Lakes unmoved at 780 and up to 954/843, run10's angle residue
settling at 1,910 of 35,984 on the wider view the pair brings.

**The lesson is item 72's, a fifth time.** `docs/ORDERS.md` §4.5 stated the
two arms correctly and the code merged them; §4.3 said `add_move_order` "is
a thin wrapper that snaps and computes the angle" and never said *what* it
computes it from. Neither was found by reading the mechanic again. Both
were found by taking one row of a residue — a farmer standing still and
facing wrong — and printing the original's own record for the twenty frames
around it.

## 3.13 The animal's hurry — East Indies' word 69 → 91 (2026-08-30)

Item 95 was booked as "East Indies' blocked stand, a frame late": on run39's
frame 69 the original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`,
`sim::anim::SITE_BLOCKED`, and this crate spent it on **70**. It is one draw,
one frame apart, and the whole of it is a **speed**.

**Whose stand it is.** Gaia's `8/2`, blocked by its herd-mate `8/1`. Both
sides start it at `(28776, 24360)` on the same frame with the same wander
goal, `(28968, 23976)`; both stop at exactly `(28856, 24197)`, because the
stop is positional — the next step's point is where `8/1` stands. The
original covers that ground in **nine** steps and this crate took **ten**.

**The steps say the speed outright.** The original's, from the dump:
`(12, −25)`, `(12, −25)`, `(8, −17)`, then `(8, −16)` six times. Ours:
`(8, −17)` three times, then `(8, −16)`. Solve each against `find_angle` to
the goal and the sine table and the answer is a single integer per frame:
the original walks at **28, 28, 19, 18…** and this crate at **19** the whole
way. The two long steps gain exactly one step's worth of ground, which is
the frame.

`19 × 3 / 2 = 28`, and the dump prints `myspeed 19` on every one of the nine
frames — so the `3/2` is applied downstream of the cached speed, not instead
of it. `AnimalData::get_speed@005d8380` is where: an animal more than
`0x180` from its current **move order's goal** walks at `speed * 3 / 2`.
`docs/MOVEMENT.md`, "The animal's own `get_speed`", has the function whole
and what a fresh reading gets wrong about it. `vector_dist` to the goal on
the three frames that matter is **432, 404, 376** — the last is the first
one under `0x181`, and it is the frame the step drops to 19.

**What it cost, and what it did not.** East Indies' word **69 → 91**, the
early window holding at 64 of 64 on the count and 64 draw for draw. Great
Lakes is untouched where it can be seen: `first_count` holds at 780,
`first_part` at 99, run10's ticks and orders at 572/776, and its collision
rows *rise* 93,357 → 93,398. run33's weak totals fall 954/843 → **938/841**,
and it can be said exactly where: with the hurry in and out, the per-frame
draw counts are **identical up to frame 1108** — 328 frames past that
capture's own parting and past every unit's first divergence. run39's player
0 goes 219 → **217** on the same argument: by 217 the two sides have been on
different mid-frame draw orders for a hundred and twenty frames, and which
of three citizens parts first is not a fact about this simulation.

**The lesson is that the residue was one draw and the cause was a table
nobody had looked at.** The queue's own framing — "a blocked stand, a frame
late" — was the *symptom*, and reading `Unit::move_step`'s collision block
again would never have found it, because the collision block is right. What
found it was taking the one row, printing the original's own record for the
twenty frames around it, and solving the steps for a speed: an integer that
changed from 28 to 19 on the frame `z_internal` changed, which looked like
terrain and was not. Item 72, a sixth time — and this one the documents had
*not* got right, which is why it needed the record.

~~**And the same rule is now the next item.** The word parts at **91** on
the same site: gaia `8/0` gets a wander order to `(28968, 23976)`, walks
one step, and is blocked. This crate sends it to `(28776, 24120)` instead —
a point the near branch can reach and the original's cannot, since
`Animal::do_idle`'s near offsets cap at `4 × 0x30` an axis — so the two
took different branches on frame 89.~~ **Wrong on the branch, right on the
symptom.** Both sides take the *far* branch on 89: the herd centre is
`(28800, 23936)` and `8/0` stands 413 away, over the `0x180` the near arm
needs. What differed was what `find_nearby_spot` had to work with — see
§3.14, which took the word to 201.

## 3.14 The blocked animal's dropped walk — 91 → 201 (2026-08-30)

The residue at 91 was one draw and the wrong animal. `8/0`'s wander spot on
frame 89 is `find_nearby_spot`'s first free candidate on the ring of
`0xc0` around the herd centre `(28800, 23936)`, and the original's answer —
`(28968, 23976)` — was free for it and taken for us. What was
standing in it was **`8/2`**, twenty frames after §3.13's blocked stand:
the original leaves it at `(28856, 24197)` for the rest of the capture, and
this crate side-stepped it, snapped it onto its cell centre, and walked it
round to the goal — arriving beside `8/0`'s spot on the very frame `8/0`
needed it.

**`Unit::resolve_unit_collision@005f9d30`'s first statement is
`SubObjectData::is_animal`,** vftable offset 48, and when it answers, the
body is the `QUEUE_NEW` clear and nothing else: `unit_masks &= ~0x4000000`,
`path.length = 0`, `close_orders`, `clear_partial_path`, `update_action`,
return. `docs/COLLISION.md` §6 now opens on it as step 0. An animal takes
**none** of the six steps below it — no sidestep, no wait-for-it, no
repath, and above all no cell-centre snap.

**The name is the whole of the finding, and only the PDB has it.** The
decompiler prints `(**(code **)(*(int *)this + 0x30))()`, and the map names
that slot after a trivial function it was COMDAT-folded with:
`Buffer::is_pending_load` in `Animal::vftable`, `Window::get_button` in
`Unit::vftable`. Those two stubs *are* the predicate — `return 1` against
`return 0` — but nothing in the export says so, which is why a document
that had read §6 six times had no step 0 in it. The PDB's `LF_ONEMETHOD`
list carries `vftable offset = 48` on `SubObjectData::is_animal`, three
slots after `is_wonder` and one before `get_gpiece`. This is the third time
that list has settled a slot the map cannot (`docs/MOVEMENT.md`, "The
animal's own `get_speed`", was the first two).

**What it cost, and what it did not.** East Indies' word **91 → 201**, the
early window holding at 64 of 64 on the count and 64 draw for draw. On
Great Lakes: run33's `first_count` holds at 780 and `first_part` at 99,
run10's ticks and orders at 572/776, and both players' first divergence at
802 and 573. Twelve of run10's thirteen units are unchanged **to the
frame**; the thirteenth, the AI's `1/9`, parts at 1320 rather than 1377,
which is the whole of the fall in the two coverage counts (collision rows
93,398 → 91,210, angle rows 35,942 → 35,188). run33's weak totals go
938/841 → 943/827 with the count *up* and the order down, and the same
argument as §3.13's applies with the same shape: with the rule in and out,
that capture's per-frame draws are **identical, count and sequence both, up
to frame 1128** — 348 frames past its own parting.

**The lesson is the one §3.13 ended on, one layer further in.** The queue
booked this as "`8/0`'s wander branch" and named the near arm; the near arm
was never taken. What found it was printing every unit within 700 of the
contested point for the twenty frames around it and noticing that a *third*
animal was somewhere the original's was not — and then that this crate's
`8/2` had walked a dog-leg no dumped animal has ever walked. The check that
now stands is that shape and not the reading: every animal walk in both
long captures that ends short of its goal — sixteen of them — ends with the
animal's position **unchanged** across the frame the order dies, and this
crate broke that on all sixteen.

## 3.15 The pasture's herder — East Indies' word 201 → 219 (2026-08-30)

The residue at 201 was two draws this crate spends and the original does
not, and the whole of it was a **branch the document had and the code did
not**.

`Unit::do_gather@005ef2a0`'s farm block asks `FarmsData::get_farm_type`
first, and when the answer is 1 — the **pasture** — the function is over
before the cell arithmetic starts (`005efd77`). A herder shows the sow
animation and returns; only on the frames where `(o · 7 + frame + who) %
256` is zero does it draw, and then it draws twice and walks to one of the
farm's *inner* four tiles: `corner + 1 + GameAccess::rnd(size / 2)` an
axis, against the crop's `corner + rnd(size)` over all sixteen.
`docs/ORDERS.md` §6.5 had that arm in its pseudocode from the day it was
written; `orders.rs::do_farm` ran the AI's herder through the crop switch
below it instead.

**What made it a hundred-frame error rather than a cosmetic one is the
clock.** A pasture is the one farm `Farms::inc_time` skips (§3.3), so
nothing under it ripens on the engine's own add. The crop switch's
"new tile" fires when a cell is ripe under a sower, and with the farm's
half of the two adds a frame missing, the herder's own `Farms::grow`
carried its cell to `RIPE_ADDS` alone — on the **two hundredth** frame
instead of the hundredth. So run39's `1/3` re-picked a tile on frame 201,
where the original had spent its two draws on **234**, its first phase
frame: `3 · 7 + 234 + 1 = 256`.

**What it cost, and what it did not.** East Indies' word **201 → 219**,
and every one of the 219 frames is now draw for draw as well as
count for count — the early window's 64 of 64 has become the whole run up
to the parting. What is at 219 is not a farm at all: the original spends
that citizen's re-target *before* the frame's road search and this crate
after it, and the two searches cost 129 nodes against 152 — the road
residue of §6 and queue item 60, arriving as the next thing in the way.
Great Lakes is untouched to the byte, because run33's AI built seven
farms and no pasture: `first_count` 780, `first_part` 99, run10's ticks
and orders 572/776, its thirteen units' divergences unchanged, and both
coverage counts (91,210 collision rows, 35,188 angle rows) equal. East
Indies' own game score holds at 167/167 with player 0 at 217 and player 1
at 168.

**The lesson is the one item 72 keeps paying for, with the sign
reversed.** Five times now a document and its code have disagreed and the
document was the one that was wrong; this is the sixth and the first where
the *document was right* and had been for weeks. Nothing read it against
the code. The check that now stands is not the reading but the record:
run39's dump moves the herder on 42 frames of 1,850, in four runs, and
every run opens the frame after a phase frame — and all seven of the
capture's phase frames spend the pair in the trace.

## 3.16 The frame is two loops — East Indies' word 219 → 274 (2026-08-30)

`Objects::process_all@0065dce0` is **two loops, and the buildings are the
second one**. The first walks the ten owner slots rotated — `(frame + i) %
10`, each owner's objects `0..unit_mark` in object order. The second walks
the ten leaders **unrotated**, each one's objects `2000..build_mark` (the
buildings) and then `3000..wall_mark` (the walls). Both call the same
vtable slot. `Objects::inc_time` — the guys' clocks, the farms — follows
both, and so do the birds' sampling and the herd's walk, which sit in the
tail of `process_all` itself.

`Sim::tick` ran the buildings **first**, and §3.2 has said the opposite
since the day it was written. That is the seventh time a document and its
code have disagreed (queue item 72) and the second running where the
document was the one that was right.

**What it cost was two frames' worth of order and one whole search.** East
Indies' frame 219 spends a citizen's `Unit::do_job+0x67` re-target and
*then* the frame's road costs; this crate spent them the other way round.
And the road search that looked **152 nodes against the original's 129**
was the same search reading a world the frame's units had not yet touched:
with the loops in their own order it costs **129**, and the frame is draw
for draw. The count was never the residue — the order was.

**Two rules came with it, and each was a compensation for the wrong
order.**

- **A unit created this frame is not skipped by `Objects::inc_time`** —
  whoever owns it. `Sim::guys_inc_time` skipped a player's newborn
  (`born == frame`), which was the only way to keep a trained citizen from
  spending *two* draws on its birth frame when the crate created it before
  the unit loop. With the loops right the citizen is created after every
  unit, is never reached by that frame's unit loop, and it is
  `Objects::inc_time` that finds its `Guy::init_real` clock at
  `cur_time 0, end_time 0` and wraps it. Run33's frame 99 is the record:
  `Guy::init_real+0x52` then `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
  where this crate had the second draw at `Unit::do_idle+0x7d` — the
  "standing swap" §6 has carried since the first trace, which was never an
  attribution question at all. And run13's `1/6` ends that frame at
  `0/232, last −1`, which is the state the wrap's `set_anim` leaves and
  not the one `Guy::init_real` does.
- **`think_peasant`'s idle threshold is 1 for an AI-driven worker.**
  `Unit::think_peasant@005f5760:16` reads the owner's idle-citizen option
  — the switch 1→7, 2→12, 3→17, 4→32, 5→62, default **2** — only when
  `unit_masks & 0x40000` is *clear*, and takes **1** when it is set. So a
  computer player's citizen finds a job on the first frame it is idle and
  a human's on the second. This crate used the option for both, and the
  extra unit-loop visit the wrong loop order gave a newborn was exactly
  the frame that hid it: run33's citizen came out on 99, was first visited
  on 100, took its gather order on 101 instead of 100, and the collision
  that ends its walk landed on 123 where the original has 122.
  `docs/ORDERS.md` §5.9.

**What it moved.** East Indies' word **219 → 274**, sequence with it.
Great Lakes' word holds at 780 and its **sequence runs 99 → 576** — the
first 576 frames of the long capture are now the original's draws in the
original's order — with the weak totals 943/827 → **943/830**. Run14's
whole traced window is **284 of 284, draw for draw**, where it had been
282 since item 66. run10's ticks and orders hold at 572/776 with both
players' first divergence at 802 and 573, and East Indies' own game score
holds at 167/167.

**Made to fail three ways**, each of them a real failure seen on the way
in: the loops swapped with the newborn still skipped took Great Lakes'
word 780 → **99** and run10's ticks 572 → **103**; the newborn's wrap
restored but the AI's threshold still 2 took it to **122**; and the
threshold alone, without the loops, is what run14's frames 99 and 100 had
always been.

~~What is at 274 is not either of these: the original creates a guy there —
`Guy::init_real+0x52` and its wrap — and this crate creates none.~~
**Answered 2026-08-30, `docs/AI.md` §17**: nothing was queued to create.
A script `static` lives on `Script::static_vars`, not in the call's frame,
and this crate's frame mirror wiped `economic.bhs`'s on every call after the
first — so the AI's `needed_citizens` was zero from frame 176 and it trained
no citizen for the rest of the game. The word and the sequence run to
**413**, where the original spends two `Unit::do_idle+0x7d` idle anims and a
nine-draw `Unit::think_scout` scan that this crate does not.

## 3.17 An unnamed draw is a wrong answer — 576 → 645 and 576 → 780 (2026-08-31)

Two things landed on East Indies' 576 and only one of them is a mechanic.

**The mechanic** is the bird's landing search (§3.9): sixty draws, run39's
frame 576, more than half of that frame's 118 against this crate's 56.

**The other is the comparison lying.** `Leader::make_stuff`'s expiry walk
had always spent its draws — the arithmetic was settled against run18's own
seeds on 2026-08-25 (`docs/AI.md` §15.3) — and never carried a
[`Sim::mark`]. An unmarked draw takes the name of whatever site marked
*last*, so the ten the AI spends founding its second city at 576 read as
`Leader::compute_sites+0x50a`, which is where the sequence appeared to
part. The frame's real hole was sixty draws further down and invisible
behind it, and Great Lakes' `first_part` had been pinned at 576 for the
same reason since item 60; it went to **780** — its word's own frame — the
moment `+0x221` and `+0x63d` reached `rondata::trace`'s table, with nothing
about the simulation changed.

**The rule.** A modelled draw site with no mark is worse than an unmodelled
one: an unmodelled draw shows up as a bare hex address and reads as a hole,
where an unnamed modelled draw borrows a neighbour's name and reads as a
*disagreement about a draw that is correct*. When a mechanic's draws land,
its marks land with them.

## 3.18 Two sites at one branch — East Indies' word 645 → 742 (2026-08-31)

Frame 645 of run39 is seven draws and this crate spent eight: an extra
`Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99`, the return stand,
because a woodcutter of ours had decided to walk home and the original's
had not. The wait it was counting down was **105** where the original's
was **355**, and both numbers came out of the same draw.

`Unit::do_non_flat_gather` reads guy 0's `cur_anim` before it reads the
tile, and the branch it takes there carries its own reroll:
`CHAR_CHOP_WOOD` is `% 100 + 300` at `+0xcc3`, the arrival frame under it
is `% 50 + 100` at `+0xdad`, and `CHAR_MINE_ORE` returns without touching
anything. `docs/ORDERS.md` §6.4 has had all three since August; the
implementation had one merged branch that marked `+0xcc3` and rolled the
arrival's formula. A woodcutter therefore ran its cycle at a third of the
original's length from its first reroll on.

**What could not see it was the count.** Both sites draw exactly once, so
the word stayed matched for six hundred frames and the sequence with it; a
label-for-label comparison of the frame agrees too, because the label was
the right one. What sees it is the record: the dump prints `GATHERORDER`'s
`wait` every frame, and every rise in it over run39's 1,850 frames is
produced by the value the trace's own draw returned — 24 of them, 19 at
`+0x54b` and 5 at `+0xcc3`, **none at `+0xdad`** in the whole capture. The
branch the implementation spent every reroll on is the one the original
reaches essentially never.

**The rule.** Where two draw sites sit on branches of one predicate, the
site is not the assertion — the *value* is. A count cannot tell them apart
and neither can a sequence; only the field the roll lands in can, and the
dump prints it.

## 3.19 The far wander's literal bearing — East Indies' word 742 → 867 (2026-08-31)

Frame 742 of run39 is seven draws and this crate spent six. The one missing
is at the head of the frame: `Guy::set_anim+0x97a < Unit::set_anim+0x56 <
Unit::move_step+0x823`, the stand a unit plays when its step is refused
(`docs/COLLISION.md` §5). The item was booked as a collision, and **the
collision model was already right**. What was wrong was three hundred units
of ground, six frames earlier.

**Which unit.** No player unit is blocked anywhere near 742 — the dump's
`collide_frame` is −1 on all sixteen. Gaia's `8/3` is: it walks from frame
737, and the frame the trace calls 742 is the one whose end state the dump
writes as `FRAME 743`, where `8/3` holds its point with `collide_o 2,
collide_who 8` and `orders_x/orders_y` collapsed onto itself — the
`QUEUE_NEW` clear of §3.14. Its blocker is its herd-mate `8/2`, standing
still at `(28856, 24197)` since before the walk began.

**The wander that took it there.** `Animal::do_idle`'s coin comes up on
frame 736 (`+0x83`, `39952 % 10 = 2`), and no direction draws follow it —
the animal is `477` from its herd centre, past the `0x181` gate, so it
takes the **far** branch, which spends no draws at all:

    UnitType::find_nearby_spot(type, cx, cy, &out_x, &out_y,
                               0xc0, -1, 0, 0x55555555, FILTER_NOT_ME, o, who, …)

The ninth argument is the bearing the sweep's thirty-one directions fan out
from (`docs/ORDERS.md` §10). Every other call site in the executable passes
a real heading — `do_gather` passes the angle to its camp, `do_build` the
angle to its site. **This one passes the literal `0x55555555`**, which is
`Unit::init@00612100`'s untouched-angle constant, 120°, and has nothing
behind it. So a far wander's sweep starts from the same direction for every
animal of every herd, whichever way the animal is looking.

This crate passed `Movement::facing`. run39's `8/3` was facing **south**
(`UNITDATA angle -2147483648`), 180° out, and the two answers are two
different walks:

| | bearing taken | spot |
|---|---|---|
| the original | `120° − 22.5°` (`k = −1`; 120° and 142.5° were refused) | `(28968, 23976)` |
| this crate | near due north | `(28728, 24120)` |

Both are the first ring, `r = 0xc0`, around the herd centre `(28800,
23936)`; the winner is snapped to its quarter-tile centre, which is why
`(28968, 23976)` sits at radius 173 rather than 192.

With the constant in, `8/3`'s walk is the dump's step for step — `(28741,
24384)`, `(28754, 24360)`, `(28767, 24336)`, `(28780, 24312)`, `(28793,
24288)` — and then the refusal, the dropped walk and the stand all fall
where the original's fall, with **no change to the collision model**.

**What it is worth.** East Indies' word **742 → 867**, and Great Lakes'
window totals **977/866 → 986/884** with its own word holding at 780 (the
same branch, a different herd). run10's collision and angle coverage each
lose two unit-frames past its parting — 97,118 → 97,108 and 37,174 →
37,170 — with the headline 572/776 and every one of the fourteen by-unit
partings unchanged. run39's game score holds at 167/167.

The widening that came with it is the wider claim, and it is
[`rondata::diff::a_far_wander_sweeps_from_the_literal_bearing`]: gaia's
animals are the one population run39 lets free-run for its whole length
with nothing installed — the capture prints no `GUY` clocks, so
`Sim::reseat_animal` never fires — and **190,417 of its 192,504 dumped
animal-frames now stand on the original's own point**. The first that does
not is frame 983.

**The rule.** A constant in an argument list is a claim about the mechanic,
not noise to be filled in from context. Where the decompiler prints a
literal where a variable would read naturally, it is worth one grep of the
other call sites before assuming the natural reading — and here the natural
reading cost 125 frames of the word.

## 3.20 The goody box — East Indies' word 867 → 879 (2026-08-31)

Frame 867 of run39 is nine draws and this crate spent five. The first three
are `Unit::explore_goody+0x27c < Unit::set_new_location+0x3cc <
Unit::move_step+0x8f4` — player 1's scout `1/0` walking south out of cell
`(45, 50)` into `(45, 49)`, one of the seven cells East Indies' `WORLD`
record marks `GOODY`. The ninth was the frame's sixth `Farms::inc_time`
draw (`+0x1de`, the sprout), which the original's stream reaches only
because the three before it moved the word.

**Three draws, not one and not six.** `explore_goody` runs a lottery over
goods 0…5, skipping knowledge outright and skipping anything
`LeaderData::type_avail@006e33a0` does not call available, and each
survivor costs one draw. In the Ancient age the survivors are food, timber
and wealth — metal and oil are not yet available and knowledge is refused
by name — so **the draw count is a test of `type_avail` over the good
types**. Every capture on disk that reaches a box agrees: run39 on 867,
run33 on 898 and 1659, three draws each.

The mechanic is `docs/GOODY.md`; the two facts a reader of this file wants
are that the trigger is `set_new_location`'s **cell** test (`p >> 8`), not
the tile test the fog reveal hangs off, and that the pile's multiplier is
`LeaderDataEncrypt +0xf4` — `epoch[3]`, the **Science** library level —
however much `GOODY_BOX_AGE` sounds like the age.

**What it is worth.** East Indies' word **867 → 879**; Great Lakes'
word holds at 780 (its boxes are at 898 and 1659, past its parting) and
run10 is untouched — world6 has no goodies its units reach. run39's
gaia agreement over the *whole* capture falls 190,417 → 189,843 with its
first parting frame **983 unmoved**, which is what a count past the word's
parting does whenever the word moves: every frame after 879 draws from a
stream that is nobody's, so the herds re-roll. The number to read there is
983, not the total.

~~Frame 879 is next, and it is the same scout: two `Unit::set_anim` stands
and then `Unit::think_scout+0x436`/`+0x458` six times over with `+0x64c`
twice — the region fallback's cell walk (`docs/SCOUT.md` §11), sixteen
draws this crate does not spend.~~ **Closed 2026-08-31** (item 110):
`Regions::rebuild_coords@0067f800` refills every region's coordinate list
by a row-major sweep of the cell grid, so the walk needs no dump and
`docs/SCOUT.md` §11 is implemented. East Indies' word **1373 → 1570**.

## 3.21 A pasture nothing stocked — Great Lakes' word 1372 → 1802 (2026-08-31)

Frame 1372 of run33 is twenty-eight sync draws and this crate spent three.
The three it spent were `Farms::inc_time`'s; the twenty-five it did not are
one building finishing:

```
5×  Farms::add_animals+0x92   < Build::activate+0x1c25 < Wall::do_construct+0x199
5×  Farms::add_animals+0x134  < …
5×  Farms::add_animals+0x182  < …
5×  Guy::init_real+0x52       < Unit::init+0xb97       < Animal::init+0x1a
5×  Guy::set_anim+0x97a       < Guy::inc_time+0x271    < Unit::inc_time+0x3e
```

interleaved four at a time — coin, `y`, `x`, guy — and then the last five
together. §3.6 and §3.11 had already read every one of those sites. What was
missing was the **caller**: nothing in this crate ever reached
`Farms::add_animals` except the harness's own setup, so a pasture built
during a game stayed empty.

**`Build::activate` stocks every food gather building it finishes.** The
tail at `006259b5` switches on the *good*, not the type. `LAB_00625a36` —
where `case 0`, `do_bonus(0, FOOD_BONUS_FOR_FARM)`, falls through — and the
`iVar18 == 0` arm of `switchD_006259b5_caseD_2`, where every exit that pays
*nothing* lands, are the same call. So the gate is only "is this a farm";
whether the completion bonus was paid, whether the building was captured,
whether the caller counted it, all of that is settled above and none of it
reaches here. `Farms::add_animals`' own two guards are then `farm_type == 1`
and the building's `flags & 4`, and neither is a count — **a farm activated
twice is stocked twice**, which no capture exercises and this crate
reproduces.

**The pin is the original's own word.** run33's frame 1372 begins the block
on `game_random == 0xc91f99f2`; twenty draws later the original's
twenty-first draw of the frame starts from `0xafa38116`, and this crate now
lands on it. A pasture's four-draw stride keeps every coin on one parity of
the stream (§3.11), so all five of these are even — five **chickens** — and
their `(dy, dx)` from the building are `(−77, −126)`, `(−89, −154)`,
`(−165, 10)`, `(79, 110)` and `(−125, 146)`, each `rnd % 0x180 − 0xc0` and
each snapped by `Unit::init`. `farms::tests::
a_finished_pasture_stocks_five_animals_for_twenty_draws` asserts the word,
the twenty labels in order and the five snapped points.

The last five draws are not `add_animals`' at all and cost no new code: a
guy fresh from `Guy::init_real` has `end_time 0`, so the same frame's
`Objects::inc_time` wraps all five clocks (§3.16's second loop is what
reaches a unit born mid-frame).

**Where the harness parts from the original, knowingly.** The original stocks
its *starting* pastures too — `Setup::build_empire` runs with `Game::frame`
still 0, the same test the completion bonus is gated on one line above. The
harness does not replay that stream; it borrows those five from the capture's
own trace instead (§3.11, `diff::borrow_pasture`). So the drawing path is
suppressed at frame 0, and only there.

**What it is worth.** Great Lakes' word **1372 → 1802** and its sequence with
it — past run10's whole 1,772, so this map now spends the original's draws in
the original's order for longer than its own capture runs. The two totals go
1467/1436 → **1832/1830** of 1,850, and the gap between them falls from 31 to
2. run10's headline goes **1375/1374 → 1772/1772 with neither player parting
at all**: no unit's position disagrees anywhere in the capture, no order field
that scores disagrees anywhere, and no gather tile disagrees anywhere. The two
residue items that had been holding it — `1/8`'s move-order `x` and `1/1`'s
cleared `last_x/last_y` — were both downstream of a stream that had been wrong
since 1372 and are gone. East Indies is untouched at 1374/1373: run39's AI
builds no farm inside its capture.

What parts run33 at 1802 is a single draw at
`Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 < Unit::do_job+0xd7` —
a gaia bird's flight physics, and the **only** time that site is reached in
all 1,851 frames.

## 3.22 The loop's bound is re-read — East Indies' word 3608 → 3687 (2026-09-01)

`Objects::process_all@0065dce0`'s unit loop does not walk a list it took
before it started. Its `do { … } while (iVar9 < *(int *)(&this_01->field_0x15c
+ iVar5 * 4))` re-reads `ObjectsData::unit_mark[who]@+0x15c` on every turn, so
**a unit created inside the loop, in an object slot above the one being
walked, takes its own turn on the frame it is born**. `Sim::tick` fixed its
visit list before the loop and said so in a comment; that was the eighth time
a document's own reading and the code disagreed, and this time the code was
carrying an assumption nothing had ever tested.

**One unit in the whole capture is on the wrong side of it, and it is the
transport barge.** `SpellType::cast_transport` runs inside the unit loop —
the caster is the AI's scout `1/0` — and gives its boat object `1/14`, so the
walk that is at `o 0` reaches `o 14` before it ends. The barge takes the
scout's move order, steps, and `Unit::move_step` sets its guy to `CHAR_WALK`;
`Objects::inc_time` then finds `cur_time 1 < end_time` and the brand-new
clock never wraps. This crate deferred the barge to the next frame, so its
guy still stood at `Guy::init_real`'s `cur_time 0, end_time 0` when the
clocks were stepped, and spent a `Guy::set_anim+0x97a < Guy::inc_time+0x271`
the original spends on no frame at all.

**The trace says it in one table, and the table is why this needed no
capture.** Over run57's 4,000 frames eleven frames create a guy through
`Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd`. Ten of
them — 274, 380, 494, 615, 1704, 1911, 3319, 3526, 3734 and the setup — are
**trained** units, born in `Build::do_queue` in `process_all`'s *second*
loop, after every unit has had its turn; every one of those spends its wrap
on its birth frame, exactly as §3.16 established. The eleventh is 3608, and
it is the only birth in the whole run with no wrap behind it. `docs/QUEUE.md`
item 135 had a `GUYS=4` window booked against it; the trace already on disk
answered it, which is the queue's own rule about grepping the disk before
booking a capture, one more level up.

**What it moved.** East Indies' long word **3608 → 3687**, sequence with it —
seventy-nine frames, and every one of them draw for draw. Great Lakes holds
at 1802, both scored captures hold, and run57's own four-thousand-frame
position test holds at one wrong building and no wrong collision field.

**A second change rode with it, and it is the same reading.** The visit
order inside an owner's band is now the object order the loop actually walks
rather than the order the units happen to sit in `Sim::units`; §3.2 has said
"in object order" since it was written, and the two only differ once a slot
is reused. Nothing in the suite moved on it.

**What is at 3687**: ten draws against seven. The first of ours is a
`Guy::set_anim+0x97a < Unit::move_step+0x823` — the blocked walker's idle,
`docs/ORDERS.md` §10 — that the original does not spend, and behind it we
throw **seven** gaia wing-beat coins where the original throws five. §3.23
is what that was.

## 3.23 The citizen offers itself to the boat — East Indies' word 3687 → 3978 (2026-09-01)

Both halves of 3687 were **one unit's**, and it had left the original's
point ninety-five frames before the word noticed. The blocked walker is the
AI's citizen `1/11`, standing where the original's is not; the two extra
wing beats are the coins that one extra draw re-throws, since a bird whose
walk animation wraps keeps re-throwing while the length it picks still
overruns its clock (§3.9).

**What the citizen should have been doing.** `Unit::think_peasant@005f5760`
opens — after its idle gate and *before* `find_build_spot` and the gather
search — with a **colonist arm**: an AI-driven unit (`unit_masks &
0x40000`) whose **base type** is `0x32` or `0x33` calls
`think_civilian_transport(1)`, and a `1` back ends the think
(`005f5809`). The test is the type itself and not the worker category, so
a scholar never asks. This crate had `think_scout`'s tail — the
`colonise = 0` caller — and not this one, and the whole colonise gate,
`xport_peasants` throttle and all, therefore had no writer between
sweeps (`docs/TRANSPORT.md` §7).

**The frame it costs, in the original's own dump.** run57's `1/11`
finishes its build on 3580 and stands with no order; on **3581** it holds
`group 65`, a `MOVE_TO` whose `orig 38784, 24192` is the centre of cell
**(50, 31)** — `cell × 0x300 + 0x180`, `think_civilian_transport`'s own
arithmetic — and a twenty-four-leg path already planned. This crate sent it
back to its gather instead, at a destination of its own, and every waypoint
after that was a different point in the same tiles: the two paths differ
only by `off_x/off_y`, the destination's offset inside its world cell,
which `find_tpath` and `go_around_building` both place their points from
(`docs/ORDERS.md` §4.1, §4.6.1).

**What it moved.** East Indies' long word **3687 → 3978**, sequence with
it. run57's four thousand frames go from **eleven** units ever off the
original's point to **four**, its earliest parting from 3582 to 3647, and
its comparable collision field-frames from 330,643 to **348,354** with none
wrong. Great Lakes holds at 1802 and both scored captures hold.

**The earliest parting is now before the word, and that is not a
contradiction**: `1/13` leaves the original's point at 3647 and costs no
draw for it. The word is what the two streams *spend*; a unit can stand a
few units off and spend exactly what the original spends. The run57 test
pins both numbers rather than assuming one bounds the other.

**What is at 3978**: four draws against five, and the one we do not spend
is the original's `61a1da` — inside `Unit::come_out@00617c10`'s tail
(`+0x25ca`), unnamed in the trace. The AI's scout `1/0` leaves the
original's point on 3979, the frame after, so the two are one thing; the
barge cast at 3608 (§3.22) is what it came out of. §3.24 is what that was.

## 3.24 The passenger comes ashore — East Indies' word 3978 → 4020 (2026-09-01)

The item was booked as one draw and was **three** things, in the order the
original does them, all inside `Unit::come_out` and its caller.

**The draw is an army coin, and only two lineages throw it.**
`come_out`'s tail (`0061a0c5`..`0061a1fb`) is `add_to_army`'s fifth caller:
an AI-driven, non-caravan, non-merchant unit leaving whatever carried it
joins an army unless a coin says otherwise — and the coin is only thrown
when the unit `is_special()` or `is(BARK)`. `is_special` is `is(SCOUT)` by
the loader, so the two arms are the scout line (`% 2`, `+0x25ca`) and the
naval-scout line (`% 3`, `+0x25b0`); everything else takes a draw-free
`is(SPY)` test. That is why a site reached eleven times in run54's 24,000
frames had gone unseen for 3,977: the AI trains citizens and soldiers, and
neither asks. `docs/ARMY.md` §4.1 has the listing and both addresses.

**The spot is `come_out`'s host arm, and every term of it is the boat's.**
Centre, bearing (`host->angle`, `+0x50`) and inner radius (the **host's**
`block_radius`) all come off the host object in `eax` at
`61845c`..`618483`, while the fallback-arm test at `618490` reads the
*passenger's*. The decompiler folds the two into one local, which is the
easy thing to misread. run57 block 3979 pins all three at once: the barge at `(35740,
26706)`, `angle -13303808`, `BLOCK_RADIUS 3` → the ring `[144, 720]` step
72, whose first candidate snaps to the scout's own `(35736, 26568)`.
Then `set_angle(passenger, host->angle, ·, 1)` and a crew seated **on** its
track offset, which is block 3979's second `GUY` at `(35640, 26616)`.

**And the ring only reaches land because the barge marks nothing.**
`docs/COLLISION.md` §2's region gate compares the cell's `region` against
`WorldData::get_tregion` of the marking figure's *tile* — and
`get_tregion` answers a coastal cell's **`region2`** for an ocean tile. A
boat on the water half of a coastal cell therefore marks no collision
cells there at all. This crate asked the plain `region_of`, which made the
gate vacuous for exactly the case it exists for: the barge filled its own
world cell, the first three rings were refused, and the scout landed four
hundred units inland.

**`Object::eject_contents` is `cast_transport` run backwards.** For a
passenger of `uber_size` 1 the boat's whole order list moves back onto it,
and the boat's path stack is inverted and popped onto its own — which
restores the order it was in — with the top's embark flag cleared. Block
3979's `MOVEORDER` and both `PATHDATA` entries are block 3978's barge's,
field for field, with `flags 4 → 0`. `docs/TRANSPORT.md` §6.4.

**What it moved.** East Indies' long word **3978 → 4020**, sequence with
it. run57's four thousand frames go from **four** units ever off the
original's point to three — `1/0` now stands where the original's does for
the whole capture — and the comparable collision field-frames go 348,354 →
**348,469** with none wrong. Great Lakes holds at 1802 and both scored
captures hold.

~~**What is at 4020**: four draws against six, and the two we do not spend
are a `Guy::set_anim+0x97a < do_cast` pair — the scout reaching the *next*
shore and casting its second transport, man and dog.~~ **The `do_cast`
pair is a citizen's, not the scout's** — §3.25 is what it was.

## 3.25 The colonist cannot see the shore — East Indies' word 4020 → 4275 (2026-09-01)

**The item was booked on the wrong unit, and the trace said so before any
capture did.** 4020's unspent pair was read as the AI scout `1/0` casting
its second transport, so the item was "the scout's second leg at 4005".
The scout is right on both sides: it takes the region fallback (`docs/
SCOUT.md` §11) on 4005 with the same eighteen draws, the same ten
candidates and the same winner — cell **(48, 30)** — and arrives there on
**4110**, which is the very frame run54's own trace throws its next
`+0x941` and ten `+0xaba` on. Two independent scans agreeing frame for
frame is not a coincidence, and it took the scout out of the frame.

What put it there was a smaller thing: at 3608 the scout's own cast spends
**two** `Guy::set_anim` draws, `+0x56` and `+0xb6`, and 4020's spends
`+0x56` alone. The scout is a man and a dog; the caster at 4020 has one
figure. It is the AI's citizen `1/15`, and it is on the other side of the
map.

**Then two reads, three hundred frames upstream.**

- **`Region::coast_here`'s neighbour probe is `get_tregion`, and this
  crate asked the plain cell region.** `0068106a` reads the neighbour
  cell's **centre tile** through `WorldData::get_tregion`, which answers a
  coastal cell's `region2` — the sea — when that tile is ocean. A coastal
  cell is a *land* cell in `WData.region`, so with the plain read the
  function can only see a wholly-ocean neighbour and a cell one in from
  the waterline coasts nothing at all. `think_civilian_transport` keeps
  only the cells `coast_here` accepts, so `1/15`'s destination was drawn
  from the wrong set: this crate sent it to cell (37, 36) where the
  original sends it to **(39, 34)** on frame 3735. Which is §3.24's lesson
  in a second caller, and the second time in one day that `region_of`
  stood in for `get_tregion` and made a gate vacuous for exactly the case
  it exists for. `docs/TRANSPORT.md` §9.3.
- **`Unit::move_step` reads the waypoint's own turn-in-place bit, and
  nothing here read it.** `005fb1a5` is `(manh < slow × 0xc0) ||
  (path.flags & 4)`: a waypoint that crosses the waterline is *turned to*
  before it is walked to, whatever the distance. `Sim::shore_flagged`
  wrote that bit and no reader existed, so `1/15` walked through the turn
  the original stands still for on 3988 and reached its embark point a
  frame early. `docs/MOVEMENT.md`, "The unit step".

**What it moved.** East Indies' long word **4020 → 4275**, and its count
to 4288. run57's four thousand frames go from **three** units ever off the
original's point to **two** — `1/15` now stands where the original's does
for the whole capture, its first parting having been 3737 before the
first fix and 3988 between them — and the comparable collision
field-frames go 348,469 → **349,794** with none wrong. Great Lakes holds
at 1802 and both scored captures hold.

**What is at 4275**: five draws against five, differing at the second —
ours a `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99`, the
original's a `< Guy::inc_time+0x271`. A woodcutter's clock, which is
§3.14's and item 103's shape. The **count** holds to 4288, where the
extra pair is a gaia herd's.

## 3.26 Great Lakes parts at 2419 on a route, not a clock (2026-09-03)

**The map had stood still for two days and no item named it.** Great
Lakes' long word has been 2419 since run61 put the bird's birth right,
through every session since; DECISIONS 25 makes the *lower* map's nearest
divergence the default item and the queue had been chasing the higher one.
Item 193 is the look, and it took a trace already on disk plus the capture
the map had earned (run69, `docs/ORACLE.md`).

**What parts is one draw.** Frames 2416–2418 and 2421–2422 agree label for
label; 2419 is one draw here and none in the original, and 2420 is one in
the original and none here. The draw is the same on both sides —
`Guy::set_anim+0x97a < Unit::set_anim+0x56 < Unit::do_non_flat_gather+0xb99`,
the return-to-camp stand of §6.4's wood machine — and the unit is the AI's
woodcutter `1/9`. So the word does not part on *what* happens; it parts on
**which frame**, by one.

**The clock behind it is exact on both sides.** `1/9` chose its tree on
frame 1959, and the tile choice's own draw — `+0x54b`, `400 + rnd % 200` —
returns **4** in run53's trace, so the wait is **404** here and there. The
countdown does not start at the choice: the walk goes in front of the
gather order, and the order is only reached again when the walk ends. 404
frames later is the return stand, and the frame it lands on is therefore
the frame the walk ended.

**The walk ended a frame early**, and run69's own `MOVEORDER` rows say why:
`1/9`'s two middle waypoints are each one 48-grid step short in x of the
original's. Same ends, same total length, same switch frame, different
dog-leg — and this crate reaches the tree on 2015 where the original
reaches it on 2016. `docs/PATHFINDER.md` §17 has the table and what is
still unread.

**So the lower map's seam and the higher map's frontier are one defect.**
run68's `1/13` on East Indies parts on exactly the same two middle slots
of its own path stack from block 6686 (item 191); this is that, on a map
with no islands, no barge and no merchant. The word did not move that day —
what moved is that one item stood in front of both numbers instead of one.

**Closed the same day** (item 191). The route was not the search's: run70's
`callwin` over `PathFinder::calc_cost` — which runs only for a cell that
passed `valid_ucoord` — puts the original's expansions beside this crate's
and they differ on exactly **one** cell, `(40776, 17592)`, which the
original refuses. The refusal turns on a corner of the standing gatherer
`1/10`'s collision block that `1/9`'s own step had cleared and
`Guy::process@005e0230`'s sixty-fourth-frame repaint had put back
(`docs/COLLISION.md` §2.2, `docs/PATHFINDER.md` §17). **The word moved 2419
→ 2808**, and the map's earliest unit parting 1993 → 2804.

## 3.27 The bird's arrival stand — Great Lakes' word 7584 → 7585 (2026-09-07)

**A bird's figure follows its unit like anybody else's, and the frames it
does not move are draws.** `Animal::process@005d72c0` is the `+0x188`
think and then `Guy::process` per figure, and nothing in `Guy::move`
excludes owner 9: its arrival arm — `des == pos`, `des_angle == angle`,
`field_0x9c == 8` (the **slot** `CHAR_WALK`, not the category) and
`field_0x9d` set — spends `set_anim(CHAR_DEFAULT, 0, 1)`, which is
`Guy::set_anim+0x97a < Guy::move+0x19f` on the stream.

What makes a bird *look* exempt is one argument:

```text
do_air_physics tail:  set_new_location(nx, ny, 0, 1)          # 005e86d0:237
Unit::set_new_location@005f8d20:
    :156  param_4 → Guy::set_angle(guy0, unit.angle, param_3)
    :158  guy0.des = the unit's new point                     # unconditional
    :161  param_3 → Guy::set_new_location(guy0, des, 1)       # the teleport
```

`param_3` is **zero**, so the figure is *told* where to be and not put
there. It catches up inside `Guy::move` itself — for guy 0 the whole step
is `:182`'s "no track offset" arm, which places it straight on `des` — one
step later in the same frame. So `des == pos` at `Guy::move`'s entry means
**the bird did not move this frame**, and for a bird under a patrol order
that happens exactly when `WorldData::restrict` clamps a refused step back
onto the boundary the bird already stands on.

The stand then feeds the wing beat, and the pair is the observable:

- the stand leaves the figure on `CHAR_DEFAULT`, so the **next** frame's
  `set_anim(CHAR_WALK, 0, 1)` is a category change and misses gaia's
  `who >= 8` early return (`set_anim:143`) — a wing-beat coin;
- the coin takes `CHAR_WALK` half the time, and `field_0x9c == 8` stands
  again on the same frame; it takes `CHAR_JOG` the other half, and the
  run ends.

run89's `[7514, 7760)` has the whole of it: **16** air-physics coins in
7,776 frames, of which **7585, 7586, 7587 and 7588** are four; 7584 stands
with no coin (the figure was already on `CHAR_WALK`), 7585–7587 pay coin
and stand together, and 7588's coin takes the jog and ends it.

**Which bird, by elimination over the whole dumped record.** No dump
prints owner 9 (§3.9), but run89 prints all forty owner-8 animals in
full: on block 7584 not one of them is on `CHAR_WALK`, and the thirteen
owner-1 guys that are all carry `stopped 0`. So no dumped figure could
have spent 7584's stand, and the draw's neighbours — the fourth and fifth
`Animal::think_bird` triples — place it in the animal pass. The bird is
named as *a* bird and not as one of the ten; nothing on this disk can do
better, and nothing downstream needs it.

**What it was worth.** Great Lakes' long word **7584 → 7585**, 7584 now
agreeing 49 draws for 49 entry for entry, and with it the value diff
beside the word: `1/3`'s fresh `MOVEORDER` on block 7585 used to land one
tile out on each axis from a base both sides agreed on, and every field of
it agrees now. Nothing at all parts at or below block 7585 but run87's own
carried residue. `rondata::diff`'s
`run89_s_window_is_great_lakes_word_frame`, and
`a_bird_s_figure_lags_its_unit_and_stands_only_when_the_step_is_refused`
in `crates/sim/src/air.rs`.

**The gull is a seam, not a rule.** `Unit::do_strafe` is still unmodelled
(§3.9), so this crate flies no gull; a figure sitting on its unit for
ever would roll an idle every other frame the original does not, and
`guys_follow` still returns early for `GULLBIRD` alone. It comes off with
`do_strafe`.

## 3.28 A building's queue runs inside its own `Build::process` — French East Indies 8236 → 8385 (2026-10-03, item 1449)

`Build::process@0061edf0` is, in order: `Wall::process`, then — for an
active building — ejection, launch, the tower's `do_attack`, and then
`Build::do_queue` (vslot `+0x1b4`, by the Build vtable), then the gather
re-entries. `Objects::process_all` calls it building by building in object
order. The tick ran three passes instead: every building's `Wall::process`
part, then every queue, then every tower — "still ours", as its comment
said. A unit that a lower-numbered building trains this frame therefore
exists in the original when a higher-numbered building's `Wall::process`
runs, and did not exist here.

**Where it showed.** run615 (blocks 7958..7999): city `1/2008` trains
citizen `1/51` on frame 7962, which is the wonder `1/2022`'s recruit phase,
`(7962 + 2022) & 31 == 0` (`docs/AI.md` §69.4). In the original the
recruiter finds it, and it is born holding `BUILD` on the wonder; on 7964
it takes the ring's approach (34632,38328). Here the recruiter ran with
203 units, before the birth, so `1/51` was born idle and took a gather
order to camp `1/2019`. 180 frames later the next builder, `1/56`, took a
different ring spot (run614: bearing −1 there, bearing 0 here), because
`1/51` stood elsewhere and refused the original's bearing 0. That spot is
what parted the word on 8236 (run612, `1/41`'s collision with `1/56`).

**The change.** The queue now runs per building, right after that
building's own `process_building`. The tower pass and the gather region
pass stay where they were; that split is still ours, and no capture shows
it.

**What moved.** French East Indies **8236 → 8385**. Widened keys: run610
156 → 127, run611 147 → 121, run612 210 → 124, run614 153 → 122, run615
156 → 120. No key arrives on any of them, and run613 holds at 117, since
it ends before the birth. Every other floor and pin in both suites holds
(rondata, sim 1324). Putting the queues back into their own pass returns
the word to 8236 and fails all five pins (measured).

**Not established.** The tower's order against the queue within one
building, and the gather re-entries per building, are read and not
reordered. No capture separates them yet.

## 3.29 The gull flies to its dock, from a snapped birth — French East Indies 9655 → 9777 (2026-10-04, item 1453)

A dock's gull is born in `Dock::init` on a `StrafeOrder` whose target is
the dock (`docs/TRANSPORT.md` §5). The crate placed it at the dock's
position less `0xc0` on each axis and gave it no flight: it stood there
for the rest of the game. That was a stated seam (`orders.rs`,
`anim.rs`).

**The flight.** `Unit::do_strafe@005eab00` on a gull first calls the
animal's think (vslot `+0x180`, the gull's arm draws nothing). With the
target valid, it then hands `do_air_physics` the target dock's own
position as the goal, every frame. run622's proxied calls show both gulls'
goals at their docks' centres, (44160,41856) and (41472,44736). The crate
now flies the gull with §3.9's physics toward `Sim::gull_dock`, the live
dock whose slot holds it. The order itself is not carried.

**The birth.** The physics alone left gull 1 about 1000 units off by
run620's 6137. `Unit::init@00612100`'s tile snap,
`div_3_table[p >> 4] · 0x30 + 0x18`, applies to the gull as it does to a
wild bird (§3.9, `crate::gaia::init_snap`). The crate had put the gull at
(43968,41664) on its dock's tile edge, where the snap gives (43992,41688).
With the snap, both gulls match the original **exactly** on every proxied
frame on disk:

- run620's 6136..6151, about 3,900 frames after gull 1's birth;
- run622's 9648..9663, including the second gull's edge coin on 9655,
  which is what parted the word.

**What moved.** French East Indies **9655 → 9777**. No floor moves.
run622 loses six figure-animation keys past the old word, and the
unit test `a_dock_with_a_gull_type_draws_twice` now pins the snapped
birth.

**Not established.** The gull's figure is still skipped in
`guys_follow`. Lifting the skip moved no draw and no word, and nothing
dumps owner 9. The other arms of `do_strafe` for a gull (invalid
target, the dock closed) are read, not exercised.

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
| 101 | **21** | the rotation puts the AI first: **0–5: the AI's farmers `1/3`, `1/4`, `1/5`, two `% 4` each** — `1,0 / 3,2 / 1,0`, and their new `MoveOrder` goals are the farm's centre `− 264 + 192·r` on each axis; **6: sheep 0 arrives** (its walk cut at `12/16`, `set_anim(CHAR_DEFAULT)` from `do_idle`, `% 100 = 27` → variant 0); **7–12: the human's farmers `0/3`, `0/4`, `0/5`** — `2,1 / 0,3 / 2,1`, likewise; **13: the human scout's wrap** (`60/61 → 0/61`, phase 7, leader 0 before anything else); **14–20: the seven farm draws** — farm 1 of the list (the AI's `2003`, twelve empties) sprouts at draw 15 (`% 1000 = 6`) and draw 16 picks `65297 % 12 = 5`, the sixth empty cell **in the search's own order** — `inc_time` walks `4·dx + dy` with `dx` inner and `dy` outer, so it visits `0, 4, 8, 12, 1, 5, …` rather than `0, 1, 2, …` — which lands on **memory index 5**, the one cell that appears in the next pass. (Print order agrees on this one by luck: both orders reach index 5 sixth. The two are told apart by the diff, which fails at frame 2 under the wrong one.) |
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

**The two struck rows were one line** (§6's stand/wrap entry;
`docs/JOURNAL.md`, 2026-08-26). The standing facts: the original draws
**no** unit-phase stand for a gathering citizen at frame 0 — those guys run
their animation out in phase 7 and re-roll there — and the sim's
camp-arrival stand, its own invention where the original branches two ways
on `CHAR_DUMP_WOOD`/`CHAR_DUMP_ORE`, had been resetting their clocks so the
wraps never fell due. Removing it moved four draws, not two. The dumped
clocks could not have settled it: a wrap whose roll lands on the slot
already running leaves `cur_time` stepping normally, so the trace says four
wraps where the dump shows two. The **sequence** did. Frame 0 is now
compared draw for draw rather than by these blocks
(`diff::tests::frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`),
on run20 and on the Great Lakes.

Frame 1 and frame 2 fall out of the same fold. Every row below is now a
site rather than a block — `--diff`'s `by phase` note prints the labels
`--trace` prints (§5):

| frame | theirs | ours | what is left |
|---|---|---|---|
| 1 (run20) | 53 | **53** | none — **draw for draw, the whole frame, since `go_around_building` landed 2026-08-26** (`docs/ORDERS.md` §4.6.1). The last `Unit::do_move+0xe84` was a unit whose straight line clips a *building* short of its waypoint; the sim now finds the same detour and spends nothing on it. `Leader::produce_building` is 39 + 4 on both sides (`docs/AI.md` §2.20) and the farm it places lands on the original's own tile; `Farms::add`'s pair, the three `do_non_flat_gather+0x54b` and the five farms were already matched |
| 2 (run20) | 5 | **5** | none — the five crop farms and nothing else, on either side |
| 1 (fuzzed) | 45 | **43** | ours is two *short*: one `Leader::produce_building+0xc99` (29 against 30 — a spiral candidate the original scores and the sim does not) and one `do_non_flat_gather+0x54b`. The jitter is **3 on both sides** here, one of the 2×2's four sub-positions being blocked — the second map that makes the inclusive reading a rule |

The unit that pays run20's last `+0xe84` is `1/1` (`docs/AI.md` §2.20 has
the arithmetic; the story is `docs/JOURNAL.md`, 2026-08-27, and the lesson
§5.1's).

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
  fold **by site**, in the same shape `by phase` prints ours. Addresses are
  bare; naming them needs the Ghidra export's `INDEX.tsv`, which never
  enters this repo, so `report.py` is where a name comes from.
- `Trace::run_in(frame, lo, hi)` isolates **one function's own draws** from
  the ones its callees took, which is what makes a per-mechanic assertion
  possible at all.
- A mechanic may mark its own draw sites (`Sim::mark`, under the
  original's offsets — `crate::scout`'s three are the first), and
  `diff::mark_sites` expands the marks into one label per draw. Seed the
  sim with the trace's own word, run the mechanic, and compare the two
  **sequences**.

That last one is the point: a count cannot tell four rotations and two
phases from three and three, a sequence can, and it is checkable **while
the frame's stream is still wrong upstream**.

### 5.1 The frame as one sequence (2026-08-26)

Widening the scout's seed-anchored, one-function check to a **whole frame**
needed one piece: the trace's addresses and the harness's marks had to be
the same strings.

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
the first parting with three draws either side. Made to fail on purpose by
dropping `sim::anim`'s wrap mark, which reads as four `guys_inc_time`
against four `Guy::set_anim+0x97a < Guy::inc_time+0x271` at draw 166.

**A phase left standing in the fold is worth more than a matching total,
and marking one has now paid twice.** The second was
`Leader::produce_building`, frame 1's last unmarked phase
(2026-08-26): the frame read 53 against 53 and the two marks split it into
`+0xc99` 39 against 41 and `+0x1805` 4 against 1 — **three** defects whose
residues happened to cancel, one of which (`buildings_allowed`) was a map
layer the dump had been carrying unused. `docs/AI.md` §2.20 has all three.
The rule that falls out: *mark the phase before believing the total*, and
mark it even when the total already agrees.

**The per-frame counts that got us here are the journal's** (2026-08-24
to 08-27). They are superseded by the draw-for-draw comparison of §5.1,
which places every draw by site rather than counting a frame's; what the
counting phase left behind as rules is here and in `docs/ANIM.md` §6, and
what it left behind as gaps is §6. Two of its conclusions are still load-
bearing and are stated here rather than in the journal.

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

  **Closed 2026-08-26**: `do_non_flat_gather`'s camp-arrival branch has no
  `CHAR_DEFAULT` (`:398`–`:416`, the listing at `5f0b5e`–`5f0b89`), and the
  invented stand had been resetting the citizens' clocks so the wraps never
  fell due. `docs/ANIM.md` §5's "the four woodcutters' draws are unit-phase
  stands" is superseded — they are wraps. The four readings this replaced,
  and why each looked right, are in `docs/JOURNAL.md` (2026-08-24 and
  2026-08-26).
- ~~**The human scout's ~~15~~ 14 and the AI scout's ~~8~~ 6** (the dogs' rolls
  are the other three, `docs/ANIM.md` §5) are placed by elimination,
  not by outcome~~ — **placed by site (run14): `Unit::think_scout` draws
  24 times at frame 0**, at three sites — `+0x436` (6), `+0x458` (2) and
  `+0x64c` (16) — all under `Unit::think+0x7da` < `Unit::do_idle+0x94`, in
  the order 436 436 436 458 64c 436 458 436 64c 64c 64c 64c 64c 436 64c ×9;
  and 15 at sim-frame 95 (`+0x436` ×6, `+0x458` ×6, `+0x64c` ×3), none on
  96–103. No pathfinder draw and no `do_move` draw appears anywhere on
  frames 0–3, so the AI scout's first path costs nothing on the stream.
  On two further maps (§4.2) the shape is identical: ten draws on run20 and
  ten on the fuzzed one, `+0x436` ×4, `+0x458` ×2, `+0x64c` ×4 in both.
  That the count matches on two unrelated maps says at least one of the
  three sites is fixed-count, which a scan of "one per unseen candidate
  cell" would not be.
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
  The arithmetic, which needed no capture — the dump on disk had both
  sides — and the seam it overturned are in `docs/JOURNAL.md` (2026-08-28,
  lifted from here).
- ~~**And the goal at the bottom of that stack is `0x18` short of the
  order's own**~~ — **settled 2026-08-26, and it is not the pre-walk.**
  It is `Group::action_move_near`'s own goal push: a group plans one path
  on the global `grouppath` whose `FINAL` entry is the **raw slot
  destination**, un-snapped, and hands it to its members
  (`docs/GROUPS.md` §6.7, and §13 for the three disagreements it costs
  run20). The simulation does not implement §6.7 at all; the queue's item
  31 is that, not a pre-walk. `docs/PATHFINDER.md` §12 records why the
  pre-walk and `do_move` are both ruled out. The numbers behind the
  superseded pre-walk reading — the original's `(41952, 36576)` against the
  order's `(41976, 36600)`, `add_move_order`'s `u*0x30 + 0x18` — are in
  `docs/JOURNAL.md` (2026-08-26).
- **The pasture's twenty creation draws, and its animals' art** — §3.6's
  own open list: the offsets, the chicken/pig coin and the animation
  lengths all live in streams or dumps that no capture carries.
- **Diplomacy's cadence** (`Leader::diplomacy`, nine sites; 0 draws on
  frames 0–3).
- ~~**The caravan road, frames 10, 11 and 171**~~ — **read 2026-08-28,
  `docs/ROADS.md`, and the caravan was a misnomer.** The caller is
  `Build::process`: a **building** whose `build_masks & 0x100` fires on the
  frame `(frame + o) % 16` picks out, replanning the road to its city
  centre. Its schedule and its ring are diff-backed; its expansion count is
  six per cent short (208, 222, 178 against 220, 248, 189), so
  `Sim::plan_roads` is off and the sim still draws 6, 6 and 7 there.
- ~~**Animals, and the animation clock**, the lengths being art data
  (run12's `end_time`s cycle 101/116/170 for the fish and 90/109/250 for
  the sheep with the scale variant `o % 3`; the scout's idle is 41 or 61,
  the citizen's 33, the sow 47), and worth 12 of sim-frame 100's 18 draws
  and 2 of 101's 21.~~ **Done, 2026-08-24 —
  `docs/ANIM.md`, `crates/sim/src/anim.rs`.** The clock, the wrap's roll,
  `init_real`'s, the arrival, the mirror, the animals as units of owner 8
  with the wander; the lengths and pieces as the `Art` input read out of
  the dumps (the variant is `(seed + o) % 3` for gaia, the gender bit `o &
  1` for a citizen); the harness installs the clocks beside the words. What
  it left open is its §9: the woodcutters' un-stepped frame 0, the dog's
  own roll, the unobserved lengths, `think_farm_animal`.
- ~~**Birds after creation** (`think_bird`, `do_air_physics`)~~ — **read
  and modelled 2026-08-28, §3.9**: `think_bird` is three draws every
  eighth frame per live bird, and its landing search is sixty more on the
  frame a bird lands (2026-08-31).
  ~~`do_air_physics` stays open~~ — **read whole 2026-09-02**, §3.9 "The
  flight", and implemented in `crates/sim/src/air.rs`; what stays open is
  the *phase* of the orbit it flies, which is why the module is not called.
  `Guy::set_anim`'s bird branch stays open, and the 25 phase-7 wraps it
  leaves unspent are what still stands between the simulation's stream and
  the original's after frame 103 — and `do_air_physics+0x639` is what parts
  run33's word at **1802**, its one appearance in 1,851 frames. And ~~`Farms::add`/`add_animals` at a farm's
  creation — event-driven, unread past their draw sites.~~ **both are read
  and modelled: `add` at placement (§3.8) and `add_animals` at activation
  (§3.21).**
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

## 7. Coverage — what a diff backs, and what rests on a reading

CLAUDE.md's rule: a claim a diff against the original's dump has
confirmed needs no second reader, and a claim that rests on a reading
alone is what a blind reader is briefed with. Each row below says which
this document's mechanics are, and names the test that keeps the first
kind honest.

- **The pasture, §3.11 (2026-08-30).** Diff-backed, on two captures: the
  five's phases, their species and offsets, the `type_index` that reaches
  the gaia table, and the destination the one draw picks are all pinned by
  `run39_s_long_trace_says_where_the_second_map_s_word_parts` (**64 of the
  first 64 frames on the count and 64 draw for draw** since the pair
  landed) and by run20's own fifteen setup draws. Reading-only, and named
  as such: `corner_x`/`corner_y`'s **row order** — the trace confirms the
  five destinations are three distinct thirds of a tile, not which corner
  belongs to which slot, because every animal here sits on the farm's own
  tile. ~~And the `find_angle` argument, the *unsnapped* point
  (`5d7889`–`5d78a5`), which no capture separates from the snapped one.~~
  **The argument is diff-backed since §3.12 (2026-08-30)** — not on the
  animal, whose two points still round to the same eighth of a turn on
  run39, but on the rule: `add_move_order@00616ed0` takes the same
  unsnapped bearing, and run10's farmers separate the two by up to 10.5°
  on frames the dump prints.

- **The pasture's reference object, §3.6 (2026-09-03).** Diff-backed, on
  run53 and run69: the arrived count is what decides whether an animal
  measures its farm or its farm's gatherer, and reading the chain's length
  instead cost Great Lakes' word 1,311 frames — `run53_s_24000_frames_put_
  the_ceiling_where_run33_did` runs 2930 → **4241**, and
  `run69_s_three_thousand_frames_stand_where_the_original_s_do` goes from
  six units ever off the original's point to **none** over its whole 3,000,
  which is now asserted outright rather than only before the word. The
  sim's half, `think_farm_animal_s_phase_and_its_covers_tile`, makes all
  three states fail on purpose: off phase, on the chain but not arrived,
  and arrived off the footprint. Reading-only, and named as such: that the
  head `gather_down` names is measured **unfiltered** once the count is
  non-zero — no capture has a pasture whose chain head is not also its one
  arrived gatherer, so a chain of two would be needed to separate them.

- **The pasture's stocking, §3.21 (2026-08-31).** Diff-backed, on run33:
  `a_finished_pasture_stocks_five_animals_for_twenty_draws` re-derives the
  twenty draws of run33's frame 1372 from the original's own word
  (`0xc91f99f2` in, `0xafa38116` out) — the twenty labels in order, the five
  species and the five snapped points — and the whole-game check is
  `run33_s_long_trace_says_where_the_word_parts`, whose word runs to 1802 on
  it. Reading-only, and named as such: that a farm activated a **second**
  time is stocked a second time, which follows from `add_animals` guarding
  on the type rather than on a count and which no capture exercises — a
  captured pasture would show it, and no capture has one.

- **The animal's hurry, §3.13 (2026-08-30).** Diff-backed, on both
  captures and with no free parameter:
  `an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`
  re-derives `move_step`'s whole step for **every** gaia unit-frame on
  which an animal moved in run39 and run33 — **264 of 264**, 39 of them
  beyond `0x180` — from the `myspeed`, `orders_x/orders_y` and positions
  the dump prints, against two different cached speeds (19 and 11). The
  threshold, the ratio and the absence of the rule are each made to fail.
  Reading-only, and named as such: that `is_move` is what slot `+0x14` is
  (from the PDB's `LF_ONEMETHOD` list — the map's COMDAT folding names it
  `StrafeOrder::is_air`), and so whether an `AttackToOrder` or
  `ExploreToOrder` on an animal would hurry too; and that the air arm
  returns before the floor of 3 as well as before the hurry, which no
  bird in any capture is slow enough to show.

- **The blocked animal's dropped walk, §3.14 (2026-08-30).** Diff-backed,
  on both captures: `a_blocked_animal_drops_its_walk_where_it_stands`
  takes every animal walk in run39 and run33 that ends short of its goal
  — **sixteen**, against seventeen that arrive — and on every one the
  animal's position is unchanged across the frame the order dies, which
  is what `docs/COLLISION.md` §6 step 6's cell-centre snap would break.
  The sim's half is `collide.rs`'s
  `an_animal_drops_its_walk_where_it_stands_and_takes_no_step`, an animal
  and a player's unit walking into the same blocker from the same point,
  written to fail first. Reading-only, and named as such: that slot
  `+0x30` is `SubObjectData::is_animal` (the PDB's `LF_ONEMETHOD` list;
  the map folds both overrides onto trivial stubs), and the meaning of
  the `unit_masks & 0x4000000` the branch clears, whose two readers an
  animal never runs.

- **The pasture's herder, §3.15 (2026-08-30).** Diff-backed, on the one
  capture that has a pasture:
  `a_pasture_herder_walks_only_on_its_own_256_frame_phase` holds run39's
  dumped herder to **42** steps in **four** runs, each opening the frame
  after a phase frame, checks that all **seven** phase frames of the
  capture spend the `GameAccess::rnd` pair in the trace, and requires this
  crate's own herder to walk the original's first run frame for frame and
  to start no walk off its phase. The arm's absence, a halved modulus and
  the crop's own span each make it fail. Reading-only, and named as such:
  that the pasture's two moduli are `x_size / 2` and `y_size / 2` rather
  than some other halving — the listing is unambiguous (`005efdd8`,
  `005efde5`) but a 4 × 4 farm cannot tell `size / 2` from a literal 2;
  and what a herder does at a pasture whose footprint is not 4 × 4, which
  this install has none of.

- **The two arrival arms, §3.12 (2026-08-30).** Diff-backed, on one
  capture and six frames apart: run10's `0/3` arrives on 110 by the
  Manhattan snap and takes its order's angle under a gather; `1/3` arrives
  on 116 by a partial step and keeps its own bearing. Not established:
  whether any *other* action index than `GATHER` (7) reaches the snap
  arm's second clause — the check is a capture with a unit arriving under
  an `ATTACK` or a `BUILD_AT` while a second order is queued, which no run
  has yet.

- **A script `static` is not a frame slot, `docs/AI.md` §17
  (2026-08-30).** Diff-backed on all three captures: East Indies' word and
  sequence 274 → **413**, Great Lakes' weak totals 943/830 → **943/851**,
  and run10's roster the original's both ways for the first time. The
  operand's `0x40000000` bit and `Script::static_vars` are read from
  `VirtualMachine::get_value`/`set_value`; what a run cannot separate is
  seeding the frame at entry from mirroring only past-declaration slots,
  because every read in the shipped scripts is below the declarations.

- **The frame's two loops, §3.16 (2026-08-30).** Diff-backed on every
  capture the harness has, and it is the strongest row here because the
  order is not a parameter: the whole of run14's traced window is **284
  of 284 frames draw for draw** with it and 282 without, Great Lakes'
  *sequence* runs 99 → **576** and East Indies' word 219 → **274**, and
  the two rules it uncovered are each pinned by a record the dump already
  carried — the trained citizen's second draw at
  `Guy::set_anim+0x97a < Guy::inc_time+0x271` (run33's frame 99) and
  run13's `1/6` ending that frame at `0/232, last −1`. Made to fail three
  ways, all three met on the way in. Reading-only, and named as such:
  that the second loop's per-owner order is by *object number* rather than
  by the list's own order — every capture here has one building per owner
  drawing on any frame, so nothing separates them; the **walls'** third
  band (`3000..wall_mark`), which no capture reaches; and that
  `unit_masks & 0x40000` is exactly "a computer player's unit", which this
  crate stands in for with the owner's `human` flag and no capture has a
  case that separates (an AI-driven unit of a human player, or the
  reverse).

- **Two sites at one branch, §3.18 (2026-08-31).** Diff-backed, and by the
  strongest oracle this document has: the first half of
  `run39_s_woodcutters_reroll_on_the_chop_branch_not_the_arrival_s` uses
  **no simulation at all**. It matches every rise in a dumped
  `GATHERORDER`'s `wait` over run39's 1,850 frames to the value the
  trace's own draw returned on that frame, under that site's formula — 24
  rerolls, 19 at `+0x54b`, 5 at `+0xcc3`, none at `+0xdad` — so the sites
  are named from the original's two records against each other. The second
  half is the whole row against ours, 16,152 fields to frame 897. Made to
  fail two ways, both met on the way in: the formulas swapped in the
  naming table (half one, at run39's frame 529) and the arrival's formula
  back at the chop site (half two, the record's floor 897 → 530); the
  branch dropped whole is the word itself, 742 → 645. Reading-only, and
  named as such: that `+0xdad` is `% 50 + 100` at all, since no run
  reaches it; and `CHAR_MINE_ORE`'s early return together with the miner's
  1,000,000, because **every non-flat gatherer in every capture here is a
  woodcutter** — run39's `GATHERORDER`s all carry `build_type 418` and not
  one carries a mine's. *Capture, owed:* a mine worked for a few hundred
  frames with `UNITS=3`, which would settle both.

- **The bird, §3.9 (2026-08-28, its counter 2026-08-31).** Diff-backed in
  the only way a bird can be, since no dump prints owner 9 at all: the
  frames a landing search fires on.
  `a_bird_s_landing_frames_are_the_trace_s_own` holds this simulation to
  run39's own 576, 944, 944, 1016, 1144 and 1368 up to the word, and
  `run39_s_bird_lands_on_576_and_spends_the_search_s_sixty` holds frame
  576 draw for draw, all 118. Both are made to fail by dropping
  `do_air_patrol`'s `spell_time = 1` tail, which puts a seventh landing on
  1256. **The flight is diff-backed too, since run61**: proxying
  `do_air_physics` and the two functions it calls turns a bird's position,
  goal and bank into a per-frame record, and
  `run61_s_birds_fly_where_the_original_s_do` holds `crates/sim/src/air.rs`
  to 45,712 of them — ten birds, birth to frame 5,400, every position and
  every bank zero-crossing (§3.9, "The birth"). `air::tests` and
  `single::tests` still hold the module and its arithmetic to the reading
  and to the host's floats; run61 is what holds it to the original.
  **And the figure's arrival stand since 2026-09-07, §3.27**, which is a
  dump-backed row and not only a trace one: `Guy::move`'s arrival arm runs
  for a bird because `do_air_physics`'s `set_new_location(x, y, 0, 1)`
  leaves the figure a step behind its unit, and
  `run89_s_window_is_great_lakes_word_frame` holds run89's 7584 to 49
  draws for 49 with the stand at index 24 on both sides, closes `1/3`'s
  spot on block 7585, and leaves nothing parting at or below it.
  Reading-only, and named as such: **which cell** the landing search
  settles on — the score is inert (`-1 < score` cannot fail), which makes
  the answer "the thirtieth sample" and no capture yet confirms it; the
  `n <= 1` skip, since run39's region is larger; and `Unit::do_strafe`,
  which is the gull's.
