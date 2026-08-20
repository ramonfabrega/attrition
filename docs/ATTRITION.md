# Attrition

The mechanic this project is named for: units lose health while standing in
hostile national territory. This is the phase 1 specification.

**How this was established.** Symbol names, struct layouts, and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing here is transcribed — the original
was read to understand it, and this document is the understanding written down.
Implementation follows from this document, not from a decompiler window. See
`docs/DECISIONS.md` entry 7.

**Confidence.** High, and higher than it was. The rate formula is derived and
then independently checked: substituting the shipped constants into it
reproduces the designers' own annotation on `ATTRITION` ("48 frames — the
baseline level for regular attrition") exactly. The territory computation is
now at the same standard: the distance unit, which was the last thing left
open, is settled from the original's own coordinate conversion and confirmed
from a second direction by the designers' `"24 tiles"` and `"44 tiles"`
annotations. What remains open is listed at the end and is small.

**Where the implementation is.** `crates/sim`, in three modules that follow
this document's three sections: `world` for the grid and its units of length,
`territory` for the border computation, `attrition` for the rate and the
damage. `crates/sim/src/lib.rs` runs the whole thing headless. Every number
below is re-read from the user's own install by
`cargo run -p rondata -- <install>`, which fails if any has drifted.

---

## The state it touches

| Where | Field | Type | Meaning |
| --- | --- | --- | --- |
| `WData` (per world cell, 28 B) | `who` @ +15 | `i8` | Owning player, `-1` unowned, `-2` ambiguous |
| | `who2` @ +16 | `i8` | Second claimant |
| | `down` @ +8, `down_who` @ +10 | `i16` | Claim strength and its claimant |
| `UnitData` (344 B) | `attrition` @ +158 | `i16` | Pending tick period, in frames |
| `LeaderData` (28,388 B) | `anti_att` @ +2036 | **`f32`** | Resistance scale, 256 = baseline |
| | *(strength)* @ +2032 | `i32` | Attrition level this player inflicts |
| | *(give disabled)* @ +2040 | `i32` | Scenario flag; suppresses the above |
| | *(take disabled)* @ +2044 | `i32` | Scenario flag; suppresses resistance |
| | `neutral_attrition` @ +2048 | `i32` | Period applied on unowned ground |
| | `attrition_stamp{,2,3}` @ +500/504/508 | `i32` | Warning-message throttles |
| `WorldData` (364 B) | `player_territory_limit{,_civic,_city}` @ +56/60/64 | `i32` | Seeded from `TERRITORY_LIMIT_*` |

`WorldData` carries a **second** triple of limits at +68/72/76. The territory
pass picks between the two on a per-region flag, so some regions are computed
against a different reach than others. Which regions, and why, is not
established.

Two incidental findings worth recording, because they will bite anyone reading
memory or a save file:

- **Unit coordinates are stored XOR-masked with `0x63637`**, the player's age
  with `0x62766`, and a city's stored age with `0x63187`. A tamper-resistance
  measure, not encryption.
- `Unit` derives from `UnitData` and adds no fields; `Leader`/`LeaderData` the
  same. The split is behaviour over state, matching the `GameAccess` /
  `GameAccessConst` accessor layering visible in `obsoletescriptfuncs.txt`.

---

## Three units of length

Everything downstream depends on getting these right, and the original uses all
three within a few lines of each other.

| Unit | Size | Used for |
| --- | --- | --- |
| position unit | 1/768 cell | stored object coordinates |
| **tile** | 1/4 cell = 192 position units | **territory distances** |
| cell | 768 position units | ownership, one `WData` record each |

This is not inferred from the constants' annotations. It comes from the
original's own conversion. Object coordinates are converted by
`div_3_table[pos >> 8]` when a cell is wanted and by `div_3_table[pos >> 6]`
when a tile is; `init_coord_lookup_array` fills that table with `i / 3`, and
for negative indices with `(i - 2) / 3`, which is floor division. So a cell is
`256 × 3` position units and a tile is `64 × 3`. The table exists to make a
divide-by-three cheap in 2002 and means nothing more than that.

The designers' annotations then agree, from the data rather than the code:
`TERRITORY_BASE` is `"24 tiles"` and `TERRITORY_LIMIT_BASE` is `"44 tiles"`,
and both are consumed as distances in exactly this unit. Two independent lines
of evidence, and they meet.

A cell's centre is the tile `4c + 2`. The original works in whole tiles and
takes that `+ 2` as exact, so every distance measured from a cell centre
carries half a tile of bias in each axis — deliberately, since the alternative
in integer arithmetic would have been a fraction.

`UNIT_MOVE_SPEED` is `1/192` of a tile per frame, which is the same 192 seen
from the movement side.

---

## The rate

Attrition is **a period, not a damage number**. A unit standing in hostile
territory accumulates a tick period in frames; the smaller the period, the
faster it bleeds. Frames are fifteenths of a second.

Three quantities combine.

### 1. Strength — what the territory owner inflicts

Recomputed per player by `Leader::calc_attrition`:

1. Count how many steps of the attrition tech chain the player holds. That
   count indexes `ATTRITION_IMPROVED` = `[1, 2, 4, 8]`. No steps means no
   attrition at all, which is where everybody starts.
2. Then apply, in this order, each as `x = (100 + pct) * x / 100` with integer
   truncation and a floor of 1:
   `COLOSSEUM_ATTRITION` (wonder, +50%) → `RUSSIAN_ATTRITION` (nation, +100%)
   → `CTW_ATTRITION` (campaign) → `KREMLIN_ATTRITION` (wonder, +100%).

Order matters: each step truncates, so reordering changes the result. The
floor matters too — at one tech step the value is 1, and +50% of 1 truncates
back to 1 rather than to zero.

Everything at once tops out at 72: `8 → 12 → 24 → 36 → 72`. That number
matters; see the note on the sentinel below.

### 2. Resistance — what the victim brings

`anti_att`, recomputed by `Leader::calc_anti_attrition`, starting at **256.0**
and multiplied by `100 / (100 - pct)` for each of:

| Source | Constant | Effect |
| --- | --- | --- |
| Foraging tier 1 / 2 / 3 | `ATTRITION_UPGRADE` = 25 / 50 / 75 | ×1.33 / ×2 / ×4 |
| Statue of Liberty | `LIBERTY_ATTRITION` = 100 | **immune** |
| Mongols | `MONGOL_ATTRITION` = 50 | ×2 |
| Titanium | `TITANIUM_ATTRITION` = 50 | ×2 |

Any `pct ≥ 100` short-circuits to total immunity, which is how the Statue of
Liberty works.

### 3. Per-unit susceptibility

`UnitData::get_attrition(owner)` combines the two and returns
`resistance / strength`:

- Base scale is `256`. **Siege units** use `25600 / (100 - SIEGE_ATTRITION)`
  instead — 512 at the shipped 50%, so half rate.
- **Militia** ignore `anti_att` entirely and use
  `scale * 100 / (100 + MILITIA_ATTRITION)` — a quarter of the resistance at
  the shipped 300%, so four times the rate.
- Everything else: `scale * anti_att / 256`. Additionally, a player holding
  Foraging tier 1 takes **no** attrition while the unit is idle.
- **Merchants, Dutch merchants and fur trappers** halve the result, and so
  does any type whose attrition mode is `2`. Those three are `TypeIndex` 61,
  62 and 400 — the economic units whose whole job is to stand outside your
  borders, which is why they were singled out.
- Finally, if the owner is at the same age or later than the victim, strength
  is scaled by `(ATTRITION_AGED_UP * ages_ahead + 100) / 100`, **rounding up**.

**Zero is the sentinel for "immune"**, and the caller tests for it rather than
for any of the individual conditions. This has a consequence worth knowing
before anybody rebalances the game: susceptibility is `resistance / strength`
with truncation, so a strength above 256 divides to zero and stops hurting
anybody at all. The shipped data tops out at 72, so it is unreachable in play —
but a mod that raised `ATTRITION_IMPROVED` would walk straight into it.

### Putting it together

```
period_frames = max(1, ATTRITION * (resistance / strength) / 256)
```

with truncating integer division throughout. `ATTRITION` is 48. The final
divide is written as an arithmetic shift with a bias that makes it truncate
toward zero rather than floor; for the non-negative values reachable here the
two agree.

**Baseline check.** A plain unit against a player with one attrition tech:
resistance `= 256 * 256 / 256 = 256`, strength `= 1`, so
`48 * 256 / 256 = 48` frames — 3.2 seconds per tick. That is exactly what the
shipped `ATTRITION` constant's own trailing comment claims, from an entirely
independent direction, which is the check that gives this section its
confidence.

**The floor is not decorative.** At the strongest attrition the shipped data
can produce, susceptibility is `256 / 72 = 3` and `48 * 3 / 256` truncates to
zero. The `max(1)` is therefore what actually sets the fastest attrition in the
game: one tick per frame, fifteen a second.

**Special periods**, assigned directly rather than computed, and halved when
the unit's attrition mode is `2`:

- `PEACE_ATTRITION` = 8 frames — border violation while at peace.
- `ASSASSIN_ATTRITION` = 8 frames — assassin game mode, in the territory of
  somebody who is not your target.

These do **not** replace the computed period; they are assigned first and the
ordinary calculation still runs, with **the smallest period winning**. So the
peace period is a floor on how slowly a border violation can hurt, not a
substitute for the rate. Against a weak player, crossing a peaceful border
hurts six times as much as being in a war zone; against a strong one the
computed rate takes over.

Only units whose attrition mode is `0` ever reach the computed period. Mode `1`
is outright immune and mode `2` gets the halved special periods and nothing
else — which means the mode-`2` halving inside `get_attrition` is unreachable
from the tick, and survives only because the function is called from elsewhere.

---

## Eligibility

`Unit::process_attrition` runs per unit per tick and returns early on any of
these. Ordering is preserved because several are reachable in the same tick and
only the first one reached is the reason.

1. The unit is inside a scenario-defined attrition-free radius.
2. The cell is unowned *and* the victim's `neutral_attrition` is zero.
3. The cell's owner is the unit's own player.
4. The owner is not an active, initialised player.
5. The owner is the victim's team.
6. Both sides hold a mutual treaty of type 2.
7. The victim has take-attrition disabled, or the owner has give-attrition
   disabled (both scenario-scriptable).
8. **Unit kind.** Workers and merchants are subject. Heroes, supply units,
   spies and "special" units are exempt, as are caravans. A worker actively
   gathering at a site flagged exempt is also spared.
9. The unit's type has attrition mode `1` (outright immune).
10. The victim has a non-zero `neutral_attrition`. A player with a neutral
    period set therefore takes *only* that period and never territorial
    attrition — it replaces the mechanic rather than adding to it.
11. In Conquer the World with a particular conquest bonus, and outside a
    specific team style.
12. Inside the `ally_to_war_grace` window after an ally became an enemy.

Check 2 has a wrinkle worth recording. When the ground is unowned and the
victim *does* have a neutral period, the original sets that period and then
keeps going, indexing its per-player arrays at `-1` — a real out-of-bounds
read that stays harmless only because `neutral_attrition` is zero in every
shipped scenario. `crates/sim` returns the neutral period there, which is
plainly what was meant. It is the clearest small example of why the engine is
not the treasure.

The remaining bulk of the function is diplomacy bookkeeping and the throttled
on-screen warning — `ally_to_war_delay` and `ally_to_war_grace` windows, a
900-frame message throttle, `Leader::meet` to reveal a previously unmet player,
and `S_ATTRITION_WARNING`. None of it changes the damage.

---

## The damage

`Unit::suffer_attrition` applies it when the period elapses.

The amount depends only on squad size — RoN units are squads of one to four
figures — via `curr_uber_size`:

| Squad size | Damage |
| --- | --- |
| 1 | 16 |
| 2 | 8 |
| 3 | **6** |
| 4 | 4 |

Size 3 is special-cased to 6 rather than the `16 / 3 = 5` the formula would
give, so the per-squad total stays near 16.

A type flagged at `+0x308` takes a different path: the damage call gets its
first argument set to 1 and an amount of 0, where the ordinary case passes 0
and the amount. That first argument is a damage *kind* rather than an amount,
and what the kind does is unread.

The victim's `who` is read from the world cell at the unit's position; the
warning message and sound are throttled by a per-player frame stamp and only
shown to the local player.

---

## Territory

Ownership is a **weighted-distance Voronoi**, recomputed wholesale rather than
incrementally. `World::compute_all_territory` drives it; `compute_reg_territory`
does the work for one map region.

### The shape of it

- The map is divided into at most 64 land regions and 64 sea regions — hence
  the `BitMask<64>` in `regions.obj`. Territory is computed per land region.
- **Sea is never owned.** Every cell of every sea region has `who` and `who2`
  set to `0xFF` after the land pass, so a coastal border stops at the water
  rather than reaching over it.
- Each player keeps a `u16` territory count per region, summed afterwards into
  a running total and an all-time peak.
- For each cell, every city and fort of every player computes a cost. The
  lowest cost owns the cell and lands in `who`; the runner-up lands in `who2`.
  If the lowest cost exceeds `TERRITORY_BASE << 8` (24 × 256 = 6144) the cell
  stays unowned.
- A claim too expensive to *own* a cell can still be recorded as its
  runner-up. A cell can therefore name a second claimant it would never grant
  to anybody.

### Two independent bounds, not one

A source is clipped twice, by different mechanisms, and conflating them gives
the wrong border.

**The limit** is a hard distance in tiles, tested against the *raw* distance
before the near-field contraction. Beyond it a source does not claim at all —
not even as a runner-up. It is `TERRITORY_LIMIT_BASE` = 44 tiles, plus
`TERRITORY_LIMIT_CIVIC` = 4 per civic tech level and per step of flat bonus,
plus `TERRITORY_LIMIT_CITY` = 4 per city level, per temple level, and per fort
level.

**The cost cap** is `TERRITORY_BASE << 8`, tested against the cost of the
*contracted* distance. With no bonuses at all it permits 52 tiles.

So for a plain city the limit binds first, at 44 tiles — eleven cells — and the
cost curve never comes into it. Bonuses raise both together, which is the point
of raising them together: a bonus that cheapened distance without extending the
limit would buy nothing.

### The cost

```
cost = TERRITORY_DEN * distance * 256
     / (TERRITORY_NUM + CITY_TERRITORY_MULTIPLIER * bonuses)
```

Integer division, `256` being an 8.8 fixed-point scale. Forts use
`FORT_TERRITORY_MULTIPLIER` in place of the city one. This is the reciprocal of
the designers' own comment on `TERRITORY_NUM` —
`(Numerator + (CityorFortMultiplier * BorderBonuses)) / Denominator` — which
describes the radius; the code works in cost per unit distance instead, so a
larger bonus makes distance cheaper and the border reach further.

Bonuses accumulate from:

- `CITY_UPGRADE_TERR` by city level — 0 for a city, 3 for a `TOWN`, 6 for a
  `METROPOLIS` — or `CAPITAL_TERRITORY_BONUS` instead. The **Forbidden City**
  both counts as a metropolis and confers the capital bonus.
- `TEMPLE_UPGRADE_TERR` by temple border tech level, for cities that have a
  temple, raised by 50% by **Tikal**.
- `FORT_UPGRADE_TERR` by fort border tech level, plus `COLOSSEUM_FORT_BORDERS`
  and `ROMAN_FORT_BORDERS`, plus a flat **4** for the **Red Fort** — a literal
  in the code with no constant behind it.
- `CIVIC_UPGRADE_TERR` by civic tech level, for cities and forts alike.
- Flat additions from the Colosseum, the Eiffel Tower, gems, and Russian
  borders (which also scale per age). The Russians take half of each of the
  others, since their own per-age bonus is meant to be the one that counts.
- An AI handicap allowance, `(handicap + 15) / 25`, which is the one bonus that
  buys no matching extension to the limit.

The per-player half of that is recomputed for all eight players at the top of
every region pass; the per-object half is resolved inside the per-cell loop.

### Distance is not Euclidean, and this matters

Two departures, neither of which anyone would arrive at by guessing, and both
of which change border *shape* rather than just size.

**First**, distance is an integer approximation of the hypotenuse, never a
square root. With `hi` the larger of `|dx|`, `|dy|` and `lo` the smaller:

```
hi + lo² / (2·hi)
```

It is exact on the axes and worst on the diagonal, where it returns 1.5× the
leg against a true 1.414×. So the border is pulled *inward* at 45 degrees
relative to a circle, which is why RoN's borders read as faintly octagonal.
Substituting a true `hypot` would visibly change every border on every map.

The original has a second branch, `hi + lo / 2`, selected by `lo < 60000`. That
is an overflow guard on `lo * lo`, not a shape decision — no map is sixty
thousand tiles across, so the branch never runs in play. An earlier draft of
this document read the test as a choice between two approximations; it is not.

The truncation in `lo² / (2·hi)` means the result can also land one *below* the
true distance — `60, 30` gives 67 where the true answer is 67.08. It is never
worse than that.

**Second**, short distances are contracted by three compounding tests, applied
in order to the running value:

```
if d < 13 { d = d * 2 / 3 }
if d <  9 { d = d * 2 / 3 }
if d <  5 { d = d / 2 }
```

Because each test reads the value the previous one wrote, they stack: 8 becomes
5, then 3, then 1, and 4 becomes 2, then 1, then 0. The effect is that the near
field around a city costs almost nothing, so borders bulge close in and taper
further out — and that the gradient is discontinuous, since 12 contracts to 5
while 13 stays 13.

### Ties are broken by the cell's x coordinate

Equal costs go to whichever source is scanned first, and the player scan order
is rotated per cell: player `(i + cell_x) mod 8`. Without it, a frontier
between two evenly matched players would belong entirely to the lower-numbered
one; with it, equal claims interleave in alternating columns.

This is the single detail in the whole mechanic least likely to be arrived at
by reimplementation, and it is directly visible on screen as the dithered look
of a contested border. `crates/sim` generalises the modulus off eight, which is
the only change.

### Ambiguous ownership

When a city's own record of its owner disagrees with the player whose list it
was found under, a winning claim writes `-2` rather than the player index.
Every consumer treats `-2` as unowned — the test everywhere is `who >= 0` — so
it survives only because it is observable in a save file. The original also
resolves such a city's *bonuses* from the owner it records while taking its
*limit* from the list it was found in.

---

## Open questions

- **The tick cadence.** `process_attrition` sets a period and
  `suffer_attrition` applies the damage, but the code that counts between them
  has not been found: `suffer_attrition` is called through a vtable, so there
  are no direct references to follow. `crates/sim` decrements the period each
  tick and re-derives it every tick, so leaving hostile ground stops the
  bleeding at once. That is the straightforward reading and it is an
  assumption, not a finding. `OVERKILL_FRAMES` is 30 and may be involved.
- **The second limit triple** at `WorldData` +68/72/76, and the per-region flag
  that selects between it and the first.
- `down` and `down_who` are named in `WData` and look like a stored claim and
  claimant, but the territory pass writes `who`/`who2` and does not obviously
  touch them. They may belong to a different system.
- What the damage *kind* is that a type flagged at `+0x308` receives.
- Whether `attrition_stamp2` and `attrition_stamp3` matter to the sim or are
  purely presentation.
- Whether the constant Ghidra resolves as `tikal_temple_hp` is really
  `TIKAL_TEMPLE_BORDERS`, which is what its use implies and what its position
  in the shipped file suggests. Both are 50, so the shipped behaviour is
  identical either way and only a mod could tell them apart.

## Not open, and why

`anti_att` is an `f32` in the original, and it sits in the middle of the sim.
It is multiplied by `1/256` and truncated to an integer inside
`UnitData::get_attrition`. Rise of Nations gets away with this because it only
ever shipped one x86 build, so every client executes identical instructions
with identical rounding. We cannot rely on that, and `CLAUDE.md` forbids it
outright.

The float is never doing anything a rational cannot. Its only inputs are `256`
and a chain of `100 / (100 - pct)` factors with integer `pct`, so `anti_att` is
exactly representable as a rational — and the original truncates once, at the
`* 1/256` step, after all the multiplications have compounded. Accumulate the
numerator and denominator and divide once, at the same place.

Nor is `Fx` the right carrier here: it would round at 1/65536 where the
original does not round at all. Exact integer rationals are the only
representation that neither invents error nor imports it. See
`docs/DECISIONS.md` entry 10.

The values are reachable in small enough number to check by hand. Of the
factors in play — 4/3, 2 and 4 from Foraging, 2 from the Mongols, 2 from
titanium — only 4/3 is not a dyadic rational, so `256 · (4/3) · 2^k` is the
only family where a truncation happens at all, and it never lands within a hair
of an integer boundary. The exact rational and the original's `f32` agree on
every reachable value. `crates/sim/src/attrition.rs` pins all eight of them.

Every other quantity is already integer arithmetic with defined truncation, so
it ports directly.
