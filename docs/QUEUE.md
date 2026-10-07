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

*2026-10-06, the twenty-sixth tranche at its sixteenth landing (counted by
`git log` from `b3b0ae03`): French East Indies **closed at 17,379** (1514,
1519); Toughest **closed at 15,432** (1510–1528, its closing state 1535:
counts `[0, 0, 0, 0, 0, 6, 0]`), and in the coverage lobby (1538, 1549) parts on
**1197**, its widening owed; the coverage pair **1183** (1511, 1530, 1532, 1539, 1544, 1546).*

- **Two lanes**: `att-1552` (Opus 5.5, the coverage pair, run 679, AI
  §151) and `att-1555` (Sonnet 5.5, Great Sahara's coverage word 1197, run
  681, AI §152), `--effort high` both. Wait on them with
  `tools/lanewait.py att-1552 att-1555`. Next run 682, next section §153.
  1497 waits for a lane.
- **The coverage pair is the newest pair** (DECISIONS 62 §3): its row is
  in `AI_WORDS`, its `Coverage pair:` line read by
  `the_handoff_s_coverage_pair_is_the_pinned_word`; 1183 since 1546.
- **Merges back out on a section stub** when a lane's base predates the
  next booking's stub: the lane takes `ccc update --keep-conflicts` and
  keeps both (1511, once). Kill rules: neither tripped.
- **Fable backlog: 30 Loop items** (1119, 1138, 1139, 1316, 1421, 1450, 1462, 1464, 1467, 1475, 1478, 1482, 1485, 1490, 1507, 1513, 1516, 1518, 1521, 1526, 1527, 1529, 1534, 1537, 1541, 1543, 1548, 1551, 1554, 1557).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w1183 of 4,730
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w1197 of 4,340
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-sixth tranche, after the clear at ten — `commander`:
wait on 1552 (Opus 5.5) and 1555 (Sonnet 5.5); at twenty it spawns `steer`.**

## The queue

In dependency order, headline-nearest first. **Two lanes**: the newest
pair's (1552) and the third map's (1555); 1497 waits for a free lane,
lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1552. **The coverage pair's word: frame 1183, 29 versus 28 draws**,
    index 21: ours `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
    `… < Guy::inc_time+0x271`. Inside run678 (block 1184), widened by
    `run678_s_word_frame_is_widened_whole`: citizen 1/26 holds one
    order (kind 6) against the original's two (kind 3, a path of 8);
    1546's hypothesis is its seat in ours' pool 66 from 1022
    (`group:66.held`). No mechanism.

1555. **Great Sahara in the coverage lobby: frame 1197, 4 versus 3
    draws**, index 0: ours `Unit::do_move+0xe84`, theirs
    `Guy::set_anim+0x97a < Guy::inc_time+0x271` (1549's 1182, moved by
    1546's second capital). Past run680's last block (969): its
    widening is owed (`WIDENINGS`' `SAHARA_COVERAGE_WORD`), a capture
    over 1191..1447 (block 1198). No mechanism.

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
  `Second pair:`, `Third map:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
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
