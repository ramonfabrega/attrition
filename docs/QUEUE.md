# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story.

This file **deletes**. A finished item leaves it for the journal, whose
"Lifted from the queue" section and dated entries carry every number that
has ever been here — items keep their numbers for that reason, and new ones
continue the count. `crates/sim/src/docs_guard.rs` fails the build if this
file strikes an entry instead of deleting it, passes 180 lines, or lets the
handoff pass 32.

## Where things stand

*2026-08-27, after item 25.*

**The headline — run10, 1,772 frames, RNG seeded from run11's trace: ticks
before divergence 99, orders 102; player 0 first diverges at frame 103,
player 1 at frame 100.** It moved for the first time since it was pinned
(from 3 / 2 / 103 / 4), and the pin in `run10_s_opening_…` carries its
second history line. What is left at 100 and 103 is no longer any one
unit: player 1's trained citizen `1/6` on 100, and player 0's three farmers
on 103.

**Landed:** item 25 — the tile choice scores only tiles that still carry
`mask & 0x4000` *and* pass `has_gather_access` (`005f0575`); player 1's
woodcutter had been walking to the tree in the middle of its own forest.
Two things fell out of it: `borrow_from_siblings` now takes the whole
`WORLD` block from a matching sibling — run6's is `BUILDS=7`'s and the
harness had been diffing it against a flat treeless world of its own
making, so **run6 and run10 are the same game and now report the same
score** — and `compare_orders` diffs the `GATHERORDER`'s whole row.

**Owed:** unchanged — items 42 and 40 (`ORDERS.md` lowered to 191,335),
`scenario.py` parked (item 41).

**Needs the user:** nothing.

**Opener (Opus):** `proceed @docs/QUEUE.md — raise the headline. Player 1
now first diverges at frame 100: unit 1/6, the AI's first trained citizen,
which the original sends somewhere this simulation does not. Player 0 is at
103, three farmers at once. Take 1/6 first, find what orders it, land it,
raise the floor in run10_s_opening_… and add its history line. Take nothing
that cannot name the score it moves.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

38. **One long traced capture, human versus AI, on both maps.** run10 is the
    longest capture and predates the trace, so its RNG is seeded from a
    sibling and nothing on disk can measure the whole against the original
    for more than 300 traced frames. Gamelog at `UNITS=3` plus `rontrace`,
    ≥ 1,800 frames, the recipe in `docs/ORACLE.md`; then pin its headline
    beside run10's and retire tests on captures it supersedes. This is the
    measurement every later item is scored on.

35. **`mylos` as a cache.** `docs/VISION.md` §7 has the design: a
    `unit_stats_dirty` bit per player and a cached `mylos` refreshed where
    `Leader::calc_unit_stats` refreshes it. Takes run10's one LOS
    disagreement to zero; change the assertion to an empty vec first.

36. **`Unit::set_angle`'s seventeen other callers.** 5,435 of run10's
    13,542 angle rows, on frames where the positions agree. Start at
    `do_gather`'s: unit `0/2` at frames 432–433 is 2,680 of them and one
    screen of trace. Each caller is also where a group's mirror flag would
    move (`docs/GROUPS.md` §4.1).

37. **The arrival frame's facing.** Two rows in run10, the AI scout the
    frame after an `EXPLORE_TO` arrival. `docs/MOVEMENT.md` open questions
    names the suspect; `GUYS=2` prints `guy_flags` on every capture — a grep.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture:* `UNITS=3` +
    `GROUPS=1` over a group ordered one way, turned right round while
    marching, re-ordered — in Refused or an Echelon, which also settles the
    `angles` byte's sign (item 19). `docs/GROUPS.md` §13. Fold into item 38's
    capture if the shapes can share a run.

39. **A debug viewer.** A thin, read-only 2D client over `Sim` state — map,
    units, fog, order lines, with the original's dump overlaid for the same
    frame. It needs no parity, would have shown the untyped trained unit and
    the seven-frame stand on sight, and is the renderer's first slice. One
    earned windowing dependency. Take it when a residue is opaque in the
    tables, not before.

40. **The spec/story split, one document per touch.** The eight documents
    over 60 KB are pinned in `docs_guard::OVER` and may only shrink. When a
    session touches one: keep the rules, fields, formulas and a **Coverage**
    section (diff-backed / reading-only / unmodelled); move the narrative to
    the journal; lower the pin. The Coverage section is also what a blind
    reader is briefed with.

41. **The second-map harness, and `scenario.py`'s fate.** `seedini.py` and
    the early window stay. `scenario.py`'s cheat generator has produced
    nothing and cannot issue orders; before deleting it, run the one
    measurement that could redeem it — `zsh tools/fuzz/run.sh 424242 1000
    1300`, then `tools/trace/report.py … blind docs/` against both traces —
    and record whether any of its 188 extra functions is on the blind list.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: fifteen
    audits adjudicated on Opus, the groups fourth pass first (it overturns
    three earlier Fable rows), then `docs/ARMY.md` §18's marker. A steering
    session's job, over marked rows only; not an Opus item.

Older backlog, one line each, unchanged: the `LEADERDATA` and `CITY`
widenings; a `find_target` block; run7's order stream under the trace; a
mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
`calc_gather` for non-flat buildings.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal under their number. Run `cargo test -p sim docs_guard`.
- **Start of session:** read "Where things stand", then the item you are
  taking, then that mechanic's document.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and read the
  memory index — a subagent inherits both.
- **Never** quote this file or the journal into `CLAUDE.md`, a subagent
  brief, a `.claude/agents/` definition, or a memory hook.
