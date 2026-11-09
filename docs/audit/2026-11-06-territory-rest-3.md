# Territory reading — group `rest-3` (2026-11-06)

Reader: Opus 5.5, the seventy-first tranche's territory fan-out. Group: 35 functions of the
`techtype`, `terrainoil`, `transitions`, `unbuiltcities`, `unbuiltforts`, `unbuiltwonders`,
`units`, `wonder` and `wonders` stems. Source: the decompile export
(`~/ghidra-projects/decomp/funcs/`), `crates/sim/src`, `docs/`. One ruling per function; the
ruling turns on the expression quoted, nothing transcribed.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00472a60 | BuildTypeData::num_preq | layout | run1016–1021 | twin | `crates/sim/src/tech.rs:966` `Tech::num_preq`: a building or tech (any non-unit, non-good kind) answers `flag_reads::BUILD_NUM_PREQ` = 3 (`crates/sim/src/flag_reads.rs:44`); the original's body is `return 3`. Agree. Backed by `crates/sim/tests/sweep/build_type_data_num_preq.rs`; `layout` only because the fn's doc comment spells `num_preq()` rather than `BuildTypeData::num_preq` (the rubric's spelling test, `docs/RUBRIC.md` §0), so a one-line doc fix would make it `port`. `docs/CENSUS.md:873` already calls it a port consumer. |
| 0066c310 | TechType::set_disabled | layout | - | dead | Sets or clears bit `who` of `+0x1e2 leader_off` (`who < 8`). **No caller in the export**: the scenario script verbs `ScenarioFuncSet::disable_tech@009ea2b0`/`enable_tech@009ea590` write `+0x1e2` inline. The crate carries the field (`crates/sim/src/tech.rs:185`, zero at load) and its one reader (`crates/sim/src/ai_research.rs:401`); in a skirmish nothing writes it. Census: a scenario-trigger mark — proposed pattern `^ScenarioFuncSet::` on the class (the writers), and this function by `is_cut` (no caller). |
| 0066d5c0 | TechType::walk_rules_data | layout | run1016–1021 | saveload | Walks `+0x4..+0x5e`, the name `String` (`+0x74`), `+0x1c8..+0x1e3` and eight `String`s at `+0x1e4` through a `DataWalk`, the strings skipped when `checksum != 0`. The rules checksum and the save (`docs/ORACLE.md:86`, `Game::walk_rules_data(SaveGame)`); entered because every game computes the rules checksum at its start. Never a rule. Proposed replaced group **"save, load and the rules checksum"**: pattern `::(walk_data|walk_rules_data|backup|restore)$`. |
| 0066d810 | TechType::restore | layout | - | saveload | Forwards to `Type::restore(TypeBak*)` when the backup is non-null: the type table's backup/restore around a scenario or a load (`docs/journal/2026-10-13-item-1932.md`). Same proposed group, `::(backup|restore)$`. |
| 0066d830 | TechType::backup | layout | - | saveload | Forwards to `Type::backup(TypeBak*)`; the twin of `restore`. Same proposed group. |
| 008a28e0 | TerrainOil::create_doobers | name | run1016–1021 | twin | Two twins split by effect. **The cell write** — a cell with rock `0x8`, `0x100` clear and `land` 1 or 2 loses `0x8` (`& 0xfff7`) — is `crates/sim/src/halfland.rs:753` `create_doobers_flags`, same predicate and mask. **The height** — a cell with rock still set and not ocean (`(0x100 clear) && land∈{1,2}` is the original's ocean test, `crates/sim/src/mapgen.rs:447` `is_ocean` the same) reaches `add_rocky_patch`, which terraforms only an `OIL` (`0x800`) cell — is `crates/sim/src/map_heights.rs:789` `create_doobers` (`ROCK && OIL && !is_ocean`). Agree under the stated assumption (map_heights.rs:784–787) that the second arm — flag 1, a good `find_good_at` finds, `LeaderData::type_avail(console, good) == 0`, `semaphore & 8` clear — never holds on an `OIL` cell. That arm also calls `add_rocky_patch` on a rockless cell (render patch only, no height unless `0x800`). The patch array (`patch_data`) is render data. `name` because the rubric's `backed` set (a sweep or a coverage section, `tools/census.py:1222` `territory_kind`) does not hold it, though the crate defines and spells it; its diff is the MAPGEN height comparison (`docs/MAPGEN.md:1325–1340`), which the rubric does not count — a rubric question, not a port gap. |
| 008a2b30 | TerrainOil::add_rocky_patch | open | run1016–1021 | twin | Height: `if (cell & 0x800) terraform_for_building(div_3_table[(12x+1.5)], div_3_table[(12y+1.5)], 4, 4)` = `crates/sim/src/map_heights.rs:801` `terraform_oil` / the body of `create_doobers` (box `4x, 4y, 4, 4`). Agree. The rest allocates an `OilPatchData` (0x3c), records five vertex indices (four corners, one at `Random::get(&internal_random,0,3)` offsets) and nine more at `get(0,7) − 2` offsets: **render data, drawn from `internal_random`**, which is not the synced stream (`docs/SYNC.md:64`, `docs/MAPGEN.md:1108`), so the 4 + 18 draws move no sim state. Absorbed for that part. |
| 008a2ac0 | TerrainOil::remove_rocky_patch | layout | - | editor | Frees the cell's `OilPatchData` and nulls the slot. Callers: only `ScenarioEditor::on_edit_feature@0099d900` and `ScenarioEditor::remove_feature@0099e060`. Census: proposed class mark `^ScenarioEditor::` callers-only — or `is_cut` by name; the render patch store has no sim reader. |
| 008a2e40 | TerrainOil::gain_tech | layout | run1017,1018,1024,1176–1178 | ui | On a tech gained (`Leader::gain_tech@006dcb60`), for each live good of flag 1 whose type the **console's** leader can now see and whose `preq[0]` or `get_preq(1)` is the tech (and not `OIL`), clears the cell's `OilPatchData +0x39` (the rocky-patch hidden flag). Console-only, render store only: no sim reader. Census: proposed `^TerrainOil::(gain_tech\|lose_tech\|remove_rocky_patch)$`, or class-wide `TerrainOil` with `create_doobers`/`add_rocky_patch` kept (they write heights). |
| 008a2f30 | TerrainOil::lose_tech | layout | - | ui | The inverse from `Leader::lose_tech@006d2850:98`: sets `+0x39 = 1` for goods the console can no longer see. Render store only. Same proposed mark. |
| 0087f2a0 | Transitions::find_map | open | run1016–1021 | ui | Picks the terrain texture transition (`AlphaInfo`) whose mask word matches the requested neighbour bits (`param_3`/`param_4`/`param_5` against each of eight 32-bit words, in groups `7`, `0x38`, `0x1c0`, `0xe00`), then chooses one candidate by `Fractal::get_height(&TerrainData::texture_fractal, x, y)` in floats. One caller: `TerrainOut::create_transition_renderables@00862c60`. Entered because every map build makes its renderables; writes only `AlphaInfo +0x24` and a function-static candidate array. Census: proposed class mark `^Transitions::` (the renderer's transition table; `is_ui_body` misses it because the class is not `*Out`). |

## Part 2

The unbuilt lists. The original keeps three per-player arrays of `(short o, byte who)` —
`unbuilt_cities`, `unbuilt_forts`, `unbuilt_wonders` — appended by `Build::init@00629740:115–222`
(inline: a city when `Build.flags & 0x20`, a fort when the type's `is_fort` slot `+0xfc`, a wonder
for `0x20e..0x21e`) and swap-removed by `Build::activate@00623e20:447,568,1263` and
`Build::close@00628980:169,220,372`. The crate keeps no list: it derives membership as "alive,
not active, owned by `who`, of the kind" each time it is asked — `crates/sim/src/place.rs:349`
`unbuilt_cities`, `:359` `unbuilt_forts`, `crates/sim/src/city.rs:1419` and
`crates/sim/src/ai_build.rs:1354` `unbuilt_wonder_points` for the wonders. Membership agrees (an
entry lives from `init` to `activate` or `close`, which is `alive && !active`); **the list's
order does not** — the original's is insertion order disturbed by swap-removal, the crate's is
building index order. Every consumer found reads a count or an existence
(`crates/sim/src/tcoord.rs:180`, `crates/sim/src/location.rs:308,351,375`,
`get_unbuilt_wonders`, `has_unbuilt_wonder`), so the order is inert on them; an order-reading
consumer would be the first place this split shows.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00460dc0 | UnbuiltCities::walk_data | open | run1016,1497,53,54 | saveload | The generic `Array<UnbuiltCity>::walk_data` over all players' lists through a `DataWalk`: on `input` it frees and re-reads, on output it writes length and entries (the "write len exceeds size" check). Callers: `CheckSums::check_unbuilt_cities@00936f50` (the out-of-sync checksum) and the save/load walk; entered on runs with a `@dumpall`/checksum. Never a rule. Proposed group "save, load and the rules/sync checksum", `::walk_data$` on a container. The list's dump (`GameLog::dump_unbuilt_cities@0092f800`, `BEGIN UnbuiltCities` under `DUMP_ALL`) is printed but not diffed by `rondata` (`crates/rondata/src/gamelog.rs:1561` reads only the heights riding on the `UnbuiltForts` block) — a diffable record the harness leaves on the floor. |
| 0073b700 | UnbuiltCity::is_started | layout | - | twin | `get_build(o)` then `Build::vftable ? flags & 2 : vtable[0x50]()`. Twin: `crates/sim/src/location.rs:820` `unbuilt_started` reads `buildings[o].started`. Agree on both arms: the non-`Build` arm's vtable `+0x50` is `WallData::is_started` in both `Build::vftable` and `Wall::vftable` (`vtables.txt`), whose body is the same `flags & 2`, and the crate's `started` is that bit (`crates/sim/src/lib.rs:1727`). Backed by `crates/sim/tests/sweep/unbuilt_city_is_started.rs`. |
| 0073b740 | UnbuiltCity::get_x | layout | run1019,1021,1176,1179,1188,1190 | twin | Returns the building's `+0x10 ^ 0x63637` (the decoded world x) through a hidden-return `Coord`. Twin: `crates/sim/src/location.rs:823` `unbuilt_tx` = `buildings[o].pos.tile().x`, the consumer's `div_3_table` already applied (location.rs:168 states it). Agree. |
| 0073b780 | UnbuiltCity::get_y | layout | run1019,1021,1176,1179,1188,1190 | twin | The same on `+0x14`; `crates/sim/src/location.rs:826` `unbuilt_ty`. Agree. |
| 0073b7c0 | UnbuiltCities::add_unbuilt_city | layout | - | absorbed | Appends `(o, who)` to `unbuilt_cities[who]`. No out-of-line caller (inlined in `Build::init@00629740:120–127`). Covered by the derived membership above (`place.rs:349`); no list is kept. |
| 0073b810 | UnbuiltCities::remove_unbuilt_city | layout | run1016–1021 | absorbed | Finds `o` and overwrites it with the last entry, length − 1 (swap-remove). Callers `Build::activate:447`, `Build::close:169`. Covered by membership (`!active` / `!alive`); the reordering it causes is not modelled and is inert on every consumer read (see the preamble). |
| 0073b880 | UnbuiltCity::walk_data | layout | - | saveload | One `DataWalk` call over the element's three bytes; no reference in the executable, inlined in its container (`crates/rondata/src/blind.rs:2922` already rules it). Same proposed group. |
| 0073bcc0 | UnbuiltForts::walk_data | open | run1016,1497,53,54 | saveload | The fort list's twin of `UnbuiltCities::walk_data`. Same group. Its `DUMP_ALL` block is the one the height table rides on (`docs/ORACLE.md:1162`); its own entries are not diffed. |
| 0073bb20 | UnbuiltFort::walk_data | layout | - | saveload | Element walk, no reference (`crates/rondata/src/blind.rs:2929`). Same group. |
| 0073bb40 | UnbuiltFort::is_started | layout | - | twin | Byte-identical to `UnbuiltCity::is_started` but for the jump address; `crates/sim/src/location.rs:820` with `fort = true`. Agree on both arms, as for the city. |
| 0073bb80 | UnbuiltFort::get_x | layout | run1019,1188 | twin | Identical to the city's on `+0x10`; `crates/sim/src/location.rs:823` (`fort = true`) and `crates/sim/src/tcoord.rs:348` `unbuilt_tile`. Agree. |
| 0073bbc0 | UnbuiltFort::get_y | layout | run1019,1188 | twin | `+0x14`; `crates/sim/src/location.rs:826`, `tcoord.rs:348`. Agree. |
| 0073bc00 | UnbuiltForts::add_unbuilt_fort | layout | - | absorbed | Append; inlined in `Build::init@00629740:192–198` (on the `is_fort` slot `+0xfc`). Covered by `crates/sim/src/place.rs:359` (`build::is_fort`). |
| 0073bc50 | UnbuiltForts::remove_unbuilt_fort | layout | run1019,1020,1022,1176–1178 | absorbed | Swap-remove; callers `Build::activate:568`, `Build::close:220`. Covered by membership; `crates/sim/src/city.rs:2705` and `crates/sim/src/ai_sites.rs:305` cite it. |
| 0073c290 | UnbuiltWonders::walk_data | open | run1016,1497,53,54 | saveload | The wonder list's twin of `UnbuiltCities::walk_data` (the generic `Array` walk). Same group. |
| 0073c050 | UnbuiltWonder::is_started | layout | - | twin | Identical shape to the city's (both arms `flags & 2`). No out-of-line caller found; the crate asks no wonder-specific "started" (its wonder consumers count alive, inactive sites regardless of start). Twin by predicate with `crates/sim/src/location.rs:820`'s field (`Building::started`). |
| 0073c1b0 | UnbuiltWonder::walk_data | layout | - | saveload | Element walk, no reference (`crates/rondata/src/blind.rs:2936`). Same group. |
| 0073c1d0 | UnbuiltWonders::add_unbuilt_wonder | layout | - | absorbed | Append; inlined in `Build::init@00629740:215–222` (types `0x20e..0x21e`, or the type's own wonder slot). Covered by the derived sites in `crates/sim/src/city.rs:1419` (`is_wonder && !active`) and `crates/sim/src/ai_build.rs:1354`. |
| 0073c220 | UnbuiltWonders::remove_unbuilt_wonder | layout | run1176–1179,1188,1190 | absorbed | Swap-remove; callers `Build::activate:1263` (the wonder arm, before `Wonders::init_wonder`), `Build::close:372`. Covered by membership; the order is inert on `get_unbuilt_wonders` (a count), `has_unbuilt_wonder`/`team_has_unbuilt_wonder` (existence), `get_unbuilt_wonder_value` (a sum). |
| 0061a960 | Units::make_valid | layout | run1020,1176,1177,1179,1215 | absorbed | Grows `units[who]` so slot `o` is valid (capacity via the array's grow slot when `o + 1 > cap`) and raises its length to `max(len, o + 1)`. Reached only past the 200 preallocated slots (`docs/GOLDEN.md:8885`). The crate keeps units by object number with its own high-water mark — `crates/sim/src/lib.rs:2308` `find_free` (`[base, mark)` scanned, then `mark`), which is the number this length bounds; a `Vec` needs no grow. Covered. |
| 0073c660 | Wonder::get_timer | layout | run1176–1179,1188,1190 | dead | `(0x21e − type) · scale(wonder_age) + scale(wonder_timer)`, each `scale(k) = max(1, (size/2 + world_xs · k) / size)` (0 when `k < 1`), `size` = `map_sizes[·] +0x144`. One caller, `Wonders::init_wonder@0073c860`, which stores it at `Wonder +0x8`. **No rule reads `Wonder +0x8`**: the readers of the wonder entries (`LeaderData::get_wonders`, `has_wonder`, `get_wonder_value`, `get_team_wonders`, `BuildData::construct_time`, `Object::do_launch`) read `+0x2`, `+0xc` and the type; the wonder victory's clock is `Game::wonder_timer@005944f0` against `leader +0x448` (`GameDaemon::process_victory@00730ef0:191`), not this field. A dead store in a skirmish; the crate's `Sim::note_wonders` (`crates/sim/src/ai_census.rs:1287`) keeps the slot and the mark and rightly no timer. The value is printed by `WonderData::log_data` in the `WONDERS` block (`GameLog::dump_wonders@0092fae0`, `DUMP_ALL`), which `rondata` does not parse. Census: no pattern fits a live dead store; propose an explicit name list ("computed, stored, unread"). Sweep-backed (`crates/sim/tests/sweep/wonder_get_timer.rs`). |
| 0073c780 | Wonder::walk_data | layout | - | saveload | One `DataWalk` call over the element's 0xe bytes; inlined in `Wonders::walk_data`. Same group. |
| 0073c7a0 | WonderData::get_value | layout | - | twin | `objects[who][o]`'s vtable `+0x14c`; when that is `ObjectData::get_wonder_value`, the type's vtable `+0x118`. Twin: `crates/sim/src/build.rs:234` `BuildType::wonder_points` (`ObjectTypeData::get_wonder_value@00661ba0`'s slot `+0x118`), summed by `crates/sim/src/ai_build.rs:1335` `wonder_points`. Agree: every object vtable in `vtables.txt` (`Unit`, `Build`, `Wall` and their `*Data`/`*Out`) holds `ObjectData::get_wonder_value` at `+0x14c`, so the override arm is never taken and the crate's one path is the whole function. No out-of-line caller (inlined in `LeaderData::get_wonder_value@006ebb90`). |
| 0073ca40 | Wonders::walk_data | open | run1016,1497,53,54 | saveload | The generic `Array<Wonder>` walk over all players (`input` frees and re-reads, output writes `+0x8` timer, `+0x4` frame, flags); callers the checksum and save/load. Same group. |

## Summary

35 functions: **twin 11 · twin-differs 0 · absorbed 7 · saveload 11 · ui 3 · editor 1 · dead 2 ·
unmodelled 0**.

- twin 11: `BuildTypeData::num_preq`, `TerrainOil::create_doobers`, `TerrainOil::add_rocky_patch`,
  `UnbuiltCity::is_started`/`get_x`/`get_y`, `UnbuiltFort::is_started`/`get_x`/`get_y`,
  `UnbuiltWonder::is_started`, `WonderData::get_value`.
- absorbed 7: `UnbuiltCities::add_unbuilt_city`/`remove_unbuilt_city`,
  `UnbuiltForts::add_unbuilt_fort`/`remove_unbuilt_fort`,
  `UnbuiltWonders::add_unbuilt_wonder`/`remove_unbuilt_wonder`, `Units::make_valid`.
- saveload 11: `TechType::walk_rules_data`, `TechType::restore`, `TechType::backup`,
  `UnbuiltCities::walk_data`, `UnbuiltCity::walk_data`, `UnbuiltForts::walk_data`,
  `UnbuiltFort::walk_data`, `UnbuiltWonders::walk_data`, `UnbuiltWonder::walk_data`,
  `Wonder::walk_data`, `Wonders::walk_data`.
- ui 3: `TerrainOil::gain_tech`, `TerrainOil::lose_tech`, `Transitions::find_map`.
- editor 1: `TerrainOil::remove_rocky_patch`.
- dead 2: `TechType::set_disabled` (no caller), `Wonder::get_timer` (a dead store).

**Proposed census classes.**
- `^Transitions::` (class): the terrain renderer's transition table — catches
  `Transitions::find_map` here; `is_ui_body` misses it because the class is not `*Out`.
- `^TerrainOil::(gain_tech|lose_tech|remove_rocky_patch)$`: the oil patch render store, console
  visibility and the editor's removal — catches those three. (`create_doobers` and
  `add_rocky_patch` stay: they write heights.)
- An explicit name list for out-of-line copies with no caller and dead stores:
  `TechType::set_disabled` (its writers are `ScenarioFuncSet::disable_tech`/`enable_tech`, the
  scenario script verbs, which a `^ScenarioFuncSet::` class mark would put under the cut
  scenario), `Wonder::get_timer`.

**Proposed replaced group: "save, load and the rules/sync checksum".** Pattern
`::(walk_data|walk_rules_data|backup|restore)$`. Evidence: every `walk_data` here is a `DataWalk`
pass whose callers are `CheckSums::check_*` (the out-of-sync checksum), the rules checksum
(`Game::walk_rules_data`, `docs/ORACLE.md:86`) and the save/load walk; `backup`/`restore` copy the
type table to a `*Bak` around a scenario or a load. None computes a rule; the lockstep entries
(run1016, run1497, run53, run54) are the start-of-game checksum and `@dumpall`. Catches the 11
`saveload` rows above. Caveat: the pattern must not touch the order classes' `walk_data`, which
are already "the order classes' plumbing", nor any ported `walk_data` (the mark never touches a
port, `tools/census.py:1187`).

**Unmodelled rows: none.** The lockstep-entered rows are all twins, absorbed, render or
checksum/save walks.

**Two fidelity notes that are not rulings.**
1. The unbuilt lists' **order** (insertion order with swap-removal in the original, building index
   order in the crate) is not modelled; every consumer found reads a count, a sum or an existence.
   It would show first in the `BEGIN UnbuiltCities` / `UnbuiltForts` / `UnbuiltWonders` blocks of a
   `DUMP_ALL` dump (`GameLog::dump_unbuilt_cities@0092f800`), which `rondata` prints past without
   diffing (`crates/rondata/src/gamelog.rs:1561` reads only the heights riding on `UnbuiltForts`).
   Those three blocks, and `WONDERS` (`GameLog::dump_wonders@0092fae0`), are dumped records no
   reader parses — candidates for `rondata::diff::coverage`'s unparsed-key pin.
2. `BuildTypeData::num_preq` and `TerrainOil::create_doobers` are ported in substance but count as
   `layout` and `name`: the first because `tech.rs:966`'s doc comment spells `num_preq()` without
   the class, the second because no sweep or coverage section backs it (its diff is MAPGEN's
   height comparison). A doc-comment spelling fixes the first; the second is a rubric question.

## Not established

- **The wonder victory clock in the crate.** Ruling `Wonder::get_timer` dead rests on finding no
  reader of `Wonder +0x8`; the victory reads `Game::wonder_timer` and `leader +0x448` instead. I did
  not find a crate twin of `GameDaemon::process_victory`'s wonder countdown or of
  `Game::wonder_timer` outside its sweep (`crates/sim/tests/sweep/game_wonder_timer.rs`); whether the
  wonder victory is modelled elsewhere is another group's question.
- **`create_doobers`' second arm** (flag 1, a good the console's leader cannot see, `semaphore & 8`
  clear) — the crate assumes it never holds on an `OIL` cell (`crates/sim/src/map_heights.rs:784`);
  I did not re-verify `World::set_oil_at` sets no flag-1 good. On a non-oil cell the arm writes only
  render data, so the assumption matters only for heights.
- **`Wonder +0x8` readers** were searched among the functions that touch `wonders.field_0x10`; an
  access through a different base (a cached `Wonder*`) would escape that search.
- **The add sites in `Build::init`** were read at the list writes (lines 115–222); the exact city
  predicate is `Build.flags & 0x20`, which I took to agree with the crate's `build::is_city` without
  tracing where `0x20` is set.
