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

*Last verified 2026-08-25, the session after `7e93e9a`.* The commit this
section was written against; if `git log` has moved well past it, trust the
queue below and the journal before trusting this.

**Last landed.** Armies (item 15): `docs/ARMY.md`, `crates/sim/src/army.rs`,
`docs/audit/2026-08-25-army.md`, runs 23–27 (`docs/ORACLE.md`), the `ARMY`
record diffed whole against run20 (frames 1–3, field for field) and read
against run22/25/26/27. The state machine's decisions are modelled; the
group orders, the muster-spot ring search and the fort pass are named
seams (`army::seams`). Two of the three windows end an army through the
ring search's failure at its own city (`docs/ARMY.md` §16.5, §18) — the
first seam a capture has reached, and the first thing to build once the
cell classes are loaded.

**In progress.** Nothing mid-mechanic. **Owed:** nothing — the audit's six
Rust changes landed with it.

**Next**, in the order the reading and the captures suggest:

- **The muster-spot ring search** (`docs/ARMY.md` §13, §18; audit B.44,
  B.56): `circle_init@006817f0`'s tables, the cell flag bits `0x4/0x8/0x20/
  0x40/0x100/0x800/0x4000` and the `lands[class].+0x100` score. Needs the
  per-cell land classes in the sim's `World`, which the `WORLD ≥ 5` dump
  carries. Closes the closes at 12129 and 15100, and makes run22's muster
  cells an assertion instead of a reading.
- **The group orders** — `Group::action_move_to` / `action_attack` /
  `action_siege_attack_to` / `action_stance` / `action_halt`,
  `Groups::push_group` — the half of the army the sim cannot issue
  (`docs/ARMY.md` §17). With them, `do_forming`, `march_to_target`,
  `engagement`, `send_here` and `charge` become behaviour, and `engagement`
  — never executed in any traced game — gets its first capture: a run
  where the attackers meet the army at its muster spot, not the city.
- **The dumped-record widenings the readers named** (audit §"What changed",
  B's §9): `LEADERDATA` with `defense_mod`, `combat`, `sea_combat`,
  `strong[]`, `weak[]`, `pop_issues`, the two win timers, `frame_attacked`,
  `attacked_by`, `fort_mark`, `city_mark` and the personality; `CITY` with
  `bordering`, `was_capital_flags`, `founder`, `ocean`. Nine tenths of what
  `find_target` reads is per-leader state the dump does not print. The
  `GROUPDATA` records (512 a block) are unread and would carry §3's group
  bookkeeping.
- Then, as before: run7's order stream replayed under the trace, a mounted
  attacker, a caravan, the `found_cities` window at 576; `make_stuff` whole
  with the goods block; `Leader::diplomacy`; `calc_gather` for non-flat
  buildings; `think_civilian_transport` (`docs/TRANSPORT.md` §12).

**Agreed with the user, 2026-08-25.** How the verification budget splits:
diff first wherever a dump exists; a blind reading scoped to what no run
reaches, its readers briefed to output assertions and handed the captures;
the soak kept as the determinism guard it is. The rules are in `CLAUDE.md`
("Prefer a diff to a reading"); the one build that changes the ratio is
item 13 below. Today's shape — read, stage the captures from the trace's
frame numbers with `tools/gamelog/runwin.sh`, spawn the readers while they
run, adjudicate against both — is the one to repeat.

**Needs the user.** Nothing this session. One caution for the next: a
`war` cheat is a no-op in a Quick Battle (it starts at war); `tools/gamelog/
rngcmp.py` shows in ten seconds whether a staged scenario took.

**Opener:** `proceed @docs/QUEUE.md — build find_muster_spot's ring search (docs/ARMY.md §13, §18; audit B.44/B.56): circle_init's tables and the cell classes into World, then make run22's muster cells and run25/27's closes assertions`

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
