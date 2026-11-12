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
artifact is the next phase's tool. **The finish line, and the tracks that
reach it, are `docs/TRACKS.md`**; this list is the arc.

0. **Extract** — one tool that reads the user's install and emits the XML
   tables as typed, index-keyed open data, plus the PDB type stream as
   documented layouts. Turns `docs/` into a real specification of RoN's data
   model and sim state. Weeks, not years; publishable alone; immediately useful
   to the RoN:EE modding community.
1. **Attrition** — one mechanic, end to end, headless. Borders → territory →
   damage, and supply cancelling it. It is the namesake, it is self-contained,
   and no open-source RTS has it. If this comes out exactly right, the method
   is proven and the rest is repetition. **Done**: `docs/ATTRITION.md` and
   `docs/SUPPLY.md`.
2. **Run the original** — 32-bit x86 Windows on Apple Silicon. **Done**, on
   free Wine, owned end to end; `docs/ORACLE.md` has the exact path and the
   loggers the executable ships; `docs/RUNS.md` every run captured so far. What it unlocks is bigger
   than recorded games: the engine's own logger dumps chosen subsystems' state
   **every frame** to a text file, and a fixed seed makes a run reproducible.
   That file, not a recording, is the per-frame ground truth the sim is diffed
   against.
3. **Sim skeleton** — the rules of a match, mechanic by mechanic, diffed
   against the original's own per-frame dumps. **Done**: every captured game
   replays in lockstep for its full length, and those lengths are floors in
   `rondata::diff` that run on every gate and fall only on a named
   regression. `docs/DECISIONS.md` entry 29 names the three counters that
   define the sim complete; how the long middle was run, and the rules a
   residue or chapter item still follows, are `docs/METHOD.md`.
4. **Renderer** — thin client. Original assets first; they are the visual
   oracle. **Open**, with the start, checksum, orders, lobby and save tracks
   beside it.
5. **AI** — the computer's decisions are ported as far as the captured games
   reach, because lockstep needs them. What remains is the branches no
   capture entered, and a whole game against a person in our own client.
   Build order and economic posture are scripted in the open under
   `game/ai/scripts/`; combat and target selection are in the executable.
6. **The fork** — swap the assets, then build past the original.

**Cut from v1**, to be revisited only once the above stands up: Conquer the
World, the scenario editor, the trigger system, GameSpy and the multiplayer
meta, and the ~90 `iface*` windows. That is roughly half of the 796 files in
the engine's `game/` module. CtW is genuinely good and worth building; it is
not worth building first.

## Working agreement

The long middle was done one mechanic at a time, and what is left is done
the same way. This is what has been working, written down so a fresh session
can pick up without re-deriving it.

**Which file holds what.** A subagent inherits this file and the memory
index, and nothing else (`docs/DECISIONS.md` entry 21).

| file | holds | inherited by a subagent |
|---|---|---|
| `CLAUDE.md` | the rules; rarely changes, held under a size ceiling | **yes** |
| `docs/QUEUE.md` | where things stand and the backlog; rewritten every session, **deletes** finished items | no |
| `docs/TRACKS.md` | the finish line and the track each item serves; the pass maintains it | no |
| `docs/METHOD.md` | the method of the long middle; a brief names it for the kinds that need it | no |
| `docs/LOOP.md` | the loop's machinery, read by the driver | no |
| `docs/journal/<date>-item-<N>.md` | a landing's story, written once by the worker whose item it is | no |
| `docs/JOURNAL.md` | the chronicle to 2026-09-17 and the passes' entries since; the main thread only | no |
| `docs/<MECHANIC>.md` | the specification, amended in place; its section numbers are an API the code cites | no |
| `docs/DECISIONS.md` | decisions and their rationale; append, amend in place, never delete | no |
| `docs/audit/` | the second readings' verdicts and the passes' records | no |
| `docs/RUNS.md` | the oracle's ledger, one section per capture; `docs/ORACLE.md` keeps the runbook | no |
| memory (`~/.claude/projects/…/memory/`) | machine and account facts only, outside git | **the index is** — keep its hooks free of findings |

Read `docs/QUEUE.md` first — its opening section is the handoff and its last
line is the next session's opener — then `docs/TRACKS.md` for the track your
item serves, then the document of the item you take. A finding never goes in
this file: it goes in the mechanic's document, the journal, or the queue.

**One mechanic per session.** The document is the handoff: a fresh session
reads `docs/<MECHANIC>.md` and knows what the last one knew. That is what makes
`/clear` between mechanics free, and it is why the document is written before
the implementation rather than after.

**An item is booked with the score it moves**, and a finding that names no
score stays in its item's journal rather than queues. A session that moved no
score says so in the handoff. **A number that moved lands with the value
diff beside it** — the record, the field and both sides' values, in the
oracle's own coordinates — because two sides can agree on a wrong answer for
a long time, and a row count is not a value diff. **A mechanism an item's
title names is a hypothesis, never a promise to build it**: a hypothesis the
evidence kills parks with its kill. The booking rules a residue or chapter
item follows are `docs/METHOD.md`.

**Definition of done**, all five:

- `docs/<MECHANIC>.md`, stating how it was established, how confident it is,
  what it has *not* established, and which of its claims a diff backs.
- The implementation, in its own module.
- Tests, including the end-to-end kind that run the new mechanic against the
  ones already there.
- `cargo test --release`, `cargo clippy --all-targets` and `cargo fmt`
  clean, and `cargo run -p rondata -- <install>` exiting zero — which is
  `python3 tools/release_gate.py <install> --test-threads 4` in one
  command, under the memory cap, with the fixture audit and the paperwork
  guards; that is the gate a landing names. **A worker's is the same line
  with `--lane --tests <its filters>`**: it runs those, the fast tests and
  the guards, and leaves the timed ones to the booking gate. It forgives a
  red only on the queue's lines, which are the driver's to write at the
  merge, and on the journal's own gate quote, which cannot exist before the
  gate has run; red on anything else it stops and names the test. The debug
  profile is for the reflex (`tools/guard.sh`) and for stepping through one
  test; a debug run that reaches a kept dump refuses, by design.
- Committed. Any open question this closes in another document is struck
  through there and pointed at its answer, per the amend-in-place rule below.

**Keep going while the path is clear; ask when it isn't.** That is the whole
rule, and it is what the sessions so far have actually done. Uncertainty inside
a mechanic is usually not a reason to stop — implement under a stated
assumption, record it under "What is not established", carry on. Uncertainty
about *direction*, a divergence worth making deliberately, anything
irreversible or outward-facing, or a finding that changes what the project
should do next: those are worth a conversation, and the conversation is cheap.

**A permission-shaped failure ends the turn with a question.** A capture
that refuses, an Apple Event that times out, a tool that hangs at 0 % CPU:
these are what a revoked macOS permission or a consent dialog look like,
and a Claude Code update resets the permissions. They are not diagnosed
into a fact about the hardware and worked around — the user cannot unblock
what they are not told about, and an hour of good side work is not worth
the hour of the work that was asked for. A refusal whose remedy is in
this repo and needs no human — a lane to relaunch through, a grant that
belongs to another bundle — is taken and reported; the rule is for what
needs a human. **And `no verdict (error)` from the auto-mode classifier
is an outage, not a refusal**: back off on a Monitor, five minutes and
up to an hour, before the turn ends with a question — a lane that
stopped on five such answers in three minutes sat 163 minutes for a
person (parked 1315).

In practice the natural boundary is the end of a mechanic. Finish it, commit
it, say where things stand and what you would do next — then it is a good
moment to clear the context and start the next one fresh, because the document
carries everything forward.

**The checks with teeth are the ones that can fail.** These guards stand
outside the per-mechanic tests, and each was written by first making it fail:

- `crates/sim/src/no_float.rs` reads the simulation's own source and rejects
  any float outside `#[cfg(test)]`. It allows the software float (an integer
  mantissa) and the test oracles, and nothing else.
- `crates/sim/src/soak.rs` generates games from a seed — a scenario, and a
  stream of orders including unreasonable ones — plays each **twice**, and
  compares a per-frame digest. It found a non-terminating loop within an hour
  of existing. The old determinism test ran one hand-built scenario; this
  runs a hundred it did not think of.
- `rondata::diff` pins the harness against the original's own logged runs,
  frame for frame, so the state of the port is a test rather than a number in
  a commit message. The dumps live outside the repo; a machine without them
  says so rather than passing quietly.
- `crates/sim/src/docs_guard.rs` reads the paperwork: the queue deletes
  rather than strikes and stays short, this file names no finding and stays
  under its size ceiling, every
  function address a specification cites is checked against the decompile
  export's index, and a document *section* over the size ceiling may only
  shrink — the unit is the section because that is what a session reads.
  Its sibling in `rondata::diff` parses the queue's scoreboard line
  against the pinned floors, so the handoff cannot drift from the score.

A guard that has never failed has not been tested; make it fail on purpose
once, then land it. A rule that could be a guard and is only prose will be
broken within the week; the queue's own rules were.

**Prefer a diff to a reading, and convert readings into diffs.** Reading is
how we find out what to check; a differential check against the original's
own dump is how we *know*, and it keeps knowing on every later commit.

- **When the original dumps a record, diff the whole record** — every slot,
  every field, every unit on the frame — not the field the mechanic happens
  to care about. **A quiet field is checked against the reader before it is
  called agreeing**: a key nothing parses cannot part, and quiet is what a
  blind instrument looks like.
- **A finding that can become an assertion must become one** before its
  audit is closed.
- **Look at what is on disk first.** Grep the dump before booking a reading,
  grep this crate for a field before reading the original's writers of it,
  and grep the archive before booking a capture.
- **Where a reading's product is a formula, the implementation is a pass of
  the audit** — build before ratifying, or alongside. Prose can cite every
  address correctly and still have the arithmetic wrong.
- **A comparison must be able to fail.** A check whose two sides are built
  from one helper agrees with itself; perturb one side once and watch it
  turn red.

The clauses these were distilled from, each with the case that taught it,
are `docs/METHOD.md`. A residue, chapter, coverage, reading or sweep item
reads that file before its first probe.

**Every reading-only claim gets a blind second reading before it is called
done.** A claim a diff against the original's dump has confirmed needs no
reader — the dump is the stronger oracle, and it keeps checking on every
commit — so each document's coverage section says which of its claims are
diff-backed and which rest on a reading alone, and the readers are briefed
with the second list. For those: one
reader writes the document from the decompile; a second, who has not seen the
document or the implementation, re-derives the same mechanic from the same
export and writes a report; a third adjudicates every disagreement back to the
decompiled function and records the verdicts under `docs/audit/`. When the
adjudication ran on Opus, its `FABLE:` markers and its code-changing verdicts
go on the ledger for the next **batched** ratification (`docs/LOOP.md`); the
next mechanic does not wait on one. The full decompile export
under `tools/ghidra/` is what makes the second reading cost an hour rather
than a session. Its record and its lessons are in
`docs/audit/README.md`; the ones that have recurred most: the arithmetic is
doubly confirmed almost everywhere and the *predicates* are where the errors
are — which kinds are exempt, which step a multiplier belongs to, which array
a level indexes — exactly what tests written from the same reading cannot
catch; read the loaders, not only the consumers; **grep the writers of every
field you call frozen, and the callers of every function you call
once-only**; a name is settled by the type record, never by the surrounding
code; and when the decompiler prints a local that cannot be right, the
listing (`llvm-objdump`) or the PE bytes settle it in a minute. A blind
reader inherits this file — so **this file must never name what a reader is
meant to re-derive**; findings go in the mechanic's document and the queue.

**The loop's machinery is `docs/LOOP.md`** — the driver, the roster, the
brief, the chain, the passes — read by the driver, never needed by a worker.
What a worker keeps of it: the model is said in user-visible text, and a
commit's trailer names the model the worker's own system prompt names; **a
landing is verified by refs and announced by the worker** — its tip SHA, a
non-empty `git log <base>..<branch>`, one line to the driver — its story is
its own `docs/journal/<date>-item-<N>.md` and it touches neither
`docs/JOURNAL.md` nor `docs/QUEUE.md`; **it commits before it gates**, reads
the gate's exit before saying pushed, takes `ccc update <ref>` at the gate's
exit and `--keep-conflicts` on a conflict, sends a one-line status at ninety
minutes unlanded, and never hands its word over for where the code sits
(`docs/DECISIONS.md` entries 34, 55). **A number it reports is measured on
the tree after its last update.** The pinned constants and the widenings are
the worker's to re-pin; the queue's lines are the driver's.

**Emit traces under the original's own names.** `docs/ORACLE.md` lists the 37
`SyncDefine` categories the engine considers sync-critical. Where a mechanic
maps onto one — `LeadersSync`, `UnitsSync`, `BuildsSync`, `WorldSync`,
`GoodsSync`, `DeathsSync`, `TerrainSync` — use that name. It costs nothing now
and makes the eventual diff mechanical rather than a translation exercise.

## Conventions

- **Earn every dependency.** Crates appear in this workspace when they have
  real code, not in anticipation. Same for third-party deps.
- **A probe shape reached for a third time graduates into `tools/`.** The
  recurring dump probes — a slicer, a draw tracker, an entity scanner —
  are authored once and invoked thereafter; one-off hypothesis probes
  stay scratch scripts, because a wrong abstraction costs more than the
  typing it saves.
- **Every test run joins one host budget** (`tools/resource_budget.py`):
  `tools/guard.sh` and every gate mode reserve from it themselves, and a
  standalone `cargo test …` joins as `zsh tools/check.sh test …`; nothing
  wraps `guard.sh` or a gate again. `touch
  ~/Library/Caches/attrition/admission-off` is the kill switch for every
  worktree at once. The design and its limits are
  `docs/TEST_PERFORMANCE.md`.
- **`tools/guard.sh [filter …]` is the reflex between edits** — the
  paperwork guards plus the current item's tests, debug, seconds. The
  full `--release` diff suite remains the gate: before a commit and
  after a document rewrite.
- **A wait on an external process is backgrounded** (`run_in_background`,
  Monitor), never a foreground sleep-and-grep loop — every poll turn
  re-bills the whole context — **and then the turn ends.** The harness
  re-invokes the session when the task exits, so a turn held open to wait
  gains nothing and costs a full context read each time. The ban is on any
  command whose purpose is to yield the turn, never on a list of spellings.
  **A detached capture is waited on with `tools/gamelog/waitrun.sh
  <viadriver log>`** in that background lane, with the Bash tool's own
  `timeout: 3600000` — the tool's argument, never a `timeout` command, which
  macOS has not.
- **A number in a rule names its source, and a price names the day of its
  rate table.**
- **A multi-line Rust patch from Bash rides the python-heredoc pattern**
  (`python3 - <<'PYEOF'` with `old="""…"""`/`new="""…"""`), chained with
  its test run in the same call — edit and verify in one turn, and safer
  than `sed` across lines. **Never chain a `git checkout <file>` onto an
  edit that can fail**: the checkout runs when the edit does not, and it
  reverts the work already there.
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
- **Ghidra, with the PDB loaded, gives named, typed decompilation, and
  `tools/ghidra/` holds everything**: `analyze.sh` builds the project once
  (hours), `export.sh` decompiles all 48k functions plus every struct and
  vtable to files (minutes), and after that reading is `grep` over
  `decomp/`. Its README lists the traps that have each cost a wrong
  conclusion once. The two things the decompiler does not do for you — name
  the field behind a `field_0xNN`, name the method behind a vtable call —
  are `types.txt` and `vtables.txt` in the export.
- **The original runs on this machine, on nothing we do not control**: free
  WineHQ in the prefix `~/wine-ron`, with DXVK-macOS (`docs/ORACLE.md`, "Off
  CrossOver"). `tools/gamelog/winelaunch.sh` is the single launch line every
  capture script sources, and `cliclick` drives it. The launch line holds a
  **lane lock**: one game at a time, a second launch refuses and names the
  holder, and a stale lock is never removed by hand (`ron_lane_state` says
  which of free, stale and held the lane is).
- **A function of the executable can be called outside the game**
  (`tools/emu/callfn.py`, under unicorn; a sweep is a `#[test]` in
  `crates/sim`), **and on the original's own state**: a packet
  (`tools/explore/frame_snapshot.py`) copies the process's data at one frame
  boundary, and `tools/recomp/step4.py` runs a function on it. Reach for
  them when the question is a value the original holds or computes, before
  a reading and before a detail capture. Lifted C is a reading tool exactly
  as the decompiler is: never a source, never in git. `docs/EMULATOR.md` has
  the costs.
- Constants are not all loaded in the representation the file writes. At least
  one rational arrives scaled to 8.8 fixed point. Read the consumer before
  believing the digits.
- `cargo run -p rondata -- <install>` surveys the data layer and re-derives
  every structural claim in `docs/FORMATS.md` from the user's own files. If a
  claim stops being true it exits non-zero. Run it after touching anything
  that reads the game's data.
- **The lab** (`docs/lab/`, `tools/explore/`, `tools/viewer/`) is a second
  harness's exploration, merged whole (`docs/DECISIONS.md` entry 38); the
  guards do not scan it. Two of its products are opt-in tools: a read-only
  diff viewer (`docs/DEBUG_VIEWER.md`) and a click-free capture lane
  (`tools/explore/unattended_capture.py`) that needs no TCC grant and no
  human at the menu.
- **A behavioural check is a logged run, and it is cheap** (`docs/ORACLE.md`,
  last section): fix the seed in `rise.ini`, enable the mechanic's
  categories under `[End Frame]` in `gamelog.ini`, play a minute, quit
  through the in-game menu, read `Logs\gamelog.txt`. Do not enable
  everything per frame; it slows the simulation to a crawl.

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
