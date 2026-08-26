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

*Last verified 2026-08-26, after item 17.* The commit this section was
written against is the one that lands it; if `git log` has moved well past
it, trust the queue below and the journal before trusting this.

**Last landed.** **`Form::compute`'s slot table** (item 17), whole, in
`crates/sim/src/form.rs` — `type_cat`, `categorize`,
`compute_rows_and_columns`, `compute_dests`, `get_form_mod_option`,
`update_positions` — with each member now taking **its own slot
destination** and `GroupState` carrying `form_num`, `off`, `curr` and
`angles`. The diff runs `unitrules.xml` → `x_spacing 660` →
`FORM_CAT_ARTILLERY` → `form_mod 50` → `cols 4` → the floor divide, and
lands on run29's own `[0, −14, 13, −28]` with `curr` matching across all
three frames. Eleven deliberate breakages, ten red on the first try; the
eleventh (the `update_positions` y-flip) stayed green for the known
reason and the test now says so in place. **The group orders are done,
slot table included.**

It also overturned **three verdicts this project's own audit had
accepted** (`docs/audit/2026-08-25-groups.md`, "Fourth pass"): the
formation block anchors on the **lowest**-indexed non-empty category, not
the last; `compute_dests` **drops the anchor's x from the destinations**
while keeping it in the offsets, so an even column count displaces a whole
group; and **formation 6, Square, is dead code** — three fields written
and no reader anywhere in the export. Plus two nobody could reach by
reading: a wedge's row count is seeded from **uninitialised stack**, and
`get_form_mod_option`'s value was printed in the dump (`form_mod 50`) all
along.

**Then, in order:**

- **run29's `UNITS=3` half** — the per-unit order lists nobody has opened,
  and now owed twice over. `scene_at` would need to load them; that pins
  `engagement`'s *choice* of unit (`docs/ARMY.md` §18); it puts a
  formation *with depth* in reach, which is the only thing that can pin
  `update_positions`' y-flip (every `off_y` in the window is zero); and it
  is the **only** capture that can see §6.4's `to`/`off` asymmetry, since
  `GROUPDATA` logs the offsets and not the destinations.
- **The order's angle** (`docs/GROUPS.md` §12, the `GroupMoveOrder` row).
  §6.6 step 6 gives a member's move `angle + (group.angles[i] << 24)` as a
  *signed* byte and an addition; the sim still uses `add_move_order`'s own
  bearing to the slot. The byte is computed and carried now, so what is
  left is an `add_move_facing_order` and `docs/ORDERS.md` §8.4's verdict
  on `GroupMoveOrder` — one entry, not two.
- **Item 13, differential fuzzing** (below) — unchanged, and still the
  entry with the largest leverage per hour.
- Then the older backlog: `find_leader`'s key is now computable
  (`sim::form::type_cat`) but the sim still takes the first on-map captain
  — a mixed army picks the wrong leader; the `LEADERDATA` and `CITY`
  widenings the army's readers named; a `find_target` block where two
  candidates sit within a multiplier of each other; run7's order stream
  under the trace; a mounted attacker; a caravan; `make_stuff` whole;
  `Leader::diplomacy`; `calc_gather` for non-flat buildings;
  `think_civilian_transport`.

**The thing this session earned, and it is about the method.** **Where a
mechanic's reading produces a *formula*, the implementation is the third
pass.** All three overturned verdicts sat under one adjudicated row —
"additions … as cited in A" — where the citations were real and nobody
re-derived the arithmetic; an adjudicator cannot check that row without
doing the work, and the Fable ratification spent budget confirming a loop
the compiler had already contradicted. Running the implementation *before*
the ratification would have been cheaper than after. (Corollary, cheap and
recurring: **grep the dump before booking a reading.**
`get_form_mod_option` was an open question in §13 and `form_mod 50` was in
the file.)

**Needs the user.** Nothing outstanding. The older Fable debt from the AI,
transport and army audits is still booked; whether to clear that ledger is
a later conversation, and the cheapest way to clear most of it is item
13's captures rather than a reading. Worth a decision at some point:
whether to reorder the working agreement so a formula-producing mechanic
is implemented before its ratification pass, per the lesson above.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — run29's UNITS=3 half: teach rondata's scene_at to load a block's UNITDATA order lists, which pins engagement's choice of unit (docs/ARMY.md §18), puts a formation with depth in reach for update_positions' y-flip, and is the only capture that can see docs/GROUPS.md §6.4's to/off asymmetry`

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
18. **run29's `UNITS=3` half** — `scene_at` does not read a block's
    `UNITDATA` order lists, and four open items all wait on the same
    capture: `engagement`'s choice of unit (`docs/ARMY.md` §18), a
    formation with depth for `update_positions`' y-flip, `docs/GROUPS.md`
    §6.4's `to`/`off` asymmetry, and `find_leader`'s key on a mixed army.
19. **The move order's formation angle** — `docs/GROUPS.md` §6.6 step 6
    and §12's `GroupMoveOrder` row, together with `docs/ORDERS.md` §8.4's
    verdict. The angle byte is computed and carried; the adder is not.

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
