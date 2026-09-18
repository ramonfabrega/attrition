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

*2026-09-17 into 09-18, an Opus commander loop: twenty landings, fourteen
word-moving. **GL +903, EI +1,899.** Next: 348, and a capture lane.*

- **The largest landing of the loop, and one change did both maps** (346):
  Great Lakes 8404 → **8582**, East Indies 8495 → **9711**. Both words were
  the seated scholar's first animation wrap. `Guy::set_anim` remaps a
  scholar's every slot from `0x19` up to `CHAR_DEFAULT`, so `inc_time`'s
  looping restart of a teach slot arrives as an **idle request** — it draws,
  and it re-rolls the variant (`docs/ANIM.md` §5.1).
- **run97 has been read, and it paid**: 127,324 clock fields below the word,
  the first test ever to open it. The capture lane's product is spent and
  it bought +178 and +1,216 in one item.
- **East Indies has outrun its captures again** — 9711 against run98's last
  block 8789, **922 short**, and run98's whole window now sits below that
  map's word. Great Lakes still has **767 frames of runway** in run97
  `[8030, 9349]`. So the lane is owed one East Indies window above 8789,
  and the map jumped 1,216 frames in a single item, so make it wide.
- Two items open, thirty-one parked. **Fable backlog: eleven Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345). 318+339 are one
  question; 341+343 are one; 345 moves the commander's own verbs into git.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w8582 of 24,000
Endpoint 24001: EastIndies 61 off, 8 unlinked · GreatLakes 58 off, 11 unlinked

**Opener: 348 — widen Great Lakes 8582, which run97 still dumps. The East
Indies capture is the lane's, not a worker's.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

348. **Great Lakes 8582 is the headline and run97 still dumps it** — 346
    took both maps past their words and named nothing on this frame, so the
    brief is its widening: every record, every field, every unit, both
    sequences whole, then the cause. run97 `[8030, 9349]` covers it with
    767 frames of runway; run94 is the neighbour below. 346 read that
    capture frame by frame for the first time, so the instrument exists.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue. 346 re-pinned its test again: still four exact of eleven, but
    `1/51` out and `1/50` in, so **re-measure before diagnosing** — the
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
