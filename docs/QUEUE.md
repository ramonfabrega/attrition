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

*2026-08-30, after item 80 (Opus).*

**ticks 572, orders 576 — the word moved five frames** (571/571), and
run33's word 571 → **576**, totals 791/662 → **802/688**. East Indies
unmoved at 167/167.

**The repath throttle was a lifetime count.**
`GameDaemon::process_all` halves every player's `repaths` each frame and
clears it under three; nothing here did, so player 1's reached 5 and
stuck, pinning `resolve_unit_collision`'s throttle in its `≥ 4` arm where
`(o + collide) & 3` throws away three collisions in four. `1/2`'s
collision on 571 was one, so the repath — and the stagger draw at its
tail, `Random::get(0, 0xffff) % 9 + 1` into `pause` — was never spent.
PATHFINDER §8 had the decay in prose; the code had none (item 72 again).

**The collision block is 80,161 comparable field-frames** (from 77,211),
still zero disagreements, and player 0 recovers item 79's fall and passes
it — 574/577/579 → 687/700/703.

**Owed:** 40 (ORACLE.md's pin is 138,489), 42.

**Opener (Opus):** `take item 81 from @docs/QUEUE.md — run33's word parts
at 576 on a city founding, sixty draws against forty; run33's 576 with
rontrace-run33.log beside it.`

## The queue

In dependency order, headline-nearest first. Take the first unstarted one
unless something has made a different order obviously better, in which case
say so. Numbers are stable; the journal is indexed by them.

81. **The word's next parting, at 576 — a city founding.** Sixty draws
    against forty, in five repeats of one block:
    `ScenarioFuncSet::place_city_with_cost+0x68` calls
    `Leader::compute_sites` for two (`+0x4ac`, `+0x50a`), then `+0x7f`
    calls `Leader::found_cities+0x696` → **`Leader::make_stuff+0x221`,
    three draws**, which this spends never. The pair is modelled; the
    founding behind it is not. `make_stuff` whole was in the older
    backlog — this is where it is owed. run33's 576 with
    `rontrace-run33.log`; `docs/AI.md`, `docs/CITIES.md`.

74. **The AI's long-run economy** (was item 51, now the other way up).
    `1/9` arrives on **897** against the original's 1,297 and `1/10` is
    never trained: the roster pair is 268 missing + 400 extra, all this
    one unit, so the AI earns food faster. `CITY.gatherers` is in every
    dump and uncompared, and a `LEADERS` dump prints
    `resource_cap`/`income`/`rate` per good. Score it 268 + 400.

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

Older backlog, unchanged: the `LEADERDATA` and `CITY` widenings; a
`find_target` block; run7's order stream under the trace; a mounted
attacker; a caravan; `Leader::diplomacy`; `calc_gather` non-flat; and (77)
ANIM §3.2's `AGE3`/`AGE5` piece rows, which no capture has ever aged into,
with `get_unit_gpiece`'s fall-back walk.

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
