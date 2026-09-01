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

*2026-09-01, Opus — the AI has been British all along, and East Indies'
word is **4462**.* Item 144 was booked as "a late decision or a slow
counter" and was neither; the score moved one frame, onto the thing behind
it.

- **`ObjectData::train_time`'s British arm** (PRODUCTION, "The tail's
  first caller"): `has_tribe_bonus(0xb)` around three tests — sea domain,
  `is(0xaa)` Bowmen, `is(0x119)` anti-air — each `t*100/(K+100)`.
  `BRITISH_SHIP_SPEED` and `BRITISH_AA_SPEED` are 33, the archer one **0**.
  Player 1 is `tribe 11` and the Dock builds a **Fisherman**.
- **It parted on the *target*.** `1/2010` queues on 4376 on both sides and
  every hundredth agrees to 4460; theirs caps at 8481, ours ran to 11,280,
  and 11280 × 100/133 is 8481. The second Fisherman checks it again on the
  ramp's next step: 12,030 → 9,045.
- **It was found by moving a loop.** The queue record was parsed on every
  capture and compared on run39 alone — 2,600 frames short of the AI's
  first ship. `diff::compare` owns it now: run58 went 0 → **109,435** queue
  fields, 238 wrong and all past the word.
- run58: no unit off point before the word, 178,326 building fields exact,
  449,279 collision field-frames, four wrong and none early. Steering last
  ran 2026-09-01 (Fable).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w4462 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `East Indies' word is 4462 and item 145 holds it:
frame 4462 is the new Fisherman's first think, 55 draws opening at
`Unit::think_fish+0x27a` where this crate makes 2. Nothing below
ORDERS §5's `think_fish` has ever been built.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

145. **`Unit::think_fish`, which does not exist here.** Frame 4462 is the
    AI's new Fisherman's first think: **55** draws opening at
    `Unit::think_fish+0x27a` (`5f4eda`) against this crate's 2. ORDERS §5
    names it as the head of `think`'s tail and nothing below it was ever
    built. The ship that reaches it is now born on the original's frame.

146. **The other nine national arms of `train_time`.** `006508c0` runs a
    fixed order after the ramp — handicap, The President, Mongol stable,
    Japanese barracks and carrier, Chinese citizen, then British, French,
    German, Roman — and only the British is built (PRODUCTION, "The tail's
    first caller"). The rest are inert in every capture, so each wants a
    nation the captures play or a reading naming the run that falsifies it;
    the Chinese predicate is the one the decompiler mangles
    (`extraout_ECX[0xae] & 8`) and the listing settles it. Behind them:
    `TROOPS_FASTER`, the three speed-upgrade counts, the rares, Monarchy,
    Socialism and the unit wonders.

103. **A woodcutter's clock at 1,686.** The same machine one map down:
    after 26,094 agreeing fields the human's `0/2` holds a wait of 445
    where the original's holds 480, tile and phase right. ORDERS §6.4.

142. **`World::tregion` is not `get_tregion`, and its callers are
    unaudited.** Two items in one day were a gate asking `region_of` where
    the original asks `get_tregion` (138, 140). `tregion` is
    `region_of(cell_of_tile)`; the real one is `tregion_alt`. Nineteen call
    sites across `orders`, `scout`, `transport`, `roads`, `army`, `place`,
    `group` and `path` are unchecked. Grep each; then try for a guard.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `Leader::produce_unit@006cb9e0` and four others. `World::danger` here
    indexes it by *region*, as `ai_build`, `ai_make`, `ai_research` and
    `ai_units` were written to; both answer 0 while nothing writes it, so
    nothing has told them apart. `think_civilian_transport`'s score is
    `dist × max(1, danger)` and picks the original's cell twice with the
    term at 1 — the grid is empty there, not irrelevant.

82. **run40's human files one slot under good 2.** `get_good@0063bd50`'s
    table at `0063bd84` cannot produce it; `Leader::plan_strategy@006b9620`
    line 1137 assigns the whole array from `City::count_gather_slots` and
    raises the high-water to match, and that second writer is unread.
    Beside it: the original holds **0** in goods 3–5 where this crate holds
    **100** — inert while none is available, so a loader question.

120. **Great Lakes' word at 1802, and a bird's flight physics.** One draw
    at `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 <
    Unit::do_job+0xd7`, reached **once** in run33's 1,851 frames and still
    the boundary over run53's 24,000 — the last thing between this map and
    a word past its own capture. SYNC §3.9 names `do_air_physics` as the
    one arm of it no run has forced.

105/45. **Gaia's positions — East Indies' half is what is left.** run39:
    191,876 of 192,504, first bad **1658**; run33 is exact at 74,040.
    `Sim::reseat_animal` still corrects them where a dump has clocks
    (SYNC §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (110's day, 133's frame 3579). **16 of the 61**
    `rng.roll()`/`rng.get()` calls in `crates/sim` have no `self.mark(`
    above them, and nothing says which are marked by a caller. Mark or
    explain each, then make the list a guard.

124. **The loop flag is per animation file** (ANIM §3.3): by slot it is no
    constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a non-looping file
    and 38 a looping one. Carry it beside `Art::piece_lengths`.

87. **The widening ledger.** Items 74, 83, 69, 113, 123 and now 144 were
    closed or sharpened by fields the parser had and nothing compared. Make
    it a number: per record, the fields `rondata::diff` parses and never
    compares — and, since 144, per *capture* as well as per field.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000` at
    the end of 202 or 203, yet its Scout's `mylos` moves 4 → 6; ours moves
    at 202 too, so what is owed is the *cache*.

89. **The instrument's last guard.** (a) and (b) are built (2026-08-31);
    left is (c): run33's floor pins totals mostly past its parting.

72. **A document and its code disagreeing is a diff waiting to be run.**
    The `name@00xxxxxx` half is a guard (`docs_guard`). Left: every `+0xNN`
    a document pins, checked against its module, and every verb.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617). Pin it as a
    floor and put it in each Coverage section.

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
    arrived as the chat key; ORACLE runs 51/52 name the next steps. With
    it COLLISION §9.

116. **The one `SITE` slot still wrong, and the rule that is not it.** AI
    §18: a 5×5 slide keeps its centre where the original leaves it, so
    `blocked_town` refuses a cell `site_clear` allows. `blocked_site`'s
    fog-majority `0x24` (CITIES §11) moves **no** number, so the slide
    wants a second reading, not that rule.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap and `was_seen`'s `reg_forts`
    arm — inert until a capture has a Temple or a Fort. (b) AI §2.1's
    `check_explore` answers the whole region grid — 900 here against the
    dump's 36 and 19 on every frame of both windows.

The road residue, both in ROADS §7.1: (57) the node counts — 1,046 v
1,043 and 1,460 v 1,870 — re-read on run43's own before-grid (ORACLE
run43); (58) the height loader's mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`, so it is **Military**, not ARMY's "current age".
    `army.rs:769,1365` read `tech[w].ages`, so every `age < 2/3/5` gate in
    the muster caps and `find_target` fires on the wrong counter.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; the `CITY`
widening; a `find_target` block; run7's order stream; a mounted attacker; a
caravan; `Leader::diplomacy`; `calc_gather` non-flat; (77) ANIM §3.2's rows.

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
