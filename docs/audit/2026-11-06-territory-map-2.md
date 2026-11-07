# Territory reading, group `map-2` (2026-11-06)

The seventy-first tranche's reading fan-out: 14 functions of the `map` stem,
all of kind `layout` (a `crates/sim/src/sweep/*.rs` test holds the original
under unicorn to an inline port in the test itself, and no non-sweep code
names the function). Each was read by name in the decompile export
(`~/ghidra-projects/decomp`), its callers taken from
`tools/trace/report.py <exe> refs` (call sites and vtable slots in the PE,
not name greps), and ruled against `crates/sim` and the data layer that
feeds it. Reader: Opus 5.5. No other file touched, nothing committed.

**Frame, as the sibling `map-1` report states it.** The simulation does not
generate its start map for a captured game: it loads `WORLD=6` and the height
grid from the start dump. A generator exists for four styles (Great Sahara,
Southwest Mesa, Great Lakes, East Indies), pinned by the setup checksum trace
(`docs/MAPGEN.md` §3, §15). So a map-generation function entered by a lockstep
run can never part a dumped field of that run; its falsifier is the setup
checksum line at its checkpoint (`Mapgen` 7807 mask, 7927 resources) on a run
of its style. `twin` and `twin-differs` are used as in `map-1`: a difference
only an unported style reaches is named in the evidence, not counted.

**What `layout` means for this group.** Each sweep's "port" is a closure in
the test (`let got = { … }`), never a call into `sim`; so the sweep proves the
reading of the function's arithmetic but not that the crate uses it. Every
ruling below therefore names the production code (or its absence)
independently of the sweep.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0068a790 | ResourceDivvyPool::add_water | layout | - | dead | Appends a good `n` times to `water_goods` (grow via the list's slot 0 at capacity). `refs`: dead, byte before 0xcc; no name caller in the export. Its loop is inlined in `add_good` (the Fish `9`-times tail). Census: the `UNCALLED` frozenset `map-1` proposes. |
| 0068a7d0 | ResourceDivvyPool::add_late | layout | - | dead | Appends one good to `late_goods`. `refs`: dead, byte before 0xcc; inlined in `add_good`'s `2 < age` arm. Census: `UNCALLED`. |
| 0068a800 | ResourceDivvyPool::add_early | layout | - | dead | Appends one good to `early_goods`. `refs`: "pointer only", but the one hit is a byte run inside `.rsrc` (00ee666b), resource data, not a reference; no call, no vtable slot. Inlined in `add_good`'s fall-through. Census: `UNCALLED` (the `.rsrc` coincidence should not keep it off the list). |
| 0068a830 | ResourceDivvyPool::add_good | layout | run1016..1021 | absorbed | Files one good: if it is in the ocean land's rare list (`lands` data `+700` count, `+0x2c0` array) it goes to `water_goods`, Fish (6) ten times in all; else `age > 2` (`ObjectTypeData::age` at +0x278, or the type's age method when it is −1) to `late_goods`, else `early_goods`. One caller, `Map::place_resources+0x89`, over goods 6..0x31 with `random_rare_drop`. No `sim` fn: the lists are built by the data layer, `crates/rondata/src/mapgen.rs:577–592` (`resource_inputs`: `ocean.contains(&g)` → water, `repeat_n(g, 10)` for g = 6; `LATE_AGES` (Gunpowder on, i.e. age ≥ 3) by `PREQ0` → late; else early), and handed to `sim::map_resources::place_resources` as `Inputs.{early,late,water}` (`crates/sim/src/map_resources.rs:84–88`). Predicates compared: water test first, Fish ×10, `age > 2` ↔ Gunpowder-or-later, list order = good index order; agree. Diff-backed: every `get_early`/`get_late` draw indexes these lists at the 7927 checkpoint (MAPGEN §8, §15). The one reading-only point: the port takes the age from `PREQ0`'s name rather than a set `age` field; `Data/resourcerules.xml` has no `AGE` tag in its tag set (checked 2026-11-06), so +0x278 starts −1 and the type's age method (vtable +0xe8; `GoodType`'s table is not in `vtables.txt`, so not read) supplies it — the port's reading is that this is `PREQ0`'s age; the 7927 diff would part otherwise. |
| 0068b0a0 | Map::is_near_ocean | layout | - | unmodelled | Answers 1 when (a) `r_out ≠ 0` and some cell in the `circle_radius[r_out−1] .. circle_radius[r_out+1]` ring has tile byte +2 set, and (b) no cell inside `circle_radius[r_in]` has it; 0 off the world or past radius 0x40. Callers (`refs`): `Map::place_start_in_region+0x1f8`, `MapEastMeetsWest::make_continents+0xb06`/`+0xcb0`, `MapMediterranean::make_continents+0x57c` — British Isles, East Meets West, Mediterranean, all unported styles (`map-1` rules `place_start_in_region` unmodelled for the same reason). No `sim` fn (grep `near_ocean` outside `sweep/`: none). Would show: the seed and part 2 at `Mapgen` 7807 on a setup trace of style 19, 22 or Mediterranean. |
| 0068b280 | Map::get_player_overage | layout | run1016,1018,1019,1021,1062,1176 | twin | Split across two places: the allowance table is `crates/rondata/src/mapgen.rs:1075` `overage_allowance` (map size 0,1 → 2; 2 → 3; 3 → 4; 4 → 6; else 8 — the original's switch on `game.info.map_size`, agree), carried as `LakeStyle::overage_allowance` (`crates/sim/src/map_lakes.rs:67`); the gate is `crates/sim/src/map_lakes.rs:419` `scale_land_area`'s `players <= allowance` ↔ the original's `param_1 <= allowance → 0` (agree). Consumers on ported styles: `map_lakes.rs:517`, `map_east_indies.rs:74–76`; diff-backed at 7807 for GL and EI. One arm differs and is unreachable: `param_1 == 0` returns the allowance (non-zero, so `scale_land_area` takes its float path), where the port's `players == 0` returns `v`; no lobby has zero nations. The over-allowance arm (`param_1 − allowance`) is what `map-1` already rules `twin-differs` under `scale_land_area`. Other callers (`MapBritishIsles`, `MapWarringStates::make_continents`) are unported. |
| 0068b2f0 | Map::get_player_underage | layout | - | dead | `min(param, cap − param)` with `cap` by map size (2,2,2,4,5,7, else 8), 0 when either is negative. `refs`: dead, byte before 0xcc; no caller in the export. Census: `UNCALLED`. |
| 0068bc90 | Map::make | layout | run1016..1021 | unmodelled | The map-generation driver (vtable +0x64 of every `Map` class; `refs`: pointer only, 21 vtables). Most of it is **absorbed**: it seeds `world+0x7c` and `game_random` from `param_2`, runs `load_map_data`, draws `base_edge = r % 4` when −1, dispatches the style mask (vtable +0x40), then the fixed pass sequence (`fix_diag_land`, `make_coastlines`, `fill_fertile`, `place_all`, `check_player_forest`, `nubify_forest`, the three `change_*_base`, `fix_transitions`, `place_resources`, `adjust_for_blocking`, `compute_all_vals`, the `sea_map` tail), each pass with its own twin (MAPGEN §2–§10) and the sequence driven only by the test drivers `great_sahara_at_7927`/`great_lakes_at_7945`/`east_indies_at_7945`/`southwest_mesa_at_79{27,45}` in `crates/rondata/src/mapgen.rs`; the playing sim loads the result from the start dump instead. **One write is a rule the crate does not have**: right after the mask, `make` copies the style's six territory limits into the world — `world+0x38/0x3c/0x40 = player_territory_limit{,_civic,_city}`, `world+0x44/0x48/0x4c = colonized_territory_limit{,_civic,_city}` — which `Map::init_map_data` reads from the style file (lines 552–792, keyed through the string table; the values are the style files' `PLAYER_/COLONIZED_TERRITORY_*_HARD_LIMIT` tags, as the dumps below confirm) (default `TERRITORY_LIMIT_*`, `load_map_data` lines 55–60), and `World::compute_reg_territory@006b0bb0` (lines 114–123) reads them, the player triple where `Region::flags & 4`, the colonized triple elsewhere. The crate's border pass reads the rules constants only (`crates/sim/src/territory.rs:167–175`, `t.territory_limit_{base,civic,city}`), and no harness code feeds the dump's limits in (`territory_limit` appears in `rondata` only in a fixture and a comment). Shipped overrides: NILEDELTA.xml and SEAPOWER.xml player 48/8/8; NEWWORLD.xml colonized 32/4/4. **Confirmed on disk**: the start dump's `WORLD` scalars read `player_territory_limit 48`, `_civic 8`, `_city 8`, `colonized_territory_limit 44` on run1098 (Nile Delta) and run1126 (Atlantic Sea Power), and `colonized_territory_limit 32` on run1123 (New World), against 44/4/4 ×2 on run1016. Entered by every lockstep run and inert there: their styles leave all six at the constants (run1016's dump), and no `rondata::diff` pin is on any of the three styles (the 1098/1123/1126 captures are coverage runs, no twin taken). Would show: `WORLD` cell ownership (`who`) after the first `compute_reg_territory` on a lockstep run of Nile Delta, Atlantic Sea Power or New World — a city's border reaching 48 rather than 44 tiles, or a colonized region's 32; gamelog category `WORLD` at ≥ 5. `docs/ATTRITION.md` (the `WorldData` row, "Seeded from `TERRITORY_LIMIT_*`") does not name the style override. |
| 0068d1a0 | MapConquest::check_sea_map | layout | - | ctw | Sets the conquest map's `+0x48` to 0 when `Regions::get_num_sea` is 0, else 3 when two starts' tiles (`world+0x90`/`+0xac` start lists, tile short +4) lie in different regions, else 2. One caller (`refs`): `MapConquest::make_continents+0x994`, Conquer the World's mask; already ruled residue by `docs/DECISIONS.md` (line ~10577) and described in `docs/AI.md` (~2347). The skirmish's `world.sea_map` is the style file's `SEA_MAP`, which `crates/sim/src/terrain_init.rs:27–40` documents and the sim carries from the dump. Census: `map-1`'s proposed class mark `^MapConquest::` (ctw), which names this row. |
| 00692e70 | Map::calc_bonus2 | layout | - | dead | Map vtable +0x54 (`refs`: pointer only, 22 vtables, no call). A lattice selector: with `f` = the cell's region's `goody_factor` (region record +4; the map's own `goody_factor` for region 0), true when `(x%f)·f + y%f == (13·(x/f) + 11·(y/f) + world.seed) % f²` — one cell per `f×f` block, by a hash. No dispatch of slot +0x54 on a `Map` receiver anywhere in the export (searched every `+ 0x54))(` site in the `Map*`, `World`, `Regions`, `Terrain*`, `Setup`, `Leader`, `Game` classes: none but an unrelated Steam stub); `make_region` copies the factor into the region record and `Leader::plan_strategy` reads that field directly, not through this. Census: proposed `UNDISPATCHED` (a vtable slot no site calls; `refs` cannot see it, so a frozenset): `Map::calc_bonus`, `Map::calc_bonus2`. |
| 00692f10 | Map::calc_bonus | layout | - | dead | Map vtable +0x50; the same lattice selector on `common_factor` (region record +0) with multipliers 61 (x block) and 17 (y block). Same search, same result: no dispatch site. Census: `UNDISPATCHED`. |
| 00697540 | MapEastIndies::make_continents | layout | run1016,1019,1021,1062,1176,1179 | twin | `crates/sim/src/map_east_indies.rs:68` `east_indies_make_continents` (doc names `@00697540`; the rubric misses it only because the fn is not named `make_continents`). Compared: `avoid_continent = max(4, scale_land_area(·))`, `A = scale_land_area(250)` (decompile lines 63–64, 96); the angle `(r & 3)·0x3fffffff + 0x1fffffff` under four players, `(r % 0xffff)·0x10000 + r' % 0xffff` from four (lines 109–122; port lines 99–106, `r ≥ 0` so `& 3` = `% 4`); the radius `m/2` or `(m/2)·7/8` at three or more (lines 115–126, the 3-player `goto` into the 7/8 arm; port 101, 105); the further-island count `xs/12 − players + 8 + (r & 1)` (line 218; port 133); the island grow to `A/3` then `A` within `first_radius/2`, a stall restarting the generation. Agree, and diff-backed at `Mapgen` 7807 on every East Indies setup trace (MAPGEN §5, §15 step 1). Inherited, not its own: a lobby over the map size's allowance makes the port return `None` where the original takes `scale_land_area`'s float path, which `map-1` counts once, as `scale_land_area`'s `twin-differs`. |
| 006a06a0 | Map::set_file | layout | - | replaced | Map vtable +0x14: `file = <string-table prefix> + name`, the style file's path for `init_map_data`. Plumbing of the XML loaders (replaced by design): the data layer opens `mapstyles/<style_file>` itself (`crates/rondata/src/mapgen.rs:207`, `:249`). `refs`: pointer only, 22 vtables. |
| 006a08d0 | Map::scale_by_axis | layout | run1016..1021 | replaced | `n < 1 → 0`, else `max(1, (std/2 + xs·n) / std)`, `std` the Standard size's dimension (`map_sizes` data +0x144), `n` in `ecx`. Callers (`refs`): `Map::scale_number+0x19d/+0x1d0/+0x28f` only, the `SCALE` suffix of a style value — so it belongs to the XML loaders, as `map-1` ruled `scale_number`. Its twin is the `by` closure in `crates/rondata/src/mapgen.rs:351–357` with `"SCALE" => by(v, xs, standard)` (line 360): same `< 1 → 0`, same rounding, same floor of 1; agree. Diff-backed through every scaled style value the 7807–7927 checkpoints consume. |

## Summary

**Counts (14 rows).** dead 6 · twin 2 · replaced 2 · unmodelled 2 ·
absorbed 1 · ctw 1.

Every row a lockstep run entered is a `twin`, `absorbed`, `replaced`, or
`Map::make`, whose one unmodelled write is inert on every lockstep style. Two
of the `layout` rows are really ported and the rubric misses them by name:
`MapEastIndies::make_continents` (its twin is
`east_indies_make_continents`, with the address in the doc) and
`Map::get_player_overage` (split between `rondata::mapgen::overage_allowance`
and `sim::map_lakes::scale_land_area`). Three more have their twin in the
data layer: `add_good`, `scale_by_axis` and `set_file`.

**Proposed census marks.**
- **`UNCALLED`**, the frozenset `map-1` proposes, gets four more from this
  group, each `refs`-dead with 0xcc before the entry:
  `ResourceDivvyPool::add_water`, `add_late`, `add_early` (its one
  "pointer" is a byte run in `.rsrc`) and `Map::get_player_underage`.
  Combined with `map-1`'s, the regex reads
  `^(MapFairness::(recompute|(un)?apply_distances|point_lowers_highest|enable_team_mode)|ResourceDivvyPool::(refresh|add)_(water|late|early)|Map::get_player_underage)$`.
- **`UNDISPATCHED`** (new, a frozenset): a virtual whose vtable slot no
  site in the export calls. `refs` reports these as "pointer only", so it
  cannot catch them. Here: `Map::calc_bonus` (+0x50) and `Map::calc_bonus2`
  (+0x54), the lattice bonus selectors on the regions' `common_factor` and
  `goody_factor`.
- **`^MapConquest::`** (`map-1`'s ctw class mark) catches
  `check_sea_map@0068d1a0` here.

**Proposed replaced groups.** None new. `Map::set_file` and
`Map::scale_by_axis` join the XML loaders, as `map-1` placed
`Map::scale_number`.

**`unmodelled`, ranked.**
1. **`Map::make@0068bc90`, entered by every lockstep run, and its
   style territory limits are a playing game's rule the crate does not
   have.** `make` copies the style's `player_territory_limit{,_civic,_city}`
   and `colonized_territory_limit{,_civic,_city}` to `world+0x38..+0x4c`.
   `World::compute_reg_territory@006b0bb0` reads them, the player triple
   where `Region::flags & 4` and the colonized triple elsewhere.
   `crates/sim/src/territory.rs:167–175` uses the rules constants
   (44/4/4) for every region. It is inert on every lockstep style,
   because those styles leave all six limits at the constants. The disk
   already proves the difference: the start dumps of run1098 (Nile Delta)
   and run1126 (Atlantic Sea Power) read `player_territory_limit 48 / 8 / 8`
   with `colonized 44 / 4 / 4`, and run1123 (New World) reads
   `colonized_territory_limit 32`. It would show as `WORLD` (detail ≥ 5)
   cell ownership after the first border pass of a lockstep capture on
   one of those three styles, and earlier, without a capture, as the
   `WORLD` scalars against the `Tuning` constants on the three start
   dumps. `docs/ATTRITION.md`'s `WorldData` row says only "Seeded from
   `TERRITORY_LIMIT_*`", so this is new to the docs.
2. `Map::is_near_ocean@0068b0a0`: not entered. It is the coast-ring test
   used by start placement in British Isles, East Meets West and
   Mediterranean. It would show in the seed and part 2 at `Mapgen` 7807 on
   a setup trace of one of those styles.

## Not established

- **Which regions carry `Region::flags & 4`.** This is the open question in
  `docs/ATTRITION.md`, and it decides where the colonized triple applies.
  On Nile Delta and Atlantic Sea Power the two triples differ (48/8/8
  against 44/4/4), so a region without the flag is bounded by the
  constants and one with it by 48. A port that took only the player triple
  from the dump would still be wrong there.
- **Whether any lockstep pair or held-out game is booked on Nile Delta,
  Atlantic Sea Power or New World.** `crates/rondata/src/diff` names none
  of them. Whether a queued item reaches them is the queue's to say.
- `Map::calc_bonus`/`calc_bonus2` are called `dead` from a textual search
  of the export's dispatch sites. A dispatch the decompiler printed in some
  other form (a stored function pointer, or a `this` recovered through a
  cast) would not have matched. A trace's coverage of either address
  would settle it, and the TSV shows no lockstep entry.
- The `GoodType` age method (vtable +0xe8) that `add_good` falls back on
  was not read, because `vtables.txt` does not carry that table. The
  port's `PREQ0` reading is backed by the 7927 diff, not by that body.
