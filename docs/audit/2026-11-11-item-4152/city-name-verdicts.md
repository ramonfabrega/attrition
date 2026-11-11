# Item 4152, adjudication: `city-name`

- Slug: `city-name`
- Leaf: `city_name` (`LeaderData` +0x40c, int)
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11
- Inputs: `city-name-reading.md` (first), `city-name-second.md` (blind second), the decompile export (`~/ghidra-projects/decomp`), the `.text` listing (`exe.s`), `crates/sim`.
- Written per claim as settled; a leaf without its closing rule below was not finished.

## Leaf `city_name` (`LeaderData` +0x40c, int)

### Layout

Both readings: `Leader` holds `LeaderData` at offset 0, so `field_0x40c` on a
`Leader*` is `city_name`, and slot `i` is `0xe3a79c + i·0x6eec`. **Agree,
confirmed**: `types.txt` 42088-42092 (`LeaderData` size 0x6ee4, `who` +0x8,
`tribe` +0xc), and `Leaders::clear (006ed5d0)` memsets from
`&leaders.list[0].field_0x6c14 - 0x1b05·4`, which is the record's own byte 0.

### 1. Claims lined up

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| store `+= 1` on the winner slot | 73614a | 73614a | agree | confirmed | `incl 0xe3a79c(%eax)` at 73614a; `generate_name:229-230` |
| store `= 1` on own when own is 0, first-name branch | 7362d2 | 7362d2 | agree | confirmed | `movl $0x1,0xe3a79c(%eax)` at 7362d2 under `cmpl $0x0` at 7362c9; `generate_name:261-263` |
| `Leader::init` clears to 0 | 6e3b14 | 6e3b14 | agree | confirmed | `Leader::init:139`, listing 6e3b14 |
| `Leader::init` also memsets `[0, 0x6929)` when `param_2 < 0` | not made | made | one-sided | **confirmed** (redundant with the explicit 0) | `Leader::init:70-77`; pushes `0x6929, 0, %ebx` then call at 6e398c-6e3995 |
| `Leaders::clear` memsets the record | not made | made | one-sided | **confirmed**; callers `Game::close (00594b80)` 594faa and `Game::run_oos_recovery (005869e0)` 586dd6 | `Leaders::clear:12` |
| `LeaderData::walk_data` restores it on a save-game load (raw range `[+0x8, +0x692a)` when `leader_flags & 1`) | not made (said a load restarts at 0, for the *scenario* path) | made | one-sided | **confirmed**; the first's claim stands only for `ScenarioRead`, which both agree drops the value | `walk_data:16-17` (`&this->who` to `&this->field_0x692a`) |
| the sync checksum reads it through that same walk | not made | made | one-sided | **confirmed** | `CheckSums::check_one_leader:5` is `walk_data` on `leaders.list + param_2` |
| absolute `0xe3a79c` sites | 8 (7 in `generate_name`, 1 in `save_leaders_chunk`) | "4" in `generate_name` plus the save | disagree (count) | **corrected to the first's 8**: 736015, 73601d, 736144, 73614a, 736150, 7362c9, 7362d2, 9a791e. Both have the same two stores, so no writer moves | grep of `exe.s` |
| `Leader::init` call sites in `ScenarioRead::load_leader_chunk` | 3 | 6 | disagree | **corrected to 6**: 9ac1a9, 9ac24f, 9ac3bc, 9ac4c7, 9ac59c, 9ac69f. Same function either way; no rule moves | `call 0x6e3930` grep |
| `0x40c(%reg)` stores outside the Leader family are other classes | listed | listed | agree | confirmed. My own sweep (two-operand and single-operand stores, x87 and string ops) finds 12 sites; only 6e3b14 is a `Leader`. 9ad098 is the `GameInfo.player[]` clear (stride 0x8c run, `movw` to +0x414 at 9ad0a4) | `exe.s` 1768470-1768505 |
| no whole-record copy | made | made (adds the memsets and walk) | agree + one-sided | confirmed, with the three bulk writers above as the complete bulk set | no `$0x6ee4`/`$0x1bb9` operand; the memsets and the walk are what remain |

**Writer completeness.** I found no writer that neither reader named. The
second reader's list is the complete one: two value stores in
`City::generate_name (00735c90)`, the clear in `Leader::init (006e3930)`, and
the three bulk writers (`Leader::init`'s memset, `Leaders::clear`,
`LeaderData::walk_data` in input mode).

### 2. Event and predicate claims

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| `generate_name`'s only caller is `City::init`, under its 4th argument 0 | made | made | agree | confirmed | the single `call 0x735c90` at 737409; `City::init:202-203` |
| `City::init`'s only caller is `Cities::init_city`, which passes its own 3rd argument through; its only caller is `Build::activate`, which passes its own first argument | made | made | agree | confirmed | `init_city:67-68`; listing 623fba-623fc7 pushes `%ebx, 0x8(%ebp), type, owner` |
| that call is in the city branch (`Build+8 & 0x20`) | `else` at activate:446 | `0x20` split | agree | confirmed: the `else` at 446 pairs with `if ((this->field_0x8 & 0x20) == 0)` at activate:160 | `activate:160`, `:446`, `:479` |
| a city activated with first argument 1 (capture, `switch_teams`) takes no step | made | made | agree | confirmed | `Cities::capture_city (00733380)` pushes `0,1,1` (733681-733683 and 733904-733906) |
| `check_capture`, `defeat_by`, `find_buildings` and `cast_bribe` reach `activate` only on non-city buildings | made (all four) | made (three; not `cast_bribe`) | agree + one-sided | **confirmed**, including `cast_bribe`: its city arm (`:103`) either calls `capture_city` (`:154`) or jumps to `LAB_00672e2d`, which refunds and returns (`:83-86`) | `check_capture:257`, `defeat_by:160`, `find_buildings:76`, `cast_bribe:103-158` |
| the `activate` caller list is complete | made, by grep of `+ 0x1a8))(` | made | agree | **confirmed**: every `call *0x1a8(%reg)` in `.text` (17 sites, 14 functions) plus the register-indirect loads in `ConquestGame::build_cities` (2) and `ScenarioRead::load_city_chunk` (1); no direct `call 0x623e20` exists | `exe.s` grep |
| `Setup::build_cities` activates the start city `(0,0,0)` when `starting_town` is set and `leader_flags2 & 0x80` is clear | made (the predicate in full) | made (no predicate) | agree + one-sided | confirmed | `build_cities:29-30`, push run 5abab5-5ababf |
| `Wall::do_construct` activates `(0,1,1)` | made | made, "may be folded" | agree + one-sided | **confirmed, not folded**: 006434d0 is its own INDEX row and the call is at do_construct:113 | listing 64365d-643663 |
| `Leader::produce_building` reaches `activate` only at frame 0 | made | not made | one-sided | confirmed: the non-zero-frame arm swarms builders and jumps past the call | `produce_building:1060-1086`, call at `:1111` |
| `produce_building`'s Setup callers may place a city building at frame 0 | not made | raised, open | one-sided | **settled: no.** The types they push are WOODCUTTER 0x1a2, FARM 0x1a1, TOWER 0x1b7, LIBRARY 0x1b3 (`large_`/`small_city_buildings`) and 0x1b4, 0x1a4, 0x1b5, 0x1a7, 0x1a8, 0x1b6 (`build_civ_specific`); none is VILLAGE 0x19e, TOWN 0x19f, METROPOLIS 0x1a0 or FORBIDDENCITY 0x213 | push runs 5aacbf-5aadf6, 5aae53-5aaf37, 5ab7b2-5ab8fc; `enums/TypeIndex.txt` 444-448, 465, 469, 562 |
| a frame-0 AI `make_this` could place a city | open | open | agree (both open) | **open**, but bounded: `Leader::make_this (006c94f0)` is called only from `make_stuff` and `plan_strategy`. The run-1562 dump (below) shows no extra step by frame 91 on that run. A capture with a computer player whose `city_name` exceeds its city count at frame 1 would settle it | `call 0x6c94f0` grep |
| gate (a): the city's own building `is(0x213)` (FORBIDDENCITY): no step | made | made | agree | confirmed | `generate_name:67-82` |
| gate (a) reachable in a plain game | open | open | agree (both open) | **settled: yes, medium.** The crate already holds that FORBIDDENCITY carries the city flag, so its completion runs the city branch: `crates/sim/src/cities/city.rs:333-336` (`flags & 0x20` is `is(VILLAGE)`), `crates/sim/src/vision.rs:977-980` (FORBIDDENCITY "is `FROM Small City`, so `is(VILLAGE)` holds"). The flag is set from the type at `SubObject::init (00662300):18-20`. I did not re-derive that type predicate | as cited |
| gate (b): CtW (`semaphore[2] & 2`) and a non-barbarian tribe (`Tribe +0x58`): no step | made | made | agree | confirmed. The `if` sends the XML path when the bit is clear **or** the tribe is barbarian, so barbarians under CtW do step | `generate_name:83-85` |
| gate (c): the cities element is missing, or its list is empty: no step | made | made | agree | confirmed | `generate_name:96-145` |
| the scan covers slots 0..7 only, unrolled | made | made | agree | confirmed. The step-6 index loop (`slots < own`) has no bound of 8, as the second reader says; it affects the name, not the counter | `generate_name:146-219`; loop at `:244-256` |
| winner = the slot holding the largest value among own-tribe slots (strict `>`, so a tie keeps the lowest slot) if that value is > 0, else own | made | made | agree | confirmed. The max starts at -1 and the winner index at -1 (`:147-151`), and the `> 0` test is `cmovgl` at 73613b | `generate_name:220-223` |
| no liveness test in the scan | made | not made | one-sided | confirmed: the scan reads `tribe` (+0xc) and `city_name` alone | `generate_name:150-219` |
| unused slots have tribe -1 and never match | made | not made | one-sided | **corrected, immaterial to the counter.** `Leaders::init` does pass -1 (push at 6ed8c6), but the run-1562 dump shows slots 8 and 9 at `tribe 0`. They sit outside the 0..7 scan. In any case an unused slot's `city_name` stays 0, and the winner needs a value > 0, so no unused slot can take the step. Unused slots can move only the name's index counts | dump lines 841437, 853482; `generate_name:220-223` |
| `prev = city_name[own]` is read before the step | made | made | agree | confirmed | 736144; `generate_name:224` |
| branch 6 is `n <= 1 \|\| prev == 0` (signed) | made | made | agree | confirmed | 7361c4 `cmpl $1; jle`, 7361cf `je` |
| the `= 1` store fires only when the winner is not own | implied ("a tribe-mate already holds the shared count") | implied | agree | confirmed by arithmetic: when the winner is own, own is ≥ 1 after the step, so the `== 0` test fails | `generate_name:261-263` |
| step 7's LFSR and modulus: signed `idivl`, arithmetic `sarl`, taps 0x170 | made | made | agree | confirmed; name-only, it writes no `city_name` | 736255-736275 |
| LFSR termination | not made | open (medium-high) | one-sided | **settled, name-only.** 0x170 shifted right once is 0xb8, the maximal 8-bit Galois polynomial: from any non-zero start it cycles all of 1..255. The loop ends whenever `M >= 2`; at `M = 1` (`L == S`) it never ends. A starting value of 0 or below needs `T + seed < 0` | 736261-736275 |

### 3. Reset, readers, crate site, reach

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| reset is `Leader::init` alone, no per-frame fold, never decremented | made | made (adds `Leaders::clear`) | agree + one-sided | confirmed; `Leaders::clear` is a game-close / OOS reset, not a reset inside play | as in 1 |
| `Setup::build_game`'s `Leader::init` runs before the start city's step | open | not made | one-sided | **settled.** `build_game` calls `Leader::init` at 5acef7 for each live slot (`leader_flags & 1`, `build_game:612-618`). It reaches `build_cities` later, through `build_empire` (5ad40b, 5ad4d2). The dump shows 1 after the start city, not 0 | listing addresses; dump 817145 |
| readers: `generate_name`, `log_data` (key `city_name`), `save_leaders_chunk` | made | made | agree | confirmed | `log_data` push at 6e6abf; 9a791e |
| reader: the sync checksum and the save game through `walk_data` | not made | made | one-sided | confirmed (see 1) | `check_one_leader:5` |
| `ScenarioRead::load_leader_chunk` does not read the saved value back | made | made | agree | confirmed, not re-read beyond both readers' listing; each branch calls `Leader::init` | 6 call sites above |
| crate: no counter today; the event is `Sim::activate` (`crates/sim/src/cities/city.rs:2420`) calling `Sim::init_city` (`:451`) at `:2509` in the `building_is_city` branch, and `captured` is `activate`'s first argument | made | made | agree | confirmed. Every `activate(.., true, ..)` outside tests is `crates/sim/src/cities/capture_view.rs:236`; `init_city`'s only caller is `:2509`; the start city enters through `crates/sim/src/setup.rs:94` | grep |
| crate: the checksum leaf is `checksum/leaders.rs:98`, unheld at `rondata/src/diff/checksum_lines.rs:1772`; the held arm goes beside `city_mark` (~1020-1037) | made | made | agree | confirmed (`city_mark` arm at `leaders.rs:1020-1033`) | grep |
| crate: `ai/ai_host.rs:445 city_name` is unrelated (a synthetic AI label) | made | made | agree | confirmed | `ai_host.rs:445` |
| crate: the builder must exempt FORBIDDENCITY by type, because the crate's `building_is_city` is true for it | not made | not made | finding | **finding**: the exemption is load-bearing in the crate, not a dead branch (see gate (a)) | `city.rs:333-336` |
| plain game: start city and every completed city step it; capture never does | made | made | agree | confirmed | as in 2 |

### 4. The value on disk (a witness, not yet a pinned diff)

Run 1562's `@dumpall`
(`gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt`, frame 91)
logs four leader records:

| slot | tribe | `city_mine` | `city_mark` | `city_name` | dump line |
|---|---|---|---|---|---|
| 0 | 4 | 1 | 1 | **1** | 817145 |
| 1 | 11 | 1 | 1 | **1** | 829402 |
| 8 | 0 | 0 | 0 | 0 | 841661 |
| 9 | 0 | 0 | 0 | 0 | 853706 |

The two players have distinct tribes, so the winner is own. Each started with
`starting_town` on, and each has stepped exactly once: for its start city.
That fits the rule above, and fits nothing the readings excluded: no frame-0
AI placement and no double step. The crate emits 0 for both walked leaders.
That is the `("city_name (unheld)", 2)` count at
`crates/rondata/src/diff/checksum_lines.rs:1772`, so a port moves that count
from 2 to 0 on this record. The record exercises only the distinct-tribe path.
The tribe-shared winner and the `= 1` store rest on the listing alone.

### 5. The rule, as a builder should port it

All values are plain `i32` counts, with no scale.

- **Storage.** One `city_name: i32` per leader slot. It is zeroed where the
  crate runs `Leader::init` (game setup, before the start cities). Nothing
  decrements it, and nothing folds it per frame.
- **Event.** `Sim::activate(b, captured, _)`'s city branch, at the
  `init_city` call (`crates/sim/src/cities/city.rs:2509`), when
  `!captured`. That covers the start city through `crates/sim/src/setup.rs:94`
  and every city completed by construction. Capture and team switches
  (`captured == true`) never step it.
- **Exemptions, no step.** The building's type `is(FORBIDDENCITY 0x213)`; the
  game is CtW and the owner's tribe is not barbarian; the tribe's city list is
  missing or empty (the crate has no list, so a port can take it as non-empty
  for every shipped tribe, and record that as an assumption).
- **Step.** Let `own` be the city's owner and `T = tribe[own]`. Over slots
  `k` in `0..8` (never 8 or 9) with `tribe[k] == T`, find the largest
  `city_name[k]`, taking the lowest `k` on a tie. If that largest value is
  `> 0`, the winner is that `k`; otherwise the winner is `own`. Read
  `prev = city_name[own]`, then `city_name[winner] += 1`, and let `n` be the
  winner's new value.
- **Second store.** If `n < L` (the list length) and (`n <= 1` or
  `prev == 0`) and `city_name[own] == 0`, set `city_name[own] = 1`. In
  practice this fires exactly when the winner is a tribe-mate and `own` had
  never named a city. Under the non-empty-list assumption, the `n < L` guard
  needs `L`. Without it, a port either carries `L` per tribe from the data
  layer, or states the assumption `n < L`, since every shipped list is long
  and the numbered-name arm leaves the counter alone anyway.
- **Not needed for the counter:** the LFSR, the name index, the numbered
  suffix.
- **Checksum.** A held arm for `"city_name"` beside `"city_mark"` in
  `crates/sim/src/checksum/leaders.rs` (~1020). The pin at
  `crates/rondata/src/diff/checksum_lines.rs:1772` falls from 2 to 0.
- **Plain game.** Yes: start cities, every completed city, and a completed
  Forbidden City (exempt). The shared-tribe branch needs two leaders of one
  nation.

**Confidence.** High on the writers: both nets were swept, and the bulk
writers were read. High on the step and the predicate: listing 735ff3-7362d2,
and both readings agree. The distinct-tribe path is backed by run 1562's
frame-91 values (two leaders at 1). The tribe-shared winner and the `= 1`
store rest on the listing alone. No dump has two leaders of one tribe.

**Open.**
- Whether a computer player can place a city at frame 0 through
  `make_this` → `produce_building`. It is bounded by run 1562, which shows
  none, but that is only one run.
- Whether `starting_town` is on by default in the lobby.
- The tribe-shared path needs a capture with two same-nation leaders to
  become a diff.
- The type predicate that sets the city flag on FORBIDDENCITY
  (`SubObject::init (00662300):18`) was not re-derived here. It is taken from
  the crate's ratified `is(VILLAGE)` reading.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `city_name` (+0x40c int) | 41 / 3 / 1 (+1 new finding) | On a city's non-transfer activation (start city, construction), unless FORBIDDENCITY / CtW non-barbarian / no list: `+1` to the own-tribe slot in 0..7 holding the largest positive count (lowest on a tie), else to own; then own `0 → 1` if a tribe-mate took the step. Zeroed by `Leader::init` (and the `Leaders::clear` / save-load bulk writers). |

Counting: 46 claim rows (the layout, and the rows of sections 1-3). The new finding is the crate's FORBIDDENCITY exemption being load-bearing. The 3 corrections are the absolute-site count, the
`load_leader_chunk` call count, and "unused slots are tribe -1". None changes
a rule. The 1 open is the frame-0 AI placement. Settled one-sided or open
items, counted as confirmed: the memset, `Leaders::clear`, `walk_data`, the
checksum reader, `cast_bribe`, `do_construct` not folded, the Setup
`produce_building` types, gate (a)'s reach, LFSR termination, and the
`build_game` order.
