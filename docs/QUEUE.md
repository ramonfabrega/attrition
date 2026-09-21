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

*2026-09-21, five landings on three lanes. **Both headlines moved** —
the AI word 9510 → 10161 and the golden record 616 → 624 — after sixteen
landings that moved neither. Every one of the five widened before it read.*

- **The AI headline moved, 9510 → 10161** (442, +651): 438's free-slot
  correction plus an `else if` that should be a third independent `if`
  (listing, 006c52ec), and the scholar arm now returns **5,755,741** and
  **4,891,136** — both of the original's offers to the unit. The 2:1
  ratio was the inputs; 1.1768 was the wrap.
- **The rules headline moved, 616 → 624** (447, after 443 closed the
  radius): `valid_target`'s fifth test is **fog on the target**
  (`is_seen`), which this crate did not have — and `add_unit` skipped
  `add_to_world`'s `update_seen`, so a unit born on the map lit nothing
  until it crossed a half-cell. Great Lakes' endpoint 55 → 53 with it;
  East Indies and every other score unmoved.
- **The met bit's bracket is a block, and it is ours** (390, run115):
  the original flips on **7945**, on both leaders at once, `diplos`
  static across 130 blocks.
- **Both words are widened and pinned** (442/448, 447): `WIDENINGS`
  names a test for each. East Indies' 9711 and chapter one's 626 still
  owe theirs (444, 445, parked). Two open, 72 parked.
  **Fable backlog: nine Loop items** (251, 335, 313, 375, 428, 446, 449, 452, 453).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10161 of 24,000
Golden: w624 of 901 (ch2) · ch1 w626 · 457 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 53 off, 8 unlinked

**Opener: 456 on the AI lane and 457 on the rules — both on a moved
word's own widened frame, and run116 is reserved for 456.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

456. **Which arm of the collision sweep refused `1/38`'s step on block
    10162** (the AI headline's frame; 448 widened it whole): one unit of
    104 parts, on an **agreed** waypoint — `collide_guy` ours −1 theirs 0,
    `half_step` 1 against 0, the original stopping dead and re-converging
    by 10165. Both sweeps ran and disagreed on one point; which arm
    decided it no dump can say, because the snap arm clears
    `collide_o`/`collide_who` two instructions on. **run116** reserved,
    the ask and its falsifier in `docs/AI.md` §54.4.

457. **`ObjectData::visible` (+0x40), the other half of `is_seen`** (the
    rules headline's frame; 447 moved the word to 624 and named this at
    `docs/COMBAT.md` §31.7): the original's `1/6` accepts `0/10` at 635
    because the slinger set its own bit at 631 **by attacking**
    (`set_attacking@005ff5b0`, `visible |= 1 << victim_who`); this crate
    has no such field. Its writers, its clear-on-`work` rule, and a gate
    on the attacker's type vtable **+0x10c** that `vtables.txt` names for
    no `*Type` vtable — settle that from the listing. run113 unspent.

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
