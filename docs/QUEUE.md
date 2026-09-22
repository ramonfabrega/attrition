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

*2026-09-22, the chain closed at twenty-one landings: Great Lakes
**10233 → 10277**, chapter two **637 → 683**. **The steering pass is due.***

- **Nobody had ever read an `AMMO` record** (485): run112 carries
  **373**, and chapter2.cmd's own command line asks for `AMMO=5`.
  They date and place every arrow. Tenth instrument defect, and a
  whole record type rather than a field. `recharging` and the
  overkill window were compared nowhere either; `recharging` parts
  **nowhere** over 81 blocks × 21 units, killing the cadence and
  the `% 16` phase in one run.
- **`combat::share` was written, documented, and had no caller.**
  `Sim::take_damage` passed the squad-sized `myhits`, so `1/8`
  reached `damage` 51 with `killed: false`. Widening 8 → **5**.
- **An impact frame that agrees is not an arrow that flew right**:
  launch *frames* were the original's arrow for arrow, launch
  *points* 80–125 units out — and 80 units crosses a frame at
  `proj_speed` 100, so two of nine impacts landed on the dump's
  own frame while the ladder was wrong.
- **Ten instrument defects, twelve named mechanisms, nine wrong.**
  490 is the first real capture question in twenty-one landings.
  Ten open, 74 parked. **Fable backlog: 3 Loop items** (313, 480, 488).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10277 of 24,000
Golden: w626 of 901 (ch1) · ch2 w683 · 484 next
Endpoint 24001: EastIndies 63 off, 11 unlinked · GreatLakes 42 off, 7 unlinked

**Opener: the steering pass is due and nothing is in flight — every
branch merged, gated, pushed and reaped, both headlines measured.
Bank, `/clear`, switch the main thread to Fable.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

489. **10278's seventeen rows, `1/40` colliding with `1/41`** (the
    AI headline's own frame; 487 moved the word to 10277). Ours
    stopped on a 1-frame-old anim where the original is ten frames
    into a walk; `collide_o 41`, `collide_who 1`. The seventeenth row
    is `1/41 half_step` — **the original sets `unit_masks &
    0x100000` on both `1/40` and `1/41` and this crate on neither**.
    487 wrote that as a row, not a mechanism, and the item is booked
    on the frame and the draw delta. `docs/ORDERS.md` §20.

476. **f10234's three value rows**: `0/5 order:length` 2/1,
    `0/5 orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. The
    word's own frame, value side. ORDERS §18.3.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3.

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
