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

*2026-09-17, the third Fable steering pass (Fable 5.1, the main thread).
**No score moved; none was meant to.** Next: an Opus commander wave.*

- **Great Lakes 7679**, **East Indies 7812**. The 09-17 wave: five landings,
  one word (+94, item 295), and 301 banked **+381 on East Indies unwired**
  (`worktree-loop-301-suspend`, `4521ccc`), which costs Great Lakes
  7679 → 6862 until 304 closes.
- **The measure**: 152 USD per word-moving landing against 102 (1.5x,
  under the doubling that stops); 1.61 USD per frame against 0.92 and
  0.13. The cause was the queue, not the workers: no item named the word's
  frame, and the wave ran residue rows. DECISIONS 34, third amendment.
- **Rulings landed here**: a finding parks by default and the headline slot
  is never empty; the journal is a directory (27 of 58 merges conflicted on
  `JOURNAL.md`); a worker edits neither journal nor queue; a number is
  measured on the tip; merge, gate, push, reap is one chain; loop items are
  the pass's, in `docs/PARKED.md`'s Loop section. Item 279 closed.
- **Two phantoms**: 304 and 224 were referenced as dependencies and never
  booked; the ledger now fails on that, and both are booked.
- Three items open, fifteen parked by the rule. `docs/audit/2026-09-17-fable-pass-3.md`.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7812 of 24,000 · GreatLakes w7679 of 24,000
Endpoint 24001: EastIndies 70 off, 10 unlinked · GreatLakes 57 off, 23 unlinked

**Opener: an Opus commander wave, two workers, Great Lakes both — 312 (the
word's own frame, a widening first) and 304 (the formation that ends early,
which unwires +381). When 304 closes, merge `4521ccc` and re-measure both
maps on the tip before attributing anything. 308 waits on 304.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

312. **Great Lakes 7679 is a figure's draw, and nothing names its cause** —
    `Guy::set_anim+0x97a` reached from `Unit::move_step+0x823` here and
    from `Guy::inc_time+0x271` in the original, index 1, four draws against
    three (AI §35.3, item 295's landing). Not the make list: 302 and 303
    closed rows there and moved nothing. **Widen every dumped record on
    7679 and its neighbours first** — name the unit and the mechanism, then
    fix. If the figure is in melee, 211's `unit_masks2 & 0x10` clock
    freeze (ANIM §5) is the first suspect; a value diff lands beside the word.

304. **run76's `1/28` ends its formation early** — a plain `ATTACK_TO`
    before 6858 where the original still holds `GROUP_ATTACK_TO` through
    6860, whose change to `ATTACK_TO` *during* 6860 frees the suspended
    search via `clear_partial_path`. Both sides run the same search
    (`start_dist` 7680 against 7669). Closing it wires the banked suspend:
    East Indies 7812 → 8193 at no Great Lakes cost (PATHFINDER §18.3).
    Grep run76's `GROUPS` record over 6850–6862 first; the formation's end
    is GROUPS §'s `move_step` zero (item 236) or an order-change arm.

308. **`UnitData::start_dist` is parsed, and twenty-three captures carry a
    non-zero value** — run16 alone has 8,752 rows, and `+0x130`'s only writer
    in the executable is astar's suspend block, so every row is a unit whose
    search the original suspended, on a capture already on disk. 301 proved
    it reads 144 for run90's `1/7` and 0 for every other unit there. It
    makes the suspend's reach assertable far past the two windows that
    found it, and every row is free. PATHFINDER §18. Takes 304.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names the headline's frame, a floor, or
  a takes-chain to one. **The headline slot is never empty**: no item on
  the word's frame means the next brief is the widening of that frame.
  Loop items — tooling, guards, these rules — go to the parked file's Loop
  section and are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes the
  `Scoreboard:` line; the worker's story is `docs/journal/<date>-item-<N>.md`.
- **A number is measured on the tip** — after the worker's last `ccc
  update`, or the report says which tree. 301's +381 was attributed to the
  wrong half against a stale base and would have landed wired.
- **Merge, gate, push, reap — one chain.** A reap left for "before the
  next spawn" was forgotten twice in one session.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together; the guard parses the line against the floors.
- **Overlap a capture's neighbours on purpose.** Six blocks each end is the
  floor (run83); a containing neighbour is free with `--exclude <the
  category you raised>`; `samegame.py` exits 0 when nothing is in common,
  so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE — not only for
  coverage of a frame.** A term zero in every dump is either zero-valued or
  switched off: 117 and 178 were the second.
- **A check on a never-cleared field asserts a CHANGE, not a value** —
  `collide_frame` is a permanent stamp (run85). Test both directions first.
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and this handoff is current with the headline
  measured. **Before a blind fan-out**, grep `CLAUDE.md` and the memory
  index: a subagent inherits both, and no brief quotes this file.
