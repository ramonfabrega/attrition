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

*Last verified 2026-08-26, after item 24.* The commit this section was
written against is the one that lands it; if `git log` has moved well past
it, trust the queue below and the journal before trusting this.

**Last landed: `Unit::think_scout`, and with it run20's frame 0 at
175/175 — the first frame-0 match the harness has had.** `docs/SCOUT.md`
is the mechanic: an idle AI scout walks rings of cells outward from every
city it knows, two draws at the head of each ring and one per cell that is
in its own region and **not really seen**, then sends itself there as a
one-member group with an `EXPLORE_TO`.

- **It is checked seed for seed on three maps, not by a count.** Every
  draw record carries the seed it was taken on, so installing the trace's
  first `think_scout` seed replays the whole sequence: 10 on run20
  (`0x9c59_1b2b`), 10 on the fuzzed map (`0x242c_b7ed`), and **24 on the
  Great Lakes** (`0x15fe_bc41`) — the only capture that exercises the
  foreign-city arm, split 4/2/1 then 2/0/15 across two cities.
- **The zero-sum defect is not zero-sum any more.** On the frame's own
  stream the harness reaches `think_scout` two draws early, because it
  spends a unit-loop stand for each gathering citizen where the original
  wraps them in phase 7 instead. `think_scout`'s count depends on the
  stream, so the fuzzed map's frame 0 is now **196 against 195** — one
  cell, and it is the whole gap. The `GUYS=4` capture the queue has been
  holding is now the check for a number.
- **One seam closed in passing.** `WorldData::is_cliff_at` is
  `(TData.mask & 3) == 1`, so `tile::OBJECT_CLIFF` is named and
  `crate::path`'s `invalid_loc` refuses a cliff (`docs/PATHFINDER.md`
  §11). It changed no count.

**And the harness reads the trace now** — `rondata::trace`, built straight
after, because doing the seed-anchored check by hand twice was the whole
argument for it. Three things it buys, all reusable by the next mechanic:
`rondata --trace <rontrace.log>` prints the original's per-frame fold **by
site** in the same shape `--diff`'s `by phase` prints ours;
`Trace::run_in` isolates one function's own draws from its callees';
and a mechanic that marks its own sites (`Sim::mark`, `diff::mark_sites`)
can be asserted **draw for draw** rather than by a total. The scout check
is now a sequence comparison — made to fail by transposing two marks,
which leaves the count at ten and the order wrong. `docs/SYNC.md` §5.

**The numbers now.** run20 frame 0 **175/175**, frame 1 52/53, frame 2
**5/5**. The fuzzed map: frame 0 196/195, frame 1 48/45. The Great Lakes
(run10): frame 0 128/120 (was 96/120), frames 1 and 2 unchanged at 54/54
and 6/6. `ticks before divergence` is still 1 everywhere; the position
divergence at frame 2 is untouched and is the next thing.

**Then, in order:**

- **The stand/wrap swap** — item 24's last piece, and it now has a price
  (the fuzzed map's one extra cell). The capture that settles it is a
  frame-0 `GUYS=4` window; `docs/SYNC.md` §6 has the half that run20's own
  dump already explains.
- **`Farms::add`'s two draws** at run20's frame 1 (`+0x23f`, `+0x25b`
  under `Build::init`), which is that frame's whole 52-against-53. Cheap,
  and the trace names both offsets.
- **The frame-1 order two of player 1's units hold and the sim does not**,
  and §9.3's sixth citizen — item 25. One of the two was the scout's
  `EXPLORE_TO` and is now issued; re-read the item before taking it.
- **Item 23, the hand-back's inversion** — unchanged, cheap, unblocked.
- **Re-run run13's window** (sim-frames 95–103) now that the scout thinks:
  §5's largest single gap was `6 / 23` at frame 95 and was attributed to
  this mechanic.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings; a
  `find_target` block; run7's order stream under the trace; a mounted
  attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings.

**The thing this session earned.** *Read the sites before the function,
and assert on the sequence rather than the count.* Three return addresses,
disassembled, gave the ring walk's shape and both its guards in ten
minutes; the decompile after that was naming. And a mechanic that replays
a **sequence** from a pinned seed is checked in a way a total never is —
which is what let this land while the stream that reaches it is still
wrong, and what turned that wrongness into a number. That second half is
now a harness primitive rather than a one-off, which is the part that
pays again next time.

**Needs the user.** Nothing blocking. The ledger (`docs/audit/README.md`)
is unchanged; its widest marker is still **`sin_table@00a46a00`'s
second-quadrant branch**, with the in-process exhaustive comparison as the
settlement. When to spend a Fable batch is still open; this session's
judgement is still **not yet**.

**Take 27 before 26, and it is not only that it is cheaper.** The
stand/wrap swap is a ±4 that cancels in every total. Once the animal idles
and the phase-7 wraps carry site marks, it stops being a total and becomes
a visible *order* mismatch at frame 0 — our stand where the original's wrap
is — which may settle it without the `GUYS=4` capture at all. Item 27 is
the instrument for item 26.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — item 27: mark the other mechanics' draw sites, so frame 0's fold is a site-by-site comparison rather than a per-phase count. rondata --trace <rontrace-run20.log> prints the target; crate::scout is the worked example and docs/SYNC.md §5 the tooling. Start with the animal idles and the phase-7 wraps, because those are what item 26 needs.`

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
    What it uncovered is item 26.

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
      something kills it during that frame. Its target differs there
      because of item 26, so start by ruling that out.
    - **`Farms::add`'s two draws** (`+0x23f`, `+0x25b` under
      `Build::init+0x4ea` < `Objects::init_build+0x82`) at run20's frame
      1, when the AI's new farm is created — the whole of that frame's
      **52 against 53**. `docs/SYNC.md` §6.

    None needs a new run. All are `rondata --diff` on a dump that exists.

26. **The stand/wrap swap, which is no longer zero-sum.** The sim spends
    a unit-loop stand for each gathering citizen; the original spends none
    there and wraps the same figures in phase 7 instead — +4/−4 on run20,
    +5/−5 on the fuzzed map. It used to net to zero and hide. It does not
    any more: `Unit::think_scout` runs two draws late on the sim's stream
    and takes one cell more on the fuzzed map, so that frame 0 is 196
    against 195 (`docs/SCOUT.md` §10, `docs/SYNC.md` §4.2). *Capture:* a
    frame-0 `GUYS=4` window — every `set_anim` logs its index at detail 4.
    Run20's own end-of-frame-0 dump explains two of the four wraps and not
    the other two; `docs/SYNC.md` §6 has that half.

27. **Mark the other mechanics' draw sites.** `rondata::trace` and
    `diff::mark_sites` make a per-mechanic draw-*sequence* assertion cheap,
    but only `crate::scout` marks its sites so far. The candidates in
    rough order of what a capture already covers: the market
    (`calc_market`'s three), `Animal::do_idle` and the phase-7 wraps
    (`docs/ANIM.md`), `Farms::inc_time`, `Objects::process_all`'s birds,
    `Leader::compute_sites`. Each is a few `Sim::mark` calls and one
    check; between them they would turn frame 0's whole fold from a
    per-phase count into a site-by-site comparison. Cheap, and no run
    needed — `rondata --trace <log>` already prints the target.

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
