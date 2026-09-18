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

*2026-09-17 into 09-18, an Opus commander loop: twenty-six landings, twenty
word-moving. **GL +1,503, EI +1,899.** Next: 362, then reap the lane below.*

- **Great Lakes 9134 → 9182** (item 360) and the residues fell again over
  `[8029, 9182)`: point-and-goal **183 → 6**, walk-slot **103 → 0**, units
  ever off position 6 → 4. `Unit::move_step` splits on `param_2 < local_28`
  and has a collision block on **each** side; `docs/COLLISION.md` §5 had
  only the partial step's (`+0x823`), and 9134 was the **snap arm's**
  (`+0x4e2`), which resolves nothing — it consumes the waypoint the unit
  stands on, so `1/32` was resolved a frame early.
- **The trace's site list can hide a call chain.** That draw printed as a
  bare `5dac7a` because `SITES` carried eleven of the address's twelve
  `ebp` chains. A missing chain looks like an unnamed address, not an error.
- **IN FLIGHT — `capture-lane-2`**, ref `74a77ee7`, branch
  `worktree-capture-lane-2`: Great Lakes `[9340, 10900)` past run97's 9349,
  East Indies `[8780, 10400)` past run98's 8789. **When it reports: `ccc
  merge <ref> --no-ff`, gate, push, `ccc rm`.** It touches only
  `tools/gamelog/captures.txt` and its own journal. run97 has **167 frames**
  left above the word, so this is owed, not urgent.
- Two items open, thirty-two parked. **Fable backlog: twelve Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345, 356). **339** is
  one flag and has been worked around by brief eight times running.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w9182 of 24,000
Endpoint 24001: EastIndies 58 off, 14 unlinked · GreatLakes 57 off, 10 unlinked

**Opener: 362 — the market at 9182, where 358 left it: three
`use_market+0x1ed` against our one, and one `set_anim+0x104b` against three.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

362. **Great Lakes 9182 is the market's, and 358 left it there** — three
    `use_market+0x1ed` draws against this crate's one, plus an animation
    tail of one `set_anim+0x104b` against three. `docs/AI.md` §41 names the
    `LEADERS=9` window that reads it. Named on the frame, so **no widening
    owed**. run97 dumps 9182 with 167 frames above; `capture-lane-2` is
    extending that above 9349 if it lands.

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
