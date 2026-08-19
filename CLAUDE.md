# attrition

A deterministic, ground-up reimplementation of **Rise of Nations** (Big Huge
Games, 2003) in Rust — simulation first, art last.

Named for the mechanic no open-source RTS has ever implemented: units bleeding
health inside hostile national borders.

## Thesis

The original executable is the only specification that exists, and a recorded
game is the only oracle that can prove we match it. So we build *toward* the
original rather than away from it: **reach parity, then fork.**

Diverging early is a one-way door. The moment the sim stops matching a real
recorded game, the oracle is gone and every subsequent bug becomes
unfalsifiable. Fidelity is not the conservative path here — it is the only
path that keeps the creative one open.

Once the sim is at parity and we own the renderer, replacing 2003 art with our
own is a *content* decision, not an engineering one. That is the road OpenTTD
walked (OpenGFX replaced every original sprite by Dec 2009, making the game
standalone) and the one Beyond All Reason walked out of Total Annihilation.

## Architecture

**The sim/renderer split is the load-bearing decision.** Everything else is
downstream of it.

- **sim** — headless, deterministic, no engine dependency, fixed tick rate.
  Knows nothing about pixels, windows, or input devices. ~90% of the work.
  Runnable in a test harness against recorded games with zero graphics.
- **renderer** — a thin client over observed sim state. Swappable. This is
  where art lives, and the only place it lives.

That split is what makes the replay differ possible, keeps determinism
testable, and makes the eventual art swap free.

## Hard constraints

- **No floating point in the sim. Ever.** Not `f32`, not `f64`, not "just for
  this one distance check". All gameplay arithmetic goes through `fixed::Fx`.
  Float results vary across compilers, architectures, and optimisation levels;
  a lockstep sim that varies is not a sim.
- **The sim crate depends on no graphics, windowing, or async runtime.** If it
  cannot run in a `#[test]` with no display attached, it is wrong.
- **No original Rise of Nations assets in this repo. Ever.** Not models, not
  textures, not audio, not the shipped XML. The tools read from the user's own
  installed copy. This is both the legal line and the OpenTTD model.
- **Every format claim is evidence-backed.** No guessing at a struct layout.
  If we assert a field, `docs/FORMATS.md` cites the bytes that prove it.

## Phases

Each phase produces something independently valuable, and each phase's
artifact is the next phase's tool.

0. **Oracle** — reverse the Recorded Game format; headless replay reader and
   sim-state differ. Publishable on its own; genuinely useful to the live
   RoN:EE community. Tells us whether the multi-year middle is real *before*
   committing to it.
1. **Content** — extract the game's data layer into open formats: BIG archives
   → `rules.xml` / `unitrules.xml` / `buildingrules.xml` / techs; BH3/BHA
   models → glTF. Yields 20 years of Big Huge Games' tuned balance numbers as
   open data. **Correct under every possible end state** — do it regardless.
2. **Sim skeleton** — load that data, replay a recorded game, diff against the
   oracle. Start with economy + one unit type + movement. Score = ticks before
   divergence. This is the long middle.
3. **Renderer** — thin client. Original assets first; they are the visual
   oracle.
4. **AI** — hardest, least-oracled. Defer as long as possible; ship
   vs-human first.
5. **The fork** — at parity, swap assets. The art project starts here, and by
   then it is content work, not engineering.

## Conventions

- **Earn every dependency.** Crates appear in this workspace when they have
  real code, not in anticipation. Same for third-party deps.
- Toolchain is pinned in `rust-toolchain.toml` so the Solana toolchain on this
  machine can never leak in.
- Format recon notes live in `docs/FORMATS.md`; decisions and their rationale
  in `docs/DECISIONS.md`.

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
