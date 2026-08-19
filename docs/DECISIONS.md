# Decisions

Newest last. Each entry records what was chosen, what it was chosen over, and
why — so a future session can tell a considered decision from an accident.

## 1. Fidelity before divergence

**Chosen:** reimplement toward the original, reaching parity before changing
anything.

**Over:** starting from RoN's design and building our own game immediately.

The original executable is the only specification that exists, and a recorded
game is the only oracle that can prove we match it. Divergence is a one-way
door: the moment the sim stops matching a real recorded game, the oracle is
gone and every later bug becomes unfalsifiable — indistinguishable from an
intentional design change.

This is not the conservative choice. It is the choice that keeps the creative
one available, because at parity the art swap becomes a content decision rather
than an engineering one.

## 2. Simulation and renderer are separate

**Chosen:** a headless, engine-independent sim crate, with the renderer as a
thin client over observed sim state.

**Over:** building inside a game engine and letting sim state live in engine
components.

This split is what makes the replay differ possible at all, keeps determinism
testable without a display attached, and makes the eventual asset replacement
free. It is the single load-bearing structural decision; everything else is
downstream.

## 3. Rust

**Chosen:** Rust.

**Over:** Swift, Unity, C#/.NET, Godot.

- **Swift** — no RTS ecosystem, ARC in a hot simulation loop, single-platform
  gravity, and no good story for cross-platform arithmetic determinism.
- **Unity** — its value is the editor and asset pipeline for content-heavy
  games. Our problem is a deterministic simulation, which Unity actively fights
  (float physics, GC pauses, update-order nondeterminism). We would end up
  writing the sim outside it and using it as a dumb renderer, at which point
  something lighter is better.
- **C#/.NET** — genuinely credible; it is OpenRA's choice, so it would come
  with a readable reference implementation for lockstep order serialization and
  the data-driven mod layer. Rejected on determinism discipline: GC and float
  behaviour both need constant vigilance rather than being structurally
  prevented.

Rust gives the best determinism story, and strong types plus a fast test loop
are what make sustained agent-assisted work productive over months rather than
days.

## 4. Fixed-point, not floating-point

**Chosen:** Q16.16 fixed point (`fixed::Fx`) for all gameplay arithmetic.

**Over:** `f64` with strict IEEE settings.

Float results vary across compilers, architectures, and optimisation levels. A
lockstep sim that varies is not a sim, and — more immediately — a sim that
varies cannot be diffed against a recorded game. Q16.16 gives ±32768 at
1/65536 resolution, which comfortably covers a tile-based RTS map.

Two sub-decisions, both pinned by tests so they cannot drift:

- **Rounding truncates toward zero** for multiplication and division alike.
  Rust guarantees integer `/` truncates toward zero on every target. A cheaper
  arithmetic shift would floor instead; mixing the two rules was rejected in
  favour of one rule that is easy to reason about.
- **Overflow saturates.** Wrapping would teleport a unit across the map
  invisibly. Panicking behaves differently between debug and release, which
  would make sim behaviour depend on the build profile — itself a determinism
  hazard. Saturation is identical in both profiles, and a pinned value shows up
  as a visible bug rather than a silent desync.

RoN's own data suits this: unit costs are stored as integers multiplied by ten,
so `Fx::ratio` brings them in exactly, with no float ever existing.

## 5. Phase 1 is unconditional

**Chosen:** extract the game's content layer into open formats before deciding
anything else.

Whatever the eventual end state — faithful remake, HD reskin, or an original
game that merely shares RoN's ideas — extracting `rules.xml`, `unitrules.xml`,
`buildingrules.xml`, the tech data, and the BH3/BHA models is the same work,
and it is weeks rather than years.

Game mechanics are not copyrightable; only the assets are. So even the
"abandon the RE and build our own thing" branch wants those tuned numbers as
its starting point rather than rediscovering them over years.

Buy the cheap option now; defer the expensive decision.
