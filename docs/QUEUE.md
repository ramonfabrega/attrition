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

*2026-08-30, after item 95's successor (Opus).*

**The headline moved further in one item than it ever has.** East Indies:
run39's word parts at **201**, up from 91, with its first 64 frames still
**64/64** draw for draw. Great Lakes holds where it can be seen: run10
ticks **572**, orders **776**, both players' first divergence 802 and
573, and twelve of its thirteen units unchanged to the frame; run33's
word at 780 and its sequence at 99. What fell is coverage, and all of it
is `1/9` parting at 1320 rather than 1377: collision rows 93,398 →
**91,210**, angle rows 35,942 → **35,188**, run33's weak totals 938/841 →
**943/827** — draws identical, count *and* sequence, to frame **1128**.
Tree green: 644 sim, 147 rondata.

**It was a collision, not a wander.** The booked "`8/0`'s near branch" was
never taken — both sides take the far arm — and what differed was `8/2`,
blocked twenty frames earlier and walked on by this crate.
`Unit::resolve_unit_collision`'s **first** statement is
`SubObjectData::is_animal` (vftable offset 48, from the PDB's
`LF_ONEMETHOD` list; the map folds both overrides onto stubs), and when
it answers the body is the `QUEUE_NEW` clear — an animal takes none of
`docs/COLLISION.md` §6's six steps. §6 step 0; `docs/SYNC.md` §3.14.
Pinned against **16 of 16** dropped animal walks on both maps, made to
fail two ways.

**Opener (Opus):** `take item 97 from @docs/QUEUE.md — East Indies' word
now parts at 201 on a citizen's job draw; docs/SYNC.md §3.14 has how the
last one was found.`

## The queue

In dependency order, headline-nearest first — the headline is the lower
map. Take the first unstarted one unless a different order is obviously
better, and say so. Numbers are stable; the journal is indexed by them.

97. **East Indies' word at 201, and it is item 84 on both maps now.** Ours
    spends 38 draws where the original spends 36, and the first difference
    is at 0: ours opens `GameAccess::rnd+0x20 < Unit::do_job+0x67` — a
    citizen on a job, item 84's own signature — where the original opens
    `Guy::set_anim+0x97a < Guy::inc_time+0x271`, the standing-turn residue
    run33 shows on 99, 205 and 319. Take the two together; the second is
    the one no capture has ever explained.

69. **East Indies' order-list length at 168.** `1/4` holds two orders on
    the original's 168 where this holds one; `1/5` at 186 and `1/3` at
    202 the same. Now *behind* the word by 33 frames rather than ahead of
    it, so it is real — but re-measure before booking: it was read off a
    simulation four items ago.

84. **The word at 780, four frames past the city.** Eleven draws against
    eight, the same `Unit::do_job+0x67` opening 97 has. Fold into it.

87. **The widening ledger.** Items 74 and 83 were closed by comparing
    dumped fields nobody had compared. Make the backlog a number:
    `rondata::diff` prints, per dumped record, the fields it parses and
    does not compare, pinned as a floor that may only fall.

85. **`gather_slots`, and a slot `get_good` cannot produce.** 120 of the
    run40 diff's 480 good-frames. `Build::init` surveys a camp against its
    still-empty `gather_from`, so a harness camp activates with zero; and
    run40's *human* files one under good 2, where `get_good@0063bd50`'s
    table at `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5.

86. **The science discount's purchase side.** `Sim::tech_price` passes
    `Modifiers::default()`, so a tech is charged undiscounted, and
    `Build::refund_cost` inverts an expression the buy side never applies.
    Zero on every traced purchase. `calc_science_discount@006da630`.

82. **A hundred knowledge, oil and wealth nobody gave anybody.** run40:
    the original holds **0** in goods 3, 4 and 5 for both players and this
    crate **100** — inert while none is available, so a loader question.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000`
    at the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6.
    Ours moves at 202 too, so what is owed is the *cache*.

89. **Two guards for the instrument itself.** (a) The handoff's headline
    numbers equal `rondata::diff`'s pinned floors — one sat two items
    stale. (b) Every `name@00xxxxxx` a document cites names that function
    in `INDEX.tsv`. Both made to fail first.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80, 74 and now **36 twice over** — `ORDERS.md` §4.5 had
    both arrival arms right while the code merged them. Every
    `name@00xxxxxx` and `+0xNN` a document pins, checked against the module
    implementing it, and every "halved"/"every frame"/"cleared" verb here.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617); intersected
    with `crates/sim`, that is every reading-only claim here. Print it,
    pin its size as a floor, put it in each Coverage section.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list
    (COLLISION §3, §7), and the dump's `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit here holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) and its six
    writers of `−1` are unmodelled. The pool slot is the hard half: run33's
    scout goes 65 → 64 on 96 where `get_open_slot` (§3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does
    not; `army`'s muster search reads it (ARMY §13). Find the writer —
    not `World::set_building_at`, which writes the *tile* mask.

90. **The capture queue.** A capture is fourteen unattended minutes — the
    screen for thirty seconds of lobby, then nothing a session needs — so
    it runs *beside* a session. A scenario file (`longtrace.sh`'s inputs)
    and a script running the lines back to back; 23, 45, 57 and 96 first.

96. **A turning vehicle or ship, which no capture has.** `guy_flags & 8`
    is set for 273 of the install's 1,359 unit pieces and for none of the
    eight any traced guy carries, so `Guy::do_turn@005d97a0:15`'s
    `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` override of the standing walk is
    unmodelled and unfalsifiable (ANIM §4.6). *Capture, owed:* a unit whose
    piece has one, turning in place, with `UNITS=3`.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION
    §9's pause capture, two units ordered head-on.

45. **Gaia's animals, and it is worth more than it looked.**
    `Sim::reseat_animal` puts them back from every traced dump (SYNC §4.2),
    so `run_traced` reported **no** divergence for `8/2` on any of the ten
    frames item 95 had its position wrong — and those ten frames cost the
    word twenty-two. Widen to `ANIMALDATA`: 70,960 uncompared.

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
