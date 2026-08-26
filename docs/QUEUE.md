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

*Last verified 2026-08-26, after the third pass.* The commit this section
was written against is the one that lands it; if `git log` has moved well
past it, trust the queue below and the journal before trusting this.

**Last landed.** The **group orders' third pass** — the Fable ratification
run as the session itself, under the charter, on 2026-08-26
(`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts"). The floor
held eight of nine: **one Rust-changing verdict was overturned in its
consequence** (a hurrying army with no city *marches* its cleared siege
unit; the order loop re-reads an `order_type` the clear loop has emptied),
and the last `FABLE:` marker is closed from the PDB's method records
(`+0x1c` is `is_wallbuild`). Outside the floor: the predicate behind every
stance decision (`UnitTypeData::get_stance_type` — the sim had its tests
in the wrong order), the slot table's rounding (a **floor**, by the table
that does it), `update_positions` reproduced to the bit from the leader's
logged heading, a fourth writer of `facing` (`Unit::set_angle`), and the
loaders' two overwrites. Two Rust changes, both run red first; one new
`GROUPDATA` widening. The group orders are **done**, third pass included.

**Then, in order:**

- **Implement `Form::compute`'s slot table** (`docs/GROUPS.md` §6.4). The
  seam stands on cost alone, and the questions the record could not settle
  are now settled by reading: the `/48` is `floor` (`div_3_table` is a
  floor table, `>> 4` arithmetic); `k = 1` in the captain arm; the `w/2`
  shift on even `cols` for a move; the block translated so the first member
  of the last non-empty category sits at `(0, 0)`. The fixture is
  `run29_s_navy_group_is_a_line_of_four_rotated_at_forty_eight_units_a_step`
  (`off_x = [0, −14, 13, −28]`, one width `w ∈ (648, 672)`), and
  `run29_s_navy_group_s_curr_is_the_leader_s_heading_applied_to_the_slot_table`
  pins `update_positions` exactly. Four functions — `categorize`,
  `compute_rows_and_columns`, `compute_dests`, `update_positions`. Still to
  read on the way: `get_form_mod_option`'s value for an AI army, the type
  widths `+0x228`/`+0x22c`, and `FormData::type_cat`'s assignment of the
  shipped types (`docs/GROUPS.md` §13). Reader A's report at
  `~/ghidra-projects/reading/groups-2026-08-25/` is the working notes.
- **run29's `UNITS=3` half** — the per-unit order lists nobody has opened.
  `scene_at` would need to load them; that is what pins `engagement`'s
  *choice* of unit (`docs/ARMY.md` §18), and it is also what would put a
  formation *with depth* in reach, which is the one thing the `GROUPDATA`
  rotation checks still cannot pin (every `off_y` in the window is zero).
- **Item 13, differential fuzzing** (below) — unchanged, and still the
  entry with the largest leverage per hour.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings the army's
  readers named; a `find_target` block where two candidates sit within a
  multiplier of each other; run7's order stream under the trace; a
  mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**Three things this session earned.** (1) **A verdict's *consequence* is a
separate claim from its *gate*, and it needs its own reading** — item 34's
two gates were read right and its consequence wrong, because nobody
followed what the first loop had done to the state the second loop reads.
(2) **The PDB's `LF_ONEMETHOD` records name a vtable slot the map cannot**
— a COMDAT-folded slot has one name per address in `rise_z.map` and its
own name in the type stream (`llvm-pdbutil dump --types`, ~10 s). (3) **A
mechanical scan is worth delegating and a judgment is not**: two Opus
scanners over the whole export found the writers and callers in a quarter
hour; every claim built on a hit was re-read here.

**Needs the user.** Nothing outstanding. The older Fable debt from the AI,
transport and army audits is still booked and was **not** in this pass's
scope; whether to clear that ledger is a later conversation, and the
cheapest way to clear most of it is item 13's captures rather than a
reading.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — implement Form::compute's slot table (docs/GROUPS.md §6.4) against the two run29 fixtures already in rondata::diff: categorize, compute_rows_and_columns, compute_dests (the /48 is a floor; k = 1; the even-cols w/2 shift; translate to the first member of the last non-empty category), and update_positions as §6.6 now states it`

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
17. **`Form::compute`'s slot table** — the one seam `docs/GROUPS.md` §6.4
    still declares, and the only one of this mechanic's that costs work
    rather than a grep. Its fixtures are two passing tests and its rounding
    is settled; the brief is "Then, in order" above.

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
