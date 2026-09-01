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

## The attrition run (run16, 2026-08-24) — the first of the blind runs

The first run taken off the blind list. Same lobby and seed as run12–14
(frame-0 word `0x3bd39ae9`), the traced exe with `cover=1` and no window,
`[End Frame] UNITS=3 GUYS=1 DEATHS=1 LEADERS=3 BUILDS=1 CITIES=1 MISC=1`,
`[Start Game] WORLD=0`. Driven by an Opus agent from a written brief, one
hour of wall clock for 6,872 frames; the predictions were written before
the log was read and every one of them was observed (`docs/ATTRITION.md`
and `docs/SUPPLY.md`, their last sections; `tools/gamelog/attr.py` is the
reader). The archive is `gamelog-run16-attrition.txt` (1.0 GB) and
`rontrace-run16.log` (93 MB). The scenario, in cheat lines:

```
peace who=1                    diplos[1] 0 → 1 at label 341
add hoplite who=0 206,78       three records o 6–8, hits 120, on the AI's ground
add supply who=0 208,80        o 9, hits 90 — bleeds at 8 too
add scout who=0 204,76         takes nothing
war who=1                      diplos[1] → 0 at 2417; periods → 0 at each refresh
tech who=1 allegiance on       (landed on the fourth try — see below)
add hoplite who=1 42,146       the AI's squad on Nubian ground: attrition 0
tech who=0 allegiance on       → 48 within four frames
add hoplite who=0 206,78       → 24 (Allegiance + Oath of Fealty on who 1)
move 10 190,60                 → 0 at the next refresh
add supply who=0 209,91        the shelter: last tick 6278, wagon at 6289
```

**What the run corrected in this document and the recipe:**

- **`move` takes no `,who`** (the table above is fixed); `die` does.
- **`LEADERS=3` is the minimum that prints `diplos[]`** — `LeaderData::log_data`
  sets detail 3 just before the array — so a diplomacy read-back needs it;
  `LEADERS=1` stops after `score`. `att`/`anti_att` sit at detail 7 and
  the census at 9, which crawls; the tech grants have **no read-back at any
  level** (`tech … show` prints into the closed console window), so a
  tech's landing is verified through its effect on a unit's `attrition`.
- **`[End Frame] MISC=1` is what emits `BEGIN FRAME n`**; zero it and
  `gl.py frames`/`lastframe.py` have nothing to count. Keep it.
- **The chat box drops about four lines in ten.** Eleven of ~19 landed
  first time; one line needed four tries and one never arrived in three.
  Two modal stalls (the city rename dialog from a stray `Return`, and an
  empty chat box left open) each froze the sim for a minute — Cancel and
  Escape respectively. This is the case for the scripted cheat channel
  below.
- **A squad left inside the enemy's borders at war walks off to fight**
  (no order given) and is dead within ~2,000 frames; two squads were lost
  that way. A long observation wants peace, or the far corner of the
  enemy's territory.
- **Object numbers are recycled** from the lowest free slot: a dead
  squad's 6–8 went to the next scout (6) and the next squad (7, 8, 10 —
  9 being the wagon's until it died). Read a unit's history as
  (kind, who, o) *and* its `myhits`/damage continuity.
- The route out is the top-right HUD icon → Game Menu → Quit Game;
  Escape closes the chat box and clears the selection but does not open
  the menu here.
- Frames 10 and 11 draw 226 and 254 times on `game_random`: 220 + 248 of
  them are `PathFinder::calc_road_cost` under `astar_caravan_road` <
  `find_road` — the game planning a caravan road on the sim's stream, a
  per-frame source `docs/SYNC.md` had not seen (its §6).

**The blind list after run16:** 424 cited, **99 never run** (from 156),
with the whole of attrition and supply, combat's arithmetic, target
selection and the AI's C++ producers now entered by at least one trace.
Still blind as groups: the order commands other than move/gather/build and
their `Group::action_*`, the scenario host functions, ships and aircraft,
the hero-generals.

**The next improvement is the loop itself, not the mechanic.** An hour of
an agent typing into a chat box that drops lines is the cost of every
blind run, and it is avoidable: `rontrace.dll` already trampolines
`Game::do_frame`, `ConsoleWin::parse_cmd@007d6470(this, String *, int
from_chat, int no_mouse)` is what the chat box calls, `MiscAccess::console_win`
is the pointer at VA `0xE7FA84` (PDB `0003:2595460`), and
`String::String(wchar_t *)@00a1edd0` builds a const string without the
heap. A `rontrace.cmd` of `frame: line` entries run at the top of the frame
— inside the tick, not through the order stream, and while paused — makes
a run reproducible to the frame and unattended; a scheduled `quit`
(console-only half, `from_chat = 0`) closes it through the menu's path.
~~That is the next thing built.~~ **Built and validated the same night —
next section.**

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
give `x,y`.

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
order list. **No console command issues an order at all**: the chat half is
a set of state pokes. This closes the open question from run17 ("whether
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
channel by construction**. Those need the real order stream, which is the
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

## The combat run (run17, 2026-08-24) — the channel's first real run

Fourteen `rontrace.cmd` lines, no driver, 2,600 frames in fourteen
minutes: six duels on unowned mid-map land with `!ai off` at frame 100.
What it found is `docs/COMBAT.md` §16 — every unit-on-unit hit in the run
is one of the sizes the formula predicts, with the flank sectors confirmed
by damage; the run itself is in `docs/JOURNAL.md` under 2026-08-29,
"Lifted from ORACLE.md".

**Five staging facts from it, used by every scenario since.**

- A cheat-placed unit faces `0x55555555` = 120° (clockwise from north, y
  south) until it is ordered, so bearings are precomputed:
  `dx = 4 sin b`, `dy = −4 cos b`.
- **A Supply Wagon flees on sight**, so it is a one-hit target unless the
  attacker spawns within striking distance.
- **`!ai off` stops the leader's strategy, not the buildings' queues** —
  the AI kept training citizens after it.
- `add`'s `find_nearby_spot` can land a unit **eight tiles** from the
  asked one, and an object number cannot be predicted from a file, so give
  each trial its own spot rather than a `die` line.
- `move` from the channel is a **teleport** (`Unit::set_new_location`),
  not an order — the question of why `move 6 190,60` "did not work" was
  the wrong shape ("The channel's vocabulary", above).
## The producers' run (run18, 2026-08-25) — the script ends, the C++ takes over

The blind list's third entry: *a long game past the script, at `LEADERS=9`
around a sweep*, which `docs/AI.md` §12.1 item 4 had been asking for since
the producers were implemented — **no dump had ever shown a non-empty make
list**, because the script blocks the C++ steps for the whole opening.

Two stages, both this lobby and seed (frame-0 word `0x3bd39ae9`), both
driven entirely from `rontrace.cmd` with three lobby clicks:

- **run18a** — `5 !ffwd 30`, `24000 !quit`; `cover=1`, no trace window, and
  **`LogStartFrame=0 LogEndFrame=0` in `rise2.ini`**, whose gate `0 <= f < 0`
  is never true, so the per-frame dump is off entirely while the
  start-of-game dump still lands. 24,000 sim-frames, 1.6 MB of gamelog,
  26 MB of trace, **~45 seconds of game time**.
- **run18b** — the same game with `[End Frame] LEADERS=9 UNITS=3 BUILDS=7
  CITIES=5 GUYS=1 DEATHS=1 MISC=1` and the window `[6374, 6590)`, i.e. the
  sweep the script dies on *and* the next one; `cover=1 window=6374-6590`.
  217 blocks, 360 MB, ~9 minutes.

**What a run costs, measured** (and the reason the earlier estimates in
this section were wrong by an order of magnitude — they were the author's
wall clock, not the game's). Fast-forwarding with the dump off runs at
**~500 sim-frames a second**, about 35× real time: run18a's 24,000 frames
are 26.7 minutes of gameplay and took three quarters of a minute. A dump
block at `LEADERS=9 UNITS=3 BUILDS=7 CITIES=5 GUYS=1` costs **~2.4 seconds
and ~1.7 MB**, stable across runs (217 blocks in ~9 min, 19 in ~45 s). So
**a run's cost is `blocks × 2.4 s` and everything else rounds to zero** —
budget the window, not the frames, and put the frames you do not need
behind `!ffwd`.

The window's own semantics are run13's exactly — inclusive start, exclusive
end (run19 asked `[8174, 8192)` and got 8174…8191) — but **`!quit` emits one
or two ungated blocks of its own**, which is why run18a's "dump off" log is
not empty but holds frames 24000 and 24001.

**`ffwd` is the lever the combat run wanted.** `ConsoleWin::run_cmd`'s
`ffwd` case sets `game->fast_forward_frame = minute × 900` (bare `ffwd`
toggles 9,999,999); `TurnControl::check_new_frame_solo` skips the
wall-clock wait while it is non-zero and `Game::loop_render` draws one frame
in sixteen, clearing it once `fast_forward_frame <= frame`. It is a
*presentation* switch — nothing in the sim reads it — and with the per-frame
dump gated off it turns "3 frames a second" into **~500**. **The speed floor
was never the input or the renderer; it is the dump.** Window the dump and
fast-forward the rest.

**What the run establishes**, all of it in `docs/AI.md` §15:

- **The script ends at sim-frame 6376**, from `defensive`'s `case 29`:
  steps 28 (tower) and 29 both run in the one call, `research_tech_with_cost
  (who, "Classical Age")` queues the age — `num_queued` gains 544 while
  `ages_get()` is still 0 — and the call returns `SCRIPT_DONE`. Not the hang
  guard, and not the attacked-city bail-out.
- **The two ladders**, frame for frame in the dump's `production_step`:
  `1, 2, 3, 4, 5, 6, 7, 8, 0` over 6375…6383 on the sweep the script dies
  on, and `1, 3, 4, 5, 6, 7, 8, 0` over 6575…6582 on the next — one frame
  shorter, because step 1 entered with `prod_script_run` already 0 is
  promoted to 2 *and runs it* in the same call. Both are pinned
  (`ai_drive.rs`), and the first-entry frames in the trace agree:
  `production_ai_setup`/`market_speculation` 6377, `research_techs` 6379,
  `upgrade_units` 6380, `create_units` 6381, `create_buildings` 6382,
  `make_stuff` 6383, `produce_unit` 6582.
- **The make list is a ranked four plus seven category slots.** The dump
  shows entries at slots 0, 5 and 8 and nowhere else — `PEASANTS cat 5`,
  `TEMPLE cat 8` — which is `MakeList::make_me`'s tail writing each object
  into `list[cat]`, over a top-four insertion at slots 0–3. A new best
  **overwrites** slot 0 without shifting the old head down, so the citizen's
  rank-0 copy is simply lost when the temple outbids it and survives only in
  slot 5. `MakeList::clear()` at step 2 empties all eleven — visible at 6577.
- **Five expiry draws, and every one of them decides by `% 3 == 0`.** The
  trace records the seed before each `Random::get`, the dump records which
  slots survived, and the two together settle the arm `docs/AI.md` §2.6's
  prose had backwards: an ordinary building takes the **probabilistic**
  arm, not the unconditional one. Two sites, both new to the documents —
  `make_stuff+0x221` is the head's walk, `make_stuff+0x63d` the bought
  slot's.
- **`val /= 100` on a buy, observed**: the citizen bought at 6582 goes
  `714 → 7` in the same block and stays in the list.
- **The easy-difficulty stockpile clamp fires**: `bucket` 261/104/107 →
  37/43/77 across the setup step, on this Easiest lobby (`d = 0`,
  `m × 3/2`), with `econ`/`rate`/`worst_good` written there for the first
  time in the game.
- **A bought *slot* is not a bought *head*.** 6582 buys the citizen out of
  slot 5 and still disarms to step 0 — the second pass 9–11 is the head's
  alone.

**The blind list after run18: 424 cited, 89 never run** (from 95). The six
retired are the long game's own — `Army::find_target`, `Unit::unpack_merchant`,
`LeaderData::calc_rare`, `get_general_upgrade`, `HeroData::get_radius`,
`BuildTypeData::max_knowledge_gatherers`. What stays blind as groups: the
order commands other than move/gather/build and their `Group::action_*`,
the command processors, ships and aircraft, the scenario host functions
(`enable_production_ai` and the `disable_*` trio are scenario-only and
cannot be reached from a skirmish at all).

`tools/gamelog/steps.py` is the instrument — one block per frame with the
step fields, the goods picture, the queue, the eleven `MAKEOBJECT`s by slot
and the ten sites, `--terse` comparing only the step machine's own fields so
the ledger's per-frame tick does not print every block.

~~**Open:** `make_stuff` was reached twice and bought once; `produce_tech`
first runs at 8182 and `found_cities`' own purchases at 576, both outside
the window, so `research_techs`' and `found_cities`' *outputs* are still
unscored. A window around 8182 is the next one, and it is now cheap.~~

**run19 is that window, the same day** (`gamelog-run19-window-8174-8192.txt`,
`rontrace-run19.log`; `docs/AI.md` §15.6). One stage, `!ffwd 9` to frame
8100 and `[8174, 8192)` — 19 blocks, 32 MB, **a minute and a half of game
time**, which is the recipe above paying for itself. By 8182 the AI is in the Classical Age with a full make
list, and it shows what run18b structurally could not: the **second pass**
(steps 9, 10, 11, with `make_stuff` at step 8 called from
`production_ai+0x1fa` and at step 11 from **`+0x236`**), two purchases in
one `make_stuff` (`produce_tech` for the head and `produce_unit` for slot
1, both demoted 9,999,999 → 99,999), `research_techs` reaching the
**`9,999,999` overflow guard** in play and taking **no draw at all**, the
runners-up shifting through slots 1–3 with one falling off the end, and —
the best of them — a **scholar kept at a non-head slot and cleared at the
head on the same `% 3` residue**, which isolates the unconditional arm from
the probabilistic one on a single type. Nine expiry observations across the
two runs, nine agreeing.

**Open:** `found_cities`' own purchases are still unscored — they happen at
frame 576, inside the script's era, so the window that catches them also
catches the script calling `place_city_with_cost`, and the two callers have
to be told apart by the step in the dump.

## run20 and run21 — the islands map (2026-08-25)

The first runs off the profile's Great Lakes, and the first `DUMP_ALL`
capture read as the harness's own oracle rather than as a sibling.

**run20** (`gamelog-run20-islands-dumpall.txt`, 278 MB, `rontrace-run20.
log`): East Indies picked in the lobby's combo (`MAP_STYLE 18`, the WORLD
block's `map 18`, `sea_map 4`), run7's seed, and every logger at once —
`DUMP_ALL=1` with `InitialDump=1` (the start-of-game full dump: cells at
every level, the tile masks, the fog grids, `master_land_heights`, the
regions, the herds, the type tables), `check_all_level=14` with `[Misc
Logging] CHECKSUM=2` (the setup trace), `LogStartFrame=0 LogEndFrame=4`
(four `DUMP_ALL` frame blocks, `FRAME 1`–`4`), `rontrace.cfg` `cover=1
window=0-3`, and `rontrace.cmd` a single `4 !quit`. About seven minutes
wall clock: four for the start dump, ~110 s a frame block, and the quit's
own block. What that buys: a capture whose `Initial` carries its own
`checksums`, `heights`, `herds` and `frame_seeds`, so `build_sim` seeds
the personality, the stride and the slide from the run itself and the
`SITES` diff runs with no sibling at all — the shape every future
behavioural capture should take when the frames wanted are few.

**run21** (`gamelog-run21-islands-long.txt`, 1.7 MB, `rontrace-run21.log`,
29 MB): the same lobby, `DUMP_ALL=0`, `LogStartFrame=LogEndFrame=0`
(dump off), `cover=1`, `5 !ffwd 30` and `24000 !quit` — run18a's recipe on
the islands. Two minutes including the load. It is the trace that reaches
the AI's sea half (`docs/AI.md` §15.8: `check_transport` at 201, the docks
at 3579, `Army::do_transporting` at 14586).

**What the two runs settled** is in `docs/AI.md` §15.8 and the audit's
fourth pass: `world+0x34` is the style's `SEA_MAP` class and not a
landmass count; `land_key[]` is the static `BASELAND, SANDY, OCEAN, NONE`;
`is_ocean` is by cell kind; `was_seen` has a territory arm that the fog
grid alone does not explain; and B4-k's guard — the ten-record `SITES`
diff on run20 — passes and fails on demand.

**Three driving facts, each of which cost a relaunch or a wrong note:**

- **The traced process is `riseofnations_trace.exe`**, and
  `tools/gamelog/waitwin.sh` waited for `riseofnations.exe` forever; it
  matches both now.
- **`!quit` returns to the Game Over screen and the process stays up**;
  under `DUMP_ALL` the end-of-game dump keeps writing for a minute after
  it. Wait for `gamelog.txt` to stop growing, then kill.
- **Save to Profile does not survive a killed process** — see "The lobby
  is a file" above. The combo is read from a screenshot every launch.

## run22 — the first dock, under the window (2026-08-25)

The run21 lobby (East Indies from the combo, seed 12345, the profile's
Nubians) with the per-frame dump **gated to the frames that matter**: run21's
trace put `Dock::init` at sim-frame 3579, so `rise2.ini` `LogStartFrame=3579
LogEndFrame=3582`, `DUMP_ALL=1`, `InitialDump=1`, `[Start Game] WORLD=6`,
`rontrace.cfg` `cover=1 window=3579-3581`, `rontrace.cmd` `5 !ffwd 30` /
`3583 !quit`. **Eight minutes wall clock** end to end
(`gamelog-run22-islands-dock-window.txt`, 249 MB; `rontrace-run22.log`):
the start dump, the fast-forward to 3579 in seconds, three blocks of ~80 MB
at about two minutes each, and the quit's own two (3583, 3584). The staging
is one script, **`tools/gamelog/window.py stage 3579 3582`** / `restore`:
every ini edit, the cfg and the cmd in one call, and the reverse.

**What it holds.** Block 3579 (the end of sim-frame 3578): no active
`DOCK`, none of the AI's 14 units with `unit_masks & 0x800000`. Block 3580:
`DOCK dock 0, o 2010, reg 65, gull_o 15, who 1, dock_flags 1`; the building
`o 2010` of leader 1 at `(44160, 41856)`, `orig_type 432`; all 14 AI units
with the bit, the human's 6 without. (A `FRAME n` block under `DUMP_ALL`
carries two `FULL DUMP`s — the end of `n − 1` and the start of `n` — so a
count over the whole block doubles; the harness reads the first.) The
trace's frame 3579 repeats
run21's to the seed — draws 0 and 1 the gull's creation and
`Dock::init+0x125`. `docs/TRANSPORT.md` §10, §12; the assertion is
`rondata::diff::tests::run22_s_first_dock…`.

**Two things it settled that the reading had wrong or missing.** The dock's
`reg` is the **sea** (65): a dock's centre cell is water, so `Dock::init`'s
`reg < 0x40` guard never counts it into `reg_docks`. And **owner 9 is not
in a `DUMP_ALL` frame block**: the 208 `ANIMALDATA` records of block 3580
are all leader 8's; the gull (`gull_o 15`, who 9) is nowhere in the dump,
so its position and heading are the trace's to attest, not the log's.

**Driving notes.** `cliclick m:X,Y w:400 c:X,Y` fired every button this
run; the press-and-hold form (`dd`/`du`) registered as a hover on the
first Solo Game click. The window sat at `(760, 152)` again; the lobby
came up on the profile's Great Lakes and the combo pick was read back as
`MAP_STYLE 18` in the log's first lines. `!quit` from the cmd file left
the process at the Game Over screen writing blocks 3583–3584 for a minute;
a size poll on `gamelog.txt` (90 s unchanged) is the "done" signal, then
`pkill -f riseofnations_trace.exe`.

## runs 23–31 — the army's and the group's captures (2026-08-25/26)

Nine runs of the run21 lobby, staged from `docs/ARMY.md` §16 and
`docs/GROUPS.md` §11 and driven unattended by `tools/gamelog/runwin.sh N LO
HI TAG`. The story is in `docs/JOURNAL.md` under 2026-08-29, "Lifted from
ORACLE.md"; what belongs here is the inventory and the traps.

| run | file | what is in it |
| --- | --- | --- |
| 23 | `gamelog-run23-islands-war.txt` | a null result worth keeping: a Quick Battle **already starts at war**, so the `war` line changed nothing and all 24,001 per-frame words are run21's. `tools/gamelog/rngcmp.py A B` is the ten-second check that a scenario took at all |
| 24 | `gamelog-run24-islands-raid.txt` | the first traced game with combat in it: seven hoplites beside the AI's capital, which falls at 13125, and the AI is defeated at 16488 |
| 25–27 | `…-emergency-`, `…-findtarget-`, `…-defending-window.txt` | `DUMP_ALL` windows of run24's game at [12129, 12132), [12024, 12027) and [15100, 15103) — `Armies::emergency`, `find_target` with a live enemy, `do_defending` |
| 28 | `gamelog-run28-islands-engagement.txt` | `cover=1` with no dump at all: `Army::engagement@006f5160` entered at **15100**, the frame's coverage naming the chain down to `Group::action_attack`. Three minutes — the cheapest behavioural check there is |
| 29 | `gamelog-run29-islands-engagement-window.txt` | the same scenario windowed: its `ARMY` half is `docs/ARMY.md` §17 and its `UNITS=3` half `docs/GROUPS.md` §6.4 |
| 30 | `gamelog-run30-humangroup-nogroups.txt` | `GROUPS=1` under `[End Frame]` only, so **no `GROUPDATA` came out**; kept as the first dump holding `GroupMoveOrder` blocks |
| 31 | `gamelog-run31-humangroup.txt` | three human right-clicks on twelve selected units: forty frames carrying a `GroupMoveOrder` and a live group (`docs/GROUPS.md` §11, §12.1) |

**Four facts from them that every later run uses.**

- **`select` is scriptable, so a group capture needs one human click and
  not twelve.** `ConsoleWin::run_cmd`'s `select` case takes
  `[[ob#|type] [who] [+]]`: with a *type* it walks every object of that
  player and adds each match to its `SelectGroup`, and the trailing `+`
  suppresses the clear. `160 select slinger who=0` then `170 select
  hoplite who=0 +` puts twelve portraits in the tray. (The old note that
  `+` "is not reliable" was the **chat box** dropping lines, not the
  command.)
- **`rontrace.cmd` clamps a frame lower than the previous line's to it**,
  so a `!quit` written after a later-frame `add` runs at the *later*
  frame. Write the file in ascending frame order.
- **`!quit` at `HI + 1` does not end the process**: the game returns to the
  Game Over screen and runs on at fast-forward until it is killed, so a
  windowed run has more blocks than its window — the extra is a free
  capture of a later frame, labelled by the frame it was written at.
- **The East Indies lobby at seed 12345** (`MAP_STYLE 18`) starts the human
  on tiles 26–30 × 25–42 and the AI on 198–214 × 201–214.
  `tools/gamelog/live.sh` launches and **returns with the game running**,
  where `runwin.sh` waits for a `!quit`; `archive.sh N TAG` is the other
  end. At ~370 KB a frame the game runs about one frame every two seconds,
  which is what leaves a driver wall clock between frames.
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

## run32 — the road on fresh ground, and the heights of another game (2026-08-28)

The capture item 55 asked for, and it settled the item twice over.

**The recipe is `tools/gamelog/roadcapture.sh`**, end to end: it probes the
three macOS permissions, puts run10–14's game back (seed 12345 in `rise.ini`,
map style 14 "Great Lakes" in `check.ini` **and** in `PlayerProfile/Player.dat`
— the `<MULTI>` block is the one Quick Battle reads), stages the `DUMP_ALL`
window `[104, 109)` and a `rontrace.cmd` of four lines, drives the lobby and
archives. Nine minutes, 366 MB, five frame blocks.

```
5 !ffwd 30
100 add granary who=0 6,171
100 add smelter who=0 33,161
110 !quit
```

**The channel can place a finished building, and it plans its road there and
then.** `run_cmd`'s `add`, for a type index past the units, calls
`Objects::init_build` and then the object's vtable slot `+0x1a8`, which
`vtables.txt` names `Build::activate`; `Build::init` calls `find_city` on the
way in, so the building joins the human's city exactly as a built one does.
What no reading had noticed is what happens next: `Wall::start@0063e810`
passes **`REGEN_FORCE`** to `mask_me`, and `BuildType::mask_me`'s tail is
`place_roads` — so the ring and the road go down at *start*, and
`City::regen_roads`' flag is what makes it happen again later. Frame 100's
first 2,913 draws are `calc_road_cost`'s, before phase 1.

**The two searches are separable, and not by counting.** `Wall::activate`
plays a sound off a *different* generator, and the trace records every
generator's draws in file order — so the one non-sync record inside frame
100 splits the road draws into the Granary's **1,043** and the Smelter's
**1,870**. (Looking a word up in the trace does not: every draw's seed is
just the LCG advanced that far, so "where does our word appear" always
answers "at our own count".)

**The sites were chosen against the simulation.** Every tile 10–20 from p0's
centre whose whole footprint carries `CITY_RADIUS`, for every type that
`connects_to_roads` and that `place_building` accepts — the city already has
a Library and a Market, so those are refused as one-per-city and the
Granary, the Lumber Mill and the Smelter are not. It is worth doing: a
site that does not bind to a city plans no road at all.

**What it produced.**

| what | where |
| --- | --- |
| the fresh road, tile for tile | 62 tiles; `run32_s_two_fresh_roads_are_the_original_s_tile_for_tile` |
| four scheduled replans, node for node | 332, 231, 232, 60; `run32_s_scheduled_replans_cost_the_original_s_nodes` |
| the terraform's before and after | run13's `FRAME 100` heights against run32's `FRAME 104`: 128 corners, in the two footprints' boxes and nowhere else |
| the search reads the **pre**-terraform grid | 1,046 nodes against the original's 1,043 on the pre-terraform table, 967 on the post |

**And the thing it was not looking for.** The harness had been feeding the
road search `master_land_heights` from **run3** — the same seed, style and
size as run10–14 but `GAME_RULES 0` rather than 1, so its starting buildings
terraformed different ground and its grid differs on 237 corners spread from
(7, 83) to (230, 163). `borrow_from_siblings` took the first sibling with a
height table and run3 is first in the list. With the map's own heights
(run12's and run13's, which agree exactly), run14's frames 10 and 11 cost
**220** and **248** nodes — the original's, exactly, where they had been 208
and 222. That, not anything in the search, was item 55's six per cent.

## run33 — the long trace, and the proof that the instrument is free (2026-08-29)

run14 traced 284 of run10's 1,772 frames, and once the simulation's word
matched all 284 there was nothing on disk that could say where the two next
parted by *site*. run33 is the replacement: **run10's own game, traced,
1,850 frames**, and the recipe is `tools/gamelog/longtrace.sh` end to end —
the three permissions, `mapstyle.py` putting style 14 in `check.ini` and the
profile, seed 12345, run10's exact detail —

```
[Start Game] WORLD=6 TERRAIN=2 GOODS=3 UNITS=3 BUILDS=7 CITIES=5 GUYS=2 LEADERS=9 DEATHS=1
[End Frame]  MISC=1 UNITS=3 BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1
```

— `rise2.ini`'s frame window `[0, 1900)`, `rontrace.cfg` `cover=1` with no
window, and a two-line `rontrace.cmd` (`5 !ffwd 30`, `1850 !quit`).
**Fourteen minutes**, 284 MB of dump and 9.5 MB of trace, unattended.

**It is the same game, and that is now a measurement rather than a hope.**
`tools/gamelog/samegame.py` reduces each `BEGIN FRAME` block to a digest of
its indented lines and compares two dumps frame for frame; run33 against
run10 is **1,771 blocks in common and not one that differs**. So the traced
executable, the `int 3` on all 48,233 function entries, the cheat channel and
`!ffwd 30` are all invisible to the simulation over 1,771 frames — where
run18a had checked four — and run33 inherits run10's siblings, its
`build_sim` and its tests. (Calibrated on run10 against run14's gamelog:
284 identical blocks, the only difference being run14's truncated 285th.)

**What it says.** The simulation's per-frame draw **count** is the
original's through frame 306 and parts at **307**, twenty-three frames past
where run14's capture ran out. The original spends eleven draws there and
this simulation nine, and the two missing are one unit's non-flat gather —
a stand issued from `Unit::do_non_flat_gather+0x10f`, then `+0x54b`, the
gather's own roll. The site fires nineteen times in the whole run — 1, 168,
204, **307**, 465, 508, 565, 687, 780, 983, 985, 1030, 1091, 1393, 1498,
1544, 1590, 1643, 1781 — and this simulation makes the first three and
misses the fourth. Over the whole 1,850, **668 frames spend the
original's number of draws and 556 are its draws in its order**; both are
pinned in `run33_s_long_trace_says_where_the_word_parts`.

**And the thing it did not buy.** The blind list did not move — 617 cited,
**101 never entered**, with run33 in or out — though run33 entered 6,702
functions against run14's 6,585. A long run of the *same* no-input game
lights nothing new: the list is shrunk by scenarios, not by frames.

## run38 and run39 — the second map, and its first score (2026-08-29)

Phase 3's finish line names **two** maps, and every number in the harness
was Great Lakes. run38 and run39 are the other one: **East Indies**
(`MAP_STYLE 18`), seed 12345, and run10's rules otherwise (`MAP_SIZE 2`,
`GAME_RULES 1`, `REVEAL_MAP 1`) — the profile's lobby rather than
`check.ini`'s, which is the whole trick (see "The lobby is a file").
run38 is `tools/gamelog/startcapture.sh`'s `DUMP_ALL` start, two frames
and 151 MB; run39 is `longtrace.sh`'s 1,850 frames, 482 MB and an 11 MB
trace. Their traces' words agree on every frame they share, so they are
one game.

**run38 is the whole sibling list.** Its own `Initial` carries the
heights, the checksum trace, the herds and the frame seeds, so `build_sim`
stands the simulation up on a map it has never seen with nothing borrowed
— and **frame 0 is 175 draws against 175 on the first try**.

**The score, first time of asking: ticks 167, orders 167**; player 0 parts
at 219, player 1 at 168
(`run39_s_islands_game_is_the_second_map_s_score`). Great Lakes stands at
252 the same day, so the residue chased on the one map was not chased into
its shape. ~~What parts it first is an order-list **length**~~ — `1/4`
holds two orders on the original's frame 168 where this simulation holds
one, and its position parts on the same frame; `1/5` at 186 and `1/3` at
202 are the same disagreement — but **that is not what parts it first**,
see the next row.

## run39's trace, read at last — the second map's word (2026-08-30)

run39 shipped with an 11 MB `cover=1` trace and nothing read it: the map
was scored on its dump alone. Read against the harness frame for frame
(`run39_s_long_trace_says_where_the_second_map_s_word_parts`), **the word
parts at 19** — 148 frames before the order-list divergence above, so
run39's 167 is a number on a stream that is nobody's, and the row above is
struck for it.

The cause is the AI's **pasture**, which East Indies has and Great Lakes
does not. `docs/SYNC.md` §3.11 has all of it, including the coin's parity
argument (a pasture is one species, always) and the five position offsets
read back out of the trace's own seeds — **which is the first thing this
harness has taken from a trace rather than from a dump**, because owner 9
appears in no dump block at all.

The score beside the word is the **early window**, not a total: past the
parting a total is noise. Of the first 64 frames, **62 spend the
original's number of draws and 55 draw for draw** (2026-08-30; 49/47
before the pasture landed). What still parts at 19 is the *first* of the
two `Animal::do_idle` draws an arrival costs — movement's and animation's
residue, not the pasture's.

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

## run42 — `LEADERS=2`, and the pile is no longer unchecked

run39's lobby and seed exactly (East Indies, `MAP_STYLE 18`, seed 12345, the
profile's lobby with no `-config`), 900 frames, at run39's detail **plus
`LEADERS=2`**. The `LEADERDATA` block that carries `bucket`, `ages_get()`
and `epoch_get(scan)` is the encrypted one, and `LeaderData::log_data@006e5110`
announces it at detail **2** — the `this_00[1].handle = 2` before the
`LeaderDataEncrypt::log_data` call — not the **9** of the census. That is
the whole reason this capture is cheap: 245 MB and eleven minutes, where
`LEADERS=9` per frame is ten thousand lines a leader a frame and crawls.
`samegame.py --exclude LEADERDATA` against run39 is **900 frames in common
and not one that differs**, so it is run39's game and inherits its
siblings.

**It settles `docs/GOODY.md` §6's owed capture, and corrects one detail of
it.** The prediction was `bucket[2]` stepping by 50 on frame 867 with
`epoch_get(scan)` reading `0 0 0 1`.

- The step is there and it is the AI's: leader **1**'s `bucket[2]` goes
  **50 → 100** between the blocks labelled `FRAME 867` and `FRAME 868` — a
  block `FRAME n` is the end of sim-frame `n − 1`, so the pay lands on
  **sim-frame 867**, the predicted frame.
- **The trace names the cause rather than leaving it to be inferred.**
  `ObjectsData::find_goody_at` and `Unit::explore_goody` are entered on
  sim-frame 867 and on no other frame in the neighbourhood — 860, 863, 865,
  866, 868, 869, 872 and 880 all have neither. A box was opened on exactly
  the frame the pile moved.
- **`epoch_get(scan)` reads `0 1 0 1`, not `0 0 0 1`.** Civic is 1 as well
  as Science. It does not enter the formula — `epoch[3] × 25 + 25` reads
  Science alone, and `1 × 25 + 25 = 50` is the observed pay — so §6's
  arithmetic stands and only its stated vector was wrong.
- So the reading that mattered is **confirmed and its alternative refuted**:
  an `ages` reading would pay 25, because `ages` is 0 in an Ancient-age
  game, and the observed step is 50.

What run42 does **not** settle is which *good* a box picks: the lottery's
winner depends on the finder's buckets, and the frame's draw count would be
identical whichever good won (§6's last row). `bucket[0]` and `bucket[1]`
are visibly a different clock — the human's step by one every eleven and
fifteen frames respectively, all run long — so the record now on disk is
enough to separate the pile's income from the box's, which it was not
before.

## run43 — the terraform's own before and after, in one game

run32's scenario exactly — the same two enhancers on the same fresh ground
at the same frame, seed 12345 and map style 14 — with the `DUMP_ALL` window
opened four frames earlier: **[100, 108) rather than [104, 109)**. A block
`FRAME n` is the end of sim-frame `n − 1`, so `FRAME 100` is the grid before
either `add` lands and `FRAME 106` the grid after the Smelter's replan on
104 and the Granary's on 105.

**Why the four frames were worth a second capture.** `docs/ROADS.md` §7.1
reads the road search's grid as the pre-terraform one, and it had to reach
into **run13** for that grid — a different game, in which nothing is ever
placed. The two games are identical up to sim-frame 100 by construction, so
the substitution was almost certainly sound; "almost certainly" is what a
capture is for. run32's own window cannot supply it: `heightdiff.py` on
run32's `FRAME 104` against its `FRAME 108` moves **not one corner**, because
both are already post-terraform.

**What run43 says**, `tools/gamelog/heightdiff.py` on its own two frames:

- **128 corners move**, which is §7.1's number, now a single game's own
  difference rather than a cross-game one.
- They fall in **exactly two clusters of 64**, and nothing lies outside
  them: columns 3..10 × rows 168..175, and columns 30..37 × rows 158..165 —
  centres (6.5, 171.5) and (33.5, 161.5), for buildings placed at tiles
  **(6, 171)** and **(33, 161)**. So "the two footprints' boxes and nothing
  else" is exact, and each box is 8 × 8.
- **The corner grid is one corner per tile.** `master_land_heights` is
  `(4·xs + 1)²` for xs = 60 *cells* of four tiles — 58,081 corners, 241 a
  side, spanning 240 tiles. The 4 in the formula is cells-to-tiles, not
  tiles-to-corners, and run43's clusters are what says so: a building at
  tile 6 moves columns 3..10, and at tile 33 columns 30..37.

What this capture does **not** do is re-derive the two short node counts
(1,046 against the original's 1,043, and 1,460 against 1,870). Those are the
harness's arithmetic over the grid, not the dump's; what changes is that the
grid the harness should read them on is now this game's own, at a frame the
same file also carries the placement for.

**A trap this run cost, and the guard that caught it.** run43 was captured
twice. The first archive was 550 MB, the right map style, the right seed,
and a full window — and held **half its scenario**: zsh's `${(j:\n:)a}`
joins with a literal backslash-n rather than a newline, so the stanza's two
`add` lines reached `rontrace.cmd` as one, `ConsoleWin::parse_cmd` took the
Granary, returned 1 and dropped the rest. Nothing in the dump looked wrong.
`cmdsran.py` read the trace's `INFO cmd` records and said "3 lines parsed,
1 at frame 100, expected 2" — which is what it was written for one run
earlier, and it caught the bug on its first real outing. The same join had
silently emptied `rontrace.cfg`'s window line too. The fix is the `p` flag;
the incomplete archive was deleted rather than kept, because a
half-happened scenario that reads as valid is precisely run23's failure
mode.

## run44 — the turn override fires, and `guy_flags` has more writers than §9 has

`docs/ANIM.md` §4.6 calls `Guy::do_turn@005d97a0:15`'s override
unfalsifiable and owes it "a capture with a vehicle or a ship turning in
place". run44 fires it, and it needed no driver — which is a reading, not
luck. `Unit::move_step` passes the override flag on only its two
turn-in-place branches, but it is **not the only caller**: `Guy::move:109`
calls `turn_towards(this, des_angle, _, 1)` on the standing arm, guarded
only by `guy_flags & 2`, and `Guy::turn_towards@005d9720` hands its
argument straight to `do_turn`. So a turner unit **turning towards a
target** fires it, and a fight is enough.

run39's lobby, 700 frames, `GUYS=4` — `cur_anim` sits past the last of
`GuyData::log_data@005de6c0`'s three level announcements, so at run39's
`GUYS=2` a `GUY` block stops after `ox` and the question cannot be asked of
the file. Seven `add` lines from `rontrace.cmd`, all nine records accepted:
catapults and a trebuchet for the AI on the tiles beside its capital,
hoplites, pikemen and a catapult for the human among them.

**452 guy-frames play a turn animation**, in nine distinct
`(who, o, slot)` combinations, both `CHAR_TURN_LEFT` and
`CHAR_TURN_RIGHT`, on **both sides** — the human's pikemen from frame 166
and the AI's own catapults at 247 and 248, which the AI ordered unaided.
Every capture before this one has **zero**: run13, which does carry
clocks, has none in its whole window.

**And the flag byte says why, per type.** `guy_flags` in run44:

| value | bits | records |
| --- | --- | --- |
| 16 | 0x10 | 165,938 |
| 48 | 0x10 0x20 | 7,984 |
| 8 | 0x8 | 4,034 |
| 56 | 0x8 0x10 0x20 | 3,206 |
| 40 | 0x8 0x20 | 1,392 |
| 24 | 0x8 0x10 | 16 |

- **0x8 is exactly the three turner types** — 134 `PIKEMEN`, 265
  `CATAPULT`, 266 `TREBUCHET` — and no others, which is
  `Guy::init_real@005db6b0:179` setting the bit for a guy whose piece names
  a turn, observed rather than read. §4.6's "none of the eight a `DUMP_ALL`
  run's guys carry" is still true of those eight; it was a fact about which
  units had been captured.
- **0x20 is set on 12,582 records and it toggles within a type**: 50
  `PEASANTS` appears as both 16 (14,276) and 48 (1,534), 132 `HOPLITES` as
  16 (30) and 48 (6,450), 134 as 24 and 56, 265 as 8 and 40. So it is
  **state, not a per-piece init bit, and it has a writer.** `docs/ANIM.md`
  §9 lists 0x20 among three bits with "no writer found", says "none of the
  three is exercised", and leaves it off in the sim — where §9 also reads
  it as *collapsing the idle roll*, which is a draw.

  **This does not move either map's score today, and the reason is worth
  stating.** §9's "every guy in both dumps carries `guy_flags 16`" is still
  exactly true of the scored games: run13, which is run10's own game under
  `DUMP_ALL`, has **2,288 records and every one of them 16**, and run38,
  the islands start, has 1,180 and the same. Nothing in either turns 0x20
  on. run44 is simply the first capture that has **combat and guy-level
  detail at once** — the earlier fights (run17, run24) were taken without
  the clocks, and the earlier `GUYS=4` runs have no fight in them. So the
  bit is real, it is reachable, and it is waiting for the sim to arrive at
  the part of the game that turns it on; it is not a divergence in the
  1,850 frames anyone is scoring.
- **0x2 and 0x4 are still unobserved**, and 0x2 is a puzzle rather than an
  absence: `do_turn`'s first statement is
  `*(ushort *)&this->field_0x9a |= 2` whenever the angle actually changes,
  and 452 turn animations means that line ran. Either the dumped
  `guy_flags` is not the whole `ushort` at `+0x9a`, or something clears the
  bit before the frame ends. Unread here, and named rather than guessed.

Also worth keeping: 265 and 266 carry `8` and `40` — **without 0x10**,
which every other type in the file has. Whatever 0x10 is, the siege pieces
do not have it.

## run45 — the AI moves the mirror flag, and never lays a group move order

Item 23's driver-free shot, and a **negative result with a reason**, which
is worth more than the run it cost. run39's lobby, 900 frames, run31's group
detail; `groupfacing.py` over the whole archive:

| | |
| --- | --- |
| `GROUPDATA` blocks | 461,824 — 902 frames × the 512-slot pool |
| `group.facing` | 0 ×461,393, **1 ×431** |
| formations (`GROUPDATA.form`) | none ×460,923, **Line ×901**, nothing else |
| `GroupMoveOrder/MOVEORDER.facing` | **none at all** |
| other `MOVEORDER.facing` | −1 ×1,522, 0 ×699, 1 ×196 |

So the AI **does** form groups and its groups' mirror flag **does** flip —
431 records carry `facing 1`, which is `Unit::set_angle@00605400` toggling
it as a leader turns 90° or more off its heading. What the AI never does,
in 900 frames, is lay a **`GroupMoveOrder`**: not one unit in the file
carries one. `Unit::kill_current_order`'s hand-back reads the dying order's
own `MoveOrder +0x28`, so with no group move order there is nothing to hand
back and the XOR term cannot fire however long the run.

**What that settles.** The trace was right that the machinery runs —
`Group::action_move_near`, `Form::compute` and `GroupData::find_leader` are
all entered at frame 0 of run39 — and it was the wrong question to ask of
it. *Entering* `action_move_near` is not the same as a unit ending the frame
holding a `GroupMoveOrder` the dump can print. Item 23 needs a **human
right-click**, which is what run31 has and what no AI game supplies, and the
capture is run31's three clicks **plus a fourth**: `groupfacing.py` on run31
shows its group move orders carrying `facing 1` from frame 356 to 367 and
the log then closing, so the mirrored order it needs is already made and
simply never dies.

**Across every capture on disk, `GroupMoveOrder` is a human-click
artifact.** Counted rather than argued, over nine archives: run31 has
**945**, and run20, run13, run22, run25, run26, run27, run29 and run45 have
**none** — that set includes three `DUMP_ALL` windows taken *during* the
AI's own fighting (`Armies::emergency`, `find_target`, `do_defending`) and
900 frames of the group pool itself. So it is not that run45 was too short
or too peaceful: no AI in any captured situation has ever ended a frame with
a unit holding one, and run31, the one capture driven by right-clicks, is
the one that has them.

**And the Echelon half needs the mouse twice over.** Every group in run45 is
a **Line**, and every group in run31 is too. `docs/GROUPS.md` §6.4's slot
table only reads `reverse` on the Echelon rows, so the mirror is invisible
in the positions of a Line whatever the flag does. The console's 102
commands, re-derived from the user's own install by
`tools/gamelog/console.py`, contain **no formation verb at all** — the chat
half is `add`, `select`, `move`, `die`, `damage`, `tech` and the diplomacy
pokes — so a formation can only be set through the unit panel.

**A trap this run cost twice, now a check.** The first attempt dumped the
pool **once**, in the start block: 512 `GROUPDATA` records against run31's
111,616. `GROUPS=1` is necessary and not sufficient —
`GroupData::log_data@0045e1d0` calls neither `set_type` nor `set_detail`, so
its lines are accepted against whatever the previous dumper left, and
`dump_deaths@0092fd80` ends by calling `WorldData::log_data` twice, leaving
the type at `WORLD`; with `WORLD=0` under `[End Frame]` the whole pool fails
`check_accept` silently. **`DEATHS` off** under `[End Frame]` is the fix, and
it is written down in this file already — it cost run30 — which is the
argument for a guard over prose. `groupfacing.py` now fails on `≤ 512`
blocks and names the cause, so the next stanza to do it is told in a minute
rather than after a twenty-minute capture.

## run46 — the XOR term fires, and the formula is right

Item 23's event, and the first time `Unit::kill_current_order@005e2cb0`'s

    group.facing = order.facing XOR reversing(leader.angle - order.angle)

has run with `order.facing` **1** in any capture. run10's lobby, 900 frames
at run31's group detail, eight hoplites added and selected from the cheat
channel, and three right-clicks from `clickdriver.sh` — the camera alternating
between tiles (30, 167) and (6, 167) either side of the units, so every order
after the first is a ~180 degree turn and `reversing` is not left to luck.

| click | frame | its order appears | the order's angle | `order.facing` |
| --- | --- | --- | --- | --- |
| 1 | 213 | 216 | **+85.8°** | 0 |
| 2 | 333 | 336 | **−92.8°** | **1** |
| 3 | 453 | 456 | **+86.0°** | 0 |

Read it as two hand-backs, and both come out as the formula says:

- **click 1 → 2.** The leader turns +85.8° to −92.8°, which is 178.6° and
  inside the `reversing` window, so the toggle is 1. `Form::compute` sees
  `order.facing XOR toggle` = `0 XOR 1` = **1**, and click 2's order is laid
  out carrying `facing 1`. That is the mirrored layout run31 also reaches.
- **click 2 → 3, which is the one nobody had.** The dying order carries
  `facing 1`, the leader turns −92.8° to +86.0° — 178.8°, the window again,
  toggle 1 — and the hand-back is `1 XOR 1` = **0**. Click 3's order is laid
  out carrying `facing 0`, which is what the dump prints from frame 456.

The mirrored order lives on frames **336 to 455** and dies on the frame click
3's replaces it; 1,509 group move orders carry `facing 1` across those 120
frames, against **none** in run45 and none in any AI capture.

**What is still owed, and it is the other half of item 23.** Every group here
is a **Line** (`form 0`, 1,587 records, nothing else), and `docs/GROUPS.md`
§6.4's slot table only reads `reverse` on the Echelon rows. So the mirror's
*consequence for the positions* — the formation byte's sign — is still
unexercised: this run proves the flag is computed as stated and not what it
then does to a slot. A formation cannot be set from the console (its 102
commands have no such verb, `tools/gamelog/console.py`), so that half needs
the unit panel, which is a click on a button rather than on the map.

**Two instrument lessons, both of which cost a run.**

- **`!ffwd` stops the renderer.** run46's first attempt clicked three times
  and produced no order at all, and its screenshot showed the capital still
  selected — which read as `select hoplite who=0` having failed. It had not.
  run47's five screenshots, taken across ninety sim frames, came back
  **byte-for-byte identical** with the in-game clock at 00:00:00: the game
  was simulating and not drawing, so the driver was clicking at a picture
  minutes stale and no screenshot of that run was evidence of anything. With
  the fast-forward dropped the shots differ, the clock runs, and `select
  hoplite who=0` puts six hoplite portraits in the tray exactly as the
  recipe above says. `FFWD` is an input now; **every stanza with a `driver:`
  sets it empty.**
- **An accepted line is not a line that did something.** `cmdsran.py` reports
  what `ConsoleWin::parse_cmd` returned, and it returned 1 for the select
  that changed nothing. The tick means the channel took the line, and no
  more.

## run50 — the Echelon half, and the four doors that are shut

Item 23's remaining half, **not** obtained, and the value here is that the
search is now bounded rather than open. `docs/GROUPS.md` §6.4's slot table
reads `reverse` on the **Echelon** rows alone — Refused is `Y = Y0 - |X|`,
with no `reverse` in it, so the queue's "Refused or an Echelon" is really
Echelon only — and every group in every capture on disk, run46's included, is
a **Line**. So the mirror is confirmed as a computed flag (run46) and still
unobserved as a *displacement*.

What was tried, each with its evidence:

- **The console.** Its 102 commands, re-derived from the user's own install
  by `tools/gamelog/console.py`, contain no formation verb of any kind. The
  chat half is `add`, `select`, `move`, `die`, `damage`, `tech`, `resource`,
  the diplomacy pokes, `finish`, `hurry`, `pack`, `deploy` and `anim`.
- **The command card.** run48 photographed it with a group of hoplites
  selected: move, attack, auto-explore, board, stop, garrison, and fourteen
  empty cells. The compass-with-arrows that looked like a formation chooser
  is **Auto Explore** — the tooltip says so, and says its key is CTRL+E.
- **Military research**, the obvious gate, since RoN unlocks formations with
  it and every capture is an Ancient-age nation. run49 raised it (`tech who=0
  all on`, `military 5 0`) and photographed the same six buttons. Not the
  gate.
- **Binding the key.** This is the one that should have worked.
  `data/playerprofile.xml` is the keymap `KeyMap::init@007d5a90` loads, and it
  lists `FORM_LINE`, `FORM_REFUSED`, `FORM_ENVELOP`, `FORM_E_RIGHT` and
  `FORM_E_LEFT` as bindable actions **with no `<INPUT>` child on any of
  them** — the file has zero `<INPUT>` elements in total, so the formations
  ship unbound and that is why neither a key nor a button reaches them. The
  element's shape is fully recovered from
  `KeyMap::save_entry@007d4220` and `KeyMap::load_entry@007d43d0`, with every
  attribute name resolved out of `int_str_array` at stride 0x14 the way
  `console.py` reads it:

      <KEY enum="FORM_E_RIGHT" dependent="-1">
        <INPUT key="120" mouse="0" ctrl="0" shift="0" alt="0"/>
      </KEY>

  `key` is taken whole and then `ctrl` sets bit 0x20000, `shift` 0x10000 and
  `alt` 0x40000. `tools/gamelog/bindkey.py` writes exactly that into the
  **profile's** `<KEYS>` — `Player.dat`, which is user state and already
  edited with the game closed by `mapstyle.py`, never the install's shipped
  data — and `--restore` empties it again. run50 bound `FORM_E_RIGHT` to F9,
  pressed it twice (once with only the selection, once after the first march
  had made a group) and marched the group back and forth. **The formations in
  its dump are `Line x540` and nothing else.** The binding did not take, or
  the keystroke did not reach the action.

**Where the next attempt should start, and it is one question.** Is the
profile's `<KEYS>` read at all? `KeyMap::save_entry` writes an entry only when
its `dependent` is negative, so the profile is meant to hold the player's own
bindings — but the loader that would read them back is `KeyMap::load`'s
`String` overload at `007d39a0`, and **its caller has not been found**; the
only references the export shows outside `KeyMap` itself are unwind funclets.
Settle that and the rest follows: if the profile is read, the binding is
wrong in some detail; if it is not, the shipped `data/playerprofile.xml` is
the only keymap and the `<INPUT>` has to go there instead. The cheap
experiment either way is to rebind an action whose binding is **visible** —
`OPTION_AUTO_EXPLORE`, whose tooltip prints its key — and photograph the
tooltip: if it stops saying CTRL+E, the profile route works.

**What run50 is still good for**, and it is not a plain replication. It
reaches the mirrored layout on its own game — **870** group move orders
carrying `facing 1`, frames **337 to 395** — and then those orders **end with
no successor**: frame 396 holds none at all. So this order died by
*completing*, where run46's died by being *replaced*, and those are
`kill_current_order`'s two different ways in. Whether the hand-back's
arithmetic is the same on the completion path is **not** settled here: the
`GROUPDATA.facing` a frame prints is the whole 512-slot pool's, so the live
group's own value cannot be read off it, and run46's proof worked because the
*next* order's `facing` showed what `Form::compute` had been handed. With no
next order there is nothing to read it from. A capture that wants the
completion path needs a fourth click after the arrival.


## runs 53/54 — the same games, thirteen times as long, for eight minutes and 70 MB

Item 91's captures, and the first two stanzas to use `poll_max:`. One
24,000-frame run per map, the trace whole (`cover=1`) and the `[End Frame]`
detail cut to `MISC` alone, `[Start Game]` left at run10's exactly because
that block is what the harness stands the simulation up from.

| run | map | gamelog | trace | frames |
| --- | --- | --- | --- | --- |
| 53 | Great Lakes (14) | 10 MB | 25 MB | 24,001 |
| 54 | East Indies (18) | 10 MB | — | 24,001 |

**Both are the same games as run33 and run39**, and that is asserted rather
than assumed: `rngcmp.py` compares the `game_random` word of every `FRAME`
record and both pairs come back **0 differing over 1,851 overlapping
frames**. So run53 is a drop-in longer sibling of run10/run33 and run54 of
run38/run39.

**The measurement that matters here is the cost.** run33 is 1,850 frames and
took the better part of an hour; run53 is 24,000 and took **100 seconds**.
The sim is not the bottleneck and never was — unrendered under `!ffwd` it
runs at roughly 240 frames a second — it is the **per-frame dump**, 155 KB a
frame at run33's detail and rising with the roster. Cutting `[End Frame]` to
`MISC` removes the floor entirely. Two consequences worth writing down:

- **A trace-only capture of any length is nearly free.** Where a question is
  about the *stream* rather than a record, there is no reason to take a short
  one.
- **A full-detail dump over 24,000 frames would be hours and gigabytes**, and
  run53 is what says whether it is worth taking. It is not, yet: the word
  parts at **1802** on this capture exactly as it does on run33's 1,850, so a
  full-detail dump buys about thirty frames of new ground past run10's own
  1,772 and then twenty-two thousand frames of a stream that is nobody's.
  Size that capture to the word, and take it when the word has moved.

**The trap this pair found, before it cost anything.** `longtrace.sh`'s poll
loop was `for i in {1..160}` at twenty seconds — **53 minutes** — and its end
is not a graceful stop: it `pkill`s the game and archives whatever has been
written, which is a truncated capture that looks exactly like a finished one.
Every run to 52 fits inside the bound and none had reason to notice. `POLL_MAX`
is a hook now, `poll_max:` a stanza key, the default is unchanged at 160, and
the give-up path says out loud that its archive is truncated. `settle_min`
needed lowering for the same captures: a `MISC`-only per-frame block never
reaches the 10 MB the settle test defaults to, so the run would never have
been called finished.

## runs 51 and 52 — the profile IS read, and the key that answered it opened the chat box

The experiment run50's section asked for, run in two halves, and it closes the
`007d39a0` caller question from the behavioural side without reading another
line of the decompile.

**run51: the profile's `<KEYS>` is read.** `bindkey.py` wrote an `<INPUT>` for
`OPTION_AUTO_EXPLORE` — chosen because it is the one action whose binding is
**visible**, printed in its own tooltip — rebinding it from its shipped
CTRL+E to key 120. Hovering the button then photographs the answer:

| run | the same tooltip |
| --- | --- |
| run48, profile untouched | `Auto Explore: ON - ... (Hotkey: CTRL + E)` |
| run51, after `bindkey.py OPTION_AUTO_EXPLORE 120` | `Auto Explore: OFF - ... (Hotkey: F9)` |

So `KeyMap::load`'s `String` overload at `007d39a0` **does** run, whatever the
export shows about its callers, and `bindkey.py`'s element is right:
`<KEY enum=... dependent="-1"><INPUT key=... mouse=... ctrl=... shift=...
alt=.../></KEY>` in the profile's `<KEYS>`, exactly as
`KeyMap::save_entry@007d4220` writes it. **That is a tool the lane keeps**:
any action in `data/playerprofile.xml` can now be given a key with the game
closed, and the tooltip is how you check the game agrees.

**run52: and the keystroke arrives, but not at the action.** With the same
bind in place, hover, press, hover:

- before — `Auto Explore: OFF - Click to turn on Auto Explore. (Hotkey: F9)`
- `osascript ... key code 101`, the mouse parked off the card
- after — **the chat dialog is open** (`Chat`, `Chat Ally`, `Chat All`,
  `Close`), the game dimmed behind it, and Auto Explore still OFF.

So input reaches the game — something plainly happened — and it did not reach
the bound action. Two readings fit and this run does not separate them: either
F9 also opens chat and chat wins, or **macOS `key code 101` does not arrive as
F9 through CrossOver** and landed as the `Return` that opens the chat box (the
recipe's own table says the chat box is opened by `Return`). The second is the
likelier, and either way the lane's conclusion is the same.

**What this settles for item 23.** run50's `FORM_E_RIGHT`-on-F9 failure is no
longer a mystery and no longer evidence about formations at all: its two key
presses were opening a modal chat box, not asking for an Echelon. The bind
itself was fine. So the Echelon half is **not** blocked on the profile route,
which works; it is blocked on sending a key the game receives as the key it
was bound to.

**One operational trap, found by tripping it.** The game **rewrites
`Player.dat` when it quits**, binding included, so a `--restore` issued while
it is still running is undone a minute later. Restore after the process has
exited — which is where `runqueue.sh` leaves things — or the profile keeps a
binding nobody meant to leave behind.

**How the next attempt should send it.** Bind to a **plain letter** and send
it with `osascript ... keystroke "j"`, which is the path the recipe says
reaches the game, rather than a function key through `key code`. Then verify
before relying on it, in this order, because each step is cheap and the one
after is not: (1) the tooltip says the letter, so the game loaded the bind;
(2) pressing it toggles Auto Explore, so that letter arrives; (3) only then
rebind `FORM_E_RIGHT` to the same letter and look for `form 3` in the dump.
Steps 1 and 2 are a 250-frame run apiece and would have saved run50.

## run55 — the call proxies, and a function's own answer (2026-09-01)

The third instrument's third question. `tools/trace` could say **which
function drew** and **which functions ran**; it could not say **what a
function answered**, and no logger can: the dumps print state, a draw record
prints a seed, an `int 3` prints that something was entered. A function that
computes a number and hands it back leaves nothing behind.

`docs/PATHFINDER.md` §10 had recorded exactly that as a dead end — the
`PATHFINDER` gamelog category emits one line at map generation, the two
`dbg_*` printers are gated on a flag nothing writes, and
`PathFinderData::log_data` needs the `DUMP_ALL=1` that hangs the game. So
item 125's check was written down as an `int 3` on `calc_cost` and left.

**What it wanted was not a breakpoint but a proxy.** `rontrace.cfg`'s new
`callwin=LO-HI` replaces each listed function with a stub of its own
signature that logs the arguments, calls the original through the
displaced-prologue trampoline, and logs `eax`. The arguments live in the
proxy's own frame, so recursion and re-entrancy cost nothing, and the
callee-clean `ret <imm>` is copied from the listing. Two are proxied:
`PathFinder::astar_path@00683770`, whose entry and return **delimit one
search**, and `PathFinder::calc_cost@00684e50`. Without a `callwin` nothing
is patched at all, which is how every earlier capture stays reproducible.
`tools/trace/README.md` has the how-to and the three things a new site needs
from the listing.

**run55 is run39's game**: East Indies, `MAP_STYLE 18`, seed 12345, the
profile's lobby, run39's own `[End Frame]` detail, 1,500 frames, seventeen
minutes and 391 MB — with `cover=0` and `callwin=1460-1490`.
`tools/gamelog/rngcmp.py` says its `game_random` word is run39's on **all
1,501 overlapping frames, zero differing**, so the proxies cost the
simulation nothing and the capture inherits run39's siblings.

**What it found, in one reading.** Over the whole thirty-one-frame window
the game ran **one** search — the AI scout's, 110 `calc_cost` calls on
sim-frame **1476** — which made identification free. 103 of the 110 already
agreed with this crate's own answers for the same arguments. All seven that
did not were steps into the four cells under player 1's second city, and
they said the same thing twice: `+176` on each, and one refusal. `176` is
`20 × 9 − 4`, the terrain term for a cell nine of whose sixteen tiles are
built over, less the own-territory discount — so `WData.blocked` is a
**count of blocked tiles that a building raises**, not a property of the
map, and `World::set_blocked_at@006b4900` is its only writer. With that
kept, the two searches are identical call for call, and East Indies' whole
capture is matched: 1,851 ticks of 1,851, 1,850 order-frames of 1,850, no
player diverging anywhere.

**Two things worth carrying.**

- **The frame label bit again.** Every document before this one called the
  scout's search "frame 1477", from the dump block it lands in. The trace
  counts `Game::frame`, and the search is on **1476**. The first `calls`
  listing came back empty because of it.
- **A proxy is cheaper than it sounds and more general than it looks.** The
  encoding was verified against `llvm-mc --disassemble` before the game was
  ever launched, and a sixty-frame smoke run proved it before the
  seventeen-minute one. Any function whose *answer* is the question — a
  score, a predicate, a chosen index — is now one table row away.

## run56 — East Indies past its own word (2026-09-01)

The capture "The capture lane"'s standing rule owes: **when a map's word
crosses the newest full-detail capture it has, the next one is sized to the
word.** East Indies' word is run54's 2176 and its only full-detail run was
run39's 1,850, so every frame of item 85's divergence fell past the end of
the only dump that could show it.

run39's recipe unchanged and nothing else — East Indies, `MAP_STYLE 18`,
seed 12345, the profile's lobby with no `-config`, run39's `[Start Game]`
and `[End Frame]` detail, no input — carried to **3,000** frames. Thirty-six
minutes, **789 MB** of dump and 12 MB of trace, at about 1.5 sim-frames a
second; the per-frame block runs ~260 KB and rises with the roster. Only the
*length* changed, deliberately: `gatherers`, `gather_down` and
`non_flat_gather` are all already in run39's `BUILDDATA` at `BUILDS=7`, so
raising a category would have bought nothing and cost the sibling-hood that
lets both same-game tools speak.

**It is the same game twice over.** `rngcmp.py` against run54: **3,001
frames, zero differing**. `samegame.py` against run39: 1,850 frames in
common, **zero differing**. So it inherits run39's siblings and run54's word.

**What it settles.** The whole of `Build::find_gather_tiles`
(`docs/ECONOMY.md`, "The gather list, and its shuffle"). Frame 2176 is the
one camp the *game* builds in either capture — player 1's `o 2009`, a
Woodcutter's Camp at tile (198, 190), 48 tiles, `4 × 48 = 192` draws — and
its `BUILDDATA` record is what makes the shuffle's **order** checkable
rather than merely its count. Seed-anchored on the trace's own word at the
entry of 2176, this crate's camp comes back with the original's list entry
for entry.

**And what it named, and how long it lasted.** The gather half of the record
is now compared on every frame — and the *only* rows were that one camp and
the quit's own four. This simulation placed player 1's farm on frame 2, its
second city on 977 and its second farm on 1577, each at the original's own
tile and object number; then the original placed `o 2009` and this one placed
nothing. ~~The successor is an AI build decision.~~ **Closed the same day**
(`docs/AI.md` §19): `produce_building` scored a camp site by the forest tiles
in a one-tile ring rather than by what the site would gather, which is zero at
every site a camp can stand on. With `blocked_site`'s own out-parameter there
the camp goes up on frame 2176 at tile `(198, 190)` as `o 2009`, and run56's
gather comparison is 1,048,118 fields with nothing but the quit's four. East
Indies' long word went 2176 → **2665** on it.

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

## run57 — East Indies at 4,000 frames, and the capture the word did not need (2026-09-01)

run56's successor by the same standing rule that owed run56: East Indies'
word crossed the map's newest full-detail capture again — 2665 → **3021** on
item 128 — so the next one is sized to the word. run39's recipe unchanged and
nothing else (`MAP_STYLE 18`, seed 12345, the profile's lobby with no
`-config`, run39's `[Start Game]` and `[End Frame]` detail, no input),
carried to **4,000**. Forty-eight minutes, **1.07 GB** of dump and 12.8 MB of
trace, at about 1.4 sim-frames a second.

**It is the same game, twice over, and the tools said so before it was read.**
`rngcmp.py` against run54: **4,001 frames, zero differing**. `samegame.py`
against run56: 3,000 frames in common, **zero differing**. So it inherits
run39's siblings and run54's word, and it is a drop-in longer run56.

**And the frame it was taken for was answered without it.** Item 129's
divergence at 3021 is a blocked stand, and the first thing asked of run56 —
already on disk — was the collision block it is made of: 249,293 agreeing
unit-frames, zero disagreements. The seam turned out to be forty-four frames
upstream and in a record nothing had ever compared, `BUILDDATA`'s own
`x_internal`/`y_internal`: the AI's Dock `o 2010`, laid on frame 2977 two
cells south of the original's (`docs/AI.md` §20). The capture cost an hour of
screen and the answer cost a widening, which is the queue's own rule about
grepping the dump before booking a reading, one level up.

**What run57 does say, and it is worth having.** Past 3,000 the two games
have parted, and the parting is all one thing's consequence: seventeen units
first diverge from **2978** on, and the two later buildings — `1/2011` on
3177 (one tile of `y`) and `1/2012` on 3977 (eight tiles of `x`) — go up
after the citizen that builds them is already walking somewhere else. Nothing
in the extra thousand frames is independent evidence, which is exactly what
makes it useful: **when the dock lands, this is the capture that says what
is next**, and it needs no second run to do it.

**And it did, the same day.** The dock landed on 2026-09-01 (`docs/AI.md`
§21), and run57 became a test the hour after —
`diff::tests::run57_s_four_thousand_frames_stand_where_the_original_s_do`,
both position records over 4,000 frames. What it says now: **130,326
building fields with two buildings wrong** — `1/2011` on 3177, one cell east
in `x`, and `1/2012` on 3977, one tile — and **322,683 collision fields with
none wrong**, the fourteen units that ever leave the original's point all
leaving it at or after 3177. So the seventeen-unit consequence was the
dock's, as this section supposed, and what is left past 3,000 is one AI
placement and its own consequence. The word went 3021 → **3435** on the
same pair of fixes.

**And where its test stands after item 133** (2026-09-01). The building
half is **130,326 fields with one building wrong** — `1/2012` on 3977, one
tile of `y` — and the collision half **337,265 field-frames with none
wrong**, eleven units ever leaving the original's point, the first on 3582.
Both numbers moved on a change that has nothing to do with either: the
dock's gull, which took East Indies' word 3579 → 3608 and so moved where
the two streams part. The building assertion is now scoped to frames
**before the word** and the collision total is a floor rather than an
equality; see the marker below.

**And it answered a capture that had been booked against it** (2026-09-01).
`docs/TRANSPORT.md` §12's fourth check wanted a new `UNITS=3` window over
frames 3600–3640 to see the first boarding. run57 *is* that window: same
game, run39's detail, 4,000 frames. Blocks 3585–3609 hold the whole
mechanic — the AI scout `1/0` idle at `(40416, 34272)` through 3583, an
eleven-waypoint path and a `MOVE_TO` to `(35712, 25728)` on 3584, an order
list of `[CASTORDER spell 650 paid 0, MOVEORDER]` with a path top carrying
`flags 4` on 3608, and on 3609 the barge `1/14` at `(41112, 33695)`, guy
`type 320`, holding the scout's path with that flag cleared and the
scout's orders minus the cast, the scout itself `inside_up 14`. No screen
time; a `sed` range. The rule it illustrates is the queue's own, one level
up: **grep the dump before booking a capture, not only before booking a
reading.**

**`FABLE:` what may be asserted past the parting.** For one item this
capture's test asserted **zero** wrong building fields over all 4,000
frames, and that assertion was luck. `1/2012` on 3977 came back one tile
north the moment the word moved 3579 → 3608 on the dock's gull, a change
with nothing to do with it, and the choice was between reverting a
29-frame word gain and rescoping the test. **It was rescoped, in code** —
`build_bad` is filtered to `frame < LONG_WORD_EAST_INDIES` and `coll`
became a `>=` — so this is a code-changing verdict to re-read from its own
citations, not a proposal to weigh.

**Why a past-the-word assertion is luck, stated precisely, because the
obvious reason is the wrong one.** The harness does *not* free-run:
`Built::tick` installs the original's `game_random` word at the end of
every frame the dump carries a checksum for, and run57 is a per-frame full
dump, so both sides **start every frame on the same word**. What is not
reset is the position *within* a frame. From the first frame whose draw
sequence differs — 3608, `cast_transport` — this simulation spends a
different number of draws before the AI's own rolls, so every later draw in
that frame takes a value that is not the one the original took there, and
the state built from it persists. A placement 369 frames on is decided by a
roll that is nobody's. So the parting is not seed drift and cannot be fixed
by more re-seeding; it is the missing draws themselves, which is the main
loop's job and not a test's.

What a pass should weigh: **for** — run53's and run54's tests already say
out loud that past the parting "the totals there are coincidence that moves
with every unrelated change", and run57's was the one place that rule was
not applied, so this is consistency rather than new policy; the scope kept
is the half a shared stream backs, and both residues the test was written
around (`1/2011` on 3177) live inside it. **Against** — the strongest claim
this project has ever made about East Indies was "nothing is wrong in
either record over all 4,000 frames", it held for a day, and scoping to the
word means a real placement defect introduced past it now passes quietly;
worse, the scope *shrinks* whenever a capture is longer than the word,
which is every capture from here on. A third way nobody costed: keep the
whole range and make it a **ratchet** — assert `build_bad.len() <= 25` with
the offending building named — which fails on a new past-the-word defect
where a scope cut cannot, at the price of a number that has to be edited
every time the word moves. Whether a ratchet is a guard or a nuisance is
the question, and it applies to every score this repo pins past a parting.

**`FABLE:` the lane's own ordering rule, proposed and not adopted.** This run
is the first evidence that "size the capture to the word" has the *order*
wrong rather than the size: the two records that answered item 129 were both
on disk, both parsed, and neither had ever been asserted, so the hour of
screen bought frames nobody needed yet. The proposal is a clause — **widen
every dumped record the mechanic touches before booking a capture, and take
the capture only for what no record on disk can answer** — and it belongs in
`CLAUDE.md`'s working agreement beside "Grep the dump before booking a
reading", which is the same rule one level down.

It is marked rather than written because it is a working-agreement change,
and those are the steering session's (`docs/DECISIONS.md` entry 22: Fable
writes `CLAUDE.md` and the queue). What a pass should weigh: **for** — item
129 is a clean instance, and item 128's own predecessor was found the same
way (`docs/QUEUE.md` item 87, the widening ledger, is the standing count of
records parsed and never compared); **against** — one instance is not a
rule, and the lane's value is that it runs *beside* the main loop and costs
it nothing, so an ordering clause that makes a capture wait on a widening
may spend the screen's idle hours rather than save them. run57 was not
wasted; it was second-best. Whether "second-best" is worth a rule is the
question.
