# Economy: income

How a resource arrives. Rise of Nations has no gatherer carrying a load back
to a drop-off: a citizen assigned to a farm is a **rate**, the rates are summed
per player into six numbers, each number is clamped by a **commerce cap**, and
the clamped rate is paid into the stockpile every frame through a remainder
accumulator. This document is that pipeline, end to end.

Not in it: what things *cost*, the market, tribute, and the tile-by-tile survey
that decides how many gatherer slots a mine or a forest offers. Those are
separate mechanics and are surveyed at the end.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the spine — the frame order, the refresh cadence, the
rate representation, the cap, and the accrual are all read end to end, and the
PDB's own field names for the encrypted resource block confirm every inference
that was made before they were dumped. High for what a city contributes and for
the per-gatherer rates, which are read at their consumers. **Lower for the
gatherer *count*:** `BuildTypeData::calc_gather` is a thousand lines of inlined
tile walking and only its arithmetic tail has been read. What remains open is
listed at the end.

**Where the implementation is.** `crates/sim/src/economy.rs`. Every constant
below is re-read from the user's own install by
`cargo run -p rondata -- <install>`, which fails if any has drifted.

---

## The six resources

| Index | Name | Letter | Notes |
| --- | --- | --- | --- |
| 0 | Food | `f` | |
| 1 | Timber | `t` | |
| 2 | Wealth | `g` | The engine's `TypeIndex` is `WEALTH`; the cost letter is `g` |
| 3 | Knowledge | `k` | Exempt from the commerce cap |
| 4 | Metal | `m` | |
| 5 | Oil | `o` | |

The order is the engine's, and it is the order of the `entry0`..`entry5` slots
of `STARTING_GOODS`, `BASIC_GATHER` and `CITY_GATHER`. It is *not* the order
`resourcerules.xml` lists its `RESOURCE` records in — those are the same six
followed by forty-four rare resources.

A resource participates at all only while `LeaderData::type_avail` says its
good type is available. Oil is not available before the Industrial age, and
`Leader::do_gather` skips it entirely until it is: no rate, no cap, no accrual.

## The state it touches

Almost all of it lives in a separate 248-byte block hanging off
`LeaderData + 0x6eb8`, whose type the symbols name `LeaderDataEncrypt`. Every
field in it is stored XOR-masked with a per-field constant — a tamper-resistance
measure, not arithmetic. Decoding is the reader's job; the masks are recorded
here because they are how you find these fields in a save file, and are then
never mentioned again.

| Where | Field | Type | XOR | Meaning |
| --- | --- | --- | --- | --- |
| `LeaderDataEncrypt` | `bucket` @ +0x00 | `i32[6]` | `0x8221` | The stockpile: what you can spend |
| | `leftover` @ +0x18 | `i32[6]` | `0x3421` | Fractional carry between frames |
| | `resource_cap` @ +0x30 | `i32[7]` | `0x1281` | The commerce cap, in ¹⁄₁₆ units |
| | `over_cap` @ +0x4c | `i32[6]` | `0x8932` | 0 under the cap, 1 at it, 2 if the cap is 999 |
| | `resources` @ +0x64 | `i32[6]` | `0x872` | The assembled gather **rate**, in ¹⁄₁₆ units |
| | `support` @ +0x7c | `i32[6]` | `0x26076` | Upkeep to subtract. Always zero — see below |
| | `income` @ +0x94 | `i32[6]` | `0x90236` | The net rate the interface shows |
| `LeaderData` | `base_rate` @ +0x4b0 | `i32[6]` | — | Added to the rate before the cap. Unread |
| | `escrow` @ +0x468 | `i32[6]` | — | A second pile, fed at `escrow_rate` percent |
| | `escrow_rate` @ +0x480 | `i32[6]` | — | Percent of income diverted to escrow |
| | `collected` @ +0x874 | `i32[6]` | — | Lifetime total, for the score screen |
| | `bonus_cap` @ +0x918 | `i32[7]` | — | Cap bonuses accumulated by `resource_cap_add` |
| | `gather_stamp` @ +0x7ac | `i32` | — | Frame of the last rate recompute |
| | `territory` @ +0x9d8 | `i32` | — | Owned tiles, for the territory tax |
| | `city_mark` @ +0x408 | `i32` | — | Number of cities; the loop bound |
| | `oil_well_mark` @ +0x430 | `i32` | — | Number of oil wells; the loop bound |
| `CityData` | `granary` @ +0x56 | `u8` | — | Food enhancer, as a percentage bonus |
| | `lumber_mill` @ +0x57 | `u8` | — | Timber enhancer |
| | `smelter` @ +0x58 | `u8` | — | Metal enhancer |
| | `refinery` @ +0x59 | `u8` | — | Oil enhancer |
| `WorldData` | `land_size` @ +0x78 | `i32` | — | Total land tiles; the territory tax divisor |

`resource_cap` and `bonus_cap` are seven long where six would do, and the loop
in `Leader::calc_resource_caps` really does run seven times. The seventh slot is
written and never read.

## A rate is sixteenths of a resource per 450 frames

Two scalings sit on top of each other and neither is announced.

**The period is `GATHER_RATE`, which ships as `450 frames`** — thirty seconds at
the simulation's fifteen frames per second. A rate of *R* pays out exactly *R*
resources every 450 frames.

**Rates are carried in sixteenths.** Every term in the assembly below is
multiplied by 16 as it goes in, and the accrual divides by `GATHER_RATE * 16`
coming out. Nothing rounds in between, which is the point: a market's ten
wealth and a granary's twenty percent survive as an exact `10 * 16 * 120 / 100`
rather than as two truncations.

So the numbers a player sees — `+120 food` — are rates in whole resources per
thirty seconds, and the engine holds them as 1920.

### Three constants arrive at 256× what the file writes

`Constants::init` reads most values with `get_item`, which takes the leading
integer and discards the designers' trailing note. Three of the ones here go
through `get_fraction(name, 0x100)` instead, which parses the same text and
multiplies by 256:

| Constant | Written | Loaded | Read back as |
| --- | --- | --- | --- |
| `PEASANT_RATE` | `10 resources` | 2560 | `>> 8` → 10 |
| `OIL_RATE` | `35 oil` | 8960 | `>> 8` → 35 |
| `SCHOLAR_RATE` | `5 7 10 15 20 25` | ×256 each | `* 16 >> 8` → ×16 rate |

This is the trap `CLAUDE.md` warns about, in its sharpest form: these are
written as **plain integers with no `/` in them**, so nothing about the text
suggests a scale. `CITY_GATHER`'s `10food`, three lines away in the same file,
is loaded plain. Only the consumer tells you which is which, and the consumers
here are a `>> 8` that would otherwise turn a farm's ten food into zero.

The project already carries one of these — `PARMENIO_RADIUS_ADJUST`, written
`3/2` — as `Slot::Ratio256`. These three join it, and the fact that they are
integers rather than rationals is why the check has to be per-constant rather
than per-syntax.

## The frame

**Income is paid before anything else in the frame happens.** `Game::do_frame`
runs `Leaders::process_all` first, and `Objects::process_all` — which is where
`docs/ATTRITION.md` and `docs/MOVEMENT.md` both live — nine calls later. So
every player is paid for a frame before any unit in it moves, fights or bleeds,
and a citizen killed this frame was already paid for it.

`Leader::process` calls `Leader::gather` every frame, for every player. It does
three things in order:

1. **`Leader::calc_gather`** — reassembles the six rates from scratch. Gated
   by a cadence; usually returns immediately.
2. **`Leader::calc_resource_caps`** — recomputes the six commerce caps. Not
   gated: it runs every frame.
3. **`Leader::do_gather`** — pays the rates into the stockpile. Every frame.

`gather` zeroes `support[0..5]` between (1) and (2), so whatever `calc_support`
was once for, the value `do_gather` subtracts is written as zero every frame.

### The rate is recomputed lazily, and the lazy period is 512 frames

`calc_gather` returns without doing anything unless one of two conditions holds.

- **A dirty flag** (`LeaderData` flag `0x2000000`) is set. Then it recomputes
  when `(who + frame) % 8 == 0`, and unconditionally on frame zero. The flag is
  cleared at the end of a recompute and set elsewhere by whatever changes the
  economy — a building finished, a city taken. This is why the income display
  responds to building a farm within half a second.
- **Otherwise** it recomputes when `frame >= gather_stamp + 300` *and*
  `(frame + who * 8) % 256 == 0`.

Those two clauses together do not give a 300-frame period, and they do not give
a 256-frame one either. The grid is 256 frames and the minimum gap is 300, so
the first grid point after the gap is **512 frames** — thirty-four seconds. A
player who changes nothing sees their income refresh about twice a minute.

The `who * 8` is the same trick attrition uses for units: work is spread across
frames by index, so eight players do not all reassemble their economies on the
same tick. It is part of the simulation, not an optimisation, because it decides
*which* frame a change becomes visible on.

## Assembling the rate

`Leader::calc_gather` fills six integers, in this order. Everything is in
sixteenths.

1. **`BASIC_GATHER[t] * 16`** — a free trickle per resource. Ships as zero for
   all six.
2. **Nation flat bonuses** — the Lakota's food per unit, the Americans' barracks
   gather.
3. **Every city**, via `City::calc_gather` → `LeaderData::calc_city_resources`.
   See below.
4. **Every oil well**, via `BuildData::calc_gather`.
5. **Buildings outside a city** of two specific types, via the same call.
6. **Every idle merchant and caravan**, via `Unit::do_gather`.
7. **Refineries**: `oil = oil * (refineries * REFINERY_BONUS + 100) / 100`.
   `REFINERY_BONUS` is `33% per refinery` and this is where "per refinery"
   lives; every other enhancer is a per-city percentage.
8. **Capitalism** adds `CAPITALISM_OIL_PROD * 16` to oil.
9. **Rare resources** — one `LeaderData::calc_rare` per owned rare.
10. **Coffee** scales all six.
11. **The territory tax** (below).
12. **`LeaderData::calc_resource_bonuses`** — the wonder and nation percentage
    multipliers: Pyramids on food, Colossus and Taj on wealth, Angkor on metal,
    Tikal on timber, Eiffel and the Russians on oil, Hanging Gardens as a flat
    addition to knowledge.
13. **The obsolete redirect** (below).

### The territory tax

```
wealth += territory * TERRITORY_TAXES[taxation_level] * 16 / land_size
```

`TERRITORY_TAXES` ships as `0, 50, 100, 200, 300` percent by taxation level.
Read as a rate: **a player holding a fraction *f* of the map's land earns
`f * TERRITORY_TAXES[level]` wealth per thirty seconds.** A fifth of the map at
full taxation is twenty wealth — two markets' worth, from borders alone. This
is the one place in the game where territory pays rather than merely hurting
whoever stands in it, and it is the economic mirror of `docs/ATTRITION.md`.

### The obsolete redirect

The last thing `calc_gather` does is walk the six goods looking for one that is
*no longer available* but whose prerequisite the player still holds. For each
such good it moves the whole rate somewhere else and zeroes it:

```
out[obs_prod_good] += out[t] * obs_prod_rate >> 8
out[t] = 0
```

`obs_prod_good` and `obs_prod_rate` are `OBS_PROD_GOOD` and `OBS_PROD_RATE`
from `resourcerules.xml` — Food's are `Wealth` and `1/2`. The `>> 8` is what
fixes `OBS_PROD_RATE` as 8.8 fixed point, so `1/2` is stored as 128 and half
the food income becomes wealth.

Nothing in a standard game makes a basic resource obsolete, so this never fires
in normal play. It is recorded because it closes a data claim with its
consumer, which is the only way this project is willing to assert a field's
meaning.

## What a city gives

`LeaderData::calc_city_resources` sums four things for one city.

**Its buildings.** Every gathering building attached to the city contributes
through `BuildData::calc_gather`. Two building types are excluded by identity,
and the general shape of the contribution is `per-gatherer rate × gatherers`,
with the per-gatherer rate coming from `PEASANT_RATE` (or `OIL_RATE` for oil,
or `SCHOLAR_RATE[university_level - 1]` for knowledge) and then scaled by the
city's enhancer for that resource.

**`CITY_GATHER[t] * 16`, once per city.** Ships as `10 food, 10 timber` and
zero for the rest — so **a city is worth ten food and ten timber per thirty
seconds before anybody works in it.** A Forbidden City replaces this with
`FORBIDDEN_CITY_BASE_GATHER` for every resource, and scales the city's whole
output by `FORBIDDEN_CITY_GATHER` percent.

**Taxes, into wealth.** `CityData::get_taxes` is:

```
VILLAGE_TAXES + buildings * BUILDING_TAXES
              + (has market ? MARKET_TAXES : 0)
              + (has temple ? TEMPLE_TAXES : 0)
```

Three of those four ship as zero. `MARKET_TAXES` is 10. **The entire taxes line,
as shipped, is "a market is worth ten wealth per thirty seconds."** The other
three constants are read, so they are tuning rather than dead weight, but they
do nothing in a stock game.

**Literacy, into knowledge.** `CityData::get_literacy` is the same shape:
`VILLAGE_LITERACY` (0) plus `UNIVERSITY_LITERACY` (10) if the city has a
university plus `LIBRARY_LITERACY` (0) if it has a library. **A university is
worth ten knowledge per thirty seconds**, and a library on its own is worth
nothing — the library's value is that scholars can live in it.

Then the Romans add wealth per city and the Germans add food, timber and metal
per city.

### The enhancers, and an array that runs off its end

`CityData::enhancer_amount(t, base)` returns `(pct[t] + 100) * base / 100`,
where `pct` is the city's `granary` / `lumber_mill` / `smelter` / `refinery`
byte. Wealth and knowledge have no enhancer and take the default zero.

`City::calc_gather` fills those four bytes from four constant arrays, and it
reaches them the way this codebase has now seen three times:

```
granary = FISHERMEN_BONUS[granary_level + 4]
```

`FISHERMEN_BONUS` is `int[5]`, so index `granary_level + 4` is past its end and
into `GRANARY_BONUS`, which follows it in the struct. Written the way the
designers meant it, that is `GRANARY_BONUS[granary_level - 1]` — the levels are
one-based, exactly as `SCHOLAR_RATE[university_level - 1]` is. The same holds
for the lumber mill and the smelter. As with `ATTRITION_UPGRADE` and
`FORT_UPGRADE_TERR` in `docs/ATTRITION.md`, the arrays are separate here and
the indices are written the way they were meant.

Shipped, `GRANARY_BONUS` is `20 50 100 200 250` percent by level.

## The commerce cap

`Leader::calc_resource_caps`, every frame, for seven slots:

```
cap = COMMERCE_CAP[commerce_level]
cap = cap * (100 + BRITISH_COMMERCE) / 100          if British
cap = cap * (100 + <nation bonus for this resource>) / 100
cap = cap * (100 + DIAMONDS_COMMERCE) / 100         if diamonds
cap += <wonder bonuses for this resource>
cap += REPUBLIC_COMMERCE_BONUS{,2,3}                by republic level
cap += bonus_cap[t]
cap = clamp(cap, 0, 999)
cap *= 16
```

with two overrides that skip the whole body: **knowledge (slot 3) is always
999**, and so is everything if the player holds the Virtual Reality bonus.

`COMMERCE_CAP` ships as `70 100 150 200 260 320 400 500`, indexed by the
player's commerce level — one of four `epoch` counters on the encrypted block.
So a player at commerce level 0 cannot earn more than seventy of any capped
resource per thirty seconds no matter how many citizens are on it, and the last
commerce tech raises that to five hundred.

The clamp to 999 is why `over_cap` distinguishes 1 from 2: a cap of exactly 999
means uncapped, and the interface says so differently.

**Knowledge being exempt is the one asymmetry that shapes the whole game.**
Every other resource has a ceiling that a commerce tech has to lift; knowledge
does not, which is why scholars scale and farmers do not.

"Exempt" is exactly what the original means and no more, though: knowledge
skips `COMMERCE_CAP` and takes 999, which is still a number, and 999 still gets
its `* 16` and is still enforced by the same clamp in `do_gather`. Fifty
scholars in a maximum university earn 1250 and receive 999. That the ceiling is
twice the highest commerce cap is why nobody notices.

## Paying it out

`Leader::do_gather`, every frame, per available resource:

```
rate = resources[t] - support[t] + base_rate[t]
if rate < 0:
    income[t] = rate            # shown, not charged
    continue                    # a negative rate takes nothing away

if rate > resource_cap[t]:
    over_cap[t] = 1 + (resource_cap[t] > 15983)
    rate = resource_cap[t]
else:
    over_cap[t] = 0

<Dutch interest on the stockpile, itself capped>

income[t] = rate                # what the interface shows

rate = rate * (100 + gather_handicap) / 100
if t == KNOWLEDGE and tech_cost setting is high:  rate = rate * 3/4, or /2
if <fast-economy game option>:                    rate = rate * 3/2
if ai_speed > 1:                                  rate = rate * ai_speed

denom      = GATHER_RATE * 16
whole      = rate / denom
leftover[t] += rate % denom
while leftover[t] >= denom:
    whole += 1
    leftover[t] -= denom

bucket[t]    += whole
collected[t] += whole
```

Four things in that are worth stating plainly.

**A negative rate costs nothing.** It is displayed and then abandoned. Since
`support` is always zero, the only way to reach one is a negative `base_rate`,
and whatever writes that has not been read.

**`income` is captured before the multipliers.** The number on screen is the
capped rate, not the rate the player actually receives. On a hard difficulty
the two differ by a quarter.

**The `while` runs at most once**, because `rate % denom < denom`. The whole
block is arithmetically `leftover += rate; whole = leftover / denom;
leftover %= denom` — an exact accumulator with no drift. It is written the
original's way in the implementation, because the equivalence holds only while
`rate` is non-negative and the guard above is the only thing that makes it so.

**The handicap is a difficulty table**, from `LeaderData::get_gather_handicap`:

| Difficulty | 0 | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- | --- |
| Income | −35% | −15% | −7% | 0 | +25% | +50% |

There is no constant behind those six numbers; they are literals in the
function. They are also, notably, the only place in this pipeline where the
difficulty setting touches the economy.

`ai_speed` appears here as a plain multiplier on income, exactly as it appears
in `Guy::move` as a plain multiplier on the step — see `docs/MOVEMENT.md`. Two
subsystems now read the same global, and the question it raises there is the
question it raises here.

### Escrow

Below the accrual, guarded by a diplomacy flag, a second accumulator runs the
same arithmetic against `escrow_rate[t]` percent of the same rate, with
`GATHER_RATE * 1600` as its denominator, and adds the result to `escrow[t]`.
`escrow` is reset alongside `bucket` when the game starts everyone at zero. It
is not spendable and nothing read so far reads it back.

## Support was removed

`Leader::calc_support` writes six zeros and returns. `Leader::gather` zeroes the
`support` array on its own account every frame. Nothing else fills it.

`resourcerules.xml` still describes the mechanic in detail — every resource
carries `UNDISC_SUPPORT_GOOD`, `UNDISC_SUPPORT_RATE`, `OBS_SUPPORT_GOOD` and
`OBS_SUPPORT_RATE`, and `unitrules.xml` gives units a `SUPPORT` field with a
full cost grammar that `crates/rondata` already parses. The engine reads none of
it into the income path.

**So: there is no unit upkeep in Rise of Nations.** An army costs what it cost
to build and nothing thereafter. That is worth stating as a finding rather than
an absence, because it is the single largest structural difference between this
economy and every other RTS economy it will be compared to, and because a
reader with `resourcerules.xml` open would reasonably conclude the opposite.

By `docs/DECISIONS.md` entry 12 — a constant the original does not read is not
tuning — the support fields do not enter `Tuning` *here*.

**What they are actually for was settled afterwards, from the other side.**
`docs/COSTS.md` reads the cost path, where all of them turn out to be live: a
unit's `SUPPORT` is the ramp that makes each one more expensive than the last,
and the four `*_SUPPORT_GOOD`/`*_SUPPORT_RATE` fields decide which resource a
price is charged in when the one it is written in is not available yet. In this
engine's data vocabulary "support" means price, not upkeep — which leaves the
conclusion above unchanged and makes it sharper. The field a reader would take
for upkeep is the one that makes the eleventh hoplite cost double the first.

---

## What is not established

- **How many gatherers a building has, and how many it may have.**
  `BuildTypeData::calc_gather` walks the tiles in a radius, counts resource
  richness and river tiles, and produces both a slot count and a per-gatherer
  rate. Only its arithmetic tail is read: the per-gatherer rates, the enhancer
  scaling, and the `PEASANT_RATE`/`OIL_RATE`/`SCHOLAR_RATE` representation. The
  tile survey itself, `max_gatherers`, `total_gather_access` and the
  `MiningList` are unread. Everything downstream of a gatherer count is
  specified here; the count is not.
- **`base_rate`.** Added to every rate before the cap, never seen written.
- **`escrow_rate`.** What sets it. `escrow` itself is no longer open:
  `docs/COSTS.md` reads it in `Type::pay_cost` as a soft reservation that
  ordinary spending may not touch and abandons entirely the moment it needs to.
- **The two building types collected outside cities**, `0x1a2` and `0x1a3`.
- **Merchants and caravans.** `Unit::do_gather` on an idle merchant, and
  `MERCHANTS_BONUS`, are how rare resources pay. Unread.
- **Rare resources.** `LeaderData::calc_rare`, forty-four of them, each with
  its own constant in `rules.xml`.
- **The market.** `MARKET_BASEMENT`, `MARKET_EQUILIBRIUM`, `MARKET_CYCLE_RATE`
  and the rest describe a price simulation with supply and demand. Entirely
  unread, and the only part of the economy that is not a sum of rates.
- ~~**Costs.**~~ and ~~**Population.**~~ Closed by `docs/COSTS.md`, which
  specifies `UNIT_COST_FACTOR` and its siblings, the ramp, the
  unavailable-resource redirect, the discount tail, `Leader::can_pay` and
  `Leader::calc_pop_cap`. What remains open there is the *production* half:
  `Build::queue_up`, `JOB_TIME` and when in an item's life the price is
  actually charged.

---

## Second reading (2026-08-20) — corrections owed

Blind second derivation and adjudication: `docs/audit/2026-08-20-economy.md`.
The frame order, the gather order, the 8/512-frame cadence, sixteenths and the
7200 denominator, the ×256 trio, the assembly order, city flat/taxes/literacy,
the per-gatherer truncation, the cap pipeline and `do_gather`'s arithmetic are
**doubly confirmed**; no number in `crates/sim/src/economy.rs` changes and the
existing tests stand. Nine behaviour-relevant points went against this
document, all about *where* something happens or *who* it applies to:

- Caravans pay through `City::compute_trade` → `trade_val` inside
  `calc_city_resources`, not `Unit::do_gather`; the `calc_rare` loop is
  Conquer-the-World-only, and owned rares pay through idle merchants and
  fishermen.
- The difficulty handicap is AI-only (`get_gather_handicap` returns 0 for a
  human); `bonus_cap` and `base_rate` are scenario-script writers only, and
  `resource_cap_add` writes `resource_cap` directly.
- The AI escrow guard is `(flags & 0xc) != 4` and rounds by frame modulus; it
  is not an accumulator.
- Forbidden City's base gather only replaces non-zero `CITY_GATHER` slots, and
  its ×125% precedes flat/taxes/literacy.
- The city `refinery` byte is always 0 — a smelter must be present — so
  `City.refinery` in the implementation can double-scale oil and should go.
- The flat per-gatherer rate is `(Σ num_make × PEASANT/OIL_RATE) >> 8` over
  the footprint, river ×2, non-ally tcoords excluded; 160/560 is the
  sum == 16 case. (The second reader was itself slightly off here: unowned
  tcoords count.)

Resolved for this document: enhancer arrays are `int[5]` with 1-based levels,
so `fishermen_bonus[lvl+4]` is `GRANARY_BONUS[lvl-1]` as written here.
