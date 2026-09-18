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

*2026-09-17 into 09-18, an Opus commander loop: nineteen landings, thirteen
word-moving. **GL +725, EI +683.** Next: 346.*

- **Great Lakes 8382 → 8404** (item 344): the mine's site test is a
  mountain-tile distance, not the camp's cell survey, and its mining list
  is the range whole — 207 tiles, run80 and run97 agreeing to the tile.
  8382 is now 865 draws against 865, entry for entry.
- **An endpoint diff is a weaker oracle than a frame below the word**, and
  this corrects yesterday's lesson rather than repeating it. 338's scholar
  value diff fell from six exact to **four** — not because the seating
  changed (`1/44`, its own unit, is still exact) but because 24001 sits
  15,600 frames past the word, where both sides are on streams that are
  nobody's and any unrelated change moves the total. Use the endpoint for
  persistent state no frame dumps; prefer a frame below the word when one
  exists. **run97 now means one usually does.**
- **run97 is read by no test** — Great Lakes `[8030, 9349]` at rich detail,
  BUILDDATA with mining lists, UNITDATA, GUY clocks, PATHDATA, every order
  type, spanning the word both sides. All that has been taken from it is
  one building record read by hand. That is the lane's real payoff and it
  is unspent.
- Two items open, thirty parked. **Fable backlog: eleven Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345). 318+339 are one
  question; 341+343 are one; 345 moves the commander's own verbs into git.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8495 of 24,000 · GreatLakes w8404 of 24,000
Endpoint 24001: EastIndies 70 off, 2 unlinked · GreatLakes 62 off, 7 unlinked

**Opener: 346 — 8404 is one missing `Guy::inc_time+0x271` wrap, ours 6
draws against 7, and run97 dumps the frame. Widen it there first.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

346. **Great Lakes 8404 is one animation wrap, and run97 dumps it** —
    ours 6 draws against the original's 7 at `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`, named by 344 on the frame. run97 `[8030, 9349]`
    spans it at rich detail and **no test reads that capture yet**, so this
    is also the first frame-by-frame widening on it. A wrap is a clock
    question (`docs/ANIM.md`): compare `cur_time`, `end_time` and
    `last_time` either side, every unit, before reading anything.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue. Its value diff now reads four exact of eleven rather than six,
    for the endpoint reason above, so **re-measure before diagnosing**: the
    vector `1/2020 − 1/2019` is the claim, not the count. Book after 346
    unless 346 names it.

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
