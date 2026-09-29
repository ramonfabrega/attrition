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
    if (unsigned)(t - BASE_AGETYPES) > 0x54: return  # only an age or a tech (0x220 ≤ t ≤ 0x274)
                                                     # UNSIGNED — `cmpl $0x54; ja`. Ghidra prints it
                                                     # signed, where every unit and building type
                                                     # (t < 0x220 → negative) falls through into the
                                                     # buy path. Second reading, B6-a.
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
regions are `< 0x40`, sea regions `≥ 0x40` (~~`0x3f..0x7e`~~ — the
boundary is `Region::is_coast`'s, `docs/TRANSPORT.md` §9; run20's one
ocean is 65) and are stored `% 0x3f` in the 63-entry arrays.

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
   `reg_terr`, `reg_buildings` (kept by `gain_/lose_building`), and
   `reg_docks` (~~`gain_/lose_building`~~ — kept by `Dock::init` /
   `Dock::close`, `docs/TRANSPORT.md` §5.2–§5.3; those two only call
   `check_transport`).
4. **Territory.** `my_team_terr = get_team_terr()`; over every other
   ~~computer~~ leader (`&2`, §43) not allied both ways (`diplos[i] != 2`
   on either side): `other_team_terr = max`, `min_other_team_terr = min`.
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
    - its region: the unit's tile, or its outermost container's (§58)
      — the building's footprint (`x_size`/`y_size`, centred
      by parity) is scanned column by column for the first cell whose
      terrain byte (`world+0x138`) has `& 0x30 == 0x20`, **water**, and
      that tile's *alternate* region (the record's short at `+6`, the sea
      region of a coastal tile flagged `0x100`) is taken; no such cell →
      the building's own tile region. So a unit inside a dock counts in
      the sea region. (The `create_units` reader's dock scan uses the same
      test and requires the result `≥ 0x3f`, which settles the sense.)
      **The dock scan is gated on the *unit's* `domain == 1`** (line 420):
      ships in docks, not citizens.
    - `active++`, **`reg_active[r]++` only under the
      domain-matches-region guard** (998); strength `s = attack() / 10`
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
    - **citizens** (`0x32/0x33`): `peasants++`; the nearest friendly city
      (`ObjectsData::find_city`, its distance in `objects+0x1fc`) — where
      **`0x200` is a *flag*, "same region as the point", not a radius, so
      the search has no distance limit**. Off the map: inside a `0x1a6` → `filled_gather_slots[5]++`;
      in a sea *unit* that is moving → `xport` in the move point's
      region, which `reg_active` reads (§58). On the map:
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
      its `gatherers++` and `peasant_dist` (§73), `reg_gatherers[r]++`;
      `8` or `0xe` (build/repair through a boarded transport) →
      `reg_xport_peasants`, `xport_peasants++`.
11. **The building census** — every active building of mine (the build
    and wall lists): a finished gather building (vslot `+0x90` on its
    type) adds its `gather_max` (`Build+0x80`) to `gather_slots[good]`;
    every active building with `type+0x1e8` (`attack`) set counts `defense
    += 1` for a tower (`is(0x1b7)`) or `2` for a fort (vslot `+0xfc` =
    `is_fort`, `docs/CITIES.md` §1.5 — corrected from "has arrows" by the
    `create_buildings` reading), and `reg_defense[r]` the same. **The guard
    is `domain != 1 && region < 0x40 && attack != 0`** (1054, 1100) — a
    water-domain building never counts. `gather_slots[2]` (wealth) is never
    written, and `City::count_gather_slots`' return excludes knowledge.
12. **Maxima**: `gather_slots_high[g]`, `peasant_high`, `scholar_high`,
    `caravan_high`, `merchant_high`, `army_high (= max combat)`,
    `city_high`, `village_high`, `population_high` — each `max(old, new)`
    (`gather_slots_high[2]` therefore never moves either).
    `resources_controlled` = **`popcount` of the `rare` words from
    `+0x6da0`** — `+0x6d9c` is the count, not the mask.
13. **Per-city site picture** (`compute_site_stats` proper is §2.7's; this
    is the sweep's own): `city_pop_high = max(city.pop)`; radius =
    **`city_center_radius + (level − 1) × city_center_pop_radius`** — the
    city's ordinary radius, and the **base term is load-bearing**: without
    it a level-1 city walks `circle_radius[1]` and `reg_land` comes out 0
    against run8's 43 (line 1239) — with level 1/2/3 for a Small/Large/Major
    or `0x213` city, `+ indians_city_radius` with tribe bonus `0x15`, capped
    at `0x40`, then `circle_radius[(radius + 2) / 4]` picks the even circle
    (`docs/CITIES.md` §3.6); zero `ocean, land, filled, dock_tile, space[3],
    ter[6]`; `reg_gather_slots[r] += City::count_gather_slots(city)`; a city
    with flag `2` (attacked) → `attacked++`, `reg_attacked[r]++`. Then over
    the circle's tiles that are mine or unowned: a water tile → `ocean++`
    and, if its region has `> 1` coasts or is ≥ a tenth of the map,
    `dock_tile++` for the first `is_dock_tile`; a land tile inside the
    inner radius (**`i + 1 < circle_radius[k]`, so the last inner entry is
    excluded** — 1319/1382, on an aliased local, listing still wanted) and
    in the city's own region: unoccupied (`flags & 0x70 == 0`) → `land++`,
    `check_building_wcoord` gives the largest square that fits there →
    `space[n − 2]++` for `n = 2..4`, and `filled++` if `< 4` — **a cell
    with `check_building_wcoord ≥ 4` instead runs `gather_at` and is *not*
    counted `filled`** (1414); occupied → `World::gather_at` and
    `ter[g] = max(ter[g], amount)`. After:
    `land − filled < 2` → `full_cities++`; `reg_land[r] += land − filled`.
14. **Wars/allies**: over every other met computer leader: `diplos` 0 on
    either side → `wars++`; 2 on both → `allies++`.
15. **Per-region strategy** (`strategy[r]`, a bit word per land region;
    `reg_wars/reg_allies/reg_neutrals[r]` zeroed and recounted): bit 1 =
    the region is *thin* (all leaders' `reg_cities[r] × 50 < region.size`).
    **A non-thin region where I hold a city starts its word at `8`, not 0**
    (1739). If I have a city there: for every other met leader with a city
    there — at war: `active_wars++`, `active_wars_with |= 1 << i`,
    `reg_wars[r]++`, and **bit 4 if I am weaker** — `my attack < theirs &&
    (r == home_reg || their reg_cities < mine)`, the city comparison **the
    other way round** from the first reading (1566; aliased local, listing
    still wanted) — else **bit 2**; then difficulty `< 2`
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
    where armies come from; `docs/ARMY.md` owns `Army::init` and the rest
    of the family (2026-08-25); `army.rs::census_seed_army` is this step.
17. **Tail**, unless `semaphore[1] & 2`: `check_orphaned_buildings()`
    (§2.8), `compute_sites(0)` (§2.7), **`production_step = 1`**.

Two sync-stream facts: the sweep itself draws no `game_random`; `compute_
sites` in its tail does (§2.7). And `check_orphaned_buildings` can issue
orders, so `docs/ORDERS.md`'s "the frame-0 sweep issues no order" (audit
F4) is true only because a fresh game has no orphan sites.

One field this sweep does **not** produce: a human leader's `peasants` and
`gatherers` come from `Leader::calc_gather@006ceee0`, not from here. Two it
zeroes and does not own between sweeps: `xport_peasants` and
`reg_xport_peasants` are also incremented by
`Unit::think_civilian_transport` each time it dispatches a colonist
(`docs/TRANSPORT.md` §7), so a mid-cycle record can read higher than step
10 alone would give.

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
`make_stuff` spends from. `t = −1` is an empty slot — and a *fresh* one
is the whole record `t −1, val −1, escrow 0, city −1, up 0, o −1, num 1,
cat 0, wx 0, wy 0`, which `Array<MakeObject>::init@0047d300` fills at
allocation and `MakeList::clear@006c9db0` rewrites, word for word, into
all eleven (§15.7). The `val −1` matters: `make_me` inserts on `val <
offered`, so a fresh slot takes an entry offered at `val 0` and a merely
expired one does not.

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
   a wonder, or ~~is *not* a military unit type~~ — **corrected: or *is* a
   university (`is(0x1a4)`) or a unit type that is not a peasant, i.e.
   exactly a military unit** (`make_stuff:93–97` reads `if (!is(UNIVERSITY,1)
   && (!is_unit_type() || is_peasant())) goto <probabilistic>`, listing
   `0x6c8cbb–0x6c8ce2`; and
   **observed** — an ordinary building takes the probabilistic arm in
   run18b, five rolls out of five, §15). One sync draw per matching slot,
   every `make_stuff`, whether or not anything was bought. The walk is
   `make_stuff+0x221`; the slot loop's own is `+0x63d` (§15).
5. **Saving**: if not `paid`, `need = average over available goods with
   `cost[g] > 0` of `(cost[g] − resources[g])`, floored at 0` — the mean
   shortfall.
6. **The rest of the list**, slots 1..10: skip empties and `val == 0`;
   from slot 4 on, skip a slot whose `(t, city)` duplicates **any of the
   four ranked slots 0–3** (all four are tested — not "three tolerated");
   then for each available good with
   `cost > 0`: if `resources[g] < cost_of_slot[g] + cost[g] + need` the slot
   is unaffordable-while-saving → skip, **except** slot 5 when I have no
   free peasants and no gatherers (buy anyway), and slot 4 when it is a
   gather building (`build_flags & 0x40` — **`GATHER` is `0x40`; `0x10` is
   `NO_CITY`**) for a good whose `econ` has bit 4
   — buy anyway; slots that pass: `can_pay_cost(who, city, o, escrow)`
   (vslot `+0x84`) `≥ num` → `make_this(slot)`, then the same 1/3 expiry
   over duplicates of *its* type (unconditional for tower/`0x1bb`/upgrade/
   wonder), only while `slot < 11`. The good loop needs **both** the head's
   and the slot's cost non-zero.
7. Return: `site_mark++` if nothing in the pass was an upgrade-kind —
   **and never when `can_pay(0)` held**; return "bought the head".

**`make_this@006c94f0`** (read, 187 lines) dispatches one slot by its
type's kind: a **build type** in `0x19e..0x21e` → `type_avail == 4`
required (else `produce_tech(t, escrow)` — i.e. the tech that unlocks it
is what gets bought); `up != 0` → `produce_upgrade(t, city, escrow)`; a
city type (`is(0x60)` vslot) → `produce_city(t, wx, wy, escrow)` unless the
site's region has no free peasants and no gatherers of mine — **but a
Large or Major City (`is(TOWN, 0)`, the predicate the first reading missed;
line 88, listing `0x6c957a`) routes to `produce_upgrade` instead**; a
wonder or
ordinary building → `produce_building(t, city.o, escrow)` unless the
city's region has no free peasants/gatherers, or the building is a land
building that is not a gather building and the city has `< 2` open tiles
(`land − filled`) with a footprint that fits (**`space[max(x, y) − 2]`
clamped to `0..3` — the clamp reaches `space[3]`, which is `ter[0]`, a
real overread and reproduced as one**) `< 2`; a **tech** in `0x220..0x274` →
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
keep = 1)` and inserted. **The insertion rule, exactly** (`006ccec8`):
`min = 0, idx = −1`; per slot, keep the pair already chosen only if
`min < val[j] && (min > 0 || new ≤ val[j])`, else take `(val[j], j)`. So an
empty slot is always taken; a beaten *positive* slot displaces an already
chosen empty one; and a site that beats nothing, with no empty slot
anywhere, is **dropped**. A tile already present with `val > 0` is skipped
first. Finally **`rank[i]` counts all ten sites with `val ≤ val[i]`,
including `i` itself and with no `+1`** — numerically the same as the first
reading's "1 + those it beats", by a different route. `compute_site_stats` (506 lines) — the site score itself —
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
  skip. If no builder: `job_counter == 0` (never started) and **the danger
  at the site** `> 0` → **disband the site**; danger `≤ 0` → the first
  idle (`action == 0`) or gathering (`7`) citizen in the site's region is
  sent with `Group::action_swarm_around(site, QUEUE_NEW, BUILD_AT, 1)`; if
  none, `build_masks |= 0x2000`. **A site already started (`job_counter !=
  0`) that is in danger skips the recruit scan entirely and only takes the
  `0x2000` mark.**

**"My territory at the site" was a misreading, here and in §2.12, §2.13
step 8 and §2.17.** `world+0x13c` is **`WorldData::danger[8]`**;
`docs/DANGER.md` has the writer, the index, and the one thing these
readers do differently from `calc_cost` — they take the grid **whole**.

### 2.9 What the driver leaves to the producers

Written when none of these had been read. **Most of them now have their own
section**: `compute_site_stats` §2.13, `found_cities` §2.12,
`research_techs` §2.14, `use_market`/`market_speculation` §2.15,
`queued_units` §2.16, `produce_tech`/`produce_city` §2.17,
`create_units` §2.18, `create_buildings` §2.19, `produce_building` §2.20.

**Still unread at document level**: `upgrade_units` (344),
`produce_unit`, `produce_upgrade`, `produce_spell`, `check_income`,
`unit_prod_value` (114), `check_transport` (124), and `Leader::init`'s
AI-relevant fields beyond the ones named in §2.4. They are implemented —
`upgrade_units` and `produce_unit` both run in run18/run19 — but the only
written derivation is in the module docstrings, so the blind second reading
has nothing here to compare against and should treat them as new ground.

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
good" and "slot 5 when I have no free peasants and no gatherers" — ~~slot
5's category is the unit readers' to name~~ — **named by run18b (§15):
slot 5 is the citizen's**, `create_units` passing `cat 5` for a peasant,
and **slot 8 an ordinary civic building's** (`create_buildings`, the
Temple). ~~`MakeList::clear` sets every `t = -1`~~ — **corrected
2026-08-25 (§15.7): `MakeList::clear@006c9db0` rewrites every slot whole,
to the init record of §2.6 (`val −1`, `num 1`, the rest cleared); only the
*expiry* in `make_stuff` (§2.6 step 4, `make_stuff@006c8af0:112` and
`:254`) writes `t = −1` alone and leaves the other nine fields standing.**
`MakeList::init` allocates the eleven, already filled with the same
record.

**The whole shape is observed** (§15): a dump with entries at slots 0, 5
and 8 and nowhere else is this insertion exactly — the citizen at its
category slot 5 and, until the temple outbid it, at rank 0; the temple at
rank 0 and category slot 8; and the citizen's rank-0 copy simply gone,
because a new best overwrites the head instead of shifting it down.

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
  ~~many-landmass map~~ **class-4 sea map** (`world+0x34` is the style's
  `SEA_MAP` class, not a landmass count — §15.8), expand only off the
  home region;
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
- `v ×= 10` (a negative product clamps to 999,999); **the danger at the
  site's cell** (`WorldData::danger`, §2.8 — not territory) `> 0` → `/3`;
  the tile's owner not me → `/3`;
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

**`place_city_with_cost@009f5860`**, the script's way in, is the same two
calls fenced (listing `009f5898`–`009f58e8`): **return −1** unless
`total_cities < city_limit`, then `compute_sites(0)` — the argument is a
literal zero, `push $0x0` at `009f58bd` — then `MakeList::clear()`,
`found_cities()`, `MakeList::clear()` **again**, and return whether
`total_cities` rose. Both clears are unconditional, where the tail above
clears only on a buy: the offers `make_stuff` prices are the sites and
nothing else. The guard is load-bearing for the draw count —
`defensive.bhs` step 11 calls this five times in one frame (`num_loops`),
and each call after a successful buy returns −1 without a draw, so the
frame's word says whether the city was bought (`rondata::diff`,
`run40_s_census_prices_the_ai_s_second_city_at_sixty`).

### 2.13 The site score — `compute_site_stats@006cd040`

Read whole (506 lines). `compute_site_stats(wx, wy, city, unit, reg, &val,
&dist, keep, &out_wx, &out_wy)`; `val = dist = 0` on every early return:

1. **Two separate early returns, not one** (second reading, B4-b/B4-c). First
   the centre tile's `TData` mask at `(4wx+2, 4wy+2)` — four tiles per cell —
   against `0x100`, which the PDB's own field list `0x5802` names
   **`MASK_CITY`**, not `CITY_RADIUS`; step 3's danger/water `0x100` reads
   **`WData.flags`** (the coastal flag): different records, same constant.
   Then, separately, `was_seen(2wx+1, 2wy+1, who)` on the fog grid — which
   the first reading denied and which is really there — **and which is not
   "always true" on any lobby run so far** (`REVEAL_MAP 1`, "Normal"):
   `WorldData::was_seen@006b53f0` is true for `reveal_map > 1` or `who >
   7`; else, unless the leader has flag `0x1000`/`0x800` or a
   `types[0x141]`, **a cell owned by an ally — `is_ally` is reflexive, so
   one's own land — is seen whenever that owner's `reg_cities[region]` or
   `reg_forts[region]` is non-zero** (`LeaderData +0x125e`/`+0x12de`);
   else `seen2[(2wy+1) × fog_xs + 2wx+1] & ally_mask`, the fog grid the
   WORLD dump prints as `seen2[scan]` (§15.8, run20: all ten of the AI's
   sites have `seen2 == 0` and are scored through the territory arm);
   the tile owned by another leader (unless CtW with no cities of mine
   and an ally's — then `ally_land = 1`); `is_ocean` → 0 —
   **`WorldData::is_ocean@006b4830` is the cell's own kind**, `land == 1`
   (SANDY, the shallows) or `2` (OCEAN) and not flagged `0x100`, not its
   region's terrain (§15.8: the beaches of an island are SANDY cells inside
   a land region).
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
   (`keep = 0`) stays. **The 5×5 itself stays centred on the *original*
   cell after a slide** — only the coordinates steps 5, 7, 8 and 10–13 use
   move — and an early return after this step reports the *slid*
   coordinates with `val = 0`. **Correction (second reading, B4-k): step 5
   is *not* in that list.** The relocation writes `param_1`/`param_2`, but
   the ring block reloads the saved originals (`iVar8 = local_14; iVar3 =
   local_18`, lines 233–234) immediately before walking, and `WVar6`/
   `WVar13` are only refreshed from the moved pair *afterwards* (253–254).
   So **the coastal ring is centred on the original cell**; steps 7, 8 and
   10–13 do use the moved one. `ai_sites.rs` had this wrong and is fixed.
   Also: `q`'s 2×2 sum degenerates at the map's right or bottom edge, where
   `valid(nx+1, ny+1)` fails and a single cell's `val` is still `>>2` — a
   quarter, not an average (B4-e).
4. `base < 1` → 0. `parity = base & 3`.
5. ~~Many landmasses~~ **A sea map** (`world+0x34 > 2` — the style's
   `SEA_MAP` class, §15.8) and I own no dock (`num_buildings[
   DOCK] + get_buildings(its upgrade)` = 0): any of the **40 offsets**
   `move_x[81..120]` (the outer ring) on ocean → `base ×= 30`. **Observed
   on run20** (East Indies, `sea_map 4`): the ten-record `SITES` diff
   matches slot for slot with the ring centred on the original cell and
   moves a whole slot when it is centred on the slid one (§15.8).
6. `v = base × 250 / (water + 1)`; `city_num == 1` → `v = v × (min(danger,
   9) + 7) / 8` if `danger`; **`== 2` is the same block with the cap at 4**
   — `(min(danger, 4) + 7) × v / 8`, not a bare `min`; else `forts` →
   `×3/2` — the `else if` chain makes the fort bonus **mutually exclusive**
   with the danger caps. `v = (target_adj + 1) × v / 2`, and **`target_adj`
   starts at 1, not 0** (line 89; incremented only in the assassination-target
   arm), so a site with no target-owned neighbour takes ×1 rather than ÷2
   (second reading, B4-a; `ai_sites.rs` already had it right).
7. Map-edge penalty unless map type `world+0x30` is `0xc`/`0x11`: with `<
   3` cities, `wx` outside `[w/5, 4w/5]` → `/4`, `wy` outside `[h/5,
   4h/5]` → `/4`; with `≥ 3`, the bands are `1/10 .. 9/10`.
8. `v /= max(1, d)` where **`d` is the danger at the site's cell**
   (`WorldData::danger`, §2.8; `docs/DANGER.md` is the writer, and a
   building weighs for every viewer including the owner, so the grid goes
   negative on one's own ground. Second reading, B7-a); then `WData+0x10`, which is **`who2`, the *second-strongest*
   territorial claimant** — written beside `who` in
   `World::compute_reg_territory@006b0bb0`, not "the territory owner" as the
   first reading had it (B4-d): unowned → `×2`, a non-ally → `×4`, an ally →
   `/2`. Since step 1 has already required the *primary* owner to be me or
   nobody, the ×4 arm means "an enemy has secondary border pressure here" —
   the scorer rewards **forward-settling**.
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
    more than two cities); `dist = 2k²`. `v ×= k`. ~~**Unresolved**: …
    whether those are `break`s.~~ **Settled in the listing**
    (`006cd9e3`, `006cd9f1` jump to the epilogue at `006cdc68`, past the
    stores): they are **`return`s**. A dead city slot, or a city of mine
    outside the site's region, ends the whole function with
    `val = 0, dist = 0`.
13. Enemy proximity (not maps `0xc`/`0x11`, and `city_num == 1` or team
    style 2): over **every alive leader not allied with me** — either
    side's `diplos != 2`, *not* "at war" and *not* restricted to computer
    leaders — (or, team style 2 with ≥ 2 cities, only my `target`): for
    each of their **alive capitals** (`city_flags & 0x11 == 0x11`, not
    "active cities"), `sum +=
    vector_dist(site, city) / num_nations`; `×100` in team style 2; `v /=
    sum` if non-zero — **closer to the enemy scores higher** when I have
    one city.
14. `ally_land` → `/10`. `val = v`, `dist` as above.

The tile record (`world+0x134`, `0x1c` bytes a tile): `+0x4` region,
`+0xc` site value, `+0xd` nearby-goods bits, `+0xf` owner, `+0x10`
territory owner, `+0x28` = the next tile's `+0xc` (used by the 2×2 sum).

**The offset tables**, which steps 3, 5 and 11 index:
`move_x/move_y[0..25]` is the 5×5 — the compass ring at 1..8 and the
ring-2 border at 9..24; `[81..121]` is the 11×11 border, clockwise from
`(−5, −5)`; `corner_x/y` is `(0,0), (−1,−1), (1,−1), (1,1), (−1,1)`.

### 2.14 Research — `research_techs@006c6ba0`

Read whole (689 lines). Capital-countdown elimination (`elimination == 1`):
the gate is `find_capital(&city, &owner, −1, −1); if (owner != who) return`,
which is on the **owner**, not the city index — so the layer is skipped
**iff another leader currently holds my capital**, and a leader with **no
capital at all is not skipped**. `LeaderData::find_capital@006eb930` writes
`*param_2 = this->who` both when I hold my own capital and on the
no-capital fall-through (`*param_1 = −1`, then `LAB_006eba39`). The first
reading's "only while I hold my capital" is inverted for the no-capital case
(second reading, B3-a). `low = get_lowest_epoch()` (the least of
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
- **Ages** (`0x220..0x226`): knowledge cost `> bucket[3] × 4/3` → skip —
  **both knowledge gates in this function read the *stockpile*
  (`bucket[3]`, `+0xc ^ 0x8221`), not `resource_cap[3]`**. **Human pacing**
  unless `leader_flags & 8`: `h =
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
- Everything: knowledge cost `< bucket[3] − 100` → `base ×= 10` (the
  stockpile again).
- **Weights** (`TechType.ai[0..10]`): `w = 1 + ai[0]/10 + Σ ai[1..10]`;
  `± ai[10]` by team style (+ for 0/8/0xb); `full_cities > (cities +
  villages)/2` → `+ 2·ai[5]`, else `full_cities` → `+ ai[5]`;
  `my_team_terr < other_team_terr` → `+ ai[5]` (×2 if `< min_other`);
  `active_wars == 0` → `+ ai[5] + ai[1]`, else `+ ai[0]/3`; ~~landmasses~~
  the sea class `m = world+0x34` (§15.8): `m == 0` → `− ai[2]`, else
  `+ ai[2] × m³`. `val = w × base`.
- **Category** (`t.cat` = `TechType+0x14` — `Line::index()` for an epoch
  tech and **`3` for every non-epoch tech**, ages and governments
  included): `0` → `/10`, then with `pop_cap < 200`:
  `effective_pop > pop_cap × 5/6` → `×20`, else `epoch[0] > 1` → `/10`.
  `2` → `×3` if `epoch[2] == 0`; and **for epoch *and age* techs alike**
  (listing `0x6c743e–0x6c748c` — not "a non-epoch, non-age tech"), `k =`
  the number of goods whose **income** is `≥ cap × 9/10` → `val = k ? val ×
  k² : val / 10`. Otherwise, **difficulty ≥ 4 and `cat == 3`** only: the
  gather-rate lines `0x264–0x266` (food), `0x261–0x263` (wood),
  `0x26c–0x26e` (metal), `0x267–0x26a` (knowledge) → `×2` for `econ[g] &
  4`, `×4` for `& 2`, `×8` for `& 1`; then `d = epoch[3] − t.age > 0` →
  `×(d + 1) × d`.
- Victory 9 → `×3` (`×9` for an age).
- **Slot** by `cat`: `0 → 10`, `1 → 9`, `2 → 4`, else `8`.
- **By building**: `where == TOWER` → skip below difficulty 2, else
  `×100`, slot `6`; `0x1b5` (the Temple) → `×4`, and **`×400` only inside
  the window `0x24d..=0x250`** — Taxation, Vassalage, Social Contract,
  Income Tax — not "while `t < 0x251`"; `0x1b3` →
  `val = (max(epoch[0..3], 0) × val + 1) / (epoch[cat] + 1)`;
  `UNIVERSITY` → `×1000`; `0x1b6` (governments), **with the polarity the
  other way round from the first reading**: `pers.raid < 0`, or `== 0` and
  an odd **`game_random` coin**, *discards* the named column
  `{0x26f, 0x271, 0x273}`; the **survivor** takes the `% 100` draw and then
  `×20` if I have no government yet. The coin is drawn **inside the
  per-tech loop** — once per government tech that reaches this branch, not
  once per call.
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
0`) and the `% 100` for the surviving government — both only for
techs researched at building `0x1b6`, and both once **per such tech**.
**Observed** (run19, §15.6): with no government tech in range, one whole
`research_techs` call takes **no `game_random` draw at all**, and its
`val` reaches the `9,999,999` coverage clamp in ordinary play.

**The eleven weights** the `w` line above sums — `TechType::ai[11]` at
`+0x1cc` — are derived at load, not in this function: `TechType::init`
zeroes them, then `Types::init@00669cc0:1239` runs
`compute_ai_values@0066cdc0` over `0x220..0x274` **ascending**, and
`add_preq_ai` adds into the *prerequisites'* arrays. By index: `ai[0]`
military (epoch cat 0; units with `role & 0x10000`; towers, forts and
attacking buildings; trainers; dependants +1 for an epoch and +2 for an age;
spells +4), `ai[1]` breadth (epoch cat 2; Village/Town; gather and
`0x8000000` buildings; goods +4/+1; spells +2; bonuses), `ai[2]` naval (docks
and sea units, with `add_preq_ai(2, 1, −1)` and `(2, 1, 2)` up the chain),
`ai[3]` transports, `ai[4]` research/commerce (epoch cat 3; dependants),
`ai[5]` cities (epoch cat 1; Temple +2, Fort +1, Town +1), `ai[6]` sea and
air units, **`ai[7]` never written**, `ai[8]` land military, `ai[9]`
resources (citizen-role units, gather buildings and sheds, goods),
`ai[10]` epoch cat 3 only; with `s = (BuildType+0x3c < 0) ? 2 : 1`.

~~This is a **loader port** and is seamed to zero until `rondata` builds
it, so every weight reads `w = 1` today (§13).~~ **Built and checked,
2026-08-25** — `sim::ai_load::compute_ai_values`, and **all 85 techs' eleven
weights equal the program's own** (`docs/DATALAYER.md` has the derivation in
full; the oracle is `TechType::log_data`'s `ai[scan]` lines in a `DUMP_ALL`
type dump). Four things in the sketch above needed correcting to get there,
and they are the kind a reading alone does not catch:

- **`ai[6]` is air alone**, not "sea and air": the non-land arm sends sea to
  `ai[2]` and everything else to `ai[6]`.
- **A non-epoch tech researched at the Fort takes `ai[0] + 1` as well as
  `ai[5] + 1`** — the `+1` falls through into the military weight, where the
  Temple's `+2` returns.
- **The bonus types add `ai[4] + 2`** as well as the `ai[1]` the sketch has,
  and the *first* bonus scores `ai[1]` twice.
- **The unit loops stop at `BASE_GAIATYPES`.** The twelve animals name the
  Large City as their `WHERE`, so a loop that includes them adds twelve to
  the Medieval Age's `ai[1]` that the original never adds.

### 2.15 The market — `use_market@006c91c0`, `market_speculation@006c8110`

Both read whole. Both require the market ability — tribe bonus 4 or
`has_preq(BUY_SELL)`, which is **Coinage** (§40) — a market
(`has_market`) and no nuclear embargo (`get_nuke_embargo`, else
`tell_embargo` and nothing); `do_buy` / `do_sell` are
`docs/ECONOMY.md`'s, called once each (`do … while (i<1)` is one try).

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
halved if `is_unassimilated`, divided by `(d + 1)` where **`d` is the
danger at the building** (`WorldData::danger`, §2.8 — not territory); the
best wins. A military trainer is searched in `mil_trainers`
(`+0x6e50`, the leader's own list) with score `10⁶ / (queue.length + 1) /
(damage + 1)` — **`Build+0x24`, `ObjectData::damage`, the building's own
damage and not a territory term** — and the same halving and danger
division. None → 1. Else
`Build::queue_up(building, t, escrow)`'s result (`docs/PRODUCTION.md`).
`tech_frame`/`tech_cat_frame` are **not** written here. ~~Nor anywhere: the
only writer in `Leader`, `LeaderData` and `Leaders` is `Leader::init`'s
zero.~~ **Corrected by the second reading (B3-b): `Build::queue_up@00620f40`
writes both, to `game->frame`.** The first reading's grep was scoped to three
classes and the writer is in a fourth — the cities audit's "grep the writers"
lesson, which this audit hit three times over (B3-b, B5-f, B7-b).
The *conclusion* survives the correction, for a different reason than the one
given: `f1 = min(4, X + 12)` and `f2 = min(3, X + 16)` saturate for every
non-negative `X`, so research's recency factors (§2.14) are the constants
`f1 = 4`, `f2 = 3` whether the stamps are live or not. Dead terms, kept in the
formula for fidelity — but not unwritten state.

**`produce_city(t, wx, wy, escrow)`**: the site's region `r`; the best
citizen of mine — on the map, **the exact base-type pair `0x32/0x33`**
(not the lineage), action none,
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
`gatherers--`. Return 0. **Which branch it takes keys on the *last
examined* citizen's action**, not the chosen one's — the action is stashed
at `−0xc(%ebp)` per candidate, before the region and distance tests, so a
later rejected candidate overwrites it. Reproduced as the original has it.

Run7's city site `2007` appears in gamelog `FRAME 777`, which is the end
of **game frame 776** (`docs/INPUT.md` §3: the dump numbers from 1, the
game from 0). `who = 1`'s sweep is at 775 (`175 + 200·3`), so 776 is
**step 1 — the script**: `city_placement`'s `place_city_with_cost`
(§2.12), and `found_cities` bought on the spot. The C++ `found_cities` step proper
would have been 778. By the same convention the farm in `FRAME 2` is game
frame 1, step 1 again: the script's `place_farm`.

### 2.18 Units — `create_units@006c40a0`

**Provenance, and it is weaker than the sections above.** §2 was written
before this function was read; what follows is the *implementation*
reading (one worker, 2026-08-24, line by line against the decompile and
the listing where the decompile printed a local that could not be right),
promoted here from what used to be §14.5 so that the blind second reading
has a claim to compare against. The full derivation is
`crates/sim/src/ai_units.rs`'s docstrings; this is its load-bearing half.

1. **The army-size ladder differs by branch**: rung 2 is `×3/2` in the air
   branch (line 442) and `×2` in the land branch (1430).
2. **The military gate is `role & 0x10000` for every domain** (292), so sea
   and air military take the wonder/domain mod **and the draw** — more sync
   draws than the first reader's table has.
3. `city_flags & 2` is `no_heal` in the simulation's naming, not `alarm`.
4. `check_income`'s parameter order is `(t, mult, o, escrow, city, num,
   out)`. The value arithmetic **wraps** (32-bit `imul`), and the tail's
   `< 0 → 9,999,999` is its own guard — observed reaching that clamp in
   run19 (§15.6).
5. ~~**What is not on `UnitType`** and is therefore seamed (§13):~~
   **All five are on it now, 2026-08-25** (`UnitType::cols`,
   `docs/DATALAYER.md`): `unit_flags` (`+0x2b4`), `unit_flags2` (`+0x2b8`),
   `carry` (`+0x2d4`), `cat` (`+0x14`), and `role` (`+0x2c8`), which
   `determine_roles@0061c320` builds: `0x200` for ids 0..=3; air `0x1000`,
   land `0x40000` (+`0x10` scout, +`0xc` cat 1), sea `0x80000` (+`0x10`
   bark); `0x8000` if `carry`; military `0x10000` when `attack != 0`,
   `cat ∉ {4, 5}` and not `0x200` — sea +`0x2000`, air +`0x4000`, land
   cat 0 +`0x800`, hoplites `0x100000|2`, ranged +`0x400` else +1;
   otherwise `0x100`. One correction from building it: the hoplite `2` and
   the infantry `0x100000` are **exclusive arms**, not a pair, and the whole
   word reproduces for 364 of 364 types.

### 2.19 Buildings — `create_buildings@006c1be0`

Same provenance as §2.18 (was §14.6; `crates/sim/src/ai_build.rs`).

1. **The gather multiplier's cap is a real `min` at `0x100`** (listing
   `0x6c2762–0x6c276b`: `cmp; cmovl; mov` — the running cap ratchets
   *down*). The `×2` terms decide whether it ever *reaches* ×1; the first
   reader's "garbage `extraout_EDX`" is the decompiler's artefact, and the
   register is reloaded.
2. The dock's value **is** multiplied by `world+0x34` (1241, listing
   `0x6c331d`); a dock that passes runs the §3.6–§3.10 tail like anything
   else, and a dock failing `sea_map > 2 || city_num > 1` is dropped
   without reaching `make_me`.
3. `get_queued(t)` recurses only for a **unit** type; for a building it is
   `num_queued[t]` exactly.
4. The `reg_wars` this function reads is the **byte array at `+0x6836`**,
   not `+0x9b4` (`active_wars`). `get_enhancing_good` is an exact-type
   switch, not a lineage test.
5. ~~**`build_flags & 0x8000000` is never set**: no `BUILD_FLAGS` string in
   the shipped data carries a digit and `init_final_flags` sets only
   `FLAT`. So the Temple, Library and Senate arms of the civic block are
   **dead code** and only the Market arm ever runs. Kept behind
   `flags::DEEP_QUEUE` in case a loader ever sets it.~~ **Wrong in its
   conclusion, corrected 2026-08-25** (`docs/DATALAYER.md`, "The derived
   words no column carries"). The premise holds — no shipped string carries
   a digit — and that is exactly *why* every bit above 25 is **derived**.
   `0x8000000` is set by `TechType::set_research@0066cba0` on every building
   in the lineage of a technology's `WHERE`: the Granary, Lumber Mill,
   Smelter, University, Library, Temple, Senate and the whole Tower and Fort
   lines. **The civic block runs.** So do `is_military_trainer`
   (`0x40000000`, set by `UnitType::init`'s tail on the Barracks, Stable,
   Siege Factory, Dock, Airbase and Missile Silo) and `is_training_building`
   (`0x80000000`), both of which read the flag on the **root** of the `FROM`
   chain. The flag is `flags::RESEARCH_HERE` now, and 129 of 129 buildings'
   `build_flags` equal the program's.
6. **The value product overflows in the shipped game.** `9,999,999 × 0x100
   > 2³¹`, so an Aztec's first Barracks (`1000 × 3 × 400 × 100 × 4`) comes
   out *negative* and loses its category slot, and a Market overflows
   inside its own `×10000`. Reproduced with wrapping arithmetic rather than
   corrected — whether it is load-bearing for the shipped AI is still open
   (§13).

### 2.20 Placing one — `produce_building`

Same provenance as §2.18 (was §14.7; `crates/sim/src/ai_place.rs`).

The census adjustments at the tail (lines 1114–1151) are **kept, not
discarded**: `filled += 1`, `space[n − 2] = max(0, · − 1)` for
`n = 2..best_sp`, and then the builder comes off the gatherers
(`gatherers`, `reg_gatherers`, `city.gatherers`) or, failing that, the free
peasants. Run8's frame 2 shows the effect directly — `gatherers 5 → 4` on
the frame the AI places its farm.

#### Its two draw sites, and the three defects they found (2026-08-26)

`produce_building` steps `game_random` at exactly two places, both marked
under the original's own offsets so the harness's fold and the trace's
line up by name (`docs/SYNC.md` §5.1; `sim::ai_place::SITE_SPIRAL`,
`SITE_JITTER`):

| site | where | when |
|---|---|---|
| `+0xc99` (`0x006e2099`) | the spiral's scoring loop | once per **friendless FARM/MINE candidate** that passes every site test — `score = 4000 / max(d, 1) + r % 500` |
| `+0x1805` (`0x006e2c05`) | the placement jitter | once per **unblocked sub-position**, `best = max(r % 100)` |

Both offsets are settled by the listing, not the decompiler's line
numbers: `llvm-objdump` puts `cltd; mov ecx, 0x1f4; idiv` (the `% 500`)
after `0x006e2099` and `cltd; mov ecx, 0x64; idiv` (the `% 100`) after
`0x006e2c05`.

Marking them split run20's frame 1 into two rows, and the rows are three
separate defects.

1. **The jitter walks a 2×2, not a single sub-position.** Both of its
   loops at `006e2a78` are **inclusive** — `while ((int)uVar11 <= (int)uVar19)`
   over `corner.x ..= corner.x + ex` and `while ((int)local_1c <= (int)local_58)`
   over `corner.y ..= corner.y + ey` — so an ordinary building, whose
   `ex == ey == 1`, tries **four** positions and draws once for each that
   `blocked_site` clears. Run20 spends four, the fuzzed map three.
2. **The stride-by-three tested the wrong index.** `local_10 = 3` is set
   when a candidate improves on an existing best beyond ring 3
   (`local_60 == 0 && local_40 != 0 && local_84 == 0 && circle_radius[3] <
   local_2c` — not a gather-scored type, a best already standing, not a
   tower, and the **current** index past `circle_radius[3]`). Comparing
   the loop's *start* index instead — 0, 1 or exactly `circle_radius[3]` —
   never engaged the stride. Worth 105 cells against ~40, and no measured
   number yet.
3. **`WorldData::buildings_allowed` is a *predicate***, not the `0x78`
   field its name suggests
   (`006b2340`: `return (flags & 0x78) == 0`), so a cell carrying `ROCK`,
   `MOUNTAIN`, `FOREST` or the unnamed `0x40` takes no building. Only an
   oil platform (`0x1a6`) skips the test.

With all three, run20's frame 1 is `+0xc99` **39/39**, `+0x1805` **4/4**,
and the farm lands at `(41856, 39552)` — the run's own `BUILDDATA`
`who 1, o 2006`, to the unit.
`diff::tests::run20_s_frame_1_spends_the_farm_s_ambience_pair` and
`the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well` pin both.

**What this leaves open on the fuzzed map**: its spiral is one candidate
*short* (29 against 30) and a `Unit::do_non_flat_gather+0x54b` short too,
so its frame 1 reads 43 against 45. The second test asserts both residues
as they stand.

#### The builder's distance is corner-to-tile, floored twice (2026-09-03)

`006e28b2`–`006e28ec` takes the candidate's **corner tile**
(`local_5c`/`local_70`, set at `006e2656`) and converts the unit's own
`x`/`y` to tiles through `div_3_table` — a floor, **one coordinate at a
time** — before handing both absolute differences to `vector_dist`.
Dividing the world-unit difference once is a different function, and the
two part wherever the floors do. On run71's frame 4176 the AI places farm
`2014` and pulls a citizen onto it: `1/11` (tile 212,99) and `1/19`
(218,133) both score **27** against the corner (216,116) — 17 of distance
and 10 of the timber penalty — so the **earlier unit keeps the tie**
(`jge` at `006e2a57`). Difference-then-divide read 28 against 25 and sent
`1/19`. Great Lakes' word ran 4241 → 4803 and its position parting
4177 → 4827.

`BuildTypeData::get_good@0063bd50` is `[this+4] − 417` into a jump table
and writes only `eax`, so the second call at `006e29a0` — Ghidra's
uninitialised `this_03` — is the **same object**: the penalty ladder
reads one good, not two.

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

## 4. The fork this opens — ~~decided: (a), 2026-08-24~~

**Folded into `docs/DECISIONS.md` entry 20**, which carries the choice (an
interpreter of our own over the install's own `.bhs` files), the two
options it was taken over, and the two conditions set with it.

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
4. ~~**Armies**, as its own document (`docs/ARMY.md`)~~ — **done
   2026-08-25**: `docs/ARMY.md`, `crates/sim/src/army.rs`,
   `docs/audit/2026-08-25-army.md`; the sea half's part stays in
   `docs/TRANSPORT.md` §8. ~~The state machine and
   `find_target`. Its oracle is a longer run with a war in it, which does
   not exist yet. **The sea half of it is read** (2026-08-25,
   `docs/TRANSPORT.md`): `check_transport`, the docks registry,
   `think_civilian_transport`, `do_mustering`'s transporting arm,
   `do_transporting`, `init_navy` / `send_navy`.
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
`combat::Rng::get`'s already. Its open items — ~~`produce_building`'s draw
count~~ (**settled 2026-08-26**: two sites, `+0xc99` per friendless
FARM/MINE spiral candidate and `+0x1805` per unblocked sub-position of an
**inclusive** 2×2 jitter — §2.20), `can_pay_cost`'s two context arguments
— the building reader and `docs/COSTS.md` answer.

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
the world's `CellData`, the driver's calls) and integrated. Those workers
found **thirty-odd** places where the decompile disagreed with §2; they were
held as an errata section until 2026-08-25 and are now folded into §2 itself
(§14 records where each went). What is not yet
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
  (§2.14) and seamed to zero until the loader ports them.
- **`crates/sim/src/ai_units.rs`** — `create_units` (every branch: air,
  missiles, sea, land civilians, scout, land military, the shared tail),
  `upgrade_units`, `produce_unit`, `queued_units`, `check_income`,
  `unit_prod_value`; the type flags the sim does not carry are seams.
- **`crates/sim/src/ai_build.rs`** — `create_buildings` (both passes, every
  family), `produce_upgrade`, `produce_spell`; the gather multiplier's cap
  settled (§2.19).
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
3. ~~**The loader's half of the producers**~~ — **done, 2026-08-25**,
   `docs/DATALAYER.md`, "The derived words no column carries", and
   `crates/sim/src/ai_load.rs`: `UnitType::cols` (`unit_flags`,
   `unit_flags2`, `cat`, `carry`, `role`), the six derived `build_flags`
   bits, and `TypeDef::ai[11]`. Every one of them is checked against the
   program's own loaded values — a `DUMP_ALL` type dump prints `role`,
   both flag words and the eleven `ai[scan]` shorts — and
   `rondata --types <dump>` reports **0 differences on all four**: 364
   roles, 364 `unit_flags2`, 129 `build_flags`, 85 × 11 weights. The two
   findings that changed behaviour beyond the seams: `build_flags &
   0x8000000` is **live**, so `create_buildings`' civic block is not dead
   code (§2.19), and the unit table has a **name group** rule that gives
   sixty-four records another record's columns (`docs/FORMATS.md`).
   Still open from this item: the market (`docs/ECONOMY.md`) and
   `gather_max` for non-flat buildings — `BuildTypeData::calc_gather` is a
   thousand-line terrain scan with its own mining lists and cliff tests, so
   it is a mechanic of its own rather than a seam, and it belongs with
   `docs/ECONOMY.md`'s gathering.
4. ~~**The producers' oracle.** No dump yet shows a non-empty make list: the
   script blocks the C++ steps for the whole of the opening. A run past the
   script's `SCRIPT_DONE` (Classical Age under `defensive`, ~step 29) with
   `LEADERS=9` on a few frames around a sweep is what scores `create_*`,
   `research_techs` and `make_stuff`.~~ — **done, 2026-08-25, run18 (§15)**,
   and the guess was right to the step: `defensive` `case 29`, Classical Age,
   at sim-frame 6376. The window `[6374, 6590)` holds the sweep the script
   dies on and the next one, with both ladders, a non-empty make list,
   `make_stuff`'s expiry rolls against the trace's own seeds, and one
   purchase. Still unscored: `research_techs`' and `found_cities`' outputs
   (`produce_tech` first runs at 8182, outside the window).
5. ~~**Fold §14 into §2**~~ — **done, 2026-08-25**; §14 is now a map of
   where each correction went, and `create_units`/`create_buildings`/
   `produce_building` were promoted to §2.18–§2.20 rather than folded,
   since §2 had never covered them. Next: **the blind second reading** of
   the whole mechanic — thirty corrections from the implementation is
   exactly the kind of first reading the audit rule exists for, and §2 now
   states them so a re-derivation can disagree with something current.
6. The soak's AI leaders (a synthetic script, or the install), and
   `docs/ARMY.md`.

## 13. What is not established

- **The woodcutter camp's slot count** — `BuildData::gather_max`, written
  by `BuildTypeData::max_gatherers@0063c430` → `calc_gather@00639e40`
  (`docs/ECONOMY.md`'s open item): 5 for camp 2001 on run9's map, 0
  (uncapped) in the sim. Pinned as a ceiling in the census test.
  **Not a loader seam** (2026-08-25): `max_gatherers` is one line — 1 for a
  flat building, else `calc_gather`'s second output — and `calc_gather` is a
  thousand-line terrain scan with mining lists and cliff tests. It is a
  mechanic of its own, and it belongs with `docs/ECONOMY.md`'s gathering.
- ~~**The frame-0 sync stream** (§12.1 item 2): every AI draw before the
  first `place_unit` — the personality, the script coin, `compute_sites`'
  stride — is on the wrong stream in the harness.~~ Read out of run11's
  trace and installed (§12.1). **What is on the wrong stream now is
  everything after frame 0's first draw** — the ~120 per-frame draws of
  units, herds, farms and ammo the sim does not model (§12.1 item 2′).
- **`compute_site_stats`' inputs the sim lacks**: ~~`find_tcoord_z`
  (heights, 0)~~ — pinned per tile from a `DUMP_ALL` dump's
  `master_land_heights` (`World::tile_z`; `(int)((h[ty+1][tx] +
  h[ty][tx+1]) × 0.5)`, 0 on ocean), ~~`was_seen` (true — the lobby reveals
  the map)~~ — it does not (`REVEAL_MAP 1`); read whole and modelled from
  the dump's fog grid and the territory arm, §2.13 step 1 and §15.8, with
  `reg_forts` and the leader-flag exits still open — `danger[]` (0), team
  style 2's `target`, the ally-land arm;
  ~~**the `Region.coords` order** — the sampler walks the region's own list
  and the sim walks row-major~~ — the same: run3's `BEGIN REGIONS` prints
  every region's list and region 1's 3,053 coordinates are row-major
  exactly, as `Regions::rebuild_coords` writes them.
- ~~**`unit_flags & 4/8/0x8000`, `unit_flags2 & 0x60`, `carry`, `cat`,
  `role`** — not on `UnitType` (§12.1 item 3); every producer test that
  needs them runs on a hand-built type.~~ **Closed, 2026-08-25**: all five
  are on `UnitType::cols`, derived by `sim::ai_load` and checked against the
  original's own type dump — 364 of 364 `role`s and `unit_flags2` equal
  (`docs/DATALAYER.md`, "The derived words no column carries").
- ~~**`TechType::ai[11]`** — derived at load, seamed to zero: `w = 1`.~~
  **Closed the same day**: `ai_load::compute_ai_values`, and all 85 × 11
  weights equal the program's.
- **The market**: `use_market` computes its need and its gate and neither
  trades nor draws (the one draw sits in the sell branch, and which branch
  is taken is a price question), `market_speculation` clamps and tiers.
- ~~**`build_masks & 1`** (the "sell me" mark) has no writer and no field.~~
  **Closed by the second reading (B7-b): the only writer is
  `Unit::resolve_block@005fccc0`**, and it is not a sell mark — it means "one
  of my units was blocked by this building of mine, and it isn't a gatherer".
  Read-and-clear, one-shot. The orphan check's disband arm is still
  unmodelled. **`city_flags 0x8/0x1000`** — no writer.
  **`CityData.ocean_filled`/`bordering`** — on `CityAi`, never written.
- **Meeting** (§2.3 step 6) is skipped whole; every other active leader
  counts as met, else steps 14–15 would be dead. A second opinion wanted.
- **Step 16, the army seeding**, is skipped (`docs/ARMY.md`).
- **`check_explore`'s extent** — the region grid at `(w/2)·(h/2)`, from the
  two index expressions; never checked against a dump.
- ~~Two census verdicts sit on aliased decompiler locals and want the
  listing: step 13's inner cutoff (`i + 1 < circle_radius[..]`) and step
  15's operand order in the weaker test (§2.3).~~ **Closed by the second
  reading (B5-a), which settled those two in the listing and found two more:
  the dock-footprint scan keeps the *last* water column's region
  (`6ba12b–6ba16a`); pass N's `local_30` is an `int` typed `Region*` and then
  a `CityData*` typed `Region*`, so `(local_30->coast).ptr[3]` is
  `CityData::dock_tile` at `+0x67`; the per-region admission test compares
  `(domain == SEA)` against `(reg < 64)` (`6baea3`); and `WData` read through
  Ghidra's `Region` names maps `climate→val`, `goody_factor→region`,
  `common_factor→flags`.**
- **The original's 32-bit overflow** in the value pipelines (§2.19, an
  Aztec barracks scores negative) is reproduced; whether it is load-bearing
  for the shipped AI needs a `LEADERS=9` dump past the script.
- ~~**`leader_flags & 1`** — taken as "alive" by the sites; bit 2 is alive
  elsewhere.~~ **Closed by the second reading (B4-h): both are right, in
  different places, deliberately.** `LEADER_VALID (1)` in `found_cities`'
  spacing loop, `LEADER_ACTIVE (2)` in `compute_site_stats`' enemy-capital
  loop, `ACTIVE|HUMAN (6)` in `max_human_cities`, `COOP_SOLO (8)` in the coop
  gate, `& 0xc == 4` for the survey skip. A defeated leader stays VALID, so
  city spacing keeps respecting his cities while the capital-distance term
  stops counting him. `best_human_age_stamp` uses a fifth variant,
  `& 7 == 7`, one line from `max_human_age`'s `& 6 == 6` (B3-f).
- ~~**`check_income`'s parameter order** — the stub's names are one off the
  original's `(t, mult, o, escrow, city, num, out)`; both callers pass the
  escrow flag in both slots until it is renamed.~~ **Closed by the second
  reading (B1-b): the callers do not disagree.** The order is
  `(type, weight, obj, no_escrow, city, min_count, out)`; `create_units`
  passes the city's **object number** in `obj` and −1 in `city`,
  `create_buildings` the reverse, `research_techs` −1 in both. Triply settled
  — through `can_pay_cost` → `get_cost`'s use of the two slots, and
  independently by `Leader::can_pay`'s own argument order. The stub's names
  are still one place off and it discards both contexts, so nothing changes
  behaviourally.
- ~~`compute_site_stats`'s per-city distance loop~~ — settled: `return`s
  (§2.13). ~~The gather multiplier's cap~~ — settled: a `min` at ×1
  (§2.19). ~~`build_flags & 0x8000000`~~ — settled negative (§2.19).
  ~~`TechType::compute_ai_values`~~ — read (§2.14). ~~The personality is
  loggable but not at `LEADERS=9`.~~ ~~Which script run7's AI drew.~~ Both
  settled by run8 (§5). ~~The census adjustments at the end of
  `produce_building` are not kept.~~ Kept. ~~City names are synthetic.~~
  Still synthetic; behaviour identical.
- The rest of the earlier list stands, less ~~`WorldData::danger[who]`'s
  writer~~ — **closed by the second reading (B7-a): `GameDaemon::calc_danger`,
  filling from enemy military units**:
  `is_ally(who, who)`; the implicit variable's declared type; the
  placement's missing map layers *other than* `val`/`goods`/`region2`
  (which the map now supplies) — ~~`buildings_allowed`~~ (**closed
  2026-08-26**: it is `(flags & 0x78) == 0` and the dump carries those
  flags — §2.20), the enemy-seen flag,
  `gather_at` amounts, the oil patches; ~~`space_at_corner`'s first-row
  early-out~~ — it is the centre's, not the first row's, and the tables
  are read from the PE (§15.9); `find_build_at_city` with
  `bool_count_inactive`; the raid
  stamp; `get_starting_town_size`'s nomad flag; the tick's origin; the
  `leader_flags` line's position in the dump.

## 14. ~~Corrections from the implementation, by module~~ — folded, 2026-08-25

**Folded into §2 and closed.** This section was the first reading's errata:
thirty-odd places where §2's prose and the two readers' reports were wrong
or incomplete, found by the seven workers who implemented the producers on
2026-08-24 and deliberately left unmerged so the blind second reading could
use them as a checklist. Leaving them there had become the thing standing
in the way of that reading — a second reader re-deriving against a §2 whose
corrections live in an appendix generates thirty false disagreements, and
the adjudicator spends its pass rediscovering what was already written
down.

Where each subsection went:

| was | now |
| --- | --- |
| 14.1 the census | §2.3, steps 10, 11, 12, 13 and 15, and its tail |
| 14.2 the make list | §2.6 (steps 4, 6, 7 and `make_this`), §2.8 (the danger map, which was the same misreading in four places) |
| 14.3 the sites and the city AI | §2.7 (the insertion rule and `rank`), §2.12, §2.13 (steps 1, 3, 6, 8, 12, 13 and the offset tables), §2.17 (`produce_city`) |
| 14.4 research | §2.14 (the knowledge gates, the `k`-goods factor, the government polarity, the Temple window, `cat`, and the eleven weights), §2.17 (`produce_tech`) |
| 14.5 units | **§2.18**, promoted — `create_units` had no §2 section at all |
| 14.6 buildings | **§2.19**, promoted — likewise |
| 14.7 `produce_building` | **§2.20**, promoted — likewise |

The last three are the ones worth flagging to the second reading. They were
never corrections to §2; they were the *only* written record of three
functions §2 never covered, so they are now sections in their own right —
carrying an explicit provenance line saying they rest on one reading, the
implementation's, and that the full derivation is in the module docstrings.
§2.9 lists what is still unread at document level: `upgrade_units`,
`produce_unit`, `produce_upgrade`, `produce_spell`, `check_income`,
`unit_prod_value`, `check_transport`.

Nothing was dropped in the fold, and no claim changed meaning; §15 and
§15.6's behavioural confirmations are cited inline where they bear on a
claim.

## 15. The behavioural run — run18, 2026-08-25

The first dump that has ever shown this mechanic running. Until it, every
claim in §2.4–§2.11 rested on the reading alone: the shipped script blocks
the C++ steps for the whole opening, so no `LEADERS=9` capture had a
non-empty make list in it (§12.1 item 4). Two stages, both on run12–17's
lobby and seed, both driven from `rontrace.cmd` with no keyboard —
`docs/RUNS.md`, "The producers' run", has the recipe and the `ffwd`
finding that made it cheap.

| | run18a | run18b |
| --- | --- | --- |
| frames | 0–24,000 | 0–6,600 |
| per-frame dump | **off** (`LogStartFrame=0 LogEndFrame=0`) | `[6374, 6590)` at `LEADERS=9` |
| trace | `cover=1`, no window | `cover=1 window=6374-6590` |
| blocks / gamelog | 2 (the quit's) / 1.6 MB | 217 / 360 MB |
| game time | **~45 s** | ~9 min |
| what it gives | every producer's first-entry frame | the state, frame by frame |

A run costs `blocks × ~2.4 s` and nothing else: fast-forwarding with the
dump off runs at ~500 sim-frames a second (24,000 frames — 26.7 minutes of
gameplay — in three quarters of a minute), and a `LEADERS=9` block is
~2.4 s and ~1.7 MB. `docs/ORACLE.md` has the measurements.

Nine predictions were written before either log was read — the script's
exit and its frame, the two ladders frame by frame, the cheap tick's
silence, the empty list before step 6, the expiry's draw count, the sweep's
own draws, the difficulty clamp, and `ffwd`'s invisibility to the sim.
**Every one of them held**, and two of them discriminated between this
document's prose and the implementation — in the implementation's favour
both times, which is what the errata section existing at all had implied.

Two were checked by *absence*, which is worth saying because the trace is
what makes absence readable: `make_this` never runs on any of the six
cheap-tick frames (6385, 6415, 6445, 6475, 6505, 6535) though
`plan_strategy` runs on all six — the head is empty or a building, never
the tech the tick requires (§2.2); and run18a's per-frame draw counts for
frames 0–11 are `120, 54, 6, 6, 6, 6, 6, 6, 6, 6, 226, 254`, **identical
to run14's**, so `ffwd` moves no draw and the sim never reads the clock.

### 15.1 The script's end, and the two ladders

`defensive` reaches **`case 29`** and returns `SCRIPT_DONE` on **sim-frame
6376**, the step-1 call of the sweep at 6375 (`(25 + 6375) % 200 == 0`).
Steps 28 and 29 both run in that one call — the `for (i = 0; i < num_loops;
i++)` with `num_loops = 5` — so the block at 6377 shows `script_step 29`,
`prod_script_run 0`, and `num_queued` carrying **`439 TOWER`** (step 28's
`place_building_with_cost`) and **`544 CLASSICAL_AGE`** (step 29's
`research_tech_with_cost`) while `ages_get()` is still **0**: the age is
*bought*, not entered. Not the hang guard (`script_step` moved), not the
attacked-city bail-out.

The `production_step` the dump prints at the end of each sim-frame:

```
6375  6376  6377  6378  6379  6380  6381  6382  6383      the script's last sweep
   1     2     3     4     5     6     7     8     0
6575  6576  6577  6578  6579  6580  6581  6582            every sweep after it
   1     3     4     5     6     7     8     0
```

They differ by one frame, and the reason is §2.4's first line: `if step ==
1 and (prod_script_run == 0 or starting_resources == 8): step = 2`. On the
transition sweep the script is still live, so step 1 *runs it* and
`SCRIPT_DONE`'s fall-through only advances to 2; on every later sweep step 1
is promoted to 2 **and 2 runs in the same call**. Both are pinned
(`ai_drive::tests::run18_s_two_ladders_differ_by_the_script_s_last_call`),
and the trace's first-entry frames agree with the dump's steps:

| function | first frame | step |
| --- | --- | --- |
| `production_ai_setup`, `market_speculation` | 6377 | 2 |
| `found_cities` (as a producer) | 6378 | 3 |
| `research_techs` | 6379 | 4 |
| `upgrade_units` | 6380 | 5 |
| `create_units` | 6381 | 6 |
| `create_buildings` | 6382 | 7 |
| `make_stuff` (as a producer) | 6383 | 8 |
| `produce_unit` | 6582 | 8 |

`found_cities`, `make_stuff`, `use_market` and `MakeList::make_me` had all
run far earlier — **frame 576** — and `make_this`/`produce_city` at 776:
that is the *script* calling `place_city_with_cost`, which reaches
`found_cities` and lets it buy on the spot (§2.17, and `docs/ORACLE.md`
§5's table). So a function's first-entry frame is not the step machine's;
only the step in the dump settles which caller it was.

### 15.2 The make list, observed

Eleven slots, and the dump fills **0, 5 and 8 and nothing else** — which is
`make_me` exactly (§2.11): a top-four insertion at 0–3 over a
one-per-category write at `list[cat]`.

```
6381  create_units       list[0] PEASANTS val 0 cat 5     list[5] PEASANTS val 0
6382  create_buildings   list[0] TEMPLE  val 2499999 cat 8  list[5] PEASANTS  list[8] TEMPLE
6580  create_units       list[0] PEASANTS val 714 cat 5   list[5] PEASANTS val 714
6582  after make_stuff   list[0] TEMPLE                     list[5] PEASANTS val 7
```

(Sim-frames; the dump's `FRAME n` label is one higher. ~~The first
sweep's citizen at `val 714`~~ — **corrected 2026-08-25, §15.7: the first
sweep lists it at `val 0`**, and it lands only because a cleared slot's
`val` is −1; the 714 is the second sweep's.)

Three things fall out, all of them predicted:

- **The head is overwritten, not shifted.** The citizen ranks 0 at 6381;
  the temple outbids it at 6382 and takes slot 0; the citizen's rank-0 copy
  is simply gone, surviving only in its category slot 5. Nothing lands in
  slots 1–3.
- **Slot 5 is the citizen's category and slot 8 an ordinary civic
  building's**, which §2.11 had left for "the unit readers to name".
- **`MakeList::clear()` at step 2 empties all eleven** — the block at 6577
  has no entries at all, one frame after 6576's two — and empties them
  *whole*: the three slots that held a `val` at 6576 read `val −1` at 6577
  (§15.7).

### 15.3 The expiry, settled against the original's own seeds

`make_stuff` step 4 draws once per slot holding the head's type. The trace
records the seed *before* each `Random::get` and the dump records which
slots survived, so the two together decide the arm §2.6's prose had
backwards. Five draws over the two `make_stuff` frames:

| frame | site | slot | type | roll | `% 3` | verdict | dump |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 6383 | `+0x221` | 0 (head) | TEMPLE | 61545 | 0 | clear | cleared |
| 6383 | `+0x221` | 8 | TEMPLE | 25792 | 1 | keep | kept |
| 6582 | `+0x221` | 0 (head) | TEMPLE | 17105 | 2 | keep | kept |
| 6582 | `+0x221` | 8 | TEMPLE | 1032 | 0 | clear | cleared |
| 6582 | `+0x63d` | 5 | PEASANTS | 48595 | 1 | keep | kept |

**Five of five**, with the seeds chaining draw to draw and on into the
frame's next record. An unconditional arm would have emptied four of those
five slots, so the Temple — an ordinary building, neither a unit type nor a
university — is **probabilistic**, exactly as `ai_make.rs::expire_all` has
it and not as §2.6 read (amended in place there). The two sites are new to
the documents: **`make_stuff+0x221` is the head's walk, `+0x63d` the bought
slot's** (§2.6 step 6's "the same 1/3 expiry over duplicates of *its*
type"). Pinned as
`ai_make::tests::run18_s_expiry_rolls_decide_exactly_the_slots_the_dump_kept`,
which fails if the predicate is put back the way the prose had it.

### 15.4 The rest, in one list

- **`val /= 100` on a buy, not a clear** (§2.6, `make_this`): the citizen
  bought out of slot 5 at 6582 reads **`val 7`** in the next block, from
  714, and stays in the list.
- **A bought *slot* is not a bought *head*.** 6582 buys the citizen and
  still disarms to step 0 — `make_stuff`'s return is the *head's* purchase
  alone, so the second pass (steps 9–11) does not run. §2.4 exactly.
- **`production_ai_setup`'s easy-difficulty clamp fires** (§2.5 step 2).
  This lobby is Easiest, so `d = 0` and `m = max(next age's cost, 300) ×
  3/2`: `bucket` goes **261/104/107 → 37/43/77** across 6376→6377, and
  `econ`/`rate`/`worst_good`/`best_good` are written at 6377 for the first
  time in the game (`econ 8/8/4`, `rate 62/62/20`, `worst_good 2`).
- **`site_mark` increments on a `make_stuff` with no upgrade in the pass**
  (§2.6 step 7): 445 → 446 at 6383, 462 → 463 at 6583.
- **The sites are re-scored every sweep** (§2.7): the same seven sites carry
  different values at 6387 and 6576, and one moves — `(54,11)` → `(59,11)`.
- **The census is stable across the window** at `active 27 control 27
  peasants 22 gatherers 19 free_peasants 1 caras 1 merchants 3 scouts 1`,
  two cities, `pop_cap 50`, `territory 488` — the state every producer in
  §2.13–§2.17 reads, now available as a fixture for the second reading.

### 15.5 What it does not establish

- ~~**`research_techs`' and `found_cities`' outputs.** Both ran, neither
  bought inside the window: `produce_tech` first runs at **8182** and
  `found_cities`' own purchases at 576. A window around 8182 scores the
  first, and it is now a fifteen-minute run.~~ — **`research_techs` scored
  the same day by run19 (§15.6)**, which also caught the second pass and
  the head clause. `found_cities`' purchases are still unscored; they sit
  at frame 576, inside the script's era.
- ~~**`create_units`' and `create_buildings`' values.**~~ Both of the two
  reproduce now (§27): the citizen at **714** and the temple at
  **2,499,999**. What does not is `create_buildings`' *other* offers —
  §27's residue.
- **Only two categories were ever exercised.** Slots 4, 6, 7, 9 and 10 stay
  empty for the whole window, so the category map of §2.11 is confirmed at
  two points and inferred everywhere else.
- **One nation, one personality, one difficulty.** Everything above is the
  British `defensive` leader of run8's roll on Easiest.

### 15.6 run19 — the second pass, and the head clause (2026-08-25)

run18's §15.5 left `produce_tech`'s output unscored: it first runs at frame
**8182**, outside that window. run19 is the bracket, and it cost **a minute and a
half of game time** (19 blocks, 32 MB) — one stage, `!ffwd 9` to frame 8100
and the window `[8174, 8192)`, on the same seed. 8182 is `sweep(8175) + 7`, the `make_stuff` frame of that
sweep's ladder, and not one of the cheap tick's frames (8145, 8205), so the
`produce_tech` there is `make_stuff` → `make_this` → `produce_tech`.

By 8182 the leader is a different animal from run18b's: **Classical Age**
(`ages_get() 1`), five combat units, four goods flowing, and a **full** make
list. Everything run18b could not show, this shows.

**The second pass runs.** `make_stuff` at 8182 buys the head, so step 8
returns 1 and the machine stays armed: **`1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0`
over 8175…8186** — steps 9, 10 and 11 observed for the first time. The two
`make_stuff` calls are distinguishable in the trace by their call site,
which is `production_ai`'s own structure showing through: step 8 is
**`production_ai+0x1fa`** (`if make_stuff() != 0: return`) and step 11 is
**`+0x236`** (`make_stuff(); step = 0`).

**Two purchases in one `make_stuff`.** The head (`COINAGE`, a tech, through
`produce_tech`) *and* slot 1 (`SCHOLARS`, through `produce_unit`); both
appear in `num_queued` in the next block and both are demoted
**9,999,999 → 99,999**, which is `val /= 100` twice over on the overflow
guard's own value. Wood 92 → 32, wealth 40 → 10, metal 159 → 19 pays for
them.

**The value guard is reached in play.** `research_techs` lists `COINAGE` at
**`val 9999999`** — §2.19's "the original's value product overflows and the
tail's `< 0 → 9,999,999` catches it", now observed rather than inferred.
`research_techs` itself **draws nothing** (frame 8178 has no `game_random`
record outside the farms and the animation clock).

**The head clause, isolated.** This is the run's best observation, and it
needs both frames to see:

| frame | slot | type | roll | `% 3` | probabilistic would | dump |
| --- | --- | --- | --- | --- | --- | --- |
| 8182 | 0 (head) | COINAGE | 11233 | 1 | keep | kept (demoted) |
| 8182 | 4 | COINAGE | 14808 | 0 | clear | cleared |
| 8182 | 1 | SCHOLARS | 22883 | 2 | keep | **kept** |
| 8185 | 0 (head) | SCHOLARS | 45911 | 2 | keep | **cleared** |

The same type on the same residue, kept at a non-head slot and cleared at
the head. A scholar is `0x34` — a unit type that is not a peasant — so the
head takes the **unconditional** arm and the roll is taken and then
thrown away, while every non-head slot is probabilistic whatever the type.
That is `expire_all`'s `if !head { return false; }` and its
`is_unit && !is_peasant` clause, both confirmed in one pair, and it is the
clause §2.6's prose inverts: read the prose literally and 8185's scholar
survives.

With run18b's five that is **nine expiry observations, nine agreeing**
(`ai_make::tests::run19_s_head_clause_clears_a_scholar_a_non_head_slot_would_keep`).

**The runners-up shift, observed.** run18b never had enough entries to
exercise slots 1–3. Here they fill and move: at 8180 the ranked four are
`COINAGE, EMPIRE, MERCENARIES, PHALANX`; `create_units` at 8181 inserts
`SCHOLARS` and `MERCHANT` and the four become `COINAGE, SCHOLARS, EMPIRE,
MERCHANT` — `MERCENARIES` has fallen off the end, which is §2.11's "shift
down, slot 3 falls off". The categories in use across the window are 4, 7,
8, 9 and 10, against run18b's two.

**What it does not establish.** The dump is one snapshot a frame and a
producer makes many `make_me` calls inside one, so the *net* movement of
slots 1–3 across a producer frame is attributable only in outline — the
insertions above are read off end states, not traced call by call. Slot 5
and slot 6 were never occupied in either run. And `found_cities`' purchases
still sit at frame 576, in the script's era, unscored.

### 15.7 The whole record, diffed — 2026-08-25

Audit B7-f asked for it and the `SITES` widening (audit, "B4-f, confirmed
behaviourally") showed the shape: **every consecutive pair of `LEADERS=9`
blocks in run18b (216 of them) and run19 (18), all eleven slots, all ten
fields**, each step of the ladder replayed with the simulation's own
operation on the previous block's record and compared to the next —
`MakeList::clear` for step 2, `MakeList::make_me` over the offers that
landed for a producer step, `expire` with the trace's seeds and the bought
slots' `val /= 100` for `make_stuff`. Frames that do not change the list
are counted; the frames each class covers are asserted so the test cannot
pass by calling everything unchanged
(`rondata::diff::tests::run18b_and_run19_s_make_list_windows_replay_slot_
for_slot`; run9's frame 1 is pinned to the init record in the census test
beside it). Everything reproduces. What the widening found, in order of
consequence:

- **`MakeList::clear` resets the whole record, not `t`.** The sim cleared
  `t` alone and its empty slot carried `val 0, num 1`; the original's is
  `val −1, num 1` (§2.6), and the difference is behavioural — the first
  sweep's citizen is offered at **`val 0`** (dump-frame 6382) and lands at
  the head and slot 5 only because `−1 < 0`. Written from
  `MakeList::clear@006c9db0`, which stores all ten words; confirmed by the
  guard failing on the old `clear` at dump-frame 6577 with the three
  stale `val`s named. The expiry is the one that writes `t` alone: run19's
  8183 has slot 4 at `t −1 val 9999999`, and the 8184→8185 shift moves that
  ghost record intact into slot 2, where the dump shows it.
- **`research_techs` offers the cat-8 line first.** Dump-frame 8179's
  record (`COINAGE` 559 at 0 and 4, `EMPIRE` 566 at 1 and 9, `MERCENARIES`
  573 at 2 and 10, `552` at 8 and *not* in the ranked four) is reproduced by
  exactly two of the 24 orders — `552` first, then `559`, then `566`/`573`
  either way — because `552`'s rank-0 copy has to be overwritten by
  `COINAGE` for slot 3 to stay empty. §2.13's loop order is what it says.
- **Step 9's `create_units` offered a merchant the end state hides.**
  8183→8184 cannot be reproduced from its visible new entry (the scholar at
  `num 5`): slot 3's merchant is cleared with its `val` standing, which is
  `make_me`'s duplicate-clear, so a merchant offer came *first* and took
  the head, and the scholar then overwrote it. The replay uses the same
  producer's merchant record from 8181 and finds that order alone.
- **The scholar batch is `num 5`** at 8184 against `num 1` on the first
  pass, which is `create_units`' arithmetic (§2.13) and not yet checked.
- **The temple's second-pass value is `391136`** (8182, over `552`'s
  `63000` at slot 8) — a `create_buildings` output, likewise unchecked.

Still not established here: every *value* a producer computes (§15.5's
second item stands), and the purchases themselves — the replay takes the
bought slots from the dump and checks only what follows from them.

### 15.8 run20 and run21 — the islands map, and the coastal ring's guard (2026-08-25)

Audit B4-k's guard needed a capture whose `compute_site_stats` reaches
step 5, and none on disk did: every run through run19 was on the
profile's Great Lakes (`MAP_STYLE 14`, `sea_map 1`), whatever `check.ini`
said. **Run20** is the East Indies lobby (`MAP_STYLE 18`), picked in the
lobby's combo, under `DUMP_ALL` with `InitialDump=1`, `[Start Game]
WORLD=6`, the checksum trace, a `[0, 4)` window and `!quit` at 4 — a
self-sufficient capture (its own setup trace, heights, herds and frame
words; 278 MB, seven minutes) that the harness reads without a sibling
(`rondata::diff::tests::run20_s_islands_sites_walk_the_coastal_ring_from_
the_original_cell`). **Run21** is the same lobby with the dump off and
`!ffwd 30` to frame 24,000 (`docs/RUNS.md`, "run20 and run21").

**The guard, and what it took to make it pass.** The assertion is the
ten-record `SITES` diff of §15.7's kind on run20's frame-1 leader record.
It failed four times before it passed, and each failure was a correction
to `crates/sim` that no Great Lakes capture could have found:

1. **`world+0x34` is not a landmass count.** The harness counted land
   regions (13, with the artefact region 0) where the original said
   `sea_map 4`. Read whole: `Map::init_map_data` reads it as the map
   style's `<SEA_MAP value="n"/>` (`mapstyles/*.xml`: 0 the land maps, 1
   Great Lakes / Mediterranean / Outback, 2 Warring States, 3 the two-shore
   and island files, 4 Colonial Powers; −1 when absent), `Map::make`
   copies it to the world when it is not −1, and the conquest maker's
   `MapConquest::check_sea_map` computes 0 / 2 / 3 (no sea / every start
   on one landmass / starts apart). **It is the style's sea class**, and
   every `world+0x34` predicate in this document — §2.13 step 5's `> 2`,
   §2.10's `< 4`, §2.14's cubed term, §2.19's dock gate, §6's `< 3` rush
   arm, §2.3 step 15's strategy bits — reads that class. `World::sea_map`
   now carries the dump's value; `landmasses()` is gone. ~~**Not
   established:** why the world reports 4 on East Indies when
   `eastindies.xml` says 3.~~ **Settled the same day, from
   `World::analyze_map@006b58b0`** (entered in setup, run21's trace),
   which the first writer search missed because it stores through a
   different pointer name. Its region walk: a land region where any
   active leader has a presence is a *player region* (`player_reg`,
   `world+0x50`; `Region.flags |= 4`; its `size` into a running sum
   *P*); any other region of at least `max(1, size × 60 / dim²)` cells —
   60 on a 60×60 map — is a *resource region* (`resource_reg`,
   `world+0x54`; `flags |= 8`; `size` into *R*). Then, at the tail: a
   `sea_map` outside `0..4` (the file's −1) is **computed** — `player_reg
   > 1` → `3 + (2P ≤ R)`; else `resource_reg != 0` → 2; else `land_size <
   4·size/5` → 1, else 0 — and **a file value of 3 is promoted to 4 by the
   same `player_reg > 1 && 2P ≤ R` test.** Run20 to the cell: the two
   player islands hold 269 + 260 = 529 cells, 2P = 1058; the eight islands
   of ≥ 60 cells (`resource_reg 8`, as the dump prints) hold 1097 ≥ 1058 →
   4. So class 4 reads "starts apart, and the free islands hold at least
   twice the players' land", which is why §2.10's `< 4` gate sends a
   two-or-three-city AI off its home island. Every class is therefore
   either the file's (0, 1, 2 and a 3 that fails the test) or this
   function's; the harness still takes the dump's value, and a sim
   without a dump would run `analyze_map`'s rule over its own regions.
2. **`land_key[]` is a static** — `BASELAND, SANDY, OCEAN, NONE`
   (`dynamic_initializer_for_'land_key'@004058a0`) — and the harness had
   numbered the dump's names by first appearance, which agreed only
   because run9's first cell is BASELAND. On the islands map the first
   cell is OCEAN, every land cell was "water" to the sweep's `land == 1 ||
   2`, and the home region's `reg_land` came out 0 against 45. Found by
   widening the census check to **every region** (run9's compares the home
   region alone, which on one landmass is every land region); the
   original's own `CITY` record at `DUMP_ALL` carries the step-13 picture
   (`ocean 11, land 96, filled 51, dock_tile 1, space 58/58/45` for the AI's
   city) and is the direct oracle.
3. **`WorldData::is_ocean@006b4830` is by cell kind**, `land == 1 || 2`
   and not `flags & 0x100`, not by region terrain — an island's beaches are
   SANDY cells inside a land region. (This one changed no site on run20 by
   itself; it was landed from the listing.)
4. **`was_seen` is not a seam**, §2.13 step 1 above. `REVEAL_MAP 1` on
   every run; the fog grid `seen2` is in the WORLD dump (`seen[scan]`,
   `seen2[scan]`, `seen3[scan]` triplets, 14,400 each), is loaded
   (`World::seen2`) and, since 2026-08-27, **grows as units move**
   (`docs/VISION.md`) rather than staying the dump's frame-0 snapshot —
   so a census taken late in a game reads a map the player has explored.
   The AI's ten sites are unseen on it — they are
   scored through the **territory arm**, an ally's (one's own) cell in a
   region with `reg_cities` or `reg_forts`. With the fog alone the harness
   scored nothing; with the arm, all ten records match, `val` included.
   `reg_forts` and the three leader exits are not modelled and are named
   in `ai_sites.rs`.

**The guard has teeth.** With the ring re-centred on the slid cell — the
reading both the document and the implementation had before B4-k — one
site's ×30 verdict changes, the ranking shifts, and the record moves by a
whole slot; the test names all nine. Restored, it passes; run9's three
tests pass unchanged with the new table and the fog loaded.

**The blind list: 438 cited, 92 → 87 never run** over ten traces. Retired:
`Unit::do_strafe`, `Type::unpay_cost`, `UnitType::log_data`,
`TechType::log_data`, `GameLog::dump_all`. The sea half is a different
story: run21 **did** enter it — `Leader::check_transport@006bc5f0` at
frame 201, `Unit::think_civilian_transport` 275, `Dock::init` and
`Docks::init_dock` 3579, `UnitData::can_ever_transport` 3579,
`LeaderData::get_ships_speed_upgrade` 4376, `Army::do_transporting@006f4690`
14586 — and the list never showed them because §9's table cites that family
by name without an address. **A citation without `@address` is invisible to
the coverage report**; the family is now on the record here by address, and
~~the reading of it (§9) is the next one the queue names~~ — read the same
day, `docs/TRANSPORT.md`, with run22 as its capture.

**Also observed:** the lobby's `Reveal Map`, `Map Style`, `Resources` and
`PLAYERn_TRIBE` are the profile's, not `check.ini`'s, on every run — and
Save to Profile does not survive a killed process (`docs/ORACLE.md`).

### 15.9 `space_at_corner`'s walk order — run20's `space[0..1]` (2026-08-25)

The one field of run20's `CITY` record the widening (§15.7, `docs/
TRANSPORT.md`'s session) left unmatched: `space[0]` and `space[1]` ours 48,
theirs 58, `space[2]` 45 both — ten cells the original scores **3** and the
harness scored **0**. The queue's diagnosis was `check_building_wcoord`'s
fifth argument to `space_at_corner`; it was wrong, and the function's own
body says so: `space_at_corner@006b27f0(tx, ty, who, param_4, need_city)`
never reads `param_4`. What differs is inside.

**The walk.** The sixteen tiles under a corner are not visited row by row
but through two tables, `grid_index_x`/`grid_index_y` (`.rdata`, VA
`0xADECF0`/`0xADED30`, read from the PE by the S_LDATA32 recipe):

```
x: 1 2 1 2   0 1 2 3   0 1 2 3   0 3 0 3
y: 1 1 2 2   0 0 0 0   3 3 3 3   1 1 2 2
```

— the **centre 2×2 first**, then the top row, the bottom row, then the two
side columns. The early-out `if (i < 0x10) return 0` on a blocked tile is
therefore on the centre four, and the 3×3 test that follows reads
`grid_threes` (`0xADECA0`, four rows of five): for each 3×3 of the 4×4, the
five of its tiles outside the centre — every 3×3 of a 4×4 contains all four
centre tiles, so with those known free, five checks are the whole test. The
rows decode to the corners `(1,1)`, `(0,1)`, `(0,0)`, `(1,0)`.

**The classes**, from the body: none blocked → 4; fewer than eight blocked
and some `grid_threes` row all free → 3; otherwise **2**. There is no path
to 0 once the centre is free — `if (blocked < 8) { …; return 2; } return
2;`. The harness returned 0 at eight or more blocked, and 0 on any blocked
tile of the *top row*; a cell whose corner tile is under a neighbour's
footprint but whose centre and one 3×3 are clear is exactly the cell the
original counts at 3 and the harness dropped to 0. Ten of them on run20's
island, and `space[0] == space[1]` on both sides because no cell scores
exactly 2 there.

**`check_building_wcoord@006b26e0`'s own gate**, also missing: it returns 0
without looking at any corner when the cell is another leader's (`who ≥ 0
&& who ≠ me`) **or** occupied (`flags & 0x70`) **or** `WData.blocked ==
0x10` (`+0x11`, `uchar`, the type record). The census reaches it only past
its own `flags & 0x70` test, so the first two are redundant there; the
placement spiral (§2.20) reaches it directly, and there they are new.

**Landed**: `crates/sim/src/ai_place.rs` — `GRID_ORDER`, `GRID_THREES`, the
rewritten `space_at_corner`, the three-way gate; three unit tests, one of
which fails on the old code at `space(&[(0,0)]) == 3` and at every-non-
centre-tile-blocked `== 2`; and the run20 guard's pin retired — the record
is now compared whole, `space[0..3]` included, and the pin flipped
(`ours 58 (was 48)`) on the first run after the walk order landed, before
the pin was removed. Confidence: high on the walk and the classes (the
tables are the PE's bytes, the body is short, the guard agrees); the
`blocked == 0x10` gate is from the listing alone.

**Not established**: what writes `WData.blocked = 0x10` (no writer greps by
name or by `+0x11` in the export; `gaia.rs` reads the same byte as `< 8`);
whether the return-2 branch at ≥ 8 blocked is ever reached on a real map —
run20 has no cell at exactly 2 on either side, so that clause is the
listing's, not a run's. The check that would settle both: a `DUMP_ALL`
window on a crowded city whose `CITY` record has `space[0] > space[1]`.

## 16. Second reading — landed, 2026-08-25

Seven blind readers, one per sub-area, all on Opus 5; adjudicated in the main
thread against the decompile and the listing. The full record, with every
verdict and its citation, is `docs/audit/2026-08-25-ai.md`; the reports are at
`~/ghidra-projects/reading/ai-2026-08-25/`. 590 numbered claims, 42 rows
doubly confirmed, 31 corrections or additions, one verdict that went against
**both** readings, one refutation of a reader's own flagged guess.

**Read the audit's first section before trusting a "doubly confirmed" row.**
The blind protocol leaked: a subagent inherits this repository's `CLAUDE.md`,
which narrates this mechanic in its working-agreement section — naming
`set_research`'s `0x8000000` mark, `research_techs`' 9,999,999 guard, the step
ladders, the make list's occupied slots and `expire_all`'s `% 3` residue. Two
rows are marked leak-affected and not counted. The fix is written down there
and it is a change to where this project keeps its handoff prose, not to the
brief.

**What changed in the simulation — two things, both landed with this section:**

- `compute_site_stats`' **coastal ring is centred on the original cell**, not
  the slid one (§2.13 step 3's correction). Neither reading had this;
  `ai_sites.rs` was wrong because it had followed §2.13's coordinate
  bookkeeping instead of re-deriving it. A guard is still owed — nothing in
  the suite reaches that branch.
- `research_techs` reads the **previous** age's stamp on difficulty 2
  (§2.14). Guarded by `difficulty_two_waits_on_the_previous_ages_stamp`,
  which was made to fail against the old form first.

**What the readers confirmed the simulation already had right**, in each case
where a document sentence would have led it astray: the unsigned head gate
(§2.2 — `ai_drive.rs` asks `kind(t).is_tech()` rather than transcribing the
arithmetic), `target_adj` starting at 1, `pers.arms`' two-stage roll, the
`% 0x3f` naval-array aliasing, the `>=` tie on the placement sub-position, and
all four building/placement draw moduli. That pattern is the section's real
lesson: **the places the implementation was right are the places it was
written from understanding, and the one place it was wrong is the place it
transcribed the document.**

**Five open questions closed** (struck in §13): `check_income`'s callers,
the `leader_flags` bit-1-versus-2 question, the census's aliased locals,
`WorldData::danger`'s writer, and `build_masks & 1`'s writer — the last of
which also turned out not to mean what §2.8 guessed.

**New facts of consequence, all cited in the audit and not yet everywhere in
§2's prose:** `unit_prod_value` is **negated on difficulty 0 and 1**, so the
easiest AI deliberately picks poor counters (B1-a); `TypeData::can_pay_cost`
reduces with **`max`**, not `min`, across resources, so it can report three
affordable when one is (B2-a, and B7 found it independently); `*out_dist` is
clobbered by the enemy-capital loop, so `Site::dist` carries the last enemy
capital's quotient (B4-f); `pers.raid` has a second writer in `process_taunt`
that sets values outside the roll's range, which moves the government choice
(B6-d); leaders are initialised in `game->start_list` order, not `who` order,
which will matter the first time a two-AI RNG stream is reproduced without a
trace (B6-c); and the whole make list is dumped at `LEADERS=9` under
`t val escrow city up o num cat wx wy`, which makes every §2.6/§2.11 claim
falsifiable from one capture (B7-f).

**Still deliberately unread**, and the audit says so rather than leaving it as
silence: `Leader::diplomacy`, the `Army`/`Armies` family, and the BHS language
and its 55 host functions — the last because they already have two dedicated
reports and run7's opening reproduces frame for frame, so the reading is not
their only evidence.


## 17. `static` is one variable on the `Script`, not a frame slot (2026-08-30)

The BHS language and its host functions were left deliberately unread by the
second reading (§16's last paragraph), on the ground that run7's opening
reproduces frame for frame. It does; the *second* call does not, and this is
what was wrong.

**Where a variable lives is two bits of the operand.**
`VirtualMachine::get_value@004d1010` and `set_value@009e07b0` both branch on
them:

| operand bit | storage |
| --- | --- |
| `0x20000000` | `ScriptFile::const_pool` — a literal |
| `0x40000000` | `Script::static_vars`, indexed by the operand less the bit |
| neither | `VirtualMachine::vars`, the call's own frame |

So a `static` never touches the frame. It is one `ScriptType *` on the
`Script` object, shared by every call and every caller, live from a call's
first instruction to its last, and the compiler keeps the two namespaces
apart — `Script` carries `static_var_names` (`+0x94`) beside `var_names`
(`+0x7c`).

Two consequences the shipped scripts depend on. A read **above** the
declaration sees the last call's value, not a zero; and a recursive callee's
write is visible to its caller on return, because both index the same array.

**What this crate had.** A `static` was given a frame slot, seeded at its
declaration statement and mirrored back into the store after every
expression — and the mirror ran over *every* static of the function,
including those whose declaration the call had not yet reached. Any script
with an expression statement above its `static` block therefore wrote zeros
over all of them, on its **second** call and every call after, never the
first. `economic.bhs` has three such statements and eight statics per
leader; the AI's `needed_citizens` was zero from its second call on, the
opening's every-call `train_unit_with_need(who, needed_citizens, "Citizen")`
found `pop < 0` false at once, and **the AI trained no citizen for the rest
of the game**. `static int needed_techs = get_techs_per_age(who)` stayed zero
for the same reason with the initialiser never re-running: "once ever" is
keyed on the store holding nothing, and the store held a zero.

**What it is now.** `Run::seed_statics` fills the frame's static slots from
the store on entry and again after every script-to-script call; the
mirror-back is unchanged in effect and walks a precomputed
`(static index, slot)` list. On the shipped scripts, seeding at entry and
mirroring only past-declaration slots are indistinguishable — every read is
below the declarations — so the model taken is the VM's own.

**Guard**: `bhs.rs`'s
`a_statement_above_the_declarations_does_not_wipe_the_statics`, written
against the old seeding first: the second call returned 1 where it must
return 2, and only the second.

**Diff-backed.** East Indies' word and sequence 274 → **413**; Great Lakes'
weak totals 943/830 → **943/851**; run10's roster the original's both ways
for the first time (268 missing + 0 extra → **0 + 0**). The story and every
number are in `docs/JOURNAL.md`.

## 18. The site list, its territory, and the slide's reach (2026-08-31)

The two census windows (`run40` `[560, 600)`, `run41` `[770, 800)`, §2.13's
own oracle at last) print `Leader::sites` **whole** — ten
`{wx, wy, val, reg, dist, rank}` a leader a frame — and
`LeaderData::territory` beside it. Nine tenths of that record had gone
uncompared for a month; `diff::tests::run40_and_run41_s_sites_and_territory_
are_the_original_s` compares all 7,000 fields of it and the territory that
feeds them.

**What it found.** §2.13 step 1's second early return is
`WorldData::was_seen@006b53f0`, and its **first** arm is territorial: a cell
owned by an ally — `is_ally` is reflexive, so one's own land — is seen
wherever that owner's `reg_cities[region]` or `reg_forts[region]` is
non-zero. On this map the AI's fog plane is empty over the whole southern
approach, so *the site scorer sees exactly the AI's own borders*. Getting
the borders one Civic step too small therefore blinds the scorer, and this
crate's borders were: `World::compute_reg_territory` rebuilds its per-player
bonus table on every pass and `crates/sim` cached it at `add_player`
(`docs/ATTRITION.md`, "Territory"). The AI held **261 cells against 290**,
the human's 266 exact.

The twenty-nine dark cells are step 3's, not step 1's: an unseen cell is
neither counted for `danger`/`water` nor eligible for the **slide**. On
run10's frame 575 the AI samples cell `(48, 29)`; the original slides it to
`(47, 28)` and scores 6,181, and this crate could not see `(47, 28)` at all
— it slid to `(49, 27)` and scored 1,159, and its second city went up at
cell `(55, 21)` instead of the original's `(54, 32)`. With the borders live,
the winning site on frames 576–799 is `(54, 32) / 6079`, the original's, and
the `BUILDORDER` the AI hands `1/1` on 777 carries the original's own
`(41448, 23928)`.

**The residue, and it is one slot.** Where the original's 5×5 leaves
`(48, 29)` for `(47, 28)`, this crate keeps `(48, 29)`: `q = 19` at offset 0
against 17 at offset 1, and offset 0 is the centre, so the original's
`blocked_town` must refuse the centre where `site_clear` allows it.
`BuildTypeData::blocked_site@00636a50` counts the footprint **tiles** whose
fog half-cell (`t >> 1`) the placer has never seen and returns `0x24` when
more than half are dark, with only a Dock (`is(0x1b0)`) exempt;
`blocked_tcoord` here grants visibility everywhere (`docs/CITIES.md` §11).
Landing that rule alone moves none of the numbers — the AI's own territory
answers "seen" through the same territorial arm — so it is **booked, not
guessed at**. The one wrong slot drags the `rank` of the four it outscores
with it, which is the rest of the 580-of-4,000 and 520-of-3,000 residue the
test pins as ceilings.

**Coverage.** Diff-backed: the territory count, both players, every frame of
both windows; the ten sites' `wx`, `wy`, `val` and `dist` on every slot but
the one above. Reading-only: `was_seen`'s `reg_forts` arm (`+0x12de`), which
`was_seen_fog` does not implement and no capture reaches — no player in the
corpus owns a fort.

## 19. What a camp's site is worth — `produce_building`'s gather score (2026-09-01)

`Leader::produce_building` scores a **woodcutter's camp** site by nothing
but what the site would gather, and the number is not the trees around it.
It is `blocked_site`'s out-parameter — `docs/CITIES.md` §2.6.7, filled from
`calc_gather`'s count, the same one `max_gatherers` reads — and the call
that fills it is the very site test the spiral already runs:

```
local_34 = 0
r = blocked_site(type, cand.x, cand.y, who, -1, &local_34)   # line 412
if r != 0: continue
…
else if type == 0x1a2:                                       # lines 658-663
    score = score · local_34³
    if not (local_34 > 2 or frame == 0): continue
    score = (score + plenty) · w1
```

and the camp's jitter — the `move_x`/`move_y` compass ring, not the 2×2 —
picks its sub-position the same way, on a strict improvement:

```
best = -1
for k in 0 .. circle_radius[ex]:                             # lines 981-999
    local_34 = 0
    c = corner + (move_x[k], move_y[k])
    if blocked_site(type, c.x, c.y, who, -1, &local_34) == 0 and best < local_34:
        best = local_34; cand = c
if best < 0: return 1
```

**This crate had a one-tile ring of forest tiles there instead**, from the
first reading of §2.20 — plausible from the name `count_trees_adjacent`
sitting two branches above in `blocked_location`, and wrong. A camp stands
on clear ground *next to* a forest whose gather radius is eight tiles, so a
ring of width one is empty at every site a camp can actually occupy: the
score went to zero, the `> 2` gate refused every candidate, and **the AI
placed no woodcutter's camp at all after frame 0**. Frame 0 hid it, because
`frame == 0` skips the gate and a uniformly-zero score accepts the spiral's
last candidate rather than its best.

**The oracle is run56's frame 2176** (`docs/RUNS.md`, "run56"), where the
original places player 1's second camp — `o 2009`, tile `(198, 190)`, 48
tiles, `4 × 48 = 192` shuffle draws. With the count in place this crate's
AI reaches that frame at script step 13 (`place_woodcutter` in
`aibestbuildlibrary.bhs`, the `num_cities > 1` arm, `place_building_with_cost`
at City 2) and places it at the original's own frame, tile and object number,
spending the original's own 192 draws. East Indies' long word moved **2176 →
2665** on it.

**Coverage.** Diff-backed: the site, the frame and the object number, and
every gather field of all 3,001 frames of run56 —
`diff::tests::run56_s_mining_lists_reach_past_the_word` has one camp the
game builds and no divergence before the quit's own frame. Reading-only:
the `frame == 0` arm of the gate (no capture simulates `Setup`), and the
`0x1a5`/`0x1a6` oil arm of §2.6.7 that sits beside it.

## 20. Where the spiral's stride is spent, and the dock it moves (2026-09-01)

`produce_building`'s index steps at the **bottom** of the iteration, by the
stride as it stands *then*:

```
local_2c = start
do {
    … body; may set local_10 = 3 …
    local_2c = local_2c + iVar13          # 006e25bb, iVar13 = local_10
} while (local_2c < circle_radius[local_c])
```

This crate stepped it at the **top**, by the stride as it stood before the
body ran (`let idx = i; i += step;`), which spends the old stride one more
time: the first strided hop starts one cell late and the whole tail of the
spiral is offset by one. §2.20's third defect had put the stride's
*condition* right and left its *timing* wrong, and the note there — "fixing
it moved no measured number on any capture" — was the tell, because the
condition alone is unobservable on a call that finds its site inside ring 2.

**The oracle is the fuzzed map's frame 1**, and it is one this crate already
had: `the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well` asserted
`Leader::produce_building+0xc99` at **29 against the original's 30** for five
days as its own open residue. With the step at the bottom it is **30 against
30**, and the frame goes 43 → 44 of 45. Nothing else in either suite moves;
run20's frame 1, whose farm is found inside ring 2, is untouched by
construction.

**What it does not close, and what that names.** East Indies' word still
parts at 3021, and the reason is now a record rather than a guess.
`run56_s_buildings_stand_where_the_original_s_do` compares every linked
building's `x_internal` and `y_internal` on every frame — 92,626 fields —
and the residue is **one field of one building**: player 1's Dock `o 2010`,
laid on frame 2977 at cell x 57 in both, at cell y **52** here against the
original's **54**. Its citizen then walks the original's own frames toward a
different point, and 44 frames later the original's step is blocked where
this one's is not: that blocked stand is the whole of the 3021 divergence
(`Guy::set_anim+0x97a < Unit::move_step+0x823`, `docs/COLLISION.md` §5).

The two candidates the spiral must part on are `(57, 52)` (index 141) and
`(57, 54)` (index 143), and this crate's strided walk 135 → 138 → 141 → 144
never visits 143. Three things could put the original on it and none is
settled: the **dock's own sub-position slide** — `blocked_site` refusing a
dock's exact cell sends `produce_building` into the nested
`local_88/2 × local_7c/2` search in its `is(0x1b0)` arm, which this crate
does not run at all, and two of the cells in this walk (`(57, 49)` and `(57, 53)`) are
refused here; a different **accepted set** earlier in the ring, which would
engage the stride at an index congruent to 143 rather than to 141; or a
`local_40` that is not the running best at the point the stride tests it.
No draw is spent anywhere on a dock's spiral or its slide — the jitter is
fenced behind `ident != Dock` — so the *only* oracle for any of this is the
dumped position, which is now a test.

**Coverage.** Diff-backed: the step's position, by the fuzzed map's spiral
count; every building's position on every frame of run56, with the one
field above as the stated residue. ~~Reading-only: the dock slide in
`produce_building`'s `is(0x1b0)` arm, unimplemented and unexercised.~~ —
built and diff-backed, §21.

## 21. The dock's slide, and the cell it buys three candidates earlier (2026-09-01)

§20 left three candidates for the two cells the spiral must part on and
named the **dock's own sub-position slide** as the strongest. It is the
one, and not for the reason the section supposed.

**The arm.** `blocked_site` refusing the centred position is not the end of
a candidate for a dock. The `else` at `006e2725` asks `is(0x1b0)` — the
lineage test, so a Shipyard or a Port answers it too — and on a yes walks a
block of whole tiles around the refused point, taking the **first** that
clears:

```
hx = local_88 / 2;  hy = local_7c / 2;             # 2 and 2 for a dock
for (dx = -hx; dx <= hx; dx++)
    for (dy = -hx; dy <= hy; dy++)                 # note: -hx, not -hy
        if (|dx| + |dy| <= local_5c)               # 8 for a dock: never binds
            if (blocked_site(bt, base + dx*0xc0, base + dy*0xc0, who, -1, NULL) == 0)
                goto accept;
```

Three details the listing settles and the shape does not:

- **The base is the *unpadded* centre.** The arm rebuilds it as
  `cell·0x300 + size·0x60` rather than reusing the padded point the
  refused call was given, so it drops the `((4 − size) · 0xc0) / 2`
  centring that `LAB_006e1c9b` adds. A dock is 4×4 and the two agree; a
  smaller type in this lineage would not. (For any size ≤ 4 the padded
  centre is exactly the cell centre, `cell·0x300 + 0x180`, because
  `size·0x60 + (4 − size)·0x60` is constant — which is why the difference
  is invisible on the only type that runs this.)
- **`dy` starts at `−(local_88 / 2)`**, the *x* half: `local_58` is loaded
  from the `dx` initialiser once and never reloaded. Invisible while
  `w == h`, and written as read.
- **The out-parameter is null**, so `local_34` — what the site would
  gather — keeps whatever the refused call left it. Inert: no dock is
  gather-scored.

**What it does, and it is not what §20 guessed.** The slide is *not* what
places the dock. Run56's dumped `o 2010` sits at `(44160, 41856)`, which is
the plain centre of cell `(57, 54)` to the unit, and the slide's whole
reach is ±2 tiles — half a cell — so no slid position from any neighbouring
cell can reach it. What the slide changes is **which candidates the spiral
accepts**, and therefore the phase of the stride of three.

The walk on frame 2976, anchor `(51, 52)`, `start 0`, `end 145`
(`rings 6`), every score `588` so ties always replace:

| | accepted at | first stride | thereafter | lands on |
|---|---|---|---|---|
| without the slide | 129 | 135 | 138, 141, 144 | 141 = `(57, 52)` |
| with it | **117**, 119 | **119** | 122 … 140, **143** | 143 = `(57, 54)` |

Indices 117 `(47, 57)`, 119 `(48, 58)` and 131 `(54, 58)` are the walk's
`blocked_site` refusals, and all three slide clear. The stride engages on
the first accepted index past `circle_radius[3] = 45` that already has a
best standing — the *second* accept — so accepting one earlier moves that
index from 135 to 119 and the phase from `0` to `2 (mod 3)`. 143 is the
last index of ring 6, `146` is past `end`, and the last accepted candidate
wins: the dock lands on the original's own cell, at the original's own
point.

**The diff.** `run56_s_buildings_stand_where_the_original_s_do` is
**92,626 fields, zero wrong** — every linked building of both players on
all 3,000 frames, where it was one field of one building. Nothing else in
either suite moves; the fuzzed map's frame 1 and run20's frame 1 are
untouched, because no draw is spent on a dock's spiral or its slide.

**The lineage test, applied where the crate had the identity.**
`produce_building` asks `is(0x1b0)` three times — the spiral's `start`, the
site block's `w`/`h`/`max`, and this arm — and `is(0x1b7)` once for the
tower. The three dock ones are now `build::is_dock`; the tower is still
`ident == Ident::Tower` and is a residue of the same kind.

**What it does not close.** East Indies' word still parts at **3021**, and
with the dock right the seam moved *upstream*, to frame 2977 and to a
different mechanic: the builder's approach. The original's `o 11` carries a
four-deep `PATHDATA` chain to `(43704, 41688)` — cell `(56, 54)` plus the
order's own `off 696, 216` — and this crate sends it to `(43896, 41400)`.
Both are points of the swarm ring of `min(x_size, y_size)·0x60 + 0x30`
around a 4×4 site, `0x30` outward; they are ~25° apart on it, so the ring's
sweep, not its radius, is what differs. `docs/ORDERS.md` §5.4 and
`docs/QUEUE.md` item 131.

**Coverage.** Diff-backed: the arm's existence, its base, and its
first-clear rule, all by run56's building positions — the slide has no
draw of its own, so the dumped position is the only oracle it can have,
and it is now a test. Reading-only: the `dy`-starts-at-`−hx` quirk and the
unpadded base, both invisible on a 4×4 dock and both taken from the
listing.

## 22. A farm's distance is in tiles, and the cell's value is not zero (2026-09-01)

`produce_building`'s spiral computes the candidate's distance from the
anchor **twice**, in two different units, and only the general arm at
`006e1f9a` is the one §2.20 described:

```
006e1f9a  eax = cand_cell_x − anchor_cell_x        # cells
          esi = cand_cell_y − anchor_cell_y
          d   = vector_dist(|eax|, |esi|)          # the Woodcutter's, and the d > 4 arm's
006e2004  esi = cand_cell_y * 4 + 2                # tiles, the cell's centre tile
          eax = cand_cell_x * 4 + 2
          eax −= div_3_table[(anchor->x ^ 0x63637) >> 6]     # the anchor's *exact* tile
          esi −= div_3_table[(anchor->y ^ 0x63637) >> 6]
          d   = vector_dist(|eax|, |esi|)          # FARM (0x1a1) and MINE (0x1a3) only
          score = 4000 / max(d, 1) + rand % 500
```

Two differences, and both matter:

- **Tiles, not cells.** `vector_dist` is `max + min² / (2·max)`, so in
  cells a straight neighbour and a diagonal one are both `1` and the
  `4000 / d` term cannot tell them apart. In tiles they are `4` and `6`,
  and the term is **1000 against 666** — a third of the whole score, where
  the random part spans 500.
- **The anchor's own position, not its cell.** `>> 6` then `div_3_table`
  is units → tiles (a tile is `3 × 64`); the general arm's `>> 8` is
  units → cells. The two agree only for a building centred in its cell,
  and the anchor of the call below is not one: it stands at
  `(34656, 36192)`, tile `(180, 188)`, where its cell `(45, 47)` would
  centre at tile `(182, 190)`. Both halves of this are load-bearing.

And the constant beside it is not a constant. `local_60 == 0` — not a
gather-scored type — adds `0xff − WData.val`, the **cell's value byte**,
so a cell the map maker rated a *better* city site scores **lower** here.
§2.20's note that it is "0 on this world" was read off the flat harness
world; run38's own dump gives the three cells East Indies' second farm
parts on `val 20`, `31` and `27`.

**The call this decides.** Frame 3176, player 1's Farm, anchor cell
`(45, 47)` at `(34656, 36192)`, `start 0`, `end 105` (`rings 5`). The
stride never engages — nothing past ring 1 can reach the standing best —
so the walk is 105 cells either way and every candidate that reaches the
score is drawn for either way. What changes is only which of them wins.
Every candidate below is one of the anchor cell's eight neighbours, so in
cells `d` is 1 for all of them; in tiles, measured from the anchor's own
point, they are not:

| idx | cell | tile Δ | `d` cells | `d` tiles | `r % 500` | `val` | cells + `0xff` | tiles − `val` |
|---|---|---|---|---|---|---|---|---|
| 3 | `(44, 48)` | `(−2, 6)` | 1 | 6 | 367 | 20 | 4622 | 1268 |
| 5 | `(45, 48)` | `(2, 6)` | 1 | 6 | 398 | 31 | 4653 | **1288** |
| 6 | `(46, 46)` | `(6, −2)` | 1 | 6 | 337 | 6 | 4592 | 1252 |
| 7 | `(46, 47)` | `(6, 2)` | 1 | 6 | 112 | 21 | 4367 | 1012 |
| 8 | `(46, 48)` | `(6, 6)` | 1 | 9 | 423 | 27 | **4678** | 1095 |

In cells all five are `4000 / 1`, the roll is the whole score, and the
last one drawn with the best roll wins — index 8, one cell east of the
original's. In tiles the far diagonal is `4000 / 9` against everything
else's `4000 / 6`, 222 less than the spread of a roll, and index 5 is the
original's own `x_internal 34944`. Nothing in ring 2 or beyond comes
within 200 of it.

**Why no earlier capture could see it.** Every farm and mine placed before
this one either had a friend (`find_friends != 0`, which skips the arm
entirely) or had one candidate so far ahead that the `4000 / d` term could
not overturn it. The arm is *drawn* on every one of them — that is what
`SITE_SPIRAL` counts, and the counts have agreed since §2.20 — so a draw
diff can never reach it. **Only the dumped position can**, and it took a
capture long enough to contain a second farm sited among equals.

**The diff.** `run57_s_four_thousand_frames_stand_where_the_original_s_do`
is **130,326 building fields, zero wrong** over 4,000 frames — it was two
buildings and 850 field-frames — and its collision block grows to
**330,643 field-frames, none wrong**, with nothing leaving the original's
point before 3582. East Indies' word on the long capture goes **3435 →
3579**. Nothing else in either suite moves.

**Coverage.** Diff-backed: the arm's unit (tiles), its origin (the
anchor's own point rather than its cell's centre) and the `− val` term,
all three by run57's building positions on the frame the readings
disagree — the call above separates every one of them. Reading-only: the
`div_3_table` shifts themselves, `>> 6` here against `>> 8` at `006e15b7`,
taken from `llvm-objdump` over `006e2004`–`006e2073`; and that the same
arm runs for a **MINE** (`0x1a3`), which no capture has placed.

## 23. The sweep runs for a human leader too, and `ter` is never written (2026-09-02)

Both come from the same place: the `CITY` record, widened from four fields
to forty and compared on every frame of run58 (`docs/CITIES.md` §5.7).

### 23.1 `strategy_all`'s gate has no test for a human

`Leaders::strategy_all@006ed430` walks the leader array and admits every one
whose `leader_flags & 3 == 3` — in play and not defeated. There is **no**
human test:

```
for L in leaders:
    if (L.flags & 3) != 3: continue
    Leader::check_explore(L); Leader::plan_strategy(L)
    Leader::compute_score(L, 0); Leader::diplomacy(L)
```

and `Leader::plan_strategy@006b9620` has none either, at its head or
anywhere in its body. The human is filtered one level *down*, in
`Leader::production_ai@006c1960`, whose first statement is

```
if ((flags & 4) and not (flags & 8)) or ai_off or (field_0x4 & 4):
    goto default          # `field_0x788 = 0; return`
```

— human (`0x4`) without computer assist (`0x8`) falls to the switch's
`default`, and the default **clears the step machine**. So the human
leader's cycle is: sweep on its phase frame, arm the machine, produce
nothing on the next frame and disarm it, sweep again on the next phase
frame — forever. Everything `plan_strategy`'s sweep computes is computed
for a human; only what `production_ai` would *do* with it is skipped.

**The dump says the same thing twice.** In run58 the human's city carries a
full site picture from frame 1 — `busy 5`, `gatherers 5`, `ocean 25`,
`land 84`, `filled 40`, `dock_tile 1`, `space 50/50/44`, `ter 1/2/0/1/0/0` —
where a leader whose sweep never ran would carry zeros; and its
`peasant_dist` moves 1 → 2 on frame **401**, which is leader 0's own phase
(`(who·0x19 + frame) % 200 == 0` → frame ≡ 0, and the log reports the
change one frame late, exactly as leader 1's 175-phase changes show at 376,
576, 776).

`Sim::strategy_all` skips a human outright instead, so the human's census
never runs and its thirteen site fields stay zero on all 5,201 frames.
**Not yet fixed, and not a one-liner**: this crate's sweep would then also
run step 16, which seeds an army — and the original's human has no
`ARMYDATA` record on any capture, so a gate this crate does not model sits
between step 13 and step 16. `Sim::check_orphaned_buildings` already has
its own human bail; `check_explore` here only writes `census.explored` and
issues nothing.

One reader does not wait for that fix. `Region::go_here` reads the human's
`reg_cities`, and it now takes them from `Sim::leader_reg_cities`'s recount
(`docs/TRANSPORT.md` §9.4, item 642). The census's zero had sent East
Indies' scout to the wrong island on 11549.

### 23.2 ~~`ter[6]` has no writer~~ — written, 2026-09-02: §24

`CityData::ter[6]` is the best per-good gather amount over the city's
occupied tiles, written by step 13's circle sweep out of
`World::gather_at@006b07f0`. `World::gather_at` is a **declared seam** in
`crates/sim/src/ai_census.rs` that answers `[0; 6]`, so `CityAi::ter` is
zero everywhere, for every player, on every frame.

It is not inert. `gather_value` — §3.2, the gather families' half of the
make-list score — reads it per good:

```
let ter = f.ter[g];
if !(ter != 0 || oil_ok) { continue; }      // ai_build.rs
```

`oil_ok` needs `oil_patches.count`, which reads 0. So with `ter` zero the
loop admits **no good at all** and `gather_value` returns `None` for every
gathering building of every city — the AI's make list can never ask for a
farm, a camp, a mine or a university. (The script path is separate: §19's
`place_woodcutter` sites camps without going through this.)

`World::gather_at` needs the `lands` table — `lands[class]` carries four
good indices at `+0x04..+0x10` and four amounts at `+0x14..+0x20`, stride
`0x138`, written by `Lands::init@0067e730` — plus the nine-neighbour tile
pass and the `GoodType` predicate behind its `×2`.

**Coverage.** The seam is closed: §24 is `World::gather_at` read whole and
implemented, and `ter` now agrees per good, per city, on every frame of
run58. Reading-only: the claim in §23.1 about step 16 — no capture has a
human army to falsify it with, because no capture has a human army at all.

**And the gate was not the whole gate.** With `ter` written, the make list
still asks for no gathering building anywhere in run58 — because
`Sim::building_value` is not reached **at all** in that capture, on either
pass, so `gather_value` never runs. The AI's camps and farms there come off
the script path (§19). What §23.2 called a hard gate is a gate on a road
run58 never drives down; ~~the road itself is the open question~~ —
**answered 2026-09-02, §25: the original does not drive it either.** The
step machine cannot leave step 1 while the script is live, and
`create_buildings` is first entered on frame **9982** of East Indies and
**6382** of Great Lakes, thousands of frames past both words and past the
end of every dump.

## 24. `World::gather_at`, whole — the lands table and the flat predicate (2026-09-02)

§23.2's seam, read and landed. `World::gather_at@006b07f0` is the only
writer of `CityData::ter`, and the only reader of the `lands` table.

### 24.1 The table — `Lands::init@0067e730`

Nine `<LAND>` records out of `rules.xml`, four `<MAKE num type>` each, into
a `0x138`-stride array: the good indices at `+0x04..+0x10` through
`Types::good_key@006691c0`, the amounts at `+0x14..+0x20`. The loader
refuses to start unless the counts are exactly `NUM_GATHER_LAND` (9) and
`NUM_MAKE` (4), so both are facts about the engine and not about the file.
`"none"` resolves to `TYPE_NONE` = −1, which the consumer's **unsigned**
`< 6` test rejects rather than indexing backwards.

Five of the nine make anything:

| class | name | makes |
| --- | --- | --- |
| 0 | Land | knowledge 1, food 1 |
| 1 | Sandy | — |
| 2 | Ocean | — |
| 3 | Coast | — |
| 4 | Forest | timber 1 |
| 5 | Mountains | metal 1 |
| 6 | Rocks | — |
| 7 | Oil | oil 1 |
| 8 | Cliffs | metal 1 |

`sim::world::LANDS` is this table and `cargo run -p rondata -- <install>`
re-derives it from the install's own `rules.xml`.

The tail of `Lands::init` calls `GoodType::compute_largest_gather@0066e920`
on all six goods. Nothing here has read it, and `ai_build.rs` still notes
`largest_gather` as reading zero.

### 24.2 The class — `WorldData::get_land@006b4730` with its third argument 1

`WData.land` — `land_key[]`, four names, `BASELAND`/`SANDY`/`OCEAN`/`NONE`
— is only the fall-through. Five flag tests come first, in this order:

```
flags & 0x004        -> 3   Coast
flags & 0x020        -> 4   Forest
flags & 0x050        -> 5   Mountains   (MOUNTAIN and the unnamed 0x40)
flags & 0x008        -> 6, or 7 with 0x800     is_rocks, then is_oil_at
is_ocean && 0x800    -> 7   Oil
else                 -> WData.land
```

**`WData.flags & 0x800` is `OIL`**, and this is what settles it:
`WorldData::is_oil_at@00472af0` is that bit and nothing else. The map dump
prints no word for it (`docs/ORACLE.md`, "The map is a dump too"), so it had
stayed unnamed since run20.

`Leaders::plan_strategy`'s muster search computes exactly these five tests
inline (`docs/ARMY.md` §13), which is what says the two are one function;
`sim::world::World::land_class` is now both.

### 24.3 The predicate — `GoodTypeData::is_flat@004780c0`

Vtable slot `+0x94` of `GoodType`, which the base `Type` leaves as the
engine's return-zero stub (`GoodType::vftable_for_Type_@00b44b70+0x94`,
read out of the PE against `rise_z.map`; the decompiler inlines it at both
call sites and prints the devirtualised `ObjectTypeData::is` at slot
`+0x60` in its place). The body is

```
!(is(TIMBER) || is(METAL) || is(OIL))
```

so **food, wealth and knowledge are flat and timber, metal and oil are
not**, and the whole shape of `gather_at` is that split.

### 24.4 The two arms

The fifth argument decides, and the two callers disagree on it.

**`plan_strategy`'s step 13 passes 1** — the centre-only arm. Read the
cell's own class, then over its four makes with a non-zero amount and a
good `< 6`:

- a **flat** good adds its face amount;
- a **non-flat** good adds *twice* its amount, and only if the cell's
  centre tile is not already gathered from (`TData.mask & 0x1000`,
  `WorldData::is_gathered_from@00472ac0` — the tile bit `docs/` already
  named for the gather-site pass).

That is the whole of what `CityData::ter` is: `ter[g] = max(ter[g], …)` over
the circle's occupied and wide-open cells. It is why every `ter` in the
captures is 0, 1 or 2 and never more.

**`produce_building`'s gather score passes 0** — the neighbourhood arm. The
flat goods still come off the centre alone; a non-flat good takes *nothing*
there, is summed over the cell and its eight neighbours (`move_x[0..8]`, the
centre first), each skipped if off the map or already gathered from, and the
whole non-flat half is doubled once at the end. The neighbour pass does not
re-test `amount != 0`; a `"none"` slot's good is −1, which matches no good,
so the two gates come to the same thing.

### 24.5 Coverage

**Diff-backed**, and this is the centre-only arm entire: run58's `CITY`
record carries `ter[6]` on every live city of every frame, and the nine
rows that were pinned as wrong — `1/2000`'s `ter[0]`, `[1]`, `[3]`, `[5]`
and `1/2007`'s `ter[0]`, `[1]`, `[3]`, `[4]`, `[5]` — are gone: **39,309
field-frames** that disagreed now agree, first frame to last. `run20`'s
frame-1 `CITY` check compares the same six.

**Reading-only**: the neighbourhood arm. Its only caller is
`produce_building`'s gather score, whose own consumers (`w1`, `plenty`,
`type_avail`, `get_need`) are still seams in `ai_place.rs`, so no capture
reaches it. It has unit tests and no oracle.

**Not established**: `GoodType::compute_largest_gather`, and where
`World::set_gathered_at@006b46b0` is called from — no caller of it survives
in the decompile export, so the writer of the bit `gather_at` reads is known
here only through this crate's own gather-site pass.

## 25. When the step machine leaves step 1 — the ladder, dated (2026-09-02)

§23.2 ended on an open question: with `ter` written, the make list still
asks for no gathering building anywhere in run58, because
`Sim::building_value` is not entered on any of its 5,201 frames. The item
was booked as "find where the step machine stops short of
`create_buildings`". **It does not stop short. It has not got there yet, and
neither has the original.**

### 25.1 What the coverage says

The trace's HIT records are function-entry coverage
(`tools/trace/README.md`): outside a `window=` every listed function is
armed once from attach, so a whole-run capture carries **one record per
function, on the frame it was first entered**. The two 24,000-frame traces
answer the question outright:

| step | function | East Indies (run54) | Great Lakes (run53) |
| --- | --- | --- | --- |
| — | `Leaders::strategy_all`, `Leader::plan_strategy` | 0 | 0 |
| — | `Leader::production_ai` | 1 | 1 |
| 2 | `production_ai_setup@006c83e0` | 9977 | 6377 |
| 3 | `found_cities@006c7a60` | *(576)* | *(576)* |
| 4 | `research_techs@006c6ba0` | 9979 | 6379 |
| 5 | `upgrade_units@006c6430` | 9980 | 6380 |
| 6 | `create_units@006c40a0` | 9981 | 6381 |
| 7 | `create_buildings@006c1be0` | **9982** | **6382** |

Five consecutive frames with one gap, and the gap is step 3: `found_cities`
was entered at **576** already and a one-shot arming does not fire twice. So
this is §2.4's ladder read straight off the original — `production_ai_setup`
at step 2, then one producer a frame — dated on two maps.

**Why it waits.** The script at step 1 answers `BLOCK_ON_THIS` on every
sweep for as long as it is live, and `BLOCK_ON_THIS` clears the machine
(§2.4). So the ladder cannot leave step 1 until the script *ends*, and the
shipped opening runs for around two and a half hours of game time.

**And the dump says it from the other side, on the same game.** run18b is
run10's `LEADERS=9` window and run53 is run10's game traced whole, so the
two instruments can be laid on each other frame for frame. §15.6 read
run18b's ladder as one sweep among many; with the coverage beside it, it is
the **first**, and every number lines up once the label is taken off (a
`FRAME n` block is the end of sim-frame `n − 1`):

| sim-frame | run18b's record at the end of it | run53's HIT |
| --- | --- | --- |
| 6375 | `step 1`, `script_step 28`, `prod_script_run 1` — the sweep arms | |
| 6376 | `step 2`, `script_step 29`, **`prod_script_run 0`** — the script's last call | |
| 6377 | `step 3` | `production_ai_setup` |
| 6378 | `step 4` | *(`found_cities`, hit at 576)* |
| 6379 | `step 5` | `research_techs` |

So `SCRIPT_DONE` is not an inference: `prod_script_run` falls 1 → 0 on the
frame the script last runs, which is the only arm of §2.4's switch that
writes it, and the ladder starts on the next frame. Two instruments, two
captures, one frame apart from nothing.

**And the 576 is the second half of the finding.** `found_cities` and
`make_stuff` are entered on frame 576 of every East Indies and Great Lakes
capture, thousands of frames before step 3 or step 8 can run. Their caller
is `ScenarioFuncSet::place_city_with_cost@009f5860` — entered on 576 too —
which is the **script's** host function, not the machine's. `found_cities`
calls `make_stuff` itself. Every producer entry before the script ends is
the script's, which is §3's "the skirmish opening is the shipped script"
stated as coverage rather than as a reading.

### 25.2 What it means for the make list

`create_buildings` is first driven at **9982** on East Indies and **6382**
on Great Lakes. Those are 4,606 and 4,580 frames past each map's word
(5376 and 1802), and past the end of every dump on disk — run58, the
longest, stops at 5,201. So:

- `Sim::building_value` answering nothing in run58 is **agreement**, not a
  defect, and nothing in §3.1/§3.2 — `gather_value`, the `ter` gate,
  `oil_patches.count`, `GoodType::compute_largest_gather` — can move either
  headline until the word reaches those frames.
- §23.2's "the AI's camps and farms come off the script path" is now the
  whole story rather than an observation about one capture: on this game
  there **is** no other path for the first two and a half hours.
- The make list's own machinery is not idle in the meantime — `make_me`,
  the expiry and `make_stuff` all run under `found_cities` — but nothing
  ever *fills* it from a producer step before 9977.

### 25.3 Coverage

**Diff-backed**, on the original's own coverage records:
`diff::tests::the_producers_are_not_reached_until_the_ai_script_ends` pins
all twelve frame numbers above, on both maps, plus run58's two absences.
`rondata::trace` keeps the HIT records now (it dropped them until
2026-09-02), so "which functions has the original ever entered, and when"
is a `#[test]` fact here rather than a `report.py` reading.

`SCRIPT_DONE` on the last call is diff-backed too, on Great Lakes:
run18b's `LEADERS=9` window covers sim-frame 6376 and prints
`prod_script_run` falling 1 → 0 there.

**Not established**: the same for East Indies. 9976 is inside no capture's
`LEADERS=9` window, and the map's script is `economic.bhs` rather than
run10's `defensive.bhs`, so the *frame* is this map's own and only the
coverage says where it is.

## 26. A friend is a footprint, not a centre — the city is four cells' neighbour (2026-09-02)

Item 164. `Leader::produce_building`'s spiral scores a candidate cell by
`BuildTypeData::find_friends` (§2.20), and `find_friends` asks each of the
eight neighbouring cells *which building of mine is there*. This crate read
that as **a building whose centre is in the cell**. It is not: it is
`ObjectsData::find_building_placed_at@00658c80`, and that function answers
by **footprint**.

### 26.1 What the function does

`find_friends@00639270` probes the neighbour cell at its **centre tile** —
`find_building_placed_at(cell.x·4 + 2, cell.y·4 + 2, who, −1, −1)`. The
callee:

- refuses without looking unless that tile's own mask carries
  `(mask & 3) == 3` or `PLACED` (`0x80`) — so a tile no building has ever
  masked is free;
- walks the **nine cells** around `tile >> 2`, following each one's object
  chain (`WData` `+0x8`/`+0xa`, the head pair `collide.rs` already keeps);
- takes the first object that is `vtable[0xc]` — `SubObjectData::is_active`
  by `vtables.txt`, *not* "placed and unstarted" — whose owner passes the
  `param_3` filter, and whose own corner satisfies
  `corner ≤ tile < corner + size` on both axes.

The last line is the whole of it. A building is the neighbour of every cell
whose centre tile its footprint covers.

### 26.2 Why it is a four-cell difference for the one building that matters

Centre tiles are four tiles apart, so how many of them a building covers is
decided by its size alone:

| `X_SIZE` | tiles spanned | centre tiles covered |
| --- | --- | --- |
| ≤ 4 | 4 | exactly 1 |
| 5–7 | 5–7 | 1 **or** 2, by where its corner falls |

So for every ordinary building the two readings agree, which is why this
survived twenty-two items — and the exception is the **city centre**.
East Indies' AI Village is 7×7 at `(39264, 40032)`, corner tiles
`201..207 × 205..211`, and it covers the centre tiles of cells `(50, 51)`,
`(50, 52)`, `(51, 51)` and `(51, 52)`. Its own cell is `(51, 52)`. Every
site the AI ever scores is next to its city, so the friend it was missing
was the one the score exists to find.

### 26.3 The frame it moves, and the market it moves there

East Indies' word was **5376**, `produce_building`'s jitter spending
**two** draws against the original's four (§2.20: one per unblocked
sub-position of the 2×2). The frame is the AI's Market, and the two are the
same defect:

| cell | friends, by centre | score | friends, by footprint | score |
| --- | --- | --- | --- | --- |
| `(49, 51)` | 0 | 1244 | Village E +2, SE +1 = 3 | 5244 |
| `(49, 52)` | Library SE +1 | 3252 | Village NE +1, E +2, Library SE +1 = 4 | **6252** |
| `(49, 53)` | Library E +2 | **4251** | Village NE +1, Library E +2 = 3 | 5251 |

The old winner `(49, 53)` puts the Market's 4×4 corner on tile 196, and
the jitter's `+1` column runs into the Library at tile 200 — two
sub-positions refused, two draws. The new winner `(49, 52)` is four rows
north of the Library, and all four sub-positions clear. Four draws, and
**the word goes to 5437**.

### 26.4 Coverage

**Diff-backed.** `LONG_WORD_EAST_INDIES` is 5437, and
`run54_s_24000_frames_are_where_the_second_map_s_word_now_parts` fails if
it falls; nothing else on the board moved (Great Lakes stays 1802, run59's
census stays 3500 of 18,000, run58's 178,326 building fields stay 0 wrong).
`the_city_centre_is_the_friend_of_four_cells` pins the geometry directly
against the Village's own dumped position.

**Not established.** The nine-cell chain walk is not modelled — this crate
scans its building list and filters by footprint, which is the same answer
whenever no building's footprint reaches more than one cell away, and every
type in `buildingrules.xml` is at most 7 tiles. **Order** under a tie is
therefore this crate's list order rather than the chain's, and no capture
has yet put two of one leader's buildings on one centre tile — which
`mask_me` makes impossible for live buildings anyway, since a second
placement on a masked tile is refused.

`place.rs`'s own `find_building_placed_at` still filters `!b.started` where
the original filters `is_active`. Its one caller is gated on the tile's
`PLACED` bit, which `mask_me` clears the moment a building starts, so the
two agree there; the general form is the one above.


## 27. `LeaderData::pop`, the number every producer's base is made of (2026-09-04)

Great Lakes' word parted at **6582** on one draw: the original spends a
third `Leader::make_stuff` roll, `+0x63d` — the expiry over the slot it has
just bought (§15.3) — where this crate spends its two `+0x221` and stops.
The slot is the citizen at 5, and `make_stuff`'s step 6 skips a slot whose
`val` is zero. The original's is **714**; this crate's was **0**.

### 27.1 What the field is, and who writes it

`create_units@006c40a0:289` opens every type's pass with

```
local_10 = (this->pop * 1000) / max(1, this->city_num)      // +0x95c, +0x3f8
```

and `research_techs@006c6ba0` does the same with `× 200`. So `pop` is the
whole of both producers' `base`: at zero, **every value either of them can
compute is zero**, and this crate never wrote the field. `docs/AI.md` §2.3
had it as "kept by the unit lifecycle, read here"; it is the **city**
lifecycle, and nothing was keeping it.

It is `CityData::get_pop_value@00738450` summed over the leader's live
cities — a pure switch on the city building's type:

| city type | `get_city_level` | `get_pop_value` |
| --- | --- | --- |
| `VILLAGE` (Small City) | 1 | **1** |
| `TOWN` (`0x19f`, Large City) | 2 | **3** |
| `METROPOLIS` (`0x1a0`) / `FORBIDDENCITY` (`0x213`) | 3 | **5** |

`2 · level − 1`, and the original keeps the running sum incrementally. Its
writers are five, and **every one of them adds or subtracts that function
of one city**: `City::init@00737050` (`+`), `City::close@00737550` (`−`),
`City::capture@00736c40` (both, across the two leaders),
`City::check_upgrade@00738b20` (the delta) and `Build::finished@00628490`
(the same delta, when the building that finished is what raised the city's
level). So a recount over the live cities is the same number at every
instant the sweep can read it, and each of those five is already a call
site of `Sim::sync_pop_cities`, which is where
[`Sim::sync_leader_pop`](../crates/sim/src/city.rs) now runs.

`reg_pop` (`+0xe62`, `ushort[64]`) is the same sum per region, written only
for a region index below `0x40` — the array's own bound — and `reg_cities`
(`+0x125e`) sits beside it as the count. The recount is repeated at the
census sweep's step 8 for one mechanical reason: step 8 is what `resize`s
the per-region arrays, so a `sync_pop_cities` that ran before the first
sweep — `build_sim` standing a dump up — would write `reg_pop` into an
empty vector.

### 27.2 The citizen's 714, end to end

run18b is run53's own game, and its `LEADERS=9` block at sim-frame 6580
carries every input. `create_units`' citizen branch (§2.18, the
`is_peasant` arm at `006c4cfc`) reads them in this order:

| step | the original's numbers | value |
| --- | --- | --- |
| `base = pop × 1000 / city_num` | `pop 2`, `city_num 2` | 1000 |
| `b = (infra_mod × base) >> 8` | `infra_mod 256` | 1000 |
| the `× 30` arm | Norwich has `busy 11`, so no | 1000 |
| the branch | `free 0 + busy 11 + q 0 = 11`, `slots 12` — **not** `< slots − 1`, so the `else`: `assigned < slots` holds and `v = b` | 1000 |
| `reg_gatherers < reg_gather_slots` | `19 < 22` | `× 3/2` → 1500 |
| `reg_gatherers × 2 < reg_gather_slots` | `38 < 22`, no | 1500 |
| `city->gatherers == 0` | 9, no | 1500 |
| `free + busy <= slots / 2` | `11 <= 6`, no | 1500 |
| `reg_peasants == 0` / `reg_free_peasants == 0` | 22 and 1, neither | 1500 |
| `queued + units` against `pop_cap / 2` and 40 | 22 against 25 and 40, neither | 1500 |
| **the tail**, `(check_income × ((want × v) / (want + units + queued))) >> 8` | `want_civ = pop_cap × 2 / 5 = 20`, `units 22`, `queued 0`: `20 × 1500 / 42` | **714** |

The dump's slot 5 reads `val 714` at that frame and `val 7` after the buy
(`val /= 100`, §15.4). This crate now reads 714 as well, and its
`make_stuff` at 6582 spends the `+0x63d` the original does.

**The tail is where the reading nearly went wrong.** 1500 is not 714, and
the temptation was to hunt for a missing multiplier in the branch above.
The divisor `want + units + queued` is the last thing `create_units` does
before `make_me`, and this crate had it right all along — the tail was
never the defect, and the arithmetic only closes when it is included.

### 27.3 Coverage

**Diff-backed.** `LONG_WORD_GREAT_LAKES` is **6612**, up from 6582, and
`run53_s_24000_frames_put_the_ceiling_where_run33_did` fails if it falls.
The census widening now compares `pop` and `reg_pop[home]` alongside the
other fourteen scalars in
`run9_s_frame_1_leader_record_is_the_census_after_the_sweep`, which was
made to fail on purpose first: with the value forced to zero it reports
`pop: ours 0 theirs 1` and `reg_pop[home]: ours 0 theirs Some(1)`. Nothing
else on the board moved — East Indies stays 7448, run58's building fields
stay 0 wrong, run59's census stays where it was.
`cities_tests::a_leader_s_pop_is_one_three_five_by_city_level` pins the
1/3/5 switch and the per-leader split directly.

**Not established.**

- **`create_buildings`' other offers.** At 6582 this crate's list carries a
  gather building (`t 418`, `cat 4`) at slots 1 and 4 at `val 41500`, and
  the original's slots 1–4 are **empty**. Neither offer changed the frame —
  slot 1 fails `can_pay_slot` and slot 4 is dropped as a duplicate of a
  ranked slot — so the draws still agree; the *list* does not. §2.19's
  arithmetic is what would settle it, and run18b's window covers it.
  **Narrowed 2026-09-06, §30.3**: the goods record over that whole window
  is now the original's, frame for frame, so the offer is not an
  affordability difference — it is `create_buildings`' own valuation or
  its gate.
- ~~**What is born at 6612.**~~ **No queue** — answered 2026-09-04,
  `docs/CITIES.md` §4.3. The AI's Barracks finishes its construction on that
  exact frame, and `Build::activate`'s high-water block pays a **British**
  leader its free archer; the three `Guy::init_real+0x52` and three
  `Guy::set_anim+0x97a < Unit::do_idle+0x7d` are **one** Bowmen, which
  `Objects::init_unit`'s `uber_size` loop makes as three one-figure objects.
  Great Lakes' word 6612 → 6650.
- **The other readers of `pop`.** Only `create_units` and `research_techs`
  are cited here. `Leader::plan_strategy@006b9620:1202` reads it too
  (against `+0x848`), and that comparison is unmodelled.

---

## 28. The producers' four draw sites, named (2026-09-04)

Every draw the production sweep takes reached the trace's comparison
under one coarse label — `strategy_all`, the mark `Sim::do_frame` writes
before `Leaders::strategy_all`. That is enough for a *count* and useless
for a *sequence*: the moment a producer spent a draw the original also
spent, the two `Vec<String>`s parted at that index anyway, so run53's
sequence number could never lead its count. Four sites are named now,
each the return address of a `call Random::get` in the listing:

| site | function | what it is |
| --- | --- | --- |
| `006c46e2` | `create_units+0x642` | the matchup bias over `unit_prod_value`, one per (city, military type) reaching it |
| `006c69d4` | `upgrade_units+0x5a4` | the same bias, one per eligible type |
| `006c2bdb` | `create_buildings+0xffb` | the wonder arm's `% 1000` |
| `006c2bf7` | `create_buildings+0x1017` | and its `% 300` |

The first two are §2.18's and §2.16's; the pair is §2.19's wonder arm,
which fires only when no wonder victory is on and nobody is winning one.
None of the four draws on difficulty 2.

### 28.1 Coverage

**Diff-backed.** Three of the four are exercised by run53 inside 6,782
frames — `create_units+0x642` on 6780 and the wonder pair on 6781 — and
naming them moved the long capture's *sequence* number from 6780 to
6782, where it now meets the count. The constants live beside the code
that spends the draw ([`sim::ai_units`], [`sim::ai_build`]) and the
addresses in `rondata::trace::SITES`, per that table's own rule.

**Not established.** `upgrade_units`' site has never fired in the
original on any capture here — it is named against the listing, and the
frame that would exercise it is the frame this crate stopped spending it
on (`docs/TECH.md`, "The starting position is a function of the
nation"). If it ever fires on both sides, the label is what will say so.

---

## 29. `mil_trainers`, the list a Barracks joins (2026-09-04)

`Leader::produce_unit`'s military-trainer arm and `produce_tech`'s (§2.17)
both search **`mil_trainers`** — `LeaderData+0x6e50`, a `SimpleArray<int>`
whose count is `+0x6e54`, capacity `+0x6e58` and list `+0x6e60`. Neither
walks the leader's buildings: a trainer that is not in this list is
invisible to both, and every unit the AI trains at a Barracks, Stable,
Siege Factory, Factory or Auto Plant is trained through it.

Nothing in this crate wrote it. `Leader::new` set it to `Vec::new()` and
only the unit tests ever pushed to it, so from frame 0 to frame 24,000 the
AI's `produce_unit` found no trainer and queued nothing military. That is
the whole of item 222.

### 29.1 The two writers

`grep` says there are exactly two, and they are the same pair that keeps
`reg_buildings` (§2.3 step 8):

- **`Wall::increment_stats@00643270`** — the function splits on
  `is_active` (`flags & 4`). The **active** arm counts the building's type
  into `reg_buildings[type]` and `reg_buildings[region][type]`, and then,
  if `BuildTypeData::is_military_trainer`, **appends the object's `o`** to
  `mil_trainers`. There is no duplicate test. The inactive arm is the
  *sites* counter and touches nothing here.
- **`Wall::decrement_stats@00642da0`** — the mirror, through
  `SimpleArray<int>::remove@00462e70`, which finds the **first** slot
  holding the value and shifts the tail down. The order of the survivors is
  preserved; it is not a swap with the last. So `mil_trainers` is in
  activation order, and `produce_unit`/`produce_tech`'s `>` tie-break gives
  the earliest-activated trainer among equals.

`is_military_trainer` is the **derived** `BUILD_FLAGS 5` read on the root
of the `FROM` chain (`docs/DATALAYER.md`), not a list of idents: an
upgraded trainer is still one.

### 29.2 Where they are called from

Four call sites, all in the building lifecycle:

| caller | which | guard at the call site |
| --- | --- | --- |
| `Wall::activate@0063e4b0:62` | increment | the type has `NO_CITY` (`build_flags & 0x10`), **or** the building is complete and in a city (`+0x72 ≥ 0`) — and `flags |= 4` is set two lines above, so the active arm is the one taken |
| `Build::close@00628980:99` | decrement | `flags & 4` (active), and the same `NO_CITY`-or-in-a-city test |
| `Build::add_to_city@00622380:43` | increment | **not** `NO_CITY`, and active |
| `Build::remove_from_city@00622030:124` | decrement | the same |
| `Wall::set_type@00640da0` | both | an upgrade in place: out, then in again |

The first two are what this crate implements —
[`sim::Sim::mil_trainer_open`] from `Sim::activate` and
[`sim::Sim::mil_trainer_close`] from `Sim::close_building`, each beside the
dock registry's own call, which is the same shape one field over
(`docs/TRANSPORT.md` §5.2–§5.3).

### 29.3 What it moved

Great Lakes' word, **6782 → 6848**. At 6782 the make list's head is two
**Longbowmen** — the British unique archer, `t 177`, `city 1`, `escrow 1` —
and `make_stuff` finds `can_pay` true and calls `make_this(0)`. With no
trainer to queue at, this crate's `produce_unit` returned "not queued"
without paying; the original queued both and paid `2 × (31 timber,
51 metal)`. Six lines later the same function tests slot 1 — a
**University**, 60 timber and 30 metal — against
`bucket[g] ≥ head_cost[g] + slot_cost[g] + need`, and 113 timber against
91 passes only on the side that never paid for the Longbowmen. So the
symptom was a building bought at slot 1 (`produce_building`'s two
`+0x1805` jitter draws and the two `+0x63d` expiries over its type) and
the cause was a unit *not* bought at slot 0, which spends no draw at all.

### 29.4 Coverage

**Diff-backed.** run53's long capture, which now runs 66 frames further.
The list's *contents* are not dumped by any capture on disk — no `LEADERS`
record prints `mil_trainers` — so what the diff checks is the consequence:
the frame the AI's resources stop agreeing.

**Established by reading, and asserted by
`a_trainer_joins_the_leader_s_list_on_activation_and_leaves_on_close`**:
the activation order, the order-preserving removal, the
`is_military_trainer` predicate on the root of the chain, and the
`NO_CITY`-or-in-a-city guard. Each of the four was made to fail once
before the test was kept.

**Not established.**

- **The two city call sites.** `add_to_city`/`remove_from_city` are
  unimplemented, so a trainer that changes city here keeps its position in
  the list where the original's would move to the end. No capture has a
  building change city.
- **`Wall::set_type`'s pair.** A trainer upgraded in place is removed and
  re-appended by the original, moving it to the end; this crate leaves it
  where it is. A Barracks does not upgrade in the shipped tree, but a
  Stable → Auto Plant line does.
- **Capture.** `Wall::swap_team@00640c00` copies the flag byte straight
  across and calls neither `activate` nor `increment_stats`, so the new
  owner's list does **not** gain the building until it joins a city — and
  this crate's `Sim::swap_team` copies `active` the same way, so the two
  agree by accident rather than by construction. No capture on disk has a
  captured building.
- **The original stores `o`, this crate stores the building index.** The
  two coincide in ordering but not in value, and a dump comparison would
  have to translate.

---

## 30. The University the AI buys and the original does not (2026-09-06)

Great Lakes' word stood at **6982** on a single make-list decision. Both
sides run `Leader::make_stuff` on that frame; the original spends three
draws (two `+0x221` over the head's type, one `+0x63d` over the slot it
buys) and this crate spends five — the same three plus
`produce_building`'s two `+0x1805` jitter draws, which is a **building
placed**. It is §29's shape a second time: the wrong thing is not in the
mechanic the draw points at.

### 30.1 The two lists, side by side

run84 (`LEADERS=9`, Great Lakes 6950–7029, taken for this item) prints the
original's eleven slots at the end of every frame. At dump-frame 6982 —
the state `make_stuff` reads on sim-frame 6982 — the two lists are the
same shopping list ranked differently:

| slot | the original | this crate |
| --- | --- | --- |
| 0 | `t 428` Stable, `val 5722784`, cat 7 | `t 420` University, `val 6075000`, cat 4 |
| … | `t 420` University, `val 1518750`, at 2 and 4 | `t 428` Stable, `val 5722784`, at 7 |

The **University's own valuation** is the whole difference: 6075000 here
against 1518750 there, exactly four times, and 6075000 is what the
original itself carried until its list was rebuilt at 6977. Four is
`Leader::check_income@006cc800`'s: it answers `0x100` for a type the
leader can afford one of and **`0x40`** for one it cannot when escrow is
on, and `create_buildings` multiplies the offer by that over 256 before
`make_me`. The original could not afford a University; this crate could.

So the head became the University instead of the Stable, `can_pay(0)`
passed, `make_this(0)` placed the building, and two draws appeared that
the original never spends. **The AI's arithmetic was right at every step.**

### 30.2 What it actually was: fourteen wealth, from two places

A University costs 60 timber and **30 wealth**. On sim-frame 6981 the
original's bucket reads `[99, 85, 28, 103, 100, 0]` and this crate's read
`[100, 87, 42, 103, 100, 0]` — food, knowledge and metal exact, wealth
**fourteen** over. Twenty-eight is under thirty and forty-two is not.

The fourteen were two separate defects, neither of them in `ai_build.rs`:

- **Twelve of them were the caravan's** — `CityData::trade_val` 240 a
  city here against the original's 128, from `get_trade_value` counting
  `members.len()` instead of `num_buildings`, and from `Caravan::distance`
  banding a world-unit distance against a tile width.
  `docs/CARAVAN.md` §4 and §8.
- **The last two were the ramp's** — the group half of
  `get_support_count` had no queued counter here, so the two Longbowmen
  the AI ordered on sim-frame 6782 both paid the first one's price
  (31 timber, 51 wealth) where the original paid 31/51 then **33/53**.
  `docs/COSTS.md`, "The count".

Both were invisible on their own: a trade route pays a city rate in
sixteenths and a ramp step is two of a good, and neither spends a draw.
What made them visible was a *third* mechanic's decision two hundred
frames downstream.

### 30.3 Coverage

**Diff-backed.** Great Lakes' long word **6982 → 6994**, and the new
parting is a movement one (`Unit::do_move+0xe84`) rather than an economy
one. East Indies unmoved at 7448. The goods record itself is now pinned
frame for frame over two windows — `rondata::diff::tests::great_lakes_
goods_record_and_its_trade_routes_are_the_original_s`, 21,312 good-frames
and 1,785 city-frames, all exact — which is the check that would have
caught either defect on the frame it started rather than on the frame the
AI acted on it.

**Not established.**

- **The original's own escrow flag on this entry.** run84 reads `escrow 1`
  on the University at 6982 and this crate writes 0. It changes nothing
  here — an unaffordable entry with escrow off would be worth `0` rather
  than a quarter, and this crate's is affordable either way — but the
  block that sets it (`create_buildings@006c1be0:1120–1180`, the
  `can_pay(0)`-of-the-head test with its gather-building arm) is
  unimplemented. It is the next thing to read in this neighbourhood.
- **§27.3's other offer stands.** At 6582 this crate's list still carries
  a gather building (`t 418`, cat 4) at slots 1 and 4 that the original's
  does not. The goods record over that window is now exact, so whatever
  produces it is not an affordability difference.
- **The make list is diffed only where the original's own transitions
  are** (§15.7's replay). Nothing compares this crate's *offers* against
  the original's slot for slot on a frame both sides fill, which is what
  would have shown both of the above at once. run84's eighty frames are
  where that check would now be cheapest.

---

## 31. The census counters are bytes, and `free` wraps (2026-09-07)

§2.3 step 2 zeroes `free`/`busy`/`gatherers`, step 10 increments them and
§2.20 decrements `free` or `gatherers` — and all three are **byte** operations on
`CityData`, not `int` ones. `+0x5a free`, `+0x5b busy` and `+0x5c gatherers`
are `uchar` in the type record, and every writer is a bare `*p = *p ± 1`
with no clamp, unlike the `space[]` loop six lines from §2.20's decrement,
which clamps at zero explicitly.

So §2.20's `free--` at zero lands on **255**, not −1, and every consumer
that *adds* the counter rather than testing it against zero sees the
difference: `find_gather_spot`'s "fewer than two citizens" gate, and
`create_units`/`create_buildings`' `free + busy` against a city's slots.
run79's `1/2007` is the case, and it is diff-backed — the full evidence,
the falsifier and the eleven fields of the same record that are still `i32`
here are `docs/CITIES.md` §5.8 and `docs/DATALAYER.md` §4.2 (item 265).

---

## 32. Great Lakes' word at 7585 is a Citizen the stockpile cannot buy (2026-09-07)

**What the word is.** run89's sim-frame 7585 is `production_ai`'s **step
11** — `make_stuff(); step = 0`, the call site `Leader::production_ai+0x236`
— and it spends **seven draws against the original's nine**, parting at
index 2 where the original takes a `Leader::make_stuff+0x63d`
(`SITE_EXPIRE_SLOT`) this crate does not. Both sides spend the two
`+0x221` of §2.6 step 4 before it, so the head and its one duplicate agree.

**What `+0x63d` is here.** Step 6's expiry, over the slot it has just
bought: `make_this(slot)` and then one `Random::get(0, 0xffff)` per slot
from `slot` to 10 holding that slot's type. The original spends exactly
one, and the trace shows **no draw between the `+0x221` pair and it** — so
whatever `make_this` bought took no draw of its own, which rules out
`produce_building`'s two (`make_stuff+0x45a` is how frame 7182's slot
purchase reads) and leaves a unit or a tech.

**The dump names it.** Block **7586** — the state at the end of sim-frame
7585, the same relation `1/3`'s spot on 7585 has to the stand on 7584 —
takes player 1's city building `2007` (`orig_type 414`) from `queued 0` to
`queued 1`, with one item `type 50`, `job_counter 100`, `cost[0] 43`, over
the stale `type 50 … cost[0] 42` the array still carried from the citizen
before it. A **Citizen**, at the price this crate's own tables give the
next one. It is the only queue row that parts at or below that block, and
this crate queues nothing.

**So the slot is 5, and every gate but one agrees.** The make list on that
frame, read off this crate:

| slot | t | val | cost | fate |
| --- | --- | --- | --- | --- |
| 0 | 82 Slingers, `num 3`, cat 6 | 9 999 999 | 55 food, 55 timber | head |
| 5 | 50 Citizen, cat 5, city 2 | 697 | 43 food | the buy |
| 6 | 82 Slingers | 9 999 999 | — | the head's duplicate |

`can_pay(0)` is 0 (timber 14 against 55), so the head is not bought and
`saving` holds; `need` is `((55 − 88) + (55 − 14)) / 2 = 4`; the type is
available and `TypeData::can_pay_cost@00667570` says **two** Citizens are
affordable against `num 1`, so the affordability test the slot reaches
would have passed. The gate that fails is step 6's **good loop**: food is
the one good both the head and the slot cost, and `88 < 55 + 43 + 4`.

**The slot-5 exception is not the way in.** `make_stuff@006c8af0:204` skips
slot 5 unless `free_peasants` **and** `gatherers` are both zero
(`LeaderData +0x9bc`, `+0x9c4`, both named in the type record). They are 1
and 20 here, and run19's own `LEADERDATA` at `LEADERS=9` has a mid-game AI
at 2 and 20 — the clause is a no-workers-at-all escape and never fires in
a running economy. The per-city halves of the same census
(`free`/`busy`/`gatherers` on `CITY`) agree with the original on all 246
blocks, so the counters are not what is wrong.

**The threshold, exactly, and it is not the arithmetic.** `need` falls as
the purse rises, so the least food that buys is **98**, not 102. Given
that food alone on the word's own frame, this crate's 7585 agrees with the
original **nine draws for nine, entry for entry** — including the bird's
`Guy::set_anim+0x97a < Guy::move+0x19f` at index 4, which is downstream of
the missing draw rather than a second fault: one word behind, the
wing-beat coin reads `CHAR_JOG` and `Guy::move`'s `field_0x9c == 8` does
not stand for a jog.

~~**So the word is the AI's stockpile, not its rules.** Bounds from the
original's own refusals: slot 5's purchase puts food **≥ 98** on 7585,
and 7582's step-8 `make_stuff` spends two `+0x221` and no `+0x63d` while
slot 9 (Empire, 160 food) is blocked by nothing else, so food was **< 160**
three frames earlier. This crate has **88**. The gap is 10 to 72.~~
**Refuted by run91 — §34.** The original's food on 7585 is **88**, this
crate's own number, and the ladder is identical tick for tick over the 72
blocks below the word. Both bounds came from assuming the original's
make-list *head* is this crate's; it is a **Temple**, which costs no food,
so the good loop never tests food and the arithmetic above never runs.

**What is not established: where the gap comes from.** ~~Nothing in the
harness compares a leader's `bucket`, because the ledger is written only
at `LEADERS=9` and run89 is at run87's detail; no Great Lakes capture on
this disk carries it past setup, run53's own 24k dump included
(`200/200/100/0/0/0`, its first frames).~~ **Both halves of that were
wrong and §33 is the answer**: run84 and run80 both carry `LEADERS=9` on
this map, the whole record is compared over run84's window now, and the
ledger is **exact** there — so the gap opens inside `[7183, 7585]`, with
the one purchase in between matched against run79's own queue record.
Inside the window the pile is
monotonic and unspent — 39 food at 7300 to 88 at 7580 with `income[0]` a
flat 1920 — so the gap did not open there. Two live seams sit upstream of
it:

- **`use_market` (§2.15).** ~~The gate is *true* for player 1 here~~ — it
  is **false**: the ability is Coinage and this crate read it off the
  market building (§40) — and its
  `need` on 7585 is `[55, 55, 0, 0, 0, 0]` against a timber purse of 14.
  ~~The narrowing this
  frame buys: **no draw in run53's whole 24,000-frame trace is made from
  `use_market`, `do_sell`, `do_buy`, `market_speculation` or
  `calc_market_prices`**, and the original's one roll lives in the sell
  branch, so that branch never runs in this game. Whatever the market does
  to this AI's pile it does through the draw-free **buy** branch alone,
  which is a smaller thing to land than the seam as written.~~ **Four of
  the five hold; `use_market` does not** — it draws on 34 frames of run53,
  the first on **8582**, and the sell branch is where Great Lakes' word
  sat until item 348. §40.
- **The caravans.** `vans.length` is 0 against 1 and `trade_val` 0 against
  128 on **both** of player 1's cities, on 246 of 246 blocks of run89 —
  and wealth is what a market buy spends.

The whole of the above but the last two bullets is asserted by
`run89_s_window_is_great_lakes_word_frame`, and every constant in it was
made to fail on purpose first.

---

## 33. The leader's ledger, compared at last (2026-09-07)

**What was missing.** `bucket` — the AI's stockpile — is written only at
`LEADERS=9`, and until item 290 the only part of that ~250-field record
compared over a *window* was six of its goods rows
(`great_lakes_goods_record_and_its_trade_routes_are_the_original_s`, §30.3)
and the whole of it on exactly one frame of one game
(`run9_s_frame_1_leader_record_is_the_census_after_the_sweep`). Great
Lakes' word at 7585 is a *value* in that record and nothing else (§32), so
the record is now mapped field for field —
`crates/rondata/src/diff/leader.rs`, whose `rows` is the whole of the
mapping and whose `UNMODELLED` names what this crate has no value for.

### 33.1 What the disk already knew, and it narrowed the question by 550 frames

Two Great Lakes archives carry `LEADERS=9`: run84 (`[6950, 7030)`) and
run80 (`[23960, 24000)`). Widening the whole record over run84's eighty
blocks, **both players**, says the ledger is **exact** there — `bucket`,
`leftover`, `resource_cap`, `over_cap`, `resources`, `income`, `escrow`,
`escrow_rate`, `econ`, `gather_slots`, `filled_gather_slots`,
`gather_slots_high`, `worst_good`, `shortages`, 160 blocks of them.

That leaves `[7030, 7585]`, and run79's own blocks close most of it. This
crate spends food exactly twice in that span: **sim-frame 7182**, where it
pays 60 food, 120 timber and 40 knowledge, and sim-frame 7582, which costs
no food. run79 dumps `BUILDS=7` over `[6910, 7250)`, so the original's
side of 7182 is on disk: block 7183 gains building **2018**
(`orig_type 428`, a Stable) and appends a third entry to Barracks
`2016`'s queue reading `cost[0] 60`, `good[0] 0` and `cost[1] 40`,
`good[1] 4` — sixty food and forty knowledge, the same purchase. Nothing
else in the original's queues moves.

So the ledger agreed at 7029 and the one purchase between there and the
word was matched, which left `[7183, 7585]` as the only place a shortfall
could open and no capture covering it. run91 is that capture
(`tools/gamelog/captures.txt`), and the grep is why its window is 86
blocks rather than 246. **It found no shortfall at all** — §34.

### 33.2 `get_mod_resource_cap` is not the ledger's cap

`LeaderData::get_mod_resource_cap@006d65b0` is what **nine** of the
production AI's call sites read where the commerce cap is wanted, and it is
not `LeaderDataEncrypt::resource_cap`. Read whole, twelve lines:

- `starting_resources == 8` (unlimited) → **0**.
- Otherwise `cap = resource_cap[g] ^ 0x1281`, then: with
  `semaphore[0] & 4` **or** `leader_flags & 4` (a human) → `cap`
  unchanged; else `get_diff() == 0` → `cap × 0.5f`, `get_diff() == 1` →
  `cap × 0.75f`, and everything above → `cap × 1.0f`.

So on the two easy difficulties a *computer* leader reads a cap smaller
than the one it holds, and a human always reads its own. The multiplier
is an `f32` truncated to `int`; 0.5 and 0.75 are exact in binary and the
caps are small, so [`sim::Sim::mod_resource_cap`]'s `cap / 2` and
`cap * 3 / 4` are the same arithmetic rather than an approximation
(`docs/DECISIONS.md` entry 16).

The nine sites: step 3 and step 4 of §2.5 (six calls between them),
`create_buildings`' three — the `income[g] < cap` gate on a gather
building's offer, and `rate[worst_good] < cap/32`'s escrow flag —
`create_units`' two, and `research_techs`' one (`goods_near_cap`). Every
one of them read the raw cap in this crate until item 290. §33.3 has what that was worth.

### 33.3 What the widening was worth

Three fields of run84's window closed on the cap alone: `rate[0]` and
`rate[1]` — 120 and 120 here against the original's 62 and 62, which is
`min(cap/2, income)/16` against `min(cap, income)/16` — and `best_good`,
which is downstream of them. With food and timber both reading 62 the
`>` in the rate pass keeps the **first** maximum and the original's
`best_good` is food; with 120 against 125 this crate's was timber, and
`create_buildings` halves the offer of a gather building for the best
good. `econ`, `worst_good` and `shortages` are unchanged by the fix,
which is what makes it safe: the thresholds land in the same places.

**The word did not move.** Great Lakes stays at **7585** and East Indies
at **7806** with the fix in. It is a record-field landing, not a score
one, and it is booked as such.

**The third scoreboard line moved a long way, though**, and mostly the
right way (DECISIONS 36 asks for the number rather than a trade):
**Great Lakes' endpoint `off` 84 → 73**, the largest single fall either
endpoint has had, with eight of its roster unlinked in exchange; East
Indies' 79 → 80 with **all eleven of its extras gone** and four building
field-rows closer; and both ladder rungs lost extras too — 22 → 13 and
20 → 9. Four rows, one change, 16,000 frames past a word that did not
move: the AI values differently everywhere the cap is read, and what that
buys is a smaller *spurious* roster on every East Indies row.

### 33.4 The residue, pinned by name

Seventy-one `(player, field)` pairs still part over run84's window and
`run84_s_window_is_the_original_s_whole_leader_record` names every one of
them (`PARTS_ON_RUN84`), so a field that leaves the list is a fix and one
that joins it is a regression. Three families:

- **Player 0's census is empty here.** `active`, `peasants`, `gatherers`,
  `filled_gather_slots`, `peasant_high`, `scouts`, `ally_mask` and the
  three team-territory counters all read zero for the human, because
  §2.3's sweep runs only for a computer leader in this crate and the
  original runs it for both. Ten fields, one cause, and none of them is
  read by anything the human does — which is why it has gone unseen.
- **The ten sites are a different list** (§2.7's known seam), and
  `SITE.reg` is a whole-list row of its own: the original writes `0` in
  a site's region where this crate writes the site's own.
- **Named unmodelled state**: `tech_frame` and `tech_cat_frame[0..3]`
  (written only by `Leader::init` here), `other_team_terr` and
  `min_other_team_terr`, the `attack`/`defense`/`scouts`/`active` unit
  classes, and `gather_stamp` — a cadence rather than a value, and whose
  outputs (`resources`, `income`, `rate`) all agree.

### 33.5 Coverage

**Diff-backed**: everything above. 36,800 field-frames over 160 blocks of
run84, both players; the 7182 purchase off run79's own queue record; the
two words measured before and after.

**Established by reading**: `get_mod_resource_cap`'s twelve lines, read
whole at `006d65b0`, and its nine call sites counted in the export
(`create_buildings` 3, `create_units` 2, `production_ai_setup` 6,
`research_techs` 1 — the six inside §2.5 are the rate pass's one and the
threshold pass's five). The human bypass is `leader_flags & 4`, which
`docs/ARMY.md`, `docs/CITIES.md` and `docs/ECONOMY.md` already read as
"human"; run84's own record has player 0 at `leader_flags 7` and player 1
at `524307`, so the bit is set for exactly one of them.

**Not established.**

- **The `semaphore[0] & 4` arm** of `get_mod_resource_cap` and of
  `get_diff` — a multiplayer path with no capture, so the crate takes the
  lobby's difficulty unconditionally.
- **Where the shortfall comes from.** Bounded to `[7183, 7585]` and
  otherwise open. Two seams stand in it and neither is implemented:
  `use_market`'s draw-free buy branch and `market_speculation`'s buy loop
  (§2.15), which on this AI's stock levels passes every one of its four
  bucket tests. Both spend wealth, which is why run91's window carries
  wealth beside food.
- **Player 0's sweep.** Running it would change nothing the human does
  but would close ten of the residue's rows; whether the original's human
  census feeds anything an AI reads is unread.

---

## 34. Great Lakes' word at 7585 is the make-list head (2026-09-07)

**Item 287 read it as the AI's stockpile and run91 refused that.** The
reasoning was sound and the conclusion was wrong: every gate of
`make_stuff` step 6 agreed, the good loop over food came out
`88 < 55 + 43 + 4`, and giving this crate **98** food made 7585 agree nine
draws for nine — so the original's food was inferred at `98 ≤ food < 160`.
run91 is the first `LEADERS=9` capture this map has ever had near the
word, and it says the original's food on block 7585 is **88**, this
crate's own number, with the two ladders identical **tick for tick over
all 72 blocks** of `[7514, 7585]`. §32's bounds are struck.

**What is actually wrong is the head of the make list.** run91's block
7585, leader 1, slot for slot:

| slot | this crate | the original |
| --- | --- | --- |
| 0 | `t 82` Slingers, `val 9999999`, cat 6, `num 3` | `t 437` **Temple**, `val 2499999`, cat 8, `num 1` |
| 1 | `t 132`, `val 2812500`, cat 6 | `t 419`, `val 810000`, cat 4 |
| 2 | `t 437` Temple, `val 2499999`, cat 8 | `t 420`, `val 506250`, cat 4 |
| 5 | `t 50` Citizen, `val 697`, cat 5 | the same, exactly |
| 6 | `t 82` Slingers, `val 9999999`, `num 3` | `t 82` Slingers, **`val 360000`**, `num 1` |
| 7 | `t 133`, `val 31488` | `t 133`, `val 62976` — exactly twice |
| 8 | `t 437` Temple, `val 2499999` | the same, exactly |

**And the head is what the good loop reads.** `make_stuff@006c8af0:172`
tests a good only when the **head's** cost in it is non-zero *and* the
slot's own is: `type_avail(g) && head_cost[g] != 0 && slot_cost[g] != 0`,
and only then `bucket[g] < head_cost[g] + slot_cost[g] + need`. A Temple
costs no food. So on the original's list **food is never put on trial**,
the Citizen at slot 5 passes, `make_this(5)` runs and the slot expiry
spends the `Leader::make_stuff+0x63d` this crate does not. `need` is the
same shape one step up — the average of `head_cost[g] - bucket[g]` over
the goods **the head costs** — so with a Temple head it is not the 4 that
287 computed either.

The dump carries the receipt on the next block: slot 5's `val` goes
**697 → 6**, which is §2.6's `val /= 100` on a buy, and `bucket[0]` goes
**88 → 45**, which is the Citizen's 43.

### 34.1 Where the head came from: an overflow guard

`9999999` is not a valuation. It is `create_units`' tail —
`out = fac × (want × val / divisor) / 256`, and then **`if out < 0 { out =
9999999 }`**, the guard the original puts on a wrapped 32-bit product
(`crates/sim/src/ai_units.rs`, the `wm` note). The original's answer for
the same Slingers on the same frame is **360,000**, so its product did not
wrap. This crate's did, the guard turned a negative into the largest value
in the list, and the Slingers took rank 0 from the Temple.

`num` says the same thing one field over: `batch_size` answers **3** here
against the original's **1**.

~~So the word is `create_units`' value chain for a land military type~~ —
and it is not. The chain is right; **the census under it was not**, twice
over, and with `combat`/`non_siege` reading 3 rather than 1 the land
branch's `base × 100` arm does not run and the same code produces
`360000` at `num 1`. §35 (item 295) has both faults, and closes the first
two of the three oracles below. Slot 7's factor of two is still open and
is `upgrade_units`' `owned`, not `create_units`'.

### 34.2 Coverage

**Diff-backed**, `run91_s_window_is_the_leader_s_ledger_at_the_word`:
58,480 field-frames over 172 blocks, both players; the food ladder
asserted equal on all 72 blocks at or below the word; the 43 the original
pays on 7586; the head's `t`, `cat`, `num` and `val` on the word's own
frame; and the whole residue pinned by name (`PARTS_ON_RUN91`, 114 rows,
35 of them make-list rows — **99 and twenty since item 295**, and the
whole of `bucket`, `attack`, `combat` and `non_siege` gone from it).

**The capture itself**: run91, `[7514, 7600)`, 170,485,873 bytes, 87
blocks. `rngcmp` against run53 **7,616 identical, 0 differing**; the run89
overlap **86 in common (7514..7599), 0 differing** under one
`--exclude LEADERDATA`, which is the whole window contained in its
neighbour; both teeth **86 (7514..7599)**. Its stanza wrote every figure
above down as a prediction *before* it ran, and named `bucket[0]` as the
one expected to part. It did not.

**Not established.**

- ~~**Which factor of `create_units`' chain wraps.**~~ §35: the chain was
  never wrong; the census under it was. Slot 7's own factor of two is
  §36.
- **The other make-list rows.** Slot 1 is a different type entirely
  (`132` against `419`), ~~slot 7's value is exactly half~~ (§36), and
  slots 9 and 10 carry types one below the original's (`565`/`566`,
  `572`/`573`) on every block of both windows — an off-by-one in a type
  index that predates this item and now has a name.
- **Why this crate's Temple sits at slot 2 and the original's at slot 8
  only.** Both lists hold the Temple at `val 2499999`; the ranks differ
  because the head differs, and whether anything else moved with it is
  unread.


---

## 35. The census counted the wrong word and the wrong objects (2026-09-17)

**Item 295 was booked as `create_units`' value chain and the chain was
never wrong.** The overflow §34.1 names is real — `out = -7777216` on
sim-frame 7583, guarded to `9999999` — but every factor of it is this
crate's own arithmetic applied to a **census that had two independent
faults**, and with the census right the same code produces the original's
`val 360000, num 1` on the first run.

Both faults are in `Sim::census_units` (`crates/sim/src/ai_census.rs`),
§2.3's step 10, and the widening found them without a reading: run91
already prints the whole leader record, and `combat`/`non_siege` part at
**ours 1, theirs 3** on every one of its 86 blocks.

### 35.1 `roles` was this crate's own bitfield, not `UnitTypeData::role`

Step 10 tests `role & 0x10000` for "military". The sweep read that bit off
[`sim::combat::Profile::roles`] — a **different word with a different
layout**, the hand-rolled one `docs/DECISIONS.md` entry 18 introduced so a
rule can ask "is this a catapult" without a type id. The two collide
twice:

| bit | `UnitTypeData::role` (`ai_load::role`) | `combat::Profile::roles` |
| --- | --- | --- |
| `0x10` | `SCOUT` — the invader-count exemption | `V2ROCKET` |
| `0x1_0000` | `MILITARY` — `combat_role` | `CARAVAN` |

So on run91's block 7514 the sweep read leader 1's roster as one soldier —
its **Caravan** (`ti 59`, `Profile::roles 0x10000`) — and every Hoplite
(`ti 132`) and Longbowman (`ti 177`) as a civilian, because their profile
word is `0x80` (`BARRACKS_MADE`) and carries no `1 << 16`. `attack` came
out **0** for the same reason: a caravan's attack is zero, and it was the
only thing counted. The original's own word for those types is `0x50803`
and `0x150c00`, both with `0x10000` set — this crate's loader derives it
correctly and puts it on `UnitType::cols.role`; nothing read it.

The fix is [`sim::Sim::role_word_of_rec`], the record-keyed sibling of the
`role_word` `create_units` already uses — `UnitTypeData::role` for a type
in the tree, the column itself for one stood up by hand.

### 35.2 The sweep is over **captains**, and it was over every object

§2.3 step 10 opens "every captain of mine (`is_captain`, vslot `+0xe8`)".
The loop had no such test, so every figure of a squad was counted — the
same error `docs/ARMY.md` §3.3 records against `release_mustering`, in a
second place.

run91's block 7514 settles it to the unit. Leader 1 owns **40** objects:
23 Citizens, 3 Merchants, a Caravan, a Scout, **nine Longbowmen in three
squads** and **three Hoplites in one**. `is_captain` is `o_up < 0`
(`docs/ARMY.md`), and the record's own `o_up` chain makes **32** of them
captains. The original's `active` on that block is **32**.

### 35.3 What the two fixes are worth

With both in, on run91's window:

- `combat`, `non_siege` and `attack` **leave the residue entirely** —
  they parted on all 86 blocks before.
- **The make list's head is the Temple**, `t 437 val 2499999 cat 8
  num 1`, where it was three Slingers at the overflow guard.
- **Slot 6 is `t 82 val 360000 num 1`, the original's answer exactly** —
  both of §34's first two oracles, closed by the same change. The chain
  that reaches it is unaltered: `land_army` is 3 rather than 1, so
  `land_army < mil_level * 3` (3 < 3) is **false**, the `base × 100` /
  `escrow = 1` arm of the land-military branch never runs, and the product
  that used to wrap — `256 × (30 × 36,000,000 / 30)` — is a hundredth of
  itself and does not.
- **The food ladder agrees on all 86 blocks, the buy included**: `88 → 45`
  on block 7586, the Citizen out of slot 5 at 43 food, where this crate
  bought nothing. §34's "the original pays 43 and we pay nothing" is
  struck.
- **Great Lakes' long-capture word moves 7585 → 7679**, and the new word
  is `Guy::set_anim+0x97a < Unit::move_step+0x823` against the original's
  `Guy::set_anim+0x97a < Guy::inc_time+0x271` at index 1 — four draws
  against three.

### 35.4 Coverage

**Diff-backed**: everything in §35.3, plus the classification itself —
`run91_s_window_is_the_leader_s_ledger_at_the_word` compares the whole
record over 172 blocks and both players, and
`the_census_counts_captains_under_the_original_s_role_word` pins the two
faults directly on run91's own frame. The **32** of §35.2 is the
original's `active`, re-derived from its `o_up` chain.

**Established by reading**: nothing new. Both faults are the documents'
own words (§2.3 step 10, `ai_load::role`) against the code.

**Not established.**

- **`active` is 31 here against the original's 32**, on 62 of 86 blocks —
  one captain short, and which one is unread. It is the last whole-roster
  counter still parting.
- ~~**`MAKE[7].val` — `t 133` Phalanx — is still exactly half**, 31488
  against 62976~~ — **closed by item 302, and not where this section
  looked.** `Muster::by_type` is right: the original's `num_units` is
  over captains too, and its own array says so (§36.2). The halving was
  `upgrade_units` dividing by a **recount** of its villages where the
  original adds `LeaderData::village_num`, which is 0 (§36.3); the
  remainder of the row was `age_p`'s default (§36.4).
- **The other make-list rows** §34.2 names (`MAKE[9]`/`MAKE[10]`'s
  off-by-one type index, the `city` column) are untouched by this item.


---

## 36. The muster was right; the denominator was not (2026-09-17)

**Item 302 was booked as `Muster::by_type` counting squad heads where
`upgrade_units`' `owned` wants units, and the original counts heads too.**
§35 left the question open — is the original's per-type count over units
or over captains — and the answer is on disk rather than in the
decompile: `LEADERDATA` prints `num_units` and `num_queued` **whole**, and
nothing here had ever compared either. Widening the record answered the
item in twenty minutes and then named the real cause, which is in neither
`by_type` nor the predecessor chain.

### 36.1 The two arrays, and they are keyed differently

`num_units` is **352** entries and its index 0 is `BASE_UNITTYPES`, so
this crate's record index *is* the dump's index. `num_queued` is **806**
and its index 0 is `TypeIndex` 0, so the same Hoplites sit at 82 in one
array and 132 in the other. Reading both as record-keyed reported every
queued type as a divergence at two indices at once, which is how the
offset was found; both are re-keyed to the record in
[`rondata::diff::leader::rows`].

The 352 is not arbitrary. It is `BASE_GAIATYPES - BASE_UNITTYPES`: the
array stops before the twelve gaia types, which is why this crate's unit
table is 364 records and the muster rows are cut at 352.

### 36.2 Over captains, settled by the original's own array

run91's block 7585, leader 1, every nonzero entry:

| record | `TypeIndex` | type | `num_units` | objects owned |
| --- | --- | --- | --- | --- |
| 0 | 50 | Citizen | 23 | 23, each its own squad |
| 9 | 59 | Caravan | 1 | 1 |
| 11 | 61 | Merchant | 3 | 3 |
| 19 | 69 | Scout | 1 | 1 |
| 82 | 132 | Hoplites | **1** | **three figures in one squad** |
| 127 | 177 | Longbowmen | **3** | **nine figures in three squads** |

The six sum to **32**, which is the block's own `active`. So the
original's per-type count is over **captains**, exactly as this crate's
`Muster::by_type` is, and the item's title is refuted by the original
rather than by a reading. The guard has teeth: incrementing `by_type` for
every figure — the item's own hypothesis — parts `num_units[82]`,
`num_units[120]` and `num_units[127]`, and nothing else.

Both arrays now agree on every type, every block and both players, over
run91's window *and* run84's — **233,728** field-frames the record was
printing and no test was reading.

### 36.3 What the factor of two was: `village_num`, recounted

`upgrade_units@006c6430` opens its value with

```
iVar5 = city_num + village_num;  local_20 = 1;  if (1 < iVar5) local_20 = iVar5;
local_20 = (pop * 1000) / local_20;
```

— `LeaderData+0x3f8` plus `+0x3fc`, the two fields the record prints
side by side. This crate divided by `city_num` plus a **recount** of
which of its cities are still villages, and on Great Lakes' leader 1 that
answers **2** where the original's `village_num` is **0** on all 86
blocks of the window. `base` was 500 against 1000 and **every value in
the make list halved** — visible only at slot 7, because it is the one
row whose other factors leave the halving unclamped and unshared.

`village_num` is a declared seam here (`ai_census.rs`, the seam table),
its census field answers 0, the original's answers 0, and every other
producer already reads it through [`sim::Sim::village_num`]. Only this
one recounted.

With the denominator right, `MAKE[7].val` on run91's block 7585 is
**62,976** against the original's 62,976 — §34's third oracle, and an
exact factor of two is a strong oracle precisely because almost no wrong
change reproduces it.

### 36.4 And the rest of the row: `age_p` is an accumulator, not a default

That left 63 of the window's 86 blocks still parting, and they are not
Phalanx at all: slot 7 carries **`t 66` Militia** on 7514–7576, `-1` on
7577–7579 and `t 133` Phalanx from 7580. Militia's own value came out
**7,936** against 41,856, and the solved difference is `gap`: ours 0,
the original's 1.

`gap` is `age_t - age_p`, and the listing settles what `age_p` is.
`6c660d xor edi,edi` zeroes the accumulator between the chain's head and
the walk, and the walk's guard is `6c6643 test edi,edi / jne` — the
accumulator itself, not a "have I found one" flag. Two consequences this
crate had wrong, both from defaulting `age_p` to the type's own age:

- A type with **no available predecessor** gets `gap = age_t - 0`, its
  own age. Militia is age 1, so its gap is 1; defaulting to `age_t`
  gives 0, and `m = 5 * gap + 2` and the later `(gap + 2) / 2` both
  collapse with it.
- A predecessor whose own age is **0** leaves the walk still looking,
  where a boolean flag stops it.

`avail` (`local_20`, `6c663c`) is a separate flag and is raised by
*every* available predecessor, which this crate had right.

With both fixes `MAKE[7].val` leaves run91's residue entirely — all 86
blocks, Militia's rows and Phalanx's alike.

### 36.5 What it moved, and what it did not

- **`MAKE[7].val` is gone from the residue**, 99 rows to 98, and it is
  the only row that left. Nothing joined.
- **The headline did not move.** Great Lakes' long-capture word is
  **7679** either side of this item, the same `Guy::set_anim+0x97a <
  Unit::move_step+0x823` against `Guy::set_anim+0x97a <
  Guy::inc_time+0x271` at index 1 — a figure's draw, not the AI's. The
  AI's value chain is not what that frame is about, and this item says
  so with a number rather than a hope.
- The endpoints moved and were re-pinned; §36.6 has the figures.

### 36.6 Coverage

**Diff-backed**, `run91_s_window_is_the_leader_s_ledger_at_the_word` and
`run84_s_window_is_the_original_s_whole_leader_record`: the whole
per-type muster, both arrays, 352 types over 332 blocks and two windows;
`MAKE[7].t` and `MAKE[7].val` on the word's own frame, asserted at
`(133, 133)` and `(62_976, 62_976)`. The record's compared field-frames
go 58,308 → 179,396 on run91 and 54,240 → 166,880 on run84. Both new
claims were made to fail on purpose before they landed — the muster one
by counting figures, the slot-7 one by restoring the village recount.

**Established by reading**: §36.4's accumulator, and it is the listing
rather than the decompiler — `6c660d`, `6c6643`, `6c663c` in the PE's own
bytes. The decompiled `iVar9 = 0` says the same thing; the listing is
what makes it evidence.

**Not established.**

- **The make list's other values.** Five `val` rows still part, and
  they are the building producer's — `MAKE[0]`–`MAKE[4]`, with
  `MAKE[2].cat` beside them — ours and theirs a factor of ten apart on
  two of them. A different producer and a different item.
- **`MAKE[9]`/`MAKE[10]`'s off-by-one type index** (`565`/`566`,
  `572`/`573`) is untouched, and the `city` column is still one high on
  every row that carries one. Both predate this item.
- ~~**`active` 31 against 32** on 62 of 86 blocks, unmoved.~~ Closed
  by item 303: it is not a census counter at all — §37.
- **Whether `age_p`'s zero-age quirk is ever reached.** No type in the
  shipped tree exercised the "a predecessor of age 0 leaves the walk
  looking" arm on either window, so that half of §36.4 is the listing's
  word and not a diff's.


## 37. `active` is the muster's cardinality, not a census counter (2026-09-17)

**Item 303 was booked as "one captain short, and which one is unread".**
It is not a captain question. §36.2 had already shown that this crate's
`Muster::by_type` agrees with the original's `num_units` on every type,
every block and both players across both windows, and that leader 1's six
nonzero entries sum to 32 — the block's own `active`. The title's reading
required a captain the sweep drops; the dump says no captain is dropped.

### 37.1 The dump answers it before any reading

`active` equals the sum of the record's own `num_units` on **every** block
of both windows — 332 of them, both players — not only on the frames a
sweep runs. That is one `grep` over a capture already on disk, and it is
now an assertion ([`rondata::diff::leader`]'s
`active_is_the_muster_summed`, called from both windows' loops).

run84 makes the point sharper than run91 can. Its window carries the one
frame in either capture where the roster *changes*: `num_units[127]` goes
1 → 2 at block 6994, and the original's `active` goes 29 → 30 **on that
same block**. A counter rebuilt by a two-hundred-frame sweep cannot do
that.

### 37.2 The writers, and the sweep is the smallest of them

Grepping `LeaderData+0x93c` — `active`, with `control` at `+0x940` beside
it — over the whole decompile gives five writers, and only one is the
census:

| function | what it does |
| --- | --- |
| `Unit::set_type@00612fa0:74` | `track_unit_type(old, −1)`, `control −= pop`, `active −= 1` |
| `Unit::set_type@00612fa0:284` | `track_unit_type(new, +1)`, `control += pop`, `active += 1` |
| `Objects::init_unit@0065e0c0:92,174` | the same three **undone**, when the new unit turns out to be a squad follower |
| `Unit::close@0060ee50:235` | the decrement on death |
| `Leader::plan_strategy@006b9620:127,508` | the sweep: zero, then one per counted unit |

The muster increment and `active` sit inside **one** guarded block in each
direction, under the same predicate: not a follower (`unit+0x68 & 1`),
and either `control_cost != 0` or `is(0x134)` or `is_gov_hero`. That
adjacency is why `active` tracks the muster exactly — it is not a
coincidence of these captures but the shape of the code.

So `active` is a live count that the sweep happens to re-derive, and this
crate modelled only the re-derivation. Its sweep runs every 200 frames
(6175, 6375, … 7575 on this game), so between sweeps `active` was a stale
snapshot: 31 from the sweep at 7375, against a roster that had gained its
Hoplite squad since. It caught up at 7576, which is why the divergence was
62 of 86 blocks rather than all of them — and why the shape looked like a
missing captain.

### 37.3 The change, and the fourth copy that was hiding

[`sim::Sim::track_unit_type`] is the original's own call shape: `by_type`,
`by_group`, `control` and `active` on one statement, `delta` either way.
Its three sim call sites — [`Sim::init_unit`]'s head, [`Sim::set_unit_type`]
both directions, and [`Sim::produce`] — were already moving the first
three; they now move the fourth too.

The fourth copy of those increments was **outside the sim**: the diff
harness's stand-up ([`rondata::diff::setup`]) has its own tally for a unit
read out of a start dump, and it is the only one that matters for a human
player, whose sweep never runs here (§33). Seeding `active` there is what
takes player 0's row from `ours 0 theirs 6` to agreement — the same fix,
one layer out, and it would have been missed by reading the sim alone.

### 37.4 What it moved

- **`active` leaves the residue on both windows and both players.**
  run91: `1/active` (62 frames, 31/32) and `0/active` (86 frames, 0/6)
  gone. run84: `1/active` (36 frames, 29/30) and `0/active` (80 frames,
  0/6) gone. Four rows, and nothing joined.
- **`control` joins the record and agrees everywhere.** `active`'s twin
  had never been compared, on any capture; it is `Muster::control` here
  and it is right on all 332 blocks.
- **The headline did not move.** Great Lakes' long-capture word is
  **7679** either side of this item, and both endpoints are unchanged —
  70 off / 10 unlinked on East Indies, 57 / 23 on Great Lakes. Nothing in
  the sim *reads* `census.active`: it is a state field the record prints
  and the AI does not consult, so the fix cannot move a draw. The item
  closes a counter, not a frame, and this is that said with a number.

### 37.5 Coverage

**Diff-backed**, `run91_s_window_is_the_leader_s_ledger_at_the_word` and
`run84_s_window_is_the_original_s_whole_leader_record`: `active` and
`control` on 332 blocks, both players; and §37.1's structural claim —
`active == Σ num_units` on the original's own side — asserted on the same
332. Compared field-frames go 179,396 → 179,568 on run91 and 166,880 →
167,040 on run84. All three claims were made to fail on purpose first:
the structural one by summing `num_queued` (6 against 0 on the first
block of each window), the `control` row by adding one (80 and 86 frames
apart on both players), and the fix by dropping `active` from
`track_unit_type` (which restores all four residue rows exactly).

**Established by reading**: only the predicate table in §37.2 — which of
the five writers exist, and the guard they share. The *consequence* of
that table is what the diff checks.

**Not established.**

- **No writer decrements the muster on death.** `by_type`, `by_group`,
  `control` and now `active` are moved by creation and by
  [`Sim::set_unit_type`], and by nothing else; the original's
  `Unit::close@0060ee50:235` has no counterpart here. No unit of players
  0 or 1 dies inside either window, so no diff reaches it — the gap
  predates this item and is `by_type`'s as much as `active`'s. It wants
  its own item, and the capture that would settle it is a window over a
  frame where the original's `num_units` **falls**.
- **The zero-pop predicate.** The original counts a `control_cost == 0`
  unit only when `is(0x134)` or `is_gov_hero`; this crate's muster seams
  apply no such test and its sweep skips every zero-pop unit outright.
  The two disagree on paper and no unit in either window is zero-pop, so
  nothing on disk separates them.
- **Why `combat` lags and `active` does not.** The original's `combat`
  moves only on a sweep (3 → 4 at block 7576, the same frame this crate
  sweeps) while `active` moves live. That this crate's sweep cadence is
  in phase with the original's is evidence from one coincidence, not a
  claim: no item has yet pinned the cadence itself.

## 38. The scholar's gate reads the raw cap, and the disk already held the frame (2026-09-17)

Item 323. Great Lakes' draw **sequence** parted at **8182** on one draw:
the original spent `Leader::make_stuff+0x63d` where this crate spent a
`Guy::set_anim` idle, and one extra `Unit::think_scout+0xaba` later in the
frame put the counts back level so the *word* did not notice. `+0x63d` is
step 6's expiry walk (§2.6) — it runs once per bought slot — so the
missing draw says the original bought a slot this crate did not.

### 38.1 The capture was already on disk

The item was booked to take one. It did not need to. **run19**
(`gamelog-run19-window-8174-8192.txt`, 2026-08-25, 32 MB, 19 blocks) holds
dump-blocks **8174–8191** of this map's own game: `rngcmp` against
run53 answers **0 differing, 8,201 identical**. A scan of every
`gamelog*.txt` in the archive for a block labelled 8182 or 8186 returns
**run19 and nothing else** — so it is not merely a capture that reaches
the frame, it is the only one.

It had been read once, for `make_stuff`'s head clause (§15.6), and
compared field for field nowhere. The comparison is now
`run19_s_window_is_the_leader_record_at_the_scholar`: 36 blocks, **37,584
field-frames**, the residue pinned by name like run84's and run91's.

**Two corrections to §15.6 fall out of reading it again.** Its
"wood 92 → 32, wealth 40 → 10, metal 159 → 19" names the wrong goods:
`resourcerules.xml` lists Food, Timber, **Wealth**, **Knowledge**, Metal,
Oil in that order, and the record's third and fourth per-good blocks are
Wealth and Knowledge. The 159 that falls to 19 is **knowledge**, and the
40 that falls to 10 is wealth. And the dumped `resource_cap` for
knowledge is 15,984 against 2,000 for every other good, which is what
makes the knowledge gate below worth having at all.

### 38.2 The gate: `bucket[knowledge] <= (resource_cap[food] / 16) * 3 / 2`

`Leader::create_units@006c40a0:1122–1127`, the scholar arm of the
civilian branch:

```
(int)(*(uint *)(*(int *)&leaders.list[who].field_0x6eb8 + 0xc) ^ 0x8221)
  <= (((int)(uVar11 + ((int)uVar11 >> 0x1f & 0xfU)) >> 4) * 3) / 2
       where uVar11 = *(uint *)(*(int *)&this->field_0x6eb8 + 0x30) ^ 0x1281
```

Both operands are fields of the encrypted goods block, and the **type
record** names them — `LeaderDataEncrypt` is a struct of arrays:

| offset | field | index |
| --- | --- | --- |
| `+0x0` | `int[6] bucket` | `+0xc` is `bucket[3]`, knowledge |
| `+0x18` | `int[6] leftover` | |
| `+0x30` | `int[7] resource_cap` | `+0x30` is `resource_cap[0]`, food |

So the gate is the knowledge stockpile against three halves of a
sixteenth of the **food** cap — an odd pairing, and it is what the bytes
say.

**The load-bearing half is that the cap is read raw.**
`LeaderData::get_mod_resource_cap` divides by two on Easiest and takes
three quarters on Easy (§2.5), and it is called **exactly twice in the
whole of `create_units`** — at `:1229` and `:1232`, both in the citizen
branch. Neither is this gate, which reads the field. This crate called
`Sim::mod_resource_cap` here, and on Great Lakes' Easiest AI that turned
a food cap of 2,000 into 1,000 and the threshold `(2000/16)*3/2 = 187`
into `(1000/16)*3/2 = 93` — against a stockpile of **159**. The original
passes; this crate refused, on that frame and every frame like it.

### 38.3 What it moved, by value

`create_units` runs at sim-frame **8180**. With the raw cap it offers the
Scholar (`0x34` = 52, `city 1`, `val 9,999,999`) into make-list slot 1,
exactly as the original's dump-block 8181 shows; `make_stuff` buys it at
8182 and step 6's walk spends `+0x63d` on it. The value diff beside the
word is the leader's stockpile, all six goods, on the dump's own blocks:

| block | dump | this crate before | this crate after |
| --- | --- | --- | --- |
| 8181 | 148 91 40 159 24 0 | 148 91 40 159 24 0 | 148 91 40 159 24 0 |
| 8182 | 148 92 40 159 24 0 | 148 92 40 159 24 0 | 148 92 40 159 24 0 |
| 8183 | 149 32 **10** 19 24 0 | 149 32 **40** 19 24 0 | 149 32 **10** 19 24 0 |
| 8184 | 149 32 11 19 24 0 | 149 32 41 19 24 0 | 149 32 11 19 24 0 |

The thirty wealth is the scholar. Great Lakes' floor — the lower of word
and sequence — goes **8182 → 8186**; the sequence moves to 8186 and the
word was already there. Frames 8182, 8183, 8184 and 8185 now agree entry
for entry.

### 38.4 The instrument was comparing two different numbering spaces

`diff::leader::rows` emitted `MAKE[i].t` as this crate's **tree id** and
`theirs` read the dump's **`TypeIndex`**. The tree is laid out gaplessly
(`crate::load`, "the type space"): a good, a unit and a building carry
the same number in both, and the tech block starts at tree id 543 against
`TypeIndex` `0x220` = 544. **Every tech offer therefore read as a
divergence** — Coinage at 558 against 559, Empire at 565 against 566,
Mercenaries 572 against 573, Mathematics 551 against 552 — and nine rows
across run84's and run91's pinned residue were that and not the
simulation. They are gone; `MAKE[3].t` on run84 is a real one and stays.

### 38.5 What this does not establish

- **The `city` field is still ours+1 on every offer.** This crate's city
  array has the human's at index 0 and the AI's at 1 and 2; the dump's
  `MAKE[i].city` for the same offers reads 0 and 1. Whether the original
  numbers cities per leader or in a different founding order is not
  settled here, and no offer in this window is chosen *by* the index, so
  nothing in the sequence turns on it yet. It is `MAKE[*].city` in
  `PARTS_ON_RUN19`.
- **Three tech `val`s part** — Empire 2,100,000 against 1,800,000,
  Mercenaries 165,000 against 216,000, Mathematics 82,500 against 63,000
  — from before the window opens. `research_techs` (§2.14) values them
  and nothing here re-derives its arithmetic.
- ~~**The Merchant is still not offered.**~~ **Offered since item 327
  (§55)**: `reg_known_rares` has its writer, and run19's `MAKE[3].t` and
  `MAKE[3].cat` agree. Only `MAKE[3].val` still parts, on 8174, at 77/75
  of the original's.
- **The other three `mod_resource_cap` call sites are unexamined here
  only in part.** `create_buildings@006c1be0` calls
  `get_mod_resource_cap` three times and reads the raw field nowhere, and
  `ai_build.rs` uses the modified cap three times — those agree.
  `production_ai_setup` calls it six times and `ai_drive.rs` uses it
  once, per good; that pairing is not checked.
- **8186 is combat, not production.** The successor is 54 draws against
  6, 46 of them `Unit::find_attack_pos+0xea9 < Unit::find_attack_pos+0x2d
  < Unit::fight+0xcb4` — a site `trace::SITES` does not name at all, which
  is why the harness prints it as `602129`. run19's own record already
  says the war state parts there: `wars`, `active_wars`,
  `active_wars_with`, `attacked_by` and `frame_attacked` are in
  `PARTS_ON_RUN19` for the human.

## 39. The mine's site test is not the camp's survey (2026-09-18)

`Leader::produce_building`'s spiral draws once per friendless FARM/MINE
candidate that **passes every site test** (§2.20), and one of those tests is
`blocked_site`'s gather verdict. For a MINE that verdict is not the camp's
cell survey: `calc_gather` sends `0x1a3` past the circle walk before it
starts and asks instead whether a **mountain tile** stands within
`MINE_RADIUS × 0xc0` world units of the footprint's centre
(`MountainsData::find_nearest@0089cd30`).

Measuring to a mountain-centred **cell** inside `MINE_RADIUS` tiles, which is
what this crate did until 2026-09-18, passes twice as many sites. Great
Lakes' frame 8382 places the game's first mine and is the measurement:
**eighteen** `+0xc99` draws against the original's **nine**, the nine whose
nearest mountain tile is inside 1152 world units. Nine extra draws moved the
placement jitter's four rolls nine places down the stream and put the mine a
tile east of `1/2021`'s own position.

The same frame is 828 draws of `Build::find_gather_tiles+0x10a`, the mine's
own 207-tile mining list, which this crate spent none of. Both halves are
`docs/ECONOMY.md`, "The mine's range" — the arithmetic, the reconstruction,
the value diff against run97 and run80, and what it leaves open. The word
moved **8382 → 8404** on them.

---

## 40. The market's ability is Coinage, and the gate was a building (2026-09-18)

Great Lakes' word stood at 8582 with nothing named on the frame. The
widening named it in one line: the original spends eight draws there and
this crate seven, and the extra one is at `6c93ad` — an address the
trace's own table did not carry, which is
`Leader::use_market@006c91c0+0x1ed`, the single `Random::get(game_random,
0, 0xffff)` in the whole market.

**§32's narrowing was four-fifths right.** It reads: *no draw in run53's
whole 24,000-frame trace is made from `use_market`, `do_sell`, `do_buy`,
`market_speculation` or `calc_market_prices`*. The last four hold.
`use_market` does not — `report.py … when Leader::use_market 1 0` dates
**34** frames of it, the first two being **8582** and **8585**, then a
200-frame cadence (8782, 8982, 9182×3, 10182×4 …). The conclusion drawn
from it — that the sell branch never runs in this game — was the thing
the word was waiting on.

### The gate is a tech, and this crate read it off a building

`use_market`'s gate is `(has_tribe_bonus(4) || has_preq(BUY_SELL)) &&
has_market && !get_nuke_embargo`. `BUY_SELL` is `TypeIndex` **685** —
`BASE_BONUSTYPES` is 684, so it is the *second* entry of `rules.xml`'s
`<TECHBONUSES>`, "Can buy and sell resources at the Market" — and
`has_preq`'s generic arm walks that type's own prerequisites and asks
`has_tech` of each. It has one: **`<PREQ preq0="Coinage"/>`**.

Coinage is the Commerce line's **second** library tech (`techrules.xml`:
`WHERE Library`, `GRID_X 1`, `GRID_Y 2`, `AGE 1`; the line is Barter,
Coinage, Trade, Mercantilism, Finance, Assembly Line, Globalization). The
**Market building's** own `PREQ0` is *Barter* — Commerce **1**
(`buildingrules.xml`). So the two are a whole library tech apart, and
this crate's standing seam — *"owning an active market is taken to imply
it, which is true of the shipped tree (the Market's own prerequisite is
the tech)"* — was false on the one word that mattered. Its gate opened on
Great Lakes **6383**; the original's opens between 8382 and 8582, which
is where `epoch[Commerce]` reaches 2.

The old gate is not merely early, it is *visibly* early: this crate's own
shortfall vector already met the sell branch's condition on **8185** and
**8382** (`need [0, 0, 32, 0, 0, 0]` against `bucket[wealth]` 11 and 27),
and the original drew on neither. Two refusals below the word, and both
are the tech.

### The draw is price-free exactly where it matters

`docs/ECONOMY.md` has no market: no `calc_market_prices`, no `do_buy`, no
`do_sell`. That is why this crate took no draw at all — which branch a
short good takes is a price question. **Except for wealth.** The
condition reads

```c
if ((TVar3 == WEALTH) ||
   (calc_market_prices(this, TVar3, &price, &need6),
    (int)(bucket[WEALTH] - price) < need[WEALTH]))
```

and C's `||` short-circuits, so a wealth shortfall enters the sell branch
— and spends its draw — without a price ever being computed. Wealth
cannot be bought with wealth; the AI can only sell for it.

The rotation that draw seeds is price-free too. `rand % 6` picks where to
start, and a candidate qualifies on stock alone: not knowledge, not
wealth, not oil, not the good being covered, `type_avail`, `need[g2] ≤
bucket[g2] − 100`, and (`cap[g2] / 2 ≤ income[g2]` or `bucket[g2] > 199`).
**And the once-flag is cleared by a candidate passing that test, not by
the sale going through** (`local_c = 0; uVar5 = 0` sits after the inner
gate's `if`, not inside it). So "nothing here is sellable" is decidable
without a price, and it is exactly the case where the outer `while` spends
one draw and stops.

Both of Great Lakes' first two are that case:

| frame | `need` | `bucket` | sellable |
|---|---|---|---|
| 8582 | `[0, 0, 64, 0, 0, 0]` | `[58, 55, 43, 68, 54, 0]` | food 58, timber 55, metal 54 — none clears `need ≤ stock − 100` |
| 8585 | `[0, 0, 34, 0, 0, 0]` | `[14, 56, 12, 68, 54, 0]` | the same three, the same refusal |

One draw apiece, nothing traded, and the trace agrees on both.

**CORRECTION, item 358 (2026-09-18): one draw does not *mean* "nothing
traded".** It is also what a pass that sold looks like when the sale
carries the bucket over `need`, because the outer `while (bucket[g] <
need[g])` then exits before the once-flag is consulted at all. The table
above still holds for 8582 and 8585 — their candidates really do fail
`need ≤ stock − 100` — but **8982 does not**: timber stands at 141 against
a need of 0, it clears the bar, and a hundred of it leaves for eighty
wealth. The inference "the sell branch never trades in this game" was the
thing the word was waiting on a second time. `docs/ECONOMY.md` §12.

### ~~What is still a seam~~ — both closed by item 358

~~Two, and **both lose draws the original takes; neither invents one**:~~

- ~~A short good that is **not** wealth still takes nothing, because the
  buy/sell choice is `calc_market_prices`.~~ It is priced now, and a
  non-wealth shortfall is **bought** where wealth can stand the buy price
  — draw-free in the original too.
- ~~A wealth shortfall that *does* find something sellable stops after its
  first draw, where the original sells, raises `bucket[wealth]` by
  `do_sell`'s price and goes round again — another draw per pass. Run53
  has frames of three and four draws (9182, 10182) and this is where they
  must come from; below the word there are none.~~ The sale is landed
  (`docs/ECONOMY.md` §12.2) and so is the loop that goes round again
  (§12.3). 9182's three draws are **above** the word at 9134 and still
  unexplained: this crate spends one there.

What remains open is not the sell branch but the Supercollider clamp and
CtW's bonus, both listed in `docs/ECONOMY.md` §12.5 with the capture that
would refuse them.

The gate's other two terms are unchanged seams: `get_nuke_embargo` is
taken as zero, and `has_market@006d5410` is still read as "an active
market building of mine" rather than the original's `num_buildings[0x16]
!= 0` **and** a city of mine with `city_flags & 0x800`. run97 says the
substitution is harmless here — who 1's first city carries `0x800` on
every one of the window's 1,320 blocks, unchanged — so no capture on this
disk can separate the two, and the seam is stated rather than closed.

**The word moved 8582 → 8619**, where this crate spends an extra
`Guy::set_anim+0x97a < Unit::do_idle+0x7d` on unit `1/39` against five
`inc_time` wraps on both sides. East Indies did not move: 9711 either way.

## 41. The scholar the make list offers and the slot loop cannot buy (2026-09-18)

Item 358's residue, and it is one building's queue: `1/2020`, the AI's
University, from **8985**. The original holds three scholars there and
this crate two.

The offers are right. The head expiry (`make_stuff+0x221`) draws **three**
times on 8985 on both sides, which is one draw per make-list slot holding
the head's type, so all three `t52` offers are in the list — slots 0, 2
and 4, `create_units` having filled them on 8983. The head is bought, and
the wealth for a second is there: §12's timber sale leaves 92 and the two
scholars cost 36 and 38.

~~What stops the second is the **order of `make_stuff`'s own steps**.~~
~~Step 4 — the head's expiry walk — runs before step 6's slot loop and
clears every duplicate to `t = −1`; step 6 then reads `list[slot]` fresh
and skips a slot whose `t` is negative. So a duplicate of the head can
never be bought in the same pass, in this crate. In the original it
plainly can.~~

~~Three readings fit and no capture on this disk separates them: the
expiry is conditional on `unconditional` being false (`expire_all`), and
this crate's is true where the original's is not; or step 6 reads a copy
of the list taken before step 4; or the second purchase is not step 6's
at all.~~ **All three were wrong, and no capture was needed — §42.** The
third is nearest: the second purchase is not step 6's, and neither is the
third. **There is only one purchase**, and the quantity is the make list
slot's own `num`. The draw stream says so without a dump: 8985 costs
**three** `make_stuff+0x221` and **zero** `make_stuff+0x63d` on *both*
sides, and a step-6 purchase always draws at least once (its own slot
holds the type it just bought). The `LEADERS=9` window over `[8980, 8990]`
is no longer owed.

~~It costs no draw (the expiry count already agrees), so it is a **value**
residue~~ — and that is exactly what named it. `diff::tests::
run97_s_build_queues_are_the_original_s` now holds **one** row over
37,899 building-frames rather than two: `1/2020` is closed.

## 42. The batch a civilian is bought in, and the field that was dropped (2026-09-18)

Item 362. `MakeObject.num` is a **quantity**, and `Leader::make_this`
hands it to `produce_unit(t, city, num, escrow)`, which queues that many
at the one building its walk chose. `Leader::can_pay(slot)` is
`can_pay_cost(…) >= num` for the same reason: the slot is an order for
`num` of something, not for one.

**Where `num` comes from.** `create_units@006c40a0` keeps it in `TVar24`,
and *every arm sets it* before falling through to the shared tail at
`LAB_006c4cfc`. The tail clamps it twice — `local_14 = min(TVar24,
remaining)` against the population headroom, then against what
`check_income` says is affordable, but **only when that is non-zero** —
and passes the result to `MakeList::make_me` as the slot's `num`. The
arms, by name:

| arm | `TVar24` | line |
|---|---|---|
| scholar | `gfree[KNOWLEDGE] − count_queue(university, scholar)` | 1156 |
| caravan | 3, 2 or 1 by `econ[WEALTH]`'s bits, capped by `get_caravan_limit` | 1080–1100 |
| citizen | `max(0, min(deficit, room))` — the same number the value squares | 1215–1257 |
| merchant, scout, spy, the two military-scout arms | `1` | 973, 999, 1025 |
| land military | `batch_size(control_cost, …)` | 1478–1501 |

This crate had the military arm and the air arm right, because both take
`&mut num`. **The civilian arms computed the number and threw it away**:
`Sim::civilian_value` returned `(value, cat, want)` and its scholar,
caravan and citizen arms each held the count in a local that nothing
read. So every civilian the AI has ever offered itself went onto the make
list as a batch of one.

### What it cost, on Great Lakes

The AI's University, `1/2020`, from 8985. The original queues **three**
scholars on that frame and this crate queued one per production cycle —
two by 8985 after item 358 gave it the wealth, never three. The frame
spends three `make_stuff+0x221` draws and **no** `make_stuff+0x63d` on
both sides, which is the whole proof that it is one purchase: step 6's
expiry walk starts *at the slot it just bought*, so a step-6 purchase
cannot cost zero draws. With `num` carried, every one of the window's
37,899 building-frames agrees on `1/2020`, and the AI's wealth on 9182
is **38 lower** than it was — one more scholar paid for, at the price
§41's ladder gives the third.

**What moved.** `first_count` on the long capture **9182 → 9362**, and
run97's build-queue widening from two residue rows to one: `1/2020` is
gone, every one of the 37,899 building-frames agrees. `first_part` did
**not** move: 9182's ten draws are the original's ten in number and not
in order, and `LONG_WORD_GREAT_LAKES` is the lower of the two.

### What 9182 is now, measured — ~~and what it waits on~~ *(§43: not the step-6 purchases; they are downstream of `use_market`'s `need`)*

The frame is no longer a count. Either side spends ten draws:

| | this crate | the original |
|---|---|---|
| `Leader::use_market+0x1ed` | 1 | 3 |
| `Leader::make_stuff+0x221` | 2 | 2 |
| `Leader::make_stuff+0x63d` | 4 | 0 |
| `Guy::set_anim+0x97a` | 2 | 2 |
| `Guy::set_anim+0x104b` | 1 | 3 |

The head expiry now **agrees** — two slots hold the head's type on both
sides, where before the fix this crate had three. What is left is that
the original goes round the market twice more and buys nothing in step 6,
where this crate draws once and buys twice.

**Three market draws mean one of exactly two things**, and this is worth
writing down because §40 and §12.3 each had only one of them:
`use_market`'s outer `while (bucket[g] < need[g])` can only repeat for a
good when a candidate passed the sell test — and **selling raises wealth,
never the short good** — so a *non-wealth* shortfall keeps drawing until
the sellable set empties, one draw per hundred it can shed, while a
*wealth* shortfall stops as soon as the sale covers it. So three draws is
either **three short goods**, or one good with two sales under it.

This crate's ledger at 9182 admits neither: `need` is `[0, 0, 40, 0, 0,
0]` against `bucket [73, 84, 35, 111, 71, 0]`, so wealth is the only
short good, and the rotation's three eligible candidates — food 73,
timber 84, metal 71 — are all under the hundred `need[g2] ≤ stock − 100`
asks for. (Knowledge, at 111, is the one stock over it and is excluded by
name.) So the original's ledger is not this one, and 9182 is still a
**value** residue rather than a rule: the market's arithmetic is the
decompile's, line for line.

### What is not established

- ~~**Which value.** No `LEADERS=9` dump on this disk covers 9182 — run97
  and run100 carry `LEADERS=1`, which is `who`, `tribe`, `score` — so the
  original's `need` and `bucket` on the frame are not on disk.~~ Still
  true of 9182 itself, and **§43 names the value anyway**: `need` is the
  make list's first `epoch[Commerce]` slots' cost, and run19's own
  `LEADERS=9` window — on disk since August — holds six `1/MAKE[*].val`
  rows in its residue. A `LEADERS=9` window over Great Lakes
  `[9175, 9190]` would still print the frame's own list and buckets, and
  §43 says what each answer would decide.
- **The caravan arm's batch.** 3/2/1 by `econ[WEALTH]` is read from
  `create_units@006c40a0:1080–1100` and **no capture exercises it**: the
  frame that moved is the scholar's. The citizen arm's is exercised (this
  crate queues a citizen at 9182 that the original does not) but is not
  *confirmed* by it — that row is the residue, not the pin.
- **`Build::queue_up`'s escrow.** `queue_batch` ignores its `escrow`
  argument, unchanged by this item.

**The batch is not what the ladder's extras are** — item 370 measured them
and they are ordinary divergence past the word with a citizen-batch
component, not an overshoot, and the rosters are not nested from `1/48` up
(`docs/journal/2026-09-19-item-370.md`).

## 43. What 9182 actually waits on, and the leader every census skipped (2026-09-18)

Item 368, booked as "9182's step-6 purchases". **It is not the step-6
purchases**, and this is the third item on this frame whose named
mechanism was innocent (§40 the market's draws, §42 `MakeObject.num`,
now this). The step-6 buys are *downstream*: what parts first, at index 1
of ten, is `use_market`'s own draw count, and the market's arithmetic is
the decompile's — re-read line for line this item, and so is
`MakeList::make_me@006c9be0`, which had never been checked against the
export.

### The frame, re-measured

Unchanged from §42: ten draws either side, `word 9362 / sequence 9182`.
Entering 9182, with the make list named rather than numbered:

| slot | type | val | city | cat | num | cost |
|---|---|---|---|---|---|---|
| 0 | Scholar | 9,999,999 | 2 | 4 | 5 | `[0,0,40,0,0,0]` |
| 1 | *(deduped)* | 6,000,000 | 1 | 4 | 5 | — |
| 2 | Phalanx | 335,616 | −1 | 7 | 1 | `[65,0,0,0,45,0]` |
| 3 | Citizen | 234,782 | 2 | 5 | 1 | `[46,0,0,0,0,0]` |

`bucket` is `[73, 84, 35, 111, 71, 0]`. `epoch[Commerce]` is 2, so
`need` is slots 0 and 1 — and slot 1 is a `t −1`, so `need` is the
Scholar's `[0, 0, 40, 0, 0, 0]` and **wealth is the only short good**.
One pass, one draw, nothing sellable (food 73, timber 84, metal 71, all
under the hundred the sell test asks). The original spends **three**.

Everything after that follows from the same vector. `can_pay(head)`
fails (5 scholars at 40 wealth against 35), so the head is not bought and
the reserve `local_18` is 5; the step loop's gate examines **only the
goods the head costs**, so a Phalanx and a Citizen that cost no wealth
are never examined at all and are bought on `can_pay` alone — four
`make_stuff+0x63d` draws, two slots apiece. The original's zero means its
`can_pay` refused both, and with these costs that is one number: **food
under 46**.

So 9182 is one question — *what is the original's `need` and `bucket`* —
and both branches of it are ruled by the same vector:

- **three short goods** wants a `need` non-zero in three goods, which the
  Scholar's cost alone cannot be; or
- **one good with two sales under it** wants two stocks at a hundred or
  more, which this crate has in none.

### Slot 1 is a dedupe, and `make_me` is the decompile's

Slot 1 carries `val 6,000,000`, `city 1`, `cat 4` under a `t −1`: a
Scholar at the *other* city, outranked by the one at city 2 and zapped
where it stood. `make_me@006c9be0`'s head branch writes the new best over
slot 0 **without shifting the old head down** and then clears `t` on any
of slots 1–3 holding the same type; the runner-up branch shifts 3←2←1,
clears over `k..=3`, then writes. Read whole against the export this
item: the shift bound (`3 − k` iterations from slot 3), the dedupe bound
(`< 0x79`, so slots `k..=3`), the "same type already ranks higher" break,
and the category slot's own `list[cat].val < val` all agree. Nothing in
the make list's *structure* is wrong.

### The widening: run97's order stacks, and what they close

`docs/RUNS.md` run97 writes a `STACK<TYPE>` per unit with every live
order and its own fields, **36,483 `GATHERORDER` records among them**,
and nothing had ever compared one on this map — run97 had had its clocks
read (§ANIM 5), its positions and leading goal (item 350) and its build
queues (item 358). `run97_s_window_orders_are_the_original_s` now reads
them: **59,747 unit-frames and 31,448 gather orders below the word, and
not one gatherer disagrees** — every `tx`, `ty`, `wait`, `goto_build`,
`been_there` and `dist_mod` is the original's. Made to fail on purpose
(`+ 1` on the comparison's own `tx` puts a row on all 31,448, one
apiece), so the nought is a measurement.

That closes a family rather than a field: **no citizen on this map is
working a camp the original does not**, on any frame below the word, so
"a gatherer carrying a different good" is no longer available as the
explanation for a stockpile gap. The whole scoring residue is 48,698 rows
on **eight** units — `1/27`, `1/28`, `1/29`, `1/40`, `1/41`, `1/42` (the
probe's six, off position from 8442, `docs/ARMY.md` §3.4's successor),
`1/23` (a standing unit carrying an `Action` order the dump does not,
older than the window) and two rows on `1/33`. The *set* is asserted, not
only the count, because a number cannot say whether a ninth unit joined.

### The cause, named against a capture already on disk

§42 said the original's ledger at 9182 "is not on disk" and booked a
capture. That is true of 9182 — and it is **not** true of the machinery
that builds the vector. run19 is a `LEADERS=9` window over `[8174, 8192]`
and `run19_s_window_is_the_leader_record_at_the_scholar` has been
comparing the whole record for weeks. Its residue holds six
`1/MAKE[*].val` rows:

| type | cat | ours | run19 |
|---|---|---|---|
| Coinage | 2 → slot 4 | 3,510,000 | 2,700,000 |
| Empire | 1 → slot 9 | 2,100,000 | 1,800,000 |
| Mercenaries | 0 → slot 10 | 165,000 | 216,000 |
| Temple | — → slot 8 | 82,500 | 63,000 |
| *(slot 3)* | | 288,750 | 281,250 |

The types agree, the `num` agrees since §42 — the **values** do not, and
some are high while one is low, which no single factor on `base` can do.
`ai_research::weight_total` is where they part, and it reads five census
facts. Four of the five are in the same residue, at nought:

```
1/active_wars       ours 0 theirs 1
1/active_wars_with  ours 0 theirs 1
1/wars              ours 0 theirs 1
1/other_team_terr   ours 0 theirs 266
1/min_other_team_terr ours 0 theirs 266
```

`weight_total` adds `ai[5]` (twice over, under `min_other`) when the
leader is behind on territory, and takes `ai[0]/3` instead of
`ai[5] + ai[1]` when it is at war. Both terms are per-tech, which is
exactly why the five values are wrong by five different ratios.

**Why they are nought.** Three of this crate's census loops skip human
leaders — `census_territory`, `census_wars` and `census_strategy` all
carry `self.nation[i].human { continue }`, and §2.3 step 4 reads "every
other **computer** leader". The export says otherwise:
`plan_strategy@006b9620:159-166` gates the territory loop on
`leader_flags & 2`, `i != who` and `i >= 0` and **nothing else**, and
`:1511` and `:1557` gate the war and region loops the same way plus
`treaties[i] & 1`. Bit 1 is set for *every* leader any capture prints —
the human's `leader_flags 7` in run97's own header among them. On a
one-human-one-AI game, which is every capture this crate is diffed
against, the human is the only other leader there is, so all five facts
read nought.

`other_team_terr 266` is `0/my_team_terr 266`, the human's own.

### What this item changed, and what it did not

**`census_territory` counts the human now.** `1/other_team_terr` and
`1/min_other_team_terr` leave run19's residue — **92 fields to 90** —
and **nothing else in the record moves**, including all six
`1/MAKE[*].val`: for these four techs `ai[5]` or the territory predicate
does not pay. **The word did not move**, and that is the honest reading
of a one-third fix to a five-fact cause.

**The other two loops park rather than land.** *(§45, item 382: and the
reason is in the clause above — `treaties[i] & 1` is **not** always set.
The original's player 1 has it at 0 on blocks 6950 and 7514 and at 1 on
8174 and 9170, so putting the human back without a met bit turns
`active_wars` on 7,600 frames early. The word measures the cost: 9182 →
7182.)* Putting the human back
into `census_wars` and `census_strategy` makes all five facts agree — and
takes run19's residue from 90 fields to **147**, because `active_wars != 0`
reaches the danger word and the region-strategy words as well as
`weight_total`, and the make list then parts on `t`, `cat`, `city` and
`val` across almost every slot (`1/MAKE[3].t` ours 566 theirs −1, and so
on down). The facts are right and the consequences are not yet, which
means something downstream of `active_wars` is wrong in a way that the
nought was hiding. That is a successor with a number on it, not a
one-line change.

### What is not established

- ~~**9182's own ledger**, still.~~ **Closed by item 369** — run107 is
  that window, taken as `[9170, 9200)`, and it refuses **both** branches:
  the original's `bucket` at 9182 is this crate's own `73 84 35 111 71 0`
  and does not move across the frame. The head is what differs. §44.
- **Which of `weight_total`'s terms pays for which tech.** The four
  ratios (13/10, 7/6, 55/72, 55/42) are recorded here rather than solved;
  solving them wants the per-type `ai[]` weight vectors beside them.
- **Whether the human's own census must run.** The original's player 0
  carries a full census in every dump (`0/gatherers 5`, `0/peasants 5`)
  and this crate's is empty, so `census_strategy`'s `weaker` test — which
  reads the *other* leader's `attack` — cannot be right for a human
  opponent even with the gate fixed. That is part of the parked successor.
- **`Build::queue_up`'s escrow**, unchanged by this item.

## 44. The head at 9182 is a tech, and the goods were never the question (2026-09-18)

Item 369, the capture §43 booked. **run107** is a `LEADERS=9` window over
blocks 9170–9199 of this map's own game — 30 blocks, no gap, `rngcmp`
against run53 0 differing over 9,216 frames, and `samegame.py --exclude
LEADERDATA` against run97 (the same frames at `LEADERS=1`) 30 in common
and 0 differing. `docs/RUNS.md`, run107.

### Both branches are refused by one row

§43 named two branches the vector at 9182 could be on, and both were
about the **bucket**. The bucket agrees:

| block | 9181 | 9182 | 9183 | 9184 |
|---|---|---|---|---|
| theirs | `73 84 35 111 71 0` | `73 84 35 111 71 0` | `73 84 35 111 71 0` | `73 84 35 111 71 0` |
| ours | the same | the same | **`27` …** | `27` … |

The original's stockpile is this crate's entering the frame and **does
not move across it**; this crate spends 46 food on a Citizen there. So
the original's refusal is not "food under 46" and not three short goods
it happens to hold — `epoch_get(scan)` agrees too. What differs is the
**head**.

### The head is Mercenaries, and the list is rebuilt two frames before

Slot 0 and the cat-10 category slot, from the dump, over the window:

```
9170..9176   slot0  t −1  val 99999      slot10  t 573  val    43,200
9177..9178   slot0  t −1  val −1         slot10  t −1   val −1          (cleared)
9179..9199   slot0  t 573 val 9,999,999  slot10  t 573  val 9,999,999
```

573 is `MERCENARIES`. **This crate clears and rebuilds on the same two
frames** — `1/MAKE[0].t` parts on 19 of 30 blocks, 9181 onward, so 9177
through 9180 agree on the type — and on the rebuild it writes
**6,600,000** where the original writes 9,999,999, the "must have"
ceiling every scripted offer carries. Two frames later `create_units`
offers the Scholar at 9,999,999: against the original's ceiling it ties
and the incumbent holds, against this crate's 6,600,000 it wins. The
Scholar takes the head here and Mercenaries leaves the ranked four
entirely.

So the whole parting reduces to **one number on one frame**:
`1/MAKE[0].val` on blocks 9179 and 9180, **6,600,000 against
9,999,999** — two frames of residue, and every other row downstream of
them.

### And that is the three draws, measured

`Sim::type_price` for the two heads, read off this crate's own data
layer at 9182:

| type | price |
|---|---|
| Mercenaries (tree 572) | `[100, 0, 0, 0, 100, 0]` |
| Scholar (52) | `[0, 0, 40, 0, 0, 0]` |
| Citizen (50) | `[46, 0, 0, 0, 0, 0]` |
| Phalanx (133) | `[65, 0, 0, 0, 45, 0]` |

`use_market`'s `need` is the first `max(1, epoch[Commerce])` = 2 slots'
prices. With the original's list that is Mercenaries + Scholar =
`[100, 0, 40, 0, 100, 0]` against a bucket of `[73, 84, 35, 111, 71, 0]`:
**food, wealth and metal are all short**, knowledge is skipped, and the
good loop draws once per short good — **three draws**, which is exactly
what the original spends at index 1 of §42's ten-draw window. With this
crate's list `need` is the Scholar's 40 wealth alone: one short good,
**one draw**. §43's branch A was right about the shape and wrong about
the cause: the `need` spans three goods because the *head* is a tech,
not because the bucket differs.

The rest follows without another assumption. `can_pay(head)` for
Mercenaries asks 100 food and 100 metal against 73 and 71 and fails, so
the original saves and buys nothing — which is why its bucket is flat
across 9182. This crate's head is the Scholar at 40 wealth against 35,
which also fails; but its step loop then examines only the goods the
*head* costs, so a Phalanx and a Citizen costing no wealth are never
examined and are bought on `can_pay` alone — four `make_stuff+0x63d`
draws and the 46 food.

### The residue, and what else this window opened

109 fields over 60 blocks and 62,640 field-frames, pinned as
`PARTS_ON_RUN107`. Three families in it are new to this window and none
had ever been compared here:

- **The ten `SITE` slots of player 1**, whole — `wx`, `wy`, `val`,
  `dist`, `rank` — parting on 30 of 30 blocks with the *same ten sites
  in a different order*: this crate's `SITE[3]` is the original's
  `SITE[1]` (`val 143,725`, `dist 1250`, `wx 44`, `wy 30`), its `SITE[5]`
  the original's `SITE[2]`, and so on. It is a ranking, not a survey —
  `rank` itself is in the residue on every slot.
- **`1/tech_frame` and `1/tech_cat_frame[0..3]`**, at nought here
  against `8382`, `4976`, `8382`, `8182`, `6376` — five fields the
  original stamps when a tech lands and this crate never writes. They
  sit in `weight_total`'s own neighbourhood, which is the reason to name
  them; that they *pay* is not established.
- **Player 0's census**, as §43 predicted: `peasants`, `gatherers`,
  `peasant_high`, `scouts`, `filled_gather_slots`, `my_team_terr 266`,
  `wars`, `active_wars`, `ally_mask`. `1/other_team_terr` is **not**
  in the residue — item 368's fix holds a thousand frames further on.

### What is not established

- ~~**Why this crate's Mercenaries value is 6,600,000.**~~ **Closed by
  item 382, §45**: the ceiling is the tail's `income × val` wrapping 32
  bits and the `< 0 → 9,999,999` guard catching the sign, and what keeps
  this crate under the cliff is `weight_total` answering 110 where the
  original answers 144 — the `active_wars` term, whose real gate is the
  met bit and not the `human` skip. The falsifier below was run: removing
  the skip does **not** move the word, it costs 2,000 frames of it. The
  measured
  facts are the two numbers and that this crate's cat-10 slot was
  **33,000** against the original's **43,200** before the rebuild —
  55/72, the same ratio §43 recorded for Mercenaries at run19 a thousand
  frames earlier — and that 6,600,000 is exactly 200 × 33,000 while
  8,640,000 (200 × 43,200) is *not* the original's 9,999,999. So the
  ceiling is reached by some other route than the ranked multiply, and
  naming that route is the successor. The falsifier is cheap and local:
  anything that puts this crate's `MAKE[0].val` at 9,999,999 on block
  9179 should move the word, and run107 is the test that says so.
- **Whether `make_me`'s head test breaks ties the way this reading
  assumes.** It does not matter at these values — 6,600,000 loses to
  9,999,999 under either rule — but it would matter to a fix that only
  half-closes the gap.
- **The `SITE` ordering**, which is a whole mechanic (`docs/AI.md` §2.9)
  and now has a 30-block value diff waiting for it.
- **`1/gather_stamp`**, ours 9183 against 8752/9095, and
  `1/leftover[1:timber]`, both unchanged in kind from earlier windows.

## 45. The ceiling is an overflow, and the gate is the met bit (2026-09-18)

Item 382, the successor §44 booked. Its question was arithmetic: this
crate writes **6,600,000** into `1/MAKE[0].val` on blocks 9179–9180 where
the original writes **9,999,999**, and `200 × 43,200` is 8,640,000, so the
ceiling is not the ranked multiply saturating. Both halves of the
question are now measured, and the second one is not where §44 looked.

### The route: `income × val` wraps, and the guard catches the sign

`tech_value`'s tail is `val = check_income(…) × val / 256`, then the
original's own `val < 0 → 9,999,999` (§2.14, `docs/audit/2026-08-25-ai.md`
note 4: the product is a 32-bit `imul` and **wraps**). So the ceiling is
an **overflow**, not a saturation, and `income` decides where the cliff
is. On block 9178, read off this crate's own `check_income`, `income` is
`0x40` — a quarter — which puts the threshold at `val > 33,554,431`:

| | pre-income `val` | `× 0x40` | after |
|---|---|---|---|
| the original | 34,560,000 | 2,211,840,000 → **−2,083,127,296** | **9,999,999** |
| this crate | 26,400,000 | 1,689,600,000 | **6,600,000** |

`the_tech_ceiling_is_the_income_multiply_wrapping_not_a_saturation`
(`crates/sim/src/ai_research.rs`) pins both rows and the cliff either
side, and [`ai_research::income_scaled`] is that tail as one function so
the claim has something to assert against. Made to fail on purpose.

### The ×200 is real, is upstream, and is on both sides

Instrumented over the whole game, this crate's Mercenaries offer is
`33,000` from block 8578 to 8978 and `26,400,000` on 9178 — 200×, exactly
as §44 recorded. It is two factors, and neither is the parting: `base ×=
10` when knowledge is comfortably in hand, and the cat-0 arm's `val ×= 20`
when `muster.cap × 5 / 6 < effective_pop`, which is `41 < 42` on 9178 and
first true there. The original takes both too — its own slot 10 goes
43,200 → over the cliff on the same frame.

### The parting is `weight_total`, 110 against 144

Mercenaries' weights are `ai = [152, 16, 4, 0, 10, 0, 0, 0, 44, 0, 0]`.
Every other fact on the frame is shared (`pop 2`, `infra_mod 256`,
`cities 2`, `my_team_terr 568 > other 266` so the territory term pays
nothing either way), and the war term is the whole difference: at peace
`ai[5] + ai[1] = 16`, at war `ai[0] / 3 = 50`. That is **110 against
144** — 55/72, the ratio §43 recorded for four techs and §44 for this one
— and 26,400,000 × 144 / 110 is 34,560,000 exactly. One term decides
whether the offer clears the cliff.

### And the gate is not `human`; it is `treaties[i] & 1`

`weight_total` reads `active_wars`, which this crate leaves at nought
because `census_wars` and `census_strategy` skip human leaders. §43 read
that skip as a mistake, and removing it is **worse**: the long capture's
word goes **9182 → 7182**, measured with both loops and with the region
loop alone.

The dumps say why. Player 1's diplomacy over four Great Lakes windows:

| block | `diplos[0]` | `treaties[0]` | `wars` | `active_wars` |
|---|---|---|---|---|
| 6950 (run84) | 0 | **0** | 0 | 0 |
| 7514 (run91) | 0 | **0** | 0 | 0 |
| 8174 (run19) | 0 | **1** | 1 | 1 |
| 9170 (run107) | 0 | **1** | 1 | 1 |

`diplos[0]` is 0 — at war — on all four. **The diplomacy never moves.**
What moves is `treaties[0]`'s low bit, the met bit, on **block 7945** —
item 390 captured a fifth window, run115's `[7880, 8010)`, and closed
(7616, 8174] to that one block, which is this crate's own
(`docs/VISION.md` §6.3). `plan_strategy@006b9620:1511,1557` gates both
loops on
it and on `leader_flags & 2` and `i != who` and **nothing else** — no
`human` test exists in the original at all. So the human skip is a
stand-in for a bit this crate never sets, and on a one-human-one-AI
capture "never met the human" is 2,000 frames closer than "met everyone
from frame 1". It is folded into [`Sim::met`] now, where the seam
belongs, rather than repeated in two loops as a rule about humans.

**`diplos` and `treaties` are in the record comparison from this item**
— `1/treaties[0]` parts on run19 and run107, `0/treaties[1]` on run84 and
run91, and `diplos` parts nowhere, which is the table above as four
standing assertions. Made to fail on purpose (`met` to self only puts a
row on all four windows).

### What is not established

- ~~**When the bit flips, and from what.**~~ **Closed by item 385, §46.**
  `Leader::treaty_on@006e1190` sets both sides' bit 0; its only caller is
  `Leader::meet@006e1250`; `meet`'s only callers are
  `Wall::check_ever_seen@0063ce70` — off `world+0x15c`, the "ever seen by"
  mask, i.e. fog — and `Unit::process_attrition@005e11a0`, at **three**
  sites (this entry said two: the generic `get_attrition() != 0` arm was
  missed). **The attrition path never fires in this game**: instrumented
  over run53's 24,000 frames, no unit of either leader reaches a
  non-exempt attrition outcome, not once. So the original's flip is the
  fog path — and that path was already built here (`docs/VISION.md` §6.1),
  with `meet` the one thing left out of its tail.
- ~~**Whether `active_wars` alone is enough** once the bit is right.~~
  **It is, for this frame** (§46): with the bit set at 7944 the whole
  `active_wars`/`wars`/`active_wars_with` family leaves run19's and
  run107's residues and the make list follows. The region loop's building
  test and `census_strategy`'s `weaker` reading of the *other* leader's
  `attack` are still seams; they did not have to be right for 9182.
- ~~**`1/MAKE[10].val` and the other tech values**~~ — **all of them
  closed together**, as this entry said they would: §46.
- **`income`'s own `0x40`** is read off this crate and not diffed; the
  record does not print it. If it is wrong the cliff moves, and the two
  measured `val`s either side of it would both be wrong by the same
  factor — which the 55/72 check above does not catch.

## 46. A visibility model: the met bit is set by the fog (2026-09-18)

Item 385, the successor §45 booked. The chain was named end to end and
one link was missing: `treaties[i] & 1` had no writer here. It has one
now — `Wall::check_ever_seen`'s own tail, which this crate had already
built for a different reason and had left the `Leader::meet` call out of.
**Great Lakes' word moved 9182 → 9415**, the first move on this frame in
six items.

### The change is nine lines, and it was never an AI change

`docs/VISION.md` §6.2 has the mechanic. In this document's terms:

- `Sim::treaties` is `LeaderData::treaties`, `int[8]` per leader, of which
  bit 0 is modelled;
- `Sim::treaty_on` is `Leader::treaty_on@006e1190`, which ors into **both**
  sides;
- `Sim::meet` is `Leader::meet@006e1250` less its two `say_meet` calls;
- and the meet loop at the end of `Sim::check_ever_seen` is the original's
  four gates in order — `o != owner`, `leader_flags & 1` (`LEADER_VALID`,
  not the `& 2` §43's census loops take), the ally mask newly in
  `ever_seen`, and `treaties[owner][o] & 1 == 0` for the call itself.

`Sim::has_met` — §43's `human` skip, which §45 had already folded into one
seam — now reads the bit. **This crate's first contact on Great Lakes is
frame 7944**, and the original's is somewhere in (7616, 8174]: four
windows of the same game bracket it (§45's table), and 7944 is inside the
bracket. Nothing else in the change is new; the fog, the footprint scan
and `ever_seen` have been right since item 322.

### What it paid, field by field

The four leader-record windows, every one of them **down**, with nothing
arriving:

| window | block | residue |
|---|---|---|
| run84 | 6950 | 82 → **81** |
| run91 | 7514 | 92 → **91** |
| run19 | 8174 | 91 → **80** |
| run107 | 9170 | 110 → **95** |

Before the flip the only row is `0/treaties[1]`, which this crate had at 1
and the original at 0 — the seam's own value, and the reason §45 put it in
the comparison. After it, the whole family:
`1/active_wars`, `1/active_wars_with`, `1/wars`, `1/treaties[0]`, and
**every `MAKE[*].val` §43 tabulated** — run19's Coinage, Empire,
Mercenaries, Temple and slot 3, all six of them, and on run107 the head's
`cat`, `num`, `t` and `val` together with `MAKE[10].val`, `MAKE[8].val`,
`MAKE[1].t` and `MAKE[4].t`. The 55/72 that §43 recorded for four techs
and §44 and §45 for one was one term on one predicate, and all of them
closed on the same line.

`1/bucket[0:food]`, `1/num_queued[0]` and `1/gather_stamp` left run107
with them — the purchase that was downstream of the head.

### The value diff, on the frame that moved

9182's own coordinates, from run107's dump:

| | ours before | ours now | theirs |
|---|---|---|---|
| `1/MAKE[0].t` at 9182 | 52 (Scholar) | **573** | 573 (Mercenaries) |
| `1/bucket[0:food]` at 9183 | 27 | **73** | 73 |
| `1/2007`'s queue at 9182 | `[(50, 100)]` | **empty** | empty |

§44 predicted each of these three from the head alone and each is now the
original's. The third is `run97_s_build_queues_are_the_original_s`, whose
residue is now **empty over its whole window** — 1,319 blocks, 37,899
building-frames, not one queue row. Made to fail on purpose: putting the
`human` skip back into `Sim::has_met` restores `frame 9182: 1/2007 ours
[(50, 100)] theirs []` as the first row, exactly.

And 9182 is a `Leader::use_market` sell on **both** sides now: the long
capture's market list below the word reads `8582, 8585, 8782, 8982, 9182,
9382`, this crate's and the original's alike.

### Where it stops, and what the window cost

**9415**, and the count-word moved with it (9362 → 9415), so the two part
on the same frame again. The frame is five draws against two:

```text
ours    unit 1/29 · unit 1/29 · Guy::set_anim+0x104b · Guy::set_anim+0x104b · Farms::inc_time+0x1ae
theirs  Guy::set_anim+0x104b · Farms::inc_time+0x1ae
```

Two of ours are at a site `trace::SITES` does not name, attributed to
`1/29` — one of the eight units item 368 left in run97's order residue,
off its position since 8442 (`docs/ARMY.md` §3.4's successor). That is the
successor's first read.

**Two floors rose, and both are the window's rather than the
simulation's**, which is the thing to be careful about when a word moves:
233 frames that had never been compared came under it.

- `run97_s_window_clocks`: the walk-slot band, closed at zero by item 352,
  now holds **two rows** — `1/35`'s guy 0 on 9338 and 9339, `cur_anim` 8
  against 7, one slot of the walk category. The band over item 352's own
  `[8443, 9182)` is still empty, and the assertion names the two rows
  rather than counting them so that stays visible. The point-and-goal
  floor went 6 → **8** on the same unit.
- `ORDER_RESIDUE_RUN97`: 48,698 → **53,622**, and per block it *fell*,
  42.3 → 40.7. The eight-unit set did not move.

**And one of them was a capture artifact, not a floor at all.**
`run97_s_window_orders` looped to the word rather than to run97's window,
and run97's file ends with a **truncated** `BEGIN FRAME 9361` — the
click-free lane gives up by stopping the process. The moment the word rose
past 9349 that partial block arrived as 47 `Length` rows on **39** units,
player 0's among them, which reads exactly like a simulation that has come
apart. The set assertion is what caught it; a row count would not have.
The loop now takes the capture's own last complete block, as its sibling
always did. This is parked 373's guard, arriving as a bug rather than as a
guard.

The endpoint at 24,001 is **54 off, 4 unlinked, 9 build_diverged**
(`DECISIONS` 36) — and that is the **merged** figure, not this item's.
Measured alone against its own base this item read 66 → 49 off and 5 → 12
unlinked; item 384's melee reach, landing the same day, read 66 → 58 and
5 → 7 against the other base. Two independent improvements compose and
the row is what the merged code prints, which is the rule the 289/290
merge set. `great_lakes_scholars_sit_on_their_universities` changed shape
with it: the scholar sitting one university away from the original's own
coordinates carried an index that moved on every upstream item (`1/53`,
`1/52`, `1/56`, `1/55`, `1/58`), and on the merged tree **nobody carries
that vector at all** — so the assertion is now the vector over the whole
roster with no index in it, which stops costing a re-measurement a
session. It is not a claim the seating is fixed.

### What is not established

- **The attrition path to `meet`**, three sites in
  `Unit::process_attrition@005e11a0` — the war arm, the assassin arm and
  the generic `get_attrition() != 0` arm. (§45 said two; the generic one
  was missed.) Read, not wired, because no unit of either leader reaches a
  non-exempt attrition outcome in run53's 24,000 frames, so no capture on
  this disk could tell whether it is right. A map where an army campaigns
  abroad would.
- ~~**The exact flip frame.** 7944 is this crate's; the original's is only
  bracketed to (7616, 8174] because no capture dumps a leader between
  7600 and 8174. A `LEADERS≥2` window anywhere in that gap would pin it,
  and it is the cheapest capture left on this mechanic.~~ **Closed by
  item 390, `docs/VISION.md` §6.3**: run115 is that window and the
  original's block is **7945**, this crate's own, flipping on both
  leaders at once with `diplos` static across all 130 blocks. The record
  needs **`LEADERS≥3`**, not `≥2` — `LeaderData::log_data@006e5110:213`
  raises the detail to 3 immediately before the `diplos`/`treaties` loop,
  and that sentence is what decides whether a capture is owed.
- **The region loop's building test and `census_strategy`'s `weaker`**,
  which reads the *other* leader's `attack` and for a human reads a census
  this crate does not run (§43). They did not have to be right for 9182;
  whether they are right is untested either way.
- **`income`'s own `0x40`** — §45's, unchanged: read off this crate, not
  printed by any record.


---

## 47. What the AI buys at Great Lakes 9382, and the birth it costs 128 frames later (2026-09-19)

Item 408, and the first widening of the AI headline's own frame. **9510 had
never been compared.** run97's value window stops at 9349, run109's is
`[9420, 9480)`; the frame the long capture scores on had no compared record
behind it at all. run100 — `[9340, 10899]` of the same seeded Great Lakes
game, at run97's detail — is the capture that covers it, and
`diff::tests::run100_s_word_frame_is_the_original_s` now walks the whole
`compare` over its blocks from the first to the word's own.

### 47.1 The word is the sixth scholar

9510's two extra draws are `Guy::init_real+0x52` and `Guy::set_anim+0x97a <
Unit::go_inside+0x280` — the birth-and-seating signature `docs/CITIES.md`
§6.5.2 gave the map's **first** scholar on 8272. This is its **sixth**:
`1/51`, `guy` 52, standing in block 9511 on University `1/2019` at
`(40416, 25248)`, beside `1/44`. The five below the word seat on 8272, 8680,
9087, 9201 and 9322; four of them are on the *other* University, `1/2020`.

The word block says it with nothing beside it. Over 172 blocks the only
object either side holds alone is that one, on that one frame —
`unlinked [(1, 51)]`, `extra []` — so the frame is a missing unit and not a
swap, and no other record on it parts at all.

### 47.2 The cause is 9382, and it is a purchase

The only record that **newly** parts anywhere inside the window is the
production queue, and it parts on one frame, on two buildings at once:

| block 9383 | ours | theirs |
| --- | --- | --- |
| `1/2007` (Village) `queued` | **2** | 0 |
| `1/2019` (University) `queued` | 0 | **1** |

The original queues one Scholar at the University; this crate queues two
Citizens at the Village. Both are in the same city — Norwich, `1/2007`'s own
— and the dump's `CITY` record for it agrees with this crate field for field
on `free 0`, `busy 15`, `gatherers 15`, so the census is not what differs.

The original's job then runs to the birth, and `run100_s_scholar_job_runs_to
_the_word` reads it off the original alone: `1/2019` holds one live slot from
block 9383 to block 9510 — type 52 at **40 wealth** (`good[0] 2`, where the
four already seated at `1/2020` were bought at 38) — `job_counter` +100 a
frame from 100 to 12700 and then a **clamped 12750** rather than a
thirteenth hundred, and block 9511 is `queued 0` with `1/51` standing.

### 47.3 A purchase draws nothing, and the stream stayed blind for 128 frames

**This is a result in its own right, and it is the reusable half of the
item.** State it on its own terms: an AI purchase is invisible to the draw
stream, so the score can sit still on a decision that has already gone
wrong and keep sitting still for as long as the job it bought takes to
finish.

9382 costs **eight draws on both sides, entry for entry**: two
`Leader::use_market+0x1ed`, two `Leader::make_stuff+0x221`, two `+0x63d`,
`Animal::think_farm_animal+0x142` and `Farms::inc_time+0x1ae`. §41 and §42
say what the two make-stuff sites count — `+0x221` once per make-list slot
holding the *head's* type, `+0x63d` once per slot the loop expires. Neither
counts a purchase, and `Leader::make_this` → `produce_unit` → `queue_up`
spends no draw of its own.

So on 9382 two make lists **of the same shape** — same head type, same
number of slots repeating it, same number of expiries — were walked to two
different answers, and nothing in the draw stream said so. The word did not
notice for **128 frames**, and when it finally did, what it noticed was a
`Guy::init_real` in the animation layer: a birth, four subsystems away from
the decision that caused it. Every frame in between agreed draw for draw.

Two consequences worth carrying past this item:

- **A draw-stream word is a lower bound on when a decision parted, not an
  estimate of it.** The distance here is the length of a train job, and it
  scales with whatever was bought.
- **The queue record is the oracle for an AI purchase**, because it is the
  only place a drawless decision leaves a mark on the same frame it is
  made. `BUILDQUEUE` is written from `BUILDS=1`, so it is on nearly every
  capture already taken — this frame's evidence had been on the disk since
  run100 was captured and went unread because nothing compared it.

That is the case for widening the record rather than the frame, and it is
made here on the frame the headline itself stands on.

### 47.4 What this has *not* established

- **Which slot each side bought is not on this disk.** The list this
  crate's `make_stuff` reads on 9382 offers `2:t50(Citizen) c2 v234782 n2`
  and `5:` the same, against `4:t52(Scholar) c1 v45568`, under a head
  `0:t572(Mercenaries)` it cannot pay for at 80 metal of 100. Two citizens
  land in the queue; whether that is one slot's `num` (§42) or two slots
  is **not** decidable from the draws, because a purchase draws nothing.
  The original's list is not printed: `LEADERS=1` is `who`, `tribe`, `score`,
  `leader_flags`. **A `LEADERS=9` window over `[9375, 9390]` is what would
  print the other side** (item 414, run111).

  ~~Until then "the value is wrong" and "the offer should not exist" are
  not separated.~~ **That pair is a false one, and the disk said so before
  the capture ran — §47.6.** There are three readings, not two.
- The Scholar this crate does offer names `city` **1**, where the original's
  job at 9382 runs in the University of its *second* city. ~~Whether that is
  the same defect as the queue item on host choice, or a second one, is
  unmeasured here.~~ **The number in that sentence needs a mapping first**:
  this crate's `MakeObject::city` is an index into its own global city list
  and the dump's is the owner's own, so `c1` here is the **first** city, not
  the second — `diff/leader.rs` compares the two raw and pins the
  `MAKE[*].city` rows as residue for exactly that reason. The two sides do
  still name different cities; the host-choice question is untouched by this
  item either way.
- The window's standing residues — three positions (`1/24`, `1/25`, `1/26`),
  seventeen order stacks, twenty `CITY` fields — were **already parted when
  the window opened** on block 9340, so this item says nothing about them
  beyond pinning the sets. Fourteen of the twenty city fields are who=0's,
  the human's, read as zero here.
- `CityData::raid_stamp` and `city_flags 0x2` on the human's `0/2000` open at
  block 9451 and are the one thing besides the queue that moves inside the
  window. `compare`'s own city block names both as unheld by this crate.

### 47.6 The disk already refuses the dichotomy — three readings, not two

Written **before** run111, from `grep` alone, because "grep the disk before
booking a capture" is the rule and it paid here.

**run107 is already a `LEADERS=9` window on this very game** — item 369's
capture, `end: MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9`,
blocks `[9170, 9199]`. It does not reach 9382, so run111 is still owed. But
it prints the record this question is about, and `diff::tests::run107_s_
window_is_the_leader_record_at_the_word` already pins the make list as a
**standing residue**: `MAKE[1].val`, `MAKE[1].num`, `MAKE[2..3].{t, cat,
val, num}` and `MAKE[{0,1,2,4,5,8}].city` all part over that window.

So the third reading is: **the Citizen is not the defect at all.** A Citizen
that outranks a Scholar because the *Scholar's* offer is too small looks
exactly like a Citizen that should not have been offered, and only the
Scholar's own `val` tells them apart.

**Two cautions this subsection exists to carry, both nearly got wrong:**

- **`9,999,999` is not a ceiling; it is an overflow** (§45). The original's
  `val < 0 → 9,999,999` catches a 32-bit `imul` that *wrapped* in
  `income × val / 256`, and the clamp is at
  `create_units@006c40a0:1567` — `if (iVar8 < 0) { iVar8 = 9999999; }`,
  immediately before the `MakeList::make_me` call that files the offer. So
  an offer sitting at 9,999,999 is not "valued at the maximum": it is one
  whose pre-income value cleared §45's cliff. Comparing it against a small
  number is comparing a wrap to an arithmetic result, and what a capture can
  settle is the *inputs*, not which number is bigger.
- **run107's block 9199 and this crate's 9382 are 183 frames and one
  completed scholar apart.** Nothing here compares a field across that gap
  and nothing later should: run107's rows are a **bracket** on the shape of
  the residue, not a measurement of it at 9382. `docs/QUEUE.md` item 390 is
  the same mistake on the other track, which is why it is named here.

### 47.5 Coverage

Diff-backed, both halves: `run100_s_word_frame_is_the_original_s` (172
blocks, the whole `compare`) and `run100_s_scholar_job_runs_to_the_word`
(the original's queue alone, 130 blocks). Each was made to fail on purpose
before landing — the first by moving the raid block, which restores the two
human-city fields as rows; the second by dropping the clamp, which restores
`(9510, [(52, 12750)])`. Nothing here rests on a reading.

---

## 48. Great Lakes 9382 is a re-offer, and the Scholar's own value is the parting (2026-09-19)

Item 414, run111 — `LEADERS=9` over blocks `[9375, 9390]`, run100's detail
with that one category raised, straddling both the re-offer on sim-frame
9380 and the purchase on 9382. §47 put the AI headline's birth at 9510 on a
purchase made 128 frames earlier and could not say which slot either side
bought: a purchase draws nothing and `LEADERS=1` prints no list.

### 48.1 What the block kills

§47.6 left three readings standing. **The block kills two of them and the
one it leaves is neither of §47.4's original pair.**

- ~~**The Citizen's value is wrong.**~~ **Killed.** On block 9381 the
  Citizen is `val 234782`, `num 2`, and — through the city mapping — the
  same city on both sides. Identical.
- ~~**The Citizen should not have been offered.**~~ **Killed.** The
  original offers it, on the same block, at the same value, and simply
  does not buy it.
- **The Scholar's own re-offer is too small.** **Stands, and is measured.**

The prediction the stanza was written on is **also killed**, and it was
mine: it said the original's Scholar would *stay* at 9,999,999 with
`num 5` where this crate's collapsed. Both collapse, on the same block.

| block | theirs | ours |
| --- | --- | --- |
| 9375, 9376 | `t52 v9999999 num 5` ×2 | the same |
| 9377–9380 | no `t52` slot (the list refills) | the same |
| 9381, 9382 | `t52` **`v5755741`** `num 1` **×2** | `t52` **`v45568`** `num 1` **×1** |

So the frame was right and the mechanism around it was wrong for the
fifteenth time: **9380's re-offer is the event**, and what parts is the
number `create_units`' scholar arm puts on it. 5,755,741 clears the
Citizen's 234,782; 45,568 does not. Two slots against one is the same
finding said a second way — the count and the value fall out of the same
arm.

### 48.2 The original names its own purchase

On block 9383 — the block after `make_stuff` runs — **one** of the
original's two Scholar offers is `57,557` and the other still `5,755,741`.
57,557 is 5,755,741/100, the bought-slot devaluation, so the record says
which of the two it spent without any inference from this crate.

The muster is the second witness and it agrees with §47.2's building
queues seen from the other end: on 9383 `num_queued` is **two Citizens on
this side and one Scholar on the original's**.

### 48.3 The goods agree entering the frame and part through it

§44's shape exactly. `bucket` is `107 127 53 129 80 0` on **both** sides at
block 9381, and **both sell 100 timber on 9382** — timber 127 → 27 on both,
which is the two `Leader::use_market+0x1ed` draws §47.3 found agreeing. What
differs is what the proceeds buy: the original spends 40 on the Scholar and
keeps **82**; this crate buys food for two Citizens and keeps **14**.

So the stockpile is not the cause here any more than it was at 9182, and
this is now twice that a Great Lakes purchase divergence has had agreeing
goods and a parting list.

### 48.4 What this has *not* established

- **Why 5,755,741 and not 45,568 is not answered.** This item measures the
  two numbers on the same block; it does not derive either. The successor
  is `create_units`' scholar arm — `docs/AI.md` §38's gate and §42's batch
  are the two pieces the arm already has documented, and neither predicts a
  126× gap. **A ratio is not a lead**: 5,755,741/45,568 is not an integer
  and nothing here says the gap is one factor rather than several.
- **`num 5 → num 1` on both sides is unexplained.** Both lists drop the
  batch to one across the same re-offer, so whatever does that is modelled;
  what is not is the value that comes with it.
- **The city still differs and this item did not measure it.** The
  original's Scholar slots name its city 1 throughout; this crate's name
  its own city 1, which is the *owner's* city 0 under the mapping §47.4
  records. The host-choice question is untouched.
- **`GOODS` in `diff/leader.rs` is mislabelled and the comparison is not.**
  The array names index 2 "metal", 3 "wealth", 4 "knowledge";
  `sim::economy::Resource` has Wealth at 2, Knowledge at 3, Metal at 4, and
  index 2 is what pays this Scholar's 40 (run100's queue slot is `good[0]
  2, cost[0] 40`, and the bucket moves 53 → 82 across the purchase). Every
  value compared is index against index, so no verdict on any capture is
  affected — but every *name* printed or pinned at 2, 3 or 4 is the wrong
  resource. ~~including rows in `PARTS_ON_RUN19`, `PARTS_ON_RUN107` and
  `PARTS_ON_RUN111`. Renaming it churns three pinned constants across three
  items, so it is recorded here and not done under this one.~~ **Done
  under item 423, and this estimate was wrong on both counts.** Only
  **index 2** was ever mislabelled in a pin — there is no `[3:wealth]` or
  `[4:knowledge]` row anywhere — and the only pinned constant carrying one
  is `PARTS_ON_RUN111`. The change is seven lines with no number among
  them, and the label turned out to be neither load-bearing nor
  decorative but **self-consistent**: both sides of every comparison
  generate the key from the same array, so it had nothing to disagree
  with. `docs/journal/2026-09-19-item-423.md`.

### 48.5 Coverage

Diff-backed: `diff::tests::run111_s_window_is_the_make_list_at_the_purchase`
— 32 leader-blocks, 33,536 field-frames, the whole `LeaderData` record; the
Scholar and Citizen offers keyed **on the slot's type, not its index**
(the two lists do not hold the same types in the same slots, and the first
draft of this test compared a Phalanx to a Scholar and read it as a
finding); the bought-slot devaluation read off the capture alone; the
wealth either side; and the 104-row residue pinned as `PARTS_ON_RUN111`.
Made to fail on purpose before landing, on the re-offer row and on the
muster row. run111's own checks are in `docs/RUNS.md`.

---

## 49. Great Lakes 9380's Scholar offer is a positive wrap (2026-09-19)

Item 422, and it stopped being a valuation question the moment the
arithmetic was done. §48 measured two numbers on the same block —
**5,755,741** offered by the original and **45,568** by this crate — and
booked the successor by those two numbers and the frame, with no factor
named. There is no factor. **45,568 is the remainder of a 32-bit
overflow.**

### 49.1 What is measured

`create_units`' closing tail is `out = wm(fac, wm(want, val) / divisor) /
256`, with the original's own `out < 0 → 9,999,999` after it (§45). On the
re-offer frame this crate's five terms are, measured off the run and not
fitted:

| term | value |
| --- | --- |
| `val` | 42,000,000 |
| `fac` | 256 |
| `want` | 20 |
| `divisor` | 25 — `want` + 0 queued + 5 standing |
| `out` | **45,568** |

`20 × 42,000,000 / 25` is 33,600,000. `256 ×` that is **8,601,600,000**,
which does not fit an `i32`. It wraps to **11,665,408**, and `/256` is
45,568 exactly, to the unit.

**And the sign is why it was invisible.** §45's `out < 0 → 9,999,999`
catches a wrap that lands *negative* — the ceiling that section is named
for. This wrap lands **positive**, so the guard does not fire and the
overflow leaves a small, plausible number behind instead of an obvious
one. That is why nobody looking at 45,568 saw an overflow.

**But the wrap itself is correct, and §49.4 measures how correct.** The
original's tail is the same 32-bit `imul` and wraps the same way (§45);
this crate must wrap to match it, and removing the wrap costs **2,727
frames of the word**. So the arithmetic here is faithful and there is
nothing to guard: the defect is entirely **upstream**, in whichever of
`val`, `want` and `divisor` is large enough to push this product over
`i32::MAX` when the original's is not. An earlier draft of this section
called the wrap "the same defect §45 documents, in the arm next door" —
that was wrong, and the probe is what said so.

That number then loses the purchase: 45,568 ranks below the Citizen's
234,782, which both sides offer at the same value on the same block, and
the original's 5,755,741 ranks above it. 5,755,741 × 256 is 1,473,469,696
— inside an `i32` with room — so **the original's does not wrap**.

`sim::ai_units::offer_value` is that tail as one function, and
`the_scholar_offer_on_great_lakes_9380_is_a_positive_wrap` pins every row
above. Made to fail on purpose by widening the tail to `i64`, which
returns 33,600,000 and says the test is about the wrap rather than about a
literal.

### 49.2 The ratio and the gap were artefacts

§48.4 recorded that 5,755,741/45,568 is not an integer and that nothing
said the gap was one factor rather than several, and declined to name one.
**Both observations were artefacts of the wrap**: a remainder has no
arithmetic relationship to the value it came from, so the ratio was never
going to factor and the "126× gap" was never a gap. Recorded because the
discipline is what let the measurement arrive — the case where a number is
not merely unexplained but *meaningless* is exactly the one a named
mechanism would have buried.

### 49.3 What this has *not* established

- **Which of the original's three terms differs is not established, and
  this document does not guess.** `want · val / divisor = 5,755,741` with
  `divisor = want + 5` yields no integer `want` for `val` of either
  6,000,000 or 42,000,000, so at least one more term differs and the
  record does not carry `want` or `divisor`. **No capture can answer it** —
  they are locals, not fields — so if it is to be settled it wants a
  **reading** of `create_units@006c40a0`'s want-and-divisor block, which is
  what reading is still for (`CLAUDE.md`). **§50.1 sharpens it and gives
  the reading two targets**: at least one of the three is **per-city** in
  the original and none of them is here, and the two numbers to hit on
  that frame are 4,891,136 and 5,755,741. **Answered in part by §51: the
  term is `val`, through `gfree[KNOWLEDGE]` — and correcting only that
  costs 925 frames, because a second defect in the same arm was
  cancelling it.**

  **It is expensive, not unfalsifiable, and the difference matters.**
  `tools/emu/callfn.py` can enter a function under unicorn with chosen
  arguments and read what it computes, which would settle all three terms
  outright. `create_units` reads a singleton, so `docs/EMULATOR.md` prices
  it at an hour of synthesized state rather than a minute — but it is a
  real falsifier and this claim should not be recorded as though none
  exists.
- **`val = 42,000,000` is `k × 6,000,000` with `k = 7`, and that is
  *this crate's* `k`**, measured on this crate's census. Whether the
  original's `k` is 7 on that frame is not known; `k` is the scholar arm's
  own local and the record does not print it either.
- **A positive wrap is invisible wherever the arm is reached, not only at
  9382.** On any frame whose terms are large enough the expression yields
  an ordinary-looking number and nothing announces it. That is a property
  of the original's arithmetic as much as of this crate's — so what is
  booked separately is the **observability** problem, not a defect in the
  expression.

### 49.4 What a fix would be worth, measured — and both probes say "not this"

Before booking a reading of `create_units@006c40a0`, the cheaper question:
**if this crate's Scholar merely outranked the Citizen, would the word
advance past 9510?** Two scratch builds, both reverted, neither committed.

| probe | what it did | word |
| --- | --- | --- |
| — | the tree as it stands | **9510** |
| 1 | every Scholar offer forced to the original's own 5,755,741 | **8985** |
| 2 | the tail computed in 64 bits, so nothing wraps | **6783** |
| 3 | **only 9380's** offer forced to 5,755,741 | **9382** |
| 4 | 9380's offer **and** 9382's expiry, each scoped to its frame | **9518** |

**All three are worse, and probe 3 answers the question with a second
cause rather than with a payoff.** Forcing only the frame under test makes
this crate buy the Scholar — and the word then falls to **9382**, the
purchase frame itself, where it spends **nine draws against the
original's eight**. The extra is at index 6 and it is
`Leader::make_stuff+0x63d`, the slot loop's own expiry
(`SITE_EXPIRE_SLOT`, §41): buying the Scholar here costs this crate a
third expiry the original does not spend.

So **the ranking is not the last thing holding 9510.** A second difference
sits behind it on the same frame, in how many slots the loop expires, and
it is nearer than the want-and-divisor terms are. That makes a reading of
`create_units@006c40a0` **less** urgent rather than more: it would say
which term differs, and 9382 would still part on the expiry count.

~~**And the structural reading is the one to carry away: 9510 is propped up
by the wrong purchase.** Correcting the valuation alone moves the word
*down* by 128 frames, so the valuation and the expiry are not a queue of
causes to peel but a **pair that has to fall together**.~~ **Struck by
§50**: there is one cause, not a pair. The 128 frames are real and the
reading of them was not — probe 3 forced *both* of this crate's per-city
Scholar offers to one number, and the tie is what bought the Scholar
twice.

**The caveat, stated because probe 3 cannot separate it.** ~~Forcing the
value changes the make *list*, and the expiry count is a function of the
list, so the third `+0x63d` may follow from the list's shape rather than
from the purchase.~~ **It did, and §50 measures it**: the third `+0x63d`
is a tie between two per-city offers the probe flattened to one number,
not anything the slot loop does.

~~**Probe 4 prices the pair, and the price is eight frames.**~~
**Superseded by §50.3: the price is seventy-two frames, 9510 → 9582.**
Probe 4's cap stopped the second purchase but left the category slot
naming the wrong city, so its eight was the cap suppressing the fix it
was meant to complete. The paragraph below stands as the record of what
was measured under it. With both
halves forced — the valuation on 9380 and the expiry capped at the
original's two draws on 9382 — **frame 9382 agrees draw for draw**, eight
against eight entry for entry, and the word advances **9510 → 9518**. So
the pair is confirmed by measurement rather than inferred, and correcting
both of these causes is worth **eight frames of twenty-four thousand**.
That number belongs in the first sentence of any successor brief.

**Three separate calls, not one walk.** The first attempt at probe 4's
expiry half capped the walk inside a single `expire` call and changed
nothing: the three `+0x63d` draws come from **three separate calls**, the
slot loop calling `expire` once per bought slot. A fact about the shape of
the difference rather than about the patch.

**And the train rate at 9518 is not a second finding** — nor is 9518
(§50.3 strikes it as probe 4's artefact; the parting past the blockage is
**9582**, one `use_market+0x1ed` short). Under probe 4 the
new parting is this crate birthing the sixth scholar eight frames late —
128 frames of job against 136 — which looks like an unblocked defect and
is not. `run97_s_build_queues_are_the_original_s` compares `(type,
job_counter)` for every live slot of every building over `[8030, 9349]`:
**1,319 blocks, 37,899 building-frames, zero divergences**, and that
window holds **three scholar jobs this crate runs today** (9087, 9201,
9322). The train rate agrees tick for tick wherever both sides actually
run a job, so 136 against 128 is an artefact of the forced purchase.

Probe 2 is the one that corrected this section; `docs/journal/2026-09-19-item-430.md`
is the story. Removing the wrap
costs 2,727 frames, which says the wrap is **load-bearing fidelity**: the
original is a 32-bit engine, its `imul` wraps, and this crate matches it
on frames all the way down to 6783. The wrap is not the bug. It is
carrying a wrong input faithfully into a wrong answer.

Probe 1 is a fact about the instrument rather than about the simulation —
forcing *every* Scholar offer to one constant breaks 8985, §41 and §42's
own frame, where the value is already right, so what it measured was its
own bluntness. Recorded so the next person does not repeat it: **a payoff
probe must change only the frames under test.** A probe wide enough to
touch a frame that already agrees reports that frame's breakage as its
result, and the number looks exactly like a finding.

So the payoff question is **not answered** by either, and the honest
reading is that neither of the two obvious levers is the shape of the fix.
What the probes do establish is where a fix cannot be: not in the wrap,
and not in the offer's final value.

### 49.5 Coverage

`the_scholar_offer_on_great_lakes_9380_is_a_positive_wrap`
(`crates/sim/src/ai_units.rs`) — the five terms, the 64-bit product, the
wrap, the sign, and §45's guard still firing on a negative wrap, which is
what says the finding is about the sign. This crate's five terms are
instrument-measured; the original's 5,755,741 is read off run111 block
9381 and is diff-backed by
`diff::tests::run111_s_window_is_the_make_list_at_the_purchase`.

---

## 50. The Scholar is one value for two cities, and that is the whole defect (2026-09-19)

Item 432. §49.4 left Great Lakes 9382 with **two** causes locked together
— a valuation and an expiry count — priced at eight frames by a probe that
forced the offer and capped the expiry. There is one cause. **The expiry
was never a cause at all**, the third `Leader::make_stuff+0x63d` draw was
the probe's own artefact, and the honest price of the one defect is
**seventy-two frames**, 9510 → 9582.

### 50.1 What this crate does, and what the original does

`create_units` offers the Scholar **once per city**. On sim-frame 9380
this crate makes three offers and two of them are Scholars:

| offer | `t` | `city` | `val` | terms |
| --- | --- | --- | --- | --- |
| Scholar | 52 | 1 (their 0) | **45,568** | `want 20, val 42,000,000, divisor 25` |
| Citizen | 50 | 2 (their 1) | 234,782 | |
| Scholar | 52 | 2 (their 1) | **45,568** | `want 20, val 42,000,000, divisor 25` |

The two Scholars are **the same number**, to the unit, because all three
terms are the same for both cities. The original's are not:

| offer | `t` | its `city` | `val` |
| --- | --- | --- | --- |
| Merchant | 61 | 0 | 869,565 |
| **Scholar** | 52 | **0** | **4,891,136** |
| Citizen | 50 | 1 | 234,782 |
| **Scholar** | 52 | **1** | **5,755,741** |

Those four offers are not read off the record — the record prints slots,
never offers. They are **recovered**: `run111_s_block_9381_list_is_make_
me_s_from_four_offers` takes the original's block 9380 list as the start,
replays the four through `MakeList::make_me`, and lands on the original's
block 9381 list slot for slot and field for field. The city-0 Scholar's
4,891,136 is the sharp end of it — block 9381's slot 2 stands at `t −1,
val 4,891,136, city 0, cat 4`, no expiry ran on 9380, and the only other
thing that clears a ranked slot's `t` is `make_me` clearing a **same-type**
entry as it inserts. So slot 2 held a `t52` for city 0 and its `val` is
that offer's.

**And this crate offers no Merchant on that frame**, which is why
`MAKE[1..3]` have been standing residue since run107 (§47.6): the original
gives those ranks `t52`/`t61` and this crate fills them with `t133`/`t66`.

### 50.2 The tie is what buys the Scholar twice

`make_me` ranks on `list[k].val <= val` and fills the category slot on
`list[cat].val < val` (`006c9be0`; both senses read off the decompile and
both are this crate's). **The asymmetry is invisible until two offers
share a number.** Then the later offer displaces the earlier in the ranked
list and does *not* in the category slot, and the two copies of one type
end up naming different cities.

`make_stuff` step 6's duplicate test is `t == list[k].t && city ==
list[k].city` over slots 0..4 (`006c8af0:147–167`, also faithful). With
the cities split it does not match, so the loop **buys the same Scholar a
second time**: one extra `make_this`, one extra `expire` call, one extra
`+0x63d`.

That is the whole of "nine draws against eight, the extra at index 6". It
is a property of this crate's *tie*, not of its expiry walk, and the
original never sees it because its two Scholar offers differ, its category
slot follows the better one, and its step 6 dedupes.

### 50.3 The measurement, and what it replaces

Forced to the original's **two** per-city values on 9380 — 4,891,136 for
its first city, 5,755,741 for its second — **with no expiry cap and
nothing else touched**:

| probe | word |
| --- | --- |
| the tree as it stands | 9510 |
| §49.4's probe 4 (offer forced, expiry capped) | 9518 |
| **the two per-city values, no cap** | **9582** |

Frame 9382 then agrees **draw for draw, eight against eight entry for
entry**, and so does 9383 — which is the item's success criterion met
without the expiry half existing.

**Why the eight was wrong, and why the seventy-two is a different kind of
number.** Capping the expiry bought the right *count* by the wrong
*route*: it stopped the second purchase but left the category slot naming
the wrong city, so everything past 9518 ran on a list the original never
had. The eight was the cap suppressing the fix it was meant to complete.
The seventy-two adds nothing — the probe changes two values on one frame
and that is all. ~~§49.4's "a pair that has to fall together"~~ is
**struck**: they are one thing, and so is ~~9518's scholar-birth
divergence~~, which was probe 4's artefact through and through.

**The seventy-two is still a probe number** and is stated as one: it
measures what fixing the 9380 valuation is worth *given everything else
this crate does*, and it becomes real only when `create_units`' scholar
arm computes per-city.

### 50.4 What this has *not* established

- **Why the original's two values differ is not answered here**, and this
  document does not guess. §49.3's question is sharper for the
  measurement: at least one of `val`, `want` and `divisor` is **per-city**
  in the original and none of them is in this crate, and the arm has two
  known numbers to hit on one frame. That is the successor reading of
  `create_units@006c40a0`.
- ~~**The missing Merchant is observed, not explained.** `create_units`'
  caravan-and-merchant arm has a civilian-ceiling `continue` that would
  drop the offer; nothing here measures whether that is what drops it.~~
  **Explained by item 327 (§55)**: `known_rares` had no writer.
- ~~**The new parting is 9582 and it is one draw.** This crate spends one
  `Leader::use_market+0x1ed` where the original spends two — the market's
  sell rotation, the same site the two sides agree on at 9382. 9581 and
  9583 agree whole. It is not reachable until the valuation is real.~~
  **Closed by §53**: the valuation is real, and 9582 is a market sell on
  both sides. Great Lakes now takes nine market draws below the word
  against six, and `run53_s_24000_frames_put_the_ceiling_where_run33_did`
  compares the list against the original's own before the literal.
- **Nothing here says the tie is rare.** A positive wrap flattens whatever
  it swallows (§49.3), so two cities whose true values differ can collide
  on one wrapped remainder anywhere the arm is reached — the tie and the
  wrap are the same event seen twice.

### 50.5 Coverage

Diff-backed: `diff::tests::run111_s_block_9381_list_is_make_me_s_from_
four_offers` — the four offers, both ends of the transition read from
run111's own record, the city-0 value's derivation, and the order claim
(the city-0 offer precedes the city-1 offer; swapped, `make_me` breaks out
on the same type and the record is not reproduced). Made to fail on
purpose three ways: the city-0 value moved by one, the Merchant dropped,
and the two Scholars swapped — each restoring a different slot.

Reading-backed, and only the two predicates:
`sim::ai::tests::a_tie_splits_the_ranked_copy_from_the_category_copy`
pins `make_me`'s `<=`/`<` asymmetry and step 6's `(t, city)` test against
`006c9be0` and `006c8af0:147–167`, in both directions — tied offers split
the cities and defeat the dedupe, distinct offers do not. Made to fail on
purpose by weakening the ranked insert to `<`.

The 9510/9582 pair is instrument-measured on
`run53_s_24000_frames_put_the_ceiling_where_run33_did` and is not pinned:
9582 exists only under the probe.

---

## 51. A University's scholars are gatherers, and two wrongs were agreeing (2026-09-19)

Item 438, and the reading §49.3 held and §50.4 booked: which of `val`,
`want` and `divisor` is per-city in the original, where this crate computes
one number for both of its cities. **The term is `val`, through
`gfree[KNOWLEDGE]`, and the defect is a predicate rather than arithmetic.**
Correcting it alone costs **925 frames**, which is the more useful half of
the item.

### 51.1 The subtraction is faithful; the free count is not

`create_units`' scholar arm is `k = gfree[KNOWLEDGE] − <call>(1, 0x34)`.
The call was checked first, because the arm's own `num_inside` neighbour
made it look like an occupancy count: vtable `+0x190` on `BuildData` is
**`BuildData::count_queue`**, so `count_queue(uni, t)` is this crate's and
it is right. `0x34` is `TypeIndex` 52 — the Scholar exactly, not the
`is_scholar` pair.

That leaves `gfree`, which `City::count_gather_slots@00737dc0` writes as

    free[g] += gather_max − BuildData::num_gatherers(0, 0)

and `num_gatherers@00630450` is **two counts, not one**:

- `count_inside(COUNT_TYPE, 0x32)` when the building `is(0x1a6)` — the
  **Oil Platform**, whose citizens sit inside. `0x1a5`, the Oil Well, is
  *not* one of them: its workers stand at the tile and ride the chain.
- `count_inside(COUNT_TYPE, 0x34)` when it `is(0x1a4)` — the **University**,
  whose scholars sit inside.
- plus the `gather_down` chain, filtered by `UnitData::is_gathering_at`.

`TypeData::is@004771c0` is `this->type == param_1`, so both tests are exact
kinds and neither follows a lineage.

**This crate's `count_gather_slots` adds `gather_down`'s raw length.** So a
University's seated scholars never reduced its free slots, and on Great
Lakes 9380 both of the AI's cities answered `k = 7`. The original's own
record says what the answer should be: at block 9381 University `1/2019`
stands in **city 1** holding **one** unit on its `inside_down` chain, and
`1/2020` in **city 0** holding **four** — §47.1's five seated scholars,
counted from the other end. Routing the one call site through
[`sim::Sim::num_gatherers`] makes `k` **6** and **3**, and the per-city term
goes live.

**The function was already in the tree.** `Sim::num_gatherers` has
implemented both inside arms since `docs/ORDERS.md` §6.1; only this call
site asked the wrong question. That is the third time in a day that the
right answer was present at the wrong call site — the others are
`run97`'s queue assertion (§49.4) and item 432's instrument.

### 51.2 What correcting it costs, and why it is not landed

| tree | word |
| --- | --- |
| as it stands | 9510 |
| `count_gather_slots` through `num_gatherers` | **8585** |

925 frames, and it fails `LONG_WORD_GREAT_LAKES`. The correction is right by
the decompile and it is **half a fix**: this crate's `val` post-correction is
36,000,000 and 18,000,000 for the two cities, against the original's own
5,755,741 and 4,891,136, and the ratio is **2** where the original's is
**1.1768**. So `val ∝ k` is not how the original's per-city variation works
either, at least one more term in the scholar arm is wrong, and this reading
did not settle it. The two have to land together.

### 51.3 The two defects were cancelling, and a green frame is worth less than it looks

**This is the part to carry past the item.** With the over-large `gfree`,
this crate's `val` overflowed and §45's `out < 0 → 9,999,999` clamped it —
and the original *also* clamps on block 9375, so the two sides agreed there
**by accident**. Correct `gfree` and the crossover inverts: at 9375 this
crate gives 7,200,000 where the original clamps, and at 9381 it clamps where
the original gives 5,755,741.

Two wrongs producing a right, on a frame a comparison calls green. Nothing
in the diff could see it, because both sides were internally consistent and
the clamp erases the distance between "correct" and "enormous". So a green
frame is evidence that the *outputs* match, never that the inputs do — and
where an arm ends in a saturating clamp, a matching output is barely
evidence at all. §50 found the score at 9510 propped up by a wrong purchase;
this finds it partly propped up by a wrong agreement.

### 51.4 What this has *not* established

- **Which further term in the scholar arm is wrong.** The multiplier chain
  (`infra_mod · k · 10000 / 256`, then ×60, ×10 or ×5 on `filled`/`total`)
  reads as this crate's, and its two branch tests are `Leader` fields, so
  they are leader-global on both sides — but the arm's product is 5–6×
  the original's and nothing here says where. The two target numbers and
  the landed guard are what the successor has to hit.
- **No term is named by fitting, deliberately.** Two values and a ratio
  are exactly the conditions under which a plausible factor appears; three
  candidate factorisations were tried and discarded during this item, and
  none is recorded, because §49.2's lesson is that a number downstream of a
  clamp or a wrap may have no arithmetic relationship to its inputs at all.
- **The `.max(0)` clamp on the free count is this crate's.** The original
  adds `gather_max − num_gatherers` raw, so a building over its own slot
  count lowers the city's total. Untested either way: no capture on disk
  has an over-full gather building on a compared frame.
- **The Korean Scholar (`0x35`) is a seam.** `num_gatherers`'s inside arm
  counts `0x34` exactly, while `is_gathering_at`'s counts the
  `{SCHOLARS, SCHOLARSKOREAN}` pair (`docs/ORDERS.md` §6.1, G17), so the
  two disagree for a Korean AI. This crate's `Worker::Scholar` is the pair
  in both places. No run on disk has a Korean scholar.

### 51.5 Coverage

Diff-backed: `diff::tests::run111_s_universities_hold_the_scholars_this_
crate_counts` — the capture's own `BUILDDATA` says each building is a
University and whose city it stands in, the `inside_down` chains give one
and four, and this crate's `Sim::num_gatherers` agrees with both. **It
passes today and is meant to keep passing**: it is the falsifier standing
ready for whoever lands the pair, not a pending failure. Made to fail on
purpose by reading `gather_down`'s raw length instead — which is the defect
itself, and answers 0 against 1.

Everything else here is **reading-only** and owed a blind second reading:
`count_gather_slots`' two counts, `num_gatherers`' two exempt kinds, the
exactness of `TypeData::is`, `BuildData::count_queue` at `+0x190`, and the
missing clamp. The 9510/8585 pair is instrument-measured and not pinned —
8585 exists only under a change that is not in the tree.

## 52. The AI's dump: the offers, read forwards (2026-09-21, the seventh pass)

DECISIONS 41 §6 said the AI would get its own dump, because the original
logs a Leader's *state* and never its reasoning — `LEADERS=9` prints the
ranked make list after the fact, and no category prints what was offered
to it. The seventh Fable pass built it, as parked 367 said: three call
proxies in a `RON_LEADER_PROBE` build of the tracer (`tools/trace/README.md`,
"The call proxies"), and run114 is the first capture through them —
run111's window exactly, with `callwin=9376-9386`.

### 52.1 The instrument

- `Leader::create_units@006c40a0` is the **bracket**: a `make_me` nested in
  it is a unit offer and not `create_buildings`' or an upgrade's.
- `MakeList::make_me@006c9be0` is the **offer**: `(t, val, escrow, cat,
  city, up, p7)` ride the record; `num`, the eighth argument, does not
  (the return record's last slot carries an out-byte), so `num` is still
  the make list's to give.
- `Leader::make_this@006c94f0` is the **purchase**, with its slot and its
  answer.

`rondata::trace::Trace::proxied` reads the run's own `PROXIED` records —
the tracer now writes each site's id beside the address it patched — so a
test says which build wrote the log rather than assuming; `tools/trace/
report.py … calls` names the sites the same way. The proxies did not
perturb the game: `samegame.py` against run111 with `LEADERDATA` included
is 16 blocks in common and 0 differing, and `rngcmp.py` against run53 is
9,406 frames identical.

### 52.2 What the record says on 9380

Four `make_me` inside one `create_units`, on the re-offer frame and no
other frame of the window, in this order:

| `t` | `val` | `city` | `cat` |
| --- | --- | --- | --- |
| 52 Scholar | **4,891,136** | 0 | 4 |
| 61 Merchant | 869,565 | 0 | 4 |
| 50 Citizen | 234,782 | 1 | 5 |
| 52 Scholar | **5,755,741** | 1 | 4 |

Every value is what §50.3 reconstructed by replaying block 9380's list into
block 9381's — the reconstruction was the stanza's written prediction and
it held. `escrow` is 1 and `up` 0 on all four; `p7` is 9,999,999 on all
four, the seventh argument `make_me` stores nowhere.

This crate's offers on the same frame, off the recorder `create_units`
now fills (`Leader::unit_offers`, stamped with its frame): Scholar
**45,568** city 1, Citizen 234,782 city 2, Scholar **45,568** city 2 — the
two numbers §49 and §51 measured, no Merchant, and the city index one
apart because it is this crate's index into `cities` and not the leader's
own numbering. That is item 442's residue, pinned in no direction.

The purchase is one `make_this` on 9382, `slot 1`, answering 0.

The window also holds what no item asked for: two building offers on
9378 (`t552` at 126,000 and `t573` at the 9,999,999 ceiling, both `cat`
8/10, city −1), two `cat 7` offers on 9379 (`t66` 167,424, `t133`
335,616), and eleven on 9381 (`t427`, `t428`, `t437` at −6,777,217,
`t438` at 100,000 and 75,000 in the two cities, `t526`–`t528` at 0) —
the AI's other steps, printed for the first time and compared to nothing
yet.

### 52.3 What it corrected

§50.3 (item 432) said the city-0 Merchant had to precede the city-0
Scholar for the replay to reproduce block 9381. The proxy prints the
Scholar first, and **both orders replay to block 9381** — the Scholar
outranks the Merchant either way, so the list cannot tell. The order
clause was the one thing the reconstruction had no evidence for, and it
is struck in `run111_s_block_9381_list_is_make_me_s_from_four_offers`'s
own note. Nothing else moved.

### 52.4 What this has *not* established

- What `make_this`'s answer means: it returned 0 on the frame the Scholar
  was bought. Whether that is "bought" or "nothing left to buy" is a
  reading of `Leader::make_this@006c94f0`, not taken.
- What `p7` is for. `create_units` passes the 9,999,999 ceiling and the
  building arm passes 0 or 4; the callee stores neither.
- Whether the building and upgrade offers on 9378, 9379 and 9381 are this
  crate's. They are on the record and nothing compares them; a `compare`
  over `create_buildings`' recorder is the item that would.

### 52.5 Coverage

Diff-backed: `diff::tests::run114_s_offers_are_the_original_s_own` — the
three sites resolved by address from the log's own records, the four
offers on 9380 and no other frame, the purchase on 9382, and this crate's
three offers on the same frame off the stamped recorder. Made to fail
first by the reconstruction's own order, which is how §52.3 was found.
The replay in §50.3 now runs in the measured order and still reproduces
block 9381.

## 53. The scholar arm pays both of the original's offers (2026-09-21)

Item 442, the successor §51 booked and could not land alone. **The term is
`val`, and it was wrong twice over.** Correcting `count_gather_slots` gives
`k` the original's own 6 and 3; reading the arm's third multiply as the
independent `if` the listing has gives the chain its missing ×5. Neither
alone reaches either target — the first clamps both cities to 9,999,999 and
costs 925 frames, the second leaves them tied — and together they hand the
tail **180,000,000** and **90,000,000**, which come out of `offer_value` as
**5,755,741** and **4,891,136**: run114's two numbers, to the unit, on the
frame it printed them.

**Great Lakes' word goes 9510 → 10161**, the first past ten thousand, and
the sixth scholar §47 named is born on 9510 on both sides.

### 53.1 The third `if` is not an `else if`

§51.4 said the multiplier chain "reads as this crate's". It did not. The
decompile prints three sequential `if`s whose first and third test the same
pair of `Leader` fields, which reads like a decompiler artefact and is not
one — the listing over `006c528c..006c52f5` settles it in ten lines:

```
006c528c  eax = [ebx+0x7a4]                    ; infra_mod
006c5292  ecx = [ebx+0x8b0]                    ; total
006c5298  imul eax, edi                        ; × k
006c529b  imul eax, eax, 0x2710                ; × 10000
006c52ab  sar esi, 8                           ; /256, toward zero
006c52ae  cmp [ebx+0x8c8], ecx  ; jge 6c52c0   ; filled < total   → ×60
006c52d6  cmp ecx, eax          ; jge 6c52e6   ; filled < 2·total/3 → ×10, escrow
006c52e6  cmp ecx, [ebx+0x8b0]  ; jge 6c4cfc   ; filled < total   → ×5
006c52f2  lea esi, [esi+4*esi]
006c52f5  jmp 6c4cfc
```

The third `jge` at `006c52ec` skips **only** its own
`lea esi,[esi+4*esi]`, and both arms land on `LAB_006c4cfc`. So a city
below two thirds takes all three multiplies — **×3000, not ×600** — and
`escrow` belongs to the middle arm alone.

This is the predicate class `docs/audit/README.md` keeps naming: not which
multiplier, but **which step it belongs to**. The arithmetic was doubly
confirmed and the control flow was never read.

### 53.2 The value diff on 9380, both directions

The item's own falsifier, off run114's `RON_LEADER_PROBE` trace (§52) and
this crate's `Leader::unit_offers` recorder on the same frame:

| `t` | the original's `val` | its `city` | this crate's `val` | our `city` |
| --- | --- | --- | --- | --- |
| 52 Scholar | 4,891,136 | 0 | **4,891,136** | 1 |
| 61 Merchant | 869,565 | 0 | — | — |
| 50 Citizen | 234,782 | 1 | 234,782 | 2 |
| 52 Scholar | 5,755,741 | 1 | **5,755,741** | 2 |

**And this confirms the city shift rather than assuming it.** §52.2 read
the one-apart index as this crate's `cities` index against the leader's own
numbering and had nothing but the index to say so. Two independent values
now agree across the shift — our city 1 carries the number the original
attributes to its city 0, our city 2 its city 1 — which an index comparison
could never have said.

~~**The Merchant is the residue.** It is observed and unexplained, exactly as
§50.4 left it: `create_units`' caravan-and-merchant arm has a
civilian-ceiling `continue` that would drop the offer, and nothing here
measures whether that is what drops it.~~ **Not the ceiling: item 327
(§55)**. The arm never reached the tail, because `known_rares` had no
writer.

### 53.3 What the pair paid

Every record that moved, and every one moved **down** with nothing
arriving:

| instrument | before | after |
| --- | --- | --- |
| Great Lakes' word | 9510 | **10161** |
| run111's leader residue | 104 fields | **93** |
| run107's leader residue | 95 fields | **90** |
| run111 block 9381's Scholar offers | one 45,568 vs two 5,755,741 | **two against two** |
| run111 block 9383's `num_queued` | 2 Citizens vs 1 Scholar | **1 Scholar vs 1 Scholar** |
| run111's wealth through 9382 | 14 vs 82 | **82 vs 82** |
| run100's queue record, whole window | two buildings parting | **nothing** |
| Great Lakes' scholar seatings below the word | 5 | **10**, each on the original's own frame |
| East Indies ladder C / B spurious units | 25 / 19 | **17 / 12** |

§50.4's third open question closes with them: 9582, the parting that
section's probe was one `Leader::use_market+0x1ed` short of, is a market
sell on both sides now, and three more of the 200-frame rotations came
under the word with it.

The eleven leader fields that closed on run111 are the make list's
`MAKE[1]` and `MAKE[2]` heads and the purchase downstream of them —
`num_queued[0]`, `num_queued[2]` and `bucket[0:food]`, the two Citizens
this crate used to buy instead of the Scholar. §50's tie, §47's missing
birth and §48's goods parting all close on the same line.

**The endpoints moved in both directions and are re-pinned as counts, not
as a trade** (DECISIONS 36): Great Lakes 51 → 55 off, 11 → 8 unlinked,
9 → 10 build_diverged; East Indies 5 → 10 unlinked, 14,000 frames past its
own word, which does not move.

### 53.4 What this has *not* established

- ~~**Why this crate offers no Merchant on 9380.**~~ **Answered by item
  327 (§55)**: `known_rares` had no writer. The Merchant is offered on 9380
  at the original's 869,565.
- **Whether `filled` and `total` are right.** Both branch tests passed at
  `filled = 0, total = 14` on this frame, and all three arms fire. A frame
  where a city sits between two thirds and full would separate the ×60
  from the ×5 and no capture on disk has one — so the *ordering* of the
  three is read from the listing and the *ceiling* between them is not
  diffed.
- **The `.max(0)` clamp on the free count is still this crate's.** The
  original adds `gather_max − num_gatherers` raw (`00737dc0`, the
  `param_2 != 0` arm), so a building over its own slot count lowers the
  city's total. Unchanged from §51.4, untested either way, and left in
  place because nothing on disk reaches it — a deliberate divergence, not
  an oversight.
- **The Korean Scholar (`0x35`) is still a seam**, exactly as §51.4 states
  it. No run on disk has one.
- **10161 is not an AI parting.** It is seven draws against eight at
  `Guy::set_anim+0x97a`, reached from `Guy::inc_time+0x271` here and
  `Unit::move_step+0x4e2` there, with the record agreeing from the other
  side — one unit's position, `1/38`, and nothing in the queue, city or
  order records moving with it. An animation clocked off the wrong caller.

### 53.5 Coverage

Diff-backed, and the pair is asserted from both ends:

- `diff::leader::tests::run114_s_offers_are_the_original_s_own` — the
  value diff above, now with the two Scholar values asserted **against the
  original's own list** rather than as literals, so the row says "ours are
  theirs" and not "ours are these numbers". Made to fail on purpose by the
  `else if`, which clamps both to 9,999,999.
- `diff::leader::tests::run111_s_window_is_the_make_list_at_the_purchase`
  — block 9381's two offers, block 9383's queue, the wealth through the
  purchase, and the 93-field residue.
- `diff::harness::tests::run100_s_word_frame_is_the_original_s` — the
  sixth scholar's block is clean, the queue record parts nowhere, and the
  word's own frame is one unit's position. Keyed on the new
  `GREAT_LAKES_SIXTH_SCHOLAR` rather than on the headline, because a test
  written about an event reports a moving headline as its own failure.
- `run53_s_24000_frames_put_the_ceiling_where_run33_did` — ten seatings,
  and the comparison against the original's own seating list **passed
  unchanged**; only the literal beside it moved.
- `run111_s_universities_hold_the_scholars_this_crate_counts` — §51's
  falsifier, standing and still green, which is what it was landed for.

Reading-backed, and owed a blind second reading: the three-`if` control
flow of `006c528c..006c52f5` (the listing is quoted above, so a reader can
check it without Ghidra), and everything §51.5 already lists as
reading-only. The 9510 → 10161 pair is measured on
`run53_s_24000_frames_put_the_ceiling_where_run33_did` and **pinned** —
`LONG_WORD_GREAT_LAKES` is 10161 in the tree.

---

## 54. Great Lakes 10161 is a refused step, and the record names the point (2026-09-21)

Item 448, the widening `LONG_WORD_GREAT_LAKES` owed since item 442 moved it
9510 → 10161 (`docs/DECISIONS.md` 43). **No capture was booked**: run100 is
`[9340, 10899]` of this very game at `UNITS=3, BUILDS=7, CITIES=5, GUYS=4,
DEATHS=1, LEADERS=1`, so the word's own block was already on the disk and
had never been read whole.

### 54.1 What the widening compares, and what was not compared before

`run100_s_word_block_is_every_record_the_dump_carries` walks **837 blocks,
`[9340, 10175]`** — run100's first complete block to fourteen past the word
— and compares **1,975,563 record rows**. Two things it does that
`run100_s_word_frame_is_the_original_s` beside it does not:

- **Every `compare` category, keyed by *field* rather than by unit.** An
  order row keyed on `(who, o)` reports the first field to part and hides
  every other for the rest of the run. `1/38` has carried a `GROUPORDER`
  `id` residue since block 9340, and it swallowed `coll_x`/`coll_y` on the
  word's own frame — the one field of the record that says in so many
  words what the original refused. Re-keying by unit is one of this test's
  deliberate failures, and the row vanishes.
- **The `UNITDATA` and `GUY` rows nothing on this map had read**, and the
  collision block **ungated by the position**: `dest_angle`, `form`,
  `form_mod`, `myspeed`, `myhits`, `stance`, `orders_x/y`, `tolerance`,
  `path_recursion`, `idle`, `orders.len`, the guy clock whole, and
  `collide*`, `safe`, `start_dist`, `mylos`, `packed`, `half_step`.
  `compare` takes the last group only where the two positions already
  agree — right for a residue count, blind on exactly the frame a position
  parts. `half_step` is `unit_masks & 0x100000`, it is dumped on every
  block, and it is one of the two rows that name the word.

### 54.2 The word's own block: fifteen rows, one unit

Block 10162 is sim-frame 10161. Of 104 units, 29 buildings, 3 cities and 4
leader blocks, **one unit parts and nothing else does** — no unlinked unit,
no extra unit, no building this crate holds that the block does not name,
and not a row of the city, queue, gather or build records.

| field | ours | theirs |
| --- | --- | --- |
| `pos` | `(42774, 22584)` | `(42754, 22582)` |
| `order:coll` | — | `(42774, 22584)` |
| `collide_guy` | `-1` | `0` |
| `half_step` | `1` | `0` |
| `g.cur_anim` | `8` | `0` |
| `g.cur_time` / `g.end_time` | `4` / `15` | `1` / `33` |
| `g.last_time` | `3` | `0` |
| `g.last_speed` / `g.avg_speed` | `20` / `18` | `0` / `13` |
| `g.stopped` | `0` | `1` |
| `g.x` / `g.y`, `g.des_x` / `g.des_y` | `42774, 22584` | `42754, 22582` |

**The original refused the point this crate stepped onto.** `1/38` is a
`group 64` member (`myhits 120`, `myspeed 25`, `size 30`) walking an
`ATTACK_TO` leg toward `(44712, 22536)`. Its record either side of the
word, read off the dump alone:

| block | `x, y` | `dest` | `dest_x, dest_y` | `coll_x, coll_y` | `length` | `last_speed` | `cur_anim` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10161 | `42754, 22582` | 1 | `42763, 22583` | `0, 0` | 2 | 15 | 8 |
| **10162** | `42754, 22582` | **0** | `42774, 22584` | **`42774, 22584`** | **1** | **0** | **0** |
| 10163 | `42786, 22586` | 1 | `42798, 22587` | `42774, 22584` | 2 | 32 | 7 |

That is `docs/COLLISION.md` §5.4's **snap arm**, field for field: the probe
writes `coll_x`/`coll_y` and stamps `collide_guy 0` (sticky — it is still 0
a thousand blocks later), `collide_o` and `collide_who` go back to −1 two
instructions later, `collide` and `collide_frame` are not touched at all,
the waypoint is consumed without a step (`dest` 1 → 0, `length` 2 → 1) and
`set_anim(CHAR_DEFAULT)` rolls the idle. The draw stream says the same
thing from the other side: the original's eighth draw is
`sim::anim::SITE_SNAP_BLOCKED`, `Guy::set_anim+0x97a < Unit::move_step+0x4e2`,
and this crate's seventh is `Guy::inc_time+0x271` — its walk animation
ticking on. **Two instruments, one frame, and it is the same signature item
360 landed for Great Lakes 9134.**

**And the two sides agree on where the unit was going.** Every waypoint
matches frame for frame through the window; on the word both hold
`(42774, 22584)`. So the parting is not a different destination or a
different path — it is the **probe's verdict on an agreed point**. The
positions re-converge on block 10165, four blocks later.

### 54.3 What else the window holds, and it is all older than the word

298 keys part over the 837 blocks: **161 already standing** on the window's
first block, **49 opening below the word**, **15 on it**, 73 after. Every
one of the 49 belongs to a family that was already standing — a newly
trained unit arriving with the same residue its predecessors carry — and
five of those families had never been compared on this map at all:

| family | rows | shape |
| --- | --- | --- |
| `form` | 48 | ours `-1`, theirs `9` on every unit outside a group |
| `dest_angle` | 22 | a whole-turn value apart, on units under way |
| `orders_x` / `orders_y` | 36 | 18–24 units off, one pair 984 |
| `g.angle[0]` | 10 | ours `1431655765` (a third of a turn), theirs `0` |
| `stance` | 6 | item 190's, unchanged |

**The collision block, ungated, agrees almost everywhere**: over 837 blocks
it produced **six rows on three units** — `1/31`'s `half_step` on 9407,
`1/15`'s `collide_o/who/guy` on 10170, and the word's own two. So
`half_step: ours 1 theirs 0` is *not* on its own the mechanism: this crate
sets the soft flag where the original does not, 754 frames earlier, with no
consequence at all. What the word's block establishes is that both sweeps
ran and reached different verdicts on the same point — not which arm of
`detect_unit_collision` decided it.

### 54.4 What this has *not* established

- ~~**Which unit the original's probe finds at `(42774, 22584)`, and why
  this crate's does not refuse it.**~~ **Taken as item 456, and the
  capture is run116** — `docs/COLLISION.md` §9, and the falsifier this
  bullet wrote held on the first pass: one bracket on 10161, returning 1,
  with a `who/o` named inside it. The collider is **`1/31`**, the hit cell
  **`(892, 471)`** off §4.2's fast-path edge sweep, and `will_be_corner 5`
  against `is_corner 0` — the corner rule was asked and refused to let the
  two slip past. Six of the eight steps already agreed, including the
  probe's own cell and the 3×3 walk's order; the parting is §4.3's
  group-mate soft arm and **one clause of it**, `UnitData +0x104`, which
  the PDB calls `openlist`: `1/31` holds a suspended 48-grid search on
  that very frame, so the original's arm declines. `Unit::search` **is**
  that field and has been since `docs/PATHFINDER.md` §18 — the seam
  comment that said the crate did not keep one outlived it by a
  fortnight. Great Lakes' word **10161 → 10232**, and the fifteen rows of
  §54.2 go to nought.
- ~~**The new word 10232 is formation pathing**: a three-unit squad
  carrying a 44-entry path plan against the original's one hop.~~ **It was
  not, and the frame was a block late.** Taken as item 463: the squad's
  *order lists* part on **10232**, a block before their positions do, and
  it is six raiders and not three — `1/27`, `1/28`, `1/29`, `1/40`,
  `1/41`, `1/42` — each holding one order where the original holds two.
  The human's building `0/2004` dies on block 10231 and the original's
  raiders keep their `ATTACKORDER` on it for the length of their reload;
  this crate dropped all six at once, and the three whose freed group move
  had somewhere to go walked. `docs/ORDERS.md` §7.12 has the arm and the
  per-unit drop frames. Great Lakes' word **10232 → 10233**, and the
  fifty-three rows of the old word's block go to two — both on the human's
  citizen `0/5`, which is the new word.

  The lesson is 456's own, one level along. 456 read its value diff at the
  word's block and named the mechanism from the *largest* row set there;
  the order row that says what happened was one block earlier and two
  lines long. **`firsts` is a first-parting map, so the block a record
  parts on is the answer and the row count is not** — and a widening that
  omits a dumped field (`recharging`, here) cannot be read as saying the
  record agrees.
- **Whether this crate reaches the snap arm here at all.** Its step was
  accepted, so nothing says whether its `move_step` took the snap branch
  and passed the probe, or took the partial branch and never asked.
  `sim::anim::SITE_SNAP_BLOCKED`'s own draw would say, and it is absent on
  this frame by construction.
- **`LEADERDATA`'s `score` and `leader_flags` are dumped four times a
  block and compared by nothing** — here or anywhere in `rondata::diff`.
  Neither is modelled, so this widening states the gap rather than closing
  it.
- **The per-frame `WORLD` census is not even parsed** — `forest_size`,
  `mountain_size`, `rock_size`, `total_metal`, `total_oil`, `goodies`,
  `land_resources`, `sea_resources`, on every block of every capture.
- **The five residue families in §54.3 are measured, not diagnosed.**
  `form 9` on every ungrouped unit is the largest and the cheapest to
  read; none of them moves a score, so none is booked here.

### 54.5 Coverage

Diff-backed, and made to fail on purpose before landing:
`run100_s_word_block_is_every_record_the_dump_carries` — the fifteen rows
above asserted as a written-out set with the dump's own numbers in them,
the block's record census as a floor (104 units, 29 builds, 3 cities, 4
leaders; 327 collision rows, 966 gather rows), and 1.9 M rows over 836
blocks so a capture without `GUYS=4` or `BUILDS=7` cannot pass by saying
nothing. **Item 463 added `recharging` to the record** — dumped on every
block, compared by nothing until then — and a second assertion beside the
word's own: the six raiders' order lists, over the whole tail of the raid
rather than on one block, so a rule that drops them together fails on
three of the four the window reaches and one that never drops them fails
on the other three. Keying the order rows by unit drops `order:coll`; restoring
`compare`'s position gate drops `half_step` and `collide_guy` together.
`RON_DEBUG_ROWS=<lo>-<hi>` prints every key that parts inside those blocks
with its value diff.

Nothing in this section rests on a reading. The *attribution* of the
original's behaviour to `docs/COLLISION.md` §5.4 is the record's own —
seven fields of the snap arm's nine-instruction block, each dumped — and
the mechanism behind the probe's verdict is left open above rather than
guessed at.

---

## 55. The Merchant arm reads a sum, and the sum has two writers (2026-09-22, item 327)

Item 520's widening named the input: on Great Lakes' block 11185 the
original's make list holds an **emptied Merchant slot** (`t −1`) where this
crate held a Cataphract (`docs/ECONOMY.md` §14). The original has offered a
Merchant on every rotation since at least **7584**, and this crate never
did: `civilian_value`'s merchant arm read a `known_rares` summed from
`census.reg_known_rares`, which the census zeroed every sweep and nothing
wrote. This section is the three halves §14.4 left unread, and the
implementation beside them.

**Great Lakes' word goes 11185 → 11531.** East Indies' holds at 9711.

### 55.1 The writer of `reg_known_rares`: the census's step 9

`reg_known_rares` is `LeaderData +0x4d4`, `int[64]`; `known_rares` is
`+0x6d4`, and the export's only functions naming either are
`Leader::plan_strategy@006b9620`, `Leader::calc_gather@006ceee0`,
`Leader::create_units@006c40a0`, `Leader::init@006e3930` and
`World::compute_reg_territory@006b0bb0` (§55.3).

`plan_strategy`'s step 8 zeroes the array (the loop at `:285–313`,
`puVar36`), and step 9 (`:314–361`) walks `new_rares` (`+0x6e6c`, a
`SimpleArray<int>`: count at `+0x6e70`, data at `+0x6e7c`) and, for each
good:

1. **Is it explored?** When `who < 8` and `reveal_map != 3`, it counts
   only if `leader_flags & 0x800`, or `+0x59e4`, or the byte of
   `World::seen2` (`+0x160`) at `div_3_table[y >> 7] × fog_xs +
   div_3_table[x >> 7]` has a bit of `ally_mask` (`+0x6929`) — which is
   `WorldData::was_really_seen`'s test at the good's half-cell, exits and
   all, and this crate calls `Sim::was_really_seen_fog` for it. Otherwise
   it counts unconditionally.
2. **Is its ground reachable?** The `WData` byte at `+0xf` of the good's
   cell (`div_3_table[c >> 8]`) is the owner. A negative owner, or `who`
   itself, counts; another leader's counts only if `diplos` reads 2 both
   ways.
3. `GoodData::ever_seen |= 1 << who`, and `reg_known_rares[r]++` for the
   cell's region (the `WData` short at `+4`).

`ever_seen` has one reader, `GoodData::is_seen`, and nothing here asks a
good whether it is seen, so it is left unwritten and named in the census's
seam table.

### 55.2 The sum, and the arm

`calc_gather@006ceee0:66–75`, the first thing past its cadence gate, sums
the array into `known_rares`. The census and the recompute run on
different cadences, so the sum lags the array by up to a recompute, and
this crate writes it where the recompute is,
`Sim::assemble_holdings_with`. `create_units`' merchant arm is its only
reader. The listing over `006c5427..006c5526` is the arm, in order:

```
006c547d  push 0x1b4 ; CityData::get_building(MARKET)  → js skip
006c54b4  call [vtbl+0xac] ; test [eax+8], 1           → je skip
006c54c4  push 0x228 ; LeaderData::has_tech(MATHEMATICS) → jne ok
006c54d4  push 1 ; push 3 ; LeaderData::type_avail(KNOWLEDGE, 1) → je skip
006c54e9  known_rares − num_queued[t] − num_units[t]   → jle skip
006c550b  effective_pop < pop_cap − 1                  → jge skip
006c551e  esi = 0xf4240 ; escrow = 1 ; jmp 006c4cfc
```

The Mathematics-or-knowledge line was **missing** from this crate's arm;
the decompile prints it with enum names, and the pushes settle which
arguments are which. It is `Roles::mathematics` and `Roles::knowledge`
here.

**The value is not the arm's.** It hands the shared tail 1,000,000, and
the tail's `offer_value` divides by `want + queued + units` and scales by
`check_income`'s factor — the same arithmetic every civilian takes. So
§14.4's 869,565, 909,090 and 227,272 needed no division of their own: once
the arm fires, the tail produces them. The emptied slot is `make_me`'s:
`MakeList::make_me@006c9be0` sets every lower slot holding the offered type
to `t −1`, keeping its value, and this crate's `make_me` already did. So
the re-offer on 11184 empties slot 1 with no further change.

### 55.3 The second writer: the territory pass

`World::compute_reg_territory@006b0bb0:76–105`: when a region's border
pass starts (`Region.borders == 0`), it zeroes `reg_terr[r]` **and
`reg_known_rares[r]`** on every live leader, and clears every city's
`bordering`. The census recounts on its next sweep. A `calc_gather` in
between sums a zero.

That is run91's one new row. The original's player 1 holds `known_rares
0` from its recompute on **7335** (`gather_stamp 7335`) to the one on
7583, while its array reads 3 on 7514. This crate reads 3 throughout.
Zeroing the array in `Sim::sync_territory` **reproduces the zero** on
7328, then costs `gather_stamp` on run84, run19 and run91, and does not
move the word. This crate recomputes every region at once, on the event,
and the original schedules a budgeted pass (`GameDaemon::check_borders`,
256 cells a frame), so the frame the zero lands on belongs to a transient
this crate does not model (`docs/ATTRITION.md`, "Territory"). ~~The writer
is named here and not implemented.~~ **Implemented by item 822, §76**: the
fix raises a flag and the next frame's `check_borders` zeroes the array,
which is the frame run227 dates. `1/known_rares` on run91 is the only row it
leaves, and it does not reach the Merchant slots on any block.

### 55.4 What it moved

The value diff is `the_merchant_slots_are_the_original_s_own`. On every
`LEADERDATA who 1` block that prints a make list, in nine Great Lakes
captures (run84, run91, run107, run115, run19, run111, run100, run117,
run123), the multiset of the original's Merchant slots, by `(val, escrow,
cat, num)` and never by slot index, is compared with this crate's.
**1,305 blocks, 1,173 Merchant rows, every block agrees.** The first is
7584, at 217,391. With the sum zeroed, 835 blocks part.

| instrument | before | after |
| --- | --- | --- |
| Great Lakes' word | 11185 | **11531** |
| East Indies' word | 9711 | 9711 |
| run114's offers on 9380 | three, no Merchant | **four, the original's four** — the Merchant at 869,565 |
| run123 block 11185 `MAKE[1].t` | 227 / −1 | **−1 / −1** |
| run123 keys first parting on 11185–11186 | 10 | **1** (`MAKE[3].val`, 77/75) |
| run19 leader residue | 80 fields | **78** |
| run115 leader residue | 89 | **84** |
| run117 leader residue | 105 | **101** |
| run91 leader residue | 91 | **90** (−2, +`known_rares`) |
| Great Lakes endpoint | 57 off, 4 unlinked, 10 build-diverged | **50, 1, 9** |
| East Indies endpoint | 62 off, 10 unlinked | **64, 9** |
| East Indies ladder B | 47 off, 15 extra | **49, 14** |

`known_rares` is a row of the leader record now. It agrees on every block
of run84, run107, run115, run19, run117 and run123, and parts only on
run91's §55.3 lag. It left `diff::coverage::UNREAD`.

**Parked (450) closes.** It was this offer on Great Lakes 9380, and 442's
own test now finds the original's four offers there with the Merchant's
value to the unit.

**The new word**: 11531, ours four draws against the original's three,
parting at index 1 — `Guy::set_anim+0x97a < Unit::do_idle+0x7d` against
`Guy::set_anim+0x97a < Guy::inc_time+0x271`. That is an animation clocked
from the idle path where the original's comes from `inc_time`, 72 blocks
past run123's last. No dump reaches it, so its widening is owed a capture.

### 55.5 What this has *not* established

- ~~**The frame of the territory zero** (§55.3). Its writer is read. Placing
  it needs `check_borders`' budgeted schedule, which this crate does not
  model.~~ **Answered by §76**: the first `check_borders` after a fix
  zeroes every region, before its budget test. Only the re-zero of regions
  the pass has not reached is unmodelled (§76.6).
- **The Mathematics-or-knowledge gate is reading-only.** No capture was
  checked for which operand holds at the first offer, and no block on
  disk separates the gate from its absence.
- **The explored and ownership predicates are diffed only in sum.**
  `reg_known_rares[scan]` is not parsed per region (the census's other
  `reg_*` arrays are not either). `known_rares` agrees wherever §55.3
  does not intervene, and on Great Lakes every rare sits in one region.
- **`MAKE[3].val` at 77/75 of the original's** on run19 (8174), run117
  and run123's 11185. ~~It is §14.4's `get_cost` row and not this item's.~~
  **Not that row**: item 545 closed §14.4's with the military discount and
  this survives it. On 11185 it is the Mine's value, a building (§56.5).

### 55.6 Coverage

Diff-backed:

- `diff::leader::tests::the_merchant_slots_are_the_original_s_own`: every
  Merchant slot of nine captures, by content. It was made to fail on
  purpose by zeroing the sum, which parts 835 blocks.
- `diff::leader::tests::run114_s_offers_are_the_original_s_own`: the
  Merchant on 9380 against the original's own row, across the city
  shift.
- `diff::harness::tests::run123_s_word_frame_is_widened_whole`: slot 1 on
  11182 and 11185, and the one key left parting on the word's old blocks,
  keyed on `GREAT_LAKES_MARKET_BLOCK` now that the headline has left
  them.
- The `known_rares` row, on the seven leader windows.

Unit: `ai_census::tests::step_9_counts_the_seen_rares_a_merchant_may_reach`
(ground ownership, a mutual and a one-sided ally, the zero before the
count, the sum waiting for the recompute).

Reading-only, and owed a blind second reading: §55.1's explored test and
its two leader-flag exits, §55.2's Mathematics-or-knowledge gate (the
listing is quoted above), and §55.3's writer.

## 56. The Mine that was a price: `get_cost`'s military discount and research arm (2026-09-22, item 545)

Item 539 left Great Lakes' word on **11582**: ours 948 draws against the
original's 9, parting at index 5, ours `Leader::produce_building+0xc99`
against `Guy::set_anim+0x104b`. Its block, 11583, is in run125 (38 rows over
11580–11583). Ours placed a Mine (`1/2022`) and walked citizen `1/9` to it.
The original queued the **Phalanx** at `1/2016` for 90 food and 54 metal.
The brief named no mechanism and pointed at `MAKE` slots 2 and 3, swapped on
11580. The swap was a consequence two steps down. The cause was two arms of
`TypeData::get_cost@00664090` this crate had never built.

**Great Lakes' word goes 11582 → 11757.** East Indies' holds at 9711, and
the golden chapters hold at 900 and 664.

### 56.1 The swap, read backwards

Walking the make list over run125 (`RON_LEADER_WALK=MAKE[`): the two lists
agree through 11579. On 11580 both sides offer Phalanx (`t133`) and Militia
(`t66`) from `upgrade_units`' military arm (`cat 7`). The Phalanx is 838,656
on both sides. The Militia is **209,664** in the original and **52,416**
here, exactly a quarter, so it sinks below slot 3's Mercantilism. Three
frames later the Mine (77,000) takes rank 3 here, above our Militia and
below the original's. `make_stuff` buys it and `produce_building`'s site
search spends the 948.

The quarter is the arm's last factor: `wm(aff, v / 256)` with `aff = 0x100`
when `type_affordable ≥ 1` and `0x40` otherwise. Militia costs 80 food and 80
metal. On 11579 this crate held **79** metal and the original **82**.

### 56.2 Three metal, from 11183

The metal gap is flat from 11250 (`bucket[4]` 50/53) back to run123's
**11183**, where it opens: ours 89 → 44 and the original's 89 → 47. That is
the Hoplites purchase `docs/ECONOMY.md` §14.4 left as "`get_cost`'s row",
65/45 against 61/42. The food gap has the same shape: 7 from 10783, the
Horse Archers of §13.4 (60/40 against 57/38).

Every one of those four pairs is the listed price less 5%, truncated.
`docs/COSTS.md` names `MILITARY_UNIT_DISCOUNT`, 5% per Military level the
player holds above the unit's own. `Tuning` loaded it and nothing applied it:
`Sim::price_of` passed empty modifiers, and `Modifiers::late_discount` had
no writer.

The decompile, `get_cost:633`–`656`, for a unit whose `role & 0x10000` is set:

- `level = max(military_level, 1)`;
- `ahead = epoch[0] − level` (`data_encrypted +0xe8`, the Military line);
- when `ahead > 0`, `pct = MILITARY_UNIT_DISCOUNT × ahead`, scaled by
  `(span × pct + 7) >> 3` and floored at 1 when the scenario spans fewer
  than eight ages;
- `cost = (100 − pct) × cost / 100`, clamped at 0.

This comes after the ramp and before Monarchy, Socialism and Salmon.

`military_level` (`UnitTypeData +0x2dc`) is written once, by
`Types::finalize_grafting@00669840`. It caches
`UnitTypeData::get_military_level_slow@0061d4d0` for every unit type. That
function takes `preq[0]` if it is a Military library tech (`is_epoch_type`,
tech `cat` 0), else `get_preq(1, −1)`, which is `preq[1]` in a game that
starts Ancient and ends Information. The level is the tech's
`TypeIndex − 0x23b` (the listing's `leal -0x23b(%esi)` at `0061d55b`), so the
line's first tech is level 1. This crate's loader already gives the Phalanx
`preq = [Classical Age, the first Military tech]`. So
`TechTree::military_level_of` is this crate's zero-based `level` plus one.

With the discount in, `bucket` food and metal agree on **every block of
run123 and run125** (10760–11599). The make lists agree through 11582.
`MAKE` slots 2 and 3 no longer part.

### 56.3 The research the frame asked for

That moved the delta from 948 draws to 11 against 9. Both sides now buy
slot 2, the Phalanx. The original queues it at `1/2016` and pays 90/54.
This crate refused, then bought the Militia, and paid for a third
`make_stuff+0x63d` expiry. A probe in `produce_tech` showed why:

1. **The route.** The Phalanx is not owned (`type_avail` 2), so
   `make_this` calls `produce_tech`, on both sides. The original's
   `produce_tech@006ca980` ends in `Build::queue_up(t, escrow)` (`:262`),
   the one queue for every kind. This crate's ended in `queue_tech`, which
   takes only a technology and answered `CantTrain`. Over the whole run to
   11599, 11582 is the first unit research the AI ever attempts.
2. **The price.** An unowned type is priced by `get_cost`'s **research**
   arm (the `leader + 0x6c18` bit clear, this crate's `PlayerTech::tech`,
   and the type without flag `h`, `unit_flags & 0x80`), and `price_of` had
   none. The arm, `get_cost:425`–`534`:
   - the scaled base;
   - `× RESEARCH_PREMIUM >> 8` (256, the identity);
   - `× RESEARCH_PREMIUM_COST >> 8` (`UnitTypeData +0x2e0`, 512 for 355
     records);
   - the refit surcharge;
   - `MILITARY_UPGRADE_DISCOUNT`, 10% per level, **without** either of the
     unit discount's floors.

   It replaces the ramp. For the Phalanx: 5f/3m × 10 × 2 is 100/60, less
   10% at `epoch[0]` 2 against level 1: **90/54**, the original's queue
   record.

The refit walk is **the predecessors**. `get_cost:441` compares `p`
with `unittypes[this].from`, then walks `p`'s `JUMP` chain looking for this
type. `docs/COSTS.md` had it as "every `p` whose `FROM` resolves to this
type", which is the available arm's walk, and is corrected in place. For the
Phalanx the refit is zero: Hoplites cost what it does.

`produce_tech` now sends a unit type to `queue_up`, and `price_of` takes the
research arm for a tree type the player does not own.

### 56.4 What it moved

The word's block, 11583, is pinned in `run125_s_word_frame_is_widened_whole`
as the move's value diff. It goes from 38 rows to **2**, both `1/0`'s figure
on 11580, which spends no draw.

| instrument | before | after |
| --- | --- | --- |
| Great Lakes' word | 11582, 948 draws / 9 | **11757**, 7 / 8 |
| East Indies' word | 9711 | 9711 |
| run125 keys parted, `[11250, 11599]` | 403 | **288** |
| run125's 11580–11583 rows | 38 | **2** |
| run123 `bucket` food / metal disagreeing | from 10783 / 11183 | **none** |
| run123 11185 inputs, timber / metal | 1/6, 44/47 | **6/6, 47/47** |
| run100's Stable row (10782, 60 / 57) | 1 | **0** |
| run107 leader residue | 90 | **85** |
| run111 leader residue | 93 | **89** |
| Great Lakes market draws below the word | 16 | **17** (11582) |
| Great Lakes endpoint off / extra / build-diverged | 46 / 3 / 10 | **47 / 2 / 9** |
| East Indies endpoint off / unlinked | 64 / 9 | **67 / 3** |
| East Indies ladder C extra | 15 | **21** |
| East Indies ladder B off / extra | 49 / 14 | **48 / 17** |

The residue falls are all `MAKE` rows on the military offers. They sit below
the word and only fall. The endpoints are 12,000 frames past both words.

**The ladder's rung check was widened, not weakened.** It required an object
number `extra` on both rungs to be one type. Rung C (15,401) holds Slingers
`1/66`–`68`, and rung B (16,489) holds them as Javelineers. That is the first
unit upgrade this crate ever researched, and `Unit::set_type` re-typed the
standing three. The check now accepts a later type that descends from the
earlier through `FROM`. A number recycled into an unrelated type still
parts.

**The new word**: 11757, ours 7 draws against the original's 8, parting at
index 0. Ours is `Guy::set_anim+0x97a < Guy::inc_time+0x271` and the
original's `Guy::set_anim+0x97a < Unit::move_step+0x823`: a figure clocked
from `inc_time` here and from a move step there. It is 158 blocks past
run125's last, so its widening is owed a capture.

### 56.5 What this has *not* established

- **The research arm's other terms are not carried.** ~~Wine's
  `WINE_UNIT_UPGRADES` before the premium~~ (built, §94), `SPECIAL_UPGRADE` (every one of
  the 364 records has an empty `<UPGRADE/>`), and the American and Dutch
  discounts after the military one. None is loaded.
- **The available arm's bump loop** (`get_cost:159`–`205`): the old unit
  charged the new one's base while the upgrade is queued. It is unbuilt, and
  now reachable, since this crate queues upgrades.
- **The age-span scaling** of both discounts, and `get_preq(1, −1)`'s scaled
  arm, are read and not exercised. Every capture is Ancient to Information.
  The span arithmetic is built. The scaled `get_preq` is not.
- **Only one research purchase is diff-backed**: the Phalanx on 11582. The
  refit surcharge is zero there and is backed by a unit test alone.
- **`MAKE[3].val` 38,500 against 37,500 on run123's 11185** survives. It is
  the Mine's value, a building. §55.5 filed it under §14.4's `get_cost` row,
  and it is not the unit discount.
- **The rest of the common tail**: Coal, Sugar, Gold, Iron and Gypsum by
  resource, and the Supercollider and Indian terms. `late_discount` carries
  only the military discount, so the fold is exact until one of them is
  built.

### 56.6 Coverage

Diff-backed:

- `diff::harness::tests::run125_s_word_frame_is_widened_whole`: 11583's
  rows, the Phalanx research at `1/2016` for 90/54 on both sides, and
  every leader and unit record over `[11250, 11599]`.
- `diff::harness::tests::run123_s_word_frame_is_widened_whole`: the 11185
  inputs. Food and metal agree over all 700 blocks.
- `diff::harness::tests::run100_s_word_frame_is_the_original_s`: the
  Stable's Horse Archers, 57/38.
- `diff::leader::tests::run107_s_window_is_the_leader_record_at_the_word`
  and `…run111_s_window_is_the_make_list_at_the_purchase`: the military
  offers.

Unit:

- `tests::a_military_unit_is_priced_by_its_research_until_owned_then_at_the_military_discount`:
  Great Lakes' numbers, both floors, and no surcharge below the level.
- `tests::a_research_is_charged_for_the_army_it_refits`: the predecessor
  walk, the halving, and the count.
- `ai_research::tests::produce_tech_queues_a_unit_s_research_on_the_unit_queue`.

Reading-only, and owed a blind second reading: §56.2's
`get_military_level_slow` fallback to `preq[1]`, and §56.3's refit
direction, read at `get_cost:441`.

## 57. The sea branch's dock is a search around the city (2026-09-23, item 576)

Item 573 left East Indies' word on **9983**, a `make_stuff` on both sides
that buys different things. On block 9984 the original queues units
(`1/2010` two, `1/2014` one, `1/2005` one). This crate queues two at
`1/2005`, places a Mine, `1/2017`, and walks citizen `1/6` to it: ours 212
draws against 15, parting at index 5 on `Leader::produce_building+0xc99`
against `make_stuff+0x63d`. No mechanism was named. run99 carries the word
at `LEADERS=1`, whose stub prints no make list, so the item took **run139**:
the same game at `LEADERS=9` over [9960, 9999] (`docs/RUNS.md`).

**East Indies' word goes 9983 → 10232.**

### 57.1 The lists, read forwards

`run139_s_word_frame_is_widened_whole` compares every key of the leader
record, both leaders, on every block. `RON_MAKE_SLOTS=<lo>-<hi>` prints
player 1's two lists slot for slot. The dump's `production_step` is the
step **after** the one that ran, and run139's coverage names each frame's
producer: `upgrade_units` on 9980, `create_units` on 9981,
`create_buildings` on 9982 and `make_stuff` on 9983.

- Both lists are empty on block 9979.
- `research_techs` and `upgrade_units` fill them alike, slot for slot,
  through block 9981.
- On **block 9982**, `create_units`' frame, the original's list holds three
  ships at the Dock, each `val 9999999`, city 0, cat 6: types 340 (slot 2),
  334 (slot 3) and 323 (slot 6). This crate's holds none, so Mercenaries and
  Militia keep slots 2 and 3.
- On block 9983 `create_buildings` puts the Mine in this crate's slot 3,
  where the original's ship stands, and `make_stuff` buys it.

The stockpile agreed throughout (one metal on 9966 and one food on 9981,
neither near a price edge), and the step machine agreed block for block.
The stanza wrote four readings before the run. The one that held is that
the lists part at the unit step (its R1). The Mine was a consequence, and
the purse and the clock were not involved.

### 57.2 The dock

`create_units@006c40a0:612`'s sea branch finds its dock with

```
find_building(city.x, city.y, SEARCH_FRIENDLY, who, 0x1800, 0,
              FILTER_BASE_TYPE, DOCK, 0)
```

and then walks that dock's footprint for a sea region. This crate searched
the **city's own chain**, on the note that "the city chain is where a dock
of this city always is". East Indies' Dock `1/2010` has `city -1` on every
block run99 and run139 print, so it belongs to no city, and this crate's
sea branch never offered a ship.

`ObjectsData::find_building@0065d260`, as far as this call reaches it:

- The radius `0x1800` takes the **bounded arm**. `(0x1800 + 0x2ff) /
  0x300` is 8, and 8 ≤ 9. The arm walks the object chains of the cells at
  `circle_x/y[..circle_radius[8]]` around the point's cell (`div_3_table[x
  >> 8]`, 768 units).
- A candidate passes `SubObjectData::is_active` and `WallData::is_active`
  (Build vslots `+0xc` and `+0x4c`, from `vtables.txt`), the friendly search
  and the base-type filter.
- The distance is `vector_dist` in **tiles**. The decompiler prints the call
  with lost arguments. The listing, `0065d471`–`0065d4b5`, shifts both
  operands `sar $6` through `div_3_table`.
- The winner test is `<=` against the running best **and** `<= 0x1800`.
  The second never binds inside eight cells. A tie goes to the last
  candidate in walk order.

`Sim::find_dock_near` is that search. `sea_value` calls it instead of the
chain walk. The rest of the branch is unchanged.

### 57.3 What it moved

With the search in, both lists agree **slot for slot on every block of
run139**. Only `MAKE[*].city` still parts, and that is §52.2's index shift,
not a value. The 17 rows on 9982 become 7 city rows, and the 38 on 9984
become 0.

| instrument | before | after |
| --- | --- | --- |
| East Indies' word | 9983, 212 draws / 15 | **10232**, 33 / 34 |
| run139 keys parted, [9960, 9999] | 312 | **187** |
| run99's rows on 9982..9984 | 27 | **0** |
| run99's floor under the word | 141 | 219 (the word moved 249 frames) |
| East Indies endpoint off / unlinked | 69 / 2 | **45 / 22** |
| East Indies endpoint build unlinked / diverged | 1 / 30 | **5 / 18** |
| East Indies ladder C off / extra / build diverged | 43 / 23 / 8 | **40 / 7 / 4** |
| East Indies ladder B extra | 21 | **1** |
| Great Lakes' word, golden chapters | 12038; 900, 900, 900, 1416 | held |

The endpoints are 13,769 frames past the new word and are the number, not
a trade (DECISIONS 36). The ladder's shared-extras guard fell from six to
one: the sixteen and twenty spurious units the rungs shed took most of the
overlap it cross-checks. It is lowered with the number, as its comment
asks, and is now all but vacuous.

**The new word**: 10232, ours 33 draws against the original's 34, parting
at index 30. Ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` and
the original `Unit::do_move+0xe84`. It lies in run99 (block 10233) and past
run139. run99's widening now pins its three rows, all on `1/32`: `form_mod`
ours -1 against 50, `idle` 5 against 0, and `stance` 0 against 3. No
figure changes animation on one side only on 10231..10234.

Between the old word and the new, 78 keys part, and none of them spends
a draw:

- the seated Scholars `1/30` (10127) and `1/31` (10149, with its group on
  10150), each with the four-row birth record as before;
- every Citizen's `myhits` and `hits_left`, 40 against the original's 50,
  and `mylos` 2 against 4, on **10165**;
- `1/32`, born on 10187 at (44184, 42888) against (45192, 41880), and in
  group 68 against 66 from 10188.

### 57.4 What this has *not* established

- **The other dock readers.** `create_buildings`' dock value (§2.19 item 2)
  and the transport code may look for a dock the same way. Only this call
  was checked against its listing.
- ~~**`dock_sea_region` walks cells, and the original walks tiles.** The
  original walks `WallData::tile_corner` over the footprint's `x_size ×
  y_size` **tiles**, testing the surface bits `& 0x30 == 0x20`, then
  `get_tregion`. This crate's walk starts at `pos.cell()`. It found the same
  sea here. It is not read further.~~ Item 579 walks the tiles
  (`dock_sea_tile`); the tile is the navy's point (`docs/ORDERS.md` §25.3).
- **The tie order.** No capture puts two docks at one distance. The walk
  order is the circle's, with building index order inside a cell standing in
  for the object chain's order.
- ~~**`Armies::init_navy`** runs on the original's 9981 (run139's coverage).
  This crate's sea branch still seeds no navy (`ai_census.rs`). Nothing in
  run139's window reads it.~~ Item 579 seeds it, and it was the word 10232:
  the navy's first order is what the original's Trireme gave up
  (`docs/ORDERS.md` §25.3).
- **10165's hit points**: a Citizen upgrade or an age effect the original
  applies and this crate does not. It is below the word and spends no draw.
- **`gather_stamp` on 9992**: ours 9991 against the original's 9719. This
  crate's gather clock stamps where the original's has not moved for 272
  frames. It spends no draw in the window.

### 57.5 Coverage

Diff-backed:

- `diff::harness::tests::run139_s_word_frame_is_widened_whole`: both make
  lists slot for slot, the stockpile and step, and every record, over
  [9960, 9999].
- `diff::harness::tests::run99_s_word_frame_is_widened_whole`: the move's
  value diff on 9982..9984 (empty) and the new word's rows on 10233.

Unit:

- `ai_units::tests::the_sea_branch_s_dock_is_the_nearest_of_mine_around_the_city`:
  a dock of no city found, the nearest of mine winning, a rival's and an
  unfinished one skipped, and the eight-cell edge. It was made to fail on
  purpose by widening the circle to ring 9.

Reading-only, and owed a blind second reading: §57.2's candidate filter
(the two `is_active` vslots) and the bounded arm's cell walk. The listing
settles the distance unit.

## 58. A citizen on a barge is counted by its container (2026-09-23, item 592)

Item 588 left East Indies' word on **10582**, player 1's `make_stuff`. The
make list parted on 10581: the original held a Citizen (type 50, `val
1714`) in slots 3 and 5, and this crate held two Scholars at `val 0` in
slot 3. The peasant census parted under it on 10576. The word spends **209
draws here against 207, parting at index 11**: ours
`Leader::produce_building+0xc99`, theirs `+0x1805`. No mechanism was named.
run143 (`LEADERS=9` over [10380, 10739]) answered everything, and no capture
was taken.

### 58.1 The census counts a rider through its container

Player 1's full sweep runs on sim-frame 10575 (§2.2's phase), and block
10576 is its census. Every citizen's order list, `peasants` 15, `gatherers`
12 and `active` 37 agree. One citizen is counted differently: `1/31`, which
boarded the transport barge `1/36` on 10486 on both sides. The dump shows
`inside_up 36`, and this crate `inside_unit 36`, `on_map false`.

`Leader::plan_strategy@006b9620`, the unit census (§2.3 step 10), read from
the decompile:

- **The region.** A unit with `inside_up ≥ 0` takes its region from
  `ObjectData::get_inside@00651a80`. That walks `inside_up` while the
  container `is_unit` (vslot `+0x18`) to the **outermost container**,
  building or unit. A unit of `domain != 1` takes that container's tile,
  with the coastal alternate-region rule. The dock scan of the footprint
  is only for a sea unit in a building.
- **The citizen branch, off the map.** The first arm is an oil platform
  (`is(0x1a6)`) → `filled_gather_slots[5]++`. The second arm fires when
  the container `is_unit`, its type's `domain` (`+0x218`) is **1**, its
  order list is non-empty, and the current order answers vslot `+0x14`.
  That vslot is `MoveOrder::is_move`, answered by every `MoveOrder` class:
  Move, AttackTo, ExploreTo, FleeTo, GroupMove, GroupAttackTo and Form.
  `get_move_order` (`+0xb8`) then gives `MoveOrder::x/y` (`+0x4/+0x8`).
  The region of **the cell** under that point (`x >> 8` through
  `div_3_table`, no `>> 2`) goes into the sweep's region local, and if it
  is `≥ 0`, `reg_xport_peasants[r]++` and `xport_peasants++`.
  **Neither arm counts the rider free, busy or in `reg_peasants`.**
- **The local is shared.** The `reg_active`/`reg_combat`/`reg_attack`
  block after the worker branch reads the same `WVar31` the second arm
  wrote. So the rider is `active` in the region the boat is going to, not
  the region it is in.

run143 prints all three. Until this item this crate counted the rider on
the map where it boarded: free, busy-less, in the home region 11. The rows
on 10576 were:

- `free_peasants` 2 against 1;
- `xport_peasants` 1 against 2;
- `reg_peasants[11]` 14 against 13;
- `reg_free_peasants[11]` 1 against 0;
- `reg_active[11]` 27 against 26;
- `reg_active[5]` 0 against 1;
- `reg_xport_peasants[5]` 0 against 1;
- city `1/2000`'s `free` 1 against 0.

Region 5 is the barge's destination, (40344, 22680). The barge's own tile
is the ocean, 65, where a land unit is not active. So `reg_active[5]` is
the shared local, printed.

`Sim::census_unit_region` now takes a boat's tile. `census_citizen` sends a
rider to the off-map arm, and `census_citizen_inside` returns the move
point's region, which `census_units` carries into the `reg_active` family.

### 58.2 The instruments this needed

- **The census's 21 per-region arrays were unread.** They sat in
  coverage's `UNREAD` pin, printed at `LEADERS=9` on every block and
  compared nowhere. `diff::leader::REGION_ARRAYS` reads all of them, keyed
  by the dump's own region through `Built::region_map`:
  - the land arrays at width 64;
  - `reg_active`, `reg_combat` and `reg_attack` at 127;
  - `reg_naval` and `reg_transports` at 63, indexed by region `− 0x3f`.

  `reg_terr` has no writer here and stays pinned.
- **`inside_up` was parsed and compared nowhere.** run143's widening now
  compares every unit's outermost container on both sides, and none parts
  in [10380, 10739]. `compare` itself still does not read it.

### 58.3 What it moved

- **The census agrees whole on 10576.** Both make lists agree slot for slot
  on 10581 and 10582, apart from §52.2's `city` shift and slot 4's `val`.
  The rows on the word's blocks go 9/6/98 → 2/2/98, and run143's floor goes
  273/309/772 → 273/301/753.
- **The word does not move.** 10582 still spends 209 draws against 207,
  parting at index 11, and the site differs:
  - both sides place a Mine (type 419, slot 4) and buy a Citizen;
  - this crate's spiral scores **12** friendless FARM/MINE candidates at
    `produce_building+0xc99` against the original's **7**;
  - it tries **4** sub-positions at `+0x1805` against **3**;
  - the Mine `1/2018` lands at y 34944 against 35136.

### 58.4 What is under it: city `1/2007`'s site picture

`1/2018` belongs to city `1/2007`, at (34656, 36192).

- **`MAKE[4].val` on 10582** is the Mine's value from `create_buildings`
  (§3.2's gather arm): 1,616,000 here against 1,584,000.
- **City `1/2007`'s step-13 picture** is old residue, and on file.
  `run58_s_five_thousand_frames_stand_where_the_original_s_do` pins its
  `land`, `filled` and three `space` counts one apart from **1819**, the
  frame its circle is first swept. On run99 the counts read:
  - `space[0]` and `space[1]` 59 against 60 from 7880;
  - `filled` 33 against 34 and `space[2]` 49 against 48 from 7976, the
    sweep of 7975.

  So `open`, and `reg_land[11]`, are one high here.
- **A payoff probe**, not kept, set the city's `filled` and `space[2]` to
  the original's from 7975:
  - `MAKE[4].val` and `reg_land[11]` closed;
  - **no draw moved.**

So the count explains the value but not the site.

**Hypothesis, not established:** one tile of `1/2007`'s circle is unfit
for a 4×4 square in the original and fit here (`check_building_wcoord`),
and the same tile admits this crate's extra spiral candidates. **What the
disk cannot answer:** no dump prints a tile's occupancy or a
`check_building_wcoord` result. A packet at 10581, with `produce_building`'s
spiral or `check_building_wcoord` run over the city's circle, would name the
tile. The count has parted since 1819, where run57's detail is on disk, and
the one-apart cell may have stood there since the city was founded.

### 58.5 What this has *not* established

- **The sea rider in a boat.** A unit with `domain == 1` asks its unit
  container a folded vslot (`+0x108`) before the footprint scan. No such
  rider exists here, and it takes the boat's tile.
- **A move point with no region.** The original writes `−1` into the
  shared local, and the `reg_active` block would index `−1`. This crate
  keeps the container's region. No capture reaches it.
- **The on-map `8`/`0xe` arm** (build or repair through a boarded
  transport) writes the same local. It stays a seam.
- **Only a plain `MoveTo` was diffed.** The other `is_move` classes are
  read, and all are `Body::Move` here.

### 58.6 Coverage

Diff-backed:

- `diff::harness::tests::run143_s_word_frame_is_widened_whole`:
  - the census and its 21 region arrays, both leaders, every block;
  - every unit's container;
  - the move's value diff on 10576, now empty;
  - the word's blocks, and the floor.
- `diff::coverage::every_key_the_dump_prints_is_read_or_pinned`: the
  region arrays leave the pin.

Unit:

- `ai_census::tests::a_rider_is_an_xport_peasant_where_its_boat_is_going`:
  - a rider on a moving barge counts xport in the destination's region and
    active there;
  - on a barge at rest it counts in the sea and nowhere active.

  It was made to fail on purpose by sending the rider down the on-map arm.

Reading-only, and owed a blind second reading:

- the `is_move` class list (only a plain `MoveTo` is diffed);
- the cell (not tile) indexing of the move point. It agrees on run143's one
  case, and one case does not separate the two.

## 59. A mine's reach is measured to the nearest solid mountain cell (2026-09-23, item 597)

East Indies' word **10582** is `Leader::produce_building`'s placement of
Mine `1/2018` (type 419) for city `1/2007`. It spent **209** draws against
**207**, parting at index 11: ours `+0xc99` against theirs `+0x1805`. The
spiral scored **12** friendless candidates against **7**. No dump prints
why a tile is fit, so this item took the lab's packet rung, its first use on
this map.

### 59.1 The packet, and what it answered

run144 (`docs/RUNS.md`) is a `RON_STATE_FRAME` packet at **logger frame
10582**, after trace tick 10581. That frame is the one before the
placement: Mine `1/2018` is first printed on block 10583, and block N is the
state after tick N−1 (the lab's Great Lakes packet reads logger 11,186
after trace 11,185). The packet is the same game: 10,591 seeds identical
to run54, and 8 blocks identical to run143.

`Leader::plan_strategy@006b9620` was entered on `leaders.list[1]`
(`0xE4127C`; `leaders` is `0xE3A390`, the listing of
`Leaders::strategy_all@006ed430`). Every call `produce_building@006e1400`
made was logged under unicorn:

- `produce_building(419, 2007, 1)` from `make_this+0x328`;
- **7** `Random::get` at `+0xc99` and **3** clear sub-positions of 4 at
  `+0x1805`, which are the original's own counts, so the packet reproduces
  the frame.

Each spiral cell's record, compared with this crate's (a temporary trace,
not kept), splits the five extras exactly. At all five,
`BuildTypeData::blocked_site@00636a50` returns **12**, `NoMountain`, where
this crate returned `Clear`:

| cell | site | packet `find_nearest` |
|---|---|---|
| (43, 44) | (33408, 34176) | 1536 |
| (42, 44) | (32640, 34176) | 1536 |
| (40, 46) | (31104, 35712) | 1536 |
| (40, 47) | (31104, 36480) | 1536 |
| (50, 48) | (38784, 37248) | 1536 |

Nothing else parts: 68 cells refused `NoMountain` on both sides, and the
`Building`, `Mountain`, `Water` and `Rare` refusals agree.

### 59.2 The arithmetic: cells, not tiles

`blocked_site` on each site, with `calc_gather@00639e40`'s calls logged,
shows `MountainsData::find_nearest@0089cd30` returning a range at all five.
Its distance out-parameter is **1536** against a reach of `gather_radius ·
0xc0` = 6 · 192 = **1152**, so `gather_size` is never reached. The accepted
sites read 1152, 984 and 576.

The listing settles what `find_nearest` walks:

- the count at `+0x7c` and the lists at `+0x88` and `+0xa4`, off the range's
  virtual base. Those are `MountainRangeData::solid_mount_wx`/`_wy`
  (`+0x74`, `+0x90`): **cells**, not `mount_tx`;
- each is measured at `(loc + off) · 0x300 + 0x180`, the cell's centre, by
  the inline `vector_dist`;
- a range whose first solid cell is in another region than the site's is
  skipped.

`docs/ECONOMY.md` "The mine's range" read "every tile of its range". That
was wrong: (43, 44)'s centre is two cells from range 2's nearest solid cell,
(43, 46), which is 1536, while a mountain tile of that range lies inside
1152.

`Sim::nearest_mountain_cell` now measures to cell centres, per region.
A cell is solid when its centre tile is a mountain. That closes (50, 48)
and puts the Mine on the original's site.

### 59.3 What the packet says the solid cells are, and this crate cannot yet

The packet's eighteen placed mountains carry **107** solid cells in three
template types: type 15 has 3 cells at offsets (−1, 0), (0, 0) and (−1, 1)
of its location, type 14 has 13 and type 13 has 14. Every range lies in one
region. Measured on East Indies:

- **centre tile a mountain**: 139 cells, the 107 and **32 more**. Range 2
  (loc (43, 46)) lists (42, 46), (43, 46) and (42, 47). This rule also
  admits (42, 45) and (41, 46), which reach the four remaining sites at
  1152 and 768;
- **all sixteen tiles a mountain**: 52 cells, 55 missing;
- **`WData.flags & 0x10`**, which `Mountains::add_mountain@0089c2e0`
  stamps on each solid cell: 91 cells at the start dump and in the packet,
  17 missing and 1 extra. Something later clears the bit. A payoff probe on
  that rule, not kept, spent 6 draws against 7 and lost (48, 43);
- **`WData.solid`**, which the dump prints: not membership. (43, 46) is
  solid and prints 0;
- **no 1-, 2- or 3-tile subset of the 4×4** separates the sets. The best,
  (1, 1) and (2, 2), misses 4.

The lists are the templates' own (`MountainRange::init` loads them with
the art; the `h*`, `m*`, `s*` mountain files). ~~No dump prints them.
Reproducing them is its own mechanic.~~ `init` derives them from the alpha
of each template's image, and a `DUMP_ALL` head prints the placements. All
107 cells are reproduced: §60.

### 59.4 What it moved

- **The word does not move: 10582.** Its count now agrees, 207 against
  207, and the sequence still parts at index 11. The spiral scores
  **11** against 7 (the four residue cells of §59.3), and the jitter spends
  3 against 3.
- **The value diff on the word's frame** (run143, block 10583): Mine
  `1/2018` at y **35136**, the original's, where it stood at 34944, and its
  builder `1/6` on the original's (38207, 36557). The one-sided animation
  change of the human's `0/4` on 10585 is gone. The block's 98 rows are 80,
  all of them the Mine's gather list order (40 `tx`, 40 `ty`), which the
  shuffle draws from a stream four spiral draws apart.
- run143's floor goes 273/301/753 → **273/301/393**: nothing under the
  word, and 360 keys fewer past it.
- The Mine's `make_stuff` value (`MAKE[4].val`, §58.4) is unchanged. That
  is city `1/2007`'s `filled`, which is not this.

### 59.5 What this has *not* established

- ~~**Solid membership** (§59.3). This is the successor on the headline: the
  four sites (43, 44), (42, 44), (40, 46) and (40, 47) are refused at 1536
  in the original and reach (42, 45) or (41, 46) here.~~ Established in
  §60; the four are refused at 1536 here too.
- ~~**The region test** is made per cell.~~ (§60: on the first solid cell,
  with the placements.) The original makes it on a range's
  first solid cell. That is the same thing wherever a range lies in one
  region, which holds for all eighteen on East Indies. It is not read on
  Great Lakes.
- ~~**The tie-break** between two ranges at one distance: the generator's
  order there, rows here.~~ The generator's order, with the placements (§60).
- **Great Lakes 8382** (ECONOMY, the nine candidates) is not re-measured
  cell by cell. The map's word holds at 12038, so its draws agree.

### 59.6 Coverage

Diff-backed:

- `diff::build::tests::east_indies_10582_mine_sites_measure_to_the_nearest_solid_cell`:
  the packet's `find_nearest` distance at eight mine sites. Four agree, and
  the four that part are pinned as the residue with the cell this crate
  reaches. It was made to fail on purpose by measuring to the centre tile's
  centre: 1062 against 1152.
- `diff::harness::tests::run143_s_word_frame_is_widened_whole`: the site and
  its builder empty on 10583, the block counted, the floor.

Packet-backed and not in a test, because the packet stays outside git
(`~/ron-data/lab-experiments/2026-09-23-item-597/`): the per-cell
`blocked_site` codes, ~~the 107 solid cells~~ and their regions. The
107 cells are in a test since §60, which reads the packet's list at run
time.

Reading-only: the `+0x7c/+0x88/+0xa4` offsets as `solid_mount_wx`/`_wy`,
checked against the packet's lists (every range's cells sit around its
location). This is owed a blind second reading.

## 60. The solid cells are the templates' alpha, and the word moves to 10782 (2026-09-23, item 604)

East Indies' word **10582** held through two items. Each fixed a value under
it: 592 the census, 597 the Mine's site. On the word the spiral scored
**11** friendless candidates against the original's **7**. Four cells, (43,
44), (42, 44), (40, 46) and (40, 47), reached a solid mountain cell that no
range lists. §59.3 named the cause as `solid_mount` membership: the lists are
the templates', and no rule over this crate's tiles separates them.

### 60.1 What the loader reads

`MountainRange::init@008998b0` does not load a list. It derives both lists
from the **alpha channel** of the template's `TEMPLATE_TEX`, a 256 × 256
TGA. Four pixels make a tile and sixteen make a cell.

- A **tile** is the range's when all four of its corner pixels are set.
- A **solid cell** is the range's when more than fifteen of a 5 × 5 grid of
  samples over its block are set.

The template index is the `<MOUNTAIN>`'s position in
`effects_graphics.xml`. The format, the frame and the evidence are in
`docs/FORMATS.md`, "The mountain templates".

### 60.2 Where the placements come from

No reading of the map generator was needed. `GameLog::dump_mountains
@0092fca0` prints `MountainsData`'s location cells and template indices in
every `DUMP_ALL` head, and both maps' harnesses already borrow such a head:

- **East Indies** from run38: eighteen ranges, the packet's list exactly.
- **Great Lakes** from run3 and run12: thirteen ranges.

`gamelog::mountains_of` finds the four arrays by their shape and keeps them
only when the float locations are `768 ×` the cells.
`borrow_from_siblings` lends them on the world block's scalars, the same
test that lends the cells. `build_sim` then lays each template at its
location (`Sim::place_mountains`).

### 60.3 The walk

`MountainsData::find_nearest@0089cd30`, read whole
(`Sim::find_nearest_mountain`):

- **The order.** Placed ranges in the generator's order, then each range's
  solid cells in the template's order.
- **The tie-break.** The best is replaced only on a **strictly** smaller
  `vector_dist`, so a tie goes to the first range and its first cell.
- **The region test.** A range is skipped when its **first** solid cell's
  region is not the site's. With no region at the site there is no test.
- **The answer.** The function returns the placed index, which is
  `MiningList::mtn`. With the placements in, this crate names the dump's
  range for both of run80's Great Lakes mines (`mtn 6` and `mtn 0`).

`mountain_range` returns that range's template tiles, in the template's
row order, which is the order the gather list is shuffled from. It also
returns its solid cells, filtered as `gather_size` filters them. Great Lakes'
range 6 is template 9 (`m4`), with 244 tiles: the count 597's stand-in
component had.

A capture with no `DUMP_ALL` head and no such sibling keeps the stand-in:
the connected component, and "the centre tile is a mountain".

### 60.4 What it moved

- **The packet's eight sites agree.** `find_nearest_mountain` gives the
  packet's distance exactly at all eight: 1536, 1152, 984 and 576, and 1536
  at the four residue cells, which are now refused. The spiral spends
  **7** draws against 7.
- **The word: 10582 → 10782.** On 10782 this crate spends 10 draws against
  11, parting at index 5: ours `Animal::do_idle+0x83`, theirs
  `Leader::make_stuff+0x63d`. No capture reaches that frame. Item 608 owes
  its widening, on run149.
- **The value diff on the frame that moved** (run143, block 10583). Mine
  `1/2018`'s gather list had 80 rows (40 `tx`, 40 `ty`); it now has
  **0**. The spiral spends the original's draws, so the shuffle reads the
  original's stream. The widening's floor goes 273/301/393 →
  **273/301/313**: nothing under the word moved, and nothing parts anew
  past it.
- **The value diff on run143's last blocks, 10735..10739.** Each block had
  274 standing record rows and now has **194**. The 80 removed are exactly
  the Mine's gather list. On 10739 the other 194 are the same rows,
  value for value, on both trees. Among them are the human's city record,
  which this crate does not keep, `form`, and the hit and loss counts of
  `1/1` and `1/2`.

### 60.5 What this has *not* established

- **Great Lakes' solid cells** are not packet-checked. Its tiles are, by the
  union with the start dump, and so are its two `mtn` indices, and its word
  holds.
- **`init`'s other products**, the mesh, the heights and the texture arrays,
  are the renderer's.
- **A capture without a `DUMP_ALL` sibling** measures on the stand-in. Every
  headline capture has one.
- **The cliff arm** is unchanged (ECONOMY, "The mine's range").

### 60.6 Coverage

Diff-backed:

- `the_templates_give_the_packet_s_solid_cells`: all 107 of the packet's
  cells, in order, range by range. The packet is read at run time and never
  enters git. The test fails when the image is read bottom row first.
- `the_placed_templates_tile_the_map_s_mountains`: the union of the tiles is
  the start dump's mountain tiles, on both maps.
- `east_indies_10582_mine_sites_measure_to_the_nearest_solid_cell`: the
  packet's `find_nearest` at eight sites. All eight agree.
- `great_lakes_first_mine_lists_its_mountain_range`: run80's two lists, 207
  and 232 tiles, and their `mtn` indices.
- `run143_s_word_frame_is_widened_whole`: 10583 empty, and the floor.

Reading-only: the XML-to-slot order in `Mountains::init` (the first
`<MOUNTAIN>` child is the image `init` reads). The packet's type indices
13, 14 and 15 confirm the positional index on three templates.

## 61. A city counts an unfinished gather building's slots, and the word moves to 10982 (2026-09-23, item 608)

East Indies' word **10782** was past every capture. run149 is run143's
line over [10730, 10879] at `LEADERS=9` (`docs/RUNS.md`), and its
widening, `run149_s_word_frame_is_widened_whole`, walks every record of
every unit, both leaders, and gaia's animals.

### 61.1 The frame

On 10782 this crate spent 10 draws against 11, parting at index 5: ours
`Animal::do_idle+0x83`, theirs `Leader::make_stuff+0x63d`. The streams are
**one stream shifted**. Both sides spend `make_stuff+0x63d` at index 4, the
original a second one at index 5, and the same animal roll follows. Gaia's
`8/3` then turns on 10783 in the original only. It is downstream, not the
cause.

The widening names the make list on block 10781, `create_units`' frame.
The original offers the Citizen (type 50) with **`num 4`** at `val
3291428`, and this crate with `num 1` at 205714. The value squares the
count (§52), so the original ranks the Citizen in slot 1, above the Dromon
and the Galley, and duplicates it into slot 5. `make_stuff` on 10782 buys
the four, then expires two Citizen slots (two `+0x63d`) where this crate
expires one. City `1/2007` queues 4 against 1.

### 61.2 What sized the count

`num` is the city's gatherer deficit, bounded by the room its caps leave
(§53). The deficit is taken against the city's gather slots. Only two
keys first part before the word, both on block 10776:

- player 1's `reg_gather_slots` of its home region (11 in the dump's
  numbering), 17 here against 20;
- `SITE[7].val`, 26 against 13, which spends no draw (below).

10776 is the first census sweep after the Mine `1/2018` was placed on
10583. No sweep runs between, on either side, so the region's count
stayed 17 on both until then. The per-good `gather_slots` do not move on
10776 on either side. They are the census's own building walk, and why
that walk leaves the Mine out is not established here.

`City::count_gather_slots@00737dc0` walks the city's chain from `City+0x8`
along `BuildData::city_down` (`+0x74`). For each building it asks the type
`+0x90` ("is a gather building"), maps the type index to a good, and adds
`gather_max` (`+0x80`). The listing (`737e50`–`737f28`) has nothing else:
**no completion test**. This crate skipped a building that was not
`active`, and it has done so since the function was first ported
(`2857f10`, no evidence cited).

### 61.3 The packet

The decompile alone left one question: whether `+0x80` is zero until a
building starts. `Build::init` and `find_gather_tiles` both write it at
placement, but a region flag this crate does not model can rewrite it
(`Build::process`, ECONOMY "The mine's range"). **run150** answers it: a
`RON_STATE_FRAME=10765` packet, after the Mine's placement and before
`Wall::start` runs on 10766.

- The Mine reads `flags 1` (placed, not started), `frame_started -1`,
  **`gather_max 3`**, and a 46-tile `gather_from` list.
- City 1's chain is 2007 → 2009 → 2011 → 2012 → 2014 → 2015 → 2017 → 2018.
  Its gather slots are 4, 1, 1, 7 (the University, knowledge, excluded)
  and the Mine's 3.
- Leader 1's `reg_gather_slots[11]` is 17, the value of a sweep that ran
  before the placement.

So the original counts the unstarted Mine as soon as a sweep runs, and
this crate never did until the Mine was finished.

### 61.4 What it moved

- **The word: 10782 → 10982.** On 10982 this crate spends 14 draws
  against 695, parting at index 2. The original places a gather building
  and shuffles a 170-tile list (`Build::find_gather_tiles+0x10a` × 680). It
  scores the spiral `produce_building` 4 `+0xc99` and 4 `+0x1805` against
  this crate's 7 and 1. No mechanism is named. Item 613 owes its widening
  (run152).
- **The value diff on the frame that moved**, run149's blocks 10781..10783:
  **20 rows → 1**. The row left is §52.2's `city` shift (`MAKE[3].city` 2
  against 1 on 10782). `reg_gather_slots` is gone from 10776, and gaia's
  `8/3` turns on both sides. The widening's floor goes 280/283/506 →
  **280/282/291**.
- **On run149's last block, 10879**: 427 standing rows → **280**. The fix
  removes 148: 84 of player 1's (the four Citizens, the leader and city
  `1/2007`), 31 of the human's `0/3` and `0/4`, and 33 of gaia's animals.
  It adds one, the same `city` shift. The other 279 are the same rows,
  value for value.

### 61.5 What this has *not* established

- **`SITE[7].val` on 10776**, 26 against 13, stands. It parted on the same
  sweep before the fix and spends no draw through 10879.
- **`open`'s clamp.** The original writes `param_2[g] += gather_max −
  num_gatherers` with no floor. This crate floors each building's share at
  0. No run has reached an over-full building.
- **Great Lakes' mines** go through the same walk. Its word holds (the
  gate, below), which says nothing about a site that has not yet been
  placed there.

### 61.6 Coverage

Diff-backed:

- `run149_s_word_frame_is_widened_whole`: the census on 10776, the word's
  blocks, the one-sided animation changes, and the floor.
- The long word `run54_s_24000_frames_are_where_the_second_map_s_word_now_parts`:
  10982.

Packet-backed and not a test: run150's `gather_max 3` on the unstarted
Mine. The packet stays outside git.

Reading-only: that `count_gather_slots` has no other gate. The listing
settles it for this function, and no second reading has been booked.

## 62. Horses cheapen the Light Horse, a mined range is taken, and the word moves to 11069 (2026-09-23, item 613)

East Indies' word **10982** was past every capture. **run152** is run149's
line over [10870, 11039] at `LEADERS=9` (`docs/RUNS.md`). Its widening,
`run152_s_word_frame_is_widened_whole`, walks every record of every unit,
both leaders and gaia's animals on 170 blocks. The floor, committed before
any fix was judged, was 278/409/616.

### 62.1 The frame, read backwards

On 10982 the original spent 695 draws: `use_market` ×3, `make_stuff+0x221`
×2, the spiral 4 `+0xc99` and 4 `+0x1805`, the shuffle of a 170-tile list
(`find_gather_tiles+0x10a` ×680), then `make_stuff+0x63d`. This crate spent
14 and parted at index 2, on `use_market`'s third draw.

The widening (`RON_MAKE=<lo>-<hi>` prints both lists slot for slot) puts the
first parting on block **10981**, `create_units`' tick 10980. Every other
slot agrees, and the Light Horse (t209, `num 2`, cat 6) is offered at
**`val 9999999`** here against **6945568**. On 10982 the original's
`make_stuff` then sorts the Barracks (7222784) above the Light Horse, so
`use_market`'s `need`, the first `max(1, epoch[Commerce])` = 2 slots,
differs, and a third sale pass follows.

### 62.2 The offer's tail: `check_income` 64 against 256

`create_units`' land-military tail (`006c40a0:1554`–`1568`) is
`out = check_income × (want × val / divisor) >> 8`, and `out < 0` becomes
9999999. A temporary probe read this crate's terms on tick 10980:
`fac 64, want 3, val 40,500,000, divisor 3`. 64 × 40,500,000 wraps
negative. With the same terms, `fac 256` gives 256 × 40,500,000 =
10,368,000,000 ≡ 1,778,065,408 (mod 2³²), and ÷256 is **6,945,568**, the
dump's value exactly. So the original's `check_income` answered 0x100.

`check_income(t, 0x400, local_74, escrow, −1, num, &afford)` with escrow on
answers 0x40 only when `can_pay_cost < num`. `can_pay_cost@00667570` is the
maximum over the goods of `bucket / price`, returning 0 at once when any
quotient is 0 (`docs/COSTS.md`, "The affordability count takes the
maximum"); this crate's `cost::affordable` agrees. Player 1 held 53 food and
70 timber, and the Light Horse is 60/40 here. The original must have priced
it at ≤ 53 food and ≤ 35 timber, a discount of at least 12.5%.

`epoch` agrees on both sides, so `MILITARY_UNIT_DISCOUNT` gives nothing.
The discount is `get_cost@00664090:320`–`331`, in the pre-ramp tail after
the nations and Terra Cotta:

- a unit whose trainer (`UnitTypeData +0x40`) is **exactly** the Stable
  (`0x1ac`) or the Auto Plant (`0x1ad`);
- `rare.ptr[1] & 1` or `rare_conquest.ptr[1] & 1`, bit 8, which is good 14,
  **Horses**: `cost = (100 − HORSES_STABLE_COST) × cost / 100`;
- then `& 2`, Rubber, `RUBBER_AUTOPLANT_COST`, **on both kinds too**.

`rules.xml` ships both at **15%**: 60/40 → **51/34**, `max(53/51, 70/34)` =
2, which is `num`. The dump's `rares_collected` (rewritten by every
`calc_gather` sweep, so fresh on the frame) has player 1 on Fish, Horses,
Citrus and Whales. This crate's `rare` mask is the same, `0x2100101`. This
crate held Horses and applied neither arm: `price_of` set only the
military discount.

`Sim::stable_rare_discounts` gives the two percentages. They ride
`cost::Modifiers::stable_rares`, each truncating on its own right after the
pre-ramp `discount`, before the research arm and the ramp.

### 62.3 The spiral: a range another Mine gathers from is taken

With the price in, the make list agrees through 10983 and the frame parts at
index 9. Both sides place the Mine `1/2019` from slot 4, and this crate's
spiral scores **7** friendless candidates against the original's **4**. A
temporary probe of each candidate's `find_nearest` put the seven on two
ranges. Three and the anchor cell (43, 48), this crate's pick, reach
**range 2**, which the Mine `1/2018` already mines (`mtn 2` in the dump);
four reach range 0. The original's pick is (47, 45) on range 0, the
best-scored survivor (one friend), jittered to (36480, 35136).

`BuildTypeData::calc_gather@00639e40:569`–`760`, the mountain arm: for a
survey (no `MiningList`), after the reach test it walks the range's tiles
in the template's order. It skips a tree, a tile that is not a mountain,
and one in a non-allied enemy's cell, and at the first that is
`is_gathered_from` it sets `mtn = −2` and the count to **−1**.
`blocked_location@006375b0:714` reads `< 0` as `MOUNTAIN_TAKEN 0xd`
(`docs/CITIES.md` §2.6.7). This crate had the verdict mapped and never
produced the −1. `gather_slots_raw` clamped `mine_slots` at zero, and
`mine_slots` did not walk the tiles.

### 62.4 What it moved

- **The word: 10982 → 11069.** On 11069 this crate spends 5 draws against
  6, parting at index 0: ours `Guy::set_anim+0x97a`, theirs
  `Unit::do_move+0xe84`. No mechanism is named. Item 620 owes its widening.
- **The value diff on the frame that moved**, run152's 10981..10983: the
  make list's 13 rows → **2** (§52.2's `city` shift, and `MAKE[4].val`,
  the Mine's value, 316800 against 310400, which spends no draw); the word's
  block, **21** unit rows → **0** (the Mine `1/2019` on the original's site
  with its 170 tiles, and the citizens `1/11` and `1/36` on the original's
  orders). The one-sided turn of `1/11` on 10983 is gone. The floor goes
  278/409/616 → **278/409/412**: nothing under the word moved.
- **On run152's last block, 11039**: standing rows **521 → 370**. 163 go
  (27 of `1/36`, 23 of `1/11`, 20 of the human's `0/5`, 17 leader rows,
  and the rest spread over player 1's units and gaia). 12 come in:
  `bucket[4:metal]` 54 against 55, `leftover[4:metal]`, two `city` shifts,
  and eight rows of `1/11` a few units off (34089, 33604) against (34104,
  33575). The other 358 are the same rows, value for value. Both trees
  were measured.

### 62.5 What this has *not* established

- **Under the word, 409 keys stand**: `bucket[0:food]` a unit off from
  10881 and `bucket[4:metal]` from 10900, `peasants` 15 against 16 and the
  unit `1/36`'s hit points (40 against 50) from 10956, `1/2018`'s gather
  list order and `1/11`'s tile from 10959, and the SITE list on 10976. None
  spends a draw before 11069 on this tree.
- **Rubber, and Horses on an Auto Plant unit.** No capture reaches either.
- **The rest of the pre-ramp tail** (Salt, the nations, Terra Cotta, Angkor
  Wat, Sulphur, Wool, Aluminium): still unbuilt, and each is a price that
  moves a `check_income` the same way.
- **`get_cost`'s context argument** (`local_74`) is still dropped. Nothing
  on this frame needed it.
- **The cliff arm's own taken walk** is not built, and neither is the cliff
  arm (`docs/ECONOMY.md`, "The mine's range").

### 62.6 Coverage

Diff-backed:

- `run152_s_word_frame_is_widened_whole`: the make list and the Mine on
  10981..10983, both mechanics' value diff, and the floor.
- The long word `run54_s_24000_frames_are_where_the_second_map_s_word_now_parts`:
  11069, which needs both. The price alone moves the parting from index 2 to
  index 9 and not the word.
- `horses_and_rubber_each_take_their_own_percentage` (`sim::cost`) and
  `a_range_already_mined_is_taken_on_the_survey` (`sim::gather`): the
  arithmetic and the predicate, from this reading.

Reading-only, and owed a blind second reading: the two arms' predicates
beyond what 10980 and 10982 exercise, namely Rubber's, the Auto Plant's,
the enemy-cell and tree skips in the taken walk, and the chain's order
against the rest of the pre-ramp tail.

## 63. A Civic level seats the Barracks in Norwich, and the word moves to 12429 (2026-09-23, item 661)

Item 657 left Great Lakes' word at **12184**: ours 47 draws against 95,
parting at index 0 on six pairs of `Leader::create_buildings+0xffb`/
`+0x1017`. The production list parts first on 12181. **The item moved it
to 12429.** The mechanism was a trade route's worth, four frames of
reasoning upstream of the draws.

### 63.1 The frame, read backwards

- **12184's draws are a second pass.** The original's step machine goes
  8 → 0 on 12183. Ours goes 8 → 9 and runs Units / Buildings / Make a
  second time (`production_step` 9, 10, 11). The extra draws on 12184 are
  that pass's `create_buildings`.
- **The pass follows a purchase.** Ours buys a Scholar on 12182. The
  original does not.
- **The purchase is an offer's value.** On sim-frame 12180
  `create_units` offers the Scholar with `k` 3 (14 knowledge slots, 11
  filled) on both sides. `check_income@006cc800` writes
  `can_pay_cost@00667570`'s answer to its out-parameter. `create_units`
  re-asks at that count only when it is short **and non-zero**
  (`006c40a0:1554–1561`; `BASE_GOODTYPES` is 0).
  - The Scholar costs **56 wealth**.
  - The original holds **55**. `can_pay_cost` answers 0 and the factor is
    0, so the offer is `val 0`, `num 3`, and it takes only the category
    slot.
  - Ours holds **57**. It affords one, re-asks at 1, gets 256, offers
    `val 6,585,365`, `num 1`, and takes the head.
- **The wealth is parked 514's income.** `income[2:wealth]` is 960
  against 992. That gap was standing from before every window on disk.
  Ours is two wealth *richer* here because the original's market spent
  more earlier. run117's 10576 buy (`docs/ECONOMY.md` §13.3) is where the
  four-wealth gap first chose a different good.

### 63.2 The 32 is two trade routes, and the instrument hid it

`LeaderData::calc_city_resources` adds each city's `trade_val` to wealth
first (`docs/CARAVAN.md` §7.2). This crate's who=1 cities each hold
**160**, and the original's **176**. 2 × 16 is the 32.

The harness had printed `trade_val: ours 0` on every capture, and
`vans.length` the same. Both rows compared the dump against a literal 0,
written when neither field had a writer here, and never re-pointed at the
sim's own field when `compute_trade` landed. So the cause stood beside
its own symptom for a month. **The floor (committed first) reads the
sim's own fields.** On East Indies both routes then agree, 128 with one
caravan each, and four standing keys leave every widening there.

`trade_val` is `trade_value(other, me) · 8`, and `trade_value` is the sum
of both cities' `num_buildings`. The original's Norwich chain on 12180 is
twelve long: `2007 → 2009 → … → 2021 → 2016 → 2018`. This crate's is
ten. The Barracks `1/2016` and the Stable `1/2018` are the two missing.

### 63.3 `find_buildings` on a Civic level

The BUILD record's `city` and `city_down` were printed on every capture
and parsed by nothing. The floor compares them: the owner's city slot,
and the next member by object number.

- **run56–58 (East Indies, 5,200 frames): 0 wrong.**
- On Great Lakes the Barracks and Stable are cityless in both games at
  7514. Bisecting run97 puts their join at **block 8734** in the
  original, both at once, appended after the Mine `1/2021`. This crate
  never seats them.

`Leader::gain_tech@006dcb60:1324–1356` has the arm this crate lacked. For
a type with `is_epoch_type` (vslot `+0x38`) and `TechTypeData +0x14 == 1`,
every live city of the leader is re-masked (`Wall::mask_city` at
`CityData::get_radius@00738410`) and swept by `City::find_buildings@007384c0`.
The same function reads `+0x14` as 0 for Military (`calc_pop_cap`) and 3
for Science (the library re-price), so 1 is Civic.

`docs/CITIES.md` had listed the gap ("`find_buildings` no Civic-tech
trigger"). What it buys is the sweep, not a radius: `get_radius@006db790`
does not read the Civic level. Placement's `get_town` wants every
footprint tile inside the city mask, and `find_buildings` only the centre
tile within the radius. A building that straddled the mask's edge when it
was placed therefore waits for a sweep. `Sim::civic_epoch_sweep` is the
arm, called from `Sim::gain_tech`.

### 63.4 What it moved

- **Great Lakes' word: 12184 → 12429**, past run163's last block (12399).
  The new delta: ours **14** draws against **12**, parting at index **5**.
  Ours spends `Guy::set_anim+0x97a < Unit::move_step+0x823` where the
  original spends `… < Guy::inc_time+0x271`. No dump on disk prints block
  12430; item 669 owes the capture.
- **The value diff on the old word.** run163's thirteen leader rows on
  12181–12184 and `1/2019`'s queue row on 12183 are gone. So is the
  bird's thirty-round `think_bird+0x2aa`/`+0x2d3` arm: nothing parts on
  12184 now, so the arm was downstream of the production pass and not a
  bird mechanism of its own.
- **Removed from the windows, with nothing new arriving:**
  - under run163's conversion, 250 → 238 keys: both `trade_val`s, who=1's
    five wealth rows, `worst_good`, and the four BUILD rows;
  - past it, 1,434 → 275;
  - run136's floor, 250 → 238;
  - run111's leader residue, 88 → 83;
  - run117's, 98 → 88, and its buy frame (10576) is now the original's to
    the unit: food at 128 and an emptied purse;
  - run123's wealth on 11185, 11/6 → 6/6.
- **Great Lakes' endpoint at 24001**: unlinked 0 → 1, extra 2 → 0,
  `build_diverged` 10 → 7; `off` holds at 42. **The market literal**,
  twenty → twenty-one below the word (12385).
- East Indies' word and every golden chapter hold.

### 63.5 What this has *not* established

- **Which Civic level lands on 8733.** The join is dated by the dump.
  That this crate's Civic level lands on the same frame is shown by the
  word, not asserted by a row.
- **The arm's order against step 11's buildings cascade.** A Civic
  level that also unlocks `TOWN` would run `check_upgrade` both ways, and
  no capture has one.
- **Every other `find_buildings` trigger's reach.** A sweep on the
  human's side, and a capture, run through the same function and are
  uncompared.

### 63.6 Coverage

Diff-backed:

- `run163_s_word_frame_is_widened_whole`: the move's value diff on
  12181–12185 and the floor.
- run100, run111, run117, run123 and run136's widenings, re-pinned above.
- The BUILD `city` and `city_down` rows in every window.
- The long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

Pinned capture-free:
`a_civic_level_sweeps_a_straddling_building_into_its_city` (`sim`). It
was made to fail with the call removed from `Sim::gain_tech`.

Reading-only: the gate's two predicates (`+0x38` and `+0x14 == 1`),
settled against the function's own siblings and not by a second reader.

## 64. A formation point is not a loose waypoint, and the word moves to 12536 (2026-09-23, item 669)

Item 661 left Great Lakes' word at **12429**, past every capture: ours
spent 14 draws against 12, parting at index 5 on
`Guy::set_anim+0x97a < Unit::move_step+0x823`. Item 669 took **run174**
(`docs/RUNS.md`), widened the word whole, and moved it **to 12536**. The
mechanism was a misread predicate in `do_move`'s near arm, eleven
blocks ahead of the collision it caused.

### 64.1 The frame, read backwards

- **12429's extra draw is `1/67` stopping.** On block 12430 ours' `1/67`
  hard-collides with `1/68` (`collide_o 68`), goes WALK → DEFAULT and
  spends `move_step+0x823`. The original's has no collider and walks on.
  The second extra draw, a bird's `set_anim+0x104b`, is downstream of the
  first: it goes with it.
- **`1/68` is where the original's is not.** Its heading parts on 12422
  and its position on 12423: ours turns onto a leg toward (37176, 23832),
  and the original's walks to (36947, 23672).
- **The leg is a plan.** `1/68`'s stack parts on **12322**: ours holds
  17 entries, the original's 13. Nothing else about the unit parts in
  between.

### 64.2 The plan, and the unwind that re-aimed it

On 12321 both sides hold `[(36017, 24010) final, (36947, 23672) tol 0]`.
The second entry is group 12280205's formation waypoint: the group move
wrote it on 12284 and moves it as the group walks. On 12322 `do_move`'s
near arm (`docs/ORDERS.md` §4.4) unwinds the loose near waypoints, then
calls `find_tpath` on what is left.

- **The original keeps the formation point.** The tile search aims at
  it, and the stack is the tiles to (37176, 23640), then the point, then
  the goal.
- **This crate popped it.** The tile search aimed at the final goal, and
  the stack ran four tiles further, to (36216, 24024).

The unwind's test, in the listing at `0x5f8ace`:

```
testb $0x21, %al ; jne keep     ; final, or 0x20
leal  -0x1(%ecx), %eax          ; ecx = tolerance
cmpl  $0x5f, %eax ; ja keep     ; unsigned
call  Stack<PathData>::pop
```

A waypoint is loose when its tolerance is **1 to `0x60`**. A tolerance of
0 is an exact point and is kept, and so is anything above half a tile.
The decompiler prints the test as `0x5f < &pPVar12[-1].field_0xcb`, which
is pointer arithmetic on a `PathFinder*` (size `0xcc`, `types.txt`), and
`docs/ORDERS.md` §4.4's pseudocode had rendered it `tolerance < 0x60`.
This crate followed the pseudocode. That test is wrong at both ends: it
pops 0 and keeps `0x60`.

`orders::is_loose` is the listing's test. The waypoints a group move
writes are the tolerance-0 kind, so any group member that re-plans on
the tile grid while walking to its formation point met this.

### 64.3 What it moved

- **Great Lakes' word: 12429 → 12536**, inside run174 (block 12537). The
  new delta: ours **93** draws against **94**, parting at index **92**.
  Ours spends `Farms::inc_time+0x1ae` where the original spends
  `PathFinder::astar_path+0x1697`. On 12537 the squad `1/27`–`1/29` takes
  its orders. The original gives `1/28` and `1/29` a kind-2 order with a
  ten-entry world plan at tolerance 384. This crate gives them kind 21
  with one entry, and leaves `1/27` with none. That is the next item's.
- **The move's value diff.** `1/68`'s sixteen plan rows (12322, 12389)
  are gone. So are `1/67`'s eighteen rows on 12430 (only parked 646's
  `form` stands), the 36 rows above run163 to the word, and every
  one-sided animation change. Nothing parts from 12400 to 12536.
- **The floors.** run174's: 275 → 259 under run163's last block, and
  2,197 → 2,120 in all. run163's: 275 → 259.
- **Great Lakes' endpoint at 24001**: `off` 42 → 53, unlinked 1 → 0,
  extra 0 → 1 (the Merchant `1/81`), `build_diverged` 7 → 6. East
  Indies' word and every golden chapter hold.
- **`collide_sends_the_re_plan_to_the_unit_grid_not_the_tile_grid`**
  (`sim`) laid a tolerance-0 sidestep plan and asserted that the tile arm
  dropped it. That was the misread test. It now lays a loose plan
  (`0x30`), so the two arms still part, and asserts that a tolerance-0
  plan survives the tile arm.

### 64.4 What this has *not* established

- **Every other reader of a waypoint's tolerance.** `find_tpath`'s own
  `0x180`/`0x60` choice and `find_upath`'s forced 0 were read before, and
  are not re-read here.
- **The group-id row.** On 12281 (`1/40`–`1/42`, `1/68`) and on 12537
  (`1/61`, `1/66`) the order's group id differs: ours 12280205 against
  12286605. Neither spends a draw, and what the id is made of is not
  read.
- **Birds.** W4, the bird's second roll, could not be decided on the
  dump, because no dump prints owner 9. The fix removes it with `1/67`'s
  stop, which is the only evidence.

### 64.5 Coverage

Diff-backed:

- `run174_s_word_frame_is_widened_whole`: the floor as committed before
  the fix, the move's value diff, and the new word's first-parting rows.
- `run163_s_word_frame_is_widened_whole`: the floor, re-pinned.
- The long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

Pinned capture-free: `a_tolerance_0_waypoint_is_not_loose_and_0x60_is`
(`sim`). It was made to fail against the old `tolerance < 0x60`.

Reading-only: none. The predicate is a byte sequence at a named address,
and the diff confirms its effect.

## 65. An attack-move buys the retry too, and the word moves to 12897 (2026-09-23, item 673)

Item 669 left Great Lakes' word at **12536**: ours 93 draws against 94,
parting at index 92, where the original spends `PathFinder::astar_path
+0x1697`. On block 12537 the squad `1/27`–`1/29` held kind 21 here and
kind 2 there, and `1/27` held nothing here. The word is inside run174,
so no capture was owed. The coverage driver had not read 12537's blocks,
and item 673 added them before judging any fix (they print nothing
unread).

### 65.1 The frame, read from the dump

- **The issuer agrees.** On 12536 all thirteen of pool group 66 take a
  `GROUP_ATTACK_TO` (flags 5) on both sides. That includes the two
  squads, `1/27`→`1/28`→`1/29` and `1/40`→`1/41`→`1/42`. "Kind 21
  against kind 2" was not a different issue. It was a different
  **ungroup**.
- **`1/27` collides on both sides.** It steps into `1/64`, which stands
  with no order, and records `collide 1`, `collide_o 64`. The original's
  `1/27` ends the frame on a plain `ATTACK_TO`: `dest` 0, `dest_x/y`
  (40456, 23011), `coll` (36485, 23365), **`retry` 6**, counting down one
  a frame after that. This crate's has no order at all.
- **`retry` is the roll.** A `% 3 + 6` in `retry` is `astar_path`'s
  failure tail (`docs/PATHFINDER.md` §21), and `+0x1697` is its draw. So
  the original's 48-grid search for `1/27` failed and bought a wait.
  This crate's search failed and killed the order.

### 65.2 The gate

`docs/PATHFINDER.md` §21.6. The roll and `find_upath`'s reprieve both
call vslot `+0x14`, `is_move`, which every move class answers with
`return 1`. This crate read them as `is_transit`, a move without the
action bit, and a group attack-move carries the bit. With the order
spared, `do_group_move`'s follower arm takes its step-10 tail
(`move_step` returned 0) and ungroups the squad from its captain down.
`1/28` and `1/29` lose `PATHED` and their paths, and each re-plans ten
world entries at tolerance 384 in the same frame.

This is not parked 674. The group id stands, and spends no draw.

### 65.3 What it moved

- **Great Lakes' word: 12536 → 12897**, inside run174 (block **12898**,
  the capture's last but one). The new delta: ours **8** draws against
  **9**, parting at index **2**. Ours spends `Guy::set_anim+0x97a <
  Guy::inc_time+0x271` where the original spends `Guy::set_anim+0x97a <
  Unit::move_step+0x823`. On 12898 the original's `1/41` hard-collides
  with `1/15` (`collide_o 15`) and stands, and this crate's walks on.
- **The move's value diff.** The 33 squad rows on 12537 are gone, and
  `1/27` is the dump's own there, field for field. run174's floor fell
  from 2,120 keys to **470**: all 1,650 above run163, and none under
  (259 stands). What stands on 12537 is 674's group id on `1/61` and
  `1/66`.
- **What stands under the new word**, 209 keys from 12538 to 12899:
  - 29 citizens' `myhits` 40 against 50 and `mylos` 2 against 4 from
    12564, with no draw;
  - `1/41`'s plan on 12626, 23 entries against 20, and its position
    from 12662;
  - on 12825 the original ungroups the squad `1/40`–`1/42` to kind 2,
    and this crate does not. That is 12537's shape on the next squad,
    73 blocks before the word, and it names no mechanism yet.
- **The ladder**: two market rotations, 12582 and 12782, came under the
  word (21 → 23).

### 65.4 What this has *not* established

- **Why the original ungroups `1/40`–`1/42` on 12825.** Its members
  part in value from 12626, so the frame is booked, not a mechanism.
- **The citizens' hit points.** 40 against 50 from block 12564 on every
  citizen of who=1 looks like an upgrade the original applies and this
  crate does not. It spends no draw before the word, and nothing here
  reads it.
- **The other `is_transit` readers.** `get_action`'s walk is its own
  predicate in the listing, and was not re-read.

### 65.5 Coverage

Diff-backed:

- `run174_s_word_frame_is_widened_whole`: 12537's value diff, `1/41`'s
  every row on block 12898, the one-sided stand there, and the floor.
- The coverage driver reads run174 on 12535..12539 and 12896..12899.
- The long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

Pinned capture-free: `a_walled_in_attack_move_buys_the_retry_and_keeps_its_order`
(`sim`), made to fail against `is_transit`.

Reading-only: none. The slot is read at three named addresses, and the
dump's `retry` 6 confirms its effect.

## 66. A resumed search reads the blocks it copied, and the word moves to 14382 (2026-09-24, item 678)

Item 673 left Great Lakes' word at **12897**. There, ours spent 8 draws
against the original's 9, parting at index 2. On 12898 the original's
`1/41` hard-collided with `1/15` and stood, and ours walked on. Above it
the chain ran through 12626's plan and 12825's ungroup of `1/40`–`1/42`,
and no mechanism was named. The word was inside run174, so no capture
was owed to read it.

### 66.1 The frame, read from the dump

- **12626 is the first parting on the squad.** `1/41`'s path holds 23
  entries here and 20 in the original. The eight world entries agree,
  and only the 48-grid sidestep round `1/34` differs.
- **The ungroup follows from the plan.** `1/41` trails the original's
  by three frames from 12662. On 12825 the original's stands one frame
  with no collider and `retry` 0, and its squad ungroups. Ours is 46
  units further back and does not.
- **12898's stand follows from the ungroup.** The two sides walk
  different plans after it, and the original's meets `1/15`.

The brief's three readings were each killed: a different world plan, by
the agreeing world entries; 12537's mechanism by another caller, by
`retry` 0; a collision predicate, by the fix. What held is that the
search read the blocks differently. The mechanism is
`docs/PATHFINDER.md` §26. `1/41`'s 48-grid search copied five world
cells on 12623 and suspended. Its resume on 12624 read those copies,
with `1/66` where it had stood a frame before. This crate read the live
blocks.

It is none of the parked items near it. The citizens' hit points (679),
the group id (674) and the birds (675) never reach the search.

### 66.2 What it moved

- **Great Lakes' word: 12897 → 14382**, past run174. The new delta:
  ours **11** draws against **13**, parting at index **2**. Ours spends
  `Leader::make_stuff+0x221` where the original spends
  `Leader::produce_building+0x1805`.
- **The move's value diff**, on run174: 74 keys on `1/40`–`1/42` are
  gone, and nothing on the squad parts through 12899. The floor fell
  470 → 396, with none added anywhere in 11400..12899.
- **run178** (`docs/RUNS.md`) is run174's line over [12894, 14899]. It
  was widened whole from 11400 across eight captures. It carries run174's
  396 to 12899 exactly, then 37 keys under the word, 60 on its block, and
  1,552 to its last block.
- **The ladder**: run53's scholar seatings under the word went 11 → 13
  (13555 and 13736), and its market draws 23 → 29. Both passed their
  comparison against the original's own streams unchanged.
- **Great Lakes' endpoint at 24001**: `off` 34 → 33, `build_diverged`
  8 → 4.
- **East Indies is the lower map again**: 13640 against 14382.

### 66.3 The new word's block, 14383

who=1 places a building, and the two sides place it apart:

- The new `1/2025` stands at (42624, 19776) in the original and at
  (39552, 17472) here. Norwich's chain `1/2022 → 1/2025` exists only
  here, and the make list's eighth slot holds type 427 there and nothing
  here.
- The citizens `1/6` and `1/7` trade roles. Each holds the other's order
  kind, 3 against 7, and with it the other's path, heading and figure.
- Under the word, the leader's `SITE` table parts on 12976 (`SITE[0]`
  and `SITE[1]`'s rank, `SITE[1]`'s cell) and on 13176 (`SITE[4]`'s cell
  and distance). The rest under it is the newborns' residue run174
  already carried.

**No mechanism is named.** The site table is the nearest input the dump
prints, and it is where the next item starts.

### 66.4 What this has *not* established

- ~~**Why `SITE[1]` parts on 12976.** It is 1,406 blocks under the word,
  and run178 prints every block between.~~ It was not the first parting:
  the site values stood at double the original's from before 11400,
  for want of `City::fix_world_vals` (§67), and the Barracks on 14383
  does not read the table at all.
- **Whether a word turns on the unwind probe's copies.** They are carried
  (`docs/PATHFINDER.md` §26.6), and no diff has isolated them.

### 66.5 Coverage

**Diff-backed**:

- `run174_s_word_frame_is_widened_whole`: the move's value diff;
- `run178_s_word_frame_is_widened_whole`: the new word's block, the rows
  under it, and the floor;
- the coverage driver, which reads run178 on 14381..14385;
- the long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Pinned capture-free**: `a_nocoll_probe_reads_the_block_the_last_one_copied`
(`sim`).

**Reading only**: none. The rule is listing-backed (PATHFINDER §26.7).

## 67. A founded city wears the site values down, and the word moves to 14529 (2026-09-24, item 688)

Item 678 left Great Lakes' word at **14382**. There, ours spent 11 draws
against the original's 13, parting at index 2: ours
`Leader::make_stuff+0x221`, theirs `Leader::produce_building+0x1805`. On
block 14383 who=1's new building `1/2025` stood at (39552, 17472) here
and (42624, 19776) there. The nearest input the dump prints was the
`SITE` table, parting on 12976. No mechanism was named.

### 67.1 The frame, read from the dump

- **Both sides place a Barracks (type 427) on 14382, for the capital
  `1/2000`.** The original spends four jitter draws (`+0x1805`, a 2×2
  all clear) before `make_stuff`'s two. Ours spent two.
- **They take different spiral cells**: (55, 25) in the original, a 4×4
  corner at tile (220, 100) with jitter `dy` 1, and (51, 22) here. The
  spiral (§2.20) scores both `1000 + 0xff − val`, since neither has a
  friend and both are `d ≤ 4`. The scores were 1251 for (55, 25) at
  `val` 4 and **1252** for (51, 22) at `val` 3.
- **Every input the dump prints agrees on 14382**: each unit's position,
  each building, and the leader's territory (568).
- **The `SITE` table is not a link.** A Barracks placement does not read
  `Leader::sites`. The 12976 rows were only the first partings of a
  table that had not agreed since before 11400. The same sites sat in
  other slots, and their values were double the original's, e.g.
  (48, 34) at 189000 against 94500.

The brief's three readings: the site ranking from a different input
(killed: the values already differed at 11400); the site right and the
spiral different (the spiral half held); the make list on another frame
(killed: both place on 14382). The spiral's gates were each killed on
this cell: the river bit `0x2`, the city's `city_flags` bit `0x2`, `find_friends`,
the distance (from the listing at `006e1f9a`), and a probe of
`blocked_site`'s unseen count. What held is the score's last input:
`WData.val`, which this crate never wrote after load.

### 67.2 `City::fix_world_vals@00735aa0`

`City::init@00737050` calls it last, unconditionally, so it runs on a
founding and on a capture alike:

- the centre is the city's cell;
- `r = min(get_radius(type), 0x40)` and `k = min((r + 3) / 4, 0x3d)`;
- for `i < circle_radius[k + 3]`, each in-bounds cell's `val >>= 2`
  while `i < circle_radius[k]`, and `>>= 1` otherwise. `circle_radius[r]`
  counts rings `0..=r`, so rings up to `k` are quartered and the three
  beyond are halved.

`City::close` gives nothing back. The only other writers of `val` are
the map build (`World::compute_all_vals`), the constructor and the
scenario loader. Readers: the spiral's score (§2.20), `compute_site_stats`
(§2.13) and `Map::make_continents`.

Norwich (54, 32) was founded after the frame-0 dump. (55, 25) is seven
cells from it, in the halved rings, so the original reads `val` 2 there
and scores 1253. (51, 22), ten cells away, keeps 3. `Sim::init_city`
now ends in `fix_world_vals`. The harness's `build_sim` writes the
dumped values back once its frame-0 cities stand, because the dump is
post-`City::init`. `sim_at_frame` already restores its whole map.

### 67.3 What it moved

- **Great Lakes' word: 14382 → 14529**, inside run178 (block 14530 of
  its 14899). The new delta: ours **7** draws against the original's
  **3,213**, parting at index **0**. Ours spends `Guy::set_anim+0x97a <
  Guy::inc_time+0x271` where the original spends `PathFinder::
  calc_road_cost+0x46`.
- **The move's value diff, on run178**: the 60 keys on block 14383 are
  gone (`1/2025`'s position, Norwich's chain, `MAKE[7]`, and `1/6` and
  `1/7` trading the build and walk orders). Nothing parts on the old
  word's five blocks. The walk's keys went 1,552 → 1,375.
- **The site table agrees as a set**, `reg` aside, on 2,776 of the walk's
  3,500 blocks, against none before. Three spans remain, 11776–11975,
  14176–14375 and from 14576, and each is one low-value site sampled
  differently. The rest of the parting is slot order.
- **Every widening under both words sheds only `SITE` rows**, with none
  arriving under a word. On Great Lakes: run136 238 → 218, run163
  259 → 238, run174 396 → 374, and run159's two ranks. On East Indies:
  run139, run143, run149, run152, run155 and run166 lose 20 to 39 each.
  run149's `SITE[7].val` 26 against 13 on 10776 (§61.5) was the halving.
  run63's site residue goes **7,122 → 0** of 27,000: the "twice and
  four times the original's" its comment named. The leader-record
  residues run19, run91, run107, run111, run115 and run117 lose 5 to 30
  `SITE` rows each. run84 goes 81 → 87, the same sites at the original's
  values sitting in other slots.
- **East Indies' word holds at 15782**, with the same delta. Every golden
  chapter holds. run53's market draws below the word go 29 → 30, since
  14385 now falls under it; its scholar seatings hold.
- **The endpoints at 24001**, pinned in no direction: Great Lakes `off`
  33 → 46, `extra` 1 → 0, `build_diverged` 4 → 2. East Indies `off`
  46 → 55, `unlinked` 10 → 0, `extra` 0 → 2, `build_unlinked` 1 → 2,
  `build_diverged` 18 → 16. East Indies B `off` 36 → 37.

### 67.4 The new word's block, 14530

- **On block 14529 the capital moves** in the original: `city_flags` bit
  `0x10` leaves `1/2000` for Norwich `1/2007`. Here it stays. CITIES §4
  reads the arm that does this: a Senate finishing in a city of the
  capital's race, when the capital has no Senate, takes the flag and
  runs `fix_borders`. That is the next item's hypothesis, not a
  finding.
- **On 14530** `1/23`'s 24 path entries carry flag `0x20` here and not
  there, and `1/28`'s figure turns here alone. 350 rows stand on the
  block.

No mechanism is named (DECISIONS 42).

### 67.5 What this has *not* established

- **The three spans where one site still differs.** Each is a single
  low-value site, which points at `compute_sites`' large-region stride
  or `site_mark`, and no draw was compared for it.
- **The slot order.** The same ten values sit in different slots on most
  blocks. §2.7's insertion rule is transliterated. Whatever orders the
  slots differently is upstream of the values, and unread.
- **A capture's re-`init`.** `City::init` runs on a capture, and so the
  wearing-down runs again. No capture on disk takes a city.

### 67.6 Coverage

**Diff-backed**:

- `run178_s_word_frame_is_widened_whole`: the move's value diff on
  14383, the rows under the new word, its block and the floor;
- run63's site residue at 0 of 27,000, both leaders, every frame;
- the widenings and leader-record windows above, re-pinned;
- the coverage driver, which reads run178 on 14528..14532;
- the long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Pinned capture-free**:
`a_founded_city_quarters_the_site_values_inside_its_rings_and_halves_three_beyond`
(`sim`). It was made to fail with the call removed from `init_city`.

**Reading only**: the capture arm, which no run reaches. The rest of
`fix_world_vals` is backed by the diffs above.

## 68. The caravan verifies a road the sweep took, and the word moves to 14650 (2026-09-24, item 695)

Item 688 left Great Lakes' word at **14529**. There, ours spent 7 draws
against the original's 3,213, parting at index 0: ours `Guy::set_anim
+0x97a`, theirs `PathFinder::calc_road_cost+0x46`. On block 14529 the
capital moves to Norwich in the original only. The hypothesis was CITIES
§4's Senate arm with a road search after it.

### 68.1 The readings, and what killed each

Each was written with its killer before the reading:

- **The old capital is lost or downgraded.** Killed if `1/2000` stands
  whole on 14528 on both sides. **Killed**: London stands, and only
  `0x10` leaves it (`city_flags` 18449 → 18433).
- **A Senate or government building finishes in Norwich.** Killed if no
  building in Norwich changes state on 14527–14529. **Holds**:
  `1/2024`, type 438 (Senate, 414 + 24), goes `flags` 3 → 7 on block
  14529. The capital's move follows from it, and it is drawless.
- **The road search is the capital's consequence.** Killed if a road
  search runs on 14528 too. **Killed**: 775 road draws on 14528, on
  which both sides agree. The word's search is someone else's.

The search: 3,206 draws on 14529 and 3,206 on 14530, which is the budget
of 3,200 plus the pops that crossed it. A building's search stops at the
budget. Only a caravan's carries on to the next frame (`docs/CARAVAN.md`
§5.2). And the schedule `(frame + o) % 16` fires no Norwich building on
14529. The caravan `1/23` (London ↔ Norwich) loses flag `0x20` on every
path entry on block 14530. That is `Unit::do_move`'s road check
(CARAVAN §10), and it fires only on a waypoint whose tile is no longer
road. The tile, (220, 101), is under the Barracks `1/2025`. Here it was
`0x190`: placed on, and road.

### 68.2 What run189 showed

run189 is a packet at logger 14529 with a `WORLD=6` dump beside it, and
matches run53 draw for draw over 14,536 frames. In the original,
(220, 101) is `0x180`, not road. Across the whole map the surface parted
on **17 tiles and no others**, the trade road from (220, 98) south to
(216, 115). That road was laid on 5573 and never re-planned. In the
packet, (220, 97)'s element claims N alone, and the cells from (220, 100)
south hold no elements at all. `Roads::scan_and_kill_stray_roads`, which
`Game::do_frame` calls after `frame++`, took them: `docs/ROADS.md` §10.
Its cursor, cell 903, is `14,529 × 7 mod 3,600`.

### 68.3 What it moved

With the sweep (ROADS §10), the check (CARAVAN §10) and the Senate arm
(`docs/CITIES.md` §15):

- **Great Lakes' word: 14529 → 14650**, inside run178 (block 14651 of
  its 14899), measured on the tree after 693's merge. **The new delta:
  ours 2 draws against the original's 3, parting at index 0.** Ours
  spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the original
  spends `Unit::do_non_flat_gather+0x54b`.
- **The sweep's kills here are the original's**: (216, 115) on 5905,
  then one tile a visit to (220, 98) on 14100. By 14529 no tile's surface
  parts from run189's (`run189_s_world_has_the_original_s_roads_at_14529`).
- **run93's block 7932**, the only other mid-game world on Great Lakes,
  loses its last parted cell. That is (54, 28)'s road flag, whose four
  tiles the sweep had taken by 7448. The tile masks go 146 → 142.
- **The value diff on run178**: nothing parts on 14528..14532. The two
  capital rows, `1/23`'s 24 path rows and `1/28`'s figure are gone. The
  walk's keys go 1,375 → 1,279.
- **East Indies holds at 15782**, and every golden chapter holds.
  run53's market draws below the word go 30 → 32 (14582, 14585).
- **The endpoint at 24001**, pinned in no direction: Great Lakes `off`
  46 → 30. East Indies holds.

### 68.4 The new word's block, 14651

- **Under it, on 14650**, `1/43`, who=1's gatherer, collides with `1/7`
  here and `1/6` there, and takes a kind-1 move here against a kind-7
  order there.
- **On 14651** one record parts: the Woodcutter's Camp `1/2009`'s gather
  list, 192 keys. The original's list is one entry along from this
  crate's (`tx[1]` here is `tx[0]` there). 517 rows stand on the block.
- **Also under it**: who=1's `known_rares` on 14536 (ours 4, theirs 0),
  with or without the Senate arm. And who=0's `gather_stamp` on 14537,
  re-stamped eight frames early because this crate recomputes territory
  at once.

No mechanism is named (DECISIONS 42).

### 68.5 What this has *not* established

- **`element_num`** is the render piece, and the sweep's "C4" exemption
  reads it. It is taken here as `is_terrain_creation` alone (ROADS
  §10.5).
- **`TerrainOut::caravan_step`'s camel steps** are not moved into
  `ref_count` here (CARAVAN §10.4). No kill reads either count.
- **The lazy border recompute** after `fix_borders` (CITIES §15),
  sixteen frames there against one here.
- **A pushed group's formation.** `group_move_follower` read an army's
  list with `army.unwrap_or(0)`. On a pushed group (no army) that indexed
  out of bounds on Great Lakes 18333, past the new word. It now reads
  `Sim::gstate`, and a missing slot stands at the leader's point. Before
  this, a group with no army that reached the line panicked. So no
  passing test ran it, chapters nine and ten included, and both pass on
  the gate.

## 69. An unfinished wonder calls its builder back, and the word moves to 15384 (2026-09-24, item 715)

Great Lakes' word was **15383**. On it this crate spent 4 draws against
the original's 3, parting at index 1 on an extra `Guy::set_anim+0x97a <
Unit::do_idle+0x7d` stand. On block 15383 the citizen `1/70` held one order
in the original and none here. On 15384 it stood idle here, and the
Woodcutter's Camp `1/2022`'s gather-chain head was `1/70` here and `1/73`
there (run202, widened by item 711).

### 69.1 The frame, read whole before any reading

run202's `1/70`, blocks 15368..15386. It gathers at `1/2022` (tile (223, 67),
`wait 999998`) and stands at (42744, 13080) through block 15382; only its
figure clock moves. On **block 15383** the original's gather order is gone
and a `BUILDORDER` on `1/2026` (`uid 106`) stands alone. Its `UNITORDER
flags` is **0**, and there is no move above it. `1/2026` is new on that
block: the Pyramids (`orig_type 526`), at (44544, 14592), `frame_started
−1`, `helpers 0`. On 15384 an `EXPLORETOORDER` to (43704, 14280) goes on top
of it, and on 15385 the citizen walks.

`1/2022`'s head is `1/73` on every block 15376..15386, and `1/73`'s next
is `1/72`. So the chain's three "missing entries" are one row, the head,
and ours reads `1/70` there only because ours pushed `1/70` back on at the
front. **The list and the order are one parting**: the order is the cause,
and the chain is its consequence a frame later.

### 69.2 The readings, and what would kill each

Written before this crate's side was read.

- **R1: `1/70` is handed a *gather* order by a list that holds it in the
  original and not here.** Killed if the lists agree on the frame the order
  is issued. **Killed by the dump**: the order the original hands `1/70` is
  a `BUILDORDER`, not a gather, and `1/2022`'s head agrees on 15382.
- **R2: the list parts earlier, on a slot or an arrival.** Killed if
  `1/2022`'s list agrees on 15370. **Killed by the pinned widening**:
  `1/2022` first-parts on 15384 and nowhere under it.
- **R3: `1/70` goes idle here because its last delivery lands a frame
  late.** Killed if its carried load and position agree on 15382.
  **Killed**: nothing of `1/70` parts before 15383, and it had been standing
  at its tile since 15368.
- **R4 (mine): `produce_building` swarms `1/70` onto the Pyramids on both
  sides, and something strips the build here.** Killed if this crate's
  `1/70` holds the build order when `produce_building` returns on 15382.

### 69.3 What killed R4, and what was left

A scratch probe (`RON_PROBE_715`, never committed) printed this crate's side
on 15382. `produce_building` picks `1/70` (kind 7), clears its orders, finds
the swarm spot **(43704, 14280)** — the original's own 15384 explore target
— and leaves `[ExploreTo, Build(2026)]`. **R4 is killed**: the build is
handed.

Then, the same frame, `do_move` kills both orders. The move-grid draw
`Unit::do_move+0xe84` is `1/70`'s and rolls 35862, the original's value
at the same index. That puts the threshold at `0x600` and the goal 2160
away, far, so `find_wpath`. `find_wpath` returns through its **near test**
(start cell (55, 17), goal cell (56, 18), 2 < 3) with the stack unchanged.
The unchanged length is a refusal, the top is final, and the kill tail takes
the build beneath (`5f8b5b`–`5f8b84`). The original's `find_wpath`
(`00688fc0:196`), `do_move` tail and `UnitData::order_type` are the same
shape, so **both sides strip `1/70`**.

`Unit::add_build_order@005e5210`'s callers are four. The one that gives an
**unflagged** build with nothing above it is `Wall::process@00640450`, the
recruiter. `docs/ORDERS.md` §5.9 had named it in one line, and this crate
did not carry it. The site's phase is due on this very frame: (15382 +
2026) = 544 × 32.

### 69.4 The recruiter

`Wall::process@00640450:39-171`, inside the `(frame + o) & 31 == 0` block,
after the under-attack decay and **before** `helpers` is reset (`:172`):

1. The site is not active (`WallData::is_active`, `+8 & 4`) and its owner is
   not human (`leader_flags & 4`).
2. SEAM: every 128 frames an oil-platform site (`is(0x1a6)`) that no friendly
   unit targets (`find_unit(…, FILTER_TARGET, o, who) < 0`) is disbanded.
3. **The gate**: `is_wonder()` (Build vslot `+0x2c`, `BuildData::is_wonder`
   in `vtables.txt`), **or** `ptype.is_fort()` (`ObjectTypeData` vslot
   `+0xfc`, named by the PDB's `LF_ONEMETHOD` record — the export has no
   `BuildType` vtable) with `damage` (`+0x24`) zero. No other site recruits.
4. `want = max(4, helpers)` (`+0x64`). For a wonder, every playing leader
   (`flags & 3 == 3`) that is not its ally adds one if its `wonderwin_timer`
   (`+0x44c`) runs. Otherwise it adds one for each of its unbuilt wonders of
   the same type whose progress `f` beats this one's `g` with `f < 2g` or
   `f < 0.35` (`__real_3eb33333`). Progress is `job_counter (+0x48) /
   construct_time(0)` (vslot `+0x18c`), in floats.
5. If `helpers < want`: `ObjectsData::find_unit@0065ca80(x, y,
   SEARCH_FRIENDLY, who, 0xf00, 0x200, FILTER_TYPE, 0x32, 0,
   FILTER_NOT_BUSY)`, and the unit found takes `add_build_order(site,
   QUEUE_NEW, 0)`.

**The search** is the nearest by `vector_dist` on world units (`65cf16`–
`65cf3c`), `<=` against the running best and the range, so the **last** of
a tie wins.

- The two walks are chosen by `total_units` against `circle_radius[5]`.
- The `0x200` flag is the cell-region gate.
- `SEARCH_FRIENDLY` is case 1 of `Search::valid_search@0067daa0`, the
  searcher's own leader only.
- `FILTER_TYPE` is arm 0 of `Search::valid_filter`'s table at `0067e57c`
  (`0067dbc3`): `ObjectData::is(0x32, 1)`.
- `FILTER_NOT_BUSY` (12) is arm 11 (`0067e019`). It is
  `SubObjectData::is_unit` (vslot `+0x18`, the PDB), then
  `UnitData::action_type` ∈ {NONE, GATHER, MOVE_TO}. **A gatherer is not
  busy**, so the recruit is the nearest citizen who is not building,
  fighting or on some other errand.

`crates/sim/src/site_recruit.rs` carries it. `process_building` calls it
ahead of its `helpers` reset. The rival count is exact rational arithmetic,
cross-multiplied, with `0.35f` as its binary fraction `11744051 / 2^25`.
`cities_tests::an_unfinished_wonder_calls_in_the_nearest_citizen_that_is_not_busy`
covers the pick, the phase, the no-action flag, a gatherer taken over a
builder, four helpers, a farm, a human's wonder and a damaged fort. It was
made to fail with the call removed.

### 69.5 What it moved

Measured after `ccc update` onto 714 (`9133578`):

- **Great Lakes 15383 → 15384.** On 15384 this crate spends 51 draws
  against the original's 46, parting at index 4: a third pair of
  `Leader::create_buildings+0xffb`/`+0x1017` (the wonder arm's `% 1000`
  and `% 300`, §2.19) where the original's third draw is
  `Animal::think_bird+0x82`.
- **East Indies holds at 15985**, its delta unchanged.
- **The value diff** (`run202_s_word_frame_is_widened_whole`): `1/70`'s nine
  rows on 15383–15384 and `1/2022`'s one are gone. Nothing of either first-
  parts up to 15385; `1/70` walks from (42728, 13100) on 15385 on both sides.
  Keys first-parting over the window: 840 → 646. Rows standing on 15384:
  327 → 315.

### 69.6 The new word's block, 15385

Nothing first-parts on 15385. On **15386** three rows do:

- the make list's second slot, `MAKE[1].t`, is 430 `SIEGEFACTORY` here and
  −1 there;
- `1/75`'s figure clock (`cur_anim` 31 against 29).

So the new word's extra `create_buildings` draws belong to a producer
that wants a building the original does not. No mechanism is named
(DECISIONS 42). It is inside run202 (last block 15440), so no capture is
owed. The coverage driver reads run202 15382..15387.

### 69.7 What this has *not* established

- **The circle walk's tie order within a cell** is this crate's
  `chain_heads` order, which is the original's object chain for units. A
  tie at equal `vector_dist` is the only place it matters.
- **The rival-wonder count is dormant**: no capture on file has two leaders
  building one wonder. It is reading-only, and so is the oil-platform arm
  (not built).
- **`wonderwin_timer`** is not carried; a leader whose wonder window is open
  would add one without its wonders being walked.
- **Coverage**: the recruit on 15382 is diff-backed, by `1/70`'s order on
  block 15383 and every row of the widening to 15385. The gate's other arms
  — a fort, `helpers ≥ 4`, the rival count, a gatherer taken — rest on the
  reading and the unit test alone.

## 70. A wonder site holds its city's wonder slot, and the word moves to 15608 (2026-09-25, item 722)

Great Lakes' word was **15384**. On it this crate spent 51 draws against
the original's 46, parting at index 4 on a third
`Leader::create_buildings+0xffb`/`+0x1017` pair (§2.19's wonder arm) where
the original's draw is `Animal::think_bird+0x82`. On block 15386 the make
list's `MAKE[1].t` was 430 `SIEGEFACTORY` here and −1 there (run202,
widened by item 715, §69.6).

### 70.1 The frame, read whole before any reading

run53's draw stream with `RON_DEBUG_SITES` and `RON_DEBUG_LEADER` over
15376..15386, both sides:

- **15384 is the step machine's second pass on both sides.** who=1 goes
  8 → 9 on 15382 (the make step places the Pyramids `1/2026` and spends
  three `use_market`, two `produce_building` and three `make_stuff`
  draws), 9 → 10 on 15383 and runs `create_buildings` again on 15384
  (§2.4's step 10). The first pass, on 15381, spent **six** pairs on both
  sides: three wonder types (Pyramids 526, Colossus 527, Hanging Gardens
  528) in two cities.
- **On 15384 ours spends four pairs and the original two**, then the same
  eleven birds. So the parting is a count of (city, wonder) pairs, not an
  order: the original spends no third pair anywhere on the frame.
- The Pyramids' own pairs are gone on both sides — who=1 has a site of
  that type, and the ally walk drops it (`create_buildings@006c1be0:1360`,
  `*local_18 != 0` on an ally, itself included). What remains is the Colossus and
  the Hanging Gardens: four pairs here, two there. **Two pairs are one
  city's.**
- Ours also spends a third `Guy::set_anim+0x104b` at index 48 against the
  original's two. It went with the pairs (§70.4), so it was their
  consequence, not a second parting.
- `MAKE[1]` on 15386 was the pairs' product: it went with them (§70.4).

### 70.2 The readings, and what would kill each

Written after the frame and before the fix. **`num_wonders` had been read
by then** (the gate's one call), so R1 is not blind; it is stated with the
killer it had to survive.

- **R1: a gate the wonder arm reads for one city differs.** The arm's
  city gate is `semaphore & 2 == 0 && num_wonders(city, 1) == 0 &&
  (starting_resources != 7 || city_count > 1)` (`006c1be0:1293–1302`),
  evaluated once per (city, wonder type) — the loop's bound is the
  leader's cities times the wonder types the tree offers, six on 15381.
  Killed if every field the gate reads agrees for who=1 on 15384 *and*
  both sides' readers of those fields agree. **The fields agree**: the
  site `1/2026` is on city 0's chain in the dump (`city 0`, and `1/2022`'s
  `city_down` is 2026), unfinished (`frame_started −1`, `job_counter 0`)
  on both sides. **The readers do not**: `CityData::num_wonders@007382b0`
  walks the chain and counts every object whose Build vslot `+0x2c`
  (`BuildData::is_wonder`, `vtables.txt`) answers — a type range,
  the wonders, `0x20e..0x21e` — that is not the Red Fort and, with the flag,
  not the city building (`+8 & 0x20`). **Nothing reads `is_active`.**
  This crate's `num_wonders` filtered `bd.active`. R1 survives, as a
  reader difference.
- **R2: the count is right and the order differs, so the bird draw is
  displaced rather than missing.** Killed if the original spends its own
  `create_buildings` pair later on 15384. **Killed by the trace**: its 46
  draws hold two pairs, at 0–3, and none after.
- **R3: 715's recruiter changed `1/70`, and that sends the leader into the
  arm.** Killed if `1/70` agrees field for field on 15383. **Killed by the
  pinned widening**: `run202_s_word_frame_is_widened_whole` asserts
  nothing of `1/70` parts on 15383..15385.

### 70.3 The fix

`City::num_wonders` (`crates/sim/src/city.rs`) drops its `active` filter.
Its three readers are the original's three that this crate carries, and
all three move with it:

- `create_buildings`' wonder gate (§2.19): a city with a wonder site
  values no wonder at all;
- `blocked_site`'s wonder clause (`docs/CITIES.md` §2.6.4, `> Egyptians`):
  a second wonder cannot be placed in a city whose first is a site;
- `do_construct`'s start test (`docs/CITIES.md` §3.3, `≤ 1 + Egyptians`):
  the bound was always written for a site that counts itself, and with
  the old filter it admitted one wonder too many.

`Army::find_target`'s `num_wonders(city, 0)` (the `× 10` clause, `docs/ARMY.md`
§12) is not carried; it is gated on `wonderwin_timer`, which is not
either (§69.7).

`ai_build::tests::an_unfinished_wonder_site_shuts_its_city_out_of_the_other_wonders`
places a site of a second wonder type in one of two cities and counts two
draws, the other city's. It was made to fail with the filter restored
("the site counts").

### 70.4 What it moved

Measured on `745c91f`, based on `2355946`:

- **Great Lakes 15384 → 15608.** On 15384 both sides spend 46 draws
  draw for draw, and 15385 and 15386 agree too. On 15608 this crate
  spends 40 against the original's 38, parting at index 0: ours throws
  `Army::find_target+0x410`, the difficulty gate's coin (`docs/ARMY.md`
  §12), where the original's first draw is the per-candidate score
  `+0x7df`. The coin is thrown only when `find_aggressive_army` answers −1.
- **East Indies holds at 15985**, its delta unchanged (ours 5 against 6,
  parting at index 4).
- **The value diff** (`run202_s_word_frame_is_widened_whole`): the three
  rows on 15386 are gone, and **so is every key that first-parted on
  run202's runway**. Nothing first-parts on 15385..15440, and the floor
  goes 400/1/624 → 400/1/401. The rows standing on 15384 and 15385 are
  unchanged (315, 316): they are the floor's.

### 70.5 The new word's block, 15609, on run211

15609 is past run202's last block, so run211 was taken: run202's line
over [15435, 15860), 250 blocks of runway (`docs/RUNS.md`).
`run211_s_word_frame_is_widened_whole` walks it from run123's 11400
across twelve captures. Floor 401/78/1003: run202's 401, then run211's
own 78 up to the word's block.

- **Nothing parts on 15441..15606.**
- **On 15607 the Despot's escort trades posts.** The Longbowmen `1/77`
  and `1/78` — group 67, who=1's army 3 with The Despot `1/79` and
  `1/76` — each walk toward the post the other holds in the original:
  `1/77`'s `orders_x`/`orders_y` are (41976, 21672) here, `1/78`'s there,
  and the reverse. On 15608 their `GUARDORDER`s carry the swapped posts.
  That is item 711's shape (`docs/GROUPS.md` §25) and sits beside
  parked 716 (`1/79`'s move `facing`) and 717 (the mirror bit's unwired
  readers).
- **On 15609** `1/3`'s move target and `1/61`'s clock part.
- **The coin is probably not the escort's.** `find_aggressive_army`
  wants an army with two captains, not mustering, outside its own land.
  This crate's who=1 armies on 15608 (`RON_DEBUG_ARMIES`): army 3, the
  escort, is status 17 (mustering); armies 1 and 2 are status 18 with
  three and eight captains, and none of their units parts before the
  word's block (`1/61`, army 2's, parts on it — a clock). So what the
  predicate reads differently is an army's own `pos` or the owner of
  its cell, and neither is dumped per frame. ~~**A packet at 15607
  (run212, `docs/EMULATOR.md` §8) reads the original's army records.**
  No mechanism is named (DECISIONS 42).~~ **The disk answered first**:
  the dump prints the human's `frame_attacked`, 10233 there against 8186
  here, and the stamp closes L=0 before any army is read (§71).

The coverage driver reads run211 15607..15611, and the compared pin walks
the same window: `OrderDump.coll_x`/`coll_y` are compared there and left
the pin.

### 70.6 What this has *not* established

- **The other two readers are not diffed.** `blocked_site`'s wonder
  clause and `do_construct`'s start test move with the count, and no
  widening on file was checked for a frame that exercises either (a
  second wonder placed in a city holding a site, or started beside a
  finished one). Reading-only: the decompile's
  `num_wonders` and the two comparisons in `docs/CITIES.md` §2.6.4 and
  §3.3.
- **`find_target`'s `num_wonders(city, 0)`** is not carried (§70.3).
- **Coverage**: the wonder gate on 15384 is diff-backed, by the draw
  stream (four pairs → two) and by every row of run202's runway. The
  unit test covers a site of a second type; a site of the same type is
  the ally walk's, already covered.

## 71. A hit stamps its owner's `frame_attacked`, and the word moves to 15619 (2026-09-25, item 729)

Great Lakes' word was **15608**. On it this crate spent 40 draws against
the original's 38, parting at index 0: ours threw `Army::find_target+0x410`,
the difficulty gate's coin (`docs/ARMY.md` §12), where the original's first
draw is the per-candidate score `+0x7df` (§70.4). Inside run211, widened by
item 722 (§70.5). No mechanism was named.

### 71.1 The frame, read whole before any reading

run53's draw stream over 15600..15612 with `RON_DEBUG_SITES` and
`RON_DEBUG_ARMIES`, both sides:

- **One parting.** Ours: the coin, two scores, the same ten birds'
  thirty draws, `1/3`'s two `do_job` draws, `1/32`'s and `1/61`'s
  clocks, then **three** `Guy::set_anim+0x104b` on `9/6`. The original:
  the same without the coin, and **two** on `9/6`. The third bird draw is
  the coin's consequence: it went with it (§71.4).
- **Both sides call `find_target` on 15608 and score the same two
  cities.** A scratch probe inside the decision (not at the frame
  boundary) named the caller: **army 2** (status 18, eight captains),
  at difficulty 0. For L=0, the human: at war, `defense_mod` 0x100, the
  stamp term `frame_attacked + 0x1c20 <= frame` passes (8186 + 7200 ≤
  15608), `find_aggressive_army(1)` answers −1 — every army's cell is
  who=1's — so the coin is thrown; it is odd, and L=0 is skipped. L=1 is
  me: its two cities, founded by who=1, are scored. So the original
  either stops L=0 at the stamp, or `find_aggressive_army` answers an
  army other than 2 (which skips L=0 without a draw). Both give two
  scores and no coin.
- **The army records agree with themselves** on 15600..15612 here: no
  status, target or membership moves on the word.

### 71.2 The readings, and what would kill each

Written after the frame and the in-decision probe, **before** reading any
writer of `frame_attacked`, but not before `find_aggressive_army@006f2e10`
had been read: the brief's three were held against the probe's output.
Each killer tests the claim's own unit.

- **R1: an army's `pos` or the owner of its cell differs, so ours takes
  the coin arm where the original's `find_aggressive_army` answers.**
  Its unit is the term the coin arm's gate reads *first*. Killed if the
  original never reaches `find_aggressive_army` for L=0 on 15608. The
  listing orders it: `6f6d7a`–`6f6d91` load L's `frame_attacked`, add
  `0x1c20`, compare the frame and `jg` to the skip (`6f81f6`) before the
  call at `6f6d9a`. **Killed by the dump**: `LeaderData::log_data`
  prints `frame_attacked`, and run211's human carries **10233** on every
  block, 15435..15859 (10233 + 7200 = 17433 > 15608). The original stops
  L=0 at the stamp; no army's position is read.
- **R2: the escort's post swap on 15607 is the cause.** Killed if army 3
  cannot reach `find_target` as the caller or as `find_aggressive_army`'s
  answer on 15608. By the listing: `find_aggressive_army` skips a
  mustering army (`6f2e3f`, `testb $0x1, 0x4(%eax)`, the loop's bound 16
  slots, `6f2e89`ff), and army 3 is status 17; the caller is army 2. And
  by R1's kill no army is read for L=0 at all. **Killed.** The escort is
  716/717's and stays parked.
- **R3: the drawing army is a different army in each game.** Its unit is
  the caller's identity, compared on what the slot holds. **Neither
  killed nor shown**: no capture writes an `ARMYDATA` record. What the
  disk does say is weaker: with the fix the frame agrees draw for draw
  (38 = 38) and so does 15610's `find_target` (army 1's, two scores), so
  if the callers differ they differ without a draw.

### 71.3 The writer this crate did not carry

`frame_attacked` (`LeaderData +0xa40`) has **two** writers in the export,
not one (`grep` of `+0xa40` and the name over `decomp/funcs`; the other
two hits are `Leader::init` and `LeaderData::log_data`):

- `Army::find_target@006f69b0`, on taking a target against an enemy and on
  the probe (§12 of `docs/ARMY.md`), with `attacked_by = me`;
- **`Object::take_damage@00652020:67-71`**: when `param_5 == 0` (not
  attrition), `semaphore[0] & 4 == 0` and `difficulty < 2`,
  `leaders[this.who].frame_attacked = frame` — **the struck object's
  owner**, human or computer, with no `attacked_by`.

This crate wrote only the first (`crates/sim/src/ai.rs` called the field
"the frame an enemy army last took a target of this leader's", and
`docs/ARMY.md` §12's gate "7,200 frames since I last took a target against
`L`"; `docs/COMBAT.md` §7.2 step 3 named the stamp as "the owner's
last-attacked frame (AI, on low difficulty)", and nothing carried it). All
three are amended in place.

**The fix**: `take_damage_typed` (`crates/sim/src/fight.rs`), the one
combat entry (`do_damage`'s; the city path passes attrition), stamps the
struck object's owner at difficulty below 2, first after the sixteenth
floor as in the original. With it, who=0 is stamped on every hit its
objects take from 9451 on and last on **10233** — the original's value to
the frame — and nothing is stamped on any other leader through 24,000.
`fight::tests::a_hit_stamps_the_struck_owner_s_frame_attacked_below_difficulty_two`
was made to fail with the stamp removed (0 against 10233).

### 71.4 What it moved

Measured on `2567873`, the lane merged onto the base at `5b2132a` (item
723's booking):

- **Great Lakes 15608 → 15619.** On 15608 both sides spend 38 draws draw
  for draw. On 15619 this crate spends 4 draws against the original's 5,
  parting at index 0: the original spends `Guy::set_anim+0x97a <
  Unit::move_step+0x823` first.
- **East Indies holds at 15985.**
- **The endpoint**: Great Lakes 45 → 22 off, 4 → 0 extra, 0 unlinked;
  East Indies unchanged at 51 off, 4 unlinked.
- **The value diff** (`run211_s_word_frame_is_widened_whole`):
  `0/frame_attacked` is gone from every block of the chain (8186 against
  10233 since the stamp's first hit), and so are the coin's consequences
  on 15609, `1/3`'s move target and `1/61`'s clock. Every Great Lakes
  widening from run136 up loses that one row, beside item 723's own one
  (the scout's formation mirror): run202's floor goes 399/1/400 →
  398/1/399 and its standing rows 315/316 → 314/315; run211's goes
  400/78/1003 → 399/89/1014, the eleven new keys being the escort's and
  the new word's block's (§71.5); run117's leader residue 61 → 60.

### 71.5 The new word's block, 15620, on run211

15620 is inside run211 (last block 15859), so no capture was owed; the
same test pins it, and the coverage driver reads run211 15607..15622.

- **Up to the word, only the escort parts.** Past 15608's swapped guard
  posts, `1/76` — army 3, The Despot's fourth — parts in `half_step` on
  15617 and in position on 15619, the word's frame (ours (41858, 21615),
  theirs (41869, 21627)).
- **On 15620**: `1/76`'s heading, the two swapped Longbowmen's collision
  (`1/78` meets `1/23` in the original and nothing here; `1/77`'s
  `collide_guy`), and `1/36`/`1/40`'s clocks.
- The original's extra `Unit::move_step` draw is very likely `1/76`'s, the
  one unit that parts in position on the word's frame. That is the
  escort's post swap carried forward: **parked 716/717's family**, and the
  commander's to route before a worker builds. No mechanism is named
  (DECISIONS 42).

### 71.6 What this has *not* established

- **R3**, the caller's identity (§71.2).
- **The gaia and third-player stamps.** `leaders.list` is indexed by the
  struck owner's byte; this crate's `ai` covers the players only, so a hit
  on a gaia object stamps nothing here. No reader of a gaia
  `frame_attacked` is carried, and none was found.
- **`semaphore[0] & 4`** is read as off, as `ai_difficulty` already reads
  it: the lobby's difficulty, never a leader's `multi_diff`.
- **Coverage**: the stamp is diff-backed by `0/frame_attacked` on every
  block of the Great Lakes chain and by 15608's draw stream. The
  difficulty-2 arm is the unit test's only.

## 72. A republic raises the commerce cap, and the word moves to 16683 (2026-09-25, item 708)

East Indies' word was **15985**: ours 5 draws against 6, parting at index
4, ours `Unit::think_scout+0x941` where the original spends
`Leader::make_stuff+0x63d`. No dump held the frame; run221 was taken over
it ([15894, 16237), `docs/RUNS.md`).

### 72.1 What the stream said before the capture

The draw stream alone (run54's trace against ours, `RON_DEBUG_SITES`)
already names the side. Both spend two `produce_building+0x1805` and the
head's two expiry rolls, `make_stuff+0x221` (the Temple in slots 0 and
8). Then the original alone spends `+0x63d`, and after it both spend
`think_scout+0x941` (ours: `1/31`, §11's region scan of `docs/SCOUT.md`).
So the scout's draw is spent on both sides, and the extra is who=1's slot
loop (§2.6 step 6) buying a slot ours did not. One `+0x63d` is one slot of
the bought type: the walk runs from the slot to the end of the eleven and
rolls once per match, none for a tower, upgrade-kind or wonder type
(`make_stuff@006c8af0`, the `do … while (iVar9 < 0x1b8)` after
`make_this`).

### 72.2 The readings, and what killed each

Written before the capture's first block was read (the stanza, run221):

- **R2, the scout spends a draw the original's does not.** Killed by the
  stream: the original spends its own `+0x941` at index 5. Its
  record-level killer, `1/31` agreeing field for field on 15985, *fires*,
  on six rows standing since the window's first block: `form` (parked
  646), the pool `group` (674), `myhits`, `hits_left` and `mylos` (679).
  None of them is a term the region scan's stride reads (`n`, the frame),
  and its position and orders agree.
- **R3, the original's slot loop buys a slot ours does not.** Holds. On
  block 15985 the original's slot 5 is a citizen, `t 50`, `val 130909`,
  `escrow 1`, `city 1`, `cat 5`; ours' slot 5 is empty. On 15986 the
  original's reads `t −1`, `val 1309`: bought (`val` /100) and cleared by
  the roll. The gate's own terms (§2.6 step 6, in order) never reach a
  purse comparison: ours has no slot 5 to test.
- **R4, the two are one parting in order.** Killed by the stream.

### 72.3 Whose offer, and why

`run221_s_word_frame_is_widened_whole` walks the whole window. Under the
word, who=1's leader record parts on two blocks and nothing else does:

- **15977**: `rate` for food, timber and metal, ours 93 against 118, 110
  and 100. `rate` is `min(cap, income) / 16` (`goods_picture`, §2.5), and
  93 on three goods at once is a cap.
- **15984**, `create_units`' frame (`create_units+0x642` ×6 on 15983): the
  original offers the citizen at the head and into slot 5, and ours
  offers nothing. The citizen's value reads the rates.

The window's first block carries the cap itself: who=1's `resource_cap`
is **2992 here and 3792 there** on every capped good (knowledge is 999 on
both). run166's 13699 agrees at 2992; run78 (`LEADERS=1`) does not print
it. The difference is 800 sixteenths, **50 a good**.

`Leader::calc_resource_caps@006ce900` ends each slot with the **republic
term**: `has_preq(REPUBLIC_3)`, else `REPUBLIC_2`, else `REPUBLIC_1`, adds
`REPUBLIC_COMMERCE_BONUS3`, `…2` or `REPUBLIC_COMMERCE_BONUS` — once, the
highest tier held, never summed — after the wonder additions and before
`bonus_cap` and the clamp. `rules.xml` ships all three as 50, and the three
`TECHBONUSES` rows (113–115, `0x31d`–`0x31f`) each carry the one
prerequisite **Republic**. who=1, the British, took Republic on 15782 (its
patriot is The Senator, item 706). At commerce level 2, `150 × 125 / 100`
is 187, plus 50, times 16: **3792**. This crate's `commerce_cap` stopped
after the nation percentages (the 2026-09-05 audit's R16), so the tech
arrived and the cap never moved.

### 72.4 The fix

`sim::economy::commerce_cap` adds `Tuning::republic_commerce_bonus[tier −
1]` for `Holdings::republic`, the highest `REPUBLIC_n` held
(`Roles::republic_preq`, loaded from bonuses 113–115, read by
`bonus_level` as the fishermen and merchants ladders are). The three
constants are pinned against `rules.xml` by `Tuning`'s slot table. The
unit test is `a_republic_adds_fifty_to_every_capped_good` (2992 → 3792,
knowledge untouched).

### 72.5 What it moved

- **The value diff** (run221, the word's own frame and the block before):
  the fifteen leader rows under the word go — the three rates on 15977,
  the citizen's nine offer rows on 15984, and the food, the queue count
  and the building's queue on 15986 — and so do the five caps on the
  first block and the 551 rows past the word the purchase parted. One row
  stands: `MAKE[5].city`, 2 here and 1 there, the list's `city` shift the
  floor's make-list rows carry. The floor goes 285/288/851 → 280/280/290.
- **East Indies 15985 → 16683**, past run221's last block (16236). On
  16683 ours spends 6 draws against 7, parting at index 5: the original
  spends six `Guy::set_anim+0x97a < Guy::inc_time+0x271` wraps before
  `Farms::inc_time+0x1ae`, ours five (`1/18` twice, `1/19`, `1/20`,
  `1/54`). One figure's wrap is missing here; whose it is, and why, is the
  next capture's (item 752). No mechanism is named.
- run221's rows past the word to its end: the human's `production_step`,
  and on 16166 who=1's newborn `1/59` on the parked families (`form`,
  `myhits`, `hits_left`, `mylos`) with its `path_recursion`, the leader's
  `peasants` 25 against 26 and its `gather_stamp`.

### 72.6 What this has *not* established

- **The frame the cap moves.** No dump prints `resource_cap` between
  13699 and 15894. This crate refreshes `Holdings` on the rate's cadence;
  the original recomputes the caps every frame. The commerce level takes
  the same path and agrees; the republic term is assumed to, and the first
  agreeing block is 15894.
- **The tiers.** All three rows carry the one prerequisite Republic, so a
  republic holds all three and takes tier 3's 50. Whether a later
  government makes the tiers differ is not read; the constants are equal,
  so no capture could tell.
- **The rest of the cap's terms**: Diamonds, ~~the wonders~~ and Virtual
  Reality (R16) are still not built. No capture on either map has
  reached one. **The wonders are built**: Great Lakes' Pyramids reached
  the cap on run226 (`docs/ECONOMY.md` §15, item 757).
- **Coverage**: the term is diff-backed by run221's `resource_cap` rows
  and by the draw stream through 16682. The has-preq order (3, 2, 1) is a
  reading, and with equal constants no run can falsify it.

## 73. A gatherer counts in its building's city, and the word moves to 16982 (2026-09-25, item 752)

East Indies' word was **16683**: ours 6 draws against 7, parting at index
5. The original spends six `Guy::set_anim+0x97a < Guy::inc_time+0x271`
idle wraps before `Farms::inc_time+0x1ae`; ours spends five (`1/18`'s two
crew figures, `1/19`, `1/20`, `1/54`). No dump held the frame, so run227
was taken over it ([16230, 16935), `docs/RUNS.md`).

### 73.1 What the disk said before the capture

The stream agrees in count and sequence on every frame to 16682 and again
on 16684. Four of ours' five wraps are three-frame crew idles that wrapped
on 16680 too. A `cover=0` trace names no unit, so the original's sixth has
no owner there. run221's last block, 16236, holds every figure's `GUY`
record, and no figure clock parts on it. The two city rows that do part
there were read as floor rows and not as a cause; §73.3 is the correction.

### 73.2 The readings, and what killed each

Written in run227's stanza before launch. Each killer tests the figure,
not the first row.

- **R2, a figure both sides hold wraps there and not here, because its
  clock or state differs.** Holds. On block 16684 the original's `1/46`, a
  citizen idle since 16529 (`cur_anim 3`, `end_time 123`), has rolled
  `cur_anim 2`, while ours is walking. Its record parts first on 16595,
  on its order, not on its clock. On tick 16594, its seventh idle count
  (`think_peasant`'s `idle − 2 ≡ 0 mod 5` gate, `docs/ORDERS.md` §5.9),
  ours' `find_gather_spot` sends it to the woodcutter `1/2009`. The
  original's finds nothing and leaves it standing to 16934.
- **R3, the original holds one more figure.** Killed: no figure's
  animation changes on one side only other than these, and the roster
  agrees.
- **R4, the farm's tick differs.** Killed by the stream (both farms' draws
  follow every wrap on 16683) and by the record: no farm and no farmer
  parts under the word.

### 73.3 Why ours crosses and the original does not

`find_gather_spot` walks to a building of another city only when its own
city's `free + gatherers` (`CityData +0x5a`, `+0x5c`) is more than two
above the other's (`crosses_to`). ours' census on 16594 read London `1 +
13` against the second city's `0 + 10` and crossed. The dump reads `1 + 11`
against `0 + 12` and does not. The totals agree (23). The split does not,
and it had not for as long as run221 prints it: `city:gatherers` 11/11
here against 10/12 on the window's first block, a floor row.

Two woodcutters make the split, `1/50` and `1/61`. Both stand nearer
London than their camp's city, and both work `1/2009`, the second city's.
This crate counted a gatherer in the **nearest** city,
`census_find_city`'s. `plan_strategy@006b9620` counts it in its
**building's** city.
The decompiler shows it as a reuse of `iVar25`. The listing is plain:
`edi` holds the found city through the arm, and `6babfb`
`movswl 0x72(%eax), %edi` overwrites it with the target building's `city`
once the building is the leader's own and its `city` is not negative. Then
`busy++` at `6bac19`, `gatherers++` (`incb 0x5c`) at `6bac3c` and the
`peasant_dist` minimum after it all index `edi`. The distance written is
still the found city's, `objects+0x1fc`. §2.3 step 10's line said
"the target building's city" and then "`city.gatherers++`", and the code
had read the second `city` as the found one. The line is amended.

### 73.4 The fix

`Sim::census_citizen`'s GATHER arm keeps the found city and replaces it
with the target building's city under the listing's conditions. `busy`,
`gatherers` and `peasant_dist` then go to that city. The unit test is
`a_gatherer_counts_in_its_building_s_city_not_the_nearest`, made to fail
on the old rule first.

### 73.5 What it moved

- **The value diff** (run227, the word's own frame and the block before).
  The two cities' counts on the window's first block go. So do `1/46`'s 24
  rows from 16595, `1/2009`'s `gather_down` head (46 against 61) and
  `1/54`'s two rows on the word's block, where the extra draw had moved
  its roll. No figure's animation changes on one side on the word's
  blocks. The 554 rows past the word go too. The floor goes 286/333/866 →
  284/306/312.
- **East Indies 16683 → 16982**, past run227's last block (16934). On
  16982 ours spends 10 draws against 15, parting at index 4: the original
  spends `Leader::make_stuff+0x63d`, a bought slot's expiry roll, where
  ours spends `Animal::do_idle+0x83`. Under it on 16779, who=1's make-list
  slot 1 reads `val` 22784 here against 91136 there. No mechanism is named.
- run227's other rows past the word: the scout `1/0`'s order point and
  formation mirror from 16782 (743's family). On its last block, 16934,
  `MAKE[1].val` still stands, 22784 against 91136, exactly a quarter.
- **The older captures** carried the same split. run58's "gatherer filed
  under the wrong city" from 2576, three rows over 800 frames, closes. So
  do who=1's two city rows on the first blocks of run99, run166, run78
  and run221.
- **East Indies' endpoint** at 24001: 47 → 48 off, 8 → 3 extra, 3 → 1
  build unlinked, 7 → 9 build diverged. Great Lakes' holds.

### 73.6 What this has *not* established

- **A gatherer with no friendly city in its region, working a building
  that has one.** The original counts it in the building's city, and this
  crate does too. Its `peasant_dist` would read the search's untouched
  `find_dist`, 99,999,999, truncated to a short. That is a seam, skipped
  here: no capture has one.
- **When the split first parted.** run58 has it from frame 2576, and it
  stands on every East Indies window since. Nothing earlier is read.
- **Coverage**: the rule is diff-backed. run227's `city:gatherers` rows
  agree on every block, and `1/46` does not leave. The listing settles
  the register. `busy`'s move to the building's city was already built and
  agreeing.

## 74. The British pay half for Taxation, and the word moves to 17189 (2026-09-25, item 767)

East Indies' word was **16982**: ours 10 draws against 15, parting at
index 4. The original spends `Leader::make_stuff+0x63d`, a bought slot's
expiry roll (§2.6 step 6), where ours spends `Animal::do_idle+0x83`.
run227 ends on 16934, so no dump holds the frame. A value parts on disk
first: from 16779, who=1's `MAKE[1].val` reads 22784 here and 91136 there,
exactly a quarter.

### 74.1 The slot, and what its value is made of

The slot holds the same entry on both sides on block 16779: `t 589`
(`0x24d`, **Taxation**; this crate's tree id is one lower, `diff::leader`'s
`ti`), `city −1`, `escrow 1`, `cat 8`. `research_techs` (§2.14) offered it
on frame 16778. It had offered the same tech on 16578, at **91136 on both
sides**, and the walk agreed then.

The offer's value is `income_scaled(check_income(t, 0x400, …, flag), v)`
(the tail of `research_techs@006c6ba0`, the decompile's line 668), and `flag` is
`wonder_mod == 0`. who=1's `wonder_mod` is 0, so `check_income@006cc800`
takes its escrow arm. It answers `0x100` when `can_pay_cost(who, −1, −1,
1) ≥ 1` and `0x40` when not, and returns at once, before the income loop.

A scratch print of ours' terms (reverted) read the same `v` on both
frames: base 2000, w 14, f1 4, f2 3, shortages 0, no missing good,
**67,200,000**. Only `check_income` moved, **256 on 16578 and 64 on
16778**. `67,200,000 × 256` and `× 64` both wrap 32 bits, to 91136 and
22784 after `>> 8`. **So the ×4 is one multiplier, not a sum**: the
original still read `0x100` on 16778, and this crate read `0x40`.

### 74.2 The readings, and what killed each

Written from the brief before the listing was read. Each killer tests the
slot, not the first row.

- **R-a, a factor this crate does not apply** (an age, a government, a
  difficulty, a count of cities). Holds, but the factor is a **nation**
  term. Every term of the value agrees on 16578, and on 16778 only
  `check_income` differs. `check_income` reads the price, and the price
  has a term this crate did not carry (§74.3).
- **R-b, the slot holds a different entry.** Killed: `t`, `city`, `cat`
  and `escrow` agree on 16779, compared on what the slot holds.
- **R-c, `val` does not decide the 16982 draw.** Killed by the
  measurement. With the price fixed, 16982 agrees in count and sequence,
  and the word moves to 17189.

### 74.3 The price

The purse agrees on every block: who=1's `bucket` reads 106 food and 76
timber on 16778, here and there. This crate priced Taxation at **88 of
each**. That is `8 × TECH_COST_FACTOR`, plus the Science surcharge of a
plain Ancient tech with no Science level (`docs/COSTS.md`, "The
discounts"). 76 cannot pay 88. 134 timber paid it on 16578, which is why
that offer agreed.

`TypeData::get_cost@00664090`'s tech arm, after the science discount, the
age-behind terms, the final ramp, the lobby's `tech_cost`, Democracy and
the Greek terms, reads:

```
if (t − TAXATION < 4 && has_tribe_bonus(0xb))
    p = (100 − BRITISH_TAXATION_DISCOUNT) × p / 100
```

`rules.xml` ships the constant as "50% cheaper". who=1 is the British
(§72.3), so the original's price is **44**, 106 and 76 pay it, and the
escrow arm answers `0x100`. The window is the one `research_techs`'
Temple ×100 tests (listing `6c76f5`, `t − 0x24d ≤ 3`): Taxation,
Vassalage, Social Contract and Income Tax.

### 74.4 The fix

`Sim::tech_price` sets `cost::Modifiers::late_discount` to
`Tuning::british_taxation_discount` when the tech is in
`Roles::taxation_line` and `has_tribe_bonus(0xb)` holds. The window is
loaded as `0x24d..=0x250` from `TypeIndex`. A tech has no ramp, so
`late_discount` is the same expression at the same place: after the
science discount and before the redirect. The constant is pinned against
`rules.xml` by `Tuning`'s slot table (301 constants agree). The unit test
is `the_british_pay_half_for_the_taxation_line` (88 → 44 against run227's
purse, `check_income` `0x40` → `0x100`), made to fail on the old rule
first. The price is the one `produce_tech` pays too, so a British
Taxation now costs the British half.

### 74.5 What it moved

- **The value diff** (run227): `MAKE[1].val` on 16779 goes, and nothing
  else on the walk moves. The floor goes 284/306/312 → 284/306/311.
- **East Indies 16982 → 17189**, past run227's last block (16934). On
  17189 ours spends 2 draws against 1, parting at index 0: ours spends
  `Guy::set_anim+0x97a < Unit::move_step+0x823`, a blocked stand (§5 of
  `docs/COLLISION.md`), where the original spends `< Guy::inc_time+0x271`.
  The original spends its own stand on 17190, so one unit stands a frame
  early here. A scratch print at ours' `SITE_BLOCKED` (reverted) named it:
  who=1's `1/55`, blocked by `1/60`. Neither parts on run227 up to its last
  block. run233 was taken over the word ([16929, 17441), `docs/RUNS.md`).
- **The word, widened** (`run233_s_word_frame_is_widened_whole`). The
  stand is `1/55`'s on both sides: the original's takes it by `1/60` a
  frame later (`collide_o 60` on block 17191). Ours' is a step ahead. It
  parts first on 17182 on `half_step`, 0 here and 1 there, and on 17183
  its `last_speed` reads 30 against 15. The original's took a half step
  where ours took a whole one. `1/57`, walking with it, does the same from
  17161, and `1/58`'s path parts from 17147. The first row on the window
  that belongs to no floor family is who=1's `scholars`, 13 here against
  14, from 16971. No mechanism is named.
- **East Indies' endpoint** at 24001: 48 → 47 off, 0 → 3 unlinked, 3 → 0
  extra, 1 → 2 build unlinked, 9 → 3 build diverged. East Indies is now
  above Great Lakes (17099).

### 74.6 What this has *not* established

- **The rest of `get_cost`'s tech tail.** Nothing in this crate applies
  the age-behind terms (`TECH_AGE_BEHIND_*`, and the colour pair),
  Incense on Temple research, Democracy's research discount, the Greek
  terms, the lobby's `tech_cost`, the Hanging Gardens or German industry
  discounts, or Versailles. Where two of them apply to one tech, the
  order truncates. Of these, only Versailles comes after the British term
  in the original. None is measured. The age-behind term is live on
  run227: the human holds no age where who=1 holds one, so any tech the
  human prices there costs it 10% less (20% of knowledge) in the original.
- **Coverage**: the term is diff-backed by run227's `MAKE[1].val` and by
  the draw stream to 17188. The window's other three techs are a reading
  of the listing; no capture has offered one.

## 75. An easy AI builds one wonder, and the word moves to 20568 (2026-09-25, item 785)

Great Lakes' word was **17181**. On it this crate spent 11 draws against
the original's 5, parting at index 0. Ours spent three
`Leader::create_buildings+0xffb`/`+0x1017` pairs (§2.19's wonder arm)
that the original does not. On block 17182, who=1's `MAKE` slots 0, 1, 2
and 8 held three wonders here (526, 528 and 527, category 8) and nothing
there (run226, widened by item 776).

### 75.1 The frame, read whole before any reading

run226's who=1 leader record, whole, on 17170..17183
(`tools/gamelog/leader.py`):

- **`production_step` goes 7 → 8 on block 17182 on both sides**, and it
  is compared and agrees. So the original's `create_buildings` runs on
  sim-frame 17181 as ours does. **Its wonder arm is entered and
  rejected; it is not skipped by the step machine.**
- **The make list agrees until the pass.** The original holds 527 and 528
  in city 1 (`val` 0, category 8) from a pass before run226 until block
  17176, and clears them on 17177. Ours does the same. The parting is
  what 17181's pass writes.
- who=1 holds two cities. The Pyramids `1/2026` stand in city 0 and are
  its only wonder (the dump's `BUILDDATA`, `otype 0x20e..0x21e`). So
  city 0's gate (`num_wonders(city, 1) == 0`) is shut on both sides,
  and the three pairs are city 1's.
- **The first field to part is `wonder_mark`**, 0 here against **1**
  there, from **block 17085**. That is the block `1/2026` activates on
  (`flags 3 → 7`, item 757). It stays 1 to the word. The record prints
  it, but `diff::leader` had it on `UNMODELLED` ("no writer here"), so
  no widening could see it part.
- `multi_diff` 0, `wonder_mod` 0 and `wonderwin_timer` 0 are the same on
  both sides.

### 75.2 The readings, and what would kill each

Written after the arm's gate had been read (§2.19, the brief's reading
list) and before the writer was read. They are not blind. Each killer
tests the claim's own unit.

- **R1: a finished wonder closes city 1 through its wonder count**
  (`docs/CITIES.md` §2.6.4, §3.3). The unit is city 1's gate.
  **Killed.** City 1 holds no wonder or wonder site on either side on
  17180, and ours counts city 0's Pyramids (§70).
- **R2: the Pyramids are offered again because this crate never marks
  them built** (parked 777). The unit is type 526's candidacy. R2
  predicts that the original still offers 527 and 528 in city 1 and
  drops only 526. **The row that splits R2 from R3 is the original's
  make list on 17182**, and it holds none of the three. So R2 cannot be
  the cause. It can only be a part of it that the draws cannot see
  (§75.6).
- **R3: a gate with no wonder in it** (age, resources, `frame_attacked`).
  The unit is the arm's gates, in order (`006c1be0:1287–1391`):
  - `semaphore & 2`;
  - `num_wonders(city, 1) == 0`;
  - `starting_resources != 7 || city_num > 1`;
  - the ally walk over `num_queued[t]`, with the rival's quarter;
  - `wonder_mod == 0`;
  - `get_diff() < 2`;
  - `victory != 6`;
  - **`wonder_mark == 0`**.

  **Killed as stated**: the only input that parts on 17180 is
  `wonder_mark`. And the killing term is a wonder's own.

### 75.3 The writer this crate did not carry

`LeaderData::wonder_mark` (+0x424) has two writers besides
`Leader::init`'s zero. The ally walk runs over every leader slot to
`0xe71af0`, and `init_wonder`'s search runs to the mark:

- **`Wonders::init_wonder@0073c860`**, called from `Build::activate`'s
  wonder arm after `remove_unbuilt_wonder`. It takes the first entry
  under the mark whose in-use bit (`+0xc & 1`) is clear, or the entry at
  the mark, and raises the mark past it. It also sets
  `Game::wonders[t − 0x20e] |= 1`, which `already_built` reads (§75.6),
  and ratchets `wonders_held` (+0x858).
- **`Wonders::close_wonder@0073c7e0`**, called from `Build::close`. It
  clears the entry's bit, then walks the mark down while the entry under
  it is clear.

So the mark is **one past the highest entry in use**, not a count.
`has_wonder` and `get_wonder_value` walk the list to it.

Under `create_buildings`' wonder arm, an easy leader (`get_diff() < 2`)
outside a wonder victory goes on only while `wonder_mark == 0`
(`LAB_006c3fbe` otherwise). **So an easy AI builds one wonder**: its
first activation shuts the arm in every city for as long as the wonder
stands.

### 75.4 The fix

- `Census::wonder_slots` carries the list, by building index.
- `Sim::note_wonders` (`crates/sim/src/ai_census.rs`) derives both
  writers off the buildings, for every leader. An entry closes when its
  building is no longer an alive, activated wonder of the leader's. An
  activation takes the first clear entry under the mark.
- It runs **at the end of the buildings' pass**, where `Build::activate`
  runs. A wonder that activates on sim-frame N is on block N + 1, as the
  dump prints it, and the next frame's `create_buildings` reads it.
- `wonder_value` reads the mark in place of its zero.
- `build.rs` is not touched: the activation itself stays where it was.
- `diff::leader` compares `wonder_mark` now. It left `UNMODELLED`, and
  the coverage pin's `LEADERDATA` row.

Two unit tests, both made to fail on the old rule first:

- `an_easy_ai_wants_no_second_wonder_once_its_first_stands`: two
  cities, one standing wonder. The pass draws before the mark and none
  after it, and a hard AI still draws.
- `the_wonder_mark_is_one_past_the_highest_entry_in_use`: a clear entry
  under the top keeps the mark, a new wonder reuses that entry, and a
  clear top walks the mark down.

### 75.5 What it moved

Measured on `f972e8d2`, based on `a1390a90`, and re-run after `ccc update` onto `adbc7483`:

- **The value diff** (`run226_s_word_frame_is_widened_whole`). The
  twelve `MAKE` rows on 17182 go, and so do four standing rows the
  wonder pass had kept: `MAKE[0]`, `[1]` and `[8]`'s `city` and
  `SITE[1].reg`. **Nothing parts on run226's own blocks, to its last,
  17350.** The floor goes 398/0/876 → 398/0/398, and `wonder_mark`
  itself agrees on every block of every leader window.
- **Great Lakes 17181 → 20568**, past run226. On 20568 ours spends 37
  draws against 38, parting at index 31. Ours spends `Guy::set_anim+0x97a
  < Guy::inc_time+0x271`, where the original spends
  `< Unit::move_step+0x823`, a blocked step. run53's draw stream agrees
  in count and sequence on every frame between.
- **Great Lakes' endpoint** at 24001: 43 → 11 off, 2 → 1 extra (`1/81`,
  a Merchant).
- The scholar seating on **17570** comes under the word, on the
  original's own frame.
- **East Indies holds at 17189**, and so does every golden chapter.
  Chapter eight's dump prints the new row, and it agrees.

### 75.6 The new word's block, 20569, on run243

20569 is past run226's last block, so run243 was taken over
[20500, 20819), **sized to the word, not to the gap** (the commander's
ruling on this item; DECISIONS 50 §7). A capture contiguous with run226
would have been ~3,470 blocks. **No dump compares a value over
17351..20499.** The draw stream agrees across it, and nothing else is
known of it. `run243_s_word_frame_is_widened_whole` walks run226's last
six blocks and run243 whole. The floor is 307/59/26/894: run226's 307
standing rows, then 59 carried onto 20500, 26 up to the word, and the
runway's 502.

- **The gap is not quiet** (the stanza's R3, whose killer fires). On
  20500, beside run226's 307 standing rows, 59 more part. They are
  four who=1 walkers on long paths, each some cells off the original,
  plus `1/80`'s formation and `SITE[1].reg`:
  - `1/40`, 76 units west of the original's;
  - `1/41`, whose path is 38 entries against 37;
  - `1/42`, whose move's `dest_x` is 16968 against 16200;
  - `1/60`, whose path parts on ten entries.
- **Up to the word**, only those walkers' later fields part.
- **On the word's block, `1/40` stands blocked in the original and not
  here** (R2 holds). The original's record reads `collide 1`,
  `collide_who 8`, `collide_o 0` and `stopped 1`. Ours is mid-walk.
  That is the original's `move_step+0x823`, and `1/40`'s own walk had
  parted before run243's first block.
- **No mechanism is named** (DECISIONS 42). Where `1/40` first parts is
  in the gap, and no dump holds it. The coverage driver reads run243
  20567..20571.

### 75.7 What this has *not* established

- **`already_built`.** `init_wonder` also sets `Game::wonders[t − 0x20e]
  |= 1`, and `LeaderData::type_avail@006e33a0` answers 0 for a wonder
  type any player has built (`BuildTypeData::already_built@0063ce10`).
  This crate's `type_avail` has no such arm, so ours still lists the
  Pyramids as a candidate after 17084. The mark closes the arm before it
  draws, so no row sees it here. A hard AI, or a wonder victory, would
  value the type again. Reading-only (parked 777's family, not the
  cause).
- **The rest of the wonder bookkeeping** still reads empty in
  `wonder_value`: the team, enemy and unbuilt wonder values and
  `Game::wonder_winning`. With a wonder standing, the original's team
  value is no longer zero, so a hard AI's `% 1000 × % 300` product is
  scaled by it. No capture on file has a hard AI.
- **A close.** No capture on file loses a wonder, so the entry that
  `close_wonder` clears and the walk-down are the reading and the unit
  test alone. A captured wonder's entry is not read: `Build::swap_team`
  and `check_capture` do not touch the list.
- **Two activations on one frame** take entries in building order here.
  The original's order is its objects' order. No capture has two.
- `wonders_held` (+0x858) and `wonders_built` (+0x854) are not carried.
- **Coverage**: `wonder_mark` is diff-backed on every leader window, and
  on run226 from its 0 → 1 on 17085. The arm's last gate is diff-backed
  by the draw stream on 17181. Entry reuse and the walk-down are
  reading-only.

## 76. A border fix zeroes the rares until the next census, and the word moves to 18938 (2026-09-26, item 822)

East Indies' word was **18182**. On it this crate spent 9 draws against
the original's 11, parting at index 0: ours `Leader::make_stuff+0x221`
(the head's expiry), the original `Leader::use_market+0x1ed` (a sale's
`rand % 6`, §2.15). On block 18181 who=1's make list parted: `MAKE[0]`
held the **Merchant** (`t 61`, cat 4, val 952,380) here against 590 (cat
8, val 22,784) there, and `MAKE[4]` a second Merchant here against the
fresh slot there (run253, widened by item 811).

### 76.1 The frame, read whole before any reading

run253's who=1 leader record, whole, on 18177..18184 (`RON_STANDING`,
`RON_MAKE`):

- **`known_rares` parts on the first block, 18177: 3 here, 0 there**, and
  stands to the word. It is the one input of `create_units`' Merchant arm
  (§55.2) that parts. Every other compared field the arm reads agrees
  (`effective_pop` 70 and `pop_cap` 100 among them).
- The original's own array reads 3 on every block of the window
  (`reg_known_rares` {7: 1, 9: 1, 11: 1}), and its `gather_stamp` is
  **18071**, as ours is. So its recompute at 18071 summed an array that
  read 0 then, and its census at 18175 has since counted 3.
- run251 (17496..17764) has the sum at 2 from the recompute at 17207 and
  3 from the one at 17656, and `reg_terr[8]` at 8. On 18177
  `reg_terr[8]` is 11: a border moved in the gap, 17753..18176, which no
  dump compares.
- Standing beside it and not on the list's path: the wealth family,
  `SITE[0].reg`, `scouts` 0/3, `tech_frame` and `tech_cat_frame[0..3]`.

### 76.2 The readings, and what killed each

Written before the build; each killer tests the claim's own unit.

- **R1, the Merchant arm's gate wrote the list differently on tick
  18180.** Killed if, with `known_rares` at the original's 0, `MAKE[0]`
  and `MAKE[4]` still part on 18181. **Holds**: they agree in every field
  but the standing `city` shift.
- **R2, the list was written the same and cleared differently.**
  **Killed on the disk**: the original's `MAKE[4]` is the fresh record
  on 18181, so no Merchant was offered after step 2's clear.
- **R3, the wealth family feeds it.** The arm reads no wealth, and the
  list parts before any `use_market` of the rotation. **Killed**: the
  list agrees with the wealth rows still standing.
- **R4, the census at 17975 counted 0.** Ours' census at 17975 counts 3
  (a scratch print of every write), and the zero built below reproduces
  the original's 0. **Killed as the cause**; the original's own 17975
  count is in the gap and no dump shows it.
- **R5, a border fix between the census at 17975 and the recompute at
  18071.** **Holds.** A scratch print of every writer on 17900..18200:
  the census writes {8, 10, 12} on 17975 and 18175, the recomputes on
  18039 and 18071 sum 3, and `sync_territory` runs once, on **18032**,
  from `gain_tech`'s Civic arm.

### 76.3 The writer this crate did not carry

`reg_known_rares` (`LeaderData +0x4d4`) has two writers in play: the
census (`plan_strategy`'s steps 8 and 9, §55.1) and
**`World::compute_reg_territory@006b0bb0:76–105`** (§55.3, parked 534).
`known_rares` (`+0x6d4`) has `Leader::init` and `calc_gather`.

- `Region::fix_borders@00680f60` and `Regions::fix_all_borders@0067f7d0`
  are the same loop: **every** region's resume index (`Region::borders`,
  `+0x2c`) goes to 0. Their callers are `Build::activate` (two sites),
  `Build::finished`, `Build::close` (two), `City::check_upgrade`,
  `City::assimilate`, `SpellType::cast_assimilate`, `Forts::close_fort`,
  `gain_tech` (the Civic epoch, two bonus-type ranges, and tribe bonus
  `0xd`), `calc_gather`'s rare arm (a rare bit changing) and
  `fix_tech_flags`.
- `GameDaemon::check_borders@00732060` runs every frame from
  `GameDaemon::process_all@00732700` (after `calc_markets`, before
  `Groups::process`). It calls `compute_reg_territory(r, 0)` on every
  region with `borders < size`.
- `compute_reg_territory`, when the region's index is 0, zeroes
  `reg_terr[r]` and `reg_known_rares[r]` for every live leader and
  clears every city's `bordering`, **before** its 256-cell budget test.
  So the first `check_borders` after a fix zeroes every region, and only
  the census's step 9 counts it again. A `calc_gather` recompute in
  between sums 0.

§55.3 read the writer and did not build it: zeroing in `sync_territory`
reproduced run91's zero and cost `gather_stamp` elsewhere. It was built
on the fix's own frame there. run227 dates the zero: the original's
`reg_known_rares[7]` and `[11]` go to 0 on block **16529**, one after the
fix's own, 16528.

### 76.4 The fix

- `Sim::fix_borders` (`crates/sim/src/city.rs`) is called from the head
  of `Sim::sync_territory`, whose call sites are the original's fixes. It
  raises `Sim::borders_fixed`.
- `Sim::check_borders` consumes the flag at the daemon's point in the
  tick, after `calc_markets` and before `groups_process`, and zeroes
  every leader's `reg_known_rares`. A fix raised by an object zeroes on
  the next frame, after that frame's recompute and census, as run227
  shows.
- `ai_census::tests::a_border_fix_zeroes_the_rares_until_the_next_census`:
  the fix only marks, the next `check_borders` zeroes, the recompute sums
  0, the next census counts again. Made to fail twice: with the call
  removed from `sync_territory`, and with the zero removed from
  `check_borders`.

### 76.5 What it moved

Measured on `59aafe06`, based on `f224f1a7` (811's booking):

- **The value diff, on run253** (`run253_s_word_frame_is_widened_whole`):
  - who=1's `known_rares` on block 18177: **3/0 before, 0/0 after**. It
    parts on no block of run253 now.
  - who=1's `MAKE[0]` on 18181: (61, cat 4, val 952,380) against (590,
    cat 8, val 22,784) before; **590, cat 8, val 22,784 on both sides**
    after. `MAKE[4]`: the Merchant against the fresh slot before; **the
    fresh slot on both** after. Only `city` stands, the floor's shift.
  - The floor goes 310/18/327/977 → **309/4/313/319**: 658 keys past the
    word go with the Merchant.
- **East Indies 18182 → 18938**, past run253's last block. On 18938 ours
  spends 7 draws against 4, parting at index 1: ours
  `Unit::do_guard+0x8fb`, the original `Guy::set_anim+0x104b`.
- **The fix's first write in the gap** is tick **18032**: `gain_tech` →
  `sync_territory` → `fix_borders`, zeroed at 18033's `check_borders`.
  Both recomputes that follow, 18039 and 18071, sum 0 as the original's
  18071 does. 17753..18176 is still compared by no dump.
- **run227**: `reg_known_rares[7]`/`[11]` on 16529 and `known_rares` on
  16536 go (parked 699's three); the floor 306/310 → 303/307.
- **run143**: `known_rares` leaves the first block (214 → 213). It parts
  again on 10384, where ours' recompute of 10383 sums the census's
  recount and the original's waits for 10439: who=1's `gather_stamp`
  (10335 against 10343 from the first block).
- **run178 (Great Lakes)**: `known_rares` first parts on **14584** where
  it parted on 14536. The Senate's fix now zeroes both sides from 14535;
  what stands is who=1's recompute on 14583 here against 14543 there
  (parked 701's shape).
- **Great Lakes holds at 20568**, and its endpoint at 11 off, 0
  unlinked. **East Indies' endpoint**: 42 → 41 off, 3 → 6
  build-diverged, 2 unlinked.

### 76.6 What this has *not* established

- **The budgeted re-zero.** The original zeroes a region on every frame
  its pass has not yet reached, so a census inside a pass is undone for
  the regions still waiting. This crate zeroes once. Where a census
  falls inside a pass (a large map, a fix just before a sweep), the two
  part. No block on file shows it; it is parked 568's transient.
- **The trigger set.** Every `sync_territory` is taken as a fix. The
  original's `gain_tech` fixes on a Civic epoch and two bonus-type
  ranges whether or not the border table changes; ours' `apply_gained`
  calls `sync_territory` only when it does. A tech that fixes and
  changes nothing would zero there and not here. No capture separates
  them.
- **The recompute's timing** stands where it stood (run143's 10383,
  run178's 14583): the zero is right on both sides, and the sum then
  follows whichever recompute comes first.
- **17753..18176** is compared by no dump. The 18032 fix and the zero
  are ours; the original's are inferred from its 18071 sum.

### 76.7 The new word's block, 18939, on run257

18939 is past run253's last block, so run257 was taken over [18933,
19190), sized to the word (DECISIONS 50 §7). **No dump compares a value
over 18434..18932**; the draw stream agrees across it.
`run257_s_word_frame_is_widened_whole` walks run257 whole, and the
coverage driver reads 18937..18941.

- **Nothing parts on 18934..18938.** The first block carries 320
  standing keys. Among them are who=1's wealth family, the make list's
  `city` shift, and a new `bucket[0:food]`, 114 against 214, from the
  gap. who=1's `known_rares` agrees on every block.
- **On the word's block four of who=1's units part whole**: `1/67`,
  `1/68`, `1/70` and `1/71`, 25 or 26 keys each. Ours stand under a
  `GUARD` (kind 12, one order, `tolerance` 144, stopped, `collide_*` on a
  neighbour). The original's walk under an `ATTACK_TO` (kind 2, two
  orders, a one-leg path), given on tick 18938. `1/67`: ours (34632,
  41112), `cur_anim` 3; theirs (34644, 41125), `cur_anim` 7. No army's
  list parts.
- **No mechanism is named** (DECISIONS 42). The four are one order's
  members on the original's side; who gave it on 18938 is the next
  item's question.

### 76.8 Coverage

**Diff-backed**:
- the zero's effect on the sum, on run253 (18177..18433, `known_rares`
  and the make list), run227 (the zero's frame, 16529), run143 and
  run178 (the sum between the zero and the next census);
- run257's word block.

**Decompile-backed and reading-only**: the writer at
`compute_reg_territory:76–105`, `check_borders`' order in
`process_all`, and the fix's callers (§76.3). None is owed a blind
second reading: the zero, its frame and its reader are diff-backed.

**Built**: `a_border_fix_zeroes_the_rares_until_the_next_census`, made
to fail twice (§76.4).

## 77. A wonder is priced by every wonder, and the word moves to 23182 (2026-09-26, item 890)

East Indies' word was **20782**. On it this crate spent 8 draws against
the original's 1, parting at index 0: ours `Leader::use_market+0x1ed`
(§2.15), the original `Farms::inc_time+0x1ae`. On block 20782 who=1's
make list had parted: `MAKE[0]`, `[1]` and `[8]` held the Mausoleum
(527) and the Colossus (528) in category 8, valued 486, 398 and 486
here against 0 there (run289, widened by item 880).

### 77.1 The frame, read whole before any reading

run289's leader, city and building records on 20777..20783, both
sides (`RON_STANDING`, `RON_MAKE`, `tools/gamelog/leader.py`), and the
trace's draws on 20781:

- **Both sides list the same wonders in the same slots** (`t` 527, 528
  and 527, `cat` 8) off the same two `create_buildings+0xffb`/`+0x1017`
  pairs on frame 20781. The pairs are the draws 60461/17844 and
  39823/14766, identical on both sides. `wonder_mark` is compared and
  agrees, and an easy AI's arm draws only at 0 (§75), so no wonder of
  who=1's stands on either side and the arm is open.
- **The `city` rows are parked 325's spelling**, not a city: this crate
  prints the global index (London is 1 of three), the dump the leader's
  own slot (0).
- **The parting field is `val`**, and it is 0 in the original for both
  wonders, the same city, the same draws.
- who=1's stock agrees on the word's blocks. `bucket[wealth]` is 208 on
  both sides. The only standing stock row is parked 851's
  `leftover[2:wealth]`, 1398 against 1390 on 20782, which is the income
  accumulator's remainder, not the purse.
- who=1 holds an unfinished **Pyramids site `1/2029`** in city 1
  (`job_counter` 162,737 of 400,000; the dump's `BUILDDATA`, `orig_type
  526`), on both sides. It is why city 1 is shut (`num_wonders`) and the
  pairs are London's.

### 77.2 The readings, and what killed each

Written before the writers were counted.

- **R1: parked 851's purse is one over.** Its unit is who=1's
  `bucket[wealth]` on 20781..20782. **Killed**: it agrees, 208 on both.
- **R2: a factor of the wonder's value differs** (the type's `+0x118`
  factor, `get_level`, the common tail). Its unit is the arm's product.
  **Killed by R3's fix**: with the price alone changed, `val` agrees on
  both wonders (§77.4). A factor of 0 would have left ours nonzero.
- **R3: `check_income` answers 0 in the original.** It returns 0 when
  `can_pay_cost` finds nothing affordable and the offer carries no
  escrow, which the arm's no-shortcut branch does not. Its unit is the
  price of 527 and 528 against the purse. **Holds**: ours priced them at
  200 wealth against 208, so it could pay. The original's `get_cost`
  counts every wonder held or sited (`docs/COSTS.md`, "A wonder is
  ramped by every wonder"), so the Pyramids site adds a step: 260.

**The writers, counted by what they write**: a make slot's `val` has one
writer on this path, `make_me` from `create_buildings`
(`inc × v / 256`). `inc` is `check_income`'s, and its only
zero-returning arm is the unaffordable one. The price's writers are
`get_cost`'s arms. The same-type count was ours alone; the listing
counts wonders.

### 77.3 The fix

- `cost::wonder_count` is the listing's count: `held − 3` past three,
  `held − 6` past six, twice `held`, and the space-race term.
  `cost::Modifiers::wonder` halves the building arm's term.
- `Sim::building_price` prices a wonder by `get_wonders` (the census's
  in-use entries under `wonder_mark`) plus `get_unbuilt_wonders` (the
  leader's inactive wonder sites).
- A mutation (the same-type count restored, from git, `touch`ed) fails
  `another_wonder_s_site_prices_a_wonder_out_of_the_purse`.

### 77.4 What it moved

Measured on `293a05fa`, based on `6a183d98`:

- **The value diff** (`run289_s_word_frame_is_widened_whole`). On block
  20782 `MAKE[0]`, `[1]` and `[8]` `val` go 486, 398 and 486 against 0
  → **0 on both sides**. Ours' prices go `[0, 200, 200]` →
  `[0, 260, 260]` (527) and `[200, 0, 200]` → `[260, 0, 260]` (528); the
  original prints no price, and its `check_income` of 0 says it is over
  208. The draws on 20782 go 8 against 1 → 1 against 1. The floor goes
  284/42/326/1,222 → 284/3/287/288: 39 keys under the word close (the
  three `val` rows on 20782 and the 36 on 20783 that ours' placement of
  `1/2030` and its spending wrote), and 934 across the runway. What stands is 325's `city`
  spelling on 20782 and who=0's `production_step` on 20801.
- **East Indies 20782 → 23182**, past run289's end. On 23182 ours spends
  49 draws against 48, parting at index 46: ours
  `Leader::produce_building+0x1805` (a placement's jitter, §2.20), the
  original `Leader::make_stuff+0x63d`.
- **Great Lakes holds at 20800**, and every golden word and control
  holds.
- **East Indies' endpoint** at 24001: 37 → 38 off, 2 → 0 unlinked,
  0 → 2 extra, 2 → 0 build unlinked, 2 → 0 build diverged.

### 77.5 The new word's block, 23183, on run299, and a gather offer's escrow

23182 is past run289's end, so run299 was taken over [23177, 23434),
sized to the word (`docs/RUNS.md`). `run299_s_word_frame_is_widened_whole`
walks it whole.

- **The make list parts on block 23182, on `escrow`**. who=1's head
  (`MAKE[0]`) and its category slot (`MAKE[4]`) hold the Farm (417) in
  Norwich at 129,600 on both sides, at `escrow 0` here and 1 there.
- **The writer, counted**: an offer's escrow is `create_buildings`'
  `local_10` (`-0xc(%ebp)`), pushed to `make_me` at `006c3fa8`. For a
  gather type the listing's divisor tail (`006c3ca3`..`006c3db0`) writes
  1 on **every** exit of the head test: the `jge` (the offer out-values
  the head, or the head is empty at −1), the `js` (no head), the
  `can_pay` refusal, the flag test, the goods walk found or exhausted. This
  crate wrote 1 only when the test passed. On 23181 the Farm became the
  head, so the first compare failed.
- **Built**: §3.10's gather branch escrows unconditionally.
  `a_gather_offer_is_escrowed_on_every_exit_of_the_head_test` fails
  under the old rule (a mutation, restored from git).
- **The value diff**: on block 23182 `MAKE[0]` and `MAKE[4]` `escrow` go
  0 against 1 → **1 on both**. The word holds at 23182: the offers'
  `val` agrees before and after, and no draw on 23182 moves.

### 77.6 Where it parts next: slot 4's jitter

On the word's frame both sides buy the head's Farm (`produce_building`'s
spiral, two jitter draws, `Farms::add`, `1/2031` at (34944, 38016)) and
spend the head's two expiry draws. Both walk slot 4's Farm through the
same eighteen spiral candidates. At the best site, corner (184, 196), the
2×2 jitter (§2.20) finds **four** clear sub-positions here, (35712,
38016), (35712, 38208), (35904, 38016) and (35904, 38208), and **three**
in the original. Both place `1/2032` at (35904, 38016), so the building
records agree. The one extra draw shifts the slot loop's expiry
(`make_stuff+0x63d`), which clears slot 4 there and not here (`MAKE[4].t`
417 against −1 on 23183), and `1/79`'s figure draws one along.

**No mechanism is named** (DECISIONS 42). ~~The blocked sub-position is one
of the three besides the chosen one. It sits beside the Farm this frame
just placed. `blocked_site`'s answer on it is a value the original
computes mid-frame, after `1/2031`'s `Farms::add`, so the instrument is a
packet at 23181 (run298, unused here) with a probe **inside**
`produce_building`'s second call, not at the frame boundary.~~ The trace's
own seeds named it, (35904, 38208), which is nowhere near `1/2031`; the
cause is a rock cell, and no packet was needed (§78).

### 77.7 What this has *not* established

- ~~**Which sub-position the original blocks, and why** (§77.6).~~
  Settled by item 904: (35904, 38208), on a rock cell (§78).
- **Past three wonders, the space-race arm and the team term** of the
  price: listing only (`docs/COSTS.md`, "A wonder is ramped by every
  wonder").
- **`get_unbuilt_wonders` is counted off the buildings**, not the
  original's `unbuilt_wonders` list. The two differ only on the frame a
  site dies.

### 77.8 Coverage

**Diff-backed**:
- the one-wonder price, through `check_income`'s 0 and run289's `MAKE`
  `val` rows on block 20782, and the draw stream to 23182;
- a gather offer's escrow, through run299's `MAKE[0]` and `MAKE[4]` on
  23182.

**Listing-backed and reading-only**: the wonder count past three, the
space-race arm, the halving's truncation (pinned by
`cost::tests::a_wonder_is_ramped_by_every_wonder_held_and_halved`), and
the gather arm's escrow on the exits no capture has reached. The listing
was read for both, so neither is owed a blind second reading of the
decompiler's text.

**Built**: `another_wonder_s_site_prices_a_wonder_out_of_the_purse` and
`a_gather_offer_is_escrowed_on_every_exit_of_the_head_test`, each made to
fail by a mutation first.

## 78. A rock cell refuses a Farm's jitter, and the word moves to 23420 (2026-09-26, item 904)

East Indies' word was **23182** (§77.6). On it this crate spent 49 draws
against the original's 48, parting at index 46: ours
`Leader::produce_building+0x1805` (§2.20's jitter), the original
`Leader::make_stuff+0x63d`. Both sides walk slot 4's Farm to corner
(184, 196), and the 2×2 jitter found four clear sub-positions here and
three there.

### 78.1 The frame, from the disk

- **run299 re-widened on today's tree** (`RON_STANDING=23181-23184`):
  the five keys under the word stood as 890 left them, and the first
  block's standing rows are the gap's (21046..23176).
- **The trace's seeds name the blocked sub-position.** A draw record
  carries the seed before the step (`rondata::trace`), so the original's
  three `+0x1805` draws on 23182 read 57, 12 and 99 (`% 100`). Ours drew
  57, 12, 99 and 42 over (0,0), (0,1), (1,0), (1,1), the order of the
  loops at `006e2a78`, and both sides placed `1/2032` at (1,0) =
  (35904, 38016). With three draws, (1,0) takes 99 and wins only if it is
  the third clear candidate. So (0,0) and (0,1) are clear there, and
  **the refused one is (1,1) = (35904, 38208)**.
- **Its one tile no clear candidate shares is (188, 200)**, in cell
  (47, 50). The start dump prints that cell's flags as
  `cell::ROCK | cell::OIL`, the bits 8 and `0x800`. `World::set_oil_at@006b2a10` is the oil bit's
  only writer (the terrain groups and the editor), so the cell is the
  map's, fixed from frame 0.
- **The scratch probe on ours** at tick 23182: all sixteen tiles of
  (1,1) are Clear, owned by player 1 and inside the city radius.

### 78.2 The readings, and what killed each

Written before the build. Each killer tests the reading's own claim.

- **R1: `blocked_tcoord`'s rock arm.** The listing (`006370b2`) refuses a
  non-oil type on a cell with `flags & 8`, verdict 3. This crate has no
  such test. The claim's unit is (1,1)'s verdict. It dies if, with the arm
  built, (1,1) stays clear or 23182 does not agree 48/48. **Holds.**
- **R2: the flat-gather arm.** `00637532` reads the tile form of
  `get_land` (`@006b4c70`, which answers Oil (7) on this cell), and
  `LandData::get_amount` of food on Oil is 0, verdict 9. This crate has
  no such test either. **Foreclosed on this tile** (903): R1's arm
  returns first, so R2 cannot be what the original answered here. It is
  built for its own cells, and its unit test is the only killer that
  reaches it.
- **R3: fog** (`blocked_site` counts dark tiles), and **R4: the cell's
  territory.** No dump prints either on this frame. **Killed by R1**:
  R1 alone closes the frame.
- **R5: "beside the Farm this frame just placed"** (§77.6). **Killed by
  the seeds**: (1,1) is two tiles from `1/2031`'s footprint (180..183),
  and the two candidates nearest it are clear.

The packet booked at 23181 (run306) was not taken. The disk answered it
(the checklist's grep-the-disk row).

### 78.3 The fix

`Sim::blocked_tcoord` (`crates/sim/src/place.rs`) gains the two arms, in
the listing's order:

- **After `AS_BUILDING`**: an Oil Well or Oil Platform (`is(0x1a5)`,
  `is(0x1a6)`) on a cell without `cell::OIL` answers `NoOil`. Every other
  type on a `cell::ROCK` cell answers `Rock`.
- **After the river**: a flat gather type (`GATHER` and `FLAT`: the Farm
  and the oil pair) answers `NoResources` when `LANDS[land]` has none of
  its good. `land` is `World::land_class_tile`, the tile form of
  `get_land`. It differs from `World::land_class`, the cell form, three
  ways:
  - a coast cell is Ocean under an ocean tile and Land under any other;
  - the mountain test is the tile's object, not the cell's flag;
  - oil is tested before rock.

`docs/CITIES.md` §2.5 had specified all three arms since the placement
reading. The code had never carried them.

The spiral already refuses a candidate on such a cell (`buildings_allowed`,
§2.20 defect 3). The jitter's footprint is what reaches into the next
cell, which is why the gap stood for 23,000 frames.

`cities_tests::a_rock_cell_refuses_a_farm_and_a_flat_gather_type_needs_its_good`
walks both arms over a Farm, a Barracks and an Oil Well. A mutation of
each arm fails it (§78.4).

### 78.4 What it moved

Measured on the tree based on `7f178d8c`:

- **The value diff on the word's frame**
  (`run299_s_word_frame_is_widened_whole`). On block 23183 who=1's
  `MAKE[4].t` goes 417 against −1 → **−1 on both sides**, and `1/79`'s
  `g.cur_anim[0]` 3 against 1 and `g.end_time[0]` 42 against 58 →
  **1 and 58 on both**. The draws on 23182 go 49 against 48 → **48
  against 48**, and (1,1)'s verdict goes Clear → Rock. The floor goes
  283/5/288/1,045 → 283/2/285/292/359: the three keys under the word
  close, and 686 across the window.
- **East Indies 23182 → 23420**, inside run299's window. On 23420 ours
  spends 7 draws against 31, parting at index 0: ours
  `Unit::do_guard+0x8fb`, the original `Farms::add_animals+0x92`. The
  original adds a pasture's five animals (twenty draws, then five
  `Animal::do_idle`), and ours spends `Farms::inc_time+0x1ae` there
  instead. **No mechanism is named.**
- **The runway, 23184..23421, is pinned in the same test.** Who=1's
  `gather_stamp` parts on 23192 (23191 here, 23183 there), and it parts
  the same under the rule before the fix (a mutation, restored from
  git). It is parked 701's shape, not this fix's.
- **Great Lakes holds at 24000.** Its long walk (`run53_s_24000_frames_…`)
  agrees on all 24,000 frames, its endpoint stays 0 off, and run80's
  widening passes. **It gains ten keys**, all in who=1's city sites:
  `SITE[4]` on 14176 and `SITE[8]` on 12976. That is a site at (47, 33)
  that ours scored 95,326 and the original never listed. A city is no
  flat gather type, so only the rock arm refuses it.
  - run178's rows under its word drop those ten.
  - run192, run196, run202, run211, run218 and run226's floors each
    drop ten (392 → 382).
  - run117's leader residue drops `SITE[4].rank` (60 → 59).
- **East Indies' endpoint** at 24001: 38 → 36 off, 2 → 0 extra.
- **Every golden word and control holds.**
- **The mutations**, each restored from git and `touch`ed:
  - **The rock arm out** fails the unit test and run178's widening. East
    Indies still reaches 23420, because the flat-gather arm refuses that
    Oil cell by itself.
  - **The flat-gather arm out** fails the unit test. The word holds, on
    the rock arm.
  - **Both out** is the old rule: 23182.

### 78.5 What this has *not* established

- **Which verdict the original returned on (188, 200).** By the listing's
  order it is Rock (3), or 0x24 unseen. Both refuse, and nothing prints
  it.
- **The flat-gather arm has no diff of its own yet.** On this tile it is
  foreclosed by the rock arm. The Farm's sand, forest and oil cells
  elsewhere are its test, and no word has named one.
- **`blocked_tcoord`'s mountain clause** still reads only the tile's
  object, not the cell's `0x10` (`00637023`). That changes Rare (7) to
  Mountain (2), and no caller reads either code.

### 78.6 Coverage

**Diff-backed**:
- the rock arm, through the trace's seeds and run299's `MAKE[4]` and
  `1/79` rows on 23183;
- the rock arm on a city site, through run178's `SITE` rows on Great
  Lakes.

**Listing-backed**: both arms' order and predicates (`006370b2`,
`00637105`, `00637532`), and `get_land@006b4c70`'s tile form, read in
`llvm-objdump` rather than the decompiler. The flat-gather arm and the
oil pair's `NoOil` rest on the listing and on
`a_rock_cell_refuses_a_farm_and_a_flat_gather_type_needs_its_good` alone.
Those are the rows a second reading would take.

## 79. An unstarted farm site is nobody's neighbour, and East Indies runs to 24000 (2026-09-27, item 919)

East Indies' word was **23420** (§78.4). On it this crate spent 7 draws
against the original's 31, parting at index 0: ours `Unit::do_guard+0x8fb`,
the original `Farms::add_animals+0x92`. The booking named no mechanism.

### 79.1 The frame, from the disk

- **The seeds first.** The original's 31 on 23420 are twenty
  `add_animals` draws under `Build::activate+0x1c25 <
  Wall::do_construct+0x199` (coin, `y`, `x`, `Guy::init_real` per animal),
  then `do_guard`, five `Animal::do_idle` rolls and five clock wraps. The
  coins read 63335, 19739, 17423, 20547 and 9655: all odd, so **five pigs**,
  and their offsets are drawn at `+0xc9` and `+0x117`, the pig arm, not the
  chicken's `+0x134`/`+0x182`. A farm finished construction that frame.
- **Which farm.** run299's `BUILDDATA` from 23420 to 23421: who=1's Farm
  `1/2032` goes `construct_hits` 397 → 400, `flags` 3 → 7. It is the second
  Farm `make_stuff` placed on 23182 (§77.6, §78), in Norwich, at
  (35904, 38016). So on the original's side `1/2032` is a pasture
  (`farm_type == 1`, SYNC §3.6), and on ours it was a crop:
  ours spent `Farms::inc_time+0x1ae` on it instead.
- **How its type was chosen.** `Farms::add` runs at placement, on 23182.
  The original's 48 draws that frame hold `+0x23f`/`+0x25b` for `1/2031`
  (its ambience, so a crop) and nothing for `1/2032`: no coin on either
  side. So `1/2032` took a no-draw arm of SYNC §3.8's table. Norwich's
  other farms were `1/2011`, `1/2012` (both crops, or `1/2031` could not
  have taken the ambience) and `1/2031`, so `others == crops == 3`. That
  reaches `get_nearest_farm_type` before the `== 3` pasture, and only a
  **NO_FARM** answer gets there. `1/2031` stands 960 east, inside `0x480`;
  `1/2011` is 1,229 away, outside it. So the original's search did not see
  `1/2031`.
- **Why.** `ObjectsData::find_any_building@00659ca0`'s cell-circle arm
  keeps a candidate only when its vtable `+0xc` (`flags & 1`) and its
  `+0x50` answer, or its owner is `find_who`, which
  `get_nearest_farm_type` passes as −1. On `Build::vftable`, `+0x50` is
  `WallData::is_started@00472360`, `flags & 2` (`vtables.txt`). run299
  prints `1/2031` at `flags 1` on 23183: placed that frame, not started.

### 79.2 The readings, and what killed each

Written before the build.

- **R1: an unstarted site is not a candidate.** Unit: `1/2032`'s
  `farm_type` on 23182. It dies if, with the gate, ours leaves it a crop,
  23182 stops agreeing 48/48, or 23420's first twenty draws are not the
  original's five pigs. Gates between it and the block: `1/2031` unstarted
  at the placement (run299: `flags 1` on 23183 on both sides);
  `others == crops == 3`; no other farm inside `0x480`; `1/2032` finishing
  on 23420 on both sides; `Sim::activate` stocking it. **Holds.**
- **R2: the cell-circle walk itself** (`circle_radius[2]` cells around
  the point, not every building). `1/2031` is in the adjacent cell, so the
  walk reaches it either way. **Killed by R1** alone closing the frame.

No packet (run310) and no capture (run311) were taken: the trace's seeds
and run299's `BUILDDATA` answered.

### 79.3 The fix

- `Sim::nearest_farm_type` (`crates/sim/src/farms.rs`) skips a farm whose
  `started` is clear. The crate already carried `started`, the dump's
  `flags & 2`.
- `farms::SITE_PIG_Y` and `SITE_PIG_X` name the pig arm's two offset
  draws, `Farms::add_animals+0xc9` and `+0x117`, and
  `rondata::trace::SITES` maps `0x008d8ff9` and `0x008d9047` to them. The
  decompile writes the loop body once a species, and until now this crate
  marked a pig's draws at the chicken's addresses. No pig had been drawn
  on either map before 23420.
- `farms::tests::a_farm_beside_an_unstarted_site_is_the_fourth_s_pasture`
  is the word's shape: with the third farm a bare site the fourth is a
  pasture with no draw, and with the third started it copies the crop and
  takes the ambience pair. The older `farms_add_picks_the_type_…` test now
  starts its first farm before the second is placed.

### 79.4 What it moved

Measured on the branch from `a3224660`.

- **The value diff on the word's block, 23421**
  (`run299_s_word_frame_is_widened_whole`): `1/78`'s `guard.retry` 8
  against 6 → 6 on both, and `1/80`'s `g.cur_anim[0]` 1 against 0 and
  `g.end_time[0]` 58 against 31 → 0 and 31 on both. The draws on 23420 go
  7 against 31 → 31 against 31. run299's floor goes 283/2/285/292/359 →
  283/2/285/289/294.
- **The building, both sides.** On 23183 `1/2031` and `1/2032` are
  `flags 1` in the dump and unstarted, inactive here. On 23421 the dump
  prints `1/2031` at 3 and `1/2032` at 7, and here `1/2031` is started
  and `1/2032` started and active, `farm_type` 1 (it was 0).
- **The animals.** No dump prints an owner-9 object, so the original's
  side is the seeds. The five `(dx, dy)` read (−103, −26), (109, 138),
  (129, −18), (−171, 18) and (105, −10), snapped to (35784, 37992),
  (36024, 38136), (36024, 37992), (35736, 38040) and (36024, 37992). Ours
  seats `9/17`..`9/21` at exactly those points, all pigs.
- **East Indies 23420 → 24000, the trace's end.** run54's trace holds
  24,001 `FRAME` records; the harness walks and compares every frame of
  0..24000, and both sides spend 275,108 draws, label for label.
  `run54_s_24000_frames_…` asserts all three, the pasture, and its five
  pigs.
- **Every value window past 23420**:
  - run299's tail, 23422..23433: nothing first parts. 288 rows stand on
    23433: the first block's 283 and `1/81`'s five, a citizen born on
    23366 with the rows every citizen carries.
  - **23434..23959 is covered by the draw stream alone.** No dump holds it.
  - run96, 23960..23999 and the end block 24001, widened whole
    (`run96_s_word_frame_is_widened_whole`, new): 293 rows stand on 23960,
    the 288 of 23433 and `1/82`'s five (a citizen born in the gap, the
    same shape). Nothing parts on 23961..23999. The end block parts on
    `orders.len` for 48 units (the quit dump prints no order list) and on
    `0/3`'s position, (+8, −23), the closing dump's one torn unit.
    run96's `GUY` records are the short form, so a gaia animal's clock is
    now named unprinted there instead of parting; all 104 gaia animals'
    positions agree on every block. run78 is a short-`GUY` capture too,
    and its 208 such rows leave its pins (419 → 211 standing on 15783,
    floor 418/421 → 210/213) with no parting closed.
  - **The endpoint at 24001** (run54's end dump): 89 compared, 0 off, 0
    unlinked, 0 extra, 1 torn; buildings 0/0 unlinked/diverged; cities
    3/0. It was 36 off.
- **Great Lakes holds**: `run53_s_24000_frames_…` agrees on all 24,000
  frames, its endpoint is 0 off, and run80's widening passes.
- **The mutations**, each on the committed build, restored from git and
  `touch`ed:
  - **the started gate out**: the new unit test fails and the word is
    23420 again;
  - **the pig labels out** (a pig marked at the chicken's sites): the
    stream's word holds to 24000 but the label sequence parts at 23420.

### 79.5 What this has *not* established

- **The cell-circle walk.** `find_any_building` walks `circle_radius[2]`
  cells around the searching point, where this crate walks every building
  within `0x480`. The two part only for a farm inside the radius whose cell
  the circle table leaves out, and none has been met.
- **The last-at-equal-distance tie** (`<=`, SYNC §3.8's open list) is
  still the first here.
- **The setup borrow reads the chicken arm only**
  (`rondata::trace::Trace::add_animals`). The setup pastures of the forty
  traces read are all chickens; a pig there would stop the borrow short.
- **The frame 24000 itself**: the trace's last record opens it, and the
  end block it writes is the endpoint's, with its one torn unit.

### 79.6 Coverage

**Diff-backed**: the started gate, through the trace's seeds on 23182 and
23420, run299's `BUILDDATA` and the rows on 23421, and every frame of run54's
trace; the pig arm's sites, through the label sequence on 23420.
**Listing-backed**: `find_any_building`'s test, `call *0x50(%eax)` at
`00659e83` with a zero answer skipping unless the owner is `find_who`, and
`00472360`'s `and $0x2`. **Export-backed only**: that `Build::vftable`'s
`+0x50` is `00472360` (`vtables.txt`). That row, and R2, are what a second
reading would take.

## 80. The second pair: the lobby's top difficulty, and what each arm would do (2026-09-27, item 971)

DECISIONS 53 §2 moves the AI track's word to a second pair: run54's and
run53's games with the lobby's difficulty at its top setting and nothing
else moved. This section is the lever, then every difficulty read the
export holds as a hypothesis with its killer, written **before** run346
and run347 were taken. The results are §80.4 onwards.

### 80.1 The lever, and what "top" is

- **The list.** `data/rules.xml`'s `difficulties` category has six
  entries — Easiest, Easy, Moderate, Tough, Tougher, Toughest — so the top
  setting is **5**, and `diff 0-5` in the console's `? 1` help says the
  same range.
- **What the setup reads.** Every AI read goes through
  `LeaderData::get_diff@006ec000` or its inlined copy: `semaphore[0] & 4`
  → `multi_diff`; else `multi_diff` only when `semaphore[1] & 0x10` and not
  `semaphore[2] & 2` and `multi_diff ≥ 0`; else **`game->info.difficulty`**,
  the lobby's byte. A solo Quick Battle takes the last arm, and the dump's
  `GAME INFO` prints it as `DIFFICULTY`. run53 and run54 both read
  `DIFFICULTY 0`; no capture on disk has ever read anything else (a grep of
  every `gamelog-run*.txt` in `Logs/`).
- **Where the lobby gets it.** `SetupWin::setup_combos` fills the
  difficulty combo (`combo_boxes[0x18]`) from `game->info.difficulty`, and
  `SetupWin::setup_game` writes the combo's selection back to it when Start
  is clicked; the eight per-player combos (`0x20..0x27`) default to 2, which
  is the `diff 2` each `PLAYER` block of `GAME INFO` prints and which
  `get_diff` does not read in a solo game. `game->info` at the lobby is
  `GameInfo::load_from_config@005d4da0`'s under `-config` — its category
  list includes `diffs`, so `check.ini`'s `difficulties=` is read — and the
  profile's `<GAME_INFO>` blocks without it (`profile.py`). ~~Great Lakes'
  captures keep `-config check.ini`, East Indies' drop it, so run347 sets
  `difficulties=Toughest` in `check.ini` **and** `DIFFICULTY=5` in the
  profile, and run346 the profile~~ — **killed by run347's first take**,
  which read back 0 with both set (§80.4): the `-config` lobby does not
  take the difficulty, so both maps drop `-config` and set the profile.
  The dump's `DIFFICULTY 5` is the only read-back that counts.
- **`diff` is not the same lever.** `ConsoleWin::run_cmd@007d6a70`'s
  `diff` writes `game->info.difficulty` at whatever frame the command
  runs — after `Leader::init`, the setup draws and `init_handicaps` —
  so a `diff 5` on `rontrace.cmd` is a different game from a lobby at 5.
  It is not used.

### 80.2 The arms, as hypotheses

Each row names what the top setting does, where it would first show, and
what kills the reading. "Carried" is this crate's code; every carried arm
above difficulty 1 has run only in a unit test until now.

| # | arm (carried?) | at 5 | first shows | killed if |
|---|---|---|---|---|
| H1 | `get_gather_handicap@006d66a0`, `(h + 100) × rate / 100` (carried, `economy::gather_handicap`) | +50 where Easiest was −35: the AI's stockpile grows ~2.3× as fast | who=1's first payment, and the first scripted purchase that becomes affordable sooner — the draw stream departs from run54's there | who=1's `income` and `bucket` agree with ours on every block to the parting |
| H2 | `production_ai_setup` step 2's clamp (carried, `d < 3`) | skipped: the stockpile is never cut | the first step 2 (6376 on the Easiest East Indies game, sooner here) | `bucket` agrees across the first frame `econ` is written |
| H3 | `get_mod_resource_cap` (carried, §33.2) | ×1.0 where Easiest read ×0.5 | the rate pass's `rate[g]` | `rate` agrees on the first step 2 |
| H4 | `research_techs`' age pacing, epoch quota and the `≥ 4` gather-rate weights (carried, §2.14) | unpaced, uncapped, weighted by `econ` | the first `produce_tech` (8182 on Easiest) | the `MAKE` list's `t`/`val` agree on the block after the first tech purchase |
| H5 | `create_units`/`upgrade_units`: the matchup coin (drawn on every difficulty but 2) and `unit_prod_value`'s negation on 0 and 1 (carried) | the coin still drawn; counters not negated | the first `create_units+0x642` draw | the draw count and the offered `t` agree on that frame |
| H6 | `produce_unit@006cb9e0`'s batch, `min(7, affordable)` above 1 (carried, `seam_missile`) | a military purchase queues up to seven | the first military purchase | the trainer's queue agrees on the block after it |
| H7 | `found_cities@006c7a60`'s human cap (carried) | uncapped at ≥ 3: a third city | the first site bought past two cities | the city count agrees at that frame |
| H8 | `create_buildings`' five reads (carried: the Temple's capital-only `< 3`, the fort `0`/`1`, the `> 2` ×10 escrow, the wonder `< 2` gate and its `/100`, `/10`) | a Temple outside the capital, the fort arm open, a wonder priced whole | the first offer each reaches | the `MAKE` slots agree on the block after the offer |
| H9 | `plan_strategy@006b9620`'s forced bit 4 below 2, `Army::do_mustering@006f4260`'s `DEFENDING` below 3, `find_target`'s gate and its `> 2` arms, `find_local_army` (carried) | a region's strategy is its own; an army that is not weak marches | the first army leaving mustering (7930 on the Easiest Great Lakes game) | the army's captains' orders agree on that frame (no dump writes `ARMYDATA`, §71.2) |
| H10 | `Object::take_damage`'s `frame_attacked` stamp below 2 (carried, §71.3) | never stamps | the first hit | `0/frame_attacked` keeps its initial value in the dump through the parting |
| H11 | `Leader::diplomacy@006bc950` (**not carried**, `ai_drive` does not step it): `+1` at 3 and `+2` at ≥ 4 to a human's score, and a `> 2` count over the other leaders | a larger score toward the idle human | a `Leader::diplomacy+` label among the original's draws | the parting frame's original draws carry no `Leader::diplomacy` label |
| H12 | the repair arms: `Unit::do_gather@005ef2a0` and `Build::process@0061edf0` above 1, `Unit::find_repair_spot@00604320` and `Unit::think_spellcaster@005f27a0` below 2 (**not carried**) | an AI repairs a damaged building | the first AI building below full hits | no who=1 building carries damage before the parting |
| H13 | `Object::find_nearby_target@00648da0`'s AI flag at 0 (carried, `search_ai`) | off: the damage weight is not inverted | the first AI target choice between two candidates | every unit's target agrees to the parting |
| H14 | `Game::init_handicaps@0058abf0` (**not carried**) | a per-leader handicap slot at setup | read only by `get_handicap` under `semaphore[0] & 4`, off here | a start-dump field of run346 differs from run54's other than `DIFFICULTY` and what H1 moves |

### 80.3 What was expected to part first

**As a hypothesis** (DECISIONS 42), written before the captures: no
script under `game/ai/scripts/` reads the difficulty, so the scripted
opening plays the same orders at a different pace, and H1 moves the pace
from the first payment. The arithmetic of H1 is the original's to the
operator (`Leader::do_gather`'s `(iVar5 + 100) * uVar10 / 100`), so the
stream should depart from run54's early and this crate should follow it.
The first parting is expected at **H2 or H3**, the first C++ producer
step on a stockpile no Easiest capture has held — or at **H11**, the one
arm with draws this crate never makes. The order of H4–H9 is the game's.

### 80.4 What the captures read

- **run346** (East Indies) and **run347** (Great Lakes) read back
  `DIFFICULTY 5`, their `GAME INFO` blocks otherwise run54's and run53's
  line for line, player blocks included (`docs/RUNS.md`). **The `-config`
  lobby did not take the difficulty**: run347's first take kept `-config
  check.ini` with `difficulties=Toughest` there and `DIFFICULTY=5` in the
  profile, and read back 0 — `load_from_config`'s `diffs` category is
  read, but the lobby it builds keeps 0, as `mapstyles=` never takes. The
  take is run53's stream on all 24,001 frames (`rngcmp.py`), kept aside
  as `…-take1-diff0`. Without `-config` the lobby is the profile's, whose
  blocks give run53's lobby but for the style, so the second take is run53's
  game at 5. **This kills §80.1's reading of the Great Lakes lever.**
- **The idle human loses both games**: `defeated_by 1` on the closing
  block, run346 at frame 18141 and run347 at 5931. The Easiest pair never
  saw a battle; this one ends in one.
- **Both words were 0**, one arm: frame 0 spends 182 draws against this
  crate's 175 on East Indies and 121 against 120 on Great Lakes, parting at
  index 24 on `Unit::think_spellcaster+0x413 < Unit::think_scout+0x7c`.
  **None of H1–H14 was the first to part**: H12 listed `think_spellcaster`
  among the repair arms by its `< 2` read, and the arm that parted is the
  one past it — the reading named the function and missed that it draws.
  The expectation (H2, H3 or H11) was wrong; the frame named it.

### 80.5 `think_spellcaster`'s coin, built

`Unit::think_spellcaster@005f27a0` for a computer leader's unit, as
`sim::spellcaster` builds it, in the listing's order: `get_diff() < 2` →
0; `unit_masks & 1` (a decoy) → 0; `is(SPY)` → the Spy's arm; not
`is_special` (`unit_flags2 & 0x10`) → the hero arm; a special unit — every
Scout — tries the Sniper (`0x281`, the Commando's craft), then throws
`Random::get(game_random, 0, 0xffff)` returning to `005f2bb3` (the listing:
`call 0xa39d70`, `cltd`, `idiv $3`), and with `coin % 3 != 0` and no army
(`Object::get_army < 0`) returns 0; else it tries Counterintelligence
(`0x277`, which a Scout casts). `think_scout`'s head calls it for a
computer leader's caster (`unit_flags2 & 2`) after the goody box.

**The seams**, each drawing nothing where it stands: the three crafts'
target searches (`spell_valid_target` refuses Bribe, Counterintelligence
and the Sniper), the Spy's cloak, the hero arm (a General's Create Decoys,
Forced March behind a coin, and Ambush), and the human arm `Unit::think`'s
special turn reaches. The third caller, `Army::use_scouts`/`use_spies`/
`use_generals` on an army's 128-frame turn (`army.rs`, "the spellcasters'
turn — a seam"), is **not wired**: at Toughest an army holding a scout
would throw the coin there.

**What it moved**: East Indies **0 → 10**, Great Lakes **0 → 1**; both
first-pair words hold at 24,000. The value diff on run349's block 1: the
AI scout `1/0`'s walk target (41976, 36600) against (35832, 42744) →
(35832, 42744) on both, its path 9 slots against 3 → 3, and all 28 of its
rows on blocks 1..3 closed (the next first parts on block 97). The 119
keys still standing on block 1 are not the coin's: the draws agree through
frame 9, and they are player 0's census, `SITE.reg`, `form` and the city
record (§33.4's families), two AI citizens' idle clocks and 46 gaia
animals' `cur_anim`, unchanged by the fix.

~~**The new words, by frame and draw delta** (DECISIONS 42; no mechanism
is named): **Great Lakes 1** — ours 54 draws, the original 85, parting at
index 44, ours `Unit::do_non_flat_gather+0x54b` where the original spends
`Leader::produce_building+0xc99`; the lower word, widened on run350. On
its block 2 who=1's record parts: `script_step` 11 here against 6 there,
`bucket` timber 28 against 92 and wealth 50 against 100, `gatherers` 4
against 3, `tech_frame` 0 against 1 (the original stamps a tech on frame
1), and pools 64 and 66 hold `2005` and `2000` there and nothing here;
90 of its 130 keys are `1/2001`'s gather slots in another order, and the
citizen `1/2` stands here where it walks there. Block 1 stands with the
draws agreeing (58 keys, §33.4's families).
**East Indies 10** — ours 197, the original 188, parting at index 183,
ours `PathFinder::calc_road_cost+0x46` where the original spends
`Farms::inc_time+0x1ae`; inside run349, where no row first parts on blocks
4..25.~~ **Both were the harness's, not the game's** (item 979, §81): the
walk installed an Easiest sibling's frame words over each Toughest game.
With the lobby gate the words are **East Indies 1576** and **Great Lakes
3776**.

### 80.6 What this has *not* established

- **The arms past the first**, H1–H14 as written: none has yet been
  compared, since every one sits past frame 1.
- **The target searches** of the three crafts, and so **the `% 3` coin's
  consequence**: with no cast following either branch, inverting the test
  fails nothing (the mutation was run); the army read (`get_army`) is
  this crate's `army_of`, never compared.
- **The hero arm and the army turn** (§80.5): no General and no army with
  a scout stands before either new word.

### 80.7 Coverage

**Diff-backed**: the coin's frame, site and count on both maps' frame 0
(the trace), and the scout's walk on run349's blocks 1..96; the lobby
lever by four `GAME INFO` read-backs. **Listing-backed**: `005f2bae`–
`005f2bbd`. **Export-backed only**: the arm's order above the coin, the
three craft ids off the spell table's `+0x9dc`/`+0x9d4`/`+0xa04` slots,
and the seams' reads.

## 81. A sibling's frame words are another game's at another lobby, and the second pair moves to 1576 and 3776 (2026-09-27, item 979)

Item 979 was booked on Great Lakes' frame 1: ours 54 draws against the
original's 85, parting at index 44 (`Unit::do_non_flat_gather+0x54b`
against `Leader::produce_building+0xc99`). The widening (run350, block 2)
had who=1's `script_step` 11 here against 6 there. **No mechanism was
named**, and the one the booking carried — that the top difficulty changes
the AI's opening — was wrong: the game's rules were never the cause.

### 81.1 The frame, from the disk

- **Which side holds what** (`diff::second`'s rows, then the dump):
  the original's who=1 sits at `defensive` step 6, holds 2 food, 92 timber
  and 100 wealth, and on block 2 has queued The Art of War (TypeIndex 572,
  120 food, at the library `2005`), three citizens at `2000` and two farms
  (`2006`, `2007`; `num_queued {50: 3, 417: 2, 572: 1}`). Ours had gone to
  step 11 — Written Word, a fourth farm, City State, three citizens —
  spending 28 timber and 50 wealth less than it holds.
- **That is `defensive.bhs`'s Ancient Age rush**: `step == 6 &&
  rush_build < 4` on a land map places farms to five, researches The Art
  of War, trains to nine citizens and returns `BLOCK_ON_THIS` with the step
  unmoved (§3's step list). So the question was the roll.
- **The trace's seeds, first** (the 922 rule): frame 1's eight
  `MathUtilFuncSet::rand_int+0x18` records carry seeds `f292f08c` …
  `c0451f69`; one LCG step and `Random::get(1, 10)`'s scaling give
  **9 3 4 9 6 3 2 4**. This crate's host rolled **9 9 3 4 9 6 3 2** — the
  same stream one draw behind, so `rush_build1` (the script's `who − 1 =
  1`) read 9 here and 3 there.
- **Where the draw went.** The original's game stream is continuous across
  frames 0 and 1 (every seed one step from the last, 206 draws). Ours ended
  frame 0 on `f292f08c` and began frame 1 on `b6194ba1`, the word before
  frame 0's last draw: between the two, `Built::tick` installed a frame-0
  word from `frame_seeds`, and run347's own dump prints none. It was
  **run12's**, borrowed by `borrow_from_siblings` because run12's setup
  stream ends on run347's word — the setup draws do not read the
  difficulty — and run12 is an Easiest game, whose frame 0 is one draw
  short of the Toughest one (§80.5's coin).

### 81.2 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| the top difficulty changes the script's first steps (971's) | the script reads no difficulty (§80.3), and the rolls differ | killed by the seeds |
| the static initialisers run in another order | the original's rolls are ours shifted one draw, not permuted | killed by the seeds |
| a draw this crate does not step | the original's stream has no gap, and ours ends frame 0 on the right word | killed: ours is right, and then overwritten |
| a sibling's word from another game is installed at frame 0's end | the gate below moves the word; without it the unit test and four walks fail | **held** |

### 81.3 The fix

`diff::setup::borrow_from_siblings` takes a sibling's per-frame words and
per-frame clocks only when `same_lobby` holds: every `GAMEINFO` field
equal, or either dump without the block. The start dump's figures are the
setup's and still travel on the setup stream alone. On disk, within one
map and seed, the only `GAMEINFO` variants are run1–5's (`GAME_RULES`,
`flags`, `STARTING_TECHNOLOGY`), run20's `flags 1`, and the second pair's
`DIFFICULTY 5`. The fix is the harness's. The simulation is unchanged.

### 81.4 What it moved

- **Great Lakes 1 → 3776.** On run350's block 2, `script_step` 11
  against 6 → 6 on both, timber 28 against 92 → 92, wealth 50 against
  100 → 100, `gatherers` 4 against 3 → 3; citizen `1/2`'s walk,
  `1/2001`'s 90 gather slots and the rush's farm `2007` all agree. Frame
  1's draws went 54/85 → 85/85, and run350's keys parted 576 → 64.
- **East Indies 10 → 1576.** run38's words had been installed the same
  way. On run349 the keys parted 576 → 72: block 1's 46 gaia `cur_anim`
  rows and `1/1`'s `g.end_time[0]` 232 against 33 (run38's clocks)
  closed, and the scout `1/0`'s block-97 walk parting (`orders_x` 38136
  against 41976) closed; its first row is now `mylos` 6 against 4 on
  block 202.
- **The new words, by frame and draw delta** (DECISIONS 42; no mechanism
  is named): **East Indies 1576**, the lower: ours 272 draws, the original
  216, parting at index 192. Ours spends `Build::find_gather_tiles+0x10a`
  where the original spends `Animal::think_bird+0x82`. **Great Lakes
  3776**: ours 223, the original 217, parting at index 180 on the same
  pair of sites.
- **The first pair is untouched**: its captures' `GAMEINFO` is their
  siblings'.
- **The new words' widenings** (run352, run355; `docs/RUNS.md`) have the
  same shape on both maps.
  - On the word's block who=1 has a new building: `1/2009` on East Indies,
    `1/2010` on Great Lakes. Its gather slots hold other tiles (91 and 79
    keys), and it stands at damage 1 here against 0 there.
  - Around it, citizens take other orders.
  - On East Indies the block before the word already parts, on
    `reg_land[11]` (102 against 103) and `2007`'s `city:space`. On Great
    Lakes nothing parts between the window's first block and the word's.
  - ~~As a hypothesis only (DECISIONS 42): the site's placement, as ours
    spends `Build::find_gather_tiles` where the original does not.~~ Both
    spend it; ours shuffled 62 tiles where the original shuffled 48,
    because a camp the script destroyed on 976 kept its ground marked
    here (§82). Great Lakes' 3776 was the same close.

### 81.5 What stands on the old words' blocks

These draw nothing before either new word, so none of them is built here:
- **`tech_frame` and `tech_cat_frame[cat]`**: `Build::queue_up@00620f40`
  stamps them to `game->frame` when a tech is queued, and this crate does
  not (§33.4's named unmodelled state). The rows are 0 here against 1 there
  on both blocks 2.
- **Pools 64 and 66** on Great Lakes: the original seats the library
  `2005` and the city `2000` in groups `who·64 + 0` and `+ 2` when the
  script queues at them. This crate seats none.

### 81.6 What this has *not* established

- **Whether any other capture on disk borrowed another game's words.**
  The gate is by `GAMEINFO`, and two captures of one lobby that differ past
  the setup (a console command, a different input) would still pass it.
  §81.3's scan names only the second pair and runs 1–5 and 20 as differing,
  and every walk of those passed the gate unchanged (§81.7).
- **The rush past its first frame**: the barracks, four Hoplites and a
  Slinger. They lie on the way to 1576 and 3776, and the words cross them.

### 81.7 Coverage

**Diff-backed**: the eight rolls against the trace's seeds, the seed
chain's continuity, block 2 on both widenings, and both walks. Each walk is
a floor, and each fails with the gate removed (the mutation was run: the
unit test, run346's and run347's walks, and run349's and run350's
widenings). **Export-backed only**: `Build::queue_up`'s stamps (§81.5),
and `defensive.bhs`'s rush branch (read in the script itself).

## 82. A destroyed camp keeps its ground, and the second pair moves to 5606 and 4555 (2026-09-27, item 989)

Item 989 was booked on East Indies' frame 1576: ours 272 draws against
the original's 216, parting at index 192, ours
`Build::find_gather_tiles+0x10a` where the original spends
`Animal::think_bird+0x82`. **No mechanism was named**, and the one §81.4
carried as a hypothesis was wrong: both sides spend the shuffle.

### 82.1 The frame, from the disk

- **The draws, both sides** (the walk's site list on 1576): ours 248
  `find_gather_tiles+0x10a` then 21 birds and 3 farms; the original 192,
  then the same 24. The shuffle is `4 × length`, so ours listed 62 tiles
  and the original 48. That is the whole delta.
- **The camp.** On run352's block 1577 the original's `1/2009`
  (`orig_type 418`, a Woodcutter's Camp) stands at (38016, 36480), 48
  tiles, city 1, `frame_started −1`. It is on neither block 1571 nor
  1576: it was placed on frame 1576.
- **Ours' camps**, by a scratch print in `find_gather_tiles` and
  `close_building`, removed before the first commit:
  - frame 976: `2009` placed at **(38016, 36480), 48 tiles**, and closed
    the same frame, unstarted;
  - frame 1576: `2009` placed at **(31680, 34752), 62 tiles**.
- **Who closed it**: the script host's `destroy_building`.
  `aibestbuildlibrary.bhs`'s `place_woodcutter` places a camp, reads
  `max_workers_at_building`, and destroys it when it is under `min_size`
  (5), dropping `min_size` by one. Frame 976's 192 draws agree on both
  sides, so the original placed and destroyed the same camp. On 1576,
  with `min_size` 4, it placed one again: the original on the same
  ground, ours not.
- **`destroy_building`** is `ScenarioFuncSet::destroy_building`,
  vtable `+0x150`, which is `Build::close`. Its tail gives the list's
  tiles back (`docs/ECONOMY.md` §17.1). `sim::city`'s `close_building`
  cleared the footprint and never the list.

### 82.2 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| ours places the camp by another rule (`produce_building`'s spiral) | the original places the frame-976 site again, at the site ours picked on 976; the spiral scored ours' 1576 site because the old one's cells carried `0x1000` | killed: the spiral is right on the ground it is given |
| the tree tiles differ (a gatherer cut one) | ours' 976 list and the original's 1576 list are the same 48 tiles | killed |
| a closed camp's tiles stay marked here and are freed there | the close tail in the listing; with it, 1576 draws 216/216 and the camp is the original's; without it (the mutation), 1576 again | **held** |

### 82.3 The fix

`Build::close`'s tail, `GameDaemon::process_all`'s region cycle and
`Build::process`'s two re-entries (`docs/ECONOMY.md` §17). There is also
**a harness correction**. `diff::harness::link_building` links a dumped
`BUILDDATA` or `CITY` record to this crate's **live** building of its
number, and to a closed one only when none is live. The first building
that ever held a number had been taken, so from 1576 the original's
live `2009` was compared with ours' dead frame-976 site.
- That is where §81.4's `damage 1` against 0 came from: a closed site
  reads damage equal to its hits, which is 1 for a site never started.
- And `city −1` against 1.

### 82.4 What it moved

- **East Indies 1576 → 5606**, now the higher. On run352 the keys parted went
  573 → 90:
  - block 1577's 115 all closed: `1/2009`'s 91 tile keys, its damage and
    city, and citizen `1/2`'s walk (`orders_x` 31944 against 37752);
  - block 1571's capital `2000` `ter[1]`, 0 against 2 → 2 on both. The
    census's `gather_at` had read the dead camp's marks.
  - Frame 1576's draws went 272/216 → 216/216.
- **Great Lakes 3776 → 4555.** On run355 the word's block 3777 went 127
  keys → 0: `1/2010`'s tiles and damage, `2007`'s `city_down`, city
  `2000`'s `gatherers` 7 against 8, and citizens `1/6` and `1/28`. No
  key of who=1 parts on 3772..3804.
- **The new words, by frame and draw delta** (DECISIONS 42; no mechanism
  is named):
  - **East Indies 5606**: ours 4 draws, the original 5, parting at index
    0. Ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the
    original spends `Unit::do_move+0xe84`. Widened on run357: the
    block before the word parts on one key, `1/14`'s `order:move.dest`,
    0 against 1.
  - ~~**Great Lakes 4555**, **the lower**: ours 7, the original 3, parting at index 1.
    Ours spends `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4` where
    the original spends `Farms::inc_time+0x1ae`. Widened on run356: on
    the block before the word, `1/21` holds order kind 10 and two orders
    here against a move (kind 2) and one there.~~ **Moved to 4593 by item
    997** (`docs/COMBAT.md` §64): `1/21`'s attack-move look had taken the
    human's unarmed Woodcutter's Camp, which the original's `flags` word
    refuses.
  - ~~**Great Lakes 4593**, **the lower** (item 997; no mechanism is
    named): ours 4 draws, the original 5, parting at index 2. Ours spends
    `Farms::inc_time+0x1ae` where the original spends
    `Guy::set_anim+0xf2f < Guy::move+0x166`. Inside run356's window
    (block 4594). The block before the word, 4592, parts on `1/26`, a
    member of the same army: the original holds an `ATTACK` (kind 10, two
    orders), stopped at (5079, 31628), and on 4593 `recharging 33` and
    `hold_attack 1`; ours walks toward its attack position (kind 1, three
    orders, `orders_x/y` (4872, 31752)) at (5052, 31644). Earlier rows in
    the window: `1/0`'s turn on 4575 (18 keys, a unit parting since
    4550), and on 4585 a guy's `whom/ox` naming the city `0/2000` on
    `1/21`'s attack-move, the shape `1/24`–`1/26` stood in from 4550.~~
    ~~**Moved to 4605 by item 1002** (`docs/COMBAT.md` §65): `1/26`'s chase
    on the city is a building's, which `do_move` asks without the `0x90`
    margin.~~ **Great Lakes 4605, the lower** (item 1002): ours 2, the
    original 43, parting at index 0 on `Farms::inc_time+0x1ae` against
    `find_attack_pos < Group::action_attack+0x41a`. **Moved to 4618 by item
    1012** (`docs/COMBAT.md` §66): an army group's attack-move looks on
    4605 and hands the city to the whole group; the retarget searches
    buildings; `1/17`'s ring starts mid-face. ~~The new word, 4618 (ours 94,
    the original 4, at index 0: `Guy::set_anim < Unit::do_idle` against
    `Unit::do_non_flat_gather+0xcc3`), is §66.5's.~~ **Moved to 4673 by
    item 1014** (`docs/SCOUT.md` §8.3): the AI scout's rival multiplier is
    the met bit, not war, so on tick 4506 it walks to (2808, 32760) as the
    original does instead of idling on 4618. ~~**Great Lakes 4673, the
    lower**: ours 4, the original 3, at index 0, `Unit::fight+0x9b0` (the
    chaser `1/24`) against `Farms::inc_time+0x1ae`. Inside run356's window:
    block 4673 parts on `1/24`'s order (kind 10 against 1), and its chase
    spot has parted since 4617.~~ **Moved to 4688 by item 1023**
    (`docs/COMBAT.md` §67): `check_target_path` re-aims a ranged chase on a
    fleeing unit out of reach of the walk spot. ~~**Great Lakes 4688, the
    lower**: 35 against 35 at index 31, `Guy::set_anim+0x97a <
    Guy::move+0x19f` against `Farms::inc_time+0x1ae`; `1/24` strikes `0/3`
    on 4688 there and retargets to the scout `0/0` here (§67.5).~~ **Moved
    to 4690 by item 1028** (`docs/COMBAT.md` §68): `compare_target`'s RAID
    arm, a `Build` target handed to the group unsearched, and a building's
    `targeted` decay. ~~**Great Lakes 4690, the lower**: ours 5, the original
    4, at index 1, `Object::take_damage+0xe1` against
    `Farms::inc_time+0x1ae`; nothing parts on block 4691 (§68.5).~~ **Moved
    to 4781 by item 1034** (`docs/COMBAT.md` §69): a city hit by another
    player sets `city_flags` `0xe`, and `0x2` vetoes the city heal until the
    city building's 200-frame decay clears it; ours had healed the human's
    city off its first wound, so 4690's strike was a first wound again.
    ~~**Great Lakes 4781, the lower**: ours 5, the original 4, at index 0,
    `Unit::fight+0x9b0` against `Farms::inc_time+0x1ae`; the citizen `0/4`
    is struck on 4779 there and not here (§69.5).~~ **Moved to 4846 by
    item 1040** (`docs/COMBAT.md` §70): the Slinger's three release bays,
    and a busy unit that is not on duty does not answer a hit. **Great
    Lakes 4877, the lower** (after 4846 and 4852 in the same item: a
    building is struck square to its side, and the one-in-five retarget
    freezes the frame). ~~Great Lakes 4877: ours 7, the original 8, at
    index 1.~~ **Moved to 4924 by item 1052** (`docs/ARMY.md` §23: an
    engaged army forms a cell behind its point). **Great Lakes 4924, the
    lower**: ours 8, the original 7, at index 0, `Guy::set_anim+0xf2f <
    Guy::move+0x166` against `Guy::set_anim+0x97a <
    Unit::move_step+0x823`; inside run373 (block 4925).
- **The first pair** holds at 24,000 on both maps.

### 82.5 What stands, drawing nothing before either new word

- **A city still a site, counted by the original's sweep and not here.**
  East Indies' `2007` on run352's block 1571 reads `land` 9, `filled` 1
  there and 0 here, ~~with `reg_cities[11]` 2 against 1~~ (that row was
  `City::init`'s count, which ours lacked: closed by §84). Great Lakes'
  `2009` on run355's block 3805 has the same shape, beside who=1's
  `territory` 488 here against 290.
- **`2007`'s site picture one cell apart** from its sweep of 1575:
  `filled` one high here and the three `space` counts one low, so
  `reg_land[11]` 102 against 103.
- Neither moved with the fix. The first-pair residue §58.4 records for
  the same city is the same one-cell shape.

### 82.6 What this has *not* established

- **The re-walk and `verify_gather_tiles` in a capture**: neither is
  exercised on disk (`docs/ECONOMY.md` §17.4).
- **Whether the harness's old link hid other partings.** Every widening
  that walks a number a closed building gave up now compares the live
  holder. The gate's widenings are the census of what moved.

### 82.7 Coverage

**Diff-backed**: the camp's site and list by run352's and run355's word
blocks, and frames 1576 and 3776's draw counts; each is a floor that
falls back under the close mutation. The harness link is backed by the
same blocks; they fail under its mutation, and so does
`a_dumped_building_links_to_the_live_holder_of_its_number`.
**Script-backed**: `place_woodcutter`'s destroy, read in the script
itself.

## 83. The third scored map: Great Sahara, and its first word at frame 8 (2026-09-28, item 1066)

DECISIONS 54 §3 scores a third map in the first pair's lobby: Easiest,
seed 12345, the profile's lobby, and `MAP_STYLE 7` in place of 14 or 18.
With only the map moved, a parting names the map and not the lobby. The
floors are `rondata::diff::FLOORS`' `GreatSahara` row and
`LONG_WORD_GREAT_SAHARA`, and `rondata::diff::third` holds the tests. **No
mechanism is named here, and none was built** (§3 of the entry). A later
pass may choose the third map again before an item is opened against its
word.

### 83.1 The lever, and the read-back

`-config check.ini` pins the style at 14 (`docs/ORACLE.md`, "The lobby is
a file"), so all three captures take the profile's lobby, which
`mapstyle.py` writes. That is run39's path and run348's. Each dump's
`GAME INFO` has 85 lines, and on all three they are run39's line for line
except `MAP_STYLE 7`. `DIFFICULTY 0` and seed 12345 read back on each. The
profile read `DIFFICULTY 0` in `<SOLO>` and `<MULTI>` before every launch,
and it was put back after each (`docs/RUNS.md`, run381–run383).

### 83.2 What the map holds

This is run381's `DUMP_ALL` head, read with the harness's own parsers
(`rondata <install> --gamelog … --diff`) and a count of the `WORLD`
block's cells.

- **All land, no water.** The map is 60 × 60 cells, and **all 3,600 cells
  are `BASELAND`**: there is no `SANDY` and no `OCEAN`, where East Indies
  has 755 `OCEAN` cells. `sea_resources` is 0. The `WORLD` scalars read
  `forest_size 151`, `mountain_size 138`, `rock_size 42`, `total_metal
  55`, `total_oil 10`, `goodies 22`, `land_resources 14`, map seed 7236.
  There are two regions, nine placed mountain ranges (134 solid cells),
  two herds, 24 goods (14 linked to a cell) and six farms, none of them a
  pasture.
- **West against east.** The human's capital `0/2000` stands at (6240,
  30048) and the AI's `1/2000` at (40800, 16992), on a map 46,080 wide.
  The nations are the first pair's: tribe 4 against tribe 11.

### 83.3 Standing up, and frame 0

run381 is its own sibling. Its head carries the height table, the cells,
the regions, the herds, the goods and the checksum trace, and the
simulation stands up on it with **nothing borrowed from another map**.
**Frame 0 is 99 draws against the original's 99 on the first try.** The
setup's personality roll lands where the original's does (`0x2f3cdb21` →
`0x76137975`), and the ten starting citizens' derived gather targets are
the ones the original issued. East Indies' frame 0 is 175 draws for
comparison. Why the two differ was not read.

### 83.4 The score and the word

- **run382 scores ticks 6, orders 5**
  (`run382_s_desert_game_is_the_third_map_s_score`). The first order
  parting is the AI citizen `1/2` on frame 6, `Move.dest_x` 38232 against
  38328. Its position parts on frame 7. Player 0 first parts on frame 103,
  on the six citizens' first walk (`0/3`–`0/5` and `1/3`–`1/5`, `Move.x`
  192 apart).
- **The word is frame 8 on both captures**
  (`run382_s_trace_says_where_the_third_map_s_word_parts`,
  `run383_s_long_trace_says_where_the_third_map_s_word_parts`). Ours
  spends 6 draws and the original 7, parting at index 0. The original
  spends `Guy::set_anim+0x97a < Unit::move_step+0x823` first, and ours
  `Farms::inc_time+0x1ae`. Frames 0..7 agree draw for draw. run383's
  first 1,851 frames are run382's word for word (`rngcmp.py`), so the
  long capture adds nothing before the word.
- **run383 ran the whole 24,000.** Its trace holds 24,001 frames, the
  game ended on the `!quit`, and block 24001 is the shutdown dump with both
  leaders at `defeated_by -1` (scores 421 and 2,495).

### 83.5 The widening: the first parting, by unit and field

`run382_s_word_frame_is_widened_whole` walks run382's blocks 1..259 (the
word's block 9, every block before it, and 250 after it) from run382's
start with run381's head. It compares every record run10's detail prints,
both directions: units, buildings, figures, orders, cities, both leaders'
short records and gaia's eight animals. The dump prints no `GROUPDATA`, so
no pool or group row is walked, and the 1,060 keys it does not print
(the long leader record and gaia's clocks) are pinned as unprinted, not
as agreeing. There are 244 keys parted over the window.

| block | unit | field | ours | theirs |
|---|---|---|---|---|
| 1 | ten citizens and the scout | `form` | −1 | 9 (0 on `0/0`) |
| 1 | `0/2000` | `city:filled`, `land`, `space[0..2]`, … | 0 | 53, 97, 57/57/44, … |
| 6 | `1/2` | `collide` | 1 | 0 |
| 6 | `1/2` | `order:move.dest_x`, `dest_y` | 38232, 16056 | 38328, 15768 |
| 6 | `1/2` | `path:length` | 1 | 2 |
| 7 | `1/2` | `pos` | (38352, 15814) | (38328, 15768) |
| 8 | `1/2` | `collide_who`/`collide_o` | 1/1 | −1/−1 |
| **9** | `1/2` | `collide_frame` | 6 | 8 |
| **9** | `1/2` | facing, heading | −328728576 | −1925840896 |

Block 1's 22 keys are the first pair's standing families (§33.4), and
run350's block 1 has them too. **The first parting past them is one
unit, the AI citizen `1/2`, and it is a collision's timing.** Ours marks
the collision on frame 6 and walks a one-leg side-step to (38232, 16056).
The original holds its two-leg move to (38328, 15768) and marks its
collision on frame 8. On block 8 the two destinations have swapped sides.
Both sides name the same partner, the citizen `1/1`: ours from block 8,
the original from block 10. On the word's block both stand at (38328,
15768) again, and what parts is the collision's age and the figure's
facing. The original's frame 8 opens on a `move_step` animation draw that
ours does not spend. `1/1` itself first parts on block 14. Gaia's
animals agree until block 31.

### 83.6 What this has *not* established

- ~~**Why `1/2` collides two frames early.** The rows above are the value
  diff. No reading was done and no mechanism is named, by the entry's rule.~~
  **Answered by item 1133 (§88, `docs/COLLISION.md` §20)**: ours collided
  on the same frame as the original and refused the sidestep the
  original took, on a corner of `1/1`'s block that `1/2`'s own exit from
  the camp at setup had cleared in the original.
- ~~**Whether the parting is the map's.**~~ It is the start's: the two
  citizens born at the camp `1/2001` (§88). Nobody has checked whether the
  first pair's early frames hold a collision between the capital's first
  two AI citizens. Both of those maps are closed, so if they do, the
  question becomes which rule this map's geometry reaches that theirs did
  not.
- **Anything past frame 8.** run383 holds the draw stream to 24,000 with
  no per-frame record, so a word past run382's 1,850 blocks needs a
  capture sized to it (DECISIONS 50 §7).

### 83.7 Coverage

**Diff-backed**: everything in §83.2–§83.5. The map's contents come from
run381's head through the harness's parsers. The score, the word and the
widening are pinned by `rondata::diff::third`'s four tests, and each pin
was raised by one and failed. The coverage driver reads run382's blocks
7..11. **Reading-only**: nothing.

## 84. A border fix reaches the map 256 cells a frame, and the second pair's East Indies word moves to 5773 (2026-09-28, item 1106)

Item 1106 was booked on East Indies' frame 5606 at Toughest: ours 4 draws
against the original's 5, parting at index 0. Ours spent
`Guy::set_anim+0x97a < Guy::inc_time+0x271`, the original
`Unit::do_move+0xe84`. The block before, 5605, parted on one key, `1/14`'s
`order:move.dest`. **No mechanism was named**, and the booking's framing,
that the original's move was the one that drew, was the wrong way round:
ours had sent `1/14` three ticks early.

### 84.1 Every instance first

- **With the group record in the walk** (`docs/GROUPS.md` §33.3), run357's
  block 5601 parts on 172 keys. `1/14` is a Citizen, alone in pool slot 67,
  and it had parted before the window. Its position is (36525, 23051)
  against (36908, 23630). Its stack is a `BUILDORDER` on the site `1/2019`
  under an `EXPLORETOORDER` to (38328, 23304) here and (38328, 23352) there.
  Group 67's point is (29568, 23424) against (30336, 24960), and its
  `stamp` 5518 against 5521.
- **Ours' history** (`RON_DEBUG_UNIT` on the long walk): `1/14` is born on
  2630, explores, and builds the AI's third city `1/2017` from 4918. It is
  idle on 5518 and takes a group move west on that tick.
- **The pushing call chain** (a scratch backtrace, removed): `think_peasant`
  → the colonist arm `think_civilian_transport(u, 1)` → `push_group`. The
  cell it picks comes from a strided search whose phase is `o + frame`
  (`005f40d0`), so three ticks move the cell.
- **The draw stream agrees through 5605**, so the decision drew nothing on
  either side.

### 84.2 The packet, and the gate that refused

The colonist arm's gate (`005f40d0`) has three conjuncts: `xport_peasants <
city_num` (`+0x9c0 < +0x3f8`), the unit's cell owned by its leader (`world
+0x134`, `+0xf`), and `reg_cities[region] != 0` (`+0x125e`). No dump prints
the cell byte, so **run407** is a packet at logger frame 5518
(`docs/RUNS.md`). Its log window prints the rest:

| block | `1/2017` | `1/14` | who=1 |
|---|---|---|---|
| 5517 | `job_counter` 59900 | building | `reg_terr[11]` 269 |
| 5518 | finished (flags 39) | no order, `idle` 0 | `city_num` 3, `reg_cities[5]` 1 |
| 5519 | | `idle` 1, group 66 | `reg_terr` all 0 |
| 5521 | | `idle` 2, group 66 | `reg_terr[5]` 34 |

The packet: the cell (48, 29) is **unowned**, and every land region's
resume index is **0**. The city's `City::init` fixed the borders
(`00737050`, the loop over `+0x2c`), and nothing had yet written the new
ground.

### 84.3 The listing

- `GameDaemon::check_borders@00732060` runs once a frame. It zeroes
  `borders`, then walks the 64 land slots and calls
  `World::compute_reg_territory` on each whose index is below its size.
- `compute_reg_territory@006b0bb0` zeroes the region's `reg_terr` when its
  index is 0. It returns early when `0xff < game_daemon->borders`.
  Otherwise, for each cell from the index, in the region's list
  (`+0x7c`), it bumps `borders`, computes the claim, and writes the cell's
  owner and runner-up bytes. That is **256 cells a frame across all the
  regions**. A region that reaches its end sets `0x2000000`, the economy
  flag, on every live leader. `check_borders`' tail sums `territory` once
  every region is done.
- The packet's regions hold 260, 153, 139, 182, 151, … cells, and the cell
  is region 5's 113th. The pass writes region 1's first 256 on tick 5518.
  On 5519 it writes the rest of region 1, region 2 and 99 of region 3. On
  5520 it writes the rest of region 3, region 4 and region 5's first 34:
  **the dump's `reg_terr[5]` of 34 on block 5521**. On 5521 it reaches the
  cell, the gate passes, and the group is stamped 5521.
- `docs/ATTRITION.md`, "Territory", had described this pass since
  2026-08-20. This crate recomputed wholesale at the fix, as a stated
  simplification "until a recorded-game diff says it matters". This is
  that diff.

### 84.4 A second writer, and what killed each reading

| reading | killer | verdict |
|---|---|---|
| the city finishes later in the original | `1/2017` finishes on block 5518 on both sides | killed |
| the idle gate waits longer there | `1/14` is idle on 5519 there, as here, with `0x40000` (threshold 1) | killed |
| `xport_peasants < city_num` refuses | 0 against 3 on every block | killed |
| the cell is not yet the leader's | unowned on the packet; the pass reaches it on 5521 | **held** |

With the pass built, ours' gate refused on 5518 too, but the think fell to
its tail. There `reg_cities[region] == 0` sends a citizen to
`think_scout(0)` (`005f5760`), and ours explored. The dump reads
`reg_cities[5]` 1 on block 5518. **Ours' only writer was the census
sweep's recount** (step 8). The original has two more: `City::init`
`+= 1` (`00737050`, beside `reg_pop`) and `City::close` `−= 1`
(`00737550`), both on the city building's cell region below `0x40`.

### 84.5 What this crate built

- **`sim::border_pass`.** A fix in play (`Sim::sync_territory`) computes
  the target owners and resets every land region's index. The daemon's
  pass (`Sim::check_borders`, before the unit loop) copies up to 256 cells
  a frame, land regions in order and row-major within each (`Regions::
  rebuild_coords`' order). A finished region marks every ledger dirty, and
  the pass's end updates the holdings.
- **A fix before the first frame is setup's** and lands whole
  (`Sim::in_play`), as `compute_all_territory`'s unlimited budget does.
  `Sim::settle_borders` finishes a pass at once.
- **`reg_cities`' lifecycle writers** in `init_city` and `close_city`
  (`Sim::bump_reg_cities`).
- **Unit tests**:
  - `border_pass::tests::a_fix_reaches_the_map_at_256_cells_a_frame_in_region_order`;
  - `cities_tests::a_city_finished_in_play_counts_at_once_and_takes_its_ground_at_the_pass_s_pace`.

### 84.6 What moved

- **East Indies 5606 → 5773.** On run357's block 5601 the keys went 172 →
  139, and none arrived:
  - `1/14`'s 27 rows (`pos`, the route, the figure);
  - group 67's four (`ox` 29568 against 30336, `stamp` 5518 against 5521);
  - who=0's `gather_stamp`, 5520 against 5528;
  - `1/2017`'s `city:peasant_dist`, 2 against 1.

  Blocks 5605 and 5607 part on nothing. Frame 5606's draws went 4 against
  5 → agreeing.
- **The new word, 5773** (no mechanism is named): ours 4 draws, the
  original 3,209, parting at index 0. Ours spends `Guy::set_anim+0x97a <
  Guy::inc_time+0x271`, the original `PathFinder::calc_road_cost+0x46`.
  Inside run357 (block 5774): the Caravan `1/33`, trained on 5772, holds a
  route of two orders there and none here. The cities `1/2000` and
  `1/2017` each list one caravan fewer here.
- **The pass reproduces the original's own transient.** Chapter four's
  border widening (`chapter_four_s_border_is_widened_cell_for_cell`,
  run132) agreed cell for cell on 4 blocks, the settled ones after each
  lever. It agrees on 17 now. The 13 new blocks are the ones where the
  original's owner count is mid-sweep: 276, 282 and 285 after the Temple,
  308 … 319 after Religion, 346 … 406 after Civic 3.
- **Keys close on every other widening the pass reaches, and none opens**
  (the rows named in each pin, measured against the mutation below):
  - who=0's `gather_stamp` on run143, run149, run178, run192, run196,
    run202, run211, run218, run226, run227, the leader windows run91 and
    run107, and run352. A region's end raises the economy flag, and the
    reassembly now falls on the original's frame.
  - who=1's wealth `leftover` or `bucket` on run253, run257, run261, run269,
    run277, run289 (parked 851's standing row) and run96.
  - who=1's `territory` on run355's block 3805, 488 against 290, and on
    run227's 16528. `territory` is summed at the pass's end.
  - who=1's `reg_cities[11]` on run352's block 1571, 1 against 2. It is
    `City::init`'s count, not a site the original counted (§82.5).
- **The first pair holds at 24,000**, Great Lakes' second game at its end
  (5930), the third map at 8, and every golden chapter's walk.

### 84.6a The killers

Each ran on a committed tree with `git diff --stat` non-empty, and was
restored from git and `touch`ed after.

- **The pass wholesale in play** (`start_border_pass` lands the target
  at once), on `54aa9d20`:
  - both unit tests fail;
  - run346's walk falls back to 5606;
  - run357's `1/14` rows return;
  - run352's block 1571 reads 81.
  - **Diff-backed.**
- **`reg_cities`' lifecycle writers off** (`bump_reg_cities` a no-op), on
  `54aa9d20`:
  - the city test fails;
  - run346's walk falls to **5518**: the colonist gate refuses there and
    the tail scouts;
  - run352, run355 and run357 fail, and run352's `reg_cities[11]` returns.
  - **Diff-backed.**

### 84.7 What this has *not* established

- **`reg_terr` is still read live off the owners** (`Sim::reg_terr`), not
  zeroed and recounted by the pass. It follows the visible grid, so a
  region the pass has not reached reads its old owners where the original
  reads 0 (parked 568's transient, narrowed).
- **The city flags the pass writes** (`City +0x65`, the `0x1000` border
  contact) and **a fix mid-pass that changes the sources** are still
  applied from the fix's target. The sources change only through a fix
  here, and a fix restarts the pass.
- **Sea cells** are never passed. This crate writes them `None` at the fix
  and they stay `None`, as the original's do.

### 84.8 Coverage

**Diff-backed**: the move of the word and every closed key above, by
run346's walk, run357's widening and the widenings the gate runs. **Packet-
backed**: the cell's owner and the regions' indices on 5518 (run407).
**Listing-backed**: the budget and the order, which the dump's
`reg_terr[5]` of 34 on 5521 confirms, and `City::init`/`City::close`'s
`reg_cities` writers.

## 85. A caravan crosses the water, and the second pair's East Indies word moves to 5975 (2026-09-28, item 1115)

Item 1115 was booked on East Indies' frame 5773 at Toughest: ours 4 draws
against the original's 3,209, parting at index 0, ours `Guy::set_anim+0x97a
< Guy::inc_time+0x271`, the original `PathFinder::calc_road_cost+0x46`. On
run357's block 5774 the Caravan `1/33` held a route of two orders there and
none here. **No mechanism was named**, and the booking's direction was open:
ours might refuse the trade, or take it and fail the plan.

### 85.1 The parting's instances first

- **`1/33`'s fields walked back.** It is trained on 5772 on both sides
  (`Guy::init_real` on 5771). On block 5773 the original holds a
  `TRADE_ROUTE` with `ox 2000` (London) and `oxx −1`, `flags 0`; ours the
  same order with `flags 4`. On block 5774 the original's order has `oxx
  2017` (Newcastle), a move to (39288, 40056) ahead of it, and the route
  links the two cities (`CARAVANLINK cara 1` on London and Newcastle); ours
  has no order and `idle 99`.
- **Every trade route on the disk.** who=1 has two: `cara 0`, London ↔
  Norwich, carried by `1/15`, laid long before run357's window; and `cara
  1`, `1/33`'s. London and Norwich are region 11, **Newcastle region 5**,
  the island `1/14` founded on 5518 (§84). London's one route is taken, so
  Newcastle is the only free partner.
- **So ours refused the trade.** `idle 99` is `do_trade`'s no-partner arm,
  and ours' selection skipped every candidate in another region, a stated
  SEAM (`docs/CARAVAN.md` §11.1).

### 85.2 Three corrections, each read off the listing

- **The region tests** (`do_trade@005ed270`, `get_tregion` returning at
  `005ed483`, `005ed631`, `005ed920`): each difference is forgiven by
  `can_transport`, and `1/33` has it (`unit_masks 0x840000`). The word moved
  5773 → 5776: the search now runs, its per-frame counts agree for three
  frames (3,203, 3,206, 3,205), and 5776 parts at 3,201 against 3,206.
- **The shore surcharge** (`calc_road_cost@00686300`): with the ends in two
  regions, `+0xa8` is added only when the parent's tile is land.
  **run413** is a packet at logger frame 5776. The original's parked
  search, read from `CaravanData +0x28`/`+0x30`, held 171 nodes apart from
  ours with the surcharge on every sea tile, and **none of 1,437** with the
  parent test. The word moved 5776 → 5975, and the search's sixteen frames
  to 5788 agree.
- **The bearing** (`add_move_order@00616ed0`, `616edc`..`616f1f`): the
  facing is `find_angle(dest − here)` in `ecx`/`edx`, not the decompiler's
  literal pair. With it, and the trade order's cleared bit 4
  (`add_trade_order@005e4dc0`'s last argument 0), `1/33`'s move angle
  agrees on 5774, and so does the first caravan `1/15`'s leg on block 5601.

### 85.3 The killers

Each was a mutation on a committed tree, with `git diff --stat` naming the
one file, and was restored from git and `touch`ed after. The tests the
re-pin touches ran green first.

| mutation | unit test | run346's walk | run357 | run414 |
|---|---|---|---|---|
| the region skip unconditional | fails | 5773 | `1/33 orders.len` 0 against 2 returns | `1/35`'s `pos` parts on 5970 |
| the surcharge on every sea tile | fails | 5776 | `1/33`'s leg angle parts on 5820 | `1/35`'s `pos` parts on 5970 |
| the literal `find_angle(1, 0)` | fails | holds at 5975 | `1/33`'s `order:move.angle` returns | holds |

The first two are **diff-backed**. The third is a value only: the facing
moves no draw by 5975.

### 85.4 What moved

- **East Indies 5773 → 5975.** On run357 the window's keys went 472 → 167
  (after the first fix → after all three). They include block 5774's `1/33`
  rows (`orders.len`, `order:flags`, `order:action`, `order:move.angle`,
  `dest_angle`), `1/2017`'s and `1/2000`'s `city:vans.length`, and block
  5601's four `1/15` rows (139 → 135).
- **The new word, 5975** (no mechanism is named): ours 63 draws, the
  original 61, parting at index 49, ours `Unit::think_scout+0xaba` where the
  original spends `Guy::init_real+0x52`. It is past run357, so **run414**
  widens it (blocks 5970..6226). Block 5970 stands on 156 keys, all standing
  families. The word's block 5976 parts on the newborn `1/40`'s figure
  clocks, which read one seed later here.
- **Holds** on the gate: the first pair, Great Lakes' 5930, Great Sahara's
  8 and every chapter (the gate's own line is in the journal).

### 85.5 The word's scan, read on run415

The AI sea scout `1/35` scans sea region 0 (1,931 cells, stride 27): 72
cells, 46 accepted here and 45 there. Nothing on run414 parts a unit's
position before the word, so the extra cell is state that no dump prints.
**run415** is a packet at logger frame 5975, the state before the scan's
tick. Its fog grid (`WorldData +0x160`, 120 × 120) differs from ours after
tick 5974 on **three half-cells**, each seen there and not here:

- (45, 73), old: seen there already on run407 (logger 5518), off every scan;
- **(91, 81), cell (45, 40)'s probe point**, one of the scan's cells — the
  extra one ours accepts;
- (90, 80), its neighbour.

run413's grid holds (90, 80) and (91, 81) unseen at logger 5776, so the
original revealed them between 5776 and 5975. In ours the nearest who=1
unit in that span is the transport barge `1/36`, born on tick 5878 when the
citizen `1/22` (sight 2) boards at (33901, 32257). Ours puts the barge at
(34076, 32173), sight 6, a radius-3 disc from fog cell (88, 83), and both
half-cells lie at √13 from it. The trace has the barge born on 5878 there
too, and by 5970 both sides hold it at the same point. **What reveals the
two half-cells there is not established**: the barge's spot, its birth
reveal at the caster's point, or the disc's edge. No dump prints
5858..5969. A packet at logger 5879 would settle it. It is parked
(`docs/journal/2026-09-28-item-1115.md`), since it is a reveal mechanism
the booking did not name.

### 85.6 What this has *not* established

- **The census's newborn lag.** `Unit::set_type@00612fa0` adds 1 to
  `LeaderData +0x980` (`caras`) for a caravan type, and `Unit::close` takes
  1 off. This crate counts caravans only at its census sweep, so who=1
  reads 1 against 2 on run357's blocks 5772..5774 and 2 against 3 on
  run414's 5976. `create_units` and `Build::process` both read it. The same
  shape is in `peasants` (5704) and `scholars` (5682). Parked.
- **`do_trade`'s AI tail on no partner** (`go_to_city`), and
  **`add_trade_order`'s transport tail**: unreached by any capture.
- **The fog reveal above.**

### 85.7 Coverage

**Diff-backed**: the word's move, run357's closed rows, run414's block, and
the killers above. **Packet-backed**: the search node for node (run413)
and the fog grid on the word's frame (run415). **Listing-backed**: the
facing and the region calls. **Pinned capture-free**:
`a_caravan_trades_across_regions_only_when_it_can_transport` and
`a_sea_crossing_between_regions_pays_once_at_the_shore`.

## 86. A barge lights its disc on the water, and the second pair's East Indies word moves to 6151 (2026-09-28, item 1120)

Item 1120 was booked on East Indies' frame 5975 at Toughest: ours 63 draws
against the original's 61, parting at index 49, ours
`Unit::think_scout+0xaba`, the original `Guy::init_real+0x52`. §85.5 had
the AI sea scout `1/35`'s region scan accepting 46 cells to 45, and run415's
fog grid holding cell (45, 40)'s probe point (91, 81) and its neighbour
(90, 80) seen there and not here, lit between 5776 and 5975. **No mechanism
was named**, and the booking's direction was open (parked 1076). The brief
phrased it as a sight ours grants and theirs does not. §85.5's grid says the
opposite, and it is right: the original sees both half-cells, and this crate
does not.

### 86.1 The instances, and the rule

- **Every who=1 unit that could reach (91, 81)** on run414's first blocks:
  the Explorer `1/18`, six rows short; the citizen `1/22`, cargo; the barge
  `1/36`. Its `los_x/los_y`, written only by `Unit::init`, is its birth
  point (33912, 32280), half-cell (88, 84). Ours then puts it on the water
  at (34076, 32173), half-cell (88, 83), on tick 5878.
- **§3's octagonal distance** puts both half-cells at 3 from (88, 83) and
  at 4 from (88, 84). A radius-3 disc at the spot lights them, and the
  birth disc does not.
- **The rule** (`docs/VISION.md` §11): `cast_transport`'s
  `set_new_location(boat, x, y, 1, 1)`, read off the pushes at
  `671027`..`67102f`, crosses a half-cell and calls `update_seen(0)`, the
  whole disc at the spot. This crate moved the boat without a reveal.
- **No packet was taken.** The disk and run415 answered it. With the
  reveal built, ours' `seen2` equals run415's on all 14,400 half-cells
  after tick 5974, and run413's after 5775. So 1115's packet at logger
  5879 (run418) was not needed, and run418..run420 are unused.

### 86.2 What moved

- **East Indies 5975 → 6151.**
  - Frame 5975's draws agree, since the scan's `+0xaba` count is the
    original's.
  - **Value diff**: run415's `seen2` on (90, 80), (91, 81) and (45, 73):
    ours 0 against 2 → 0 of 14,400 apart. (45, 73) is the barge `1/21`'s,
    born on tick 4444 at (16128, 28960).
  - Block 5976's `1/40` `g.cur_time[0..2]` (ours 1 against 0) closed.
  - run414's keys went 675 → 327.
  - No key parts earlier than it did before the fix: every key the fix
    makes part is on block 6152 or later.
- **The new word, 6151** (no mechanism is named): ours 9 draws, the
  original 8, parting at index 0.
  - Ours `Guy::set_anim+0x97a < Unit::move_step+0x823`, the original
    `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0x10f`.
  - It is inside run414 (block 6152), so run414 is its widening.
  - On 6152 only the Caravan `1/15`'s 35 rows part. Ours stands it at
    (35597, 37251) against the Caravan `1/33` (`collide_o 33`, every figure
    `stopped`). The original walks it on to (35617, 37267).
- **Standing before the word**, on the same blocks as before the fix:
  - 5976: the census's newborn lag (parked 1122) and `1/40`'s `form`;
  - 5982: who=1's `MAKE[7].city`; 5984: `gather_stamp`;
  - 6001: who=0's `production_step`;
  - 6026 and 6069: `1/23`'s and `1/22`'s figure `avg_speed` and `mirror`;
  - 6071: group 74's `form`; 6074: `1/31`'s `path_recursion`;
  - 6111: the newborn `1/41`'s `form`, figure angle and `orders_x/y`;
  - 6135: group 72's `speed`/`new_speed`.
- **The compared pin** walks 6150..6154. It pins `BuildDump.job_counter`
  as a site the window never reaches, since no building is under
  construction there.

### 86.3 What this has *not* established

- The rest of `set_new_location`'s callers (`docs/VISION.md` §11.5).
- The value rows standing before 6151, above. Each needs its own reading.
- What stops `1/15` on 6151 here and not there.

### 86.4 Coverage

**Packet-backed**: the fog grids (`run413_s_and_run415_s_fog_grids_are_ours`).
**Diff-backed**: the word's move and run414's block 6152.
**Listing-backed**: the call's pushes. **Pinned capture-free**:
`a_boat_born_on_the_shore_lights_its_disc_on_the_water`.

## 87. Two caravans pass through each other, and the second pair's East Indies word moves to 6321 (2026-09-29, item 1127)

Item 1127 was booked on East Indies' frame 6151 at Toughest: ours 9 draws
against the original's 8, parting at index 0, ours `Unit::move_step+0x823`,
the original `Unit::do_non_flat_gather+0x10f`. §86.2 had only the Caravan
`1/15`'s rows parting on block 6152, with ours standing it against the
Caravan `1/33`. **No mechanism was named.**

### 87.1 The frame, both sides

- **The draw delta** (`report.py … draws 6151`): the original's eight are
  `1/31`'s two gather draws, a bird's idle, three `Guy::inc_time` wraps
  and the two farm rolls. Ours spends `1/15`'s blocked stand first
  (seed `175a6b05`, the original's `1/31` draw), then the same eight.
- **The event**: on block 6152 the original's `1/15` holds the half-step
  bit (`unit_masks & 0x100000`), the only instance on run414 for either
  caravan, with `collide_o` −1. Both caravans' stacks are `MOVEORDER` over
  `TRADEORDER` on both sides.
- **The rule** is `detect_unit_collision`'s first soft row
  (`docs/COLLISION.md` §19). This crate had written it as a SEAM since the
  table was first read: "no capture has entered either".

### 87.2 What moved

- **East Indies 6151 → 6321.** On run414 block 6152, `1/15`'s `pos`,
  `collide_o`, `half_step` and figures closed (35 rows). The window's keys
  went 327 → 176, and none parts earlier.
- **The new word, 6321** (no mechanism is named): ours 9 draws and the
  original 8, parting at index 2. Ours spends `Guy::set_anim+0x97a <
  Unit::do_idle+0x7d` on the barge `1/42`. The original spends
  `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
- **run419 is its widening** (blocks 6316..6572,
  `run419_s_word_frame_is_widened_whole`). No dump covers 6227..6315, and
  its first block stands on 169 keys.
  - Among them is `1/42`, born on 6190 (run414) and carrying the citizen
    `1/32` (`inside_down` 32). It is already apart on 6316: `pos`
    (38028,11677) against (38016,11689). Its leg ends at (38028,11542)
    here and at (38016,11136) there.
  - On 6321 ours holds it with no order, and the original with one to
    (38040,11160).
  - On 6322 the original has landed `1/32` at (38040,11400) and closed the
    barge. Ours still carries both: `1/32`'s `inside` is 42 against −1,
    and `1/42` is this crate's alone.
  - Its `form` is −1 against 0, the newborn row that stands on 5976, 6111,
    6191 and 6218.

### 87.3 What this has *not* established

- Why the barge's leg ends short, or where in 6227..6315 it parts. The
  barge's landing point is the next item's frame, not its mechanism.
- The value rows standing before 6321 (§86.2's list, and 6182's
  `MAKE[4].val`). Each names no score yet.

### 87.4 Coverage

**Diff-backed**: the move and run414's block 6152. **Listing-backed**: the
row (`docs/COLLISION.md` §19.2). **Pinned capture-free**:
`two_caravans_on_their_legs_pass_through_each_other`.

## 88. A starting citizen is born at its camp, and Great Sahara's word moves to 12783 (2026-09-29, item 1133)

§83's first word was frame 8, ours 6 draws against 7. The original's extra
draw was the blocked stand of the AI citizen `1/2`
(`Guy::set_anim+0x97a < Unit::move_step+0x823`). The mechanism is the
collision index, and it is `docs/COLLISION.md` §20: `Setup::build_units`
creates the two citizens of the camp `1/2001` on the camp's own cell, and
`1/2`'s `come_out` clears the corner of `1/1`'s block that ours kept. On
frame 5 that corner was the only thing refusing the sidestep the original
took.

### 88.1 The move, and its value diff

- **run382: ticks 6, orders 5 → 1850, 1850**, and the word 8 → 1850: the
  whole capture walks.
- **run383's long word: 8 → 12783.**
- **The value diff, run382's block 6** (`run382_s_word_frame_is_widened_whole`):
  `1/2`'s `collide` ours 1 against 0, `order:move.dest` (38232, 16056)
  against (38328, 15768) and `path:length` 1 against 2 → the original's
  values on both. Block 9's `collide_frame` 6 against 8 and facing
  −328728576 against −1925840896 agree too. Over blocks 1..259 the parted
  keys fall 244 → 23: block 1's 22 standing rows, and the AI scout
  `1/0`'s `mylos`, ours 6 against 4, on block 202.

### 88.2 The new word, 12783, widened

**Frame 12783: ours 12 draws against 11, at index 9** — ours
`Leader::make_stuff+0x63d`, theirs `Guy::set_anim+0x97a <
Guy::inc_time+0x271`. run416 (`docs/RUNS.md`) is run383's game at run414's
detail over blocks 12778..13034, and `run416_s_word_frame_is_widened_whole`
walks it from run383's start, both directions, every record the capture
prints. None of its keys is left unprinted.

| block | who | field | ours | theirs |
|---|---|---|---|---|
| 12778 | both | 133 standing keys (§33.4's families, `tech_cat_frame`) | | |
| 12780 | `1/-1` | `MAKE[1].val`, `MAKE[10].val` | 9999999 | 1632000 |
| 12782 | `1/-1` | `MAKE[2].cat` / `num` / `t` | 10 / 1 / 573 | 6 / 3 / 132 |
| **12784** | `1/-1` | `bucket[0:food]` | 149 | 86 |
| **12784** | `1/-1` | `num_queued[82]` | 0 | 3 |
| **12784** | `1/39` | `g.cur_anim[0]` | 31 | 29 |

**No mechanism is named.** The AI's make list parts two blocks before the
word, and this window cannot say when it first parted: run382 stops at
1851. *(Item 1147: the 12780 row was who=1's population cap, uncompared
until then; §91.)*

### 88.3 What this has *not* established

- Whether the make list's parting on 12780 is the first thing to part
  between blocks 1851 and 12778. No dump covers those frames.
- Whether the first pair's maps have a camp hole. Their starts have two
  citizens at `2001` too, and every one of their floors held, so no pinned
  value moved there.

### 88.4 Coverage

**Diff-backed**: §88.1 by the third map's four tests; §88.2 by
`run416_s_word_frame_is_widened_whole`. The compared pin walks run416's
12782..12786, and the coverage driver drives the same blocks.

## 90. A barge keeps its goal ashore, and the second pair's East Indies word moves to 6609 (2026-09-29, item 1143)

Item 1143 was booked on East Indies' frame 6321 at Toughest: ours 9 draws
against the original's 8, parting at index 2, ours `Guy::set_anim+0x97a <
Unit::do_idle+0x7d`, the original `Guy::set_anim+0x97a <
Guy::inc_time+0x271`. §87.2 had the barge `1/42`, carrying the citizen
`1/32`, already apart on run419's first block 6316. **No mechanism was
named.**

### 90.1 The frame, both sides

- **The draw delta** (`report.py rontrace-run419.log draws 6321`): the
  original's eight are two `Unit::do_job` rolls, four `Guy::inc_time`
  wraps and two farm rolls. Ours spends the barge's idle `set_anim` at
  index 2 (seed `c0609b5d`, the original's first wrap), then the same.
- **The event**: on 6321 ours holds `1/42` with no order at (38028,11542),
  its passenger aboard; the original's barge still moves, and on 6322 it
  has put `1/32` ashore at (38040,11400) and closed.
- **Walked back to its first parted field** (`RON_DEBUG_UNIT` on the long
  walk, and run414's last blocks): the barge's path agrees field for field
  on run414's 6226 — its last entry (38016,11136) `flags 1` on both sides.
  On **6271**, in the gap 6227..6315 no dump covers, ours takes that final
  leg and rewrites it to **(38028,11542)**, `flags 1`; the original prints
  it on 6316 unchanged in place and **`flags 5`**.
- **The direction** was ours. `Unit::find_path`'s pull-back (`docs/ORDERS.md`
  §4.6) walked the land goal back toward the barge, 406 units north and
  onto its own column, to the first tile its `invalid_loc` took: a boat's
  refuses land unless the path's top carries `flags & 4`, and ours' did
  not.

### 90.2 The rule

The writer of that bit is `Unit::do_move`'s waypoint take, read off the
listing (`5f8544`–`5f86f4`; `docs/ORDERS.md` §4.4). A **final** waypoint,
or any waypoint under `TRADE_ROUTE`, whose tile's `get_tregion@006b52e0`
is not the unit's is popped and pushed back with `flags | 4` and tolerance
0. ORDERS §4.4 had read the row since its second reading, as a turn in
place, and listed it as not modelled; the bit is also `invalid_loc`'s
transport relax (`docs/TRANSPORT.md` §6.3), which is what lets a barge
walk onto the shore and `set_new_location` put its passenger out (§6.4).
Built in `Sim::do_move`.

### 90.3 What moved

- **East Indies 6321 → 6609.** On run419 the barge's `pos` (6316),
  `orders.len` (6321) and `extra` (6322) and the passenger's `inside` and
  `pos` (6322) all agree. The first block went 169 → 160 keys and the
  window's 992 → 317. What is left of the landing is `1/32`'s figure
  `avg_speed` on 6322, ours 0 against 11.
- **Residue that moved and was there before**: the barge identities
  `1/36` against `1/38` (a later ride's boat takes another free index)
  part on 6393 now, 6402 before the build.
- **The new word, 6609** (no mechanism is named): ours 9 draws, the
  original 7, at index 2. The original's are two `Unit::do_job` rolls,
  `Guy::set_anim < Unit::set_anim < Unit::move_step+0x823`, an
  `Animal::do_idle+0x83`, and three `Guy::inc_time` wraps. Ours spends
  no `move_step` stop, so the animal's idle roll takes the original's
  stop seed and goes on to `+0x1a4`, `+0x1d4` and `+0x212` before the
  same three wraps.
- **run420 is its widening** (blocks 6604..6860,
  `run420_s_word_frame_is_widened_whole`). Its first block stands on 263
  keys, 31 frames past run419's last, the gap 6573..6603.
  - Among them the AI unit `1/28`, whose `pos` agrees through run419's
    6572, walks apart: (29017,24361) against (28893,24233) on 6604.
  - On the word's block 6610 the original stands it against the animal
    `8/3` (`collide_o` 3, `collide_who` 8, `order:coll` (28743,24227),
    seven waypoints), and ours walks on with one. On 6611 the animal
    stands at (28661,24120) here and (28680,24120) there.
  - That is the block's reading, not a mechanism.

### 90.4 What this has *not* established

- The region check's `TRADE_ROUTE` arm on a non-final leg: built from the
  listing, and no capture has been read on it.
- The passenger's figure `avg_speed` after the landing (6322). ~~And where
  in 6573..6603 `1/28` parts.~~ On 6577, its region scan's cell: §93.

### 90.5 Coverage

**Diff-backed**: the move, run419's rows above. **Listing-backed**: the
branch (`5f8544`–`5f86f4`). **Pinned capture-free**:
`a_barge_taking_a_goal_ashore_flags_it_and_keeps_it`.

## 91. The AI holds Peacocks, and Great Sahara's word moves to 13182 (2026-09-29, item 1147)

§88's word was frame 12783, ours 12 draws against 11. Both sides spend
two `Leader::make_stuff+0x221` rolls, and then ours spends **eight**
`Leader::make_stuff+0x63d` (the slot loop's expiry, §2.6 step 6) against
the original's seven. The rest of the frame agrees (`report.py
rontrace-run416.log draws 12783`; ours from
`run383_s_long_trace_says_where_the_third_map_s_word_parts` with
`RON_DEBUG_SITES=1`).

### 91.1 The value, walked back to its field

Run416's first parting past the standing block is the make list on 12780:
slots 1 and 10, the Mercenaries offer (`t` 572 here, 573 in the dump's
numbering), priced **9999999 here against 1632000 there**. A probe in
`tech_value` gave ours: base 20000, weight 136, `val` 2720000. The
category-0 arm divides that by ten to 272000. Then `cap < 200` and
`cap × 5 / 6 < effective_pop` read **50 × 5 / 6 = 41 < 44**, which
multiplies by 20. Recency and shortages multiply by 3 and 2, giving
32,640,000, and `income 256 × val` wraps to the 9,999,999 clamp. The
original's 1632000 is exactly `272000 × 3 × 2`, with no `×20`.

The dump's who=1 record on 12779 and 12780 prints `effective_pop 44` (which
agrees, and is compared) and **`pop_cap 55`**. This crate held 50, and
`pop_cap` stood on the coverage pin as parsed but never compared
(`coverage.rs`), so nothing had seen it.

### 91.2 Where 55 comes from

`Leader::calc_pop_cap@006dc490`'s tail, read off the listing at
`006dc656`:

```
testb $0x8, 0x6da6(%esi)      ; rare.ptr[2] & 8
jne   6dc668
testb $0x8, 0x6dce(%esi)      ; rare_conquest.ptr[2] & 8
je    6dc695
cap = (PEACOCKS_POP + 100) × cap / 100   ; imul, then the /100 magic
```

`rare` is `LeaderData +0x6d98`, a `BitMask<44>` whose data begins at `+0xc`,
so the byte is `ptr[2]` and the bit is **19 = `PEACOCKS`(25) −
`BASE_RARE`(6)** (`enums/TypeIndex.txt`). Every path through the function
jumps to that tail, the scenario and ignore-cap ones included. 50 × 110 /
100 = 55. On 12780 the AI's merchant `1/18` prints `rare 25`: it holds
Peacocks.

`calc_pop_cap` is called from `Leader::calc_gather@006ceee0:353`, in the
rare mask's change arm after the assignment, and from `gain_tech`, the
city and building writers and the scenario functions. This crate had the
term in `cost::pop_cap`, but nothing ever set `PopBonuses::peacocks`, and its
change arm (`Sim::tick`, `Leader::gather`'s tail) never recomputed the cap.

### 91.3 The build

- `economy::PEACOCKS` (25).
- `Sim::recompute_pop_caps` reads the live mask for each player, as the
  original's test reads it on every call.
- The rare mask's change arm calls it.

`rares::tests::peacocks_raise_the_owner_s_pop_cap_by_a_tenth` and
`the_rare_mask_s_change_recomputes_the_pop_cap` hold the two halves.

`pop_cap` is now a compared leader row (`leader::rows`), and it has left
the coverage pin. On every window the suite walks it agrees on both
leaders: the first pair's leader windows (run19, 84, 91, 107, 115, 117),
the East Indies and Great Lakes widenings from run139 to run299, and both
of Great Sahara's.

### 91.4 What moved

- **run383's long word: 12783 → 13182.** On frame 12783, 12 against 11 →
  11 against 11.
- **The value diff, run416's block 12780**
  (`run416_s_word_frame_is_widened_whole`): who=1's `pop_cap` 50 against
  55, and `MAKE[1].val` and `MAKE[10].val` 9999999 against 1632000, → the
  original's values on both. Over run416 the parted keys fall 951 → 258.
  The word's block 12784 falls from 10 keys to 4, and `1/39`'s figure
  agrees. What is left there is the food and metal buckets (70 and 199
  here against 86 and 215) and building `1/2014`'s queued Militia (66),
  priced 80 food and 80 metal here against 64 each there. That 16 is
  both buckets' gap.
- Every floor, every pair's word, every closed chapter and run382's
  1850/1850 hold.

### 91.5 The new word, 13182, widened

**Frame 13182: ours 9 draws against 8, at index 3.** Ours spends
`Leader::make_stuff+0x63d`; theirs spends `GameAccess::rnd+0x20 <
Unit::do_job+0x67` (seed `f7f1d4fc`). The original's frame is one
`use_market`, two `make_stuff+0x221`, two `Unit::do_job` rolls, two
`Guy::set_anim` and one `Farms::inc_time`. run417 (`docs/RUNS.md`) is
run383's game at run414's detail over blocks 13177..13433, and
`run417_s_word_frame_is_widened_whole` walks it from run383's start, both
directions, every record the capture prints. None of its keys is left
unprinted.

| block | who | field | ours | theirs |
|---|---|---|---|---|
| 13177 | both | 238 standing keys: run416's families, plus what parted between the captures | | |
| 13177 | `1/-1` | `bucket[0:food]` / `bucket[4:metal]` | 52 / 126 | 68 / 142 |
| 13177 | 25 AI citizens | `myhits` / `mylos` (first on run416's 12945) | 40 / 2 | 50 / 4 |
| 13181 | `1/-1` | `MAKE[1].t` / `val` | 61 / 211538 | 177 / 9999999 |
| **13183** | `1/2017` | `queue[3].type` | 177 | 82 |
| **13183** | `0/3` | `order:move.off_x` | 312 | 504 |
| **13183** | `1/-1` | `bucket[2:wealth]` | 72 | 137 |

**No mechanism is named.** Two partings stand between the captures, and
either may be the word's. Run416 shows the first on 12784 (the Militia's
price, 80 against 64) and the second on 12945 (the citizens' hits and line
of sight, 40 and 2 against 50 and 4). The make list parts two blocks
before the word.

### 91.6 What this has *not* established

- The other readers of the rare mask. `calc_pop_cap` is the one this
  document now models; it was found because it moved a number.
- ~~Whether the Militia's price on 12784 and the citizens' hits on 12945 are
  one cause or two, and which of them the word at 13182 descends from.~~
  Two: the word descends from the Militia's price, which is Wine's research
  discount (§94). The citizens' hits still stand on run417's first block.

### 91.7 Coverage

**Diff-backed**: §91.1 and §91.4 by run416's widening, which pins
`pop_cap` and the Mercenaries slot as agreeing over blocks 12778..13034.
§91.2's cap is diff-backed on every window the suite walks. §91.5 is
`run417_s_word_frame_is_widened_whole`. The compared pin walks run417's
13181..13185, and the coverage driver drives the same blocks and run416's
12782..12786.
**Read only**: §91.2's listing, bit and offset. The dump's 55 against a
Military table's 50 is the only value that proves them, and it is on one
map.

## 93. A sibling is counted where it is going, and the second pair's East Indies word moves to 6743 (2026-09-29, item 1156)

Item 1156 was booked on East Indies' frame 6609 at Toughest: ours 9 draws
against the original's 7, parting at index 2, ours `Animal::do_idle+0x83`,
the original `Guy::set_anim+0x97a < Unit::move_step+0x823`. §90.3 had the
AI citizen `1/28` apart on run420's first block 6604 and first parting in
the gap 6573..6603 no dump covers. **No mechanism was named.**

### 93.1 The frame, both sides

- **The draw delta** (`report.py rontrace-run420.log draws 6609`): the
  original's index 2 is `1/28`'s stop, `Unit::set_anim < Unit::move_step+0x823`
  (seed `16947416`), where it stands against the animal `8/3`; ours spends
  that seed on the animal's idle roll and walks on.
- **Walked back to its first parted field** (`RON_DEBUG_UNIT` on the long
  walk, and **run421**, booked for the gap 6573..6603): `1/28` arrives at
  (29568,24192) on 6576 on both sides. On 6577 the original holds
  `orders_x/y` **(26616,22776)**, its leg to (26592,22752) — cell (34,29) —
  and ours held (28920,25080), cell (37,32). On 6588 `do_explore_to`'s
  fifteen-frame look re-aims both onto the goody box at (28056,24216)
  (`docs/GOODY.md` §7), from different places, so the re-aim did not
  part: the leg taken on 6576 did.
- **The event** is `Unit::think_scout`'s region scan (`docs/SCOUT.md` §11),
  reached through `think_peasant+0x2ac`: both sides draw the same 26
  `+0xaba` jitters on 6576, so the same cells pass the fog, location and
  surface tests. Ours' scores, printed in the scan: (37,32) at 38 wins;
  the three row-31 cells beside the box (76, 110, 136) are refused by
  `find_unit_ordered`, which found `1/22`.

### 93.2 The rule

`ObjectsData::find_unit_ordered@0065bc40` measures a candidate sibling at
**`UnitData +0x70/+0x74`**, `orders_x`/`orders_y` (`0065be35`–`0065be51`,
`types.txt`'s names), where the decompiler prints `vector_dist(unaff_EDI,
unaff_ESI)`; the XOR-ed position is read only for the `0x200` region test.
`Sim::scout_unit_near` measured the body. `1/22` stood at (27221,25195),
more than `0x600` from (37,32)'s centre, and was headed for (27384,25080),
inside it. Measured there it refuses (37,32), and the scan takes (34,29)
at 128. `docs/SCOUT.md` §8.1, amended in place; the merchant's search had
the same listing right.

### 93.3 What moved

- **The value diff, run421** (`run421_s_gap_is_walked_whole`): `1/28`'s
  `orders_x/y` (28920,25080) against (26616,22776) on 6577 → agreeing.
  What else parts on the gap's 44 blocks (282 keys) is the first block's
  263 standing keys, one-member groups' `speed`/`form`, the barge
  identities `1/36`/`1/38`, and `1/42`'s `path[16].flags`, ours 4 against
  0 on 6576.
- **East Indies 6609 → 6743.** On run420 `1/28`'s `pos`, its move's
  `angle`, group 65's `o_angle`, its `collide_o` (6610) and `path:length`
  all agree; the first block went 263 → 249 keys, the window's 1444 → 963.
  The animal `8/3` now stands apart first on 6791.
- **The new word, 6743** (no mechanism is named): ours 4 draws, the
  original 8, at index 2, ours `Guy::set_anim+0x104b`, the original
  `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Inside run420's window
  (block 6744). The citizen `1/33`, aboard the barge that is `1/36` here and
  `1/38` there (`inside` apart since 6604; parked since item 1143), is put
  ashore: its `orders_x/y` (37439,33407) against (38952,24840) on 6735, and
  its `pos` (38944,24864) against (38952,24840) on 6743. That is the
  block's reading, not a mechanism.

### 93.4 What this has *not* established

- `find_unit_ordered`'s `FILTER_BASE_TYPE` against this crate's `ty`: the
  same for two citizens, not read for a lineage that differs.

### 93.5 Coverage

**Diff-backed**: `1/28`'s rows above, run421 and run420. **Listing-backed**: the
distance's operands (`0065be2c`–`0065be51`). **Pinned capture-free**:
`a_sibling_blocks_a_cell_by_where_it_is_going_not_where_it_stands`.

## 94. The research arm's Wine discount, and Great Sahara's word moves to 14587 (2026-09-29, item 1163)

§91's word was frame 13182, ours 9 draws against 8. Both sides spend one
`use_market` and two `Leader::make_stuff+0x221` rolls. Then ours spends one
`Leader::make_stuff+0x63d`, the slot loop's expiry (§2.6 step 6), where the
original goes straight to its two `GameAccess::rnd+0x20 < Unit::do_job+0x67`
rolls (seeds `f7f1d4fc`, `fb2f2c2b`). The rest of the frame agrees
(`report.py rontrace-run417.log draws 13182`; ours from
`run383_s_long_trace_says_where_the_third_map_s_word_parts` with
`RON_DEBUG_SITES=1`). §91 named no mechanism. It named two partings between
the captures as candidates, and the first of them is the word's.

### 94.1 The make list, both sides

`RON_DEBUG_LEADER=13176-13184` on run417's widening printed ours, and the
dump's who=1 `LEADERDATA` printed theirs. Both lists are empty on 13177 and
13178, and they agree on the Upgrades step's list on 13179 and 13180. The
Units step (block 13181) parts:

| slot | ours | theirs |
|---|---|---|
| 0 | Longbowmen (177), 9999999 | **Slingers (82)**, 9999999 |
| 1 | Merchant (61), 211538 | Longbowmen (177), 9999999 |
| 2 | —, 211538 | Merchant (61), 211538 |
| 6 | Longbowmen, 9999999 | Slingers, 9999999 |

The original offers the Slingers and ours does not. On 13183 the barracks
`1/2017` queues the Slingers there and the Longbowmen here. The extra
expiry roll on 13182 comes from the head's repeat, which is a different
unit on each side.

### 94.2 Why ours drops the Slingers

A probe in `create_units` for who=1, the Slingers, city 1, frame 13180,
gave `v` 4800000, `remaining` 3, and `val = v × 3 × 10 = 144,000,000`.
`want` and the divisor are both 33. `check_income`'s factor was **64**,
because `type_affordable` answered 0: the Slingers cost 55 food and 55
timber, and this crate held **53 food against the original's 69**. The
tail `fac × (want × val / divisor) / 256` wraps 33 × 144,000,000 to
457,032,704. That divides to 13,849,475, and at a factor of 64 it gives
3,462,368, **positive**, so no clamp. At a factor of 256 the same product
wraps negative and clamps to 9,999,999. That is the original's value, and
ours on 12780 and 12980, when the Slingers were affordable. The Longbowmen
were unaffordable on both sides and clamp on both, because `remaining` is 1
and the product is smaller. `offer_value`'s docstring explains why the
wrap is kept.

So the word is the food bucket, 16 short. It has stood short since
run416's block 12784.

### 94.3 The 16: the Militia research, priced without Wine

On run416's 12784 building `1/2014` (`orig_type` 439) queues type 66, the
Militia, which it does not yet own, so this is a research. The price is
**80 food and 80 metal here against 64 and 64 there**. Ours comes from
`research_modifiers`: base 8 and 8, the unit factor 10,
`RESEARCH_PREMIUM_COST` 256, no refit, and no upgrade discount (Military
level 1 against the type's 1). 64 is 80 × 4 / 5.

`TypeData::get_cost@00664090`'s research arm (`get_cost:425`–`432`), read
off the listing:

```
00664eb5  testb %dl, 0x6c18(%esi,%eax)   ; the availability bit: set → the train arm
00664ed0  testb $0x80, 0x2b4(%eax)       ; the type's `h` flag: set → the train arm
00664edd  testb $0x4, 0x6da4(%ecx)       ; rare.ptr[0] & 4
00664ee6  testb $0x4, 0x6dcc(%ecx)       ; else rare_conquest.ptr[0] & 4
00664eef  ecx = 100 − constants.wine_unit_upgrades    ; +0x8d8
00664f04  ecx *= cost ; the /100 magic 0x51eb851f, sar 5, toward zero
00664f13  cost = cost × RESEARCH_PREMIUM >> 8, × RESEARCH_PREMIUM_COST >> 8
```

`rare` is `LeaderData +0x6d98` with its data at `+0xc` (§91.2), so the bit
is **2 = `WINE`(8) − `BASE_RARE`(6)** (`enums/TypeIndex.txt`).
`WINE_UNIT_UPGRADES` is `20%` in `rules.xml`. The only other function that
reads `+0x8d8` is `get_desc_parsed`, the description text, and the
building arm's `RESEARCH_PREMIUM` (`get_cost:705`) has no Wine test.
run416's who=1 record prints `rares_collected` at 2, 4 and 19: Wine, Dye
and Peacocks. The AI holds Wine. `cost::Research`'s docstring had listed
Wine as a term not carried.

### 94.4 The build

- `economy::WINE` (8), and `Tuning::wine_unit_upgrades` (20,
  `WINE_UNIT_UPGRADES`, checked against the install with the other slots).
- `cost::Research::wine`: the percentage, which `cost_of`'s research arm
  applies before the premium.
- `Sim::research_modifiers` sets it from `has_rare(who, WINE)`, which is
  `rare` with `rare_conquest` folded in.

`harness_tests::wine_takes_a_fifth_off_a_research_and_nothing_off_a_train`
holds it on the Militia's own numbers: 80 without Wine, 64 with it, the
same 80 on the train arm, and another player's Wine gives nothing.

### 94.5 What moved

- **run383's long word: 13182 → 14587.** On frame 13182, 9 against 8 → 8
  against 8.
- **The value diff, run416's block 12784**
  (`run416_s_word_frame_is_widened_whole`): `1/2014`'s `queue[0].cost[0]`
  and `cost[1]`, 80 and 80 against 64 and 64, → 64 and 64. The food and
  metal buckets, 70 and 199 against 86 and 215, → the original's. Block
  12784 no longer parts, and run416's parted keys fall 258 → 254.
- **run417** (`run417_s_word_frame_is_widened_whole`): the parted keys fall
  935 → 277 on this item's own tree. On the tree merged with item 1111,
  which built a citizen's hits and line of sight, they fall 831 → 173
  (run416's 254 → 250). The food bucket agrees over the window. The make
  list agrees on 13181, with the Slingers at the head; the only thing that
  parts on that block is the city shift (§53.2). The old word's block 13183
  agrees. What parts next is who=1's `known_rares` on 13184 (7 here against
  6), then an AI group of three on 13222 that the original stands up and
  this crate does not.
- Every other walk in the suite holds without a re-pin: every floor, every
  pair's word, every closed chapter, and run382's 1850/1850.

**Mutation W** turns the Wine term off in `research_modifiers`. It was
committed first, run on the merged tree with the constant at 14587, then
restored from git and touched. The long trace falls back to **13182, 9
against 8, at `make_stuff+0x63d`**. `run416_s_…`, `run417_s_…`,
`run418_s_…` and the unit test fail.

### 94.6 The new word, 14587, widened

**Frame 14587: ours 9 draws against 10, at index 3.** Both sides spend
three `Guy::set_anim+0x97a < Unit::move_step+0x823`. The original then
spends a fourth, and ours goes on to `Guy::set_anim+0x97a <
Guy::inc_time+0x271`. run418 (`docs/RUNS.md`) is run383's game at run414's
detail over blocks 14582..14838. `run418_s_word_frame_is_widened_whole`
walks it from run383's start, both directions, and no key is left
unprinted. The counts are on the merged tree.

| block | who | field | ours | theirs |
|---|---|---|---|---|
| 14582 | both | 155 standing keys: run417's families and the AI's army group | | |
| 14582 | 12 AI soldiers (`1/28` … `1/57`) | `order:group.id` | 14330130 | 14336430 |
| 14582 | `1/-3` | `group:64.role` | 0 | 1379331 |
| 14587 | `1/-3` | group 64's slots | 21 | 15 |
| 14587 | `1/52` | `pos` | (28637, 20233) | (28623, 20241) |
| **14588** | `1/52` | `collide_o` / `collide_who` | −1 / −1 | 30 / 1 |

The original's `1/52` collides with `1/30` on the word's frame and stops
through `move_step`; ours walks on. The army group's id is stamped with
its frame, and the stamps are six frames apart (14330 against 14336). The
compared pin now walks run418's 14586..14590, where this group's order row
is compared (`coverage.rs`). ~~**No mechanism is named.**~~ — **§97**:
`push_group`'s walk shifted army group 64's list and not its slot
arrays, so `1/52` walked for `1/47`'s slot on 14586.

### 94.7 What this has *not* established

- `rare_conquest`'s arm of the test is not held by any capture apart.
  `has_rare` folds it into `rare`, as for Horses and Tobacco.
- The order of Wine against `RESEARCH_PREMIUM` is read off the listing, and
  no run separates it: the Militia's `RESEARCH_PREMIUM_COST` is 1, so on
  this purchase the two orders give the same number.
- `known_rares` on run417's 13184, 7 here against 6. It is the first
  non-standing parting after the make list agrees, and nothing here reads
  it.
- Why the army group forms six frames apart, and whether that is the new
  word's cause.

### 94.8 Coverage

**Diff-backed**: §94.3's price and §94.5's value diff by
`run416_s_word_frame_is_widened_whole`, and §94.1's list by
`run417_s_word_frame_is_widened_whole`. Both fail under mutation W, and
so does the long trace's floor. §94.6 is `run418_s_…`. **Read only**:
§94.2's arithmetic. The probe printed its terms, and the wrap is
`offer_value`'s, which Great Lakes' floor already holds. §94.3's listing,
bit and offset are also read only, and one purchase on one map is the
value that proves them. **Pinned capture-free**:
`wine_takes_a_fifth_off_a_research_and_nothing_off_a_train`.

## 95. A passenger comes ashore at the speed it boarded with, and the second pair's East Indies word moves to 7382 (2026-09-29, item 1164)

Item 1164 was booked on East Indies' frame 6743 at Toughest: ours 4 draws
against the original's 8, parting at index 2, ours `Guy::set_anim+0x104b`,
the original `Guy::set_anim+0x97a < Guy::inc_time+0x271`. §93.3 had the
merchant `1/33` put ashore from the barge that is `1/36` here and `1/38`
there, its `orders_x/y` apart on 6735 and its `pos` on 6743. **No mechanism
was named.**

### 95.1 The frame, both sides

- **The draw delta** (`report.py rontrace-run420.log draws 6743`; ours
  `RON_DEBUG_SITES`): the original draws four `+0x97a < Guy::inc_time+0x271
  < Unit::inc_time+0x6e`, three `+0x104b < Unit::inc_time+0x3e` and
  `Farms::inc_time+0x1ae` (seeds `f534a7e2` … `258f6a57`). Ours draws two
  `+0x97a` for `1/15`, one `+0x104b` for the bird `9/7`, and the farm's.
  On 6739 and 6741 ours' `1/33` spends two `+0x97a` for its crew, as the
  original does.
- **Both sides' event** (`RON_DEBUG_UNIT=1/33@6725-6745` against run420's
  blocks): the barge lands `1/33` on 6734 on both sides, at (38952,24840),
  with the same eleven-step path and the same move to (38328,23352). Its
  guy 0 then turns in place, anim 7, from 165478400 to the path's
  -1925840896.
  - The original's steps are -71582784, -95443712, -143165568,
    -143165568, then -286331136 a frame. It faces the path on 6745 and
    steps to (38944,24864).
  - Ours stepped -286331136 from the first frame. It faced the path on
    6743 and walked on, so on 6743 its crew took no idle roll.
- **The first parted field**, walked back: 6735's `g.avg_speed[0..2]`,
  ours 0 against the original's 12. The original's decays 9, 6, 4, 3, 2,
  1, 0 over the turn, and `docs/MOVEMENT.md`'s `turn_speed` divides by
  `avg_speed / 4 + 1`: 4, 3, 2, 2, 1. That is the ramp, step for step. On
  the same block `orders_x/y` parts, (37439,33407) against the landing
  point.

### 95.2 The rule

- **The speeds**: `GuyData +0x80`/`+0x84` (`last_speed`, `avg_speed`) have
  three writers by offset: `Guy::move`, `Guy::clear` and `Guy::init_real`.
  `Guy::process` and `Guy::set_anim` read `+0x84`.
  - Neither `Unit::set_new_location@005f8d20` nor
    `Guy::set_new_location@005d86f0` writes either field.
  - So a passenger lands at the average the ride froze.
  - Ours' `disembark` built the passenger's movement from `Movement::at`,
    and `seat_guys` built each crew body from `Body::at`. Both zeroed it.
- **The landing point**: `come_out`'s tail at `6187c8`..`618836`, read in
  the listing. It skips a human's unit (`leaders.list[who] & 4`), a plane
  and the `0x32..0x35` citizen types. It gives every other unit step 1's
  `update_action` again, from the spot.
- Both are in `docs/TRANSPORT.md` §6.4, amended in place.

### 95.3 What moved

- **The value diff, run420** (`run420_s_word_frame_is_widened_whole`):
  - 6735: `1/33`'s `g.avg_speed[0..2]` 0 against 12 → agreeing, and its
    `orders_x/y` (37439,33407) against (38952,24840) → agreeing.
  - 6736: its guys' `angle` -120852736 against 93895616 → agreeing.
  - 6743: its `pos` (38944,24864) against (38952,24840) → agreeing.
  - 6791: the animal `8/3`'s `pos` → agreeing.
  - The window went 963 → 303 keys.
- **Earlier landings agree too**:
  - run414's passengers `1/23` and `1/22`, their figure `avg_speed`
    (6026, 6069): 176 → 174 keys.
  - run419's `1/32`, 0 against 11 on 6322: 317 → 304 keys.
  - run420's `1/0` on 6658: its figure `avg_speed` and `orders_x/y`.
- **East Indies 6743 → 7382.** Frame 6743's draws, 4 against 8 → agreeing.
- **The new word, 7382** (no mechanism is named): ours 15 draws, the
  original 9, at index 0. Ours spends `Leader::use_market+0x1ed`, the
  original `Leader::make_stuff+0x221`.
  - It is past run420's last block (6860), so it is widened on **run425**
    (§95.5).
  - From 6781 run420 already has player 1's make list apart (`MAKE[].city`
    on 6781, 6782 and 6784), then `gather_stamp` on 6792. That is where it
    stands, not a mechanism.

### 95.4 What this has *not* established

- `come_out`'s **building** arm (`Sim::come_out`, `garrison.rs`) still
  builds the unit's movement from `Movement::at`, which zeroes the body's
  speeds except for a plane's. It reads through the same
  `set_new_location`, so the original keeps them there too. Not built: no
  row on the word's frames names it.
- The plane clause of the tail (`6187e4`..`6187f7`) is not built. A barge
  carries no plane.

### 95.5 The new word, 7382, widened

**run425** (`docs/RUNS.md`) is run346's game at run420's detail over
blocks 7377..7633; `run425_s_word_frame_is_widened_whole` walks it.

- Block 7377 stands on 238 keys, from the gap 6861..7376.
- Player 1's make list parts first:
  - `MAKE[].val` on 7379 (slot 0: ours 1800000 against 4194000, about
    three sevenths in every slot);
  - `MAKE[7].val` on 7380;
  - `MAKE[].city` on 7381.
- On the word's block 7383: its `bucket`s, `num_queued`, and the citizen
  `1/12`'s order.
- On the word's frame ours draws two `Leader::use_market+0x1ed` and three
  `Leader::produce_building+0x1805` that the original does not, and then
  the same `make_stuff` draws.
- run420 had the make list's `city` apart from 6781. That is the block's
  reading, not a mechanism.

### 95.6 Coverage

- **Diff-backed**:
  - `1/33`'s rows, run420;
  - the earlier passengers' figure speeds, run414 and run419;
  - the word 7382's window, run425.
- **Listing-backed**: the tail's branch (`6187c8`..`618836`).
- **Export-backed, by offset**: the speed fields' writers.
- **Pinned capture-free**:
  - `a_passenger_comes_ashore_at_the_average_it_boarded_with`;
  - `a_computer_player_s_passenger_is_ordered_where_it_lands_and_a_human_s_where_it_boarded`.

## 97. A kill shifts the group's slots with its list, and Great Sahara's word moves to 15586 (2026-09-29, item 1171)

§94.6's word was frame 14587, ours 9 draws against 10, at index 3. Both
sides spend three `Guy::set_anim+0x97a < Unit::set_anim+0x56 <
Unit::move_step+0x823`. The original then spends a fourth (seed `7ce0cc85`),
and both go on to six `+0x97a < Guy::inc_time+0x271`
(`report.py rontrace-run418.log draws 14587`; ours `RON_DEBUG_SITES`). §94.6
named no mechanism. Its word's-block row, `1/52` colliding with `1/30` in
the original only, was the event: the fourth `move_step` draw is `1/52`'s
stop.

### 97.1 The frame the state first parted, walked back

`1/52`'s first parted field on run418 is its position on block 14587, so
frame 14586 is where it parts. The rows below are run418's dump and
`RON_DEBUG_UNIT=1/51@…,1/52@…` on the widening.

| block | `1/52` | ours | theirs |
|---|---|---|---|
| 14586 | `pos`, order | (28637, 20233), `GROUP_ATTACK_TO` in group 64 | the same |
| 14587 | `pos` | (28637, 20233), `last_speed` 0 | (28623, 20241), `last_speed` 16 |
| 14587 | order | `ATTACK_TO` (28248, 20424), no group | the same |
| 14587 | waypoint (`dest_x/y`, `dest 0`) | (28373, 20104) | (28614, 20246) |

Both waypoints are `1/52`'s formation slot as its follower arm computes it
(`docs/ORDERS.md` §8.3, `do_group_move@005e79a0`): the leader `1/51`'s position on that
frame, (28546, 20120), plus the slot's rotated offset. The original's
offset is (68, 126). Ours was (−173, −16), which is `1/47`'s. It walked
toward a slot 295 away, cut across the formation and did not step. Later
in the same frame `1/53`'s follower arm ended the formation and
ungrouped the squad `1/51`–`1/53`, on both sides.

### 97.2 Why the offset was another member's

On frame 14586 the AI pushes `1/28`–`1/30` and `1/65`–`1/67` out of army
group 64 into a new group 68 (`GROUP_MOVE`, stance 5), on both sides. The
original's group 64 prints 15 slots on block 14587, `1/52` fifth at `off`
(−3, 0), `curr` (68, 126). A probe in the follower arm printed ours: the
list is 15, and `off`/`curr` are still 21 long, in the old order.
`push_group`'s walk (`docs/GROUPS.md` §30.3) took the leaving squads out of
the list with a retain and left the arrays. `Group::kill@00714110` shifts
`list`, `angles` and the four offset arrays together (§4.2 there already
said "all five"). So `1/52`, listed fifth, read slot 4 of the old table.

### 97.3 The build

- `Sim::kill_from_named` calls `Sim::seat_kill`, which is `Group::kill` on
  a seat, once per member, with the arrays shifted.
- `unseat_group`'s broader removal goes through `Sim::seat_drop`, which
  shifts the arrays too.
- A unit test, `a_member_leaving_an_army_group_takes_its_slot_with_it`.

### 97.4 What moved

- **run383's long word: 14587 → 15586**, on the tree merged with item 1164.
- **The value diff, run418's block 14587**
  (`run418_s_word_frame_is_widened_whole`): group 64's `off[4]` (2, 3) →
  (−3, 0) and `curr[4]` (−173, −16) → (68, 126). Slots 15–20, which the
  original does not print, are gone. `1/52`'s `pos` (28637, 20233) →
  (28623, 20241). The old word's block 14588 agrees, `1/52`'s
  `collide_o` 30 on both sides. run418's parted keys fall 833 → 168.
  14587 still parts on group 68's `speed`/`new_speed`, 26 here against 0,
  which parted there before this item too.
- Every other walk in the suite holds without a re-pin.

**Mutation M** turns both removals back into a list-only retain. It was
committed first, run with the constant at 14587, then restored from git
and touched. The long trace falls back to **14587, 9 against 10**, at
`Guy::inc_time+0x271`. run418's widening moves 7 pins, and the unit test
fails.

### 97.5 The new word, 15586, widened

**Frame 15586: ours 7 draws against 6, at index 3.** Both sides spend
`Animal::do_idle`'s `+0x97a` and two `Unit::do_job+0x67` rolls. Ours then
spends a `Guy::set_anim+0xf2f < Guy::move+0x166` the original does not,
and both end on three `+0x97a < Guy::inc_time+0x271`. run426
(`docs/RUNS.md`) is run383's game at run414's detail over blocks
15581..15837. `run426_s_word_frame_is_widened_whole` walks it, both
directions, and no key is left unprinted. 741 keys part over the window.

| block | who | field | ours | theirs |
|---|---|---|---|---|
| 15581 | both | 159 standing keys: run418's families | | |
| 15581 | the army group's members | `order:group.id` | 15354136 | 15360436 |
| 15582 | `1/-1` | `MAKE[1].val`, `MAKE[4].val` | 59500 | 51000 |
| 15585 | `1/29` | head order, `orders.len` | `ATTACK`, 2 | `MOVE_TO`, 3 |
| 15585 | `1/29` | `pos` | (7292, 27822) | (7269, 27840) |
| 15586 | `1/29` | `attack[0].in_range` / `recharging` | 1 / 30 | 0 / 0 |
| **15587** | `1/29` | `g.cur_anim[0]` | 13 | 9 |

`1/29` is one of group 68's raiders, walking for the human's city. Ours
has dropped its move and stands attacking `0/2004`; the original's is
still walking, with a third order under it. The army group's id is again
stamped six frames apart. The compared pin and the coverage driver now
walk run426's 15585..15589. **No mechanism is named.**

### 97.6 What this has *not* established

- `Army::normalize`'s own cull (`Sim::army_normalize`) still rewrites an
  army's list without shifting its arrays, where `Group::normalize`
  (`Sim::seat_normalize`) shifts them. No walk on disk parts on it, and
  this item did not build it.
- Group 68's `speed`/`new_speed` on block 14587, 26 here against 0. It
  parted before this item as well, and no draw reads it over run418.
- Why the army group forms six frames apart (§94.7's open question,
  again on run426), and whether it is the new word's cause.

### 97.7 Coverage

**Diff-backed**: §97.1's and §97.4's rows by
`run418_s_word_frame_is_widened_whole`, which moves 7 pins under mutation
M, as does the long trace's floor. §97.5 is `run426_s_…`. **Read only**:
§97.2's array layout, read off `Group::kill@00714110`'s decompile and the
type record's `GroupData` (`off_x +0x4c` … `list +0x8cc`). **Pinned
capture-free**: `a_member_leaving_an_army_group_takes_its_slot_with_it`.
