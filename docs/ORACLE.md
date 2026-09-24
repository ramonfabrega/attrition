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
data in the binary rather than inference. ~~**Untested throughout**~~ — as of
2026-08-20 the original runs on this machine (CrossOver, D3DMetal), and the
tracer has been switched on and has written files; the last section records
what it and a second, older logger actually produce. ~~The recorded-game
container claims in Part 1 remain unexercised: this install ships no recorded
games and none has yet been made.~~ **As of 2026-08-24 a real recording is on
disk**: `ron.heavengames.com`'s downloads section holds ~245 recorded games,
and the one explicitly EE-era file (fileid 2170, "Seifer008 vs 4 Toughest AI",
2018, recorded on EE 1.2 build `00.2017.08.2100`) is saved at
`game/external-recgames/seifer008 - Toughest AI 4 v 1.rcx` (gitignored with
the rest of `/game`). It settles the extension (`.rcx`), the container (gzip
end to end — `write_package`'s gzip arm is the shipped path), and the header's
opening (a length-prefixed UTF-16 version string). **Version caveat:** this
install's executable carries the build string `00.2024.06.20` (the PE
VERSIONINFO resource is stale, `00.2009.09.1500`), so in-game *playback* of
2017-era recordings may be refused or desync — the author's own note says as
much — but a reader does not need playback, and ground-truth recordings for
the diff should be made by this install anyway (any game it records is
same-version by construction).

**Where the implementation is.** ~~Nowhere yet, deliberately.~~ The
**gamelog** reader exists as of 2026-08-20: `crates/rondata/src/gamelog.rs`,
written against two dumps from this install, with the constants check in
`dump.rs` and the diff harness in `diff.rs` (`docs/DATALAYER.md`). ~~The
recorded-game reader still waits for a file to read~~ — as of **2026-08-24**
it exists too: `crates/rondata/src/recgame.rs` (`rondata --recgame`), written
against the heavengames sample, and **`docs/RECGAME.md` supersedes Part 1 as
the container's document** — the full header walk, the embedded rules tables
(including the composed combat table, which the sample matches cell for
cell), and the package stream to EOF, which settles this document's
"where does read_package stop" question: end of file.

**Where the runs went.** As of 2026-09-18 this file is the **runbook**
only — the instruments, the keys, the staging channel, the lane and the
traps. The story of every capture, run by run, is `docs/RUNS.md`, moved
there whole by the fifth Fable pass; the machine-readable stanza of each is
`tools/gamelog/captures.txt`. A citation of a `runNN` heading that used to
point here points there now.

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

### It runs in single player

`SyncLogger::initialize` is called from `Main::main`, so the logger exists from
startup. `setupWithConfigSettings` and `beginNewLogSession` are then called
**unconditionally** from `Game::run_solo` and `Game::run_playback`, and
`beginNewLogSession` additionally from `SetupWin::setup_game`.

`run_solo` settles the question the first draft of this document left open. The
tracer does not need a networked game, a lobby, or a second client: a skirmish
against the AI drives it. And `run_playback` means it also runs while a
recorded game is being played back — so a recording plus the tracer gives a
per-frame trace of a real match, which is the combination worth having.

### It is switched on by a file, not a build flag

`SyncLogger::setupWithConfigSettings` sets `mSettingsReason =
SettingsCameFromConfig` and reads through `Prefs`.

**The file is not beside the executable**, despite the literal. The path in the
binary is `.\synclogger.ini`, but `Prefs::init` strips a leading `.` and
appends the rest to `Prefs::get_primary_app_directory`, which is
`SHGetFolderPathW(CSIDL_APPDATA)` plus `\Microsoft Games` plus
`\Rise of Nations`, creating each. So the real path is:

```
%APPDATA%\Microsoft Games\Rise of Nations\synclogger.ini
```

**The syntax is a standard Windows INI** — `Prefs::get`/`put` go through
`GetPrivateProfileStringW`/`WritePrivateProfileStringW` — and the section name
is the second argument to `Prefs::init`, an eight-character wide string at
`0xb18294`:

```ini
[Settings]
```

Nothing is created if the file is absent. `Prefs::init` copies a template when
one exists and otherwise leaves the defaults alone, and this install ships no
template, so the file has to be authored. The keys, also wide strings in the
binary, are:

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

## Running a check: the recipe in one place (2026-08-20, consolidated)

The sections that follow were written as the method was discovered, each
correcting the one before, and they are kept that way. This one is the
reading order for someone about to *run* a check: the recipe, with the facts
that are scattered below gathered into one table each. Everything here is
established further down; nothing here is new.

**The run, end to end.**

1. **Configure from files, not menus.** `rise.ini`: `AllowLogs=1`,
   `Seed=<fixed>`, `InitialDump=0`. `rise2.ini`: `Console Coord Mode=2`
   (tile coordinates for `add`/`move`), `StartConsole=0` unless the run wants
   the console (see the trap below). `gamelog.ini`: `DUMP_ALL=0` and, under
   `[End Frame]`, `UNITS=3 BUILDS=6 CITIES=5 DEATHS=1 LEADERS=1` — the values
   are **detail thresholds** (`tools/gamelog/setlog.py` writes the file). The
   lobby comes from `-config check.ini`, `-automation` drops the modal
   furniture; then Solo Game → Quick Battle → Start.
2. **Stage the situation with cheats**, each typed into the chat box by
   `tools/gamelog/cheat.sh` and **read back** before the next step: `add NEW
   tower 56,156` places a site on a tile, `add 1 citizen 32,141` a builder,
   `tech all on`, `war`, `damage select +N`, `die <o>,<who>`.
3. **Give orders** with `cheat select <o>` then a right-click
   (`tools/gamelog/rclick.sh`), one unit at a time, ~2 s apart. Aim by
   `cheat camera X,Y`, which puts tile `(X, Y)` at the viewport centre —
   desktop `(1719, 574)` on this machine's window — **re-measured 2026-08-24
   as `(1720, 620)`; the window moves between launches, so re-probe rather
   than trust either number** (`docs/INPUT.md` §10) — or by `aim.py` for any
   other tile.
4. **Read the answer** out of `Logs\gamelog.txt`: `lastframe.py` + `objs.py`
   + `one.py` for "what is on the map now", `track.py … --changes` for a field
   across frames. Object numbers are per player (units from 0, buildings from
   2000), so an object is `(kind, who, o)`.
5. **Quit through the in-game menu.** The `Log` system flushes per line so
   `gamelog.txt` survives a kill, but the end-of-game full dump and the sync
   log do not.

**Two input paths, and they do not mix.**

| path | opened by | prefix | reaches | notes |
|---|---|---|---|---|
| chat box | `Return` | `cheat ` | `run_cmd`'s second switch only | pauses the sim while open; executes on the next tick; the box sometimes does not open, so read state back |
| `~` console | `StartConsole=1` at launch | none | the whole 102-entry table (`?` lists it): `ai off`, `human`, `coord`, `pause`, `break`, `ffwd`, `quit` | invisible until it prints; in the run where it was open the mouse tile stopped updating and only a relaunch restored it |

A run is therefore **console-on** (tile coordinates, no mouse, no orders) or
**console-off with `Console Coord Mode=2`** (tile coordinates *and* a working
mouse, which an order needs). Keys reach the game through `osascript … keystroke`;
the mouse through `cliclick`; System Events clicks do not arrive.

**Coordinates, all four kinds.** One tile is 192 internal units; this map is
180 × 180 tiles.

| what | meaning |
|---|---|
| `add … x,y` in the default Coord mode | a **4-tile cell**: the object lands at internal `x × 768 + half its footprint`, i.e. near tile `4x + 2` |
| `add … x,y` in TCoord mode (`coord t` once in the console, or `Console Coord Mode=2`) | the **tile** `(x, y)` exactly — a tower centres on `tile × 192` (even footprint), a city on `tile × 192 + 96` (odd) |
| `add …` with no `x,y`, `move <o> cursor` | the tile under the mouse, per-tile |
| `cheat camera a,b` | centres the viewport on tile `(4a + 3, 4b + 3)` in Coord mode, on tile `(a, b)` in TCoord |

**The control that makes a placement verdict readable**: `add` force-places
and never calls `blocked_site`, so a placement cheat cannot test a placement
rule; the verdict arrives when the first builder does, and every refusal
looks the same (the site vanishes). Run the tile twice — a **tower site
first**, which is subject to the stricter territory test and the same
terrain, then the building under test — and a tower that starts leaves the
rule under test as the only candidate reason for a refusal. The full recipe
is "A scripted placement test, end to end" below.

## The detail level is the knob (2026-08-20, third session)

The two earlier sessions read `gamelog.ini`'s 37 per-category keys as
booleans, and read `DUMP_ALL` as the switch that adds per-object detail. Both
are wrong, and the correction is what makes a per-frame diff affordable.

**The ini value is a threshold, not a flag.** Every `log_data` announces the
detail level of the block it is about to write, by calling the `Log` vtable's
`+0x28` slot — `GameLog::set_detail`, which stores `current_detail`. Every
line then passes through `GameLog::check_accept`, whose only test is

> `if (detail_override == 0 && details[current_mode][current_type] < current_detail) return 0;`

`details[mode][type]` is the number parsed out of the ini for that category
under that phase. So `UNITS=1` does not mean "units on"; it means **"accept
unit lines up to detail level 1"**, and every richer field is silently
dropped. That is why two sessions of logging produced nothing but the object
base: the base is what `ObjectData::log_data` writes at level 2 or below, and
everything interesting sits above the threshold.

The levels each record uses, read out of the decompile (`0x28))(n)`):

| record | levels it opens |
|---|---|
| `ObjectData::log_data` | 2 |
| `UnitData::log_data` | 3 |
| `WallData::log_data` | 3 |
| `BuildData::log_data` | 4, 5, 6, 7 |
| `CityData::log_data` | 1, 2, 3, 4, 5 |
| `GuyData::log_data` | 1, 2, 3, 4 |

So the useful settings are **`UNITS=3`, `BUILDS=6`, `CITIES=5`**, and they are
what the mechanics' checks want: at `UNITS=3` a unit line carries `damage`,
`myhits`, `damage_frac`, `angle`, `attrition`, `supply`, `stance`, `myspeed`,
`myarmor`, `healing`, `collide_frame`, `damage_frame`, `orders_x/y`, `group`,
`hero`, `special`, `unit_masks`, `recharging`, `idle`, `num_queued` and some
forty more; at `BUILDS=6`, `job_counter`, `job_counter_2`, `constr_time`,
`construct_hits`, `helpers`, `city`, `city_down`, `recharging`, `attack_whom`,
`founder`, `wonder`, `fort`, `dock`, `orig_type` and the whole 20-slot
`BUILDQUEUE` with each entry's `type`, `job_counter` and three costs. Those
are, field for field, the quantities `docs/CITIES.md` and `docs/COMBAT.md`
list as needing a behavioural check.

**And the `OBJECT` level costs nothing** — every object record, unit or
building, at every detail level, carries `myhits`, `damage`, `damage_frac`,
`uid`, `hold_frames`, `infiltrated`, **`mylos`**, `visible`, `near_o`,
`near_who` and `healing`. `mylos` is `Unit::update_los`' whole output and
it turned `docs/VISION.md` §2 from prose into a 26,433-unit-frame diff for
the price of a `grep`; it is worth remembering that this level is free
before booking a run for anything it already prints. `los_x`/`los_y`, one
level in, are **dead**: `Unit::init` is their only writer.

**`DUMP_ALL=1` is a different thing entirely, and not the one you want per
frame.** It is `game_log.do_dump_all`, and its only use is as the argument to
`GameLog::full_dump`. Non-zero takes the early branch: set `detail_override=1`
— which makes `check_accept` return 1 unconditionally, ignoring every
threshold — and call `dump_all`, i.e. *every* subsystem regardless of its ini
key. Under `[End Frame]` that writes ~2.5M lines and ~70 MB **per frame**, the
game stops responding to input, and the run is useless. Under `[Start Game]`
with `InitialDump=1` it is exactly right, and it is the only way to get the
type tables (above).

**The recipe that works**, measured on this machine:

| configuration | cost |
|---|---|
| `DUMP_ALL=1`, `[End Frame]` anything | ~70 MB/frame, ~1 frame per 30 s, unusable |
| `DUMP_ALL=1`, `[Start Game]` only, `InitialDump=1` | ~150 MB once, ~4 min to load, has every type table |
| `DUMP_ALL=0`, `[End Frame] UNITS=3 BUILDS=6 CITIES=5 DEATHS=1 LEADERS=1` | **560 frames in 55 MB, full speed** |
| `DUMP_ALL=0`, `[End Frame] UNITS=3 BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1`, and the **same values under `[Start Game]`** | **432 frames in 60 MB, full speed** (2026-08-21, `gamelog-run6`) |

**Raise `[Start Game]` as well as `[End Frame]`.** They are separate threshold
tables and the harness needs both: the frames come from `[End Frame]`, but the
*initial state* the simulation is stood up from comes from `[Start Game]`, and
at its shipped `BUILDS=1` the start block carries no `orig_type` and no mining
list — so the buildings arrive untyped and a woodcutter has no tiles. Setting
both is what made `BUILDS=7` useful rather than merely present.

The last row is the setting for a behavioural check: fix `Seed`, set
`InitialDump=0`, play a minute, quit through the in-game menu. `~100 KB` a
frame buys every field the mechanics documents ask about.

~~**`LEADERS` is the exception, and it does not obey the threshold.**~~
**Corrected 2026-08-24 (run8, `gamelog-run8-personality.txt`):** it obeys
it like every other key. `LeaderData::log_data@006e5110` calls
`set_detail` ten times, levels 0–9 in source order, and
`GameLog::check_accept@009309a0`'s only filter is the ini threshold against
the current detail. The seven base fields are level 0; the leader's
internals sit at the levels between; and the **level-9 tail** is
`Personality::log_data` (the `PERSONALITY` block, 24 ints in `docs/AI.md`
§6's order), the tech bitmasks, `sites`, `make_list`, `mil_trainers`,
`new_rares`, `oil_patches` and **`prod_script`** — the script's name, the
last line of the record. So `LEADERS=9` under `[End Frame]` is the cheap
per-frame way to watch a leader, and `gamelog-run4-…-leaders9.txt` had 186
`PERSONALITY` blocks in it all along; the earlier claim was a misreading.
Two layout traps in that record, each of which has cost a wrong reading:
**`leader_flags`/`leader_flags2` are printed *before* `BEGIN LEADERDATA who
N`** and belong to the block that follows (the gamelog parser was corrected
for this in the pathfinder audit; a human reading the file trips on it
just the same), and `PERSONALITY` sits *inside* the block after `who`.

**`GameLog::end_game` calls `full_dump` too**, with the same `do_dump_all`
argument. Quitting through the in-game menu therefore writes a complete
final-state dump, which is the cheap way to snapshot a *late* state — one
dump instead of one per frame. (It does not rescue the leader problem above:
`do_dump_all` is read once at init, so a run that wants a full end dump is
also paying for full frame dumps throughout.)

**A `DUMP_ALL` frame writes its dump twice, and half of one window's
states were unreachable because of it** (2026-08-26). `full_dump` runs at
`begin_frame` and at `end_frame` (`docs/SYNC.md` §1). The `begin_frame`
one is written *inside* the `FRAME n` block; the `end_frame` one is
written at **`FRAME`'s own indent**, so the parser makes it a sibling
rather than a child. The two are the same state: for run29 all 2,583,636
lines of frame 15100's pair match except the `CHECKSUM` index,
`turn_control`, the two command stamps and the timing counters — so a
window costs twice what its frames do, and `Log::frames`, which takes the
nested one, is right to ignore the twin.

Where it is **not** a twin is the end of a run: `!quit` leaves a `FRAME n`
block with nothing under it and the final `full_dump` lands after it, as a
sibling with no following `FRAME` to duplicate it. run29's free 15105
state — the one run25–27 taught us to expect — was invisible to every
reader for exactly that reason. `Log::dumps` is the walk that finds it: a
frame's nested dump when it has one, the sibling that follows when it does
not.

The artifacts kept in the bottle's `Logs\` (they are large and outside
the repo, per `CLAUDE.md`): `gamelog-run1-fulldump.txt` (114 MB, the first
everything-per-frame run), `gamelog-run3-fulldump-types.txt` (152 MB, the
start-of-game dump **with the type tables and `COMBATTABLE`**),
`gamelog-run2-units.txt` (21 MB, 1,730 frames at the old detail 0),
`gamelog-run4-gunpowder-nubian-leaders9.txt` (72 MB, 47 frames at `UNITS=3
BUILDS=6` — the first dump with order lists) and
**`gamelog-run6-ancient-nubian-builds7.txt`** (60 MB, 432 frames at `UNITS=3
BUILDS=7` on both `[Start Game]` and `[End Frame]`, seed 12345, Nubians vs
Nubian AI, Ancient Age, Small Town — **the harness's dump**: it is the only
one carrying `gather_from`, and `docs/DATALAYER.md` §3 is measured on it),
`gamelog-run7-ancient-nubian-orders.txt` (260 MB, 1,732 frames, the paired
run with a recording — `docs/INPUT.md`), `gamelog-run8-personality.txt`
(93 MB, 60 frames at `LEADERS=9` — the personality and the census oracle)
**`gamelog-run9-world6.txt`** (62 MB, 36 frames, `WORLD=6` — the map,
see "The map is a dump too") and **`gamelog-run10-world6-long.txt`**
(272 MB, 1,772 frames, the same lobby with the map at start and run7's
per-frame detail — `[Start Game] WORLD=6 TERRAIN=2 GOODS=3 UNITS=3
BUILDS=7 CITIES=5 GUYS=2 LEADERS=9 DEATHS=1`, `[End Frame] UNITS=3
BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1`, no input, quit through the
menu at 1:58; its recording is `Playback - 2026.08.24 14'03'12`). Run10 is
the harness's long run with the map: 744 unlinked unit-frames against
run7's 1,970 on the day it was captured, and it exposed that the
harness's players earned no income — `Holdings` was never assembled from
the live gather chains (`docs/ECONOMY.md`); with that landed, **268**, all
of them the last citizen `1/10`, and `1/9` trains on the original's frame
— on the simulation's *own* sync stream. Two more of this lobby:
**`gamelog-run11-checksum.txt`** (30 MB, 144 frames, run10's settings plus
`check_all_level=14` and `[Misc Logging] CHECKSUM=2` — **the setup path's
checksum trace**, 146 records with the sync stream's state, see "The setup
path's checksum trace is the RNG state") and **`gamelog-run12-dumpall-
seeds.txt`** (`DUMP_ALL=1` on top of run11's settings, killed after a few
frames — the per-frame state, `game_random seed` at every `begin_frame`
and `end_frame`). A third: **`gamelog-run13-window-95-105.txt`** (642 MB,
25.95M lines, run12's settings plus `LogStartFrame=95` / `LogEndFrame=105`
in `rise2.ini` — **the frame window**, ten `DUMP_ALL` frame blocks at 95–104
and nothing at all for frames 0–94; see "The frame window is real, and it is
not the keys we guessed"). On run11's stream run10 scores **744** — the script's
`rand_int`s at frame 1 land on a stream displaced by the ~120 per-frame
draws the simulation does not model, and pick the rush order; the 268 was
the boom order reached by the sim's own stream's luck.

## The lobby is a file: `-config` and `-automation` (2026-08-20)

`System::init_cmdlineopts` splits the command line on `/` and `-` and matches
six options by name. Two of them matter here.

| option | effect |
| --- | --- |
| `-config <file>` | extension `rcx` → `sys.playback_file`; extension `ini` → `sys.autostart_file` |
| `-automation` | `sys.automation = 1` |
| `-inifile <file>` | replaces `prefs_file`, i.e. which `rise.ini` is read |
| `-distribution`, `-executable`, `-touchpatch` | patcher plumbing |

**`-config foo.ini` fills the lobby from the file.** `SetupWin::exec` reads
`sys.autostart_file` and, when it is set, calls
`GameInfo::load_from_config(info, file, 1)` in place of the recorded-game
branch. The file's `[CONTROL] USESECTION` names the section to read; every
lobby combo is then matched by name against its own option list —
`String::ignore` exactly first, then a prefix of 8 characters down to 3 — and
`PLAYERn_TRIBE` against the tribe list by exact name. `SEED` and `UNITBALANCE`
are read as numbers, and **a non-zero `UNITBALANCE` skips the whole combo
loop**, so leave it at 0. The shipped `autostart.ini` is a working template
and its `[OPTIONS]` section is the authoritative list of legal values.

It does **not** press Start: the flag it sets (`local_3c`) only reaches
`ConnectionData::init`. So a run is two clicks — Solo Game, Quick Battle — and
then Start, with every rule already correct.

Observed 2026-08-20: the rules half of the file took, while `mapstyles`,
`startingresources`, `revealmaps` and `PLAYERn_TRIBE` did not. **Not
established:** why those four differ. Confirmed the hard way on
2026-08-25: every run through run19 was on `MAP_STYLE 14` — **Great
Lakes** — while `check.ini` said Great Sahara, so `docs/AI.md`'s
`map_style` predicates were exercised on 14 and not on 7.

**`-config` pins the map style and no file can move it** (2026-08-29,
three wasted captures). With `-config check.ini` the style is the default
**14, Great Lakes**, whatever the profile says, and `mapstyles=` does not
take in either spelling (`East Indies`, `#ICON102East Indies`). **Without
`-config` the lobby is the profile's** and a written style does take. So
`tools/gamelog/mapstyle.py N` writes `check.ini` and all three of the
profile's copies — `<SETTINGS><MAP_STYLE>N</MAP_STYLE>`, and a
`<MAP_STYLE value="N"/>` in each of `<SOLO>` and `<MULTI>` — and the
capture scripts drop `-config` for any style but 14. Two corollaries:
**a traced run can never write the profile** (quitting
`riseofnations_trace.exe` through the menu dies in a `Program Error` box
first, which is why a combo pick never survives a launch), and the run's
own `GAME INFO` `MAP_STYLE` is the only read-back there is — **grep it
before reading anything else.**

**`-automation` suppresses the modal furniture.** `Options::exec` skips the
quit confirmation, `EndGameWin::exec` skips the end-game window, and
`AchieveWin::exec` and two `CommandPackage`/`CommandManager` error paths skip
their popups. It is the flag to pass for any unattended run.

`StartConsole=1` in `rise2.ini` is read by `Game::solo_checks` and calls the
console window's show slot at game start, which opens the `~` console and with
it the console-only half of `run_cmd`'s command table (`ai off` among them).
~~Set on this machine, no console appeared.~~ It does appear — it is simply
**invisible until it has printed a line**, so the first look at it found
nothing. See "The `~` console, which documents itself" below.

## The `~` console, which documents itself (2026-08-20)

`StartConsole=1` in `rise2.ini` does work — `Game::solo_checks` calls the
console window's show slot at game start — and an earlier session's "no console
appeared" was wrong in an instructive way: **the console is invisible until it
has printed something**, so a game where the chat box seems not to open is
usually a game where the console already has the keyboard. What gave it away
was a screenshot full of

> `>cheat camera 8,39`
> `>Unknown Command: cheat camera 8,39`
> `>Use '?' for list of commands, 'exit' to return to game.`

**The console takes the command with no `cheat ` prefix**, and it reaches
`run_cmd`'s *first* switch as well as the second — the half the chat box cannot
see. It also echoes what it did, which turns a silent cheat into a checked one:
`resource all +12345` prints the new totals, `camera 32,156` prints
`Camera X = T32 / Camera Y = T156`, and `select 1` prints
`Citizen, Red, (o=1)  HP=40/40  X = T1  Y = T8  Stance = …`.

`?` lists eight categories, and they are the engine's own documentation of the
command table. Transcribed from the running game:

**`? 1` Control & Misc.** `?` list; `cls` clear; `close`/`exit` close the
console; **`break #`** stop execution when the game frame reaches `#`; `go`
close, unpause and turn off reveal; **`quit`** quit the game; **`coord
(c|t|w|a)`** coordinate display **and read** mode — Coord, TCoord, WCoord, All;
`name (n|c|#|a)` who display/read mode — Names, Colors, Numbers, All;
**`pause 1|0`**; **`sandbox`** sets all players to human and the map to reveal
all; `safe` machine guns around every human capital; **`ai on|off|debug`**;
`diff 0-5`; `pointer text`; `netcmd` send a console command to all players;
**`ffwd #`** fast-forward to a given minute; `keys 1|0` the Alt-key cheats.

**`? 5` Player/Nation.** `be who`; `ally|peace|war who`; `meet|unmeet who`;
**`human who`** turn *off* computer control; `computer who` turn it on;
`defeat|victory who`; `tech who tech|all (on|off)`; `resource who
goodtype|all ±amount`; `age age who`; `military|civic|commerce|science level
who`; `library level who`.

**`? 7` Unit/Building/Object.** `select ob#|type who +`; **`object ob# who`**
show object info; `die o,who|select`; `damage (o,who|select) ±damage`; `craft
(o,who|select) ±craft`; `move (o,who|select) (x,y|cursor)`; `insert|add #
typename who=RED x,y`; `finish`; `next objtype`; `hurry`; `bird`; `nuke`;
`pack`; `deploy`; `anim #`.

The other four are `? 2` Tools & Script, `? 3` Audio, `? 4` Graphic System,
`? 6` Camera/Display, and `? Objects` — "objects that parse themselves".

**Three of these change what a behavioural check costs.**

- **`coord t` removes the mouse from placement.** In TCoord mode `add` reads
  its `x,y` as **tile** coordinates, so `add NEW tower 56,156` lands exactly on
  tile `(56, 156)` — no screen transform, no calibration probe, no camera. The
  mode persists after the console is closed, and it is also the `rise2.ini` key
  **`Console Coord Mode`**, where `2` is TCoord; setting it there gets tile
  coordinates in a run with the console switched off.
- **`ai off` and `human <who>`** take the opponent out of the experiment
  entirely, which the chat box could only approximate with `diff 0`.
- **`break #`, `pause`, `ffwd` and `quit`** are the makings of an unattended
  run: fast-forward to a frame, stop there, dump, quit.

**The console and the mouse did not coexist**, which cost a game to learn.
In the run where the console opened at start, the game stopped updating its
cursor tile: `add` at the cursor and `move … cursor` both acted on a stale
position near the map corner, and closing the console with `exit` did not bring
the tracking back — only a relaunch did. **Whether the console is the cause is
not established**; a focus interaction with the driving script is just as
consistent with what was seen, and clicking inside the window did not restore
it. Either way the practical rule held: a run is one or the other. **Console
on** for the
console-only commands and tile-coordinate placement with no mouse at all, or
**console off** (with `Console Coord Mode=2` in the ini) for tile coordinates
*and* a working mouse, which is what an order needs.

With the console off and TCoord set, aiming needs one measurement rather than a
fit: `cheat camera X,Y` centres the viewport on tile `(X, Y)`, and on this
machine's window that tile sits at desktop **(1719, 574)** — checked by placing
a tower at the cursor there and reading back `(40, 170)` for `camera 40,170`.
Any tile can then be put under the cursor by centring on it.

**The anchor is per-launch, not per-machine.** Re-measured 2026-08-24 with
the same probe — `cheat camera 16,160`, then `cheat add NEW tower` at the
cursor, then the site's tile out of the dump — it was **(1720, 620)**: the x
identical, the y 46 out, because the window came up at a different position
(the trap is already in "Traps that cost a run each"). One cheat line
re-probes it, so probe rather than trust a stored number.

## Staging a scenario: the chat cheats

A behavioural check needs a *situation* — two builders on one site, a citizen
against a tower, one squad hitting another from behind — and building one by
playing is slow and imprecise. The engine has a console for exactly this, and
in a solo game it is reachable from the chat box.

`ChatBox::on_modal_end` compares the typed line's prefix against
`get_cheat_string` (`translated_strings.xml` 263 = `"CHEAT "`,
case-insensitive). In a solo game the whole line is issued through
`CommandManager::issue_chat`, so **a cheat travels in the order stream and
executes inside the tick** — which is what makes it safe for a logged run.
`CommandPackage::process_chat` strips the prefix and hands the rest to
`ConsoleWin::parse_cmd`, which first captures the live mouse tile, then
matches token 0 against the 102-entry command table. `run_cmd`'s first switch
is skipped when the call came from chat, so only its second switch is
chat-reachable; the rest are `~`-console only.

The ones that stage a scenario:

| `cheat …` | syntax | what it does |
|---|---|---|
| `add` / `insert` | `[#] [NEW] typename [who=RED] [x,y]` | places units or a building **at the mouse cursor**. Without `NEW` a building is **completed instantly**; with `NEW` it is left as a construction site. Count capped at 300 |
| `be` | `[who]` | switches the viewpoint every other cheat defaults to |
| `war` / `peace` / `ally` | `[who \| All]` | `Leader::set_diplo(…, 0 / 1 / 2)` |
| `tech` | `[who] [techname\|all] [on\|off\|show]` | `gain_tech` / `lose_tech` |
| `age` | `[who] <n>` | `Leader::set_age` |
| `military` / `civic` / `commerce` / `science` | `[who] <n>` | `Leader::set_epoch` for that category |
| `library` | `[who] <n>` | all four epochs **and** the age |
| `resource` | `[who] [goodtype\|all] [+\|-]amount` | `bucket_set` |
| `die` | `[o[,who] \| select]` | kills |
| `damage` | `(o[,who]\|select) [+\|-]n` | writes the damage field, clamped to max hits |
| `move` | `(o\|select) (x,y\|cursor)` — **no `,who`**: `move 10,0 190,60` read `0` as the x and `190` as the y and put the unit at tile (0, 190) (run16) | `find_nearby_spot` + `set_new_location` |
| `finish` / `hurry` | — | completes the selected building, queue item or research |
| `select` | `[[ob#\|type] [who] [+]]` | selects by object number or type; `+` appends |
| `reveal` / `explore` | `[1\|0]` / `normal\|explored\|all` | vision |
| `ffwd` | `[minute]` | `fast_forward_frame = n × 900` |
| `keys` | `[1\|0]` | enables the Alt-key cheats (Alt+F5 resources, Alt+F9 hurry, Alt+Q reveal) |
| `diff` | `[0-5]` | difficulty |

`parse_who` accepts a player name, one of the colour words (`RED BLUE PURPLE
GREEN YELLOW CYAN WHITE ORANGE GAIA`), or a bare number **only** with a `who=`
prefix — which is why `cheat age 3` reads 3 as the age and not as a player.
`parse_type` takes a minimum-match prefix (3 characters for `add`) over units
50–401, buildings 414–542, techs 544–628.

Two things this buys beyond convenience. The AI can be taken out of the
picture (`ai off` is console-only, but `diff 0` and a `war`/`peace` set-up get
most of the way), and a scenario is *reproducible*: the same seed plus the
same cheat lines in the same order is the same run, which is the property the
eventual diff harness needs.

#### The coordinate argument, and what `add` does not check

`add`'s optional `x,y` is worth calibrating once, because it removes the mouse
from the loop entirely. Measured against the logged `x_internal`/`y_internal`
of what it places:

- **One internal unit is 1/192 of a tile.** The world dump's per-tile records
  come to 32,400 for this map, i.e. a **180 × 180** tile grid, and the largest
  coordinate seen is 34,272 < 180 × 192. (`xs 60 ys 60` in the `WORLD` block
  is therefore *not* the tile count; it counts something 3× coarser.)
- **One unit of `add`'s bare `x,y` is four tiles**, and the object lands at
  the cell's centre: `internal = arg × 768 + half a footprint`. `30,20`
  placed a tower at `(23424, 15744)`; `20,20` placed a city at
  `(15840, 15840)`. So the argument gives 4-tile granularity, and a distance
  that is not a multiple of 4 tiles cannot be expressed with it — use the
  mouse cursor, which is per-tile, when you need finer.
- The `w`-prefixed form in the shipped usage string (`w30,w20`) placed
  nothing; not pursued.

**`add` force-places. It does not run `blocked_site`.** Two cities were
placed four tiles apart, which no legality check would permit, and neither
`ConsoleWin::run_cmd` nor `Objects::init_build` references `blocked_site` at
all. This matters more than it sounds: **a placement cheat can never be used
to test a placement rule.** The legality check the rule lives in runs on the
*normal* build path, and — per `docs/CITIES.md` §3.3 — again inside
`Wall::do_construct` when the first builder reaches a site, where a verdict
outside `{0, ONE, ONE_OTHER, FARM, NEED_WALL}` disbands the site with a
refund. So the way to test a placement rule from a cheat-staged scenario is
to `add NEW` the site at the distance under test, send one builder, and watch
whether the site **starts** (`flags` 1 → 3) or **vanishes**.

#### Issuing an order without a human: `select` then right-click

The cheat vocabulary has no order verb, which looked for a while like a hard
floor on what could be staged from a script. It is not. **`cheat select <o>`
takes an object number, and a right-click afterwards issues the normal order
to whatever is selected** — a build order onto a construction site, a move
order onto ground. Both were verified against the log: after
`cheat select 3` and a right-click on a tower site's tile, unit 3's
`orders_x/y` became the tile beside the site; after a right-click on open
ground, they became that ground.

**Order units one at a time.** `select <o>` followed by `select <p> +` does
sometimes append — two portraits appear in the panel — but it is not
reliable, and a third `+` was seen to replace the selection instead of adding
to it. There is no need for it: orders are per-unit and persist, so
`select 3` → right-click, `select 4` → right-click, `select 5` → right-click
puts three units on the same job with no multi-selection anywhere. That is
the whole trick, and it means **every remaining behavioural check is
scriptable**; nothing needs a human to drag a selection box.

Two practical notes. Leave ~2 s between the `select` and the right-click —
the cheat travels in the order stream and executes a tick later, and clicking
too early orders whatever was selected before. And a right-click that lands
on nothing orderable produces *no* order at all, which is distinguishable in
the log from a move order onto terrain — a useful way to tell "I missed the
target" from "the target refused".

#### Aiming a right-click: the screen ↔ world transform

A building's sprite is drawn well above the tile it stands on, so clicking
the crates misses. Two anchors are enough to solve the projection, and it is
exactly linear in `u = wx − wy` and `v = wx + wy`:

```
screen_x = a·u + c        screen_y = b·v + d
```

Take one anchor for free from any `cheat add … x,y` (the cursor tile is the
object's tile) and a second from a right-click on open ground (the resulting
`orders_x/y` is the world point under the cursor). On the run this was
measured on, `a = 0.1605`, `b = 0.05167` — but they depend on the camera, so
re-derive after any scroll rather than storing them. `cheat camera x,y`
recentres the view, which is the cheap way to bring an off-screen site under
a known screen point.

#### A scripted placement test, end to end (2026-08-20)

The city-spacing check (`docs/CITIES.md` §2.6.2) was run this way, and the
recipe generalises to every `blocked_site` verdict. Five pieces, none of which
needs a human:

**Calibrate the transform against the game's own answer.** The two-anchor fit
above gets within a tile or two, which is not enough when the question is
whether 24 blocks and 25 does not. `cheat add NEW tower` at the cursor reads
the cursor's tile back exactly: a tower's footprint is **even**-sized, so
`snap_center` puts its centre at `tile × 192` with no half-tile, and the
logged `x_internal / 192` *is* the tile the mouse was over. Place one, read it,
shift the offsets by the error, place another, confirm. One tile of error in
both axes is a `v` error and no `u` error, so it moves `screen_y` alone by
`b × 384`. A **city** placed the same way centres on `tile × 192 + 96` — odd
footprint — and its `tile(x)` is still the cursor tile, which is what the
spacing loop compares.

**Then the camera is free.** `cheat camera a,b` centres the viewport on tile
`(4a + 3, 4b + 3)` — the same `× 4 + 3` the `add` coordinates use, one further
than the `+ 2` an *object* lands on, because the object is placed at a cell
centre and the camera at a tile. So `C` and `D` follow from the viewport
centre and `A`/`B` alone and there is no need to re-probe after moving:
`C = cx − A(x_cam − y_cam)`, `D = cy − B(x_cam + y_cam)`, with the viewport
centre measured once. `tools/gamelog/aim.py --cam a,b` does it, and it
reproduces both probe-calibrated points exactly.

**Put the builder where you want it.** `cheat move <o> cursor` teleports a unit
to the mouse tile (`find_nearby_spot` + `set_new_location`), so one citizen can
be reused for every trial instead of walking it across the map or adding a new
one each time.

**Remove what you placed.** `cheat die <o>,<who>` removes a building. The bare
`cheat die <o>` worked on an unstarted site and did nothing to a started one;
the `,who` form worked on both, so use it always.

**Control every trial with a tower.** A cheat-placed site is force-placed, so
the verdict only arrives when the builder does — and *every* refusal looks the
same from outside: the site vanishes. The way to tell which rule fired is to
run the same tile twice, once with a building that is subject to more rules
than the one under test. A tower needs friendly territory (`docs/CITIES.md`
§2.6.1) where a city does not, and needs the same terrain; a tower that starts
proves the tile is yours, is buildable, and is reachable. Then a city refused
on that same tile has one candidate cause left.

**Trial shape**, about a minute each:

```
cheat.sh <sx> <sy> "add NEW tower"     # the control
cheat.sh <sx'> <sy'> "move 7 cursor"   # the builder, a few tiles off
rclick 7 <sx> <sy>                     # select, then right-click the site
  ... BUILDDATA flags 1 → 3 and job_counter climbing  = the tile is fine
cheat.sh - - "die <o>,0"
cheat.sh <sx> <sy> "add NEW city"      # the test
rclick 7 <sx> <sy>
  ... the record disappears            = refused;  flags 33 → 35 = allowed
```

**Verify that every cheat landed.** Three `add` lines in a row did nothing at
all — no object, no camera move — and then the identical line worked a minute
later, so the failure is the chat box not opening rather than anything about
the command. Nothing distinguishes "the cheat was refused" from "the cheat
never arrived" except reading the state back, so read it back after every step
that the next step depends on. Two `cheat` syntaxes also matter: **`damage`
wants `select`** (`cheat select <o>` then `cheat damage select +20`; the bare
`cheat damage <o> +20` and `<o>,<who>` forms left `damage` at 0 on a unit),
and **`die` wants the owner** (`cheat die <o>,<who>`; the bare form removed an
unstarted site and did nothing to a started one). `add`'s type name is matched
against the **displayed** name, not the `TYPENAME` — `citizen` reaches
`PEASANTS`, `hoplite` reaches `HOPLITES` — and `add 1 hoplite` placed *three*
squads, so the leading number is not the count it looks like.

Two things that surprised the run. **The chat box pauses the simulation**: the
clock reads `PAUSED` for as long as it is open and resumes when the line is
submitted, so a cheat is always typed into a stopped game and always executes
on the next tick — the trap above about pausing is about the *menu* pause, not
this one. And **a city far from home is refused for reasons that have nothing
to do with spacing**: a site 51 tiles out, on unowned ground in a region where
the player had no city, was disbanded on arrival — the foothold and `COLONIZE`
branches of §2.4/§2.6.1 — which is why the ladder has to be run on ground the
tower probe has already proven.

## Traps that cost a run each (merged 2026-08-20)

Two lists were kept in two places as the sessions found them; this is the one
list, and `tools/gamelog/README.md` points here.

- **`F10` opens the Game Menu; `Escape` does not.** `key code 53` reaches the
  game and does nothing visible, which reads as "the window has lost the
  keyboard" and invites a relaunch. `key code 109` opens it (Return to Game,
  Options, Save, Load, Game Stats, Show Tips, Resign Game, **Quit Game**) and
  pauses the simulation while it is up. Quit Game returns to the main menu —
  which is where `Game::run_solo` has returned and the end-of-game `full_dump`
  has been written, so the dump is safe from that point on. (Exiting the
  *application* afterwards crashed Wine on 2026-08-21, after the dump; a
  `Program Error` dialog with `winedbg` behind it is the expected sight, and
  it costs nothing.)
- **The end-of-game dump is a sibling of the frames, not of the start
  state.** `GameLog::end_game`'s `full_dump` writes its `UNITDATA`/`BUILDDATA`
  at the same indent as every `FRAME n`, i.e. as further children of `BEGIN
  GAME`, *after* the last frame. A reader that takes "the object records among
  `GAME`'s children" as the start-of-game state therefore silently merges the
  first frame's world with the last one's — 27 buildings where the game began
  with 13, and 400 "citizens" on a long fulldump. Cut at the first `FRAME`
  child (`rondata::gamelog::records`).
- **Cheats are orders, and orders need ticks.** Issued while the game is
  paused (the *menu* pause) they queue and do nothing, and a probe that
  "failed" while paused looks exactly like one that was refused — check the
  frame counter is advancing before believing any negative result. The chat
  box's own pause is different and harmless: the sim stops while the box is
  open and the cheat executes on the tick after it is submitted.
- **Verify that every cheat landed.** Three identical `add` lines did nothing
  and a fourth worked a minute later; the chat box had not opened. Nothing
  distinguishes "refused" from "never arrived" except reading the state back,
  so read it back after every step the next step depends on.
- **Creating a city opens a modal rename dialog**, which pauses the
  simulation and swallows every subsequent keystroke, including the Return
  that would open the chat box. `add NEW city` (a site) does not; a completed
  one does.
- **Keystrokes sent while the sim is busy land wherever focus is.** During a
  `DUMP_ALL` frame the window stops servicing input for minutes; a `Return`
  meant for the chat box and the cheat line after it went into the city
  rename dialog instead, twice. Confirm the box is open before typing, and
  never type while the log is growing by tens of MB.
- **The window is 1920 × 1080 inside whatever the desktop is** — here a
  3440 × 1440 ultrawide, window top-left — and it is not always real
  fullscreen: relaunched from a terminal it came up borderless-windowed once
  and fullscreen the next time, with the menu coordinates ~70 px apart.
  Screenshot and locate before clicking; never trust stored coordinates
  across launches. `tools/gamelog/waitwin.sh` does the locate.
- **It does not take focus on launch.** `osascript -e 'tell application
  "System Events" to set frontmost of process "riseofnations.exe" to true'`
  before driving, and verify.
- **Keys go through `osascript … keystroke`; the mouse through `cliclick`.**
  System Events clicks do not reach the game. ~~`cliclick t:` types~~ — it may,
  but every script in `tools/gamelog/` types through System Events and that
  is the path that is known to work. Two `cliclick` habits: a fast `c:` often
  does not register, so press and release with a hold (`dd:X,Y w:250 du:X,Y`);
  and `m:` to the point the pointer is already on emits no motion event, so
  the game's cursor tile does not update — move somewhere else first
  (`cheat.sh` jiggles).
- **The console and the mouse did not coexist.** In the run with
  `StartConsole=1` the cursor tile froze near the map corner and `exit` did
  not restore it; only a relaunch did. Whether the console is the cause is not
  established, but the rule held: one run is console-on or console-off.
- **Screenshots: capture the game's rectangle, never the whole desktop.**
  `screencapture` returns the full 3440 × 1440 by default, and other
  applications' notifications land in the frame — which is what tripped a
  safety stop mid-session on 2026-08-20. `tools/gamelog/shot.sh -r X Y W H`.
- **Quit through the in-game menu, never `pkill`.** The SyncLogger writes only
  when `Game::run_solo` returns, and the end-of-game full dump comes from the
  same place. The `Log` system flushes per line, so `gamelog.txt` alone
  survives a kill.

---

## The map is a dump too: `WORLD` under `[Start Game]` (2026-08-24)

Every run so far had `WORLD=0`, and "the dump carries no terrain"
(`docs/DATALAYER.md`) was true of those runs, not of the logger.
`WorldData::log_data@006b6080` prints the world's scalars (`forest_size`,
`mountain_size`, `total_metal`, `total_oil`, `goodies`, `land_size`, `seed`,
the start positions) at levels 0–1, then **loops over every cell** —
`for i in 0..size: WData::log_data(wdata[i])` — and after that per-tile
and per-fog loops and `danger[8][reg_size]`. `WData::log_data@006af7e0` is
the per-cell record (`struct /rise.pdb/WData`, `docs/FORMATS.md`), by
detail level:

| level | fields |
|---|---|
| 2 | `land` — the terrain kind, printed as its `land_key[]` name |
| 3 | `flags`, `goods` (the nearby-goods bits `compute_site_stats` reads) |
| 4 | the flag words, spelled out |
| 5 | `who`, `who2`, `region`, `region2`, `val` (the city-site value byte), `land_sub` |
| 6 | `light`, `blocked`, `bad`, `solid`, `down`, `down_who`, the `block` bitmask, `was_seen` |

The level-4 words are one per bit — `BUILDING 0x4000`, `NEARBLOCK
0x200`, `HALFLAND 0x100`, `ROAD 0x80`, `FOREST 0x20`, `MOUNTAIN 0x10`,
`ROCK 0x8`, `COAST 0x4`, `GOODY 0x8000` — solved from run20's 3,600 cells
on 2026-08-25 and kept as `crates/sim/src/world.rs`'s `cell` module;
`0x1`, `0x40`, `0x400`, `0x800`, `0x1000` and `0x2000` print no word.

And after the cells, the start positions and the 57,600 tile masks, **the
fog grids** — `seen[scan]`, `seen2[scan]`, `seen3[scan]` as triplets, one
per fog cell (`fog_xs × fog_ys`, two per cell each way: 14,400 on a 60×60
map), one bit per player — of which `seen2` is what
`WorldData::was_seen@006b53f0` reads (run20, 2026-08-25; `rondata` loads
it, `docs/AI.md` §15.8). Then `danger[8][reg_size]`.

So **`WORLD=6` under `[Start Game]`** dumps the whole tile layer once —
regions, the coastal `region2`, the site values, the goods bits, the
owners — which is every map input the production AI's census and sites
read (`docs/AI.md` §2.3, §2.7, §2.13) and the forest layer the pathfinder
diff has been missing (`docs/PATHFINDER.md` §10). ~~`TERRAIN=2` adds
`TerrainData`'s height table and waterline.~~ **It does not** — `TERRAIN`
is not among the categories `full_dump` dispatches (run9 has no such
block); the heights are a `DUMP_ALL` item, see "The setup path's checksum
trace is the RNG state". It is a start-of-game cost only; leave `[End
Frame] WORLD=0`.

**Run9 (`gamelog-run9-world6.txt`, 2026-08-24, 62 MB, 36 frames)** is that
capture for the harness's lobby — run7/run8's settings and seed, `[Start
Game] WORLD=6 TERRAIN=2 GOODS=3` on top of run8's `UNITS=3 BUILDS=7
CITIES=5 GUYS=2 LEADERS=9`, quit through the menu after two seconds of game
time (the level-9 leader records make the sim crawl, ~0.3 frames a
second). What the `WORLD` block holds, in order: the scalars, then
**3,600 cells** at 17 lines each (`BASELAND`/`SANDY`/`OCEAN`, `flags`,
`goods`, the feature words — `NO FEATURE`, `NEARBLOCK`, `HALFLAND,COAST,`,
`NEARBLOCKFOREST,`, `RIVER,` … — `who`, `who2`, `region`, `region2`, `val`,
`land_sub`, `light`, `blocked`, `bad`, `solid`, `down`, `down_who`,
`was_seen`), then the six `SimpleArray<WCoord>` blocks (`start_x/y` =
(55, 21) and (4, 40), the city starts, the oil patches), then **57,600
`tdata[scan].mask` lines** — the per-tile `TData` masks, row-major
240 × 240 (10,749 ocean, ~1,800 forest as `0x30` under the blocked bits,
1,871 mountain, 810 river, 1,321 in a city radius) — then the fog run and
`danger[8][reg_size]`. `rondata` reads the cells and the tile masks
(`gamelog::world_cells`, `world_tiles`) and `build_sim` builds the world
from them; the flat world is the fallback. First effect: the AI's fourth
farm lands on the original's tile and its builder tracks the whole run
(`docs/DATALAYER.md` §3).

## `LEADERS=9` is the census oracle (2026-08-24)

The `LEADERDATA` record at level 9 is the whole of `LeaderData` — about
10k lines a leader a frame, most of it the `reg_buildings[64][129]` array —
and between the level-0 base and the level-9 tail sit **every count
`plan_strategy`'s sweep writes**, under the PDB's names: `active`,
`peasants`, `gatherers`, `free_peasants`, `home_reg`, `explored`,
`gather_slots[]`, `filled_gather_slots[]`, `escrow_rate[]`, `econ[]`,
`site_mark`, `production_step`, `script_step`, `effective_pop`, the
`reg_*[64]` arrays, `strategy[]`, `num_buildings[129]`, `num_units[352]`,
`num_queued[806]`, then `sites` (ten `SITE`s) and `make_list` (eleven
`MAKEOBJECT`s). `tools/gamelog/leader.py FRAME WHO [file]` prints one
leader's record flattened, arrays summarised to their non-zero entries.
Run8's frame 1 is the AI leader after its frame-0 sweep and is the
acceptance oracle for `crates/sim/src/ai_census.rs`; the human's block
carries `peasants`/`gatherers` too, from `Leader::calc_gather@006ceee0`
(the goods display's pass), not the sweep.

## The setup path's checksum trace is the RNG state (2026-08-24, run11)

The `CHECKSUM` category was never wired into `full_dump`'s per-category
dispatch, which is why every run with `CHECKSUM=1` printed nothing of it.
It is something better: **`GameLog::say_checksum@00930b30` is the map
maker's own sync trace**, and it prints the sync stream.

`say_checksum(detail, file, line)` is called at ~130 sites, all on the
**setup path** — `Game::init` (detail 1), `init_rules_and_teams` (1),
`init_teams` (1–6), `init_tribes` (5), `Setup::build_game` (1), `Map::make`
(1), `make_rivers` (1), `Terrain::init` (2), `mark_halfwater` (10),
`check_and_set_halfland_wcoord` (5), `TerrainGroups::place_all` (5),
`place_player_group` (10, 20), `place_oil_deposits` (20), `Leader::init`
(2) — and once more from `full_dump` at detail 100 under `DUMP_ALL`. Each
call passes `check_accept` against `[Misc Logging] CHECKSUM` (setup runs
under `GAMELOGMODE_NONE`, which is that section), then consults a static
`level`, read once through `prefs_get` from **`rise.ini`'s
`check_all_level`** (internal string 494; the game writes the key itself
with its default, 0). `level ≥ 1` prints `CHECKSUM n`, `FILE`, `LINE` and
an ammo checksum; each further level adds a walked subsystem's checksum
(deaths, groups, leaders, cities, items, goods, the world ×16, walls, guys,
units, builds, the rules); **`level ≥ 14` prints `game_random seed`** —
`GameAccess::game_random->random_seed`, the sync stream's state at that
call. `DUMP_ALL` sets `detail_override`, which forces `level = 0xff`.

So the recipe is two lines: `check_all_level=14` in `rise.ini` and
`[Misc Logging] CHECKSUM=2` in `gamelog.ini` (2 takes everything except
the per-cell loops at 5/10/20, which would walk the world sixteen times a
cell). **Run11** (`gamelog-run11-checksum.txt`, 30 MB, 144 frames, the
run9/run10 lobby) is that capture: 146 records, each `CHECKSUM n / FILE /
LINE / … / game_random seed`, in the preamble before `GAME INFO`.
`tools/gamelog/rngtrace.py` prints them with the number of `Random::get`
draws between consecutive records (a forward walk of the LCG — every draw
is one step, `docs/COMBAT.md` §9.5), and the whole setup stream reads off:

| between | draws | what |
|---|---|---|
| the lobby seed → `game.cpp` 6467 | 1 | the AI's random nation, `rand % 24` |
| `build_game` re-seeds; `setup.cpp` 716 | 1 | `world->seed = rand % 0xffff + 1` (one draw from 12345 either way) |
| `Map::make` 7805 → 7945 | 3, 5568, 1016, 3, 1824, 231, 2920 | the map maker |
| `make_rivers` 5076 | 173 | rivers; `Terrain::init` draws nothing |
| `setup.cpp` 884 → 955 | 8 | the start permutation |
| `leaders.cpp` 13383 → 13457, the AI | **20** | `random_personality` (the human's visit draws 0) |
| `setup.cpp` 1032 → 1148 | 1293 | the two `build_empire`s |
| `setup.cpp` 1169 → 1240 | 44 | `Herd::create_units` |
| → `game.cpp` 5024 | 0 | the rest of `Game::init` |

The last record — **`0x3bd39ae9` on this lobby** — is the state the
simulation enters frame 0 with, and run12's `DUMP_ALL` start dump prints
the same word at `begin_game`. `rondata` reads the trace
(`gamelog::Checksum`, `Initial.checksums`); `build_sim` seeds each computer
leader's personality roll from its `Leader::init` bracket and checks the far
end (`Personality::roll` from `0x9991b076` reproduces run8/run11's
`PERSONALITY` block field for field **and** lands on `0xf2299eda` — twenty
draws, the original's count), then installs the last record for frame 0.
With that stream the frame-0 sweep's site sampler takes the original's
stride (`site_mark` 16 in both, which needs the second draw exact), and with
the terrain heights (below) its best site is the original's `(52, 14)/370`
(`docs/AI.md` §12.1). Run9 and run10 predate the trace; `rondata --diff
--sibling` and `diff::run_traced` borrow it from run11, the same lobby.

**Two corrections that fell out.** `TERRAIN` under `[Start Game]` does
*not* dump the height table — `full_dump` never dispatches it (nor
`MAPMAKE`, `PATHFINDER`, `CHECKSUM`); the earlier claim above is wrong. The
heights come out only under `DUMP_ALL`, where `dump_all` prints
`terrain->master_land_heights` — `(4·xs+1)²` floats, `list[scan] 369.375000`
… with no block of their own, riding on the `UnbuiltForts` block that
precedes them — and then **`BEGIN REGIONS`**, every region's coordinate
list in the order `compute_sites` samples it (row-major, as
`Regions::rebuild_coords` writes it; `docs/AI.md` §13's open item, closed).
Run3 is this map's `DUMP_ALL` capture and supplies both.

**What the trace does not cover: the per-frame draws.** Under `DUMP_ALL`
`full_dump` runs `say_checksum` at `begin_frame` and `end_frame` too, so
**run12** (`gamelog-run12-dumpall-seeds.txt`, this lobby, `DUMP_ALL=1` with
`check_all_level=14`, killed after four frames, 262 MB) is the per-frame
oracle: `gamelog.cpp` 135 at `begin_game` and then at each `begin_frame`
and `end_frame` — **frame 0 draws 120** times from `game_random` where the
simulation's sweep draws 2, **frame 1 draws 54** (the script's eight
`rand_int`s and the farm's placement are the simulation's share), and
**frames 2 and 3 draw 6 each** — a steady six a frame from something that
is not the AI. ~~The 39 classes that draw from `game_random` include `Unit`
(16 functions), `Animal`/`Herd`, `Farms`, `Guy`, `Object`, `Ammo` and
`PathFinder`; modelling them is the next item (`docs/AI.md` §12.1) and it
is what the script's first `rand_int`s at frame 1 need.~~ **Read,
2026-08-24: `docs/SYNC.md`** — the frame's draw sites by phase, run12's
four frames attributed draw by draw against the dump's own outcomes (the
market's flux values, the animals' animation variants, the herd's step,
the farm sprout), the steady six being `Farms::inc_time`. `rondata` reads
the per-frame words (`Log::frame_seeds`), borrows them from run12 for its
siblings, and installs them frame by frame with the count on both sides.
~~What run12 also shows: **`Checksum Dump` / `Checksum Break`** in
`gamelog.ini` are read by `GameLog::init` right after `DUMP_ALL`, default
−1, into the `log_start_frame`/`log_end_frame` that `begin_frame` and
`end_frame` gate the dump on — very likely the frame window that makes a
`DUMP_ALL` trace of frame 100 a 130 MB file instead of a 6 GB one.
Untried.~~

## The frame window is real, and it is not the keys we guessed (run13, 2026-08-24)

**The window exists and works. It is `LogStartFrame` / `LogEndFrame` in
`rise2.ini`, not `Checksum Dump` / `Checksum Break` in `gamelog.ini`** — the
two pairs are read a hundred lines apart in `GameLog::init@00933190` and land
in different fields:

| ini | key | field | consumer |
| --- | --- | --- | --- |
| `gamelog.ini` `[Logging Options]` | `Checksum Dump` (int_str 2857) | `game_log.checksum_dump` | `say_checksum@00930b30`: when `checksum_count == checksum_dump`, set `detail_override=1` and `dump_all` **once** |
| `gamelog.ini` `[Logging Options]` | `Checksum Break` (2858) | `game_log.checksum_break` | `say_checksum`: when `checksum_count == checksum_break`, execute `int 3` |
| `rise2.ini` `[RISE OF NATIONS]` | **`LogStartFrame`** (2866) | `game_log.log_start_frame` | `begin_frame`/`end_frame`'s gate |
| `rise2.ini` `[RISE OF NATIONS]` | **`LogEndFrame`** (2867) | `game_log.log_end_frame` | same |

Both `rise2.ini` keys are read through `Prefs::Prefs(&prefs2_file,
&prefs2_key, …)` with the write-back argument **0**, which is why the game has
never written them into the file and why three sessions of reading
`gamelog.ini` never found them. Default −1 → no window.

`checksum_count` is the running index of `say_checksum` calls (the record
number `rngtrace.py` prints in its first column), **not** a frame. So
`Checksum Dump=N` is "one full dump at checksum record N" and `Checksum
Break=N` is a debugger trap at record N — under CrossOver with no debugger
attached that is a crash. Neither was run; both are read out of
`say_checksum`'s tail, which is four lines long and unambiguous.

**The recipe.** `gamelog.ini` `DUMP_ALL=1`; `rise2.ini`

```ini
LogStartFrame=95
LogEndFrame=105
```

Semantics as observed: the gate is `log_start_frame < 0 || (log_start_frame
<= gamec->frame && gamec->frame < log_end_frame)` — **inclusive start,
exclusive end** — and it wraps the *whole* per-frame `full_dump`, so outside
the window nothing at all is logged per frame, not even the `[End Frame]`
categories. `begin_game`'s dump is **not** gated and still costs its ~30 MB.

**Run13** (`gamelog-run13-window-95-105.txt`, 642,100,524 bytes, 25,954,510
lines) is the capture: run11/run12's lobby and settings with the window
above. Frames 0–94 ran at full speed and wrote **nothing**; the file then
holds exactly ten blocks, `BEGIN FRAME 95` … `BEGIN FRAME 104`, each
2,473,481 lines / ~61 MB / two `dump_all` passes, at ~70 s of wall clock per
block. The setup trace is byte-identical to run11/run12 down to `game_random
seed 0x3bd39ae9` at `begin_game`, so it is the same game.

**The frame label, settled.** `Game::do_frame@00591ef0` calls `end_frame` at
line 105 and `begin_frame` at 108, runs the frame, increments `this->frame`
at 293, and calls `end_frame` again at 315 — *after* the increment. So a log
block `FRAME n` holds **the end of simulation frame n−1 followed by the
beginning of frame n**, which is why run12's first block is `FRAME 1` and why
a window `[95, 105)` yields blocks 95…104. The consequence for the trace:
each block's first checksum record is the end of sim-frame n−1 (it carries
that frame's draws) and its second is the start of sim-frame n (always +0).
A window `[a, b)` therefore measures the per-frame draw counts of sim-frames
**a … b−2**, plus one cumulative figure for everything before `a`.

Run13's numbers, from `rngtrace.py`:

| sim-frame | draws | word at its end |
| --- | --- | --- |
| 0–94 (cumulative) | 1268 | `0x5f8f3d9d` |
| 95 | 23 | `0x14313bee` |
| 96 | 28 | `0xcf825dba` |
| 97 | 7 | `0xe5bc808f` |
| 98 | 6 | `0x259a53dd` |
| 99 | 8 | `0x60032f25` |
| 100 | 18 | `0xa45fecaf` |
| 101 | 21 | `0x08670a66` |
| 102 | 6 | `0xafa2e63c` |
| 103 | 6 | `0xd359bfe2` |

(Sim-frame 104's end falls at `gamec->frame == 105`, outside the window, so
its count is not in this capture — ask for `LogEndFrame=106` to get it.)
Against run12's 120 / 54 / 6 / 6 for frames 0–3 this is the same shape with
a bigger economy: a floor of six from `Farms::inc_time` and spikes where the
AI, a new farm or a herd steps.

## The draw-site trace and function coverage (run14, 2026-08-24)

The dumps show a draw's *outcome*; the draws that leave none — the scouts'
scan, frame 0's four-draw tail — were placed by elimination
(`docs/SYNC.md` §6). The instrument that shows a draw's *site* is
**`tools/trace/`**: an in-process DLL, `rontrace.dll`, loaded into a
**copy** of the executable (`riseofnations_trace.exe`, the original plus one
import descriptor in a new section — `patch_exe.py`; the install's own exe is
never touched), built freestanding with the tools already here (Homebrew
clang → `llvm-dlltool` → the pinned toolchain's `rust-lld -flavor link`; no
CRT, kernel32 only, no floats). It does two things:

- **Every step of the LCG, with its caller.** Four sites in `.text` carry
  the multiplier `0x19660d`: `Random::get()@00a39cf0`,
  `Random::get(int,int)@00a39d70`, **`MathUtilFuncSet::rand_real@009e18b0`**
  (the script VM's, which inlines the step on `game_random` and never calls
  `Random::get`) and `NukeOut::init_shroom_fire` (a local seed, graphics).
  The first three and `Random::reseed` are trampolined at entry (each has
  8–10 relocatable prologue bytes, checked against the expected bytes before
  patching); each call is logged with the RNG it hit, the seed before, the
  return address, two more frames of the `ebp` chain, and the sim-frame.
  `Game::do_frame` is trampolined too: the frame comes from `Game+0x550` and
  every FRAME record carries `game_random`'s word at that moment.
- **Function coverage.** An `int 3` on every one of the 48,233 function
  entries in the Ghidra export, caught by a vectored exception handler that
  logs the first hit, restores the byte and resumes — armed once at attach,
  and re-armed at the start of every frame in `rontrace.cfg`'s window. So a
  run yields the set of functions it ever entered (with the frame each was
  first entered on) and a per-frame set for the windowed frames.

`tools/trace/README.md` is the how-to and the log format; `report.py`
reads it (`summary`, `draws`, `sites`, `coverage`, `functions`, `blind`).

**Verified.** Run14 is this lobby, launched exactly as run11–13 were
(`-config check.ini -automation`), window `0-3`, 285 frames, quit through
the menu: the word at frame 0's `do_frame` entry is **`0x3bd39ae9`**, the
last checksum record of run11/12/13; the per-frame `game_random` counts are
**120, 54, 6, 6** for frames 0–3 and **23, 28, 7, 6, 8, 18, 21, 6, 6** for
95–103 — run12's and run13's numbers exactly. Run15, a different game
(launched without the arguments — see the traps), has its frame-0 word equal
to its own gamelog's `begin_game` record and to the seed its first draw read.
The traces are `Logs\rontrace-run14.log` and `rontrace-run15.log` beside the
gamelogs, with `gamelog-run14-trace.txt` as run14's sibling.

**Frame 0's 120, by site, in order** (`report.py … draws 0`; the
`Class::method+0xNN` is the return address inside the function that called
`Random::get`, then its framed ancestors):

| draws | n | site |
| --- | --- | --- |
| 0–1 | 2 | `Leader::compute_sites+0x4ac`, `+0x50a` < `plan_strategy` < `Leaders::strategy_all` |
| 2–19 | 18 | `GameDaemon::calc_market+0x54`, `+0x7e`, `+0xbe` × six goods < `calc_markets` |
| 20–23 | 4 | `Guy::set_anim+0x97a` < `Unit::set_anim+0x56` / `+0xb6` (two each) < **`Unit::do_idle+0x7d`** — unit-phase stands |
| 24–47 | 24 | **`Unit::think_scout`** — `+0x436` ×6, `+0x458` ×2, `+0x64c` ×16 < `Unit::think` < `Unit::do_idle` |
| 48–87 | 40 | `Guy::set_anim+0x97a` < `Unit::set_anim+0x56` < `Animal::do_idle+0x19` — the animals' idles |
| 88–107 | 20 | `Objects::process_all+0x2df`, `+0x30b`, alternating — the birds' sampling |
| 108–109 | 2 | `Herd::process+0x17`, `+0x36` |
| 110–113 | 4 | `Guy::set_anim+0x97a` < **`Guy::inc_time+0x271`** < `Unit::inc_time+0x3e` — phase-7 wraps |
| 114–119 | 6 | `Farms::inc_time+0x1ae` < `Objects::inc_time+0x147` |

So the scouts are **24**, not 23, at three sites of `think_scout`; the
four-draw tail is **not at the end** — the farms are last — but at 110–113,
and it *is* the phase-7 wrap (`Guy::inc_time` → `set_anim`), which is also
what leaves four guys at `cur_time 0` at the end of frame 0 (a wrap resets
the clock; `docs/ANIM.md` §9). The 4 at 20–23 are stands issued from
`Unit::do_idle` in the unit phase.

**Frame 1's 54:** 0–7 eight `MathUtilFuncSet::rand_int+0x18` <
`ScriptFuncSet::call_func` < `VirtualMachine::call_func` (the script);
8–43 thirty-six `Leader::produce_building` (35 at `+0xc99`, one at
`+0x1805`) < `ScenarioFuncSet::place_orphan_building_with_cost` <
`place_building_with_cost`; 44–46 three `Unit::do_non_flat_gather+0x54b` <
`Unit::do_gather` < `Unit::do_job`; 47–53 the farms — six at `+0x1ae` and
**one at `Farms::inc_time+0x1de`**, the sprout. **Frames 2 and 3** are six
farm draws each and nothing else, which settles frame 3: the sim's extra
`do_move` draw there is the sim's own (`docs/SYNC.md` §6).

**Run13's window, by site:** 95 — fifteen `think_scout` (`+0x436` ×6,
`+0x458` ×6, `+0x64c` ×3), two `Unit::do_idle` stands, six farms; 96 —
the twenty bird draws, **one `Guy::init_real+0x52` < `Unit::init` <
`Animal::init`** (an animal created that frame), one wrap, six farms; 97 —
six farms and one `Guy::set_anim+0x104b` < `Unit::set_anim` <
**`Unit::do_air_physics+0x683`** (a bird's animation change); 98 — farms;
99 — farms, one `Guy::init_real` < `Objects::init_unit` (the trained
citizen — one creation draw, plus one wrap, not two creation draws); 100 —
twelve wraps (the fish) and the farms; 101 — twelve **`GameAccess::rnd+0x20`
< `Unit::do_job` < `Unit::work`** (the farmers' `rnd(4)`; the chain skips
the frameless `do_gather`), one `Animal::do_idle` (the sheep's arrival), one
wrap (the scout), one `Farms::inc_time+0x1de` (a sprout), six farms; 102,
103 — farms.

**The setup path** is 13,105 `game_random` draws (`report.py … sites
setup`): `World::compute_val+0x343` 2,920, `Map::grow_valid+0x2b0` under
`grow_region`/`point`/`stamp` ~3,900 over nine call chains,
`grow_region+0x3d0`/`+0x404` 894, `Build::find_gather_tiles+0x10a` 572,
`PathFinder::calc_road_cost+0x46` 438 (the caravan roads),
`TerrainGroups::change_forest_base`/`coast`/`mountain` 1,116,
`nubify_transitions` 687, `make_continents` 438,
`Leader::produce_building+0xc99` under `Setup::small_city_buildings` 235,
`place_region_resource` 185, `astar_river` 172, `randomize_orthogs` 250,
`add_doobers` 410, `treeify_mountains` 92, forty `Guy::init_real` under
`Animal::init`, and a tail of smaller ones — the per-phase counts of
`docs/ORDERS.md` §9.2 can now be cross-checked site by site. Beside them
**254,806 draws on other generators** in setup and ~60 a frame after: the
UI's `IFaceRenderManager::tile_random` (its own `Random`), `place_tree`'s
(a heap object under `ObjectsOut::sort_trees`), `internal_random`
(`Surf::inc_time`, `update_wake_polys`, `modify_ocean_floor_texture`),
the particle system's, `JukeBox::shuffle`'s — the whole set the sim never
has to model, now enumerated rather than assumed.

**Coverage.** Run14 entered **6,585** of the 48,233 functions: 5,288 first
in setup (menu, lobby, map maker), 629 first at frame 0, 131 at frame 1,
the rest over 285 frames; per frame, 1,595 / 1,329 / 993 / 986 functions
ran on frames 0–3. Against the documents (`report.py … blind docs/`): of
the **423** functions `docs/` cites by `name@address`, run14 entered 267
and **156 never ran** — with run15 added, 155. Those are the claims with
no behavioural check behind them, and they group cleanly:

- **Attrition and supply never ran** — `Unit::process_supply`,
  `suffer_attrition`, `UnitData::get_attrition`, `in_supply`, `recharge`,
  `Supplies::find_supply`, `SupplyData::get_radius`, `HeroesData::find_hero`,
  `LeaderData::get_supply_upgrade`/`get_general_upgrade`. No unit has left
  its borders in any traced run. (The Nubian step check needs exactly
  this.)
- **Combat never ran** — `Unit::fight`, `find_new_target`,
  `target_opportunity`, `find_attack_pos`, `Object::take_damage`, every
  `do_attack*`, `do_strafe`, `land_plane`, `Group::action_attack`,
  `distribute_attack`.
- **The AI's C++ producers never ran** — `Leader::make_stuff`,
  `make_this`, `MakeList::make_me`, `create_units`, `create_buildings`,
  `research_techs`, `found_cities`, `produce_tech`, `produce_city`,
  `use_market`, `market_speculation`, `production_ai_setup`,
  `enable_production_ai`/`disable_*`. The third AI session implemented
  these from the reading alone; run14's 285 frames are all script.
- **Orders other than move/gather/build** — `add_`/`do_` for garrison,
  board, await_board, repair, guard, follow, patrol, group move/attack,
  form change, `go_to`, `go_to_unit`, `resolve_block`, `repath`,
  `find_nearby_spot`, the `Group::action_*` for them, and the command
  processors `process_attack`/`move_to`/`move_near`/`group`/`form`/
  `halt`/`patrol`/`siege_attack`/`attack_ground` — run14 had no input.
- **Economy** — `City::compute_trade`, `CityData::get_trade_value`,
  `lumber_level`, `Caravan::distance`/`trade_value`,
  `LeaderData::calc_rare`/`get_granary`/`get_smelter`/`resource_cap_add`,
  `Type::unpay_cost`, `Build::unpay_cost`, `Leader::can_pay`.
- **Tech** — `Leader::lose_tech`. **Cities** — `Build::finished`. **Path** —
  `find_tpath`, `find_wpath_army`, `get_estimate`. **Setup** —
  `init_wild_life`, `large_city_buildings`, `build_leader`,
  `place_start_in_region`. **Sync** — `Army::find_target`,
  `Animal::think_farm_animal`.

The list is a queue, not a verdict: each group names the run that would
exercise it (a border crossing with `cheat war`; a fight; a long run past
the script's steps; the recorded order stream of run7 replayed under the
trace; a caravan). Regenerate it after any run with
`report.py <log> blind docs/ <other logs…>` — it accepts several traces and
counts a function as entered if any of them entered it.

**Traps, each of which cost a run:**

- **Two threads on one `int 3`.** The first launch died with an unhandled
  `0x80000003` at `_Task_impl::scalar_deleting_destructor` — a PPL pool
  thread. Two threads had trapped on the same freshly armed entry; the
  first handler restored the byte, the second found it "not armed" and
  declined. The handler now claims every breakpoint at an address it ever
  planted on, armed or not.
- **The PDB's `section:offset` is an RVA, not a VA.** `game_random` at
  `0003:2300556` is `.data` (RVA `0x806000`) + `0x231A8C` = RVA `0xA37A8C`,
  **VA `0xE37A8C`**; the first build read `0x637A8C` and logged a constant
  word. `GameAccess::game_random` is the pointer to it at VA `0xC06184`
  (`0003:0388` — decimal 388 = `0x184`). The draw records carry the real
  `Random*`, which is how the mistake showed.
- **The lobby is not the launch line's.** `-config check.ini -automation`
  left the profile's lobby on screen (Nubians, Great Lakes, Small Town —
  run6's), and that run *was* run12's game; the plain launch that followed
  was a different game. Check the frame-0 word before reading anything.
- **The process is `riseofnations_trace.exe`**, so `waitwin.sh`'s
  `pgrep -f riseofnations.exe` and the `osascript` focus by process name
  miss it; the job's `win.sh`/`drive.sh` variants used the trace name.
- **The `ebp` chain lists framed ancestors, not callers.** `GameAccess::rnd`
  reports `Unit::do_job` above it because `do_gather` keeps no frame; the
  first entry (the return address at the hook) is always exact.
- The hooked functions carry no `int 3`, so a coverage report has to count
  them as entered from their own records (`report.py` does).

**What it does not establish:** which unit a draw belongs to — a record has
the site and the seed, not the object; the dump pairs it (`framediff.py`,
`anims.py`). The identities behind frame 0's four wraps and the variants
they read are the one open reconciliation (`docs/ANIM.md` §9). The setup
path's per-phase counts have not been cross-checked against
`docs/ORDERS.md` §9.2. Basic-block coverage (which *branches* ran, not which
functions) would need a different instrument — DynamoRIO's `drcov` does not
run under Wine; a `winedbg --gdb` single-step is too slow for a frame — and
is not needed for the question the blind list answers.

## The cheat channel: a scenario from a file (run16b, 2026-08-24)

`rontrace.cmd` beside the exe, one entry per line — `<sim-frame> <text>`,
where `<text>` is what would follow `cheat ` in the chat box, or `!` plus a
console-only command (`!quit`, `!ai off`); `#` comments. `rontrace.dll`
reads it at attach, and at the entry of `Game::do_frame` for that frame
hands each line to `ConsoleWin::parse_cmd(console_win, &line, from_chat,
no_mouse = 1)` — `from_chat` 1 for a cheat line, 0 for a `!` line — as a
const `String` built by `String::String(wchar_t *)@00a1edd0` and closed by
`~String@00a1ee20`. Every executed line is an `INFO cmd` record (frame,
index, half, `parse_cmd`'s return); `INFO cmds` at attach says how many
parsed. Lines run in file order; a frame lower than the previous line's is
clamped. `tools/trace/README.md` has the format.

**Run16b is run16 replayed from twelve lines** (`gamelog-run16b-cmd.txt`,
`rontrace-run16b.log`; the file is in the README), the same lobby and seed
(`0x3bd39ae9`), quit by `!quit` at sim-frame 2400. Every line ran on its
frame and shows in that frame's dump: `peace who=1` at 300 → `diplos[1]`
1 in the block labelled 301; `add hoplite who=0 206,78` at 330 → three
records in 331, period 8 at each figure's first refresh, 22 of 22 ticks on
the grid; `war` at 900 → 0 at each refresh; **`tech who=1 allegiance on`
at 1000 with the wagon already standing → `attrition 48` and no damage
for 290 frames, `unit_masks2` = 0x40000 on 1051, 1099, 1147, 1195, 1243,
1291 (the 48-grid) and cleared on the 32-grid — the sheltered-from-the-
start case run16 never reached**; `die 9,0` at 1300 → the bleed resumes;
`oath` at 1500 → 24; the AI's squad on Nubian ground at 1700 → 0, then 48
at 1910 after the human's `allegiance` at 1900, one tick, 0 as it walks
out; `!quit` at 2400 → the main menu, the log closed. What the typed run
took an hour of driving and ~19 chat lines with four in ten dropped, the
file took **twelve minutes unattended** and dropped nothing — and it can
be re-run to the frame.

What it changes in the recipe: a scenario is now a file; the keyboard is
gone from the loop; the remaining human-shaped steps are the three lobby
clicks (Solo Game, Quick Battle, Start twice — fixed coordinates while the
window stays at (760, 152), which it has for four launches) and reading
the window once. A driver is needed only for a right-click on a sprite.
The differences from the chat path, for anyone comparing: a line runs at
the top of the frame before phase 1, not in the command-processing phase;
it is not in the order stream (a recording of the run does not carry it);
it runs while paused; `no_mouse = 1`, so `add`/`move` without coordinates
have ~~no cursor tile to fall back on~~ **a stale one** — `parse_cmd`
refreshes `mouse_coord_x/y` only when `no_mouse == 0`, so an omitted
coordinate silently uses whatever the last real cursor read left. Always
give `x,y`. **In an unattended launch nothing has read the cursor, and the
field is heap.** Neither `ConsoleWin`'s constructor nor its `init` writes
it. run169's packet reads **(0, 6)** (item 652), and run168's draw stream
agrees with the same value. So a coordinate-less `add`, `move` or `bird`
lands in the world's corner on the two launches measured. Whether every
launch leaves it there is parked 653. `bird` has no `x,y` to give
(`docs/GOLDEN.md` §10).

## The channel's vocabulary, and what it cannot do (2026-08-26)

The whole console vocabulary is readable without running anything, and
`tools/gamelog/console.py` re-derives it from the user's own install on
demand. `ConsoleWin::init_cmds@007e1340` fills `ConsoleWin::commands` — an
array of `ConsCmd`, 0x28 bytes, name at +0 and help at +0x14, both `String`
— by copying entries out of `int_str_array`, the positional table the game
loads from `Data/internal_strings.xml` at stride 0x14. Parse the 204
assignments, divide by the strides, index the XML: **102 commands, each with
the help text the game itself prints.** One trap, and it cost an hour: eight
entries in that file are self-closing `<STRING/>`, and a reader that matches
only `<STRING>…</STRING>` silently shifts every index after each one — the
symptom is a table that looks almost right, with `BASE_ARMIES` and
`in_game_chat_box` sitting in command-name slots. The table is correct when
command 0 is `?`, which is `parse_cmd`'s own special case.

**The two switches are the reachability rule.** `parse_cmd` calls
`run_cmd(this, index, args, from_chat, no_mouse)`, and `run_cmd` opens

```c
if (param_3 /* from_chat */ != 0) goto switchD_007d6c52_caseD_9;
switch (param_1) { ... }          /* console only        */
switchD_007d6c52_caseD_9:
switch (param_1) { ... }          /* chat-reachable too  */
```

The two `case` sets are **disjoint** — 56 console-only, 45 chat-reachable, 101
of the 102 labelled (`pointer` has no label of its own). So a `.cmd` line's
`!` prefix is not a convenience: it selects which half of the vocabulary the
line can reach. `loglevel`, `restart`, `seed`, `mapgen`, `mapsize`,
`numplayers`, `break`, `go`, `pause`'s console twin and `quit` are all
console-only; `add`, `select`, `move`, `die`, `damage`, `tech`, `resource`,
the diplomacy verbs, `finish`, `hurry`, `pack`, `deploy` and `anim` are the
chat half.

**`move` is a teleport, not an order — and that is the ceiling on the whole
channel.** Case `0x4c` reads `mouse_coord_x/y` as the default destination,
lets a `x,y` token override it through `parse_coord`, and then calls
`Unit::find_nearby_spot` followed by `Unit::set_new_location@005f8d20` —
which is `Object::remove_from_world` and `Object::add_to_world` at the new
coordinate, with `Guy::set_new_location` for each figure. Nothing touches an
order list. ~~**No console command issues an order at all**~~ — **exactly
one does**: `bird`, table case 82, calls
`Unit::add_air_patrol_order@005e4350` (2026-09-19, item 365; `docs/GOLDEN.md`
§13). Otherwise the chat half is a set of state pokes. This closes the open question from run17 ("whether
`move`'s coordinate arm reads the mouse tile the channel does not supply")
in the other direction than it was asked — the command ran and did what it
does, which is not what an order does. Two details from the same case worth
keeping: `no_mouse` makes the channel *more* permitted, not less
(`if ((game->semaphore.ptr[0] & 4) != 0 && no_mouse == 0) break;` — a guard
the channel skips), and the destination defaults to the stale
`mouse_coord_x/y`, because `parse_cmd` refreshes them only when
`no_mouse == 0`.

The consequence for coverage is the one that matters: every
`Unit::add_*_order`, `Unit::do_*`, `Group::action_*` and
`CommandPackage::process_*` on the blind list is **unreachable from the
channel by construction** — ~~every one~~ **every one but
`Unit::add_air_patrol_order@005e4350`, which case 82 reaches** (item 365). Those need the real order stream, which is the
UI — run31's three right-clicks are the only thing that has ever entered
`CommandPackage::process_move_to`. A scenario file stages the world; only a
click or a hotkey orders it.

**`break` is an assert dialog, not a breakpoint.** Case 4 sets
`Game::frame_to_break`, and `Game::do_frame@00591ef0` reacts to it by calling
`Error::report` with `game.cpp` — a modal, which in an unattended run is a
hang rather than a pause. It clears `frame_to_break` afterwards and honours
`ignore_always`. For a frame-exact halt use `cheat pause 1` from the channel
instead, which reaches `TurnControl::issue_toggle_pause`; note that a paused
game stops calling `do_frame`, so the channel cannot unpause itself and the
driver has to.

**`loglevel` is a runtime lever, but only over `[Start Frame]`.** Case `0x1b`
is `loglevel <label> <level>` → `GameLog::set_level`, which matches the label
case-insensitively against the global `game_log_strings[]` (stride 0x14) and
writes `game_log.details[3][index] = level` for index < 0x24 — 36 category
slots. `loglevel reset` → `GameLog::reset_levels`.

Row 3 is `[Start Frame]`, and that is settled rather than assumed.
`GameLog::init@00933190` fills `details` with a nested loop — outer over an
array of five section-name `String`s, inner over the 36 `game_log_strings`,
`Prefs::get` per cell out of `.\gamelog.ini` — and the array is destroyed
with `_eh_vector_destructor_iterator_(&local_128, 0x14, 5, …)`, five
entries at `int_str` 0xde6c, 0xde80, 0xde94, 0xdea8, 0xdebc: **`Misc
Logging`, `Start Game`, `End Game`, `Start Frame`, `End Frame`**. So row 3
is `[Start Frame]`, matching the help text — and `GameLog::check_accept`
gates on `details[current_mode][current_type]`, where `begin_frame` sets
`current_mode = GAMELOGMODE_START_FRAME` and `end_frame` sets
`GAMELOGMODE_END_FRAME`.

The consequence is the useful part: **`loglevel` cannot touch `[End Frame]`**,
which is the section every capture so far and the whole of `rondata::diff`
are built on. Per-category verbosity at a frame is real, but it is a
`[Start Frame]` dump — a different snapshot point, an off-by-one against
every existing expectation, and not a free substitute for the
`LogStartFrame`/`LogEndFrame` window in `rise2.ini`. Whether a `[Start
Frame]` block at frame n is interchangeable with an `[End Frame]` block at
n−1 is unestablished and is the check to run before building on it.

## `restart` from the channel wedges the game (gate run, 2026-08-26)

Item 13's plan was many scenarios per launch, on the strength of `restart
<seed>` being a console command. It is, and it does what the reading says —
`run_cmd` case `0x5d` checks `Game::is_solo`, sets `(game->info).seed` from
`Syllable::parse` of the argument (a bare `restart` takes `timeGetTime()`
instead, so it is *not* reproducible), then `Game::close(game, 1)`,
`Game::init(game, 0, 0, 0)`, `Camera::outdate`, `WorldMap::reinit_all` and
`TurnControl::set_pause(turn_control, 1)`. No lobby, no loading screen, and
the game left paused.

**It cannot be driven from the channel, because the channel fires inside the
tick.** `rontrace.dll` hands each line to `parse_cmd` at `Game::do_frame`
entry, so `restart` closes and re-initialises the game whose `do_frame` it is
executing. The gate run staged `900 !restart 305419896`, `900 !go`,
`900 !ffwd 30`, and the trace ends with:

```
FRAME      3005                    (well, 901 for the gate run)
INFO cmd   0x5   0 0 1             !ffwd 30      ran, returned 1
INFO cmd   0x258 1 1 1             add hoplite   ran, returned 1
                                   ...and nothing for the restart
```

The `INFO cmd` record is emitted *after* `parse_cmd` returns, so its absence
says `parse_cmd` never returned. The window went black, the title bar stayed,
the process stayed alive, and no further `FRAME` record was ever written.
`!go` and the second `!ffwd`, queued behind it in the same frame's batch,
never ran either.

**What replaces it: the seed goes in `rise.ini`.** `Seed (0 for random)` is
the master game seed and is what the dump's own `game_random seed` line
reports back, so one launch per seed is reproducible, needs no re-entrancy,
and still regenerates the map — which is what the fuzzer wanted `restart`
for. `tools/fuzz/seedini.py` writes it. Note that `check.ini` also has a
`SEED=` and it is **not** the knob: "The lobby is not the launch line's",
above — the profile's last-used lobby is what appears, and `USEAUTOSTART=1`
does not auto-start either (the gate run's no-click launch sat on the Main
Menu with `frames 0` until the six lobby clicks were sent).

**What a fuzzed run costs, measured.** The `DUMP_ALL` window that the diff
needs is far more expensive than the `[End Frame]` figure in run18: a
`FULL DUMP` block is **~15 MB and about a minute**, not 1.7 MB and 2.4 s.
run29 is 16 `FULL DUMP`s and 251 MB for a five-frame window, and a 20-frame
window staged for seed 424242 was still writing at 311 MB when it was
killed at frame 3005. So the real budget is roughly

    launch + six lobby clicks   ~2.5 min
    fast-forward to the window  seconds (the dump is gated off)
    each dumped frame           ~1 min, ~15 MB

which makes a three-frame window about six minutes and 50 MB a seed — ten
seeds an hour, half a gigabyte. That is affordable overnight and it is an
order of magnitude away from the "one to two minutes a seed" the queue entry
assumed.

**The tension this leaves open, and it is the real one.** The score wants a
wide window — `ticks before divergence` can only reach as far as the dumped
frames go — and the stand-up wants a `FULL DUMP`, which is what costs the
minute. Whether the original can be made to emit one `FULL DUMP` at the
window's first frame and cheap `[End Frame]` blocks after it, in a single
run, is unresolved; `loglevel` cannot do it, because it reaches
`[Start Frame]` only. Until it is, a fuzzed seed scores over a handful of
frames rather than hundreds.

## The group pool is a per-frame record (2026-08-26)

`GameLog::full_dump@00930380` has two halves. `do_dump_all != 0` takes the
early branch — `detail_override = 1`, then `dump_all`, which is the 70 MB
a frame that makes `DUMP_ALL` unusable per frame. **The other half is a
list of `if (details[current_mode][k]) dump_<something>(this)`**, one per
`gamelog.ini` key, and `GameLog::end_frame@009329d0` calls it every frame
inside the `LogStartFrame`/`LogEndFrame` window. `details[mode][0x12]` is
`GROUPS`, and it gates `dump_groups` — the whole 512-slot pool, ~260
bytes a slot, ~130 KB a frame.

So **the group pool does not need `DUMP_ALL` at all**. `[End Frame]
GROUPS=1` puts it in every frame **without `DUMP_ALL`**, which turns a
ten-minute 250 MB window of three frames into a run that dumps hundreds.

**Measured, so nobody budgets from "full speed".** run31 wrote **684 KB a
frame** and ran at about **0.6 sim-frames a second** — 218 frames in six
minutes, 160 MB. That is ~25× cheaper per frame than `DUMP_ALL`'s ~80 MB
and ~12× faster, but it is *not* the 15 frames a second the engine runs at
with the dump off, and it is seven times heavier than the `UNITS=3
BUILDS=6 CITIES=5` line in the table above. The pool is 512 records
whether or not any of them is live, which is most of it. `docs/QUEUE.md`
item 20 was written expecting the expensive form; run31 used the cheap one.

**But it will not come out that way on its own, and the reason is a trap
worth carrying.** `GroupData::log_data@0045e1d0` never calls the `Log`
vtable's `set_type`/`set_detail` (`+0x24`/`+0x28`), so its lines are
accepted against whatever the **previous** dumper left in
`current_type`/`current_detail`. And `dump_deaths@0092fd80` ends by
calling `WorldData::log_data` **twice** — which is why a frame shows two
small `WORLD` blocks with `WORLD=0` — leaving `current_type` at `WORLD`
and `current_detail` high. With `WORLD=0` the pool is then dropped
silently: `GROUPS=1` is on, `dump_groups` runs, and not one line survives
`check_accept`. The start-of-game dump escaped it only because that
section had `WORLD=6`.

The fix that works, and the settings run31 used:

```
tools/gamelog/setlog.py 0 'end:UNITS=9,GROUPS=9,GUYS=9,LEADERS=1,MISC=9' \
                          'start:UNITS=3,GROUPS=1,BUILDS=7,CITIES=5,GUYS=2,\
LEADERS=9,DEATHS=1,GOODS=3,TERRAIN=2,WORLD=6,MISC=1'
```

**`DEATHS=0`** under `[End Frame]`, so `dump_deaths` and its two
`WorldData` calls never run and the type `dump_groups` inherits is `UNITS`
— which is then set high enough to accept anything. `setlog.py` takes
`CAT=N` now rather than only a bare name; the value is a threshold, not a
flag, and `GROUPS` is a category whose own record has no levels at all.

A second-order lesson: `LEADERS=9` is the expensive key. run30 ran at
1.6 MB a frame with `UNITS=3 LEADERS=9 GUYS=2`; run31 at ~370 KB with
`LEADERS=1` and `GROUPS` added.

## The 300-frame window, and the `DUMP_ALL` the fuzzer did not need (2026-08-26)

`docs/QUEUE.md` item 13's Tier 1 windowed with `window.py stage`, which
sets `DUMP_ALL=1`. The stated reason was that `rondata`'s `scene_at` wants
a `WORLD` block with its 3600 cells, and only `DUMP_ALL` was thought to
write them. **Both halves of that are wrong**, and the second was already
disproved by a capture sitting on disk:

- **`[Start Game] WORLD=6` writes the cells with `DUMP_ALL=0`.** run31's
  start dump has all 3600 of them (`grep -c who2` → 10800 over its three
  `WORLD` blocks). `WorldData::log_data@006b6080` calls `set_detail(2)`
  before its per-cell `WData::log_data` loop, so `WORLD=6` clears it
  comfortably. It is not a `DUMP_ALL` feature.
- **No frame inside the window needs a `WORLD` block at all.**
  `rondata::diff::run_traced` builds the sim from `log.initial()` — the
  **start** dump — and ticks forward to each logged frame. The per-frame
  blocks are compared against, not stood up from.

So the whole question "what does `[End Frame] WORLD=6` cost per frame"
never had to be asked. The cheap window (`window.py frames`, plus
`setlog.py`'s `[End Frame]` set) is what a fuzzed seed should have been
using from the start.

**Measured, on the same seed, back to back:**

| | `DUMP_ALL` window | cheap window |
|---|---|---|
| staging | `window.py stage 3000 3020` | `setlog.py 0 …` + `window.py frames 1000 1300` |
| dump | 249 MB | 207 MB |
| frames the harness stepped | **5** | **301** |
| per frame | 49.9 MB | **0.69 MB** |
| unit-frames compared | 39 | 3,913 |
| game start → dump settled | ~15 min | **9 min 22 s** (1.87 s a frame) |

**72× cheaper a frame and 60× more frames for less wall clock**, from ini
settings alone. `tools/fuzz/run.sh` uses the cheap window now.

**What the cheap window does give up: the height table.** `master_land_
heights` is a `DUMP_ALL`-only record, so a cheap-window capture stands the
world up **flat** — the diff says so in a note (`no height table (flat)`),
and `borrow_from_siblings` fills it in from a `DUMP_ALL` dump of the *same*
map when one exists. For run30/31 that was run3's. **For a fuzzed seed
there is no sibling**, because the whole point is a map no other capture
has, so every fuzzed seed is flat and everything the height feeds — the
pathfinder's cost, `calc_gather`'s non-flat term — is untested by it. A
seed worth keeping can be re-run once with `window.py stage` at three
frames to get its heights; that is the only thing `DUMP_ALL` is still for.

**And what it unlocked: three hundred early frames, not the first one.**
The `ticks before divergence` a run reports is an absolute sim-frame, so it
is mostly a statement about where the window was put — `ledger.py` already
says so and keeps `survived = ticks + 1 − lo` instead. But `survived` only
discriminates if the window opens **before** the simulation has drifted,
and almost every capture opened late: run13 at 95, run31 at 149, the
fuzzer's first at 3000. The exception is **run20**, whose `DUMP_ALL` window
was [0, 4) — four frames for 278 MB. What the cheap window changes is the
*width*: 300 early frames for 195 MB instead of 4 for 278.

So the third run of the day was the control: `scenario.py --no-stage`
(`FUZZ_STAGE=0`), which issues no cheat at all — not even `ai off`, whose
whole effect is to stop a leader the sim would keep playing — and a window
at **[1, 301)**. Seed 424242, 195 MB, ten minutes, 301 frames stepped,
3,913 unit-frames compared, and **20** unlinked instead of 3,610, because
nothing was spawned by cheat.

**It scores `survived = 1`.** Player 1's unit `o 0` is 24 position units
off on both axes at sim-frame **2** — an eighth of a tile — and player 0's
`o 1` by 10 units at frame 4. Before that, at frame **1**, `who 1 o 0` and
`o 6` already hold an order the sim has not issued (`Length { ours: 0,
theirs: 1 }`).

**run20 scores 1 as well**, on the tuned lobby, over its four frames. So
the number is *reproducible across two maps*, which is a better thing to
have than a novel one — and the first draft of this section claimed
sim-frame 1 had never been compared before, which run20 disproves and one
`--diff` would have caught. It was corrected the same day.

**The height caveat was checked, not assumed.** The control's world is
flat, so a 24-unit drift at frame 2 could have been terrain the sim cannot
see. So the seed got a heights sibling of its own — `window.py stage 1 3`
on the same `rise.ini` `Seed`, six minutes, 186 MB, then `rondata --diff
… --sibling <heights dump>`. With **58,081 heights pinned per tile** the
result is *identical*: player 1 at frame 2, `(7320, 41880)` against
`(7344, 41904)`; player 0 at frame 4, `(37840, 6385)` against `(37850,
6384)`. **`survived = 1` is the port, not the flat map.**

**Two leads came out of the same run, and only one of them needed a new
map** — which was settled by re-diffing run20 rather than assuming:

- **The frame-0 draw count is 15 short — on both maps.** `rng: frame 0:
  ours 180 draws, the original's 195` on the fuzzed map, and **160 against
  175 on run20**. The same fifteen. ~~So it is one missing block in the
  start-of-game path~~ — **and that inference was wrong, 2026-08-26
  (`docs/SYNC.md` §4.2): it is four blocks, and two of them cancel**,
  which is precisely why the shortfall came out the same on two unrelated
  maps and read as one thing. Three are fixed (a pasture's five animals,
  its missing crop draw, `Unit::think_scout`'s ten); the fourth is a
  ±4/±5 pair that nets to zero and hides a real ordering defect. The
  lesson is the section's own: *the same number on two maps is not
  evidence of the same cause.* Still a good lead, and still not one
  fuzzing found.
- **The start-of-game gather rule does not generalise.**
  `check_start_orders` (`docs/AI.md` §9.3) fails on one citizen: *`who 1 o
  6: we derived None, the log has Some(2001)`* — and is `[ok]` on run20.
  **This one is the map variation's**, and so far it is the only finding
  that is.

**What a control run costs, for the record:** 195 MB, ten minutes, plus
six more and 186 MB if the seed is worth its heights. Two runs, and the
second is optional.

**And what a re-diff of an old dump costs: thirteen seconds.** Three
claims in the first draft of this section were tested that way afterwards
— that sim-frame 1 had never been compared (false, run20), that the
draw gap was map-specific (false, run20 has it), and that the who-8 panic
needed the fuzzer (false, run31 panics on the pre-fix sim). Two of the
three were wrong. **Before crediting a new capture with a finding, run
the same diff against a dump already on disk**; it is the cheapest
control this project has and it had not been in the habit.

## The capture lane (2026-08-31)

The second lane of `docs/DECISIONS.md` entry 27, running beside the main
loop. Its products are logs outside the repo, so it needs no git
coordination at all; its repo writes are `tools/` and these run sections.
**The screen, the `ron` bottle and the install's INIs belong to it while it
exists** — a main-loop item that wants a behavioural check appends a stanza
to `tools/gamelog/captures.txt` rather than taking the screen.

**The queue is a file now.** `longtrace.sh` kept its four positional
arguments and grew the hooks a queue of captures needs — `DETAIL_END` and
`DETAIL_START`, `CMD_EXTRA` for the scenario's `rontrace.cmd` lines,
`TRACE_COVER`, `WINDOW` for a `DUMP_ALL` window, `SETTLE_MIN`, and `DRIVER`
for the right-clicks the cheat channel cannot issue. Every one defaults to
exactly what the script did before it existed, so an unset environment still
reproduces run33 and run39. `captures.txt` holds one stanza per owed
capture and `runqueue.sh` walks it **one at a time**, skipping any stanza
whose archive already exists, so an interrupted queue resumes rather than
restarts.

**Run numbers start at 42, and the reason is a trap worth naming.** 40 and
41 are taken by two `census` captures from 2026-08-30 that no run section
here mentions — and `rondata::diff` reads `gamelog-run40-census.txt`, while
`economy.rs`, `cost.rs`, `nations.rs` and `cities_tests.rs` all cite run40
or run41 by name. `longtrace.sh` archives its trace as `rontrace-run$N.log`
**unconditionally**, so re-using a number silently overwrites another run's
trace while leaving its gamelog beside it, which is the worst of the two
outcomes: the run still looks archived. Read the `Logs` directory, not the
documents, before picking a number.

**Two guards that were prose and are now checks.** `samegame.py --exclude
NAME` drops one record type from both digests, which is what makes the
same-game question answerable when a capture raises a category's threshold
to read a field — the raised category's blocks differ on every frame by
construction, and everything else is still compared frame for frame. It is
the weaker claim and the tool says so. `cmdsran.py` reads the `INFO cmd`
records back out of a run's trace and fails when a staged line did not run,
or ran and `parse_cmd` refused it: run23 is the standing example of a
capture whose scenario did not happen and whose dump looked ordinary, and
only a word-for-word comparison against run21 caught it.

## What is not established

- ~~**Everything, empirically.** None of this has been run.~~ **Run.** The
  container and `walk_data` claims remain unexercised; everything about the
  loggers is now observed.
- ~~**The format of a `SyncLog` line.**~~ Not observed in a no-desync solo
  game, which snips the frames at write time. **Open:** whether anything
  (`FinalSync`, `NoteOnlySync`, `DesyncUploadsWanted`, or a condition in
  `writeToFileAndReset` around `mDesyncOnTurn`) keeps them. One read of that
  function's middle answers it.
- ~~**Whether `DesyncTrackingEnabled` alone is sufficient.**~~ It is sufficient
  to set up and write the header in a solo game; the per-category keys are what
  was missing.
- ~~**`DesyncCategoryMask`'s encoding.**~~ There is none; 37 keys.
- ~~**`DUMP_ALL=1`, `Checksum Dump` and `Checksum Break`** in `gamelog.ini` —
  the first presumably unlocks the detail-level-1 fields, the other two
  presumably take a frame number; all three untried.~~ `DUMP_ALL` is now run
  and read ("The detail level is the knob" above): it is not a detail flag at
  all, it is the argument to `GameLog::full_dump` and it dumps *everything*
  every frame. ~~`Checksum Dump` / `Checksum Break` are still untried.~~
  **Read 2026-08-24** ("The frame window is real" above): they are a
  one-shot dump and an `int 3`, both keyed on the *checksum record index*,
  and neither is the frame window. The frame window is `LogStartFrame` /
  `LogEndFrame` in `rise2.ini`, and run13 is its capture.
- **The whole `walk_data` graph**, which is the actual header format. ~~Read
  in outline only.~~ The *recording* path — `Game::walk_data`,
  `GameInfo::walk_data`, `Game::walk_rules_data` and the wire primitives —
  is read in full as of 2026-08-24 (`docs/RECGAME.md`). The save-game graph
  (`write_save_game`, `World`/`Player::walk_data` and the other ~250
  walkers) remains outline only.
- ~~**The recorded game's file extension and naming.** `String::time_stamp` builds
  it from two strings in the runtime string table rather than from literals, so
  it was not recoverable the way the INI keys were.~~ **Settled 2026-08-24 by a
  real file** (below): the extension is **`.rcx`**, and the shipped write path
  is the **gzip stream** — the file is gzip end to end, decompressing to the
  walked stream, which opens with a length-prefixed UTF-16 version string
  (`Version: 00.2017.08.2100` in the sample).
- **Whether `SimulationFps` moves the 15 frames per second** that
  `crates/sim/src/lib.rs` holds as `FRAMES_PER_SECOND`, or something else.
- ~~**Where `read_package` stops** — the stream has no count that has been
  seen, so the reader presumably runs to end of file. Unconfirmed.~~ End of
  file, confirmed by the sample: the package stream parses to exactly EOF
  (`docs/RECGAME.md` §4.3).

## The fourth permission, and the probe that did not test it (2026-09-03)

Every capture script says it needs three macOS permissions — Screen
Recording, Automation and Accessibility — and probed all three before doing
anything. On 2026-09-03 a Claude Code update to 2.1.259 replaced
`~/.local/share/claude/ClaudeCode.app`, and the grants, which are keyed to
the bundle, went with it. Two came back with the obvious symptom:
`screencapture -x` wrote nothing and said `could not create image from
display`, and `osascript -e 'tell application "System Events" to get name of
first process'` hung at 0 % CPU and returned `AppleEvent timed out. (-1712)`.

The third did not. **`cliclick p` needs no privilege at all** — reading the
cursor position is not an Accessibility operation — so the Accessibility
probe answered a real `1649,0` while every `System Events` *UI-scripting*
call was still refused. `longtrace.sh` therefore passed its own probe,
staged the INIs, launched the game, and handed off to `waitwin.sh`, whose
loop asks

```
tell application "System Events" to get name of every window of process "riseofnations_trace.exe"
```

every three seconds. That call is the privilege, and it answers
`osascript is not allowed assistive access. (-1728)` — a *different* error
from the Automation timeout, and one the script never saw because the loop
swallows it and sleeps. The capture sat there with the game running fine and
no dump ever started.

**The fix is a probe that asks the same question before the launch.**
`Finder` always exists, so the four capture scripts now run

```
osascript -e 'tell application "System Events" to get name of every window of process "Finder"'
```

and exit on `-1728` with the bundle to re-grant. It was made to fail on
purpose first, which was free: the grant was still missing when it was
written.

**What to check when a capture stalls with the game up and no `gamelog.txt`.**
The three errors are distinguishable and each names its own toggle:

| symptom | permission | toggle |
| --- | --- | --- |
| `could not create image from display` | Screen Recording | Privacy & Security → Screen Recording |
| `AppleEvent timed out. (-1712)`, 0 % CPU | Automation | Privacy & Security → Automation → System Events |
| `not allowed assistive access. (-1728)` | Accessibility | Privacy & Security → Accessibility |

~~All three are granted to **`ClaudeCode.app`**, not to the terminal, and an
in-place update invalidates them — if the bundle is already listed, toggle
it off and on.~~ **Both halves of that are wrong**, and the second cost an
afternoon on 2026-09-03 before `tccd`'s own log was read; the truth and the
fix are below. `cliclick p` answering a position still proves none of them.

### The grant is keyed to a path with a version number in it (2026-09-03)

`tccd` names the process it blames, and it is not the bundle:

```
AUTHREQ_ATTRIBUTION: responsible={identifier=com.anthropic.claude-code,
  responsible_path=/Users/…/.local/share/claude/versions/2.1.259,
  binary_path=/Users/…/.local/share/claude/versions/2.1.259}
AUTHREQ_SUBJECT:     subject=/Users/…/.local/share/claude/versions/2.1.259
```

The responsible process is the **bare versioned binary**, not
`ClaudeCode.app`. It has no bundle, so TCC has nothing to key on but the
absolute path — and that path carries the version number. Two consequences,
both of which look like something else:

- **Every update revokes all three grants**, because every update writes a
  new path. This is not a stale checkbox and re-ticking does not fix it.
- **Adding `ClaudeCode.app` in System Settings does nothing at all**, because
  macOS never evaluates that path. The row appears, stays ticked, and is
  never consulted. Only a live prompt — or `+` pointed at
  `versions/<VERSION>` itself via ⇧⌘G — records a row that matches.

Read the verdict rather than guessing at it. `TCC.db` needs Full Disk Access
and its mtime is not evidence (a *denial* updates it too), but the log is
open:

```
/usr/bin/log show --last 3m --predicate 'subsystem == "com.apple.TCC"' --style compact
```

— the absolute path matters, a `log` shell function shadows it here — then
filter for `kTCCServiceAccessibility` or `kTCCServicePostEvent` and read
`AUTHREQ_SUBJECT`. It names the exact path being judged.

### `RonDriver.app`, which ends the tax (2026-09-03)

`tools/gamelog/rondriver/` builds `~/bin/RonDriver.app`: a fixed-path bundle
whose only job is to be the responsible process for a capture.
`tools/gamelog/viadriver.sh` runs one through it —

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 197
```

— and `open -a` is what makes it work: LaunchServices launches the bundle,
so the bundle, rather than whatever spawned the script, is responsible, and
every child inherits that. `viadriver.sh` itself returns at once, so **the
session waits with `tools/gamelog/waitrun.sh <the log viadriver printed>`**
under its background lane: the runner's summary banner goes to its stdout,
which is that log and never the `runqueue-<ts>.log` beside it, and item 571
watched the wrong one for two hours after its capture had finished (parked
656, the thirteenth pass). The launcher **spawns and waits**; an `exec`
would replace its image with `/bin/zsh` and hand the attribution back to the
interpreter, which is the whole bug it exists to escape.

Under it the subject is an identifier rather than a path, which is the
stable thing the native install never had:

```
responsible = {identifier=com.ramonfabrega.rondriver,
  responsible_path=/Users/…/bin/RonDriver.app/Contents/MacOS/RonDriver}
AUTHREQ_SUBJECT: subject=com.ramonfabrega.rondriver
```

Grant that bundle Accessibility once (`+`, ⇧⌘G, `~/bin/RonDriver.app`) and
approve the Screen Recording and Automation prompts on the first capture.
Nothing in the path is version-numbered, so the three survive every Claude
Code update. **Do not rebuild it casually** — a new cdhash costs a re-grant,
which is the tax being abolished; `build.sh` refuses without `--force`.

## The window nobody can see — CrossOver's expired bottle (2026-09-04)

**run 76 did not run, and the reason had nothing to do with permissions.**
Two launches through `viadriver.sh` reached `probe ok (synthetic move
landed)` and then sat in `waitwin.sh` for ten minutes apiece: the game
process alive, every wine thread at 0 % CPU, `wineserver` idle in its
select loop, `gamelog.txt` never created, and `tccd`'s log carrying no
denial for anything the lane touches.

`waitwin.sh` polls `System Events` for *windows of the process*
`riseofnations_trace.exe`, and the answer is a hard error — `Can't get
process "riseofnations_trace.exe"`. The app that **is** visible is
`wineloader`, and it has exactly one window:

    windows of wineloader: [Expired Bottle: ron]

CrossOver's licence for the `ron` bottle has expired, and the dialog it
puts up blocks the game before it ever creates a window. That is a
human-only action — renew or re-activate CrossOver — and nothing in the
harness can clear it.

**The probe, and it costs one line.** Ask System Events, *from inside
RonDriver's trust domain*, what windows the visible wine app has:

    zsh tools/gamelog/viadriver.sh <a script that runs>
      osascript -e 'tell application "System Events" to get name of every
                    window of process "wineloader"'

Run it whenever `waitwin.sh` has been polling for more than about two
minutes. Every other route was tried first and none of them says anything:
`sample` on the game shows a windowed app idling in `CFRunLoopRun`,
`sample` on `wineserver` shows the select loop, and the TCC log is clean —
because a dialog waiting for a human is not an error anywhere. **The
window title is the diagnosis**, and it is the first thing to ask for, not
the last.

Two rules follow. `waitwin.sh` looks for a process name the app may not
have — ~~`wineloader` until the game names itself~~ (under free Wine it is
`wine` for *every* GUI process, whatever executable it runs; the window is
matched by title in `tools/gamelog/focus.sh`, "Off CrossOver" below) — so a
stall there is always worth a window query rather than a second launch;
and a driver-lane
stall is *not* automatically the permission story
(`docs/ORACLE.md`, "The fourth permission"), which is what this session
assumed for half an hour on the strength of `cliclick`'s own warning.

## Off CrossOver — the lane runs on free Wine (2026-09-04)

**Why this was a project question and not a purchasing one.** The capture lane
is how every remaining mechanic gets settled (`docs/DECISIONS.md` 29, and both
long words sit near 6,800 of 24,000), so the oracle is needed for months yet.
A licence that renews is a dependency on somebody else's business decisions,
on a project whose whole point is outliving its source material — the OpenTTD
line in `CLAUDE.md`. **It is now off that dependency**: WineHQ Stable 11.0 plus
two free, redistributable pieces reach the main menu and run the traced
executable.

**What CrossOver was actually providing was neither of the two things we
thought.** `wow64` is upstream Wine's, and D3DMetal — Apple's, in the free
Game Porting Toolkit — is **x86_64-windows only** in CrossOver's own bundle
(`lib64/apple_gptk/wine/` has no `i386-windows`), so it never served this
PE32 game at all. The Game Porting Toolkit was the wrong answer to a
32-bit question.

### The blocker had a name, and it was not the one on the box

`d3dgl.dll` owns the *"Could not initialize DirectX! … DirectX 10 or higher"*
message, and despite the name it is **not** a D3D-to-OpenGL wrapper: its
import table names `d3d11.dll` and `D3DCOMPILER_47.dll`, and at
`d3dgl+0x240c0` it makes exactly one call —

    D3D11CreateDevice(NULL, D3D_DRIVER_TYPE_HARDWARE, NULL, 0,
                      {0xa000}, 1, 7, &device, &level, &context)

— one feature level, `0xa000` = `D3D_FEATURE_LEVEL_10_0`, `SDKVersion` 7 —
and boxes on any negative HRESULT (`d3dgl+0x24183` is the instruction after
the error call, which is where the backtrace lands). So the requirement is
precise and small: **one D3D11 device at feature level 10_0.**

Two free translators can answer that call. Neither stock one does:

- **wined3d** asks `winemac.drv` for a 3.2+ GL context and upstream Wine
  refuses on macOS — *"OS X only supports forward-compatible 3.2+
  contexts"*, then *"None of the requested D3D feature levels is supported
  on this GPU with the current shader backend"*. `MaxVersionGL` (a DWORD of
  `(major<<16)|minor` under `HKCU\Software\Wine\Direct3D`) only changes
  which version it is refused for: at the default it also complains
  *"Profile version 4.4 not supported"*, and at `0x40001` it stops
  complaining and still fails.
- **stock DXVK** (2.7) skips the GPU outright: *"Found device: Apple M4 Max
  … Skipping: Device does not support required feature 'geometryShader'"*,
  then *"No adapters found"*. Apple's GPUs have no geometry shaders.
- **DXVK-macOS** — Gcenx's fork of DXVK 1.10.3, the build Whisky used —
  drops that requirement and answers *"D3D11CoreCreateDevice: Using feature
  level D3D_FEATURE_LEVEL_10_0"*. That is the level `d3dgl` asks for, and
  the game draws.

`tools/gamelog/prefix.sh` prepares the prefix — the DLLs, and the
`C:\users\crossover` symlink every CrossOver-era ini path needs (x32 `d3d11`, `dxgi`, `d3d10core` into
the prefix's `syswow64`; nothing enters this repo), and
`tools/gamelog/winelaunch.sh` is the single launch line every capture script
now sources. Since the eighth Fable pass (2026-09-21, parked 446) that line
also holds **the lane lock**: `ron_wine` writes the launched game's pid to
`$RON_WINEPREFIX/.lane.lock` and refuses, with exit 75 and the holder's
name, to launch while that pid is alive — the one case the human protocol
(astra asks, Ramon relays, the commander holds) could not cover. The lock
releases itself when the game exits; `RON_LANE_FORCE=1` overrides it for a
human who knows the other game is theirs to kill; a launch that does not go
through `ron_wine` is not covered.

### What runs, established by running it

- **The prefix boots and the paths need no re-plumbing.**
  `WINEPREFIX=~/wine-ron`, and its `AppData\Roaming\Microsoft Games` is a
  symlink at the old bottle's, so `rise.ini`, `gamelog.ini` and `Logs/` are
  the same files `setlog.py`, `longtrace.sh` and `rondata::diff` already know.
- **The stock executable reaches the main menu**, fullscreen, 1920x1080,
  `VK_FORMAT_B8G8R8A8_UNORM`, exclusive. `profile_log.txt` runs the whole
  way: `BIGHUGE_INIT`, `INIT_GRAPHICS (pass -1)`, `D3D11GL::INIT_DISPLAY_
  QUICK`, both later passes, `Types::load_sound_tables`,
  `Game::init_common_data`.
- **A launch needs a GUI session, whatever the renderer says.** From a
  process with none, `winemac.drv` refuses the window and the run dies in
  3.8 s at `a2c457` with a `nodrv_CreateWindow` line that reads exactly like
  a DXVK failure: "The click-free lane needs a window", below.
- **The traced executable runs too — with `cover=0`.** The
  `7BF21139` page fault predicted here as possibly downstream of the DirectX
  failure is **not**: it survives the renderer being fixed, reproduces at the
  same address, and disappears the moment `rontrace.cfg` says `cover=0`. So
  the fault is in the int3 forest's **VEH dispatch**, exactly as the
  `cover=0` diagnostic was written to decide, and the draw-site half of a
  capture — which is what the *word* is computed from — needs only the
  trampolines. **Function coverage is the price**, and it is what
  `tools/trace/report.py … blind` reads, so the blind-reading queue stops
  shrinking until VEH is fixed.

### The proof: run903, and it is the same game

A 400-frame capture was driven end to end on this stack — perm probe, launch,
the three lobby clicks at the 1920x1080 table, `!ffwd`, the per-frame dump,
`!quit`, settle, archive. It came back **MAP_STYLE 14, seed 12345, 401 frame
blocks, 66 MB**, and both oracles agree it is the *same game* the paid stack
produced:

    rngcmp.py  rontrace-run53.log rontrace-run903.log
      -> differing frames: 0, identical frames: 401
    samegame.py gamelog-run10-world6-long.txt gamelog-run903-wineproof.txt
      -> 400 frames in common, differ: 0

So the runner is invisible to the simulation, which is the only property the
lane actually needs. **Every capture on disk stays comparable to every capture
taken from here on.**

**Two things had to be fixed to get there, and both were silent.**

- **`$0` inside a zsh function is the function's name.** `lobby.sh` is
  *sourced*, so `${0:A:h}/focus.sh` inside `lobby_click` resolved against
  nothing, and under `set -e` the capture died at the first click having
  printed no error at all. Each script now captures `RON_TOOLS=${0:A:h}` at
  load time, where `$0` is still the file.
- **`C:\users\crossover` has to exist in the prefix.** Every ini in the
  install carries absolute Windows paths written under CrossOver, whose
  prefix user was `crossover` — `gamelog.ini`'s `LogFile=` above all. Free
  Wine's user is the macOS one, so the logger opened nothing and **said
  nothing**: run903's first attempt played its 400 frames, quit cleanly, and
  produced no `gamelog.txt`. A capture that looks perfect and writes no dump
  is what a wrong path looks like here. `tools/gamelog/prefix.sh` makes the
  symlink.

### The corpus moved too, and that was the last thing CrossOver owned

The runner was free before the *data* was. 21 GB of capture corpus — every
`gamelog-run*.txt` and `rontrace-run*.log` the diffs are pinned against —
still lived inside the expired bottle, at
`…/CrossOver/Bottles/ron/drive_c/users/crossover/`, and 37 files in this
repo named that path. **An uninstall would have taken the archive with
it.** It now lives at `~/ron-data`, the prefix's `AppData` points straight
there, and a symlink is left at the old location so nothing that still
names it breaks. Re-verified the way everything else here is:
`samegame.py` run904 against run10 is 400/400, `rngcmp.py` against run53
401/401, and the 207-test diff suite reads its dumps through the new path.

`$RON_GAMELOG_DIR` still overrides, as it always did. The shim at the
CrossOver path went with CrossOver, the same afternoon; nothing read it.

**And then the corpus got a second copy** (steering, Fable, the same day).
The move had left one copy of 21 GB on a disk 95 % full with no Time
Machine destination — the floors in `rondata::diff` are assertions against
files a dead SSD deletes, and the long human-driven runs cannot be re-taken
by script. `tools/gamelog/backup.sh` mirrors the corpus, the recordings
under `~/Documents/My Games` and `~/ghidra-projects` into a private R2
bucket (`ron-data`; credentials from passage at run time, nothing on disk).
`sync` after a capture session pushes what is new; `restore` is the
new-machine line. The bucket is insurance, not the working copy: nothing in
the lane reads it. `~/ron-data` was also pruned to `AppData/Roaming` — the
moved folder had carried Wine's `Documents`/`Downloads`/… symlinks into the
real home, which a backup following links would have uploaded.

### What is not established

- ~~**Why VEH dispatch faults.** `7BF21139` sits between kernel32 and ntdll in
  Wine's own DLL region.~~ It is neither kernel32 nor ntdll and it is not the
  handler: it is `wow64cpu.dll+0x1139`, and the answer is "226: the fault is
  the bop, not the handler" below.
- **Whether DXVK-macOS's 1.10.3 lineage costs anything in fidelity.** It
  renders the menu; nothing says the in-game frame is identical to
  CrossOver's, and the diff is against the *logger*, not the picture — so
  this is a risk to the driven captures (a button in a different place),
  not to the dumps.

### The clock nobody controls

**Rosetta 2 ends with macOS 28, autumn 2027**, with a carve-out for
unmaintained games; macOS 26.4 already warns on launch. Free Wine on macOS
has no announced ARM64EC equivalent. So the durable answer is still
**getting the oracle off macOS**: any x86 machine runs the original
natively, with no translation layer, no licence, and captures faster than
the 3 frames/second `UNITS=3` costs here.

## 226: the fault is the bop, not the handler (2026-09-04, capture lane)

**Superseded on 2026-09-08 by "Coverage is back", below.** The measurements
here stand; two of the readings do not. The fault is not "the thread that
went through the exception path": it is another thread's mode switch,
broken by this thread writing code or running `popad`/`popfd`, and the
falsifier's ROUNDS loop was not the loop its source described. `cover=1`
runs again on this machine.

**Three probes, no lobby drive, six seconds each.** The fault lands during
startup — after `Game::init_common_data`, before the window — so the
discriminator needs no screen at all: set `rontrace.cfg` to
`window=0-3` + `cover=1`, launch under `WINEDEBUG=+seh` with stderr to a
file, and wait for the process to die.

**The three bits the queue asked for.**

- **(a) Did `arm_all` complete?** **Yes.** `INFO armed 0xffffffff 0xbc64` —
  48,228 of the 48,233 entries planted, the five hook sites skipped. Arming
  is not the problem: `VirtualProtect` over the 7 MB of `.text` succeeds and
  the 0xCC writes all land.
- **(b) Did the VEH ever complete a breakpoint?** **Not in the shipped
  build** — zero `HIT` records. But the handler *is* reached: `+seh` shows
  `call_vectored_handlers calling handler at 77FB2C70 code=80000003`, with
  no matching "returned". It faults inside itself.
- **(c) Is the `Eip` convention different here?** **No** — zero `DECLINED`
  records, ever. Wine delivers `ExceptionRecord->ExceptionAddress` and
  `Eip` **both equal to the int3's own address**, which is the first of the
  two conventions `veh` already knows. That whole branch of the hypothesis
  is dead.

**What `7BF21139` actually is.** `WINEDEBUG=+loaddll` names the module:
`wow64cpu.dll` loads at `0x7BF20000`, so the address is `wow64cpu+0x1139`.
That module holds the two 32→64 **bop** entries — `+0x1110` for a syscall,
`+0x1214` for a `__wine_unix_call` — and both begin

    xchg  rsp, r14              ; 4c 87 f4
    mov   [r13+0x9c], edi       ; the 32-bit context, saved
    …
    mov   edx, [rip+0x4ecd]     ; 8b 15 cd 4e 00 00   <- +0x1139
                                ;   (+0x123d in the other, disp 0x4dc9)

**The reported faulting address is the displacement.** `8b 15 …` is
RIP-relative in 64-bit mode and **absolute** in 32-bit mode, so a thread that
arrives at the bop entry *without the mode switch having happened* reads
address `0x00004ECD` — exactly what Wine reports, and `0x00004DC9` for the
other entry. So the fault is one thing and one thing only: **the 32-bit
thread entered the 64-bit bop still in 32-bit mode**, and ran the 64-bit
prologue as 32-bit code until the first RIP-relative operand.

**It is not the handler's Win32 calls either, though those made it worse.**
`veh` called `FlushInstructionCache` and `GetCurrentThreadId` before
emitting; the first is a syscall, and it was the *first* bop attempt after
the exception path was entered — hence the death inside the handler. With
both removed (the thread id comes out of `fs:[0x24]` and the `HIT` record is
buffered like every other), the run gets strictly further: **two
breakpoints handled, `handler at 77FB2C70 returned ffffffff` both times, the
byte restored and `Eip` rewound** — and then the game dies on its *own* next
bop, at `wow64cpu+0x123d`, with a 32-bit `esp` and the game's own registers
live. So the handler is correct, and the broken thing outlives it.

**It is not a volume effect, and the two breakpoints are not special ones.**
The first version of this section said "once a thread has been through the
32-bit vectored-exception path, its next 32→64 transition does not switch
mode", which is **too broad and was falsified within the hour** — see the
reproducer below. What survives is narrower and measured:

- The two breakpoints the shipped list reaches are `WinMainCRTStartup` and
  `__security_init_cookie` — the exe's first two functions. So the game dies
  a few instructions into its own entry point, and nothing about the
  simulation is involved.
- Truncating `rontrace.funcs` to the 38,664 entries at RVA ≥ 0x180000 moves
  the two breakpoints to entirely different functions and changes nothing
  else: **two continues, then the same fault, at the same instruction, with
  the same `eax`/`ebx`/`ecx`/`edx` and the same `esp = 0x7ffc2000`.** So
  arming fewer functions does not buy coverage back, and a cited-functions-
  only list would not either.
- The bop the game dies on is the **unix-call** one (`+0x1214`), not the
  syscall one — the syscall entry is where it died when `veh` still called
  `FlushInstructionCache`.

**The reproducer, and what it costs to defend a claim.** `tools/trace/
wow64bop.c` is 3,584 bytes: one vectored handler, one `int 3` on a function
of its own, a continue, and a `WriteFile` afterwards. It reproduces the
game's fault exactly — `7BF21139`, read of `0x00004ECD` — **12 runs out of
12** across four builds. And a one-difference variant of the same program,
the version committed an hour earlier, **passes 5 runs out of 5**: same
handler, same single continue, same `WriteFile`, only a different shape
around the call. Wine's C cannot see a difference between those two
programs. **A JIT that translates 32-bit code can**, which is where the
suspicion now points — the bop is eight bytes of patched 32-bit in `ntdll`
(`0x7BC0E0C4`, which every `Nt*` stub `call`s through) reaching a call gate
`BTCpuProcessInit` installs, and on this machine those eight bytes run under
Rosetta's 32-bit translation. That is an inference from layout sensitivity,
not proof, and the falsifier below is still what decides it.

**What this costs and what it does not.** `cover=0` remains the floor for
captures: the trampolines are plain jumps, the draw stream and the trace
word are untouched, and `rngcmp.py` still pins every capture to run53.
What stays blocked is `report.py … blind` — one third of `docs/DECISIONS.md`
entry 29's counter 2 — until this is settled or the oracle moves to a
machine that runs 32-bit x86 natively ("The clock nobody controls", below,
which is the same answer for a second reason now).

**run905 is the proof that the instrument did not move.** `tracer.c` changed,
so the rebuilt `rontrace.dll` owes the same evidence run903 gave: a 400-frame
Great Lakes `cover=0` capture, driven end to end, came back **MAP_STYLE 14,
seed 12345, 401 frame blocks, 66,459,736 bytes**, and

    rngcmp.py  rontrace-run53.log rontrace-run905.log
      -> differing frames: 0, identical frames: 401

`veh` is never registered with `cover=0`, so the change is structurally
invisible to a capture — and this says so rather than assuming it. Every
capture on disk stays comparable to every capture taken from here on.

**What is not established**, and it is two things now, not one.

- ~~**Wine's call-gate setup or Rosetta's translation of the far transfer.**~~
  Settled on this machine alone: the two-thread shapes of "Coverage is
  back" separate the instruction classes that break the switch from the
  ones that do not, and the tracer routes around them. Whether the same
  program passes on an x86 host is still unmeasured, and no longer needed.
- ~~**Why one shape of the same program faults and another does not.**~~
  Because the shapes were not what the source said: the ROUNDS loop's
  call was hoisted by the compiler, so `-DROUNDS=8` was eight plain
  stores interleaved with eight syscalls and `-DROUNDS=1` was one — the
  single-threaded form of the finding ("Coverage is back").

## The shutdown dump — 65 archives of free ground truth, and its one-tick tear (2026-09-07)

`GameLog::end_game` writes a **whole-map state on the way out of every game
that is quit rather than killed**. It costs no capture, no window and no
`DUMP_ALL`; it is already on disk, in almost every archive, and until item 241
nothing had parsed one. This is the sweep of all of them.

### The census: 65 of 95

`cargo test -p rondata the_census_is_the_corpus` is the table, one row per
archive, and it is asserted rather than reported:

| shape | archives | who reads it |
| --- | --- | --- |
| **sibling** of the last `FRAME n`, at `FRAME`'s own indent | **65** | `Log::final_state`, and nothing before it |
| **child** of that block — the quit landed before the indent popped | 25 | `Log::frame_states`, all along |
| a `DUMP_ALL` run's trailing `FULL DUMP` | 4 | `Log::dumps`, all along |
| no closing state at all | 1 (run1) | — |

A rough grep for "object records after the last `FRAME` line" answers **92 of
92** and is the wrong number: it counts the `FULL DUMP` and the nested shapes
too, and the gap between 92 and 65 is exactly the population those two make
up. Only the reader can say.

**Which shape a run lands in is not a property of the capture's settings.**
run81 and run82 are the same recipe on the same map and both are siblings;
run83 through run86 are the same recipe again and all four are nested. It is
where inside the frame the quit fired.

### It is frame `n`, and the label is not a guess

Two independent checks, and they have to be two, because within one file both
records are written at the same instant:

- **Within a file.** 36 archives carry a closing dump *and* an ordinary
  `FRAME n` body at the same `n`. On all 36 the two agree on **every unit's
  position and every unit's flags** — 110 unit-moves inside those frames, so
  it is not a corpus of armies standing still
  (`a_closing_dump_and_its_own_block_are_one_state`). That is consistency.
- **Across captures.** run72 quit at 4811 with its window long closed, so its
  closing dump is the only record it has of that frame; run71 is the same
  Great Lakes game running past it, dumping every frame. **27 of the 28 units
  agree.** The twenty-eighth, `0/3`, sits on run71's **4810** position, and on
  4811 run71 has it (−18, +18) further on
  (`the_closing_dump_lags_one_tick_on_one_unit`). That is accuracy.

So the closing dump is frame `n` for almost everything, and one tick behind
for a unit whose update had not run when the quit fired.

### The tear is decidable, which is what makes the dump scoreable

run68 saw the same lag from the other side — `0/5`, one unit, one tick, on two
captures — and concluded that *the harness scores no unit position in a
closing block*. It can. A torn unit is on the simulation's own `n − 1`
position, so stepping to `n − 1` and taking the last frame inside the
comparison separates the dump's tear from a divergence that is ours.
`rondata::diff::compare_shutdown` is that, and `ShutdownResult` keeps the two
apart: `torn` and `off`.

It has to be decided rather than excepted, because **it is a different unit
every time**: `0/3` on run72 and run82, `0/4` on run64, `0/5` on run56, run57
and run68's pair, nothing at all on the other nine. An exception list would be
a list of accidents.

It had already cost an assertion. `run82_s_window_is_the_east_indies_ride_s_run_up`
landed the night before with `0/3` booked as a gap of (−18, 18) this crate
did not have; it is the tear, and the row is gone.

### What the sweep bought, and what it did not

Fourteen closing dumps are now diffed whole, every unit of every record, on
the two scored maps:

| map | frames | new? |
| --- | --- | --- |
| Great Lakes | 2001, 3001, **4811**, 5001, **5591** | 4811 is 6 frames past run72's last block; 5591 is 12 past run73's |
| East Indies | 1501, 3001, 3584, 4001, 5201, 5401, 6181, 6221, **6816**, **6946** | 6816 is 17 past run81's; 6946 is 17 past run82's |

All fourteen reproduce. The only residue anywhere in them is run82's `1/13`
(item 253) and its `1/19` unpack constant.

**No word moved, and none could have.** Great Lakes' long word is 7176 and
East Indies' is 7529; the furthest closing dump below either is 6946. Every
archive that quit *above* a word — run77, run78, run85, run86 — is the nested
shape, so `frame_states` has had its state all along. What the sweep bought is
**coverage under the word**, which is worth having precisely because a draw
stream that matches is not a position that matches: item 250 bought 182 frames
of exactly matching draws on a destination that was still two tiles wrong.

**Setup is the whole game in a sweep like this.** The same nine East Indies
archives read 18 divergences apiece against a generic sibling set and **zero**
against run38's start block with the pasture borrowed from their own trace.
A sweep's first numbers are a finder, not a finding.

### What is still on the table

Ten archives carry a closing dump and **no ordinary block anywhere in the
file** — the long traces, whose per-frame categories were off. Among them are
whole-map states at frame **24001** on both scored maps (run53 Great Lakes,
run21/run23/run54 East Indies) and at 15105, 15401, 16007 and 16489 on East
Indies. Those are the finish line's own frames, already on disk, free. They
are 8,000 frames past where either map holds today, so they are an endpoint
oracle rather than a next item — but when a map's word reaches them, the
record is waiting.

**Taken, 2026-09-07 (item 258).** Both 24001s and two of the four East Indies
rungs are diffed and pinned; the next section is what they say, and why the
other two rungs are not there. The endpoint became a scoreboard line without
waiting for the word to reach it.

## The finish line's own frame — 24,001 diffed whole, and the four families (2026-09-07)

`rondata::diff::endpoint` walks the simulation to the frame each long capture
quit on and compares the whole closing state against it — every unit, every
building, every city. The two counts are the third scoreboard line, pinned
exactly and asserted in no direction — the ratchet 258 booked was reverted
the same evening (DECISIONS 36; the 09-01 rule, assert up to the word and
print past it, stands):

| map | capture | compared | **off** | **unlinked** | extra | torn | builds unlinked/diverged | cities |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| East Indies | run54 | 89 | **79** | **0** | 10 | 1 | 0 / 30 | 3 / 0 |
| Great Lakes | run53 | 80 | **73** | **7** | 0 | 0 | 0 / 14 | 3 / 0 |

The two maps are wrong in different shapes, which is the first thing the
number says that no word could. East Indies **has every unit the original
has** at 24,001 and eleven it should not, and it has 80 of the 89 standing
somewhere else. Great Lakes is missing seven and has invented none. Both are
8,000 frames past their word, so this is a measure of how far a diverged run
wanders, not of a defect — but it is a *coordinate* measure, and the queue has
never had one at the end of a game.

**The ladder underneath**, on East Indies, is 45 off at 15,401 (run28) and 51
at 16,489 (run24) against 80 at 24,001: a **slope**, not a plateau. The last
7,500 frames cost about as much again as the first 15,000.

### The archives are four games, not two

The ladder's rungs are *not* rungs of the endpoint's own run, and the way that
was found is `Trace::frames` — `game_random`'s word at every `do_frame` entry,
which `tools/gamelog/rngcmp.py` compares and
`the_endpoint_captures_are_four_families` now asserts:

| family | captures | identical over | closing dumps |
| --- | --- | --- | --- |
| Great Lakes | run53, run18a | all 24,001 frames | 24,001 (both) |
| East Indies A | run54, run21, run23 | all 24,001 frames | 24,001 (all three) |
| East Indies B | run24, run25, run26, run27 | 16,007 | 16,489, 16,007 |
| East Indies C | run28, run29 | 15,105 | 15,401, 15,105 |

The three East Indies families are **one game to 12,000 and part on 12,001**;
B and C stay together to 15,020 and part on **15,021**. So walking run54's
simulation to 15,401 and reading run28's closing dump there compares two
different games — a mistake that produces a plausible number rather than an
error. Each rung is its own walk from its own capture's start block.

What splits them at 12,001 is not established. A round number in a run that
was driven suggests a click or a window opening; that is a guess.

The families also buy the endpoint its own cross-check: run21's and run23's
closing dumps at 24,001 are asserted to be run54's, unit for unit, and
run18a's to be run53's. That is stronger than item 252's within-file pair —
those two records are written at the same instant, and these are four separate
runs of the same game.

### The setup trap, and it cost the first number

`borrow_from_siblings` gates the world cells and the heights on the sibling's
own world scalars, but takes **`regions`, `herds`, `goods` and `farms` from
the first sibling that has them, whatever map it is**. run54's own start block
carries cells and none of those four, so offering it the Great Lakes sibling
set rebuilt East Indies on **Great Lakes' 13 herds and 36 goods** and produced
a completely plausible 65 units off. The correct sibling — run38, which the
East Indies tests have used since item 252 — gives 41 and 66, and 80 off.

A setup defect reads exactly like a fidelity number. `check_setup` now asserts
both halves per row: every list non-empty after the borrow, and `(herds,
goods)` equal to the map's own pair. Without a sibling at all, run28 walks a
region-less world to **12 units compared of 71** with its AI leader 24,789
units out; that is what the first half catches.

### Why 15,105 and 16,007 are not on the ladder

They live only in the 250 MB window captures (run29, run25/26/27), which
`Log::parse` reads whole — a quarter-gigabyte parse to reach one record, which
is the cost item 260 exists to remove. The queue records run29's 15,105 as 51
off and 22 unlinked; the rungs go in when the parser goes lazy.

## The falsifier for 226, costed — and it turned out to be 3.5 KB

**Overtaken on 2026-09-08**: no second host was needed. The same file,
given a second thread, reproduced the game's death on this machine and
named the cause ("Coverage is back", below). The plan here is kept as the
record of what it would have cost.

The open question above is one bit: **Wine's wow64, or this machine's 32-bit
x86 emulation?** Nothing on this Mac separates them, because there is only
one 32-bit executor here. The falsifier is therefore a second host — and the
first thing to establish was how much of the oracle has to travel with it.

**The answer is: none of it.** The mechanism needs one vectored handler, one
`int 3` and one syscall afterwards. The game, the install, the renderer and
the window are all incidental, and `tools/trace/wow64bop.c` is that and
nothing else: a **3,584-byte** 32-bit console PE importing five kernel32
entries, built by `tools/trace/wow64bop.sh` from the same clang /
`llvm-dlltool` / `rust-lld` toolchain `build.sh` already uses. It prints
phase A (the output channel works), plants the breakpoint, continues from it
in a vectored handler, and then prints phase C — and phase C is the syscall
that dies here. Its header says how to read the three outcomes.

**And it is a validated falsifier, not a hopeful one.** It was run here
first, where the answer is known, and it reproduces the game's fault
exactly — `7BF21139`, read of `0x00004ECD` — in **12 runs out of 12**. A
falsifier that had not been made to fail would have been worth nothing, and
this one nearly was: its first shape *passed* five runs out of five, which
is how the over-broad verdict above got caught (see "the reproducer" there).
`PASS` from the shipped shape on another host therefore means something.

So the costed plan, cheapest first:

**1. `wow64bop.exe` on an x86_64 Linux box with Wine — free, minutes.**
Copy one 3.5 KB file. No install, no `rontrace.funcs`, no ini, no profile,
no prefix beyond a default one, **and no display**: it is a console
subsystem binary that touches kernel32 only, so there is no `user32`, no
X connection and no `DISPLAY` to arrange. Match the version to keep it a
single-variable test — this machine is **WineHQ Stable 11.0**, which WineHQ
also packages for Debian, Ubuntu and Fedora — and run

    WINEPREFIX=/tmp/bop wine wow64bop.exe

`PASS` means a 32-bit vectored handler can continue an `int 3` there and the
next syscall still switches mode, which puts the fault on Rosetta; the same
`Unhandled page fault … at address <wow64cpu+0x1139>` means it is Wine's,
and the report goes upstream with this binary attached.

**2. Only if the small one disagrees with the game: the game itself.**
This is the expensive path and it is not the first move. It needs the
install — **2.7 GB whole**, or roughly 1 GB once `conquest`, `scenario`,
`credits` and `_CommonRedist` are left behind — plus
`riseofnations_trace.exe`, `rontrace.dll`, `rontrace.funcs`, a
`rontrace.cfg` of `window=0-3` / `cover=1`, and `check.ini`. **It does need
a display**, unlike the small probe: the fault lands after
`Game::init_common_data`, and `INIT_GRAPHICS` precedes that, so the D3D11
device has to come up first — the MoltenVK banner is in the stderr of every
faulting run here, ahead of the fault. On Linux that is easy and free
(Mesa's GL satisfies wined3d, or lavapipe satisfies DXVK; `Xvfb` is enough
of a display), so DXVK-macOS is a macOS problem only. Then

    WINEDEBUG=+seh wine <install>/riseofnations_trace.exe -config check.ini -automation

and `report.py rontrace.log summary` decides it: a non-zero `HIT` count is
the answer.

**QEMU TCG on this Mac would do it too, and is the honest fallback.**
`qemu-system-x86_64` (Homebrew, free) emulates x86_64 on Apple Silicon in
software, and **that is precisely why it is a discriminator**: TCG
implements 32-bit protected mode and the far transfer through the call gate
itself, with no Rosetta in the path. A minimal Debian guest with i386
multiarch and WineHQ 11.0 is ~6 GB of disk and an hour or two of downloading,
and after that `wow64bop.exe` runs in well under a minute even at TCG's
speed, because it does almost nothing. The one caveat: a QEMU run changes
*two* things at once — the executor and the Wine build — so pin the Wine
version to 11.0 there, or a `PASS` is ambiguous. The game probe under TCG is
possible but not worth it: software Vulkan under software x86 for a
2.7 GB install is hours, and the small probe answers the same question.

None of this is booked. It is Ramon's spend and Ramon's machine time, and
nothing here has been rented, downloaded or installed.

## Coverage is back — the stub forest, and what actually broke the bop (2026-09-08, tools lane, Fable 5.1)

`cover=1` runs under free Wine again. run906 is the first coverage capture
off CrossOver: run903's game, 401 frame blocks, **10,950 HIT records over
6,733 functions**, a per-frame set for each of the four window frames, and
the word identical to run53 on every one of the 401 frames, and
`game_random` drawing **120, 54, 6, 6** times on frames 0–3, run14's own
verification numbers — the instrument records without moving the
simulation. run907, `cover=0` with
the rebuilt DLL, is the proof the instrument owes after any change:
rngcmp against run53 **0 differing, 401 identical**; samegame against
run10 **400 in common, 0 differ**. Five minutes each, end to end.

**What the coverage instrument is now.** `funcs.py` reads the executable
as well as the index and writes a table: for every listed entry, the
smallest run of whole instructions covering the five bytes a `jmp rel32`
overwrites, with every EIP-relative branch in it rewritten to its rel32
form and its target left for the DLL to fix up. `rontrace.dll` builds one
stub per entry in a region it writes as data and then makes executable,
and plants the jmps **once, at attach, with one thread alive** — nothing is
ever restored, no exception is ever raised. A stub records the entry (the
first per run; inside the window, the first per frame), runs the copy and
jumps back. Outside a window a recorded function's stub is a flag test and
a branch. The stub saves `eax`, `ecx`, `edx` by hand and the flags through
`lahf`/`seto`, and it does so for a measured reason (below). The table
excludes **47** of the 48,233 entries, each named in
`rontrace.funcs.excluded.txt` beside it: 7 whose next entry is under five
bytes away, 13 with a direct branch from elsewhere landing inside the
displaced range (found by a linear sweep of `.text`; the run without this
check died on exactly one, `FUN_004ec042`), and Ghidra's 27 unnamed
`FUN_` chunks, which are labels rather than functions. The five hook sites
and, when a `callwin` is set, the eight proxied sites carry their own
jumps and are skipped as before.

**How the cause was found, because the falsifier had been wrong twice.**

- The shipped `wow64bop.c` loop was not its source. At `-O1` clang hoists
  the pure `target(41)` out of the ROUNDS loop, so the 12/12 fault was
  "eight plain stores to a translated page, each followed by a syscall,
  then one breakpoint", and the 5/5 pass at `ROUNDS=1` was one store and
  one syscall. Read off the disassembly; the recorded numbers stand, their
  explanation did not.
- Single-threaded, every stub shape passes: locked or plain writes,
  batched or interleaved with syscalls, one page or 1,024, a write across
  a page boundary, an `OutputDebugString` exception first, `EFLAGS.ID`
  set, the syscall from inside the stub. The first page-count matrix ran
  with its flag never defined — zsh does not word-split `${X:+-DA -DB}` —
  and was re-run.
- The game died on a single armed function (the entry point, the cookie
  init, the 100th, 1,000th and 2,500th startup function alone), and did not
  die with attach and arming but no hit, nor with a hit that restored and
  logged nothing. Eleven startup probes, 25 seconds each, no display.
- With a flusher thread added so records survived, the next death was on a
  thread that had run no stub at all: a Concurrency-runtime worker,
  `wow64cpu+0x123d`, and then the flusher itself.
- **The two-thread falsifier reproduces it, and the control does not.** A
  second thread that only makes syscalls, beside a main thread running the
  rounds: with writes and `popad`/`popfd`, dies; with no writes after the
  arm, dies; with no flag ops either, intermittent. With no stub at all —
  both threads storming syscalls — **survives 4/4**. Then one instruction
  class per build, a million times: **`popad` dies 3/3, `popfd` dies now
  and then**; an indirect call into the RWX page, `lahf`/`seto` and
  `sahf`, `push`/`pop` and a direct call **survive 3/3**. The victim is
  the *other* thread: it enters the bop still in 32-bit mode (read of
  `0x4ECD`), or runs 32-bit ntdll as 64-bit code (`rip=0xC7BC628D4`).

So the rule the instrument is built on is three lines: no write to the
executable's code after attach; no `popad`, no `popfd`, anywhere a stub
runs; no syscall from a stub (the records are buffered and the flusher and
the frame hook write them). The two probes that put the last line in: a
`WriteFile` from inside the stub, right after a locked restore, died at
the next bop 4 times out of 4, while the same restore followed by the
game's own syscalls lived.

**What it costs.** Every call of every function passes through its stub.
run906's 400 frames took the same five minutes run907's did — startup and
the dump dominate at this length — but a 24,000-frame coverage run is not
free the way a `cover=0` one now is; budget it. The re-arm per window frame
is gone: a window costs a `memset` of 48 KB at its first frame and the C
callback once per function per frame inside it.

**What is not established.**

- **Why `popad` and a code write on one thread break another thread's
  mode switch.** The instruction classes are measured, the mechanism is
  not; it lives in Rosetta's 32-bit support for Wine's wow64 and is not
  ours to fix. The draw hooks still use `pushad`/`popad` and
  `pushfd`/`popfd` at a few thousand calls a frame; ninety captures have
  not tripped on it, the coverage stubs at millions of calls a second did
  within seconds. Moving the hooks onto the same flag-free sequence is a
  one-change item that owes its own run905-shaped proof.
- **A branch target inside a displaced range that the sweep cannot see** —
  a jump-table entry. The 13 the sweep found were direct branches; an
  indirect one into the first five bytes of a listed function would run a
  displacement as code. No run has shown one; the signature is a fault in
  game code at `entry+n` for small `n`.
- **Whether the falsifier's int3 shape passes on an x86 host.** It was the
  question the previous section costed; the answer is no longer needed
  and nothing was rented to get it.

The blind list, counter 2 of `docs/DECISIONS.md` entry 29, re-read on the
three coverage traces there are: **802 functions cited under `docs/`, 650
entered by run53, run54 and run906, 152 never** — run906 alone enters
511, and adds one function the two CrossOver runs never reached.
`report.py … blind docs/ <logs>` is the command, and it accepts as many
logs as there are.

## The click-free lane needs a window, and it is not DXVK (2026-09-18, item 363)

`tools/explore/unattended_capture.py` needs no TCC grant and no human at the
menu, which is what makes it the golden record's lane (`docs/DECISIONS.md`
entry 41 §5). It does need a **GUI session**, and a launch from inside Claude
Code's own process tree does not have one.

**The failure reads like a renderer failure and is not one.** Launched
directly, the run dies in **3.8 s with exit 5, zero frames**, a `rontrace.log`
whose only interesting record is `INFO 176 0xa2c457` — the teardown-fault
address the lab's autostart note already names — and, buried a hundred lines
under MoltenVK's banner in `wine.log`:

```
wine: Unhandled page fault on read access to 00000000 at address 00A2C457
err:winediag:nodrv_CreateWindow Application tried to create a window, but no
                                driver could be loaded.
err:winediag:nodrv_CreateWindow L"The graphics driver is missing. Check your build!"
```

MoltenVK enumerates the M4 Max and creates its `VkInstance` **before** this, so
every DXVK-shaped diagnostic looks healthy: the tempting reading is a broken
prefix or a bad d3d11 override, and both are wrong. `winemac.drv` is what
refuses, because the process has no window server connection; `a2c457` is the
game dereferencing the window it never got.

**The fix is the one the clicked lane already uses.** `viadriver.sh` launches
through LaunchServices, which makes `RonDriver.app` the responsible process and
gives the run a real GUI session — and `tools/explore/golden_capture.sh` is the
shim that hands that launcher a Python runner:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh \
    ~/ron-golden/<name> --map 14 --end-frame 900 \
    --cmd-file tools/gamelog/golden/chapter1.cmd \
    --log-window 605 900 --detail end:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2,DEATHS=1
```

So the click-free lane's independence is from **TCC and the cursor**, not from
the desktop: it still cannot run on a box with no login session, and a lane
that launches it any other way gets a fault that will be misread as DXVK.

**`--timeout` truncates rather than stops**, and it defaults to 180 s — the
same trap `tools/gamelog/captures.txt` documents for `poll_max`. run106's first
attempt came back with 593 of 1,900 blocks in a 62 MB file that reads as
entirely ordinary; only `receipt.json` says `success: false`. Size it from the
dumping (run106 wrote 3.6 blocks a second), and read the receipt before the
dump.

**The window, the detail and the command file are the caller's** since this
item (parked 341 closes here). `live_session.stage` took none of them: it
hardwired `LogStartFrame 18` / `LogEndFrame 36` and forced every `gamelog.ini`
category to 0 but `[End Frame] UNITS=3` and `[Misc Logging] COMMANDMANAGER=1`.
`--log-window`, `--detail` (setlog.py's `SECTION:CAT[=N],...`, repeatable),
`--cover`, `--cmd-file` and `--ffwd-minute` now reach it, each staged value is
echoed into the run's `receipt.json` so a capture is read back from the run
rather than from the command that asked for it, and a category the ini does not
have is **refused** — the failure that guard catches is a correctly-numbered
window whose blocks come back empty because `end:UNIT=3` was a typo.

## A packet is a capture too (2026-09-23, the eleventh Fable pass)

PR #7 merged the lab's `RON_STATE_FRAME` tracer build: two hooks, at
`GameLog::end_frame` and at the `do_frame` continuation after it, copy the
process's private data at one frame boundary into a `frame-snapshot-v1`
stream beside the dump (`tools/explore/frame_snapshot.py` stages it and
demands the receipt, both hooks and the logger-return check). Three
rules for the lane: it is a **build variant**, mutually exclusive with the
restore probe by `#error`, so a packet run is its own launch; it is a
**second game on this lane**, holding the lane lock like any capture and
serialized with the loop's own runs; and the frame it is taken on is
~~**the one before the divergence**~~ **the word's own logger frame**,
because the packet at frame N is the state after tick N−1's decision, so
tick N — the word's — is still ahead of it (item 597, the twelfth pass;
the worked case is `docs/EMULATOR.md` §8). Costs and what a packet establishes are
`docs/EMULATOR.md` §8; the evidence is `docs/lab/TYPED-STATE-REVIEW.md`.
