# CHECKSUM — the original's sync checksum, its oracle and its instrument

Item 4060, the first item of step 5 (`docs/TRACKS.md`, "The milestone";
`docs/DECISIONS.md` entry 119). It changes no simulation code. It builds
what every later checksum item diffs against: a reader for the record the
original prints, an instrument that re-runs the original's own walks on a
packet and prints every byte they hand over, and the ledger of those bytes.

**Confidence.** The arithmetic, the fifteen lines and the pairing of a
packet with its record are **diff-backed**: on run1100's packet the
instrument reproduces all fifteen printed lines of record 3502, and on
run 1562's — the minted game's playback, item 4088 — all fifteen of
record 3201. On both it parts, as it should, from the record one tick
later (§3). The ledger's field names come from the type records
(`types.txt` in the decompile export, and the PDB's own field lists
where the export's record is incomplete). Its byte counts are measured
on those two frames (§4). The first state line ported, items, agrees
byte for byte on run 1562's frame (§11), and so does the second, cities
(§12). Goods is held out on one byte, good 15's `ever_seen` (§13). What
rests on a reading alone is listed in §10.

## 1. What the checksum is

- **A walk folded by Adler-32.** A checksum is a `DataWalk` whose fold,
  `CheckSum::walk_function@00936ff0`, takes each byte range a
  `walk_data` hands it, adds its length to `size` and folds its bytes into
  `accum` with the original's Adler-32, `adler32@00a46830` (zlib's: modulus
  65521, the sum seeded from `accum`). Order matters, and every byte of
  each range goes in raw. That includes padding, and it would include a
  pointer or a float if a range spanned one (§9).
- **One walk, one figure, outside the fold too.** `CheckSums::check_groups@00937530`
  folds eight words through zlib's own `adler32@005089d0` after its walk.
  They are copied to a stack local, so they never reach the fold. The
  instrument's first run parted from the original's `groups` sum on
  exactly those 32 bytes (§3).
- **Fifteen lines.** `CheckSums::check_all@00936560` resets `accum` to 1
  before each of units, builds, walls, ammo, deaths, groups, guys,
  leaders, cities, items, goods, world, rules, scenario data and script
  run time. It logs each line through `SyncLogger::logToMemory<unsigned_long_&>@00463930`,
  and their wrapping sum as `total`. The first twelve are the simulation's
  state; rules is the data layer's; the last two are the scripting
  machine's image (DECISIONS 119).
- **Two records a frame, inside a `DUMP_ALL` window only.**
  `Game::do_frame@00591ef0` calls `GameLog::begin_frame@00932a70`, runs
  the tick, then calls `GameLog::end_frame@009329d0`. Each calls
  `GameLog::full_dump@00930380`, which, with the dump-all flag set, prints
  `GameLog::say_checksum@00930b30`'s record before the dump. So a windowed
  frame `n` prints a **begin record** (the first child of `BEGIN FRAME n`)
  and an **end record** (beside the block, after the tick).
- **A packet at logger frame `n` is the end record's state.** The packet's
  boundary is `GameLog::end_frame` entry, before `full_dump`
  (`tools/explore/frame_snapshot.py`). The end record of frame `n` is
  computed on exactly the state the packet copies. That is diff-backed
  (§3): the instrument on run1100's packet (logger frame 5408) reproduces
  record 3502, frame 5408's end record. It parts from 3503, frame 5409's
  begin record, on the six lines frame 5409's tick moves.

## 2. The record, and its reader

`rondata::checksum` (`crates/rondata/src/checksum.rs`) reads every
`CHECKSUM n` record of a log whole: its number, `FILE`, `LINE`, then every
labelled line in print order — a prefix of `RECORD_LABELS`'s 64 labels
(the eleven subsystem lines with the sixteen `World` lines among them,
`Rules`, `game_random seed`, `Game`, and level 16's 34). `Record::get`
gives a line by label (`World` is the sixteenth, the whole world);
`Record::fifteen` gives `check_all`'s fifteen in its order.

Each record is placed by the hook that printed it: `Setup` (any `FILE`
other than `gamelog.cpp`), `Game` (`begin_game`'s, before the first frame
block), `Begin(n)` (the line before is `BEGIN FULL DUMP` and the one
before that `BEGIN FRAME n`), `End(n)` (any other windowed record after
`BEGIN FRAME n`). It reads lines, not the block tree: a frame's end record
sits beside its block, and a window the tracer's `@dumpall` opens mid-frame
prints that record with no `BEGIN FULL DUMP` above it at all (run1100's
3502), so the tree has no one place for it. Labels are matched in print
order, so a `label value` line past a record's last line is not taken
into it.

**The hook is where the record printed, not which half of the tick it
is.** An `@dumpall` window holds two records: the first is the
packet's state, the second is one tick later (§3, on both pairs). Where
the frame's block is already open when the window starts, the first
prints beside it and reads `End(n)` (run1100's 3502). Where nothing
else prints in the frame, the first opens the block and reads
`Begin(n)` (run 1562's 3201, under `BEGIN FRAME 91`; its second, 3202,
reads `Begin(92)`). A packet at logger frame `n` reproduces the first
windowed record of frame `n` either way, and
`checksum::tests::the_instrument_reproduces_the_printed_record` takes it
so.

`gamelog::Log::frame_seeds` reads the begin record's seed and calls it
engine frame `n − 1`'s; the reader agrees with it on run1100 (its test,
`run1100_s_records_are_read_whole`). The reader is not one of
`diff::oracle`'s `SCHEMA_SOURCES`, so editing it writes no oracle-cache
generation.

Its tests: a synthetic log (all four hooks, a level-14 setup record, a
label out of print order); a changed line that parts on that line alone;
run1100's two records, each 64 lines, its quiet lines (`searcher`,
`scene`, `message_win`, `storm`, `Ammo`, `Walls`, `Deaths`) reading 1 and
its busy ones not — a reader that defaulted to 1 would pass the first half
and fail the second — and the seed held against `frame_seeds`; and the
instrument's values (§3). All four were red first under a stubbed reader.

## 3. The instrument

`tools/recomp/checksum.py <exe> <packet> [--gamelog F --record N]
[--stream] [--ledger] [--values]` enters `CheckSums::check_all@00936560`
on a packet under unicorn, the way `tools/recomp/step4.py` enters any
function (`docs/EMULATOR.md` §8): the capturing thread's TEB as `fs:`, a
fresh stack, the clock and heap imports stubbed (plus `strrchr`). Its two
sync-logger calls are answered in Python, and each value-carrying one
ends a line and carries the original's `accum`. The fold
(`CheckSum::walk_function@00936ff0`) and both Adler-32s are hooked at
entry and left to run, and each range is recorded before it is folded:
line, owner (the function of the call site, and its `this` when that
function is the innermost live walker frame), site, address, length,
bytes. The tool's own Adler over the stream is a second sum, held against
the original's line by line. It takes 8–18 s.

**Reproduced on run 1562** (item 4088; the minted game's playback,
packet `~/ron-golden/run1562/map-14/frame-snapshot.bin`, logger frame
91; record 3201 of its `gamelog_p.txt`, archived as
`gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt`; 9.2 s):

| line | printed (3201) | instrument | ranges | bytes |
|---|---|---|---|---|
| units | 27230589 | 27230589 | 1084 | 19,642 |
| builds | 2869973639 | 2869973639 | 519 | 4,923 |
| walls | 1 | 1 | 0 | 0 |
| ammo | 1 | 1 | 0 | 0 |
| deaths | 1 | 1 | 0 | 0 |
| groups | 3941922214 | 3941922214 | 556 | 37,010 |
| guys | 2502350161 | 2502350161 | 368 | 9,308 |
| leaders | 2813959641 | 2813959641 | 267 | 58,343 |
| cities | 1233128295 | 1233128295 | 6 | 228 |
| items | 1548048405 | 1548048405 | 110 | 484 |
| goods | 1939436390 | 1939436390 | 180 | 792 |
| world | 3643872705 | 3643872705 | 7994 | 294,769 |
| rules | 314192132 | 314192132 | 251160 | 997,846 |
| scenario_data | 404497739 | 404497739 | 4082 | 8,453 |
| script_run_time | 3867914385 | 3867914385 | 1307 | 31,530 |

15 of 15, the stream's own sum equal to the original's on every line;
`total` 3631689818. Against 3202, one tick later, 10 of 15: units
1527086071, builds 978539139, guys 2918305641, leaders 1852546327 and
world 2695435849 printed part from the packet's. The rules line is
run1100's 314192132 again (§10).

**Reproduced on run1100** (item 4060; packet `~/ron-golden/run1100/map-14/frame-snapshot.bin`,
logger frame 5408; record 3502 of its `gamelog.txt`, archived as
`gamelog-run1100-greatlakes-team-herd-packet-5408.txt`):

| line | printed (3502) | instrument | ranges | bytes |
|---|---|---|---|---|
| units | 2947883129 | 2947883129 | 2262 | 41,193 |
| builds | 3850321411 | 3850321411 | 1518 | 14,355 |
| walls | 1 | 1 | 0 | 0 |
| ammo | 1 | 1 | 0 | 0 |
| deaths | 1 | 1 | 0 | 0 |
| groups | 3519151140 | 3519151140 | 556 | 37,010 |
| guys | 357023178 | 357023178 | 727 | 19,033 |
| leaders | 867664653 | 867664653 | 399 | 87,517 |
| cities | 1678386672 | 1678386672 | 31 | 630 |
| items | 3751690511 | 3751690511 | 80 | 352 |
| goods | 109026513 | 109026513 | 230 | 1,012 |
| world | 3084183382 | 3084183382 | 8103 | 300,441 |
| rules | 314192132 | 314192132 | 251160 | 997,846 |
| scenario_data | 3465681875 | 3465681875 | 4085 | 8,463 |
| script_run_time | 1667663035 | 1667663035 | 1307 | 36,016 |

15 of 15, and the stream's own sum equals the original's on every line.
`total` is 4138031154. The first run's stream sum parted on `groups`
(2145712412 against 3519151140): that run hooked only the fold, and
`check_groups`' eight words go through zlib's Adler. Both Adlers are
hooked now.

**The comparison parts where it should.**

- *The same packet against record 3503* (frame 5409's begin record, one
  tick later): 9 of 15 agree. Six lines part: units 3427215241, builds
  2673950517, guys 1295434655, leaders 2952299625, goods 1211997648,
  world 670623352 printed, against the packet's values above.
  `rondata::checksum`'s test pins that list.
- *run1420's packet (logger frame 4586, another game) against 3502*: 2 of
  15 agree, walls and ammo, both empty in both games (Adler 1). Deaths
  parts at 307509077 against 1. So does rules, at 1227894240 against
  314192132: the rules line differs between these two games, and §10
  says what that leaves open.

The stream, the ledger and the values file stay outside git under
`~/ron-data/lab-experiments/2026-11-10-item-4060/run1100/` and
`…/run1562/`, since they are the original's memory.

## 4. The ledger: the twelve state lines on run 1562's frame 91 and run1100's 5408

Per line: elements walked, ranges, bytes, the walker, the layout's
pointers and floats. The per-range detail is §6–§8, and the classes of
byte (pointers, floats, padding, locals) are §9.

**Run 1562, logger frame 91** (item 4088; the score's frame, §5), the
minted game one frame after its last order:

| line | elements | ranges | bytes | pointers | floats |
|---|---|---|---|---|---|
| walls | 0 walls | 0 | 0 | — | — |
| ammo | 0 shots in flight | 0 | 0 | — | — |
| deaths | 0 deaths | 0 | 0 | — | — |
| cities | 2 cities, 0 caravan links | 6 | 228 | none | none |
| items | 22 items | 110 | 484 | none | none |
| goods | 36 goods | 180 | 792 | none | none |
| builds | 15 buildings, 100 queue items, 143 mining tiles | 519 | 4,923 | none | none |
| guys | 54 guys in 52 units | 368 | 9,308 | none | 5 per guy |
| units | 52 units, 54 guys, 4 path nodes, 13 orders | 1084 | 19,642 | none | 5 per guy |
| groups | 512 group slots, 6 in use | 556 | 37,010 | none | none |
| leaders | 8 slots, 2 walked whole | 267 | 58,343 | none | 2 per leader |
| world | 3,600 cells, 25 collision blocks, 692 terrain coords | 7994 | 294,769 | none | none |
| **the twelve** | | 11,084 | **425,499** | **none** | **the same 7 fields** |

The same seven floats and no pointer, by the same type records. Walls,
ammo and deaths are empty here too, so they are not counted on this
frame (`docs/DECISIONS.md` entry 120 §2).

**Run1100, logger frame 5408** (item 4060):

| line | walker | elements | ranges | bytes | pointers | floats |
|---|---|---|---|---|---|---|
| walls | `CheckSums::check_walls@00937360` | 0 walls | 0 | 0 | — | — |
| ammo | `CheckSums::check_ammo@009374e0` | 0 shots in flight | 0 | 0 | — | — |
| deaths | `CheckSums::check_deaths@00936bb0` | 0 deaths | 0 | 0 | — | — |
| items | `CheckSums::check_items@00937790` | 16 items | 80 | 352 | none | none |
| cities | `CheckSums::check_cities@00937600` | 5 cities, 4 caravan links | 31 | 630 | none | none |
| goods | `CheckSums::check_goods@00937710` | 46 goods | 230 | 1,012 | none | none |
| builds | `CheckSums::check_builds@00937290` | 39 buildings, 282 queue items, 516 mining tiles | 1518 | 14,355 | none | none |
| guys | `CheckSums::check_guys@00937430` | 111 guys in 101 units | 727 | 19,033 | none | 5 per guy |
| groups | `CheckSums::check_groups@00937530` | 512 group slots, 6 in use | 556 | 37,010 | none | none |
| units | `CheckSums::check_units@009371d0` | 101 units, 111 guys, 87 path nodes, 47 orders | 2262 | 41,193 | none | 5 per guy |
| leaders | `CheckSums::check_leaders@009375a0` | 8 slots, 3 walked whole | 399 | 87,517 | none | 2 per leader |
| world | `World::walk_data@006b5cf0` (part −1) | 3,600 cells, 79 collision blocks | 8103 | 300,441 | none | none |
| **the twelve** | | | 13,906 | **501,543** | **none** | **7 fields** |

**No walked range of the twelve holds a pointer** on this frame, by the
type records. Each walker stops short of its record's pointers: a
`SubObject`'s `ptype` at +0x18, an `ObjectData`'s `launching` at +0x44, a
guy's `curr_bbox` and `last_good_river`, a `WData`'s `block` at +0x18, the
arrays' `list`, a unit's path-finding trees at +0x100 on. **Seven float
fields are walked raw**: `GuyData` `turret_inc`, `bank`, `last_bank`,
`pitch`, `last_pitch` (+0x40..+0x54, in both guys and units), and
`LeaderData` `anti_att` (+0x7f4) and `plunder_scale` (+0x914). So a port
that emits these lines must hold those seven as the original's exact
IEEE-754 bit patterns. That is the residue's question, and §10 states it.

**Cheapest first, on run 1562's frame 91** (item 4088), the order the
items after 4088 follow. The three empty lines are not counted there
and are named last; by bytes:

1. cities (228; ported, §12) · 2. items (484; ported, §11) · 3. goods
(792; held out on one byte, §13) ·
4. builds (4,923) · 5. guys (9,308) · 6. units (19,642; it walks the
guys again) · 7. groups (37,010) · 8. leaders (58,343) ·
9. world (294,769) · then walls, ammo and deaths (0 each), counted on a
frame that holds one.

Cities and items swapped places against run1100's frame (630 and 352
there). Goods is the record items is, over another array; its one
parting byte is a setup writer our built start does not run (§13).

*On run1100's frame 5408* (item 4060), by the bytes there, with the
three empty lines first:

1. walls (0) · 2. ammo (0) · 3. deaths (0) · 4. items (352) ·
5. cities (630) · 6. goods (1,012) · 7. builds (14,355) ·
8. guys (19,033) · 9. groups (37,010) · 10. units (41,193; it walks
the guys again) · 11. leaders (87,517) · 12. world (300,441).

Two orderings are not bytes alone. Guys before units, because the
units line contains the guys line's whole per-guy walk. Groups'
36,864 bytes are 512 slots of one 72-byte range, so it is cheaper than
its size. The empty lines are free on this frame and not established on
any other: run1420's packet walks 4 deaths, 300 bytes (§8).

## 5. The score

The queue's `Checksum:` line is `rondata::diff::floors::checksum_line`
of `checksum::CHECKSUM_REPRODUCED` (the instrument's lines reproduced, of
fifteen), `CHECKSUM_PACKET` (run 1562, logger frame 91, record 3201;
item 4060's was run1100, 5408, 3502) and `CHECKSUM_STATE` (the state
lines our simulation emits equal to the original's, of twelve, and the
bytes of the twelve walks ours reproduces, of 425,499 on that frame):

    Checksum: instrument 15 of 15 lines (run1562 f91) · state 2 of 12 lines, 712 of 425,499 bytes

Item 4088 moved it from `instrument 15 of 15 lines (run1100 f5408) ·
state 0 of 12 lines, 0 of 501,543 bytes`: the pair moved to the minted
game, and items is ported (§11). Item 4091 moved it from `state 1 of 12
lines, 484 of 425,499 bytes`: cities is ported (§12), and goods is held
out on one byte (§13).

`floors::tests::the_handoff_s_checksum_line_is_the_pinned_counter` holds
the queue's line against it. `checksum::tests::the_instrument_reproduces_the_printed_record`
holds the 15 against the instrument's values file and the archive's
record, the twelve state lines' bytes against `CHECKSUM_STATE`'s whole,
and run1100's pair beside it (it skips, saying so, on a machine without
them). `diff::checksum_lines` holds each ported line's bytes and its
share of `CHECKSUM_STATE`. An item that ports a line re-pins
`CHECKSUM_STATE`. The driver writes the queue's line.

## 6. Ledger: units and guys

Offsets are in the walked object's record. A local is a value the walker
computes into its stack and folds (§9).

**Units** (`CheckSums::check_units@009371d0`): per active leader, the
objects list at `objects +0x1e8`, units with `flags & 1`, each through
its `vtable +0x7c`, i.e. `Unit::walk_data@0060cf40`. It sets the walk's
flags to −1 itself, so path, orders and guys are all walked. Per unit,
in walk order:

| walker | range | record and fields | ×, bytes |
|---|---|---|---|
| `SubObject::walk_data@006621d0` | [+0x8, +0x9) | `SubObjectData` `flags` | 101, 101 |
| | [+0x9, +0x18) | `who`, `o`, `z_internal`, `x_internal`, `y_internal` (stops before `ptype`) | 101, 1,515 |
| | 4-byte local | (site 00662287) | 101, 404 |
| `Object::walk_data@00647830` | [+0x20, +0x42) | `ObjectData` `myhits`..`launch_frames`, 19 fields (stops before `launching`) | 101, 3,434 |
| | 1-byte local | `Object::must_walk@00647930` (site 00647910) | 101, 101 |
| `Unit::walk_data@0060cf40` | 1-byte local | (site 0060cfae) | 101, 101 |
| | [+0x48, +0xb7) | `UnitData` `collide_frame`..`play`, 49 fields, two int/short unions (+0x54 `rare`/`air_alt`/`former_type`, +0x86 `hero`/`cara`/`doober`/`special`/`herd`) | 101, 11,211 |
| | 1-byte locals ×2 | `Unit::must_walk@0060d040` (site 0060d07f) | 202, 202 |
| `Stack<PathData>::walk_data@0046d8b0` | [+0xb8 +4, +0xb8 +0xd) | `Stack<PathData>` `size`, `length`, `increment` (`list` skipped) | 101, 909 |
| | elements, 16 B | `PathData` `to_x`, `to_y`, `tolerance`, `flags` | 87, 1,392 |
| `OrderList::walk_data@00730270` | 4-byte local | the order count | 101, 404 |
| | per order: 4-byte local, node byte | the order's type (its `vtable +0x10`), the node's byte +0xc | 47, 188 + 47 |
| `GatherOrder::walk_data@00486e60` | [+0x8, +0x12) | `TargetOrder` `ox`, `whom`, `uid` (PDB) | 37, 370 |
| | [+0x14, +0x28) | `tx`, `ty`, `build_type`, `wait`, `goto_build`, `non_flat_gather`, `dist_mod`, `been_there` | 37, 740 |
| | 1 byte | `UnitOrder` `flags` (the virtual base, +0x2c) | 37, 37 |
| `AttackToOrder::walk_data@00482e70` | [+0x4, +0x50) | `MoveOrder` `x`..`off_y`, 19 fields (PDB) | 8, 608 |
| | 1 byte | `UnitOrder` `flags` (+0x54) | 8, 8 |
| `TradeOrder::walk_data@00483260` | [+0x8, +0x12), [+0x14, +0x26) | `TargetOrder` `ox`, `whom`, `uid`; `oxx`, `whose`, `started`, `loaded`, `uid2` | 2, 20 + 36 |
| | 1 byte | `UnitOrder` `flags` (+0x2c) | 2, 2 |
| `PtrArray<Guy>::walk_data@0046df30` (+0xe4) | locals and header | count (4), size (4), [+0x8, +0xe) and [+0xc, +0xe) of the array, flags masked `& 0xbf` (1) | 101, 1,717 |
| | per guy: 1-byte local | (site 0046e199) | 111, 111 |
| `GuyData::walk_data@005e0210` | [+0x8, +0xa3) | `GuyData` `type`..`guy_num`, 42 fields, **5 floats** (§4) | 111, 17,205 |
| `AnimalData::walk_data@005d7ea0` | 1-byte local; [+0x150, +0x155) | `AnimalData` `ox`, `whom`, `aid` (55 of the 101 are animals) | 55, 55 + 275 |

The `TargetOrder` and `MoveOrder` fields are absent from the export's
records for these orders (multiple inheritance with a virtual base). The
PDB's field lists name them (`llvm-pdbutil dump --types`): `TargetOrder`
holds its `vfptr` at +0, its `vbptr` at +4, and `ox`, `whom`, `uid` at
+0x8, +0xc, +0x10; `MoveOrder` holds its `vbptr` at +0 and `x`..`off_y`
at +0x4..+0x50. The walk is entered on the `UnitOrder` virtual base, which
sits at +0x2c of a `GatherOrder` or `TradeOrder` and at +0x54 of an
`AttackToOrder`.

**Guys** (`CheckSums::check_guys@00937430`): per active leader, per unit,
the same `PtrArray<Guy>::walk_data@0046df30` as above: its header and
locals (101 × 17 bytes) and each guy's 1-byte local and
`GuyData::walk_data@005e0210` range (111 × 156). That is 19,033 bytes,
all of them also inside the units line.

## 7. Ledger: builds, groups, leaders

**Builds** (`CheckSums::check_builds@00937290`): per active leader, the
list at `objects +0x1ec`, each building through `vtable +0xac` into
`BuildData::walk_data@0062f270`. Per building (39): `SubObject` as for
units (1 + 15 bytes and a 4-byte local) and a 2-byte `Object::must_walk`
pair. Then `Object` [+0x20, +0x42) (34) with its 1-byte local, and
`WallData::walk_data@00642510` [+0x48, +0x66), `job_counter`..`demolition`
(30), with its 1-byte local. Then `BuildData` [+0x6c, +0x70)
`orig_type`, [+0x70, +0x86) `gather_down`..`infiltrate2` (22, one
`fort`/`dock`/`oil_well`/`farm` short union), [+0x7f, +0x80) `founder`,
[+0x83, +0x84) `max_age`, [+0xb4, +0xb6) of `gather_from` (`mtn`,
`cliff`), and a 1-byte local. Then `BuildQueue::walk_data@006305f0`:
[+0x0, +0x4) `queue_size`, then 18 of each `QueueItem`'s 20 bytes
(`job_counter`, `type`, `good[3]`, `cost[3]`; 282 items, 5,076 bytes; the
record's last two bytes are not walked). Then the `gather_from`
`Array<TCoordData>` (516 tiles × 8, 4,128, with its header) and the
`GatherPointList` count local.

**Groups** (`CheckSums::check_groups@00937530`): as many slots of the
groups array as its count at +0x4 says (512 here), `Group::walk_data@00708400` [+0x4, +0x4c) — `GroupData`
`id`..`march`, 21 fields, 36,864 bytes. For each slot whose `num` is not
0 (6 here, each `num` 1): `num` entries of `off_x`, `off_y`, `curr_x`,
`curr_y` (4 bytes each), `list` (2) and `angles` (1). Then the eight
ints of the array the groups object points to at +0x3c, folded by zlib's
Adler from a stack copy (32 bytes; §1).

**Leaders** (`CheckSums::check_leaders@009375a0`, the same loop as
`check_all`'s): every slot of the `leaders` array to 0xe71af0, through
`LeaderData::walk_data@006d6750`. Each of the 8 slots hands over
[+0x0, +0x8), `leader_flags` and `leader_flags2` (64 bytes). The 3 slots
walked whole (here the leaders at 0x00e3a390, 0x00e4127c, 0x00e48168)
also hand over:

- [+0x8, +0x692a): `who`..`ally_mask`, 276 leaves, 26,914 bytes,
  among them **`anti_att` (+0x7f4) and `plunder_scale` (+0x914), floats**,
  and two uncovered runs, +0x606e..+0x6070 (after `num_queued[806]`) and
  +0x6922..+0x6924 (after `num_ctw_rate_bonuses[6]`), zero in all three.
- `dip[0..8]`, eight `Diplomacy` records of 92 bytes; `pers`, the
  `Personality`, [+0x6dd4, +0x6e34), 24 ints.
- Nine `BitMask`s: each one's `bits` and `size` (8 bytes), then its
  `size` bytes of the inline `ptr` array. `tech`, `tech_at_start` and
  `obs_flags` take 101 bytes each; `conquest_wonders`,
  `conquest_wonders_in_game` and `conquest_racial_powers` take 3; `rare`,
  `rare_owned` and `rare_conquest` take 6. Despite its name, `ptr` is a
  `uchar[]` inside the record, not a pointer.
- `sites` (`Array<Site>`, 30 × 24), `make_list` (`Array<MakeObject>`,
  33 × 40), `mil_trainers`, `new_rares`, `oil_patches`
  (`SimpleArray<int>`), each with its header.
- `prod_script`, a `String` (`String::walk_data@00a1b2d0`): its length as
  a 4-byte local, then its characters (two leaders, 16 bytes each).
- `data_encrypted` through `LeaderDataEncrypt::walk_data@006d9900`: the
  decrypted values as 4-byte locals, 62 words a leader. That is six at
  each of nine sites (the six-resource arrays), four at one, and one at
  each of four more. The encrypted words are not walked; their plain
  values are.

## 8. Ledger: cities, items, goods, the world, and the three empty lines

**Cities** (`CheckSums::check_cities@00937600`): per active leader, per
`City` with `flags & 1`, [+0x4, +0x6) `city_flags` and [+0x6, +0x72)
`city`..`ter` (108 bytes; `vfptr` at +0 and the two `String` names at
+0x90 and +0xa4 are skipped — an input walk alone takes them). Then
**that city's own** `vans` (`Array<CaravanLink>` at +0x74) through
`Array<CaravanLink>::walk_data@00489040` — per city, not once after
the last, as the recon read it (the stream settles it, item 4091): its
length as a 4-byte local; with links, its size (a 4-byte local, 10 on
every link array read), `increment` [+0xc, +0xe) (−1), the flags byte
`& 0xbf` (a 1-byte local) and 8 bytes a link, `cara` and `who`. An
empty array is its length alone. 5 cities and 4 links on run1100's
frame (31 ranges); 2 cities and no link on run 1562's (6 ranges, 228
bytes; §12).

**Items** (`CheckSums::check_items@00937790`) and **goods**
(`CheckSums::check_goods@00937710`): each live record (`flags & 1`, the
matching vtable) hands over [+0x20, +0x21) `ever_seen`, then `SubObject`
[+0x8, +0x9) `flags`, `SubObject::must_walk@006623a0`'s 1-byte local,
[+0x9, +0x18) `who`..`y_internal` and a 4-byte local, in that order
(the stream's; §11 names the locals): 22 bytes a record, so 16 items
make 352 bytes and 46 goods make 1,012.

**World** (`World::walk_data@006b5cf0` with part −1, so every section).
The walker reads its world from the global at 0x00c06188, not from
`ecx`. The record is `WorldData`, at +0 of `World`. This map, Great Lakes,
has 60 × 60 cells, 240 × 240 tiles, 120 × 120 fog cells and 900 regions.
Section by section:

| section | range | bytes |
|---|---|---|
| 1 | [+0x0, +0x8) `xs`, `ys` | 8 |
| 2 | `start_x`, `start_y`, `start_city_x`, `start_city_y` (`SimpleArray<WCoord>`): each one's count and size locals, `increment`, flags local, then its elements; two arrays of 12 and two of 3 | 120 + 44 |
| 3 | `oil_x`, `oil_y`: both empty here, so each is its zero count alone (the record's 524289, the Adler-32 of 8 zero bytes) | 8 |
| 4 | [+0x8, +0x80) `size`..`seed`, 30 ints | 120 |
| 5 | each `WData` cell, [+0x0, +0x15) of its 0x1c: `flags`..`was_seen`, 17 fields (stops before `block`) | 3,600 × 21 = 75,600 |
| 6 | `tdata`: `tile_size` × 2 (`TData` `mask`); `seen`, `seen2`, `seen3`: `fog_size` bytes each | 115,200 + 43,200 |
| 7 | `wcoord_seen`: `size` bytes | 3,600 |
| 8 | `danger[0..8]`: `reg_size` × 4 each | 28,800 |
| 9 | each cell's collision block: a 4-byte presence local, then for each of the 79 blocks [+0x0, +0x8) and [+0xc, +0xc + its size) | 14,400 + 632 + 7,584 |
| 10 | the terrain's `Array<WCoordData>` at +0x4b80 | 693 × 8 = 5,544, header 11 |
| 11–13 | the terrain's `SimpleArray<int>`s at +0x4b9c, +0x4bb8, +0x4bd4 | two of 693 ints, one empty: 5,544, headers 26 |

Those total 300,441. The record's sixteen `World` lines are sections 0
to 14 alone, then all of them. The walker has no block for section 0
or 14, so both print 1. Section 13, an empty `SimpleArray<int>`, prints
262145, the Adler-32 of its 4-byte zero count.

**The three empty lines.** `CheckSums::check_walls@00937360` walks each
wall (objects list +0x1f0, `flags & 1`) through `vtable +0x7c` into
`WallData::walk_data@00642510`. `CheckSums::check_ammo@009374e0` walks
each shot in flight through `vtable +0` into `AmmoData`'s walker.
`CheckSums::check_deaths@00936bb0` walks each death record of 0xa4 bytes
whose first word is not 0: 4 bytes, then [+0x4, +0x4b). None of the
three has an element on run1100's frame. On run1420's packet the deaths
line walks 8 ranges of 300 bytes in all, which is consistent with 4 × (4 + 71). The
walls and ammo record shapes rest on the recon's reading alone (§10).

## 9. Pointers, floats, padding and locals

- **Pointers: none walked**, in any of the twelve on this frame (§4).
  Every range was mapped onto its record's leaves (`tools/recomp/checksum.py
  --ledger`, nested records and arrays expanded). No pointer-typed leaf
  overlaps a walked range: the `ptr` of a `BitMask` and the `list` of a
  `PathData` stack are named so, but the first is an inline array and the
  second is skipped.
- **Floats: seven fields**, named in §4. The original holds them as `f32`
  and folds their bytes. `docs/DECISIONS.md` entry 16 keeps floats out of
  the simulation, so a line that walks one is exact only if those bits
  can be derived from integers the port does hold. That is open (§10).
- **Unions: four**, `UnitData` +0x54 and +0x86 and `BuildData` +0x78, all
  int or short members, and `String`'s +0x0 (skipped).
- **Padding inside a walked range: 4 bytes a leader**, the two runs in
  §7, zero here. Every other range ends on its last field. A walker that
  stops two bytes short of its record (`QueueItem`, `WData`, `City`'s
  +0x72) leaves the padding out.
- **Locals.** A walker often folds a value it computes rather than a
  field: an array's count and size, its flags masked `& 0xbf`, a
  `must_walk` answer, an order's type, a leader's decrypted resources, a
  cell's collision-block presence. They are on the stack, so the stream
  marks them `stack`. Their bytes are the original's, and a port must
  compute the same value.

## 10. How it was established, and what it has not established

**Diff-backed** (the instrument against the printed record, and the
reader's tests): the fold, the two Adlers, the fifteen lines and their
order, which bytes each walk hands over on this frame (the stream sums to
the printed value on all fifteen lines), and the packet ↔ end-record
pairing. Also the begin-record ↔ `frame_seeds` agreement, and the
reader's hook placement on run1100.

**From the type records and the PDB, held against the stream's bytes:**
the field names of §6–§8, the pointer and float census of §9, and the
order records' base fields.

**On a reading alone:** the hooks' order in `Game::do_frame@00591ef0`
(begin, tick, end), and what each `must_walk` local and each order-type
local means. Also the walls and ammo walkers' record shapes (no element
on either frame read), and `say_checksum`'s labels beyond the fifteen.

**Not established:**

- ~~Whether the minted file's playback prints the windowed record at all,
  and whether its record is the 67-line one. No `DUMP_ALL` playback was
  taken; this item's frame came from run1100, which was on disk.
  Run 1562, reserved for that, was not taken.~~ It does, and it is:
  run 1562 (item 4088, `docs/RUNS.md`) prints records 3201 and 3202,
  `CHECKSUM n`, `FILE`, `LINE` and 64 labelled lines each, and the
  packet reproduces the first (§3).
- ~~Any frame but 5408 of run1100.~~ Two frames now, run1100's 5408 and
  run 1562's 91 (§4); the order of the twelve is re-measured on the
  second. Deaths, walls and ammo are empty on both: their record shapes
  stay a reading, and their lines are not counted until a frame holds
  one.
- Whether the seven floats' bits follow from integers the simulation
  holds.
- Why the rules line differs between run1100 and run1420 (314192132
  against 1227894240). The recon read it as static, and it is static
  across run1100's two records. Run 1562, another lobby (the harness
  lobby's two seats, tribes 4 and 11), prints run1100's 314192132 again. Whether it varies with the lobby (tribes
  and their bonuses) is open.
- Atomicity: the packet is not atomic across threads. It reproduces
  every line here, which argues that nothing the walks read moved during
  the copy.
- Whether a field the simulation does not hold stands in any line. That
  is each later item's finding: for example the world's fog bytes
  (`seen`, `seen2`, `seen3`, `wcoord_seen`, `was_seen`), the danger maps,
  and a unit's collision and path-finding state.

## 11. The items line, ported (item 4088)

`sim::checksum` (`crates/sim/src/checksum.rs`, traced as
`ChecksumSync`) emits a line's byte image from this crate's state:
ranges in the original's order and at its widths. `Sim::checksum_items`
is the first. `rondata::diff::checksum_lines` walks the minted game's
built start under the pump to frame 91 (`docs/RECGAME.md` §9) and holds
the image against the instrument's stream of run 1562's packet, range by
range, then reads the Adler against record 3201's `Items`.

**The record, per live goody box** (`CheckSums::check_items@00937790`,
the items array's order, `flags & 1` and the `Item` vtable), five ranges
and 22 bytes, in walk order — the stream settles it: `must_walk`'s byte
comes third, before the range (§8 listed it last until this item).

| # | range | ours | on run 1562's frame |
|---|---|---|---|
| 1 | `ItemData::ever_seen` (+0x20), 1 | `GoodyItem::ever_seen` | 0 on all 22 |
| 2 | `SubObjectData::flags` (+0x8), 1 | 1: only a live box is walked, and no other bit is held | 1 on all 22 |
| 3 | `SubObject::must_walk@006623a0`'s byte, a local | 1 | 1 on all 22 |
| 4 | `who`..`y_internal` (+0x9..+0x18), 15 | 255; the box's index as `o`; `z`, `x`, `y`, each `^ 0x63637` | `o` 0..21 in order |
| 5 | `SubObject::walk_data@006621d0`'s 4-byte local (site 00662287) | `BASE_ITEMTYPES`, 0x21f | 543 on all 22 |

**The two locals.** `must_walk`'s byte is 1 when the walk is not an
input walk and the object has a `ptype` or `flags & 1`, so 1 for every
walked box. The 4-byte local is `ptype +4` or 0 without a `ptype`: the
type's `TypeIndex`. It reads 543 = `BASE_ITEMTYPES` on every item of
both packets. On the goods line it reads the good's own type (5 for
oil, 6 for fish, and 10, 15, 17 … for the rares on this frame), the
same field.

**`z` is `refresh_good_z`'s, not `SubObject::init`'s.** The first image
took the ground as `SubObject::init@00662300` writes it,
`find_tcoord_z` of the tile (`World::object_z`), and parted on 20 of 22
boxes, `z` alone: 180 against 156 on box 0, 31 against 28 on box 16,
96 against 116 on box 20; boxes 13 and 21 agreed.
`Terrain::refresh_good_z@00850b60` runs at the map's close
(`docs/MAPGEN.md`, the 894 → 933 span) and after a building's terraform
(`cities/city.rs`, `Wall::init`'s arm). It rewrites every
live good's and item's `z` to `TerrainOut::find_data_z@00866560` at the
object's point with the fourth argument 1 (`pushl $0x1` at `00850bfa`):
`World::data_z`, the cell's two-triangle interpolation, with 0 on an
ocean tile and below zero.
With that, all 22 agree. Every one of the 22 reads only corners known
to be the original's own singles (the test pins it).

**The value diff**: 110 of 110 ranges and 484 of 484 bytes equal,
record by record and field by field. Ours sums to 1548048405, the
printed `Items`. On box 0: `ever_seen` 0, `flags` 1, `must_walk` 1,
`who` 255, `o` 0, `z` 156, `x` 36480, `y` 22656, type 543, both sides.
The test prints all 22.

**Made to fail**: an empty image parts on all 110 ranges (red first,
before the port). One perturbed byte of ours, box 3's `x`, parts on
that range alone (range 18, record 3, `who..y_internal`), and the Adler
with it. The first image's `z` was the measured parting above.

**Goods does not ride along.** It is the same record over the goods
array, but its first byte is `GoodData::ever_seen` (+0x20), which
`world::Good` does not hold: one good of 36 reads 2 on this frame
(leader 1 has seen it), the rest 0. Holding it needs a reading of that
field's writers, which items did not, so it is the next goods item's.
~~Not read~~ — read by item 4091 (§13): the crate holds the field as
`territory.good_seen_bits`, and the one parting byte is a setup writer
our built start does not run.
The rest of the record — `flags` 1, `who` 255, `o`, `z` by the same
`refresh_good_z`, `x`, `y` and the type local — is what items already
emits.

**How it was established, and how confident.** Diff-backed: every
byte of the line on one frame, against the stream, and the Adler against
the printed record. On a reading, held by that diff on this frame: that
`refresh_good_z` is the last writer of an item's `z` (its callers were
read, not traced), that a box's flags hold no bit but the live one
(`SubObject::init` ORs in 0x20 when the type's vtable +100 answers
nonzero; the dump and the stream print 1),
and that a box's index is its `o` (the built start lays the boxes in
the original's order, and the stream agrees here).

**Not established**: any frame but 91; an `ever_seen` other than 0 on
the items line (every box reads 0 here, so that byte agrees by default
on this frame: a later frame, after the scout has seen a box, is the
check); a box taken (`flags` 0, not walked) or one whose ground a later
terraform moved, which `refresh_good_z` follows and this crate's boxes
read live; a box on an ocean tile or below zero.

## 12. The cities line, ported (item 4091)

`Sim::checksum_cities` (`crates/sim/src/checksum.rs`) emits
`CheckSums::check_cities@00937600`'s image from this crate's state, and
`rondata::diff::checksum_lines::the_cities_line_is_the_original_s_range_by_range`
holds it against the instrument's stream of run 1562's packet the way
§11 holds items: on the minted walk's frame 91, range by range, then
every field decoded on both sides, then the Adler against record 3201's
`Cities`.

**The line, per active leader in slot order, per live city in that
leader's list** — 3 ranges a city with no link, 114 bytes:

| # | range | ours |
|---|---|---|
| 1 | `CityData::city_flags` (+0x4), 2 | the word from the city's own bits and its building marks (below) |
| 2 | `city`..`ter` (+0x6..+0x72), 108 | the table below |
| 3 | `vans.length`, a 4-byte local | the caravan links ending at the city; with links, §8's four more ranges |

`city_flags` is built bit by bit (`docs/CITIES.md` §1.4): `0x1` alive,
`0x2`/`0x4`/`0x8` the attack stages, `0x10` capital, `0x40` alarm,
`0x100` unassimilated, `0x2000` no muster, `0x4000` founding capital,
`0x8000` was founding capital, all on the city record; `0x80`, `0x200`,
`0x400` and `0x800`, an active Temple, Granary, Lumbermill and Market
in the city, read off its building chain (`Sim::city_has`); `0x1000`,
the owner's `CityAi::near_border`. **The human's city reads `0x4811`
and the British `0x4011`**: the `0x800` is the human's Market, which
ours holds in the city (it is the bit run1031's `CITIES=5` row
recorded, `docs/RUNS.md`). Bit `0x20` is not held; no frame read sets
it.

**The 108 bytes, as the type record names them** (`types.txt`,
`CityData`, size 0xb8; `sim::checksum::CITY_FIELDS`, which a test holds
to tile +0x6..+0x72 exactly). "Holds" is a field of this crate's;
"derives" is computed from state it holds; "zero" is a field no writer
here keeps, written as the constant `City::init@00737050` leaves.

| field | offset, width | ours | city 0 (who 0) | city 1 (who 1) |
|---|---|---|---|---|
| `city` | +0x6, 2 | derives: the city's place in its owner's list | 0 | 0 |
| `o` | +0x8, 2 | holds: the city building's `index` | 2000 | 2000 |
| `reg` | +0xa, 2 | derives: `City::reg` in the original's numbering (`World::region_number`) | 1 | 1 |
| `x`, `y` | +0xc, +0x10, 4 each | holds: `pos`, **plain** (`City::init` unmasks the building's `Coord`s) | 3168, 30816 | 42336, 16224 |
| `attack_stamp`, `raid_stamp`, `reduce_stamp`, `capture_stamp`, `assimilation_timer` | +0x14..+0x28, 4 each | holds | 0 each | 0 each |
| `capture_strength` | +0x28, 4 | holds | 0 | 0 |
| `traded_with[8]` | +0x2c, 4 each | holds | 0 each | 0 each |
| `scouted` | +0x4c, 2 | zero (`City::init`, copied by `City::capture@00736c40`) | 0 | 0 |
| `in_port`, `peasant_dist` | +0x4e, +0x50, 2 each | holds (`CityAi`, the owner's copy) | 0, 1 | 0, 1 |
| `trade_val` | +0x52, 2 | holds | 0 | 0 |
| `conquest_node` | +0x54, 2 | zero: −1, `City::init`'s `0xffff` (Conquer the World, cut) | −1 | −1 |
| `granary`, `lumber_mill`, `smelter`, `refinery` | +0x56..+0x5a, 1 each | derives: `City::calc_gather@00737c60`'s four bytes (`Sim::city_holdings`), the fourth an unconditional 0 | 0 each | 0 each |
| `free`, `busy`, `gatherers` | +0x5a..+0x5d, 1 each | holds (`CityAi`) | 0, 5, 5 | 0, 5, 4 |
| `pop` | +0x5d, 1 | holds | 1 | 1 |
| `who`, `race`, `founder` | +0x5e..+0x61, 1 each (`char`) | holds (`race` −1 when `None`) | 0, 0, 0 | 1, 1, 1 |
| `plundered` | +0x61, 1 | zero (`City::init`; `Build::process@0061edf0` counts it down every 200 frames; its setter not read) | 0 | 0 |
| `ocean`, `land`, `filled`, `bordering`, `ocean_filled`, `dock_tile` | +0x62..+0x68, 1 each | holds (`CityAi`) | 0, 77, 38, 0, 0, 0 | 0, 82, 40, 0, 0, 0 |
| `was_capital_flags` | +0x68, 1 | holds | 0 | 0 |
| `space[3]` | +0x69, 1 each | holds (`CityAi`) | 45, 45, 39 | 53, 53, 42 |
| `ter[6]` | +0x6c, 1 each | holds (`CityAi`) | 1, 2, 0, 1, 2, 0 | 1, 2, 0, 1, 2, 0 |

Every value in the last two columns is the same on both sides.

**The value diff**: 6 of 6 ranges and 228 of 228 bytes equal; every
field of both cities equal (`city_flags` and the 52 fields above, both
sides decoded by the same table); ours sums to 1233128295, the printed
`Cities`. The original prints the same values in frame 91's `CITY`
records (`CityData::log_data@004895c0`), which `diff::harness` already
compares on every capture that dumps them, which is why the AI's half
agreed on the first image.

**Made to fail**: the empty image parted on all 6 ranges (red first,
before the port). One perturbed byte of ours, the second city's `land`,
parts on that range alone (range 4, `city..ter`), and the Adler with
it.

**How it was established, and how confident.** Diff-backed: every byte
of the line on one frame, against the stream, and the Adler against the
printed record; the walk's order (each city's `vans` after its own two
ranges) is the stream's. From the type record: every field name,
offset and width. On a reading, held by that diff on this frame: what
`City::init` writes to `scouted`, `conquest_node` and `plundered`, and
the shape of a non-empty `vans` array (run1100's stream, not ours).

**Not established**:

- Any frame but 91, and any lobby but the minted game's.
- **Agreeing by default on this frame** (the same constant on both
  cities, so the diff cannot tell a held field from a written zero):
  every stamp and `capture_strength`; `traded_with[8]`; `scouted`,
  `in_port`, `trade_val`; `conquest_node` (−1); the four enhancer
  bytes; `free`; `plundered`; `ocean`, `bordering`, `ocean_filled`,
  `dock_tile`, `was_capital_flags`; `ter[2]` and `ter[5]`; and city
  flags `0x2`, `0x4`, `0x8`, `0x20`, `0x40`, `0x80`, `0x100`, `0x200`,
  `0x400`, `0x1000`, `0x2000`, `0x8000`. `scouted` and `plundered` have
  no writer here; a capture that sets one is the check.
- **A city with a caravan link.** The image emits §8's shape with `size`
  10, `cara` the caravan's slot in its owner's list and `who` that
  owner, in owner-then-slot order. Each of those is an assumption: no
  link stands on the scored frame, and the array's growth and add order
  are not read.
- A leader whose list holds a dead slot before a live one, or a captured
  city. `city` is the city's place among its owner's live cities, and
  the original's is its slot in the leader's `PtrArray<City>`. The two
  agree until a slot dies.
- Which leaders are "active": the walk takes every owner of a live city
  in slot order, and the original takes `leaders.list & 1`.

## 13. The goods line, held out on one byte (item 4091)

`Sim::checksum_goods` is §11's record over the goods array, per live good
in `World::goods`' order (`o` its index): `ever_seen`, `flags` 1,
`must_walk` 1, `who` 255, `o`, `z` as `refresh_good_z` last wrote it,
`x`, `y`, and the type local, the good's own `TypeIndex`.
`rondata::diff::checksum_lines::the_goods_line_is_held_out_on_good_15_s_ever_seen`
holds it against the stream.

**`GoodData::ever_seen` (+0x20), and its writers.** The type record names
it a `uchar`, a bit per leader. `Good::log_data@0066e610` prints it, and
its one reader is `GoodData::is_seen@0066e4c0` (`ever_seen & ally_mask`
first). Every writer in the export:

1. `Good::init@0066da20`: zero.
2. `World::reveal_fog@006b3d30:87`, the rare arm: `|= 1 << who` right
   after `Leader::new_rare@006d9e70`, on a fog cell `World::set_seen`
   newly revealed whose tile carries a good's footprint (`0x200`).
3. `World::compute_reg_territory@006b0bb0:651`, the per-cell goods
   scan: `|= 1 << who` beside `new_rare`, on a cell the pass gives
   `who`, where the cell's first live good is not oil and `who` or an
   ally has seen its second half-cell each way.
4. `Leader::plan_strategy@006b9620:337`, the census's step 9: `|= 1 <<
   who` on every good of the leader's own `new_rares` that it has
   explored. `new_rare` hands a good to every ally as well as the finder
   (`diplos` 2 both ways), so this arm adds a bit only in an alliance.
   Outside one, the leader's `new_rares` holds only goods arms 2 and 3
   put there, and each set the same bit.
5. `Animal::work@005d7330:53`, the herd arm: every 16 frames, phased by
   the animal's `o`, an animal of a herd whose `HerdData::good_o` names
   a good sets the bit of every live leader that `WorldData::is_seen`
   the animal's fog cell. On Great Lakes the herds name goods 19 and
   24..35.

This crate holds the field as `territory.good_seen_bits`, written by
`Sim::mark_good_seen` from arms 2 and 3 (items 1555 and 3332).
`Sim::good_ever_seen_byte` reads it for the image. Arms 4 and 5 are not
modelled.

**The value diff, and the byte held out**: 179 of 180 ranges equal, and
791 of 792 bytes. Every good's `flags`, `must_walk`, `who`, `o`, `z`,
`x`, `y` and type agree, and every `ever_seen` agrees but one:

| good | type | point | `ever_seen` ours | theirs |
|---|---|---|---|---|
| 15 | 17 (the SILK) | (40320, 14208) | 0 | **2** (leader 1) |

Ours sums to 1878881124 against the printed 1939436390. With that one
byte at the original's 2, a substitution made only in the test, ours
sums to 1939436390. So the rest of the line is the original's.

**Why ours lacks it.** The good stands on a cell leader 1 owns (cell
(52, 18)), and leader 1's `seen2` holds that cell's half-cell (fog
(105, 37) reads 2). The cell is seen through the starting city's
`Object::update_seen_ally@006537a0`, which writes `seen2` directly and
never calls `reveal_fog`, so arm 2 does not fire. The original's
setup `compute_all_territory` then runs arm 3 on the cell: `new_rare`,
then the bit. Its frame-91 `LEADERDATA` prints leader 1's `known_rares 1`,
the census's count of that list. Our built start's wholesale setup pass
(`start_border_pass` before play, and `settle_borders`) runs only the oil
half of the scan (`claim_oil_from_owners`), so on frame 91 ours holds
`new_rares[1]` empty, `known_rares` 0 and no bit. Our first in-play
border pass gives the same cell to leader 1 through `claim_cell_goods`
on **frame 387**, and from then on ours reads 2 (probed on the walk:
`new_rares[1]` `[15]` from 387, `[15, 17]` by 2000).

So the byte is not a field the simulation lacks. It is a writer the
built start does not run: setup's goods scan, and with it the AI's
`new_rares` and `known_rares` for the first 296 frames. Porting it is
not this line's to do. It changes the AI's state on the minted walk
(its merchant reads `new_rares`), which is a mechanic under its own
document (`docs/ECONOMY.md`, "The rares a leader has seen"), with every
floor run against it. **The goods line is held out on good 15's
`ever_seen`**, and the test pins it so, both ways: a setup pass that
sets the bit turns it red, and so does any other byte that parts.

**Made to fail**: the empty image parts on all 180 ranges. One more
perturbed byte of ours, good 3's `x`, parts on its own range as well as
good 15's, and on no other. The held-out byte is itself the measured
parting.

**The items line is not touched by this reading.** An item's
`ItemData::ever_seen` (+0x20) has its own writers: `Item::init` and
`reveal_fog`'s goody arm (`:148`), which `GoodyItem::ever_seen` carries
(item 3335). None of the five above writes an item. All 22 boxes read 0
on both sides, and items stays 110 of 110.

**How it was established, and how confident.** Diff-backed: 791 bytes
of the line on one frame, and the one byte that parts with both sides'
values. On a reading: the five writers (found by grepping the export
for every `|= 1 << who` and every store at +0x20 of a goods-list
pointer, then read with their callers), and arm 4's redundancy outside
an alliance. Probed on our walk: the frame ours sets the bit.

**Not established**:

- Any frame but 91. After frame 387 ours holds good 15's bit. Whether
  the line then agrees depends on arm 5: a leader that sees a herd
  animal sets its good's bit in the original and not in ours. On frame
  91 every herd good (19, 24..35) reads 0 on both sides, so those
  goods agree by default here.
- Arm 4 in an alliance (no allied lobby is read).
- The order of `new_rares` when the setup scan is ported. Arm 3 appends
  in the pass's region-and-cell order (`docs/ECONOMY.md`).
- A dead good (`flags` 0, not walked), and a good a terraform moved.
