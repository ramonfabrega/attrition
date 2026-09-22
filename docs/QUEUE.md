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

*2026-09-22, the tenth Fable pass: no score moved and none was meant to.
Great Lakes **10277**, chapter two **683**, both inside their first fight.*

- **Fourteen landings since the ninth pass, every worker on Opus 5, the
  chain run unattended to its stop** — the ninth pass's clauses held.
  Both words moved. Workers 26 USD a landing against 20, 53 a word
  against 46, and **4 USD a frame against 0.19**: a fight draws on every
  frame, and a residue there buys a frame where one between fights
  bought 651.
- **Ten of fourteen landings turned on an instrument defect**, six of
  them a field the original prints that nothing read. That is a guard
  now: `rondata::diff::coverage` records what the parser asks each
  block for and pins the dump's own keys against it — **20 paths, 239
  keys unread** on the two headline windows, three record families
  never opened (`DEATH_OBJS`, the frame-level `GUY` list, per-frame
  `WORLD`). A key a landing reads is deleted from the pin in that
  landing. The count is the next pass's measure.
- **Struck text no longer counts** against a section's ceiling (480): a
  correction is not an addition. Four pins fell; `AI.md` §15 left the table.
- **683's two sides were parked by the worker who found them** while the
  rules slot stood empty; they are item **491**. 476 and 342 named no
  score and are parked. **Fable backlog: 1 Loop item** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10277 of 24,000
Golden: w626 of 901 (ch1) · ch2 w683 · 491 next
Endpoint 24001: EastIndies 63 off, 11 unlinked · GreatLakes 42 off, 7 unlinked

**Opener: nothing is in flight. A fresh Opus commander spawns 489 on the
AI track and 491 on the rules track, and runs the chain to twenty.**

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

491. **683's two draws: `1/8`'s death, and the arrow that lands on
    nothing** (the rules headline's own frame). Theirs is
    `Unit::close@0060ee50`'s death-animation `Random::get` — `DEATH_OBJS`'
    `cur_anim 17` says `dtype 2`, `roll % 2 == 0` — and this crate takes
    none; ours is §39's puncture from an arrow whose target died in
    flight, which the original **holds**: the dump's `1/7` takes `19+5`
    on 685. Item 485 named both halves and parked them as (491) and
    (492); `hold_frames` and `DEATH_OBJS` are pinned unread. COMBAT §41.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3. Stays because `1/28` is the word's
    own squad: a takes-chain candidate to 10278's collision, written as
    one, not a cause. A second lane's item, beside 489 and never instead.

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
