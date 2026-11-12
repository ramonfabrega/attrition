# Item 4152, family 13 (`marks-built`): adjudication

- Slug: `marks-built`. Leaves (`LeaderData`, all `int`): `fort_mark`
  (+0x428), `oil_well_mark` (+0x430), `supply_mark` (+0x434),
  `wonders_built` (+0x854), `wonders_held` (+0x858), `senates_built`
  (+0x85c), `forts_built` (+0x8f0).
- Readings: `marks-built-reading.md` (first), `marks-built-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original. The spawn line names one question to settle above the
  rest: whether a captured wonder credits the captor `wonders_built += 1`.

## Leaf `wonders_built` (+0x854): the spawn line's question first

### 1. Claim by claim

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | one gameplay writer: `Build::activate (00623e20)`, `625b4b` `incl 0xe3abe4(%eax)`, `eax = owner·0x6eec` | yes | yes | agree |
| 2 | resets: `Leader::reset_score (006e37f0)` `6e3883`; `Leaders::clear (006ed5d0)` memset | yes | yes | agree |
| 3 | whole-record `LeaderData::walk_data (006d6750)` writes it on a load | "not read" | yes | one-sided |
| 4 | predicate: Build vtable `0xb42174` and type vtable `0xb42b94` → type in `[0x20e, 0x21e]`; else type slot `+0x1c` / Build slot `+0x2c` | yes | yes | agree |
| 5 | order: JukeBox (console only), `remove_unbuilt_wonder`, `+= 1`, `init_wonder` | yes | yes (no JukeBox) | agree |
| 6 | **no `captured`/`announce`/`counted` gate** between `625a45` and the increment | yes | yes | agree |
| 7 | every path through `Build::activate` reaches `625a45` | yes | "top-level region" | agree |
| 8 | a wonder in a captured city reaches the arm through `Cities::capture_city (00733380)` `(1,1,0)` **if** the wonder type lacks `build_flags & 0x2000` | open (flags not read) | yes, unconditionally | **disagree on scope; settled below** |
| 9 | `Leader::defeat_by (006d1c80)` can hand a wonder over and count it | not followed | yes ("counts again") | one-sided |
| 10 | reset order: `build_game`'s `reset_score` loop (`5ad1e7`) after the per-slot `Leader::init` (`5acef7`) and before `build_empire` | yes | "before goods placement" | agree (both true) |
| 11 | reader: `BuildData::construct_time (0062d5c0)`, the tribe-bonus `0x14` first-wonder arm, `get_wonders()==0 && wonders_built==0`, type not `0x21d`/`0x21e` | yes | yes | agree |
| 12 | no other reader; `0x854(%reg)` in `Build::plunder`/`Object::disband` is off `*(0xc061f0)` | yes | yes | agree |
| 13 | crate: no counter; `Cities::wonders_built` (`cities.rs:63`) is `Game::wonders`, a type bitmask — do not merge | yes | yes | agree |
| 14 | crate site: "`Sim::activate`'s non-city tail beside the wonder test" (first) vs "under `is_wonder_building(ty)` ~line 2490" (second) | non-city tail | line 2490 | disagree |
| 15 | `american_free_wonder` (`city.rs:1827`) is a proxy that parts once a wonder is lost (mark trims, counter never falls) | "a proxy" | yes, names the case | one-sided (the case) |
| 16 | crate: no `"wonders_built"` arm in `checksum/leaders.rs` | — | yes | one-sided |
| 17 | plain game reaches it | yes (capture open) | yes | agree |

### 2. Predicate, re-read instruction by instruction (claims 4, 6, 7)

**Confirmed.** Listing `625ac4..625b5b`: Build vtable compare `625ac9`
(`cmpl $0xb42174`), type vtable compare `625ad8` (`cmpl $0xb42b94`), type
index `0x4(%ecx)` against `0x20e` (`625ae2`, `jl` out to `626f11`) and
`0x21f` (`625aed`, `jl` in to `625b05`); the fallback is `call *%edx` with
`edx` = Build slot `+0x2c` (`625ac6`) or type slot `+0x1c` (`625ad5`), and
`je 0x626f11` on zero (`625aff`). The PE's own vtable bytes put
`TypeData::is_wonder_type (00470780)` at `0xb42b94 + 0x1c`, and its body is
`0x20d < type < 0x21f`, so both branches name the same 17 types (Pyramids
`0x20e` .. Space Program `0x21e`). Between `625b05` and the `incl` at
`625b4b` the only tests are the console-player JukeBox block
(`625b10..625b2d`, owner against `*(0xc06210)+0x298`, and `0xecba20`), which
falls through to `625b32` either way. **The first read of `param_1`
(`0x8(%ebp)`) after the arm is `625b6d`, after `init_wonder` returns.**
Deciding line: `625b05..625b4b`.

Reachability: `Build::activate` has one `ret` (`62710d`, `retl $0xc`), no
indirect `jmp`, and every branch from below `625a45` into the tail lands
**on** `625a45` itself (six: `6258dd`, `6259e6`, `6259fc`, `625a16`,
`625a30`, `625a34`); nothing enters `625a46..625b4b` from elsewhere. The
decompile agrees: one `return` (line 2032), the label `LAB_00625a45` at
line 1212, the increment at 1265. So **every activation of a wonder-type
building, the city branch included, increments `wonders_built[owner]`**.
Deciding line: `62710d` and the jump scan above.

### 3. Claim 8, does a captured wonder credit the captor — **confirmed, and narrowed to 15 of the 17 wonders**

The settling evidence, in order:

1. **The member loop's gate.** `Cities::capture_city (00733380)` walks the
   old city's member chain (`+0x74`) and converts a member only when
   `testl $0x2000, 0x2c0(type)` is zero (`7337cb`); it then skips a
   FARM (`0x1a1`, `733814`) or GRANARY (`0x1a7`, `733850`) taken by a leader
   with tribe bonus `0x13` (`733877`), and, for an **active** member
   (`flags & 4`), calls `swap_team(A)` (slot `+0x94`, `7338da`) and
   activates the new object with pushes `0, 1, 1` → `activate(1, 1, 0)`
   (`733902..73390a`). Decompile lines 259–301.
2. **The wonders' `build_flags`.** In the shipped `buildingrules.xml`, 15
   wonders carry `BUILD_FLAGS` = `a` alone (bit `0x1`): Pyramids,
   Colossus, Hanging Gardens, Terra Cotta Army, Colosseum, Temple of Tikal,
   Porcelain Tower, Angkor Wat, Versailles, Statue of Liberty, Kremlin, Taj
   Mahal, Eiffel Tower, Supercollider, Space Program. **The Forbidden City and the
   Red Fort carry `ean`** (`0x10 | 0x1 | 0x2000`). Letter `n` is bit 13 =
   `0x2000` (`docs/CITIES.md` §1.5 table). No derived bit is `0x2000`, and
   the loaded words were held to the program's own `BuildType::log_data`
   dump, 129 of 129 (`docs/DATALAYER.md` "The derived words no column
   carries"). So the gate passes for those 15 and fails for these 2.
3. **Wonders are members.** None of the 15 carries `e` (`0x10`), so each
   must be placed inside a city's radius (`docs/CITIES.md` §1.5, `e` row)
   and `Build::find_city (00628180)` → `BuildTypeData::get_town
   (00639b90)` → `Build::add_to_city (00622380)` links it into the
   city's `+0x74` chain. `add_to_city` excludes only a city object
   (`flags & 0x20`), not any type (decompile lines 13–28).
4. **The increment has no gate** (§2).

So a finished (active) wonder of those 15 types in a captured city is
`swap_team`ed to the captor and activated with `captured = 1`, and the
captor's `wonders_built` rises by one (and `init_wonder` runs for the
captor, so `wonders_held` may rise too). An **unfinished** wonder site is
not active and is closed or disbanded instead, so it does not count. The
first reading's condition was the right question; the answer is "yes, for
every wonder except the two `n` ones".

The two exceptions:

- **The Forbidden City** (`0x213`, `FROM` Small City, so
  `BuildTypeData::is_city`, `docs/CITIES.md` §1.5 lineage paragraph) is a city object
  itself, so it is never a member. When its city falls, `capture_city`'s
  head swaps the city building and activates it `(1, 1, 0)` at `733687`
  (pushes `73367f..733683`), and `Build::activate`'s city branch still
  reaches `625a45` (§2). **The captor is credited for a captured Forbidden
  City too**, by the head rather than the member loop.
- **The Red Fort** (`0x216`, `FROM` Fort) carries `n`, so the member loop
  leaves it alone. It carries `e`, so `City::find_buildings (007384c0)`
  will not adopt it from a foreign owner (decompile lines 77–82: a foreign
  object must lack `0x10`). `BuildData::check_capture_eligible (0062d1d0)`
  admits city types only. `Leader::defeat_by (006d1c80)` hands over only
  military trainers near a victor's city (lines 145–190). No capture route
  reaching a Red Fort was found; **open** whether any exists (a sweep of every
  `0x1a8` caller's predicate against `is_fort` would settle it).

Deciding lines: `7337cb`, `73390a`, `733687`, `625b4b`; the XML rows.

### 4. Claim 9, `defeat_by`

**Corrected (for the second).** `Leader::defeat_by`'s one activation
(decompile line 190, `activate(0)`) sits under `!is_fort`, not `0x1b7`, not
`0x209`, not a city, `is_military_trainer` and a victor city at the spot
(lines 145–171). A wonder is no military trainer, so `defeat_by` does not hand one
over. A wonder changes hands on defeat only through whatever city capture
the defeat path runs (`docs/CITIES.md` §7.3 step 4's "self/ally
hand-over"), which is claim 8's route. Deciding line: `defeat_by
(006d1c80)` line 164, `is_military_trainer`.

### 5. Claim 14, the crate site

**Corrected (for the first); the second's site is the right one.**
`Sim::activate` (`crates/sim/src/cities/city.rs:2420`) tests
`self.is_wonder_building(ty)` at line 2490, **before** the
`building_is_city` split (line ~2506). The original's increment is reached
on the city path too, and the Forbidden City is a city type and a wonder.
The counter belongs beside line 2490, for `captured` true and false alike,
not in the non-city tail. The crate's capture member loop
(`crates/sim/src/cities/capture_view.rs:156–170`) already gates on
`not_civilian` (`0x2000`) and activates through `Sim::activate` with
`captured = true`, so a counter at line 2490 reproduces claim 8 with no
further work. `is_wonder_building` asks the tech tree's `wonder` kind
rather than the type range. That is equivalent only if the tree files exactly the 17
types `0x20e..0x21e` as wonders (not re-derived here).

### 6. Claim 15, the American proxy

**Confirmed (the second's case), with the capture case added.**
`american_free_wonder` (`city.rs:1827`, tail `~1855`) reads
`census.wonder_mark == 0` and no standing active wonder. The original reads
`get_wonders() == 0 && wonders_built == 0`. The two part when a leader has
**ever** activated a wonder (built or captured) and now holds none: the
census mark trims back to 0 once the last wonder closes
(`Sim::note_wonders`, `ai/ai_census.rs:1276`), whereas `wonders_built`
never falls before the next `reset_score`. The original then refuses the
free-wonder arm and the crate grants it. A captured wonder is itself seen
by the crate's census (it derives slots from active owned wonders), so
capture alone does not part them; capture followed by loss does.
Deciding line: `construct_time (0062d5c0)` line 41.

### 7. Claims 3, 10, 16

- 3: **confirmed in shape.** `walk_data`'s first range is `[&who,
  +0x692a)` handed to the walker's slot 0 (decompile line 17), and
  `param_1->input` selects the load fixups (lines 28–47). A loading walker
  overwrites the range. This is save/restore only, not a plain game.
- 10: **confirmed.** `5acef7` (`Leader::init`) precedes `5ad1e7`
  (`reset_score`), and both precede `build_empire` (decompile lines 812,
  841). There are only two `call 0x6e37f0` sites in `.text`: `5ad1e7` and `6e3b1a`
  (inside `Leader::init`).
- 16: **confirmed.** `checksum/leaders.rs:175` lists `("wonders_built",
  0x854, 4, 1)`, but `leader_long` (`:949`) has no arm for it, nor for any of
  this family's seven leaves.

### 8. Writer completeness

Listing, absolute: `0xe3abe4 + k·0x6eec` for k = 0..9 hits only `625b4b`.
Register-relative stores to `0x854(%reg)` come from `Leader::reset_score`
(`6e3883`) and from non-leaders (`Constants::init` `56dfae`, `Game::loop`
`5916e4`, `CP2PAuthPlayer`). Export: `field_0x854`/`wonders_built` appear in
`Build::activate`, `reset_score` and `construct_time`, and nowhere else. **Complete.**

### 9. The rule as a builder should port it

- **Writer:** `wonders_built[owner] += 1` on **every** `Build::activate` of
  a building whose type index is in `0x20e..=0x21e`, with no test of
  `captured`, `announce` or `counted`, after `remove_unbuilt_wonder` and
  before `init_wonder`.
- **What reaches it in a plain game:** construction (`Wall::do_construct`
  `(0,1,1)`); **capture of a city**, which counts every active member wonder
  of the 15 `a`-only types (`capture_city`'s member loop, `(1,1,0)`) and a
  Forbidden City that is itself the captured city (the head, `(1,1,0)`).
  The Red Fort is not counted on capture.
- **Reset:** `0` in `Leader::reset_score` (every `Leader::init`, and
  `Setup::build_game`'s loop over seated slots before `build_empire`);
  `Leaders::clear`.
- **Crate site:** a per-leader `i32` incremented in `Sim::activate`
  beside `is_wonder_building(ty)` (`cities/city.rs:2490`), outside the
  city split. Zero it at setup. Feed `american_free_wonder` from it, and add a
  `"wonders_built"` arm to `checksum/leaders.rs`.
- **Confidence:** high. The increment and its missing gate are listing-read.
  The capture route is listing-read at `7337cb`/`73390a`/`733687`, and the
  flags are the shipped file's, held to the program's loaded words by an
  existing guard. **Open:** a capture route to a Red Fort (none found);
  `is_wonder_building`'s equivalence to the type range.

## Family-wide claims (they apply to every leaf below)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| F1 | `Leader` = `LeaderData` at +0 plus 8 tail bytes; slot `i` at `0xe3a390 + i·0x6eec` | yes | yes | agree | confirmed (`Fort::init`'s `incl 0xe3ac80(%eax)` = `+0x8f0`, `73eaea`) |
| F2 | the leaders array holds **8** slots, ending `0xe71af0` (first) / **10** (second) | 8 | 10 | disagree | **corrected for the first**: `types.txt` line 42082 `struct Leaders // size 0x4554c`, `+0x0 Leader[10] list`. Deciding line: that record. |
| F3 | `Leaders::clear (006ed5d0)` memsets `[+0, +0x6929)` of every slot | yes | yes | agree | confirmed (decompile line 13, loop to `0xe864dc`, line 69) |
| F4 | `Leaders::clear`'s callers: `Game::close`, `Game::run_oos_recovery`, **plus** CtW's `ConquestGame::advance_round`, `ConquestLeaders::clear_no_attacks` (first) | 4 callers | 2 callers | disagree | **corrected for the first**: `.text` has two `call 0x6ed5d0`, at `586dd6` (`run_oos_recovery`) and `594faa` (`Game::close`), and the export names no other caller. Deciding line: those two sites. |
| F5 | `Leader::close (006b7f80)` memsets `0x6929` too; no direct caller | — | yes | one-sided | confirmed: body line 11 `memset(this,0,0x6929)`; `0x6b7f80` occurs nowhere in `.text`, so any call to it is virtual |
| F6 | `LeaderData::walk_data` hands `[+0x8, +0x692a)` to the walker, and on load overwrites it | read only | load too | one-sided | confirmed in shape (decompile line 17, `input` fixups at 28–47); save/restore, not a plain game |
| F7 | `ScenarioRead::load_leaders_chunk (009acde0)`'s `0x428/0x430/0x434(%edx)` stores are off the `Game`, not a leader (first) / base not read (second) | Game | open | one-sided | **confirmed for the first**: `9acde6 movl 0xc061ec,%edx` (`GameAccess::game`), then `testb $0x4,0x823(%edx)` / `testb $0x8,0x821(%edx)` = the decompile's `game->semaphore` tests (lines 27–28) |
| F8 | the non-leader register-relative hits (`Constants::init`, `GraphicWinXML`, `PlayerProfile`, `ConquestLeader`, `Game::loop`, `TileSet`, `RiverOut`, `EventGroup`, `GraphicPieces`, `ObjectsOut`, `CP2PAuthPlayer`, `GameSpyGroup`, `String` off `%esp`) | yes | yes | agree | confirmed: my own net over `mov/inc/dec/add/sub/or/and/xor` into `0x428|0x430|0x434|0x854|0x858|0x85c|0x8f0(%reg)` returns exactly those plus `Leader::init` and `Leader::reset_score`; `ConquestGame::advance_round 7905a0` and `ConquestGame::close 79b233` are CtW |
| F9 | crate: `checksum/leaders.rs` lists all seven names (`:105–108`, `:175–177`, `:189`) but `leader_long` (`:949`) has no arm for any | — | yes (per leaf) | one-sided | confirmed |

## Leaf `fort_mark` (+0x428)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers `Forts::init_fort (0073ed70)` (`73edee`, `73ee84`), `Forts::close_fort (0073ec90)` (`73ed51`), `Leader::init` (`6e4136`), `Leaders::clear` | yes | yes | agree | confirmed (absolute net `0xe3a7b8 + k·0x6eec`: those three stores only) |
| 2 | take: first free under the mark; else the allocated record at the mark (mark+1); else append; then `mark = max(mark, idx+1)` | yes | yes | agree | confirmed (`init_fort` decompile: search, `field_0x4` count test, malloc arm, `LAB_0073ee6e`) |
| 3 | step on a take: "0 or +1" (first) / "+1 or more" (second) | 0/+1 | +1 or more | disagree | **first confirmed**: the tail arm takes `idx = mark` and the append arm runs only when `mark ≥ count`, so `idx + 1 ≤ mark + 1`; a take never raises it by more than 1. A close can lower it by several. |
| 4 | caller `Build::activate` `6245a9`, every activation (no `captured`), `is_fort` = `is(FORTX 0x1bb, 0)` by type slot `+0xfc` | yes | yes | agree | confirmed: PE dword at `0xb42b94 + 0xfc` = `00472ba0` (`BuildTypeData::is_fort`) |
| 5 | close: `Build::close (00628980)` `628ce7` when `is_fort` and `+0x78 ≥ 0`; `Region::fix_borders`, `Fort::close`, trim | yes | yes | agree | confirmed (`Build::close` decompile lines 211–216) |
| 6 | reset in `Leader::init` sits after the `tribe < 0` return ("only when tribe ≥ 0" / "unconditional, after the return") | yes | yes | agree | confirmed, same fact: `return 0` at lines 321–324, zeroing at 370–372 |
| 7 | not in `Setup::build_game`'s unrolled block | yes | yes | agree | confirmed (`5ac35f..5ac4e5` writes `+0x408/+0x420/+0x434/+0x43c` only) |
| 8 | readers: `find_fort`, `compute_reg_territory`, `Army::find_target`, `Object::do_launch`, `Leader::produce_unit`, `blocked_location`, `log_data` (both; second adds `init_fort`'s own search) | yes | yes | agree | confirmed (absolute-net reads at `64fa76`, `64fb9e`, `6b1552`, `6b16d8`, `6f7fbb`, `6f81db`, `73ebf2`) |
| 9 | crate: no registry; template `Sim::dock_open`/`dock_close` (first) or `units::slots::SlotList` + give-back beside `dock_close` `city.rs:2829` (second) | dock | SlotList | agree in substance | both exist; **`SlotList` (`units/slots.rs:24`) is the exact shape** (in-use bits plus a mark, take/give-back as above) and carries no extra fields |
| 10 | plain game: yes | yes | yes | agree | confirmed |

**Rule.** `fort_mark[who]`: an index high-water mark over the leader's
fort records. Take, on every `Build::activate` of an `is(FORTX)` type
(captured too): reuse the first free record under the mark, else the record
at the mark, else append; then `mark = max(mark, idx + 1)` (+0 or +1). The
index is stored at `Build +0x78`. Give back, in `Build::close` when
`is_fort` and `+0x78 ≥ 0`: clear the bit, then walk the mark down past free
tops. Reset: `Leader::init` (seated slots only), `Leaders::clear`. Crate:
a per-leader `SlotList`, taken in `Sim::activate` beside the `is_fort` test
(`cities/city.rs:2533`), given back in `Sim::close_building`'s fort arm
(`~2905`). Plain game: yes. Confidence high; nothing open.

## Leaf `oil_well_mark` (+0x430)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers `init_oil_well (0073f6f0)` (`73f76e`, `73f7fa`), `close_oil_well (0073f670)` (`73f6d1`), `Leader::init` (`6e4142`), `Leaders::clear` | yes | yes | agree | confirmed (absolute net: those three stores) |
| 2 | take rule as `fort_mark`; caller `Build::activate` `625a9f` on `is(OILWELL 0x1a5)` or `is(OILPLATFORM 0x1a6)`, no `captured` test | yes | yes | agree | confirmed (`625a45..625a9f`) |
| 3 | close gate: `Build::close` `628daf` needs `is_gather_type` (type slot `+0x90`) **and** well/platform **and** `+0x78 ≥ 0` (first) / "same `+0x78 ≥ 0` guard" (second) | yes | omits `+0x90` | one-sided | **first confirmed**: `Build::close` decompile lines 244–272 test slot `+0x90` first; PE dword at `0xb42b94 + 0x90` = `00472bb0` (`BuildTypeData::is_gather_type`, `build_flags & 0x40`); both well (`gdae`) and platform (`igbe`) carry `g` in `buildingrules.xml` |
| 4 | what close writes into the record: live bit, `who = 0xff`, `o = 0xffff` (first) / live bit, region 0 (second) | partial | partial | disagree | **both incomplete; the full set**: `+6 &= ~1`, `+7 = 0xff`, `+2 = 0xffff`, `+4 = 0` (`close_oil_well` decompile lines 9–12) |
| 5 | reset: `Leader::init` (after the return), `Leaders::clear`; not `build_game`'s block | yes | yes | agree | confirmed |
| 6 | readers: `Leader::calc_gather (006ceee0)` (loop bound, order-free sum), `log_data` | yes | yes | agree | confirmed (`73f6aa`, `73f71c`, ... are the registry's own; `log_data` line 817) |
| 7 | crate: none; wells found by kind in `economy/holdings.rs` | yes | yes | agree | confirmed |
| 8 | plain game: yes, Industrial onward | yes | yes | agree | confirmed |

**Rule.** As `fort_mark`, on the oil-well records: take on every
activation of an `is(OILWELL)` or `is(OILPLATFORM)` building (captured too),
give back in `Build::close` when `build_flags & 0x40` and the type is a well
or platform and `+0x78 ≥ 0`. Crate: a second `SlotList`, same two sites.
Plain game: yes. Confidence high; nothing open.

## Leaf `supply_mark` (+0x434)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers `init_supply (0073ad40)` (`73adbe`, `73ae51`), `close_supply (0073acc0)` (`73ad21`), `Leader::init` (`6e4148`), `Leaders::clear` | yes | yes | agree | confirmed |
| 2 | `Setup::build_game (005ac190)` zeroes it unconditionally in an unrolled block with `+0x408`, `+0x420`, `+0x43c`: **8** slots (first) / **10** slots (second) | 8 | 10 | disagree | **corrected for the first**: ten stores, `5ac373` .. `5ac4db` (the last two at `0xe71f24` and `0xe78e10`, slots 8 and 9), matching `Leader[10]` (F2). Deciding line: `5ac4b3`, `5ac4db`. |
| 3 | callers: `Unit::init` `612943` when `is_supply`; `Unit::set_type` `613796` when supply and `+0x84 < 0`; close from `Unit::close` `60f7cd` (supply, `+0x84 ≥ 0`) and `set_type` `613750` (not supply, `+0x84 ≥ 0`) | yes | yes | agree | confirmed in the listing: `cmpl $0x46ce80` (`UnitData::is_supply`) devirtualisation at `612924`, `613722`, `613767`; `testw 0x84` / `js` at `60f7c4`, `613747`; `cmpw $0,0x84` / `jge` at `61378a` |
| 4 | `is_supply` = `unit_flags2 & 0x40` | yes | yes | agree | confirmed (agreed, not re-read beyond the address) |
| 5 | readers: `find_supply (0073aba0)` (`73abb7`, `73ac94`), via `UnitData::in_supply`, decision-changing; `log_data` | yes | yes | agree | confirmed |
| 6 | crate: `SupplyList` (`territory/supply.rs:171`), `len()` (`:207`) **is** the mark because it pops free tails; opened `units/store.rs:145`, closed `territory/upkeep.rs:62`; `set_type` pair stubbed (`units/set_type_adapter.rs:207–211`) | yes | yes | agree | confirmed (sites exist as cited) |
| 7 | plain game: yes | yes | yes | agree | confirmed |

**Rule.** As `fort_mark`, on the supply records, taken at the birth of a
unit with `unit_flags2 & 0x40` (and on a `set_type` into such a type with
no slot held), given back at its close (and on a `set_type` out of it).
Reset also by `Setup::build_game` for all ten slots before any unit
exists. Crate: already modelled; expose `territory.supply[who].list.len()`.
Plain game: yes. Confidence high. Open, from both readings: whether a plain
game ever retypes a unit across the `0x40` flag.

## Leaf `wonders_held` (+0x858)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers: `Wonders::init_wonder (0073c860)` (`73ca11` read, `73ca1f` store), `reset_score` (`6e3889`), `Leaders::clear`; `close_wonder` writes none of it | yes | yes | agree | confirmed (absolute net: `73ca11`/`73ca1f` only) |
| 2 | `wonders_held = max(wonders_held, live entries in [0, wonder_mark))`, the new one included; never lowered | yes | yes | agree | confirmed (`init_wonder` decompile lines 87–103: count `+0xc & 1` over `field_0x424` records, then max) |
| 3 | one caller, `625b5b`, right after the `wonders_built` increment, so the same predicate and no `captured` gate | yes | yes | agree | confirmed (only `call 0x73c860` in `.text`) |
| 4 | reader: `AchieveChartPane::set_chart_data (007f2c40)` line 233, UI only; checksum | yes | yes | agree | confirmed; `log_data` does not print it |
| 5 | crate: none; `Sim::wonders_held` (`economy/holdings.rs:572`) is a type bitmask; site `Sim::note_wonders` (`ai/ai_census.rs:1276`) | yes | yes | agree | confirmed |
| 6 | caveat: `note_wonders` batches closes then opens, which can under-count against the original's event order | — | yes | one-sided | confirmed by reading `note_wonders` (closes over every slot first, then opens); a builder who wants exactness counts at `Sim::activate`'s wonder test instead |
| 7 | plain game: yes; capture follows `wonders_built`'s answer | yes (capture open) | yes | agree | **settled by `wonders_built` §3**: a captured wonder of the 15 `a`-only types, or a captured Forbidden City, runs `init_wonder` for the captor |

**Rule.** At every wonder activation (built or captured), after the slot is
taken, count the leader's in-use wonder records under `wonder_mark` and keep
the maximum. Reset: `reset_score`, `Leaders::clear`. Readers: UI and the
checksum only. Crate: per-leader `i32`, raised where `init_wonder` is
modelled. Plain game: yes. Confidence high. Open: `Wonder::get_timer`
(it does not touch this leaf).

## Leaf `senates_built` (+0x85c)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers `Build::activate` `6246ec`, `reset_score` `6e388f`, `Leaders::clear` | yes | yes | agree | confirmed (absolute net: `6246ec` only) |
| 2 | predicate: non-city arm; the `is` chain GRANARY, LUMBERMILL, SMELTER, REFINERY, TEMPLE, MARKET, then SENATE `0x1b6` | yes | yes | agree | confirmed (agreed; listing push at `00624627` per the first) |
| 3 | `+= 1` iff `+0x72 ≥ 0` and `captured == 0` (`6246cf` `cmpw $0,0x72(%esi); jl`, `6246da` `testl %edi,%edi; jne`), `edi = param_1` | yes | yes | agree | **confirmed, with the register traced**: `edi` is loaded from `0x8(%ebp)` at `6241fd`. Its only other writes before `6246da` are `6244ff` (`movl 0xc06188,%edi`, the temple arm) and `624561` (`movl 0x8(%ebp),%edi`, which restores `param_1` before the shared join `624564`). The call at `6246ca` cannot clobber it (callee-saved). |
| 4 | the `(0,1,0)` transfer routes would count a Senate | yes | yes | agree | confirmed as a predicate; whether a Senate travels them stays open (a Senate lacks `e`, so `find_buildings` could adopt a cityless foreign one; `check_capture` only takes cities; `defeat_by` only trainers) |
| 5 | reader: `log_data` line 906; checksum | yes | yes | agree | confirmed |
| 6 | crate: `senate_arm_steps` (`cities/activate.rs:251`, call `:261`), `SimArms::senates_built_inc` (`:394`) an empty SEAM | yes | yes | agree | confirmed |
| 7 | plain game: yes | yes | yes | agree | confirmed |

**Rule.** `senates_built[owner] += 1` on a non-city activation of a
SENATE-lineage building with city index `≥ 0` and `captured == 0`.
`announce` and `counted` are ignored. Reset `reset_score`, `Leaders::clear`.
Crate: fill `SimArms::senates_built_inc`. Plain game: yes. Confidence high
(listing-read and sweep-backed per the first). Open: whether a Senate
travels a `(0,1,0)` transfer in a plain game.

## Leaf `forts_built` (+0x8f0)

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | writers: `Fort::init (0073eac0)` `73eaea` `incl 0xe3ac80(%eax)`, `reset_score` `6e38b9`, `Leaders::clear` | yes | yes | agree | confirmed (absolute net: `73eaea` only) |
| 2 | `Fort::init` increments unconditionally; its one caller is `init_fort` (`73ee93`), whose one caller is `Build::activate` `6245a9`; no `captured` gate | yes | yes | agree | confirmed (`Fort::init` decompile: the increment precedes the region work and has no condition) |
| 3 | `Fort::init` also bumps the per-region short at `+0x12de + region·2` when `region < 0x40` (another family's leaf) | yes | yes | agree | confirmed |
| 4 | the Red Fort steps both `forts_built` and `wonders_built` "if it is in the FORTX lineage" (first) | conditional | — | one-sided | **confirmed**: `buildingrules.xml` gives the Red Fort `FROM` Fort, so `is(FORTX, 0)` admits it, and its type `0x216` is in the wonder range |
| 5 | an upgrade in place (Fort → Castle) does not count (first) | yes | — | one-sided | confirmed as far as the writer net reaches: `init_fort` has one caller, so only an activation counts |
| 6 | a fort taken with a city counts again for the captor (second) | — | yes | one-sided | **corrected**: a fort carries `n` (`ean`/`ecan`), so `capture_city`'s member loop skips it (`7337cb`). It carries `e`, so `find_buildings` will not adopt it from a foreign owner. `check_capture_eligible` admits city types only, and `defeat_by` excludes `is_fort` (line 145). **No capture route reaching a fort was found**, so the predicate has no gate but no plain capture path feeds it. |
| 7 | readers: `AchieveChartPane::set_chart_data` line 235, UI only; not `log_data` | yes | yes | agree | confirmed |
| 8 | crate: none; site beside `is_fort` in `Sim::activate` (`city.rs:2533`) | yes | yes | agree | confirmed |
| 9 | plain game: yes | yes | yes | agree | confirmed |

**Rule.** `forts_built[who] += 1` per `Fort::init`, which means per
`Build::activate` of an `is(FORTX)` building (Fort line and Red Fort), with
no gate. Never decremented. Reset `reset_score`, `Leaders::clear`. Crate:
increment with the `fort_mark` take in `Sim::activate`. Plain game: yes,
by construction. Confidence high. Open: any route that activates a fort for
a new owner (none found).

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| family-wide | 6 / 2 / 0 | `Leaders` is `Leader[10]`; `Leaders::clear` has two callers (`Game::close`, `run_oos_recovery`); `load_leaders_chunk`'s stores are the `Game`'s |
| `fort_mark` | 9 / 0 / 0 (one disagreement settled for the first) | high-water mark of fort records: take ≤ +1 on every `is(FORTX)` activation, trim on close with `+0x78 ≥ 0`; `SlotList` |
| `oil_well_mark` | 7 / 1 / 0 | same, on well/platform activation; close also needs `build_flags & 0x40`; close writes bit, `who`, `o` and region |
| `supply_mark` | 6 / 1 / 1 | same, on supply-unit birth/death/retype; `build_game` zeroes all **ten** slots; crate's `SupplyList::len()` is it |
| `wonders_built` | 13 / 2 / 2 | +1 on every activation of type `0x20e..0x21e`, no gate; **a captured city's active wonders credit the captor** (15 `a`-only types via the member loop, the Forbidden City via the head; not the Red Fort); crate site `city.rs:2490` |
| `wonders_held` | 7 / 0 / 0 | max(held, in-use wonder records under `wonder_mark`) at each `init_wonder`, capture included |
| `senates_built` | 7 / 0 / 1 | +1 per non-city Senate activation with a city and `captured == 0` |
| `forts_built` | 8 / 1 / 1 | +1 per `Fort::init` (every FORTX activation, Red Fort included); no capture route reaches a fort |

The most important correction: the second reading's "a fort taken with a city
counts again for the captor" (`forts_built`) does not hold. Forts carry
`build_flags & 0x2000`, which `capture_city`'s member loop exempts. Its
companion claim for wonders does hold, for every wonder but the Red Fort.
