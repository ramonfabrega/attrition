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

A blind second reading (`docs/audit/2026-08-20-supply.md`) re-derived the whole
mechanic from the same export. It **doubly confirmed** the registry, the
radius, the distance query, the general's radius, the heal fold and the reload
multipliers — every number in this document survived. What it overturned was
the *predicates* around them: which types are sources, what a field is called,
which units bleed, what gates the heal. Those corrections are landed below and
marked where they changed what an earlier draft said.

**Confidence.** High for the network — the registry, the radius, the query, and
the three consumers are all read end to end, and the radius conversion falls
out of the code arithmetically rather than from an annotation. High for the
reload rule and the heal period. The one thing taken on inference rather than
observation is that `PARMENIO_RADIUS_ADJUST` is loaded as 8.8 fixed point;
the reasoning is given where it is used. What remains open is listed at the end.

**Where the implementation is.** `crates/sim/src/supply.rs`, and the shelter
branch of `Sim::tick` in `crates/sim/src/lib.rs`. Every constant below is
re-read from the user's own install by `cargo run -p rondata -- <install>`,
which fails if any has drifted. The domain field this document shares with
attrition has one definition in the crate, `attrition::Domain`, and `supply`
uses it rather than a second name for the same three values.

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
| `UnitTypeData` | `unit_flags2` @ +0x2b8 | `u32` | Bit `0x40` supply, `0x20` hero, `0x10` special |
| `ObjectTypeData` (the base `UnitTypeData` extends at +0x2b8) | `domain` @ +0x218 | `i32` | 0 land, 1 sea, 2 air |
| | `recharge` @ +0x1f4 | `i32` | Reload delay in frames, before the supply penalty |

Two of those rows changed in the second reading. **+0x218 is `domain`** — the
PDB names it, between `armor` and `los`, and every reader agrees: ships never
bleed, aircraft only take the halved special periods, `in_supply` treats
anything that is not land as supplied, and `process_healing` sends sea and air
down branches of their own. (An earlier draft of this document called it a
"supply category" with values *ordinary / always supplied / never healed*, and
guessed that aircraft were value 1. The branches were read correctly; the
meaning was invented, and the guess was wrong — 1 is sea and 2 is air. It cost
nothing here only because the name never reached the arithmetic.
`docs/ATTRITION.md` had the same field as an "attrition mode" and is corrected
there too.)

**The three `unit_flags2` bits are all set in one place**, `UnitType::init_final_flags`,
from non-strict `TypeData::is` tests at load: `0x40` for the supply set below,
`0x20` for the `GENERAL` line, `0x10` for the `SCOUT` line. `UnitData::is_hero`,
`is_supply` and `is_special` are one-line reads of those bits — so attrition's
exemption for "special" units is, concretely, the scout line.

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

**Which types carry the flag is decided once, at load.**
`UnitType::init_final_flags` sets `unit_flags2 & 0x40` for a type that
non-strictly `is` any of four things:

| `TypeIndex` | The type | Which is |
| --- | --- | --- |
| `0x3f` | `SUPPLYWAGON` | and, through the non-strict test, the national grafts `SUPPLYWAGONAMER` and `SUPPLYWAGONDUTCH` |
| `0x160` | `BASE_GOV_HEROTYPES` | the Despot |
| `0x162` | `THEMONARCH` | the Monarch |
| `0x164` | `THECITIZEN` | the third military patriot, the Comrade — `THECITIZEN` is the PDB's name for the slot, and the display name differs |

So **the three military patriots are supply wagons**, as far as every reader of
that bit is concerned. (An earlier draft of this document said only "wagons",
which is what the section is still mostly written as; the patriots were
missed.) Non-strict `is` is what pulls the grafts in: `ObjectTypeData::is`
consults a precomputed lineage list when the strict flag is 0, so an American
or Dutch wagon answers yes to `is(SUPPLYWAGON)`.

Everything that follows from the one bit follows for a patriot too. He supplies
at the **wagon** radius, 14 tiles and up, not at his general radius of 9 — the
two are separate queries and `find_supply` never asks whose radius it is. He
takes no war-time territorial attrition, for the same reason a wagon does not,
and like a wagon he is never supply-healed: `Unit::process_healing` turns a
supply unit away. He gets the wagon's HP and speed upgrades in
`Unit::update_hits` and `Unit::update_speed`. His *general* radius, which
drives his other auras, is a different number and additionally carries
`MIL_PATRIOT_RADIUS_BONUS` — that test is strict, so it is the patriots' alone.

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
three `SUPPLY_WAGONS_*` steps — `TypeIndex` 763, 764, 765 — and counts the ones
`has_preq` says the player has researched. Nothing else adds to the count, so
the radius is 14, 16, 18 or 20 tiles and nothing in between.

(An earlier draft of this document said one of those steps was additionally
granted outright by a nation bonus, "the same bonus `FRENCH_FREE_SUPPLY` is
named for". **That is dead code.** The loop body reads `if the step is
BUY_SELL, credit has_tribe_bonus(4) instead of has_preq` — and `BUY_SELL` is
`TypeIndex` 685, which the loop, running 763 to 765, can never reach. The
identical arm appears in `get_general_upgrade`, in `get_troops_los_upgrade`
and in `Leader::calc_attrition`, each with its own range: it is a shared loop
macro rather than a Nubian perk, and in the ones read here it never fires. The
enum values were re-dumped from the analysed program rather than assumed.)

`TERRA_COTTA_RANGE` **ships as 0**. The wonder is wired to the radius and
contributes nothing at the shipped balance; it is a hook, not a bonus.

Two things follow that are easy to get wrong:

- **The radius depends only on the player, never on the wagon.** Every one of a
  player's wagons has the same reach. Which wagon supplies a unit is therefore
  not observable through the mechanic, only through the interface.
- ~~**`TRUCK_RADIUS` is not this.**~~ It is a separate constant, also 14, and
  **nothing reads it**: across all 48k decompiled functions the only mention is
  `Constants::init` loading it. It is dead, and a mod that changes it changes
  nothing.

The same upgrade count buys two other things, which is what the tech
description means by "Radius/Speed/HP":

- `Unit::update_hits` adds `SUPPLY_HP_UPGRADE[count]` — `[0, 20, 40, 60]` — to
  a supply unit's hit points.
- `Unit::update_speed` adds `count * speed / 4`, truncating toward zero: a
  quarter of the base per step. `docs/MOVEMENT.md` already has this as one of
  the three `+ n/4` upgrade terms at the end of the cached base, alongside the
  spy and general counts; this is the one that reads `get_supply_upgrade`.

Both test the same `is_supply` bit, so both apply to the military patriots as
well. The HP half is not implemented — hit points are combat's, and combat does
not exist here yet — but it is the answer to an open question this document
used to carry.

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

**`has_general` matches the asking unit itself first.** Before it goes near the
hero list it tests whether the caller is a unit, is a hero, and `is` the type
asked for — and if so returns its own index, with no radius test at all. So
Darius, Kutosov and Chandragupta always count as supplied. That matters more
than it looks: heroes are *not* exempt from attrition (`docs/ATTRITION.md`,
eligibility check 8), so without this the three supplying generals would bleed
inside their own aura. `crates/sim`'s `find_general` reaches the same answer by
scanning the generals list at distance 0 — the only divergence is a general who
is inactive or off the map, and such a unit is not being processed anyway.

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

The Terra Cotta term applies to **any** hero, including the three supplying
generals: `has_wonder` is asked about the owner and nothing is asked about the
type. The patriot terms below it are the strict `is` tests, which is why they
are the patriots' alone. The second reading disputed the first half of that and
the decompile settled it as written here; with `TERRA_COTTA_RANGE` shipping at
0 nothing observable turns on it either way.

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
- **A wagon does not supply itself** — but it does not need to. (An earlier
  draft of this document finished that sentence "wagons in hostile territory
  bleed, which is why a supply line is a thing you can cut". **They do not.**
  `Unit::process_attrition` returns with no period at all for a land supply
  unit whose bleed did not come from the peace or assassin path, so at war a
  wagon standing in enemy territory takes nothing; `docs/ATTRITION.md`
  eligibility check 13 has the exact test. Cutting a supply line means killing
  the wagon, not waiting for the ground to do it.)
- **The peace and assassin bleeds bypass supply outright.** A wagon protects an
  army in a war zone and does nothing for a unit caught over a border in
  peacetime — and that is the one state in which the wagon itself bleeds, since
  the same flag that makes this check give up is what lets the wagon past
  check 13.

Those last two together make the `is_supply` refusal in `process_supply`
**unreachable**. Reaching it needs a supply unit with a period and the
peace/assassin flag clear — and the only period a supply unit can be given is
the peace or assassin one, which sets that flag on the way. The line is written
for a case the rest of the code will not construct.

## Consumer 2: healing

`Unit::process_healing` runs a supply-fed heal on its own period, phased the
same way everything else is — against `frame + o`, the unit's index in its
owner's object list.

**Five gates stand in front of it**, and an earlier draft of this document had
only the last of them. In the order the function tests them, the unit must:

1. be a **captain** — the figure that carries the squad's state; the rest of
   the squad is repaired through it, not alongside it;
2. **not be air** (`domain == 2` returns immediately, so an aircraft never
   heals anywhere in this function);
3. **have damage** to repair, `has_damage(1)`;
4. **not be garrisoned** — `inside_up` negative means on the map; a garrisoned
   unit goes down a separate branch further on;
5. **not be sea** (`domain == 1` takes the ship-and-carrier branch, which
   returns before it ever reaches `find_supply`) and **not itself be a supply
   unit**.

So the supply heal is for damaged, un-garrisoned, land captains. (The draft
said "supply units do not heal this way, and a type whose supply category is 2
does not heal at all". The first half is right; the second was the domain field
misread, and the ships are excluded too — by a different branch, not by this
clause.)

```
period = SUPPLY_HEAL_RATE
if the player has the supply-heal nation bonus:  period += FRENCH_SUPPLY_HEAL_RATE
if the player has Versailles and VERSAILLES_SUPPLY_HEAL_RATE != 0:
    period = VERSAILLES_SUPPLY_HEAL_RATE
    if period_before != 0:  period = (VERSAILLES_SUPPLY_HEAL_RATE + period_before) / 4
if period != 0 and (frame + o) % period == 0 and find_supply(...) >= 0:
    Unit::repair_damage(1, 1, 1)
```

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

**The supply heal is not the only heal in that function**, and this document
covers only the supply one. Between the gates and the fold, `process_healing`
also runs a garrison heal, a ship-and-carrier heal, an Antipater/Wellington
aura heal on `ANTIPATER_HEAL_RATE`, an Iroquois heal in allied territory, a
Conquer-the-World hero heal, the three economic patriots' aura heal, and a
civilian heal after it. They share the `frame + o` phasing and the same
`repair_damage(1, 1, 1)` call and are otherwise unrelated to supply. Whoever
writes the healing document should start there rather than here.

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
if type.domain != land:                           return in supply
if the world cell under the unit is owned by the unit's own player:
                                                  return in supply
return find_supply(...) >= 0
```

**Ships and aircraft are always in supply** for this purpose — the first line
is the domain field again, and it is the same test the sim's `in_supply` makes
with `attrition::Domain`.

So **your own territory supplies you**, for reload purposes, with no wagon
anywhere. Attrition's version has no such clause — it does not need one, since
a unit on its own ground is not taking territorial attrition in the first
place.

And the test is `== the unit's own player`, not "friendly": **allied territory
does not supply you** either. A siege train fighting on an ally's land reloads
at the out-of-supply rate unless it brings its own wagon.

---

## Open questions

- ~~**What reads `TRUCK_RADIUS`.**~~ **Nothing does.** The only mention in the
  whole executable is `Constants::init` loading it.
- ~~**`SUPPLY_HP_UPGRADE`.**~~ `Unit::update_hits` adds
  `SUPPLY_HP_UPGRADE[count]` to a supply unit's hit points, and
  `Unit::update_speed` adds `count * speed / 4` to its speed — both indexed by
  the same `get_supply_upgrade` count as the radius. That is the whole
  "Radius/Speed/HP" the tech description promises.
- ~~**The supply category at `UnitTypeData` +0x218.**~~ It is `ObjectTypeData::domain`,
  named in the PDB: 0 land, 1 sea, 2 air. Which types carry which needs no
  reading — it is what the unit is.
- ~~**Which nation the two bonus ids are.**~~ **4 is Nubia and 10 is France.**
  `LeaderData::has_tribe_bonus(id)` compares the id against a single `int` at
  +0x54 in the player's tribe record — a nation has exactly one power id —
  plus, in Conquer the World, a bitfield of granted racial powers. The id →
  nation mapping is readable straight off `Tribe::parse`, a switch on the id
  that loads that nation's constants for its tooltip: 0 Aztec, 1 Maya, 2 Inca,
  3 Bantu, **4 Nubia**, 5 Greece, 6 Rome, 7 Egypt, 8 Turkey, 9 Spain,
  **10 France**, and on through 0x17. That is exactly the order of the
  `<TRIBES>` block in `rules.xml` — the list `cargo run -p rondata` prints from
  the user's own install, where 4 is `nubians`, 10 is `french` and 18 is
  `iroquois`, and `process_healing` asks `has_tribe_bonus(0x12)` for the
  Iroquois heal. So a nation's power id is its index in that block, for the
  shipped data. The engine still stores it as its own field, so a mod could
  break the correspondence. Bonus 4 is confirmed a second time by
  `Unit::update_hits`, which gates `NUBIAN_HIT_POINTS` on it; bonus 10 by
  `FRENCH_SUPPLY_HEAL_RATE` here and by `Build::activate`, where
  `FRENCH_FREE_SUPPLY` trains a free `SUPPLYWAGON` — that, and not a radius
  step, is what the constant does.
- ~~**Whether `Supplies` is consulted anywhere the reading missed.**~~ It is
  not. `find_supply` has exactly five callers — attrition, healing,
  `in_supply`, and two interface draws — re-checked by grep over all 48k
  decompiled functions.

Still open:

- **Whether the military patriots being supply sources is deliberate.** The bit
  is set for them explicitly and by name, so the code means it; whether the
  design meant a Despot to project a 14-tile supply radius, or whether the flag
  was reused to get him the wagon's exemptions and the radius came along, is
  not something the code can answer. It is a cheap behavioural check
  (`docs/ORACLE.md`): park a unit inside enemy borders beside a Despot with no
  wagon anywhere near, log the frame, and see whether its attrition period is
  ever set.
- ~~**What `unit_flags2` bits `0x4` and `0x8` are.**~~ Closed by
  `docs/COMBAT.md` §2.1: `0x4` is **packs** (the types that stand packed
  between moves — catapult, flaming arrow, machine gun, merchants, fishermen,
  Katyusha) and `0x8` is **caravan** (`CARA`, `MERCHANTFLEET`). Nothing in
  supply reads them.
- **The rest of `Unit::process_healing`.** Six other heals share the function
  and none is derived here.

---

## Second reading (2026-08-20) — landed

A blind second derivation and its adjudication are in
`docs/audit/2026-08-20-supply.md`. The registry, the radius, the distance
query, the hero-general radius, the heal fold and the reload multipliers are
**doubly confirmed** — no number in this document changed. Eight predicates
around them did, all landed above and in `crates/sim/src/supply.rs` on the same
day; each place that changed says so inline. The ones that change what the
mechanic does:

- Wagons do **not** bleed at war (`docs/ATTRITION.md` check 13), so a supply
  line is cut by killing the wagon; the `is_supply` branch in `process_supply`
  is unreachable.
- The three **military patriots are supply sources**, at the wagon radius.
- `+0x218` is `domain`, not a supply category, and the supply heal is
  land-only, captain-only, damaged-only, un-garrisoned.
- The nation-bonus free supply step never existed: `BUY_SELL` is 685 and the
  loop runs 763 to 765.

All five of this document's open questions closed: `TRUCK_RADIUS` is dead,
`SUPPLY_HP_UPGRADE` lives in `update_hits` with the speed third in
`update_speed`, the +0x218 values are the domain, the bonus ids are Nubia and
France, and `find_supply` has no sixth caller. Three smaller ones opened in
their place. One point resolved for the earlier draft rather than against it:
Terra Cotta does apply to a supplying general.

`crates/sim/src/supply.rs` lost its `Category` enum in the same pass — the
reload query now takes `attrition::Domain`, so the crate has one name for the
field. No arithmetic changed, and the tests that covered it still pass
unaltered apart from the type.

---

## Behavioural check (run16, 2026-08-24) — the shelter observed

The attrition run (`docs/ATTRITION.md`, last section; `docs/ORACLE.md`,
"The attrition run") is the first traced game in which `process_supply`,
`Supplies::find_supply`, `SupplyData::get_radius` and
`LeaderData::get_supply_upgrade` ever executed. What it showed of this
document:

- **Consumer 1 works as written.** Two hoplite figures bleeding at period 24
  on the enemy's ground; a wagon of their own placed two tiles away at log
  frame 6289; their last tick is at 6278 and their `damage` does not move
  again for 280 frames, until archers arrive. The period stays 24 and stays
  displayed — the veto is on the damage, not the rate.
- **The display flag is `0x40000` in the unit's second mask word**, and the
  log prints it: `unit_masks2` reads 262144 on exactly the frames where the
  tick was due (the 24-grid) and 0 on the frames where the period was
  refreshed (the 32-grid), which is `Unit::process` setting it on a
  sheltered tick and clearing it on entry to the upkeep block.
- **The wagon at war took nothing** (check 13), standing on the same enemy
  ground beside the bleeding squad: `attrition 0` throughout.
- **The wagon at peace bled at 8**, one whole point a tick, for 90 points —
  the peace flag defeats its own exemption, as the second reading found.
- **The wagon's own `supply` field is its slot** in the player's list —
  `0` for the first wagon, `-1` on every other unit — the cached
  `init_supply` return this document describes; it is *not* a "sheltered"
  mark on the units it protects, which show it in `unit_masks2` instead.
- The trace's first-entry frames say what the reading could not:
  `init_supply` ran the frame the wagon was placed (1127), `close_supply`
  the frame it died (1855), `find_supply` first at 4877 (a squad with no
  wagon on its list — the loop over `supply_mark` is empty and `get_radius`
  is not reached) and `get_radius` first at 6300, the first sheltered tick.

What it does not establish: the radius's edge (the wagon was two tiles
away, not fourteen), the upgrade steps, the patriots as sources, the
hero-generals, militia's refusal, and the healing and reload consumers.
