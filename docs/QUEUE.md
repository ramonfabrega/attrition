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

*2026-08-28, after the road capture was staged and blocked (Opus).*

**The headline — run10, 1,772 frames: ticks before divergence 181, orders
180.** Unmoved. **The ledger — run14: 198 of 284 frames.** Unmoved.

**Blocked on the user, and nothing else is: macOS Accessibility is off
for this terminal.** `cliclick p` answers `0,0` and an `osascript` window
query answers *not allowed assistive access (-1728)*, so the lobby cannot
be clicked; Screen Recording and Automation both pass, and the three come
back separately. **System Settings → Privacy & Security → Accessibility,
for the terminal Claude Code runs in.**

**Item 55's capture is staged and is one command:
`zsh tools/gamelog/roadcapture.sh`.** Its site was chosen against the
simulation rather than by eye (journal, today): a Granary at `(6, 171)`
and a Smelter at `(33, 161)`, both binding to p0's city and laying road on
fresh sloped ground, replanning on sim-frames 105 and 104 under the
`DUMP_ALL` window `[104, 109)` and the trace. `add` without `NEW` calls
`Build::activate` itself, so no hand is needed past the lobby. **The
install is left staged** — anything else launching the game first gets a
full dump; `tools/gamelog/window.py restore` undoes it.

**Owed:** 40. **Needs the user:** the Accessibility checkbox.

**Opener (Opus):** `Accessibility is on — run
zsh tools/gamelog/roadcapture.sh, then item 55: diff the road tile by tile.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

55. **The road's missing twelve nodes a search — the headline, by a path
    oracle.** `docs/ROADS.md` §7: 208, 222 and 178 costed nodes against
    220, 248 and 189, and three readings that agree on every line. Do not
    read it a fourth time. **The capture is `tools/gamelog/roadcapture.sh`**
    (staged 2026-08-28; it needs Accessibility). Then: take the trace's
    seed at the search's first `calc_road_cost` draw, build a `Sim` from
    the frame block's own `WORLD` and `master_land_heights`, set
    `sim.rng.seed` to that word, run `find_road` from the new building and
    **diff the road tile by tile** before touching `astar_road`; then
    re-measure the ledger with `plan_roads` on.

38. **One long traced capture, human versus AI, on both maps.** run14's
    trace reaches 284 of run10's 1,772 frames, so nothing on disk measures
    the whole against the original by site. Gamelog at `UNITS=3` plus
    `rontrace`, ≥ 1,800 frames, the recipe in `docs/ORACLE.md`; then pin
    its headline beside run10's and retire the tests it supersedes. Item
    55's capture is its first minute; fold them if the trace holds.

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
    frame. No parity needed; it is the renderer's first slice. Take it when
    a residue is opaque in the tables.

40. **The spec/story split, one document per touch.** The documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. When a
    session touches one: keep the rules, fields, formulas and a **Coverage**
    section (diff-backed / reading-only / unmodelled) — which is also a
    blind reader's brief; move the narrative to the journal; lower the pin.

41. **The second-map harness, and `scenario.py`'s fate.** `seedini.py` and the
    early window stay. `scenario.py`'s cheat generator has produced nothing and
    cannot issue orders; before deleting it, run the one measurement that could
    redeem it — `zsh tools/fuzz/run.sh 424242 1000 1300`, then
    `tools/trace/report.py … blind docs/` against both traces — and record
    whether any of its 188 extra functions is on the blind list.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: nine
    audits of 2026-08-20 and five of 2026-08-23/25, adjudicated on Opus and
    never ratified. A steering session's job, over marked rows only; a
    capture retires a row faster than a pass.

45. **Gaia's animals, the re-seat crutch, and the pasture's lengths.**
    `Sim::reseat_animal` puts the animals back from every traced dump
    (`docs/SYNC.md` §4.2's tail). Widen the diff to the `ANIMALDATA`
    record — 70,960 animal-frames on run10, uncompared — as a sub-score;
    they track the original to frame 90 and part at 91. `rondata::artdata`
    lacks `FARMPIG`/`FARMCHICKEN` lengths (`docs/ANIM.md` §3.1).

51. **The AI's long-run economy.** The roster pins (`extra_units`,
    `unlinked`) say the AI reaches eight citizens on the original's frames
    and then stalls: `1/9` at 1297 and `1/10` at 1505 are never trained
    here. Two threads: citizens on `AttackTo` moves rather than gathering
    (`docs/ARMY.md`), then the income — `CITY.gatherers` is in every dump
    and uncompared. Score it on the roster pin, 744 + 0.

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`, from
    `Wall::init` for every non-farm building: the footprint-plus-pad box of
    `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`. The frame-0 dump already carries the
    setup buildings', but a building placed *during* a game moves the grid
    behind `Unit::update_z`, `Leader::compute_site_stats` and the road
    cost. The software float carries it; any frame window after a
    placement is the oracle. `docs/ROADS.md` §7.

58. **The height loader's arithmetic.** `rondata::diff` means the heights in
    exact millionths; the original does `(f32 + f32) × 0.5f` and truncates,
    and on run12 they differ on tiles (14, 147), (99, 173), (14, 223). Do it
    in `f32`, pin the three. Ten lines, and a prerequisite of 57.

Older backlog, unchanged: the `LEADERDATA` and `CITY` widenings; a
`find_target` block; run7's order stream under the trace; a mounted
attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
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
