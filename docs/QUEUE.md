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

*2026-09-26, the commander (Opus 5.5), two landings since the sixteenth
pass: **East Indies 20007 → 20782 (880); Great Lakes holds 20568 and is
now the lower map**, so 795 is the headline. 884 is live on the rules
lane.*

- **880** landed §28's kept fields (GROUPS §30): `Group::kill` clears an
  emptied pool record, and `Army::add_unit`'s group is its slot's record
  (874 built). Every `GROUPDATA` `facing` and `order_num` on every East
  Indies dump agrees to 20257; Great Lakes' walk parts on no record. The
  value diff on 20002: slot 69's `order_num` 6 and `facing` 1 on both
  sides (ours was 1/0), and `1/64`..`1/66`'s orders' `facing` 0 on both.
- **882** built `Unit::come_out`'s push for every owner (GROUPS §31):
  chapter twenty-four's standing rows 34 → 25, thirteen long pins closed
  and none opened; 561 closed. What stands there is 646 and 887.
- **Fable backlog: 10 Loop items** (677, 685, 697, 745, 775, 799, 838, 852, 889, 894).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w20782 of 24,000 · GreatLakes w20568 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · 884 next
Endpoint 24001: EastIndies 37 off, 2 unlinked · GreatLakes 11 off, 0 unlinked

**Opener: 884 is live on the rules lane (ref ba89613b); merge it when it
reports. 795 on the AI lane (Great Lakes is lower now): run243's 20568,
widened, no mechanism. 890 (East Indies' 20782) after 795. A brief names
only the fenced modules; a capture is waited on with
`tools/gamelog/waitrun.sh`; a journal's "for the Loop" line is filed at
its merge. The count runs from f0b9d296: two landings.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

795. **Great Lakes' word is 20568** (785 moved it 17181 → 20568:
    `Wonders::init_wonder` raises `wonder_mark`, AI §75). On 20568 ours 37
    draws against 38 at index 31: the original's is `1/40`'s blocked step
    by `8/0`. `1/40` is already off on 20500, run243's first block (the
    gap, 796). Inside run243 ([20500, 20819)), widened. No mechanism.

890. **East Indies' word is 20782** (880 moved it 20007 → 20782: the
    kept fields, GROUPS §30). On 20782 ours 8 draws against 1 at index 0:
    `Leader::use_market+0x1ed` here against `Farms::inc_time+0x1ae`.
    run289 holds the word, widened: who=1's `MAKE` list parts first on
    20782, and the only stock row parting before it is the standing
    `leftover[2:wealth]` (851). No mechanism.

884. **Chapter twenty-five: the cancel line — an issuer, no capture yet**
    (877's park). The player's `action_unqueue` on a Barracks queue of
    two, and a single cancel on an infinite queue, which 877 read as
    turning the bit off and removing nothing: a claim to check, not a
    premise. The refund's arithmetic under the emulator first, and
    `input::Stream`'s skipped `QueueUp`/`Unqueue` beside it. **run292**
    the capture, run293 a staging run. GOLDEN §34. 883 (the research arm) after.

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
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
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
  to a file, never piped — a pipe launders the 137 — ending `Gate steps:`. A
  clear is free once every landed branch is merged, gated, pushed and reaped.
