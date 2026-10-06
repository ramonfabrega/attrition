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

*2026-10-06, **the twenty-fifth pass is due**: twenty landings since the
twenty-fourth (80285dfe), 19:05 to 08:14, 13 h 09 m, three lanes. French
East Indies **17244** of 17,379 (was 12794); Toughest **14512**, floor
14512 (was 11985); fifty-one chapters closed; battery 576 / 6566 /
2974 (a measure); the coverage pair at frame 177 (from 0).*

- **The roster, lane by lane**: Opus 5.5 on the newest pair, nine
  landings, +4450; Sonnet 5.5 on the third map, five, +2527, and on the
  rules lane six (ch51, 1468's verdicts, the battery, the coverage pair
  stood up and moved 0 → 177). No kill rule counted as tripped: 1429's
  booking gate was red for two landings together (the pass rules, 1469);
  1429 stalled 49 min on a permission prompt and asked a person.
- **Merges**: seven of twenty backed out on `docs/AI.md` sections or
  `diff::*` pins and cost a second lane gate (1491); `ccc push` answered
  "nothing to push" with origin behind (1492) — pushed with git since.
- **The clear at ten** was not armed (DECISIONS 61 §3: refused, not retried).
- **The user's**: the Sonnet lanes at this steer; 1464's model; 1139;
  whether the coverage pair is scored, and on which model (1498).
- **Fable backlog: 25 Loop items** (1119, 1138, 1139, 1225, 1316, 1421, 1450, 1462, 1464, 1467, 1469, 1471, 1475, 1478, 1482, 1485, 1488, 1490, 1491, 1492, 1495, 1498, 1501, 1507, 1513).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17244 of 17,379 · GreatLakesFrench w5638 of 5,638
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w14512 of 15,432
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the twenty-fifth steering pass — `steer`, spawned by the
commander at twenty landings; it writes the next commander's opener.**

## The queue

In dependency order, headline-nearest first. **Three tracks, a lane
each**: the newest pair's, the third map's, and the rules lane's coverage items,
lower map first — East Indies (French).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1514. **French East Indies frame 17244, 9 versus 6 draws**, index 0:
    ours `Guy::set_anim+0x97a < Unit::do_move+0x11cf`, theirs
    `< Unit::do_idle+0x7d`. run661 widens it (254 keys); the first
    parting near it is 1/67's move `pause` on block 17244. 135 frames
    from the game's end: a word that reaches 17,379 closes the pair
    with its closing state scored (DECISIONS 55). No mechanism.

1510. **Great Sahara at Toughest's word: frame 14512, 53 versus 53
    draws in another order**, index 40: ours `Guy::move+0x19f`, theirs
    `Guy::do_turn+0x4a < Guy::turn_towards+0x69`, in 1/141's (a
    Bombard's) four figures; the count first parts at 15101. run659
    (14506..14529) widens it: 83 keys, all standing on its first block,
    none on 14513 or after — no dumped record parts. A packet at 14512
    or a per-figure read is the next question; no mechanism is named.

1511. **The coverage pair's word: frame 177, 9 versus 8 draws**,
    index 2: ours `Leader::make_stuff+0x221` three times, theirs twice
    then `Guy::set_anim+0x97a` — `make_me` fills three Village slots
    here (t 414: slots 0, 1, 9) against two. run660 (blocks 171..184).
    Leader 1's `SITE` list stands since block 1 (parked 1506: `SITE[2]`
    blank there, (50, 57) here; `SITE[9].val` 797 against 375130 on
    176) — a hypothesis, not yet shown the only cause. No mechanism.

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
