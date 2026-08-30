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

*2026-08-30, the third steering pass (Fable).*

**The headline is the pair, lower map first.** East Indies: run39 ticks
**167**, orders 167, of 1,850. Great Lakes: run10 ticks **572**, orders
776, of 1,772; run33's word parts at **780**, totals 951/838, of 1,850.
Since the 08-28 steer: twelve items, Great Lakes 200 → 572 and the word
284 → 780, the roster 468 + 0 → 268 + 0, the second map stood up — sixteen
Opus sessions, no subagents, every step booked. The tree is green with the
install wired in: 637 sim, 143 rondata, no `skipping`.

**What this pass changed** (DECISIONS 25): the docs guard measures `## `
sections at 16 KB, not files at 60 — eleven pinned, the rest may grow.
The run10 floor was stale (orders 576, player 0 @ 687) and is 776 / 802.
The fifteen "owed" audits carried no markers; the ratification surface is
the blind list (item 88). Steers run every twenty items or a stalled
headline. Both captures are **two minutes of game** at 15 frames a second;
the next length is item 91, not yet due.

**Owed:** ORACLE.md's run40/run41 rows — promote its `### run` headings to
`## ` and the 122 KB pinned section dissolves.

**Opener (Opus):** `take item 69 from @docs/QUEUE.md — East Indies parts at
168: 1/4 holds two orders on the original's frame where this holds one.
run39's 160–210, rontrace-run39.log beside them.`

## The queue

In dependency order, headline-nearest first — the headline is the lower
map. Take the first unstarted one unless a different order is obviously
better, and say so. Numbers are stable; the journal is indexed by them.

69. **The second map's own first divergence, at 168.** East Indies is
    167/167 and what parts it is an order-list **length**: `1/4` holds two
    orders on the original's 168 where this holds one, its position parting
    the same frame; `1/5` at 186 and `1/3` at 202 the same. run39's
    160–210, `rontrace-run39.log` beside them.

84. **The word at 780, four frames past the city.** Eleven draws against
    eight; ours opens `GameAccess::rnd+0x20 < Unit::do_job+0x67` where the
    original has `Unit::do_non_flat_gather+0x54b`. A citizen is on a job
    where the original has it gathering.

87. **The widening ledger.** Items 74 and 83 were closed by comparing
    dumped fields nobody had compared, with no reading. Make the backlog a
    number: `rondata::diff` prints, per dumped record, the fields it parses
    and does not compare, and pins the count as a floor that may only
    fall. `ANIMALDATA` (item 45) and `down`/`down_who` (item 48) are the
    first rows.

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

89. **Two guards for the instrument itself.** (a) The handoff's headline
    numbers equal `rondata::diff`'s pinned floors — the run10 floor sat
    two items stale while the queue carried the right number. (b) Every
    `name@00xxxxxx` a document cites names that function at that address
    in the export's `INDEX.tsv`; skips when the export is absent, like the
    install-backed tests. Both made to fail first.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80 and 74 — four in a row. Both halves: every
    `name@00xxxxxx` and `+0xNN` a document pins, checked across documents
    and against the module implementing it, and every "halved"/"every
    frame"/"cleared" verb about a field this crate owns.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617); intersected
    with what `crates/sim` implements, that is every reading-only claim
    the harness rests on, and what a steer ratifies. Print it, pin its
    size as a floor, and put it in each document's Coverage section.

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

90. **The capture queue.** A capture is fourteen unattended minutes — the
    screen for thirty seconds of lobby, then nothing a session needs — so
    it runs *beside* a session, not between them. A scenario file
    (`longtrace.sh`'s inputs) and a script that runs the lines back to
    back; items 23, 45 and 57 are the first three.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION
    §9's pause capture, two units ordered head-on.

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so a wrong animal is invisible until an
    untraced run. Widen to `ANIMALDATA`: 70,960 uncompared.

The road residue, all in ROADS §7.1 and SYNC §6: (57) the terraform,
`terraform_for_building@00875210` after `place_roads`, oracle run13's
`FRAME 100` against run32's 104; (58) the height loader's mean in `f32`,
three of run12's tiles; (60) the frame's second search, 1,460 nodes
against 1,870; (62) the standing swap at frame 99, the first draw
*sequence* difference, costing no word.

91. **The final scenario, not a final dump.** One 24,000-frame game per
    map: the **trace** whole (`cover=1`, no window — cheap; the word over
    the whole game is the long headline) and the dump in windows
    re-captured *on demand*, since the game is reproducible (`samegame.py`)
    and a new window is a five-minute re-run. A full dump is
    24,000 × 2.4 s. Due when a map's word matches its 1,850; before that
    the long word parts where the short one does.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — run `zsh tools/fuzz/run.sh 424242
1000 1300` then `report.py … blind docs/` before deleting it; the `CITY`
widening; a `find_target` block; run7's order stream under the trace; a
mounted attacker; a caravan; `Leader::diplomacy`; `calc_gather` non-flat;
and (77) ANIM §3.2's `AGE3`/`AGE5` piece rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`.
- **Start of session:** "Where things stand", the item, then its document.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both.
- **Never** quote this file or the journal into `CLAUDE.md`, a subagent
  brief, an agent definition, or a memory hook.
