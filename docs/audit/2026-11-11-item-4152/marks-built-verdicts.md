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

