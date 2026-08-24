# attrition

A deterministic, ground-up reimplementation of **Rise of Nations** (Big Huge
Games, 2003) in Rust — simulation first, art last.

Named for the mechanic no open-source RTS has ever implemented: units bleeding
health inside hostile national borders.

## Thesis

Rise of Nations' Extended Edition depot ships **full, unstripped private debug
symbols for the shipped executable** — `game/sbl/rise.pdb`, GUID-matched to
`riseofnations.exe`. 5,880 struct layouts, 1,251 source file paths, function
names and line numbers. Alongside them: intact RTTI, a 13 MB linker map, and
947 lines of the engine's own C++ left in `obsoletescriptfuncs.txt`.

So the original is not a black box to be probed. **It is a readable
specification**, and the job is translation rather than archaeology.

That changes what fidelity is for. The old plan reached parity before changing
anything, because divergence destroyed the only oracle — a recorded-game diff —
and every later bug became unfalsifiable. With per-subsystem ground truth
available directly from the symbols, that trap is gone. Parity stops being a
gate and becomes a menu.

So we copy what is worth copying and leave the rest:

- **The rules are the treasure.** Twenty years of tuned balance, the age and
  tech pacing, supply, borders, attrition. Full fidelity, verified against the
  original.
- **The engine is not.** Hardcoded eight-player arrays, positionally-parsed
  XML, a 2002 scripting VM. We are here to escape that, not to reproduce it.
- **The art is the upgrade.** Original assets are the visual oracle while the
  renderer is built, and then they go.

The end state is our own game that plays like the best RTS nobody maintains.

## Architecture

**The sim/renderer split is the load-bearing decision.** Everything else is
downstream of it.

- **sim** — headless, deterministic, no engine dependency, fixed tick rate.
  Knows nothing about pixels, windows, or input devices. ~90% of the work.
  Runnable in a test harness with zero graphics.
- **renderer** — a thin client over observed sim state. Swappable. This is
  where art lives, and the only place it lives.

That split is what makes the replay differ possible, keeps determinism
testable, and makes the eventual art swap free.

## Hard constraints

- **No floating point in the sim. Ever.** Not `f32`, not `f64`, not "just for
  this one distance check". Float results vary across compilers, architectures,
  and optimisation levels; a lockstep sim that varies is not a sim. Gameplay
  arithmetic is **integers at the original's own scales** — 8.8 where it keeps
  8.8, hundredths where it keeps hundredths, an exact rational where it keeps
  the one `f32` it has, and a pinned table where it builds one with doubles
  before the first frame. `fixed::Fx` is not the rule; it is a crate that stays
  unearned until a mechanic genuinely needs a fraction the original does not
  already store as an integer. See `docs/DECISIONS.md` entry 16.
- **The sim crate depends on no graphics, windowing, or async runtime.** If it
  cannot run in a `#[test]` with no display attached, it is wrong.
- **Nothing from the user's install ever enters this repo.** Not models, not
  textures, not audio, not the shipped XML — and not PDB dumps, symbol lists,
  or decompiler output. `/game` is gitignored. Tools take an install path and
  generate what they need on demand. This is both the legal line and the
  OpenTTD model.
- **The decompiler is a reading tool, not a source.** Read anything; write from
  understanding. Decompiled function bodies are never transcribed into Rust —
  that would import the design we are here to escape, for no gain, since the
  expensive part is understanding a mechanic rather than typing it.
- **Every format claim is evidence-backed.** No guessing at a struct layout.
  If we assert a field, `docs/FORMATS.md` cites what proves it.

## Phases

Each phase produces something independently valuable, and each phase's
artifact is the next phase's tool.

0. **Extract** — one tool that reads the user's install and emits the XML
   tables as typed, index-keyed open data, plus the PDB type stream as
   documented layouts. Turns `docs/` into a real specification of RoN's data
   model and sim state. Weeks, not years; publishable alone; immediately useful
   to the RoN:EE modding community.
1. **Attrition** — one mechanic, end to end, headless. Borders → territory →
   damage, and supply cancelling it. It is the namesake, it is self-contained,
   no open-source RTS has it, and everything needed is in reach: `borders.cpp`
   in the symbols, and `TERRITORY_BASE`/`_DEN`/`_NUM`/`_LIMIT_*`,
   `CITY_TERRITORY_MULTIPLIER`, the `*_UPGRADE_TERR` arrays, and
   `ATTRITION = 48 frames` in the data. If this comes out exactly right, the
   method is proven and the rest is repetition. **Done**, specified in
   `docs/ATTRITION.md` and `docs/SUPPLY.md`.
2. **Run the original** — 32-bit x86 Windows on Apple Silicon. **Done**, via
   CrossOver's D3DMetal, after Wine, DXVK and wined3d/Vulkan all failed on the
   renderer's D3D11 feature-level requirement; `docs/ORACLE.md` has the exact
   path. It gives us a visual and behavioural oracle, and lets us check any
   claim instead of reasoning about it.

   **What it unlocks is bigger than recorded games**, per `docs/ORACLE.md`. The
   shipped executable contains two loggers. `SyncLogger` (the EE-era desync
   tracer, `synclogger.ini`, 37 per-category keys) writes its frames only on an
   actual desync. The older `Log` system (`AllowLogs=1` in `rise.ini`, then
   `gamelog.ini`) dumps chosen subsystems' state **every frame** to
   `Logs\gamelog.txt` in a nested text format — every unit's position, every
   leader, the loaded `Constants` struct by name — and `Seed (0 for random)`
   in `rise.ini` makes a run reproducible. That file, not a recording, is the
   per-frame ground truth the sim is diffed against.
3. **Sim skeleton** — economy, one unit type, movement. Replay a recorded game
   and diff. Score is ticks before divergence. This is the long middle.
4. **Renderer** — thin client. Original assets first; they are the visual
   oracle.
5. **AI** — hardest, least-oracled, and less bad than it looked: build order
   and economic posture are scripted in the open under `game/ai/scripts/`, and
   the whole BHS toolchain is enumerated in the symbols. Combat and target
   selection are still in the executable. Ship vs-human first.
6. **The fork** — swap the assets, then build past the original.

**Cut from v1**, to be revisited only once the above stands up: Conquer the
World, the scenario editor, the trigger system, GameSpy and the multiplayer
meta, and the ~90 `iface*` windows. That is roughly half of the 796 files in
the engine's `game/` module. CtW is genuinely good and worth building; it is
not worth building first.

## Working agreement

Phase 3 is a long middle, and it is done one mechanic at a time. This is what
has been working, written down so a fresh session can pick up without
re-deriving it.

**The queue, in dependency order.** A default rather than a contract — take the
next unstarted one unless something has made a different order obviously
better, in which case say so and take that.

0. ~~**Corrections from the second reading**~~ — **done, 2026-08-20.** All six
   mechanics' audits under `docs/audit/` are landed; each document now ends
   with a "Second reading — landed" section, and each place a claim changed
   says so inline. The ones that changed observable behaviour: attrition's
   sixteenths-damage and bleeding wagons, production's library-only fan-out
   and research-that-trains-nothing, costs' redirect table, movement's
   unit-versus-body step. Done the way the next one should be: one worker per
   mechanic, each re-verifying every claim against the decompile before
   changing it, document first, then implementation and tests.
1. ~~**The tech tree**~~ — **done, 2026-08-20**, `docs/TECH.md` and
   `crates/sim/src/tech.rs`: `has_tech`, `get_preq`, `has_preq`,
   `type_eligible`, `type_avail`, `queue_here`, `gain_tech` with its cascades,
   `lose_tech`, `set_age`, the starting position, the lobby's start/end ages.
   The `TypeIndex` enum is now dumped whole by
   `tools/ghidra/scripts/DumpEnumAll.java`. Second reading done and landed
   (`docs/audit/2026-08-20-tech.md`): it found the loaders' derived fields —
   every combat unit implicitly needs its age's Military epoch — which a
   reading of the predicates alone cannot see. Read the loader too.
2. ~~**Combat**~~ — **done, 2026-08-20**, `docs/COMBAT.md`, `crates/sim/src/
   combat.rs` (the arithmetic), `fight.rs` (the attack step, targeting,
   projectiles, buildings) and `balance.rs` (the combat table's generation).
   The damage formula end to end, sixteenths delivery, the recharge cadence,
   accuracy/scatter/flight time/hit test/splash, the game's RNG, buildings'
   arrows, the target search and ranking. Second reading done and adjudicated
   (`docs/audit/2026-08-20-combat.md`). What it left as inputs: the nation,
   wonder and patriot modifiers, terrain flags, the `type_damage` lineage
   predicates that need a tree loader; what it left open: the flank direction
   convention (a behavioural check) and the `balance.xml` flag-row separator
   (the `RULES=1` log settles it).
3. ~~**Cities and buildings**~~ — **done, 2026-08-20**, `docs/CITIES.md`,
   `crates/sim/src/build.rs` (the type, the construction clock, the site's
   hit points, repair, refunds), `place.rs` (the tile layer and the
   `blocked_site` predicates with the original's `BlockIndex` verdicts),
   `city.rs` (the record, membership, the automatic level-up, capture,
   assimilation, the city heal, plunder, default-mode elimination) and
   `garrison.rs` (the FIFO chain, `can_garrison`, the `do_garrison` gates,
   one-squad-a-frame ejection, the garrison heal). Read by five readers in
   parallel, one per sub-area, and re-read blind the same way
   (`docs/audit/2026-08-20-cities.md`). Two of the first reading's claims were
   settled in the disassembly rather than the decompile — the argument
   `do_build` passes and the value `capture_strength` takes — which is the
   cheap move whenever the decompiler prints a local that cannot be right.
   What it left as inputs: the nation, wonder and tech layers (`city::Nation`,
   `build::BuildMods`/`ClockMods`/`HitsMods`), the per-wonder one-offs of
   `activate`, the AI branches, visibility, the capital-countdown elimination
   mode.
4. **The behavioural-check batch** — **run 2026-08-20, two sessions; the
   method is proven and the list is mostly closed.** What it took was finding
   the loggers' real switchboard (the ini values are per-category *detail
   thresholds*, `UNITS=3 BUILDS=6 CITIES=5` is the setting, `DUMP_ALL=1` is a
   different switch that hangs the game but is the only way to the type
   tables and `COMBATTABLE`), the chat-cheat vocabulary, the `~` console and
   its tile-coordinate mode, `-config`/`-automation` for the lobby, and the
   fact that `cheat select <o>` plus a right-click issues a normal order —
   which made every remaining check scriptable. All of it is one recipe in
   `docs/ORACLE.md`, "Running a check: the recipe in one place", with the
   scripts in `tools/gamelog/`.

   **Landed:** the construction clock, both halves — one builder is +100 a
   frame, two are +150, the clock re-bakes on a tech change for unstarted
   sites only (`docs/CITIES.md` §3.2–§3.3); the lobby start age grants no
   epochs, techs or pop cap (`docs/TECH.md`); a city is refused at exactly 24
   tiles and starts at 25, on two bearings, with the tower-site-first control
   that makes every other `blocked_site` verdict testable (`docs/CITIES.md`
   §2.6.2); the city heal, period 4 and one point per level (`docs/CITIES.md`);
   `UnitData.angle` is the facing, so the flank convention is settled at its
   premise — level 1 is the rear (`docs/COMBAT.md` §14.1, `docs/MOVEMENT.md`);
   the `balance.xml` separator, settled against the dumped table (see 6).

   **Still open, none blocking, each a 20–60 minute drive when the game is up
   for another reason:** the city mask's remaining three rungs (recipe in
   `docs/CITIES.md` §3.6); the garrison heal — no script path into a building
   found yet, the remaining route is `Build::train` + `cheat finish`
   (`docs/CITIES.md` §15); the Nubian attrition step, which needs attrition to
   actually run (`docs/ATTRITION.md`); the flank damage ratio, for which
   `tools/gamelog/hits.py` is the instrument and one damage source on the
   target is the setup (`docs/COMBAT.md` §14.1); and, new from the
   `BUILDS=7` run, **`FARM_GROWS`** — the original re-targets a farmer at
   frame 102 against the documented ~200, which the harness's order diff
   measures for free on any `UNITS=3` dump (`docs/ORDERS.md` §6.5).
5. ~~**The data layer into the sim, and the diff**~~ — **done, 2026-08-20**,
   `docs/DATALAYER.md`, `crates/rondata/src/{gamelog,dump,load,diff}.rs`:
   the gamelog reader, the loader (364 units, 129 buildings, 628-entry tree,
   the combat table — every key resolved, by `TYPENAME`), and the harness
   (`rondata <install> --gamelog <dump> --diff`). The dump's `CONSTANTS`
   block checked 231 of 232 `Tuning::RON` slots against the program's own
   loaded values and classified the representation of all 716. Taken out of
   order, before 4, because the game window was on the user's screen; the
   harness's score is the expected ceiling (1 tick — no AI, no orders) and
   its next inputs are `DUMP_ALL=1` and an order stream. Two corrections
   landed on the way: XML *fields* are read by tag name from
   `internal_strings.xml` (records stay positional — `docs/FORMATS.md`), and
   `RULES=1` does not dump the type tables (`docs/ORACLE.md`).
6. ~~**The combat table's hardcoded half**~~ — **done, 2026-08-20**,
   `docs/COMBAT.md` §15.2, `crates/rondata/src/typesdump.rs` and
   `rondata --types <dump>`. The start-of-game dump's `COMBATTABLE` is the
   engine's own composed table, and its `UNITTYPE` blocks are the inputs —
   so the check compares both, the 364 `Kind`s field by field and then the
   364 × 364 block. It found three predicate errors, not thirty: the age one
   too low for 306 units (`get_age_slow` adds 1 to an age tech's `AGE`), four
   lineage roots keyed by a display name that was the wrong unit (`ECOMPANION`
   is *Royal* Companion), and `0x14000` misread as M|O where it is O|Q. With
   them landed **0 of 132,496 cells differ**, and the check is the
   regression guard from here. The `Flag_` rows are dead data (§14.9), landed
   with it. The building half followed the same day (§15.3): `combat::Table`
   is two-family, `rondata` builds a `Kind` per building, the fight path
   applies the table to every pair, and **all four quadrants of the
   493 × 493 are equal** — the one finding on the way was a building's
   domain, `BuildType::set_domain` from `BUILD_FLAGS b`.

7. ~~**Orders**~~ — **done, 2026-08-21**, `docs/ORDERS.md`,
   `crates/sim/src/orders.rs`: the order list and its three enqueue modes,
   one order stepped per frame, the move order end to end with the
   pathfinder as a named seam, build/repair/garrison through the swarm ring,
   the gather registration and the wood/ore machine, the attack order's
   reload gate and chase, the idle cadence, the start of a game, the spot
   search, the log format. Taken before AI because AI issues orders and the
   harness's score could not move without them. The blind second reading and
   all seven adjudications are landed (`docs/audit/2026-08-21-orders.md`),
   and so is the harness work: the start of a game in `build_sim` and the
   `UNITS=3` order blocks read back and diffed **every frame**
   (`docs/DATALAYER.md` §3.1). The intent diff earned itself immediately —
   six farm citizens holding `THINK` where the original holds `GATHER`, from
   frame 1, with both simulations agreeing on every position. The
   `BUILDS=7` dump that was blocking it **has been captured**
   (`gamelog-run6`, `docs/ORACLE.md`), `gather_from` and `orig_type` are
   read into the harness, and **a woodcutter's citizen now matches the
   original's position and order list for all 432 logged frames**. What
   still cannot move: the AI's units (they need the order stream) and the
   farmers, which part at frame 102 — the run's own finding, that
   `FARM_GROWS = 200` is wrong by about a factor of two (`docs/ORDERS.md`
   §6.5).
8. ~~**The pathfinder**~~ — **done, 2026-08-23**, `docs/PATHFINDER.md` and
   `crates/sim/src/path.rs`: `astar_path` on all three grids (greedy
   heuristic, LIFO tie-break, metric-keyed dedup, budgets, suspend,
   reconstruction), `calc_cost` with every term modelled or a named seam,
   the three wrappers with their pre-walks and post-processing,
   `invalid_loc` with the original's five flags, `do_move`'s planner wiring,
   and §4.6's march fixed point **settled** — the original recomputes the
   angle every step and clamps each axis, so the 2026-08-22 guard and its
   sync divergence are retired. `PathLength`/`PathTo` **score** now; on
   `gamelog-run6` the first woodcutter matches all 432 frames and the
   second's path stack matches for 427 (both pinned as tests). Every
   remaining path disagreement is player 1's units crossing **forest the
   flat harness world does not carry** — so the next path-diff win is
   world data (the map's tile layer into the harness), not search. Found
   on the way: flat gatherers' footprints were wrongly `BLOCKED`
   (`mask_me`'s template only blocks marked tiles), and the cost function
   clamps its additive term at zero, so the own-territory −4 only ever
   offsets danger. The blind second reading ran the same day and is landed
   (`docs/audit/2026-08-23-pathfinder.md`): 32 verdicts, the structure
   doubly confirmed, five behavioural corrections applied — among them
   `leaders.flags & 4` = `is_human` gating `army`/`worker` (which exposed
   the gamelog parser handing every leader its successor's flags — fixed),
   the pull-back walk's give-up exit, and the corner-cutting probe table,
   settled in the listing. Still open: `go_around_building`, collision
   recovery (`find_upath`'s caller), suspend/restore stashing, and the
   forest tile layer for the harness world (the remaining run6 path gap).
9. **AI** — last, because it is the least oracled, and better oracled once
   4, 5 and the pathfinder stand.

**One mechanic per session.** The document is the handoff: a fresh session
reads `docs/<MECHANIC>.md` and knows what the last one knew. That is what makes
`/clear` between mechanics free, and it is why the document is written before
the implementation rather than after.

**Definition of done**, all five:

- `docs/<MECHANIC>.md`, stating how it was established, how confident it is,
  and what it has *not* established.
- The implementation, in its own module.
- Tests, including the end-to-end kind that run the new mechanic against the
  ones already there.
- `cargo test`, `cargo clippy --all-targets` and `cargo fmt` clean, and
  `cargo run -p rondata -- <install>` exiting zero.
- Committed. Any open question this closes in another document is struck
  through there and pointed at its answer, per the amend-in-place rule below.

**Keep going while the path is clear; ask when it isn't.** That is the whole
rule, and it is what the sessions so far have actually done. Uncertainty inside
a mechanic is usually not a reason to stop — implement under a stated
assumption, record it under "What is not established", carry on. Uncertainty
about *direction*, a divergence worth making deliberately, anything
irreversible or outward-facing, or a finding that changes what the project
should do next: those are worth a conversation, and the conversation is cheap.

In practice the natural boundary is the end of a mechanic. Finish it, commit
it, say where things stand and what you would do next — then it is a good
moment to clear the context and start the next one fresh, because the document
carries everything forward.

**A behavioural check is a logged run, and it is cheap.** The original runs
here (`docs/ORACLE.md`, last section): fix the seed in `rise.ini`, enable the
mechanic's categories under `[End Frame]` in `gamelog.ini`, play a minute, quit
through the in-game menu, read `Logs\gamelog.txt`. A claim that needs a
behavioural check is still written into the document's open questions with the
check named — and then, when it is the cheapest way to settle it, the check is
run rather than deferred. Do not enable everything per frame; it slows the
simulation to a crawl.

**The checks with teeth are the ones that can fail.** Three guards now stand
outside the per-mechanic tests, and each was written by first making it fail:

- `crates/sim/src/no_float.rs` reads the simulation's own source and rejects
  any float outside `#[cfg(test)]` — the one hard constraint that was
  enforced by convention until 2026-08-22. It allows the software float
  (`combat::F32`, integer mantissa) and the test oracles, and nothing else.
- `crates/sim/src/soak.rs` generates games from a seed — a scenario, and a
  stream of orders including unreasonable ones — plays each **twice**, and
  compares a per-frame digest. It found a non-terminating loop in
  `find_path`'s march within an hour of existing (`docs/ORDERS.md` §4.6).
  The old determinism test ran one hand-built scenario; this runs a hundred
  it did not think of.
- `rondata::diff`'s `the_original_s_own_run_is_still_matched_frame_for_frame`
  pins the harness against the original's own 432 frames, so the state of the
  port is a test rather than a number in a commit message.

A guard that has never failed has not been tested; make it fail on purpose
once, then land it.

**Every mechanic gets a blind second reading before it is called done.** One
reader writes the document from the decompile; a second, who has not seen the
document or the implementation, re-derives the same mechanic from the same
export and writes a report; a third adjudicates every disagreement back to the
decompiled function and records the verdicts under `docs/audit/`. The first
pass over the seven existing mechanics (2026-08-20) found the arithmetic
doubly confirmed almost everywhere and the *predicates* wrong in several places
— which unit kinds are exempt, which step the 11/8 belongs to, which array a
level indexes — exactly the kind of error that tests written from the same
reading cannot catch. The full decompile export under `tools/ghidra/` is what
makes the second reading cost an hour rather than a session. The cities audit (the tenth
mechanic, same day, five readers each way) added a sibling to the tech audit's
"read the loaders": **grep the writers of every field you call frozen, and
the callers of every function you call once-only** — its corrections were a
caller nobody looked for (`calc_wall_stats`), a table taken on trust
(`even_circle_init`) and a gate read with its sense inverted, all at the edges
of the first reading's scope rather than inside it. And when the decompiler
prints a local that cannot be right, the listing (`llvm-objdump`) or the PE
bytes settle it in a minute.

**Emit traces under the original's own names.** `docs/ORACLE.md` lists the 37
`SyncDefine` categories the engine considers sync-critical. Where a mechanic
maps onto one — `LeadersSync`, `UnitsSync`, `BuildsSync`, `WorldSync`,
`GoodsSync`, `DeathsSync`, `TerrainSync` — use that name. It costs nothing now
and makes the eventual diff mechanical rather than a translation exercise.

## Conventions

- **Earn every dependency.** Crates appear in this workspace when they have
  real code, not in anticipation. Same for third-party deps.
- Toolchain is pinned in `rust-toolchain.toml` so the Solana toolchain on this
  machine can never leak in.
- Format recon notes live in `docs/FORMATS.md`; decisions and their rationale
  in `docs/DECISIONS.md`. A decision that gets overturned is amended in place
  with its successor named, never deleted.
- One document per mechanic, written from the original and implemented from the
  document — `docs/ATTRITION.md` is the first. Each states how confident it is
  and lists what it has not established, so a reader can tell a derived formula
  from a plausible guess.

## Tooling

- `llvm-pdbutil` (Homebrew LLVM) reads `rise.pdb` on macOS. `dump --types`
  works; `pretty` needs the Windows DIA SDK and does not.
- Ghidra 12.1.3 (`brew install ghidra` — a formula now, not a cask; it wants
  `openjdk@21`). With the PDB loaded it gives named, typed decompilation.
  **`tools/ghidra/` holds everything**: `analyze.sh` builds the project once
  (hours), `export.sh` decompiles all 48k functions plus every struct and
  vtable to files (minutes), and after that reading is `grep` over
  `decomp/` rather than a two-minute pass per question. `run.sh` runs the
  remaining one-off scripts. Its README lists the traps that have each cost a
  wrong conclusion once. The two things the decompiler does not do for you —
  name the field behind a `field_0xNN`, name the method behind
  `(*(code **)(*this + 0xcc))()` — are `types.txt` and `vtables.txt` in the
  export.
- **The original runs on this machine.** CrossOver (D3DMetal) in a bottle
  named `ron`, launched with
  `wine --bottle ron --workdir <install> <install>/riseofnations.exe`; see
  `docs/ORACLE.md` for the two loggers it ships and how they are switched on.
  `cliclick` drives it; System Events clicks do not reach it.
- Constants are not all loaded in the representation the file writes. At least
  one rational arrives scaled to 8.8 fixed point. Read the consumer before
  believing the digits.
- `cargo run -p rondata -- <install>` surveys the data layer and re-derives
  every structural claim in `docs/FORMATS.md` from the user's own files. If a
  claim stops being true it exits non-zero. Run it after touching anything
  that reads the game's data.

## Prior art worth reading

- **OpenRA** (C#) — the reference implementation for lockstep order
  serialization and a data-driven mod layer. Read it for the hard parts.
- **OpenTTD / OpenGFX** — proof of the full inside-out arc, end to end.
- **Beyond All Reason** — Total Annihilation lineage that freed itself of
  proprietary assets and became a standalone game.
- **ptasev/Rise-of-Nations** — existing BH3/BHA ↔ glTF converters and a BIG
  archive extractor. The only serious RoN format work that exists publicly.
- **banteg's Crimsonland writeup** — the method: exe-as-spec, no guessing,
  independence from original runtime assets.
