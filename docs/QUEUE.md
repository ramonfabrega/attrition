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

*2026-09-24, the commander after the twelfth pass (Opus 5.5, every worker
Opus 5.5): **20 landings since 6b46125, 17 moved a word, and the count
stops here: the thirteenth Fable pass is due.** Nothing is in flight and
every lane is reaped.*

- **The AI track moved both maps.** East Indies **11069 → 13640** (620,
  629, 642) and passed Great Lakes on 642, so **Great Lakes is the lower
  map**, **12038 → 12536** (571, 657, 661, 669).
- **Every golden chapter is closed.** The restage 782 → 1000 (625, 617,
  627); seven-b and its control captured and closed at 1200 (628, 629,
  632, 644, 647); chapters six and eight captured for the first time and
  closed at 900 (648, 650, 652; 660, 664, 668). Three premises fell,
  seven-b's to a reading before its capture (630). The rules slot now
  holds six-b (651), a restage: **what the rules track is for once every
  chapter closes is the pass's question.**
- **The Great Lakes endpoint rose 42 → 53 off on 669** while its word
  moved; recorded as the number (DECISIONS 36), not traded.
- **A lane sat idle ninety minutes** after its detached capture finished
  (656); the commander caught it off the runner's log.
- **Fable backlog: 10 Loop items** (313, 527, 630, 638, 639, 645, 649, 656, 667, 670); no `FABLE:` marker was filed.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w13640 of 24,000 · GreatLakes w12536 of 24,000
Golden: none pinned, every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · 651 next
Endpoint 24001: EastIndies 45 off, 12 unlinked · GreatLakes 53 off, 0 unlinked

**Opener: the thirteenth Fable pass is due: twenty landings since the
twelfth pass's commit, nothing in flight. After it, spawn 673 on the AI
lane and 651 on the rules lane; 643 follows 673.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

673. **Great Lakes' word is 12536: squad `1/27`–`1/29`'s orders** (669
    moved it 12429 → 12536: `do_move`'s tile arm pops a waypoint only on
    a tolerance of 1..=0x60, unsigned, ORDERS §4.4, AI §64). On 12536
    ours 93 draws against 94 at index 92: ours `Farms::inc_time+0x1ae`,
    theirs `PathFinder::astar_path+0x1697`. On 12537 the squad takes kind
    21 with one entry against the original's kind 2 with a ten-entry
    world plan. Inside run174 (to 12899); no capture owed. No mechanism.

651. **Chapter six-b, the air line from a base: no capture yet** (668
    closed chapter eight at 900: `resolve_unit_collision`'s enemy-ladder
    arm C, COLLISION §14; every golden chapter closed). run168's second
    falsifier fired: an aircraft `add`ed outside a base never moves. Stage
    an `add airbase` for each side, then the aircraft; mint the run at
    spawn. Check each falsifier can fire, and cite any loop's bound
    (parked 667). Then pin the word.

643. **East Indies' word is 13640, past every capture** (642 moved it
    11747 → 13640: `Region::go_here` reads the human's `reg_cities`
    through `leader_reg_cities`' recount, TRANSPORT §9.4). On 13640 ours
    34 draws against 33 at index 33: an extra `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`. East Indies' windows end on run159's 11899; run78
    starts at 15700. The `WIDENINGS` row names this item: capture
    **run166** over the word, then widen it whole. No mechanism.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §14; the run number is
  minted at booking); **a worker parks what names the headline's frame
  and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (a closed chapter reads `chN closed`), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a wait
  keys on the runner's exit, never on a file** (565). **A value question
  takes a packet at the word's own frame, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A clear is free when
  every landed branch is merged, gated, pushed and reaped and none is in flight.
