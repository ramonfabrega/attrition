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

*2026-08-29, after item 68 (Opus).*

**ticks 322, orders 320 — the headline moved seventy frames** (252/252),
player 0 at 356 and player 1 at 323. **run33's word parts at 345**, was
307. East Indies is unmoved at 167/167, which is what says this was not
chased into one map's shape.

**Item 68 was not a clock; it was item 67, and both are closed.** The two
draws frame 307 was short were `1/7`'s, and `1/7` was not at its camp to
spend them because the AI had marched it away on 252:
`Sim::think_join_army` joined "an attacker that is not a scout or a
caravan", where `Unit::think`'s tail at `005f7615` joins **a supply wagon
or a hero and nothing else**. A citizen has an attack, so `1/6` (frame 99)
and `1/7` (205) were conscripted and leader 1's army 0 — `frame ≡ 252 (mod
256)` — sent them on a siege attack. `docs/SCOUT.md` §2 had the listing
right; `docs/ARMY.md` §4 described its complement, and the implementation
followed §4. Two oracles settle it: run33's coverage never enters
`Unit::add_to_army`, and every citizen's dumped `group` is −1 always.
`1/6` now holds to 735 and `1/7` to 937; **player 0 moved with them, 326
→ 356**, so item 65 is gone too.

**Owed:** 40 (ORACLE.md had two passes; the pin is 138,489). ARMY.md's pin
is now 84,493 and ORDERS.md is 188,730, under its 190,800.

**Opener (Opus):** `take item 70 from @docs/QUEUE.md — the AI's ninth
citizen 1/8 is sent to the Woodcutter's Camp 2001 on run10's frame 321
where the original sends it to the farm 2006; find_gather_spot's choice
is the headline's first divergence.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

70. **`find_gather_spot` sends the ninth citizen to the wrong building.**
    The headline's first divergence, both scores: on run10's frame 321
    `1/8`'s gather order names the Woodcutter's Camp `2001`, `dist_mod
    4`, where the original names `2006`, `dist_mod 0` — a **farm** — and
    its position parts at 323. ORDERS §6.6's walk over the owner's
    buildings, against run33's 315–325. Half of item 51.

71. **`0/4`'s farm walk, one tile north-west.** Player 0's first
    divergence, successor to item 65: the path goal parts at **351** —
    ours `(1848, 31800)`, theirs `(2040, 31992)` — the position at 356,
    and `0/3`/`0/5` hold to 450 and 455, so it is one farmer's cell
    pick. ORDERS §6.5, run33's 345–360.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000`
    at the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6.
    **run33's trace covers 195–210** — read it.

69. **The second map's own first divergence, at 168.** East Indies is
    167/167 and what parts it is an order-list **length**: `1/4` holds
    two orders on the original's frame 168 where this holds one, its
    position parting the same frame; `1/5` at 186 and `1/3` at 202 are
    the same. run39's 160–210 with `rontrace-run39.log` beside them.

72. **Two documents citing one address are a diff waiting to be run.**
    Item 68 was ARMY §4 and SCOUT §2 disagreeing about `005f7615` for
    four days, the implementation following the wrong one. Cheapest
    form: list every `name@00xxxxxx` cited by more than one document —
    `report.py`'s `blind` already collects the citations.

36. **`Unit::set_angle`'s seventeen other callers.** Most of run10's
    angle rows, where the positions agree — now 9,378 of 24,120. Start at
    `do_gather`'s: `0/2` at 432–433 is thousands (GROUPS §4.1).

37. **The arrival frame's facing.** Two rows in run10, the AI scout the
    frame after an `EXPLORE_TO` arrival. MOVEMENT's open questions name
    the suspect; `GUYS=2` prints `guy_flags` — a grep.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list
    (COLLISION §3, §7). The dump prints `down`/`down_who` on every object
    and the harness compares none of it.

73. **`UnitData::group`, compared on no frame.** In every `UNITDATA`
    record; item 68 read it by hand. No unit here holds the back-pointer,
    so `Group::normalize`'s cull (GROUPS §4.3) and the six writers of
    `−1` (`do_non_flat_gather`, `do_build`, `build_done`, `do_attack`,
    `do_group_move`, `think_peasant`) are unmodelled. The pool slot is
    the hard half: run33's scout goes 65 → 64 on frame 96 where
    `get_open_slot`'s `last_group` rule (GROUPS §3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does
    not: every cell with the bit got it from the start dump. `army`'s
    muster search reads it (ARMY §13). Find the writer — not
    `World::set_building_at`, which writes the *tile* mask's `0x3`.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group ordered one way, turned right round
    while marching, re-ordered, in Refused or an Echelon (GROUPS §13).

39. **A debug viewer.** A thin read-only 2D client over `Sim` state —
    map, units, fog, order lines, the dump overlaid. For an opaque
    residue.

40. **The spec/story split, one document per touch.** Documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. Keep the
    rules, fields, formulas and a **Coverage** section; move the story to
    the journal; lower the pin. ORDERS.md went that way 2026-08-29.

41. **`scenario.py`'s fate.** Its cheat generator has produced nothing
    and cannot issue orders; before deleting it run `zsh
    tools/fuzz/run.sh 424242 1000 1300` then `report.py … blind docs/`,
    and say whether any of its 188 extra functions is on the blind list
    (101 as of run33).

42. **The ratification ledger, in batches.** `docs/audit/README.md`:
    nine audits of 2026-08-20 and five of 08-23/25, adjudicated on Opus
    and never ratified. Marked rows only; also rule on a **leads not
    pursued** heading for audit records (journal, 08-28).

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so a wrong animal is invisible until an
    untraced run. Widen the diff to `ANIMALDATA` (70,960 uncompared
    animal-frames on run10).

51. **The AI's long-run economy.** The AI reaches eight citizens on the
    original's frames and stalls: `1/9` at 1297 and `1/10` at 1505 are
    never trained here. Item 70's building choice, then the income —
    `CITY.gatherers`, in every dump and uncompared. Score it 744 + 0.

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`,
    from `Wall::init` for every non-farm building: the footprint-plus-pad
    box of `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`, *after* `place_roads`. **The oracle is
    on disk**: run13's `FRAME 100` heights against run32's `FRAME 104`,
    128 corners (ROADS §7.1).

58. **The height loader's arithmetic.** `rondata::diff` means the
    heights in exact millionths; the original does `(f32 + f32) × 0.5f`
    and truncates, and on run12 they differ on tiles (14, 147), (99,
    173), (14, 223). Do it in `f32`: run32's first placement search costs
    1,046 nodes against 1,043 (ROADS §7.1).

60. **The second search of a frame, 410 nodes short.** run32's Smelter
    lays the original's road and costs 1,460 nodes against 1,870, and it
    is the *second* `place_roads` of its frame — where `PathFinder`'s own
    state between two searches would show. ROADS §7.1 rules out the rest.

62. **The standing swap, at frame 99.** The first frame whose draw
    *sequence* differs, costing no word: ours spends `Guy::set_anim+0x97a
    < Unit::do_idle+0x7d` where the original has `< Guy::inc_time+0x271`.
    SYNC §6; it needs the gate keeping a standing unit out.

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
