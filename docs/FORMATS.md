# File formats

The reverse-engineering log. **Every claim here must cite evidence** — a byte
offset from a real file, a cross-check against a second file, or a link to
prior work. No inferred struct layouts, no "probably a length prefix". If we
cannot show why we believe something, it does not get written down as fact; it
goes under *Open questions*.

That bar got much easier to clear on 2026-08-19. See *The PDB* below.

---

## The PDB — the depot ships full private debug symbols (2026-08-19)

`game/sbl/rise.pdb` is 55 MB of unstripped private debug symbols, and they are
the symbols for the executable we have:

```
$ llvm-pdbutil dump --summary game/sbl/rise.pdb
  GUID: {51D4F219-61C6-4F84-9D5B-C3361B0D291F}   Age: 1
  Has Debug Info: true   Has Types: true   Has IDs: true
  Has Globals: true      Has Publics: true
  Is stripped: false
```

`riseofnations.exe`'s `RSDS` debug directory carries GUID
`51D4F219-61C6-4F84-9D5B-C3361B0D291F`, age 1, path
`E:\agent\_work\2\s\main\game\rise.pdb`. Exact match — these symbols describe
this binary.

Contents: **5,880 `LF_CLASS`/`LF_STRUCTURE` definitions** with complete field
layouts (name, type, byte offset), **1,251 source file paths**, and function
names, addresses, and line numbers. `game/sbl/` also holds `rise_z.map` (13 MB
linker map) and PDBs for `CrossplayNetLib`, `CrossplayProxy`, `d3dgl`, `dssl`,
`PlayFabMultiplayerWin`, `PartyWin`, `version_maker`, and `xasl`.

`llvm-pdbutil` reads it on macOS. `pretty` needs the Windows DIA SDK and does
not work; `dump --types` uses the native reader and does.

**The source tree**, from the file paths (`e:\agent\_work\2\s\main\...`):

| Module | Files |
| --- | --- |
| `game` | 796 |
| `bighuge` | 167 |
| `basic` | 161 |
| `steamworks_sdk`, `packages`, `cellsdk`, `zlib`, `pnglib`, `cpclib`, … | 127 |

Sim-relevant units in `game/`: `recordgame`, `commandpackage`,
`commandmanager`, `commands`, `turncontrol`, `checksums`, `syncpoint`,
`syncfile`, `syncdir`, `syncdisplay`, `timesync`, `gamemath`, `orders`,
`ordmemmgr`, `pathfinder`, `borders`, `balance`, `unitbalance`, `coord`,
`world`, `object`, `unit`, `build`, `techtype`, `leaders`, `players`,
`groups`, `save`. Plus a complete BHS toolchain under `game/script/`:
`lexer`, `compiler`, `opcodes`, `virtualmachine`, `syntaxtree`, `symtable`,
`symtype`, `scripttype`, `runtimeenv`, `scriptfile`, `scriptfuncset`,
`scriptsyslib`, `breakpoints`, and `bighugescript.l` — a flex grammar for the
language.

### Layouts read directly from the PDB

```c
struct CommandPackage : GameAccess {   // sizeof 536
  +0    u32   stamp;      // turn stamp
  +4    int   play;       // player
  +8    int   valid;
  +12   int   group;
  +16   i16   size;       // bytes used in data
  +18   byte  data[514];  // packed variable-length commands
  +532  byte  padding[4];
};
// methods: add_command, add_group, add_chat, add_spline, copy_data,
//          walk_data, log_data, process, process_all, process_group,
//          process_ungraceful_player_drop, is_valid, init, close, clear

struct CheckSum : DataWalk {   // sizeof 24
  +16   u32 accum;
  +20   u32 size;
};

struct RandomLogEntry {   // sizeof 32
  +0    int    frame;
  +4    String file;
  +24   int    line;
  +28   int    seed;
};
```

Three things follow, and they are the most consequential facts we have.

1. **`CommandPackage` is the lockstep unit.** A turn stamp, a player, a group
   handle, and a 514-byte buffer of packed variable-length commands walked by
   `walk_data`. `add_command` / `add_group` / `add_chat` / `add_spline` are the
   writers. This is what `recordgame.cpp` persists and what
   `commandmanager.cpp` schedules. A matching error string in the executable:
   `CommandPackage::process_group --- broken replay`.
2. **`CheckSum` derives from `DataWalk`** — the same serialization visitor the
   save system uses (`basic/datawalk.h`). So the checksummed state *is* the
   serializable state, and reading `checksums.cpp` tells us exactly which
   fields participate.
3. **`RandomLogEntry` records the source file and line of every RNG draw**,
   with the frame and the seed. Big Huge Games built a desync-tracing rig into
   the engine. Whatever else is true, the RNG call sites are enumerable.

Supporting strings in `riseofnations.exe`: `CheckDesyncsEveryXFrames`,
`checksum_window_size`, `checksum_deep`, `checksum_failure_threshold`,
`Player: %d checksum: %d`, `%d/%d prior games have desynched`, `checksums.cpp`.

### How a position is stored (2026-08-19)

The single most reusable fact here, because every later subsystem needs it:
movement, maps, recorded games, and anything that reads a save file.

An object's coordinates live in `SubObjectData`, the 28-byte base of every
object in the game:

```c
struct SubObjectData {   // sizeof 28
  +8    u8          flags;
  +9    u8          who;          // owning player
  +10   i16         o;            // index in the owner's object list
  +12   Coord       z_internal;
  +16   Coord       x_internal;   // XOR-masked
  +20   Coord       y_internal;   // XOR-masked
  +24   ObjectType* ptype;
};
```

**Coordinates are XOR-masked with `0x63637`.** A player's age is masked with
`0x62766` and a city's stored age with `0x63187`. This is tamper resistance
against a memory editor, not encryption; unmask before doing anything.

**There are three units of length**, and the engine uses all three within a few
lines of each other:

| Unit | Size | Used for |
| --- | --- | --- |
| position unit | 1/768 cell | what `x_internal` holds |
| tile | 192 position units | territory distances, movement speed |
| world cell | 768 position units | ownership, one `WData` record each |

The evidence is the conversion itself rather than any annotation. The engine
converts a raw coordinate with `div_3_table[pos >> 8]` when it wants a cell and
`div_3_table[pos >> 6]` when it wants a tile. `div_3_table` is `.bss` — all
zeros in the image — and `init_coord_lookup_array` fills it at startup with
`i / 3`, and for negative indices with `(i - 2) / 3`, which makes it a floor
division rather than C's truncation. So a cell is `256 × 3` position units and
a tile is `64 × 3`. The table exists only to make a divide-by-three cheap on
2002 hardware.

Two independent confirmations arrive from the data side, which is what takes
this from a reading to a fact. `UNIT_MOVE_SPEED` is `1/192` of a tile per
frame. And the territory constants are annotated in tiles by the designers —
`TERRITORY_BASE` is `"24 tiles"`, `TERRITORY_LIMIT_BASE` is `"44 tiles"` — and
are consumed as distances in exactly this unit.

A cell's centre is the tile `4c + 2`. There is also a **half-cell** grid, at
`div_3_table[pos >> 7]`, used for the visibility and fog planes.

One inconsistency to watch for: the `City` record caches its own coordinates
unmasked, while the object it belongs to stores them masked. The territory pass
reads the city's copy directly and the fort's through the XOR.

See `docs/ATTRITION.md` for how these are used and for the rest of the object
and world layouts.

### RTTI is intact

Independently of the PDB, the executable retains RTTI — 1,818 demangled class
names via `.?AV…@@` symbols. The order vocabulary falls straight out:

> `MoveOrder` `AttackOrder` `AttackToOrder` `AttackGroundOrder`
> `AirAttackGroundOrder` `AirOrder` `AirPatrolOrder` `AwaitBoardOrder`
> `BoardOrder` `BuildOrder` `CastOrder` `ExploreToOrder` `FleeToOrder`
> `FollowOrder` `FormOrder` `GarrisonOrder` `GatherOrder` `GuardOrder`
> `PatrolOrder` `RepairOrder` `SpecialAnimOrder` `StrafeOrder` `TargetOrder`
> `ThinkOrder` `TradeOrder` `UnitOrder` `GroupOrder` `GroupMoveOrder`
> `GroupAttackOrder` `GroupAttackToOrder` `GroupPatrolOrder` `OrderList`

Also `Group` / `GroupData` / `GroupOut` and `HotKeyGroup` / `HotKeyGroupData` /
`HotKeyGroupOut`. The `…Out` suffix pattern appears to mark serialized forms.

### `obsoletescriptfuncs.txt` is engine source

At the install root, outside any archive: 947 lines of Big Huge Games C++ —
`Scriptfuncs.h` and `Scriptfuncs.cpp`, 122 signatures with real bodies. It
opens with a note that these are unsupported functions kept for reference.

It gives us the accessor layer by name (`LEADER2`, `WORLD`, `OBJECTX`,
`OBJECTS`, `BASETYPE`, `UNITX`, `BUILDX`, `MYLEADER`, `CAMERA`) and the
coordinate type `TCoord` (30 occurrences). Most usefully, it shows the shape of
order issuance: every unit-order function builds a local `Group`, calls
`add(object_id, player)` on it, then calls one of `issue_move_to`,
`issue_attack`, or `issue_stance`. `issue_move_to` takes a `TCoord` pair plus a
queue mode and a trailing order-kind selector — the same entry point serves
move, waypoint, explore, and flee, distinguished only by that last argument
(`EXPLORE_TO`, `FLEE_TO`).

**Orders are issued on a `Group`, not a unit.** A set of `(object_id, player)`
pairs, a verb, and parameters. The file also shows that a requested type is
remapped through `current_upgrade()` and `get_graft()` before it resolves — the
player's upgrade level and nation-specific substitution — and that the
object-id space is partitioned per player (`unit_mark[whom]`,
`build_mark[whom]`, `BASE_BUILDS`) while the *type* space is one range split by
base (`t - BASE_UNITTYPES`, `t - BASE_BUILDTYPES`).

---

## The XML layer is parsed positionally. Tag names are comments.

This supersedes the earlier claim in this document that the shipped DTDs are
authoritative. They are not, and building loaders from them would produce
types that cannot read the shipped data.

### The DTDs are stale editor artifacts

`rules.dtd` was generated by XMLSpy from an instance document — its own header
says so — and from an old one:

| | `rules.dtd` | shipped `rules.xml` |
| --- | --- | --- |
| `ROOT` content model | `(CONSTANTS, TECHBONUSES)` | 47 children |
| `CONSTANTS` members | 156 | **723** |

All 156 DTD constants exist in `rules.xml`, but the DTD's ordering is not even
a *subsequence* of the real ordering. `rules.xml` also declares no `DOCTYPE`,
so the DTD is never applied at load time.

### The parser ignores tag names — four independent lines of evidence

1. **Duplicate tag names within one parent.** `rules.xml:539-549` contains
   `CTW_STARTING_TRIBUTE`, `CTW_NO_ATTACK_BONUS`, `CTW_TRIB_NO_ATTACK`,
   `CTW_CONTINENT_BONUS`, and `CTW_ATTRITION` twice each, in two adjacent
   blocks. Name-keyed lookup cannot resolve that.
2. **The names are absent from the binaries.** Scanning 67,860,289 bytes of
   `riseofnations.exe`, `patriots.exe`, and every shipped DLL, for both ASCII
   and UTF-16LE: `UNIT_MOVE_SPEED`, `CITY_SPACING`, `CONSTANTS`, `TECHBONUSES`,
   `TRIBE_MASK`, `JOB_TIME`, `OBJ_MASK`, `SPLASH_PERCENT` — none present. The
   loader's own error strings *are* present in plain ASCII, so the string table
   is not packed.
3. **The loader's assertions are about counts, never names.**
   > `Number of tribes in rules.xml does not equal NUM_TRIBES`
   > `Num formations in rules.xml does not equal NUM_FORM_ALL`
   > `Number of unittypes in unitrules.xml doesn't equal NUM_UNITTYPES!`

   Also `NUM_BUILDTYPES`, `NUM_TECHTYPES`, `NUM_CRAFTTYPES`, `NUM_BONUSTYPES`,
   `NUM_GATHER_LAND`, `NUM_MAKE`.
4. **The serialization is visibly struct-shaped.** `entry0`…`entry8` attribute
   families; fixed-width space-padded text (`AZTECS.XML   `, `aztecs   `,
   `Line         `).

**Therefore: a record's index is the engine's type id.** `unitrules.xml` record
*N* is unit type *N*. This is very likely how orders encode unit and building
types, which makes the content work a direct input to the replay work.

One consequence worth noting: `rules.xml:2432` has
`<!--<CATEGORY name="Record Game"/>-->` commented out inside
`CATEGORIES id="gameinfo_flags"`. Under positional parsing that shifts every
subsequent flag index.

### The shape of the data

```
rules.xml     ROOT ─ CONSTANTS       723 slots    global tuning
                   ├ TECHBONUSES     122 BONUS    each with one PREQ
                   ├ FORMATIONS       10 FORM     Line/Refused/Envelop/Wedge/…
                   ├ LANDS             9 LAND     each with 4 <MAKE num type>
                   ├ TRIBES           24 TRIBE    FILE + KEY → per-nation XML
                   ├ TRIBES_TRIAL_VERSION  18 TRIBE
                   └ CATEGORIES  × 41            setup enums, id-tagged
unitrules.xml        UNIT           364 records, 55 fields, 300 distinct names
buildingrules.xml    BUILDING       129 records
techrules.xml        TECH            85 records, 17 fields
```

The 24 `TRIBE` entries point at per-nation files (`AZTECS.XML`, `MAYA.XML`, …)
that are **not** in `Data/`. Locating them is an open question.

### Every scalar is `<number><unit> <free-text commentary>`

All 851 values under `CONSTANTS` parse as a leading numeric literal: 560
integer, 252 percent, 36 rational, 3 multiplier, 0 non-numeric. Everything
after the number is prose the parser discards.

```xml
<UNIT_MOVE_SPEED value="1/192 tile (granularity for unit movement speeds)"/>
```

**Values are rationals, not decimals** — `1/192 tile`, `2/3`, `3/2 tile`,
`6/5 base rate`. This is exactly what `fixed::Fx::ratio` consumes, so no float
need ever exist. `rules.xml:17` establishes the time base: frames are
fifteenths of a second. Worked example — a Citizen's `<MOVES>25</MOVES>`
against `UNIT_MOVE_SPEED = 1/192 tile` gives 25/192 tiles per frame.

`UNIT_COST_FACTOR`, `BUILD_COST_FACTOR`, and `TECH_COST_FACTOR` are all
`10 resources`, confirming that costs are stored ×10 — a Citizen's
`<COST>2f</COST>` is 20 food, the letter being the resource.

`ATTRITION` is `48 frames`, described in its own trailing comment as the
baseline level for regular attrition.

**A `/` does not always mean division.** In `CONSTANTS` it does. In a `COST` or
`SUPPORT` field it separates resources: `75g/40m` is seventy-five gold *and*
forty metal. The two are told apart by the resource letter. Six letters occur
across all 27,645 record-table field values — `f` food, `t` timber, `g` gold,
`k` knowledge, `m` metal, `o` oil — matching the six `entry0`..`entry5` slots
of `STARTING_GOODS`, whose commentary names them. The only other numeric
suffixes anywhere are `rng` (on `RANGE`, which is a `min-max` pair) and `tsx`
(on `JOB_EXTRA_TIME`).

`support` is likewise an annotation, not data: all 364 `SUPPORT` values end
with the word. Thirteen of them are *only* the word — records 351–363,
Boadicea and the herd animals — which are engine-spawned objects that never
ramp. A parser that requires an amount before the annotation rejects those
thirteen.

The word is misleading and the annotation is the least of it: `SUPPORT` is not
upkeep. It is the **ramping cost**, and `docs/COSTS.md` establishes that from
its consumer. The designers say so themselves in the column comment above
`COST`.

### `SUPPORT` reaches the engine as two ordered slots (2026-08-19)

`COST` and `SUPPORT` share a grammar and do not share a storage. `Type::load_cost`
zeroes a six-integer array and writes each parsed pair into the slot its
resource letter names, so a duplicate letter overwrites. `ObjectType::load_support`
does something else entirely: it keeps `int support_good[2]` and
`int support_amount[2]`, walks the pairs in written order, **skips any whose
amount is zero without consuming a slot**, and **stops after the second**.

Two behaviours follow, and `cargo run -p rondata` checks both against the
install. Anything written past the second non-zero pair is silently dropped —
no shipped record does that. And a field naming the same resource twice fills
both slots with it, so the ramp matches both and that resource ramps twice:
records 16, 17 and 18 — Militia, Minuteman, Partisan — write `2f/2f support`
and therefore ramp four food each and no metal, which reads as a typo for
`2f/2m` and behaves as written.

Buildings reach the same two slots through named columns instead —
`SUPPORT0`/`SUPPORTVALUE0` and `SUPPORT1`/`SUPPORTVALUE1` in
`buildingrules.xml`, with the resource written out as a word rather than a
letter. Same fields, two spellings.

### `resourcerules.xml` wraps its records one level deeper

Every other record table hangs its records off the document root.
`resourcerules.xml` puts its 50 `RESOURCE` records inside a `RESOURCES`
element, the way `rules.xml` groups its several unrelated arrays. A reader that
only looks at root children finds nothing and must be told the container.

The first six records are the basic goods in the engine's order — Food, Timber,
Wealth, Knowledge, Metal, Oil — and the other forty-four are the rare
resources. Each of the six carries four redirect pairs, of which
`docs/COSTS.md` depends on the first two — `UNDISC_COST_GOOD`/`_RATE` and
`OBS_COST_GOOD`/`_RATE`. The `*_SUPPORT_*` pairs that follow them look
identical and are read by nothing found; `cargo run -p rondata` checks the cost
pairs specifically, because checking the support pairs is what let a wrong
redirect table stand.

### A rounding hazard the data creates

The game's units are deliberately tiny, and `Fx` truncates toward zero on
every operation by design. So evaluating a scaled rational in two steps is not
the same as evaluating it in one:

| | raw Q16.16 |
| --- | --- |
| `ratio(1,192)` then `× 25` | 8525 |
| `ratio(25,192)` | 8533 |

Eight raw units is a fifth of a thousandth of a tile. Over ten thousand steps
it is a tile and a half, and in a lockstep sim that is a desync rather than a
rounding error. **Scale inside the ratio.** `rondata::Scalar::scaled_fx` exists
to make the correct form the easy one, and the difference is pinned by a test
so it cannot quietly change.

### `TRIBE_MASK` is a 24-bit string, MSB-first

Leftmost character is tribe 23; rightmost is tribe 0. Verified three ways
against the `TRIBES` ordering (`0:aztecs … 16:koreans … 23:persian`):

- `Samurai` → japanese; `Cossack` → russians. Naive left-to-right indexing
  yields turks and french.
- `CITIZENS` and `CITIZENSKOREAN` are exact complements, and the differing bit
  is koreans (index 16) under MSB-first.
- `GENERAL` and `GENERALGERMAN` are exact complements on three bits.

This is how 364 unit records collapse to 300 units: per-nation art variants are
separate records selected by mask.

---

## BHS — the scripting language

`game/ai/scripts/` holds three files: `aibestbuildlibrary.bhs`,
`defensive.bhs`, `economic.bhs` (2,400 lines total). C-like, with `int` and
`String` as the only types, no arrays and no structs. Persistence is `static`
at script scope, so the eight player slots are hand-unrolled
(`prev_step0`…`prev_step7`, dispatched by `switch (who)`).

- `int ai economic(int who, ref int step, int boom_vs_rush, int num_loops)` —
  the engine calls repeatedly and reads the return as scheduler feedback
  (`BLOCK_ON_THIS` / `DONT_BLOCK_ON_THIS` / `SCRIPT_DONE`, declared in a
  `labels { … }` enum block).
- Control flow within a routine is a continuation machine:
  `trigger name() { … }` plus `enable_trigger("name")`.
- `include "aibestbuildlibrary.bhs"` is a preprocessor directive.
- Content is referenced by **display-name string** — `"City State"`,
  `"Woodcutter's Camp"` — so a name→ordinal lookup sits on top of the
  index-keyed tables.
- 68 distinct built-ins are called across the three files. **`rand_int` is one
  of them**, which means the scripted AI draws from the sim's RNG and is
  therefore part of the deterministic state. We inherit that constraint.

These three files are opening build orders and economic posture only. Combat,
pathing, and target selection are in the executable. The full language
implementation — lexer, compiler, opcodes, VM — is enumerated in the PDB under
`game/script/`.

---

## Verified: the shipped install (2026-08-19)

Depot `287450` pulled to `game/` (2.87 GB).

**Two executables:** `riseofnations.exe` (9.9 MB, **32-bit i386**, machine type
`0x14c`) and `patriots.exe` (2.3 MB). The base game and Thrones & Patriots
ship side by side rather than merged. The 32-bit target is what makes running
the game on Apple Silicon awkward — see `docs/DECISIONS.md`.

**Formal schemas beside the data** in `game/Data/`: `rules.dtd` (685 lines),
`unitrules.dtd`, `buildingrules.dtd`, `techrules.dtd`, plus `resourcerules`,
`craftrules`, `citytemplates`, `goods`, `paramtypes`, `soundtypes`,
`soundfiles`, `sound`, `playerprofile`, `triggerbuilder`; also `unitrules.xsd`,
`sound.xsd`, and `.sps` schema-project files. All of it is stale editor output
— see above.

**Loose plain-text data at the install root**, outside `Data/` and outside any
archive: `balancerules.txt`, `counterchart.txt`, `game.txt`, `graphics.txt`,
`interface.txt`, `labels.txt`, `masks.txt`, `soundlist.txt`, `soundtypes.txt`,
`taunts.txt`, `saveobjects.txt`, `obsoletescriptfuncs.txt`.
`counterchart.txt` is the likely home of the rock-paper-scissors matrix.

**`rules.dat`** is a gzip stream (502,699 bytes decompressed) containing a
binary record array with no strings. Not read by name from either executable.
Purpose unidentified.

**No BIG archives are present** anywhere in the depot. Assets are in loose
directories: `art/`, `terrain art/`, `sounds/`, `tribes/`, `mapstyles/`,
`conquest/`, `scenario/`, `bond/`. This removes the BIG reader from the
critical path entirely — the Extended Edition appears to ship unpacked.

**Not yet identified:** `rules.dat`, `Ron.s14`, `rise xml.spp`, and the `bond`
directory.

**Localisation noise:** many `Data/` files have `.xml.4`, `.xml.7`, `.xml.9`
siblings. Per-language variants; ignorable wholesale.

---

## Verified: the per-nation files are in `game/tribes/`

An earlier open question asked where the 24 XML files `rules.xml`'s `<TRIBES>`
block names had gone, since they are not in `Data/` and there are no BIG
archives to hide in. They are in `game/tribes/`, alongside the `alex_*`
campaign factions, each with the usual `.xml.4` / `.xml.7` / `.xml.9`
localisation siblings.

What they hold is **less than expected**: a display name, a list of leader
names, a list of city names, and three art-style indices —
`UNIT_CONTINENT`, `BUILD_CONTINENT`, `BACKUP_BUILD_CONTINENT`. No bonuses, no
unit substitutions, no tech modifiers.

So the file answers the "where" and sharpens the real question. A nation's
mechanical identity is not in its own file. `LeaderData::has_tribe_bonus` reads
one power id per nation from the loaded tribe record at +0x54, with tribe
records at a stride of 0x5f0 — and `rules.xml`'s `<TRIBE>` entries carry only
`<FILE>` and `<KEY>`. Where that id comes from is unlocated, and it is what
gates the nation powers that `docs/SUPPLY.md` and `docs/ATTRITION.md` both
have to name indirectly.

### Some constants are loaded as 8.8 fixed point

`PARMENIO_RADIUS_ADJUST` is written `3/2` in `rules.xml` and consumed by
`HeroData::get_radius` as a multiply followed by an arithmetic shift right by
eight. A `3/2` scale through a `>> 8` means the loader stored 384, so the
rational was scaled by 256 on the way in rather than kept as a pair.

That matters beyond one constant: it means `Scalar::Ratio` values do not all
arrive in the same representation, and a checker that compares the written
digits will disagree with a simulation that holds the scaled value.
`sim::tuning::Slot::Ratio256` exists for exactly this, and `rondata`
reconstructs the scale rather than comparing text.

**And it is not a rule about rationals.** `Constants::init` was later read
directly, at the lines that load the economy's constants, and it settles the
question: `PEASANT_RATE` is written `10 resources`, has no `/` in it, and is
still read with `get_fraction(name, 0x100)` — so it arrives as 2560. Three
lines away in the same file, `CITY_GATHER`'s `10food` goes through
`convert_int` and arrives as 10. `OIL_RATE` and `SCHOLAR_RATE` are scaled;
`GATHER_RATE`, `COMMERCE_CAP`, `TERRITORY_TAXES` and every tax constant are
not.

So the scale is a property of the one line of `Constants::init` that reads the
constant, and nothing about the syntax, the file, or the trailing unit predicts
it. It has to be established per constant, at the loader or at the consumer.
See `docs/DECISIONS.md` entry 14, and `sim::tuning::Slot::Entries256` for the
array form.

### And 256 is not the only scale (2026-08-19)

Production adds two more conventions. `ACCEL_TRAIN`, `ACCEL_CONSTRUCT`,
`ACCEL_RESEARCH`, `UNIT_RATE_BASE` and `UNIT_RATE_PROGRESSION` go through
`get_fraction(name, 100)`, so `6/5` arrives as 120 and `1/1` as 100 — while
`RESEARCH_PREMIUM` and `RESEARCH_TICK_PREMIUM`, a few lines away in the same
loader, go through `get_fraction(name, 0x100)` and arrive as 256.
`sim::tuning::Slot::Ratio100` is the third variant.

And one constant is scaled outside `Constants::init` entirely.
`JOB_EXTRA_TIME`, a per-record column in `unitrules.xml`, is parsed inline by
the unit-type loader: `_wtoi` up to the `/`, `_wtoi` after it, a denominator of
1 when there is no slash, and `(numerator * 100) / denominator`. It never
touches `String::fraction`. Two integers away in the same struct,
`RESEARCH_PREMIUM_TIME` does go through `String::fraction(s, 0x100)`, so the
written `2` that 356 of 364 records carry arrives as 512.

**A rounding hazard this creates**, and one the checker walked into: rescaling
through `Fx` is not a safe proxy for the engine's own arithmetic. Q16.16 is
exact for `3/2` and for every denominator the 8.8 constants happen to use, and
inexact for `6/5` — 78643 raw, which rescales to 119 where the original
computes `6 * 100 / 5` as 120. `Scalar::fraction(scale)` reproduces
`String::fraction` directly and both slots now use it. See
`docs/PRODUCTION.md`.

---

## Verified: `QueueItem` is twenty bytes, and remembers three resources

A production queue entry is `QueueItem` in the PDB, which names every field:

```
+0x00  int       job_counter    progress, in hundredths of a frame
+0x04  short     type
+0x06  short[3]  good           resource indices, 0xffff for none
+0x0c  short[3]  cost           amounts actually paid
                                two bytes of tail padding to 20
```

The evidence is the type record itself plus `BuildQueue::set_queue`, which
writes it: a `short` at `+4`, a zeroed `int` at `+0`, then a walk over the
price's **six** resource slots that skips zero amounts, writes at most
**three** `(index, amount)` pairs at `+6` and `+0xc`, and pads the rest with
`0xffff` and 0. `BuildQueue::un_queue` `memcpy`s by `0x14`, confirming the
stride.

The three-against-six is the load-bearing part: an entry cannot remember a
price in four resources, and `Build::unpay_cost` refunds a cancellation from
these pairs and nothing else. No shipped record spends four, so this is
invisible in a stock game and would bite a mod.

---

## Prior art

The only serious public RoN format work is
[ptasev/Rise-of-Nations](https://github.com/ptasev/Rise-of-Nations): a BIG
archive extractor, a BH3/BHA ↔ glTF converter (both directions — which means
new art can be tested inside the original game long before our renderer
exists), an unmaintained Blender addon, and unreleased 3ds Max plugins.

Neither that repo nor RoN Heaven's modding library publishes a byte-level
format spec. That mattered a great deal before we found the PDB and matters
much less now — but it remains the reference for the model formats, which the
PDB describes structurally without describing their on-disk encoding.

---

## Formats by priority

### 1. Recorded games — still the behavioural oracle, no longer a mystery

Written to `Documents/My Games/Rise of Nations/Recorded Games/`. Playback via
*Tools and Extras*. No public parser exists; `RepInfo`, an old third-party
replay manager, is proof the header is tractable.

**Specified in `docs/ORACLE.md` (2026-08-20)**, which also corrects this
paragraph. The claim that "the engine computes per-frame checksums and keeps a
random log keyed by frame, source file and line" is true of the *engine* and
was wrongly carried here as a property of the *file*. It is not one.
`RecordGame::write_package` writes six fields — frame, play, valid, stamp,
size, payload — and `CommandPackage::checksums` is a **static** `ulong *`
beside the class, never serialised. A recorded game is the command stream, a
full initial-state snapshot walked by `Game::walk_data`, and the random seeds.
Nothing per-frame.

The random log is real but is a separate, switchable facility — `SyncLogger`,
37 named categories, driven by `.\synclogger.ini`, writing plain text in which
each entry carries its own source file and line. It is off by default and it is
a better oracle than the recording. See `docs/ORACLE.md`.

**We have no recorded games yet** — the install is fresh. Producing one
requires running the game, which requires solving 32-bit x86 Windows on Apple
Silicon. That is now less urgent than it looked: the tracer needs a running
game too, but the reading it enables no longer depends on obtaining a
recording first.

### 2. XML rules — solved, see above

Not a reverse-engineering problem. Positional loading, `Fx::ratio` for the
rationals, index-keyed tables.

Notable: **multiplayer aborts on data-file mismatch between clients.** Strong
evidence these files feed the deterministic sim directly rather than a
presentation layer, and that our sim can consume them unmodified.

### 3. BH3 / BHA models

3D geometry and animation. Converters exist in both directions. Not needed
until the renderer; *reading* them early is what proves out an eventual art
swap.

### 4. Map / scenario formats

Undocumented publicly. A recorded game is worthless without the map it was
played on. `game/scenario/` and `game/mapstyles/` are unexamined.

---

## Open questions

- Where is a nation's *bonus* loaded from? The per-nation files themselves are
  found — see below — and they do not contain one, yet `LeaderData::has_tribe_bonus`
  reads a single power id per nation from the loaded tribe record at +0x54.
- ~~Does the recorded-game container embed the checksums and the random log, or
  only the command stream?~~ **Answered** in `docs/ORACLE.md`: only the command
  stream, plus an initial-state snapshot and the seeds. So a replay diff is an
  *inferred* oracle — but the executable ships a switchable per-frame,
  per-category sync tracer that is an *exact* one, and it is independent of
  recordings.
- What is `rules.dat`? Gzipped binary records, not referenced by filename from
  either executable.
- Is the map stored in the recording, or referenced by name and hash?
- What is the `bond/` directory?
- Does `gamemath.cpp` imply the original sim is fixed-point, and if so at what
  scale? This bears directly on whether `Fx` at Q16.16 is the right shape.
