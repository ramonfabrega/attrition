# Attrition

The mechanic this project is named for: units lose health while standing in
hostile national territory. This is the phase 1 specification.

**How this was established.** Symbol names, struct layouts, and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied. Nothing here is transcribed — the original
was read to understand it, and this document is the understanding written down.
Implementation follows from this document, not from a decompiler window. See
`docs/DECISIONS.md` entry 7.

**Confidence.** High. The rate formula is derived and then independently
checked: substituting the shipped constants into it reproduces the designers'
own annotation on `ATTRITION` ("48 frames — the baseline level for regular
attrition") exactly. The territory computation is at the same standard: the
distance unit is settled from the original's own coordinate conversion and
confirmed from a second direction by the designers' `"24 tiles"` and
`"44 tiles"` annotations. The cadence — when a period actually fires — is read
from `Unit::process`, the sole caller of both halves. What remains open is
listed at the end and none of it changes the numbers. **Observed, 2026-08-24:**
the periods 8, 48, 24 and 0, the sixteenths, the phase lock (489 of 489
ticks), the wagon's shelter and its own peacetime bleed, the scout's
exemption and the Nubian clause's deadness were all seen in a logged,
traced run of the original that was predicted before it was read — the
last section of this document.

A blind second reading (`docs/audit/2026-08-20-attrition.md`) doubly confirmed
every formula above and overturned several of the *predicates* around them —
which unit kinds are exempt, what the damage number is a unit of, what two
type fields mean. Those corrections are landed below and marked where they
changed what an earlier draft said.

**One correction to an earlier draft of this document**, which claimed the
cadence could not be found because `suffer_attrition` "is called through a
vtable, so there are no direct references to follow". That was wrong, and
wrong in an instructive way: the Ghidra script behind it looked the symbol up
with `getGlobalSymbols`, which only searches the global namespace and so never
matched a C++ method at all. A lookup that finds nothing reads exactly like a
symbol with no callers. Both functions have one caller and it is ordinary
code.

**Which territory claims a diff backs.** ~~Two~~ Three, and they are
named where they are made. `run40_and_run41_s_sites_and_territory_are_the_original_s`
pins `LeaderData::territory` for both players over two early windows of a
ticked game; `run80_s_gem_widens_the_ai_s_border_by_forty_three_cells` pins
it at the far end of a 24,000-frame capture from a seeded state, and is
what carries the gem term. **And `chapter_four_s_border_is_widened_cell_for_cell`
(item 552, run132, golden chapter four) pins every cell's `who` and `who2`
across three staged levers**: a Temple beside the capital (266 → 296
cells), Religion (temple border level 2, → 327) and Civic 3 (→ 445). So
the **temple arm** (a city whose `city_flags` carry `0x80` adds
`TEMPLE_UPGRADE_TERR[level]` and `level × TERRITORY_LIMIT_CITY`), the
**temple level** (`1` plus the highest `TEMPLEBORDERS2..4` held) and the
**civic term** at level 3 are diff-backed now, cell for cell, and with them
the contraction and the tie-break on every cell those three moved. Still
reading-only: the fort, Colosseum, Eiffel and handicap arms, Tikal's
scaling, and the temple levels above 2.

**The budgeted recompute is observed, and this crate does not model it.**
run132 shows each lever's change land **five blocks after its line and
over five blocks** (306–310, 406–411, 505–511), which is
`GameDaemon::check_borders`' 256 cells a frame working through the
invalidated regions. This crate recomputes wholesale on the line's own
frame, so the two part on those blocks and agree on every block after
them. What sets the five-block delay before the first cell changes is not
read.

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
| `SubObjectData` (28 B) | `who` @ +9 | `u8` | Owning player |
| | `o` @ +10 | `i16` | Index in the owner's object list; **phases every periodic thing the unit does** |
| | `x_internal` @ +16, `y_internal` @ +20 | `Coord` | Position, XOR-masked |
| `ObjectData` | `damage` @ +36 | `i32` | Hit points lost so far; the unit dies when it reaches `hits` |
| | `damage_frac` @ +59 | `i8` | Sixteenths of a hit point not yet carried into `damage` |
| `UnitData` (344 B) | `attrition` @ +158 | `i16` | Pending tick period, in frames |
| | `inside_up` @ +130 | `i16` | Sign bit set means on the map |
| | `o_up`, `o_down` | `i16` | Links to the other figures of the same squad |
| `ObjectTypeData` | `domain` @ +536 | `i32` | 0 land, 1 sea, 2 air — values from `num_aircraft_here` / `in_a_ship` |
| | `uber_size` @ +776 | `i32` | Figures per squad |
| | *(attack)* @ +488 | `i32` | Base attack; zero makes the type exempt |
| `LeaderData` (28,388 B) | `who` @ +8 | `i32` | The player's own slot index |
| | `anti_att` @ +2036 | **`f32`** | Resistance scale, 256 = baseline |
| | *(strength)* @ +2032 | `i32` | Attrition level this player inflicts |
| | *(give disabled)* @ +2040 | `i32` | Scenario flag; suppresses the above |
| | *(take disabled)* @ +2044 | `i32` | Scenario flag; suppresses resistance |
| | `neutral_attrition` @ +2048 | `i32` | Period applied on unowned ground |
| | `attrition_stamp{,2,3}` @ +500/504/508 | `i32` | Warning-message throttles |
| `WorldData` (364 B) | `player_territory_limit{,_civic,_city}` @ +56/60/64 | `i32` | Seeded from `TERRITORY_LIMIT_*` |

`WorldData` carries a **second** triple of limits at +68/72/76:
`colonized_territory_limit{,_civic,_city}`. The territory pass picks the
player triple for a region whose `Region::flags` has bit 4 set and the
colonized triple otherwise, so some regions are computed against a different
reach than others. Which regions carry the flag, and why, is not established.

Two incidental findings worth recording, because they will bite anyone reading
memory or a save file:

- **Unit coordinates are stored XOR-masked with `0x63637`**, the player's age
  with `0x62766`, and the player's civic level — `LeaderDataEncrypt::epoch[1]`,
  which the territory pass reads — with `0x63187`. A tamper-resistance measure,
  not encryption. (An earlier draft called the last one "a city's stored age";
  it is the leader's civic level, and it indexes `CIVIC_UPGRADE_TERR`.)
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
  does any **air** unit (type domain `2`; see below). Those three are
  `TypeIndex` 61, 62 and 400 — the economic units whose whole job is to stand
  outside your borders, which is why they were singled out.
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

**Special periods**, assigned directly rather than computed, and halved for
air units:

- `PEACE_ATTRITION` = 8 frames — border violation while at peace.
- `ASSASSIN_ATTRITION` = 8 frames — assassin game mode, in the territory of
  somebody who is not your target.

These do **not** replace the computed period; they are assigned first and the
ordinary calculation still runs, with **the smallest period winning**. So the
peace period is a floor on how slowly a border violation can hurt, not a
substitute for the rate. Against a weak player, crossing a peaceful border
hurts six times as much as being in a war zone; against a strong one the
computed rate takes over.

**Which units reach which period is decided by the type's domain**, the `i32`
at `ObjectTypeData + 0x218`, which the PDB names `domain` (between `armor`
and `los`): `0` land, `1` sea, `2` air. The values come from
`ObjectData::num_aircraft_here` counting types with `2` and
`ObjectData::in_a_ship` / `Unit::add_to_army` testing for `1`, consistently
everywhere. (An earlier draft called this a three-way "attrition mode", and
the audit's reading called the field unnamed; neither is right. The code paths
were right; the meaning was not.) Only **land** units ever reach the computed
period. **Ships** are
outright immune — eligibility check 9 below. **Aircraft** get the halved
special periods and nothing else: in a war zone with no assassin target an
aircraft takes no attrition at all, and the air halving inside
`get_attrition` is unreachable from the tick, surviving only because the
function is called from elsewhere.

---

## Eligibility

`Unit::process_attrition` runs per unit every 32 frames — see the cadence
below — and returns early on any of these. Ordering is preserved because several are reachable in the same tick and
only the first one reached is the reason.

1. The unit is inside a scenario-defined attrition-free radius.
2. The cell is unowned *and* the victim's `neutral_attrition` is zero.
3. The cell's owner is the unit's own player.
4. The owner is not an active, initialised player.
5. The owner is `leaders[victim].who` — the victim's own slot index again.
   There is no team concept here; this is check 3 a second time, and it can
   never fire. (An earlier draft read it as a team test. The implementation
   had grown a `team` field to match, defaulting to 0, which exempted every
   unit on player 0's ground until a test set it by hand. Both are gone.)
6. Both sides hold a mutual treaty of type 2.
7. The victim has take-attrition disabled, or the owner has give-attrition
   disabled (both scenario-scriptable).
8. **Unit kind.** The block is nested, and the nesting is the rule. A
   **worker** is exempt only while gathering from a site whose
   `GatherOrder::non_flat_gather` is set — a property of the building type
   being gathered from (set in `Unit::add_gather_order` when the type's
   vslot `0x94` returns 0 and the type is not the University), not a scenario
   flag. **Merchants, heroes and supply units** skip the rest of the block
   and are **not** exempt here. Everything else is exempt if its type has
   **zero base attack** (`+0x1e8`), is flagged `is_special` — `unit_flags2 &
   0x10`, which `UnitType::init_final_flags` sets for the **scout** line, so
   that is concretely what "special" means here — is a spy
   (`TypeIndex` 0x3a), or is a caravan. (An earlier draft had heroes and
   supply units exempt and did not have the zero-attack gate at all; supply
   units get their own, conditional, exemption at the computed-period step
   below.)
9. The unit's type is **sea** domain — ships are outright immune.
10. The victim has a non-zero `neutral_attrition`. A player with a neutral
    period set therefore takes *only* that period and never territorial
    attrition — it replaces the mechanic rather than adding to it.
11. In Conquer the World with a particular conquest bonus, and outside a
    specific team style.
12. Not at war, with an alliance broken less than `ally_to_war_grace` frames
    ago — "alliance broken, not yet at war". This lives in the *not-at-war*
    branch, and with shipped data it is **dead code**: `ALLY_TO_WAR_DELAY`
    and `ALLY_TO_WAR_GRACE` are absent from `rules.xml`, `Constants::get_item`
    returns `-1` for a missing attribute, and with a delay of `-1` every
    "recently broke alliance" test is already past its window. Only a mod that
    adds the two constants can make it fire.
13. A **land supply unit** whose bleed did not come from the peace or
    assassin path. The test is on the `0x400000` flag those two paths set:
    clear, and a supply unit returns with no period; set, and the wagon bleeds
    like anything else — the peace period *and* the computed one, smaller
    wins. So a supply wagon is safe in a war zone and bleeds over a peaceful
    border. (An earlier draft had wagons exempt outright.)

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

## The cadence

`Unit::process` is the only caller of both halves, and the cadence is **not a
countdown**. Everything is phased against `SubObjectData::o` — the unit's own
index in its owner's object list — so that a hundred units do not all do their
upkeep on the same frame. Writing `phase` for `frame + o`:

```
if phase % 16 == 0 {              // the per-unit upkeep block
    process_cloak()
    if phase % 32 == 0 {
        attrition = 0             // process_attrition clears it on entry
        process_attrition()       // and may set it again
    }
}
if attrition != 0 && phase % attrition == 0 {
    if !process_supply() { suffer_attrition() }
    else { mark sheltered }
}
```

Three consequences, none of which a countdown would give, and all of which are
observable in play:

- **The damage is phase-locked to the global frame**, not to when the unit
  entered hostile ground. A unit whose period changes from 48 to 24 re-locks to
  the 24-frame grid immediately, mid-interval.
- **The period is refreshed only every 32 frames.** Leaving hostile territory
  does not stop the bleeding at once — the stale period survives until the next
  refresh, so a unit can take one more tick up to two seconds after walking
  out.
- **And the same in reverse: walking in costs nothing until the next refresh.**
  A raid that is in and out inside 32 frames of a unit's own phase takes no
  attrition at all. Whether that was designed or merely tolerated, it is what
  the code does.

Because the two moduli share the phase, a unit with a 48-frame period lands a
tick and a refresh on the same frame every 96.

## Supply is the counter to attrition, and this is where that happens

`Unit::process_supply` gets first refusal on every tick that comes due. It
returns "supplied" — and the tick is dropped, with only a display flag set —
when all of:

- the unit is not flagged as a peace or assassin bleed (see below);
- the unit is not itself a supply unit;
- the unit is not militia;
- and `Supplies::find_supply` finds one of the player's own supply sources
  within its radius, or one of Darius, Kutosov and Chandragupta is standing
  within their own aura.

So a supply wagon does not reduce attrition; it **cancels** it. That is the
mechanic that lets an army campaign abroad at all, and it is implemented as a
veto on the damage rather than as anything to do with the rate — the period is
still computed, still set, and still shown in the interface.

Two exclusions carry weight. **Militia are never sheltered**, which is the
other half of why they take four times the rate: they are the emergency
defenders of your own ground, and the game declines to let them campaign behind
a wagon. And the **peace and assassin paths both set the same flag,
`0x400080`, and the supply check gives up immediately on `0x400000`**, so a
supply wagon protects an army in a war zone but does nothing for a unit caught
over a border in peacetime — and, per eligibility check 13, that is also the
state in which the wagon itself bleeds.

The supply network itself — what registers as a source, how far it reaches, and
the two other things it does that have nothing to do with attrition — is
`docs/SUPPLY.md`. One fact from it belongs here, because it is the first thing
a player gets wrong: **supply is per player and never shared.** An ally's
wagon does nothing for your units.

## The damage

`Unit::suffer_attrition` applies it when the period elapses, and **the unit it
applies it to is one figure, not one squad.** RoN units are squads of one to
four figures, and each figure is its own `UnitData` in the owner's unit list,
linked to the others through `o_up` / `o_down`; each runs its own
`Unit::process`, its own cadence and its own `suffer_attrition`. The squad is
the thing the player selects; the figure is the thing that bleeds.

The amount is in **sixteenths of a hit point**, and depends only on how many
figures the squad currently has — `curr_uber_size`, which walks up to the
captain and counts down the chain:

| Figures in the squad | Sixteenths per figure per tick |
| --- | --- |
| 1 | 16 |
| 2 | 8 |
| 3 | **6** |
| 4 | 4 |

Size 3 is special-cased to 6 rather than the `16 / 3 = 5` the formula would
give, so the per-squad total stays near 16 — that is, near **one hit point per
squad per tick**. A lone figure loses one whole point a tick; a figure in a
squad of four loses one whole point every fourth tick.

The sixteenths go through `Object::take_damage(whole, sixteenths, …)`, which
accumulates them in `ObjectData::damage_frac`: `acc = damage_frac + sixteenths;
damage += whole + acc / 16; damage_frac = acc % 16`. Nothing is ever lost to
truncation; a squad of three carries its 18/16 forward exactly. The figure dies
when `damage` reaches its share of the type's hits.

A type whose `uber_size` (`+0x308`) is exactly 1 takes the same call with
`(whole = 1, sixteenths = 0)` — one whole point directly, which is what
sixteen sixteenths comes to anyway. (An earlier draft read that first argument
as a damage *kind* and left it open. It is the whole-point half of the same
amount, and the question is closed.)

**At the baseline rate this is slow.** One attrition tech, a lone figure, 48
frames a tick: one hit point every 3.2 seconds, so a hundred-point unit lasts
over five minutes. That is the right order of magnitude for the mechanic the
game actually has — a campaign abroad without supply is a slow bleed, not a
rout — and it is sixteen times slower than an earlier draft of `crates/sim`
applied it, which took the sixteenths as whole points.

The victim's `who` is read from the world cell at the unit's position; the
warning message and sound are throttled by a per-player frame stamp and only
shown to the local player.

---

## Territory

Ownership is a **weighted-distance Voronoi**. `World::compute_reg_territory`
does the work for one map region, and is driven two ways:

- **At setup**, `World::compute_all_territory` runs every region to completion
  with the cell budget set to "unlimited" (`GameDaemon::borders = -1`).
- **In play**, `GameDaemon::check_borders` runs every frame with a **shared
  budget of 256 cells per frame**. A region whose borders have been
  invalidated (`Region::fix_borders` resets its resume index, `Region::borders`
  at +0x2c) is recomputed cell by cell, region by region, picking up next frame
  where the budget ran out; per-player territory totals and the all-time peak
  are only re-summed once every region is complete. So after a city is
  captured, a cell can carry **stale ownership for up to `land cells / 256`
  frames**, and `process_attrition` reads whatever is there.

~~`crates/sim` recomputes wholesale, every region, whenever territory changes.
That is a deliberate simplification: steady-state ownership is identical, and
the transient — a few frames of stale borders on a large map — is not worth
reproducing until a recorded-game diff says it matters.~~ **A diff said so on
2026-09-28** (item 1106, `docs/AI.md` §84): East Indies' AI citizen reads its
new city's cell in the colonist gate, and the pass reaches it four ticks after
the fix. `crates/sim::border_pass` now copies the fix's owners onto the map at
256 cells a frame, land regions in order; a fix before the first frame is
setup's and lands whole. `reg_terr` is still read off the visible owners
rather than zeroed and recounted (§84.7). (An earlier draft said the original
recomputed wholesale too. It does not.)

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
- The runner-up bookkeeping differs between the two source kinds, and it looks
  like an oversight rather than a rule. When a **city** becomes the new best,
  the previous best is demoted to runner-up only if there was one; when a
  **fort** does, the previous best is copied unconditionally — so a fort that
  is the first in-cap claim after an over-cap city was recorded as runner-up
  *erases* that runner-up. It touches `who2` only, never `who`, so it has no
  ownership or attrition consequence. `crates/sim` matches it, because it is
  one line and the alternative is being silently different.

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
- An AI handicap allowance, `(get_handicap() + 15) / 25` — **not** the raw
  `handicap` field — which is the one bonus that buys no matching extension to
  the limit, and the one no single-player game can reach at all.

The per-player half of that is recomputed for all eight players at the top of
every region pass; the per-object half is resolved inside the per-cell loop.

**Recomputed, not cached — and that is behaviour** (2026-08-31, item 113).
`compute_reg_territory@006b0bb0` lines 125–260 build the whole eight-row table
on *every* pass, so a leader's borders widen on the frame its Civic level
rises. The level is `data_encrypted->epoch[1] ^ 0x63187` — the same field
`LeaderData::get_city_limit` reads, so `epoch[1]` is Civic and `epoch[0]` is
Military (queue item 107) — and it is read **twice** in the same loop body:
once for `CIVIC_UPGRADE_TERR` and once for the Russians' flat bonus, whose
"per age" is therefore per *Civic level*, not per age. `crates/sim` built the
table once in `Sim::add_player` and never rewrote it; [`Sim::player_borders`]
now reads it live and `sync_territory` rebuilds all eight rows, with
`apply_gained` re-syncing when a gain moves the row. The cost of getting this
wrong was not the border: `WorldData::was_seen@006b53f0`'s **first** arm is
territorial — a cell of an ally's, and `is_ally` is reflexive — so twenty-nine
cells the AI should have owned were *dark* to the site scorer, and it founded
its second city in the wrong place (`docs/AI.md` §18).

Four inputs were seams here. **run80 split them three ways** (2026-09-06,
Great Lakes `[23960, 24000)` at `LEADERS=9`, verified identical to run53
over all 24,001 frames):

- **The gem term is live, it is modelled, and it is diff-backed**
  (2026-09-06, item 117). Player 1's `rares_collected[44]` reads
  `{11, 13, 23, 27}`. The array runs over resources **6–49** — the six
  below it are the base goods, which `escrow[]`'s six entries confirm
  independently — so offset by six the four are Amber, Tobacco, **Gems**,
  Wool. What settles the indexing is that a `BEGIN GOOD` record names its
  resource in words: the start dump's 35 goods are Oil ×14, Fish ×12, then
  one each of Amber, Dye, Tobacco, Cotton, Wool, **Gems**, Citrus,
  Aluminum, Rubber. All four offset-6 readings are on this map and none of
  the offset-0 readings (Silk, Salt, Bison, Sugar) is.

  **The predicate, from the type record rather than the code around it.**
  `compute_reg_territory`'s arm tests `field_0x6da6 & 0x80` or
  `field_0x6dce & 0x80`. `LeaderData` has `rare` at `+0x6d98` and
  `rare_conquest` at `+0x6dc0`, both `BitMask<44>` — `bits`, `size`,
  `flags`, then `ptr` at `+0xc` — so the two bytes are `rare.ptr[2]` and
  `rare_conquest.ptr[2]`, and `0x80` there is **bit 23**.
  `LeaderData::has_rare@006e0770` indexes `good - 6`, which puts bit 23 on
  resource 29, `GEMS`. The arm is an inlined `has_rare(GEMS)` and nothing
  else. `Leader::calc_gather@006ceee0` corroborates it from the other
  side: it maintains `rare = rare_owned | rare_conquest` and, of the
  forty-four bits, calls `Regions::fix_all_borders@0067f7d0` when **`ptr[2]
  >> 7` alone** has moved. `rare_conquest` has no writer outside the
  constructor and `Leader::init@006e3930`'s clear, and the dump prints it
  empty, so `rare` is the whole of the test here.

  **The assignment is not a departure.** The gem arm *sets* the flat slot
  where the Colosseum and Eiffel arms add to it — but the line above zeroes
  that slot, so gems-first-then-add is the same shape, and its one step
  onto the limit is `+= TERRITORY_LIMIT_CIVIC`.
  `crates/sim/src/territory.rs` builds them in that order. And
  `fix_all_borders` only zeroes each region's border stamp (64 records,
  stride `0x88`, offset `0x2c`), so the sweep it schedules reads the *new*
  mask: the decompile's assign-after-invalidate order is not observable,
  and `Sim::tick` assigns first for that reason.

  **What the diff pins.** `run80_s_gem_widens_the_ai_s_border_by_forty_
  three_cells` seeds the world and regions from run80's own start dump, the
  three cities from the window's `CITY` records and each leader row's Civic
  level and rare set from the frame itself, then runs the border pass:
  **266 and 568**, the original's own `LeaderData::territory`, on all forty
  frames. With the gem bit taken back out player 1 falls to **525**, so the
  term is worth **forty-three cells** in this configuration — which the
  earlier draft of this bullet had down as unestablished. The seed reads
  `rares_collected[44]` rather than the `rare` mask because
  `BitMask::log_data` prints each byte with `%u` and no separator
  (`040128800`), which is not uniquely decodable; the `int[44]` beside it
  is written by the same walk.

  Still **not** established: the flat addition and the limit step fire
  together, so one frame cannot separate them — 43 is their sum for two
  Small Cities at Civic 2 — and nothing here checks *when* the bit arrives.
  Both want a capture either side of the Merchant reaching the Gems.
- **The two building terms are blocked at their prerequisite**, which is
  stronger than "did not fire": at 23,999 player 1 holds 27 buildings and
  player 0 five, with no Temple (437), no Fort (443), no Fortress (445),
  and `fort_mark 0` — so `has_preq(TEMPLEBORDERS2..4)` and
  `has_preq(FORTBORDERS2..4)` (bonus types `0x2c8..0x2ca` and
  `0x2d1..0x2d3`, which the loader reads as `bonus_preqs` and does not
  expose) cannot be true in this game at all. The Colosseum and Eiffel
  terms are *reachable* — player 1 built the Pyramids, so the AI does
  build wonders here — and simply are not reached by 24,000.
- **The handicap term is unreachable, and not because the lobby happens to
  hold 0** (2026-09-06, capture lane; the earlier "a click, not a longer
  wait" was wrong). `compute_reg_territory@006b0bb0:255` takes it only when
  `leader_flags & 4` — an AI — **and** `Game::semaphore` bit 2 are both set,
  and that bit is set in **exactly one function in the executable**:
  `Game::run_gamespy@00587060`, the multiplayer lobby. `Game::run_solo@00587830`,
  `Game::run_scenario@005860c0`, `Game::run_editor@00586440` and
  `RecordGame::read_package@00952d90` all **clear** it; the console sets
  `game->semaphore` bits 1, 11 and 12 and never 2. The only sibling consumer,
  `ObjectData::train_time@006508c0`, and the lobby handicap's one other
  reader, `Game::init_teams@0058ae70`, carry the same gate. So `handicap 0`
  in every dump understates it: the branch is dead in any game run here.
- **And the value is not the field.** `LeaderData::get_handicap@006da740`
  returns `handicaps.list[handicap].DATA`, which `rules.xml` fills with
  `index * 5` over 21 entries, so the allowance runs **0 to 4**, not the 0 or
  1 the raw field gives. Nor is `LeaderData::handicap` the lobby's
  `PLAYERn_HANDICAP`: `Game::init_handicaps@0058abf0` makes it
  `clamp(strongest team's summed handicap - own team's, 0, 20) /
  max(num_teams - 1, 1)` — a catch-up deficit, so equal lobby handicaps leave
  every leader at 0 and the *weaker* side is the one paid.

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

This is a shared primitive rather than something the territory pass invented:
the engine has it as a free function, `vector_dist`, which the territory pass
inlines and `Supplies::find_supply` calls. Supply radii are octagonal in
exactly the way borders are. `crates/sim` therefore keeps one copy, in `world`,
under the engine's own name.

The original has a second branch, `hi + lo / 2`, selected by `lo < 60000`, and
computes both **unsigned**. The guard is an overflow guard on `lo * lo` — for
`unsigned`, and in world units, not tiles: 60,000 units is 78 cells, and
`lo²` passes `i32::MAX` from `lo = 46341`, sixty cells short of it. The
captured maps are 60 cells square, so their longest diagonal leg is 46,080
and neither the guard nor the overflow is reached in them; a larger map
reaches both. `crates/sim` squared in `i32` until the emulated original
(`docs/EMULATOR.md`) answered 89998 for `(59999, 59999)` on 2026-09-01. An
earlier draft of this document read the test as a choice between two
approximations; it is not.

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

## The building half

Units are not the only things that bleed. `Wall::process` — `docs/CITIES.md`
§9.5 — checks every 16 frames (phased by `o`) whether the tile under a
building is owned by an enemy and, if so, removes an unstarted site outright
and puts eight hit points through `take_damage` with the attrition flag on a
started one. It does not go through `process_attrition`, `calc_attrition` or
supply; it is its own rule, and it lives with the buildings.

## Open questions

- ~~**The namesake has never fired in a scored capture.**~~ **It has now:
  golden chapter four (item 552, run133) scores it, tick for tick, to 1337**,
  and since item 567 on every block to 1500
  — "Golden chapter four" below. And the measurement this bullet stood on
  was vacuous: this crate never wrote a player's attrition strength from
  the tech tree, so every war-zone refresh was the sentinel 0 whatever the
  original would have done. Measured 2026-09-18
  by item 382, instrumented at `attrition_for`'s refresh: **zero non-exempt
  attrition outcomes for either leader across run53's 24,000 frames.** Both
  long captures are idle-human-versus-AI games whose armies never stand in
  hostile borders long enough, so every claim in this document below the
  reading line is still reading-backed rather than diff-backed, and no
  amount of further long-capture work will change that. The mechanic that
  names the project is the strongest argument for the golden record's
  **Temple chapter** (item 365, `docs/DECISIONS.md` entry 41 §1): a staged
  chapter is the only thing that will ever exercise it.

- ~~**The second entry into `suffer_attrition`.**~~ Closed by the second
  reading: it is the **decoy** path. The `else` of `(unit_flags2 & 2) == 0` in
  `Unit::process` runs a counter at `+0x96` against
  `decoy_time * (general_upgrade + 2) / 2`, and while the decoy is on the map
  and the half-cell visibility byte (`div_3_table[pos >> 7]`) has bits outside
  the owner's ally mask, it calls `suffer_attrition(frame % 7 == 0)` every
  frame — a seen decoy dissolves at one hit point a frame. Same damage
  routine, not the border mechanic.
- ~~**The second limit triple** at `WorldData` +68/72/76, and the per-region flag
  that selects between it and the first.~~ The triple is
  `colonized_territory_limit{,_civic,_city}` and the selector is
  `Region::flags & 4` (set → player limits, clear → colonized). **Still open:**
  which regions carry the flag, and what sets it.
- `down` and `down_who` are named in `WData` and look like a stored claim and
  claimant, but the territory pass writes `who`/`who2` and does not obviously
  touch them. They may belong to a different system.
- ~~What the damage *kind* is that a type flagged at `+0x308` receives.~~
  `+0x308` is `uber_size`; see the damage section.
- ~~Whether `attrition_stamp2` and `attrition_stamp3` matter to the sim or are
  purely presentation.~~ `attrition_stamp2` (`+0x1f8`) is the once-per-game
  "your units are suffering attrition" message throttle in
  `suffer_attrition` — presentation. `attrition_stamp3` is still unseen.
- Whether the constant Ghidra resolves as `tikal_temple_hp` is really
  `TIKAL_TEMPLE_BORDERS`, which is what its use implies and what its position
  in the shipped file suggests. Both are 50, so the shipped behaviour is
  identical either way and only a mod could tell them apart.
- ~~**Tribe bonus 4 counts one attrition tech step as held** in
  `Leader::calc_attrition` — the step whose `TypeIndex` Ghidra resolves as
  `BUY_SELL`. The code is there; which step that actually is stays unresolved,
  because the `TypeIndex` enum is not in the type stream. A logged run with a
  Nubian player (`docs/ORACLE.md`) would settle it.~~ **Settled 2026-08-24,
  twice.** By reading: with the enum dumped whole, the loop runs
  `ATTRITION1..ATTRITION4` = 733–736 and `BUY_SELL` is 685, so the
  `has_tribe_bonus(4)` arm can never be reached — the same dead macro arm
  `docs/SUPPLY.md` found in `get_supply_upgrade` and its siblings. By
  observation: in run16 a British squad stood on the Nubian human's ground
  at war for 98 frames and three refreshes with `attrition 0`, and read 48
  within four frames of the human being granted `Allegiance` ("Behavioural
  check" below). The tribe bonus contributes nothing. The earlier attempt:
  **Attempted 2026-08-20 and
  still open, but the search is now narrower.** A Nubian game was run and
  dumped whole; the leader carries `att` and **`anti_att` as a float**, and
  both the Nubian (`tribe 4`) and the British computer (`tribe 11`) read
  `att 0`, `anti_att 256.000000` at the start and in a later frame alike. So
  the tribe bonus does **not** pre-bias the stored value: `anti_att` sits at
  its base 256 (i.e. ×1) until `calc_attrition` actually runs, which needs
  units taking attrition. The check therefore has to be a *running* one — a
  Nubian unit and a non-Nubian unit inside hostile borders, `anti_att`
  compared between the two leaders — and that needs a unit order, which the
  cheat vocabulary does not provide (`docs/ORACLE.md`).
  Two useful by-products: `anti_att` is logged as a float and prints as
  `256.000000`, confirming the `f32` this document objects to on the
  determinism ground; and `attrition_stamp3` **is** in the dump, next to its
  two siblings, all zero at start — so the field this document lists as
  "still unseen" is at least observable now.
- In Conquer the World, the Tikal temple bonus is further scaled by
  `(100 + CTW_MISSIONARIES_BONUS) / 100` from a leader byte at `+0x6916`.
  Campaign-only; recorded, not implemented.

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

---

## Second reading (2026-08-20) — landed

A blind second derivation and its adjudication are in
`docs/audit/2026-08-20-attrition.md`. The strength chain, the resistance
rational, `get_attrition`, the period/floor, the 32-frame cadence, the supply
predicate and the whole territory formula (contraction, tie-break, the `-2`
owner, the limits) are **doubly confirmed**. Twelve disagreements, nine
resolved against the earlier draft of this document, all landed above and in
`crates/sim` (`attrition.rs`, `lib.rs`, `territory.rs`) on the same day; each
place that changed says so inline. The two that changed observable behaviour:
damage is sixteenths of a hit point per figure, and supply units and heroes
bleed. The one point resolved for the earlier draft: the assassin path sets
`0x400080`, so it defeats supply as written here.

---

## Behavioural check (run16, 2026-08-24) — every prediction observed

Until this run **no traced game had ever entered `process_attrition`'s
bleeding half**: `suffer_attrition`, `process_supply`, `get_attrition`,
`find_supply` and `calc_attrition` were on the blind list the draw-site
trace produces (`docs/ORACLE.md`, "The draw-site trace and function
coverage"). Run16 is the run that exercised them — the same lobby and seed
as run12–14, a hoplite squad and a supply wagon placed by cheat inside the
other player's borders, diplomacy and techs switched by cheat, 6,872 frames
at `UNITS=3`, under the trace. The recipe and its traps are in
`docs/RUNS.md`, "The attrition run"; the predictions were written down
before the log was read, and `tools/gamelog/attr.py` is the reader (every
unit's period timeline, every tick in sixteenths, and the phase-lock fit).

| claim above | predicted | observed |
| --- | --- | --- |
| The cadence: refresh at `(f + o) % 32 == 0`, tick at `(f + o) % period == 0`, `f` the sim-frame (log label − 1) | every tick on the grid | **489 of 489 attrition ticks** across seven units fit at label offset −1, **0 of 489** at offset 0; the three figures of a squad refresh one frame apart, highest `o` first |
| Peace: `PEACE_ATTRITION` = 8, assigned regardless of the owner's tech | period 8, owner at zero strength | `attrition 8` on the first refresh after placement; the owner held no attrition tech |
| Sixteenths by squad size — 6 per figure for three, 16 for one | +6/16 per hoplite figure, +16/16 (whole point, `frac` untouched) per wagon | exactly that, every tick |
| A wagon bleeds over a peaceful border (check 13's flag set) | period 8, one point a tick | `attrition 8`, `damage` +1 every 8 frames, 90 points in 720 frames |
| The scout is exempt (`is_special`, check 8) | never bleeds | 290 frames inside the same border at peace, `attrition 0`, `damage 0` |
| War with no attrition tech: strength 0 is the sentinel | period → 0 at the next refresh; the stale 8 survives until then | `attrition` 8 → 0 six to nine frames after the diplo change, on each figure's own 32-grid; `damage` frozen from there |
| `Allegiance` = one step → strength 1 → `48 × 256 / 256` | 48 | `attrition 48`, +6/16 every 48 frames (the AI's hoplites on the human's ground, after the human's grant) |
| `Allegiance` + `Oath of Fealty` = two steps → strength 2 | 24 | `attrition 24`, +6/16 every 24 frames (the human's hoplites on the AI's ground, after both grants) |
| **The Nubian clause is dead** — `calc_attrition` compares the loop's 733–736 against `BUY_SELL` = 685 (re-read 2026-08-24, `docs/SUPPLY.md` "The radius" found the same in three siblings) | an AI squad on Nubian ground at war with no tech: 0 | `attrition 0` for 98 frames (three refreshes), then 48 within four frames of `tech who=0 allegiance on` |
| Supply cancels the tick and sets a display flag (`Unit::process` `field_0x6c \|= 0x40000`) | damage stops the moment a wagon is within 14 tiles; the flag on each tick frame, cleared at each refresh | the last tick at label 6278, the wagon placed at 6289 two tiles away, `damage` frozen at 9 + 6/16 for 280 frames until combat; **`unit_masks2` reads 262144 = 0x40000** on the 24-grid frames and 0 again on the 32-grid ones |
| Leaving the ground: check 2 on unowned cells | one stale tick at most, then 0 | `attrition 0` at the next refresh after the teleport; `damage` frozen |
| The figure dies at its share of the squad's hits (`docs/COMBAT.md` §7.3) | a hoplite figure at 40 of `myhits 120` | every record vanished with `damage` 39–40 and a `DEATH_OBJS` block; the wagon (one figure, 90) at 90 |

Two things the run showed that the reading had not. **A refresh and a tick
on the same frame take the new period**: the AI's squad, granted a period at
a frame where `(f + o)` was a multiple of both 32 and 48, ticked 6/16 on the
refresh frame itself — `Unit::process` runs `process_attrition` before the
tick test, as the pseudo-code in "The cadence" has it, and the run is the
first time that order was observable. And **object numbers are recycled**:
a dead squad's `o` 6–8 went to the next scout and the next squad, which is
`find_free`'s bands (`docs/AI.md`) seen from the unit side, and is why a
unit's history in a dump has to be cut where its `damage` drops.

Coverage: with run16 in the set the blind list falls from 156 to **99**, and
of attrition and supply only `UnitData::in_supply` (the interface's query),
`ObjectData::in_a_ship`, `num_aircraft_here`, `HeroData::get_radius` and
`LeaderData::get_general_upgrade` have not run — no ship, aircraft, or
hero-general was in the game.

What this does **not** establish: the Statue of Liberty, Mongol, titanium
and Foraging resistances (the rational's other values), the age scaling,
militia's ×4 and their refusal of shelter, siege's ×½, the merchants'
halving, the assassin path, and `ATTRITION_IMPROVED`'s upper steps. Each is
one `cheat tech`/`cheat add` line on the same recipe.

---

## Golden chapter four (run132, run133, 2026-09-23) — the first diff

Item 552 staged the namesake end to end on Great Lakes with the Leader AI
off: a Temple, Religion and Civic 3 on three frames, then Allegiance for
player 0, a hoplite squad and a scout of player 1's on player 0's ground,
and player 1's Supply Wagon 500 frames later (`docs/GOLDEN.md` §8,
`docs/RUNS.md` run132 and run133). Run16 observed the same rules by eye;
this is the first time the harness is compared with the original on
them, field for field.

**Diff-backed now** (`chapter_four_s_word_frame_is_widened_whole`, which
compares `UNITDATA`'s `attrition` and `unit_masks2`'s supply mark since
this item):

- **The period is 48 at one step**, `48 × 256 / 256`, on each figure's
  first refresh after the grant. It is set on the 32-grid, one frame apart
  by `o`, highest `o` first.
- **The tick is 6/16 per figure for a squad of three**, on `(f + o) % 48`,
  carried through `damage_frac` exactly. That is 13 ticks and every
  sixteenth, 617 to 1337.
- **The scout is exempt** (`is_special`, check 8) and **the wagon at war
  is exempt** (check 13): `attrition 0` on every block.

**Two defects in this crate, both fixed.** Neither was a formula: the
arithmetic here was right. Both were wiring.

- **`Leader::calc_attrition` was never called.** `attrition::strength` was
  built and tested against hand-set `PlayerState`s, and nothing wrote
  `strength` from the tech tree, so the period was 0 on every figure. It
  now runs from `Sim::apply_gained`, over the leading run of
  `ATTRITION1..4` (bonuses 49–52: Allegiance, Oath of Fealty, Patriotism,
  Nationalism), with the Russian scaling. The Colosseum and Kremlin arms
  stay false, because this crate holds no owned wonder.
  `calc_anti_attrition` (Foraging, Mongols, Titanium, Liberty) is **still
  unwired**. Every resistance is the base 256 until a capture needs one.
- **The squad size was a stored 1.** `Unit::suffer_attrition` asks
  `curr_uber_size`, which walks the `o_up`/`o_down` chain. `Unit::squad_size`
  is set nowhere past its default, so every figure took 16/16 a tick. The
  damage now counts the chain, as the original does. `fight.rs` still reads
  the stored field, which is a separate question.

**~~Observed and not yet diff-backed~~ Diff-backed since item 567: the
supply veto.** The squad marched about 30 tiles east of where it was
placed, still on player 0's ground, and the wagon trailed after it. Every
tick with the wagon 23 tiles or more away landed; every tick due with it
11–13 tiles away was vetoed, with `0x40000` on the figure's own tick frame.
That brackets the 14-tile radius rather than hitting it. ~~This crate's
squad parts from the original's at 1277, so the veto cannot be compared
yet.~~ Item 567 built the escort the squad takes on 1277 (`docs/ORDERS.md`
§24), and with it every bleed row — `attrition`, `damage_frac`, the hit
points and `sheltered` (`unit_masks2 & 0x40000`) — agrees on every block
from 601 to 1500. The veto is compared, and it agrees. ~~the bracket is not
narrowed, because the wagon's positions still differ.~~ Since item 569 the
wagon's positions agree too (`docs/PATHFINDER.md` §25, `docs/ORDERS.md`
§24.9). The bracket is the capture's own and is not narrowed.
`docs/SUPPLY.md` has the shelter's half.

