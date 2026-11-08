# Structure

The target shape of the Rust tree, and the moves that reach it. Written at
the seventy-fourth pass (2026-11-08, `docs/DECISIONS.md` entry 114 §4) from
three Opus 5.5 inventories of the tree as it stood at `db6c12985`
(`inventory-structure.md`, `inventory-constants.md`, `inventory-tests.md`
in the pass's job directory; their counts are quoted here with the file
they came from). This is the `structure` track's specification
(`docs/TRACKS.md`): the queue books its items from §5 in order, and the
track's number is §2's counts falling to their targets.

## 1. The rule

**No behaviour change.** A structure landing moves, splits, renames and
re-exports; it never alters what the simulation computes. Its gate is the
whole release gate with **every floor unchanged** — every scoreboard word
at its length, the melees' closing states, the mapgen phases, the census's
`SIMULATION_BACKED` and port counts, the soak digest, `rondata::diff`'s
widenings — and a landing that moves any of them by one is wrong by
definition, whatever the move was. This repository can certify that frame
for frame on about 190 lockstep games, which is what makes a refactor of
this size cheap to verify; it is also why the sim side runs structure
items alone (`docs/DECISIONS.md` 114 §2): every move conflicts with every
concurrent sim change.

Three sub-rules, each learned from the inventories:

- **The random stream is sacred.** `rng` is advanced from 34 production
  files (`inventory-structure.md` §1c); reordering two draws moves every
  floor. A move never changes the order in which code runs. The `ORTHOG`
  neighbour tables that look like duplicates walk in different orders and
  are not merged (`inventory-constants.md` §1).
- **A name is merged only when it names the same original field.** Of 130
  duplicated `pub` names, 48 are true duplicates, 23 differ in integer
  type or bitset layout, 44 are collisions, 15 are legitimately
  per-module; no true pair disagrees in value. The pairs that look alike
  and are not — `ai_load::cat` (rules.xml, `FOOT = 0`) against `form::cat`
  (the PDB's formation categories, `FOOT = 3`); `ai_load::role` (the
  original's word) against `combat::role` (the port's invention);
  `world::cell::MOUNTAIN` against `map_heights::code::MOUNTAIN` — are kept
  apart and renamed so the collision cannot recur.
- **Guards before moves.** The float guard (`crates/sim/src/no_float.rs`)
  scans the flat `src/` only, so every moved file would leave it silently;
  `docs_guard`'s seam table and test-file list are keyed by flat paths;
  `docs/` cites `crates/sim/src/<file>.rs` 2,909 times across 657 files.
  The first item makes each of these recursive or path-checked and adds
  the shape counter, red first, before a single file moves.

## 2. The shape, and its targets

The track's number: `tools/shape.py` prints these on the tree and the
queue's `Structure:` line carries them as `flat · test-tails · sweep-lines ·
names · fields · largest`, in this table's order; each may only fall.

| count | at `db6c12985` | target | source |
|---|---|---|---|
| `.rs` files in the flat root of `crates/sim/src` | 203 | ≤ 12 (`lib.rs`, the subsystem `mod.rs` files' parents, the guards) | structure §2 |
| lines of inline `#[cfg(test)]` tails inside production files of `crates/sim/src` | 65k across 100 files (19k more in sibling test files already) | 0 — in sibling `tests.rs` files or the crate's `tests/` | tests §2 |
| lines of sweep code under `crates/sim/src` | 149.5k in 1,462 files (124k of them obj-with-windows, 58 % generated row parsing and asserts, 32 % one generated doc line) | ≤ 60k, outside `src/`: the generated parsing folded into one harness function taking the port as a closure, the hand-written ports kept | tests §1 |
| `pub` constant names defined more than once | 130 | 15 (the per-module ones) | constants §1 |
| fields of `Sim` | 92 | ≤ 24: the core, the object store, the load-time tables, the per-player records, and one state per subsystem | structure §1b |
| lines of the largest production file | 14,223 (`orders.rs`) | ≤ 4,000 | structure §3 |
| `Sim` fields and methods `rondata` touches directly | 56 fields, 161 methods | through `observe` and `command` only, with the harness's setup paths named as the third door | structure §4 |

## 3. The map

Twenty-three directories under `crates/sim/src`, from `inventory-structure.md` §2, every one of the 203 files and 92 fields assigned there (the file-by-file table is in the inventory and is the move list; the directory is what the queue names).

| directory | files · lines | `Sim` fields it owns | notes |
|---|---|---|---|
| `core/` | 4 · 11,503 | `tuning`, `rng`, `frame`, `in_play`, `game_seed` | the tick (`lib.rs` 7068–7473, moved verbatim with its 12 `mark` phases), `rnd`, the frame constants, `Tick` |
| `num/` | 2 · 802 | — | the software double and the fixed-point helpers |
| `world/` | 3 · 3,272 | `world`, `mountains`, `cliffs`, `chain_heads` | |
| `mapgen/` | 20 · 14,105 | — | the map generators; their per-generator draw-site addresses stay per file |
| `territory/` | 6 · 3,775 | `players`, `sources`, `borders`, `supply`, `territory_won`, `borders_fixed`, `border_pass`, `good_seen_bits` | attrition and supply live here |
| `vision/` | 2 · 2,172 | — | fog and line of sight |
| `units/` and `units/think/` | 14 · 9,482 and 9 · 5,584 | `units`, `unit_types`, `marks`, `unit_stats_dirty`, `leader_options`, `art`, `muster` | `Unit`, `UnitType`, `Muster` and the object-number allocator leave `lib.rs` |
| `movement/` | 14 · 17,576 | `coll`, `path_memo`, `coll_copies`, `pf_start`, `repaths`, `mesh`, `plan_roads` | collision, paths, roads, `process_movement` from `lib.rs` |
| `orders/` | 6 · 15,289 | `pushed`, `last_group`, `set_aside`, `spells` | `orders.rs` cut six ways: order types, the queue, think, move, jobs, attack |
| `groups/` | 25 · 17,574 | — | `group.rs` cut four ways: state, pool, query, actions; the group-order views |
| `combat/` | 17 · 39,778 | `table`, `projectiles`, `nukes`, `deaths`, `death_pool`, `mods`, `hits`, `retargets`, `second_range` | `fight.rs` (45 % test) cut six ways: stats, target validity, the attack step, damage and deaths, acquisition, projectiles |
| `spells/` | 10 · 4,827 | — | `spell` and the cast job times from `lib.rs` |
| `air/` | 4 · 8,507 | — | |
| `cities/` | 19 · 16,664 | `buildings`, `build_types`, `cities`, `nation`, `farm_order`, `wonders_built`, `wall_stats_dirty`, `building_high`, `city_tally`, `lost_city_stamp` | `Building` and the production queue (`lib.rs` 3670–5278) |
| `economy/` | 11 · 14,304 | `holdings`, `ledgers`, `good_types`, `redirects`, `market`, `caravans` | pricing (`lib.rs` 2540–3541, `TypeData::get_cost`'s ~45 discount arms) |
| `leaders/` | 2 · 738 | `at_war`, `allied`, `treaties`, `agendas`, `defeated`, `forced_march` | diplomacy from `lib.rs` 2061–2198 |
| `tech/` | 1 · 3,387 | `tech_tree`, `setup`, `tech` | tech gain from `lib.rs` 5732–6342 |
| `ai/`, `ai/army/`, `ai/transport/` | 15 · 24,547, 8 · 8,227, 1 · 3,880 | `lobby`, `ai`, `ai_speed`, `ai_off`, `scripts`, `script_env`, `armies`, `transport`, `docks` | |
| `gaia/` | 4 · 2,302 | `gaia`, `goody_items`, `flock_stamp` | |
| `trace/` | — | `staged_marks`, `trace_phases`, `phase_marks`, `trace_costs`, `cost_marks`, `road_marks`, `building_log`, `removed`, `sweep_watch`, `probe_refuse`, `ground_inexact` | the sync trace: `mark` and the `SITE_*` draw-site constants, which are the lockstep diff's vocabulary and stay public |
| `data/` | 2 · 15.8k | — | `launch.rs` and `pivot.rs`: 8.6k lines of constant tables (art piece ids), 0.4k logic, 5.9k tests; tables, not subsystem complexity |
| `ids/` | new | — | §4 |

**Shared state** (`inventory-structure.md` §1c) belongs to no directory: `units` (77 production readers, 42 writers), `buildings` (62, 21), `world` (67 readers; 38 files write through its 33 `&mut self` methods) and `rng` (34 files advance it) are the object store and the game's two globals and sit in the `Sim` itself; `unit_types`, `build_types`, `tuning`, `tech_tree`, `nation`, `setup` are read by 27–55 files and written by at most three, so they become one immutable load-time `tables` bundle; 18 fields are per-player slices of the original's `LeaderData`, and gathering them into one record is a layout change, done last and only if the tranche reaches it.

**The view pattern** (§2b): 107 files define 138 traits; the live side is an adapter struct over `&mut Sim`, the sweeps implement the same trait. It is the crate's deep-module seam already, and the map keeps every view beside the walk it serves. Thirty views have no live adapter (sweep-only) and three different traits are named `GatherView`; the move renames them.

**The public surface today** (§4): `rondata` reads 56 `Sim` fields, calls 161 methods and `Sim::new`, and names 72 flat module paths including the `SITE_*` constants of ~25 modules; `sim/tests/sweep_*.rs` re-mount the sweeps with `pub use sim::*`, so flat `crate::<mod>` paths are public API. Every move keeps a `pub use` shim at the old path in `lib.rs` until the sweeps have left and `rondata` reads through the doors; the shim is deleted by the last item.

## 4. Names defined once

`sim` has no dependencies and `rondata` depends on `sim`, so the single home for every id family is a module inside the sim crate, `sim::ids` (`inventory-constants.md` §2; a new crate is not earned). It carries the nation ids (19, today in `ai.rs` and `nations.rs`, agreeing on every pair), the type ids (no central definition today: `nations::ty`, `transport::ty`, `ai_census::ty` and loose `i32`s; `rondata` re-declares about 30 privately as `TypeIndex − 0x32`), the spell ids, the order kinds (one integer type, not `u8` here and `i32` there), the army status bits, the terrain bits, `UNITS_PER_TILE` (0xc0, used bare 132 times) and its multiples, the quarter and half turn, the 8.8 one, the percent denominator. `rondata`'s job becomes checking each family against the user's XML — only the roster has that check today (`rondata/src/bin/rondata.rs`) — and its own `Resource` enum (`Gold` where `sim` says `Wealth`) goes. The 23 type-drifted pairs unify on the field's own type; the 44 collisions take the renames the inventory proposes per row; `combat::role` and `balance::line`, two bitsets encoding the same nine lineage tests at different bits, become one.

## 5. The moves, in order

Each is one queue item of the `structure` kind, gated as §1 says; the item names the directories or files it touches and the shape counts it moves.

1. **The guards and the counter.** `no_float.rs` scans recursively; `docs_guard`'s seam table and test-file list read nested paths; a guard pins that every `crates/sim/src/…` path `docs/` cites exists (2,909 citations, 657 files), with `tools/paths.py` rewriting citations when a file moves; `tools/shape.py` prints §2's counts and the queue's `Structure:` line is parsed against them. Red first on the tree as it stands (`do_attack/live.rs` is already outside the float guard).
2. **The sweeps leave the source.** Not as a table — 667 sweeps keep no generator line, every file carries a hand-written port block (42 % of the obj-with-windows lines), and of the 786 regenerated in a mirror only 292 came back byte-identical (`inventory-tests.md` §1). What is true: 1,309 of the 1,462 already compile as four integration targets (`crates/sim/tests/sweep_*.rs` `#[path]` into `src/sweep/standalone_*.rs`) against the public API, and only the 148 in `sweep/mod.rs` need crate-private access. So the 1,309 move out of `src/` as files, the 148 stay listed, the three tracked `.rs-E` sed backups go, and one shared harness function taking the port as a closure replaces the generated row-parsing and assert code (about 70k lines). **`tools/emu` and the Python twins are not touched**: the emulator cache's key hashes every file there (`support.rs:574-633`), and an edit sends the 13 GB, 2,344-entry cache cold; the twins are needed on a cache miss.
3. **The in-file tests move to siblings.** `foo.rs`'s `#[cfg(test)] mod tests` becomes `foo/tests.rs` with `use super::*`, keeping private access; `cities_tests.rs`, `harness_tests.rs` and the other whole test files go to the crate's `tests/` where they need no private items; `orders.rs`, `air.rs` and `cast.rs` have production code after a test mod and are cut by hand.
4. **The directories.** The §3 map, two to four directories a landing, each a `git mv`, the `mod` tree, the `pub use` shim and the citation rewrite; the three `GatherView`s renamed.
5. **`sim::ids`.** §4, one family a landing where the family is large (type ids), the rest together.
6. **`lib.rs` split.** `Unit`, `UnitType`, `Muster` and the allocator to `units/`; `Building` to `cities/`; pricing to `economy/`; the production queue to `cities/`; tech gain to `tech/`; the tick to `core/` verbatim; `mark` to `trace/`; `rnd` to `core/`. `Sim`, `new` and the module tree stay, about 800 lines.
7. **The four cuts.** `orders.rs`, `fight.rs`, `group.rs` along §3's lines; `launch.rs` and `pivot.rs` to `data/`.
8. **The `Sim` split.** The load-time `tables` bundle; one state struct per subsystem directory holding its §3 fields; the object store and `rng` stay on `Sim`; the per-player record last and optional.
9. **The two doors.** `sim::observe` (`docs/RENDERER.md` §2.3) as the read surface and `sim::command` as the order surface, defined without changing `rondata`; then `rondata` moved onto them door by door, the harness's setup paths named as the third; the shim deleted when nothing reads the old paths.
10. **The README**, written for a person: what this is, the finish line (`docs/TRACKS.md`), how to run the sim against an install, where the documents are.

## 6. What is not established

- Whether the shared harness function reproduces every obj-with-windows sweep's assertions exactly; the emulator cache's receipts are the check (a sweep whose rows re-run is a key change, so the fold must keep each sweep's script and tsv rows as they are).
- The `rondata` split (`diff/` is 175k lines, 12k of them non-test harness code) is not in this tranche: a cycle-free cut takes `golden.rs`, `input.rs` and `debug_view` along, and the 1,788 exact `diff::` test names in the gate's timings file would stop matching without an error, so lane gates would run the 5,079 s of slow tests rather than skip them (`inventory-tests.md` §3). It waits on a rename that `release_gate.py`, `isolated_tests.py` and about 14 tools read.
- Whether `rondata`'s 161 direct method calls all fit two doors; the third door, the harness's setup paths (`init_build`, `activate`, …), is named so the count is honest.
- The per-player record (§3): a layout change, not a move; the tranche may stop before it with the counts still honest.
- `docs_guard`'s test-file list parses `lib.rs`'s `mod` lines; nested `mod.rs` files change the shape it reads, and item 1 re-checks the parser before item 3 moves anything.
