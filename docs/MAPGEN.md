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
three of the first, which fits parts 10–12 moving in `Terrain::init`, and
the port's three lists walked in that order agree with parts 10–12 at 720
on all 185 Great Lakes files (item 3843, §16), so the shapes are
diff-backed. The 0x40 is unexplained.

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

**Base, `Map::make_continents@00694ab0`** (no lobby reaches it, §18.1;
no trace enters it). It builds four `Fractal`s on private `Random`s
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
`MapFairness::calc_distances@0068a1c0` (integer-exact, §18.2).
`Map::force_continent_spacing@0068e6a0`: editor-only (§18.1).

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
on a data path (§11). No trace in `TRACES` enters either, or
`Cliffs::place_cliff@008a7ea0`; run1118 (Southwest Mesa) enters all three (§18.3).

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

**Checked on Great Lakes** (item 3830, no sim change): from 3826's 7927
grid, the whole stream through the last value draw is the trace's on **all
222 traces**, 209 at 2 nations (2,920 value draws) and the 13 of the team
lobby (2,908), and seed and parts 2, 5 and 6 agree at 7937 and 7945 on
**all 185 files**, first run
(`great_lakes_s_values_are_the_traced_stream_and_7937_and_7945`). Six water
mutants turn it red: open water drawn, either water factor dropped
(`water` in `(count / 5, count / 3)` → ×3/2, `water ≥ count / 3` → ÷10),
the ring's `water += 1` or its `count −= 1` dropped, and the square's wet
−1 dropped. Nine do not, and each is inert by the grid, as measured at
7927 and 7945 (3830a): no ocean cell lies on the map's border, so the
0x2000 edge, its `bad` count and its band write nothing; no cell is
`SANDY`; every coast cell's `land` is 0 and every dry cell's food bit is
already set, so counting a coast as `Land` moves no `goods`; no coast cell
is rough; ocean oil's record 7 makes only Oil, which neither `val` nor
`goods` reads; every cell is labelled, so the re-run `find_all` writes
nothing; and the one land region holds every dry cell, so no region-size
factor fires.

**Checked on East Indies** (item 3840, no sim change): from 3831's 7927
grid, the whole stream through the last value draw is the trace's on **all
418 traces** (1,479 value draws on the jungle run, 1,403 on the fuzz
lobby's), and seed and parts 2, 5 and 6 agree at 7937 and 7945 on **all
202 files**, first run
(`east_indies_s_values_are_the_traced_stream_and_7937_and_7945`). The
archipelago reaches six of 3830a's nine arms, and a mutant of each turns it
red: 236 border cells are open sea, so the 0x2000 edge, its `bad` count and
its two-tile band each part 5 and 6 (part 6 alone for `bad`) at 7937; the
coast record 3 and the rough coast's exemption (20 and 23 rough coast
cells) part 5 at 7945; and 930 and 996 dry cells lie in a region under a
sixteenth of the map, so the `size / 16` factor parts 5 at 7945. Five
mutants fail nothing, each inert by the grid (3840a): no cell is `SANDY`;
ocean oil's record 7 makes only Oil (14 sea oil cells); the re-run
`find_all` relabels 0 cells; and the 37 and 85 dry cells under `size / 64`
(37 and 0 under `size / 128`) all keep `val` 0 — islets that small are
nearly all water, so the ÷10 factor has already truncated the byte and
the extra `/ 2` or `× 3 / 4` writes the same 0.

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
  COAST spreading, and parts 10–12. **This is where a float decides cells**
  (the body, §16).
- 720 → 782: oil patches flatten heights
  (`TerrainOil::create_doobers@008a28e0`, `TerrainOut::terraform_for_building@00875210`),
  then skirts, renderer nodes and water.
- 815 → 894:
  - `Terrain::convert_beach_to_sandy@0084b8e0` sets `land` 1 on land next
    to `land` 2.
  - `Map::make_rivers@00692fb0` runs (`map.cpp` 5012..5118).
  - `RiversOut::calculate_all@00888820` carves troughs (§17).
  - `TerrainOut::adjust_for_mountains@008703c0` runs.
  - `Cliffs::init@008a8020` runs: `Cliffs::assess_wcoord@008a6a40` marks a
    cliff where a tile's corner heights span more than
    **192·tanf(25°) = 89.531075f** (`TerrainOut::has_plus_slope@00869790`),
    which blocks it and moves goodies (`Terrain::move_goody@0084a3d0`).
    It is tested only on cells with flag 0x40, and only run1118
    enters the test (§18.4).
- 894 → 933: surf (built and closed), borders, and `refresh_good_z`, the
  integer object z from `TerrainOut::find_data_z@00866560`.

**`make_rivers` is the span's only `game_random` user** (173 draws on
Great Lakes at 2 nations, 62 at 3, none on an empty `RIVERLIST`): the
`make_rivers` block below.

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
  falsifier there**, only the start dump's grid. On East Indies, with no
  river, 894 and the grid are the oil, the beach and the mountains alone
  (item 3851, §17).
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

**On Great Lakes the gate opens, and 720 is a height's** (item 3832,
`great_lakes_s_terrain_init_is_734_to_706_and_its_gate`, no `sim`
change). All 196 Great Lakes files that carry the span share one pattern,
the run100 column above: 734 moves part 4, nothing moves from 745 to 706,
720 moves 5, 6 and 10–12, and 738 moves 5. 185 of them are keyed to a run
(the other 11, runs 16–19 and 47–52, have no start grid or no 7805).
- On the 7945 grid: 2,920 `BASELAND` and 680 `OCEAN` cells, no `SANDY`,
  `NONE` or HALFLAND. The 360 lake-shore cells carry world flags 0x4 and
  0x400 both. `CoordInfo`: 360 0x4 (each as `NONE`), 406 0x20, 680
  `OCEAN`, 516 of them shallow, so **876 cells are gated**.
- Seed and parts 2, 5, 6 and 10–13 at every record from 734 to 706 are the
  7945 grid's on all 185 files (2,960 records), and part 4 at 734 is
  `GREATLAKES.xml`'s `SEA_MAP` 1, first run.
- `_wcoord` compares each sub-tile's four corner heights with −30.0f
  before any write, so 720 was the first record a height decides; its
  body is §16 (item 3843). Every start-dump `HALFLAND` cell is gated; the
  0x4-as-`NONE`, 0x400 and 5×5 arms fail nothing there and are
  unit-tested (3832b).

East Indies is the same (item 3844, §15 step 6).

**`make_rivers`** (items 3811 and 3845, `sim::map_rivers`; the
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
  so a threshold of 0 would take every eligible forest cell (the
  listing's dead inlined land class: its doc comment). `river_thresh` is `default.xml`'s `RIVER_THRESHOLD`, 8;
  `SAHARA.xml` sets none (`Map::Map@006a06e0`'s default is 30). No draw.
- 5014; every cell loses 0x1000. With `river_beds` empty, the function
  returns here and prints nothing more.
- One `Map::make_riverlist@00693820` per `RIVER` of the style's
  `RIVERLIST`, else `default.xml`'s (empty on Great Sahara and East
  Indies), `river_length` (capped at 399),
  `river_min_length` and `river_valley` `SCALE`d by width:
  - one draw `% (spread + 1)` (`+0x76`); `SCALE(start + r)` rivers, ≥ 1;
  - the sources in a Galois LFSR's order over `1..=n` (8 bits, taps 0xb8
    under 251; 12, 0xca0 under 4001; else 16, 0xb400) from `World.seed`
    (+0x7c) `% n + 1`, to its third return; with nations, a source whose
    nearest start city ends more rivers than the least is passed;
  - `PathFinder::astar_river@00686690` from each: greedy best-first over
    `orthog[1..5]`, equals newest first, one draw `% 20 + 10` (`+0xa28`)
    per dry neighbour it prices, times `twist + 1` straight or `101 − twist` turning, plus a
    pull toward `river_length` under flag 8; it ends on water, a path or
    the border that far off (every refusal: its doc comment);
  - a path of 1..399 cells ending on water (flag 1), the border (2) or
    inland (4), at least `min_length` and `length / (wraps + 1)` long, is
    kept: 0x1000 on its cells, 0x2 on the `circle_radius[valley]` disc
    round each within the cell's region; the walk ends when the count is
    spent or at its second wrap.
- 5076; every cell loses 0x2000 and 0x2.
- 5116; `Regions::find_all` (it relabels nothing on Great Sahara).
- 5118.

Great Sahara: 26 and 31 sources, 525 records on 105 files
(`great_sahara_s_rivers_are_5012_to_5118`). Great Lakes (item 3845): 20
and 22; the river draws meet every trace whole (173 on 209, 62 on 13),
and seed and parts 2, 5, 6 and 10–13 agree at every record from 5012 to
5118 on all 185 files (925; 59 path and 707 bank cells at 5076), first
run (`great_lakes_s_rivers_are_5012_to_5118`). The spline is the
renderer's, not built.

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
  by `Terrain::init`. No 0x20 cell, so no shore blend.
- `TerrainOil::create_doobers` (738), before the mountains: a cell with
  the rock bit, not ocean, reaches `add_rocky_patch`. On an `OIL` cell,
  that runs `terraform_for_building(4c, 4c, 4, 4)` over the cell's tiles,
  with the building's own mean, border and refusals (`docs/ROADS.md`
  §7.4).
- `adjust_for_mountains` → `fill_mountain_data`: each placed mountain's
  `TEMPLATE_TEX` displacement, `byte · (height · 3) / 255` per vertex,
  once per mountain (`sim::map_heights::Mesh`).

Nothing else moves a vertex there. **All 47 Great Sahara start dumps that print the grid agree on every
vertex**: 2,729,807 vertices, 25 distinct grids on the two generated
ones (`great_sahara_s_heights_are_the_start_dump_s_grid`). The slope
test is refused by name (`Unported::Slope`).

**The heights on Great Lakes, through 706** (item 3835,
`sim::map_heights`, whose comments carry each constant and address):
- **the shore blend**: `find_closest_coordinfo@0086a260` searches
  ring by ring for a 0x20 cell and stops two rings past the first find;
  with `d ≥ 0` the height is `0.25·(d·h) + ((d·0.05·7.5·192 + 30)·(4 −
  d))·0.25`, and the vertex is listed;
- `smooth_tcoord@0086c3a0` runs over that list, twice on an odd
  coordinate;
- `build_coastal_data`'s height half: the valid influencers (no inner bend
  corner), `LineState` lines at `texture_fractal` sub-tile offsets (the
  second line reads the *current* cell, 0x868e80), `adjust_aquatic_height`'s
  `min(d/768, 10000)²·(1/9)·(−273) + 30`, and five
  `relax_aquatic_verts` passes.

**Every vertex agrees** on all 45 start dumps that print the grid,
the troughs (§17) included (`great_lakes_s_heights_are_the_start_dump_s_grid`).
**And they decide HALFLAND cell-exact**: on all 185 files, a cell is
HALFLAND exactly when it is gated and has 1–15 sub-tiles with four corners
below −30.0f (`water_subtiles`; 243 a file). `_wcoord`'s writes on them
are §16.

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
| the heights (`generate_land` … `relax_aquatic_verts`) | single, sqrt through `_libm_sse2_sqrt_precise` (equal to a correctly rounded sqrt) | correctly rounded | half-land (§10), cliffs, rivers, object z | 19, and its software single (`sim::single`); the fractals' bilinear sample in a software double (`sim::double`, item 3812); the shore blend, smoothing and coastal pass the same (item 3835) |
| `TerrainOut::has_plus_slope@00869790` | `tanf(25°)` | **transcendental**, constant argument | the cliff threshold 89.531075f (`0x42b30fe9`) | **13**: pin it; the double result sits 0.30 ulp from the float, so a faithful libm agrees |
| `Cliffs::verify_defensive_position@008a8680` | `sqrtf`, then **`acos`** ×57.29578 against 22.5° or 17.5° | **transcendental, data-dependent** | whether a cliff group places | **neither**: run1118 (Southwest Mesa, the one cliff style) enters it, in `TRACES` since item 3858; no value diff (§18.3) |
| `MapFairness::calc_distances@0068a1c0` | single, but an integer score × 1.0f | **exact** (every value is an integer below 2²⁴) | Warring States only (run1122 enters it) | 19; integer-exact as read (§18.2, item 3856) |

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
- §9's values on Great Lakes: draw for draw on 222 traces through the last
  value draw, both lobbies, and seed and parts 2, 5 and 6 at 7937 and 7945
  on 185 files (item 3830). Open water drawing nothing, the water count in
  the rings and the square, and both water factors are diff-backed here;
  the border edge, `SANDY`, the coast record, the rough coast's exemption,
  ocean oil's record and the region-size factors run but cannot part
  (3830a);
- §9's values on East Indies: draw for draw on 418 traces through the last
  value draw, and seed and parts 2, 5 and 6 at 7937 and 7945 on 202 files
  (item 3840). The border water edge, its `bad` count and band, the coast
  record 3, the rough coast's exemption and the `size / 16` factor are
  diff-backed here; `SANDY`, ocean oil's record, the re-run `find_all` and
  the `size / 128` and `size / 64` factors run but cannot part (3840a);
- §10's `Map::make` tail: part 4 at 734 on 511 files of every style; and
  on Great Sahara, `Terrain::init`'s span with no cell gated: seed and
  parts 2, 5, 6 and 10–13 at every record from 734 to 5012 on 105 files
  (item 3809). The flags' arms on a gated cell, the height-free passes'
  water arms and the beach predicate are unit-tested only (3809a). On
  Great Lakes the gate opens: seed and parts 2, 5, 6 and 10–13 at every
  record from 734 to 706 on 185 files, and the gate holds every one of the
  start dump's 243 `HALFLAND` cells on all 185 (item 3832); the 0x4,
  `NONE`, 0x400 and 5×5 arms are redundant or beyond that pin there and
  stay unit-tested (3832b); 720 onward is height-decided (3832a), and on
  the 706 heights `_wcoord`'s body moves seed and parts 5, 6 and 10–13 at
  720 and every record to 5012 as the original's on all 185 files (item
  3843, §16): the scan's break and last `region2`, the 0x4 spread's break,
  oil's third ring and the relabel's swap are unit-tested only, the good
  arm and the beach's write are not reached (3843a);
- §10's `make_rivers` on Great Sahara: `make_riverbeds`' sources and the
  three clears, with seed and parts 2, 5, 6 and 10–13 at every record from
  5012 to 5118 on 105 files (item 3811), and on East Indies' 202 (1,010
  records, item 3846), where the start-city test is held by the walk; the square's water and oil arms, the radius's `− 1`
  and `find_all`'s relabel are unit-tested only (3846a). `make_riverlist`
  and `astar_river` on Great Lakes (item 3845): draw for draw on all 222
  traces (173 on 209, 62 on 13) and every record from 5012 to 5118 on 185
  files (925), the 5076 bank clear included; 26 of 47 mutants held, the
  21 that failed nothing are 3845a;
- §10's heights on Great Sahara: `generate_land`, the oil cells'
  terraform and `adjust_for_mountains`, on every vertex of 47 start
  dumps through each file's roster (item 3812). Mutations that drop an
  octave, the mountains or the oil, or swap the two weights, each fail
  it; dropping the rock bit from the oil test fails nothing (§14);
- §10's heights on Great Lakes through 706: the shore blend, smoothing
  and `build_coastal_data`, on every vertex off the rivers of 45 start
  dumps, and the HALFLAND they decide cell-exact on 185 files (item
  3835). Eight mutants fail it; the 15.5 clamp, the search's second ring
  past the first find, and `adjust_aquatic_height`'s `land` 1/3 arm fail
  nothing, being inert on this grid (3835a);
- §17's troughs on Great Lakes (item 3850): seed and parts 2, 5, 6 and
  10–13 at 894 on all 185 files, every vertex of 45 start dumps
  (2,613,645) and the 0x1000 cells on 185, first run; 23 of 33 mutants
  fail it, the ten inert arms named in §17;
- §10's step 7 tail on East Indies (item 3851): seed and parts 2, 5, 6 and
  10–13 at 894 on all 205 files, every vertex of 57 start dumps
  (3,310,617), and no 0x1000 or 0x40 cell on 205, first run with no `sim`
  change; the arms that can part there fail it, the rest are measured
  inert (§17, "On East Indies");
- §10's `Terrain::init` without heights and its heights on East Indies
  (item 3844): seed and parts 2, 5, 6 and 10–13 from 734 to 706 on 202
  files, every vertex of 56 start dumps, and `HALFLAND` cell-exact on 202.
  Of 3832b's and 3835a's seven arms, `coord_land`'s 0x4-as-`NONE` now
  fails it; the other six fail nothing here too, measured inert (3844a).
  §16's half-land body, 720 to 5012 on all 202 (item 3844, on 3843's
  `sim::halfland`): the 0x4 spread's break and the shallow 5×5 ring now
  fail it; five of 3843a's arms stay unit-tested;
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
- ~~`Cliffs::position_cliff@008a8bc0`, `Cliffs::verify_defensive_position@008a8680`, `Cliffs::place_cliff@008a7ea0`,
  `MapFairness::calc_distances@0068a1c0`, `TerrainOut::has_plus_slope@00869790`~~ — entered on −1 by run1118 and
  run1122, in `TRACES` since item 3858; entry only, so their values stay reading-only (3856b).

Their claims are reading-only until a capture enters them; a cliff
tileset or a base-mask style is what would.

**Item 3856's verdicts, row by row** (a reading, with no capture taken;
the trace scan covered every `rontrace-*.log` under `~/ron-data`):

| function | what enters it | the take | verdict |
|---|---|---|---|
| `Map::make_continents@00694ab0` | nothing: only `MapScenario` keeps the slot, and its `make` never dispatches +0x40 (§18.1) | none | `RESIDUE` candidate |
| `Map::force_continent_spacing@0068e6a0` | `ScenarioEditor::import_image` only (§5) | none, since the editor is cut | `RESIDUE` candidate |
| `MapFairness::calc_distances@0068a1c0` | every Warring States lobby with nations (style 15) | **run1122**, in `TRACES` (3858) | entered |
| `Cliffs::position_cliff@008a8bc0` | Southwest Mesa's type-8 groups (style 10) | **run1118, on disk** | entered |
| `Cliffs::verify_defensive_position@008a8680` | Mesa's `pattern="player"` cliffs | run1118 | entered |
| `Cliffs::place_cliff@008a7ea0` | any queued cliff, through `Cliffs::init` | run1118 | entered |
| `TerrainOut::has_plus_slope@00869790` | a 0x40 cell, which Mesa's cliffs make (§18.4) | run1118 | entered |

Five of the seven need **no capture**: item 3858 joined `rontrace-run1118.log`
and `rontrace-run1122.log` to `blind::TRACES`, and eight rows left `NEVER`
(263 → 255, measured), the five and `verify_placement`, `add_cliff`,
`MapWarringStates::make_continents`. A value diff of the cliff chain is a
second item: a Southwest Mesa checksum capture at `check_all_level ≥ 14`
whose falsifiers are 7859 and 894 (§18.3, §18.4).

**Item 3863's verdict, reading-only** (§19): `TerrainOut::check_for_diagonal@008595f0`
and `WorldData::is_valid@00471170` are kept out by a predicate. The arm
needs P1 (orthogonal neighbours mutually unacceptable) or P2 (an orthogonal
`BASELAND`–`OCEAN` pair). Every claim of §19.1–§19.3 is a reading. §19.4's
counts (0 P1, 0 P2 on 86 start grids and on the four ported pre-beach grids)
come from a scratch port, which is not pinned. The predicate is owed a blind
second reading before the row is called `RESIDUE`.

## 14. What is not established

- That map generation holds §19's P1 and P2 before `Terrain::init`'s
  transitions is measured on four lobbies and 86 dumps, not read. Whether
  `verify_land`'s depth ever saturates in a v1 game is not read either
  (3863a).
- The draw formulas reconciled to 1,016, 1,824 and 231
  (`place_all`'s are reconciled on Great Sahara, 1,238 and 1,324, §7; the
  5,568 on Great Lakes, item 3815, and the 49,611 on East Indies, item
  3820, §5; the rivers' 173 and 62 on Great Lakes, item 3845, §10). Each
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
- `verify_defensive_position`'s players-centre target divides Σx by
  `World+0x84` and Σy by `World+0xa0` (§18.3, item 3856). Whether the two
  counts are ever unequal is unread (§18.3).
- Warring States' fairness break compares +0x4c with +0x48 as the
  decompile prints it, and the compare is not listing-read (§18.2).
- Whether `create_doobers`' rock-bit test decides a height: every
  Great Sahara oil cell carries the rock bit, so the conjunction is
  untested there. The flag-1 arm (a good the console's leader cannot
  see yet) reaches no oil cell, since `set_oil_at` sets no flag 1.
- `relax_aquatic_verts`' summation order, which needs a listing read
  before a bit-exact port. ~~The river spline's predicates~~: the spline
  (`RiverOut::add_spline_point`) writes nothing a part reads — every
  record from 5012 to 5118 agrees without it (item 3845, §10).

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
   `sim::map_values`; the `Mapgen:` line's fifth phase); **Great Lakes
   built** (item 3830, the same port on its 7927 grid, the water count's
   first diff; its fifth phase); **East Indies built** (item 3840, the
   same port on its 7927 grid, the border water edge's and the region-size
   factor's first diff; its fifth phase);
6. `Terrain::init` without heights (§10, 734–5012) — **Great Sahara
   built** (item 3809, `sim::terrain_init`; the `Mapgen:` line's sixth
   phase): no cell opens the half-land gate there, so every record to
   5012 stands still but part 4 at 734. **Great Lakes checked to 706**
   (item 3832, no `sim` change; its phase count unmoved at five): the gate
   opens on 876 cells and holds the start dump's 243 HALFLAND cells, and
   720 is the first record a height decides. So on Great Lakes the rest of
   step 6 cannot be checked before step 7's heights reach 720. The
   heights through 706 now agree (item 3835) and decide the 243 cells
   exactly. **Great Lakes built** (item 3843, `sim::halfland`, §16; its
   sixth phase): `_wcoord`'s writes and `mark_halfwater` agree at 720 and
   every record to 5012 on all 185 files. **East Indies built** (item
   3844, no `sim` change; its sixth phase): all 206 files share Great
   Lakes' pattern; from 734 to 5012 seed and parts 2, 5, 6 and 10–13
   agree on the 202 keyed (4,646 records), `SEA_MAP` 3 at 734. The gate
   opens round every island (1,921 and 1,932 cells), and the 706 heights
   decide the start dump's `HALFLAND` cell for cell on all 202 (81,086
   cells);
7. the heights in `sim::single`, with half-land and cliffs (§10, 720–894)
   — **Great Sahara built** (item 3812, `sim::map_heights`; the `Mapgen:`
   line's eighth phase). No checksum part moves for them there, so their
   falsifier is the start dump's height grid, which all 47 files meet.
   **Great Lakes through 706** (item 3835): the shore blend,
   `smooth_tcoord` and `build_coastal_data` on every vertex off the rivers
   of 45 start dumps; the half-land they decide is step 6's (item 3843).
   **Great Lakes built** (item 3850, `sim::map_troughs`, §17; its eighth
   phase): the troughs, then `adjust_for_mountains`; 894 on all 185 files
   and every vertex of the 45 grids. The slope test is inert there: no
   Great Lakes cell carries 0x40, which `great_lakes_at_894` asserts. **East Indies through 706**
   (item 3844): every vertex of the 56 start dumps that print the grid
   (3,252,536; `RIVERLIST` empty, so none is set apart). **East Indies
   built** (item 3851, no `sim` change; its eighth phase): the oil
   terraform, the beach, `make_rivers`, the troughs and
   `adjust_for_mountains` in `Terrain::init`'s order
   (`east_indies_at_894`); seed and parts 2, 5, 6 and 10–13 at 894 on all
   205 files (`east_indies_s_tail_is_894`) and every vertex of the 57
   start grids (3,310,617; `east_indies_s_heights_at_894_are_the_start_dump_s_grid`),
   first run. No river, so no trough; no start dump carries a 0x1000 or
   0x40 cell, and the slope arm is handed no tile;
8. rivers (5012–5118) — **Great Sahara built** before step 7 (item 3811,
   `sim::map_rivers`; the `Mapgen:` line's seventh phase): there the
   phase is height-free (`make_riverbeds` at 5014, the 0x2000/0x2 clear
   at 5116) and draws nothing, since `SAHARA.xml`'s `RIVERLIST` is empty.
   **East Indies built** (item 3846; its seventh phase): `EASTINDIES.xml`'s
   `<RIVERLIST/>` is empty too, so the same height-free phase — 21 and 23
   sources, seed and parts 2, 5, 6 and 10–13 at every record from 5012 to
   5118 on all 202 files (1,010 records; all 202 carry all five lines),
   first run, no sim change (`east_indies_s_rivers_are_5012_to_5118`).
   **Great Lakes built** (item 3845, `make_riverlist`, `astar_river` and
   the `RIVER` rows; its seventh phase): 20 and 22 sources, the river
   draws whole on all 222 traces (173 at 2 nations, 62 at 3), and seed
   and parts 2, 5, 6 and 10–13 at every record from 5012 to 5118 on all
   185 files (925 records), first run
   (`great_lakes_s_rivers_are_5012_to_5118`).

## 16. Great Lakes' half-land: `_wcoord`'s body, 720 → 5012

Item 3843, `sim::halfland`, on the 706 heights §10 checks (item 3835). Each
arm below was read off the decompile, and off the listing where the
decompile was doubtful. **Seed and parts 5, 6 and 10–13 at 720 agree on all
185 Great Lakes files, and every record from 720 to `map.cpp` 5012 does
too** (1,295 records, `great_lakes_s_halfland_is_720_to_5012`), which moves
the `Mapgen:` line's Great Lakes count 5 → 6. On the run: 692 half-land
list entries, four goodies moved, two oils moved, and `verify_land`'s depth
back at 0.

`check_and_set_halfland_wcoord@0084dec0`, on a gated cell
(`CoordInfo.flags & 0x84`), in order:
- **`clear_halfland_wcoord@0084e500`**: the cell's wet tiles dry
  (`World::clear_waterhalf@006b1bd0`), HALFLAND goes, 0x400 goes on all 16
  tiles, and **a cell that was not HALFLAND loses `region2`** (the listing's
  `notw %di; shrl 8; andl 1` at 0x84e534: the decompile's `~flags` is
  right).
- **The region scan, `move[1..25]`**: the first neighbour not HALFLAND whose
  `region` differs from the cell's stops the scan with that region (`near`);
  otherwise each HALFLAND neighbour whose `region2` is not the cell's region
  is kept, the last one winning (`half`).
- **The sub-tile test**: all four corners strictly below −30.0f, compared
  as `Single` bits. Each wet sub-tile is counted; `mark_halfwater` runs only
  when an other region exists (`near`, else `half`), and the region it used
  becomes `near` for the cell's later sub-tiles.
- **Any wet sub-tile: `spot_update_land(1, 0)`**, so the cell turns
  `SANDY`. **None: the dry arm** — `OCEAN` becomes `BASELAND` at
  `COAST_BASE`; flags `& 0xffc3`; `CoordInfo` `& ~4`; 0x4 to the first
  orthogonal neighbour with `land != 0` and no 0x4, which is queued;
  `verify_land`; then the region rescan, which calls `move_ocean_goody`
  when the region changes.
- **The two sweeps**: every cell (y outer, x inner), then the queue until
  it stays empty. A cell may be listed twice; its second visit is a first
  touch again.

**`mark_halfwater@0084ece0`**, sub-tile `(tx, ty)` of a cell:
- On a first touch (not HALFLAND): `move_ocean_goody` runs; the tile takes
  `& 0xffef | 0x20` and the cell 0x100; `region` and `region2` take the land
  and water regions, in that order, swapped when the cell's region is above
  0x40 (it is then the water one). The three lists parts 10–12 walk take
  `(x, y)`, `land` and `land_sub`; then `land` and `land_sub` go to 0 as one
  2-byte write and `verify_land(0, 0)` runs.
- The sixteenth wet tile: HALFLAND clears, `region` takes the water region,
  `region2` 0, `land`/`land_sub` `2/0`, and `verify_land(2, 0)` runs.

**`verify_land@0084d8a0`** walks the eight neighbours against
**`TileSet::is_combo_acceptable@0087abe0`** and recurses into the ones it
rewrote. Its static `depth` is never decremented past a refusal at 500, so
it is carried across the pass. `num_tiles` holds the `BASE` counts of
`BASELAND`, `COASTAL` and `OCEAN`, in the land order `TileSet::init@0087b370`
hands `load_land_data`; only three lands exist, so `num_tiles[3]` is
`None` and refused by name (`Unported::NumTiles`).

**`move_ocean_goody@0084aec0`**: a goody box (`Item::close` takes 0x8000 and
`clear_down`; `Item::init` and `set_down(−3)` go to the first clear cell of
`move[9..25]`) and oil (`set_oil_at` 0, `& ~8`, then the first
`is_clear_land(0x88)` water cell of `move[9..49]` takes oil and 0x8) are
built. **720's part 5 was the oil kill**: `Good::close@0066d860` →
`World::clear_down@006b3ad0` writes `down_who` −1 on the old oil cell. The
Adler difference was +1020 low and 7729 high, exactly `down_who` 0 → −1
on both oil cells, and it was solved before the arm was built. The
good arm (a non-oil good) is refused, `Unported::Goody`; the oil
footprint's tile 0x200 clear is not held (part 6 does not move for it here).

**Parts 10–13's walk**: `num`, `size` (4, doubling: `increase_size` at grow
−1), the grow short 0xffff, the flag byte 0, then the entries — read off
`SimpleArray<int>::walk_data`'s listing at 0x4731a4..0x4731dc and
`TerrainData::TerrainData`. Parts 10–12 agree with the port's lists in that
order, so §3's shapes are diff-backed; its 0x40 offset is not.

After the sweeps: `check_and_set_halfland`'s relabel and its 0x80 tail run
on the half-land result; `create_doobers`' cell write (738) takes 0x8 off a
wet cell; `rebuild_spot_list` has no writer by name or by offset, so the
782–815 drain is empty; and `convert_beach_to_sandy` runs through
`spot_update_land`, entering **0 cells** on Great Lakes (unit-tested only).

**The value diff at 720**: the start dump agrees with ours on `land`,
`region`, `region2`, HALFLAND, the moved goodies and the oil. Its other
differences are later phases', and no record shows them at 720: 47
`land_sub` rows reading 2 in start-city areas (0x1000), the roster's
goodies, and the 0x1000, 0x4000 and 0x80 flags.

**Mutants** (`tools/mutate.py --batch`, on `99ed165a9` and `32e1bcd03`):
24 of 25 held. 17 by the walk; two more by the unit tests already there
(`clear_halfland_wcoord`'s `~flags`, the dry mask `0xffc3`); and five the
walk cannot part on Great Lakes — the scan's break, its last HALFLAND
`region2`, the 0x4 spread's break, oil's third ring and the relabel's swap
— by unit tests written for them. The one left, the first touch keeping
`land_sub`, is unobservable as built: `spot_update_land(1, 0)` follows
every wet sub-tile (3843a).

**Not established**: the good arm of `move_ocean_goody`; the beach's write
on a map that enters it; `num_tiles[3]`; the `Regions` lists
`mark_halfwater` and the rescan keep (no checksum part walks them); the
oil footprint's 0x200 clear (3843a).

## 17. Great Lakes' troughs: `RiversOut::calculate_all`, 5118 → 894

Item 3850, `sim::map_troughs`, whose comments carry each constant and
address. **Diff-backed**: seed and parts 2, 5, 6 and 10–13 at
`Terrain.cpp` 894 on all 185 Great Lakes files
(`great_lakes_s_troughs_are_894`), every vertex of the 45 start dumps that
print the grid and every 0x1000 cell on all 185
(`great_lakes_s_heights_are_the_start_dump_s_grid`), first run. Two
rivers at 2 nations, 59 and 29 sections; 49 cells take 0x1000.

- **The spline points** are `make_riverlist`'s marked cells, each stretch
  its own `start_river` (`MapGrid::rivers`); a border end adds the end
  cell again and one cell off the map. `add_spline_point` stores the cell
  centre `(768c + 384)` and sums the length in cells, `sqrt(dy² + dx²) /
  768`.
- **`ensure_ocean_end@00885400`**: the direction in (normalised); the
  nearest cell of the end's 7 × 7 that is off the map or `OCEAN` and not
  behind it becomes a point, and a step on from it along the **last
  scanned candidate's** offset less the direction another, when on the
  map and `OCEAN`.
- **`calc_spline`**: a clamped cubic B-spline, `(u16)(length · 1.7)`
  steps; `build_normals` with `up = +z`, each vertex the normalised sum of
  its two segments' normals.
- **`create_sections`**: steps of `768 / 1.7` along `next_at_dist`
  (ratio 0.33); each width `(byte · 0.005 + 0.5) · 0.75` from a 50 × 50
  width fractal (smoothing 2, flags 2, seed 0), and once a step's midpoint
  is wet, `min(1.5 · last, 1.5)` from then on. Each section is three
  trough edges, nine vertices each, `width · o · 192` apart, sunk to
  `−60 − profile · −60 + 15` (profile 1, .75, .26, 0, 0, 0, .26, .75, 1).
- **`terraform_from_river_vert`**: a tile not wet, not `SANDY`, with no
  `SANDY`, `OCEAN` or HALFLAND neighbour cell and not a mountain tile has
  its four corners averaged with the depth, its cell listed for the land
  batch and its 11 × 11 listed to smooth. **`set_river_at`** marks each
  of the 3 × 3 tiles with a corner strictly below 0 whose cell passes
  `move_ocean_goody`'s free-cell test.
- **Then** `ensure_45_deg_slope` until its list empties (a neighbour more
  than 79.68 above comes down to 4 under that), two `smooth_tcoord`
  passes, and `spot_update_land_batch(0, riverbase)` — `riverbase` from
  the `TILESET` element.
- **`Terrain::move_goody@0084a3d0`** is `move_ocean_goody`'s body, except
  that moved oil takes the rock bit only from a dry cell and gets its
  rocky patch's terraform (`TerrainOil` is initialised at 493). It moves
  nothing on Great Lakes.

**Inert here, measured** (3850a, mutants that fail nothing): the wet
tile's deepening (the terraform refuses a wet tile, so its depth is read
by nothing), its 30 and its `|i − 4| < 2`; the wet section's `1.5`; the
0x40 and HALFLAND-neighbour tests; `generate_bspline`'s tail test;
`ensure_ocean_end`'s normalisation. One-ulp nudges of 1.7, 0.33 and
0.005 fail nothing, and larger ones fail it.

**Not established**: oil, a goody box or a good in a trough (no trace
meets one); the render-only callees are taken to write no height or part.

**On East Indies** (item 3851, no `sim` change): `EASTINDIES.xml`'s
`RIVERLIST` is empty, so `calculate_all` has no river to calculate, and
894 is the oil terraform (738), the beach (5012) and
`adjust_for_mountains` on 706's heights. Seed and parts 2, 5, 6 and 10–13
at 894 agree on all 205 files and every vertex of the 57 start grids
(3,310,617), first run. Measured (3851a, `tools/mutate.py` on each test):
- **fail it**: the oil terraform switched off (the grid), the mountains
  switched off and their used-vertex mark cleared (the grid),
  `create_doobers_flags`' 0x8 clear (894's part 5), and the slope arm
  handed a tile (894's assertion); the oil terraform let onto the ocean
  hangs the grid test and is counted caught by `mutate.py`'s rule;
- **inert**: the oil's rock-bit test (every oil cell off the ocean carries
  it), the beach (`convert_beach_to_sandy` enters 0 cells on all three
  runs, so its predicate and its `CoordInfo` clear are read by nothing),
  the troughs and `move_goody`'s oil arm (no river), and `Cliffs::init`'s
  slope test (no 0x40 cell on any of 205 start dumps).

## 18. The never-entered seven: entry and falsifiers

Item 3856 read the seven functions `blind::NEVER`'s 256 row named, using
the decompile, the listing and the install's `mapstyles/`, and took no
capture. Every claim here is reading-only (§13's verdict table), unless
it names the trace that enters it.

### 18.1 The base mask and the continent spacing

`Map::make_continents@00694ab0`: **No lobby can enter it** (item 3856, reading-only). `Map::make@0068bc90`
reaches the mask only through vtable slot +0x40, and `Map::new_map@006a0950`
builds one class per style 6–0x1a. Every random-map class overrides +0x40;
`MapConquestSea` inherits `MapConquest`'s. The base entry sits in only two
vtables, `Map::vftable` (no `new_map` case leaves it installed) and
`MapScenario::vftable` (style 0x1a). `MapScenario` overrides +0x64 with
`MapScenario::make@0068c8f0`, which makes no indirect call through +0x40,
+0x64 or +0x68 (listing 0x68c8f0..0x68d1a0). The one dispatcher of +0x64 on
`game_map` is `Setup::build_game@005ac190` (line 271), alongside
`ScenarioEditor::generate_map@009a0950`. The other route to +0x40,
`Map::run_map_script@0068c870` (slot +0x68), has no dispatch on `game_map`
in the export. So the base body is a **`blind::RESIDUE` candidate**, kept
out by the predicate "every `new_map` class but `MapScenario` overrides
+0x40, and `MapScenario::make` never dispatches it". What would kill the
claim: a `cover=1` trace of any lobby whose function list holds
`00694ab0`, or a `+0x68` dispatch on a map found by
`report.py <exe> refs 0068c870`. **Item 3859's blind second reading agrees**
(`docs/audit/2026-11-05-item-3859-blind.md`): no `call`/`jmp [reg + 0x68]`
in the whole listing has a map receiver, and `Map::~Map` is the only other
writer of `Map::vftable`; it is a `blind::RESIDUE` row, its two vtable
slots pinned in the residue's image test.

`Map::force_continent_spacing@0068e6a0`: **Item 3856 corrects that** (reading-only): it has exactly one caller,
`ScenarioEditor::import_image@009a3f30` (call at 0x9a4420; the listing holds
no other `call`, `jmp` or immediate copy of the address). The scenario
editor is cut from v1, so no lobby enters it. It is a **`blind::RESIDUE`
candidate** whose predicate is "only the editor's image import calls it".
What would kill the claim: a `refs 0068e6a0` hit outside
`ScenarioEditor`. **Item 3859's blind reading finds the predicate false**
(`docs/audit/2026-11-05-item-3859-blind.md`): `import_image` has a second
caller, `MapConquest::make_continents@0068dba0` (call at 0x68e142), the
mask of map styles 0x18/0x19, which only `ConquestGame::build_conquest_game`
mints and the skirmish lobby's 23-entry style combo cannot select. The
verdict stands for the lobby. A scenario file's stored style byte
(`ScenarioRead::import@009b1430`) and the console's `map_style` command
~~are not settled, so the function stays an open `NEVER` row (parked 3859a).~~
**Settled by the sixty-sixth pass** (DECISIONS 106 §3): the console's `map`
command clamps its parsed style by `range(0x16)` (the listing at
0x7d9eb6–0x7d9ece), so it writes 0..22 and never 0x18/0x19; and scenario
play is cut from v1 with the editor and the trigger system
(`Game::run_scenario`'s residue row), so a scenario file's style byte
reaches no game a capture runs. `force_continent_spacing`, `import_image`,
`ScenarioEditor::generate_map`, `MapConquest::make_continents`,
`GameSpy::UpdateLobbyFromDTO` and `ScenarioRead::import` are
`blind::RESIDUE` rows.

### 18.2 Warring States' fairness distances

**`MapFairness::calc_distances@0068a1c0`, the Warring States entry**
(reading; **run1122 enters it**, a `cover=1` trace of style 15, seed 12345,
click-free pool, archived and not in `blind::TRACES`).
- It is called once, at 0x699a8a in `MapWarringStates::make_continents@00698b60`,
  inside the start-placement loop (the loop runs when `param_1` ≠ 0, the
  nation count). The loop draws a radius `r % 55 + 193`, takes its integer
  square root, and sets `local_18 = max((avoid_continent + isqrt) / 2, 5)`.
  It then walks a candidate ring from one draw's offset. A candidate first
  passes `Map::land_dist(.., 1) ≥ local_18`, then is scored.
- For each team member, the score is dx = |start x − cx| and dy the same
  for y. With M the larger and m the smaller, it is
  `M + m²/(2M)` (or `(m + 2M)/2` once m ≥ 60000; 0 when M is 0).
  `dists[team] = (float)score · w`, where w is xmm3 = `[0xb69514]` = **1.0f**
  (read off the PE bytes). So every float here is an exact integer below
  2²⁴. Then `lowest_dist` (+0x4c) and `highest_dist` (+0x50) take the
  argmin and argmax team index. Those are the type record's names, though
  the fields hold indices. The compares are strict, `<` from 0x5368d4a5 and
  `>` from 0, so the first in team order wins a tie.
- The loop breaks on `local_34 ≠ 0` or `+0x4c == highest_player (+0x48)`,
  as the decompile has it; that compare has not been listing-read. Then
  `fairness[team] += dists`.

So §11's "float, single" row is **integer-exact on its one call site**.
What would kill the claim: a Warring States checksum trace whose seed and
part 2 at 7807 part from an integer port of this scorer. The lobby is
run1122's at `CHECKSUM` detail 1 with `check_all_level ≥ 14` and a start
`WORLD` dump.

### 18.3 The cliff chain

**Item 3856, the cliff chain read (reading-only; run1118 enters all three).**
- **Which style enters it.** In the install's `mapstyles/`, with comments
  stripped, only `SOUTHWESTMESA.xml` (style 10, `MapSouthwestMesa`) carries
  a live `<GROUPENTRY type="cliffs">`. It has five: two with
  `pattern="player"` (`cliff_face` 1 and 2) and three with
  `pattern="world"` (`cliff_face` 2). `default.xml` has two, but every
  style carries its own `TERRAIN_GROUPS`, so `tileset_groups` never falls
  back to them. The two in `CONQUESTMAP*.xml` are commented out.
- **The entry.** `TerrainGroup::drop_tile@006a2ab0`'s type-8 case calls
  `Cliffs::position_cliff(x, y, template = size − 1, −1, −1, 0,
  FACE_WORLD_CENTER, ..)` (0x6a2c6b). `place_player_group@006a4190` calls
  `verify_defensive_position` (0x6a4a0b), then `position_cliff` (0x6a4a51),
  and on type 8 widens the ring's inner bound by one (`param_3 − 1`).
- **`position_cliff@008a8bc0`.** With `param_6` = 0, a wanted facing is
  read from the 2×2 table `[0xb69908]` by the signs of the offset from
  (`param_4`, `param_5`). With `param_6` ≠ 0, a facing below `FACE_ANY` (2)
  measures against the map centre, and `FACE_ANY` asks for facing 1. The
  wanted facing is −1 when `param_4` is −1, which matches any. Each
  template of the size whose facing matches and that passes
  `Cliffs::verify_placement@008a6310` joins a list. If the list holds two
  or more, one `game_random` draw picks one (`r % n`); one entry is taken
  without a draw. Then `Cliffs::add_cliff@008a6690` queues it.
- **`verify_defensive_position@008a8680`.** It returns 1 for `FACE_ANY`
  with no float. Otherwise it takes the angle between (target − cell) and
  (cell → the cliff's point), each a single-float vector normalised by
  `1/sqrtf` unless len² is 0 or 1. The angle is
  `|(float)acos(dot) · 57.29578|`, where acos is `_libm_sse2_acos_precise`
  in double. It passes when the angle is below **22.5°** (`0x41b40000`)
  for `FACE_WORLD_CENTER`, with the target at (xs/2, ys/2). It passes below
  **17.5°** (`0x418c0000`) for the other facing, with the target at the
  mean of the starts: Σx / `World+0x84` but Σy / `World+0xa0`, two
  different counts as read, not settled.
- **`place_cliff@008a7ea0`.** It is called by `Cliffs::init@008a8020` (and
  by `ConsoleWin::run_cmd`) once per queued cliff, so it is entered exactly
  when `add_cliff` queued one. Over the template's
  `index_to_size[t]`² vertex box at `offtable[t]`, it skips vertices whose
  cell is off-map, carries 0x10, or is dry-skipped (no 0x100 and `land` 1
  or 2). On a cell with no 0x1000 and tile mask bit 4 clear, it adds the
  template's float offset to the height vertex. **No draw.** `Cliffs::init`
  then stamps the cliff's index into each footprint cell's tile +0x6a.
- **What would kill these claims.** run1118's function list already holds
  `008a8bc0`, `008a8680` and `008a7ea0`. A Southwest Mesa lobby at
  `CHECKSUM` detail 1 with `check_all_level ≥ 14` and a start `WORLD`
  dump (run1118's click-free line with the checksum detail added) parts
  at 7859 (`place_all`'s draws) if the template draw is wrong. It parts at
  894 (part 5, the heights) if `place_cliff`'s offsets are wrong.

### 18.4 The slope test

`TerrainOut::has_plus_slope@00869790`: **run1118 enters it** (Southwest Mesa), which is the one shipped style with a
live cliff group (§7). As read: the four corner vertices of the tile,
at stride `4·xs + 1`, are each clamped to ±10000. The test passes when
max − min > 192 · tanf(angle · 0.017453292), with the angle in xmm3
from `assess_wcoord`. It writes min and max through the two out
pointers. What would kill it: a Southwest Mesa start dump with 0x40
cells (zero on all three closed maps' 205 files) whose `blocked` cells
or part 5/6 at 894 part from a port that applies this test.

## 19. The diagonal-transition arm: the predicate that keeps it out

Item 3863 read the arm 3480a and 3493a left open:
`TerrainOut::check_for_diagonal@008595f0` and the `int` overload
`WorldData::is_valid@00471170`, which no `cover=1` trace of the seventeen
concrete styles enters. Its sources are the decompile, the listing and the
install's `Data/tilesets.xml`; no capture, packet or emulator run was
taken. Every claim here is reading-only (§13) unless it names a count. The
counts come from a scratch port of the predicate run over the dumps and
the ported grids. It is not committed, and it backs no claim as a diff
would.

### 19.1 The gate is `check_blocks` alone

`TerrainOut::create_transition_renderables@00862c60` walks the list at
`+0x66f0`: data at `+0x6700`, count at `+0x66f4`, and cursor at `+0x6708`,
which 3480a read as the count. At its end it zeroes the count. For each
listed cell, `TerrainOut::build_trans_flags@00864160` builds a 12-bit word
at shifts 6, 3 and 0. The word is the north field (`+0x49a0` at (x, y))
`<< 9`, then the east field (`+0x499c` at x + 1) `<< 6`, then the
bit-reversed south field (`+0x49a0` at y + 1) `<< 3`, then the
bit-reversed west field (`+0x499c` at x). `+0x499c` holds the vertical
edges, row-major at stride `ys + 1`. `+0x49a0` holds the horizontal
edges, stored by column at stride `xs + 1`. Only
`TerrainOut::check_blocks@0085b3e0` writes the shift-6 word.
`TerrainOut::check_connects@0085b790` and `TerrainOut::check_fills@0085b2c0`
read it as their second argument and write the shift-3 and shift-0 words.
So "empty after the three passes" means empty after `check_blocks`. The
arm's test is an integer one (`movl -0x1c(%ebp),%edx; testl; jne` at
0x862d97), and the decompile's float compare is a misprint.

`check_blocks`, for each side M (N, E, S, W, in bounds): if the field is
full (7; `andl $0xe00; cmpl $0xe00` at 0x85b421) and C → M is 1, the
field goes to 0. If the field is not full and M → C is 1, it goes to 7.
Its closing ORs only spread bits that survived. Here "A → B is 1" is
`TileSet::is_combo_acceptable@0087abe0` with the neighbour's land first,
read as "B stands one step above A". From the body, that holds when the
land is the same and B's sub is A's + 1; when A is `BASELAND` at
`COAST_BASE` and B is `SANDY`; and when B is `OCEAN` and A is any land.
It is never 1 both ways. **The arm is entered only when each side's field
is 0 with M → C not 1, or is 7 with C → M 1.**

### 19.2 Who is listed, and what writes a shift-6 field

`TerrainOut::mark_trans_flags@008625a0` runs row-major from
`TerrainOut::mark_nodes@0085bc20`, which `Terrain::init@00850f70` calls at
line 759. It also runs over the update lists of
`Terrain::spot_update_land@0084fa50` and
`Terrain::spot_update_land_batch@0084d540`, which reset those cells' edges
to 0x38 first: shift 6 at zero.
- **(A)** An orthogonal M with M → C 1: `ArrayBase<int>::add_unique@0047adb0`
  lists C. It returns −1 for a cell already listed (`cmpl $-0x1; je` at
  0x862721), so only the **first such M in the order N, E, S, W** is acted
  on. That M gets the four shift-3 fields, then
  `TerrainOut::check_for_tendrils@00859eb0`. If that returns 0,
  `TerrainOut::set_edge_x@00859500` or `set_edge_y@00859410` writes 7 at
  shift 6 on the edge toward M.
- **(B)** No in-bounds orthogonal with M → C 1, and at least one diagonal
  with D → C 1: C is listed if it is not already, and `check_for_tendrils`
  runs.

Only `set_edge_*` writes shift 6: an OR, which also clears the same bits
at shift 3. Its values are 7 from (A), and 4, 1 and 5 from
`check_for_tendrils`. Nothing clears a shift-6 bit between
`TerrainOut::create_nodes@0085bc70`'s `memset` and the pass. **A full
field is (A)'s 7, and nothing else writes one.**

### 19.3 The predicate

- **An (A) cell never reaches the arm.** `check_blocks` makes the first
  M's side 7, and only C → M 1 clears it, which cannot hold since M → C
  is 1.
- **A (B) cell**, whose four orthogonal flags are all 0, gets a nonzero,
  non-full value from `check_for_tendrils` on both flank edges of every
  corner whose diagonal D has D → C 1. A NW corner writes 4 on W and N; NE
  writes 4 on E and 1 on N; SW writes 1 on W and 4 on S; SE writes 1 on E
  and on S. `check_blocks` clears them only if each flank edge is full
  and the flank F has C → F 1. That needs (A)'s 7 from F, with C as F's
  first orthogonal.
- So the arm needs a chain **D → C → F**, each step one up, with D
  orthogonally adjacent to F, on both flanks of the corner.
- Every shipped tileset has four `BASELAND` bases, one `SANDY` and one
  `OCEAN`. `COAST_BASE` is 3 on sixteen of them and 0 on `SW_Desert`.
  The chains the numbers allow are four:
  - `(B_i, B_{i+1}, B_{i+2})` and, at `COAST_BASE` 3, `(B_2, B_3, SANDY)`.
    In both, D and F are **mutually unacceptable**: `is_combo_acceptable`
    is 0 both ways.
  - `(B_i, B_{i+1}, OCEAN)` and `(B_cb, SANDY, OCEAN)`. In both, D is a
    `BASELAND` cell **orthogonally beside an `OCEAN`** cell.
- **The predicate: the arm is entered only on a grid that holds either
  (P1), two orthogonal neighbours mutually unacceptable, or (P2), an
  orthogonal `BASELAND`–`OCEAN` pair.** On the ocean chains, D → F is 1
  as well. The first-hit order in (A) then leaves only the **south-west**
  corner: a flank whose north neighbour is D, or whose east neighbour is
  D, acts toward D before C (the east neighbour comes before its south
  neighbour C). The scratch port bears this out. In a 9×9 ocean with one
  `SANDY` cell and `BASELAND` sub 3 at its south-west diagonal, the cell
  reaches the arm under `COAST_BASE` 3. With the land at any of the other
  three corners, it does not.

### 19.4 No grid on file holds P1 or P2

- **The initial pass.** `Terrain::init` builds the transitions at line
  760, before `Terrain::convert_beach_to_sandy@0084b8e0` at line 904. On
  the ported pre-beach grids of the four Great Lakes and East Indies
  lobbies, as the rondata half-land tests build them, there are 0 P1 and
  0 P2 pairs (scratch count).
- **The start dumps.** The disk holds 86 distinct start grids (styles 7,
  9, 14, 18 and 22). Under each grid's own `COAST_BASE`, they hold 0 P1
  and 0 P2 pairs. The scratch port lists 1,068–1,970 cells per grid,
  167–532 of them (B), and sends none to the arm. That agrees with 3480's
  five styles and 3493's twelve.
- **The beach.** It only turns `BASELAND` into `SANDY`, so P2 holds. Each
  conversion goes through `spot_update_land`, whose
  `Terrain::verify_land@0084d8a0` runs before the re-mark: around a
  `SANDY` centre, it rewrites an unacceptable neighbour to `BASELAND` at
  `COAST_BASE`.
- **Mid-game.** `Terrain::object_placed@00850c40` rewrites only
  `BASELAND` cells under a footprint, and only to `BASELAND` at the
  tileset's `weatherbase` (0–3), so P2 holds. P1 can break, for instance
  `weatherbase` 0 beside sub 2, and `verify_land` repairs it around the
  centre and recurses into each cell it rewrote.
- `WorldData::is_valid@00471170` has two callers in the listing: the four
  call sites in `check_for_diagonal` (the decompile prints one) and
  `MapScenario::make@0068c8f0` at 0x68cafc and 0x68cc24, which §18.1
  keeps out of every lobby.

**Verdict: unreachable while P1 and P2 hold.** ~~The next item is the
predicate's blind second reading.~~ The second reading
(`docs/audit/2026-11-05-item-3864-blind.md`) re-derives P1 ∧ P2, the
south-west corner and the tileset arithmetic independently. Its blind
part claimed more and was wrong twice: on which grid the initial pass
marks (the pre-beach grid, which §19.4 counts) and on `verify_land`'s
mid-game hold (3863a). So the arm and the `is_valid` row stay open, not
`RESIDUE`, until 3863a's flood is bounded below depth 500. **The
sixty-sixth pass** (DECISIONS 106 §3): the developer console is the
oracle's own instrument (`crate::golden` models its `add`, `resource`,
`tech` and `age`; every chapter is staged through it), so a row the
console can reach is open, never residue on scope (Loop 3473) — the arm
stays open and item 3870 takes 3863a's bound first, the console-planted
contact second. `TerrainOut::generate_transitions@00861960` is referenced
nowhere in the image (the pass's scan; the byte before it is the
preceding function's own `ret`) and is a `RESIDUE` row; `River::calculate`
stays open, entered by the console's two river arms and
`TerrainMap::river_stop`.

### 19.5 What would kill it, and what is not established

- **Killers.**
  - A `cover=1` trace that enters `008595f0` on a grid with no P1 or P2
    pair kills the predicate.
  - A start dump (`WORLD` ≥ 3) that holds a P1 or P2 pair kills §19.4's
    "no grid", and names the lobby to trace.
  - The frame-0 packet, run 1489, is reserved for a tie: two edge arrays
    and the list, at `Terrain.cpp` 815, with a listed cell whose shift-6
    word `check_blocks` empties.
- **The loophole** (parked 3863a). `verify_land`'s early return past
  depth 500 (line 40) skips the decrement at line 175. After one flood
  that deep, every later call returns at once, so a later placement can
  leave a P1 pair, the `(B_i, B_{i+1}, B_{i+2})` chain. Whether a v1
  game's floods ever reach 500 is not read. ~~"every later call returns
  at once"~~: each early return leaks one, so the headroom shrinks by
  the count of capped frames (§19.6).
- **Not walked.** That map generation guarantees P1 and P2 before line
  760 is measured on four lobbies and 86 dumps, not read. The writers of
  `land` and `land_sub` before the transitions are not walked for it.
- **Out of scope.** `ConsoleWin::run_cmd@007d6a70` and the scenario
  editor also reach `spot_update_land`; both are cheat or cut.

### 19.6 The flood's depth, writer by writer (item 3870)

The sources are the decompile and the listing. The counts come from a
scratch model of the flood as read, run over the first `WORLD` block of
the 43 distinct start grids on disk (Great Sahara 23, Great Lakes 13,
East Indies 5, Himalayas 1, Southwest Mesa 1). The model is not
committed. Every claim here is reading-only.

- **The counter leaks; it never sticks.** `depth` is the global at
  0xee1224, which has three references in the image, all in
  `Terrain::verify_land@0084d8a0`: the load and `incl` at entry, and the
  `decl` at 0x84dd43. A frame entered past 500 (`cmpl $0x1f4; jg
  0x84dd49`) skips the `decl`, but its callers still take theirs. So
  after a flood, `depth` holds the number of capped frames, L, and every
  later flood runs with 500 − L levels. Nothing resets it. *Falsifier*:
  a packet read of 0xee1224 at any frame (0 means no flood was ever
  capped). No dump prints it.
- **One frame per rewrite.** A nested frame enters only a neighbour its
  caller rewrote, or the centre itself after `verify_land` sets it to
  `BASELAND` at `COAST_BASE`. That second case happens at most once per
  cell, because `SANDY` and `OCEAN` then accept the centre. So a flood's
  depth is at most one plus its rewrites. Below the top frame, the centre
  is always `BASELAND`.
- **Q, the precondition.** No two `BASELAND` 8-neighbours differ by 2 or
  more, and every `BASELAND` cell beside `SANDY` sits at `COAST_BASE`.
  All 43 start grids hold Q: 0 violations under each grid's own
  `COAST_BASE`, as 3864's count found. A flood that is not capped
  restores Q around every cell it rewrote, because each rewrite is
  followed by that cell's own frame. *Falsifier*: a `WORLD` dump (`GLOG_TERRAIN`)
  with a pair that breaks Q.
- **Locality, proved for one write on a Q grid.** Write value v at a
  cell, and let F(n) be the old value L(n) clamped into
  [v − d, v + d], where d is the Chebyshev distance to the write. L and
  F are both 1-Lipschitz. A rewrite moves a cell one step toward a
  neighbour at least 2 away, so every cell stays between L(n) and F(n).
  Subs run from 0 to 3, so nothing at d ≥ 3 changes. A `BASELAND` write
  (`Terrain::object_placed@00850c40` at `weatherbase`, `mark_halfwater`'s
  sub-0 call) changes at most the 5×5 around it. A `SANDY` or `OCEAN`
  centre (`Terrain::convert_beach_to_sandy@0084b8e0`, the two arms of
  `Terrain::check_and_set_halfland_wcoord@0084dec0`, and
  `Terrain::mark_halfwater@0084ece0`'s ocean call) first sets its
  neighbours to `COAST_BASE`, and the changes stay in the 7×7 around it.
  The rewrite count inside that window is measured, not proved. Over
  every cell of the 43 grids, the deepest flood of each writer is: a
  `weatherbase` write 7 (v = 0..3), a `SANDY` centre 26, an `OCEAN`
  centre 17. Random Q grids of 11, 13 and 25 cells a side give the same
  depths and the same radius (3), whatever the grid size.
- **Mid-game holds.** `Wall::start@0063e810` is the v1 mid-game writer.
  It calls `object_placed`, which runs one top-level `spot_update_land`
  flood per footprint cell. Each such flood is a single write on a Q grid
  (induction from the start's Q, while L = 0), so it stays far below 500.
  **3863a's route needs L ≥ 494 when the game begins.**
- **The batches have no grid-free bound.**
  `Terrain::spot_update_land_batch@0084d540` writes every listed cell
  before it verifies any of them. So the first flood finds the later
  cells' unrepaired neighbours and runs along the list. Two batches run
  in `Terrain::init@00850f70`: the half-land list at line 867 (`SANDY`
  0) and `RiversOut::calculate_all@00888820` → `River::calculate@008858e0`
  (`BASELAND` at `riverbase`). In the model, a 544-cell `SANDY` batch
  through ground two steps off `COAST_BASE` caps at 502–531 and leaves
  1,698–2,408 Q violations behind. A 60-cell diagonal reaches 356. Only
  the ground along the list keeps these floods shallow, and the
  pre-river ground is not on disk. The 43 start grids' Q is consistent
  with L = 0 but does not prove it: a capped flood need not leave a
  violation.

**Verdict: no bound by reading.** Every single-write flood is bounded,
and the mid-game is bounded given L = 0 at the start. The start's two
batches are not bounded. The rows stay open: a second reading would
have to re-derive the leak, the locality lemma and the batch chain, and
a bound would still owe the start's L.

**The console-planted contact.** `ConsoleWin::run_cmd@007d6a70` case 99
is `weather`, a console-only command. It calls `spot_update_land` at the
cursor cell with land `BASELAND` and sub 3, whatever the cell held, so
on an ocean cell it plants fertile land in the sea. Case 0x65 is
`river`: its `grow` adds a control point at the cursor, its `calc` runs
`River::calculate`, and its fourth word runs `PathFinder::astar_river`
from the cursor. **All three aim at `mouse_coord`.** A `.cmd` line runs
with `no_mouse` = 1, which leaves the cursor stale: (0, 6) raw, the cell
(0, 0) (ORACLE.md, "The cheat channel"). The cell (0, 0) is open ground
on Great Sahara, Great Lakes and Southwest Mesa, and open sea on East
Indies. None of those makes §19.3's shape.

**Great Sahara cannot host the contact.** Its 23 start grids hold no
`OCEAN` and no `SANDY` cell, so only P1 is open there, and only through
a capped flood. The south-west shape for `weather` is a `SANDY` cell C
with `OCEAN` on its west, its south and north of its west flank, and no
`BASELAND` at `COAST_BASE` on its north or east. Planted at C's
south-west diagonal, `weather`'s sub 3 survives its own flood. The model
finds the cells (41, 42) and (16, 45) on all 13 Great Lakes start grids,
and 13 cells on East Indies, among them (2, 17) and (43, 17). The
`coverage` capture is `weather` with the cursor on (41, 42) in Great
Lakes' default lobby, `cover=1`, then the same frame's `WORLD` block
(`GLOG_TERRAIN`). Entering `008595f0` confirms the predicate, and a
non-empty word at C kills it. A `.cmd` line cannot aim the cursor.
Either the click lane hovers the cell, or the tracer stores
`mouse_coord_x/y` (`ConsoleWin` +0x518/+0x51c) before the line, which
is tooling this item does not build.
