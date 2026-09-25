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

**Where the implementation is.** `crates/sim/src/economy.rs` for the
arithmetic, `crates/sim/src/holdings.rs` for the walk that fills its inputs,
and `crates/sim/src/rares.rs` for step 6 and what an owned rare does to a
unit. Every constant
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
   `is(0x13d)` (the Fisherman lineage) *or* `UnitData::is_merchant`, and
   `UnitData::order_type == NONE` — a merchant walking somewhere earns nothing.
   **This is also where every owned rare resource pays**; see below.
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
one of the territory-affecting ones, and **`0xc000000`** goes up
(`006ce280`, the `operator!=` arm's last line) — *both* dirty bits, not
one: `0x4000000` is the unit-stats pass below, and `0x8000000` is
`Leader::calc_wall_stats`, which re-bakes every **unfinished** building's
`constr_time`. The second half is Tobacco's ten per cent reaching a site
already standing, and it was missing here until item 261
(`docs/CITIES.md` §3.2).

**There is exactly one territory-affecting rare, and it is Gems**
(2026-09-06, item 117). The redraw arm compares `rare.ptr[2] >> 7` before
and after — bit 23, which is `GEMS - BASE_RARE` — and calls
`Regions::fix_all_borders@0067f7d0`, which zeroes each region's border
stamp rather than recomputing anything, so the sweep it schedules reads the
mask *after* the assignment. It is the same bit
`World::compute_reg_territory@006b0bb0` tests, as an inlined
`LeaderData::has_rare@006e0770`; `docs/ATTRITION.md`, "Territory", has the
field derivation and the diff that pins it.

### Step 6, whole (2026-09-02)

`Unit::do_gather@005fce20` is `UnitData::calc_gather@00609180` with its
`param_7` **zero** — the other form of the same search `docs/ORDERS.md` §6.10
specifies for `think_fish`. Three things differ at this site, and all three
matter:

- **A packed fisherman or merchant answers nothing at all.** It is the
  function's *first* test — `(is(0x13d) || type ∈ {0x3d, 0x3e, 0x190}) &&
  unit_masks & 0x80000 && param_7 == 0` — and `think_fish`'s call, which
  passes 1, does not have it. A boat still sailing to its deposit claims
  nothing on the way.
- **The crowd count skips a packed unit outright** (`param_8 == 0`), where
  `think_fish`'s form exempts one that is mid-unpack.
- **The three outputs are live**: the six rates, the `BitMask<44>` the good's
  bit goes into, and a per-good tally. `think_fish` passes null for all three,
  which is why the search itself was readable long before this was.

Each unit's rates are `LeaderData::calc_rare@006e08d0` for the good it found,
**divided by `crowd + 1`** — each unit's own truncation, so three boats on one
fish pay 159 sixteenths between them and not 160.

`calc_rare` reads the good's own two `(BONUS_TYPE, BONUS_NUM)` pairs out of
`resourcerules.xml` — Fish is `Food 10` and `Wealth 10`, Whales is `Food 10`
and `Metal 10` — times sixteen. Two percentages then sit on them, and *which
one reaches which half* is the part a reader gets wrong:

- **`MERCHANTS_BONUS[level]` replaces the 100**, and it reaches a resource only
  when the unit stands in its own or an ally's ground, **or** when the good is
  Fish or Whales and the resource is **not food**. So a fish's wealth half is a
  merchant's business and its food half is not.
- **`FISHERMEN_BONUS[level]` is added**, to the **food slot only**, and only
  for those same two water goods.

Both ship inert at level zero — `MERCHANTS_BONUS` entry 0 is 100% and
`FISHERMEN_BONUS` entry 0 is 0% — so a level-0 fisherman on a fish pays exactly
`10 × 16` food and `10 × 16` wealth and nothing else. That is the number run59's
census measured from the other side: the AI's `income[food]` was 1440 against
1600 and its `income[wealth]` 0 against 160, from frame 4992, which is the frame
its `1/14` settled on its fish.

The upgrade ladders are `rules.xml`'s `TECHBONUSES`: `FISHERMEN1`–`3` are rows
19–21 (`0x2bf`–`0x2c1`, Agriculture / Crop Rotation / Food Industry) and
`MERCHANTS_1`–`4` are rows 99–102 (`0x30f`–`0x312`, Taxation / Vassalage /
Social Contract / Income Tax). `LeaderData::get_fishermen@006d6e80` and
`get_merchants_level@006d6dc0` answer the highest row held.

### The membership changes are what set the dirty flag

Step 6's walk is over units whose `order_type` is `NONE`, so its membership
changes every time a fisherman or a merchant goes idle or is given an order —
and **each of those two transitions marks the owner's economy dirty**, dropping
the refresh period from 512 frames to 8:

- `Unit::check_idle@006032c0`'s tail takes a per-unit latch (`ObjectData +0x8 &
  8`) the first frame a unit is idle, and raises `0x2000000` if the unit
  `is(0x13d)` or is one of the three merchant ids.
- `Unit::work@0060d180:268` clears the same latch when the unit has an order
  again, and raises the same flag for the same kinds.

Without that pair a deposit a boat has just arrived at waits up to 512 frames
to be counted. With it, run63's whale is claimed on frame 5551 — seven frames
after the boat went idle — which is exactly where the dump has it.

### What an owned rare does

The mask is what the rest of the game asks, and it is read in a dozen places.
Only one of them is modelled here, because only one has ever moved a number:
**Whales**, rare bit 25 (`TypeIndex` 31). `Unit::update_speed@006055c0` scales
a type with objmask `0x2000` (`NAVAL`) by `(WHALES_SHIPS_MOVE + 100) / 100`,
which as shipped is `+20%`.

The cached speed is not recomputed on demand: `Leader::gather`'s tail
raises `LeaderData`'s `0x4000000` (with `0x8000000` beside it, above), `Leader::process@006b88b0` acts on it in the
**same frame**, and `Leader::calc_unit_stats@006cf970` walks every one of that
player's units calling `Unit::update_speed` and `Unit::update_armor`. Because
`Leaders::process_all` runs before `Objects::process_all`, the new speed is the
one that frame's own step uses.

run63 is the capture: on frame 5551 East Indies' AI settles its second
Fisherman on a whale, and on 5552 all three of its Fishermen read `myspeed 45`
where they read 38, and its Transport Barge reads 30 where it read 25. Its
citizens and its scout do not move — they are not naval. `38 × 120 / 100` is
**45**, and the truncation is the original's.

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

~~**What this does not close.** A leader still *starts* with a hundred of each
of the three, where the original starts with none.~~ **Closed** (2026-09-06):
the starting grant arrives **with the good**, and this crate pays it that way.
`Leader::init@006e3930` zeroes each bucket and pays it back only if
`type_avail(g, 1)` already holds, and `Leader::gain_tech@006dcb60` pays a good
whose **first** prerequisite is the tech just gained and which the leader did
not hold `has_preq` for before the call — the gate is `has_preq`, not the zero
bucket the earlier draft named. `docs/COSTS.md`, "The starting grant arrives
with the good", writes out both halves, the nation terms that ride in the
first, and the amount. The 240 rows `run40_s_census_…` booked are 0.

## The rares a leader has seen (2026-09-02)

A rare pays nothing until a merchant stands on it, and a merchant is not
built until the AI knows a rare exists. That knowledge is a list, and it is
the reason East Indies' AI trains a Merchant on frame **6353**.

**The list.** `LeaderData +0x6e6c` is `SimpleArray<int> new_rares` — the
**goods-list indices** of the rares this leader has seen, in the order it
saw them. `+0x6e70` is its length. Nothing ever shortens it except
`Unit::think_merchant`, which takes an entry out and puts it back on the
end.

**The writer.** `Leader::new_rare@006d9e70`, and its only two callers are
`World::reveal_fog@006b3d30` and `World::compute_reg_territory@006b0bb0`.

`reveal_fog(fx, fy, who)` runs on exactly the fog cells `World::set_seen`
answered *changed* for — so a cell is offered once per player for the life
of a game, and a rare under the **start** fog is recorded during `Setup`,
before frame 0. Its rare arm is two tests and a call:

```
if (tile_mask[2fy + 1][2fx + 1] & 0x200) == 0:   return   # a good's footprint
g = find_good_at(fx >> 1, fy >> 1, who, index = 1)         # alive, not OIL
if g >= 0: Leader::new_rare(leaders[who], g)
```

`find_good_at`'s **index** form is the one taken here, and it answers ahead
of `type_avail` — the availability test that would refuse the good is
`new_rare`'s own, applied per recipient.

**"Recorded during `Setup`, before frame 0" is the sentence a harness has to
act on, and for a year it did not** (2026-09-04, item 207).
`rondata::diff::build_sim` installs the dump's `seen2` grid instead of
walking the sweeps that produced it, so every one of those pre-frame-0
offers was skipped — and skipped for good, because those cells now read as
seen and `set_seen` can never answer true for them again. `new_rares` then
starts empty where the original's already holds a rare, and since the score
is `200 − 10 · position` the missing entry is the one at the **front**:
Great Lakes' AI Merchant walked to the wrong rare, right across the map,
and that was the map's word at 5786. `Sim::seed_new_rares_from_fog` replays
the reveals once over the installed grid; the word ran to **6080**
(`docs/MERCHANT.md` §2.2.2). The order two goods seen in the same start
reveal take is the one thing a snapshot cannot carry back, and it is that
document's §7.

`World::compute_reg_territory@006b0bb0` is `new_rare`'s **second** caller
and nothing here models it, so a rare a leader learns of through territory
rather than through sight is still missing.

`new_rare(this, g)` then walks every leader:

```
if (this.leader_flags & 0xc) == 4:  return          # a plain human records nothing
for L in leaders:
    if not (L.leader_flags & 1):            continue
    if L is not this and not (L allied this and this allied L): continue
    if (L.leader_flags & 4) and not (this.leader_flags & 8):    continue
    if g in L.new_rares:                    continue
    if not L.type_avail(good_type, strict): continue
    if good.is(FISH) or good.is(WHALES):    continue
    L.new_rares.add(g)
```

**`FISH` and `WHALES` are excluded, and that is the whole design.** They are
the two rares `LeaderData::calc_rare` pays to a *fishing boat* rather than a
merchant (this document, step 6), so a coastline full of fish buys no
merchants at all. The two tests are `SubObject::is` — vtable slot `0xb8`,
which the decompiler prints as `ppuVar1[0x2e]` with arguments `(6, 0)` and
`(0x1f, 0)`.

**The reader.** `ScenarioFuncSet::num_rare_resources_seen@009ea010` is the
script function, and it is nothing but the length:

```
w = who - 1                                   # the script's who is 1-based
if (unsigned)w < 8 and (leaders[w].flags & 1) and (leaders[w].flags & 2):
    return leaders[w].new_rares.length
return -1
```

Its base register reads as `Window::key_states + who * 0x6eec + 0xe4` in the
decompile, which is `&leaders.list[who - 1]` — the same leader the return
indexes, and the reason the two look inconsistent.

**What it buys.** `economic.bhs`, the AI's opening script, gates its whole
merchant block on it:

```
if (num_type(who, "Market") >= 1) {
  if (num_type_with_queued(who, "Caravan") < caravan_lim)
      train_unit_with_cost(who, 1, "Caravan");
  if (at_least_type(who, 100, "Wealth"))
    for (j = num_type_with_queued(who, "Merchant");
         (j < 3) && (j < num_rare_resources_seen(who)); j++)
      if (train_unit_with_cost(who, 1, "Merchant") < 1) break;
}
```

so a host that answers zero — which this crate's did, on the grounds that
rares were not in the simulation — trains **no merchant ever**, on any map,
for any AI.

**How it is established, and how confident it is.** The list, its writer and
its reader are read from the decompile; the exclusion of `FISH` and
`WHALES` is a vtable-slot resolution rather than a guess at a name
(`vtables.txt`, `Good::vftable +0xb8`). The consequence is **diff-backed on
run65**: East Indies' AI leader has seen `CITRUS` (good 20) and `HORSES`
(good 23) by frame 5000, its Market's queue is one `MERCHANT` deep on all
eighteen blocks of `[6196, 6214)`, its `job_counter` is the original's on
every one of them, and the unit comes out on 6353 — the frame East Indies'
word had been parked on
(`run65_s_window_is_the_original_s_unit_for_unit`).

**What is not established.**

- **The start fog is not replayed.** A rare already under a leader's fog at
  frame 0 was recorded by the original's `Setup` and is not recorded here,
  because this simulation begins from an installed fog grid rather than
  from the reveals that built it. On East Indies it costs nothing: the one
  good under the AI's start fog is an **oil patch**, which `find_good_at`
  refuses on both sides. A map where it is not oil would part.
- **`World::compute_reg_territory`'s call is not carried.** Territory
  changing hands is the second way a rare is learned, and nothing here
  reaches it.
- **`leader_flags & 8`**, the AI-driven human, is never set here, so the
  human's own reveals record nothing — which is what the original does for
  a human without computer assist and is a seam for one with it.
- **The second merchant.** The original trains another on **6571**; the
  script's `j` bound is three and the leader has seen two rares, so
  whichever call queues it is past the window every capture on disk covers.
  It is also the only thing that can check `Unit::think_merchant`'s
  rotation of this list (`docs/MERCHANT.md` §2.3), and it sits one frame
  past the word.

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
block. It is the section below — and the "separate pair of counters at
`+0x8ac`/`+0x8dc`" this document used to describe it by are not separate at
all.

## The wealth slot, and the thirty it pays

`Build::activate@00623e20` **line 590**, before everything above:

```
if is_dock(type) or is(MARKET) or is(TEMPLE):
    gather_slots[WEALTH] += 1
    if gather_slots[WEALTH] > gather_slots_high[WEALTH]:
        gather_slots_high[WEALTH] = gather_slots[WEALTH]
        if frame != 0 and not captured and counted and <not a fenced scenario>:
            do_bonus(2, 0x1e)                       # thirty wealth
```

**`+0x8ac` is `gather_slots[2]` and `+0x8dc` is `gather_slots_high[2]`.**
`LeaderData +0x8a4` is `gather_slots` and `+0x8d4` is `gather_slots_high`, so
the two addresses are the third entry of each — the type record says so and
the arithmetic agrees. The wealth slot no writer here could produce
(`get_good`'s table has no wealth arm, and never will) and the thirty wealth
nobody paid were one line all along.

The first of the three kinds is the vtable call the decompiler leaves as
`(**(code **)(**(int **)&this->field_0x18 + 0x108))()`. It is settled
without the PDB's vtable records: `ObjectData::is_dock@004711e0` is that
call and nothing else, so slot `+0x108` on the type is `is_dock`. The other
two are spelled out — `is(0x1b4)` is MARKET, `is(0x1b5)` is TEMPLE.

`Build::close@00628980` line 107 is the mirror, three lines below the gather
slots' and inside the same `flags & 4` guard: the slot goes back, the mark
does not follow it down, and a rebuild is therefore free of bonus.

**How it is established.** The block is read; both halves are diff-backed.

- **The slot.** run59's census at `[5150, 5400)` has `gather_slots[wealth]`
  **1** on both players against this crate's 0, for every one of its 250
  frames. The human's is its **starting Market**, stood up at frame 0; the
  AI's is the **Dock** it finishes on sim-frame 3579. Both are exact now.
- **The thirty, on its own frame.** run60's per-frame census (below) has the
  AI's `bucket[wealth]` step by thirty more than this crate's between the
  blocks labelled `FRAME 3580` and `FRAME 3579` — the end of sim-frame 3579,
  which is the frame its Dock activates and the only frame in 5,400 on which
  the two sides' wealth parted at all.
- **Frame 0 pays nothing.** The human's Market claims the slot and its wealth
  bucket is the original's on all 250 of run59's frames, which is the same
  gate the gather bonuses take.

**What it does not establish.** The German multiplier is applied here by
analogy with `Build::do_bonus`'s other callers — `do_bonus` is one function
and the multiplier is inside it, so this is a reading of the call rather
than of the arm. The scenario fence (`Game::is_scenario` and
`ScenarioData::building_resource_bonus`) is read and not modelled; no
capture is a scenario.

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
stamping `MiningList::mtn`/`::cliff` so the search is not repeated.
~~It is **not modelled**: a mine placed during a run gets an empty list,
which is what it got before any of this existed.~~ **The mountain half is
read and implemented, 2026-09-18** — "The mine's range" below. The cliff
half is not.

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

**What it does not establish.** ~~The **metal** branch, above.~~ Landed
2026-09-18; see "The mine's range" below for what *it* leaves open. And
`Build::process`'s two re-entries — `verify_gather_tiles` on a region's
`0x20`, `find_gather_tiles` again on its `0x10` — which no run has been
seen to take, because the region flags are not modelled at all.

## The mine's range (2026-09-18)

A mine does not survey the ground around it at all. `calc_gather` sends
`0x1a4` (the university) and `0x1a3` (the mine) past the circle walk before
it starts — `if (!is_flat && !is(0x1a4) && !is(0x1a3))` at `00639e40`'s head
— and the mine's own arm is the **nearest mountain range**, whole.

That is two separate things, and until 2026-09-18 this crate had the second
missing and the first standing in for it with the camp's cell walk.

### The site test — `calc_gather`'s mine arm

```
reach = gather_radius(type) * 0xc0                  # MINE_RADIUS 6, so 1152 world units
centre = corner_tile(type, corner)                  # the footprint's centre, world units
if list is null or (list.mtn < 0 and list.cliff < 0):
    mtn, mtn_d = MountainsData::find_nearest(mountains, centre, region)
    clf, clf_d = CliffsData::find_nearest(scary_cliffs, centre, region)
else:
    mtn, clf, mtn_d, clf_d = list.mtn, list.cliff, 0, 0
if clf >= 0 and mtn >= 0 and clf_d < mtn_d:   mtn = -1      # the cliff is nearer
if mtn >= 0 and reach < mtn_d:                mtn = -1      # and both may be too far
if list is null:                                              # the survey only
    for t in range(mtn).tiles:                                # the template's order
        if not tree(t) and mountain(t) and not enemy(t) and gathered_from(t):
            return -1                                         # MOUNTAIN_TAKEN
count = MountainRangeData::gather_size(mtn, …)   # or CliffsData::gather_size
```

The survey's walk (item 613, `docs/AI.md` §62) is what refuses a second
mine on a range another building already mines. The cliff arm has the same
walk over the cliff's tiles, and it is not built.

`find_nearest@0089cd30` walks every placed mountain and ~~**every tile of
its range**~~ its range's **solid cells** (`solid_mount_wx`/`_wy`), keeping
the smallest `vector_dist` in world units to a cell's **centre**; the region
argument skips a range whose first solid cell is on another landmass. The
tile reading was wrong, and run144's packet measured it (`docs/AI.md` §59).
So a mine site is refused unless a solid mountain cell's centre is within
six tiles of where the building would stand. That is not the camp's walk
from the anchor either.

**The difference is a factor of two, measured.** Great Lakes' frame 8382
places the game's first mine. `Leader::produce_building`'s spiral draws once
per friendless FARM/MINE candidate that clears `blocked_site`
(`docs/AI.md` §2.20), and the cell walk let **eighteen** candidates through
where the original passed **nine**: the nine whose nearest mountain tile is
inside 1152 world units, exactly. The same nine pass the solid-cell measure,
because Great Lakes' word holds (`docs/AI.md` §59.5). Nine extra draws moved the placement
jitter's four rolls nine places down the stream and put the mine a tile east
of the original's.

### The list — `find_gather_tcoords@0063bdc0`'s metal arm

The range's tiles, in the template's own order, keeping each one that

1. is not `SURFACE_FOREST` (`(mask & 0x30) != 0x30`),
2. stands on unclaimed, own or allied territory (`WData +0xf`), and
3. is not already gathered from (`mask & 0x1000`).

`MiningList::mtn` is stamped with the range so a later `calc_gather` on the
same building skips the search.

### What `gather_size` counts, which is not the tiles

`MountainRangeData::gather_size@0089d170` walks `solid_mount_wx`/`_wy` — a
**cell** list — and counts a cell whose own `WData.flags & 0x20` is clear and
whose centre tile is a mountain. Those are `total`; the ones in friendly
territory and (with a list in hand) whose centre tile is on it are `usable`.
The rung comes from `mount_tx`, the range's **tile** count, against
`MTN_TINY_SIZE` … `MTN_HUGE_SIZE`, and is then scaled `rung * usable /
total`, floored at 1 when anything is usable.

### The reconstruction, and what run97 and run80 say about it

The original's ranges are map-generator objects: a placed mountain has a
location and a template with a tile-offset list. ~~Nothing in a gamelog dump
carries them.~~ A `DUMP_ALL` head prints the placements
(`GameLog::dump_mountains`), and the lists are the template image's alpha.
Since item 604 `sim::Sim::mountain_range` returns the placed range
(`docs/AI.md` §60, `docs/FORMATS.md`, "The mountain templates"). Without
placements, it stands one up as the **eight-
connected component of mountain tiles** containing the nearest one, and
`solid_mount` as that component's cells whose centre tile is a mountain and
whose cell is not a forest one.

It is pinned by value rather than by argument:

| record | says |
| --- | --- |
| run97 frame 8383, `1/2021` | `mtn 6`, `cliff -1`, `length 207`, and the 207 tiles |
| run80 frames 23960–23999 | the same 207, unchanged 15,600 frames later |
| run80, `1/2022` | a second mine, `mtn 0`, **232** tiles |

This crate's component around Great Lakes' first mine holds **244** tiles, of
which exactly **37** carry `SURFACE_FOREST` — 207 — and every one of the
original's 207 is among them. The second mine's 232 agree too, on a range
the reconstruction was not fitted to.
`great_lakes_first_mine_lists_its_mountain_range` (`rondata::diff`) is that
comparison.

### What it moved

Great Lakes' word **8382 → 8404**. Frame 8382 went from 46 draws to the
original's **865**, entry for entry: three `make_stuff`, nine spiral, four
jitter, `4 × 207 = 828` shuffle, and the rest of the frame unchanged. The
mine lands at `(41088, 25920)`, which is the original's own position for
`1/2021`. East Indies does not move.

### What it does not establish

- **The range index.** `mtn 6` is an index into the generator's placed-
  mountain array. ~~A component walk has no such order, so this crate does not
  reproduce the number and does not store one;~~ `find_nearest_mountain`
  reproduces it for both of run80's mines (`docs/AI.md` §60), and this crate
  does not store it. `MiningList::mtn`'s caching
  role is unmodelled with it. The falsifier is a capture where a mine's
  `calc_gather` is re-run after its list exists and the two answers differ.
- ~~**The pre-shuffle order**, and therefore the post-shuffle one. The original
  walks the template's tile arrays; this crate walks rows. The **count** is
  what the sync stream sees and the count is right, and the set is right, but
  `Unit::do_non_flat_gather` ranks tiles by `i >> 2` — so *which* tile a
  miner works first is not the original's. The falsifier is a `GATHERORDER`
  dump over a mine's citizens.~~ The template's tile arrays are in row order,
  and with the placements this crate walks them (`docs/AI.md` §60). run143's
  10583 gather list agrees on all 80 rows.
- **The cliff arm.** `CliffsData::find_nearest` runs beside the mountain one
  and wins when a scary cliff is nearer, with its own `gather_size`. Neither
  scored map has a single `OBJECT_CLIFF` tile, so nothing here is refused by
  a capture; a mine beside a cliff still gets an empty list. The falsifier is
  a map with cliff tiles and a mine on one.
- ~~**The tie-break in `find_nearest`.** The original's order is the placed-
  mountain array's and then each range's own tile order; this crate's is
  row-major. Two ranges the same distance away would seed different
  components. No capture holds a tie.~~ The placed order, strictly nearer
  wins (`docs/AI.md` §60.3). It holds only with the placements.
- ~~**The region filter.** `find_nearest` takes the site's region and skips a
  range whose location cell is elsewhere; this crate scans a box that cannot
  reach another landmass at six tiles, so the filter never bites and is not
  written.~~ It skips a range whose **first solid cell** is in another region
  (§59.2), and is written (§60.3).

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

**The republic term is built and diff-backed, 2026-09-25** (`docs/AI.md`
§72). `REPUBLIC_COMMERCE_BONUS{,2,3}` are 50 each, and the three
`REPUBLIC_n` bonuses each need Republic alone. run221's British AI, at
commerce level 2 with Republic, carries **3792** against the 2992 this
crate had: `150 × 125 / 100 + 50`, times 16. Diamonds, the wonders and
Virtual Reality are still not built.

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
  cities' `trade_val`. ~~What is still unread is `UnitData::calc_gather`'s own
  body~~ — read and landed 2026-09-02 (step 6, above; `crates/sim/src/rares.rs`).
- ~~**Rare resources.** `LeaderData::calc_rare`, forty-four of them, each with
  its own constant in `rules.xml`.~~ **What each one pays is `resourcerules.xml`'s
  own two `BONUS_TYPE`/`BONUS_NUM` pairs**, not a constant in `rules.xml`, and
  `calc_rare` reads nothing else off the good (step 6, above). What a rare
  *does* beyond paying is a per-rare rule scattered across the executable —
  `WHALES_SHIPS_MOVE` is read and landed, and the other forty-three are not.
  The **Porcelain Tower** and **Nubian** terms in `calc_rare`, and the
  **Japanese** fishing-boat one, are the nation and wonder layer and are always
  zero here; so is the Porcelain Tower's pass at the top of `calc_gather`,
  which is the one thing that makes a rare pay with nobody standing on it.
- **The market.** `MARKET_BASEMENT`, `MARKET_EQUILIBRIUM`, `MARKET_CYCLE_RATE`
  and the rest describe a price simulation with supply and demand. ~~Entirely
  unread, and the only part of the economy that is not a sum of rates.~~
  **The price cycle is read and landed, 2026-08-24** (`docs/SYNC.md` §3.1,
  `crates/sim/src/market.rs`): `GameDaemon::calc_markets` — the drift toward
  the equilibrium, the eight-tick rota, and the three sync-stream draws a
  good takes when its trend runs out, pinned on run12's frame-0 values.
  ~~What a *trade* does with the price and the flux (`Leader::buy/sell`,
  `MARKET_SUPPLY_DEMAND`) is still unread — and **it is now what the AI's
  own market is waiting on**.~~ **The trade is read and landed, 2026-09-18
  — §12 below** (item 358): `calc_market_prices`, `do_buy` and `do_sell`,
  and `use_market`'s own loop over them. What is left of the seam is the
  Supercollider's clamp and Conquer-the-World's bonus, neither of which
  any capture on disk reaches.
- ~~**Costs.**~~ and ~~**Population.**~~ Closed by `docs/COSTS.md`, which
  specifies `UNIT_COST_FACTOR` and its siblings, the ramp, the
  unavailable-resource redirect, the discount tail, `Leader::can_pay` and
  `Leader::calc_pop_cap`. What remains open there is the *production* half:
  `Build::queue_up`, `JOB_TIME` and when in an item's life the price is
  actually charged.

---

## The docs-versus-code pass, 2026-09-05

`docs/audit/2026-09-05-economy-vs-code.md` (Opus reader, Opus
adjudication). 112 stated rules traced to the code that implements them;
twenty-one disagree, all twenty-one confirmed, none struck.

**The sharpest was that this document's five indexed ladders never moved
— closed, item 232, 2026-09-07.** `Levels::for_player` ignored its player
and answered a constant whose `taxation` is 0, so "The territory tax"'s
`TERRITORY_TAXES[taxation_level]` was `TERRITORY_TAXES[0]` — zero per cent
— for the life of every game, and "The enhancers"' granary, lumber mill and
smelter and "What a city gives"' university were all pinned at level 1. It
was not a level too low but a whole income line that could never be
non-zero.

The five now load like the two ladders beside them.
`crate::tech::Roles` carries `granary_preq`, `lumbermill_preq`,
`smelter_preq`, `university_preq` and `taxation_preq`, read from
`TECHBONUSES` at 15–18, 22–24, 25–27, 53–57 and 95–98 off `BASE_BONUSTYPES`
(`enums/TypeIndex.txt`; they bracket `FISHERMEN1..3` at 19–21 and
`MERCHANTS_1..4` at 99–102, which the loader already had), and
`Levels::for_player` answers each one's highest held rung through
`Sim::bonus_level`. The four enhancers floor at 1 and taxation at 0 —
`get_granary@006db340`'s `(held) + 1` against `get_taxation@006d6e20`'s
`(uint)(held != 0)` — so a tree naming none of the rows still answers
`Levels::BASE`, and the change is a widening. `assemble_holdings_with` now
writes `Holdings::taxation`, which was the second half of the defect:
nothing wrote it at all, so it kept its `Default` zero however the levels
were computed.

**No capture on disk can falsify any of this**, and that is stated here
rather than left implied. The original's own territory tax is zero through
every captured window too — neither leader reaches a taxation level inside
one ("What run60 leaves") — so the two sides agreed before the fix and
agree after it, and the endpoint at 24,001 does not move. This is
correctness argued from `get_taxation`'s own listing, not from a diff. The
check that would give it an oracle is a capture taken past a `TAX_1`
research, with `LEADERDATA`'s `resources[2]` and `income[2]` on the frame
after; run60's `resource_cap` step at 2958 says the commerce line *is*
being researched there, and the taxation ladder sits beside it.

Two rows are deliberate negatives and are not bugs. The `over_cap == 2`
threshold is `999 × 16 − 16` in the code and `999 × 16 − 1` here, and the
two predicates agree on **every** value the pipeline can produce, because
`commerce_cap` returns only multiples of sixteen; the row stays because a
`bonus_cap` added after the shift would part them silently. The obsolete
redirect is a dead path.

The remaining eighteen are **stated, unimplemented and unreached**, and
most are whole clauses rather than constants: Dutch interest; the knowledge
tech-cost penalty; the fast-economy option and `ai_speed` in the income
multiplier chain, where `pay` scales by the handicap and nothing else; the
infinite-resources short-circuit; `base_rate`, which has no representation
at all and is blind (`ScenarioFuncSet::set_base_rate@009fbb80`); the
territory tax's three modifiers; the slot bound on the rate path; the flat
branch's per-gatherer rate, a constant where this document has a footprint
survey; the Forbidden City and the CEO hero; six nation terms inside the
city and branch arithmetic; five of the thirteen assembly steps; four terms
of the commerce cap; the Porcelain Tower's market-tax modifier and its
`rare_owned` pass; a rare-mask change recomputing nothing but unit stats;
and a player starting with all six goods where the original grants three
with the age.

And one row is not about this document at all: **four doc-comments in
`economy.rs` still carry claims this document has retracted**, one of them
contradicting four pinned diff tests. That is the cheapest to fix and the
most misleading to leave, because a stale comment at the site is what the
next session reads while editing.

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

---

## The census at the word — run59, East Indies `[5150, 5400)` (2026-09-02)

East Indies' word is **5376**, and the frame is a purchase: `economic.bhs`
case 15 asks for a Market at the AI's capital, the original places it, and
this crate cannot pay. A Market is eighty timber. That made the headline a
**resource level**, and until run59 no capture on disk carried one past
frame 800 — `LEADERS=1` at `[End Frame]` is five scalars and no goods, and
the only `LEADERS=9` windows were run40's `[560, 600)` and run41's
`[770, 800)`, both on Great Lakes.

run59 is that window moved (`docs/RUNS.md`, "run59"): run58's own game,
`LEADERS=9` at `[End Frame]`, and the dump narrowed to 250 frames around the
word. **18,000 good-frames** — 250 frames, two players, six goods, six
fields — and 4,798 of them disagreed, in nine shapes. Every one is a
**standing state**: each is wrong on all 250 frames, which is what says it
is a level rather than anything the window does. **1,500 are left**, all of
them item 156's.

### The item: fifty timber, and it was a lump — closed 2026-09-02

| | ours | theirs |
| --- | --- | --- |
| the AI's `bucket[timber]` at 5150 | 118 | **168** |
| at 5376, the Market's own frame | 34 | **84** |

Fifty short on every frame before the purchase — and `leftover[timber]`, the
fractional accumulator, **agreed on 234 of the 250**. The two sides were
therefore paid the same amount of timber every frame *in this window*, so
whatever banked the fifty was older than the window. On 5377 the gap flipped
to **thirty ahead**, because the original was eighty poorer and this crate
was not.

It is a **goody box on sim-frame 4988**, and the good it paid was wrong here
because a dock's thirty wealth was missing 1,409 frames earlier. The chain is
under "run60" below; the AI's timber is now the original's on every frame of
this window and across the Market.

### The six timber slots the same census found, and why they are not the fifty

`gather_slots` is the running inventory `Build::activate` adds a finished
gather building's `gather_max` to ("What a finished gather building pays"),
and the dump prints it per good beside its own high-water mark — which is
what decides the per-slot bonus. The census found both players short the same
rows:

| | ours | theirs |
| --- | --- | --- |
| human `gather_slots[timber]` | **0** | 6 |
| AI `gather_slots[timber]` | 4 | **10** |
| both players' `gather_slots[wealth]` | 0 | **1** |
| both players' `gather_slots[food]` | — exact — | |

The human builds nothing at all in this game, so its six are its **starting**
camp's; the AI's ten are the same six plus the four its own camps add. The
food slots were exact on both players, which is what said this was one
lineage rather than an array nobody writes.

**It was a harness defect, and it is closed (2026-09-02).** `Build::init`
fills a camp's `gather_from` at placement and `Build::activate` then adds the
`gather_max` surveyed *from that list* to `gather_slots`. A camp stood up
from a dump could never survey anything: `build_sim` loads the dump's own
tile masks into the world verbatim, so the camp's tiles already carry
`0x1000` (`is_gathered_from`) before the camp exists, `find_gather_tiles`
skips every cell that does, and the walk comes back empty. The list was then
installed from the dump **after** `activate` had already surveyed the empty
one. Installing it — and the count with it — before `activate` is the whole
fix, and it closes 500 of run59's 4,798 wrong good-frames: both players'
timber slots are now exact on all 250.

**And it moved the timber not at all**, which is the measurement that refutes
the first reading of the fifty. The arithmetic is why. The bonus is
`TIMBER_BONUS_PER_WOOD_SLOT` = 5 per slot **past the high-water mark**, and
`Build::activate` pays nothing at frame 0 — so in the original the starting
six arrive during `Setup::build_empire`, raise the mark to six, and are never
paid for; the one camp the AI builds in the run (frame 2424, four slots) then
pays for `6 → 10`. This crate used to pay for `0 → 4`. **Twenty timber
either way**, and now it is the same twenty on the same frame. Ten slots
times five is a true sentence about a number that was never paid.

The **human is the control that says so from the other side**: it held six
fewer timber slots than the original for the whole window and its
`bucket[timber]` was exact on every one of the 250 frames. An unpaid slot
costs nothing, and the six were unpaid.

The wealth slot is item 82's, open since run40 — `BuildTypeData::get_good`'s
table cannot produce a wealth-gathering building, and this crate's
`Sim::gather_good` answers `None` for wealth by construction, so the one slot
has no writer here at all.

### Two rate seams beside it — one thing, closed 2026-09-02

Every one of the **human's** six incomes was exact on every frame of the
window. The AI's were not:

- `income[food]` **1440 against 1600** (sixteenths, so ninety against a
  hundred) on all 250;
- `income[wealth]` **0 against 160** on all 250, with `leftover[wealth]` 0
  against 2136 — this crate gave the AI no wealth income at all;
- ~~`income[timber]` agrees until **5384**~~ — that one went with the fifty.

**Both are one deposit.** `160` is `10 × 16`, and `10` is both of Fish's
`BONUS_NUM`s in `resourcerules.xml`: the AI's Fisherman `1/14` has been
standing on its fish since frame 4992 and nothing here walked the idle
fishermen. That is `Leader::calc_gather` **step 6** (above), landed with the
rest of it; run59's census is now 1,500 wrong good-frames of 18,000, and
every one of the remaining is item 156's three hundreds. Both players' six
rates and six incomes agree on every frame of the window.

## The census before the word — run42's nine hundred frames (2026-09-02)

The census above could not say *when* the fifty was banked, and the note it
left said localising it needed a second capture. It did not: **run42 was
already on disk and nothing had ever read its leader block.**

run42 is run39's game — the same game as run54, run56, run57, run58 and
run59, by `samegame.py` and by `rngcmp.py` in turn — captured at run39's
detail plus `LEADERS=2`, which is where `LeaderData::log_data` announces the
encrypted block. So it prints `bucket`, `leftover`, `resources`, `income`,
`rate` and `resource_cap`, per good, on every one of its **900** frames
(`docs/RUNS.md`, "run42").

`diff::tests::run42_s_nine_hundred_frames_are_the_census_before_the_word`
compares five of those six fields for both players on all 900 frames —
**54,000 good-frames** — and the *only* disagreement is item 156's three
unavailable goods, on both players, from frame 1.

That is a stronger statement than the count suggests:

- The AI's `income[food]` and `income[wealth]`, both wrong on all 250 frames
  of run59's window, are **exact for the first 900 frames**. Both seams
  therefore open somewhere in (900, 5150) — they are not standing errors in
  the rate arithmetic.
- The goody box the AI opens on frame 867 pays fifty **wealth**, and this
  crate pays the same fifty to the same good on the same frame
  (`docs/GOODY.md` §6). The one lump on the map that is worth fifty is not
  the fifty.
- Every purchase, refund and completion bonus either side makes in its first
  900 frames nets to the same six buckets.

**So the fifty is banked in (900, 5150)**, and the AI's timber gap is
`0` for the whole of run42's record — the test prints every frame the gap
steps on, and the list is empty.

## run60 — the whole curve, and the two frames it names (2026-09-02)

run42 pinned frames 1–900 and run59 measured a standing fifty at 5,150.
Between them nothing on disk carried a resource level at all, and that is
what run60 is: run58's game with `[End Frame]` cut to `MISC,LEADERS=2` and
nothing else. Every one of its 5,400 frames prints both leaders' six goods,
and the run took **under five minutes** and 67 MB — against run58's
sixty-one minutes for two hundred fewer frames — because the per-frame block
is a few hundred lines rather than ten thousand. `rngcmp.py` against run59 is
5,401 frames with **zero** differing.

`diff::tests::run60_s_whole_curve_is_where_the_ai_s_timber_parts` compares
five fields, six goods, two players, 5,400 frames — **324,000 good-frames** —
and prints every frame the AI's bucket gap *steps* on. Before the two fixes
below there were exactly **three** steps in 5,400 frames, and two of them
were the items:

| frame | what stepped |
| --- | --- |
| 1 | goods 3, 4 and 5 at 100 here and 0 there — item 156 |
| **3579** | the AI's `bucket[wealth]`, thirty behind |
| **4988** | the AI's `bucket[timber]`, fifty behind — and its wealth fifty *ahead* |

### The chain, end to end

**4988 is a goody box, and the lottery picked the wrong good.** The pile
goes to one good (`docs/GOODY.md` §3): `score = draw % 25 + bucket[good]`,
lowest wins, over the available goods. The AI's buckets going into it were

| | food | timber | wealth |
| --- | --- | --- | --- |
| theirs | 208 | **99** | 130 |
| ours | 208 | **100** | 100 |

Theirs: timber is thirty-one clear of the field and the jitter is at most 24,
so **timber wins outright**. Ours: timber and wealth **tie at 100**, and the
jitter decided it — wealth. Same fifty, same frame, same draw count, and a
different bucket. That is why the fifty was invisible to every count-based
check, and it is exactly the open question `docs/GOODY.md` §6 left standing:
"whether a box's *good* is right … answerable from disk and still
unanswered".

**And the thirty at 3579 is why the buckets tied.** The AI finishes its
**Dock** there, and the original pays it thirty wealth — "The wealth slot,
and the thirty it pays", above. With the thirty paid, this crate's wealth is
130 like the original's, timber wins the lottery by thirty-one, and the fifty
timber lands where the original puts it.

**So one unread block at frame 3579 cost fifty timber at 4988 and the Market
at 5376.** Both are closed, and run59's census falls from 4,798 wrong
good-frames to **3,500** — the whole timber lineage, bucket and rate.

### What run60 leaves

- `income[food]` 1440 against 1600 and `income[wealth]` **0 against 160**,
  from frame **4992** — three frames after `SpellType::cast_unpack` is first
  entered, which is the AI's **merchant** unpacking on a rare
  (`UnitData::good_merchant_spot@006068a0`; the merchant family is `0x3d`,
  `0x3e` and `0x190`). run58's `UNIT` record has the other half: `1/14` takes
  `rare 6, good_obj 1` on 4992. Nothing here models a merchant on a rare, so
  the AI earns no wealth at all from 4992 on.
- `bucket[food]` and `bucket[wealth]` one apart from 5002 and 5061 — the
  accumulator downstream of that same rate.
- `resource_cap` on five goods, **two frames** from 2958: 1392 against 2000.
- Item 156's three goods, from frame 1.

### Coverage

**Diff-backed**: all of the above, by
`diff::tests::run59_s_census_is_where_the_ai_s_timber_goes` — which pins each
remaining shape as `(frames wrong, ours, theirs)` rather than filtering it
out, pins the total at 3,500, and pins the timber gap frame by frame across
the Market — by
`diff::tests::run42_s_nine_hundred_frames_are_the_census_before_the_word`,
which pins the 900 frames before it, and by
`diff::tests::run60_s_whole_curve_is_where_the_ai_s_timber_parts`, which
carries the same comparison over all 5,400.

**Not established**: the merchant on a rare — ~~the whole of it~~ the
*arrival* half of it; the order that sends one there is
`docs/MERCHANT.md` — and the `resource_cap` pair at 2958.

**Also on the record and already booked**: `bucket` on goods 3, 4 and 5 is
100 here and 0 in the original, on both players, every frame — item 156,
`STARTING_GOODS` arrives with the age. Inert: an unavailable good is never
charged and never accrues.

---

## 12. The trade — `calc_market_prices`, `do_sell`, `do_buy` (2026-09-18)

Item 358. The price cycle has been landed since 2026-08-24 and the
**trade** that reads it was the last unread piece of the AI's market. It
is three short functions, and the word was sitting on all three.

### 12.1 The two prices

`LeaderData::calc_market_prices@006dc2a0` answers a good's `(buy, sell)`
from the market's own state and nothing else:

```text
sell = market.price[g] + market.flux[g]
buy  = market.price[g] * 2 + market.flux[g]
```

Then **one** spread bonus, added to the sell price and taken off the buy
price — never both, and never two:

- the **Nubians'** `NUBIAN_MARKET_PRICES` (20) when `NUBIAN_MARKET_PRICES`
  is non-zero *and* the player has tribe bonus 4 — the same bonus that
  opens `use_market`'s gate without Coinage;
- otherwise **Amber**, `AMBER_MARKET` (10), when the player holds the rare
  — `rare.ptr[1] & 8`, byte 1 bit 3, which is bit `17 − BASE_RARE = 11`
  and `resourcerules.xml`'s eighteenth `RESOURCE`. It is the only rare
  this function reads (`economy::AMBER`).

Then three floors, in this order:

```text
sell = max(sell, 1)
buy  = max(buy, MARKET_BASEMENT * 2)      // 20
buy  = max(buy, sell + 10)
```

So the shipped default — price 50, flux 15 — is **65 to sell and 115 to
buy**, and a good the market has been flooded with still costs at least
20 and still sells ten under whatever it buys for.

### 12.2 The two trades

`Leader::do_sell@006cfc60` — a hundred of `g` for the sell price:

```text
if bucket[g] < 100: refuse (the original's 1)
bucket[g]      -= 100
escrow[g]       = max(0, escrow[g] - 100)
bucket[WEALTH] += sell
market.price[g] = max(0, market.price[g] - MARKET_SUPPLY_DEMAND)   // 3
```

`Leader::do_buy@006cfbd0` — a hundred of `g` for the buy price:

```text
if bucket[WEALTH] < buy: refuse
bucket[WEALTH]      -= buy
escrow[WEALTH]       = max(0, escrow[WEALTH] - 100)
bucket[g]           += 100
market.price[g]     += MARKET_SUPPLY_DEMAND
```

Two asymmetries are worth naming because neither is a typo. The escrow a
**buy** releases is *wealth's*, not the bought good's — it is always the
resource that left the purse. And the price has a floor on the way down
and **no ceiling on the way up**; `GameDaemon::calc_markets`' drift toward
`MARKET_EQUILIBRIUM` is the only thing that brings it back.

### 12.3 `use_market`'s loop, and why one draw is not a refusal

`Leader::use_market@006c91c0` walks the six goods. For each that is not
knowledge, is available, and whose bucket is under the make list's `need`,
it runs the original's `do { … } while (bucket[g] < need[g])` with a
once-flag (`uVar5`):

1. **Which branch.** `g == WEALTH || (bucket[WEALTH] − buy) < need[WEALTH]`
   → sell; otherwise buy. C's `||` short-circuits, so wealth never asks a
   price: it cannot be bought with itself.
2. **The sell branch** spends the mechanic's one draw,
   `Random::get(game_random, 0, 0xffff) % 6`, which is where the six-good
   rotation starts. Every rotation position that passes the candidate test
   (§40 of `docs/AI.md`, unchanged) is sold — the loop does not stop at the
   first — and the once-flag clears **on the test passing**, not on the
   sale going through.
3. **The buy branch** spends no draw at all, buys a hundred, and clears
   the once-flag unconditionally.

**A single draw therefore has two meanings, and `docs/AI.md` §40 had only
one of them.** It is a pass that found nothing sellable — *or* a pass that
sold, and whose sale carried the bucket over `need`, so the outer `while`
exited before the once-flag was ever consulted. Great Lakes' 8582, 8585,
8782 and 8982 each cost one draw, and 8982 is the second kind: a hundred
timber leaves for eighty wealth, and the scholars that wealth buys are in
run97's own `BUILDDATA`.

### 12.4 What this moved

`LONG_WORD_GREAT_LAKES` **8985 → 9134**; East Indies unmoved at 9711.

The word was a `Leader::use_market+0x1ed` this crate spent where the
original spent a third `Leader::make_stuff+0x221` — an exchange, not a
count, which is why 8985 held eight draws either side. What named it was
not the frame: it was run97's `BUILDDATA` widened whole, every building
and every field, over all 1,320 frames of the window. **One** divergence
came out of 37,899 building-frames and it was the word's own — the AI's
University holds three scholars from 8985 and this crate could pay for
one.

### 12.5 Coverage

**Diff-backed**: §12.1's prices and §12.2's two trades, through the word
itself — `diff::tests::run53_s_24000_frames_put_the_ceiling_where_run33_did`
and `diff::tests::run97_s_build_queues_are_the_original_s`, which compares
every building's whole live queue, its `job_counter` and its `constr_time`
on every block of `[8030, 9349]`. The arithmetic is pinned clamp by clamp
by `sim::ai_make::tests::the_market_prices_a_trade_and_moves_its_own_price`
and the loop by `…::the_market_draws_once_for_wealth_behind_coinage`, both
capture-free.

**Not established**, and each names the capture that would refuse it:

- **The Supercollider's clamp.** `has_wonder(0x21d)` — building type index
  541 — pins the buy price under `SUPER_BUY` (125) and the sell price over
  `SUPER_SELL` (50). `tech::PlayerTech::wonders` has no writer in this
  crate at all, so the predicate could only be a constant `false`; it is
  stated instead. A capture that reaches the Information age with that
  wonder standing, dumping `LEADERS=9` across a `use_market` frame, would
  refuse it. Nothing on disk is past the Classical age.
- **Conquer-the-World's per-leader market bonus**, behind
  `semaphore[2] & 2`. CtW is cut from v1.
- **`RUSSIAN_COMMUNISM`** ships as **0**, which switches its own arm off in
  the original too — the flat 100/100 it would impose on a Russian past the
  fifth age is dead data rather than a seam.
- **Whether a sale's *hundred* is the only quantity.** Both trades move
  exactly a hundred, and no capture on disk shows a bucket step that is
  not a multiple of it; a `LEADERS=9` window over Great Lakes 9182, where
  the original spends **three** market draws, is the run that would say.
  **What three draws can mean, narrowed (item 362).** The outer `while
  (bucket[g] < need[g])` only repeats when a candidate passed the sell
  test, and a sale raises **wealth**, never the short good. So a
  non-wealth shortfall draws once per pass until the sellable set
  empties — one draw per hundred it can shed — and a wealth shortfall
  stops as soon as the sale covers it. Three draws on one frame is
  therefore *three short goods*, or one good with two sales under it, and
  nothing else. This crate's ledger on 9182 has one short good and
  nothing sellable, so the frame is a value residue upstream of the
  market rather than an arithmetic one. `docs/AI.md` §42.
- **The rotation past the first sale.** 8982 sells once and stops because
  the sale covers the need. A frame where two goods both pass the
  candidate test is not on any capture, so "every qualifying position
  sells" rests on the decompile alone.


## 13. `market_speculation` — the trade with no draw in it (2026-09-22)

`Leader::market_speculation@006c8110` is `production_ai_setup`'s last act
and the only place the AI trades **off** a shortfall rather than to cover
one. §12's `use_market` runs inside `make_stuff` and is what a short good
provokes; this runs one step earlier in the production rotation, reads no
make list at all, and buys or sells on the **stockpile alone**.

Its sell and buy passes were a declared seam in `crates/sim/src/ai_make.rs`
from the day the function was written — both need `calc_market_prices`,
which did not exist then. §12 built it on 2026-09-18 and the comment
outlived it by four days.

**Nothing in the function spends a draw.** That is why it could be missing
for two hundred items without the draw stream saying so, and why Great
Lakes' word sat five frames downstream of it.

### 13.1 Three passes behind one gate

The gate is §12's exactly — `has_tribe_bonus(4)` or `has_preq(BUY_SELL)`,
a live market, no nuclear embargo — plus `starting_resources != 8`.

1. **The tier**, and the escrow clamp beside it. An escrow over 4000 is
   pulled back to 2000; the smallest stockpile decides how hungry the
   leader is — **2** when any available good is under 100, **1** when one
   is under 200 and none under 100, **0** otherwise. Read over *every*
   available good, wealth and knowledge included.
2. **The sell pass**, over the four tradeable goods in index order (not
   wealth, not knowledge): sell a hundred when the unescrowed pile clears
   `2000 >> tier` **and** the sell price clears `75 / (tier + 1)`. A
   hungry leader parts with a smaller pile and takes a worse price.
3. **The buy pass**, over the same four, and it reads **no tier**. Buy a
   hundred when the buy price is within the unescrowed *wealth*, the stock
   is under 2000, the price is under 201, and a ladder that tightens as
   the pile grows: at 500 or more the price must be under 26, at 200 under
   51, at 100 under 101. Each purchase spends wealth and the next good
   re-reads the purse, so a leader can empty it on the first good it wants
   and buy nothing else.

Both passes call `do_sell`/`do_buy` once per qualifying good — the
original's `do { … } while (i < 1)` runs the body exactly once, the same
shape §12.3 reads in `use_market`.

### 13.2 What the capture says, and it is a single frame

run117 is run53's own game at `LEADERS=9` over `[10375, 10620)` — 245
blocks straddling the AI's market rotations at 10382 and 10582
(`docs/RUNS.md`). On sim-frame **10576**, the rotation's `Setup` step, the
original's player 1 does this:

| block | food | timber | wealth |
| --- | --- | --- | --- |
| 10576 | 93 | 86 | **128** |
| 10577 | **194** | 86 | **0** |

A hundred food for a hundred and twenty-eight wealth, on a frame that
spends no draw. This crate bought nothing there and so entered the
rotation's `make_stuff` five frames later holding `94 87 125` against the
original's `194 87 0` — which was Great Lakes' word, in full:

- with no wealth the original's `Leader::can_pay` refuses the head (three
  Horse Archers at 60 timber and 40 wealth each) where this crate's
  `affordable` answers 3 off a purse of 125, so this crate **bought** one;
- its `use_market` then finds wealth short of the shortfall vector
  `0 60 90 0 60 0` and spends the sell branch's draw — which is the
  `Leader::use_market+0x1ed` at index 0 that the sequence parted on;
- and its step machine **disarms** at `Make` where this crate, having
  bought its head, ran a second pass — which is the five
  `Leader::create_units+0x642` draws the *count* parted on one frame later.

One missing purchase, five frames early, and three different-looking
partings downstream of it.

### 13.3 The price was never wrong — the purse was

With the passes in, this crate buys on the original's own frame and still
buys the **wrong good**: timber at 94 rather than food at 128. Probed
inside the pass on 10576, this crate's own prices are

```
food (buy 128, sell 83)   timber (94, 58)   wealth (113, 68)
knowledge (126, 81)       metal (107, 62)   oil (121, 76)     tier 2
```

— and **128 is what the original paid, to the unit**. The buy pass walks
goods in index order, so food is asked first on both sides; the original's
purse is 128 and reaches it at exactly its price, and this crate's is
**124** and does not. Four wealth choose the good.

That four is not this mechanic's. `1/income[2:wealth]` is **960** here
against the original's **992** on all 245 blocks of run117, with
`resources[2:wealth]` and `rate[2:wealth]` (`income / 16`, 60 against 62)
saying it twice more. §13.4.

### 13.4 What it moved, and what it left

| | before | after |
| --- | --- | --- |
| Great Lakes long word, sequence / count | 10582 / 10583 | **10817** / **10817** |
| Great Lakes market draws below the word | 11 | **13** |
| block 10583, keys parted | 1 | **0** |
| Great Lakes endpoint `off` / `unlinked` / `build_diverged` | 55 / 5 / 10 | **53** / **3** / **9** |
| East Indies endpoint `off` / `unlinked` / `build_diverged` | 59 / 12 / 30 | **64** / **10** / **27** |
| East Indies ladder B `off` / `extra` | 45 / 6 | **48** / **14** |
| East Indies ladder C `extra` | 13 | **14** |

East Indies' word holds at 9711 and the golden record's at 626/695. Every
map with a market and Coinage speculates, so East Indies' 24,000th frame
moves too, 14,290 frames past its own word where the stream is nobody's
(`docs/DECISIONS.md` 36 asks for the number rather than a trade).

**The new word is 10817** and its block is `1/28` alone — eight rows, a
collision this crate takes and the original does not (`collide` 1 against
0, `collide_who` 8 against −1) with the stand that follows it. The draw
stream parts on the same frame, three draws against two, the extra
`Guy::set_anim+0x97a < Unit::move_step+0x823`.

**Two rows came under the line with the word and neither is this item's.**
`1/2018 queue[0].cost[0]` on **10782**: both sides queue Horse Archers at
the Stable now and this crate charges 60 timber / 40 wealth where the
original charges **57** / **38**, which is `get_cost`'s and not the
ledger's. **Answered by item 545**: `MILITARY_UNIT_DISCOUNT`
(`docs/AI.md` §56.2). And `1/28 order:coll` on 10618.

### 13.5 Coverage

**Diff-backed**: the frame, the good, the quantity and the price, by
`diff::leader::tests::run117_s_window_is_the_leader_record_across_two_rotations`
— 490 blocks of the whole `LEADERDATA` record, 513,520 field-frames
(515,480 since item 520 added the four epochs, §14.5), with
10576's buy asserted as the four-number pair either side and the residue
pinned field by field. The word itself by
`diff::harness::tests::run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Pinned capture-free**, each made to fail on purpose before landing:

- `sim::ai_make::tests::market_speculation_buys_the_first_good_its_purse_can_reach`
  — run117's own numbers, both halves: a purse of 128 buys the food and
  empties itself, a purse of 127 cannot reach it and buys the timber.
  Fails with the purse test written `<` instead of `<=`.
- `…::the_buy_pass_ladder_gets_stricter_as_the_pile_grows` — all four
  rungs at their boundaries, and the 2000 ceiling.
- `…::the_sell_pass_scales_both_of_its_thresholds_by_the_tier` — the pile
  and the price at each of the three tiers, on both sides of each.

**Not established:**

- ~~**Where the 32 of wealth income comes from.**~~ **Answered by item 661
  (`docs/AI.md` §63)**: the two trade routes, 160 against 176, because the
  Barracks and Stable never joined Norwich; the `ours 0` below was the
  harness comparing against a literal 0. The original text: run107's thirty blocks at
  9170–9199 carry no `income` row at all, so the gap opens inside
  `(9199, 10375]` and no capture covers it. The AI holds two caravans this
  crate does not model (`vans.length` 0 against 1 and `trade_val` 0 against
  176 on both its cities, standing since run89) — which is a candidate and
  not a measurement, because 176 is not 32 and the gap is not there at
  9199. The falsifier is a `LEADERS=2` window over that span: it is thin
  enough to run whole, and it would date the divergence to the frame.
- **Whether the sell pass ever fires in a real game.** Nothing on disk
  shows it: every pile in run117's window is under 300 and the loosest
  threshold is 500. The passes are asserted from the decompile and from
  the unit tests above, and only the *buy* pass is diff-backed.
- **The `2000 >> tier` divisor's form.** The original divides by a 64-bit
  `1 << tier`; the tier cannot exceed 2, so the shift and the division are
  the same arithmetic here. Read, not measured.

## 14. Great Lakes 11185 — the market reads a make-list slot the original has emptied (2026-09-22, item 520)

Item 518 moved Great Lakes' word to **11185**, past every dump on disk,
and booked it by its frame and its draw delta alone: ours nine draws,
the original eight, parting at index 2 — `Leader::use_market+0x1ed`
against `Leader::make_stuff+0x221`. No mechanism was named. This section
is the widening, and it names the input that parts.

### 14.1 The frame, from both sides

run123 is run53's game at run100's detail with `LEADERS=9`, over
`[10760, 11459]` (`docs/RUNS.md`). The word's tick is the production
rotation's `Make2` step: it enters on block **11185** and leaves on
**11186**. §12.3's `use_market` needs the summed costs of the first
`max(1, epoch[Commerce])` make-list slots whose `val` and `t` are both
positive, then spends one sell draw per short good when nothing can be
sold. Player 1 on block 11185, ours / theirs:

| | ours | theirs |
| --- | --- | --- |
| `epoch_get(scan)`, commerce | 2 | 2 |
| `bucket` food, timber, wealth, metal | 13, 1, **11**, 44 | 24, 6, **6**, 47 |
| `MAKE[0]` | Siege Factory (t430) | Siege Factory (t430) |
| `MAKE[1]` | **Cataphract (t227)**, 65 wealth, 75 metal | **`t −1`**, cat 4, val 909,090 |

So this crate's need is `0 60 65 0 135 0`, and wealth, timber and metal
are all short: three draws. The original's slot 1 is skipped (`t ≤ 0`),
its need is the head alone, `0 60 0 0 60 0`, and only timber and metal
are short: two. Everything else is the same arithmetic on both sides.

### 14.2 What was predicted, and what killed it

The stanza (`tools/gamelog/captures.txt`, run123) named four readings
before the run, each with what would kill it. The block decides them:

- **R1, the purse** (parked 514's 32 of wealth income): **dead.** The
  original's wealth is *lower*, 6 against 11, and with its own list it
  needs none. The income gap is on every block, 960 against 992, and it
  does not decide this frame.
- **R3, the commerce level**: **dead.** 2 on both sides, from the four
  `epoch_get(scan)` lines, which nothing in the harness had parsed.
- **R4, the stock**: **dead.** Timber and metal are under the head's 60
  on both sides.
- **R2, the list**: **alive, and it is the frame.** Slot 1 parts.

R5 (a sale covering the good) is also dead: no bucket steps by a hundred
between 11185 and 11186.

### 14.3 What the emptied slot was

The original enters the rotation's `Make` on block 11182 holding a
**Merchant** (`t 61`, cat 4, escrow 1, val 909,090) in slot 1, where this
crate holds the Cataphract. On 11184, after `Units2`'s `create_units`
re-offers a Merchant at 227,272 in slot 0, the old slot reads `t −1` at
its old value. `Buildings2` then puts the Siege Factory at the head, and
the emptied slot is in slot 1 when `Make2` sums the need. Coverage over
11176..11190 has `Leader::create_units`, `MakeList::make_me` and
`TypeData::is_merchant` on 11183, and `MakeList::make_me` on 11184.

The Merchant is `docs/AI.md` §38.5's standing residue: the original has
offered one since at least dump-block 8181, and this crate never does. The
merchant arm of `civilian_value` (`crates/sim/src/ai_units.rs`) returns
`None` unless `known_rares` less the Merchants queued and alive is
positive. This crate sums that total from `census.reg_known_rares`,
which the census zeroes every sweep and **nothing writes**, so the arm
is dead code. On run123's own block 10760 the original already holds
Merchants at `MAKE[3]` and `MAKE[4]` (val 227,272) and this crate a
Phalanx and a Mine.

**So 11185 is a consequence of the Merchant offer, not of (514).** The
Merchant slot has been in the original's list at every rotation since
8181. What this frame adds is the one arrangement where the slot's
*emptying* moves `use_market`'s need.

### 14.4 What is not established

- ~~**The writer of `known_rares`** (`LeaderData +0x6d4`) and of its
  per-region array `reg_known_rares` (`+0x4d4`).~~ **Answered by item 327,
  `docs/AI.md` §55.1–§55.3**: the census's step 9 writes the array,
  `calc_gather` sums it, and `World::compute_reg_territory@006b0bb0` is a
  second writer that zeroes a region's count when its border pass starts.
- ~~**The Merchant's value.**~~ **Answered, §55.2**: the arm hands the
  shared tail a flat 1,000,000, and the tail's `offer_value` produces the
  original's numbers. The arm was missing its
  `has_tech(MATHEMATICS) || type_avail(KNOWLEDGE, 1)` gate. Every Merchant
  slot of nine captures now agrees by value.
- ~~**Why the old slot reads `t −1` on 11184.**~~ **Answered, §55.2**:
  `MakeList::make_me@006c9be0` empties every lower slot of the offered
  type, and this crate already did. The re-offered Merchant empties slot 1
  here too.
- ~~**`1/2016 queue[0].cost` 65/45 against 61/42 on 11183**: the Hoplites
  the rotation buys cost this crate six percent more. That is §13.4's
  `get_cost` row (10782, 60/40 against 57/38) again. It moves the food
  and metal buckets and not this frame's draws.~~ **Answered by item 545,
  `docs/AI.md` §56.2**: `MILITARY_UNIT_DISCOUNT`, 5% at `epoch[0]` 2
  against the unit's level 1, loaded and never applied. Both rows agree
  now, and so do food and metal on every block of run123.

### 14.5 Coverage

**Diff-backed**: every claim in §14.1 and §14.3's list contents, by
`diff::harness::tests::run123_s_word_frame_is_widened_whole`, which
compares every unit record `diff::harness::widen_block` reads and the whole leader
record (plus `epoch_get(scan)`) on all 700 blocks, 2,222,681 unit rows
and 1,472,800 leader rows. The word's inputs, slot 1 on 11182, and the
set of keys that first part on 11185 and 11186 are pinned. **Trace-backed**:
§14.3's functions, from run123's per-frame coverage. **Reading-only**:
nothing. §14.4's first three questions are answered in `docs/AI.md` §55
(item 327).

**The epochs are a row of the record now.** `epoch_get(scan)` is read
by `diff::leader::theirs` as `epoch[0..3]`, and this crate's
`tech[who].epoch` is the other side, so every leader widening compares
it. It agrees on every block of all seven windows: run19, run84, run91,
run107, run115, run117 and run123. The coverage driver runs that reader
too, and its Great Lakes window is run123's word ±2 now, not run100's
10834 (`diff::coverage::UNREAD`).

