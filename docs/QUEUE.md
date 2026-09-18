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

*2026-09-17 into 09-18, an Opus commander loop: twenty-one landings, fifteen
word-moving. **GL +940, EI +1,899.** Next: 350.*

- **Great Lakes 8582 → 8619** (item 348): the word was the AI's first market
  draw, and `BUY_SELL`'s prerequisite is **Coinage/Commerce 2**, not the
  Market building or Barter. East Indies unchanged at 9711.
- **Pin a growing residue with its rate, not just its size.** 348 re-pinned
  the walk-slot floor at 5,853 and wrote the arithmetic beside it — **33.5
  fields a frame against the 33.2 it has always cost** — so the next session
  can divide before calling growth a regression. Every item that widens a
  window inherits a bigger residue; this is how the number stays readable.
- **The successor is named with a unit**: 8619 is one extra
  `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, folded to **`1/39`**, against
  five `inc_time` wraps both sides agree. run97 covers it with 730 frames
  above, so the value diff is on disk.
- **East Indies is still owed a capture above 8789** — 922+ frames short
  since 346, its whole window below the word. Great Lakes needs none:
  run97 runs to 9349. Awaiting the go, it is the lane's and not a worker's.
- Two items open, thirty-two parked. **Fable backlog: eleven Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345). 318+339 are one
  question; 341+343 are one; 345 moves the commander's own verbs into git.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w8619 of 24,000
Endpoint 24001: EastIndies 61 off, 11 unlinked · GreatLakes 63 off, 7 unlinked

**Opener: 350 — a strict successor, unit already named: the extra
`do_idle+0x7d` on `1/39` at 8619, with run97 dumping the frame.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

350. **Great Lakes 8619 is one extra idle draw on `1/39`** — 348 named it
    on the frame: `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, folded to that
    unit, against five `inc_time` wraps both sides agree on. A strict
    successor, no widening owed. run97 `[8030, 9349]` dumps 8619 with 730
    frames above, so compare `1/39`'s clocks and order state either side
    before reading `do_idle`.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned twice since. **Re-measure before diagnosing**: the
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
