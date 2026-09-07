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

> **Amended 2026-08-21 by entry 16, and the callout added 2026-09-04.** This
> entry chose Q16.16 before any of the original had been read. Entry 16
> overturns the *representation*: gameplay arithmetic is integers at the
> original's own scales, and `fixed::Fx` is a crate that stays unearned
> until a mechanic needs a fraction the original does not already store as
> an integer. What survives of this entry is the ban on floats, the
> truncation rule and the saturation rule.

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

> **Amended 2026-08-20.** Record index is the type id, as below; but fields
> within a record are looked up **by tag name** from the internal string
> table (`docs/FORMATS.md`, the correction under "parsed positionally"). So:
> records by index, fields by name. The loader in `crates/rondata/src/
> load.rs` works that way; nothing else in this entry changes.

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


## 20. The AI's scripts are data, and the interpreter is ours

**Chosen:** the shipped `ai/scripts/*.bhs` are read from the user's install at
load time, like the XML tables, and executed by an interpreter of our own —
`crates/sim/src/bhs.rs`, a tree-walker written from the language's observed
semantics (`docs/AI.md` §3, §11) over a `Host` trait the simulation
implements. The engine's bytecode compiler and VM were read for what the
language *means* — that truth is `> 0`, that arguments evaluate right to left,
that a `static` initialises once ever, that a trigger disarms when it fires —
and nothing of their design was carried across.

**Over:** transcribing `economic.bhs` and `defensive.bhs` into Rust, or
skipping the script and starting the step machine at its second step.

The finding that forced the choice is `docs/AI.md` §3: every skirmish AI's
opening is one of those two scripts, run by `production_ai` at step 1, so the
oracle's first hundred seconds are the script's decisions and nothing else's.
Transcription would have copied shipped content into the repository (the
line the XML tables respect) and frozen the one part of the AI the original
left open to modders; skipping it would have reproduced nothing the oracle
shows. An interpreter costs the language (~1,500 lines, sixteen tests) and
the 55 host functions the scripts call, each a thin predicate or action over
state the simulation already keeps.

Two conditions were set when the choice was made, so that the interpreter
does not loosen the rules the rest of the simulation runs under. Script
statics, trigger bits and timers are simulation state — they live in
`bhs::State` and the host, and the soak digests them — because the original
keeps them on the `Script` object shared by all eight leaders, which is
observable (the scripts index their own eight-way arrays by hand). And the
language's `float`, which no shipped script uses, is not implemented on
`f32`: a program declaring one fails to load today, and when a mod earns it
the type is built on `combat::F32`, the integer-mantissa software float, so
`no_float.rs` keeps its jurisdiction over the whole simulation crate.

## 21. `CLAUDE.md` carries the rules; the queue and its history live in `docs/`

**Decided 2026-08-25; tier 1 executed the same day** (see "Executed" below).

`CLAUDE.md` is 787 lines and 51 KB, of which the working agreement's running
commentary — the queue, mechanic by mechanic, session by session — is lines
126–734, **78% of the file**. Every session loads it. So does **every
subagent spawn**, because a subagent inherits the project's instructions.

Two costs, and the second is the one that forced the decision.

**The standing cost.** ~13 k tokens injected into every spawn. This project
fans out constantly — the AI's second reading alone spawned seven readers,
so about 90 k tokens of narrative rode along, none of it useful to a reader
whose brief is one function.

**The correctness cost.** `docs/audit/2026-08-25-ai.md` records it: the
blind second reading of the AI **leaked**, because `CLAUDE.md`'s queue names
that mechanic's findings outright — `TechType::set_research` writing
`build_flags & 0x8000000`, `research_techs` reaching the 9,999,999 guard, the
step ladders, the make list's occupied slots, `expire_all`'s `% 3` residue.
Readers had those in context before they read the brief, so two headline
"independent confirmations" were nothing of the kind. A brief cannot fix
this: the system prompt arrives first. **Every future blind reading of a
mechanic `CLAUDE.md` narrates is degraded the same way**, and the project's
own definition of done requires one per mechanic.

### The split

- **`CLAUDE.md`** keeps what a subagent should inherit and nothing else:
  thesis, architecture, hard constraints, phases, conventions, tooling, prior
  art, and the working agreement's *rules* — one mechanic per session, the
  five-part definition of done, the three guards, "keep going while the path
  is clear", "emit traces under the original's own names", the blind-reading
  method. No findings, no per-mechanic narrative, no queue. Target: under 200
  lines.
- **`docs/QUEUE.md`** takes the queue and its history — what is done, what is
  next, what is owed. The main session reads it; subagents do not. This is
  the "continuity" file the project had early on and lost.

### The hazard, which decides the method

**The documents are an API.** `crates/` carries **453 section references**
and 73 mentions of `AI.md` alone, in module docstrings that cite claims by
number (`docs/AI.md §2.7`). Renumbering or merging sections inside a mechanic
document silently invalidates hundreds of them, and nothing tests it.

So the restructure is done in two tiers, and only the first is safe:

- **Tier 1 — pure moves, no deletion.** Split `CLAUDE.md`; lift the queue
  into `docs/QUEUE.md` verbatim. Nothing is rewritten, so nothing can be lost
  and the diff is reviewable. This alone fixes the leak and the per-spawn
  cost. Do this first and commit it on its own.
- **Tier 2 — pruning inside the mechanic documents, selectively.** Each one
  mixes three things: the **specification** (the derived rules — the
  treasure), the **provenance** (how it was established, confidence, what is
  *not* established — required by the working agreement, and how a reader
  tells a derived formula from a guess), and the **chronicle** (which session
  landed what, which run showed it). Only the third is git history restated,
  and only the third should go.

  Do **not** sweep all 22 documents. Apply the rule when a mechanic is next
  touched, plus one pass over the two or three worst offenders — `ORDERS.md`
  (178 KB), `AI.md` (127 KB), `ORACLE.md` (114 KB). **Preserve section
  numbers**, or fix every citation in the same commit.

The amend-in-place rule (`Conventions`) is unchanged and stays: a claim that
was corrected keeps its correction inline, with the successor named. That is
provenance, not chronicle.

### Executed, 2026-08-25

Tier 1 landed as two commits. The first is the pure move: lines 132–680
lifted into `docs/QUEUE.md` verbatim, a six-line pointer in their place,
every other line of `CLAUDE.md` byte-identical (847 → 304 lines). The second
trimmed the remaining rules of the findings they cited as examples — the
attrition constants in the phases, the loggers' ini keys, `calc_wall_stats`
and `even_circle_init`, the AI audit's figures — to the rule and a pointer
at `docs/audit/README.md`, which already carried every example. One rule was
added in their place: this file must never name what a blind reader is meant
to re-derive.

Two refinements to the split as decided:

- **The queue and the continuity file are one file.** `docs/QUEUE.md` opens
  with "Where things stand" — the handoff a session rewrites on its way out
  (in progress, owed, next, needs the user) — and the backlog follows in
  dependency order. They were considered as two files and rejected: the
  handoff *is* "where in the queue are we", and two files that state the
  same fact drift.
- **`docs/ORACLE.md` is the next chronicle** (114 KB, nineteen runs). The
  tier-2 candidate there is a split into the recipe and a run ledger, on the
  same preserve-the-citations terms as the mechanic documents. Not done.

Later the same day, after a review by the `lore` session against its corpus
of other repositories' continuity files, two more moves: the chronicle —
the 545 lines of per-item narrative the verbatim lift had carried into the
queue — went to a dated, append-only **`docs/JOURNAL.md`**, and the queue's
struck entries were cut to one line and a pointer each, which is the
condition under which a single queue file stays a queue. And a probe
settled a question this entry had left as an assumption: **a subagent
inherits the memory index as well as `CLAUDE.md`**, so the index's hooks
are held to the same rule as this file. The file map in `CLAUDE.md`'s
working agreement records all of it, with an inherited-by-subagents column.

## 22. Opus by default; Fable by choice; an Opus adjudication is ratified on Fable

**Agreed 2026-08-20 (cities), trialled 2026-08-21 (orders), ratified
2026-08-23, written down here 2026-08-25** — until then it lived only in a
session memory file, which no other machine or contributor could see.

The question was which model runs which kind of subagent, given that Fable
is the only model metered tightly enough to need care, and that a wrong
specification is the expensive failure in this project — a decompiler local
that cannot be right, a folded vtable slot, a predicate read with its sense
inverted, all of which a less careful reading produces confidently.

**The arrangement.** Opus is the default for every subagent whose output is
a list or a survey — dumps, greps, extractions, patches, implementation
workers over a shared surface. Fable is chosen explicitly for the delicate
core: a first decompile reading, an adjudication. Blind second readers may
run on Opus. An Opus adjudication is acceptable under a discipline — append
each verdict as it is settled; mark what cannot be settled `FABLE:` rather
than produce a verdict not believed — and is closed by a Fable ratification
pass over the markers and the code-changing verdicts before the next
mechanic builds on them. Never Sonnet. Which model actually ran is verified
from the transcript, not from the spawn parameter.

**The evidence.** The orders audit (`docs/audit/2026-08-21-orders.md`,
`docs/audit/README.md`): seven Opus blind readers, 355 claims, all seven
finished. The Fable adjudicator fan-out hit the account's limit and six of
seven died before a verdict — a fan-out is a quota commitment, not a token
cost. The Opus re-run in waves completed all seven — A 32 · B 89 · both 165
· neither 18 · open 8, ten corrections to `crates/sim`, five `FABLE:` rows.
The Fable third pass re-verified all ten corrections and every marker:
**zero wrong verdicts, five honest gaps, four of them settled in under an
hour.** The AI audit (`docs/audit/2026-08-25-ai.md`) repeated the
arrangement — Opus readers in two waves, adjudication in the main thread, a
Fable third pass — and retracted nothing.

**What it says.** The quality of an adjudication is a property of the
discipline — append per row, mark rather than guess, ratify — more than of
the model; and the quota wall is a property of fan-out size, which waving
fixes. What stays in memory is nothing of this: account limits, bottle
names and paths are machine facts and stay there.

**Amended 2026-08-26 — the ratification pass runs in the main thread, and
its brief is a charter rather than a checklist.**

Two corrections to the arrangement above, both from applying the group
orders' audit.

*Where it runs.* A Fable ratification is **not** a subagent. It is the
session: bank the work, `/clear`, switch the main thread to Fable, and let
the pass run there. A subagent inherits `CLAUDE.md` and the memory index
and nothing else, which is the right envelope for a *blind* reader and the
wrong one for a ratifier, who needs the document, the implementation, the
audit and the captures at once. It also means the pass is paced by a
context window rather than by a spawn budget, so it can follow a thread it
did not expect to follow.

*What it is asked.* A brief that says "verify these nine verdicts" caps the
stronger model at the framing of the weaker one that wrote the list. The
best available outcome is then agreement, and agreement teaches nothing.
So the code-changing verdicts and the `FABLE:` markers are the **floor** —
they must be re-read from their own citations — and the **mandate** is to
find what the passes before it missed. The evidence that this is the real
yield: every one of the four audits so far produced its sharpest finding
*outside* the brief it was given — a caller nobody grepped for, a dock that
registers in the sea region, a writer in another class entirely — and in
each case the finding arrived by following a contradiction in a capture
rather than by working down a list. A ratifier that reports "the brief was
the limit, and here is what lies past it" has done the job; one that
reports nine confirmations has been fenced in.

**Amended 2026-08-26 — Opus drives; Fable is for the first read, the
overarching, and a *batched* ratification.**

Two further corrections, agreed after the group orders' slot table found
three of its own audit's accepted verdicts wrong (entry 23).

*Who drives.* The arrangement above reads as though Fable is the careful
half and Opus the bulk half. That is no longer how the work has gone: Opus
carries implementation, diffs, widenings, adjudication and the ordinary
reading end to end, and has produced the sharpest corrections of the last
three sessions — including three overturned verdicts that a Fable
ratification had confirmed. So the split is stated the other way round.
**Opus drives.** Fable is chosen for a **first** decompile reading of an
unread mechanic, for overarching or genuinely new design, and for the
ratification. Never Sonnet, and the choice is still said in user-visible
text and verified from the transcript.

*When ratification runs.* "Before the next mechanic builds on them" made
every mechanic wait on a pass of its own, and paid a whole context to
confirm a handful of rows. Markers and code-changing verdicts now
**accumulate on a ledger and are ratified in batches**, over what is
marked. A mechanic is not blocked waiting for one; nothing marked is
quietly dropped. **How large a batch, and how often, is deliberately left
open** — the cadence has not been trialled enough to fix, and writing a
number down now would be prescribing rather than recording. What the
ledger already holds is the older Fable debt from the AI, transport and
army audits.

## 23. Where a reading's product is a formula, the implementation is a pass of the audit

**Agreed 2026-08-26**, from `docs/QUEUE.md` item 17 — the group orders'
slot table.

The audit method assumes a reading can be checked by another reading. For
*predicates* that holds, and the record shows it: which kinds are exempt,
which array a level indexes, which step a multiplier belongs to. For
**arithmetic** it does not, and the slot table is the case that proves it.

Three of that audit's verdicts were wrong, and all three sat under a
single adjudicated row that read "additions — the document has a four-line
sketch and nothing else, **as cited in A**". Every citation in it was
real. Nobody re-derived what the citations computed, because re-deriving a
formula *is* implementing it, and an adjudicator working down a list of
rows has no reason to stop and do that. A Fable ratification then spent
its budget confirming a loop the compiler had already contradicted — the
listing showed a local written only on one arm, and the reading had
rewritten it as written on both.

**So: a mechanic whose reading yields arithmetic gets its implementation
before the ratification pass, or in parallel with the reading.** The
implementation is not the audit's output, it is one of its passes — the
one that cannot accept a citation in place of a result. A mechanic whose
reading yields predicates and call graphs does not have to wait; the
existing order is fine there.

**This is a default, not a gate.** The general shape it belongs to is the
one already written down as "prefer a diff to a reading": tools,
captures and implementations earn their place *ahead* of reads wherever
they can, because each of them can fail and prose cannot. The corollary is
smaller and recurs more often — **grep the dump before booking a
reading**: the same session found an open question booked as a reading
whose answer the original had been printing beside the record all along.

**The evidence.** Eleven deliberate breakages of the implemented table,
ten red on the first try; the three overturned verdicts
(`docs/audit/2026-08-25-groups.md`, "Fourth pass"); two further findings
no reading could have reached, one of which is a genuine
non-reproducibility in the original — a wedge's row count is seeded from
uninitialised stack. None of that came from reading the functions again.
It came from writing them out and running the diff.

## 24. The score is the finish line, the queue deletes, and Fable steers

**Decided 2026-08-27**, in the first Fable steering session, after two days
and twenty Opus sessions of the long middle. Amends entries 21, 22 and 23
in place; overturns nothing. *Amended 2026-08-30 by entry 25: the headline
is the pair of maps, the size pin is per section, the ratification ledger
is the blind list, and the steer is every twenty items.*

**What was found.** The tranche was real — every finding spot-checked held
against the decompile and the PDB, the tree was green with the install
wired in, nothing had leaked — and the loop had changed shape without a
decision: from one mechanic per session to one diff residue per session,
each closed item spawning one or two more at the front of the queue. Every
sub-score had improved. The score phase 3 names, ticks before divergence,
was **3** on the longest capture, had been 2–5 throughout, was asserted
nowhere and stated nowhere. The queue was 409 lines against its own "about
twenty", its struck entries had grown paragraphs, three documents were over
150 KB, and fifteen audits were owed a ratification that never came because
nothing scheduled it.

**The decisions.**

1. **The headline is pinned and stated first.** `ticks_before_divergence`,
   the order score and each player's first-divergence frame are asserted
   as a *floor* on the longest capture, with a dated history line; the
   handoff opens with them and says whether they moved. Phase 3's finish
   line is written into `CLAUDE.md`: a traced human-versus-AI capture on
   two maps, in lockstep for its full length.
2. **An item is booked with the score it moves**, and the default item is
   the one nearest the headline's first divergence. Spawned items go to the
   back. This is the stopping rule the long middle lacked.
3. **The queue deletes.** A finished item leaves for the journal under its
   number; no struck lines. `crates/sim/src/docs_guard.rs` fails the build
   on a strike, on 180 lines, on a 32-line handoff — made to fail first on
   the 409-line file, as the working agreement asks of every guard.
4. **Specification and story are different documents.** `docs/<M>.md` keeps
   rules, fields, formulas and a coverage section; narrative goes to the
   journal. The eight documents over 60 KB are pinned at their size and may
   only shrink — one per touch, not a rewrite campaign.
5. **Blind readings are for reading-only claims.** A diff-backed claim has a
   stronger, standing oracle. The coverage section is the reader's brief.
6. **Fable steers; it never reads.** Readers and adjudicators are Opus. A
   steering session runs every ten items or two days in the main thread,
   ratifies marked rows only, and writes the opener. The reasons are cost —
   a reading scales with the decompile, not the model — and evidence: the
   11/8 survived a Fable reading and a Fable adjudication and died on a
   diff's first frame.

**Why not sooner.** The long middle was deliberately left to run — that was
the point of writing it down as one — and it ran well. What it could not do
alone was notice that the number had not moved, because the number was not
written down. Now it is.

## 25. The headline is the pair, the pin is the section, the ledger is the blind list

**Decided 2026-08-30**, in the third Fable steering session, after twelve
Opus items in two days. Amends entry 24 in place; overturns nothing.

**What was found.** The loop entry 24 set up is doing better than it was
designed to. Twelve items, sixteen single-threaded Opus sessions, no
subagent spawned since 08-25, every item booked with its number: run10's
ticks before divergence 200 → 572 of 1,772, run33's draw-word 284 → 780
of 1,850, the roster 468 + 0 → 268 + 0, and the second map stood up at
167 of 1,850 on its first try. The two largest moves (items 74 and 83)
were made by **widening** — comparing dumped fields nobody had compared —
with no decompiler reading at all. Four paperwork rules had drifted from
what the work needs, and each is a one-line amendment:

1. **The headline is the pair, the lower map first.** Entry 24 pinned "the
   longest traced capture", and both maps' captures are 1,850 frames; the
   finish line names two maps, and one was at 32–42 % while the other was
   at 9 %. The default item is the nearest divergence on the lower map. A
   residue that shows on one map only is exactly what a single-map chase
   cannot find, and the Great Lakes chase would otherwise have run on
   until it stalled. *Refined by entry 26: the pair is the tick
   pair, and a word chase is bounded by its map's tick divergence.*
2. **The size pin's unit is the `## ` section, not the file.** A file
   ceiling taxed whoever added a finding to *any* section of a large file,
   and under a 254-byte margin what got cut was whatever the session
   needed least — once, nearly evidence (`docs/JOURNAL.md`, 2026-08-30,
   "what the byte pins are actually doing"). Nobody reads a file; a
   session reads a section. `crates/sim/src/docs_guard.rs` now holds
   every section under 16 KB, pins the eleven over it at their size, and
   drops the file ceiling. A section that needs room splits at a heading.
3. **The ratification ledger is the blind list.** The fifteen audits
   "owed" a Fable pass carried **no** `FABLE:` markers — the nine of 08-20
   predate the marker discipline — so "marked rows only" over them was a
   batch of nothing, owed forever. The surface a pass ratifies is
   claim-level and mechanical: a function `docs/` cites that no traced
   run has entered (`report.py … blind`, 101 of 617 today) and that
   `crates/sim` implements. A run that enters it retires it; a marker on
   it is what a steer takes. The file-level list is struck in
   `docs/audit/README.md` and pointed here.
4. **The steer runs every twenty items, or sooner when the headline has
   not moved for two sessions.** Three steers in four days each found the
   loop sound and moved only paperwork; the cadence was costing a session
   per ten items for a check whose answer had not changed.

**Why not more.** Opus driving, one item per session, `Continue` as the
whole prompt, no subagents: the telemetry says this is the most productive
configuration the project has had, and nothing in it is changed. Sonnet for
widenings would save little — the judgment is in what a dumped field means,
not in the comparison. Blind readings have stopped on their own, because
the diff, the trace and the listing settle a claim faster; the machinery
stays for a mechanic no run reaches.

## 26. The word is the instrument; the finish line is the tick pair

**Decided 2026-08-31**, in the fourth Fable steering session, after
sixteen Opus item commits in a day. Refines entry 25's first amendment;
overturns nothing.

**What was found.** The tranche is real and verified — every commit
trailer Opus, floors equal to the handoff, the tree green with the
install, item 109's branch re-read against the decompile — and every
booking after item 102 was on the wrong number. Phase 3's score is
**ticks before divergence** (entry 24, CLAUDE.md phase 3), and the pair
stands at East Indies 167/167 (run39) and Great Lakes 572/776 (run10);
neither moved in the whole tranche. The *words* — the first frame whose
draw count parts the original's, run33's 780 and run39's 1373 — are
instruments built beside the score, because a state diff on divergent
draw streams chases noise ("item 69 was a consequence", 2026-08-30). The
queue promoted the instrument to headline: "the lower map" was read off
the words (780 < 1373) when the score reads 167 < 572, so the handoff
called Great Lakes stuck while East Indies' own headline sat still.

**The bound that was missing.** A word behind its map's tick divergence
is the honest thing to chase. The frame the word *passes* it, the
inference reverses: every draw before the divergence agrees, so what
parts there is draw-free and no later word motion can explain it — the
tick item is due. run39's word passed 168 between items 95 and 97; item
69 was then ripe for eleven items while the chase ran to 1373. Three
rules:

1. **The headline the handoff states first is the tick pair**, run39 and
   run10, lower first, with each map's word stated beside it as the
   instrument.
2. **A word chase is bounded by its map's tick divergence.** The frame
   the word passes it, the tick item is the default booking.
3. An item booked off the default **says so in the handoff** — breached
   silently three times this tranche, caught by the handoff's own writer
   two items late. Rule 2 removes the judgment call that made the
   silence easy.

**Why not more.** The chase was not wasted: the goody and bird mechanics
are cross-map, Great Lakes' landings went from meeting one of the
original's thirteen to two, run10's collision and angle rows rose, and
the gather record reached 26,094 fields — retiring item 103's earlier
versions unbooked. The loop — Opus driving, one item a session, no
subagents — stays exactly as entry 25 left it. The batch: the orders
audit's R4 marker (`find_gather_tcoords@0063bdc0`) is unparked — its
condition, a harness-built camp, is item 85's own finding — and travels
with that item; R2 O1 stays parked, nothing depending on it.

## 27. The second lane, if it runs, is the capture lane — an independent session, never a subagent

**Decided 2026-08-31**, in the fourth steering session's coda, with the
user. Settles the shape of entry 25's "one measured trial"; books
nothing — the trial starts when the user opens the second terminal.

**Why this lane.** Entry 25 allowed a parallel session only if fenced to
widenings and captures. Item 90 *is* the fence: a capture's products are
logs outside the repo — self-announcing to the main loop through
`rondata::diff`, with no git coordination at all — and its only repo
writes are `tools/` and ORACLE.md run sections, files the main loop
rarely touches. The merge tax entry 25 feared rounds to zero.

**Topology.** An independent session in its own worktree off
`worktree-replan-pdb`, committing to its own branch, on Opus; the main
loop or the user merges when convenient. Not a subagent and not a team
hung off a main conversation, for three reasons with scars behind them:
`/clear` or a session's end kills its in-flight agents, and a capture is
fourteen minutes times four; the quota wall kills every agent in flight
at once; and a permission-shaped failure — the consent dialogs macOS
aims at screen drivers — must end a visible turn with a question, which
a buried subagent cannot do. Messages between live sessions are allowed
and never required; the filesystem is the interface.

**The single-writer rule.** The screen, the `ron` bottle and the
install's INIs are one global resource. While the capture lane exists,
all game-driving belongs to it: a main-loop item that needs a
behavioural check appends the run to the scenario file rather than
taking the screen. One capture at a time.

**The trial's measure.** The fifth steer asks two questions: did the
owed captures (23, 96, 108, 57) land and retire their claims, and was
the merge cost actually zero. Keep or kill on the answer; entry 25's
"one trial and no more" stands.

**Measured, 2026-08-31, at intake (Fable).** Both answers are yes. Runs
42–50 landed: 108, 96 and 57's capture half retired outright; 23's XOR
half fired and held, its Echelon half re-booked with the blocker named.
The merge measured zero — no conflicts, no file overlap with the loop
since 651f9fe — and the lane left the instrument cheaper than it found
it: the capture queue is a file (`tools/gamelog/captures.txt`), and a
stanza is the whole cost of a run. **Kept**, still fenced to widenings
and captures per entry 25.

## 28. A check asserts its own inputs, and a harness source belongs to every run

**Decided 2026-08-31**, on item 69. Refines entry 24's "the score is the
finish line"; overturns nothing.

**What was found.** East Indies' score read 167/167 for two days and the
whole of the previous tranche was reasoned against it. It was not a
mechanic and not a bad number — it was a *different game*. `rondata::diff`
stands the simulation up from a dump plus whatever a run "borrows": the
sibling dumps' checksum trace, heights, herds and farms, and — since
2026-08-26 — a pasture's five owner-9 animals, which appear in no dump at
all and come only from the draw-site trace (`docs/SYNC.md` §3.6).

`borrow_pasture` was called by exactly one test: the one that measures the
*word*. `run_traced`, which every tick-and-order score is measured on, did
not call it. On a map whose AI builds a pasture, five idle rolls a frame
were missing from the scoring run, so its stream parted almost at once
while the word check reported draw-for-draw agreement to frame 1373. Two
numbers in one file, describing two different simulations, for a week.

**The rule.** Two halves, and the second is the one with teeth.

1. **A source is a source for every run.** Anything a capture borrows to
   stand the simulation up belongs to `run_traced`, not to whichever check
   first needed it. A borrow reachable from one test only is a fork of the
   harness wearing the same name.
2. **A check asserts its own inputs.** Not "the mechanic works" — that was
   already asserted, five days earlier, and it was true — but "*this run
   has one*". The second map's score now fails outright if the pasture is
   not in the run it scores, rather than quietly scoring low. Any figure a
   run's setup can silently lose gets the same treatment: assert the input,
   in the test whose number depends on it.

**What it cost, and what it bought.** Three hours, and the pair went
167/167 → 1374/1373 and 572/776 → 781/776 in one session — after sixteen
items had moved neither. The two mechanics found on the way (the swarm
move's angle, `do_move`'s `goto STEP`) both came from the *other* half of
the same item: widening `OrderMismatch` to the whole `MOVEORDER` row, a
record the parser had carried and nothing had compared. That is CLAUDE.md's
"diff the whole record" paying for the fourth time, and it is why the
widening ledger (queue item 87) is worth building rather than repeating.

## 29. The finish line past the scored captures — what "sim done" measures

**Decided 2026-09-01**, in conversation (Fable 5, with the user). Extends
entries 24 and 26; overturns nothing.

**The question.** Great Lakes already runs its whole scored capture in
lockstep and East Indies' pair is closing on its own full length. The
finish line phase 3 names — two maps, full capture length — was pinned
when ~1,850 frames was what a capture was. That is two minutes of
opening: no war between the leaders, one age, and the namesake mechanic
never fires (the attrition census is inert until a capture holds a
Temple or a Fort — queue item 117). Crossing it is the milestone phase 3
promised, and it is not the sim complete.

**The decision.** The project's stated goal is the sim complete before
any phase-4 renderer work, and "complete" is three counters, each
already built, each held by the suite rather than by prose:

1. **The pair, on the long captures.** run53 and run54 are 24,001-frame
   same-game extensions of the scored runs, verified identical over the
   whole overlap (`docs/ORACLE.md`). When the current line closes, the
   floors, the scoreboard and the headline re-pin to them, with
   full-detail dumps sized to the word per ORACLE's own rule — take the
   expensive capture when the word has moved into it.
2. **The blind list toward zero.** The cited functions no traced run has
   ever entered (101 of 617): a diff only checks what a run reaches, so
   this is the counter for what any capture of the same game never
   touches. Shrinking it takes *targeted* captures — a Temple, a Fort, a
   war, a transport — not longer ones. Item 88 pins it as a floor;
   whatever residue remains at the end is enumerated and accepted
   deliberately, function by function, not left implicit.
3. **The widening ledger to zero.** Item 87: every field the parsers
   carry is compared, or stands on a listed, reasoned exemption.

When all three stand, the sim is as done as the evidence can make it,
and phase 4 opens. One carve-out, already in the queue as item 39: a
read-only debug viewer over `Sim` state is a *tool*, not phase-4
renderer work, and may come whenever it starts paying for a mechanic.

**The milestone is still said out loud.** Crossing the scored-capture
line gets its sentence in the README when it happens — the first full
game-slice in lockstep is the proof of method, and it should read as
one — and the re-pin lands in the same session, so the headline never
sits at a ceiling with nothing to say.

**Ratified and completed 2026-09-01, Fable steering, the same day.** The
line closed (item 125), the sentence is in the README, and the headline
is `LONG_WORD_EAST_INDIES = 2176` with run54's own assert behind it. The
open half — which full-detail captures to size to the new word — is
decided: **East Indies takes one, 3,000 frames**, because its word
crossed its scored capture's 1,851 and item 85 changes exactly what the
new records would check; **Great Lakes takes none yet**, its word (1802)
still inside run33's 1,850 full-detail frames. The general rule stands as
ORACLE wrote it: a map earns its next expensive capture when its word
crosses the newest one it has, and the capture is sized to the word with
headroom, not to the trace's 24,000.

**Amended 2026-09-03, Fable steering.** Great Lakes' word crossed run33's
1,850 on 2026-09-02 (run61, 1802 → 2419) and the rule above then owed it
a capture that two days of sessions did not take, because the queue had
quietly let the higher map lead. Queue item 193 books both the capture
and the diagnosis. The rule stands; what was missing was the check that
the *lower* map is the one being chased (entry 25), and the handoff now
says which map leads in so many words.

## 30. A mechanic that is read, built and *worse than nothing* is landed unwired

The bird's flight (`Unit::do_air_physics`, `docs/SYNC.md` §3.9) is the
first thing this project has read whole, implemented faithfully, and then
**not called**. The rule it settles is worth writing down, because the case
will recur.

`crates/sim/src/air.rs` reproduces the function arm for arm against the
listing, and `crates/sim/src/single.rs` reproduces the single-precision
arithmetic its bank angle is written in, exactly, in integers. Wired into
the unit loop it flies a bird the way the original flies one — the same
orbit, the same overshoot, the same edge coin — and it moves East Indies'
word from **5437 to 5404**, because the bird it flies reaches that map's
north edge thirty-three frames before the original's does and spends a draw
where the original spends none.

**A floor does not fall for a mechanic that is only nearer than the one it
replaces.** "The bird stands still" is plainly wrong and "the bird flies,
with the wrong phase" is plainly less wrong, and it does not matter: the
score is the number of frames the stream is the original's, and a mechanic
that shortens it has made the port worse *at the only thing being measured*.
The temptation to take the better model and lower the floor is exactly the
trap `CLAUDE.md`'s "the score is how anyone can tell where in it we are"
exists to close.

So the module lands, with its own tests, and the call site does not — and
the call site carries the comment saying why, so nobody re-derives it. What
is banked is the expensive half: the reading, the arithmetic, and a
falsifiable statement of the residue. What is not banked is a number nobody
can defend.

**The corollary is the interesting one.** An implementation that cannot be
wired is a *reading whose oracle is missing*, and that names the next move
precisely: the bird has no observable but one draw because nothing dumps
owner 9, so the work is to make one. `tools/trace/`'s `CALLS` proxy already
logs a chosen function's arguments; `Unit::set_new_location` carries the new
position, and a window of it is a per-frame record of where a bird actually
is. Reach for the instrument before reaching for another reading — entry 23's
rule, one level up: when a diff has no field to compare, build the field.

**Outcome (2026-09-02, run61): the corollary paid, and the decision holds.**
The field was built the next session — three proxies, a five-minute capture,
47,533 air frames — and against it `air.rs` reproduced ten birds *exactly*
for the whole capture. The residue was not in the flight at all but in
`Unit::init@00612100`'s tile snap, twenty-four position units at birth
(`docs/SYNC.md` §3.9, "The birth"). With that fixed the module is wired and
both long words rose: East Indies 5437 → 5466, Great Lakes 1802 → 2419. So
the unwired landing cost nothing and bought the reading; what it did not do
is let a defensible number be replaced by an indefensible one while the
oracle was missing.

## 31. Where a mechanic's answer is a number, proxy it — and the terrain grid keeps millionths

**2026-09-02.** Two decisions from item 57, which stood a month and closed
in an afternoon.

**The instrument.** `docs/DECISIONS.md` entry 26 says the word is the
instrument. The word is a *count*, and a count is not a sequence: six road
searches matched the original's node count exactly while two stood 3 and 410
out, and no reading could say **which node**, because `calc_road_cost`
computes a number and hands it back. run55 built the proxy for exactly this
and run61 found the general shape — bracket the dispatcher, log the mutator.
run62 completes it: **where the mechanic's answer is a number, proxy the
function that computes it *and* the predicate that chose its argument.** The
gate is what carries the coordinate a pooled-argument function cannot. The
rule now stands beside "diff the whole record" (`CLAUDE.md`): before booking
a third reading of a function whose arithmetic is already doubly confirmed,
ask what the *record* is missing, and whether a `CALLS` row would supply it.
Cost of a row: twenty minutes of listing work — the prologue bytes, the
argument count, the `ret <imm>` — and a capture.

**The scale.** `TerrainOut::terraform_for_building` is `f32` in the
original: a mean over a box of corner heights, then `(h + mean) × 0.5` on
the box's border. `crate::terrain` does it in **millionths**, the scale the
dump prints, and `World` carries the corner grid at that scale. This is
entry 16's clause read as it was written — "an exact rational where it keeps
the one `f32` it has, and a pinned table where it builds one with doubles
before the first frame" — with the difference that the table now has a
writer, so it is kept rather than pinned. The residue is stated and
measured: the exact-millionths mean and the `f32` mean truncate differently
on **three corners of 58,081** (`docs/QUEUE.md` item 58), and none of the
three is near any search yet measured. Should a divergence ever land on one,
the sim already carries a software float (`combat::F32`, an integer
mantissa) and the upgrade is local to one module — which is the reason not
to pay for it now.

## 32. The oracle runs on free Wine, and a runner is a dependency like any other

*2026-09-04, item 225.*

CrossOver's bottle licence expired and took the capture lane with it. The
choice was renew, or find a free stack — and it is a project question rather
than a purchasing one, because the lane is how every remaining mechanic gets
settled and a project whose thesis is outliving its source material should
not rent its only oracle.

**The decision: free WineHQ Stable 11.0 with DXVK-macOS.** It draws the main
menu and runs the traced executable, and nothing in it renews.

Two things made the choice cheap, and both are method rather than luck:

- **The requirement was read, not guessed.** `d3dgl.dll` — which despite its
  name imports d3d11 — makes one call, `D3D11CreateDevice(NULL, HARDWARE,
  NULL, 0, {0xa000}, 1, 7, …)`, and boxes on a negative HRESULT. A single
  feature level, 10_0. Once that was on the page, three candidate stacks
  could be *measured* against it in an hour instead of argued about. The
  exe-as-specification thesis applies to the runner too.
- **The plan of record was wrong and the disk said so in four minutes.**
  The queue named Apple's Game Porting Toolkit as "probably the answer";
  CrossOver's own bundle ships D3DMetal for `x86_64-windows` only, and the
  game is PE32. **Check the artifact before buying the plan**, even when
  the plan is yours.

The cost is recorded rather than hidden: the traced executable needs
`cover=0` under free Wine (item 226), so **function coverage is off** until
that is fixed, and the coverage list is the queue of blind readings
(`CLAUDE.md`, "Prefer a diff to a reading"). Captures and diffs are
unaffected.

This does not overturn the horizon in `docs/ORACLE.md`: Rosetta 2 ends with
macOS 28, and the durable answer is still an x86 machine running the
original natively.


## 33. Two lanes for the two starved counters, and only the main loop writes the queue

**Decided 2026-09-04**, in the sixth Fable steering session, with the user,
after a handover from the `lore` session. Extends entries 25, 27 and 29;
overturns nothing.

**What was found.** The loop is sound: eleven items and about a thousand
frames on the lower map in one day, and the lane rebuilt off CrossOver the
same day. What the loop's own rule starves are the two other counters entry
29 names as "sim done". The blind list is frozen at 101 of 617 because
function coverage page-faults under free Wine (item 226), and nothing can
shrink it until that is looked at. And the rule "every reading-only claim
gets a blind second reading" has been broken at scale: nine mechanic
documents have never had one, 261 commits landed since the last audit, and
four times this week a document stated a rule the code did not implement,
found only when the word reached it, 70 to 312 frames later.

**The decisions.**

1. **The capture lane is a session again**, `att-capture`, on Opus, in its
   own worktree on branch `lane-capture`, per entry 27's fence. A routine
   stanza needs no judgment and stays a background shell in whichever
   session wrote it; the lane exists for the runs that need judgment and
   the screen for an hour: 226 first, then the second-map squad birth that
   would kill 227, then the targeted captures entry 29 asks for. Its first
   step on 226 is an experiment, not a reading: one `cover=1` run under
   Wine's exception debug channel, read for whether arming completed, any
   breakpoint was ever handled, and any was declined. A Fable session reads
   Wine's dispatch against the handler only once that log exists.
2. **An audit lane**, `att-audit`, on Opus, branch `lane-audit`, as entry
   25's second measured trial. Its first wave is not the blind reading but
   the cheaper check the week's evidence asks for: one reader per document
   listing the rules the code does not implement, adjudicated by the lane,
   each confirmed row becoming a failing test where a capture reaches it or
   a dated coverage line where none does. Then the nine documents with no
   second reading, per the audit README; then item 72. It also writes the
   **no-writer guard**: a field the diff compares must have a writer in the
   sim or a listed exemption, in the test gate, because three such fields
   passed vacuously this week. Its measure is verdicts that became
   assertions or code, and two documents in a row with none ends it.
3. **Only the main loop writes the queue.** A lane's products are audit
   files, coverage appendices, tests, run sections and messages; the main
   loop merges both lane branches at its session start and files what they
   report. A lane never edits a rules section, a sim module's logic, or
   `CLAUDE.md`.
4. **The diff harness is split before the lanes collide in it.** One file
   of 23,243 lines, touched in 121 of the last 197 commits, is where every
   widening lands and where every lane would conflict. The split is
   mechanical, per record, one commit, test count identical, done by the
   main loop between items (queue item 228). This is the one
   "simplification" the project takes now: review-for-bugs and semantic
   simplification stay off the table until phase 4, because the oracle diff
   is a stronger reviewer than any model and a rewrite of code no capture
   reaches has no net at all.

**Rejected.** A second *game* lane: one screen, one writer, and the audit
lane shares nothing with the screen. Fan-out readings on items no capture
reaches (220, 211, 146): unfalsifiable until the capture exists. Fable as a
blind reader, budget or no budget: entry 24's evidence stands.

**The measure.** The seventh steer asks whether 226 was settled either way,
whether the audit lane's verdicts changed code, and whether the merges cost
anything. Keep or kill on the answer, per lane.

## 34. The commander loop: one worker per item, serial until the gate is cheap

**Decided 2026-09-06**, with the user, after the first full day of the
lane-and-worker shape and the crash in the middle of it. Extends entry 33;
overturns nothing.

**The bottleneck was never the model.** Before this shape the loop was a
person typing `/clear` and `continue` between items — so the throughput of
a project with a 24,000-frame oracle and a hundred-item queue was set by
how often its author was at a keyboard. What the swarm actually bought was
not parallelism. It was **removing the human from the routine step**: an
orchestrating session spawns a worker per item, merges its branch, files
its findings into the queue, runs the guards, pushes, fast-forwards
`origin/main`, and spawns the next. That ran for hours unattended on
2026-09-06 and it is the shape to keep.

**And it is serial, deliberately, for a measured reason.** Every worker's
own gate is `cargo test -p rondata --release`: **15.4 GB and 256 seconds**
(item 235). Two workers on that gate at once is not a coordination problem,
it is Friday's crash — 27.6 GB resident, swap full, the machine gone in
eight minutes. So the default is **one worker at a time**, and the
question reopens on 235's measurement rather than on taste. The capture
lane is the standing exception: it owns the screen, costs no memory, and
its products are logs outside the repo.

**The stopping rule is the one the queue already has.** A commander that
never stops will spend a budget on items that move nothing. So: an item is
spawned only with the score it moves named in its brief, and after **two
items in a row that move no score** the commander stops and asks rather
than continuing. That is what keeps no-human-in-the-loop from becoming
no-judgment-in-the-loop, and it is the same instinct as entry 24's.

**The operating notes, each of which cost something to learn.**

- **Stop a lane's old job before respawning it.** A lane lives in a named,
  persistent worktree, so a respawn under the same name gives two live
  sessions one working tree and makes by-name routing ambiguous. Workers
  never hit this: a fresh worktree and a fresh name per item.
- **Every spawn carries `--permission-mode auto`.** A worker that stops on
  a prompt is a worker that has silently ended the run.
- **Identity comes from liveness, not from a pid.** `ccc list`'s pid column
  can point at a pre-warmed spare process rather than the session.
- **A command chain gates on an exit code, never on grep finding text.** A
  red paperwork guard reached a commit that way on the first night.
- **`tools/memcap.sh` in front of anything that can grow.** A warning
  message cannot save a machine that is already thrashing.

**The measure.** The next steering pass asks whether the commander moved
the headline without the user in the loop, and what it stopped to ask
about. If the answer to the second is "nothing", the stopping rule is too
loose rather than the work too smooth.

**Amended the same day: two of the notes were already obsolete when they
were written.** `ccc` 0.1.25 shipped on 2026-09-04, minutes after this
session had hand-cut its first worktrees, and it closes both: `ccc spawn`
**defaults to `--permission-mode auto`**, and `--worktree` from a
non-default branch cuts off *that* branch and records it
(`branch.<b>.ccc-base`), with `--base <ref>` to force one and `ccc base
<session> <branch>` to record one for a worktree cut by hand — which
`merge`, `update`, `pull` and the roster's ahead/behind all read. So the
per-item loop is one command, `ccc spawn --worktree`, rather than a hand-cut
plus `--cwd`, and nothing can land a worker's branch on `main` by
accident. The bases for the branches live at the time were recorded by
hand. The lesson is the older one: **the tool's help was read once and
believed for a day**, while the tool was replaced underneath. What stands
unchanged is the note that a pid never names a session — that is the
harness pre-warming spare processes, and liveness is the only identity.

**Amended 2026-09-06, Fable steering, with the user — the width question,
reopened on 235's measurement as promised.** The gate is 14.8 GB at two
threads, so two workers are about 30 GB against 128 with `memcap.sh` in
front of each: memory no longer decides the width. What decides it is the
brief. So: **two workers when the second brief is as tight as the first's
and on a module the first does not touch**, at the commander's
discretion — one otherwise. The stopping rule counts landings in landing
order, whatever the width. The measure above was applied: the loop moved
Great Lakes 6848 → 6862 with nobody in the room and stopped to ask about
width and about the docs-versus-code wave — two real questions and no
spurious one. **The wave is not a ratification question**: its rows are
disagreements between two texts this repo owns, the diff is their oracle,
and they are taken per document by the worker that next touches it (the
queue's maintenance rule), never by a reader; entry 22's scope stands.
One more operating note: a worker takes its base's tip with **`ccc update
<ref>`**, and a brief never says `git merge` — the raw merge is what the
auto-mode classifier refused, and `update` is the same merge with guards.
Its origin-following fix (`origin/<base>` when origin strictly contains
local) is on ccc's next cut; 0.1.25 merges the local ref, so a commander
that pushes promptly is what keeps it honest until then.

**Amended 2026-09-07, Fable steering — the stopping rule met its first
stop, and the width note met the ceiling.** The rule fired at three
no-score landings in a row (241, 117, 249) and the commander spawned one
more worker instead of asking, on the stated ground that 249 had named the
headline's cause and pinned it with a failing-first assertion on the frame
itself; 250 then moved Great Lakes 6994 → 7176. The override was right and
the rule now names it: **a no-score landing that leaves a diff-backed
assertion on the headline's own frame resets the count**, because that is
the successor named with evidence rather than a residue chased. Anything
else still stops at two. The second half is a measurement the loop had
made without recording it: the gate — `cargo test -p rondata --release` —
peaks at **15,128 MiB at two test threads** and was **killed by the 20 GiB
memcap at 23,810 MiB** at this machine's default sixteen, on the same 222
tests, in the same worker, fifty seconds apart (loop-250, 05:11Z); the
steering session then reached **32,998 MiB** on the 230-test suite the same
way, and a `| tail` on the chain laundered memcap's 137 into an exit 0 —
the "gates on an exit code" note above, broken by the session that wrote
it. Nothing in the repo pinned the count; every worker that ran a green
gate had typed `--test-threads 2` by hand, and one ran the sampler under
`bash` and never measured. So width two rests on a pin, and the pin is
now a guard: `.cargo/config.toml` sets `RUST_TEST_THREADS=2` for every
invocation, `diff::floors::the_gate_is_pinned_to_two_threads` fails the
suite on a machine with dumps when neither it nor a `--test-threads` at
most two is set, and **no pipe stands between `memcap.sh` and the exit
code** — redirect to a file and read the file. With the pin, two workers
at two threads each are about 32 GB against 128, and the discretion above
stands unchanged.
