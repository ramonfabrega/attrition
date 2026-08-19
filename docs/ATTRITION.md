# Attrition

The mechanic this project is named for: units lose health while standing in
hostile national territory. This is the phase 1 specification.

**How this was established.** Symbol names, struct layouts, and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing here is transcribed — the original
was read to understand it, and this document is the understanding written down.
Implementation follows from this document, not from a decompiler window. See
`docs/DECISIONS.md` entry 7.

**Confidence.** The rate formula below is derived and then independently
checked: substituting the shipped constants into it reproduces the designers'
own annotation on `ATTRITION` ("48 frames — the baseline level for regular
attrition") exactly. The territory *computation* is not yet at that standard
and is marked open at the end.

---

## The state it touches

| Where | Field | Type | Meaning |
| --- | --- | --- | --- |
| `WData` (per world cell, 28 B) | `who` @ +15 | `i8` | Owning player, or negative for unowned |
| | `who2` @ +16 | `i8` | Second claimant |
| | `down` @ +8, `down_who` @ +10 | `i16` | Claim strength and its claimant |
| `UnitData` (344 B) | `attrition` @ +158 | `i16` | Pending tick period, in frames |
| `LeaderData` (28,388 B) | `anti_att` @ +2036 | **`f32`** | Resistance scale, 256 = baseline |
| | *(strength)* @ +2032 | `i32` | Attrition level this player inflicts |
| | `neutral_attrition` @ +2048 | `i32` | Period applied on unowned ground |
| | `attrition_stamp{,2,3}` @ +500/504/508 | `i32` | Warning-message throttles |
| `WorldData` (364 B) | `player_territory_limit{,_civic,_city}` @ +56/60/64 | `i32` | Seeded from `TERRITORY_LIMIT_*` |

Two incidental findings worth recording, because they will bite anyone reading
memory or a save file:

- **Unit coordinates are stored XOR-masked with `0x63637`**, and the player's
  age with `0x62766`. A tamper-resistance measure, not encryption.
- `Unit` derives from `UnitData` and adds no fields; `Leader`/`LeaderData` the
  same. The split is behaviour over state, matching the `GameAccess` /
  `GameAccessConst` accessor layering visible in `obsoletescriptfuncs.txt`.

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
   attrition at all.
2. Then apply, in this order, each as `x = (100 + pct) * x / 100` with integer
   truncation and a floor of 1:
   `COLOSSEUM_ATTRITION` (wonder, +50%) → `RUSSIAN_ATTRITION` (nation, +100%)
   → `CTW_ATTRITION` (campaign) → `KREMLIN_ATTRITION` (wonder, +100%).

Order matters: each step truncates, so reordering changes the result.

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
- Three specific unit type ids, and any type whose attrition mode is `2`, halve
  the result (double rate).
- Finally, if the owner is at the same age or later than the victim, strength is
  scaled by `(ATTRITION_AGED_UP * ages_ahead + 100) / 100`, **rounding up**.

### Putting it together

```
period_frames = max(1, ATTRITION * (resistance / strength) / 256)
```

with truncating integer division throughout. Both divisions truncate toward
zero, and `ATTRITION` is 48.

**Baseline check.** A plain unit against a player with one attrition tech:
resistance `= 256 * 256 / 256 = 256`, strength `= 1`, so
`48 * 256 / 256 = 48` frames — 3.2 seconds per tick. That is exactly what the
shipped `ATTRITION` constant's own trailing comment claims, from an entirely
independent direction, which is the check that gives this section its
confidence.

**Special periods**, assigned directly rather than computed, and halved when
the unit's attrition mode is `2`:

- `PEACE_ATTRITION` = 8 frames — border violation while at peace.
- `ASSASSIN_ATTRITION` = 8 frames.

When more than one claim applies, **the smallest period wins**.

---

## Eligibility

`Unit::process_attrition` runs per unit per tick and returns early on any of
these. Ordering is preserved because several are observable.

1. The unit is inside a scenario-defined attrition-free radius.
2. The cell is unowned *and* the victim's `neutral_attrition` is zero.
3. The cell's owner is the unit's own player.
4. The owner is not an active, initialised player.
5. The owner is the victim's team.
6. Both sides hold a mutual treaty of type 2.
7. The victim has take-attrition disabled, or the owner has give-attrition
   disabled (both scenario-scriptable).
8. **Unit kind.** Workers and merchants are subject. Heroes, supply units, and
   "special" units are exempt, as are caravans and one further type. A worker
   actively gathering at a site flagged exempt is also spared.
9. The unit's type has attrition mode `1` (outright immune).
10. In Conquer the World with a particular conquest bonus, and outside a
    specific team style.

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
give, so the per-squad total stays near 16. A separate flagged case deals 1.

The victim's `who` is read from the world cell at the unit's position; the
warning message and sound are throttled by a per-player frame stamp and only
shown to the local player.

---

## Porting notes

**`anti_att` is an `f32` in the original, and it is in the middle of the sim.**
It is multiplied by `1/256` and truncated to an integer inside
`UnitData::get_attrition`. Rise of Nations gets away with this because it only
ever shipped one x86 build, so every client executes identical instructions
with identical rounding. We cannot rely on that, and `CLAUDE.md` forbids it
outright.

The good news is that the float is never doing anything a rational cannot. Its
only inputs are `256` and a chain of `100 / (100 - pct)` factors with integer
`pct`. So `anti_att` is exactly representable as a rational, and `Fx::ratio`
carries it without a float existing at any point.

The hazard is *where the truncation lands*. The original truncates once, at the
`* 1/256` step, after all the float multiplications have compounded. A naive
port that truncates after each factor will drift. Accumulate the numerator and
denominator and divide once — the same lesson `Scalar::scaled_fx` already
encodes for movement speeds, arriving here from a different direction.

Every other quantity is already integer arithmetic with defined truncation, so
it ports directly.

---

## Open questions

- **How territory is actually computed.** `World::compute_all_territory` and
  `World::compute_reg_territory` have been decompiled but not yet read to the
  standard the rest of this document meets. What is established is the
  designers' own formula, left as a trailing comment on `TERRITORY_NUM`:
  `(Numerator + (CityorFortMultiplier * BorderBonuses)) / Denominator`, which
  with the shipped constants is `(11 + 4 * bonuses) / 5` tiles. Border bonuses
  come from `CITY_UPGRADE_TERR`, `FORT_UPGRADE_TERR`, `TEMPLE_UPGRADE_TERR`,
  `CIVIC_UPGRADE_TERR`, `CAPITAL_TERRITORY_BONUS`, and the Colosseum, Eiffel
  Tower, and gems bonuses. Unverified against the implementation.
- What resolves a contested cell — the roles of `who2`, `down`, and `down_who`
  are inferred from their names and not yet confirmed.
- What the three specific unit type ids that take double attrition are.
- Whether `attrition_stamp2` and `attrition_stamp3` matter to the sim or are
  purely presentation.
- The exact tick cadence: `process_attrition` sets a period, but where the
  countdown lives and how it interacts with `OVERKILL_FRAMES` is unread.
