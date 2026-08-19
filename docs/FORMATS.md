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
Boadicea and the herd animals — which are engine-spawned objects with no
upkeep. A parser that requires an amount before the annotation rejects those
thirteen.

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

What we now know without having seen one: the payload is a stream of
`CommandPackage` records (layout above), the engine computes per-player
per-frame checksums with a configurable window, and it keeps a random log
keyed by frame, source file, and line. `recordgame.cpp` is named in the PDB and
its symbols are readable.

**We have no recorded games yet** — the install is fresh. Producing one
requires running the game, which requires solving 32-bit x86 Windows on Apple
Silicon.

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

- Where are the 24 per-nation XML files that `rules.xml`'s `<TRIBES>` block
  references? Not in `Data/`, and there are no BIG archives to hide in.
- Does the recorded-game container embed the checksums and the random log, or
  only the command stream? *(Determines whether the oracle is exact or
  inferred — still the highest-value unknown, but no longer unanswerable.)*
- What is `rules.dat`? Gzipped binary records, not referenced by filename from
  either executable.
- Is the map stored in the recording, or referenced by name and hash?
- What is the `bond/` directory?
- Does `gamemath.cpp` imply the original sim is fixed-point, and if so at what
  scale? This bears directly on whether `Fx` at Q16.16 is the right shape.
