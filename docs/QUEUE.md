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

*2026-09-02, Opus — a unit is not one figure.* **East Indies 6164 → 6166**;
Great Lakes unchanged at 2419.

- **The guy stack is `CREW_SIZE + 1`.** `Unit::init@00612100:471`–`508`
  sizes it to `crew_size (+0x30c) + squad_size (+0x304)`, and
  `UnitType::init@0061ab50:723` writes the literal **1** into `+0x304` for
  every type, on the line before it reads `UBER_SIZE` and `CREW_SIZE`. So a
  Citizen has 1 figure, a Scout 2 (the dog), a Caravan 3, a Trebuchet 4.
  `guy_mark` is `Unit::init`'s copy of `squad_size`, not the length.
- **A dump-seeded unit never had it**: `build_sim` hands a unit the `GUY`
  blocks the file prints, so the crew machinery was right on twenty scout
  dogs while `Sim::init_guys` — the path a unit *born in the sim* takes —
  held `let count = 1usize`.
- The reading is a diff: `CREW_SIZE + 1` is asserted against every player
  unit of every `DUMP_ALL` capture — 120 units, 20 two-figure.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w6166 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `East Indies' next cause is a road. On 6166 the original
spends 3,207 `PathFinder::calc_road_cost+0x46` draws and this crate spends
none — a whole `astar_caravan_road` search, two frames after the Caravan
that owns them was born (item 173 gave it its figures). `roads.rs` models
the search and run62 pinned it node for node, so the question is the
*caller*: which of `find_road`'s conditions fires on 6166 here and does not
there. Start by counting road draws per frame on both sides across run54 —
whether this is one road the crate skips or the first of many is the whole
shape of the item.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

174. **The Caravan's road, and East Indies' word.** On 6166 the original
    spends 3,207 `PathFinder::calc_road_cost+0x46` draws — an
    `astar_caravan_road` — and this crate none, two frames after the
    Caravan was born. run62 pinned the search node for node; the gap is
    the caller. ROADS.

169. **`compute_site_stats`' arithmetic, on 7,122 of run63's 27,000 site
    fields.** Four of leader 1's sites predate the window: `(45, 52)` scores
    twice the original's, `(44, 52)` four times, and one extra site takes a
    slot the original leaves alone, dragging every `rank` and the order with
    it. run59's census has the same shape 250 frames earlier — two oracles
    on disk. AI §2.13 steps 6–12 hold the factors.

172. **The `bucket` pair 165 leaves behind.** The AI's six rates and six
    incomes are the original's on every frame of run59's window; `bucket`
    was one apart on 5002 and 5061 in run60's curve and nothing has
    re-measured it. Cheap: run60 is on disk. **Takes 155's `rare`/`good_obj`
    writers along** — `Unit::think`'s rare-collector arm (ORDERS §6.10) is
    unmodelled, `idle += 1` and all.

166. **`resource_cap` on five goods, two frames from 2958** — 1392 here,
    2000 there, right again on 2960. ECONOMY, "The commerce cap": a
    transient nothing else in 324,000 good-frames does.

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1,
    decompile and dump agreeing. Move the gate from `Sim::strategy_all` into
    `production_ai` and step 16 seeds a **human army**, which no capture has
    — find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned. (a)
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, each
    one apart — the circle sweep at a mid-game city. (b) From 2576 `1/2000`
    holds 11 gatherers and `1/2007` none against 10 and 1 — step 2/10.

146. **The other nine national arms of `train_time`.** `006508c0`'s fixed
    order after the ramp, only the British built (PRODUCTION, "The tail's
    first caller"). Behind them: `TROOPS_FASTER`, the speed upgrades, the
    rares, Monarchy, Socialism, the wonders.

142. **`World::tregion` is not `get_tregion`.** Four items were a gate
    asking the wrong one (138, 140, 145 — PATHFINDER §15; 147 the other
    way, §16). Eleven callers are left, unaudited.

136. **The danger grid has no writer, and four readers index it wrongly.**
    `WorldData::danger[who]@+0x13c` is `int[reg_size]`, a half-resolution
    **cell** grid — `danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]` in
    `produce_unit@006cb9e0` and four others; `World::danger` reads it by
    *region*. Both answer 0.

122. **A draw with no mark of its own.** `mark_sites` gives an unmarked
    draw the label still standing, so a sequence reads wrong rather than
    short (110's day, 133's 3579). **16 of 61** `rng.roll`/`get` calls have
    no `self.mark(`; mark each, then guard.

153. **The `TRIBE` record, whole.** `Tribe::log_data@006f0d70` prints
    `graft[352]`, `barbarian`, `build_continent`, `people` and
    `text_substitute` in every `DUMP_ALL` dump; none parsed (TECH).

The ledgers, each a number nothing yet counts: (87) **the widening
ledger** — items 74, 83, 69, 113, 123, 144, 154, 169 and now 168/165 (one
`track.py --changes` over `myspeed`) were closed or
sharpened by fields the parser had and nothing compared, so count, per
record *and* per capture, the fields `rondata::diff` parses and never
compares; (88) **the blind list** — `report.py … blind docs/` lists the
cited functions no traced run has entered (101 of 617), pin it as a floor
and put it in each Coverage; (72) every `+0xNN` a document pins, checked
against its module; (89) the instrument's last guard, (c) alone; (35)
**`mylos` as a cache** — VISION §7, whose Scout moves 4 → 6 a frame early.

175. **`Unit::squad_size` is the guy count and should be the uber chain.**
    `build_sim` sets it from `u.guys.len()`; its doc calls it
    `curr_uber_size@0060a760`, which walks `o_up`/`o_down` over chained
    `UnitData`, not figures. Disjoint in `unitrules.xml`: the 109
    `UBER_SIZE 3` types have `CREW_SIZE 0`, every crewed type `UBER_SIZE 1`,
    so a scout bleeds as a squad of two. **Takes 48's chain with it.**

Three fields nothing here writes: (48) **the object chain, whole** —
`collide.rs` chains units only where the original threads buildings and
goodies through it too (COLLISION §3, §7, GOODY §1), `down`/`down_who`
uncompared; (73) **`UnitData::group`** — no back-pointer, so
`Group::normalize`'s cull (GROUPS §4.3) is unmodelled and run33's scout goes
65 → 64 on 96; (56) **the cell's `BUILDING` bit** — run13's frame-95 world
carries it on `(52, 22)` and this does not (ARMY §13's muster search).

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker; ORACLE 51/52 the
    next steps.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, `was_seen`'s `reg_forts` arm.
    (b) AI §2.1's `check_explore` answers the whole region grid, 900 v 36/19.

Four measured one-liners: (103) a woodcutter's clock at **1,686** — after
26,094 agreeing fields the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half left — run39 191,876 of
192,504, first bad **1658**, run33 exact at 74,040, SYNC §4.2; (124) **the
loop flag is per animation file**, ANIM §3.3 — 42 `<UNIT>` entries give
`CHAR_DUMP_WOOD` a non-looping file and 38 a looping one, carry it on `Art`;
(116) **the one `SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its
centre where the original leaves it (`blocked_town` v `site_clear`).

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`; `army.rs:769,1365` read `tech[w].ages`, so the muster caps
    fire on the wrong counter.

161. **The whole make-list block is 4,300 frames behind the word.**
    `create_buildings` first runs on East Indies frame 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` are unreachable
    until the word passes 9982.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930`
    zeroes all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose prerequisite
    is the tech just gained — so the original holds 0 knowledge, metal and oil
    through Ancient where this holds 100. Unread: where food, timber and
    wealth are paid (COSTS).

167. **Two of run61's leavings.** (a) §3.9's reading-only pair: the
    landing search's *cell* (the thirtieth sampled, only its sixty draws
    checked; a `callwin` over `think_bird`'s tail settles it) and
    `Unit::do_strafe`, so the dock's gull is unmodelled. (b) **`Unit::init`'s
    tile snap is every unit's**: `(p / 48) · 48 + 24`, and only `spawn_bird`
    has it. It cannot go in `Unit::new` (dump-loaded positions are mid-step),
    so it goes site by site.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; a `find_target`
block; run7's order stream; a mounted attacker; a caravan; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2's rows; (58) ROADS §7.4's
three `f32` height means, none near a measured search.

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
