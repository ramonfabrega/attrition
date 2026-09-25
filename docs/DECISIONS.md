# Decisions

Newest last. Each entry records what was chosen, what it was chosen over, and
why — so a future session can tell a considered decision from an accident.

## Index — what stands, as of 2026-09-25

Nobody reads this file whole; a session reads the entry it is pointed at,
and an entry that has been amended or superseded does not say so at its
own heading. This index is rewritten by the steering pass, one line per
entry — `standing`, `amended by N`, `extended by N`, `superseded by N` —
and `docs_guard` checks that every entry has a row. The ledger below it
is append-only and amended in place, as it always was.

- 1 amended by 8 — Fidelity before divergence
- 2 standing — Simulation and renderer are separate
- 3 standing — Rust
- 4 superseded by 16 — Fixed-point, not floating-point
- 5 standing — Phase 1 is unconditional
- 6 standing — The shipped debug symbols are the specification
- 7 standing — The decompiler is a reading tool, not a source
- 8 standing — Fidelity is chosen per subsystem
- 9 standing — Data loaders are index-keyed; tag names are labels
- 10 extended by 19 — Exact rationals where the original used a float
- 11 standing — The simulation takes its tuning as an input
- 12 standing — A constant the original does not read is not tuning
- 13 standing — Pin a table the original computes with floating point
- 14 amended in place — A constant's scale is a fact about the loader
- 15 standing — Reproduce the original's arithmetic, including where it is wrong
- 16 standing — Integers at the original's scales; `Fx` is not the rule
- 17 standing — Derived per-player state is recomputed where the original recomputes it
- 18 standing — The rules name types by role
- 19 standing — A mid-frame float is reproduced operation by operation
- 20 standing — The AI's scripts are data, and the interpreter is ours
- 21 amended by 35 — `CLAUDE.md` carries the rules; the queue lives in `docs/`
- 22 amended in place — Opus by default; Fable by choice
- 23 standing — Where a reading's product is a formula, the implementation is a pass of the audit
- 24 amended by 25 — The score is the finish line, the queue deletes, and Fable steers
- 25 amended by 26 — The headline is the pair, the pin is the section
- 26 standing — The word is the instrument; the finish line is the tick pair
- 27 extended by 33 — The second lane is the capture lane, a session
- 28 standing — A check asserts its own inputs
- 29 amended by 41 — The finish line past the scored captures
- 30 standing — A mechanic that is worse than nothing is landed unwired
- 31 standing — Where a mechanic's answer is a number, proxy it
- 32 standing — The oracle runs on free Wine
- 33 extended by 34 — Two lanes for the two starved counters
- 34 extended by 39, 40 and 45 — The commander loop
- 35 standing — The paperwork is bounded by items, and a deletion is a claim
- 36 standing — The endpoint is telemetry until the word reaches it
- 37 standing — `forbid(unsafe_code)` is the whole tree's
- 38 extended by 41 — A lab branch lands by merge
- 39 extended by 40 — The queue holds what names a score
- 40 extended by 42 and 45 — The loop governs itself
- 41 extended by 42 — Two tracks: the rules on a golden record, the AI on the long captures
- 42 extended by 43 — The frame is the item; the loop's holes become guards
- 43 extended by 44 — A word is pinned with its widening, and a landing is committed before it is gated
- 44 extended by 50 — The instrument that agrees because it is not looking: four guards, and the lab merged beside the sim
- 45 extended by 50 — The chain's last link is the spawn, and lanes are throughput, not a pair
- 46 standing — A printed field is read or pinned, and struck text is not live text
- 47 standing — The lower map is a guard, a closed chapter is named closed, and the ledger checks its own list
- 48 standing — A booking cites the ledger, a falsifier names where it fires, and the second map is open
- 49 standing — The rules track's next axis is the issuer, and the day's holes are guards
- 50 standing — The instrument records what it compares, and the chain reaps the remote

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

**Amendment to 19, 2026-09-22 (item 495).** The second mid-frame float
the port reproduces is a rolled shot's arc and the ground under it:
`Ammo::inc_time`'s rolling arm and `TerrainOut::find_data_z`, scalar SSE
throughout. They are reproduced in `sim::single::Single`, the aircraft
bank's software single, operation for operation. `GRAV_Z` is a pinned
bit pattern (`0xc127cccd`), because its one writer runs before the first
frame. Entry 19's argument holds unchanged: every operation is correctly
rounded, so the original is deterministic here and the port is exact.


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

**Amendment to 31, same day.** The height grid stays in the millionths
the dump prints, and a single is recovered from each corner as the one
single whose six-decimal print it is. Where two to four share a print
(2,058 of Great Lakes' 58,081 corners, every one under 16 in magnitude)
the nearest is taken, a load-time choice at most a few ulp off. That is
not arithmetic in a frame. A terraformed corner is no longer known to be
the original's single, for the reason 31 already gives. Every read of
either kind is counted (`Sim::ground_inexact`) and held to a per-test
pin of zero, so the day a read lands on one fails a test by name. The
upgrade 31 anticipated, carrying the grid as singles and the terraform
in `Single`, stays local to `crate::terrain` and is owed when that
count first moves.


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

The cost is recorded rather than hidden: ~~the traced executable needs
`cover=0` under free Wine (item 226), so **function coverage is off** until
that is fixed~~ — paid on 2026-09-08: `cover=1` runs again on stubs that
write nothing after attach and use no `popad`/`popfd` (`docs/ORACLE.md`,
"Coverage is back"; run906). The coverage list is the queue of blind
readings (`CLAUDE.md`, "Prefer a diff to a reading"). Captures and diffs
were unaffected throughout.

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
shrink it until that is looked at — *unfrozen 2026-09-08: 802 cited, 650
entered, 152 never, on run53, run54 and run906 (ORACLE, "Coverage is
back")*. And the rule "every reading-only claim
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

**Amended 2026-09-07, the commander's own loop — a worker's message is the
completion signal, and session state is not.** An orchestrator needs to know
"has the work landed"; session state answers "what is the process doing",
and the two come apart constantly. In one day: a worker read `working idle`
long after it was finished; another read `working busy` while its report was
already delivered; a third finished unnoticed because the watcher was
filtering the refs of two workers already reaped, and the user had to say so.
**And a fourth never transitioned at all** — `loop-284` sat at `working
idle` with its branch pushed and its item done, so no watch could fire,
because there was no event. That one is not a filter bug and no flag fixes
it: the state machine simply did not move. It is the case that settles the
argument. **So every brief ends with an instruction to send the commander a
short "done" message** — the report stays where it is, and the ping is one
line on top of it; a worker that only writes its own transcript has told
nobody. The user proposed this three times before the fourth failure made
it unarguable, which is its own lesson about whose signal to trust.

**The ping is the wake; refs are the evidence.** An earlier draft of this
note said the worker's message *is* the completion signal, and that
overstates it in exactly the direction that nearly bought a bad merge: the
message is what makes the commander look, and `git log <base>..<branch>` is
what makes it true. ccc built the other half the same day — a `landed` event
keyed on the branch tip rather than on daemon state, so it fires for a
session that never leaves `working` — which covers the git-shaped part
without the worker having to remember anything. It covers **only** that
part: a capture whose product is a log outside the repo, or a reading whose
product is a document section, lands nothing a ref can see. So the ping
stays unconditional rather than "unless the detector covers you", because a
worker cannot reliably tell which case it is in.

**Measured the same evening, and it inverted the order.** On `lane-286`'s
landing the detector's `pushed` arrived at 20:52:27 and the worker's ping
seconds later — refs poll while a worker still has to finish its turn and
compose a message, so on a git-shaped landing **the refs are also the wake,
and the earlier one**. The ping keeps its unconditional place for the two
products refs cannot see; what it loses is the claim to being first. The
same run answered the rate: three commits, two `committed` events, one
`pushed` — so **`pushed` fires once per landing unless the commander
reopens the item**, which no detector can see because that is a
conversation and not a ref. `ahead` is the truth rather than the event
count: two commits inside one poll collapse into a single event.

**And a long-lived observer holds a snapshot of a world that moves.** Three
separate watchers outlived their subject in one day: one filtering the refs
of two reaped workers while the live pair had never been in it; one left
running against a file after its diagnosis was done; one still emitting a
superseded format hours after the binary under it was replaced, because
installing a build relaunches the *app* and does nothing to a running CLI.
The commander asserted "nothing on my side is on the old image" and was
wrong by two processes. So the rule is a measurement, not a recollection:
**`ps -eo pid,lstart,command` against the binary's mtime**, and a watcher
is restarted after any tool update rather than assumed to follow. This is
the same defect as the stale filter one level up — the observer is the last
thing anyone thinks to check, because it is the thing doing the checking.
Two earlier attempts failed the same way — a hand-rolled `ccc list --json`
poller that exited 0 with no output, and a `ccc watch | grep <name>` whose
filter matched other repos' sessions and then the commander's own status
text, because the state line and the detail line share a stream. **Every
peer message, all day, was accurate and prompt.** So: the worker's own
report is the signal, every brief asks for it explicitly, and monitoring is
demoted to a coarse dead-man watch — a crashed or quota-killed worker sends
nothing, and the quota wall takes every agent in flight at once, so silence
must still be detectable. Two riders. A brief must say **verify your own
landing before reporting it** — state the tip SHA and that
`git log <base>..<branch>` is non-empty — because a worker wrote "pushed"
from its plan while the run was still going, and the same sentence in a
journal entry has no verifier. And **an enumerated watch filter is a
snapshot of a roster that moves**: the missed landing was watched by a
filter naming two workers that had since been reaped, while the two live
ones had never been in it — valid syntax, matching nothing, silently. Any
filter written once has that bug; only lineage ("what I spawned") or push
survives it. The tooling half was filed with ccc and is not this repo's to
build, and ccc corrected the ask in a way worth keeping: a completion event
whose payload the *worker types* is the same unverified claim in a new
shape, so the payload must be **computed from refs**, and a dead-man
watchdog cannot itself be an agent — the quota wall would take it in the
same instant as everything it watches. Which is the real split: the roster's
state field answers "is it alive" well and "is it done" badly.

**Amended 2026-09-07, Fable steering, with lore in the room — the loop's
price, and the rule that was taxing it.** Ramon's doubt, raised while 289
and 290 ran, was right and the width was innocent: the hour a worker lost
went to the dump-backed suite in debug, reached through the definition of
done's own literal `cargo test`, and lore's census found the shape four
times in a day (235, 276, 289, 290). `testenv::dump` now refuses a debug
run in a second and the definition of done says `--release`. Width two
stands — the one contention event was self-inflicted twice over, and clean
width two is measured by lore per landing at the next steer, not ruled from
one pair. Two riders on the worker contract above, and both now live in
`CLAUDE.md` where a worker reads them: the done ping and the landing check,
verbatim; and **a status line at ninety minutes** from a worker that has
not landed, because wall clock is the signal contention corrupts and the
worker's own signal is the one that survives it. **The measure for the
next pass is a ratio, not a feeling**: lore priced the human-driven days at
21 USD a landing and 42 a word-moving landing, the loop's day at 32 and
102, with around 200 USD of the loop's 1,018 in shapes now guarded —
polling, idle turns, the debug suite. The loop pays for itself while those
stay closed; the next steer reads USD per word-moving landing, and a
doubling from 102 is a stop. `docs/audit/2026-09-07-fable-pass-2.md`.

**Amended 2026-09-17, Fable steering — the width is four under the
wrapper, and the wrapper is the gate.** The lab (entry 38) rebuilt the dump
readers so a test owns one frame's bytes rather than the capture it came
from, and measured the release suite twice on the same install: 248 tests
in 288 s at 10,869 MiB peak at two threads before, 280 tests in 122 s at
13,307 MiB at four monitored threads after; this tip's own merge gate ran
132 s at 12,646 MiB. So width two stays the bare `cargo test` default
in `.cargo/config.toml`; the floors test is now
`diff::floors::the_gate_has_a_bounded_test_width`, which reads the
`RON_TEST_MEMCAP_GIB` marker `memcap.sh` sets on its child and refuses a
wider run without it; and `python3 tools/release_gate.py <install>
--test-threads 4` is the gate a landing names — survey, the offline Python
tests, the release suite under `memcap.sh 20` with a fixture-request audit,
clippy, fmt and the paperwork guards, one command, gating on exit codes.
The marker is cooperative and the cap is what enforces. The loop's width
discretion is unchanged: two workers at four threads are about 27 GB
against 128.

**Amended 2026-09-17, the third Fable pass — the measure was applied and
the queue's population, not the loop, was the defect.** The rulings are
entry 39, which extends this one and overturns nothing in it.

**Amended 2026-09-18, the fourth Fable pass — the loop's own governance:
the commander's half of the chain, the stop at twenty, width on parked
rows, and the gate that ran every binary.** Entry 40, which extends this
one and overturns nothing in it.

## 35. The paperwork is bounded by items, and a deletion is a claim

**Decided 2026-09-07**, with the user, after lore priced the queue's
200-line cap and the morning's reap found three booked items that had died
without anyone noticing. Extends entry 21 (which file holds what) and entry
33 (only the main loop writes the queue); overturns the line bound entry 21
implied and `docs_guard.rs` enforced.

**The queue's line cap was buying the opposite of what it was for.** lore's
measurement across 221 sessions: 43 *fitting episodes* — the span from the
first `QUEUE.md` edit to the last `docs_guard` run in the same turn — 614
requests and 117 USD of list price since 08-25, with 22 of the 43 needing
two or more guard runs. The fitting loop was the cost, not the edit, and
the "count the lines before writing, not after" rule of `d03c279` had not
ended it. The session that received this measurement was itself the 44th
episode: it compressed **eleven** entries to seat three, and in one round
reflowed a paragraph past the file's own 76-column wrap to get under the
count — meeting the metric while defeating what it measures.

**But the argument that decided it is correctness, not tokens.** A global
line count is satisfied by compressing *any* item, so its remedy is a
retelling of entries the author has no reason to have read, and every one
of those edits is a chance to drop somebody else's finding. That is not a
worry, it is a post-mortem: `8b37e5f`, the post-crash rewrite that landed
item 227 and whose own message says "Recovered from the worktree the crash
left behind", shed 196 lines of queue and took items 230 through 234 with
it — the only run of five in the file's history. Two had landed. **Three
had not**, and one of those three, item 234, existed only as 406
uncommitted lines in a worktree that the next session was instructed to
delete. It survived because `git worktree remove` refuses a dirty tree.

So the queue is bounded **by items**: at most 18 open, at most 8 lines
each, with each section's *non-item* prose pinned where it stands and
falling only. There is no line count on the file. A new item costs no
compression of anything, which is the whole point — the pressure lands on
the item being added, applied by the person who knows what is safe to cut.

**And the backlog stops booting.** `docs/PARKED.md` holds what is not in
flight or next, and a fresh session does not read it; it is opened when a
*wave is composed*, which is a rarer moment. Two conditions came out of
asking the commander whether it could pick a wave without the backlog in
view — it can pick, but it cannot *brief* without it, because cross-item
constraints ("this file is about to be split", "this mechanism cannot move
a scored word") live in items nobody is taking. So: the parked file is read
at wave composition, and a cross-item constraint is stated in the worker's
brief or the item comes back. Rot is lore's to surface.

**A deletion is a claim, and `tools/queueledger.py` audits it.** A number
that leaves the queue is named in the journal (`item N`) or carries a line
in `docs/audit/queue-ledger.md` saying where it went. This wrote down the
existing practice rather than inventing one: **165 of the 191 retirements
before it already complied**. Its `not-landed` disposition is a promise
rather than a pardon — it fails until the item is booked again, because a
ledger that lets "this was lost" sit forever is the original bug wearing
the fix's clothes.

**The lesson under all three is one sentence: every check anyone ran was a
branch check, and the loss was one level below it.** `loop-234`'s branch
tip was a docs-only handoff commit that merged clean, so `git rev-list
--count`, ccc's roster and the queue's own reap instruction all agreed
nothing was owed while the work sat in the tree. ccc took that half (their
item 32) with a sharper name for it than we had: a detector keyed on the
wrong field, the same shape as their item 25.

**The measure.** If the next tranche books items without anyone mentioning
line counts, this worked. If a session is seen compressing an item it did
not write, it did not.

## 36. The endpoint is telemetry until the word reaches it, and the lane is owed by a rule that already exists

**Decided 2026-09-07**, the eighth steer (Fable, with the user), opened by
the commander under entry 34's stopping rule: five landings after 261 moved
no word, and the commander asked instead of spawning. Overturns the
assertion item 258 booked on the morning's steer; restores the 2026-09-01
rule in `docs/ORACLE.md`'s run57 ratification; extends entry 29.

**The ratchet is reverted.** The 09-01 pass *declined* a ratchet past the
word on evidence — item 134 improved fidelity and the past-the-word totals
moved both ways — and left the rule "assert up to the word, print past
it". Item 258 overturned that on a booking, arguing that a count of units on
the original's own tile cannot be made worse by making the simulation
better. The first landing after it falsified the claim (261: Great Lakes
`off` 73 → 75 while `unlinked` 7 → 2, on two value-diff-backed corrections),
and the line was then re-pinned *upward* three times in two days — 261,
265, 267 — every time on a mechanic the decompile owns, every time under a
rule that said it may only fall. The number is measured 16,500 frames inside
the divergence; a behavioural change at 6,751 re-deals every position after
it, and `off` counts the deal. A count like that is monotone in fidelity
only as the word approaches the frame it is measured on, and a guard whose
failures teach number-editing is not a guard.

What replaces it is an **exact pin, asserted in no direction**:
`rondata::diff::endpoint` fails when a count moves either way, and the
queue's `Endpoint` line must equal the pinned counts. A moved count is
re-pinned with the item's number beside it and nothing more is owed — no
trade under entry 26, no justification in the journal beyond the item's own
story. That keeps the one thing the ratchet was buying, that the number is
noticed and written down, and drops the ceremony. When a map's word reaches
24,001 the pin becomes the finish line's own assertion, and it reads zero.

**Item 266 resolves to entry 29's own rule.** It was booked when both words
sat in capture gaps; run87 landed one commit later and covers Great Lakes
7244–7520, so that map's word (7455) and its cause — the six-slot
assignment at the army's 7418 tick, 267's second row — are a diff against
disk. East Indies is the real gap: its coverage ends at run85's 7480 and
its word is 7529, so by the rule entry 29 ratified — a map earns its next
capture when its word crosses the newest one it has — the map is owed a
lane, sized to the word with headroom: `[7474, 7800)` at run85's detail,
six blocks over run85's tail, on run86/run87's precedent about ten minutes
and 200 MB. The lane runs beside the worker under entry 34's standing
exception. 266's other half, batching captures ahead of the word, is
declined for now and its reason filed on item 260: every capture is sized
to what `Log::parse` can hold, so the parser going lazy is what would make
a batch cheaper than a capture per word.

**The stopping rule stands.** It fired at two after 264's reset, exactly as
written, and the commander asked. Entry 34's own measure was applied: the
loop stopped for something real, not for nothing. What it caught was not a
regression in the work — 264, 265 and 267 are each sound, and 267's
widening found a never-compared bit of a word in twenty minutes — but a
queue with no headline-nearest item reachable from disk, which is the
lane's job to fix and the reason it runs beside the workers rather than
instead of them.

**One operating note for entry 34: the commander spawns; it does not work
items.** 267 was done in the commander's own thread — 224 requests, nine
tenths of them shell — on a handoff that read "commander on Opus", taken as
naming the session's model rather than the loop. The item cost what a
worker would have cost and left the commander's context spent and a clear
owed. The opener now says "spawning — never working", and a handoff that
names a model says which seat it names.

**And the pin failed on its first run, in the direction the ratchet never
looked.** East Indies' 24,001 counts were below their pins — `off` 80 → 79,
`extra` 11 → 10, `build_diverged` 32 → 30 — and so was the 16,489 rung. The
ratchet printed a fall to stderr, which a green `cargo test` captures and
shows nobody, so the handoff's `Endpoint` line had been wrong for at least
one landing. Re-pinned here; which landing moved them is not established.

**The measure.** If the endpoint line moves in the next tranche and nobody
writes more than the item's number beside the re-pin, the pin is doing what
the ratchet could not. If East Indies' word moves on the lane's window,
266 was the bottleneck it said it was.

## 37. `forbid(unsafe_code)` is the whole tree's, the data reader included — and a memory map is not a reason to narrow it

**Decided 2026-09-07**, with the user, on item 280. Overturns the exception
item 260 took (`docs/DATALAYER.md` §1); the rule it states is the one
`Cargo.toml` has always said out loud and no entry had ever written down.

**The workspace's `[workspace.lints.rust] unsafe_code = "forbid"` covers
every crate in `crates/`, and a crate lowering it to `deny` for a module of
its own is the same act as removing it.** `crates/sim` and `crates/fixed`
were never the point on their own: the tenet is that this codebase's
memory-safety argument is the compiler's, entire, and does not have a
carve-out a reader has to know about. The acceptance test is a grep —
`grep -rn "allow(unsafe_code)" crates` answers nothing — because a rule
that is only prose is broken within the week (`CLAUDE.md`).

**What was traded for it, and why the trade was wrong.** Item 260 found
that nothing a parse holds is ever given back: three captures parsed in one
process left 5,332 MiB resident with nothing alive, because macOS's
allocator keeps a freed block of that size rather than unmapping it. Making
the capture's text and the arena's chunks mappings gave that back —
measured at 14,721 → 12,182 MiB of the release suite's peak on the tree of
the day — and it cost a hand-rolled `unsafe extern "C"` mmap/munmap behind
a **safe** `read()` that handed back a `Deref<Target = str>` over
`MAP_PRIVATE`, with `from_utf8_unchecked` on top.

Three things make that a bad price rather than a close call:

- **The hazard is live in this repo, not theoretical.** `memmap2::Mmap::map`
  is an `unsafe fn` precisely because another process can rewrite or
  truncate the file under the `&[u8]` Rust believes is frozen.
  `tools/gamelog/runqueue.sh` renames `gamelog.txt` over an archive name,
  `tools/gamelog/captures.txt` warns in its own header that re-using a run
  number silently overwrites an existing archive, and the capture lane runs
  **concurrently with the test suite by design** (entry 34). So the
  sequence that turns a green gate into undefined behaviour is one this
  workflow can produce on an ordinary day.
- **The wrapper hid the obligation.** `read()` was safe, so no caller could
  see there was one; and the unchecked UTF-8 turned what would have been a
  SIGBUS into UB. A crate that keeps `map` unsafe was rejected in favour of
  hand-rolled FFI, which is a second surface the tenet exists to avoid.
- **It was not where the win was.** 260 landed two independent halves, and
  the larger one — indexing the frames and reading one when asked — is pure
  safe Rust. It took the arena of a 1.3 GB capture from 2,221 MiB to 17.
  The mapping was the smaller half of a change that was mostly not about
  mapping at all, and it is the only half that had to be paid for.

**What the rule permits instead.** A gate that is too heavy is a real
problem and the answer is not "hold your nose": read less, not more
dangerously. The successor named in `docs/DATALAYER.md` §1 is the one the
lazy index already makes possible — a frame read out of the file with
`FileExt::read_exact_at` into a reusable buffer, so a capture's text is
never resident at all, which beats the mapping rather than conceding to it.
A returning allocator (`#[global_allocator]`) is another, and needs no
`unsafe` in this tree because the crate that has it carries it. Both are
work; neither is an exception.

**The measure.** `grep -rn "allow(unsafe_code)" crates` stays empty, and
the next entry that wants to narrow this comes here first — an exception
that lives only in a mechanic's document and the journal is how this one
was taken without the user's word (item 280's brief).

## 38. A lab branch lands by merge, and adoption is a decision about defaults

**Decided 2026-09-17**, Fable steering, with the user, after a week away.

**What happened.** Between 09-08 and 09-10 a second harness (Codex, "the
lab") ran unattended off this branch's tip on `codex/methodology-exploration`:
80 commits, 185 files, 18k lines, a 54-row claim ledger of its own
(`docs/lab/LEDGER.md`), its narrative kept out of the journal
(`docs/lab/HISTORY.md`), and no score moved. It was asked to explore the
method, not to work the queue, and its ledger says of itself that it is an
adoption menu and not a merge proposal.

**The decision is to merge it whole and decide adoption by default versus
opt-in, not to cherry-pick.** Three reasons. It merged onto the tip with no
conflicts, and every runtime-side piece is behind a build flag or a
separate script. Its own adoption map warns that the commit titles are
mixed — a reader commit carries a tool, a tool commit carries a doc — so
picking groups is a week of archaeology that ends with the same tree. And
this repo's rule for landing is the gate, not provenance: the full release
gate ran green on the merge, so it is in.

**What is a default now**, because each is a guard or reader that was
measured against the old one and made to fail first: the dump readers own
one frame rather than the capture (the release suite 288 s → 122 s, entry
34's amendment); the soak digest carries the RNG seed, which it had
omitted, and reports gameplay activity apart from the clock, which a frozen
game had been passing on; the trace reader rejects unknown versions,
truncated tails and duplicate or skipped `FRAME` records; `memcap.sh`
refuses to launch when it cannot sample RSS instead of printing a zero
peak; `focus.sh` exits 1 on a missed window, and six `set -e` capture
scripts abort there rather than click the desktop.

**What is opt-in until a landing on the queue uses it**: the read-only
diff viewer and the in-memory replay checkpoint (`docs/DEBUG_VIEWER.md`),
to be piloted on the opener's first divergence; and the click-free capture
lane (`tools/explore/unattended_capture.py`, a `RON_AUTOSTART` tracer
build), which starts 6 of 10 pairs and is parked as item 298 until the
startup fault is closed. **What stays research**: the live call capsules,
native probes, search census and restore-prefix work under
`tools/explore/`. None of it is on any gate's path.

**What the lab said about the method, and the disposition.** Its central
critique is that the completion counters of entry 29 have no external
denominator: a citation regex over Markdown and two long games cannot
enumerate what a whole engine must do. The 09-09 steering disposition
stands: contracts and capability inventories are evidence and item types,
not an alternate score, and the three counters remain the finish line. The
two findings under it are real and are booked — 300, that the original's
command manager has 68 `issue_*` entries the fuzzer never calls while the
replay adapter lowers only `MoveTo`; and 299, a Gaia reseat correction
hiding a 556-frame heading difference the player comparator never scores.

**The rule going forward.** A lab branch lands by merge when its gate is
green on this tip; its ledger's "ready for pilot" rows become defaults
only when a queue landing uses them; its paperwork lives under `docs/lab/`,
which the guards do not scan, and anything it finds that wants the loop is
booked with a number like everything else. And the loop resumes after such
a landing, never during it: the gate a worker runs must hold still.

## 39. The queue holds what names a score; the board, not the loop, set the price

**Decided 2026-09-17**, the third Fable pass, with the user. Extends
entry 34 and its measure; overturns nothing.

The first Opus wave after the lab landed ran five workers and moved one
word: Great Lakes +94 (item 295), East Indies unmoved, item 301 banking
+381 unwired. lore priced it at **152 USD per word-moving landing against
the 102 the second pass set** — 1.5x, under the doubling that stops the
loop — and **1.61 USD per frame against 0.92 and 0.13** over the three
windows on record. The stopping rule fired correctly after three no-score
landings. What the price measured was not the workers: the queue's own
handoff named item 305 as headline-nearest while item 302's landing had
just written that the word's frame is a figure's draw and "not where the
make list's residue is", and no item on the board named that frame at
all. Sixteen of the eighteen open items named no score, because a queue
held at its ceiling for weeks (12.5 numbers booked a day against five
landings) had made every booking a forced triage under a rule — "least
score-connected parks" — that only the commander had. **The defect was
the population, not the cap.** Six rulings, each in the place a worker
reads it:

- **A finding parks by default**; the queue takes only what names the
  headline's frame, a floor, or a takes-chain to one. **The headline slot
  is never empty** — with no item on the word's frame, the next brief is
  the widening of that frame, the twenty-minute shape 294, 295 and 301
  all followed. `CLAUDE.md`, and the queue's maintenance rules.
- **Loop items are the steering pass's** — tooling, guards, the queue's
  rules — filed in `docs/PARKED.md`'s Loop section, never spawned. The
  user's framing: loop improvement is what the pass is *for*.
- **The journal is a directory**, `docs/journal/<date>-item-<N>.md`, one
  file per landing, the worker's; `git merge-tree` over the 58 merges since
  09-06 found **27 real conflicts** in `JOURNAL.md` and 7 in `QUEUE.md`,
  because an append-only file puts every addition at the same anchor. A
  worker now edits neither; it reports and the commander books.
- **A number is measured on the tip**, after the worker's last `ccc
  update`, or the report says which tree — 301's +381 was attributed to
  the wrong half against the pre-295 base, and would have landed wired.
- **Merge, gate, push, reap are one chain.** The reap was forgotten twice
  in one session as a step "before the next spawn"; a `land` verb is
  filed with ccc (item 313).
- **The ledger fails on a number the queue refers to and never booked.**
  Its first run found two: 304 ("wire when 304 closes"), the Great Lakes
  cause the banked suspend waits on, and 224, referenced since 09-04.
  Item 279's two regex defects landed in the same change, each made to
  fail first, and the real item 7 they had hidden was ledgered.

The next steer reads the same ratio, and asks one more thing: whether the
first wave under the parking rule spent its workers on the word's frame.
If 312 and 304 both land and neither moves Great Lakes, the diagnosis
here was wrong, and that pass says so.
`docs/audit/2026-09-17-fable-pass-3.md`.

## 40. The loop governs itself: the commander's chain in git, a stop at twenty, and width on parked rows

**Decided 2026-09-18**, the fourth Fable pass, with the user
(`docs/audit/2026-09-18-fable-pass-4.md`). Extends entries 34 and 39;
overturns nothing.

**The measure was applied and the loop converged.** Twenty-six landings
under one Opus commander since the third pass, twenty word-moving, Great
Lakes +1,503 and East Indies +1,899, at **31–39 USD a word-moving
landing** (lore, list price: workers 620.52 over 23 wells, the commander's
well 160.56) against 57, 152, 102 and 42 before it, and **0.18–0.23 USD a
frame** against 0.45 and 1.61. The third pass's test — 312 and 304 both
landing and both moving Great Lakes — passed. So this pass ruled on
nothing about the board or the workers, only on the loop's own rules,
which were living in three places and a per-user memory file.

**The commander's chain is in `CLAUDE.md`, and it is ccc's** (parked
345). `ccc merge <ref> --no-ff`, the gate to a file, `ccc push <ref>
--base`, `ccc rm <ref>` — the reap removes session, worktree and the
merged branch, and keeps a branch whose commits are nowhere else. Items
334, 336 and 338 were merged raw and reaped by hand, leaving dead roster
rows, because the only text saying otherwise was a memory hook a day
stale. Memory is invisible to every session but its own and read by no
guard; the fan-out rules are inherited by every session. The memory file
is now a pointer at the rule.

**Nothing under `docs/` is edited while a gate runs** (parked 318). The
handoff guards read the working tree because the gate runs before the
commit that would be the other source, and that stays. The race they hit
is the commander editing the queue during its own gate — a worker's tree
cannot see the commander's rewrite except through `ccc update`, which a
worker never runs mid-gate. A discipline, not a code change; it costs a
re-run each time it is broken and nothing when it is kept.

**The commander stops at twenty landings** (parked 331). The charter
fired a steering pass every twenty items and named no counter, so the
loop ran seven hours on 09-17 and twenty-six landings on 09-18 with
nothing in its rules able to end it; both clears were Ramon's. The
commander owns the count now — the handoff already carries it — and at
the twentieth landing since its boot it writes the handoff and ends its
turn saying the pass is due. It cannot switch the model, so stopping is
the whole act. Between, a free clear is taken at a **seam** — the
successor chain gone cold, a map crossover — never mid-chain, where the
context that made the briefs cheap is paid for again. The lineage rider:
`ccc spawn --json`'s answer is never filtered, because seven landings of
`lore jobs` parentage were lost to a `grep -E '"ref"|"branch"'` that
saved four lines of scrollback.

**Width two may run a parked value-diff row beside the word's frame,
never instead of it** (parked 330). Entry 39's rule — only what names a
score books — was made against a wave that ran residue rows *instead of*
the headline. Eight of the twelve landings item 330 measured were strict
successors, so the second lane sat idle. With the headline lane occupied,
a parked row that names a dump field and a frame, needs no capture, and
touches a module the headline item does not may be spawned from
`docs/PARKED.md` directly; its products park unless they name a score.
The stop stays a doubling — 78 from today's 39 — and whether the lane
*paid* is the next pass's, from `lore spawns`.

**The worker's gate runs every binary** (parked 339). `release_gate.py`
passes `--no-fail-fast`, tees the release run to a log with no pipe
between `memcap.sh` and the exit code, and prints a per-binary summary
naming each failed test. Rondata is red by design on every word-moving
item, and cargo's default stop at the first red binary had kept the sim
suite — `no_float`, `soak`, `docs_guard` — out of every worker's gate;
336 landed a red guard that way. The test that asserts the flag was run
against the old command first.

**The handoff's count of the Loop backlog is a guard** (parked 332).
`docs_guard::the_handoff_counts_the_loop_backlog` parses the handoff's
`Fable backlog: N Loop items` against the parked file's Loop section; it
failed first on twelve against thirteen. The steering pass has no queue
of its own: the section is the queue and the count is its opener.

**The measure for the next pass**: the ratio against 78; whether a parked
row ran beside the headline and what it cost; and whether the
twenty-landing stop fired. If the handoff's count and `lore jobs`
disagree, the commander's is the wrong one.

## 41. Two tracks: the rules on a golden record, the AI on the long captures — and the executable as the denominator

**Decided 2026-09-18**, the fifth Fable pass, with the user, in
conversation (`docs/audit/2026-09-18-fable-pass-5.md`). Amends entry 29's
finish line; extends entries 33 and 38; reverses one line of
`docs/INPUT.md` §8 for the rules track; overturns nothing else.

**What was asked.** Not the next item: how done the simulation is *as
code*, and whether anything measures it. The answer was that nothing did.
Entry 29's three counters — the long captures' word, the blind list, the
widening ledger — are all relative to what has already been inspected:
frames of two games, functions our own documents cite, fields the parsers
already carry. The lab's methodology note said so on 2026-09-09
(`docs/lab/2026-09-09-methodology-exploration.md` §4) and proposed a
capability inventory with an external denominator; it was ruled "not an
alternate completion score" then, correctly for the loop, and it is the
answer to this question now.

**What was measured** (`tools/census.py`, this pass; `docs/CENSUS.md`):

| slice | functions | cited in `docs/` | entered, three coverage traces |
|---|---|---|---|
| the executable | 48,233 | 862 in 158 classes | 7,180 |
| the 31 `*Order` classes | 410 | 44 | 205 |
| `Unit` | 184 | 123 | 120 |
| `Leader` / `LeaderData` | 128 / 173 | 46 / 34 | 60 / 95 |
| `CommandManager` | 82 | 2 | 12 |

Six order classes have no citation at all, and they are air, cast, trade
and the special animation; the thin ones — target, strafe, attack, guard,
follow, patrol at one citation each — are combat. Everything mapped is
economy, cities, movement, building and the AI's posture. By ages the
simulation is diffed through the Ancient and into the Classical, two of
eight, and the 24,000-frame long captures end in the Medieval at best: the
finish line entry 29 names leaves five ages never diffed *when it closes*.

**Why the loop is serial, structurally.** Lockstep replicates orders and
re-derives the AI, so no recording holds an AI decision (`docs/INPUT.md`
§1), and every scored capture since 08-24 has been an idle human against
an AI. Every rule verified so far was verified through the AI's own
orders, one decision at a time, on one trajectory per map; the harness's
input side maps two of eighty-two command kinds. That is why the headline
moves one landing at a time and nobody can write ahead of it.

**What already exists to change that**, all of it in the tree and run:
`rontrace.cmd` stages chat and console lines at exact sim-frames (run16:
`add`, `war`, `peace`, `tech`, `die`, `move`); `!ffwd` runs the sim at
~500 frames a second with the dump windowed off; the lab's congestion
probe (L15) staged a seeded scenario click-free — fast-forward, 32 spawns,
orders through the original's own `issue_move_to` from the DLL — and two
launches matched on every observed projection; `ai off` and `human who`
silence the Leader AI, and in `Unit::think` the auto-engage call precedes
that gate. A 24,000-frame draw-stream trace is 17–29 MB; a full-detail
window is ~0.5 MB a frame. The speed floor was never the sim.

**The decisions.**

1. **Two tracks, two words.** The *rules* track runs on a **golden
   record**: one staged game, AI off, in chapters — a chapter per order
   class and unit line, an age jump between chapters by the `age` cheat, a
   Temple chapter for the namesake, a war — driven from `rontrace.cmd` for
   the spawns and cheats and from native issuers for the orders, so the
   orders are in the recording and the cheats in the script. The *AI*
   track stays on the long captures exactly as today. Chapters are
   independent, so rules items run in parallel lanes with no dependency on
   the AI's next decision. The handoff carries a `Golden:` line beside
   `Scoreboard:` and `Long captures:` from this pass on — `none pinned`
   with the takes-chain until the first chapter pins, then the golden
   word — and `docs_guard` requires it, so the rules track has a slot that
   is never empty.
2. **Digest first, detail on demand.** Every capture is a draw-stream
   trace for its whole length; full detail is a windowed re-run sized to
   the word, regenerated rather than stored when the window moves. The
   good captures already do this; it is the rule now, and with
   fast-forward a golden-record re-run costs seconds.
3. **The harness's input side is a mechanic.** The command dispatch of
   `docs/COMMANDS.md` §3 mapped onto the sim's entry points, and an
   interpreter for the fixed cheat set the record uses (`add`, `age`,
   `tech`, `resource`, `war`/`peace`/`ally`, `die`, `damage`, `move`,
   `human`/`ai off`). `docs/INPUT.md` §8's "capture without cheats" is
   reversed for this track: a cheat is a modelled input from a small,
   listed set. Item 364.
4. **The census is the fourth counter, and the only absolute one.**
   `tools/census.py` over the Ghidra index, the documents and the coverage
   traces; regenerated at every steering pass into `docs/CENSUS.md`. It is
   the map, not a score — an entered function is one a run reached, not
   one whose predicate was checked — and the "diff-backed" column is
   hand-tagged later. Entry 29's finish line becomes four: both long
   captures in lockstep; the golden record in lockstep for every chapter;
   the blind list at an enumerated residue; the widening ledger at zero.
   Whether phase 4 waits on all four stays as 29 has it; the option to
   open it on the rules track alone is noted here and not decided.
5. **The golden record runs on the lab's lane** (entry 38's adoption
   question, answered for one tool): `live_session.py stage` with the
   tracer's command file, intro skipped, controlled exit. Parked 341's
   "expose the window and the categories" is part of item 363's brief,
   not a pass item; once 363 lands, a new capture stanza takes the
   click-free lane unless it needs the mouse. The native issuer covers
   `move_to` alone; further issuers are added the way the lab validated
   that one, under the emulator first.
6. **The AI gets its dump.** The original logs state and never reasoning
   because a re-derived AI needs no reasoning logged to stay in sync; we
   do. The tracer's proxy table gains the Leader's deciding functions —
   `compute_sites`, `action_respond`, `create_units` first — so the AI
   track diffs decisions, not only their effects. A Loop item, the
   pass's.

**Two corrections to the tree this pass found.** The blind list is not
frozen: `docs/PARKED.md` still said "101 of 617" a week after ORACLE's
"Coverage is back" (802 cited, 650 entered, 152 never), and the per-user
memory said `cover=1` page-faults — both fixed. And the blind *readings*
were the expensive, low-yield thing the diff-first rule starved on
purpose; the blind *list* is a free by-product of every coverage run and
the census subsumes it.

**The estimate, written down to be wrong on record.** By frames, the long
captures close in two to three months at the fourth pass's cadence. By
the census, the untouched half is the half with no script to read, so:
four to six months of the loop, at that cadence, to every order family and
every age diff-backed on the golden record. The next pass checks it.

**The measure for the next pass**: the golden word exists and which
chapters pin; whether a rules item ran beside an AI item; the census
re-run, order family first; the held-out third map's word, measured and
never debugged against; and the proxy table's first AI-decision diff.

**A candidate sentence for `CLAUDE.md`'s thesis, not landed**: *copy the
rules and the designs, leave the code and the container.* The file's
"engine" means the plumbing; the mechanics' implementations are the rules
it treasures, and the sync-category table, `DataWalk`, and a re-derived AI
are designs worth owning. Ramon's reading, in the room; the pass that
rewrites the file decides it.

## 42. The frame is the item; the loop's holes become guards

**Decided 2026-09-19**, the sixth Fable pass, in the main thread
(`docs/audit/2026-09-19-fable-pass-6.md`). Extends entries 40 and 41;
overturns nothing.

**What was measured.** Nineteen landings since the fifth pass, two lanes
all day, every worker verified on Opus 5 from its transcript. Nine moved a
word — Great Lakes 9,182 → 9,510 and the golden record 617 → 626, one of
those steps backward on purpose — and **thirteen of the nineteen briefs
named a mechanism that turned out not to be the cause**, while the frame
each named was right every time. Workers' list price 517 USD; per
word-moving landing 57 workers alone and 67 with the commander, against
31 and 39 at the fourth pass and the stop of 78 that pass set. Split by
track the picture is not one number: the rules track spent 268 USD over
ten items for six word moves (45 each); the AI track 249 over nine for
three (83 each, past the stop). Per landing the price did not move — 27
against 24. What doubled is the share of landings that moved nothing,
and every one of those was a brief that sent its worker to build on a
hypothesis the item before it had already refuted.

**The frame is the item.** Entry 40's width-two stop was written for a
lane running parked value-diff rows; that lane never ran — the second
lane became entry 41's rules track, and it is the cheaper of the two. The
stop is not applied to it. What the doubling on the AI track measures is
parked 377, three times over: a residue item titled by a mechanism books
a reading, and the reading has been wrong at every step since the word
entered combat. So a residue item is booked by its **frame and its draw
delta** — the two things that have held — and any mechanism the title
names is written as the previous item's hypothesis, not the subject. The
headline slot's standing rule already says this for the case of no open
item; it is now the rule for every residue item. The next pass measures
the AI track's price per word against 83, and the fraction of briefs
whose named mechanism survived.

**The loop's holes become guards, with the row pinned.** Six of the
fifteen Loop items were the same shape — a rule the day broke and a check
that could have caught it — and each is now a check or a clause:

- the handoff's `Golden:` line is read against the pinned word, the way
  the other two lines already were (parked 406; it read `w624` over 621
  for an item);
- the endpoint and golden guards say **who owns which half** — the
  constant is the worker's, the queue's line is the commander's (398),
  and `CLAUDE.md` says the same in the chain;
- the handoff guard names the span it counted and the literal phrases the
  other guards read (374);
- the chain gains `book` between merge and gate, with no spawn or update
  in the window that leaves the tree red (404), and the rule that a
  refused reap is a lane with work on the floor, never a lane that did
  nothing (411 — item 405's whole landing was found that way);
- a brief reserves the run number and the section number two lanes could
  both take (397);
- **a dead-listed function may be cited only where pinned** (321): the
  list is read from `docs/EMULATOR.md` §4's own enumeration, fourteen
  document-address pairs stood on the day and are pinned, a new one fails,
  and each pinned one is owed a reconciliation (parked 412);
- **a constant a specification names is built or pinned** (356): every
  `0x` value of two to four digits that no crate carries, seventy-seven
  across eighteen documents, pinned by file and allowed only to shrink — the
  shape that would have found `0xf6` a month early (parked 413);
- the unattended lane mutes the game (343), by the mechanism and restore
  `set_map` already uses; unexercised until the next click-free run.

Each guard was made to fail first, on an empty baseline and on a wrong
line, and the failing output is what the pins are.

**The estimate, read against the day.** Entry 41 said two to three months
for the long captures at the fourth pass's cadence, which was 1,500 Great
Lakes frames a day through the economy. This day moved 328, through
combat, and East Indies did not move at all. The census says why: the
new citations landed in `Unit`, `Leader` and `Group`, and not one in an
`*Order` class, so the word is now inside the half the documents never
mapped. The estimate is not revised on one day; the next pass reads the
Great Lakes rate against 328, and if it holds, the months are the census's
untouched half and not entry 41's.

**The held-out map is the generalisation number, and it is 1.** run106,
Himalayas, one tick and zero orders — the AI's scout picks a wrong
destination on frame 1. It stays undebugged by rule, and the pass records
it so nobody reads the two scored maps' floors as the sim's.

**Not taken**: the trailer guard (335), `memcap.sh`'s fixture (251), ccc's
`land` verb (313), the staged-run gate (375), and the AI's dump through
the tracer's proxy table (367) — the last is the one with a score behind
it and the first thing the next pass should build, since it is the only
instrument that would let the AI track diff a decision rather than its
effects.

## 43. A word is pinned with its widening, and a landing is committed before it is gated

**Decided 2026-09-21**, the seventh Fable pass, in the main thread
(`docs/audit/2026-09-21-fable-pass-7.md`). Extends entry 42; overturns
nothing.

**What was measured.** Sixteen landings on two lanes since the sixth
pass, both workers verified on Opus 5 from their transcripts. **No word
moved**: Great Lakes 9,510, East Indies 9,711, chapter one 626; chapter
two was pinned new at 616. Workers' list price 195 USD, twelve a landing
against twenty-seven at the sixth pass; the per-word price is undefined.
Of the eleven residue items booked with a hypothesis, the frame held in
all eleven and the hypothesis in one (438). And two days of the tranche
were a lost notification: item 441 finished, started its gate, and the
harness never woke the lane; the work sat uncommitted on a cleared
worktree with its branch showing nothing ahead of base, until the
commander read the tree by hand.

**Why nothing moved, and it is two different stories.** The AI lane
widened its frame first (408: 172 blocks, every record), and every item
after it was a measurement — a `LEADERS=9` window, a scoped probe, a grep
of a dump already on disk — that killed a reading with data. Six items
later it holds a defect confirmed against the original's own `inside_down`
chains and a coupling measured at −925 frames (438). That is the loop
working. The rules lane pinned a word (415), read a value diff **six
frames past it**, called the 140 units a seating error, and four items
read decompile on that premise (426, 434, 435, 437, 439) before the
whole-cast widening at the word's own frame (441) showed the seating exact
and the divergence one order. The rule that would have stopped it was
already in `CLAUDE.md` — widen every record on the frame before naming a
cause — and entry 42's convention was followed to the letter: every brief
named its frame and wrote its mechanism as the previous item's hypothesis.
The convention made the items short and honest, and three findings were
withdrawn inside the tranche instead of a pass later. It did not order the
widening ahead of the readings, because nothing checked whether a word's
widening existed, and a value diff at the wrong frame looks exactly like
one.

**The decisions.**

1. **A word is pinned with its widening.** `rondata::diff::testkit::
   WIDENINGS` names, for every pinned word constant, the test that
   compared every dumped record on the word's own frame — or the open item
   that owes it — and `floors::the_widening_behind_each_pinned_word_exists`
   checks the test is a `fn` in the diff crate or the item is booked or
   parked. Two words have none on file today: East Indies' 9,711 (parked
   444) and chapter one's 626 (parked 445), both parked because neither is
   a headline. **The widening is the first item on a new word**, before
   any item that names a mechanism, and an item that pins a word delivers
   it or names the item that will.
2. **A worker commits before it gates.** The gate's verdict is a second
   commit or an amend; a lane that dies mid-gate then has its work on its
   branch, where `git log <base>..<branch>` — the check `CLAUDE.md`
   already names — can see it. Entry 34's "committed" in the definition
   of done keeps its meaning; the order is fixed.
3. **The `Golden:` line composes lowest chapter first** (parked 417,
   `docs/GOLDEN.md` §1's own recommendation and the AI track's lower-map
   rule one level across): the first `w<frame>` is the lowest pinned
   chapter's word and every pinned chapter's word is on the line; the
   guard reads both constants.
4. **No conflict marker survives in the tree** (420):
   `docs_guard::no_conflict_marker_survives_in_the_tree`, made to fail on
   a fixture line in `captures.txt` — the file a stray `=======` once
   disabled whole for the capture driver.
5. **Six clauses, each a rule the tranche paid for once**, in `CLAUDE.md`:
   a payoff probe changes only the frames under test (431); a widening's
   own framing is a hypothesis, and a stanza writes what would kill each
   reading (419); a dumped record's slot index is not an identity (424); a
   message a commander sends during a gate says it is to be applied after
   it (433); an action announced in a closing message is performed in that
   turn or has not happened (436); a refusal whose remedy is in this repo
   and needs no human is taken and reported (421).
6. **A purchase draws nothing** (416): the draw-stream word is a **lower
   bound** on when a decision parted, never an estimate — 9382's purchase
   stayed invisible for 128 frames — and `BUILDQUEUE`, written from
   `BUILDS=1`, is the oracle for an AI purchase, because it is the only
   record that marks the frame a drawless decision was made on. Measured
   by 408 on the AI headline's own frame.

**Not taken**: ~~the AI's dump through the tracer's proxy table (367), for
the third pass, and it is the first thing this pass says to build~~ —
**built the same day, once the screen was free**: `RON_LEADER_PROBE`,
run114, `docs/AI.md` §52; the four offers item 432 reconstructed are on
the record with their values exact and their order corrected — the AI
lane's whole tranche had been spent recovering `create_units`' inputs by
probe and grep, which is what the dump now prints; the RUNS.md append point
(428); the trailer guard (335); `memcap.sh`'s fixture (251); ccc's `land`
verb (313); the staged-run gate (375).

**The estimate, read against the tranche.** Entry 41's months were
written at 1,500 Great Lakes frames a day; the sixth pass measured 328;
this tranche measured 0, with +72 in hand behind one unknown term. Two
combat points are not a rate and the estimate is not revised. The next
pass reads three: whether 442 landed its 72 or more and what stands at
9,582; whether 443 named the producer with the widening on file; and
whether the dump exists.

**The measure for the next pass**: the words, both tracks; the price per
landing against 12 and per word against 57; whether any word was pinned
without its widening (the guard says); whether a lane died with work on
its branch rather than on its floor; and the AI's dump.

## 44. The instrument that agrees because it is not looking: four guards, and the lab merged beside the sim

**Decided 2026-09-21**, the eighth Fable pass, in the main thread
(`docs/audit/2026-09-21-fable-pass-8.md`). Extends entries 42 and 43;
takes the lab's second tranche on entry 38's terms; overturns nothing.

**What was measured.** Seven landings on three lanes since the seventh
pass the same morning, every worker verified on Opus 5 from its
transcript. **Both headlines moved**: Great Lakes 9,510 → 10,161 →
10,232 (442, 456) and chapter two 616 → 624 (447); East Indies and
chapter one unmoved. Workers' list price 139 USD — twenty a landing
against twelve, forty-six a word-moving landing against fifty-seven at
the sixth pass, 0.19 USD a Great Lakes frame against 0.18–0.23 at the
fourth. Of six briefs that named a mechanism, the frame held in six and
**the mechanism survived in five** — against one in eleven at the
seventh pass — because each was written as a survivor of a widening with
its falsifier beside it, not as a reading's conclusion. No lane died;
every branch was merged; both headline words carry their widening.

**Why the day's findings are one family.** Four Loop items were booked
on 2026-09-21 and each is an instrument that agreed because it was not
looking: a widening keyed on `(who, o)` hid every order field after the
first behind a group-id residue, on the AI word's own frame (452); the
collision block was compared only when the positions agreed, so it was
unreadable on exactly the frame a position parts (453); the widening
guard checked that a named test *existed* and not what it widened, so a
row went stale by success for 651 frames (449); and a worker minted two
taken item numbers in the wrong file's form and no guard could see either
(461). Entry 42's rule stands — a loop's holes become guards — and each
is now a check made to fail first.

**The decisions.**

1. **A row is a field, never a unit, and the gated blocks are read at the
   parting.** `OrderMismatch::label` is the one name for an order row
   and `unit::rows` uses it; `compare` files the collision block,
   `start_dist` and the angles on the unit-frames whose position parted
   in `collide_parted`, `search_parted` and `angle_parted` — counted
   nowhere, so no residue floor moves, and readable everywhere, so a
   widening at a word sees `half_step@parted` beside `pos`.
2. **A widening declares its window beside the word.** `WIDENINGS` rows
   carry the block window the named test walks, the test reads its
   bounds from the same constant, and the floors guard requires the word
   strictly inside. A window the word walks out of fails rather than
   passing by saying nothing.
3. **An item number is minted once, in its file's form.**
   `docs_guard::an_item_number_is_minted_once_and_in_its_file_s_form`
   reads the queue's `N. **`, the parked file's `(N) **` and the journal
   directory, and fails on a number live twice, on a landed number still
   standing live, and on the other file's form. Its first run found
   three: two parked rows in the queue's form, and 423 live a week after
   it landed. A landed item's parked entry is closed or deleted.
4. **`docs/RUNS.md` merges by union** (`.gitattributes`), so two lanes'
   captures no longer conflict at the append point, and
   `docs_guard::no_runs_section_heading_stands_twice` catches the
   driver's one failure mode. Two lanes editing one section is not
   covered; the brief's section reservation is.
5. **The lane has a lock.** `ron_wine` refuses to launch while the game it
   last launched is alive, names the holder, and releases itself on the
   game's exit; `RON_LANE_FORCE=1` overrides. It closes the one case the
   human protocol (446) could not — a capture launched into a capture —
   and replaces nothing else.
6. **`memcap.sh` has its fixture** (`sim::memcap_guard`): a process past
   the cap dies with 137 in two seconds, and a status under the cap comes
   through. The guard had never fired on purpose in seventeen days.
7. **A trailer is the worker's own**, read from its own system prompt,
   never dictated by a brief and never the harness's reminder alone — a
   clause in `CLAUDE.md`, and a pass-time check against lore's
   first-request model, which this tranche passes throughout.
8. **The lab's second tranche is merged whole** (PR #4, `codex/jev-lab`)
   on entry 38's terms: its own ledger, unscanned by the guards, its
   products opt-in tools beside the sim. Nothing in it names a score and
   nothing books. What it holds for this loop is the retained memory
   packet — the "hour of synthesized state" `docs/EMULATOR.md` prices a
   singleton-reading function at, read offline — and the day a reading
   needs one names it.

**Not taken**: ccc's `land` verb (313), which is not this repo's to
build; 375 leaves the Loop for the ordinary parked list as a worker's
guard. **The estimate** is not revised: three combat points — 328, 0,
722 — average 350 Great Lakes frames a day, forty days to that map's
close, and East Indies has not moved since the sixth pass. Entry 41's
months stand.

**The measure for the next pass**: both words, and whether East Indies
moved; whether 463 was re-measured with the field-keyed rows and the
parted blocks before anything was named; the price per landing against
20 and per word against 46; whether the number guard or the lane lock
fired in anger; and whether the mechanism-survival rate held above a
half.

## 45. The chain's last link is the spawn, and lanes are throughput, not a pair

**Decided 2026-09-21**, the ninth Fable pass, in the main thread with
Ramon (`docs/audit/2026-09-21-fable-pass-9.md`). Extends entries 34 and
40; overturns nothing.

**Why a pass after two landings.** Not the count. The eighth pass ended,
the thread was cleared, and a fresh Opus commander read the opener,
spawned 462 and 463 together, landed both — both headlines moved, the
gate green — reaped both, and then **ended its turn asking whether to
continue**. The rules were followed to the letter, which is the point:
entry 40's chain is "merge, gate, push and reap are one chain" and "the
commander stops at twenty", and nothing between those two clauses says
what happens after a reap. The twenty-landing count had nothing that
counted past two. The user's question — did we break something, or was
it that nobody said "continue" — has one answer: "continue" would have
restarted it, and the chain would have stopped again at the next reap.
The rule was the defect.

**The chain's last link is the spawn** (parked 468). After `ccc rm` the
same turn refills the lane with the queue's first unstarted item on that
track, and a turn does not end with a lane empty while the queue is not.
The commander never asks whether to continue; the queue's opener is the
standing answer, and the stop at twenty is the only other exit.

**Lanes are independent** (parked 469). The commander read "a second
lane may run a parked row beside the word's frame" as two lanes moving
together, held 463's finished chain to batch it with 462's, and then
treated the empty roster as a natural end. Two lanes are concurrency:
each lands, is chained and is refilled on its own clock, and the
couplings that exist — the merge, and the run number and section number
a brief reserves — are handled at merge time and in the brief, never by
waiting. The width-two clause says what the second lane may work on, not
when it is spawned; it now says so.

**A booking commit is never amended under a live lane** (parked 467).
463's booking commit failed a queue guard and was amended; lane 462 had
already merged the earlier commit and caught the moved base only by
re-reading it before its own commit. A red gate on a booking commit is a
second commit. This is the same shape as entry 43's "a landing is
committed before it is gated": the tree that a lane sees is never
rewritten under it.

**The measure for the next pass**: whether the loop ran from a fresh
commander to twenty landings with no human turn between; whether any
lane sat empty while the queue held an unstarted item on its track,
which `lore agents` and the landing times answer; and whether a booking
commit was amended.

## 46. A printed field is read or pinned, and struck text is not live text

**Decided 2026-09-22**, the tenth Fable pass, in the main thread
(`docs/audit/2026-09-22-fable-pass-10.md`). Extends entries 42 and 44;
overturns nothing.

**What was measured.** Fourteen landings on two lanes since the ninth
pass, every worker verified on Opus 5 from its transcript, the chain run
from a fresh commander to its stop with no human turn — the ninth pass's
three clauses did what they were written for. **Both headlines moved**:
Great Lakes 10,233 → 10,277 (465, 478, 483, 487) and chapter two 637 → 683
(472, 479, 481). Workers' list price 369 USD, 26 a landing against 20,
53 a word-moving landing against 46 — and **4 USD a frame against 0.19**,
because both words are inside their first fight, where every frame
draws. Of twelve briefs that named a mechanism the frame held in twelve
and the mechanism in three; five journals price a wrong one at twenty
minutes, because the widening ran first. That number is retired as a
thing to steer on: it measures the previous worker's guess, and entry
42 made the guess cheap on purpose.

**What the tranche was made of.** Ten of the fourteen landings turned on
an instrument that agreed because it was not looking (entry 44's
family), and six of the ten were one shape: **a field the original
prints on every frame that nothing in this crate reads** — `damage_frac`
on a figure, `build_masks` on the right block, `near_o`, `recharging`,
`damage_frame`, and a whole `AMMO` family in a capture three days old.
Each was quiet everywhere, because a field that is not read cannot
part, and no widening of a window can find it. `CLAUDE.md`'s "diff the
whole record" was a rule with nothing checking it.

**The decisions.**

1. **Every key the dump prints is read, or pinned as unread** (parked
   488). `gamelog::reads` records, in test builds only, every key a
   parse asks a block for — keyed on the block and not its name, so the
   `OBJECT` under `UNITDATA` and the one under `WALLDATA` are two rows,
   which is 484's defect exactly. `rondata::diff::coverage` drives the
   harness's frame decode and every per-frame reader over the two
   headline windows, reads the printed side off the text by the indent
   rule (the arena's both-candidates rule would hold the reader to keys
   it never printed there), and pins the difference path by path. The
   pin is exact both ways: a key that arrives unread fails until it is
   read or pinned with its item; a key that is read fails until its row
   shrinks. First run: **20 paths, 239 keys**, three families never
   opened — `DEATH_OBJS`, the frame-level `GUY` list past its clock
   keys, the per-frame `WORLD` totals. A family with a parser of its own
   is named (`AMMO`) and left to it. The count is the fourth pass's
   measure from here on: it is the number of blind spots on the frames
   the loop stands on.
2. **Struck text is not live text** (parked 480). The section ceiling
   counts bytes outside `~~…~~` spans; a `~~~` fence is not a marker; a
   span is credited to the section it opens in and never past its end.
   A correction is not an addition, and a section at its pin gains
   exactly the room it strikes. Four pins fell on the first run and
   `AI.md` §15 — 27,421 bytes, 21,771 struck — left the table.
3. **The rules slot is the commander's to fill from the parked file at
   the merge.** Item 485 found the two sides of 683 and parked them,
   which a worker must; the commander's `Golden:` line then named a
   landed item as next. Booked as item 491, folding 492. A finding that
   names the headline's frame is never left parked over a merge.
4. **476 and 342 park** — value rows forty-three blocks under the word,
   and a residue with no frame — and 477 stays as the one value row on
   the word's own squad, written as a takes-chain candidate.
5. **Two clauses in `CLAUDE.md`**: a quiet field is checked against the
   reader before it is called agreeing; a `git checkout <file>` is never
   chained onto an edit that can fail.

**Not taken**: `combat::share`, written and documented and never called
until 485 — a function rather than a field, and nothing cheap catches
it; ccc's `land` verb (313). **The estimate**: not revised, and the
sentence under it changes — 328, 0, 722, 44 frames a day is not a rate
but two regimes, inside a fight and between them, and what ends the
expensive one is the instruments, not the mechanisms.

**The measure for the next pass**: the pin's count against 239 and how
many rows landings deleted; whether 491 moved 683; whether Great Lakes
left its fight and the frame price with it; whether any lane sat empty;
the price per landing against 26.

## 47. The lower map is a guard, a closed chapter is named closed, and the ledger checks its own list

**Decided 2026-09-23**, the eleventh Fable pass, in the main thread
(`docs/audit/2026-09-23-fable-pass-11.md`). Applies entry 41 §1 and
extends entries 44 and 46; overturns nothing.

**What was measured.** Twenty-nine landings since the tenth pass by
`git log` — nine on Opus 5 under one commander, twenty on Opus 5.5 under
the next, which counted only its own and called it twenty. Twenty-three
moved a word, against seven of fourteen at the tenth. Great Lakes
10,277 → 12,038; chapter two 683 → 900, chapter one 626 → 900, chapter
five pinned at 621 and closed at 900 the same day, chapter four pinned
at 1,277 — three chapters closed, and the namesake met its oracle,
where five reading-only claims were unwired rather than wrong.
Workers' list price 765 USD, 26.4 a landing, 33 a word-moving landing
against 53; **Great Lakes 0.23 USD a frame against 4.07** — it left its
fight, as entry 46 said the price would show. Opus 5.5's twenty cost
25.4 a landing against Opus 5's 28.6. The coverage pin: 239 → 224 by
reads, 369 when item 520 opened the leader record (165 keys pinned at
once), 360 now — the count doing what it is for. Nine of twenty-nine
landings turned on an instrument that agreed by not looking.

**What the pass found that no landing could.** East Indies' word,
9,711, has been the lower of the two since Great Lakes passed it on
2026-09-21. Entry 41 §1 makes the lower map's nearest divergence the
default item, and parked 444 said in so many words what to book the day
it happened. Three passes and some forty landings ran on Great Lakes
instead, because the queue's own line said "lower map first — Great
Lakes" and a commander reads the queue, never the constants. The rule
was prose, and the Great Lakes chase did exactly what the rule was
written to prevent.

**The decisions.**

1. **The lower map is a guard.** `the_handoff_s_default_map_is_the_lower_word`
   reads the queue's `lower map first — <map>` line against
   `LONG_WORD_*`. Made to fail on the tip, which named Great Lakes, and
   the default flipped: item 573 is East Indies' widening at 9,711; 571,
   Great Lakes' at 12,038, runs when a lane frees. Both words stand
   without their widening, so both are widenings before any mechanism.
2. **A closed chapter is named closed** (parked 528). A chapter whose
   word is its trace's last block reads `chN closed` on the `Golden:`
   line and never as a `w`; the line leads with the lowest *open* word,
   which is the rules headline. The guard parses the line part by part
   and fails on a chapter that reopens without being rewritten as a
   word. Made to fail on the tip's line.
3. **The ledger checks its own list** (parked 517).
   `every_differ_module_is_on_the_ledger` reads `src/diff/` and fails on
   a file on neither `DIFF_FILES` nor `NOT_A_DIFFER`. Six differs were on
   neither — `leader.rs`, the leader record's whole reader since 520,
   among them — and counted, the ledger fell 14 → 10 uncompared and
   39 → 36 single-capture: four fields called uncompared had been
   compared all along. Made to fail on a stray file.
4. **A comparison gated on both sides says which side is quiet** (parked
   503). Every `if let (Some(` in a differ carries a `both sides:` line
   above it naming what an absent side means on each; eight sites, one
   of which — `order.rs`'s `GROUPORDER` row — names a quiet disagreement
   it had never said. The first guard aimed at an instrument that says
   *yes*.
5. **The capture runner writes `success: false` with its error** on a
   timeout (parked 565), and a brief's wait keys on the runner's exit,
   never on a file.
6. **Five clauses in `CLAUDE.md`**: start is not stop (507); a commander
   files for the steering pass in the turn it notices (509); the count is
   since the last pass, by `git log`; the booking cites what the disk
   could not answer, and a field only where its write condition can
   answer (508); the delta in the constant, the block in the widening
   (513).
7. **526 leaves the Loop** for the ordinary parked list — a listing
   sweep is a reading, and the trap is in `tools/ghidra/README.md`;
   **527 stays**, its design named: a `compared` recorder in the
   comparators joined to `reads`; **313 stays**.
8. **The lab's two open PRs, #7 and #8, book nothing.** Both are
   score-neutral by their own account. #8's recompiled functions are a
   faster twin of `tools/emu/callfn.py` for a pure function, and this
   tranche's items turned on wiring and instruments, which no oracle of
   a function reaches. ~~What would earn a booking is #8's step 4: a
   native frame on a captured packet~~ *Amended the same day on lore's
   read*: what earned 327's formula was a **function run on a packet
   with its writes watched**, not a frame, and step 4 would only
   reproduce what a capture yields on stubbed imports. What earns a
   booking is that move inside a booked item — first **571**, a packet at
   the frame *before* 11922 (a packet at N is after N−1's decision) and
   the squad's move/stop path run on the nine guys under unicorn — with
   the wall time stated against the 12038 capture. A packet is a second
   run on the single capture lane, and #8's driver reads #7's decode, so
   the two opt-ins land together or not at all. ~~Merging is Ramon's
   call, on entry 38's terms.~~ **Merged the same day on Ramon's word**,
   `--no-ff`, on entry 38's terms — nothing books on the merge, nothing
   enters the gate, packets and lifted C never enter git — with the
   rungs named where a worker can find them: `CLAUDE.md`'s Tooling,
   `docs/EMULATOR.md` §8 for the costs, `docs/ORACLE.md` for the lane's
   rules, and 571's own entry for the first use.

**Not taken**: a guard on the commander's count (git inside a test);
the estimate, with no rate to revise it by and a second map and two
chapters still to open.

**The measure for the next pass**: whether 573 landed and where East
Indies parts; whether 571 ran and Great Lakes' frame price; whether the
lower-map guard fired on a move; the coverage pin against 360 and the
ledger against 10/36; whether any lane sat empty; the price per landing
against 26; whether the count reached the pass at twenty.

## 48. A booking cites the ledger, a falsifier names where it fires, and the second map is open

**Decided 2026-09-23**, the twelfth Fable pass, in the main thread
(`docs/audit/2026-09-23-fable-pass-12.md`). Applies entries 41, 46 and
47; overturns nothing.

**What was measured.** Twenty landings since the eleventh pass by `git
log`, 03:43 to 11:33 the same day, one commander and every worker on
Opus 5.5 by its transcript — the count reached the pass at twenty, as
entry 47's clause asked. Seventeen moved a word. **East Indies 9,711 →
11,069** over nine AI-lane landings at 0.19 USD a frame — the second map
opened at the price Great Lakes left its fight at, and it is still the
lower map, so the guard never had to flip. Chapter four closed at 1,500,
chapter seven at 1,200, chapter three pinned at 621 and closed at 900 in
six landings, and its restage stands at 782. Workers' list price 532 USD,
26.6 a landing, 31 a word-moving landing. The coverage pin 360 → 340 on
nineteen paths; the ledger 10/36 unchanged. Eight of the twenty turned on
an instrument that was not looking, and the three that moved no word
each closed values under it. The packet rung ran three times and named a
value the dump could not print each time; its frame rule was off by one.

**What the pass found that no landing could.** Two bookings were wrong
about the disk or the design and both were the loop's own: the pass
that wrote "grep the disk before booking a capture" booked 573 against
a dump it had not grepped for (575), and two chapters were staged so
their own falsifiers could not fire (584). Neither is a mechanic; both
are the shape entry 42 names — prose one level above the item.

**The decisions.**

1. **A capture booked on a map cites the window the ledger holds**
   (parked 575). `docs_guard::a_capture_booked_on_a_map_cites_the_window_the_disk_holds`
   reads `tools/gamelog/captures.txt`'s windowed stanzas and requires an
   open item that books a capture on a map to name its map and every
   window on it that spans the item's frame. Made to fail first twice:
   on 571 as written, which named a squad, a block and a capture and no
   map at all; and on 571 with `run136` spelled apart. Golden chapters
   are exempt — their ledger is `docs/GOLDEN.md` §14.
2. **A falsifier names where it could first fire** (parked 584).
   `docs/GOLDEN.md` §3 has a fifth point: the frame and the record,
   checked against the staging before the run; a staging that cannot
   reach a falsifier is restaged before it is pinned, and a premise a
   falsifier kills closes the chapter on what it measured and restages
   as a new one. **Chapter seven-b is item 628**, run156 and run157: the
   same five civilians for who=1, where the cheat's block decides, and
   the frame of `docs/INPUT.md` §11.9's seam. The closed chapters did
   not lean on the fallen premise: their who=1 units are soldiers whose
   auto-attack arm runs above the block.
3. **`writers::literal_fields` reads a one-line literal** (parked 596):
   a line is cut at the commas outside any bracket, and a bare
   identifier is shorthand when a comma follows it or shares its line.
   Made to fail first on `width, height: h, depth`.
4. **The ladder's shared-extras floor is an exact pin** (parked 583):
   ~~`SHARED_EXTRA_NUMBERS = 1`~~ **18 on the pin's first run** — the
   count had risen from 576's one under `!is_empty()` without a word,
   through the East Indies landings that put both rungs back on one late
   roster; re-pinned by the item that moves it in either direction; the
   rungs' identity rests on the 6,000-frame test.
5. **The packet is taken at the word's own logger frame** (parked 605),
   not the frame before: a packet at N is after tick N−1, so tick N is
   still ahead of it. `docs/EMULATOR.md` §8 and `docs/ORACLE.md` amended
   in place. **Where `lift.py` refuses, the oracle runs under unicorn on
   the packet** (parked 612), written into §8.
6. **Two clauses in `CLAUDE.md`**: a gate's exit is read before anything
   says pushed (574); a word's widening puts its window in the coverage
   driver (621's finding).
7. **598 leaves the Loop for item 620's brief**: `inside_up`'s outermost
   container becomes a `compare` row inside the next East Indies
   widening, where a worker can pin it against a dump. **313 and 527
   stay.**

**Not taken**: 527's recorder, again — the eight instrument findings
this tranche were each one field or one window, and a recorder is the
general fix a pass builds when the shape recurs past what the pins
catch; the estimate, still — but the band is now measurable: both maps
stand near 11–12k of 24k, and a frame has cost 0.19 USD between fights
and 4 USD inside one.

**The measure for the next pass**: whether 620's run155 landed and where
East Indies parts next; whether 628's pair ran and which falsifier
fired; East Indies' frame price against 0.19; the coverage pin against
340 and the ledger against 10/36; whether the new capture guard fired on
a booking; whether any lane sat empty; whether the count reached the
pass at twenty.

## 49. The rules track's next axis is the issuer, and the day's holes are guards

**Decided 2026-09-23**, the thirteenth Fable pass, in the main thread
(`docs/audit/2026-09-23-fable-pass-13.md`). Applies entry 41 §1 and §5
and `docs/GOLDEN.md` §13; extends entries 42, 45 and 48; overturns
nothing.

**What was measured.** Twenty landings since the twelfth pass by `git
log`, 13:03 to 22:19 the same day, one commander and every worker on
Opus 5.5 by transcript. Seventeen moved a word: **East Indies 11,069 →
13,640** at 0.034 USD a frame — 1,893 of them off the disk in one
landing — and **Great Lakes 12,038 → 12,536**, the lower map since 642;
the restage, seven-b, its control, chapter six and chapter eight all
closed, so every golden chapter is closed. Workers 437 USD, 21.8 a
landing, 25.7 a word — the cheapest tranche measured, and no fight was
entered. Twelve of twenty took no capture. Ten turned on an instrument
that was not looking, four of them the shape parked 527 names. One lane
sat idle ninety minutes on a wait keyed to the wrong file. The census,
regenerated: cited 922 → 981 in 169 classes, entered unchanged at
7,180, and the order family unmoved for the third pass — 44 cited of
410.

**What the pass found that no landing could.** The rules track had
closed every chapter it had, and the handoff asked what it is for.
`docs/GOLDEN.md` §13 answered on 2026-09-19 and no item ever booked the
answer: the chapters reach the order family *by accident* — every order
in a golden capture is one the original's own automatic play issued —
and order coverage is a separate axis whose unit of work is a native
issuer, validated the way the lab validated `issue_move_to` (L15). The
census row is what that looks like when nobody books it.

**The decisions.**

1. **The rules track's chapters are issuer chapters from here.** After
   six-b (651, the air premise's restage), chapter nine is **the move
   line**, item 676: `CommandManager::issue_move_to@00941720` from the
   tracer DLL on one unit and on a squad, over land with a world plan,
   `!ai off`, the lab's L15 shape, the issuer run under the emulator
   before the pair. The move line first because it is the one issuer the
   lab has validated live, and because the AI word's own residue (673) is
   a squad's move order and world plan. The order after it is GOLDEN
   §13's table by what a failure would teach; each chapter's section is
   its worker's; the census's order-family row is the counter, and the
   next pass expects it to move for the first time since it was built.
2. **A detached capture is waited on with `tools/gamelog/waitrun.sh`**
   (parked 656): it keys on `runqueue.sh`'s banner in the file the banner
   actually reaches — the viadriver log — and on the runner's life, and
   exits 0 / 1 / 2 on ok / FAILED / no banner. Made to fail first on the
   banner-less case, the one item 571 sat in for two hours. Named in
   `CLAUDE.md`'s wait bullet, `viadriver.sh` and `docs/ORACLE.md`.
3. **The golden-line guard reads every pinned word** (parked 638): a
   verdict function over eleven names — `ch1`–`ch8`, `restage`, `ch7b`,
   `ch7b-control` — and, with nothing open, the line leads with `every
   chapter closed` rather than borrowing `none pinned`. Fixtures made to
   fail on the line that fooled the old guard (`restage w792` over a
   closed constant), on a missing part, and on the borrowed phrase.
4. **clippy and fmt precede the release suite, and the gate ends with
   its steps line** (parked 639): `Gate steps: N of 6 ran (…); not
   reached: …`, printed in a `finally`, so a worker's "red only on my
   lines" is read off the gate and never inferred. The order test was
   run against the old gate first and saw clippy unreached; the two old
   tests that indexed the step list failed on the reorder before they
   were rewritten.
5. **The memcap self-test tolerates the box's load** (parked 645):
   timeouts 50 s, and the child's death is an elapsed assertion against
   a minute-long child.
6. **The receipt reads `callwin` back from `rontrace.cfg`** (parked 649),
   the file the tracer reads, as it already read `rontrace.cmd`.
7. **`Block::fields_of(&[keys])` reads named keys only** (parked 670),
   and `build_of` uses it for its `tx`/`ty` pairs. The coverage pin then
   failed first, as designed: fourteen `BUILDDATA` keys printed and never
   read, hidden since the pin was built by a whole-fields iteration.
   Pinned as unread with the item. The literal-0 half was fixed by 661
   and is the standing "grep this crate for a field" clause.
8. **A staging names its premise's killer and greps its writers, and a
   falsifier resting on a loop cites the loop's bound** (parked 630,
   667): `docs/GOLDEN.md` §3, point 5.
9. **A hypothesis the floor kills is not built by the item that killed
   it**: a clause in `CLAUDE.md`'s residue rule. Item 644 killed parked
   633 as the cause and built it "because it is the booked hypothesis";
   the booking is a frame, never a promise to build a name.
10. **313 closes on the measurement.** ccc 0.1.37 has no `land` verb; the
    chain as clauses and guards has run three tranches — sixty-nine
    landings — without a forgotten reap.
11. **527 stays, as a commitment rather than a deferral**: four instances
    this tranche (617, 642, 644, 661), none caught by a pin; **the
    fourteenth pass builds the recorder first, or writes here why not.**
    **677 is filed**: the bird's proxy — five landings this tranche met
    owner 9's rolls that no dump prints, and one took a packet to read a
    cursor; the fix is the AI dump's shape (entry 41 §6, run114) on
    `Unit::think_bird`.

**Not taken**: the candidate thesis sentence from entry 41, again; the
estimate — both maps stand past half their long captures with the price
between fights at 0.03–0.20 USD a frame and 4 inside one.

**The measure for the next pass**: whether 673 moved Great Lakes past
12,536 and at what price; whether 651 ran and which falsifier fired;
whether 676's issuer ran under the emulator before its pair, and whether
the census's order row moved; the coverage pin against 359 keys on
twenty-one paths and the ledger against 10/36; whether every detached capture
was waited on with `waitrun.sh` and no lane sat empty; whether workers'
reports quote the gate's steps line; whether any killed hypothesis was
built; the price per landing against 21.8; whether the count reached the
pass at twenty; and that 527 was built first.

## 50. The instrument records what it compares, and the chain reaps the remote

**Decided 2026-09-25**, the fourteenth Fable pass, in the main thread
(`docs/audit/2026-09-25-fable-pass-14.md`). Applies entry 44 (the
instrument that agrees because it is not looking) and entry 49 §11;
extends entries 40, 42 and 45; overturns nothing.

**What was measured.** Twenty landings since the thirteenth pass by
`git log`, one commander and every worker on Opus 5.5 by transcript, a
day's work from 23:05 on 09-23. Nineteen moved a word and all twenty
landed with the value diff beside it: **Great Lakes 12,536 → 15,384**,
**East Indies 13,640 → 15,985**, both past run78's end; six-b closed,
and five issuer chapters — move, patrol, guard, follow, garrison — each
run under the emulator before its pair, each closed at its trace's end,
and the census's order row moved for the first time since it was built,
44 → 50 cited of 410. Workers 679 USD, 33.9 a landing against 21.8 —
the issuer chapters' price, a first capture, a built issuer and a walk
each — with the AI lane at 0.055 a frame. Fifteen of twenty turned on an
instrument that was not looking, three of them the shape parked 527
names, and six on a specification line that called the cause dormant.
Four journals found the detached-capture waiter exiting 2 on the golden
lane and each wrote its own workaround; nobody filed it. Thirty-seven
merged remote branches stood.

**The decisions.**

1. **The shared instrument records every field it compares, and a pin
   holds the parser's records against it** (parked 527, owed since the
   eleventh pass). `crate::diff::compared` notes each `Record.field` a
   site of `compare`, `compare_orders` or `widen_block` compares with
   both sides present; `coverage`'s compared pin walks the Great Lakes
   word's own window with the recorder on and is exact both ways —
   fifteen records named as not the instrument's with who reads each,
   ninety fields of eight records pinned with the reason each stands.
   Made to fail first both ways; 12 s on the gate. Its first run caught
   a comment claiming a comparison the code does not make (parked 728).
   What it cannot see is a comparison that exists and is wrong — three
   of 527's five instances — and that half stays the workers'.
2. **`tools/gamelog/waitrun.sh` reads the click-free lane's receipts**
   (parked 656's sequel): once no runner is alive, a log holding the
   runner's JSON receipts is judged by them, and the default runner
   pattern covers both runners. Made to fail first on three fixture
   logs, 2/2/2 → 0/1/2.
3. **The chain's last link before the spawn deletes the lane's remote
   branch** (parked 727): a worker pushes its branch and `ccc rm`
   deletes only the worktree and the local one. The sweep of the
   thirty-seven standing is the user's command; the session's classifier
   refuses a remote deletion, and that refusal is not worked around.
4. **A brief reserves the code module when two items sit in one, and a
   fence lifts at the other lane's merge** (parked 726): three
   collisions in one tranche, each coordinated by message.
5. **A killer tests the claim's own unit, never the first row** (item
   711): one reading was false and its killer did not fire.
6. **A journal's own "for the Loop" line is filed at its merge**: the
   filing rule reached the commander's own findings and not a worker's.
7. **687 closes on a measurement**: `LEADERDATA` is 67% of a block at
   `LEADERS=9` and is what every AI widening reads, so the level stays;
   a capture's runway is 250 blocks past the word, from the tranche's
   ten word jumps (median ~205), and a jump past it takes a second
   capture at ~20 minutes rather than a first at 155.
8. **692 closes**: the cited column is the counter and it moved; the
   entered column waits on a coverage capture on the golden lane (697).
9. **677, 685 and 697 stay**, each priced with its next step in
   `docs/PARKED.md`: a proxy kind that reads memory, a packet that
   copies the game thread's stack, three six-second probes.

**Not taken**: the candidate thesis sentence from entry 41, again; the
estimate — both maps stand past 15,000 of 24,000, and at this tranche's
rate (+2,848 and +2,345 a tranche, no fight entered) the long captures
close in three to four tranches each, which the next pass checks.

**The measure for the next pass**: whether 722 and 708 moved their words
and at what price against 0.055 a frame; whether 723 ran `issue_form`
under the emulator before its pair and which falsifier fired; the
compared pin against 25 records and 137 registrations; the coverage pin
against 369/25 and the ledger against 10/36; every stanza's runway at
250 and no capture run to the gap; the golden lane waited on by the
receipt; a journal's Loop line filed at its merge; the remote listing
empty; the price per landing against 33.9; and the count at twenty.
