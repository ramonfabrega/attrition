# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story.

This file **deletes**. A finished item leaves it for the journal, which
carries every number that has ever been here — items keep their numbers for
that reason, and new ones continue the count. `docs_guard.rs` fails the
build if this file strikes an entry instead of deleting it, passes 200
lines, or lets the handoff pass 32.

## Where things stand

*2026-09-01, Opus — the scout sails, and East Indies' word **holds at
3608**.* Item 134 was three missing draws and turned out to be the AI's
whole sea half; it spends all three now and parts one draw later.

- **The caster is the scout, and run57 said so without a new capture** —
  §12's check 4 wanted a `UNITS=3` window over 3600–3640 and run57 is
  that game at that detail already. Blocks 3585–3609 carry the cast
  order, the eleven waypoints, the barge `1/14`, `inside_up 14`.
- **Five pieces for one frame**: §6's boarding, §7's island choice and
  `think_scout`'s tail, `Region.flags` from the dump's `REGIONS` block
  (`go_here`'s bit 1, a seam because no cell implies it),
  `invalid_loc`'s sea and air arms, and **`find_wpath`'s pull-back gate**
  — `00689375` walks the goal only for a unit that cannot board, and
  without it the word fell 3608 → 3585. With it the route is the dump's
  entry for entry and the cell chosen is the original's.
- **One draw is left** (item 135): the barge's new guy wraps its clock
  here and never in the original, where the gull and every trained unit
  do. TRANSPORT §13 names the capture.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w3608 of 24,000 · GreatLakes w1802 of 24,000

**Steering ran 2026-09-01 (Fable); the clock resets.** run57's rescope
ratified, its ratchet declined, four conventions into `CLAUDE.md`,
`tools/guard.sh` the new reflex. Journal has the story; 135 stands.

**Opener (Opus):** `East Indies' word is 3608 and item 135 is the one
draw holding it: the transport barge's own guy wraps its animation clock
on the frame it is born and the original's never does, on that frame or
any later. docs/TRANSPORT.md §13 has the evidence and the capture.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

135. **The barge's own guy wraps, and nobody else's new one doesn't.**
    East Indies' word is 3608 and this is the whole of what holds it: one
    `Guy::set_anim+0x97a < Guy::inc_time+0x271` this crate spends for the
    transport born that frame and the original spends on no frame at all.
    The dock's gull (3579) and run33's trained citizen (99) both wrap and
    both agree here, and nothing read separates them — `squad_size` is a
    literal 1 for every type, `Guy::init_real` writes `end_time` 0.
    `docs/TRANSPORT.md` §13 names the capture: a `GUYS=4` window over
    3606–3612 prints every guy's clock.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `Leader::produce_unit@006cb9e0` and four others. `World::danger` here
    indexes it by *region*, as `ai_build`, `ai_make`, `ai_research` and
    `ai_units` were written to; both answer 0 while nothing writes it, so
    nothing has told them apart. `think_civilian_transport`'s score is
    `dist × max(1, danger)` and picks the original's cell with the term at
    1, which is evidence the grid was empty there, not that it never
    matters. Find the writer.

82. **run40's human files one slot under good 2.** `get_good@0063bd50`'s
    table at `0063bd84` cannot produce it;
    `Leader::plan_strategy@006b9620` line 1137 assigns the whole array from
    `City::count_gather_slots` and raises the high-water to match, and that
    second writer is unread. Beside it: the original holds **0** in goods
    3–5 where this crate holds **100** — inert while none is available, so
    a loader question. The run40 diff asserts both as they stand.

120. **Great Lakes' word at 1802, and a bird's flight physics.** One draw
    at `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 <
    Unit::do_job+0xd7`, reached **once** in run33's 1,851 frames and still
    the boundary over run53's 24,000. It is the last thing between this map
    and a word past its own capture. `docs/SYNC.md` §3.9 has the bird;
    `do_air_physics` is named there as unread and is the one arm of it no
    run has forced.

103. **The wood machine's record parts at 1,686, on a clock.** After 26,094
    agreeing fields the human's `0/2` holds a wait of 445 where the
    original's holds 480, tile and phase right. ORDERS §6.4.

105/45. **Gaia's positions — East Indies' half is what is left.** run39:
    191,876 of 192,504, first bad **1658**; run33 is exact at 74,040.
    `Sim::reseat_animal` still corrects them where a dump carries clocks
    (SYNC §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (item 110's day went to one, item 133's frame 3579 to
    another). **16 of the 61** `rng.roll()`/`rng.get()` calls in
    `crates/sim` have no `self.mark(` in the six lines above them — some
    are marked by their caller, and the point is that nothing says which.
    Mark or explain each, then make the list a guard.

124. **The loop flag is per animation file** (ANIM §3.3): by slot it is
    not a constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a
    non-looping file and 38 a looping one. Carry it beside
    `Art::piece_lengths`.

87. **The widening ledger.** Items 74, 83, 69, 113 and 123 were closed or
    sharpened by fields the parser had and nothing compared. Make the
    backlog a number: per dumped record, the fields `rondata::diff` parses
    and never compares. run42 adds the per-frame `LEADERDATA` to that pool.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000` at
    the end of 202 or 203, yet its Scout's `mylos` moves 4 → 6; ours moves
    at 202 too, so what is owed is the *cache*.

89. **The instrument's last guard.** (a) and (b) are built (2026-08-31);
    what is left is (c): run33's floor pins totals mostly past its
    parting — fallen three times (109, 69, 113).

72. **A document and its code disagreeing is a diff waiting to be run.**
    The `name@00xxxxxx` half is a guard now (`docs_guard`, 2026-08-31).
    Left: every `+0xNN` a document pins, checked against its module, and
    every "halved"/"every frame"/"cleared" verb.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617). Print it, pin
    it as a floor, put it in each Coverage section.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through it too (COLLISION §3,
    §7, GOODY §1), and `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled;
    run33's scout goes 65 → 64 on 96 where `get_open_slot` says 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries it on `(52, 22)` and this does not; `army`'s
    muster search reads it (ARMY §13). Find the *tile* mask's writer.

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker — the keystroke
    arrived as the chat key; ORACLE runs 51/52 name the next steps (bind a
    plain letter, `osascript keystroke`, confirm by tooltip, then
    `FORM_E_RIGHT` and `form 3` in the dump). With it COLLISION §9.

116. **The one `SITE` slot still wrong, and the rule that is not it.** AI
    §18: a 5×5 slide keeps its centre where the original leaves it, so
    `blocked_town` refuses a cell `site_clear` allows. `blocked_site`'s
    fog-majority `0x24` (CITIES §11) was run once and moves **no** number,
    so the slide wants a second reading, not that rule.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, and `was_seen`'s `reg_forts`
    arm — all inert until a capture has a Temple or a Fort; the bonus-preq
    detail is in that section. (b) AI §2.1's `check_explore` answers the
    whole region grid — `explored` is 900 here against the dump's 36 and
    19 on every frame of both windows.

The road residue, both in ROADS §7.1: (57) the node counts — 1,046 v
1,043 and 1,460 v 1,870 — re-read on run43's own before-grid (ORACLE
run43); (58) the height loader's mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`, so it is the **Military** level, not ARMY's glossary's "current
    age". `army.rs:769,1365` read `tech[w].ages`, so every `age < 2/3/5`
    gate in the muster caps and `find_target` fires on the wrong counter.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — run `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; the `CITY` widening;
a `find_target` block; run7's order stream; a mounted attacker; a caravan;
`Leader::diplomacy`; `calc_gather` non-flat; (77) ANIM §3.2's piece rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`.
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** "Where things stand", the item, then its document.
- **Run the diff suite with `--release`.** run53's 24,000 frames tripled
  `cargo test -p rondata` in debug — 114 s to 285 s — where `--release` is
  92 s including the build. The five-minute wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a subagent brief, an agent definition, or a memory hook.
