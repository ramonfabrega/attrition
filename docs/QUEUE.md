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

*2026-10-06, the twenty-sixth steering pass done (`docs/audit/2026-10-06-fable-pass-26.md`,
DECISIONS 63): French East Indies **closed at 17,379**; Toughest **closed
at 15,432**; Great Sahara in the coverage lobby **1582**; the coverage
pair **1408**. The twenty-seventh tranche counts from the pass's commit.*

- **Three lanes** (DECISIONS 63 §2): the newest pair's (1563, Opus
  5.5), the third map's (1565, Sonnet 5.5) and the rules track's (1497,
  Sonnet 5.5), `--effort high` all. Next run 684, next section §155.
- **1563's first row is 1450's**: the shared widening takes `ever_seen`
  and `ever_seen_completed` (1558's cause was a vision gate).
- **The coverage pair is the newest pair** (DECISIONS 62 §3).
- **A lane that lands second takes `ccc update att-<n> --keep-conflicts`**
  on a section stub and keeps both (seven lanes last tranche).
- **A `--replace` spawn archives the row it stopped** (`ccc archive
  <ref>`; CLAUDE.md's clause, parked 1541 closed).
- **Fable backlog: 23 Loop items** (1119, 1138, 1139, 1316, 1421, 1450, 1462, 1464, 1467, 1475, 1478, 1485, 1490, 1507, 1516, 1518, 1521, 1534, 1537, 1548, 1554, 1560, 1566).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w1408 of 4,730
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w1582 of 4,340
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-seventh tranche — `commander` (Opus 5.5): spawn
1563 (Opus 5.5), 1565 (Sonnet 5.5) and 1497 (Sonnet 5.5), each
`--effort high`; the chain is `CLAUDE.md`'s; stop at twenty landings,
counted by `git log` from the pass's commit, and spawn `steer`.**

## The queue

In dependency order, headline-nearest first. **Three lanes**: the newest
pair's (1563), the third map's (1565) and the rules track's (1497),
lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1563. **The coverage pair's word: frame 1408, 39 versus 40 draws**,
    index 25: ours `Objects::process_all+0x2df`, theirs `Guy::set_anim
    < Unit::do_cast+0xc89`, then `Guy::init_real`. Inside run679 (block
    1409), widened by `run679_s_word_frame_is_widened_whole`: citizen
    1/8 casts its transport into barge 1/46 in the original only. 1/8
    parts first on block 1340 (`ExploreTo` (37416, 33864) against
    (36168, 35976)) — 1558's hypothesis. No mechanism.

1565. **Great Sahara in the coverage lobby: frame 1582, 28 versus 29
    draws**, index 8: ours `Leader::make_stuff+0x63d`, theirs
    `Leader::produce_building+0x1805`. Inside run683 (block 1583),
    widened by `run683_s_word_frame_is_widened_whole`: the Persian
    Refinery 1/2045's site, y 22176 against 19872, city 1 against 2.
    No mechanism.

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
