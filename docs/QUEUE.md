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

*2026-09-21, the tenth chain. **Neither headline moved**; both items
closed rows short of the word.*

- **The AI word is 10233** (464): `0/5`'s 22 rows → 3, its FLEE_TO on
  the dump's `(792,31800)`, draws 6/4 → 5/4. `Unit::think`'s step-3
  gate wants `role & 0x10000` beside the attack column, so idle
  citizens ran `find_melee_target`; plus the flee arm §12.4 described
  and nothing built. COMBAT §34.
- **The run100 widening read health against the ceiling** — `myhits`
  is the maximum, so every wound printed as a divergence; both are rows
  now, 444 → 655 keys. Endpoint 45/9 → **48/10** past the word (DEC 36).
- **The rules word is 637** (466): §33.1's decaying `targeted` penalty
  closes three of 635's six, parting still 635. §33.2 — the `ai` arm
  **dividing** where a human's multiplies — is **established, not
  landed**: it reds the `visible` *shape* row, because `1/8`'s strike
  rests on a wrong two-slot assignment. **§33.4: that green is
  unearned.** 470 carries both.
- **A widening named "whole" walked ten of fourteen vectors**:
  `gather_`, `build_`, `queue_` and `city_diverged` went unnoted. All
  four walk now, map unchanged — which is how we know they were empty,
  not ignored. Chapter two still has **no health row** (§33.6). Three
  open, 68 parked. **Fable backlog: 1 Loop items** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10233 of 24,000
Golden: w626 of 901 (ch1) · ch2 w637 · 470 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 48 off, 10 unlinked

**Opener: 465 and 470 are both live, one lane each. The loop stands at
5 of 20, and no headline moved on either of the last two items.**

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

470. **`find_ordered_collision`'s reach: three chasers, one slot**, and
    it carries 466's withdrawn `ai` hunk with it (the rules headline's
    frame). `find_open_slots@00600e30` walks the object chain of the
    cells around the **slot**; chapter two's hoplites stand 1,200 east
    of the ring, so none sees the others' orders. Landing both closes
    635's last three `Target` rows and moves the parting 635 → 636.
    Neither half lands alone — COMBAT §33.2, §33.4. **Read
    `find_ordered_collision`'s callers before assuming a second chain.**

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
