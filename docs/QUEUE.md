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

*Last verified 2026-08-25, the session after `576fce2`.* The commit this
section was written against; if `git log` has moved well past it, trust the
queue below and the journal before trusting this.

**Last landed.** The **group orders** — `docs/GROUPS.md`,
`crates/sim/src/group.rs`, and the army's other half wired to them
(`docs/ARMY.md` §17): `do_forming`, `march_to_target`, `engagement`,
`send_here`, `charge`, `set_stance` and `Army::close`'s halt now issue
real orders, so an AI army's units walk to their muster spot and fight.
The find is `action_move_near`'s **AI branch** (`docs/GROUPS.md` §6.5),
which neither `docs/ORDERS.md` §8.2 nor the army's reading had: a
hurrying army stables its siege, wagons and heroes in the nearest
friendly city, and a non-hurrying one never interrupts a siege unit that
is already shooting. Both are tested and both were made to fail first.
**run28** made `Army::engagement` execute on a predicted frame (15100)
with the whole chain named; **run29**, a `DUMP_ALL` window at
[15100, 15103) of the same scenario, turned that into an **assertion**:
its two blocks show `status 1 → 32`, `city 1 → −1` and the point moving
to the muster cell's centre, and the sim's own `do_mustering` reproduces
all of it — including the absence of the `FORMING` bit, which is the gate
(`docs/ARMY.md` §16.6, §17 item 6). Journal entry of the same date.

**In progress.** `docs/GROUPS.md`'s **blind second reading is in
flight** — two readers on Opus 5, launched at the end of the session that
wrote it, split A (`action_move_near`/`compute_form`/`Form::*`,
`action_halt`, `action_stance`) against B (the pool, membership,
`action_siege_attack_to`, `action_attack`). Reports land at
`~/ghidra-projects/reading/groups-2026-08-25/{A-move-form-halt-stance,
B-pool-membership-attack}.md`, written incrementally, so a dropped agent
still leaves what it settled. Both were briefed with the run28 coverage
frames and told to name, per claim, the capture that would falsify it.

**Owed — and it is wider than the reading.** The whole of commit
`c53e4c6` was written, implemented and self-checked by **one model in one
session (Opus 5)**, with no independent pass over any of it. Treat the
tranche as unaudited, not just the document. What specifically wants a
second pair of eyes, hardest first:

1. **Three `docs/ARMY.md` predicates I changed mid-session** and that
   **changed Rust**, from my own re-reading of the decompile with no
   adjudicator: §8.4/§9's friendly test is `is_ally`, not `!is_enemy`;
   §9's 90 %-damage test's sense was inverted; and it applies only to a
   city centre. Under `docs/DECISIONS.md` entry 22 a verdict that changes
   Rust is exactly what a ratifying pass is for, and these never got one.
   `crates/sim/src/army.rs`, `army_target_is_a_friend_under_attack` and
   `march_to_target`'s `nearly_dead`.
2. **`docs/GROUPS.md` §6.5 and §9** — the AI branch and the siege
   sub-group. Both are predicates; both are new; both are implemented.
3. **The rest of `docs/GROUPS.md` and `crates/sim/src/group.rs`.**
4. **`docs/ARMY.md` §16.6's blind-list correction** — the claim that
   `engagement` had never executed was wrong, and the replacement list was
   derived by me from `report.py … blind docs`. Re-run it rather than
   trust the prose.
5. **The run29 test** (`§17` item 6) and the `strategy[reg]` input it
   declares.

**The second reading in flight covers 2 and 3 only.** Its adjudication:
Deliberately not done in the session that wrote the document — the first
reader adjudicating their own document is the conflict the three-role
split exists to prevent, so it wants a cleared context that reads
`docs/GROUPS.md` cold and goes back to the decompiled function for every
disagreement. Verdicts to `docs/audit/2026-08-25-groups.md`, appended as
each is settled; anything that cannot be settled marked `FABLE:` rather
than guessed. An Opus adjudication is acceptable under that marker
discipline (`docs/DECISIONS.md` entry 22) and books one debt: **a Fable
pass over every marker and every verdict that changes Rust**, before the
next mechanic builds on this one. §6.5 and §9 are the rows to read
hardest — both are predicates, which is where four mechanics running have
put the errors.

**Next**, in the order the captures suggest:

- **run29's window, the half not yet read: the units.** The `ARMY`
  records are asserted (§17 item 6); the block also carries, at
  `UNITS=3`, every unit's order list, and two things fall out of that.
  First, §11's *choice* of unit — the sim picks the first engaged member
  whose target is a map unit, and no dump has shown which the original
  picks. Second, the **slot table**: `Form::compute_dests` runs on frame
  0, so the `MOVEORDER` destinations of any early group pin
  `docs/GROUPS.md` §6.4's seam. Both need `scene_at` to load the block's
  `UNITDATA` order lists — the next afternoon's extension of it, and
  `scenes()` already parses a 250 MB window once for several blocks.
- **The dumped-record widenings the army's readers named** (unchanged
  from last session): `LEADERDATA` with `defense_mod`, `combat`,
  `sea_combat`, `strong[]`, `weak[]`, `pop_issues`, the two win timers,
  `frame_attacked`, `attacked_by`, `fort_mark`, `city_mark` and the
  personality; `CITY` with `bordering`, `was_capital_flags`, `founder`,
  `ocean`, and `city_flags` whole. **`GROUPDATA` is now worth more than
  it was**: 512 records a block, and `crates/sim/src/army.rs` carries a
  `group::GroupState` to diff them against.
- **A capture for `find_target`'s score, not just its choice**
  (`docs/ARMY.md` §18): a block where two candidates sit within a
  multiplier of each other — an ally's city against an enemy's at
  `defense_mod == 0x100`, or a difficulty-1 lobby with a Large City of
  the AI's own. The draw count and the choice are the observables;
  `runwin.sh` stages it.
- Then, as before: run7's order stream replayed under the trace, a
  mounted attacker, a caravan, the `found_cities` window at 576;
  `make_stuff` whole with the goods block; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`
  (`docs/TRANSPORT.md` §12).

**Three things this session earned.** (1) **Ask the blind list against
*every* log.** `docs/ARMY.md` had said `engagement` never executed; it had,
in run16, since 2026-08-24 — the claim was true of the four islands runs
and had never been checked against the corpus. `report.py <log> blind docs
<every log>` answers it in ten seconds. (2) **Predict the frame from the
state machine before staging the run.** run28 was three minutes because
the reading said which path reaches `engagement` and run27's record said
where the army stood. (3) A `cover=1` trace with **no** `DUMP_ALL` is a
three-minute run and answers "did this function ever execute"; the
ten-minute window is only for the records.

**Needs the user.** Nothing this session. One standing note: Fable is
being saved, so first readings and adjudications are running on Opus for
now, and each document says which model wrote it.

**Opener:** `proceed @docs/QUEUE.md — adjudicate the two blind readings of docs/GROUPS.md at ~/ghidra-projects/reading/groups-2026-08-25/ against the decompile and the listing, verdicts appended to docs/audit/2026-08-25-groups.md as each is settled, FABLE: on anything you cannot settle; then the queue's next item`

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
