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

*2026-09-17, an Opus commander loop, two waves of two.
**Both maps moved: East Indies +381, Great Lakes +251.**
Next: 317, the new Great Lakes frame.*

- **Great Lakes 7930**, **East Indies 8193**. 304 wired 301's suspend
  (+381); 314's capture closed the `get_loc` seam (+251); 312 named 7679's
  cause and 308 took the suspend's reach from two windows to 23 captures.
- **Four items, four named mechanisms, three of them wrong** — and the
  widening found each inside twenty minutes. 304's formation was never
  early; 312's draw site is a *blocked* step, not the walk; 314's extra
  waypoint is the `get_loc` seam, not `find_wpath_from`'s emission, which
  only the listing could settle. Brief the widening as the item, not as a
  preliminary.
- **`MoveOrder +0x4`/`+0x8` is `x`/`y` by the type record**, not
  `orig_x`/`orig_y`; GROUPS §6.3 said "origin" and was wrong. A name is
  settled by the type record, never by the surrounding code.
- **The capture lane is four minutes and needs no human** (`viadriver.sh
  runqueue.sh - <item>`, run92): a capture is no longer a reason to stop.
- One item open, twenty parked. Journals: items 304, 308, 312, 314.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8193 of 24,000 · GreatLakes w7930 of 24,000
Endpoint 24001: EastIndies 66 off, 16 unlinked · GreatLakes 55 off, 28 unlinked

**Opener: `loop-317` is live on the headline and owns the screen — Great
Lakes 7930, eight draws against ten parting at index 0, `Army::find_target`
unspent here, and no dump under the frame. Widen first. When it lands, take
its pins (they are the commander's), then spawn the next off what it found;
the parked file has nothing that names a score, so a second lane needs a
takes-chain or it waits.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

317. **Great Lakes 7930 is eight draws against ten, parting at index 0**
    — the original spends `Army::find_target+0x410` once and `+0x7df`
    twice that this crate does not (item 314). **No dump sits under it**:
    the nearest Great Lakes window is run89's, which ends 154 frames
    short, so the brief is the widening first — a window over 7930 and
    its neighbours, every dumped record, every unit — and only then the
    mechanism. ARMY is the module; 314's run92 shows the capture lane is
    four minutes and click-free.

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
