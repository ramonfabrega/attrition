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

*Last verified 2026-08-26, after the who-8 panic and the cheap window.* The
commit this section was written against is the one that lands it; if
`git log` has moved well past it, trust the queue below and the journal
before trusting this.

**Last landed.** Two things, and both were smaller than billed while hiding
something larger.

- **The who-8 panic is fixed, and the fix is a bound, not a guard.**
  `Leaders::list` is `Leader[10]`; two independent places stop every object
  search at leader eight, so the original never asks a diplomacy question
  about gaia — it could not answer one, since `diplos` is `int[8]`. Nothing
  in RoN can attack an animal. `world::PLAYER_SLOTS`, `docs/ANIM.md` §6.1,
  five tests and five red breakages. Commit `d1f5be4`.
- **The window experiment was answered by a file already on disk, and the
  premise was wrong twice.** `[Start Game] WORLD=6` writes the 3600 cells
  with `DUMP_ALL=0` (run31's start dump has them), and no frame *inside* a
  window needs a `WORLD` block at all — `run_traced` stands the sim up from
  the start dump. So `[End Frame] WORLD=6` was never needed. Same seed,
  back to back: `DUMP_ALL` bought **5 frames for 249 MB**; the cheap window
  bought **301 for 207 MB** in less wall clock. `docs/ORACLE.md`, "The
  300-frame window". `tools/fuzz/run.sh` uses it now.

**The number that came out of it.** A control run — `scenario.py
--no-stage`, no cheats at all, window at **[1, 301)** — is the first time
sim-frame 1 has ever been compared against the original **on any map**.
It scores **`survived = 1`**: player 1's `o 0` is 24 position units off on
both axes at frame 2, player 0's `o 1` by 10 at frame 4, and at frame 1
two of player 1's units already hold an order the sim never issued. A
heights sibling for the same seed (`window.py stage 1 3`, six minutes)
returns **identical** coordinates, so this is the port and not the flat
map that a cheap window leaves behind.

Two leads came with it, both impossible on the one lobby everything else
was captured on:

- **`rng: frame 0: ours 180 draws, the original's 195`** — 15 short on a
  map we did not tune against, then 4 *over* at frame 1. On run7's lobby
  the setup draws have matched for weeks.
- **`check_start_orders` fails on one citizen** — *`who 1 o 6`: we derived
  None, the log has 2001*. `docs/AI.md` §9.3 does not generalise off run7's
  map.

**Then, in order:**

- **The frame-0 draw gap** (15 short, above). It is the cheapest lead on
  the board, it is upstream of everything, and the trace names every draw
  site. Start there.
- **The frame-1 order two of player 1's units hold and the sim does not**,
  and §9.3's sixth citizen. Same run, same dump
  (`gamelog-fuzz-424242-early.txt`, with `-heights` as its sibling).
- **Item 23, the hand-back's inversion** — unchanged, cheap, unblocked,
  and now affordable to capture over hundreds of frames rather than three.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings; a
  `find_target` block; run7's order stream under the trace; a mounted
  attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**The thing this session earned.** Yesterday's note was *measure the oracle
before building a better one*. Today's is the same rule one step earlier:
**check whether the expensive setting is doing anything before pricing
it.** This file had budgeted a 300-frame window against 25 MB a frame and
concluded it was probably unaffordable. It costs 0.69, and the evidence was
a `grep -c who2` on a file that had been on disk for eleven hours. The
corollary for the port: **every window until today opened late** — 95,
149, 3000 — so the first frames were never checked, and that is where the
divergence turns out to be.

**Needs the user.** Nothing blocking. The ledger (`docs/audit/README.md`)
is unchanged; its widest marker is still **`sin_table@00a46a00`'s
second-quadrant branch**, with the in-process exhaustive comparison as the
settlement. When to spend a Fable batch is still open; this session's
judgement is still **not yet**.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — the frame-0 draw gap: the sim makes 180 draws where the original makes 195, on the fuzzed map in gamelog-fuzz-424242-early.txt (sibling -heights). Find the missing 15.`

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
    ledger's `survived` column is 1, not hundreds. But **one seed, run
    three ways**, produced a panic (`Sim::is_enemy` with the nature
    player), a frame-0 draw-count gap (180 against 195) and a broken
    generalisation (§9.3's start-of-game gather rule, one citizen) — all on
    a map nobody chose. That is what it is for, and no hand-built capture
    on the one tuned lobby could have produced any of the three.

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
24. **The first frames, on a map we did not tune against** — the two
    leads the fuzzer's control run left, and the first entry that comes
    from a capture opened at sim-frame 1. Both live in the same pair of
    dumps: `gamelog-fuzz-424242-early.txt` with
    `gamelog-fuzz-424242-heights.txt` as its `--sibling`.

    - **The frame-0 draw gap.** `note: rng: frame 0: ours 180 draws, the
      original's 195` — fifteen short — then frame 1 four *over*, 49
      against 45. On run7's lobby the setup draws have matched for weeks,
      so this is something the map generation or the start-of-game path
      does on some maps and not that one. `tools/trace/` names every draw
      site; `tools/gamelog/rngcmp.py` and `draws.py` are the instruments.
      Upstream of everything else, and the cheapest lead on the board.
    - **§9.3's sixth citizen.** `check_start_orders` fails on exactly one:
      *`who 1 o 6`: we derived None, the log has 2001*. The rule was built
      and confirmed on one map. `docs/AI.md` §9.3.

    Neither needs a new run. Both are `rondata --diff` on a dump that
    exists.

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
