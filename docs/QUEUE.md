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

*2026-09-17, an Opus commander loop: fourteen landings, nine word-moving,
~567 USD over 22 sessions. **EI +654, GL +522.** Next: 336.*

- **East Indies 8193 → 8466** (item 334): a half-built market replanning
  its roads, which `Build::process`'s `is_active` gate forbids — 200 extra
  `calc_road_cost` draws on one frame, the replan run above the gate
  instead of below it. So **Great Lakes 8201 is the headline again**, one
  item after it took the lead.
- **The cheapest worker of the day moved the most frames.** 334 cost
  **11.33 USD** and landed **+273**. Its brief carried the disk grep
  already done — this map's dumps stop at 7916, run54 is `MISC` alone — so
  the worker widened where it would have surveyed. Keep that brief shape.
- **Grep the disk before writing the brief, not only before a capture.**
  Four minutes, and it changed both maps' items: East Indies' word had no
  dumped record within 277 frames, and Great Lakes' **does** — run19 runs
  to **8201**, its last block, behind a filename that says 8192.
- **A dictated commit trailer records what the commander expected**, not
  what ran — Ramon's catch, parked as 335. The realised half is this
  session: the harness's attribution reminder said Fable on an Opus run.
- One item open, twenty-seven parked. **Fable backlog: seven Loop items in
  `docs/PARKED.md`** (313, 318, 321, 330, 331, 332, 335), invisible at boot
  until this line. 330/331 carry the price, 335 the trailer's sources.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8466 of 24,000 · GreatLakes w8201 of 24,000
Endpoint 24001: EastIndies 66 off, 13 unlinked · GreatLakes 69 off, 13 unlinked

**Opener: 336 — widen Great Lakes 8201 whole. run19's last block IS that
frame, so unlike 334 the record is on disk; the frame after it is not.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

336. **Great Lakes 8201 is the headline again, and run19 dumps it** —
    East Indies crossed back at 8466 (334). No item names 8201's frame, so
    by this file's own rule the brief is its widening: every record, every
    slot, every field, every unit on 8201 and the frames below it.
    **run19 runs 8174–8201** despite its filename — the word's own frame is
    its **last block**, so the frame *after* is the one thing no capture
    holds. Parked 333 (`1/28`'s path stack 41 against run19's 42, a
    `find_wpath` waypoint) is a candidate the widening tests, never assumes.

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
