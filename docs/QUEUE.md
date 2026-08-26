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

*Last verified 2026-08-26, after item 13's first tier.* The commit this
section was written against is the one that lands it; if `git log` has moved
well past it, trust the queue below and the journal before trusting this.

**Last landed.** **Differential fuzzing, Tier 1** (item 13) — built, run end
to end, and it changed its own plan twice on the way. `tools/fuzz/` holds
`scenario.py`, `seedini.py`, `run.sh` and `ledger.py`;
`tools/gamelog/console.py` re-derives the console vocabulary from the
install. `docs/ORACLE.md` has three new sections.

- **The channel cannot issue an order.** `run_cmd` jumps past its first
  switch when `from_chat` is set, and the two switches are disjoint: 56
  console-only, 45 chat-reachable. `move` is in the chat half and is a
  **teleport** (`Unit::set_new_location`). So every `add_*_order`, `do_*`,
  `action_*` and `process_*` on the blind list needs the UI, and Tier 1
  cannot touch them. Closes run17's open `move` question.
- **`restart` from the channel wedges the game.** It fires inside
  `Game::do_frame`, so `Game::close`/`Game::init` tear down the game whose
  tick it is in: `parse_cmd` never returns, the screen goes black, no
  further frame. The seed goes in `rise.ini` instead — one launch per seed.
- **The fuzzer's first seed found a panic.** Seed 424242 drove
  `Sim::is_enemy` with the nature player (`who 8`, 7,329 records in its
  dump) against diplomacy tables sized by the lobby's two players:
  `index out of bounds: the len is 2 but the index is 8`,
  `crates/sim/src/lib.rs:930`. run29's dump carries `who 8` too and does
  **not** panic, so this is a path 31 hand-built runs never reached, not a
  new input. **Unfixed, and it is the next session's first job.**

**The dump's cost, measured — and half of it is ours to stop paying.**
A `FULL DUMP` block is ~25–30 MB and about a minute. With indentation
normalised, **12.9 MB of 25.1 MB (51%) is byte-identical across all six
blocks compared** — `COMBATTABLE` 36%, `UNITTYPE` 14%, and the small type
tables. That is the rulebook, and we already have it from the XML.
`DUMP_ALL=1` is what forces it, and `window.py stage` sets `DUMP_ALL` for
one reason: `scene_at` asserts on a `WORLD` block at the stand-up frame.
**`WORLD` is 31% and genuinely changes**, so it cannot simply be taken from
the start block — that was claimed here first and disproved by checking.
The cheap experiment nobody has run: `DUMP_ALL=0` with `[End Frame] WORLD=6`
plus the state categories, which should be ~9 MB rather than ~25 with no
code change. And gzip on a real dump is **57×** (60 MB → 1.06 MB).

**Then, in order:**

- **The `who 8` panic**, above. Small, and it blocks every further seed.
- **The cheap-window experiment**, above — it decides whether a fuzzed
  seed can score over hundreds of frames instead of three, which is what
  decides whether item 13 is worth keeping at all.
- **Item 23, the hand-back's inversion** — unchanged, cheap, unblocked.
- Then the older backlog: the `LEADERDATA` and `CITY` widenings; a
  `find_target` block; run7's order stream under the trace; a mounted
  attacker; a caravan; `make_stuff` whole; `Leader::diplomacy`;
  `calc_gather` for non-flat buildings; `think_civilian_transport`.

**The thing this session earned.** **Measure the oracle before building a
better one.** A whole afternoon's plan — an in-process binary dumper to
replace the game's logger — was retired by one twenty-minute measurement
showing the logger is expensive for a reason we control. And the corollary,
which cost a wrong claim in this very file: **"static" is a claim about what
changes between frames, so check it between frames.** The first pass called
85% of a block static by reading section names; the honest figure is 51%,
and the difference was found by hashing the sections rather than arguing
about them.

**Needs the user.** Nothing blocking. The ledger (`docs/audit/README.md`) is
unchanged; its widest marker is still **`sin_table@00a46a00`'s
second-quadrant branch**, and this session found a better settlement than
another reading: the DLL runs **in-process**, so the original's own
`sin_table` can be *called* over its whole 2^30 input domain and compared
with `quarter_lookup` exhaustively — total, not symbolic, and minutes of
brute force. That rig would then serve every ported leaf formula. When to
spend a Fable batch is still open; this session's judgement is **not yet**,
because the design question that fit Fable's mandate was the binary dumper
and the measurement shelved it.

**Opener (for an Opus session):** `proceed @docs/QUEUE.md — fix the who-8 panic in Sim::is_enemy (crates/sim/src/lib.rs:930, seed 424242), then run the cheap-window experiment: DUMP_ALL=0 with [End Frame] WORLD=6, and see what a 300-frame window costs`

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
13. **Differential fuzzing against the original** — **Tier 1 built and run
    2026-08-26**; whether it is kept is an open question with a named
    experiment, below. `tools/fuzz/{scenario,seedini,ledger}.py` and
    `run.sh`; `tools/gamelog/console.py`; `docs/ORACLE.md`, "The channel's
    vocabulary, and what it cannot do" and "`restart` from the channel
    wedges the game".

    Three of the entry's own assumptions were wrong, and each was settled by
    a run or a measurement rather than an argument:

    - **A scenario cannot contain an order.** The entry said it could
      ("move, gather, build, attack, garrison"). The chat half of the
      console is 45 state pokes and `move` is a teleport. So Tier 1 varies
      the map and the staging, and reaches **none** of the blind list's
      `add_*_order` / `do_*` / `action_*` / `process_*` family. That is
      Tier 2's job and it needs the UI.
    - **`restart` cannot drive it.** In-tick re-entrancy wedges the game.
      One launch per seed, the seed in `rise.ini`.
    - **A seed costs ~6 min and ~50 MB, not "one to two minutes".** And
      the score is over three frames, not hundreds, because the window has
      to be narrow.

    **What it has already earned:** its first seed found a panic 31
    hand-built runs and the soak never reached — `Sim::is_enemy` with the
    nature player against a two-player table. That is the argument for
    keeping it, and it is one data point.

    **The experiment that decides it:** `DUMP_ALL=0` with `[End Frame]
    WORLD=6` and the state categories. 51% of a `FULL DUMP` is the rulebook,
    byte-identical across frames and already ours from the XML, and
    `DUMP_ALL` is what forces it. If a block drops from ~25 MB to ~9 MB, a
    seed can score over hundreds of frames and the ledger means something.
    If it cannot, Tier 1 is a harness waiting for an oracle it does not
    have, and the honest move is to keep `scenario.py` and the ledger and
    stop there.

    **What it still cannot do**, unchanged: reach what the channel cannot
    stage — diplomacy beyond the verbs, the sea half until transports can be
    ordered, CtW, multiplayer. For those the reading stays the only
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
