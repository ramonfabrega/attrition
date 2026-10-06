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

*2026-10-05, **the twenty-fourth pass** (DECISIONS 61,
`docs/audit/2026-10-05-fable-pass-24.md`), after a steer with Ramon, and seventeen
landings since. French East Indies **17171** of 17,379 (was 12794);
Toughest **14363**, floor 14363 (was 11985); fifty-one chapters closed.*

- **Landed**: 1461, 1470, 1476, 1479, 1481, 1487, 1500, 1502 (Opus, newest pair, now 1508); 1429, 1472,
  1477, 1493 (Sonnet, third map, now 1503); 1423, 1468, 1465, 1466, 1496 (Sonnet, rules;
  battery 576 / 6566 / 2974, a measure; coverage pair frame 8, now 1505). Lanes:
  the newest pair's on Opus 5.5, the others on Sonnet 5.5, `--effort
  high` each; entry 58's two kill rules ride with each Sonnet lane.
- **You are `commander`**, spawned by the pass: wait on
  `tools/lanewait.py`, and at twenty landings, lanes drained and gate
  green, spawn `steer` (`CLAUDE.md`, "spawn each other").
- **The rules lane is the coverage lane**: 1505, then 1497.
- **Ratified** (parked 1447, closed): nine blind readings on Opus;
  every captured predicate agrees; 42 code-changing rows are 1468.
- **The user's**: the Sonnet lanes at the next steer; 1464's model; 1139.
- **Fable backlog: 24 Loop items** (1119, 1138, 1139, 1225, 1316, 1421, 1450, 1462, 1464, 1467, 1469, 1471, 1475, 1478, 1482, 1485, 1488, 1490, 1491, 1492, 1495, 1498, 1501, 1507).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17171 of 17,379 · GreatLakesFrench w5638 of 5,638
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w14363 of 15,432
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: spawn 1461 on `claude-opus-5-5[1m]` and 1429 and 1423 on
`claude-sonnet-5-5[1m]`, `--effort high` each, off this commit; wait on
`tools/lanewait.py`; at twenty landings spawn the pass.**

## The queue

In dependency order, headline-nearest first. **Three tracks, a lane
each**: the newest pair's, the third map's, and the rules lane's coverage items,
lower map first — East Indies (French).
Take the first unstarted item unless a better order is clear, and state why.
The backlog is `docs/PARKED.md`, back only when a score names it.

1508. **French East Indies frame 17171, 11 versus 10 draws**, index
    1, dated by 1502 (run658): leader 0's dirty bit (`leader_flags`
    0x2000000) rises there on 17026..17032 and never here; the first
    strike on player 0 (1/69 on Napata, 17053) sets treaties bit 2 and
    Napata's `raid_stamp` there. The word's chain is a third event: an
    attack pushed onto a guard's AttackTo leg, where ours keeps the walk
    and the original drops it (run657, 1/69 on 17167). No mechanism.

1503. **Great Sahara at Toughest's word: frame 14363, 11 versus 177
    draws**, index 5: theirs `PathFinder::calc_road_cost+0x46`. run653
    (14357..14380) widens it: on block 14364 six of who=1's buildings
    part on `regen_roads` (1/2031, 2034, 2038, 2040, 2042, 2054 set
    there; 1/2053 set here only), and 1/66 and 1/115's order kind 6
    against 7. Date the first parting; no mechanism is named.

1505. **The coverage pair's word: frame 8, 21 versus 24 draws**,
    index 2: ours `Leader::make_stuff+0x63d`, theirs
    `Leader::produce_building+0x1805`. run656 (blocks 1..33) widens
    it: on block 9 the original buys 2006 a city, 2007 a Shipyard,
    2008 an Oil Well (190, 202), 2009 a University, 2010 and 2011
    Airbases; ours buys four and no well (`ai_place` 4.2 never places
    an Oil Well; `oil_patches` unmodelled). Not on the scoreboard
    (parked 1498). No mechanism is named.

1497. **Chapter fifty-two: the arms the unit tests hold and no walk**
    (DECISIONS 56 §3): parked 1473's member `find_melee_target` arm and
    `update_local_seen_build`'s fog arm, 1476's Construction hp arm
    (`myhits`, `construct_hits` unread), and 1500's `move_step` snap's
    `< 2` order-count bound (the two-order converse). Whether one
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
