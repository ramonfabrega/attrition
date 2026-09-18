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

*2026-09-17, an Opus commander loop: fifteen landings, ten word-moving.
**EI +654, GL +593.** Next: 338. The day's price is parked (330).*

- **Great Lakes 8201 → 8272** (item 336): `Unit::fight@005fd4d0` snaps the
  unit onto its 48-unit cell centre at its own entry, one call `do_attack`
  never made. Great Lakes stays the headline; East Indies sits at 8466,
  so the maps have swapped twice in a day.
- **Both words are now past every detail capture on disk** — Great Lakes'
  nearest is run19's 8201, **71** frames below; East Indies' run90's 7916,
  **550** below. The value diff, which tells a right destination from a
  wrong one the draw stream agrees with, is unavailable at the frontier
  until a capture is taken. This file's rule *books* it; spending a lane
  on it is the next structural call.
- **A self-written trailer was verified for the first time**: 336 wrote
  `Claude Opus 5`, its transcript says so. Briefs no longer dictate it (335).
- **A worker's gate can be green while the tree is red.** 336 reported the
  rondata half, "299 passed / 1 failed", and never saw its §7.11 put
  `ORDERS.md` §7 over the size ceiling. Split on merge; the reporting gap
  is worth a guard row.
- One item open, twenty-seven parked. **Fable backlog: seven Loop items in
  `docs/PARKED.md`** (313, 318, 321, 330, 331, 332, 335), invisible at boot
  until this line. 330/331 carry the price, 335 the trailer's sources.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8466 of 24,000 · GreatLakes w8272 of 24,000
Endpoint 24001: EastIndies 66 off, 13 unlinked · GreatLakes 64 off, 17 unlinked

**Opener: 338 — widen Great Lakes 8272. Nothing dumps it (run19 stops at
8201), so it is trace-first like 334, and its other product is the capture
brief the frontier now needs on both maps.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

338. **Great Lakes 8272, and the frontier has outrun the captures** —
    336 moved the word past run19's last block, so **no dumped record
    covers it**; the nearest is 8201, 71 frames below. Trace-first, then,
    exactly like 334: `RON_DEBUG_SITES`/`_FOLD`/`_UNIT` on run53's test,
    both sequences whole, every unit, then the cause. Its other product is
    the **capture brief** both maps now need — the window, the categories,
    the rows that would refuse the reading. Parked 333 (`1/28`'s stack 41
    against run19's 42) stays a candidate, never an assumption.

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
