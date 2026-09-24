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

*2026-09-23, the commander (Opus 5.5) after the thirteenth Fable pass:
**1 landing since 29a46bd.** 673 moved Great Lakes 12536 → 12897
(`astar_path`'s retry gates on `is_move`, PATHFINDER §21.6, AI §65).
Lane att-651 is live on chapter six-b (run175 captured, word 632).*

- **Both maps moved and every golden chapter closed.** East Indies
  11069 → 13640 at 0.034 USD a frame (1893 of them off the disk in one
  landing), Great Lakes 12038 → 12536 at 0.20; the restage, seven-b and
  its control, six and eight all closed. **Great Lakes is the lower map.**
- **The rules track's next axis is the issuer** (GOLDEN §13, DECISIONS
  49): the chapters reached the order family by accident and the census's
  order row has not moved in three passes. After six-b (651), chapter
  nine is the move line through `issue_move_to` (676).
- **Nine Loop items ruled**, each a guard, a tool or a clause: 313, 630,
  638, 639, 645, 649, 656 (`tools/gamelog/waitrun.sh`), 667, 670.
- **Fable backlog: 2 Loop items** (527 with a commitment, 677 the bird's
  proxy); no `FABLE:` marker was filed.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w13640 of 24,000 · GreatLakes w12897 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · 651 next
Endpoint 24001: EastIndies 43 off, 11 unlinked · GreatLakes 34 off, 0 unlinked

**Opener: land 651 when it reports; 678 is live on the AI lane (lower
map first, ahead of 643), and 676 follows 651. A detached capture waits on
`tools/gamelog/waitrun.sh`; a report quotes the gate's `Gate steps:` line.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

678. **Great Lakes' word is 12897: `1/41` against `1/15` on 12898** (673
    moved it 12536 → 12897: the retry roll gates on `is_move`, PATHFINDER
    §21.6). On 12897 ours 8 draws against 9 at index 2: ours `Guy::set_anim
    +0x97a < Guy::inc_time+0x271`, theirs `< Unit::move_step+0x823`. The
    original's `1/41` stands on `1/15`; ours walks on. Above it: 12626's
    plan (23 against 20), 12825's ungroup of `1/40`–`1/42` to kind 2.
    Inside run174, its last but one; a move past 12898 owes run178.

651. **Chapter six-b, the air line from a base: no capture yet** (668
    closed chapter eight at 900: `resolve_unit_collision`'s enemy-ladder
    arm C, COLLISION §14; every golden chapter closed). run168's second
    falsifier fired: an aircraft `add`ed outside a base never moves. Stage
    an `add airbase` for each side, then the aircraft; mint the run at
    spawn. Name the premise's killer and grep its writers, cite any
    loop's bound (GOLDEN §3, point 5). Then pin the word.

643. **East Indies' word is 13640, past every capture** (642 moved it
    11747 → 13640: `Region::go_here` reads the human's `reg_cities`
    through `leader_reg_cities`' recount, TRANSPORT §9.4). On 13640 ours
    34 draws against 33 at index 33: an extra `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`. East Indies' windows end on run159's 11899; run78
    starts at 15700. The `WIDENINGS` row names this item: capture
    **run166** over the word, then widen it whole. No mechanism.

676. **Chapter nine, the move line — the first issuer chapter, no capture
    yet** (DECISIONS 49; GOLDEN §13). `CommandManager::issue_move_to
    @00941720` from the tracer DLL on one unit and on a squad, over land
    with a world plan, `!ai off`, the lab's L15 shape. The issuer runs
    under the emulator first (`tools/emu/callfn.py`), then the pair; the
    premise names its killer and the booking greps its writers (GOLDEN
    §3, point 5). Takes GOLDEN §17; mints its run at spawn. Then pin.

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
