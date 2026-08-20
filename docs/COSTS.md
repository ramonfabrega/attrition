# Costs: what a thing is worth, and what stops you paying

`docs/ECONOMY.md` is how a resource arrives. This is where it goes.

Three things decide what one unit costs: a base price written in the data and
multiplied by ten, a **ramp** that raises the price of everything you already
have a lot of, and a **redirect** that charges an entirely different resource
for anything the age has not made available yet. Then a long tail of national,
wonder, government and rare-resource discounts, all of the same shape, and
finally the two questions a caller actually asks — can I pay, and is there
room in the population.

Not in it: production queues and build times, the market, tribute, and the
tech tree's prerequisites. `JOB_TIME` and `Build::queue_up` are the mechanic
that spends this price over time; they are surveyed at the end and specified
elsewhere.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the spine — the cost factors, the ramp in all four of
its shapes, the four ramp ceilings, the unavailable-good redirect, the payment
and refund arithmetic, and the population cap are each read end to end at their
consumers, and two of them are confirmed against the designers' own column
comments in `unitrules.xml`. High for the *shape* of the discount tail, which
is one expression repeated forty times; the individual predicates behind each
are enumerated rather than each separately derived. **Lower for the research
path** — what an *upgrade* costs, as opposed to a unit — where the refit
surcharge is read but the loop that drives it is only partly understood. What
remains open is listed at the end.

**Where the implementation is.** `crates/sim/src/cost.rs`. Every constant below
is re-read from the user's own install by `cargo run -p rondata -- <install>`,
which fails if any has drifted.

---

## "Support" means cost

Before anything else, one word has to be taken away from the reader.

Every unit record in `unitrules.xml` carries a `SUPPORT` field written in the
same grammar as `COST` — `1f support`, `20m/20g support`, `40t/40f support`.
Every instinct an RTS player has says that is upkeep. `docs/ECONOMY.md` reached
the opposite conclusion from the income side, where `Leader::calc_support`
writes six zeros and returns, and stated the finding as an absence: there is no
unit upkeep in Rise of Nations.

The cost side says what the field is *for*, and the designers said it first, in
the column comment two lines above `COST`:

> `SUPPORT` = Ramping cost of unit

`TypeData::get_cost` reads it as the per-unit ramp, and the four
`*_SUPPORT_GOOD` / `*_SUPPORT_RATE` fields in `resourcerules.xml` — which
`docs/ECONOMY.md` correctly reported the income path never touches — are read
here, as the redirect that decides which resource a cost is charged in. In this
engine's data vocabulary **"support" means price, not upkeep**, and the two
subsystems together settle it: the income path reads none of these fields and
the cost path reads all of them.

`docs/ECONOMY.md`'s conclusion is unchanged and gets stronger. Not only is
there no upkeep — the field a reader would take for upkeep is the thing that
makes your eleventh hoplite cost double your first.

## A price is ten times what the file says

`unitrules.xml` gives the Citizen `<COST>2f</COST>`. A citizen costs twenty
food. The designers annotated the column:

> `COST` = Base cost of unit (multiply by 10)

and `Type::sum_rules_cost` does exactly that, choosing the multiplier by what
kind of type it is:

| Type | Id range | Factor | Ships as |
| --- | --- | --- | --- |
| Unit | `0x32`–`0x19d` | `UNIT_COST_FACTOR` | 10 |
| Building | `0x19e`–`0x21e` | `BUILD_COST_FACTOR` | 10 |
| Spell | `0x275`–`0x2ab` | `SPELL_COST_FACTOR` | 10 |
| Anything else, techs included | | `TECH_COST_FACTOR` | 10 |

All four ship as ten, so the distinction is invisible in a stock game and is
recorded because the ranges themselves are useful: they are how the engine
tells a unit from a building from a tech, by numeric identity, in a dozen
places. `Type::load_cost` parses the `<amount><letter>` grammar into a plain
six-integer array with no scaling of its own — `f t g k m o` to slots 0 to 5,
which is `docs/ECONOMY.md`'s resource order and `rondata`'s `Cost` type.

So the four numbers a player sees on a build button are the file's numbers
times ten, and everything below operates in those units.

## The ramp

**The count.** `LeaderData::get_support_count(type)` decides how many of a
thing you are considered to have. It is not simply how many are alive:

```
count = num_queued[type] + num_units[type]
```

plus, for nukes and missiles, the number already *used* — a spent nuke goes on
making the next one more expensive forever. The Americans subtract their free
bombers.

If the type's `PROGRESSION` has bit 0 set, the count is taken over the whole
**production group** instead of the type — barracks units, stable units,
factory units, dock units, or air units, each as `queued + built`. This is why
switching from crossbowmen to musketeers does not reset the price: they draw
on the same barracks count.

**The shape.** `PROGRESSION` is two bits, and the designers' comment enumerates
all four combinations:

| Value | Bit 1 | Bit 0 | Meaning | Ships on |
| --- | --- | --- | --- | --- |
| 0 | | | linear, by type | 128 units |
| 1 | | group | linear, by group | none |
| 2 | progressive | | triangular, by type | 4 units |
| 3 | progressive | group | triangular, by group | 232 units |

Bit 1 replaces the count with its triangular number:

```
count = (count + 1) * count / 2
```

**The term.** `SUPPORT` is stored as **two ordered `(resource, amount)`
slots**, not as a six-slot array like `COST`. `ObjectType::load_support` walks
the slash-separated pairs in written order, skips any whose amount is zero
without consuming a slot, stops after the second, and leaves the rest naming no
resource at all. Buildings reach the same two slots through their own named
`SUPPORT0`/`SUPPORTVALUE0` and `SUPPORT1`/`SUPPORTVALUE1` columns.

Two consequences follow that a six-slot array would not have. Anything written
past the second pair is silently dropped — no shipped record does. And a field
naming the same resource twice fills both slots with it, so the engine matches
both and that resource ramps twice: the militia line writes `2f/2f support` and
therefore ramps four food per militia and no metal at all, which reads as a typo
for `2f/2m` and behaves exactly as written.

The ramp for the resource being priced is then:

```
term = support_amount * count
if ramp_max != 0 and term > ramp_max:  term = ramp_max
if maize:                              term = term * (100 - MAIZE_RAMPING_BONUS) / 100
cost += term
```

`SUPPORT` amounts are *not* multiplied by the cost factor — a citizen's
`1f support` is one food, against a base of twenty. Buildings scale theirs by
`BUILD_SUPPORT_FACTOR`, which ships as one, so nothing scales in practice; the
constant exists only for the building side and is recorded because its absence
on the unit side is the thing that has to be established rather than assumed.

**The ceiling.** `ramp_max` is a percentage of the base price *before any
discount* — `ramp_max_percent * base * cost_factor / 100` — chosen by what
kind of unit it is:

| Constant | Ships | Applies to |
| --- | --- | --- |
| `UNIT_SCHOLAR_RAMP_MAX` | 2000% | scholars |
| `UNIT_WORKER_RAMP_MAX` | 500% | citizens, merchants |
| `UNIT_OTHER_CIVILIAN_RAMP_MAX` | 200% | generals, spies, supply wagons, caravans |
| `UNIT_MILITARY_RAMP_MAX` | 125% | everything that fights |

**A military unit's price at most doubles, and it gets there fast.** A hoplite
is `5f/3m` — fifty food and thirty metal — with `1f/1m support` and
`PROGRESSION 3`. The ramp is triangular over the barracks count, so the term
reaches the 125% ceiling of 62 food at eleven barracks units and stops. Your
first hoplite costs 50/30 and your twelfth costs 112/67, and so does your
hundredth. That plateau is the whole design: it taxes the opening, not the
army.

**A citizen's price climbs for a hundred citizens.** `2f` base, `1f support`,
`PROGRESSION 0`: twenty food, plus one per citizen you have, to a ceiling of
500% — a hundred extra food. The twentieth citizen costs thirty-nine food, the
fiftieth sixty-nine, and only a player at a two-hundred population cap ever
sees the ceiling.

**A scholar's price is the one that runs away.** `3g` base, `2g support`, a
2000% ceiling — six hundred wealth of headroom — and a second term nothing
else has:

```
extra = 0
k = 7
while k < count:
    extra += count - k
    k += 7
```

Seven is a university's capacity. So the first university's worth of scholars
ramps linearly at two wealth each, and every university after that adds a
convex term on top of it. The ninth scholar costs 47 wealth against a base of
30; the twenty-second costs 93. Knowledge is the one resource with no commerce
cap (`docs/ECONOMY.md`), and this is the counterweight the designers put on the
other side of it: the rate is unbounded and the price is not.

## The redirect: why an Ancient-age tech costs food

The last thing `get_cost` does, for every caller, is this loop over the six
goods:

```
for g in goods:
    if g is available:            continue
    if player has g's prerequisite:  redirect, rate = OBS_SUPPORT_GOOD,    OBS_SUPPORT_RATE
    else:                            redirect, rate = UNDISC_SUPPORT_GOOD, UNDISC_SUPPORT_RATE
    if redirect == the good being priced:
        cost += get_cost(g) * rate >> 8
```

and `Type::pay_cost` charges only the goods that *are* available. Together
those two facts say: **a cost written in a resource you cannot yet gather is
charged in a different resource instead**, at a rate the data gives per
resource. The `>> 8` fixes the rate as 8.8 fixed point, the same scale as
`OBS_PROD_RATE` in `docs/ECONOMY.md`.

Shipped, the undiscovered table is:

| Unavailable | Charged as | Rate |
| --- | --- | --- |
| Food | Wealth | 1/1 |
| Timber | Wealth | 1/1 |
| Wealth | Wealth | 1/1 |
| Knowledge | Food | 1/1 |
| Metal | **Timber** | 1/1 |
| Oil | Wealth | 1/1 |

The obsolete table is Wealth at 1/1 for all six.

**Three of the six resources are not available from the start.**
`resourcerules.xml` gives Knowledge and Metal the Classical Age as their
prerequisite and Oil the Industrial Age, so in the Ancient Age exactly two
entries of that table are live and both of them bite.

Knowledge is the visible one. `Mathematics` costs `8k/12g` and its own
prerequisite, Written Word, is Ancient — so a player who researches it before
reaching Classical is charged eighty **food** and a hundred and twenty wealth,
and the same tech costs eighty knowledge from the Classical Age on. The `25f`
on `Classical Age` itself is not a special case in the code; it is a tech
priced in food like any other.

Metal is the same rule against timber, and it is why the shape of an early
army's bill is different from a later one's: a Militia is `8m/8f`, which is
eighty timber and eighty food in the Ancient Age and eighty metal and eighty
food afterwards. Oil behaves the same way against wealth, though nothing
reachable before the Industrial Age is priced in oil, so that entry never
fires in a normal game.

Two notes on the arithmetic. The redirected amount is the source good's own
full price — base, ramp and all its discounts — and it is added *after* the
target good's discounts, so it is not discounted twice. And Wealth's redirect
names Wealth, which would recur forever; it cannot fire, because wealth is
never unavailable. Nothing else in the shipped table points at an unavailable
good, so one pass is exact.

## The discounts

Between the base price and the ramp sits a tail of about forty adjustments,
and with three exceptions every one of them is the same expression:

```
cost = cost * (100 - X) / 100
```

They are worth enumerating by *what* rather than one at a time, because the
shape carries no information and the predicates do:

- **Nations.** The Mongols on stables, the French on siege and specials, the
  Japanese on barracks and ships, the Germans on submarines and mines, the
  Americans on aircraft and upgrades, the British on anti-air, the Nubians on
  merchants, the Russians on spies, the Romans on legions and forts, the Turks
  on citizens, the Koreans on towers, the Iroquois on senates, the Indians on
  elephants, the Dutch on ships, the Bantu on cities, the Egyptians on wonders
  and on scholars — where the Egyptian bonus does not discount at all but
  *swaps the resource index*, charging food where the file wrote wealth.
- **Rare resources.** Horses, Rubber, Sulphur, Aluminium, Uranium, Wool,
  Marble, Bison and Incense each cheapen one category; Sugar, Coal, Gold and
  Iron each cheapen one *resource* across every category; Gypsum cheapens
  everything.
- **Wonders.** The Terra Cotta Army on barracks and stables, Angkor Wat on
  ships, the Pyramids on cities, the Colosseum on forts, the Space Program on
  aircraft, the Statue of Liberty on missiles and bombers. The Supercollider is
  the only one that runs the other way: an *enemy* holding it makes your
  missiles cost more.
- **Governments.** Despotism has three tiers on barracks units, Monarchy two on
  stable units, Socialism one on siege, aircraft and ships, Democracy two on
  research.
- **Being behind.** `MILITARY_UNIT_DISCOUNT` is 5% per military level the
  player's age is ahead of the unit's, and `MILITARY_UPGRADE_DISCOUNT` is 10%
  per level when researching the upgrade rather than building the unit. Both
  are scaled down when the scenario spans fewer than eight ages, by
  `discount * ages / 8`. Obsolete units get cheap; that is the mechanism.

Three that are not the standard shape, and are the interesting ones:

- **Producing at a captured building doubles the price.** If the building the
  unit comes out of is unassimilated, `cost *= 2`.
- **Being behind in ages discounts techs.** Against the most advanced
  non-allied player: one age behind is `100 - TECH_AGE_BEHIND_DISCOUNT`
  percent, and two or more ages behind is `100 - 2 * TECH_AGE_BEHIND_DISCOUNT`.
  The knowledge component uses its own constant, twice the size. Shipped, that
  is 10% and 20% behind by one age, 20% and 40% behind by two.
  `TECH_COLOR_BEHIND_DISCOUNT` is the same idea against the library's tech
  colours.
- **Final techs ramp against each other.** `cost *= (100 + RAMP_FINAL * n) / 100`
  where `n` is how many of the four final techs you already hold or are
  researching, and `RAMP_FINAL` ships as 50%. Four final techs, each half again
  as expensive as the last.

And the game option:

```
tech_cost <= 1:  cost = cost * 2/3
tech_cost 5..6:  cost = cost * 3/2   for ages and knowledge, * 5/4 otherwise
tech_cost >= 7:  cost = cost * 2     for ages and knowledge, * 3/2 otherwise
```

which is the other half of the knowledge adjustment `docs/ECONOMY.md` found in
`do_gather`. One lobby setting, pulling income down and prices up at the same
time.

## Researching an upgrade is not building a unit

`get_cost` takes the same fork everywhere: is this type already available to
this player? A bit per type id in the leader's availability set answers it, and
the two arms are entirely different prices.

**Available** — you are building one, and the ramp above is what happens.

**Not available** — you are paying to *research* it, and instead of the ramp:

```
cost = cost * RESEARCH_PREMIUM >> 8
cost = cost * RESEARCH_PREMIUM_COST >> 8
```

`RESEARCH_PREMIUM` is a global, written `1/1` and loaded through
`get_fraction(name, 0x100)` — so it is 8.8 fixed point, it is 256, and it is
the identity. Its own comment in `rules.xml` says as much: *"Please don't use
except on your own machine for testing; for main line game adjust times
individually"*. `RESEARCH_PREMIUM_COST` is the per-unit one that was adjusted
individually, and it is **2** for 355 of the 364 unit records. **Researching an
upgrade costs twice what one of the new units costs**, and the same holds for
time through `RESEARCH_PREMIUM_TIME`.

On top of that, a surcharge for the army you already have. For each unit type
that upgrades into this one, the engine takes the price difference per unit,
clamps it to `UNIT_REFIT_MAX_COST` — 40 per resource per unit — and adds
`UNIT_COST_FACTOR * difference * count`. So upgrading is not free of your
existing army's size: refitting thirty knights into cuirassiers is charged for.
This is the part of the cost path that is read but not fully understood; see
the open questions.

## Paying

`Type::pay_cost` and `Type::unpay_cost`, per available good:

```
pay:    bucket[t] = max(0, bucket[t] - cost[t])
unpay:  bucket[t] = bucket[t] + cost[t]
```

`unpay` is skipped entirely when the game's "costs are free" flag is set, which
is the only asymmetry between them.

`Build::refund_cost` is a third path and a stranger one: a cancelled item is
refunded what it *paid*, and the refund is re-derived by dividing out the
science discount that was in force when it was paid rather than by remembering
the number. If the discount changed in between, the refund differs from the
payment. It is recorded because it is the only place in this subsystem where a
price is reconstructed instead of stored.

### Escrow, and what it is for

`docs/ECONOMY.md` recorded `LeaderData::escrow` as a second pile fed at
`escrow_rate` percent of income and never saw it read back. It is read here.

```
affordable = bucket[t] - escrow[t]        # ordinary spending
affordable = bucket[t]                    # spending marked as against escrow
```

and in `pay_cost`, if an ordinary payment would need more than the
non-escrowed part:

```
if bucket[t] - escrow[t] < cost[t]:  escrow[t] = 0
```

So **escrow is a soft reservation**: resources set aside are invisible to
ordinary spending, and the moment ordinary spending actually needs them the
whole reservation is abandoned rather than partially consumed. A payment marked
as against the escrow draws the escrow down instead. What sets `escrow_rate` is
still unread, and the mechanic's purpose — saving toward a wonder, most likely
— is inference. Its arithmetic is not.

### The affordability count takes the maximum

`TypeData::can_pay_cost` answers "how many of these can I afford", and
`Leader::can_pay` compares its answer against the count in a queue order:

```
best = -1
for t in available goods:
    c = cost[t]
    if c == 0: continue
    n = affordable[t] / c
    if n == 0: return 0
    if n > best: best = n
return best if best != -1 else 10
```

The last comparison is a maximum. The number of items you can afford is the
minimum over resources, not the maximum, and the difference is real: with a
tank costing metal and oil, a player holding ten tanks' worth of metal and two
of oil is told they can afford ten.

It is stated here as arithmetic rather than as a verdict, because the early
return blunts it. Any resource you cannot afford even one of returns zero
immediately, so for a single item — which is what `can_pay` is asked about
almost every time it is asked — the maximum and the minimum agree. Whether a
multi-item queue order visibly overcommits in play is a question for phase 2,
and it is on the list.

The implementation takes the maximum, because that is what the original takes.
See `docs/DECISIONS.md` entry 15, for which this is the motivating case.

Two flags short-circuit the whole function. A game-wide "no costs" semaphore
returns 10, and the scenario editor's separate `units_free`, `buildings_free`
and `techs_free` return 10 for their own category. Ten is not a count of
anything; it is the sentinel this function uses for "plenty".

## Population

`Leader::calc_pop_cap`, whose result lives in `LeaderData::pop_cap`:

```
if a scenario set this player's cap explicitly:  use it, and skip to the end

cap   = POP_CAP[age]
limit = the lobby's population setting

if limit > POP_CAP[7]:
    if age == 7:                      cap = limit
    elif limit - POP_CAP[7] >= 100:   if age > 3:  cap = POP_CAP[age] + (age - 3) * 25
    elif limit - POP_CAP[7] >  49:    if age > 5:  cap = POP_CAP[age] + (age - 5) * 25

for each city:  cap += CityData::pop_cap(city)

if Bantu:
    cap   = cap   * (100 + BANTU_POP_CAP) / 100
    limit = limit * (100 + BANTU_FINAL_POP_CAP) / 100

cap = min(cap, limit)
if Virtual Reality:  cap = limit

if peacocks:  cap = cap * (100 + PEACOCKS_POP) / 100
if Colossus:  cap = cap + COLOSSUS_POP_CAP
```

`POP_CAP` ships as `25 50 75 100 125 150 175 200` by age, and the lobby offers
`50 75 100 125 150 200`. **The largest lobby setting is exactly `POP_CAP[7]`**,
so the three bracketed clauses cannot fire in a stock game — they exist for
scenarios, which reach a custom limit through `ScenarioFuncSet::set_population_cap`
or through `MAX_POP_LIMIT`'s 300. `CityData::pop_cap` is `VILLAGE_POP` for a
city, twice that for a town, three times for a metropolis or a Forbidden City,
and `VILLAGE_POP` ships as **zero**.

So, shipped and stripped of everything that does not fire: **your population
cap is `min(POP_CAP[age], the lobby setting)`, plus fifty for the Colossus,
plus a tenth for peacocks, doubled for the Bantu.** Cities do not raise it.
Twenty-five in the Ancient age, two hundred in the Information age, and the
age table is the whole of it.

`MILITARY_POP` and `GRANARY_POP` are read into `Constants` and ship as zero,
and nothing that has been read consumes them. By `docs/DECISIONS.md` entry 12
they do not enter `Tuning`.

**What a unit occupies.** `UnitData::control_cost` returns the type's `POP`
field, except that a garrisoned or otherwise inactive unit returns zero. `POP`
ships as 1 for 317 unit records, 2 for 38 — the larger warships and fireships —
and 0 for nine, the transports and meta-units that exist to hold other things.

**The check.** `LeaderData::check_population(type)` is a single line:

```
would_exceed = pop_cap < control + type.POP
```

where `control` is the population currently occupied. Note the strictness: a
unit that lands you exactly on the cap is allowed. Note also what it does *not*
do — there is no queue-aware variant here, so what stops the queue rather than
the spawn is a question for the production document.

---

## What is not established

- **The upgrade path's cost.** Two loops in `get_cost` walk unit types looking
  for ones that upgrade or graft into the type being priced, one adding the
  positive price difference and one adding the refit surcharge. The surcharge's
  arithmetic is read; which of the two loops runs when, and what the leader's
  two per-type count arrays at `+0x56fe` and `+0x5a22` are counting, is not.
  Everything in this document about *building* a unit is unaffected.
- **A third `(resource, amount)` pair on the type**, at `+0x260`/`+0x264`,
  which the research branch adds as a flat surcharge when it names the resource
  being priced. It sits immediately before the two `SUPPORT` slots and is
  filled by neither `ObjectType::load_support` nor `Type::load_cost`. Which
  column writes it is unread.
- **`Build::queue_up` and when the price is actually charged.** Whether a
  queued item pays on queue, on start, or on completion decides whether
  cancelling can profit from a discount that arrived in between — which
  `Build::refund_cost` suggests it can.
- **`JOB_TIME`, `JOB_EXTRA_TIME` and `RESEARCH_PREMIUM_TIME`.** The time half
  of the same record. `JOB_EXTRA_TIME` ships as `1/10tsx` for 360 of 364 units
  and is described as a ramp on build time, so time almost certainly ramps the
  way price does. Unread.
- **What writes `escrow_rate`.**
- **Whether the maximum in `can_pay_cost` is visible in play**, which needs
  phase 2.
- **The tech tree.** `LeaderData::has_preq`, `type_avail` and `type_eligible`
  decide whether a thing can be bought at all, and this document assumes their
  answers rather than deriving them. The redirect above depends on `type_avail`
  for goods, which `docs/ECONOMY.md` also leans on.
- **Building and wonder ramps.** The building branch of `get_cost` has its own
  count escalation — military production buildings and forts step their count
  up super-linearly past the second, third and fourth — and wonders ramp
  against how many wonders you and your team already hold. Read in outline,
  not derived.
- **`MIN_POP_LIMIT` and `MAX_POP_LIMIT`.** Loaded into `Constants` and not
  consumed by anything read so far; the lobby is the likely reader and the
  lobby is cut from v1.
