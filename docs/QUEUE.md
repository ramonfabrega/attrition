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

*2026-08-28, after item 52.*

**The headline — run10, 1,772 frames: ticks before divergence 181, orders
180; player 0 first diverges at frame 182, player 1 at 203.** Orders
moved 168 → 180 and ticks 185 → 181, and that is one event, not two: the
residue is `0/3`'s gather tile, its pick moved 169 → 181, and at 169 the
wrong order simply did not move the unit for seventeen frames. A tick score
above the order score is slack, not progress.

**The ledger — run14: 192 of 284 frames match draw for draw**, up from 184.

**Landed:** item 52 — **the wing beat**. The bird's two animation lengths
are in the install, not in any dump: *Bird Soar* 31 frames, *Bird Flap* 23,
read from `art/*.bha` by the new `rondata::artdata` and cross-checked
against the dump on six sheep and fish lengths (`docs/FORMATS.md`,
`docs/ANIM.md` §3.1, `docs/SYNC.md` §3.9). Three side-findings that are not
about birds: `set_anim`'s walk arm re-resolves from the *category*;
`Guy::init_real` leaves `end_time` at zero, so a guy created outdoors wraps
on its own birth frame; and `spawn_bird` never set `type_index`, which made
every bird rule inert.

**Owed:** unchanged — items 42 and 40 (`ORDERS.md` 191,190, `CITIES.md`
106,854).

**Needs the user:** nothing.

**Opener (Opus):** `proceed @docs/QUEUE.md — raise the order score, now
180. Run the ledger test first: its failure message names the ninety-two
frames that still part, and the two biggest families are one table row
each (item 53). Take nothing that cannot name the score it moves.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

53. **The ledger's other ninety-two frames.** The failure message of
    `run14_s_frames_match_the_trace_draw_for_draw` lists them. Two
    families dominate and each is a `trace::SITES` row: `5d74e3` is
    `Animal::do_idle+0x83`, the animals' wander roll, which has no row at
    all, and a bare `5dac7a` is `Guy::set_anim+0x97a` under a caller the
    table does not carry. Naming them is not the whole of either — several
    of those frames differ in *count* as well — but it is what makes the
    residue readable. `PathFinder::calc_road_cost+0x46` (frames 10, 11,
    171; 226, 254 and 197 draws) is the caravan road, its own item. The
    `Farms::inc_time+0x1ae` / `+0x1de` orderings are a third family and
    the cheapest: same count, wrong order, on about a dozen frames.

38. **One long traced capture, human versus AI, on both maps.** run14's
    trace reaches 284 of run10's 1,772 frames, so nothing on disk measures
    the whole against the original by site. Gamelog at `UNITS=3` plus
    `rontrace`, ≥ 1,800 frames, the recipe in `docs/ORACLE.md`; then pin
    its headline beside run10's and retire the tests it supersedes. It
    would also lengthen the ledger sixfold.

35. **`mylos` as a cache.** `docs/VISION.md` §7 has the design: a
    `unit_stats_dirty` bit per player and a cached `mylos` refreshed where
    `Leader::calc_unit_stats` refreshes it. Takes run10's one LOS
    disagreement to zero; change the assertion to an empty vec first.

36. **`Unit::set_angle`'s seventeen other callers.** 6,866 of run10's
    16,206 angle rows, on frames where the positions agree. Start at
    `do_gather`'s: unit `0/2` at frames 432–433 is 2,680 of them and one
    screen of trace. Each caller is also where a group's mirror flag would
    move (`docs/GROUPS.md` §4.1).

37. **The arrival frame's facing.** Two rows in run10, the AI scout the frame
    after an `EXPLORE_TO` arrival. `docs/MOVEMENT.md`'s open questions name
    the suspect; `GUYS=2` prints `guy_flags` on every capture — a grep.

48. **The object chain, whole.** `crates/sim/src/collide.rs` chains units
    only; the original threads buildings and goodies through the same list
    (`docs/COLLISION.md` §3, §7). The units' order is unaffected, so the
    mechanic is right — but the dump prints `down`/`down_who` on every
    object at every detail level and the harness parses them and compares
    nothing. Put buildings in the chain and widen the diff to it; it is the
    cheapest untaken widening on the board.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture:* `UNITS=3` +
    `GROUPS=1` over a group ordered one way, turned right round while
    marching, re-ordered — in Refused or an Echelon, which also settles the
    `angles` byte's sign (item 19). `docs/GROUPS.md` §13. Fold into item 38.

49. **Collision's unrun arms.** `docs/COLLISION.md` §8 lists what no
    capture has executed: three of the four soft-collision arms, the
    wait-for-it branch (`unit_masks & 0x40`), the throttle above four
    repaths, the `pause = % 9 + 1` draw, and any `BLOCK_RADIUS ≠ 1` unit.
    §9 names the capture for each; most are one run of two units ordered
    head-on with `UNITS=3` and `rontrace`. Fold into item 38's capture.

39. **A debug viewer.** A thin, read-only 2D client over `Sim` state — map,
    units, fog, order lines, with the original's dump overlaid for the same
    frame. It needs no parity, would have shown the untyped trained unit and
    the seven-frame stand on sight, and is the renderer's first slice. One
    earned windowing dependency; take it when a residue is opaque in the
    tables.

40. **The spec/story split, one document per touch.** The documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. When a
    session touches one: keep the rules, fields, formulas and a **Coverage**
    section (diff-backed / reading-only / unmodelled); move the narrative to
    the journal; lower the pin. The Coverage section is also what a blind
    reader is briefed with.

41. **The second-map harness, and `scenario.py`'s fate.** `seedini.py` and the
    early window stay. `scenario.py`'s cheat generator has produced nothing and
    cannot issue orders; before deleting it, run the one measurement that could
    redeem it — `zsh tools/fuzz/run.sh 424242 1000 1300`, then
    `tools/trace/report.py … blind docs/` against both traces — and record
    whether any of its 188 extra functions is on the blind list.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: fifteen
    audits adjudicated on Opus, the groups fourth pass first (it overturns
    three earlier Fable rows), then `docs/ARMY.md` §18's marker. A steering
    session's job, over marked rows only; not an Opus item.

45. **Gaia's animals, the re-seat crutch, and the pasture's lengths.**
    `Sim::reseat_animal` puts the animals back from every traced dump
    because their wander rides a stream the harness only holds at those
    frames (`docs/SYNC.md` §4.2's tail). It is a correction, it is noted as
    one, and it should die: widen the diff to the `ANIMALDATA` record — 40
    of run10's 54 objects, uncompared to this day — as its own sub-score,
    then close what it shows. `rondata::artdata` deliberately leaves
    `FARMPIG` and `FARMCHICKEN` out of the install's length table — they
    name a `-TYPE0` and a `-TYPE1` and no `-TYPE2`, so a third of the
    pasture would be a guess — and until that is settled their idle rolls
    never wrap here (`docs/ANIM.md` §3.1).

51. **The AI's long-run economy, now that the roster is measured both
    ways.** `FrameResult::extra_units` is the mirror of `unlinked`, and
    together they say the AI reaches eight citizens on the original's
    frames and then stalls: the original trains `1/9` at 1297 and `1/10` at
    1505 and this simulation reaches neither inside 1,772 frames. Two
    threads, in order: the AI's citizens spend long stretches on `AttackTo`
    moves rather than gathering (`docs/ARMY.md`; the original's `1/6`–`1/8`
    sit on the camp and the fourth farm for the whole run), and the income
    itself — `CITY.gatherers` is in every dump at every detail level and is
    uncompared. Score it on the roster pin, 744 + 0.

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
