# The AI

**Status: first reading complete; the opening script runs end to end —
2026-08-24.** The driver, the interpreter, the 55 host functions and
`produce_building` are built and wired (§12), and run7's first script call
is reproduced at frame 2 — the queues, the prices, the site number. The
census and the C++ producers are next (§12.1). §2
(the production AI: the frame hook, the cadence, `plan_strategy`'s sweep,
the step machine, the goods picture, the make list, the sites, the orphan
check, the city AI, the site score, research, the market, two producers)
and §3.1 (the scripts) are **read** by the main thread from the full
decompile export (`~/ghidra-projects/decomp/`, `tools/ghidra/`), every
function named there end to end. The four halves the main thread did not
read — the script language, the 55 host functions, `create_units`,
`create_buildings`/`produce_building` — are four readers' reports under
`~/ghidra-projects/reports/ai/`, ratified in §11, and are the
specification for what they cover. §4–§10 keep the survey's framing where
it still holds and say where it changed. §12 is what is built; §13 what is
not established. Confidence: **high** on the cadence, the step machine,
the make list, the language and the host functions' contracts; **medium**
on the census's per-unit classification, where two virtual slots are still
named by use (§2.10), and on the parts of the producers the reports flag
for the listing (§13).

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

## 2. The production AI — read

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
      building's — the building's footprint (`x_size`/`y_size`, centred
      by parity) is scanned column by column for the first cell whose
      terrain byte (`world+0x138`) has `& 0x30 == 0x20`, **water**, and
      that tile's *alternate* region (the record's short at `+6`, the sea
      region of a coastal tile flagged `0x100`) is taken; no such cell →
      the building's own tile region. So a unit inside a dock counts in
      the sea region. (The `create_units` reader's dock scan uses the same
      test and requires the result `≥ 0x3f`, which settles the sense.)
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
    every active building with `type+0x1e8` (`attack`) set counts `defense
    += 1` for a tower (`is(0x1b7)`) or `2` for a fort (vslot `+0xfc` =
    `is_fort`, `docs/CITIES.md` §1.5 — corrected from "has arrows" by the
    `create_buildings` reading), and `reg_defense[r]` the same.
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
`worst_good`, `best_good`, `shortages`, the AI's own `rate[6]`, and on easy
difficulties clamps the stockpile. The encrypted block is
`LeaderDataEncrypt` (`data_encrypted`, `+0x6eb8`), field names and keys as
`docs/ECONOMY.md` §"The ledger" has them: `bucket` (+0, the stockpile, `^
0x8221`), `resource_cap` (+0x30, `^ 0x1281`, sixteenths), `over_cap`
(+0x4c, `^ 0x8932`), `resources` (+0x64, the gather rate, `^ 0x872`),
`income` (+0x94, the net rate, `^ 0x90236`), **`rate` (+0xac, `^ 0x73862`)
— written only here**, `ages`/`epochs`/`epoch[4]` (`docs/TECH.md`).
Every figure below is in the ledger's own units (rates in sixteenths,
`rate` in whole units).

1. `shortages = worst_good = best_good = 0`. **`starting_resources == 8`**:
   every `econ[g] |= 8` and return.
2. **Difficulty** `d` = `multi_diff` when `semaphore[0] & 4`, else the
   lobby's `difficulty` unless a multiplayer-AI semaphore pair says
   `multi_diff ≥ 0` (three identical reads; `get_diff` is the same
   choice). **If `d < 3`**: `m` = the current age's type (`ages + 0x220`)
   is an age type → `max over available goods of get_cost(age_type, g,
   who, −1, −1, 0, 1, −1)`, at least `300`, then `d == 0` → `m × 3/2`, `d
   == 1` → `m × 2`, `get_diff() == 2` → `m × 5/2`; otherwise `m = 11000`.
   Then for each good with **`bucket[g] > m`: `escrow[g] = escrow[g] × m /
   bucket[g]`, `bucket[g] = m`** — an easy AI's stockpile is thrown away
   above ~1.5–2.5 times the next age's price, every step 2. (`m == 0`
   skips the clamp.)
3. **Rate pass**: for each good, `econ[g] = 0`; `rate[g] = min(get_mod_
   resource_cap(g), income[g]) / 16` (truncating toward zero); for
   available goods, `rate[g]` picks `worst_good` (min) and `best_good`
   (max), and `rate[g] < 30` → `econ[g] |= 1`, `shortages++`.
4. **Threshold pass**, per available good (an unavailable one gets `econ
   = 0`): `cap = get_mod_resource_cap(g)`; `lo = min(city_num × 15,
   cap/32, 250)`, `hi = min(city_num × 30, (cap/16) × 4/5, 350)`; with more
   than four cities `lo = cap/32`, `hi = min((cap/16) × 3/4, 175)`; for
   goods other than food and wood while `city_num < 3`: `lo = 20`, `hi =
   city_num × 20`, and bit 1 is cleared (`shortages--`). Then, with
   `income[g] < cap`: `rate[g] < lo` → `|= 2`; `rate[g] < hi` → `|= 4` and
   done; otherwise (rate at or above `hi`, or income at the cap) `|= 8`.
   So the bits are all about the **rate**: 1 = under 30, 2 = under `lo`,
   4 = under `hi`, 8 = comfortable.
5. **Food/wood balance** while `city_num ≤ 2`: if food is not "under hi"
   (bit 4) or the tribe has bonus `0x13`: when wood is, clear food's bit 4
   and the other four goods'; else (food under, no bonus): clear wood's
   bit 4 and the other four's. The same shape for bit 2, promoted to 4.
   Net effect: with two cities the AI only ever calls *one* of food/wood
   short at a time, and never the later goods.
6. `market_speculation()` — §2.15.

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
| `+0xfc` | BuildType | `is_fort` — `is(FORTX, 0)` at every devirtualised site (settled by the `create_buildings` reading; `docs/CITIES.md` §1.5) | settled |
| `+0x64` on `Type` | Type | the `create_units` reading resolved `UnitType`'s table from the PE: `+0x60 is`, `+0x78 get_cost`, `+0x84 can_pay_cost`, `+0xe8 get_age_slow`, `+0xec get_age`, `+0x10c is_siege`, `+0x114 is_missile`, `+0x130 is_caravan`; `+0x64` is an ICF-folded trivial there — on a *build* type it is `is_city` (`docs/CITIES.md` §1.5), and `make_this`'s use of it routes a city-kind slot to `produce_city` | settled |

### 2.11 The make list's insertion — `MakeList::make_me@006c9be0`

Read whole. `make_me(t, val, escrow, cat, city, up, o, num, wx, wy)` —
`cat` is **the slot index** of the entry's category (4..10), and the first
four slots are a ranked overall list:

```
if val > list[0].val:                       # a new best
    list[0] = entry (o = -1)                #   overwrites — the old head is NOT shifted down
    for k in 1..3: if list[k].t == t: list[k].t = -1
else:
    for k in 1..3:
        if list[k].val <= val:
            list[k+1..3] = list[k..2]       #   shift down, slot 3 falls off
            for j in k..3: if list[j].t == t: list[j].t = -1
            list[k] = entry (o = -1); break
        if list[k].t == t: break            #   the same type already ranks higher: not inserted
if list[cat].val < val: list[cat] = entry   # the category slot, best val wins
```

So `make_stuff`'s slots 1–3 are runners-up that never beat the head at
the time they were offered, and slots 4–10 are one-per-category. Categories
seen so far: **9** — a city site (`found_cities`) *and* a category-1 tech
(`research_techs`, §2.13) share it; **10, 4, 8** — techs of `cat` 0, 2,
other; **6** — a tower tech. The two `make_stuff` exceptions (§2.6 step 6)
are therefore "the cat-2 slot when it holds a gather building for a low
good" and "slot 5 when I have no free peasants and no gatherers" — slot
5's category is the unit readers' to name. `MakeList::clear` sets every
`t = -1`; `MakeList::init` allocates the eleven.

### 2.12 The city AI — `found_cities@006c7a60`

Read whole (281 lines). Returns at once when `leader_flags2 & 0x10`.
The type loop runs over `BASE_BUILDTYPES .. 0x19e` — i.e. **only `0x19e
VILLAGE`, the Small City** (`TOWN` and `METROPOLIS` are upgrades, never
founded; `FORBIDDENCITY 0x213` has a special-case branch the loop never
reaches). Gate: `get_total_cities() < get_city_limit()` (`city_mine +
queued Small Cities across every type whose `from` chain ends in a Small
City, −1 with the Forbidden City`; the limit is `epoch[1] ^ 0x63187`
(Civic) `+ bantu_city_limit` with tribe bonus 3, `+ 1`, `+
pyramids_city_limit` with the Pyramids).

Then: `type_avail(VILLAGE) == 4` and `free_peasants || gatherers`. **The
human cap** (unless `leader_flags & 8`): `max_human_cities()` = the most
cities-plus-queued any human holds (−1 with the Forbidden City); on
difficulty 0/1 the AI stops at `max(that, 2)` total cities, on 2 at
`max(that + 1, 2)`; ≥ 3 uncapped.

For each of the ten sites with `val > 0`, region `r = site.reg`:

- require `reg_free_peasants[r] || reg_gatherers[r]`;
- require `world+0x34 < 4` or `map_style == 0x14` or `city_num > 3` or
  `city_num < 2` or `r != home_reg` — with two or three cities on a
  many-landmass map, expand only off the home region;
- `blocked_site(VILLAGE, site × 0x300, who, −1, 0) == 0`
  (`docs/CITIES.md` §2.6);
- `v = site.val`; `reg_cities[r] == 0` → `v ×= 4`; else `v = (reg_pop[r] +
  1) × v / (2 × reg_cities[r])`, halved if the region is not thin
  (`strategy[r] & 1 == 0`);
- with **no Small City queued** (`num_queued[VILLAGE] == 0`): `reg_pop[r] ≤
  reg_peasants[r]` → `×2`; `reg_cities[r] ≤ reg_free_peasants[r]` → `×2`;
- **drop the site** if any active city of mine, or of any leader with
  cities in `r`, lies within **5 tiles** (the octagonal metric `max +
  min² / (2·max)`, ids from 2000); also if `v == 0`;
- `reg_land[r] < reg_cities[r]` → `×2`; `reg_land[r] < 2 × reg_cities[r]`
  → `×3/2`;
- under the city limit → `want = VILLAGE`, `v ×= 30`;
- `pop_cap < 200`: `effective_pop > pop_cap × 3/4` → `v = (n + 2) × v /
  (n + 1)` with `n = (cities + villages) / 5`; `effective_pop > pop_cap −
  4` → `×3/2`;
- thin region without a weaker-side flag (`strategy[r] & 5 == 1`): `c =
  cities + villages`; `peasants ≥ 5c` → `×2` else `≥ 3c` → `×3/2`; `≥ 2c`
  → `×2` if `c < 4`, `×3/2` if `c < 8`;
- `v ×= 10` (a negative product clamps to 999,999); my territory at the
  site's cell `> 0` → `/3`; the tile's owner not me → `/3`;
- `val = (v / 256 × f) / 256 / (num_queued[VILLAGE] + 1)` with `f = 0x100`
  if `can_pay_cost(VILLAGE, who, −1, −1, 1) > 0`, else `0x40` — so an
  unaffordable site is quartered;
- `make_me(VILLAGE, val, 1, 9, −1, 0, ·, 1, wx, wy)`.

After the loop: if `want` was set, the list's head is non-empty and
`make_stuff()` returns non-zero → `make_list.clear()`. **The city AI
buys on the spot** when it is under the limit, and clears the list so
the rest of the sweep starts fresh. No `game_random` draw of its own; the
draws on this path are `compute_sites`' (§2.7) and `make_stuff`'s expiry
(§2.6).

### 2.13 The site score — `compute_site_stats@006cd040`

Read whole (506 lines). `compute_site_stats(wx, wy, city, unit, reg, &val,
&dist, keep, &out_wx, &out_wy)`; `val = dist = 0` on every early return:

1. The site's fine cell flagged `0x100`; not `was_seen(2wx+1, 2wy+1, who)`;
   the tile owned by another leader (unless CtW with no cities of mine
   and an ally's — then `ally_land = 1`); `is_ocean` → 0.
2. `base = tile.value` (the tile record's byte at `+0xc`), or 0 if
   `blocked_site(TOWN, …)` — **the Town's footprint**, not the Village's.
3. Over the **25 offsets** of `move_x/move_y` (the 5×5 around the site),
   each seen, in-bounds tile: land and unflagged: owner unowned → `danger
   += 1` (`+2` when I have more than two cities); owned by my `target`
   (team style 2) → `target_adj++`, `danger += 2`; owned by a non-ally →
   `danger += 2`. Water or flagged: `water++`, `danger = max(danger − 1,
   0)`. Then, if the tile is mine (or the ally's) and **`keep`**: a fort
   there → `forts++`; its own score `q = (value(x,y) + value(x+1,y) +
   value(x,y+1) + value(x+1,y+1)) / 4 + z(x,y) / 25` (heights from
   `find_tcoord_z`); `q > base` and `blocked_town(TOWN, x, y, who) == 0` →
   the site **moves** there (`out_wx/out_wy`, `base = q`). So a sampled
   site slides to the best-valued open tile in its 5×5; a re-scored one
   (`keep = 0`) stays.
4. `base < 1` → 0. `parity = base & 3`.
5. Many landmasses (`world+0x34 > 2`) and I own no dock (`num_buildings[
   DOCK] + get_buildings(its upgrade)` = 0): any of the **40 offsets**
   `move_x[81..120]` (the outer ring) on ocean → `base ×= 30`.
6. `v = base × 250 / (water + 1)`; `city_num == 1` → `v = v × (min(danger,
   9) + 7) / 8` if `danger`; `== 2` → `min(danger, 4)`; else `forts` →
   `×3/2`. `v = (target_adj + 1) × v / 2`.
7. Map-edge penalty unless map type `world+0x30` is `0xc`/`0x11`: with `<
   3` cities, `wx` outside `[w/5, 4w/5]` → `/4`, `wy` outside `[h/5,
   4h/5]` → `/4`; with `≥ 3`, the bands are `1/10 .. 9/10`.
8. `v /= max(1, my territory at the site's cell)`; the tile's *territory*
   owner (`+0x10`): unowned → `×2`, a non-ally → `×4`, an ally → `/2`.
9. Unless `map_style == 0x14`: `reg_cities[reg] == 0` → `×2`; `== 1` →
   `×2`; `reg_land[reg] == 0` → `×4`; `< 3` → `×4` (so a region with no
   open tiles gets ×16); `reg_land < reg_cities` → `×2`; `< 2 ×
   reg_cities` → `×2`.
10. **Goods**: `bits = tile+0xd` (which goods are gatherable near the
    tile); for each available good `g`: if bit `g` set → `have |= econ[g]
    & 6` and `×3/2` per set bit 2 / bit 4; else `lack |= econ[g] & 6`.
    `have == 0` → `v = 2v/3`; `lack & ~have` → `/2` when `parity == 0`,
    else `×3/4`.
11. **A nomad** (`city_num == 0`): the nearest citizen's distance
    (`find_unit`, base type `0x32`) → `v /= (dist / 0xc00 + 1)`; and unless
    `starting_resources == 8`, one of the five `corner_x/y` offsets must
    have bit 2 in its `+0xd` (wood nearby) or the site scores 0.
12. **Distance term**: no citizen given → `k = 3`, `dist = 9`; else `k =
    1`, and with a city given, for each of my cities `d = vector_dist(site,
    city)`: `d < 5` → `/2`; `d < 8` → `/2`; `k += d` (capped at 10 with
    more than two cities); `dist = 2k²`. `v ×= k`. **Unresolved**: the
    decompile shows this loop returning (`val = 0`) on the first inactive
    city slot or a city in another region, and prints `vector_dist`
    without its arguments; the listing settles whether those are `break`s
    — flagged for the implementation.
13. Enemy proximity (not maps `0xc`/`0x11`, and `city_num == 1` or team
    style 2): over every computer leader at war (or, team style 2 with ≥ 2
    cities, only my `target`): for each of their active cities, `sum +=
    vector_dist(site, city) / num_nations`; `×100` in team style 2; `v /=
    sum` if non-zero — **closer to the enemy scores higher** when I have
    one city.
14. `ally_land` → `/10`. `val = v`, `dist` as above.

The tile record (`world+0x134`, `0x1c` bytes a tile): `+0x4` region,
`+0xc` site value, `+0xd` nearby-goods bits, `+0xf` owner, `+0x10`
territory owner, `+0x28` = the next tile's `+0xc` (used by the 2×2 sum).

### 2.14 Research — `research_techs@006c6ba0`

Read whole (689 lines). Capital-countdown elimination (`elimination ==
1`): only while I hold my capital. `low = get_lowest_epoch()` (the least of
`epoch[0..3] ^ 0x63187`, capped 99); `over = number of available goods
whose `over_cap` marker is not the clean value 0x8932` (§ECONOMY — goods
over their cap). Then for every `t` in `0x220..0x274` with
**`Leader::tech_avail(t) ≥ 4`** (a tech type; not `leader_off`-masked for
me; not had; not researching; a government tech whose pair `(t −
BASE_GOVTYPES) ^ 1` is neither had nor researching; then `LeaderData::
type_avail`):

- `where = t.where` — the building it is researched at; require
  `num_buildings[where] + get_buildings(where's upgrade) > 0`.
- `base = (pop × 200 / max(1, cities + villages)) × infra_mod / 256`.
- *(The `starting_resources == 7 && starting_technology == 8` lobby has a
  hard-coded branch that buys tech `0x243` when affordable; skipped here.)*
- **Ages** (`0x220..0x226`): knowledge cost `> resource_cap[3] × 4/3` →
  skip. **Human pacing** unless `leader_flags & 8`: `h =
  max_human_age()` (the highest age of any human, −1 if none) `≥ 0` —
  difficulty 0/1: only ages `a = t − 0x220 ≤ h − 1`, and only once
  `frame − best_human_age_stamp(t) ≥ (who + 16 | 8) × 225` (the earliest
  frame any human reached that age, `age_stamp[a]`; 16 on 0, 8 on 1;
  unstamped passes); difficulty 2: `a ≤ h`, delay `(who + 20) × 225`;
  otherwise unpaced. Then `base ×= 10`, or `×= 600` when `a < low`
  (catching my lowest line up).
- **Epoch techs** (`0x227..0x242`, the four library lines): knowledge cost
  as above; while `ages < ending_technology`, `t != 0x23c`, and not
  (`cat == 3 && tribe == 9`): `n = techs_per_age(my age)` (`docs/TECH.md`
  §"The age quota"), `+4` for `cat 0` when `effective_pop ≥ pop_cap ×
  7/8`, `+4` for `cat 2` when `over ≥ 2`; **difficulty 0/1: only while
  `epochs < n + 2` and `epoch[cat] < n`; difficulty 2: `epochs < n + 6`
  and `epoch[cat] < n + 1`**; ≥ 3 uncapped. Then `epoch[cat] == low` →
  `base ×= 10`; `epoch[cat] > low + 1` → skip (keep the four lines within
  one of each other); `cat` 1 or 2 under victory 8/9 → `×30`; more than one
  landmass and `types[0x2ae]`'s predicate `+0xdc(t)` → `×30`; `t` in
  `{0x22f, 0x236}` with `has_tech(CLASSICAL_AGE)` → `×20`.
- Everything: knowledge cost `< resource_cap[3] − 100` → `base ×= 10`.
- **Weights** (`TechType.ai[0..10]`): `w = 1 + ai[0]/10 + Σ ai[1..10]`;
  `± ai[10]` by team style (+ for 0/8/0xb); `full_cities > (cities +
  villages)/2` → `+ 2·ai[5]`, else `full_cities` → `+ ai[5]`;
  `my_team_terr < other_team_terr` → `+ ai[5]` (×2 if `< min_other`);
  `active_wars == 0` → `+ ai[5] + ai[1]`, else `+ ai[0]/3`; landmasses
  `m = world+0x34`: `m == 0` → `− ai[2]`, else `+ ai[2] × m³`. `val = w ×
  base`.
- **Category** (`t.cat`): `0` → `/10`, then with `pop_cap < 200`:
  `effective_pop > pop_cap × 5/6` → `×20`, else `epoch[0] > 1` → `/10`.
  `2` → `×3` if `epoch[2] == 0`; and for a non-epoch, non-age tech, `k =`
  the number of non-knowledge goods at `≥ 90 %` of cap → `val = k ? val ×
  k² : val / 10`. Otherwise, **difficulty ≥ 4 and `cat == 3`** only: the
  gather-rate lines `0x264–0x266` (food), `0x261–0x263` (wood),
  `0x26c–0x26e` (metal), `0x267–0x26a` (knowledge) → `×2` for `econ[g] &
  4`, `×4` for `& 2`, `×8` for `& 1`; then `d = epoch[3] − t.age > 0` →
  `×(d + 1) × d`.
- Victory 9 → `×3` (`×9` for an age).
- **Slot** by `cat`: `0 → 10`, `1 → 9`, `2 → 4`, else `8`.
- **By building**: `where == TOWER` → skip below difficulty 2, else
  `×100`, slot `6`; `0x1b5` → `×4`, `×400` while `t < 0x251`; `0x1b3` →
  `val = (max(epoch[0..3], 0) × val + 1) / (epoch[cat] + 1)`;
  `UNIVERSITY` → `×1000`; `0x1b6` (governments): `pers.raid < 0`, or `== 0`
  and a **`game_random` coin** → prefer `{0x26f, 0x271, 0x273}`, else
  `{0x270, 0x272, 0x274}`; the non-preferred of a pair gets `× (rand %
  100)` — **a second draw** — and `×20` if I have no government yet.
- **Recency**: `f1 = min(4, (frame − tech_frame) × ai_speed / 14400 + 12)`,
  `f2 = min(3, (frame − tech_cat_frame[cat]) × ai_speed / 25600 + 16)` —
  `val = f2 × (f1 × val / 2) / 2`. Both stamps are only ever written by
  `Leader::init` (to 0; §2.17), so `f1 = 4` and `f2 = 3` always: `val ×=
  3`. `shortages == 0` → `×2`.
- **Coverage**: `total = Σ costs[0..5]`; `missing = 2 × Σ costs of goods
  neither available nor prerequisite-reachable`; `val = (2·total −
  missing) × val / (2·total)`; negative → 9,999,999; `wonder_mod != 0` →
  `/2`; `val = check_income(t, 0x400, −1, wonder_mod == 0, −1, 1, 0) ×
  val / 256`; negative → 9,999,999.
- `make_me(t, val, 1, slot, −1, 0, ·, 1, 0, 0)`.

Sync draws in this function: the government coin (only when `pers.raid ==
0`) and the `rand % 100` for a non-preferred government — both only for
techs researched at building `0x1b6`.

### 2.15 The market — `use_market@006c91c0`, `market_speculation@006c8110`

Both read whole. Both require the market ability — tribe bonus 4 or
`has_preq(BUY_SELL)` — a market building (`has_market`) and no nuclear
embargo (`get_nuke_embargo`, else `tell_embargo` and nothing); `do_buy` /
`do_sell` are `docs/ECONOMY.md`'s, called once each (the decompile's
`do … while (i < 1)` is a single try).

**`use_market`** (from `make_stuff`, first thing): `need[g] = Σ` over the
first `max(1, epoch[2])` (Commerce) make-list slots with `t > 0` and `val >
0` of `get_cost(t, g, who, o, city, 0, 1, −1)`. For each available
non-knowledge good `g` with `bucket[g] < need[g]`, once: if `g` is wealth,
or `calc_market_prices(g)`'s buy price would leave `bucket[wealth] −
price < need[wealth]` → **sell** instead: starting from **`rand % 6` —
one `game_random` draw** — go round the six goods and sell the first
that is available, not knowledge/wealth/oil, not `g`, has `bucket − 100 ≥
need`, and either `income ≥ cap / 2` or `bucket > 199`; else **buy** `g`.
The outer `while bucket[g] < need[g]` loop is bounded by the once-flag,
so at most one buy or one sell per good per call.

**`market_speculation`** (from `production_ai_setup`, last thing; skipped
under `starting_resources == 8`): `escrow[g] > 4000` → `2000`; `tier` = 2
if any available good's stock `< 100`, else 1 if any `< 200`, else 0.
**Sell** each available non-wealth, non-knowledge good with `bucket −
escrow ≥ 2000 >> tier` and sell price `≥ 75 / (tier + 1)`. **Buy** each
such good when wealth minus its escrow covers the buy price, `bucket <
2000`, price `< 201`, and (`bucket < 500` or price `< 26`) and (`< 200`
or `< 51`) and (`< 100` or `< 101`). No draws.

### 2.16 `queued_units`, `compute_score`

`queued_units@006ce000` (read): over my buildings from id 2000 that are
active and complete (vslots `+0xc`, `+0x4c`), Σ `control_cost` of every
queued item that is a unit type with `type_avail ≥ 4` — the population the
queues will add. `effective_pop = queued_units + control + 1` (§2.4).

`compute_score@006ec560` (read): at frame 0, or when forced, or when
`semaphore[0] & 0x40`, or on `(who + frame) % 10 == 0`: the score
components (`score_units` on the `0x1800000` flags, `compute_build_score`,
`compute_economy_score`, the upgrade and research scores on `0x1000000`,
`score_territory = territory × 1000 / world+0x78`), summed into `score`,
zeroed after Armageddon. No decision reads it in the production AI; it is
the score screen's, and the `LEADERDATA` dump carries `score`.

### 2.17 Two producers — `produce_tech@006ca980`, `produce_city@006cb120`

Both read whole; `produce_unit`, `produce_building`, `produce_upgrade`,
`produce_spell` are the two readers' (§4 reports). A producer returns
**0 when it queued or placed** (`make_this` then demotes the slot's `val`
a hundredfold) and 1 when it could not.

**`produce_tech(t, escrow)`**: 0 if `has_tech(t)` or `researching(t)`.
The research building is `t.where`. If it is not a military trainer
(`BuildTypeData::is_military_trainer`): over every active city of mine
that `count_buildings(where)` finds one in, walk the city's building
chain (`city+8` first, `Build+0x74` next) for a building that is
complete (vslot `+0x20`), `is(where)`, active (`Build+0x8 & 4`), not
upgrading (vslot `+0x60` clear), and whose queue accepts `(1, t)` (vslot
`+0x190` returns 0); score `= city.level × 10⁶ / (queue.length + 1)`,
halved if `is_unassimilated`, divided by `(my territory at the building
+ 1)`; the best wins. A military trainer is searched in `mil_trainers`
(`+0x6e50`, the leader's own list) with score `10⁶ / (queue.length + 1) /
(Build+0x24 + 1)` and the same halving and division. None → 1. Else
`Build::queue_up(building, t, escrow)`'s result (`docs/PRODUCTION.md`).
`tech_frame`/`tech_cat_frame` are **not** written here — nor anywhere:
the only writer in `Leader`, `LeaderData` and `Leaders` is `Leader::init`'s
zero, so research's recency factors (§2.14) are the constants `f1 = 4`,
`f2 = 3` for the whole game. Dead terms, kept in the formula for fidelity.

**`produce_city(t, wx, wy, escrow)`**: the site's region `r`; the best
citizen of mine — on the map, base type `0x32/0x33`, action none,
`GATHER (7)` or `EXPLORE_TO (3)`, in region `r` — by `vector_dist(site,
citizen)`, **+24 for a gatherer**; none → 1. Then the type's pay slot
(`BuildType` vslot `+0xd4(who, −1, −1, escrow, 0)` — the charge; named by
use), `blocked_site(t, site × 0x300, who, −1, 0)` again, `Objects::
init_build(who, t, wx × 0x300, wy × 0x300, 0, −1)` → the site object;
failure → 1 (**after** paying — the found_cities check makes this
unreachable in practice, but a mod's script can reach it through
`place_city_with_cost`). Then a `Group` of the chosen citizen — or, for
a **nomad** (`city_num == 0`), **every alive citizen of mine** —
`action_swarm_around(site, who, QUEUE_NEW, BUILD_AT, 1)` (`docs/
ORDERS.md` §5), and the census is adjusted in place: a non-gatherer →
`free_peasants--`, `reg_free_peasants[r]--`, the nearest city's
`free--`; a gatherer → `gatherers--`, `reg_gatherers[r]--`, the city's
`gatherers--`. Return 0.

Run7's city site `2007` appears in gamelog `FRAME 777`, which is the end
of **game frame 776** (`docs/INPUT.md` §3: the dump numbers from 1, the
game from 0). `who = 1`'s sweep is at 775 (`175 + 200·3`), so 776 is
**step 1 — the script**: `city_placement`'s `place_city_with_cost`, which
is `compute_sites(0)` + `found_cities()` under the hood, and
`found_cities` bought on the spot. The C++ `found_cities` step proper
would have been 778. By the same convention the farm in `FRAME 2` is game
frame 1, step 1 again: the script's `place_farm`.

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

### 3.1 The two scripts, read

Both `economic.bhs` (934 lines, "boom script") and `defensive.bhs` (816,
"rush script") have the same skeleton, and `aibestbuildlibrary.bhs` (650)
is included by both:

1. **Bail-outs.** No city and no citizen queued → `SCRIPT_DONE`. A city
   attacked or raided (`was_city_attacked/raided(who, "", −1)`) → with
   Military 1 and no barracks, `place_building_with_cost(Barracks,
   capital)` and `SCRIPT_DONE` (or `BLOCK_ON_THIS` if it could not);
   otherwise `SCRIPT_DONE`. **The opening ends the moment the AI is hit.**
2. **The statics.** `prev_step`, `needed_citizens`, `timer_started`,
   `fishermen_total` × 8, indexed by hand through `switch (who − 1)`
   ("ghetto array") because a `static` is one slot for every leader
   (§3); `static int needed_techs = get_techs_per_age(who)` (initialised
   from a host call — *whose* `who`, on first execution, is the language
   report's question 4); and in `defensive` **`static int rush_build0..7
   = rand_int(1, 10)` — eight sync-stream draws at static
   initialisation**, one roll per leader slot, `< 4` (a third of the time)
   selecting the Ancient-Age rush.
3. **The first call** (`step == 1`): `get_starting_resources(who) > 4` or
   more than one city → `SCRIPT_DONE` (the opening is for a fresh, poor
   start only); `get_starting_town_size(who)`: 0 → nomad (`step = 1`), 1
   → `2`, ≥ 2 → `6` (`economic`: `7` for Greeks/Bantu with powers). A
   Conquer-the-World start of size > 2 takes a separate four-step
   military opening.
4. **Every call**, before the step loop: `train_unit_with_need(who,
   needed_citizens, "Citizen")` (`economic` always; `defensive` only when
   `needed_citizens > 0`); caravans up to `cities(cities − 1)/2` and up to
   three merchants with a market; `population ≥ 23` → Military 1;
   scholars on spare wealth; fishermen after a dock; `economic` adds a
   tower after Military 1 and Allegiance after Mathematics; `defensive`
   adds the German granary/lumber mill, docks and farms from step 17 on,
   and **the rush branch** at step 6 when `rush_build < 4` (not a
   conquest, not a sea map, town size > 1): five farms, Military 1, nine
   citizens, a barracks, four Hoplites and a Slinger, then `rush_build =
   25`. **The hang guard**: `set_timer(who, 300 | 240)` when the step has
   not moved since the last call, `stop_timer` when it has, `timer_expired`
   → `SCRIPT_DONE`.
5. **The step loop**: `for (i = 0; i < num_loops; i++) switch (step) { … }`
   with `num_loops = 5` from the engine (§2.4). Each case ends in
   `return_value = BLOCK_ON_THIS; break;` — **`break` leaves the `switch`,
   not the `for`**, so one call advances through up to five steps as long
   as each succeeds, and a step that fails is retried on every remaining
   iteration. A `return` inside a case ends the call at once.
6. **The end**: the statics are written back through the second `switch
   (who − 1)`, and the return value is 1/2/3 by the `labels`.

**`economic`'s steps** (Small Town enters at 6): 6 Science I (skipped if
under 75 wealth), 7 Civic I (done if a second city exists), 8 citizens to
9, 9 `place_farm`, 10 `city_placement` once City State is in (Bantu: a
third), 11 citizens to 11, 12 farm, 13 second woodcutter, 14 Commerce I,
15 market, 16 citizens to 14, 17 farms to 7 (British: 10, then 28), 18
**Classical Age** (only when `needed_techs ≤ 3`; `SCRIPT_DONE` otherwise
or once in it), 19 second market, 20 two universities, 21 mine (Inca:
three), 22 citizens to 23, 23 Commerce II, 24 Military I, 25 farms to 9
(Egyptians 11), 26 granary, 27 citizens to 28, 28 Science II, 29 Civic II,
30 third city, 31 barracks, 32 third university, 33 third market, 34
third woodcutter → `SCRIPT_DONE`, 35 dock (sea maps), 36 Mongol stables.
Nation branches (Greeks, Bantu, British, Egyptians, Inca, Koreans,
Lakota, Mongols) re-route between them; sea maps take 35/18/23/30/…

**`defensive`'s steps**: 6 Science I, 7 a fourth farm, 8 university
(Classical or Greeks only), 9 Civic I, 10 `train_unit_with_cost(3,
Citizen)`, 11 `city_placement`, 12 citizens to 10, 13 a fifth farm, 14
second woodcutter, 15 Commerce I, 16 a scholar, 17 citizens to 16, 18
Commerce II (needs a second city), 19 citizens to 20 (Bantu third city),
20 market, 21 two scholars, 22 citizen 21, 23 third woodcutter if the
camps hold fewer than 12, 24 citizen 22, 25 Military I, 26 citizens to 25,
27 barracks, 28 tower, 29 Classical Age → `SCRIPT_DONE`.

**The library**: `city_placement` (City State → `place_city_with_cost`;
two `trigger` blocks re-armed by `enable_trigger`, returns 1 once the
site has started, else −1); `place_woodcutter` (capital, or the unfinished
second city, or the second city; a camp with fewer than `min_size = 5`
slots is **destroyed** and `min_size` lowered — a `static`); `train_unit_
with_need(who, high, what)` (per city, open farm/camp/mine slots minus the
queue and the idle, then `train_unit_at_with_cost` at the neediest city
up to `high`, then `assign_idle`); `assign_idle` (idle citizens moved to a
camp with room, or a new camp placed); `woodcutter_check`, `place_dock`,
`place_mine` (each tries the cities in order, then orphan sites next to
camps, farms and the library), `place_farm` (five a city, seven for
Egyptians).

**Run7's opening, traced.** A Small Town, Ancient, no input on player 1.
Under `defensive`: game frame 1 runs steps 6 (Science I — `research_tech_
with_cost` or the 75-wealth skip), 7 (**the fourth farm — site `2006` in
`FRAME 2`**), 9 (City State), 10 (**three citizens — `1/6`, `1/7`, `1/8`
at 100/206/320**), 11 (blocks until City State); 176, 376, 576 re-block
at 11; **776: `city_placement` → site `2007`**, step 12; 976: `needed_
citizens = 10` → citizens `1/9`, `1/10` (1297, 1505 — food-bound), 13 a
fifth farm (site `2008` at 1177 fits here or under `place_farm`'s
every-call branch at `step > 16`? — no: 13, since `step > 16` is not yet
reached). Under `economic`: 6, 7, 8 (`train_unit_with_need(9,
"Citizens")` — note the **plural type name**, which the host function's
name lookup either resolves or returns 0 for), 9 (`place_farm` — the same
farm), 10. The dump's exact counts decide which script the rush roll
gave this game; the interpreter over either reproduces the table, and
`rush_build`'s eight draws sit at the first `defensive` call in the game.

**Language facts to ratify from the scripts themselves**: `String`
compared to an `int` (`my_capital > −1`, `find_inactive_build(...)` used
as a truth value and as an id); `!` on a host return; `step += 2` on a
`ref`; `return` inside `for`/`switch`; a `while` (`defensive`'s rush
farms); nested function calls as arguments; `static` initialised from a
call; implicit globals (`my_capital`, `wood_camp_2`, `xpos`, `i`, `wc`,
`f`, `m`, `size` — never declared in the caller); `//` comments;
`labels`; `include`; string literals with an apostrophe (`"Woodcutter's
Camp"`); the double `;;` after `return −1` in `assign_idle`.

## 4. The fork this opens — decided: (a), 2026-08-24

Three ways to have the opening, in the order of the project's own rules.
**(a) was chosen the same day**, with the two conditions stated at the
time: the interpreter's `float` (unused by the shipped scripts, present in
the language) is implemented on `combat::F32`, the integer-mantissa
software float, so a mod's script stays under `no_float.rs`; and script
statics and timers are sim state, digested by the soak from the first
commit. `docs/DECISIONS.md` gets the entry when the interpreter lands.

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
seed 12345, Nubians vs a **British** AI (the dump's `tribe 11`; "a Nubian
AI" in `docs/INPUT.md` §2 is imprecise), Ancient Age, Small Town — `docs/INPUT.md`
§2) and `gamelog-run6` (432 frames, same lobby, no input). Player 1 is the
AI, and its first 115 seconds are legible in the dump today:

| frame | what appears | source |
|---|---|---|
| 1 | scout `1/0` takes an `EXPLORETO` (path of 3) | `think_scout` — the idle path, `docs/ORDERS.md` §2.4 |
| 2 | **site `2006` (type 417, Farm)** placed; citizen `1/1` pulled off woodcutter `2001` onto `BUILDORDER 2006` with a move inserted ahead | game frame 1 = step 1: **the script's `place_farm`** → `place_building_with_cost` (`FRAME n` is the end of game frame `n − 1`, `docs/INPUT.md` §3) |
| 17 | `2006` starts building (`frame_started 17`) | |
| 2 | the city's queue holds **three citizens** (costs 25, 26, 27 — the ramp) and the library's **two techs, `551` and `565`** (Science I, Civic I) | `defensive` steps 6, 7, 9, 10 in one call (§3.1); `economic` would show the same techs and up to four citizens |
| 100, 206, 320 | citizens `1/6`, `1/7`, `1/8` | the three queued at game frame 1, one queue |
| 777 / 1121 | site `2007` (type 414, city) placed / started | game frame 776 = step 1 of the sweep at 775: **the script's `city_placement`** → `place_city_with_cost` → `found_cities` buying on the spot (§2.17) |
| 1177 / 1212 | site `2008` (Farm) placed / started | |
| 1297, 1505 | citizens `1/9`, `1/10` | |

The harness (`rondata --recgame … --gamelog … --diff`) already scores it:
player 1's units diverge at `1/0@1` (scout order length), `1/1@2` (the
build order), `1/2@4` (a path — the same frame-4 forest path as `0/2`, not
AI), `1/3..5@102` (`FARM_GROWS`), and **6,543 unit-frames belong to units
the sim never trains**. Every one of those numbers is a target this
mechanic moves; none of them can move without it.

~~What the dump does **not** carry: `production_step`, `script_step`, the
`MakeList`.~~ **All three are in a `LEADERS=9` record**, with the whole
census — `active`, `peasants`, `gatherers`, `free_peasants`, the
`gather_slots`, the `reg_*` arrays, `strategy[]`, `site_mark`, the ten
`sites`, the eleven `MAKEOBJECT`s, `effective_pop` — under `LeaderData`'s
own names (`docs/ORACLE.md`, "`LEADERS=9` is the census oracle";
`tools/gamelog/leader.py FRAME WHO`). Run8's frame 1 is the AI after its
frame-0 sweep: `active 6 control 6 peasants 5 scouts 1 gatherers 5
free_peasants 0 home_reg 1 explored 22 territory 261 site_mark 16`,
`gather_slots {food 3, wood 5}`, `filled {3, 2}`, `reg_land[home] 43`,
`reg_gather_slots[home] 8`, `strategy[home] 1`, and two sites — `(52, 14)
val 370 dist 27 rank 10`, `(45, 59) val 0 rank 9`; frame 2, after the
script's first call: `production_step 0 script_step 11 gatherers 4
effective_pop 7` (the builder pulled off its camp by `produce_building`'s
census adjustment; `effective_pop` computed before the script ran). And
**the map is a dump too**: run9 (`gamelog-run9-world6.txt`, `WORLD=6`
under `[Start Game]`) carries every cell's `val`, `goods`, `region`,
`region2` and owner and every tile's mask — with them loaded the fourth
farm lands on the original's tile (`docs/ORACLE.md`). ~~Or the
personality.~~ **Run8 (2026-08-24,
`gamelog-run8-personality.txt`, same lobby and seed, `LEADERS=9`, 60
frames, quit cleanly) settled it**: the personality is `LEADERS=9`'s tail
(`docs/ORACLE.md`, corrected), and the AI leader rolled

```
rush 1  cities 1  upgrades 0  arms -1  army -1  army_size 1  raid 1  invade 1
target 0  strategy 1  raze 0  spells 0  forts -1  nukes 1  air 1  naval 0
market -1  scouts 1  civilians -1  early_army 1
friendly_human 1  alliance_human -1  friendly_ai 1  alliance_ai -1
```

with **`prod_script = defensive`** — consistent with `rush = 1` (§3), with
`early_army = 1`, with the pairs `friendly_ai = friendly_human` and
`alliance_ai = alliance_human` that `random_personality` writes, and with
the frame-1 trace below. The human and the two nature leaders are
all-zero. Run8's start dump is byte-identical to run7's in every unit
position, so the same values hold for run6 and run7. The harness cannot
*roll* them — the map maker's draws sit before `Leader::init` in the
stream (`docs/ORDERS.md` §9.2) — so the acceptance test sets them from
this block (`rondata::diff::tests::run7_s_first_script_call_fills_the_
queues_the_dump_shows`).

## 6. The RNG accounting this forces

Three sets of `game_random` draws belong to the AI and sit on the sync
stream every other mechanic reads:

- **`Leader::random_personality@006cfd00`** (156, read): ~25 draws per
  computer leader — `rush`, then per-tribe adjustments (tribe indices 0,
  2, 3, 4, 5, 6, 7, 8, 0xc, 0xe, 0xf, 0x11 each bend the roll), a pass
  over every other ~~computer~~ leader's tribe (**corrected 2026-08-24**:
  the loop's guard is `leader_flags & 3 == 3`, not-ally, not `& 0x10` —
  the human is in it), then one draw each for
  `cities`, `upgrades` (two-stage), `arms`, `army`, `army_size`, `raid`,
  `invade`, `target`, `strategy`, `raze`, `spells`, `forts`, `nukes`,
  `air`, `naval`, `market`, `scouts`, `civilians`, `early_army`, with
  tribe overrides (`rush_rules == 8` forces `raid = -1`). Called from
  `Leader::init`, which `Setup::build_game` runs **after its re-seed and
  before `build_empire`** (`docs/ORDERS.md` §9.2) — so the AI's draws sit
  between the start permutation and every `place_unit` draw. ~~The
  harness's `build_sim` reads positions from the dump and has not needed
  to model them.~~ **Settled 2026-08-24 by the setup path's checksum
  trace** (`docs/ORACLE.md`, "The setup path's checksum trace is the RNG
  state"): `Leader::init`'s checkpoints bracket the roll, and on this lobby
  it is **exactly twenty draws**, `0x9991b076` → `0xf2299eda` —
  `Personality::roll` from the near end reproduces the dump's
  `PERSONALITY` block field for field and lands on the far end.
  `build_sim` seeds the roll from the bracket and the frame-0 state from
  the trace's last record (`0x3bd39ae9`).
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

## 11. The four reports, and their ratification

Four readers worked the halves the main thread did not, each writing to
`~/ghidra-projects/reports/ai/` incrementally; the main thread then took
each report's load-bearing claims back to the decompile before building
on them. The reports are the specification for what they cover and are
cited here rather than restated.

**`bhs-language.md`** (Opus, 1,376 lines — the language, from
`Compiler`/`VirtualMachine`/`RunTimeEnv`, 73 opcodes tabulated, the
constructs the scripts use mapped to their emitted shapes). Ratified
directly: `ScriptInt::is_false` is `value < 1` (truth is `> 0`);
`ScriptFunc::make_params` prepends argument 0 first, so the stream runs
right to left, **and auto-casts every argument to its declared parameter
type** (the `set_timer(who, 300)` int→String); `SyntaxNode::eval_binary_op`
casts the right operand to the left's type first, else the left to the
right's — so `my_capital > -1` is a case-insensitive string compare, true
whenever a capital exists. Taken on the report's listing checks: `OP_BIT_SET`
is a `btrl` (a trigger disarms on firing), `OP_JUMP_IF_INITED` guards a
static's initialiser once ever, `String::==` is `_wcsicmp` after a length
check. **One correction**: the report calls a timer's unit "game ticks
(frames)"; `Game::do_frame` advances `tick` every 15 frames, so **a
`set_timer(who, 300)` fires 300 seconds — 4,500 frames — later**, which is
what a five-minute hang guard should be, and what `was_city_attacked`'s
`/15` already implied.

**`host-functions.md`** (Opus, 1,257 lines — the 55 host functions plus
`rand_int`, each with what it reads, decides, mutates, draws and returns).
Ratified directly: `get_type_index` matches `TypeData+0x60 name` under
flag 0 (`+0xb0 type_name` under 1), so the scripts' type names are the
display names, not `TYPENAME` — `rondata` must carry both; the internal
`find_unit` clamps its cursor and laps once from `cursor + 1`, wrapping to
0, and `find_counters[]` is global rotating state the interpreter's host
must model; `population` returns `control`; `get_starting_town_size` is
version-gated; `ScriptTimers::check` removes the timer it reports
expired; `research_tech_with_cost` returns 1 for an owned tech and seeds
`find_counters[0x1e]` at 2000. `rand_int`'s exclusive upper bound is
`combat::Rng::get`'s already. Its open items — `produce_building`'s draw
count, `can_pay_cost`'s two context arguments — the building reader and
`docs/COSTS.md` answer.

**`create-units.md`** (Fable, 679 lines, two listing checks) and
**`create-buildings.md`** (Fable, ~760 lines). The two producers read
end to end, every `val` formula and every gate; the `role` bits from
`UnitType::determine_roles`; the make list's categories (**4** economy —
merchants, caravans, scholars, fishermen, gather buildings and enhancers;
**5** citizens; **6** military units, spies, scouts; **7** military
trainers and military upgrades; **8** other buildings and other upgrades;
**9** cities and cat-1 techs; **10** cat-0 techs; **6** also tower techs);
`produce_unit`, `produce_upgrade`, `produce_spell`; `check_income`;
`unit_prod_value`; `upgrade_units`; and `produce_building` whole — the
spiral, `find_friends`, the frame-0 free placement, the paid site with its
citizen ordered on at once. Their corrections to this document are
applied inline (§2.3 step 11's fort, §2.5's keys, §2.10's slots). What
they leave open is under §13.

**The complete inventory of `game_random` draws in the production AI**,
from a grep of every `Leader`/`Leaders`/`MakeList` function and the two
reports (diplomacy's 33 sites and `process_taunt`'s excluded):

| where | when |
|---|---|
| `random_personality` | 20 fixed + the nation and rival bends, once per computer leader at `Leader::init` |
| `Leader::init` 810/820 | the `economic`/`defensive` coin, when `pers.rush == 0` |
| `compute_sites` 198/212 | two per large region with my peasants, every sweep (frame 0 included) |
| `make_stuff` 110/252 | one per duplicate of the head's type / of a bought slot's type, every `make_stuff` |
| `research_techs` 588/603 | governments only: the `pers.raid == 0` coin and the `% 100` on the non-preferred pair |
| `create_units` 1621 | one per (city, land-military type) reaching the army-size gate, on every difficulty but 2 |
| `upgrade_units` 267 | one per eligible type, on every difficulty but 2 |
| `create_buildings` 1403/1404 | two per (city, wonder) on the no-shortcut path |
| `produce_building` 443/960 | one per friendless FARM/MINE spiral candidate; one per unblocked sub-position of the 2×2 jitter |
| `use_market` 72 | one when a good is short and wealth cannot cover a buy |
| the script's `rand_int` | eight at `defensive`'s first call in the game, then as written |

## 12. The implementation so far

**2026-08-24, third session — the census and the C++ producers landed.**
Seven modules were written in parallel, one worker each (Opus, `lean`),
from this document and the two producer reports, every value formula
re-read against the decompile as it was implemented; the main thread wrote
the shared surface first (`ai.rs`'s `Census`/`CityAi`/`Site`, `ai_types.rs`,
the world's `CellData`, the driver's calls) and integrated. §14 lists every
place a worker found the decompile disagreeing with §2 — **thirty-odd
corrections**, which are the second reading's material and are not yet
folded back into §2's prose.

- **`crates/sim/src/ai.rs`** — the driver's pure half: `Personality::roll`,
  `Leader::choose_script`, `MakeList::make_me`, the goods picture, the
  cadence, the `Step` machine, `Lobby`; and now the records the sweep
  writes — `Census` (every count under `LeaderData`'s names, the per-region
  arrays one slot per sim region), `CityAi` (the per-city `free`/`busy`/
  `gatherers`/`peasant_dist` and the circle picture), `Site` × 10,
  `mil_trainers`, `tech_frame`/`tech_cat_frame`.
- **`crates/sim/src/ai_types.rs`** — the shared type helpers: `Class`,
  `unit_record`/`build_record`, `num_buildings_of`/`num_sites_of`/
  `buildings_of_line`, `type_price`/`type_affordable`/`type_available`,
  `cities_of`/`city_chain`, `count_gather_slots` (`City::count_gather_slots
  @00737dc0`, knowledge excluded from the total), `village_num`, `sea_map`.
- **`crates/sim/src/ai_census.rs`** — `plan_strategy`'s sixteen sweep steps
  (§2.3) and `check_explore`; the seams tabulated in the module. **Pinned
  against the original's own leader record**: run9's frame-1 `LEADERDATA
  who 1` (`LEADERS=9`, §5) agrees with the harness's frame-0 sweep on the
  same map — `active 6, peasants 5, gatherers 5, free_peasants 0`, the
  per-region active/peasants/gatherers/cities, `reg_land 43`, `strategy 1`,
  `escrow_rate`, `filled_gather_slots` — with one ceiling: the woodcutter
  camp's five gather slots are `calc_gather`'s (§13).
- **`crates/sim/src/ai_sites.rs`** — `compute_sites`, `compute_site_stats`,
  `found_cities`, `produce_city`, `place_city_with_cost`'s body
  (`place_city_ai_impl`); `move_x/move_y[0..25]`, the ring `[81..121]` and
  `corner_x/y` read from the PE's `.rdata` (`rise_z.map` names them,
  `compass.obj`); §13's distance loop settled in the listing.
- **`crates/sim/src/ai_research.rs`** — `research_techs`, `produce_tech`;
  the eleven `ai[]` weights traced to `TechType::compute_ai_values@0066cdc0`
  (§14.4) and seamed to zero until the loader ports them.
- **`crates/sim/src/ai_units.rs`** — `create_units` (every branch: air,
  missiles, sea, land civilians, scout, land military, the shared tail),
  `upgrade_units`, `produce_unit`, `queued_units`, `check_income`,
  `unit_prod_value`; the type flags the sim does not carry are seams.
- **`crates/sim/src/ai_build.rs`** — `create_buildings` (both passes, every
  family), `produce_upgrade`, `produce_spell`; the gather multiplier's cap
  settled (§14.6).
- **`crates/sim/src/ai_make.rs`** — `make_stuff`, `make_this`, `use_market`,
  `market_speculation` (the market's gates and draws' *positions*; it trades
  nothing until `docs/ECONOMY.md`'s market lands), `check_orphaned_buildings`.
- **`crates/sim/src/ai_place.rs`** — `produce_building`, now with §4.6's
  census adjustments in place (run8 frame 2: `gatherers 5 → 4`).
- **`crates/sim/src/ai_drive.rs`** — the sweep calls the census, the orphan
  check and `compute_sites`; the cheap research tick; every step calls its
  producer; `effective_pop` computed on entry.
- **The world** (`world.rs`) carries the rest of each cell's `WData` —
  `CellData` (`flags`, `land`, `region2`, `val`, `goods`, `blocked`,
  `solid`…), `danger[who][region]`, `landmasses`, `tregion_alt` — and the
  harness fills it from a `WORLD ≥ 5` dump (`gamelog::world_cells`,
  `world_tiles`; `docs/ORACLE.md`, "The map is a dump too"). `home_reg` is
  set where the capital is founded (`init_city`; the original's
  `Setup::build_cities`).
- **The oracles** (`docs/ORACLE.md`): `LEADERS=9` is the census
  (`tools/gamelog/leader.py`), `WORLD=6` is the map — run9 has both for
  this lobby; `Log::leader_block(frame, who)` reads the record.
- **The score.** Run9 (36 frames, the map): the fourth farm lands on the
  original's tile and its builder `1/1` tracks the run — the first
  divergence on every earlier dump — order disagreements 60 → 43, path
  stacks 84 → 68. Run7 (flat): unchanged at 1,970 unlinked unit-frames,
  and now explained — with every cell's `val` 0 no site scores, the
  script's `city_placement` never succeeds, and the citizens of step 12
  are never trained. **Run10** (this lobby, the map, 1,772 frames, no
  input — `docs/ORACLE.md`): **744 unlinked unit-frames**, the three
  citizens on the original's frames and `1/1` tracking to frame 171; the
  744 are `1/9` and `1/10`, and the probe that explains them is the
  session's last finding — **the harness's players earn nothing**
  (`income [0; 6]`, food 2 for the whole run on the AI, 200 on the human):
  `economy::Holdings` was a hand-filled model that nothing assembled from
  the live cities and gather chains (`Leader::calc_gather`'s job,
  `docs/ECONOMY.md`). With no food the script cannot pay for the city at
  step 11 and the food-bound citizens of 1297/1505 never train. **The
  holdings assembly landed the same session** (`crates/sim/src/holdings.rs`,
  pinned against run8's frame-2 rate): run10 now scores **268 unlinked
  unit-frames** — the script gets through `city_placement` and step 12 on
  our map, `1/9` trains at **1297**, the original's own frame, and every
  unit the original has before 1505 exists here too. The 268 are `1/10`
  (1505), pinned as a ceiling.

### 12.1 Where to pick up

1. **The last citizen, `1/10` at 1505**, and the rest of the economy's
   terms that decide its frame: idle merchants and fishermen, the nation
   flat terms (`GERMAN_CITY_GATHER` is visible on run8's human), the
   bonus-tech levels (`GRANARY2..`, `TAX_1..` — the BONUS band the tree
   does not load), `calc_resource_bonuses`; and the sites' frames — `2007`
   at 776, `2008` at 1177 — against the dump's `BUILDDATA`.
2. ~~**The sync stream at frame 0.**~~ **Done, 2026-08-24** — read out of
   a dump: the setup path's `say_checksum` records print `game_random seed`
   at `check_all_level=14` (`docs/ORACLE.md`, "The setup path's checksum
   trace is the RNG state"; run11). `build_sim` installs the frame-0 state
   and seeds the personality from its bracket; the sampler's stride is the
   original's (`site_mark` 16 in both) and, with the terrain heights from
   run3's `DUMP_ALL` (`World::tile_z`, the `find_tcoord_z / 25` term of the
   5×5 slide), **run9's best site is the original's `(52, 14)/370`** —
   pinned in `diff.rs`. What it exposed is the item that replaces it:

   ~~**2′. The per-frame draws.** Run12's `DUMP_ALL` states show frame 0
   drawing **120** times where the sweep draws 2 (then 54, 6, 6 on frames
   1–3 — a steady six a frame that is not the AI's), so by the script's first
   `rand_int(1, 10)`s at frame 1 the stream is displaced and the script
   takes the *rush* order (two farms at step 3, no city) instead of the
   boom order the original took — run10 scores 744 on the true stream
   against 268 on the sim's own (a lucky branch).~~ **Read and landed,
   2026-08-24 (fifth session) — `docs/SYNC.md`.** Run12's four frames are
   attributed draw by draw with the LCG replayed against the dump's
   outcomes: the market at draws 2–19 (pinned by its flux values), the 40
   animals' idle anims at 48–87 (pinned by their variants), the herd at
   108–109, the farms at 110–115, frame 1's woodcutters and sprout at
   44–49 — and the steady six is `Farms::inc_time`, one draw per complete
   farm a frame, which is also the missing half of the farm clock
   (`FARM_GROWS` retired: the 201st `0.005f` add ripens, at frame 100, the
   re-target on the log's 102). Landed: `market.rs`, `farms.rs`,
   `gaia.rs` (birds' sampling, the herd walk, herds loaded from a
   `DUMP_ALL` sibling), and the harness's per-frame install (`Built::tick`)
   with the count on both sides: **frame 0 ours 48 / theirs 120; frame 1
   54 / 54; 6 / 6; 7 / 6**. The script's frame-1 branch is now the
   original's on run7 and run6 (pinned), and run10 holds 268 with `1/9`
   linking. What the 72 of frame 0 still are — the idle-anim draw per unit
   with the art's animation lengths (a table from the dumps' `GUY` blocks),
   the animals' wander, the scouts' `think_scout` scan, four unattributed
   — is `docs/SYNC.md` §6, and it is the next item here.
3. **The loader's half of the producers** — the seams that are `rondata`'s
   to close: `UnitType.unit_flags/unit_flags2/role/carry/cat`
   (`determine_roles@0061c320`, §14.5), `TypeDef.ai[11]`
   (`compute_ai_values`, §14.4), the market (`docs/ECONOMY.md`), and
   `gather_max` for non-flat buildings (`calc_gather`; a worker's report is
   pending). Each retires a named seam in one module.
4. **The producers' oracle.** No dump yet shows a non-empty make list: the
   script blocks the C++ steps for the whole of the opening. A run past the
   script's `SCRIPT_DONE` (Classical Age under `defensive`, ~step 29) with
   `LEADERS=9` on a few frames around a sweep is what scores `create_*`,
   `research_techs` and `make_stuff`.
5. **Fold §14 into §2**, then **the blind second reading** of the whole
   mechanic — thirty corrections from the implementation is exactly the
   kind of first reading the audit rule exists for.
6. The soak's AI leaders (a synthetic script, or the install), and
   `docs/ARMY.md`.

## 13. What is not established

- **The woodcutter camp's slot count** — `BuildData::gather_max`, written
  by `BuildTypeData::max_gatherers@0063c430` → `calc_gather@00639e40`
  (`docs/ECONOMY.md`'s open item): 5 for camp 2001 on run9's map, 0
  (uncapped) in the sim. Pinned as a ceiling in the census test.
- ~~**The frame-0 sync stream** (§12.1 item 2): every AI draw before the
  first `place_unit` — the personality, the script coin, `compute_sites`'
  stride — is on the wrong stream in the harness.~~ Read out of run11's
  trace and installed (§12.1). **What is on the wrong stream now is
  everything after frame 0's first draw** — the ~120 per-frame draws of
  units, herds, farms and ammo the sim does not model (§12.1 item 2′).
- **`compute_site_stats`' inputs the sim lacks**: ~~`find_tcoord_z`
  (heights, 0)~~ — pinned per tile from a `DUMP_ALL` dump's
  `master_land_heights` (`World::tile_z`; `(int)((h[ty+1][tx] +
  h[ty][tx+1]) × 0.5)`, 0 on ocean), `was_seen` (true — the lobby reveals
  the map), `danger[]` (0), team style 2's `target`, the ally-land arm;
  ~~**the `Region.coords` order** — the sampler walks the region's own list
  and the sim walks row-major~~ — the same: run3's `BEGIN REGIONS` prints
  every region's list and region 1's 3,053 coordinates are row-major
  exactly, as `Regions::rebuild_coords` writes them.
- **`unit_flags & 4/8/0x8000`, `unit_flags2 & 0x60`, `carry`, `cat`,
  `role`** — not on `UnitType` (§12.1 item 3); every producer test that
  needs them runs on a hand-built type.
- **`TechType::ai[11]`** — derived at load, seamed to zero: `w = 1`.
- **The market**: `use_market` computes its need and its gate and neither
  trades nor draws (the one draw sits in the sell branch, and which branch
  is taken is a price question), `market_speculation` clamps and tiers.
- **`build_masks & 1`** (the "sell me" mark) has no writer and no field;
  the orphan check's disband arm is unmodelled. **`city_flags 0x8/0x1000`**
  — no writer. **`CityData.ocean_filled`/`bordering`** — on `CityAi`, never
  written.
- **Meeting** (§2.3 step 6) is skipped whole; every other active leader
  counts as met, else steps 14–15 would be dead. A second opinion wanted.
- **Step 16, the army seeding**, is skipped (`docs/ARMY.md`).
- **`check_explore`'s extent** — the region grid at `(w/2)·(h/2)`, from the
  two index expressions; never checked against a dump.
- Two census verdicts sit on aliased decompiler locals and want the
  listing: step 13's inner cutoff (`i + 1 < circle_radius[..]`) and step
  15's operand order in the weaker test (§14.1).
- **The original's 32-bit overflow** in the value pipelines (§14.6, an
  Aztec barracks scores negative) is reproduced; whether it is load-bearing
  for the shipped AI needs a `LEADERS=9` dump past the script.
- **`leader_flags & 1`** — taken as "alive" by the sites; bit 2 is alive
  elsewhere.
- **`check_income`'s parameter order** — the stub's names are one off the
  original's `(t, mult, o, escrow, city, num, out)`; both callers pass the
  escrow flag in both slots until it is renamed.
- ~~`compute_site_stats`'s per-city distance loop~~ — settled: `return`s
  (§14.3). ~~The gather multiplier's cap~~ — settled: a `min` at ×1
  (§14.6). ~~`build_flags & 0x8000000`~~ — settled negative (§14.6).
  ~~`TechType::compute_ai_values`~~ — read (§14.4). ~~The personality is
  loggable but not at `LEADERS=9`.~~ ~~Which script run7's AI drew.~~ Both
  settled by run8 (§5). ~~The census adjustments at the end of
  `produce_building` are not kept.~~ Kept. ~~City names are synthetic.~~
  Still synthetic; behaviour identical.
- The rest of the earlier list stands: `WorldData::danger[who]`'s writer;
  `is_ally(who, who)`; the implicit variable's declared type; the
  placement's missing map layers *other than* `val`/`goods`/`region2`
  (which the map now supplies) — `buildings_allowed`, the enemy-seen flag,
  `gather_at` amounts, the oil patches; `space_at_corner`'s first-row
  early-out; `find_build_at_city` with `bool_count_inactive`; the raid
  stamp; `get_starting_town_size`'s nomad flag; the tick's origin; the
  `leader_flags` line's position in the dump.

## 14. Corrections from the implementation, by module (2026-08-24)

Each worker read its functions line by line against the decompile — and
the listing where the decompile printed a local that could not be right —
and these are the places §2 and the two reports were wrong or incomplete.
They are implemented as the decompile reads; §2's prose is **not yet
amended** and the second reading should treat this section as the first
reading's errata.

### 14.1 The census (`plan_strategy@006b9620`, `ai_census.rs`)

1. **Step 13, the radius** (line 1239): `city_center_radius + (level − 1)
   × city_center_pop_radius [+ indians_city_radius]`, capped `0x40` — the
   city's ordinary radius. §2.3 omitted the base; without it a level-1 city
   walks `circle_radius[1]` and `reg_land` is 0, against run8's 43.
2. **Step 13, the inner cutoff** (1319/1382): `i + 1 < circle_radius[k]`,
   so the last inner entry is excluded. On an aliased local; listing wanted.
3. **Step 13, `filled` vs `gather_at`** (1414): an unoccupied cell with
   `check_building_wcoord ≥ 4` runs `gather_at` and is *not* `filled`.
4. **Step 11, the defence guard** (1054, 1100): `domain != 1 && region <
   0x40 && attack != 0` — a water-domain building never counts.
5. **Step 10, the dock scan** (420) is gated on the *unit's* `domain == 1`:
   ships in docks, not citizens.
6. **Step 10, `reg_active`** (998) is under the domain-matches-region guard.
7. **Step 15, the default word** (1739): a non-thin region where I hold a
   city starts at **8**, not 0.
8. **Step 15, the weaker test** (1566): `my attack < theirs && (r ==
   home_reg || their reg_cities < mine)` — the city comparison the other
   way round from §2.3. Aliased local; listing wanted.
9. `find_city`'s `0x200` is a *flag* ("same region as the point"), not a
   radius — the search has no distance limit. `resources_controlled` is
   `popcount` of the `rare` words from `+0x6da0` (`+0x6d9c` is the count).
   `gather_slots[2]`/`gather_slots_high[2]` (wealth) are never written.
   `count_gather_slots`' return excludes knowledge. The human's
   `peasants`/`gatherers` come from `Leader::calc_gather@006ceee0`.

### 14.2 The make list (`make_stuff@006c8af0`, `make_this@006c94f0`, `check_orphaned_buildings@006c9f20`, `ai_make.rs`)

1. **§2.6 step 4 is inverted** (93–97, listing `0x6c8cbb–0x6c8ce2`): the
   *unconditional* expiry is `is(UNIVERSITY, 1)` **or a unit type that is
   not a peasant** — a military unit; the one-in-three draw is everything
   else.
2. **`make_this`' build arm has a missing predicate** (88; listing
   `0x6c957a` = `is(TOWN, 0)`): a Large/Major City routes to
   `produce_upgrade`, not `produce_city`.
3. **`build_flags & 0x10` is `NO_CITY`**, not "a gather building" (121);
   `GATHER` is `0x40`, which the slot-4 exception reads (194).
4. **"My territory at the site" is the danger map**: `world+0x13c` is
   `WorldData::danger[8]`, indexed `(y>>9)/3 × reg_xs + (x>>9)/3`
   (196–199, 228–232). The same misreading is in §2.12, §2.13 step 8 and
   §2.17's `produce_tech` (§14.3, §14.4).
5. A started site (`job_counter != 0`) in danger skips the recruit scan and
   only gets `build_masks |= 0x2000`. All four ranked slots are tested for
   a `(t, city)` duplicate (not "three tolerated"). When `can_pay(0)`
   holds, `site_mark` never advances. The good loop needs both the head's
   and the slot's cost non-zero. The space clamp reaches `space[3]`, which
   is `ter[0]` — a real overread, reproduced.

### 14.3 The sites and the city AI (`compute_sites@006cc950`, `compute_site_stats@006cd040`, `found_cities@006c7a60`, `produce_city@006cb120`, `ai_sites.rs`)

1. **The per-city distance loop's exits are `return`s** (listing
   `006cd9e3`, `006cd9f1` → the epilogue at `006cdc68`, past the stores):
   a dead city slot, or a city of mine outside the site's region, scores
   the site `val = 0, dist = 0`. §13's first item, settled.
2. **Step 13's filter** is every alive leader **not allied** (either
   side's `diplos != 2`), not "at war" and not "computer"; the cities are
   `city_flags & 0x11 == 0x11` — alive **capitals**, not "active cities".
3. Step 6's `city_num == 2` arm is the same block as `== 1` with cap 4:
   `(min(danger, cap) + 7) × v / 8`.
4. `rank[i]` counts all ten including `i`, no `+1` (numerically §2.7's).
5. **The insertion rule** (`006ccec8`): `min = 0, idx = −1`; per slot keep
   the old pair only if `min < val[j] && (min > 0 || new ≤ val[j])`, else
   take `(val[j], j)`. An empty slot is always taken; a beaten positive
   slot displaces an already-chosen empty one; a site beating nothing with
   no empty slot is dropped.
6. Step 1's and the slide's `0x100` read the **centre tile's `TData`**
   (`CITY_RADIUS`); step 3's danger/water `0x100` reads **`WData.flags`**
   (the coastal flag) — different records.
7. The 5×5 stays centred on the **original** cell after a slide; only the
   coordinates used by steps 5, 7, 8, 10–13 move. An early return after
   step 3 reports the slid coordinates with `val = 0`.
8. Step 8's "my territory" is `danger` (§14.2.4).
9. `produce_city`'s census decrement keys on the **last examined**
   citizen's action (`−0xc(%ebp)`, written per candidate before the region
   and distance tests), not the chosen one's. Its citizen test is the exact
   pair `0x32/0x33`, not the lineage.
10. `move_x/move_y[0..25]` is the 5×5 with the compass ring at 1..8 and
    the ring-2 border at 9..24; `[81..121]` is the 11×11 border clockwise
    from (−5, −5); `corner_x/y` = (0,0), (−1,−1), (1,−1), (1,1), (−1,1).

### 14.4 Research (`research_techs@006c6ba0`, `produce_tech@006ca980`, `compute_ai_values@0066cdc0`, `ai_research.rs`)

1. **Both knowledge gates read the stockpile** (`bucket[3]`, `+0xc ^
   0x8221`), not `resource_cap[3]`.
2. **The `k`-goods factor applies to epoch *and age* techs** (listing
   `0x6c743e–0x6c748c`), not to "a non-epoch, non-age tech"; `k` counts
   goods whose *income* is `≥ cap × 9/10`.
3. **The government polarity is inverted**: the named column `{0x26f,
   0x271, 0x273}` is the one *thrown away* under `raid < 0` or an odd coin;
   the survivor gets the `% 100` draw, then `×20` with no government. The
   coin is drawn **inside the per-tech loop**, once per government tech
   reaching the Senate branch.
4. **The Temple's `×400` is the window `0x24d..=0x250`** (Taxation,
   Vassalage, Social Contract, Income Tax), not `t < 0x251`.
5. `produce_tech`'s trainer score divides by `ObjectData::damage + 1`
   (`Build+0x24`) and both branches by `danger + 1` — not territory.
6. `cat` is `TechType+0x14` = `Line::index()` for epochs and **3 for every
   non-epoch tech**, ages and governments included.
7. **The eleven weights** (`TechType::ai[11]`, `+0x1cc`): zeroed by
   `TechType::init`, then `Types::init@00669cc0:1239` runs
   `compute_ai_values` over `0x220..0x274` ascending, and `add_preq_ai`
   adds into the *prerequisites'* arrays (cumulative, order-dependent). By
   index: `ai[0]` military (epoch cat 0; units with `role & 0x10000`;
   towers/forts/attacking buildings; trainers; dependants +1 epoch/+2 age;
   spells +4), `ai[1]` breadth (epoch cat 2; Village/Town; gather and
   `0x8000000` buildings; goods +4/+1; spells +2; bonuses), `ai[2]` naval
   (docks, sea units, with `add_preq_ai(2, 1, −1)` and `(2, 1, 2)` up the
   chain), `ai[3]` transports, `ai[4]` research/commerce (epoch cat 3;
   dependants), `ai[5]` cities (epoch cat 1; Temple +2, Fort +1, Town +1),
   `ai[6]` sea/air units, **`ai[7]` never written**, `ai[8]` land military,
   `ai[9]` resources (citizen-role units, gather buildings and sheds,
   goods), `ai[10]` epoch cat 3 only; `s = (BuildType+0x3c < 0) ? 2 : 1`.
   A loader port; seamed to zero meanwhile.

### 14.5 Units (`create_units@006c40a0`, `ai_units.rs`)

1. **The army-size ladder differs by branch**: rung 2 is `×3/2` in the air
   branch (442) and `×2` in the land branch (1430).
2. **The military gate is `role & 0x10000` for every domain** (292), so sea
   and air military take the wonder/domain mod and the draw — more draws
   than the report's table says.
3. `city_flags & 2` is `no_heal` in the sim's naming, not `alarm`.
4. `check_income`'s original order is `(t, mult, o, escrow, city, num,
   out)`. The value arithmetic wraps (32-bit `imul`); the tail's `< 0 →
   9,999,999` is its own guard.
5. What is not on `UnitType`: `unit_flags` (`+0x2b4`), `unit_flags2`
   (`+0x2b8`), `role` (`+0x2c8`, `determine_roles@0061c320`: `0x200` for
   ids 0..=3; air `0x1000`, land `0x40000` (+`0x10` scout, +`0xc` cat 1),
   sea `0x80000` (+`0x10` bark); `0x8000` if `carry`; military `0x10000`
   when `attack != 0`, `cat ∉ {4, 5}` and not `0x200` — sea +`0x2000`, air
   +`0x4000`, land cat 0 +`0x800`, hoplites `0x100000|2`, ranged +`0x400`
   else +1; otherwise `0x100`), `carry` (`+0x2d4`), `cat` (`+0x14`).

### 14.6 Buildings (`create_buildings@006c1be0`, `ai_build.rs`)

1. **The gather multiplier's cap is a real `min` at `0x100`** (listing
   `0x6c2762–0x6c276b`: `cmp; cmovl; mov` — the running cap ratchets
   down). The `×2` terms decide whether it *reaches* ×1; the report's
   "garbage `extraout_EDX`" is the decompiler's, the register is reloaded.
2. The dock's value **is** multiplied by `world+0x34` (1241, listing
   `0x6c331d`); a passing dock runs §3.6–§3.10; a dock failing `sea_map >
   2 || city_num > 1` is dropped without `make_me`.
3. `get_queued(t)` recurses only for a unit type; for a building it is
   `num_queued[t]` exactly.
4. §3.7's `reg_wars` is the byte array at `+0x6836`, not `+0x9b4`
   (`active_wars`). `get_enhancing_good` is an exact-type switch.
5. **`build_flags & 0x8000000` is never set**: no `BUILD_FLAGS` string
   carries a digit, `init_final_flags` sets only `FLAT`; the Temple/
   Library/Senate arms of the civic block are dead and only the Market arm
   runs (kept behind `flags::DEEP_QUEUE` for a loader that sets it).
6. **The original's value product overflows in play**: `9,999,999 × 0x100
   > 2³¹`, so an Aztec's first Barracks (`1000 × 3 × 400 × 100 × 4`) comes
   out negative and loses its category slot; a Market overflows inside its
   own `×10000`. Reproduced with wrapping arithmetic.

### 14.7 `produce_building` (`ai_place.rs`)

The census adjustments of report §4.6 (1114–1151) are kept: `filled += 1`,
`space[n − 2] = max(0, · − 1)` for `n = 2..best_sp`, then the builder off
the gatherers (`gatherers`, `reg_gatherers`, `city.gatherers`) or the free
peasants — run8's frame 2 shows `gatherers 5 → 4`.
