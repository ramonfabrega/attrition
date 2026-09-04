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

*2026-09-05, Fable orchestrating — four Opus seats, no score moved yet.*
**Both words unmoved: Great Lakes 6848, East Indies 7448.** Each item is
a fresh Opus worker cut off this branch's tip; this seat merges, files
and steers (DECISIONS 33).

- **`att-loop`**, 227 (Great Lakes): the squad member's exit spot.
- **`att-loop-ei`**, 7448 (East Indies): the collision gate and wait
  guard (231), then `line_ok`'s lifecycle (230), both audit-verified.
- **`att-capture`**: 226 answered — `cover=1` dies in the wow64 bop,
  Rosetta by inference, falsifier built and costed (ORACLE); run77, the
  East Indies squad birth at FRAME 10188, in flight; the corpus is its.
- **`att-audit`**: 34 rows adjudicated (ROADS, MOVEMENT, COLLISION,
  MERCHANT, ANIM R1), zero struck; the no-writer guard is in
  (`rondata/src/writers.rs`); CITIES, ORDERS, ECONOMY next.
- **Only this seat writes this file**; a worker's prompt is its item's
  text plus the five gates, it deletes its item and writes the journal;
  lanes write audit files, coverage lines and tests. The harness is
  split (228): a test goes in `diff/<record>.rs`.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7448 of 24,000 · GreatLakes w6848 of 24,000

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
    all. Falsified by run77, the second map's squad birth. **Takes 219**,
    `do_group_move`'s four seams (ORDERS §15; the flock's **one sync draw**
    is the one with teeth). **With `att-loop`.**

231. **The collision gate is three rules** (COLLISION §4.3, §6 step 5):
    the original short-circuits kinds 0 and 0xc *before* the action test
    and gates `1,2,3,4,0x12,0x13,0x15` on it (`detect_unit_collision@
    00617060:366`); the document says "the last four"; `same_group_soft`
    adds a `GroupMove` test; the wait guard grants where a negative
    `collide_o` refuses. **With `att-loop-ei`.**

230. **`line_ok`'s whole lifecycle is wrong** — `unit_masks & 8` against
    `Unit::line_ok`, 335 of 450 on run65's window, pinned to fall
    (`diff/unit.rs`); cleared on a refused step (`move_step@005faf30`),
    set on standing units; gates `do_move`'s re-path. **With `att-loop-ei`.**

232. **`Levels::for_player` ignores its player**, answering a constant
    whose `taxation` is 0: `territory_tax` is 0 % for the life of every game
    and the granary, mill, smelter and university ladders sit at level 1
    (ECONOMY, audit R8); zero on every capture so far because the original's
    is too. Plus four `economy.rs` doc-comments ECONOMY.md has retracted.

226. **`cover=1` dies in the wow64 bop at the exe's entry**, layout-sensitive
    and reproduced by `tools/trace/wow64bop.c` (ORACLE, "The falsifier for
    226, costed"); Rosetta is the inference, an x86 host the falsifier.

220. **§13's twenty range blocks.** Every free-upgrade row whose candidates
    are a run of tech indices — Chinese herbal lore, the Red Fort's two, the
    four `TwoPreq` blocks, the temple and taxation lines, the wonders — is
    unloaded, and no capture reaches one (TECH §13).

209. **The other once-per-game events a dump install swallows** (207's
    general half): a `set_*` whose **return value** drives an irreversible
    record, seeded from the installed state as `seed_new_rares_from_fog`
    is. **Takes 224**, AI §29.4's other three `mil_trainers` writers: a
    trainer that changes city, upgrades in place or is captured.

203. **`Wall::mark_behind_tiles`' `0x4` is a building *finishing*.** 32 at
    4802, **45** at 4803: the thirteen ring the Market at [223, 227] ×
    [78, 81], `set_behind@006b4230`'s `0x4` alone; nothing in `Roads` writes
    it, nothing reads it. Pinned by count and kind (ROADS §6, 2026-09-05). (192) **A merchant has never been seen to
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

Two records with no reader: (122) **a draw with no mark of its own**, 16 of
61 `rng.roll`/`get` calls without `self.mark(`; (153) **the `TRIBE` record,
whole** — `Tribe::log_data@006f0d70`'s five fields (TECH).

178. **The danger map's unit pass, unexercised** (DANGER §8): no capture has
    a military unit on a frame divisible by 200, so `role & 0x10000`,
    `(attack · 5) / 10` and the war gate rest on the reading; a `DUMP_ALL`
    `WORLD` block on such a boundary settles it. (181) **CARAVAN §7.2–§7.3's
    unreached arithmetic**: the arrival box, `× 16 / 2`, `(epoch[1] + 1) · 10`.

**The widening ledger is built** (87, DATALAYER §4): **22** parsed fields
the harness names nowhere and **42** one capture names, each pinned one way;
its blind spot is a field the parser never had — `avg_speed` (210).

The ledgers still uncounted: (88) **the blind list** — 101 of 617 cited
functions never entered; pin it as a floor, put it in each Coverage; (72)
every `+0xNN` a document pins vs its module — **with `att-audit`**; (89) the
instrument's last guard, (c) alone; (35) **`mylos` as a cache**, VISION §7.

175. **The uber chain past its birth.** `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3), nothing else reads the chain,
    `build_sim` leaves `o_up`/`o_down` unset — **227 may need it**. Takes
    (48) **the object chain, whole** (COLLISION §3, §7), `down`/`down_who`
    uncompared; (73) **`UnitData::group`**, no back-pointer, so `Group::
    normalize`'s cull is unmodelled (run33's scout 65 → 64 on 96); (56)
    **the cell's `BUILDING` bit** on `(52, 22)`, ARMY §13.

117. **Two seams the census windows measure** (ATTRITION, "Territory"): the
    temple and fort border levels, the wonders, a gem rare, the handicap,
    `reg_forts`; and AI §2.1's `check_explore` answers the grid whole, 900 v 36/19.

Seven measured one-liners: (23) **the formation byte's sign, Echelon half**,
read only on GROUPS §6.4's Echelon rows, run52 the blocker; (103) a
woodcutter's clock at **1,686**, `0/2` waits 445 against 480 (ORDERS §6.4);
(105/45) **Gaia's positions**, East Indies, first bad **1658** (SYNC §4.2);
(124) **the loop flag is per animation file** (ANIM §3.3), carry it on `Art`;
(116) **the one `SITE` slot still wrong** (AI §18); (166) **`resource_cap` on
five goods, 2958–2959**, 1392 v 2000 (ECONOMY); (107) **`epoch[0]` is the
Military level and `army.rs` reads `ages`**, `army.rs:769,1365`.

161. **The make-list block is 2,500 frames behind the word**:
    `create_buildings` first runs on East Indies 9982 (AI §25), so
    `building_value`, `gather_value`, §24.4's arm, `oil_patches.count` and
    `compute_largest_gather@0066e920` wait on it. (195) **`find_repair_spot`
    is `build_done`'s last of three** (ORDERS §5.5–§5.9); no damaged building.

156. **`STARTING_GOODS` arrives with the age**: `gain_tech` pays
    `bucket_add(g, starting[g])` for a zero bucket whose preq is the tech
    just gained, so the original holds 0 knowledge, metal and oil through
    Ancient where this holds 100 (COSTS). (167) run61's two: the landing
    search's *cell*, `Unit::do_strafe` (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT, "The river halves…"):
    `unit_masks & 0x10`, set and cleared inside a frame; `has_general(0,
    0x162)`'s siege doubling; the group cap, whose `movement::group_capped`
    is uncalled — now 219's second half.

229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5), pinned off run17/run44's own
    `last_time`, 26 of 26. `unit_masks2` has no field here; the arm in
    `Sim::guy_inc_time` waits on a word that reaches a melee frame.

Older backlog: (39) a read-only 2D viewer over `Sim` with the dump overlaid;
(41) `scenario.py`'s fate — fuzz `424242 1000 1300`, the blind report, then
delete; a `find_target` block; run7's order stream; a mounted attacker;
`calc_gather` non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

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
