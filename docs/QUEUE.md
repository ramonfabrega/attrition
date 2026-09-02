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

*2026-09-02, Opus — the opener's item, and it was two other items.* **East
Indies 5669 → 5819**; Great Lakes unchanged at 2419.

- **The 175 `think_fish` draws were never a `think_fish` bug.** They are the
  AI's third Fisherman arriving on 5668 and searching; this crate's arrived
  on 5691. `UNITDATA myspeed` says why: on 5552 all three of leader 1's
  Fishermen go **38 → 45** and its barge **25 → 30**, its land units none.
- **`Unit::update_speed`'s one rare arm is Whales**, `WHALES_SHIPS_MOVE 20%`
  on an objmask `0x2000` type. `1/16` takes `rare 31` on 5552, and the
  writer is `Leader::calc_gather` **step 6** — the idle fisherman/merchant
  walk `holdings.rs` had as not modelled. Item 165 was the same step from
  the other side: `160` food and `160` wealth is `10 × 16` twice, and `10`
  is Fish's two `BONUS_NUM`s in `resourcerules.xml`.
- **The piece that nearly did not land**: `Unit::check_idle`'s tail latches
  the first idle frame and marks a **fisherman's** owner's economy dirty —
  the 512-frame refresh becomes 8, and the whale lands on 5551 rather than
  130 frames later. `holdings.rs` had that writer as peasant/scholar/merchant.
- run59's census: **3,500 → 1,500** wrong good-frames of 18,000, the rest
  item 156. run63's position residues: two → one.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w5819 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `East Indies' next cause is 1/18, the AI's Transport
Barge: at 5819 this crate ejects its passengers (Unit::come_out+0x25ca <
Object::eject_contents+0x292) and the original does it on 5823. run63 has
1/18 running ahead on the same order and the same row from 5430; what cost
the original those frames is in (5202, 5430), which nothing on disk
measures.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

171. **`1/18` runs ahead, and it is the word.** The AI's Transport Barge
    ejects its passengers on 5819 here and 5823 there
    (`Unit::come_out+0x25ca < Object::eject_contents+0x292 <
    Unit::set_new_location+0x2b7`, run54's trace), and run63 has it ahead of
    the original on the same `MoveOrder` and the same row from the window's
    first frame. The cause is in **(5202, 5430)**, which nothing on disk
    measures — run58 ends at 5201 and run63 opens at 5430. A `LEADERS=9`
    window over that gap is the capture; grep the disk first.

169. **`compute_site_stats`' arithmetic, on 7,122 of run63's 27,000 site
    fields.** Four of leader 1's sites predate the window: `(45, 52)` scores
    twice the original's and `(44, 52)` four times, and one extra site takes
    an empty slot the original leaves alone, dragging every `rank` and the
    slot order with it. run59's census has the same shape 250 frames
    earlier — two oracles on disk. AI §2.13 steps 6–12 hold the factors.

172. **The `bucket` pair 165 leaves behind.** With step 6 landed the AI's
    six rates and six incomes are the original's on every frame of run59's
    window; `bucket` was one apart on 5002 and 5061 in run60's curve and
    nothing has re-measured it. Cheap: run60 is on disk. **Takes 155's
    `rare`/`good_obj` writers along** — `Unit::think`'s rare-collector arm
    (ORDERS §6.10) is still unmodelled, `idle += 1` and all.

166. **`resource_cap` on five goods, two frames from 2958** — 1392 here,
    2000 there, and right again on 2960. ECONOMY, "The commerce cap": a
    transient nothing else in 324,000 good-frames does.

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1,
    decompile and dump agreeing. Move the gate from `Sim::strategy_all` into
    `production_ai` and step 16 seeds a **human army**, which no capture
    has — find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned. (a)
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, each
    one apart — the circle sweep at a mid-game city. (b) From 2576 `1/2000`
    holds 11 gatherers and `1/2007` none against 10 and 1, `free` 1 v 0 from
    4176 — step 2/10's attribution.

146. **The other nine national arms of `train_time`.** `006508c0`'s fixed
    order after the ramp, only the British built (PRODUCTION, "The tail's
    first caller"). Behind them: `TROOPS_FASTER`, the speed upgrades, the
    rares, Monarchy, Socialism, the unit wonders.

142. **`World::tregion` is not `get_tregion`, and its callers are
    unaudited.** Four items were a gate asking the wrong one (138, 140, 145
    — PATHFINDER §15; 147 the other way, §16). Eleven callers are left.

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
    `oil_patches.count` and `compute_largest_gather@0066e920` are
    unreachable. Kept until the word passes 9982.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930`
    zeroes all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose
    prerequisite is the tech just gained — so the original holds 0
    knowledge, metal and oil through Ancient where this holds 100. Unread:
    where food, timber and wealth are paid (COSTS).

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
