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

*Last verified 2026-08-26, after item 20.* The commit this section was
written against is the one that lands it; if `git log` has moved well past
it, trust the queue below and the journal before trusting this.

**Last landed.** **The human group move** (item 20), and it came in
cheaper and richer than the queue had budgeted. `GameLog::full_dump`'s
non-`DUMP_ALL` half gates `dump_groups` on `gamelog.ini`'s own `GROUPS`
key, so the 512-slot group pool is an ordinary per-frame record at full
speed — hundreds of frames instead of a two-frame `DUMP_ALL` window,
which is what made a *human-timed* right-click affordable at all. The
selection turned out to be scriptable too (`select <type> who=0 [+]` from
the cheat channel), so the only human-shaped step left is the click.
`tools/gamelog/live.sh`, `archive.sh` and `groups.py` are new;
`setlog.py` takes `CAT=N`.

run31 (`gamelog-run31-humangroup.txt`) closes all three items it was
booked for and opens two better ones:

- **`find_leader`'s key** — `GroupOrder::oxx` names the leader outright,
  and it is a hoplite where `list[0]` is a slinger. `docs/GROUPS.md`
  §4.4, §12.1.
- **§6.4's `to`/`off` asymmetry — observed.** The anchor's own order does
  not point at the click. Its *size* is not measured yet.
- **`update_positions`' y-flip — pinned**, with the unflipped matrix as a
  control. The eleventh deliberate breakage of the slot-table session,
  which had not been able to turn a test red, now does.
- Two bugs found on the way: the parser dropped `GroupMoveOrder` because
  it is the one block in the family the binary keeps in **mixed case**
  (and the positional `type` pairing slid with it), and
  `compute_dests`' **follower arm** had never executed — a player's
  selection group keeps every figure, so 36 members and not 12.
  `Sim::form_follower_slot` implements it.
- And one correction to carry: **`curr` is a mid-frame quantity.** The
  end-frame `angle` is the heading a hair past the one the rotation used;
  nine frames of forty are exact and the rest match at a heading within
  0.05° of it.

**Then, in order:**

- **The 36-member table** (new item 21). Reproduce run31's whole slot
  table — which turns §6.4's *observed* asymmetry into a *measured* one,
  and answers §4.4's new question of why the leader is object 9 on one
  frame and object 6 on the next when both are hoplite captains and 6 is
  first in `list`. It needs `Group::add`'s `keep_captain` on the sim
  side. The record is on disk; no run is owed.
- **The order's angle** (item 19, unchanged), and now with a record:
  `docs/GROUPS.md` §6.6 step 6's `angle + (group.angles[i] << 24)`
  against `GroupMoveOrder`'s own `MOVEORDER` `angle`.
- **Item 13, differential fuzzing** — unchanged, and still the entry with
  the largest leverage per hour.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings the
  army's readers named; a `find_target` block where two candidates sit
  within a multiplier of each other; run7's order stream under the trace;
  a mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**The thing this session earned.** Item 18's lesson was *the record was
already there and nobody had opened it*. This one is a level up: **the
record was never written, because a key nobody had questioned was zero** —
and it stayed zero because turning it on did nothing, which reads as "not
a per-frame category" rather than "inherits its acceptance from whatever
dumper ran last". Worth asking before booking any window: which of the
thirty-seven `gamelog.ini` categories has never been on, and what happened
the one time it was?

**Needs the user.** Nothing outstanding. The ledger
(`docs/audit/README.md`) still has the two rows from item 18; this session
added no `FABLE:` markers. When to spend a Fable batch is still open.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — item 21, the 36-member table: reproduce run31's whole slot table from the install's own columns, which measures docs/GROUPS.md §6.4's to/off asymmetry and answers §4.4's leader question; the record is on disk and no run is owed`

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
13. **Differential fuzzing against the original** — proposed and agreed
    2026-08-25, not started. The three pieces exist and have each been run:
    `crates/sim/src/soak.rs` generates a scenario and an order stream from a
    seed; `rontrace.cmd` stages a scenario into the original from a file,
    inside the tick, unattended, and `!quit`s it cleanly (`docs/ORACLE.md`,
    "The cheat channel"); `rondata --gamelog <dump> --diff` scores a dump
    frame for frame. Wired together, every seed is a behavioural run, and
    the trace's blind list shrinks without anyone writing a scenario by
    hand. That is the whole argument, and it is why this is worth a queue
    entry rather than a remark. What is not yet true, in the order it has
    to become true:

    - **A map both sides carry.** The soak's world is flat and hand-built;
      the original needs a lobby and a map. Stage on run9's map, which the
      harness already loads from its `WORLD=6` dump, with the lobby seed
      fixed — a scenario is then "these types at these tiles, these orders
      at these frames" on ground both simulations agree about.
    - **A vocabulary bounded by the channel.** A scenario can contain only
      what the cheat channel can stage: a spawn by type at a tile, a
      selection, an order (move, gather, build, attack, garrison). Start
      with move, gather and attrition — the mechanics that already match
      frame for frame — and widen by the blind list, one family a time.
    - **An AI-free lobby, or the AI scored apart.** The original's AI runs
      unless the lobby has none (`-config`); run6's lobby is the template.
      With an AI in, its units are noise the diff must be told to skip.
    - **The dump is the whole cost.** ~3 frames a second at `UNITS=3`,
      ~500 with the per-frame dump gated off. A scenario is a window:
      fast-forward to it, dump one to three hundred frames, quit. Budget one
      to two minutes a seed, and run seeds in a batch overnight.
      `tools/gamelog/runwin.sh` is that window run, unattended, today.
    - **Definition of done:** a tool (`tools/fuzz/`, or beside
      `tools/gamelog/`) takes a seed, writes the `.cmd` and the ini, drives
      the game, runs the diff, and appends one line to a ledger — seed,
      first divergent frame, the functions the trace saw execute. The ledger
      is what `report.py … blind` reads next, and the first divergent frame
      is the score to move.
    - **What it cannot do**, said now so nobody expects it: reach what the
      channel cannot stage — diplomacy, the sea half until transports can
      be ordered, CtW, multiplayer. For those the reading stays the only
      evidence, and that list is the reading's brief.
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
19. **The move order's formation angle** — `docs/GROUPS.md` §6.6 step 6
    and §12's `GroupMoveOrder` row, together with `docs/ORDERS.md` §8.4's
    verdict. The angle byte is computed and carried; the adder is not.
20. ~~**The human group move**~~ — done 2026-08-26. run30 and run31
    (`docs/ORACLE.md`), `docs/GROUPS.md` §4.4, §6.4, §6.6, §11, §12.1 and
    §13, `docs/ORDERS.md` §8.4 and §11.1; `tools/gamelog/live.sh`,
    `archive.sh`, `groups.py`. Four checks in `rondata::diff`, one in
    `sim::form`, one in `crate::gamelog`.
21. **The 36-member table** — reproduce run31's group whole, from the
    install's own columns through `categorize`, `compute_rows_and_columns`,
    `compute_dests` and its follower arm, and compare with `GROUPDATA`'s
    `off` and the members' `GroupMoveOrder` destinations. Two things fall
    out of it and nothing else reaches them: **the size** of §6.4's
    `to`/`off` displacement, which the record shows but does not measure,
    and **why the leader is object 9 on frame 204 and object 6 from 328
    on** when both are `FORM_CAT_FOOT` captains and 6 is first in `list`
    (§4.4). It needs `Group::add`'s `keep_captain` on the simulation side,
    so that a group can hold a follower at all. No run is owed — the
    record is `gamelog-run31-humangroup.txt` and the fixture is
    `rondata::diff`'s `run31_moves()`.

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
