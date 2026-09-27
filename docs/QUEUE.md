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

*2026-09-26, the commander (Opus 5.5), seven landings since the sixteenth
pass: **East Indies 20007 → 23182 (880, 890); Great Lakes 20568 → 20800
(795)**, and Great Lakes is the lower map; 899 is live on it and reports
20800 → 24000, the trace's end, not yet gated. **Chapters twenty-five to
twenty-seven closed** (884 at 1466, 883 at 1492, 901 at 1560).*

- **901**: a unit upgrade through the player's command; `gain_tech`'s
  queue loop re-targets the line's queued entries (TECH's "The queue
  loop"); run300, no falsifier fired; the word 1102 → 1560, closed.
- **890**: `get_cost`'s wonder count (COSTS "A wonder is ramped by every
  wonder", AI §77): on 20782 who=1's `MAKE[0]/[1]/[8]` read 0 on both
  sides (ours was 486/398/486); and the gather offer escrowed on every
  exit of `create_buildings`' head test: on 23182 `MAKE[0]/[4].escrow` 1
  on both sides. **795**: `do_move`'s TAKE (GROUPS §32). **880**: the
  pool's kept fields (§30). **882**: `come_out`'s push (§31).
- **883**: the player's research; `diff::leader` compares ages, epochs
  and discovered on every window. **884**: the player's cancel.
- **Fable backlog: 15 Loop items** (677, 685, 697, 745, 775, 799, 838, 852, 889, 894, 898, 900, 903, 907, 910).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w23182 of 24,000 · GreatLakes w20800 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · 888 next
Endpoint 24001: EastIndies 38 off, 0 unlinked · GreatLakes 14 off, 0 unlinked

**Opener: 899 is live on the AI lane (ref 17f9682d); merge it when it
reports, with the closed word's evidence. 888 on the rules lane: two
Barracks under one `@queueup`, the emulator first, then run304. 904 (East
Indies' 23182) after 899. A brief names only the fenced modules; a
capture is waited on with `tools/gamelog/waitrun.sh`; a journal's "for
the Loop" line is filed at its merge. The count runs from f0b9d296: seven.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

899. **Great Lakes' word is 20800** (795 moved it 20568 → 20800:
    `do_move`'s TAKE, GROUPS §32). On block 20801 of run243 `1/60` stands
    blocked by `1/64` in the original and walks here. **run243 has 18
    blocks of runway past it** (it ends on 20818), and `1/60`'s world
    route already differs below run294's first block, 19840: the parting
    is in 17351..19839 (796, narrowed). No mechanism.

904. **East Indies' word is 23182** (890 moved it 20782 → 23182:
    `get_cost`'s wonder count and the gather escrow, AI §77). On 23182
    ours 49 draws against 48, parting at index 46:
    `produce_building+0x1805` here against `make_stuff+0x63d`. run299
    (23177..23433) holds it, widened. 890's reading (§77.6): slot 4's Farm
    jitter at corner (184, 196), 4 clear sub-positions here against 3; a
    hypothesis. A packet at 23181, inside the second `produce_building`.

888. **Chapter twenty-eight: two buildings under one command — an
    issuer, no capture yet** (882's and 883's park). `@queueup` on a
    selection of two Barracks: `action_queue_up`'s sort by `queued`,
    least first, and `num` laid across the members; and the command's
    building group of two, which `push_command_buildings` does not seat
    (its `SEAM:`, GROUPS §31). The emulator's two-member rows first;
    **run304** the capture, run305 a staging run. GOLDEN §37.

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
