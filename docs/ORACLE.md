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
what it and a second, older logger actually produce. The recorded-game
container claims in Part 1 remain unexercised: this install ships no recorded
games and none has yet been made.

**Where the implementation is.** ~~Nowhere yet, deliberately.~~ The
recorded-game reader still waits for a file to read — a reader written
against no sample is a reader that cannot be wrong in any detectable way,
which is the opposite of what `docs/FORMATS.md`'s evidence rule is for. The
**gamelog** reader exists as of 2026-08-20: `crates/rondata/src/gamelog.rs`,
written against two dumps from this install, with the constants check in
`dump.rs` and the diff harness in `diff.rs` (`docs/DATALAYER.md`).

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

## Running it: how far Wine gets, and what stops it (2026-08-20)

The first attempt at phase 2, recorded because the failure is specific and the
partial success is reusable.

**What works.** Homebrew's `wine-stable` cask (Wine 11.0, x86-64 under Rosetta)
installs without admin rights if `--skip-cask-deps` skips the `gstreamer-runtime`
`.pkg`, which needs a password and which Wine only wants for media playback. The
cask fails Gatekeeper, so `xattr -dr com.apple.quarantine` on the app bundle is
required or the binary is `SIGKILL`ed on launch. A prefix built with `wineboot`
comes up `win64` with a populated `syswow64`, and **32-bit PE execution works** —
`syswow64\cmd.exe /c ver` returns `Microsoft Windows 10.0.19045`.

`riseofnations.exe` then launches, loads 70 modules, and runs far enough to
write its own configuration.

**Which incidentally confirmed this document's INI derivation.** The game
created `rise.ini` and `rise2.ini` at
`%APPDATA%\Microsoft Games\Rise of Nations\`, the exact path derived above from
`Prefs::get_primary_app_directory`, alongside the `synclogger.ini` placed there
in advance. `rise.ini` also confirms the `[Section] key=value` shape, and turns
up two settings worth knowing:

```ini
[RISE OF NATIONS]
GraphicsDLL=d3dgl.dll
AllowLogs=0
Dialog Error Level (0 - 3)=2
```

`GraphicsDLL` means the renderer is a swappable module — but `d3dgl.dll` is the
only one the install ships, so there is no D3D9 fallback to switch to.
`rise2.ini`'s `Fullscreen=3` accepts `0` for windowed, which works.

**What stops it, and it is not a configuration problem.** Despite its name,
`d3dgl.dll` implements a **Direct3D 11** context — the strings around its error
are `d3d11context.cpp`, `IDXGIDevice`, `IDXGIFactory`, `IDXGIAdapter` — and it
requests exactly one feature level, `D3D_FEATURE_LEVEL_10_0`, with no fallback.
Three ways of providing that were tried and all three fail:

| path | failure |
| --- | --- |
| wined3d over OpenGL (default) | `wined3d_select_feature_level`: none of the requested levels supported with the current shader backend — macOS OpenGL caps at 4.1 |
| DXVK 3.0.2 | `Skipping: Device does not support required feature 'geometryShader'` → no adapters |
| wined3d over Vulkan (`renderer=vulkan`) | creates a `VkDevice` on the M4 Max, then `dxgi_device_init` fails `0x80004005` |

The DXVK line is the informative one. **Metal has never had geometry shaders**,
so MoltenVK cannot advertise the feature, and DXVK requires it. That is an
architectural gap rather than a missing package, and no amount of prefix
configuration closes it.

The remaining candidate was **D3DMetal**, Apple's Game Porting Toolkit
translation of D3D11 straight to Metal, which handles the gaps MoltenVK
exposes because it targets Metal directly rather than going through Vulkan.
CrossOver bundles the same technology. The next section is what happened when
it was tried.

It is worth being clear about what GPTK is *not* for here: it translates a
Windows binary's D3D calls, which is useful for running the original as an
oracle and has nothing to do with this project's own renderer. Phase 4 is a
Rust client and will not go near it.

---

## Running it, second attempt: the original runs, and it writes (2026-08-20)

**CrossOver with D3DMetal renders the game fully.** The whole path, so it can
be repeated without rediscovery:

```
brew install --cask crossover
cxbottle --bottle ron --create --template win10_64 \
         --param 'EnvironmentVariables:CX_GRAPHICS_BACKEND=d3dmetal'
wine --bottle ron --workdir <install> --wait-children <install>/riseofnations.exe
```

(`cxbottle` and `wine` are under
`/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/`; the
`--cx-app` form wants a bottle-internal path and fails on a native one.) The
game's own configuration lands at
`~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/`,
the same `%APPDATA%` path as before. The first launch after creating the bottle
page-faulted once in a system DLL; the second and every later one ran: player
profile, main menu, Quick Battle setup, map generation, the in-game view with
the economy ticking at its normal rate. Keyboard input reaches it from
`osascript`; synthetic clicks from System Events do not, and `cliclick` (brew)
does. The window is borderless at screen size, so screenshot coordinates are
click coordinates. The in-game menu is the icon at the top-right corner of the
screen; Escape does not open it.

**What it wrote back settles most of the open questions below, and one of the
answers is a second oracle nobody had derived.**

### The SyncLogger's real configuration

The game rewrites `synclogger.ini` on first read with its full key set. There
is no mask. Every one of the 37 categories is its own key — `WorldSync=0`,
`UnitsSync=0`, … — defaulting to **0**, so a file that sets only
`DesyncTrackingEnabled=1` and `DesyncCategoryMask=-1` enables nothing. It also
adds `SkipCountdown`, `NoteOnlySync`, `FinalSync` and, after a run, a
`LogFile=` line naming the `Logs\` directory. The categories-to-track header it
later writes lists `NoteOnlySync` and `FinalSync` as "cannot be turned off".

With every category on and a Quick Battle played for two minutes and quit
through the menu, four files appear in `Logs\`: `SyncLog Standard .txt`,
`SyncLog TurnLog .txt`, `SyncLog SendLog .txt`, `SyncLog ReceiveLog .txt` —
the `%s %s` of the pattern above are the session mode and an empty lobby
string. Each holds the settings header and then the line
`<snipped data frames>` under `Game completed without desync`. **In a solo game
that does not desync, the frame data is dropped at write time.** That matches
the code: `Game::run_solo` calls `setupWithConfigSettings`,
`beginNewLogSession(SessionModeStandard)`, runs the whole game, and only then
`writeToFileAndReset(null)`; the other writer is `CommandPackage::end_process`,
on an actual desync, which also sets `mDesyncOnTurn`. Whether a flag makes the
no-desync write keep its frames is the remaining question, named below. Killing
the process writes nothing, which is why the first two runs produced no file.

### The older logger, which is the one that works

`rise.ini` carries `AllowLogs=0`. `Log::init` reads exactly that key through
`Prefs` and returns before opening anything when it is 0; with `AllowLogs=1`
the game writes `gamelog.ini` with its own full key set and then
`Logs\gamelog.txt`, and adds `AllowLogs_ToConsole=1` to `rise.ini`.

`gamelog.ini` is the 2003 engine's logging switchboard:

```ini
[Logging Options]
Checksum Dump=-1
Checksum Break=-1
DUMP_ALL=0
LogFile=...\Logs\gamelog.txt
DumpFileName=Logs\dumplog.txt
[Misc Logging]   [Start Game]   [End Game]   [Start Frame]   [End Frame]
WORLD=0  CITIES=0  BUILDS=0  UNITS=0  ANIMALS=0  WALLS=0  AMMO=0  DEATHS=0
GROUPS=0  LEADERS=0  GUYS=0  GOODS=0  ITEMS=0  MAPMAKE=0  TERRAIN=0
PATHFINDER=0  CHECKSUM=0  RULES=0  SCRIPT=0  ...            (37 per section)
```

— the same 37 categories as `sSyncDefines`, under five phases. Each category
under `[Start Game]` dumps that subsystem once when the game starts; under
`[End Frame]`, **every frame**. The output is a nested text dump produced by the
objects' own `log_data` virtuals — `UnitData::log_data` is slot 0 of
`Unit::vftable` — in the shape:

```
BEGIN FRAME 100
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 73
     o 0
     who 0
     x_internal 4248
     y_internal 32664
     z_internal 528
   BEGIN GUY
  BEGIN LEADERDATA
   who 0
   tribe 11
   ...
```

The initial dump with everything enabled (`InitialDump=1` in `rise.ini` plus
`[Start Game]` all on) is 337k lines and contains `BEGIN CONSTANTS` — **every
field of the loaded `Constants` struct, by name, in its in-memory
representation** (`unit_move_speed 1`, `river_modifier 512`,
`fort_upgrade_terr[scan] 2 4 6 9`, `peasant_rate …`) — followed by `BEGIN
WORLD` (`seed`, `xs ys`, `player_territory_limit 44`, …), every leader, every
city, building and unit with positions. That is a direct check on every
`Slot::Ratio256`/`Ratio100` claim `rondata` makes, and on `Tuning::RON` as a
whole, read from the program rather than from our reading of its loader.

Two practical facts about cost. With every category on under both `[Start
Frame]` and `[End Frame]`, the simulation crawled to about one frame per five
seconds and the file grew at ~25 MB a minute — unusable. With `[End Frame]`
`UNITS`, `LEADERS`, `DEATHS`, `CHECKSUM` only, the game ran at full speed and
logged 1,730 frames (1:55 of game time) in 20 MB: one `BEGIN FRAME n` per
simulation frame, every unit's `flags o who x y z` per frame. Per-unit detail
beyond the object base (`UnitData::log_data` goes on to `collide_frame`,
`damage_frame`, `angle`, and some fifty more fields) is emitted with a
detail-level argument of 1, which is presumably what `DUMP_ALL=1` unlocks;
untried.

**`Seed (0 for random)` in `rise.ini` fixes the game.** Two runs with
`Seed=12345` produced the same nation, the same map and the same opening; two
runs with `0` did not. So a logged run is reproducible from a config file,
which is the property the SyncLogger section above wanted and now has, from the
older system.

### What this makes possible

A replay diff no longer needs a recorded game at all. Fix the seed, enable
`[End Frame]` for the categories a mechanic emits, play or script a short game,
and `gamelog.txt` is a per-frame ground truth for exactly those subsystems —
positions for movement, leader fields for economy and tech, deaths for
attrition — against which `crates/sim` can be run from the same initial dump.
The `BEGIN CONSTANTS` block is the cheapest win and should be wired into
`rondata` first.

Two more things the running game offers, both read from the binary before it
ran: the **unit balance tool** (`game/balancerules.txt`, `UnitBalance` in
`unitbalance.cpp`) runs scripted unit-versus-unit combats and writes results —
its switch is `game.semaphore.ptr[1] & 2`, set somewhere unread — and the
`[Start Game]` dump with `RULES=1` ~~should print the loaded type tables~~
— **it does not**: the 114 MB run had `RULES=1` under `[Start Game]` and
wrote no type table and no `COMBATTABLE`; the only `RULES` in it are the
`GAME_RULES` and `RUSH_RULES` lobby settings. `Game::log_rules_data` is
reached some other way, or under a flag not yet found.

### Read back (2026-08-20, later)

The dump is now read by `crates/rondata/src/gamelog.rs`, and what it holds
is written up in `docs/DATALAYER.md`. In short: at detail level 0, per unit
per frame the object base (`flags o who x_internal y_internal z_internal`)
and nothing else — the `GUY` blocks carry `type` (a `TypeIndex`), position
and `angle` only in the start-of-game dump; per leader `who tribe
defeated_by gov score leader_flags leader_flags2`, no goods; buildings the
same base with no type; no terrain under any category the two runs enabled.
The `CONSTANTS` block's keys are the `Constants` struct's field names — the
lowercased `rules.xml` tags, one renamed — and its values the loaded
representation, which `rondata --gamelog` classifies for all 716 matched
constants and checks against every `Tuning::RON` slot (231 of 232 equal;
`LIBERTY_FREE_UPGRADES` is loaded and not logged). And the seed reproduces
the game to the position unit: two runs with `Seed=12345` move the same
units to the same coordinates on the same frames.

---

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
- **`DUMP_ALL=1`, `Checksum Dump` and `Checksum Break`** in `gamelog.ini` —
  the first presumably unlocks the detail-level-1 fields, the other two
  presumably take a frame number; all three untried.
- **The whole `walk_data` graph**, which is the actual header format. Read in
  outline only.
- **The recorded game's file extension and naming.** `String::time_stamp` builds
  it from two strings in the runtime string table rather than from literals, so
  it was not recoverable the way the INI keys were.
- **Whether `SimulationFps` moves the 15 frames per second** that
  `crates/sim/src/lib.rs` holds as `FRAMES_PER_SECOND`, or something else.
- **Where `read_package` stops** — the stream has no count that has been seen,
  so the reader presumably runs to end of file. Unconfirmed.
