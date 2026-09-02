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

*2026-09-02, Opus — the map has no danger on it.* **East Indies 6189 →
6198**; Great Lakes 2419. Items 177(a) and **136** closed together:
177(a) *was* 136.

- **The walk was a cost function.** `add_move_order` writes no waypoint;
  `find_wpath` did, because `calc_cost`'s world arm was missing
  `danger[who][half-cell] / 8` — 65 to 135 **negative** around a leader's
  own city, so eight steps were 8–16 too dear. DANGER §6.
- **The trace and the dump named it, not a reading**: run64's proxied
  `calc_cost` gave the eleven wrong prices, their deltas grouped by
  *half-cell*, and `log_data` has printed the whole map all along.
- **`calc_danger`, every 200th frame**: buildings write for every viewer
  *including the owner*, which is what makes it negative. DANGER §3–§5.
- **Nothing installed `diplos`** — every capture ran at peace where the
  dump says war, 46 readers of `is_enemy` answering wrong. Installed.
- **Two new diffs**: the map whole (7,200 values) and the 47 prices of the
  search it broke. run64's window is whole, 34 pinned fields → 0.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w6198 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `Item 177's second half is the word: 6198 is
`Unit::do_trade+0x40`, three `Guy::set_anim` draws over the caravan's three
figures as it **arrives** at the near city — one through
`Unit::set_anim+0x56` and two through `+0xb6`. Nothing past the road is
modelled at all (CARAVAN §7), so this is a mechanic rather than a residue:
read `do_trade` past its `build_road` arm and `Unit::work`'s §6 block, and
give the arrival its `set_anim`. run64's window and run54's 24,000 frames
are on disk; CARAVAN §7 names what a *new* capture would have to hold.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

177. **The caravan's legs, and its wealth** — (a) closed, and it was
    item 136. What is left is the arrival and everything past it:
    `Unit::do_trade+0x40`'s three `set_anim` draws (the word, 6198),
    `TradeOrder +0x20 loaded`, the trade income (`docs/ECONOMY.md`'s
    `trade_val`), and what a fallen city does to a route. CARAVAN §7 lists
    what a capture would have to hold to separate the destination loop's
    arms — three cities of one leader.

169. **`compute_site_stats`' arithmetic, on 7,122 of run63's 27,000 site
    fields.** `(45, 52)` scores twice the original's, `(44, 52)` four
    times, and an extra site drags every `rank`; run59's census has the
    same shape 250 frames earlier. AI §2.13 steps 6–12.

172. **The `bucket` pair 165 leaves behind.** The six rates and incomes
    are the original's on every frame of run59's window; `bucket` was one
    apart on 5002 and 5061 in run60's curve. Cheap: run60 is on disk.
    **Takes 155's `rare`/`good_obj` writers along** — `Unit::think`'s
    rare-collector arm (ORDERS §6.10) is unmodelled.

166. **`resource_cap` on five goods, two frames from 2958** — 1392 here,
    2000 there, right again on 2960. ECONOMY, "The commerce cap": a
    transient nothing else in 324,000 good-frames does.

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1.
    Move the gate from `Sim::strategy_all` into `production_ai` and step 16
    seeds a **human army**, which no capture has — find the gate between
    steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned. (a)
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, each
    one apart. (b) From 2576 `1/2000` holds 11 gatherers and `1/2007` none
    against 10 and 1 — step 2/10.

146. **The other nine national arms of `train_time`.** `006508c0`'s fixed
    order after the ramp, only the British built (PRODUCTION, "The tail's
    first caller"); behind them `TROOPS_FASTER`, the speed upgrades, the
    rares, Monarchy, Socialism, the wonders.

142. **`World::tregion` is not `get_tregion`.** Four items were a gate
    asking the wrong one (PATHFINDER §15, §16). Eleven callers, unaudited.

122. **A draw with no mark of its own.** `mark_sites` gives an unmarked
    draw the label still standing, so a sequence reads wrong rather than
    short. **16 of 61** `rng.roll`/`get` calls have no `self.mark(`.

153. **The `TRIBE` record, whole.** `Tribe::log_data@006f0d70` prints
    `graft[352]`, `barbarian`, `build_continent`, `people` and
    `text_substitute` in every `DUMP_ALL` dump; none parsed (TECH).

178. **The danger map's unit pass, unexercised.** DANGER §8: run64's two
    leaders have no military unit on the map on any frame divisible by
    200, so `role & 0x10000`, `(attack · 5) / 10` and the war gate rest on
    the reading alone. A `DUMP_ALL` `WORLD` block on a 200-frame boundary
    after either side has an army settles it; run16's scenario is one and
    its window is cheap. Takes `UnitData::is_seen` with it — this crate
    answers it with the building's ever-seen fog bit.

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
    captured group is a Line. run52 names the blocker.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, `was_seen`'s `reg_forts` arm.
    (b) AI §2.1's `check_explore` answers the whole grid, 900 v 36/19.

Four measured one-liners: (103) a woodcutter's clock at **1,686** — after
26,094 agreeing fields the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half left — run39 191,876 of
192,504, first bad **1658**, run33 exact at 74,040, SYNC §4.2; (124) **the
loop flag is per animation file**, ANIM §3.3 — 42 `<UNIT>` entries give
`CHAR_DUMP_WOOD` a non-looping file and 38 a looping one, carry it on
`Art`; the two index tests in front of it landed with 176 and this is the
flag itself;
(116) **the one `SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its
centre where the original leaves it (`blocked_town` v `site_clear`).

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`; `army.rs:769,1365` read `tech[w].ages`.

161. **The whole make-list block is 3,800 frames behind the word.**
    `create_buildings` first runs on East Indies frame 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` are
    unreachable until the word passes 9982.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930`
    zeroes all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose
    prerequisite is the tech just gained — so the original holds 0
    knowledge, metal and oil through Ancient where this holds 100. Unread:
    where food, timber and wealth are paid (COSTS).

167. **Two of run61's leavings.** (a) §3.9's reading-only pair: the
    landing search's *cell* and `Unit::do_strafe`, so the dock's gull is
    unmodelled; a `callwin` over `think_bird`'s tail settles it. (b)
    **`Unit::init`'s tile snap is every unit's**: `(p / 48) · 48 + 24`, and
    only `spawn_bird` has it — it goes site by site.

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
