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

*2026-09-17 into 09-18, an Opus commander loop: twenty-four landings,
eighteen word-moving. **GL +1,306, EI +1,899.** Next: 358.*

- **Great Lakes 8663 → 8985** (item 354), and run97's point-and-goal residue
  **601 → 6**. `find_wpath`'s `army` mode was off for AI armies, so they
  stopped paying 32x for NEARBLOCK cells. Great Lakes' endpoint is now
  **0 unlinked** — every unit at 24001 has a counterpart.
- **A virtual that is `return 0` everywhere is not a predicate.**
  `UnitData::is_attacking` calls the current order's `+0x18`, and this
  crate read it as "has a combat target". Verified here: **all seventeen**
  order vtables land on `Window::get_button`, a bare `return 0` — fifteen
  directly, and `AttackGroundOrder`/`AirAttackGroundOrder` through a
  `vtordisp` thunk that *looks* like a real implementation and calls the
  same stub. COMDAT folding chose the surviving name; read the body.
- **Both maps now need captures, and Great Lakes is the newer problem.**
  run97 ends at 9349 and the word is 8985 — **364 frames of runway**, less
  than this one item moved. East Indies has had none above 8789 for seven
  items. Awaiting the go; it is the lane's, not a worker's.
- Two items open, thirty-two parked. **Fable backlog: twelve Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345, 356).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w8985 of 24,000
Endpoint 24001: EastIndies 61 off, 11 unlinked · GreatLakes 67 off, 0 unlinked

**Opener: 358 — widen Great Lakes 8985. run97 still dumps it, but only just:
364 frames of runway left on that capture.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

358. **Widen Great Lakes 8985** — the headline's own frame, which no item
    names. Both sequences whole, every record, every field, every unit, then
    the cause. run97 `[8030, 9349]` still dumps it, with **364 frames above
    and no more** — say in the report whether the cause needed frames past
    9349, because that decides whether the next capture is owed urgently.

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
