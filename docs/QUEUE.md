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

*2026-09-17, an Opus commander loop. **East Indies +381, Great Lakes
+352.** Next: 320, already dated.*

- **Great Lakes 8031**, **East Indies 8193**. Six landings: +381 (304's
  suspend), +251 (314's `get_loc` seam), +100 (317's `do_marching` arm),
  +1 (319's `scout_danger`). 312 and 308 named causes and reach.
- **The last move was one frame, and that is the signal.** 319 fixed a
  whole subsystem answering zero and bought one frame, because 8002's
  pathfinder row was already under it. Expect stacked causes here.
- **Six items, five named mechanisms wrong**, each overturned by the
  widening inside twenty minutes: 304's formation was never early, 312's
  draw site is a *blocked* step, 314's waypoint is the `get_loc` seam
  (only the listing settled it), 317's was a status re-read, 319's draw
  was the scout's, not the army's. Brief the widening as the item, and
  tell the worker to assume its named mechanism is wrong.
- **A comment is not evidence.** `scout_danger` returned 0 behind one
  claiming the grid was keyed differently; ARMY §18 filed `role` as
  carried and nothing does; GROUPS §6.3 called `MoveOrder +0x4` an origin
  where the type record says `x`. Grep the writers.
- **A capture is four minutes and needs no human** (`viadriver.sh
  runqueue.sh - <item>`); grep the disk first — twice today it answered a
  question it was not taken for. One item open, twenty parked.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8193 of 24,000 · GreatLakes w8031 of 24,000
Endpoint 24001: EastIndies 66 off, 16 unlinked · GreatLakes 59 off, 23 unlinked

**Opener: 320 — the scout's route at 8002, dated and asserted, run94 on
disk. Its pins are the commander's; a red `great_lakes_endpoint_is_pinned`
in a worker's gate is expected, not its bug.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

320. **The scout's route parts at 8002, and it is a pathfinder item**
    — dated and asserted by 319, not a hypothesis. `1/0` carries no row
    below block 8002; there its order parts on `find_wpath`'s route (the
    original swings west of the human city footprint to (2040, 31224),
    this crate keeps to column 2808 and runs **through** it — five stack
    entries against six), and its position parts at 8014. run94 is on
    disk and covers it. PATHFINDER, not SCOUT: the danger grid is now
    diff-backed and is not the cause.

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
