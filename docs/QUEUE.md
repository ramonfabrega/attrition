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

*2026-08-27, the first Fable steering session, after item 34.*

**The headline — run10, 1,772 frames, RNG seeded from run11's trace: ticks
before divergence 3, orders 2; player 0 first diverges at frame 103, player
1 at frame 4.** Pinned as a floor in `run10_s_opening_…` (`rondata::diff`)
with a history line; it has not moved since the capture was taken. Every
sub-score has — frame 0–2 draws exact on run20, `mylos` 26,433 rows and one
disagreement, path stacks 0 on run20, 13,542 angles compared. The whole has
not, and from now on the handoff says whether it did.

**Landed this session:** the headline pin; this file's rewrite and its
guard; the `sin_table` marker closed from the listing — the mirror is the
original's own, inlined at 63 of 65 call sites, and the odd branch is live
only in `MapGrass::make_continents`; `CLAUDE.md` amended (the finish line,
the stopping rule, Fable's lane, spec versus story); `DECISIONS.md` 24.

**Owed:** the ratification ledger (`docs/audit/README.md`), taken by
steering sessions over marked rows only; the spec/story split of the eight
documents over 60 KB, one per touch under the guard's ratchet; `scenario.py`
is parked and `seedini.py` stays as the second-map harness (item 41).

**Needs the user:** nothing.

**Opener (Opus):** `proceed @docs/QUEUE.md — raise the headline. Player 1
first diverges at frame 4 because unit 1/2 holds a second order at frame 3
that the sim does not (item 25, Length { ours: 1, theirs: 2 }). Find what
issues it, land it, raise the floor in run10_s_opening_… and add its history
line. Then the next-earliest first divergence. Take nothing that cannot
name the score it moves.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

25. **The first frames, on a map we did not tune against.** Two rows left:
    the frame-1 order two of player 1's units hold and the sim does not —
    **this is the headline's frame 4** — and §9.3's sixth citizen, which
    `check_start_orders` derives on run20 and not on the fuzzed map
    (`gamelog-fuzz-424242-early.txt`, sibling `-heights`). Also the fuzzed
    map's frame 1, one `Leader::produce_building+0xc99` and one
    `Unit::do_non_flat_gather+0x54b` short, asserted as they stand in
    `the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well`.
    `docs/AI.md` §9.3, `docs/SYNC.md` §4.2.

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
