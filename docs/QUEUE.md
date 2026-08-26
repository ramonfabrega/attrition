# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees.

`CLAUDE.md` carries the rules and is inherited by every subagent spawn. This
file carries the **state**: which mechanics are done, what is in progress,
what is owed, and what to take next. It moved out of `CLAUDE.md` on
2026-08-25 (`docs/DECISIONS.md` entry 21) because a blind second reader
inherits `CLAUDE.md`, and a queue that narrates a mechanic's findings
contaminates the reader before the brief arrives. Nothing below may be quoted
back into `CLAUDE.md`.

Two sections, two cadences, and both **subtract**: what is done leaves this
file for `docs/JOURNAL.md`, which is where the story goes.

- **Where things stand** — the handoff. Rewritten from scratch at the end of
  every session: what landed, what is in progress, what is owed, what to
  take next, and anything that needs the user. About twenty lines. Its last
  line is the next session's opener, verbatim.
- **The queue** — the backlog, in dependency order. One entry per mechanic or
  piece of tooling. A struck entry is **one line and a pointer** — to its
  document, its audit, and its journal entry — never a paragraph; the moment
  a struck entry grows a paragraph this file is a changelog again, which is
  how `CLAUDE.md` reached 847 lines. An entry in progress may carry a short
  brief until it is taken.

## Where things stand

*Last verified 2026-08-26, after item 25's last half —
`Leader::produce_building`'s two draw sites and the three defects they
found.* The commit this section was written against is the one that lands
it; if `git log` has moved well past it, trust the queue below and the
journal before trusting this.

**Last landed: `produce_building` draw for draw, and the farm on the
original's own tile.** The queue asked only for the marks. They cost
twenty minutes and turned one number into three bugs
(`docs/AI.md` §2.20, `docs/JOURNAL.md`):

- **The jitter walks a 2×2.** Both loops at `006e2a78` are inclusive, so
  an ordinary building tries four sub-positions and draws for each one
  `blocked_site` clears — four on run20, three on the fuzzed map where one
  is blocked. The code had one.
- **The stride-by-three tested the loop's start index** instead of the
  current one, so it could never engage. Fixing it moved no measured
  number; it is in because the next early-accepting call walks 105 cells
  instead of 40.
- **`WorldData::buildings_allowed` was not modelled.** It is a predicate,
  `(flags & 0x78) == 0` — rock, mountain, forest and the unnamed `0x40`
  take no building — and the world dump has carried those flags all along.
  The sim was scoring forest cells and drawing for them.

**The numbers now.** run20 frame 0 **175/175 draw for draw**, frame 1
**54/53** — and the residue is now a single named thing, the AI scout's
`Unit::do_move+0xe84`; `produce_building` is 39 + 4 on both sides and its
farm lands at `(41856, 39552)`, the run's own `BUILDDATA` to the unit.
Frame 2 **5/5**. The Great Lakes: 120/120, 54/54, 6/6, 7/6, unchanged byte
for byte. The fuzzed map: frame 0 **195/195**, frame 1 **43/45** (was
48/45). `ticks before divergence` is still 1 everywhere. Run6's long pin
was re-based: totals **fell** 1,679/1,199 → 1,552/1,160, the asserted
non-farmer half rose 1,246/879 → 1,267/892 because the farmers' share fell
further.

**Then, in order:**

- **`go_around_building@005fc350`** — item 29, and a mechanic rather than
  a fix. Run20's frame 1, unit `1/0`: the AI scout's line clips a
  *building* three tiles short of a clear goal, so the pull-back has
  nothing to pull. `docs/ORDERS.md` §4.6 has the shape (up to three
  `PathData`s, a recursive `find_path` on the first, `return 0` when it
  verifies). It is the **only** residue left in run20's frame 1 and the
  last `+0xe84` on any capture. Re-measured 2026-08-26 and still one draw:
  the building it clips stands beside the city, not where the farm moved.
- **The fuzzed map's frame 1**, now two rows rather than a lump: one
  `Leader::produce_building+0xc99` **short** (29 against 30 — a spiral
  candidate the original scores and the sim does not, the mirror image of
  the gap just closed) and one `Unit::do_non_flat_gather+0x54b` short.
  `diff::tests::the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well`
  asserts both as they stand, so closing either fails the test.
- **§9.3's sixth citizen** and **the frame-1 order two of player 1's units
  hold and the sim does not** — the rest of item 25; re-read it, both the
  scout half and the `produce_building` half have moved under it.
- **Item 23, the hand-back's inversion** — unchanged, cheap, unblocked.
- **Re-run run13's window** (sim-frames 95–103): §5's largest single gap
  was `6 / 23` at frame 95, and nothing has re-measured it since the
  scout, the stands or the pull-back.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings; a
  `find_target` block; run7's order stream under the trace; a mounted
  attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings.

**The thing this session earned.** *Mark the phase before believing the
total — and mark it even when the total already agrees.* Run20's frame 1
read 53/53 for two days while carrying three independent errors that
cancelled, and one of them was a map layer the harness had been loading
and never reading. Two corollaries for this file: **a residue that
vanishes when an unrelated object moves has not been closed** (the
`do_move` row read zero on an intermediate build and came back), and
**a queue item that says "one draw short" is a question, not a
measurement** — the answer is rarely one draw.

**Needs the user.** Nothing blocking. The ledger (`docs/audit/README.md`)
is unchanged; its widest marker is still **`sin_table@00a46a00`'s
second-quadrant branch**, with the in-process exhaustive comparison as the
settlement. When to spend a Fable batch is still open; this session's
judgement is still **not yet**.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — go_around_building@005fc350, item 29 and the only residue left in run20's frame 1: unit 1/0's straight line clips a building at tile (201, 207) three tiles short of a clear goal, and the sim pays a Unit::do_move+0xe84 grid draw the original never spends. docs/ORDERS.md §4.6 has the shape; docs/SYNC.md §6 has the measurement and §5.1 the instrument. Then the fuzzed map's frame 1 — one produce_building+0xc99 short and one do_non_flat_gather+0x54b short, both asserted where they stand.`

## The queue

In dependency order. A default rather than a contract — take the next
unstarted one unless something has made a different order obviously better,
in which case say so and take that. The story of each struck item is in
`docs/JOURNAL.md`, "Lifted from the queue", under the same number.

0. ~~**Corrections from the second reading**~~ — done 2026-08-20. Each
   mechanic's `docs/<M>.md` ends with "Second reading — landed";
   `docs/audit/2026-08-20-*.md`.
1. ~~**The tech tree**~~ — done 2026-08-20. `docs/TECH.md`,
   `crates/sim/src/tech.rs`, `docs/audit/2026-08-20-tech.md`.
2. ~~**Combat**~~ — done 2026-08-20. `docs/COMBAT.md`, `combat.rs` /
   `fight.rs` / `balance.rs`, `docs/audit/2026-08-20-combat.md`; observed in
   run17 (§16).
3. ~~**Cities and buildings**~~ — done 2026-08-20. `docs/CITIES.md`,
   `build.rs` / `place.rs` / `city.rs` / `garrison.rs`,
   `docs/audit/2026-08-20-cities.md`.
4. ~~**The behavioural-check batch**~~ — run 2026-08-20 and 2026-08-24. The
   recipe is `docs/ORACLE.md`, "Running a check: the recipe in one place".
5. ~~**The data layer into the sim, and the diff**~~ — done 2026-08-20.
   `docs/DATALAYER.md`, `crates/rondata/src/{gamelog,dump,load,diff}.rs`.
6. ~~**The combat table's hardcoded half**~~ — done 2026-08-20.
   `docs/COMBAT.md` §15.2–§15.3, `crates/rondata/src/typesdump.rs`,
   `rondata --types`.
7. ~~**Orders**~~ — done 2026-08-21. `docs/ORDERS.md`,
   `crates/sim/src/orders.rs`, `docs/audit/2026-08-21-orders.md` (with its
   third pass).
8. ~~**The pathfinder**~~ — done 2026-08-23. `docs/PATHFINDER.md`,
   `crates/sim/src/path.rs`, `docs/audit/2026-08-23-pathfinder.md`; its open
   items in §10.
9. ~~**The recorded-game container**~~ — done 2026-08-24. `docs/RECGAME.md`,
   `crates/rondata/src/recgame.rs`, `docs/audit/2026-08-24-recgame.md`.
10. ~~**The command payload encoding**~~ — done 2026-08-24.
    `docs/COMMANDS.md`, `crates/rondata/src/commands.rs`,
    `docs/audit/2026-08-24-commands.md`.
11. ~~**The recorded order stream into the harness**~~ — done 2026-08-24.
    `docs/INPUT.md`, `crates/rondata/src/input.rs`.
12. ~~**AI**~~ — done 2026-08-25, over nine sessions. `docs/AI.md`,
    `crates/sim/src/ai*.rs` and `bhs.rs`, `docs/audit/2026-08-25-ai.md`
    (four passes); landed under it: `docs/SYNC.md`, `docs/ANIM.md`,
    `tools/trace/`, the cheat channel, runs 7–21.
13. ~~**Differential fuzzing against the original**~~ — **Tier 1 kept**,
    decided 2026-08-26. `tools/fuzz/{scenario,seedini,ledger}.py` and
    `run.sh`; `tools/gamelog/console.py`; `docs/ORACLE.md`, "The channel's
    vocabulary, and what it cannot do", "`restart` from the channel wedges
    the game" and "The 300-frame window". The journal has the story.

    **The verdict, and it is not the one it was being judged on.** The
    ledger's `survived` column is 1, not hundreds, and the three findings
    it was credited with on the day were **audited the same day and two of
    them handed back**:

    - **The who-8 panic is not a fuzzer finding.** run31 — tuned lobby, no
      cheats, on disk a day earlier — panics identically on the pre-fix
      sim. What reached it was *stepping more frames*, not a new map and
      not a cheat.
    - **The frame-0 draw gap is not a fuzzer finding.** run20 is 15 short
      too.
    - **§9.3's sixth citizen is.** `check_start_orders` is `[ok]` on run20
      and fails on the fuzzed map: a rule built on one map that does not
      generalise. One finding, from **map variation**.

    **And zero findings came from the cheat staging.** All three surfaced
    at frames 0–2, before `scenario.py`'s first `add` at frame 200 could
    fire; the control shape issues no cheat at all. So what has earned its
    place is `seedini.py` (a new map per seed) and the early wide window.
    `scenario.py`'s random `add`/`resource`/`military` generator — the part
    that looks most like fuzzing — has produced nothing yet, and the next
    session should either point it at something it can reach or drop it.

    **The one measurement that could still redeem it, and it needs a
    re-run.** The staged shape entered **6,872** functions against the
    control's **6,684** (`ledger.tsv`). Whether any of those 188 are on
    the blind list — `tools/trace/report.py … blind docs/`, the queue of
    runs — is the whole question, and it is unanswerable right now
    because `run.sh` named both traces `rontrace-fuzz-424242.log` and the
    control clobbered the staged one. **Fixed** (the archive name carries
    the shape now), but the answer costs one ten-minute staged run:
    `zsh tools/fuzz/run.sh 424242 1000 1300`, then the blind report
    against both traces. Do that before deciding the generator's fate.

    **The shape it settled into.** `FUZZ_STAGE=0` — no cheats, window at
    [1, 301) — is the measuring shape; the staged shape is the coverage
    shape. Its cost is 195 MB and ten minutes, plus six more for a heights
    sibling when a seed is worth one (`master_land_heights` is the only
    thing `DUMP_ALL` is still for, and a fuzzed map has no sibling to
    borrow it from).

    **What it still cannot do**, unchanged: reach what the channel cannot
    stage — no console command issues an order, so diplomacy beyond the
    verbs, the sea half until transports can be ordered, CtW and
    multiplayer stay out of reach. For those the reading is still the only
    evidence.
14. ~~**The sea half — transports and docks**~~ — done 2026-08-25.
    `docs/TRANSPORT.md`, `crates/sim/src/transport.rs`,
    `docs/audit/2026-08-25-transport.md`; run22 and `tools/gamelog/window.py`.
15. ~~**Armies**~~ — done 2026-08-25. `docs/ARMY.md`,
    `crates/sim/src/army.rs`, `docs/audit/2026-08-25-army.md`; runs 23–27,
    `tools/gamelog/runwin.sh`, `rngcmp.py`, `armyrecs.py`.
16. ~~**The group orders**~~ — done 2026-08-25, audited and applied
    2026-08-26, third pass (Fable, main thread) the same day.
    `docs/GROUPS.md`, `crates/sim/src/group.rs`,
    `docs/audit/2026-08-25-groups.md`; runs 28 and 29, and the whole
    `GROUPDATA` record in `rondata::diff`.
17. ~~**`Form::compute`'s slot table**~~ — done 2026-08-26.
    `docs/GROUPS.md` §6.4 and §15, `crates/sim/src/form.rs`,
    `docs/audit/2026-08-25-groups.md` ("Fourth pass"); the diff is
    `run29_s_navy_slot_table_is_reproduced_from_the_install_s_own_spacing`.
18. ~~**run29's `UNITS=3` half**~~ — done 2026-08-26. `scene_at`'s unit
    loader and `Log::dumps` in `crates/rondata`, `docs/ARMY.md` §11 and
    §18, `docs/GROUPS.md` §4.4, §6.4, §7 and §13, `docs/ORDERS.md` §4.1
    and §11.1, `docs/ORACLE.md`. Two of its four open items closed, one
    reassigned to item 20, one bug found in `engagement` and one in
    `find_leader`.
19. ~~**The move order's formation angle**~~ — **the adder landed
    2026-08-26** with item 22: `Sim::add_move_facing_order` carries
    `angle + (angles[i] << 24)` and the mirror, `docs/GROUPS.md` §6.6 step
    6 and §12's seam row, `docs/ORDERS.md` §8.4. What is left of it is the
    byte's **sign**, which no run can separate while every dumped group
    lays out in Line — folded into item 23's capture.
20. ~~**The human group move**~~ — done 2026-08-26. run30 and run31
    (`docs/ORACLE.md`), `docs/GROUPS.md` §4.4, §6.4, §6.6, §11, §12.1 and
    §13, `docs/ORDERS.md` §8.4 and §11.1; `tools/gamelog/live.sh`,
    `archive.sh`, `groups.py`. Four checks in `rondata::diff`, one in
    `sim::form`, one in `crate::gamelog`.
21. ~~**The 36-member table**~~ — done 2026-08-26. `docs/GROUPS.md` §6.8
    (new), §12.2 (new), §4.1, §4.4, §6.3 and §6.4;
    `Sim::group_add_keeping` and `Unit::o_up`/`o_down`,
    `GroupState::reorigin` and `Sim::group_refresh_order`,
    `Sim::group_find_leader_slot`. One check in `rondata::diff`, two in
    `sim::group`, five deliberate breakages.
22. ~~**The mirror's predicate**~~ — done 2026-08-26, **and no run was
    needed**. `docs/GROUPS.md` §6.3's new subsection, §12.3 (new), §4.1,
    §6.6 step 6, §12's seam table and §13; `docs/ORDERS.md` §3.2 and §8.4.
    `sim::group::reversing`, `Sim::unit_set_angle`,
    `Sim::hand_back_facing`, `Sim::add_move_facing_order`,
    `MoveOrder::facing`, and the `QUEUE_NEW` clear hoisted ahead of the
    layout. One check in `rondata::diff` and two in `sim::group`; five
    deliberate breakages, four red.
23. **The hand-back's inversion** — what item 22 left, and it is the
    narrow half of the run that entry had booked. `kill_current_order`
    writes `order.facing XOR reversing(leader.angle − order.angle)`, and
    run31's two kills catch the leader 10.6° and 6.3° off the dying
    order's angle, so the `XOR` term never fires; the listing at
    `5e3062`–`5e307b` is its only evidence. *Capture:* a `UNITS=3` +
    `GROUPS=1` window over a group ordered one way, turned **right
    around** while marching, then re-ordered. Set the formation to Refused
    or an Echelon in the same window and it also settles item 19's sign —
    `angles` is all zero in every run on disk, so nothing has separated
    `compute_form`'s subtraction from the order adder's addition.
    `docs/GROUPS.md` §13.
24. ~~**The frame-0 draw gap**~~ — done 2026-08-26 over two sessions.
    `docs/SCOUT.md`, `docs/SYNC.md` §3.6, §3.7 and §4.2;
    `crates/sim/src/{farms,scout}.rs`. run20 frame 0 175/175, frame 2 5/5.
    What it uncovered was item 26, closed the same day by item 27.

25. **The first frames, on a map we did not tune against** — what item 24
    was carrying besides the draw gap. `gamelog-fuzz-424242-early.txt`
    with `gamelog-fuzz-424242-heights.txt` as its `--sibling`.

    - **§9.3's sixth citizen.** `check_start_orders` fails on exactly one:
      *`who 1 o 6`: we derived None, the log has 2001* — and is `[ok]` on
      run20. The rule was built and confirmed on one map. `docs/AI.md`
      §9.3.
    - **The frame-1 order two of player 1's units hold and the sim does
      not**, and the position divergence at frame 2 that scores
      `survived = 1` on both maps. One of the two is the scout, and it is
      **half fixed**: on run20 its `EXPLORE_TO` now matches, on the fuzzed
      map the sim issues one at frame 0 and has none by frame 1, so
      something kills it during that frame.
    - ~~**`Leader::produce_building`'s draws**~~ — done 2026-08-26.
      Both sites marked; three defects behind the one-draw gap (an
      exclusive 2×2 jitter, a stride test on the wrong index, an
      unmodelled `WorldData::buildings_allowed`). Run20's frame 1 is
      39 + 4 on both sides and the farm lands on the original's tile.
      `docs/AI.md` §2.20, `docs/SYNC.md` §4.2 and §6.
      **What it left**: the fuzzed map is one `+0xc99` short (29 against
      30) and one `do_non_flat_gather+0x54b` short, 43 against 45.
    - ~~**`Farms::add`'s two draws**~~ — done 2026-08-26, `docs/SYNC.md`
      §3.8, `crates/sim/src/farms.rs`.

    None needs a new run. All are `rondata --diff` on a dump that exists.

26. ~~**The stand/wrap swap, which is no longer zero-sum.**~~ — done
    2026-08-26, by item 27's instrument and with no capture. It was one
    line: `do_non_flat_gather`'s camp-arrival branch has no
    `CHAR_DEFAULT`, and the sim's invented stand was resetting the
    citizens' clocks so the phase-7 wraps never fell due.
    `docs/SYNC.md` §6, `docs/ORDERS.md` §6.4, `docs/ANIM.md` §5 and §9.

27. ~~**Mark the other mechanics' draw sites.**~~ — done 2026-08-26.
    `crate::trace::SITES` and the `SITE_*` consts in `sim::{ai_sites,
    market, anim, orders, farms, gaia}`; `docs/SYNC.md` §5.1. The check is
    `diff::tests::frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`.

28. ~~**The `do_move` grid draw the sim spends and the original does
    not.**~~ — done 2026-08-26 for the gather's transit, which was
    `find_path`'s missing pull-back; run10's frame-3 draw was never this.
    `docs/ORDERS.md` §4.6, `docs/SYNC.md` §5 and §6. The scout's remains,
    as item 29.

29. **`go_around_building@005fc350`** — item 28's remainder, and a
    mechanic rather than a fix. The last `Unit::do_move+0xe84` on any
    capture is run20's frame 1, unit `1/0`: the AI scout's straight line
    clips a *building* at tile `(201, 207)`, three tiles short of a
    waypoint whose own tile is clear, so the pull-back has nothing to pull
    and the march has nowhere to go. `docs/ORDERS.md` §4.6 has the shape —
    a tile-edge walk pushing up to three `PathData`s (turn-in point, a
    `flags 8` midpoint, the target tile centre, each `off % 0xc0 / 2 −
    0x30` from the centre), a recursive `find_path` on the first, `return
    0` when it verifies, and on failure `mo->dest = 0; masks &= ~8;
    path_recursion = 10`. No run needed: run20 is on disk and both sides
    are marked.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — do not
  append to it — and update its banner. Write the session's entry in
  `docs/JOURNAL.md`. Strike a finished entry to one line and a pointer; its
  story goes to the journal, not here.
- **Start of session:** read "Where things stand", then the entry you are
  taking, then that mechanic's document. `CLAUDE.md` is the rules; this is
  the state; the journal is on demand.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and read the
  memory index — a subagent inherits both. If either names a finding the
  readers are meant to re-derive, move it here or to the journal first. The
  brief cannot undo what the system prompt already delivered.
- **Never** quote this file or the journal into `CLAUDE.md`, a subagent
  brief, a `.claude/agents/` definition, or a memory hook.
