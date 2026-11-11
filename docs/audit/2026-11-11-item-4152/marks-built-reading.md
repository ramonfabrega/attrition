# Item 4152, family 13 `marks-built` — first reading

- Family: 13, slug `marks-built`.
- Leaves (`LeaderData` offsets): `fort_mark` (+0x428), `oil_well_mark` (+0x430),
  `supply_mark` (+0x434), `wonders_built` (+0x854), `wonders_held` (+0x858),
  `senates_built` (+0x85c), `forts_built` (+0x8f0); all `int`.
- Reader: first reading, Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- Sources: the full decompile export (`~/ghidra-projects/decomp/`), the
  listing (`llvm-objdump` of `riseofnations.exe`), `crates/sim/src`.

## Method and the nets

- `Leader` (size `0x6eec`) is `LeaderData` (`0x6ee4`) at offset 0 plus 8
  trailing bytes: slot `i`'s `LeaderData` is at `0xe3a390 + i·0x6eec` and
  the array ends at `0xe71af0`, and `Forts::init_fort`'s stores to
  `leaders.list[who].field_0x428` assemble as `0xe3a7b8(%reg)` =
  `0xe3a390 + 0x428`. So a `Leader*` and a `LeaderData*` share offsets; no
  prefix to subtract.
- Export net: the field names (`->fort_mark` etc.) appear only in readers
  (`LeaderData::log_data`, `BuildData::construct_time`); every write is
  printed as `field_0x<off>`. A third spelling exists: `Setup::build_game`
  prints `leaders.list[k]._1076_4_` (decimal offset) for `+0x434`; the
  decimal forms of all seven offsets were grepped (`_1064_`, `_1072_`,
  `_1076_`, `_2132_`, `_2136_`, `_2140_`, `_2288_`): only `_1076_` hits,
  in `Setup::build_game`.
- Listing net: a full `.text` disassembly (`llvm-objdump`), grepped for
  the absolute address of each leaf in each of the 8 slots (56 patterns;
  43 hits, all slot-0 base + index register except `Setup::build_game`'s
  8 constant-slot stores), and for every store/`inc`/`dec`/`add` whose
  destination is `0x428|0x430|0x434|0x854|0x858|0x85c|0x8f0(%reg)` (55
  hits in 29 functions; each function's base register typed below).
- Register-relative stores attributed to `LeaderData`: `Leader::init
  (006e3930)` (`0x428/0x430/0x434(%ebx)`, `ebx = this`, at
  `006e4136..006e4148`) and `Leader::reset_score (006e37f0)`
  (`0x854/0x858/0x85c/0x8f0(%ecx)`, `006e3883..006e38b9`).
- Register-relative stores **not** `LeaderData` (checked): `ScenarioRead::
  load_leaders_chunk (009acde0)` stores off `edx = *(0xc061ec)`, the
  `Game` object (it tests `0x821`/`0x823(%edx)`, the semaphore bytes), not
  a leader; `ConquestLeader::*` is a separate CtW class of size `0x484`
  (its `+0x430` takes a vtable `0xb22a60`); `Constants::init`,
  `GraphicWinXML`, `PlayerProfile`, `TileSet::init`, `RiverOut::render`,
  `String::operator=`, `GraphicPieces`, `Game::loop`, `CP2PAuthPlayer`,
  `ConquestMap`, `EventGroup`, `GameSpyGroup`, `ObjectsOut` are other
  records (none takes a `Leader*`).
- Whole-record writers that cover all seven leaves: `Leaders::clear
  (006ed5d0)` memsets `[+0x0, +0x6929)` of every slot (callers:
  `Game::close (00594b80)`, `Game::run_oos_recovery (005869e0)`, and CtW's
  `ConquestGame::advance_round`, `ConquestLeaders::clear_no_attacks`); and
  `LeaderData::walk_data (006d6750)` hands `[+0x8, +0x692a)` to a stream
  (a read for the checksum; whether its load direction writes the record
  back on a saved-game load was not read).

## Leaf `fort_mark` (+0x428)

**What it is.** The per-player high-water mark of the fort registry
`forts.lists[who]` (a `PtrArray<Fort>`, 0x1c bytes per player at
`forts`): one past the highest index whose `Fort` is live (`Fort +6` bit
0). Every scan of a player's forts runs `i < fort_mark`. It is not a count:
free entries below the mark are counted in it.

### 1. Writers

| writer | site (listing) | store |
|---|---|---|
| `Forts::init_fort (0073ed70)` | `0073edee` `mov %eax,0xe3a7b8(%ecx)` | mark = old mark + 1 (append into an already-allocated tail entry) |
| `Forts::init_fort (0073ed70)` | `0073ee84` `mov %ecx,0xe3a7b8(%edx)` | mark = max(mark, idx + 1) |
| `Forts::close_fort (0073ec90)` | `0073ed51` `mov %eax,0xe3a7b8(%edx)` | mark-- per loop turn |
| `Leader::init (006e3930)` | `006e4136` `mov %eax,0x428(%ebx)` | `= 0` |
| `Leaders::clear (006ed5d0)` | memset | `= 0` |

No other store at `0xe3a7b8+k·0x6eec` or `0x428(%leader)` in `.text`.

### 2. Event, step, predicate

- **`init_fort(who, o)`**: find the first entry `i < fort_mark` whose live
  bit is clear; if found, reuse `i` (mark unchanged by the search). Else if
  `fort_mark < list.count` (allocated records left over above the mark),
  take index `fort_mark` and set `fort_mark += 1`. Else allocate a new
  `Fort` record, append, take `count − 1`. Then, in all three cases,
  `fort_mark = max(fort_mark, idx + 1)`, `Fort::init(idx, who, o)`. Step:
  0 or +1 per call, never more. Index is `who` (`param_1`, the building's
  owner byte `+0x9`).
- **Its one caller** (listing: one `call 0x73ed70`, at `006245a9`) is
  `Build::activate (00623e20)`, when the building's type answers
  `BuildTypeData::is_fort (00472ba0)` = `is(FORTX 0x1bb, 0)` (type vslot
  `+0xfc`, PE `0xb42b94+0xfc` = `00472ba0`). The arm is reached on **every**
  activation — no `captured`/`announce`/`counted` gate (decompile lines
  ~558–568, between `regen_roads` and the dock test). Then
  `UnbuiltForts::remove_unbuilt_fort` and `Build +0x78 = idx`.
- **`close_fort(who, idx)`**: `Region::fix_borders` if the fort's cell has a
  region, `Fort::close` (clears the live bit), then while `fort_mark > 0`
  and entry `fort_mark − 1` is not live, `fort_mark -= 1`. Step: 0 down to
  −mark. Its one caller (listing `00628ce7`) is `Build::close (00628980)`,
  when the type `is_fort` **and** `Build +0x78 ≥ 0` (decompile lines
  211–216).
- **Units**: an index count, dimensionless.

### 3. Reset

`Leader::init (006e3930)` line 372 zeroes it — **only when the tribe
argument is ≥ 0**: `init` returns early at line 321–323 when `param_2 < 0`,
before the mark clears at 370–376 (the score reset at line 140 is before
that return). `Leaders::clear` memsets it. `Leader::reset_score` does
**not** touch it. `Setup::build_game` does not zero it directly (it zeroes
`+0x408`, `+0x420`, `+0x434`, `+0x43c` of all eight slots at
`005ac35f..005ac495`, not `+0x428`).

### 4. Readers

All are loop bounds over `forts.lists[who]`: `Forts::find_fort (0073ebe0)`
(a live fort on exactly a cell), `World::compute_reg_territory (006b0bb0)`
(territory sources — **decision-changing**: forts project territory),
`Army::find_target (006f69b0)`, `Object::do_launch (0064f3b0)`,
`Leader::produce_unit (006cb9e0)` (`< 1` early-out at line 191),
`BuildTypeData::blocked_location (006375b0)`, `LeaderData::log_data`. The
mark itself changes no decision except as a loop bound; the decision is in
the live bits it bounds.

### 5. Crate site

The crate keeps no fort registry: forts are the buildings themselves
(`crates/sim/src/ai/ai_sites.rs:311` `fort_at`, `crates/sim/src/cities/place.rs:326`
`reg_forts`, both filter `alive && active && is_fort`). A builder adding
`fort_mark` needs a slot registry like `territory::supply::SupplyList`
(`crates/sim/src/territory/supply.rs:171`, which already reproduces the
same first-free / trim-the-tail rule for supply) opened in
`Sim::activate` (`crates/sim/src/cities/city.rs:2420`, the `is_fort` test at
line 2533) and closed in `Sim::close_building` (`crates/sim/src/cities/city.rs:2782`;
its fort arm at ~2906 already names `Forts::close_fort` and the `+0x78`
gate). Slot reuse makes the mark depend on close order, so a derived
"highest live index + 1" over buildings is only exact if index assignment
is reproduced.

### 6. Plain game?

Yes: any fort finished by construction (`Wall::do_construct` →
`activate(0,1,1)`), and any fort destroyed. Also reached by scenario load
and the editor, which a plain game does not use.

### 7. Confidence, not read

High for the rule (decompile and listing agree on all four store sites).
Not read: `Fort::close`'s body beyond the live bit; `Build::close`'s other
paths into the fort arm (only its one fort test was read).

## Leaf `oil_well_mark` (+0x430)

**What it is.** The high-water mark of `oil_wells.lists[who]` (0x1c bytes
per player at `oil_wells`; entry live bit at `OilWell +6` bit 0), same
discipline as `fort_mark`.

### 1. Writers

| writer | site (listing) | store |
|---|---|---|
| `OilWells::init_oil_well (0073f6f0)` | `0073f76e` `mov %eax,0xe3a7c0(%ecx)` | mark + 1 (reuse an allocated tail entry) |
| `OilWells::init_oil_well (0073f6f0)` | `0073f7fa` `mov %ecx,0xe3a7c0(%edi)` | mark = max(mark, idx + 1) |
| `OilWells::close_oil_well (0073f670)` | `0073f6d1` `mov %ecx,0xe3a7c0(%esi)` | mark-- while the top entry is dead |
| `Leader::init (006e3930)` | `006e4142` `mov %eax,0x430(%ebx)` | `= 0` (tribe ≥ 0 only) |
| `Leaders::clear (006ed5d0)` | memset | `= 0` |

### 2. Event, step, predicate

- **`init_oil_well(who, o)`**: the same first-free-under-the-mark / tail /
  append rule as `init_fort`; then mark = max(mark, idx + 1); the entry
  takes `o`, `who`, live = 1 and the region id of the building's cell
  (`OilWell +4`). Step 0 or +1.
- **Caller** (listing: one `call 0x73f6f0`, `00625a9f`): `Build::activate
  (00623e20)`, when the building `is(OILWELL 0x1a5, 0)` **or**
  `is(OILPLATFORM 0x1a6, 0)` (listing `00625a45..00625a93`: two pushes
  `0x1a5` then `0x1a6`, each through the Build vslot `+0xb8` `ObjectData::is`
  or, on the Build's own vtable, the type's `+0x60` `ObjectTypeData::is`).
  No `captured`/`counted` gate: every activation of a well or platform.
  The returned index goes to `Build +0x78`.
- **`close_oil_well(who, idx)`**: clears the live bit, sets the entry's
  `who` to `0xff` and `o` to `0xffff`, then trims the mark as `close_fort`
  does. Caller (listing `00628daf`): `Build::close (00628980)` when the
  type answers `BuildTypeData::is_gather_type (00472bb0)` (type vslot
  `+0x90`, PE `0xb42b94+0x90`), **and** `is(OILWELL)` or `is(OILPLATFORM)`,
  **and** `Build +0x78 ≥ 0` (decompile lines 244–272).

### 3. Reset

`Leader::init` (when tribe ≥ 0), `Leaders::clear`. Not `reset_score`, not
`Setup::build_game`'s direct zeroing.

### 4. Readers

`Leader::calc_gather (006ceee0)` lines 166–190: the loop over the player's
oil wells, adding each live well's `BuildData::calc_gather` into the
player's gather totals — the oil income (**decision-relevant only as a
bound**: the sum is order-independent). `LeaderData::log_data`. No other
reader in either net.

### 5. Crate site

No oil-well registry. `Sim::gather_sites_outside_cities`
(`crates/sim/src/economy/holdings.rs:535`) walks every building and takes
the active oil wells and platforms, which is what the original's loop
adds; the mark itself has nowhere to live. A builder would add a
`SupplyList`-style registry opened in `Sim::activate`
(`crates/sim/src/cities/city.rs:2420`) and closed in `Sim::close_building`
(`crates/sim/src/cities/city.rs:2782`). Note the close gate also needs
`is_gather_type`; `docs/CITIES.md` §1.5's `build_flags` table gives bit
`0x40` (`g`, type slot `+0x90`) to the oil well and platform, so both
close (that table's claim, not re-derived here).

### 6. Plain game?

Yes, once a player builds an oil well or platform (Industrial age onward)
and when one is destroyed.

### 7. Confidence, not read

High for the rule (both nets agree). Not read: `is_gather_type`'s body
(taken from `docs/CITIES.md` §1.5; were it false for a well type, its
entry would stay live and the mark would never trim); the `OilWell`
struct's other fields.

## Leaf `supply_mark` (+0x434)

**What it is.** The high-water mark of `supplies.lists[who]` (entry live
bit at `Supply +4` bit 0), same discipline as `fort_mark`. It bounds the
supply search that decides whether a unit in hostile territory is supplied.

### 1. Writers

| writer | site (listing) | store |
|---|---|---|
| `Supplies::init_supply (0073ad40)` | `0073adbe` `mov %eax,0xe3a7c4(%ecx)` | mark + 1 (reuse an allocated tail entry) |
| `Supplies::init_supply (0073ad40)` | `0073ae51` `mov %ecx,0xe3a7c4(%edx)` | mark = max(mark, idx + 1) |
| `Supplies::close_supply (0073acc0)` | `0073ad21` `mov %ecx,0xe3a7c4(%esi)` | mark-- while the top entry is dead |
| `Leader::init (006e3930)` | `006e4148` `mov %eax,0x434(%ebx)` | `= 0` (tribe ≥ 0 only) |
| **`Setup::build_game (005ac190)`** | `005ac373`, `005ac39b`, `005ac3c3`, `005ac3eb`, `005ac413`, `005ac43b`, `005ac463`, `005ac48b` (`movl $0x0, 0xe3a7c4 + k·0x6eec`, k = 0..7) | `= 0`, all eight slots, unconditionally |
| `Leaders::clear (006ed5d0)` | memset | `= 0` |

`Setup::build_game`'s eight stores are **not in the README row's writer
list**. They sit in a block (decompile lines 193–224, printed as
`leaders.list[k]._1076_4_ = 0`) that zeroes `city_mark` (+0x408),
`hero_mark` (+0x420), `supply_mark` (+0x434) and `special_mark` (+0x43c)
of every slot, before `Leader::init` runs per live slot (line 618).
`fort_mark`, `oil_well_mark`, `wonder_mark`, `dock_mark` and
`caravan_mark` are not in that block.

### 2. Event, step, predicate

- **`init_supply(who, o)`**: first-free-under-the-mark / tail / append,
  then mark = max(mark, idx + 1); the entry takes `idx`, `o`, `who`,
  live = 1. Step 0 or +1.
- **Callers** (listing: `00612943`, `00613796`):
  - `Unit::init (00612100)`: when the unit `is_supply` (vslot `+0xcc`; on
    `UnitData::is_supply` itself, type `unit_flags2 +0x2b8 & 0x40`), with
    no other gate; `Unit +0x84 = idx`.
  - `Unit::set_type (00612fa0)`: past its version gate (the
    `xpack_rtm_version` compare at lines ~320–333 that returns early), if
    the new type is a supply type and `+0x84 < 0`.
- **`close_supply(who, idx)`**: clear the live bit, entry `who = 0xff`,
  `o = 0xffff`, then trim the mark. Callers (listing `0060f7cd`,
  `00613750`): `Unit::close (0060ee50)` when the unit `is_supply` and
  `+0x84 ≥ 0`; `Unit::set_type` when the new type is **not** supply and
  `+0x84 ≥ 0` (then `+0x84 = −1`).

### 3. Reset

`Setup::build_game` (all slots, every game start), `Leader::init` (tribe
≥ 0), `Leaders::clear`. Not `reset_score`.

### 4. Readers

`Supplies::find_supply (0073aba0)` (loop bound, `0073abb7`, `0073ac94`),
called from `UnitData::in_supply (00609ef0)` — **decision-changing**: it is
the supply test that cancels attrition. `LeaderData::log_data`.

### 5. Crate site

Already modelled: `territory::supply::SupplyList`
(`crates/sim/src/territory/supply.rs:171`) reproduces first-free reuse and
tail trimming; since it pops trailing free slots, `SupplyList::len()`
(`:207`) **is** `supply_mark` (the original's search only inside
`[0, mark)` and the crate's search over all slots coincide because the
crate never holds free slots above its last live one). Opened in
`Sim::add_unit` (`crates/sim/src/units/store.rs:145`), closed in
`Sim::close_supply_in` (`crates/sim/src/territory/upkeep.rs:62`); the
`set_type` pair is a trait (`crates/sim/src/units/unit_set_type_view.rs:156`)
whose `SimSetType` adapter is a no-op
(`crates/sim/src/units/set_type_adapter.rs:210–211`). A builder exposing the
leaf reads `territory.supply[who].list.len()`.

### 6. Plain game?

Yes: every supply wagon trained, upgraded (if the version gate passes) and
killed.

### 7. Confidence, not read

High for the rule. Not read: whether `Unit::set_type` ever changes a
unit between supply and non-supply types in a plain game (it would need a
lineage that crosses the `0x40` flag); the version gate's exact semantics.

## Leaf `wonders_built` (+0x854)

**What it is.** A count of wonder activations credited to the player since
the last score reset. Not a count of standing wonders, and not the crate's
`Cities::wonders_built` (`crates/sim/src/cities.rs:63`), which is
`Game::wonders` (`Game +0x618`), a per-wonder-type bitmask — **a name
collision a builder must not merge**.

### 1. Writers

| writer | site (listing) | store |
|---|---|---|
| `Build::activate (00623e20)` | `00625b4b` `incl 0xe3abe4(%eax)`, `eax = owner·0x6eec` | +1 |
| `Leader::reset_score (006e37f0)` | `006e3883` `mov %eax,0x854(%ecx)` | `= 0` |
| `Leaders::clear (006ed5d0)` | memset | `= 0` |

### 2. Event, step, predicate

- **+1 per `Build::activate` of a building whose `is_wonder` answers**
  (listing `00625ac4..00625aff`): on the Build's own vtable (`0xb42174`)
  and the type's own `TypeData::is_wonder_type (00470780)` (type vtable
  `0xb42b94`, slot `+0x1c`), the test is **type index in `[0x20e, 0x21e]`**
  (PYRAMIDS .. SPACEPROGRAM; the listing compares `jl 0x20e` / `jl 0x21f`);
  otherwise the Build's vslot `+0x2c` (`BuildData::is_wonder`) answers.
  Indexed by the building's owner byte (`Build +0x9`).
- Order in the arm: `JukeBox` mood (console player only),
  `UnbuiltWonders::remove_unbuilt_wonder (0073c220)`, **then** the
  increment, then `Wonders::init_wonder`.
- **No `captured`, `announce` or `counted` gate**, and `Build::activate`
  has no return before this arm (every path reaches `LAB_00625a45`). So it
  counts every activation route of a wonder, read from the `+0x1a8`
  call sites in the listing (16 direct `call *0x1a8(%reg)` sites; a
  vslot load-then-call search adds only CtW, the cheat console and
  `UnitBalance`):
  - `Wall::do_construct` `(0,1,1)`: built.
  - `Cities::capture_city (00733380)` `(1,1,0)`: an active member of a
    captured city whose type lacks `build_flags & 0x2000` is
    `swap_team`ed and activated (decompile line 259 the flag test, 305
    the call). **If a wonder type lacks `0x2000`, capturing a city with a
    standing wonder credits the captor `wonders_built += 1`.** The
    wonders' `build_flags` were not read here.
  - `City::find_buildings (007384c0)` `(0,1,0)`, `Leader::defeat_by
    (006d1c80)` `(0,1,0)`, `Build::check_capture (006276a0)` `(0,1,0)`,
    `SpellType::cast_bribe (00672d00)` `(0,1,0)` (args read from the
    listing pushes at `007388a8`, `006d20ec`, `006734e2`): transfers.
    Whether each can carry a wonder was not followed.
  - `Leader::produce_building (006e1400)` `(0, arg, 0)`,
    `Console::hurry_action` `(0,1,1)`, `Setup::build_cities` `(0,0,0)`,
    scenario load and editor `(0,0,0)`.
- Units: a count of events.

### 3. Reset

`Leader::reset_score` zeroes it. Its callers (listing): `Leader::init`
(line 140, **before** the tribe < 0 early return, so on every init) and
`Setup::build_game` (`005ad1e7`, a loop over every slot with
`leader_flags & 1`, after the per-slot `Leader::init` at `005acef7` and
**before** `Setup::build_empire`'s calls at `005ad40b`/`005ad4d2`, which
place the starting cities and units — so starting objects activated by
`build_empire` are counted). `Leaders::clear` memsets it.

### 4. Readers

- `BuildData::construct_time (0062d5c0)` (listing `0062d646` `cmpl
  %eax,0x854(%esi)`, `esi = leadersc + owner·0x6eec`): **decision-
  changing**. For a wonder, an owner with tribe bonus `0x14` that holds no
  wonder (`LeaderData::get_wonders == 0`) and has `wonders_built == 0`,
  and a type other than SUPERCOLLIDER (`0x21d`) and SPACEPROGRAM
  (`0x21e`), takes the arm that the crate calls `american_free_wonder`.
- The checksum (`LeaderData::walk_data`, `[+0x8, +0x692a)`, the slots
  walked whole).
- No other reader in either net. Register-relative `cmpl $0x0,0x854(%eax)`
  in `Build::plunder`, `Object::disband` and `0x858` in `Leader::gain_tech`
  read off `*(0xc061f0)` (constants), not a leader.

### 5. Crate site

The crate has no counter. The activation is `Sim::activate`
(`crates/sim/src/cities/city.rs:2420`), but the crate's wonder-list writes
are not done there: `Sim::note_wonders` (`crates/sim/src/ai/ai_census.rs:1276`,
called from `crates/sim/src/core/frame.rs:443`) derives `init_wonder` from
the buildings at the objects pass's end and states "a capture is not
read". The one reader, `Sim::american_free_wonder`
(`crates/sim/src/cities/city.rs:1827`), stands in for `wonders_built == 0`
with the census's `wonder_mark` at zero, a proxy. A builder adds the
counter in `Sim::activate`'s non-city tail beside the wonder test,
incremented for **every** activation (captured included), and zeroes it
where the crate's setup corresponds to `reset_score` (the crate has no
`reset_score`; `crates/sim/src/setup.rs` has no such step).

### 6. Plain game?

Yes for construction. Capture of a city holding a wonder is a plain-game
event; whether it reaches this arm depends on the wonder types'
`build_flags & 0x2000` (not read).

### 7. Confidence, not read

High for the increment and its missing gate (listing read instruction by
instruction from `00625a45` to `00625b5b`). Not read: wonder types'
`build_flags`; whether `find_buildings`/`defeat_by`/`bribe` can hand over
a wonder; `BuildData::is_wonder`'s body for a non-Build vtable.

## Leaf `wonders_held` (+0x858)

**What it is.** The most wonders the player has held **at once**, sampled
only at a wonder's activation: a high-water mark, never lowered by a
wonder's loss. Not the current count.

### 1. Writers

| writer | site (listing) | store |
|---|---|---|
| `Wonders::init_wonder (0073c860)` | `0073ca1f` `mov %ecx,0xe3abe8(%ebx)` (after the read at `0073ca11`) | `= max(wonders_held, live)` |
| `Leader::reset_score (006e37f0)` | `006e3889` `mov %eax,0x858(%ecx)` | `= 0` |
| `Leaders::clear (006ed5d0)` | memset | `= 0` |

`Wonders::close_wonder (0073c7e0)` does not write it (its only `+0x8xx`
access is none; it writes `wonder_mark` +0x424 only).

### 2. Event, step, predicate

- **`init_wonder(who, o)`**, called once per wonder activation (listing:
  one `call 0x73c860`, `00625b5b`, in `Build::activate` right after the
  `wonders_built` increment, same predicate: `is_wonder`, no captured
  gate). It places the wonder in `wonders.lists[who]` by the same
  first-free / tail / append rule as the other registries (raising
  `wonder_mark`, +0x424), fills the entry (`who`, index, `o`, the frame,
  live = 1, `Wonder::get_timer`), sets `Game::wonders[type] |= 1`, then
  **counts the live entries in `[0, wonder_mark)`** (live bit at
  `Wonder +0xc` bit 0) — the new one included — and stores
  `wonders_held = max(wonders_held, that count)`.
- Step: 0 or +1 per activation in practice (the live count can exceed
  the old maximum by at most the one just added); it can stay put when an
  earlier wonder was lost and this one only restores the count.
- Indexed by `who` (`param_1`, the owner byte).

### 3. Reset

`Leader::reset_score` (from `Leader::init` on every init and from
`Setup::build_game` per live slot before `build_empire`), `Leaders::clear`.

### 4. Readers

`AchieveChartPane::set_chart_data (007f2c40)` (line 233, listing
`007f31d4` `pushl 0x858(%esi,%eax)`): the end-game achievements chart, UI
only. The checksum walk. **No sim decision reads it.** (`TerrainOut::render`'s
`field_0x858` is a `TerrainOut` field.)

### 5. Crate site

None. The crate's `Sim::wonders_held` (`crates/sim/src/economy/holdings.rs:572`)
is a different thing: a bitmask of wonder types standing, which the crate
reads for every wonder effect. The natural site is `Sim::note_wonders`
(`crates/sim/src/ai/ai_census.rs:1276`), which already reproduces
`init_wonder`'s slot choice per leader: after each insertion, count the
`Some` entries under `wonder_mark` and keep the maximum. Its own caveats
apply: it runs at the objects pass's end, in building order, and does not
read a capture.

### 6. Plain game?

Yes, whenever a player finishes (or, subject to the capture question
under `wonders_built`, takes) a wonder.

### 7. Confidence, not read

High (decompile and listing agree; one writer besides the resets). Not
read: `Wonder::get_timer`; whether any path sets a `Wonder` entry live
other than `init_wonder` (none found by the `+0xc` store grep, which was
not run — only the leaf's own nets were).

