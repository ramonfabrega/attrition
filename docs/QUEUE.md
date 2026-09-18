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

*2026-09-17 into 09-18, an Opus commander loop: twenty-two landings, sixteen
word-moving. **GL +949, EI +1,899.** Next: 352.*

- **Nine frames of word, and the state behind it collapsed** (item 350).
  `Groups::push_group` kills each member out of the group it was in, so
  8186's probe took **six units out of the AI army for good**. GL 8619 →
  8628 — but run97's walk-slot residue went **6,258 → 1,747** (33.8 → 9.4 a
  frame) and its point-and-goal residue 11,159 → 7,009 of 121,224, with the
  endpoint off 63 → 58.
- **So read the word beside the residues, not alone.** A +9 item that
  removes 4,511 wrong fields is worth more than its headline, and the
  headline cannot show it. This is the clearest case the loop has produced
  of the score under-reporting a landing.
- **Parking 347 with its rate is what made that legible.** It was parked as
  "names no score, no draw cost" with a falsifier and a per-frame rate; 350
  fixed most of it as a side effect and the rate is how we can tell. A
  parked residue is not a discarded one.
- **East Indies is still owed a capture above 8789** — 922+ short since 346,
  its whole window below the word. Great Lakes needs none: run97 runs to
  9349. Awaiting the go; it is the lane's, not a worker's.
- Three items open, thirty-two parked. **Fable backlog: eleven Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w8628 of 24,000
Endpoint 24001: EastIndies 61 off, 11 unlinked · GreatLakes 58 off, 7 unlinked

**Opener: 352 — the muster destination at 8442, named with coordinates and
below the word. 354 widens 8628 if it does not take.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

352. **The AI musters where neither city says** — 350's successor, named
    on the frame in `docs/ARMY.md` §18. At Great Lakes **8442** both sides
    spend the same two `find_target+0x7df` score draws; this crate takes
    London (347 against Norwich's 328) and musters at cell **(50,27)**,
    where the original's `GROUPATTACKTOORDER` carries orig
    **(44851,22480) = cell (58,29)** — **neither city's muster**. So the
    score is not the disagreement and the destination is. Below the word,
    and a takes-chain: an army in the wrong place keeps paying.

354. **Widen Great Lakes 8628** — the headline's own frame, which no item
    names. Take this only if 352 does not move the word; run97 `[8030,
    9349]` dumps 8628 with 721 frames above.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned twice. **Re-measure before diagnosing**: the vector
    `1/2020 − 1/2019` is the claim, never the count.

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
