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

*2026-09-02, Opus — the AI had never trained a merchant, on any map.*
**East Indies 6353 → 6356**; Great Lakes 2419. Item **180 closed**, and it
was one host function.

- **6353 is a Merchant**, and run65's own window names it: the AI Market
  `1/2013` is one `MERCHANT` deep on all eighteen blocks, `job_counter`
  climbing a hundred a frame from the caravan's hand-over on 6164 to
  **18,720** — `JOB_TIME` 156 at `UNIT_RATE_BASE` 120 — on 6353.
- **`economic.bhs` gates its merchant block on
  `num_rare_resources_seen(who)`** and this crate's host answered a flat
  zero. `Leader::new_rare@006d9e70` fills the list, `World::reveal_fog`
  calls it on exactly the fog cells `set_seen` answered *changed* for, and
  `FISH`/`WHALES` are excluded — they pay a boat, not a merchant. The AI
  has seen `CITRUS` and `HORSES` by 5000 (ECONOMY, "The rares a leader has
  seen"). Left standing: the **start fog is not replayed**, so a rare under
  a leader's frame-0 fog is unrecorded; East Indies' one such good is oil.
- **The widening carried it.** run65's test reads the **build queues** now
  — `queued` and every live slot's type and clock — so the count and the
  18,720 are assertions, not a frame that happened to land. Item 87.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w6356 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `6356 is the Merchant's first step — `Guy::do_turn <
Unit::move_step` at the head of the frame, three frames after its birth.
Item 182: `Unit::think_merchant@005f4740`, head `unpack_merchant(3)`, body
a score over `new_rares` and a `MOVE_TO` at the winner.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

182. **`Unit::think_merchant@005f4740` — the Merchant's walk.** The word, and
    the whole of it. A **deployed** merchant returns 1 at once, so only a
    packed one `unpack_merchant(this, 3)` refuses searches. It scores every
    `new_rares` entry — `good.ever_seen & (1 << who)`, the good's region owner
    allied or none, its cell's ocean bit equal to the merchant's, and three
    refusals (a friendly unit of my type within `0x300`, its ordered sibling,
    an enemy combat unit within `0xc00`) — as `200 − 10·i`, `+100` for my own
    `get_tregion`, minus `danger[who][cell >> 1]`, floored at 1; highest wins,
    is **removed and re-appended**, and takes a `MOVE_TO` behind
    `close_orders`/`clear_partial_path`. Takes 142's `get_tregion`.

169. **`compute_site_stats`' arithmetic, on 7,122 of run63's 27,000 site
    fields.** `(45, 52)` scores twice the original's, `(44, 52)` four times, an
    extra site drags every `rank`; run59 the same 250 frames earlier (AI §2.13
    steps 6–12).

172. **The `bucket` pair 165 leaves behind.** The six rates and incomes are
    the original's on every frame of run59's window; `bucket` was one apart
    on 5002 and 5061 in run60's curve. Cheap: run60 is on disk. **Takes
    155's `rare`/`good_obj` writers** — `Unit::think`'s rare-collector arm
    (ORDERS §6.10) is unmodelled.

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1.
    Moving the gate from `Sim::strategy_all` into `production_ai` makes step
    16 seed a **human army**, which no capture has — find the gate between
    steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned. (a)
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, each one
    apart. (b) from 2576 `1/2000` holds 11 gatherers and `1/2007` none against
    10 and 1 — step 2/10.

146. **The other nine national arms of `train_time`.** `006508c0`'s fixed
    order after the ramp, only the British built (PRODUCTION, "The tail's first
    caller"); behind them `TROOPS_FASTER`, the speed upgrades, the rares,
    Monarchy, Socialism, the wonders. The Merchant's 18,720 is the ramp alone,
    so East Indies has never measured the tail.

142. **`World::tregion` is not `get_tregion`.** Four items were a gate asking
    the wrong one (PATHFINDER §15, §16); eleven callers unaudited, and
    `caravan.rs`'s two known wrong (179's by-catch).

Two records with no reader: (122) **a draw with no mark of its own** —
`mark_sites` labels an unmarked draw with the one still standing, and **16 of
61** `rng.roll`/`get` calls have no `self.mark(`; (153) **the `TRIBE` record,
whole** — `Tribe::log_data@006f0d70` prints `graft[352]`, `barbarian`,
`build_continent`, `people` and `text_substitute` in every `DUMP_ALL` dump,
none parsed (TECH).

178. **The danger map's unit pass, unexercised.** DANGER §8: run64's two
    leaders have no military unit on the map on any frame divisible by 200,
    so `role & 0x10000`, `(attack · 5) / 10` and the war gate rest on the
    reading alone. A `DUMP_ALL` `WORLD` block on a 200-frame boundary after
    either side has an army settles it (run16's scenario is one). Takes
    `UnitData::is_seen`; 182 reads the same map.

181. **§7's unreached arithmetic, which 179 did not take.** The arrival box,
    `compute_trade`'s `× 16 / 2` and `new_caravan`'s `(epoch[1] + 1) · 10` fire
    at 6511 and 6766, inside the word now and unasserted. CARAVAN §7.2–§7.3; a
    `LEADERS=9` census over `[6760, 6790)` would also name the **second
    Merchant**, trained on 6571 and covered by nothing.

The ledgers, each a number nothing yet counts: (87) **the widening ledger** —
items 74, 83, 69, 113, 123, 144, 154, 169, 168/165, 179 and now 180 were
closed or sharpened by fields the parser had and nothing compared, so count,
per record *and* per capture, the fields `rondata::diff` parses and never
compares; (88) **the blind list** — `report.py … blind docs/` lists the cited
functions no traced run has entered (101 of 617), pin it as a floor and put it
in each Coverage; (72) every `+0xNN` a document pins, checked against its
module; (89) the instrument's last guard, (c) alone; (35) **`mylos` as a
cache** — VISION §7, whose Scout moves 4 → 6 early.

175. **`Unit::squad_size` is the guy count and should be the uber chain.**
    `build_sim` sets it from `u.guys.len()`; its doc calls it
    `curr_uber_size@0060a760`, which walks `o_up`/`o_down` over chained
    `UnitData`. Disjoint in `unitrules.xml`: the 109 `UBER_SIZE 3` types have
    `CREW_SIZE 0` and every crewed type `UBER_SIZE 1`, so a scout bleeds as a
    squad of two. **Takes 48's chain.**

Three fields nothing here writes: (48) **the object chain, whole** —
`collide.rs` chains units only where the original threads buildings and
goodies through it too (COLLISION §3, §7, GOODY §1), `down`/`down_who`
uncompared; (73) **`UnitData::group`** — no back-pointer, so
`Group::normalize`'s cull (GROUPS §4.3) is unmodelled and run33's scout goes
65 → 64 on 96; (56) **the cell's `BUILDING` bit** — run13's frame-95 world
has it on `(52, 22)` and this does not (ARMY §13's muster search).

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, `was_seen`'s `reg_forts` arm.
    (b) AI §2.1's `check_explore` answers the whole grid, 900 v 36/19.

Five measured one-liners: (23) **the formation byte's sign, the Echelon
half** — `reverse`'s *displacement* is read only on GROUPS §6.4's Echelon rows
and every captured group is a Line, run52 naming the blocker; (103) a
woodcutter's clock at **1,686** — after
26,094 agreeing fields the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half left — run39 191,876 of
192,504, first bad **1658**, run33 exact at 74,040, SYNC §4.2; (124) **the
loop flag is per animation file**, ANIM §3.3 — 42 `<UNIT>` entries give
`CHAR_DUMP_WOOD` a non-looping file and 38 a looping one, carry it on `Art`
(the two index tests in front of it landed with 176); (116) **the one
`SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its centre where the
original leaves it (`blocked_town` v `site_clear`).

Two one-field reads: (166) **`resource_cap` on five goods, two frames from
2958** — 1392 here, 2000 there, right on 2960 (ECONOMY, "The commerce cap"),
a transient nothing else in 324,000 good-frames does; (107) **`epoch[0]` is
the Military level and `army.rs` reads `ages`** — `get_epoch_base(0)` is
`BASE_MILITARYTYPES` and the army family reads `+0xe8`, where
`army.rs:769,1365` read `tech[w].ages`.

161. **The whole make-list block is 3,600 frames behind the word.**
    `create_buildings` first runs on East Indies frame 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` are unreachable
    until the word passes it.

156. **`STARTING_GOODS` arrives with the age.** `Leader::init@006e3930` zeroes
    all six; `Leader::gain_tech@006dcb60` pays `bucket_add(g,
    game->starting[g])` for a good whose bucket is zero and whose preq is the
    tech just gained — so the original holds 0 knowledge, metal and oil through
    Ancient where this holds 100. Unread: where food, timber and wealth are
    paid (COSTS).

167. **Two of run61's leavings.** (a) §3.9's reading-only pair: the landing
    search's *cell* and `Unit::do_strafe`, so the dock's gull is unmodelled;
    a `callwin` over `think_bird`'s tail settles it. (b) **`Unit::init`'s
    tile snap is every unit's** — `(p / 48) · 48 + 24`, and only
    `spawn_bird` has it.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — `zsh tools/fuzz/run.sh 424242 1000 1300`
then `report.py … blind docs/` before deleting it; a `find_target` block;
run7's order stream; a mounted attacker; a caravan; `calc_gather` non-flat;
`Leader::diplomacy`; (77) ANIM §3.2's rows; (58) ROADS §7.4's `f32` heights.

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
