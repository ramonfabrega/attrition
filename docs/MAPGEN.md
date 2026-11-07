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
Himalayas, Old World and Southwest Mesa read the same; for Mesa
(`MapSouthwestMesa::make_continents@0069bb20`) the listing is the same
instruction for instruction (item 3869), and run1493's 7807 agrees. **This is why GS's
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

**Southwest Mesa, diff-backed since item 3872** (checked by
`rondata::mapgen::tests::southwest_mesa_s_zero_draw_passes_are_the_traced_7835_to_7854`,
Great Lakes' walk, which now takes a one-entry `TILECHANCE` list's tileset
as Great Sahara's test does): from the ported 7807 grid (§5), run1493's
one file carries the four records, one each, and the seed (2,355,140,353,
held from 7807), part 2 (292,424,319) and part 5 agree at all four on the
first run. Part 5 is 1,267,075,849 at 7807, 2,334,202,137 at 7835, 7843
and 7846 (every cell `region` 1, Great Sahara's value), and 969,480,808 at
7854 under `SW_desert` (clump 3, 80/5/5/10, Great Sahara's numbers). The
mask is Great Sahara's, all land: **one land region and no sea** at both
region passes. So of eleven mutants (`tools/mutate.py --batch` on
`49243ec26`), five hold: the first land label moved to 2 (at 7835), and
at 7854 `fill_fertile`'s `>=` as `>`, its bucket one lower, its fractal
read transposed and the clump factor + 1. Six fail nothing, each for the
mask's reason: `fix_diag_land` skipped, `fix_lakes` at `> 6`, the COAST
marking and the thinning skipped (no water cell), the land record search
never advancing (one region), and the second `clear_all` leaving the
cells' `region` (every cell already 1). Those arms stay held by Great
Lakes' and East Indies' walks.

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
draw when two or more templates fit), through `drop_tile`'s type-8 case and
`place_player_group`'s cliff arm, and mark their footprints 0x40 for every
later group's predicates. `sim::terrain_groups` ports the chain (item 3875,
§18.3), diffed on Southwest Mesa at 7859; the angle test of
`Cliffs::verify_defensive_position@008a8680`, the one libm transcendental
on a data path (§11), is a `SEAM` no shipped style reaches.

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
  (`mountains`). The cliffs' pass, `Cliffs::set_edge_flags@008a85e0`,
  walks `cliff_mining_data` (`Cliffs` +0x40: count +0x44, list +0x50 at
  0x8a85f0..0x8a85fd; each `CliffMiningData`'s `tcoords` at +0x1c, count
  +0x20, list +0x2c), not the queue `add_cliff` appends to
  (`deferred_cliffs`, +0x7a4, count at 0xe89658). Only `Cliffs::init@008a8020`
  fills that list, after `CliffsOut::close`, inside `Terrain::init` after
  7945 (`walk_data`, a load, is the only other writer). So at 7937 the pass
  has nothing to walk, and the port has no cliffs' pass (item 3882).
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

**Checked on Southwest Mesa** (item 3882, no sim change): from 3878's
7927 grid, the first with cliff cells (292 0x40), the whole stream from
`place_all` through the last value draw is run1493's trace, 6,161 draws
with 3,600 value draws, and seed and parts 2, 5 and 6 agree at 7937
(3871037036 / 292424319 / 253482500 / 644716173) and 7945 (718896060 /
292424319 / 3822669751 / 644716173) on its one file, the one record of
each it carries, first run
(`southwest_mesa_s_values_are_the_traced_stream_and_7937_and_7945`). Of 18
mutants on `1ceb95bb1`, 13 turn it red. Three are the cliff cells' own
arms: the 0x200 ring's `0x70` as `0x30`, `get_land(…, 1)`'s 0x40 as class
5, and the square's rough `0x70`. The others are the forest cells' 0x8000
pass, `has_gather_access`' blocked test, the forest tiles' 0x38, the east
border's unforest, the walk transposed, the draw's `% 10`, the timber and
metal factors, and two made to fire on every cell (the 0x1000 centre test
inverted, the `size / 16` threshold raised), which say only that each
branch's write reaches the parts: the grid has no 0x1000 tile and one
region of 3,600 cells, so neither decides a cell here. Five fail nothing, each measured on the 7937 grid as inert (3882a):
Mesa places no mountain range, so `Mountains::set_edge_flags` walks
nothing; `eliminate_dead_mount_spots` finds no dead spot; no cell is wet,
so the `÷10` water factor never fires; every cell counts at least 10 food
slots, so `> 4` and `> 5` agree; and none of the 4,672 tiles of the cliff
cells has a gather mask at 7937, so even a filled cliff list would add no
0x8000 here. The cliffs' 0x8000 therefore stays reading-only, now because
its list is empty and its tiles unmarked at 7937, not because no map has
cliffs.

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
  mutant there too (item 3825); the same four on Southwest Mesa's one
  file, the label and `fill_fertile` held by a mutant there and the water
  arms inert on its all-land mask (item 3872); the `region2` pass and the adoption arm are
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
- §7's tail and §8's `place_resources` on Southwest Mesa: draw for draw on
  run1493's trace (916 draws), and seed and parts 2, 5 and 6 at 7861,
  7865, 7870, 7877, 7880 and 7927 on its one file (item 3878), the first
  grid with cliffs. The player predicate's cliff refusal is held by a pair
  mutant with the goody box's `0x70`, which covers it alone; the nonplayer
  side's, the class-5 reading's and the water arms stay inert (3878a);
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
- §9's values on Southwest Mesa: draw for draw on run1493's trace through
  the last value draw, and seed and parts 2, 5 and 6 at 7937 and 7945 on
  its one file (item 3882), the first values over cliff cells. The 0x200
  ring, `get_land`'s class 5 and the square's rough test on a 0x40 cell
  are diff-backed here; the 0x1000 tile and the region-size factors are
  not (no 0x1000 tile, one region). The cliffs'
  0x8000 stays reading-only: its pass walks a list nothing fills before
  7945, and no cliff cell's tile has a gather mask at 7937 (§9). The ranges' 0x8000,
  the dead spots, the water factor and the food bit's threshold also
  run but cannot part (3882a);
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
  arm and the beach's write are not reached (3843a). On Southwest Mesa,
  the first all-land grid with cliff cells: no cell gated, seed and parts
  2, 5, 6 and 10–13 at all 23 records from 734 to 5012 on its one file,
  part 4 at 734 its `SEA_MAP` 0 (item 3885); the wet pass's mask, the
  coast's 0x400, the 0x20 and 5×5 rings stay inert there (3885a);
- §10's `make_rivers` on Great Sahara: `make_riverbeds`' sources and the
  three clears, with seed and parts 2, 5, 6 and 10–13 at every record from
  5012 to 5118 on 105 files (item 3811), and on East Indies' 202 (1,010
  records, item 3846), where the start-city test is held by the walk; the square's water and oil arms, the radius's `− 1`
  and `find_all`'s relabel are unit-tested only (3846a). `make_riverlist`
  and `astar_river` on Great Lakes (item 3845): draw for draw on all 222
  traces (173 on 209, 62 on 13) and every record from 5012 to 5118 on 185
  files (925), the 5076 bank clear included; 26 of 47 mutants held, the
  21 that failed nothing are 3845a; on Southwest Mesa's
  cliff grid, run1493's five records with 40 sources (item 3888): the
  cliff cells pass the square's class-2 refusal as class 5, held by a
  planted 0x40 refusal; the entry and 0x1000 clears, the cell's mountain
  tile and 0x1, the eight's mountain and rock, class 2, the radius's
  `− 1` and `find_all` have nothing to decide there (3888a);
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
whose falsifiers are 7859 and 894 (§18.3, §18.4). **Taken: run1493**
(item 3869), whose trace holds `position_cliff`'s eleven template draws
(site 0x8a8d2f) in `place_all`'s segment. **Diffed by item 3875**:
`position_cliff`, `verify_placement`, `add_cliff`, `is_clear_land` at masks
2 and 0x22, the template masks and both callers' arms are diff-backed by
run1493's 1,645 `place_all` draws and 7859's part 5;
`verify_defensive_position`'s angle stays reading-only. **Diffed by item
3890**: `place_cliff`, `assess_wcoord`, `has_plus_slope` (its threshold
pinned by a native-`f32` oracle), the mining sweep, `set_edge_flags` and
`move_goody`'s good arm, by 894's parts 5 and 6 and every vertex of
run1493's start grid; `place_cliff`'s four skips, `assess_wcoord`'s two
refusals, the clamp, the strict `>`, the `& 0xfffd`, the closing
`move_goody`, the `n != 0` and the good arm's south-forest test are
handed nothing there and rest on the reading (§15 step 7, 3890a).

**Item 3863's verdict, reading-only** (§19): `TerrainOut::check_for_diagonal@008595f0`
and `WorldData::is_valid@00471170` are kept out by a predicate. The arm
needs P1 (orthogonal neighbours mutually unacceptable) or P2 (an orthogonal
`BASELAND`–`OCEAN` pair). Every claim of §19.1–§19.3 is a reading. §19.4's
counts (0 P1, 0 P2 on 86 start grids and on the four ported pre-beach grids)
come from a scratch port, which is not pinned. The predicate is owed a blind
second reading before the row is called `RESIDUE`. **Item 3876** (§20, "The offline search"): a model of
one plant's re-mark, `tools/gamelog/plant.py`, reproduces run1495's 81
printed writes. That is the one diff-backed claim. Its passes (62 single
plants on 18 of 68 grids, East Indies (2, 17) first) are reading-only until
a take enters `008595f0`.

**Item 3908's** (§22): the eleven styles' `make_continents`,
`Map::eliminate_edge_canals` and `Map::find_region_centroid` are entered on
eleven joined traces (measured, `blind::TRACES`); the lobby ids, the callers,
the `set_map` table and the `!eval` route to `check_starting_loc` are
reading-only, the route's falsifier a `cover=1` take §22.3 writes.

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
   phase); **Southwest Mesa built** (item 3869,
   `sim::mapgen::southwest_mesa_make_continents`, Great Sahara's listing
   at 0x69bb20; its first phase): the four draws site and word on
   run1493's trace and 7807's seed, part 2 and part 5 on its one file,
   first run. run1493 (the default lobby at `--map 10`,
   `--dump-all-start`) carries every checkpoint 7805..7945 and
   `Terrain.cpp`'s, so the chain's later steps need no capture;
2. the zero-draw passes (§6, 7854) — **Great Sahara built** (item 3798,
   `sim::mapgen`; the `Mapgen:` line's second phase); **Great Lakes
   built** (item 3819, the same passes on its 7807 grid, the water arms'
   first diff; its second phase); **East Indies built** (item 3825, the
   same passes on its 7807 grid, twelve islands, the land record search's
   first diff; its second phase); **Southwest Mesa built** (item 3872,
   the same passes on its 7807 grid, run1493's four records, first run;
   its second phase; an all-land mask, so the water arms are inert there);
3. `place_all` (§7, 7859) — **Great Sahara built** (item 3803,
   `sim::terrain_groups`; the `Mapgen:` line's third phase); 7861–7870's
   `nubify_forest`, `change_*_base` and `fix_transitions` — **Great Sahara
   built** with step 4 (item 3805); **Great Lakes built**, `place_all`
   and its tail together (item 3822, the sea-oil disc and
   `change_coast_base`'s first diff; its third phase); **East Indies
   built** (item 3828, the disc's reach and `check_player_forest`'s
   first diff; its third phase); **Southwest Mesa built** (item 3875,
   `place_all` alone with the cliff chain of §18.3, the chain's first
   diff; its third phase): the first parting before the port was draw 25,
   theirs `position_cliff`'s 0x8a8d2f (word 0x76715ae4) where ours drew
   the next player's ring (0x6a42de), as booked. Ported, run1493's 1,645
   draws are whole and 7859's seed 2248721384, part 2 292424319 and part 5
   1583465931 are both sides', first run: eleven cliffs (six at size 4,
   three at 3, two at 1), 292 cells 0x40. **The tail built** (item 3878,
   no `sim` change, with step 4): `check_player_forest`, `nubify_forest`
   and the base passes with `fix_transitions` on the first grid with
   cliffs; the seed and parts 2, 5 and 6 at 7861, 7865 and 7870 agree on
   run1493's one file, first run (7870: seed 2562479304, part 5
   403494480), and its 864 draws are the trace's;
4. resources (§8, 7927) — **Great Sahara built** (item 3805,
   `sim::map_resources`; the `Mapgen:` line's fourth phase); **Great Lakes
   built** (item 3826, the same port on its 7870 grid, the water arms'
   first diff; its fourth phase); **East Indies built** (item 3831, the
   same port on its 7870 grid, the Whales' and `nonplayer`'s first diff;
   its fourth phase); **Southwest Mesa built** (item 3878, no `sim`
   change; its fourth phase): on the 7870 grid above, the seed and parts
   2, 5 and 6 at 7877, 7880 and 7927 agree on run1493's one file, and
   the 916 draws from the tail's first to the first `compute_val` (864 of
   the tail, 52 of the resources) are the trace's, site and word, first
   run (`southwest_mesa_s_tail_and_resources_are_the_traced_stream_and_7861_to_7927`;
   7927: seed 3871037036, part 5 2072961574, part 6 724224914; 29
   objects). The six records 7861..7927 are all the file carries in the
   window. **The cliff cells are refused twice**: of the 796 cliff-cell
   tests the predicates make (a probe, reverted whole), 121 are the player
   predicate's, every one a goody box (543), which the `0x1878` mask and
   the box's own `0x70` both refuse; the other 675 are the nonplayer
   predicate's (goody boxes, and types 18 and 14 three times), and each
   also fails `bounds_ok` or the spacing. So each of the four cliff
   refusals (the two masks, `get_land(…, 1)`'s 0x40 as class 5, the box's
   `0x70`) is inert alone; the player pair together (`0x1878` and `0x70`)
   holds — one box lands on a cliff cell and part 5 at 7927 parts
   (256434726 against 2072961574); the nonplayer three together stay
   inert (3878a). Of 20 single mutants on `7c56b3734`, 6 hold
   (`change_base`'s `<`, `step_down`'s keep list, `nubify_transitions`'
   `% 3 − 1` and its run of five, `nubify_forest`'s two-sided run,
   `has_south_forest`'s FOREST bit), and the other 14 have nothing to decide on
   this map, each measured on the 7870 grid: no ocean, COAST, `land` 2,
   0x1000 cell or class-1 tile, one land region, and city discs of 7 and
   5 FOREST cells, so `check_player_forest` never patches (3878a);
5. values (§9, 7937 and 7945) — **Great Sahara built** (item 3807,
   `sim::map_values`; the `Mapgen:` line's fifth phase); **Great Lakes
   built** (item 3830, the same port on its 7927 grid, the water count's
   first diff; its fifth phase); **East Indies built** (item 3840, the
   same port on its 7927 grid, the border water edge's and the region-size
   factor's first diff; its fifth phase); **Southwest Mesa built** (item
   3882, no `sim` change; its fifth phase): on 3878's 7927 grid, the seed
   and parts 2, 5 and 6 at 7937 and 7945 agree on run1493's one file,
   which carries one record of each, and the 6,161 draws from `place_all`
   through the last `compute_val` are the trace's, site and word, first run
   (7945: seed 718896060, part 5 3822669751, part 6 644716173). The cliff
   cells' three value arms hold, and five mutants are inert by the grid,
   each measured (3882a; §9);
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
   cells). **Southwest Mesa built** (item 3885, no `sim` change; its
   sixth phase): Great Sahara's case on an all-land mask with 292 cliff
   cells, so no cell is gated. On run1493's one file, part 4 at 734 is
   `SEA_MAP` 0, and seed and parts 2, 5, 6 and 10–13 stand at 7945's
   values through all 23 records to 5012, first run. 7 of 12 mutants
   hold (3885a);
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
   0x40 cell, and the slope arm is handed no tile. **Southwest Mesa
   built** (item 3890, `sim::cliffs`; its eighth phase, the last arm no
   chain had run): step 8's grid, the heights in `Terrain::init`'s order
   (no river, so no trough) and `Cliffs::init` (§18.3, §18.4). On
   run1493's one file the seed and parts 2, 5, 6 and 10–13 at 894 agree,
   part 4 held from 5118 (`southwest_mesa_s_cliffs_are_894`), and every
   vertex of its start grid (58,081), its 253 0x40 cells and the moved
   goods' cells agree
   (`southwest_mesa_s_heights_at_894_are_the_start_dump_s_grid`). Eleven
   cliffs placed; 4,672 tiles handed to the slope test, 1,827 passed; 39
   cells cleared of 0x40; 14,869 `move_goody` calls moving oil (33, 34) →
   (32, 32) and two goods, one twice (`move_goody`'s good arm, first
   reached); 3,758 mining tiles, 1,435 given 0x8000. The first run parted
   parts 5 and 6: the templates' height byte is the TGA's **red** byte,
   not its byte 0 (§18.3). 18 of 31 mutants hold; the other 13 arms are
   handed nothing on this grid, each count printed and asserted nought
   (`sim::cliffs::Inert`; 3890a);
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
   (`great_lakes_s_rivers_are_5012_to_5118`). **Southwest Mesa built**
   before step 7 (item 3888, no `sim` change; its seventh phase):
   `<RIVERLIST/>` empty, so Great Sahara's height-free phase on step 6's
   grid. run1493's one file carries all five lines; 40 sources, and seed
   and parts 2, 5, 6 and 10–13 at all five records agree, first run
   (`southwest_mesa_s_rivers_are_5012_to_5118`; part 5 3822669751 →
   2939507895 at 5014 and 5076, back at 5116 and 5118). **The cliff bit
   reaches one arm**: `get_land(…, 1)`'s 0x40 as class 5 in the 7×7
   refusal, where 22 sources have a cliff cell and class 5 is not 2. No
   forest cell has a cliff cell in its eight and no cliff cell a mountain
   tile, so the threshold count never meets one. A planted refusal of
   0x40 in the 7×7 leaves 25 sources and parts 5014, so the original does
   not refuse there. 5 of 18 mutants held (3888a).

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
+0x64 or +0x68 (listing 0x68c8f0..0x68d1a0). ~~The one dispatcher of +0x64 on
`game_map` is `Setup::build_game@005ac190` (line 271), alongside
`ScenarioEditor::generate_map@009a0950`.~~ **Corrected by item 3891**
(`docs/audit/2026-11-05-item-3891.md`, row 2): line 271 runs only when
`param_2` (`is_scenario`) is 0, on `new_map(World::map)`, which the lobby and the
console never set to 0x1a. Style 0x1a is minted by
`ConquestGame::build_conquest_game` (line 75, `(map_style == 8) * 2 + 0x18`)
and dispatched there at 0x797e7d, so `MapScenario::make` is reached by a
custom-map Conquer the World and by the editor. Both are cut from v1, and the
function is a `blind::RESIDUE` row. The other route to +0x40,
`Map::run_map_script@0068c870` (slot +0x68), has no dispatch on `game_map`
in the export. ~~It is an open row.~~ **Two readings agree, item 3891** (row 3):
none of the image's +0x68 dispatches has a Map receiver, and it is a
`blind::RESIDUE` row. So the base body is a **`blind::RESIDUE` candidate**, kept
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
  `Cliffs::position_cliff(x, y, template = range − 1, −1, −1, 0,
  FACE_WORLD_CENTER, ..)` (0x6a2c6b; `range` is its sixth argument).
  `place_player_group@006a4190`'s arm takes template `t = size − 1` (the
  ring's `param_3 − 1`, item 3856's "inner bound" misread), stores
  `t == 4` at `0x10(%ebp)` (0x6a49c9), and unless `t` is 4 calls
  `verify_defensive_position` (0x6a4a0b) first; then `position_cliff(x, y,
  t, start_x[p], start_y[p], t == 4, cliff_face)` (0x6a4a51), returning 1
  on success. `cliff_face` is parsed for type 8 alone, −1 read as 0
  (`init_terrain_data@006a6540`). **Diff-backed** (item 3875, run1493 at
  7859); mutating `t == 4` or the template's `− 1` breaks the stream.
- **The templates** (`CliffsOut::init_templates@008a71a0`). Each
  `Data/Cliffs.xml` `CLIFFS/CLIFF` in order loads its `path` TGA (all RLE,
  32 bits, bottom-left origin; row 0 is the image's top). The size index
  is the width (64 → 4, 32 → 3, 16 → 2, 8 → 1, else 0), not the `size`
  attribute; the facing is `orientation`. A texture with any zero alpha
  covers the cells holding one. Otherwise a cell is covered when the heights
  `red · (1/255) · 480` (the engine's `PF_32` texel byte 0 is the TGA's
  byte 2, red; item 3890, by every vertex at 894) (two `mulss`) over its 6 × 6 pixel window
  (`4c − 1 ..= 4c + 4`, 0 off the texture) span more than 192, in single
  precision (`cliff_cells`, by `Single`). Box offsets `offtable@00af2fb0` =
  `[0, −1, −2, −4, −8]`. Row order, the transposition and the alpha arm are
  diff-backed. The height arm reaches six 32-px templates that no run1493
  verdict turns on, so it is held by a native-`f32` oracle test
  (`the_cliff_mask_s_height_arm_is_single_precision`).
- **`position_cliff@008a8bc0`.** With `param_6` = 0, a wanted facing is
  read from the table `[0xb69900]` = `[1, 2, 4, 3]` at `2·[dy ≥ 0] +
  [dx ≥ 0]` of (x, y) − (`param_4`, `param_5`); loose, of (xs/2, xs/2) −
  (`param_4`, `param_5`), both halves from `xs`. The decompile shows only
  two slots: the listing copies all four (`movaps`, 0x8a8bdb). With `param_6` ≠ 0, a facing below `FACE_ANY` (2)
  measures against the map centre, and `FACE_ANY` asks for facing 1. The
  wanted facing is −1 when `param_4` is −1, which matches any. Each
  template of the size whose facing matches and that passes
  `Cliffs::verify_placement@008a6310` joins a list. If the list holds two
  or more, one `game_random` draw picks one (`r % n`); one entry is taken
  without a draw. Then `Cliffs::add_cliff@008a6690` queues it.
  `verify_placement` with `param_6` ≠ 0 passes outright: it returns 1 at
  its first on-map cell and at the list's end. Strict, every cell must be
  on the map, with no 0x40 among the 48 cells round it (`move[1..49]`, the
  7 × 7), no 0x20 among the 8 (`move[1..9]`), and none of 0x10, 0x28,
  0x800, 0x1000 or 0x40 on the cell itself; it must not be dry-skipped (no
  0x100, `land` 1 or 2) or blocked (+0x11). `add_cliff` re-verifies,
  queues (x, y, t, i), and marks 0x40 on each on-map cell
  `is_clear_land(.., 2)` passes (0x22 when loose, which admits a 0x40 or
  blocked cell). **Diff-backed** (item 3875): the facing table, the
  offsets, the 7 × 7 and 3 × 3 tests and the loose facings break the
  stream when mutated. Inert on run1493, with each cause read: the
  dry-skip (all land), both blocked tests (no cell blocked yet), the
  loose 0x40 admit (no overlap), the one-candidate no-draw (every list
  had 0 or ≥ 2), and loose's pass-outright (both loose sites pass strict
  too).
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
  **Diff-backed** (item 3890, 894 on run1493): the skip on tile mask bit 4
  is `CoordInfo::flags & 4` (`terrain+0x6ab0`, +0x1c), and the template's
  index counts skipped vertices too (listing 0x8a7fe7).
- **What would kill these claims.** run1118's function list already holds
  `008a8bc0`, `008a8680` and `008a7ea0`. A Southwest Mesa lobby at
  `CHECKSUM` detail 1 with `check_all_level ≥ 14` and a start `WORLD`
  dump (run1118's click-free line with the checksum detail added; **run1493**,
  item 3869) parts at 7859 (`place_all`'s draws) if the template draw is wrong. It parts at
  894 (part 5, the heights) if `place_cliff`'s offsets are wrong.

### 18.4 The slope test

`TerrainOut::has_plus_slope@00869790`: **run1118 enters it** (Southwest Mesa), which is the one shipped style with a
live cliff group (§7). As read: the four corner vertices of the tile,
at stride `4·xs + 1`, are each clamped to ±10000. The test passes when
max − min > 192 · tanf(angle · 0.017453292), with the angle in xmm3
from `assess_wcoord`. It writes min and max through the two out
pointers. What would kill it: a Southwest Mesa start dump with 0x40
cells (zero on all three closed maps' 205 files) whose `blocked` cells
or part 5/6 at 894 part from a port that applies this test. The port's
`place_all` marks 292 cells 0x40 on run1493 at 7859 (item 3875), so
Southwest Mesa's step 7 is the first to hand this test a tile.
**Diff-backed** (item 3890): it is handed 4,672 tiles there and passes
1,827, and 894's parts 5 and 6 agree. As built: the angle is 25
(`[0xb69658]`), so the threshold is 89.531075 (`0x42b30fe9`); only the
first corner is clamped, each way once (low against 10000, high against
−10000), and the test is strict. No span comes within 70,139 ulps of the
threshold and no corner passes ±10000, so the clamp and the strictness
rest on the listing (0x8697ce..0x86984e).

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
Its one other 1 (a lower land beside sub 0) needs no shipped land (item
3891's audit, row 1).
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
  three corners, it does not. (Not on the original: §20.)

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
was entered by the console's `river calc` (run1499, §21).

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
is tooling this item does not build. Taken: §20.

## 20. The console-planted contact, taken (item 3871)

Run1494 is the golden start (Great Lakes, `--map 14`, seed 12345) at
`cover=1` on the click-free pool, to 640. It uses run1129's line, with
`620 @aim 41 42` and `620 !weather` added. `@aim` is a new tracer line
(`tools/trace/tracer.c`, record `I_AIM` 33). It stores 768 c + 384 into
`ConsoleWin` +0x518/+0x51c, because `weather` reads its cell as
`div_3_table[coord >> 6] >> 2` and a `.cmd` line's `no_mouse` 1 skips
the mouse read. The store was taken instead of an `@click` because a
click reaches the cell only through `TerrainOut::get_mouse_coords@00864270`'s
float camera projection. It also needs the typed path, since `parse_cmd`
reads the mouse only when `no_mouse` is 0.

- **The aim landed.** `I_AIM` reads cell (41, 42) and the stores 31,872
  and 32,640. `!weather` ran with `parse_cmd` returning 1. Block 621
  (`@dumpall`) against the golden start's own `WORLD` grid (run1030's
  first block, the same seed) differs in exactly one cell: index
  `xs · 42 + 41`, `OCEAN` 0 → `BASELAND` 3. It is the engine's own
  `spot_update_land` index (`Terrain::spot_update_land@0084fa50`), so the
  dump is row-major. §19.6's two cells are row-major too: the south-west
  shape found by search on that grid is (41, 42) and (16, 45), and
  C = (42, 41) is `SANDY` 0. The flanks (41, 41) and (42, 42) are `OCEAN`,
  as are P's other seven neighbours.
- **The arm was not entered.** `report.py functions` has no `HIT` for
  `TerrainOut::check_for_diagonal@008595f0` or `WorldData::is_valid@00471170`,
  and both are in the trace's function table. The re-mark did run over C:
  `TerrainOut::add_update_wcoord@008738a0` lists P and its eight
  neighbours, and `spot_update_land` marks the list and then calls
  `create_transition_renderables` in the same call (line 111).
- **What the kill reads.** `TileSet::is_combo_acceptable@0087abe0` is
  three-valued. Same land within one sub returns 1 when B's sub is
  greater and 2 otherwise, so ocean beside ocean is 2, never 1. With that
  read, §19.2's (A) and (B) arms agree with `mark_trans_flags`. C is a (B)
  cell: no orthogonal M → C is 1, and only the south-west diagonal D → C
  is. Each flank's first M with M → F 1 is C. So the gate that withheld
  C's 7, or emptied a flank, sits in a branch of
  `TerrainOut::check_for_tendrils@00859eb0` that §19.3 did not walk. Its
  return decides (A)'s 7 (`mark_trans_flags`, `iVar6 == 0`), and its
  three-neighbour arm (`local_14 + local_10 + local_c + local_18 == 3`)
  returns 1 after writing 5s. ~~Which branch fired is not read.~~ The
  three-neighbour arm, on the S flank, printed (item 3874, below).
- **The instrument that names it.** Every edge write in
  `mark_trans_flags` and `check_for_tendrils` is a `Log::say` at
  `GLOG_TERRAIN`, detail 0x28. No dump on disk prints `TERRAIN` at 40, so
  a retake of this line that prints `TERRAIN` at 40 on 620 prints the
  re-mark write by write. ~~That retake is the next booking (parked
  3871a).~~ Taken as run1495 (item 3874, below).

**The printed retake (item 3874, run1495).** `check_for_tendrils`'
eight slots are `move_x/move_y[1..8]` (`world::MOVE_49`'s compass ring),
stored through `local_50` as NW `local_24`, N `local_18`, NE `local_28`,
E `local_c`, SE `local_30`, S `local_10`, SW `local_2c`, W `local_14`.
A slot is 1 when neighbour → cell is 1. The three-neighbour arm tests
only the four orthogonals. A `say` outside the two dumps runs at
`current_mode` 0 (`begin_frame@00932a70` and `end_frame@009329d0` set it
back), so it is read against `[Misc Logging]`'s row, which the console's
`loglevel` (row 3) cannot reach. The new tracer line `@logdetail 0 28 40`
(`I_LOGDETAIL` 34) writes that one byte, `game_log` +0xc0 + 0x1c, at
620's top, and `@logdetail 0 28 0` clears it at 621's. Every write then
prints, and the print agrees with the walk:
- **W flank F1 = (41, 41)**: E (C) and S (P) are one up, and NE (42, 40)
  `SANDY` is one up too, but the arm counts orthogonals only, so the sum
  is 2. (A) with M = C, then `check_for_tendrils` returns 0, then
  `Set edge x (42,41) with flags 7`: C's W field is 7 at shift 6.
- **C = (42, 41)**: the (B) batch, then `x (42,41) 1` and `y (42,42) 4`
  (§19.3's SW corner), then two raw 1s.
- **S flank F2 = (42, 42)**: N (C), E (43, 42) `SANDY` and W (P) are one
  up, so the sum is 3. (A) with M = C, then the three-neighbour arm's
  `local_10 == 0` branch: `y (42,43) 5`, then `|2` and two `0x38`s, and it
  returns 1. No 7 reaches C's S field, which stays 4, and `check_blocks`
  leaves a non-full field with F2 → C not 1. 3871a's killer (both flanks
  7) did not fire. The gate is that arm.
- **The predicate grows.** On an `OCEAN` flank, the SW contact needs each
  flank's other two orthogonals ~~not one up~~ both not one up or both one
  up (item 3876, below): (x − 1, y − 1) and (x − 2, y)
  for F1, and (x + 1, y + 1) and (x, y + 2) for F2. (16, 45) fails on F2's
  E (18, 45) `SANDY` (3871b, killed). A search of block 621's grid finds
  no `SANDY` cell with an `OCEAN` SW, both `OCEAN` flanks and both flanks
  at sum 2 (scratch count, seven candidates, none passing). So no single
  `weather` plant on Great Lakes' golden start reaches the arm (parked
  3874a).
- **The (B) batch writes the vertical field four times.** Its four
  `orl $0x38` stores (0x862984, 0x862a45, 0x862afa, 0x862bbb) all go
  through `+0x499c`, at (x, y), (x + 1, y), (x, y) and (x, y + 1). They
  print as such (run1495's lines for (40, 41) and C), and none reaches
  the N or S field at `+0x49a0`. That is the listing, not a misprint. A
  port copies the order, not the symmetry.
- **Twin.** Against run1130, `rngcmp.py` finds 641 identical and
  `samegame.py --from 620` the same game before 620. Against run1494,
  `rngcmp.py` finds 641 identical. 620 parts on the printed lines, and 622
  parts on 9 lines (`turn_control` and the `COMMANDMANAGER` package),
  both after F = 620. 6,728 functions were entered, and
  `check_for_diagonal` and `is_valid` were not, so the run is not pinned.
- **Twin.** run1130, run1129's `cover=0` twin: `rngcmp.py` finds 641 in
  common, 641 identical, 0 differing. `samegame.py --from 620` finds 40
  in common and the same game before 620. It parts on 621 and 622, the
  `@dumpall` block and the next opening dump, by construction.
- **Not pinned.** The trace enters no `NEVER` row (0 of 264), so joining
  `blind::TRACES` would cost a read and move nothing. `Blind:` is
  unmoved: never 264 · residue 223 · open 41.

**Verdict: the plant is not a contact.** §19.3's south-west shape is not
sufficient on the original. The predicate keeps the arm out of a single
plant on Great Lakes' golden start (~~a single plant~~: East Indies holds
five that pass, item 3876, below), and the arm and `is_valid` stay open. The row is not `RESIDUE`
while the console can plant. ~~(16, 45) is the same shape on the same
grid and is not worth a take until the gate is named.~~ The gate is named,
and (16, 45) fails it (item 3874, above). 3870a (the
start's leak, 0xee1224) stays parked, because the trace does not print
it.

**The offline search (item 3876).** The model is `tools/gamelog/plant.py`,
the chain as read: `ConsoleWin::run_cmd@007d6a70` case 99, then
`Terrain::spot_update_land@0084fa50` (land 0, sub 3),
`TerrainOut::add_update_wcoord@008738a0` (`move[0..8]`, P first),
`Terrain::verify_land@0084d8a0`, the four-edge reset, `mark_trans_flags`
per listed cell (`TerrainOut::set_edge_x@00859500`,
`TerrainOut::set_edge_y@00859410`, `TerrainOut::neighbor_count@0085bb30`),
and `check_blocks`' empty word. It is a reading tool, never a source, and
it logs each edge write as the `TERRAIN` 40 print does.
- **It reproduces both of 3874's verdicts.** On run1495's grid with (41, 42)
  set back to `OCEAN` 0, the plant predicts run1495's 81 `Set edge` writes in
  order and equal (`plant.py cell … 41,42 --set 41,42,2,0 --printed …`),
  C's word after `check_blocks` is N 0 · E 0 · S 4 · W 0, and (16, 45)
  fails the same way.
- **`check_for_tendrils`' return is three arms.** It returns 1 when the
  one-up orthogonals are exactly N and S, exactly E and W, or any three
  (`local_14 + local_10 + local_c + local_18 == 3`). Otherwise it returns 0.
  A SW flank's set always holds C and P, an L. So (A)'s 7 is written when
  the flank's other two are both not up (sum 2) or both up (sum 4). On the
  East Indies melee grid, (40, 24) passes at sum 4: F2's E (42, 24) and S
  (41, 25) are `SANDY`. 3874a's "a second plant cannot lower a sum" holds,
  but a plant can raise a 3 to 4.
- **The count.** 1,134 gamelogs, of which 595 carry a `WORLD` block with
  cells. Their first blocks are 68 distinct grids, and 516 traced start
  heads are 45 of them. (§19.4 counts 86 and §19.6 43; that difference is
  not read.) Every cell is planted under `COAST_BASE` 3 and 0. **62 plants
  pass, on 18 grids**, the same under both values. None pass on the other
  50 grids, including all 25 Great Sahara grids, all 19 Great Lakes grids,
  Southwest Mesa and Himalayas. Every pass is §19.3's south-west shape,
  C = P + (1, −1).
- **East Indies.** The two-player grid holds 17 SW sites (C `SANDY`, with
  P and both flanks `OCEAN`) where §19.6's model found 13. Five pass:
  (2, 17), (27, 38), (29, 40), (14, 57) and (26, 59). (43, 17) fails. The
  five hold on every two-player East Indies grid on disk, at the start and
  in the dumps at 2040, 8955 and 17145. The three-player melee grid passes
  (36, 21), (40, 24), (36, 57) and (16, 59).
- **The take (i).** East Indies, `--map 18`, map size 2, two players, seed
  12345 (run1156's lobby). Use run1494's line with `620 @aim 2 17`, `620
  !weather`, `621 @dumpall`, and 3874's `@logdetail 0 28 40` / `0 28 0`.
  The plant predicts 80 writes. Write 19 is F1 (2, 16)'s `x (3,16) 7`, and
  write 38 is F2 (3, 17)'s `y (3,17) 7`. C = (3, 16)'s own SW tendrils OR
  their 1 and 4 into those two fields, so both are full, `check_blocks`
  empties them (`SANDY` → `OCEAN` is 1), and N and E are 0. So
  `check_for_diagonal@008595f0` is entered at (3, 16). *Falsifier*: that
  take at `cover=1` with the `@dumpall` showing (2, 17) `OCEAN` 0 →
  `BASELAND` 3 alone, and no `HIT` for 008595f0, kills the model.
  ~~`plant.py cell … 2,17 --printed <the take>` names the first write that
  parts.~~ **Taken (item 3879, run1497): entered.** `check_for_diagonal`
  is first entered on 620; `--printed` reads 80 writes, equal to the
  model's in order; block 621's grid against run1157's start changes
  (2, 17)'s land alone (`OCEAN` 0 → `BASELAND` 3), plus three `BASELAND`
  sub-indices at (48..50, 47..48), 46 cells off and outside the re-mark.
  `WorldData::is_valid@00471170` is not entered, so the arm's body
  reaches no `is_valid` call on this cell (not read). The model is
  diff-backed on this take.
- **On the golden line, two plants.** On Great Lakes' golden grid, `weather`
  at (42, 43) and then at (41, 42) lifts F2 = (42, 42) from sum 3 to 4 and
  enters at C = (42, 41). (17, 46) then (16, 45) enters at (17, 44).
  (41, 42) then (42, 43) does not, because the second re-mark does not
  list C.
- **Not established.** That the take's frame-620 grid equals the start's
  around (2, 17) (the `@dumpall` settles it). `verify_land`'s `depth`
  (3870a) does not matter here: its walk rewrites nothing around
  (2, 17), so an early return changes nothing. ~~And
  everything past the entry: `check_for_diagonal`'s body and
  `WorldData::is_valid@00471170` are not modelled.~~ Modelled (item 3886,
  below). Every claim here is
  reading-only except the 81 writes.

**The arm's body (item 3886).** `check_for_diagonal@008595f0` reads C's
four orthogonals (land, sub), zeroed off the map, then runs four pair
tests in order: N = E "tr", N = W "tl", S = E "br" and S = W "bl". Each is
true when both cells hold the same land and sub, and that differs from C's.
Each true test prints `Found <tr|tl|br|bl> corner at (x,y)
(t=…,r=…,b=…,l=…)` (`TERRAIN`, detail 0x24, the four lands). It then sets
the caller's shift-6 block word to 0xfc0, 0xe07, 0x1f8 or 0x3f, and ORs the
shift-0 fill word with 0xc, 0x60, 0x801 or 0x300
(`create_transition_renderables@00862c60` line 95 passes `&local_20` and
`&local_30`).
- **Then** (the listing, 0x859a4b..0x859b1e, which agrees with the
  decompile):
  - if "tr" is true and so are the other three, it prints `ISLAND`, zeroes
    the block word, restores the fill word and returns;
  - if "tr" is true and any other is false, it returns;
  - if "tr" is false, then only when tl, br and bl are also false does it
    reach the four `WorldData::is_valid` calls (0x859b33, 0x859c16,
    0x859cf1, 0x859dd4). Those calls bounds-check the diagonals NW, NE, SW
    and SE, and for each diagonal D with D → C 1 the arm prints `CORNER:
    Found … corner`, sets the block word (0x801, 0x300, 0xc, 0x60) and ORs
    the fill word (0x1fc, 0x3f, 0xfc0, 0xe07).
- **The arm writes nothing persistent.** It writes only the two words on the
  caller's stack, which the caller's 0xe00/0x1c0/0x38/7 tests then read.
- **Diff-backed on run1497.** Its one print is `Found bl corner at (3,16)
  (t=1,r=1,b=2,l=2)`: S and W are both `OCEAN` 0 and differ from C, which is
  `SANDY` 0. `plant.py cell … 2,17 --printed <run1497>` predicts that line,
  and only that line, and exits 0. That is why (3, 16) reaches no
  `is_valid`.
- **The predicate.** A cell enters only as §19.2's (B). At each corner
  whose D has D → C 1, both flank fields must be cleared, so C → F is 1 for
  each flank, and two equal flanks fire that corner's own pair test. So
  `is_valid` is reached only when every such corner's two flanks differ.
  Over every shipped land and sub, at `COAST_BASE` 3 and 0, distinct
  one-up flanks always put D beside a flank it is mutually unacceptable
  with. `SANDY`'s only one-up land is `OCEAN`. **So the route to
  `is_valid` needs §19.3's P1 pair, and P2 alone, the console's
  south-west contact, never reaches it.**
  `halfland::tests::the_diagonal_arm_reaches_is_valid_only_past_a_p1_pair`
  pins it on the crate's own `is_combo_acceptable`, made to fail once.
- **The search.** `plant.py scan` covers 69 distinct grids (597 files):
  66 single plants enter the arm and 0 reach `is_valid`. A two-plant scratch
  search, with the second plant within Chebyshev 3 of the first, at
  `COAST_BASE` 3, gives 8,201,021 plant pairs on the 69 grids, 2,152 entering the arm (on 41 grids), and 0 reaching `is_valid`.
- **Verdict: (ii), none.** `WorldData::is_valid@00471170` is a
  ~~`blind::RESIDUE` candidate, moved only on a second reading's agreement.~~
  **`blind::RESIDUE` row: two readings agree, item 3891**
  (`docs/audit/2026-11-05-item-3891.md`, row 1). The blind reader re-derived
  the pair tests and the fall-through. Its UNSURE rested on a shape that
  `check_for_tendrils`' own shift-6 writes on C's flank edges close. The
  adjudicator re-derived the other three steps on the crate's
  `is_combo_acceptable`, compared branch by branch with the body.
  ~~That reading must re-derive four things:~~
  - the four pair tests and the fall-through (0x859776..0x859b1e);
  - that C → F 1 holds on both flanks of an entered corner (§19.2–§19.3);
  - `SANDY`'s single one-up land;
  - the distinct-flank ⇒ P1 step.

  The one route that reopens it is a P1 pair surviving the re-mark, which
  takes `verify_land`'s depth leak past 500 (3863a, §19.6). No plant
  sequence on disk is read to reach that. *Falsifier*: a `cover=1` take
  that prints `CORNER: Found` (`TERRAIN` 0x24 by `@logdetail 0 28 40`), or
  that has a `HIT` on 00471170, with no P1 pair around C in its `@dumpall`.

## 21. The console's river arm, taken (item 3889)

`River::calculate@008858e0` is an open `blind::NEVER` row (§19.4: the
console's two river arms and `TerrainMap::river_stop` reach it). This take
enters it through the first of the two arms.

- **The words.** `ConsoleWin::run_cmd@007d6a70` case 0x65 is `river`
  (`console.py table` index 101, console-only). Its words are
  `loc_str_array_orig` entries, which `Main::init_string_tables@00595a10`
  loads from `Data/translated_strings.xml` at stride 0x14. Read
  positionally (as `console.py` reads `internal_strings.xml`): +0x43d0 is
  868 `start`, +0x45ec 895 `grow`, +0x4600 896 `calc`, +0x4614 897 `path`,
  +0x4628 898 `flow`. The `start` arm's message, +0x45d8, is 894 `New
  river started`, which checks the read. Each test is `String::ignore(arg,
  word, -1)`, a whole-string `_wcsicmp` of the first argument token, so a
  line carries one word.
- **The arms.** `grow` calls `RiversOut::start_river` when the global
  `cur_river` is null, then `RiverOut::add_spline_point` at
  `div_3_table[mouse_coord >> 8]`, which is cell c for `@aim`'s 768 c +
  384. `calc` calls `River::calculate(cur_river, …, 0, 0, 0)` when
  `cur_river` is non-null. `path` runs `PathFinder::astar_river` from the
  cursor; on a path it starts a river, adds the path's points and calls
  `calculate`, the second arm.
- **The staging** (`tools/gamelog/coverage/river3889-start.cmd`, run1494's
  golden-start line): `@aim` with `!river grow` at (32, 28), (33, 28) and
  (34, 28) on 620–622, then `!river calc` on 623. The staging uses three
  points, not one: `RiverOut::ensure_ocean_end` runs only past two points,
  and `Spline::calc_spline` sets the order to `count − 1`, which is 0 for
  one point. The 11×11 around (33, 28) is `BASELAND`, so
  `ensure_ocean_end`'s 7×7 holds no `OCEAN` and no off-map cell.
- **Taken (run1499): entered.** `River::calculate` is first entered on
  623. All five console lines return 1, and `I_AIM` stores
  768 c + 384 for each of the three cells. `rngcmp.py` against run1130 gives 641
  identical frames, and `samegame.py --from 620` gives the same game
  through 623; it parts only on the `@dumpall` block 624 and on 625. No
  other `NEVER` row is entered. `TerrainMap::river_stop@009b5d20` is
  called only by `TerrainMap::on_left_down` and
  `ScenarioEditor::modify_terrain_map`, the editor's UI — `TerrainMap`
  is built by `IFaceEditorMain::init` alone — so it is `blind::RESIDUE`
  on the editor's cut, with the editor's three other open rows (the
  sixty-seventh pass, DECISIONS 107 §3).
  `World::set_orig_coast@006b1b80` has no caller in the export, and the
  pass's scan of the image finds no call and no vtable slot: the one
  four-byte match of its RVA, at 0x67b5ac, is the rel32 of
  `AmmoOut::draw`'s call at 0x67b5ab, which `pe.references` excludes
  (`every_unreferenced_row_is_unreferenced_in_the_image` asserts none);
  `RESIDUE`, dead, on the guard's dead list.
- **Not established.** What `calculate`'s body did on 623. The 624 grid
  equals the start grid in every cell's land and sub, so its
  `spot_update_land_batch` at `riverbase` changed no cell this dump
  prints, or ran on an empty list. The coverage records only a
  function's first entry (every callee was first entered on −1, at map
  generation), so the trace cannot say which (parked 3889a). The `path`
  arm is not taken. Every claim here is reading-only, except the entry
  and the unchanged grid.

## 22. The map styles no chain runs, read to their lobby (item 3908)

Item 3908 took the implicit residue's `map` stem: fifteen functions that no
pinned trace entered and no document cited (`python3 tools/census.py
--territory --pin --implicit`). The item read each style to its lobby id and
each `Map` row to its callers, from the decompile and the listing. **It took
no capture, and the styles' bodies are unread on purpose**: Ramon has not
named the held-out style (3888c, DECISIONS 108 §2; ~~not named~~ named since: East Meets West, §24), and the held-out style
is captured and scored, never read against. So this section reads the
dispatch, the lobby and the callers, and nothing inside a `make_continents`
past its entry.

### 22.1 The dispatch: a lobby id is a `new_map` case

`Map::new_map@006a0950` switches on the style. Each case 6..0x1a installs
one class's vtable, and each class's `make_continents` is that vtable's
+0x40 entry, which `Map::make@0068bc90` dispatches (§1). A style below 6
first goes through `Map::set_map@006a1340`, which turns it into a concrete
style before the switch. `set_map` draws `r % 17 + 6` until a 19-byte stack
table accepts it: styles 0 and 3 accept any non-zero byte, 1 and 4 accept the
land byte 2, and 2 and 5 accept the water byte 1. Some arms also turn down
16, 17 and, under team style 2, 15. The table and arms are reading-only.
They come from the decompiler's stack layout, and no traced lobby has used a
random style. The click-free lane's `--map N` writes the profile's
`<MAP_STYLE>` (`tools/explore/unattended_capture.py`, `set_map`), and
`verify_game` refuses a game whose dumped style is not `N`. So a capture's
`--map N` is `new_map`'s case `N`:

| `--map` | class | `make_continents` | entered by |
|---|---|---|---|
| 6 | Old World | `MapOldWorld::make_continents@0069c0e0` | run1117 |
| 8 | Amazon Basin (the lobby's "Amazon Rainforest") | `MapAmazonBasin::make_continents@0069be00` | run1099 |
| 9 | Himalayas | `MapHimalayas::make_continents@0069bc90` | run1128 |
| 11 | African Watering Hole | `MapAfricanWateringHole::make_continents@0069b650` | run1119 |
| 12 | Mediterranean | `MapMediterranean::make_continents@0069add0` | run1120 |
| 13 | Australian Outback | `MapAustralianOutback::make_continents@0069a650` | run1121 |
| 16 | New World | `MapNewWorld::make_continents@006984d0` | run1123 |
| 17 | Colonial Powers | `MapColonialPowers::make_continents@00697dd0` | run1124 |
| 19 | East Meets West | `MapEastMeetsWest::make_continents@00696640` | run1125 |
| 20 | (Atlantic) Sea Power | `MapSeaPower::make_continents@00696080` | run1126 |
| 21 | Nile Delta | `MapNileDelta::make_continents@00695bf0` | run1098 |

`report.py <exe> refs` reads each of the eleven as "pointer only": one
`.rdata` copy, at its class's vftable +0x40 (Nile Delta's at 0xb45714,
Amazon Basin's at 0xb45d38). The other cases are corpus or chain styles
already read: 7 Great Sahara, 10 Southwest Mesa, 14 Great Lakes, 15 Warring
States and 18 East Indies. Case 22 is British Isles (run1127), 23 is
`MapGrass`, the blank map, 0x18 and 0x19 are `MapConquest`, and 0x1a is
`MapScenario` (§18.1).

### 22.2 The takes were on disk: eleven traces joined, no capture

**The disk already answered the question the booking put.** Item 3480
(runs 1098 and 1099) and item 3493 (runs 1117–1128) swept the styles at
`cover=1` on the click-free pool, with `--end-frame 36`, seed 12345 and the
default lobby, looking for `check_for_diagonal`. They filed the traces and
pinned none. Item 3858 joined two of them, run1118 and run1122. The census's
"no pinned trace entered" therefore meant that nobody had joined the traces,
not that no trace existed. A scan of every `rontrace-run*.log` in the
archive's `Logs` (the record kind and `HIT` address that `blind::entered`
reads) finds the following:

- each of the eleven `make_continents` above is entered on −1 by the run in
  its row, inside the map script and before any frame a twin compares;
- **`Map::eliminate_edge_canals@0068b360`** is entered by run1098, run1125
  and run1126. `refs` names exactly three callers: Nile Delta's
  `make_continents+0x444`, Sea Power's `+0x591` and East Meets West's
  `+0x6cf`. It runs in the mask phase (§5) of styles 21, 20 and 19, and in
  no other style;
- **`Map::find_region_centroid@0068ae50`** is entered by run1125. Its one
  caller is East Meets West's `make_continents+0x642`, so it runs in style
  19's mask phase alone;
- none of the eleven traces enters a `blind::NEVER` row.

Item 3908 joined the eleven to `blind::TRACES` the way item 3858 did, with
no twin (parked 3858a gives that shape's reason). So thirteen of the
fifteen rows leave the implicit residue entered, and none joins `NEVER`.
Run1127 (British Isles) enters none of the fifteen and was not joined.

**What the entry does not give.** These are coverage traces. They are not
setup traces: none carries `check_all_level` 14 or a `CHECKSUM` line, so no
checkpoint 7805..7945 for these styles is on disk ~~in these traces~~ — the
AI track's Himalayas files run348, run464 and run465 carry style 9's (§27.2,
parked 3946a) — and nothing here backs a value. **A style's chain take** is still one run with run1493's line at its
own `--map`, written here and not run:

```
viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run<N> --map <style> \
  --end-frame 36 --timeout 1200 --dump-all-start --cover cover=0
```

That take is what a port of a style's chain needs (its falsifiers are 7807's
seed, part 2 and part 5, then §2's checkpoints). The commander books it as a
`coverage` item per style only once Ramon has named the held-out style. That
style's take is scored and never read against. The other ten are read from
their own setup traces when a chain is booked. The blind list does not need
any of them.

### 22.3 The two `MapConquest` rows: one on scope, one with a script route

**`MapConquest::add_additional_players@0068d260`: `RESIDUE` on Conquer the
World's scope.** `refs` names two callers, `MapScenario::make+0x550`
(0x68ce40) and `MapConquest::make_continents+0x986` (0x68e526). The second
is the mask of styles 0x18 and 0x19, which only
`ConquestGame::build_conquest_game` mints. The first is style 0x1a, which
only that function and the editor reach (§18.1, DECISIONS 106 §3). Both
callers are `blind::RESIDUE` rows already. The function sits after a `ret`
(byte before 0xc3), so `refs` cannot exclude a fall-through into it. The
listing is what settles the entry. What would kill the claim: a `cover=1`
trace of a skirmish lobby that enters `0068d260`.

**`MapConquest::check_starting_loc@0068d7e0` is not on scope: it has a
third caller.** `refs` names `add_additional_players+0x169` and `+0x3b4`,
and also **`ScenarioFuncSet::find_extra_starting_loc+0xec`** (0x9e515c).
That caller,
`ScenarioFuncSet::find_extra_starting_loc@009e5070`, is a BHS script
function. `ScenarioFuncSet::init_funcs` registers it as
`find_extra_starting_loc` with three parameters (`num_nation_space`,
`num_land_space`, `bool_only_check_player`), and its only reference is that
registration's pointer at 0x9c8363. The console's `!eval` runs a BHS
expression against every registered set (`docs/AI.md`, Loop 3473; run1092
entered seventeen `ScenarioFuncSet` rows this way). So the pair has a
skirmish route, and both rows join `blind::NEVER` as live. No pinned trace
enters either function.

- **The predicate**, read at 0x9e5070. The list the function walks is
  `Regions.list`'s region 1's `coords` (`Region` +0x6c, count at +0x70,
  data at +0x7c; `types.txt`). `Regions::rebuild_coords` fills it in every
  skirmish (`docs/SYNC.md` §3.20). The function returns 0 without a call
  when that count is 0 or the second argument is negative. Otherwise it
  makes one `game_random` draw, `Random::get(0, 0xffff) % count`, when the
  count is above 1. It walks the list from that index, and for each cell it
  calls `check_starting_loc` when the third argument `v` satisfies
  `v − 1 < 8` and `leaders.list[v − 1] & 1` is set. It appends the first
  accepted cell to `ScenarioData::extra_starting_locs`.
- **The staging**, written here and run by item 3921 (§23: both functions
  entered on 620, so the falsifier did not fire): the golden start (`--map 14`,
  run1129's lobby) at `cover=1` with one line, `620 !eval
  find_extra_starting_loc(4, 4, 1)`, where `v` is 1 for the human's seat. The
  draw parts the twin on the `!eval` frame, so the twin is compared to 619.
  What would kill the claim: that trace with `009e5070` entered and
  `0068d7e0` not. That would mean region 1's list was empty, or that the
  decompiler's argument order is wrong. The argument order is read from
  `add_param`'s order and the dereferences alone.
- **Not read**: `check_starting_loc`'s body past its bounds test. Its
  register arguments (`in_ECX`, `in_EDX`) make the decompiled signature
  unreliable, and nothing here depends on what it accepts.

### 22.4 What a capture would change, and what is not established

Every claim in §22.1 and §22.3 is a reading. §22.2's entries are measured
from the eleven traces and pinned by
`blind::tests::the_blind_list_is_the_pinned_residue`. Three things are not
established:

- the `set_map` table;
- ~~whether region 1's list is non-empty on the golden start's map (it is
  for any map with a land region 1, which this item did not check on
  run1129's dump);~~ it is non-empty: run1515 enters `check_starting_loc`
  (§23);
- §5's sentence that ~~Amazon~~, Himalayas and ~~Old World~~ "read the same" as
  Great Sahara (Old World's listing is Great Sahara's: §25.1; Amazon Basin's: §26.1). That sentence predates this item, and this item did not
  re-read it because of 3888c.

## 23. The `!eval` take: `find_extra_starting_loc` enters `check_starting_loc` (item 3921)

§22.3's staging, run as written. The golden start (`--map 14`, run1129's
lobby) at `cover=1` on the click-free pool, with one line, `620 !eval
find_extra_starting_loc(4, 4, 1)`
(`tools/gamelog/coverage/startloc3921-start.cmd`), to 640. This is run1515.
Its `cover=0` twin, run1515c, sends the same input, so no `--from` is
needed (`docs/RUNS.md`).

- **Entered, both on trace frame 620** (`report.py rontrace-run1515.log
  functions`): `ScenarioFuncSet::find_extra_starting_loc@009e5070` and
  `MapConquest::check_starting_loc@0068d7e0`. The falsifier written
  before the take (`009e5070` entered, `0068d7e0` not) did not fire.
  So region 1's `coords` list is non-empty on this map, and the
  argument order §22.3 read from `add_param` is the order `!eval`
  passes. Both rows leave `blind::NEVER` with run1515 in
  `blind::TRACES`.
- **`MapConquest::add_additional_players@0068d260` is not entered**, which
  agrees with §22.3's `RESIDUE` ruling. The script route reaches
  `check_starting_loc` directly, never through it.
- **The draw.** run1515's `game_random` word agrees with run1129's and
  run1130's (the same lobby with no `!eval`) on every frame through 620,
  and parts from both on trace frame 621, harness word 620 (`rngcmp.py`).
  That is §22.3's `Random::get(0, 0xffff) % count`, made on the `!eval`
  frame. A word compares the stream's state and not its count, so the
  take shows that a draw happened on 620. It does not show that exactly
  one did.
- **Against the twin**: run1515c, `rngcmp` 641 in common, 641 identical, 0
  differing; `samegame` 40 in common (600..639), 0 differing. Both send
  the `!eval` line, so the two agree after 620 too.

**Not established.** Which cell was accepted, and what
`ScenarioData::extra_starting_locs` holds after the call, because no
category in this take's detail prints either. How many cells the walk
passed to `check_starting_loc` before one was accepted, because a
coverage trace marks entry and does not count calls. And what
`check_starting_loc` accepts (§22.3's "Not read" still holds). None of
these moves a score. A packet at `RON_STATE_FRAME=620`, reading
`extra_starting_locs`, would settle the first two (parked 3921b).

## 24. The held-out style, East Meets West: 8 of 8 phases on its first take (item 3941)

East Meets West (style 19) is the held-out style, on Ramon's word (3888c;
DECISIONS 54, 61, 111 §1). It is scored and never read against. **Its
port held all eight of §15's phases on run1524's first comparison.** The
pin is `floors::MAPGEN`'s `EastMeetsWest` row, `ported: 8, phases: 8`, and
`EAST_MEETS_WEST_HELD` in
`rondata::mapgen::tests::east_west::east_meets_west_on_its_first_take`.

**The order.** The port was written from the decompile and the listing
alone, with no style-19 dump read. It was pushed as `004dbbab7` before
run1524 was launched. `sim::map_east_west` holds
`MapEastMeetsWest::make_continents@00696640` and the arms no other ported
style runs: `Map::fill_cont@0068a960`, `Map::find_region_centroid@0068ae50`,
`Map::eliminate_edge_canals@0068b360`, `Map::place_start_in_region@0068ac00`,
`Map::is_near_ocean@0068b0a0`, and `Map::check_player_land@0068ef00`'s
guarded arm. The growers, `make_region`, `scale_by_area` and
`eliminate_pools` are `sim::map_lakes`', shared with Great Lakes and East
Indies, with no body changed.

**The take.** run1524 used §22.2's line at `--map 19` (`docs/RUNS.md`). It
has one file. On it the port drew 6,788 mask draws with no stalls and no
unplaced seat. It read `avoid_continent` as 14, the two regions'
centroids as (46, 36) and (10, 21), and the starts as (15, 16) and
(37, 40).

| phase | checkpoints held |
|---|---|
| 1 the mask | the stream against the trace's segment, 6,788 draws; then 7807 |
| 2 zero-draw passes | 7835, 7843, 7846, 7854 |
| 3 `place_all` and tail | 7859, 7861, 7865, 7870 |
| 4 resources | 7877, 7880, 7927 |
| 5 values | 7937, 7945 |
| 6 `Terrain::init` | 734 part 4 and the 24 records to 5012 (489 half-land cells) |
| 7 rivers | 5012, 5014, 5076, 5116, 5118 (no rivers) |
| 8 heights | 894 |

**What the first execution was.** The harness's first run stopped in
phase 2 on a panic, before it reached a phase-2 checkpoint, so no
phase-2 verdict was printed. The mask had held. The cause was the
harness: `EASTWEST.xml` carries no `TILECHANCE`, so the style's
`init_map_data` makes no draw (§4). The draw stepped back from 7805 is
then `default.xml`'s. The harness had asked the style file for its list.
It now falls back to `default.xml`'s, as §4 reads, and the second run is
the comparison above. Nothing in `sim` was touched between the two runs.

**Confidence and what is not established.** Phases 1–2 are the strongest
evidence. Phase 1 compares the whole mask draw stream site by site, and
7807 checks the whole grid. Phases 3–8 compare the seed and the dump's
parts at each checkpoint. They run no draw-stream check and no
start-dump vertex check, so "held" there means the checksums and the
seed agree. The take is one lobby: two seats, seed 12345, no teams. Two
readings are untested because the lobby cannot falsify them. The first
is that `get_team` 8 and team = own seat give the same continents. That
can only part under teams. The second is that a style with no
`TILECHANCE` keeps the tileset `default.xml` drew. The takes so far
cannot tell this from the style resetting the tileset to a list that
happens to agree, since the 7854 agreement backs the tileset in either
case. `place_start_in_region`'s second-pass band ran, but this take does
not show which pass placed each seat. Only run1524 is scored. The rule
for a held-out style is that a parting on a later take parks by frame
and books nothing.

## 25. Old World's chain, 8 of 8 (item 3944)

The first of the ten chain takes (§22.2, 3944–3953), and the method the other
nine copy. One take, run1527 at `--map 6` on §22.2's line (`docs/RUNS.md`).
The chain is then walked phase by phase against the take's own checkpoints,
in `Map::make`'s and `Terrain::init`'s order
(`rondata::mapgen::tests::old_world_walk`). The first checkpoint that parts
names the arm to read. Old World is **read against**: it is not the held-out
style (§24).

### 25.1 The mask is Great Sahara's listing

`MapOldWorld::make_continents@0069c0e0` and
`MapGreatSahara::make_continents@0069bf70` decompile to the same body: the
two files differ only in the class name. The listing settles it.
`llvm-objdump -d` from 0x69c0e0 to 0x69c260 and from 0x69bf70 to 0x69c0e0,
with addresses stripped, differ in four lines, and those are the four
relative jumps (`jns`, `jns`, `jle`, `jne`). Each targets the same
instruction 0x170 further on, and the calls name the same absolute
targets. So the port is `great_sahara_make_continents`, as Southwest
Mesa's is (§15 step 1). Its four draws are the trace's
`make_continents` segment, site for site inside 0x69c0e0..0x69c260
(`rondata::mapgen::old_world_part`), and 7807's seed and parts 2 and 5
agree. What makes Old World Old World is `OLDWORLD.xml`, read from §4 on:

- **No `TILESET`.** `default.xml`'s six `TILECHANCE`s make the one
  `init_map_data` draw (§4). Nothing draws between it and `base_edge`'s
  draw, so it is the word `tileset_before` reads off 7805's seed. On
  run1527 that word gives `jungle`, and 7854 agreeing backs it.
- `grouping="1"` groups with chances of 50 and 25 (§7), a `GOODIES`
  section of its own (§8), and one `RIVER` (`start_range` 2, `end_range` 4,
  `river_length` 50 SCALE, flags 11: water or border ends, the length a
  cap).

### 25.2 What the shared phases held, first run

From 7805's seed through `map.cpp` 5014, every checkpoint the file carries
agrees on the first run with no `sim` change. That covers the zero-draw
passes (all land, one region), `place_all` and its tail, the resources and
the values. It also covers the 5,552 draws from `place_all` through the last
`compute_val`, site and word, and Southwest Mesa's height-free
`Terrain::init` on an all-land grid with no cliff: part 4 at 734 by `SEA_MAP`,
then 21 records to 5012. Seed and parts 2, 5, 6 and 10–13 agree throughout.
Phases 1–6 of §15 therefore hold as they were built.

### 25.3 The two arms built

**The river walk's nearest start** (5076, phase 7). The first run parted
at 5076: seed 3320652424 against 422556960. Our 1,758 river draws were the
trace's first 1,758 of 2,070, so every `astar_river` agreed, and the walk
stopped one search early. `make_riverlist@00693820`'s two inline
nearest-start loops read `WorldData` `field_0x90` and `field_0xac`. Those
are `start_x` and `start_y` (+0x80 and +0x9c, with each `SimpleArray`'s data
at +0x10, `types.txt`), one entry per nation. The port read `start_city_x`
and `start_city_y`, the footprint, which holds four cells per city. With two
nations its "two starts" were both city 0's cells. So both border-ended
rivers counted against one start, and every later source near that start
was passed over. Great Lakes' lake-ended rivers never let the count bind.
Fixed in `sim::map_rivers::nearest_start`, and
`the_nearest_start_is_a_nation_s_start_not_a_footprint_cell` fails on the
old line. After the fix the 2,070 draws are whole, the three rivers (16
sources) are the trace's, and 5076, 5116 and 5118 agree.

**The troughs move a good** (894, phase 8). Old World is the first map
whose troughs reach a good. `RiverSection::create_trough_edge` calls
`Terrain::move_goody` on every vertex it terraforms, and the troughs
refused with `Goody([(44, 20), …, (14, 30), …])`. Item 3890 built
`move_goody`'s good arm for the cliffs (`halfland::Pass::with_goods`). The
troughs now take the types' footprints
(`map_troughs::calculate_all(…, goods)`) and move the good from (44, 20) to
(43, 18). The repeats on that cell then find nothing to move. At 894,
seed 422556960, part 5 1499455944 and part 6 1423377562 agree, part 4 is
held from 734, and all 58,081 start-dump vertices agree with the roster
stood on them, as do the 0x1000 cells. There are three troughs (181, 134
and 107 sections), and two oil moves ((4, 49) → (5, 51) → (7, 50)).
`Cliffs::init` is inert, since no cliff is placed.

**Made to fail** (`tools/mutate.py --batch --timeout 900` on
`old_world`, `badee562f`): six breaks, five held. The footprint lists put
back; no footprints for the troughs; the pre-search skip at `>=`; the kept
river counted at its source rather than its end; flag 2 refused. The sixth
break, ties to the last in `nearest_start`, has nothing to decide on
run1527 (parked 3944a). run1528 decides it: ties go to the first (§26.2).

### 25.4 What is not established

- The good's new cell (43, 18) is backed by part 5 at 894 and not by the
  start dump. A setup writer between 894 and 1240 writes `down` 3,
  `down_who` 8 there (parked 3944c).
- Only one file: 2 nations, 60×60, `jungle`. The other five tilesets and
  other nation counts are untaken. So are `default.xml`'s second tileset
  draw (an unavailable name) and a `grouping="1"` group whose chance
  refuses every clump.
- Great Lakes and East Indies still hand the troughs no footprints
  (parked 3944b). That is harmless while none of their troughs reaches a
  good.

## 26. Amazon Basin's chain, 8 of 8 (item 3945)

The second chain take (§22.2, §25's method). One take, run1528 at `--map 8`
on §22.2's line (`docs/RUNS.md`), walked phase by phase against its own
checkpoints by the walk §25 built, which is now one `Chain` row per style
(`rondata::mapgen::tests::AMAZON_BASIN`, `chain_tests!`). Amazon Basin is
**read against**. It is not the held-out style (§24).

### 26.1 The mask is Great Sahara's listing

`MapAmazonBasin::make_continents@0069be00` decompiles to Great Sahara's
body. `llvm-objdump -d --no-show-raw-insn` from 0x69be00 to 0x69bf70 and
from 0x69bf70 to 0x69c0e0, with addresses stripped, gives 144 lines each.
They differ in four lines, the four relative jumps (`jns` 0x69beb1,
`jns` 0x69bee2, `jle` 0x69bf5a, `jne` 0x69bf20), and each targets the same
instruction 0x170 further on. So the port is
`great_sahara_make_continents`, as it is for Old World (§25.1). Its four
draws are the trace's `make_continents` segment, at sites 0x69be74,
0x69be8c, 0x69bea2 and 0x69bed3 (`rondata::mapgen::amazon_basin_part`).
7807's seed 2355140353 and parts 2 and 5 agree. What makes Amazon Basin
Amazon Basin is `AMAZON.xml`, read from §4 on:

- **A `TILESET` of its own**: `jungle`, `tropical` and `tropical_wet`, each
  at 100. The one `init_map_data` draw is the style file's, as on Great
  Lakes (§4). On run1528 it gives `tropical`, and 7854 agreeing backs it.
- More goodies than Old World's `GOODIES` (3, 2, 6 and 6 SCALE by player,
  and two a corner), and one `RIVER` at `start_range` 4, `end_range` 6
  (Old World's is 2 and 4). The groups match Old World's in shape.

### 26.2 Every phase held, first run, with no `sim` change

Every checkpoint agrees on the first run with no `sim` change: from 7805's
seed through 5014, then 5012..5118 and 894. That covers 21,411 draws from
`place_all` through the last `compute_val`, and 8,290 river draws (one
`SPREAD`, then `astar_river`'s `PRICE`s) over 78 sources. Part 4 at 734 is
0x1e5405db on both sides and is held to 894. There are 22 records from 734
to 815 and 5 from 5012 to 5118. At 894, seed 345508960, part 5 3390494892
and part 6 841929190 agree, as do parts 10–13 at 262145 ×4 and all 58,081
start-dump vertices. The troughs (116, 167 and 101 sections, 220 cells
newly 0x1000) make four oil moves and six good moves. One good is walked
four times, (41, 36) → (41, 34) → (42, 32) → (41, 30) → (40, 28): each
cell it lands on is terraformed by a later section. `calculate_all` is
handed `Some(&rows.goods)`, as Old World's is (§25.3). Without the goods
it would refuse with `Goody`. `Cliffs::init` is inert.

**3944a is decided.** `make_riverlist`'s nearest start breaks ties to the
*first* start. The mutant `d <= best.0` in `sim::map_rivers::nearest_start`
fails `amazon_basin_s_rivers_are_5012_to_5118` and
`amazon_basin_s_troughs_are_894_and_the_start_dump_s_grid` on run1528,
where it failed nothing on run1527.

**Made to fail** (`tools/mutate.py --batch --timeout 900` on `ada08dd63`,
seven breaks, six held): the mask segment at Old World's range; the
tileset always from `default.xml`, and never from it (the two halves of
`chain_tileset`'s branch: the first reddens Amazon, the second Old World);
`AMAZON.xml` replaced by `OLDWORLD.xml`; the style row at 6; Old World's
mask segment. The seventh break passes style 6 in place of 8 to
`place_all_inputs_of`, and it failed nothing. That argument's only reader
is `terrain_groups`' `inputs.map_style != 9` (Himalayas skips
`treeify_mountains`), so 6 and 8 are the same game by construction. The
break to 9 reddens six of the eight checks.

### 26.3 What is not established

- One file only: 2 nations, 60×60, `tropical`. The `jungle` and
  `tropical_wet` draws and other nation counts are untaken.
- The first moved good's new cell (10, 28) reads `down` 3, `down_who` 8 in
  the start dump, against ours (−2, 17), the same pair as Old World's
  (43, 18) (parked 3944c). The four later cells agree with the dump.
  3944c's writer is still unnamed. It now has two cells to be read on
  (parked 3945a).

## 27. Himalayas' chain, 8 of 8 (item 3946)

The second chain take (§22.2, 3944–3953), in §25's shape. The take is
run1529 at `--map 9` on §22.2's line (`docs/RUNS.md`). The chain is walked
against it and against three older files of the same lobby by
`rondata::mapgen::tests::chain_walk`, which is §25's Old World walk made a
style's. Himalayas is **read against**, and no fix was taken (§27.3).

### 27.1 The mask is Great Sahara's listing

`MapHimalayas::make_continents@0069bc90` decompiles to Great Sahara's body.
The two files differ only in the class name. `llvm-objdump -d` of
0x69bc90..0x69be00 and of 0x69bf70..0x69c0e0, with addresses stripped, gives
144 lines each. They differ in four lines, the relative jumps (`jns`, `jns`,
`jle`, `jne`), and each of those targets the same instruction 0x2e0 further
on. Every call names the same absolute target. So the port is
`great_sahara_make_continents`, as it is for Old World (§25.1). Its four
draws are the trace's `make_continents` segment, site for site, inside
0x69bc90..0x69be00 (`rondata::mapgen::himalayas_part`).

`HIMALAYAS.xml` shows what makes the style differ from the others, read
from §4 on:

- **Its own `TILESET`**, Snowy 50 and Arctic 50. The one `init_map_data`
  draw is the style file's own, and on every file the word gives `Snowy`.
- `grouping="1"` home trees at chances 15, 25, 25 and 35, forty-eight
  scaled edge forests (`edge_stay_near`), and `edge_keep_away` mountains
  (§7).
- `numrare="-1"` world bonuses (per player, East Indies' shape) and no
  `GOODIES` section, so `default.xml`'s is used (§8).
- **An empty `RIVERLIST`**, which `river_list` reads as the style's own,
  so there are no rivers (§10).

### 27.2 The disk, and the walk

The four files are run1529 (this item's), and the AI track's held-out-map
captures run348, run464 and run465 (DECISIONS 53 §4, item 972). All four are
one lobby: 2 nations, 60×60, seed 12345. §22.2's on-disk claim read only the
`cover=1` traces, so it missed these three (parked 3946a). The first walk
agrees at all 42 checkpoints of each file, 168 of 168, with no `sim`
change:

- the mask at 7807 (seed 2355140353, part 2 292424319, part 5 1267075849);
- the zero-draw passes, `place_all` and its tail, the resources, and the
  values. That is 90,970 draws from `place_all` through the last
  `compute_val`, site and word, and 7945 is at seed 3700877363, part 5
  2809422360, part 6 156400355;
- `Terrain::init` without heights, 21 records a file, with part 4 at 734
  by `SEA_MAP` 0 and held;
- `make_rivers`' five lines with no draw. The 44 candidate sources are
  listed and none is walked, and the trace draws none either;
- 894: there is no trough, so part 5 is 7945's. The 58,081 start-dump
  vertices of run1529 and run464 agree with the roster stood on them.

**Made to fail** (`tools/mutate.py --batch --timeout 900` on `himalayas`,
`719fc1a96`): five breaks, five held. They were the mask segment moved to
Amazon Basin's range, the tileset drawn from `default.xml`, the walk
filtered to style 8, `place_all`'s map style passed as 6, and Old World's
file read in place of `HIMALAYAS.xml`. No arm was built, so the breaks are
the walk's own choices. No condition here is a disjunction.

### 27.3 What is not established

- **Himalayas is also the AI track's held-out map** (DECISIONS 53 §4). The
  chain took nothing from it, since nothing parted. Whether a parting here
  could be debugged is the commander's question (parked 3946b).
- There is only one lobby: 2 nations, 60×60, `Snowy`. `Arctic`, other sizes
  and other nation counts are untaken.
- A `numrare="-1"` row is backed here as East Indies backs it, on one
  nation count.

## 28. African Watering Hole's chain, 8 of 8 (item 3947)

The fourth chain take (§22.2, §25's method), and the first whose mask is
not Great Sahara's. One take, run1530 at `--map 11` on §22.2's line
(`docs/RUNS.md`), walked phase by phase against its own checkpoints by
the `Chain` walk (§26), now with a mask choice and a water path
(`rondata::mapgen::tests::AFRICAN_WATERING_HOLE`, `MaskPort`). African
Watering Hole is **read against**. It is not the held-out style (§24).

### 28.1 The mask is its own: a lake grown and inverted

`MapAfricanWateringHole::make_continents@0069b650` runs to 0x69bb20, 1,232
bytes against Great Sahara's 368: 376 instructions against 144 in `llvm-objdump -d`,
and six `Random::get@00a39d70` calls against four. So it is not a
dispatch, and the port is `sim::map_watering_hole`, read from the
decompile and checked against the listing where the decompile was
ambiguous (the `grow_region` call's pushes: `size` in `edi`, the radius
at `[ebp−0x14]`; the loop leaves on a zero return, `je 0x69b974`):

1. `avoid_continent` (+0x30) becomes `max(4, scale_land_area(it))`, once.
2. Each generation: `World::wipe`, `Regions::clear_all`; the radius
   `min(m/2 − 4, (m/2)·80/100)`, `m` the shorter side; the angle's two
   halves `(r % 0xffff) << 16 + r % 0xffff` (sites 0x69b78c, 0x69b7a3);
   region 1 opened on `(xs/2 + (r & 1), ys/2 + (r & 1))` (0x69b858,
   0x69b889) and grown by `Map::grow_region` to `World.size·5/100` cells
   within the radius. A growth that ends short starts the generation
   again, angle and all.
3. `Map::invert_land`: the region's cells become water, the rest land.
4. Two fresh parity draws for the ring's centre (0x69ba30, 0x69ba61),
   `start_dist = max(4, xs/16)`, and each player on Great Sahara's ring
   at the last generation's angle stepped by `0xffffffff / players`.

The growers are Great Lakes' (`crate::map_lakes`, §5). `AFRICA.xml`
carries no `AVOID_*`, so `default.xml`'s apply: edges 0 make
`grow_valid` draw nothing, and with one region neither `avoid_center` nor
`avoid_continent` refuses a cell.

On run1530 (2 nations, 60×60): one generation, a 180-cell lake, 91 draws
— the four, then 76 ring picks, 7 `large_stamp` cases and its two side
draws inside `grow_region`, then the ring's two parities. **7807 and the
91-draw stream agree on the first run**, every site the port's. 7807's
seed 3818626210, part 2 267389567, part 5 3682864241 on both sides.

### 28.2 The shared phases on a map with water

- **The tileset**: `AFRICA.xml` has one `TILECHANCE`, `african_savannah`
  at 100, and one entry makes no draw (§4), so `chain_tileset` takes it
  without reading the seed. 7854 agreeing backs it.
- **7835..7945** agree on the first run: 6,044 draws `place_all` through
  the last `compute_val`, whole against the trace's.
- **`Terrain::init`**: the lake opens the half-land gate, so the chain
  walk takes Great Lakes' path (§10, `great_lakes_at_894`) where the
  height-free one refuses with `HalfLand`: 734..706 on the 7945 grid with
  the lists empty; then the heights, `build_coastal_data` and
  `check_and_set_halfland_heights`, whose return is **720** (parts 5, 6 and
  10–12 move there, read off run1530: 720 parted until the check was
  moved before the doobers and the beach); then `create_doobers_flags`,
  `create_doobers` and `convert_beach_to_sandy` before 738. All 22
  records 734..815 agree. Great Lakes' own walk never checked 720..815;
  this one does.
- **Rivers**: 19 sources and no river draw on either side; the 5 records
  5012..5118 agree. The row is `lays_rivers: false` (§27's key, item
  3946), which asserts both streams empty.
- **894**: no trough section, no good moved, `Cliffs::init` inert; seed
  and parts 2, 5, 6 and 10–13 agree.

### 28.3 The mountain mesh reads the texel's red

The start dump's grid parted at 15 of 58,081 vertices, scattered over
cells (40–45, 38–46), each off by `450·3/255` (one byte of a
height-450 mesh) or twice it. No unit stands there; the cells are
mountain (`land_sub` 2, flag 0x200) under the size-3 template placed at
(44, 42), `h4_disp_0.tga`. That is the one template image whose channels
disagree: 314 of its 11,841 opaque pixels differ between B, G and R by a
step or two, where the other six templates are pure grey, so no chain
before this one could tell the channels apart. `mesh_pixels` read the
file's blue (`[b, g, r, a]`'s first byte). Read as red, all 58,081
vertices agree; blue and green each part the same 15. So the texel's
lowest byte that `MountainRange::init`'s `movzx` reads (0x899d90) is the
file's red as the loader leaves it in memory. This is measured, not read:
the loader's byte order is not.

**Made to fail** (`tools/mutate.py --batch --timeout 900`): nineteen breaks
across the three files, sixteen held. In the port: the radius without its
`min`, the lake at 6 % in place of 5 %, the angle's halves swapped, the
lake's x parity negated and its y parity flipped (negating y failed
nothing, since run1530's y draw is even), the restart taken on success
(hung, counted held), `invert_land` dropped, the ring's two parities
negated, `start_dist`'s floor at 5, and `scale_land_area`'s refusal
swallowed. In the walk: the one-entry tileset branch never taken (reddens
African Watering Hole) and taken on any list (reddens Amazon Basin), the
720 check dropped, a river check keyed on a traced file (since replaced
by §27's `lays_rivers` at the merge), and `mesh_pixels` back on blue. Two failed nothing, and both are unreached on
this take (§28.4): the restart never taken, and `avoid_continent`'s floor
at 0. The troughs' `depth` was not broken: no trough section is cut here,
so nothing reads it.

### 28.4 What is not established

- One file only: 2 nations, 60×60. Other nation counts and sizes are
  untaken.
- The restart arm: run1530 grows the lake in one generation. Breaking the
  restart so that it never happens fails nothing. The arm is read from the
  listing (`je 0x69b974` on a zero return) and has no take (parked
  3947a).
- `avoid_continent`'s `max(4, …)`: with one region nothing reads it on
  this mask (parked 3947a).
- The loader that leaves red lowest is not read (§28.3).

## 29. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 30. Australian Outback's chain, 8 of 8 (item 3949)

The fifth chain take (§22.2, §25's method), and the second whose mask
is its own after African Watering Hole's (§28). One take, run1532 at
`--map 13` on §22.2's line (`docs/RUNS.md`), walked phase by phase
against its own checkpoints by the `Chain` walk (§26) with a third
`MaskPort` (`rondata::mapgen::tests::AUSTRALIAN_OUTBACK`). Australian
Outback is **read against**. It is not the held-out style (§24).

### 30.1 The mask is its own: an island grown from the spots and the middle

`MapAustralianOutback::make_continents@0069a650` runs to 0x69add0,
1,920 bytes against Great Sahara's 368: 599 lines of `llvm-objdump -d`
against 144 instructions, four `Random::get@00a39d70` calls but nine
`Map::grow_region@0069c600` calls and two `Map::make_region@0069d3f0`.
So it is not a dispatch, and the port is `sim::map_outback`, read from
the decompile and checked against the listing where the decompile hides
a register (the `project@0092d000` centre in `ecx`/`edx`, the middle's
`(cx, cy)` at `[ebp−0x2c]`/`[ebp−0x1c]`; x out at `[ebp−0x38]`):

1. Each generation: `World::wipe`, `Regions::clear_all`. `half` is half
   the shorter side (`[ebp−0x30]`), the reach `half·55/100`
   (`[ebp−0x14]`), the island `total = size·5/9` (`[ebp−0x10]`).
2. The angle's two halves `(r % 0xffff) << 16 + r % 0xffff` (sites
   0x69a762, 0x69a77a); the middle `(xs/2 + (r & 1), ys/2 + (r & 1))`
   (0x69a833, 0x69a867); `start_dist = max(2, xs/16)` — not Great
   Sahara's 4, so the walk's `start_dist` check takes the port's floor.
3. Player `i` from 1: `project(middle, angle, half·66/100)`, its start
   added, record `i` opened there and given flag bit 1 (+8) and +0xc 1;
   the angle steps by `0xffffffff / players` *after* each, so the first
   player stands at the drawn angle (Great Sahara's steps first). Record
   9 opens on the middle without the flag, +0xc 0. The mask `joined` is
   every player's bit and 0x200.
4. Each player's record grows to `total/players/4`, then every one
   again to `total/players/2`, within the reach of the middle (the
   growers' centre given, `param_4`/`param_5`). Record 9 still lacks the
   flag here, so `grow_valid`'s `avoid_center` test keeps the players
   off the middle. Then record 9 takes flag bit 1 and grows to
   `total/3` within a third of the reach, about its own first cell.
5. Until record 9's adjacency mask (+0x24) holds `joined`: for each of
   records **1..8, whatever the player count**, not yet joined to 9
   (`mask & 0x200` clear): when `count + total/64` passes `total`,
   record 9 grows by `total/64` within a quarter of the reach; otherwise
   the record grows by `total/64` toward the middle, and only when that
   stalls does record 9 grow. A stall of record 9 in either arm starts
   the generation again (`jmp 0x69a680`, all four draws anew). An empty
   record (3..8 on a two-player map) returns 1 from `grow_region` at
   once, so each of them grows record 9 a step every pass.
6. Joined: every player's cells are appended to record 9 and their
   cells' `region` written 9 (no `point`, so no mask), record 9 grows to
   `total` within the reach, `Map::eliminate_pools@0068b890` on the
   whole world keeps the largest water, and `Regions::clear_all`.

**What the growers had to learn.** Every mask before this one opened
its regions with the map's `region_flags`, 0 on every corpus style, so
the growers' port carried neither the flags nor the masks (§5). This one
sets flag bit 1 itself, and two arms wake: `Map::grow_valid@0069d000`'s
two refusals — another live region with the point within
`avoid_center` of the map's centre, and another region inside the
`avoid_continent` disc — both pass over a record that shares bit 1 with
the grower; and `Map::point@0069d5a0`, for a record with bit 1, meets
each other region among the cell's nine neighbours (`move_x`/`move_y`)
and writes the union of both masks and both bits into both and into
every one of the first 32 records whose mask meets it. Both live in
`sim::map_lakes` (`Lakes::flags`, `Lakes::adjacent`,
`Grower::grow_region_about`), inert on every flag-0 style: the other
styles' chains, East Meets West's and Himalayas' among them, held.

On run1532 (2 nations, 60×60, `OUTBACK.xml`'s four `AVOID_EDGE` at 4):
one generation, **17,262 draws** — the four, then 10,116 edge draws in
`grow_valid`, 1,989 gates, 1,973 outer picks, 1,456 whole-region and 681
ring picks in `grow_region`, 943 `large_stamp` cases and 100 side draws.
**7807 and the whole stream agree on the first walk**: seed 2524927299,
part 2 181013119, part 5 1934301065, part 6 1792933889 on both sides.

### 30.2 The shared phases, first walk

- **The tileset**: `OUTBACK.xml`'s four `TILECHANCE`s at 25 draw
  `autumn` off 7805's seed; 7854 agreeing backs it.
- **7835..7945** agree: 4,678 draws `place_all` through the last
  `compute_val`, whole against the trace's.
- **`Terrain::init`**: the sea opens the half-land gate, so the walk
  takes §28.2's path; all 22 records 734..815 agree, 720 among them
  (part 5 4114045574 on both sides).
- **Rivers**: an empty `RIVERLIST`; 21 sources and no river draw on
  either side; the 5 records 5012..5118 agree. The row is
  `lays_rivers: false`.
- **894**: seed 85646897, parts 2, 5 (2363972222), 6 (3983024254) and
  10–13 (942242802, 515048837, 1919615731, 262145) as the file's; no
  good moved; all 58,081 start-dump vertices.

No shared phase parted, and no shared fix was taken.

### 30.3 Made to fail

Each arm first against a unit test (`sim::map_outback::tests`, and
`map_lakes::tests::a_join_reaches_every_record_whose_mask_meets_it` for
`point`'s join), then `tools/mutate.py --batch --timeout 900` on the
walk: thirty breaks, then six and one more, **twenty of the thirty
targets held** (the walk, or the join's unit test for the two
propagation breaks). Held: the island's `5/9`, the angle's halves
swapped, both middle parities, `start_dist`'s floor at 4, the ring and
the reach once broken past their rounding (`66 → 65` and `55 → 56` give
19 and 16 either way on 60×60; `60` and `50` hold), the first player a
step on, either flag bit dropped, the second share at 3, record 9's
`total/3` at `/2`, the merge loop's exit, the last growth at
`total·9/10`, and in the growers the shared-bit exemption gone (whole,
and each of `grow_valid`'s two refusals alone), the masks never kept,
the join pass dropped or cut to nine records, and the centre left to
the first cell. Ten failed nothing, each for a reason read off the take:

- **The merge loop's body and the restart** (seven: the player's
  centre, `total/64`, the loop's test made any-bit, records `1..=8`
  made `1..=players`, `<` made `<=`, the `&&` made `||`, the restart
  never taken): record 9's `total/3` growth joins both players at once,
  so no pass runs (`Outback::passes` 0). Not a port's blind spot: every
  pass draws, and the original's 17,262-draw stream is whole, so it took
  none either. 112 lobbies of the port (40–100 cells a side, 2–8
  nations, four seeds) take no pass and no restart (parked 3949a).
- **Record 9's reach** (`reach/3`, broken to `/2` and to 1): every
  outer pick it draws lies 9 to 29 from the middle, so any reach under 9
  refuses all of them alike.
- **The absorb's `region` write**: an equivalent mutant here — every
  record shares bit 1, so no refusal reads which record a cell is in,
  and `clear_all` follows.
- **`eliminate_pools`**: the island encloses no pool on run1532.

### 30.4 What is not established

- One file only: 2 nations, 60×60. Other nation counts and sizes are
  untaken; a map with more than two players makes fewer empty records
  among 1..8, and the step that grows record 9 for each is read, not
  taken beyond two.
- The merge loop's body and the restart: unreached on run1532 and on
  every lobby the port was run on (§30.3, parked 3949a); read from the
  decompile and the listing (`jle 0x69abd0`, `je 0x69ac1e`, `jmp
  0x69a680`), no take decides them.
- `eliminate_pools` and record 9's reach under 9: nothing on this take
  reads them (§30.3).
- `point`'s propagation is a single pass over the first 32 records, as
  read; whether a second join in one `point` could order differently is
  not decided by a take with two players.

## 31. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 32. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 33. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 34. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 35. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 36. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.

## 37. Reserved for the next item on the map track

A stub the opening booking lands so a lane's rewrite and the next booking
never touch adjacent lines (parked 1741): the booking assigns it by number
in the queue's own words and edits nothing here; the item's worker renames
the heading and writes the section.
