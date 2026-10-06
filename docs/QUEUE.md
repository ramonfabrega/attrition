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

*2026-10-06, the twenty-fifth pass (`docs/audit/2026-10-06-fable-pass-25.md`):
twenty landings in 13 h 09 m, three lanes, 16.5 USD a landing. French East
Indies **closed at 17,379** (1514, 1519); Toughest **15378** (1510, 1517, 1522, 1524), floor 15378; fifty-one chapters
closed; the battery 576 / 6566 / 2974, unmoved by fifteen later landings.*

- **Two lanes this tranche** (DECISIONS 61 §5: a landing waited 77.5
  minutes against 50; DECISIONS 62): the newest pair's on **Opus 5.5** —
  1514, then the coverage pair's word 1511 when the French pair closes —
  and Toughest's on **Sonnet 5.5**, `--effort high` both; 1497 waits for a
  lane. The roster is the user's word of 2026-10-06: the mix runs once
  more, judged at the twenty-sixth pass, with 1464's trial and 1139.
- **The coverage pair is the newest pair** (DECISIONS 62 §3): the French
  pair closed on 1519; 1511 put its row in `AI_WORDS`; 583 since 1530.
- **Built**: `lanewait` wakes on a push only when the row has stopped;
  `waitrun.sh` calls a traceback with no receipt a dead take; the runner
  gives up on a second stall; `ron_lane_state` exits 1 held; `CLAUDE.md`:
  a merge-moved pin is no kill, the push is read back, a section stub lands.
- **Fable backlog: 23 Loop items** (1119, 1138, 1139, 1316, 1421, 1450, 1462, 1464, 1467, 1475, 1478, 1482, 1485, 1490, 1507, 1513, 1516, 1518, 1521, 1526, 1527, 1529, 1534).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w583 of 4,730
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15378 of 15,432
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-sixth tranche — `commander`, spawned by the pass: two
lanes, 1532 on Opus 5.5 and 1528 on Sonnet 5.5; at twenty it spawns `steer`.**

## The queue

In dependency order, headline-nearest first. **Two lanes**: the newest
pair's (1532) and the third map's (1528); 1497 waits for a free lane,
lower map first — East Indies (All Technologies).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1528. **Great Sahara at Toughest's word: frame 15378, 14 versus 51
    draws**, index 2: ours `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`,
    theirs `Unit::do_group_move+0xb03`. run668 (15230..15433) widens it
    on block 15379: on 15377, 77 units' orders and group 65's record
    change beside 0/2000's `reduce_stamp`, which nothing here writes.
    54 frames from the game's end (DECISIONS 55). No mechanism.

1532. **The coverage pair's word: frame 583, 198 versus 5 draws**,
    index 0: ours `PathFinder::calc_road_cost+0x46`, theirs
    `Farms::inc_time+0x1ae`; both search a caravan road on 582, ours
    again on 583. No East Indies coverage capture holds 583 (run669 ends
    at 436): a capture
    over 577..833 widens it first. The state first parts on block 183:
    1/10's order stack, 4 against 6 — ours three Builds behind one
    `ExploreTo`, theirs each behind its own. No mechanism.

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
