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

*2026-10-06, the twenty-sixth tranche at its seventeenth landing (counted by
`git log` from `b3b0ae03`): French East Indies **closed at 17,379** (1514,
1519); Toughest **closed at 15,432** (1510–1528, its closing state 1535:
counts `[0, 0, 0, 0, 0, 6, 0]`), and in the coverage lobby (1538, 1549) parts on
**1197**, its widening owed; the coverage pair **1277** (1511, 1530, 1532, 1539, 1544, 1546, 1552).*

- **Two lanes**: `att-1558` (Opus 5.5, the coverage pair, run 682, AI
  §153) and `att-1555` (Sonnet 5.5, Great Sahara's coverage word 1197, run
  681, AI §152), `--effort high` both. Wait on them with
  `tools/lanewait.py att-1555 att-1558`. Next run 683, next section §154.
  1497 waits for a lane.
- **The coverage pair is the newest pair** (DECISIONS 62 §3): its row is
  in `AI_WORDS`, its `Coverage pair:` line read by
  `the_handoff_s_coverage_pair_is_the_pinned_word`; 1277 since 1552.
- **Merges back out on a section stub** when a lane's base predates the
  next booking's stub: the lane takes `ccc update --keep-conflicts` and
  keeps both (1511, once). Kill rules: neither tripped.
- **Fable backlog: 31 Loop items** (1119, 1138, 1139, 1316, 1421, 1450, 1462, 1464, 1467, 1475, 1478, 1482, 1485, 1490, 1507, 1513, 1516, 1518, 1521, 1526, 1527, 1529, 1534, 1537, 1541, 1543, 1548, 1551, 1554, 1557, 1560).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w1277 of 4,730
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w1197 of 4,340
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-sixth tranche, after the clear at ten — `commander`:
wait on 1558 (Opus 5.5) and 1555 (Sonnet 5.5); at twenty it spawns `steer`.**

## The queue

In dependency order, headline-nearest first. **Two lanes**: the newest
pair's (1558) and the third map's (1555); 1497 waits for a free lane,
lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1558. **The coverage pair's word: frame 1277, 10 versus 15 draws**,
    index 2: ours `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
    `Unit::explore_goody+0x27c` (five). Inside run679 (block 1278),
    widened by `run679_s_word_frame_is_widened_whole`. The scout 1/0
    (`TypeIndex` 77) parts on run678's block 1201: the original turns
    it to (29592, 26520), ours keeps (24312, 26616) — 1552's
    hypothesis, the goody sweep answering on 1200. No mechanism.

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
