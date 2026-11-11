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
(§12), and so does the third, goods (§13, item 4093), once the built
start runs setup's goods scan. ~~Goods is held out on one byte, good 15's
`ever_seen` (§13).~~ Builds (§15), guys (§16) and units (§18, item 4113)
agree byte for byte too. What
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
IEEE-754 bit patterns. That is the residue's question, and §10 states it;
~~for the five figure floats~~ they are `+0.0` on every figure of both
frames and constant on this game's figures, emitted as bits (§16).

**Cheapest first, on run 1562's frame 91** (item 4088), the order the
items after 4088 follow. The three empty lines are not counted there
and are named last; by bytes:

1. cities (228; ported, §12) · 2. items (484; ported, §11) · 3. goods
(792; ported, §13) ·
4. builds (4,923; ported, §14, §15) · 5. guys (9,308; ported, §16) ·
6. units (19,642; it walks the
guys again; ~~held out on three fields, §17~~ ported, §17, §18) · 7. groups (37,010; ported, §19) · 8. leaders (58,343) ·
9. world (294,769) · then walls, ammo and deaths (0 each), counted on a
frame that holds one.

Cities and items swapped places against run1100's frame (630 and 352
there). Goods is the record items is, over another array; ~~its one
parting byte is a setup writer our built start does not run~~ its one
parting byte was a setup writer our built start did not run, and runs
since item 4093 (§13).

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

    Checksum: instrument 15 of 15 lines (run1562 f91) · state 6 of 12 lines, 35,377 of 425,499 bytes

Item 4088 moved it from `instrument 15 of 15 lines (run1100 f5408) ·
state 0 of 12 lines, 0 of 501,543 bytes`: the pair moved to the minted
game, and items is ported (§11). Item 4091 moved it from `state 1 of 12
lines, 484 of 425,499 bytes`: cities is ported (§12), and goods is held
out on one byte (§13). Item 4093 moved it from `state 2 of 12 lines, 712
of 425,499 bytes`: goods is ported (§13). Item 4094 left it at `state 3
of 12 lines, 1,504 of 425,499 bytes`: builds is held out on three fields
of the two camps, 516 of its 519 ranges and 4,845 of its 4,923 bytes
agreeing, which `CHECKSUM_STATE` does not count (§14). Item 4097 left it
there too: builds is held out on one `up_who`, 518 of 519 ranges and
4,889 bytes agreeing. Item 4098 moved it from `state 3 of 12 lines,
1,504 of 425,499 bytes`: builds is ported, every byte (§15). Item 4104
moved it from `state 4 of 12 lines, 6,427 of 425,499 bytes`: guys is
ported, every byte (§16). Item 4109 left it at `state 5 of 12 lines,
15,735 of 425,499 bytes`: units is held out on three fields of six units,
1,078 of its 1,084 ranges and 18,976 of its 19,642 bytes agreeing, which
`CHECKSUM_STATE` does not count (§17). Item 4113 moved it from `state 5
of 12 lines, 15,735 of 425,499 bytes`: units is ported, every byte (§18).

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
| | 1-byte local | ~~`Object::must_walk@00647930`~~ `launching != 0` (site 00647910; corrected by item 4109, §17) | 101, 101 |
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
| `TargetOrder::walk_data` (a `BUILD_AT`; item 4109, §17) | 1 byte, [+0x8, +0x12) | `UnitOrder` `flags`; `TargetOrder` `ox`, `whom`, `uid` | 0 here; 2, 2 + 20 on run 1562 |
| | 1 byte | `UnitOrder` `flags` (+0x54) | 8, 8 |
| `TradeOrder::walk_data@00483260` | [+0x8, +0x12), [+0x14, +0x26) | `TargetOrder` `ox`, `whom`, `uid`; `oxx`, `whose`, `started`, `loaded`, `uid2` | 2, 20 + 36 |
| | 1 byte | `UnitOrder` `flags` (+0x2c) | 2, 2 |
| `PtrArray<Guy>::walk_data@0046df30` (+0xe4) | locals and header | count (4), size (4), [+0xc, +0xe) of the array, flags masked `& 0xbf` (1), then after the per-guy locals [+0x8, +0xe) (the stream's order, §16) | 101, 1,717 |
| | per guy: 1-byte local | (site 0046e199) | 111, 111 |
| `GuyData::walk_data@005e0210` | [+0x8, +0xa3) | `GuyData` `type`..`guy_num`, 42 fields, **5 floats** (§4) | 111, 17,205 |
| `AnimalData::walk_data@005d7ea0` | 1-byte local; [+0x150, +0x155) | `AnimalData` `ox`, `whom`, `aid` (55 of the 101 are animals) | 55, 55 + 275 |

*Corrected by the stream* (item 4109, run 1562's frame 91; §17 has the
order): the two `Unit::must_walk` bytes are the vcalls of `SubObject`'s and
`Object`'s walkers — the first after `flags`, the second before the
`ObjectData` range — and an order's 1-byte `flags` comes **before** its
fields, the type local and node byte before both.

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

*Corrected by the stream* (item 4094, run 1562's frame 91; §14 has the
table): `founder` and `max_age` come **first**, before the sub-object;
the "2-byte `must_walk` pair" is **four** one-byte `must_walk` locals, at
`SubObject`'s and `Object`'s vcalls and inlined by `WallData` and
`BuildData` at their own sites; `orig_type` comes **last**, after the
gather points' count; the queue walks **`queue_size`** slots (the array's
allocation, 2, 10 or 20), not the live entries; and the tile array hands
over its `length` (4), and when it is not 0 its `size` (4, a local),
`increment` (2) and `flags & 0xbf` (1, a local) before the tiles.

**Groups** (`CheckSums::check_groups@00937530`): as many slots of the
groups array as its count at +0x4 says (512 here), `Group::walk_data@00708400` [+0x4, +0x4c) — `GroupData`
`id`..`march`, 21 fields, 36,864 bytes. For each slot whose `num` is not
0 (6 here, each `num` 1): `num` entries of ~~`off_x`, `off_y`, `curr_x`,
`curr_y` (4 bytes each), `list` (2) and `angles` (1)~~ `list` (2), then
`off_x`, `off_y`, `curr_x`, `curr_y` (4 bytes each) and `angles` (1), one
range an array, in that order (the stream, item 4116; §19). Then the eight
ints of the array the groups object points to at +0x3c — ~~unnamed~~
`GroupsData::last_group`, the pointer being `&groups + 0x1c` (§19) —
folded by zlib's Adler from a stack copy (32 bytes; §1).

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
  can be derived from integers the port does hold. ~~That is open
  (§10).~~ For the five of a figure it is answered (§16): `+0.0` on every
  figure of both frames, emitted as bits; the leaders' two stay open
  (§10).
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
- ~~Whether the seven floats' bits follow from integers the simulation
  holds.~~ The five of a figure: one bit pattern, `+0.0`, on every
  figure of both frames, constant on this game's figures, and on an
  aircraft's guy 0 the singles `air::Airframe` holds (§16). The leaders'
  `anti_att` and `plunder_scale` are still open.
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
  and a unit's collision and path-finding state (~~a unit's~~ *for units*,
  item 4109, §17: no walked range holds the path-finding trees, which
  start at `+0x100`; two fields were not held and are carried now, `uid`
  and the birth cell `los_x`/`los_y`; and three ~~stand unheld, holding the
  line out~~ stood unheld, holding the line out until item 4113 (§18) —
  `play`, the `+0x86` union of a special or a woodcutter, and
  `unit_masks & 0x8`). *For builds* (item 4094,
  §14): three fields were not held and are carried now (`uid`,
  `frame_started`, `build_masks & 0x400`); `WallData::gpiece` is art the
  data layer hands in; and two stood unheld, holding the line out — ~~a
  camp's `recharging`, short of `Guy::move`'s bump~~ (held since item
  4097, `docs/ORDERS.md` §6.11), and ~~one `up_who`, the object chain's
  history of a building~~ (held since item 4098 with the chain's other
  three words, `docs/COLLISION.md` §3.2; §15). *For groups* (item 4116,
  §19): one field was not held and is carried now, a group's `role`
  (`docs/GROUPS.md` §3.4).

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

## 13. The goods line, ported (item 4093; held out on one byte at item 4091)

`Sim::checksum_goods` is §11's record over the goods array, per live good
in `World::goods`' order (`o` its index): `ever_seen`, `flags` 1,
`must_walk` 1, `who` 255, `o`, `z` as `refresh_good_z` last wrote it,
`x`, `y`, and the type local, the good's own `TypeIndex`.
`rondata::diff::checksum_lines::the_goods_line_is_the_original_s_range_by_range`
holds it against the stream (item 4091's
`the_goods_line_is_held_out_on_good_15_s_ever_seen` turned into it).

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
`Sim::mark_good_seen` from arms 2 and 3 (items 1555 and 3332), and from
arm 3 at setup since item 4093 (`Sim::claim_rares_from_owners`).
`Sim::good_ever_seen_byte` reads it for the image. Arms 4 and 5 are not
modelled.

**The value diff, ported (item 4093)**: 180 of 180 ranges equal, and
792 of 792 bytes. Every good's nine fields agree. One good of 36 has a
bit on the frame, on both sides:

| good | type | point | `ever_seen` ours | theirs |
|---|---|---|---|---|
| 15 | 17 (`AMBER`) | (40320, 14208) | 2 (leader 1) | 2 (leader 1) |

Ours sums to **1939436390**, the printed `Goods`. Made to fail: good 15's
byte at the unported 0 parts on range 75 alone, and the Adler with it;
good 3's `x` perturbed parts on range 18 alone; the empty image parts on
all 180. The frame-0 line beside it: run9's `[Start Game]` prints 35 of
the 36 goods' `ever_seen` (the first record lost its head lines in the
print), and the built start's agree on all 35, good 15 at 2
(`diff::start_walk::the_built_start_s_goods_ever_seen_is_the_start_dump_s`).
Leader 1's `new_rares` is `[15]` from frame 0 (ours; no dump prints the
list), and on frame 91 its `known_rares` is 1 on both sides.

**The value diff at item 4091, and the byte held out**: 179 of 180 ranges equal, and
791 of 792 bytes. Every good's `flags`, `must_walk`, `who`, `o`, `z`,
`x`, `y` and type agree, and every `ever_seen` agrees but one:

| good | type | point | `ever_seen` ours | theirs |
|---|---|---|---|---|
| 15 | 17 (~~the SILK~~ `AMBER`: `TypeIndex` 17, and run9's `GOOD` prints `Amber`) | (40320, 14208) | 0 | **2** (leader 1) |

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
`new_rares` and `known_rares` for the first 296 frames. ~~Porting it is
not this line's to do. It changes the AI's state on the minted walk
(its merchant reads `new_rares`), which is a mechanic under its own
document (`docs/ECONOMY.md`, "The rares a leader has seen"), with every
floor run against it. **The goods line is held out on good 15's
`ever_seen`**, and the test pins it so, both ways: a setup pass that
sets the bit turns it red, and so does any other byte that parts.~~
Ported by item 4093 under that document ("Setup's goods scan"), with
every floor that walks a built start run against it and none moved
(the value diff above).

~~**Made to fail**: the empty image parts on all 180 ranges. One more
perturbed byte of ours, good 3's `x`, parts on its own range as well as
good 15's, and on no other. The held-out byte is itself the measured
parting.~~ The line's own made-to-fail is in the value diff above.

**The items line is not touched by this reading.** An item's
`ItemData::ever_seen` (+0x20) has its own writers: `Item::init` and
`reveal_fog`'s goody arm (`:148`), which `GoodyItem::ever_seen` carries
(item 3335). None of the five above writes an item. All 22 boxes read 0
on both sides, and items stays 110 of 110.

**How it was established, and how confident.** Diff-backed: all 792
bytes of the line on frame 91 (item 4093; 791 and the one parting byte
at item 4091), and every printed good's `ever_seen` at frame 0. On a reading: the five writers (found by grepping the export
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
- The order of `new_rares` ~~when the setup scan is ported~~ that setup
  leaves. The scan is ported (item 4093) and appends in the pass's
  region-and-cell order (`docs/ECONOMY.md`, "Setup's goods scan"), but
  on this lobby each leader meets one rare at most, so no diff has
  tested the order.
- A dead good (`flags` 0, not walked), and a good a terraform moved.

## 14. The builds line, ported (items 4094, 4097; whole since item 4098, §15)

`Sim::checksum_builds` (`crates/sim/src/checksum/builds.rs`) emits
`CheckSums::check_builds@00937290`'s image from this crate's state, and
`rondata::diff::checksum_lines::the_builds_line_is_the_original_s_range_by_range`
(until item 4098 `the_builds_line_is_held_out_on_one_up_who`) holds it against the instrument's stream of run 1562's packet the way §11
to §13 hold theirs: on the minted walk's frame 91, range by range, every
range decoded field by field on both sides (ours by our range names,
theirs by each row's call site), then the Adler against record 3201's
`Builds`. ~~**The line is held out on three fields of the two
Woodcutter's Camps, then on one field, the British camp's `up_who`**~~ **The
line is whole since item 4098** (§15): the camps' `recharging` agree
since item 4097 (below), and the camp's `up_who` since the chain of
objects is held (`docs/COLLISION.md` §3.2).

**Which buildings, in which order.** Per leader whose `leaders.list & 1`
is set, in slot order; per slot of that leader's object array from
`obj_base[1]` (2000) to its count (`objects +0x1ec`), the object behind
vtable `+0xac` whose `flags & 1` is set, walked through its vtable `+0x7c`
(`BuildData::walk_data@0062f270`). So: per owner, every live building in
`o` order. On frame 91 that is the human's eight (`0/2000..0/2007`) and
the British seven (`1/2000..1/2006`).

**One building's ranges, in walk order** — the stream settles it (the
record names come from `types.txt`; what each local is, from the
walkers):

| # | site | range | bytes | what |
|---|---|---|---|---|
| 1 | `0062f28c` | `BuildData::founder` (+0x7f) | 1 | |
| 2 | `0062f2a0` | `BuildData::max_age` (+0x83) | 1 | |
| 3 | `006621fe` | `SubObjectData::flags` (+0x8) | 1 | `SubObject::walk_data@006621d0` |
| 4 | `00647965` | a local | 1 | `Object::must_walk@00647930`'s byte, through `SubObject`'s vcall `+0x78` |
| 5 | `0066221c` | `who`..`y_internal` (+0x9..+0x18) | 15 | `z`, `x`, `y` under the `Coord` mask |
| 6 | `00662287` | a local | 4 | the type's `TypeIndex` (`ptype +4`), the type it is **now** |
| 7 | `00647965` | a local | 1 | `must_walk` again, `Object::walk_data@00647830`'s vcall |
| 8 | `00647885` | `ObjectData` `myhits`..`launch_frames` (+0x20..+0x42) | 34 | 19 fields (`OBJECT_FIELDS`) |
| 9 | `00647910` | a local | 1 | `launching != 0`; a non-null `launching` walks its `SimpleArray<int>` after it |
| 10 | `00642570` | a local | 1 | `WallData::walk_data@00642510`'s inlined `must_walk` |
| 11 | `00642586` | `WallData` `job_counter`..`demolition` (+0x48..+0x66) | 30 | 11 fields (`WALL_FIELDS`) |
| 12 | `0062f2fc` | a local | 1 | `BuildData::walk_data`'s inlined `must_walk` |
| 13 | `0062f316` | `BuildData` `gather_down`..`infiltrate2` (+0x70..+0x86) | 22 | 15 fields (`BUILD_FIELDS`); +0x78 the union |
| 14 | `0063061b` | `BuildQueueData::queue_size` (+0x88) | 4 | `BuildQueue::walk_data@006305f0` |
| 15 | `00630653` | each of `queue_size` `QueueItem`s, 18 of 20 bytes | 18 each | `job_counter`, `type`, `good[3]`, `cost[3]` |
| 16 | `0062f336` | `MiningList::mtn`, `cliff` (+0xb4) | 2 | |
| 17 | `00471c66` | a local | 4 | the mining list's `length`, `Array<TCoordData>::walk_data@00471c30` |
| 17a | `00471cc7` | a local | 4 | its `size`, when `length` is not 0 |
| 17b | `00471cd5` | `ArrayBaseMaster::increment` (+0xc of the array) | 2 | when `length` is not 0 |
| 17c | `00471ced` | a local | 1 | its `flags & 0xbf`, when `length` is not 0 |
| 17d | `00471df3` | each `TCoordData` (`tx`, `ty`) | 8 each | when `length` is not 0 |
| 18 | `004708c1` | a local | 4 | the gather points' count (`PtrLinkListAbstract<GatherPoint,unsigned_char>::walk_data`); with points, a 4-byte 0, the node's byte and 9 of each `GatherPoint` |
| 19 | `0062f35a` | `BuildData::orig_type` (+0x6c) | 4 | |

Eighteen ranges a building and one per queue slot, and four more and one
per tile for a building with a mining list: 15 × 18 + 100 + (4 + 82) +
(4 + 61) = 519 ranges, 4,923 bytes, the stream's count. The four
`must_walk` bytes are each 1 on a live building (`flags & 1`). **The
queue walks `queue_size` slots, not `queued`**: the array's allocation
(2, 10 or 20, the only sizes `BuildQueue::init@006307d0` accepts). The two
two-row sites of the tile array (17a, 17b, 17c) are the two camps', the
only buildings with a list.

**What ours emits for each field.** "Holds" is a field of this crate's;
"derives" is computed from state it holds; "carried" is new at this item;
"init's" is the value the original's initialiser writes and nothing here
changes.

- **Sub-object**: `flags` derives — `0x1` alive, `0x2` started
  (`Wall::start`), `0x4` active, `0x20` a city's type (`SubObject::init`
  ORs it in when vtable `+100`, `BuildTypeData::is_city`, answers),
  `0x80` attacking. The human's city reads 39, a finished building 7, the
  British farm under way 3, the human's just-placed farm 1. `who`, `o`,
  `z`, `x`, `y` hold. The type local derives: `0x19e` plus the build
  type's index.
- **`ObjectData`**: `myhits`, `damage`, `hold_frames`, `healing`,
  `infiltrated`, `damage_frac`, `mylos`, `targeted`, `visible`,
  `launch_frames` hold; `near_o`/`near_who` derive from `near`;
  `inside_down`/`inside_down_who` derive from the garrison (the last
  captain to enter, and the owner when empty — `Object::init@00647750`
  writes `who` there). **`uid` is carried** (below). `up`, `down`,
  `down_who` and `up_who` are the world cell's object chain
  (`Object::add_to_world@0064d8c0`), ~~which this crate threads with units
  only: `up` derives as the bottom unit of the building's cell, `down` −1,
  `down_who` 0, `up_who` that unit's owner or −1. One of them parts
  (below)~~ **held** since item 4098, buildings threaded and the stale
  words kept (`docs/COLLISION.md` §3.2; §15).
- **`WallData`**: `job_counter`, `job_counter_2`, `constr_time`,
  `construct_hits`, `ever_seen`, `ever_seen_completed`, `helpers` hold.
  `gpiece` is the renderer's piece (`GraphicPieces::get_build_gpiece
  @0090bce0`), which the simulation does not load: the data layer hands it
  in (`rondata::free::placed_buildings`, the walk the renderer's K of M
  already holds to every dumped `gpiece`). **`frame_started` is
  carried**. `build_masks` derives from the bits this crate keeps (`0x10`,
  `0x20`, `0x40`, `0x88`, `0x100`, `0x800`, `0x1000`, `0x4000`), and
  **`0x400` is carried**. `demolition` is init's 0 (`Wall::init@0063e9b0`
  zeroes `+0x64`'s two bytes; no writer is read).
- **`BuildData`**: `gather_down` derives (the newest gatherer), `city` (the
  city's place in its owner's list) and `city_down` (the next building of
  the city's chain, `city_chain`) derive; `wonder` is init's −1
  (`Build::init@00629740`); the union at +0x78 derives (a farm's place in
  `Farms`' list, a dock's slot, else −1); `recharging` holds (~~and
  **parts**~~ — agrees since item 4097);
  `attack_ox`/`attack_whom` derive from `target`; `stance` derives
  (`Sim::build_stance`, `BuildTypeData::get_stance_type@006396c0`'s
  switch); `founder`, `gather_max` hold; `queued` is the live entries;
  `max_age` derives (the owner's ages: `Build::init` writes the owner's
  age, `Leader::gain_tech` every building's); `infiltrate`,
  `infiltrate2` are init's 0.
- **The queue**: each live entry from the queue (`type` the unit's
  `TypeIndex`, a technology's `0x220` plus its distance from the first
  age in the tree, `DISBAND` 0x29a); a slot past the live ones is
  `BuildQueue::init`'s: zero, slot 0's `type` −1.
  `BuildQueue::un_queue@00630ad0` shifts without clearing, so a slot a
  finished or cancelled entry vacated keeps that entry's bytes; this crate
  does not keep them (none had been vacated on frame 91: every slot past
  the live ones reads init's zeros on both sides).
- **The mining list**: the tiles hold (`gather_from`); `mtn`/`cliff` −1
  (no mine stands on the frame); `size` derives as the least 5·2ᵏ not
  below `length` — an empty list reports 5 on every record of the
  frame-91 dump, and `ArrayBaseSimpleCopy<WCoordData>::increase_size
  @004225c0` with `increment` −1 doubles; `MiningList::clear@00471160`
  keeps the allocation, so this assumes the list never held more than it
  does; `increment` −1, `flags` 0.

**Three fields carried into the simulation** (one writer each, already
ported, a few lines; nothing in the simulation reads them):

| field | writer | ours |
|---|---|---|
| `ObjectData::uid` (+0x30) | `Object::init`: the owner's counter at `Objects +0x1d4 + 2 who`, post-incremented by every object the player inits, unit or building | `Building::uid`, `Marks::uid`, `Sim::next_uid` from `add_unit`, `add_building` and `init_build` |
| `WallData::frame_started` (+0x5c) | `Wall::init` −1; `Wall::start@0063e810` the game frame, beside `flags \|= 2` | `Building::frame_started`, written in `Sim::wall_start` |
| `build_masks & 0x400` | `Wall::process@00640450`: set when `helpers` is nonzero, cleared when zero, `helpers` zeroed | `Building::had_helpers`, beside this crate's `helpers = 0` |

Before them the image parted on 18 ranges: `uid` on 13 buildings (ours 0;
theirs 1..6 and 13 for the human, 1..5 and 12 for the British), the
British farm's `frame_started` (−1 against 17) and `build_masks` (0
against 1024), and two technology entries (below).

**A technology's `TypeIndex` is not its tree id.** The British Library's
two entries read 550 and 564 on our first image, 551 and 565 on theirs:
the tree has no entry for the goody box's 0x21f, so a tech's id is one
below its `TypeIndex`. `docs/` says the tree index *is* the `TypeIndex`
(`ids.rs`); that holds for goods, units and buildings only.

**The ~~three fields~~ field the line is held out on** (the two
`recharging` rows were ranges 50 and 339, ours 83 and 67 against 90 and
89; struck at item 4097):

| range | building | field | ours | theirs | what writes it |
|---|---|---|---|---|---|
| 334 | `1/2001` | `ObjectData::up_who` (+0x3f) | −1 | 1 | `Object::add_to_world`; left behind by `Object::remove_from_world@00647970` |

*`recharging`* — **answered by item 4097, `docs/ORDERS.md` §6.11**: on
a gather building it counts the frames a gatherer worked it, the first
of `Unit::do_gather@005ef2a0`, `Unit::do_non_flat_gather@005f0170`,
`Wall::do_construct@006434d0` and `Guy::move`'s carrying walk to reach
it each frame adding one under `build_masks & 0x800`. The fourth is
ported; with it every building's `recharging` is run1561a's on all 299
of its frames, and the camps read 90 and 89 here. ~~`recharging` has
readers (`Wall::inc_time@0063fb60` compares it, and this crate's
`units/think/work_gate.rs`), so porting the arm changes behaviour~~: on
a camp it has no reader in the simulation — `Wall::inc_time`'s arm is
the anti-air types', and the work gate reads the *unit's* `+0xae`; its
reader is the renderer's work-animation clock (§6.11). ~~No dump
compares it~~: the dump **prints** it on every `BUILDDATA`, and
`gamelog::BuildDump` does not parse it (§6.11 has the hand-back).

*`up_who`.* ~~The British camp's cell has had a British unit above it on
its object chain, which moved off: `remove_from_world` sets the
building's `up` back to −1 and leaves `up_who`. This crate does not
thread buildings on the chain (`Sim::cell_chain` is units only), so it
holds no such history.~~ **Answered by item 4098** (§15,
`docs/COLLISION.md` §3.2): the reading held — a British unit stood above
the camp and stepped off, the head arm of `remove_from_world` wrote `up`
−1 and left `up_who` — and with buildings threaded at their init and the
four words held, ours reads 1.

**The value diff**: ~~516 of 519 ranges equal, and 4,845~~ **518 of
519 ranges equal, and 4,889** of 4,923 bytes in the ranges that agree
(item 4097); every field of all 15 buildings, their 100 queue slots and
143 tiles agrees but the one above. Ours sums to ~~3593622376~~
3506852741 against the printed **2869973639**; with the field at the
original's value (1, a substitution made only in the test) the image is
the stream's, range for range, and sums to 2869973639. Two
values on both sides, for a reader: the human's city `myhits` 1200,
`constr_time` 180000, `gpiece` 51339, `city_down` 2001, `queued` 1, its
queue slot 0 `(3100, 50, [0, −1, −1], [25, 0, 0])`; the British farm
`1/2006` `uid` 12, `job_counter` 7400, `construct_hits` 197,
`frame_started` 17, `build_masks` 1024, `up` 1, the union 6.

**Made to fail**: the image before the carried fields parted on 18 ranges
(above), and the empty image on all 519. One perturbed byte more, the
fourth building's `x`, parts on its range beside the held one, and on
no other.

**How it was established, and how confident.** Diff-backed: every byte
of the line on one frame but the one held out, against the stream, and
the whole line's Adler with it substituted; `recharging` on every
building of 299 frames, against run1561a's dump. From the type
records: every field name, offset and width; the union's members are the
dump's names. From the walkers: the order and the locals; the stream
agrees on all 519. On a reading, held by that diff on this frame: what
`Object::init`, `Wall::init`, `Build::init`, `BuildQueue::init` and
`Wall::start` write, the `0x400` latch, and the held-out field's
writer.

**Not established**:

- Any frame but 91, and any lobby but the minted game's.
- **Agreeing by default on this frame** (the same value on every
  building, or a field no writer here keeps): `damage`, `inside_down`
  and `inside_down_who` (no garrison stands), `down`/`down_who`,
  `hold_frames`, `near_o`/`near_who`, `healing`, `infiltrated`,
  `damage_frac`, `targeted`, `visible`, `launch_frames`, the `launching`
  local, `helpers`, `demolition`, `wonder`, `attack_ox`/`attack_whom`,
  `stance`, `max_age`, `infiltrate`, `infiltrate2`, `mtn`/`cliff`, the
  gather points (none set), and `build_masks` bits `0x10`, `0x20`,
  `0x40`, `0x80`, `0x8`, `0x100`, `0x800`, `0x4000`.
- `0x8` of `build_masks` is emitted with `0x80` (`Build::init` sets both
  for a hangar); only the player's toggle clears `0x80`, and whether it
  clears `0x8` is not read.
- ~~A building whose cell held a unit before it was placed (its `down`
  would be that unit), a building on a good's or a box's cell, and~~ (the
  chain is held, §15; still no capture) a garrisoned building (which end
  of the garrison `inside_down` names).
- A queue slot vacated by a finished or cancelled entry (the original
  keeps its bytes; ours writes zeros).
- A mining list that shrank or a slot reused by a larger list (`size`),
  a mine's `mtn`/`cliff`, a non-null `launching` (its array's ranges),
  and a building with gather points (the `GatherPoint`'s `x`/`y` are
  emitted plain; whether they are stored masked is not read).
- That every `Object::init` of the original is one of `add_unit`,
  `add_building` and `init_build` here: a wall, or a unit the original
  re-inits where this crate does not, would advance the original's
  counter alone.

## 15. The builds line, whole (item 4098)

**The line is the original's on run 1562's frame 91**: 519 of 519
ranges, 4,923 of 4,923 bytes, every field of the fifteen buildings, their
100 queue slots and 143 tiles, and the Adler **2869973639**, the printed
`Builds`, with no substitution.
`diff::checksum_lines::the_builds_line_is_the_original_s_range_by_range`
holds it, and `CHECKSUM_STATE` counts it: 4 of 12 lines, 6,427 of
425,499 bytes (§5).

**What moved it.** The last parting field (§14's range 334) was the
British Woodcutter's Camp `1/2001`'s `ObjectData::up_who`, ours −1 and the
original's 1. It is one of the four words of the world cell's chain of
objects, which this crate now threads with buildings and holds as the
original writes them — `up_who` and `down_who` left stale by both
`Object::add_to_world@0064d8c0` and `Object::remove_from_world
@00647970`, a building threaded at its `Object::init@00647750` on its own
point's cell, a number's two `who`s kept across a re-init
(`docs/COLLISION.md` §3.2). The camp's four words on both sides: `up` −1,
`down` −1, `down_who` 0, `up_who` 1. The emitter reads `Building::chain`;
it derived the words from the units on the building's cell before.

**The units' four words**, which stand in every unit's walked range of
the units line (§6, `ObjectData` `myhits`..`launch_frames`): all 52 units
agree on `up`, `down`, `down_who` and `up_who`
(`the_units_four_chain_words_are_the_original_s`, reported; the units line
is not ported). The chain alone of units gave 52, 31, 28 and 50.

**Made to fail**: the camp's `up_who` back at the derived −1 parts on
range 334 alone and moves the Adler; one perturbed byte (the fourth
building's `x`) parts on its own range alone; the empty image on all
519. Mutating `remove_from_world`'s head arm to clear the `up_who` of the
object below turns both tests red.

**Not established**: the frames after 91 (a building's close, a wreck, a
reused number: `docs/COLLISION.md` §3.2), and everything §14 lists but
the chain.

## 16. The guys line, ported (item 4104)

**The line is the original's on run 1562's frame 91**: 368 of 368
ranges, 9,308 of 9,308 bytes — 52 units, 54 figures, every one of the 42
fields — and the Adler **2502350161**, the printed `Guys`, with no
substitution. `Sim::checksum_guys` (`crates/sim/src/checksum/guys.rs`)
emits it; `diff::checksum_lines::the_guys_line_is_the_original_s_range_by_range`
holds it range by range and field by field; `CHECKSUM_STATE` counts it:
5 of 12 lines, 15,735 of 425,499 bytes (§5).

**Which units, in which order.** `CheckSums::check_guys@00937430` is the
units line's loop: per leader whose `leaders.list & 1` is set, in slot
order, per unit of that leader's band with `flags & 1`, in `o` order —
the human's seven, the British seven, then gaia's 38 herd animals (owner
8: four sheep, `TypeIndex` 0x198, and 36 fish, 0x19b) on frame 91.

**One unit's ranges, in walk order** (`PtrArray<Guy>::walk_data@0046df30`;
the stream settles it, and corrects §6's order):

| # | site | range | bytes | what |
|---|---|---|---|---|
| 1 | `0046df67` | a local | 4 | `length` (`+0x4`), the figure count; an empty array stops here |
| 2 | `0046dfc8` | a local | 4 | `size` (`+0x8`), the allocation |
| 3 | `0046dfd6` | `increment` (`+0xc`) | 2 | 1 on every unit of both frames |
| 4 | `0046dfee` | a local | 1 | `flags & 0xbf` (`+0x14`), 0 |
| 5 | `0046e199` | a local, one a figure | 1 | the figure's pointer is not null: 1 |
| 6 | `0046e220` | `size`, `increment` (`+0x8..+0xe`) | 6 | again, from the array |
| 7 | `005e0228` | each figure's `GuyData` `+0x8..+0xa3` | 155 | `type`..`guy_num` (`GuyData::walk_data@005e0210`) |

52 × 5 + 54 × 2 = 368 ranges; 52 × 17 + 54 × 156 = 9,308 bytes. Fifty
units have one figure and two have two (the scouts and their dogs); on
run1100's frame 93 have one, six two and two three, `size` equal to the
count on all 101. **`size` is emitted as the count**: `Unit::init@00612100`
grows the array to the crew it needs and never shrinks it, so a number
re-inited with a smaller crew would keep the larger allocation — not
read on any frame here. A `GuyData` `Coord` is stored **plain**:
`Guy::set_new_location@005d86f0` writes the value it is handed, where a
`SubObjectData`'s is masked.

**The 42 fields** — values first (the stream decoded per figure, ours
beside it from the minted walk on frame 91), then the port:

| field | ours | first image | now |
|---|---|---|---|
| `type`, `o`, `who`, `guy_num` | the unit's `type_index`, `index`, owner; the figure's slot | 54 | 54 |
| `x`, `y`, `angle`, `last_speed`, `avg_speed` | the body: guy 0's `Movement::body`/`facing`, a tracked crew's `Follow` | 54 | 54 |
| `z` | derives, by domain (below) | 54 | 54 |
| `des_x`, `des_y`, `des_angle`, `track_dx`, `track_dy` | guy 0: the unit's point and `heading`; a crew: `Follow` | 54 | 54 |
| `turret_angles[4]`, `des_turret_angles[4]`, `node_flags`, `des_node_flags` | `Turret` (zero on every figure here) | 54 | 54 |
| `cur_time`, `end_time`, `last_time`, `cur_anim`, `stopped`, `hold_attack`, `queued_attack` | the figure's clock (`Guy`) | 54 | 54 |
| `ox`, `whom` | `Guy::aim` (−1, −1 on every figure here) | 54 | 54 |
| `guy_flags` | derives: `init_real`'s `0x8`, `0x10`, `0x40`, `0x80`, `0x100` and `set_in_danger`'s `0x20` (16 on every figure here) | 54 | 54 |
| `variation`, `off_x`, `off_y` | `Guy::clear@005db590`'s 0; no writer read | 54 | 54 |
| the five floats | bits (below) | 54 | 54 |
| `gpiece` | **derives** for a gaia type (below) | 14 | 54 |
| `last_x`, `last_y`, `last_angle` | **carried**: `Guy::last` | 49, 51, 53 | 54 |
| `last_z` | derives from `last` by domain | 49 | 54 |

So 37 of the 42 agreed on all 54 figures from the first image; one was
held with another value (`gpiece`, on the 40 herd figures); four were
not held (`last_x`, `last_y`, `last_angle`, `last_z`). The first image
parted on 45 ranges and summed to 2626631255.

- **`gpiece`** — ours −1, theirs 60063, 60064, 60065, 60063 on the four
  sheep and 60072..60074 on the fish: this crate's herd figure keeps no
  piece. It is `GraphicPieces::get_unit_gpiece@0090c030`'s gaia arm,
  `first_bird_piece + 3·(type − 0x192) + (seed + o) % 3` (`docs/ANIM.md`
  §3; `first_bird_piece` 0xea8d), which the emitter derives; the
  figure's own `gpiece` is left as it is, so no reader meets anything new.
- **`last_x`, `last_y`, `last_angle`** — `Guy::move@005d9240:44–50`
  copies `x`, `y` and `angle` at its head, before anything in it moves or
  turns the figure; `Guy::set_angle@005d9010`'s snap and
  `Guy::set_new_location`'s seat write them too, and `Guy::clear` zeroes
  them. Carried as `Guy::last` (`docs/ANIM.md` §1), written at the head of
  `Sim::process_movement_on` for every figure the frame walks, and read
  only by this line. Before it, ours was the figure's own point: the five
  moving figures parted — the scout `0/0` and its dog (5554 against
  5520, 5506 against 5472), the citizen `0/1` (4023, 28316 against 4008,
  28296), the British scout `1/0` and its dog (45082, 19586 against
  45100, 19556; 45189, 19594 against 45206, 19565), the dog's
  `last_angle` −1785069568 against −1750794240 (its tracked body turned
  this frame).
- **`z` and `last_z`** — `Guy::set_new_location` writes `z` by the type's
  domain: an aircraft's climb, a ship's 0, and the ground under the point
  otherwise (`TerrainOut::find_data_z(x, y, 0)`, `World::data_z`). The 36
  fish read 0 for that reason. `last_z` is the `z` `Guy::move`'s head
  copies, the ground under `last` (373 against 379 on `0/0` before).

**The five floats** (`turret_inc`, `bank`, `last_bank`, `pitch`,
`last_pitch`, `+0x40..+0x54`) — **one bit pattern, `0x00000000`, on every
figure of both frames**: 54 of 54 on run 1562's frame 91 (citizens,
scouts, the sheep, the fish), 111 of 111 on run1100's frame 5408 (a team
game fighting: citizens 38, `TypeIndex` 61 six, caravans six, scouts six,
`0x197` four, fish 51). Their writers, every one in the export and the
listing (the names, each `+0x40..+0x50` store through a figure, and the
image's references):

| writer | stores | reached by |
|---|---|---|
| `Guy::clear@005db590` | all five 0 | every figure's birth |
| `Guy::init_real@005db6b0` | `bank`..`last_pitch` 0 (`param_1 == 0`) | every figure's init |
| `Unit::do_spec_anim@005e5880` | `pitch`, `bank` 0, the old value to `last_*` | a special animation |
| `Unit::bank_aircraft@005e9520` | `bank` ← a computed single, `last_bank` ← `bank` | an aircraft or a bird's flight |
| `Unit::pitch_aircraft@005e8de0` | `pitch` ← a computed single or 0, `last_pitch` ← `pitch` | the same |
| `Unit::come_out@00617c10` | guy 0's `turret_inc` ← `0x42b40000` (90.0, at `618003`, the image's one store of it to `+0x40`) | a missile (`type +0x1e4 & 0x8000000`) leaving its silo |
| `Unit::swap_team@006118d0` | `turret_inc` copied from the old figure | a conversion |
| `Guy::set_bank`, `Guy::set_pitch`, `Guy::set_elev_angle`, `Guy::copy_data` | a single | **nothing**: no call and no pointer in the image (a PE scan for each address, item 4104; `docs/audit/2026-11-06-territory-object-1.md`, `rondata::blind::RESIDUE`) |

Their readers are the renderer's (`GuyData::get_render_bank` and
`GuyData::get_render_pitch`, named and not entered by any trace) and the
dump. So, by DECISIONS
entry 120 §3, **all five are constant on this game's figures** and are
emitted as their bits: `+0.0`, the store of `Guy::clear` and
`init_real`. That is narrower than constant: on an **aircraft** or a
**bird** (`bank`, `last_bank`, `pitch`, `last_pitch`) the bits follow
from integers and singles this crate already holds — `air::Airframe`,
software floats by their bits (`num::single::Single`) — and the emitter
writes guy 0's from there; on a **missile** `turret_inc` is 90.0's bits
(`come_out_view::MISSILE_FIGURE_90`), which the emitter does not yet
write. No float enters the simulation.

**Made to fail**: a herd's `gpiece` back at −1 parts on its range
alone and moves the Adler; the first moving figure's `last_x` at its
own `x` parts on its range alone; a `-0.0` bank (the sign bit) parts on
its range alone; the empty image on all 368.

**How it was established, and how confident.** Diff-backed: every byte
of the line on one frame, and the five floats' one value on two frames
of the original's (run1100's is not a frame of ours). From the type
records: every field name, offset and width. From the walkers and the
stream: the order and the locals. On a reading, held by that diff on
this frame: the writers above, the gaia piece arm, the domain arms of
`z`.

**Not established**:

- Any frame but 91, and any lobby but the minted game's.
- **Agreeing by default on this frame** (one value on every figure, or a
  field no writer here keeps): `variation`, `off_x`, `off_y` (no writer
  read past `Guy::clear` and `Unit::init`'s `variation = 0`), the
  turrets and node flags (no pivot figure), `ox`/`whom` (no strike),
  `hold_attack`/`queued_attack`, `guy_flags` (16 everywhere: `0x8`,
  `0x40`, `0x80`, `0x100`, `0x20` unexercised; `0x2` — `Guy::do_turn`'s,
  cleared after each `Guy::move` — and `0x4` are not emitted), the
  floats.
- A bird's figure: its `bank` is `air::Flight::roll`, and its
  `last_bank`, `pitch` and `last_pitch` are not held; an aircraft's crew
  past guy 0; a missile's `turret_inc`; a converted figure's copied one.
- `size` past a re-init with a smaller crew; `last` past a teleport
  (`Unit::set_new_location(…, 1)`'s seat, which writes it equal to the
  point) or on a unit whose `Unit::process` this crate skips while the
  original runs it, and the converse.
- Owner 9's figures (none on either frame).

## 17. The units line, held out on three fields (item 4109; whole since item 4113, §18)

**1,078 of 1,084 ranges and 18,976 of 19,642 bytes agree** on run 1562's
frame 91 — all 52 units, their 4 path nodes and 13 orders, the 54 figures
(the guys line's 9,308 bytes, emitted by its own emitter) and the 40
animals' records. The six ranges that part are the `UnitData` records of
the scouts `0/0` and `1/0` and the citizens `0/1`, `0/2`, `1/1`, `1/2`, on
three fields between them. Ours sums to 280398657 against the printed
`Units` 27230589. `Sim::checksum_units` (`crates/sim/src/checksum/units.rs`)
emits it; `diff::checksum_lines::the_units_line_is_the_original_s_range_by_range`
holds it range by range and field by field, the three fields named with
their counts; `CHECKSUM_STATE` does not count it (§5).

**One unit's ranges, in walk order** (the stream's; `Unit::walk_data@0060cf40`):

| # | site | range | bytes | what |
|---|---|---|---|---|
| 1 | `006621fe` | `SubObjectData::flags` | 1 | `1`, `0x10` (`cant_reach`), `0x40` (below) |
| 2 | `0060d07f` | a local | 1 | `Unit::must_walk@0060d040`, `SubObject::walk_data`'s vcall: 1 |
| 3 | `0066221c` | `who`..`y_internal` | 15 | the owner, `o`, `z`/`x`/`y` masked; `z` is `World::object_z` |
| 4 | `00662287` | a local | 4 | the type's `TypeIndex` |
| 5 | `0060d07f` | a local | 1 | the same vcall, `Object::walk_data`'s: 1 |
| 6 | `00647885` | `ObjectData` `+0x20..+0x42` | 34 | the builds line's `OBJECT_FIELDS` |
| 7 | `00647910` | a local | 1 | `launching` (`+0x44`) is not null: 0 on all 153 units of both frames |
| 8 | `0060cfae` | a local | 1 | the inlined `must_walk`: 1 |
| 9 | `0060cfcd` | `UnitData` `+0x48..+0xb7` | 111 | `UNIT_FIELDS`, 49 fields; `Coord`s plain |
| 10 | `0046d8df` | `size`, `length`, `increment` | 9 | 10, the node count, 10 (a `char`) |
| 11 | `0046d980` | a node, `+0x0..+0x10` | 16 | `to_x`, `to_y` (plain), `tolerance`, `flags` |
| 12 | `00730291` | a local | 4 | the order count |
| 13 | `007303ac`, `007303be` | a local; the node's `+0xc` | 4 + 1 | per order, front first: its `OrderIndex` (vslot `+0x10`), the node's `metric` (0) |
| 14 | the order's walker | 1 + its fields | | the virtual base's `UnitOrder::flags`, then a move's `MoveOrder` `x`..`off_y` (76), a `BUILD_AT`'s `TargetOrder` `ox`, `whom`, `uid` (10), a `GATHER`'s that and `tx`..`been_there` (20) |
| 15 | the figures | §16's seven | | `PtrArray<Guy>::walk_data@0046df30`, the guys line's bytes |
| 16 | `005d7f0a`, `005d7f26` | a local; `AnimalData` `+0x150..+0x155` | 1 + 5 | an animal's: the inlined `must_walk` (1), `ox`, `whom`, `aid` (−1, `Animal::init`'s) |

**Values first** (item 4109's journal has the whole table): the first image
of ours, each field from where this crate held it, parted on 88 ranges —
the forty animals' `mylos`, `orders_x`, `orders_y` and `dest_angle`, and
the two scouts' `flags` — beside the three fields that still part. Agreeing
from the first image, every other field of the 19 `ObjectData`, the 49
`UnitData`, the path, the orders and the figures, once the unheld ones below
were carried or derived.

**Carried** (new state, read by nothing but this line):

- **`ObjectData::uid`** (`+0x30`) — `Object::init@00647750`'s per-player
  counter, which `Sim::add_unit` already advanced and now keeps on the unit
  (`Unit::uid`); a target order's `uid` is its building's.
- **`UnitData::los_x`, `los_y`** (`+0x78`, `+0x7c`) — written by
  `Unit::init@00612100:103` alone: the centre of the 48-unit cell it was
  handed, `div_3(v >> 4) · 0x30 + 0x18`. `UnitData::log_data` prints it.
  Kept as `Unit::birth_cell`, taken at `Sim::add_unit`. 52 of 52.

**Derived** (from state this crate holds): `inside_down`/`inside_down_who`/
`inside_up_who` (the owner while nothing is seated or the unit is not inside;
the container's owner inside); `launch_frames` 0 (a launcher's clock);
`gather_down` (the next unit down its building's gatherers, whose head is
the builds line's); `myarmor` (`ObjectData::armor`, the term
`Unit::update_armor` caches; `Sim::object_armor`); `cavarch_uid` (0, the
target's `uid` with one); `guy_mark` (`anim::SQUAD_SIZE`); `collide_frame`
and `damage_o` (−1, `Unit::init`'s, where the crate keeps 0 for "never");
`unit_masks` from the 17 bits the crate keeps; the `+0x86` union for a
caravan or a herd; a move's `off_x`/`off_y` (`x mod 0x300`) and `tolerance`
(0: nothing in the crate writes it, and run29's field table pins it); a
gather's `build_type` (the building's `TypeIndex`) and `non_flat_gather`
(`Unit::add_gather_order`'s lay: a gather type, not flat, not a University);
`flags & 0x40`, which `Object::init@00647750:39` sets for a type whose
`objmask` carries `0x2000000` — `combat::mask::DETECT`, the type's
`obj_masks` (the scouts).

**Corrected in the simulation** (the forty animals, `docs/VISION.md` §2,
`docs/ORDERS.md` §3.3): `Animal::update_los` writes `mylos` 0 for every
animal; `Animal::work` is `Unit::do_job` alone, so an animal never runs
`Unit::work`'s `update_action` and keeps `Unit::init`'s `orders_x/y` (the
herd's cell centre) and `dest_angle`; and `Unit::set_angle@00605400` writes
`angle` (`+0x50`) and never `dest_angle` (`+0x58`), so a herd's founding
leaves `0x55555555` there. Ours had the animal's own point, its facing and
4 — `8/0` (17448, 26616), 835132528, 4 against (17304, 26520),
1431655765, 0.

~~**What holds the line out** — each named in the test with its count:~~
**What held the line out** until item 4113, each carried there (§18):

| field | units | ours \| theirs | writers | what it would take |
|---|---|---|---|---|
| `play` (`+0xb6`) | 2 | −1 \| 0 on `0/0`, `0/1` | `Unit::init` (−1); ~~`SelectGroup::add`, the tutorial, the console~~ `CommandPackage::process_group` (§18) | ~~the interface's selection groups, which the simulation does not keep~~ the `group` command's write, `docs/ORDERS.md` §8.1 |
| `+0x86` (`hero`, `cara`, `doober`, `special`, `herd`) | 4 | −1 \| 0 on the scouts `0/0`, `1/0`; 0 on `0/2`, 2 on `1/2` | `Unit::init` (`Specials::init_special`, `Heroes::init_hero`, `Caravans::init_caravan`), `Unit::do_non_flat_gather`, `Unit::do_move`, `Unit::close` | ~~the specials' table and the doobers' slots, neither read~~ read and carried: `docs/SCOUT.md` §16, `docs/ORDERS.md` §6.12 |
| `unit_masks & 0x8` | 6 | clear \| set on all six | `Unit::do_move` (sets twice, clears twice), `Unit::find_path` (sets), `Unit::move_step`, `Unit::go_around_building` (clear) | ~~the movement step's own "a waypoint is out" bit, at six sites~~ `Unit::line_ok`, which the crate already kept |

**Made to fail**: the first image itself (88 ranges, the rules above
absent); in the test, one byte of each of the 24 kinds of range that agree,
bent, parts on that range alone; the empty image on all 1,084.

**How it was established, and how confident.** Diff-backed: every range
of 52 units on one frame, field by field, 1,078 of 1,084 whole. From the
type records and the PDB's field lists: every name, offset and width. From
the walkers and the stream: the order and the locals. On a reading, held by
that diff on this frame: `los_x`'s one writer, the animal's overrides,
`flags & 0x40`'s rule, the non-flat lay.

**Not established**:

- Any frame but 91. **Agreeing by default** (one value on every unit of
  both frames, or no writer here moved it): `damage`, `inside_down`,
  `hold_frames`, `near_o`, `near_who`, `healing`, `infiltrated`,
  `damage_frac`, `targeted`, `visible`, `launch_frames`, `damage_frame`,
  `trench_angle`, `queue_time`, `unit_masks2`, `inside_up`, `supply`,
  `collide`, `o_up`, `o_down`, `mana_burn`, `spell_time`, `myarmor`,
  `attrition`, `num_queued`, `cavarch_o`, `damage_o`, `cavarch_uid`,
  `cavarch_who`, `damage_who`, `full`, `waiting`, `recharging`, `safe`,
  `guy_mark`, the animal's three.
- The path stack's `size` past 10 nodes (run1100 carries 20 and 110; the
  emitter rounds the node count up to the next 10, the stack's growth not
  read) and after a long path shrinks; `inside_down` of a carrier; a
  `TRADE_ROUTE`'s walker and every order kind but a move, a `BUILD_AT` and a
  `GATHER` (their type and node byte alone are emitted); `flags & 0x8`
  (run1100's); `unit_masks & 0x200000` (`Unit::init`'s `can_carry(AIR)`)
  and the bits not listed (§18 names which agree by rule); `myarmor`'s cache timing and the cattle
  citizens' term; `los_x` of a unit the harness seats from a dump.

## 18. The units line, whole (item 4113)

**The line is the original's on run 1562's frame 91**: 1,084 of 1,084
ranges, 19,642 of 19,642 bytes, every field of the 52 units, their paths,
orders and figures and the forty animals' records, and the Adler
**27230589**, the printed `Units`, with no substitution.
`diff::checksum_lines::the_units_line_is_the_original_s_range_by_range`
holds it with `UNITS_HELD_OUT` empty, and `CHECKSUM_STATE` counts it: 6 of
12 lines, 35,377 of 425,499 bytes (§5).

**What moved it**: §17's three fields, each by a rule under its own
mechanic's document, read from every writer in the export.

| field | units | was \| now (theirs) | the rule | document |
|---|---|---|---|---|
| `unit_masks & 0x8` | 6 | clear \| set on `0/0`, `0/1`, `0/2`, `1/0`, `1/1`, `1/2` | the verified line, `Unit::line_ok`, which the crate already kept at all six of the original's write sites (`do_move` ×4, `find_path`, `move_step`, `go_around_building`) and through `Unit::init`'s 0, `swap_team`'s and `repair_damage`'s copies; the emitter had not mapped it | `docs/MOVEMENT.md`, "The verified line's lifecycle" |
| `+0x86` | 4 | −1 \| 0 on the scouts `0/0`, `1/0` | the specials list's slot, taken at birth | `docs/SCOUT.md` §16 |
| `+0x86` | (same) | −1 \| 0 on `0/2`, 2 on `1/2` | the hold doober's slot, the first free in one global table, taken at the tile and given back with a fade | `docs/ORDERS.md` §6.12 |
| `play` | 2 | −1 \| 0 on `0/0`, `0/1` | `CommandPackage::process_group`'s write of the package's player on each listed unit and its `o_down` chain — the minted `move_to` names `0/0`, the `build` names `0/1`, the Village's `queue_up` names a building | `docs/ORDERS.md` §8.1 |

**The values on both frames** (all units, the stream's, before any port;
item 4113's journal has the table): run 1562's 52 — `play` 0 on two and −1
on fifty; the union −1 on eight player units, the scouts' 0, the
woodcutters' 0 and 2, the forty animals' herd; `unit_masks` bits `0x2` on
39, `0x8` on 6, `0x400` on 2, `0x40000` on 6. Run1100's 101 — `play` −1 on
all (its commands are console lines, no `group`); the union 0 on three
scouts and two merchants (their caravan slots), slots 0 to 16 once each on
seventeen citizens of three players, −1 on 24, herds on gaia; `unit_masks`
`0x2` on 72, `0x8` on 42, `0x200` on 2 (the merchants: `Unit::do_trade`'s
bit, not held), `0x40000` on 40.

**Which `unit_masks` bits the emitter carries by a rule, and which agree
by default.** By a rule that a frame on disk exercises: `0x2` (the
figure's mirror, `Movement::mirror`), `0x8` (`line_ok`), `0x400`
(`was_builder`), `0x40000` (the AI bit). Emitted from a field the crate
keeps but clear on every unit of frame 91: `0x1`, `0x10`, `0x20`, `0x40`,
`0x800`, `0x1000`, `0x2000`, `0x8000`, `0x10000`, `0x20000`, `0x80000`,
`0x100000`, `0x800000`, `0x2000000` and the carrying nibble `0x78000000`
— these agree by default here. Not emitted: `0x4` (`Unit::in_danger`,
which the crate keeps), and `0x80`, `0x100`, `0x200`, `0x4000`,
`0x200000`, `0x400000`, `0x1000000`, `0x4000000`, `0x80000000`, which it
does not; every one clear on frame 91.

**Made to fail**: the three fields back at item 4109's emission (the bit
dropped, the union without the caster and doober slots, `play` at −1)
part on exactly §17's six ranges, the test naming `play` on 2, `u86` on 4
and `unit_masks` on 6; one perturbed byte of each of the line's kinds of
range parts on that range alone; the empty image on all 1,084.

**How it was established, and how confident.** Diff-backed: the whole
line on one frame, every field. On a reading, consistent with run1100's
101 units and not diffed there (it is not a frame of ours): the slot
tables' reuse after a death, the hold doober's 26-frame fade, the hero
list, the `num = 0` repeat of a `group`.

**Not established**: every frame but 91, and §17's list but these three
fields; §16 of `docs/SCOUT.md`, §6.12 and §8.1 of `docs/ORDERS.md` each
list their own.

## 19. The groups line, ported (item 4116)

**The line is the original's on run 1562's frame 91**: 556 of 556 ranges,
37,010 of 37,010 bytes, all 512 slots, the six in use with their one member
each, and the eight `last_group` words, with the Adler **3941922214**, the
printed `Groups`, and nothing substituted. `Sim::checksum_groups`
(`crates/sim/src/checksum/groups.rs`) emits it, and
`diff::checksum_lines::the_groups_line_is_the_original_s_range_by_range`
holds it range by range and field by field with `GROUPS_HELD_OUT` empty.
`CHECKSUM_STATE` counts it: 7 of 12 lines, 72,387 of 425,499 bytes (§5).

**The walk, in the stream's order** (`CheckSums::check_groups@00937530`,
`Group::walk_data@00708400`):

| site | range | bytes | what |
|---|---|---|---|
| `00708419` | `GroupData` `+0x4..+0x4c` | 72 | `id`..`march`, 21 fields (`GROUP_FIELDS`), every slot `0..512` |
| `0070843a` | `list` | 2·`num` | the members' `o`; a building group's are its buildings' (`2000`+) |
| `0070844b`, `00708464` | `off_x`, `off_y` | 4·`num` each | the formation offsets |
| `0070847d`, `00708496` | `curr_x`, `curr_y` | 4·`num` each | their rotation, plain `Coord`s |
| `007084ae` | `angles` | `num` | the slots' eighth-turns |
| `00937587` | a local | 4 × 8 | `last_group[who]`, `who·64 + s` |

The member arrays are walked only for a slot whose `num` is not 0, and
`list` comes first (§7 is corrected in place).

**The values, both frames** (the stream, decoded by the type record before
any port; item 4116's journal has the tables):

- **The unused slots**: 506 on each frame. Each one's `id` is its own index,
  so the 506 raw images are all distinct. With `id` set aside, run 1562
  holds **one** image: `Groups::clear@00713f20`'s, which is `army` −1, `num`
  0, `form` −1, `stamp` 0 and every other field 0, `who` included. Run1100
  holds **two**: that image ×505, and slot 67 (player 1's), whose `stamp` is
  2559 and `who` 0. That is a group emptied by `Group::kill`, whose
  `Group::clear(-1)` stamps the frame and writes `who` 0.
- **The six in use** have the same shape on both frames. A player's first
  push takes slot 1, because `last_group` starts at slot 0. A command's
  building group takes slot 0 or 2. A scout's group has `form` 0, its
  point, `order_num`, `role` `0x40110`, `speed`/`new_speed` 34 and
  `form_num` 1.
  - Run 1562: slots 0, 1 and 2 are the human's three minted commands. Slot 0
    is the Village's `queue_up` (stamp 60), slot 1 the Scout's `move_to`
    (30) and slot 2 Citizen 1's `build` (90, `facing` 1). Slots 64, 65 and
    66 are the computer's: buildings 2005 and 2000 on frame 1 and its
    scout's explore on frame 0.
  - Run1100: slots 64–66 and 128–130 are two computer players, and slot 0
    of player 0 is never pushed.
- **`last_group`**: run 1562 holds 2, 66, 128, 192, …, 448. Run1100 holds
  0, 66, 130, 192, ….

**What the crate held before this item, and what it holds now.** The pool
(`docs/GROUPS.md` §3, §19, §29, §31) already held all six records on the
right slots, with every field the original walks. The exceptions were
`role`, which it did not hold, and the three words in `docs/GROUPS.md` §3.4
that are constant by their writers. Read from that state, the first image
parted on **two ranges**: slots 1 and 65, `role` 0 against `0x40110`. Every
other range already agreed.

- **Carried**: `GroupState::role`. This is `Group::find_role@007081f0`, run
  wherever the crate stands in for `Group::normalize`, in both arms of
  `Groups::process` (`docs/GROUPS.md` §3.4), and zeroed by
  `GroupState::emptied`. `copy_group` leaves it, so slot 2 still reads the
  0 it held when it was pushed on frame 90.
- **By a writer's constant**: `disband` 0 and `priority` 0. `who` is the
  owner, except that an emptied record that was cleared reads 0.
- **From the slot table**: `army`, which is the army's number while it
  holds the slot and is linked, and −1 otherwise. `buildings` is a building
  group's 1. `last_group` is `who·64 + s`.
- **A slot the crate holds no record on** is emitted as `Groups::clear`'s
  image.

**Made to fail.** Slot 1's `role` set back to 0 parts on slot 1's record
alone. That is the first image's parting, which slot 65 shares. Emitting
`role` derived at walk time from the current members, instead of the
carried word, parts on slot 2 alone (`0x40300`, the Citizen's role word, against 0). Of each of the
eight kinds of range (a record, the five member arrays, `angles` and a
`last_group` word), one bent byte parts on that range alone and moves the
Adler. The empty image parts on all 556.

**How it was established, and how confident.** Diff-backed: the whole
line on one frame, every field of every slot. From the type record and
the PDB's globals: the field names and widths, and `last_group` as the
eight words (`const_last_group` is `0xe85f2c`, and the map puts `groups` at
`0xe85f10`). The following rest on a reading, consistent with run1100's
frame and not diffed there (it is not a frame of ours): the cleared
record's `who` 0, `disband` and `priority` constant by their writers, and
`find_role`'s callers.

**Not established**:

- Every frame but 91.
- A group of more than one member, and so the member arrays' values other
  than 0. Every in-use slot on both frames holds one member at offset
  (0, 0).
- `role` at a normalize site the crate does not mirror. The callers this
  crate stands in for are listed in `docs/GROUPS.md` §3.4, and others
  exist (`HotKeyGroups::find_group`, `Object::find_nearby_target`,
  `Unit::do_build`, the `is_*` queries).
- A building group emptied by `Groups::process`'s prune. The original
  keeps `buildings` 1 at `num` 0, and this crate's emptied entry would emit
  0.
- `army` on a pushed slot whose previous occupant was an army.
  `copy_group` leaves the number, and this emitter writes −1 unless a
  linked army still holds the slot.
- A slot an army and a pushed entry both hold. The emitter takes the army's
  record while the army has members.
- A frame where `get_open_slot`'s fallbacks fire.
