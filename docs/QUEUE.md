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

*2026-09-03, Opus — item 193: the two maps' frontiers turned out to be one
defect.* **East Indies 6739; Great Lakes 2419.** No score moved; 193 is
closed into 191, which now stands in front of both numbers.

- **Great Lakes parts at 2419 on a route, not a clock.** One draw — the AI
  woodcutter `1/9`'s return-to-camp stand — spent here on 2419 and there on
  2420. The 404-frame chop clock behind it is exact on both sides (the
  tile choice's own draw on 1959 returns 4 either way), so the frame is the
  frame the walk ended: 2015 here, 2016 there. `1/9`'s two middle waypoints
  are each one 48-grid step short in x. That is **item 191's signature**,
  on a map with no islands, barge or merchant (SYNC §3.26, PATHFINDER §17).
- **run69 is on disk**: Great Lakes at run33's detail out to 3,000 frames,
  the capture DECISIONS 29 had owed since 09-01. 468 MB, twenty minutes,
  zero differing against run53 and against run33. Launched in the first
  five minutes, read in the last — the lane, not a second agent.
- **The map's own widening came with it.** `1/9` is the only one of
  fourteen units to leave the original's point before the word — 1993
  against the next-earliest 2467 — and the collision block is 228,821
  field-frames with none wrong.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w6739 of 24,000 · GreatLakes w2419 of 24,000

**Opener (Opus):** `191 is the default and it is now both maps: the same
two-middle-slot, one-48-grid-step route difference parts run68's 1/13 on
East Indies at 6686 and run69's 1/9 on Great Lakes at 1993. Both routes are
legal, equal-length and on the same grid, so read astar_path's tie-break
(PATHFINDER §4.2) before anything else, and run55's callwin over
astar_path/calc_cost on Great Lakes frame 1960 is the check that settles
whether it is a price or a tie. 190 is second.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, lower map first — Great Lakes. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

191. **The middle nodes are one 48-grid step short, and it is both maps'
    frontier** (PATHFINDER §17). East Indies: run68's `1/13` from 6686,
    `path[2].y` 38712 against 38760 and `path[3].x` 40584 against 40536,
    the body following on 6718. Great Lakes: run69's `1/9` from **1993**,
    the same two middle slots 48 short in x, arriving a frame early and
    putting the word on 2419. Ends, length, flags and switch frames all
    agree either way, so it is a **tie-break and not a price** — read
    `astar_path`'s wheel and its `<`/`<=` on an equal `f` (§4.2), then a
    run55-style `callwin` over `astar_path`/`calc_cost` on Great Lakes
    frame 1960 to settle it.

190. **`stance` is 1 on every unit this crate creates and 0 on the
    original's**, 2,700 rows from run68's first block. `Unit::init`
    switches five ways on `get_stance_type` and reads the leader's options
    (`00612100:282–309`); `Unit::new` writes a flat 1, and a unit stood up
    from a dump takes the dump's — which is why every earlier capture
    agreed. Unread: `LeaderOptions +0x4/+0xc/+0x1c`, and which types
    answer which case.

192. **A merchant has never been seen to unpack.** run68 ends eleven frames
    short of `1/19`'s deploy spot (MERCHANT §7): nothing on disk has the
    cast firing, the two-by-two going `PLACED`, `unit_masks & 0x80000`
    clearing, or what `Unit::do_gather` writes into `rare`/`good_obj`
    (ECONOMY step 6). A `[6730, 6800)` window buys all of it, right at the
    word. **Takes 184's `&self` `detect_unit_collision`**,
    `find_merchant_spot`'s untested third test.

169. **`compute_site_stats`' arithmetic**, 7,122 of run63's 27,000 site
    fields: `(45, 52)` scores twice, `(44, 52)` four times, an extra site
    drags every `rank`; run59 the same 250 frames earlier (AI §2.13 6–12).

172. **The `bucket` pair 165 leaves behind**: one apart on 5002 and 5061 in
    run60's curve, every rate and income exact. **Takes 155's
    `rare`/`good_obj` writers** (ORDERS §6.10).

158. **The sweep runs for a human leader; this crate skips it.** AI §23.1.
    Moving the gate from `Sim::strategy_all` into `production_ai` makes step 16
    seed a **human army**; find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned:
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, one
    apart each; and from 2576 `1/2000` holds 11 gatherers and `1/2007` none
    against 10 and 1 — step 2/10.

146. **The other nine national arms of `train_time`** after the ramp,
    `006508c0`'s fixed order (PRODUCTION, "The tail's first caller"):
    `TROOPS_FASTER`, the speed upgrades, the rares, Monarchy, Socialism,
    the wonders. Only the ramp has ever been measured.

142. **`World::tregion` is not `get_tregion`.** Four items were a gate
    asking the wrong one (PATHFINDER §15, §16); eleven callers unaudited.

Two records with no reader: (122) **a draw with no mark of its own** —
`mark_sites` labels an unmarked draw with the one still standing, and **16
of 61** `rng.roll`/`get` calls have no `self.mark(`; (153) **the `TRIBE`
record, whole** — `Tribe::log_data@006f0d70` prints `graft[352]`,
`barbarian`, `build_continent`, `people` and `text_substitute` in every
`DUMP_ALL` dump, none parsed (TECH).

178. **The danger map's unit pass, unexercised.** DANGER §8: run64's
    leaders have no military unit on any frame divisible by 200, so
    `role & 0x10000`, `(attack · 5) / 10` and the war gate rest on the
    reading. A `DUMP_ALL` `WORLD` block on a 200-frame boundary after
    either side has an army settles it. Takes `UnitData::is_seen`.

181. **§7's unreached arithmetic, which 179 did not take.** The arrival
    box, `compute_trade`'s `× 16 / 2` and `new_caravan`'s
    `(epoch[1] + 1) · 10` fire at 6511 and 6766, inside the word and
    unasserted (CARAVAN §7.2–§7.3).

**The widening ledger is built** (87, DATALAYER §4): **23** parsed fields
nothing in `diff.rs` names and **43** one capture names, pinned, falling
only — it named `stance` the day before run68 found it wrong.

The ledgers still uncounted: (88) **the blind list** — `report.py … blind
docs/` lists the cited functions no traced run has entered (101 of 617), pin
it as a floor and put it in each Coverage; (72) every `+0xNN` a document
pins, checked against its module; (89) the instrument's last guard, (c)
alone; (35) **`mylos` as a cache** — VISION §7, whose Scout moves 4 → 6
early.

175. **`Unit::squad_size` is the guy count and should be the uber chain**:
    `curr_uber_size@0060a760` walks `o_up`/`o_down`; the 109 `UBER_SIZE 3`
    types have `CREW_SIZE 0`. **Takes 48's chain.**

Three fields nothing here writes: (48) **the object chain, whole** — only
units are threaded (COLLISION §3, §7, GOODY §1), `down`/`down_who`
uncompared; (73) **`UnitData::group`** — no back-pointer, so
`Group::normalize`'s cull (GROUPS §4.3) is unmodelled, run33's scout 65 →
64 on 96; (56) **the cell's `BUILDING` bit** on `(52, 22)`, ARMY §13.

117. **Two seams the census windows now measure.** ATTRITION, "Territory":
    the temple and fort border levels, the Colosseum, the Eiffel Tower, a
    gem rare, the handicap, `was_seen`'s `reg_forts`; and AI §2.1's
    `check_explore` answers the whole grid, 900 v 36/19.

Five measured one-liners: (23) **the formation byte's sign, the Echelon
half** — `reverse`'s *displacement* is read only on GROUPS §6.4's Echelon
rows, every captured group a Line, run52 the blocker; (103) a woodcutter's
clock at **1,686** — the human's `0/2` waits 445 against 480, ORDERS §6.4;
(105/45) **Gaia's positions**, East Indies' half — run39 191,876 of
192,504, first bad **1658**, SYNC §4.2; (124) **the loop flag is per
animation file**, ANIM §3.3 — 42 `<UNIT>` entries non-looping, 38 looping,
carry it on `Art`; (116) **the one `SITE` slot still wrong**, AI §18 — a
5×5 slide keeps its centre (`blocked_town` v `site_clear`).

Two one-field reads: (166) **`resource_cap` on five goods, 2958–2959** —
1392 here, 2000 there (ECONOMY, "The commerce cap"); (107) **`epoch[0]` is
the Military level and `army.rs` reads `ages`**, `army.rs:769,1365`.

161. **The whole make-list block is 3,200 frames behind the word.**
    `create_buildings` first runs on East Indies frame 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's neighbourhood arm,
    `oil_patches.count` and `compute_largest_gather@0066e920` are all
    unreachable until the word passes it.

156. **`STARTING_GOODS` arrives with the age.** `Leader::gain_tech@006dcb60`
    pays `bucket_add(g, game->starting[g])` for a zero bucket whose preq is
    the tech just gained and `Leader::init@006e3930` zeroes all six, so the
    original holds 0 knowledge, metal and oil through Ancient where this
    holds 100. Unread: food, timber and wealth (COSTS).

167. **Two of run61's leavings.** (a) the landing search's *cell* and
    `Unit::do_strafe`, SYNC §3.9, reading-only — a `callwin` over
    `think_bird`'s tail; (b) `Unit::init`'s tile snap is every unit's and
    only `spawn_bird` has it.

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
