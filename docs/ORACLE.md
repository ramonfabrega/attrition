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
simulation frame, every unit's `flags o who x y z` per frame. ~~Per-unit detail
beyond the object base (`UnitData::log_data` goes on to `collide_frame`,
`damage_frame`, `angle`, and some fifty more fields) is emitted with a
detail-level argument of 1, which is presumably what `DUMP_ALL=1` unlocks;
untried.~~ **Both halves of that were wrong** — the detail argument is not 1,
and `DUMP_ALL=1` is not how you ask for it. See "The detail level is the
knob" below.

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
— ~~**it does not**: the 114 MB run had `RULES=1` under `[Start Game]` and
wrote no type table and no `COMBATTABLE`; the only `RULES` in it are the
`GAME_RULES` and `RUSH_RULES` lobby settings. `Game::log_rules_data` is
reached some other way, or under a flag not yet found.~~ **Found: the flag is
`DUMP_ALL=1`.** `RULES=1` was never the switch — `Game::log_rules_data` is
reached from `dump_all`, which `full_dump` calls only when it is passed a
non-zero argument, and the only thing that passes one is `do_dump_all`. With
it the type tables are all there: 1,820 `UNITTYPE` blocks, 387 `BUILDTYPE`,
255 `TECHTYPE`, and a `COMBATTABLE` of 493×493 shorts. See below.

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

### Running a check: the recipe in one place (2026-08-20, consolidated)

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

### The detail level is the knob (2026-08-20, third session)

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
and **`gamelog-run9-world6.txt`** (62 MB, 36 frames, `WORLD=6` — the map,
see "The map is a dump too").

### The lobby is a file: `-config` and `-automation` (2026-08-20)

`System::init_cmdlineopts` splits the command line on `/` and `-` and matches
six options by name. Two of them matter here.

| option | effect |
| --- | --- |
| `-config <file>` | extension `rcx` → `sys.playback_file`; extension `ini` → `sys.autostart_file` |
| `-automation` | `sys.automation = 1` |
| `-inifile <file>` | replaces `prefs_file`, i.e. which `rise.ini` is read |
| `-distribution <n>`, `-executable <n>`, `-touchpatch` | patcher plumbing |

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

Observed 2026-08-20: the rules half of the file took (team style, map size,
game speed, rules, difficulty, starting town, tech costs, population, rush
rules, start and end age, elimination, victory), while `mapstyles`,
`startingresources`, `revealmaps` and `PLAYERn_TRIBE` fell back to the player
profile. Not chased; ticking **Save to Profile** once in the lobby makes those
four stick across launches, which is enough. **Not established:** why those
four differ.

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

### The `~` console, which documents itself (2026-08-20)

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

### Staging a scenario: the chat cheats

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
| `move` | `(o[,who]\|select) (x,y\|cursor)` | `find_nearby_spot` + `set_new_location` |
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

### Traps that cost a run each (merged 2026-08-20)

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

### The map is a dump too: `WORLD` under `[Start Game]` (2026-08-24)

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

So **`WORLD=6` under `[Start Game]`** dumps the whole tile layer once —
regions, the coastal `region2`, the site values, the goods bits, the
owners — which is every map input the production AI's census and sites
read (`docs/AI.md` §2.3, §2.7, §2.13) and the forest layer the pathfinder
diff has been missing (`docs/PATHFINDER.md` §10). `TERRAIN=2` adds
`TerrainData`'s height table and waterline. It is a start-of-game cost
only; leave `[End Frame] WORLD=0`.

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

### `LEADERS=9` is the census oracle (2026-08-24)

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
  every frame. `Checksum Dump` / `Checksum Break` are still untried.
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
