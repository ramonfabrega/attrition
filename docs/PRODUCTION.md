# Production: spending a price over time

`docs/COSTS.md` is what a thing is worth. This is what happens after you agree
to pay it.

A production queue in Rise of Nations is a fixed array of twenty-byte records
hanging off a building. Queueing an item charges its price **immediately**,
writes down what was charged, and starts a counter at zero. Every frame the
building runs, that counter climbs by a constant. When it reaches the item's
time, the building tries to hand the item over; if it can, the entry is
removed, and if it cannot, the entry sits at full progress and tries again next
frame.

That is the whole shape. Everything else is which constant, which time, and
what "tries to hand it over" means.

Not in it: the tech tree's prerequisites, which decide whether a type may be
queued at all; the market; and what a *building* under construction does, which
shares `BuildData::construct_time` with this document but is otherwise its own
mechanic. Those are surveyed at the end and specified elsewhere.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the spine — the queue record's layout is the PDB's own
`QueueItem` rather than an inference, and the queue, charge, clock, completion
and cancellation paths are each read end to end at their consumers. High for
the time formula's base and ramp, which are confirmed against the designers'
own column comments in `unitrules.xml`. High for the four loading scales, each
read at its own loader line. **Lower for the modifier tail** on `train_time`,
which is one expression repeated about thirty times; the individual predicates
are enumerated rather than each separately derived. The one arm of it that is
**built** is the British — and that one is diff-backed twice over on run58's
Dock clock, which is a different standard from the rest of the tail. What
remains open is listed at the end.

A blind second reading (`docs/audit/2026-08-20-production.md`) doubly
confirmed every number above and overturned several of the *predicates* around
them — which building the parallel-slot rule belongs to, what a blocked head
lets through, and what completing the first of a unit type actually produces.
Those corrections are landed below and marked where they changed what an
earlier draft said. Where the decompile had to settle a devirtualised `is`
call whose argument Ghidra dropped, the audit did it from the raw bytes; this
document cites the audit for those.

**Where the implementation is.** `crates/sim/src/production.rs`. Every constant
below is re-read from the user's own install by
`cargo run -p rondata -- <install>`, which fails if any has drifted.

---

## The queue record

`BuildData` carries two fields:

```
+0x82  uchar        queued        how many entries are live
+0x88  BuildQueue   build_queue
```

and `BuildQueueData` is a length and a pointer:

```
+0x00  int          queue_size    capacity
+0x04  QueueItem *  queue
```

`QueueItem` is twenty bytes and the PDB names every field of it:

```
+0x00  int       job_counter    progress, in hundredths of a frame
+0x04  short     type
+0x06  short[3]  good           resource indices, -1 for none
+0x0c  short[3]  cost           amounts actually paid
                                two bytes of tail padding
```

**Three pairs, for a price that has six slots.** `Type::load_cost` writes a
six-element array keyed by resource letter (`docs/COSTS.md`), and
`BuildQueue::set_queue` copies it into the entry by walking those six slots,
skipping any whose amount is zero, and **returning early once it has written
three**. Slots it does not reach are padded with `good = -1` and `cost = 0`.

So a queue entry remembers at most three of the six resources it cost. Nothing
in the shipped data spends four, which is why this is invisible in a stock
game and load-bearing in a modded one — and it decides exactly what a
cancellation gives back, because the refund path reads these three pairs and
nothing else.

---

## Queueing charges the price

`Build::queue_up(type, flag)` returns 0 on success and 1 on refusal, and sets
`BuildData::queue_fail` to say why.

**Research is queued at one library.** Before anything else, if this building
is a library and is not `LeaderData::get_first_library`, the entire call is
forwarded to that first library. Every research job in the game therefore lives
on one building's queue, no matter which library the order was given to.
`Build::unqueue` forwards the same way, and `Build::do_queue` returns at once
for a library that is not the first — so a non-first library's own queue never
advances, and **the parallel-slot rule below is a rule about that one
building.** `get_first_library` walks the owner's building list from its
lowest index and returns the first library that is active, stands in a city
and is not unassimilated; a library queue_up with no such building refuses
outright.

Then two gates, in this order:

- **`can_pay_cost`** — the type's own affordability check, described in
  `docs/COSTS.md`. Failing sets `queue_fail = QUEUE_COST`.
- **`BuildData::could_queue(type)`**, which sets `queue_fail = QUEUE_SUCCESS`
  and then:
  - `can_make(type, 1)` — can this building make this at all. Failing sets
    `QUEUE_CANT_TRAIN`.
  - `queued < queue_size`. Failing sets `QUEUE_FULL`.
  - one special case: a **University** (`0x1a4`) queueing a **Scholar**
    (`0x34`/`0x35`) refuses when its queued scholars plus its current
    gatherers would exceed six, also as `QUEUE_FULL`. (An earlier draft called
    this a dock's six-fishing-boat rule; the `is` argument is `0x1a4` and the
    type test is `0x34 || 0x35`, which by position in the shipped files are the
    University and the Scholar. The rule is not implemented either way.)

Note what is **not** here: the population cap. A player at the cap can queue
freely. What the cap stops is further down, at completion.

`queue_size` is decided in `Build::init`: **20** for a building whose
`build_flags` say military trainer or training building, **10** for one with
`build_flags & 0x08000000`, **2** for everything else, and `BuildQueue::init`
accepts only those three. ~~`crates/sim/src/production.rs` still takes the
capacity as an input, because the flags that choose it are a building-type
system the simulation does not have.~~ **The flags exist now**
(`docs/DATALAYER.md`, "The derived words no column carries"): all three are
derived at load and checked against the program's own values, so
[`sim::build::queue_capacity`] answers this exactly and `City::init_build`
calls it. `production.rs` still takes the number as an argument, which is now
only a parameter rather than a seam.

Past the gates:

```
if game.semaphore bit 11: cost = [0, 0, 0, 0, 0, 0]    # the scenario editor
else:                     cost = type.pay_cost(...)     # charges the stockpile

slot = queued
queued += 1
set_queue(slot, type, cost, 0)      # zeroes job_counter, records ≤3 pairs
num_queued[type] += 1
```

`pay_cost` both debits the six resources and reports what it debited, and that
report is what lands in the entry. **The price is charged on queue.** This
closes the question `docs/COSTS.md` left open: not on start, not on
completion — on queue, and the entry remembers the amount rather than the
recipe.

A handful of counters move alongside: per-production-building tallies used by
the AI, `ages_queued` and `epochs_queued` on the leader, and — for a queue that
was empty — an extra zeroing of slot 0's counter.

---

## The clock

`Build::process` calls `do_queue(0)` **once, every frame the building is
processed** — it returns before the call only for a building that is not
`is_active`, or, in the subclasses, one that is neutralized. There is no
phasing modulo here, unlike the per-unit upkeep in `docs/ATTRITION.md`, and no
gating on the building's index. The head of every live queue in the game
advances on every frame. (An earlier draft said "unconditionally"; the two
gates above are the only conditions, and both readings agree on "every frame
the building is processed".)

`Build::do_queue(i)` then returns at once in two cases, and otherwise does four
things.

**The two early returns.** A library that is not the player's first library
returns (unless the entry is `DISBAND`), which is the other half of the
forwarding above. And a **missile silo** (`is(0x208, 1)`) whose `inside_down`
is non-negative and whose entry is a unit type with the availability bit set
returns without advancing — a silo with a missile already garrisoned does not
build the next one. Neither is modelled; the second is recorded under what is
not established.

**It picks an accelerator**, by what kind of thing the entry is:

| entry | constant |
| --- | --- |
| a building type, or `DISBAND` | `ACCEL_CONSTRUCT` |
| a unit type whose availability bit is set | `ACCEL_TRAIN` |
| anything else — research, and the first of a unit type | `ACCEL_RESEARCH` |

All three ship as `1/1` and load as **100**. They are debug knobs; the
designers say so in the file, in a comment that ends "for main line game adjust
times individually -- BR". A global `ai_speed` above 1 multiplies the chosen
accelerator, which is a game-speed control and a cheat, and is noted in
`docs/MOVEMENT.md` as a possible desync of exactly the same kind.

**It advances the counter**, comparing before it adds:

```
target = ObjectData::train_time(this, type)
done   = target <= job_counter          # evaluated on the OLD counter
job_counter = min(job_counter + accel, target)
```

Because `done` is read before the increment, an item takes one extra frame past
the one on which its counter first reaches the target: the frame that lands on
the target sets it, and the next frame observes it. A `train_time` of exactly 1
is special-cased to complete immediately.

**At a library — and only at a library — it recurses into the next slot**,
when `i + 1` is below both `queued` and `LeaderData::get_building_cities()`.
That function counts how many of your cities contain a **library**. This is
the parallel-research rule: with the whole research queue living on the first
library, the number of library-holding cities is the number of that queue's
slots that advance at once. One library, one tech at a time. Four libraries,
four. The recursion happens before the handover below, so a deeper slot
completes in the same frame before the head does.

(An earlier draft stated the recursion unconditionally, and the implementation
applied it to every building — a barracks owned by a player with four library
cities advanced four entries at once. The recursion sits inside `do_queue`'s
third `is(0x1b3, 0)` branch, the library test; the audit settled the dropped
argument from the bytes. **Every other building advances slot 0, and slot 0
only** — with the one exception below, which is not a fan-out but a redirect
when the head is stuck.)

**And when the counter is done, it hands the item over** — the next section.

---

## Time

`ObjectData::train_time(type)` is the price document's `get_cost` with the
units changed: a base, a ramp against how many you already have, then a long
tail of national, wonder, government and technology modifiers. It returns
`max(1, t)`.

### The base, and the research step

Two virtuals, chosen by one bit:

```
if type is a unit type and the leader's availability bit for it is CLEAR:
    base = type.research_time(player)
else:
    base = type.time(player)
```

and those are:

```
TypeData::time(player)          = job_time * 100
TypeData::research_time(player) = time(player) * RESEARCH_TICK_PREMIUM >> 8
UnitTypeData::research_time(p)  = TypeData::research_time(p)
                                    * research_premium_time >> 8
```

This is the mechanic every Rise of Nations player knows without having a name
for it. **The first one of a unit type you build is a research job**, and it is
a research job in both senses at once: it takes `RESEARCH_PREMIUM_TIME` longer,
and — because `do_queue` tests the same availability bit — it advances at
`ACCEL_RESEARCH` rather than `ACCEL_TRAIN`. Every one after it is a train job.
The same bit drives both, so the two can never disagree.

`RESEARCH_PREMIUM_TIME` ships as `2` for 356 of the 364 unit records, and as 1,
3, 4 or 5 for the other eight. So researching a unit usually costs twice its
build time.

The `time` path has one branch of its own: a spell type the leader does not yet
have returns `res_time * 100` instead, which is the same "research it first"
idea for spells.

### The ramp

For a unit type whose availability bit is set — that is, everything past the
first — three lines:

```
t   = UNIT_RATE_BASE * t / 100
cap = t * 3
t   = num_units[type] * job_extra_time * UNIT_RATE_PROGRESSION + t
t   = min(t, cap)                       # with both terms guarded non-negative
```

`num_units` is the leader's live count of that unit type. It is reached as
`leader + 0x56fe + type * 2`, which is not a separate array: `num_units` is
`ushort[352]` at `+0x5762`, unit type ids start at `0x32`, and
`0x56fe + 0x32 * 2 = 0x5762`. The compiler folded the subtraction into the base
pointer. This identifies the first of the two per-type count arrays
`docs/COSTS.md` listed as unknown; the second, at `+0x5a22`, is `num_queued`,
`ushort[806]`, which `queue_up` increments and `unqueue` decrements.

**And only the first of the two appears here.** The price ramp sums
`num_units + num_queued`, so ordering five hoplites at once makes each one
dearer than the last. The time ramp reads `num_units` alone, so those five all
take the same time. Order five and you pay a rising price for a flat wait;
build them one at a time and you pay the same rising price for a rising wait.
Nothing in the data suggests that asymmetry is deliberate, and it is the
original's arithmetic either way — `docs/DECISIONS.md` entry 15.

**The ramp is inside the availability branch, and that matters more than it
looks.** `UNIT_RATE_BASE` is applied here and nowhere else, so a *research*
job — the first of any unit type — is not scaled by `6/5` at all. A Citizen's
first costs `JOB_TIME` doubled by `RESEARCH_PREMIUM_TIME`, a flat hundred
frames, and produces no citizen (see "Completion"); the first one *trained*
costs `JOB_TIME` stretched by a fifth with nothing yet owned, sixty; the next,
ramped once for the one standing, sixty-seven and a half. The twenty percent
every trained unit pays does not apply to the one that unlocks it. (An earlier
draft had the second order at sixty-seven and a half, because it also had the
research entry delivering a unit.)

`UNIT_RATE_BASE` ships as `6/5` and `UNIT_RATE_PROGRESSION` as `3/4`, both
scaled by 100, so 120 and 75. `job_extra_time` is the `JOB_EXTRA_TIME` column,
`1/10tsx` for 360 of 364 records and `0/10tsx` for four, loaded as 10 and 0.

So for a Citizen, whose `JOB_TIME` is 50:

| | hundredths | frames | seconds at 15 fps |
| --- | --- | --- | --- |
| base × `UNIT_RATE_BASE` | 6000 | 60 | 4.0 |
| per citizen already owned | +750 | +7.5 | +0.5 |
| ceiling, 3× the base | 18000 | 180 | 12.0 |
| citizens at which the ceiling binds | | | 16 |

The ceiling is the same shape as the price ramp's four ceilings and a good deal
blunter: one multiplier, three, for everything. Build time at most triples.

### The tail

Roughly thirty modifiers follow, all of the same three shapes —
`t = t * 100 / (K + 100)` to speed up by K percent, `t = K * t / 100` to scale
directly, and a plain `t * num / den` for the handful written as shifts. Which
of them apply depends on the tech tree, the wonder list and the nation roster,
none of which the simulation models yet, so `crates/sim/src/production.rs`
takes the tail as an ordered list of those three shapes rather than as thirty
predicates — the same call `docs/MOVEMENT.md` made for `speed`. They are
enumerated here rather than each derived:

- **Nations.** Japanese barracks and carriers, Chinese citizens (with an
  `_INSTANT` variant that returns 1 outright), British ships, archers and
  anti-air, French siege and special units, German aircraft and submarines,
  Roman legions, Mongol stables, Greek research, Lakota razing.
- **Wonders.** Angkor Wat on research, Statue of Liberty on ground units,
  Porcelain Tower on ships, Space Program on aircraft, and the
  **Supercollider** — wonder `0x21d` — which zeroes research time entirely for
  everything that is neither a unit nor a building. (An earlier draft named
  `0x21d` the Pyramids; by position in `buildingrules.xml` the Pyramids are
  `0x20e` and `0x21d` is the Supercollider, whose whole point in the shipped
  game is free research.)
- **Governments and technologies.** Monarchy at two levels, Socialism,
  `TROOPS_FASTER` and `RESEARCH_FASTER` as a three-quarters multiplier,
  `SPIES_GENERALS_CREATED_FASTER` as a half, `INSTANT_UNIT_BONUS` returning
  `ACCEL_TRAIN * 5 - 1` so the item completes in five frames.
- **Rares and upgrades.** Cotton, wool, uranium, relics, and the ship, troop
  and vehicle speed upgrade counts as a `(10 - n)/10` factor.
- **Generals.** "The President" on both buildings and units.
- **Lobby and handicap.** The tech-cost setting as 2/3 or 3/2, a handicap as
  `(200 - h)/200`, and the Conquer-the-World Great Thinker card, applied once
  per card in a loop.
- **A catch-up discount on ages and epochs**, which is the interesting one:
  the game counts how many other players are ahead of you, weighting an
  opponent double and a teammate single, and scales the time by
  `(2N - ahead + 2) / (2N + 2)`. Falling behind in ages makes catching up
  cheaper in time, automatically. The Greek research bonus is applied after
  this, and only on this path.
- **A catch-up discount on ordinary technologies** too, which the earlier
  draft omitted: for a tech that is neither an age nor an epoch,
  `n = TypeData::count_discovered()` — how many players already have it — and
  if `n > 0`, `t = (P - n + 1) * t / (P + 1)` with `P` the player count. The
  Greek bonus does not apply here.
- **A science speedup**, `TECH_SCIENCE_SPEEDUP` percent per level of science
  above the tech's own level — `t -= (epoch[3] - level) * TECH_SCIENCE_SPEEDUP
  * t / 100`, written as the `-0x51eb851f` magic multiply *added* to `t`, so
  it is a truncated term subtracted rather than a `(100 - pct)` scale. Three
  things separate it from the price side's `calc_science_discount`, and each
  is worth stating because two of them look like the other function and are
  not: the constant is `TECH_SCIENCE_SPEEDUP`, a second `Tuning` entry that
  merely ships at the same ten; `level` is the tech's `AGE` column **raw**,
  with none of the price side's plus-one for a plain tech; and it is gated on
  `level < epoch[3]`, so falling behind costs nothing in time where the price
  side turns the same distance into a surcharge. For a **unit or building**
  research job — a type whose availability bit is clear — `level` is not the
  type's own, since only a tech record has an `AGE`, but its first
  prerequisite's: `TypeData +0x30` is `preq[0]`, `TechTypeData +0x1c8` is the
  column, and a negative `preq[0]` reads as level zero.
  `crates/sim/src/production.rs`'s `science_speedup`, and **diff-backed** —
  see "The record, diffed".
- **An unassimilated-city penalty** at the very end, applied to everything:
  a building in a city whose owner's race is not the player's, while the
  assimilation flags say so, takes `t = t * 5 / 4`. Then the `max(1, t)`.

**The tail is partitioned, and the partition is load-bearing.** `train_time`
reaches the research-only block — `RESEARCH_FASTER`, Angkor Wat, relics, the
Great Thinker loop, the handicap, the lobby tech-cost setting, the science
speedup, the Supercollider zero — only when the type is a technology or a unit
or building type whose availability bit is **clear**; a train job jumps over
it to the common tail. The train-only block — `UNIT_RATE_BASE`, the ramp, the
national unit bonuses, `TROOPS_FASTER`, the speed-upgrade counts, cotton, wool,
`SPIES_GENERALS_CREATED_FASTER`, Monarchy, Socialism, the unit wonders,
Kremlin and `INSTANT_UNIT_BONUS` — is reached only with the bit set. Since
`crates/sim/src/production.rs` takes the tail as an ordered input, **the
caller must never feed a research-only modifier to a train job or a train-only
one to a research job.** The full order, spot-checked against the decompile, is
in `docs/audit/2026-08-20-production.md` §3 O7.

### The tail's first caller: the British arm (2026-09-01)

`Sim::queue_target` now builds a train tail as well as a research one, and
the partition above is what it obeys — a researched type gets the national
block and no science speedup, a research entry the reverse.

**Only the British arm of the national block is built**, and that is a scope
claim rather than an omission. `ObjectData::train_time@006508c0` runs ten
national arms in a fixed order; nine of them belong to nations no capture on
disk plays, so each would be a predicate nothing could falsify — and the
audit's standing lesson is that the predicates, not the arithmetic, are
where a reading goes wrong. The British arm is the exception: **player 1 is
British on both East Indies captures** (`tribe 11` in run38's `PLAYER`
block), and it is what the AI's Dock clock is measured against.

The arm is three tests inside one `has_tribe_bonus(0xb)`, in this order:

```
if domain == 1 (sea):    t = t * 100 / (BRITISH_SHIP_SPEED + 100)
if is(0xaa)   (Bowmen):  t = t * 100 / (BRITISH_ARCHER_SPEED + 100)
if is(0x119)  (AA gun):  t = t * 100 / (BRITISH_AA_SPEED + 100)
```

`domain` is `UnitTypeData +0x218` — the same field the German air arm reads
as 2 and `LeaderData::get_ships_speed_upgrade` reads as 1. The two lineage
roots are `TypeIndex` ids, not names: `0xaa` is **Bowmen**, the Archers
line's root, and `0x119` the **Anti-Aircraft Gun**. `BRITISH_SHIP_SPEED` and
`BRITISH_AA_SPEED` both ship as **33**; `BRITISH_ARCHER_SPEED` ships as
**0**, so that middle test is live and inert at once — `t * 100 / 100`. The
anti-air constant is one constant with two readers: this one and
`Wall::update_construct_time@0063d560`, which `crates/sim/src/build.rs`
already had.

**Starting the tail at the British arm is exact here rather than
approximate.** Everything the original applies before it is absent from this
game: the lobby handicap is `0` on both players in run38's dump, and The
President, the Mongol stable, the Japanese barracks and carrier and the
Chinese citizen are all other nations' powers, gated by the same
`has_tribe_bonus` this player fails. What comes *after* it — `TROOPS_FASTER`,
the speed-upgrade counts, the rares, the governments, the unit wonders — is
unbuilt and listed under what is not established.

---

## Completion, and what the population cap actually stops

When `do_queue` sees `done`, it pins the counter to the target, calls
`Build::finished(type)` and reads the sign of the answer.

```
if type is a unit type and the availability bit is SET:
    if pop_cap < type.control_cost + leader.control:   return 0
    if it is a caravan and count >= caravan limit:      return -1
    if it is an aircraft and the pad is full:           return -1
    train(type)
    return 1
if type is a spell the leader has:   cast it;            return 1
if type is a building type:          (the construction path)
otherwise:
    Leader::gain_tech(type, x, y, 1, 1)                  # sets the bit
    return 1
```

**The first of a unit type completes as a technology, and trains nothing.**
A unit-type entry whose availability bit is clear falls past every branch
above to `Leader::gain_tech`, which sets the bit — `BitMask::set` on the
leader's `tech` mask at `+0x6c0c`, the bit this whole document turns on — and
returns 1. No unit is placed. The pop-cap, caravan and aircraft checks are
inside the bit-set branch, so a research entry is subject to none of them
either. The player then queues again, and *that* entry is the first train job:
`ACCEL_TRAIN`, `UNIT_RATE_BASE`, the ramp. (An earlier draft was silent on
this and the implementation spawned a unit from the research entry, so the
first order of a type yielded both the bit and a unit. It yields the bit.)

`Build::train` itself is short. `Objects::init_unit` places the unit at the
building's own position and returns its index, or a negative on failure;
`Unit::go_inside` then puts it *into* the building, so every unit is born
garrisoned and leaves by the ejection path — **except a scholar trained at a
university, which stays in it** (`docs/CITIES.md` §6.5.2: the exit block's
`is(0x1a4)` arm re-reads the trained type's `is_scholar` and compares
`gather_max` against `num_inside(1)`). Stance and gather-point orders are
applied on the way out. `finished` returns 1 whether or not `init_unit`
succeeded, so a placement failure still removes the entry.

The caller reads the sign, and what it does with it differs between the
library and everything else.

**At a non-library building:**

- **positive** — `Build::unqueue(i, 0)` removes the entry, with no refund. If
  the entry was a train job and the building's `build_masks & 0x40` — the
  **infinite-queue** flag — is set, the bit is cleared, `queue_up(type, 0)` is
  tried, and the bit is set back on success; `unqueue` clears it when the
  queue empties. ~~Not modelled.~~ Built and diff-backed: "The infinite
  queue" below (item 877, run285).
- **zero or negative, at any slot but 0** — nothing. The entry stays at full
  progress and the attempt repeats next frame.
- **zero or negative, at slot 0** — the entry stays, and the building looks
  for a *different* slot to advance this frame instead. First
  `BuildQueueData::get_next_non_unit`: the lowest slot from 1 up whose entry
  is not a unit type, or is a unit type whose bit is clear — **the first
  research entry** — and if there is one, `do_queue` runs on it. Only if there
  is none, the queue holds at least two, and the answer was **negative**, the
  building tries `get_next_helicopter` (at an Airbase, `is(0x1bf, 0)`) or
  `get_next_non_caravan` (anywhere else), and runs `do_queue` on that. A
  redirected slot that is itself done and refused does nothing further, by the
  "any slot but 0" rule.

(An earlier draft said zero means "nothing happens" and that "a pop cap blocks
the whole queue". It blocks every *train* entry behind it; a research entry
behind it advances, one frame per frame, and completes. The negative case's
non-caravan / helicopter search is a second fallback after the research
search, not the first response.)

**At the first library:** `finished()`, then `unqueue(i, 0)` if the answer is
non-zero and the building's `+0x8 & 1` is set. There is no redirect — the
library's parallelism is the `get_building_cities` recursion above, which runs
whether or not the head is done. Note the quirk: a *negative* answer at a
library removes the entry too. With shipped data a library queues only
technologies, whose answer is always 1, so the quirk is unreachable.

**This is what the population cap does.** `docs/COSTS.md` asked what stops the
queue rather than the spawn, and observed that `check_population` has no
queue-aware variant. It does not need one. `do_queue` calls
`LeaderData::check_population` too, but only to raise a message and bump a
counter — the accelerator it picks afterwards is `ACCEL_TRAIN` either way, so
**progress runs to completion at the cap and then stalls there**, at 100%,
paid for, indefinitely. The unit appears on the frame after a citizen dies or
a house goes up. Which is exactly what the interface shows a player.

The distinction between 0 and -1 is still worth keeping: both let a research
entry through, but only -1 — a caravan or aircraft limit — goes on to let a
non-caravan or a helicopter through as well.

---

## Cancelling refunds exactly what was paid

`Build::unqueue(i, refund)`.

With `refund` set — which is the player cancelling, as opposed to the item
completing — it first **walks forward while the next entry has the same type**.
Cancelling one of a run of five identical items therefore removes the fifth,
not the first, so the one in progress keeps its progress. Then:

```
job_counter = 0
num_queued[type] -= 1                 # and the category counters
if refund: unpay_cost(i)
un_queue(i, queued)                   # memcpy the tail down one slot
queued -= 1
```

`Build::unpay_cost(i)` adds each of the three stored `(good, cost)` pairs
straight back into the stockpile, skipping any whose good is -1. Not a
recomputed price — the recorded one.

Completion calls `unqueue(i, 0)`, so it neither refunds nor skips forward.

Two consequences. **A discount that arrives while an item is queued cannot be
harvested by cancelling**, because the refund is the number that was paid.
And a cost with four or more non-zero resources would be partly non-refundable,
by the three-pair limit above.

---

## Science re-prices the queue in place

`Build::refund_cost(i)` has exactly one caller: `Leader::gain_tech`, and a
narrow one. When the tech just gained is an **epoch** whose `techtype` category
is 3 — the science line — and the game is past frame 0 and the player has a
first library, `gain_tech` walks **that library's queue** and re-prices every
entry that is a tech type other than the one just gained, where it sits,
handing the difference back. (An earlier draft said "every queued item"; unit
entries in a barracks are never re-priced, and `refund_cost` reads the tech
type's age at `+0x1c8`, so it is only meaningful for a tech entry anyway.)

For each of the three stored pairs, with `d` the number of science levels
above the tech's own level:

```
base = cost * 100 / (100 - TECH_SCIENCE_DISCOUNT * d)     # undo the old discount
new  = base - (d + 1) * TECH_SCIENCE_DISCOUNT * base / 100  # apply one more
stockpile += cost - new
cost = new                                                # remember the new price
```

`TECH_SCIENCE_DISCOUNT` is 10 percent. The entry's stored `cost` is updated in
place, so a later cancellation refunds the *new* price, and a second science
level re-prices from there.

This is the sharper half of the answer to `docs/COSTS.md`'s question about
discounts arriving mid-queue. Cancelling cannot profit from one. Sitting still
can, and does, automatically.

---

## Four scales in one mechanic

`docs/DECISIONS.md` entry 14 says a constant's scale is a fact about the one
line of the loader that reads it, established one constant at a time. This
mechanic is the strongest evidence for that rule so far, because it uses four
different conventions at once and the file does not distinguish them.

| constant | loader | `1/1` arrives as |
| --- | --- | --- |
| `ACCEL_TRAIN`, `ACCEL_CONSTRUCT`, `ACCEL_RESEARCH`, `UNIT_RATE_BASE`, `UNIT_RATE_PROGRESSION` | `get_fraction(name, 100)` | 100 |
| `RESEARCH_PREMIUM`, `RESEARCH_TICK_PREMIUM` | `get_fraction(name, 0x100)` | 256 |
| `TECH_SCIENCE_DISCOUNT`, `TECH_SCIENCE_SPEEDUP` | `get_item` | as written |
| `JOB_EXTRA_TIME` (per unit record) | parsed by hand | 100 |

The last is not in `Constants::init` at all and does not go through
`String::fraction` either. The unit-type loader reads the text, runs `_wtoi` up
to the `/`, runs `_wtoi` after it, treats a missing slash as a denominator of
1, and computes `(numerator * 100) / denominator` inline. `1/10tsx` becomes 10.
`RESEARCH_PREMIUM_TIME`, two lines away in the same loader and two integers
apart in the same struct, goes through `String::fraction(s, 0x100)` and becomes
512 for the `2` that 356 records carry.

Nothing about how a value is *written* predicts any of this. Only the consumer
does.

`sim::tuning::Slot` gains a `Ratio100` variant for the first row, and adding it
turned up a bug in how the existing one was checked. The drift check used to
rescale through `Fx`, dividing a Q16.16 value by 256 to reach 8.8. That is
exact for `3/2` and for every other denominator `Ratio256` happens to use, and
it is not exact in general: `6/5` is 78643 raw, `78643 * 100 / 65536` is 119,
and the original computes `6 * 100 / 5` as 120. A fifth is not a dyadic
rational, so no binary fixed-point representation carries it.

`Scalar::fraction(scale)` now reproduces `String::fraction` directly —
`num * scale / den`, denominator one when the value is written without a
slash — and both slots go through it. The check was one constant away from
reporting a false drift for twenty years' worth of tuning, and it reported one
the first time it was asked a question with a five in the denominator.

---

## The record, diffed (2026-08-30)

Until this date every claim in this document rested on a reading plus one
hand-transcribed frame. The dump carries the whole thing —
`BuildQueue::log_data` writes `queue_size` and then all `queue_size` slots,
each as `type`, `job_counter`, `cost[0..2]`, `good[0..2]`, under a
`BUILDQUEUE` block inside `BUILDDATA`, with `BuildData::queued` beside it —
and nothing parsed it.

It is parsed now (`rondata::gamelog::QueueItemDump`) and diffed whole:
`run39_s_build_queues_are_the_original_s_clock` compares every building of
both players on every one of run39's 1,851 frames, `queued` and then the
first `queued` slots' every field. **33,631 fields, of which 21 disagreed** —
four times further than the sync word reaches on that map. So the clock (the
accelerator, the compare-before-add, the cap at the target), the charge, the
per-entry price ramp and the handover are the original's on the record, not
merely on the reading.

**The 21 were the science speedup, and are now 1.** They were twenty
consecutive frames of `1/2005: queued ours 1 theirs 0` — the AI's library,
holding an entry the original had already finished. The AI queues Written
Word and City State at frame 2 and both are `JOB_TIME 200`, so both start at
20,000 hundredths. Written Word is itself the Science epoch, so while it is
being researched `epoch[3]` is zero against its own level of zero, the gate
`level < epoch[3]` is false, and it takes the full 20,000 and lands on 201.
From 202 the player is a Science level ahead of City State's own zero, so
City State's target is 18,000 and it lands on **382**. This simulation
applied no speedup, charged the second entry 20,000 too, and emptied the
queue on 403.

That is what makes the speedup diff-backed rather than read: the twenty
frames are not a coincidence of magnitudes, they are exactly ten per cent of
one `JOB_TIME 200` entry, on the frame the level arrives, in a record that
pins the counter to the hundredth. The one field still disagreeing is on
frame 1851 — the capture's last — at `1/2000`, and is unrelated.

The floor is pinned in `rondata::diff`: `first >= 1851`, `wrong.len() <= 1`.

### On every capture (2026-09-01)

The loop above lived inside its own test for two days, so the record was
compared against **one** dump — run39's, which is 1,851 frames. It is
`diff::compare`'s now, which means every capture the harness reads gets the
queue, and the ledger `docs/QUEUE.md` item 87 asks for should count captures
as well as fields.

The first thing that bought was the headline. run58 is the same game at
5,200 frames and had never had its queues looked at: **109,435 fields**, of
which the earliest wrong one is the AI Dock `1/2010`'s `job_counter` on
frame 4461 — ours 8,500 where the original caps at 8,481, followed by
twenty-eight frames of `queued ours 1 theirs 0`. Everything before it
agrees, hundredth for hundredth, from the frame the job is queued. That is
what says the divergence is the **target** and not the decision or the
accelerator, and it is the whole of what "diff the whole record" is for:
nobody had to guess which of the three it was.

The assertion is scoped to before the word, as run58's other two are —
past it the two streams are running on draws that are nobody's, and the
238 fields that disagree there are all `1/2010`'s and all downstream of
`Unit::think_fish`.

## The infinite queue (item 877)

`WallData::build_masks & 0x40`, the player's repeat-production button
(`Options::exec@007188c0`'s option 0x40 on a selection of buildings →
`GroupOut::issue_buildmask@00708820` → `CommandManager::issue_buildmask@
00941f80` → `CommandPackage::process_buildmask@00947680` →
`Group::action_buildmask@006fc9a0`). `docs/GOLDEN.md` §33 is the chapter
and run285 its capture; every clause below is diff-backed there unless it
says otherwise.

**The gate.** `WallData::valid_buildmask@0063e2a0` admits 0x40 on a
member whose `BuildData::can_infinite@0062d4d0` answers: the building is a
training building (`BuildTypeData::is_training_building`, `build_flags &
0x80000000` on the root type) and **a train job is queued there**, an
entry whose type is a unit type with its availability bit set. A research
entry, a technology and an empty queue do not admit it (under the
emulator; run285's second press, on an empty queue, leaves 4096).
`action_buildmask` then **toggles** the bit off the first admitted
member's state, whatever the command's `set` (`docs/GOLDEN.md` §32).

**The re-queue.** In `Build::do_queue@0061e410`, once `finished` answers
> 0 for slot `i`:

```
was_train = the entry was a unit type with its bit set before `finished`
word      = build_masks                      # 61ec24, before the unqueue
unqueue(i, 0)                                 # clears 0x40 if queued -> 0
if was_train and word & 0x40:
    build_masks &= ~0x40
    if queue_up(type, 0) == 0:                # paid; appended at the end
        build_masks |= 0x40
```

So the finished unit's type goes to the **end** of the queue at its
current price, the bit survives the queue emptying under it, and a
re-queue the stockpile refuses leaves the bit off with the queue as it
is. run285: the Bowmen out on 1060 and re-queued at 0, 46 timber and 56
wealth, 4160; out again on 1272, refused on 19 wealth, the queue empty and
4096. A research entry's completion (`gain_tech`) never re-queues.

**The other clears**: `Build::clean_queue@00620b60` (a closed, captured
or defeated building's queue) clears the bit when it empties, reading-only;
`Build::action_unqueue@00620280`, the player's cancel, clears it first,
and for a single cancel returns without removing the entry — the first
cancel on an infinite queue only turns it off. **Diff-backed by run292**
(item 884, "The player's cancel" below): 4096 and the entry kept on 842. No AI function reads or writes the bit, and no dump before
run285 holds it on any building.

**The command that fills the queue** is `CommandPackage::process_queue_up@
00948230` → `Group::action_queue_up@006fdbb0(type, num)`: the members are
sorted by `queued`, least first, then `num` times over each live, finished
member gets `Build::queue_up(type, 1)`, whose answer is not read (a missile
silo asks `can_carry(type)` first). Diff-backed on two Barracks by run304 ("The command on a selection of
buildings" below). The research arm — a unit type whose
bit is clear, or a technology — goes through `LeaderData::researching` and
one building with an empty queue first; ~~it is read and not built~~ built
and staged in "The player's research" below (item 883).

**In the code**: `production::Queue::infinite` is the bit, and
`Queue::unqueue` clears it on the empty queue; `Sim::can_infinite`,
`Sim::requeue_infinite` and `Sim::action_queue_up` are in
`crates/sim/src/production.rs`; the 0x40 arm of `Sim::action_buildmask`
is in `air.rs`; `do_queue`'s read-before-unqueue is `Sim::advance_slot`'s
`Handover::Trained` arm. The harness compares the bit beside 0x80.

## The player's cancel (item 884)

`CommandPackage::process_unqueue@009466f0` → `Build::action_unqueue@
00620280(p)`. The command is `CommandManager::issue_unqueue@00942c40(b,
p)`, 15 bytes `[who][o][p][uid]` and **no `group`**, so it seats nothing
in the pool (`docs/COMMANDS.md` §3). `docs/GOLDEN.md` §34 is the chapter;
every clause below is read off the listing and run under the emulator, and
says when a capture backs it. **run292 backs every arm but −5 and −10**:
a slot inside a run (702), a slot across two types (762), a single cancel
on an infinite queue (842), and −1 on a two-type queue (1002), each on
its block with the refund of the recorded pairs, and no pool seat on any.

```
forward to the first library, as `unqueue` does, unless slot p is DISBAND
if queued == 0: return                         # the bit too is left
if build_masks & 0x40:
    build_masks &= ~0x40                       # and the console's feedback
    if p >= -1: return                         # a single cancel: nothing removed
if p <= -10:   while queued: unqueue(queued - 1, 1)
elif p <= -5:  repeat min(queued, 5): unqueue(queued - 1, 1)
else:          unqueue(p if p >= 0 else queued - 1, 1)
```

The interface's `p` is `Options::exec@007188c0`'s option 0xa6 `object`:
a slot, or −5 and −10 (`~PAPYRUS`) under its two modifiers.
`unqueue(i, 1)` is "Cancelling refunds exactly what was paid" above: the
walk over the run, the recorded pairs refunded, `num_queued` and the AI
tallies taken down (neither below 0), `queued` less one and the tail
copied down; a slot past the end is nothing.

**In the code**: `Sim::action_unqueue` in `crates/sim/src/production.rs`
over `Sim::cancel`; `input::unqueue` is the command's entry, and the
recorded stream's `Unqueue` goes through it (no kept stream carries one).

## The player's research (item 883)

`CommandPackage::process_queue_up@00948230` → `Group::action_queue_up@
006fdbb0(type, num)` for a type that is not a train job: a technology, a
unit type whose availability bit is clear, or a building type with
`build_flags & 4`. `docs/GOLDEN.md` §35 is the chapter; the arm below is
read off the listing and run under the emulator with `Build::queue_up`
stubbed. **run296 backs it** on who=0's Library: one entry on an idle
Library (622), a second press refused by `researching` (642), the second
pass behind a busy head (652), a held technology refused by `can_make`
(852), the Science re-price and the cancel of the re-priced entry (1023,
1042), and the same technology queued again at the science discount
(1062). ~~The `num` clause and a two-member group are the emulator's alone.~~
**run300 backs the unit arm** (item 901, `docs/GOLDEN.md` §36) on who=0's
Barracks: Javelineers laid once at 80/80 with `num` 2 (622), and a second
press refused (642). A unit research is not refused by
`BuildData::can_make@0062db10`, which asks `researching` for a technology
only, and the price would have paid for a second entry. So both blocks
split their readings, and **the `num` clause is diff-backed** there. A
two-member group is still the emulator's alone. The finish's queue loop is
`docs/TECH.md` "The queue loop".

```
sort the members by queued, least first          # as the train arm
if LeaderData::researching(type, -1, 0, 0):      # 6fde39
    the console's player hears add_feedback; return
for pass in 0, 1:                                # num's slot, 6fe08a
    for each member, alive (+0xc) and finished (+0x4c):
        if pass == 0 and member.queued != 0: continue      # 6fe135
        if Build::queue_up(type, 1) == 0: return           # 6fe16b
```

So a research lands **once**, whatever `num` says (the train arm lays
`num` per member), on an idle member if one takes it and else on the
least-queued that does; a refusal goes on to the next member, and a
technology forwards to the first Library inside `queue_up` as every
research does. `researching` walks the player's buildings from 2000, each
whose `+0x4c` answers, and each entry of its queue: the type itself, or —
for a unit type, `param_4` 0 — an entry of a unit type `u` with neither
bit set and `type.is(u, 0)`. `queue_up`'s `can_make` then refuses a
technology already held (`has_tech`, the bit) or researching; the price
is asked before it (`can_pay_cost` at `vslot 0x84`, then `could_queue`).

**In the code**: `Sim::action_queue_research` (a technology) and
`Sim::action_queue_up`'s research arm (a unit type whose bit is clear),
over `Sim::research_arm` and `Sim::researching_unit`, in
`crates/sim/src/production.rs`; `Sim::queue_tech` carries `can_make`'s
held and researching refusals; `input::group_queue_up` maps a
technology's `TypeIndex`. A building type's arm is not mapped.

## The command on a selection of buildings (item 888)

A player's command on two or more buildings — a unit's button, the
infinite-queue button — reaches every selected building through one
`Group`. `docs/GOLDEN.md` §37 is the chapter and **run304 backs every
clause below** on two Barracks; the listing and the emulator came first.

```
process_group:  add each listed live object (Group::add; a repeat dropped),
                push_group(who, g, 1) once          # 0x94a6cf
action_queue_up(type, num), a train job:
    sort the members by queued (+0x82, unsigned), least first:
        for i < n-1, a pivot alive and finished:
            for j > i: swap if queued[j] < queued[pivot]   # 6fdd46, jbe
    repeat num times:                                        # 6fdf20
        for each member, alive (+8&1), finished (+8&4):
            queue_up(type, 1)            # the answer is not read, 6fe056
action_buildmask(mask):
    all_set = 1
    for each admitted member: set if it lacks the bit and all_set,
                              else clear it and all_set = 0
```

So `num` is laid **one entry a member a pass**, from the least-queued
member, and every entry raises the next one's price: run304's `num` 3 on
`[2007 (1 queued), 2008 (0)]` laid 53/41 at 2008, 56/45 at 2007, 60/50 at
2008 and 65/56 at 2007, the fifth refused on 9 timber (642). A member at
the sort's slot that is not finished is not a pivot but can be swapped
forward, and is never called (the emulator alone: no cheat leaves a
building unfinished). The toggle on `[on, off]` clears both (722).

**The pool.** The command's group is one record, `buildings 1`, listing
the buildings **in the command's order**, `speed` 0, and nothing is
written on a building. `Group::equals_group@00708000` compares the member
lists in order against `last_group`'s record: an equal group seats nothing
and keeps its stamp (762); the same two in the other order take a new
slot (742). A building-group slot is open to the next push
(`Groups::get_open_slot@006fa460`), so the squads trained after took
slots 0 and 1 from the two-member records (825, 876).
`CommandPackage::add_group@0094bb60`'s own reuse test is an ordered
compare too: 760's press sent the three-byte reuse.

**In the code**: `Sim::action_queue_up`'s sort (`by_queued`) and passes
and `Sim::action_buildmask`'s rule were already the reading's;
`Sim::push_command_buildings` (`garrison.rs`) and
`Sim::push_buildings_group` (`group.rs`) seat the group, and
`push_building_group`, the AI's one-building push, goes through it.

## The trained aircraft: an Airbase keeps it (item 915)

`Build::train@0062f9b0` asks the **trainer's** type for `CARRY_AIR`
(`obj_masks` `J`, `+0x1e4 & 0x200`, at `62fac0`) before any exit, and that
arm calls no `Unit::come_out@00617c10`. `J` is on two building types, the
Airbase and the Missile Silo (`buildingrules.xml`, `GJ` both), which is
this crate's `Sim::is_hangar`. `docs/GOLDEN.md` §38 is the chapter and
**run308 backs it**: a Biplane trained at `0/2007` on 1746 is inside the
base, behind `0/8` in its chain (`inside_up 8`), with no order and
`mana_burn` 0, and stays so to 2069; `do_launch` passes it over on every
block.

```
train(type):   init_unit at the building's point; go_inside(this)
    if trainer's type & CARRY_AIR:                       # 62fac0
        options.rebuild = 1
        gather points (+0xcc) == 0:                      # 62fadf
            if unit.is(HELICOPTER 0x136)
               and num_aircraft_here > num_aircraft_limit: come_out(0)
            (else: nothing -- the unit stays inside)
        else, the unit not a missile and not unit_flags & 0x20:
            each gather point: add_air_patrol_order(point, this, who, 1)
                               (or a waypoint on the order it has)
        else: the first point -- a strike on an enemy building there,
              or a patrol over it
    else: the ordinary exit (gather_inside, the scholar's arm, come_out)
    unit_masks &= ~0x4000000                             # 62fc17
```

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`,
the unit and type through stub vtables): with `CARRY_AIR` and no gather
point, `init_unit`, `go_inside(2007)`, `is(0x136, 0)` and no `come_out`;
with one gather point, `add_air_patrol_order(x, y, 2007, 0, 1)` — the
action bit — and no `come_out`; without `CARRY_AIR`, `come_out(0)`.

**The gather point's arm is not built** (SEAM): its writers are
`Build::add_gather_point` and `Build::clear_gather`, reached from the
player's `issue_gather_point`, `Options::do_clear_gather`,
`Group::action_city_gather` and the scenario functions. ~~No capture issues
one, so every building on disk has none.~~ run312 issues five, on two
Barracks and a City (§ "The gather point", below), and none at an Airbase,
so this arm is still unreached. Neither is the helicopter's
over-the-limit arm.

**What stands**: the Biplane's birth point, (11640, 13944) there against
the base's own (11616, 13920) here — `Unit::init@00612100`'s seat, parked
646, and the plane never leaves the base to read it.

**In the code**: `Sim::build_train` (`lib.rs`), the hangar test before the
exit. Test: `air::launch_tests::an_aircraft_trained_at_an_airbase_stays_inside_with_no_order`,
made to fail with the arm dropped, which also re-parts run308 on 1745.

## The gather point (item 928)

The player's rally point. `docs/GOLDEN.md` §39 is the chapter, and
**run312 backs every arm below that says so**. The emulator's table is
there, and the capture's in `docs/RUNS.md`.

**The command.** `CommandManager::issue_gather_point@00941b20` appends
`16 [x][y][action][add_to_end]` behind a group of buildings.
`CommandPackage::process_gather_point@00948510` hands it to
`Group::action_gather_point@006ff1b0`. The action is 0 on the ground, 1
on a friendly object's point, 2 an enemy's, and 3 an object by
`(o, who)`. The Clear button is −1, −1, 0, 0. A member is live and
neither a University nor a Missile Silo. It takes the point when it
trains (`build_flags & 0x80000000`), has a garrison limit, or is a
Terracotta Army, Kremlin or Senate.

```
action_gather_point(x, y, action, add):
    x < 0 or y < 0: clear_gather on every member; done
    clamp (x, y) to the world
    add &= no member's head point is "inside"
    every member a City centre (COUNT_TYPE VILLAGE), action != 3:
        forest tile (& 0x30 == 0x30) -> the nearest friendly Woodcutter
        mountain (& 3 == 2)          -> the nearest friendly Mine
        within 0x600: (x, y) = that building's point     # action kept
    building tile (& 3 == 3), !add, a member covers it:
        Terracotta/Kremlin/Senate: clear_gather(first member); done
        else inside = true
    each member (not an Airbase under action 3):
        add_gather_point(inside ? (-1, -1, 0), NEW : (x, y, action), add ? LAST : NEW)
```

`Build::add_gather_point@00622e70` clears the list under `QUEUE_NEW` and
appends; `Build::clear_gather@00623180` empties it. The list is
`BuildData +0xb8`, with the count at `+0xc8`, and the dump prints it at
`BUILDS=7` as a second `length`, then `type`, `metric` and a
`GATHERPOINT` per point. **run312**: 2007 (1344, 12096, 0) on 618; the
City's forest click (3936, 28128) stored as its Woodcutter's (4224,
28608) on 652; 2008's click on itself (−1, −1, 0) on 702; 2007's point
replaced by (4224, 14208, 1) on 902 and emptied by the Clear on 1102.

**The trained unit.** `Build::train@0062f9b0`'s ordinary arm first asks
`BuildData::gather_inside@0046f180`, whether the head point is "inside".
If it is, the unit has room (`num_inside ≤ get_garrison_limit`, 10 when
the limit is 0), and it is not an Aircraft Carrier, it **stays in**. Then
`Unit::come_out@00617c10`:

- **the gather block** (`6181a3`..`618377`): for a captain whose host has
  points, not "inside", `UnitType::find_nearby_spot(head point, 0,
  0x600, 0x55555555)` finds the spot (`FILTER_NOT_ME` for a type with a
  `block_radius`). `find_angle(spot − building)` is stored in the unit's
  `angle` (`+0x50`) and in the exit sweep's bias, in place of south;
- **the routing** (`618b22`..`619fe2`), for a captain with no order,
  after the members' exits and 882's push of the squad. With one point,
  the last:
  - a building at the point (`find_any_building_at`, `FILTER_SEEN`):
    - a citizen builds it (its own, unfinished), repairs it (damaged), or
      gathers there (a gather building of its own, not a University,
      `action` ≠ 0);
    - then a friendly building with room the type can garrison, `action`
      ≠ 0, takes a `GARRISONORDER` down the squad (`QUEUE_LAST`);
  - otherwise a move: `ATTACK_TO` for an armed type (`+0x1e8`) whose
    stance is not 5 and that is made at a Barracks, Stable or Dock
    (`TypeData::where`, `+0x40`); `MOVE_TO` otherwise. A squad with a
    group goes through `Group::action_move_to(the free spot nearest the
    gather spot, QUEUE_LAST, set_angle, find_angle(target − exit),
    action)`, and a lone unit through `add_move_facing_order(the gather
    spot, …, QUEUE_LAST, action)`.

A member's own angle is `Unit::set_angle(host->angle)`, whose `reversing`
test flips its mirror past 90°. The captain's is the bare store, which
flips nothing.

**The two re-seats** (item 945). Before it orders anything, the routing
may sweep the captain round its trainer's **land** exit ring again
(`UNIT_TRAIN_DISTANCE` to `UNIT_TRAIN_MAX_DISTANCE`, step 0, no boat arm,
no dying-building zero) under `FILTER_ALL`, and
`Unit::set_new_location@005f8d20(·, spot, 1, 1)` moves it there. The
members stay where the exit seated them.

- **At a building point** (`61923e`..`619381`): when `find_any_building_at`
  finds a building at the last point, whatever it goes on to order. The
  bias is `find_angle(that building − trainer)`, the register pair at
  `6192e4`/`61930b`, and **not** the gather block's bearing to the free
  spot beside it.
- **In the lone move arm** (`619ea2`..`619f86`): the bias is
  `find_angle(the gather spot − trainer)` (`619ee5`/`619efa`). That is
  the exit's own bearing, so a lone unit the first re-seat moved is put
  back.

`FILTER_ALL` takes `find_nearby_spot`'s general path
(`ObjectsData::find_unit_with_radius@00659890` and its ordered twin,
`docs/COLLISION.md` §5.2.1), `orders::Coll::All` here. Its seeker is
exempted **as an untested assumption**: no capture's winning candidate
is within reach of where the unit stands.

**run312**, on the units they move:
- the Bowmen `0/17` (point on 2008): the exit's bearing is 0x4a590000,
  and the exit puts the captain at (3336, 14376), where its members are
  seated. The re-seat's bearing is due east, and it puts the captain at
  (3384, 14232) on 1060, as the original does;
- the Citizen `0/10` (point on the Woodcutter): moved to (3576, 29928),
  then put back at (3576, 29976) on 760.

The reading 928 booked was that the original refuses the whole first
ring. It is dead: under the agreed bias, theirs is the sweep's 61st
candidate, on open ground.

**run312**, each on its birth block:
- the Hoplites `0/11`..`0/13` out north-west on 856, each a
  `GROUPATTACKTOORDER` under one group to (1368, 12120);
- 2008's Hoplites kept inside from 953;
- the Bowmen `0/17`..`0/19` out east on 1060, each a `GARRISONORDER` on
  2008, inside on 1079;
- the Citizen `0/10` out north-east on 760 with a plain `MOVEORDER` to
  the Woodcutter's spot (4248, 28632) and no gather order: the snapped
  point keeps action 0. It gathers of its own accord from 984.

**In the code**: `crate::rally` (`Sim::action_gather_point`,
`add_gather_point`, `clear_gather`, `gather_inside`, `gather_exit`,
`gather_route`, `gather_reseat`), with `Building::gather`; `Sim::come_out` and
`come_out_place` (`garrison.rs`); `Sim::build_train`'s "inside" arm
(`lib.rs`). The entry is `rondata::input::group_gather_point`, and
`input::Stream` applies a recorded one (run7's, 1184). Seven tests in
`cities_tests`, each made to fail by a mutation of its arm.

**What stands on run312**:
- ~~the Bowmen's exit point~~: the building point's re-seat, above
  (item 945);
- the group move's id (parked 676's first) and `Unit::init`'s seat and
  `form` (parked 646).

**Not established, and reading only**:
- a list of more than one point: the waypoints before the last, and
  whether `gather_inside` and `come_out` read the head or the tail after
  a `QUEUE_LAST` (`add_gather_point` moves `+0xcc` back to the old tail);
- action 3 (the Airbase's strike, `action_flight`) and the Airbase's arm
  of `add_gather_point` / `clear_gather` under `build_masks & 8`;
- an enemy at the point (the attack arms), a caravan's trade arm, and
  the third re-seat (`619aa3`..`619be7`: a unit found at a ground point),
  ~~with the lone arm's second sweep~~ (built, item 945);
- whether `FILTER_ALL` exempts the seeker: a lone unit trained under a
  ground point is the capture that tests it;
- a citizen's build, repair and gather arms (read and built, not
  captured);
- the Terracotta Army and the Kremlin beside the Senate (wonders with no
  ident here), and `find_building`'s own metric (the nearest here).

## What is not established

- **Nine of `train_time`'s ten national arms**, and everything after them.
  The British arm is built and diff-backed (see "The tail's first caller");
  the handicap, The President, the Mongol stable, the Japanese barracks and
  carrier, the Chinese citizen, the French siege and special, the German air
  and submarine and the Roman legion are read here and implemented nowhere,
  as are `TROOPS_FASTER`, the ship/troop/vehicle speed-upgrade counts, the
  rares, Monarchy, Socialism, the unit wonders, the Kremlin and
  `INSTANT_UNIT_BONUS`. Each is inert in every capture on disk. The Chinese
  arm is the one whose *predicate* is not settled: the decompiler prints its
  final test as `extraout_ECX[0xae] & 8` on a pointer it lost, and the
  listing is what would settle which flag of which record that is.

- ~~**`BuildQueue::init` and where `queue_size` comes from.**~~ Closed by the
  second reading: `Build::init` chooses 20, 10 or 2 from the building type's
  flags (see "Queueing charges the price"), and `BuildQueue::init` accepts
  only those. `crates/sim/src/production.rs` still takes the capacity as an
  input, because the flags are a building-type system the simulation lacks.
- ~~**`BuildData::can_make`**, the first gate in `could_queue`.~~ Read by the
  second reading: active; unless the scenario-editor semaphore or
  `DISBAND`/`DEPOPULATE`, not neutralized and not unassimilated;
  `BuildTypeData::queue_here(type)`; `type_avail(type, p) != 0`; a tech is
  refused if already held or already researching; a unit with the bit set and
  `build_flags & 0x10` clear returns `type_avail` itself. `queue_here` and
  `type_eligible` are the tech tree's — **now read, in `docs/TECH.md`**: a
  building queues what its lineage (`is(where, 0)`) makes, and eligibility is
  the tribe mask, the ending age, obsolescence and the jump rule.
- ~~**What sets the availability bit at `leader + 0x6c18`.**~~ It is the
  `tech` bitmask — `BitMask<806>` at `LeaderData + 0x6c0c`, whose pointer
  sits at `+0x6c18` — and `Leader::gain_tech` sets it, for a unit type exactly
  as for a technology (see "Completion"). `LeaderData::type_avail` reports 2
  for a type whose prerequisites are met but whose bit is clear and 4 once it
  is set.
- ~~**`game.semaphore` bit 3**, which makes everything free at queue time.~~
  It is bit 11 of the `BitMask<256>` (byte 1, bit 3 — same predicate, the
  earlier draft named it by the byte), set by `ConsoleWin::run_cmd` on
  entering the **scenario editor** and cleared on leaving. It also skips
  `can_make`'s neutralized/unassimilated gate and makes `can_pay_cost` return
  10.
- ~~**`BuildData::construct_time`.**~~ Closed by `docs/CITIES.md` §3.2: the
  stored base at `+0x50` is written once, at placement, by
  `Wall::update_construct_time` (`job_time × 100` through the nation, wonder
  and speed-tech factors and the nomad's first-city ×3), and the four per-call
  clauses are the Americans' free first wonder, the Hanging Gardens, the
  President and the Iroquois' first senate. It is the foundation clock that
  `Wall::do_construct` compares `job_counter` against; implemented in
  `crates/sim/src/build.rs`.
- ~~**The build-time analogue of the price ramp's four ceilings.**~~ There is
  exactly one cap in `train_time`, the `×3`, and the second reading found no
  other. Closed.
- **Whether any two of the tail's thirty modifiers can apply at once in an
  order that matters.** The full order is now recorded (audit §3 O7) and the
  research/train partition is stated above; which pairs can coincide is a
  balance question, not a fidelity one.
- ~~**`TECH_SCIENCE_SPEEDUP` is loaded and never applied.**~~ Applied, both
  sides, 2026-08-30. The **time** side is `ObjectData::train_time`'s own
  inline step and is enumerated under "Time" above; it is
  `crates/sim/src/production.rs`'s `science_speedup`, reached from
  `Sim::tech_time` for a technology and from `Sim::queue_target` for a unit
  research job, and it is **diff-backed** — see "The record, diffed". The
  **price** side is `LeaderData::calc_science_discount@006da630`, which
  returns `value − ahead × TECH_SCIENCE_DISCOUNT × value / 100` with `ahead`
  the decrypted `epoch[3]` less the tech's own level — the `AGE` column plus
  one unless the type is an age (`0x220..0x226`) or an epoch
  (`0x227..0x242`); it is `crates/sim/src/cost.rs`'s `science_discount`,
  reached from `Sim::tech_price` through `Modifiers::science_ahead`, and it
  is **reading-only**: no traced capture queues anything but the two epochs
  `0x227` and `0x235`, for which `ahead` is zero. `docs/COSTS.md`
  §"The discounts" carries the price side's own record.
- **The AI counters** at `leader + 0xa10` through `+0xa24`, moved by both
  `queue_up` and `unqueue`. They are per-production-building tallies and
  nothing in the simulation reads them yet.
- **Not modelled, by choice, and recorded so nobody looks for it:** the
  caravan-limit and aircraft-pad refusals (`finished` returning -1) and the
  `get_next_non_caravan` / `get_next_helicopter` fallback that follows them,
  because the simulation has neither caravans nor aircraft; the missile-silo
  gate in `do_queue`; ~~the infinite-queue flag~~ (built, "The infinite
  queue"); the University's six-scholar
  rule; the library quirk that removes an entry on a negative answer. Each is
  named where it belongs above and each is a small addition once the thing it
  depends on exists.
- **Which building is a library** is, in the simulation, a flag on the
  building (`Building::is_library`) set by `Sim::add_library`, and "the first
  library" is the lowest-indexed such building the player owns — the same
  order `get_first_library` walks. That is the smallest honest stand-in for a
  building-type system, and it is what `docs/PRODUCTION.md`'s two library
  rules — forwarding and the parallel fan-out — are keyed on.

---

## Second reading (2026-08-20) — landed

A blind second derivation and its adjudication are in
`docs/audit/2026-08-20-production.md`. The queue record, the 3-of-6 pair
recording, charge-on-queue, every loader scale, the accelerator choice,
done-before-increment, the base and research-time shifts, the ramp on
`num_units` only, the age/epoch catch-up, `finished`'s sign semantics and the
cancel walk with exact refund are **doubly confirmed**. Seven disagreements,
six resolved against the earlier draft of this document, all landed above and
in `crates/sim` (`production.rs`, `lib.rs`, `harness_tests.rs`) on the same
day; each place that changed says so inline. The three that changed observable
behaviour: the parallel-slot fan-out belongs to the first library and to no
other building; a blocked head lets the first research entry behind it
advance; and the first of a unit type completes through `gain_tech` and trains
nothing. The one point resolved for the earlier draft: frames-to-complete as
written here (the second reader's `ceil(T/a)` is off by one). Every open
question the second reading closed is struck above with its answer.
