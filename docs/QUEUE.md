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

*2026-09-06, Opus commanding a nine-landing loop at width two — **Great
Lakes 6862 → 6982 → 6994**, suite 220. East Indies **7448 unmoved**, but
241 found its cause and it is not a collision.*

- **Great Lakes 6994 is the headline**, unmeasured beyond this: at draw 2
  the original spends `Unit::do_move+0xe84` and this crate spends nothing
  (ours 2, theirs 4). run84 `[6950, 7029]` at `LEADERS=9` covers it.
- **East Indies 7448 is a position, not a predicate** (241): `1/20` is off
  by (36, 792) at run85's *first* block, ~34 frames of its own walk, so it
  is never there to meet the animal. The gap is a **transport ride**
  between run82's 6929 and 7400 — barge birth 7093 on both sides, eject
  spends no draw. TRANSPORT §13; nothing covers 6930–7399.
- **Two commander hypotheses were refuted in a row** (239's deflection,
  241's `who < 8` fence), both killed by going to the dump. Brief the
  *measured seam*, not the mechanism you suspect behind it.
- **Width two on the gate** (14.8 GB each; four threads stay off). `ccc
  update <ref>` for a base, never `git merge`; reap at merge time. `ccc
  rm` leaves the branch, so reusing that `--worktree` name fails.
- **The capture lane is per-item now**: cycling cost nothing and the fresh
  lane closed 178 by grep rather than spend a drafted 154 MB stanza.
- **Owed to ccc** (their item 25): `~/ccc-stream/` holds `watch.jsonl`,
  `watch-stall10.jsonl` and a 60 s `list.jsonl` sampler; each `stalled`
  line wants a word in `verdicts.txt`. Restarting `ccc watch` blinds it.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7448 of 24,000 · GreatLakes w6994 of 24,000

**Opener: the main thread is the commander (DECISIONS 34), on Opus. Two
repo slots are free and both maps have a named seam — Great Lakes 6994,
East Indies' transport gap. `att-capture` (23d49f15) is mid-117.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

246. **Step 6's repath is diff-backed on one capture only** — run83's
    single event (239); run85's obstacle class differs and does not serve
    as the second.

226. **`cover=1` dies in the wow64 bop** (ORACLE), an x86 host the
    falsifier. (220) TECH §13's twenty range blocks, unloaded. (209) **the
    once-per-game events a dump install swallows** — a `set_*` whose
    **return value** drives an irreversible record; **takes 224**. (203)
    `mark_behind_tiles`' `0x4` is a building *finishing* (ROADS §6). (240)
    **the deployed merchant's record** — `gather_down`/`special` −1 at
    6929 against run76's 18 and 6; grep what they count before booking a
    window; **takes 184**. (216) `create_buildings` offers a gather
    building the original does not, 6582's slots 1 and 4.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945; run79
is the cheapest second capture for 237's rows. (245) `focus.sh` matched a
concurrent worker's shell on run84.

AI residues, measured, none near a word: (169) `compute_site_stats`'
arithmetic, 7,122 of run63's 27,000 site fields (AI §2.13); (172) the
`bucket` pair one apart on 5002/5061; (158) the sweep runs for a human
leader, skipped here (AI §23.1); (159) the `CITY` record's two seams on
run58; (146) `train_time`'s nine national arms (PRODUCTION); (142)
`World::tregion` is not `get_tregion` (PATHFINDER §15–16); (122) 16 of 61
draws without a `self.mark(`; (153) the `TRIBE` record; the thirteen
`Census` fields written for consumers not built (20 rows, may only fall).

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's frame **6737**, the three Archers going guy **170 → 177**
    keeping `(who, o)` and `group 64`, not a modifier on a type; East
    Indies' `1/32` does 340 → 341 and its danger row moves by exactly
    `(110 − 100) / 2`. Anything reading cost, attack or speed across an
    upgrade wants it. (181) CARAVAN §7.2–§7.3's unreached arithmetic.

**The widening ledger** (87, DATALAYER §4): **19** fields the harness
names nowhere, **44** one capture names (237 moved both); its blind spot
is a field the parser never had, `avg_speed` (210). Uncounted: (88) the
blind list, 101 of 617; (72) every `+0xNN` a document pins vs its module,
**with `att-audit`**; (89) the instrument's last guard; (35) VISION §7.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3), nothing else reads the chain and
    `build_sim` leaves `o_up`/`o_down` unset. Takes (48) the object chain
    whole (COLLISION §3, §7); (73) `UnitData::group`'s missing
    back-pointer — **242 is its other half**; (56) ARMY §13's cell bit.

117. **Territory's remaining terms are out of reach, the handicap
    permanently.** The gem term landed (43 cells, run80). Temple and fort
    are blocked at their prerequisite in every game on disk. The handicap
    is **multiplayer-only**: `compute_reg_territory:255` needs
    `Game::semaphore` bit 2, set only by `run_gamespy@00587060` and
    *cleared* by `run_solo`/`run_scenario`/`run_editor`/`read_package`, so
    no capture and no scripted setup reaches it. Two corrections came with
    it: the term is `(get_handicap() + 15) / 25` and `get_handicap`
    returns `DATA` = index × 5, so it runs **0–4**; and
    `LeaderData::handicap` is a catch-up **deficit**
    (`init_handicaps@0058abf0`), not `PLAYERn_HANDICAP`. **248**:
    ATTRITION's Territory section is 15,625 bytes of 16,000 and wants a
    retelling pass before anything else is added. (AI §2.1's
    `check_explore` answers the grid whole, 900 v 36/19.)

Seven measured one-liners: (23) the formation byte's sign, Echelon half
(GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock at 1,686, 445
against 480 (ORDERS §6.4); (105/45) Gaia's positions, East Indies, first
bad 1658 (SYNC §4.2); (124) the loop flag is per animation file (ANIM §3.3);
(116) the one `SITE` slot still wrong (AI §18); (166) `resource_cap` on five
goods, 1392 v 2000 (ECONOMY); (107) `epoch[0]` is Military, `army.rs`
reads `ages`.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value`, §24.4's arm and `compute_largest_gather@0066e920` wait
    on it — **238 is the Great Lakes half, live now**. (195)
    `find_repair_spot` is `build_done`'s last of three.

241. **East Indies' word, 7448, is a POSITION** — 241 closed the
    collision reading three ways (a step probes the bitmask, which
    `coll_paint` writes with no owner test; `detect_unit_collision`'s walk
    never asks `who`; the dump reads `collide_who 8`). `1/20` is off by
    (36, 792) at run85's *first* block, ~34 frames of its own walk, so it
    never meets the animal. The gap is a **transport ride** between
    run82's 6929 and 7400 — barge birth 7093 on both sides, eject spends
    no draw — and **nothing covers 6930–7399**. TRANSPORT §13. (167)
    run61's two: the landing search's *cell*, `Unit::do_strafe`
    (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action.


229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5), 26 of 26 off run17/run44; no field
    here, and the arm waits on a word that reaches a melee frame.

Older backlog: (235) the capture `String` each test reads, ~5 GB of the
suite's 10 — a rounded-up read buffer at ~100 call sites, no score; (39) a
2D viewer over `Sim`, dump overlaid; (41) `scenario.py`'s fate; a
`find_target` block; run7's orders; a mounted attacker; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`. **The 200-line
  bound is a budget: a new item is paid for by compressing old ones.**
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** merge the lanes, then "Where things stand", the
  item, then its document. Lanes never write this file.
- **A brief names `docs/audit/2026-09-05-<doc>-vs-code.md`** where one
  exists: its rows are candidate items, taken on the way through.
- **Overlap a capture's neighbours on purpose.** A window is priced by its
  dumped blocks and the run-up is free, so the overlap costs seconds and
  turns "same seed, therefore same game" into a state check where `rngcmp`
  is only a word. Six blocks each end is the floor (run83); when a
  neighbour's window **contains** yours it is free and total (run84, 80 of
  80) with `--exclude <the category you raised>`. **The trap**:
  `samegame.py` exits 0 when nothing is in common, so assert the common
  count and range, not the verdict.
- **Grep for the derived quantity, and for the mechanic's SHAPE — not
  only for coverage of a frame.** `BUILDQUEUE` prints
  `queue[scan].cost[0..2]`, so a price the original *paid* is on disk
  wherever `BUILDS` is on, which retired a booked `LEADERS=9` window
  (238). And 178 needed no capture at all: its two passes write different
  footprints — the building pass a 3×3 to every viewer including the
  owner, the unit pass one half-cell and never the owner's — so a
  half-cell with no building in its 3×3 isolates the unit pass in
  archives already held, with no contemporaneous dump required.
- **Grep for the GATE, not only the value.** A term that is zero in every
  dump is either zero-valued or switched off, and those cost very
  different things to reach: 117's handicap and 178's unit pass were both
  booked on the first reading and were both the second.
- **A check on a field that is never cleared must assert a CHANGE, not a
  value** — `collide_frame` is a permanent stamp, so "some unit has one in
  the band" is true of any window on the game, and run85's first teeth
  check passed on a band where nothing happened. Test both directions on
  real data before the capture runs.
- **Run the diff suite with `--release`**; a long wait is the capture, not
  a hang. **Before a blind fan-out**, `grep -n <mechanic> CLAUDE.md` and
  the memory index — a subagent inherits both, and neither they nor a
  brief nor an agent definition may quote this file or the journal.
