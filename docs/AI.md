# The AI

**Status: brief, not read — 2026-08-24.** This is the survey half of the
mechanic, written by the main thread from the full decompile export
(`~/ghidra-projects/decomp/`, `tools/ghidra/`) before any function was read
end to end. It scopes the work, names every function with its decompiled
line count so the cost is visible in advance, states what the shipped dumps
already show the AI doing, and records the one finding that changes the
project's framing. Whoever runs the first reading replaces this document;
until then every claim below is a survey claim — from grep, a function's
head and tail, and the PDB layouts — not a derived one.

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

## 2. The frame hook and the cadence

`Game::do_frame@00591ef0`, in order (line numbers of the decompile):

```
199  Leaders::process_all        income, stats, elimination, taunts (Leader::process — no AI)
267  Leaders::strategy_all       gated: (semaphore[1] & 8) == 0          ← production AI
270  GameDaemon::process_all
272  Armies::process_all         gated the same way                       ← combat AI
275  Objects::process_all        units step their orders (ORDERS.md §2.1)  ← unit AI inside
279  Leaders::end_process_all
288  Leader::process_event_frame  per active leader
```

`Leaders::strategy_all@006ed430`: for every leader with `leader_flags & 3 ==
3`, **every frame**: `check_explore`, `plan_strategy`, `compute_score(0)`,
`diplomacy`; then `Game::check_victory` if `semaphore[1] & 2`.

`Leader::plan_strategy@006b9620`, head (read):

```
phase = (who * 25 + frame) % (200 / ai_speed)          // ai_speed = 1 (Game::init_data), cheat-raised
if production_step != 0:  production_ai();  return     // the step machine runs first, every frame
if frame != 0 && phase != 0:
    if phase % 30 != 0: return                          // nothing
    // the cheap tick: if the next age type is known, affordable, not had, not researching: make_this(0)
    return
... the full sweep (1,600 lines): per-city stats, army founding via Armies::init_army, sites ...
tail: check_orphaned_buildings(); compute_sites(0); production_step = 1     // unless semaphore[1] & 2
```

So each AI leader takes its **full sweep at frame 0 and then every 200
frames on its own phase** (`who = 1` → frames 175, 375, …), and the sweep's
last act is to arm the step machine. `Leader::production_ai@006c1960` (read,
114 lines) then runs **one step per frame** on `production_step`
(`LeaderData+0x788`):

| step | what | notes |
|---|---|---|
| — | bail to 0 if `leader_flags & 4 && !(& 8)`, `ai_off`, or `leader_flags2 & 4` | the human gate, the console's `ai off`, the switch |
| 1 | **run the script** `prod_script` with `(who+1, ref script_step, pers.rush+2, 5)` | skipped straight to 2 if `prod_script_run == 0` or `starting_resources == 8`. Return 1 (`BLOCK_ON_THIS`) → `production_step = 0`, done for this sweep; 3 (`SCRIPT_DONE`) → `prod_script_run = 0` and advance; anything else → advance |
| 2 | `production_ai_setup` (316, read); `make_list.clear()` | the goods picture: `econ[6]` flag words, `worst_good`/`best_good`, `shortages`; then `market_speculation` |
| 3 | `found_cities` (281) | the city layer |
| 4 | `research_techs` (689) | |
| 5 | `upgrade_units` (344) | |
| 6, 9 | `create_units` (1,674) | |
| 7, 10 | `create_buildings` (1,728) | |
| 8 | `make_stuff` (305) → `use_market` (147) | returns non-zero → stay armed; else `production_step = 0` (unless `starting_resources == 8`) |
| 11 | `make_stuff`, then 0 | |

`effective_pop = queued_units() + control + 1` is recomputed at every step.
`MakeList` (`+0x6ec8`, 0x1c bytes) is the sweep's shopping list —
`make_this`, `produce_*` (`produce_building` 1,173, `produce_unit` 437,
`produce_tech` 273, `produce_city` 178, `produce_upgrade` 152,
`produce_spell` 93) and `check_income` (the affordability oracle, called
from `research_techs`, `create_units`, `create_buildings`) are the
consumers.

`Leader::check_explore@006bc860` (47, read): at frame 0 and on `(who*25 +
frame + 12) % (200/ai_speed) == 0`, counts the explored cells of the world's
per-player bit (`world+0x160`, byte 3 of each cell, bit `who`) into `+0x9d4`,
or takes the whole map with `EXPLORE_MAP_BONUS`.

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
