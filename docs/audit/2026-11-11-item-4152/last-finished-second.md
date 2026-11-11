# Item 4152, second reading: `last-finished`

- Slug: `last-finished`
- Leaves: `last_building_finished` (`LeaderData` +0x6070, `int[129]`),
  `last_unit_finished` (`LeaderData` +0x6274, `int[352]`)
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), blind second reading, 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp`), `types.txt`,
  the `.text` listing of `riseofnations.exe`, and `crates/sim`.

## Common ground: layout, base, init

- `types.txt:42354–42355`: `+0x6070 int[129] last_building_finished`,
  `+0x6274 int[352] last_unit_finished`. 129 = the building `TypeIndex`
  range `0x19e..=0x21e`; 352 = the unit range `0x32..=0x19d`. The two
  arrays are back to back (`0x6070 + 129*4 = 0x6274`, `0x6274 + 352*4 =
  0x67f4 = ages_queued`).
- `Leader` (`types.txt:42086`, size `0x6eec`) lists no members of its own;
  `Leader::init (006e3930)` addresses `num_units` as `field_0x5762` and
  `high_buildings` as `field_0x5660`, the `LeaderData` offsets, so
  `LeaderData` sits at `Leader +0` and a `Leader*` displacement equals the
  `LeaderData` offset (the extra 8 bytes are at the end; `Leader::Leader
  (006d2bf0)` puts a vbtable pointer at `+0x6ee4`).
- **Both writers index with the type bias folded into the base.** The
  global array is `0xe3a390 + who*0x6eec`, written as `who*0x1bbb +
  TypeIndex` scaled by 4:
  - buildings: `0x6070 − 0x19e*4 = 0x59f8`, absolute `0xe3fd88`;
  - units: `0x6274 − 0x32*4 = 0x61ac`, absolute `0xe4053c`.
  So index `i` of `last_building_finished` is building `TypeIndex 0x19e+i`,
  and index `i` of `last_unit_finished` is unit `TypeIndex 0x32+i` — the
  same scheme as `num_buildings`/`num_units` (`0x5762 − 0x32*2 = 0x56fe`).
- **Init.** `Leader::init (006e3930)`, decompile lines 744–753, listing
  `6e4bef` (`leal 0x6070(%ebx),%edi`) and `6e4bfc` (`leal 0x6274(%ebx),%edi`),
  each followed by a `rep stosd` of `0xffffffff`: counts `0x81` and `0x160`.
  Every element is **−1**. This is on the seated path only (`param_2 >= 0`).
  An unseated slot (`param_2 < 0`, lines 70–105 and the early `return 0`
  at 323) takes `memset(this, 0, 0x6929)` instead (listing `6e398c`
  `pushl $0x6929; xor eax; push ebx; call memset`), so there both arrays
  are **0** — but `LeaderData::walk_data` only walks past the two flag
  words when `leader_flags & 1`, so an unseated slot's zeros are never
  checksummed.
- **Search nets run** (both leaves together):
  - export by name: `grep -rn -- 'last_building_finished\b\|last_unit_finished\b'
    funcs/` — no hits (the decompiler never names either field);
  - export by offset: every `field_0xNNNN` in `[0x59f8,0x5a22)` ∪
    `[0x6070,0x67f4)` outside `Conquest*`/`Terrain*` classes, and every
    leader-context line with a raw `0xNNNN` in that range: four hits —
    `Leader::init` ×2, `Build::activate:155`, `Build::train:81` — plus the
    reader `ScenarioFuncSet::get_last_type_built:69–80`;
  - listing, absolute: every immediate in `[0xe3fd88, 0xe40b84)`: only
    `0xe3fd88` (×3: `623f47` store, `9e99fb` load, `9e9a02` store −1),
    `0xe4053c` (×3: `62fa35` store, `9e9a21` load, `9e9a28` store −1), and
    `0xe3fdb2` (×8, which is `num_queued` at `+0x5a22`, not these leaves);
  - listing, register-relative: every displacement in `[0x59f8,0x67f4)`.
    No scaled-index (`(,%reg,4)`) access off a register at all; the
    unscaled ones are `Leader::init`'s two `lea`s and otherwise
    `ConquestWin`/`ConquestGame`/`ConquestNode`/`TerrainData`/`Game::run_solo`/
    `LoadGame`/`Popup` fields of their own structs (checked by `INDEX.tsv`
    owner of each site); `Achieve::record_data`'s hit is `-0x6238(%edi)`
    off a mid-record anchor, i.e. `+0x9d8`, not these leaves;
  - whole-record copies: no `rep movs` of `0x1bb9`/`0x1bbb` dwords and no
    push of `0x6ee4`/`0x6eec`/`0x37760` as a size; the only bulk access is
    `LeaderData::walk_data (006d6750)`'s raw range `[&who, +0x692a)`, which
    contains both arrays.

## Leaf 1: `last_building_finished` (`LeaderData +0x6070`, `int[129]`)

### 1. Writers

| writer | site | what it stores |
|---|---|---|
| `Leader::init (006e3930)` | dec. 744–747; listing `6e4bef` `rep stosd`, count `0x81` | −1 into all 129 (seated slot); 0 via `memset(…,0x6929)` for an unseated slot |
| `Build::activate (00623e20)` | dec. 153–157; listing `623f26`–`623f47` | the building's own object number `o` |
| `ScenarioFuncSet::get_last_type_built (009e9910)` | dec. 79–80; listing `9e99fb`/`9e9a02` | −1 after reading (read-and-clear) |

No other writer by either net (see "Search nets run" above).

### 2. The event, the step, the predicate

**Event: a building goes active, uncaptured and counted.** In
`Build::activate (00623e20)` (`thiscall`, three stack ints, here
`captured`, `announce`, `counted` for readability; the PDB gives no
parameter names), at its head before it calls `Wall::activate (0063e4b0)`:

- predicate: `captured == 0 && counted != 0` (listing `623f29 test edi,edi;
  jne` with `edi = [ebp+8]` loaded at `623e70`, then `623f2d test ebx,ebx;
  je` with `ebx = [ebp+0x10]` at `623f26`);
- index: the building's `ptype->type` (`SubObjectData +0x18`, then
  `TypeData +0x4`, a `TypeIndex`), minus `0x19e` (folded into `0x59f8`);
- slot: the building's `who` (`SubObjectData +0x9`, a `uchar`);
- value: the building's `o` (`SubObjectData +0xa`, a `short`, sign-extended
  by `movswl` at `623f34`) — its number in its owner's object array (the
  crate's `Building::index`, `[2000, 3000)`).
- It is an **overwrite**, not a step: the cell holds the most recent such
  building of that type for that leader. Beside it the same `o` goes to the
  scalar `ScenarioData::last_build_built_o[who]` (`0xcc1950 + who*4`,
  listing `623f56`) — a global, not a `LeaderData` leaf.

The call is virtual (slot `+0x1a8`; `vtables.txt:28415` `Build::activate`,
the other classes' `+0x1a8` is `Wall::activate`). Every slot-`0x1a8` call in
`.text` outside the UI classes, by both the `call *0x1a8(%r)` and the
`mov 0x1a8(%r),%r2; call *%r2` forms, with its three pushes read from the
listing (`arg3, arg2, arg1` in push order):

| caller | site | (captured, announce, counted) | writes? |
|---|---|---|---|
| `Wall::do_construct (006434d0)` | `643663` | (0, 1, 1) | **yes** |
| `Console::hurry_action (007c3810)` | `7c38bd` | (0, 1, 1) | **yes** |
| `ScenarioFuncSet::create_building (009f4d30)` | `9f4e9f` | (0, 1, 1) | **yes** |
| `Build::check_capture (006276a0)` | `627f82` | (0, 1, 0) | no |
| `SpellType::cast_bribe (00672d00)` | `6734e2` | (0, 1, 0) | no |
| `Leader::defeat_by (006d1c80)` | `6d20ec` | (0, 1, 0) | no |
| `City::find_buildings (007384c0)` | `7388a8` | (0, 1, 0) | no |
| `UnitBalance::next (009b8ac0)` | `9b9ece`, `9ba152` | (0, 1, 0) | no |
| `Leader::produce_building (006e1400)` | `6e2e0a` | (0, `[ebp+0xc]`, 0) | no |
| `Cities::capture_city (00733380)` | `733687`, `73390a` | (1, 1, 0) | no |
| `ScenarioFuncSet::switch_teams (009f46b0)` | `9f4833` | (1, 1, 0) | no |
| `Setup::build_cities (005ab910)` | `5ababf` | (0, 0, 0) | no |
| `ScenarioEditor::add_build`, `::activate_selected`, `ScenarioRead::load_build_chunk` ×2, `::load_city_chunk` | `99d134`, `9a06a2`, `9aa8c4`, `9aa9c0`, `9aa411` | (0, 0, 0) | no |

(The decompiler drops or truncates the argument list at several of these —
`City::find_buildings` prints `(0,1)`, `Leader::defeat_by` `(0)`,
`UnitBalance::next` and `cast_bribe` `()` — so the table is from the
listing.)

So in play the one writer is **`Wall::do_construct`'s completion**: each
builder's contribution adds to the site's work (`+0x48`), and when it
reaches the type's requirement (vtable `+0x18c`) the site is activated with
`(0, 1, 1)` (decompile 110–113). A Wall object reaches the same line, but
its slot `+0x1a8` is `Wall::activate`, which does not write; only a `Build`
records. The start city (`Setup::build_cities`, `(0,0,0)`), a captured or
bribed building, a building `Leader::produce_building` activates on the
spot (`counted` 0; the crate uses that arm at frame 0 for
`Setup::small_city_buildings`, `crates/sim/src/setup.rs` near
`produce_building`; I did not read which other callers reach it) and a
scenario-loaded building do **not** record.

Arithmetic: none — a copy of a `short` into an `int`.

### 3. What clears or resets it

- `Leader::init (006e3930)`: all −1 (seated) at game setup.
- `ScenarioFuncSet::get_last_type_built (009e9910)` sets the cell it reads
  back to −1.
- Nothing else. In particular, **nothing clears it when the building dies,
  is captured, or is deleted** — no writer on any death path — so the cell
  can hold the `o` of a dead building, or of an `o` since reused.

### 4. Readers

- `ScenarioFuncSet::get_last_type_built (009e9910)`, registered as the
  scenario-script function `"get_last_type_built"` with two arguments by
  `ScenarioFuncSet::init_funcs (009c7570)` line 2019. It takes a 1-based
  player and a type *name*; it walks `types` `0..0x326` for a name match,
  refuses a player outside `1..=8` or an unseated one (`leader_flags & 1`),
  and refuses a type that is neither a unit `0x32..=0x19d` nor a building
  `0x19e..=0x21e`. For a building it returns the cell and writes −1
  (read-and-clear). No file under the install's `ai/` calls it (grep, 0
  hits).
- `LeaderData::walk_data (006d6750)`'s raw range `[&who, +0x692a)` for a
  seated slot, reached from `CheckSums::check_one_leader (00937020)`,
  `CheckSums::check_leaders (009375a0)`, `CheckSums::check_all
  (00936560)`, `Leaders::walk_data (006e38e0)`, `SaveGame::verify_save
  (005a76b0)` and `LoadGame::verify_load (005a39f0)`: the sync checksum and
  the save/load round trip. That is why the leaf is on the leaders line.
- No sim rule reads it.

### 5. Where `crates/sim` has the event

It already has the exact site, stubbed: `crates/sim/src/cities/activate.rs:53`
`activate_head_steps` calls `v.record_last_built()` under `captured == 0 &&
counted != 0` (line 74–75), and the `Sim` view's
`SimHead::record_last_built` at `crates/sim/src/cities/activate.rs:139` is
an empty body marked `SEAM: nothing here reads the tables.` (a second empty
one at `:349` is a test view). It is reached from `Sim::activate`
(`crates/sim/src/cities/city.rs:2420`, calling the head at `:2454`), which
`do_construct_steps`' `SimConstruct::activate` calls as `activate(b, false,
true)` (`crates/sim/src/cities/construct.rs:207–208`). A builder fills
`SimHead::record_last_built` with `table[who][ty] = buildings[b].index`
(the crate's building record number is `TypeIndex − 0x19e`, as
`economy/cost.rs:1254` indexes `num_buildings`), adds the 129-entry `i32`
table to the leader state initialised to −1, and exposes it to
`crates/sim/src/checksum/leaders.rs:300`, where the leaf is already in the
range table.

### 6. Reached by a plain game?

**Yes.** Every completed construction of a building in a skirmish against
the computer — by a human's citizens or the AI's — runs
`Wall::do_construct`'s completion. The other two writers are off the plain
path: `Console::hurry_action` is the debug console's hurry (from
`ConsoleWin::run_cmd` and `Console::exec_keymap`), and
`ScenarioFuncSet::create_building` is the trigger layer. The reader
`get_last_type_built` (the only thing that can reset a cell after init) is
trigger layer too, so in a plain game the cell only ever moves from −1 to
the latest finished `o` per type.

### 7. Confidence and not read

High for index, value, predicate and init: the decompile and the listing
agree byte for byte at both stores, and the caller table is from the
listing's pushes. Not read: the bodies of `Console::hurry_action`'s and
`create_building`'s surrounding logic beyond the call; whether
`UnitBalance::next` is reachable at all in a shipped build; a pointer
anchored mid-record with a *negative* displacement into these arrays (the
sweep covered displacements in `[0x59f8,0x67f4)` and every absolute in the
range; the decompile's leader-context sweep found none either).

## Leaf 2: `last_unit_finished` (`LeaderData +0x6274`, `int[352]`)

### 1. Writers

| writer | site | what it stores |
|---|---|---|
| `Leader::init (006e3930)` | dec. 749–752; listing `6e4bfc` `rep stosd`, count `0x160` | −1 into all 352 (seated); 0 for an unseated slot (the same `memset`) |
| `Build::train (0062f9b0)` | dec. 72–81; listing `62f9fe`–`62fa35` | the new unit's object number |
| `ScenarioFuncSet::get_last_type_built (009e9910)` | dec. 67–71; listing `9e9a21`/`9e9a28` | −1 after reading (read-and-clear) |

No other writer by either net.

### 2. The event, the step, the predicate

**Event: a building trains a unit.** `Build::train (0062f9b0)`, `thiscall`
on the trainer, one argument, the unit `TypeIndex`:

- it calls `Objects::init_unit (0065e0c0)` (listing `62f9fe`) for the
  trainer's `who`, the type, the trainer's own position (`+0x10`/`+0x14`
  de-obfuscated with `^ 0x63637`), and three −1s;
- predicate: the return is not negative (`62fa08 test ecx,ecx; js
  630431` — a refusal returns at once with no write); there is no type,
  kind or player exemption inside `train`;
- index: the `TypeIndex` argument minus `0x32` (folded into `0x61ac`);
  slot: the trainer's `who` (`+0x9`);
- value: `init_unit`'s return, the unit's number in its owner's object
  array (the crate's `Unit::index`, `[0, 2000)`), as an `int`;
- an **overwrite**; beside it the same number goes to
  `ScenarioData::last_unit_built_o[who]` (`0xcc1930 + who*4`, `62fa16`).

`Build::train` is non-virtual; every direct `call 0x62f9b0` in `.text`,
mapped by `INDEX.tsv`:

| caller | sites | what it is |
|---|---|---|
| `Build::finished (00628490)` | `628665`, `628887` | a production-queue entry that is a unit (dec. 69), and the government hero (dec. 167) |
| `Build::process (0061edf0)` | `61f4cb`, `61f564`, `61f5c1`, `61f63d`, `61f72e` | the building's own periodic units (dec. 315 a `SPY`, 340/361/413 a type, 379 a `CARA`) |
| `Build::activate (00623e20)` | 15 sites `6246ca`…`6258cb` | the free-unit arms on a building going active |
| `Leader::gain_tech (006dcb60)` | `6dd93a` | the free bombers (`constants->americans_free_bomber`, dec. 629–640) |
| `City::assimilate (00738e90)` | `739199` | the government hero (dec. 139) |
| `Setup::place_unit (005abca0)` | `5abf3a` | the start's **fallback** only: after every placement try fails (`60` with a starting town, else `500`), a unit with a city trains out of it (dec. 43–53); the normal start placement calls `init_unit` directly and does not write |
| `ConsoleWin::run_cmd (007d6a70)` | `7df14d` | the debug console |
| `UnitBalance::next (009b8ac0)` | `9b9f2a` | the balance harness |

A unit made by any path that calls `Objects::init_unit` directly (the start
placement, a herd, a scenario's create-unit, a conversion) does not write.

Arithmetic: none — a copy of an `int`. Not read: what `init_unit` returns
for a squad type (the captain's number, presumably), so for a squad the
cell holds whichever number that is.

### 3. What clears or resets it

- `Leader::init (006e3930)`: all −1 (seated).
- `ScenarioFuncSet::get_last_type_built (009e9910)` sets the read cell to
  −1.
- Nothing else; a death does not clear it.

### 4. Readers

The same two as leaf 1: `ScenarioFuncSet::get_last_type_built (009e9910)`
(its unit arm, `TypeIndex` in `0x32..=0x19d`, read-and-clear) and
`LeaderData::walk_data (006d6750)`'s raw range for the checksum and the
save round trip. (The sibling script function `get_last_unit_built
(009e9a70)` reads the scalar `ScenarioData::last_unit_built_o`, not this
leaf.) No sim rule reads it.

### 5. Where `crates/sim` has the event

`Sim::build_train` (`crates/sim/src/cities/queue.rs:1693`, whose doc
comment names `Build::train (0062f9b0)`, "a unit born at a building") is the crate's single
`Build::train`; its first two lines are `who = buildings[at].owner` and
`unit = self.init_unit(who, ty, pos)`. A builder writes
`table[who][ty] = units[unit].index` right there (the crate's unit record is
`TypeIndex − 0x32`, `units/unit.rs:798`). Its non-test callers are the
production queue (`queue.rs:1612`, `:1675`), the free units of
`Build::activate` (`cities/activate.rs:391`, `cities/nations.rs:432, 524,
583, 655, 686`) and `cities/city_views.rs:591` — the same set as the
original's plain-game callers. `Sim::place_unit`
(`crates/sim/src/units/starting.rs:40`) stops before the fallback: it
returns `Err(PlaceRefused::Train)` where the original would train from the
city, so the start's rare fallback write is a SEAM there. The leaf is in
`crates/sim/src/checksum/leaders.rs:301`.

### 6. Reached by a plain game?

**Yes**, on the first unit any building trains, and on every free unit
`Build::activate`/`Build::process` hand out. Off the plain path: the
console (`ConsoleWin::run_cmd`), `UnitBalance::next`, and the reader's
clear (trigger layer). `Leader::gain_tech`'s bombers need the Americans;
`City::assimilate` needs a captured city — both reachable in a skirmish.

### 7. Confidence and not read

High for index, value, predicate and init: the decompile and the listing
agree at the store, and the caller list is the listing's complete set of
direct calls (no indirect call to `0x62f9b0` was looked for beyond the
`jmp`/`call` grep). Not read: the free-unit arms of `Build::activate` and
`Build::process` individually (they only choose the type; the write is in
`train`); `init_unit`'s squad return.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `last_building_finished` `+0x6070` `int[129]`, index `TypeIndex − 0x19e` | `Build::activate (00623e20)` (from `Wall::do_construct (006434d0)`'s completion; also `Console::hurry_action (007c3810)`, `ScenarioFuncSet::create_building (009f4d30)`); `Leader::init (006e3930)`; `ScenarioFuncSet::get_last_type_built (009e9910)` | overwrite with the building's `o` (`SubObjectData +0xa`, sign-extended `short`) when `captured == 0 && counted != 0` | −1 at `Leader::init` (0 for an unseated slot, never walked); −1 on a script read; never on death | `SimHead::record_last_built`, `crates/sim/src/cities/activate.rs:139` (empty SEAM), under the predicate already at `:74` | yes — every finished construction | high |
| `last_unit_finished` `+0x6274` `int[352]`, index `TypeIndex − 0x32` | `Build::train (0062f9b0)` (from `Build::finished (00628490)`, `Build::process (0061edf0)`, `Build::activate (00623e20)`, `Leader::gain_tech (006dcb60)`, `City::assimilate (00738e90)`, `Setup::place_unit (005abca0)` fallback, `ConsoleWin::run_cmd (007d6a70)`, `UnitBalance::next (009b8ac0)`); `Leader::init (006e3930)`; `ScenarioFuncSet::get_last_type_built (009e9910)` | overwrite with the new unit's `o` (`Objects::init_unit`'s return) when that return is ≥ 0 | as above | `Sim::build_train`, `crates/sim/src/cities/queue.rs:1693`, after its `init_unit` | yes — every unit a building trains | high |
