# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/journal/` is the story; `docs/PARKED.md` is the backlog,
read when a wave is composed and never at boot.

This file **deletes**; `queueledger.py` fails the build on a number that
leaves in silence or is referred to and never booked; `docs_guard.rs` bounds
it by items (18 at 8 lines, the handoff 32). **A finding that names no
score parks**: only the headline's frame, a floor, or a takes-chain books here.

## Where things stand

*2026-10-07, the twenty-seventh tranche (`commander`, Opus 5.5, from
`dfa4b699`), fifteen landings in: the coverage pair **1408 → 2868** (1563,
1586, 1588, 1589, 1591, 1594, 1598, 1602, 1605); Great Sahara in the coverage lobby **1582 →
2206** (the race's winner 1565, 1581, 1583, 1584, 1585; then 1600).*

- **The race is scored** (`docs/audit/2026-10-07-race-1565.md`): Sonnet
  5.5 reached 2185 for 74.43 USD in 5 h 57; Opus 5.5 reached 1985 for
  71.08 in 5 h 04. The Sonnet chain merged; `origin/worktree-att-1565-opus`
  stays as the record; the roster decision is Ramon's. Its one
  disagreement, 1578, was already in the merged tree (1600's re-check).
- **Three lanes**: `att-1608` (Opus 5.5) on the coverage pair's word, runs
  716–717, AI §170; `att-1611` (Sonnet 5.5, the third map's lane) on Great
  Sahara's word, runs 718–719, AI §171; `att-1567` (Opus 5.5) on 1575, the sweep.
- **Next number 1614, run 720, section §172**; 1497 waits. Lane gate
  filters: 1608 `coverage_pair:: floors:: coverage::`, 1611 `sahara_coverage:: floors:: coverage_pair::`.
- **Fable backlog: 32 Loop items** (1119, 1138, 1316, 1421, 1462, 1464, 1475, 1478, 1485, 1490, 1507, 1516, 1518, 1521, 1534, 1537, 1548, 1554, 1560, 1566, 1570, 1572, 1573, 1574, 1590, 1593, 1597, 1601, 1604, 1607, 1610, 1613).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w2868 of 4,730
Census: simulation backed 45 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w2206 of 4,340
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-seventh tranche, live — `commander` (Opus 5.5):
merge 1608's, 1611's and 1575's landings as they come; stop at twenty
landings and spawn `steer`.**

## The queue

In dependency order, headline-nearest first. **The sweep lane (1575)
beside two word lanes**: the newest pair's (1608) and the third
map's (1611); 1497 waits; lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1608. **The coverage pair's word: frame 2868, 17 versus 18 draws**,
    index 7: ours `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    `Guy::set_anim+0x97a < Unit::move_step+0x823`. Inside run715 (block
    2869), widened by `run715_s_word_frame_is_widened_whole` (6,302 keys):
    the word's own block parts first — `1/74`'s `order:kind` 12 against 14,
    `orders.len` 1 against 2, and `1/94` blocked by `1/74` in the original
    alone (`collide_o` 74) — 1605's hypothesis. No mechanism.

1611. **Great Sahara in the coverage lobby: frame 2206, 14 versus 16
    draws** (count and sequence), index 3: ours `Unit::do_idle+0x7d`,
    theirs `Unit::move_step+0x823`. Inside run702, widened by
    `run702_s_word_frame_is_widened_whole`: `1/38`'s move order parts
    first on 2205, `x` 40920 against 40728, `y` 1944 against 1752, and its
    order's kind, path and heading on 2206 — 1600's hypothesis. No mechanism.

1575. **The sweep: ten never-backed simulation functions, both models**
    (DECISIONS 63 (iv), parked 1464's trial; the score `Census:` 45 of
    3611): each function run under the emulator on chosen inputs against
    this crate's (`callfn.py` pure, `step4.py` on a packet), a `#[test]` in
    `crates/sim` naming the address; agreed, or the input that parts,
    parked with its frame. The lane fans the batch to subagents and merges
    once; the same ten swept by Opus 5.5 subagents and by Sonnet 5.5,
    scored in functions agreed, inputs parted, USD a function; no dump taken.

1497. **Chapter fifty-two: the arms the unit tests hold and no walk**
    (DECISIONS 56 §3): parked 1473's member `find_melee_target` arm and
    `update_local_seen_build`'s fog arm, 1476's Construction hp arm
    (`myhits`, `construct_hits` unread), and 1500's two-order snap and 1508's
    M5 and M6 (the drop, the captain's re-search). Whether one
    staging reaches each is the item's to establish first. run654.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis, not a build order.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §13, §14; the run
  number is minted at booking); **a worker parks what names the
  headline's frame and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Census:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a detached
  capture waits on `tools/gamelog/waitrun.sh`** (565, 656). **A value question
  takes a packet at the word's own frame, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137 — ending `Gate steps:`;
  **a worker's takes `--lane`** and its landing quotes `Lane verdict:` (969).
