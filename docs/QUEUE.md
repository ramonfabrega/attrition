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
`docs/audit/2026-10-05-fable-pass-24.md`), after a steer with Ramon, and eleven
landings since. French East Indies **15344** of 17,379 (was 12794);
Toughest **12816**, floor 12816 (was 11985); fifty-one chapters closed.*

- **Landed**: 1461, 1470, 1476, 1479, 1481 (Opus, newest pair, now 1487); 1429, 1472,
  1477 (Sonnet, third map, now 1493); 1423, 1468, 1465 (Sonnet, rules;
  the battery reads 576 / 6566 / 2974, a measure). Lanes:
  the newest pair's on Opus 5.5, the others on Sonnet 5.5, `--effort
  high` each; entry 58's two kill rules ride with each Sonnet lane.
- **You are `commander`**, spawned by the pass: wait on
  `tools/lanewait.py`, and at twenty landings, lanes drained and gate
  green, spawn `steer` (`CLAUDE.md`, "spawn each other").
- **The rules lane is the coverage lane**: 1466.
- **Ratified** (parked 1447, closed): nine blind readings on Opus;
  every captured predicate agrees; 42 code-changing rows are 1468.
- **The user's**: the Sonnet lanes at the next steer; 1464's model; 1139.
- **Fable backlog: 21 Loop items** (1119, 1138, 1139, 1225, 1316, 1421, 1450, 1462, 1464, 1467, 1469, 1471, 1475, 1478, 1482, 1485, 1488, 1490, 1491, 1492, 1495).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w15344 of 17,379 · GreatLakesFrench w5638 of 5,638
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w12816 of 15,432
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

1487. **French East Indies frame 15344, 44 versus 38 draws**, index
    0: ours `Army::find_target+0x7df`, theirs
    `Animal::think_bird+0x82`. run642 (15339..15351, 1818 keys): 100
    standing on 15339, nothing on 15340..15344, 1447 on 15345 — who=1's
    army group 64 is the original's alone, and about 30 of who=1's
    units' orders and positions part. Date the first parting; no
    mechanism is named.

1493. **Great Sahara at Toughest's word: frame 12816, 41 versus 40
    draws**, index 33: ours `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
    Guy::turn_towards+0x69`, theirs `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`. run640's second take (12811..12834) widens
    it: on block 12817 army 65's formation (`group:65.curr/off`, 27
    slots) and 1/141's `g.end_time[2]` 31 against 23 part. No mechanism
    is named.

1466. **The coverage pair** (DECISIONS 61 §6): the next pair is chosen
    by `tools/census.py --never`, not by adjacency. First whether the
    click-free lane can set a late starting age (the `GAME INFO` line
    that carries it, named in the stanza); then the lobby that enters
    the most never-entered rows — air, oil, the later ages' orders —
    stood up as 1442 stood the French pair up. Opens after 1465, or
    when French East Indies or Toughest closes.

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
