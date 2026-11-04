# Map generation

How the original builds a random map before frame 0:
`Map::make@0068bc90`, the `Terrain::init@00850f70` that runs after it, and
everything they call. Today the simulation does not generate a map. It
**loads** one from the original's own start dump: `rondata::gamelog::world_cells`
and `world_tiles` read the `WORLD=6` block, `Log::terrain_heights` reads the
`DUMP_ALL` height grid, and `rondata::diff::setup` builds `sim::world::World`
from them (`docs/DATALAYER.md` §3). A sim that cannot start a game without
the original's dump is not a full sim (Loop 3730). This document is the
reading a generator would be written from, and the falsifiers that would
check it.

**How this was established.** Item 3734, a reading lane (Opus 5.5). Four
readers each took one phase of the call tree in the decompile export
(`~/ghidra-projects/decomp`) and checked doubtful locals against the
listing. I read `Map::make` and the checksum machinery myself and checked
what the disk could answer: the setup checksum trace and the start dumps
under the archive's `Logs` directory. The readers' per-function notes are
kept outside the repo. Nothing here is transcribed; the formulas are given
at the level a port needs.

**Confidence.** The phase order, the checkpoint lines and the draw budget
per phase are **measured**: they come from the original's own checksum
trace and are exact. **§3's falsifier is a diff**: the checksum's cell part
equals an Adler-32 over the dumped cells on 337 of 337 files (496 of 496
since item 3776 pinned it, §3). Which
function draws, and how often, is a reading. It is checked as a total
against the measured budget in two places (§4's 3 draws and §9's 3,600 on
the all-land maps). Elsewhere it is consistent in kind but not reconciled
to a number, because the count depends on the grid at that moment. Every
other claim is reading-only (§13).

**What this is not.** It is not a whole port: its first two steps, the
checksum reader with Great Sahara's mask (item 3776) and the zero-draw
passes on Great Sahara (item 3798), are built (§15), and the rest of the
chain is parked. It does not cover scenario maps (`MapScenario::make`
overrides +0x64), Conquer the World's `MapConquest`, or the editor.

## 1. Entry: who calls it, with what

`Setup::build_game@005ac190` makes one draw for `world->seed`
(`rand(0, 0xffff) % 0xffff + 1`, checkpoint `setup.cpp` 716). It then builds
the map object through `Map::new_map@006a0950`, whose class is chosen by
map style. It calls vtable +0x64 (`Map::make`) with three arguments: the
nation count, `-1`, and the splash flag. The `-1` means `make` never
re-seeds the stream; a non-negative value would set both `world->seed` and
`game_random`'s state. After `make` returns, `build_game` calls
`Terrain::init@00850f70` (checkpoints `setup.cpp` 745 → 759), and later
`Terrain::mark_forest@00850d40`.

The per-map classes override exactly one slot, **+0x40 `make_continents`**.
`vtables.txt` shows no `Map*` vftable replacing any other slot except
`MapScenario`'s +0x64. So the three corpus maps share every phase of this
document except §5's: Great Lakes (style 14), East Indies (18) and Great
Sahara (7).

The map's data is two XML files read the same way: `mapstyles\default.xml`
first, then the style's own file, which overrides it (§4).

## 2. The phases and their draws

`Map::make`'s `say_checksum` calls, at detail 1, bracket its phases. The
`map.cpp` line numbers are the immediates in the body. The draws between
two checkpoints come from walking the LCG between the printed seeds
(`tools/gamelog/rngtrace.py`).

| checkpoints | calls between them | GL | EI | EI (fuzz) | GS 2p | GS 3p | Him | BI |
|---|---|---|---|---|---|---|---|---|
| setup 716 → 7805 | `load_map_data`; `base_edge` | 3 | | | | | | |
| 7805 → 7807 | `make_continents` (+0x40) | 5,568 | 49,611 | 55,533 | 4 | 4 | 4 | 19,644 |
| 7807 → 7854 | `Regions::clear_all`/`find_all`, `fix_diag_land`, `make_coastlines`, `fill_fertile` | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 7854 → 7859 | `TerrainGroups::place_all` | 1,016 | 2,119 | 2,102 | 1,238 | 1,324 | 84,014 | 1,744 |
| 7859 → 7861 | `check_player_forest` | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 7861 → 7865 | `nubify_forest` | 3 | 0 | 0 | 5 | 6 | 124 | 0 |
| 7865 → 7870 | `change_*_base`, `fix_transitions` | 1,824 | 1,276 | 1,123 | 1,004 | 1,259 | 3,175 | 1,337 |
| 7880 → 7927 | the MAPMAKE log, `place_resources` | 231 | 89 | 90 | 65 | 94 | 57 | 101 |
| 7927 → 7937 | `adjust_for_blocking`, regions again | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 7937 → 7945 | `World::compute_all_vals` | 2,920 | 1,479 | 1,403 | 3,600 | 3,600 | 3,600 | 1,975 |
| map 5014 → 5076 (in `Terrain::init`) | `make_rivers` | 173 | 0 | 0 | 0 | 0 | 0 | 0 |

The columns are files: GL `run100` (and run11, the same lobby), EI `run1016`,
EI fuzz `fuzz-424242-early` (a different lobby seed), GS 2p `run1000`, GS 3p
`run1412`, Him `run348`, BI `run316`. The first row was read off run11
only; the other files' traces carry it too.

**The corpus is narrow.** 493 files carry the `map.cpp` records with the
seed (all at `check_all_level ≥ 14`, sixteen `World` lines a record).
Grouped by (style, size, the seed at 7805, the final cell checksum), they
hold **seven** distinct generator runs: the six columns above, plus GS
split by player count. Every lobby but the fuzz run enters `Map::make` on
`0xa289127d`, because every lobby seed is 12345. 414 files carry a start
`WORLD` block within their first 12,000 lines (417 by a looser grep); 337 carry both. The queue's figure of 956 start dumps counts
a wider tree than this directory. Fourteen further styles have one
click-free start dump each and no checksum trace (`run1098`..`run1128`): a
falsifier for the final grid only.

**Great Sahara, 2 against 3 players.** The two lobbies draw the same 4 in
`make_continents` and agree on the cell checksum at 7807. Only part 2, the
start arrays, differs. They part from `place_all` on, in both draws and
cells. §5 and §7 say why.

## 3. The falsifier: the checksum's cell part is an Adler-32 of the dump

`GameLog::say_checksum@00930b30` prints its subsystem lines through a
`CheckSum` walker. `CheckSum::walk_function@00936ff0` feeds each walked
byte range `[begin, end)` into a running **Adler-32** (`accum` starts at 1).
The tell is in the trace itself: an empty array prints 1, and N zero bytes
print `N·65536 + 1`.

`World::walk_data@006b5cf0` takes a part number. `say_checksum` prints
parts 0..14 and then −1 (all), so a record's sixteen `World` lines are:

| part | bytes walked (`World` fields, `types.txt`) |
|---|---|
| 0 | nothing (always 1) |
| 1 | the header, `xs` … (from +0x8) |
| 2 | `start_x`, `start_y`, `start_city_x`, `start_city_y` |
| 3 | `oil_x`, `oil_y` |
| 4 | the scalars `size` … `seed` (+0x8 .. +0x80) |
| 5 | **every cell's `WData` bytes 0x00..0x14**: `flags`, `land`, `land_sub`, `region`, `region2`, `down`, `down_who`, `val`, `goods`, `light`, `who`, `who2`, `blocked`, `bad`, `solid`, `was_seen` |
| 6 | `tdata` masks (2 bytes a tile), then `seen`, `seen2`, `seen3` |
| 7 | `wcoord_seen` |
| 8 | `danger[8]` |
| 9 | each cell's `CollBlock`, if any |
| 10–13 | four `MiscAccess::terrain` arrays, one `Array<WCoordData>` and three `SimpleArray<int>` |
| 15 | all of the above |

**Diff-backed (2026-10-28).** I packed each start-dump cell's sixteen
fields into the `WData` byte layout, with `land` as its `land_key` index
(`BASELAND`, `SANDY`, `OCEAN`, `NONE`), and took one Adler-32 over all
3,600 cells. It equals the trace's part 5 at `setup.cpp` 1240 and at every
record after it, through `game.cpp` 5024, on **337 of 337** files with
both. **Pinned since item 3776** as
`rondata::mapgen::tests::the_cell_part_is_the_packed_start_dump`, on
**496 of 496** files on 2026-11-03 (the archive has grown; `floors::MAPGEN`
holds the count as a floor, and any file that parts fails it). The reader is `rondata::mapgen`: `start_head` cuts a
file just past its start `WORLD` block, `Log::parse` and `world_cells` read
it, and `sim::mapgen::WCell::walked` packs a cell. The walked range is
`[cell, cell + 0x15)`, `flags` through `was_seen`, read off `walk_data`'s
part-5 loop; moving one byte of the packing parts all 496. So every phase's cell writes are checkable **at that phase's own
checkpoint** to the byte. No intermediate dump is needed: run the port's
phase, pack its cells, compare the Adler-32. The seed delta checks its
draws, and part 2 its start positions. A port's whole map maker therefore
has 14 checkpoints of falsifier on every traced lobby.

The parts each phase moves (run100, GL): `make_continents` 2 and 5; each
zero-draw pass 5; `place_all` 5 and 6; `nubify_forest` **none** (§7);
`change_*_base` 5; `place_resources` 5 and 6; `adjust_for_blocking` 5 and 6;
`compute_all_vals` 5. After 7945, `Map::make`'s tail moves part 4 at
`setup.cpp` 734 (`sea_map`, §10, on every traced file), and on run100
`Terrain::init` moves 5, 6 and 10–12 at `Terrain.cpp` 720 and 5 at 738
before `map.cpp` 5012; on Great Sahara it moves none (§10, item 3809).

**Not settled.** Parts 10–13 name `MiscAccess::terrain` offsets 0x40 past
the PDB's `Terrain` record (`halfland_locs` +0x4b40, `halfland_types`,
`halfland_subtypes`, `nuke_hits`). The shapes match: one `WCoordList` and
three `SimpleArray<int>`. `Terrain::mark_halfwater@0084ece0` writes all
three of the first, which fits parts 10–12 moving in `Terrain::init`. The
0x40 is unexplained. Its falsifier is the same Adler-32 taken over the
dump's half-land lists, which no reader of ours parses yet.

## 4. Load: the map data and the first three draws

`Map::load_map_data@0069dad0` calls `Map::init_map_data@0069f250` twice,
once for `default.xml` and once for the style's file. It makes no draw of
its own. Each `init_map_data` makes **one draw when the file's `TILECHANCE`
chances sum to T ≥ 2**. It takes the first tileset whose running total is
at least `r % T`; because the test is `≤`, the first entry wins one extra
value. If the chosen name is not an available tileset, it makes a second
draw, `available[r % count]`. Then `Map::make` makes one draw for
`base_edge = r % 4` when `BASE_EDGE` is −1. That is default.xml's value, and
none of the three corpus files overrides it.

**1 + 1 + 1 = 3, the measured count** (run11, GL). The style file's draw
is therefore the 7805 seed stepped back once, and a reader recovers the
tileset from it (`rondata::mapgen::tileset_before`): on Great Lakes'
generator run, 2,332,836,374 gives `r` 16,917 and `r % 100` 17, so
**`early_autumn`** (CLUMP_FACTOR 2, baseland 40/30/20/10). Its 7854
agreement backs the choice (§6), and the choice forced to `jungle` parts
there (item 3819). The same holds for
SAHARA.xml and EASTINDIES.xml. `Map::init_resources@0069e990`,
`Map::init_rivers@0069e520` and `TerrainGroups::init_terrain_data@006a6540`
also run here, and none of them draws.

`Map::scale_number@006a0320` reads every count the XML gives:
- `AREA`, `SCALE`, `SCALEWATER` and `SCALELAND` scale the value; any other
  word leaves it as written.
- A negative value is per nation: it is multiplied by `make`'s first
  argument. That is one way the player count enters the groups (§7).
- `SCALEWATER` and `SCALELAND` are the only float arithmetic (§11). No stock
  `mapstyles` file uses them.

Falsifier: the seed at 7805 (the three draws), and part 4 at 7805.

## 5. `make_continents` — the land/sea mask

**Base, `Map::make_continents@00694ab0`** (no override in the corpus calls
it; no trace enters it). It builds four `Fractal`s on private `Random`s
seeded from `World.seed` × 19, 29, 31 and 37, with smoothing 3, 4, 2 and 4.
It mixes B and C into A. A cell is water (`land` 2) where A's height is
below 55, land (0) otherwise; D is never read. **No `game_random` draw.**

**Great Sahara, `MapGreatSahara::make_continents@0069bf70`: exactly 4
draws.** Two make the angle (`(r % 0xffff) << 16 + r % 0xffff`) and two
jitter the centre by `r & 1` each. `World::wipe@006b2c00` and `invert_land`
then make every cell land 0, region 0. The starts sit on a ring of radius
`min(m/2 − 4, (m/2)·80/100)`, stepped by `0xffffffff / players`. Amazon,
Himalayas, Old World and Southwest Mesa read the same. **This is why GS's
mask is independent of the player count**: only the angular step reads
it. `World::add_starting_location@006b2de0` writes the start arrays and
the start-city bits, never a `WData`, so at 7807 only part 2 can differ —
as measured.

**Diff-backed since item 3776** (`sim::mapgen::great_sahara_make_continents`,
checked by `rondata::mapgen::tests::great_sahara_s_mask_is_the_traced_7807`):
from the seed at 7805, the port lands on the seed, part 2 and part 5 at
7807 on all 105 Great Sahara files that carry both checkpoints, which are
the two generator runs of §2. What the listing settled that the reading
left open:
- The draw order is the angle's high half, its low half, then the centre's
  x jitter, then its y. The first point is at `angle + step`, not at
  `angle`. The centre stays in the callee-saved registers across the
  loop, so the points do not chain. Reordering the angle's two draws parts
  run1000's part 2.
- `project@0092d000` folds the angle as `crate::army::step_along` does:
  `x + sin·r`, `y − cos·r`, with r the ring radius in cells.
- The wiped cell is `land` 2, `region` 0x40, `down`, `who` and `who2` −1,
  and every other field 0. `Regions::clear_all@00680060` and
  `invert_land` then zero `region`, and `invert_land`'s 16-bit store at +2
  zeroes `land_sub` with `land`. 3,600 copies of that cell are 7807's
  part 5, 1,267,075,849.
- `add_starting_location` pushes x and y once each. It pushes the city's
  footprint as `(x, x−1, x, x−1)` and `(y, y, y−1, y−1)`.
- Part 2 walks each `SimpleArray<WCoord>` as
  `SimpleArray<WCoord>::walk_data@0047c660` does. An empty array walks its
  4-byte `num` alone, which is why 7805 prints 1,048,577. A full one walks
  `num`, the capacity, the 2-byte increment at +0xc, the flag byte with
  0x40 cleared, and then the entries. `increase_size@00434630` doubles on
  a negative increment, starting from 4. **The increment, −1, is measured,
  not read**: I tried every increment from −1 to 64 and every flag byte.
  Only −1 with flag byte 0 gives 7807's part 2 on both lobbies. The
  constructor that sets it was not read.
- The function also writes `Map::start_dist` (+0x28) as `max(4, xs/16)`.
  No `World` part walks `Map`, so nothing here checks that write; the port
  returns the value.

**Great Lakes, `MapGreatLakes::make_continents@00699e40`: the 5,568.**
The lakes are grown as *land* regions on a wiped, all-water world, and the
world is inverted at the end, so a lake ends as water and the background
as land.
- Three opening draws: two for the angle, one for the lake count
  `xs/12 + (r & 1)`. Before them `avoid_continent` becomes
  `max(6, scale_land_area(avoid_continent))`; `scale_land_area` returns
  its argument unless the nations pass `Map::get_player_overage`'s
  allowance for the map size (2, 2, 3, 4, 6, 8), where it divides in
  floats. No corpus lobby passes it; the port refuses one that would.
- Each lake centre is two draws, `(r % xs, r % ys)`, retried until
  `Map::land_dist@0069d970` (the first ring round a water cell holding a
  non-water or off-grid cell) reaches
  `max(3, (avoid_continent + 1)/2 + isqrt(size·7/22))`. The root is the
  function's own integer Newton loop from `n/2`. After 1,000 misses the
  lake shrinks to `small`, then `small` by 13/16 while it stays over 19.
  A hundred attempts in a row that open no lake end the loop.
- The first lake is `max(90, cells/20)` cells. **After every attempt**,
  opened or not, the next size is `small + r % (first − small)`, one draw,
  with `small = max(24, cells/40)`.
- A centre that `Map::grow_valid@0069d000` passes opens a region
  (`Map::make_region@0069d3f0`), grown by `Map::grow_region@0069c600` to
  the lake's size, no further than `xs` from its first cell. Region ids
  count up from 1.
- `grow_valid` refuses, with no draw, when another region is live and the
  point is within `avoid_center` of the **map's** centre. It also refuses
  when another region lies in the disc of `avoid_continent` round the
  point. Only then does it make one draw per non-zero `avoid_edge`, in
  slot order. Each draw nudges a persistent `avoid_modify` (+0x64) by
  `r % 3 − 1`, clamped to 0..4, and tests edge `(base_edge + slot) & 3`
  at `avoid_edge + avoid_modify`. The first edge that fails ends the
  walk. All four GL edges are `8 SCALE`, so a full pass costs 4.
  `base_edge` is `Map::make`'s `r % 4`, recovered from the seed at 7805.
- `grow_region` first gives a one-cell region the disc to ring `k`, the
  first `k ≥ 3` with `3k² ≥ size/4` (else 2). It takes each cell
  `grow_valid` passes, and each one is tested twice, since
  `Map::point@0069d5a0` asks again. Each later step picks one of the
  region's cells. Past `scale_by_area(512)` cells (512 on a 60×60), one
  step in `count/8` (a draw) picks in the newest eighth (a second draw)
  if that cell is within the distance. Otherwise, while ring `k` is
  unfilled, it picks any cell (one draw), else a cell of ring `k` onward
  (one draw). Within 64 of the size the pick takes a cross of `point`s;
  before that it takes `Map::large_stamp@0069cb20`, a stamp and then one
  draw for one of nine shapes (two more in shape 4). A step that grows
  nothing counts: `8·size` of them in all, or `size` in a row, ends it.
  `point` returns false only on a `grow_valid` refusal. A cell off the
  grid or already taken is a success, which is what the stamps'
  short-circuits read.
- Then the drawless tail:
  - `Map::eliminate_pools@0068b890` over the whole world: the regions are
    relabelled, and every water region but the largest is made land (the
    smallest first, the first of equals);
  - `invert_land`;
  - each start, on the angle stepped by `0xffffffff / players`, walked in
    from `3·xs/2` in steps of 3 from `(xs/2, xs/2)` (y also `xs/2`, from
    the listing) to the first land cell two in from every edge;
  - `Map::check_player_land@0068ef00`, called with `ecx = 1` and
    `edx = avoid_continent` (listing). It makes land of the disc to ring
    `ceil(r/4)` round each start-city cell, where `r` is unit type 349's
    `max_range`, 24 in the shipped rules, so ring 6. With every `region`
    0 after `invert_land`, its neighbour searches find nothing to adopt
    or protect.

**Diff-backed since item 3815** (`sim::map_lakes::great_lakes_make_continents`;
`rondata::mapgen::tests::great_lakes_s_mask_is_the_traced_stream` and
`…_traced_7807`):
- **Reconciled to 5,568 draw for draw**, site and word, on all 222 Great
  Lakes traces.
- The seed, part 2 and part 5 at 7807 agree on all 185 Great Lakes files
  carrying both, first try.
- All of them are one generator run: 2 nations, 60×60, seed 2726892157.
  Eleven more files carry the checkpoints but no start grid, so they give
  no size (3815a).
- Fail first: dropping `avoid_modify`'s upper clamp parts every trace at
  draw 419. Dropping the spacing's root parts them at draw 5. Shrinking
  `check_player_land`'s ring parts part 5 alone.
- Removing the first lake's `max(90)` fails nothing, because `cells/20` is
  180 here.
- Skipping `eliminate_pools` fails nothing, because this run's background
  is one region. Its merge is unit-tested only (3815b).

**East Indies, `MapEastIndies::make_continents@00697540`.** The islands
are *land* regions grown on the wiped water, as GL's lakes are, but never
inverted, so each island's region number stands in its cells at 7807.
- `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`,
  once. `A = Map::scale_land_area@0068b1e0(250)`; both refuse the float
  path as GL's does.
- Each generation runs `World::wipe` and `Regions::clear_all`. `wipe`
  zeroes the four start arrays' counts (+0x84, +0xa0, +0xbc, +0xd8), so a
  restart starts them empty. `avoid_modify` is a `Map` field, so it carries
  across a restart.
- The angle is one draw under four players: `(r % 4)·0x3fffffff +
  0x1fffffff`, a quadrant's middle. With four or more it is two, the high
  half and then the low half, as GS's. The ring's radius is `m/2`, or
  `(m/2)·7/8` with three or more players, where `m` is the shorter side.
- Each player's point is `project` about `(xs/2, ys/2)` (ecx and edx at
  `006977be`, read off the listing) at the angle stepped by `0xffffffff /
  players`; the first point is a step past the drawn angle. While the
  point is within `max(avoid_edge) + 4` of any edge (`x ≤ e`, `xs − x ≤ e`,
  and the same for y), the radius falls by one. It stays fallen for the
  players after. The point is the start (`add_starting_location`) and
  opens region `i`. The region record's flags take `|= 2`, which no grower
  tests, since `grow_valid` and `point` read bit 1 only.
- Two `grow_region` passes follow: every island to `A/3`, then every
  island to `A`. Both use `max_dist` equal to half the **first** radius,
  not the fallen one (`-0x10(%ebp)` at `0069788c` and `006978df`).
- **A pass that ends short restarts the generation**, angle draws and all
  (`jne 006979c4` and `jne 00697a71`, both back to `00697590`). Short means
  `grow_region` returned non-zero with `param_6` 0, which happens on an
  empty record or on either stall. Nothing caps the restarts: a lobby that
  always stalls never ends.
- Then one draw sets the further islands' count,
  `xs/12 − players + 8 + (r & 1)`. They are placed exactly as GL's lakes
  are, with the same sizes, spacing, 1,000-miss shrink and hundred-fail
  stop, grown to `xs`, and numbered on from `players + 1`. Their record
  flags are 5, 5 and +0xc = 1, which no grower reads. The function ends
  there, with no `eliminate_pools`, invert or `check_player_land`.
- **Player count changes the mask here**, unlike GS and GL: it changes the
  ring's step, the radius and the further islands' count.

**Diff-backed since item 3820** (`sim::map_east_indies::east_indies_make_continents`;
`rondata::mapgen::tests::east_indies_s_mask_is_the_traced_stream` and
`…_traced_7807`):
- **Draw for draw**, site and word, on all 418 East Indies traces. A trace
  counts as East Indies when its run has a `MAP_STYLE` 18 file, or when its
  gamelog's name says `eastindies`; run1016's name does not say it. The
  ported totals are §2's measured **49,611** and the fuzz lobby's
  **55,533**.
- The seed, part 2 and part 5 at 7807 agree on all 202 East Indies files
  carrying both, first try. Two generator runs: seed 2726892157 on 200
  files and 2426541702 (the fuzz lobby) on 2. Both are 2 nations on 60×60,
  `MAP_SIZE` 2. Four more files (run21, run23, run24, run28) carry no start
  grid.
- **The restart never fires on the corpus**, which kills the booking's
  reading that restarts made the 49,611. The two players' islands reach
  `A` on the first generation of both runs. The draws are the ten further
  islands' growth, each up to 180 cells on the open water, against GL's
  lakes capped by one another. So the restart, the three-player radius and
  the four-player angle are unit-tested only (3820a).

The other overrides were only skimmed. `MapWarringStates` alone calls
`MapFairness::calc_distances@0068a1c0` (float, §11).
`Map::force_continent_spacing@0068e6a0` has no caller in the export.

Falsifier: seed and parts 2 and 5 at 7807, per style. All seven runs in §2
check it, and the fourteen single-dump styles check the final grid only.

## 6. The zero-draw passes: regions, diagonals, coasts, fertility

All draw nothing, as measured (7807 → 7854, and 7927 → 7937 for the
region pair). `Map::make` prints a checkpoint after each group. 7835 comes
after `Regions::clear_all`, `find_all` and `fix_diag_land`. 7843 comes after
`make_coastlines`. 7846 comes after the region pair again. 7854 comes after
`fill_fertile`.

- `Regions::clear_all@00680060` resets the 128 region records (each count
  at +0x14 to 0), zeroes every cell's `region`, and sets `land` 0 and `sea`
  0x40. It does **not** zero `region2`.
- `Regions::find_all@0067eff0` labels 8-connected components. Water is
  judged by `WorldData::is_ocean@006b4830` (`!(flags & 0x100) && land ∈
  {1, 2}`), so a HALFLAND cell counts as land. It scans y outer, x inner. An
  unlabelled cell first tries to **adopt**: it takes the label of the first
  of its `move_x/move_y[1..=8]` neighbours that has the same wetness and a
  label other than 0 or 0x7f. Otherwise it opens a region:
  - land takes the first record from 1 whose count is 0, clamped to 0x3e;
  - sea increments the `sea` counter, so the first sea region is **0x41**,
    clamped to 0x7d.
  `Regions::find@00680180` floods the new region breadth-first over the
  eight neighbours, and the record's count is the cells it dequeues. A
  region that lands on its clamp (0x3e or 0x7d) evicts. The smallest record
  in `[lo, clamp)` by count (`lo` 0 on land, 0x40 at sea), if any is
  smaller, moves into the bin after the clamp (0x3f, 0x7e), and the new
  region takes its number; with none smaller, the new region goes to the
  bin. On land `lo` is record 0, which never counts, so it is always the one
  taken, and its relabel moves every still-unlabelled cell into 0x3f. That
  is the original's arithmetic, and no corpus grid reaches 62 land regions.
  A second pass, x outer and y inner, sets `region2` on HALFLAND cells where
  it is 0, to the first ocean neighbour's `region`, so a stale `region2`
  survives a re-run. Then the coastal sets, `sort_regions` (rank only) and
  `rebuild_coords`, the order `compute_sites` samples (`docs/AI.md` §13).
  None of the three writes a cell.
- `Map::fix_diag_land@0069c250` scans x outer, y inner. A `BASELAND` cell
  that meets another `BASELAND` cell across a corner (`CORNER_X/CORNER_Y`,
  `crate::world`), while both cells flanking that corner are on the grid and
  not `BASELAND`, becomes water: `land` 2, by the 16-bit store that also
  zeroes `land_sub`. Only the (+1, ±1) corners can fire. A partner across a
  (−1, ±1) corner is earlier in the scan, with the same two flanks, so it
  has already broken.
- `Map::make_coastlines@006947a0` starts with `Map::fix_lakes@0069c470`.
  Over the interior, y outer and x inner, an ocean cell with **more than 5**
  non-ocean neighbours becomes `BASELAND`. The sweep repeats in place until
  nothing changes. Then, y outer and x inner, a cell that is not ocean and
  not yet COAST (4), with a neighbour that is ocean and not COAST, takes
  `(flags & 0xffc3) | 4 | 0x400`. Then the thinning, in the same order: a
  COAST cell with no neighbour that is land without COAST goes to `land` 2
  and `flags & 0xfac3`. On GS (all land) it moves nothing; part 5 is equal
  across 7835 → 7843.
- `TerrainGroups::fill_fertile@006a6f90` scans x outer, y inner. It sets
  each `BASELAND` cell's `land_sub` to the number of leading partitions at
  or below the cell's byte of the tileset `Fractal`. The `Fractal` and the
  partitions are built in load, by `TerrainGroups::init_tileset_data@006a61f0`:
  - **The `Fractal`**, `Fractal::init@006aa2d0` on `xs × ys`. It is
    smoothed by the tileset's `CLUMP_FACTOR` (clamped to 0..5), flag 0, and
    seeded with `World.seed`, the `seed` the start dump's `WORLD` block
    prints (7236 on every Great Sahara file), on the fractal's **own**
    `Random`, so `game_random` does not move. It is an integer midpoint
    displacement into `xs + 1` columns of `ys + 1` bytes:
    - The top level draws each `2^smooth` lattice point `get(0, 0xff)`.
    - Each finer level `i`, x outer and y inner, fills the points that have
      an odd coordinate at that spacing. The value is the rounded mean of the
      two edge parents (`+1 >> 1`) or of the four diagonal parents
      (`+2 >> 2`), plus `get(0, 2^(8−smooth+i) − 1) − 2^(7−smooth+i)`,
      clamped to a byte.
    - Column `xs` copies column 0 before each level and after the last.
    - Its doubles, `x_inc` and `y_inc`, are `Fractal::get_height`'s only, so
      the bytes take no float.
  - **The partitions** are the running sum of `(int)(f/100.0f·255.0f) &
    0xff` over all but the last of the tileset's `BASELAND` frequencies
    (`mapstyles/default.xml`'s `TILESET_DATA`, `LANDKEY name="baseland"`,
    `frequency_0..`; one per `BASE` under the tileset's `BASELAND` in
    `Data/tilesets.xml`). The style file's own entry is read over it if it
    has one, and a frequency missing there becomes −1. The product is
    `f·255/100` truncated for every |f| < 6,420
    (`the_partition_float_is_the_integer_quotient`; it first parts at
    ±6,420). GS_Desert's 80/5/5/10 give 204, 216, 228.

**Diff-backed since item 3798** (`sim::mapgen`; checked by
`rondata::mapgen::tests::great_sahara_s_zero_draw_passes_are_the_traced_7835_to_7854`):
from the ported 7807 grid, the seed, part 2 and part 5 agree at 7835, 7843,
7846 and 7854 on all **105 Great Sahara files**, which are §2's two
generator runs, on the first run. 7835 and 7846 are every cell `region` 1
(2,334,202,137). 7854 is the fertility map over the `Fractal`
(969,480,808), which checks every byte's bucket on 3,600 cells. A second
check covers the labels across styles: **`find_all` re-labels 494 of 496 start grids**
whole (`find_all_relabels_every_start_grid`: GS 105, GL 185, EI 202, and
styles 9 and 22). A start dump's `region` is the 7937 pair's, so this checks
the label order, the sea numbering from 0x41 and the adoption on real
multi-region maps. The two that part are the fuzz lobby's East Indies,
named (parked 3798b): a half-land row that was water at 7937 moves an island
ahead in the scan, and `Terrain::mark_halfwater@0084ece0` writes those
cells' `region` later.

**The water arms, diff-backed since item 3819** (checked by
`rondata::mapgen::tests::great_lakes_s_zero_draw_passes_are_the_traced_7835_to_7854`):
from Great Lakes' ported 7807 grid (§5), the seed, part 2 and part 5 agree
at 7835, 7843, 7846 and 7854 on all **185 Great Lakes files**, one
generator run, on the first run, with the `early_autumn` tileset §4's
draw picks. Each water arm's mutant parts on that walk: `fix_diag_land`
skipped and the first sea label moved to 0x42 part at 7835; `fix_lakes`
at `> 6`, `fix_lakes` skipped, the COAST marking skipped and the thinning
skipped part at 7843; the tileset forced to `jungle` parts at 7854. Two
arms stay **unit-tested only**, and neither can part at these
checkpoints on any map (parked 3819a). The `region2` pass writes nothing
here, because nothing in `Map::make` sets HALFLAND before `Terrain::init`
(§10), and a start dump's `region2` belongs to a later writer (185 GL
files part on two cells when it is zeroed first). The adoption arm is
equivalent after `clear_all`: a same-wetness neighbour with a label was
flooded by `find`, and the flood took this cell too.

**The archipelago's arms, diff-backed since item 3825** (checked by
`rondata::mapgen::tests::east_indies_s_zero_draw_passes_are_the_traced_7835_to_7854`,
which shares Great Lakes' walk): from East Indies' ported 7807 grid (§5),
the seed, part 2 and part 5 agree at 7835, 7843, 7846 and 7854 on all
**202 East Indies files**, both generator runs, on the first run. The
tilesets §4's draw picks are `jungle` (seed 2726892157) and `sw_desert`
(the fuzz lobby), both clump 3 and 80/5/5/10. `find_all` opens **12 land
regions and 3 seas** at 7835 on both runs. At 7846 it opens 12 land
regions and 1 sea, or 11 and 1 on the fuzz lobby: `fix_lakes` has filled
the two enclosed seas. Great Lakes opens 1 land region and 6 seas at both.
So **the land record search** (the first record from 1 whose count is 0)
has its first diff here. Its mutant, which never advances and puts every
island in record 1, parts East Indies and fails nothing on Great Lakes.
Each of 3819's water-arm mutants also parts East Indies: `fix_diag_land`
skipped, `fix_lakes` at `> 6`, the COAST marking skipped and the thinning
skipped. So do two more: `find_all`'s scan with x outer (all 202 files at
7835), and `clear_all` leaving the cells' `region`. Dropping the (−1, ±1)
corners from `fix_diag_land` fails nothing, as the paragraph above says it
cannot. No map reaches the clamps (0x3e land, 0x7d sea) or their eviction:
twelve islands are far from 62. The sea side is unit-tested from the
reading (`the_sea_clamp_evicts_into_0x40_then_bins`), and the land side
has no test (parked 3825a).

Falsifier: part 5 at 7835, 7843, 7846 and 7854.

## 7. Terrain groups: forest, mountains, rock, oil, cliffs

`TerrainGroups::place_all@006a70d0` (the 1,016) runs only when
`semaphore & 2` is clear.
1. `Mountains::randomize_mountains@0089ca70`, up to 3 draws.
2. A selection pass over the tileset's groups: one `r % 100` per group
   that opens a new grouping, taken on a cumulative-chance ladder, plus one
   draw for the clump count when `min < max`.
3. A placement pass: per clump, one draw for its size if it has a range,
   plus one for oil on type-6 groups. Then by pattern: **0** runs
   `TerrainGroup::place_player_group@006a4190` for every player and every
   clump; **1** every region larger than 5; **2/3** a weighted region list
   (float, §11) plus one start draw.
4. `TerrainGroups::add_doobers@006a1540`, then
   `TerrainGroups::treeify_mountains@006a1cc0` (one draw per mountain-edge
   cell; skipped on style 9).

The group types are named from what they write, not from an enum: 4
forest, 5 mountain, 6 rock with oil, 7 oil by water, 8 cliff.

`place_player_group` costs:
- one draw for its ring start;
- per growth step, one draw when the candidate list holds more than one,
  plus 2 for `TerrainGroup::randomize_orthogs@006a2320` per tile index
  tried, plus 4 more when the list holds three or more;
- one `r & 3` mirror when a later player's forest copies player 1's shape.

`TerrainGroup::place_region_group@006a2f60` makes one start draw, then
visits cells in the order of a 16-bit Galois LFSR: `(s >> 1) ^ 0xB400` on
an odd state, period 65,535. The listing XORs 0x16800 before the shift,
which is why it was first written here as 17-bit
(`the_region_walk_visits_every_state`).

**The groups** (`TerrainGroups::init_terrain_data@006a6540`). The style
file's `TERRAIN_GROUPS` replace `default.xml`'s, which is read only when the
style has none. Patterns map `player` 0, `continent` 1, `world` 2,
`nonplayer` 3 and `corner` 4; pattern 4 places nothing, because the jump
table at 0x6a9380 has four entries. Counts go through
`Map::scale_number@006a0320`. `SCALE` is `max(1, (35 + xs·v) / 70)`, where
70 is `rules.xml`'s Standard map size (`map_sizes` record 3). `AREA` is the
same over the area, and any other word leaves the value. Mountain sizes are
not scaled. `rondata::mapgen::terrain_groups` reads all eleven of Great
Sahara's groups (`great_sahara_s_groups_parse_as_read`). Two of them,
"general trees" and "sprinkling of oasis", are pattern 2. The mountain
templates are `rondata::mountains`' (§4 of its reader), and the cell list
`place_all` lays is their tiles' cells in first-appearance order.

**Ported on Great Sahara** (item 3803, `sim::terrain_groups`). Every draw
is tagged with the return address the original's draw-site trace records
for it. The port's stream is the trace's `place_all` segment, site for site
and word for word, on **all 280 Great Sahara traces**: 267 at 2 players
(1,238 draws) and 13 at 3 (1,324). Seed and parts 2, 5 and 6 agree at 7859
on all 105 files that carry it (`great_sahara_s_place_all_is_the_traced_
stream_and_7859`). It agreed on the first run. The 7859 grid holds forest,
mountain, rock and oil on both lobbies; the oil cells' `goods` byte stays 0,
because the OIL object `World::set_oil_at` makes walks no part. That object
is still made: `World::set_oil_at@006b2a10` kills any live oil good on its
cell on both arms (listing 0x6b2a10..0x6b2b05), and the on arm then calls
`Objects::init_good(OIL)`, which takes the first dead slot. So a later
good's index, the `down_who` §8 writes, counts `place_all`'s oil
(`MapGrid::goods`, item 3805). A moved
LFSR tap, a reordered ring draw, a fringe cell's lost FOREST bit and a
stray tile-mask bit each turn the check red (`tools/mutate.py`), the last
two at 7859 only.

**Where the player count enters** — the GS finding from this side. The
grid entering `place_all` is equal for 2 and 3 players. `place_all` is the
first reader of the starts and the count:
- pattern-0 groups draw per player;
- `place_player_group` rejects a cell nearer another start;
- `place_region_group` requires every start within its bounds;
- pattern 3 excludes regions holding a start;
- `Mountains::excluding_verify@00898570` measures against every start;
- the start-city bits block;
- §4's negative XML counts scale with the nations.

At a higher `CHECKSUM` detail, `place_all`'s inner checkpoints would place
the first difference (`TerrainGroups.cpp`): detail 5 at 2076, 2079, 2097,
2167, 2276/2547/2638 and 2753; detail 10 inside `place_player_group` (110,
335, 352, 383, 430, 474, 495, 736, 802); detail 20 at 777 and 786, and
1345 and 1383 in `place_oil_deposits`.

**Cliffs.** Type-8 groups reach `Cliffs::position_cliff@008a8bc0` (one
draw when two or more templates fit) and
`Cliffs::verify_defensive_position@008a8680`, the one libm transcendental
on a data path (§11). **No trace enters either**, and none enters
`Cliffs::place_cliff@008a7ea0`. As read, no corpus tileset places a cliff
group.

Then:
- `Map::check_player_forest@0068e8f0` draws nothing, but it can write.
  The decompile's list storage is broken, so it is read from the listing
  (0x68e8f0..0x68eeff). A start whose city disc holds fewer than three
  FOREST cells gets a patch. The disc is `circle_radius[0]` to
  `circle_radius[k]`, where `k = CITY_CENTER_RADIUS / 4 − 1`, which is 4.
  The patch is tried at sizes three, then two, then one. At each size the
  disc's cells are walked from `circle_radius[2]` out. A cell may grow when
  it is:
  - not forest or mountain (0x30);
  - dry (`land` not 1 or 2), unless it carries 0x100;
  - not rock or 0x40 (0x48);
  - not a start-city cell;
  - not COAST.

  A growing cell opens a list, and each on-map orthogonal neighbour that
  may grow (`orthog_x/y[1..=4]`, N E S W) joins it. After every on-map
  neighbour, joined or not, a list of exactly the size wanted turns
  FOREST (flags `& 0xffc3 | 0x20`). Two quirks follow from the listing:
  - a lone cell is a patch of one only when its first on-map neighbour
    cannot grow;
  - once a patch lands, each smaller size stops at the first cell or
    neighbour it tests.

  Great Sahara and Great Lakes never fire it. East Indies' fuzz lobby does
  (below).
- `TerrainGroups::nubify_forest@006a93b0` (the 3) draws `r & 3` per run of
  four straight forest-edge cells. As read, its target sits on the side
  already forest, so it mostly rewrites a FOREST cell's flags to what they
  were. **The trace agrees: on GL the 3 draws move no `World` part.**
- `TerrainGroups::change_forest_base@006a9a60`, `TerrainGroups::change_mountain_base@006a9c10`
  and `TerrainGroups::change_coast_base@006a9dc0` run when the tileset names a base. Each
  draws once per eligible orthogonal neighbour of a source cell.
  `TerrainGroups::fix_transitions@006a9f70` and its
  `TerrainGroups::nubify_transitions@006a1f30` draw once per over-long
  fertility-run cell until a nub lands. Together these are the 1,824.

**Ported on Great Sahara** (item 3805, `sim::terrain_groups::nubify_forest`
and `change_bases`). `nubify_forest` walks columns, then rows; a run of four
with forest on one side draws `r & 3` and re-forests the side cell that many
back, skipping a start-city cell, ocean, and a region edge
(`World::is_edge_of_region`). The base passes take `orthog_x/y[1..=4]` (N,
E, S, W) as the tables give them, and `nubify_transitions`' row pass bounds
its rows by `xs`. The chances are the tileset's `FOREST_BASE`,
`MOUNTAIN_BASE` and `COAST_BASE`, read by `TileSet::init@0087b370` (−1 when
absent; GS_Desert 3/30, 2/30, 3/30). At 2 players the 5 are 5
`nubify_forest` draws and the 1,004 are 370 `change_forest_base`, 202
`change_mountain_base`, 0 coast (no Great Sahara cell is COAST) and 432
`nubify_transitions`. The stream is the trace's through the last of them on
**all 280 Great Sahara traces**, and seed and parts 2, 5 and 6 agree at
7861, 7865 and 7870 on 105 files each, first try
(`great_sahara_s_group_tail_is_the_traced_stream_and_7861_to_7870`). Five
mutations turn it red; `nubify_forest` keeping the COAST, ROCK and MOUNTAIN
bits it clears turns nothing red, because its 5 nubs land on forest cells
that carry none of them.

**Ported on Great Lakes** (item 3822, the same `sim::terrain_groups` on
3819's 7854 grid, tileset `early_autumn`). Great Lakes' ten groups include
the first type-7 group any corpus map runs: "lake oil", pattern 2, 8
clumps of size 1, `rock_space` "6 SCALE" (5 at 60×60). Its start test was
the port's one refusal. `place_region_group@006a2f60` reads it as follows.
A type-7 start must be wet (`land` 2, or COAST). Then a disc is walked
from the centre over the circle table to `circle_radius[r]`, where `r` is
the call's last argument when over 3 (at most 63), else 3. In the world
pattern that argument is the group's `rock_space`. Every on-map cell
inside `circle_radius[2]` must be wet (`is_ocean`), and no wet cell in
the disc may hold oil. Off-map cells pass. The test comes after the
flag/blocked tests and before the centre, edge and corner bounds.

**The team lobby is a second generator run on the same seed.** The
run1077 lobby (a human and two computers) and its click-free kin
(run1086–1113) log no checksum, so they have no start grid. Their setup
path is the 2-nation run's to 7805, and the mask draws the same 5,568 on
both. `place_all` is the first pass that reads the count. The tail of
that lobby draws no `nubify_forest`, so `place_all_part` now cuts at the
first draw of any tail pass.

The stream is the trace's, site and word, on **all 222 Great Lakes
traces**: 209 at 2 nations (1,016 draws, §2's figure) and 13 at 3 (1,319).
Through the tail it is 2,843 and 3,247. At 2 nations the tail's passes
draw:

| pass | 2 nations | 3 nations |
|---|---|---|
| `nubify_forest` | 3 | 0 |
| `change_forest_base` | 436 | 488 |
| `change_mountain_base` | 296 | 288 |
| `change_coast_base` | 384 | 386 |
| `nubify_transitions` | 708 | 766 |

`change_coast_base` runs for the first time here, from 722 COAST cells.
Seed, part 2 and part 5 agree at 7859, 7861, 7865 and 7870 on all 185
files, first try (`great_lakes_s_place_all_is_the_traced_stream_and_7859`,
`great_lakes_s_group_tail_is_the_traced_stream_and_7861_to_7870`). The
185 files are all 2-nation, so the 3-nation lobby is checked by its
stream alone.

The 7859 grid holds 189/203/16/15 forest, mountain, rock and oil cells.
Five mutations on the water arms turn the check red:
- the disc's dry core dropped;
- the disc's oil test dropped (this one only on the 13 team-lobby traces);
- the water region span (type 7 lists regions 0x40..0x7f) narrowed to land;
- `available_rough_tile`'s coast ring emptied;
- `change_coast_base` given no source cells.

Three fail nothing, and each is inert on this corpus:
- **The disc's reach.** The disc's reach is `rock_space` 5 against 3, and
  no start is decided by rings 4 or 5. The unit test
  `the_sea_oil_disc_reaches_rock_space` pins the reading instead.
- **`drop_tile`'s COAST → `land` 2 rewrite.** Every forest and rock group
  has `coast_space` of at least 1, so `available` refuses a COAST cell
  first.
- **The start test's COAST alternative.** Type 7 walks only water
  regions, and COAST marks land, so the disc's dry core refuses such a
  cell anyway.

The pattern-2 weight did not run: there are 6 water regions over five
cells, not fewer (3803b stands).

**Ported on East Indies** (item 3828, the same `sim::terrain_groups` on
3825's 7854 grid, tileset `jungle`, or `sw_desert` on the fuzz lobby).
East Indies' eleven groups:
- a pattern-0 mountain at `coast_space` 0;
- pattern-3 mountain, forest and rock groups;
- "lake oil" at 14 clumps, in the one sea over five cells.

There are 12 land regions over five cells (11 on the fuzz lobby). The
stream is the trace's, site and word, on **all 418 East Indies traces**,
2,119 draws in `place_all` and 3,395 through the tail. Every one is the
jungle run's; the fuzz lobby has no trace. Seed, part 2 and part 5 agree
at 7859, 7861, 7865 and 7870 on **all 202 files**
(`east_indies_s_place_all_is_the_traced_stream_and_7859`,
`east_indies_s_group_tail_is_the_traced_stream_and_7861_to_7870`).

`place_all` agreed first try. The tail did not: on the fuzz lobby's two
files, 7861's part 5 parted with the seed and part 2 agreeing.
`check_player_forest` was unported. One of that lobby's island starts
holds fewer than three forest cells, and its 3-cell patch is that part.
With the pass ported (`sim::terrain_groups::check_player_forest`, now in
every map's chain), the patch's cells feed `change_forest_base` (310 → 314
draws), and all three checkpoints agree.

The tail's draws:

| pass | jungle | sw_desert (fuzz) |
|---|---|---|
| `nubify_forest` | 0 | 0 |
| `change_forest_base` | 364 | 314 |
| `change_mountain_base` | 211 | 192 |
| `change_coast_base` | 518 | 530 |
| `nubify_transitions` | 183 | 87 |

COAST cells entering: 566 and 574. The 7859 grid holds 157/107/14/20
forest, mountain, rock and oil cells (128/100/16/22 on the fuzz lobby).

Mutations (`tools/mutate.py`, scored by the East Indies walks):
- **Held on the corpus:**
  - the pass skipped;
  - its forest threshold at 2;
  - its sizes walked one, two, three;
  - its COAST test dropped;
  - its ring walked from `circle_radius[0]`;
  - its convert setting COAST;
  - **the sea-oil disc's reach fixed at ring 3.** That one parts all 202
    files at 7859: fourteen oil clumps in one sea are decided by rings 4
    and 5, where Great Lakes' eight in six lakes were not (3822a's first
    arm, struck).
- **Held by the unit test** `check_player_forest_patches_three_then_two_then_one`
  only: the start-city bit and the dry-or-0x100 test.
- **Fail nothing** (parked 3828a), for 3822a's reasons:
  - `drop_tile`'s COAST → `land` 2 rewrite: East Indies' forest and rock
    groups are all `coast_space` 1, and the `coast_space` 0 group is a
    mountain, which `drop_tile` does not reach;
  - the type-7 start test's COAST alternative.

The pattern-2/3 weights did not run with more than one region:
- the lake oil weighs its one sea (`c == T`);
- every pattern-3 list holds nine or ten islands, so no list is under six.

3803b stands.

Falsifier: seed and parts 5 and 6 at 7859, 7865 and 7870.

## 8. Resources

The MAPMAKE log in `Map::make` draws nothing. `vector_dist@0046cff0` is
the integer octagonal distance `m·m/(2M) + M`, capped at 60,000. The log
prints the style, size, dimensions, a scale percentage, the area, and each
start with its distance to the centre. A capture with `[Misc Logging]
MAPMAKE` would print the generator's own account (named, not booked).
`make` reaches `Map::place_resources@0068f4f0` by a direct call.

`place_resources` (the 231):
- It fills a `ResourceDivvyPool` with the goods (6..0x31) that have
  `random_rare_drop`. It reads `BONUSES`, `FISH ` (the map file's key has a
  trailing space) and `GOODIES`, from the style's file where it has the
  section and from `default.xml` where it does not. An empty style section
  still wins: SAHARA.xml's `<FISH/>` is found through the trailing space,
  and Great Sahara's 10 ladder rolls are 5 `BONUSES` and 5 default
  `GOODIES` (a default `FISH` would add 2).
- Each bonus entry rolls once per group (`r % 100` on a chance ladder).
- Pattern `player` goes to `Map::place_player_resource@00691f70`; patterns
  `continent`, `world`, `nonplayer` and `corner` go to
  `Map::place_region_resource@00690480` through
  `Map::find_avail_regions@0068f380`.
- The bag draws (`ResourceDivvyPool::get_early@0068a640`,
  `ResourceDivvyPool::get_late@0068a570`) are rejection sampling without replacement, so
  their count is geometric. `Random` and `Late` both take `get_late`.
- `place_player_resource` makes one random start in a ring, scans it, and
  takes the first cell that passes, with no retry. The region path walks
  a region with a 16-bit LFSR and retries a failed `world` attempt in the
  next region with fresh draws.
- Every placement and failure logs at `GLOG_MAPMAKE` detail 7–8. That is the
  cheapest falsifier of this path.

It makes game objects, and the objects write cells:
- `Objects::init_good@00653f30` sets `down` −2 and `down_who`, and starts the
  herds.
- `Good::init@0066da20` sets tile mask 0x200 and cell flag 1, and blocks
  through `World::set_blocked_at@006b4900`.
- `Item::init@006770e0` sets `down` −3 and flag 0x8000.

So this phase moves parts 5 and 6, and the `Goods` and `Items` checksum
lines. The 231 is reconciled on Great Lakes by the port (item 3826, below):
the port draws exactly the trace's 231 at 2 nations.

**Ported on Great Sahara** (item 3805, `sim::map_resources`, from three
blind readings kept outside the repo; inputs `rondata::mapgen::resource_inputs`).
- The ladder's roll and group carry across sections; the selected flag
  resets per section.
- The pool has two lists, told apart by the counts: early 23 goods, late 6
  (Rubber, Sulphur, Aluminum, Coal, Uranium, Titanium — `PREQ0` Gunpowder
  or the Industrial Age). Every repeat draw in both traces lands on a taken
  index, and a modulus of 24 or 5 contradicts run1000. Only `default.xml`'s
  `LAND_0` lists land goods.
- `player_keep_away` is never scaled, so `16 SCALE` reads 16.
- The region walk is `place_all`'s LFSR. Every Great Sahara region call saw
  exactly one available region, so `find_avail_regions` drew nothing.
- `create` marks the footprint 0x200 with `set_blocked_at`, then the cell's
  flag (1 a good, 0x8000 a goody box), `down` (−2, −3) and `down_who`, the
  good's slot in `MapGrid::goods` (§7's oil counts).

The stream from `place_all` to `compute_all_vals` is the trace's on **all
280 traces**, and seed and parts 2, 5 and 6 agree at 7877, 7880 and 7927 on
105 files each (`great_sahara_s_resources_are_the_traced_stream_and_7927`):
36 objects at 2 players and 45 at 3. The first run parted on part 5 alone,
on `down_who`, until the oil goods were counted. Five mutations turn it red;
a dead slot never reused turns nothing red, since no Great Sahara oil good
dies before a later good is made (3805a).

**Ported on Great Lakes, first try** (item 3826, no change to
`sim::map_resources`). It runs from 3822's 7870 grid, reading
`GREATLAKES.xml`'s own `BONUSES` and `FISH ` sections and `default.xml`'s
`GOODIES`. The style's `FISH ` is one `world` row of Fish at 45 SCALE with
`saturate` 1, and it overrides the default section that holds the Whales,
so no Great Lakes run places a whale.

The stream from `place_all` to `compute_all_vals` is the trace's on **all
222 traces** (`great_lakes_s_resources_are_the_traced_stream_and_7927`):
- 209 at 2 nations: 3,074 draws, 231 of them here;
- 13 in the 3-nation team lobby: 3,469 draws.

Seed and parts 2, 5 and 6 agree at 7877, 7880 and 7927 on **all 185
files**. 43 objects are placed at 2 nations (12 Fish) and 58 at 3 (17
Fish).

This is the first diff of the water arms:
- the water region list (0x40..0x7f);
- `find_avail_regions`' draw over six lakes (`0x68f49d`);
- the water cell's 5×5 shore test;
- the `world` pattern's move to the next region, and its retry bookkeeping
  across regions.

Seven mutations turn the walk red. Three turn nothing red (parked 3826a):
- a water cell's COAST refusal: COAST marks land, so no water cell carries
  it;
- `saturate`'s balance switch: Fish saturates its lakes before half its
  count;
- the sea goods' spacing against each other alone: the shore test already
  keeps a Fish two cells from any land good.

East Indies parts the last two (item 3831, below); the COAST refusal stays
inert there too.

**Ported on East Indies, first try** (item 3831, no change to
`sim::map_resources`). It runs from 3828's 7870 grid, reading
`EASTINDIES.xml`'s own three sections. `BONUSES` is two `player` home rows
and two `nonplayer` divvy rows (`Early`, `Late`); `FISH ` is two `world`
rows, Whales (12 SCALE, `saturate` 0) and Fish (50 SCALE, `saturate` 1);
`GOODIES` is one `player` and one `nonplayer` row. No row is `continent`
or `corner`, and both sea rows name their type, so `get_water` never
draws.

The stream from `place_all` to `compute_all_vals` is the trace's on **all
418 traces** (`east_indies_s_resources_are_the_traced_stream_and_7927`):
3,484 draws on the jungle run, and 3,315 on the fuzz lobby, which no trace
walks. Seed and parts 2, 5 and 6 agree at 7877, 7880 and 7927 on **all 202
files**. 53 objects are placed on the jungle run (30 Fish, 10 Whales) and
58 on the fuzz lobby (35 Fish, 10 Whales); `find_avail_regions` draws 3
times on each, over twelve and eleven islands.

This is the first diff of:
- the Whales, a sea good spacing only against Fish;
- `nonplayer`: the islands holding a start left out of the list, its
  move to the next region after each try, and its multi-region threshold
  arm (patterns 1 and 3 keep the balance off when more than one region
  is available);
- an unsaturated water row's starved-player balance (the Whales reach half
  their count), and the starved player's strict maximum;
- `saturate`'s switch on the Fish;
- the sea goods' spacing against each other alone.

Twelve of thirteen mutations turn the walk red; `last_ok` forced does not
terminate. The water cell's COAST refusal turns nothing red, as on Great
Lakes (parked 3831a).

Falsifier: seed and parts 5 and 6 at 7927, and the `Goods`/`Items` lines.

## 9. Values: `compute_all_vals` and its 2,920 draws

`World::adjust_for_blocking@006b3230` draws nothing, as measured. It
marks 3×3 around 0x70 mountains with 0x200, blocks forest tiles, releases
0x40 regions, re-runs the regions and `eliminate_dead_mount_spots`, marks
water edges 0x2000, unblocks forest borders, and sets the gather bit 0x8000.
Its `eliminate_dead_mount_spots` decompiles as a loop that never exits;
the listing says it runs exactly twice (the counter is set to 2 at
0x84fdd5 and decremented at 0x850009), over one visited set the second
pass does not clear.

**Ported on Great Sahara** (item 3807, `sim::map_values`, read from the
decompile and the listings by me; the readings' detail is the module's):
- The forest pass (y outer) blocks a forest cell's 16 tiles through
  `set_blocked_at`, takes one `solid` back for each tile not yet 0x30,
  and marks each 0x38. The border pass (y outer) clears 0x30 and 0x08 on
  a forest cell's east column or south row when that neighbour has
  neither forest nor mountain, gives the `solid` back, and unblocks every
  tile that is not mountain art (`mask & 3 != 2`).
- `eliminate_dead_mount_spots` visits a mountain cell's 3×3 once a game.
  A cell with no open orthogonal neighbour (`flags & 0x30 == 0`) has each
  unblocked tile blocked, and each block rewrites its flags `& 0xffc3 |
  0x10`. The `land` write there stores back the byte it read.
- The 0x8000 bit is `World::set_edge_flags@006b0240`'s:
  `WorldData::has_gather_access(…, −1, 0, 1)` on each placed range's
  tiles, then the cliffs', then each forest cell's. The ranges are
  `Mountains::add_mountain`'s list, so `MapGrid` now carries it
  (`mountains`). No corpus map places a cliff (§13).
- The released regions' flag (`Region.flags & 0x40`) has no writer found
  in the export; the port releases none, and 7937 agrees.
- `compute_val`'s land counts are `LandData::make[4]` (+0x4 of the 0x138
  `Land` record), `rules.xml`'s `LANDS`, as `TypeIndex` values: `none`
  reads −1, since 0 contradicts 7945 (a mutation). `get_land(…, 1)` counts
  a coast, forest, mountain, rock or oil cell as its own land record.

The whole stream from `place_all` through the last value draw is the
trace's on **all 280 traces**, and seed and parts 2, 5 and 6 agree at 7937
and 7945 on 105 files each, first run
(`great_sahara_s_values_are_the_traced_stream_and_7937_and_7945`). All
fourteen mutations turn it red, the two the booking named among them: a
mark rule dropped, and the walk transposed. So does
`eliminate_dead_mount_spots` dropped: on Great Sahara it makes no new
mountain cell, but its blocks move parts 5 and 6. The water-edge arm, the
region release, the 0x1000 tile and the water factors are reading-only
here, because Great Sahara has no water (3807a).

`World::compute_all_vals@006b3790` walks **x outer, y inner**, so
column-major. Per cell it stores `val` (clamped) and `goods`, a bit byte
`0x20 | (count0 > 4) | …`. **`World::compute_val@006b0350` draws once per
cell (`r % 10`) unless the cell is water**, where water is
`!(flags & 0x100) && land ∈ {1, 2}`. So the 2,920 is the number of
non-water cells at 7937.

**Checked against the disk.** On GS and Himalayas, where every cell is
`BASELAND`, the count is 3,600 = 60 × 60 exactly. On the three maps with
water it lies between the final dump's land-only count and its
land-or-HALFLAND count:
- GL: 2,810 ≤ 2,920 ≤ 3,053;
- EI: 1,268 ≤ 1,479 ≤ 1,669;
- BI: 1,870 ≤ 1,975 ≤ 2,111.

That is as it should be, because `Terrain::init` sets HALFLAND after 7945
(§10). The order matters to the stream, and to `val` through the draw.

Falsifier: seed and part 5 at 7945; `val` and `goods` in the start dump
(`Terrain::init` does not write `val`). `compute_val`'s arithmetic, from its
listing (0x6b03de..0x6b07d1), is integer throughout, with every divide
truncating; it is ported as read in `sim::map_values::compute_val`.

## 10. `Terrain::init`: the height grid, half-land, rivers, cliffs

`Terrain::init@00850f70` runs between `setup.cpp` 745 and 759. Its
`Terrain.cpp` checkpoints match the run11 list exactly. **Only
`make_rivers` draws from `game_random`.** Everything else uses private
`Random`s (`internal_random`, each `Fractal`'s own), which do not move the
sync stream. In order:
- 408 → 666: setup of roads, rivers, oil and the renderer. Then constants
  `land_height = 30.0f` and −303.0f.
- 666 → 699: `TerrainOut::refresh_data@00870050` builds the Fractals, and
  `TerrainOut::generate_land_lists@0085f6a0` sets each cell's `CoordInfo`
  flags.
- 699 → 706: `TerrainOut::generate_land@0085f8d0` builds the corner grid
  `master_land_heights`, (4·ys+1) × (4·xs+1) floats.
  `TerrainOut::determine_land_height_color@0086c1e0` gives each vertex its
  height (it returns in xmm0, which the decompile drops):
  - a river vertex 0, `0x1000` −273, `0x4` 0, `0x2` 100;
  - otherwise a height from two Fractals (seeds `World.seed` and twice it),
    blended by the distance to shore.
  
  Then `TerrainOut::smooth_tcoord@0086c3a0` (3×3 in place, list order
  matters) and `TerrainOut::build_coastal_data@0086c540`, which calls
  `TerrainOut::adjust_aquatic_height@00868820` and five in-place passes of
  `TerrainOut::relax_aquatic_verts@00868470`.
- 706 → 720: `Terrain::check_and_set_halfland@0084e610` calls
  `Terrain::check_and_set_halfland_wcoord@0084dec0`. **A sub-tile is water
  when all four of its corner heights are below −30.0f.** That drives
  `Terrain::mark_halfwater@0084ece0`: HALFLAND, `region`, `region2`, `land`,
  COAST spreading, and parts 10–12. **This is where a float decides cells.**
- 720 → 782: oil patches flatten heights
  (`TerrainOil::create_doobers@008a28e0`, `TerrainOut::terraform_for_building@00875210`),
  then skirts, renderer nodes and water.
- 815 → 894:
  - `Terrain::convert_beach_to_sandy@0084b8e0` sets `land` 1 on land next
    to `land` 2.
  - `Map::make_rivers@00692fb0` runs (`map.cpp` 5012..5118).
  - `RiversOut::calculate_all@00888820` carves troughs.
    `RiverSection::create_trough_edge@00889660` calls
    `World::set_river_at@006b1f80` (tile 0x800) where the carved height is
    ≤ 0.
  - `TerrainOut::adjust_for_mountains@008703c0` runs.
  - `Cliffs::init@008a8020` runs: `Cliffs::assess_wcoord@008a6a40` marks a
    cliff where a tile's corner heights span more than
    **192·tanf(25°) = 89.531075f** (`TerrainOut::has_plus_slope@00869790`),
    which blocks it and moves goodies (`Terrain::move_goody@0084a3d0`).
    It is tested only on cells with flag 0x40, and no trace enters the
    slope test (§13).
- 894 → 933: surf (built and closed), borders, and `refresh_good_z`, the
  integer object z from `TerrainOut::find_data_z@00866560`.

**`make_rivers`, the 173.** Only two `game_random` sites are reachable:
- `Map::make_riverlist@00693820` draws `r % (spread + 1)` per river
  element with a spread ≥ 1. It picks sources from
  `Map::make_riverbeds@00694530` (FOREST cells with bit 1 clear, no draw)
  through an LFSR seeded from `World.seed`.
- `PathFinder::astar_river@00686690` draws `r % 20 + 10` per orthogonal
  neighbour it expands.

It writes the flag bits 0x1000 (path), 0x2 (bank) and 0x2000 (source).
It clears 0x2000 and 0x2 before the sources and again after 5076, and it
clears 0x1000 after 5014. The count is data-dependent and not
reconciled; it is 0 on every map but GL.

Two quirks:
- **The Fractals are rebuilt only when the map size changes**, keyed on a
  process-lifetime static `last_map_size`. A second same-size game in one
  process reuses the old ones. Fresh-process captures are unaffected.
- **A `Prefs` value flattens every height to 0.**

**Which parts move, on the disk (item 3809).** Every record from 7945 to
`setup.cpp` 884, on all 105 Great Sahara files (one pattern, two value
sets: the 2- and 3-nation lobbies), and on run100 for Great Lakes:

| record | moved on Great Sahara | moved on run100 (GL) | the write |
|---|---|---|---|
| `setup.cpp` 734 | 4 | 4 | `Map::make@0068bc90`'s last line, after 7945: `world.sea_map = map.sea_map` when not negative |
| `setup.cpp` 745 … `Terrain.cpp` 699, 706 | — | — | |
| `Terrain.cpp` 720 | — | 5, 6, 10–12 | `check_and_set_halfland` → `mark_halfwater` |
| `Terrain.cpp` 738 | — | 5 | the oil patches |
| `Terrain.cpp` 749 … 815, `map.cpp` 5012 | — | — | `convert_beach_to_sandy` among them |
| `map.cpp` 5014 | 5 | 5 | `Map::make_riverbeds@00694530` |
| `map.cpp` 5076 | — | 5 (and the seed) | the rivers; on Great Sahara an empty `RIVERLIST` |
| `map.cpp` 5116 | 5 | 5 | `make_rivers` clears 0x2000 and 0x2 on every cell |
| `map.cpp` 5118 | — | | `Regions::find_all` |
| `Terrain.cpp` 894 | — | 5, 6 | troughs, `adjust_for_mountains`, cliffs |
| `setup.cpp` 884 | 6 | 6 | after `Terrain::init` |

- **Part 4 at 734 is one write, `sea_map` (+0x34) from −1 to the style's
  `SEA_MAP`.** Adler-32 weighs byte `i` of an `n`-byte run 1 in its low
  half and `n − i` in its high half, so a 4-byte change at a known offset
  is checkable without the run's other bytes
  (`sim::mapgen::adler_rewrite`). Over part 4's 120 bytes, 7945 → 734 is
  `0xffffffff` → the style's value at byte 44 on **all 511 traced files**
  of every style: 0 on Great Sahara and Himalayas, 1 on Great Lakes, 3 on
  East Indies and British Isles, each the style file's
  `<SEA_MAP value=…/>` (`Map::init_map_data@0069f250`, default −1 from
  `Map::Map@006a06e0`). Nothing else in part 4 moves there.
- **The half-land gate is `generate_land_lists`' flags, not a height.**
  `check_and_set_halfland_wcoord@0084dec0` — the only reader of a corner
  height in `check_and_set_halfland`, and the only caller of
  `mark_halfwater` — enters a cell only when its `CoordInfo.flags`
  (+0x1c) carries 0x84. `TerrainList::add_new_coord_info@0084ba10` sets
  0x4 on a cell that is `NONE`, flagged 0x4, HALFLAND (0x100) or 0x400,
  and 0x80 on an `OCEAN` cell with a `NONE` or 0x4 cell in its 5×5; it
  also sets 0x8000 (`NONE`), 0x1000 (`OCEAN`) and 0x20 (`BASELAND` with a
  `NONE` or 0x4 cell among its eight neighbours). In `Terrain::init`'s span
  the only other writers of either gate bit are `check_and_set_halfland`'s
  tail (0x80) and `_wcoord` itself (it clears 0x4), by a grep of the
  export for the `| 4`/`| 0x80`/`& 0xfffb` writes. **On Great Sahara no
  cell is gated** (and no cell is 0x20), from the port's own 7945 grid, so
  no float is read and no sub-tile can turn water. The dump's
  `Log::terrain_heights` is the final grid, after the oil, the troughs and
  `adjust_for_mountains`, so it is not the 720 grid and is not the proof;
  parts 5, 6 and 10–13 standing still at 720 on 105 files are.
- `check_and_set_halfland(0)`'s height-free passes, in order: a cell not
  HALFLAND whose `land` is `SANDY` or `OCEAN` marks its 16 tiles `& 0xffef
  | 0x20`; `check_and_set_coastal@0084dd70` gives every wet tile (`mask &
  0x30 == 0x20`) in the 7×7 of each wet tile's first dry neighbour 0x400;
  a `BASELAND` cell in a region above 0x40 is moved to a neighbour's
  region (the region lists' arm, not built); a cell flagged 0x4 or `OCEAN`
  with a HALFLAND cell at `orthog[0..5]` (itself first) sets `CoordInfo`
  0x80. Each needs `SANDY`, `OCEAN`, HALFLAND or 0x4 to enter.
- `convert_beach_to_sandy` converts a `BASELAND` cell with an `OCEAN`
  neighbour (`move[1..9]`) through `spot_update_land(x, y, 1, 0)`, whose
  World half is `land`, `land_sub` and `Terrain::verify_land@0084d8a0`
  (not read), and clears `CoordInfo` 0x2000.
- Between 782 and 815 `Terrain::init` clears flag 0x4 (cell and
  `CoordInfo`) on the cells of a `Terrain` list at +0x6460 and hands them
  to `spot_update_land_batch`; only `Terrain::init` names that offset, and
  on Great Sahara no part moves there. Not read further (3809a).
- **Which writes a height governs (step 7's):** `_wcoord`'s body —
  `mark_halfwater`, its `spot_update_land`, its land and region rewrite,
  its 0x4 spread to orthogonal neighbours — on a gated cell; the oil
  patches' flattening (738 on run100); the troughs' `set_river_at` where a
  carved height is ≤ 0, `adjust_for_mountains` and the cliffs (894). On
  Great Sahara none of them moves a part, so **step 7 has no checksum
  falsifier there**: the heights are checked only against the start
  dump's `Log::terrain_heights`.
- **`make_rivers` on Great Sahara reads no height**: `make_riverbeds`
  (5012 → 5014, an LFSR, no draw) and its clear of 0x2000 and 0x2 on every cell (5076 → 5116)
  are its only writes that move a part there. Step 8 on Great Sahara can
  precede step 7 (§15).

**Ported on Great Sahara** (item 3809, `sim::terrain_init`): the `sea_map`
write, the `CoordInfo` flags, the gate, `check_and_set_halfland`'s
height-free passes and `convert_beach_to_sandy`'s predicate. An arm the
grid enters that is not built (the `_wcoord` body, the region relabel, the
beach's write) is refused by name, never guessed. Seed and parts 2, 5, 6
and 10–13 agree at every record from 734 to 5012 on 105 files (2,415
records), and part 4 at 734 on 511 files, first run
(`great_sahara_s_terrain_init_without_heights_is_734_to_5012`). Parts
10–13 are their lists' empty walk, a 4-byte zero count, 262,145; a full
list's entries are not read.

**`make_rivers` on Great Sahara** (item 3811, `sim::map_rivers`; the
function's own order, `Map::make_rivers@00692fb0`):
- 5012; `Map::make_riverbeds@00694530` first clears 0x2000 and 0x2 on every
  cell, then scans y outer, x inner. A cell becomes a source (0x2000, and
  appended to `river_beds`) when:
  - it is forest without 0x1 (`flags & 0x21 == 0x20`);
  - none of its 16 tiles is mountain (`WorldData::has_mountain_tcoords@006b3050`);
  - it is not within a start city's radius
    (`WorldData::start_city_rad_wcoord@006b3850`: the octagonal distance
    to a `start_city_x/y` entry, times 4, under `CITY_CENTER_RADIUS − 1`);
  - its 7×7 square (`move[1..49]`) is all on the map, holds no source yet,
    and holds no cell of land class 2 (`WorldData::get_land@006b4730`,
    argument 1);
  - at least `river_thresh` of its eight neighbours have no mountain tile
    and no rock (0x8).

  A refusal in the square zeroes the count rather than skipping the cell,
  so a threshold of 0 would take every eligible forest cell. The listing
  also inlines a land class of the cell (0x6945b1..0x6945f9), but it is
  reached only when `flags & 0x74` is clear, which the forest bit rules
  out. `river_thresh` is `default.xml`'s `RIVER_THRESHOLD`, 8;
  `SAHARA.xml` sets none (`Map::Map@006a06e0`'s default is 30). No draw.
- 5014; every cell loses 0x1000. With `river_beds` empty, the function
  returns here and prints nothing more.
- One `make_riverlist` per `RIVER` element of the style's `RIVERLIST`, or
  of `default.xml`'s when the style has none. `SAHARA.xml`'s `<RIVERLIST/>`
  is empty, so **on Great Sahara neither `make_riverlist` nor
  `PathFinder::astar_river@00686690` runs**: the seed holds from 5012 to
  5118 on all 105 files, and part 5 does not move at 5076, so no path
  (0x1000) or bank (0x2) is written. The LFSR over `river_beds` seeded
  from `World.seed` (+0x7c) is `make_riverlist`'s, not `make_riverbeds`'.
- 5076; every cell loses 0x2000 and 0x2, which takes the sources back. Part
  5 at 5116 is 5012's sum again on every file.
- 5116; `Regions::find_all`, which relabels no cell there (every cell
  already carries a region).
- 5118.

The sources: 26 on the 2-nation lobby, 31 on the 3-nation one. Seed and
parts 2, 5, 6 and 10–13 agree with the port at every record from 5012 to
5118 on 105 files (525 records), first run
(`great_sahara_s_rivers_are_5012_to_5118`). `make_riverlist`, `astar_river`
and the two `RIVER` lists' readers are not built; a non-empty list is
refused by name (`Unported::RiverList`).

Falsifier: parts 5, 6 and 10–12 at `Terrain.cpp` 720 and 894, and the
seed at 5076. The height grid itself is in the `DUMP_ALL` start dump
(`Log::terrain_heights`), already read by `crates/sim/src/terrain.rs`.

**The heights on Great Sahara** (item 3812, `sim::map_heights`). The
start dump prints `master_land_heights` whole (241 × 241 singles there),
after `Terrain::init` **and after the starting cities and the start
roster have terraformed it**. So the check stands each file's own roster
on the generated grid, the cities first (`build_cities`), and compares
every vertex. Three writes reach a height there, in this order:
- `generate_land`: `(a · 7.5 + land_height) + b · 1.875` in singles.
  `a` and `b` are `Fractal::get_height` on (smooth 5, flags 2,
  `World.seed`) and (smooth 3, flags 2, `2 · World.seed`), and flags & 2
  makes it answer `h·100 >> 8`. `land_height` (0xcbe54c) is 30.0f, set
  by `Terrain::init`; the decompile prints it as a constant.
  `find_closest_coordinfo` finds no 0x20 cell and returns −1.0, so there
  is no blend and nothing for `smooth_tcoord`.
- `TerrainOil::create_doobers` (738), before the mountains: a cell with
  the rock bit, not ocean, reaches `add_rocky_patch`. On an `OIL` cell,
  that runs `terraform_for_building(4c, 4c, 4, 4)` over the cell's tiles,
  with the building's own mean, border and refusals (`docs/ROADS.md`
  §7.4).
- `adjust_for_mountains` → `fill_mountain_data`. Each placed mountain
  adds its `tcoord_verts` z to the vertex its mesh vertex stands on, once
  per mountain. `MountainRange::init` takes every 4th pixel of
  `TEMPLATE_TEX` whose alpha is set, and the vertex adds
  `byte · (height · 3) / 255` (`byte` the pixel's first byte, `height`
  the `<MOUNTAIN>`'s); the cells walked are `Template.cells`.

`build_coastal_data`, the troughs and the cliffs move no vertex there.
**All 47 Great Sahara start dumps that print the grid agree on every
vertex**: 2,729,807 vertices, 25 distinct grids on the two generated
ones (`great_sahara_s_heights_are_the_start_dump_s_grid`). The shore
blend, `smooth_tcoord`, `build_coastal_data` and the slope test are
refused by name (`Unported::Coast`, `Unported::Slope`).

## 11. The float sites, and which decision governs each

DECISIONS 13 pins a float the original computes before the first frame as
a constant. DECISIONS 19 reproduces a mid-frame float operation by
operation, in integers, where every operation is correctly rounded. Map
generation is a third case: **it runs before the first frame, but its
floats depend on the seed and the map**, so no table can be pinned. And
DECISIONS 19's argument — every operation correctly rounded, so the
original is deterministic across hosts — holds for all but one site below.
All the arithmetic is SSE or SSE2; the readers found no x87 in the
reachable set.

| site | ops | class | feeds | decision |
|---|---|---|---|---|
| `Fractal::get_height@006aa870` | double bilinear, truncated | correctly rounded | the base mask (§5; no corpus style) and the heights (§10) | 19 |
| `Fractal::init@006aa2d0` | double increments | correctly rounded | the same | 19 |
| `Map::scale_land_area@0068b1e0` | single, area·nominal/players | correctly rounded | EI island area, GL spacing (only above the nominal player count) | 19 |
| `Map::scale_number@006a0320` | single, `SCALEWATER`/`SCALELAND` | correctly rounded | no stock map | 19 |
| `TerrainGroups::init_tileset_data@006a61f0` | `(int)(freq/100f·255f)` | correctly rounded | fertility partitions (§6) | 19 |
| `place_all`'s pattern-2/3 weights (only when `clumps > 6 && regions < 6`) | single: `cvtdq2ps`, an `addss` sum in list order, `divss` then `mulss`, `cvttss2si` (0x6a7e24..0x6a7f29) | correctly rounded | a draw's modulus and the order | 19: `sim::single`, in the port; on Great Sahara one region, so the weight is `n` exactly |
| the heights (`generate_land` … `relax_aquatic_verts`) | single, sqrt through `_libm_sse2_sqrt_precise` (equal to a correctly rounded sqrt) | correctly rounded | half-land (§10), cliffs, rivers, object z | 19, and its software single (`sim::single`); the fractals' bilinear sample in a software double (`sim::double`, item 3812) |
| `TerrainOut::has_plus_slope@00869790` | `tanf(25°)` | **transcendental**, constant argument | the cliff threshold 89.531075f (`0x42b30fe9`) | **13**: pin it; the double result sits 0.30 ulp from the float, so a faithful libm agrees |
| `Cliffs::verify_defensive_position@008a8680` | `sqrtf`, then **`acos`** ×57.29578 against 22.5° or 17.5° | **transcendental, data-dependent** | whether a cliff group places | **neither**: no trace reaches it (§7) |
| `MapFairness::calc_distances@0068a1c0` | single | correctly rounded | Warring States only | 19 |

The `acos` site is the only one no decision covers. It is unreached on
every corpus map. It names DECISIONS work for the day a cliff tileset is
in scope, not now.

## 12. What in these stems is not map generation

Of the territory's 198 open or layout-only functions under the nine stems
(`tools/census.py --territory`, 79 under `map`), these belong elsewhere:
- **`TerrainVis::*`** is the renderer's vertex-stream and visibility cache.
- **`TerrainMap::*`** is the scenario editor's terrain window, cut from v1.
- **`Surf::*`** is shoreline waves, renderer; `Terrain::init` builds it and
  closes it at once.
- `Terrain::walk_data@00852a60` and `Terrain::walk_roads@00852b00` are the
  save game.
- `Terrain::object_placed@00850c40` is in-game, and
  `Terrain::add_weathering@00850120` is renderer decals.
- `MapConquest::*` is Conquer the World, cut from v1.
- The 16 non-corpus `make_continents` overrides are map generation, but no
  trace enters them.

## 13. Coverage: what a diff backs, what a reading alone holds

**Measured or diff-backed:**
- the phase order;
- every checkpoint line;
- every draw count in §2;
- §3's Adler-32 identity (337 of 337; 496 of 496 pinned, item 3776);
- §5's Great Sahara `make_continents`: its seed, part 2 and part 5 at 7807
  on 105 files (item 3776);
- §5's Great Lakes `make_continents`: draw for draw on 222 traces, the
  5,568, and its seed, part 2 and part 5 at 7807 on 185 files, one
  generator run (item 3815); `eliminate_pools`' merge, the 1,000-miss
  shrink and `scale_land_area`'s float path are reading-only or
  unit-tested (3815b);
- §5's East Indies `MapEastIndies::make_continents@00697540`: draw for
  draw on 418 traces, the
  49,611 and 55,533, and its seed, part 2 and part 5 at 7807 on 202 files,
  two generator runs of 2 nations (item 3820); the restart, the
  three-player radius and the four-player angle are unit-tested only
  (3820a);
- §6's passes on Great Sahara at 7835, 7843, 7846 and 7854 on 105 files,
  `fill_fertile`'s `Fractal` and partitions with them, and `find_all`'s
  labels on 494 of 496 start grids (item 3798); the same four on Great
  Lakes' 185 files, with `fix_diag_land`, `fix_lakes`, the COAST marking,
  the thinning, the sea labels and the drawn tileset each held by a mutant
  (item 3819); the same four on East Indies' 202 files, with the land
  record search, the scan order and the second `clear_all` each held by a
  mutant there too (item 3825); the `region2` pass and the adoption arm are
  unit-tested only, inert at these checkpoints (§6);
- §7's `place_all` on Great Sahara: draw for draw on 280 traces, and seed
  and parts 2, 5 and 6 at 7859 on 105 files (item 3803). The groups' parse
  and the templates come with it. Cliffs, patterns 3 and 4, oil by water
  (type 7) and `clear_group`'s first branch are reading-only;
- §7's `place_all` and its tail on East Indies: draw for draw on 418
  traces, and seed, part 2 and part 5 at 7859, 7861, 7865 and 7870 on 202
  files (item 3828), with the sea-oil disc's reach and
  `check_player_forest` each held by a mutant there; the pass's start-city
  and dry tests are unit-tested only, and `drop_tile`'s COAST rewrite and
  type 7's COAST alternative stay inert (3828a);
- §7's tail and §8's `place_resources` on Great Sahara: draw for draw on
  280 traces, and seed and parts 2, 5 and 6 at 7861, 7865, 7870, 7877, 7880
  and 7927 on 105 files (item 3805). Slot reuse, patterns 1 and 3, the
  Whales and the `Goods`/`Items` lines are reading-only (3805a–c);
- §8's `place_resources` on Great Lakes: draw for draw on 222 traces, both
  lobbies, and seed and parts 2, 5 and 6 at 7877, 7880 and 7927 on 185
  files (item 3826). `find_avail_regions`' draw, the water region list,
  the shore test and the `world` move across regions are diff-backed here;
  the water cell's COAST refusal, `saturate` and the sea-only spacing run
  but cannot part (3826a);
- §8's `place_resources` on East Indies: draw for draw on 418 traces, and
  seed and parts 2, 5 and 6 at 7877, 7880 and 7927 on 202 files (item
  3831). The Whales, `nonplayer`, the unsaturated balance, `saturate` and
  the sea-only spacing are diff-backed here; the COAST refusal stays inert
  (3831a), and `get_water`, `continent` and `corner` run on no corpus map;
- §9's `adjust_for_blocking` and `compute_all_vals` on Great Sahara: draw
  for draw on 280 traces through the last value draw, and seed and parts
  2, 5 and 6 at 7937 and 7945 on 105 files (item 3807). The water edges,
  the region release, the 0x1000 tile, the water and region-size factors
  and the cliffs' 0x8000 are reading-only (3807a);
- §10's `Map::make` tail: part 4 at 734 on 511 files of every style; and
  on Great Sahara, `Terrain::init`'s span with no cell gated: seed and
  parts 2, 5, 6 and 10–13 at every record from 734 to 5012 on 105 files
  (item 3809). The flags' arms on a gated cell, the height-free passes'
  water arms and the beach predicate are unit-tested only (3809a);
- §10's `make_rivers` on Great Sahara: `make_riverbeds`' sources and the
  three clears, with seed and parts 2, 5, 6 and 10–13 at every record from
  5012 to 5118 on 105 files (item 3811). The square's water and oil arms,
  the start-city radius and `find_all`'s relabel are unit-tested only;
  `make_riverlist` and `astar_river` are not built (3811a);
- §10's heights on Great Sahara: `generate_land`, the oil cells'
  terraform and `adjust_for_mountains`, on every vertex of 47 start
  dumps through each file's roster (item 3812). Mutations that drop an
  octave, the mountains or the oil, or swap the two weights, each fail
  it; dropping the rock bit from the oil test fails nothing (§14);
- §2's GS finding that the land mask is independent of the player count;
- §4's three draws as a total;
- §9's one draw per non-water cell, exact on GS and Himalayas and
  bracketed on GL, EI and BI;
- §7's `nubify_forest` moving no part on GL.

**Reading-only**, each with its falsifier named in its section: every
per-function formula and predicate in §§4–10. The draw formulas for
`make_continents` (GL's and EI's reconciled, items 3815 and 3820), `place_all`, `change_*_base`/`fix_transitions`,
`place_resources` (reconciled on Great Sahara, Great Lakes and East Indies, items 3805, 3826 and 3831) and `make_rivers` are consistent in kind but not
reconciled to their measured totals. Also §3's parts 10–13 offsets and
§11's float classification.

**Never entered by any trace in `rondata::blind::TRACES`** (on
`blind::NEVER` with this item):
- `Map::make_continents@00694ab0`,
- `Map::force_continent_spacing@0068e6a0`,
- `Cliffs::position_cliff@008a8bc0`,
- `Cliffs::verify_defensive_position@008a8680`,
- `Cliffs::place_cliff@008a7ea0`,
- `MapFairness::calc_distances@0068a1c0`,
- `TerrainOut::has_plus_slope@00869790` — `Cliffs::assess_wcoord` is entered, but it reaches the slope test only on a cell with flag 0x40 that is not COAST, `land` 1 or 2, or a tile whose mask's low bits read 2. So §10's 89.53 threshold has decided no corpus cell; a start dump's count of 0x40 cells is its falsifier.

Their claims are reading-only until a capture enters them; a cliff
tileset or a base-mask style is what would.

## 14. What is not established

- The draw formulas reconciled to 1,016, 1,824, 231 and 173
  (`place_all`'s are reconciled on Great Sahara, 1,238 and 1,324, §7; the
  5,568 on Great Lakes, item 3815, and the 49,611 on East Indies, item
  3820, §5). Each
  needs the grid at its phase, which a port's own phase supplies, checked
  by §3. `GLOG_MAPMAKE` at detail 7–10 would count them in the original.
- ~~Whether MSXML's `selectSingleNode` honours `FISH ` with its trailing
  space, as read (§8).~~ It does: §8, item 3805.
- ~~The unnamed int at +0x1fc of unit type 349, which GL takes as
  `check_player_land`'s radius, rounded up after dividing by 4.~~ It is
  `max_range`, 24 in the shipped rules, so ring 6; 7807 meets it on 185
  files (§5, item 3815).
- `clear_group` writes `land_sub = num_tiles[0] − 1` on plain land, so it is
  not a true undo (listing-confirmed); whether that reaches a corpus grid.
- Several loops bound y by `xs`. That is harmless on the corpus's square
  maps and would part on a non-square one.
- The 0x40 offset of §3's parts 10–13.
- Whether `create_doobers`' rock-bit test decides a height: every
  Great Sahara oil cell carries the rock bit, so the conjunction is
  untested there. The flag-1 arm (a good the console's leader cannot
  see yet) reaches no oil cell, since `set_oil_at` sets no flag 1.
- `relax_aquatic_verts`' summation order and the river spline's
  predicates. Both need a listing read before a bit-exact port.

## 15. The port chain

Parked, not queued (`docs/PARKED.md`, item 3734's heading), in order. Each
step is checked by §3 at its own checkpoint before the next begins:
1. the land/sea mask against every grid (§5, 7807) — **Great Sahara
   built** (item 3776, `sim::mapgen`; the `Mapgen:` line's first phase);
   **Great Lakes built** (item 3815, `sim::map_lakes`; its first phase);
   **East Indies built** (item 3820, `sim::map_east_indies`; its first
   phase);
2. the zero-draw passes (§6, 7854) — **Great Sahara built** (item 3798,
   `sim::mapgen`; the `Mapgen:` line's second phase); **Great Lakes
   built** (item 3819, the same passes on its 7807 grid, the water arms'
   first diff; its second phase); **East Indies built** (item 3825, the
   same passes on its 7807 grid, twelve islands, the land record search's
   first diff; its second phase);
3. `place_all` (§7, 7859) — **Great Sahara built** (item 3803,
   `sim::terrain_groups`; the `Mapgen:` line's third phase); 7861–7870's
   `nubify_forest`, `change_*_base` and `fix_transitions` — **Great Sahara
   built** with step 4 (item 3805); **Great Lakes built**, `place_all`
   and its tail together (item 3822, the sea-oil disc and
   `change_coast_base`'s first diff; its third phase); **East Indies
   built** (item 3828, the disc's reach and `check_player_forest`'s
   first diff; its third phase);
4. resources (§8, 7927) — **Great Sahara built** (item 3805,
   `sim::map_resources`; the `Mapgen:` line's fourth phase); **Great Lakes
   built** (item 3826, the same port on its 7870 grid, the water arms'
   first diff; its fourth phase); **East Indies built** (item 3831, the
   same port on its 7870 grid, the Whales' and `nonplayer`'s first diff;
   its fourth phase);
5. values (§9, 7937 and 7945) — **Great Sahara built** (item 3807,
   `sim::map_values`; the `Mapgen:` line's fifth phase);
6. `Terrain::init` without heights (§10, 734–5012) — **Great Sahara
   built** (item 3809, `sim::terrain_init`; the `Mapgen:` line's sixth
   phase): no cell opens the half-land gate there, so every record to
   5012 stands still but part 4 at 734;
7. the heights in `sim::single`, with half-land and cliffs (§10, 720–894)
   — **Great Sahara built** (item 3812, `sim::map_heights`; the `Mapgen:`
   line's eighth phase). No checksum part moves for them there, so their
   falsifier is the start dump's height grid, which all 47 files meet.
   The shore blend, `smooth_tcoord`, `build_coastal_data` and the slope
   test remain;
8. rivers (5012–5118) — **Great Sahara built** before step 7 (item 3811,
   `sim::map_rivers`; the `Mapgen:` line's seventh phase): there the
   phase is height-free (`make_riverbeds` at 5014, the 0x2000/0x2 clear
   at 5116) and draws nothing, since `SAHARA.xml`'s `RIVERLIST` is empty.
   `make_riverlist` and `astar_river`, which Great Lakes' 173 draws need,
   remain.
