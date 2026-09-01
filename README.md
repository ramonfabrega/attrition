# attrition

A deterministic, ground-up reimplementation of **Rise of Nations** (Big Huge
Games, 2003) in Rust — simulation first, art last.

Named for the mechanic no open-source RTS has ever implemented: units bleeding
health inside hostile national borders.

> **Status: the sim skeleton, thirteen mechanics in.** Attrition and supply,
> movement and unit collision, the economy, costs and production, the tech
> tree, combat, cities and buildings, orders, the pathfinder, the animation
> clock, the per-frame random stream, and the AI — its scripted opening through an interpreter of
> our own, and its C++ producers behind it. All of it headless, integer-only,
> and diffed frame for frame against the original's own per-frame log — and
> **on both scored maps a full human-versus-AI capture now runs in lockstep
> for its whole length**, 1,772 frames on one and 1,851 on the other: every
> unit position, every order list, every animal, and the game's RNG placed
> draw for draw by call site, with no disagreement anywhere in either. The
> measurement moves on to the same two games at 24,000 frames, where the
> first divergence is now at frame 2,176. No renderer yet.
> `docs/QUEUE.md` says exactly where things stand.

## What this is

There is no open-source Rise of Nations engine. Not on osgameclones, not in
awesome-game-remakes, not on Wikipedia's engine-recreation list. No public
decompile, no format wiki. The field is empty.

This aims to fill it, following the arc that OpenTTD and Beyond All Reason both
completed: reimplement the engine against the original's own data files, then
progressively replace the original assets until the game stands on its own.

Three things make Rise of Nations an unusually good target:

- **Its design is already readable.** `rules.xml`, `unitrules.xml`, and
  `buildingrules.xml` ship as plain text — nation powers, attrition rates, pop
  caps, per-unit attack/HP/cost/speed. Twenty years of tuned balance, sitting
  in files you can open in a text editor.
- **Its art is not the point.** Nobody is nostalgic for 2003 low-poly RTS
  units. The appeal is the systems — territory, attrition, eight ages in forty
  minutes — and those survive a total art replacement intact.
- **It ships its own debug symbols.** The Extended Edition depot includes a
  full, unstripped private PDB that GUID-matches the shipped executable:
  complete struct layouts, the source tree, function names and line numbers.
  The original is a readable specification rather than a black box, which is
  the difference between years of archaeology and months of translation.

And a fourth, found on the way: **the original will tell you what it did.**
The shipped executable carries a per-frame state logger, switched on by an
ini file, and a fixed seed makes a run reproducible. That log — every unit's
position and order list, every leader's census, the loaded constants by name —
is the ground truth this simulation is diffed against, and it turns every
claim in `docs/` from a reading into a check.

## How it is built

One mechanic at a time, and each one the same way:

1. **Read** the original — the decompiled executable with its own symbols —
   and write `docs/<MECHANIC>.md`: the rules, how they were established, how
   confident the reading is, and what it has *not* established.
2. **Implement** from the document, in integers at the original's own scales.
   Decompiled code is read, never transcribed; the sim contains no floating
   point, enforced by a lint that reads its own source.
3. **Second-read it blind**: an independent reader re-derives the mechanic
   from the same export without seeing the document, and every disagreement
   is adjudicated back to the decompiled function (`docs/audit/`).
4. **Diff it** against the original's own log, and pin the match as a test.

Where a run of the original can reach a mechanic, the diff is the evidence;
where none can yet, the reading is — and the trace tool reports which
functions the documents cite that no run has ever executed, which is the
queue of runs.

## Requirements

You need your own copy of **Rise of Nations: Extended Edition** (Steam app
`287450`). No game assets are distributed here and none ever will be — not
models, not textures, not the XML, and not symbol dumps or decompiler output.
The tools read from your installed copy and generate what they need on demand.

## Getting the game files on macOS

Extended Edition has no Mac build, but its files can be pulled down natively
with no Wine, VM, or Windows machine involved:

```sh
scripts/fetch-depot.sh
```

This uses SteamCMD with a forced Windows platform type to download the depot
without running it. Extraction needs the files, not a running game.

**Running** it on a Mac — to log a controlled game and have a behavioural
oracle — works under CrossOver's D3DMetal; `docs/ORACLE.md` has the exact
path, the loggers, and the recipe for a logged run.

## Layout

```
crates/sim/         the simulation: headless, deterministic, integer-only
crates/rondata/     reads an install's tables into the sim's types, reads the
                    original's logs and recordings, and diffs the two
crates/fixed/       fixed-point arithmetic — used by rondata's constant
                    classification, deliberately not by the sim (decision 16)
tools/ghidra/       builds and exports the decompile once; reading is grep after
tools/gamelog/      one-line readers for a logged run
tools/trace/        in-process draw-site trace and function coverage of the original
scripts/            fetch-depot.sh
```

`docs/`:

| | |
|---|---|
| `QUEUE.md` | where things stand, and what is next — read this first |
| `JOURNAL.md` | the chronicle, session by session |
| `DECISIONS.md` | architectural decisions and their rationale, amended in place |
| `FORMATS.md` | file formats, every claim evidence-backed |
| `ORACLE.md` | running the original, its loggers, and every logged run |
| `DATALAYER.md`, `SYNC.md` | the install into the sim; the per-frame random stream |
| `RECGAME.md`, `COMMANDS.md`, `INPUT.md` | recorded games: the container, the command payloads, the order stream |
| `ATTRITION.md`, `SUPPLY.md`, `MOVEMENT.md`, `ECONOMY.md`, `COSTS.md`, `PRODUCTION.md`, `TECH.md`, `COMBAT.md`, `CITIES.md`, `ORDERS.md`, `PATHFINDER.md`, `COLLISION.md`, `ANIM.md`, `AI.md` | one document per mechanic |
| `audit/` | the blind second readings and their verdicts |

`CLAUDE.md` is the working agreement: thesis, hard constraints, phases, and
the rules the sessions run by.

## Development

```sh
cargo test --workspace     # ~740 tests
cargo clippy --all-targets
cargo fmt
```

The data-layer tests need an install: set `RON_INSTALL=/path/to/Rise of
Nations` (and `RON_GAMELOG_DIR` for the logged runs, which live outside the
repo). Without one those tests say so and skip rather than pass quietly.

To check that everything the documents believe about the format still holds
against your own files:

```sh
cargo run -p rondata -- /path/to/Rise\ of\ Nations
```

It re-derives each structural claim in `docs/FORMATS.md`, re-reads every
tuned constant the simulation depends on, and exits non-zero if any of it
stops being true. Nothing is copied anywhere; it only reads. With
`--gamelog <dump> --diff` it replays a logged run of the original and reports
the first frame that differs; `--types <dump>` checks the loaded type tables
and the combat table against the program's own; `--recgame <file.rcx>`
decodes a recording.

The toolchain is pinned in `rust-toolchain.toml`.

## Prior art

OpenRA for lockstep order serialisation and a data-driven mod layer; OpenTTD
and OpenGFX for the whole inside-out arc; Beyond All Reason for a lineage that
freed itself of proprietary assets; ptasev/Rise-of-Nations for the only
public work on the model and archive formats.

## License

Licensed under either of the MIT license (`LICENSE-MIT`) or the Apache License,
Version 2.0 (`LICENSE-APACHE`), at your option. Rise of Nations is a trademark
of Microsoft; this project is unaffiliated and contains none of its assets.
