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

*2026-09-17, an Opus commander loop, twelve landings and seven
word-moving. **East Indies +381, Great Lakes +508.** Next: 329.*

- **Great Lakes 8187**, **East Indies 8193** — six frames apart, from 133
  this morning: +381 (304), +251 (314), +151 (322), +100 (317), +4 (323),
  +1 (319), +1 (328). **Great Lakes is about to stop being the lower map**
  and the queue's "lower map first" order changes with it.
- **Floors take the lower of word and sequence**; ask for figures by
  symbol, since two today needed a quantity attached, not a correction.
- **An elimination is a claim, and a seam comment is a prediction.** 320
  ruled out every named `seen2` writer and 322 found one in `BUILDDATA`;
  `army.rs`'s "nothing changes" seam was 8186's whole frame.
- **Four comparisons that could not fail** were found today: a symbol
  against a raw hex string, a binary trace grepped for a symbol name, a
  tree id against the dump's `TypeIndex` (nine pinned rows rested on it),
  and a site `trace::SITES` modelled nowhere. When a comparison has never
  failed, ask whether it can.
- **Grep the source's citations, not only the log dir.** 323's frame had
  been on disk since 08-25 and `ai_make.rs`'s docstring was what said so.
  Nine items named a mechanism up front and eight were wrong.
- One item open, twenty-four parked.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8193 of 24,000 · GreatLakes w8187 of 24,000
Endpoint 24001: EastIndies 73 off, 7 unlinked · GreatLakes 67 off, 14 unlinked

**Opener: 329 — the pathfinder tail behind 8187, on disk in run19.
328's override is the precedent worth keeping: building §17 found four
arithmetic errors a single reading had missed, so a reading that yields
arithmetic gets its implementation before or beside the ratification.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

329. **Great Lakes 8187 is `astar_path@00683770:964`, a unit-grid
    search's failure tail** — open list exhausted, `Random::get % 3 + 6`
    into the order's move data `+0x1c` as a 6-8 frame retry delay, under
    two further gates (COMBAT §17.6 has both). **Not "emit one more
    draw"**: this crate's unit-grid search does not fail there, and on
    8187 it spends two `Guy::set_anim+0x97a < Guy::move+0x19f` instead —
    two of the probe's six stepping where the original does not.
    `path.rs`'s exhaustion tail names the seam; run19 covers it.

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
