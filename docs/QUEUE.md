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

*2026-09-01, Opus — a camp's site is scored by what it would gather.* East
Indies' word moved **2176 → 2665**.

- **Item 126 is closed, and it was one number in the wrong place.**
  `Leader::produce_building` scores a woodcutter's camp site by the **cube**
  of `blocked_site`'s own out-parameter — `calc_gather`'s count, the same
  one `max_gatherers` reads — and refuses a site under three. This crate
  counted the forest tiles in a **one-tile ring**, which is zero at every
  site a camp can stand on: the score went to zero, the gate refused every
  candidate, and **the AI placed no camp at all after frame 0**. Frame 0 hid
  it — it skips the gate, and a uniformly-zero score takes the spiral's
  *last* candidate. AI §19, CITIES §2.6.7.
- **The out-parameter is modelled whole now** (`blocked_site_slots`,
  `gather_verdict`): a non-flat gather type with nothing under it is refused
  — `NoForest`, `NoMountain`, `NoResources`, the two `Taken` verdicts — a
  rule `blocked_location` had never carried here. Seven flat-world unit
  tests failed on it at once, each standing a camp on a treeless map; they
  plant trees now (`Sim::plant_camp_forest`).
- **run56's frame 2176 is the diff.** The AI reaches it at script step 13 —
  `place_woodcutter`'s `num_cities > 1` arm — and places `o 2009` at tile
  (198, 190) on the original's frame, with its 192 shuffle draws. run56's
  gather comparison is **1,048,118 fields and nothing but the quit's four**.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w2665 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `The headline is East Indies' word at 2665 and item 127
is its divergence: this simulation's scout thinks a second time on 2665
where the original thought once, on 2664, and stopped. run54 and run56 are
the captures; docs/SCOUT.md is the document.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

127. **The scout thinks twice, and the headline's own divergence at 2665.**
    On 2664 both take a whole `Unit::think_scout` — ten `+0x436`, six
    `+0x458`, one `+0x941` — and agree on every draw of the frame. On 2665
    the original takes nine draws and this simulation takes **thirty**: the
    first three agree (a step's `set_anim`, the new camp's
    `Unit::do_non_flat_gather+0x54b`, an idle `set_anim`), then this one
    idles **one unit more** and runs a *second* whole `think_scout` — ten
    `+0x436`, **ten** `+0x458`, one `+0x941` — that the original does not.
    The `+0x458` count is the tell and it is `frame % 8`: SCOUT §6 skips the
    phase draw when `ring / 4 + frame % 8 == 0`, which is rings 1–3 on 2664
    (`2664 % 8 == 0`) and no ring at all on 2665. So the question is not the
    walk, it is **why the unit is still idle**: the original's scout took its
    order on 2664 and left `do_idle`, and from there thinks every 32 frames
    exactly (2688, 2720, 2752, …, 2976). Start at SCOUT §9's order and what
    `do_idle` does with it. run54 is the word's capture, run56 the
    full-detail sibling.

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
