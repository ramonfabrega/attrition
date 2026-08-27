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

*Last verified 2026-08-26, after items 30 and 31 — `toff`, and the goal
that turned out to be the group's.* The commit this section was written
against is the one that lands it; if `git log` has moved well past it,
trust the queue below and the journal before trusting this.

**Last landed: `toff` is the current move order's own `off_x/off_y`.**
`docs/PATHFINDER.md` §4.1 and §7 had it as a *target's*, behind two
COMDAT-folded vtable slots; the PDB's own method list names them
`UnitOrder::is_move` (`+0x14`) and `update_move_order` (`+0x40`), which
`docs/ORDERS.md` §4.1 had printed correctly all along, and their bodies
are `mov eax,1`, `xor eax,eax` and `lea eax,[ecx-0x54]`. So a plain move
fills `toff` too, and `crates/sim/src/path.rs`'s stated zero seam is gone
(`Sim::toff`). The same misreading in §3's `find_tpath` line is corrected
with it.

**The numbers now.** Run20's unit `1/0` walks the original's own
`cell*0x300 + 504` lattice, and **three of its five world nodes are the
original's entry for entry** where before **none** were
(`diff::tests::run20_s_world_chain_sits_on_the_move_orders_own_offset`,
made to fail twice on purpose). Everything else is untouched and was
re-measured, not assumed: run20 175/175, 53/53, 5/5; the fuzzed map
195/195, 43/45; the Great Lakes 120/54/6 with 7 at frame 3; run6's long
pin exactly **1,552 / 1,160**. `ticks before divergence` is still 1.

**The path-stack count did not move, and that is the honest reading.**
Still 17 over five frames. The two stacks are now the same numbers
*shifted by one slot* — the original's is nine entries and ours seven — so
a slot-wise diff cannot see a chain that agrees. The count is the wrong
instrument for this; the entry-for-entry test is the right one.

**Item 31 was not a pathfinder item at all.** The goal `0x18` short on
both axes cannot come from `find_wpath`'s pre-walk (it steps through
`sin_table`, never `−0x18` on both axes) nor from `Unit::do_move` (which
pushes `mo->x` verbatim, and `add_move_facing_order` makes every `mo->x`
`≡ 0x18 (mod 0x30)` — `41952` is `≡ 0`). It is
`Group::action_move_near@00704990`, and **`docs/GROUPS.md` §6.7 already
had it in plain words**: the group plans one path on the global
`grouppath` whose `FINAL` entry is the leader's **raw slot destination**,
un-snapped, and hands it to its members. The simulation does not
implement §6.7 at all — `docs/GROUPS.md` §12 claimed it did, and is
corrected.

**Then, in order:**

- **§6.7, the group's own path — the new item 31.** It is three of
  run20's disagreements at once and needs no capture: the goal (the raw
  slot), the *timing* (the original's `1/0` is `is_pathed` with nine
  entries on the frame ours has an empty stack and `flags 0`, because the
  group plans at order time and we wait for `do_move`), and the *route*
  (theirs is the leader's chain translated by `slot[i] − slot[leader]`;
  ours is each member's own from its own position). `pathfinder +0x70 = 1`
  around an army group's call has to land with it, and the `< 0x900`
  short-circuit is what keeps a short group move from planning at all.
- **The fuzzed map's frame 1**, two rows: one
  `Leader::produce_building+0xc99` **short** (29 against 30 — a spiral
  candidate the original scores and the sim does not) and one
  `Unit::do_non_flat_gather+0x54b` short.
  `diff::tests::the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well`
  asserts both as they stand, so closing either fails the test.
- **§9.3's sixth citizen** and **the frame-1 order two of player 1's units
  hold and the sim does not** — the rest of item 25.
- **Item 23, the hand-back's inversion** — unchanged, cheap, unblocked.
- **Re-run run13's window** (sim-frames 95–103): §5's largest single gap
  was `6 / 23` at frame 95, and nothing has re-measured it since the
  scout, the stands, the pull-back or the detour.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings; a
  `find_target` block; run7's order stream under the trace; a mounted
  attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings.

**The thing this session earned.** *The answer to an open question is
often already written down in another mechanic's document.* The queue
named `find_wpath`'s pre-walk and the pre-walk was innocent; one `grep` of
`docs/GROUPS.md` for "slot" would have cost a minute. The rule:
**before booking a reading for a residue, grep the documents of every
mechanic the value passes through, not only the one it was measured in.**
Its sibling, from the same session: *a folded vtable slot is named by the
type record, never by the listing* — `llvm-pdbutil dump --types`'s
`LF_ONEMETHOD ... vftable offset` gave both slots in one pass, and Ghidra
named neither.

**Needs the user.** Nothing blocking. The ledger (`docs/audit/README.md`)
is unchanged; its widest marker is still **`sin_table@00a46a00`'s
second-quadrant branch**, with the in-process exhaustive comparison as the
settlement. When to spend a Fable batch is still open; this session's
judgement is still **not yet**.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — item 31, and it is a GROUPS item, not a pathfinder one. docs/GROUPS.md §6.7 has Group::action_move_near planning ONE path on the global grouppath — FINAL entry at the leader's raw, un-snapped slot destination from form+0x514/form+0x714, nothing planned within 0x900, pathfinder +0x70 = 1 around an army group's find_wpath, then every waypoint popped from the top and translated by slot[i] - slot[leader] with the area-id guard and the 0x600 follower cutoff. crates/sim/src/group.rs ends its member loop at add_move_facing_order and never plans, so run20's 1/0 disagrees three ways at once: the goal is 0x18 long, the order is not is_pathed at frame 1 where the original's is with nine entries, and the route is each member's own instead of the leader's. docs/GROUPS.md §13's first entry has the whole shape; no capture needed, run20 is on disk.`

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
    `docs/ORDERS.md` §4.6, `docs/SYNC.md` §5 and §6. The other half was
    item 29 — and it was another unit's, not the scout's.

29. ~~**`go_around_building@005fc350`**~~ — done 2026-08-26, and the last
    `Unit::do_move+0xe84` on any capture with it. `docs/ORDERS.md` §4.6.1,
    `docs/SYNC.md` §4.2 and §6, `crates/sim/src/orders.rs`. Run20's frame 1
    is 53/53 draw for draw and unit `1/1`'s path stack is the original's
    entry for entry. The residue's owner in the old text (`1/0`) was
    wrong; the count was not.

30. ~~**`toff`, and the world grid's waypoints**~~ — done 2026-08-26.
    `docs/PATHFINDER.md` §4.1/§7/§12, `crates/sim/src/path.rs`
    (`Sim::toff`), `docs/JOURNAL.md`.

31. **`Group::action_move_near` plans the group's path — §6.7, which the
    simulation does not have.** ~~`find_wpath`'s pre-walk moves the
    goal~~ — that was the wrong suspect (`docs/PATHFINDER.md` §12 says
    why). The goal on a group member's stack is the **leader's raw slot
    destination**, the path is planned **at order time**, and the members'
    waypoints are the leader's chain translated by `slot[i] −
    slot[leader]`. `docs/GROUPS.md` §6.7 has the mechanic and §13's first
    entry has the three disagreements it costs run20, each with the field
    that measures it. No capture needed.

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
