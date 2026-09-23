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

*2026-09-22, nine landings under an Opus 5 commander, stopped at Ramon's
word for a Claude Code update. Great Lakes **10277 → 10834**, golden
chapter two **683 → 725** — both headline words moved.*

- **Two of the nine moved no word on purpose and were the more
  valuable**: 496 landed the comparator, 510 the `GUY` block chapter
  two's widening had **never opened** — eleven items blind to the
  record its word is spent in.
- **Five instrument defects of one family, the tranche's real
  finding**: a comparator reading an empty side as agreement
  (`compare_orders`, third time — 462, 496, 502); coverage keys nothing
  parses; run118's truncation readable as emptiness; `ledger::DIFF`
  missing `diff/golden.rs`, so the **whole rules track** read as
  uncompared. All five **report health they have not measured**, and in
  each the instrument's *scope* is hand-maintained and unchecked.
- **The briefed mechanism was wrong eight times of eight**; the frame
  was right every time. COMBAT §44.2.1 is the rule it produced.
- **495 promoted, 477 closed**, both by measured takes-chains rather
  than argument. 495 is the first hard-constraint item.
- **Disk at 99%, 15 GiB free**; `ccc spawn` refuses under 10. **Stops
  the loop, not slows it.** Coverage pin 239 → 232 keys.
- **Fable backlog: 10 Loop items** (313, 503, 507, 508, 509, 513, 517, 526, 527, 528).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w11903 of 24,000
Golden: w900 of 901 (ch5) · ch1 w900 · ch2 w900 · 552 next
Endpoint 24001: EastIndies 63 off, 8 unlinked · GreatLakes 58 off, 0 unlinked

**Opener: an Opus 5.5 commander is live; 563 in flight on att-563 (AI), 552 on
att-552 (rules). Workers spawn with `claude-opus-5-5[1m]`.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

563. **Great Lakes' word 11903 has no widening: run135 ends short**
    (560 moved the word 11806 → 11903: the soft one-shot is set only
    when the sweep ends without a hard hit, COLLISION §12). Ours 5 draws
    against 4 at index 1: `1/64`'s `Unit::move_step+0x823` stop against
    `Guy::inc_time+0x271`. The capture first, run135's line over about
    `[11860, 11960)` overlapping run135, then the widening; re-point
    `WIDENINGS`' row. No mechanism.

552. **Golden chapter four, the border and the bleed — the next unpinned
    chapter** (549 closed chapter five at 900; ch1 and ch2 are closed;
    GOLDEN §14 runs four next). The namesake: `chapter4.cmd`'s Temple,
    tech and civic border levers, then a hostile squad bleeding inside
    player 0's border and a Supply Wagon cancelling it. Two windows,
    run132 and run133; §8's five falsifiers are the first result. Then
    pin the word and its widening. No mechanism.

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
