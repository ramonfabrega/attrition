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

*2026-09-21, the tenth chain. **The AI word moved 10233 → 10234**
(465) and **chapter two's unearned green was earned** (470).*

- **The rules word is 637** (470), parting **635 → 636**, 635's three
  `Target` rows to nought. `find_ordered_collision`'s group pass read
  `pushed_last` — the last slot `push_group` filled — where `65b4d4`
  takes the group off the asker's own `unit +0x80`; 465's own doc
  comment said so. Open count 10 → 9 → 8, `pushed_last` has no reader
  left and is gone. **465 is why this item closed.**
- **§33.4's unearned green is now earned, not re-pinned.** `1/8`
  strikes for the first time in 899 frames (`visible` 675 v the dump's
  665); the two-slots-of-twelve figure held exactly and §33.4 carries
  both counts. `exact` still 5. A disclosure, closed.
- **The widening floor sat above a live row**: `[633, 641)` hid
  `order 0/10`/`pos 0/10` at **630**, pre-existing, reported by
  nothing. Floor is run112's first block (606) and the map is pinned
  whole — so the earliest known divergence is now 630, revealed and
  not caused. That is 472. Fourth instrument defect today.
- **The AI word is 10234** (465): the invented `g.army.is_some()` gate
  in `action_move_near`; the original exempts a `go_to` group by
  `unit_masks & 4`, and without the bit the word is 6994. 471 is live
  on 10235. Three open, 68 parked. **Fable backlog: 1 Loop items** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10234 of 24,000
Golden: w626 of 901 (ch1) · ch2 w637 · 472 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 49 off, 9 unlinked

**Opener: 471 is live; 472 takes the freed lane. The loop stands at 7
of 20; chapter two's parting is 636 and its earliest row is now 630.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

471. **`1/28`'s `order:move.off_x` 120 against 648 on block 10235**
    (the AI headline's frame; 465 moved the word to 10234). 10234's
    whole dumped record now agrees and `1/28` reopens one block later,
    the two routes 240 apart in `x`. 465's reading: `Form::compute`'s
    slot table, **not** the pathfinder's draw count — the original
    spends 204 draws at `PathFinder::calc_road_cost+0x46` for a
    43-node route where this crate spends six for a route of its own.
    That is its hypothesis, not this item's. `docs/ORDERS.md` §16.

472. **`order 0/10` / `pos 0/10` on block 630** — the rules
    headline's earliest divergence, uncovered when 470 dropped the
    widening floor to run112's first block. Pre-existing and reported
    by nothing until now. This crate drops `0/10`'s move on the first
    frame it reads in range; the original drops it one frame later at
    the same `attack_dist` 1108 against the same reach 1158, both
    sides snapping to one quarter-tile. **The difference is state, not
    geometry** (COMBAT §35.2) — and that is a reading, not a mechanism.

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
