# attrition

A ground-up reimplementation of **Rise of Nations** (Big Huge Games, 2003)
in Rust. It is deterministic, and the simulation comes first and the art
last.

It is named after a mechanic no open-source RTS had implemented: units lose
health while they stand inside an enemy's national borders.

## What this is

No open-source Rise of Nations engine existed before this one: no public
decompile and no format wiki. This project follows the path OpenTTD and
Beyond All Reason took. First it reimplements the engine against the
original game's own data files. Then it replaces the original art until the
game no longer needs it.

Rise of Nations is a good target for three reasons:

- **The balance ships as plain text.** `rules.xml`, `unitrules.xml` and
  `buildingrules.xml` hold twenty years of tuning: every unit's attack, hit
  points, cost and speed, the nation powers, the attrition rates.
- **The game ships its own debug symbols.** The Extended Edition includes a
  full program database that matches the shipped executable: struct
  layouts, source file names, function names and line numbers. So the
  original can be read like a specification. Nobody has to guess how it
  works.
- **The game reports what it did.** The executable carries a per-frame
  state logger that an ini file switches on, and a fixed seed makes a game
  reproducible. That log is the ground truth: this simulation is checked
  against it frame by frame, and each match is pinned as a test.

The design has two parts. The **simulation** is headless and integer-only,
runs at a fixed tick rate, and has no graphics dependency, so the whole game
runs inside a test. The **renderer** is a thin client that draws what the
simulation reports. Art lives only in the renderer, so swapping the art
later costs the rules nothing.

## The finish line, and where it stands

The goal is set out in [`docs/TRACKS.md`](docs/TRACKS.md). In short: a
standalone app, with no Wine. A lobby asks for a map, a nation and computer
opponents, and you play a whole game against the computer, drawn with the
original's art. Under the picture the rules match the original exactly: fed
the same orders, Rise of Nations does the same thing, frame for frame. Games
can be saved and replayed. Multiplayer, Conquer the World, the scenario
editor and new art are left out on purpose. Each is a later goal.

The work is split into tracks. As of November 2026:

- **Rules: whole.** About 130 recorded games of the original, across maps,
  nations, ages, difficulties and scripted battles, each replay in lockstep
  for its full length. That means every unit's position and orders and every
  draw of the game's random number generator agree with the original's log
  on every logged frame. The longest are 24,000 frames, about 27 minutes of
  play. A held-out battery of games that nobody tuned for measures how well
  this carries over to new games. Of the staged chapters, which cover
  commands a recorded game rarely issues, all but one are closed.
- **Map generation:** fifteen of the original's map types generate the same
  start, phase by phase.
- **Structure: in progress.** The simulation crate is being reorganised into
  subsystem directories with a public surface ([`docs/STRUCTURE.md`](docs/STRUCTURE.md)),
  and no move may change behaviour.
- **Renderer: started.** The camera matches the original's projection, and
  the original's model and animation files decode in a dependency-free
  `render` crate. The first decoder handles 1,439 of 1,452 pieces. Nothing
  is drawn to the screen yet.
- **Not started:** game start without a log of the original, the original's
  full-state checksum, mouse and keyboard input, our own lobby, the computer
  opponent's own logic, and save and replay.

[`docs/QUEUE.md`](docs/QUEUE.md) carries the exact numbers. Its scoreboard
lines are parsed against the pinned tests, so they cannot drift.

## Running it

**You need your own copy of Rise of Nations: Extended Edition** (Steam app
`287450`). "An install" means the directory that holds `riseofnations.exe`.
Every tool takes the install's path as an argument and reads from it.

There is no Mac build, but the files can be downloaded on macOS without
Wine, a virtual machine or Windows:

```sh
scripts/fetch-depot.sh            # SteamCMD, Windows depot, into ./game (gitignored)
```

The Rust toolchain is pinned in `rust-toolchain.toml`. To survey the install
and check every claim the documents make about its formats:

```sh
cargo run --release -p rondata -- /path/to/install
```

This re-reads the tables, checks each structural claim in
[`docs/FORMATS.md`](docs/FORMATS.md) against your files, decodes the models
and animations, and confirms that the simulation's tuning matches your
`rules.xml`. It exits non-zero if any of that stops being true. It only
reads, and it copies nothing.

The tool has two more modes. `--gamelog <dump> --diff` stands the simulation
up from a log of the original and steps it frame by frame, then reports the
first frame that differs. `--recgame <file.rcx>` decodes a recorded game.
Run it with no arguments to list the rest. Some scored games carry
scripted orders, and those replay through the test suite, not this flag.

The full check, the one every change must pass, is:

```sh
python3 tools/release_gate.py /path/to/install --test-threads 4
```

It runs the release-mode test suite, clippy, the format check, the survey
above and the documentation guards, with the tests under a memory cap. The
replays against the original need its logs. Those live outside the repo,
and `docs/ORACLE.md` explains how to make your own. Tests that need an
install or a log that is not there say so and skip; they never pass
silently. For a quicker loop, `cargo test --release -p sim` runs the
simulation's own tests (the few that read an install take it from
`RON_INSTALL`).

**Running the original itself** on a Mac, to record your own logged games,
works on free WineHQ Stable with DXVK-macOS and nothing licensed.
[`docs/ORACLE.md`](docs/ORACLE.md) has the exact setup, the loggers, and the
recipe for a logged run.

## Where the documents are

| | |
|---|---|
| [`docs/TRACKS.md`](docs/TRACKS.md) | the finish line and the tracks toward it |
| [`docs/QUEUE.md`](docs/QUEUE.md) | where things stand and what comes next |
| `docs/<MECHANIC>.md` | one specification per mechanic, for example [`ATTRITION.md`](docs/ATTRITION.md), [`SUPPLY.md`](docs/SUPPLY.md), [`COMBAT.md`](docs/COMBAT.md), [`ECONOMY.md`](docs/ECONOMY.md), [`AI.md`](docs/AI.md), [`MAPGEN.md`](docs/MAPGEN.md). Each covers the rules, how they were established, how confident the reading is, and what it has *not* established |
| [`docs/FORMATS.md`](docs/FORMATS.md) | the game's file formats, with the evidence for each claim |
| [`docs/ORACLE.md`](docs/ORACLE.md) | running the original, its loggers, and logged runs ([`RUNS.md`](docs/RUNS.md) lists every capture) |
| [`docs/RENDERER.md`](docs/RENDERER.md) | the renderer's design and its counter |
| [`docs/STRUCTURE.md`](docs/STRUCTURE.md) | the simulation crate's layout, and the moves under way |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | the decisions and why they were made, amended in place, never deleted |
| `docs/journal/`, [`docs/JOURNAL.md`](docs/JOURNAL.md) | the story: one file per landed item, and the chronicle before that |
| [`docs/audit/`](docs/audit/) | independent second readings and their verdicts |

The crates are `sim` (the simulation), `rondata` (reads an install and the
original's logs, and diffs the simulation against them), `render` (the
renderer's headless core) and `fixed` (fixed-point helpers, kept out of the
simulation on purpose). [`CLAUDE.md`](CLAUDE.md) holds the working rules the
development sessions follow.

## The legal line

**Nothing from your install ever enters this repository.** That covers
models, textures, audio and the shipped XML, and also symbol dumps,
decompiler output and logs of the original. The game directory is
gitignored, and the tools generate what they need from your own copy each
time they run. This is the OpenTTD model.

**The decompiler is used for reading, never as a source.** The original's
code is read in order to understand a mechanic. That understanding is
written down as a document, and the Rust is written from the document.
Decompiled function bodies are never copied or transcribed.

## Prior art

- OpenRA, for lockstep order serialisation and a data-driven mod layer.
- OpenTTD and OpenGFX, for the whole path from reimplementation to a
  standalone game.
- Beyond All Reason, for a game that freed itself of proprietary assets.
- ptasev/Rise-of-Nations, for the only other public work on the model and
  archive formats.

## License

Licensed under either the MIT license (`LICENSE-MIT`) or the Apache License,
Version 2.0 (`LICENSE-APACHE`), at your option. Rise of Nations is a
trademark of Microsoft. This project is not affiliated with Microsoft and
contains none of its assets.
