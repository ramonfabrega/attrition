# The oracle: how the original tells us what it did

`docs/FORMATS.md` has carried the same question at the top of its open list
since the beginning: does a recorded game embed the checksums and the random
log, or only the command stream? It was described there as "the highest-value
unknown", because the answer decides whether a replay diff is an exact
comparison or an inferred one.

This answers it, and then makes the answer mostly irrelevant, because the
executable ships something better.

**The recorded game carries the command stream, a full initial-state snapshot,
and the random seeds — but not per-frame checksums.** On its own that makes a
replay diff *inferred*: run the same orders from the same start and compare
what comes out, with no per-tick ground truth to compare against on the way.

**The executable also ships a per-frame, per-category sync tracer**, switchable
from an INI file dropped next to the binary, with three forced RNG seeds, a
configurable frame history, and a plain-text log in which **every entry names
the source file and line that emitted it**. That is an exact oracle, it is
localised to a line rather than to a tick, and it needs no patching and no
reverse engineering to turn on.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra. The `sSyncDefines` table and the configuration keys were read directly
out of `riseofnations.exe` at their own addresses. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the container's shape and for the package record,
which is a short function read end to end. High for the sync tracer's existence,
its 37 categories, its configuration keys and its output shape, all of which are
data in the binary rather than inference. **Untested throughout** — phase 2 has
not happened, this install ships no recorded games, and not one byte of any of
this has been read from a real file or produced by a running program. Every
claim below is a claim about code, not about a file that exists.

**Where the implementation is.** Nowhere yet, deliberately. A reader written
against no sample is a reader that cannot be wrong in any detectable way, which
is the opposite of what `docs/FORMATS.md`'s evidence rule is for. The reader
waits for a file to read.

---

## Part 1 — the recorded game

### It is not a struct, it is a visitor

`RecordGame` has a symmetric API — `write_header`/`read_header`,
`write_rules`/`read_rules`, `write_random_game_info`/`read_random_game_info`,
`write_package`/`read_package` — but the first three pairs do not write fields.
They run a **generic serialization visitor** over live objects:

```
write_header  →  Game::walk_data(SaveGame)   then String::walk_data(info.save_name)
read_header   →  Game::walk_data(LoadGame)   then the same
write_rules   →  Game::walk_rules_data(SaveGame)
```

`SaveGame` and `LoadGame` are the two subclasses of `DataWalk`, which is
sixteen bytes:

```
+0x00  int  (vtable)
+0x04  int  input      the Liberr from opening the file
+0x08  int  checksum   a running checksum over everything walked
+0x0c  int  flags
```

Two consequences matter more than the layout.

**The format is defined by every `walk_data` in the codebase**, not by a header
struct. `BuildQueue::walk_data`, `BuildData::walk_data`, `UnitType::walk_rules_data`
and their peers all participate; the file is whatever the object graph
serialises to. Deriving it in full means reading the whole walk, which is a
large but entirely mechanical job — and one worth doing only once there is a
file to check the result against.

**`DataWalk` carries a checksum field**, so the engine can checksum arbitrary
walked state. This is the machinery a full-state checksum would be built from,
and it is the reason the sync tracer below can be as precise as it is.

### The package record is small and fully derived

`RecordGame::write_package` is short enough to read completely. Per command
package it writes, in order, either to a `FILE*` or to a gzip stream:

| bytes | source | meaning |
| --- | --- | --- |
| 4 | `game->frame` | the frame this package belongs to |
| 4 | `package->play` | which player slot issued it |
| 4 | `package->valid` | |
| 4 | `package->stamp` | `CommandPackage +0x00` |
| 2 | `package->size` | payload length |
| `size` | `package->data` | the command payload |

and sets `record_game.empty = 0`.

`CommandPackage` itself is 536 bytes:

```
+0x00  ulong      stamp
+0x04  int        play
+0x08  int        valid
+0x0c  int        group
+0x10  short      size
+0x12  uchar[512] data
+0x214 Random     padding
```

Note what is **not** written: `group`, and the embedded `Random` at `+0x214`.

### The checksums are not in the file

This is the direct answer to the open question. `CommandPackage::checksums`
exists, but its decorated name is `?checksums@CommandPackage@@2PAKA` — `2` for
a static member, `PAK` for `unsigned long *`. It is a **static pointer to an
array**, not a per-package field, and it does not appear in the struct.
`checksum_fail_count` and `checksum_recheck` are static ints beside it.

They are live multiplayer desync machinery — computed as the game runs and
compared between clients — and `write_package` never touches them. So a
recorded game is orders plus a starting state, and a replay diff against it can
only observe divergence, not locate it.

### The rest of the container

`write_random_game_info` walks, for each active player slot, the region from
`info.player[n].team` to `.handicap`, a four-byte region on that player's
leader, and one four-byte field at `world + 0x30` — the seeds and per-player
settings a random map needs to regenerate identically.

`init_record` builds the path from `PlayerProfile::get_record_game_directory`,
`_wmkdir`s it, and timestamps the filename. `finalize` closes the file, and
unless `empty` is set, copies it through `recordgame.tmp` — so an abandoned
recording leaves nothing behind.

---

## Part 2 — the sync tracer, which is the real find

`SyncLogger` is a purpose-built desync diagnostic that the shipped executable
still contains in full: `beginNewLogSession`, `newTurn`,
`numEntriesCurrentFrame`, `setFrameHistorySize`, `setRecordingMode`,
`writeToFileAndReset`, `calcSizeInMB`, `mitigateMemoryExplosions`,
`reportSettingsAndOptions`, `setupWithConfigSettings`.

The two memory-management methods are the tell: this logs enough per frame that
it has to defend itself against its own size.

### It is switched on by a file, not a build flag

`SyncLogger::setupWithConfigSettings` sets `mSettingsReason =
SettingsCameFromConfig` and reads through `Prefs`. The path is a wide string in
the binary:

```
.\synclogger.ini
```

— beside the executable. The keys, also wide strings in the binary, are:

| key | what it does |
| --- | --- |
| `DesyncTrackingEnabled` | the master switch; `setupDefaultSettings` leaves it false |
| `DesyncTrackingFrameHistorySize` | frames retained; the default is 100 |
| `DesyncCategoryMask` | which of the 37 categories below to track |
| `CheckDesyncsEveryXFrames` | defaults to **1** — every frame |
| `DesyncUploadsWanted` | ships the result somewhere; leave it off |
| `SkipCountdown` | |
| `SimulationFps` | overrides the simulation rate |
| `RandomSeedHost` | forces the host seed; default −1 for "don't" |
| `RandomSeedGame` | forces the game seed |
| `RandomSeedMap` | forces the map seed |

Three forced seeds and a forced simulation rate mean **a run can be made
reproducible from a config file**, which is the property that makes any of this
usable as an oracle. Our own sim can then be seeded identically.

### The output names the line that produced it

The log file pattern is `SyncLog %s %s.txt` — plain text — and each entry is a
`SyncLogLineEntry`:

```
+0x00  SyncTags  mTag     which category
+0x04  wstring   mText
+0x1c  char *    mFile    source file
+0x20  int       mLine    line number
```

`mFile` and `mLine` are the point. The PDB carries 1,251 source file paths, so a
divergence does not merely land on a frame — it lands on
`borders.cpp:<line>`. That is a strictly stronger signal than a checksum
mismatch, which tells you only that two states differ.

### The 37 categories, read out of the binary

`sSyncDefines` is a static `SyncDefine[]` at `0x00c06380`, stride 20:

```
+0x00  SyncTags  mTag
+0x04  char[6]   mAbbrName
+0x0c  char *    mConfigName
+0x10  bool      mTracking
+0x11  bool      mLogTag
```

Read directly from the image, the whole table is:

| # | abbr | config name | | # | abbr | config name |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | `Note` | `NoteOnlySync` | | 19 | `Group` | `GroupsSync` |
| 1 | `Final` | `FinalSync` | | 20 | `Leadr` | `LeadersSync` |
| 2 | `Misc` | `MiscSync` | | 21 | `Guys` | `GuysSync` |
| 3 | `Perf` | `PerformanceSync` | | 22 | `GpcCd` | `GraphicChadsSync` |
| 4 | `ComMg` | `CommandManagerSync` | | 23 | `GpcEt` | `GraphicEventSync` |
| 5 | `Tunin` | `TurnTuningSync` | | 24 | `Goods` | `GoodsSync` |
| 6 | `TrnCt` | `TurnControlSync` | | 25 | `Items` | `ItemsSync` |
| 7 | `Time` | `TimeSync` | | 26 | `AnimC` | `AnimCheckSync` |
| 8 | `NetDm` | `NetDaemonSync` | | 27 | `PrgCt` | `ProgressChartSync` |
| 9 | `DrpCt` | `DropControlSync` | | 28 | `MapMk` | `MapMakeSync` |
| 10 | `ConnD` | `ConnectionDataSync` | | 29 | `Terrn` | `TerrainSync` |
| 11 | `World` | `WorldSync` | | 30 | `Pthfd` | `PathfinderSync` |
| 12 | `Citie` | `CitiesSync` | | 31 | `Chksm` | `ChecksumSync` |
| 13 | `Build` | `BuildsSync` | | 32 | `Sound` | `SoundSync` |
| 14 | `Units` | `UnitsSync` | | 33 | `Rules` | `RulesSync` |
| 15 | `Animl` | `AnimalsSync` | | 34 | `Scrpt` | `ScriptSync` |
| 16 | `Walls` | `WallsSync` | | 35 | `GpVfy` | `GpieceVerifySync` |
| 17 | `Ammo` | `AmmoSync` | | 36 | `Gmspy` | `GamespySync` |
| 18 | `Death` | `DeathsSync` | | | | |

All 37 ship with `mTracking` true in the static initialiser;
`setupDefaultSettings` then narrows it at runtime to categories 0 and 1 —
`NoteOnlySync` and `FinalSync` — so the mask in the INI is what re-widens it.

**This table is worth more than the tracer.** It is the original's own
enumeration of what is sync-critical, written by the people who had to debug
desyncs in it. Entries 11 through 30 — world, cities, builds, units, animals,
walls, ammo, deaths, groups, leaders, guys, goods, items, terrain,
pathfinder — are a list of exactly which subsystems must agree bit for bit,
and it is a list this project can check itself against today, with nothing
running. Notably `SoundSync` and the two graphics categories are on it too,
which says the original's renderer was not as cleanly separated from its
simulation as `docs/DECISIONS.md` entry 2 has us keeping ours.

---

## What this changes

**The plan does not need a recorded game to get an exact oracle.** `CLAUDE.md`
phase 3 proposes replaying a recorded game and scoring ticks before divergence.
That still works, and the container above is what it would read. But the sync
tracer is the better instrument and it is independent of recordings: force the
three seeds, enable the categories, play or replay anything at all, and the
original writes down what it did, per frame, per subsystem, with file and line.

**It sharpens what our sim should be able to emit.** The categories map onto
mechanics that already exist here — `LeadersSync` onto `docs/ECONOMY.md`,
`UnitsSync` and `DeathsSync` onto `docs/ATTRITION.md`, `BuildsSync` onto
`docs/PRODUCTION.md`, `WorldSync` and `TerrainSync` onto `docs/ATTRITION.md`'s
territory pass. A matching trace on our side, tagged the same way, turns the
diff from "the states differ somewhere" into "`LeadersSync` diverged at frame
4,312".

**It lowers what phase 2 has to achieve.** Booting the original stops being a
prerequisite for *trust* and becomes a prerequisite for *evidence*: the reading
can continue without it, and when it happens, one configured run yields more
than a recorded game would have.

---

## What is not established

- **Everything, empirically.** None of this has been run. The strongest claims
  here are about data in the image; the weakest are about what a file produced
  by that code would contain.
- **The format of a `SyncLog` line.** `writeToFileAndReset` builds it and has
  not been read past its prologue. Whether entries carry values or only tags
  and text decides how directly a diff can be automated.
- **Whether `DesyncTrackingEnabled` alone is sufficient**, or whether a
  multiplayer session, a lobby id, or `mIsMultiplayerRestore` is also required.
  `mKeyInfoForFilenameGeneration` mentions a lobby id, which is a hint that the
  tracer expects a networked game.
- **`DesyncCategoryMask`'s encoding** — presumably a bitmask over the 37 tags,
  but 37 exceeds 32, so it is either wider than an int or is parsed as a list.
  Unread.
- **The whole `walk_data` graph**, which is the actual header format. Read in
  outline only.
- **The recorded game's file extension and naming.** `String::time_stamp` builds
  it from two strings in the runtime string table rather than from literals, so
  it was not recoverable the way the INI keys were.
- **Whether `SimulationFps` moves the 15 frames per second** that
  `crates/sim/src/lib.rs` holds as `FRAMES_PER_SECOND`, or something else.
- **Where `read_package` stops** — the stream has no count that has been seen,
  so the reader presumably runs to end of file. Unconfirmed.
