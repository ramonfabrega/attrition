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

*2026-08-29, after item 38 (Opus).*

**The headline is unmoved at ticks 252, orders 252** — this item bought an
instrument, not a tick. **The trace sub-score moved 284 → 307.**

**run33 replaces run14.** run10's game, traced, 1,850 frames
(`tools/gamelog/longtrace.sh`, fourteen minutes unattended), and it is
run10's game by measurement rather than by the frame-0 word:
`tools/gamelog/samegame.py` digests every `BEGIN FRAME` block and run10
against run33 is **1,771 in common and none differing**, so the traced
executable, the `int 3`s and `!ffwd` are invisible over 1,771 frames where
run18a had checked four.

**The word parts at 307, on a non-flat gather** — item 68. Over the run,
668 frames spend the original's number of draws and 556 are its draws in
its order. **The blind list did not move**: 101 never-entered with run33
in or out, so a run booked to shrink it must do what no earlier run did.

**East Indies (`MAP_STYLE 18`, seed 12345) is being captured** as this is
written — run36 (`DUMP_ALL` start) and run37 (the long trace) — for the
second map the finish line names; wiring it into the harness is item 69.

**Owed:** 40 (ORACLE.md had its first pass; the pin is 138,731).

**Opener (Opus):** `take item 68 from @docs/QUEUE.md — run33's word parts
at 307 on a non-flat gather this simulation does not make; the site fires
on frames 1, 168, 204, 307 and it makes the first three.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

68. **The fourth non-flat gather, at 307.** Where run33's word parts: the
    original spends eleven draws and this nine, and the two missing are one
    unit's — a stand from `Unit::do_non_flat_gather+0x10f`, then `+0x54b`.
    The site fires on 1, 168, 204, **307**, 465, 508, … and this simulation
    makes the first three, so it is a clock, not a missing mechanism.

65. **The human farmers' re-target, a frame late and a cell out.** Player
    0's first divergence, and now the headline's own (326): `0/3` and
    `0/5` re-pick on the original's 325 and ours' 326, onto a different
    cell. `docs/ORDERS.md` §6.5's clock against run10's frames 320–330.

67. **What parts `1/6` and `1/7` at 253.** Player 1's new first
    divergence, both units on the same frame: `1/6`'s order kind goes 2
    against the original's 7 (a `MOVE_TO` where it holds a `GATHER`), and
    `1/7` carries one order where it holds two. run10's frames 248–256
    under `UNITS=3`; `docs/ORDERS.md` §6.4's clock is the suspect.

35. **`mylos` as a cache.** `docs/VISION.md` §7's open question: player 1
    carries no `0x4000000` at the end of frame 202 or 203, yet its Scout's
    `mylos` moves 4 → 6. **run33's trace covers 195–210** — read it.

69. **The second map, wired in.** run36/run37 (East Indies, `MAP_STYLE 18`,
    seed 12345) are the `DUMP_ALL` start and the 1,850-frame trace of a
    game no test builds yet. Stand `build_sim` up from run36's own
    `Initial` — it carries its heights, checksums and herds, so it needs no
    sibling — and score it. Phase 3 is done when *both* maps hold.

36. **`Unit::set_angle`'s seventeen other callers.** Most of run10's angle
    rows, on frames where the positions agree. Start at `do_gather`'s: `0/2`
    at 432–433 is thousands of them; each caller is where a group's mirror
    flag moves (`docs/GROUPS.md` §4.1).

37. **The arrival frame's facing.** Two rows in run10, the AI scout the frame
    after an `EXPLORE_TO` arrival. `docs/MOVEMENT.md`'s open questions name
    the suspect; `GUYS=2` prints `guy_flags` — a grep.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list (COLLISION
    §3, §7). The units' order is unaffected — but the dump prints
    `down`/`down_who` on every object at every detail level and the harness
    compares none of it. The cheapest untaken widening.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's frame-95
    world carries `cell::BUILDING` on `(52, 22)` and this does not: every
    cell with the bit got it from the start dump, and a building finished
    later leaves it clear. `crate::army`'s muster search reads it (ARMY §13).
    Find the writer — not `World::set_building_at`, which writes the *tile*
    mask's `0x3` — and the clearer with it.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group ordered one way, turned right round
    while marching, re-ordered, in Refused or an Echelon — `select <type>
    who=N [+]` from the channel does the picking and only the right-clicks
    are human. Settles the `angles` byte's sign too (item 19; GROUPS §13).

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
    `report.py … blind docs/` — and record whether any of its 188 extra
    functions is on the blind list (101 as of run33).

42. **The ratification ledger, in batches.** `docs/audit/README.md`: nine
    audits of 2026-08-20 and five of 08-23/25, adjudicated on Opus and never
    ratified. Marked rows only; also rule on a **leads not pursued** heading
    for audit records — the slot `FABLE:` lacks (journal, 08-28).

45. **Gaia's animals, and the pasture's lengths.** `Sim::reseat_animal` puts
    the animals back from every traced dump (`docs/SYNC.md` §4.2), so a
    wrong animal is invisible until an untraced run — item 49's sheep cost
    the trace score for a week. Widen the diff to `ANIMALDATA` (70,960
    uncompared animal-frames on run10) as a sub-score; `rondata::artdata`
    lacks `FARMPIG`/`FARMCHICKEN` lengths (`docs/ANIM.md` §3.1).

51. **The AI's long-run economy.** The roster pins say the AI reaches eight
    citizens on the original's frames and then stalls: `1/9` at 1297 and
    `1/10` at 1505 are never trained here. Two threads: citizens on
    `AttackTo` moves rather than gathering (ARMY), then the income —
    `CITY.gatherers`, in every dump and uncompared. Score it 744 + 0.

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`, from
    `Wall::init` for every non-farm building: the footprint-plus-pad box of
    `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`, *after* `place_roads`. It moves the grid
    behind `Unit::update_z`, `compute_site_stats` and the road cost, and
    **the oracle is on disk**: run13's `FRAME 100` heights against run32's
    `FRAME 104`, 128 corners (`docs/ROADS.md` §7.1).

58. **The height loader's arithmetic.** `rondata::diff` means the heights in
    exact millionths; the original does `(f32 + f32) × 0.5f` and truncates,
    and on run12 they differ on tiles (14, 147), (99, 173), (14, 223). Do it
    in `f32`, pin the three — with a score attached: run32's first placement
    search costs 1,046 nodes against 1,043, a rounding difference (ROADS §7.1).

60. **The second search of a frame, 410 nodes short.** run32's Smelter lays
    the original's road and costs 1,460 nodes against 1,870, and it is the
    *second* `place_roads` of its frame — where `PathFinder`'s own state
    between two searches would show (the `Recycler<PathNode>` pool, the
    containers' reuse). `docs/ROADS.md` §7.1 lists what is ruled out.

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
