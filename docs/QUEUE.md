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

*Last verified 2026-08-25, the session after `ced7bc8`.* The commit this
section was written against; if `git log` has moved well past it, trust the
queue below and the journal before trusting this.

**Last landed.** The AI's sea half, as its own mechanic: `docs/TRANSPORT.md`
(the transport level, the unit bit, the docks registry, `is_dock_tile`,
`think_civilian_transport`, the army's transporting step, the navy hooks),
`crates/sim/src/transport.rs` with the census's two seams closed
(`num_coasts`, `is_dock_tile`) and `carry`/fishermen wired, the blind second
reading (`docs/audit/2026-08-25-transport.md`, 114 claims, two Rust
changes), and **run22** — the run21 lobby under a `DUMP_ALL` window at the
frame the trace gave for the first dock, eight minutes end to end, staged by
`tools/gamelog/window.py`. Two assertions on disk: run20's `CITY` record
widened to all of step 13 (`dock_tile` 1 matches), and run22's `DOCK`
record and the 14-unit `0x800000` flip, both passing and both made to fail
once. Item 14 is struck below.

**In progress.** Nothing mid-mechanic. **Owed:** one finding the run20
widening produced, pinned as a known divergence rather than fixed —
`space[0]`/`space[1]` of the AI's `CITY` record (ours 48, theirs 58):
`WorldData::check_building_wcoord` passes its `space_at_corner` a fifth
argument (the corner's `|dx| + |dy|`, or the running best on an axis) that
`ai_place.rs`'s four-argument form does not. A placement item
(`docs/CITIES.md`), a morning's work, and `diff.rs`'s run20 test is the
guard that flips when it lands.

**Next.** The blind list is now 471 cited by address, 375 entered over
eleven traces, **96 never** — the rise is `TRANSPORT.md`'s 33 new
citations, of which `Dock::close`, `close_dock`, `remask_docks`,
`find_dock`, `send_navy`, `coast_here`, `action_set_transport` have never
run. Each is one cheat-channel scenario: a dock destroyed (`cheat die` on
the dock's `o`), a land army with a cross-sea target (needs run21's game
past 14586 under a window), a citizen dispatched off-island (`think_
civilian_transport` succeeding — a window past 3608 with `UNITS=3`,
`docs/TRANSPORT.md` §12 item 3). The one reading this closes points at:
`docs/ARMY.md` — the `Army`/`Armies` family now has its transporting arm,
its cadence (audit B.45), its merge path (B.67) and its eviction rule
(B.59) written down by the second reading and nowhere else. Then, as
before: run7's order stream replayed under the trace, a mounted attacker,
a caravan, the `found_cities` window at 576; `make_stuff` whole with the
goods block; `Leader::diplomacy`; `calc_gather` for non-flat buildings.

**Agreed with the user, 2026-08-25.** How the verification budget splits:
diff first wherever a dump exists; a blind reading scoped to what no run
reaches, its readers briefed to output assertions; the soak kept as the
determinism guard it is. The rules are in `CLAUDE.md` ("Prefer a diff to a
reading"); the one build that changes the ratio is item 13 below. Today's
shape — read, stage the capture from the trace's frame numbers, spawn the
readers while it runs, adjudicate against both — is the one to repeat.

**Needs the user.** Nothing this session: the lobby's clicks were driven
(`docs/ORACLE.md`, run22's driving notes). A window run is now `window.py
stage LO HI`, launch, five `cliclick`s, wait, `restore`.

**Opener:** `proceed @docs/QUEUE.md — take the check_building_wcoord / space_at_corner finding (the run20 CITY record's space[0..1], docs/CITIES.md), or open docs/ARMY.md from the second reading's army rows and run21's frames`

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
   recipe is `docs/ORACLE.md`, "Running a check: the recipe in one place";
   what is still open is owed item 4 above.
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
