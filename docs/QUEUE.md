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

*Last verified 2026-08-26, at `f74a743`.* The commit this section was
written against; if `git log` has moved well past it, trust the queue below
and the journal before trusting this.

**Last landed.** The **group orders' audit, applied whole** — three
commits: `a7c043e` the nine Rust-changing verdicts, `bdf8bc3` the
twenty-five document corrections, `f74a743` the whole `GROUPDATA` record in
`rondata::diff`. Every item of
`docs/audit/2026-08-25-groups.md`'s "What must change" is in;
`docs/GROUPS.md` §14 is now the ledger of what was wrong rather than a
warning. 648 tests green.

**Four of the audit's five `FABLE:` markers were settled on the way**, each
by the check the audit itself named and each in minutes (the audit's
"Markers settled" section carries the citations, `docs/GROUPS.md` §14 the
summary). Two changed conclusions: `unit_flags` bit `f` is "flies like a
helicopter", so item 33 was **not** vacuous; and the "network semaphore
bit" is bit 11 = *the scenario editor is open*. Settling the third turned up
a writer of `GroupData::facing` that no reading had —
`Unit::kill_current_order@005e2cb0`, outside the `Group` family. Only the
name of object vslot `+0x1c` is still marked, and nothing depends on it.

**Owed, and it needs the user's word before it is spent:** the **Fable
ratification pass**. Its scope is now just the nine Rust-changing verdicts
(the audit's last section) plus the one surviving marker — the four settled
ones narrowed it but did not discharge it, because they were settled on
Opus. `docs/DECISIONS.md` entry 22, `docs/audit/README.md` step 6. This has
been booked since the AI mechanic and is the only thing standing between
this mechanic and "done".

**Then, in order:**

- **Implement `Form::compute`'s slot table** (`docs/GROUPS.md` §6.4). The
  seam now stands on cost alone, and the fixture it will be checked against
  is landed:
  `run29_s_navy_group_is_a_line_of_four_rotated_at_forty_eight_units_a_step`
  holds `off_x = [0, −14, 13, −28]` for a four-member group in formation 0.
  Four functions — `categorize`, `compute_rows_and_columns`,
  `compute_dests`, and `update_positions`' rotation composed with a y-flip
  (determinant −1; a naive port mirrors). Reader A's report at
  `~/ghidra-projects/reading/groups-2026-08-25/` is the working notes; the
  audit's assertion 5 names the one thing the record cannot settle (the
  rounding behind `−14` versus `+13`).
- **run29's `UNITS=3` half** — the per-unit order lists nobody has opened.
  `scene_at` would need to load them; that is what pins `engagement`'s
  *choice* of unit (`docs/ARMY.md` §18), and it is also what would put a
  formation *with depth* in reach, which is the one thing the `GROUPDATA`
  rotation check cannot pin today.
- **Item 13, differential fuzzing** (below) — unchanged, and still the
  entry with the largest leverage per hour.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings the army's
  readers named; a `find_target` block where two candidates sit within a
  multiplier of each other; run7's order stream under the trace; a
  mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**Three things this session earned.** (1) **A `FABLE:` marker is a question
with a costed answer, and the cost is usually smaller than the estimate
written beside it** — run the named check before booking the debt. (2)
**The cheapest correction is always the one whose evidence is already on
disk**: `rise_z.map`, `GroupData::log_data`, and `unitrules.xml`'s own
comment header each settled something a reading had guessed at. (3) **The
first `GROUPDATA` assertion failed on its first run** — `priority` is 0 on
an emptied hotkey slot — which is two for two on `CLAUDE.md`'s rule about
widenings.

**Needs the user.** One decision: **spend Fable on the ratification pass
now, or keep booking it?** Fable has been conserved since the AI mechanic,
so the debt now covers four mechanics' worth of Opus-adjudicated verdicts.
The scope for *this* mechanic is small (nine verdicts, one marker) and
would cost one subagent. Verify the model from the transcript
(`lore spawns`), never from the spawn parameter.

**Opener:** `proceed @docs/QUEUE.md — implement Form::compute's slot table (docs/GROUPS.md §6.4) against the run29 fixture that is already in rondata::diff: categorize, compute_rows_and_columns, compute_dests, and update_positions' rotation-with-a-y-flip`

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
16. ~~**The group orders**~~ — done 2026-08-25, audited and **applied**
    2026-08-26, less the Fable ratification pass (see "Owed" above).
    `docs/GROUPS.md`, `crates/sim/src/group.rs`,
    `docs/audit/2026-08-25-groups.md`; runs 28 and 29, and the whole
    `GROUPDATA` record in `rondata::diff`.
17. **`Form::compute`'s slot table** — the one seam `docs/GROUPS.md` §6.4
    still declares, and the only one of this mechanic's that costs work
    rather than a grep. Its fixture is already a passing test; the brief is
    "Then, in order" above.

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
