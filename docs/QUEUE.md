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

*2026-08-30, after item 74 (Opus).*

**run33's word 776 → 780**, totals 944/828 → **951/838**. run10's headline
is unmoved at ticks 572, orders 776 — but its **roster is**, by its largest
step: 468 missing + 0 extra → **268 + 0**, the lowest it has been, because
`1/9` now arrives on the original's own frame. Collision 87,548 → 93,341,
angles 33,992 → 35,868. East Indies unmoved at 167/167.

**The AI's thirty-two food was two lumps, and no reading found them.**
Widening run40's leader record showed `leftover` exact on all forty frames
— so a lump, not a rate. run13's `[Start]` census (frames 95–105, same
game) put it in `(105, 560)`. `rontrace-run40.log`'s first-entry list,
crossed against every `bucket` writer in the decompile, named two:
`Build::do_bonus` on 166 (twenty food, a farm finishing) and
`Build::refund_cost` on 201 (twelve, a queued tech re-priced when Science
rose). Both landed; `docs/ECONOMY.md` and `docs/COSTS.md` write them out.

**Owed:** 40 (ORACLE.md's pin is 138,489, and it owes run40/run41 rows),
42.

**Opener (Opus):** `take item 83 from @docs/QUEUE.md — nothing in the
harness reads the dump's own tribe, so every traced game has been played
with no nation power on either side; run40's resource_cap measures the
first one.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, and say so.
Numbers are stable; the journal is indexed by them.

83. **Nothing reads the dump's `tribe`.** So every traced game is played
    with **no nation power on either side** — none of the twenty-four. The
    dump prints `tribe 11` and `tribe 4`, which `rondata`'s nation table
    maps to the British and the Nubians. Oracle on disk: run40's
    `resource_cap`, 1392 against 1120, `70 × 125 / 100` truncated before
    the `<< 4` (ECONOMY, "The commerce cap"). Wiring `tribe` to `Nation`'s
    flags is the item.

84. **The word at 780, four frames past the city.** Eleven draws against
    eight; ours opens `GameAccess::rnd+0x20 < Unit::do_job+0x67` where the
    original has `Unit::do_non_flat_gather+0x54b`. A citizen is on a job
    where the original has it gathering.

85. **`gather_slots`, and a slot `get_good` cannot produce.** 120 of the
    run40 diff's 480 good-frames. `Build::init` surveys a camp against its
    still-empty `gather_from`, so a harness camp activates with zero; and
    run40's *human* files one under good 2, where `get_good@0063bd50`'s
    table at `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5.
    Second writer, unread: `plan_strategy@006b9620` line 1137, the whole
    array from `City::count_gather_slots@00737dc0`.

86. **The science discount's purchase side.** `Sim::tech_price` passes
    `Modifiers::default()`, so a tech is charged undiscounted, and
    `Build::refund_cost` (landed) inverts an expression the buy side never
    applies. Zero on every purchase a traced game reaches, so nothing
    measures it. `calc_science_discount@006da630`, with age-behind beside it.

82. **A hundred knowledge, oil and wealth nobody gave anybody.** run40:
    the original holds **0** in goods 3, 4 and 5 for both players and this
    crate **100**. Inert while none is available — an unavailable good is
    never charged — so a loader question, until the AI ages up.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000`
    at the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6.
    Ours moves at 202 too, so what is owed is the *cache*. run33's trace
    covers 195–210.

69. **The second map's own first divergence, at 168.** East Indies is
    167/167 and what parts it is an order-list **length**: `1/4` holds two
    orders on the original's 168 where this holds one, its position parting
    the same frame; `1/5` at 186 and `1/3` at 202 the same. run39's
    160–210, `rontrace-run39.log` beside them.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80 and now 74 (COSTS §Paying called `refund_cost` a
    cancel refund while its own open questions had it right) — four in a
    row. Both halves: every `name@00xxxxxx` and `+0xNN` a document pins,
    checked across documents and against the module implementing it, and
    every "halved"/"every frame"/"cleared" verb a document uses about a
    field this crate owns. `report.py`'s `blind` collects the citations.

36. **`Unit::set_angle`'s seventeen other callers.** Most of run10's
    angle rows where the positions agree, and the farmers are all of it:
    `0/3`–`0/5`, `1/3`–`1/5` and `1/8` (GROUPS §4.1).

37. **The arrival frame's facing.** Three rows in run10 (96, 362, 721),
    each the AI scout the frame after an `EXPLORE_TO` arrival, where the
    original's body has already snapped onto the order's angle. MOVEMENT's
    open questions name the suspect; `GUYS=2` prints `guy_flags`.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list
    (COLLISION §3, §7), and the dump's `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit here holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) and its six
    writers of `−1` are unmodelled. The pool slot is the hard half: run33's
    scout goes 65 → 64 on 96 where `get_open_slot`'s `last_group` rule
    (GROUPS §3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does not;
    `army`'s muster search reads it (ARMY §13). Find the writer — not
    `World::set_building_at`, which writes the *tile* mask.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION
    §9's pause capture, two units ordered head-on.

40. **The spec/story split, one document per touch.** Documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. Keep the
    rules, formulas and a **Coverage** section; the story goes to the
    journal; lower the pin.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: nine
    audits of 2026-08-20 and five of 08-23/25, adjudicated on Opus and never
    ratified. Marked rows only; also rule on a **leads not pursued** heading
    (journal, 08-28).

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so a wrong animal is invisible until an
    untraced run. Widen to `ANIMALDATA`: 70,960 uncompared.

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`, from
    `Wall::init` for every non-farm building: the footprint-plus-pad box of
    `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`, *after* `place_roads`. Oracle on disk:
    run13's `FRAME 100` against run32's `FRAME 104`.

58. **The height loader's arithmetic.** `rondata::diff` means the heights
    in exact millionths; the original does `(f32 + f32) × 0.5f` and
    truncates, differing on three of run12's tiles. Do it in `f32`:
    run32's first search costs 1,046 nodes against 1,043 (ROADS §7.1).

60. **The second search of a frame, 410 nodes short.** run32's Smelter
    lays the original's road and costs 1,460 nodes against 1,870, on the
    *second* `place_roads` of its frame — where `PathFinder`'s own state
    between searches would show. ROADS §7.1 rules out the rest.

62. **The standing swap, at frame 99.** The first frame whose draw
    *sequence* differs, costing no word: `Guy::set_anim+0x97a` under our
    `Unit::do_idle+0x7d`, the original's `Guy::inc_time+0x271`. SYNC §6.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid, for an opaque residue; (41) `scenario.py`'s fate — run `zsh
tools/fuzz/run.sh 424242 1000 1300` then `report.py … blind docs/` before
deleting it; the `CITY` widening; a `find_target` block;
run7's order stream under the trace; a mounted attacker; a caravan;
`Leader::diplomacy`; `calc_gather` non-flat; and (77) ANIM §3.2's
`AGE3`/`AGE5` piece rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`.
- **Start of session:** "Where things stand", the item, then its document.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both.
- **Never** quote this file or the journal into `CLAUDE.md`, a subagent
  brief, an agent definition, or a memory hook.
