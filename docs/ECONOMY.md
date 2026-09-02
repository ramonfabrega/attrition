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
the per-gatherer rates, which are read at their consumers. ~~**Lower for the
gatherer *count*:** `BuildTypeData::calc_gather` is a thousand lines of inlined
tile walking and only its arithmetic tail has been read.~~ The count is read
and verified as of 2026-08-24 (`crates/sim/src/gather.rs`; the last item
below). What remains open is listed at the end.

A blind second reading (`docs/audit/2026-08-20-economy.md`) doubly confirmed
every formula in that spine and overturned several of the *predicates* around
them — where caravans pay, where rare resources pay, who the difficulty
handicap applies to, which enhancer bytes are ever non-zero. **No number
changed.** Those corrections are landed below and marked where they changed
what an earlier draft said.

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
`Leader::do_gather` skips it entirely until it is. **Only the accrual is
skipped**, though: the rate is still assembled — `calc_gather` has no per-good
availability gate except the Americans' bonus and the obsolete pass — and the
cap is still computed, since `calc_resource_caps` has no gate at all. So an
unavailable resource has a live rate and a live cap that nothing ever pays out.
(An earlier draft said "no rate, no cap, no accrual"; only the last is true.)

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
| `LeaderData` | `base_rate` @ +0x4b0 | `i32[6]` | — | Added to the rate before the cap. Scenario-script only, and stored `<< 4` |
| | `escrow` @ +0x468 | `i32[6]` | — | A second pile, fed at `escrow_rate` percent |
| | `escrow_rate` @ +0x480 | `i32[6]` | — | Percent of income diverted to escrow |
| | `collected` @ +0x874 | `i32[6]` | — | Lifetime total, for the score screen |
| | `bonus_cap` @ +0x918 | `i32[7]` | — | A flat addition to the cap. Scenario-script only |
| | `rare_owned` @ +0x6dac | `BitMask<44>` | — | Rares a merchant is standing on, rebuilt each recompute |
| | `rare_conquest` @ +0x6dc0 | `BitMask<44>` | — | Conquer-the-World conquest rares |
| | `gather_stamp` @ +0x7ac | `i32` | — | Frame of the last rate recompute |
| | `territory` @ +0x9d8 | `i32` | — | Owned tiles, for the territory tax |
| | `city_mark` @ +0x408 | `i32` | — | Number of cities; the loop bound |
| | `oil_well_mark` @ +0x430 | `i32` | — | Number of oil wells; the loop bound |
| `CityData` | `trade_val` @ +0x52 | `i16` | — | Caravan income, in ¹⁄₁₆ units; see below |
| | `granary` @ +0x56 | `u8` | — | Food enhancer, as a percentage bonus |
| | `lumber_mill` @ +0x57 | `u8` | — | Timber enhancer |
| | `smelter` @ +0x58 | `u8` | — | Metal enhancer |
| | *(oil enhancer)* @ +0x59 | `u8` | — | Read for oil, written zero unconditionally |
| `WorldData` | `land_size` @ +0x78 | `i32` | — | Total land tiles; the territory tax divisor |

`resource_cap` and `bonus_cap` are seven long where six would do, and the loop
in `Leader::calc_resource_caps` really does run seven times. The seventh slot is
written and never read.

**`base_rate` and `bonus_cap` have exactly one writer each, and it is the
scenario scripting layer:** `ScenarioFuncSet::set_base_rate` and
`ScenarioFuncSet::set_bonus_cap`. Nothing in a skirmish touches either, so both
are zero in ordinary play. `set_base_rate` writes `value << 4`, which fixes
`base_rate` as sixteenths like everything else on the rate path.
(An earlier draft listed both as "never seen written" and additionally
attributed `bonus_cap` to `LeaderData::resource_cap_add`. It does not: that
function — Angkor Wat's metal bonus is its caller — decodes `resource_cap`,
adds, and re-encodes, never touching `+0x918`. The distinction matters because
`bonus_cap` is added *after* the percentage multipliers and `resource_cap_add`
lands wherever in the sequence its caller sits.)

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

(All three confirmed against the program's own `CONSTANTS` dump, 2026-08-20,
`docs/DATALAYER.md`: `peasant_rate 2560`, `oil_rate 8960`, `scholar_rate
1280 1792 2560 3840 5120`. The dump writes five of `scholar_rate`'s six —
the symbols say `int[6]`, `LeaderData::get_university` goes to 6, and the
fifth line of the log is a logging count, not a loader one.)

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
5. **Buildings outside a city** — the Woodcutter's Camp (`0x1a2`) and the Mine
   (`0x1a3`), via the same call. The filter is: the object is active, its
   `city` link is negative, it is not neutralized, and its type is one of those
   two.
6. **Every idle fisherman and merchant**, via `Unit::do_gather`. The test is
   `type == 0x13d` (Fisherman) *or* `UnitData::is_merchant`, and
   `UnitData::order_type == NONE` — a merchant walking somewhere earns nothing.
   **This is also where every owned rare resource pays**; see step 9.
7. **Refineries**: `oil = oil * (refineries * REFINERY_BONUS + 100) / 100`.
   `REFINERY_BONUS` is `33% per refinery` and this is where "per refinery"
   lives; every other enhancer is a per-city percentage.
8. **Capitalism** adds `CAPITALISM_OIL_PROD * 16` to oil.
9. **Conquest rares** — one `LeaderData::calc_rare` per bit of
   `rare_conquest`, and only when no other in-game leader exists. See below.
10. **Coffee** scales all six.
11. **The territory tax** (below).
12. **`LeaderData::calc_resource_bonuses`** — the wonder and nation percentage
    multipliers, in this order: Russians on oil, Pyramids on food, Colossus on
    wealth, Hanging Gardens as a flat `HANGING_GARDENS_KNOWLEDGE * 16` addition
    to knowledge, Angkor on metal, Taj on wealth, Eiffel on oil, Tikal on
    timber, then the Virtual Reality bonus scaling all five capped resources by
    `GLOBAL_PROSPERITY` (knowledge is skipped), then a per-good Conquer-the-World
    bonus.
13. **The obsolete redirect** (below).

### Rare resources pay through merchants, not through ownership

Step 9 is not "one `calc_rare` per rare you own", which is what an earlier
draft of this document said and what a reader would reasonably expect. The
loop there walks `rare_conquest` — the Conquer-the-World campaign's *conquest*
rares — and it is reached only when no other initialised in-game leader
exists, so in a skirmish it never runs at all.

Owned rares pay in **step 6**. `Unit::do_gather` on an idle merchant or
fisherman calls `UnitData::calc_gather`, which calls `calc_rare` for the rare
the unit is standing on and sets that rare's bit in `rare_owned` on the way
past. `rare_owned` is cleared at the top of every recompute and rebuilt from
the units, which is why **a rare with no merchant on it pays nothing** — the
bonus is the merchant, not the deposit. The one exception is the Porcelain
Tower, whose pass at the top of `calc_gather` sets `rare_owned` bits directly
for every rare inside the player's own territory.

After the rare terms, `rare_owned | rare_conquest` is compared against the
previous frame's `rare` mask; when it differs the player's population cap is
recomputed, the borders are marked for a redraw if the rare that changed is
one of the territory-affecting ones, and two more dirty bits go up.

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

Three modifiers sit on it, all read: the British scale `TERRITORY_TAXES` by
`(BRITISH_TAXATION + 100) / 100`, which as shipped doubles it; a Conquer-the-World
conquest bonus scales it by `CTW_MISSIONARIES_BONUS`; and the Mongols
additionally take **food** from the same ratio, `num_nations * territory * 800
/ land_size / MONGOL_NOMADIC_FOOD`, which is the one term in the economy that
reads how many players are in the game.

### The obsolete redirect

The last thing `calc_gather` does is walk the six goods looking for one that is
*no longer available* but whose prerequisite the player still holds. For each
such good it moves the whole rate somewhere else and zeroes it:

```
if obs_prod_good >= 0:
    out[obs_prod_good] += out[t] * obs_prod_rate >> 8
out[t] = 0
```

The guard matters: a good with no redirect target still has its rate zeroed.
And the shift is the sign-corrected kind — `(x + (x >> 31 & 0xff)) >> 8`, which
truncates toward zero rather than down, the same idiom the rest of the engine
uses wherever an 8.8 value can go negative.

`obs_prod_good` and `obs_prod_rate` are `OBS_PROD_GOOD` and `OBS_PROD_RATE`
from `resourcerules.xml` — Food's are `Wealth` and `1/2`. The `>> 8` is what
fixes `OBS_PROD_RATE` as 8.8 fixed point, so `1/2` is stored as 128 and half
the food income becomes wealth.

Nothing in a standard game makes a basic resource obsolete, so this never fires
in normal play. It is recorded because it closes a data claim with its
consumer, which is the only way this project is willing to assert a field's
meaning.

### Availability, and where a price lands instead (2026-09-02)

`Holdings::available[6]` and `Holdings::discovered[6]` are the two arrays the
whole cost layer steers by — `pay` accrues nothing into a good that is not
available, and `cost::charges` writes a zero there and redirects the price
(`docs/COSTS.md`, "Three of the six resources are not available from the
start"). **Both were all-true from frame 0 for as long as they existed**, so
the redirect layer under them — read, tabled and unit-tested against
hand-built arrays — never fired in a played game.

The test is `LeaderData::type_avail(good, 1)`, and `TypeData::can_pay_cost@
00667570` is where to read it: its loop over the six goods opens with exactly
that call before it asks `get_cost` anything. `docs/TECH.md` ("Two questions
this answers for the other documents") settles what it comes to for a good —
`has_preq`, whose slot 0 is the unlocking age and whose slot 1 is nothing,
with `obs` never set, and a `type_eligible` that is always 4. **So
`available` and `discovered` are the same test for a good**, which is why
`Redirects::of` never reaches the obsolete table in a stock game.

`Sim::sync_goods_available` writes them, from `set_tech_tree` for a player
who never starts and from `apply_gained` on every gain after that; the goods
are the first six `Kind::Good` entries of the tree, in `resourcerules.xml`
order.

What it costs the AI: Knowledge and Metal carry the Classical Age and Oil the
Industrial, so through the whole Ancient age a knowledge price is charged as
food at three halves. **Coinage** is `14k/6t`, and the East Indies AI takes it
in its library on frame **5177** — two hundred and ten food and sixty timber,
where this crate had been asking for a hundred and forty knowledge it could
not hold and refusing the job on every frame to the end of the capture. The
twenty-four rows of `1/2005 queued ours 0 theirs 1` in
`diff::tests::run58_s_five_thousand_frames…` are what that was, and they are
gone.

**What this does not close.** A leader still *starts* with a hundred of each
of the three, where the original starts with none: `Leader::gain_tech@
006dcb60` walks the six goods on every gain and, for one whose bucket is zero
and whose `goodtypes[g] + 0x30` prerequisite is the tech just gained, calls
`bucket_add(g, game->starting[g])` — the starting grant arrives **with the
age**, not at `Leader::init`, which zeroes all six. It is inert while the
good is unavailable (nothing accrues into it and nothing is charged from it)
and it is the 240 rows `run40_s_census_…` books.

## What a city gives

`LeaderData::calc_city_resources` sums five things for one city.

**`trade_val`, into wealth, before anything else.** The very first thing the
function does is add the city's own `trade_val` — `CityData + 0x52`, a `short`
already in sixteenths — to wealth. **This is where caravan income arrives.**
(An earlier draft put caravans in the idle-unit loop alongside merchants. They
are not in `Leader::calc_gather` at all; a caravan on a route has an order, and
the loop takes only idle units.) See below.

**Its buildings.** (`CityData::num_buildings`, where the taxes and trade value
read it, walks the member chain from the city building itself, so **the city
counts as one of its own buildings** — `docs/CITIES.md` §5.5.) Every gathering
building attached to the city contributes
through `BuildData::calc_gather`. Two building types are excluded by identity,
and the general shape of the contribution is `per-gatherer rate × gatherers`,
with the per-gatherer rate scaled by the city's enhancer for that resource
before the multiply.

Where the per-gatherer rate comes from depends on which of four branches the
building takes, and **only three of them are the land-independent constant this
document originally described**:

| Building | Slots | Per gatherer, in ¹⁄₁₆ |
| --- | --- | --- |
| Mine | `MountainRangeData::gather_size` (+ German, Taj, Kremlin) | `(PEASANT_RATE >> 8) << 4` = 160 |
| Woodcutter's camp | from the tile survey, `(sum + 8) >> 4` | 160, or `IROQUOIS_FOOD * 16` for the Iroquois' food |
| University | 7 | `(SCHOLAR_RATE[level - 1] * 16) >> 8` |
| **Flat** (farm, oil well, oil platform) | 1 | `(Σ num_make × PEASANT_RATE or OIL_RATE) >> 8` |

The flat branch is the correction. Its per-gatherer rate is not a constant: it
walks the building's whole footprint — `[cx, cx + X_SIZE) × [cy, cy + Y_SIZE)`
in tcoords — summing each tcoord's `num_make` richness for the land under it,
doubling a river tcoord by `RIVER_RESOURCE_VALUE`, and **skipping any tcoord
owned by somebody who is neither the owner nor an ally.** Unowned ground counts
normally. **160 is what that comes to for a farm whose sixteen tcoords each
make one food**, which is the ordinary case and the reason the constant looked
land-independent; a farm straddling an enemy border earns strictly less, and an
oil well on a rich site earns more. (An earlier draft stated 160 and 560 as the
rule. They are one case of it.)

Two further terms live in the same branch: the Japanese scale food by
`(JAPANESE_FISHING_BOATS + 100) / 100`, and the Egyptians add
`EGYPTIAN_FARM_WEALTH * 16` of wealth per farm. The mine branch has one of its
own — the Inca earn `gatherers * INCA_WEALTH_PER_MINER * 16` of wealth per
mine, when that constant is positive. When it is *negative*, a separate branch
back in `Leader::calc_gather` instead adds the player's whole metal rate to
their wealth rate, which is a strange enough way to spell "the Inca sell their
metal" that it is recorded here rather than explained.

The flat branch is also the only one that does not bound gatherers by slots:
it adds `per × n` for whatever `n` it was handed, where the mine, woodcutter
and university branches all take `min(slots, n)` — and the woodcutter
additionally caps at twice its `total_gather_access`.

**`CITY_GATHER[t] * 16`, once per city.** Ships as `10 food, 10 timber` and
zero for the rest — so **a city is worth ten food and ten timber per thirty
seconds before anybody works in it.**

A Forbidden City changes that in two ways, and both are narrower than an
earlier draft of this document said.

- The `FORBIDDEN_CITY_BASE_GATHER` substitution happens **inside the loop's own
  `CITY_GATHER[t] != 0` guard**, so it replaces only the slots that already
  pay. With shipped data that is food and timber and nothing else: a Forbidden
  City is worth fifty food and fifty timber, not fifty of all six. (It also
  needs `FORBIDDEN_CITY_BASE_GATHER` to be non-zero, or the ordinary value is
  used.)
- The `FORBIDDEN_CITY_GATHER` percentage — 25%, applied as
  `(pct + 100) * out[t] / 100` to all six — runs **immediately after the
  building walk and before everything below it.** So it multiplies the
  gatherers and nothing else: not the flat city gather, not the Roman or German
  per-city bonuses, not taxes, not literacy.

Between those two sits one more multiplier of the same shape: a CEO hero
(`TypeIndex 0x165`) standing in the city scales all six by
`THECEO_PRODUCTION_BONUS`, with the same reach as the Forbidden City's — the
building walk only.

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

The market term has one modifier, which lives inside `get_taxes` rather than in
the wonder layer: a player holding the **Porcelain Tower** takes
`MARKET_TAXES * (PORCELAIN_MARKET + 100) / 100` instead. It is per city with a
market, so it scales with how many the player has.

**Literacy, into knowledge.** `CityData::get_literacy` is the same shape:
`VILLAGE_LITERACY` (0) plus `UNIVERSITY_LITERACY` (10) if the city has a
university plus `LIBRARY_LITERACY` (0) if it has a library. **A university is
worth ten knowledge per thirty seconds**, and a library on its own is worth
nothing — the library's value is that scholars can live in it.

Then the Romans add wealth per city and the Germans add food, timber and metal
per city.

### Caravans, and the city they are cached in

`trade_val` is not recomputed with the rate. `City::compute_trade` rebuilds it
when a trade route starts or ends, walks the city's caravan links, and for each
one whose caravan is active and whose *both* endpoint cities are active adds:

```
trade_val += (Caravan::trade_value(a, b) * 16) / 2
```

then, if the total changed, sets the owner's `0x2000000` dirty flag — so a new
trade route makes the income display move within eight frames, by the same
route a new farm does.

`Caravan::trade_value` is:

```
v = get_trade_value(a) + get_trade_value(b)
if distance != 0:  v = (distance + 3) * v / 3
if owners differ:  v = v * 3 / 2
Indians:           v = (INDIANS_CARAVAN   + 100) * v / 100
Spice:             v = (SPICE_CARAVAN_INCOME + 100) * v / 100
```

`CityData::get_trade_value` is the city's building count plus 2 for a Large
City, plus 4 for a Major City or a Forbidden City. `Caravan::distance` is a
four-step bucket on world width `W`: 0 under `W/4`, 1 under `W/2`, 2 under
`4W/5`, else 3 — so the multiplier runs 1, 4/3, 5/3, 2. **A long international
route between two big cities is worth roughly four times a short domestic one
between two small ones**, and the whole thing is halved on the way into
`trade_val` because both endpoint cities compute it and each keeps half.

None of this is implemented; trade routes are not modelled yet.

### The enhancers, and an array that runs off its end

`CityData::enhancer_amount(t, base)` returns `(pct[t] + 100) * base / 100`,
where `pct` is the city's `granary` / `lumber_mill` / `smelter` byte. Wealth
and knowledge have no enhancer and take the default zero.

**So does oil.** There is a fourth byte at `+0x59` that `enhancer_amount` would
read for oil, and `City::calc_gather` writes it as an unconditional zero — it
calls `count_buildings` for the oil enhancer first and throws the answer away.
A city therefore never scales oil; refineries act nation-wide instead, through
step 7 of the assembly. (An earlier draft called this byte the oil enhancer and
`crates/sim` carried a `City::refinery` field a caller could set. Anyone who
had set it would have counted their refineries twice. The field is gone.)

The three that do exist are each gated, and the gates differ:

- **granary** on the city's flag `0x200`,
- **lumber mill** on the city's flag `0x400`,
- **smelter** on the metal enhancer building actually standing in this city —
  `count_buildings(get_enhancer(METAL))` non-zero. So the smelter, alone of the
  three, is a building the city must contain rather than a bit it must carry.

`City::calc_gather` fills the three bytes from three constant arrays, and it
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

## What a finished gather building pays

`Build::activate@00623e20` ends with a block that has nothing to do with
`calc_gather` and everything to do with income, and it was missing here until
2026-08-30. It is what puts twenty food in the AI's hands on run40's frame 166.

The block runs for a **gather building** — `BuildTypeData::is_gather_type`, a
`build_flags & 0x40` — and does two things in order.

**One: the slots join the leader's running count.**

```
good = BuildTypeData::get_good(type)
gather_slots[good] += (signed char) BuildData::gather_max
```

`get_good@0063bd50` is a six-entry jump table at `0063bd84`: Farm (`0x1a1`) is
food, the Woodcutter's Camp (`0x1a2`) timber, the **Mine (`0x1a3`) metal**, the
University (`0x1a4`) knowledge, and the Oil Well and Platform (`0x1a5`,
`0x1a6`) oil. Anything else answers −1, and the write then lands on
`gather_slots[-1]`, the int in front of the array — the same running-off-the-end
idiom `GRANARY_BONUS` and `ATTRITION_UPGRADE` are reached by. No shipped
gather type takes that arm.

`Build::close@00628980` line 104 is the mirror, inside the same `flags & 4`
guard that gates the wall stats: an **active** gather building subtracts its
slots again as it goes. So `gather_slots` is an inventory, not a total.

**Two: the part of them nobody has held before is paid for.**

```
fresh = gather_slots[good] - gather_slots_high[good]
if fresh <= 0:  (a farm still calls Farms::add_animals) and stop
gather_slots_high[good] = gather_slots[good]
if frame == 0 or captured or not counted or <loading>:  stop
do_bonus(good, amount)
```

The parenthesis is not an aside. **Every** exit above lands on the same
label as `do_bonus(0, FOOD_BONUS_FOR_FARM)`'s fall-through, so a finished
**farm** runs `Farms::add_animals` whatever the bonus did — and a farm whose
`farm_type` is the pasture is stocked with five animals there, twenty draws'
worth. That call was missing here until 2026-08-31 and it was Great Lakes'
word: `docs/SYNC.md` §3.21.

`gather_slots_high` never falls, which is the whole rule: **the bonus is paid
once per slot in the life of a player**, so a farm rebuilt where one was razed
is worth nothing, and a camp on a richer patch pays only for the slots the last
one did not have.

The amounts are two shapes, and the constants' names do not give the split
away:

| Building | Good | Pays |
| --- | --- | --- |
| Farm | food | `FOOD_BONUS_FOR_FARM` = 20, **flat** |
| Woodcutter's camp | timber | `TIMBER_BONUS_PER_WOOD_SLOT` = 5 **per new slot** |
| University | knowledge | `KNOWLEDGE_BONUS_FOR_UNIVERSITY` = 25, flat |
| Mine | metal | `METAL_BONUS_PER_MINE_SLOT` = 5 **per new slot** |
| Oil well or platform | oil | `OIL_BONUS_FOR_WELL` = 50, flat |

Wealth has no case: the original's second switch — on the *good*, not the
type, at `006259b5` — falls through for 2, and `Build::do_bonus` is not
reached. `do_bonus` itself is one line of arithmetic and a message: the
Germans (`has_tribe_bonus(0xc)`) scale the amount by
`(GERMAN_COMPLETION_BONUS + 100) / 100`, and then `bucket[good] += amount`.

There is a **third** `do_bonus`, earlier in `activate` and not part of this
block: thirty wealth (`do_bonus(2, 0x1e)`) the first time a player finishes a
kind of building they have never finished before, off a separate pair of
counters at `+0x8ac`/`+0x8dc`. It is unread and unimplemented; no traced game
has reached it, since both players start with every building they own.

**How it is established.** The block is read; the *amount* is diff-backed.
run40's census has the AI's food thirty-two behind the original's on every
frame of `[560, 600)` while its `leftover` — the fractional accumulator —
agrees on all forty, which can only be true if the gap is a lump. Twenty of
the thirty-two is this bonus, on the frame the AI's fourth farm finishes; the
other twelve are `Build::refund_cost`'s (`docs/COSTS.md`). With both, the AI's
food is the original's on every frame of the window, and run33's word runs
776 → 780.

**What it does not establish.** ~~`gather_slots` itself is still wrong in the
harness on the camps: `Build::init` surveys a camp's slots against its own
still-empty `gather_from`~~ — that is "The gather list, and its shuffle"
below, landed 2026-09-01: `Build::find_gather_tiles` is implemented, and a
camp placed during a run fills, marks and shuffles its own list. And run40's
human files **one slot under good 2**, which `get_good`
cannot produce — `Leader::plan_strategy@006b9620` line 1137 assigns the whole
array from `City::count_gather_slots` and raises the high-water to match, and
that second writer is unread. Both are booked in the queue; the run40 diff
asserts the disagreement as it stands so that fixing either moves an
assertion.

## The gather list, and its shuffle

`BuildData::gather_from` — the `MiningList` at `+0x98` — is the tile list a
camp's citizens work, and the thing "What a finished gather building pays"
surveys its slot count *out of*. It is not read from the map on demand: it is
built once, at **placement**, and it is the one part of a building's creation
that costs the sync stream.

`Build::init@00629740` clears the list (`+0x9c = 0`) and then branches on the
type, in this order:

```
if is_gather_type(type):                       # build_flags & 0x40
    if not is_flat(type) and not is(0x1a4):     # not a farm/oil well, not the university
        find_gather_tiles(this)                 # fills the list, then sets gather_max from it
    else:
        gather_max = max_gatherers(type, o, who, corner)
```

So a farm, an oil well, an oil platform and the university take the plain
survey — `max_gatherers` answers 1, 1, 1 and 7 without a list — and only the
**Woodcutter's Camp and the Mine** go the long way round.

### `Build::find_gather_tiles@00623350`

Four steps, and the third is the one a diff can see:

1. `BuildTypeData::find_gather_tcoords` appends the tiles (below).
2. Every tile of the list gets the world mask's `0x1000`
   (`is_gathered_from`), so the next camp cannot take the same ground. That
   bit is why the *second* camp on a patch is worth less than the first.
3. **If the list grew**, `4 × length` rounds of: draw
   `Random::get(game_random, 0, 0xffff) % length`, remove that entry, append
   it at the back. One draw a round, off the sync stream, and the round is
   skipped without a draw when `length ≤ 1` (`GameAccess::rnd`'s early
   return). The list is left shuffled, and `Unit::do_non_flat_gather` ranks
   tiles by `i >> 2`, so this ordering is the order the ground is worked in
   (`docs/ORDERS.md` §6.1).
4. `gather_max = max_gatherers(…)`, now against the filled list.

The removal is by **value** in the original (`Array<TCoordData>::remove`);
the list holds each tile once — a cell is walked once and cells do not
overlap — so removing by index is the same operation.

`Build::process@0061edf0` is the second caller: a region carrying `0x10`
re-runs `find_gather_tiles`, and one carrying `0x20` runs
`verify_gather_tiles`. Neither is modelled; the region flags are not.

### `BuildTypeData::find_gather_tcoords@0063bdc0`, the timber branch

The same walk `calc_gather`'s survey takes — the ring `(radius + 3) / 4` of
the octagonal cell spiral, the `vector_dist ≤ radius` test against the
footprint's centre tile, the owner test, and the `0x1000` skip on the cell's
centre tile — and inside a cell that passes, it adds the cell's **tree**
tiles that do not themselves carry `0x1000`, in `(i % 4, i / 4)` order.

The decompiler prints that inner loop with only the `0x1000` test in it,
which would make every qualifying cell worth all sixteen of its tiles. The
record says otherwise and says it twice: run39's two camps each list **73**
tiles across **six** cells — 16, 12, 12, 12, 12, 9 — and those six cells hold
exactly 16, 12, 12, 12, 12 and 9 forest tiles, with every listed tile among
them. Sixteen-a-cell would be 96.

The **metal** branch is a different walk altogether — `MountainsData::
find_nearest` / `CliffsData::find_nearest`, then the range's or the cliff's
own tiles, skipping forest, enemy-owned and already-gathered ones, and
stamping `MiningList::mtn`/`::cliff` so the search is not repeated. It is
**not modelled**: a mine placed during a run gets an empty list, which is
what it got before any of this existed.

### How it is established

- **The tiles.** `find_gather_tiles_rederives_run39_s_camp_lists`
  (`rondata::diff`) clears the `0x1000` marks off each of run39's two
  pre-placed camps and runs the walk at its corner: it returns **exactly**
  the dump's 73 tiles, as a set, for both camps, on a map neither the walk
  nor the survey had seen.
- **The shuffle's count.** run54's trace spends **584** draws at
  `Build::find_gather_tiles+0x10a < Build::init+0x55b < Objects::init_build
  +0x82` during **setup**, where the two 73-tile camps are placed:
  `4 × (73 + 73) = 584`, to the draw. Its four later bursts are 192, 184,
  680 and 248 — all divisible by four — on frames 2176, 10582, 10982 and
  16382.
- **The list, every frame.** `run39_s_mining_lists_are_the_gather_record`
  and its Great Lakes sibling compare `gather_from` entry for entry, plus
  the `MiningList` header's `length` and `BuildData::gather_down`, for every
  building of both players on every frame: 594,618 and 584,712 fields, and
  the only disagreements are the four the quit's own last block writes.
  run56 carries the same comparison to 3,001 frames and **1,048,118**
  fields, and since 2026-09-01 the only rows are the quit's own four:
  the camp below is a camp this crate's AI builds too.
- **The order, seed-anchored.** `run56_s_new_camp_is_this_crate_s_own_shuffle`
  is the whole of it against the one camp the *game* built. run56's frame
  2176 places player 1's `o 2009`, 48 tiles, 192 draws; step there, install
  the original's own word from the trace — `find_gather_tiles` is the
  **first** thing that frame draws, so the anchor is exact — place the camp
  where the original placed it, and the list comes back **entry for entry**
  in the original's shuffled order, for exactly `4 × 48` draws. That is the
  walk's order, the marking, the round count and the modulus in one
  assertion, and it is the half the re-derivation above cannot reach.

**A second reader of the count.** `BuildTypeData::blocked_location` runs the
same survey and hands the count out through `blocked_site`'s last parameter
(`docs/CITIES.md` §2.6.7). That is what refuses a camp with nothing under it,
and it is what `Leader::produce_building` scores a camp *site* by — the cube
of it (`docs/AI.md` §19). So the count is no longer only `gather_max`'s: it
decides where the AI's camps go, and run56's frame 2176 is the diff that says
so.

**What it does not establish.** The **metal** branch, above. And
`Build::process`'s two re-entries — `verify_gather_tiles` on a region's
`0x20`, `find_gather_tiles` again on its `0x10` — which no run has been
seen to take, because the region flags are not modelled at all.

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

**The first two lines are diff-backed, 2026-08-30.** run40's `LEADERDATA`
prints `resource_cap`, and the AI — the British, `tribe 11` — carries **1392**
on every frame of the window against the human's 1120. `70 × 125 / 100` is
87.5, and 87 × 16 is 1392: the truncation is per-percentage and *before* the
`<< 4`, which is what fixes the order of the whole pipeline. The British term
and the three per-resource nation terms are implemented
(`sim::economy::commerce_cap`).

**And the British term now fires.** `Holdings::british` is
`Nation::british`, which nothing set until the harness read the dump's own
`LeaderData::tribe` — the roster index, 11 here and 4 for the Nubian human
(`crates/sim/src/nations.rs`, `docs/TECH.md`). All 200 of run40's
`resource_cap` disagreements went with it, and both players' whole cap is
now the original's on all forty frames. It changed nothing else on any
capture, which is what the arithmetic predicts: the AI's largest rate in
that window is 800, well under either ceiling, so a cap that was 272 too
low still never bound.

with two overrides that skip the whole body: **knowledge (slot 3) is always
999**, and so is everything if the player holds the Virtual Reality bonus.

Filled in, the modifiers are: British `BRITISH_COMMERCE` on every slot; then
one nation term per resource — Egyptian food, French timber, Inca wealth;
Diamonds (rare bit 22, in either `rare` or `rare_conquest`) on every slot;
then the flat wonder additions — Pyramids on food and wealth, Colossus on
timber and wealth, Taj on wealth, Eiffel on oil, Kremlin on food, timber, metal
and oil but *not* wealth, Tikal on timber, and Angkor on metal via
`resource_cap_add`; then the republic term, which takes the highest tier held
rather than summing; then `bonus_cap`; then the clamp and the `<< 4`.

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
if <infinite-resources option>:
    bucket[t] = 99999; escrow[t] = 0; continue

rate = resources[t] - support[t] + base_rate[t]
if rate < 0:
    income[t] = rate            # shown, not charged
    over_cap[t] = 0
    continue                    # a negative rate takes nothing away

if rate > resource_cap[t]:
    over_cap[t] = 1 + (resource_cap[t] > 15983)
    rate = resource_cap[t]
else:
    over_cap[t] = 0

<Dutch interest on the stockpile, itself capped — below>

income[t] = rate                # what the interface shows

rate = rate * (100 + gather_handicap) / 100
if t == KNOWLEDGE and tech_cost > 4:  rate = rate * 3/4 if tech_cost < 7 else rate / 2
if <fast-economy game option>:        rate = rate * 3/2
if ai_speed > 1:                      rate = rate * ai_speed

denom      = GATHER_RATE * 16
whole      = rate / denom
leftover[t] += rate % denom
while leftover[t] >= denom:
    whole += 1
    leftover[t] -= denom

bucket[t]    += whole
collected[t] += whole
```

Five things in that are worth stating plainly.

**A negative rate costs nothing.** It is displayed and then abandoned. Since
`support` is always zero, the only way to reach one is a negative `base_rate`,
which only a scenario script can write.

**Two options short-circuit the whole thing.** With the infinite-resources
setting (`starting_resources == 8`) the stockpile is *assigned* 99999 and the
escrow zeroed, every frame, per resource: nothing accrues because nothing needs
to. And the knowledge penalty from a high tech-cost setting has two steps, not
one — three quarters up to setting 6, a half from 7. Both truncate toward zero
the way the rest of the pipeline does.

**`income` is captured before the multipliers.** The number on screen is the
capped rate, not the rate the player actually receives. On a hard difficulty
the two differ by a quarter.

**The `while` runs at most once**, because `rate % denom < denom`. The whole
block is arithmetically `leftover += rate; whole = leftover / denom;
leftover %= denom` — an exact accumulator with no drift. It is written the
original's way in the implementation, because the equivalence holds only while
`rate` is non-negative and the guard above is the only thing that makes it so.

**The handicap is a difficulty table, and it is for the AI**, from
`LeaderData::get_gather_handicap`:

| Difficulty | 0 | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- | --- |
| Income | −35% | −15% | −7% | 0 | +25% | +50% |

There is no constant behind those six numbers; they are literals in the
function. They are also, notably, the only place in this pipeline where the
difficulty setting touches the economy.

**A human never reaches that table.** The function's first act is to test the
human flag (`leader_flags & 4`, which is exactly `LeaderData::is_human`) and
return **zero** — unless the multiplayer-handicap option is on, in which case a
human takes their own `get_handicap()` from the per-player handicaps instead
and still never sees the difficulty table. (An earlier draft presented the
table as the whole function, and this document's tests apply it to an ordinary
player. It is not wrong as arithmetic — the handicap is an input — but it
invites the wrong reading: "hard" does not mean you earn 100% and the AI earns
100%; it means you earn 100% and every AI earns 125% or 150%. On the easiest
setting the AI earns 65%.)

For an AI, which difficulty the table is indexed by has its own small tangle:
the leader's own `multi_diff` when the multiplayer-handicap option is on or
when a lobby setting says so, and the game's global `info.difficulty`
otherwise.

`ai_speed` appears here as a plain multiplier on income, exactly as it appears
in `Guy::move` as a plain multiplier on the step — see `docs/MOVEMENT.md`. Two
subsystems now read the same global, and the question it raises there is the
question it raises here.

### Dutch interest

Between the cap and the `income` capture sits the one term that reads the
*stockpile* rather than the rate. For a Dutch player and any resource but
knowledge:

```
start  = <this player's starting amount for t>
excess = bucket[t] - start
if excess > 0:
    rate = rate + (DUTCH_INTEREST * excess / 100) * 16
    rate = min(rate, DUTCH_INTEREST_CAP * 16 + resource_cap[t])
rate = min(rate, 16000)
```

So the Dutch earn interest on savings above what they started with, capped
twice — once by `DUTCH_INTEREST_CAP` above the ordinary commerce cap, and again
by a flat 16000 sixteenths (a thousand a rate) that exists **only inside this
branch**. Nobody else in the game is subject to it. `start` is the game's
starting amount, scaled by the starting-resources setting in a team game and by
`CTW_NOMAD_STARTING_RES_X` for a Conquer-the-World nomad start.

### Escrow

Below the accrual, a second amount is computed from `escrow_rate[t]` percent of
the same (post-multiplier) rate with `GATHER_RATE * 1600` as its denominator,
and added to `escrow[t]`.

Two corrections to an earlier draft. **The guard is `(leader_flags & 0xc) != 4`
— it is not a diplomacy flag.** Bit `0x4` is the human bit; bit `0x8` is
unnamed. So a plain human never accrues escrow and everything else does, which
fits what `docs/COSTS.md` found escrow *is*: a soft reservation the AI holds
back for something it is saving toward.

**And it is not an accumulator.** Where the stockpile carries its remainder
forward in `leftover`, escrow throws its remainder away and buys it back with a
frame modulus:

```
q, r = divmod(escrow_rate[t] * rate, GATHER_RATE * 1600)
if r != 0:
    n = max(2, (denom + r/2) / r)
    if frame % n == 0: q += 1
escrow[t] += q
```

`n` is "one frame in how many should round up", derived from how big the
remainder is; a remainder of half the denominator rounds up every other frame.
Nothing is stored between frames, so it is approximate where the stockpile is
exact — which is affordable precisely because escrow is a reservation rather
than a balance. `pay()` does not implement it; when it does, it must not be
written as the accumulator next to it.

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
`docs/COSTS.md` reads the cost path, where a unit's `SUPPORT` turns out to be
the ramp that makes each one more expensive than the last. In this engine's
data vocabulary "support" means price, not upkeep — which leaves the conclusion
above unchanged and makes it sharper. The field a reader would take for upkeep
is the one that makes the eleventh hoplite cost double the first.

~~The four `*_SUPPORT_GOOD`/`*_SUPPORT_RATE` fields decide which resource a
price is charged in when the one it is written in is not available yet.~~ Not
those four. `docs/COSTS.md`'s second reading (2026-08-20) found the redirect
reads the `*_COST_GOOD`/`*_COST_RATE` pair that precedes them in each record;
**no reader of the four support pairs has been found in either path**, so as
far as anything read goes they are as dead as `calc_support`.

---

## What is not established

- ~~**How many gatherers a building has, and how many it may have.**~~ **The
  slot count is read and implemented, 2026-08-24** — `crates/sim/src/gather.rs`,
  from `BuildTypeData::max_gatherers@0063c430` → `calc_gather@00639e40`'s
  count out-parameter: a flat type (farm, **and the oil well and platform**,
  which `init_final_flags` marks flat) is 1, a university 7, a mine the
  mountain range's `MTN_*_SIZE`→`MTN_*_GATHER` rung scaled by its usable
  cells, and a camp the circle walk — every cell within `WOODCUTTER_RADIUS`
  (octagonal, `vector_dist`) of the footprint centre whose centre tile is a
  tree, or is on the building's own `gather_from` list, adds `16 ×
  LandData::num_make[good]` (the 16-tile inner loop has no test in it — a
  cell with nine forest tiles pays as one with sixteen), rivers under the
  footprint add a sixteenth, `(accum + 8) >> 4`, capped at twice the tiles
  with gather access. Verified against the original's own survey: run9's
  camps get 7 (from 82 listed tiles) and 5 (from 61), the counts the
  `LEADERS=9` record carries (`docs/ORACLE.md`). Still open there:
  `LandData::num_make` (seamed to 1; every run9 cell answers 1), the mountain
  range's grouping (reconstructed as 8-connected cells; no mine on run9 to
  check), ~~`find_gather_tiles` (the list still comes from the dump)~~ — see
  "The gather list, and its shuffle"; what is left of it is the **metal**
  branch, which walks a mountain range rather than the circle — and
  `WOODCUTTER_RADIUS`/`MINE_RADIUS`/`MTN_*` not yet on `Tuning`. *How many a
  building has* — the assembly of `Holdings` from the live chains — is the
  remaining half, in hand.
- ~~**`base_rate`.**~~ `ScenarioFuncSet::set_base_rate` is its only writer, and
  it stores `value << 4`. Scenario-script only; zero in a skirmish.
- **`escrow_rate`.** What sets it. `escrow` itself is no longer open:
  `docs/COSTS.md` reads it in `Type::pay_cost` as a soft reservation that
  ordinary spending may not touch and abandons entirely the moment it needs to,
  and the accrual is specified above.
- ~~**The two building types collected outside cities**, `0x1a2` and `0x1a3`.~~
  `BuildTypeData::gather_radius` returns `WOODCUTTER_RADIUS` for `0x1a2` and
  `MINE_RADIUS` otherwise, which names them: the Woodcutter's Camp and the
  Mine. They are collected outside the city loop because they are the two
  gathering buildings that are placed on terrain rather than in a city.
- ~~**Merchants and caravans.**~~ Both are read: merchants pay in the idle-unit
  loop and are what makes an owned rare pay at all; caravans pay through their
  cities' `trade_val`. What is still unread is `UnitData::calc_gather`'s own
  body — `MERCHANTS_BONUS`, `FISHERMEN_BONUS` and the Porcelain/Nubian terms
  appear in `calc_rare`, in the `((pct + extra) * out) / 100` shape, but the
  chain has not been followed end to end.
- **Rare resources.** `LeaderData::calc_rare`, forty-four of them, each with
  its own constant in `rules.xml`. *Where* they pay is settled (above); what
  each one pays is not.
- **The market.** `MARKET_BASEMENT`, `MARKET_EQUILIBRIUM`, `MARKET_CYCLE_RATE`
  and the rest describe a price simulation with supply and demand. ~~Entirely
  unread, and the only part of the economy that is not a sum of rates.~~
  **The price cycle is read and landed, 2026-08-24** (`docs/SYNC.md` §3.1,
  `crates/sim/src/market.rs`): `GameDaemon::calc_markets` — the drift toward
  the equilibrium, the eight-tick rota, and the three sync-stream draws a
  good takes when its trend runs out, pinned on run12's frame-0 values.
  What a *trade* does with the price and the flux (`Leader::buy/sell`,
  `MARKET_SUPPLY_DEMAND`) is still unread.
- ~~**Costs.**~~ and ~~**Population.**~~ Closed by `docs/COSTS.md`, which
  specifies `UNIT_COST_FACTOR` and its siblings, the ramp, the
  unavailable-resource redirect, the discount tail, `Leader::can_pay` and
  `Leader::calc_pop_cap`. What remains open there is the *production* half:
  `Build::queue_up`, `JOB_TIME` and when in an item's life the price is
  actually charged.

---

## Second reading (2026-08-20) — landed

A blind second derivation and its adjudication are in
`docs/audit/2026-08-20-economy.md`. The frame order, the gather order, the
8/512-frame cadence, sixteenths and the 7200 denominator, the ×256 trio, the
assembly order, city flat/taxes/literacy, the per-gatherer truncation, the cap
pipeline and `do_gather`'s arithmetic are **doubly confirmed**.

Nine behaviour-relevant points went against the earlier draft of this document
— every one of them about *where* something happens or *who* it applies to,
not about a number. **No arithmetic changed and every existing test stands.**
All nine are landed above, each marked where it changed what an earlier draft
said, and the one implementation change they force — deleting `City::refinery`,
a field the original always writes zero and a caller could have used to count
refineries twice — is landed in `crates/sim/src/economy.rs` on the same day.
The two that most change what a reader would build: caravans and rares pay
somewhere other than where this document put them, and the difficulty handicap
is the AI's.

The nine also came with a dozen extensions that closed open items rather than
contradicting anything: Dutch interest, the infinite-resources option, the
tech-cost thresholds, the four gatherer branches and their slot counts, the
full wonder and cap-modifier lists, the Porcelain Tower on market taxes, the
territory-tax modifiers, the writers of `base_rate` and `bonus_cap`, and the
names of the two outside-city building types. Those are folded in above too.

Resolved for the earlier draft: the enhancer arrays are `int[5]` with 1-based
levels, so `fishermen_bonus[lvl + 4]` really is `GRANARY_BONUS[lvl - 1]`, as
written here; the second reader had it off by one.

Two places where re-reading the decompile sharpened the adjudication itself:
the infinite-resources literal is `0x104be` *as stored*, which is 99999 once
the `bucket` XOR mask comes off — not 66750; and the enhancer reaches
`BuildTypeData::calc_gather` as the rational pair
`(enhancer_amount(t, 100), 100)`, applied to the per-gatherer value before the
multiply by gatherers, which is what this document already said.
