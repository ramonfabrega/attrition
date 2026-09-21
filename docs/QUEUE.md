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

*2026-09-21, the tenth chain. **Neither headline moved** — Great Lakes
holds at 10233, chapter two at 637; both items closed rows short of it.*

- **The AI word is 10233** (464): `0/5`'s 22 rows → 3, its FLEE_TO on
  the dump's `(792,31800)`, draws 6/4 → 5/4. Two predicates:
  `Unit::think@005f6e40:150`'s step-3 gate wants `role & 0x10000`
  beside the attack column — a Citizen's attack is 40, so every idle
  citizen ran `find_melee_target` — and `target_opportunity`'s flee
  arm, which §12.4 described and nothing built. COMBAT §34.
- **The widening read health against the ceiling**: `myhits` is the
  maximum with `damage` beside it, so every wound printed as a
  divergence. `hits_left` and `myhits` are both rows now, +54k
  comparisons, all 909 blocks agreeing; [9340,10247] 444 → 655 keys.
  Endpoint 45/9 → **48/10**, re-pinned past the word (DECISIONS 36):
  every new row at 10234+, nothing below it moved.
- **The rules word is 637** (466, in flight): §33.1's decaying
  `targeted` penalty closes three of 635's six. §33.2 — the `ai` arm
  **dividing** where a human's multiplies — is **established, not
  landed**: it reds the `visible` *shape* row, because `1/8`'s strike
  rests on a wrong two-slot assignment. **470 carries both.** Three
  open, 71 parked. **Fable backlog: 1 Loop items** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10233 of 24,000
Golden: w626 of 901 (ch1) · ch2 w637 · 466 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 48 off, 10 unlinked

**Opener: 465 on the AI track, spawned with 464's reap; 470 on the rules
track the moment 466 merges. The loop stands at 3 of 20.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

465. **`1/28`'s remaining draw on 10234 at `Unit::do_move+0xe84`, one
    frame early** — the AI headline's frame, the last draw 464 left
    (5/4). Hypothesis is **464's, not this item's**: the original's
    `1/28` flips its `GROUP_MOVE`'s `oxx` 40 → 28 between 10233 and
    10234 — it becomes its group's leader on the frame it stands
    still, and `do_group_move` plans for the leader alone. **Not** the
    plan lag this item carried, nor 463's order-death delay; both are
    struck. COMBAT §34.4. Falsifier on disk: run100 10233-10235.

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
