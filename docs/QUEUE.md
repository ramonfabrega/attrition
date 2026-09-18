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

*2026-09-17 into 09-18, an Opus commander loop: twenty-three landings,
seventeen word-moving. **GL +984, EI +1,899.** Next: 354.*

- **Great Lakes 8628 → 8663, and run97's residues went to almost nothing**
  (item 352): the walk-slot band **2,898 → 0**, point-and-goal **8,672 →
  601**. 8442's muster cell *is* the original's `(58, 29)`, read off the
  `GROUPATTACKTOORDER`'s orig through `do_forming`'s one-tile step.
- **Two predicates were wrong, and one of them was written down and never
  built.** The spacing bound is `<= 4` (`jle` at `6f633a`); and
  `BuildType::mask_me`'s `W.flags |= 0x4000` on a building's own cell had
  been in `docs/CITIES.md` §3.6 **in prose since the first reading, and in
  no code**. Without the bit the ring's score cannot separate two open
  cells, so the earliest tie won. Parked as **356**: nothing checks that a
  documented claim reached the implementation.
- **347 is closed.** The walk-slot band is zero. It was parked as naming no
  score, with a falsifier and a rate; 350 took it 6,258 → 1,747 and 352 took
  the rest. A parked residue is not a discarded one, and the rate is what
  made both steps legible.
- **East Indies is still owed a capture above 8789** — 922+ short since 346,
  five items without a value diff at its frontier. Great Lakes needs none:
  run97 runs to 9349. Awaiting the go; it is the lane's.
- Two items open, thirty-two parked. **Fable backlog: twelve Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345, 356).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w8663 of 24,000
Endpoint 24001: EastIndies 59 off, 12 unlinked · GreatLakes 60 off, 8 unlinked

**Opener: 354 — widen Great Lakes 8663; no item names it and run97 dumps it
with 686 frames above.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

354. **Widen Great Lakes 8663** — the headline's own frame, which no item
    names, so by this file's rule the brief is its widening: both sequences
    whole, every record, every field, every unit, then the cause. run97
    `[8030, 9349]` dumps it with **686 frames above**, and 346, 348, 350 and
    352 have all read that capture, so the machinery exists — extend it.
    run97's residues are now near zero, which makes anything left loud.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times. **Re-measure before diagnosing**: the
    vector `1/2020 − 1/2019` is the claim, never the count.

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
