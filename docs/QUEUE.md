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

*2026-09-21, the ninth chain, two lanes. **Both headlines moved** — Great
Lakes 10232 → 10233 and chapter two 624 → 637 — and on both lanes the
mechanism the previous item named was wrong.*

- **The AI word is 10233** (463): a dead target outlives its order by the
  attacker's reload — `Unit::do_attack` never tests aliveness for a type
  with `attack`, and `Unit::fight`'s recharging arm returns before its
  `valid_target`, so six AI raiders keep a dead building's ATTACKORDER
  where this crate dropped all six at once. Three assertions hold it, not
  the word. **Not formation pathing**: 456's hypothesis, DECISIONS 42.
- **The rules word is 637** (462), +13, [620, 628) closed whole. Parked
  460 was **falsified by its own falsifier**: nothing reads
  `Profile::x_size`. The causes: `Built::unit_ids` resolved only through
  the start dump's link table, so the dumped ATTACKORDER target went
  uncompared for the whole chapter; then `find_nearby_target` walking the
  unit index where it walks the cell's own `down` chain, and the unit
  half of `find_attack_pos`.
- **Neither lane's endpoint counts survived the merge**: 463 alone read
  51 off, 462 alone 48, the merged tree 45. Re-measured there, never
  hand-merged; 467 parks the amend hazard. Four open, 70 parked.
  **Fable backlog: 4 Loop items** (313, 467, 468, 469).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10233 of 24,000
Golden: w626 of 901 (ch1) · ch2 w637 · 466 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 45 off, 9 unlinked

**Opener: the ninth steering pass is due — not on the count (2 of 20)
but because the loop broke: the chain has no link past reap (468) and
lanes were run as a pair (469). Then 464 and 466, each on its own lane.**

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

466. **635's six `Target` rows in the new window** (the rules headline's
    frame; 462 moved the word to 637 and widened [633, 641)): the bowmen
    take `1/6` where the dump takes `1/8`, and all three hoplites take
    `0/7` where the dump takes `0/11`. Same shape as 621's parting, and
    **the cell's `down` chain is explicitly not the cause here** — 462
    fixed that and these six survive it. Booked by the frame and the six
    rows, not by a mechanism. `docs/COMBAT.md` §32 is the specification.

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
