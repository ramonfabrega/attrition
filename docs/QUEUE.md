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

*2026-08-30, after item 76 (Opus).*

**ticks 436, orders 427 — the headline moved seventy-four frames**
(362/361), and **player 1's scout `1/0` went 363 → 484**. run33's word
361 → **432** and its totals 696/523 → **724/606**. **Player 0 fell
464 → 450**, and every unit that fell parts *after* the word does. East
Indies unmoved at 167/167.

**The scout's surface probe was reading the wrong tile.** SCOUT §7 had
`tdata[(4y + 2)·tile_xs + 4x]` from a decompiled `+ 4` read as a field
offset — but `TData` is `size 0x2` with `mask` at `+0`, so four bytes is
**two elements** and the tile is the cell **centre**, the same one
`invalid_loc` is handed four lines later. Cell `(48, 23)` is ocean at
tile `(192, 94)` and land at `(194, 94)`, so run33's frame-361 re-target
refused the candidate the original takes. Every capture that had
exercised the mechanic was a **frame-0** one, where no candidate cell
straddles a shoreline. Lesson in `docs/audit/README.md`.

**Owed:** 40 (ORACLE.md had two passes; the pin is 138,489). **ORDERS.md
is 190,546 of its 190,800.**

**Opener (Opus):** `take item 78 from @docs/QUEUE.md — run33's word parts
at 432 on a gather stand, twenty-three draws against twenty-two; run33's
425–440 with rontrace-run33.log beside it.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

78. **The word's next parting, at 432 — a gather stand.** run33's
    word now runs to 431 and parts on **432**, twenty-three draws
    against twenty-two: at index 15 ours is
    `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` where the
    original spends a farm's `Farms::inc_time+0x1ae`. So a gatherer
    stands here that does not there. run33's 425–440 with
    `rontrace-run33.log` beside it; ORDERS §6.6, ANIM §5.

74. **The AI's long-run economy** (was item 51, now the other way up).
    Item 70 put `1/9` back and it arrives on **897** against the
    original's 1,297; `1/10` at 1505 is still never trained. The roster
    pair is 268 missing + 400 extra and the extra is all this one unit,
    so the AI now earns food faster than the original rather than
    slower. `CITY.gatherers` is in every dump and uncompared, and a
    `LEADERS` dump prints `resource_cap`/`income`/`rate` per good.
    Score it 268 + 400.

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
caravan; `make_stuff` whole; `Leader::diplomacy`; `calc_gather` non-flat. New
(77): ANIM §3.2's `AGE3`/`AGE5` piece rows, which no capture has ever aged
into, and `get_unit_gpiece`'s fall-back walk with them.

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
