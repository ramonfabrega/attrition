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

*2026-09-02, Opus — the bird flies, in a module nobody calls.* **Neither
headline moved.** Item 120's reading is done and its implementation is in
`crates/sim/src/air.rs`; wiring it in costs East Indies 33 frames, so it is
not wired.

- **`do_air_physics` is narrow for a bird.** `is_animal`, `AirOrder::
  returning == 0` and **owner 9 being over eight** excuse `check_fuel`, the
  landing approach, `land_plane`, both speed cuts and the whole of
  altitude. What is left is a heading, a bank, a step and the coin (SYNC
  §3.9, "The flight").
- **The coin is the world's rectangle and nothing else.** `invalid_loc` on
  an air type returns valid before every terrain test; `AirOrder::
  sharp_turn` stands until a step lands inside again, which is why run53's
  55 coins cluster a hundred frames apart.
- **The bank is the state.** Ten a frame toward ±55, and
  `air_turn_speed` reads the *previous* frame's bank — so a bird orbits its
  patrol point on a radius near 400, and the orbit is a limit cycle.
- **`single.rs` is the second software float**: `addss`/`subss`/`mulss`/
  `divss` and the conversions, exact, checked against the host's.
- **Why it is not called**: it throws `+0x639` on East Indies at **5404**
  against the original's 5437, and 20 coins against run53's 55. The residue
  is the orbit's *phase*, and no reading will settle it — DECISIONS 30.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w5437 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `Item 120 is a missing oracle, not a missing reading.
Nothing dumps owner 9, so build the field: proxy Unit::set_new_location
in tools/trace's CALLS over a window and a bird's position becomes a
per-frame record — then air.rs's residue is arithmetic.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

120. **A bird's flight is read and built; what is missing is its
    oracle.** `crates/sim/src/air.rs` reproduces `do_air_physics`,
    `bank_aircraft` and `air_turn_speed` (SYNC §3.9, "The flight"), and
    `orders.rs` does not call it: wired in it throws `+0x639` on East
    Indies at **5404** against 5437, and 20 coins against run53's 55.
    Every arm is checked against the listing and the turn rates are exact
    frame for frame, so the residue is the orbit's *phase*. **Next move is
    a capture, not a reading**: `tools/trace/`'s `CALLS` proxy logs a
    chosen function's arguments, and `Unit::set_new_location@005f8d20`
    carries the new position — a window of it over a Great Lakes run is
    the per-frame record owner 9 has never had. Then wire the module in
    and the floors move: one draw still bounds both maps.

165. **A merchant on a rare, and it is the whole of what run60 leaves.**
    `income[food]` **1440 against 1600** and `income[wealth]` **0 against
    160** from frame **4992**, with `bucket` one apart from 5002 and 5061.
    The AI's merchant unpacks at 4988 — `SpellType::cast_unpack@006709c0`
    → `UnitData::good_merchant_spot@006068a0`, the family being `0x3d`,
    `0x3e`, `0x190` — and run58's `1/14` takes `rare 6, good_obj 1` on
    4992. Nothing here models it, so the AI earns no wealth at all after
    4992. **Takes item 155's `rare`/`good_obj` writers with it.**

166. **`resource_cap` on five goods, two frames from 2958** — 1392 here,
    2000 there, and right again on 2960. ECONOMY, "The commerce cap": a
    transient nothing else in 324,000 good-frames does.

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1,
    decompile and dump agreeing. Move the gate from `Sim::strategy_all` into
    `production_ai` and step 16 seeds a **human army**, which no capture
    has — find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned.
    (a) `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976,
    each one apart — the circle sweep at a mid-game city. (b) From 2576
    `1/2000` holds 11 gatherers and `1/2007` none against 10 and 1, and from
    4176 its `free` is 1 against 0 — step 2/10's attribution.

146. **The other nine national arms of `train_time`.** `006508c0`'s fixed
    order after the ramp, of which only the British is built (PRODUCTION,
    "The tail's first caller"); the rest want a nation the captures play.
    Behind them: `TROOPS_FASTER`, the speed upgrades, the rares, Monarchy,
    Socialism, the unit wonders.

142. **`World::tregion` is not `get_tregion`, and its callers are
    unaudited.** Four items were a gate asking the wrong one (138, 140,
    145 — PATHFINDER §15; 147 the other way, §16). `path`'s four are done;
    eleven elsewhere are not.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `produce_unit@006cb9e0` and four others; `World::danger` reads it by
    *region*, both answering 0.

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so a sequence reads wrong
    rather than short (110's day, 133's 3579). **16 of 61** `rng.roll`/`get`
    calls have no `self.mark(`; mark each, then guard.

153. **The `TRIBE` record, whole.** `Tribe::log_data@006f0d70` prints
    `graft[352]`, `barbarian`, `build_continent`, `people` and
    `text_substitute` in every `DUMP_ALL` dump; this crate parses none (TECH).

The ledgers, each a number nothing yet counts: (87) **the widening
ledger** — items 74, 83, 69, 113, 123, 144 and 154 were closed or sharpened
by fields the parser had and nothing compared, so count, per record *and*
per capture, the fields `rondata::diff` parses and never compares; (88)
**the blind list** — `report.py … blind docs/` lists the cited functions no
traced run has entered (101 of 617), pin it as a floor and put it in each
Coverage; (72) every `+0xNN` a document pins, checked against its module;
(89) the instrument's last guard, (c) alone; (35) **`mylos` as a cache** —
VISION §7: player 1 carries no `0x4000000` at the end of 202 or 203, yet its
Scout's `mylos` moves 4 → 6 and ours moves at 202.

Three fields nothing here writes: (48) **the object chain, whole** —
`collide.rs` chains units only where the original threads buildings and
goodies through it too (COLLISION §3, §7, GOODY §1), and `down`/`down_who`
go uncompared; (73) **`UnitData::group`** — no unit holds the back-pointer,
so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled and run33's scout
goes 65 → 64 on 96; (56) **the cell's `BUILDING` bit** — run13's frame-95
world carries it on `(52, 22)` and this does not, and `army`'s muster search
reads it (ARMY §13).

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker (the keystroke arrived
    as the chat key); ORACLE 51/52 name the next steps.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap and `was_seen`'s `reg_forts` arm.
    (b) AI §2.1's `check_explore` answers the whole region grid, 900 v 36/19.

Four measured one-liners: (103) a woodcutter's clock at **1,686** — after
26,094 agreeing fields the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half left — run39 191,876 of
192,504, first bad **1658**, run33 exact at 74,040, SYNC §4.2; (124) **the
loop flag is per animation file**, ANIM §3.3 — 42 `<UNIT>` entries give
`CHAR_DUMP_WOOD` a non-looping file and 38 a looping one, carry it on `Art`;
(116) **the one `SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its
centre where the original leaves it, so `blocked_town` refuses a cell
`site_clear` allows.

The road residue, both ROADS §7.1: (57) the node counts — 1,046 v 1,043
and 1,460 v 1,870 — on run43's own before-grid; (58) the height loader's
mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`; `army.rs:769,1365` read `tech[w].ages`, so the muster caps fire
    on the wrong counter.

161. **The whole make-list block is 4,606 frames behind the word.**
    `create_buildings` first runs on East Indies frame 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` are
    unreachable by any capture. Kept here until the word passes 9982.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930`
    zeroes all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose
    prerequisite is the tech just gained — so the original holds 0
    knowledge, metal and oil through the Ancient age where this crate holds
    100. Unread: where food, timber and wealth are paid (COSTS).

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; a `find_target`
block; run7's order stream; a mounted attacker; a caravan; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2's rows.

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
