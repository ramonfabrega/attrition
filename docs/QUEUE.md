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

*2026-09-27, the commander (Opus 5.5), twelve landings since the
sixteenth pass: **both long captures are closed at the trace's end.**
Great Lakes 20568 → 24000 (795, 899) and East Indies 20007 → 24000 (880,
890, 904, 919): every frame's draw stream agrees on both maps (275,108
and 295,910 draws, asserted), and both endpoints are 0 off. What stands
is value residue: 293 rows on East Indies' 23960 and 310 on Great Lakes',
each standing since before its last word. **Chapters twenty-five to
twenty-nine closed.** The AI lane turns to the blind list (923), the
commander's call on the user's word, 2026-09-27; the next pass may
overturn it.*

- **The mechanisms**, each landed with its value diff: 915 a trained
  aircraft stays in its Airbase (`Build::train`'s CARRY_AIR arm); 919 an unstarted
  site is no farm (AI §79); 904 a rock cell
  refuses a non-oil type (`blocked_tcoord`, AI §78); 899 `is_attack`
  and army mode (PATHFINDER §29); 890 the wonder count and the gather
  escrow (AI §77); 795 `do_move`'s TAKE (GROUPS §32); 880 the pool's kept
  fields (§30); 882 `come_out`'s push (§31); 884, 883, 901, 888 the
  player's cancel, research, upgrade and two-building command.
- **Fable backlog: 22 Loop items** (677, 685, 697, 745, 775, 799, 838, 852, 889, 894, 898, 900, 903, 907, 910, 914, 918, 922, 926, 927, 932, 933).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · 928 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: 928 on the rules lane (the gather point, run312) and 923 on
the AI lane (the blind list, run314); merge each when it reports.
`cover=1` works on the queue lane only (697). A brief names only the
fenced modules. The count runs from f0b9d296: twelve landings.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

923. **The blind list toward an enumerated residue** (DECISIONS 29,
    41): both long words are closed at 24000, so the AI lane measures the
    cited-never-entered functions (census 2026-09-25: cited 1,069,
    entered 7,180, three traces), ranks them by family, and takes the
    first targeted `cover=1` capture on the queue lane, **run314**, most
    likely a golden chapter's script re-run. The commander's call on the
    user's word; the value residue (293 and 310 standing rows) waits.

928. **Chapter thirty: the gather point — an issuer with no DLL verb, no
    capture yet** (915's park). `issue_gather_point` sets a building's
    rally point; `Build::train`'s patrol-with-the-action-bit arm and its
    strike arm, and `come_out`'s gather-point routing for every trained
    unit, read it. A new DLL verb, the emulator first; **run312** the
    capture, run313 a staging run. GOLDEN §39, and §14's row in a
    continuation (932).

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
