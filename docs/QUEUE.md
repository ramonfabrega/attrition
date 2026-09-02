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

*2026-09-02, Opus — the opener's item, closed by the trace and confirmed by
a capture.* **East Indies 5592 → 5669**; Great Lakes unchanged at 2419.

- **A draw's caller offset is a predicate's answer.** `think_peasant` has
  two `think_scout` calls — `+0x2ac` (no site claims this region) and
  `+0x2ca` (one does, and I have been idle seven frames). The original
  spends `+0x2ac` on 5455 and `+0x2ca` on 5666, so what changed between
  them was its **site list**, not the walk — read off the trace before
  anything was booked.
- **run63** is run58's recipe with `[End Frame]` narrowed to `[5430, 5700)`
  and `LEADERS=9` in it: 16 min, 482 MB, zero differing on 5,701 frames.
  The AI citizen `1/15` walks the original's own 137 frames and arrives on
  5592; the original stands there seventy-three more, and on 5577 its tenth
  site becomes `(34, 33) val 9728` — the citizen's own cell.
- **`COLONIZE_BONUS` is a technology's, not a nation's**: the fourth of
  `rules.xml`'s `TECHBONUSES`, `preq0 = Coinage`. Carried here as a
  `Nation` flag nothing set, so `blocked_location` refused every first city
  in an unsettled region with `COLONIZE 0x1c` and `compute_site_stats`
  scored the whole region zero. The AI takes Coinage on 5177, which is why
  no earlier capture reached it. CITIES §2.6.1 question 4 closed.
- **The free half came first**: run59's census had printed `Leader::sites`
  uncompared for a day, and it said the site *values* were wrong before the
  capture was booked (item 169).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w5669 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `East Indies' next cause is Unit::think_fish+0x27a at
5669 — one frame of 175 draws off Unit::think's idle tail where this crate
spends none. run63's window covers it, so the dump is already on disk.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

168. **`Unit::think_fish+0x27a` — 175 draws in one frame, and the word.**
    East Indies 5669: the original spends them off `Unit::think+0x6a6 <
    Unit::do_idle+0x94` and this spends none. run63's `[5430, 5700)` window
    holds the frame in full detail, so no capture is owed.

169. **`compute_site_stats`' arithmetic, on 7,122 of run63's 27,000 site
    fields.** Four of leader 1's sites predate the window: `(45, 52)` scores
    twice the original's and `(44, 52)` four times, and one extra site takes
    an empty slot the original leaves alone, dragging every `rank` and the
    slot order with it. run59's census has the same shape 250 frames
    earlier — two oracles on disk. AI §2.13 steps 6–12 hold the factors.

170. **Two position residues run63 found on new ground**, both on the
    original's own order and line: `1/18` **five frames ahead** from 5430 at
    `myspeed 25`; `1/17` **turns three frames late** at 5552 (`(-5,-37)` held
    to 5555 where the original goes `(-6,-44)`, `(-28,-36)`).

165. **A merchant on a rare, and it is the whole of what run60 leaves.**
    `income[food]` **1440 against 1600** and `income[wealth]` **0 against
    160** from **4992**, `bucket` one apart from 5002 and 5061. The AI's
    merchant unpacks at 4988 — `SpellType::cast_unpack@006709c0` →
    `UnitData::good_merchant_spot@006068a0`, family `0x3d`, `0x3e`, `0x190`
    — and run58's `1/14` takes `rare 6, good_obj 1` on 4992. The AI earns no
    wealth at all after it. **Takes 155's `rare`/`good_obj` writers along.**

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
ledger** — items 74, 83, 69, 113, 123, 144, 154 and now 169 were closed or
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
