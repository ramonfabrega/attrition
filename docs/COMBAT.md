# Combat

How one object hurts another: what an attack is worth against a given target,
how often it lands, where a shot falls, how damage is carried on a figure, and
when a figure dies. Attrition (`docs/ATTRITION.md`) was the only thing in the
simulation that could kill before this document; now the units can.

In scope: the combat columns of the unit and building tables and the derived
stats the engine computes from them; the **combat table** — the 493×493
percentage matrix that decides what every type is worth against every other
type — and how the engine builds it at load; the damage formula, end to end;
how damage is delivered in sixteenths and accumulated on a figure; the firing
cadence; projectiles — accuracy, scatter, flight time, the hit test at landing,
splash; buildings that shoot; squads and what the death of one figure does to
the rest; and the automatic choice of target.

Not in it: what makes a unit *want* to fight — orders, stances and the AI's
posture are surveyed only where the firing path reads them; aircraft, missiles
and nukes, which share the damage formula but have their own flight and a
separate dispatch (`Nuke::do_damage`, `AirAttackGroundOrder`); capture and
plunder of cities, which damage triggers and which belong to cities; spells
and abilities; the garrison and ejection paths, except for the one number
they feed the tower (arrows).

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied, through the export under
`~/ghidra-projects/decomp` (`tools/ghidra/`): `ObjectData::get_damage`,
`Object::do_damage`, `Object::take_damage`, `Unit::fight`, `Unit::do_attack`,
`Object::fire_ammo`, `Ammo::init`/`inc_time`/`do_damage`/`hit_target`/
`check_hit`, `Build::process`/`do_attack`, `Balance::fill_tables`/
`compute_modifier`/`type_damage`, `UnitType::init`, `Constants::init`, and the
stat accessors, each read end to end. Two vtables the export did not contain —
`UnitType`'s and `BuildType`'s `vftable{for Type}` — were read out of the PE
bytes at the addresses `rise_z.map` gives them, which is how every
`(**(type + 0x10c))()` below has a name. Where a virtual's slot is quoted it
is because that is how the decompile reads, and the name follows. Nothing is
transcribed; see `docs/DECISIONS.md` entry 7.

The `obj_masks` bits are named from the legend the shipped `balance.xml`
carries in its own row names (`Flag_A_OBJMASK_ARMORED` … `Flag_6_OBJMASK_ANTI_AIR`),
so every flag test below is a word rather than a hex constant. That file is
the user's install and is not reproduced here; the legend is one line of it
per letter and is listed in §3.

**Confidence.** High for the data layout and loaders, the damage formula's
arithmetic and order, the sixteenths delivery, the per-figure hit share, the
recharge cadence and its decrement, the melee and projectile paths,
`Ammo::init`'s accuracy and scatter, the landing-time hit test, buildings'
reload and arrows, the combat table's *structure* and the RNG. High for the
hardcoded `type_damage` rules as a list (§5), medium for their completeness:
there are around a hundred multiplicative steps and each was read once.
The flank direction (§6 step 19) was medium and is now **high**: the
arithmetic was always certain and the convention it rested on — that a unit's
`angle` is its facing — is measured in a logged run (§14.1), so level 1 is the
rear. Medium for three virtual calls whose argument the decompiler dropped
(noted inline).
Medium for target selection (§12), read by a second reader and adjudicated.
Everything open is listed at the end.

A blind second reading (`docs/audit/2026-08-20-combat.md`, two readers — one
over the damage pipeline, one over the cadence and projectiles) doubly
confirmed the formula step by step, the scaling, the per-figure share, the
RNG, the recharge rule, `attack_dist`, the hit tests and the building reload,
and **overturned five things in the first draft**, each marked "second
reading" where it is landed below: the accuracy formula (`− attenuate × (d /
192)`, not `+ (d / 96) × attenuate`), when the one-in-five retarget draw is
taken, that siege fire at a unit is ground fire, the splash search square,
and the anti-air building gate. It also added the `total_time` floor and the
moving-target lead the first reading had missed.

**Where the implementation is.** `crates/sim/src/combat.rs` (the RNG, the
profile, the table, the damage formula, the scaling and the take, recharge,
accuracy, scatter, flight time, the hit tests, splash, arrows, `attack_dist`,
`is_in_range`); `crates/sim/src/fight.rs` (the `Sim` glue: the attack step,
orders, the target search and ranking, buildings, ammo in flight, death);
`crates/sim/src/balance.rs` (the combat table's two halves and their
composition); and `crates/rondata/src/balance.rs` (the shipped `balance.xml`,
the 399 category names, the structural checks). What the implementation
leaves out is said again at each point it applies: the nation, wonder and
patriot layer arrives as `combat::Modifiers` (the choice `docs/COSTS.md`
made); terrain, river, height and the unnamed `unit_masks` bits are `Side`
inputs; aircraft and missiles are not modelled; the move-to-attack path is a
straight line; a squad is figures sharing a captain index; and ~~the
regeneration of the combat table from an install has its XML half and its
flag half but not yet the age and named-lineage rules, which need a tree
loader `rondata` does not have — so `rondata` prints the masks-only entry
and the `RULES=1` log remains the oracle~~ — as of 2026-08-20 the
regeneration is whole for the unit block: `rondata::load` builds every
unit's `Kind` with its age and named lineages from the tree, and
`rondata --types <dump>` checks the 364 `Kind`s and the 364 × 364 table it
builds against the program's own start-of-game dump, **cell for cell equal**
(§15.2) — and, the same day, the building half too: `combat::Table` is
two-family (`TypeRef::Unit`/`Build`), `rondata` builds a `Kind` per building,
and all four quadrants of the 493 × 493 are equal (§15.3). The fight path
now applies the table to every pair, buildings included.

---

## 1. The shape of it

Every attack in Rise of Nations is one call to `Object::do_damage(target,
angle, ammo, count)` on the attacker. It computes the damage with
`ObjectData::get_damage`, scales it by `count` (an 8.8 fraction — a full hit
is `0x100`, a splash fringe less), divides a unit's shot by its squad size and
a building's by its ammo per volley, splits the result into **whole hits and
sixteenths**, and hands both to the target's `take_damage`, which carries the
sixteenths in the same `damage_frac` byte attrition uses, compares the
figure's accumulated damage against *its share* of the squad's hit points,
and kills it when the share is reached.

What differs between unit kinds is only **how `do_damage` is reached**:

- a **melee** unit (`max_range == 0`) calls it directly from `Unit::fight`,
  once per figure, the frame its recharge counter reaches zero;
- a **ranged** unit fires one `Ammo` per figure (`Object::fire_ammo`), and
  each ammo calls it when it lands, `total_time` frames later, if the landing
  point is still on its target — otherwise on whatever it hit instead;
- a **building** fires `ammo_per_att` ammo per volley, every
  `recharge / arrows` frames, at the target its own search picked.

The recharge counter is a byte on the unit (`UnitData::recharging`) set to
`UnitData::recharge()` after every attack and decremented once at the top of
every `Unit::process`. Attack rate is therefore **one attack every `RECHARGE`
frames exactly**, with no phase and no randomness; the only randomness in
combat is where a projectile lands.

---

## 2. The data

### 2.1 The combat columns

`UnitType::init` reads these out of each unit's record (`unitrules.xml`;
`BuildType::init` reads the same `ObjectTypeData` columns out of
`buildingrules.xml`). Offsets are on `ObjectTypeData` unless marked `U`
(`UnitTypeData`) or `B` (`BuildTypeData`).

| column | field | how it is loaded |
| --- | --- | --- |
| `ATTACK` | `+0x1e8 attack` | **×10** — `attack = get_text_num(...) * 10`. Every attack figure in the engine is in tenths; the formula divides by ten at the end (§6 step 20). Bonuses that say "+N attack" are added as `N * 10`. |
| `TO_HIT` | `+0x1ec to_hit` | plain; default −1 |
| `ATTENUATE` | `+0x1f0 attenuate` | **absolute value** of the number read — the file writes it negative (`-1`), the engine stores `1`, and the accuracy formula subtracts it (§9.1) |
| `RANGE` | `+0x1f8 min_range`, `+0x1fc max_range` | one string, `min-max`; the part before the `-` is `min_range`, the part after is `max_range`. No `-`: `max_range = min_range`. **A cavalry-archer type (`unit_flags & 0x400`, letter `k`) has its `max_range` moved to `U +0x2d0 second_max_range` and `max_range` set to 0**, so it is a melee unit whose ranged mode is switched on by stance (§8.5). |
| `SPLASH_AREA` | `+0x200 splash_area` | plain, in tiles |
| `SPLASH_PERCENT` | `+0x204 splash_percent` | plain |
| `AMMO_PER_ATT` | `+0x208 ammo_per_att` | plain |
| `RECHARGE` | `+0x1f4 recharge` | plain, frames |
| `ARMOR` | `+0x214 armor` | plain |
| `HITS` | `+0x210 hits` | plain |
| `PROJ_SPEED` | `+0x20c proj_speed` | plain; a ranged type that has none gets `200` and a load-time error |
| `OBJ_MASK` | `+0x1e4 obj_masks` | a string of letters, upper-cased: each letter `A`–`Z` sets bit `c − 'A'` (0–25), each digit `1`–`9` sets bit `c − 0x17` (26 up). See §3 for the names. |
| `FLAGS` | `U +0x2b4 unit_flags` | the same encoding, lower-cased, `a`–`z` → bits 0–25 |
| `TARGET_SIZE` | `U +0x300 target_size` | **× `UNIT_BLOCK_RADIUS`** (the shipped `1 UCoord`, see `docs/MOVEMENT.md`) |
| `UBER_SIZE` | `U +0x308 uber_size` | plain — figures per squad |
| — | `U +0x2cc fire_proj` | **derived**: `1` if `attack != 0` (or the type has `unit_flags & 0x200000`), else `0`. This is what `Unit::fight` tests to decide a melee unit actually strikes — so a melee type with zero attack swings and does nothing. |
| `BASE_ARROWS` | `B +0x2cc base_arrows` | plain |
| `MOST_SHOTS` | `B +0x2c4 most_shots` | plain |

`UnitType::init_final_flags` derives the `unit_flags2` bits this mechanic
reads, and this is the first document to name them all: `0x1` **machine-gun
line** (`is(MACHINEGUN)` or `is(FLAMETHROWER)`); `0x4` **packs** (Catapult,
Flaming Arrow, Merchant, Dutch Merchant, Fur Trapper, Machine Gun,
Fishermen, Katyusha — every type that stands packed between moves); `0x8`
caravan (`CARA`, `MERCHANTFLEET`); `0x10` special (Scout); `0x20` hero
(General); `0x40` supply (Supply Wagon and the three patriots `THEDESPOT`,
`THEMONARCH`, `THECITIZEN`). `docs/SUPPLY.md` left `0x4` and `0x8` open;
they are closed here.

### 2.2 The per-object state

`ObjectData` (both units and buildings): `+0x20 myhits` (the squad's hit
points, copied onto every figure by `update_hits`); `+0x24 damage` (whole
hits taken); `+0x3b damage_frac` (sixteenths taken, 0–15); `+0x32
hold_frames`; `+0x3d targeted`.

`UnitData`: `+0x4c damage_frame`, `+0xa4 damage_o`, `+0xa9 damage_who` (the
overkill record, §6 step 21); `+0x50 angle` (facing; `docs/MOVEMENT.md`);
`+0x5c trench_angle`; `+0x68 unit_masks` — bits this mechanic reads: `0x1`
**decoy** (set on a spy's decoy; read in `Unit::process` as "age out after
`decoy_time`"), `0x10`, `0x80000` packed, `0x400000`, `0x1000000`
entrenched, `0x4000` plunder-on-kill (Aztec); `+0x6c unit_masks2` — `0x1`
under attack, `0x1000` Antipater-entrenched; `+0x9c myarmor` (the cached
`update_armor`); `+0xae recharging`; `+0x8e o_up` (high bit: captain), `+0x90
o_down` (next figure of the squad, −1 at the end); `+0xb5 guy_mark` (figures
on this `UnitData`, normally 1); `+0xe4 guys`.

`BuildData`: `+0x7a recharging` (a short), `+0x7c attack_ox`, `+0x81
attack_whom`, `+0x60 build_masks` (`0x4`: has an explicit attack order;
`0x8000`: jammed this frame), `WallData +0x54 construct_hits`.

`AmmoData` (one projectile in flight): `+0x6 accuracy`, `+0xc/0x10/0x14
sx,sy,sz` (launch), `+0x18/0x1c/0x20 ex,ey,ez` (landing), `+0x24 cur_time`,
`+0x28 total_time`, `+0x2c angle`, `+0x30 splash_area`, `+0x3c who`/`+0x40 o`
(shooter), `+0x44 num_guys`, `+0x48 whom`/`+0x4c ox` (target), `+0x4 flags`
(`0x2` live, `0x4` rolling, `0x8` missed-and-rolling, `0x10` cosmetic — no
damage).

### 2.3 The constants

All in `Constants`, loaded by `Constants::init` from `rules.xml`; the scale
is a fact about the loader line, per `docs/DECISIONS.md` entry 14.

| constant | field | loaded | shipped |
| --- | --- | --- | --- |
| `FLANK_BONUS` | `flank_bonus` | plain, percent | 50 |
| `CAVALRY_FLANK_BONUS` | `cavalry_flank_bonus` | plain, percent of base — but consumed with `>> 8` (step 19), so the shipped `40` is `40/256` of the base, not 40 % | 40 |
| `VEHICLE_FLANK_BONUS` | `vehicle_flank_bonus` | plain, consumed `>> 8` | 33 |
| `ROCKY_MODIFIER` | `rocky_modifier` | `get_fraction(…, 0x100)` — 8.8 | `2/3` → 170 |
| `OVERKILL_FRAMES` | `overkill_frames` | plain, frames | 30 |
| `OVERKILL_DAMAGE` | `overkill_damage` | 8.8 | `1/3` → 85 |
| `ENTRENCHMENT_MODIFIER` | `entrenchment_modifier` | 8.8 | `2/3` → 170 |
| `RIVER_MODIFIER` | `river_modifier` | 8.8 | `2/1` → 512 |
| `RECAPTURE_CITY_MODIFIER` | `recapture_city_modifier` | 8.8 | `2/1` → 512 |
| `HEIGHT_INCREMENT` | `height_increment` | plain, z units | 200 |
| `HEIGHT_BONUS` | `height_bonus` | plain, percent per increment | 10 |
| `ONE_AGE_DOWN` … `FIVE_AGES_DOWN` | `one_age_down` … `five_ages_down` | plain, percent | 15, 20, 50, 60, 70 |
| `RANGE_INACCURACY` | `range_inaccuracy` | 8.8 | `1/100` → 2; **not read by the firing path** (see §9.1) |
| `TARGET_RADIUS` | `target_radius` | `get_fraction(…, 0xc0)` — position units | `1/2` tile → 96 |
| `UNIT_MOVE_SPEED` | `unit_move_speed` | see `docs/MOVEMENT.md` | `1/192` tile |
| `CITY_CAPTURE_RADIUS` | `city_capture_radius` | plain, tiles | 10 |
| `UNIT_DEFENSIVE_RESPOND_RANGE` and the other `*_RESPOND_RANGE` | plain, tiles | 4, 12, 8, 12, 32, 10, 12, 8 |
| `JAM_UNIT_RADAR_PROB` | `jam_unit_radar_prob` | plain, percent | 50 |
| `SIEGE_OUT_OF_SUPPLY_RELOAD`, `ARTILLERY_OUT_OF_SUPPLY_RELOAD` | 8.8 | `3/2`, `2/1` — **not read by `recharge()`**, which hardcodes `×3/2` and `×2` (§8.3) |
| `ARTILLERY_UNDER_ATTACK_FIRES_SLOWLY` | plain flag | 1 |
| `SUPER_IMMUNE` | plain flag | 0 |
| `RED_FORT_AIR_DEFENSE`, `JAPANESE_DAMAGE`, `RUSSIAN_COSSACK_DAMAGE`, `DUTCH_ATTACK_BONUS`, `CARAVAN_ATTACK_BONUS` (×10), `TERRA_COTTA_ATTACK`, `WELLINGTON_SIEGE_ATTACK`, `GENERAL_RALLY_ARMOR`, `ANTIPATER_ENTRENCH_BONUS` (8.8), the general/patriot attack and armour bonuses | plain unless noted | nation and wonder layers |

---

## 3. The `obj_masks` legend

Bit `n` is letter `'A' + n`; the names are the row names of the shipped
`balance.xml`, which is also how the engine's own designers referred to them.

```
0x1        A  ARMORED        0x2        B  BOMBARD       0x4        C  CIVILIAN
0x8        D  MUSKET_INF     0x10       E  ELEPHANT      0x20       F  FOOT
0x40       G  GUN            0x80       H  HEAVY_INF     0x100      I  MODERN_INF
0x200      J  CARRY_AIR      0x400      K  FOOT_ARCHER   0x800      L  LARGE
0x1000     M  MOUNTED        0x2000     N  NAVAL         0x4000     O  HORSE_ARCHER
0x8000     P  SPARSE         0x10000    Q  LIGHT_INF     0x20000    R  ARCHERY
0x40000    S  SIEGE          0x80000    T  WAR_MACHINE   0x100000   U  ARMORPIERCE
0x200000   V  VEHICLE        0x400000   W  MELEE         0x800000   X  EXPLOSIVE
0x1000000  Y  HEAVY_CAV      0x2000000  Z  DETECT        0x4000000  1  UNUSED
0x8000000  2  MISSILE        0x10000000 3  AIR           0x20000000 4  LIGHT_CAV
0x40000000 5  PIKE           0x80000000 6  ANTI_AIR
```

Earlier documents quoted these bits raw (`obj_masks & 0x40000` in
`docs/ATTRITION.md`, `0x4` in `docs/COSTS.md`, `0x20`/`0x1000` in
`docs/MOVEMENT.md`); they read as SIEGE, CIVILIAN, FOOT and MOUNTED.

---

## 4. The stats at runtime

Every stat the formula reads is a virtual on the object, and every one starts
from the type's column and adds the nation, wonder and patriot layers. The
additions are listed because they are the rules; the simulation takes them as
`combat::Modifiers` until those layers exist.

### 4.1 `attack()`

`ObjectData::attack` — the base: `type.attack` (×10). A **caravan** whose
owner has the `CARAVAN_ATTACK_BONUS` tech has its attack **replaced** by
`constants.caravan_attack_bonus` (itself loaded ×10). Then the Dutch: a
unit that is a merchant, Dutch merchant, fur trapper, caravan or supply
unit, and not a patriot, gets `+ ages × dutch_attack_bonus × 10`, where `ages`
is the owner's age count.

`UnitData::attack` — if the base is zero, zero, full stop (nothing below
turns a non-combatant into one). Otherwise `+ terra_cotta_attack × 10` with
the Terra Cotta Army; `+ thecitizen_attack_bonus × 10` under The Citizen's
aura; `+ obsidian_archers_attack × 10` for a FOOT_ARCHER or HORSE_ARCHER type
whose owner has the Obsidian bonus; `× (space_air_attack + 100) / 100` for an
air unit with the Space Program; and the named generals' auras, each `× 10`
and each gated on the owner *having* that general and the unit being under
it: Alexander (all; extra for Hoplites), Napoleon (siege), Parmenio
(Cataphracts), Ptolemy (Catapults), Spitamenes (Horse Archers), Blücher
(stable units), Djezzar (all), Memnon (Greek Mercs), The Monarch (stable
units).

`BuildData::attack` — the base, then `+ general_building_attack × 10` for a
general garrisoned (a tower, fort or city building only), `+
obsidian_archers_attack × 10`, `+ antipater_garrison_attack_bonus × 10` with
Antipater inside.

### 4.2 `armor()`

`ObjectData::armor` — `type.armor`, plus the Dutch `ages × dutch_attack_bonus`
on the same unit kinds (the patriot exclusion only from patch version 9).

`Unit::update_armor` caches `ObjectData::armor` (+ `cattle_citizen_armor` for
citizens with the Cattle bonus) into `myarmor` on every figure of the squad;
`UnitData::armor` returns `myarmor` plus `general_rally_armor × (general
upgrade + 1)` when a general (or the unit *is* a hero) is in rally range,
plus Ptolemy, Darius, Wellington, Blücher, Memnon, The Citizen and The
Monarch's armour bonuses under the same gating as attack.

### 4.3 `hits` — `Unit::update_hits`

The squad's hit points: `type.hits` (for citizens, the age-variant type's),
`+ americans_marine_hp_bonus` (entrenched marines), `× (copper_factory_hp +
100) / 100` (ships and factory units with Copper), `× (bananas_hp_bonus +
100) / 100`, `× (iroquois_extra_hits + 100) / 100` (Iroquois barracks
units), `+ ages × dutch_hp_bonus` (the Dutch kinds), spies' `spy_upgrade_hp[
level]`, heroes `+ lvl² × hits / 2`, `× (nubian_hit_points + 100) / 100` for
caravans, `+ supply_hp_upgrade[level]` for supply. The result is written to
`myhits` on the captain and **every figure down the chain**. What one figure
can absorb is not this number; see §7.3.

### 4.4 `max_range()`

`UnitData::max_range` — zero stays zero. A siege type adds `turk_siege_range`,
`liberty_siege_range`, `eiffel_siege_range`; a non-siege ranged type adds
`theceo_unit_range` under The CEO. Then `+ ptolemy_range_bonus` (ranged, under
Ptolemy), `+ napoleon_siege_range` (siege, under Napoleon), `+
rum_naval_range` (domain sea, Rum), `+ obsidian_archers_range` (archers,
Obsidian). `BuildData::max_range` is `BuildTypeData::get_building_range(who)`
— the `FORT_UPGRADE_RANGE`/`TOWER_FORT_RANGE` arrays `docs/SUPPLY.md` lists —
plus `general_building_range` for a garrisoned general and a city's temple
range. `min_range()` is the type's.

### 4.5 `recharge()` — see §8.3.

---

## 5. The combat table

### 5.1 What it is

`Balance::combat_table.final_balance_table` is a `short[493][493]`: for an
attacker type `a` and a target type `b`, both `TypeIndex` values in
`BASE_UNITTYPES (0x32)` … `END_BUILDTYPES (0x21e)`, entry `[a − 0x32][b −
0x32]` is a **percentage** — what `a`'s attack is worth against `b`. It is the
first thing `get_damage` multiplies by and the only place a type's matchups
live; there is no per-unit bonus list. A unit type is an index into its
family's table (`TypeIndex − 0x32` is the unit id, `− 0x19e` the building
id), so this is a table over the union of the two families.

It is **built at load**, not shipped. `Balance::fill_tables` (from
`Balance::init`, once per type-table load) does, for each `(a, b)`:

```
entry(a, b) = compute_modifier(a, b)
            = 100                                   if a or b is a gaia type
            = type_damage(a, b) * xml_product(a, b) / 100   otherwise
```

and `xml_product` is a running product of percentages out of `balance.xml`,
starting from 100, over every category `a` belongs to (rows) against every
category `b` belongs to (columns): `p = (xml[row][col] * p) / 100`, truncating
each step, outer loop rows, inner loop columns. `balance.xml` is one
`<TABLE>` of `<ENTRY name="row" col="pct" .../>` rows; the 399 category
names are: the 352 unit types by their unit-table index; five lines (`SIEGE`,
`FORTS`, `TOWERS`, `CITIES`, `OBSPOST`); two objects (`BUILDINGS`, `UNITS`);
eight ages (`AGE_0`–`AGE_7`); and the 32 `Flag_X_OBJMASK_*` rows, one per
`obj_masks` bit. A type's categories (`Balance::return_pack`) are, in this
order: its own row (units only — `TypeIndex − 0x32`); its **line** — for a
unit, `SIEGE` if `UnitTypeData::is_siege` (`unit_flags & 0x20000`); for a
building, `FORTS` if `is_fort` (the `FORTX` lineage), else `CITIES` if
`is_city` (the `VILLAGE` lineage), else `OBSPOST` if `is(LOOKOUT)`, else
`TOWERS` if `is(TOWER)`, else none; its object class (`UNITS`/`BUILDINGS`);
its age (`ObjectTypeData::get_age` — `+0x278`, or `get_age_slow`'s
first-tech-prerequisite age); and one row per `obj_masks` bit set, bits
ascending. The 399 names are built from `internal_strings.xml` (the lines,
objects, ages and flag comments) and the unit types' `<NAME>` with spaces
turned to underscores — which is exactly the order the shipped file's rows
are in. A row the file lacks, or an attribute it lacks, is 100 — and so is a row
the file *has* under a name the unit table no longer carries: the shipped
file's `Pathfinder`, `Pioneer`, `Ranger` and `Marines` rows (and columns)
name units since renamed, and contribute nothing. The shipped file is far
from identity — some five hundred entries are not 100 — so the table is not
recoverable from the hardcoded rules alone.

The table is built **once per type-table load** (`Balance::init` is the last
call of `Types::init`, from `Game::init_non_rules_data` and a mod switch),
walked into the rules checksum (`Balance::walk_rules_data`), and **dumped in
full by `Game::log_rules_data`** under a `COMBATTABLE` heading as
`final_balance_table[a][b]` lines — so a `RULES=1` logged start
(`docs/ORACLE.md`) prints all 243,049 values and is the oracle a
regenerating tool is checked against. ~~One fact about the names is open:
the separator between the flag letter and `OBJMASK_…` is internal string
17, shipped as an *empty* element whose `hash` is that of a single space;
if the engine loads it empty the 32 `Flag_` rows and columns never match
and contribute 100. Open question 8; the `RULES=1` dump settles it.~~
**Settled (§14.9, §15): the 32 `Flag_` rows and columns never match and
contribute 100**; `rondata::balance::tail_names` composes them under a name
that is not the file's, and the survey lists the file's 32 as dead.

This is `docs/DECISIONS.md` entry 13's case exactly: a table the original
computes once before the first frame. The simulation takes it as an input
(`combat::Table`), `rondata` regenerates it from the user's install —
`sim::balance::type_damage` for the hardcoded half, the install's
`balance.xml` for the other — and the document records the rules rather than
the numbers.

### 5.2 `Balance::type_damage` — the hardcoded half

A single ordered chain of integer percentage multiplications starting from
100, each `p = (p * k) / 100` truncating, keyed on the attacker's and target's
`obj_masks`, a few named types, whether the target is a building, and the age
difference. The order matters only through truncation, and it is recorded in
full in §5.3 exactly as executed.

The first step is **the age bonus**: if both are unit types and the
attacker's age exceeds the target's, `p = 100 + {one,two,three,four,five}_
ages_down` for a difference of 1, 2, 3, 4, 5+ — the shipped 115, 120, 150,
160, 170. (A unit type here is `0x32 ≤ TypeIndex < 0x19e`, or whatever the
type's `is_unit_type` virtual says; `get_age` is the type's age, with the
`−1` fallback resolved through `get_age_slow`.) **`get_age_slow`, read
2026-08-20** (`ObjectTypeData/get_age_slow@00661a00`): the first
prerequisite slot that names a tech decides — if that tech is an **age**
tech the result is its `AGE` column **plus one** (the column is 0 for
Classical … 6 for Information, so a unit needing Classical is an age-1
unit), any other tech's `AGE` as written; no tech prerequisite is 0. The
loader's first draft returned the age tech's index without the +1 and put
306 of 364 units one age early; the dump's per-type `age` caught it (§15.2).

### 5.3 The `type_damage` chain

The chain below is the function in execution order, as read by the second
reader of this mechanic and spot-checked block by block against the decompile
by the first (blocks 2–9 line by line; the rest by their constants). Notation
is the report's: `A`/`T` are the attacker's and target's `obj_masks` with §3's
letters; `unit(x)`/`build(x)`/`wonder(x)` are the `TypeIndex` range tests
(`0x32..0x19d`, `0x19e..0x21e`, `0x20e..0x21e`) behind the `is_*_type`
virtuals; `siege(x)` is `UnitTypeData::is_siege` — which is **`unit_flags &
0x20000`, letter `r` of the `FLAGS` column, not the SIEGE object mask** — and
is the `return 0` stub on a building type; `missile(x)` is `has_objmask(
MISSILE)`; `caravan(x)` is `unit_flags2 & 8`; `is(x, N)` is `ObjectTypeData::
is` with the type name from the enum. `×n` is one truncating `v = (v * n) /
100`; `×2`/`×4`/`×5`/`×8` are exact; `/2`, `/3` truncate. The early return at
block 43 is the only one.

Every step is independent (no `else` between steps unless shown); within a
step the lines apply in the order listed.

| # | attacker condition | target condition | multiplier |
| --- | --- | --- | --- |
| 0 | — | — | `v = 100` |
| 1 | `unit(a)` | `unit(b)` and `tAge < aAge` | `v = ((c+100)*100)/100 = c+100`, `c` = `constants->one_age_down` (diff 1), `two_ages_down` (2), `three_ages_down` (3), `four_ages_down` (4), `five_ages_down` (≥5). `rules.xml`: `ONE_AGE_DOWN 15%, TWO 20%, THREE 50%, FOUR 60%, FIVE 70%`, read by `Constants::get_item` (`Constants/init@00569a90.c:129-138`, leading integer per `docs/ECONOMY.md:136`) → v = 115/120/150/160/170 |
| 2 | `A&Q` | `T&K` | ×162 |
| 2 | `A&Q` | `T&O` | ×152 |
| 3a | `A&H` and `!(A&G)` | `T&M` | ×166 |
| 3a | `A&H` and `!(A&G)` | `build(b)` | ×130 |
| 3a | `A&H` and `!(A&G)` | `T&K` | ×86 |
| 3a | `A&H` and `!(A&G)` | `T&D` | ×75 |
| 3a | `A&H` and `!(A&G)` | `T&I` | ×75 (written `v*0x4b; goto /100` — one step) |
| 3b | `A&H` and `A&G` | `T&M` | ×257 |
| 3b | `A&H` and `A&G` | `T&O` | ×66 |
| 3b | `A&H` and `A&G` | `T&4` | ×114 |
| 3b | `A&H` and `A&G` | `is(b, ARMOREDCAR 0xd8, 0)` | ×225 |
| 3b | `A&H` and `A&G` | `is(b, LIGHTTANK 0xef, 0)` | ×132 |
| 3b | `A&H` and `A&G` | `T&I` | ×80 |
| 4 | `A&K` | `T&Q` | ×65 |
| 4 | `A&K` | `T&D` | ×90 |
| 4 | `A&K` | `T&H` | ×253 |
| 4 | `A&K` | `T&O` | ×138 |
| 4 | `A&K` | `T&P` | ×80 |
| 4 | `A&K` | `build(b)` | ×33 |
| 4 | `A&K` | `siege(b)` | ×75 |
| 5 | `A&4` | `T&K` | ×114 |
| 5 | `A&4` | `T&Q` | ×158 |
| 5 | `A&4` | `T&D` | ×149 |
| 5 | `A&4` | `T&I` | ×159 |
| 5 | `A&4` | `is(b, MACHINEGUN 0x7b, 0)` | ×130 |
| 5 | `A&4` | `is(b, FLAMETHROWER 0x83, 0)` | ×125 |
| 5 | `A&4` | `T&S` | ×122 |
| 5 | `A&4` | `T&A` | ×98 |
| 5 | `A&4` | `build(b)` | ×33 |
| 5 | `A&4` | `caravan(b)` or `b ∈ {MERCHANT 0x3d, MERCHANTDUTCH 0x3e, FURTRAPPER 0x190}` | `/3` |
| 5 | `A&4` | `b ∈ {PEASANTS 0x32, PEASANTSKOREAN 0x33, SCHOLARS 0x34, SCHOLARSKOREAN 0x35}` or `is(b, MILITIA 0x42, 0)` | `/2` |
| 6 | `A&O` | `build(b)` | ×33 |
| 6 | `A&O` | `siege(b)` | ×75 |
| 6 | `A&O` | `T&4` | ×57 |
| 6 | `A&O` | `T&K` | ×57 |
| 6 | `A&O` | `T&Q` | ×66 |
| 6 | `A&O` | `T&D` | ×66 |
| 6 | `A&O` | `T&H` | ×160 |
| 6 | `A&O` and `A&V` (nested in 6) | `T&4` | ×160 |
| 6 | `A&O` and `A&V` | `T&K` | ×135 |
| 6 | `A&O` and `A&V` | `T&Q` | ×2 |
| 6 | `A&O` and `A&V` | `T&D` | ×2 |
| 6 | `A&O` and `A&V` | `T&H` | ×50 |
| 6 | `A&O` and `A&V` | `T&Y` | ×97 |
| 6 | `A&O` and `A&G` (nested in 6) | `is(b, MACHINEGUN, 0)` | ×155 |
| 6 | `A&O` and `A&G` | `T&A` | ×92 |
| 7 | `A&Y` | `T&K` | ×170 |
| 7 | `A&Y` | `T&Q` | ×179 |
| 7 | `A&Y` | `T&D` | ×155 |
| 7 | `A&Y` | `T&I` | ×170 |
| 7 | `A&Y` | `is(b, MACHINEGUN, 0)` | ×155 |
| 7 | `A&Y` | `is(b, FLAMETHROWER, 0)` | ×140 |
| 7 | `A&Y` | `build(b)` | ×33 |
| 8 | `A&D` | — | ×133 |
| 8 | `A&D` | `T&K` | ×125 |
| 8 | `A&D` | `T&H` | ×180 |
| 8 | `A&D` | `T&O` | ×114 |
| 8 | `A&D` | `build(b)` | ×66 |
| 8 | `A&D` | `siege(b)` | ×75 |
| 8 | `A&D` and `aAge == 4` | `T&H` | ×110 |
| 9 | `A&A` and `is(a, LIGHTTANK 0xef, 0)` | `T&D` | ×155 |
| 9 | `A&A` and `is(a, LIGHTTANK, 0)` | `T&I` | ×185 |
| 9 | `A&A` and `is(a, LIGHTTANK, 0)` | `T&Y` | ×115 |
| 9 | `A&A` and `is(a, LIGHTTANK, 0)` | `T&4` | ×120 |
| 9 | `A&A` and `is(a, LIGHTTANK, 0)` | `T&O` | ×120 |
| 9 | `A&A` and `is(a, LIGHTTANK, 0)` | `is(b, MACHINEGUN, 0)` | ×175 |
| 9 | `A&A` and `is(a, ARMOREDCAR 0xd8, 0)` | `T&I` | ×175 |
| 9 | `A&A` and `is(a, ARMOREDCAR, 0)` | `T&D` | ×175 |
| 9 | `A&A` and `is(a, ARMOREDCAR, 0)` | `is(b, LIGHTTANK, 0)` | ×120 |
| 9 | `A&A` | `build(b)` | ×66 |
| 10 | `A&I` | `T&H` | ×260 |
| 10 | `A&I` | `is(b, ARMOREDCAR, 0)` | ×170 |
| 10 | `A&I` | `is(b, LIGHTTANK, 0)` | ×75 |
| 10 | `A&I` | `is(b, MACHINEGUN, 0)` | ×66 |
| 10 | `A&I` | `T&C` | ×120 |
| 10 | `A&I` | `siege(b)` | ×75 |
| 10 | `A&I` | `build(b)` | ×66 |
| 11 | `is(a, MACHINEGUN 0x7b, 0)` | `T & (Q\|D)` (0x10008) | ×330 |
| 11 | `is(a, MACHINEGUN, 0)` | `T&I` | ×330 |
| 11 | `is(a, MACHINEGUN, 0)` | `T&H` | ×330 |
| 11 | `is(a, MACHINEGUN, 0)` | `T&C` | ×5 (exact) |
| 11 | `is(a, MACHINEGUN, 0)` | `T&A` | ×50 |
| 11 | `is(a, MACHINEGUN, 0)` | `build(b)` | ×50 |
| 11 | `is(a, MACHINEGUN, 0)` | `siege(b)` | ×75 |
| 12 | `is(a, FLAMETHROWER 0x83, 1)` (strict) | `T&D` | ×88 |
| 12 | `is(a, FLAMETHROWER, 1)` | `T&I` | ×80 |
| 12 | `is(a, FLAMETHROWER, 1)` | `T&M` | ×125 |
| 13 | `A&U` | `T&A` | ×180 |
| 14 | `is(a, MILITIA 0x42, 0)` | `T & (O\|Q)` (0x14000 — bits 14 and 16, HORSE_ARCHER and LIGHT_INF; ~~M\|O~~ was a misread of the literal, M\|O being 0x5000; the dump's Citizen → Slingers 200 and Citizen → General 100 settle it) | ×2 |
| 15 | `a ∈ {PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN}` (0x32..0x35) | `T & (O\|Q)` | ×2 |
| 16 | `A&N` and `!(A&S)` | `build(b)` | ×33 |
| 16 | `A&N` | `unit(b)` and `!(T & (N\|3))` (0x10002000) and `!siege(b)` | ×33 |
| 16 | `A&N` | `unit(b)` and `!(T&N)` and `siege(b)` | ×66 |
| 16 | `A&N` and `is(a, BOMBARDSHIP 0x15a, 0)` | `T&N` | ×33 |
| 16 | `A&N` and (`is(a, BARK 0x143, 0)` or `is(a, SUB 0x152, 0)`) | `(T & (N\|C)) == (N\|C)` | ×2 |
| 17 | `is(a, FIRERAFT 0x14e, 0)` | `is(b, TRIREME 0x154, 0)` | ×310 |
| 18 | `is(a, FIRERAFT, 0)` | `is(b, BOMBARDSHIP 0x15a, 0)` | ×4 (`<<2`) |
| 19 | `is(a, FIRERAFT, 0)` | `is(b, FIRERAFT, 0)` | ×4 |
| 20 | `is(a, TRIREME 0x154, 0)` | `is(b, BARK 0x143, 0)` | ×160 |
| 21 | `unit(a)` and `!(A&N)` and `!(A&S)` | `T&N` | ×33 |
| 21 | `unit(a)` and `!(A&N)` and `siege(a)` | `T&N` | ×350 |
| 22 | `is(a, BOMBER 0x130, 0)` or `is(a, FIGHTERBOMBER 0x134, 0)` | `build(b)` and `T&6` | ×25 |
| 23 | `A&3` and `!missile(a)` and `A&6` | `build(b)` and `!(T&6)` | ×15 |
| 23 | same | `build(b)` and `T&6` | ×85 |
| 23 | same | `is(b, BOMBER 0x130, 0)` | ×2 |
| 23 | same | `siege(b)` | ×125 |
| 23 | same | `T&N` | ×350 |
| 24 | `is(a, HELICOPTER 0x136, 0)` | `is(b, FIRERAFT, 0)` | ×8 (`<<3`) |
| 24 | `is(a, HELICOPTER, 0)` | `is(b, LIGHTTANK, 0)` | ×450 |
| 25 | `unit(a)` and `!(A&6)` | `is(b, HELICOPTER 0x136, 0)` | ×25 |
| 26 | `is(a, V2ROCKET 0x139, 0)` | `build(b)` and `wonder(b)` | `/2` |
| 27 | `A&W` | `is(b, SUPPLYWAGON 0x3f, 0)` | ×180 |
| 27 | `build(a)` | `is(b, SUPPLYWAGON, 0)` | `v = (v*14)/5` (= ×280) |
| 28 | `A&S` | `build(b)` | ×430 |
| 29 | `A&B` | `build(b)` | ×250 |
| 30 | `a.domain (+0x218) != 0` or `A&S` | `is(b, AIRBASE 0x1bf, 0)` | `/3` |
| 31 | `is(a, BALAMOBSLINGERS 0x58, 0)` | `T&Q` | ×150 |
| 32 | `is(a, KUSHITEARCHERS 0xae, 0)` | `T&K` | ×125 |
| 32 | `is(a, KUSHITEARCHERS, 0)` | `T&O` | ×125 |
| 33 | `is(a, INTICLUBMEN 0x5b, 0)` | `T&Y` | ×115 |
| 33 | `is(a, INTICLUBMEN, 0)` | `T&4` | ×115 |
| 33 | `is(a, INTICLUBMEN, 0)` | `T&O` | ×115 |
| 34 | `A&O` and `is(a, CAMELRANGE2 0xbf, 0)` | `T&Q` | ×135 |
| 34 | `A&O` and `is(a, CAMELRANGE2, 0)` | `T&D` | ×120 |
| 34 | `A&O` and `is(a, CHARIOT 0xc3, 0)` | `T&O` | ×150 |
| 34 | `A&O` and `is(a, NOMAD 0xc7, 0)` | `T&Q` | ×160 |
| 34 | `A&O` and `is(a, NOMAD, 0)` | `T&D` | ×145 |
| 35 | `A&4` and `is(a, RUSINYLANCER 0xe0, 0)` | — | ×105 |
| 36 | `A&K` and `is(a, LONGBOWMEN 0xb1, 1)` | `T&H` | ×130 |
| 36 | `A&K` and `is(a, ELONGBOWMEN 0xb2, 1)` | `T&H` | ×130 |
| 36 | `A&K` and `is(a, KINGSYEOMANRY 0xb3, 1)` | `T&H` | ×130 |
| 36 | `A&K` and `is(a, KUSHITEARCHERS 0xae, 1)` | `T&K` | ×135 |
| 37 | `A&Y` and `is(a, ECOMPANION 0xe8, 0)` | — | ×105 |
| 38 | `A&H` and `is(a, LEGIONS 0x92, 0)` | `T&H` | ×2 |
| 38 | `A&H` and `is(a, SAMURAI 0xa0, 1)` | — | ×50 |
| 38 | `A&H` and `is(a, HALBERDIERS 0x95, 0)` | `T&4` | ×180 |
| 38 | `A&H` and `is(a, HALBERDIERS, 0)` | `T&Y` | ×180 |
| 38 | `A&H` and `is(a, HALBERDIERS, 0)` | `T&O` | ×110 |
| 38 | `A&H` and `is(a, TERCIOS 0x97, 0)` | — | ×130 |
| 38 | `A&H` and `is(a, RECOILGUN 0x90, 0)` | `T&V` | ×135 |
| 39 | `is(a, HIGHLANDERS 0x72, 0)` | `T&F` | ×130 |
| 40 | `is(a, HVYMACHINEGUNMG42 0x82, 1)` | `T&F` | ×130 |
| 41 | `is(a, TIGERTANK 0x102, 1)` | `T & (Q\|I\|D)` (0x10108) | ×115 |
| 41 | `is(a, TIGERTANK, 1)` | `is(b, MACHINEGUN, 0)` | ×120 |
| 42 | `is(a, LEOPARDTANK 0x103, 1)` | `T & (Q\|I\|D)` | ×115 |
| 42 | `is(a, LEOPARDTANK, 1)` | `is(b, MACHINEGUN, 0)` | ×120 |
| 43 | `!(A&S)` | — | **return v** |
| 44 | `is(a, FLAMINGARROW 0x117, 0)` | `build(b)` | ×135 |
| 45 | `is(a, BASILICABOMBARD 0x114, 0)` | `build(b)` | ×110 |
| 46 | `is(a, MORTAR 0x112, 0)` | `build(b)` | ×110 |
| — | — | — | return v |

Notes on the table:
- Step 1's "unit type" is `is_unit_type` = 0x32..0x19d, which includes the
  gaia range — irrelevant here because `compute_modifier` never reaches
  `type_damage` for gaia.
- Steps 6's VEHICLE and GUN sub-blocks are nested inside the HORSE_ARCHER
  block (4-space indent at `type_damage@0057fb50.c` filtered lines 278/298,
  closed at 308).
- Step 43 is the only early exit: everything from 44 on needs `A&S`.
- The target-side `siege/missile/caravan` slots are real only on unit types;
  for a building target the stubs make those lines dead, for a building
  attacker likewise (`is(a, …)` on a BuildType still works — `ObjectTypeData::is`
  is shared).
- All `/100` are single truncating divisions on the running value; there is no
  accumulation of a product before dividing anywhere except the `goto` in 3a,
  which is still one `×75/100`.

---

## 6. The damage formula — `ObjectData::get_damage`

`get_damage(this = attacker A, o, who = target T, angle, splash, check_overkill,
*dtype) → int`. Every step below is in the order the function executes it;
all divisions truncate toward zero except the `>> 8`s, which are the
sign-fixed arithmetic shift the compiler emits for a signed `/ 256`. `A.is(X)`
is `ObjectData::is(TypeIndex, strict)` — the type's `is_list`, i.e. "is X or
an upgrade of X" unless `strict`. "Build-proper" is the `Object` virtual at
`+0x20`, true for a `Build` and false for a `Wall`; "is a building" is `+0x1c`,
true for both.

```
pct    = combat_table[A.type − 0x32][T.type − 0x32]        (§5)
dtype  = 2
armor  = T.armor()                                           (§4.2)
attack = A.attack()                                          (§4.1)
amask  = A.type.obj_masks
tmask  = T.type.obj_masks
```

1. **Gunpowder arrows.** If A is a `CASTLE` or `FORTX`, or A's object flag
   `0x20` is set (it is a city building), and A's owner has `GUNPOWDER_AGE`
   (`has_tech`, inlined as `tech[0x44] >> 2 & 1`), clear **ARCHERY** from
   `amask` for the rest of this call. The castle's arrows are bullets.
2. `base = attack * pct / 100`.
3. If `amask` has **MUSKET_INF**: `armor = armor * 133 / 100`.
4. If A is Build-proper, `tmask` has **SIEGE**, T is a unit whose type packs
   (`unit_flags2 & 4`) and is **not** packed (`unit_masks & 0x80000 == 0`):
   `base /= 3`. A tower does a third to deployed siege.
5. If A is Build-proper, T is a unit with non-zero `attack()` that `is_moving`:
   if `tmask` lacks **FOOT** and (T's type `is_siege` or T `is_supply` or
   `is_caravan`): `base /= 2`; if `tmask` has FOOT: `base = base * 3 / 4`
   (as `(base*3 + sign) >> 2`). Buildings hit moving targets for less.
6. If A is Build-proper and T is Build-proper and T's building is not active
   (`WallData::is_active`, flag `0x4` — under construction): `base *= 4`.
7. If T is Build-proper and A is a `PEASANTS`, `PEASANTSKOREAN`, `SCHOLARS`,
   `SCHOLARSKOREAN`, or `A.is(MILITIA)`: if the tile under T is not owned by
   A's player: `base /= 2`. Citizens and militia do half to buildings
   outside their own borders.
8. If `A.is(V2ROCKET)` and T is Build-proper and T's building is not active:
   `base /= 2`.
9. If A is a unit of domain **air** (2) and `amask` lacks MISSILE, and
   `T.is(REDFORT, strict)`: `base = base * (100 − red_fort_air_defense) /
   100`.
10. If T is a unit: if T is packed: `base *= 2`; if T's type packs and T is
    not packed: `armor += 1`.
11. `dtype`: `3` if `amask` has **EXPLOSIVE**, `3` if it has **BOMBARD**; if A
    is a building, `3` when `BuildData::get_shot() == 4` (the `REDOUBT`),
    else `2`. (`dtype` is cosmetic downstream — it picks the death
    animation's facing — and is carried but not modelled.)
12. **Wellington**: A is a unit, A's owner has a Wellington, A is under him
    (`has_general(0, WELLINGTON) ≥ 0`), and T's type is made `where ==
    FACTORY` (`T.type.where ≥ 0 && objecttypes[where].is(FACTORY)`): `base +=
    wellington_siege_attack` (not ×10; a raw addition to a tenths quantity).
13. **Japanese**: A is a unit made at a `BARRACKS` and A's owner has tribe
    bonus 0xf: `n = min(ages, epoch[0])`; `j = japanese_damage`; if `j < 0`,
    `j = −(j * n)`; `base = (j + 100) * base / 100`.
14. **Double damage**: if (A is Build-proper, or A's domain is land (0) and
    T's is not) and T is a unit and (`T.unit_masks & 0x400000`, or T's domain
    is sea (1) and the game's `team_style == 2` and the leader of T's owner's
    `get_target()` is not A's owner): `base *= 2`.
15. **Splash** (the `splash` argument): if T is a unit: `base /=
    T.type.uber_size`. `base = A.type.splash_percent * base / 100`. If T is a
    unit: if T's type `is_siege`, packs, and is not packed: `base *= 3`; if
    `T.is(BARK)`: `base = base * 25 / 100`.
16. **River**: T is a unit and T's `z < 0`: `base = (base * river_modifier) >>
    8`.
17. **Decoy**: T is a unit with `unit_masks & 1`: `armor = 0`; `base =
    max(A.attack(), base) * 1000`. Anything kills a decoy.
18. **Unfriendly territory**: T is a building and `WallData::in_unfriendly_
    territory` and the game is not in balance-test mode: `armor = 0`; if T is
    not Build-proper (a wall) or T's type has zero attack: `base *= 4`.
19. **Flanking.** Only if A and T are both units; neither `amask` nor `tmask`
    has **CIVILIAN**; neither has **AIR**; `amask & NAVAL == tmask & NAVAL` and
    `tmask` lacks NAVAL. Let `d = T.angle − angle` (the `angle` argument is
    the attacker's facing toward T at the moment of the attack, §8.4) and
    `e = d + 0x80000000` (i.e. `d + 180°`) as a 32-bit unsigned angle. No
    flank if `e < 0x2aaaaaaa` (60°) or `e > 0xd5555555` (300°) — the target is
    facing the attacker to within 60°. Otherwise `level = 1 + (e + 0xa0000000
    > 0x40000000)`, which is `1` for `e ∈ [135°, 225°]` (`d` within ±45° of
    0: T faces the way the attack travels — **attacked from behind**) and `2`
    for the remaining sectors on either side. Then `fb = flank_bonus`; if the
    mask (see note) has **VEHICLE**: `fb = (fb * vehicle_flank_bonus) >> 8`,
    else if it has **MOUNTED**: `fb = (fb * cavalry_flank_bonus) >> 8`; and
    `base = (fb * level + 100) * base / 100`. With the shipped 50: rear
    ×1.5, side ×2.0 — unless the facing convention is the other way round,
    which is open question 1. *Note:* the mask the VEHICLE/MOUNTED test reads
    is a register the decompiler lost (`extraout_EDX`); the natural candidate,
    and the one taken, is `amask` — a vehicle or cavalry **attacker** takes a
    reduced flank bonus. Open question 2.
20. **Net damage:** `dmg = (base + 5) / 10 − armor`. Attack was in tenths;
    this is the rounding back to whole hits, and armour is subtracted
    *after* every multiplier above.
21. **Overkill** (only when `check_overkill`, which `do_damage` always passes):
    if `A.max_range() != 0` and A and T are both units, and T's
    `damage_frame != 0` and `frame − T.damage_frame < overkill_frames` and
    `A.get_captain() != T.damage_o`: `dmg = (dmg * overkill_damage) >> 8`; and
    if `T.is(CATAPULT)` and A's type is not siege: `dmg /= 2`. §7.1 says how
    the record is kept: the **first** ranged squad to hit a figure owns a
    30-frame window; every other squad's hits in it are cut to a third.
    Melee attackers neither suffer it nor open a window.
22. **Rocky**: `tmask` has any of **LIGHT_INF**, **MODERN_INF**, **MUSKET_INF**
    and the tile under T has terrain flag `0x8`: `dmg = (dmg *
    rocky_modifier) >> 8`.
23. **Height**: neither domain is air, A's type is not siege, and `T.z <
    A.z`: `dmg += (A.z − T.z) * height_bonus * dmg / (height_increment *
    100)`.
24. **Entrenchment**: T is a unit with `unit_masks & 0x2000000` and
    `!A.is(FLAMETHROWER)`: `d = T.trench_angle − angle`; `c = 0` if `|d| >
    120°` (as `d + 0x55555556 ≥ 0xaaaaaaac`), else `1 + (d + 0x20000000 >
    0x40000000)`. If `splash` or `c == 0`: `dmg = (dmg *
    entrenchment_modifier) >> 8`, and if `T.unit_masks2 & 0x1000`: `dmg =
    (dmg * antipater_entrench_bonus) >> 8`. The trench protects against fire
    from within 60° of the direction it faces, and against all splash.
25. **Cossacks**: `russian_cossack_damage != 0`, A's owner has tribe bonus
    0xd, A is a unit made at a `STABLE`, and T `is_supply` or T's type
    `is_siege`: `dmg = (russian_cossack_damage + 100) * dmg / 100`.
26. **At least one**: if `dmg < 1`: if (A's domain is not land, or T's domain
    is not sea) and `(tmask & AIR) == (amask & ANTI_AIR ? AIR : 0)` and not
    `splash`: `dmg = 1`. So a land attacker never scratches a ship, an
    anti-air unit never scratches a ground target, a non-anti-air unit never
    scratches a plane, and splash can do nothing.
27. `super_immune`: T's type is `SUPERCOLLIDER` and A is air and the flag is
    on: `dmg = 0`.
28. **Recapture**: T is Build-proper, active, a city building (flag `0x20`)
    with a city index, and the city's **`race`** (`CityData +0x5f`, the nation
    it is assimilated to) is A's owner: `dmg = (dmg * recapture_city_modifier)
    >> 8`. *(Corrected by `docs/CITIES.md` §7.4: an earlier draft said "the
    city's original owner", which is `founder` at `+0x60`. So the bonus is for
    the nation a stolen city still belongs to while it is unassimilated, and
    is gone once it assimilates.)*

Return `dmg` — whole hits, possibly zero or negative; `do_damage` clamps and
scales next.

**What the formula does not do**, and which earlier mechanics might lead one
to expect: no randomness (hit or miss is decided before this is called, by
where the projectile landed), no per-figure variation (every figure of a
squad does the same), and no dependence on the *target's* attack, age or
count except through the table.

---

## 7. Delivery — `Object::do_damage` and `Object::take_damage`

### 7.1 `do_damage(A, o, who, angle, num_guys, ammo, count, splash, quiet)`

Returns at once if `count < 1` or if A is a decoy. Then:

1. `dmg = get_damage(o, who, angle, splash, 1, &dtype)` — §6.
2. If the target is a unit and `!quiet`: the target's **captain** runs
   `Unit::target_opportunity(A)` — it may turn and engage (§12) — and if the
   target's `unit_masks & 0x10`: `dmg *= 2`. Then the **overkill record**:
   if `T.damage_frame == 0` or `frame − T.damage_frame >= overkill_frames`:
   `T.damage_frame = frame; T.damage_o = A.get_captain(); T.damage_who =
   A.who`. (So the window belongs to whoever hit first; a later hit inside
   it neither takes it over nor extends it.)
3. If A is not a missile and is active: `Object::attempt_launch(T, A.pos)` —
   the target, if it holds aircraft, may scramble them. Not modelled.
4. Sound, unless `splash`.
5. **Scale**, branching on what A is:
   - A is **not a unit** (a building): `s = dmg * count / A.type.ammo_per_att`.
   - A is a **unit**: `s = dmg * count`; if `s < 0x101`: `s = 0x100` (a unit's
     volley is at least one whole hit before the next two divisions); if
     `ammo ≥ 0`: `s /= A.type.ammo_per_att`; then `s /= A.type.uber_size`.
   Then `sixteenths = s >> 4` (sign-fixed), `frac = sixteenths & 0xf`
   (sign-fixed), `whole = sixteenths >> 4`. With `count = 0x100` and a unit of
   `uber_size` 3 attacking for 30: `s = 30 * 256 / 3 = 2560`, `sixteenths =
   160`, `whole = 10`, `frac = 0` — each figure's shot is a third of the
   squad's attack, and three of them fire.
6. Bookkeeping: which owner is attacking which (`attacked` bits both ways),
   the under-attack messages and the city alarms — all interface or AI. And
   `dtype`: `4` if the ammo's graphic piece has flag `0x10`; `1` if T is
   Build-proper.
7. `died = T.take_damage(whole, frac, dtype, ammo, 0, angle, A.o, A.who,
   splash)` — §7.2.
8. If `died` and T is a building: buildings-destroyed stat, the type's
   `get_kill_value / 5` to score, `Build::plunder` for a non-air, non-splash,
   non-allied kill. Return.
9. If not died: emergency AI; a `FLAMETHROWER` hitting a building **ejects
   its garrison** (and against a city, spawns citizens); entrenchment is
   **broken** on a hit that did not kill (`unit_masks &= ~0x2000000`, the
   two Antipater bits, `remove_entrench`); the Lakota coin trickle; planes
   ejected from a hit carrier/airbase by anti-air or bombers; then the
   **building's own splash**: every unit on the tiles within `circle_radius[1]`
   of a hit building, filtered by `Search::valid_search`/`valid_filter` (on
   the map, hostile to A, not Korean-building-under-fire), within
   `max(x_size, y_size) × 0xc0` of the building, not air, not a ship hit by
   siege, gets `do_damage(A, that unit, …, count / 8, splash = 0, quiet = 1)`
   — or `count / 4` when A is siege with a land domain — if within A's `max(
   max_range × 0xc0, 0x180)` of the building. Then the city-capture check
   and text bubble.

### 7.2 `take_damage(T, whole, frac, dtype, ammo, attrition, angle, o, who, splash)`

`attrition` is non-zero when attrition is the caller (`docs/ATTRITION.md`);
combat passes 0. Returns 0 (alive), 1 (died), 2 (died and the overflow was
passed on).

1. Not active → 0.
2. If `whole < 1` and `frac < 1`: `frac = 1`. Every hit that reaches here
   takes at least a sixteenth.
3. Combat only (`attrition == 0`): the owner's last-attacked frame (AI, on
   low difficulty); on the **first** damage to a building (`damage == 0`)
   a roll of `Random::get(0, 0xffff) % 100 < 5` is taken, and if the building
   is a fort (`BuildTypeData::is_fort`), `TEMPLE` or `TOWN` and the attacker
   is siege a second draw sizes a flock of birds — cosmetic, but both
   **consume game random** *(second reading corrected the type test)*; a missile fired by a non-`NUCLEARMISSILE` at someone not yet at war
   declares war.
4. **Accumulate**: `t = T.damage_frac + frac; whole += t / 16 (sign-fixed);
   T.damage_frac = t % 16 (sign-fixed); T.damage += whole`.
5. A building under construction (not `WallData::is_active`), hit by
   something other than an aircraft, in combat, with `whole > 0`: its
   `WallData::job_counter` loses `whole * 50` (clamped at 0). Hits knock
   progress off a building site.
6. T **not** Build-proper (a unit, a wall): the AI's peasant-alarm (a
   citizen hit inside a city with a finished city building: push an alarm
   group). No change to the numbers.
7. T **Build-proper**: city-under-attack flags; if `T.damage ≥ T.hits(0)` and
   T cannot carry aircraft: `eject_contents`; if T is a city building
   (`0x20`) that `is_active`: if `damage < hits` return 0, else set flag
   `0x10`, **clamp `damage = hits`** and return 0 — **a city never dies from
   damage**; it sits at zero and waits to be captured.
8. `share = T.hits(0)`. If T is a unit: `Unit::set_in_danger`; if
   `uber_size > 1`: if T is not the captain or `curr_uber_size() != 1`:
   `share = hits / uber_size`; else `share = hits − (uber_size − 1) * hits /
   uber_size`. **Each figure carries `hits / uber_size`, and the captain,
   once it is the last one standing, carries the remainder as well**, so
   the squad's total is exactly `hits`. An Aztec (tribe bonus 0) attacker
   from a `BARRACKS`/`STABLE`/`DOCK` marks T's captain `unit_masks |=
   0x4000` (plunder on kill).
9. If `T.damage < share`: mark the captain's flag `0x10` (damaged); a
   building that is not started, with `hits ≤ 2 × damage` and `build_masks
   & 0x2000`: `disband`, return 1. Else return 0.
10. **Death.** For a non-decoy unit hit by `dtype == 3` that is FOOT or
    MOUNTED: the figures' angles are set for the death animation. Stats:
    units-lost, and for the killer (`o ≥ 0`): units-killed, the dead type's
    `get_kill_value / 5` to score (subtracted, floored at 0, for an allied
    kill), `Unit::plunder` by the killer on a non-allied, non-splash kill
    (the Despot's unit plunder or a General/Supply Wagon/Caravan kill),
    the Inca refund. Then `T.die(dtype, gpiece, angle)` — `Object::die`
    → `close` (§11). If T was a **decoy** and the overflow `damage − share >
    0`: the captain takes it (`take_damage(overflow, 0, …)`), return 2.
    Else return 1.

### 7.3 What a figure can absorb, and why `hits` is the squad's

`update_hits` writes one number, the squad's, onto every figure, and
`take_damage` divides it on the way in; so a "Hoplites" squad with `HITS`
90 and `UBER_SIZE` 3 is three figures of 30, and when the first two fall the
captain's threshold becomes `90 − 2 × 30 = 30` — no change, because 90
divides; with `HITS` 100 the remainders land on the captain: 33, 33, 34.
`docs/ATTRITION.md` already treats the figure as the thing that bleeds; this
is where the figure's size comes from. `ObjectData::hits_left` and
`health_level` read `UnitData::total_damage`, which sums the chain and adds
`(uber_size − curr_uber_size) × hits / uber_size` for the dead — the health
bar shows the squad.

---

## 8. The firing cadence

### 8.1 Where it sits in the frame

`Unit::process` (every frame, every active unit; `docs/ATTRITION.md` has the
order) begins by decrementing `recharging` if it is non-zero, and `full` (the
"ready" counter) likewise. It then runs the unit's order: `Unit::work` →
`Unit::do_job(order_type)` → `Unit::do_attack` for an `ATTACK` order →
`Unit::fight`. An idle unit runs `Unit::do_idle` → `Unit::think` →
`Unit::think_attack`, which is how it acquires a target without being told
(§12). A `recharging` unit with no range and no cavalry-archer flag returns
from `work` before doing anything else unless its order has flag `4`; so a
recharging melee unit does not even move that frame.

### 8.2 `Unit::fight(o, who, flag, …, cavarch)` — the attack

The function is long because it also manages the order (switching to a
figure of the same squad when the targeted one died, re-targeting, moving
to an attack position, unpacking, the cavalry-archer dual mode). The
**attack itself** happens when: the order's target is a valid target
(`Object::valid_target`, §12.1); `recharging == 0` (a non-zero counter
returns 0 at the top, after a little animation housekeeping); and `T` is in
range (`ObjectData::is_in_range`, §13). Then, in this order:

0. *(second reading)* Before the range test, on every frame a **captain**
   with a non-mandatory order is not recharging and its target is a unit,
   **one `Random::get` draw is taken**; if the target is not a combat unit
   (`role & 0x10000` clear — or, for a unit with `unit_masks2 & 4`, is one
   that is not a `unit_flags & 0x2000` type) and the draw is not `% 5 == 0`,
   or the order has flag `0x10`, the unit looks for a better target
   (`find_new_target`) and rewrites the order if it finds one; otherwise
   `poor_target` decides. (An earlier draft took the draw only for
   non-combat targets.) A group order takes this path only for the group's
   leader. Under AI control a guarding unit also rolls once per frame and
   drops a charge on an odd draw.
1. A packed siege type in range **unpacks** (`add_cast_order(UNPACK)`) and
   returns — it cannot fire packed — unless it can get a better position.
   *(second reading)* **An unpacked packer type (`unit_flags2 & 4`, not the
   Dutch merchant) that is siege, with a unit target, does not shoot the
   unit: it inserts an `ATTACK_GROUND` order at the target's current position**
   (`accuracy` flag set when the target is at sea) and returns — siege fire
   at units is ground fire, with no target to home on, and what it hits is
   what stands where it lands (§9.3, §9.4).
2. `unit_masks |= 0x11000` (in combat, attacking). `set_attack(o, who)`:
   every figure's `attack_o`/`attack_who` are set and, for a ranged type with
   pivot restrictions, `Guy::set_all_pivots` — a return that means "wait for
   the pivot", which makes the unit skip the attack this frame (the
   `param_4 != 0` early-out below).
3. **Facing**: `angle = find_angle(T − A)`; for a wall target, the angle is
   snapped to the side of the wall the attacker is on; a `GUN`-flagged type
   (`unit_flags & 0x40`) that is not a `PATROLBOAT` vs a sea target snaps to
   whichever of `angle ± 90°` is nearer its current facing (broadside). If
   the pivot said wait, `angle = A.angle`. `set_angle(angle)` if it changed;
   each figure's desired angle is set (for a squad of `squad_size` guys each
   gets the angle to the target from its own position — except single-figure
   and sea types, which all take the unit's).
4. **Animation**: `CHAR_ATTACK1`/`ATTACK2`/`ATTACKSPECIAL` by flags; an
   `IMMORTALS` type within `0xc0` of its target strikes with the special and
   does its damage *here* (per figure, `count 0x100`); a unit with `DETECT`
   and a jammed radar rolls `rnd < jam_unit_radar_prob` (`GameAccess::rnd`,
   the other RNG) to misfire.
5. `set_attacking(who)`: the attacked-by bits, and the target owner sees the
   attacker (`update_local_seen`) if it did not already.
6. **The strike.** If `A.type.max_range == 0` (melee): if `A.type.fire_proj
   != 0` (any type with non-zero attack) or this is the cavalry-archer melee
   call: for each of the unit's `guy_mark` figures: `Object::do_damage(A, o,
   who, angle, guy_mark, −1, 0x100, 0, 0)`. **Melee damage lands the frame of
   the attack**, not at an animation event; there is no event path
   (`Object::execute_events` is empty). Otherwise (ranged): if
   `unit_flags & 0x10000` is clear: `Object::fire_ammo(o, who)` (§9) — a
   `MERCHANTDUTCH` that is not packed temporarily swaps its projectile for
   the eighth ammo piece (a flare) — and a type with `unit_flags & 0x2000`
   **dies after firing** (`die(3)`). If `unit_flags & 0x10000` is set (the
   sweep types): every enemy unit and building within `(3 × max_range + 6) ×
   0x40` / `(3 × max_range + 3) × 0x40`, that is land-domain or the target
   itself, within `max_range × 0xc0 + 6` by `attack_dist`, within ±90° of
   `angle`, and whose perpendicular distance from the firing line is at most
   the two big radii (units) or `(x_size + y_size) × 0x30` (buildings), takes
   `do_damage(…, 0x100, splash = (not the target), 0)` per figure — a cone.
7. `recharging = recharge()` (§8.3; the byte truncates anything over 255).
   The army remembers a building target. Return 1.

`Unit::do_attack` calls `fight` every frame the order is current; the
cavalry archer calls it twice (§8.5). Building targets that are transport
barges, and targets inside a transport, are special-cased before `fight`.

### 8.3 `UnitData::recharge()`

`r = type.recharge`. If the type is **not** siege: `r`. If it is siege and
(`artillery_under_attack_fires_slowly == 0` or the unit is not under attack
(`unit_masks2 & 1`)) and the unit is **in supply** (`in_supply`,
`docs/SUPPLY.md`): `r`. Otherwise — siege out of supply, or under attack
with the flag on — `r * 2` if the type `is(BOMBARD)`, else `r * 3 / 2`. The
two `*_OUT_OF_SUPPLY_RELOAD` constants exist and are loaded (8.8, `3/2` and
`2/1`) and **are not read here**; the ratios are literals.

### 8.4 The angle that reaches `get_damage`

The `angle` argument to `do_damage` — and so to the flank and entrenchment
tests — is the attacker's facing toward the target as `fight` just set it
(step 3), or the ammo's stored `angle` (set at launch from the unit's
`+0x50 angle`) for a projectile. For splash from a projectile it is
`find_angle(landing → victim)`.

### 8.5 The cavalry archer

A type with `unit_flags & 0x400` has `max_range = 0` and `second_max_range
= RANGE`'s max (§2.1). `Unit::do_attack`: with the combat stance at 2 it
sets `type.max_range = second_max_range` for the duration of one `fight`
call and restores 0 — the ranged mode. Otherwise it calls `fight` as melee
and then, if that attacked, `cavarch_fight` with `recharging` forced to 0
and restored — a second, melee-range swing at the nearest melee target. The
type's `max_range` is **mutated on the shared type object** for the call;
two cavalry archers of the same type cannot interleave inside one call, so
this is safe and deterministic, but it is a fact about the data model worth
knowing.

### 8.6 Buildings — `Build::process` and `Build::do_attack`

A building with non-zero `type.attack` runs `do_attack` every frame it has a
target, every 32nd frame (`(frame + o) & 0x1f == 0`) when it has none, and
whenever the object adjacent to it (`near_o`) is in range. `do_attack`:

- *(second reading — the first draft had this inverted)* A building without
  ANTI_AIR, or a `LOOKOUT`/`OBSERVATIONPOST`: if `recharging` is non-zero,
  decrement it and return. Any **other** ANTI_AIR building skips the reload
  gate altogether and is refused at the firing step below — its shots go
  through the `do_launch` path, not this one.
- Every 32 frames the seen-by mask is cleared; `hits_left() == 0` returns;
  `is_jammed` (a radar-jammed building) sets the misfire flag and returns;
  `get_garrison_arrows() == 0` returns.
- Without an explicit attack order (`build_masks & 4`), `find_target` —
  `Object::find_nearby_target(max(x_size, y_size) + 2 × max_range) × 0x60`
  (§12) — whenever it has no target, and every 32 frames (`(frame + o + 14)
  & 0x1f == 0`) when its current target is a unit that is **not moving** and
  is not a `role & 0x10000` spellcaster: the tower prefers a moving target
  and re-looks for one every two seconds. An invalid target → `find_target`.
- In range → `fire_ammo(target)` (§9; `ammo_per_att` ammo at random points
  in the footprint), the seen-by update, and **`recharging = type.recharge /
  arrows`**, `arrows = get_garrison_arrows()`.

`get_garrison_arrows`: `a = (attack() + 5) / 10`; zero → zero. `base =
base_arrows (+ maya_garrison_arrows for a Mayan, except an unassimilated
city)`. `g = count_inside(GARRISON_ARROWS)` — the sum over garrisoned
**captain, non-decoy, FOOT, non-siege** units of `attack / 10`, halved
(`(a+1)/2`) for a melee one; a militia-line unit, and a worker whose owner
has the militia tech, counts `(militia upgrade type's attack / 10 + 2) / 3`
— a third, rounded *(second reading; an earlier draft had militia counting
nothing)*. If `base == 0`: `g += a / 2`. `g /= a`;
`arrows = base + min(g, most_shots)`. A tower with three archers inside
reloads in a third of the time.

---

## 9. Projectiles — `Object::fire_ammo`, `Ammo::init`, `Ammo::inc_time`, `Ammo::do_damage`

### 9.1 Launch — `fire_ammo(o, who)`

A **unit** adds one `Ammo` **per figure** on this `UnitData` (`guy_mark`), at
the figure's position, `z + 100`, with the unit's facing, `gpiece =
type.fire_proj`'s graphic, `who/o = A`, `whom/ox = T`. A **building** adds
`type.ammo_per_att` ammo, each at a `Random::get(0, 0xffff) % (x_size × 0x60)`
by `% (y_size × 0x60)` offset from the building's centre (two draws per
ammo; none on a zero-width axis), `z + 250`, angle 0, graphic by
`get_shot()`. Nothing is launched at no target unless the order is
`ATTACK_GROUND`/`AIR_ATTACK_GROUND`.

`Ammo::init` then (for a normal shot at a unit or building target):

**Accuracy.** `acc = A.type.to_hit − A.type.attenuate × (attack_dist(A, T) /
0xc0)`, floored at 5, stored as `accuracy` — `attenuate` percent lost per
whole tile of `attack_dist`, with `attenuate` the column's absolute value.
*(Second reading: the first draft had `+ (d / 96) × attenuate`; the
compiler's `× −0x2aaaaaab >> 37` is a division by `−192`.) For a ground shot
the distance is the plain `vector_dist` to the point. `RANGE_INACCURACY` is
not read.

**Air targets.** Before anything else, a shot at an air unit (domain 2, not a
helicopter, not a missile) rolls to miss: a shooter without ANTI_AIR draws
`% 100` against the target's `fly_high` (or `fly_low`), and if that passes
draws again against its own; an ANTI_AIR shooter on the ground draws once
against its own `fly_high`/`fly_low`; an ANTI_AIR aircraft does not roll. A
miss sets the cosmetic flag — the shot flies and does nothing. Aircraft are
not modelled here; the draws are listed for the sequence (§9.5).

**Scatter radius `s`.** If A is a unit with `unit_flags & 0x400000` ('w'):
`s = 0`. Else for an **attack-ground** shot: `s = 0` if the order's `accuracy`
flag is set (a point at sea), else the land-unit formula below against the
plain distance. Else if T is a unit of **land** domain: `s = target_radius * 100
/ ((100 − acc) / 5 + acc)`, quartered when `acc > 100`, and the shot is
marked **rolling** (`flags |= 4`, and its landing z is raised by 0x4b) if it
is not a lofted piece (ammo flag 8). Else (T is a building, ship or
aircraft): `s = 0xc0` — one tile, fixed. A **missile** (MISSILE flag)
doubles `s`, a `NUCLEARMISSILE` zeroes it.

**Landing point.** `angle = find_angle(T − launch)`; `(ex, ey) = T.pos`; for
a building target, shot by a non-siege, non-tank unit, `ex, ey` are
projected `x_size × 0x30` back along `angle + 180°` (it aims at the near
face). Then `ex += Random::get(0, 0xffff) % s − s / 2`, `ey += Random::get(0,
0xffff) % s − s / 2` — **two draws when `s > 1`, none otherwise**; `ez = T.z`
(a plane's: its first guy's), floored at 0. A `CHAR_ATTACK2` foot archer
firing `unit_flags & 0x400000` gets a small per-figure offset (float, so
noted as the one cosmetic float on this path). Clamp to the map.

**Flight time `total_time`.** For a unit shooter that is not a missile:

- a non-siege: `total_time = (int)(sqrtf((ex − sx)² + (ey − sy)²) /
  (float)(type.proj_speed * unit_move_speed))` — **the one float on the
  combat path.** It is an IEEE single-precision square root of an integer,
  divided by an integer, truncated. It is emulated exactly in integers in
  the implementation (`combat::flight_time`): a correctly-rounded 24-bit
  square root and a correctly-rounded 24-bit quotient are both computable
  without a float, which is `docs/DECISIONS.md` entry 10's argument applied
  to a different operation. Open question 4 records the residual.
- a siege: `total_time = max_range() * 0xc0 / (type.proj_speed *
  unit_move_speed)` — integer, and **independent of the actual distance**.
- a building shooter: `sqrtf(dx² + dy²) / (proj_speed × unit_move_speed)`
  likewise, or `/ (unit_move_speed × 90)` for `get_shot() == 0` (arrows).
- a lofted piece (ammo flag 8) flies a spline; missiles fly a nuke spline
  (`Spline::calc_nuke_spline`, float) — not modelled.

**`total_time` of zero becomes 1** *(second reading)* — a shot always lands
at least a frame after launch, which with `cur_time` counted up before the
test means the same frame. Then **the lead** *(second reading; missed by the
first)*: a unit target whose order is a move (or an air order) has the
landing point pushed `sinx(·) × total_time` in x and `− cosx(·) × total_time`
in y — the decompiler dropped the operands; the natural reading, taken here,
is the target's heading and per-frame speed, so the shot is aimed where the
target will be. Not for ground shots, bombers or lofted pieces.
`splash_area = A.type.splash_area`; `flags |= 2` (live); `cur_time = 0`;
`num_guys = A.guy_mark` (unit) or 0.

### 9.2 Flight — `Ammo::inc_time` (every frame, all ammo, after the objects)

`cur_time += 1`; a target that is no longer active has its `hold_frames`
bumped. A live ammo with `cur_time < total_time` returns (a rolling one
recomputes its graphic position in floats — cosmetic). At `cur_time >=
total_time`: a **rolling** shot (flag 4) that has not yet missed (flag 8)
does `hit_target`, then `check_hit(GROUND)`, and if neither found anything
sets flag 8 and keeps going along its line, up to `3 × total_time`, until the
terrain is above it (float terrain heights; cosmetic to the outcome except
that a rolling miss can land on something further along — open question 5);
then `graphic_finish` and, unless cosmetic (flag 0x10), `Ammo::do_damage`.

### 9.3 Impact — `Ammo::do_damage`

If the target is no longer active it is forgotten (`whom = ox = −1`). Then:

**No splash** (`splash_area == 0`): `hit_target()`, and if that fails
`check_hit(domain of the original target, or GROUND)`. If there is now no
target, or the target is the shooter itself: a landing point is picked
`±20` around `(ex, ey)` with two more `Random::get` draws and the ground is
punctured (cosmetic); return. Else if the target is active: `Object::
do_damage(A, ox, whom, find_angle(launch → target), num_guys, index,
0x100, 0, 0)`.

**Splash**: (nuke and missile-shield branches aside) `hit_target()` /
`check_hit()` as above — then, for every object on every cell of the
**square** the spiral table `move_x/move_y` walks to `radius[k]`, `k =
splash_area / 4 + 1` capped at 10 (the `(2k+1)²` cells within Chebyshev
distance `k` of the landing cell — 3×3 for a splash of up to three tiles;
*second reading: the first draft had a circle of `splash_area / 4`*), walking
each cell's object chain through `down/down_who`, skipping the shooter's own
team and allies, **damaging the intended target with `splash
= 0` and everything else with `splash = 1`**: a unit that is active, on the
map, in the same air/ground class as the ammo's domain, not a missile:
`d = max(0, vector_dist(landing, unit) − 0xc0 − unit.type.guy_radius)`;
`count = 0x100 − (d << 8) / (splash_area × 0xc0)`; if `count > 0`:
`do_damage(A, unit, …, count, splash, 0)`. A building, for a non-air ammo:
`d = vector_dist(|Δx| − x_size × 0xc0, |Δy| − y_size × 0xc0)` — the x term is
explicit and the y term is in a register the decompiler lost, read by
symmetry; `count` likewise; `do_damage` if `count ≥ 0`. The `splash`
argument is what makes `get_damage` apply step 15 and lets the fringe do
nothing (step 26).

### 9.4 The hit test — `Ammo::hit_target` and `check_hit`

`hit_target`: the target must be active and on the map. For a **unit**:
`vector_dist(landing, T) ≤ T.type.target_size` — with the distance **halved
when `accuracy > 100`**. For a **building**: the landing point must be inside
the footprint, `|ex − T.x| ≤ x_size × 0x60` and `|ey − T.y| ≤ y_size × 0x60`.
Hit → 1; miss → forget the target, 0.

`check_hit(domain)`: `ObjectsData::find_unit` at the landing point, radius
`0x180` (two tiles), non-friendly only if the shooter is an aircraft, and
filtered to the ammo's domain class (AIR → only air; land/sea → not air);
a found unit whose `target_size` is less than the search's own distance
is rejected; failing a unit, `find_building_at` on the landing tile; failing
that, a miss sound by terrain and no target. **So a shot that misses its
mark can hit another unit within two tiles of where it lands, or the
building it lands on, and does full damage to it.** Not gaia's, though:
`find_unit`'s leader loop stops at eight (§12.1), so an arrow that comes
down on a sheep passes through it.

### 9.5 What a shot costs in random draws

In order: two per shot when a building is the shooter (the launch offset,
x then y, each only when the footprint axis exceeds one unit); one or two
`% 100` for a shot at an aircraft; two for the landing scatter (when `s >
1`); at landing, two more for where a no-target shot punctures the ground;
in `take_damage`, one `% 100` on the first wound of a fort, temple or town
and a second for the flock's size when the attacker is siege; and, outside
the shot, one per frame from `fight`'s retarget test (§8.2 step 0). Every
one is `Random::get(game_random, 0, 0xffff)` and the order above is the
order they are taken, so a sim that reproduces the sequence reproduces the
misses.

---

## 10. The game's random number generator — `Random`

One `ulong random_seed`. `Random::get(lo, hi)`: if `lo == hi` return `lo`
**without advancing**; if `lo > hi` swap; `seed = seed * 0x19660d +
0x3c6ef35f` (32-bit wrap); return `((seed & 0xffff) * (hi − lo) >> 16) + lo`.
`Random::reseed(x)` XOR-swaps the seed with `x` and returns the old one. The
per-game stream `game_random` is what every `Random::get(GameAccess::
game_random, …)` above reads; `GameAccess::rnd` is a second, unsynchronised
stream used for effects (the radar jam roll above is the one place this
mechanic reads it, and that roll is gated on a display flag). So `get(0,
0xffff)` is `(seed >> 16 & 0xffff) * 0xffff >> 16` — a value in
`0..=0xfffe` — and the `% 100`, `% s` that follow are ordinary truncations.
`combat::Rng` is this, exactly.

---

## 11. Death

`take_damage` calls the object's `die` virtual — `Unit::die` → `Object::die`.
`Object::die(dtype, gpiece, angle)`: `close()` (the object's removal:
unregistering from the world, the supply list, the army, the group), then,
for a type with range, `hold_frames = max(1, for every live ammo this
object fired: total_time − cur_time + nuke_effect[0x108] + 1)` — **the slot
is held until its last shot has landed**, so that a dead archer's arrow
still hits. A `UnitData` in the chain dies alone; `Unit::die_uber` (the
whole squad) is a separate call from the scripting and disband paths, not
from combat. Death of a non-captain figure does nothing to the others
except through `curr_uber_size` (§7.3); death of the captain is handled by
`close` (the next figure up becomes captain — `docs/ATTRITION.md` has the
chain). The `hits` share of the survivors is unchanged except the last one's
remainder.

---

## 12. Target selection

Read by a second reader (`Object::valid_target`, `ObjectData::valid_target_const`,
`Object::find_nearby_target`, `compare_target`, `check_target`, `poor_target`,
`Unit::find_melee_target`, `find_new_target`, `change_target`,
`target_opportunity`, `think_attack`, `Build::find_target`) and adjudicated
here against the decompile on the predicates the implementation carries. The
enum names come from the PDB's `OrderIndex` and `CombatStanceIndex`:
combat stances are `AGGRESSIVE 0, DEFENSIVE 1, STAND_GROUND 2, RAID 3, RAZE 4,
HOLD_FIRE 5`; `UnitData::get_combat_stance` is the unit's stance if its type's
stance type is combat, else `STAND_GROUND`.

### 12.1 Validity — `ObjectData::valid_target_const(o, who)`

All of, in order: the target exists, **its owner is a leader below eight**,
and it is not mine; **the owners are at war**
(`LeaderData::is_enemy`: my diplomacy toward them is war, or theirs toward
me); the target is active; it is **seen** by my owner (`is_seen(who, 0)`); a
unit target is on the map (not garrisoned); a submarine or a self-destructing
attacker (`unit_flags & 0x102000`) only targets sea units; **a melee attacker
(`max_range == 0`) cannot target a unit standing on a tile whose class bits
are `0x30`** (`TData.mask & 0x30 == 0x30` — the same two bits read `0x20`
for ocean in `check_hit`; `0x30` is **forest** — `docs/CITIES.md` §2.3 names the surface field); a submarine attacker on land domain cannot target sea; **air targets**
need a ranged attacker, and then a ladder of `fly_high`/`fly_low` against the
attacker's own `fly_high`/`fly_low` and ANTI_AIR flag (helicopters are
targetable by anything ranged but siege, tanks, missiles and bombers;
missiles never target or are targeted by missiles); and finally **a land-
domain or building attacker with ANTI_AIR never targets ground or sea**.
**The leader bound is the function's literal first line** —
`if (param_1 < 0 || param_2 < 0 || 7 < param_2) return 0` at `006472c0`,
*before* `LeaderData::is_enemy` is reached. `Leaders::list` is `Leader[10]`
and 8 and 9 are gaia's, so **an animal or a bird is nobody's target**, and
the diplomacy question is never asked about one. It could not be answered:
`LeaderData::diplos` is `int[8]`, so `is_enemy(8)` would read `treaties[0]`.
`ObjectsData::find_unit@0065ca80` states the same bound independently — its
per-leader loop steps `0x6eec` (one `Leader`) while the cursor is `<
0x37760`, exactly eight, and its by-cell branch guards `(int)leader < 8`
before `Search::valid_search`. See `docs/ANIM.md` §6.1.

`Object::valid_target` adds: a city that is capture-eligible is not attacked
(the capture path takes it) unless the attacker is a `VEHICLE` **and** `WAR_MACHINE` (both masks; `docs/CITIES.md` §7.1)
unit under a mandatory attack order on it, or a missile. `GroupData::
valid_target` is "any captain in the group passes". `LeaderData::get_target`
is the diplomatic target (the next in-play leader in start order), not a
combat one.

### 12.2 The search — `Object::find_nearby_target(max_dist, &who, add_order, cavarch, flags)`

Returns the best `o` or −1. An attacker with `ObjectData::attack() == 0`
finds nothing. `rings = 32` when `max_dist == 0`, else `ceil(max_dist /
0x300)` plus one for a building, one for ANTI_AIR, one in RAID, capped at 32.
The centre is my cell (a guard's guard point). The rings are `circle_x/
circle_y` — every `(dx, dy)` whose `max + min² / (2·max)` equals `r`, ring by
ring, `dx` then `dy` ascending — and on each cell every object on it (the
cell's `down/down_who` chain, **all players**). For each active object that
passes `valid_target` and the `flags` filters (units only / buildings only /
buildings with attack or wonders or military trainers), `check_target` must
pass (below; it also yields `dist = attack_dist`); the nearest so far is
cached in `near_o/near_who` (cleared afterwards if beyond `0xf00`); `dist >
max_dist` skips; the **range gate**: a unit in STAND_GROUND, or entrenched
without the Antipater bit, or an unpacked packer, considers anything
(`anything`); otherwise a unit that is not guarding (or whose guard target
has attack) is deemed `in_range` without testing, and everything else must be
`is_in_range` or be a unit, have attack, or be a wonder (then `in_range =
0` and it is still scored); **distance shaping** for units: a ranged type
with no minimum range and `dist + 0x180 < max_range × 0xc0` halves `dist`;
inside `min_range`: `dist = min_range×0xc0 + (max_range×0xc0 − dist)`; then
**`dist += (target.targeted + 8) × 0x30`** — every attacker already on it adds
a quarter tile; `value = compare_target(o, who, in_range, ai)`; **`score =
value / (dist / 0xc0 + 1)`**; the previous mandatory target halves; the
`flags` preferences halve the wrong class; a cavalry archer's second-weapon
search weights by bearing (×4 within 30°, ×2 within 60°, /10 beyond 90°,
skip beyond 135°); `score == 0 && value != 0 → 1`; strictly greater wins
(ties to the earlier — nearer ring, lower `dx`); and **after ten unit
candidates with a best in hand the scan stops**. The winner's `targeted`
goes up by one (cap 100). With `add_order` the attack order is added:
`QUEUE_FIRST` for a guard or a DEFENSIVE stance, `QUEUE_NEW` otherwise, and
`mandatory` only for AI-controlled siege on a city.

`Object::check_target(o, who, duty, &dist, guarding, use_poor, cavarch)`:
`dist = attack_dist`; a unit not on duty in DEFENSIVE with an order, or a
target in another region, must be `is_in_range`; a guarding unit rejects
beyond `unit_guard_respond_range × respond × 0x60` (`respond` 2, or 3
against a non-worker under AI control) of the guard point; a building target
must be buildable outside a city (`build_flags & 0x10`) or stand on an owned
cell; and, with `use_poor`, not `poor_target`. **`poor_target`** (unit vs
unit): an airborne figure unless I am ANTI_AIR within range; else a target
on a move order that is faster than me, heading away (the `flanking` test
says its back is to me), and more than a tile off (a cavalry archer: more
than `max_range` tiles).

### 12.3 Ranking — `Object::compare_target(o, who, in_range, ai)`

`v = 4 × Type::sum_rules_cost(T)` (a wonder: `/25`). An active building:
`ai` → `×10`; not raiding and not human: a **city** `×5` (`ai`: 100), else
a **defensive** building (the Tower/Fort/Airbase lines or `most_shots != 0`)
not ANTI_AIR `×4` (`ai` `×40`), else a military trainer `×3` (a silo with
something queued or inside `×15`), else a training building `×2`. A target
with non-zero attack and `hits_left`, not ANTI_AIR: `v = v × T.attack × 100 /
T.hits_left` (`×10` more for a building) — **wounded, hard-hitting targets
first** — or `/20` when a raider looks at a building. A building attacker:
its current target `×2` (with fewer than two arrows) or `/2`; a damaged
target `×3/2`; a moving one `/4`; a supply wagon `×5000`. Then `dmg =
get_damage(o, who, angle 0, …)`: `v ×= dmg` (AI: `v /= dmg`, or 0). Building
targets (not raiding): armed and not human → `+1,000,000` with the SIEGE mask
else `×5`; siege vs armed `+100,000`. Unit targets: combat-role `×20`;
spellcasters casting at me `+10,000,000`, spies `+6,000,000`; a detected
hidden unit `/4`; raid weights (citizens and caravans `+9,000,000`, else
`/10`; the stealth-ship weights); not raiding: combat-role `+1,000,000`, a
supply wagon `+4,000,000` for a building attacker, else `+100,000`; land siege
vs a non-sea, non-siege unit `/10,000`; `v /= (T.full + 1)`. A city at zero
hits `99,999`; negative `9,999,999`; out of range and not raiding `/5`;
`v = ceil(v / 100)`, above 100,000 compressed to `100,000 + (v − 100,000) /
5`, floor 15; a detected hidden unit 1; an air target for a non-ANTI_AIR
attacker 2.

The implementation carries the skeleton of this — cost, the building class
multipliers, the `attack × 100 / hits_left` preference, `× dmg`, the combat-
role and supply bonuses, `/(full+1)`, the ceil/compress/floor — over what the
simulation has, and leaves the raid, spell, stealth and AI branches as
inputs. See `combat::compare_target`.

### 12.4 Switching and opportunity

- **Idle**: `Unit::think` runs when `(o + frame) & 0xf == 0` once the unit has
  been idle more than two frames; on frames where `(o + frame) & 0x1f != 0`
  it only checks the **nearest cache** (`near_o` active, in range, stance
  below HOLD_FIRE → `add_attack_order(QUEUE_NEW)`); every 32nd frame and the
  first idle frame a combat unit (`attack != 0 && role & 0x10000`) runs
  `think_attack` → `find_melee_target(−1)`: radius `(max_range + 1) × 0xc0`
  (`+ 0x180` in AGGRESSIVE, `0x120` for melee), at least `unit_respond_range
  × 0xc0` (`× 0x180` under AI control); DEFENSIVE: `max(max_range × 0xc0,
  unit_defensive_respond_range × 0xc0)`; a packed packer with auto-stance
  unpacks instead after 3 (machine guns) / 7 (human) / 21 (AI) idle frames;
  HOLD_FIRE finds nothing.
- **Walking to attack**: every 16 frames (`(o + frame) & 0xf == 0`) a
  non-mandatory attack re-searches and switches to a nearer in-range unit
  that is not a poor target.
- **Fighting**: with probability 1/5 per frame (`Random::get % 5 == 0` — one
  more draw from `game_random`, taken only when the target is a unit and
  the order has flag `0x10` clear) or when the order asks, `find_new_target`;
  a different answer rewrites the target. A DEFENSIVE unit beyond `max(
  unit_defensive_respond_range, max_range) × 0xc0` of its post goes home.
- **Hit** (`do_damage` step 2 → the victim's captain's `target_opportunity`):
  ignored unless at war; a group member forwards to the group (which every
  15 frames at most tells its idle combat captains to `find_melee_target`
  within `min(dist + 0xc0, unit_respond_range × 0x240)`); forwards to the
  captain; HOLD_FIRE ignores; a unit already attacking ignores it unless its
  target is invalid, not mandatory, not combat-role, the attacker is a unit
  in range and its own target is not, and it is not raiding/razing — then
  it drops the order and retaliates; spies bribe, commandos sabotage;
  **non-combatants flee** (`find_nearby_spot` at `0x600`, `0x180`/`0x300` for
  heroes and supply); a combat unit off duty with an order ignores; else
  `add_attack_order` — `QUEUE_NEW` if DEFENSIVE and idle, `QUEUE_FIRST`
  otherwise.
- **Stances**: AGGRESSIVE chases at respond range; DEFENSIVE ties the
  attack to a return point; STAND_GROUND ignores range in the search and
  never chases; RAID weights civilians and never retaliates; RAZE prefers
  buildings and never retaliates; HOLD_FIRE does nothing — **except under
  a mandatory order**, which skips every stance test in `Unit::do_attack`:
  `005f1b80:225`-`244` reads the order's `+0x1c` and jumps straight to
  `LAB_005f2224`, the unconditional `fight(…)`, so the stance arms all sit
  under `mandatory == 0`. §17's probe is what found it — it sets
  `action_stance(5)` on six units and then gives them a mandatory attack
  order, and run19's block 8187 has all six chasing (item 328).
- **Buildings**: §8.6.


---

## 13. Range and distance

### 13.1 `ObjectData::attack_dist(o, who, x, y)`

The distance every range test uses, read from the disassembly at `0x6488f0`
because the decompiler dropped its register operands:

```
snap(v) = div_3_table[v >> 4] * 0x30            (the quarter-tile grid)
dx = |snap(x) − snap(T.x)|;  dy = |snap(y) − snap(T.y)|
a plane that is not a missile:        vector_dist(dx, dy)
target extent:  building → ex = T.x_size * 0x60, ey = T.y_size * 0x60
                unit     → ex = ey = T.block_radius + 0x18
dx = max(0, dx − ex);  dy = max(0, dy − ey)
own extent:     building → sx = x_size * 0x60, sy = y_size * 0x60
                unit     → sx = sy = block_radius + 0x18
dx = max(0, dx − sx);  dy = max(0, dy − sy)
return vector_dist(dx, dy)
```

— edge to edge, per axis, then the integer hypotenuse (`docs/MOVEMENT.md`'s
`vector_dist`). `x_size` is in tiles, so `x_size × 0x60` is half the
footprint; `block_radius` is the type's `BLOCK_RADIUS × UNIT_BLOCK_RADIUS`.

### 13.2 `ObjectData::is_in_range(o, who, x, y, …, melee_bonus, &dist)`

The target must be active, a unit target on the map, and **the attacker's
own** tile not of class `0x30` (`& 0x30 == 0x30`; *second reading* — the
first draft had the target's tile, and called it fog). `d = attack_dist(o, who, snap(x) + 0x18, snap(y) +
0x18)`. **Melee** (`type.max_range == 0`): `d ≤ 0x66` (102 units — a little
over half a tile), or `d ≤ 0xf6` for the `HOPLITES` line. **Ranged**: `d <
min_range() × 0xc0 − 6` is out — unless adding both units' `big_radius`
brings it in; `d > max_range() × 0xc0 + 6` is out (`+ 0x90` more with
`melee_bonus`); `min_range()`/`max_range()` are the virtuals of §4.4. The
position-only overloads (`(x, y)`) use `vector_dist` less one `big_radius`
(unit) or `(x_size + y_size) × 0x30` (building), the same `0x66` and
`[min×0xc0 − 6, max×0xc0 + 6]`, and call an ungarrisoned missile always in
range. `GroupData::get_max_range` is the largest `max_range()` of the armed
members, which `Unit::do_attack` uses to stand an unarmed group member off at
`(group_range + 2) × 0xc0`.


---

## 14. Open questions

1. ~~**The flank direction.**~~ **Settled at the premise, 2026-08-20.** The
   arithmetic was never in doubt; what it rested on was whether a unit's
   `angle` is the direction it *faces*. It is. A logged run
   (`docs/ORACLE.md`; `UNITS=3` under `[End Frame]`) was read frame by frame
   for a walking squad, comparing the logged `angle` against `find_angle` of
   its own per-frame position delta (`tools/gamelog/heading.py`):

   | frames | step | `angle` | heading | difference |
   | --- | --- | --- | --- | --- |
   | 11115–11129 | 25.5 | 133.90 → 133.45 | 135.00 | −1.1 … −1.6 |
   | 11130–11135 | 28.3 | −43.20 → −42.85 | −45.00 | +1.8 … +2.2 |

   The heading quantises to the diagonal because the step does; `angle` sits
   within about two degrees of it and drifts smoothly, which is a unit walking
   forwards and turning as it goes. So `angle` is the facing, and with §8.4's
   `angle` argument — the attacker's facing toward the target — `d` near zero
   means the target's facing points **along** the direction the attack
   travels, i.e. away from the attacker. **Level 1 is the rear**, and at the
   shipped `FLANK_BONUS` of 50 the rear is ×1.5 and the side ×2.0.

   ~~That the side bonus exceeds the rear one still reads oddly, so the
   confirming measurement is worth having and is **not** done: the damage
   ratio itself, one attacker on one target, from behind and from the front.~~
   **Done, run17 (2026-08-24), by the damage itself** — the last section of
   this document: hoplite on hoplite is 48 sixteenths from the front, **85
   from the rear (`d` near 0°) and 117 from the side (`d` near ±90°)**, the
   three values the formula gives for levels 0, 1 and 2 and no other value
   in the run; the side *is* the larger bonus. The original text of the
   plan follows.
   The instrumentation exists — `tools/gamelog/hits.py` prints every damage
   increment in sixteenths with the victim's angle, the attacker, and the
   computed `d` — and it was run, but on a melee with a tower and six squads
   in it, where increments could not be attributed to a single attacker
   (`damage_o`/`damage_who` name a last damager that does not update per hit,
   so a tower's arrows arrive labelled with a hoplite). The setup that will
   work: the target already engaged with an unarmed unit of the measurer's, so
   it faces *that*, and exactly one attacker striking it from a chosen
   bearing, moved between trials with `cheat move <o> cursor` — the target's
   facing is then fixed by its own fight and `d` is set by where the attacker
   stands.

   *What the question was, before the check.* §6 step 19 is arithmetic;
   whether level 1 (`d` within ±45° of zero) is "attacked from behind"
   depended on whether a unit's `angle` is the direction it faces and
   `find_angle(T − A)` is the direction A→T, both as `docs/MOVEMENT.md` reads
   them — and `rules.xml`'s own gloss, "per level of flank (max bonus is twice
   this number)", is silent on which level is which.

2. **Whose mask the cavalry/vehicle flank reduction reads** — the attacker's
   is taken; the register was lost.
3. **`unit_masks & 0x10` and `0x400000`** (the ×2 in `do_damage` step 2 and
   `get_damage` step 14) are read but not named here; neither is set by
   anything this reading covered.
4. **The flight-time float.** `(int)(sqrtf(n) / (float)d)` is emulated by
   exact integer rounding of each IEEE operation. If the original's
   `sqrtf` is not the correctly-rounded one (x87 `fsqrt` is; an SSE
   `sqrtss` is), the emulation can differ by one frame when `√n / d` is
   within one float ulp of an integer — a measure-zero event for a game,
   and a recorded-game diff would show it. The implementation notes where.
5. **A rolling miss.** A cannonball that misses keeps rolling and can land on
   something further along. Where it stops depends on float terrain heights;
   the implementation stops it at the scatter point. Cosmetic for the
   intended target, not for whoever was standing behind it.
6. **The building-splash filter.** `Search::valid_search`/`valid_filter` in
   §7.1 step 9 were not read below the call; "hostile and on the map" is the
   assumption.
7. **Two sounds' worth of `Random::get`.** The `puncture_ground` landing
   jitter and the flock roll are taken only on the paths described; if a
   later reader finds another `Random::get` between launch and impact, the
   draw order in §9.5 is what has to change.
8. **`fire_proj` after the graphics loader.** `UnitType::init` sets it to 1
   for any attacking type and `GraphicEvents::init_unit_events` then
   rewrites it — to the first ammo piece, to a named ammo piece, or to
   **zero** for graphics with certain elements — and `fight` strikes in melee
   only when it is non-zero. Every melee type plainly does strike, so the
   zeroing must not reach them; which graphics it does reach was not read.
   The simulation strikes whenever `attack != 0`.
9. ~~**The `Flag_` rows of `balance.xml`.** Whether internal string 17 loads as
   a space (the rows match and the file's `Flag_Y_OBJMASK_HEAVY_CAV="95"`
   and friends apply) or as empty (they never match). `rondata` generates
   both and the `RULES=1` dump picks; until then the implementation takes
   the space, because that is what the hash says was written.~~
   **Settled 2026-08-20 against the program's own loaded table: the `Flag_`
   rows never match. They are dead data.** The dump is `BEGIN COMBATTABLE`
   in a `DUMP_ALL=1` start-of-game log (`docs/ORACLE.md`) — 243,049 values,
   **493 × 493**, which is `TypeIndex − 50` over the 364 units and 129
   buildings, not the 399 balance categories (`types.txt` declares
   `short[493][493] final_balance_table`; `Balance::fill_tables` loops
   `BASE_UNITTYPES`…`0x21f`, and the 399×399 XML table is the `malloc`
   scratch inside it). Orientation is `[attacker][target]`.
   Over the 364×364 unit block the two variants differ in 22,629 cells; on
   those the dump agrees with *never-match* 19,387 times and with *match*
   **zero** times. Direct probe: `Flag_Y_OBJMASK_HEAVY_CAV="115"` would give
   Cataphract-versus-War-Elephant 115, and the dump holds 100. So the engine
   builds the row name with a separator that is not the file's, all 64
   lookups miss, and every one of those entries keeps the 100 default.
   `rondata::balance::tail_names()` must stop emitting the matching names.
   **This also left a live defect** — see §15.

---

## Second reading (2026-08-20) — landed

Two blind readers (`docs/audit/2026-08-20-combat.md`): one over `get_damage`,
`do_damage`, `take_damage`, the stat accessors and the table's composition;
one over `fight`, `recharge`, `fire_ammo`, `Ammo::*`, `Build::do_attack` and
the range tests. **Doubly confirmed:** every step of the damage formula and
its order, the `(base + 5) / 10 − armour` rounding, the `0x101` floor and the
two `>> 4`s of the scaling, the per-figure share and the captain's remainder,
the decoy overflow, the city clamp, the construction-progress loss, the
RNG's constants and mapping, the recharge rule and its literals, `attack_dist`
from the disassembly, the melee `0x66`/`0xf6`, the ranged bounds and the big-
radius rescue, the building reload `recharge / arrows` and the arrows formula,
the hit tests, `check_hit`'s two-tile search across all players, the splash
falloff, the flank sectors, and the combat table's name space and product
loop. **Overturned and landed above:** the accuracy formula (§9.1), the
retarget draw's condition (§8.2 step 0), siege fire at units (§8.2 step 1),
the splash square (§9.3), the anti-air building gate (§8.6), the flock roll's
type test (§7.2), the militia arrows (§8.6), whose tile `is_in_range` reads
(§13.2). **Added:** the `total_time` floor and the lead (§9.1), the
air-target miss rolls (§9.1), `work`'s early return (§8.1). The
implementation was corrected to match and `a_shot_flies…`, the overkill and
the new `siege_fires_at_the_ground…` tests pin the changes.

---

## 15. The table, checked against the original's own (2026-08-20)

A `DUMP_ALL=1` start-of-game log contains `BEGIN COMBATTABLE`: the engine's
`final_balance_table` as it stands after `Balance::fill_tables` has run, which
is the *composed* result of the XML table and the hardcoded `type_damage`
half. That is a direct oracle for §5 — not for one formula, for all 243,049
entries at once — and it is the first time this document has had one.

Extracted and compared against `crates/sim/src/balance.rs` over the 364×364
unit block (132,496 cells):

- **The `Flag_` question is closed** (§14.9): the flag rows are dead, and the
  variant that ignores them is right in 19,387 of the 22,629 cells that
  distinguish the two.
- **14,577 cells still disagree**, and they are the hardcoded half, not the
  file half. 11,917 of them (82%) are pairs whose two units sit in different
  ages, i.e. the age-bonus step.

The three sharpest leads, in the order worth chasing:

1. **`Companion` (type 181): 352 of 364 columns wrong**, almost all
   `dump=100` against `ours=105`. A whole `type_damage` factor is being
   applied to Companion that the original does not apply — a lineage
   predicate, of exactly the kind the second reading found wrong elsewhere.
2. **The elephants (211–214): ~890 cells**, each one multiplication step
   below ours (`115 → 100`, `80 → 70`). A step that should not run, or runs
   against a different mask.
3. **The patriots (304, 306, 307 — The Monarch, The Comrade, The CEO):
   ~490 cells.** These have no `balance.xml` row at all, so their errors are
   purely hardcoded-half.

None of this was reachable before: the second reading confirmed the *formula*
twice over, and the formula is right — what is wrong is which units it is
applied to, which is precisely what a table dump can see and a reading cannot.
~~The building half of the dump (indices 364–492) is still unchecked; `rondata`
builds no building side of the table at all.~~ Checked and equal, §15.3.

**Not established.** Whether the 14,577 are three bugs or thirty. The count is
a ceiling on the damage, not a diagnosis, and each lead above needs the same
treatment as a mechanic: read the predicate, fix the document, then the code.

### 15.1 How to pick this up

Written down because this is the next mechanic and it wants a careful reader
rather than a fast one — the arithmetic is settled and doubly confirmed, so
every remaining error is a *predicate*, which is the class the audits kept
finding and the class a hurried read produces confidently.

What is already in hand, and needs no game and no clicking:

- **The oracle.** `Logs\gamelog-run3-fulldump-types.txt` in the bottle (152 MB,
  outside the repo per `CLAUDE.md`) contains `BEGIN COMBATTABLE` at line
  190655 — 493 × 493 shorts, row-major, `[attacker][target]`, indexed by
  `TypeIndex − 50`. Extract with `tools/gamelog/gl.py`. The 1,820 `UNITTYPE`
  blocks from line 6067 give the index ↔ type mapping (`type` is each block's
  first field, 50…413 repeated five times), which is `units.xml` order and so
  is `rondata`'s unit id order.
- **The comparison already done**: over the 364 × 364 unit block, the
  flags-never-match variant leaves 14,577 mismatches, 82 % of them across an
  age boundary.

The order worth working in:

1. **Land the `Flag_` correction first** (§14.9) — `rondata::balance::
   tail_names()` still emits the matching names, and the dump says all 64 of
   those lookups miss. That is a known-answer change and it shrinks the noise.
2. **Companion (type 181), 352 columns wrong, `dump=100` vs `ours=105`.** One
   factor is being applied that the original does not. Start at
   `type_damage`'s lineage predicates and ask *which* test Companion passes in
   our reading and fails in the original's — the cities and tech audits both
   found errors of exactly this shape ("read the loader too", "grep the
   writers of what you call frozen").
3. **The elephants (211–214)** are a whole multiplication step low
   (`115 → 100`, `80 → 70`), which is a different failure from Companion's:
   a step that should not run at all, or runs against another mask.
4. **The patriots (304, 306, 307)** have no `balance.xml` row, so their cells
   are purely the hardcoded half — the cleanest isolate of `type_damage` in
   the whole table, and probably the best place to *start* reading rather than
   the last.

~~The building half of the dump (indices 364–492) is unchecked and `rondata`
builds no building side at all; that is additional scope, not a bug.~~ Done,
§15.3.

The reason this is cheap despite being a reading job: **every hypothesis is
falsifiable in one diff run.** That is not true of the mechanics documented
from the decompile alone, and it is what makes this the highest-value item in
the queue.

### 15.2 Closed (2026-08-20): zero cells differ

`rondata <install> --types <dump>` (`crates/rondata/src/typesdump.rs`,
`types_report` in the binary) reads the dump's 1,820 `UNITTYPE` blocks and
its `COMBATTABLE`, and checks two things: the **inputs** — every unit's
`obj_masks`, `age`, `is_siege`, `is_caravan` and `domain` against the
loader's `Kind` — and the **output**, the 364 × 364 unit block against the
table the loader builds. Checking the inputs first is what made this an hour
rather than a session: two of the three errors were visible there before a
single cell was looked at.

| cause | where | what the dump showed |
| --- | --- | --- |
| **The age was one too low** for 306 of 364 units | `load.rs` `age_of_tree` returned an age tech's index; `get_age_slow` returns its `AGE` column **+ 1** (§5.2) | `age` Militia 1 vs ours 0, Minuteman 4 vs 3 … — and 12,705 of the cell mismatches were across an age boundary |
| **Four lineage roots were wrong**, keyed by display name | `ECOMPANION 0xe8` is *Royal* Companion, not Companion; `BOMBARDSHIP 0x15a` is the Bomb Vessel; `CAMELRANGE2 0xbf` is the Camel Archer; `HALBERDIERS 0x95` is Scutari. Now rooted by `TypeIndex − 0x32` (`LINE_ROOTS`), with `is_slow`'s strict rule — self, or a direct graft of a root that is not a unique (`y`) unit | Companion's 352 wrong columns (§15 lead 1), all `100` vs ours `105` |
| **`0x14000` is O\|Q, not M\|O** (steps 14–15) | `sim::balance::type_damage` | Citizen → Slingers `200` vs ours `100`; Citizen → General `100` vs ours `200` |

With the three landed, **0 of 132,496 cells differ**, and all five input
checks pass for all 364 units. The `Flag_` correction (§14.9) landed with
them. Two things learned about the dump on the way, recorded in
`typesdump.rs`: the mask words are logged as signed `int`s, so a mask with
bit 31 (`6`, anti-air) prints negative; and the `age` field is the stored
`+0x278`, `−1` for the twelve gaia animals, which `get_age` resolves to 0.

~~**What this does not establish.** The building half of the table (indices
364–492, and every unit-versus-building cell) — `rondata` builds no building
`Kind` and the check covers the unit block only.~~ Done the same day, §15.3.

`rondata --types` is now the regression guard for §5: any later change to
`type_damage`, the tree loader's ages, or the lineage roots that moves a
cell fails the check. Since 2026-08-25 the same comparison
(`rondata::typesdump::compare`) also runs as an install-gated `cargo test`
against run3's dump, so the guard cannot be skipped by omitting the flag.

### 15.3 The building half, and the whole table (2026-08-20)

The same session took the other three quadrants. `sim::combat::Table` is now
over two families — `TypeRef::Unit(id)` then `TypeRef::Build(id)`, units
first as `final_balance_table` lays `BASE_UNITTYPES..END_BUILDTYPES` out —
with `pct_of` for any pair, `pct` the unit-versus-unit shorthand, and
`grown` so the sim's type space can keep growing on either axis.
`fight.rs` looks the table up for **every** pair now; before this a building
on either side was a flat 100. `rondata::load` builds a `Kind` per building:
the `return_pack` line ladder (FORTS, CITIES, OBSPOST, TOWERS — §5.1) from
`is(x, 0)` rooted by `TypeIndex` (`VILLAGE 0x19e`, `TOWER 0x1b7`, `FORTX
0x1bb`, `LOOKOUT 0x209`, and `AIRBASE 0x1bf` for step 30), the BUILDINGS
object, `get_age`, `OBJ_MASKS`, the wonder range `0x20e..0x21e` (step 26),
no siege and no caravan flag. `Loaded::build_kinds` exposes them and
`rondata --types` checks them — `obj_masks`, `age`, `domain` per building
against the dump's `BUILDTYPE` blocks — and then all four quadrants.

The first run came to **4 of 243,049** cells: Oil Platform, Dock, Anchorage
and Shipyard against the Airbase, `33`/`83` against ours `100`/`250` — step
30's `a.domain != Land` third, on buildings the program loads with
`domain 1`. `buildingrules.xml` has no `DOMAIN` column; the rule is
`BuildType::set_domain@00633390`: **`BUILD_FLAGS b` ("can be built on sea
squares", bit 1) makes a building Sea, or Air when `a` (bit 0) is also set,
else Land.** Landed in the building `Kind` and in the building
`combat::Profile`'s `domain`, which had been Land for every building.

With it, **0 of 243,049 cells differ** — unit→unit, unit→building,
building→unit and building→building alike — and every per-type input check
passes for all 364 units and 129 buildings. The two type dumps and the table
are one oracle for §5 entire; `rondata --types` guards all of it.

**A second, cheaper source for the same oracle (2026-08-24):** every
recorded game embeds the loaded rules, including this composed 493 × 493
(`docs/RECGAME.md` §4.2). `rondata --recgame` runs the same comparison, and
on a 2017-build recording against this 2024-build install **every cell is
equal** — the table survived seven years of EE patches unchanged, and the
check no longer needs the `DUMP_ALL=1` run that hangs the game.

**What this does not establish.** Nothing about §5 is open. What the table
*feeds* — the nation, wonder and patriot modifiers, the terrain and height
inputs, aircraft and missiles — is the same list as before (§14); this
section settled the multiplier, not its consumers.

---

## 16. Behavioural check (run17, 2026-08-24) — the formula's numbers, hit for hit

Until this run no traced game had entered `Unit::fight`, `Object::take_damage`,
`find_new_target` or `Object::fire_ammo` (`docs/ORACLE.md`, the blind list).
Run17 is the combat run, staged from a file through the cheat channel
(`docs/ORACLE.md`, "The cheat channel"; the file is in "The combat run"
there): `!ai off`, then in an unowned mid-map arena a who-1 Supply Wagon
with a who-0 hoplite squad placed four tiles off at three bearings, a
hoplite squad against a hoplite squad, slingers against a wagon, and a
tower against a wagon — 2,600 frames at `UNITS=3 AMMO=3`, same lobby and
seed as run12–16. **The numbers were computed before the log was read**
(`rondata`'s `run17_s_hits_are_the_formula_s`: §6 `get_damage` then §7.1's
`scale` on the loaded profiles) and `tools/gamelog/hits.py` read every
damage rise with the victim's facing and the attacker's bearing.

| attacker → target | predicted sixteenths per figure-strike | observed |
| --- | --- | --- |
| Hoplites → Supply Wagon, any bearing | **122** — the wagon's `OBJ_MASK` is `VC`, CIVILIAN, so step 19 never runs | 122 at `d` = +5°, −156°, −90°, −45°, −79°, −41°, +45°; 244 on the frames two figures struck |
| Hoplites → Hoplites, front / rear / side | **48 / 85 / 117** (levels 0, 1, 2: ×1, ×1.5, ×2 before the `/10` and armour) | exactly 48, 85 and 117 and nothing else, 85 at `d` −3°…−39° and 117 at `d` −61°…−114° |
| Slingers → Hoplites, front / rear / side | **32 / 58 / 85** | 32 at `d` ≈ 170°, 58 at `d` 24°…66°, 85 |
| Slingers → Scout; → Supply Wagon | **53; 53** | 53 (f2022, `d` −6°); 53 and 106 |
| the Tower → Supply Wagon | not computed (a building's `do_attack`, §8.6) | **544** per arrow, `damage_o` = the tower's 2007 |

Three things the run adds to the reading:

- **The sector assignment is confirmed by damage, not only by the facing
  premise** (§14.1): level 1 is the rear and level 2 the side. The handful
  of hits whose `d` sits in the wrong sector for their size (a 48 at
  `d` −8°, a 58 at `d` 66°) are attribution noise: `hits.py` takes the
  attacker's position from `damage_o`, the squad's captain, while the
  striking figure may be a tile away; the *sizes* are exact.
- **A shot's draws, by site.** A unit's attack rolls once at
  `Unit::fight+0x9b0` (75 over the ranged window; 14 more at `+0x824`), and
  its projectile draws twice at launch — `Ammo::init+0xcd9` and `+0xd0b`
  under `Objects::add_ammo` < `GraphicEvents::execute_game_events`, i.e. the
  launch is an *event* executed after the unit's own step — and nothing at
  impact. A **building's** arrow draws twice in `Object::fire_ammo` itself
  (`+0x3d5`, `+0x406` under `Build::do_attack` < `Build::process`) and then
  the same two `Ammo::init` draws directly from `fire_ammo+0x429`, not
  through the event. §9.5's accounting should be re-read against these
  sites; the order within a frame is in `rontrace-run17.log`.
- **A Supply Wagon flees at first sight of an enemy** — the three-bearing
  trial got one hit (the hoplite spawned 3.5 tiles away and struck within
  eleven frames) and then a chase the wagon won at equal speed, turning to
  face east as it ran; the second and third wagons ran before contact. The
  wagon's own `think` does this with the AI off, and it is **not**
  `MoveOrder::is_fleeing` (still blind after the run). Hoplites do not
  flee; they retaliate.

What it does not establish: the tower's 544 (the building arrow chain),
the cavalry/vehicle flank reduction (no mounted unit was placed — the run
had no `Cataphract`-class name in Ancient), splash, the overkill window
(no two squads shared a target long enough to read it), the projectile's
scatter and flight-time arithmetic against the logged `AMMO` records, and
the height and rocky terms (flat arena). Each is a line in the next file.

---

## 17. `Unit::find_attack_pos` — where to stand to attack (item 324, 2026-09-17)

Great Lakes' long word parted at frame **8186** on this function — six
draws against fifty-four, forty-six of them at
`Unit::find_attack_pos@00601280+0xea9`. ~~Nothing in `crates/sim` spends
it yet~~ — **item 328 implemented it** (`crates/sim/src/attack_pos.rs`);
the frame agrees draw for draw with all six of §17.5's destinations
exact, and the word parts at 8187. Writing the code found **four** errors
below that the reading had not, each struck in place with its successor
named (the story is in `docs/journal/2026-09-17-item-328.md`).

### 17.1 It is reached twice in 24,000 frames, and both times it is a raid

`tools/trace/report.py rontrace-run53.log when Unit::find_attack_pos 1`
answers with two frames: **8186** (48 draws) and **17656** (14), both
`Army::find_target` frames. In a traced game the function is one
mechanic, `Army::find_target@006f69b0:1120`–`1199`, **the two-unit
probe**:

1. `ArmyData::get_unit(army, 0)` and `get_unit(army, count − 1)` go into
   a stack-local `Group`; `Groups::push_group(who, who, &g, 1)` returns
   the slot the orders are given to. **Six** units take them, not two:
   `Group::add`'s subordinate recursion (`docs/GROUPS.md` §4.1) brings
   each captain's two followers, and run19's block 8186 has group 64
   holding **fifteen** units in five squads of three under the captains
   `27`, `31`, `34`, `37` and `40`.
2. When the *target's* leader has `LeaderData +0x944` zero — `combat`, the
   human with no soldiers, read in `docs/ARMY.md` §12 — then
   `ObjectsData::find_building(target.x,
   target.y, SEARCH_FRIENDLY, target_who, −1, 0x200, FILTER_TYPE, 0x1a1,
   0)` — **type 417 is `FARM`**. ~~`0x200` is the radius~~: the radius is
   the **−1** before it, and a negative radius takes
   `find_building@0065d260:36`'s **exhaustive** arm — every object of
   every leader `valid_search` admits, no distance bound, distance in
   **tiles** (`65d2f4`), `<=` on both tests so a tie goes to the **last**
   in walk order. `0x200` is `param_6 & 0x200`, the **region** filter.
   Read as a radius the probe finds no farm at all: `0/2004` is 1,165
   position units away and `0x200` is 512. Then
   `Group::action_stance(g, 5)`,
   `Group::action_attack(g, farm, target_who, mandatory = 1, QUEUE_NEW, 0)`
   and `Group::action_move_to(g, leader.x, leader.y, QUEUE_LAST, …,
   MOVE_TO, …)`. Otherwise a single `action_move_to(…, ATTACK_TO, …)`.
   The point that move goes back to is the **local pair's leader**, from
   `GroupData::find_leader` at `:1152` and read back at `:1191`, not
   `get_unit(0)`; `docs/ARMY.md` §12 said the first unit's position and is
   amended to match.
3. `Group::action_attack@00712490:215` calls `find_attack_pos` **once**,
   on the group's leader, and only when `is_in_range` says the leader
   cannot already shoot — the `+0x41a` family, two draws at 8186 and
   four at 17656.
4. Every member that takes the order then runs its **own** call from
   `Unit::fight@005fd4d0+0xcb4`'s out-of-range arm — the `+0x2d` family,
   46 draws at 8186 and 10 at 17656. `+0x2d` is the *seven*-argument
   overload `@00602e60`, a thunk that fills the last two arguments with
   the caller's own encrypted position.

~~`crates/sim/src/army.rs` files step 1 as "a group order (seam); nothing
changes."~~ That comment was false — the probe is the whole of frame 8186
— and item 328 replaced it with `Sim::find_target_probe`.

### 17.2 The ring

The candidate positions are a walk around the target's footprint, eight
**sides** numbered 1..8 anticlockwise from the north-west corner: the odd
numbers are the corners and the even ones the edge midpoints. Two arms walk
the ring from the same starting side in opposite directions, one step each
per iteration, arm 0 decrementing the side and arm 1 incrementing it.

The starting side is an octant of the vector from the asking point to the
target. With `ex = |from.x − t.x| − t.x_size × 0x60`, `ey = |from.y − t.y| −
t.y_size × 0x60` and `diag = ex ≥ 1 && ey ≥ 1`:

| | not `diag` | `diag` |
|---|---|---|
| `ey < ex` (x-dominant) | `from.x ≤ t.x` → 8 else 4 | `t.x < from.x` → (`t.y < from.y` ? 5 : 3) else (`t.y < from.y` ? 7 : 1) |
| otherwise | `t.y < from.y` → 6 else 2 | `t.y < from.y` → (`from.x ≤ t.x` ? 7 : 5) else (`t.x < from.x` ? 3 : 1) |

An **edge** side's base point is the face pushed out by the stand-off
`local_18`: side 2 is `(t.x, t.y − t.y_size × 0x60 − d)`, side 4
`(t.x + t.x_size × 0x60 + d, t.y)`, side 6 `(t.x, t.y + t.y_size × 0x60 +
d)`, side 8 `(t.x − t.x_size × 0x60 − d, t.y)`, and an arm steps along it by
a linear stride until the overshoot past the corner exceeds the footprint.
A **corner** side's base point is the corner itself and the arm sweeps an
arc: the candidate is `(x + sin(a) × d, y − cos(a) × d)` with
`a = step × angle_step + base_angle`, the base angles being `0xc0000000`,
`0`, `0x40000000` and `0x80000000` for sides 1, 3, 5 and 7. The arc is
entered at its middle — `step = (steps_per_side >> 1) + 1`.

`steps_per_side` and `angle_step` come from the asker's range: `q =
min(0x20000000, (0x40000000 / range) / (0xc0 / stride))`, then `k =
0x40000000 / q`, `steps_per_side = k − 1`, `angle_step = 0x40000000 / k`.
`stride` is `0x20`, `0x40` or `0xc0` by the two footprints and the asker's
~~block radius~~ **`big_radius`** (`UnitTypeData +0x244`, settled by the
type record): `big_radius < 0x31 || t.x_size < 3 || t.y_size < 3` →
(`big_radius > 0x18 && t.x_size > 1 && t.y_size > 1` → `0x40`, else
`0x20`), otherwise `0xc0`. An asker whose type has no `+0x1fc` skips all
of it and takes `q = 0x20000000` — two steps to the quarter turn — and
`+0x1fc` is `max_range`, so that arm is the **melee** asker
(`determine_roles@0061c320:58` sets `role & 0x400` from `+0x1fc != 0`).

### 17.3 The stand-off `local_18`, and the arms that never reach the ring

`local_18` is set before the ring from `ObjectData::attack_dist` — asked
from the asker's **own** position, not from the approach point
(`60138b`) — and the asker's own range. **The outer switch is not the
distance; it is `is_unit(target)`**, and only then the distance
(`601390`: `iVar5 = target->vtable+0x18(); if (iVar5 == 0 || d <= (range +
8) × 0xc0)`):

- ~~target further than `(range + 8) × 0xc0`: `local_18 = (range + 2) ×
  0xc0`, and the ring is entered directly.~~ **That arm needs
  `is_unit(target)` too, and a unit target never reaches the ring at
  all** — the ring's gate is the target's `+0x1c`, `is_build`. (`+0x18`
  and `+0x1c` are `is_unit`/`is_build`; both COMDAT-fold onto the two
  `return` constants, so `vtables.txt` cannot name them and what
  identifies them is that the Unit and Build vtables cross.) For a
  building the far arm is **unreachable**, and its `local_2c` only feeds
  the half of `00601280` that ends in `find_nearby_spot`.
- otherwise, a type without `+0x2c8 & 0x400` — `max_range == 0` — takes
  `local_18 = 0x30`; and **only then**, if the target `is_unit` **and**
  is not moving (`+0xd8`), the **melee** path
  `Unit::find_melee_pos@006010b0` and a return. A melee asker with a
  **building** target falls through with `0x30` and walks the ring, which
  is what `1/40`-`1/42` do.
- otherwise: inside `min_range × 0xc0 − 6` the asker projects *away* and
  `local_18 = min_range × 0xc0 + 0x90`; past `range × 0xc0 − 6` it is
  `max(0xc0, v)` where ~~`v = range × 0xc0 − 0x60 [− the target's block
  radius]`~~ **`v = range × 0xc0 − 0x60`, `+ big_radius − 0x30` when
  `range < 10`, `−` the *target's* `big_radius` only when the target
  `is_unit`** (`601544`-`60158a`). The three archers have `range = 12`,
  so `v = 2208` with neither correction — and 2208 is what puts `1/27`
  on run19's own `(4344, 29736)`.

Two more returns precede all of this: an asker `is_on_map` already in range
returns its own position, and a unit inside a carrier (`+0x28 ≥ 0` with the
carrier's `+0x218 == 1`) tail-calls the carrier's own `find_attack_pos` —
that is the recursion at `+0x2d` of the **nine**-argument form, not the
thunk. And a third, missed by the first reading: **an asker whose
activity order is index 12, `GUARD`, is not moved at all** (`601616`:
`local_34 == 0xc` → the out-parameters take the approach point and the
function returns 0; `local_34` is `get_activity` of the asker *itself*).
When the ring finds nothing the function falls back to
`UnitType::find_nearby_spot@0061de70`.

### 17.4 The draw, and the ceiling on a call

Per iteration the candidate is snapped to the quarter-tile centre
(`div_3_table[v >> 4] × 0x30 + 0x18`) and then tested, in order:
`UnitData::invalid_loc(tile, 1, 0, 0, 0, 1, …)`, the world tile's `0x4000`
bit, `Objects::find_collision@0065b1b0` and
`Objects::find_ordered_collision@0065b440`.

**`find_ordered_collision` is two passes, and the second is the whole
frame.** After the 3 × 3 chain walk it walks **the asker's own group's
member list** (`65b4d4`-`65b58c`): the group is the asker's `+0x80`, it
must be the asker's own player's, the asker must be *in* the list, and
every other member alive, on the map and with a block is tested by the
same unit-cell Chebyshev predicate against its `orders_x`/`orders_y`.
That catches a member ordered next to the candidate *from anywhere on
the map*, which the chain walk cannot. `crates/sim` skipped it because
"every unit in every capture so far is ungrouped"; the probe is what
stops that being true (`docs/COLLISION.md` §5.2, §9). Without it the six
pile onto one spot and every call runs to its ceiling — 51 draws; with
it, **15 + 12 + 11 + 2 + 2 + 4 = 46** and every destination in §17.5 is
exact.

A candidate that clears all four **draws once** — `Random::get(game_random, 0, 0xffff)` at `00602124`,
returning to `00602129` — and scores
`vector_dist(candidate − from) + draw % 0xc0`, lowest kept.

The loop counts iterations in `local_28` and stops when `iter − 4 ≥
local_24`. `local_24` starts at **100** and is cut the first time a
candidate beats the best-so-far:

- `local_18 > 0x300`: `local_24 = min(local_24, iter + 11)` (`60215d`)
- `local_18 ≤ 0x300`: `local_24 = min(local_24, iter)` (`60216f`)

A candidate that reaches the draw always beats a best-so-far of −1, so the
cut lands on the **first** draw and the loop then runs fourteen (or three)
more iterations. **One call therefore draws at most 15 times on the ranged
arm and 4 on the near one**, whatever the map looks like —
`sim::fight::ATTACK_POS_CAP_RANGED` and `ATTACK_POS_CAP_NEAR`. That is a
lower bound on the cast of any frame, taken from the draw stream alone:
8186's forty-six chase draws need **at least four** calls.

### 17.5 What run19 says the frame did

`gamelog-run19-window-8174-8192.txt` holds blocks 8174–8191 of this map's
own game, and the probe's orders land in block **8187**. Six of player 1's
units — `1/27`, `1/28`, `1/29`, `1/40`, `1/41`, `1/42` — gain a mandatory
`ATTACKORDER` on `ox 2004 / whom 0 / uid 4`, the farm at (2112, 31296);
`stance` goes 0 → 5 and `group` 64 → 65 on all six; the three
`ATTACKTOORDER`s of `1/40`, `1/41` and `1/42` go. Every member's
`GroupMoveOrder` carries `oxx 40`, so `1/40` is the leader — and its own
`form_id` is 3, not 0. Each of the six also gains a **plain `MOVEORDER`**
that was not there on 8186, which is the chase's answer:

| unit | sent to | from the farm |
|---|---|---|
| `1/27` | (4344, 29736) | far cluster |
| `1/28` | (4440, 29880) | far cluster |
| `1/29` | (4200, 29496) | far cluster |
| `1/40` | (2424, 30888) | within half a tile |
| `1/41` | (2280, 30888) | within half a tile |
| `1/42` | (2568, 31320) | within half a tile |

The squad structure is in the same block and is pinned with the rest:
fifteen units in group 64, `o_up` −1 on the five captains and the
predecessor's object on each follower.

`run53_s_8186_is_find_target_s_probe_and_its_ring_walks` pins all of it —
the frame's fifty-four labels entry for entry, 17656's two families, the
≥ 4 bound against the cast of six, the six destinations, the stance, the
group, and the fifteen-unit squad structure the two captains are drawn
from.

**And `crates/sim` now answers it**, entry for entry and to the unit:
`great_lakes_8186_sends_the_probe_s_six_where_the_original_does`. The
per-call split is `1/27` 15, `1/28` 12, `1/29` 11, `1/40` 2, `1/41` 2,
`1/42` 4 — the three archers carry `range = 12` and so `local_18 =
2208`, the three melee figures take §17.3's `0x30` arm and the near
budget's ceiling of 4.

### 17.6 Confidence, and what this has not established

~~Diff-backed: the frame's draw sequence and the two chains' sizes
(run53's trace); the cast, the target, the stance, the group and the six
destinations (run19's dump). Reading-only, and none of it has a second
reader yet: every word of §17.2, §17.3 and §17.4.~~ **Item 328 moved most
of that column.** Now:

**Diff-backed**: the frame's draw sequence and the two chains' sizes
(run53's trace); the cast, target, stance, group and six destinations
(run19's dump); and, new, **the ring's geometry, the octant table, the
stand-off arms, the stride/`q` arithmetic, the four rejections and the
budget** — all of §17.2, §17.3's building arm and §17.4 — because the
implementation built from them reproduces the forty-six draws *and* all
six destinations to the unit. A ring one step out does not land on
run19's own coordinates six times.

**Two readings agreeing, no run**: the blind second reading
(`docs/audit/2026-09-17-find-attack-pos-blind.md`, written without sight
of this section or the implementation) re-derived the scoring formula,
the minimum with strict `<` and its tie to the earlier candidate, the two
counter-rotating walkers alternating one candidate each, the snap, and
the budget's `100` / `4` / `+11`. Its three least-confident claims — a
**Wall** target, the sea-domain guard on the carrier delegation, and the
flanking block at `602800`-`602a1a` — are all outside the ring.

**Reading-only still**, and nothing here has executed in any traced game:
- `Unit::find_melee_pos@006010b0` and the whole **unit-target** half of
  `00601280` — the flanking chase and the `find_nearby_spot` fallback.
  `crates/sim` answers `None` for a unit target and keeps its
  straight-line approach.
- The far arm of §17.3, which is unreachable for a building.
- The `project`-away arm of a minimum-range asker inside its own dead
  zone: nothing modelled has a minimum range, so the stand-off is taken
  and the projection is not.
- The carrier delegation, and `local_28` (`vtable+0x10c && unit_masks &
  0x40000 && is_build(target)`), whose middle term is a per-unit mask bit
  this crate does not model and is taken as clear.

Not established:

- ~~**Which arm each of the six chases took.**~~ Settled: the three
  archers take the `range × 0xc0 − 0x60` arm at `local_18 = 2208` and the
  three melee figures the `0x30` arm; §17.5 has the split.
- ~~**The even sides' stride and their exit test** are read but not
  checked against a candidate count~~ — they are now, by the 46.
- ~~The frame after this one, **8187**, spends a draw at
  `PathFinder::astar_path+0x1697` … a pathfinder item, and the word now
  parts at 8187.~~ **Closed by item 329**, and the mechanism named here
  was only half of it. The chase does not wait for the next frame at all:
  `fight@005fd4d0`'s tail re-enters `Unit::work` (vtable `+0x188`) behind
  a latch on the action order, so the six plan on **8186** — run19's block
  8187 shows all six with full path stacks. §17.5's `add_move_order` is
  therefore not the end of the frame's work. The retry roll itself, its
  two tails and their unequal gates are `docs/PATHFINDER.md` §21. Great
  Lakes' word parts at **8201**.
