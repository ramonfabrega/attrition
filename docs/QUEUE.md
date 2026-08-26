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

*Last verified 2026-08-26, after item 18.* The commit this section was
written against is the one that lands it; if `git log` has moved well past
it, trust the queue below and the journal before trusting this.

**Last landed.** **run29's `UNITS=3` half** (item 18). `scene_at` now
stands every unit of a block up — typed from its first guy's `TypeIndex`,
facing the record's own `angle`, carrying its formation bytes, its order
list front-first and its path stack — and gives each army the membership
of its `GROUPDATA` slot **in the record's own `list` order**. The parser
carries the whole `MOVEORDER` and `ATTACKORDER` rows and the order
layer's `UNITDATA` fields. Four new checks, in `rondata::diff` and
`sim::group`.

What it settled, and what it did not:

- **`engagement`'s choice of unit** (`docs/ARMY.md` §11, §18's item
  struck): the seed is `o 54`'s target, object 15, and the next block
  carries it. It also found **two bugs** — `is_engaged`/`engagement`
  tested the *front* order where `6f51fa` calls `get_action`, so the
  mechanic was dead on the one frame that reaches it; and the listing
  says `is_map_unit` gates only the loop's break, so an army with no
  map-unit target adopts the **last** qualifying unit's. The second is
  unobserved and on the ledger.
- **`find_leader`'s key** is implemented (`Sim::group_find_leader`,
  `type_cat`, `docs/GROUPS.md` §4.4) and unit-tested. Unobserved: every
  group in every dump has one category. The capture the old queue named
  would not have worked; §13 has the one that would.
- **§6.4's `to`/`off` asymmetry: run29 is not its capture.** The one
  group with offsets holds no orders; the one with orders has `form −1`.
  It wants a **human** group move — four units of one type,
  right-clicked, two frames of `DUMP_ALL`. Written into §6.4 and §13.
- **`update_positions`' y-flip: still unpinned, and run29 cannot pin
  it.** No group in the window has a non-zero `off_y`; the same human
  capture, in a formation with depth, is what would.
- Two things nobody was looking for: `action_halt`'s §7 write is
  **observed** (group `form −1`, every member `form 0`), and
  `get_form_mod_option` is a **mean over the members that have a byte**,
  not `get_form`'s all-agree twin — §4.4 said twin and was wrong.
- **A frame nobody could reach.** `full_dump` runs at `begin_frame` and
  `end_frame`, so a `DUMP_ALL` frame writes its dump twice (identical bar
  the checksum index, `turn_control`, the stamps and the timing). At the
  end of a run the second lands as a **sibling** of the `FRAME` block, so
  run29's free 15105 state was unreadable. `Log::dumps` finds it; the
  window is four states.

**Then, in order:**

- **The human group move** (new item 20). One run, two frames, and it
  closes three things at once: §6.4's `to`/`off` asymmetry,
  `update_positions`' y-flip, and — with two unit types in the selection
  and their headings apart — `find_leader`'s key. It is the cheapest
  capture on the list by a wide margin and it needs `cliclick` on the
  live game rather than the cheat channel.
- **The order's angle** (item 19, unchanged): `docs/GROUPS.md` §6.6 step
  6's `angle + (group.angles[i] << 24)` as a signed byte and an addition;
  the byte is computed and carried, the `add_move_facing_order` is not.
- **Item 13, differential fuzzing** — unchanged, and still the entry with
  the largest leverage per hour.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings the
  army's readers named; a `find_target` block where two candidates sit
  within a multiplier of each other; run7's order stream under the trace;
  a mounted attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**The thing this session earned.** Three of its five findings are the
same shape: *the record was already there and nobody had opened it*. The
order lists had been parsed since the orders mechanic and never loaded;
§4.1's field table had been read off the PE and compared with nothing;
the 15105 state had been on disk since the day run29 was captured. The
audit README's "diff the whole record" is about fields inside a record —
this session says the same thing one level up, about **records inside a
dump**. Worth a look before booking anything: what else does a dump on
disk already carry that no reader has opened?

**Needs the user.** Nothing outstanding. The ledger
(`docs/audit/README.md`) has two new rows from this session, both Opus
adjudications of live code: `engagement`'s last-qualifying fallback and
`find_leader`'s key. When to spend a Fable batch is still open.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — item 20, the human group move: one DUMP_ALL run of two frames with four units of one type right-clicked to a far point, which closes docs/GROUPS.md §6.4's to/off asymmetry, update_positions' y-flip, and find_leader's key at once`

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
20. **The human group move** — one `DUMP_ALL` window of two frames in
    which a *player* right-clicks a selection to a far point, which is
    the capture three open items are now waiting on and which the AI's
    lobbies cannot produce. Four units of one type is the minimum: an
    **even** column count is what displaces the formation block, which is
    what makes `docs/GROUPS.md` §6.4's `to`/`off` asymmetry visible —
    the members' `MOVEORDER` `x`/`y` against their `GROUPDATA` `off_x`.
    With a formation that has **depth** (more than one rank, or a second
    category) it also pins `update_positions`' y-flip, every `off_y` in
    every dump so far being zero. And with **two type categories** whose
    members face different ways, `curr` names the heading
    `update_positions` rotated by, which is the only observable that can
    pin `find_leader`'s key (§4.4). Driven with `cliclick` on the live
    game rather than the cheat channel, which cannot select or click;
    `tools/gamelog/window.py stage LO HI` is the rest of the setup and
    `docs/ORACLE.md`'s recipe is the run.

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
