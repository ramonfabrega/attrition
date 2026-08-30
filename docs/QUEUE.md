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

*2026-08-30, after item 81 (Opus).*

**ticks 572, orders 576 → 776 — two hundred frames**, and run33's word
576 → **776**, totals 802/688 → **944/828**, the largest move either
has made. Player 0 687 → **802**. East Indies unmoved at 167/167.

**A building's ramp has no ceiling.** `TypeData::get_cost` has two: the
unit arm reads `UNIT_COST_FACTOR` and one of the four `*_RAMP_MAX`, the
building arm (`00665787`–`00665b5a`) reads `BUILD_COST_FACTOR`,
`BUILD_SUPPORT_FACTOR` and **no ceiling at all**. This crate gave every
building `RampClass::default()`, clamping the Small City's `SUPPORT 50` to
12 and pricing the AI's second city at 22 instead of **60** — so it
founded it on 576 where the original cannot pay until 776. Both readings
of 2026-08-20 called the four ceilings doubly confirmed, on one arm.

**Two new captures, and they are the census oracle.**
`tools/gamelog/censuswindow.sh` is run10's game with `LEADERS=9` over a
frame window — run40 `[560, 600)`, run41 `[770, 800)`. They carry the AI's
own `resources`, and they measured the sixty twice over.

**Owed:** 40 (ORACLE.md's pin is 138,489, and it owes run40/run41 rows),
42.

**Opener (Opus):** `take item 74 from @docs/QUEUE.md — the AI's food is
thirty-two short on every frame of run40's window and it is what parts
the word at 776; gamelog-run40-census.txt beside it.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

74. **The AI's food, thirty-two short — and it is the word.** run40
    measures it: `1/`'s food is **36 against 68** on every frame of
    `[560, 600)`, where player 0's food and both players' timber and metal
    are exact on all forty. That gap is why this cannot pay for the second
    city on 776 when the original can — the word parts there, twenty-five
    draws against five — and why `1/9` arrives on 1497 against 1297 (the
    roster is **468 missing + 0 extra**, from 268 + 400). run40's
    `LEADERDATA` also prints `income`, `rate`, `leftover` and
    `gather_slots` per good; `CITY.gatherers` is still uncompared.

82. **A hundred knowledge, oil and wealth nobody gave anybody.** run40:
    the original holds **0** in goods 3, 4 and 5 for both players on every
    frame and this crate **100**. Inert while none is available — an
    unavailable good is never charged — so it is a loader question, and it
    stops being inert the moment the AI ages up.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000`
    at the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6.
    Ours moves at 202 too, so what is owed is the *cache*, not the value.
    **run33's trace covers 195–210** — read it.

69. **The second map's own first divergence, at 168.** East Indies is
    167/167 and what parts it is an order-list **length**: `1/4` holds
    two orders on the original's frame 168 where this holds one, its
    position parting the same frame; `1/5` at 186 and `1/3` at 202 are
    the same. run39's 160–210 with `rontrace-run39.log` beside them.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Item 68 (ARMY §4 vs SCOUT §2 on `005f7615`), item 79 (VISION §3
    pinned `UnitData +0x50`, the code read another field), item 80
    (PATHFINDER §8 described a per-frame decay the code never ran) —
    three in a row, and the third had no address at all. So both halves:
    every `name@00xxxxxx` and `+0xNN` a document pins, checked across
    documents and against the module implementing it, **and** every
    "halved"/"every frame"/"cleared" verb a document uses about a field
    this crate owns. `report.py`'s `blind` collects the citations.

36. **`Unit::set_angle`'s seventeen other callers.** Most of run10's
    angle rows, where the positions agree — now 7,870 of 31,022, and the
    farmers are all of it: `0/3`–`0/5`, `1/3`–`1/5` and `1/8` are 7,818,
    no other unit above 28 (GROUPS §4.1).

37. **The arrival frame's facing.** Three rows in run10 now (96, 362,
    721), each the AI scout the frame after an `EXPLORE_TO` arrival, where
    the original's body has already snapped onto the order's angle.
    MOVEMENT's open questions name the suspect; `GUYS=2` prints
    `guy_flags` — a grep.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list
    (COLLISION §3, §7). The dump prints `down`/`down_who` on every object
    and the harness compares none of it.

73. **`UnitData::group`, compared on no frame.** In every `UNITDATA`
    record. No unit here holds the back-pointer, so `Group::normalize`'s
    cull (GROUPS §4.3) and its six writers of `−1` are unmodelled. The
    pool slot is the hard half: run33's scout goes 65 → 64 on 96 where
    `get_open_slot`'s `last_group` rule (GROUPS §3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does
    not: every cell with the bit got it from the start dump. `army`'s
    muster search reads it (ARMY §13). Find the writer — not
    `World::set_building_at`, which writes the *tile* mask.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching,
    re-ordered, in Refused or an Echelon (GROUPS §13) — and with it
    COLLISION §9's pause capture, two units of one player ordered head-on.

39. **A debug viewer.** A thin read-only 2D client over `Sim` state —
    map, units, fog, order lines, the dump overlaid. For an opaque residue.

40. **The spec/story split, one document per touch.** Documents over
    60 KB are pinned in `docs_guard::OVER` and may only shrink. Keep the
    rules, formulas and a **Coverage** section; the story goes to the
    journal; lower the pin. The journal of 08-30 argues the metric should
    be "the largest chunk an item must read", not file bytes.

41. **`scenario.py`'s fate.** Its cheat generator has produced nothing
    and cannot issue orders; before deleting it run `zsh
    tools/fuzz/run.sh 424242 1000 1300` then `report.py … blind docs/`,
    and say whether any of its 188 functions is on the blind list.

42. **The ratification ledger, in batches.** `docs/audit/README.md`: nine
    audits of 2026-08-20 and five of 08-23/25, adjudicated on Opus and
    never ratified. Marked rows only; also rule on a **leads not pursued**
    heading for audit records (journal, 08-28).

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so a wrong animal is invisible until an
    untraced run. Widen the diff to `ANIMALDATA` (70,960 uncompared).

57. **The terraform.** `TerrainOut::terraform_for_building@00875210`,
    from `Wall::init` for every non-farm building: the footprint-plus-pad
    box of `master_land_heights` set to its mean, the border blended
    `(h + mean) × 0.5`, in `f32`, *after* `place_roads`. **Oracle on
    disk**: run13's `FRAME 100` against run32's `FRAME 104`.

58. **The height loader's arithmetic.** `rondata::diff` means the heights
    in exact millionths; the original does `(f32 + f32) × 0.5f` and
    truncates, and on run12 they differ on three tiles. Do it in `f32`:
    run32's first search costs 1,046 nodes against 1,043 (ROADS §7.1).

60. **The second search of a frame, 410 nodes short.** run32's Smelter
    lays the original's road and costs 1,460 nodes against 1,870, and it
    is the *second* `place_roads` of its frame — where `PathFinder`'s own
    state between searches would show. ROADS §7.1 rules out the rest.

62. **The standing swap, at frame 99.** The first frame whose draw
    *sequence* differs, costing no word: ours spends `Guy::set_anim+0x97a
    < Unit::do_idle+0x7d` where the original has `< Guy::inc_time+0x271`.
    SYNC §6; it needs the gate keeping a standing unit out.

Older backlog (the `LEADERDATA` widening is item 74's now): the `CITY`
widening; a `find_target` block; run7's order stream under the trace; a
mounted attacker; a caravan; `Leader::diplomacy`; `calc_gather` non-flat; and
(77) ANIM §3.2's `AGE3`/`AGE5` piece rows, which no capture has aged into.

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
