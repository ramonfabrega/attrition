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

*Last verified 2026-08-26, the session after `e8b2c71`.* The commit this
section was written against; if `git log` has moved well past it, trust the
queue below and the journal before trusting this.

**Last landed.** The **group orders** (`docs/GROUPS.md`,
`crates/sim/src/group.rs`, commit `c53e4c6`) and their **full second
reading, adjudicated** — `docs/audit/2026-08-25-groups.md`, 124 verdict
rows. Two blind readers and one adjudicator, all three verified
`claude-opus-5` from their transcripts. With them the army's order-issuing
half moves units for the first time: `do_forming`, `march_to_target`,
`engagement`, `send_here`, `charge`, `set_stance`, `Army::close`'s halt.
run28 made `Army::engagement` execute on a frame predicted from the state
machine; run29 turned that into an assertion (`docs/ARMY.md` §16.6, §17
item 6).

**Owed, and it is the next session's first act: apply the audit.**
`docs/audit/2026-08-25-groups.md`'s "What must change" is written to be
applied mechanically — do not re-derive it. In order:

1. **Nine verdicts change Rust**, listed in the audit's last section. Two
   are outright bugs, not imprecision: `group_action_halt` writes each
   *unit's* `form` where `0070d0c0:29` writes the *group's* — and
   `group_get_form` reads the unit bytes, so it is live — and
   `group_action_attack`'s "already attacking" skip is unconditional
   where the original's has two sub-arms.
2. **`docs/GROUPS.md` §14 already says the document is wrong** and names
   the four worst: the dead seam justification, three misread vtable
   slots, `Group::priority`'s five writers. Strike them as each
   correction lands, per the amend-in-place rule.
3. **Five `FABLE:` markers**, each with its check. Those plus the nine
   Rust-changing verdicts are the scope of the Fable ratification pass
   owed before the next mechanic builds on this one
   (`docs/DECISIONS.md` entry 22, `docs/audit/README.md` step 6).

**Then, and the order matters — it changed when the audit landed:**

- **Diff the whole `GROUPDATA` record**, the audit's first named
  assertion and the cheapest thing on this list.
  `GroupData::log_data@0045e1d0` dumps `off_x`, `off_y`, `curr_x`,
  `curr_y`, `angles`, `form`, `form_num`, `o_dist`, `o_angle` per member;
  run29's window has **5,392 records, 44 live**, already on disk. The
  project's own rule — when the original dumps a record, diff the whole
  record — and nobody has read one.
- **Then implement `Form::compute`'s slot table**, against that diff
  rather than ahead of it. The seam `docs/GROUPS.md` §6.4 declared has no
  justification left: there is no float barrier (fourteen instructions,
  all in `compute_dests`, all integer-exact) and the capture exists.
  `compute_dests`, `compute_rows_and_columns`, `categorize`, and
  `update_positions`' rotation-composed-with-a-y-flip (determinant −1 — a
  naive port mirrors). Reader A's report at
  `~/ghidra-projects/reading/groups-2026-08-25/` is the working notes.
- **run29's `UNITS=3` half** — the per-unit order lists nobody has
  opened. `scene_at` would need to load them; that is what pins
  `engagement`'s *choice* of unit (`docs/ARMY.md` §18).
- Then the older backlog: the `LEADERDATA` and `CITY` widenings the army's
  readers named; a `find_target` block where two candidates sit within a
  multiplier of each other; run7's order stream under the trace; a
  mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**Four things the group-orders session earned, all of them the hard way.**
(1) **Read the queue's own handoff before working** — it had said "spawn
the readers while they run" and the session read past it, so the fan-out
went out at the end instead of alongside. (2) **Open `rise_z.map`.** The
13 MB linker map `CLAUDE.md`'s thesis names recovers vtable slot names
that three COMDAT folds hide; guessing at five of them put wrong rules in
two sections. (3) **Ask the blind list against *every* trace on disk** —
`docs/ARMY.md` had claimed `engagement` never executed; run16 had it since
2026-08-24. (4) **Before declaring a seam, grep `log_data` for the fields
it covers.** The slot table was called uncapturable while
`GroupData::log_data` was dumping it every frame.

**Needs the user.** Nothing outstanding. Standing note: Fable is being
conserved, so first readings, blind readings and adjudications are all
running on Opus 5 — which the marker discipline permits and which books
the ratification debt named above. Verify the model from the transcript
(`lore spawns`), never from the spawn parameter.

**Opener:** `proceed @docs/QUEUE.md — apply docs/audit/2026-08-25-groups.md's "What must change" to docs/GROUPS.md and crates/sim/src/group.rs (nine verdicts change Rust; two are live bugs), striking each item in GROUPS.md §14 as it lands; then diff the whole GROUPDATA record from run29's window`

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
16. ~~**The group orders**~~ — done 2026-08-25, **less the adjudication
    of its second reading** (see "Owed" above). `docs/GROUPS.md`,
    `crates/sim/src/group.rs`; runs 28 and 29.

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
