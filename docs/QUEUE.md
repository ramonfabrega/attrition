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

*2026-09-21, seven landings on three lanes. **Both headlines moved** —
the AI word 9510 → 10161 → 10232 and the golden record 616 → 624 — after
sixteen landings that moved neither. Every one widened before it read.*

- **The AI headline moved twice, 9510 → 10232** (442 +651, 456 +71): the
  scholar arm's third `if` read as an `else if` (listing, 006c52ec), then
  the group-mate soft arm reading `UnitData +0x104` as zero — **the field
  exists** (`Unit::search`, PATHFINDER §18), and a seam comment saying
  otherwise outlived it a fortnight, widening the arm by every unit busy
  re-planning.
- **The rules headline moved, 616 → 624** (447, after 443 closed the
  radius): `valid_target`'s fifth test is **fog on the target**
  (`is_seen`), which this crate did not have — and `add_unit` skipped
  `add_to_world`'s `update_seen`, so a unit born on the map lit nothing
  until it crossed a half-cell. Great Lakes' endpoint 55 → 53 with it;
  East Indies and every other score unmoved.
- **The met bit's bracket is a block, and it is ours** (390, run115):
  the original flips on **7945**, on both leaders at once.
- **Both words are widened and pinned** (448, 447, re-pinned by 456 with
  the move): East Indies' 9711 and chapter one's 626 still owe theirs
  (444, 445, parked). Two open, 76 parked.
  **Fable backlog: ten Loop items** (251, 335, 313, 375, 428, 446, 449, 452, 453, 461).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10232 of 24,000
Golden: w624 of 901 (ch2) · ch1 w626 · 462 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 48 off, 13 unlinked

**Opener: 463 on the AI lane and 462 on the rules — both on a moved
word's own frame, a named candidate each, and no capture owed.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

463. **10233's formation pathing, 53 rows on four units** (the AI
    headline's frame; 456 moved the word to 10232 and read its far side):
    `0/5 orders.len` ours 1 theirs 0 — the human's, an order the original
    drops — and the squad `1/27`–`1/29` parting on positions, headings,
    tolerances and path stacks together, **ours a 44-entry plan against
    the original's one hop**. Draw stream 99 against 95, parting at draw
    92. Not collision: 456's sweep is pinned line-for-line against
    run116. No capture booked; grep the disk first.

462. **622's three destinations, ours one seat and theirs three points**
    (the rules headline's frame; 447 widened [620, 628), 457 left the word
    at 624): we plan all three slingers to `(2424, 7800)` — `1/6`'s own
    seat — against `(1608, 8184)`, `(1560, 7848)`, `(1704, 8424)`, ten
    path slots to six, and 624's extra draw is that destination. **A named
    candidate**: parked 460, `Profile::x_size` 0 where the original's is
    `CIRCLE_RADIUS`, so `find_attack_pos` reads a unit target's extent as
    `(0, 0)` here and `(0x60, 0x60)` there. Falsifier written.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times, and now `1/55`. **Re-measure before
    diagnosing**: the vector `(768, 9984)` is the claim, never the count.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap — is `CLAUDE.md`'s. **A number is
  measured on the tip**, after the last `ccc update`, or names its tree.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:`, `Endpoint <frame>:`, `Fable backlog: N Loop items` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is either zero-valued or switched off (117, 178). **A
  never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
