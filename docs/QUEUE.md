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

*2026-09-27, the commander (Opus 5.5), seventeen landings since the
sixteenth pass: **both long captures are closed at the trace's end.**
Great Lakes 20568 → 24000 (795, 899) and East Indies 20007 → 24000 (880,
890, 904, 919): every frame's draw stream agrees on both maps (275,108
and 295,910 draws, asserted), and both endpoints are 0 off. What stands
is value residue: 293 rows on East Indies' 23960 and 310 on Great Lakes',
each standing since before its last word. **Chapters twenty-five to
thirty closed.** The AI lane turned to the blind list, the
commander's call on the user's word (the next pass may overturn it):
**228 cited never entered, 29 of them residue with reasons**, pinned in
`rondata::blind` (923, 935, 940).*

- **The mechanisms**, each with its value diff: 945 `come_out`'s
  re-seat at a building point (ch30 105 → 35 rows); 915 CARRY_AIR, 919 an
  unstarted site (AI §79), 904 a rock cell (§78), 899 `is_attack`
  (PATHFINDER §29), 890 the wonder count (AI §77), 795 `do_move`'s TAKE,
  880 the kept fields, 882 `come_out`'s push (GROUPS §30–§32).
- **Fable backlog: 36 Loop items** (677, 685, 697, 745, 775, 799, 838, 852, 889, 894, 898, 900, 903, 907, 910, 914, 918, 922, 926, 927, 932, 933, 936, 937, 938, 939, 941, 942, 943, 944, 949, 950, 952, 953, 954, 958).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · 955 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: 934 is live on the AI lane (ref ab7115e3; nineteen chapters at
`cover=1`, run319–run337); merge it when it reports. 955 on the rules
lane: the gather point's other arms in one capture, run338. The count
runs from f0b9d296: seventeen; the steering pass is due at twenty.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

934. **The blind list's rows 1, 2 and 5: the tracer's issuer guard** (77
    functions). `cover=1`'s `jmp` overwrites the prologue the `@` issuer
    guard checks, so every issuer is refused with code 2 (run314). Read
    the displaced bytes from the stub, then re-run each issuer chapter's
    script at `cover=1` on the queue lane, `rngcmp` against its golden
    run, chapter twenty-three first. Eleven of the 77 are issuers the game
    never calls, only the DLL (940); say what entering them counts for.

955. **Chapter thirty-one: the gather point's other arms — no capture
    yet** (928's and 945's parks). One capture, the arms placed between
    staged events: a lone unit trained under a ground point (FILTER_ALL's
    seeker, 956), a unit found at a ground point (the third re-seat,
    957), a list of two points (946), an Airbase's gather point (947),
    and a citizen's build, repair and gather arms (948). The emulator
    first; **run338** the capture, run339 a staging run. GOLDEN §40.

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
