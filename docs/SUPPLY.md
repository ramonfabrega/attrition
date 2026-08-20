# Supply

The counter to attrition, and two other things nobody mentions in the same
breath: supply also **heals**, and being out of it **slows artillery reload**.
One radius query answers all three.

`docs/ATTRITION.md` said supply "cancels" attrition and left `Supplies::find_supply`
as a name. This is that name opened up.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied, plus one short disassembly read where the
decompiler dropped a register argument. Nothing here is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High for the network — the registry, the radius, the query, and
the three consumers are all read end to end, and the radius conversion falls
out of the code arithmetically rather than from an annotation. High for the
reload rule and the heal period. The one thing taken on inference rather than
observation is that `PARMENIO_RADIUS_ADJUST` is loaded as 8.8 fixed point;
the reasoning is given where it is used. What remains open is listed at the end.

**Where the implementation is.** `crates/sim/src/supply.rs`, and the shelter
branch of `Sim::tick` in `crates/sim/src/lib.rs`. Every constant below is
re-read from the user's own install by `cargo run -p rondata -- <install>`,
which fails if any has drifted.

---

## The state it touches

| Where | Field | Type | Meaning |
| --- | --- | --- | --- |
| `Supply` / `SupplyData` (16 B, `malloc(0x14)` with a vbase word) | *(self)* @ +0 | `i16` | The record's own slot index |
| | *(o)* @ +2 | `i16` | The supplying unit's index in its owner's object list |
| | *(flags)* @ +4 | `u8` | Bit 0: slot in use |
| | `who` @ +5 | `u8` | Owning player |
| `Supplies` (global) | `lists` | `PtrArray<Supply>[8]`, stride 28 B | One slot list per player |
| `LeaderData` | *(supply count)* @ +0x434 | `i32` | High-water mark of used slots, and the scan bound |
| | *(hero counts)* @ +0x56fe | `i16[]` | Indexed by `TypeIndex`; how many of that hero the player has |
| `UnitData` | *(supply slot)* @ +0x84 | `i16` | Which slot this unit registered as, `-1` if none |
| `UnitTypeData` | *(kind flags)* @ +0x2b8 | `u32` | Bit `0x40` supply, `0x20` hero, `0x10` special |
| | *(supply category)* @ +0x218 | `i32` | 0 ordinary, 1 always supplied, 2 never healed |
| | *(base recharge)* @ +0x1f4 | `i32` | Reload delay in frames, before the supply penalty |

The eight-element `Supplies` array is the hardcoded eight players `CLAUDE.md`
names as the reason not to copy the engine. `crates/sim` keeps one list per
player and does not assume a count.

---

## The network

### Who is a supply source

Exactly one rule: **a unit whose type carries the supply flag**. Not cities,
not forts, not buildings — `Supplies::init_supply` has two callers and both are
on `Unit`.

- `Unit::init` registers the unit if `UnitData::is_supply`, and caches the
  returned slot in the unit.
- `Unit::set_type` closes the old registration and opens a new one when a type
  change crosses the flag either way.
- `Unit::close` closes it.

`init_supply` reuses the first free slot below the high-water mark, extends the
list if there is spare capacity, and allocates a new record otherwise.
`close_supply` clears the in-use bit, blanks the unit index and the player, and
then walks the top of the list down past any trailing free slots. So slot
indices are stable while a wagon lives and are recycled afterwards.

### The radius

`SupplyData::get_radius`, in **tiles**:

```
radius = SUPPLY_RADIUS
       + SUPPLY_RADIUS_UPGRADE * (supply upgrade steps held)
       + (owner has the Terra Cotta Army ? TERRA_COTTA_RANGE : 0)
```

`SUPPLY_RADIUS` is 14 and `SUPPLY_RADIUS_UPGRADE` is 2, so a plain wagon
reaches 14 tiles and a fully upgraded one 20.

The upgrade count comes from `LeaderData::get_supply_upgrade`, which walks the
three `SUPPLY_WAGONS_*` steps and counts the ones the player has researched.
One of those steps is additionally granted outright by a nation bonus — the
same bonus `FRENCH_FREE_SUPPLY` is named for — so a player with it counts that
step whether or not they researched it.

`TERRA_COTTA_RANGE` **ships as 0**. The wonder is wired to the radius and
contributes nothing at the shipped balance; it is a hook, not a bonus.

Two things follow that are easy to get wrong:

- **The radius depends only on the player, never on the wagon.** Every one of a
  player's wagons has the same reach. Which wagon supplies a unit is therefore
  not observable through the mechanic, only through the interface.
- **`TRUCK_RADIUS` is not this.** It is a separate constant, also 14, and
  `SupplyData::get_radius` does not branch on unit type. What reads it is not
  established.

### The query

`Supplies::find_supply(x, y, player)` returns a slot index, or `-1`:

```
for i in 0 .. leader.supply_count:
    s = supplies.lists[player][i]
    if !(s.flags & 1):                 continue    # free slot
    u = units[player][s.o]
    if !u.is_active() || !u.is_on_map(): continue
    if vector_dist(x - u.x, y - u.y) <= get_radius(s) * 192: return i
return -1
```

Four things in that are load-bearing.

**It is your own list only.** `find_supply` is called with the *victim's*
player index and indexes that player's list. An ally's supply wagon does
nothing for your units — a fact the interface never explains and which changes
how a team plays.

**The wagon must be active and on the map.** Those are the two virtual calls at
`Unit::vftable` `+0x8` and `+0xbc` — `SubObjectData::is_active` and
`UnitData::is_on_map`. A wagon inside a transport, garrisoned, or mid-death
supplies nothing.

**`× 192` is the tile-to-position-unit conversion.** The radius is in tiles and
the positions are in the original's fine coordinate unit; 192 position units to
the tile is exactly the scale `docs/FORMATS.md` derives from `div_3_table`.
Arriving at the same 192 from a completely different function is the check on
that derivation. The compare is `<=`, so the boundary tile is inside.

**`vector_dist` is the same integer hypotenuse the territory pass uses.** Not a
similar one — literally the same function, `hi + lo² / (2·hi)` with the
`lo < 60000` overflow branch, which `docs/ATTRITION.md` derives from the
territory side. Territory inlines it; supply calls it. So supply radii are
octagonal in exactly the way borders are, and the two subsystems cannot drift
apart. `crates/sim` therefore has one copy, in `world`, and both call it.

The decompiler drops the two arguments here, because they are passed in
registers; the deltas are recomputed per candidate against each wagon's own
XOR-masked coordinates, which is visible in the disassembly and is the only
claim in this document read at that level.

**First match wins, not nearest.** The scan returns on the first wagon in range
in slot order. Since every wagon of a player shares one radius, this changes
only the returned index, and the only consumers of the index are the interface.

### Generals supply too

If no wagon is in range, `Unit::process_supply` tries three specific heroes, in
this order: **Darius**, **Kutosov**, **Chandragupta**. Each is gated first on a
cheap per-player counter — `LeaderData` +0x56fe indexed by type — and then on
`ObjectData::has_general`, which searches for a hero of that type whose own
radius covers the unit.

A general's radius is not the supply radius. `HeroData::get_radius`:

```
r = GENERAL_RADIUS * (general upgrade steps + 3) / 2
if Parmenio:      r = r * PARMENIO_RADIUS_ADJUST / 256      # 3/2, truncating toward zero
if Wellington:    r = r * WELLINGTON_RADIUS / 100           # 200
if Kutosov:       r = r * KUTOSOV_RADIUS / 100              # 300
if Terra Cotta:   r = r + TERRA_COTTA_RANGE                 # 0
if military patriot: r = r + MIL_PATRIOT_RADIUS_BONUS       # 3
if economic patriot: r = r + ECON_PATRIOT_RADIUS_BONUS      # 1
```

At `GENERAL_RADIUS` = 6 and no upgrades that is 9 tiles, against a wagon's 14 —
and Kutosov, who is both a supplying general and the one with the ×3 radius,
covers 27.

`PARMENIO_RADIUS_ADJUST` is written `3/2` in `rules.xml` and applied as a
multiply followed by a shift right by 8 with a truncate-toward-zero bias. A
`3/2` scale through a `>> 8` means the loader stores it as 8.8 fixed point,
value 384. That is the one number here inferred rather than observed, and two
independent facts agree on it.

This radius is the general's, not supply's — it also drives their other
auras. It lives in `supply` only because supply is the first mechanic to need
it, and it moves the day a generals document exists.

---

## Consumer 1: attrition

`Unit::process_supply` is called from `Unit::process` on every frame a bleed is
due, and gets first refusal on the damage. `docs/ATTRITION.md` covers the
cadence; the function itself is short:

```
if unit is flagged as a peace or assassin bleed:  return not supplied
if unit is itself a supply unit:                  return not supplied
if unit is militia:                               return not supplied
if find_supply(unit.pos, unit.owner) >= 0:        return supplied
if any of Darius, Kutosov, Chandragupta is in range: return supplied
return not supplied
```

Supplied means the tick is **dropped entirely** — not reduced. The period is
still computed, still stored, and still shown. That is what lets an army
campaign abroad at all, and it is implemented as a veto on the damage rather
than as anything to do with the rate.

The three refusals each carry weight:

- **Militia are never sheltered**, the other half of why they take four times
  the rate. They defend your own ground; the game declines to let them campaign
  behind a wagon.
- **A wagon does not supply itself.** Wagons in hostile territory bleed, which
  is why a supply line is a thing you can cut.
- **The peace and assassin bleeds bypass supply outright.** A wagon protects an
  army in a war zone and does nothing for a unit caught over a border in
  peacetime.

## Consumer 2: healing

`Unit::process_healing` runs a supply-fed heal on its own period, phased the
same way everything else is — against `frame + o`, the unit's index in its
owner's object list.

```
period = SUPPLY_HEAL_RATE
if the player has the supply-heal nation bonus:  period += FRENCH_SUPPLY_HEAL_RATE
if the player has Versailles and VERSAILLES_SUPPLY_HEAL_RATE != 0:
    period = VERSAILLES_SUPPLY_HEAL_RATE
    if period_before != 0:  period = (VERSAILLES_SUPPLY_HEAL_RATE + period_before) / 4
if period != 0 and (frame + o) % period == 0 and find_supply(...) >= 0:
    Unit::repair_damage(1, 1, 1)
```

Supply units do not heal this way, and a type whose supply category is 2 does
not heal at all.

**`SUPPLY_HEAL_RATE` ships as 0**, and its own annotation says `0 means don't
heal at all`. So in the shipped game supply heals **nobody** by default. It
heals only for a player with the nation bonus (period 20 frames), or with
Versailles (20), or with both — where the fold gives `(20 + 20) / 4` = **10
frames**, the fastest supply healing in the game.

The fold is worth staring at, because it is written as though it were a rate
and behaves as a period. Adding `FRENCH_SUPPLY_HEAL_RATE` to a nonzero base
would make healing *slower*, not faster. It is only correct because the base
ships at 0 — a latent bug that the shipped data steps around, of the same
family as the `-1` index `docs/ATTRITION.md` records in the neutral-attrition
path.

## Consumer 3: reload

`UnitData::recharge` is where being out of supply hurts a siege line:

```
delay = type.base_recharge
if the type is not a siege type:                        return delay
if !(ARTILLERY_UNDER_ATTACK_FIRES_SLOWLY and unit is under attack)
   and UnitData::in_supply():                           return delay
if the unit is of the Bombard line:                     return delay * 2
return delay * 3 / 2
```

"Is a siege type" is a virtual on the type record, slot `+0x10c` — and it is
the **same slot** that `UnitData::get_attrition` tests to apply
`SIEGE_ATTRITION`. So exactly the set of units that take half attrition is the
set whose reload cares about supply. That cross-check is what makes the reading
safe: two unrelated functions, one predicate.

`ARTILLERY_UNDER_ATTACK_FIRES_SLOWLY` ships as 1, so a siege unit that is
currently under attack is treated as out of supply no matter where it stands —
which is precisely what the constant's own annotation says it does.

**The `3/2` and `2` are literals in the code.** `SIEGE_OUT_OF_SUPPLY_RELOAD`
and `ARTILLERY_OUT_OF_SUPPLY_RELOAD` exist in `rules.xml`, are loaded into the
constants table, and are not read here; the annotations `"3/2 normal delay"`
and `"2/1 normal delay"` describe literals rather than feeding them. Editing
them in a mod would change nothing. `crates/sim` writes the literals for the
same reason, and does not carry the two constants in `Tuning` — a tuning entry
that cannot change the result would be a lie about what the simulation reads.

### `in_supply` is a different question from `process_supply`

This is the subtlety in the whole mechanic. `UnitData::in_supply` is not the
attrition test:

```
if type.supply_category != 0:                     return in supply
if the world cell under the unit is owned by the unit's own player:
                                                  return in supply
return find_supply(...) >= 0
```

So **your own territory supplies you**, for reload purposes, with no wagon
anywhere. Attrition's version has no such clause — it does not need one, since
a unit on its own ground is not taking territorial attrition in the first
place.

And the test is `== the unit's own player`, not "friendly": **allied territory
does not supply you** either. A siege train fighting on an ally's land reloads
at the out-of-supply rate unless it brings its own wagon.

---

## Open questions

- **What reads `TRUCK_RADIUS`.** It is 14, sits beside `SUPPLY_RADIUS`, and
  `SupplyData::get_radius` does not branch on unit type.
- **`SUPPLY_HP_UPGRADE`** = `[0, 20, 40, 60]` is indexed by the same upgrade
  count as the radius and is not read by anything in this document. The tech
  descriptions say supply wagons gain "Radius/Speed/HP", so the HP third is
  presumably applied where unit hit points are computed, and the speed third
  somewhere else again.
- **The supply category at `UnitTypeData` +0x218.** Three values are
  distinguished — 0 ordinary, 1 always in supply, 2 never healed — and which
  types carry 1 or 2 is unread. Aircraft are the obvious guess for 1 and
  buildings-in-unit-form for 2, but that is a guess.
- **Which nation the two bonus ids are.** `has_tribe_bonus(4)` grants a free
  supply upgrade step and `has_tribe_bonus(10)` grants the supply heal; both
  are named after the French in the constants they gate. The mapping from a
  bonus id to a nation is not in `game/tribes/*.xml`, which carry only leader
  names, city names and art styles, and not in `rules.xml`'s `<TRIBES>` block,
  which carries only a filename and a key. Where the per-nation power id is
  loaded from is unlocated.
- **Whether `Supplies` is consulted anywhere the reading missed.**
  `find_supply` has five callers and all five are accounted for: attrition,
  healing, `in_supply`, and two interface draws.

---

## Second reading (2026-08-20) — corrections owed

Blind second derivation and adjudication: `docs/audit/2026-08-20-supply.md`.
The radius, the distance query, the hero-general radius, the heal fold, the
reload multipliers and the registry are **doubly confirmed**. Eight
behaviour-relevant points went against this document; until they land here and
in `crates/sim/src/supply.rs` (doc comments and the `Category` name — no
arithmetic changes), this document is wrong on:

- ~~Whether wagons bleed: a land supply unit takes no attrition unless in the
  peace-violation/assassin state (`process_attrition`), and heroes are *not*
  exempt — both corrections belong to `docs/ATTRITION.md` and are listed there.~~
  Landed in `docs/ATTRITION.md` (eligibility checks 8 and 13) and
  `crates/sim/src/attrition.rs` on 2026-08-20.
- The nation-bonus "free supply step" is dead code (`BUY_SELL = 0x2ad` is
  outside the `0x2fb..0x2fd` loop); bonus 4 is Nubia, 10 is France, and
  `FRENCH_FREE_SUPPLY` spawns a wagon.
- The military patriots (Despot, Monarch, Comrade) are supply sources
  (`UnitType::init_final_flags`).
- `+0x218` is the unit's **domain** (land/sea/air), not a supply category; sea
  units never reach the supply heal.
- The heal has gates this document omits: captain present, damaged, not
  garrisoned, land, not itself a supply unit.

Closed open questions: `TRUCK_RADIUS` is unread; `SUPPLY_HP_UPGRADE` is
applied in `update_hits`, and the wagon's `+level/4` speed in `update_speed`.
Resolved for this document: Terra Cotta does apply to a supplying general.
