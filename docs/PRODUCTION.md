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
are enumerated rather than each separately derived. What remains open is listed
at the end.

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
  above the tech's own level.
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
one to a research job.** No caller builds a tail yet; when one does, this is
its first rule. The full order, spot-checked against the decompile, is in
`docs/audit/2026-08-20-production.md` §3 O7.

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
garrisoned and leaves by the ejection path. Stance and gather-point orders are
applied on the way out. `finished` returns 1 whether or not `init_unit`
succeeded, so a placement failure still removes the entry.

The caller reads the sign, and what it does with it differs between the
library and everything else.

**At a non-library building:**

- **positive** — `Build::unqueue(i, 0)` removes the entry, with no refund. If
  the entry was a train job and the building's `build_masks & 0x40` — the
  **infinite-queue** flag — is set, the bit is cleared, `queue_up(type, 0)` is
  tried, and the bit is set back on success; `unqueue` clears it when the
  queue empties. Not modelled.
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
first `queued` slots' every field. **33,631 fields, of which 21 disagree** —
four times further than the sync word reaches on that map. So the clock (the
accelerator, the compare-before-add, the cap at the target), the charge, the
per-entry price ramp and the handover are the original's on the record, not
merely on the reading.

The 21 are one residue, below.

## What is not established

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
- **`TECH_SCIENCE_SPEEDUP` is loaded and never applied.** `Tuning` carries it
  at 10 and nothing reads it. `LeaderData::calc_science_discount@006da630`
  returns `value − levels × TECH_SCIENCE_DISCOUNT × value / 100`, where
  `levels` is the decrypted `epoch[3]` less the tech's own level — plus one
  unless the type is an age (`0x220..0x226`) or an epoch (`0x227..0x242`) —
  and it is applied to a research job's **time** as well as its price. The
  diff above measures it exactly: run39's AI library takes 20,000 hundredths
  where the original takes 18,000, so its research lands on frame 402 rather
  than 382, and those twenty frames are the whole of the 21. Its purchase-side
  twin — `Sim::tech_price` passing `Modifiers::default()` — is the same
  function's other caller, and the two belong in one session
  (`docs/QUEUE.md` item 86). `crates/sim/src/cost.rs`'s `reprice` is the
  *refund* side and is already right.
- **The AI counters** at `leader + 0xa10` through `+0xa24`, moved by both
  `queue_up` and `unqueue`. They are per-production-building tallies and
  nothing in the simulation reads them yet.
- **Not modelled, by choice, and recorded so nobody looks for it:** the
  caravan-limit and aircraft-pad refusals (`finished` returning -1) and the
  `get_next_non_caravan` / `get_next_helicopter` fallback that follows them,
  because the simulation has neither caravans nor aircraft; the missile-silo
  gate in `do_queue`; the infinite-queue flag; the University's six-scholar
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
