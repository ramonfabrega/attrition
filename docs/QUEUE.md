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

*2026-08-28, after the road's measurement.*

**The headline — run10, 1,772 frames: ticks before divergence 181, orders
180.** Unmoved. **The ledger — run14: 198 of 284 frames.** Unmoved.

**What moved is knowing what the headline *is*.** Player 0's first
divergence at 182 is farmer `0/3` re-picking its farm tile thirty-one
frames early, and the chain behind it is now measured end to end:
`Sim::plan_roads` is off, so run14's three road searches never spend their
220 + 248 + 189 draws, the stream is **466 draws behind by frame 94**, and
the farms' sprout coins land on the wrong words — the cell `0/3` sows was
sprouted here on frame 49 and is empty in the original, so it ripens on
180 against 211. **Item 55 is the headline item** (`docs/ROADS.md` §7).

**Landed:** the world widening — `run13_s_world_at_frame_95_is_the_
original_s_cell_for_cell`: 3,600 cell owners, 3,600 cell flags and 57,600
tile masks against run13's `DUMP_ALL` at frame 95, all agreeing but one
cell's `BUILDING` bit (item 56). Made to fail twice first.

**Not blocked.** A `screencapture` failure and a System Events timeout were
read mid-session as "no display"; they were **revoked screen and automation
permissions**, which a Claude Code update resets. Re-approved, both work. A
permission-shaped failure ends the turn with a question.

**Owed:** 42 and 40. **Needs the user:** a Fable pass — see below.

**Opener (Fable):** `ratify @docs/QUEUE.md — this session's claims are
unadjudicated and the driving conclusion was wrong once already. Check
item 55's chain and the new guard, then take the marked rows (item 42).`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

55. **The road's missing twelve nodes a search — the headline.**
    `docs/ROADS.md` §7: the search costs 208, 222 and 178 nodes on run14's
    frames 10, 11 and 171 where the original costs 220, 248 and 189, and
    `Sim::plan_roads` stays off until it is exact. Frame 10 carries **no
    jitter noise** — frames 0–9 match the trace draw for draw, so both
    sides start it on the same word — and frame 11 is two searches, which
    is what says the deficit is a **constant per search** (+12, +13 +13,
    +11), about three expansions, not a percentage. The world, the
    heights, the territory term, the fog, the cost function, the
    containers, the wheel, the endpoints and the direction are each ruled
    out by measurement. §7's last paragraph names the three shapes left.

38. **One long traced capture, human versus AI, on both maps.** run14's
    trace reaches 284 of run10's 1,772 frames, so nothing on disk measures
    the whole against the original by site. Gamelog at `UNITS=3` plus
    `rontrace`, ≥ 1,800 frames, the recipe in `docs/ORACLE.md`; then pin
    its headline beside run10's and retire the tests it supersedes. It
    would also lengthen the ledger sixfold.

35. **`mylos` as a cache.** `docs/VISION.md` §7 has the design — and, as of
    2026-08-28, the correction that its list of refresh sites was wrong
    and an open question the dump does not answer: player 1 carries no
    `0x4000000` at the end of frame 202 or 203, yet its Scout's `mylos`
    moves 4 → 6 across them. Settle that before modelling the bit; the
    check is a `rontrace` run over frames 195–210 — item 38's capture.

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

56. **The cell's `BUILDING` bit, which nothing here sets.** Item 55's
    widening found it: run13's frame-95 world carries `cell::BUILDING` on
    cell `(52, 22)` and this simulation does not, because every cell that
    has the bit got it from the start dump and a building finished later
    leaves it clear. `crate::army`'s muster search reads it
    (`docs/ARMY.md` §13), so it is a live gap. Find the writer — it is not
    `World::set_building_at`, which writes the *tile* mask's `0x3` — and
    the clearer with it, then tighten the guard to zero differences.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture:* `UNITS=3` +
    `GROUPS=1` over a group ordered one way, turned right round while
    marching, re-ordered — in Refused or an Echelon, which also settles the
    `angles` byte's sign (item 19). `docs/GROUPS.md` §13. Fold into item 38.

49. **Collision's unrun arms.** `docs/COLLISION.md` §8 lists what no
    capture has executed: three of the four soft-collision arms, the
    wait-for-it branch, the throttle above four repaths, the
    `pause = % 9 + 1` draw, and any `BLOCK_RADIUS ≠ 1` unit. §9 names the
    capture for each. Fold into item 38's.

39. **A debug viewer.** A thin, read-only 2D client over `Sim` state — map,
    units, fog, order lines, with the original's dump overlaid for the same
    frame. It needs no parity, would have shown the untyped trained unit
    and the seven-frame stand on sight, and is the renderer's first slice.
    Take it when a residue is opaque in the tables.

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
    because their wander rides a stream held only at those frames
    (`docs/SYNC.md` §4.2's tail). Widen the diff to the `ANIMALDATA`
    record — 40 of run10's 54 objects, 70,960 animal-frames, uncompared to
    this day — as its own sub-score. **Measured 2026-08-28:** they track
    the original exactly to frame 90 and first part at 91, which is the
    stream's own drift showing. `rondata::artdata` leaves `FARMPIG` and
    `FARMCHICKEN` out of the length table — they name a `-TYPE0` and a
    `-TYPE1` and no `-TYPE2` — so until that is settled their idle rolls
    never wrap here (`docs/ANIM.md` §3.1).

51. **The AI's long-run economy, now that the roster is measured both
    ways.** `FrameResult::extra_units` is the mirror of `unlinked`, and
    together they say the AI reaches eight citizens on the original's
    frames and then stalls: the original trains `1/9` at 1297 and `1/10` at
    1505 and this simulation reaches neither inside 1,772 frames. Two
    threads, in order: the AI's citizens spend long stretches on `AttackTo`
    moves rather than gathering (`docs/ARMY.md`), and the income itself —
    `CITY.gatherers` is in every dump at every detail level and is
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
