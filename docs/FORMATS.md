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
  +18   byte  data[512];  // packed variable-length commands
  +530  byte  alignment[2];
  +532  Random padding;
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

The exported `types.txt` `/rise.pdb/CommandPackage` record establishes the
512-byte array and `Random` at +532; the two intervening bytes are alignment,
not payload. This corrects the earlier 514-byte label. The live clear-call
capsule verifies writes at +16 and +532; see
`audit/2026-09-09-live-call-capsule.md`.

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

> **Corrected, 2026-08-20 (`docs/DATALAYER.md`).** Half of this heading is
> wrong. **Records are positional** — the *N*th `UNIT` is unit type *N*, the
> per-class arrays are filled in document order and asserted by count — and
> that half stands. **Fields within a record are read by tag name.**
> `UnitType::init`, `BuildType::init`, `Constants::init` and the rest call
> `XMLElement::get_text(element, &out, tag)` / `get_attrib(…, tag)`, and the
> tag is a `String` fetched from the **internal string table** —
> `Data/internal_strings.xml`, where `TYPENAME`, `HITS`, `ATTACK`, `MOVES`,
> `RESEARCH_PREMIUM_TIME`, `UNIT_MOVE_SPEED` and the rest sit as `<STRING
> hash=…>` entries. `get_text` goes through `XMLNode::get_element(tag, 0, 0)`,
> which upper-cases the name and asks the MSXML DOM for that child. That is
> why evidence point 2 below found none of the names in the binaries: they
> are in a data file, not in the executable. Point 1 (duplicate tags) means
> only that a by-name lookup takes one of the duplicates — presumably the
> first; the shipped pairs are equal so nothing observable depends on it.
> Points 3 and 4 and the **Therefore** below are about records and still
> hold. `rondata` already reads fields by tag; the loader in
> `crates/rondata/src/load.rs` does so deliberately.

> **And one more turn of the screw, 2026-08-25** (`docs/DATALAYER.md`, "The
> name group"): a *unit* record does not always get its own element. The
> table is loaded in five passes, and from the third on, a record whose
> `NAME` repeats the previous one's is initialised from the **first record of
> that run** — the nation art variants share one row's columns. Only `NAME`,
> `GRAPH`, `TYPENAME`, `WHERE`, `FROM`, `JUMP`, `TRIBE_MASK` and `GRAFT` are
> always the record's own. Four shipped rows are read past this way, and the
> program's own type dump confirms all four.

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

### A dumped `type` and a `orig_type` index one object-type space

Established 2026-09-06 from run79's window, and it names every object any
dump prints. A guy's `type` and a `BuildData`'s `orig_type` are offsets
into a **single** space laid out in load order, so the record index is
recovered by subtraction:

    resources   0-49
    units       50-413    (50 + the `unitrules.xml` record)
    buildings   414-542   (414 + the `buildingrules.xml` record)

Buildings begin at 414 and `unitrules.xml` holds 364 records, which puts
units at 50 and leaves exactly the 50 of `resourcerules.xml` below them.
Every type in run79's window resolves hits for hits under it — guy 50
Citizen at 40, 59 Caravan at 90, 61 Merchant at 90, 69 Scout at 50, 177
Longbowmen at 88 (`UBER_SIZE 3`, upgraded from Bowmen), building 427 a
Barracks and 428 a Stable — with nothing left over.

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

### The combat columns, and `balance.xml` (2026-08-20)

`docs/COMBAT.md` §2.1 has the full table; the facts a reader of the data needs:

- **`ATTACK` is stored ×10.** `UnitType::init` does `attack = number × 10`,
  and every attack bonus in the engine is added as `N × 10`. The displayed
  value is the file's; the arithmetic is in tenths, and the damage formula
  divides by ten at the end (`(base + 5) / 10 − armour`).
- **`OBJ_MASK` is a string of letters.** Upper-cased; each letter `A`–`Z` sets
  bit `c − 'A'`, each digit `1`–`9` sets bit `c − 0x17` (so `1`–`6` are bits
  26–31). The legend — which bit means what — is not in `unitrules.xml`; it is
  in the row names of `balance.xml` (`Flag_A_OBJMASK_ARMORED` …
  `Flag_6_OBJMASK_ANTI_AIR`), and `docs/COMBAT.md` §3 lists it. `FLAGS` uses
  the same encoding lower-cased (`a`–`z`, bits 0–25); `r` is `is_siege`.
- **`RANGE` is `min-max`**, split at the `-`; no `-` means `max = min`. A
  cavalry-archer type (`FLAGS` `k`) has its maximum moved to
  `second_max_range` and its `max_range` zeroed at load.
- **`TARGET_SIZE` is multiplied by `UNIT_BLOCK_RADIUS`** at load;
  `SPLASH_AREA`, `SPLASH_PERCENT`, `AMMO_PER_ATT`, `RECHARGE`, `ARMOR`,
  `HITS`, `TO_HIT` (default −1), `ATTENUATE` (absolute value) are plain;
  `PROJ_SPEED` defaults to 200 for a ranged type that lacks it.
- **`data/balance.xml`** is `<ROOT><TABLE><ENTRY name="row" col="pct" …/>`:
  291 rows in the shipped file, each with the same 291 attributes. The
  engine's category space is 399 names — the 352 unit types by table index
  (their `<NAME>` with spaces as underscores and apostrophes dropped), then
  `SIEGE FORTS TOWERS CITIES OBSPOST BUILDINGS UNITS AGE_0…AGE_7` and the 32
  `Flag_X_OBJMASK_…` rows — and a row or attribute the file lacks is 100. It
  is read by *name* (`ENTRY[@name="…"]`), unlike the rules tables, so a row
  whose name matches no category is silently dead — the shipped file has four
  (`Pathfinder`, `Pioneer`, `Ranger`, `Marines`, units renamed since). `cargo
  run -p rondata` checks the rows come in the generated order and lists the
  dead ones. What the table is for, and how the engine multiplies it into the
  combat table, is `docs/COMBAT.md` §5.

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

**What the id *is*, in the shipped roster, now has a diff behind it**
(2026-09-01). Every `has_tribe_bonus` call site in the executable passes the
nation's own roster index — `0xb` guards the British arm of
`ObjectData::train_time`, `0xf` the Japanese, `0xe` the Chinese, `0xa` the
French, `0xc` the German, `6` the Roman, `0x11` the Mongol, `5` the Greek —
so `tribes[n] + 0x54` is `n` for at least those eight. One of them is now
measured rather than read: run58's AI is `tribe 11`, and its Dock's clock
takes `BRITISH_SHIP_SPEED` to the hundredth on the frame the original does
(`docs/PRODUCTION.md`, "The tail's first caller"). The *loader* that writes
`+0x54` is still unlocated; what is settled is what it must be writing.

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
swap — and one number in a `.BHa` turned out to be **simulation state**, so
the first slice of it is read already (below).

#### The animation file (`.BHa`) — its length is on the sync stream (2026-08-28)

`Guy::set_anim` writes `end_time = animmgr.frames[packet.action_ids[slot]]`,
and a clock that runs out **draws from `game_random`** (`docs/ANIM.md` §4).
So an animation's frame count is not art: it decides when a draw is spent,
and a unit whose length is unknown is a unit whose stream is wrong. Gaia's
bird is the case that forced it — no dump prints owner 9, so its piece and
its lengths had no other source (`docs/SYNC.md` §3.9).

`crates/rondata/src/artdata.rs` reads it. Three files, all of them open data
in the install:

| file | what it gives |
|---|---|
| `Data/unit_graphics.xml` | `<UNIT name="HERDSHEEP-TYPE1">` with `<ANIM name="CHAR_DEFAULT" file="Sheep Idle3"/>` children — the packet's `action_ids`, one row per slot |
| `Data/anim_graphics.xml` | `<ANIM name="Sheep Idle3" file=".\art\sheep_idle3.bha"/>`, under `<LOOPING>` or `<NONLOOPING>` |
| `art/*.bha` | the animation, whose root node's key times give the length |

**The container** is a chunk stream. A chunk header is eight bytes —
`{u32 size; u16 id; u16 num_chunks}` (the PDB's `ChunkHeader`; this
paragraph called the last field `version` until item 4019 read the type) —
and `size` is measured **from the `size` field itself**, which is what
`ChunkRead` does (`(int)&header->size + header->size`). A `.BHa` is three
nested headers deep: the file (id 0) at offset 0, the root node's wrapper
(id 8; written "the object" until item 4019 found it is a node) at 8, and
that node's key chunk (id 7) at 16.
The node's payload is a `u32` key count at 24 and then that many **36-byte**
keys from 28.

**The number** is the first `f32` of each key: that key's own duration in
seconds. `AnimObj::load_hier` accumulates `int(seconds · 1000)` — truncated,
then masked to 16 bits — into a `u16` per key, and `AnimMgr::force_load`
takes the **last** of them (the second-to-last for a non-looping animation)
as `times[]`. Then:

```
frames = round(times · 3 / 200)          # fifteen frames a second
```

which is `AnimMgr::force_load@0053ade0`'s own `(float)(times*3) / 200.0f`
with a half-up round, and `AnimationPacket::get_game_frames@00918cc0`
returns **3** for a slot the packet does not name.

*How it is checked.* Two ways, both of which can fail. The header arithmetic
is self-checking — a node's chunk must end exactly where its keys do, and
`cargo run -p rondata -- <install>` fails if it stops holding. And the
lengths themselves are cross-checked against the *other* oracle: a
`DUMP_ALL` dump prints `end_time` on every `GUY`, and
`the_install_s_gaia_lengths_match_the_dump_s` asserts every one of run12's
thirteen gaia rows against the file — the three sheep pieces' idles
(90 / 109 / 250, from `Sheep Idle1` / `Idle3` / `Idle5`) and the three fish
pieces' (170 / 101 / 116). That agreement is also what pins the `-TYPE<v>`
suffix in `unit_graphics.xml` to the variant `(seed + o) % 3` picks, which
is the one link in the chain neither file states.

*What is not read.* ~~The geometry, the bone hierarchy, the child nodes' own
key arrays, and the whole of `.BH3`.~~ All four are read since item 4019:
"The model files — `.BH3` and `.BHa` read whole", below. Only the gaia types' rows are built:
a player's unit needs `GraphicPieces::get_unit_gpiece`'s tribe, age and
gender walk to know which `<UNIT>` entry it plays, and that is unmodelled.

### 4. Map / scenario formats

Undocumented publicly. A recorded game is worthless without the map it was
played on. `game/scenario/` and `game/mapstyles/` are unexamined.

---

## The model files — `.BH3` and `.BHa` read whole (2026-11-08)

Item 4019. `crates/render/src/bh3.rs` and `bha.rs` decode both files from
bytes, and `rondata::artdata` resolves a piece to them. **How it was
established**: the loaders read in the decompile (`BH3Method::load`,
`load_bh3@009156f0`, `load_vertex_data`, `load_verts`, `load_faces`,
`load_texs`, `load_hier@00915010`, `BH3LightMethod::load_norms`;
`AnimMgr::force_load@0053ade0`, `AnimObj::load_hier@0054b700`). The PDB
types name the fields: `ChunkHeader`, `HierObj`, `Quat<float>`,
`Transform<float>`, `AnimObj` and `AnimKeyFrame`. A strict walk of every
file the install ships then proved the reading's arithmetic. ptasev's
converter was not needed as a reading aid: the loaders and the arithmetic
left no field unexplained but the ones listed below as not read.

### The chunk container

Both files are the `.BHa` chunk stream above. **`num_chunks` is the count of
chunks directly inside a container, and zero for a leaf**, in every one of
the 3,909 files. The readers hold each file to it, and to every chunk
ending inside its parent, so a file whose sizes do not sum is refused.

### The model file (`.BH3`)

```text
id 0  file                     num_chunks 2
  id 1  vertex data            num_chunks 4   load_bh3 → load_vertex_data per child
    id 2  vertices   u32 n, n × {f32 x, y, z, w}      load_verts: Point<float>, w = 1
    id 3  normals    u32 n, n × 16 bytes               read only by BH3LightMethod
    id 4  texcoords  u32 n, n × {f32 u, v}             load_texs: n × 8 bytes
    id 5  faces      u32 n, n × u16                    load_faces: n × 2 bytes
  id 6  node                   num_chunks 1 + children   load_bh3 → load_hier
    id 7  node data  u32 start, u32 num, u32 name_len, name (NUL inside),
                     f32 quat x, y, z, w, f32 trans x, y, z, w
    id 6  child node …
```

- **Each array chunk is exactly `12 + n · stride`**, the strides being the
  `memcpy` lengths of its loader (`n << 4`, `n · 8`, `n · 2`). Normals are
  16 bytes on disk, while `load_norms` copies `n · 12`. The size arithmetic
  settles the stride: all 1,847 files are exact at 16. Normals, vertices
  and texcoords are one per vertex. Faces are whole triangles whose indices
  are vertices, in every file.
- **A vertex's `w` is 1** in every file, which is what pins the 16-byte
  stride as `Point<float>`.
- **The normals are unchecked.** `BH3Method::load_vertex_data` returns an
  error for id 3, so the method that loads a model without lighting never
  reads them. 749 models carry `0xffffffff` words there.
- **A node is a `HierObj`.** `start`/`num` land in `HierObj::start` and
  `num`. They are the node's run of vertices: in 1,844 of the 1,847 files
  every run lies inside the vertex array and the runs' lengths sum to the
  vertex count. The data chunk is exactly
  `20 + name_len + 32` bytes in every file.
- **`load_hier` truncates the transform** to `(int)(v · 1000) · 0.001` for
  the quaternion and `(int)(v · 100) · 0.01` for the translation, then
  treats it as identity within ±0.001 (±0.01 for the translation).
  `Node::loaded_rotation` and `loaded_translation` give the kept values.
  The eighth float, read into the point's `w`, equals the quaternion's `x`
  in every file.
- **The file stores no bounds.** `Model::bounds` derives them from the
  vertices.
- **Two models fail the reader**: `citizen6_fem_Na` (a node's run 219 + 21
  past 237 vertices) and `granary_3kz` (0 + 104 past 58). The original
  copies without checking. `merchantfleet` has overlapping runs that stay
  inside the array, and it passes.

### The animation file (`.BHa`), the rest of it

- **A node is an `id 8` wrapper**: its key chunk (id 7) first, then its
  child nodes, with `num_chunks` equal to 1 plus the children. This is the
  same shape as a model's id 6 / id 7.
- **A key is 36 bytes**: `f32` duration, the quaternion `x y z w`, the
  translation `x y z` (`AnimKeyFrame::q` and `t`), then one word
  `load_hier` steps over. The kept values are truncated as a model node's
  are.
- **A key chunk's room is whole keys** in all 2,062 files. 269 nodes have
  room for one key past their count; none has a count that overruns. A
  count above 100,000 is "Illegal number of nodes".
- **`force_load` plays the first root**: a later `id 8` (16 files carry
  two roots, one carries three) and every `id 10` are closed unread. An
  id 10 is `u32 n, n × 24` bytes in all three files that carry one
  (`diplomat2_bribe`, `scout_sloop_death`, `nuke_sub_attack`). Any other
  id there is never closed, and the loop gives up as "Invalid Animation
  File".
- **The whole reader agrees with `artdata::key_times`** on the first
  root's times for all 2,062 files
  (`the_whole_animation_reader_agrees_with_key_times`). `key_times` is
  diff-backed against every dumped `end_time`, so that much of this reader
  is too.

### From a piece to its files

`artdata::unit_art`: a `<UNIT>` entry's `model=` is its `.bh3`. Its `<ANIM>`
rows name animations, which resolve through `anim_graphics.xml` as the
lengths' resolution does (the first case-folded match). The pieces an entry
covers are `entry_pieces`' arithmetic (ANIM §3.2). `artdata::piece_decoding`
decodes each file once, and the result is the score below.

**Pieces decoding: 1,439 of 1,452** (`artdata::PIECES_DECODING`; **1,465
of 1,478** since item 4028, below). The
1,452 distinct pieces come from 1,399 of the 1,435 entries. The 13 failures,
named by `the_pieces_decode`, are slots that cite `.bha` files the install
does not ship (`armsuplytruck_attackwalk`, `ceo_crew3_idle3`,
`siegeship_ship_walk`) and the one model `citizen6_fem_NA.bh3`.
`cargo run -p rondata -- <install>` re-derives the file counts and the
score.

### The gaia pieces and the building pieces (item 4028)

**A gaia entry is a piece.** `GraphicPieces::init_piece_ranges@008f70e0`
writes `first_bird_piece = 0xea8d` (60045) and `num_bird_pieces = 0x24`:
three variants for each of the twelve gaia types `0x192..0x19e`, which
`get_type@00902540` reads back as `(piece − first_bird_piece) / 3 + 0x192`.
For a piece in that range, `unit_node_from_gpiece@00901c30` builds the name
from the type's `GRAPH` (`+0x88`), `-`, `TYPE` (string 3709) and
`(piece − first_bird_piece) % 3`. It takes the `<UNIT>` whose name equals
that. There is no `-TYPE0` fall-back: a variant with no entry has no node,
and `init_unit_data@008f4290` returns `LIBERR_FILE_NOT_FOUND`.
`artdata::bird_pieces` adds these entries to `unit_art`, which was 26 more
entries and 26 more pieces, all decoding. Their pieces agree with the start
dumps' owner-9 `GUY` blocks: the sheep's 60063–60065 and the fish's
60072–60074. The sim-fed tables (`piece_lengths` and its siblings) keep
`entry_pieces` and are unchanged.

**Names compare without case.** `String::operator==@00a1f140` compares the
insensitive hashes, or falls to `_wcsicmp`. Every name lookup below depends
on this. Four building addon subjects match their `<AMMO>` only by ignoring
case (`metro_6t_ADDON` against `metro_6t_addon`, for example).
`a_building_s_addon_subject_matches_without_case` checks this. A
case-sensitive lookup drops M's building types from 20 decoding to 13.

**A building piece names a `<BUILD>`.** `init_piece_ranges` writes
`first_build_piece = 0xc606`, `num_build_pieces = 0x81` and
`num_build_pieces_per_age = 0x489`:
- `0x81` is 129, one per `buildingrules.xml` record, at `TypeIndex 0x19e + i`;
- `0x489` is nine art styles of 129;
- `total_num_build_pieces = 0x2448` is eight ages of `0x489`.

`get_build_gpiece@0090bce0` adds `0x81·style + 0x489·age + (TypeIndex −
0x19e)`. It walks the age down to 0, under the nation's `BUILD_CONTINENT`
(tribe `+100`), then under `BACKUP_BUILD_CONTINENT` (`+0x60`), then under
style 0. On each step it takes the first piece whose data loaded. Failing
all three, it returns `first_build_piece + TypeIndex − 0x19e`.

`build_node_from_gpiece@008fd550` builds the name as `GRAPH`, `-`, the
style, `-AGE`, then the raw age `(piece − first) / 0x489`.
`say_build_art_style_name@006f03f0` gives the style:
- arm 0 is empty, written `Default` (string 402);
- arms 1–8 are `internal_strings.xml` entries 6920–6927: `NEurope`, `Asian`,
  `Korean`, `American`, `Arab`, `Iroquois`, `Colonial`, `EIndian`.

`build_node_from_gpiece@00905050` looks the name up among
`building_graphics.xml`'s `<BUILD>` elements (`artdata::build_piece_name`
and `build_art`).

**What a `<BUILD>` loads.** In `init_build_data@008f2f20`:
- `texture`, string 136, with a snowy-tileset variant;
- `zbuffer`, string 3850. This is a `.bh3`, and the piece loads only when
  `zbuffer` is empty or its file exists;
- `anim`, through `decipher_animation`;
- `halfoffset`, string 3854 (`int_str_array + 0x12d18`, default 0), into
  `BuildRDataStruct::build_half_tcoord` (item 4035). The install writes
  it 0 on 622 `<BUILD>` entries and 1 on 291, and the same value on every
  entry of one `GRAPH` (133 of 133). Its neighbours read the same way:
  3850 `zbuffer`, 3851 `constructsize` into `build_construction`.
  `WallOut::draw@00641930` moves a building's anchor back `0x60` world
  units on both axes for each unit of it (`docs/RENDERER.md` §3.5).
  Diff-backed: 4,564 dumped building records anchor within a pixel of the
  original's projection with it, 3,397 without.
- `spritexoff`, `spriteyoff`, strings 3852 and 3853 (`int_str_array +
  0x12cf0`, `+ 0x12d04`, default −1; `internal_strings_uwp.xml` holds them
  between `zbuffer` and `halfoffset`), written into the sprite packet
  (`packets[0xf]`, its third pair) after `RData::load`
  (`artdata::BuildArt::sprite_offset`, item 4076).

**What a building draws: its `texture=` sprite** (item 4076), a 32-bit
`.tga` with near two-level alpha, at `build_x − (int)(spritexoff · s)`,
`build_y − (int)(spriteyoff · s)` (`SpriteMethod::execute@0091ca30`),
`build_x`/`y` the anchor's projection and `s = (int) fixed_zooms[6] ·
game_sprite_scale / (int) zoom_distance` (`draw_build@008e7610`, listing
`8e7a10`–`8e7a3a`). Packet-backed: run1559b's packet gives 13905 / 20858;
the values and the check by eye are in item 4076's journal. The `zbuffer`
model is the depth, not the picture.

**Which piece a building plays** (item 4035). `Wall::update_gpiece@00643070`
calls `get_build_gpiece@0090bce0` with the current type, the founder
(`BuildData::founder`, `+0x7f`) and the building's `max_age`. The walk:
down the ages from `max_age` (clamped to 0–7) in the founder's nation's
`BUILD_CONTINENT` (`Tribe +0x64`); then, where it differs, in its
`BACKUP_BUILD_CONTINENT` (`+0x60`); then in the default style; the first
piece that loads, else `first_build_piece + type`. Diff-backed: on all
4,564 building records of the Aztec Modern lobby's dumps, the walk from
the simulation's state gives the dump's `gpiece` (`artdata::BuildLook`).

Each `EXISTEVENT` in `init_build_events@008ea3c0` names a `subject`, an
ammo piece matched by `<AMMO name>` in `effects_graphics.xml`, and an
`anim`.

**Diff-backed by `BuildDump::gpiece`**, the `gpiece` the dump prints at
`WALLDATA`'s level. Run1428's capital `0/2000` prints 57144, which is
`VILLAGE-ARAB-AGE5`. All 25 pieces that the lobby's three dumps print for
M's 21 building types name an existing `<BUILD>`. Seven of them are the
last-resort `first_build_piece + type`, and each still names a
`…-DEFAULT-AGE0` entry. A building piece meets condition 1 when its
`zbuffer` model, its own `anim`, and each addon's model and `anim` decode
(`artdata::building_decoding`). 20 of the 21 types do, from 80 files.
~~The one short is the farm. `FARM-DEFAULT-AGE0` writes `texture`,
`zbuffer` and `anim` empty, and its look comes from the `<FARM>` element,
which nothing here reads.~~ All 21 since item 4037, from 81 (the farm,
below).

**The farm** (item 4037). It is not a mesh at a point.
`FARM-DEFAULT-AGE0` writes `texture`, `zbuffer` and `anim` empty, and
`GraphicPieces::init_farm_map` only returns `LIBERR_OK`. Its
look is a wheat field that `FarmsOut::draw@008d75f0` builds over the
farm's footprint every frame. The type is `0x1a1` (417): `Build::init
@00629740` hands a building of that type to `Farms::add@008d8a40`.
Three readers feed the field:
- `Farms::init@008d9310` (called from `Game::init@0058c480`) walks
  `building_graphics.xml`'s `Root/Buildings/Farm` (strings 400, 401,
  2673, names compared without case). It reads `stalk_red`,
  `stalk_green`, `stalk_blue` and `stalk_alpha` (strings 2674–2677,
  `get_attrib_num`) into four bytes at `farms + 0x40`. It reads
  `min_height` and `max_height` (2678, 2679, `get_attrib_float`) each
  × 3.0 into `+ 0x50` and `+ 0x4c`, and names the render state `Farm
  Blend` (2680). The install writes 166, 216, 188, 173 and 10, 36. The
  draw lerps a stalk's height between the two by its growth, and its
  colour from the stalk colour toward white.
- **The `<FARM>`'s own `texture=` is read and dropped.** `Farms::init`
  reads string 136 into the scratch `String` at `0xed6730` (`msg`; the
  listing, `008d93c8`–`008d93e9`), and nothing loads it.
- **The image is the tileset's.** The draw binds stage 0 to
  `&tilesets.cur_tileset->wheat_texture` (`008d84b1`–`008d84c5`:
  `TileSetData +0x98c`; `TileSet::get_wheat_texture` returns
  the same). `TileSet::init@0087b370` fills it from the tileset's
  `FARMTEX/TEXTURE` (string 6766) `file=` (743), 32-bit with two mips,
  and `TileSetData::load_all@008798b0` loads it. The seventeen
  `<TILESET>`s name three images: `farm_wheat_1.tga` (12),
  `farm_snowy_wheat_1.tga` (2), `farm_jungle_wheat_1.tga` (3). Great
  Lakes draws its tileset (`docs/MAPGEN.md` §4). The Aztec Modern lobby
  draws `early_autumn`, from the seed run1428 prints after `map.cpp`
  7805, 2,726,892,157.

`artdata::farm_art` reads the field's inputs the same way, and
`farm_decoding` decodes the image. A farm piece's condition 1 is its
`<BUILD>` plus that image (`building_decoding`, given the tileset).

**The images** (`render::tga`, item 4037). The install ships 2,901
`.tga` under `art/` and `terrain art/`, in three kinds, none colour-mapped:
- type 10, run-length (2,114: 2,113 of them 32-bit, one 24-bit);
- type 2, uncompressed (786: 782 of them 32-bit, four 24-bit);
- type 3, 8-bit grey (1).

Each ends exactly at its last pixel (283 files) or at a 26-byte
version-2 footer ending `TRUEVISION-XFILE.\0`. Every 32-bit file has
descriptor `0x08`: eight alpha bits, bottom-left origin. The reader
holds every file to this and decodes 2,901 of 2,901
(`every_image_the_install_ships_decodes`). Which image a model binds
is still unread beyond the attributes above.

### What is checked, and what is not established

- **Checked by a run over the install**, on every gate that has
  `RON_INSTALL`: every structural claim above
  (`every_model_and_animation_the_install_ships_decodes_but_two`, the
  survey). The `.BHa` root times are diff-backed through `key_times`.
- **Reading alone**: which field is a quaternion and which a translation,
  and the order of their components; the truncation; `start`/`num` as a
  vertex run; texcoords as `u, v`. No packet has been compared with a
  loaded `HierObj` yet. That comparison is what would back these against
  the original.
- **Not established**: how an animation binds to a model whose tree it
  does not match. Of the 1,439 decoded pieces, 801 have every played
  animation's tree equal in shape to the model's (`SAME_SHAPE`; 823 of
  1,465 since item 4028). The
  citizen's `peasant.bh3` has 29 nodes, and `man_walk.bha` has 27: the
  same tree without its two `ammo node` leaves. The normals' fourth word.
  The 24-byte `id 10` element. The key's ninth word. Which TGA a model
  binds, beyond the `texture=` attribute. Item 4028 left four things
  unread: ~~the farm's `<FARM>` reader~~ (item 4037: `Farms::init
  @008d9310`, above); what a building's `zbuffer` model
  is drawn as, read only as a `.bh3` here; the snowy-tileset texture
  variant (string `0x12b9c`), which Great Lakes may take; and the
  building's sprite TGA, which is outside condition 1 as a unit's
  texture is.

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

## Suspended-search cleanup fixture (2026-09-09)

`tools/explore/search_cleanup_oracle.py` uses authored instances of the local
PDB export's `Tree<PathNode*,int>` (28 bytes), its 20-byte tree node, and the
24-byte BRTree headers. `types.txt` names the tree's current data/metric at
0/4, length at 8, root/current/parent pointers at 12/16/20, and the ordinary
tree's ordered word at 24. Its node has left/right/parent pointers at 0/4/8,
data at 12 and metric at 16. The CollBlock tree has the same 28-byte header.
These are PDB-backed fixture layouts, not a new serialized format.

The listing of `Unit::clear_partial_path@005e3920` establishes the five unit
pointers at +0x104 through +0x114 and the recycler list/capacity/length words
used by this probe. `Tree<PathNode*,int>::clear@0046da60`,
`Tree<PathNode*,int>::delete_children@004550c0` and
`PathFinder::kill_tree@00687c10` establish the node and payload recycler
operands. The fixture maps only those three recycler words; the growth
increment is deliberately inaccessible. The executable's bytes are loaded
from the user's install, never stored in source. Complete-record native checks
and the untested branches are documented in
`docs/lab/2026-09-09-search-cleanup.md`.

## Suspended-search census metadata (2026-09-09)

The optional `RON_SEARCH_CENSUS` callback observes a natural
`PathFinder::astar_path@00683770` return of −1. `PathFinderData`'s PDB fields
`pathing_unit`, `limit` and `saving` are at +0x14/+0x40/+0x44 within the data
base, which `docs/PATHFINDER.md` §2 places at +0x40 in the complete object.
Together with the listing's singleton address, that selects the three reads
in `live_search_census.h`. The five saved unit pointers and recycler header
operands use the preceding cleanup fixture's evidence. The callback copies
headers through checked `ReadProcessMemory` calls, not direct graph walks.

INFO tags 130–136 are **our authored diagnostic protocol**, not original-game
format: 130 starts an event; 131 supplies each of five container pointer,
length and root triples; 132 supplies each of seven pool list, capacity and
length triples; 133 reports a failed or short read; 134 completes an event;
135 marks the 64-event cap; 136 declares version/cap or refuses installation.
Every packet carries its suspension ordinal and frame. The congestion experiment
now validates the metadata-reading branch in two live runs; host fixtures also
test its protocol and failure handling. Coverage is recorded in
`docs/lab/2026-09-09-search-census.md` and its successor
`docs/lab/2026-09-09-congestion-probe.md`.


## Congestion driver registry and receipts (2026-09-09)

`Units::make_valid@0061a960` establishes the owner-zero high-water length and
capacity at the units singleton +4/+8, with pointer array +0x10 (the PDB
`PtrArray<Unit>` layout); owner arrays stride 0x1c. The opt-in driver reads at
most 512 slots, retaining active ID/UID pairs across its baseline and selection.
The existing command probe's ObjectData/UnitData fields and
`CommandPackage::add_group@0094bb60` establish the group layout and ID packing.
No additional original serialization format is introduced.

INFO tags 140–144 are authored receipts: 140 is group/ID/UID/x/y, 141 is the
two group counts/centroid x/y/registry length, 142 is group/count/package length
before/after/issue frame, 143 is group/target x/y/two reserved zeros, and 144 is
failure reason/context/three reserved zeros. The enclosing record carries the
current frame. Source evidence, packet bounds and live coverage are in
`docs/lab/2026-09-09-congestion-probe.md`.

## Structural suspended-search graph (2026-09-09)

PDB `Tree<PathNode*,int>` and `Tree<CollBlock*,int>` have 28-byte headers and
20-byte nodes; the three BRTree variants have 24-byte headers and nodes. All
nodes place left/right/parent/data at +0/+4/+8/+0xc. BRTree nodes additionally
place `red` and `removed` at +0x14/+0x15. PDB `PathNode` is 36 bytes with its
parent at +0x20; `CollBlock` has size 108 but no exported fields. Native
`PathFinder::kill_tree@00687ba0` confirms the removed predicate for closed-tree
payload disposal. `PathFinder::kill_tree@00687c10` is its open-tree counterpart.

The authored graph file starts with eight u32s: magic 0x31475352, version 1,
frame, unit address, record count, file bytes, physical node count and PathNode
count. Each record is kind/owner/address/byte length followed by those bytes.
Kinds 1–7 are unit search slice, container, tree node, PathNode, opaque CollBlock,
recycler header and recycler array. Owner selects the existing five-container
or seven-recycler census order; scalar unit/PathNode records use owner zero.
The unit slice is +0x104 through +0x14b; recycler headers include 16 bytes, while
the replay grants only the first 12 to reject allocator growth. Complete bounds,
physical versus logical counts and live coverage are recorded in
`docs/lab/2026-09-09-natural-search-graph.md`.

## Restore-entry prefix packet (2026-09-09)

`PathFinder::find_upath_restore@00688f40`'s PE listing establishes its three
stack arguments (path stack, owner, object ID), ret 12 and the direct call to
`PathFinder::find_upath@00682f30` at 0x688fa5. It reads GameAccess globals
0xc061bc/0xc0618c, the selected repath counter, object pointer and encoded
coordinate words, and writes the established pathfinder limit/saving pair.

The authored `restore-prefix.bin` packet is 49 u32s: magic 0x31545352, version 1,
frame, unit; nine entry registers in pushad/flags order with corrected ESP;
four entry stack words; daemon/objects/slots/object pointers, repaths, encoded
x/y and pre-call limit/saving; nine delegation registers; twelve delegation
stack words; final limit/saving. INFO 160 names the hook version and three code
boundaries; 161 records unit/owner/ID/path-stack/repaths; 162 records
status/unit/bytes-written/expected-bytes/limit; 163 records a refusal. Every
record retains the tracer's current frame. The graph receipt and unit/frame
must also match. Evidence and scope: `docs/lab/2026-09-09-restore-entry.md`.

## The mountain templates — a range's cells are its image's alpha (2026-09-23)

*Item 604. How a placed mountain's tiles and solid cells are defined. A mine's
reach is measured on them (`docs/AI.md` §60, `docs/ECONOMY.md`, "The mine's
range"), so they are simulation state, not art.*

**Where they come from.** `Mountains::init@0089ad70` walks
`Data/effects_graphics.xml`'s `<MOUNTAINS>`. Each `<MOUNTAIN>` gets one
`Mountains::add_range@008992b0` call, and the call hands out the first free
of sixteen slots. So **the template index is the element's position**:
the shipped file has sixteen (`h1`–`h6`, `h6_3`, `m1`, `m2`, `m4`–`m8`,
`m10`, `s1`). The call passes three `file` paths. The first is
`TEMPLATE_TEX` (`*_disp_*.tga`), which `MountainRange::init@008998b0` loads
32-bit into a scratch image and reads. The other two (`MAIN_ALPHA_TEX`,
`RING_ALPHA_TEX`) are loaded as textures for the renderer.

**The file.** Every `TEMPLATE_TEX` is a TGA: type 10 (run-length), 256 ×
256, 32 bits, descriptor `0x08`. Eight alpha bits, and the **first stored
row is the bottom one**. A pixel is `b, g, r, a`; `init` tests `& 0xff000000`
on the loaded `A8R8G8B8`, which is the alpha.

**The frame.** Four pixels make a tile and sixteen make a cell. The origin is
`x0 = (w/2) % 16` (0 here). An offset is measured from the image centre:
`x/16 − (w/2 − x0)/16` for a cell, and the same over 4 for a tile, with
truncating division. The image's top row is the range's smallest `y`.

**The tiles** (`mount_tx`/`_ty`, `MountainRangeData +0x8`/`+0x24`). One pass
marks every pixel at `(x0 + 4i, y0 + 4j)` whose alpha is not zero, as a
vertex of a lookup grid. A second pass, row by row, adds a tile wherever all
four of its corner vertices are marked. That order is the list's order.

**The solid cells** (`solid_mount_wx`/`_wy`, `+0x74`/`+0x90`). Take every
16-pixel block whose far edge is still inside the image (`x + 16 < w`).
Sample a 5 × 5 grid at 0, 4, 8, 12 and 16, so a block shares its far edge
with the next block's near edge. The cell is solid when **more than
fifteen** of the 25 samples are set. `init` also accepts all 25 set, which
the count already covers.

**The placement.** `Mountains::add_mountain@0089c2e0` lays a template at a
location in cells. A solid cell is `loc + off` (and `WData.flags |= 0x10`
there). A tile is `4 · loc + off` (blocked, and `TData |= 2`). There is no
rotation and no mirror. `GameLog::dump_mountains@0092fca0` prints the
generator's placements in a `DUMP_ALL` head, after the tribes and before
`CONSTANTS`, as four arrays on the `FULL DUMP` block:

- `mountain_loc_wcoords_x` and `_y`: `length`/`size`/`increment`/`flags`,
  then `list[scan]` per entry. Despite the name, these are **cells**.
- `mountain_locs`: the same locations as `x`/`y`/`z` floats, `768 ×` the
  cell. They are the location cell's corner, not its centre.
- `mountain_types`: the template index per entry.

**How it is established:**

- **The packet.** run144's packet holds East Indies' eighteen placed ranges
  and their 107 solid cells, in three templates (`s1`, `m10`, `m8`, i.e.
  15, 14, 13). These rules give every cell, in the original's list order.
  Reading the image bottom row first loses all three.
  `the_templates_give_the_packet_s_solid_cells` (`rondata::diff::build`)
  asserts it, and skips loudly without the packet.
- **The tiles, on both maps.** The union of the placed templates' tiles at
  `4 · loc` is exactly the start dump's `OBJECT_MOUNTAIN` tiles: 1,956 on
  East Indies (run38) and all of Great Lakes' (run12).
  `the_placed_templates_tile_the_map_s_mountains` asserts both.
- **The placement and the index.** run38's and run20's placements equal the
  packet's list (location and type) on all eighteen. Great Lakes' run3,
  run12 and run34 agree with each other on thirteen. `find_nearest`'s answer
  is the dump's `MiningList::mtn` for both of run80's mines (6 and 0).
- **The survey.** `cargo run -p rondata -- <install>` re-derives the file
  shape and four counts: `s1`, `m10` and `m8` have 3, 13 and 14 solid cells,
  and `m4` has 244 tiles, which is run97's range.

**Not established:**

- the image sizes other than 256. Every shipped template is 256, so `x0` is
  always 0.
- `init`'s mesh, heights and texture arrays, which are the renderer's;
- `MountainRange::validate_tcoord`;
- the generator's own placement search (`verify_template` and its
  siblings). This crate reads the placements from the dump and does not
  place mountains itself.
