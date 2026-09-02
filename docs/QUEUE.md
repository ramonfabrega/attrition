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

*2026-09-02, Opus — three of the six resources were available from frame 0,
and East Indies' word is still **5376**.* Item 154's first half landed and
its second is measured: the AI now takes Coinage on 5177 as the original
does, and is forty-six timber short of the Market it sites on 5376.

- **`Holdings::available` had no writer.** Knowledge and Metal need the
  Classical Age and Oil the Industrial, so an Ancient knowledge price is
  charged as food at 3/2 — the whole redirect layer `cost.rs` carries had
  never fired in a played game. `Sim::sync_goods_available` writes it from
  `type_avail` now (ECONOMY, "Availability, and where a price lands
  instead").
- **run58's queue tail is closed**: both sides start the AI's Coinage on
  **5177**, and what is left is frame 5201, where the original clears both
  jobs and prints `BUILDDATA` twice, truncated — the run was quitting.
- **The word did not move, and the reason is a number.** The AI reaches
  `economic.bhs` case 15 and calls `place_building_with_cost(Market)` on
  5376 as the original does; `affordable` answers 0. A Market is eighty
  timber and the AI holds **34**, accruing ~3 per 25 frames. No capture on
  disk carries a resource level past frame 800, or any on this map.
- **`STARTING_GOODS` arrives with the age, not at init** —
  `Leader::gain_tech@006dcb60`'s six-good loop. run40's booked 240 rows,
  now explained; item 156.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w5376 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `East Indies' word is 5376 and item 154 still holds it:
the AI is 46 timber short of the Market it sites there. Widen the CITY
record first — run58 dumps it every frame and nothing compares it.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

154. **The AI is 46 timber short of the Market it sites on 5376 — the
    headline.** Coinage lands on 5177 now, `economic.bhs` case 23 jumps to
    case 15 on a sea map, and this crate calls
    `place_building_with_cost(who, "Market", my_capital)` on 5376 as the
    original does — `affordable` 0 against eighty timber, 34 held. A
    resource level, and no capture past frame 800 carries one. **(a)**
    Widen the `CITY` record: run58 dumps it every frame, `rondata` parses
    four of its forty fields and only for the initial dump, and
    `gatherers`, `busy`, `free`, `filled`, `space[scan]`, `ter[scan]` are
    the rate's own inputs (item 87). **(b)** Only for what that cannot
    answer, a `LEADERS=9` census window on East Indies at 5150–5400
    (`censuswindow.sh`, the `run40_s_census_…` instrument).

146. **The other nine national arms of `train_time`.** `006508c0` runs a
    fixed order after the ramp — handicap, The President, Mongol stable,
    Japanese barracks and carrier, Chinese citizen, then British, French,
    German, Roman — and only the British is built (PRODUCTION, "The tail's
    first caller"). The rest are inert in every capture: each wants a
    nation the captures play, or a run that falsifies it. The Chinese
    predicate is the one the decompiler mangles (`extraout_ECX[0xae] & 8`).
    Behind them: `TROOPS_FASTER`, the speed upgrades, the rares, Monarchy,
    Socialism, the unit wonders.

103. **A woodcutter's clock at 1,686.** After 26,094 agreeing fields the
    human's `0/2` holds a wait of 445 where the original's holds 480
    (ORDERS §6.4).

142. **`World::tregion` is not `get_tregion`, and its callers are
    unaudited.** Four items were a gate asking the wrong one of the two
    (138, 140, 145 — PATHFINDER §15; 147 the other way, §16). `path`'s
    four are done; eleven elsewhere are not. Grep each, then guard.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `Leader::produce_unit@006cb9e0` and four others; `World::danger` here
    indexes it by *region*, and both answer 0 while nothing writes it.

82. **run40's human files one slot under good 2.** `get_good@0063bd50`'s
    table at `0063bd84` cannot produce it; `Leader::plan_strategy@006b9620`
    line 1137 assigns the whole array from `City::count_gather_slots` and
    raises the high-water to match — that second writer is unread.

120. **Great Lakes' word at 1802, and a bird's flight physics.** One draw
    at `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 <
    Unit::do_job+0xd7`, reached **once** in run33's 1,851 frames and still
    the boundary over run53's 24,000 — SYNC §3.9's unforced arm.

105/45. **Gaia's positions — East Indies' half is what is left.** run39:
    191,876 of 192,504, first bad **1658**; run33 exact at 74,040 (SYNC
    §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (110's day, 133's 3579). **16 of the 61** `rng.roll`/
    `rng.get` calls have no `self.mark(`. Mark each, then guard the list.

124. **The loop flag is per animation file** (ANIM §3.3): by slot it is no
    constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a non-looping and
    38 a looping file. Carry it on `Art`.

153. **The `TRIBE` record, whole.** `Tribe::log_data@006f0d70` prints
    `graft[352]`, `barbarian`, `build_continent`, `people` and
    `text_substitute` in every `DUMP_ALL` dump and this crate parses none:
    the graft table is identity and `barbarian` false by assumption (TECH).

The ledgers, each a number nothing yet counts: (87) **the widening
ledger** — items 74, 83, 69, 113, 123, 144 and now 154 were closed or
sharpened by fields the parser had and nothing compared, so count, per
record *and* per capture, the fields `rondata::diff` parses and never
compares; (88) **the blind list** — `report.py … blind docs/` lists the
cited functions no traced run has entered (101 of 617), pin it as a floor
and put it in each Coverage section; (72) every `+0xNN` a document pins,
checked against its module (the `name@00xxxxxx` half is `docs_guard`
already); (89) the instrument's last guard, (c) alone; (35) **`mylos` as a
cache** — VISION §7: player 1 carries no `0x4000000` at the end of 202 or
203, yet its Scout's `mylos` moves 4 → 6 and ours moves at 202.

Three fields nothing here writes: (48) **the object chain, whole** —
`collide.rs` chains units only where the original threads buildings and
goodies through it too (COLLISION §3, §7, GOODY §1), and `down`/`down_who`
go uncompared; (73) **`UnitData::group`** — no unit holds the back-pointer,
so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled and run33's scout
goes 65 → 64 on 96; (56) **the cell's `BUILDING` bit** — run13's frame-95
world carries it on `(52, 22)` and this does not, and `army`'s muster
search reads it (ARMY §13); find the *tile* mask's writer.

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker: the keystroke
    arrived as the chat key; ORACLE 51/52 name the next steps.

116. **The one `SITE` slot still wrong.** AI §18: a 5×5 slide keeps its
    centre where the original leaves it, so `blocked_town` refuses a cell
    `site_clear` allows. `blocked_site`'s fog-majority `0x24` (CITIES §11)
    moves **no** number.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap and `was_seen`'s `reg_forts`
    arm — inert until a capture has one. (b) AI §2.1's `check_explore`
    answers the whole region grid — 900 against 36 and 19.

The road residue, both ROADS §7.1: (57) the node counts — 1,046 v 1,043
and 1,460 v 1,870 — on run43's own before-grid; (58) the height loader's
mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8` — **Military**, not ARMY's "current age". `army.rs:769,1365`
    read `tech[w].ages`, so the muster caps fire on the wrong counter.

155. **The other two writers of `rare` and `good_obj`.** Both are in every
    `UNIT` record and nothing compares them (item 87). run58's `1/14` takes
    `rare −1` on 4872 from `Unit::think@005f6e40:179`'s rare-collector arm
    — a **packed** AI `is_rare_collector` on `idle == 1`/32 takes
    `Unit::do_gather@005fce20`, which also bumps `idle` and tries
    `unpack_merchant(4)` — and `rare 6, good_obj 1` on 4992 from the gather
    job (`Unit::do_job@00617a10` → `Unit::do_gather@005ef2a0`).
    ORDERS §6.10; land both and the two fields become a diff.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930`
    zeroes all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose
    prerequisite is the tech just gained. So the original holds 0
    knowledge, metal and oil through the Ancient age where this crate holds
    100 — `run40_s_census_…`'s 240 booked rows, inert. Unread: where food,
    timber and wealth are paid. COSTS, "What is not established".

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; a `find_target`
block; run7's order stream; a mounted attacker; a caravan;
`Leader::diplomacy`; `calc_gather` non-flat; (77) ANIM §3.2's rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`. **The 200-line
  bound is a budget: a new item is paid for by compressing old ones.**
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** "Where things stand", the item, then its document.
- **Run the diff suite with `--release`** — 92 s against debug's 285 s
  since run53's 24,000 frames. A long wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a subagent brief, an agent definition, or a memory hook.
