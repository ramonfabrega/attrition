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

*2026-09-17 into 09-18, an Opus commander loop: seventeen landings, eleven
word-moving. **GL +695, EI +683.** Next: 340. A capture lane is running.*

- **One mechanic moved both maps** (item 338): the game's **first scholar**
  seating itself. `Unit::go_inside`'s scholar-gated tail snaps it to its
  host and forces the idle roll — a fourteenth caller of
  `Guy::set_anim+0x97a` that sites never carried. GL 8272 → **8374**, EI
  8466 → **8495**. Great Lakes stays the headline.
- **A value diff need not sit on the word's frame.** 338's proof came from
  run80's *endpoint*, 23960–24001, where 14 scholars sit on their
  universities' points — not from 8272, which nothing dumps. Persistent
  state is answered at the endpoint, cheaply; only a frame-local cause
  (336's snap) needs the frame. The commander's "the frontier has outrun
  the captures" brief was too narrow, and 338 disproved it same session.
- **The residue is exact, which names the successor**: four scholars off by
  precisely `1/2020 − 1/2019`, seated on the *other* university. The
  arithmetic is right and **host choice** is what is left (342).
- **A worker's gate never runs the sim suite on a word-moving item** —
  no `--no-fail-fast`, rondata red by design, 831 sim tests skipped. It
  cost 336 an unseen red guard. Parked as **339**; a one-flag fix.
- Two items open, twenty-eight parked. **Fable backlog: eight Loop items in
  `docs/PARKED.md`** (313, 318, 321, 330, 331, 332, 335, 339). 318 and 339
  are the same family and should be decided together.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8495 of 24,000 · GreatLakes w8374 of 24,000
Endpoint 24001: EastIndies 62 off, 10 unlinked · GreatLakes 61 off, 15 unlinked

**Opener: 340 — widen Great Lakes 8374. Check the endpoint dumps before
calling a frame undumped: that is the mistake 338 caught.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

340. **Great Lakes 8374 is the headline** — 338 took both maps with one
    mechanic, and GL stays the lower word. No item names 8374's frame, so
    the brief is its widening: both sequences whole, every unit, then the
    cause. **Before calling the frame undumped, check the endpoint and the
    late captures** — 338's proof was run80's 24001, not its own frame, and
    the commander's contrary brief was wrong. run19 (8174–8201) and run94
    (7754–8061) are the near neighbours; run80 is the census.

342. **Host choice seats the scholar on the wrong university** — 338's own
    residue, and exact: four scholars off by precisely `1/2020 − 1/2019` at
    run80's 24001, six of eleven already agreeing. The arithmetic is
    settled, so this is a predicate, not a formula. Takes a chain to the
    word only if the seating order feeds it; book after 340 unless 340's
    widening names it.

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
