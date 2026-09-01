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

*2026-09-01, Opus — the dock landed, and East Indies' word went **3021 →
3435**.* Item 130 closed on the arm §20 named, for a reason §20 had
backwards, and it took two more defects with it.

- **The slide is not what places the dock.** `produce_building`'s
  `is(0x1b0)` arm slides ±2 tiles, and the original's dock sits on the
  plain centre of cell `(57, 54)`, which no slide can reach. What it does
  is make the spiral *accept* index 117 instead of 129, which moves the
  stride-of-three's phase from `0` to `2 (mod 3)` and lands the walk on
  143. run56's buildings: **92,626 fields, zero wrong**. AI §21.
- **Two more, each first reached by the dock's new cell.** All three
  `find_*path` pull-backs applied `sin_component`'s own sign fold twice,
  walking the goal *away* from the start and never terminating — no
  capture had run that body twice (PATHFINDER §13, listing-settled, with
  `find_tpath`/`find_upath`'s steps `0x60`/`0x18`). And
  `find_nearby_spot` had no terrain test, so a citizen could stand in the
  sea: the swarm ring took `k = +1` where the original takes `−1` (ORDERS
  §10).
- **run57 is a test now, and it names the next item.** 4,000 frames, both
  position records: **130,326 building fields, two wrong** — `1/2011` on
  3177 one cell east, `1/2012` on 3977 one tile — and **322,683 collision
  fields, none wrong**, nothing leaving the original's point before 3177.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w3435 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `The headline is East Indies' word at 3435 and item 131
is what holds it: the AI's building 1/2011 goes up on frame 3177 one cell
east of the original's. run57's diff is the oracle and docs/AI.md 21 is the
method that closed its predecessor.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

131. **`1/2011`, one cell east, on frame 3177.** run57's own residue and
    the whole of East Indies' word at 3435: the AI's building `o 2011`
    goes up at `x 35712` where the original's is at `34944`, exactly one
    cell, and every unit that leaves the original's point in the capture
    leaves it at or after that frame. Same shape as item 130 and the same
    oracle — a dumped position with no draw of its own — so the method is
    §21's: instrument `produce_building`'s walk for this call, find the
    accepted set and the stride's phase, and read the arm that differs.
    `1/2012` on 3977 is one *tile* in `x`, a jitter or a slide rather than
    a cell, and is probably a second thing.

132. **The word's own frame, 3435.** Three draws against two: this crate
    spends a `Farms::inc_time+0x1ae` the original does not. It is past
    run57's building seam at 3177, so it may be that item's consequence —
    take 131 first and re-measure. If it survives, FARMS is the document
    and the growth clock is the suspect.

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
    rather than short (item 110's day went to one). Grep `rng.roll()` in
    `crates/sim` for calls with no `mark` above them; mark or explain each.

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
