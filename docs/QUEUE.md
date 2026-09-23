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

*2026-09-23, twenty landings under an Opus 5.5 commander, workers on
Opus 5.5 throughout; stopped at twenty. Great Lakes **10834 → 12038**;
golden chapters one, two and five **closed at 900**, chapter four (the
namesake) **pinned at 1277**. East Indies did not move from 9711.*

- **The namesake met its oracle and was half unwired** (552): the
  building arm, the temple level, `set_epoch`'s tail, `calc_attrition`
  (never called, so item 382's zero-outcome survey was **vacuous**) and
  `curr_uber_size`. Fixed, border cell for cell, bleed tick for tick to
  1337. **Every Phase 1 claim resting on a reading deserves this test.**
- **The frame was right and the briefed mechanism mostly wrong again**
  (518, 523, 530, 539, 566); **three register pairs the decompiler
  dropped** (495, 523 twice) — the trap is in `tools/ghidra/README.md`.
- **Steering questions**: East Indies (9711) sits 2,300 frames under
  Great Lakes and took no item — is "lower map first" the frame or the
  map? And the `Golden:` guard reads a closed chapter's 900 as the
  lowest word (528).
- **A timed-out capture writes no receipt** (565, 1.5 h lost).
- **Fable backlog: 11 Loop items** (313, 503, 507, 508, 509, 513, 517, 526, 527, 528, 565).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w12038 of 24,000
Golden: w900 of 901 (ch1) · ch2 w900 · ch5 w900 · ch4 w1277 of 1501 · 567 next
Endpoint 24001: EastIndies 63 off, 8 unlinked · GreatLakes 42 off, 4 unlinked

**Opener: nothing in flight, every lane reaped, tree pushed. The steering
pass is due (twenty landings). After it, a fresh Opus 5.5 commander takes
571 on the AI track and 567 on the rules track; workers spawn with
`claude-opus-5-5[1m]`.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

571. **Army 1's squad stops on block 11922 in the original and walks on
    here** (566 moved the word 11903 → 12038 by carrying the pathfinder's
    validity memo across searches, PATHFINDER §24). Nine figures,
    `1/37`–`1/42` and `1/62`–`1/64`: `stopped 1`, speeds 0 against
    walking; positions agree, no draw — 29 rows pinned in run136's
    widening. Then 12038's capture and widening past run136's end
    (11959): ours 4 draws against 5, `move_step+0x823`. No mechanism.

567. **Golden chapter four's word is 1277: the squad's `GUARDORDER`
    beside its wagon** (552 captured run132/run133 and pinned the chapter;
    none of §8's falsifiers fired, five wiring defects fixed). Ours
    spends `Guy::set_anim+0x97a < Guy::move+0x19f` where the original
    spends the farm's, 5 draws against 4. On 1277 the original's squad
    holds a `GUARDORDER` and a move back to (8376, 32136), `timer` 419;
    ours keeps its `AttackTo`. The rules headline. No mechanism.

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
  means the next unpinned chapter; **a worker parks what names the
  headline's frame and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s. **A
  number is measured on the tip**, after the last `ccc update`.
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
  free when every landed branch is merged, gated, pushed and reaped, and
  nothing is in flight. **Before a blind fan-out**, grep `CLAUDE.md` and
  the memory index: a subagent inherits both.
