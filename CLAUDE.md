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
  this one distance check". All gameplay arithmetic goes through `fixed::Fx`.
  Float results vary across compilers, architectures, and optimisation levels;
  a lockstep sim that varies is not a sim. The original's own data is stored as
  rationals (`1/192 tile`, `2/3`, `6/5`), so `Fx::ratio` takes it exactly.
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
2. **Run the original** — 32-bit x86 Windows on Apple Silicon, via Wine,
   CrossOver, or a VM. Gives us a visual and behavioural oracle, and lets us
   check any claim instead of reasoning about it. Not a prerequisite for 0 or 1;
   a hard prerequisite for trusting 3.

   **What it unlocks is bigger than recorded games**, per `docs/ORACLE.md`. The
   shipped executable contains `SyncLogger`: a per-frame, per-category state
   tracer over 37 named subsystems, switched on by a `synclogger.ini` beside
   the binary, with three forced RNG seeds and a plain-text log in which every
   entry carries the source file and line that emitted it. A recorded game
   turns out to hold only the command stream, an initial-state snapshot and the
   seeds — no per-frame checksums — so the tracer, not the recording, is the
   exact oracle. One configured run is worth more than a library of replays.
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

1. **The tech tree** — `has_preq`, `type_avail`, `type_eligible`, ages, epochs,
   research. Three documents already assume its answers, and
   `docs/PRODUCTION.md` specifically needs whatever writes the availability bit
   at `leader + 0x6c18`, which is the bit deciding research-versus-train.
   `Leader::gain_tech` is the obvious entry point and is read only as far as
   its call to `Build::refund_cost`.
2. **Combat** — attack, damage, armour, target selection. Attrition is still
   the only thing in the simulation that can kill.
3. **Cities and buildings** — placement, construction, city levels, the
   `BuildData::construct_time` path `docs/PRODUCTION.md` read but did not
   implement.
4. **AI** — last, because it is the least oracled.

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

**Anything needing phase 2 is written down, not waited on.** Nothing is blocked
on running the original. A claim that needs a behavioural check goes in the
document's open questions with the check named, and the work continues. See
`docs/ORACLE.md` for what a running game will eventually be able to tell us,
and for how far Wine currently gets.

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
  Scripted work goes through `analyzeHeadless`, under
  `$(brew --prefix ghidra)/libexec/support/`. Two things the decompiler does
  not do for you and a script must: name the field behind a `field_0xNN`, and
  name the method behind an indirect call like `(*(code **)(*this + 0xcc))()`.
  The second is a vtable slot; resolving it turns a wall of offsets into
  ordinary code, and is what settled both the supply eligibility checks and
  the shared siege predicate.
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
