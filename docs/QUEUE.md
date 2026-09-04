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

*2026-09-04, Fable steering — two lanes opened, no score moved.* **Both
words unmoved: Great Lakes 6848, East Indies 7448.** The loop is sound; the
two finish-line counters it starves now have lanes (DECISIONS 33):

- **`att-capture`** (Opus, branch `lane-capture`): 226 first, since the
  blind list cannot move without `cover=1`; then the second-map squad
  birth that would kill 227; then the targeted captures of DECISIONS 29.
  It owns the screen — a main-loop capture is a stanza in `captures.txt`.
- **`att-audit`** (Opus, branch `lane-audit`): docs-versus-code readers
  over the eight documents amended this week, the no-writer guard (87),
  then the nine documents with no second reading, then 72. Verdicts
  arrive by message; this loop files them.
- **Only this loop writes this file.** Merge both lane branches at session
  start and read their messages before choosing an item.
- 227 stands as opened: the squad is apart on 6640, the captain exact,
  `1/28`/`1/29` not candidates of the sweep (CITIES §6.5.1, ORDERS §10).
- Corpus mirror short: re-run `backup.sh sync`, check `ls` (~212 / ~22 GB).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7448 of 24,000 · GreatLakes w6848 of 24,000

**Opener (Opus):** `Merge lane-capture and lane-audit if they moved, then
227 — where a squad member's exit spot comes from. The captain's is exact
off the building's ring; 1/28 and 1/29 are not candidates of that sweep
from either centre. Three suspects: find_nearby_spot's squad flag
(param_13, which inflates the block radius at 61deb8 and come_out passes
0), the local Group come_out clears, and a formation offset inside
set_new_location. Read those three; att-capture has the screen.`

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

227. **Where a squad member's exit spot comes from** (CITIES §6.5.1). Great
    Lakes 6848 is the Archers born 144 units off, not the march: the captain
    is exact, `1/28` and `1/29` are not candidates of the building's ring
    from either centre, and `1/29`'s 11.25° bearing is one the sweep never
    makes. Three suspects: `find_nearby_spot`'s squad flag (`param_13`,
    `61deb8`), the local `Group` `come_out` clears, `set_new_location`'s
    formation offset. The squad chain is `UnitData +0x8e`/`+0x90`, **not**
    the `up`/`down` the dump prints — that is the cell list, buildings and
    all. Falsified by a second map's squad birth — with `att-capture`.
    **Takes 219**, `do_group_move`'s four seams (ORDERS §15): the group's
    speed pair and its cap, the flock an invalid slot near an ocean cell
    adds (**one sync draw**, the one with teeth), `cavarch_fight`, the
    attack hand-offs. 7448 (214) is the same shape.

226. **`cover=1` page-faults under free Wine** at `7BF21139` (ORACLE, "Off
    CrossOver"); function coverage, and so the blind list, waits on it.
    **With `att-capture`**: the discriminator run first (DECISIONS 33).

220. **§13's twenty range blocks.** Every free-upgrade row whose candidates
    are a run of tech indices — Chinese herbal lore, the Red Fort's two, the
    four `TwoPreq` blocks, the temple and taxation lines, the wonders — is
    unloaded, and no capture reaches one (TECH §13).

209. **The other once-per-game events a dump install swallows.** 207's
    general half: grep the gates of that shape — a `set_*` whose **return
    value** drives an irreversible record — and seed each from the installed
    state, as `Sim::seed_new_rares_from_fog` does. **Takes 224**, AI §29.4's
    other three `mil_trainers` writers: a trainer that changes city, upgrades
    in place or is captured is filed by neither.

203. **`Wall::mark_behind_tiles`' `0x4`.** The residue is **32** at 4802 and
    **45** at 5565, every bit `0x4` — `set_behind@006b4230`'s low arm, which
    nothing reads (ROADS §9.5). (192) **A merchant has never been seen to
    unpack.** run68 ends eleven frames
    short of `1/19`'s deploy spot (MERCHANT §7) — the cast, the `PLACED`
    two-by-two, `unit_masks & 0x80000`, `do_gather`'s pair; a `[6730, 6800)`
    window buys it all. **Takes 184.** (216) **`create_buildings` offers a
    gather building the original does not**: 6582's slots 1 and 4 carry
    `t 418 cat 4 val 41500` against empty; inert, so the *list* differs.

169. **`compute_site_stats`' arithmetic**, 7,122 of run63's 27,000 site
    fields: an extra site drags every `rank` (AI §2.13 6–12); run59 says it
    250 frames earlier. (172) **The `bucket` pair 165 leaves behind**: one
    apart on 5002 and 5061 in run60's curve, rates and income exact; takes
    155.

158. **The sweep runs for a human leader; this crate skips it** (AI §23.1).
    Moving the gate from `Sim::strategy_all` into `production_ai` seeds a
    **human army** at 16; find the gate between steps 13 and 16 first.

159. **The `CITY` record's two smaller seams**, on run58, both pinned:
    `1/2007`'s `land`/`filled` from 1819 and `space[0..2]` from 1976, one
    apart; from 2576 the gatherers, 11/0 against 10/1 — step 2/10.

146. **The other nine national arms of `train_time`** after the ramp, in
    `006508c0`'s order (PRODUCTION, "The tail's first caller"); only the ramp
    is measured. (142) **`World::tregion` is not `get_tregion`** — PATHFINDER
    §15, §16; eleven callers unaudited.

Two records with no reader: (122) **a draw with no mark of its own** — 16 of
61 `rng.roll`/`get` calls have no `self.mark(`; (153) **the `TRIBE` record,
whole** — `Tribe::log_data@006f0d70`'s `graft[352]`, `barbarian`,
`build_continent`, `people`, `text_substitute` (TECH).

178. **The danger map's unit pass, unexercised.** DANGER §8: no capture has
    a military unit on a frame divisible by 200, so `role & 0x10000`,
    `(attack · 5) / 10` and the war gate rest on the reading; a `DUMP_ALL`
    `WORLD` block on such a boundary settles it. (181) **§7's unreached
    arithmetic** (CARAVAN §7.2–§7.3): the arrival box, `compute_trade`'s
    `× 16 / 2`, `new_caravan`'s `(epoch[1] + 1) · 10`.

**The widening ledger is built** (87, DATALAYER §4): **23** parsed fields
`diff.rs` names nowhere and **41** one capture names, pinned and falling only;
its blind spot is a field the parser never had — `avg_speed` (210).

The ledgers still uncounted: (88) **the blind list** — 101 of 617 cited
functions never entered; pin it as a floor, put it in each Coverage; (72)
every `+0xNN` a document pins vs its module — **with `att-audit`**; (89) the
instrument's last guard, (c) alone; (35) **`mylos` as a cache**, VISION §7.

175. **The uber chain past its birth.** `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3) and nothing else reads the chain:
    `curr_uber_size@0060a760` unwritten, `build_sim` leaves `o_up`/`o_down`
    unset — and **227 may need it**. Takes 48's chain: (48) **the object
    chain, whole**, only units threaded (COLLISION §3, §7), `down`/`down_who`
    uncompared; (73) **`UnitData::group`** — no back-pointer, so
    `Group::normalize`'s cull (GROUPS §4.3) is unmodelled, run33's scout
    65 → 64 on 96; (56) **the cell's `BUILDING` bit** on `(52, 22)`, ARMY §13.

117. **Two seams the census windows now measure.** ATTRITION, "Territory":
    the temple and fort border levels, the wonders, a gem rare, the handicap,
    `reg_forts`; and AI §2.1's `check_explore` answers the grid whole,
    900 v 36/19.

Seven measured one-liners: (23) **the formation byte's sign, the Echelon
half** — `reverse`'s *displacement* is read only on GROUPS §6.4's Echelon
rows, every captured group a Line, run52 the blocker; (103) a woodcutter's
clock at **1,686** — `0/2` waits 445 against 480, ORDERS §6.4; (105/45)
**Gaia's positions**, East Indies' half — run39 191,876 of 192,504, first bad
**1658**, SYNC §4.2; (124) **the loop flag is per animation file**, ANIM §3.3
— 42 `<UNIT>` entries non-looping, 38 looping, carry it on `Art`; (116) **the
one `SITE` slot still wrong**, AI §18 — a 5×5 slide keeps its centre; (166)
**`resource_cap` on five goods, 2958–2959** — 1392 here, 2000 there (ECONOMY,
"The commerce cap"); (107) **`epoch[0]` is the Military level and `army.rs`
reads `ages`**, `army.rs:769,1365`.

161. **The make-list block is 2,500 frames behind the word.**
    `create_buildings` first runs on East Indies 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's arm, `oil_patches.count` and
    `compute_largest_gather@0066e920` all wait on it. (195)
    **`find_repair_spot` is the last of `build_done`'s three**, a seam on all
    four arms (ORDERS §5.5, §5.6, §5.9); no capture has a damaged building.

156. **`STARTING_GOODS` arrives with the age.** `Leader::gain_tech` pays
    `bucket_add(g, game->starting[g])` for a zero bucket whose preq is the
    tech just gained, so the original holds 0 knowledge, metal and oil through
    Ancient where this holds 100 (COSTS). (167) run61's two: the landing
    search's *cell* and `Unit::do_strafe` (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT, "The river halves…"):
    `unit_masks & 0x10`, set and cleared inside a frame; `has_general(0,
    0x162)`'s siege doubling; the group cap, whose `movement::group_capped`
    is uncalled — now 219's second half.

228. **Split the diff harness by record.** `rondata/src/diff.rs` is
    23,243 lines, touched in 121 of the last 197 commits, and both lanes
    land in it. Mechanical, one commit, test count identical (DECISIONS 33).

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
- **Start of session:** merge `lane-capture` and `lane-audit`, then "Where
  things stand", the item, then its document. Lanes never write this file.
- **Run the diff suite with `--release`** — 92 s against debug's 285 s
  since run53's 24,000 frames. A long wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a brief, an agent definition, or a memory hook.
