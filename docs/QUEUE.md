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

*2026-09-01, Opus — a deployed fishing boat stays on its fish, and East
Indies' word is **5376**.* Item 151 was `UnitData::calc_gather`, and it
moved 270 frames in two halves.

- **`calc_gather` is modelled** (`crates/sim/src/calc_gather.rs`, ORDERS
  §6.10): the two tile spirals, the surface-and-`0x200` predicate, the
  cell lookup under it, and the crowd count that turns "there is a good"
  into "and it is mine alone". A boat on its fish answers **yes** on 5106
  and stays, where this crate walked a 17 × 17 for 165 draws to four.
- **The scales are the mechanic.** The spirals are in **tiles** and
  `find_good_at` is in **cells**, so a boat on a fish answers ring **1**,
  not 0 — the fish marks its own tiles and the boat is on another. run58
  prints `good_obj 1` on 210 frames, `rare 6` beside it.
- **The claim test is the live chain, not the snapshot.** A deployed boat
  *is* its cell's `WData.down`, so the second Fisherman must refuse the
  fish the first sits on; `think_fish` read the start dump's `WData` and
  accepted 178 cells to 177. That is 5285, and one reader of item 48.
- **`unit_masks & 0x20` can never be set by either gameplay caller** —
  `*param_2` is written only past a `param_7` test both fail, and no
  `unit_masks` in run33/39/53/54/58 carries it. Great Lakes is unchanged
  at 1802; run58's own three numbers as they were.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w5376 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `East Indies' word is 5376 and item 154 holds it: the
AI sites a building there and this crate sites nothing — 37 draws to 33,
opening on Leader::produce_building+0x1805.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

154. **`Leader::produce_building`'s jitter on 5376 — the headline.** The
    AI draws 37 where this crate draws 33, opening on
    `Leader::produce_building+0x1805` against `Animal::think_farm_animal+
    0x142`: the original sites a building and this one never scores a
    site. AI §2.20 and §21; the first East Indies divergence not a boat's.
    run58's last 25 frames are the same family and now inside the word —
    the AI holds a job in `1/2005` from **5177** and this crate none;
    `RUN58_QUEUE_TAIL` asserts those rows as they stand.

146. **The other nine national arms of `train_time`.** `006508c0` runs a
    fixed order after the ramp — handicap, The President, Mongol stable,
    Japanese barracks and carrier, Chinese citizen, then British, French,
    German, Roman — and only the British is built (PRODUCTION, "The
    tail's first caller"). The rest are inert in every capture: each
    wants a nation the captures play, or a reading naming the run that
    falsifies it. The Chinese predicate is the one the decompiler mangles
    (`extraout_ECX[0xae] & 8`); the listing settles it. Behind them:
    `TROOPS_FASTER`, the speed-upgrade counts, the rares, Monarchy,
    Socialism, the unit wonders.

103. **A woodcutter's clock at 1,686.** After 26,094 agreeing fields the
    human's `0/2` holds a wait of 445 where the original's holds 480, tile
    and phase right. ORDERS §6.4.

142. **`World::tregion` is not `get_tregion`, and its callers are
    unaudited.** Four items were a gate asking the wrong one of the two
    (138, 140, 145 — PATHFINDER §15; 147 the other way, §16). `path`'s
    four are done; eleven in `orders`, `scout`, `transport`, `roads`,
    `army`, `place` and `group` are not. Grep each, then guard.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `Leader::produce_unit@006cb9e0` and four others; `World::danger` here
    indexes it by *region*, as `ai_build`/`ai_make`/`ai_research`/`ai_units`
    were written to, and both answer 0 while nothing writes it.

82. **run40's human files one slot under good 2.** `get_good@0063bd50`'s
    table at `0063bd84` cannot produce it; `Leader::plan_strategy@006b9620`
    line 1137 assigns the whole array from `City::count_gather_slots` and
    raises the high-water to match — that second writer is unread. Beside
    it: the original holds **0** in goods 3–5 where this holds **100**.

120. **Great Lakes' word at 1802, and a bird's flight physics.** One draw
    at `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 <
    Unit::do_job+0xd7`, reached **once** in run33's 1,851 frames and still
    the boundary over run53's 24,000. SYNC §3.9 names `do_air_physics` as
    the one arm of it no run has forced.

105/45. **Gaia's positions — East Indies' half is what is left.** run39:
    191,876 of 192,504, first bad **1658**; run33 exact at 74,040.
    `Sim::reseat_animal` corrects them where a dump has clocks (SYNC §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (110's day, 133's frame 3579). **16 of the 61**
    `rng.roll()`/`rng.get()` calls in `crates/sim` have no `self.mark(`
    above them. Mark or explain each, then make the list a guard.

124. **The loop flag is per animation file** (ANIM §3.3): by slot it is no
    constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a non-looping and
    38 a looping file. Carry it beside `Art::piece_lengths`.

153. **The `TRIBE` record, whole.** `Tribe::log_data@006f0d70` prints
    `graft[352]`, `barbarian`, `build_continent`, `people` and
    `text_substitute` in every `DUMP_ALL` dump and this crate parses none:
    the graft table is identity and `barbarian` false by assumption (TECH,
    "Where a player's nation comes from"). One parser, one comparison.

The ledgers, each a number nothing yet counts: (87) **the widening
ledger** — items 74, 83, 69, 113, 123 and 144 were closed or sharpened by
fields the parser had and nothing compared, so count, per record *and* per
capture, the fields `rondata::diff` parses and never compares; (88) **the
blind list** — `report.py … blind docs/` lists the cited functions no
traced run has entered (101 of 617), pin it as a floor and put it in each
Coverage section; (72) every `+0xNN` a document pins, checked against its
module, and every verb (the `name@00xxxxxx` half is `docs_guard` already);
(89) the instrument's last guard, (c) alone — run33's floor pins totals
mostly past its parting; (35) **`mylos` as a cache** — VISION §7: player 1
carries no `0x4000000` at the end of 202 or 203, yet its Scout's `mylos`
moves 4 → 6 and ours moves at 202, so what is owed is the *cache*.

Three fields nothing here writes: (48) **the object chain, whole** —
`collide.rs` chains units only where the original threads buildings and
goodies through it too (COLLISION §3, §7, GOODY §1), and `down`/`down_who`
go uncompared; (73) **`UnitData::group`** — no unit holds the back-pointer,
so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled and run33's scout
goes 65 → 64 on 96 where `get_open_slot` says 65; (56) **the cell's
`BUILDING` bit** — run13's frame-95 world carries it on `(52, 22)` and this
does not, and `army`'s muster search reads it (ARMY §13); find the *tile*
mask's writer.

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker: the keystroke
    arrived as the chat key; ORACLE 51/52 name the next steps. With it
    COLLISION §9.

116. **The one `SITE` slot still wrong, and the rule that is not it.** AI
    §18: a 5×5 slide keeps its centre where the original leaves it, so
    `blocked_town` refuses a cell `site_clear` allows. `blocked_site`'s
    fog-majority `0x24` (CITIES §11) moves **no** number — the slide wants
    a second reading, not that rule.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap and `was_seen`'s `reg_forts`
    arm — inert until a capture has a Temple or a Fort. (b) AI §2.1's
    `check_explore` answers the whole region grid — 900 against 36 and 19.

The road residue, both in ROADS §7.1: (57) the node counts — 1,046 v 1,043
and 1,460 v 1,870 — re-read on run43's own before-grid (ORACLE run43); (58)
the height loader's mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8` — **Military**, not ARMY's "current age". `army.rs:769,1365`
    read `tech[w].ages`, so every `age < 2/3/5` gate in the muster caps
    and `find_target` fires on the wrong counter.

155. **The other two writers of `rare` and `good_obj`, and the widening
    that waits on them.** Both fields are in every `UNIT` record and
    nothing compares them (item 87). run58's `1/14` takes `rare −1` on
    4872 from `Unit::think@005f6e40:179`'s rare-collector arm — a
    **packed** AI `is_rare_collector` on `idle == 1`/32 takes
    `Unit::do_gather@005fce20`, and the arm also bumps `idle` and tries
    `unpack_merchant(4)` — and `rare 6, good_obj 1` on 4992 from the
    gather job (`Unit::do_job@00617a10` → `Unit::do_gather@005ef2a0`).
    ORDERS §6.10; land both and the two fields become a diff.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; the `CITY` widening;
a `find_target` block; run7's order stream; a mounted attacker; a caravan;
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
