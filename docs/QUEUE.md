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

*2026-09-21, the ninth chain. **The AI headline moved again**, 10232 →
10233 — and 456's named mechanism was wrong for the fourth time running,
which is now the rule rather than the surprise.*

- **The AI word is 10233** (463). A dead target outlives its order by the
  attacker's reload: `Unit::do_attack` never tests aliveness for a type
  that has `attack`, and `Unit::fight`'s recharging arm returns before its
  `valid_target`. Six AI raiders keep a dead building's ATTACKORDER; this
  crate dropped all six at once. Held by three assertions, not by the word
  — one made to fail both ways, one over the raid's whole tail, and
  `recharging` now a compared field of the widening's `UNITDATA` rows.
- **Not formation pathing.** 463's title was 456's hypothesis and the
  frame was the booking; DECISIONS 42 paid again.
- **The rules word is 624** (447/457), widened and pinned. Item 462 is in
  flight on 622's three destinations with parked 460's `x_size` as its
  first falsifier.
- East Indies' 9711 and chapter one's 626 still owe widenings (444, 445,
  parked). Four open, 68 parked. **Fable backlog: 1 Loop item** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10233 of 24,000
Golden: w624 of 901 (ch2) · ch1 w626 · 462 in flight
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 51 off, 8 unlinked

**Opener: 464 on the AI lane — `0/5` on block 10234, the human citizen
hit on 10233 that the original answers a block later with a FLEE_TO —
and 465 beside it, `1/28`'s one-frame plan lag, whose falsifier is
already on disk in run100 blocks 10233-10235.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

464. **`0/5` on block 10234, 23 rows and both the frame's spare draws**
    (the AI headline's frame; 463 moved the word to 10233): the human's
    citizen is hit on 10233 — `damage_frame 10233`, `damage_o 27
    damage_who 1` — and **ours answers same-frame with a MOVE_TO where the
    original answers a block later with a FLEE_TO**. `order:kind` 1/4,
    `orders_x/y` (2280,31032)/(792,31800), `idle` 0/2, `path:length` 1/0,
    guy clock whole. Draws 6/4, parting at index 1. `docs/COMBAT.md`.
    Not a mechanism yet: re-measure first.

465. **`1/28`'s one-frame plan lag on 10234, 14 rows** — 463's fix
    exposed it and it is a lag, not a destination: the original spends the
    frame after the order dies doing nothing (`dest 0`, `tolerance 0`,
    one-entry stack, unmoved) and plans on the next; ours plans and steps
    at once. `path:length` 43 against 1, `tolerance` 384 against 0.
    `1/27` does the same on 10240. `docs/ORDERS.md`; **its falsifier is
    already on disk** — run100 blocks 10233-10235, no capture to book.

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
