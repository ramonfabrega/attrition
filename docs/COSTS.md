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
that spends this price over time, and they are `docs/PRODUCTION.md`.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the spine — the cost factors, the ramp in all four of
its shapes, the four ramp ceilings, the unavailable-good redirect, the payment
and refund arithmetic, and the population cap are each read end to end at their
consumers, and two of them are confirmed against the designers' own column
comments in `unitrules.xml`. All of it has now been derived twice, blind, and
adjudicated back to the decompiled functions; see the last section for what that
changed. High for the *shape* of the discount tail, which is one expression
repeated forty times; the individual predicates behind each are enumerated
rather than each separately derived, and *where in the sequence* each one sits
was the second reading's most common correction. The research path — what an
*upgrade* costs, as opposed to a unit — is now read end to end too. What remains
open is listed at the end, and the building and wonder ramps are the largest
piece of it.

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

`TypeData::get_cost` reads it as the per-unit ramp. In this engine's data
vocabulary **"support" means price, not upkeep**, and the two subsystems
together settle it: the field the income path never touches is the one the cost
path builds its ramp out of.

The word is overloaded twice over, and the second one cost this document a
whole table. `resourcerules.xml` also carries four `*_SUPPORT_GOOD` /
`*_SUPPORT_RATE` columns per resource, sitting immediately after four
`*_COST_GOOD` / `*_COST_RATE` columns that look just like them. The redirect
below reads the **cost** pair. Nothing found reads the support pair — not the
cost path, not the income path in `docs/ECONOMY.md`. (An earlier draft of this
document transcribed the support columns into the redirect table and got every
rate wrong; see §The redirect.)

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
bombers. For the Nuclear Missile specifically, `nukes_used` is added **twice** —
once by `get_support_count` and once again by `get_cost` itself, which repeats
the test for that one type. Nothing in the data suggests that is deliberate.

If the type's `PROGRESSION` has bit 0 set **and its `attack` is not zero**, the
count is taken over the whole **production group** instead of the type —
barracks units, stable units, factory units, dock units, or air units, each as
`queued + built`. This is why switching from crossbowmen to musketeers does not
reset the price: they draw on the same barracks count. (An earlier draft had
only the progression bit. The `attack != 0` half is inert in the shipped data —
no non-attacking type sets the bit — but it is the rule, and the same pair
gates the counters in `LeaderData::track_unit_type`.)

Two more terms belong to the count and not to the arithmetic, and both are
about citizens. A citizen's count adds every militia, minuteman and partisan
the player has made, minus the ones made from *scholars*; a scholar's count
adds those back. So converting citizens to militia does not reset the citizen
ramp — the militia go on making the next citizen more expensive, and a scholar
who picked up a rifle goes on making the next scholar more expensive. (Not in
an earlier draft at all.)

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
| `UNIT_WORKER_RAMP_MAX` | 500% | citizens, merchants, caravans, merchant fleets |
| `UNIT_OTHER_CIVILIAN_RAMP_MAX` | 200% | generals, spies, supply wagons |
| `UNIT_MILITARY_RAMP_MAX` | 125% | everything that fights |

The test is a nest rather than a table, and the order decides two records.
Scholars go by name; then citizens, `is_merchant`, and anything with
`unit_flags2 & 8` are workers; and only if all of those fail does `obj_masks &
4` — the designers' letter `C`, for Civilian — separate the other civilians
from the fighting units. `UnitType::init_final_flags` sets bit 8 for exactly
two types, the Caravan and the Merchant Fleet, and both also carry the `C` in
their `OBJ_MASK`. The flags test runs first, so both ramp to 500%, not 200%.
(An earlier draft had caravans in the 200% class and merchant fleets nowhere.)

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
    if player has g's prerequisite:  redirect, rate = OBS_COST_GOOD,    OBS_COST_RATE
    else:                            redirect, rate = UNDISC_COST_GOOD, UNDISC_COST_RATE
    if redirect == the good being priced:
        cost += get_cost(g, include_redirect=1) * rate >> 8
```

and `Type::pay_cost` charges only the goods that *are* available. Together
those two facts say: **a cost written in a resource you cannot yet gather is
charged in a different resource instead**, at a rate the data gives per
resource. The `>> 8` fixes the rate as 8.8 fixed point, the same scale as
`OBS_PROD_RATE` in `docs/ECONOMY.md`; `GoodType::init` loads all four columns
through `String::fraction(text, 0x100)`, which is where the scale comes from.
The fields are `GoodTypeData +0x2b4`/`+0x2c8` and `+0x2b8`/`+0x2cc`.

Shipped, the undiscovered table is:

| Unavailable | Charged as | Rate |
| --- | --- | --- |
| Food | Wealth | 3/2 |
| Timber | Wealth | 3/2 |
| Wealth | Wealth | 3/2 |
| Knowledge | **Food** | 3/2 |
| Metal | **Timber** | 5/4 |
| Oil | **Metal** | 3/2 |

and the obsolete table is Wealth at 1/1 for all six except Timber, which goes
to Oil at 1/2. Every resource's `<OBS>` ships as `disable`, so the obsolete
table never fires in a stock game; the undiscovered one fires constantly.

(An earlier draft of this document had all six undiscovered entries at 1/1,
Oil going to Wealth, and a uniform obsolete table. It had read the
`*_SUPPORT_GOOD`/`*_SUPPORT_RATE` columns, which sit four fields further along
each record and which nothing found reads. Every worked number below moved as a
result, and so did `Redirects::RON` and the `rondata` check that was supposed to
be guarding it — the check was reading the same wrong columns, so it agreed
with the wrong table and proved nothing.)

**Three of the six resources are not available from the start.**
`resourcerules.xml` gives Knowledge and Metal the Classical Age as their
prerequisite and Oil the Industrial Age, so in the Ancient Age three entries of
that table are live. Two of them bite on shipped prices; the third, Oil, has
nothing priced in it that early and matters only because of what it points at.

Knowledge is the visible one. `Mathematics` costs `8k/12g` and its own
prerequisite, Written Word, is Ancient — so a player who researches it before
reaching Classical is charged `(80 × 384) >> 8` = **a hundred and twenty food**
and a hundred and twenty wealth, and the same tech costs eighty knowledge from
the Classical Age on. The rate is not a formality: **researching early costs
half again as much as the file's number**, which is the designers' thumb on the
scale against skipping the age. The `25f` on `Classical Age` itself is not a
special case in the code; it is a tech priced in food like any other.

Metal is the same rule against timber at five quarters, and it is why the shape
of an early army's bill is different from a later one's: a Militia is `8m/8f`,
which is `(80 × 320) >> 8` = a hundred timber and eighty food in the Ancient
Age, and eighty metal and eighty food afterwards.

Oil is the one that chains. It redirects into **metal**, not wealth, and metal
is itself unavailable until the Classical Age — so in the Ancient Age an oil
price is charged in metal at three halves, and that metal price is charged in
timber at five quarters. Thirty oil is forty-five metal is fifty-six timber.
The original gets this by recursion: the redirect calls `get_cost` on the
missing good with `include_redirect` set, so the inner call runs the whole loop
again. (An earlier draft said one pass was exact, on the strength of a table
where nothing pointed at an unavailable good. In the real table Oil points at
Metal, and it does chain. Nothing shipped is priced in oil that early, so it is
a correctness note rather than a stock-game effect — but the implementation
recurses now, and `charges()` is tested on the chain.)

Two notes on the arithmetic. The redirected amount is the source good's own
full price — base, ramp and all its discounts — and it is added *after* the
target good's discounts, so it is not discounted twice. And Wealth's redirect
names Wealth, which would recur forever; it cannot fire, because wealth is
never unavailable. The original has no cycle guard at all, so a modded table
that closed a loop would run the stack out; `crates/sim` refuses to re-enter a
good already on the recursion stack, which is exact for the shipped table and
terminates for any other.

## The discounts

Around the ramp sits a tail of about forty adjustments, and with three
exceptions every one of them is the same expression:

```
cost = cost * (100 - X) / 100
```

Most of them land between the scaled base and the ramp. Six land **after** it,
in this order: `MILITARY_UNIT_DISCOUNT`, Monarchy on stable units, Socialism on
siege, air and dock units, Salmon on ships, then the common tail of Coal /
Sugar / Gold / Iron by resource and Gypsum on everything, and after the
captured-building doubling the Supercollider surcharge and the Indian elephant
discount. (An earlier draft said the whole tail sat before the ramp. Where a
discount sits decides whether the ramp's ceiling — a percentage of the
*undiscounted* base — is measured against a number that discount has already
touched, so it is not bookkeeping. `Modifiers` in `crates/sim` has carried two
slots for this all along; the late one now names them.)

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
  missiles cost more. Two of these are not what they look like. Terra Cotta's
  clause has no `has_wonder` test on it at all — it applies
  `TERRA_COTTA_COST` to every barracks, stable and auto-plant unit whether or
  not anyone built the wonder, which is invisible only because the constant
  ships as `0% reduction`. And nested inside that same clause is a second
  application of `ANGKOR_SHIPS`, gated on holding Angkor Wat but applied to
  those **land** units, on top of the correct one that the sea domain gets a
  few lines later. `ANGKOR_SHIPS` ships as 25%, so this one is live: an Angkor
  Wat owner buys barracks, stable and auto-plant units a quarter off, which no
  description of the wonder mentions.
- **Rare resources that are not.** Salt on barracks units does not read
  `SALT_BARRACKS_COST`, which ships as 15%. The code multiplies by a literal
  nine tenths, so Salt is worth 10% and the constant is decoration.
- **Governments.** Despotism has three tiers on barracks units, Monarchy two on
  stable units, Socialism one on siege, aircraft and ships, Democracy two on
  research. Despotism's three are an else-chain, so only the highest tier the
  player holds applies rather than all three compounding.
- **Being behind.** `MILITARY_UNIT_DISCOUNT` is 5% per **Military tech level**
  the player holds above the unit's own `MILITARY_LEVEL`, and
  `MILITARY_UPGRADE_DISCOUNT` is 10% per level when researching the upgrade
  rather than building the unit. Obsolete units get cheap; that is the
  mechanism. (An earlier draft called it "per military level the player's age
  is ahead of the unit's". It is the Military library line — `epoch[0]`, the
  same count that sets the population cap — and the age has nothing to do with
  it.) Both are scaled down when the scenario spans fewer than eight ages, and
  the scaling rounds **up**: `(ages × pct + 7) >> 3`, not `pct × ages / 8`. The
  unit discount additionally floors the result at 1 and reads the unit's
  military level as `max(level, 1)`, so a level-zero unit is treated as
  level 1.

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
- **Science cheapens every technology, and can make one dearer.**
  `LeaderData::calc_science_discount` subtracts
  `(science_level - age) × TECH_SCIENCE_DISCOUNT × cost / 100`, where
  `science_level` is `epoch[3]`, the Science library line, and `age` is the
  tech's own `AGE` column plus one unless the tech is itself an age or a
  library tech. A player whose Science level is behind the tech's age gets a
  negative discount — a surcharge, from the same expression. It is
  part of every technology's price and was missing from an earlier draft of
  this list. `Build::refund_cost` inverts exactly this expression, which is why
  a queued item's price is re-derived rather than remembered.
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
this player? A bit per type id in the leader's availability set — `leader +
0x6c18` — answers it, and the two arms are entirely different prices.

**Available** — you are building one, and the ramp above is what happens. With
one addition that an earlier draft could not place. Before the cost factor is
applied, the engine walks every unit type with something *queued*, and for each
one whose `FROM` resolves to this type, or which lies on this type's graft
chain, adds `max(0, that type's base − this type's base)` per resource. So
**ordering the upgrade makes the old unit cost the new one's price while the
research is in the queue**: start researching Musketeers and the Arquebusiers
you keep training in the meantime are charged at the Musketeer's base. It is
added to the unscaled base, so the cost factor multiplies it too.

**Not available** — you are paying to *research* it, and instead of the ramp:

```
if wine:  cost = cost * (100 - WINE_UNIT_UPGRADES) / 100
cost = cost * RESEARCH_PREMIUM >> 8
cost = cost * RESEARCH_PREMIUM_COST >> 8
if type.SPECIAL_UPGRADE == the resource being priced:
    cost += type.SPECIAL_UPGRADE_COST
```

`RESEARCH_PREMIUM` is a global, written `1/1` and loaded through
`get_fraction(name, 0x100)` — so it is 8.8 fixed point, it is 256, and it is
the identity. Its own comment in `rules.xml` says as much: *"Please don't use
except on your own machine for testing; for main line game adjust times
individually"*. `RESEARCH_PREMIUM_COST` is the per-unit one that was adjusted
individually, and it is **2** for 355 of the 364 unit records. **Researching an
upgrade costs twice what one of the new units costs**, and the same holds for
time through `RESEARCH_PREMIUM_TIME`.

On top of that, a surcharge for the army you already have:

```
for each unit type p whose FROM resolves to this type:
    d = this.base[res] - p.base[res]
    if p.base[res] == 0:  d = d / 2
    if d > 0:
        d = min(d, UNIT_REFIT_MAX_COST)
        n = num_queued[p] + num_units[p]
        cost += UNIT_COST_FACTOR * n * d
```

`UNIT_REFIT_MAX_COST` is 40 per resource per unit. So upgrading is not free of
your existing army's size: refitting thirty knights into cuirassiers is charged
for. The halving is the interesting clause — if the old unit was free in this
resource, the difference is the new unit's whole price and the engine charges
half of it. The citizen types 0x42–0x44 skip the loop entirely.

**Which loop runs when is now settled**, and it is the fork itself: the
bump-while-queued loop is the *available* arm and the refit loop is the *not
available* arm. They never both run. (An earlier draft of this document could
not tell them apart and said so; this closes it.)

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

Note the early return: it is `n == 0`, not `n <= 0`. A purse smaller than its
own escrow divides to a *negative*, which neither returns zero nor raises the
running maximum — so a price met by that one resource alone falls out of the
loop having raised nothing and answers the "plenty" sentinel. (An earlier draft
of `crates/sim` wrote `n <= 0` and returned zero there. The original's quirk is
reproduced now, and tested.)

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
if the leader's 0x100000 flag is set:            cap = limit, skip to the end

mil   = epoch[0], the player's Military library level
cap   = POP_CAP[mil]
limit = the lobby's population setting

if limit > POP_CAP[7]:
    if mil == 7:                      cap = limit
    elif limit - POP_CAP[7] >= 100:   if mil > 3:  cap = POP_CAP[mil] + (mil - 3) * 25
    elif limit - POP_CAP[7] >  49:    if mil > 5:  cap = POP_CAP[mil] + (mil - 5) * 25

for each city:  cap += CityData::pop_cap(city)

if Bantu:
    cap   = cap   * (100 + BANTU_POP_CAP) / 100
    limit = limit * (100 + BANTU_FINAL_POP_CAP) / 100

cap = min(cap, limit)
if Virtual Reality:  cap = limit

if peacocks:  cap = cap * (100 + PEACOCKS_POP) / 100
if Colossus:  cap = cap + COLOSSUS_POP_CAP
```

**The index is not the age.** `Leader::calc_pop_cap` reads `epoch[0]`, and
`LeaderData::compute_epoch` builds that by counting how many consecutive
`BASE_MILITARYTYPES` techs the player holds — the Military library line, The
Art of War through Selective Service, seven of them. So the eight entries of
`POP_CAP` are "no Military tech" through "all seven", and **the way a player
raises their population limit is by researching Military techs**, not by aging
up. (An earlier draft of this document read the index as the age, and said "the
age table is the whole of it". It is the Military table. The arithmetic below is
unchanged and the numbers in the tests still hold; what changes is what makes
them move.)

`POP_CAP` ships as `25 50 75 100 125 150 175 200`, and the lobby offers
`50 75 100 125 150 200`. **The largest lobby setting is exactly `POP_CAP[7]`**,
so the three bracketed clauses cannot fire in a stock game — they exist for
scenarios, which reach a custom limit through `ScenarioFuncSet::set_population_cap`
or through `MAX_POP_LIMIT`'s 300. `CityData::pop_cap` is `VILLAGE_POP` for a
city, twice that for a town, three times for a metropolis or a Forbidden City,
and `VILLAGE_POP` ships as **zero**.

So, shipped and stripped of everything that does not fire: **your population
cap is `min(POP_CAP[Military level], the lobby setting)`, plus fifty for the
Colossus, plus a tenth for peacocks, doubled for the Bantu.** Cities do not
raise it. Twenty-five with no Military tech, fifty with The Art of War, two
hundred with all seven, and the Military line is the whole of it.

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
do — there is no queue-aware variant here.

**And it turns out it does not need one.** `docs/PRODUCTION.md` answers the
question this paragraph used to leave open: nothing stops the queue. A player
at the cap may order freely and the counter runs to completion at the ordinary
training pace; the same comparison, written out again inside `Build::finished`,
refuses the *handover*. The item then sits at a hundred percent, already paid
for, and retries every frame until room appears. Which is exactly what the
original's interface shows a player, and not at all what "the cap stops
production" would predict.

---

## What is not established

- ~~**The upgrade path's cost.** Which of the two loops runs when is not
  read.~~ **Closed** by the second reading: the availability fork decides it.
  The loop that adds the positive base difference is the *available* arm — the
  old unit costs the new one's base while the upgrade sits in the queue — and
  the refit surcharge is the *not available* arm. See §Researching an upgrade,
  which now writes both out.

  **The two count arrays are settled**, by `docs/PRODUCTION.md`. `+0x5a22` is
  `num_queued`, `ushort[806]`, indexed by type id and moved by `Build::queue_up`
  and `Build::unqueue`. `+0x56fe` is not an array at all: it is `num_units`,
  `ushort[352]` at `+0x5762`, addressed with the unit type id directly because
  unit ids start at `0x32` and the compiler folded the `- 0x32` into the base
  pointer. So the pair is "ordered" and "standing", which is what makes the
  `built + queued` above literal.
- **A third `(resource, amount)` pair on the type**, at `+0x260`/`+0x264`,
  which the research branch adds as a flat surcharge when it names the resource
  being priced. ~~What it is~~ is settled: the symbols call it
  `special_upgrade` and `special_upgrade_cost`. **Which column writes it is
  still unread** — neither reader found a `SPECIAL_UPGRADE` in `unitrules.xml`
  or `buildingrules.xml`, and it is filled by neither `ObjectType::load_support`
  nor `Type::load_cost`.
- ~~**`Build::queue_up` and when the price is actually charged.**~~ **Closed**
  by `docs/PRODUCTION.md`: on queue. `Type::pay_cost` both debits and reports,
  and the report is written into the queue entry as up to three
  `(resource, amount)` pairs, which is exactly what `Build::unpay_cost` gives
  back on a cancel. So cancelling *cannot* profit from a discount that arrived
  in between — the refund is the number that was paid. `Build::refund_cost` is
  a different thing entirely: `Leader::gain_tech` is its only caller, and it
  re-prices every queued item in place when your science rises, handing the
  difference back where the item sits.
- ~~**`JOB_TIME`, `JOB_EXTRA_TIME` and `RESEARCH_PREMIUM_TIME`.**~~ **Closed**
  by `docs/PRODUCTION.md`. Time does ramp the way price does, with one ceiling
  instead of four and against a different count: the price ramp reads
  `num_units + num_queued` and the time ramp reads `num_units` alone.
- **What writes `escrow_rate`.**
- **Whether the maximum in `can_pay_cost` is visible in play**, which needs
  phase 2.
- ~~**The tech tree.**~~ **Closed** by `docs/TECH.md`: `has_preq`,
  `type_eligible` and `type_avail` are derived there. For a good, `type_avail`
  is its first prerequisite — the age that unlocks it — and nothing else, so
  Knowledge and Metal arrive with Classical and Oil with Industrial, which is
  what the redirect above and `docs/ECONOMY.md` assumed.
- **Building and wonder ramps.** The building branch of `get_cost` has its own
  count escalation and wonders ramp against how many wonders you and your team
  already hold. The second reading read the escalation out — military
  production buildings step `n → 2n−2` past two, `+2n−10` past five, `+3n−24`
  past eight, `+4n−48` past twelve; forts `n → 2n−1` past one, then `+n−3`,
  `+n−5`, `+n−7`; the city count is all three city types, queued and built; the
  wonder count is twice your own with team adjustments and its ramp is halved;
  and the Indians zero the ramp on everything but forts. It is recorded here
  rather than in the body because neither reader has derived it line by line,
  and nothing in `crates/sim` builds buildings yet.
- **`MIN_POP_LIMIT` and `MAX_POP_LIMIT`.** Loaded into `Constants` and not
  consumed by anything read so far; the lobby is the likely reader and the
  lobby is cut from v1.

---

## Second reading (2026-08-20) — landed

A blind second derivation and its adjudication are in
`docs/audit/2026-08-20-costs.md`. The spine is **doubly confirmed**: the cost
factors and the `f t g k m o` slots, `SUPPORT` as two ordered slots, the ramp in
all four of its shapes, the four ceilings measured against the undiscounted
base, the scholar's second term, the maize bonus, the research premium pair, the
payment and escrow arithmetic, the `can_pay_cost` maximum, and the whole
population chain from the scenario override to the Colossus.

Nine disagreements, eight resolved against the earlier draft of this document
and one an implementation slip neither reading had stated; all landed above and
in `crates/sim/src/cost.rs` (with `lib.rs`, `harness_tests.rs` and the `rondata`
check) on the same day, and each place that changed says so inline. Three
changed observable behaviour:

- **The redirect table was the wrong table.** It reads
  `UNDISC_COST_GOOD`/`RATE`, not `UNDISC_SUPPORT_*`: Knowledge → Food ×3/2,
  Metal → Timber ×5/4, Oil → **Metal** ×3/2, obsolete Timber → Oil ×1/2. An
  Ancient-age Mathematics costs 120 food, not 80; a Militia's `8m` is 100
  timber, not 80. And the redirect recurses, so Oil chains through Metal into
  Timber. The `rondata` check that should have caught this was reading the same
  wrong columns; it now reads both cost tables and passes on the corrected one.
- **The population cap is indexed by the Military library level**, `epoch[0]`,
  not the age. `Muster::age` is now `Muster::military_level`.
- **`affordable()` returned zero on a negative quotient**, where the original
  returns only on an exact zero and lets the negative fall through to the
  sentinel.

The rest were a wrong ramp class for caravans and merchant fleets, three terms
missing from the ramp count, the `attack != 0` gate on group counting, and six
discounts placed before the ramp that belong after it. The second reading also
settled the two upgrade loops, `+0x260` as `special_upgrade`, the ignore-pop-cap
flag `0x100000`, Salt's hard-coded nine tenths, Terra Cotta's missing wonder
check with Angkor Wat's live 25% on land units, the tech science discount, and
the exact `refund_cost` inversion.
