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

*2026-08-28, after item 64 (Opus).*

**The headline is ticks 209, orders 208** — up from 207/206. Player 0
holds at 326, player 1 moves 208 → 210. run14's traced word is unmoved
at 232 and 260 of 284 frames still match draw for draw.

**The item was a stated seam, and the answer was already in `docs/`.**
`resolve_unit_collision`'s step 2 asks the gather target's type `+0x94`;
that is `BuildTypeData::is_flat`, named in `docs/CITIES.md` §1.5 and read
correctly by `add_gather_order` one function away. `collide.rs` read it
as `true`, so a woodcutter bumped on its own camp's footprint killed its
walk and re-made it every other frame for ever.

**The collision block now agrees entire**: 0 disagreements in 48,790
field-frames, from 245 in 42,840 — and 243 of those were the sticky
`collide_guy` nobody could explain away. The pin is emptiness, not a
ceiling.

**Owed:** 40. **Needs the user:** nothing.

**Opener (Opus):** `take item 66 from @docs/QUEUE.md — the AI's new
citizen 1/7 is sent to the cell 1/6 is standing on; find_nearby_spot
does not filter by other units' positions or their ordered ones.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

66. **`find_nearby_spot` walks onto the unit already standing there.**
    The first divergence (player 1 @ 210). `1/7` is trained on 206 and
    sent to its camp: ours picks `(40680, 17688)` — the cell `1/6` is
    standing on — the original `(40680, 18024)`, seven cells further
    south. `docs/ORDERS.md` §10 states the seam: the ring-and-bearing
    sweep's collision half, "other units' positions **and ordered
    positions**", is unmodelled, so the first walkable candidate wins.
    Read `UnitType::find_nearby_spot`'s filter; run10's frames 206–212
    for `1/7`, whose `MOVEORDER` names both destinations.

65. **The human farmers' re-target, a frame late and a cell out.** Player
    0's first divergence (326): `0/3` and `0/5` re-pick on the original's
    325 and ours' 326, onto a different cell. `docs/ORDERS.md` §6.5's
    clock against run10's frames 320–330.

38. **One long traced capture, human versus AI, on both maps.** run14's
    trace reaches 284 of run10's 1,772 frames, so nothing on disk measures
    the whole by site. Gamelog at `UNITS=3` plus `rontrace`, ≥ 1,800
    frames, the recipe in `docs/ORACLE.md`; then pin its headline beside
    run10's and retire the tests it supersedes.

35. **`mylos` as a cache.** `docs/VISION.md` §7 has the design and an open
    question the dump does not answer: player 1 carries no `0x4000000` at
    the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6 across
    them. Settle it with a `rontrace` run over 195–210 — item 38's.

36. **`Unit::set_angle`'s seventeen other callers.** Most of run10's angle
    rows, on frames where the positions agree. Start at `do_gather`'s: unit
    `0/2` at 432–433 is thousands of them and one screen of trace; each
    caller is where a group's mirror flag moves (`docs/GROUPS.md` §4.1).

37. **The arrival frame's facing.** Two rows in run10, the AI scout the
    frame after an `EXPLORE_TO` arrival. `docs/MOVEMENT.md`'s open
    questions name the suspect; `GUYS=2` prints `guy_flags` — a grep.

48. **The object chain, whole.** `crates/sim/src/collide.rs` chains units
    only; the original threads buildings and goodies through the same list
    (`docs/COLLISION.md` §3, §7). The units' order is unaffected — but the
    dump prints `down`/`down_who` on every object at every detail level and
    the harness compares none of it. The cheapest untaken widening.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does
    not: every cell with the bit got it from the start dump, and a building
    finished later leaves it clear. `crate::army`'s muster search reads it
    (`docs/ARMY.md` §13). Find the writer — not `World::set_building_at`,
    which writes the *tile* mask's `0x3` — and the clearer with it.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture (fold into 38):*
    `UNITS=3` + `GROUPS=1`, a group ordered one way, turned right round
    while marching, re-ordered, in Refused or an Echelon — which also
    settles the `angles` byte's sign (item 19). `docs/GROUPS.md` §13.

39. **A debug viewer.** A thin, read-only 2D client over `Sim` state — map,
    units, fog, order lines, the original's dump overlaid for the same
    frame. The renderer's first slice; take it when a residue is opaque.

40. **The spec/story split, one document per touch.** The documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. When a
    session touches one: keep the rules, fields, formulas and a **Coverage**
    section (diff-backed / reading-only / unmodelled) — which is also a
    blind reader's brief; move the narrative to the journal; lower the pin.

41. **`scenario.py`'s fate.** Its cheat generator has produced nothing and
    cannot issue orders; before deleting it run the one measurement that
    could redeem it — `zsh tools/fuzz/run.sh 424242 1000 1300`, then
    `report.py … blind docs/` against both traces — and record whether any
    of its 188 extra functions is on the blind list.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: nine
    audits of 2026-08-20 and five of 2026-08-23/25, adjudicated on Opus and
    never ratified. Marked rows only; also rule on a **leads not pursued**
    heading for audit records — the slot `FABLE:` lacks (journal, 08-28).

45. **Gaia's animals, and the pasture's lengths.** `Sim::reseat_animal`
    puts the animals back from every traced dump (`docs/SYNC.md` §4.2), so
    a wrong animal is invisible until an untraced run — item 49's sheep
    cost the trace score for a week. Widen the diff to `ANIMALDATA` —
    70,960 uncompared animal-frames on run10 — as a sub-score;
    `rondata::artdata` lacks `FARMPIG`/`FARMCHICKEN` lengths
    (`docs/ANIM.md` §3.1).

51. **The AI's long-run economy.** The roster pins say the AI reaches eight
    citizens on the original's frames and then stalls: `1/9` at 1297 and
    `1/10` at 1505 are never trained here. Two threads: citizens on
    `AttackTo` moves rather than gathering (`docs/ARMY.md`), then the income
    — `CITY.gatherers`, in every dump and uncompared. Score it 744 + 0.

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`, from
    `Wall::init` for every non-farm building: the footprint-plus-pad box of
    `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`. It moves the grid behind `Unit::update_z`,
    `compute_site_stats` and the road cost, and **the oracle is on disk**:
    run13's `FRAME 100` heights against run32's `FRAME 104`, the same grid
    before and after two placements, 128 corners. It runs *after*
    `place_roads` (`docs/ROADS.md` §7.1).

58. **The height loader's arithmetic.** `rondata::diff` means the heights in
    exact millionths; the original does `(f32 + f32) × 0.5f` and truncates,
    and on run12 they differ on tiles (14, 147), (99, 173), (14, 223). Do it
    in `f32`, pin the three — now with a score attached: run32's first
    placement search costs 1,046 nodes against 1,043, the size of a
    rounding difference (`docs/ROADS.md` §7.1).

60. **The second search of a frame, 410 nodes short.** run32's Smelter lays
    the original's road and costs 1,460 nodes against 1,870, and it is the
    *second* `place_roads` of its frame — where `PathFinder`'s own state
    between two searches would show (the `Recycler<PathNode>` pool, the
    containers' reuse). `docs/ROADS.md` §7.1 lists what measurement has
    ruled out; the capture is on disk, so this is a reading and a re-run.

62. **The standing swap, at frame 99.** The first frame whose draw
    *sequence* differs, costing no word: ours spends `Guy::set_anim+0x97a
    < Unit::do_idle+0x7d` where the original spends `< Guy::inc_time+0x271`.
    `docs/SYNC.md` §6; it needs the gate keeping a standing unit out.

Older backlog, unchanged: the `LEADERDATA` and `CITY` widenings; a
`find_target` block; run7's order stream under the trace; a mounted attacker; a
caravan; `make_stuff` whole; `Leader::diplomacy`; `calc_gather` non-flat.

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
