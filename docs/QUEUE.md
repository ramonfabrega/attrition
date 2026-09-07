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

*2026-09-07, Fable steering — the tranche is real and **both words
moved**: Great Lakes 6848 → **7176** and East Indies 7448 → **7529** over
thirteen landings, all merged, all Opus. Verdicts below; Opus resumes.*

- **Great Lakes 7176** (leads): 97 draws differing at 93 — the original in
  `Unit::do_idle+0x7d`, this crate in `Guy::inc_time+0x271`; a unit goes
  idle there and not here, and run79's `[6910,7250]` covers it. **East
  Indies 7529**: a `Guy::set_anim` under `Unit::do_non_flat_gather+0xb99`
  the original spends and this does not.
- **The gate is pinned to two threads** — `.cargo/config.toml`, guarded in
  `floors.rs`. The same suite is 15 GB at two and 24–33 GB at the default
  sixteen; memcap killed two runs this tranche and a `| tail` laundered one
  137 into exit 0. **No pipe between `memcap.sh` and the exit code.**
- **The stopping rule stands at two, with its exception named** (DECISIONS
  34): a no-score landing that pins the headline's cause with a
  failing-first assertion on the frame resets the count. 249 → 250 was it.
- **Two rules went to CLAUDE.md**: widen the whole cast before reading the
  candidate (249, 250); a word moved lands with the value diff beside it.
- **258 is booked beside the headline** at width two: the finish frame's
  units-off on both maps, a third scoreboard line that may only fall.
- **Reap before spawning**: nine finished worktrees on disk (loop-117, 156,
  234, 236–239, att-capture, att-capture2), nothing unmerged — `ccc rm`.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7529 of 24,000 · GreatLakes w7176 of 24,000

**Opener: commander on Opus (DECISIONS 34). Great Lakes 7176 is the item —
widen run79's `[6910,7250]` whole before naming a mechanism — with 258 at
width two beside it. Reap the nine worktrees first.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

246. **Step 6's repath is diff-backed on one capture only** — run83's
    single event (239); run85's obstacle class is not a second.

258. **Booked (steer 09-07): the finish line's own frame is on disk, free**
    (252) — whole-map states at **24001 on both scored maps** (run53;
    run21/23/54), East Indies at 15105/15401/16007/16489. Diff each against
    the sim run to its frame and pin **units off / unlinked at 24001** per
    map as a third scoreboard line that may only fall; (259) run29's 15105
    already reads **51 off, 22 unlinked**. Harness only, no mechanic.

226. **`cover=1` dies in the wow64 bop** (ORACLE), an x86 host the
    falsifier. (220) TECH §13's twenty range blocks. (209) **the
    once-per-game events a dump install swallows** — a `set_*` whose
    **return value** drives an irreversible record; **takes 224**. (203)
    `mark_behind_tiles`' `0x4` is a building *finishing*. (240) the
    merchant's `gather_down`/`special` −1 at 6929; **takes 184**. (216)
    `create_buildings` offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945; run79
is the cheapest second capture for 237's rows. (245) `focus.sh` matched a
concurrent worker's shell on run84.

255. **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
    never run** — every joiner on disk is farther than `0x480` from its
    army's `get_unit(0)`; the falsifier is one born beside its army.
    (256) `come_out`'s three `action_move_to` sites stay unreached.

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's **6737**, three Archers going guy **170 → 177** keeping
    `(who, o)` and `group 64`, not a modifier on a type; East Indies'
    `1/32` does 340 → 341, danger row moving by `(110 − 100) / 2`. (181)
    CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4, §4.1): **19** fields the
harness names nowhere, **44** one capture names; blind spot `avg_speed`
(210) — the ledger counts *fields* and cannot see a record the parser
never visits, which is what hid 252's dumps. Uncounted: (88) the blind
list, 101 of 617; (72) every `+0xNN` a document pins vs its module, **with
`att-audit`**; (89) the guard; (35) VISION §7.

257. **Nineteen kept tests compare a torn block** (252): a closing dump is
    frame n except on the one unit the quit caught mid-update, which sits
    on n−1, and every nested archive's last `FRAME n` body *is* its
    closing dump. No live case found — a tear bites only if a unit's first
    divergence lands on the last frame — but unaudited. Fix is
    `compare_shutdown`'s n−1 allowance, given to those last blocks.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
    COLLISION §3/§7; (73) `UnitData::group`'s back-pointer; (56) ARMY §13.

248. **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants
    a retelling pass before anything else is added to it. Territory's
    remaining terms are closed — handicap, temple and fort are unreachable
    in any game that runs here and ATTRITION has the reasons; read it
    before re-booking any. (AI §2.1's `check_explore`, 900 v 36/19.)

Measured residues, none near a word: (169) `compute_site_stats`, 7,122 of
run63's 27,000 site fields; (172) the `bucket` pair on 5002/5061; (158)
the human-leader sweep (AI §23.1); (159) the `CITY` record's two seams;
(146) `train_time`'s nine national arms; (142) `World::tregion`
(PATHFINDER §15–16); (122) 16 of 61 draws without a `self.mark(`; (153)
`TRIBE`; (20) the `Census` rows; (23) the formation byte's sign, Echelon
half (GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock, 445 v
480 (ORDERS §6.4); (105/45) Gaia's positions, first bad 1658 (SYNC §4.2);
(124) the loop flag is per animation file; (116) the one `SITE` slot (AI
§18); (166) `resource_cap` on five goods (ECONOMY); (107) `epoch[0]`.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

253. **`1/13` parts at 6938**, a citizen at `myspeed 25` — run86's only
    other divergence, and run82's 6946 dump shows it off by (11,7). A
    diff, not a capture. (254) `come_out`'s ring has one sample (7284)
    and wants a second disembark. (167) run61's two (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action.

229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5); the arm waits on a melee frame.

251. **`memcap.sh` has no fixture and a real hole** (lore): its `ps | awk`
    reads `0 0` when `ps` answers nothing, so a refused sample reads as no
    memory used and the ceiling never fires — Friday's crash by another
    door, under the guard DECISIONS 34 cites for width two. Wants one:
    past the cap, dead in N seconds, exit 137. Its sibling landed as
    `tools/pollguard.py`, live once a checkout's `tools/` holds it
    (`$CLAUDE_PROJECT_DIR` is the repo root, never a worktree).

Older backlog: (235) the capture `String` each test reads, ~5 GB of the
suite's 10 — lore's shape: tests over one dump share one parsed instance,
so memory scales with distinct dumps, not threads; no score; (39) a 2D viewer over `Sim`; (41) `scenario.py`; a
`find_target` block; run7's orders; a mounted attacker; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items to the journal. **The
  200-line bound is a budget** — a new item is paid for by compressing old
  ones, and you count the lines before writing, not after.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together.
- **Start of session:** merge the lanes, then "Where things stand", the
  item, its document. Lanes never write this file.
- **A brief names `docs/audit/2026-09-05-<doc>-vs-code.md`** where one
  exists — candidate items, taken on the way through.
- **Overlap a capture's neighbours on purpose.** A window is priced by its
  dumped blocks and the run-up is free, so the overlap costs seconds and
  turns "same seed, therefore same game" into a state check where `rngcmp`
  is only a word. Six blocks each end is the floor (run83); when a
  neighbour **contains** yours it is free and total (run84, 80 of 80) with
  `--exclude <the category you raised>`. **The trap**: `samegame.py` exits
  0 when nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the mechanic's SHAPE and the GATE — not
  only for coverage of a frame.** `BUILDQUEUE` prints
  `queue[scan].cost[0..2]`, so a price the original *paid* is on disk
  wherever `BUILDS` is on (238); 178's two passes write different
  footprints, so a half-cell with no building in its 3×3 isolates the unit
  pass in archives already held. And a term zero in every dump is either
  zero-valued or switched off, which cost very different things to reach:
  117 and 178 were both booked on the first and were the second.
- **A check on a field never cleared must assert a CHANGE, not a value** —
  `collide_frame` is a permanent stamp, so "some unit has one in the band"
  is true of any window, and run85's first teeth check passed on a band
  where nothing happened. Test both directions on real data first.
- **`stat -t '%FT%TZ'` prints local time under a literal Z** — use `%z`;
  a self-consistent before/after pair can be uniformly wrong.
- **A commander's clear is free when**: every landed branch merged, gated
  and pushed; nothing in flight owing its result elsewhere; this handoff
  current with the headline *measured*; nothing unfiled. `ccc clear <ref>
  --then "continue"` arms it, and **0.1.29 fixed the gate** 0.1.28 stalled
  on a Monitor or background shell. A clear is not a restart: both kinds
  survive it tracked *and* writing (`~/ccc-stream/clear-probe.txt`).
- **Run the diff suite with `--release`** under `memcap.sh`, redirected to
  a file — a pipe launders its 137 — and keep the `peak` line; a long wait
  is the capture, not a hang. **Before a blind fan-out**, `grep -n <mechanic> CLAUDE.md` and
  the memory index — a subagent inherits both, and neither they nor a
  brief nor an agent definition may quote this file or the journal.
