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

*2026-09-07, Opus commanding — **both words moved**: Great Lakes 6994 →
**7176** (250, +182) and East Indies 7448 → **7529** (241). The stopping
rule fired at three no-score landings and reset on the fourth; 252 is live.*

- **Great Lakes 7176**: 97 draws differing at 93 — the original in
  `Unit::do_idle+0x7d`, this crate in `Guy::inc_time+0x271`. A unit goes
  idle there and not here; run79's `[6910,7250]` covers it.
- **East Indies 7529**: a `Guy::set_anim` under
  `Unit::do_non_flat_gather+0xb99` the original spends and this does not.
- **Widen the record before reading the candidate** (249, 250): twice the
  named mechanism was wrong and eighty `UNITDATA` blocks said so in twenty
  minutes — 6994 was `add_to_army`'s walk, never `come_out`. And **a
  draw-stream gain is not a correctness proof**: 182 frames of exact draws
  came with an anchor two tiles wrong, because that tile refused the same
  lines. Diff the value, not the word.
- **Take the gap whole, not sampled windows** (241): run86 covered
  `[6924,7410)` in one run where two were booked, and its eject at 7284
  predates the booked window. A reading can miss a scalar for geometry.
- **Loop hygiene**: width two on the gate (15.8 GB of 20 GiB); `ccc
  update <ref>` for a base, never a raw merge; **reap at merge time** —
  `done idle` is not reaped, and 0.1.29's `ccc rm` takes worktree and
  branch. A lane is cheap on tokens only if it waits **blocking** (251b);
  each `stalled` line in `~/ccc-stream/` still wants a word (25).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7529 of 24,000 · GreatLakes w7176 of 24,000

**Opener: commander on Opus (DECISIONS 34). `loop-252` is live and alone;
the screen is free. A Fable steering pass is owed once it lands.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

246. **Step 6's repath is diff-backed on one capture only** — run83's
    single event (239); run85's obstacle class does not serve as a second.

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

AI residues, measured, none near a word: (169) `compute_site_stats`'
arithmetic, 7,122 of run63's 27,000 site fields (AI §2.13); (172) the
`bucket` pair one apart on 5002/5061; (158) the human-leader sweep (AI
§23.1); (159) the `CITY` record's two seams on run58; (146) `train_time`'s
nine national arms; (142) `World::tregion` (PATHFINDER §15–16); (122) 16
of 61 draws without a `self.mark(`; (153) `TRIBE`; (20) the `Census` rows.

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's **6737**, three Archers going guy **170 → 177** keeping
    `(who, o)` and `group 64`, not a modifier on a type; East Indies'
    `1/32` does 340 → 341, danger row moving by `(110 − 100) / 2`. (181)
    CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4): **19** fields the harness
names nowhere, **44** one capture names; blind spot `avg_speed` (210).
**(252) the shutdown dump**: `GameLog::end_game` writes a whole-map state
at `FRAME`'s own indent — a *sibling* the frame walk never reached — and
`Log::final_state` reads it now, but **one** capture uses it where every
windowed capture that quit has one. run82's gave two divergences no test
had seen. The cheapest widening on the board, unbounded until swept.
Uncounted: (88) the blind list, 101 of 617; (72) every `+0xNN` a document
pins vs its module, **with `att-audit`**; (89) the guard; (35) VISION §7.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3), nothing else reads it and
    `build_sim` leaves `o_up`/`o_down` unset. Takes (48) COLLISION §3/§7;
    (73) `UnitData::group`'s back-pointer, **242's half**; (56) ARMY §13.

248. **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants
    a retelling pass before anything else is added to it. Territory's
    remaining terms are closed — handicap, temple and fort are unreachable
    in any game that runs here and ATTRITION has the reasons; read it
    before re-booking any. (AI §2.1's `check_explore`, 900 v 36/19.)

Seven measured one-liners: (23) the formation byte's sign, Echelon half
(GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock at 1,686, 445
v 480 (ORDERS §6.4); (105/45) Gaia's positions, East Indies, first bad
1658 (SYNC §4.2); (124) the loop flag is per animation file (ANIM §3.3);
(116) the one `SITE` slot still wrong (AI §18); (166) `resource_cap` on
five goods, 1392 v 2000 (ECONOMY); (107) `epoch[0]` is Military.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value`, §24.4's arm and `compute_largest_gather@0066e920` wait
    on it. (195) `find_repair_spot` is `build_done`'s last of three.

253. **`1/13` parts at 6938**, a citizen at `myspeed 25` — run86's only
    other divergence, and run82's 6946 dump shows it off by (11,7). A
    diff, not a capture. (254) `come_out`'s ring has one sample (7284)
    and wants a second disembark. (167) run61's two (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action.

229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5), 26 of 26 off run17/run44; no field
    here, and the arm waits on a word that reaches a melee frame.

251. **The guard the loop leans on and does not have** (lore). (a)
    `memcap.sh` has no fixture and a real hole: its `ps | awk` reads
    `0 0` when `ps` answers nothing, so a refused sample reads as no
    memory used and the ceiling never fires — Friday's crash by another
    door, under the guard DECISIONS 34 cites for width two. Wants one:
    past the cap, dead in N seconds, exit 137. The poll guard that stood
    here has landed as `tools/pollguard.py` — it activates in a checkout
    once `tools/` holds it, because `$CLAUDE_PROJECT_DIR` resolves to the
    repo root and never to a worktree.

Older backlog: (235) the capture `String` each test reads, ~5 GB of the
suite's 10 — a rounded-up read buffer at ~100 call sites, no score; (39) a
2D viewer over `Sim`, dump overlaid; (41) `scenario.py`'s fate; a
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
- **Grep for the derived quantity, and for the mechanic's SHAPE — not
  only for coverage of a frame.** `BUILDQUEUE` prints
  `queue[scan].cost[0..2]`, so a price the original *paid* is on disk
  wherever `BUILDS` is on, which retired a booked window (238). And 178
  needed no capture: its two passes write different footprints — the
  building pass a 3×3 to every viewer including the owner, the unit pass
  one half-cell and never the owner's — so a half-cell with no building
  in its 3×3 isolates the unit pass in archives already held.
- **Grep for the GATE, not only the value.** A term zero in every dump is
  either zero-valued or switched off, and those cost very different things
  to reach: 117 and 178 were both booked on the first and were the second.
- **A check on a field that is never cleared must assert a CHANGE, not a
  value** — `collide_frame` is a permanent stamp, so "some unit has one in
  the band" is true of any window on the game, and run85's first teeth
  check passed on a band where nothing happened. Test both directions on
  real data before the capture runs.
- **A timestamp wearing a `Z` it has not earned** survives every check
  comparing only its own two sides: `stat -t '%FT%TZ'` prints *local* time
  and appends a literal Z (`%z` is the fix). A self-consistent
  before/after pair can be uniformly wrong.
- **A commander's clear is free when**: every landed branch merged, gated
  and pushed; nothing in flight owing its result elsewhere; this handoff
  current with the headline *measured*; nothing unfiled. `ccc clear <ref>
  --then "continue"` arms it, and **0.1.29 fixed the gate** 0.1.28 stalled
  on a Monitor or background shell. A clear is not a restart: both kinds
  survive it tracked *and* writing (`~/ccc-stream/clear-probe.txt`).
- **Run the diff suite with `--release`**; a long wait is the capture, not
  a hang. **Before a blind fan-out**, `grep -n <mechanic> CLAUDE.md` and
  the memory index — a subagent inherits both, and neither they nor a
  brief nor an agent definition may quote this file or the journal.
