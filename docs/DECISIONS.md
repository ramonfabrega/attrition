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

## 6. The shipped debug symbols are the specification

**Chosen:** treat `game/sbl/rise.pdb` as the project's primary specification.

**Over:** black-box reverse engineering, with a recorded-game diff as the only
oracle.

The Steam depot ships a full, unstripped private PDB for the shipped
executable. Not a public symbol file — the real one:

```
Has Types: true   Has Globals: true   Has Publics: true   Is stripped: false
GUID: {51D4F219-61C6-4F84-9D5B-C3361B0D291F}   Age: 1
```

`riseofnations.exe`'s own `RSDS` debug directory carries that same GUID and
age, so these symbols belong to the binary we have, not to some other build.
It yields 5,880 class and struct definitions with complete field layouts,
1,251 source file paths, and function names, addresses, and line numbers.
`game/sbl/` also holds `rise_z.map`, a 13 MB linker map, and PDBs for five
further modules.

This retires the founding premise. `CLAUDE.md` used to open with "the original
executable is the only specification that exists, and a recorded game is the
only oracle that can prove we match it." Both halves were wrong the moment the
depot finished downloading. The executable is now a *readable* specification,
and the checksum and RNG machinery it describes is a better oracle than a
replay diff — see [4] below and `docs/FORMATS.md`.

The consequence that matters: "diverge early and you lose the oracle" was true
only because divergence destroyed the ability to test. With per-subsystem
ground truth available directly, it no longer is. Parity stops being a gate
and becomes a menu — which is what makes decision 8 possible.

Same rule as assets: **no PDB-derived dump is ever committed.** Tools in this
repo generate them from the user's own install, on demand.

## 7. The decompiler is a reading tool, not a source

**Chosen:** use Ghidra with the PDB loaded to read, understand, and verify;
write the understanding down as prose in `docs/`; implement from the prose.

**Over:** transcribing decompiled function bodies into Rust.

The immediate reason is not legal. It is that the thing being decompiled is
2003 C++ built on hardcoded eight-player arrays, global singletons, and a
bespoke `String`, and one of this project's actual goals is to escape exactly
that. Transcription would import the design we are trying to leave, and the
result would be worse Rust for no speed gain — the expensive part is
understanding a mechanic, not typing it.

The secondary reason is that it keeps the door open. Field layouts, symbol
names, and file/line data are interface facts; a line-by-line port of a
function body is a different kind of artifact. Reading widely costs us
nothing, and writing from a spec leaves the redistribution question open
rather than answering it early and badly.

## 8. Fidelity is chosen per subsystem

**Chosen:** full fidelity where the original's behaviour is the asset;
deliberate divergence everywhere else, from the start.

**Over:** whole-game parity first, then fork.

Decision 1 said reach parity, then fork, because divergence was a one-way
door. Decision 6 removed the door. What is left is a straightforward question
of where the original is actually worth copying:

| Layer | Posture | Why |
| --- | --- | --- |
| Sim rules, balance, pacing | **Full fidelity** | Twenty years of tuned numbers. The reason the game still holds up, and the part nobody else has. |
| Engine internals | **Diverge immediately** | Eight-player arrays, positional XML, a 2002 scripting VM. Copying this buys nothing. |
| Renderer and art | **Diverge** | This was always the point. |
| Content | **Superset** | Cheap once the tables are open data. |

Decision 1 is not reversed — it is narrowed to the row where it was always
doing the work.

**Cut from v1:** Conquer the World, the scenario editor, the trigger system,
GameSpy and the multiplayer meta, and the ~90 `iface*` windows. That is
roughly half of the 796 files in the `game/` module. CtW in particular is
worth building eventually; it is not worth building first.

## 9. Data loaders are index-keyed; tag names are labels

**Chosen:** load the XML tables positionally — Nth record into slot N — and
treat element names as human-readable annotation only.

**Over:** generating types from the shipped DTDs and keying by tag name.

This is not a preference. It is what the data requires; the evidence is in
`docs/FORMATS.md`. The engine's parser ignores tag names entirely, `rules.xml`
contains duplicate tag names inside a single parent, and the shipped
`rules.dtd` describes about a fifth of one section of the file it claims to
document. A name-keyed loader built from the DTD cannot load the shipped game
data.

The upside is large: a record's index *is* the engine's type id, which is
almost certainly how orders encode unit and building types. Getting this right
is what connects the content work to the replay work later.

## 10. Exact rationals where the original used a float

**Chosen:** carry `anti_att` as an exact `i64` numerator and denominator, and
divide once at the point the original truncates.

**Over:** `fixed::Fx`, and over truncating to an integer after each factor.

`CLAUDE.md` says all gameplay arithmetic goes through `Fx`. That rule exists to
keep floating point out of the simulation, and here the better way to satisfy
it is not to reach for `Fx` at all.

`anti_att` is the one value Rise of Nations keeps as an `f32` in the middle of
its simulation. Its only inputs are `256` and a chain of `100 / (100 - pct)`
factors with integer `pct`, and the original truncates it exactly once, at the
`* 1/256` step, after every multiplication has compounded. So:

- **Truncating per factor** would drift. This is the same lesson
  `Scalar::scaled_fx` already encodes for movement speeds, arriving from a
  different direction.
- **`Fx`** would round at 1/65536 where the original does not round at all. It
  would import error the original does not have, in exchange for nothing: the
  value is a ratio of small integers and never needs a fractional
  representation.
- **An exact rational** reproduces the original bit for bit and is checkable by
  hand. Of the factors in play only 4/3 is not dyadic, so the reachable values
  form a family of eight, all pinned by tests.

Nothing in attrition needs fixed point. `crates/sim` therefore depends on
nothing at all, not even `fixed` — the dependency will be earned when movement
arrives. See `docs/ATTRITION.md`, "Not open, and why".

## 11. The simulation takes its tuning as an input, and a tool checks it

**Chosen:** `sim::Tuning` is a plain struct the simulation is handed;
`Tuning::RON` holds the shipped values, and `rondata` re-derives all of them —
71 as of the economy — from the user's own `rules.xml` and fails if any has
drifted.

**Over:** reading the game's XML from inside the sim, and over hardcoding the
numbers with no check on them.

Two rules pull against each other here. Nothing from the user's install may
enter the repository; and a formula whose inputs nobody can see is not a
specification. Writing the numbers down resolves it — game mechanics are not
copyrightable, and `docs/ATTRITION.md` already quotes them, because a document
that says "substitute the shipped constants" and then does not is useless.

Writing them down is also how they go stale, which is what the check is for. It
is the same discipline `docs/FORMATS.md` gets from the structural checks,
applied to values instead of structure: a patch that rebalances attrition shows
up as a failed check rather than as a simulation that is quietly wrong.

The consequence is that `rondata` depends on `sim`. That is the right
direction: `rondata` is the tool that validates this repository's claims
against a real install, and the tuning table is one of those claims.


## 12. A constant the original does not read is not tuning

**Chosen:** `Tuning` carries only values that can change the simulation's
answer. `SIEGE_OUT_OF_SUPPLY_RELOAD` and `ARTILLERY_OUT_OF_SUPPLY_RELOAD` are
therefore absent, and `crates/sim` writes the same literals the original does.

**Over:** carrying every named constant in `rules.xml` that touches a mechanic
we implement, on the grounds that entry 11 says to write the numbers down.

Both constants exist in the shipped file, annotated `"3/2 normal delay"` and
`"2/1 normal delay"`. Both are parsed and stored in the engine's constants
table. Neither is read: `UnitData::recharge` multiplies by literal `3 / 2` and
literal `2`. The annotations describe the code rather than feeding it, and a
mod that edited them would change nothing.

Entry 11 exists so that a formula's inputs are visible and so that drift
against a real install is caught. A tuning entry here would defeat both. It
would be visible and *wrong* — a reader would take it for an input — and the
drift check would faithfully compare a number that cannot affect anything. The
simulation would also diverge from the original for exactly the modded data the
check is meant to protect.

So the rule is about behaviour, not about names: a value belongs in `Tuning`
when changing it changes what the simulation does. Where a constant is inert,
`docs/SUPPLY.md` records that it is inert and why, which is the part a reader
actually needs.

The same reading in the other direction added `Slot::Ratio256`.
`PARMENIO_RADIUS_ADJUST` *is* read, but not in the form the file writes it —
`3/2` on disk is 384 in memory, because the original consumes it with a shift
right by eight. Comparing the written digits would have failed a correct table.
The check reconstructs the scale instead. Reading the consumer is what
distinguishes the two cases, and it is the only thing that could have.


## 13. Pin a table the original computes with floating point

**Chosen:** write the 256 integers of the engine's sine table down as
constants, and reproduce the generator only in prose.

**Over:** recomputing the table at startup from the same expression, and over
treating it as install-derived data that may not be written down.

`trig_init` fills the table once, before any simulation runs, with
`sin(i * 1.570796327 / 255.0) * 65535.0` in doubles. Three consequences pull
the same way.

The values are a **specification, not a computation**. They are fixed for the
lifetime of the process, and every later frame reads them as integers. Nothing
downstream ever sees a float.

Recomputing them would import the one hazard `CLAUDE.md` exists to prevent.
`sin` is not correctly rounded and is not required to agree between platforms
or libm versions, so a table regenerated at startup could differ by a unit in
the last place between two clients — which is a desync, arriving through the
back door of a rule meant to prevent exactly that.

And they are not the user's data. The generator was read and understood, and
the numbers follow from it and from pi; this is the same standing as
`div_3_table`, which `docs/FORMATS.md` records as `i / 3` rather than as a
dump. Writing down a table anybody can regenerate from a documented formula is
the opposite of copying an asset.

The general rule this sets: **where the original computes a constant with
floating point before the simulation starts, we pin the result.** Where it
computes with floating point *during* the simulation — `anti_att` — we carry an
exact rational instead, which is decision 10. Both are the same principle
applied at different times: no float ever reaches a frame.

This is also why `Fx` is still unearned. Four mechanics in — attrition, supply,
the kinematic half of movement, and income — nothing has needed a fraction, and
`crates/sim` no longer depends on `fixed` at all. Income is the strongest case
so far that this is not luck: the original had the same problem and solved it
the same way, by carrying rates in sixteenths and constants in 8.8 rather than
by reaching for a fractional type. See entry 14.


## 14. A constant's scale is a fact about the loader, one constant at a time

**Chosen:** record per constant whether the engine scales it on load, and *by
what*, by reading `Constants::init` at that constant's line. `Slot::Ratio256`,
`Slot::Entries256` and `Slot::Ratio100` mark the scaled ones, and `rondata`
rescales rather than compares digits.

**Over:** inferring the scale from how the value is written, and over assuming
a whole file shares one convention.

`CLAUDE.md` already warns that "constants are not all loaded in the
representation the file writes", and until the economy the only example was
`PARMENIO_RADIUS_ADJUST`, written `3/2` and loaded as 384. It was tempting to
read that as a rule about rationals: a `/` means the engine wants a fraction,
so it scales.

It is not that. `PEASANT_RATE` is written `10 resources`, with no `/` anywhere,
and arrives as 2560 — because `Constants::init` happens to read that one line
with `get_fraction(name, 0x100)`. Three lines away in the same file
`CITY_GATHER`'s `10food` goes through `convert_int` and arrives as 10. Nothing
in the text distinguishes them. Only the loader does, and only the consumer
proves it: `PEASANT_RATE` is read back with a `>> 8` that would otherwise turn
a farmer's ten food into zero.

So the scale is neither a property of the syntax nor of the file nor of the
suffix. It is a property of the one line of `Constants::init` that reads that
constant, and it has to be established there. `PEASANT_RATE`, `OIL_RATE` and
`SCHOLAR_RATE` are the three in the economy; there is no reason to think they
are the last three in the game.

This is the same principle as `docs/FORMATS.md`'s evidence rule — no asserting
a field without citing what proves it — applied to a value's units rather than
a struct's layout. It is also cheap to enforce: the check re-derives the scaled
value from the install and fails if it drifts, so a wrong guess about a scale
shows up as a failed check rather than as an economy that is a hundred and
fifty times too fast.

**Amended by production, twice.** Neither amendment changes the rule; both
sharpen what "the scale" means.

First, **256 is not the only scale.** The production accelerators and the unit
rate pair go through `get_fraction(name, 100)`, so `6/5` arrives as 120 and
`1/1` as 100 — while `RESEARCH_TICK_PREMIUM`, three lines of the same file
away, goes through `get_fraction(name, 0x100)` and arrives as 256. And
`JOB_EXTRA_TIME` is scaled by a hundred by a hand-written parser in the
*unit-type* loader, which is not `Constants::init` and does not call
`String::fraction` at all. The entry said "one constant at a time" and meant
it; the production mechanic uses four conventions at once. See
`docs/PRODUCTION.md`.

Second, and more practically: **the check must not rescale through `Fx`.** It
used to, dividing Q16.16 by 256 to reach 8.8, which is exact for every
denominator `Ratio256` happens to see and is not exact in general.
`UNIT_RATE_BASE` is `6/5`; a fifth is not a dyadic rational; Q16.16 says 119
where the original computes 120. `Scalar::fraction(scale)` now reproduces
`String::fraction` itself — `num * scale / den` — and both slots use it. The
lesson generalises past this entry: when checking what the original computes,
compute it the way the original does, rather than through a representation
that is merely close.


## 15. Reproduce the original's arithmetic, including where it is wrong

**Chosen:** implement what the original computes, in the order it computes it,
even when the result is plainly not what the code was trying to compute. Say so
in the document and in the test name.

**Over:** quietly writing the corrected version, and over refusing to implement
something until we understand why it is like that.

The economy already had one of these and it was easy: `Leader::do_gather`'s
remainder loop is arithmetically an accumulator with no drift, but is written
as a `while` that runs at most once, and the equivalence only holds because a
guard above makes the rate non-negative. Writing the loop rather than the
accumulator costs nothing and keeps the guard load-bearing.

The cost path has a sharper one. `TypeData::can_pay_cost` answers "how many of
these can I afford" and takes the **maximum** across resources where the answer
is the minimum. It is not ambiguous and it is not a decompiler artefact; it is
a `<` that should be a `>`. `crates/sim/src/cost.rs` takes the maximum, and the
test that pins it is called `the_affordability_count_takes_the_maximum`.

Three reasons, in order of weight.

**It is the specification.** Entry 6 says the executable is what we are
translating. A behaviour is not less part of the spec for being unintended, and
"unintended" is a claim about the authors' minds that we cannot check. Twenty
years of balance was tuned against what the code *does*.

**Divergence is unfalsifiable.** Entry 1's trap in miniature: once we have
silently corrected one thing, a later disagreement with a recorded game has two
possible causes and no way to tell them apart. A deliberate divergence recorded
in a document and a test is a different object — it can be turned off.

**Most of them do not matter, and finding out which is the interesting part.**
This one is blunted by an early return: any resource you cannot afford one of
answers zero immediately, so for a single item the maximum and the minimum
agree, and a single item is what almost every caller asks about. That
observation is worth more than the fix would have been, and we would not have
made it if the first instinct had been to correct the code.

The limit is entry 8: fidelity is chosen per subsystem. When a subsystem is one
we have decided to diverge from, this entry does not apply — but the divergence
is then a decision with an entry of its own, not a quiet repair inside a
translation.


## 16. Integers at the original's scales; `Fx` is not the rule

**Chosen:** restate the arithmetic constraint as *no floating point, integers at
the scales the original keeps them*. `fixed::Fx` stays a crate in the
workspace, unused by `crates/sim`, until a mechanic needs a fraction the
original does not already store as an integer.

**Over:** the founding phrasing — "all gameplay arithmetic goes through
`fixed::Fx`" — which `CLAUDE.md` carried for seven mechanics while none of them
used it.

Entry 4 chose Q16.16 before any of the original had been read, on the
reasonable guess that an RTS simulation needs fractions. It does; and it turned
out the original supplies every one of them as an integer with a scale already
chosen: 8.8 for rates (entry 14), hundredths for accelerators, sixteenths for
income, a 65535-scaled sine table built once before the first frame (entry 13),
and an exact rational for the single `f32` it keeps mid-simulation (entry 10).
Three entries had each said "and so `Fx` is still unearned" from a different
direction. This is the entry that says it once, as the rule.

The rule matters because Q16.16 is *close* to those scales and not equal to
them. `UNIT_RATE_BASE` is `6/5`; at 8.8 it is 307, at ×100 it is 120, and
Q16.16 gives 78643 — which rounds the same way by luck on most inputs and
differently on some, and entry 14's second amendment already caught one case of
"merely close" (119 where the original computes 120). Reproducing the original
bit for bit means reproducing its representation, not approximating it with a
finer one.

Entry 4's two sub-decisions — truncate toward zero, saturate on overflow — are
unchanged, and are now statements about how `crates/sim` does integer
arithmetic rather than about a type.

If a mechanic ever needs a fraction the original computes in floating point
*during* a frame, that is a new entry, because it is the first place the
original itself is not deterministic across hardware.


## 17. Derived per-player state is recomputed where the original recomputes it

**Chosen:** when a value is *derived* — a unit's effective speed, a player's
population cap, the research-versus-train bit, a price's discount tail, the
anti-attrition factor — `crates/sim` computes it at the moment and with the
cadence the original does: on read where the original reads through a function
(`UnitData::get_speed`), on change where the original recomputes wholesale
(`Leader::calc_pop_cap`), and on a fixed cadence where the original caches on
one (`Unit::process_attrition`'s 32-frame refresh). The mechanic that *produces*
the value is its only writer; the mechanics that *consume* it keep taking it as
an input.

**Over:** a single uniform pattern — everything on read, or everything
recomputed on a dirty flag — chosen for tidiness.

Seven mechanics in, `Sim` is still a harness: parallel `Vec`s kept in step by
`add_player`, and every value one mechanic needs from another taken as a plain
input (`Movement.speed`, `Muster.age`, `Muster.researched`, `cost::Modifiers`).
That was the right shape while each mechanic was downstream of nothing we had
built. The tech tree is the first mechanic *upstream* of several, and it is
where the question of how derived state flows has to be answered rather than
deferred.

Uniformity is the tempting answer and it is wrong for the same reason entry 15
is right: push versus pull is *observable*. A cached value refreshed every 32
frames behaves differently from one computed on every read — a unit that walks
out of hostile territory keeps bleeding for up to 31 frames, and the original's
players have twenty years of intuition built on that. A pop cap recomputed on
change behaves differently from one computed on read only if the recompute is
ever skipped, which is exactly the kind of thing a faithful implementation has
to be able to reproduce when it happens.

So the rule is per value, and it is cheap to state per value because the
reading already establishes it: the document for the producing mechanic says
where the original computes each output and on what cadence, and the
implementation follows. The consuming mechanic's signature does not change —
`train_time` still takes `researched: bool` — which keeps each mechanic
testable alone, as it has been.

The structural consequence for `Sim` is small and deliberate: per-player state
grows by one struct per producing mechanic (the tech layer's flags and levels,
alongside `Muster`, `Holdings`, `Ledger`), and `add_player` grows by one line.
The harness is not yet an architecture, and this entry does not make it one;
it decides the one thing that would otherwise have been decided by accident.

## 18. The rules name types by role, and a tree without the role leaves the rule inert

**Chosen:** where a rule in the original names a specific `TypeIndex` — the
Market the Nubians get early, the Senate a capital may hold one of, the
machine-gun line that needs a rifle-line unit, the Tower-to-Redoubt line whose
prerequisites the Romans waive — `crates/sim` names it through a role
(`tech::Roles`), each of which is optional. A tree built without that role
simply never fires the rule. The tree's own indices are its own; the original's
ranges are represented by `tech::Kind`, which is what its `is_unit_type` /
`is_age_type` / … predicates test.

**Over:** reproducing the original's 806-slot `TypeIndex` space, with every
rule written against the literal index it has in the shipped tables.

The literal space is what the decompile reads, and it would make the
transcription mechanical. It would also make the simulation unable to hold a
tree that is not the shipped one — a five-type test fixture, a mod, the game we
intend to build past it — without every rule silently pointing at the wrong
row. The roles are exactly the list of types the rules single out, twenty-odd
of them, and naming them is how the document records *which* rules are about
*which* things; the reader of `docs/TECH.md` sees "the Senate" rather than
`0x1b6`. The cost is one `Option` test per rule, and a tree loader that fills
the roles from the shipped names, which is the loader's job anyway.

This is entry 9 one level up: the shipped tables are index-keyed and we keep
them so; the *rules* about particular rows are keyed by what the row is for.


## 19. A float the original evaluates mid-frame is reproduced operation by operation, in integers

**Chosen:** where the original computes a gameplay value with a floating-point
expression *during* a frame — so far exactly one: a projectile's flight time,
`(int)(sqrtf(dx² + dy²) / (float)(proj_speed × UNIT_MOVE_SPEED))` in
`Ammo::init` — `crates/sim` evaluates each IEEE operation with the rounding
it actually has (a correctly rounded 24-bit square root, a correctly rounded
24-bit quotient, a truncation), using integer arithmetic only. See
`combat::flight_time` and `docs/COMBAT.md` §9.1.

**Over:** carrying an `f32` for that one expression, or replacing it with an
integer square root and accepting a one-frame difference at the boundaries.

Entry 16 said that if a mechanic ever needed a fraction the original computes
in floating point during a frame, that would be a new entry because it is the
first place the original is not itself deterministic across hardware. This is
that place, and the answer is narrower than either alternative. Both
operations in the expression are ones IEEE 754 requires to be correctly
rounded, so on any conforming host the result is a pure function of two
integers — the original *is* deterministic here, provided its `sqrtf` is the
hardware one (x87 `fsqrt` and SSE `sqrtss` both are). Reproducing each
rounding in integers gives the same function without admitting a float into
the simulation; an integer square root would agree except when `√n / d` lies
within a float ulp of an integer, which a recorded-game diff would eventually
find. The exact emulation costs a few dozen lines and a test that compares it
against the host's `f32` over a spread of inputs.

The other thing combat added to the arithmetic rules is not a decision so much
as a recognition: **the game's random number generator is a rule.** `Random`
is a 32-bit LCG and `get(lo, hi)` a fixed mapping of its state; every
projectile's scatter, the flock roll on a building's first wound and the
one-in-five retarget roll draw from the same stream in a fixed order, so the
stream is part of what "plays like the original" means. It is `combat::Rng`,
and the document lists the draws in order (`docs/COMBAT.md` §9.5).
