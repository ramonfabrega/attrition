# The AI

**Status: first reading in progress — 2026-08-24.** §2 (the driver: the
frame hook, the cadence, `plan_strategy`'s sweep, the step machine, the
goods picture, the make list, the sites, the orphan check) is **read**, by
the main thread from the full decompile export
(`~/ghidra-projects/decomp/`, `tools/ghidra/`), every function named there
read end to end. §3–§10 are still the survey of the same day (grep, heads
and tails, PDB layouts) and say so; two Opus readers are deriving the
script language (`~/ghidra-projects/reports/ai/bhs-language.md`) and the
host functions (`…/host-functions.md`) in parallel, to be ratified here
before anything is built on them. Confidence on §2: **high** on the
cadence, the step machine and the make list's mechanics; **medium** on the
census's per-unit classification, which reads eight virtual slots the
vtable export mislabels (§2.10) — named by use, to be settled in the
listing where implementation needs them.

Read with `docs/ORDERS.md` (§2.4 `think_scout`, §5.9 `think_peasant`, §9 the
start of a game) and `docs/INPUT.md` §1 (why the AI is not in a recording).

---

## 1. Shape of the mechanic

The original's AI is **four engine layers and a script**, each switchable by
a scenario function, and the switches name the layers
(`ScenarioFuncSet::{enable,disable}_{production,combat,city,unit}_ai`):

| layer | switch | where it lives | entry | size |
|---|---|---|---|---|
| **production** (economy, build order, research, upgrades, market) | `leader_flags2 & 0x4` | `Leader` | `Leader::plan_strategy` → `production_ai` | ~9k lines |
| **city** (founding cities) | `leader_flags2 & 0x10` | `Leader::found_cities` reads the bit at its head | inside `production_ai` | 281 |
| **combat** (armies) | `leader_flags2 & 0x8` | `Armies` / `Army` | `Armies::process_all` → `Army::process` | ~5k |
| **unit** (idle behaviour) | `leader_flags2 & 0x2` (all), or per unit `Object+0x68 \| 0x1000000` | `Unit::think` | the idle path | **already read**, `docs/ORDERS.md` §2.4/§5.9, `docs/COMBAT.md` §8.1 |
| **script** (the opening) | `prod_script_run` | `ai/scripts/*.bhs` via `RunTimeEnv` | `production_ai` step 1 | 2,400 lines of script + the VM |

The bits are from the decompile of the switches
(`enable_production_ai@009ff5e0` clears `0x4`, `disable_combat_ai@009ff860`
sets `0x8`, `disable_city_ai@009ffbf0` sets `0x10`, `disable_all_unit_ai@
009ff8e0` sets `0x2`, `disable_unit_ai@009ffa10` sets `0x1000000` on every
unit of the chain). `leader_flags2` is `LeaderData+0x4` (PDB); `leader_flags`
at `+0x0` has bit 1 = active, bit 2 = computer (`strategy_all`'s `& 3 == 3`),
bit 4 = human (`docs/audit/2026-08-23-pathfinder.md`).

**Diplomacy** (`Leader::diplomacy`, 3,282 lines, plus `action_respond` 639,
`consider_tribute`, `Diplomacy::*`) is part of the production sweep but is
scoped out of the first pass: a two-player skirmish against one AI at war
from frame 0 does not exercise it before the armies do.

## 2. The driver — read

### 2.1 The frame hook

`Game::do_frame@00591ef0`, in order (decompile line numbers):

```
199  Leaders::process_all        income, stats, elimination, taunts (Leader::process — no AI in it)
267  Leaders::strategy_all       gated: (semaphore[1] & 8) == 0   ← the production AI
270  GameDaemon::process_all
272  Armies::process_all         gated the same way               ← the combat AI
275  Objects::process_all        every unit steps its orders (ORDERS.md §2.1) — the unit AI is inside
279  Leaders::end_process_all
288  Leader::process_event_frame  per active leader
```

`Leaders::strategy_all@006ed430` (read): for every leader whose
`leader_flags & 3 == 3` (active **and computer**), **every frame**, in this
order: `check_explore`, `plan_strategy`, `compute_score(0)`, `diplomacy`.
Then `Game::check_victory` if `semaphore[1] & 2`. A human leader never
enters `plan_strategy`; the "human gate" inside `production_ai` is for the
`human <who>` console switch that flips a leader mid-game.

`Leader::check_explore@006bc860` (read, 47 lines): at frame 0 and whenever
`(who·25 + frame + 12) % (200 / ai_speed) == 0`, recount `explored`
(`+0x9d4`): the whole map's cell count (`world+0x2c`) with
`EXPLORE_MAP_BONUS`, else the number of cells whose visibility byte
(`world+0x160` row stride `world+0xc`, byte 3 of each 4-byte cell) has bit
`who` set. Pure recount, no decision.

### 2.2 `plan_strategy`'s cadence

`Leader::plan_strategy@006b9620` (read, 1,754 lines), head:

```
phase = (who * 25 + frame) % (200 / ai_speed)      # ai_speed: 1 from Game::init_data, +1 per `ai speed increase` cheat
if production_step != 0:  production_ai(); return   # the step machine pre-empts everything, every frame
if frame != 0 and phase != 0:
    if phase % 30 != 0: return
    t = make_list[0].t                              # the top of last sweep's shopping list
    if t - BASE_AGETYPES > 0x54: return              # only an age or a tech (0x220 ≤ t ≤ 0x274)
    if !can_pay(0) or has_tech(t) or researching(t, -1, 0, 0): return
    make_this(0); return                            # the cheap tick: buy the top research when it becomes affordable
… the full sweep (§2.3) …
```

So a computer leader takes the **full sweep at frame 0** (every leader,
regardless of phase) **and then once every `200/ai_speed` frames on its
own phase** — `who = 1` at 175, 375, 575 … — and in between, every 30
phase-frames, a research-only tick off the top of the make list. The sweep
ends by arming `production_step = 1`, and the step machine then runs one
step per frame (§2.4) until it disarms itself. For `who = 1`: sweep at 0,
steps on 1, 2, 3 …; sweep at 175, steps on 176 …. Because `production_ai`
is checked *before* the phase test, a step machine still running when the
next sweep frame arrives delays that sweep to the next frame on which it
has finished — the phase is a floor, not a schedule.

### 2.3 The sweep: `plan_strategy` is the census

Everything after the cadence check is one long recount into `LeaderData`,
in this order. Field names are the PDB's (`struct /rise.pdb/LeaderData`);
`Leader` is an unpopulated shell over it, so the decompile prints
`field_0xNNN` for all of them. "Region" is the tile's `tregion`
(`world+0x134` tile record, short at `+4`; `docs/CITIES.md` §2.3); land
regions are `0..0x3e`, sea regions `0x3f..0x7e` and are stored `% 0x3f` in
the 63-entry arrays.

1. **Escrow rate.** If `city_num + village_num > 2`: `escrow_rate[0..5] =
   40`.
2. **Per city** (active): `free = busy = gatherers = 0`, `peasant_dist =
   100`. Later, `in_port = 0`.
3. **Zero the census**: `active, combat, non_siege, sea_combat, siege,
   defense, attack, naval, air, missile, transports, fishermen,
   idle_fishermen, peasants, scholars, caras, merchants, fighters, bombers,
   cruise, nuke, free_peasants, xport_peasants, gatherers, attacked,
   full_cities`. **Not** zeroed here: `control`, `pop`, `scouts` (kept by
   the unit lifecycle), the `*_high` maxima, `reg_pop`, `reg_forts`,
   `reg_docks`, `reg_terr`, `reg_buildings` (kept by `gain_/lose_building`).
4. **Territory.** `my_team_terr = get_team_terr()`; over every other
   computer leader not allied both ways (`diplos[i] != 2` on either side):
   `other_team_terr = max`, `min_other_team_terr = min` (0 means unset).
5. `filled_gather_slots[0..5] = 0`; `ally_mask = 1 << who`, or'd with
   `ScenarioData::ally_mask[who]` in a scenario/CtW game.
6. **Meeting** (only with `ALLY_LOS` or `reveal_map`): for every other
   computer leader, `ally_mask |= bit` if allied both ways; and if not yet
   met (`treaties[i] & 1 == 0`), scan their units then buildings for one
   `is_seen(who)` (vslot `+0x48`) — the first hit sets `treaties` bit 1 on
   both sides and `LeaderOut::say_meet` for the console player. Diplomacy's
   edge; noted, not this mechanic.
7. `invaders[who] = 0` on every active leader (the count of *my* units in
   *their* land; refilled in step 10).
8. **Zero the per-region arrays**: `reg_active, reg_combat, reg_attack`
   (127), `reg_naval, reg_transports` (63), `reg_defense, reg_attacked,
   reg_land, reg_peasants, reg_free_peasants, reg_xport_peasants,
   reg_gatherers, reg_gather_slots, reg_known_rares, reg_unpack_merch`
   (64); and **`reg_cities[r] = reg_buildings[r][0] + [1] + [2] + [0x75]`**
   — the four city types relative to `BASE_BUILDTYPES` (Small, Large,
   Major, and `0x213`).
9. **Rares** (`new_rares`, the leader's list of rare-resource objects):
   each rare I can see (its tile's `explored` byte has my `ally_mask` bit,
   or the game reveals all, or `leader_flags & 0x800`, or `+0x59e4`) is
   marked seen for me (`good+0x20 |= 1 << who`) and, unless its tile's
   owner is an enemy, `reg_known_rares[tregion]++`.
10. **The unit census** — every captain of mine (`is_captain`, vslot
    `+0xe8`) that is alive and whose type has `control_cost != 0`:
    - its region: the unit's tile, or when garrisoned, the containing
      building's region — the first land tile scanned across the
      building's footprint (`x_size`/`y_size`, centred by parity) for a
      non-water tile (`world+0x138` terrain `& 0x30 == 0x20` selects the
      tile record's alternate region short at `+6`);
    - `active++`, `reg_active[r]++`; strength `s = attack() / 10`
      (vslot `+0x120`), or `10` for a sea/air domain;
    - `role & 0x10000` (military): `attack += s`, `combat++`, then
      `siege++` or `non_siege++` by vslot `+0x10c`; `reg_attack[r] += s`,
      `reg_combat[r]++` (the two `reg_*` only when the unit's domain
      matches the region's kind, or it is air);
    - domain 1 (sea): `naval += s`, `sea_combat++` if `s`, `transports++`
      if `carry`, `fishermen++` if `unit_flags2 & 4` (+ `idle_fishermen`
      if `unit_masks & 0x80000`); sea regions: `reg_naval`, `reg_transports`;
    - domain 2 (air): `obj_masks & 0x8000000` → `missile++` and
      `cruise`/`nuke` by type `0x139`/`0x13b`; else `air++` and
      `fighters`/`bombers` by type `0x11f`/`0x130`;
    - a non-siege military unit standing on a tile owned by another
      active leader: `leaders[owner].invaders[who]++`;
    - by base type: `0x34/0x35` (scholar) → `scholars++`, and if inside a
      university (`0x1a4`) `filled_gather_slots[3]++`; `0x3d/0x3e/0x190`
      (merchant) → `merchants++`, `reg_unpack_merch[r]++` unless
      `unit_masks & 0x80000`; `is(0x3b)` (caravan) → `caras++`;
    - **citizens** (`0x32/0x33`): `peasants++`; nearest friendly city
      within `0x200` (`ObjectsData::find_city`, its distance in
      `objects+0x1fc`). Garrisoned: inside a `0x1a6` → `filled_gather_slots
      [5]++`; inside a sea-domain building with a building-targeting order
      → `reg_xport_peasants[r]++`, `xport_peasants++`. On the map:
      `reg_peasants[r]++`; if `reg_cities[r] == 0` and the action is not
      BUILD → `reg_xport_peasants[r]++`, `xport_peasants++`; then by the
      **action kind** (`get_action`, vslot `+0x10`; kinds per
      `docs/ORDERS.md` §3): none or `3` → `free_peasants++`,
      `reg_free_peasants[r]++`, `city.free++`, `city.peasant_dist =
      min(city.peasant_dist, dist / 0x300)`; otherwise `city.busy++`, and
      `GATHER (7)` → `filled_gather_slots[good]++` by the target
      building's type (`0x1a1` food, `0x1a2` wood, `0x1a3` metal,
      `0x1a5/0x1a6` oil; knowledge is the university above), `city.busy--`
      and the *target building's* city `busy++`, `gatherers++`,
      `city.gatherers++`, `peasant_dist` as above, `reg_gatherers[r]++`;
      `8` or `0xe` (build/repair through a boarded transport) →
      `reg_xport_peasants`, `xport_peasants++`.
11. **The building census** — every active building of mine (the build
    and wall lists): a finished gather building (vslot `+0x90` on its
    type) adds its `gather_max` (`Build+0x80`) to `gather_slots[good]`;
    every active building with `type+0x1e8` set counts `defense += 1` for
    a tower (`is(0x1b7)`) or `2` for one that has arrows (vslot `+0xfc`),
    and `reg_defense[r]` the same.
12. **Maxima**: `gather_slots_high[g]`, `peasant_high`, `scholar_high`,
    `caravan_high`, `merchant_high`, `army_high (= max combat)`,
    `city_high`, `village_high`, `population_high` — each `max(old, new)`.
    `resources_controlled = bit_count(the rares bitmask at +0x6d9c)`.
13. **Per-city site picture** (`compute_site_stats` proper is §2.7's; this
    is the sweep's own): `city_pop_high = max(city.pop)`; radius = `(level
    − 1) × city_center_pop_radius` with level 1/2/3 for a Small/Large/Major
    or `0x213` city, `+ indians_city_radius` with tribe bonus `0x15`, capped
    at `0x40`, then `circle_radius[(radius + 2) / 4]` picks the even circle
    (`docs/CITIES.md` §3.6); zero `ocean, land, filled, dock_tile, space[3],
    ter[6]`; `reg_gather_slots[r] += City::count_gather_slots(city)`; a city
    with flag `2` (attacked) → `attacked++`, `reg_attacked[r]++`. Then over
    the circle's tiles that are mine or unowned: a water tile → `ocean++`
    and, if its region has `> 1` coasts or is ≥ a tenth of the map,
    `dock_tile++` for the first `is_dock_tile`; a land tile inside the
    inner radius and in the city's own region: unoccupied (`flags & 0x70 ==
    0`) → `land++`, `check_building_wcoord` gives the largest square that
    fits there → `space[n − 2]++` for `n = 2..4`, and `filled++` if `< 4`;
    occupied → `World::gather_at` and `ter[g] = max(ter[g], amount)`. After:
    `land − filled < 2` → `full_cities++`; `reg_land[r] += land − filled`.
14. **Wars/allies**: over every other met computer leader: `diplos` 0 on
    either side → `wars++`; 2 on both → `allies++`.
15. **Per-region strategy** (`strategy[r]`, a bit word per land region;
    `reg_wars/reg_allies/reg_neutrals[r]` zeroed and recounted): bit 1 =
    the region is *thin* (all leaders' `reg_cities[r] × 50 < region.size`).
    If I have a city there: for every other met leader with a city there —
    at war: `active_wars++`, `active_wars_with |= 1 << i`, `reg_wars[r]++`,
    and **bit 4 if I am weaker** (`my attack < theirs` and (`r == home_reg`
    or `reg_cities[r] < theirs`)) else **bit 2**; then difficulty `< 2`
    forces bit 4, and `starting_resources == 8` forces bit 2; allied both
    ways → `reg_allies[r]++`; else `reg_neutrals[r]++`. Then by
    `world+0x34`: `< 1` clears bit 8; `== 2` and no war/ally bits → bit 8
    if some *other* region with region flag 8 has no city of anyone; `≥ 3`
    and no war/ally bits → bit 8.
16. **Army seeding** (computer leaders only): count my armies (16 slots per
    leader) that are active (`Army+0x4 & 1`) or flag `0x20` in this region;
    if fewer than two, score every city of mine in the region that has no
    active army assigned: `level × 20 × ((barracks + siegeworks + stables)
    × 4 + 1 + docks)`, `/ 3` if the city is flagged attacked, `× 12` if no
    army of mine is near (`Armies::find_army < 0`) else `× (find_dist / 4 +
    6)`; the best → **`Armies::init_army(who, city)`** — the lowest-numbered
    free slot, or the one with the smallest `+0x10`, re-`init`ed. This is
    where armies come from; `docs/ARMY.md` will own `Army::init`.
17. **Tail**, unless `semaphore[1] & 2`: `check_orphaned_buildings()`
    (§2.8), `compute_sites(0)` (§2.7), **`production_step = 1`**.

Two sync-stream facts: the sweep itself draws no `game_random`; `compute_
sites` in its tail does (§2.7). And `check_orphaned_buildings` can issue
orders, so `docs/ORDERS.md`'s "the frame-0 sweep issues no order" (audit
F4) is true only because a fresh game has no orphan sites.

### 2.4 The step machine — `production_ai@006c1960`

Read whole (114 lines). Bail to `production_step = 0` when `leader_flags &
4 && !(leader_flags & 8)` (a human that is not being AI-driven), `ai_off`
(the console's `ai off`), or `leader_flags2 & 4` (the scenario switch).
Otherwise `effective_pop = queued_units() + control + 1`, then:

```
if step == 1 and (prod_script_run == 0 or starting_resources == 8): step = 2
switch step:
  1:  push script_step (by ref), 5, pers.rush + 2, who + 1        # (who, ref step, boom_vs_rush, num_loops)
      r = run_script(script_run_time, prod_script)
      if r == 0 (ran):  ret = get_ret_int(); script_step = <the ref read back>
                        ret == 1 (BLOCK_ON_THIS) → step = 0; return
                        ret == 3 (SCRIPT_DONE)   → prod_script_run = 0     # never again
                        (anything else, incl. 2) → fall through
      else (compile/run error, r ≠ 0): prod_script_run = 0                  # the script is dropped, silently
      step += 1; return
  2:  step = 3; production_ai_setup(); make_list.clear(); return
  3:  step = 4; found_cities();     → tail
  4:  step = 5; research_techs();   → tail
  5:  step = 6; upgrade_units();    → tail
  6:  step = 7; create_units();     → tail
  7:  step = 8; create_buildings(); → tail
  8:  step = 9; if make_stuff() != 0: return           # bought something: stay armed, run 9–11
               if starting_resources == 8: return
               step = 0; return
  9:  step = 10; create_units();    → tail
  10: step = 11; create_buildings(); return
  11: make_stuff(); step = 0; return
tail (steps 3–7, 9): if starting_resources == 8: make_stuff(); make_list.clear()
```

So the ordinary lobby runs 1–8 and stops unless step 8 bought something,
in which case a second pass 9–11 (units, buildings, make) follows; the
`starting_resources == 8` lobby (the "unlimited"-style row) skips the
script and buys after every producer step. Steps 3–7 only *fill* the make
list; step 8 (`make_stuff`) is the only place in the ordinary path that
spends. `found_cities` is the "city AI" — it returns at once when
`leader_flags2 & 0x10`.

The script call, precisely: four `ScriptInt`s popped from the recycler and
pushed as `(script_step, tag 3 = ref)`, `(5, tag 1)`, `(pers.rush + 2, tag
1)`, `(who + 1, tag 1)`; `run_script` is `RunTimeEnv::run_script@009c4460`
(reads §3); on success the first `ScriptInt`'s value (`script_step`) is read
back through its vtable and the four are released. `run_script`'s return
`0` is success; it returns `3` when the function is not found, `4` on a
parameter mismatch (`check_params`), `6` on a runtime error — any of which
**disables the script for the rest of the game** (`prod_script_run = 0`)
and advances to step 2 as though the script had returned 2.

`Leader::init` sets `script_step = 1`, `production_step = 0`,
`prod_script_run = 0` (then `1` for a computer leader, §3), the five
`*_mod` fields to `0x100` (`wonder_mod` 0), `pop_cap`, `misery`, `att`
etc. to 0.

### 2.5 The goods picture — `production_ai_setup@006c83e0`

Read whole (316 lines). Writes `econ[6]` (a flag word per good),
`worst_good`, `best_good`, `shortages`, and on easy difficulties lowers the
resource caps. The encrypted block is `LeaderDataEncrypt` (`data_encrypted`,
`+0x6eb8`; fields XOR'd with per-field keys — `resources ^ 0x90236`, `rate ^
0x73862`, `resource_cap ^ 0x8221`, `ages ^ 0x62766`; `docs/ECONOMY.md`):

1. `shortages = worst_good = best_good = 0`. **`starting_resources == 8`**:
   every `econ[g] |= 8` and return.
2. **Difficulty** `d` = `multi_diff` when `semaphore[0] & 4`, else the
   lobby's `difficulty` unless a multiplayer-AI semaphore pair says
   `multi_diff ≥ 0` (three identical reads; `get_diff` is the same
   choice). If `d < 3` and my current age (`ages ^ 0x62766`, as a type
   `0x220 + age`) is an age type: `m = max over available goods of
   get_cost(age_type, g, who, −1, −1, 0, 1, −1)`, at least `300`; `d == 0`
   → `m × 3/2`, `d == 1` → `m × 2`, `get_diff() == 2` → `m × 5/2`; then for
   each good with `resource_cap > m`: `escrow[g] = escrow[g] × m / cap`,
   `cap = m`. (The age type's per-good cost is the next age's price, so an
   easy AI cannot bank more than ~1.5–2.5 ages' worth.)
3. **Rate pass**: for each good, `econ[g] = 0`; `r = min(get_mod_
   resource_cap(g), resources[g]) / 16` is stored into `rate[g]` (XOR'd);
   for available goods, `rate[g]` (the *stored* one, i.e. the income
   figure `do_gather` maintains) picks `worst_good` (min) and `best_good`
   (max), and `rate < 30` → `econ[g] |= 1`, `shortages++`.
4. **Stock pass**: thresholds `lo = min(city_num × 15, cap/32, 250)`, `hi
   = min(city_num × 30, cap/16 × 4/5, 350)`; with more than four cities `lo
   = cap/32`, `hi = min(cap/16 × 3/4, 175)`; for goods other than food and
   wood while `city_num < 3`: `lo = 20`, `hi = city_num × 20`, and the
   shortage bit is cleared (`shortages--`). Unavailable good → `econ = 0`.
   Otherwise, with `resources < cap`: `< lo` → `|= 2`; `< hi` → `|= 4` and
   done; else (or at cap) `|= 8`.
5. **Food/wood balance** while `city_num ≤ 2`: if food is not "low" (bit
   4) or the tribe has bonus `0x13`: when wood is low, clear food's bit 4
   and metal/knowledge/wealth/oil's; else (food low, no bonus): clear
   wood's bit 4 and the other four's. Same shape for bit 2 → promoted to 4.
   Net effect: with two cities the AI only ever calls *one* of food/wood
   short at a time, and never the later goods.
6. `market_speculation()` (109 lines, not yet read — `docs/ECONOMY.md`'s
   market).

### 2.6 The make list — `MakeList`, `make_stuff`, `make_this`

`make_list` (`+0x6ec8`, `Array<MakeObject>`; `list` at `+0x10` is what the
decompile prints as `field_0x6ed8`) holds **eleven** `MakeObject`s (`0x28`
bytes: `t, val, escrow, city, up, o, num, cat, wx, wy`) — a ranked
shopping list the producer steps fill (`docs`: §4's reading) and
`make_stuff` spends from. `t = −1` is an empty slot.

**`make_stuff@006c8af0`** (read, 305 lines), returns 1 when it bought the
head:

1. `head = list[0]`; `head.t == −1` or `head.val == 0` → return 0.
2. `use_market()`; `paid = can_pay(0)`; `cost[g] = get_cost(head.t, g,
   who, head.o, head.city, 0, 1, −1)` for the six goods.
3. If `paid`: `make_this(0)` (§ below) and remember whether it succeeded;
   else if `resources[3] < cost[3]` — knowledge short — treat as `paid`
   for the rest (do not save up for it).
4. **Expiry of the head** (and every duplicate of its type in the list):
   each slot whose `t == head.t` is cleared with probability **1/3** —
   `Random::get(game_random, 0, 0xffff) % 3 == 0` — **or unconditionally**
   when the head is a tower/`0x1bb`/an upgrade-kind type (vslot `+0x64`) or
   a wonder, or is *not* a military unit type (`is(0x1a4)` or not a unit
   type or a peasant type). One sync draw per matching slot, every
   `make_stuff`, whether or not anything was bought.
5. **Saving**: if not `paid`, `need = average over available goods with
   `cost[g] > 0` of `(cost[g] − resources[g])`, floored at 0` — the mean
   shortfall.
6. **The rest of the list**, slots 1..10: skip empties and `val == 0`;
   from slot 4 on, skip a slot whose `(t, city)` duplicates slot 0, 1, 2 or
   3 (three duplicates are tolerated); then for each available good with
   `cost > 0`: if `resources[g] < cost_of_slot[g] + cost[g] + need` the slot
   is unaffordable-while-saving → skip, **except** slot 5 when I have no
   free peasants and no gatherers (buy anyway), and slot 4 when it is a
   gather building (`build_flags & 0x40`) for a good whose `econ` has bit 4
   — buy anyway; slots that pass: `can_pay_cost(who, city, o, escrow)`
   (vslot `+0x84`) `≥ num` → `make_this(slot)`, then the same 1/3 expiry
   over duplicates of *its* type (unconditional for tower/`0x1bb`/upgrade/
   wonder), only while `slot < 11`.
7. Return: `site_mark++` if nothing in the pass was an upgrade-kind;
   return "bought the head".

**`make_this@006c94f0`** (read, 187 lines) dispatches one slot by its
type's kind: a **build type** in `0x19e..0x21e` → `type_avail == 4`
required (else `produce_tech(t, escrow)` — i.e. the tech that unlocks it
is what gets bought); `up != 0` → `produce_upgrade(t, city, escrow)`; a
city type (`is(0x60)` vslot) → `produce_city(t, wx, wy, escrow)` unless the
site's region has no free peasants and no gatherers of mine; a wonder or
ordinary building → `produce_building(t, city.o, escrow)` unless the
city's region has no free peasants/gatherers, or the building is a land
building that is not a gather building and the city has `< 2` open tiles
(`land − filled`) with a footprint that fits (`space[max(x, y) − 2]`
clamped to `0..3` `< 2`); a **tech** in `0x220..0x274` →
`produce_tech(t, escrow)`; a **unit** in `0x32..0x19d` → `type_avail == 4`
→ `produce_unit(t, city, num, escrow)`, else `produce_tech`; a **spell**
`0x275..0x2ab` → `produce_spell(t, city, escrow)`. A `produce_*` returning
0 (bought) → `slot.val /= 100` (demoted a hundredfold, not cleared).
`sys.ai_logging` bit `who` gates on-screen messages only.

### 2.7 The sites — `compute_sites@006cc950`

Read whole (308 lines). `sites` (`+0x6e34`, `Array<Site>`, list at
`+0x6e44`) holds **ten** `Site`s (`wx, wy, val, reg, dist, rank`). At frame
0 all ten are zeroed. Otherwise each site with `rank ≥ 6` and `val ≥ 1` is
re-scored in place (`compute_site_stats` with `keep = 0`, which may move
`wx, wy`) and the rest are cleared. Then, for a computer leader (or when
called with `1`, as `ScenarioFuncSet::place_city_with_cost` does): for
every region where I have peasants (`reg_peasants[r] != 0`): the reference
city is the nearest friendly city within `0x200` of the region's first
listed tile (or, with no city of mine there, the nearest citizen); the
region's tile list (`Region+0x7c`, `size` at `+0x14`) is sampled with
**stride 2**, or for a region of 200+ tiles: **`stride = max(10, size /
(world_w − rand % world_w / 2))`** and **start offset `site_mark % stride`,
then `site_mark += rand % (stride / 4) + 3`** — two `game_random` draws
per large region, the first skipped when `world_w ≤ 1`, the second when
`stride / 4 ≤ 1`. Each sampled tile is scored by `compute_site_stats(…,
keep = 1)` and inserted: a tile already present with `val > 0` is skipped;
otherwise it takes the slot with the smallest `val` among those it beats
(the loop keeps the running minimum and its index; an empty slot's `val` 0
is always beaten). Finally `rank[i] = 1 + count of sites with val ≤ val[i]`
over the ten. `compute_site_stats` (506 lines) — the site score itself —
is §4's reading. **`compute_sites(0)` runs at the end of every sweep,
including frame 0, so the sync stream takes these draws at frame 0 for
every computer leader** with a 200-tile region.

### 2.8 The orphan check — `check_orphaned_buildings@006c9f20`

Read whole (294 lines). Computer leaders only. For every building of mine
(build and wall lists, in id order):

- A building whose `build_masks & 1` (`+0x60`) is set gets it cleared, and
  if it is not a city (`+8 & 0x20`) and is a tower/`0x1bb`/`0x1bf`, or its
  type has `+0x1e8`, or I own more than one of its type (`num_buildings[t]
  > 1`): **disband it** (`Group::action_disband`). The bit is the "sell
  me" mark some other path sets (`Leader::lose_building`? — not traced).
- An **unfinished site** (not `is_active`): find a citizen of mine whose
  action is `BUILD (6)` on this site; if its region differs from the site's
  and the site is a land building, `Unit::clear_orders` on that citizen and
  skip. If no builder: `job_counter == 0` (never started) and my territory
  at the site `> 0` → **disband the site**; territory `≤ 0` → the first
  idle (`action == 0`) or gathering (`7`) citizen in the site's region is
  sent with `Group::action_swarm_around(site, QUEUE_NEW, BUILD_AT, 1)`; if
  none, `build_masks |= 0x2000`.

### 2.9 What the driver leaves to §4

`compute_site_stats` (506), `found_cities` (281), `research_techs` (689),
`upgrade_units` (344), `create_units` (1,674), `create_buildings` (1,728),
the `produce_*` family, `check_income`, `use_market` (147),
`market_speculation` (109), `queued_units`, `unit_prod_value` (114),
`check_transport` (124). And `Leader::init`'s AI-relevant fields beyond
the ones named in §2.4.

### 2.10 Slots named by use, to settle in the listing

The vtable export prints a colliding name for these; the decompile's own
direct calls (`is_captain`, `is_on_map`, `ObjectData::is`, `WallData::
is_active`, `TypeData::is_*`, `get_cost`, `can_pay_cost`) are trusted.

| slot | on | used as | evidence |
|---|---|---|---|
| `+0xc` | Object | `is_active` | Build's slot is `SubObjectData::is_active`; Object's is purecall |
| `+0x10` | UnitOrder | `get_kind` | `docs/ORDERS.md` §3 |
| `+0x48` | Object | `is_seen(who)` | `UnitData::is_seen` on Unit |
| `+0xac`, `+0xb0` | Object | the object's `Build` view (`+0x18` type, `+0x72` city, `+0x80` gather_max follow) | use only |
| `+0x10c` | UnitType | `is_siege`-like split of military units | use only |
| `+0x120` | Object | `attack()` (`UnitData::attack`, `BuildData::attack`) | export |
| `+0x60` | Type | `is(TypeIndex, flag)` — `TypeData::is` | export |
| `+0x64` | Type | an "upgrade-kind" predicate (`make_this` routes it to `produce_upgrade`) | use only |
| `+0x90` | BuildType | "is a gather building" (adds `gather_max` to `gather_slots`) | use only |
| `+0xfc` | BuildType | "has arrows" (defence 2 vs a tower's 1) | use only |

## 3. The finding: the skirmish opening is the shipped script

`CLAUDE.md` says the build order is "scripted in the open under
`game/ai/scripts/`" and, elsewhere, that the 2002 scripting VM is what we
are here to escape. Both are true and they collide, because the decompile
shows the scripts are not scenario material — **they are the skirmish AI's
opening, for every computer leader, every game**:

1. `Leader::init@006e3930` lines 777–830: for every leader that is not
   human (`leader_flags & 0xc != 4`): `prod_script_run = 1`;
   `random_personality()`; then `prod_script =` one of two
   `int_str_array` entries — `+0x169b8` (index 4630, **`economic`**) when
   `pers.rush < 0`, or `pers.rush == 0` and a `game_random` coin says so,
   and the tribe lacks bonus `0x13`; else `+0x169cc` (4631, **`defensive`**)
   and `pers+0x4c early_army = 1`. (The patch-version `< 9` branch inverts
   the coin's sense; this install is later.) The indices are the
   `internal_strings.xml` records `economic`, `defensive`, and 4632/4641
   `.\ai\scripts\` (`Leaders::prod_script_path`) — `rondata` can re-derive
   them.
2. `Leaders::init_production_script@006ed490`: for each computer leader
   with `prod_script` set and no save being loaded,
   `Compiler::compile(prod_script_path + prod_script + ".bhs",
   SCRIPT_RELOAD_FULL)`.
3. `production_ai` step 1 (above) pushes four `ScriptInt`s and calls
   `RunTimeEnv::run_script(script_run_time, prod_script)` — which finds
   the script function of that name and executes its bytecode
   (`RunTimeEnv::exec` → `VirtualMachine::execute_next` per opcode) — then
   reads the return and the `ref step` back.

`economic.bhs` is `int ai economic(int who, ref int step, int boom_vs_rush,
int num_loops)`; `defensive.bhs` is the same signature under `defensive`
("rush script"); both `include "aibestbuildlibrary.bhs"` (650 lines:
`city_placement`, `place_woodcutter`, `train_unit_with_need`,
`assign_idle`, `woodcutter_check`, `place_dock`, `place_mine`,
`place_farm`). Their `labels { BLOCK_ON_THIS = 1, DONT_BLOCK_ON_THIS,
SCRIPT_DONE }` are exactly the return codes `production_ai` switches on.
The scripts are a step machine on `ref step` with per-`who` **shared
statics** (a `static` in a script function is one slot for all eight
leaders — hence the hand-rolled `switch (who-1)` "ghetto array" in
`economic`), and `static int needed_techs = get_techs_per_age(who);`
initialises from a host call.

**What the scripts need from the engine**: 55 host functions, every one
resolving 1:1 by name to a `ScenarioFuncSet::` method (`num_cities`,
`place_building_with_cost`, `num_type_with_queued`, `find_build_at_city`,
`train_unit_with_need`'s inputs, `have_tech`, `research_tech_with_cost`,
`can_pay_cost`, `find_city_with_num`, `age`, `get_mapstyle`, timers, …) plus
`MathUtilFuncSet::rand_int` = `Random::get(game_random, lo, hi)` — **the sync
stream**. The one call that is not a host function, `enable_trigger`, is a
compiler intrinsic: a `trigger name() { … }` block inside a function
compiles to `OP_JUMP_IF_BITSET` over a per-script bit
(`Script::toggle_trigger`, `is_trigger_enabled`) — inline code guarded by a
flag, not a scheduler. Two such blocks exist, both in `city_placement`.
Language surface actually used: `int`/`String`/`static`/`ref`, `if`/`else`,
`for`/`while`/`switch`/`case`/`break`, `++`/`--`/`+=`, `&&`/`||`, string
literals as type names, `labels`, `include`, implicit globals (`my_capital =
…` undeclared). No floats, no arrays, no user types.

**The VM, if it were to be read**: `VirtualMachine` (9 functions, 1,109
lines; `execute_next` 667), `OpCode` (513), `SymTable` (1,479), `Compiler`
(14, 1,360), `RunTimeEnv` (22, 1,390; `run_script` 129, `check_params` 266,
`call_script` 80), the `Script*` value classes (~3,000), and a flex lexer
(skip). Around 5k lines for the semantics that matter — integer
arithmetic and division, evaluation order, short-circuiting, `static`
initialisation timing, `ref`, how `String` compares — versus 35k for the
whole `ScenarioFuncSet` (872 methods, `init_funcs` 4,080 lines of
registration), of which 55 are needed.

## 4. The fork this opens

Three ways to have the opening, in the order of the project's own rules:

- **(a) A BHS interpreter of our own, reading the `.bhs` files from the
  install.** The scripts are shipped data like the XML tables — "nothing
  from the user's install ever enters this repo" makes them *data to load*,
  not source to translate — and this is the only option under which a
  modder's edited script still drives our AI. Our interpreter is a
  tree-walker over the language above (~1.5–2k lines of Rust), not a port
  of the bytecode VM; the reading of `VirtualMachine`/`Compiler` is for
  the semantics list in §3, not for its design. Cost: the interpreter plus
  55 host functions, each a small predicate or action over state the sim
  already has (cities, sites, queues, techs, costs).
- **(b) Transcribe the two scripts into Rust.** Cheaper to start, and
  wrong twice: it copies shipped content into the repo (the legal line),
  and it freezes the one part of the AI the original left open.
- **(c) Skip the script** — start `production_step` at 2 as the
  `starting_resources == 8` branch does — and take the C++ layers only.
  Reproduces nothing the oracle shows for the first 1,732 frames (§5) and
  is not what the original does in any lobby but one.

**Recommendation: (a).** It is the rule-consistent one, it is the one the
oracle can score, and it is smaller than it looks: the language is C
without pointers, the VM is 5k lines to *read* and 0 to *port*, and the
host surface is 55 named functions over existing state.

## 5. The oracle, already on disk

`gamelog-run7-ancient-nubian-orders.txt` (1,732 frames, `UNITS=3 BUILDS=7`,
seed 12345, Nubians vs a Nubian AI, Ancient Age, Small Town — `docs/INPUT.md`
§2) and `gamelog-run6` (432 frames, same lobby, no input). Player 1 is the
AI, and its first 115 seconds are legible in the dump today:

| frame | what appears | source |
|---|---|---|
| 1 | scout `1/0` takes an `EXPLORETO` (path of 3) | `think_scout` — the idle path, `docs/ORDERS.md` §2.4 |
| 2 | **site `2006` (type 417, Farm)** placed; citizen `1/1` pulled off woodcutter `2001` onto `BUILDORDER 2006` with a move inserted ahead | the script's `place_farm` → `place_building_with_cost`, or `create_buildings` — the reading settles which, and why frame 2 rather than 1 |
| 17 | `2006` starts building (`frame_started 17`) | |
| 100, 206, 320 | citizens `1/6`, `1/7`, `1/8` | three queued at the city, `train_unit_with_need` |
| 777 / 1121 | site `2007` (type 414, city) placed / started | `city_placement` or `found_cities` |
| 1177 / 1212 | site `2008` (Farm) placed / started | |
| 1297, 1505 | citizens `1/9`, `1/10` | |

The harness (`rondata --recgame … --gamelog … --diff`) already scores it:
player 1's units diverge at `1/0@1` (scout order length), `1/1@2` (the
build order), `1/2@4` (a path — the same frame-4 forest path as `0/2`, not
AI), `1/3..5@102` (`FARM_GROWS`), and **6,543 unit-frames belong to units
the sim never trains**. Every one of those numbers is a target this
mechanic moves; none of them can move without it.

What the dump does **not** carry: `production_step`, `script_step`, the
`MakeList`, or the personality. `LeaderData::log_data@006e5110` does call
`Personality::log_data` and `LeaderDataEncrypt::log_data`, so the rolled
personality is loggable at *some* detail level — `LEADERS=9` (run4) shows
no `PERSONALITY` block, so it is above that or under `DUMP_ALL` (a
one-run check, when the game is up).

## 6. The RNG accounting this forces

Three sets of `game_random` draws belong to the AI and sit on the sync
stream every other mechanic reads:

- **`Leader::random_personality@006cfd00`** (156, read): ~25 draws per
  computer leader — `rush`, then per-tribe adjustments (tribe indices 0,
  2, 3, 4, 5, 6, 7, 8, 0xc, 0xe, 0xf, 0x11 each bend the roll), a pass
  over every other computer leader's tribe, then one draw each for
  `cities`, `upgrades` (two-stage), `arms`, `army`, `army_size`, `raid`,
  `invade`, `target`, `strategy`, `raze`, `spells`, `forts`, `nukes`,
  `air`, `naval`, `market`, `scouts`, `civilians`, `early_army`, with
  tribe overrides (`rush_rules == 8` forces `raid = -1`). Called from
  `Leader::init`, which `Setup::build_game` runs **after its re-seed and
  before `build_empire`** (`docs/ORDERS.md` §9.2) — so the AI's draws sit
  between the start permutation and every `place_unit` draw. The harness's
  `build_sim` reads positions from the dump and has not needed to model
  them; the first mechanic that consumes the sync stream during play
  (combat's accuracy roll) will.
- **The script's `rand_int`** — the sync stream, at whatever frame the
  script runs.
- **`Leader::init`'s coin** for `economic` vs `defensive` — one draw, only
  when `pers.rush == 0`.

`Personality` is `LeaderData+0x6dd4`, 0x60 bytes, 24 ints (PDB):
`rush cities upgrades arms army army_size raid invade target strategy raze
spells forts nukes air naval market scouts civilians early_army
friendly_human alliance_human friendly_ai alliance_ai`.

## 7. What is already established elsewhere

- The unit layer: `Unit::think@005f6e40` (343) and its dispatch —
  `think_scout`, `think_peasant(0/1)`, `think_fish`, `think_merchant`,
  `think_attack` (245, `docs/COMBAT.md` §8.1) — `docs/ORDERS.md` §2.4,
  §5.9. The scout's frame-1 order is that path, not the production sweep
  (audit F4).
- The start of a game, including what the AI leader owns at frame 0 and
  the fact that `plan_strategy`'s frame-0 sweep issues no order:
  `docs/ORDERS.md` §9.
- Why no AI decision is in a recording, and why the diff cannot pass
  player 0 without this mechanic: `docs/INPUT.md` §1.
- The script API's *targets* — placing a site, queueing a unit, gaining a
  tech, paying a cost — are `docs/CITIES.md`, `docs/PRODUCTION.md`,
  `docs/TECH.md`, `docs/COSTS.md`; the host functions are thin over them.

## 8. Sizes, for the reading's budget

`Leader` (24,585 lines total; the AI half):

| function | lines | | function | lines |
|---|---|---|---|---|
| `diplomacy` | 3,282 | | `production_ai_setup` | 316 |
| `plan_strategy` | 1,754 | | `compute_sites` | 308 |
| `create_buildings` | 1,728 | | `make_stuff` | 305 |
| `create_units` | 1,674 | | `check_orphaned_buildings` | 294 |
| `produce_building` | 1,173 | | `found_cities` | 281 |
| `init` | 935 | | `produce_tech` | 273 |
| `research_techs` | 689 | | `make_this` | 187 |
| `action_respond` | 639 | | `produce_city` | 178 |
| `compute_site_stats` | 506 | | `random_personality` | 156 |
| `produce_unit` | 437 | | `produce_upgrade` | 152 |
| `upgrade_units` | 344 | | `use_market` | 147 |
| | | | `check_transport` | 124 |
| | | | `unit_prod_value` | 114 |
| | | | `production_ai` | 114 |
| | | | `market_speculation` | 109 |
| | | | `check_explore` | 47 |

`Army` / `Armies` (5,060): `find_target` 1,219, `find_muster_spot` 463,
`Armies::walk_data` 211, `process` 192, `find_besieged_city` 160,
`march_to_target` 136, `release_mustering` 134, `do_transporting` 132,
`do_forming` 126, `normalize` 115, `engagement` 109, `find_local_army` 102,
`do_marching` 99, `stop` 95, `is_moving` 95, `do_defending` 92, `add_group`
86, `find_army` 82, `init` 77, `find_waiting_unit` 76, `use_scouts` 67,
`use_generals` 67, `send_here` 65, `use_spies` 64, `is_engaged` 60,
`add_unit` 59, `do_mustering` 58, `Armies::init` 58, `center_of_gravity`
53, `find_useful_army` 46, `member` 44, `remove_group` 43, `update_city` 41,
`process_all` 36 (read), `send_navy` 35, `init_army` 32, `set_stance` 31.
`Armies::process_all` skips leaders with `leader_flags2 & 0xa` and any
`leader_flags & 0xc == 4`; the per-army flag `Army+0x4 & 0x80` is consumed
into `process`'s argument.

The script side: §3.

## 9. Proposed order of work

1. **The driver.** `strategy_all`'s gate and loop, `plan_strategy`'s
   cadence and its frame-0/every-200 sweep as a named seam, the
   `production_ai` step machine exactly, `production_ai_setup`. Pins:
   the step sequence per frame for `who = 1` from frame 0.
2. **The interpreter and the 55 host functions**, from the language
   semantics read out of `Compiler`/`VirtualMachine`, with `rand_int` on
   the sync RNG. Pin: `economic`'s first return and the farm at frame 2.
3. **The C++ production layers** in step order — `found_cities`,
   `research_techs`, `upgrade_units`, `create_units`, `create_buildings`,
   `make_stuff`/`use_market`/`market_speculation` — with `MakeList` and
   `produce_*`. Pin: run7's table in §5, all of it.
4. **Armies**, as its own document (`docs/ARMY.md`): the state machine and
   `find_target`. Its oracle is a longer run with a war in it, which does
   not exist yet.
5. **Diplomacy**: deferred, named.

Each of 1–3 ends in the harness: `rondata --recgame … --gamelog … --diff`
with player 1's units scoring, and the soak (`crates/sim/src/soak.rs`)
extended with AI leaders so the interpreter is under the determinism guard
from its first commit.

## 10. Traps, from the survey

- `grep` for callers of `Leader::process` finds only itself; the AI is not
  in it. The driver is `strategy_all`, reached from `do_frame` at line 267.
- `field_0xNNN` on a `Leader *` is a `LeaderData` field: the PDB's `Leader`
  is a 0x6eec-byte shell over `LeaderData` (0x6ee4) and Ghidra did not
  populate it. `types.txt` under `struct /rise.pdb/LeaderData` names every
  offset used above.
- `production_ai_setup`'s difficulty reads are a three-way choice
  (`semaphore[0] & 4` → `multi_diff`; else lobby `difficulty` unless
  `multi_diff ≥ 0` under two other semaphore bits) — read `get_diff` once
  and use it, do not re-derive the branch each time.
- The scripts' `static`s are per function, not per leader. An interpreter
  that scopes them per leader will agree with the original for a
  one-AI game and diverge for two.
- `String` in the scripts is a city name or a type name, compared and
  passed by value; `find_city_with_num` returns a `String`, and `""` is
  the not-found value the scripts test with `== ""`? — unverified, read
  `ScenarioFuncSet::find_city_with_num` first.
- The `_global` hits for `CommandManager::issue_*` were all unwind
  funclets (`docs/INPUT.md` §1 re-verified today); expect the same noise
  for any `grep -rl` over `funcs/_global`.
