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

*2026-09-17, an Opus commander loop: thirteen landings, eight
word-moving, ~397 USD. **EI +381, GL +522.** Next: 334, new map.*

- **Great Lakes 8201 crossed East Indies 8193** (item 329). "Lower map
  first" changes hands for the first time, so **East Indies is the
  headline** — and its chain is cold: nothing has worked it since 304,
  and 334 is a widening from scratch, not a successor.
- **Eight of twelve items were strict successors**, each brief its
  predecessor's product — the cheapest briefs this loop has had, and the
  property that just ended. Expect 334 to cost more to brief.
- **An elimination is a claim; a seam comment is a prediction; a seam
  justified by "no capture reaches this yet" has a shelf life.** All
  three bit today, the last a month after it was written.
- **Five comparisons that could not fail** were closed — a symbol against
  a raw hex string, a binary trace grepped for a symbol name, a tree id
  against the dump's `TypeIndex` (nine pins rested on it).
- **Build what a reading specifies, before ratifying it or beside it.**
  324 specified `find_attack_pos` and declined to implement; 328 then
  found four arithmetic errors in that specification by writing the code.
- One item open, twenty-five parked. **Fable backlog: six Loop items in
  `docs/PARKED.md`** (313, 318, 321, 330, 331, 332), invisible at boot
  until this line (332). 330 and 331 carry the day's measurements.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8193 of 24,000 · GreatLakes w8201 of 24,000
Endpoint 24001: EastIndies 73 off, 7 unlinked · GreatLakes 64 off, 15 unlinked

**Opener: 334 — widen East Indies 8193 whole before naming anything. Nine
items today named a mechanism up front and eight were wrong, every one
overturned by the widening inside the session that booked it.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

334. **East Indies 8193 is the headline, and nothing on disk dumps it**
    (loop-334, spawned). Great Lakes crossed it at 8201 (329), so "lower
    map first" changes hands for the first time. The disk was greped
    before the spawn: this map's detail stops at run90's **7916**, 277
    frames short, and run54's `[End Frame]` is `MISC` alone — a draw
    stream and a closing dump, no per-frame record. So the widening is
    of the **trace**: `RON_DEBUG_SITES`/`_FOLD`/`_UNIT` over a window,
    both sequences whole. Its other product is a capture brief.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names the headline's frame, a floor, or
  a takes-chain to one. **The headline slot is never empty**: no item on
  the word's frame means the next brief is the widening of that frame.
  Loop items — tooling, guards, these rules — go to the parked file's Loop
  section and are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes the
  `Scoreboard:` line; the worker's story is `docs/journal/<date>-item-<N>.md`.
- **A number is measured on the tip** — after the worker's last `ccc
  update`, or the report says which tree. 301's +381 was attributed to the
  wrong half against a stale base and would have landed wired.
- **Merge, gate, push, reap — one chain.** A reap left for "before the
  next spawn" was forgotten twice in one session.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together; the guard parses the line against the floors.
- **Overlap a capture's neighbours on purpose.** Six blocks each end is the
  floor (run83); a containing neighbour is free with `--exclude <the
  category you raised>`; `samegame.py` exits 0 when nothing is in common,
  so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE — not only for
  coverage of a frame.** A term zero in every dump is either zero-valued or
  switched off: 117 and 178 were the second.
- **A check on a never-cleared field asserts a CHANGE, not a value** —
  `collide_frame` is a permanent stamp (run85). Test both directions first.
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and this handoff is current with the headline
  measured. **Before a blind fan-out**, grep `CLAUDE.md` and the memory
  index: a subagent inherits both, and no brief quotes this file.
