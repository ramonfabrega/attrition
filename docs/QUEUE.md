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

*2026-09-06, Opus commanding — **Great Lakes 6848 → 6862**, the first
headline the loop moved with no human in it.* East Indies 7448 unmoved.

- **219 is done, and it was neither seam the brief named**: the march's
  halving is `unit_masks & 0x100000`, the one-shot half step a **soft**
  collision leaves behind — set by `detect_unit_collision@00617060` (at
  `617817`), spent by `Unit::move_step@005faf30` (at `5fb1f4`) on the arm
  owing under 45° (ORDERS §15.1). So
  the "26, 13, 26, 26, 13" is two halvings and then 26. `Unit::half_step`
  had sat unread for two days. run79's second squad is the second sample,
  asserted and made to fail on a neighbouring bit. **236** is what stands
  at 6862 and is the default item; no capture needed.
- **235 blocks every worker's gate** — the release suite is 15 GB and 337 s
  serialized. Until it lands: **one worker at a time**, and every release
  run is `zsh tools/memcap.sh 20 cargo test -p rondata --release --
  --test-threads=1` (DECISIONS 34; the Mac went down without it on 09-04).
- **117 split three ways** on run80: the **gem term is live** and
  unmodelled (a diff target, not a capture), the temple and fort terms
  cannot fire in any game on disk, the handicap is 0 by the lobby.
- **A late-game capture is now cheap**: 40 s to reach frame 23,960. The
  hours in run53's stanza were the `cover=1` int3 forest, not the game.
- Lanes: `att-capture` idle, screen free, corpus current at 229 objects;
  `att-audit` held at the user's call while the loop is ironed out.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7448 of 24,000 · GreatLakes w6862 of 24,000

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

235. **The diff suite's memory is the gate's own hazard.** 793 MB of dump
    becomes **7.8 GB** in one test (run71): 2.96 M blocks × two `Vec`s each
    is ~5.9 M small allocations, slack and overhead, on a parse that is
    already zero-copy. Serialized the suite peaks at 13.5 GB, in parallel it
    passes 18 GB and took the machine down once. An arena for fields and
    children, or a frame-lazy `Log`, and a measured peak in the guard's own
    output. Everything else waits on the gate this owns.

236. **The squad meets the citizen — Great Lakes 6862's blocked stand.**
    219's successor and the map's nearest divergence: the original spends
    `Guy::set_anim+0x97a < Unit::move_step+0x823` on 6862 and this crate
    does not. First field to part is `1/29` on 6861 — the original widens
    its `tolerance` to 384 (`manh × 2`, COLLISION §5) and steps a full 26
    to `(42968, 24407)` where this crate steps 22 to `(42971, 24409)`;
    `1/28` then stands on 6862 where the original steps. run76's window
    covers it all.

226. **`cover=1` dies in the wow64 bop at the exe's entry**, layout-sensitive
    and reproduced by `tools/trace/wow64bop.c` (ORACLE, "The falsifier for
    226, costed"); Rosetta is the inference, an x86 host the falsifier.

220. **§13's twenty range blocks** are unloaded — every free-upgrade row
    whose candidates are a run of tech indices (TECH §13); no capture reaches one.

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

AI residues, measured, none near a word: (169) `compute_site_stats`'
arithmetic — an extra site drags every `rank` (AI §2.13), 7,122 of run63's
27,000 site fields, (172) the `bucket` pair one apart on 5002/5061 (run60);
(158) the sweep runs for a human leader, here it is skipped (AI §23.1), the
gate is between steps 13 and 16; (159) the `CITY` record's two seams on
run58, `1/2007`'s `land`/`filled` from 1819 and the gatherers from 2576;
(146) `train_time`'s nine national arms after the ramp (PRODUCTION), (142)
`World::tregion` is not `get_tregion`, eleven callers (PATHFINDER §15–16);
(122) 16 of 61 draws without a `self.mark(`; (153) the `TRIBE` record, whole;
the thirteen `Census` fields the sweep writes for consumers not built
(`rondata::writers`' no-reader ledger, 20 rows, may only fall).

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

117. **The gem term is live and unmodelled** — run80 reads
    `rares_collected[44] = {11, 13, 23, 27}` on Great Lakes, offset 6, so
    **Gems** is collected and territory at 23,999 (266 v 568) is wrong
    without it (ATTRITION, "Territory"). Model it, then the size falls out
    of a capture either side of the Merchant reaching it. The temple and
    fort terms are **blocked at their prerequisite** in every game on disk
    and the handicap is 0 by the lobby: all three want a scripted setup, a
    session's work, not the screen's. (Also AI §2.1's `check_explore`
    answers the grid whole, 900 v 36/19.)

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
    is `build_done`'s last of three** (ORDERS §5.5–§5.9), and the repair gate
    never asks `Building::hit_frame`, written for it; no damaged building.

156. **`STARTING_GOODS` arrives with the age**: `gain_tech` pays
    `bucket_add(g, starting[g])` for a zero bucket whose preq is the tech
    just gained, so the original holds 0 knowledge, metal and oil through
    Ancient where this holds 100 (COSTS). (167) run61's two: the landing
    search's *cell*, `Unit::do_strafe` (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT, "The river halves…"):
    `unit_masks & 0x10`, set and cleared inside a frame; `has_general(0,
    0x162)`'s siege doubling; the group cap, whose `movement::group_capped`
    is uncalled — and 219 established the cap's gate is
    `action_type == 0`, which a marching unit never answers (ORDERS §15.1),
    so it needs a capture with a grouped unit holding no action.


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
