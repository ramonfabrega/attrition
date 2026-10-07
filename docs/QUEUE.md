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
`dfa4b699`), eighteen landings in: the coverage pair **1408 → 2969** (1563,
1586, 1588, 1589, 1591, 1594, 1598, 1602, 1605, 1608); Great Sahara in the coverage lobby **1582 →
2273** (the race's winner 1565, 1581, 1583, 1584, 1585; then 1600, 1611); the
census's backed column **45 → 57** (1575, the sweep's first batch).*

- **Two trials, scored; the roster is Ramon's.** The race
  (`docs/audit/2026-10-07-race-1565.md`): Sonnet 5.5 2185 for 74.43 USD,
  Opus 5.5 1985 for 71.08. The sweep (1575's journal): Opus 8 of 10 for
  5.23, Sonnet 9 for 3.68, each finding the parting the other missed.
- **Three lanes**: `att-1614` (Opus 5.5) on the coverage pair's word, runs
  716–717, AI §172; `att-1620` (Sonnet 5.5, the third map's lane) on Great
  Sahara's word, runs 719–720, AI §173; `att-1619` (Sonnet 5.5) on the sweep.
- **Next number 1623, run 721, section §174**; 1497 waits. Lane gate
  filters: 1614 `coverage_pair:: floors:: coverage::`, 1620 `sahara_coverage:: floors:: coverage_pair::`.
- **Fable backlog: 35 Loop items** (1119, 1138, 1316, 1421, 1462, 1464, 1475, 1478, 1485, 1490, 1507, 1516, 1518, 1521, 1534, 1537, 1548, 1554, 1560, 1566, 1570, 1572, 1573, 1574, 1590, 1593, 1597, 1601, 1604, 1607, 1610, 1613, 1616, 1618, 1622).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w2969 of 4,730
Census: simulation backed 57 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w2273 of 4,340
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-seventh tranche, live — `commander` (Opus 5.5):
merge 1614's, 1620's and 1619's landings as they come; stop at twenty
landings and spawn `steer`.**

## The queue

In dependency order, headline-nearest first. **The sweep lane (1619)
beside two word lanes**: the newest pair's (1614) and the third
map's (1620); 1497 waits; lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1614. **The coverage pair's word: frame 2969, 18 versus 19 draws**,
    index 0: ours `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`, theirs
    `Guy::set_anim+0x97a < Unit::move_step+0x823`. Inside run715 (block
    2970), widened by `run715_s_word_frame_is_widened_whole` (5,842 keys):
    on 2970 the original's `1/16` is blocked by `1/9` (`collide_o` 9) and
    ours walks on under a half step — 1608's hypothesis. No mechanism.

1620. **Great Sahara in the coverage lobby: frame 2273, 99 versus 91
    draws**, index 66: ours `Unit::think_scout+0x64c`, theirs
    `Unit::think_scout+0x436`. Inside run718 (block 2274), widened by
    `run718_s_word_frame_is_widened_whole` (737 keys, 66 standing): on 2102
    the original's `treaties[1]` is 3 (ours 1), on 2105 every human
    building's `ever_seen` 255 (ours 1), the human's units dead by 2139 —
    after Napata's strike on 2101; 1611's hypothesis. No mechanism.

1619. **The sweep's second batch: ten never-backed simulation functions**
    (DECISIONS 63 (iv); the score `Census:` 57 of 3611): 1575's shape —
    `crates/sim/src/sweep/`, one subagent a function, merged once; picked
    from the census's never-backed functions this crate ports, packet-free
    first. `is_in_range` and `get_speed`'s land arm wait on a packet (parked
    1617). The swarm runs on the lane's model until Ramon rules on 1575's trial.

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
