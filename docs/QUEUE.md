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

*2026-09-04, Opus — item 207 landed, and it moved the headline 294
frames.* **Great Lakes 5786 → 6080 by draw and by sequence; East Indies
6739, unmoved.** One capture, run74, the cheapest useful one yet — three
minutes and 31 MB, because the question was a position and a path stack,
which the *cheap* per-frame dump writes.

- **Booked as a turn, settled as a destination.** 5786's draw was
  `move_step`'s **far** turn-in-place arm, the only turn draw either side
  spends in 5,800 frames; of the six units moving exactly one type packs,
  so the unit was named before the capture — the AI's Merchant `1/24`
  (ANIM §4.8's row: packs, names no turn, pays the idle roll).
- **The two merchants were walking to different rares.**
  `think_merchant` scores `new_rares` at `200 − 10 · position`, so with a
  flat region bonus the list's *order* is the pick; theirs held three
  goods, ours two, and the missing one was first.
- **The cause was the harness, not the sim.** `reveal_fog` fires only
  where `set_seen` says `seen2` **changed** — once a game — and
  `build_sim` *installs* `seen2` instead of sweeping it, so every
  pre-frame-0 offer was skipped unrecoverably.
  `Sim::seed_new_rares_from_fog` replays them (MERCHANT §2.2.2, ECONOMY);
  run74's hundred blocks carry no order, path or angle disagreement at
  all now, and the score has its third rare.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w6739 of 24,000 · GreatLakes w6080 of 24,000

**Opener (Opus):** `208 is the default: run53 frame 6080, where the AI
scout 1/0 spends 41 draws the original spends none of — 12
think_scout+0x436, 6 +0x458, 21 +0x64c, two do_idle+0x7d — and their frame
is birds and farms alone. SCOUT §3; start with RON_DEBUG_UNIT=1/0@6050-6090.`

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

208. **Great Lakes' word is a scout that thinks and theirs does not, at
    6080.** `1/0` spends **41** draws the original spends none of — 12
    `think_scout+0x436`, 6 `+0x458`, 21 `+0x64c`, two `do_idle+0x7d` —
    while their frame is birds and farms alone (SCOUT §3). run74 stops at
    5800: read the cadence off run53's trace, or take a `[6050, 6090)`
    cheap window on run74's recipe. Nothing here is nearer the headline.

209. **The other once-per-game events a dump install swallows.** 207's
    general half: grep for the other gates of that shape — a `set_*` whose
    **return value** drives an irreversible record — and seed each from the
    installed state as `Sim::seed_new_rares_from_fog` does.
    `World::compute_reg_territory`, `new_rare`'s second caller, belongs here.

203. **`Wall::mark_behind_tiles`' `0x4`, and nothing else.** The tile-mask
    residue is **32** at 4802 and **45** at 5565, every one bit `0x4` —
    `World::set_behind@006b4230`'s low arm, from
    `Wall::mark_behind_tiles@0063d230` (`Wall::start`/`close`,
    `refresh_nearby_tiles`, `cast_bribe`) and `Mountains::add_mountain`.
    Nothing reads it. Pinned as a count that only falls; the surface half
    is pinned at **zero** (ROADS §9.5).

192. **A merchant has never been seen to unpack.** run68 ends eleven
    frames short of `1/19`'s deploy spot (MERCHANT §7): the cast, the
    two-by-two going `PLACED`, `unit_masks & 0x80000` clearing and
    `do_gather`'s `rare`/`good_obj` (ECONOMY step 6) are all past it. A
    `[6730, 6800)` window buys it all. **Takes 184's `&self`
    `detect_unit_collision`.**

169. **`compute_site_stats`' arithmetic**, 7,122 of run63's 27,000 site
    fields: `(45, 52)` twice, `(44, 52)` four times, an extra site drags every
    `rank`; run59 the same 250 frames earlier (AI §2.13 6–12).

172. **The `bucket` pair 165 leaves behind**: one apart on 5002 and 5061 in
    run60's curve, every rate and income exact. **Takes 155's `rare`/`good_obj`
    writers** (ORDERS §6.10).

158. **The sweep runs for a human leader; this crate skips it** (AI §23.1).
    Moving the gate from `Sim::strategy_all` into `production_ai` seeds a
    **human army** at step 16; find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned:
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, one
    apart; and from 2576 `1/2000` holds 11 gatherers and `1/2007` none
    against 10 and 1 — step 2/10.

146. **The other nine national arms of `train_time`** after the ramp, in
    `006508c0`'s fixed order (PRODUCTION, "The tail's first caller"):
    `TROOPS_FASTER`, the speed upgrades, the rares, Monarchy, Socialism,
    the wonders. Only the ramp is measured.

142. **`World::tregion` is not `get_tregion`.** Four items were a gate
    asking the wrong one (PATHFINDER §15, §16); eleven callers unaudited.

Two records with no reader: (122) **a draw with no mark of its own** — 16 of
61 `rng.roll`/`get` calls have no `self.mark(`; (153) **the `TRIBE` record,
whole** — `Tribe::log_data@006f0d70`'s `graft[352]`, `barbarian`,
`build_continent`, `people`, `text_substitute`, none parsed (TECH).

178. **The danger map's unit pass, unexercised.** DANGER §8: run64's
    leaders have no military unit on a frame divisible by 200, so
    `role & 0x10000`, `(attack · 5) / 10` and the war gate rest on the
    reading. A `DUMP_ALL` `WORLD` block on a 200-frame boundary after either
    side has an army settles it. Takes `UnitData::is_seen`.

181. **§7's unreached arithmetic, which 179 did not take.** The arrival
    box, `compute_trade`'s `× 16 / 2` and `new_caravan`'s
    `(epoch[1] + 1) · 10` fire at 6511 and 6766, unasserted (CARAVAN
    §7.2–§7.3).

**The widening ledger is built** (87, DATALAYER §4): **23** parsed fields
`diff.rs` names nowhere and **43** one capture names, pinned and falling
only — it named `stance` the day before run68 found it wrong.

The ledgers still uncounted: (88) **the blind list** — `report.py … blind
docs/` lists the cited functions no traced run has entered (101 of 617), pin it
as a floor and put it in each Coverage; (72) every `+0xNN` a document pins,
checked against its module; (89) the instrument's last guard, (c) alone; (35)
**`mylos` as a cache**, VISION §7 (Scout 4 → 6 early).

175. **`Unit::squad_size` is the guy count and should be the uber chain**:
    `curr_uber_size@0060a760` walks `o_up`/`o_down`; the 109 `UBER_SIZE 3`
    types have `CREW_SIZE 0`. Takes 48's chain.

Three fields nothing here writes: (48) **the object chain, whole** — only
units are threaded (COLLISION §3, §7, GOODY §1), `down`/`down_who` uncompared;
(73) **`UnitData::group`** — no back-pointer, so `Group::normalize`'s cull
(GROUPS §4.3) is unmodelled, run33's scout 65 → 64 on 96; (56) **the cell's
`BUILDING` bit** on `(52, 22)`, ARMY §13.

117. **Two seams the census windows now measure.** ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, `was_seen`'s `reg_forts`; and
    AI §2.1's `check_explore` answers the grid whole, 900 v 36/19.

Five measured one-liners: (23) **the formation byte's sign, the Echelon
half** — `reverse`'s *displacement* is read only on GROUPS §6.4's Echelon
rows, every captured group a Line, run52 the blocker; (103) a woodcutter's
clock at **1,686** — the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half — run39 191,876 of 192,504,
first bad **1658**, SYNC §4.2; (124) **the loop flag is per animation file**,
ANIM §3.3 — 42 `<UNIT>` entries non-looping, 38 looping, carry it on `Art`;
(116) **the one `SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its
centre (`blocked_town` v `site_clear`).

Two one-field reads: (166) **`resource_cap` on five goods, 2958–2959** — 1392
here, 2000 there (ECONOMY, "The commerce cap"); (107) **`epoch[0]` is the
Military level and `army.rs` reads `ages`**, `army.rs:769,1365`.

161. **The whole make-list block is 3,200 frames behind the word.**
    `create_buildings` first runs on East Indies 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` wait on it.

156. **`STARTING_GOODS` arrives with the age.** `Leader::gain_tech@006dcb60`
    pays `bucket_add(g, game->starting[g])` for a zero bucket whose preq is
    the tech just gained and `Leader::init@006e3930` zeroes all six, so the
    original holds 0 knowledge, metal and oil through Ancient where this holds
    100. Unread: food, timber, wealth (COSTS).

167. **Two of run61's leavings.** (a) the landing search's *cell* and
    `Unit::do_strafe`, SYNC §3.9 — a `callwin` over `think_bird`'s tail; (b)
    `Unit::init`'s tile snap is every unit's, only `spawn_bird` has it.

195. **`find_repair_spot` is the last of `build_done`'s three**, a seam on
    all four arms (ORDERS §5.5, §5.6, §5.9): `find_any_building(
    FILTER_DAMAGED, FILTER_NOT_UNDER_ATTACK)`, the territory-owner gate, the
    AI's `difficulty >= 2`. No capture has a damaged building.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000 1300`
then `report.py … blind docs/` before deleting it; a `find_target` block;
run7's order stream; a mounted attacker; `calc_gather` non-flat;
`Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4's `f32` heights.

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
  into `CLAUDE.md`, a brief, an agent definition, or a memory hook.
