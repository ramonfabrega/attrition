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

*2026-09-21, the tenth chain. **The AI word moved, 10233 → 10234**
(465); the rules word holds at 637.*

- **The AI word is 10234** (465): `1/28`'s fifteen rows on 10234 go to
  none — `pos` equal, `path:length` 43/1 → 1/1, `tolerance` 384/0 →
  0/0 — leaving 464's three, pinned as their own set so the closure
  cannot come undone behind the headline. Window 655 → 440 keys.
- **The defect was ours, not a misreading of theirs**: `action_move_near`
  carried an invented `g.army.is_some()` line the original's gate
  (`705f00-705f61`) has not, so six raiders pushed out of the army got
  plain `MOVE_TO`s and never entered `do_group_move`. It exempts a
  `go_to` group by `unit_masks & 4`; without it the word is 6994.
- **Three hypotheses, one frame, all wrong, frame right each time** —
  456's pathing, 463's order-death delay, 464's leader promotion.
  `oxx 40 → 28` is the whole of group 65. Read the cast (DEC 42).
- **The rules word is 637** (466): §33.1 closed three of 635's six;
  §33.2's `ai` divide arm is **established, not landed** — it reds the
  `visible` shape row because `1/8`'s strike rests on a wrong two-slot
  assignment (**§33.4: that green is unearned**). 470 carries both and
  is live. `ORDER_RESIDUE_RUN97` 53,622 → 81,534, all of it fields
  never compared before. Three open, 68 parked. **Fable backlog: 1
  Loop items** (313).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10234 of 24,000
Golden: w626 of 901 (ch1) · ch2 w637 · 470 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 49 off, 9 unlinked

**Opener: 470 is live; 471 takes the freed lane. The loop stands at 6
of 20 and the AI word moved on the last item.**

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
