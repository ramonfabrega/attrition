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

