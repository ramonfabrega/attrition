# Item 4152, family 10 (`end-of-game`): adjudication

- Slug: `end-of-game`. Leaves (`LeaderData`): `defeated_by` (+0x10),
  `wonderwin_stamp` (+0x448), `wonderwin_timer` (+0x44c), `victory_type`
  (+0x7d8), `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924); all int.
- Readings: `end-of-game-reading.md` (first), `end-of-game-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original.

(Written per leaf as settled.)

## Common ground, checked once

- **Layout.** Both: `Leader*` displacement = `LeaderData` offset; global
  form `0xe3a390 + i·0x6eec + off`. **Confirmed** (every loop below steps
  `0x6eec` from `0xe3a390` to `0xe71af0`).
- **Bulk writers.** Both: `Leader::init (006e3930)`'s `memset(this, 0,
  0x6929)` under `tribe < 0`, and `LeaderData::walk_data (006d6750)` under
  a load. **Confirmed.** One precision neither made: `init` with `tribe < 0`
  **returns at `6e3fee`** (`cmpl %eax,0xc(%ebp); jl 0x6e50f3`), after the
  `defeated_by = −1` (`6e3b6b`) and `defeat_stamp = 0` (`6e3e0f`) stores but
  **before** `wonderwin_stamp/timer = 0` (`6e4160`/`6e4166`). Those two are
  zero on the `tribe < 0` path only through the memset. Same value either
  way, so no rule changes.
- **The adjudicator's own nets.** (a) A script over the full listing
  (`adj/eog/net.py` in the job scratch) follows every `movl/leal $imm,%reg`
  with `imm` in `[0xe3a390, 0xe3a390+0x6eec+0x100)` for 80 lines, and
  resolves each store `disp(%reg)` to `(imm+disp−0xe3a390) mod 0x6eec`.
  For the six leaves it returns only the stores both readers name, plus
  two false hits in `TurnControl::toggle_pause (00957ab0)`
  (`957be0`/`957c73`, `orb $1,0x10(%esi)`). There `esi` is `TurnControl`'s
  `this` (`957ae6`), reached by a jump from `957b03`, not the walked
  leader. (b) Every store `disp(%reg)` with disp `0x448/0x44c/0x7d8/0x7dc/
  0x6924` in the listing: the list both readers typed, no extra
  leader store. `ScenarioRead::load_leaders_chunk (009acde0)` `9ad0e7/ed`
  sits in a `0x8c`-stride run (`0x3bc/0x3c0/0x3c4` → `0x448/0x44c/0x450`),
  so it is `GameInfo.player[]`. Both readers said so; **confirmed**.
  (c) The absolute-address forms `0xe3ab6c` at `584425` and `0xe3a7dc` at
  `59495c` are `movl $imm,%ecx` **loop bases for reads** (`584430` reads
  `-0x7dc(%ecx)`; `594961` reads `-0x44c(%ecx)`), not stores. The first
  reading's wording ("a `mov $imm`") is right, but neither is a write.
- **The lobby the captures run.** Neither reader checked it, and it moves
  several "plain game?" verdicts: run600's dump has `ELIMINATION 1` and
  `VICTORY 0` (`gamelog-run600-islands-french-toughest-24k-trace.txt` line
  4727). The crate's default lobby is `elimination: 1`
  (`crates/sim/src/ai.rs:1132`, "capital-countdown elimination"). So
  **elimination mode 1 is the plain game**, not an option.
- **A diff-able record already on disk.** Neither reader grepped a dump.
  run600's closing block (frame 17380) has these values:

  | slot | `defeated_by` | `defeat_stamp` | `defeat_type` | `victory_type` | `wonderwin_stamp/timer` |
  |---|---|---|---|---|---|
  | 0 (human) | −1 | 17380 | 5 | 0 | 0/0 |
  | 1 (AI, `popwin_stamp 14293`, `popwin_timer 1`) | −1 | 0 | 0 | **2** | 0/0 |
  | 2..7 | −1 | 0 | 0 | 0 | 0/0 |

  A territory victory (type 2) cascades into the human's type-5 defeat with
  `by = −1`, on the closing frame. That backs four rules below. It also
  contradicts the crate's "the game never ends by territory"
  (`crates/sim/src/territory/victory.rs:21`). And the human's
  `defeat_type` is 5, not 6, which sits badly with
  `crates/rondata/src/diff/third_pair.rs:432`'s "this capture quits on
  17379 (`defeat_stamp 17379`)". The quit, if there was one, was not the
  first effective defeat.

## Leaf 1: `defeated_by` (+0x10)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| writer: `Leader::defeat_by (006d1c80)` `6d1da6` | yes | yes | agree | **confirmed**: `6d1da6 movl %esi,0x10(%edi)`, `esi` = `0x8(%ebp)` = first arg `by` |
| writer: `Leader::init (006e3930)` → −1 | yes | yes | agree | **confirmed** `6e3b6b` (`edi` = −1 from `6e3aca or $-1`) |
| `defeat_by`'s only caller is `Leader::defeat (006ecb00)`, args `(by, type)` | yes | yes | agree | **confirmed** `6ecb19..6ecb1d`: pushes `esi` (= `type`) then `0xc(%ebp)` (= `by`), so `defeat_by(by, type)` |
| guard `defeated_by >= 0` → return | yes | yes | agree | **confirmed** `6d1c8e cmpl $0,0x10(%edi); jge` (signed) |
| `by < 0` skips the store | yes | yes | agree | **confirmed** `6d1d9e testl; 6d1da0 js` |
| `defeat_by` runs before `defeat`'s `& 0x40` guard | yes | yes | agree | **confirmed** `6ecb1d` call, `6ecb24` test. Consequence neither stated outright: **an already-defeated leader whose earlier defeats were all captor-less can still receive `defeated_by`, and the captor arm (`capture_city`) runs for it** |
| step: `= by`, write-once | yes | yes | agree | confirmed |
| `process_elimination`'s `by` = holder of the capital | yes | yes | agree, incomplete | **corrected**: `by` = `LeaderData::find_capital (006eb930)`'s second out (`6b8a93`). That out **falls back to the loser's own `who`** when neither its own list holds a live capital-flagged city nor another leader holds a live city with its was-capital bit (`find_capital` tail, `*param_2 = this->who`). So `defeated_by` = self whenever the capital city is gone (razed), not only in `leave_game` |
| `process_elimination`'s by-self arm (`6b8a75`) is dead | yes | yes | agree | **confirmed**: `6b8a30` tests `game+0x37 == 1` (via `0xc061ec`), `6b8a65` retests via `0xc061e8` (`gamec`, the same object) |
| `gain_tech` arm: `by` = the gainer, type 1 | "the bonus holder" | "for every other leader with no capital held by gainer or ally" | agree, both incomplete | **corrected**: `6dd51d..6dd599`. When the gained tech is the `0xae4` bonus's preq, it walks all ten slots `j ≠ gainer`. It calls `defeat(1, gainer, 1)` when `LeaderData::has_capital (006d54c0)` returns 0 and its holder is the gainer or a leader the **gainer** counts as ally (`is_ally(gainer, holder)`, `6dd586`). `has_capital` returns 1 unless `elimination == 1`, so the arm exists **only under elimination 1**. `find_capital` returns a foreign holder only when `j`'s own `flags & 3 == 3`, so defeated or empty slots never fire it |
| `lost_a_city` GG arm | yes | yes | agree | **confirmed** `6dc72c..6dc7bf`: `param_1 != 0`, captor `>= 0`, `elimination == 1`, `!has_wonder(0x213)`, then some slot `j` with `flags & 3 == 3` that is the captor or that **the captor** counts as ally (`is_ally(leaders[captor], j)`, `6dc76d`) has preq `0x2b9` → `defeat(1, captor, 1)` |
| `lost_a_city` tail passes the captor as `by` for types 0/2/3 | first: "captor, may be −1" | second: "captor" | agree | **confirmed** `6dc802/6dc7b2/6dc816` all push `edi` (= `0x1c(%esp)`, the captor) |
| `leave_game` capital-timer arm: holder under elimination 1, else self | yes | yes | agree | **confirmed** `6ee0a8..6ee0ef` (timer `0xe3a7a8` = `+0x418`; `gamec+0x37 == 1` → `find_capital` holder, which can itself be self; else `defeat(1, who, 0)`). One-sided omission: in multiplayer (`semaphore[0] & 4`), `6ee047..6ee079` returns with **no defeat** when another seated human shares the leaver's `who` |
| reset: only `init` | yes | yes | agree | confirmed |
| readers: `log_data`, `walk_data`, the guard; no sim decision | yes | yes | agree | confirmed (export: no `list[i].field_0x10` read in a leader context; `Build::close`'s `list[0].field_0x10 ^ 0x63637` is a mistyped `this`) |
| crate site `Sim::defeat` (`crates/sim/src/cities/city.rs:623`), guard `leaders.defeated[who]` | yes | yes | agree | **confirmed**. The crate's early return blocks the late write the original allows (row 6) |
| plain game | yes; types 1–3 "need a non-default elimination option" | yes; "capital-elimination lobby" | disagree on the lobby | **corrected**: the captures run elimination 1, so `process_elimination` and both GG arms are plain-game. run600 shows `defeated_by = −1` on the type-5 loser (diff-backed for the cascade's `by = −1`) |

**The rule as a builder should port it.** `defeated_by: [i32; N]`, −1 at
`Leader::init`.

- **Writer and predicate.** In `Sim::defeat(who, type, by, instant)`, at
  entry and **before** the `defeated[who]` return: if `defeated_by[who] < 0`
  (signed), stamp `defeat_stamp` (leaf 6). Then, if `by >= 0`, set
  `defeated_by[who] = by` and, unless `elimination == 3`, run the captor
  arm.
- **Step.** Write-once.
- **What `by` holds, by caller.**
  - `lost_a_city`: the captor (types 0, 2, 3; type 1 in the GG arm).
  - `process_elimination`: `find_capital`'s holder, self when the capital
    is gone.
  - `gain_tech`'s GG arm: the gainer. Elimination 1 only.
  - `leave_game` with a capital timer: the holder, or self.
  - Every other caller: −1.
- **Crate site.** `crates/sim/src/cities/city.rs:623`.
- **Plain game?** Yes.

Confidence high: listing for every store, guard and push. Open: whether a
captured game ever shows a non-negative `defeated_by` (run600 has none;
the archive sweep below answers it if the archive holds one), and
`lost_a_city`'s callers' derivation of `param_1`/`param_2`/captor (owned
by the lost-capital family's reading of `Build::close`).

## Leaf 6: `defeat_stamp` (+0x6924)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| writers: `defeat_by` `6d1d98`, `init` `6e3e0f` | yes | yes | agree | **confirmed** (both nets) |
| value `game->frame` (`Game+0x550`) | yes | yes | agree | **confirmed** `6d1d92` |
| stamped under `defeated_by < 0` only, **before** the `by < 0` test | yes | yes | agree | **confirmed** `6d1d98` precedes `6d1da0` |
| re-stamped by any later `defeat` while `defeated_by` is still −1, even on a defeated leader | yes | yes | agree | **confirmed** (`6ecb1d` before `6ecb24`) |
| `Unit::close (0060ee50)` re-enters with no defeated test | first: example | second: listing | agree | **confirmed** `60f42f..60f474`: owner `< 8`, `+0x93c` (units) 0, `+0x3f8` (cities) 0, `semaphore[1] & 8` clear, `leader_flags2 & 1` clear → `defeat(0, −1, 0)`. The count is decremented first (`60f406`), so it fires on the **last** unit's close. The crate's `Sim::resign` (`crates/sim/src/leaders/resign.rs:43`) kills only planes, so a resigned leader's last land unit dying later re-stamps it in the original |
| the type ≠ 5 blow-up arm precedes the stamp and does not touch it | yes | yes | agree | **confirmed** `6d1c98 cmpl $5,0xc(%ebp); je 0x6d1d89` (the stamp) |
| reset: `init` → 0 | yes | yes | agree | confirmed |
| readers: `AchieveChartPane::set_chart_data (007f2c40)`, `log_data`, `walk_data`; no sim decision | yes | yes | agree | confirmed by export grep |
| crate: no field; route ahead of `defeated[who]` | yes | yes | agree | **confirmed** (`city.rs:624`, `resign.rs:44..52`) |
| `third_pair.rs` names a capture reading 17380 | — | yes | one-sided | **confirmed** and strengthened: run600's dump has slot 0 `defeat_stamp 17380` with `defeat_type 5` and `defeated_by −1` (the cascade from slot 1's type-2 win). That is a value diff waiting to be pinned |

**The rule as a builder should port it.** `defeat_stamp: [i32; N]`, 0 at
`init`.

- **Writer and predicate.** At entry to every `defeat` call, **before**
  the defeated-flag return: if `defeated_by[who] < 0`, set
  `defeat_stamp[who] = frame`.
- **Step.** Assignment. It moves again on each later call while
  `defeated_by` stays −1, and freezes once a `by >= 0` lands.
- **Crate site.** `Sim::defeat` and `Sim::resign` entry. Re-stamping also
  needs `Unit::close`'s last-unit `defeat(0, −1, 0)`, which the crate
  lacks.
- **Plain game?** Yes. run600 is the diff-backed case.

Confidence high. Open: whether any capture shows a re-stamp (a stamp later
than the frame `defeat_type` was written). The sweep below looks.

## The archive sweep (adjudicator's, all six leaves)

`rg` over all 3,805 dumps in the gamelog directory (110 GB) for a nonzero
value of each leaf (`defeated_by >= 0`, any nonzero stamp or type, a
running `wonderwin_timer`). Output kept at `adj/eog/sweep.txt` in the job
scratch. 7,040 rows:

| leaf | values seen (rows) | what they are |
|---|---|---|
| `defeated_by` | 1 (130), 0 (1) | the `*-trace` captures: the human's last city taken by AI slot 1, e.g. run1200 `defeated_by 1`, `defeat_stamp 5639`, `defeat_type 0` (the lost-capital family's 5639); run24 (truncated, old format): AI slot 1 `defeated_by 0`, defeated at 16488, 3363 frames after its capital fell at 13125 (`docs/RUNS.md` line 349), the shape of `process_elimination` |
| `defeat_type` | 6 (238), 9 (82), 5 (5) | 6: the `*-start` captures' quit (run1199 `defeat_stamp 36`, `defeated_by −1`); 9: the team-tournament scenario (run1113/1114, stamp 600); 5: the type-5 cascade |
| `victory_type` | 2 (5) | territory wins: run600, run972b, run981b (`defeat_stamp 18884`), run1256, run1316, each beside one type-5 loser |
| `wonderwin_timer` | 1 (3,064) | 30 captures (run652 … run1423c), stamps 4240, 4369, 4552, 4573, 5037, 5285, 5734 |
| `wonderwin_stamp` | positive only | **no negative stamp anywhere**: no stop has been captured, so the stop arm and the pay-back are reading-only |
| `defeat_stamp` | one value per file | no file shows two stamps, so no re-stamp has been captured |

## Leaf 4: `victory_type` (+0x7d8)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| sole writer `Leader::victory (006ec9b0)` `6ec9ec` | yes | yes | agree | **confirmed** (both nets; `esi` = `this`, `eax` = `0x8(%ebp)`) |
| guard `leader_flags & 0x60` → return | yes | yes | agree | **confirmed** `6ec9be testb $0x60,%al; jne` |
| sets `0x20`, `flags2 & 0x100` from `instant` | yes | yes | agree | confirmed `6ec9c6..6ec9da` |
| cascade: every other slot with `flags & 3 == 3` gets `0x80`, then `victory(type, instant)` if `is_ally(that, who)` (that slot's view), else `defeat(5, −1, 0)` | yes (export only for the argument order) | yes | agree | **confirmed in the listing** `6ecaa2..6ecadd`: `6ecabf` `is_ally` with `ecx` = the walked slot and arg = `this->who`; `6ecaca/6ecacd` re-push `0xc(%ebp)`, `0x8(%ebp)`; `6ecad7..6ecadd` push 0, −1, 5. **Diff-backed**: run600, run972b, run981b, run1256, run1316 each show `victory_type 2` beside one `defeat_type 5` / `defeated_by −1` |
| a prior winner not allied to a new winner can get `defeat(5)` (`defeat` tests only `0x40`) | yes | — | one-sided | **confirmed** (`6ecb24` tests `0x40` only; the cascade's `& 3 == 3` admits `0x20`). Reachable only with two unallied winners in one game |
| type codes 0..6 by caller | yes | yes (table) | agree | **confirmed** by the pushes in `GameDaemon::process_victory (00730ef0)`: `73132c` 1, `731940` 2, `7319ad` 4, `731a61`/`731abf` 6, `731f6f` 5. Type 3: `Leader::gain_tech (006dcb60)` export lines 1246, 1273 (VICTORY 9). Type 0: `Leader::set_diplo (006ec6a0)` line 135, `TurnControl::toggle_pause` line 149, `Game::check_victory` |
| `check_victory`'s gate | first: "muddled, export only" | second: "all remaining undefeated seated leaders mutually allied" | disagree (incomplete) | **corrected** (`Game::check_victory (005926b0)`, export lines 26–166). (1) Returns at once under `semaphore[0] & 0x40` (the game is already over). (2) Counts seated humans and live humans (`flags & 0x43 == 3`). If any human lives, or there are no humans (or balance mode), it returns unless every live leader is mutually allied with the first live one (`diplos == 2` both ways). If humans exist but none lives, it skips that test. (3) Then it latches `semaphore[0] |= 0x40` and computes `A = armageddon_per_team·num_sides + armageddon_per_nation·num_nations + armageddon` (×3 at starting resources 7, ×2 at 5/6/11/12, floor 100 at 8). Only if `game.armageddon < A`: it walks the slots with `flags & 3 == 3` (setting `0x80` on each); the first not yet defeated is the candidate. If any later live, not-won slot is not mutually allied with it, it sets `semaphore[2] |= 0x40` and **no victory**. Otherwise `victory(candidate, 0, 0)`. So when all humans are dead and the computers are not all allied, the game ends with **no winner** and no `victory_type` |
| reset: `init`'s `tribe >= 0` path does not touch it | first: `stosl` stops at `+0x7d4` | second: init never stores it | agree | **confirmed** `6e3b80 leal 0x7c8(%ebx),%edi` + four `stosl` (`6e3bee..6e3bf8`) → `+0x7c8..+0x7d4` |
| readers: `EndGameWin::exec (0083bcc0)`, `determine_end_of_game_stats_and_achievements (00583b30)` `58403c`, `log_data`; no sim decision | yes | yes | agree | confirmed (export grep, global-form net) |
| crate: no victory event; seam in `territory/victory.rs` | yes | yes | agree | **confirmed** (`crates/sim/src/territory/victory.rs:21`; no `fn victory` in `crates/sim/src`). The seam's "the game never ends by territory" is **contradicted by five captures** |
| plain game: conquest 0, wonder 1, territory 2 under VICTORY 0 | yes | yes | agree | **confirmed**, and type 2 is diff-backed in five captures |

**The rule as a builder should port it.** `victory_type: [i32; N]`, 0 from
`Leaders::init`'s memset and never reset again. Pair it with a won flag
(`0x20`), since 0 is also conquest.

- **Writer.** `Sim::victory(who, type, instant)`.
- **Predicate.** Neither won nor defeated.
- **Step.** Set `victory_type[who] = type` once. Then cascade to every
  other seated live slot: `victory(type, instant)` if that slot counts
  `who` as an ally, else `defeat(5, −1, 0)`.
- **Callers.** The territory expiry (`victory(2, ·)`, the first port, with
  run600's closing block as its diff), the wonder arm (1), and
  `check_victory` (0) under the gate above.
- **Crate site.** A new `Sim::victory` beside `Sim::defeat`.

Confidence high on the writer, the guard and the cascade (listing, five
dumps). The exact frame of run600's territory expiry (from `popwin_stamp
14293`) is the territory family's arithmetic, not checked here. Open: what
`Game::popwin_timer` returns for run600's map.

## Leaf 5: `defeat_type` (+0x7dc)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| sole writer `Leader::defeat (006ecb00)` `6ecb7b` | yes | yes | agree | **confirmed** (both nets) |
| guards `flags & 0x40` (`6ecb24`), `semaphore[1] & 2` (`6ecb32`), after `defeat_by` | yes | yes | agree | **confirmed** |
| order: `flags2 & 0x200` ← `instant`, `flags = (flags & ~2) \| 0x40`, `Armies::leader_defeated`, `flags2 \|= 2`, `musical_chairs = frame` (`Game+0x6c4`), store | yes | yes | agree | **confirmed** `6ecb3f..6ecb7b` |
| per-caller types (0..9) | yes | yes (table with addresses) | agree | **confirmed** by the listing pushes: `lost_a_city` `6dc7b8` (1), tail `6dc802` (2), `6dc7b2` (3), `6dc816` (0); `process_elimination` `6b8a9b` (1); `gain_tech` `6dd594..6dd598` (1, `instant` 1); `leave_game` `6ee0c4`/`6ee0ed` (1), `6ee0ff` (pass-through `t`); `Unit::close` `60f46c` (0); victory cascade (5); `process_victory` `731e78`/`731efa` (4; the first passes `edx` as `instant`, not 0). **Diff-backed**: 6 (238 rows, the start captures' quit), 5 (5), 9 (82, scenario), 0 with `by = 1` (run1200) |
| `lost_a_city` mode switch | yes | yes | agree | **confirmed** `6dc79d..6dc81b` (`elimination` byte `game+0x37`). 2 → type 2 for `param_1 != 0` unless tribe bonus `0x17` and `find_capital` still finds a capital the loser holds itself (`6dc7f2..6dc800`). 3 → type 3 when `param_2 != 0`. Any other mode (0, **1**, …) → type 0 when `flags2 & 1` is clear and `+0x3f8 == 0` |
| value 10 | first: CtW, "`ConquestLeader::defeat` is the CtW path" | second: no caller pushes 10; reaches the field only by a load | disagree | **second confirmed, first corrected**: `ConquestLeader::defeat (0079c460)` does not touch `+0x7dc` (export: no `0x7dc`), and the register-store net finds no other leader store. The `cmpl $0xa,0xe3ab6c` at `799194` reads a value no code in `.text` writes, except through `walk_data` |
| reset: memset only | yes | yes | agree | confirmed (as `victory_type`) |
| readers: UI, stats, CtW; no sim decision | yes | yes | agree | confirmed (export grep) |
| crate: `Sim::defeat` has no type; `Sim::resign` is 6 | yes | yes | agree | **confirmed** (`city.rs:623`, `resign.rs:43`). The store sits after the `defeated[who]` return, which matches `0x40` |
| plain game: type 1 needs a non-default elimination | yes | partly ("capital elimination lobby") | disagree on the lobby | **corrected**: elimination 1 is the captures' default (run600 line 4727), so type 1 (`process_elimination`, GG arms) is plain-game. Type 2/3 need modes 2/3 |

**The rule as a builder should port it.** `defeat_type: [i32; N]`, 0 from
the memset and never reset again.

- **Writer and predicate.** `Sim::defeat(who, type, by, instant)` writes
  `defeat_type[who] = type` after `defeat_by` (leaves 1 and 6), only if the
  leader is not already defeated and not in the balance-test mode.
- **Step.** Write-once.
- **What `type` holds, by caller.**
  - `lost_a_city`: 0, 1, 2 or 3 by mode.
  - `process_elimination`: 1.
  - `gain_tech`: 1.
  - `leave_game`: 1 when a capital timer runs, else 6 (resign) or 7 (drop).
  - `Unit::close`: 0.
  - The victory cascade: 5.
  - Musical chairs: 4.
  - Armageddon (`Game::defeat_all`): 8.
  - The scenario: 9.
- **Crate site.** `crates/sim/src/cities/city.rs:623`. `Sim::resign`
  passes 6, but under a running `lost_capital_timer` the original passes 1
  with `by` = holder or self (`6ee0a8`). That is a plain-game divergence
  the crate's resign does not model.

Confidence high (listing for every push). Diff-backed for 0 (with captor),
5, 6 and 9. Open: whether any capture resigns under a running capital
timer (it would show `defeat_type 1` with `defeated_by` = the quitter's
own slot; the sweep found no `defeat_type 1`).

## Leaf 3: `wonderwin_timer` (+0x44c)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| writers: `process_victory` `731070` (=1), `731357` (=0); `init` `6e4166` (=0) | yes | yes | agree | **confirmed** (both nets; `ebx` = `0xe3a390 + i·0x6eec`, `730f60`/`7313a2`) |
| a 0/1 flag | yes | yes | agree | confirmed; the archive's 3,064 nonzero rows are all 1 |
| gate `VICTORY` ∈ {0,1,6} | yes | yes | agree | **confirmed** `730f2d..730f3a` (`game+0x38`) |
| `W = Game::wonder_winning (005948a0)`, read once per frame | yes | yes | agree | confirmed `730f4e` |
| walk: eight slots (`-0x14` from `0x74` to `0x94` by 4), `flags & 3 == 3` | first: "0..7" | second: listing | agree | **confirmed** `7313ae cmpl $0x94` |
| off side: `W < 0`, or `i ≠ W` and `who_i ≠ W` and not (`diplos_i[W] == 2` and `leaders[W].diplos[who_i] == 2`) | yes | yes | agree | **confirmed** `730f84..730fb4` (`0x1bbb·4 = 0x6eec`) |
| `gg`: some `j` with `flags & 1` that is `i`, has `who_j == i`, or is mutually allied with `i`, holds preq `0x2b9` | first: "alive (`flags & 1`)" | second: "seated (`leader_flags & 1`)" | disagree on the word | **second confirmed, first corrected**: `730fd0 testb $0x1,(%esi)` tests bit 0 only. `Leader::defeat` clears bit 1 and leaves bit 0 (`6ecb58..6ecb5e`), so **a defeated teammate holding the tech still makes `gg`**. The loop walks all ten slots (to `0xe71af0`, `73101d`) |
| immediate branch when `VICTORY != 0`, or (not MP and `num_nations <= 1`), or `gg`: first on-side slot only, `victory(i, 1, gg)` | yes | yes | agree | **confirmed** `731031..731056`, `731320..73132e` (`instant` = `gg`, pushed from `-0x18`) |
| timer branch: flag 0 → set, `stamp += frame`, event 9; flag set and first on-side → `remaining = wonder_timer() − (frame − stamp)`, win on `remaining == 0` exactly | yes | yes | agree | **confirmed** `73105c..731080`; `7311cc..7311e4` (`subl %esi,%eax` sets ZF; the two `movl` between do not touch flags; `jne`) |
| the walk does not stop; only the first on-side slot (the `-0x1c` latch) is tested | yes | yes | agree | **confirmed** `7311b6`, `73133c`, `7311c6` |
| the wonder expiry is `== 0` where the territory arm's is `<= 0` | — | yes | one-sided | **confirmed**: territory `7317d8 subl; 7317df jg` (wins on `<= 0`) against wonder `7311e4 jne`. A copy of the territory arm would be wrong here. Corollary: with `constants.wonder_timer < 1`, `wonder_timer()` is 0 and the clock **never** wins (the first test, one frame after the start, sees −1) |
| warnings at 4500/1800/900 for a viewer not allied | yes | yes | agree | confirmed `731205..73121b` |
| `Game::wonder_timer (005944f0)` = 0 if `constants+0xd08 < 1`, else `max(1, (M/2 + world[0]·T) / M)`, `M` = `map_sizes` current row `+0x144`, signed | yes | yes | agree | **confirmed** `5944f0..59452f` (`M/2` is the signed halve `cltd; sub; sar`; `idivl` truncates). `M`'s row (`0xe7fca8`) and `world[0]` (`*0xc06188`) are open |
| reset: `init` → 0 | "both branches" | yes | agree, route corrected | 0 in both, but by the memset under `tribe < 0` (`6e3fee` returns first); the explicit store is the `tribe >= 0` path's |
| readers: `Army::find_target` (`6f6c38`, `6f72e0`), `Leader::diplomacy` (`6be27f`), `Game::is_victory_timer (00594950)`, `Wall::process` | yes | yes | agree | **confirmed** (global-form net: the three reads plus the `59495c` loop base). `Wall::process (00640450)` detail (first, one-sided): **confirmed** at export lines 105–150. For a seated rival that does not count the wall's owner as ally, a running flag adds **one**, else each of its competing unbuilt wonders inside the progress band adds one. The crate's SEAM says the same (`crates/sim/src/cities/site_recruit.rs:157`) |
| crate: not carried; `w44c` stub returns 0 (`ai/army/target_gate.rs:428`) | yes | yes | agree | **confirmed**. **The stub is wrong in 30 captures**: the archive has `wonderwin_timer 1` in run652 … run1423c, so `find_target`'s ×10 and `about_to_win` gates and `Wall::process`'s count read a live 1 there |
| plain game | yes | yes | agree | **confirmed, diff-backed**: VICTORY 0, ≥ 2 nations, 30 captures |

**The rule as a builder should port it.** `wonderwin_timer: [i32; N]`
beside `popwin_timer` (`crates/sim/src/ai.rs`), 0 at init. Writers:
a new `process_victory_wonder`, run in `core/frame.rs` **before**
`process_victory_territory` (`crates/sim/src/core/frame.rs:222`). It runs
every frame under `VICTORY` ∈ {0, 1, 6}, with `W = Sim::wonder_winning()`
(`crates/sim/src/ai/ai_build.rs:1439`). For slots 0..7 with `flags & 3 ==
3`, in order:

- **Off side.** If the flag is set, clear it and do `stamp −= frame`.
- **On side.** Compute `gg` over the ten seated slots (bit 0).
  - Under `VICTORY != 0`, or (not MP and `num_nations <= 1`), or `gg`: if
    no earlier on-side slot was handled this frame, call
    `victory(i, 1, gg)`.
  - Else, if the flag is clear: set it and do `stamp += frame`.
  - Else, if no earlier on-side slot was handled:
    `remaining = wonder_timer() − (frame − stamp)`, and call
    `victory(i, 1, 0)` iff `remaining == 0`.
  - Mark handled.
- **The constant.** Port `WONDER_TIMER` (`Constants+0xd08`) with it.

Confidence high (listing end to end; the running state is diff-backed in
30 captures). Open: the stop arm's state (never captured), `M`'s row and
`world[0]`, and whether any capture reaches a wonder win (`victory_type
1`: none in the archive).

## Leaf 2: `wonderwin_stamp` (+0x448)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| writers: `process_victory` `731080` (`+= frame`), `731367` (`−= frame`) | yes | yes | agree | **confirmed**. The archive's stamps (4240 … 5734) are positive start frames, as `0 + frame` predicts |
| pay-back writer: `Leaders::process_all (006ed2a0)` `6ed3c8` | yes | yes | agree | **confirmed** `6ed35f..6ed3c8`: `constants+0xd00` (`TIMER_REFRESH_RATIO`) non-zero, `frame % r == 0` (signed `idivl`), `stamp < 0` (`jns`), flag 0 → `+1`. It runs after `process_elimination` (`6ed35a`) and after the `+0x414` and `+0x440` pay-backs, inside the slot gate `flags & 2` (`6ed2b0`) |
| `Leader::process (006b88b0)` `6b89c8` is a dead copy | yes | yes | agree | **confirmed**: no `0x6b88b0` operand in `.text` (the only hit is its own label) |
| `Leader::end_process`/`end_process_all` word stores are `GameInfo`'s | yes | yes | agree | confirmed (the `+0x3bc`, `0x8c` stride both readers cite) |
| reset: `init` → 0 | yes | yes | agree | as leaf 3 (memset under `tribe < 0`) |
| readers: the expiry, `IFaceMainBase::draw_timers (0080b060)`, `log_data` | yes | yes | agree | confirmed |
| crate: the pay-back belongs in `Sim::refresh_victory_timers` (`crates/sim/src/territory/victory.rs:88`) | yes | yes | agree | **confirmed** |
| the crate's pay-back loop has no `flags & 2` gate | yes | — | one-sided | **confirmed**: `crates/sim/src/core/frame.rs:94` loops every player and calls `refresh_victory_timers(who)` at `:194` with no defeated test, while `6ed2b0 testb $0x2,(%esi); je` skips a defeated slot. A defeated leader with a banked (negative) `popwin_stamp` and a stopped timer is paid back in the crate and not in the original. Same gap for the wonder pair once ported. No capture checked; no negative stamp is in the archive for this leaf |

**The rule as a builder should port it.** `wonderwin_stamp: [i32; N]`,
0 at init, frames.

- **Writers.** `+frame` at a start and `−frame` at a stop (leaf 3's arm).
  Also `+1` in the pay-back when `TIMER_REFRESH_RATIO != 0`,
  `frame % ratio == 0`, `stamp < 0` and the flag is 0, **only for a slot
  with `flags & 2`** (alive).
- **Readings.** While running, elapsed = `frame − stamp`. While stopped,
  `−stamp` is the banked time.
- **Crate site.** `crates/sim/src/territory/victory.rs:88`
  (`refresh_victory_timers`), with the alive gate added for both pairs,
  and the arm of leaf 3.
- **Plain game?** Yes. Diff-backed for the start.

Confidence high on the arithmetic (listing). Open, reading-only: the stop,
the banked negative and the pay-back (no capture holds a negative stamp).

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `defeated_by` +0x10 | 13 / 3 / 1 | `= by` once, inside `defeat_by` at entry to every `defeat` (before the defeated guard), when still < 0 and `by >= 0`; `by` falls back to self via `find_capital`; GG arm of `gain_tech` only under elimination 1, which is the captures' default; −1 at `init` |
| `defeat_stamp` +0x6924 | 10 / 0 / 1 | `= frame` at entry to every `defeat` while `defeated_by < 0`, so it is re-stamped by later captor-less calls (`Unit::close`'s last unit); 0 at `init`; diff-backed by run600 (17380, type 5) |
| `victory_type` +0x7d8 | 10 / 1 / 1 | `= type` once in `Leader::victory` (guard `& 0x60`), copied to allies, unallied get `defeat(5, −1, 0)`; `check_victory` gives 0 only when all live leaders are mutually allied and `armageddon < A`; memset only; type 2 diff-backed in five captures |
| `defeat_type` +0x7dc | 9 / 2 / 1 | `= type` once in `Leader::defeat` after `defeat_by` (guards `& 0x40`, balance mode); 10 has no writer; resign under a capital timer is 1, not 6; memset only; 0, 5, 6, 9 diff-backed |
| `wonderwin_timer` +0x44c | 14 / 2 / 1 | 0/1 flag set for every on-side slot under VICTORY 0 with ≥ 2 nations and no `gg` (bit-0 teammates, defeated included), cleared off side; the first on-side slot wins at `remaining == 0` exactly; running state in 30 captures, against the crate's 0 stub |
| `wonderwin_stamp` +0x448 | 8 / 0 / 1 | `+frame` at a start, `−frame` at a stop, `+1` per `TIMER_REFRESH_RATIO` frames while negative and stopped, alive slots only (the crate's pay-back lacks that gate) |
