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
   and no open-source RTS has it. If this comes out exactly right, the method
   is proven and the rest is repetition. **Done**: `docs/ATTRITION.md` and
   `docs/SUPPLY.md`.
2. **Run the original** — 32-bit x86 Windows on Apple Silicon. **Done**, via
   CrossOver's D3DMetal; `docs/ORACLE.md` has the exact path, the loggers the
   executable ships, and every run captured so far. What it unlocks is bigger
   than recorded games: the engine's own logger dumps chosen subsystems' state
   **every frame** to a text file, and a fixed seed makes a run reproducible.
   That file, not a recording, is the per-frame ground truth the sim is diffed
   against.
3. **Sim skeleton** — the rules of a match, mechanic by mechanic, diffed
   against the original's own per-frame dumps. **The score is ticks before
   divergence** on the longest traced capture, pinned as a floor in
   `rondata::diff` and stated first in `docs/QUEUE.md`. **Done when** a
   fixed-seed, traced human-versus-AI capture on two maps stays in lockstep
   — no position or order disagreement — for its full length. This is the
   long middle, and the number is how anyone can tell where in it we are.
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

**Which file holds what.** The column that matters is the last one: a
subagent inherits this file and the memory index, and nothing else
(`docs/DECISIONS.md` entry 21).

| file | holds | how it grows | inherited by a subagent |
|---|---|---|---|
| `CLAUDE.md` | the rules | rarely changes | **yes** |
| `docs/QUEUE.md` | where things stand, and the backlog | rewritten every session; **deletes** finished items; guarded | no |
| `docs/JOURNAL.md` | the chronicle, dated, and every finished item's story | append-only | no |
| `docs/<MECHANIC>.md` | the specification: rules, fields, formulas, coverage | amended in place; its section numbers are an API the code cites; the story goes to the journal; size guarded | no |
| `docs/DECISIONS.md` | decisions and their rationale | append; amend in place, never delete | no |
| `docs/audit/` | the second readings' verdicts, and the method's record | one file per audit | no |
| memory (`~/.claude/projects/…/memory/`) | machine and account facts only | per user, outside git | **the index is** — keep its hooks free of findings |

Read `docs/QUEUE.md` first — its opening section is the handoff and its last
line is the next session's opener — then the document of the item you take.
A finding never goes in this file: it goes in the mechanic's document, the
journal, or the queue.

**One mechanic per session.** The document is the handoff: a fresh session
reads `docs/<MECHANIC>.md` and knows what the last one knew. That is what makes
`/clear` between mechanics free, and it is why the document is written before
the implementation rather than after.

**An item is booked with the score it moves.** The headline, or a sub-score
the queue names — and the item nearest the headline's first divergence is
the default. An item that turns out to be something else spawns its
successors at the *back* of the queue, not the front, and a session that
moved no score says so in the handoff. This is the stopping rule for the
long middle: the residue chase is productive and unbounded, and only the
number says whether the whole is converging.

**Definition of done**, all five:

- `docs/<MECHANIC>.md`, stating how it was established, how confident it is,
  what it has *not* established, and which of its claims a diff backs.
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
  rather than strikes and stays short, this file names no finding, and a
  document over the size ceiling may only shrink.

A guard that has never failed has not been tested; make it fail on purpose
once, then land it. A rule that could be a guard and is only prose will be
broken within the week; the queue's own rules were.

**Prefer a diff to a reading, and convert readings into diffs.** Reading is
how we find out what to check; a differential check against the original's
own dump is how we *know*, and it keeps knowing on every later commit. Two
rules follow:

- **When the original dumps a record, diff the whole record** — every slot,
  every field — not the field the mechanic happens to care about. Nine tenths
  of a dumped record once went uncompared for a month, and the first widening
  failed on its first run.
- **A finding that can become an assertion must become one before its audit
  is closed.** A second reading's budget is best spent on *what to assert*,
  not on more prose; the twenty-minute widening has out-produced the
  million-token reading.
- **Diff first, then read what no run reaches.** Before a blind reading,
  widen every dumped record the mechanic touches; then brief the readers
  with the mechanic's *never-executed* functions, not the whole mechanic,
  and ask each claim to name the capture that would falsify it — the ini
  category, the frame, the field. A claim a run has already confirmed does
  not need a second reader. The soak tests the simulation against itself —
  determinism and termination, never fidelity — and stays for that.
- **Grep the dump before booking a reading.** An open question whose
  answer is a field the original already prints costs a `grep` and has
  more than once cost a reading instead.
- **Where a reading's product is a formula, the implementation is a pass
  of the audit — so build before ratifying, or alongside.** Prose can cite
  every address correctly and still have the arithmetic wrong, and an
  adjudicator cannot tell without doing the work: a row read as
  "additions, as cited" is exactly the row nobody re-derived. Writing the
  code, and running the diff, is what finds that. So a mechanic whose
  reading yields arithmetic gets its implementation *before* the
  ratification pass, or in parallel with the reading; a mechanic whose
  reading yields predicates and call graphs does not have to wait.

The last one is a **default, not a gate** — the point is that tools and
implementations earn their place ahead of reads wherever they can, not
that a fixed order is owed.

The reason reading does not go away is coverage, and it is measurable: the
trace's report (`tools/trace/report.py … blind docs/`) lists the functions
`docs/` cites that have never executed in any traced game. A diff can only
check what a run reaches, so for those the reading is the only evidence there
is. **Every behavioural run shrinks that list, and shrinking it is what would
eventually make the second reading unnecessary** — so the list is the queue
of runs, and a run that lights up a whole family is worth more than one that
lights up a function.

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
go on the ledger for the next **batched** ratification (see the fan-out rules
below); the next mechanic does not wait on one. The full decompile export
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

**Fan-out rules.** **Opus drives.** It is the default for a subagent and for
the session, and it carries this work end to end — implementation, diffs,
widenings, readings, adjudication under the marker discipline, the
documents and the queue. **Fable never reads blind and is never a
subagent.** It is the **steering session** — every ten items or two days,
in the main thread: is the tranche real, has the headline moved, what is
the finish line; then the batched ratification of the *marked rows only*,
any verdict that overturns an earlier one, a listing read where the
decompiler is wrong, and the rewrites of this file and the queue. It writes
the next opener. Never Sonnet; the model is said in user-visible text each
time.

An Opus adjudication is acceptable under the marker discipline — append each
verdict as it is settled, and mark what cannot be settled `FABLE:` rather
than guess. **Ratification runs in batches, over what is marked**, rather
than gating every mechanic on its own pass: markers and code-changing
verdicts accumulate, and a pass takes the accrued set. How large a batch and
how often is deliberately not fixed yet; what is fixed is that a mechanic is
not blocked waiting for one, and that nothing marked is quietly dropped.

**A ratification pass is not a subagent: it is the session.** Bank, `/clear`,
switch the main thread to Fable. And its brief is a **charter, not a
checklist** — the verdicts and markers are the floor, the mandate is what the
earlier passes missed, and a pass fenced to the floor can only ever agree
with the framing that fenced it.

Launch a fan-out in waves, not whole; readers write to durable storage from
their first finding; verify which model actually ran from the transcript,
never from the spawn parameter. Rationale in `docs/DECISIONS.md` entries 22
and 23; the operating checklist in `docs/audit/README.md`.

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
