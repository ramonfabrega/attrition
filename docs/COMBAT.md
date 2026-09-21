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
**Per-section coverage** — which claims a diff against the original's own
dump now backs and which still rest on a reading — is stated where the
measurement was made: §16 for the formula's numbers, §18 for the engagement
frame, and §20.4 for delivery's first wound and the launch point. A reading
§20 has *contradicted* is struck where it stood (§9.5) rather than quietly
replaced. Everything open is listed at the end.

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
   a roll of `Random::get(0, 0xffff) % 100 < 5` is taken at
   **`+0xe1`**, and if the building is a fort (`BuildTypeData::is_fort`),
   `TEMPLE` (`0x1b5`) or `TOWN` (`0x19f`) **and** the attacker exists
   (`o >= 0`) and is siege, a second draw at **`+0x18b`** sizes a flock of
   `roll % 2 + 3` birds — cosmetic, but both **consume game random**
   *(second reading corrected the type test; item 394 measured the frame
   and added the addresses)*; a missile fired by a non-`NUCLEARMISSILE` at
   someone not yet at war declares war.
   **"A building" is the vtable's**: slot `+0x1c` answers 1 on `Build` and
   `Wall` and 0 on `Object`, `Unit` and `Animal` — a unit's first wound
   draws nothing. And **the gate reads `damage`, not `damage_frac`**: a hit
   worth less than a whole point moves only the fraction, leaves `damage`
   at zero, and the *next* hit draws again. §20 is the frame where that
   matters.
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

0. **The cell-centre snap, before everything else.** `fight`'s first
   statement puts the unit on `(v / 48) * 48 + 24` on both axes unless it
   is recharging and is neither a cavalry archer nor holding a re-entry
   latch — `docs/ORDERS.md` §7.11 has the gate, the diff and what it does
   not establish.
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

### 9.0 A unit's shot is launched by its **animation**, not by `fight` (item 389, 2026-09-18)

Everything below §9.1 describes what happens once an `Ammo` exists. This
is about **when** one comes into existence, and for a unit it is not on
the frame `Unit::fight` runs.

**The chain, from the trace's own ebp record** on run53 sim-frame 9425:

```text
Ammo::init+0xcd9 < Objects::add_ammo+0x119 < GraphicEvents::execute_game_events+0x40d
```

`Object::fire_ammo` is not in it. `GraphicEvents::execute_game_events
@008e48e0` walks the **event list of the guy's current animation slot**
and, for every event of kind 1, adds one `Ammo` itself. The route in is
`Objects::inc_time@0065db70`'s own pairing — per object, the vtable's
`+0xa0` (`Unit::inc_time`, every guy's clock) and then `+0x154`
(`Unit::execute_events@0060edc0` → `Guy::execute_events@005d99c0`, every
guy's event list) — so the shot is added in the animation pass, between
the clock wraps and `Farms::inc_time`, which is exactly where run53 puts
its two `Ammo::init` draws.

**The gate**, `008e4941`–`008e4989`, per event:

- `event.anim == package.cur_anim` — the list is the slot's;
- `package.last_time < event.time && event.time <= package.cur_time` — a
  **crossing**, not an equality, so a clock stepping by two fires it once
  and a clock that wraps past it does not fire it at all;
- the target: `whom != −1 && ox != −1`, or the order is `ATTACK_GROUND` /
  `AIR_ATTACK_GROUND`. `Guy::execute_events` fills those from the
  **current `UnitOrder`** (`+0xdc`'s arm: the order's `+0x8`, `+0xc`,
  `+0x10`), not from the guy, and forces `last_time` to `−1` on the frame
  `cur_time` reads zero;
- a pivot-restriction test on the piece, which no shipped archer trips.

`Guy::execute_events`' own tail adds one more for the attack category: the
target must still be active **and carry the same `uid`** the order stored.

**The times are in the install, in plain XML.** `unit_graphics.xml`'s
`<UNIT>` entries carry the event track directly — Longbowmen's is

```xml
<RELEASEEVENT starttime="466"  anim="CHAR_ATTACK1" type="ArcherArrowWOtip" node="0"/>
<RELEASEEVENT starttime="1533" anim="CHAR_ATTACK1" type="ArcherArrowWOtip" node="0"/>
<RELEASEEVENT starttime="333"  anim="CHAR_ATTACK2" .../>
<RELEASEEVENT starttime="1533" anim="CHAR_ATTACK2" .../>
<RELEASEEVENT starttime="733"  anim="CHAR_ATTACK3" .../>
<RELEASEEVENT starttime="1666" anim="CHAR_ATTACK3" .../>
```

— two per attack slot, which is `AMMO_PER_ATT` 2 arriving by a different
road than §9.1's building loop. `starttime` is milliseconds; the event
list holds a **frame**, at the fifteen-a-second rate §3.1's lengths use:

> `event.time = starttime × 3 / 200`, **truncated**.

**This is a data coupling, not a renderer one.** `CLAUDE.md`'s
load-bearing rule is that the *sim crate* depends on no graphics,
windowing or async runtime; it is not that the simulation may never read
a table whose filename says "graphics". These rows are what the
original's own simulation consults to decide when an arrow exists — they
change the random stream and the outcome of a fight — and
`rondata::artdata` already read this file for §3.1's lengths and the crew
tracks. `rondata::artdata::piece_releases` resolves them to integer
frames at load and the sim is handed a table; nothing here needs a pixel.

#### The truncation is measured, not assumed

`game_frames` (§3.1) **rounds**. The event times **truncate**, and the
two differ on every shot that pins it, because all four of the shipped
products land at remainder 198 or 199 of 200. Run53's first four
`Objects::add_ammo` calls in 24,000 frames are Great Lakes' first two
archers, and the dump gives each guy's animation clock frame by frame:

| shooter | slot | `starttime` | truncated | rounded | fired on |
|---|---|---|---|---|---|
| `1/29` | `CHAR_ATTACK3` | 733 | **10** | 11 | **9425** (`cur_time` 9 → 10) |
| `1/28` | `CHAR_ATTACK2` | 333 | **4** | 5 | **9426** (`cur_time` 3 → 4) |
| `1/29` | `CHAR_ATTACK3` | 1666 | **24** | 25 | **9439** |
| `1/28` | `CHAR_ATTACK2` | 1533 | **22** | 23 | **9444** |

Four for four on the truncation, nought for four on the rounding.

#### What it cost, and the negative that pins it

Great Lakes' word moved **9415 → 9451**. 9415 was five draws against two:
this crate's `1/29` launched on the frame it came into range, spending
`scatter_point`'s two draws (§9.5) where the original spends none, and
the shifted stream then cost a bird its idle variant. Every frame from
9415 to 9450 now agrees draw for draw — both archers' first arrows, both
their second, and the thirty frames of reload between.

The **negative** is the cheap permanent one:
`run53_s_first_ammo_is_the_animation_s_own` — the original's first
`Objects::add_ammo` in 24,000 frames is sim-frame **9425**, so a
simulation that launches an arrow before it is wrong by that fact alone,
whatever its draw count says.

#### What this has **not** established

- **`UnitType +0x2cc`**, the type's fire-projectile graphic, is the one
  other route into `Object::fire_ammo` for a unit: `Unit::fight`
  @`005fee?` calls it when `+0x2cc != 0` or the merchant arm
  (`is(0x3e)` outside `unit_masks & 0x80000`) holds. Longbowmen reach
  neither — their arrow is a `RELEASEEVENT` — and nothing on this disk
  reaches the field at all. This crate has no reading of it, and so
  defers exactly when the install's `<RELEASEEVENT>` table knows the
  piece and launches immediately otherwise. That fallback is a **seam,
  not a claim**: it is what keeps every sim built from tables alone
  meaningful.
- ~~**The launch point.**~~ Open still, but no longer vague: **§20** names
  the table it comes from, measures what it has to be worth on four shots,
  and names the capture that closes it. It is the only thing between this
  crate and Great Lakes 9451.
- **`Guy::execute_events`' `uid` test.** This crate tests that the target
  is still active; the original also requires the stored `uid`. A target
  that died and had its slot reused inside one animation would part them.
- **Melee.** `Unit::fight`'s `do_damage` arm is untouched, and no capture
  here has a melee unit in a fight: run53's first `Object::do_damage` is
  9451, and it is an arrow landing.

### 9.1 Launch — `fire_ammo(o, who)`

A **unit** adds one `Ammo` **per figure** on this `UnitData` (`guy_mark`), at
the figure's position, `z + 100`, with the unit's facing, `gpiece =
type.fire_proj`'s graphic, `who/o = A`, `whom/ox = T` — **but `fire_ammo`
is not how an archer shoots**, and §9.0 is where a unit's `Ammo` actually
comes from. A **building** adds
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
x then y, each only when the footprint axis exceeds one unit;
`Object::fire_ammo`'s own loop, which a unit's animation-launched shot
does not run — §9.0); one or two
`% 100` for a shot at an aircraft; two for the landing scatter (when `s >
1`) — `Ammo::init+0xcd9` and `+0xd0b`, named in
[`trace::SITES`](../crates/rondata/src/trace.rs) since item 389; at landing, two more for where a no-target shot punctures the ground;
in `take_damage`, ~~one `% 100` on the first wound of a fort, temple or
town~~ — **wrong; §20 measured it**: the `% 100` is *any* building's first
wound, the fort test gates only the flock, and one frame can spend it
twice; and, outside
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
a quarter tile; `value = compare_target(o, who, in_range, ai)` — and
**`in_range` is a permission to test, not a verdict**, see §12.3 —; **`score =
value / (dist / 0xc0 + 1)`**; the previous mandatory target halves; the
`flags` preferences halve the wrong class; a cavalry archer's second-weapon
search weights by bearing (×4 within 30°, ×2 within 60°, /10 beyond 90°,
skip beyond 135°); `score == 0 && value != 0 → 1`; strictly greater wins
(ties to the earlier — nearer ring, lower `dx`); and **after ten unit
candidates with a best in hand the scan stops**. The winner's `targeted`
goes up by one (cap 100). With `add_order` the attack order is added:
`QUEUE_FIRST` for a guard or a DEFENSIVE stance, `QUEUE_NEW` otherwise, and
`mandatory` only for AI-controlled siege on a city.

**Both constants of the score are read off the listing, not the decompiler**
(item 386). `00649701`–`0064970e` is `movsbl 0x3d(%eax); addl $8; leal
(%eax,%eax,2); shll $4` — `targeted` at `ObjectData+0x3d`, `+ 8`, `× 3`, `×
16`, so `(targeted + 8) × 0x30` to the byte. `00649718`–`00649731` is `movl
$0x2aaaaaab; imull; sarl $5` with the sign fixup, and `⌈2³⁷ / 192⌉ =
0x2AAAAAAB`, so the divisor is `0xc0` and nothing else. Both were §18's
suspects and both are exonerated; the answer was in `compare_target`. run108's
`max_dist = 4608` and its three `attack_dist`s of 288, 198 and 339 are §12.4's
radius and §13.1's extent measured from the same run.

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
hits `99,999`; negative `9,999,999`; **out of range and not raiding `/5`, on
this function's own range test** (below); `v = ceil(v / 100)`, above 100,000 compressed to `100,000 + (v − 100,000) /
5`, floor 15; a detected hidden unit 1; an air target for a non-ANTI_AIR
attacker 2.

**The `/5` is the function's own range test, and `in_range` is only its
gate** (item 386, `0064f1ed`):

```
if (param_3 != 0 && !raiding &&
    is_in_range(this, o, who, this->x, this->y, this->y, 0, NULL) == 0)
    v /= 5;
```

`param_3` is the caller's `in_range` — §12.2's range gate, which for a
non-guarding unit that is not STAND_GROUND, not entrenched and not an unpacked
packer is `1` **without any test having been made**. So the argument says *"I
did not measure; you measure"*, and `compare_target` measures. Reading it the
other way round — divide when the caller reports out of range — makes every
candidate of an ordinary aggressive unit score alike, which is exactly the tie
§18 could not break. `raiding` is the attacker's RAID stance, qualified for a
raider that is `is(0x40000)` on a `field_0x218 == 1` type; both sit inside the
RAID arm at the top of the function, so a non-raider reaches this line with
`raiding` false.

run108 measures the arm firing: three identical undamaged hoplites at 198, 288
and 339, all three handed `in_range = 1`, come back **10771**, 2155 and 2155,
and each candidate spends a *second*, nested `attack_dist` — `is_in_range`'s —
on top of `check_target`'s. That is the only reason the golden record's
captain does not take the first candidate the cell hands it.

The implementation carries the skeleton of this — cost, the building class
multipliers, the `attack × 100 / hits_left` preference, `× dmg`, the combat-
role and supply bonuses, **the `/5`**, `/(full+1)`, the ceil/compress/floor —
over what the simulation has, and leaves the raid, spell, stealth and AI
branches as inputs. See `combat::compare_target`.

### 12.4 Switching and opportunity

- **Idle**: `Unit::think` runs when `(o + frame) & 0xf == 0` once the unit has
  been idle more than two frames; on frames where `(o + frame) & 0x1f != 0`
  it only checks the **nearest cache** (`near_o` active, in range, stance
  below HOLD_FIRE → `add_attack_order(QUEUE_NEW)`); every 32nd frame and the
  first idle frame a combat unit (`attack != 0 && role & 0x10000`) runs
  `think_attack` → `find_melee_target(−1)`: radius `(max_range + 1) × 0xc0`
  (`+ 0x180` in AGGRESSIVE, `0x120` for melee **and no bonus on top of it** —
  the AGGRESSIVE term is inside the `max_range != 0` arm at `005ff9c0`), at
  least `unit_respond_range × 0xc0`, and at least `unit_respond_range ×
  0x180` again when the unit carries `unit_masks & 0x40000`; both floors and
  the nesting landed in the code with item 384; DEFENSIVE: `max(max_range × 0xc0,
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
- **Hit** (`do_damage` step 2 → `Unit::target_opportunity` on the victim):
  ignored unless at war; then a `while (true)` whose body ends
  `if (is_captain(this)) break; param_3 = 0; this = objects[who][this->o_up]`
  — so **every test below runs on the head of the victim's squad, never on
  the figure that was hit** (§18.2 measures it on the golden record). The
  loop's other arm, tested at each level *before* the captain test, sends a
  group member whose type is **not** combat-role (`type +0x2c8 & 0x10000`)
  to `Group::target_opportunity` instead — which every 15 frames at most
  tells its idle combat captains to `find_melee_target` within
  `min(dist + 0xc0, unit_respond_range × 0x240)`. HOLD_FIRE ignores; a unit already attacking ignores it unless its
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
over half a tile), or `d ≤ 0xf6` for the `HOPLITES` line — `is(0x84, 0)` on
the **attacker**, at `006486b0`, and `0x84` is the tech-tree id the golden
record's `add hoplite` lands on. Implemented and diff-backed since item 384
(`sim::fight::HOPLITES`, §18); until then `combat::in_range`'s `hoplites`
argument existed and nothing passed it, so every melee unit in the
simulation fought at `0x66`. **Ranged**: `d <
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

8. ~~**§12.2's two ranking constants have never been diffed.**~~
   **Settled, 2026-09-18 (item 386), and neither was wrong.** The listing
   reads `(targeted + 8) × 0x30` instruction for instruction and `÷ 0xc0`
   out of `0x2aaaaaab`/`sarl $5`; §12.2 carries both derivations. What
   made frame 615 look like it contradicted them was the assumption that
   its three candidates score equally, and run108 says they do not —
   `compare_target`'s own `is_in_range` discounts two of them by five.
   §12.3 and §18.1 have it, and the constants are now diff-backed as
   well: the run's `attack_dist`, `max_dist` and per-candidate `value` are
   all on the record.

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
- ~~`Unit::find_melee_pos@006010b0` and~~ the rest of the **unit-target**
  half of `00601280` — the flanking chase and the `find_nearby_spot`
  fallback, which a *moving* unit target or a ranged asker takes;
  `crates/sim` answers `None` for those. `find_melee_pos` itself is
  diff-backed, §19.
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

---

## 18. The engagement frame, read off the golden record (item 384, 2026-09-18)

Chapter one spawns three hoplites for who=0 at frame 610 and three for who=1
at 615 (`docs/INPUT.md` §11), two tiles apart, and what happens next is the
rules headline's frame. Every claim here is the dump's own, at
`UNITS=3`/`GUYS=2` over frames 615–621 (`docs/RUNS.md` run101–run105); the
harness reproduces the positions exactly (`docs/ANIM.md` §6.3).

**Only the captain searches**, and `near_o` is the witness.
`ObjectData::near_o`/`near_who` are written by `find_nearby_target` and by
nothing else, on any call that finds a candidate within `0xf00` — a per-unit
record of *having searched*. Over all 901 frames only **`1/6` carries one**
(`7`/`0`, from 616 on); `1/7`, `1/8` and all three of who=0 read `-1`
throughout. The other two members get their order from
`Unit::think_attack@005f5a80`'s tail, which on a find calls
`Group::target_opportunity(group, o, who, …)` — the captain's target handed
to every member. All three of who=1's carry `group 64`; who=0's carry `-1`.

**who=0 never searches, and its 617–618 orders are retaliation.** `think`
reaches `think_attack` only on a unit's first idle frame or when
`(o + frame) & 0x1f == 0`; who=0's three are born at 610 and their next
cadence frames are 632, 633, 634. All three take an ATTACKORDER on `1/6` —
the unit that struck `0/7` — at 617 and 618: §12.4's "Hit" path. The dump's
discriminator is `unit_masks`, `262144` (`0x40000`) on who=1's and `0` on
who=0's.

**Six range verdicts, and one constant decides two of them.** Seated, the
cross-squad `attack_dist`s are

| pair | `attack_dist` | the original |
| --- | --- | --- |
| `1/6` → `0/7` | 198 | **strikes from its seat** — `in_range 1`, `recharging 32` at dump-617, never moves off `(1368, 7992)` |
| `0/7` → `1/6` | 198 | **strikes back** at dump-619, never moves off `(1032, 7800)` |
| `1/6` → `0/8` | 288 | walks |
| `1/8` → `0/7` | 316 | walks |
| `1/6` → `0/6`, `1/7` → `0/7` | 339 | walk |

`0x66` (102) puts none of them in reach; `0xf6` (246) puts exactly the two
the dump has and no others. That is §13.2's HOPLITES arm, and it is what the
implementation was missing.

**~~What is *not* settled: which target the captain picks.~~ Settled by
run108** (item 386; the paragraphs below are kept for the reasoning that was
wrong, and §18.1 has the answer). The original's
`1/6` takes `0/7`; the harness took `0/6`. Under §12.2's formula the three
candidates appear to tie exactly — `dist + (0 + 8) × 0x30` is 723, 582 and 672, each
`/ 0xc0` is 3, so each scores `value / 4`, and `compare_target` returns the
same `value` for three identical undamaged hoplites (its `get_damage` is
called at `find_angle(0, 0)`, so bearing cannot separate them). A tie is
broken by scan order, and **neither order gives `0/7`**: the world cell's own
`down` chain, which `find_nearby_target` walks, is `1/8, 1/7, 1/6, 0/8, 0/7,
0/6` (confirmed against the dump's `up`/`down`/`up_who`/`down_who` on all
six) and yields `0/8`; the harness's unit-index order yields `0/6`. So one
step of §12.2 or §13.1 is wrong, and `0/7` is the only answer either can be
asked to produce.

**~~What would settle it.~~ Refused by run108**: its three `attack_dist`
calls answer 288, 198 and 339, which is §13.1's formula at `block_radius +
0x18 = 72` exactly, so the extent below is not the original's. The arithmetic
is only satisfied by a larger
per-side extent: at `block_radius + 0x18 = 72` the three tie, and at ~84
(`0x54`) `0/7` alone lands in the `/ 0xc0` bucket below the other two while
all six range verdicts above still hold — 96 is already too large, because
`1/8` → `0/7` then falls to 246 and would strike where the dump has it
walking. That is a derivation, not evidence, and it is written here rather
than fitted into the code: an extent that reproduces one frame's pick is a
fit until something independent of this frame agrees with it.

§13.1 is against it, too: that formula was read off the
disassembly rather than the decompile, so the `block_radius + 0x18` on each
side is the best-evidenced step of the three.

~~Two candidates are left~~, and both are **reading-only constants of §12.2**
that no diff has ever exercised: the `/ 0xc0` divisor, and the
`(targeted + 8) × 0x30` term. Either one, differently scaled, separates 198
from 288 without touching a range verdict: a `/ 0x60` divisor, for one,
gives `0/7` outright. This frame cannot falsify that — `1/6` is the only
unit in chapter one that ever searches, so there is exactly one observation
and any number of constants fit it. That is the whole reason it is here and
not in the code. ~~Three checks, cheapest first~~ — all three were run, and
§18.1 says what each returned; the arithmetic above is sound and its premise,
that the three `value`s are equal, is what was false.

One thing this rules out. `check_target`'s "a target in another region must
be `is_in_range`" cannot be what rejects `0/6` and `0/8`: `get_tregion`
resolves all six tiles to region 0 — cell `(1, 10)` carries `region 0`,
`region2 1` and no tile of it reads ocean — so the gate never fires here.

### 18.1 The pick, measured (item 386, 2026-09-18)

`docs/RUNS.md` run108 is chapter one re-run with the trace's **call proxies**
over frames 614–618 and three new sites: `find_nearby_target@00648da0`, whose
entry and return bracket one search, and `attack_dist@006488f0` and
`compare_target@0064e5c0` inside it. The whole answer is one bracket:

| candidate | `attack_dist` | `in_range` handed in | `compare_target` |
| --- | --- | --- | --- |
| `0/8` | 288 | 1 | 2155 |
| `0/7` | **198** | 1 | **10771** |
| `0/6` | 339 | 1 | 2155 |

`max_dist = 4608`, `add_order = 1`, `cavarch = 0`, `flags = 0`, return **7**.

**They do not tie.** The reachable candidate stands a factor of 4.998 above
the other two, and the factor is §12.3's out-of-range `/5` — which fires on
**`compare_target`'s own `is_in_range` call**, not on the `in_range` argument
the search hands it. Each candidate spends a second, nested `attack_dist` on
top of `check_target`'s, which is that call showing through.

So the constant that decides the frame is `0xf6` — the same HOPLITES reach
item 384 landed. `0/7` at 198 is inside it and the other two are not. With the
old `0x66` reading all three would have been out of reach, all three would
have taken the `/5`, and the tie would have been real; the two findings are one
constant read at two sites.

**What this crate had** was the predicate inverted — `if !in_range &&
!is_in_range(…)` — so the discount never fired for an aggressive unit and the
three tied at 112,506 apiece. `chapter_one_s_captain_picks_the_one_it_can_reach`
pins both halves: the captain's target after frame 615 is `0/7`, and with
`in_range = false` (the old arm) the three values are equal again — the tie
that hands the pick to whichever the cell's `down` chain reached first.

**Four open questions close with it**, all from the same bracket: the candidate
order *is* the cell's `down` chain; `attack_dist` *is* §13.1 at `block_radius +
0x18 = 72`, so §18's ~84 extent is refused; `max_dist` *is*
`unit_respond_range × 0x180`, so §12.4's radius arm is diff-backed; and §12.2's
range gate does take the "deemed in range without testing" arm for a
non-guarding aggressive unit.

**What the fix moved, and what it did not.** The golden word is unchanged at
617 and the value diff at 618, but the frame's anatomy moved a long way toward
the original's — the dump's own coordinates beside the harness's, reading each
frame's end:

| | dump 617 | ours, before | ours, after |
| --- | --- | --- | --- |
| `1/6` target | `0/7` | `0/6` | **`0/7`** |
| `1/6` seat | `(1368, 7992)` | walking, `(1341, 7982)` | **`(1368, 7992)`** |
| `1/6` `recharging` | 32 | 0 — first strike at 621 | **32** |
| `0/7` order | retaliates at 618 | none, idle through 621 | **ATTACKORDER at 617** |

~~**What is still wrong, and it is a group question.**~~ **It was not a group
question** — §18.2 has the cause and the frame moved. Kept for the shape of
the wrong reading: at the end of frame 617 this crate's `1/6` dropped its
target and took a `GROUP_ATTACK_TO`, and walked off the seat it held, and
`0/7`'s retaliation struck a frame early. Both were one defect, and it was
neither the group's nor `Group::target_opportunity`'s.

### 18.2 The hit is the captain's (item 391, 2026-09-18)

**`Unit::target_opportunity@005fffc0` never answers on the figure that was
hit.** Its opening `while (true)` ends

```text
    if (is_captain(this)) break;                    // (ushort)o_up >> 15
    param_3 = 0;
    this = objects[who][this->o_up]->get_unit();    // ObjectData::get_captain
```

so the valid-target test, the stance test, the "already attacking" arm and
the `add_attack_order` at its foot all run on the **head of the victim's
squad**. `Object::do_damage` passes the victim; the function walks up.

**The golden dump says it outright**, and it is five fields on two frames —
who=0's `o_up` chain is `0/8 → 0/7 → 0/6`, and `0/6` alone reads `o_up -1`:

| dump block | `0/6` | `0/7` | `0/8` |
| --- | --- | --- | --- |
| 617 (end of 616, the frame `1/6` struck `0/7`) | **ATTACKORDER `ox 6 whom 1 uid 12`, `new_ord 1`** | none, `damage 3`, `damage_frame 616` | none |
| 618 (end of 617) | ATTACKORDER + MOVEORDER, walking | ATTACKORDER, `new_ord 1` | ATTACKORDER, `new_ord 1` |
| 619 (end of 618) | walking | `in_range 1`, **`recharging 32`** — it struck | ATTACKORDER + MOVEORDER |

The captain takes the order on the frame of the hit; the two members take
theirs a frame later; and `0/7`'s own retaliation therefore lands on **618**,
not 617.

**What this crate had** was the retaliation on `victim` itself
(`fight.rs::target_opportunity`), which is one `squad_captain` call away from
the original and cost the golden record's frame 617 outright:

- `0/7` took the order at the end of 616 and struck `1/6` during **617**.
- That `do_damage` reached **this crate's** `Armies::emergency(1)` →
  `Army::process` → `Group::action_siege_attack_to`, and the army walked all
  three of who=1's hoplites off toward `(38646, 13305)`. `1/6` lost its
  target and its seat. (The original's `do_damage` does not: §23.)
- So frame 617 lost **both** of the draws the original spends there:
  `Unit::fight+0x9b0`, `0/6`'s one-in-five re-search on its fresh order
  (§12.4's captain gate, `state.captain == index`), and
  `Guy::set_anim+0xf2f < Guy::move+0x166`, `1/6`'s deferred swing from 616
  paid on the first frame its turn settles (`docs/ANIM.md` §6.2) — which a
  walking guy never reaches.

`Armies::emergency` is **not** gated by `ai off`: the cheat's only readers in
the simulation are `Unit::think@005f6e40:206`, `Leader::production_ai` and
`Leader::diplomacy` (`grep GameAccess::ai_off`). ~~It fires in the original too
— one frame later, on 618 — and does not move the squad.~~ ~~Whether this
crate's `Army::process` would still walk it there is **not established**~~
~~**It does** (item 395, §21.4): once `0/7` strikes on 618 rather than 617 the
call is reached again, and the bypass tick walks all three of who=1's
hoplites toward `(38664, 13320)` where the dump holds them. The gate is
right and the tick is not — `Object::do_damage`'s gate at `0064bbfd` wants the victim's
leader to read `leader_flags & 4 == 0`, which is
`LeaderData::is_human@006ec170`, so the emergency is the computer leader's
and who=1 is the computer leader.~~ **It does not fire at all** — neither on
617 nor on 618, and neither in the original nor, now, here. The
`leader_flags & 4` gate above is the *second* conjunct of the call's `if`;
the **first** is `local_30`, which only the city-alarm arms of `do_damage`'s
**building** branch ever set, so a soldier taking a hit never reaches the
emergency (item 399, §23; `docs/ARMY.md` §15.8). What cost this crate frame
617 stands: the march was ours alone.

**The word: 617 → 618.** The value diff moves with it, 618 → 619.

~~**What stands at 618** is one draw, `0/6`'s second `Unit::fight+0x9b0`.~~
**Closed by item 392, and it was not §17's ring.** The original's `0/6`
spends 617 in `Unit::find_melee_pos` — §19, the arm a melee asker with a
seated unit target takes *instead* of the ring — which answers `(1080,
8280)`; `add_move_order` takes it and the chase re-entry plans the
five-node path the dump prints at block 618 (`(1080,8280) ← (1032,8280) ←
(792,8040) ← (792,7848) ← (840,7800)`), so from 618 its current order is
the move and `do_attack` is not reached again until 650. This crate
answered `None` for every unit target, fell back to the target's own
point, lost the move inside the frame that ordered it, and rolled again on
618, 620 and 621. **The word: 618 → 619.**

~~**What stands at 619** is `Guy::set_anim+0xf2f < Guy::move+0x166`, the
deferred swing, which this crate pays on 621.~~ **Closed by item 395, and
the mechanism was not `Group::target_opportunity`** — §21. The chase was
planned on the right frame but to `(1224, 7704)` where the original walks
to `(1080, 8280)`, because three of §19's six ring rejections hang on
`1/7`'s ordered destination and this crate's `1/7` searched for its own
target (`0/6`) where the original's squad carries the captain's (`0/7`).
What hands it down is **`Unit::think`'s first statement**: a non-captain
mirrors its captain's standing ATTACK order and the think ends there, so a
member never searches at all. `Group::target_opportunity` could not have
been it — its loop only enters a member that is itself a captain, and group
64 has one. **The word: 619 → 624**, and §21.4 has what stands there.


---

## 19. `Unit::find_melee_pos` — the arm that never sees the ring (item 392, 2026-09-19)

**A melee asker with a unit target never walks §17.2's ring.** §17.3's
second arm says so and the listing is plain: with `max_range == 0` the
stand-off is `0x30`, and then, if the target `is_unit` (`vtable +0x18`)
**and** is not moving (`vtable +0xd8`, `UnitData::is_moving`, the head
order's own `Order::is_move`), `6017eb` calls
`Unit::find_melee_pos@006010b0` and `6017f4` returns its answer. The whole
call is that function; nothing below it runs.

`find_melee_pos(this, o, who, p3, &x, &y, from.x, from.y, p8)` — `ret
0x20`, and the call site fills `from` with **the asker's own position**
(`6017c8`-`6017db`, `this->+0x10`/`+0x14` unencrypted). It is two steps:

1. `Unit::find_open_slots@00600e30` fills a `SimpleArray<CoordData>`.
2. The slot with the smallest `vector_dist(slot − from)` wins, ties to the
   **earlier** in ring order (`6011f5`'s `cmp bestd, d` / `jle` keeps the
   incumbent; `bestd` starts at `99999999` in the `param_1` slot).

An **empty** array is `return 0` — and then `find_attack_pos` writes the
target's own position into its out-parameters, bumps the *target's*
`UnitData::full` (`+0xac`) by 5 capped at `0x1e`, and returns 0 as well,
so every caller falls back to the target's point. `full` is not modelled
and nothing in this crate reads it.

**`find_open_slots` is an axis-aligned square ring, and it costs no draw.**
That is why a melee chase is invisible in the draw stream and only the
*dump* can measure it.

- The centre is the target's quarter-tile, `div_3_table[pos >> 4]` each
  way — the cell `× 0x30 + 0x18` grid §17.4 also snaps to.
- The radius `r` is `asker.+0x248 + target.+0x248` plus **4**, or plus
  **1** when the asker's `ObjectData::is(0x84, 0)` is false (`600eaa`,
  `600ebc`; `600e94` devirtualises `is` to the type's own vtable `+0x60`
  because `Unit::vftable +0xb8` *is* `ObjectData::is`). `0x84` is
  `HOPLITES`, the heavy-infantry root, so the line stands a full tile
  further out than everything else. `+0x248` is `BLOCK_RADIUS` before its
  `× 48` — `Sim::coll_size`.
- The step is `2 × asker.+0x248 + 1` quarter-tiles.
- The walk starts at the **south-west** corner `(cx − r, cy + r)` and
  takes `orthog_x@00add250` / `orthog_y@00add210` **indices 1..4** —
  `(0, −1)`, `(1, 0)`, `(0, 1)`, `(−1, 0)` — north, east, south, west. A
  coordinate that overshoots the band is clamped to `cx ± r` (or `cy ±
  r`), the index advances, and **one step in the new direction is applied
  before the next test** (`601030`-`601067`). The walk ends when the index
  passes 4, so the starting corner is tested **twice** — first and last.
- Each candidate is tested by four predicates in order: the map bounds
  (`0 ≤ q < world.w × 16`), `UnitData::invalid_loc(tx, ty, 0, 0, 0, 0, 1,
  ty)`, `Objects::find_collision` and `Objects::find_ordered_collision`,
  the last two with the asker's own `o`/`who` so it does not block itself.
  The `param_3` here is **0** where §17.4's ring passes 1, which is what
  puts the tile's `0x4000` bit inside `invalid_loc` rather than beside it.

**Chapter one's own numbers, and they are the whole confirmation.** `0/6`
is ordered onto `1/6` at the end of 616 and asks on 617. `1/6` sits at
`(1368, 7992)`, quarter-tile `(28, 166)`; both hoplites carry
`BLOCK_RADIUS 1`, so `r = 6` and the step is 3, and the ring's seventeen
candidates run `(22, 172)` → `(22, 160)` → `(34, 160)` → `(34, 172)` →
`(22, 172)`. The dump's answer is the **first** one, `(1080, 8280)` —
which is the *farthest* corner from the asker at `(888, 7800)`. Six
nearer candidates beat it on `vector_dist`, and the original rejects
every one of them on `find_ordered_collision` against three
`orders_x`/`orders_y` the same block prints:

| ordered destination | quarter-tile | rejects |
| --- | --- | --- |
| `0/7` seated, `(1032, 7800)` | `(21, 162)` | `(22, 160)`, `(22, 163)` |
| `1/7` walking to `(1320, 7800)` | `(27, 162)` | `(25, 160)`, `(28, 160)` |
| `1/8` walking to `(1176, 8088)` | `(24, 168)` | `(22, 166)`, `(22, 169)` |

Two unit cells each way, which is the two `coll_size`s summed. That is an
exact account of the frame: the six that would have won, and nothing else,
are the six an ordered destination covers.
`chapter_one_s_melee_chase_stands_where_the_golden_dump_puts_it` builds
those six units at the dump's own positions and ordered points and gets
`(1080, 8280)`; seat `1/7` and `1/8` on themselves instead and it answers
`(1080, 7992)`.

**What this is backed by.** The ring's geometry, the `+4` spread, the
step, the walk order and the `vector_dist` minimum are all pinned by that
one coordinate — no other radius, spread or start corner puts `(1080,
8280)` first — and by a second, independent one: this crate's `1/8`, which
targets `0/7` as the original's does, now answers the original's own
`(1176, 8088)` in the live harness where it used to answer the target's
point. The `orthog` tables are read straight out of the PE.

**What it has not established.** The `+1` arm (a non-`HOPLITES` asker) has
no observation: every melee asker in chapter one is a hoplite. `full`
(`+0xac`) is written and never read here. And `find_melee_pos` is still
not reached for a **moving** unit target, which is the rest of §17.6's
unit-target half.

---

## 20. The first wound, and where an arrow starts (item 394, 2026-09-19)

Great Lakes 9451 is the frame the long capture's word sat on after item 389,
and it is the first hit point lost in the whole 24,000-frame game. The
original spends **seven** draws there and this crate spent five; the two
missing were both `Object::take_damage@00652020+0xe1`, back to back, with the
identical chain `< Object::do_damage+0x159e < Ammo::do_damage+0xc11`. Nothing
in this document explained a site drawing twice in one call chain — §9.5 said
the draw belonged to a fort, a temple or a town, and `0/2004` is a Farm.

### 20.1 It is two arrows, and the fraction is why the gate re-opens

The value diff answers it without a hypothesis. `track.py BUILDDATA
damage,damage_frac --where who=0,o=2004 --changes` over run100:

```text
block 9340   damage 0   damage_frac 0
block 9452   damage 1   damage_frac 10      +26 sixteenths
block 9465   damage 2   damage_frac 7       +13
block 9470   damage 3   damage_frac 4       +13
block 9477   damage 4   damage_frac 1       +13
block 9481   damage 4   damage_frac 14      +13
```

Every step is **thirteen sixteenths** and the first is **twenty-six**, which
is two of them. So two arrows land on sim-frame 9451, and thirteen is one
Longbowman figure's share: `ATTACK 15` against a Farm gives `get_damage` 5,
and §7.1 step 5 divides it by `AMMO_PER_ATT 2` and `UBER_SIZE 3` —
`5 × 0x100 / 2 / 3 = 213`, `213 >> 4 = 13`, `whole 0`, `frac 13`.

**`whole 0` is the finding.** §7.2 step 4 accumulates `damage_frac + frac`
and carries only at sixteen, so the first arrow leaves `damage` at **0** and
`damage_frac` at 13. Step 3's gate is `damage == 0` — the whole-hit count, not
the fraction — so the second arrow, in the same frame, finds it still open and
draws again. The first draw returns `60391 % 100 = 91` (no flock, and no fort
test reached); the second returns `24102 % 100 = 2`, which passes, and then
the fort/`TEMPLE`/`TOWN` test fails on a Farm, so no second draw at `+0x18b`.
Both are `Random::get(game_random, 0, 0xffff)` and the trace's own seeds chain
one to the next with nothing between.

So the predicate §9.5 carried was wrong in the way readings usually are —
the *order of the tests*, not the arithmetic. The roll is any building's, and
`"a building"` is the vtable's: slot `+0x1c` is a folded `return 1` on
`Build` and `Wall` and a folded `return 0` on `Object`, `Unit` and `Animal`
(the map's COMDAT folding gives both thunks other classes' names —
`Buffer::is_pending_load` and `Window::get_button` — and the six-byte bodies
settle it).

**Implemented** as `Sim::first_wound_draws` (`crates/sim/src/fight.rs`), under
the site names `Object::take_damage+0xe1` and `+0x18b`, both now in
`trace::SITES`. 9451 is **six** of the original's seven draws with it.

**The negative is a trap, and it was run.** Adding `damage_frac == 0` to the
gate halves the first wounds from two to one — and makes 9452 agree with the
original **eight draws for eight**, where the correct gate spends nine.
Past-the-word counts rise with it (9,722 → 9,764 frames draw for draw). That
is `docs/QUEUE.md` item 89(c) in miniature: both streams are nobody's past
the word, so a frame that starts agreeing there is not evidence. What *is*
evidence is the original's own 9451, which spends the site twice, and only a
gate that ignores the fraction can. Asserted in
`run53_s_24000_frames_put_the_ceiling_where_run33_did`.

### 20.2 The seventh draw is the second arrow, and it is a frame late

The launches below the word are the original's own, `[9425, 9426, 9439,
9444]`, and this crate takes all four on the right frames (item 389). The
landings are not:

| launch | shooter | anim | this crate's `total_time` | lands | the original |
|---|---|---|---|---|---|
| 9425 | `1/29` | `CHAR_ATTACK3` | 27 | 9451 | 9451 |
| 9426 | `1/28` | `CHAR_ATTACK2` | 27 | 9452 | **9451** |
| 9439 | `1/29` | `CHAR_ATTACK3` | 26 | 9464 | 9464 |
| 9444 | `1/28` | `CHAR_ATTACK2` | 27 | 9470 | **9469** |

`1/29` is right twice and `1/28` wrong twice. Everything that feeds
`total_time` has been checked against the capture and agrees: both units stand
on the original's own coordinates (`4680, 29928` and `4776, 30168`, the dump's
own, and each unit has one figure at the unit's position); the scatter draws
are the original's values (`38791 % 192 − 96 = −89` and so on, four for four);
and the aim — the building's centre pulled `x_size × 0x30` back along
`find_angle(T − launch)` — reproduces the landing points that make `1/29`'s
two flights come out right. The arithmetic is the listing's, read at
`0067d0a3`: `dx = ex − sx`, `dy = ey − sy`, `cvtdq2ps`, `sqrtf`, `divss` by
`(float)(PROJ_SPEED × unit_move_speed)`, `cvttsd2si`. There is no rounding
freedom in it.

What is left is `sx, sy` — the ammo's own launch point, `AmmoData +0xc/+0x10`.
`GraphicEvents::execute_game_events@008e48e0+0x40d` adds
`GraphicPieces::get_position`'s vector to the **guy's** `x/y/z` immediately
before `Objects::add_ammo`, and that vector is per **(piece, node, anim,
starttime)**. `1/28` fires on `CHAR_ATTACK2` and `1/29` on `CHAR_ATTACK3`, so
they get different ones — which is exactly the shape of a defect that is right
for one archer and wrong for the other.

**Two arms of the same function were ruled out rather than assumed.** The
`CHAR_ATTACK2` arm of `Ammo::init` that shifts the *landing* point (`project`
along the unit's facing by `(t/end − 0.3) × 6 × 192`, then `±0x30` at ninety
degrees) is gated on `unit_flags & 0x400000`, and `unitrules.xml` gives
Longbowmen `<FLAGS>lmjiy</FLAGS>` — no `w`, so the arm cannot run, and the two
scatter draws the capture shows confirm it (that flag also zeroes `s`).

### 20.3 What the offset has to be, and what would close it

Bounds, from the four shots, as a reduction in the shooter-to-landing distance
(`d = PROJ_SPEED 100 × unit_move_speed 1`, so a frame is 100 position units):

| shooter | this crate's distances | the floor each must take | the offset's component along the shot |
|---|---|---|---|
| `1/29` | 2785.3, 2675.7 | 27, 26 | **0 … 75.7** |
| `1/28` | 2776.6, 2714.1 | 26, 26 | **76.6 … 114.1** |

No single scalar fits both, and no integer divisor fits either — which is the
arithmetic statement that the difference is per-animation and lives in the
launch point rather than in the formula.

**This does not license a float.** `GraphicPieces::get_position` reads
`GraphicPieces::positions`, an `Array<AttachPos>` of `{ushort time; char anim;
char node; Vector<float> pos; Vector<float> vel}` indexed by `pos_start`/
`pos_num` per graphic piece, and `GraphicPieces::init_position@00901820`
**fills that array at load** by running the model's own animation at
`starttime × 0x43` and reading the node out. The original is therefore already
the second of `docs/DECISIONS.md` entry 16's two shapes — a table built once
before the first frame — and the port's answer is the same table, pinned. The
blocker is not the float rule; it is that building the table needs a `.bh3`
skeleton reader and a bone transform, which is phase 4 work and does not exist
here. `get_position` then rotates the entry by the guy's angle and scales by
`guy_scale`, so the pinned entry is a model-space triple and the rotation is
`movement`'s own integer sine table.

**The capture that closes it without any of that** is one line of
`gamelog.ini`: `AMMO=1` under `[End Frame]`, over `[9420, 9460)` on the Great
Lakes seed. `AmmoData::log_data@00679c00` writes the record the PDB names —
`sx, sy, sz, ex, ey, ez, cur_time, total_time, angle, gpiece, who, o, whom,
ox, num_guys` — so one capture gives the launch point directly, the offset is
`(sx − guy.x, sy − guy.y)` **measured** rather than derived, and five rows
(the Longbowman's two `CHAR_ATTACK1`, two `CHAR_ATTACK2` and one
`CHAR_ATTACK3` release events) pin the whole type. No dump on this disk has
`AMMO` enabled — run53, run97 and run100 all write zero `AMMODATA` blocks —
so this is a booking, not a grep. `tools/emu/callfn.py` is the other route and
is the more expensive one: `get_position` reads the `GraphicPieces` singleton,
which is an hour of synthesized state (`docs/EMULATOR.md`).

### 20.4 What is diff-backed here, and what is not

- **Diff-backed** (`run100_says_great_lakes_9451_is_two_arrows_on_one_farm`,
  and `run53_s_24000_frames_put_the_ceiling_where_run33_did` for the stream):
  that the first-wound roll is taken on a Farm — so on any building; that one
  frame can take it twice; that the gate is `damage` and not `damage_frac`;
  that the per-figure hit is thirteen sixteenths, which is §6's formula, §7.1
  step 5's two divisions and §7.2 step 4's accumulator end to end; and that
  the farm's damage record then tracks the original's, one landing behind.
- **Reading only**: the `+0x18b` flock draw and its `% 2 + 3` — no capture on
  this disk reaches a siege attacker's first wound on a fort, temple or town,
  and `Objects::add_flock` has never executed in any traced game. The
  `TEMPLE`/`TOWN` type constants (`0x1b5`, `0x19f`) are the listing's
  immediates and are certain; what is untested is the whole arm firing.
- **Reading only**: the war declaration below the flock, and the `param_7 < 0`
  jump that skips it.
- **Open, and bounded above**: the launch offset (§20.2, §20.3).

## 21. The captain mirror — how a squad gets its target (item 395, 2026-09-19)

**A squad member never searches for a target. Its whole think is copying
its captain's.** `Unit::think@005f6e40`'s *first* statement, above the
citizen mask-clear, above both cadence gates, above the auto-attack and
everything under it:

```text
if (!is_captain(this)) {                        // (ushort)o_up >> 15
    a = get_action(units[who][get_captain()]);
    if (a == 0) return;
    if (a->get_type() != 10) return;            // must be an ATTACK order
    o   = a->target_o;  who2 = a->target_who;   // +0x8 / +0xc
    if (!valid_target(this, o, who2)) return;
    add_attack_order(this, o, who2, QUEUE_NEW, a->mandatory, 0);
    return;
}
```

Five tests and a return, and the `return` is the function's — a non-captain
reaches nothing else in `Unit::think`. `Unit::think` is only called from
`Unit::do_idle`, so the mirror fires on the frames a member is idle and
costs nothing on the frames it is walking or swinging.

**It is not gated by anything the auto-attack is gated by**: not the stance,
not `unit_masks & 0x100`, not `idle == 1 || (o + frame) & 0x1f`, not the
mod-sixteen gate. It reads the captain's **action** (`UnitData::get_action`,
the intent under the pathing legs), so a captain already walking to a chase
point still hands its target down. `valid_target` is tested on the
**member**. The order is `QUEUE_NEW` carrying the captain's own `mandatory`
byte, and `add_attack_order`'s `action` argument is 0, so a DEFENSIVE member
still takes a post.

### 21.1 The frame, and it is measured twice

Both of chapter one's squads are three unit objects threaded `o_up`/`o_down`
by one `Objects::init_unit` (`docs/INPUT.md` §11), so each has exactly one
captain — `0/6` and `1/6`, the only two of the six that read `o_up -1`.

**who=1, the frame it is born.** The golden dump's block 616 — the end of
frame 615, the frame `add hoplite who=1 5,40` runs — already has all three
carrying `type 10 ox 7 whom 0 new_ord 1`, and `near_o` on the captain
alone:

| block 616 | `type ox whom` | `near_o/near_who` | `orders_x, orders_y` |
| --- | --- | --- | --- |
| `1/6` captain | `10 7 0` | **`7 / 0`** | `1368, 7992` |
| `1/7` | `10 7 0` | `-1 / -1` | `1512, 7992` |
| `1/8` | `10 7 0` | `-1 / -1` | `1416, 8136` |

The captain searched (§18.1's bracket is that search); the two members did
not, and they hold its answer on the same frame because `1/6` is processed
before them. Three units, one search — and §18's "only the captain
searches" is now a mechanism rather than an observation.

**who=0, one frame later.** The retaliation reaches the captain only
(§18.2), and who=0's members are processed *before* `1/6` fights, so they
mirror on the next frame: `0/6` carries the ATTACKORDER at the end of 616
and `0/7`/`0/8` at the end of 617. That one-frame stagger is the mirror's
signature and nothing else in the frame produces it.

### 21.2 Why it is not `Group::target_opportunity`

The item was booked as `Group::target_opportunity` handing the squad its
captain's target. It cannot: the loop at `007107d0` only enters a member
that is itself a **captain** (`o_up < 0`, `UnitData::is_captain`'s own
`(ushort)o_up >> 15`), and group 64's only captain is the asker `1/6`. What
the function does for this frame is hand the target *back to the asker* —
and by the arm at `00710964`, not the direct one: `1/6` has no action yet,
`order_type` is `NONE`, so it runs its own
`find_melee_target(min(dist + 0xc0, unit_respond_range × 0x240))`, whose
`add_order` argument is 1 and which is therefore where the captain's own
ATTACK order comes from. `docs/GROUPS.md` §13 carries the listing.

### 21.3 The same predicate lives in `find_melee_target` too, and is not implemented

`Unit::find_melee_target@005ff9c0`'s head is the mirror again, for callers
that are not `Unit::think` (`005ff9d8`–`005ffb9a`, read off the listing):

- `is_captain` is tested first; a captain falls through to `on_duty` and
  the search.
- a non-captain with `cavarch != 0` (`param_3`) skips straight to the
  search;
- otherwise the captain's action must exist and be kind **10**, and
  `valid_target` must pass on the member; then
  `(stance != 2 && <two `unit_masks & 0x2000000` / `+0x6c & 0x20000` pairs
  on the member and the captain>) || is_in_range(member, target)`;
- and with `add_order` (`param_4`) set it adds the order itself:
  **`QUEUE_FIRST`** when the member's own `order_type` is `ATTACK_TO` (2),
  `GROUP_ATTACK_TO` (0x15) or `GUARD` (0xc), **`QUEUE_NEW`** (2) otherwise
  — the listing's own `pushl $0x0` at `5ffb78` against `pushl $0x2` at
  `5ffb4f` — carrying the captain's `mandatory` byte;
- a failure of `valid_target` or the range disjunct falls to `on_duty` and
  the ordinary search, so the arm is a shortcut rather than a veto.

**Not implemented here**, and deliberately: with the `think` mirror landed
no non-captain reaches `think_attack` at all, and the one other caller that
would take this arm is `Group::action_attack`'s per-member
`find_melee_target` (`docs/GROUPS.md` §10), which runs all over the long
captures. No diff asks for it and the golden record cannot see it.
*Falsifier:* a `UNITS` window over an army group given a non-mandatory
`Group::action_attack` whose members are not captains, read for whether a
member takes `QUEUE_FIRST` where this crate gives it the outer
`add_attack_order` alone.

### 21.4 What stands at 624

The word moved 619 → **624**, the value diff 620 → 625, and blocks 616, 617
and 618 now carry the dump's own coordinates for all six units — including
the three the last item could not reach:

| block 618 (end of 617) | dump | ours, before 395 | ours, after |
| --- | --- | --- | --- |
| `1/7` `orders_x, orders_y` | `1320, 7800` | `1176, 7944` | **`1320, 7800`** |
| `1/8` `orders_x, orders_y` | `1176, 8088` | `1176, 8088` | `1176, 8088` |
| `0/6` `orders_x, orders_y` | `1080, 8280` | `1224, 7704` | **`1080, 8280`** |
| `1/7` position | `1471, 7954` | — | **`1471, 7954`** |
| `1/8` position | `1360, 8126` | — | **`1360, 8126`** |

§19's ring is answered from the right state now: `1/7`'s ordered point is
the one two of its six `find_ordered_collision` rejections hang on.

What parts at 624 is `Guy::set_anim+0x97a < Unit::move_step+0x823`, a
walking figure's step, and **two** residues on frame 618 can produce it.
Both are value diffs against the dump, not readings:

1. **who=1's army marches where the original's holds.** `0/7` now strikes
   `1/6` during 618 — the dump's `1/6` reads `damage 3 damage_frame 618` —
   and that hit reaches `Armies::emergency(1)`, whose bypass tick
   (`Army::process(·, 1)`) recounts the army, finds `num_standard` 1 for
   the first time, and issues `Group::action_siege_attack_to`: from 618
   this crate walks all three of who=1's hoplites toward
   `(38664, 13320)`, and the dump holds `1/6` on `(1368, 7992)` with
   `recharging` counting 32 → 27 and `1/7`/`1/8` on their own chases for
   the rest of the record. ~~**The emergency's gates are not the answer.**
   `Object::do_damage`'s gate at `0064bbfd` requires the *victim's* leader to read
   `leader_flags & 4 == 0`, and that bit is `LeaderData::is_human@006ec170`
   — one instruction, `return leader_flags & 4` — so the emergency is the
   **computer** leader's, who=1 is the computer leader, and it fires in the
   original too. The divergence is inside the tick.~~
   **Closed by item 399, §23, and the gates were exactly the answer.** The
   `leader_flags & 4` test above is real and it passes — but it is the
   *second* conjunct of the call's `if`. The **first** is `local_30`, set
   only by the city-alarm arms of `do_damage`'s **building** branch, so a
   unit taking a hit never reaches `Armies::emergency` at all. The
   falsifier below was run (`docs/RUNS.md` run110) and answered it: group
   `64` carries `army 0` and `order_num 0` on every block 616..629 — the
   army issues no order in the window.
   ~~*Falsifier:* chapter one re-captured with `ARMY` and `GROUPS` under
   `[End Frame]` over 610–630, read for who=1's army block at 618 — its
   `status`, `num_standard`, `target_o/target_who` and its group's order —
   which the present capture's categories do not print at all.~~ (`ARMY` is
   not an `[End Frame]` category at all: there is no such key in
   `gamelog.ini`, `GameLog::dump_armies@0092fc50` has no caller, and
   `ArmiesData::log_data` is reached only from `dump_all`. run110 took the
   `GROUPS` half, which settled it.)
2. ~~**`0/8` plans its chase a frame late, and to the wrong point.** The dump
   has `0/8` ordered to `(1224, 8280)` at the end of 618; this crate leaves
   it on its seat through 618 and plans `(1368, 7992)` — the target's own
   point, not a ring slot — at 619. `0/6`, which took its order a frame
   earlier, plans correctly at 617, so the lag is in *when* a freshly
   mirrored member reaches `do_attack`, not in §19.~~ **Closed by item 399
   with the same change**: `0/8` carries `(1224, 8280)` at block 619 now
   (§23.2). It was the emergency's march all along — `1/6` losing its seat
   moved what `0/8`'s chase was aimed at.

### 21.5 The bit the mirror's own return does not clear

`Unit::think`'s **shared** epilogue at `005f761a` is
`andb $-0x11, 0x8(%ebx)` — it clears `SubObjectData.flags & 0x10`, the
"could not reach" bit that `think`'s own mod-sixteen cadence gate reads
(§`docs/ORDERS.md`, "The global cadence gate"). The mirror's arm does not
reach it: `add_attack_order` at `005f6f71` is followed by the function's
**own** epilogue at `005f6f76`, and the three early returns above it jump to
`005f761e`, one instruction *past* the clear. So a non-captain never has the
bit cleared by thinking, and a unit that has just failed to reach something
keeps searching every frame until something else clears it.

This crate **never clears it at all** — `cant_reach` has three writers and
no reader but the gate (`grep cant_reach crates/sim/src/orders.rs`) — so the
mirror's return is faithful by accident and the captain path is not. Not
implemented, because the clear moves the cadence of every unit on both long
captures that has ever failed to reach anything and no diff asks for it.
*Falsifier:* a `UNITS` window over a unit ordered into ground it cannot
reach, read for whether its think interval returns to sixteen frames on the
frame after the failure.

### 21.6 Confidence, and what this has not established

Diff-backed: the mirror's existence and its timing on both squads (§21.1 is
the golden dump's own blocks 616–618, and the harness reproduces every
coordinate in §21.4's table); the golden word at 624. Read-only: §21.3's
`find_melee_target` head, and within §21 itself the `mandatory` byte — every
observation here has it 0, so nothing distinguishes "the captain's" from
"always 0". The two `unit_masks & 0x2000000` pairs in §21.3 have no
observation either. And the mirror has never been seen to *fail* its
`valid_target` test, so which of the five returns fires is only backed for
the success path.

## 22. Where an arrow starts, measured (item 396, 2026-09-19)

§20.3 left the launch offset open and bounded above, and named the capture
that would close it. run109 is that capture — `AMMO=5` over `[9420, 9480)` on
the Great Lakes seed, sixty blocks, `docs/RUNS.md` run109 — and it closes it
by subtraction rather than by a `.bh3` skeleton reader.

**Two things the booking said were wrong, and the first cost nothing to find.**
The ammo block is named **`AMMO`**, not `AMMODATA`: `AmmoData::log_data@00679c00`
hands `Log::begin` the string at `int_str_array + 0x9ec`, the table's stride is
a 20-byte `String`, and 0x9ec/20 = 127 indexes `Data/internal_strings.xml` —
where `UNITDATA` 7086, `OBJECT` 142, `BUILDDATA` 287, `GUY` 3940, `LEADERDATA`
4627, `WALLDATA` 7128 and `ANIMALDATA` 131 all land on their own names, seven
for seven. Under the right name **two archives already carried ammo records**:
run17 has 173 and run29 one. And `AMMO=1` would not have answered the question
either — `log_data` opens level 1 for `cur_time/total_time/who/o/whom/ox` and
**level 2 for `sx, sy, sz, ex, ey, ez, angle`**, so the brief's setting drops
the launch point silently.

### 22.1 The table, and every row is measured

run109 prints nine arrows from three Longbowmen at two facings, and run100
already carried the shooters' guy records at `GUYS=4` — position, angle,
`cur_anim`, `cur_time`. The offset is `(sx − guy.x, sy − guy.y)`:

| block | shooter | anim | `t` | guy | `s` | offset | `total_time` |
|---|---|---|---|---|---|---|---|
| 9426 | `1/29` | `CHAR_ATTACK3` | 10 | 4680, 29928 | 4613, 29951 | −67, +23, +185 | **27** |
| 9427 | `1/28` | `CHAR_ATTACK2` | 4 | 4776, 30168 | 4708, 30215 | −68, +47, +137 | **26** |
| 9440 | `1/29` | `CHAR_ATTACK3` | 24 | 4680, 29928 | 4620, 29954 | −60, +26, +184 | **26** |
| 9445 | `1/28` | `CHAR_ATTACK2` | 22 | 4776, 30168 | 4703, 30204 | −73, +36, +169 | **26** |
| 9452 | `1/29` | `CHAR_ATTACK1` | 6 | 4680, 29928 | 4609, 29969 | −71, +41, +169 | 26 |
| 9457 | `1/28` | `CHAR_ATTACK2` | 4 | 4776, 30168 | 4708, 30215 | −68, +47, +137 | 25 |
| 9463 | `1/27` | `CHAR_ATTACK2` | 4 | 4584, 29784 | 4523, 29840 | −61, +56, +137 | 25 |
| 9468 | `1/29` | `CHAR_ATTACK1` | 22 | 4680, 29928 | 4609, 29969 | −71, +41, +169 | 25 |
| 9475 | `1/28` | `CHAR_ATTACK2` | 22 | 4776, 30168 | 4703, 30204 | −73, +36, +169 | 25 |

**The four `total_time`s §20.2 is about come back 27, 26, 26, 26** against this
crate's 27, **27**, 26, **27** — the two that were a frame long are `1/28`'s,
exactly as predicted before the run.

**The six starttimes are every one the type has.** `unit_graphics.xml` gives
`LONGBOWMEN` six `<RELEASEEVENT>`s — 466 and 1533 ms on `CHAR_ATTACK1`, 333 and
1533 on `CHAR_ATTACK2`, 733 and 1666 on `CHAR_ATTACK3` — which through §9.0's
measured `ms × 3 / 200` truncation are frames 6, 22, 4, 22, 10, 24. The capture
caught all six, so the Longbowman's table is complete rather than sampled.

### 22.2 It is a rotation, and the proof is `1/27`

`1/27` fires `CHAR_ATTACK2` at `t 4` from a **different facing** to `1/28`'s —
`−1449000960` against `−1348206592`, 8.45° apart — and its world offset is
`(−61, +56)` where `1/28`'s is `(−68, +47)`. Same node, same length (82.8
against 82.7), different answer. So `GraphicPieces::get_position`'s rotate-by-
facing is real and the entry is model-space, exactly as §20.3 read it.

The same shape was already on the disk and nobody had looked: run17's 24
Slinger shots (type 82), rotated back by each shooter's own angle, collapse
onto **three** model-space vectors — `(+39.4, −118.0)` at `dz 151`,
`(−42.9, −112.3)` at `dz 177`, `(+68.2, −68.2)` at `dz 178` — each held to ±2,
which is the integer quantisation of `sx` and `guy.x`. That is the confirmation
this section would otherwise have had to buy.

### 22.3 The port's table, and why it is not a float

`crates/sim/src/launch.rs`. The entry is stored the way the rest of the crate
stores a direction — a bearing **relative to the guy's facing** and a radius —
so the world offset is `movement::sin_component`/`cos_component` and nothing
else. `docs/DECISIONS.md` entry 16's second shape: the original builds its
`GraphicPieces::positions` array once at load with floats, and this is the same
array with the building step replaced by a measurement.

| anim | `t` | bearing | radius | `dz` |
|---|---|---|---|---|
| `CHAR_ATTACK1` | 6, 22 | −26,692,241 | 81 | 169 |
| `CHAR_ATTACK2` | 4 | −136,574,224 | 82 | 137 |
| `CHAR_ATTACK2` | 22 | −36,624,995 | 80 | 169 |
| `CHAR_ATTACK3` | 10 | +106,731,108 | 70 | 185 |
| `CHAR_ATTACK3` | 24 | +59,520,002 | 64 | 184 |

These reproduce **all nine** measured launch points to the unit, across three
units and two facings — `launch::tests::run109_launch_points_are_reproduced_to_the_unit`
carries the dump's own columns. The key is `Guy::gpiece`, which is **127** for
`LONGBOWMEN` (the piece, not the type 177) and is the same key `PieceReleases`
is built on; a piece with no row launches from the unit's own position, which
is what every shot did before run109.

`dz` is recorded because the record prints it and nothing reads it: the flight
time is a plan distance.

### 22.4 What it moved, and what it did not

**Great Lakes' long-capture word: 9451 → 9510.** With the offset live, all
**nine** of this crate's launches inside run109's window reproduce the
original's `sx, sy`, its `ex, ey` **and** its `total_time` exactly — 9425,
9426, 9439, 9444, 9451, 9456, 9462, 9467 and 9474 — and the seventh draw at
9451 is `1/28`'s second arrow arriving on the frame it always arrived on.

**Three tiers of evidence, and the assertions keep them apart.** Below the
word this crate launches **sixteen** arrows where it used to launch four, and
every one of the sixteen frames is the *original's* — that is
`run53_s_24000_frames_put_the_ceiling_where_run33_did`'s equality against the
trace's own `Ammo::init` draw sites, not a list of ours. What differs is how
much of each launch anything on this disk can check: the nine at or below
9478 have all five numbers from run109; the seven above it are past run109's
window (`[9420, 9480)`, and an arrow first prints in the block after it
launches), so the draw sites say the original launched there and nothing says
from where, to where or for how long. Nothing is ours alone.

- **Diff-backed**: the nine launches' launch point, landing point and flight
  time, re-read from the archive every run by
  `run109_says_great_lakes_s_launches_land_where_the_bow_hand_aims`; the
  sixteen launch frames, against run53's trace; and the word.
- **Pinned rather than re-read**: `launch.rs`'s own unit test carries the
  dump's columns as literals, so it cannot notice the archive changing. The
  diff test above can, and does. ~~**What is still owed is the rest of the
  record** — `AmmoData::log_data` prints twenty-five fields and this reads
  five, where the rule is to diff the whole record.~~ **Paid by item 402,
  §24** — and the field count is twenty-**seven**, not twenty-five: fifteen
  are now compared by value on all 183 blocks, ten are pinned as the
  original's, and `Node::dz` — recorded here and read by nothing — is
  checked against run100's guy record in §24.2.
- **Not established**: every other unit type. The table has one piece in it.
  The Slinger's three families are measured in run17 and are *not* in the
  table, because no capture ties them to an animation — run17's `GUY` detail
  is 1, so it prints no `cur_anim`. A `GUYS=4` re-run of any Slinger fight
  would close that, and the same shape closes every missile type.
- **Not established**: whether the bearing/radius pair is what the original
  stores. It is not — the original stores a model-space `Vector<float>` and
  a `Transform` — but it is what reproduces the original's integers, and the
  two cannot be told apart from nine samples.
## 23. The emergency is a city alarm (item 399, 2026-09-19)

**`Armies::emergency` is not reached by hitting a soldier.** That is the
whole of this section, it closes §21.4's first residue, and it **corrects
both of that residue's predecessors**: item 391 booked "the emergency is
not reached" for the wrong reason, item 395 corrected it to "the emergency
IS reached, its gate reading the victim's leader" — and 395's gate is real
but it is the *second* conjunct of two.

`Object::do_damage@0064a480:951`:

```
if (local_30 != 0 && (leaders[param_2].leader_flags & 4) == 0)
    Armies::emergency(param_2)
```

`param_2` is the **victim**'s owner — `do_damage(A, o, who, …)` (§7.1), so
`this` is the attacker and `this->field_0x9` its owner. The second conjunct
is `LeaderData::is_human@006ec170` (`return leader_flags & 4`), and the
golden record's own `LEADERDATA` blocks settle who is who: **who=0 carries
`leader_flags 0x7`** (`& 4` set, the human) and **who=1 `0x800013`**
(`& 4` clear, the computer), so that conjunct passes at 618 exactly as 395
said.

**`local_30` is what fails.** It is zeroed at `0064a8dc` and has exactly
**two** writers before the test, both inside the *building* branch — the
`else` arm that resolves the target through vslot `0xac` to a `BuildData`
and reads `+0x72 city` — and both beside a city alarm: the non-capital
city's `S_CITY_BEING_ATTACKED` (under `city_flags & 0x10 == 0`, the alarm
threshold `local_34`, and `300 < frame − city.attack_stamp`) and the
capital's `S_YOUR_CAPITAL_ATTACKED`. `docs/ARMY.md` §15.8 carries the full
predicate and what this crate implements of it.

At golden 618 `0/7` strikes `1/6` — a **unit**. `local_30` stays 0, no
army of who=1 ticks, and nothing marches.

### 23.1 The group pool says it directly (run110)

The golden captures print no `GROUPDATA` at all, so this was captured:
chapter one again, `end:MISC=9,UNITS=9,GROUPS=9,GUYS=9,LEADERS=1` over
`[610, 630)` (`docs/RUNS.md` run110). Two live groups in the whole game,
both who=1:

| slot | `who` | `num` | `army` | `order_num` | `form` | `form_num` | members |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 64 | 1 | 3 | **0** | **0** on every block 616..629 | −1 | 0 | `1/6`, `1/7`, `1/8` |
| 65 | 1 | 1 | −1 | 2, unchanged | 0 | 1 | `1/0`, the explorer |

Two things fall out and both had been guessed the other way.

- **Membership was right.** The three hoplites *are* in who=1's army 0
  (`army 0`), so `Unit::add_to_army` at 615 and `docs/ARMY.md` §4.2's
  `think_attack` head are doing the right thing. The `group 64` the
  `UNITDATA` record carries is the army's group.
- **The army issues nothing.** `order_num` is stepped by every
  `Group::action_*` (`docs/GROUPS.md` §10), and it is 0 on all fourteen
  blocks — so `do_forming`, `march_to_target`, `engagement` and
  `send_here` are all unreached in the window. With the army's own cadence
  frames at 508 and 764 (`docs/ARMY.md` §5), the window has no tick in it
  at all, which is exactly what "the emergency never fired" predicts.

`1/0`'s `form_mod 50` — which had looked like the army's formation, and is
not — belongs to group 65, whose `army` is −1: the explorer's own group.

### 23.2 The word, and what it bought

The word **fell 624 → 621**, and the value diff is why it lands anyway.
Blocks 616, 617, 618 and 619 now carry the dump's own coordinates for all
six units; before, three of the six were on a march to `(38664, 13320)`
from 618 and further wrong every frame:

| block 619 | dump | before 399 | after |
| --- | --- | --- | --- |
| `1/6` pos | `1368, 7992` | `1379, 7970` | **`1368, 7992`** |
| `1/6` `orders_x, orders_y` | `1368, 7992` | `38664, 13320` | **`1368, 7992`** |
| `1/7` pos | `1451, 7935` | `1471, 7954` | **`1451, 7935`** |
| `1/7` `orders_x, orders_y` | `1320, 7800` | `38760, 13224` | **`1320, 7800`** |
| `1/8` pos | `1332, 8121` | `1360, 8126` | **`1332, 8121`** |
| `1/8` `orders_x, orders_y` | `1176, 8088` | `38568, 13416` | **`1176, 8088`** |
| `0/8` `orders_x, orders_y` | `1224, 8280` | `936, 7944` | **`1224, 8280`** |

The last row closes §21.4's *second* residue as well: `0/8`'s chase is no
longer a frame late and no longer aimed at the target's own point.

The three frames the march was buying were a draw stream agreeing on a
destination the original never takes — the case `CLAUDE.md`'s "a word that
moved lands with the value diff beside it" exists for, read in the other
direction.

### 23.3 What stands at 621, and it is located

`Guy::set_anim+0xf2f < Guy::move+0x166`, one draw this crate spends at
frame 621 and the original does not. The residue under it is a **short
leg**, and it is not combat:

- `1/8` ends its leg at `(1332, 8121)` on frame 619 and `1/7` at
  `(1431, 7915)` on 620 — both about **171** from their ordered points,
  under the `0xc0` the parked-collider arm allows — where the dump walks
  both to the end, `1/8` reaching `(1176, 8088)` at block 626 and `1/7`
  `(1320, 7800)` at 626. `0/8` does the same at `(972, 7988)` on 620.
- ~~That is `docs/COLLISION.md` §5.1: `do_move`'s waypoint probe widens
  `tolerance` to `other.big_radius × 3` against a **parked** collider…
  so for it to fire at 619 the waypoint was *re-taken* mid-leg.~~
  **Falsified, item 405 — and by the falsifier this section named.**
  run110's own `UNITS=9` records answer it in one `grep`: `tolerance` is
  **0** and `collide`, `collide_o`, `collide_who` and `collide_frame` are
  all clear for `1/7` and `1/8` on every block 616..629, in the dump
  *and* in this crate. §5.1's widening never fired on either side and the
  waypoint re-take is not implicated.
- **What ends both legs is `do_move`'s own ATTACK-action block, and it is
  a *ranged* attacker's** — see §24 and `docs/ORDERS.md` §4.4.

### 23.4 Confidence

Diff-backed: that no order reaches group 64 in `[616, 630)` (run110's
pool); that the six units' coordinates are the dump's through block 619
(the harness reproduces §23.2's table); the golden word at 621; and which
leader carries the human bit (the dump's own `LEADERDATA`). Read-only:
the alarm's damage threshold and its 300-frame cooldown (`docs/ARMY.md`
§15.8), neither implemented; and the claim that `local_30` has no third
writer, which is a read of one function rather than a run — a capture in
which an AI leader's **unit** is hit and its armies' `target_o` goes to
`-1` would falsify it.

## 24. The `AMMO` record, whole (item 402, 2026-09-19)

§22 read five numbers out of run109 and left twenty-two unread. This is the
rest, on the same capture and with no new one: `AmmoData::log_data@00679c00`
prints **twenty-seven** fields, item 396 compared five of them at each
arrow's first block, and `rondata::diff::ammo` now compares the whole record
on **all 183 blocks** of the window.

`CLAUDE.md` says why this comes before the next residue row: *when the
original dumps a record, diff the whole record*. What the widening actually
bought is below — and the honest headline is that it bought no ticks. The
Great Lakes word did not move, because the fourteen fields this crate holds
were already the original's on every one of the 183 blocks. **A widening
that passes first time is the outcome the rule is for**: the five numbers
§22 checked are now fifteen, and nine first-blocks are now 183 blocks.

**The count of fields is twenty-seven, not the twenty-five §22.4 and the
queue both said.** `log_data`'s own unwind indices run `0x00`..`0x1a`, and
every block on the disk carries twenty-seven `key value` lines. Nothing
turned on the number; it is recorded because two documents had it wrong and
the re-count cost a `grep`.

### 24.1 The five levels

`log_data` opens a detail level (`Log`'s vtable `+0x28`) before each group,
and the whole record is gated on `this->flags & 3` — an ammo with neither
low bit prints nothing at any level.

| level | fields |
|---|---|
| 1 | `cur_time`, `total_time`, `who`, `o`, `whom`, `ox` |
| 2 | `sx`, `sy`, `sz`, `ex`, `ey`, `ez`, `angle` |
| 3 | `traj`, `v1z`, `dx`, `start_roll_angle`, `bank_dx`, `bank_dy`, and a nested `SplineData` block when `ammo_path` is set |
| 4 | `flags`, `rolling`, `gpiece`, `graph_index`, `splash_area`, `index`, `num_guys` |
| 5 | `accuracy` |

`who`/`o` is the **shooter** and `whom`/`ox` the **target**, the same
`(player, object)` pair every other record here is keyed by; run109's nine
arrows are `1/29`, `1/28` and `1/27` shooting `0/2004`, the farm §20 and §22
are about.

### 24.2 Fifteen of the twenty-seven are compared by value

`run109_says_every_ammo_field_this_crate_models_is_the_original_s` builds
the sim from run53's initial state, runs it to 9479, and compares
**fourteen** columns on every block: `who`, `o`, `whom`, `ox`, `sx`, `sy`,
`ex`, `ey`, `angle`, `cur_time`, `total_time`, `splash_area`, `num_guys`,
`accuracy`. All 183 agree, field for field.

The fifteenth is `sz`, and it takes a second capture.
`run100_and_run109_agree_on_where_the_bow_hand_is_off_the_ground` reads the
shooter's guy record out of **run100** at each launch block — `GUYS=4`, so
position including `z`, `cur_anim`, `cur_time` and `gpiece` — and asserts
`guy.z + launch::node(gpiece, cur_anim, cur_time).dz == sz`. That is the
only thing that reads `Node::dz`, which §22 recorded and nothing consumed,
and it exercises the table's key as well as its value: the lookup item 396
actually got wrong — the unit type 177 in place of the piece 127 — returns
`None` here and the test names it. Three shooters at three heights (538,
569, 585) and all six of the Longbowman's release events.

**What the widening adds beyond the extra columns is the arrow's lifetime.**
§22 compared nine first blocks; this compares every frame of every flight.
`cur_time` runs 1 to `total_time − 1` and the record then stops, because
`Ammo::inc_time@0067d380` lands and frees the shot on the frame its clock
would reach `total_time` — a0 launches on 9425 with `total_time` 27, prints
9426..9451, and lands on 9452 as the first half of the farm's
twenty-six-sixteenth step that §20's test asserts from the other end.
`crates/sim`'s `process_projectiles` is the same predicate and the counts
match, 183 against 183.

### 24.3 Ten are the engine's, and they are pinned rather than left silent

`run109_s_ammo_record_pins_the_thirteen_fields_nothing_here_models` asserts
the original's own values for every field `sim::combat::Projectile` has no
room for, so a residue row says so when the archive changes instead of going
uncompared. Eight are constant across all 183 blocks:

| field | value | what it is |
|---|---|---|
| `ez` | 548 | the ground under the farm — the one target in the window |
| `traj` | 1 | and so no block carries a nested `SplineData` |
| `start_roll_angle` | 0 | the roll/bank triple, for a model that spins in flight |
| `bank_dx`, `bank_dy` | 0.0 | ditto |
| `flags` | 2 | `log_data`'s own gate is `flags & 3`; these carry bit 1 |
| `rolling` | 0 | |
| `gpiece` | 60157 | the **ammo's** graphic piece, against the Longbowman's own 127 |

and two vary — `index` and `graph_index`, which §24.4 is about. `sz`, `v1z`
and `dx` are in the same test because the struct has no room for them
either, but all three are tied back to this crate's integers (§24.2, §24.5)
and are not residue.

### 24.4 The pool: `index`, `graph_index`, and the order a frame prints in

`Objects::add_ammo@00658b10` scans its pool from slot 0 and **breaks at the
first whose `flags & 3` is clear** — the lowest free slot, growing the array
only when every slot is taken — then calls `Ammo::init@0067bbf0` with that
slot as one argument and `objects+0x1f8` as the next, incrementing
`+0x1f8` after. So the two indices are different animals: `index` is a
**reused slot** and `graph_index` a **monotone count of every ammo the game
has created**. `GameLog::dump_ammo@0092fe40` then walks the same pool
`0..count` and skips the slots whose `flags & 3` is clear, which is why a
frame's blocks ascend in `index` *with gaps* rather than running 0, 1, 2.

Every part of that is measured, not taken on the reading. Replaying "take
the lowest slot no live arrow holds" over the nine launches gives 0, 1, 2,
3, 4, **0, 1, 2, 3** — four reuses, all four right. `graph_index` runs 0..8,
which says these are the first nine arrows of the game and agrees with the
trace's own `Ammo::init` draw count. No frame's blocks descend, and 9463
carries five at once.

**This crate does none of it, and that is a named residue with a
consequence.** `Sim::projectiles` is a `Vec` that `process_projectiles`
`swap_remove`s from, so after the first landing its order is neither launch
order nor slot order: at 9453 it holds a4, a3, a2 where the original's pool
reads a2, a3, a4. Nothing in run109's window turns on it — no two arrows
land on the same frame after 9451, and landing is what draws — but a frame
where two do would spend the original's `Random::get` draws in the other
order. A slot pool in `crates/sim` is the successor, and it is a mechanism
rather than a widening.

### 24.5 The two floats, and both halves confirmed

`Ammo::init` ends with the trajectory:

```text
v1z = ((ez - sz) - GRAV_Z * 0.5 * T * T) / T
dx  = sqrtf(dx*dx + dy*dy) / T
```

with `GRAV_Z = -10.4875`, set as a literal in
`GraphicPieces::init@008ffcc0`. (The decompiler drops `sqrtf`'s result into
a discarded temporary — the FPU-stack artefact `tools/ghidra/README.md`
warns about — so `field_0x58 = fVar22 / local_28` with `fVar22` still the
*sum of squares* is that artefact, not the engine dividing an area.)

Neither is a number this crate can hold: no float in the simulation, and a
ballistic `z` is one of the few places the original genuinely needs one. So
the assertion is the honest weaker one — that the original's printed floats
*are* those formulae over the integers this crate does reproduce — computed
in integers at a millionth, to within a float32 ulp at their own magnitude.
Fitting `GRAV_Z` from the nine arrows alone gives −10.487500 ± 1e-6 across
three different flight times, which is the constant the decompile names.

It earns its place twice. It is the only check on `total_time` that does not
go through the launch point, and it is what says the flight is a **plan**
distance: `dx × T` is `hypot(ex − sx, ey − sy)` and not the 3-D length,
which for a0 would be 2722 against 2714.

### 24.6 Coverage

- **Diff-backed** (all nine arrows, all 183 blocks, re-read from the archive
  every run): `who`, `o`, `whom`, `ox`, `sx`, `sy`, `ex`, `ey`, `angle`,
  `cur_time`, `total_time`, `splash_area`, `num_guys`, `accuracy` against
  this crate's own state; `sz` against run100's guy record and §22's table;
  `v1z` and `dx` derived from this crate's endpoints and flight time.
- **Pinned, not modelled**: `ez`, `traj`, `start_roll_angle`, `bank_dx`,
  `bank_dy`, `flags`, `rolling`, `gpiece`, `index`, `graph_index`. The last
  two have a read-confirmed rule and a reproduced replay; the other eight
  are constants of this window and nothing more.
- **Read-only, no run behind it**: the nested `SplineData` branch of the
  walk. All 183 blocks print `traj 1` and carry none, so the parser's
  by-indentation nesting is written from `log_data` and exercised by a
  hand-built body (`the_walk_survives_a_sibling_and_a_nested_spline`),
  labelled synthetic in its own doc comment. A capture with a
  spline-following ammo — a catapult stone, anything whose `ammo_path` is
  set — is what would make it evidence.
- **Not established**: whether `flags` bit 1 means anything beyond "print
  me", what `rolling` and `start_roll_angle` drive, and whether
  `graph_index` is ever reused — nine allocations from a cold pool cannot
  see a wrap.
- **Not established**: every other ammo type. This is one piece firing one
  arrow at one building, and `traj 1` is the only trajectory the window
  contains.

**The parser has teeth, and they were tested by being made to fail.**
Reverting the flush-before-open that item 396 landed fails all six of
run109's tests; reverting the by-indentation nesting fails the synthetic
one; a one-off perturbation of `angle` prints the first parting row and its
neighbours; and dropping a block per frame trips the count assertion with
"the arrows' *lifetimes* differ, not their numbers" before any value is
compared. **Assert a count before you assert a value** is the whole of what
kept item 396 honest, and it is asserted here at two levels: the number of
blocks, and the number of fields in each.

---

## 25. The chase a melee unit does not drop (item 405, 2026-09-19)

§23.3's residue, settled — and settled the way this project keeps asking
for: the falsifier that section wrote down cost a `grep` over a capture
already on disk, and it said the named mechanism was wrong before anything
was read.

### 25.1 What the record says

run110 prints `tolerance`, `collide`, `collide_o`, `collide_who` and
`collide_frame` on every `UNITDATA` at `UNITS=9`. For who=1's `1/7` and
`1/8`, on every block 616..629:

| field | dump | this crate, before 405 |
| --- | --- | --- |
| `tolerance` | 0 | 0 |
| `collide` | 0 | 0 |
| `collide_o` / `collide_who` | −1 / −1 | clear |
| `collide_frame` | −1 | never stamped |

No collision is detected for either unit anywhere in the window, on either
side. `docs/COLLISION.md` §5.1's parked-collider widening cannot have ended
a leg it never ran on, and `0xc0` never appears in `tolerance`.

### 25.2 What does end them

`do_move`'s action-under-the-move block (`docs/ORDERS.md` §4.4 step 4),
which this crate entered for every type. The original gates the whole of
it on the attacker's **raw `max_range` column** —
`do_move@005f7b30:212`, `*(int *)(*(int *)&this->field_0x18 + 0x1fc)`,
which is `SubObjectData +0x18 ptype` → `ObjectTypeData +0x1fc max_range`
by the type record, with `ObjectType::backup@0065fac0` confirming the
offset in the engine's own assignment. §4.4 carries the full predicate and
the three conjuncts still unmodelled.

The frame is a coincidence of one unit. A HOPLITES-line attacker reaches
`0xf6` = 246 (§13.2, item 384), and on frame 619 `1/8` at `(1332, 8121)`
has `attack_dist` **246** to `0/7` — in range by a single unit. So the
kill fires, 171 short of `(1176, 8088)`; `1/7` follows at 240 a frame
later. Without the gate the two legs die there; with it they run to the
end.

### 25.3 The word, and the value diff beside it

**621 → 626.** The value diff is the golden dump's own coordinates, and it
now agrees on **all six units through frame 624** — where before 405 three
of the six parted at 619 or 620:

| block | `1/7` dump = ours | `1/8` dump = ours | before 405, `1/7` / `1/8` |
| --- | --- | --- | --- |
| 620 | `1431, 7915` | `1304, 8116` | `1431, 7915` / `1332, 8121` |
| 621 | `1411, 7895` | `1276, 8111` | `1431, 7915` / `1320, 8136` |
| 623 | `1371, 7855` | `1220, 8099` | `1416, 7896` / `1320, 8136` |
| 624 | `1351, 7835` | `1192, 8093` | `1416, 7896` / `1320, 8136` |
| 626 | `1320, 7800` | `1176, 8088` | `1416, 7896` / `1320, 8136` |

`orders_x/orders_y` hold `(1320, 7800)` and `(1176, 8088)` throughout on
both sides. **§21.4's second residue closes with it**: `0/8` was never
planning its chase a frame late in its own right — it was the same short
leg, and it now matches the dump frame for frame from 619 to 624,
reaching `(1044, 8076)` where it had stopped at `(972, 7988)`.

The cost is on the other map and 23,375 frames past this one: Great Lakes'
endpoint at 24001 goes **48 off → 51**, re-pinned in `ENDPOINTS` under
DECISIONS 36, which asks for the number rather than a trade. Every melee
unit in a 24,000-frame game now walks a leg it used to drop, so that row
is evidence about the run-up and not about the predicate.

### 25.4 What stands at 626

The **arrival frame itself**. Both hoplites land exactly on their ordered
points on 626 and hold; the original spends `Guy::set_anim+0xf2f <
Guy::move+0x166` there and this crate does not, and `Guy::set_anim+0x104b`
is ordered differently beside it. So the residue is in what an arriving
figure rolls, not in where it arrives — §21.4's `0/8` is the other live
one, whose `orders_x/orders_y` the dump snaps to `(1044, 8076)` at block
625 with `collide_o 8` while this crate walks on toward `(1224, 8280)`.

### 25.5 Confidence

**Diff-backed**: the gate itself (the golden word 621 → 626 and §25.3's
table, reproduced by the harness on every commit); that no collision is
detected for `1/7` or `1/8` in the window (run110's own fields); the
`0xf6` reach (§13.2, item 384). **Read-only**: the three unmodelled
conjuncts of the kill and the re-path arm's two guards, all listed in
`docs/ORDERS.md` §4.4 — each of them only makes the kill rarer, so none
can be costing a frame in the same direction.

**Not established**: whether `find_melee_target`'s retarget really is
inside the `max_range` gate, as the decompile has it. It reads oddly — the
arm is named for melee — but it is also inside the unit-target branch and
guarded by `attack->+0x1c == 0`, and nothing this crate models reaches it.
*Falsifier:* a capture with an Archer or Slinger squad walking onto a
target under an `ATTACK` action, read for a target change on an
`(o + frame) & 0x8000000f == 0` phase.

## 26. Why a cheat-spawned melee captain engages a frame early — a hypothesis (item 434, 2026-09-19)

**This section is a hypothesis, and it is one because fourteen booked
mechanisms have been wrong in two days while the frame held every time**
(`docs/DECISIONS.md` 42). What it settles is what the divergence is *not*;
what it proposes is one constant and one unresolved arm, with the check
that would decide between them named and unrun.

The frame is chapter two's **616** — the rules headline
(`docs/GOLDEN.md` §6, item 426). The original issues no attack order there
at all; this crate gives the hoplite squad one on its birth frame, and
`Unit::fight`'s one-in-five re-search then spends a twenty-sixth draw
against the original's twenty-five.

### 26.1 What run112 measures

| fact | value |
| --- | --- |
| first `ATTACKORDER`, slingers `0/9`–`0/11` | **621**, their own birth frame |
| first `ATTACKORDER`, bowmen `0/6`–`0/8` and hoplites `1/6`–`1/8` | **635** |
| damage on who=1's hoplites through 650 | **none** — so 635 is a search, not retaliation |
| who=1's nearest unit outside the squad | **198 tiles** |
| LOS: Hoplites / Slingers / Bowmen | **6 / 8 / 11** |
| bowmen to the hoplite captain at 616 | **7.25 tiles** (1392 units) — not engaged |
| nearest who=0 unit at 634 | `0/10` at **6.57**; the target chosen is `0/11` at **6.85** |
| slingers to their target at 621 | **8.6 tiles** — engaged |

### 26.2 Three mechanisms the reading rules out

- **Visibility.** There is no fog or seen-test anywhere on the search path:
  `Unit::find_melee_target@005ff9c0`, `Object::find_nearby_target@00648da0`
  and `Object::valid_target@00648ba0` carry none between them. "who=1
  cannot see them" is a good story — who=1's only eyes are three Hoplites
  at LOS 6, 7.25 tiles short — and the decompile does not support it.
- **The gate's shape.** `Unit::think@005f6e40` reaches the search only when
  `idle == 1` or `(o + frame) & 0x1f == 0`; otherwise it takes a cheap
  remembered-target path and skips `think_attack` entirely. That is what
  this crate has, and the bowmen confirm the grid exactly: `o` 6 gives
  `(6 + 634) & 0x1f == 0`, and 635 is the dump's label for it.
- **The army join.** `Unit::think_attack@005f5a80` calls
  `Unit::add_to_army@005f7740` and falls through to the search regardless,
  so a join cannot suppress it.

### 26.3 The arithmetic, and the contradiction in it

The radius `find_melee_target` searches, aggressive stance:

```
r == 0 :  0x120
r != 0 : (r + 1) * 0xc0 + 0x180
then   : max(·, unit_respond_range * 0xc0)
then   : max(·, unit_respond_range * 0x180)   when the searcher's unit_masks
                                              carries the AI-driven bit
```

This crate's own numbers, printed rather than derived: **24 tiles** for the
hoplite captain, 13 for the bowmen, 12 for the slinger. The 24 is the
AI-driven arm at `unit_respond_range` 12.

- For the hoplites not to reach 7.25 **without** that arm,
  `unit_respond_range` must lie in `[6.85, 7.25)` — **exactly 7**.
- **With** it, `unit_respond_range * 0x180 < 1392` forces 3 or less, and
  then the plain floor cannot reach the 6.85 tiles the original *does*
  engage at.

Both cannot hold. Two supporting facts sharpen it rather than resolve it:
`UNIT_RESPOND_RANGE` **is not in `rules.xml` at all**, so this crate's 12
is a default and happens to equal the neighbouring
`UNIT_BUILD_RESPOND_RANGE`; and `UnitData +0x68` is `unit_masks` by the
type record, with the dump printing 262144 on who=1's hoplites and 0 on
who=0's — so the arm's own word is not misread.

### 26.4 The hypothesis, and why it explains the slingers

~~`unit_respond_range` is **7**, and the 6.85-tile lower bound belongs to
`think`'s *cheap* remembered-target arm — whose range test is
`ObjectData::is_in_range@006486b0` and not `find_melee_target` — rather
than to the search. If so the lower bound is vacated, a smaller
`unit_respond_range` becomes admissible too, and the whole divergence is
one constant this crate defaulted.~~ **Refuted by §27** (item 435): the
dump prints `near_o`, and it is −1 on who=1's captain through frame 634.
No remembered target existed, so the cheap arm cannot have run and the
lower bound stands. `unit_respond_range` 7 survives; the reason the arm
does not apply does not.

**The slingers are invariant under every value the hypothesis ranges
over**, which is the point. Their own term dominates —
`(6 + 1) * 0xc0 + 0x180` is 1728, nine tiles, against the 8.6 they engage
at — and who=0's units carry `unit_masks` 0, so the AI-driven arm never
applies to them. That is why only the melee squad is wrong, and why a fix
must not move them: `chapter_two_s_first_attack_orders_are_the_dump_s`
asserts six of its nine rows still agree, so making the hoplites right by
making the slingers wrong fails.

### 26.5 What would settle it, cheapest first

1. **Read the remembered-target arm** — whether a unit with no order can
   have `+0x34`/`+0x36` set, which decides whether the 6.85 datum is the
   search's at all. No run needed.
2. **Read `unit_respond_range`'s shipped value out of the PE globals.** It
   is a number, not an inference, and the recipe exists.
3. A `UNITS=9` window over run112 `[614, 640)`, which would print each
   unit's own target fields either side of both frames. No capture on disk
   carries them.

### 26.6 Coverage

**Diff-backed**: every row of §26.1 — they are run112's own dump and
trace, and `chapter_two_s_first_attack_orders_are_the_dump_s` holds the
engagement timeline against it. The three radii in §26.3 are this crate's,
printed from a probe.

**Reading-only, and owed a blind second reading**: all of §26.2, the
radius formula in §26.3, and the whole of §26.4. Nothing in §26.4 has been
run: no value of `unit_respond_range` has been tried against the floors,
and the one experiment that was tried — 12 to 7 — left 616 unchanged,
because the AI-driven arm re-raises the radius to 14 tiles. That is
evidence *for* the contradiction and not for the hypothesis.

## 27. The range `think_attack` passes — a belief, sharpened (item 435, 2026-09-19)

**§26.4's belief is refuted, and this section is its successor belief**, so
it carries the same label: nothing here has been run against the floors.
What the read adds is a branch this crate does not have at all, and an
arithmetic solution that is unique except for one element.

### 27.1 The refutation, from the dump rather than the decompile

`ObjectData +0x34` and `+0x36` are **`near_o` and `near_who`** by the type
record, and the dump prints them. On who=1's hoplite captain they are
**−1 from 616 through 634** and become `10` / `0` at **635**, the frame the
attack order appears.

So no remembered target existed before 635, `think`'s cheap arm cannot have
run, and **635 is the search's own result**. The 6.85-tile lower bound
stands, and §26.4's way out is closed.

### 27.2 The branch this crate does not have

**Corrected 2026-09-21, item 443 (§30.3), from the listing.** The branch
is real and §28.2's reading of which arm this captain takes stands, but
`iVar7` is **−1 at the call on every path**: the AI-driven arm runs
`add_to_army` and then `orl $-1, %esi` regardless. So `uVar12` selects
`add_to_army`, not a radius, and the zero path's table below — its two
grids and its 128- and 64-tile sweeps — is unreachable from
`think_attack`. Read the table as a description of `find_melee_target`'s
`param_1 == 0` block, which that caller never enters.

`Unit::think_attack@005f5a80` does **not** call the search with −1. It
computes, at its head,

```
uVar12 = ~(unit_masks >> bit 18) & 1        # 1 when NOT AI-driven
if (uVar12 == 0 && (leaders.list[who] & 2) == 0) uVar12 = 1
iVar7  = -(uVar12 != 0)                     # -1, or 0
```

and passes `iVar7` to `Unit::find_melee_target@005ff9c0`. Several later
arms force it back to −1 — a caravan, a military trainer, a type of 0x3d,
0x3e or 400, and `add_to_army` having run. This crate passes **−1 always**;
the zero path does not exist in it.

And the zero path is a different function, **cadenced by the frame**:

| when | radius |
| --- | --- |
| stance is defensive | `unit_defensive_respond_range * 0xc0`, floored by the unit's own reach |
| `(o + frame)` on the 1024-frame grid | max(24,576 units — 128 tiles, `(r+1) * 0xc0`) |
| `(o + frame)` on the 256-frame grid | max(12,288 units — 64 tiles, `(r+1) * 0xc0`) |
| otherwise | max(`unit_respond_range * 0x180`, `(r+1) * 0xc0`) |

Three radii and two grids, none of them modelled here. A unit that is
AI-driven under a leader carrying the flag sweeps the map twice on two
different periods and searches narrowly between.

### 27.3 The gate above it is identical, and that is measured

The dump prints `idle` (`UnitData +0xb0`) and `stance`. **Both captains
carry `idle 1, stance 0` on their birth frame** — the hoplite at 616, the
slinger at 621. So both reach the search by the same arm of the same gate,
this crate included, and the difference between them is only *which range
they search with*. That closes the last alternative to §27.2 that did not
need a reading.

### 27.4 The arithmetic has one solution, and one hole

Take `unit_respond_range` = **7** and the −1 branch without its final
AI-driven arm:

| searcher | own term | radius | the distance in question | verdict |
| --- | --- | --- | --- | --- |
| hoplite, reach 0 | 0x120 | 7 tiles | 6.85 in, 7.25 out | both ✓ |
| slinger, reach 6 | 1728 | 9 tiles | 8.6 in | ✓ |
| bowmen, reach 10 | 2496 | 13 tiles | engages on its own grid frame | ✓ |

Every measured datum fits, and 7 is the only integer that does. **The
hole** is the AI-driven arm: it would raise the hoplite to 14 tiles and
find the bowmen at 616. Its word is not misread — `UnitData +0x68` is
`unit_masks` by the type record and the dump prints 262144 on who=1's
hoplites against 0 on who=0's — and the zero path's own narrow arm has the
same shape, where no integer fits either (3 gives 6 tiles, 4 gives 8).

~~**So the belief is:** `unit_respond_range` is 7, and for these units the
AI-driven arm is not reached — most likely because they take the zero path
of §27.2 under a leader flag this dump does not print, and that path's
narrow arm is chosen by a grid neither 616 nor 634 sits on.~~ **Refuted by
§30.4**: 616 and 635 scan the *same object-grid cell*, so no value of
`unit_respond_range` and no choice of arm can separate them. The bracket
this subsection reads out of two distances was never a bracket on the
radius.

### 27.5 Why it still explains the slingers

who=0's units carry `unit_masks` 0, so `uVar12` is 1 unconditionally,
`iVar7` is −1, and they take the −1 branch with their own term dominating
— nine tiles against the 8.6 they engage at — while the AI-driven arm can
never apply to them. **The slingers are invariant under every value and
every branch this section ranges over**, which is why only the melee squad
is wrong, and `chapter_two_s_first_attack_orders_are_the_dump_s` asserts it
by requiring six of its nine rows to keep agreeing.

### 27.6 What would settle it

1. **A `LEADERS=9` window over run112.** The branch turns on
   `leaders.list[who] & 2`, and the dump at `LEADERS=2` does not carry the
   field. This is the one question here a capture answers and the reading
   cannot.
2. **`unit_respond_range`'s shipped value from the PE globals.** It is not
   in `rules.xml` at all, so the 7 is inferred from a bracket rather than
   read.

### 27.7 Coverage

**Diff-backed**: §27.1 and §27.3 entirely — `near_o`, `near_who`, `idle`
and `stance` are run112's own printed fields, and the timeline is held by
`chapter_two_s_first_attack_orders_are_the_dump_s`.

**Reading-only, and owed a blind second reading**: §27.2's branch and its
table, §27.4's arithmetic and the belief in it, §27.5's invariance
argument. No value of `unit_respond_range` and no branch has been run
against the floors.

## 28. The branch is settled and the arithmetic still does not close (item 437, 2026-09-19)

**The capture this section was booked for was not taken.** §27.6 named a
`LEADERS=9` window as the one question a reading cannot answer; run112's
own dump already carries the field, printed immediately *before* each
`BEGIN LEADERDATA` rather than inside it, which is why an earlier parse
missed it. The answer cost a grep.

### 28.1 `leader_flags & 4` is not the computer-leader test

Measured on three captures — chapter two's run112, chapter one's run105
and its AI-on control run104 — identical in all three, so it is a lobby
property and neither a chapter's nor `ai off`'s:

| leader | `leader_flags` | `& 2` | `& 4` |
| --- | --- | --- | --- |
| who=0, the **human** | `0x00000007` (run112 at 605) | set | **set** |
| who=1, the **computer** | `0x03000013` | set | **clear** |
| who=8, who=9, gaia | `0x02000007` | set | set |

`docs/INPUT.md` §11.4 reads `Unit::think@005f6e40`'s
`if ((leader_flags & 4) != 0 || ai_off != 0)` as the computer-leader gate,
and `docs/RUNS.md` run104's note says who=1 "still carries `leader_flags`
bit 2 at frame 899". **On this evidence the human carries bit 2 and the
computer does not.** Whatever bit 2 means, it is not "is a computer
leader", and every claim resting on that reading is owed a re-check. What
it *is* is not established here.

### 28.2 The branch, settled by the listing

The decompiler prints `think_attack`'s leader test as
`*(byte *)(leaders.list + who) & 2` — a byte index into an array whose
element is nearly 29,000 bytes, which cannot be right. `CLAUDE.md`'s rule
applies and the listing settles it in one command:

```
movzbl 0x9(%ebx), %eax          ; who
imull  $0x6eec, %eax, %eax      ; who * the LeaderData stride
testb  $0x2, 0xe3a390(%eax)     ; leader_flags, bit 1
cmovel %edi, %esi               ; if clear, uVar12 = 1
```

So it is proper indexing after all, on a stride of **28,396**.

~~where the type record gives `LeaderData` a size of **28,388** — an
eight-byte difference worth carrying, and not load-bearing here.~~
**Closed 2026-09-19, item 439, and it was never a discrepancy**: the
array's element is `Leader`, not `LeaderData`, and the type record gives
`Leader` **28,396** exactly. `docs/AI.md` already says why — `Leader` is a
shell over `LeaderData` that Ghidra did not populate, so a `field_0xNNN`
on a `Leader *` is a `LeaderData` field — and `docs/ANIM.md` already uses
28,396 as the stride. This section compared `Leader`'s stride against
`LeaderData`'s size, which are two different types. Nothing shifts:
`LeaderData` sits at the shell's offset 0, which the listing above
confirms by reading `leader_flags` at `base + who × 28,396 + 0`, and every
`leaders.list[who] + 0xNN` in `docs/` is therefore an unshifted
`LeaderData` offset.

who=1's `leader_flags` low byte is 19, so bit 1 is **set**, the `cmovel`
does not fire, and `uVar12` stays 0. ~~and **`think_attack` passes 0**.
who=1's hoplite captain takes the **zero path** of §27.2.~~ **Corrected
2026-09-21, item 443 (§30.3)**: what `uVar12 == 0` selects is
`Unit::add_to_army@005f7740`, not the range — `005f5d86`–`005f5da6` calls
it and then sets the range to −1 anyway, so **`think_attack` passes −1**,
on this path and on every other. The branch is settled, by the dump and
the listing together, with no capture; its consequence was misread.

### 28.3 And the arithmetic still does not close

Both frames in question sit on neither of the zero path's two grids —
`o + frame` is 621 at the birth frame and 640 at 634, and neither is a
multiple of 256 or 1024 — so both take the same narrow arm,
`max(unit_respond_range * 0x180, (r + 1) * 0xc0)`, and with reach 0 that
is `unit_respond_range * 0x180`.

The measured bracket is the search reaching its chosen target at **1340**
units on frame 634 and not reaching the bowmen at **1392** on the birth
frame. So:

| multiplier | needs `unit_respond_range` in | integer? |
| --- | --- | --- |
| `* 0x180` (both paths' wide arms) | [3.49, 3.62) | **none** |
| `* 0xc0` (the −1 path's plain floor) | [6.98, 7.25) | **7** |

~~**The multiplier that fits is the one neither path applies to this
unit.**~~ **Dissolved 2026-09-21, item 443 (§30.3–§30.4).** The arm that
applies is the −1 branch's, because from this caller that is the only
branch there is; and the table is beside the point either way, because
§30.4 shows 616 and 635 scan the same object-grid cell and therefore that
no radius can separate them. Resolving the branch made the contradiction
sharper rather than closing it, and this section stops here rather than proposing a third account: the
two that have been proposed — visibility (§26.2) and the remembered-target
arm (§26.4) — were each refuted by evidence fetched after they were
written, and a third built on the same footing would be worth less than
the bracket.

### 28.4 What is now known not to be the answer

Visibility (§26.2), the remembered-target arm (§27.1), the seating error
(§26 — and **withdrawn outright** by §29.2: it was five frames of
marching, not a ring) and the `0x40000` respond arm considered alone
(§26), the `idle`/`stance` gate (§27.3), the choice of branch (§28.2),
and — seventh, item 439 — **an
offset shift through the leader array**: the eight bytes §28.2 recorded
were `Leader` against `LeaderData` and not a disagreement at all, so no
offset any document computes through that array is displaced. ~~What
remains
unexplained is narrow and stated as a number: **a melee searcher on the
zero path behaves as though its radius were `unit_respond_range * 0xc0`
with `unit_respond_range` 7.**~~ **Superseded 2026-09-21, item 443.** Two
more entries join the list and between them they retire the programme:
**eighth**, the zero path — `think_attack` passes −1 on every path
(§30.3), so nothing reaches `find_melee_target`'s `param_1 == 0` block
from it; and **ninth, the radius itself** — the bowmen at 616 and the
slinger `0/10` at 635 sit in the *same* object-grid cell relative to a
searcher that never moves, so no radius can accept one and refuse the
other (§30.4). What remains is a target-acceptance predicate, and §30.5
names the two readings that survive.

### 28.5 Coverage

**Diff-backed**: §28.1's table entirely — three captures' own printed
`leader_flags` — and §28.3's two distances, which are run112's own
coordinates.

**Listing-backed, which is stronger than the decompile**: §28.2.

**Reading-only, owed a blind second reading**: §28.3's identification of
which arm each path applies, inherited from §27.2.

## 29. 616 widened whole, and what four items had assumed (item 441, 2026-09-19)

Four items chased a radius and the bracket did not move. `CLAUDE.md`'s
answer to that is to widen every dumped record on the frame and name the
cause before any mechanism. Run112's frames 612–640 compared whole —
every unit of both real players, every field the record carries, both
directions — and the result is one row wide and one row deep.

### 29.1 What the widening says

| frame | divergences | what |
| --- | --- | --- |
| 612–615 | **0** | nothing at all, either direction |
| 616 | 3 | who=1's `1/6`, `1/7`, `1/8`: order list **ours 1, theirs 0** |
| 617 | 3 | the same three: **ours 2, theirs 0** |
| 618 | 9 | the three positions part, and a **path** appears: `PathLength` ours 1, theirs 0 |
| 619–640 | 9 a frame | the same three, walking |

**who=0's nine units never diverge at all**, at any frame, in any field.
Neither does any other record. The whole of chapter two's divergence is
three units of one squad, and it begins exactly at the word.

### 29.2 The seating finding of item 415 is withdrawn

415 walked to 622, found who=1's squad 140 units west of the dump's own
cells on all three figures, and read it as `find_nearby_spot`'s ring going
wrong on clear ground — a finding this document carried in §26 and
`docs/GOLDEN.md` §6.

It is wrong. The seating is **exact** at 616 and 617; the positions part
only from 618; and from there this crate's squad walks west at **28 units
a frame** while the original's never moves. `622 − 617` is five frames and
`5 × 28` is 140. The measurement was right and the mechanism was invented,
which is the same failure mode as §26.2's visibility and §27.1's
remembered-target arm — a number that fits, explained before it was
bracketed. `chapter_two_s_squads_stand_where_the_dump_stands_them` now
asserts both halves separately: the seating is the dump's, and the drift
is arithmetic.

### 29.3 The cause, named from the data

At **616** this crate gives the captain an `ATTACK`; at **617** it pushes
a `MOVE_TO` in front of it — the chase; at **618** a path appears and the
squad sets off. The original's hoplites hold **no order at all** through
634, and therefore never move, never path, and never spend the
twenty-sixth draw.

So the whole of the divergence is one order, and everything after it —
the second order, the path, the march, the 140, the draw — is downstream.
That is the cause, and it is narrower than any of the seven accounts
§28.4 lists: those all asked *why the search found something*, and the
widening does not establish that a search ran at all. Item 435 established
that **635** is a search, from `near_o` turning from −1 to 10 on that
frame. ~~On 616 `near_o` stays −1 on both sides — so whatever gives this
crate its order at 616, it is **not** the path that writes `near_o`.~~
**Withdrawn 2026-09-21, item 443 (§30.2): this crate models no `near_o`
at all**, so there were never two sides to that comparison — and read on
the original's side alone the field says the opposite, that 616 is its
search too.

### 29.4 What this leaves

~~The question is no longer "what radius reaches 7.25 tiles". It is
**which producer of an attack order runs on a cheat-spawned captain's
birth frame and does not write `near_o`**.~~ **Answered and corrected
2026-09-21, item 443 (§30.1–§30.2).** The producer *is* the search, on
both sides: a `#[track_caller]` probe names `orders.rs:1588` for the
captain and the mirror for its two members, and the original's `near_o` —
written above the range test, so the search's footprint rather than the
order's — proves its captain never produced an order at 616 rather than
producing and dropping one. §26–§28's radius arithmetic is not wrong and
was not misdirected; it is simply **not the discriminator**, which §30.4
settles by measurement.

`chapter_two_s_first_attack_orders_are_the_dump_s` remains the oracle and
is unchanged: the timeline is what it was, and six of nine rows still
agree.

### 29.5 Coverage

**Diff-backed**: all of §29.1, §29.2 and §29.3 — they are
`crate::diff::harness::compare` run over run112's own records, and both
halves of §29.2 are asserted by a test. §29.4 is an inference from them.

## 30. The producer, named — and why no radius can be the answer (item 443, 2026-09-21)

§29 widened 616 and named the divergence as **one order on one frame**,
then asked which producer of an attack order runs on a cheat-spawned
captain's birth frame. This section answers that on both sides, and the
answer costs §26–§28's radius programme its subject: the two frames the
bracket was built from scan the **same object-grid cell**, so no value of
any radius can separate them.

Nothing here moved the word. `GOLDEN_WORD_CHAPTER_TWO` is 616 of 901,
sequence 616, first value disagreement 617 — the same three numbers §29
left.

### 30.1 This crate's producer, measured

A `#[track_caller]` probe on `Simulation::add_attack_order`, run over
chapter two's own walk, prints the whole of it:

| sim frame | unit | caller |
| --- | --- | --- |
| 615 | `1/6` | `orders.rs:1588` — `think`'s step-3 search |
| 615 | `1/7`, `1/8` | `orders.rs:1722` — `captain_mirror` |
| 620 | `0/9` | `orders.rs:1588` — the same search |
| 620 | `0/10`, `0/11` | `orders.rs:1722` — the same mirror |

(Sim frame 615 is the dump's frame **616**; the dump's label is the sim
frame plus one throughout this chapter.) Four things follow, and the
fourth is the one that matters:

- The order is the **captain's search**, not a mirror, not an order-queue
  artefact, and not a birth-time default.
- The two members mirror it **on the captain's own frame**, because the
  captain has the lower `o` and is visited first. The original does the
  same at 621, where all three slingers take theirs together — so the
  mirror's timing is *right*, and only the captain's frame is wrong.
- The slinger squad's rows are byte-identical in producer and frame to the
  original's. The same code is right five frames later.
- **So the producer is `find_melee_target`'s caller after all**, which is
  what §26–§28 assumed and what §29.4 retired.

### 30.2 §29.3's `near_o` argument is void, and the field says the opposite

~~On 616 `near_o` stays −1 on both sides — so whatever gives this crate
its order at 616, it is **not** the path that writes `near_o`.~~
**Withdrawn here.** This crate models no `near_o` at all: there is no such
field on `sim::Unit`, no writer anywhere under `crates/sim`, and
`crate::gamelog` leaves the whole `ObjectData` half — `near_o`,
`near_who`, `healing`, `visible` and the rest — unparsed on purpose
(`ledger.rs`'s rule is that a parsed field is a compared field). The
widening never compared it. "−1 on both sides" is the original's −1 set
beside nothing, and it carried an inference it could not support. That is
the same failure as §26.2, §27.1 and §29.2, one level up: this time the
*absence* of a number was explained rather than a number.

What the field does say, read from the original:
`Object::find_nearby_target@00648da0` writes `ObjectData::near_o` /
`near_who` (`+0x34` / `+0x36`) for **every** candidate that clears
`Object::valid_target@00648ba0` and `Object::check_target@00649e00` and is
nearer than the best so far — the write sits **above** the
`if ((param_1 < 1) || (dist <= param_1))` arm that decides whether the
candidate may be attacked, so it is not conditioned on the range at all.
The pair is reset to −1 when the circle is empty, and again after the loop
when the nearest acceptable candidate is beyond **`0xf00`** (3,840 units,
20 tiles).

So `near_o` is the search's **footprint**, and it is one-directional:

> an attack order out of `find_nearby_target` implies a `near_o` write,
> because choosing a target requires a candidate to have cleared the same
> two predicates that set it.

`1/6`'s `near_o` is −1 at 616 and the nearest enemy is 7.25 tiles, far
inside the 20-tile reset. **The original therefore never produced an
order at 616 — it did not produce one and drop it.** That is the item's
question, answered on `ObjectData +0x34`, frame 616, value −1. The dump
also shows the field behaving as described rather than as a target
record: at 635 `1/6` carries `near_o 10` while the order it takes is on
`0/11`, so nearest and chosen are genuinely two different things.

### 30.3 `think_attack` passes −1 on every path — the zero path is unreachable

~~`Unit::think_attack@005f5a80` does **not** call the search with −1 …
and the zero path is a different function, cadenced by the frame … Three
radii and two grids, none of them modelled here.~~ (§27.2) and
~~who=1's hoplite captain takes the **zero path** of §27.2~~ (§28.2) are
both **withdrawn here**. The branch §28.2 settled is real; what it selects
is not the range.

`005f5d86`–`005f5da6`, from the listing rather than the decompiler:

```
5f5d86:  testl  %esi, %esi          ; esi = the range so far
5f5d88:  js     0x5f5d97            ; already negative: skip both
5f5d8a:  movl   %ebx, %ecx
5f5d8c:  calll  0x5f7740            ; Unit::add_to_army
5f5d91:  movl   %eax, -0x10(%ebp)
5f5d94:  orl    $-1, %esi           ; ... and then esi = -1 regardless
5f5d97:  testl  %edi, %edi
5f5d99:  movl   $0xffffffff, %eax
5f5d9e:  cmovnel %eax, %esi
5f5da1:  jmp    0x5f5da6
5f5da3:  movl   -0x4(%ebp), %esi    ; the early-out: esi = -(uVar12 != 0)
5f5da6:  ...                        ; -> find_melee_target(this, esi, ...)
```

Three ways in and every one arrives with **−1**. The AI-driven branch
(`esi >= 0`) calls `add_to_army` and then `orl $-1` unconditionally; the
other branch is already −1; and the early-out at `5f5da3` loads
`-(uVar12 != 0)`, which is −1 exactly when that path is taken
(`5f5c04`: `negl %esi; sbbl %esi, %esi`). The only later write is the
`cmovnel` at `5f5dba`, which substitutes a fixed 11,520-unit range when
`game->semaphore[1] & 2` — a game-global, not a per-unit or per-frame
term.

Two consequences:

- **`uVar12` selects `add_to_army`, not a radius.** §28.2's finding — that
  who=1's hoplite captain has `uVar12 == 0` — stands, and is exactly why
  that captain joins an army; it says nothing about how far it searches.
- **`find_melee_target`'s `param_1 == 0` block is dead code for every
  `think_attack` call**, so §27.2's 1024- and 256-frame grids and their
  128- and 64-tile sweeps never run from this caller, and §28.3's
  "the multiplier that fits is the one neither path applies to this unit"
  dissolves: the applicable arm is the −1 branch's, whose floors are
  `unit_respond_range * 0xc0` and — because `unit_masks & 0x40000` is set
  on these hoplites — `unit_respond_range * 0x180`.

And the arm the −1 branch computes has **no frame-dependent term** in it:
stance, reach, `unit_respond_range`, `unit_masks`. The hoplite captain's
search radius at 616 and at 635 is the same number.

### 30.4 The search is quantised in cells, and both frames scan the same one

`find_nearby_target` does not test a distance to decide where to look. It
walks the world's **object grid**: from the searcher's own cell it takes
`circle_x[i]` / `circle_y[i]` for `i` under
`circle_radius[min(0x20, (range + 0x2ff) / 0x300 + bonuses)]`, and only
objects threaded on those cells are ever passed to `valid_target`. A cell
is `div_3_table[pos >> 8]` — floor division by **768**, four tiles
(`docs/ATTRITION.md`, the units table, where `init_coord_lookup_array` is
what makes `div_3_table[i] == i / 3`).

Run112's own coordinates, in cells:

| frame | searcher | its cell | what it is looking at | that cell | `near_o` |
| --- | --- | --- | --- | --- | --- |
| 616 | `1/6` hoplite | `(3, 10)` | bowmen `0/6`–`0/8` | `(1, 10)` | **−1** |
| 621 | `0/9` slinger | `(1, 10)` | `1/6` | `(3, 10)` | **6** |
| 635 | `1/6` hoplite | `(3, 10)` | slinger `0/10` | `(1, 10)` | **10** |
| 635 | `0/6` bowman | `(1, 10)` | `1/6` | `(3, 10)` | **6** |

`1/6` stands on `(2424, 7800)` at all three frames and never moves; the
bowmen stand on their seats at both 616 and 635. **One cell pair,
`(1, 10)` ↔ `(3, 10)`, is traversed successfully by three searches and
refused by a fourth — and the fourth is the word.**

Since §30.3 makes the radius frame-independent, the cell set `1/6` scans
at 616 is the cell set it scans at 635, and at 635 that set demonstrably
contains `(1, 10)`. **So the bowmen were scanned at 616 and refused.** No
value of `unit_respond_range`, and no choice among the arms, can produce
that: a radius large enough for 635 is large enough for 616, and one small
enough to miss 616 misses 635 too. The same statement in the old units:
missing `(1, 10)` needs `(range + 0x2ff) / 0x300 <= 1`, so
`range <= 768`, so `unit_respond_range <= 2` on the `* 0x180` floor — and
then 635 cannot happen.

**That closes §26–§28's radius programme by measurement**, which is the
thing four items could not do by argument. `chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached`
asserts all four rows and both seats, so the closure is a test rather than
a paragraph.

### 30.5 What is left, and what would kill each reading

The search ran at 616 and refused every bowman. The gate is not in doubt:
the dump prints `idle 1` on each captain's birth frame, and the slinger
captain's own frame is on **neither** of `think`'s grids —
`(9 + 620) & 0x1f == 21` and `& 0xf == 5` — so its birth-frame search can
only have come through the `idle == 1` arm, which means `check_idle` runs
before `think` and the hoplite captain reached `think_attack` at 616 by
the same arm. Every other gate in `think` is frame-independent for that
unit and was passable at 635, where it searched.

So what remains is a **target-acceptance predicate**, and the distance is
not available to it: `check_target` has no distance test on this path
(its only one is `unit_guard_respond_range`, behind an activity of `0xc`
these units do not have), and the loop's distance test sits below the
`near_o` write. Two readings survive, and neither is booked:

1. **`Object::valid_target@00648ba0`** refuses a Bowman to an AI-driven
   melee unit and does not refuse a Slinger. *Killed by*: a reading that
   shows `valid_target` carries no term that can separate two enemy
   military units of the same player at the same instant.
2. **`Object::check_target@00649e00`'s tail** — `(target is a unit) &&
   world cell byte `+0xf` `!= -1` && !poor_target(...)`. *Killed by*:
   `Object::poor_target@0064a270` needing the target to be **moving**
   (`is_move` on its order) on its main arm, which the bowmen are not —
   that arm is already dead, so only its type-record arm (`UnitTypeData
   +0x9a`, bit 6) and the cell byte survive — and the cell byte is read
   from the *same cell* in both rows of the table above, which kills it
   too.

Written out, that leaves `valid_target` and `poor_target`'s type-record
arm, and this section stops there rather than choosing: §26.2, §26.4,
§27.1 and §29.2 were each a mechanism named one fetch too early, and the
pattern is the reason `docs/DECISIONS.md` 42 exists.

**What a capture would ask, if one is ever booked for this.** Not a new
window — the fields are already printed at `UNITS=3`. It would need the
*target's* side of the predicate, which no level prints: a run with
`UNITS` raised high enough to carry `UnitTypeData +0x9a` per unit, over
`[614, 640)`, so that a Bowman and a Slinger can be compared on the one
byte `poor_target`'s surviving arm reads. ~~Nothing on disk answers it.~~
No capture was taken for this item. **Nothing on disk answers *that arm*;
something on disk answered the question** — §31.3:
`ObjectData::visible` (`+0x40`) is the *other* survivor's own input and
the dump has printed it at `UNITS=3` all along. The `+0x9a` capture stays
unbooked and run113 unspent.

### 30.6 Coverage

**Diff-backed**: the whole of §30.1 (a probe over this crate's own
chapter-two walk), and every number in §30.2's and §30.4's tables — they
are run112's own printed `x_internal`, `y_internal`, `near_o` and `idle`,
and `chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached`
holds them. §30.5's `idle`/grid argument is arithmetic over dumped
fields.

**Listing-backed, which is stronger than the decompile**: §30.3's claim
that `think_attack` passes −1 on every path.

**Reading-only, and owed a blind second reading**: §30.2's rule for when
`near_o` is written, §30.4's description of the circle walk and the
`0x300` cell step, and the two survivors in §30.5.

~~**What this has not established**: why the bowmen are refused. It
narrows the question from "what radius" to "which predicate", and names
the two that survive, and that is all.~~ **Answered by §31**: the
predicate is survivor 1, and it is `UnitData::is_seen@00607a60` reached
through `ObjectData::valid_target_const`'s fifth test. Survivor 2 is not
killed, only no longer needed — §31.7.

## 31. The predicate is `is_seen`, and a unit born on the map lit no fog (item 447, 2026-09-21)

§30 narrowed 616 from "what radius" to "which predicate" and left two
survivors, neither booked. This section answers it on the first:
**`ObjectData::valid_target_const@006472c0`'s fifth test is a visibility
test**, this crate had no term for it, and putting one in moves chapter
two's word **616 → 624**.

`GOLDEN_WORD_CHAPTER_TWO` is 624 of 901, sequence 624, first value
disagreement 625 — and the value diff beside it is §31.6, which is the
part worth reading: the draw stream holds to 624 and the *positions* part
at **622**.

### 31.1 The chain, read down to the test

`Object::valid_target@00648ba0` opens with a virtual call and returns 0
on it before anything else:

```
iVar2 = (**(code **)(*(int *)this + 0x138))(param_1,param_2);
if (iVar2 == 0) return 0;
```

`+0x138` on `Unit::vftable` is **`ObjectData::valid_target_const`**
(`tools/ghidra/decomp/vtables.txt`; the rest of `Object::valid_target`
is the *capture* ladder — `check_capture_eligible`, `Build::check_capture`
— and a unit target falls straight through it to `return 1`).

`valid_target_const` is five tests before it ever asks what class the
candidate is:

1. `param_1 < 0 || param_2 < 0 || 7 < param_2` — the gaia bound this
   crate already had.
2. `param_2 == this->who`.
3. `!LeaderData::is_enemy(leaders[this->who], param_2)`.
4. the candidate's `flags & 1` — alive.
5. **`target->vtable[0x48](this->who, 0)`**, which is
   `UnitData::is_seen@00607a60`. Zero → `return 0`.

Everything §26–§30 argued about — domain, `unit_masks`, the air ladder,
the two `0x218` arms — sits *below* test 5. `Sim::valid_target` had tests
1–4 and the ladder, and its own doc comment said "no fog".

### 31.2 `is_seen`, and the half this crate can answer

```
is_seen(who, 0):
    if (unit_masks & 0x800) == 0 && (type->unit_masks & 0x4000) == 0
            && (unit_masks2 & 0x8000) == 0:
        if (type->unit_masks & 0x40000) == 0 or get_order() != 0:
            goto fog
    if (unit_masks & 0x1000) == 0 && !is_detected(this, who):
        return 0                            # the stealth arm
fog:
    if who != this->who and game.reveal_map != 3:
        if !WorldData::is_seen(pos / 0x180, who):
            return (visible >> who) & 1
    return 1
```

and `WorldData::is_seen@006b55c0` is `seen[fy * fog_xs + fx] &
leader->ally_mask`, with three always-true arms above it: `who > 7`,
`reveal_map == 3`, and two leader flags (`0x800`, and a `num_units` count)
this crate does not carry.

The fog grid is `div_3_table[pos >> 7]` — `pos / 0x180`, half a world cell,
**the same plane `docs/VISION.md` §5 already builds**. So the whole of
test 5's fog half was already answerable here; nothing asked it.

`Sim::target_is_seen` is that half, called from `Sim::valid_target` at the
original's own position. Two seams, both **refusing**: the stealth arm is
not carried, and `ObjectData::visible` is not modelled, so the fallback
answers 0 where the original may answer 1.

### 31.3 What `visible` is, and why the seam does not touch 616

`ObjectData +0x40` is `visible` (`types.txt`), and run112 prints it on
every unit record at `UNITS=3` — so the fallback's own input is on disk
and no capture was needed to read it. Its writers are
`Unit::set_attacking@005ff5b0` and `Unit::do_cast@005ebfe0`, cleared by
`Unit::work` (`docs/VISION.md` §6); `set_attacking(this, victim_who)` does
`this->visible |= 1 << victim_who`. **A unit that attacks you becomes
visible to you**, through fog, until it goes back to work.

run112's own values, over the window §30 was fought on:

| unit | born | gains player 1's bit | at what distance from `1/6` |
| --- | --- | --- | --- |
| `0/6`, `0/7`, `0/8` bowmen | 611 | **636** | 7.9, 7.1, 7.7 tiles |
| `0/10` slinger | 621 | **631** | 6.56 tiles |
| `0/11` slinger | 621 | 640 | 5.71 tiles |
| `0/9` slinger captain | 621 | 646 | 7.71 tiles |
| `1/6`–`1/8` hoplites | 616 | never in the chapter | — |

The three bowmen carry `visible 0` at **616** and at **635**. So on the
item's own frame the fallback is 0 whatever the fog says, the hoplite's
search is left with `WorldData::is_seen` alone, and the hoplite's line of
sight is 6 tiles against a gap of 7.25 to the nearest bowman. **That is
the refusal.** The bit arrives at 636 not because anyone saw the bowmen
but because the bowmen started shooting.

It also explains the row the table above makes look like vision and is
not: `0/6` accepts `1/6` at 635 while `1/6` carries `visible 0` — a
bowman's own 11 tiles of sight reach 8, so it never needs the fallback.
Nearest and chosen and *seen* are three different things, which is the
same warning §30.2 drew about `near_o`.

### 31.4 `Object::add_to_world` does three things and this crate did two

The gate alone moves the word 616 → **621** and stops, because at 621 the
*slinger* captain's search is refused too — player 0's fog does not cover
the hoplites' cell either. It does not because **nothing ever lit it**:
`Sim::add_unit` did `add_to_world`'s two collision indices and not its
third job, `update_seen(0)`. `Sim::come_out_place` has made that call
since garrisoning landed (`docs/VISION.md` §6, the ejection row); a unit
*born* on the map made none, so its owner's line of sight did not exist
until it first crossed a half-cell or the hundredth-frame resync came
round.

Nothing read the plane closely enough to notice. With both halves in,
the word is **624**.

### 31.5 What it moved

| counter | before | after |
| --- | --- | --- |
| `GOLDEN_WORD_CHAPTER_TWO` | 616 | **624** |
| chapter two, first value disagreement | 617 | 625 |
| Great Lakes endpoint, `off` at 24001 | 55 | **53** |
| `LONG_WORD_GREAT_LAKES`, `LONG_WORD_EAST_INDIES`, `GOLDEN_WORD_CHAPTER_ONE` | — | unmoved |
| East Indies endpoint and both ladder rungs | — | unmoved |

Both halves touch every unit on every map — a target out of sight is
refused wherever it stands, and a unit born on the map now lights its own
disc — so the endpoint row is evidence about 24,000 frames of run-up and
not about the predicate (DECISIONS 36).

And one row moved the other way, which is the honest half:
`chapter_two_s_first_attack_orders_are_the_dump_s` had this crate's
hoplites taking an attack order at **616** where the dump has **635**;
they now take none at all. The original's `1/6` accepts `0/10` at 635 on
the `visible` fallback — the slinger set its own bit at 631 by attacking a
hoplite — and player 1's sight never reaches that cell. The row moved from
*wrong and early* to *absent*; §31.7 books it.

### 31.6 624's widening, and the residue two frames in front of it

`chapter_two_s_word_frame_is_widened_whole` compares every record run112
carries over `[620, 628)`, both directions. The whole map of first
partings:

| key | first frame |
| --- | --- |
| `order 0/9`, `angle 0/9` | 622 |
| `order 0/10`, `pos 0/10` | 622 |
| `order 0/11`, `pos 0/11` | 622 |

Nothing on the hoplites, nothing on the bowmen, nothing unlinked or extra
in either direction, and no `los`, `packed`, `collide` or `search` row at
all. **The draw stream agrees for two frames after the values stop
agreeing**, which is the thing a widening exists to catch and the reason
`CLAUDE.md` asks for a value diff beside every word that moves.

`0/9`'s *position* is not in the map — the captain stands where the dump
stands it and only its order parts — so the residue is a **destination**
and not a step. At 622 this crate plans all three slingers' move to
`(2424, 7800)`, which is `1/6`'s own seat, where the original plans
`(1608, 8184)`, `(1560, 7848)` and `(1704, 8424)`: three spread points,
six path slots to this crate's ten. That is parked 400's shape — a chase
planned to the target's own point rather than to `Unit::find_attack_pos`'s
ring (§17) — one squad over.

**Closed 2026-09-21 by item 462 (§32), and the map above was two
divergences short of the truth.** The table is the map the *harness* could
see: `OrderMismatch::Target` was dead for any target not on the board at
`BEGIN GAME`, which in this chapter is every unit, so the row that said
these three slingers chase the wrong **figure** — `1/6` where the dump has
`1/8` — never printed. With it live the window's first parting is 621 and
not 622. §32.1 is the cause of the target and §32.2 of the destination;
the whole of `[620, 628)` now agrees, on every record in both directions,
and the window has moved with the word to `[633, 641)`.

### 31.7 Coverage

**Diff-backed**: §31.5's whole table (the pinned words and endpoint rows,
each a test), §31.6's map
(`chapter_two_s_word_frame_is_widened_whole`), and §31.3's table, which is
run112's own printed `visible` beside its own printed coordinates.

**Listing- and export-backed**: §31.1's call chain (`vtables.txt` for the
`+0x138` and `+0x48` slots, `types.txt` for `visible` at `+0x40`) and
§31.2's transcription of `is_seen` and `WorldData::is_seen`.

**Reading-only, and owed a blind second reading**: §31.2's stealth arm,
which no run on disk executes.

**What this has not established.**

- **§30.5's survivor 2 is not killed — it is no longer needed.**
  `find_nearby_target` calls `check_target` with `param_6 = 1`, so
  `Object::poor_target@0064a270` is reached, and its type-record arm
  (`UnitTypeData +0x9a`, bit 6) is not excluded by anything here. What
  changed is that it can no longer move chapter two's word: the frame it
  would explain now agrees. The capture §30.5 named — `UnitTypeData
  +0x9a` per unit over `[614, 640)` — remains unbooked and unspent, and
  it is the only way to settle that arm on its own terms.
- ~~**`ObjectData::visible` is not modelled**~~ — **answered 2026-09-21 by
  item 457, `docs/VISION.md` §9.** `Unit::set_attacking`'s `visible |= 1 <<
  victim_who`, its 32-frame clear and `Unit::update_local_seen` are all
  modelled now, and this crate's `1/6` takes the 635 order the dump has:
  `chapter_two_s_first_attack_orders_are_the_dump_s` went from six of nine
  rows to **nine of nine**. Two corrections to the sentence struck above.
  The `+0x10c` gate is **`ObjectTypeData::is_siege`** — settled from the
  PDB type record and `ObjectData::is_dock@004711e0`'s thunk, VISION §9.2 —
  and it gates one of `Unit::fight`'s two call sites, not the write. And
  the writer list of two was short by three: `Build::do_attack` and
  `Build::do_missile_launch` write the field too, so §31.3's "no building
  path reaches it" is withdrawn (VISION §9.1). **The word did not move**:
  624 stands on the residue below.
- **The two always-true leader arms of `WorldData::is_seen`** — `0x800`
  and the `num_units` count — are not carried, and no capture on disk
  exercises them.
- ~~**The residue at 622 is measured and not diagnosed.**~~ **Diagnosed
  and closed 2026-09-21 by item 462, §32.** The mechanism was not parked
  400's `find_attack_pos` alone and it was not parked 460's `x_size` at
  all (falsified, `docs/VISION.md` §9.8): it was two faults, a candidate
  **order** and a missing **half**, and the frame was right while both
  named mechanisms were wrong.

## 32. The candidate order is the cell's own chain, and the chase has a ring (item 462, 2026-09-21)

Chapter two's golden word **624 → 637**, and the widening the word carried
— every record run112 prints over `[620, 628)`, both directions — went
from six first-partings to **nought**. Two faults, one upstream of the
other, and the item's first product is neither: it is the reason nobody
had seen the first one.

### 32.1 `find_nearby_target` walked the unit index, not the cell chain

§12.2 has said since it was written that the scan takes "on each cell
every object on it (the cell's `down/down_who` chain, all players)", and
§18.1 measured that order against run108's own bracket: the chain in
chapter one's cell is `1/8, 1/7, 1/6, 0/8, 0/7, 0/6`, confirmed against
the dump's printed `up`/`down`/`up_who`/`down_who` on all six. The
implementation walked **`(0..units.len()).chain(buildings)`** instead.

The two orders agree whenever nothing on a cell ties, which is why this
survived six items on this very chapter. Chapter two's slinger squad is
the case that does not tie. From `0/9` at `(888, 8376)` the three
identical hoplites score

| candidate | `attack_dist` | `+ (targeted + 8) × 0x30` | `/ 0xc0` | bucket |
| --- | --- | --- | --- | --- |
| `1/6` `(2424, 7800)` | 1459 | 1843 | 9 | `value / 10` |
| `1/8` `(2472, 7944)` | 1468 | 1852 | 9 | `value / 10` |
| `1/7` `(2568, 7800)` | 1596 | 1980 | 10 | `value / 11` |

`1/6` and `1/8` **tie exactly** — nine units apart on a metric that
divides by 192 — and `1/7` does not. The keep test is strictly greater on
both sides (`00649a6e`, `if (local_70 < iVar8)`), so the tie goes to
whichever is reached first: the unit index reaches `1/6`, and the chain,
which `Object::add_to_world` pushes on the head, reaches `1/8`. The dump's
`ATTACKORDER` on all three slingers reads `ox 8 whom 1 uid 14`.

[`Sim::cell_chain`] is the walk; the scan takes it and appends the cell's
buildings after it. **Stated seam**: a building is an object on the
original's chain too and this crate has never threaded one, so a cell
holding a building and a unit that tie can still be scanned in the wrong
order. The appended order is the one the index-ordered scan already had
between the two classes, so nothing but the units among themselves moved.

### 32.2 The unit half of `find_attack_pos`, which was a `return None`

`00601280` splits on the target's `is_build` at `601604`. §17 is the
building side. The other side was not modelled at all, and its callers
fell back to the target's own position — so this crate sent a chaser to
stand *inside* the figure it was shooting at.

Three steps:

1. **The radius**, `60256a`-`602595`. `spot = target.big_radius +
   my.big_radius + stand`, where `stand` is §17.3's stand-off. Over
   `0x240` the sweep is a **band** — `min = spot − 0xc0`, `max = spot`,
   `step = 0x60`, three rings — and at or under it a single ring at `spot`
   on `find_nearby_spot`'s own defaults.
2. **The bearing**, `local_24`: `find_angle` of the approach vector, so
   the sweep's first candidate is the point on the asker's own side of the
   target. [`Sim::find_nearby_spot`] walks `0, ±1, ±2, … ±7` sixteenths
   from it.
3. **The acceptance**, `6025dd`-`602620`: a spot the sweep found is taken
   outright when the far arm asked (`local_2c`) or when the flanking
   projection supplied the centre (`local_40`), and otherwise only if the
   asker would be **in range standing on it**. Anything else falls through
   to the caller's fallback.

The **far arm** came with it: a unit target beyond `(range + 8) × 0xc0`
sets `local_2c` and `stand = (range + 2) × 0xc0`, and was a `return None`
before.

**The arithmetic was checked against the dump before the code was
written**, which is what made this a measurement rather than a try. For a
slinger (`max_range` 6, `big_radius` 48) against a hoplite (`big_radius`
48) at `attack_dist` 1468:

```
stand = max(0xc0, 6·0xc0 − 0x60 + (48 − 0x30) − 48) = 1008
spot  = 48 + 48 + 1008 = 1104  >  0x240
rings = 912, 1008, 1104
```

and on ring **912**, at bearings `0`, `+1` and `−1` sixteenths from
`find_angle(from − target)`, the snapped points are `(1608, 8184)`,
`(1560, 7848)` and `(1704, 8424)` — the dump's three destinations, to the
unit. The implementation then produced exactly those, with the positions
to match.

### 32.3 637's widening, and the residue in front of it

`chapter_two_s_word_frame_is_widened_whole` now walks `[633, 641)` — the
window moved with the word rather than being left naming a frame the word
has walked out of, which is parked 449's lesson applied at the move. The
map of first partings:

| key | first frame |
| --- | --- |
| `order 0/6`, `order 0/7`, `order 0/8` | 635 |
| `order 1/6`, `order 1/7`, `order 1/8` | 635 |
| `angle 0/6`, `angle 0/7`, `angle 0/8` | 636 |
| `order 0/11`, `pos 0/11` | 636 |
| `pos 1/6`, `pos 1/7`, `pos 1/8` | 636 |
| `visible 0/11` | 637 |
| `order 0/5` | 639 |
| `pos 0/5` | 640 |

**The values part at 635, two frames before the draw stream**, and every
one of the six earliest rows is a `Target`: the bowmen `0/6`, `0/7`, `0/8`
take `1/6` where the dump takes `1/8`, and all three hoplites take `0/7`
where the dump takes `0/11`. That is §32.1's shape again — a tie among
near-equidistant identical figures — and the cell chain **did not** settle
it here. Everything on 636 is downstream: a chase planned at a different
figure walks a different way, and `visible 0/11` at 637 arrives when its
first arrow does.

`order 0/5` and `pos 0/5` at 639-640 are a **citizen** far from the
engagement, in no earlier window, and nobody's item yet. They are in the
pinned map so a regression in them cannot hide behind the engagement.

### 32.4 What this has not established

- **The flanking branch is read and not built** (`602870`-`602a9c`). When
  the target carries a **move** order the original first asks whether its
  back is turned: within 60° of running away (`(target.heading − bearing)
  + 0x80000000 <u 0x2aaaaaaa`) it may return the asker's own position
  outright once inside `(range + 4) × 0xc0`, and otherwise `flanking()`
  projects a point ahead of the target and sweeps from *there*, with
  `local_40` making the result unconditional. Neither is modelled; a
  moving target takes the centre a standing one would. Chapter two's
  target stands still (`STACK<TYPE> length 0` on `1/8` at 622), so nothing
  on disk reaches the branch. *A capture would settle it:* `UNITS=3` over
  a chase of a **fleeing** unit, and the chaser's `MOVEORDER x/y`.
- **The mandatory fallback is not modelled** (`601604`-`601700` and the
  function's tail): when the current order's `mandatory` byte is set and
  the target is further than `max(0x600, (max_range + 4) × 0xc0)`, the
  original does *not* fall back to the target's seat — it sweeps
  `[x_size × 0xc0, x_size × 0x300]` around it and, failing that, snaps to
  a `0x300` grid and calls `WorldData::restrict`. Nothing on disk carries
  a mandatory attack order at that range.
- **`find_nearby_spot`'s `tregion` argument is dropped.** `00601280`
  passes `local_3c` — the approach point's terrain region, computed only
  when the asker's order is an ATTACK and the two leaders differ — and
  [`Sim::find_nearby_spot`] has no such parameter. It can only refuse
  further, so the seam is one-directional.
- **§32.3's 635 target residue is measured and not diagnosed.** The frame
  and the six rows are in the table; what separates `1/8` from `1/6` for a
  bowman, and `0/11` from `0/7` for a hoplite, is not established, and the
  cell chain is now the wrong hypothesis to reach for twice.
- **The scan order is not the whole of §12.2 either.** The score this
  crate computes is missing the previous **mandatory** target's halving
  (`00649793`, `TargetOrder +0x8/+0xc` when the current order is mandatory)
  and the cavalry archer's bearing weights (`param_4`, ×4 / ×2 / ÷10 /
  skip). Neither can fire on anything on disk — no capture has a mandatory
  attack order alive during a re-search, and none has a cavalry archer —
  but both are in the ranking and neither is in the code.

### 32.5 Coverage

**Diff-backed**: §32.1's table (the two candidates' `attack_dist` are the
crate's own and the pick is `chapter_two_s_word_frame_is_widened_whole`'s
`Target` row, which is the dump's `ATTACKORDER ox`), §32.2's arithmetic
and its three destinations (the same test, now empty over `[620, 628)`),
§32.3's whole map (that test), and the three counters item 462 moved —
chapter two's word, Great Lakes' endpoint, and
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`'s
exact-match count 4 → 5.

**Listing- and export-backed**: the keep test at `00649a6e`, the chain
walk's `+0x2c`/`+0x2e` link at `0064937f`, and `00601280`'s split at
`601604` with the radius and acceptance line numbers above.

**Reading-only, and owed a blind second reading**: every bullet of §32.4.
No run on disk executes the flanking branch, the mandatory fallback, the
mandatory-target halving or the bearing weights.

## 33. The tie is broken by a decaying counter and an inverted weight (item 466, 2026-09-21)

§32.3 measured chapter two's *values* parting at **635**, two frames
before the draw stream, on six `Target` rows: the three bowmen took `1/6`
where the dump took `1/8`, and the three hoplites took `0/7` where the
dump took `0/11`. §32.4 wrote the residue down and said the cell chain was
"now the wrong hypothesis to reach for twice".

It was two hypotheses, one per squad, and neither is the chain.

**Three of the six are closed and three are not**, and the split is not
an accident of effort: the two causes are independent arithmetic, each
owning one squad, and only one of them can be landed today. §33.1 landed
and closed the bowmen's three. §33.2 is established against the dump to
the same standard and is **deliberately not landed**, because the
correct ranking exposes a defect in `find_open_slots` that would turn a
pinned assertion red — §33.4 has the reasoning and item 470 carries
both. The values still part at **635**, on the hoplites' three rows.

### 33.1 `ObjectData::targeted` is a decaying penalty, not a reference count

§12.2's ranking shapes the distance before the divide:

```
dist += (targeted + 8) * 0x30
```

`targeted` is `ObjectData +0x3d`, a `char`. **Four functions in the whole
executable write it**, and the list is the finding:

| writer | what it does |
| --- | --- |
| `Object::init@00647750` | `= 0` |
| `Object::find_nearby_target@00648da0`, `00649ba6` | `if (t < 100) t++` on the **winner it returns** |
| `Unit::process@00610bc0`, `006114e6` | `t = t / 4`, every sixteenth frame |
| `Wall::process@00640450` | the same `/4` |

Nothing decrements it when an attacker dies, drops its target or is given
another, and `add_attack_order` never touches it at all. So it is not a
count of current attackers: it is a **crowding penalty that decays**, and
the decay is its whole bookkeeping. The slot is the one the attrition
refresh is nested inside, under `inside_up < 0`:

```
if ((frame + o) % 16 == 0) {
    targeted = targeted / 4;          // signed, toward zero
    process_cloak();
    if ((frame + o) % 32 == 0) { … process_attrition(); … }
}
```

This crate bumped in `add_attack_order` — so a three-figure squad handed
one target through §21's captain mirror counted **three** — and never
decayed. Chapter two's arithmetic is where that bites. The slinger squad
took `1/8` at 622; by 634 this crate had `1/8.targeted = 3` and the
original had `0`, its captain's single bump having been quartered at
frame 632 (`o = 8`, so `632 + 8 ≡ 0 (mod 16)`). The bowman captain `0/6`
at `(888, 7800)` then scores its three candidates, in the cell chain's own
order (`1/8`, `1/7`, `1/6` — §32.1):

| candidate | `attack_dist` | halved | `+ (targeted + 8) × 0x30` | `/ 0xc0` | score |
| --- | --- | --- | --- | --- | --- |
| `1/8` | 1440 | 720 | this crate 1248 (`t = 3`) | 6 | 23005 |
| `1/8` | 1440 | 720 | the original 1104 (`t = 0`) | **5** | **26839** |
| `1/7` | 1536 | — | 1920 | 10 | 14639 |
| `1/6` | 1392 | 696 | 1080 | **5** | **26839** |

With the stale `+3` the bowmen take `1/6`; without it `1/6` and `1/8` tie
exactly and the chain hands the tie to its head, `1/8` — the dump's
`ATTACKORDER ox 8 whom 1 uid 14` on all three bowmen at 635. **The same
tie §32.1 found one squad over**, broken by a different stale number.

The halving above is §12.2's `dist /= 2` arm, and it is what puts `1/6`
and `1/8` in one bucket while `1/7` is two away: `maxr` is 1920 here, so
`1392 + 0x180` and `1440 + 0x180` clear it and `1536 + 0x180` does not.

### 33.2 `compare_target`'s fourth argument inverts the damage weight — established, not landed

**Status.** Everything in this subsection is established against run112's
own dump and was implemented, measured and then *withdrawn* from the
tree; the code carries the human arm still, and
[`Sim::compare_target`]'s doc comment says so. It is not a doubt about
the finding. It is that landing it alone turns
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` red for a
reason that has nothing to do with the ranking — §33.4 — and a red tree
costs the next lane its oracle for three rows that do not move the word.
Item 470 lands it together with the defect it uncovers.


`Object::compare_target(o, who, in_range, ai)` at `0064ef4b`:

```
dmg = get_damage(this, o, who, find_angle(0, 0), 0, 0, NULL);
if (ai == 0) v = v * dmg;
else { if (dmg == 0) return 0; v = v / dmg; }
```

So the two rankings differ in **sign**, not in scale. A human's search is
drawn to what it kills fastest; a computer leader's is drawn to what it
kills *slowest*, and cost and fragility carry the pick instead. §12.3 and
§18 both recorded the arm and left it unmodelled — "the crate's own
numbers differ while the factor of five does not" — and run108's proxy had
already printed `ai=1` on chapter one's hoplite captain. It is a property
of the **searcher's leader**, computed once before the rings
(`00648e6e`-`00648e8d`):

```
ai = (leaders[who].flags & 4) == 0            // not human-controlled
     && LeaderData::get_diff(leaders[who]) == 0
     && (game.semaphore[1] & 2) == 0;
```

`get_diff@006ec000` answers the lobby's own `DIFFICULTY` outside a
multiplayer game, and run112's `GAMEINFO` carries `DIFFICULTY 0`. who=0 is
human and who=1 is not, so chapter two runs **both** arms in one frame —
which is exactly why one mechanism could not explain all six rows.

**The reading that would kill this, run before it landed.** A ranking
that divides rather than multiplies compresses its spread, and the
symptom of a collapsed spread is every member of a squad converging on
one answer — which is what this crate's three hoplites do downstream
(§33.4). So the dump was asked directly: do who=1's three hoplites hold
*three* targets or one? They hold **one**, `ox 11 whom 0`, on **every
frame from 635 to 700**, while their `MOVEORDER` destinations stay three
and distinct throughout — `(1416, 8328)`, `(1128, 8424)`, `(1368, 8472)`
at 636, still diverging at 665. Convergence on a single target is the
original's own behaviour; the spread lives in the melee slot, a different
function, and the dump separates the two for us. Had the dump's hoplites
held three targets, this reading would be dead whatever the six rows did.

The second falsifier is the target's *identity* rather than its
multiplicity: the dump picks `0/11`, a **slinger**. Multiplying by damage
puts a bowman 26% ahead at these distances, on every candidate and in
every scan order; only the divide arm puts the two slingers in front.
That is a value comparison and not a draw comparison.

Its hoplite captain `1/6` at `(2424, 7800)` scores six candidates in the
chain's order `0/9, 0/11, 0/10, 0/8, 0/7, 0/6`:

| candidate | `attack_dist` | `/ 0xc0` bucket | `v × dmg` (this crate) | `v / dmg` (the original) |
| --- | --- | --- | --- | --- |
| `0/9` slinger | 1507 | 9 | 3061 | 208 |
| `0/11` slinger | 1125 | 7 | 4955 | **257** |
| `0/10` slinger | 1083 | 7 | 4955 | **257** |
| `0/8` bowman | 1344 | 9 | 5630 | 211 |
| `0/7` bowman | 1248 | 8 | **6256** | 234 |
| `0/6` bowman | 1392 | 9 | 5630 | 211 |

A bowman is the fatter target on either weighting — `4 × cost × attack ×
100 / hits_left` is 61,714 against a slinger's 37,647 — and multiplying by
damage keeps it in front. Dividing does not: the two slingers tie at 257,
the chain reaches `0/11` first, and that is the dump's `ox 11 whom 0` on
all three hoplites. The dump's `near_o 10` is consistent with both, because
`near_o` is the nearest *acceptable* candidate and `0/10` is nearest on
every reading (§30).

### 33.3 What the widening ruled out, so nobody runs it twice

Each of these was a hypothesis the 635 frame invited, and each was killed
by reading the dump rather than by argument:

- **The fog / `is_seen` plane.** The obvious story — who=1 sees the two
  slingers and not the bowmen — is **false**. `0/10` fires at 630 and
  gains `visible 2`; `Unit::update_local_seen@0060e410` then writes its
  own disc into who=1's `seen` plane with mask `visible`, and the disc is
  `circle_radius[CIRCLE_RADIUS]` = the full 3×3 (the slinger's
  `CIRCLE_RADIUS` is 1 and `vector_dist(1, 1) = 1`). That lights fog cells
  `(2, 20)` and `(2, 21)` — the bowmen's and `0/9`'s — five frames before
  the search. Both sides' fog agrees here; the separation is arithmetic.
- **`tregion`.** `Object::check_target@00649e00` makes a target in another
  terrain region prove `is_in_range`. All nine figures stand in region
  **0** (tiles `(4, 40)`…`(12, 41)`), so the arm never fires.
- **The previous mandatory target's halving** (`00649793`,
  `local_68`/`local_6c`). It is the *current order's* target and only
  when that order is mandatory; both captains carry no order at all at
  634, and it can only ever halve **one** candidate.
- **`UnitData::full`.** §12.3's `v /= (full + 1)` is a no-op: the dump
  prints `full 0` on all nine.
- **A search radius.** `find_melee_target(-1)` gives an AI-driven melee
  unit `unit_respond_range × 0x180` = 4608 (run108's proxy printed
  `max_dist=4608`), and no radius can separate the bowmen at 1248-1392
  from `0/9` at 1507 while keeping `0/10` at 1083 — §30's argument, in
  distances rather than cells.

### 33.4 Why §33.2 is not in the tree, and what its absence hides

`chapter_two_s_word_frame_is_widened_whole` walks `[633, 641)` and its map
with §33.1 alone is:

| key | first frame |
| --- | --- |
| `order 1/6`, `order 1/7`, `order 1/8` | **635** |
| `order 0/11`, `pos 0/11` | 636 |
| `pos 1/6`, `pos 1/7`, `pos 1/8` | 636 |
| `visible 0/11` | 637 |
| `order 0/5` | 639 |
| `pos 0/5` | 640 |

The bowmen's three `Target` rows are gone, and so are the three `angle`
rows that followed them on 636. The word itself **held at 637** — and
holds at 637 under §33.2 as well, which is the fact that decides the rest
of this subsection: neither half moves the headline.

**The two causes split cleanly by squad**, and the split was measured
rather than argued — each was disabled in turn and the same two tests run:

| landed | 635's `Target` rows | `chapter_two_s_visible_byte` |
| --- | --- | --- |
| §33.1 alone | bowmen closed, `order 1/6`-`1/8` still 635 | **green** |
| §33.2 alone | hoplites closed, `order 0/6`-`0/8` still 635 | **red** |
| both | none — 635 clean | red |

§33.1 is free. §33.2 carries the whole cost, and the cost is this:

> ```text
> assertion `left == right` failed: `visible` no longer arrives on the
> same units, or no longer carries the same players' bits
>   left:  [((0,6),2), ((0,7),2), ((0,8),2), ((0,9),2), ((0,10),2),
>           ((0,11),2), ((1,6),1), ((1,7),1)]
>   right: [((0,6),2), … , ((1,7),1), ((1,8),1)]
> ```

It is the **shape** assertion — the one whose own comment says it is the
mechanic's own claim and that nothing about the engagement's timing can
excuse a failure there. It fails first, so the nine pinned frames and the
`exact == 5` row below it are never reached.

**And the shape row passes today by accident.** That is the finding this
subsection exists to record, and it outlives either half:

- With §33.2, `1/8` strikes nothing in the chapter's 899 frames, so its
  `visible` bit never arrives and the row goes red.
- Without §33.2, `1/8` *does* strike — but only because it is chasing the
  wrong target, and only because that wrong target happens to hand it a
  slot of its own. Given the dump's own target all three hoplites plan
  the **same** melee slot, `(1512, 8040)` on every one. Given the wrong
  one they plan **two** between three, `(1320, 7800)` twice and
  `(1320, 7944)` once, and `1/8`'s distinct slot is the entire reason the
  assertion is green.

So the green is resting on a wrong two-slot assignment. `find_open_slots`
was already collapsing two chasers of three before this item; the correct
ranking makes it three of three and merely *reveals* that. A pinned
assertion that passes for the wrong reason is worse than one that fails,
because nothing in the tree says so — this paragraph is the only record
that it does.

The mechanism is §19's: `find_open_slots` rejects a slot another unit is
ordered to through `find_ordered_collision`, and that predicate walks the
object chain of the cells around the **slot**. The three hoplites' bodies
stand 1,200 units east of it, so none of them is on those chains and none
sees the others' orders. Two then queue behind the first.

**Item 470 is the successor and carries §33.2 with it.** The slot fix and
the ranking land together, green: that is the landing that closes the
hoplites' three rows, moves the parting to 636 and puts the `ai` argument
into [`Sim::compare_target`]'s signature. Neither is landable alone —
the ranking without the slot fix is the red above, and the slot fix
without the ranking is a correct chase toward a target the dump does not
pick. Whether `find_ordered_collision` should reach them through a second
chain, through the `pushed_group` arm this crate has, or because a melee
squad's members are ordered from somewhere that already holds the list,
is not established here.

### 33.5 Coverage

**Diff-backed**: §33.1's table and the three bowman rows it closes (the
distances and buckets are this crate's own; the picks are
`chapter_two_s_word_frame_is_widened_whole`'s `Target` rows, which are
the dump's `ATTACKORDER ox`/`whom`); §33.2's two falsifiers, both read
straight off run112 — the three hoplites' single target over 635→700 and
their three distinct destinations, and the dump's pick being a slinger;
§33.3's five eliminations (every one is a field the dump prints or a
number this crate computes on the frame); §33.4's map, its split table,
and the two-slot assignment the green rests on.

**Established and measured, but not in the tree**: §33.2's table. It was
implemented and run — that is where its numbers come from — and then
withdrawn, so no test checks it today and a later edit could silently
contradict it. Item 470 is what puts it back under an oracle.

**Listing- and export-backed**: the four writers of `+0x3d` and the
`(frame + o) % 16` slot at `006114e6`; `compare_target`'s `ai` split at
`0064ef4b`; the `ai` gate at `00648e6e`; `LeaderData::get_diff@006ec000`;
`Unit::update_local_seen@0060e410` and `circle_init@006817f0`;
`Object::check_target@00649e00`'s region arm.

**Reading-only, and owed a blind second reading**:

- **`ai`'s other arms are still unmodelled.** §12.3's building-class
  multipliers (`×10`, a city `100`, a defensive building `×40`, a silo
  `×15`) are all gated on a building target, and no capture on disk has a
  building in a target search.
- **The `ai` gate's third term** — `game.semaphore[1] & 2` — has no model
  here. It can only *clear* the flag, so this errs toward the AI arm on a
  setting no capture uses, and `get_diff`'s own multiplayer branches
  (`multi_diff`, the three semaphore reads) are not carried either.
- **`find_nearby_target`'s `flags` are adjusted on the AI arm too**
  (`00648e85`: `local_30 = 0` unless `param_5 != 1`). This crate computes
  its `anything` bits from the searcher's stance instead, and does not
  take that step. Nothing on disk passes `flags = 1`.
- **`Wall::process`'s decay is not modelled**, so a *building*'s
  `targeted` now grows without bound. It never had a decay here either;
  what changed is that the bump moved to the search, so a building's count
  is no longer inflated by its own `add_attack_order` as well.

---

## 34. What a citizen does when it is hit (item 464, 2026-09-21)

Great Lakes' long word is 10233 and the human's citizen `0/5` is the half
of it this item is about: the original's runs away, and this crate's took
an attack order of its own on the frame before. Two predicates, both read
off the listing and both measured against run100's block 10234.

### 34.1 `Unit::think`'s step 3 needs the military bit

`think@005f6e40:150` — the gate above `think_attack`, transcribed as a
condition rather than as code:

```
if ( ( is(0x3e, 1)
       && ( ((unit_masks & 0x80000) && think_merchant())
            || (!is_packing_or_unpacking() && think_attack()) ) )
     || ( type->attack != 0                 /* +0x1e8 */
          && (type->role & 0x10000) != 0    /* +0x2c8, MILITARY */
          && think_attack() ) )
    goto LAB_005f761a;                      /* the think ends here */
```

So a type with an `attack` column and **no** `role & 0x10000` never enters
`think_attack` at all. A Citizen's `attack` is **40** — it is an armed
civilian, not an unarmed one — so the column alone lets every idle citizen
in the game run `find_melee_target` once a frame on its first idle frame
and once every 32 after.

[`Sim::think_attack_join_army`] has carried the pair since item 350
(`docs/ARMY.md` §4.2: "without it this crate conscripted run53's
woodcutters on frame 307"). The **search** beside it carried only the
attack, so the gate was half-applied for four months. `crates/sim`'s
`orders::Sim::think` now reads `p.attack != 0 && p.combat_role`, off the
type's base column rather than the runtime stat, which is what `+0x1e8`
is.

The merchant arm (`is(0x3e, 1)`) is a stated seam; no capture on disk
reaches it, and `docs/MERCHANT.md` owns it.

### 34.2 `Unit::target_opportunity`'s flee arm

The last thing the function does before `LAB_00600877`'s retaliation, and
it is reached when the front order is **not** one of the seven move kinds
(`is_move(order_type())` is `local_c`) and `is_fleeing` is false. The
gate, `6006f0`–`600731`:

```
   ( (has_objmask(CIVILIAN) && type->max_range == 0)      /* 0x4, +0x1fc */
     || type->attack == 0                                  /* +0x1e8 */
     || local_8 )                                          /* the packing latch */
&& ( is_worker(this) || is_idle(this) )                     /* orderlist empty */
&& ( (type->unit_flags & 4) == 0                            /* +0x2b8 */
     || (!is_packing() && (unit_masks & 0x80000)) )
&& ( !is_hero() || (unit_masks & 0xa000) == 0 )
```

then the ring — `0x600` for anything that is neither a hero nor a supply
unit; for those, `0x180` when the attacker answers vslot `+0x20` false and
vslot `+0x130` true, and `0x300` otherwise — and

```
find_nearby_spot(type, x, y, &ox, &oy, ring, -1, 0,
                 find_angle(x - ax, y - ay), FILTER_NOT_ME, o, who, …)
add_move_order(this, ox, oy, FLEE_TO, 0, QUEUE_FIRST, 0, …)
```

**The bearing's arguments are the listing's, not the decompiler's.** Ghidra
prints `find_angle(unaff_EDI, unaff_ESI)` — it lost both. `6007cb`–`6007f2`
builds `ecx = this->x − attacker->x` and `edx = this->y − attacker->y` and
calls `0x92d130` fastcall, so the sweep's base bearing points **directly
away from whoever landed the hit**. `find_nearby_spot` leaves its out-pair
at the input point when it finds nothing (`docs/ORDERS.md` §10), so a
failed sweep still issues a `FLEE_TO` — to where the unit already stands.

`Unit::do_attack`'s reading of `Object::valid_target` is not involved; the
flee is the **victim's** answer, taken on the captain of the victim's squad
like every other arm of this function (§18.2).

**What this replaced.** `Sim::target_opportunity` carried
`obj_masks & CIVILIAN && max_range == 0 → return` with the comment "a
non-combatant does nothing (it would flee)". That test has no counterpart
in the original's tail — `LAB_00600877`'s own gate is `type->attack != 0`
and nothing more — so it is gone, and the flight is what stands where it
stood.

### 34.3 The measurement

run100, Great Lakes, block 10234. The human's citizen `0/5` is at
`(2232, 31224)` with an empty order list; `1/27`, at `(4584, 29784)`, has
an arrow in the air, and the block records `damage 2 damage_frame 10233
damage_o 27 damage_who 1`. The original's answer, in its own coordinates:

| field | dump, block 10234 | this crate, after 464 |
|---|---|---|
| order kind | `4` (`FLEE_TO`) | `4` |
| `orders_x/y` | `792, 31800` | `792, 31800` |
| `dest_angle` | `-1334771712` | `-1334771712` |
| `pos` | `(2232, 31224)` | `(2232, 31224)` |
| `g.cur_anim` / `g.stopped` | `0` / `1` | `0` / `1` |
| `idle` | `2` | `2` |

`(792, 31800)` is 1,551 units from the citizen — on the `0x600` ring, 1,536
plus the quarter-tile snap — and on the far side of it from `1/27`. The
citizen's twenty-two rows on the word's own block are **three**, and the
draw stream's delta on 10233 goes **6/4 → 5/4**.

### 34.4 What the frame still carries, and its mechanism

The word did **not** move. 10233 has two residues and only one was the
citizen's; the other is `1/28`'s `Unit::do_move+0xe84`, spent a frame
early — this crate plans and steps the group move its dropped
`ATTACKORDER` freed on 10233, where the original plans on 10234
(`docs/ORDERS.md` §7.12's closing paragraph named it).

**It is not a generic order-death delay.** Between blocks 10233 and 10234
the original's `1/28` changes exactly three things: the unit's
`flags & 0x80` clears, the guy's animation clock ticks, and its
`GROUP_MOVE`'s **`oxx` goes 40 → 28**. It stops following `1/40` and
becomes its own group's leader on the frame it does nothing — and
`do_group_move` runs `do_move` for the leader alone (`docs/GROUPS.md`
§8.3), which is why the plan is on the next frame and not on this one.
This crate's `1/28` holds a plain `MOVE_TO` there, not a `GROUP_MOVE` at
all, so it has no handoff to wait for. That is the next item on this
frame, and its falsifier is the `oxx` pair above.

The citizen leaves one residue of its own, also on 10234: with the attack
order gone, `Unit::think` falls through to `think_peasant`, and this
crate's `find_gather_spot` hands the idle citizen a job the original's
does not — so the order list is `[FLEE_TO, GATHER]` where the dump carries
one order, and `0/2001`'s `gather_down` chain carries `0/5` with it. The
original's citizen is idle for two frames and takes no job in the fifteen
blocks run100 carries after the flight.

**And the widening lost a false row.** `myhits` is the record's *maximum*
— `ObjectData::myhits`, the type's hit points after tech — with `damage`
the accumulator beside it (§4.3, and `crate::diff::army`'s loader has read
the pair that way since it existed). `run100_s_word_block_is_every_record_
the_dump_carries` compared this crate's *current* health against `myhits`,
so it agreed only while a unit was untouched and printed every wound as a
divergence; `0/5 myhits: ours 38 theirs 40` was one such, with the dump's
own `damage 2` making the two sides equal. `hits_left` and `myhits` are
both rows now — 54,000 more comparisons — and both agree on all 909
blocks.

### 34.5 Coverage

**Diff-backed**: §34.1's gate (the word block's `0/5` takes no attack
order, and `an_armed_citizen_does_not_take_an_attack_order_on_its_idle_
frame` is the rule against a built type); §34.2's bearing, ring, kind and
queue position (§34.3's table, every row of it, in
`run100_s_word_block_is_every_record_the_dump_carries`); §34.4's `myhits`
pair (the same test, over 909 blocks).

**Listing-backed**: `think@005f6e40:150`'s disjunction; the flee gate at
`6006f0`–`600731`; `find_angle`'s two arguments at `6007cb`–`6007f2`;
`find_nearby_spot`'s argument list at `6007f7`–`60082d`; `add_move_order`'s
`FLEE_TO`/`QUEUE_FIRST` at `60084a`–`600856`; and `1/28`'s `oxx` handoff,
which is read off the dump rather than the code.

**Not established.** The packing latch `local_8` and the `unit_flags & 4`
packer arm are unmodelled, and so are the hero and supply rings — the
latter need `is_hero`/`is_supply` and the attacker's vslot `+0x130`, and
no capture on disk has either unit taking a hit. The group arm at the head
of `target_opportunity` — a grouped, non-combat-role victim forwarding to
`Group::target_opportunity` — is still the seam `docs/GROUPS.md` §13
names. And `0/5`'s gather residue above is named, not explained: whether
the original's human citizen is kept from the job by the leader flag
`Unit::think@005f6e40` tests at `5f7179`, by its own idle threshold, or by a
`find_gather_spot` that simply finds nothing, is the next question and is
not answered here.
