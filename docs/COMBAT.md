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
~~caravans~~ the three merchant ids and caravans (`0060ec72`; **the one term
this crate carries**, `Sim::unit_hits`, item 1268: run492's Nubian Merchant
is 135 on a type of 90, `docs/GOLDEN.md` §54), `+ supply_hp_upgrade[level]`
for supply. The result is written to
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
    else `2`. ~~(`dtype` is cosmetic downstream — it picks the death
    animation's facing — and is carried but not modelled.)~~ **It is
    modelled from item 491 and it is not cosmetic**: `Unit::close` spends a
    draw on it, and three when it is 4 (§42.1). The default is 2, written
    at the head of the function.
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
    is a register the decompiler lost (`extraout_EDX`). ~~The natural
    candidate, and the one taken, is `amask`: a vehicle or cavalry
    **attacker** takes a reduced flank bonus. Open question 2.~~ The
    listing says it is **`tmask`**: a flanked vehicle or rider takes the
    reduced bonus (§53.1, item 601).
20. **Net damage:** `dmg = (base + 5) / 10 − armor`. Attack was in tenths;
    this is the rounding back to whole hits, and armour is subtracted
    *after* every multiplier above.
21. **Overkill** (only when `check_overkill`, which `do_damage` always passes
    and `compare_target` never does, §53.2):
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
   Build-proper. **Both are overrides of §6 step 11's value, and the death
   draw is what reads them** (§42.1). The `0x10` test is
   `GraphicPieces::verify_ammo_flags(ammo[index].gpiece, 0x10)` and runs
   only when the ammo index is `>= 0`; the Build-proper test is the
   target's vtable `+0x20`, and it runs after it, so a `0x10` piece hitting
   a building is 1.
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
3. Combat only (`attrition == 0`): the owner's last-attacked frame (~~AI,
   on low difficulty~~ **the struck object's owner, human or computer**, at
   `difficulty < 2`: `leaders[who].frame_attacked = frame`, lines 67–71,
   carried since item 729, `docs/AI.md` §71); on the **first** damage to a building (`damage == 0`)
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
   what stands where it lands (§9.3, §9.4). Built, and the point is the
   target's own position (§57).
2. `unit_masks |= 0x11000` (in combat, attacking). `set_attack(o, who)`:
   ~~every figure's~~ the `guy_mark` figures' `attack_o`/`attack_who` are
   set and, for a ranged type with pivot restrictions, `Guy::set_all_pivots`
   — ~~a return that means "wait for the pivot", which makes the unit skip
   the attack this frame (the `param_4 != 0` early-out below)~~ a return
   that means **"the pivot bears"**: the unit keeps its heading and the
   attack goes ahead this frame (§52, item 595; run145's chariot shoots on
   633 without turning).
3. **Facing**: `angle = find_angle(T − A)`; for a wall target, the angle is
   snapped to the side of the wall the attacker is on; a `GUN`-flagged type
   (`unit_flags & 0x40`) that is not a `PATROLBOAT` vs a sea target snaps to
   whichever of `angle ± 90°` is nearer its current facing (broadside). If
   the pivot ~~said wait~~ bears (§52), `angle = A.angle`. `set_angle(angle)` if it changed;
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
whenever the object adjacent to it (`near_o`) is in range. **Only a finished
one** (item 1323): `Build::process@0061edf0` returns straight after
`Wall::process` unless the building is active — `61ee1b` compares the
vtable with `Build::vftable@00b42174` and reads `flags & 4` inline, else
calls vslot `+0x4c`, which the PE holds as `0x472350`,
`WallData::is_active` (`flags & 4` again) — and `61ee31` jumps to the
epilogue at `0x6201c3`, past `do_attack` and everything after it. A site
never shoots: run514's Tower site `0/2008` holds `attack_ox −1` from its
laying to its death on 901 (`flags 19`, bit 4 clear), under who=1's
squad from 681. Its `recharging` meanwhile counts **up**, one a frame
while a builder works it (`Wall::do_construct@006434d0`, `+0x7a`, 0 on
622 to 56 on 679, held once the builders walk off on 680), and
`Build::activate@00623e20` zeroes it; this crate carries neither, and
nothing reads the field on a site. `do_attack`:

- *(second reading — the first draft had this inverted)* A building without
  ANTI_AIR, or a `LOOKOUT`/`OBSERVATIONPOST`: if `recharging` is non-zero,
  decrement it and return. Any **other** ANTI_AIR building skips the reload
  gate altogether and is refused at the firing step below — its shots go
  through the `do_launch` path, not this one.
- Every 32 frames the seen-by mask is cleared; `hits_left() == 0` returns;
  `is_jammed` (a radar-jammed building) sets the misfire flag and returns;
  `get_garrison_arrows() == 0` returns.
- ~~Without an explicit attack order (`build_masks & 4`), `find_target` —
  `Object::find_nearby_target(max(x_size, y_size) + 2 × max_range) × 0x60`
  (§12) — whenever it has no target, and every 32 frames (`(frame + o + 14)
  & 0x1f == 0`) when its current target is a unit that is **not moving** and
  is not a `role & 0x10000` spellcaster: the tower prefers a moving target
  and re-looks for one every two seconds. An invalid target → `find_target`.~~
  **The listing, `622a37`–`622c1c` (item 1131)**: without an explicit attack
  order (`build_masks & 4`, which `Group::action_attack` sets at `712654`),
  `Build::find_target@00622c80` — `find_nearby_target((max(x_size, y_size) +
  2 × max_range) × 0x60)` (§12), clearing the order bit — runs on **every**
  call (`622a3f`). `compare_target`'s current-target ×2 (or /2 with two
  arrows) is what holds a target (§12.3). Then: no target → cleared, return
  (`622c1c`); `valid_target` refuses → `find_target`, return without a shot
  (`622b25`); unordered, on `(frame + o + 14) & 0x1f == 0` → `find_target`
  and return, **unless** the target is a unit that is not moving
  (`UnitData::is_moving`, the front order's vslot `0x14`: 0 for a
  `StrafeOrder`, read off the PE) and is combat-role (`type +0x2c8 &
  0x10000`) or casting (`action_type == CAST_SPELL`); out of range
  (`is_in_range@00648d70`, `622b45`) → the target is cleared (`622c1c`).
  `Build::process@61ee64` runs `do_attack` every frame a target is held,
  else on `(frame + o) & 0x1f == 0`, else when `near_o` is in range (a SEAM
  here, parked 1118). A dead target is **not** cleared by anything: run404's
  Radar prints `attack_ox 6` on block 1007, after `0/6` was shot down on
  1006, and turns to `0/7` on tick 1007.
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

> ~~`event.time = starttime × 3 / 200`, **truncated**.~~
> `max(1, starttime / 67)`, §50.1.

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

Four for four on the truncation, nought for four on the rounding; `/ 67`
gives the same four (§50.1).

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
in y — ~~the decompiler dropped the operands; the natural reading, taken here,
is the target's heading and per-frame speed~~ they are its `angle` and its
first figure's `avg_speed` (§47.4), so the shot is aimed where the target will
be. Not for ground shots, bombers or lofted pieces.
`splash_area = A.type.splash_area`; `flags |= 2` (live); `cur_time = 0`;
`num_guys = A.guy_mark` (unit) or 0.

### 9.2 Flight — `Ammo::inc_time` (every frame, all ammo, after the objects)

`cur_time += 1`; ~~a target that is no longer active has its `hold_frames`
bumped~~ — **the shooter's slot, not the target's: §42.3.**
A live ammo with `cur_time <
total_time` returns (a rolling one
recomputes its graphic position in floats — cosmetic). At `cur_time >=
total_time`: a **rolling** shot (flag 4) that has not yet missed (flag 8)
does `hit_target`, then `check_hit(GROUND)`, and if neither found anything
sets flag 8 and keeps going along its line, up to `3 × total_time`, until the
terrain is above it (float terrain heights; ~~cosmetic to the outcome except
that a rolling miss can land on something further along — open question 5~~
— **not cosmetic; §42.2 is the frame it cost**);
then `graphic_finish` and, unless cosmetic (flag 0x10), `Ammo::do_damage`.

### 9.3 Impact — `Ammo::do_damage`

If the target is no longer active it is forgotten (`whom = ox = −1`). Then:

**No splash** (`splash_area == 0`): `hit_target()`, and if that fails
`check_hit(domain of the original target, or GROUND)`. If there is now no
target, or the target is the shooter itself: a landing point is picked
`±20` around `(ex, ey)` with two more `Random::get` draws and the ground is
punctured (cosmetic); return. **A rolling shot reaches this arm only after
its roll** (§42.2). Else if the target is active: `Object::
do_damage(A, ox, whom, ~~find_angle(launch → target)~~ find_angle(ex − sx,
ey − sy), num_guys, index, 0x100, 0, 0)` (§47.3).

**Splash**: (nuke and missile-shield branches aside) `hit_target()` /
`check_hit()` as above — then, for every object on every cell of the
**square** the spiral table `move_x/move_y` walks to `radius[k]`, `k =
splash_area / 4 + 1` capped at 10 (the `(2k+1)²` cells within Chebyshev
distance `k` of the landing cell — 3×3 for a splash of up to three tiles;
*second reading: the first draft had a circle of `splash_area / 4`*), walking
each cell's object chain through `down/down_who`, skipping the shooter,
players past 7 and, **for a non-target**, its side and allies
(`6787d9`), **damaging the intended target with `splash
= 0` and everything else with `splash = 1`**: a unit that is active, on the
map, in the same air/ground class as the ammo's domain, not a missile:
`d = max(0, vector_dist(landing, unit) − 0xc0 − unit.type.guy_radius)`;
`count = 0x100 − (d << 8) / (splash_area × 0xc0)`; if `count > 0`:
`do_damage(A, unit, …, count, splash, 0)`. A building, for a non-air ammo:
`d = vector_dist(|Δx| − x_size × 0xc0, |Δy| − y_size × 0xc0)`, each
axis floored at zero (the listing, ORDERS §35.3); `count` likewise; `do_damage` if `count ≥ 0`. The `splash`
argument is what makes `get_damage` apply step 15 and lets the fringe do
nothing (step 26).

### 9.4 The hit test — `Ammo::hit_target` and `check_hit`

`hit_target`: the target must be active and on the map. For a **unit**:
`vector_dist(landing, T) ≤ T.type.target_size` — with the distance **halved
when `accuracy > 100`**. For a **building**: the landing point must be inside
the footprint, `|ex − T.x| ≤ x_size × 0x60` and `|ey − T.y| ≤ y_size × 0x60`.
Hit → 1; miss → forget the target, 0.

`check_hit(domain)`: `ObjectsData::find_unit` at the landing point, radius
`0x180` (two tiles), not its own for an aircraft
(search 6), and
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
[`trace::SITES`](../crates/rondata/src/trace.rs) since item 389; at landing, two more for where a no-target shot punctures the ground — **not a rolling
one on the frame it was due** (§42.2);
in `take_damage`, ~~one `% 100` on the first wound of a fort, temple or
town~~ — **wrong; §20 measured it**: the `% 100` is *any* building's first
wound, the fort test gates only the flock, and one frame can spend it
twice; **one for the death animation of any unit the hit kills** (§42.1),
and two more when its `dtype` is 4; and, outside
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
unregistering from the world, the supply list, the army, the group — **and
one `Random::get` for the death animation, §42.1**), then,
for a type with range, `hold_frames = max(1, for every live ammo this
object fired: total_time − cur_time + nuke_effect[0x108] + 1)` — **the slot
is held until its last shot has landed**, so that a dead archer's arrow
still hits. The ammo is matched on its **shooter** (`Ammo +0x3c`/`+0x40`),
which is the same pair `Ammo::inc_time`'s per-frame bump reads and the
correction §9.2 carries. A `UnitData` in the chain dies alone; `Unit::die_uber` (the
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
max_dist` skips; the **range gate**: ~~a unit in STAND_GROUND, or entrenched
without the Antipater bit, or an unpacked packer, considers anything
(`anything`)~~ a unit in STAND_GROUND, or entrenched without the Antipater
bit, or an unpacked packer (`local_24`) **must be `is_in_range` of the
candidate or skip it**, and is scored with `in_range = 0` (item 627, the
listing, §60.1); otherwise a unit that is not guarding (or whose guard target
has attack) is deemed `in_range` without testing, and everything else must be
`is_in_range` or be a unit, have attack, or be a wonder (then `in_range =
0` and it is still scored); **distance shaping** for units: a ranged type
with no minimum range and `dist + 0x180 < max_range × 0xc0` halves `dist`;
inside `min_range`: `dist = min_range×0xc0 + (max_range×0xc0 − dist)`; then
**`dist += (target.targeted + 8) × 0x30`** — every attacker already on it adds
a quarter tile; `value = compare_target(o, who, in_range, ai)` — and
**`in_range` is a permission to test, not a verdict**, see §12.3 —; **`score =
value / (dist / 0xc0 + 1)`**; the previous mandatory target halves; the
`flags` preferences halve the wrong class (the word and its filter: §64); a cavalry archer's second-weapon
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
get_damage(o, who, ~~angle 0~~ the bearing (§46.5), splash 0,
check_overkill 0 (§53.2))`: `v ×= dmg` (AI: `v /= dmg`, or 0). Building
targets (not raiding): armed and not human → `+1,000,000` with the SIEGE mask
else `×5`; siege vs armed `+100,000`. *(Item 1131, the listing
`64f124`–`64f1ed`: "human" is the **attacker's** `leader_flags & 4`
(`testb $0x4, 0xe3a390(who · 0x6eec)`, `jne` past the weight); "armed" is the
target type's `attack`, zeroed for an ANTI_AIR target of an attacker not of
the air domain. The arm between (`64f171`: a target whose object flags carry
`0x20` takes the weight only with `num_inside` non-zero) is a SEAM: read as
"a city", it moved Great Lakes' second game from its close at 5930 to 5158,
and what the bit is was not read. This crate gave the ×5 to every attacker
until then, and run404's human `0/7` re-pointed its strafe at the
Radar on tick 818 for it.)* Unit targets: combat-role `×20`;
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
over what the simulation has, and leaves the ~~raid,~~ spell, stealth and AI
branches as inputs. See `combat::compare_target`. **The raid arm is built
(item 1028, §68.1)**, and the weights above are corrected there: a
computer's land raider takes `+900,000` for a peasant or a caravan and
`+10,000` for a combat unit; the `+9,000,000` is a human raider's.

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

2. ~~**Whose mask the cavalry/vehicle flank reduction reads** — the attacker's
   is taken; the register was lost.~~ The target's, read off the listing
   (§53.1, item 601).
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
same `value` for three identical undamaged hoplites ~~(its `get_damage` is
called at `find_angle(0, 0)`, so bearing cannot separate them)~~ (the call
takes the real bearing, in `ecx`/`edx`, §46.5). A tie is
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
  this disk reaches a siege attacker's first wound on a fort, temple or town;
  `add_flock` itself is `docs/AI.md` §99.12's, from a group move. The
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
  ~~The Slinger's three families are measured in run17 and are *not* in the
  table, because no capture ties them to an animation — run17's `GUY` detail
  is 1, so it prints no `cur_anim`.~~ **Tied by item 1040 (§70.2)**: the
  unit's own `recharging` names each stone's release event, and piece 32's
  three are `launch::BAYS` rows. A `GUYS=4` re-run of any Slinger fight
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

~~**This crate does none of it, and that is a named residue with a
consequence.**~~ Landed as [`sim::Sim::add_ammo`], §46.4, on the frame
this predicted. `Sim::projectiles` is a `Vec` that `process_projectiles`
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

~~Neither is a number this crate can hold: no float in the simulation, and a
ballistic `z` is one of the few places the original genuinely needs one.~~
`v1z` now is, bit for bit, in [`sim::single::Single`] (§46.1). So
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
`docs/ORDERS.md` §4.4 — ~~each of them only makes the kill rarer, so
none can be costing a frame in the same direction.~~ **The inference was
wrong** (item 470, §35.3): rarer is a *direction*, and chapter two's
kill fires too **early**, not too late. The `mandatory` conjunct is
worth three frames on `0/11` there, measured both ways.

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
   that arm is already dead, so only its ~~type-record arm (`UnitTypeData
   +0x9a`, bit 6)~~ **plane arm** (the target's first figure's `guy_flags &
   0x40`, which only a plane's figures carry, §61.3) and the cell byte
   survive — and the cell byte is read
   from the *same cell* in both rows of the table above, which kills it
   too.

Written out, that leaves `valid_target` and `poor_target`'s ~~type-record~~
plane arm (dead for a bowman, §61.3), and this section stops there rather than choosing: §26.2, §26.4,
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
unbooked and run113 unspent. **It is not needed**
(item 650, §61.3): the byte is the first figure's `guy_flags`, which
`Guy::init_real` sets for a plane alone, so a bowman's and a slinger's
read alike.

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
  `Object::poor_target@0064a270` is reached, and its ~~type-record arm
  (`UnitTypeData +0x9a`, bit 6)~~ **plane arm** reads the first figure's
  `guy_flags & 0x40`, set on a plane's figures only (§61.3), so it cannot
  fire on a bowman. What
  changed is that it can no longer move chapter two's word: the frame it
  would explain now agrees. ~~The capture §30.5 named — `UnitTypeData
  +0x9a` per unit over `[614, 640)` — remains unbooked and unspent, and
  it is the only way to settle that arm on its own terms.~~ The arm is
  settled by the type record and the listing (`0064a2c2`–`0064a2ca`),
  and no capture is owed.
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

**Status — landed on item 470** (2026-09-21). Everything in this
subsection was established against run112's own dump on item 466,
implemented, measured and then *withdrawn*; it went back in whole on 470,
where [`Sim::compare_target`] gained the fourth argument and
[`Sim::search_ai`] the gate that computes it. 635's three `Target` rows
are closed and the values now part at **636**. It cost
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`, which is
red on purpose; §33.4 and §35 have why, and §35 has the reason that
subsection's *predicted* fix was the wrong one.


`Object::compare_target(o, who, in_range, ai)` at `0064ef4b`:

```
dmg = get_damage(this, o, who, find_angle(o - this), 0, 0, NULL);
if (ai == 0) v = v * dmg;
else { if (dmg == 0) return 0; v = v / dmg; }
```

~~`find_angle(0, 0)`~~ was the decompiler's: the pair is `ecx`/`edx`
(§46.5).

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

`chapter_two_s_word_frame_is_widened_whole` walks `[633, 641)`; with
§33.1 alone its map is `order 1/6`/`1/7`/`1/8` at **635**, `order 0/11`
and `pos 0/11` at 636, `pos 1/6`-`1/8` at 636, `visible 0/11` at 637,
and the citizen `0/5`'s two rows at 639 and 640. The bowmen's three
`Target` rows are gone and the three `angle` rows that followed them on
636 went with them. **The word held at 637 under both halves**, which is
what decides the rest of this subsection: neither moves the headline.

The two causes split cleanly by squad, measured by disabling each in
turn:

| landed | 635's `Target` rows | `chapter_two_s_visible_byte` |
| --- | --- | --- |
| §33.1 alone | bowmen closed, `order 1/6`-`1/8` still 635 | **green** |
| §33.2 alone | hoplites closed, `order 0/6`-`0/8` still 635 | **red** |
| both | none — 635 clean | red |

§33.1 is free; §33.2 costs the **shape** assertion, whose own comment
says nothing about the engagement's timing can excuse a failure there:
`1/8` strikes nothing in 899 frames, so its bit never arrives, and the
row drops `((1, 8), 1)`. It fails first, so the nine pinned frames and
`exact == 5` below it are never reached.

**And the shape row passes today by accident** — the finding this
subsection exists to record, and it outlives either half. Without §33.2
`1/8` strikes, but only because it chases the wrong target and only
because that wrong target hands it a slot of its own: three hoplites plan
**two** slots between them, `(1320, 7800)` twice and `(1320, 7944)` once.
Given the dump's own target they plan **one**, `(1512, 8040)` on every
one. So the green rests on a wrong two-slot assignment.
`find_open_slots` was already collapsing two chasers of three before this
item; the correct ranking reveals that rather than causing it. A pinned
assertion that passes for the wrong reason is worse than one that fails,
because nothing in the tree says so — this paragraph was the only record
that it did.

**Both counts re-measured on item 470's tree, after 465's group pool
landed under it**: without §33.2, `(1320, 7800)` twice and
`(1320, 7944)` once out of **twelve** open slots — two slots between
three chasers, unchanged from 466's figure; with §33.2, `(1512, 8040)`
on all three out of **ten**. 470 then closed the collapse itself
(§35.1), so the green rests on three slots and not on two.

~~The mechanism is §19's `find_ordered_collision` **chain** reach, which
cannot see a squadmate 1,200 units away.~~ Half right, and §35.1 has the
measured half. The chain cannot see them — and the crate's *group* pass,
which was written for exactly that case, was reading the **last slot
pushed** rather than the asker's own group, so it saw nobody's members.
Reading the asker's group (`65b4d4`) hands the three hoplites three
slots and `1/8` strikes. The **destinations** are still wrong, and for a
different reason: the original's `0/11` is *moving* when the hoplites
ask, so `find_melee_pos` is never called there at all.

### 33.6 Re-measured against 464, which landed alongside

Item 464 landed while this one was held, so everything above was measured
again on the merged tree. **Nothing moved**: word **637** (sequence 637,
`game_random` 638); 635 carries the hoplites' three `Target` rows and
nothing else; `chapter_two_s_visible_byte_…` green; §33.4's slot counts
unchanged — **two** slots between three chasers without §33.2, **one**
with it.

Neither of 464's changes could reach this window, and the reasons were
stated rather than assumed: its health fix is in the **other** harness
(chapter two's walk carries no health row, and all nine units are
`damage 0` for all nine blocks), and its `role & 0x10000` gate does not
touch units carrying the military bit.

**That chapter two's `compare` carries no health row is a real gap**,
just not a live one here, and not this item's to close.

**What the re-measurement did change**: the widening enumerated **ten**
of `FrameResult`'s fourteen vectors — `gather_diverged`,
`build_diverged`, `queue_diverged` and `city_diverged` were never noted,
so a divergence in any of them left the map silent. All four are in the
walk now and the map is **unchanged**, which is the only way to learn
they were empty rather than ignored.

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

**Diff-backed since item 470**: §33.2's table. The ranking is in the
tree and `chapter_two_s_word_frame_is_widened_whole` checks its product
on every commit — the hoplites' three `Target` rows are gone from 635 and
`order 1/6`, `1/7` and `1/8` stand at 636.

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
`GROUP_MOVE`'s **`oxx` goes 40 → 28**. ~~It stops following `1/40` and
becomes its own group's leader on the frame it does nothing — and
`do_group_move` runs `do_move` for the leader alone (`docs/GROUPS.md`
§8.3), which is why the plan is on the next frame and not on this one.~~
This crate's `1/28` holds a plain `MOVE_TO` there, not a `GROUP_MOVE` at
all, so it has no handoff to wait for. That is the next item on this
frame, and its falsifier is the `oxx` pair above.

**Struck, and the falsifier named above is what struck it** (item 465,
`docs/ORDERS.md` §16). `oxx 40 → 28` is not `1/28` promoting itself: it
is the whole of group 65 on that block — `1/27`, `1/28`, `1/29`, `1/40`,
`1/41` and `1/42` every one — so it is `Group::refresh_group_order`
re-seating the block, and its **early return** is what costs the frame.
`do_group_move`'s follower arm does plan and step for a follower whose
leader is usable, which `1/27` proves on 10239 by doing exactly that.
The reading above came from one unit's value diff; the whole cast on the
same two blocks dissolved it, which is now the third time on this frame
(`docs/DECISIONS.md` 42). The cause was upstream of the symptom and it
was this crate's own: an invented `g.army.is_some()` line in
`Group::action_move_near` that the original's gate does not have.

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
`FLEE_TO`/`QUEUE_FIRST` at `60084a`–`600856`. ~~and `1/28`'s `oxx` handoff,
which is read off the dump rather than the code~~ — struck by item 465;
the `oxx` pair is group-wide and its reading is `docs/ORDERS.md` §16's.

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

---


## 35. The group pass was reading somebody else's group (item 470, 2026-09-21)

Three things landed and the score moved on two of them. §33.2's `ai` arm
is in the tree: 635's three `Target` rows are closed and the values part
at **636**. `find_ordered_collision`'s group pass reads the **asker's
own** group: the three hoplites take three slots instead of one and
`1/8` strikes for the first time, which puts
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` back on an
honest footing (§33.4). And the widening window's floor came down to
run112's first block, which put a row on the map that nothing had ever
reported. Word **637**, unmoved (sequence 637, `game_random` 638).

**The item was booked on a mechanism and the mechanism was half wrong**,
which is the fifth time on these frames (`docs/DECISIONS.md` 42). The
half that was right was the function; the half that was wrong was the
line inside it, and the destinations it produces are still not the
original's for a reason that has nothing to do with either.

### 35.1 One slot, three chasers — and it was not the chain walk

A probe inside `Unit::find_attack_pos`'s melee arm, printing each asker's
answer, its group, and its squadmates' `orders_x`/`orders_y` at the
moment it asks:

```
f=635 melee 1/6 target=(0,11) group=3 -> (1512, 8040)  open=10
         mates orders_pos=[(6, 2424, 7800), (7, 2568, 7800), (8, 2472, 7944)]
f=635 melee 1/7 target=(0,11) group=3 -> (1512, 8040)  open=10
         mates orders_pos=[(6, 1512, 8040), (7, 2568, 7800), (8, 2472, 7944)]
f=635 melee 1/8 target=(0,11) group=3 -> (1512, 8040)  open=10
         mates orders_pos=[(6, 1512, 8040), (7, 1512, 8040), (8, 2472, 7944)]
```

§33.4's reading was that the three are invisible to each other because
`find_ordered_collision` walks the object chains of the cells around the
**slot** and they stand 1,200 units east of it. True of the chain walk —
and beside the point, because the second pass exists for exactly this
case (§17.4) and the middle line above shows it had the answer in its
hand: when `1/7` asks, `1/6`'s `orders_pos` is **already** the slot
`1/7` goes on to take.

It was looking in the wrong place. `65b4d4`-`65b58c` takes the group off
the asker's `unit +0x80`; this crate took the **last slot `push_group`
filled**, which is where the reader stood when the pool was one slot and
which item 465's own comment on `Sim::pushed_last` said in as many
words. With a pool and [`Sim::group_of`] to resolve it, the asker's own
group is answerable, and then:

```
f=635 melee 1/6 -> (1512, 8040)  open=10
f=635 melee 1/7 -> (1512, 8328)  open=9
f=635 melee 1/8 -> (1512, 8472)  open=8
```

Ten, nine, eight: each taken slot is gone for the next asker. `1/8`
reaches its target, strikes, and gains player 0's `visible` bit at 675
where it had never gained one at all. `Sim::pushed_last` has no reader
left and is gone with the fix.

**The destinations are still not the dump's** — `(1416, 8328)`,
`(1128, 8424)`, `(1368, 8472)` — and the reason is upstream of this
function and of the slot ring:

```
f=635 do_move/attack u=0/11 t=(1,8) max_range=6 attack_dist=1130
      reach=1158  ->  kill_current_order
```

§17.3's arm calls `Unit::find_melee_pos@006010b0` only when the unit
target is **not moving** (§19), and run112's `0/11` carries a
`MOVEORDER` on every block from 622 to 638 — it is walking to
`(1704, 8424)` the whole time the hoplites choose. The original never
enters this function here; its askers take `find_attack_pos`'s sweep,
which is why their three points sit at three different Chebyshev radii
from the target (`(+4, 0)`, `(−2, +2)`, `(+3, +3)` quarter-tiles) rather
than down one side of a square ring. **No square ring can produce that
set**, and the dump's own coordinates said so before any probe ran.

So the chain is four links long — the chase dies three frames early →
the target is not moving → the melee arm is entered at all → the slots
are shared — and item 470 fixed the fourth. The first is §35.3's.

### 35.2 The window's floor was above a live row

`chapter_two_s_word_frame_is_widened_whole` walked `[633, 641)` — "the
word, and three frames either side". Widening it to run112's **own first
block** puts a divergence on the map that no run had ever reported:

| row | frame | whose |
| --- | --- | --- |
| `order 0/10`, `pos 0/10` | **630** | nobody's — pre-existing |
| `order 0/11`, `pos 0/11` | 636 | §35.3's |
| `order 1/6`, `1/7`, `1/8` | 635 → **636** | closed by §33.2 |

`0/10` is the other slinger on the same chase. This crate drops its move
on the first frame it reads in range; the original drops it **one frame
later**, at the same `attack_dist` of 1108 against the same reach of
1158 — both sides are at `(1206, 8207)` when this crate kills and at
`(1227, 8186)` when the original does, and the two snap to one
quarter-tile, so the distance is identical on the frame each side
decides. ~~The difference is state, not geometry, and it is not settled
here.~~ **Settled by item 472, §36**: the state is a sixteen-frame
phase, and the mechanism is `Unit::check_target_path` rather than
anything in `do_move`. The row is gone and the word is 645.

The row was measured **both ways**, with §33.2's ranking and with it
disabled, and stands identically. It is recorded because a floor chosen
to sit just under the word is the same shape as §33.4's unearned green:
an instrument that agrees because it is not looking. The floor is now
the dump's first block and the map is pinned whole.

### 35.3 `is_in_range`'s sixth argument — measured, and not landed

`do_move@005f7b30:216` is the **only** call to `ObjectData::is_in_range`
in the executable that passes a non-zero sixth argument. Every other one
— across `Unit::fight`, `Unit::think`, `Group::action_attack`,
`Object::find_nearby_target`, `Object::compare_target`,
`Build::do_attack` and a dozen more — passes `0`. What it carries is
`attack->+0x1c == 0`, which the type record names
`AttackOrder::mandatory`, and what it does at `006486b0` is add `+0x90`
to the measured distance before the max-range test. So a
**non-mandatory** chase is dropped three quarters of a tile inside the
attacker's reach rather than at its edge. `docs/ORDERS.md` §4.4 has
carried the conjunct as unmodelled since the second reading;
[`combat::in_range`] has carried the parameter, misnamed `melee_bonus`,
since it was written, and **nothing has ever passed it `true`** — the
same shape as the `0xf6` reach item 384 found.

It was implemented and measured, and it was **not in the tree** until
item 472 landed it beside §36's review:

| | `0/10` parts | `0/11` parts | the hoplites' rows |
| --- | --- | --- | --- |
| without the margin | 630 (one frame early) | 636 (three early) | 636 |
| with the margin | 631 (one frame **late**) | 639 (the dump's own) | back at **635** |

It is right about `0/11`, the unit this item is about, and it moves
`0/10` from one frame early to one frame late — and because `0/10` then
stands where the original does not, who=1's ranking at 635 changes and
the three rows §33.2 had just closed come back.
`chapter_two_s_first_attack_orders_are_the_dump_s` goes red with them.

So the conjunct is real, its effect is measured in both directions, and
landing it alone is a net loss on this map. What it needs is whatever
stops `0/10` at 631, which §35.2 leaves open — **`Unit::work`'s
sixteen-frame review, §36**. With the two together the map is empty and
the word is 645. The measurement is written
down so nobody runs it twice, and so the next reader of `melee_bonus`
knows the name is wrong twice over: it is not a bonus, and it never
reaches a melee attacker, whose arm returns before the test.

**And it refutes an inference, not just a gap.** §25.5 said the
unmodelled conjuncts "only make the kill rarer, so none can be costing a
frame in the same direction". Rarer is a *direction*. Chapter two's kill
fires too **early**, and the conjunct is worth three frames on `0/11`.

### 35.4 What the `visible` row now says

The shape row is green because all nine units gain a bit, `1/8`
included. Its **frames** row — pinned in no direction, by its own doc
comment — moved on the three hoplites:

| | ours before | ours after | run112 |
| --- | --- | --- | --- |
| `1/6` | 677 | **669** | 672 |
| `1/7` | 694 | **678** | 698 |
| `1/8` | never | **675** | 665 |

Two of the three are closer to the dump and `1/7` is twenty frames
under it, which is the engagement's own residue and not this field's
(§31.6). `exact == 5` is untouched: the five arrivals this crate puts on
the dump's own frame are the three bowmen, `0/10` and `0/11`, and none
of them moved.

### 35.5 Coverage

**Diff-backed**: §33.2's ranking and the three rows it closes; the
three-slots-from-three-askers count and the ten/nine/eight open counts
(a probe inside `find_attack_pos`, re-measured after 465 landed
underneath); §33.4's two-slot figure, re-measured the same way; the
widened map including `order 0/10` / `pos 0/10` at 630, measured with
§33.2 both enabled and disabled; `1/8`'s first strike and its `visible`
bit at 675; that run112's `0/11` carries a `MOVEORDER` on every block
622 to 638 and `0/10` on every block 622 to 630 (the dump's own
records); `0/11`'s `attack_dist` of 1130 against a reach of 1158 on the
frame this crate kills its chase; §35.3's table, both rows.

**Listing- and export-backed**: `compare_target`'s `ai` split at
`0064ef4b` and the gate that computes it at `00648e6e`;
`find_ordered_collision`'s group pass at `0065b440` taking the asker's
own `unit +0x80`; `is_in_range@006486b0`'s sixth argument and the single
call site that sets it, `do_move@005f7b30`; `AttackOrder::mandatory` at
`+0x1c` from the type record; `find_melee_pos@006010b0`'s `is_moving`
guard (§19).

~~**Not established**: what stops `0/10` at 631 rather than 630~~ —
**answered by §36**: `Unit::work`'s sixteen-frame review. The rest of
this paragraph stood and is kept because its two negative findings
still hold: the `find_collision` conjunct beside the margin is false at
both of `0/10`'s positions (the two snap to one quarter-tile and `0/11`
is three unit cells away at each), and the `(o + frame)` retarget that
does fire ends in `change_target` rather than a kill — which is exactly
what the word at 645 now turns on (§36.6). The three hoplites'
destinations came right without anyone measuring
`find_attack_pos`'s flanking branch: they were link four of §35.1's
chain and fixing link one fixed them (§36.5).

## 36. A chase ends on a clock, not on a radius (item 472, 2026-09-21)

§35.2 left `order 0/10` / `pos 0/10` at block **630** with a reading
rather than a mechanism: "the difference is state, not geometry, and it
is not settled here". It is settled here, and the state is a
**sixteen-frame phase**. `Unit::work@0060d180:440` reviews a walking
unit's chase one frame in sixteen, phased by `o`, and
`Unit::check_target_path@005e22d0` ends it when the target is in reach.
Ours ended it on the first frame it read in range; the original does not
look on that frame.

With `is_in_range`'s sixth argument beside it (§35.3, now landed), the
word moves **637 → 645**, the value parting 636 → 646, and the whole of
`[606, 645)` — every record run112 carries, both directions — goes to
**nought**. `visible`'s exact count goes 5 → **8** of nine (§36.5).

### 36.1 The geometry was bit-identical on the two frames

`attack_dist` snaps both sides to the quarter-tile, and run112's `1/8`
stands still from block 621 to 635. So `0/10`'s own dump rows read:

| block | `0/10` at | snapped | `attack_dist` to `1/8` | the original |
| --- | --- | --- | --- | --- |
| 629 | `(1206, 8207)` | `(1200, 8160)` | **1108** | walks on |
| 630 | `(1227, 8186)` | `(1200, 8160)` | **1108** | kills the move |

The two positions snap to one cell, the target has not moved, and the
reach is 1158 on both. **No threshold can produce that pair** — and the
pair is the whole proof that the answer is a clock. `0/11` seals it from
the other side: the original kills its chase at `attack_dist` **979**
and declines at **1082**, so a single radius would have to be at once
under 1082 and over 1108.

The frame numbers say what the radius cannot. A kill's tick is the sim
frame that *produces* the next block, so `0/10`'s is 630 and `1/8`'s (at
block 665) is 664: **`(o + frame) % 16 == 0`** on 640 and 672, and on
`1/7`'s 704. Every tick the original declined fails it.

### 36.2 `Unit::work@0060d180:440` — the review, and where it sits

For a head order of the move family, before the dispatch:

```
if ((ptype +0x2b8 & 4) == 0 || (unit_masks & 0x80000) != 0)
  if ((frame + o) & 0xf == 0
      && (a = update_action()) != 0 && a->vt+0x20() != 0 && a->type != 9)
    if (a->type == 0xc)  { every 64th: repath; }
    else if (head->type != 0x12 && (head->vt+0x2c() == 0 || head->vt+0x94() is me))
      if (check_target_path(a->vt+0x3c()))  goto 0060d710;   // re-read the head
```

`0060d710` re-reads the head with `update_order` and dispatches on the
**new** type, so a chase this ends is answered by `do_attack` — and
therefore by `Unit::fight` — **on the same frame**. `do_move`'s own
in-range kill (§35.3) returns instead, and the attack waits a frame.
That difference is printed in the dump and is how the two are told
apart without a probe: `AttackOrder::ever_in_range` (`+0x1f`) has
exactly one writer in the executable, `Unit::fight+0xba9`.

| unit | block the move dies | `in_range`/`ever_in_range` | so the killer is |
| --- | --- | --- | --- |
| `0/10` | 631 | **same block** | `check_target_path` |
| `1/8` | 665 | same block | `check_target_path` |
| `1/7` | 698 | same block | `check_target_path` |
| `0/11` | 639 | next block | `do_move`'s in-range kill |
| `0/9` | 645 | `in_range` 1, `ever` **0**, `ox` 8 → 6 | `do_move`'s captain `change_target` (§36.6) |
| `1/6` | 648, 671 | unchanged | arrival (`pos == dest`) |

Those are **all seven** of run112's `[ATTACK, MOVE] → [ATTACK]`
transitions, and the phase agrees with the split: 640, 672 and 704 are
the three that are `0 (mod 16)` and they are exactly the three rows the
flags date to the same block.

### 36.3 `check_target_path`'s first arm

`005e2434`-`005e24d4`, for an action of type 10 on a seen, active unit:

1. **The flank triple**, `do_move@005f7b30`'s own at `005f7fbe`, with one difference that
   matters: `e = target.angle − find_angle(t.x − my.x, t.y − my.y) +
   0x80000000`. The reference is the **bearing to the target**, not the
   attacker's own heading. `e < 0x2aaaaaaa` is tested inline and jumps
   past the call, so the predicate a caller means is `0x2aaaaaaa <= e &&
   flanking(e) != 0`, which is `|signed difference| <= 120°`: the target
   is facing away from me. A flanked target that `is_moving` is **chased
   rather than shot at**, and the review returns 0.
2. `is_in_range(o, who, …, 0)` — the five-argument overload at
   `00648d70`, which forwards the attacker's own position and passes the
   sixth argument **zero**. So the review uses the plain radius where
   `do_move`'s kill uses the radius less `0x90`, and there is no
   `max_range != 0` gate: a melee attacker is reviewed too, which is
   why `1/7` and `1/8` are on the list above.
3. `Unit::repath` — pop the transit legs — and return 1.

`flanking@0092cfe0` is three instructions and is in
[`combat::flanking`]; its 1/2 answer is read by nothing.

### 36.4 What is not established

- ~~**Everything past the range test.** The original falls through to
  `find_attack_pos`, `add_move_order`, `find_new_target` and
  `Group::action_attack` when the target is **inactive**, or when the
  flank triple holds. No capture on file reaches either arm — run112's
  four reviews all have a stationary target — so this crate returns
  false there and the arms are unmodelled.~~ **The fleeing unit's arm is
  §67** (item 1023, run347's `1/24` on 4616), **and the inactive unit's
  is §76** (item 1086, run347's `1/13` on 5011). The group arm and a
  building target's fall-through stay unmodelled (§67.5).
- **The two guards above the review**: `ptype +0x2b8 & 4` with
  `unit_masks & 0x80000` (the packable lineage, which takes an
  `add_cast_order` branch instead) and the `GUARD` arm (`action type ==
  0xc`, a `repath` on its own sixty-four-frame phase). Neither is
  reached by any capture.
- **The group conjunct** `head->vt+0x2c() == 0 || head->vt+0x94()` names
  me. This crate asks `m.group.is_none()`, which is the same answer for
  every capture on file: all four reviews fire on a plain `MOVE_TO`.
- The `find_collision` conjunct of `do_move`'s own kill (§35.3's SEAM)
  is still unmodelled; §35's reading that it is false at both of
  `0/10`'s positions stands and nothing here tested it again.

### 36.5 What moved

Word **637 → 645**, sequence 645, values **646**. The widening window is
now `[606, 649)` and its map is five rows, all at 645-646 and all
`0/9`'s (§36.6); `order 0/10` at 630, `order 0/11` at 636, the three
hoplites' rows at 636 and the citizen `0/5`'s at 639/640 are **gone**.

`visible`'s exact count is **8 of 9**, from 5. The three that came over
are the three hoplites — 669/678/675 → 672/698/665, the dump's own —
and they came over because §35.1's chain was four links long and this
is its first: `0/11` is still walking when who=1 chooses at 635, so
`find_attack_pos` takes its sweep rather than the melee ring, the three
destinations are the dump's, and the strikes land on the dump's frames.
Item 470 fixed the fourth link and predicted this; it is measured here.

`rondata`'s endpoints do not move — 49/9/0/0/8/3/0 on Great Lakes,
unchanged, and neither ladder rung moves.

### 36.6 The new word, 645: the captain's `change_target`

The value diff on the frame the word moved, the dump's own coordinates:

```
645 pos   0/9   ours (942, 8098)      theirs (912, 8096)
645 order 0/9   Length ours 2 theirs 1 · Kind ours 1 theirs 10 · PathLength 3/0
645 order 0/10  Target ours (1,8) theirs (1,6)
645 order 0/11  Target ours (1,8) theirs (1,6)
646 visible 0/9 (ours 648, the dump 646)
```

This is the third of §36.2's mechanisms and the one not implemented:
`do_move@005f7b30`'s arm at `005f803f`, which runs only for a **captain** (`o_up < 0`
— run112's `0/9` has `o_up -1`, `0/10` and `0/11` have 9 and 10). It
takes an incumbent from `ObjectData::near_o`/`near_who` (`+0x34`/`+0x36`
by the type record — `0/9` carries `near_o 6, near_who 1` from block
643), asks `find_melee_target` as well on `(o + frame) % 16 == 0`,
takes the better of the two by `is_in_range`, refuses a
`Object::poor_target`, writes `AttackOrder::in_range = 1` at `005f820a`
and calls `Unit::change_target@005e36c0`. That function rewrites the
target in place and then **kills head orders until one answers
`vt+0x20`**, which drops the move — and it walks `o_down` to the whole
file, which is why `0/10` and `0/11`'s targets move to `(1, 6)` on the
same block without either of them deciding anything.

`ever_in_range` staying **0** at 645 is what identifies the path: only
`Unit::fight` writes it, and `fight` runs a frame later here.

**Confirmed and landed by item 479** (§37). The fingerprint held on the
first grep — `0/9` prints `in_range 1` with `ever_in_range 0` on 645 and
nothing else on the map does — and the `o_up` reading above is exactly
right: `0/9` carries `o_up -1`, `0/10` `9` and `0/11` `10`, so the
`o_down` chain is `9 → 10 → 11` and stops, which is why the bowmen
`0/6`-`0/8` keep `1/8` through the same block. One correction to the
last sentence: the walk is the **squad's**, not the file's. The dump
prints a second, unrelated `up`/`down` pair in the `OBJECT` block —
`ObjectData +0x2a`/`+0x2c`, the world cell's own occupancy chain, which
`Object::add_to_world@0064d8c0` threads and whose loop check is the
error string `UNIT LINKED LIST LOOPS <ADD>`. On 644 that pair reads
`9 → 11 → 10 → 8 → 7 → 6`, all six of who=0's soldiers, because all six
stand in cell `(1, 10)`; `o_up`/`o_down` are printed further down the
same record, after `play`, and they are the squad. Reading the first
pair for the second says the walk reaches the bowmen, and it does not.

## 37. A captain retargets to its cached incumbent (item 479, 2026-09-21)

§36.6 named `do_move@005f7b30`'s captain arm as its successor and did
not take it. It is taken here and it is the mechanism: a ranged captain
walking to a target it cannot yet reach asks, **every frame**, whether
the incumbent its last search left in `ObjectData::near_o` is one it
can — on the *plain* reach where the kill above it uses the reach less
`0x90` — and `Unit::change_target@005e36c0` then writes that answer down
the whole `o_down` chain **in place**.

The word moves **645 → 680**, the widening's map over `[606, 684)` goes
from five first-partings at 645 to six at **671**, and every record
run112 carries over the sixty-five frames `[606, 671)` — both
directions, every field, every unit — goes to **nought**.
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` goes
**8 of 9 → 9 of 9**: every one of chapter two's nine first strikes now
lands on the original's own frame.

### 37.1 `near_o` is the search's footprint, and this crate had no term for it

`ObjectData::near_o`/`near_who` (`+0x34`/`+0x36` by the type record) are
written by `Object::find_nearby_target@00648da0` and by nothing else in
the executable. `00649527`-`0064953f`, inside the ring walk:

```
local_4c = 9999999;                               // 00648e0e
...
if (check_target(this, o, who, …, &dist, …)) {     // 00649332
    if (dist < local_4c && (this->is_unit() || cand->is_unit())) {
        local_4c = dist;  near_o = o;  near_who = who;
    }
    if (max_dist < 1 || dist <= max_dist) { … the scoring … }
}
...
if (0xf00 < local_4c) { near_o = -1; near_who = -1; }   // 006498de
```

Three things follow, and all three are the difference between a
footprint and an answer:

- it is the **nearest** candidate, not the chosen one — the score
  (`compare_target` divided by the shaped distance) is computed below
  it and never read here;
- it is written **above the `max_dist` gate**, so a candidate too far
  to be ordered against still leaves its mark;
- it is **cleared** to `-1` when the nearest thing the rings saw is
  beyond `0xf00` — and cleared on an empty ring table too, so a search
  that finds nothing overwrites a good incumbent rather than leaving it
  standing.

`check_target`'s out-distance is `attack_dist` from the searcher's own
position (`00649e1c`, the first thing the function does). The metric is
therefore the same one §36.1's table is in.

**The field survives untouched between searches, and that is the point.**
run112's `0/9` carries `near_o 6, near_who 1` from block 638 to 652
without a single change: it recorded `1/6` on its birth-frame search at
621 (`chapter_two_s_hoplite_captain_refused_a_cell_three_searches_
reached` asserts that row) and never searched again in the window. So
the incumbent the arm below reads on 644 was chosen twenty-three frames
earlier.

**Item 523's residue on 847 closed on item 1089** (§79): the bowmen `0/7`
and `0/8` are followers, and `find_new_target`'s squad head hands them
their captain's target without a search, so this crate no longer writes
their pair to `-1`. The whole of run112 agrees, 17,219 of 17,219.

[`sim::Unit::near`] is the field, written at `find_nearby_target`'s own
site. **SEAM**: the original's is an `ObjectData` member and a building
carries one too; this crate holds it on a unit only, which nothing
either side models reads.

### 37.2 The arm, `005f803f`-`005f8216`

Inside `do_move`'s `ptype->max_range != 0` gate — a **block**, whose
brace closes past this whole arm, so the retarget is a ranged
attacker's exactly as the kill above it is (§38.2; this landing lost the
nesting in translation and item 481 restored it) —
and inside its "the target is not a wallbuild" arm (`piVar2->vt[0x1c]`
is `SubObjectData::is_wallbuild`, offset 28 by the field list),
**after** the in-range kill has failed:

```
if (attack->mandatory == 0 && is_captain(this)) {         // o_up < 0
    melee = -1;
    if ((o + frame) % 16 == 0)
        melee = find_melee_target(this, -1, &t_who, 0, 0, mode);
    if (near_o >= 0 && near_who >= 0) {
        c_o = near_o;  c_who = near_who;
        if (melee >= 0 && is_in_range(this, melee, t_who, …, 0))
            { c_o = melee;  c_who = t_who; }
        if (cand->flags & 1
            && is_in_range(this, c_o, c_who, …, 0)
            && poor_target(this, c_o, c_who) == 0
            && cand->is_unit()
            && military(cand)) {
            attack->in_range = 1;                          // 005f820a
            change_target(this, attack->ox, attack->whom, c_o, c_who);
            return 0;
        }
    }
}
```

**The two range tests are the same function with a different sixth
argument, and that is the whole of why both can decide on one frame.**
The kill above passes `attack->mandatory == 0`, so it asks the reach
less `0x90` (§35.3); this asks the plain reach. A ranged captain can
therefore be out of range for the purpose of ending its chase and in
range for the purpose of switching targets, on the same frame, against
the same distance.

`military(cand)` is the arm's own disqualifier and it is a two-sided
thing:

```
uVar17 = 0;
if (cand->is_unit()) {
    if ((this->unit_masks2 & 4) == 0)  uVar17 = ~(cand_role >> 16) & 1;
    else if ((cand_role & 0x10000) && !(cand_unit_flags & 0x2000)) uVar17 = 1;
}
```

`cand_role` is `UnitTypeData +0x2c8 role`, and `*(ushort *)(t + 0x2ca)
& 1` is `role & 0x10000` read as the upper halfword — the **military**
bit this crate already carries as `Profile::combat_role`. So for an
attacker without `unit_masks2 & 4` the retarget wants a military
candidate, and for one with it the sense inverts. run112's slingers
print `unit_masks2 0`, so the first arm is the one measured.

**Why 644 and not 638.** `near_o` has been `1/6` the whole time and the
arm runs every frame; what changes is the plain-reach test as `0/9`
walks from `(791, 8088)` to `(912, 8096)`. Nothing in the arm is phased
— the sixteen-frame phase gates only the `find_melee_target` probe, and
`(9 + 644) % 16 = 13`, so on this frame the candidate is `near_o` alone.

**SEAM**: the `find_melee_target` probe is not implemented. Its own
sixth argument (`1` when the target is not a wallbuild and the type's
`vt+0x10c` answers zero, `2` otherwise) has no counterpart in this
crate's two-argument helper, and the call re-enters
`find_nearby_target` — which bumps `targeted` and rewrites `near`, so
wiring it is a behavioural change of its own rather than an addition.
No frame of any capture on disk reaches it.

### 37.3 `poor_target` refuses a futile chase, and the conjunct that decides is the speed

`Object::poor_target@0064a270`, three conjuncts and a floor:

1. the candidate's head order is in the move family (`is_move@0046f050`
   — `1, 2, 3, 4, 0x12, 0x13, 0x15`): it is walking;
2. `get_speed(me, my pos, 0) < get_speed(cand, its pos, 0)` — it is
   **faster than me**;
3. it is facing away from me: `cand.angle − find_angle(to cand) +
   0x80000000` inside the window **and** `flanking` answering `2` — the
   one caller in the executable that reads the 1/2 split §36.3 says
   nothing reads;

then `attack_dist > 0xc0`, or `> max_range * 0xc0` for the lineage
`role & 0x400` names.

For run112 the answer is `0` and **the second conjunct is what says
so**: `0/9` prints `myspeed 28` against `1/6`'s `25`. A slinger does
not think chasing a hoplite is futile, so the floor — one tile, against
the 1296 between their snapped quarter-tile centres — is never reached.
That ordering is why the function is worth having exactly rather than
stubbed: stubbed `true` it kills §37.2 outright, stubbed `false` it
accepts every chase the original refuses.

**SEAM**, both above conjunct 1: ~~a candidate whose first `Guy` carries
`guy_flags & 0x40` takes a different arm entirely
(`has_objmask(0x80000000)` and then the reach floor), and this crate
has no such guy flag~~ **built by item 650, §61.3**: the flag marks a
plane, and the arm refuses one to a searcher without `ANTI_AIR` or out
of its reach; `role & 0x400` is unloaded, this crate's
`combat::role` word being its own synthesis, so the floor is always the
tile.

### 37.4 `change_target` is one decision written down the squad in place

`Unit::change_target@005e36c0` is a walk of `o_down`, and it is the
whole of why three slingers retarget on one block:

```
loop {
    order = update_action(this);
    if (order && (t = order->update_target_order())
        && t->o == old_o && t->who == old_who) {
        t->o = new_o;  t->who = new_who;
        t->uid = objects[new_who][new_o]->uid;
        while (head->is_targeted() == 0) kill_current_order(this, 0);
    }
    if (this->o_down < 0) break;
    next = objects[this->who][this->o_down];
    if ((next->flags & 1) == 0) return;
    this = next;
}
```

`vt+0x3c` is `UnitOrder::update_target_order` (offset 60) and `vt+0x20`
is `UnitOrder::is_targeted` (offset 32), both by the field list. So the
target is rewritten **in the order that holds it**, conditional on that
order still holding the old one, and then the head is popped until it is
the targeted order — which drops the chase the retarget has just made
pointless.

**The in-place rewrite is measurable, and it is what rules out every
order-creating path.** On block 645 `0/10` and `0/11` take `1/6` while
keeping `in_range 1`, `ever_in_range 1` and `new_ord 0` — the flags they
were already carrying against `1/8` — and their positions, `orders_x/y`
and every other field are unchanged. A fresh `add_attack_order` through
the captain mirror (§21) would have reset all three flags. `0/9` itself
shows the other half: `in_range` goes `0 → 1` because `005f820a` writes
it, and `ever_in_range` stays `0` because only `Unit::fight+0xba9`
writes that and `fight` runs a frame later — which is also why `0/9`'s
`visible` byte arrives on 646 and not 645.

**SEAM**: the head-popping test is `is_targeted`, true for every class
deriving `TargetOrder`; this crate asks for an `ATTACK` body instead.
They agree wherever the action is an attack, which is every frame of
every capture that reaches this.

### 37.5 `Unit::think`'s own `near_o` arm, which is not implemented

`near_o` has a second reader and it is a bigger one.
`Unit::think@005f6e40:104`-`134`, above the step-3 cadence gate: on the
frames where `(o + frame) % 32 != 0` — thirty-one in thirty-two — a
captain whose `idle` is not 1, whose `near_o` is valid, and whose order
count is under 5 takes `add_attack_order(near_o, QUEUE_NEW)` if it is
`is_in_range` of it, **instead of running the search at all**. So the
field is a cache that lets the engine skip `find_nearby_target` almost
always, and this crate runs the search on the phase frames and nothing
on the others.

It is not this word. `Unit::think` is reached from `Unit::do_idle` and
from nowhere else in the executable — one caller, grepped — so a unit
with a `MOVE` at the head of its list never enters it, and `0/9` on 644
has one. The dump says the same thing independently: `think`'s arm
creates an order, and a created order prints `in_range 0`.

Parked as a successor. It needs `idle`, the order count and the
`% 32` phase, all of which this crate has, and it will change what every
idle captain in every capture does.

### 37.6 What moved

Word **645 → 680**, sequence 680, values 681. 680's own extra draw is
`Guy::set_anim+0x97a < Unit::move_step+0x823` against the original's
`Animal::think_bird+0x82` — the chase destination again, which is
§31.6's residue and not this mechanic's.

The widening window is `[606, 684)` and its map is six rows, `angle
1/6` at 672 and `order`/`pos` on `1/6`, `1/7` and `1/8` at **671**. All
five of §36.6's rows are gone and so is every record in between: sixty-
five frames of run112 from its own first block, every field of every
dumped record, both directions, at nought.

~~What stands at 671 is **who=1's side of a tie this crate has broken the
other way twice before**. All three hoplites hold `0/11` where this
crate holds `0/10`; the two slingers are near-equidistant identical
figures, and `pos 1/6`, `pos 1/7` and the whole of `1/7`'s move order
follow from a hoplite walking to a different one. §33 named that shape
and closed it on who=0's two squads; this is its mirror.~~ — **falsified
by item 481, §38.** It was not a tie and it was not who=1 choosing
anything: the dump has all three hoplites on `ox 11 whom 0 uid 18` for
**every block of the capture**, with `new_ord 1` throughout, so the
original never ranks those two slingers against each other at 671 and
never rewrites the target. The six rows are this landing's own — §37.2's
`max_range != 0` gate is a *block* in the executable and was written
here as a conjunct of the in-range kill alone, so a **melee** captain
reached the retarget. §38.

`visible`'s exact count is **9 of 9**, from 8. The one that came over
is `0/9`'s own, 648 → 646, and because `visible`'s arrival frame is the
frame of a unit's first strike, nine of nine says chapter two's whole
engagement — both squads, both directions — now opens fire on the
original's own frames.

`rondata`'s endpoints do not move and neither ladder rung moves.

### 37.7 Coverage

**Diff-backed**, by `chapter_two_s_word_frame_is_widened_whole` over
`[606, 684)` and `chapter_two_s_visible_byte_is_the_dump_s_on_every_
unit_frame`:

- that a ranged captain retargets to its cached incumbent on the frame
  the plain reach admits it — `0/9` on 645, `order 0/9`'s `Length`,
  `Kind` and `PathLength` and `pos 0/9` all the dump's;
- that the switch propagates down `o_down` in place on the same block —
  `order 0/10` and `order 0/11`'s `Target`, with their flags unchanged;
- that the strike follows a frame later — `visible 0/9` at 646;
- that `near_o`'s value on 621 is `1/6`, by the older cell test;
- **`near_o`/`near_who`'s value on every unit of every block** of
  `[606, 684)` —
  `chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame`, written
  here: 4668 unit-frames, 161 of them carrying a live pair, **4668
  agreeing**. The dump has printed the pair at every detail level since
  2026-09-19 and nothing had ever compared it, because until this
  landing there was nothing to compare it *to* — `crate::diff::golden`'s
  own reader said so in a note this closes. Made to fail on purpose:
  deleting the write parts exactly the three searching captains, `0/9`
  from 621 and `0/6`/`1/6` from 635.

**Reading-only**, and each names the capture that would falsify it:

- `near_o`'s own **rule** — the `0xf00` clear, the write above the
  `max_dist` gate, the either-side `is_unit` conjunct. run112 runs three
  searches inside the window, all before 636, and neither removing the
  clear nor taking the last qualifying candidate instead of the nearest
  turns the row below red, so the rule stays a reading. A capture with a
  crowded, far search would reach it.
- `poor_target`'s conjuncts 1 and 3 and its floor. Conjunct 2 decides
  every frame on disk, so 1 and 3 are never the answer here; a capture
  with a faster target walking away would reach them.
- `is_targeted` as the head-popping test, and `unit_masks2 & 4`'s
  inverted arm. No capture reaches either.
- the `find_melee_target` probe and its mode argument: unimplemented,
  not merely unverified.

## 38. The ranged gate is a block, not a conjunct (item 481, 2026-09-22)

§37.6 booked 671 as "who=1's side of a tie this crate has broken the
other way twice before" — §33's shape mirrored, a target-selection tie
among two near-equidistant identical slingers. **The dump falsifies that
before any code is read**, and what the six rows actually are is §37's
own landing firing on units it was never meant to reach: `do_move`'s
`ptype->max_range != 0` is a **block** in the executable whose brace
closes past the captain retarget, and §37.2 was implemented with it as a
conjunct of the in-range kill above. A melee captain therefore reached
the retarget, and run112's hoplite captain switched its whole squad off
a target the original never changes.

The word moves **680 → 683**, and every record run112 carries over
`[606, 684)` — both directions, every field, every unit, seventy-eight
frames from the capture's own first block — goes to **nought**.
`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` holds at
**9 of 9**.

### 38.1 The dump had already answered it, in a field it prints every block

`docs/COMBAT.md` §37.6 read the six rows as a choice. A choice leaves
marks, and none of them are there:

| block | `1/6` | `1/7` | `1/8` |
| --- | --- | --- | --- |
| 655 | `ox 11 whom 0 uid 18` | same | same |
| 670 | same | same | same |
| **671** | same | same | same |
| 683 | same | same | *(dies)* |

All three hoplites hold **`0/11`** on every block of the capture, on the
same `uid 18`, and `new_ord` is **1** on `1/6` and `1/7` from birth to
671 — the flag a freshly created order carries (§37.4). So the original
neither ranks the two slingers against each other at 671 nor rewrites a
target in place: whatever happens on that frame, who=1 does not choose.
That leaves one side that can have moved, and the row says which — this
crate held `0/10` where the dump holds `0/11`, on **all three** figures
of one squad at once, which is `Unit::change_target`'s own signature
(§37.4) and nothing else's.

`1/6`'s `near_o` is the rest of it: **`10`, from block 621 to the end of
the window**, unchanged, because §37.1's field is the *nearest*
`check_target`-passing candidate and `0/10` is nearer than the `0/11` the
squad is walking to. So the incumbent this crate acted on was right, the
arm it acted through was right, and the only thing wrong was that a
hoplite may not run it.

**The cost of the framing was twenty minutes, and the cheap check was a
`grep` of a field already on disk.** `docs/DECISIONS.md` 42's rule held
again: the frame was right and the named mechanism was wrong.

### 38.2 `do_move@005f7b30:207` — where the brace closes

```
if ((piVar2[2] & 1) != 0 && (short)piVar2[0xc] == target->uid) {
  if (*(int *)(*(int *)&this->field_0x18 + 0x1fc) != 0) {         // 207  max_range
    if ((*(code **)(*piVar2 + 0x1c))() == 0) {                    //      not a wallbuild
      … the flank triple, is_in_range(margin), find_collision …   //      the in-range kill
      if (attack->mandatory == 0) {
        if (is_captain(this)) { … near_o … change_target … }      // 005f803f
      }
    }
    else { … the building-target kill … }
  }
  …
}
```

`ObjectTypeData +0x1fc` is `max_range` by the type record, and the brace
opened on line 207 closes **after** the captain arm — the same nesting
item 405 established for the unit-target kill and the building-target
kill (§35.3, `docs/ORDERS.md` §4.4). Everything the ATTACK action does
under a MOVE is a ranged attacker's. A melee type walks the leg it was
given and `do_attack` takes over when the move ends; it never abandons a
chase because something came into reach, and it never retargets.

**A hoplite's `max_range` is 0, and the measurement is the proof rather
than the table.** Nesting the retarget inside the gate removes exactly
the six rows at 671/672 and nothing else — so the three units the rows
name are on the zero side of the test, both directions, with no reading
of `ObjectTypeData` required. (§36.3 said the same thing from the other
end: `check_target_path`'s review has *no* `max_range` gate, "which is
why `1/7` and `1/8` are on the list".)

This is the second time the arithmetic was right and a **predicate** was
wrong (`docs/audit/README.md`'s recurring lesson), and the first time
the predicate was lost in *translation* rather than in reading: §37.2
states the gate correctly in prose and even names it — "Inside
`do_move`'s `ptype->max_range != 0` gate" — while the Rust wrote it as
`self.profile(me).max_range != 0 && self.is_in_range_at_margin(…)`, a
conjunct of one arm. A gate stated as prose and implemented as a
conjunct is the shape to look for; the fix is one `if` block.

### 38.3 What moved

Word **680 → 683**, sequence 683, values 684. 683's own extra draw is on
the original's side, `Farms::inc_time+0x1ae` against an unnamed
`60fb06` — outside the engagement entirely.

The value diff on the frame the word moved, the dump's own coordinates —
the six rows that are **gone**:

```
671 pos   1/6   ours (1623, 8037)  theirs (1608, 8040)
671 pos   1/7   ours (2088, 8280)  theirs (2091, 8312)
671 order 1/6   Target ours (0,10) theirs (0,11)
671 order 1/7   Target ours (0,10) theirs (0,11) · Move x 1512/1128 · y 8328/8424 · PathLength 0/5
671 order 1/8   Target ours (0,10) theirs (0,11)
672 angle 1/6   Heading ours -1320157184 theirs -1605566464
```

The widening window is `[606, 687)` and its map is **three** rows, all of
them above the old window's ceiling at 684:

```
684 extra 1/8            the original's hoplite died on 683; ours has not
685 order 1/4            a citizen's move, ours (1512,8328) theirs (1128,8424)
686 pos   1/4            ours (41016, 17016) theirs (41033, 17034)
```

**All three stood with this landing reverted**, measured on the same
widened window, so the ceiling was hiding them and the fix did not open
them. That is the floor's lesson from item 470 arriving at the other end
of the window: a ceiling sized to the word reports agreement it has not
measured, for exactly as long as the word stands still.

`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` holds at
**9 of 9** — §37's own `0/9` retarget at 645 is a slinger's and is
untouched, which is the check that says this change narrowed the arm to
the types the original runs it on rather than switching it off.
`rondata`'s endpoints do not move and neither ladder rung moves.

### 38.4 What stands at 684, and what this widening cannot see

`extra 1/8` is a **death**. The dump's `DEATH_OBJS` on block 684 carries
`who 1 o 8 first_frame 683 cur_anim 17`, and `1/8`'s own accumulator
reaches it: `damage` 0 → **8** (656) → **17** (657) → **25** (660) →
**34** (682), against a figure's share of `myhits 120`, which
`docs/ATTRITION.md`'s figure table puts at **40**. The hit on 683 takes
it over and the record vanishes before the accumulator is printed again.

~~**`compare` carries no hit-point row at all.** No `myhits`, no
`damage`, no `hits_left` — so this walk can see that `1/8` is *gone*
and not that it was wounded on different frames, and the successor item
owes that row before it names a mechanism.~~ **Closed by item 484**,
§40: the record is three fields, `damage_frac` was never parsed on a
unit at all, and with them in the map over this window goes from three
first-partings to **eight** — the earliest at **656**, twenty-seven
blocks under the word. `1/8` is one hit behind the dump from its first
wound on, and 684's `extra` is the end of that lag rather than a fact
of its own.

`order 1/4` / `pos 1/4` are a **citizen** — guy type 50, `myhits 40`, a
GATHER on `uid 3` forty thousand units from the engagement — and they
are the same family as `order 0/5` at 639 (§32.5) and `0/5`'s flight
residue (§34.4): an economy divergence named here so a regression in it
cannot hide behind the engagement, and nobody's item yet.

### 38.5 Coverage

**Diff-backed**, by `chapter_two_s_word_frame_is_widened_whole` over
`[606, 687)` and `chapter_two_s_visible_byte_is_the_dump_s_on_every_
unit_frame`:

- that a **melee** captain does not retarget under a move — the six rows
  at 671/672 are gone, and `1/6`'s `near_o 10` is still compared right
  through the window by `chapter_two_s_near_o_is_the_dump_s_on_every_
  unit_frame`, so the incumbent is still read and still not acted on;
- that a **ranged** captain still does — `visible 0/9` at 646 and
  §37.7's rows all hold, which is the same change measured from the
  other side;
- that the three hoplites keep `0/11` for the whole window, on the
  dump's own `uid`;
- `near_o`/`near_who` on every unit of every block of the **widened**
  window — `chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame`
  re-pinned upward with the ceiling, 4668/161 → **4848** unit-frames,
  **170** of them live, **4848 agreeing** and no unit parting. A tally
  that grows with the window is the only direction this row may move
  (§37.7).

**Reading-only**, and each names the capture that would falsify it:

- that the gate's block also encloses the **building**-target arm and the
  flank triple. run112's chase is against a unit on every frame, so no
  row separates them; a capture of a ranged unit ordered onto a building
  it is walking to would reach it.
- everything §37.7 already lists as reading-only is unchanged by this
  landing: `near_o`'s `0xf00` clear, `poor_target`'s conjuncts 1 and 3,
  `is_targeted` as the head-popping test, `unit_masks2 & 4`'s inverted
  arm, and the `find_melee_target` probe.
- **What this does not establish**: whether a melee captain has *another*
  retarget path of its own. `Unit::think`'s `near_o` arm (§37.5, parked
  482) has no `max_range` gate on it, and it is thirty-one frames in
  thirty-two — so "a hoplite never changes target under a move" is what
  is measured here, not "a hoplite never changes target".

## 39. The word was a label: naming `Ammo::do_damage`'s puncture pair (item 483, 2026-09-22)

Great Lakes' long word had **separated into two numbers** and item 478's
§19.3 (`docs/ORDERS.md`) named why: the *count* word had reached 10244
while the *sequence* word stood at 10237, where both sides spend seven
draws and part on the sixth. What parted was not a simulation
disagreement at all. The original's trace names that draw `678cb9`;
`rondata::trace::SITES` did not carry the address, so it printed as a raw
hex string against this crate's unattributed `projectiles` phase mark —
a comparison that could neither pass nor fail. **Until the site was named
the sequence could not pass 10237 however the simulation behaved.**

It is named now, the two words have met again at **10244**, and no line
of simulation changed to do it.

### 39.1 The site, from the executable's own bytes

`Ammo::do_damage@00678060` has **exactly two** `Random::get` calls, and
they are a pair. The arm they sit in is the miss: after `Ammo::hit_target`
and `Ammo::check_hit` have both failed to name an object, the shot lands
on open ground and the decal's position is jittered.

```
678ca7: mov  0xc06184, %ecx          ; GameAccess::game_random
678cad: push $0xffff
678cb2: push $0x0
678cb4: call 0xa39d70                ; Random::get  -> returns to +0xc59
678cb9: cltd
678cba: mov  $0x29, %esi
678cbf: idiv %esi
678cc1: mov  0x18(%ebx), %eax        ; the ammo's x
678cca: sub  $0x14, %edx
678cd2: add  %edx, %eax              ; x + n % 41 - 20
678cd9: call 0xa39d70                ; Random::get  -> returns to +0xc7e
678cde: cltd
678cdf: idiv %esi
678ce1: mov  0x1c(%ebx), %eax        ; the ammo's y
678cea: sub  $0x14, %edx
678ced: add  %edx, %eax              ; y + n % 41 - 20
678cfa: call 0x6b53a0                ; WorldData::restrict
678d0a: call 0x9072e0                ; GraphicPieces::verify_ammo_flags(.., 1)
                                     ;   then AmmoOut::puncture_ground
```

So the two sites are **`0x0067_8cb9`** (`+0xc59`, x) and
**`0x0067_8cde`** (`+0xc7e`, y), each the return address of a call to
`game_random` — the sync generator, which is also why the trace printed
them at all. Both are `% 41 - 20`, a ±20-world-unit jitter, and the
result is consumed by `puncture_ground`: **cosmetic, and it still moves
the stream.**

This crate has spent both draws since the ammo list existed —
`Sim::land`'s `let Some(t) = target else { … }` arm, two bare
`rng.roll()`s under a comment that said what they were — and named
neither. `sim::fight::SITE_PUNCTURE_X` and `SITE_PUNCTURE_Y` are the
labels now, marked per draw for `SITE_AMMO_SCATTER_X`'s reason: two
addresses are two entries in the compared sequence and one label folds
them into one.

### 39.2 What it moved, and what it did not

| | before | after |
|---|---|---|
| Great Lakes long word, **sequence** | 10237 | **10244** |
| Great Lakes long word, **count** | 10244 | 10244 (unchanged) |
| run53 frames draw for draw, of 24,000 | 10,506 | **10,509** |
| run53 frames on the original's count | 11,922 | 11,922 (unchanged) |

The three frames are the whole of the +3, and they are the whole of the
change. **The original takes this draw on exactly three frames of
run53's 24,000 — 10237, 10242 and 10249 — and this crate on the same
three and no others**, which is what makes the pair a pin and not just a
name. All three now agree draw for draw, whole:

```
frame 10237: ours 7 theirs 7   (…, calc_market ×3, two inc_time wraps, puncture x, y)
frame 10242: ours 8 theirs 8   (…, six set_anim rolls, puncture x, y)
frame 10249: ours 6 theirs 6   (…, four set_anim rolls, puncture x, y)
```

**No draw attribution changed on any other frame**, and that is measured
rather than argued: the whole 24,000-frame label dump was taken either
side of the change (`RON_DEBUG_SITES=0-24000`) and the diff is 52 lines,
every one of them on 10237, 10242, 10249 or the summary line. Before the
change the bare label `projectiles` occurred on those three frames and
nowhere else in the game; after it, nowhere at all. On the original's
side, `Ammo::do_damage` draws on those three frames in every Great Lakes
trace on disk (run53, run80, run100, run18a), on sixteen frames of run16
and one of run24, and in none of the other ninety-odd traces — and
neither run16 nor run24 is read for labels by any test.

The word is again **one** number, and what stands at 10244 *is* a
simulation disagreement: four draws against three, parting at index 1,
ours `Guy::set_anim+0x97a < Unit::move_step+0x823` — the blocked stand —
against the original's `Guy::inc_time+0x271` wrap.

### 39.2.1 The value diff beside the move

A draw stream can agree on a wrong destination for a long time, so the
move is booked with the dump's own coordinates on the frame it moved.
The window `run100_s_word_frame_is_the_original_s` and
`…_block_is_every_record_the_dump_carries` walk runs to the headline, so
raising the word by seven opened **seven blocks nobody had ever
compared** — and this is what is in them. None of it is new behaviour;
483 changed no simulation line. `ORDER_RESIDUE_RUN97`'s lesson, in the
direction that adds rows rather than counts.

**The word's own block (10245, the state after frame 10244) is eleven
rows and they are all one unit:**

```
1/27 collide:            ours 2    theirs 1
1/27 collide_o:          ours 29   theirs -1
1/27 collide_who:        ours 1    theirs -1
1/27 g.stopped[0]:       ours 1    theirs 0
1/27 g.cur_anim[0]:      ours 0    theirs 7
1/27 g.cur_time[0]:      ours 1    theirs 3
1/27 g.end_time[0]:      ours 31   theirs 13
1/27 g.last_time[0]:     ours 0    theirs 2
1/27 order:coll:         ours Some((4701, 29818))  theirs (4637, 29827)
1/27 order:move.dest:    ours 0    theirs 1
1/27 path:length:        ours 49   theirs 43
```

**The dump and the draw stream name the same unit and the same
mechanism.** `1/27` is colliding in this crate and not in the original —
`collide_o 29` is the unit it has run into — so it is stopped on an
animation that has just begun where the original's is three frames into
a walk. That *is* the extra draw: `Unit::move_step+0x823` is the blocked
stand. Nothing had to be hypothesised to join them; the two instruments
met on their own.

~~**And the cause is upstream, two frames and one unit over.** The thing
`1/27` collides with is `1/29`, which on block 10241 drops its move
order where the original keeps walking … Twenty-eight world units short
in `x` and standing still, two frames before `1/27` walks into the space
it should have left.~~ **Struck by item 487 — the reading was
backwards**, and `docs/ORDERS.md` §20.1 carries the correction. The rows
themselves stand; `orders_front_first` is the log's list **reversed**, so
`f10241 1/29 order:kind ours 10 theirs 1` is this crate still holding an
ATTACK where the original is already on the move underneath it — not the
original walking while this crate drops. And `1/29` is not twenty-eight
units short of anything: it sits at `(4680, 29928)` with an empty path on
every block of 10237-10248 and never moves at all. 487's widening killed
the reading before it changed a line, and the mechanism it landed instead
was `ungroup_move_order`'s prepend and `do_move`'s `0x480` dead-target
re-path.

Both rows are pinned rather than waved at: `RAIDER_STOPS` and
`RAIDER_COLLIDES` name them in the window test the way `RAIDER_X` and
`RAID` name theirs, so the day either closes, that test fails. The
standing position residue over the window goes four keys to six —
`(1,24), (1,25), (1,26), (1,28)` plus `(1,27)` and `(1,29)` — and the
block the word left (10238) is pinned empty, which is what it was when
the word stood on it.

### 39.3 The guard this item is really for, and what it found

A label change moves the score without changing behaviour, which is
exactly the shape a **wrong** label would also have. `SITES` is the one
place in `rondata` where a number carries a name, and a wrong pairing is
the one error the differential check cannot report: it makes a frame read
as agreeing, or as parting somewhere else, and nothing else in the suite
looks at the pairing at all.

So the row is now checked rather than trusted.
`rondata::trace::tests::every_site_s_address_is_the_function_its_label_names`
resolves **every** row's address to its containing function in the
decompile export's `INDEX.tsv` and requires the label's own
`Name+0xoff` to be that function and that offset; a chain-qualified row
is checked at both ends, its `via` against the label's last link. The
whole table, in a millisecond, every commit. It skips loudly on a machine
without the export, the way `sim`'s
`every_cited_address_names_its_function` does.

It was made to fail four ways before it landed — one digit off each of
the two new sites, `SITE_PUNCTURE_X`'s address moved onto
`SITE_FIRST_WOUND`'s row, and `SITE_TURN_NEAR`'s `via` replaced by its
neighbour's — **and it failed for real on its first run**, on a row
nobody had reason to doubt:

> `Object::take_damage+0x18b 0x0065218b is Object::take_damage+0x16b and
> the label says +0x18b`

`SITE_FIRST_WOUND_FLOCK` is the siege-hit flock's `% 2 + 3`
(`docs/COMBAT.md` §7.2 step 3), and its label is right: the
`Random::get` at `6521a6` returns to **`0x0065_21ab`**. The table held
`0x0065_218b` — two digits transposed, onto an address that is not even
an instruction boundary (`65218a` is a three-byte `mov 0x18(%eax),%ecx`),
so the row could never have matched a draw and the flock's would have
printed as a bare `6521ab`. Corrected here. **No comparison moved**: a
scan of all 102 traces on disk finds site `0x6521ab` in none of
them, so the flock has never been drawn in a captured game. The row was
dead and it is now right, which is the cheapest kind of correction and
the kind only a guard finds.

### 39.4 Coverage

**Diff-backed**:
`run53_s_24000_frames_put_the_ceiling_where_run33_did` now pins the three
puncture frames — that the original takes the pair on exactly
`[10237, 10242, 10249]` of 24,000 frames, that this crate takes it on the
same three, that each of those frames ends on the pair in that order, and
that 10237 and 10242 (both below the word) agree draw for draw. Made to
fail on purpose by dropping the `SITE_PUNCTURE_X` mark, which parts
10237 at the pair's first entry.

**Dump-backed**: §39.2.1's rows, from
`run100_s_word_block_is_every_record_the_dump_carries` with
`RON_DEBUG_ROWS=10236-10246`. The word's own block is pinned as the
eleven rows above and the block it left as empty; the window test names
the two newly-exposed position rows by frame. All three re-pins **failed
for real** before they were written — they are what the gate reported
when the word moved, not assertions written to a tree that already
passed. The rows are diff-backed; the **reading** §39.2.1 put on
`1/29`'s pair is struck, and `docs/ORDERS.md` §20.1 has the direction
right (item 487).

**Listing-backed**: §39.1's disassembly, read from `riseofnations.exe`
with `llvm-objdump` at `0x678c60`–`0x678d10`, and the same for
`0x652170`–`0x6521c0` in §39.3. The containing functions are the
decompile export's (`00678060 Ammo::do_damage`, `00652020
Object::take_damage`), and `every_site_s_address_is_the_function_its_
label_names` re-derives the pairing from that index on every commit
rather than trusting this paragraph.

**What this does not establish, and the one predicate it found.** The
draw arm's *gate* is not quite this crate's, and the difference is
one clause that no capture on disk reaches. The original takes the two
draws when

```
target_slot < 0 || target_owner < 0 || (target == shooter)
```

— so a shot that lands on **its own shooter** punctures the ground and
spends both draws. `Sim::land` returns without drawing there
(`if t == p.shooter { return; }`). The clause is very likely dead in
practice: `Ammo::check_hit`'s `find_unit` is filtered by the shooter's
own owner, and this crate's `check_hit` excludes the shooter outright,
so the pair can only carry the shooter if a unit fired at itself. It is
left alone deliberately — this item changed no behaviour, and mixing a
behavioural change into a label-only measurement would have destroyed
the one thing the measurement proves. **It is a parked row, with its
falsifier named**: a capture in which a unit's shot lands on itself, or a
synthesized `callfn` entry into `do_damage` with `0x48/0x4c` equal to
`0x3c/0x40`, would turn the two draws into a visible count delta. No
capture on disk has one.

Nor does this section establish anything about the *arithmetic* the two
draws feed: this crate discards both rolls, because `puncture_ground`
leaves a decal and nothing the simulation reads. Only the draws' count,
order and sites are modelled, and only those are pinned.


## 40. The hit-point record, and five rows nobody was looking at (item 484, 2026-09-22)

`crate::diff::compare` carried **no hit-point row at all** — no
`myhits`, no `damage`, no `hits_left` — where the dump prints the
record inside the `OBJECT` block at *every* detail level and
`run100_s_word_block_is_every_record_the_dump_carries` has read two
thirds of it since §34.4. So chapter two's widening could see that the
original's hoplite `1/8` was **gone** on block 684 and not that it had
been wounded on 656, 657, 660 and 682 to get there (§38.4).

The row is in. Chapter two's map goes from **three first-partings to
eight**, and the earliest is **656 — twenty-seven blocks under the
word.** No word moves and no already-scoring family changes.

### 40.1 The record is three fields, and one of them was never parsed

```
 BEGIN UNITDATA
  BEGIN OBJECT
   BEGIN SUBOBJECT
    flags 65
    o 8
    who 1
    x_internal …
   damage 34            <- OBJECT's indent
   uid 18
   myhits 120
   hold_frames 0
   infiltrated 0
   damage_frac 8
   mylos 4
  collide_frame -1      <- UNITDATA's own
```

`myhits` and `damage` have been parsed off `OBJECT` since item 394.
**`damage_frac` had not**, on a unit — it was read for a building from
the same item and never for a figure, which is the shape item 478 found
one record over: a field the dump prints on every block of every
capture, not merely uncompared but unparsed, and therefore invisible to
every widening ever run.

`the_unit_s_hit_points_are_the_object_block_s` pins the indent with a
decoy on **both** sides — `7/700/70` at `UNITDATA`'s indent and
`9/900/90` inside `SUBOBJECT` — so a reader that walks out one level or
in one level reddens on the *value* rather than on `None`. Made to fail
on purpose both ways before landing.

### 40.2 What each field means, and why `hits_left` is not a fourth row

| dump | what it is | this crate |
| --- | --- | --- |
| `myhits` | the **squad's** whole hit points (§7.3) | `Unit::max_health` |
| `damage` | whole points this **figure** has taken | `max_health − health` |
| `damage_frac` | the sixteenths under them (§7.2 step 4) | `Unit::damage_frac` |

`update_hits` writes one number — the squad's — onto every figure, and
`take_damage` divides it on the way in; each figure of a squad is its
own `UnitData` in the owner's list (`docs/ATTRITION.md`, "the figure is
the thing that bleeds"). This crate keeps the **complement**: `health`
is that same squad-sized number with this figure's damage already
subtracted. So the three line up field for field, and `hits_left` —
what `run100_s_word_block_is_every_record_the_dump_carries` calls its
row — is `myhits − damage` on both sides. It would part exactly when
one of the two above does and print every disagreement twice, so
`compare` does not carry it. The widening keeps its own because it has
no `damage` row.

The row is **ungated on the position**, for `ObjectData::visible`'s
reason (§ `docs/VISION.md` §7): what a unit has taken is not a
consequence of where it is standing. It is ungated on `on_map` too,
unlike `visible` and `mylos` — a garrisoned unit is healed by its
building on both sides, so the record stays comparable where those two
do not.

### 40.3 What it reads, on both maps

| capture | map | field-frames compared | wrong | wounded unit-frames in the dump |
| --- | --- | --- | --- | --- |
| run57 (4,000 frames) | East Indies | **205,302** | 0 | **0** |
| run79 (341 blocks) | Great Lakes | **40,377** | 0 | **0** |
| run100 (909 blocks) | Great Lakes | +`damage_frac` on every block | 0 | — |
| ch2/run112 (whole chapter) | Great Lakes | **16,362** | 9 keys | **580** |

**Not one unit record of run57's 484,779, nor of run79's 27,194,
carries a wound**: neither capture contains a fight. What those two pin
is the maximum and the two accumulators at rest — this crate's
squad-sized `myhits` is the original's on every linked unit-frame of
4,000 frames of the headline map, and neither side invents damage
nobody dealt.

**The live values are all in the golden chapter-two engagement**, and
`chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame` is where
the anti-vacuity lives: 16,362 comparisons, **580** of them on a
unit-frame the dump wounds, over six units. A window in which every
value is zero on both sides passes on a crate that never writes the
field — which is exactly what `compare` was before this — so the
wounded count is asserted separately from the count read.

### 40.4 `myhits` is the original's everywhere; `damage` is not

`myhits` parts on **nothing**, on any capture, at any frame: all 21 of
chapter two's units, the six staged squads and the citizens beside
them, carry this crate's own squad-sized maximum. Corrupting it by one
reddens the assertion on all 21 at their birth blocks, which is what
says the row is reading and not agreeing by silence.

The nine keys that do part are all `damage`/`damage_frac`, and they are
one fact:

```
  damage      1/8  first at 656 — ours  0 theirs  8
  damage_frac 1/8  first at 656 — ours  0 theirs 10
  damage      1/6  first at 657 — ours  0 theirs  2
  damage      1/7  first at 686 — ours  0 theirs 19
  damage_frac 1/7  first at 686 — ours  0 theirs  5
  damage_frac 1/6  first at 712 — ours  0 theirs 10
  damage      0/11 first at 697 — ours 12 theirs  8
  damage      0/10 first at 737 — ours  0 theirs  4
  damage      0/9  first at 792 — ours  0 theirs  4
```

And inside the word's own window the trajectory is visible frame by
frame — this crate one hit behind from 656 on:

```
  656 damage 1/8 ours  0 theirs  8      656 damage_frac 1/8 ours  0 theirs 10
  657 damage 1/8 ours  8 theirs 17      657 damage_frac 1/8 ours 10 theirs  4
  660 damage 1/8 ours 17 theirs 25      660 damage_frac 1/8 ours  4 theirs 14
  682 damage 1/8 ours 25 theirs 34      682 damage_frac 1/8 ours 14 theirs  8
  684 extra 1/8  — the original's hoplite died on 683; ours has not
```

**Every value this crate holds is the dump's own previous one.** Not a
different arithmetic: the same ladder, one arrival late, from the very
first wound. `extra 1/8` at 684 is the end of that lag and not a
separate fact.

### 40.5 The instrument agreed because it was not looking, for the third time on one window

`chapter_two_s_word_frame_is_widened_whole` has now been wrong in all
three of the ways a widening can be:

- its **floor** sat just under the word and hid `order 0/10` at 630
  (item 470);
- its **ceiling** sat four frames over the word and hid `extra 1/8` at
  684 (item 481);
- and a **field** was missing from the comparator, which hid five rows
  from 656 — twenty-seven blocks *below* the word the window was
  built around (this item).

The first two are about the window; the third is not, and no widening
of the window could have found it. That is the argument for
`crate::ledger`'s count and for `CLAUDE.md`'s "when the original dumps
a record, diff the whole record": the window can be perfect and the
comparison still blind.

### 40.6 Coverage

**Diff-backed**:

- that the unit's hit-point record is written at `OBJECT`'s indent and
  nowhere else — `the_unit_s_hit_points_are_the_object_block_s`, with a
  decoy on either side;
- that this crate's `max_health` is the dump's `myhits` on **every**
  linked unit-frame of run57 (205,302 field-frames, East Indies),
  run79 (40,377, Great Lakes) and the whole of chapter two (16,362,
  580 of them live);
- that the three fields agree at rest on both maps: not one of the
  245,679 field-frames of run57 and run79 is wrong;
- that the nine keys that part in chapter two part on `damage` and
  `damage_frac` alone, from block 656.

**Reading-only**:

- that `damage_frac` is sixteenths on a *unit* as it is on a building.
  The pair moves as sixteenths in run112's own record (`1/8`: 8/10,
  17/4, 25/14, 34/8 — each step a whole point plus a carry) and the
  representation is the same one `docs/COMBAT.md` §7.2 step 4 reads for
  `BuildData`, but no capture separates a unit's carry rule from a
  building's. A capture of attrition on a squad would.

**What this does not establish** — deliberately, because it is item
485's:

- ~~**why** this crate's `1/8` runs one hit behind and does not die on
  683. `Object::take_damage` divides the squad's `myhits` by
  `uber_size` on the way in (§7.3) and `Sim::take_damage` uses
  `u.health` as the threshold outright, so a hoplite figure here
  absorbs a squad's worth; that is a *hypothesis* the frame does not
  yet carry, and the frame is 656.~~ **Answered by item 485, §41, and
  it is two facts rather than one.** The hypothesis is right about the
  *death* and wrong about the *frame*: the divide is 684's fact, and
  656's is that this crate's arrows left the shooter's own square where
  the original's leave the bow hand eighty-odd units ahead of it, which
  lengthened the flight by a frame (§41.2). Every value in the ladder
  above is gone; `damage 1/6` stands at 680 and `extra 1/8` is closed.
- whether `damage` is ever *reset* on a figure's death — the question
  §7.3's `total_damage` sum implies and no capture on disk answers,
  because no squad on disk loses a figure and survives.

## 41. The arrow left the wrong square (item 485, 2026-09-22)

§40.6 handed this item a frame and a hypothesis: `1/8`'s wound ladder
runs one arrival behind the dump's from block **656**, and
`Object::take_damage` divides the squad's `myhits` by `uber_size` on the
way in where `Sim::take_damage` did not. **The widening says the
hypothesis is right about the death and wrong about the frame.** They
are two defects, twenty-eight blocks apart, and the one at 656 is not in
`take_damage` at all: this crate's arrows left the shooter's own square
where the original's leave the bow hand eighty-odd units ahead of it,
and that lengthened four of the window's nine flights by a frame.

`chapter_two_s_word_frame_is_widened_whole` goes from **eight
first-partings to five**. `1/8`'s whole ladder — `damage`,
`damage_frac`, the new `damage_frame` — is gone, `extra 1/8`'s death is
on the original's own block, `pos 1/4` is gone, and `damage 1/6` moves
657 → 680. The word **holds at 683** and its cause has changed; §41.5
names what stands there now, with an address each.

### 41.1 The firing record, and what `recharging` settled in one run

`UnitData` prints four fields at its **own** indent that no comparison
read: `recharging`, the reload clock, and the overkill window
`damage_frame` / `damage_o` / `damage_who` beside it (§7.1 step 2). They
are written on every unit of every block of every capture.
`run100_s_word_block_is_every_record_the_dump_carries` has read
`recharging` since item 463 and `crate::diff::compare` read none of the
four, which is item 484's shape one record over: the mechanic's own
timestamps, uncompared.

They are rows now — `FiringDivergence`, keyed on the **field** — in
`compare` and in run100's walk, and between them they answered the item
before any code was read:

- **`recharging` parts nowhere.** Not once, on any of chapter two's 21
  units, over the 81 blocks of `[606, 687)`. The swing frames are the
  original's, so the cadence was never the fault and neither was the
  1-frame deferral §6.2 puts between `Unit::fight` and the attack
  animation.
- **`damage_frame` parts by exactly one, on both wounded units**: `1/8`
  ours 656 against theirs **655**, `1/6` ours 657 against theirs
  **656**. That is the dump dating its own hits, where §40.4 had to read
  the frame off the accumulator that moved.

`damage_o` and `damage_who` are compared only where **both** sides hold
a live window (`damage_frame != 0`). A never-hit unit is `damage_o −1`,
`damage_who 0` in the original and `0, 0` here, which is a difference of
encoding and not of state; inside a live window they are the same
quantity, and they agree — `1/8`'s `damage_o 6` is the bowmen's captain
on every block of its life, `1/6`'s `damage_o 9` the slingers'.
`damage_who` had no counterpart in `combat::State` at all until this
item; `do_damage` step 2 writes the pair together and this crate kept
only the captain.

**The direction here is a scalar's, not a list's.** `damage_frame` is a
frame number and `damage` an accumulator; "one arrival late" and "one
arrival early, read backwards" are the same claim only where the reading
comes from an ordered list, and neither of these is one. The dump's own
`total_time` in §41.2 is the third independent statement of the same
sign.

### 41.2 `AMMO=5` was in the capture's own detail line

`tools/gamelog/golden/chapter2.cmd`'s capture command asks for
`--detail end:UNITS=3,GUYS=2,AMMO=5,…`, so run112 has carried **373
`BEGIN AMMO` records** since the day it was taken and nothing had ever
read one. `AmmoData::log_data` prints the whole arrow:

```
 BEGIN AMMO
  cur_time 1          total_time 12
  who 0   o 6         whom 1   ox 8
  sx 975  sy 7806  sz 445
  ex 1925 ey 8221  ez 276
  angle 1215758336    accuracy 300
  flags 6  rolling 0  splash_area 0  num_guys 1
```

Every arrow of the golden window, its block, and this crate's own beside
it. `cur_time 1` is the block after the frame the arrow was created in,
so the left column is the launch **frame**:

| launch | shooter | theirs `sx, sy` | ours | theirs `total_time` | ours | impact theirs | ours |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 644 | `0/6` | 975, 7806 | 888, 7800 | **12** | 13 | 655 | 656 |
| 647 | `0/7` | 1120, 7804 | 1032, 7800 | **10** | 11 | 656 | 657 |
| 650 | `0/8` | 1002, 7953 | 936, 7944 | **10** | 11 | 659 | 660 |
| 652 | `0/10` | 1347, 8201 | 1224, 8184 | **5** | 6 | 656 | 657 |
| 660 | `0/11` | 1406, 8373 | 1320, 8328 | 4 | 4 | 663 | 663 |
| 667 | `0/9` | 1058, 8112 | 936, 8088 | 4 | 4 | 670 | 670 |
| 674 | `0/6` | 961, 7846 | 888, 7800 | **8** | 9 | 681 | 682 |
| 677 | `0/7` | 1103, 7853 | 1032, 7800 | **7** | 8 | 683 | 684 |
| 677 | `0/8` | 1017, 7979 | 936, 7944 | **7** | 8 | 683 | 684 |

Three things fall out of it at once.

**The launch frames are the original's, arrow for arrow.** §9.0's whole
model — the shot added by the animation's release event, the event times
truncated from the install's `<RELEASEEVENT starttime>`, the one-frame
deferral §6.2 puts between the swing and the animation — is confirmed on
nine shots of a second piece pair, and `recharging` says the swing
frames under them are right too.

**The launch point is the unit's own square.** `sx, sy` is the guy's
position plus a release-node offset of eighty to a hundred and
twenty-five units; ours was the guy's position exactly, because
`launch::MEASURED` had a row only for the Longbowman's piece and §22's
own seam sends an unmeasured piece to the unit's square.

**So the whole of the lag is `total_time`**, which is
`(int)(sqrtf(dist²) / (float)(proj_speed × unit_move_speed))` (§9.1) —
a truncated quotient, and a launch eighty units short of where the
original's starts lengthens `dist` by eighty. The bowmen's `proj_speed`
is 100 and the slingers' 150, so the extra eighty crosses a whole frame
most of the time and not always: `0/11`'s and `0/9`'s arrows keep their
`total_time` of 4 and land on the dump's own frame, which is why two of
the nine impacts agreed while the ladder was wrong. **That is what made
the residue look like arithmetic.** An impact frame that agrees is not
an arrow that flew right.

An impact lands at `launch + total_time − 1` on both sides: `cur_time`
is stepped before the test and the ammo list is walked in the same frame
the animation pass adds to it (§9.2), which the table's right-hand
columns confirm nine times over.

### 41.3 Five release nodes, solved from run112's arrows

The offsets, `sx, sy` less the shooter's guy position (which is the
unit's position exactly on all nine blocks, and whose `angle` is the
unit's):

| key | samples | facings | bearing | radius | `dz` |
| --- | --- | --- | --- | --- | --- |
| 472, `ATTACK2`, 9 | 2 | 2 | −17,683,648 | 86 | 164 |
| 472, `ATTACK1`, 12 | 3 | 3 | −37,883,648 | 89 | 163 |
| 472, `ATTACK3`, 15 | 1 | 1 | 81,516,352 | 67 | 164 |
| 384, `ATTACK2`, 22 | 2 | 2 | 216,316,352 | 125 | 151 |
| 384, `ATTACK1`, 21 | 1 | 1 | 532,316,352 | 97 | 178 |

Piece **472** is chapter two's bowmen and **384** its slingers; the key
is §22's own — `(gpiece, animation slot, the starttime frame
`rondata::artdata::release_frame` produces)`. Each bearing is stored
relative to the guy's facing and turned by `movement`'s integer sine, so
the two keys with two and three samples at *different* facings are a
test of the rotation and not a stored world vector: they are the same
two numbers answering differently.

**Three of the nine are a unit or two out**, where run109's nine were
exact. A single planar `(bearing, radius)` cannot hit all three rows of
`(472, ATTACK1, 12)` at once — the original rotates a float vector that
has a height in it, and `dz` is 163 there — so
`run112_launch_points_are_reproduced_to_within_a_unit` pins the error
**per row** rather than asserting it away. Six are exact, and the worst
is two units of Manhattan distance. Nothing integral depends on the
residue: the flight time is a truncated `dist / proj_speed` and a unit
of distance moves it only across a boundary, and the consequence the fix
is measured on is the impact *frame*, which the widening now puts on the
dump's own for every arrow whose node is here.

**What is not established, and it is `damage 1/6`'s 680 row.** The
slingers' `(384, ATTACK3, 16)` has no measurement, because the one arrow
the window fires from it lands in its own launch frame and so is never
printed: `total_time` reaches its end before the block is written. That
arrow is the fourth hit on `1/6`, ours at 680 against the dump's 679,
and it is the only launch-point row the widening still carries. A
capture that raises `AMMO` on a slinger volley at longer range, or a
window whose slinger fires from further off, closes it; nothing on disk
does.

### 41.4 A figure falls at its share, not its squad's

§40.6's hypothesis, measured rather than read: `Sim::take_damage` passed
`u.health` — the squad-sized `myhits` less this figure's damage — as the
threshold, where §7.2 step 8 divides. So a hoplite figure of a squad of
three absorbed 120 points instead of 40, and run112's `1/8` reached
`damage` **51** on block 685, `killed: false`, where the original's died
on 683 at its fortieth point.

**`combat::share` was already there.** It was written for §7.2 step 8,
it carries the captain's remainder, it is documented, and no call site
had ever used it. The fix is that call: the accumulated whole points go
in as `max_health − health` and the share beside them, and `health`
stays the complement of the dump's `damage` against the squad-sized
`myhits`, which is what item 484's row pins on 245,679 field-frames.

`extra 1/8` closes with it. And the squad that loses a figure on the
original's own block then behaves like the original's: the set this
crate wounds over the whole chapter gains `(0, 9)` — a second slinger,
which the dump wounds too — and is now one unit short of the dump's own
six rather than two.

**The attrition path still uses the squad-sized threshold.**
`Sim::attrition_tick` deducts from `health` and kills on `health <= 0`,
so a figure bleeding inside hostile borders still absorbs its whole
squad's hit points. That is the same defect on the other caller of
`Object::take_damage` (the `attrition != 0` arm), it is
`docs/ATTRITION.md`'s to fix, and no capture on disk has an attrition
death to score it against — which is exactly why it survived. Named
here, not fixed here.

### 41.5 What moved, and what 683 is now

The map over `[606, 687)`, eight rows to five:

```
  gone   damage 1/8        656   ours 0  theirs 8      the whole ladder
  gone   damage_frac 1/8   656   ours 0  theirs 10
  gone   damage_frame 1/8  656   ours 0  theirs 655    (new row, 485)
  gone   extra 1/8         684   the original's 1/8 dies on 683; ours does now
  gone   pos 1/4           686
  moved  damage 1/6        657 → 680     the unmeasured ATTACK3 node
  stands damage 1/7        686   ours 0  theirs 19
  stands damage_frac 1/7   686   ours 0  theirs 5
  stands damage_frame 1/7  686   ours 0  theirs 685    (new row, 485)
  stands order 1/4         685
```

`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` holds at
**9 of 9**; `chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame` at
4848 of 4848 with 170 live; item 484's floors — run57's 205,302 and
run79's 40,377 field-frames — are unmoved, as they must be, since
neither capture contains a fight. Chapter one's word is 626 and Great
Lakes' 10277, both unmoved by this landing.

**The word holds at 683, and it is a different 683.** Ours is now seven
draws against the original's six, parting at draw 0, and both sides of
the parting have a name:

```
  ≠  0  ours Ammo::do_damage+0xc59   theirs 60fb06
  ≠  1  ours Ammo::do_damage+0xc7e   theirs Farms::inc_time+0x1ae
     2  ours Farms::inc_time+0x1ae   theirs Farms::inc_time+0x1ae
```

- **`0060fb06` is inside `Unit::close@0060ee50`** (the decompile export's
  index; §11's death path), and it is the death animation's own draw:
  `anim = dtype × 2 + 0xd + (Random::get(0, 0xffff) % 2)`, taken when
  `dtype != 0`, the piece answers both vtable slots and
  `ObjectData +0x68 & 1` is clear — three draws instead of one when
  `dtype == 4`. The dump confirms the outcome as well as the site:
  `DEATH_OBJS` on block 684 carries `cur_anim 17` for `1/8`, which
  forces `dtype = 2` and `roll % 2 == 0`. This crate's `close` takes no
  draw at all, and it never could be scored before this item, because
  before it the figure did not die. **`dtype` is what the successor
  owes**: §7.1 step 6 gives only 4 (an ammo graphic flagged `0x10`) and
  1 (a Build proper), and `cur_anim 17` says a normal arrow on a foot
  unit is 2.
- **Our own two are §39's puncture pair**, and they are a second
  successor. Both of the window's last arrows land on 683 in both
  simulations; the first kills `1/8`; the second finds its target dead,
  and where the original holds it, this crate lands it on nothing and
  punctures the ground. ~~§9.2's own sentence is the mechanism — "a
  target that is no longer active has its `hold_frames` bumped"~~ —
  **wrong twice over, and item 491 measured both**: that bump is the
  *shooter*'s slot (§42.3), and what the original does is
  `Ammo::inc_time`'s flag-4 roll (§42.2). The frame and the draw delta
  were right; the mechanism was not, which is `docs/DECISIONS.md` 42
  again. The dump's own `1/7` takes `19+5` in one step on frame **685**,
  which is where the rolled arrow comes down, and that stays open as the
  successor 491 parked (§42.5).

### 41.6 Coverage

**Diff-backed**:

- that `recharging` is the dump's on every unit-frame of chapter two's
  widened window and on every AI unit of run100's word block — the
  reload clock agrees, so the swing frames do;
- that the overkill window agrees: `damage_frame` on every unit-frame of
  both, and `damage_o`/`damage_who` wherever both sides hold a live one;
- that `1/8`'s wound ladder is the dump's, value for value and frame for
  frame, over the whole window — `damage`, `damage_frac` and
  `damage_frame` all gone from the map;
- that `1/8` dies on block **683**, the original's own block, which is
  `extra 1/8` closing;
- that a melee captain's retarget, `near_o`, `visible` and both maps'
  words are untouched by all of it.

**Dump-backed** (measured from run112's records, not from a diff that
re-runs): §41.2's nine arrows and §41.3's five nodes.
`run112_launch_points_are_reproduced_to_within_a_unit` pins each launch
point against the dump's `sx, sy` with its own error, six of the nine at
zero; it was made to fail on purpose by moving one radius by a unit.

**Listing-backed**: none. `0060fb06`'s pairing with `Unit::close` is the
decompile export's index and the draw arm is read from its decompiled
body; `cur_anim 17` is the dump agreeing with the reading, which is why
§41.5 states the outcome and not just the site.

**Reading-only**:

- that `dz` is carried and unread. The flight time is a plan distance in
  §9.1's own formula, and the three `dz` values here (163, 164, 178,
  151) are recorded because the record prints them.
- §41.4's captain remainder — `combat::share`'s `captain_alone` arm —
  which needs a squad down to one figure and which no capture on disk
  reaches. Its arithmetic is tested; its predicate is §7.2 step 8's
  reading.

**What this does not establish**:

- `(384, ATTACK3, 16)`'s node, which is `damage 1/6` at 680 (§41.3).
- ~~`dtype`, which is the `Unit::close` draw's gate (§41.5).~~ **Closed by
  item 491**: §42.1 derives it and the `DEATH_OBJS` row backs it.
- ~~what the original does with an arrow whose target dies in flight
  (§9.2's `hold_frames` bump), which is our puncture pair at 683 and
  `1/7`'s `19+5` at 685.~~ **Closed by item 491, and the named mechanism
  was wrong**: the bump is the *shooter*'s and the arrow **rolls on**
  (§42.2). `1/7`'s `19+5` stays open as the roll's *landing* — the
  successor 491 parked, §42.5.
- the attrition caller's threshold (§41.4).

---

## 42. The death draw, and the arrow that rolls on (item 491, 2026-09-22)

§41.5 handed this item one frame and two hypotheses, and the frame was
right both times. Chapter two's word stood at **683** with this crate
spending seven draws against the original's six and the streams parting
at **draw 0**:

```
  ≠  0  ours Ammo::do_damage+0xc59   theirs 60fb06
  ≠  1  ours Ammo::do_damage+0xc7e   theirs Farms::inc_time+0x1ae
     2  ours Farms::inc_time+0x1ae   theirs Farms::inc_time+0x1ae
```

Two arrows land on 683 and the sides differ about both of them. Theirs
takes a draw this crate had never taken — the death animation's, and
§41.5 named its site correctly. Ours spends two the original does not,
and §41.5's mechanism for that — §9.2's `hold_frames` bump — is not the
mechanism and is not even about the right object. One draw gained, two
lost, and the word moves **683 → 695**.

### 42.1 `Unit::close@0060ee50+0xcb6` — one draw per death

`Object::take_damage` step 10 ends `T.die(dtype, gpiece, angle)` — the
virtual at `+0x158` — and `Object::die@00647080` is `close()` (the
virtual at `+0x150`, `Unit::close` for a unit) and then the slot hold of
§11. Near the foot of `close`, before `Object::close` runs:

```
  60fa68  call *0x8(%eax)          ; SubObjectData::is_active  → else skip
  60fa77  call *0xbc(%eax)         ; UnitData::is_on_map       → else skip
  60fa85  testb $0x1, 0x68(%ebx)   ; unit_masks & 1, the decoy → else skip
  …
  60fb01  call Random::get         ; (game_random, 0, 0xffff)
  60fb06  and   $0x80000001, %eax  ; % 2, sign-fixed
  60fb12  lea   0xd(,%ebx,2), %ebx ; %ebx is the (signed char) dtype
  60fb19  add   %eax, %ebx         ; cur_anim = dtype*2 + 0xd + roll % 2
  60fb1b  cmpb  $0x4, 0x8(%ebp)    ; dtype == 4?
  60fb31  call Random::get         ;   cur_anim = roll % 2 + 0x11
  60fb55  call Random::get         ;   facing   = 0xe - roll % 4
```

Above all of it, `dtype != 0` and the function's own `local_1c`, which is
set only on the arm that unblocks four tiles for a **Merchant**, a Dutch
Merchant or a Fur Trapper (type index `0x3d`, `0x3e`, `400` —
`TypeIndex`'s `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER`); those three
take no death draw. `is_active` here is the slot's allocation flag, not
its health: the object is still allocated and still on the map while
`close` runs, and it is `Object::close` at the foot of the function that
takes it off.

So **one draw on an ordinary death and three on a `dtype == 4` one**, and
the anim the first chose is discarded in the second case.

**`dtype` is derived, not assumed.** §41.5 had `cur_anim 17` agreeing
with a reading; two instruments make it a derivation.

- The formula's only unknowns are `dtype` and the parity. run112's trace
  carries the seed *before* the draw at `0060fb06` on frame 683:
  `0xadfe4aea`. Stepping the LCG (§10) gives `0x19660d × seed +
  0x3c6ef35f`, and `((seed & 0xffff) × 0xffff) >> 16` = **64832**, which
  is even.
- run112's `DEATH_OBJS` carries `cur_anim 17` for `1/8`. With the parity
  known, `2 × dtype + 0xd = 17` and `dtype = 2` — no other value fits,
  because 17 is odd only through the roll and the roll is even.

And 2 is what the reading gives independently: `get_damage` writes 2 at
its head (§6 step 11) and the only writers after it are EXPLOSIVE
(`obj_masks & 0x800000`) and BOMBARD (`& 0x2`), each 3, and a **building**
attacker, which overwrites to 3 for a `REDOUBT` shot and 2 otherwise.
`do_damage` then overrides to 4 for an ammo whose graphic piece carries
flag `0x10` and to 1 for a Build-proper target (§7.1 step 6). An arrow
from an archer at a hoplite is none of those.

**The other three deaths of the capture say the same.** run112 prints
625 `DEATH_OBJS` records over four deaths — `1/8` from 683, `0/11` from
729, `1/6` from 743, `1/7` from 816 — and every one carries `cur_anim
17`. Four even rolls in a row is a one-in-sixteen coincidence and it is
not one: it is four deaths at `dtype 2`, and the trace settles the first.

### 42.2 The puncture was never the mechanism: `Ammo`'s flag 4

§41.5 proposed §9.2's sentence — "a target that is no longer active has
its `hold_frames` bumped" — as what the original does with an arrow whose
target dies in flight. It is not, and §42.3 says why. What the original
actually does is in `Ammo::inc_time@0067d380`'s tail, and run112 had been
printing it since the day the capture was taken.

The two arrows `0/7` and `0/8` fire on 677 with `total_time 7`; both
carry `flags 6`. The block after the impact frame:

```
  block 683   AMMO o 7  cur_time 6/7  whom 1 ox 8  flags 6
  block 683   AMMO o 8  cur_time 6/7  whom 1 ox 8  flags 6
  block 684   AMMO o 8  cur_time 7/7  whom -1 ox -1  flags 14      ← still there
  block 685   AMMO o 8  cur_time 8/7  whom -1 ox -1  flags 14
  block 686   —                                                    ← gone
```

`0/7`'s arrow is gone on 684: it hit, and killed `1/8`. `0/8`'s is
**still in the list**, its target forgotten and flag `8` set, and it flies
two more frames. The arm:

```
if (flags & 4) {
    if (!(flags & 8) && !hit_target() && !check_hit(GROUND)) flags |= 8;
    if (flags & 8) {
        if (total_time * 3 < cur_time) { close(); return; }   // no do_damage
        …advance along the line at t = cur_time / total_time…
        if (!WorldData::is_valid(x, y)) { close(); return; }  // no do_damage
        if (terrain_z < arrow_z) return;                      // still flying
        pos = (x, y, z); total_time = cur_time;               // landed
    }
}
graphic_finish();
if (!(flags & 0x10)) do_damage();
```

**Flag 4 is set in `Ammo::init@0067bbf0+0x96f`** when three things hold:
the shooter's order is not `ATTACK_GROUND`/`AIR_ATTACK_GROUND` (the
function's `local_38`, which is the ground-order pointer and is null for
a building shooter), the target is a unit whose type's domain
(`+0x218`) is **land**, and the piece is not ~~lofted~~ a missile
(`ammo_flags & 8`, the same flag that makes `traj` 2 and sends the shot
down a spline; it is `<ammo missile="1">`, §55.1).
Every arrow of run112's window is `flags 6` — live and rolling — and the
three that ever miss are `flags 14`.

So the original spends **nothing** on the frame a rolling arrow finds its
target gone; this crate spent §39's puncture pair. That pair is real and
its site is right — it is what a *non*-rolling shot, or a rolled one that
has come down, spends — and Great Lakes' three puncture frames (10237,
10242, 10249, §39.2) are untouched by this landing, because their target
is a farm and a building target never sets flag 4.


**A unit strafer's round never takes flag 4** (`Ammo::init`, `67c548`..
`67c557`, item 1200): the `w` test jumps past `67c633`, where the flag and
the `0x4b` over a land unit are set. The Biplane's round on 1529 lands
where it lands, short of `0/9` (`docs/GOLDEN.md` §50).
### 42.3 `hold_frames` is the shooter's, and it is zero on every living unit

`Ammo::inc_time` opens its loop with

```
if (this->field_0x3c >= 0 && this->field_0x40 >= 0 &&
    (objects[field_0x3c][field_0x40]->flags & 1) == 0)
    *(short *)(that + 0x32) += 1;
```

and `Ammo::init` fills `+0x3c`/`+0x40` from the **shooter**, not from
`+0x48`/`+0x4c`, which is the target. `Object::die`'s own scan matches on
the same pair (§11). So all three writers of `hold_frames` — `die`'s
initial `max`, `inc_time`'s bump, and `DeathObj::inc_time`'s — are on a
**dead** object, and the field's whole job is to keep a dead archer's
slot alive until its last arrow lands. It has nothing to say about a
target.

That is now a row rather than a reading. `hold_frames` is printed in the
`OBJECT` block at every detail level and `coverage`'s pin had carried it
as unread since the guard was built; `crate::diff::compare` reads it as a
fourth `hits_diverged` field, and what it asserts is that **every living
unit-frame of both headline windows is zero** — which is what a dump
whose only writers are on dead slots must say, and which nothing had
checked. It was made to fail on purpose by starting a unit at 3.

### 42.4 What moved

| | before | after |
|---|---|---|
| golden chapter two, **word** | 683 | **695** |
| golden chapter two, **sequence** | 683 | **695** |
| golden chapter two, **values** | 684 | 696 |
| the widening's window | `[606, 687)` | `[606, 699)` |
| `near_o` unit-frames read / live / agreeing | 4848 / 170 / 4848 | 5568 / 206 / 5568 |

Twelve frames, and the new word is an animation wrap: ours has a third
`Guy::set_anim+0x97a < Guy::inc_time+0x1ed` — the attack's own end —
where the original's third is a plain `+0x271` wrap.

The widened map over `[606, 699)` is thirteen rows and they are two
facts. `order 1/4`, the far-off citizen, is **gone**, and nothing new
stands under 686.

```
  gone   order 1/4        685
  stands damage 1/6       680   §41.3's unmeasured release node
  stands damage 1/7       686   ours 0     theirs 19
  stands damage_frac 1/7  686   ours 0     theirs 5
  stands damage_frame 1/7 686   ours 0     theirs 685
  new    order 0/6        696   ours target (1,7)  theirs (1,6)
  new    order 0/7        696   ours target (1,7)  theirs (1,6)
  new    order 0/8        696   ours target (1,7)  theirs (1,6)
  new    angle 0/7        696 · angle 0/8 696 · angle 0/6 697
  new    recharging 0/7   696 · recharging 0/8 696 · recharging 0/6 697
```

The twelve are **one arrow**. The original's rolled `0/8` shot comes down
two frames late and `check_hit` finds `1/7` within two tiles, for
`19+5/16` on frame 685 — a **flank** hit, which is what makes it worth
more than double an ordinary arrow: the ladder on `1/8` is `8+10/16` a
shot, and both reduce to `get_damage`'s own arithmetic with `base` 320
and `armor` 6 (`(320+5)/10 − 6 = 26`) against `base` 640 doubled by §6
step 19's side sector (`(640+5)/10 − 6 = 58`), scaled by §7.1 step 5's
`dmg × 0x100 / uber_size 3`. ~~A wounded `1/7` then outranks `1/6` on §33's
damage weight, and the original's three bowmen retarget on 696 where ours
do not — which is the nine rows above.~~ The retarget is `compare_target`'s
bearing, not the wound, and the flank is 57 plus a height point (§46.5). That landing is the successor 491 parked, and it is **not** a residue: §42.5 says what it needs.

### 42.5 What is not established

- ~~**Where a rolled shot comes down.** The arc is
  `v1z × t + sz + GRAV_Z × t² / 2` in IEEE singles, `v1z` itself a float
  the `AMMO` record prints (`18.134823` for `0/8`'s), and
  `TerrainOut::find_data_z@00866560` is a bilinear interpolation over a
  float height surface this crate does not carry — the corner grid
  `crate::terrain` models is the integer `master_land_heights`, a
  different quantity. So a rolled shot here flies to its `3 × total_time`
  cap and is dropped where the original's lands and damages. Neither
  spends a draw, so the word does not see it; `damage 1/7` at 686 does.~~
  Landed in §46. The surface *is* `master_land_heights`: `TerrainOut+0x4a4`
  is `TerrainData+0x464` behind the 0x40 prefix (§46.2).
- ~~**The lofted piece.** `Ammo`'s flag 4 also requires `ammo_flags & 8`
  clear, and this crate loads no ammo flags; every shot is treated as
  non-lofted. A siege shot is a ground shot and so never reaches the
  question, which is why no capture on disk can tell.~~ Named by item 602, §55.1: the bit is `<ammo missile="1">` in
  `effects_graphics.xml`, which no arrow carries. This crate still loads no
  ammo flags, so a missile (`SmallRocket`, `SAM Rocket`, `SubTorpedo`, the
  `AARocket`s) at a land unit would roll here; no capture fires one.
- **`get_shot`**, so a `REDOUBT`'s shot gives `dtype` 2 here and 3 in the
  original. It changes a death animation index; no capture has one.
- **`DEATH_OBJS`' `gpiece`**, the dead unit's own death piece
  (`DeathObj::init`'s `vtable[0x178](param_7, packed)`). Parsed and not
  compared — this crate loads no art for it.
- **The cull.** `DeathObj::inc_time` clears the object when its animation
  packet ends, and this crate never does. No capture on disk reaches a
  cull: run112's four deaths are all still in the list on its last block,
  216 frames after the first, so the grow-only list is exact on every
  window that exists. ~~run146 culls on 771 (parked 617).~~ It does not:
  its three records live from their deaths to 999 on both sides. What
  parted on 771 is the slot the record holds, §59.
- **`nuke_effect[0x108]`** in §11's hold, taken as zero.

### 42.6 Coverage

**Diff-backed**:

- that the death draw is taken where and when the original takes it —
  chapter two's word goes 683 → 695 and the streams agree draw for draw
  over twelve more frames, with `0060fb06` now named rather than read as
  a bare address;
- that `dtype` is 2 for an arrow on a foot unit, and that the roll was
  even: the whole `DEATH_OBJS` record agrees on every block of the
  window, `cur_anim 17` included — 45 field-comparisons over fifteen
  blocks, pinned so an instrument that stops looking fails;
- that `hold_frames` is zero on every living unit-frame of both headline
  windows (§42.3);
- that a rolling shot spends no draw on the frame its target is gone —
  the two draws this crate spent are gone from 683 and Great Lakes'
  three puncture frames are unmoved;
- that the launch points, the wound ladder, `recharging`, the overkill
  window, `near_o` and `visible` are all untouched by it (§41.6's list,
  re-measured on the wider window).

**Listing-backed**: the three draw sites and the gate above them,
`0060fa4c`-`0060fb70`, read from `llvm-objdump` where the decompiler's
`fVar19 = (float)(param_1 * 2 + 0xd + uVar10)` prints an integer
expression through a float local.

**Dump-backed** (measured from run112's records rather than from a diff
that re-runs): §42.2's five `AMMO` blocks and the three `flags 14`
records of the whole capture; the four `cur_anim 17` deaths.

**Reading-only**:

- the `MERCHANT`/`MERCHANTDUTCH`/`FURTRAPPER` exclusion, which no capture
  reaches;
- the `dtype == 4` pair of draws, likewise;
- §42.2's flag-4 predicate beyond "a land unit target that is not a
  ground shot" — the lofted term is unmodelled and the ground-order term
  is exercised only by siege, which no capture on disk fires here.

## 43. The frozen frame, and the target the order keeps (item 496, 2026-09-22)

§42 left chapter two's word at **695** with this crate spending twenty-one
draws against the original's twenty and **every value on the frame
agreeing** — a draw-stream parting with no value row under it, which is a
different item shape from the four before it. The three rows the widening
carried on 695 were item 495's, standing unchanged since 686.

The extra draw is one `Guy::set_anim+0x97a < Guy::inc_time+0x1ed` — an
attack animation running out — and the count is three against two.
Naming the unit needed no hypothesis: the walk's marks carry it
(§43.4), and the three are the **bowmen** `0/6`, `0/7` and `0/8`, whose
clocks this crate runs in perfect lockstep from the frame they all
started an attack (666) to the frame they all end one (695).

### 43.1 `unit_masks2 & 0x10` — the step is zero, and the dump says so

`Guy::inc_time@005d9e10` opens with the step, and the step has three
values rather than two:

```text
  step = 1
  if (guy_flags & 4 && cat[cur_anim] == CHAR_ATTACK2) step = 2
  if (unit->unit_masks2 & 0x10)                       step = 0     ; +0x6c
  if (guy_num < type->squad_size || cat[cur_anim] == CHAR_WALK) {
      last_time = cur_time; cur_time += step
      while (cur_time >= end_time) { … the wrap … }
```

`docs/ANIM.md` §5 has stated the zero arm since the mechanic was
written, and `crate::diff::unit`'s
`the_frozen_frame_s_figures_do_not_step_their_clocks` has asserted it
against the corpus since 2026-09-05 — its own note ends "the day
`guy_inc_time` learns the bit, this is the check that says what it
should do". [`sim::Sim::guy_inc_time`] steps by one always.

**run118 is the measurement.** run112 was taken at `GUYS=2`, where a
`GUY` block stops after `ox`; at `GUYS=4` it carries the whole clock, and
`GuyData::log_data@005de6c0`'s fourth level is where `cur_time`,
`end_time`, `last_time`, `cur_anim`, `hold_attack`, `queued_attack` and
`guy_flags` live. Chapter two re-taken at `GUYS=4` (`docs/RUNS.md`
run118), block 696 — the state after frame 695:

```text
  0/6  cur_anim 12  cur_time 29  end_time 30  last_time 29   ← no step
  0/7  cur_anim 0   cur_time 0   end_time 31  last_time -1  hold_attack 1
  0/8  cur_anim 0   cur_time 0   end_time 31  last_time -1  hold_attack 1
```

`last_time` is `cur_time` before the step, so their difference **is** the
step the original took, per figure, per frame, already on disk. Every
other block of the window has `last = cur − 1` on all three. On 695
`0/6`'s is `last = cur = 29`: the clock did not move, the wrap did not
come, and the draw was not spent. `0/7` and `0/8` wrapped, which is the
original's two.

And the bit itself is in the **`OBJECT` block at every detail level**, so
run112 has printed it since the day it was taken. It is non-zero on
exactly **six unit-frames of the whole capture** — `0/6` at 696, `1/6` at
736, `0/9` at 745, `0/7` and `0/8` at 756, `0/6` at 757, `1/7` at 762 —
and the first of them is this item's frame. (Seven unit-frames over
**six** blocks: `0/7` and `0/8` share 756.)

### 43.2 What sets it: the frame a unit finds its target gone

~~`Unit::fight@005fd4d0`'s only writer is `LAB_005fe502`~~ — **wrong, and
corrected by item 502 (§44.1): `Unit::fight` writes the bit at three
sites, and `LAB_005fe502` is not the one this frame takes.** The other two
are in the **invalid-target** branch, which is where a unit whose target
died actually goes, and one of them is `LAB_005fdb9e`'s tail. All three
read the same pair above them, so the *meaning* below is unchanged and
only the site is: `LAB_005fe502`, past the strike block, reached from two
arms —

```text
  find_new_target(this, NULL, 0)
  … the order shuffle: a matching ATTACK order is killed, a move may go in …
  if (order_type() == ATTACK && recharging == 0) {
      unit_masks2 |= 0x10
      return 0                       ; no strike this frame
  }
```

`field_0xae` is `recharging` (`UnitData +0xae`, from the type record, not
from the surrounding code), so the pair reads: *the unit is still ordered
to attack, its reload is open, and it is not going to fire this frame.*
`Unit::process@00610bc0` clears it (`& 0xffffffef`) immediately after
`process_healing`, under the same `inside_up < 0` guard §33.1's
`targeted` decay sits in. So it is a **one-frame** mark, set inside the
frame's order step and read by phase 7's `Objects::inc_time`.

run112's own records say which frame that is for the bowmen, and the
answer is not a hypothesis — it is the `ATTACKORDER`'s target, block by
block:

| block | `0/6` order | `0/7` order | `0/8` order | recharging |
|---|---|---|---|---|
| 683–695 | `1/8` | `1/8` | `1/8` | 13 → 1 |
| 696 | **`1/6`** | **`1/6`** | **`1/6`** | 0 · 30 · 30 |
| 697 | `1/6` | `1/6` | `1/6` | 30 · 29 · 29 |

`1/8` **dies on 683** (§42.1) and all three orders keep it for twelve
more frames, because `Unit::do_attack`'s reload gate returns before
`fight` is ever entered. On 695 the reload opens, `fight` runs, the
target is invalid, `find_new_target` retargets all three to `1/6` — and
then `0/7` and `0/8` strike (`recharging` → 30) while `0/6` does not
(`recharging` stays 0, `unit_masks2` gains `0x10`, and its `GUY`
`whom`/`ox` stay on the dead `1/8` a frame longer).

The `ATTACKORDER`'s own `new_ord`/`ever_in_range` on block 696 say which
of the two routes each took, and they were on disk before anyone asked:
`0/6` carries `new_ord 1 in_range 0 ever_in_range 0` — a **fresh** order,
which is what `add_attack_order` leaves — and `0/7` and `0/8` carry
`new_ord 0 in_range 1 ever_in_range 1` on the same target, which is the
**same order object** with its `(ox, whom, uid)` overwritten. §44.2.

~~**What separates them is the search.** The three bowmen are one squad on
the `o_down` chain, `0/6` at its head (`o_up −1`); at block 696 `0/6`
carries `near_o 6 near_who 1` — §30's search, run this frame and won —
and `0/7` and `0/8` carry `−1`. The captain searched and spent its frame
doing it; the followers took the target down the chain and fired on it.
That reading is **not** diff-backed and §43.5 says what would falsify it.~~

**Half right, and the half that was wrong was the evidence** (item 502,
§44.2). The captain does search and the followers do not — but `near_o 6
near_who 1` is **not** this frame's search. `0/6` has carried that pair
since block **635**, sixty-one blocks earlier, and neither `find_new_target`
nor anything else rewrote it on 695; the pair is a search *footprint* that
is only ever written when a nearer candidate is seen (§37.1), so "unchanged"
and "not searched" look identical in it. What the field actually says is the
weaker thing: captains search and followers never do.

What separates them is in `Unit::fight`'s own head, and it is **not** the
search at all: a **follower takes its captain's target in place, before the
validity test is ever reached**. §44.2 has the block and the two dumped
fields that tell it apart from a fresh order.

### 43.3 The comparison could not see the target at all

The three `order` rows the widening reports at 696 are not where this
crate and the original first disagree about these orders. They part at
**684**, the first block after `1/8` dies, and the comparison read twelve
frames of it as agreement:

```rust
let mine = built.target_ids(link.unit, ours);       // None
let logged = theirs.whom.zip(theirs.ox);            // Some((1, 8))
if let (Some(a), Some(b)) = (mine, logged) && a != b { … }   // silent
```

`target_ids` answers `None` for two unrelated reasons and both were
skipped. For a `Move`, `Cast`, `Trade` or `Think` order, and for a
building the simulation made itself, `None` means *this crate does not
model a target here* and reporting it would be noise. For an **attack**
order it means the opposite: the original's `AttackOrder` **is** a
`TargetOrder` and carries the target, while here the order is a wrapper
and the target lives in `combat::State` (`docs/ORDERS.md` §13) — so
`None` is this crate saying the unit is attacking nothing while the dump
names what it is attacking. `crate::diff::order::compare_orders` now
reports that, for an attack order only, and only against a logged pair
that names a real object (`whom >= 0 && ox >= 0`).

What it surfaces is [`sim::Sim::forget`], which drops a dead object from
every attacker's target slot on the frame it dies. The dump refutes it:
the original leaves `ox`/`whom`/`uid` where they are and lets
`Object::valid_target` find out at the next use — which is the same shape
item 463 found for the order's `mandatory`, one field along, and fixed
only for that field. Thirty-six unit-frames, under the word, quiet.

**It is not landed here, and the reason is measured.** Removing the clear
alone leaves the word at 695 — the freeze has no writer without §43.2's
whole arm — and turns two green pins red: `near_o 0/7` and `0/8` part at
696, because this crate's invalid-target arm kills the order and runs
`find_melee_target` on *every* member of the squad where the original
searches only on the captain, and `chapter_two_s_hit_points` gains
`(0, 10)`. The three landing together is the successor, and it is a
mechanic rather than a residue: §43.5.

### 43.3.1 The rule, because the next comparator will have the same hole

State it as a rule about comparators rather than as this item's fix.

> **A comparison written as "compare when both sides carry it" reads an
> empty side as agreement.** Where one side can legitimately carry
> nothing, `None` has to be split into *this crate does not model the
> field here* — which must stay quiet — and *this crate says there is
> nothing there* — which is a divergence and must be reported. A
> comparator that cannot tell the two apart is silent on exactly the
> disagreements that matter most, because a field this crate has stopped
> writing is a bigger fault than one it writes wrongly.

It is a rule and not an anecdote because it has now happened twice in
the same function, one level apart. Item 462: `unit_ids` answered `None`
for a unit that was not on the board at `BEGIN GAME`, so
`OrderMismatch::Target` went uncompared for *every* target born in play
— chapter two's whole cast. Item 496: `target_ids` answers `None` for an
attack order whose unit is attacking nothing, and the same `if let
(Some(a), Some(b))` swallowed it. Both were found by widening a record,
neither by a reading, and between them they hid the golden record's
order targets for a month.

The shape to grep for is `if let (Some(a), Some(b)) = (mine, theirs)` and
its cousins — `zip`, `and_then`, a `?` in a helper that feeds a
comparison. `crate::diff::coverage` catches a key the **parser** never
asks for; nothing catches a key the parser reads and the comparison then
drops on the floor for want of a value on this side. This is the second
kind, and it is the more expensive one, because the first shows up as a
key with no reader and the second shows up as green.

### 43.4 The golden walk's own site window

`walk_chapter` now reads `RON_GOLDEN_SITES=<lo>-<hi>` and prints each
draw of those frames with the unit that spent it, and the original's
labels beside them on a frame they part. It is
`crate::diff::harness::attributed_sites` and `site_window_named`, which
the Great Lakes walk has carried since item 204; this item rebuilt it by
hand as a scratch test before noticing, which is the second time that has
happened (item 432 was the first), so it graduates. `CLAUDE.md`: a probe
shape reached for a third time.

### 43.5 What is not established

- ~~**That the captain's search is what costs it the frame.**~~ **Closed
  by item 502, and the answer was a different mechanism** (§44.2). The
  captain does spend the frame, but not because the search is expensive:
  the invalid-target branch never strikes, whether it searches or not.
  The followers escape it because they never enter it. What the `near_o`
  split does *not* establish is in §43.2's strike-through.
- ~~**Which arm of `Unit::fight` the followers take to strike in the same
  frame they are retargeted.** The re-entry latch (§7.10,
  [`flag::FIGHT_REENTRY`]) is the candidate and it is unread here.~~
  **Neither arm: the follower inherit at `fight`'s head** (§44.2). The
  latch was the wrong candidate — it needs `unit_flags2 & 1`
  (`is(MACHINEGUN)`/`is(FLAMETHROWER)`), which a bowman does not have,
  and the dump agrees: with the latch live these three would have run
  `find_new_target` on every one of the twelve recharging frames and
  carried `unit_masks2 0x10` on each, where the dump has them quiet.
- **The search budget above `LAB_005fdb9e`**, which item 502 read and did
  not model. `waiting < 5 && leaders[who].searches > 10` sets the same
  bit, adds 2 to `UnitData::waiting` and returns *without* searching;
  `Leader::process@006b88b0` zeroes the counter at the head of each
  leader's frame, so it is a per-leader per-frame throttle, and
  `Unit::resolve_unit_collision@005f9d30` reads the same `< 10`. run112
  says it did not fire on 695 — `0/6`'s `waiting` is 0 on block 696,
  where the throttle would have left 2 — so nothing on disk exercises it.
  A capture with a dozen simultaneous retargets under one leader would.
- **`guy_flags 48`** on every bowman-frame of run118 — `0x10 | 0x20`,
  where `docs/ANIM.md` §9 has `0x20` as a bit with no writer found and
  reads it as collapsing the idle roll, which is a draw. run44's table
  (`docs/RUNS.md`) has the same split by type. `0/8` carries **16** on
  block 697 alone, so the bit is written and cleared inside this window,
  and nothing here reads it.
- ~~**`unit_masks2` as a compared field.**~~ **Landed with the
  implementation** (item 502): `crate::diff::compare`'s firing record
  carries it, and `chapter_two_s_frozen_frames_are_the_dump_s_on_every_
  unit_frame` pins both sides' lists over the whole chapter. **Only the
  `0x10` bit is compared**, and §44.4 has the histogram that decides it —
  across every dump on this disk the word takes four values and the other
  two have no writer here, one of them on the *other* headline's own
  capture.
- **run118 is a partial capture**, and run119 is the same line with
  `--timeout` raised (`docs/RUNS.md`). It timed out at 180 s having written
  243 of the window's 296 blocks — `GUYS=4` costs about four times
  `GUYS=2` per block — so it covers 605–846 and stops mid-record.
  Everything above is inside it; nothing here reads past 700.

### 43.6 Coverage

**Diff-backed**: the three `order` rows at 684, both directions, on the
widening window — this crate's attack order names no target from the
block after `1/8` dies while the dump names `1/8`, and the map's first
parting moves 686 → 684 with the comparator's fix and nothing else.
That the fix changes no behaviour is itself the measurement: the release
gate is green on every other test on both maps.

**Dump-backed** (measured from records rather than from a diff that
re-runs): §43.1's three `GUY` blocks of run118 and the `last = cur`
step; the six unit-frames of run112 that carry `unit_masks2 & 0x10`;
§43.2's `ATTACKORDER` table, block by block, and the `near_o` split
between the captain and its two followers.

**Listing-backed**: `Guy::inc_time@005d9e10`'s three-valued step and the
`unit_masks2 & 0x10` read at its head; `Unit::fight@005fd4d0`'s
`LAB_005fe502` and the `order_type() == ATTACK && recharging == 0` pair
above it; `Unit::process@00610bc0`'s clear under `inside_up < 0`.
`field_0xae` is named `recharging` from `UnitData`'s type record rather
than from the surrounding code.

**Reading-only, and one of them has been in that state for
seventeen days.** §5 of `docs/ANIM.md` has stated the zero step since the
mechanic was written and `sim::Sim::guy_inc_time` has never implemented
it. `the_frozen_frame_s_figures_do_not_step_their_clocks` is green and
asserts the **original's** rule against the corpus — its own note says
`rondata::diff` compares no clock on a frame a unit is in melee, so no
diff could fail on the gap. So the claim was corpus-backed on the
original and reading-only against this crate at the same time, and it is
the second half that decides whether a divergence can be seen: the word
sat at 695 through four items while three separate artefacts — the
document, the assertion and the dump's own field — already carried the
answer. Named here because `docs/audit/README.md` wants claims in that
state named rather than counted as covered.

Also reading-only: §43.2's captain-searches split (§43.5's first
bullet), the arm the followers take to strike in the frame they are
retargeted, and the two predicates above `LAB_005fe502`.

## 44. §43.2's arm, landed whole (item 502, 2026-09-22)

§43 measured the cause of chapter two's word at **695** and implemented
none of it, deliberately: removing [`sim::Sim::forget`]'s clear on its own
left the word where it was and turned two green pins red. This is the
landing of all four pieces together, and the word moved **695 → 725**.

The four, in the order the frame runs them:

1. `Sim::forget` stops clearing a unit's attack target (§43.3). The order
   holds the dead `1/8` from 684 to 695 as the dump does, so all three
   bowmen enter `fight` on **one** frame instead of going idle twelve
   frames early.
2. `do_attack` gained `Unit::fight`'s **follower inherit** (§44.2), which
   is what lets `0/7` and `0/8` strike on the frame `0/6` spends
   searching.
3. The invalid-target arm sets `unit_masks2 & 0x10` (§44.1).
4. `Sim::guy_inc_time` steps by **zero** while the unit carries it —
   `docs/ANIM.md` §5's third step value, stated since the mechanic was
   written and never implemented.

### 44.1 The bit has three writers, and the one this frame takes is not §43.2's

`unit_masks2` is `UnitData +0x6c` by the type record, and
`Unit::fight@005fd4d0` sets `0x10` at three sites rather than one. Two of
them are inside the **invalid-target** branch — the branch a unit whose
target has died actually takes, which `LAB_005fe502`'s is not:

```text
LAB_005fdb9e:                                  ; valid_target failed, param_5 == 0
  if (param_4 == 0) {
      if (waiting < 5 && leaders[who].searches > 10) {
          unit_masks2 |= 0x10                  ; (A) the search throttle
          waiting += 2
          return 0                             ; …and no search at all
      }
      find_new_target(this, NULL, 0)           ; kill the order, search, re-order
  }
  leaders[who].searches += 1
  if (order_type() != ATTACK)               return 0
  if (recharging != 0 && !reentry_latch)    return 0
  unit_masks2 |= 0x10                          ; (B) the one 695 takes
  return 0
```

`(B)` and `LAB_005fe502` read the same pair — *still ordered to attack,
reload open* — so §43.2's reading of what the bit **means** stands whole;
only the site was wrong. What `(B)` adds is the condition §43.2 could not
state: the mark is set **only if the search put an attack order back in
front**. A unit that finds nothing has no order to be "still ordered"
under and takes no mark, which is why the bit is as rare as §43.1 found
it.

**The dump tells `(A)` from `(B)` in one field.** The throttle adds 2 to
`UnitData::waiting` and run112's `0/6` carries `waiting 0` on block 696,
so 695 is `(B)`. `(A)` is unmodelled and §43.5 says what would exercise
it.

### 44.2 The follower does not search because it never gets there

Between `fight`'s cell-centre snap (§7.11) and `:196`'s
`Object::valid_target` sits a block that runs for a unit that is **not** a
captain:

```text
  if (!is_captain(this) && param_5 == 0 && collide_frame < frame - 1) {
      cap = get_captain()                      ; vtable +0xe4, recurses to the head
      act = get_action(units[who][cap])
      if (act && act->type == ATTACK) {
          (o, whom, uid) = act->target
          if ((o, whom) != my order's target
              && valid_target(this, o, whom)           ; tested on ME, not the captain
              && (get_combat_stance() < STAND_GROUND || is_in_range(this, o, whom)))
          {
              my order's (ox, whom, uid) = (o, whom, uid)   ; in place
          }
      }
  }
```

On 695 `0/6` is processed first (units walk in `o` order), finds `1/8`
invalid, searches, retargets to `1/6` and freezes. `0/7` and `0/8` then
read their captain's **already updated** action, adopt `1/6`, pass
`valid_target`, and fall through to the strike. Nobody searches twice and
nobody loses a frame.

**Two dumped fields tell this apart from a fresh order**, and they cost a
grep: an `add_attack_order` leaves `new_ord 1` and `ever_in_range 0`,
while this writes into the existing `TargetOrder` and touches neither. On
block 696 `0/6` carries `new_ord 1 in_range 0 ever_in_range 0` and `0/7`
and `0/8` carry `new_ord 0 in_range 1 ever_in_range 1` — on the same
target, `ox 6 whom 1 uid 12`. That is the falsifier §43.5 asked a capture
for, sitting in run112 since it was taken.

It is a **second** captain mirror and it is not §21's: that one is
`Unit::think`'s opening arm, runs only on an idle unit and adds a
`QUEUE_NEW` order. This one runs on the attack step of a unit that
already has one.

### 44.2.1 A field whose write condition makes it silent cannot date an event

State this one as a rule too, because it is how §43.2 came to cite the
wrong evidence for a mechanism it had otherwise measured.

> **Before a field is used to say that something happened *this frame*,
> its write condition has to be checked — not its value.** A field that
> is written only when it *changes*, or only when a better candidate is
> found, or only on the branch that succeeds, reads identically whether
> the event happened this frame or has not happened for sixty. Its value
> is evidence about the last write, and nothing about when.

`near_o`/`near_who` is exactly that shape. `Object::find_nearby_target`
writes the pair only when a candidate closer than every previous one
clears `check_target`, and clears it to `-1` only when the nearest thing
it saw is past `0xf00`; a search that re-finds the same winner writes the
same values, and a search that finds nothing nearer writes nothing at
all. So `0/6` carrying `near_o 6 near_who 1` at block 696 is consistent
with "searched this frame and won" and with "has not searched since block
**635**", which is what the dump actually says — sixty-one blocks earlier,
and `--changes` on `tools/gamelog/track.py` says so in one line.

It is the sibling of `CLAUDE.md`'s "grep the writers of every field you
call frozen": grep the **history** of every field you call fresh. The two
failures are the same one from either end, and this one is the more
dangerous, because a stale value looks like a measurement and a frozen
one looks like a bug.

### 44.3 What it moved

The word is **725**. Ours spends **7** draws against the original's **9**,
parting at draw **1**, and the whole difference is two
`Guy::set_anim+0xf2f < Guy::inc_time+0x271`. **That is the measurement;
what follows is a hypothesis** (`docs/DECISIONS.md` 42).

The measured part, from run118's clocks and this crate's own marks:

- **An attack-end wrap costs the original two draws here and this crate
  one.** Three of them fall in three consecutive frames and all three
  behave the same: `0/7` (`cur_anim 12 → 11`) and `0/8` (`11 → 12`) wrap
  on **725**, `0/6` (`12 → 11`) on **726**, each spending
  `+0x97a < inc_time+0x1ed` — the `set_anim(CHAR_DEFAULT, 0, 0)` — **and**
  `+0xf2f < inc_time+0x271`. This crate spends the first of each pair and
  not the second. It is not one figure's oddity: 725 is two of them and
  726 is the third, and 726's whole label list is otherwise identical.
- **The same wrap on 695 cost the original one draw, not two.** `0/7`
  and `0/8` wrapped there too and the frame carries two `+0x1ed` and no
  `+0x271` at all — run118's block 696 has both of them at `cur_anim 0`
  (the idle) with `hold_attack 1`, where block 726 has `0/7` already on
  an attack slot. So whatever the second draw is, it is not spent on
  every attack-end wrap, and the dump's own field says which frames have
  it.
- **Both sides strike on 725**: `recharging` goes 1 → 30 on both, and
  no `recharging` row appears anywhere on the widening window.

The hypothesis is `Guy::inc_time`'s **queued attack**, the second call at
the shared `+0x271`:

```text
  set_anim(this, CHAR_DEFAULT, 0, 0)                 ; +0x1ed
  if (!is_hero(unit) && queued_attack != 0) {
      slot = queued_attack; queued_attack = 0
      if (slot < 2) { slot = CHAR_ATTACK2; roll = 1 }
      else          {                      roll = 0 }
      set_anim(this, slot, 0, roll)                  ; +0x271
  }
```

~~and the thing to re-derive before acting on it is the **roll**, because
the slots the dump lands on do not sit easily with the branch as written:
`0/8` goes to 12, which is the `slot < 2` arm, but `0/7` and `0/6` go to
**11**, which is the `slot >= 2` arm — and that arm passes `roll = 0`,
yet both of them spend the draw. Either `set_anim`'s attack-variation
roll at `+0xf2f` is not gated on that argument, or the slot is decided
somewhere this reading has not looked. [`sim::Sim::guy_inc_time`] marks
[`sim::anim::SITE_ATTACK_WRAP`] only on the `slot < 2` arm and hard-codes
`ATTACK2` there, so whichever it is, that is where it is wrong.~~ **Item
510 settled it: the branch is right, and both figures come off the
`slot < 2` arm.** `slot < 2` rewrites the slot to `CHAR_ATTACK2` *and*
sets `roll`, and the roll then picks `ATTACK1`/`ATTACK2`/`ATTACK3`; 11
and 12 are two outcomes of one branch, not evidence of two. What decides
the draw is not the slot but whether the swing was **queued** rather than
**held**, which is the facing — `docs/COMBAT.md` §45. The
successor is booked by the frame and the delta, not by this paragraph.

**§42.5's parked arrow is the leading hypothesis for what sits under it,
and it is a hypothesis.** *(Item 510 made it a measurement — §45.2 — by
reading the clock out of run118 and by moving the word to 743 with the
facing forced settled and nothing else changed. The paragraph below
stands as written because its reasoning was right; only its status
changed.)* From 696 this crate's three bowmen are on `1/7`
and the dump's are on `1/6` — our `1/7` was never wounded by the rolling
shot on 685, so §33's damage weight ranks the two hoplites the other way
— and every row at 720-728 below is downstream of that one target. What
makes it a hypothesis rather than the cause is that the *draw* is spent
in `Guy::inc_time`, and no run has shown the target choice reaching that
branch. 725 is therefore booked on its **own** frame and delta, not
parked under the arrow: this chain's record on "the obvious cause" is
eight wrong out of eight, and each of those eight also had a mechanism
that looked downstream of something already known.

The value diff, on the widening window (now `[606, 729)`, the ceiling
having followed the word):

| row | before | after |
|---|---|---|
| `recharging 0/6` 697, `0/7` 696, `0/8` 696 | three rows | **closed** |
| `order 0/6`, `0/7`, `0/8` | 684 | **696** (twelve blocks recovered) |
| `damage_frame 1/6` 728, `order 1/0` 722, `pos 1/0` 723 | — | **closed** |
| `unit_masks2 0/6` 696 | — | **closed** |
| `damage_frac 1/6` 712 | hidden by the old ceiling | stands |

The last row is the raised ceiling's and not the fix's, measured the way
item 481's was: with the arm switched off and the window at 729 it stands
identically. The same run is this landing's anti-vacuity for the new
`unit_masks2` row — with the arm off, `unit_masks2 0/6` at 696 is reported
and this crate's frozen-frame list is empty.

`chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame` now names
the **same** wounded set on both sides: `0/10` was the one the dump wounds
and this crate did not, and the frames the bowmen stop losing are what
reach it. `NEAR_TALLY` goes 5568/206 → 7368/296 with the agreement still
**total** — which is the direct refutation of the fear §43.3 recorded,
that the arm would part `near_o` on `0/7` and `0/8`: it does not, because
with the inherit landed the followers never run a search to write a
footprint with.

### 44.4 The instrument, twice more

Neither piece above is visible without a comparator fix, and both are
§43.3.1's rule again.

**A `TargetOrder`'s `(ox, whom)` is a slot number, not a liveness claim.**
`Built::unit_ids` gates its ad-hoc fallback on `alive()`, which is right
wherever the question is "which unit of the dump is this" — a dead figure
is in no `UNITDATA` block. It is wrong for an order's target, and with
`forget`'s clear removed this crate carried exactly what the dump carried
and the comparison still read `None`. `unit_ids_dead_or_alive` is the
target's reader; the three `order` rows move 684 → 696 on it alone. That
is the **third** appearance of the same hole in one function — item 462's
was a target not on the board at the start, item 496's an attack order
with no target, this one an attack order whose target is dead.

**`unit_masks2` was parsed and never compared**, which is the kind
`crate::diff::coverage` cannot catch: the pin records what the *parser*
asks for, and the parser has asked for this field all along. Only the
`0x10` bit is compared, and the reason is measured rather than assumed —
across every dump on this disk the word takes four values: `0` (5.5 M
unit-frames), `0x40000` (731, run16 and run16b, `process_supply`'s display
flag), `4` (688, the East Indies captures **including run99's own value
window**) and `0x10` (148). Comparing the whole word would pin a defect
this landing is not fixing, on the other headline's own capture. On both
headline windows the mask is the whole field: run112 carries `0x10` on
seven unit-frames and zero everywhere else, run100 zero throughout.

### 44.5 Coverage

**Diff-backed**: the whole of §44.3's table, both directions, on
`chapter_two_s_word_frame_is_widened_whole`'s `[606, 729)`; the word
itself; the seven frozen unit-frames of the whole chapter, both
directions, on `chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_
frame` — five of the seven reproduced, and the two that are not are past
the word, which the test asserts rather than assumes.

**Dump-backed**: §44.2's `new_ord`/`ever_in_range` split at block 696;
§44.1's `waiting 0` ruling out the throttle; §44.3's run118 clocks at
blocks 696 and 726; §44.4's histogram of `unit_masks2` over every capture
on disk.

**Listing-backed**: `LAB_005fdb9e`'s two writers and the search budget
above them; `UnitData::get_captain@00610ab0`'s recursion; the inherit
block's four conditions; `Leader::process@006b88b0`'s zeroing of the
budget. `unit_masks2` is `UnitData +0x6c` and `waiting` is `+0xad` from
the type record, not from the surrounding code — which is how §43.2's
`near_o 6 near_who 1` was caught: `+0xa2` is `cavarch_o`, and the dump's
`near_o` is `ObjectData +0x34`, a different field with a different writer.

**A guard, now earned rather than proposed.** §43.3.1's hole has now
appeared **three** times in `compare_orders` — item 462's `unit_ids` on a
target not on the board at `BEGIN GAME`, item 496's `target_ids` on an
attack order with no target, and this item's `unit_ids` on an attack
order whose target is dead. Three instances of one shape, each found by
widening and none by a reading, each hiding a live divergence for weeks,
is past a rule and into a check: `crate::diff::coverage` catches a key the
**parser** never asks for, and nothing catches a key the parser reads and
the comparison then drops for want of a value on this side. The grep is
`if let (Some(a), Some(b)) = (mine, theirs)` and its cousins — `zip`,
`and_then`, a `?` in a helper that feeds a comparison. Named here so the
count is on the record.

**Reading-only**: the search throttle (§43.5), unmodelled and unexercised;
`Unit::fight`'s `param_5 != 0` sub-call path, which writes
`cavarch_o`/`cavarch_who`/`cavarch_uid` instead of searching and which no
run on disk enters.

## 45. The word is the swing's facing, and the record it is spent in was never compared (item 510, 2026-09-22)

Chapter two's word stands at **725**: 7 draws against the original's 9,
parting at draw 1, the delta two `Guy::set_anim+0xf2f <
Guy::inc_time+0x271`. This item did not move it. What it did is make the
word's own frame *readable* — the widening had never opened a `GUY`
record — and then measure, in both directions, that the two draws are
§42.5's parked arrow wearing a third mechanism.

### 45.1 Held or queued: the two ways an attack survives an animation

`Guy::set_anim@005da300`'s attack block, from the listing, is two tests in
a fixed order:

```text
if (UnitAnimCat[param_1] == CHAR_ATTACK2 && !(guy_flags & 0x40)) {   ; an attack, not a plane
    v = param_3 ? 1 : param_1
    if (des_x != x - off_x || des_y != y - off_y || des_angle != angle) {
        hold_attack (+0x9e) = v ; return          ; (A) still walking or still turning
    }
    if (UnitAnimCat[cur_anim] == CHAR_ATTACK2) {
        queued_attack (+0xa0) = v ; return        ; (B) an attack is already playing
    }
}
```

**The deferral is tested first and the queue second**, which is what
decides the draw. `(A)` is paid on a later frame by `Guy::move`'s own
arms; `(B)` is paid inside `Guy::inc_time`'s wrap loop, immediately after
the `set_anim(CHAR_DEFAULT, 0, 0)` at `+0x1ed`:

```text
    slot = queued_attack ; queued_attack = 0
    if (slot < 2) { slot = CHAR_ATTACK2 ; roll = 1 }
    else          {                       roll = 0 }
    set_anim(this, slot, 0, roll)                      ; +0x271
```

and `roll` is `Guy::set_anim`'s third argument, so only `(B)` with a
stored `1` reaches the attack-variation draw at `+0xf2f`. So an
attack-end wrap costs **one** draw when the swing was held and **two**
when it was queued, and the branch that decides is the **facing**.

[`sim::Sim::guy_set_anim`] and [`sim::Sim::guy_inc_time`] already
implement both arms in this order, and the listing above is the check:
§44.3 flagged the in-loop arm as suspect because `0/7` and `0/6` come out
of the wrap on slot **11**, which is not `CHAR_ATTACK2`. That reading was
wrong, and the correction is worth stating because it is the shape this
chain keeps hitting: **11 is the roll's outcome, not the stored slot.**
`slot < 2` rewrites the slot to `CHAR_ATTACK2` *and* sets `roll`, and the
roll then picks `ATTACK1`/`ATTACK2`/`ATTACK3` at 30/40/30 (§6). `0/7`'s
11 and `0/8`'s 12 are two draws off one branch, not evidence of two.

### 45.2 Why the original queues on 725 and this crate holds

Both sides swing on 725 — `recharging` goes 1 → 30 on `0/7` and `0/8` in
the dump and here — and both figures are mid-attack when the swing asks,
so the only question is `des_angle != angle`. `Unit::fight` writes the
guy's `des_angle` to the attack angle two statements before it asks, so
the test is really *was this unit already facing its target*.

| | the original | this crate |
|---|---|---|
| target from 696 | `1/6` | `1/7` |
| target's position 690 → 726 | (1608, 8040), never moves | (1800, 8472) → (1656, 8472) by 700 |
| attack angle on 695 | 1344339968 | 1607204864 |
| attack angle on 725 | 1344339968 | **1632043008** |
| the swing lands in | `queued_attack` | `hold_attack` |
| the wrap spends | `+0x1ed` **and** `+0xf2f` | `+0x1ed` |

The original's bowmen last swung on 695 at a hoplite that has not moved
since, so the angle their facing was turned to then is still the angle
`Unit::fight` computes now, and the attack is queued. This crate's swing
on 695 aimed at `1/7` where it stood *on that frame*; `1/7` finished
walking five frames later, and the recharge is 30 frames, so the next
swing computes a different angle from a facing that is thirty frames
stale. The attack is held, the wrap pays once, and the figure comes out
of it on the idle.

**Measured in both directions, so it is not an argument.** Forcing the
facing settled on 725–727 alone — nothing else changed, no target moved,
no damage touched — takes the word from 725 to **743**, where the next
parting is a death draw (`Unit::close+0xcb6`, §42.1) the original spends
and this crate does not. That is the arrow again from the other end: the
hoplite the original has killed by 743 is the one it has been shooting
since 686.

So 725 is **not** an independent residue. ~~§42.5's rolled arrow is its
cause through three links, each of them now a measurement rather than a
reading: the arrow wounds `1/7` on 685 and this crate cannot land it
(§42.5); §33's damage weight therefore sends all three bowmen to `1/7`
where the dump sends them to `1/6` (`order 0/6`, `0/7`, `0/8` at 696);
and~~ The middle link was a reading: with the arrow landed, the bowmen
still took `1/7` until `compare_target` ranked at the real bearing (§46.5).
The third link stands: `1/7` walks where `1/6` stands, so the swing is
held where the original's is queued.

### 45.3 The record the word is spent in was compared nowhere

This is the finding with teeth, and it is the third instrument defect on
this window in four items.

**`crate::diff::compare` builds no `GUY` row at all**, and
`chapter_two_s_word_frame_is_widened_whole` built none of its own. The
one walk in this crate that compares a figure's animation clock is Great
Lakes' `run100_s_word_block_is_every_record_the_dump_carries`, which has
had the rows since item 400. Chapter two's word has been an *animation*
draw since 695 — eleven items — on a window that could not see
`cur_anim`, `cur_time`, `end_time`, `last_time`, `des_angle`, `gpiece` or
`stopped`.

**And the comparator capture is blind on all of them.**
`GuyData::log_data@005de6c0` announces four detail levels and stops a
`GUY` block after `ox` at the second; run112 was taken at `GUYS=2`
(`docs/RUNS.md` run112), so those keys **are not in the file**. A walk
that had built the rows would have compared `Some(ours)` against `None`
on every block and reported agreement. That is `CLAUDE.md`'s "a quiet
field is checked against the reader before it is called agreeing" with
the roles reversed — here the reader was willing and the *dump* was
silent — and it is a second way for a comparison to agree because it is
not looking.

The fix is a second capture. run118 is the same lobby, the same seed and
the same `chapter2.cmd` at `GUYS=4`, truncated at block 846, which
contains the whole of the widening's `[606, 729)`. The walk opens it
beside run112 and, for every guy of every unit of every block, asserts
that **every key both files print** — the figure's position and its angle
— is equal before reading a key only run118 has. A capture from a
different game cannot arrive quietly, and the count of records actually
taken from it (2694) is pinned so a reader that stopped finding blocks
fails rather than printing nothing.

`hold_attack` and `queued_attack` are new to the parser in this landing
and are deleted from `crate::diff::coverage`'s `UNREAD` pin on both
`UNITDATA/GUY` paths.

### 45.4 What the widening now says

Eleven rows became **thirty-nine**, and the twenty-eight new ones are
three families.

- **The word's own delta, as values.** `g.cur_anim`, `g.end_time`,
  `g.stopped` and `g.hold_attack` on `0/7` and `0/8` at **726** and on
  `0/6` at **727**: the original comes out of the attack-end wrap on
  `cur_anim 11`/`12`, `end_time 30`, `stopped 1`, `hold_attack 0`; this
  crate on `cur_anim 0`, `end_time 31`, `stopped 0`, `hold_attack 1`.
  Before this landing the word had a draw-stream parting and no value
  diff at all.
- **`g.des_angle` beside each `g.angle`**, on the same three figures and
  the same three blocks as the unit-level `angle` rows. It is §45.2's
  own field, one level down.
- **`g.gpiece` on ten pre-existing units at 606**, the window's floor —
  `0/1`–`0/5` and `1/1`–`1/5`, five types across both players. This
  crate's piece is **exactly [`sim::anim::PIECES_PER_AGE`] (0x840) below
  the dump's** on every one of them, which is one age bracket:
  `Sim::unit_gpiece`'s `(0..=bracket).rev()` walk settles an age lower
  than `GraphicPieces::get_unit_gpiece@0090c030` does for this game. It
  costs no draw on this window — `cur_anim`, `cur_time` and `end_time`
  agree on all ten for all 123 blocks, so the two pieces share their
  lengths here — and chapter two's nine staged figures are unaffected
  (`0/6`'s `gpiece 472` is the dump's). It is `docs/ANIM.md`'s and is
  named here only so it cannot hide again.

### 45.5 What is not established

- **Which of `bracket` or `piece_lengths` is wrong** in §45.4's third
  family. The difference is one age on ten units of two players, so it is
  the walk's input rather than a per-type accident, but this item read no
  further: it spends no draw on either headline window.
- **Whether `(A)`'s `des != pos` half ever fires here.** Every figure of
  this window is standing (`g.x`/`g.y` agree with `g.des_x`/`g.des_y`
  throughout), so the deferral in §45.1 is only ever taken on the angle.
  A walking unit's swing is untested by this capture.
- **`guy_flags & 4`'s step of two** (§43.5) stays unobserved: run118's
  bowmen are all `guy_flags 48`.

### 45.6 Coverage

**Diff-backed**: the whole of §45.4's map, both directions, on
`chapter_two_s_word_frame_is_widened_whole`'s `[606, 729)` — every field
of every `GUY` record of every unit on every block, the clock borrowed
from run118 under a same-game assert. The word itself and its 7-against-9
delta.

**Dump-backed**: §45.2's table — run118's blocks 694–697 and 724–727 for
the clocks and the `hold_attack`/`queued_attack` pair, run112's own
`UNITDATA` for the hoplites' positions from 690 to 726.

**Probe-backed**: §45.2's 725 → **743**, from a scratch run of the golden
chapter with the facing forced settled on 725–727 and nothing else
changed. It is the reason this item parks rather than queues a successor
of its own.

**Listing-backed**: §45.1's two orderings, both read from
`Guy::set_anim@005da300` and `Guy::inc_time@005d9e10` rather than from
the decompiler's control flow alone.

**Reading-only**: nothing new. §44.3's queued-attack paragraph was a
hypothesis and is struck there.

## 46. The rolled arrow comes down, and the target is ranked at its bearing (item 495, 2026-09-22)

Chapter two's word stood at **725**, and item 510 had measured its cause
from the far end: `damage 1/7` on 686 (ours 0, theirs 19) was §42.5's
rolled arrow, which this crate flew to its `3 × total_time` cap and
dropped. Forcing the bowmen's facing alone moved the word to 743. This
item lands the arrow, and the word goes **725 → 762**. It took four
things, and only the first was the one the item was booked for.

What would have killed the arrow reading, written before the build: *the
arrow lands and `damage 1/7` still parts on 686.* It did not survive
whole. The arrow landed, `damage 1/7` closed, and `damage_frac 1/7` stood
at 0 against 5. The frame was right and the mechanism was a chain.

### 46.1 The arc, operation for operation

`Ammo::inc_time@0067d380`'s rolling arm, from the listing
(`0x67d922`–`0x67da21`), is scalar SSE throughout:

```text
t     = (float)cur_time                  T = (float)total_time
frac  = t / T
z     = (v1z * t + (float)sz) + ((GRAV_Z * 0.5) * t) * t
x     = (int)((float)(ex - sx) * frac + (float)sx)      ; y likewise
if !WorldData::is_valid(x, y)            close()
if z > (float)find_data_z(x, y, 0)       return          ; still flying
ex, ey, ez = x, y, (int)z ; total_time = cur_time ; do_damage()
```

and `Ammo::init@0067bbf0` (`0x67cf2a`–`0x67cf7d`) fixes `v1z = ((float)(ez − sz) −
((GRAV_Z × 0.5) × T) × T) / T` once. `GRAV_Z` is `0xc127cccd` (−10.4875)
at `0xcab378`, written only by `GraphicPieces::init@008ffcc0` before the
first frame, so it is a pinned constant. Every SSE operation is correctly
rounded, so the arc is a pure function of integers and pinned bits, and
[`sim::single::Single`] reproduces it in integers: [`combat::arc_v1z`],
[`combat::arc_z`], [`combat::arc_point`]. That module already existed,
for the aircraft bank (§46.8 says how this item nearly wrote a second
one).

The operands, each read from its writer:

- **`sz`** is the launch height. For an animation release it is the
  figure's ground plus the node's `dz` (§22, run109's
  `run100_and_run109_agree_on_where_the_bow_hand_is_off_the_ground`). For
  `Object::fire_ammo@0064c8b0`'s unit arm it is the figure's `z + 100`,
  and for its building arm the object's `z + 250`.
- **`ez`** is `Ammo::init`'s `+0x20`: the target object's own `z`, **plus
  75** (`0x4b`) when flag 4 is set, the first figure's `z` for an air
  target, and floored at 0. `1/8`'s tile gives 220, and 220 + 75 is the
  295 run112 prints.

**Diff-backed, to the last printed digit.** [`combat::arc_v1z`] prints
as exactly the original's six decimals on all 183 of run109's records and
on all 134 live records of run112 (`run109_s_v1z_is_arc_v1z_to_the_last_digit`,
`chapter_two_s_v1z_is_arc_v1z_to_the_last_digit`). The three rolled shots
of run112 come down on the original's step, from the launch record alone
(`every_rolled_shot_comes_down_where_the_original_s_does`):

| shot | due | lands | point | arc `z` | ground |
|---|---|---|---|---|---|
| `0/8` from 677 | 7 | **9** | (1896, 8431) | 163 | 205 |
| `0/7` from 771 | 8 | 10 | (2043, 8504) | 140 | 197 |
| `0/6` from 769 | 9 | 11 | (2068, 8443) | 135 | 198 |

The last two stick in the ground, and their spent records print the
landing: `total_time` is the step, `ex ey` the point and `ez` is
`(int)z`. All four fields match.

### 46.2 The surface is `master_land_heights`, and a print names its single

~~§42.5: the surface is "a float height surface this crate does not
carry … a different quantity" from `master_land_heights`.~~ It is the
same array. `TerrainOut` is `TerrainData` behind a
0x40-byte prefix, so `find_data_z`'s `this+0x4a4` is `TerrainData+0x464`,
which is `master_land_heights.list`. `tools/ghidra/README.md` has recorded
that trap since the road search fell into it. `+0x4b7c` is
`tesselation_level` at `+0x4b3c`, and `rowsize` is `tesselation × xs +
1`, written by `Terrain::init@00850f70`.

`TerrainOut::find_data_z@00866560`, from the listing: the corner cell is
`div_3_table[x >> 6]` (a floor of `x / 192`), clamped to the grid. The
cell splits on its anti-diagonal: `fx > 192 − fy` takes the far triangle
based at `h10`, and otherwise the near one at `h00`. Each term is `(Δh ×
0x3baaaaab) × (float)offset`, with the constant being the single nearest
1/192. The sum is `((base + term) + 0.0) + term`, and `cvttss2si`
truncates it. The fourth argument is 0 at every caller this crate has
(`Ammo::inc_time`, `Guy::update_z@005d9950`, `Unit::update_z@00606590`),
so the water arm and the zero clamp never run. That is
[`sim::World::data_z`].

**The representation.** This crate keeps the grid in the millionths the
dump prints. [`sim::single::Single::from_millionths`] takes the single
nearest a printed value and says whether it is the only one that prints
that way. From |h| ≥ 16 up it always is, because a single's spacing there
exceeds a millionth. Of Great Lakes' 58,081 corners, **2,058 are not**,
every one under 16 in magnitude (the largest is 15.970737). There two to
four singles share a print, and the nearest is a choice at most a few ulp
off, under 2e-6 of a world unit.

**Diff-backed, on every point run112 carries.** A figure's `z` *is*
`find_data_z(x, y, 0)` for a unit not of sea domain. A unit's own `z` is
`find_tcoord_z` at its tile, and so is a building's. So
`every_object_s_z_is_the_ground_it_stands_on` holds [`sim::World::data_z`]
against **5,310** figure-blocks at 947 distinct points, and
[`sim::World::tile_z`] against 17,271 unit-blocks and the start dump's 13
buildings. Every one agrees. Ten of the figure reads touched an
ambiguous corner: `1/0` walking the shore at z 8–15 over 713–726. All
ten still agree, and the ten are pinned.

**An object's own `z` is clamped at zero** (item 1131). `find_tcoord_z
@008544a0` takes a fourth argument: 1 returns a negative height as 0, and a
tile on the map whose surface is ocean (`& 0x30 == 0x20`) is 0 either way.
Every writer of `ObjectData +0xc` passes 1 — `Unit::update_z` (`pushl $0x1`
at `00606598`), `Unit::set_new_location`, `Unit::come_out`, `SubObject::init`
and `SubObject::set_new_location` — and only the air physics' ground probes
pass 0. No dump on disk prints a negative `z_internal`. This crate handed
`get_damage` the raw [`sim::World::tile_z`], so a plane over run404's river
east of the Barracks, at −4 to −6, took the river step's ×2 (§6 step 16):
64 from the Radar where the original dealt 32. It is
[`sim::World::object_z`] now, for both sides of the damage and the ranking.

### 46.3 The guard

[`sim::Sim::ground_z`] counts a read that touched an ambiguous corner, or
one a terraform has *changed* in millionths since the grid was installed
(DECISIONS 31: the terraform is exact millionths, not the original's
singles), in [`sim::Sim::ground_inexact`]. The test-only `Drop` of
`rondata::diff::Built` holds every replay the harness runs to
`testkit::GROUND_INEXACT` for its test's name, which defaults to **zero**,
and fails the test by that name on an excess. The whole suite passes with
the table empty. Installing a grid clears the marks, because setup
terraforms and then reinstalls the dump's own grid, and marks kept across
that turned 1,911 of §46.2's sweep reads into false alarms on its first
run.

### 46.4 The pool is stepped in slot order

The arrow landed and `damage 1/7` came out **19 + 10/16** against the
dump's **19 + 5**, from the wrong bowman. `0/7` and `0/8` both fire on
677, both are due on 683, and whichever is stepped first kills `1/8`
while the other finds it dead and rolls. In the original, `0/7` holds
slot 1 and `0/8` slot 2. `Objects::inc_time@0065db70` walks the pool
`0..count`, and `Objects::add_ammo@00658b10` takes the lowest free slot
(§24.4). This crate `swap_remove`d, so `0/6`'s landing on 681 put `0/8`
first. [`sim::Sim::add_ammo`] now assigns the slot and keeps the list
sorted, and a landing removes without reordering. That is the "slot pool
in `crates/sim`" §24.4 named as the successor.

### 46.5 `compare_target` ranks at the real bearing, and the height is live

With the right bowman, `damage_frac 1/7` still stood at 0 against 5, and
it was 57 against the original's 58. `get_damage`'s step 23, the height
bonus, compares the attacker's `z` (`+0xc`) with the target's, and this
crate passed `z: 0` on both sides. §46.2's sweep shows both are
`find_tcoord_z`, so both are now [`sim::World::tile_z`]. `0/8` stands at
253 and `1/7` at 211, and `42 × 10 × 57 / 20000` is the one point.

Then the bowmen still retargeted to `1/7` on 696 where the original
takes `1/6`. The wound was not what decided it. ~~§42.4 and §45.2: a
wounded `1/7` "outranks `1/6` on §33's damage weight".~~ With `damage
1/7` agreeing, the ranking still parted. `Object::compare_target`'s
`get_damage` call is printed by the decompiler as `find_angle(0, 0)`
(§18.1's and §33.2's reading), but `find_angle@0092d130` takes its pair
in `ecx`/`edx`, and the listing loads them at `0064ebe7`–`0064ec00` as
the target's position less the attacker's. The angle is the real
bearing, so the flank sector step 19 reads separates two hoplites at
different bearings. With it the captain `0/6` scores `1/6` at 27,307
against `1/7` at 24,967 (it had been 17,627 against 35,840) and takes
`1/6`, as the dump does.

### 46.6 What moved

| | before | after |
|---|---|---|
| golden chapter two, **word** / sequence | 725 | **762** |
| values | 726 | 763 |
| the widening's window | `[606, 729)` | `[606, 766)` |
| widening rows | 39 | **20** |
| frozen frames missed | `0/9` 745, `1/7` 762 | none |
| `near_o` unit-frames / live / agreeing | 7368 / 296 / 7368 | 9530 / 409 / 9530 |
| chapter one, Great Lakes, East Indies | 626, 10834, 9711 | unchanged |

**The value diff on the frame it moved.** On 686 every record agrees:
`damage 1/7 19`, `damage_frac 5`, `damage_frame 685`. On 696 the three
bowmen hold `ox 6 whom 1` on both sides. The window gains two deaths,
`0/11` on 729 and `1/6` on 743, and both are the original's own, on its
frames (420 death comparisons, all agreeing).

The widening at 762 is on file (`chapter_two_s_word_frame_is_widened_whole`,
ceiling 766). **Nothing parts on 762 or 763.** The draw delta is 8
against 9: the original spends a `Unit::fight+0x9b0` first. The first
values part on **764**, `g.cur_anim 0/10` (ours 13, theirs 12), and on
**765**, `1/7`, the original's last hoplite, whose order becomes kind 2
with a 53-node path to (38646, 13305) where this crate's stays kind 1.
`pos`, `g.x`, the angles and the speeds follow it. No mechanism is named
here. Standing below: `damage 1/6` at 680 (§41.3) and the ten `g.gpiece`
rows at 606 (§45.4, parked). `0/9`'s wound, first on block 792 in the
dump, is past the word, and this crate no longer reaches it.

### 46.7 What is not established

- **The 2,058 ambiguous corners.** Each is the nearest single to its
  print, at most a few ulp and under 2e-6 of a world unit from the
  original's. A read lands on one only near water. The guard is zero on
  every replayed window, and ten sweep reads that agree. A capture that
  reads `master_land_heights.list` as raw bits (an in-process memory read
  of `TerrainData+0x464` at frame 0) would settle all 2,058. Not booked:
  it answers no score today.
- **A terraformed corner.** `crate::terrain` works in exact millionths
  where the original works in singles (DECISIONS 31), so a corner a
  building has changed is not known to be the original's single. The
  guard counts any read of one, and none happens on a replayed window.
- **The spent arrow.** `Ammo::do_damage@00678060`'s puncture arm leaves a
  shot whose piece can stick, on valid non-water ground, as `flags 1`,
  with `cur_time` reset. `inc_time` holds it for 200 frames, **in its
  slot**. This crate removes it, so a later shot can take a slot the
  original's cannot. run112 carries 239 such records, the first on 780,
  past the word. It is parked rather than landed.
- **`sz` for a piece with a release table and no measured node** takes a
  `dz` of 0, and `ez` for a building target is its tile's `z`. Neither is
  reached by a rolled shot on any capture.
- **Negative coordinates** into `find_data_z` index `div_3_table` below
  0 in the original. Every caller here is gated by `is_valid` first, and
  this crate clamps.

### 46.8 Coverage

**Diff-backed**: the arc (`v1z` on 317 records, three landings with
step, point and `ez`), the surface and both heights (§46.2's sweep), the
height bonus and the slot order (`damage 1/7`/`damage_frac` on 686), the
bearing (`order`/`angle` 696 closed), and the word 725 → 762 with its
widening.

**Listing-backed**: §46.1's arm and `v1z`, §46.2's `find_data_z`, the
fourth argument 0 at three callers, `find_angle`'s register pair and
`compare_target`'s load of it, and `GRAV_Z`'s single writer.

**Reading-only**: the unit and building arms of `sz` (`+100`, `+250`),
the air arm of `ez`, and the spent arrow's 200-frame hold.

One lesson for the next reader. Grep the crate for the *type*, not only
for the name the brief used. This item first wrote a software single
from scratch over `crates/sim/src/single.rs`, which already had every
operation, and caught it only on the module list.

## 47. The captain the roll reads, and the shot's own bearing (item 523, 2026-09-22)

Chapter two's word stood at **762**. There this crate spent 8 draws
against the original's 9, and the one it missed was a `Unit::fight+0x9b0`.
Nothing parted on 762 or 763, and no mechanism was named. This item
moves the word **762 → 900**, which is the end of run112's trace. No draw
parts on any frame of chapter two and no word parts (`values` is `None`).
It took three things, and the widening found each one on the frame the
last had left.

### 47.1 The draw's site, and every input to its branch

`Unit::fight@005fd4d0+0x9b0` is `0x5fde80`, the return from
`Random::get@00a39d70` at `0x5fde7b`. From the listing, the branch that
reaches it is `fight`'s captain arm:

```text
if order.mandatory == 0 && param_4 == 0 && recharging == 0     ; +0x1c, arg, +0xae
  && is_captain()                                               ; o_up < 0
  && (cavarch || !order.is_group() || order.group_leader == me) ; vtable +0x2c, +0x94
  && target.is_unit()                                           ; vtable +0x18
    roll = Random::get(0, 0xffff)                               ; the draw
```

The draw is unconditional once the branch is entered. The two
suppressions (`roll % 5 == 0`, the order's `flags & 0x10`) are read after
it. Against the dump, input by input:

| input | printed? | on 762 |
|---|---|---|
| `mandatory` | `ATTACKORDER mandatory` | 0 on both |
| `param_4` | a call argument, always 0 from `Unit::do_attack` | n/a |
| `recharging` | `UNITDATA recharging` | compared since item 485 |
| `is_captain` | `UNITDATA o_up` | **printed, and compared nowhere** |
| the group test | not printed | `1/7`'s order is a fresh `add_attack_order`, not a group order |
| `target.is_unit` | `ox`/`whom` | `0/10`, a unit |

Vtable slot `+0x18` is `Buffer::is_pending_load` after COMDAT folding: a
`return 1` that answers "is a unit". This crate's gate read
`combat.captain == index`, a cached copy of the captain. For run112's
`1/7` on 762 it held **6**, the index of `1/6`, which died on 743.

### 47.2 A dying figure is relinked, and a dying head hands its squad down

`Unit::close@0060ee50`, from the listing `0060f28e`–`0060f758` (the
decompiler prints two of these shorts as floats). It sits above the death
draw's `dtype` gate and under nothing but the slot's `flags & 1`, so it runs
on every death:

```text
if o_up >= 0:    above.o_down = this.o_down            ; and edi = 0
if o_down >= 0:  below.o_up   = this.o_up              ; a head's −1
                 if below.flags & 1: goto append
if edi != 0:     the whole-squad arm                   ; nobody left to lead
append:  tail = walk o_down from this.o_down, else this
         tail != this:  tail.o_down = this ; this.o_up = tail
         tail == this:  above.o_down = this            ; undoes the first line
         this.o_down = −1
```

The dead figure goes to the tail of its chain, where
`Unit::repair_damage@0060de10` looks for a slot to regrow a figure into
(`find_free(…, o_down)`, and its "negative o_down" error string). So a
dying tail changes nothing its captain can see, and a dying **head**
promotes the figure below it. run112 prints both:

- `0/11` dies on 729, and `0/10` still carries `o_down 11` on every block
  after.
- `1/6` dies on 743, and block 744 has `1/7` with `o_up -1 o_down 8`, a
  chain of 7 → 8 → 6.

[`sim::Sim::relink_squad`] is that relink, called from both death paths
(`take_damage`'s and attrition's). This crate keeps the captain in two
caches, [`sim::Unit::captain`] and [`sim::combat::State::captain`], and
the original reads both live (`UnitData::is_captain@0046ceb0` is `o_up <
0`, and `UnitData::get_captain@00610ab0` walks `o_up`). So a promotion
re-points both down the whole chain, dead slots included.

**Diff-backed.** `o_up` and `o_down` are compared per frame now (§47.5).
Across the whole of run112's window every figure's links agree. With the
relink skipped, the row parts on block 744, `o_up 1/7` ours 6 theirs −1,
which was made to fail once before landing. On 762 `1/7` spends the roll.

### 47.3 A landed shot strikes at its own bearing

With the draw closed, the next parting was `damage 1/7` on 771: ours 22 +
15/16, theirs 24 + 10/16. That is one shot from `0/9`, fired on 766, due
on 770, with the same `damage_o` on both sides. Everything else agreed.
The original's hit was 85/16 and this crate's 58/16. That is flank level
2 against 1, and the bearing is what differed.

~~§9.3: `Object::do_damage(A, ox, whom, find_angle(launch → target), …)`.~~
`Ammo::do_damage@00678060`'s no-splash arm loads `ecx = ex − sx`, `edx =
ey − sy` at `00678c2e`–`00678c37` and calls `find_angle@0092d130`
fastcall. The splash arm does the same at `006786a5`–`006786cd`, once for
every object it walks (its midpoint is the cell centre, not the bearing).
The decompiler dropped the pair and printed the two pushes behind it,
`num_guys` (`+0x44`) and `index` (`+0x34`), as `find_angle`'s arguments.
This crate took the target's position at landing. `1/7` walked about a
hundred units while the shot flew, so the flank sector moved.
[`sim::combat::Projectile::bearing`] is the fix, on both arms.

### 47.4 The lead is the first figure's `avg_speed`

The bearing alone did not close 771, because this crate's shot also came
down elsewhere. Ours was (1772, 8386) and the dump's `ex ey` was (1717,
8421), from the same launch and the same scatter draws.
~~§9.1: the lead is "the target's heading and per-frame speed".~~
`Ammo::init@0067bbf0` at `0067ceb4`–`0067cec7` loads `ecx = T.angle`
(`UnitData +0x50`) and `edx = guys.list[0]->avg_speed` (`UnitData +0xf4`,
then `GuyData +0x84`), and calls `cosx@0092d0c0` and `sinx@0092d100` on that
pair. `1/7` had just set off, so its figure's average stood at **9**
(run118, block 766: 5, 9, 12 over three frames) where the unit walked 24.
The lead is `sinx(angle, 9) × 5`, and it puts the landing on the dump's.

`tools/ghidra/README.md` now has this trap in its list. It is the third
dropped register pair in two items (495's `find_angle(0, 0)` was the first).

### 47.5 The instrument: a stand-up read is not a comparison

`o_up` had been parsed since the army reader was written, for one
purpose: seeding [`sim::Unit::captain`] at stand-up (`diff::army`). That
one read kept it off `coverage`'s unread pin, and the ledger counts any
appearance as "named". No per-frame comparison ever read it. `1/7`'s
promotion parted on 744, nineteen frames before the draw it cost, and
nothing looked. `o_up` and `o_down` are rows beside the firing record now
(`rondata::diff::harness`), keyed by field, with a dead slot compared as
the `o` the dump prints.

The second gap is of the same kind. run112's 373 `AMMO` records feed the
`v1z` and rolled-landing tests, and none of them is compared against this
crate's own shots. `diff::ammo`'s whole-record comparison runs on run109
alone. The shot's `ex ey` parted on 767 and was found only by printing it.
Parked for the commander.

### 47.6 What moved

| | before | after |
|---|---|---|
| golden chapter two, **word** / sequence | 762 | **900** (the trace's end) |
| values | 763 | none |
| the widening's window | `[606, 766)` | `[606, 901)`, the whole capture |
| widening rows | 20 | **11** (the two standing residues) |
| deaths in the window | 3 | 4 (`1/7` on 816) |
| wounded sets | ours lacks `0/9` | identical |
| `near_o` unit-frames / live / agreeing | 9530 / 409 / 9530 | 17219 / 859 / 17113 |

**The value diff on the frames it moved.** On 744 `1/7` carries `o_up -1
o_down 8` on both sides. On 762 its order is `ox 10 whom 0 new_ord 1` and
it spends the roll. On 771 `damage 1/7` is 24 + 10/16 on both sides, and on
816 both `1/7`s die, carried from 817 as the dump's fourth `DEATH_OBJS`.

What stands is older than this window's words and spends no draw:
`damage 1/6` at 680 (§41.3) and ~~the ten `g.gpiece` rows at 606 (parked,
`docs/ANIM.md`)~~ (closed by item 1281: the age's `update_gpiece`,
`docs/TECH.md`, "The piece moves with the age"). One new value row appears past the last draw either
side differs on. On 846 the bowmen `0/7` and `0/8` drop their attack on
the dead `1/7`. The original's `near_o 7 near_who 1` stands through the
drop, and this crate's goes to −1 (`NEAR_PARTED`, from 847). `waiting` is
0 on both sides, so it is not the search throttle, and no mechanism is
named. Parked.

### 47.7 What is not established

- ~~**The lead's gate.** The original leads a target whose order `is_move`
  or `is_air`. This crate still asks `movement.dest.is_some()`, which is
  not the same predicate. Every lead on run112 agrees under it.~~ Built,
  with the angle read from `+0x50` (`heading`): §74.
- **The splash arm's bearing** follows the same listing and is not
  reached by any capture on disk.
- **The whole-squad arm** of the relink (a dying head with nobody live
  below) is taken from the listing's branch and does nothing in this
  crate beyond leaving the links. What else that arm does (the counters
  at `+0x2f0`, the heroes) is not read here.
- **A dead slot's `captain` flag.** A dead head's successor slot, when it
  is dead too, gets `o_up −1` in the original and keeps `captain = false`
  here. No live reader asks.
- **The group test** in §47.1 is read, not diffed. No order on run112 is
  a group order when the roll is reached.

### 47.8 Coverage

**Diff-backed**: the relink and promotion (`o_up`/`o_down` on every
unit-frame of run112's window, and the 744 row made to fail), the roll on
762, the bearing and the lead (`damage 1/7` on 771 and every hit after,
the fourth death on 816), and the word 762 → 900 with its widening.

**Listing-backed**: §47.1's branch, §47.2's relink, both
`find_angle(ex − sx, ey − sy)` sites, and the lead's `angle`/`avg_speed`
pair.

**Reading-only**: the whole-squad arm and the group test. ~~The lead's
`is_move`/`is_air` gate~~ is diff-backed since §74.

## 48. A bump from another enemy ends the chase, and chapter one is widened (item 445, 2026-09-22)

Golden chapter one's word stood at **626** from item 405 on, and no
widening was on file. On 626 the original spent `Guy::set_anim+0xf2f <
Guy::move+0x166` and this crate did not. The booked hypothesis was an
arriving hoplite's roll. This item widens the word's frame whole and
moves the word **626 → 774**.

### 48.1 The widening

`rondata::diff::golden::chapter_one_s_word_frame_is_widened_whole`,
window `testkit::WIDENING_CHAPTER_ONE`. It is `compare` plus every row of
the `UNITDATA` and `GUY` records that `compare` does not read or reads
only where the positions agree: `near_o`/`near_who` read raw off the
block, `orders_x/y`, `dest_angle`, `tolerance`, `idle`, `stance`, `form`,
`group`, the collision block ungated, and the figure's whole clock.

**Two captures, one game.** run105 (`g4`) prints `GUYS=2`, which stops a
`GUY` block after `ox`, so its clock fields are absent. run110 (`g6`) is the
same script at `GUYS=9` over `[610, 630)` (`docs/RUNS.md` run110) and
supplies them. On every block both files print, their shared `GUY` keys are
asserted equal. The `theirs` side was printed once, raw, before any quiet
row was believed (§48.2's table is that print).

Before the fix, the window `[605, 630)` held 87 keys. **Nothing parted on
626 that had not already parted on 625.**

### 48.2 The frame: `0/8` on tick 624

Attributing the draws per unit (`RON_GOLDEN_SITES=620-627`): the two
arriving hoplites `1/7` and `1/8` spend their `Guy::move+0x166` rolls on
**627**, on both sides. The extra 626 roll is `0/8`'s, and it is the
attack animation (`cur_anim 13` on block 627). `0/8` had already parted on
block 625, the state after tick 624:

| block 625, `0/8` | dump | this crate before |
|---|---|---|
| pos | `1044, 8076` | `1032, 8088` |
| `orders_x, orders_y` | `1044, 8076` (its own cell) | `1224, 8280` |
| order list | `ATTACK` on `1/6` | `MOVE_TO`, `ATTACK` |
| path stack | empty | four entries, three `SIDESTEP` |
| `collide`, `collide_o`, `collide_who`, `collide_guy` | `0, 8, 1, 0` | `1, 8, 1, 0` |
| `collide_frame` | `-1` | 624 |

On tick 624 both sides spend `Guy::set_anim+0x97a < Unit::move_step+0x823`:
a hard collision with `1/8`, and the blocked step's idle roll
(`docs/COLLISION.md` §5). After that the original writes the collider's
name and nothing else. It does not count, does not stamp `collide_frame`,
does not sidestep or repath. It drops the move and keeps the attack. On
tick 625 it swings (`recharging 32`, `in_range 1`), and on 626 it rolls the
attack animation.

### 48.3 The mechanism: `resolve_unit_collision`'s enemy ladder

`Unit::resolve_unit_collision@005f9d30`, lines 96–254, which
`docs/COLLISION.md` §6 lists as step 3 and which no capture had reached
before this one. After step 0 (the animal) and step 1 (a suicide attacker,
type `+0x2b4 & 0x2000`, not modelled), the function calls `update_action`
on every path (`:96`), then branches on `collide_who` against the owner
byte (`:98`):

- **Same player** → step 2's flat-farm test (`LAB_005fa057`).
- **Another player, and the action is not an attack** (vslot `+0x18`,
  `is_attack`) → the same test. This crate had gated step 2 on the
  collider being the same player's. An attack is never a gather, so the
  collider's owner is not part of step 2's predicate at all.
- **Another player, and the action is an attack** → the ladder:
  - **A** (`:162-174`): the collider is the attack's own target
    (`update_target_order` `ox`/`whom` against `collide_o`/`collide_who`).
    For a group order (vslot `+0x2c`), set `collide = 1` and return.
    Otherwise set `collide_o = collide_who = -1`, `kill_current_order`,
    return.
  - **B** (`:176-187`): `get_target_order`'s `target_exists` (slot `+4`:
    `ox`, `whom` non-negative, the object `flags & 1`, `uid` matching) and
    `ObjectData::is_in_range@00648d70(ox, whom)` from where the unit
    stands → `kill_current_order`, return. **The listing was read** at
    `005f9eb2`–`005f9ed3`. It pushes `0`, a junk `ecx`, `+0xc` and `+8` of
    `get_target_order`'s answer, and the wrapper passes `this`'s own
    position and a zero sixth argument, so the margin arm (§35.3) is off.
    The decompile's argument list is right here.
  - **C** (`:189-252`): when the attack is not `mandatory` (`+0x1c`), the
    current order is not a group's, and the collider's player is an enemy.
    A follower whose captain's action is an attack with a target in range
    repaths, kills, and queues that target `QUEUE_FIRST`. A captain calls
    `find_new_target(0, 1)` while `LeaderData +0x9f4 < 10`. Otherwise,
    unless type virtual `+0x10c` answers, it repaths, kills, and queues the
    collider. ~~**Not modelled**: `+0x9f4` and `+0x10c` are unnamed, and a
    unit that would take this arm falls through to step 4 as before.~~
    **Built by item 668**: `+0x9f4` is `LeaderData::retargets` and
    `+0x10c` is `is_siege`, and run171's `1/6` reaches the captain's
    arm on 658 (`docs/COLLISION.md` §14).
  - Anything the ladder does not return from falls through to step 4
    (`:255`), never to step 2.

`0/8`'s attack names `1/6`, the collider is `1/8`, and `1/6` is in reach
(HOPLITES `0xf6`, §13.2), so arm B fires. This is the other half of item
405's `do_move` gate (`docs/ORDERS.md` §4.4). A melee type never abandons
a leg because its target came into reach, but the first bump from any
enemy ends it.

`Sim::resolve_unit_collision` (`crates/sim/src/collide.rs`) now takes
`update_action` at the top, arms A and B, and step 2 without the
same-player gate.

### 48.4 What it moved

- **Golden chapter one 626 → 774.** On block 625 `0/8` is the dump's,
  field for field, and every row of §48.2's table closes, along with its
  626–627 clock, `recharging`, `visible` and `damage 1/6` on 626.
- The widening's window moved with the word, `[605, 630)` → `[605, 779)`,
  and its map is 88 keys. The families are written beside the pin in the
  test. Everything under 765 is older than 616 or spends no draw.
- ~~**What stands at 774** is `Guy::set_anim+0x97a`, a blocked step's idle
  roll the original spends and this crate does not.~~ **It was not a
  blocked step**: it was `Unit::fight`'s reloading stand, the fourth of
  four links, and the word is 900 (`docs/ORDERS.md` §22). The first record
  parting under it is **765**: `1/7` and `1/8` take the far walk to about
  (38.6k, 13.4k). That is the march destination item 399's table in
  `GOLDEN_WORD_CHAPTER_ONE`'s comment records who=1's army buying early. The
  original's order is `ATTACK_TO` with `stance 1` and this crate's is
  `GROUP_ATTACK_TO` with `stance 0`. Their captain `1/6` is dead by 764.
  No mechanism named.
- **Item 1281** took the map's ten `g.gpiece` rows at 610 and three
  citizens' `g.end_time` at 617–623 off it: the chapter's third age
  re-pieces every figure one bracket up (`docs/TECH.md`, "The piece moves
  with the age").

### 48.5 Coverage

- **Diff-backed**: arm B, by run105/run110's `0/8` on block 625 and
  onward. The item made it fail by construction: the widening's first run,
  without the arm, is §48.2's table.
- **Reading only**: arm A (unit test
  `a_bump_from_the_target_itself_drops_the_chase_and_forgets_the_collider`),
  the group arm's `collide = 1`, and step 2's foreign-collider reach. No
  capture on disk is known to reach any of them.
- **Not modelled**: ~~arm C, and~~ step 1. Arm C is built
  (`docs/COLLISION.md` §14).

### 48.6 What is not established

- Whether `update_action`'s `orders_x/y` and `dest_angle` writes at `:96`
  move anything on the long captures. It now runs on every resolve that
  reaches it, as in the original.
- ~~Arm C's two unnamed predicates, `LeaderData +0x9f4` and type vslot
  `+0x10c`.~~ `retargets` and `is_siege` (`docs/COLLISION.md` §14.1).
- ~~The five `dest_angle` rows under the word (618–652). They spend no draw
  in the window and are not examined.~~ **Closed by item 530**: they were
  `Unit::work`'s unconditional `update_action` (`docs/ORDERS.md` §22.2).

## 49. A ship attacks broadside (item 535, 2026-09-22)

Golden chapter five, run127, is the first capture with ships in it: two
triremes and a fishing boat on sea region 70 (`docs/GOLDEN.md` §9). Its
first walk parted at **617**, the frame after who=1's trireme `1/6` is
born. The original spent 8 draws and this crate 6.

### 49.1 The frame, and the record it was spent in

The original's extra draw leads the frame: `Guy::set_anim+0xf2f <
Guy::move+0x166`. Its other extra, a trailing `Farms::inc_time+0x1de`, is
the value shift one draw causes, and this crate spends it on 618. The
widening (`chapter_five_s_word_frame_is_widened_whole`) printed both sides
of block 617 for the three hulls. Both hold `1/6` on an attack order
against `0/6` from the same seat, (12408, 35832). What differs is the
angle:

| | unit angle | heading | figure angle |
|---|---|---|---|
| original, block 617 | `671481856` | `671481856` | `671481856` |
| this crate, block 617 | `596523349` | `−402259968` | `596523349` |

`−402259968` is the true bearing to `0/6` at (11640, 34680): dx −768,
dy −1152, −33.7°. The original's `671481856` is that bearing plus exactly
`0x40000000`. The hull turned side-on to its target, and it got there in
one frame. So its figure had arrived on 617, and `Guy::move`'s arrival arm
spent the draw. This crate's figure was turning its bow toward the
target at 70° a frame and had not arrived.

### 49.2 The rule

`Unit::fight@005fd4d0:698–714`, after the bearing `param_4` is computed:

- **When** the attacker's `unit_flags & 0x40` is set. That is the
  `FLAGS` letter `g`, whose legend in `unitrules.xml` is "Unit attacks
  sideways (most ships)". The exemption is an attacker whose own
  `TypeIndex` is `0x185` (`PATROLBOAT`, written into `local_20` from
  `this->ptype + 4` at `:609`) and whose target's type has `domain == 1`
  (`ObjectTypeData +0x218`, sea).
- **The two sides** are `param_4 − 0x40000000` and `param_4 + 0x40000000`.
  Each side's distance from the unit's current angle (`+0x50`) is the
  unsigned difference, folded by `~` when it is past `0x80000000`. The
  minus side is taken only when it is **strictly** nearer.
- **What takes the offset angle**: `set_angle` when it differs from
  `+0x50` (`:724`), every figure's `des_angle` (`+0x64`), and the melee
  `do_damage` calls (`uVar27`). `Object::fire_ammo` takes no angle. The
  rocking arm (`z`, `:831`) compares the *direct* bearing against it.

From run127's 120° heading, the plus side (56.3°) is 63.7° away and the
minus side is 116.3° away, so the plus side it is. `0/6` turns the same
way on block 635, from the other end: bearing 146.3°, plus side 56.3°,
also `671481856`.

`Sim::attack_angle` in `crates/sim/src/fight.rs` is the rule. `uflags::SIDEWAYS` is the
bit.

### 49.3 What moved

The word went **617 → 621**. On block 617 every `1/6` angle row closed,
four of them. Nothing under the word parts now except the standing
families and the fisher's birth-block orders (§49.5). The widening pins
that shape, and it was made to fail once with the rule reverted: the four
rows came back on 617 and 618.

On 621 the original spends `Ammo::init+0xcd9` and `+0xd0b`, the landing
scatter (§9.1). who=1's first round is in the original's air on block
622, launched from (12445, 35807), a point off the hull. This crate
launches it one frame later from the hull's own square, (12408, 35832),
and it lands at (11560, 34771) against the original's (11642, 34749).
That is the next item's frame. Its draw delta is those two draws, 8
against 6. **Item 542 closed it** (§50): the release frame, the keel
node, and a sea figure's `z`. The word went to 664.

### 49.4 Coverage

- **Diff-backed**: the plus side, on `1/6` block 617 by run127, through
  the widening and the golden word.
- **Reading only**: the minus side's strict comparison, and the
  PATROLBOAT exemption. The unit test
  `a_sideways_ship_attacks_broadside_on_the_nearer_side` covers both
  from this reading. ~~`0/6`'s turn on 635 fits the rule, but this crate's
  `0/6` never gets there in the window, so it is not diff-backed yet.~~
  Since item 542 it is: the word is 664, and `0/6`'s broadside turn on
  635 agrees on every angle row (§50.4).

### 49.5 What is not established

- ~~The fisher `0/7`. On its birth block, 621, the original gives it two
  orders, a `CASTORDER` first (order index 14, `uid 65535`), and walks it
  about 130 units from 622. This crate gives it none. It spends no draw
  under the word. Which spell the Fishermen cast at birth is not read.~~
  Settled by item 543 (`docs/ORDERS.md` §23): the Fishermen's unpack
  `0x292`, issued by `Unit::think`'s human rare-collector arm.
- ~~The trireme's launch. The release point off the hull, the frame, and
  `AMMO_PER_ATT 3`, a three-round volley whose second round is on block
  626.~~ Settled by item 542 (§50): three `<RELEASEEVENT>`s on frames 5, 9
  and 17, from node 0 on the keel.
- `Object::fire_ammo`'s near-face aim at a building still reads the
  direct bearing here. No capture has a ship shooting a building.

## 50. The trireme's first round: a divisor, a keel and the water (item 542, 2026-09-22)

Golden chapter five's word stood at **621** (§49.3). There the original
spends `Ammo::init+0xcd9` and `+0xd0b`, the landing scatter's two draws
at `0x67c8c9` and `0x67c8fb`: `Random::rand(0, 0xffff) % accuracy`, once
per axis, with `accuracy` at `ebp−0x14`. Those two sites are on every
shot's path, so the draw names only *that* a round launched on 621 in
the original. It names no branch a ship takes and a bowman does not. The
item booked two symptoms, "a frame early" and "a release point off the
hull", and no mechanism. It turned out to be three causes, and the draw
named only the first.

**What would kill each reading**, taken as they were tested:

- *The frame is a timing predicate on the fight* (reload, facing, the
  turn of §49). Killed if the original's `recharging` is set on the same
  block the round launches. It is not: `1/6` prints `recharging 36` on
  block 621 and the round is first in the air on 622. Reload and launch
  are separate steps, and the launch is the animation's (§9.0).
- *The frame is the release event's time.* Killed if the three rounds of
  a volley are spaced as `× 3 / 200` puts them (6, 9 and 18: three and
  twelve apart). They are four and twelve apart on every volley in the
  capture.
- *The point is a stored offset or arithmetic on the bearing.* Killed if
  the three rounds of one volley leave from one point. They leave from
  three different points, the same three on every volley.

### 50.1 The release frame is `starttime / 67`

The event list `GraphicEvents::execute_game_events@008e48e0` walks is
built by `GraphicEvents::init_unit_events@008e2520`, not by
`GraphicPieces::init_unit_events@008eabb0`. The latter keeps the
milliseconds for the renderer. For each `releaseevent` element
(`internal_strings` 3578; its `starttime` is 3579, `type` 395 and `node`
3580), the loader writes `event_type` 1 and reads `starttime` into
`GraphicEvent +0xc`. Then, at `008e296d`–`008e299f`:

```text
movzx ecx, word [ebx+0xc] ; imul 0x7a44c6b ; sar edx, 1 ; +sign  → ms / 67
if (ushort)result < 1: result = 1
[ebx+0xc] = [ebx+0xe] = result          ; start_time and end_time
```

`execute_game_events`' type-1 gate compares `start_time` with the
package's `last_time`/`cur_time` in frames (§9.0), so this is the frame.
~~§9.0: `starttime × 3 / 200`, truncated.~~ That reading was measured on
four run53 shots and agreed with every release run109 and run112 later
measured. It had to: all eleven of those `starttime`s give the same frame
under either divisor. Of the install's 2,593 `<RELEASEEVENT>` rows, 356
separate the two readings. The Trireme's `400` and `1200` are the first a
capture has reached, at 5 and 17 against 6 and 18.

**Diff-backed** by run127: every volley of both ships over the capture
launches on swing frames 5, 9 and 17. The first seven rounds, 622 to 662,
now launch on the original's blocks. `rondata::artdata::release_frame` is
the rule, and `a_release_frame_is_starttime_over_sixty_seven` pins it.

### 50.2 The point is node 0, walking the keel

The Trireme's three events all name `node="0"`, and node 0 moves as the
`Trireme Attack1` animation plays. `GraphicPieces::get_position` is the
same per-(piece, node, anim, starttime) vector §22 measured for the
Longbowman. It is art the original builds before the first frame, so
this crate measures it and does not load it (§22.3, DECISIONS 16's
second shape). From run127's `AMMO` records, less the hull's own
position:

| frame | offset | `sz` | as `(bearing, radius)` |
|---|---|---|---|
| 5 | (+37, −25) | 88 | (0, 45): dead ahead |
| 9 | (−12, +7) | 88 | (−2,131,755,008, 13) |
| 17 | (−56, +35) | 87 | (−2,131,755,008, 65) |

The first round leaves from the bow and the other two from aft of centre,
all on the keel line. Both ships print these same three offsets for every
round they fire. They also hold one facing, `671481856`, broadside, for
the whole capture (§49), so the table is exact at that facing and the
rotation is §22.2's, read rather than measured for this piece. At that
facing no stern bearing reproduces the last two rows' integers, so each
row is the solution nearest the keel, 1.3° off it. ~~`sim::launch`'s table
carries piece 290's rows~~ run437's trireme fires the same three at 176°,
where those rows put the third a unit or two off and one flight a frame
long; the rows are whole-degree `BAYS` since item 1200, each reproducing
both facings (`run437_trireme_rounds_turn_with_the_hull`), and
`run127_trireme_rounds_leave_from_the_keel` holds all six launches. `CHAR_ATTACK2` plays the same file with the same
events and takes the same rows. ~~Which slot the original swings is not
printed at `GUYS=2`.~~ Whichever slot the roll gives, the swing plays
`Trireme Attack1`: the packet names no `CHAR_ATTACK3`, and `Guy::set_anim`
plays `CHAR_ATTACK2` in place of a slot the packet lacks (`docs/ANIM.md`
§4.13, item 549).

### 50.3 A sea figure stands on the water, not the lake bed

The item's first run with the widening comparing the whole `AMMO`
record, not the eleven fields it had, found a defect the draw had not
named. Every trireme round launched at `sz −185` against the dump's 88,
and its `v1z` was off with it. §46.1 took `sz` as the node's `dz` over
the figure's ground, `find_data_z`. That is right for a land figure and
wrong at sea. `Guy::update_z@005d9950`, and the figure loop in
`Unit::update_z@00606590`, give a figure whose unit type has
`domain == 1` a `z` of **0** and call `find_data_z` only otherwise.
Under sea region 70 the lake bed is at −273. `Sim::guy_release_events`
now takes 0 for a sea figure. **Diff-backed**: `sz` and `v1z` agree on
all seven rounds under the word, `v1z` to the last printed digit.

### 50.4 What moved

| | before | after |
|---|---|---|
| golden chapter five, **word** / sequence | 621 | **664** |
| values | 622 | 665 |
| widening window | `[605, 901)` | unchanged, run127 whole |
| `AMMO` fields compared | 11 | 16 (`sz`, `ez`, `angle`, `v1z`, `rolling`) |
| rows under the word, standing families aside | 5, plus the round | 13, all the fisher's |

**The value diff on the frame it moved.** On block 622 `1/6 ammo[0]` is
in both airs: `sx, sy` (12445, 35807), `ex, ey` (11642, 34749),
`total_time` 13, `sz` 88, `v1z` 61.399521, on both sides. `0/6`'s
broadside turn on 635 (§49.2) and its whole first volley, 640 to 652,
agree on every compared field. §49.4's "not diff-backed yet" is closed.

**The new word, widened.** On 664 the original spends
`Guy::set_anim+0x97a < Guy::inc_time+0x271`, 28 draws against 27. Block
665's only rows are the fisher `0/7`'s: the original's has dropped its
orders, cleared `unit_masks`' packed bit (`0x80000`) and raised `mylos`
4 → 6. Its birth `CASTORDER` has run out, and an animation ended with it.
This crate gave the fisher no orders on 621 (§49.5, parked 543), so its
fisher never casts. Both hulls agree on every compared field on 664 and
665. The word is now 543's, and no mechanism is named here. **Item 543
closed it** (`docs/ORDERS.md` §23), and the word went to 739.

The widening was made to fail twice. With `× 3 / 200` restored, the word
falls to 621 and `1/6 ammo[0]` returns on 622. With the lake bed's `z`
restored, `sz` and `v1z` part on 622 under the word.

### 50.5 What is not established

- **The keel nodes at any other facing.** They are measured at one
  facing and rotated by §22.2's reading. A capture with a ship shooting
  on a different bearing would test it: `AMMO=5`, any second trireme
  engagement.
- **The 355 other separating events.** Crossbowmen (`400`, `267`,
  `467`), the gunpowder line and most ships and aircraft now fire a frame
  earlier than they did here, by the listing. None is reached by a
  capture on this disk except the Trireme, and the long captures' words
  hold.
- **`starttime` under 67.** The floor makes it frame 1, so an event
  stamped 0 fires on the clock's first step, not at `cur_time` 0. Shipped
  attack slots do carry such events (the machine guns' packed
  `CHAR_ATTACK1` at 0, and aircraft `CHAR_ATTACK2`s). No capture reaches
  one, and whether `Guy::execute_events`' `last_time` of −1 at `cur_time`
  0 would ever have fired a frame-0 event is moot under the floor.
- **`ez` for a ship target** agrees at 0 on every round, through
  `aim_z`'s floor at 0. `Unit::update_z` gives the unit itself
  `find_tcoord_z` whatever its domain. That is reading-only here, and
  the floor hides it.

### 50.6 Coverage

- **Diff-backed**: the divisor (§50.1) on seven rounds' launch blocks
  and the volley spacing through the capture; the three keel nodes at
  one facing; a sea figure's `z` of 0 through `sz` and `v1z`; `0/6`'s
  broadside turn; the word 621 → 664 with its widening.
- **Listing-backed**: `init_unit_events`' divide and floor, the element
  and attribute names through `internal_strings.xml`, and
  `Guy::update_z`'s sea arm.
- **Reading only**: the node rotation for piece 290, and the `ATTACK2`
  rows.

## 51. A human's packed siege engine never searches (item 590, 2026-09-23)

Golden chapter three's word stood at **621** (run145, `docs/GOLDEN.md`
§7): this crate spent `Unit::fight+0x9b0` on the Catapult `0/9` the frame
after its birth, 7 draws against 6, where the original's catapult, born
packed (`unit_masks 0x80000`), holds no order at all until its unpack
(`CASTORDER spell 652`) on block 696. The item booked no mechanism;
587's hypothesis was `Unit::think_attack`'s packed-unit arm.

**What would kill each reading**, written before the fix
(`docs/journal/2026-09-23-item-590.md`): the AI-off block hiding the
catapult from its think (killed by the dump: the unpack cast *is* an
order, taken under `!ai off`); `idle` at birth (killed: `idle 1` on 621
both sides); the packed bit (killed: the harness compares it,
`harness.rs`' `packed_diverged`, and it agrees). What was left is the arm
itself, and its test was that a fix return before the search, lose 621's
draw, and put this crate's own unpack on 696 at `idle 7`.

### 51.1 The arm

`Unit::think`'s auto-attack step (`think@005f6e40:150`, the arm
`type.attack != 0 && role & 0x10000 && think_attack()`) runs **above**
the AI-off exit, so it runs for a human's unit under `!ai off`
(`docs/INPUT.md` §11.9's lesson again). This crate's step went straight
to `find_melee_target`. The original's enters `Unit::think_attack
@005f5a80`, and between that function's stance head and its army join
sits this (`llvm-objdump 0x5f5b0a..0x5f5c00`; the devirtualised `is`
calls take their arguments from the pushes at `5f5b40` and `5f5b76`):

```text
if type.unit_flags2 & 4 and unit_masks & 0x80000:        # 0x2b8 & 4; packed
    if !is(MERCHANTDUTCH 0x3e, 1):
        if get_packer_stance() == PACKER_AUTO:            # vslot +0x100
            thr = is(MACHINEGUN 0x7b, 0) ? 3
                : leader_flags & 4 ? 7 : 0x15             # human : computer
            if (unsigned) idle >= thr:
                add_cast_order(-1, -1, -1, -1, 0x28c, QUEUE_FIRST, 0)
                return 0
        if manual: return 0                               # 5f5bf6 → 5f5fdd
```

- `manual` is the head's `uVar12`: set when the unit is not AI-driven
  (`unit_masks & 0x40000` clear), when it is but its leader lacks
  `leader_flags & 2`, or under `leader_flags2 & 8`. A human's catapult
  is `manual`, so **below the threshold it returns before the search,
  and at it, the unpack is its first order**. A computer's packed engine
  falls through to the join and the search while it waits out its 21.
- `get_packer_stance@00610970` answers the unit's `stance` byte when the
  type's stance type is `STANCE_PACKER` (3), else `PACKER_NEVER` (1).
  `PACKER_AUTO` is 0. The enumerators are the PDB's `LF_ENUMERATE`s; the
  dump prints `stance 0` on both captures' catapults.
- `QUEUE_FIRST` is 0 (the PDB), the second push at `5f5bc9`.
- `add_cast_order@005e4a60` re-aims `0x28c` at `0x28e` for the machine-gun
  lineage before the merchants and the fishing boat. This crate carried
  the other two arms. The machine gun's is now reachable, and modelled.

`Sim::think_attack_packed` is the arm, called in `think`'s auto-attack
step ahead of the join and the search. A `true` is the function's
`return 0`: the think goes on past the step, as it does when nothing is
found.

### 51.2 The cadence

The two captures place the cast differently, and the arm explains both.
The auto-attack step runs on a unit's first idle frame and then one frame
in thirty-two, phased by `o`; `idle` climbs one frame in sixteen on the
same phase. So the cast lands on the first **32-phase** frame at which
`idle ≥ 7`:

| capture | unit | `idle 7` reached | 32-phase? | cast | unpacked |
|---|---|---|---|---|---|
| run145 | `0/9` | 696 (`695 + 9 = 704`) | yes | 696, `idle 7` | 776 |
| run146 | `0/6` | 683 (`682 + 6 = 688`) | no, 16-phase only | 699, `idle 8` | 779 |

(Block `N` is sim frame `N − 1`.) 587 read run146's 699 as the unpack
arriving "one grid step late". It was the cadence, not the threshold.

### 51.3 What moved

| | before | after |
|---|---|---|
| chapter three (run145), word / sequence | 621 | **633** |
| values | 622 | 634 |
| the restage (run146) | 633 | 633, unchanged |
| catapult rows parting, run145 whole | 14 under the word, then the fight | **none** but the standing `form` |

**The value diff on the frames it moved**
(`chapter_three_s_catapult_unpacks_on_the_dump_s_frame`), `(idle,
packed, head order, spell)` on both sides, identical on every row:
run145 `0/9` on 621 `(1, 1, −, −)`, 695 `(6, 1, −, −)`, 696 `(7, 1,
CAST_SPELL, 652)`, 776 `(0, 0, −, −)`; run146 `0/6` on 683 `(7, 1, −, −)`,
698 `(7, 1, −, −)`, 699 `(8, 1, CAST_SPELL, 652)`, 779 `(0, 0, −, −)`.

**A reader the move needed.** The dump prints `spell` and `paid` on every
`CASTORDER`, and nothing parsed them: `order:kind` reads `CAST_SPELL` for
every cast, so an unpack agreed with any other spell. `OrderDump` carries
both now, and `OrderMismatch::Cast` compares them (`order:cast.spell`,
`order:cast.paid`). The row was made to fail first: with the arm casting
`0x28e`, the widening reports `f696 0/9 order:cast.spell: ours 654 theirs
652`.

**The new word, widened.** Both captures now part on **633**, in one
shape: the original's chariot `0/8` spends one `Unit::fight+0x9b0` and
two `Guy::set_anim+0xf2f` (printed bare, `5db22f`), and this crate spends
a second re-search. What parts at and one block past it, standing
families aside, is `0/8`'s facing on 634, and nothing else: this crate
has turned, and the dump's still faces the way it was born. Both sides
hold the same `ATTACKORDER` on `1/8` on 633 and the same `recharging 25`
on 634. The dump's `0/8` moves and turns on no block from 630 to 720 in
either capture, and its figure carries `ox 8 whom 1` from 634. So the two
`set_anim` draws are **an attack swing starting without a turn**, not the
walk run146's pin read them as. That is a measurement; why the chariot
need not face its target is not established.

### 51.4 What is not established

- **A computer's packed engine.** It falls through to the search while
  packed until `idle` reaches 21. The unit test holds this crate to the
  reading. No capture has an AI siege engine idle and packed with a
  target in view: run44's two catapults are ordered at birth. It would
  take a capture with `!ai off` absent, `UNITS=3`, and an AI catapult
  trained within its LOS of an enemy.
- **`PACKER_NEVER`.** A human's engine with stance 1 neither casts nor
  searches. No capture sets the stance; `Unit::set_stance` from the
  interface would.
- **The machine gun's threshold of 3** and its `0x28e`, both
  listing-only.
- **`is(0x3e, 1)` read as the Dutch merchant's strict lineage.** A
  merchant reaches `think_attack` only through `think`'s merchant arm,
  and only when it is not packing, so this test cannot fire on the path
  modelled here.
- **`leader_flags2 & 8`** in `manual`, the seam `think_attack_join_army`
  already names.

### 51.5 Coverage

- **Diff-backed**: the arm for a human's packed catapult, in both
  captures, whole (no catapult row parts in run145 but the standing
  `form`); the 7 threshold and the 32-phase cadence (run145's cast at
  `idle 7`, run146's at 8 after a 16-phase 7); the unpack's `spell 652`
  and its 80 frames; the word 621 → 633 with its widening.
- **Listing-backed**: the `is` arguments, `QUEUE_FIRST`, the unsigned
  compare, the `manual` return; `StanceTypes` and `PackerStanceIndex`
  from the PDB.
- **Reading only**: the computer's fall-through and its 21, the machine
  gun's 3 and its `0x28e`, and `PACKER_NEVER`, each pinned by
  `a_human_s_packed_siege_engine_unpacks_before_it_searches` from the
  same reading.

## 52. A pivot that bears shoots without turning (item 595, 2026-09-23)

Golden chapter three's word stood at **633** in both captures (run145,
run146, `docs/GOLDEN.md` §7). The original's chariot `0/8` took its first
attack on its first frame in range. It spent one `Unit::fight+0x9b0` and
two `Guy::set_anim+0xf2f`, and its `angle` stayed `1431655765`
(`0x5555_5555`, 120°, the facing it was born with) through the shot and to
720. This crate turned it to the bearing (`991232000` in run145,
`998768640` in run146), deferred the swing into `GuyData +0x9e`, and spent
a second re-search in place of the two rolls. The item booked no mechanism.

**The disk first, and what it killed** (`docs/journal/2026-09-23-item-595.md`
has the kill conditions, written before any code). The target and its
frame agree: `ATTACKORDER ox 8 whom 1` with `new_ord 1` on 633 on both
sides. The range agrees: `attack.in_range` is 1 on 634 on both sides, a
row item 595 added to chapter three's widening, because the order
comparison read the kind and the target and never this field (§44.2.1).
The reload agrees: `recharging 25` on 634. What was left was the facing.

### 52.1 The rule

`Unit::fight@005fd4d0:591` calls `set_attack(o, who)` once the unit is
attacking, and **before** it chooses an angle. At `:722`, a non-zero
answer makes the angle `this->angle` in place of `find_angle` to the
target (and a wall, broadside or air-target adjustment), so `set_angle`
does not run and the figures' `des_angle` get the heading they already
have. The attack goes on in the same frame.

`Unit::set_attack@005fce70`:

- aims figures `0 .. guy_mark` (`UnitData +0xb5`, which is
  `anim::SQUAD_SIZE`, 1): `GuyData +0x8e ox` and `+0x9f whom`. **A crew
  figure is not aimed.** run145 prints it: `0/8`'s second figure, the
  chariot's horse, reads `ox −1 whom −1` from 634 while figure 0 reads
  `ox 8 whom 1`. This crate had aimed every figure.
- when the unit's graphic type has `<RESTRICTION>` rows, its type has a
  `max_range` (`UnitType +0x1fc`), and the target is a live unit or a
  building, it answers with the **last** of those figures'
  `Guy::set_all_pivots`. Otherwise it answers 0 and the unit turns.

`Guy::set_all_pivots@005d8bc0` (listing `0x5d8d80..0x5d8f93`):

```text
if !(guy_flags & 0x100): return 0              # Guy::init_real: the piece has restrictions
if aim invalid and order != ATTACK_GROUND: return 1
can = 1
for node in 4 .. 4 + count:                    # count = the type's rows
    (lo, hi) = get_restrictions(type, node)    # (0, 0) for a node with no row
    p = get_position(gpiece, node, facing)     # the node's point, rotated (float)
    b = find_angle(target − (unit + (int)p))   # from the UNIT's point, not the figure's
    d = angle_to_degrees(b − guy.angle)        # GuyData +0x18
    if d > 180: d −= 360
    if d > 45 or d < −45: can = 0              # 00b69674, 00b697bc
    inside = lo < hi ? lo ≤ d ≤ hi : d ≥ lo or d ≤ hi
    if !inside: can = 0; continue
    des_turret_angles[node − 4] = b − guy.angle; the node bits
return can
```

The constants are the PE's own floats (180.0, 360.0, 45.0, −45.0, read at
`00b696c0`, `00b696e4`, `00b69674` and `00b697bc`). Every compare is on
whole degrees, so the floats change nothing. A wrapped range (`lo ≥ hi`)
is an arc through 180°: the Dreadnought's node 5, `45..−45`, is its rear
turret.

`angle_to_degrees@00a28e00` is **not** `round(a × 360 / 2³²)`. It takes
the angle apart in steps of truncated constants (90°, 45°, 30°, 15°, 5°:
`0x4000_0000`, `0x2000_0000`, `0x1555_5555`, `0x0aaa_aaaa`,
`0x038e_38e3`), and rounds the remainder at `0x005b_05b0` against
`0x00b6_0b60` a degree. Each step leaves the remainder a few units long,
so a half degree rounds up a few units early: `5_965_232` reads 1 where
rounding reads 0. It was checked against the decompiled arithmetic over
two million random angles and every half-degree edge ±40 units, outside
the crate. `movement::angle_to_degrees` is written as those steps.

### 52.2 The data

`GraphicPieces::init_pivot_restrictions@008ebfe0` reads
`unit_graphics.xml`'s 81 `<RESTRICTION name node minangle maxangle>` rows.
It walks every unit type, names each by its `GRAPH` column
(`set_unit_graph_name`), and files each row whose `name` matches under
that type. **The match is case-insensitive**: `String::operator==
@00a1f140` ends in `_wcsicmp`. So `Cruiser`, `Mameluke`, `Katyusha` and
`CamelArcher` bind to their upper-case `GRAPH`s, and `SuperBattleship`,
which no `GRAPH` carries, binds to nothing. The Chariot's row is `CHARIOT
node 4, −180..180`. `rondata::artdata::pivot_restrictions` reads the rows
into `sim::anim::Art::pivots` (`TypeIndex → node → (min, max)`). The table
is gameplay data, as the release frames are (§9.0): it decides whether
`fight` turns the unit.

### 52.3 The swing that is not deferred

`set_anim`'s attack deferral (`docs/ANIM.md` §6.2) fires while a figure's
`des_angle` differs from its angle. A unit that keeps its heading owes
no turn, so the swing plays and **rolls in `fight`'s own frame**. The
original's trace names both rolls: `Guy::set_anim+0xf2f <
Unit::set_anim+0x56 < Unit::fight+0x19f6` for figure 0, and `+0xb6` for
the crew. Those two were the bare `5db22f` at 633. They are
`anim::SITE_ATTACK_FIGHT` and `SITE_ATTACK_FIGHT_CREW` now, and
`Unit::set_anim`'s loop marks them, since `fight`'s swing is the one
caller that asks for an attack with the roll argument set.

### 52.4 What moved

| | before | after |
|---|---|---|
| chapter three, run145: word / sequence / values | 633 / 633 / 634 | **682 / 682 / 683** |
| the restage, run146 | 633 / 633 / 634 | **664 / 664 / 665** |
| `0/8` on 633–634, both captures | turned, swing deferred, crew aimed | agrees on every row |

**The value diff on the frame it moved**, 634: `0/8`'s `angle`,
`heading` and both figures' `g.angle` read `1431655765` on both sides
(ours had `991232000` in run145 and `998768640` in run146). Its crew
figure's `g.ox`/`g.whom` read −1 on both (ours had `8`/`1` and `6`/`1`).
Nothing parts on 633 or 634 in either capture.

**What the frame says next.** Values now part before the draws in both
captures, and none of these rows names a mechanism:

- **run145, 635**: the chariot `0/6` takes its first target, `1/7` here
  and `1/8` in the dump. The arrows follow the targets, and the word,
  682, is this crate killing `1/7` (`Unit::close+0xcb6`), which the dump
  kills on 704.
- **run146, 651**: `0/8`'s first arrow leaves on the dump's frame, but
  from the unit's own square and height (`792, 13368, z 198`). The dump's
  leaves from `764, 13303, z 406`, the archer's release node, which §22's
  table has not measured for the Chariot.
- **run146, 664** (the word, 31 draws against 30): `0/9` swings again, and
  this crate's horse rolls a fresh attack (`+0xb6`) where the original's
  rolls nothing. This crate's horse has stood on figure 0's slog slot
  (7) since its first swing on 639. Figure 0 deferred that swing for its
  turn, the crew rolled at once, and phase 7's mirror (`docs/ANIM.md` §5)
  then copied figure 0's walk slot into the crew. From then on the horse
  is walk-category and steps its own clock, so on 664 it is not swinging.
  The dump prints no figure clock at `GUYS=2`, so no widening row parts
  on 664 or 665.

### 52.5 What is not established

- ~~**The pivot node's offset.** `GraphicPieces::get_position@0090b750`
  rotates the node's model-space point by the figure's facing, in
  floats, and the bearing starts there, truncated to integers. This
  crate starts it at the unit's point. For the Chariot at shooting range
  that is under a degree, so it can decide only a bearing within about a
  degree of ±45°. A pinned per-type table of node offsets would settle it.~~
  Settled by item 603, §54: the Chariot's node is 118 units from the
  unit's point, not "a few", and on run145's 684 it moves the bearing 8°.
- ~~**The turret angle and node bits** (`+0x30`, `+0x96`, `+0x98`) are not
  carried. Nothing in the simulation reads them, and `GUYS=2` does not
  print them. `GUYS=4` does (`turret_angles`, `des_turret_angles`,
  `node_flags`, `des_node_flags`), and item 603 reads two of them against
  the node's bearing (§54.2).~~ Carried since item 602 (§55.3), `+0x98` aside: the release reads
  the turret.
- **The air-target clause** (`fight`: a target with domain 2 and the
  attacker's `has_objmask(0x80000000)` keeps the heading too) is not
  modelled. No capture has a unit shooting at a plane.
- **A target with no ids.** `fight` enters `set_attack` only when both
  ids are ≥ 0. Otherwise its `param_4` stays at 1 and the unit keeps its
  heading. Whether any attack reaches `fight` that way is not read.
- **Which attribute is `v[0]`.** `init_pivot_restrictions` reads two
  floats, and the decompile loses which lands in which slot. This crate
  takes `minangle` as the low bound. Every Chariot row is symmetric; the
  asymmetric rows (a PT Boat's `−40..140`) are untested.
- **The crew figure's slot after a deferred swing** (run146's 664, above).

### 52.6 Coverage

- **Diff-backed**: the Chariot shooting on its heading at 36.9° off, in
  both captures, on every row of 633–634; the crew figure left unaimed;
  both roll sites and their callers, named by the trace.
- **Listing-backed**: the ±45° test and its constants (the PE's floats);
  the restriction range's two branches; the bearing from the unit's point;
  `set_attack`'s last-figure answer; the return addresses
  `0x616f96`/`0x616ff6`.
- **Reading only**: the case-insensitive binding (the decompile of
  `String::operator==`, and `a_pivot_restriction_binds_by_graph_whatever_its_case`
  holds this crate to it); every pivot type other than the Chariot; the
  wrapped ranges. `a_pivot_that_bears_shoots_without_turning` pins the
  45° edge and both kinds of range from the same reading.

## 53. The flank reduction is the target's, and the ranking skips overkill (item 601, 2026-09-23)

Golden chapter three's word stood at **682** on run145. It came down from
a value parting on **635**: the chariot `0/6` took `1/7` there, where the
dump's took `1/8`. All three original chariots take `1/8`: `0/8` on 633,
`0/7` on 634 and `0/6` on 635, each on its own 32-frame phase. None of
them takes the nearest, `1/6`. `docs/journal/2026-09-23-item-601.md` has
the disk and the kill conditions, written before any code.

**The disk first, and what it killed.** The widening already compared
every record on 635. The search's inputs are the positions, hits,
facings and the unit's own `z`, which step 23 reads. `z` had not been
compared, so `widen_chapter_three` now reads it on every unit of every
block. It is quiet. With all three hoplites in one fog half-cell and all
three in range, order, metric, visibility and staleness were killed on
the disk. What was left was the value. This crate scored `1/7` at
113892 against 112506 for the other two. With the height bonus zeroed,
all three read 112506. So `1/7` won on step 23 alone, `⌊94 × 10 × 22 /
20000⌋ = 1`: a `dmg` of 22, and the tallest drop of the three.

### 53.1 The reduction reads the target's mask

§6 step 19's VEHICLE/MOUNTED test is printed on `extraout_EDX`. From
`llvm-objdump 0x6441a0..0x644b90`:

- `006441d3`–`006441d9`: `-0xc(%ebp)` is `this`'s type `+0x1e4`, the
  attacker's mask (`local_10`).
- `006441df`–`006441e7`: `-0x8(%ebp)` is the target's type `+0x1e4`
  (`uVar12`). Nothing else in the function stores to it.
- `00644ace`: `edx` is loaded from `-0x8(%ebp)` for the CIVILIAN test,
  and `00644af1` copies it to `ecx` rather than masking it.
- `flanking@0092cfe0` uses `ecx` and `eax` only.
- `00644b3a`: `testl $0x200000, %edx`, then `$0x1000`.

So a flanked **vehicle** or **rider** takes `flank_bonus ×
vehicle_flank_bonus >> 8` (or the cavalry one), and anything flanking
foot deals the whole `flank_bonus`. This crate had read `amask`, so the
chariot (MOUNTED) got 50 × 40 >> 8 = 7 % where the original gives 50 %.
With the whole bonus, `dmg` on 635 is 32. Every hoplite then takes the
height bump (`⌊77·10·32/20000⌋ = ⌊94·10·32/20000⌋ = ⌊86·10·32/20000⌋ =
1`), all three score alike, and the tie goes to `1/8`, first on the
cell's chain, as it did for `0/8` on 633 on both sides.

A real arrow's damage agrees on every compared row. The first hits on
`1/8` (659) and on `1/7` (679) read 20 on both sides. Whether any of
them flanked is not read here.

### 53.2 `compare_target` passes `check_overkill` 0

With 635 agreeing, the next parting was 685: `0/7` re-searched after
`1/8` died and took `1/6`, where the dump's took the wounded `1/7`. This
crate had ranked `1/7` at a third: `0/8`'s stray arrow wounded `1/7` on
679, and overkill (step 21) applied inside the ranking. The listing at
`0064ebce`–`0064ec10` pushes `$0x0`, `$0x0`, `$0x0` (`param_6`,
`param_5`, `param_4`), then the bearing, `who` and `o`, and calls
`get_damage@00644130`. `param_5` gates step 21 and nothing else
(`get_damage:371`), and `Object::do_damage` passes 1 there (`:145`). So
a search ranks a freshly wounded target at its whole damage.
[`sim::combat::get_damage_checked`] takes the flag. `get_damage` is the
`do_damage` call, and `Sim::compare_target` passes `false`.

### 53.3 What moved

| | before | after |
|---|---|---|
| chapter three, run145: word / sequence / values | 682 / 682 / 683 | **684 / 684 / 685** |
| first value parting | 635 (`0/6`'s target) | 651 (the first rounds' launch) |
| the restage, run146 | 664 | 664 |
| chapters one, two, four, five and seven | closed | closed |

**The value diff on the frames it moved.** On 635, `0/6`'s
`ATTACKORDER` reads `ox 8 whom 1` on both sides (ours had `ox 7`). On
685, `0/7`'s reads `ox 7 whom 1` on both (ours had `ox 6`).

**What the frame says next.** On 684, `0/8` re-searches and takes `1/7`,
on both sides. `1/7` bears 69.9° from `0/8`'s square, 50° off its
heading of 120°, so this crate turns it (685: `834011136` against
`1431655765`). The dump's does not turn, and it rolls figure 0's swing
(`Unit::set_anim+0x56`) where this crate defers it. From the dump's own
release point for `0/8` (`957, 8075`, 67 units behind and left of the
unit's square), `1/7` is 43.3° off, inside ±45°. ~~That is §52.5's pivot
node offset (item 603), and it is more than a degree here. It is a
reading, not a measurement: the pivot node is not the release node.~~
Measured by item 603, §54: the pivot node is not the release node, and
from it `1/7` is 42° off. The
first value parting, 651, is item 602's shape on run145: the first
rounds leave from each unit's square and height. ~~`rolling` parts with
them (ours 1, theirs 0), the lofted-piece flag this crate loads no art
for (§42.2).~~ That row read the wrong field (§55.1). The rounds leave
through the turret (§55.2).

### 53.4 What is not established

- **A mounted or vehicle target's reduced flank** is read from the
  listing, and no capture on disk flanks one in a window this crate
  replays. `flanking_is_rear_one_side_two_and_front_nothing` pins both
  directions from the same reading.
- **`param_4` in `compare_target`** is splash, 0, which this crate
  already passed.
- **Whether any other caller of `get_damage` passes `check_overkill` 0.**
  Only `compare_target` and `do_damage` are modelled.

### 53.5 Coverage

- **Diff-backed**: `0/6`'s target on 635 and `0/7`'s on 685, on run145.
  The unit's `z` on every block of both chapter-three captures.
- **Listing-backed**: the target mask in step 19; the three zero pushes
  before `compare_target`'s `get_damage`.
- **Reading only**: the vehicle-before-cavalry order of the two
  reductions (`00644b3a`, then `00644b47`), which no capture separates.

## 54. The pivot bears from its node (item 603, 2026-09-23)

Golden chapter three's word stood at **684** on run145. The chariot `0/8`
re-searched after `1/8` died and took `1/7` on both sides. `1/7` bears
69.9° from the unit's square, 50° off the 120° heading, so this crate
turned. The original swung on its heading. Item 601 read it as the pivot
node's offset (§52.5), a reading from the release point. This section is
the measurement. `docs/journal/2026-09-23-item-603.md` has the kill
conditions, written before run147.

### 54.1 The rule

`Guy::set_all_pivots@005d8bc0`, from `llvm-objdump 0x5d8d80..0x5d8f93`
(the decompile garbles every argument of the call):

- `005d8d9d`–`005d8de1`: `get_position(graphic_pieces, gpiece = guy
  +0x88, node, anim 0, time 0, (float)angle_to_degrees(guy +0x18 −
  0x8000_0000), NULL, count, &v, NULL)`. The angle is **whole degrees**,
  `0..=360`: an `int`, made a `float` by `cvtdq2ps`, and taken back as an
  `int` by `Transform<float>::rotate`.
- `005d8e0a`–`005d8e35`: `find_angle(tx − ux − cvttss2si(v[0]), ty − uy −
  cvttss2si(v[1]))`. So the bearing is from **the unit's point plus the
  node's truncated vector**, per node.

`GraphicPieces::get_position@0090b750`, with `param_6` NULL, takes the
flat branch. It finds the piece's `AttachPos` entry `(node, anim 0,
time 0)`, rotates its point about z by that many degrees, negates y and
scales by `guy_scale × RData +0x88`. `GraphicEvents::init_unit_events
@008e2520:817–820` fills nodes 4–7 at anim 0, time 0 from the model
(`GraphicPieces::init_position@00901820`'s hierarchy walk) when the
piece's events are built. With no entry it writes a zero vector.

The rotation goes through `Quat<float>::set@00420c70`, which indexes a
360-entry half-degree table built at first use from `cosf`/`sinf`
(`fast_half_degree_to_cosine@00a29010`) with `deg % 360`, the C
remainder. **A negative degree reads before the table**: −60° gives
`(−103, −57)` for the Chariot's node where 300° gives `(−102, −59)`. The
pivot never passes one, because `angle_to_degrees@00a28e00` answers
`0..=360`.

Fitted to 361 degrees of two nodes on run147's packet, one of them a
release node with `px ≠ 0`, to within 2.6·10⁻⁵ (`s` the scale, `d` the
degree):

```text
v0 = s · (px · cos d + py · sin d)
v1 = s · (px · sin d − py · cos d)
```

### 54.2 The measurement, run147

The disk could not answer: both earlier packets (run144 and the lab's
Great Lakes one) hold the Chariot's pieces loaded but with no `AttachPos`
at all, and no dump prints the pivot node. `AMMO`'s `sx, sy` is the
release node, `(−31, −37)` from `0/6`, not the pivot's. So run147 is a
packet at logger frame 684 on chapter three's staging (`docs/RUNS.md`),
with `misc:MISC=10`. Mid-frame says run in `GAMELOGMODE_NONE`, the
`[Misc Logging]` row (`GameLog::check_accept@009309a0`), and
`set_all_pivots` says the node at detail 10 (`005d8eb4`):

```text
GUY get_positiong -180 180 -102 -59 1512 7944
```

That is `lo hi (int)v[0] (int)v[1] tx ty`, said during tick 684 against
`1/7`. Every Chariot of the chapter prints `-102 -59` on every frame it
bears. The packet gives the rest:

- piece 145's `(4, 0, 0)` entry is `(0.0, 24.64, 11.55)` (`0x41c51eb9`),
  its `RData +0x88` is 1.0, and `guy_scale` (`0xc06244`) is 4.8.
- `Unit::set_attack@005fce70(0/8, o 7, who 1)`, run under unicorn on the
  packet, answers **1**, and for `1/6` (`1608, 7800`) it answers 0. From
  the node (`882, 8077`), `1/7` is 42° off the heading and `1/6` 51°.
- run147 prints `GUYS=4`, which carries `des_turret_angles`: the node's
  bearing less the figure's angle, as `set_all_pivots` writes it. `0/8`'s
  reads `−496063829` on 685, the bearing on `1/7` from the node, and
  `−382227797` on 684, the bearing it last took, on `1/8` at `1828, 8040`
  on 658. The integer form gives both
  (`pivot::tests::the_bearing_from_the_node_is_the_dump_s_turret_angle`).

### 54.3 The integer form, and where it can differ

`sim::pivot` is the same function in integers: the node's point in
hundredths, `guy_scale` as 48/10, and a whole-degree sine pinned at 2³⁰
(`docs/DECISIONS.md` entry 16's third shape). Both sides truncate, so
they can differ only where the exact product lies within the float's
error of an integer. For the Chariot's node the float strays at most
4.0·10⁻⁵ from exact over all 361 degrees. No exact value comes closer
than 0.0161 to an integer (25°, 65° and so on give 49.9839), which is a
margin of 400×. The original's own `get_position` rows, called under
unicorn on the packet for `d = 0..=360`, agree on every degree
(`pivot::tests::the_original_s_node_agrees_on_every_degree`, with the
table outside git). Shifting the node by 0.01 of a model unit fails that
test at 40°.

Everything after the vector is integer already: `find_angle`,
`angle_to_degrees`, and compares against 45 and 180 that are whole
numbers in `float`.

### 54.4 What moved

| | before | after |
|---|---|---|
| chapter three, run145: word / sequence / values | 684 / 684 / 685 | **706 / 706 / 707** |
| the restage, run146 | 664 | 664 |
| chapters one, two, four, five and seven | closed | closed |

**The value diff on the frame it moved**, 685: `0/8`'s `angle`,
`heading` and both figures' `g.angle` read `1431655765` on both sides
(ours had `834011136`). `1/4`'s move order, which followed 684's draws,
agrees until 804.

**What the frame says next.** On 706 the original spends
`Ammo::do_damage@00678060+0xc59` and `+0xc7e` at draw 12, and this crate
spends nothing (18 draws against 20). The two sites are the ±20 scatter
of a round landing with no live target (`00678bea`–`00678c08`: `ox` or
`whom` negative, or the target is the shooter). The rounds on 706 already
differ from their launch. Each chariot's leaves from the unit's square
here and from the release node in the dump, with `rolling` 1 against 0.
That is 651's shape and item 602's. The widening pins 706 and 707 whole:
only rounds part there.

### 54.5 What is not established

- **Every other restricted piece.** Only the Chariot's figure (piece 145,
  node 4) is pinned. Any other piece bears from the unit's point, as this
  crate did before. The rows are the original's once the piece's events
  are built, and a packet from a game that fields the type reads them in
  a minute (`get_position` on the packet).
- **Another nation's Chariot.** `get_unit_gpiece` built 145 for chapter
  three's Nubians. A different art set could name a different piece with
  a different node.
- **`RData +0x88` other than 1.0**, and `get_gpiece_type` other than 0
  (`build_scale`, 3.0, then applies). Neither is modelled.
- **A node with `px ≠ 0` in the integer form.** The convention is fitted
  on one, but no pivot node on the disk has one, so the sweep covers only
  `px = 0`.

### 54.6 Coverage

- **Diff-backed**: `0/8` keeps its heading on 684–685 in run145; the
  node's vector at 120° is the dump's own `MISC=10` line; and the bearing
  from the node is the dump's `des_turret_angles[0]` on 684 and 685.
- **Oracle-backed** (the original's functions under unicorn on run147's
  packet; the native twin refuses both on packed SSE, `005d8dcb` and
  `0090b818`): the
  node's model point and scale; `get_position` on all 361 degrees against
  the integer form; `set_attack`'s answer for `1/7` and `1/6`; the
  rotation's convention on two nodes.
- **Listing-backed**: the call's arguments and the bearing's arithmetic
  (`005d8d9d`–`005d8e35`); `Quat<float>::set`'s `% 360`.
- **Reading only**: that `init_unit_events` is where the entry comes from.

## 55. A pivot piece releases through its turret (item 602, 2026-09-23)

Golden chapter three's two words, run145's 706 and the restage run146's
664, both ran through the chariots' rounds from 651. Items 595 and 603
read two rows there: the arrow leaves from the unit's square here and the
release node in the dump, and `rolling` reads 1 against 0.
`docs/journal/2026-09-23-item-602.md` has the kill conditions, written
before any fix. The floor killed three of the four candidates (the lofted
term, the round's target, the flight's step), and the fourth, the launch
node, turned out to be the turret.

### 55.1 The floor: `rolling` was the wrong field

`AmmoData` is `+0x4 flags` and `+0x5 rolling`, two bytes. §42.2's flag 4
lives in `flags`. `rolling` has two writers: `Ammo::init@0067bbf0:830`
zeroes it on every round, and `Ammo::init_crash@0067b800` gives a falling
aircraft a random roll. The widenings had compared this crate's flag 4
with the byte, so every rolling round read "ours 1 theirs 0". The dump's
own `flags 6` carries bit 4 on both sides for every chariot round.
`rondata::diff::golden::ammo_flag_rows` reads `flags & 4`, `flags & 8`
and the byte as three rows at both `AMMO` comparison sites.

§42.2's third term is not "lofted" either. `ammo_flags & 8` is
`GraphicPieces::init_ammo_piece_ranges@008f6140`'s reading of `<ammo
missile="1">` (`internal_strings` 1567; the other bits are `puncture` 1,
`puff` 2, `explosive` 4, `flame` 0x10, `pulsewave` 0x20, `flak` 0x40, and
`do_damage="0"` 0x80). `ArcherArrowWOtip`, the Chariot's arrow, has
`missile="0"`.

With the floor right, 651 and 652 part only on the launch point, `sx`,
`sy`, `sz`, and on the two fields taken from it, `angle` and `v1z`. That
is true in both captures, on every record and figure of both blocks.

### 55.2 The release goes through `get_position`'s pivot branch

The dump's offsets from the figure are not one vector per `(slot, frame)`
(§22's table). At one facing, 120°, run145's 13 live rounds leave from
radii of 43 to 88 units. But every one lies 71–75 units from the pivot
node's point `(−102, −59)` (§54).

`GraphicEvents::execute_game_events@008e48e0`'s release, from the listing
(~~`008e4a8a`–`008e4ae7`~~ **`008e4c7d`–`008e4cdb`**: item 1257; the
first range is the kind-5 event's, a unit put out of a carrier by
`Unit::come_out` and `set_new_location`, and the release's `add_ammo`
returns to `8e4ced`, `execute_game_events+0x40d`, the trace's own
caller):

- `get_position(piece, node = event +0x23, anim = event +0x8, time =
  event +0xc, param_5 = ~~`(float)angle_to_degrees(package.angle −
  0x8000_0000)`~~ **`fast_angle_to_degrees(package.angle −
  0x8000_0000)`** (`8e4c7d`–`8e4ca1`: `leal −0x80000000(%ebx)`, `call
  0xa28f70`, `movss %xmm0, (%esp)`), param_6 = package.pivot_angles,
  param_7 = has_restrictions(gpiece), &v, &dir)`, then `x, y, z +=
  cvttss2si(v)`. The two degree functions differ by up to two degrees:
  the Bomb Vessel of run484 faces 354 by `angle_to_degrees` and 353 by
  the table, and only 353 puts its rounds at the dump's offset (`docs/GOLDEN.md` §53).
- `package.pivot_angles[k]` is `fast_angle_to_degrees(turret_angles[k])`
  (`Guy::execute_events@005d99c0`, `005d9a2e`–`005d9a6d`), zero when all
  four are zero.
- `fast_angle_to_degrees@00a28f70` is a 256-entry table,
  `(float)angle_to_degrees(i << 24)`, indexed by the angle's top byte
  (`a28ffe`). run147's packet holds it: `0, 1, 3, 4, 6, 7, 8, …, 359`.
- The event fires only if `has_restrictions` is 0, or `node & 3` is past
  the count, or `node_flags` has bit `node & 3`. The gate reads the
  event's node and the piece's restriction count, never the pivot
  vectors: §85 (item 1117), which found this crate keying it on them.

`GraphicPieces::get_position@0090b750`'s pivot branch, taken when `node &
3 < param_7` and `param_6` is set (`90b866`–`90b926`):

- the **pivot node's** vector: a nested call for node `(node & 3) + 4`
  at the same anim and time (`param_4` is `0x14(%ebp)`, `90b7f3`; the
  decompile shows it right), at `param_5` degrees, scaled, then unscaled
  and its y un-negated;
- plus the release node's entry rotated by `cvttss2si(pivot_angles[node &
  3] + param_5)` (`90b916`–`90b926`), both floats whole numbers;
- y negated, all scaled by `guy_scale × RData +0x88`.

So, with `d₁` the facing's degree **by the table** (`sim::pivot::
release_rotation`) and `t` the turret's step, both integers:

```text
v = s · (R(d₁)·P + R(d₁ + t)·E), y negated, each component truncated once
z = s · (P_z + E_z)
```

`GraphicEvents::init_unit_events@008e2520:833–841` builds the entries:
for each release (and type-5) event, nodes 4–7 and the event's node at
the event's anim and every frame of it. Piece 145's, read from run147's
packet (`(x, y, z)`, bit patterns):

| slot | frame | pivot node 4 | release node 0 |
|---|---|---|---|
| `ATTACKWALK` 10 | 18 | `3ebd0bc6 41c51eb9 4144caeb` | `400ff9ae c17700f8 41f6fd6e` |
| `ATTACK1` 11 | 18 | `0 41c51eb9 4138cccd` | `bdcf99c0 c1788744 41ffc9b0` |
| `ATTACK2` 12 | 17 | `0 41cdc290 4138cccd` | `3f8401aa c17c62fc 41ea87a0` |
| `ATTACK3` 13 | 18 | `0 41c51eb9 4138cccd` | `3d4a3e80 c176f97e 41ff66bc` |

The height is `4.8 × (11.55 + 31.97) = 208.9` for `ATTACK1`, 208 printed,
and `196.2` for `ATTACK2`: the dump's two `dz`, 208 and 196.

**The integer form, and the six cells it misses.** `sim::pivot::
release_offset` holds the entries in millionths and uses §54.3's
whole-degree sine. The original's own `get_position` was called under
unicorn on run147's packet for **every reachable cell**: four rows, `d₁ =
0..=360`, each of the 256 turret steps, 369,664 cells in 23 s. The exact
form misses **six**, each one where the float sum rounds onto an integer
from just below (`97.0` against 96.99999…). They are pinned per row
(`Release::float_lands`), and the table outside git is
`~/ron-data/lab-experiments/2026-09-23-item-602-release-oracle.txt`
(`the_original_s_release_agrees_on_every_cell`, made to fail by emptying
one row's list).

### 55.3 The turret is carried

`GuyData +0x20 turret_angles[4]`, `+0x30 des_turret_angles[4]`, `+0x96
node_flags` (`anim::Turret`):

- **`Guy::set_all_pivots@005d8bc0`** first zeroes `node_flags` and
  `des_node_flags` (one 32-bit store). If the aim is gone (and the order
  is not `ATTACK_GROUND`) it answers 1 there. Otherwise, for each node
  whose range holds the bearing, it writes `des[k] = bearing − the
  figure's angle` and `des_node_flags` bit `k`, and `node_flags` bit `k`
  when `des[k]` is within 15° of `turret[k]` (`005d8ed7`–`005d8f02`: the
  unsigned difference, `~` past a half turn, below `0xaaa_aaaa`). The
  ±45° test does not gate the write.
- It is called from `Unit::set_attack@005fce70`, for each aimed figure;
  from `Guy::move@005d9240:71, 95, 102`, when a deferred swing is paid,
  on both arms; and from `Unit::move_step`'s cavalry-archer arm
  (`unit_flags & 0x200000`).
- **`Guy::process@005e0230:30–56`**, right after `Guy::move`, for a figure
  with `guy_flags & 0x100`: each turret on its `des` sets its bit; one
  within 15° snaps there and sets it; any other turns 15° toward it.

run147's `GUYS=4` shows the result on 684–686. `turret_angles[0]` equals
`des_turret_angles[0]` on every chariot. `0/8`'s moves from
`−382227797` to `−496063829` on 685 in one step, a 9.5° snap. And
`node_flags` reads 15 on every figure of the three chariots.

### 55.4 What moved

| | before | after |
|---|---|---|
| chapter three, run145: word / sequence / values | 706 / 706 / 707 | **900 / 900 / none**, the capture's end |
| the restage, run146: word / sequence / values | 664 / 664 / 665 | **780 / 780 / 781** |
| first value parting, run145 / run146 | 651 / 651 | 678 / 736 |
| chapters one, two, four, five and seven; both long captures | | unchanged |

run146's move also takes `docs/ANIM.md` §5.2, the crew swinging with its
leader. Without it, run146 stays at 664 with the rounds right.

**The value diff on the frames that moved.** On 706, `0/6`'s round in
pool slot 1 reads `sx 856, sy 7754, sz 477`, `angle 1267531776`, and
`v1z` 7.629168 on both sides (ours had `888, 7800, 281`, `1225588736` and
40.295834). `0/7`'s 703 round, landed and missed, reads `flags 14` on
both sides. The two scatter draws the original spends on 706 are this
crate's too. On 664, `0/7`'s and `0/9`'s rounds read `553, 13160, 378`
and `606, 13618, 370` on both sides (ours had `600, 13176, 182` and
`648, 13608, 162`). On 651, `0/8`'s reads `956, 8071, 448` in run145 and
`764, 13303, 406` in run146, both sides.

**What the frames say next.** On run145 nothing draws differently to 900.
The widening still parts on three `AMMO` families:

- a round's target, cleared here when its target dies (678, 705, 757 on
  run145; 736 on run146), where the original keeps `whom`/`ox` until
  `Ammo::check_hit` rewrites them on the round's due frame
  (`00678d90:42–82`);
- the pool slot a round takes (703, 729, 730, 754), one apart;
- `0/8`'s launch point on 753 (and its 729 round, under a pool-slot row),
  a unit off (§55.5).

run146's word, 780, is the catapult `0/6`: the original takes an
`ATTACKORDER` the block after its unpack and turns on 781. This crate
stays idle. Nothing is named for it.

### 55.5 What is not established

- ~~**The turned chariot's turret.** After `0/8` turns to 61.7° on 711,
  both of its rounds (729, 753) need a turret step of 1°. The bearing
  from the node gives `des` 0.06°, step 0, and this crate's is that. Only
  a turret between 1.4° and 2.8° reproduces the dump, and no reading so
  far gives one: the target stands still from 700, and the node, the
  facing and `fast_angle_to_degrees`' table are all measured. `GUYS=2`
  prints no turret. A `GUYS=4` capture over 705–760 on chapter three's
  staging answers it in one run.~~ **Answered by item 1257 without a
  capture** (`docs/GOLDEN.md` §53): the release's `d₁` is `fast_angle_to_degrees`', 240
  at 61.7° less a half turn where `angle_to_degrees` gives 242, and with
  it both rounds leave from the dump's point at the turret this crate
  already has.
- **Every other pivot piece that releases.** Only the Chariot's piece
  145 has rows, and the Bomb Vessel's 296 (`docs/GOLDEN.md` §53). Any other releases through §22's table, or from the
  unit's point, and skips the `node_flags` gate.
- **`Unit::move_step`'s cavalry-archer arm** (`unit_flags & 0x200000`,
  which the Chariot's `v` sets). It re-aims every figure on
  `UnitData +0xa2`/`+0xa8` while walking. It is not modelled, and the
  chariots of run145 never walk. run146's do, and they agree to 780.
- **`ATTACK_GROUND`**: `set_all_pivots` bears on the order's point. This
  crate writes nothing for it.
- **`des_node_flags`** is not carried: nothing in the simulation reads it.

### 55.6 Coverage

- **Diff-backed**: ~~19 of the 21~~ **all 21** live chariot rounds of
  run145 and run146 (item 1257: the turned chariot's two with the
  table's `d₁`), launch point, height, angle and arc, on every block they
  print (the chapter's widening); the Bomb Vessel's three rounds of
  run484 (`docs/GOLDEN.md` §53); chapter three's run145 draw stream and values to 900, and
  run146's to 780; the flag bits on every round.
- **Oracle-backed** (the original's `get_position` under unicorn on
  run147's packet): the pivot branch on all 369,664 cells; the four rows'
  entries; `fast_angle_to_degrees`' table. On run486's packet (item
  1257, `tools/recomp/get_position.py`): piece 296's seven entries, its
  scale, 277,248 release cells and its node's 361 degrees.
- **Listing-backed**: the release call's arguments, the pivot branch's
  nested call and rotation, `set_all_pivots`' writes, `Guy::process`'
  step, the gate.
- **Dump-backed without a diff**: `turret == des` and `node_flags` 15 on
  run147's 684–686.
- **Reading only**: `Guy::move`'s re-aim calls, which no dump can tell
  apart from `set_attack`'s here.

## 56. The unpack lights its disc (item 616, 2026-09-23)

Golden chapter three's restage (run146, `docs/GOLDEN.md` §7) stood at
**780**: the catapult `0/6`, unpacked on 779, takes an `ATTACKORDER` on 780
in the original (`Unit::fight+0x9b0`) and turns to it on 781, and this
crate's stayed idle, 5 draws against 6. The item booked no mechanism; 602
read it as the catapult's re-search.

**What would kill each reading**, written before the fix
(`docs/journal/2026-09-23-item-616.md`): the unpack's completion frame
(killed by the disk: `unit_masks 0` on 779 on both sides, no `0/6` row
parts there); the search's phase after an unpack (killed: `idle 1` on 780
on both sides, and this crate's auto-attack arm is entered and searches);
the 3-tile minimum (killed: the refusal is `valid_target`'s, above the
range gate, and the hoplites are 7 tiles off); the target choice (judged
only once the search sees a candidate). What was left was the fog.

### 56.1 The call

`SpellType::cast_unpack@006709c0`, after the merchants' arm:

```text
unit_masks &= ~0x80000                # 670b62
this->update_los()                    # vslot +0x160, 670b75
this->update_seen(0)                  # vslot +0x174, push $0x0 at 670b82
Unit::update_gpiece(this)             # 670b99
Unit::set_new_location(this, x, y, 1, 1)   # the unit's own position
```

`update_seen(0)` is the whole disc, not the ring (§31, `docs/VISION.md`
§6). The argument is the listing's push. The tail's `set_new_location` is
to the point the unit already stands on, so it crosses no half-cell and
lights nothing. This crate's `cast_unpack` cleared the bit and re-read the
line of sight lazily (`Sim::unit_los`), and nothing re-lit the fog. A
siege engine that unpacks where it stands kept its packed four-tile disc
until the next `update_all_seen` (`frame % 100 == 33`). Now
`Sim::cast_unpack` calls `update_seen(u, false)` between the bit and
`update_gpiece`, in the original's order.

**The measurement, before the fix.** A probe staged run146 and asked the
catapult's search, before each tick from 778 to 781, what it saw.
`find_melee_target(−1)` answered nothing on 780, and `valid_target`
refused all three of arena A's hoplites, at `attack_dist` 1344, 1488 and
1392, in `target_is_seen`. The dump's hoplites print `visible 0`, so the
fog was the only way the catapult's player could see them, and the dump's
catapult prints `mylos 10` from 779.

### 56.2 What moved

| | before | after |
|---|---|---|
| the restage, run146: word / sequence / values | 780 / 780 / 781 | **782 / 782 / 783** |
| first value parting, run146 | 736 | 736 |
| chapter three (run145) | 900, closed | 900, closed |

**The value diff on the frame it moved**
(`chapter_three_s_unpacked_catapult_sees_its_hoplites`): on 779 neither
catapult holds an order at `idle 0`. On 780 both hold an `ATTACKORDER`
(index 10) at `idle 1` on the hoplite standing at (2472, 8136). The row
compares the target by position, because its `o` is not an identity here:
the dump's `1/11` is this crate's `1/8` (parked 617, the `DEATH_OBJS`
cull). On 781, `0/6`'s `angle` (`1131216896`), figure 0's `g.angle`
(`1372003669`) and the crew's positions (`(938, 7883)`, `(660, 7965)`)
now read the dump's. This crate had `1431655765` and `(948, 7887)`,
`(663, 7945)`.

`the_unpack_lights_the_whole_disc_at_the_new_line_of_sight` holds the call
in a unit test. It was made to fail with the call removed: the disc after
the unpack was the packed one, 21 fog cells against 105.

### 56.3 What the frame says next: the siege arm

The new word, **782**: ours 7 draws against 5. This crate spends two
`Guy::set_anim+0x97a < Guy::inc_time+0x271` before the
`Guy::set_anim+0x104b` the original spends first. Values part on 781, and
only on `0/6`:

- the dump's catapult holds **two** orders on 781, the attack and, over
  it, an `ATTACKGROUNDORDER` (index 23, `flags −128`, `att_x 2472`,
  `att_y 8136`, `accuracy 0`, `attack_unit 1`). This crate has no such
  order, and the widening reads the row as `order:unspellable 23`;
- its figure 0 keeps `ox −1`/`whom −1`, where this crate's aims at the
  hoplite;
- its `recharging` reads 83 against this crate's 82.

That is §8.2 step 1's second half, which this crate does not carry.
`Unit::fight@005fd4d0`'s in-range branch, for a packing type
(`type +0x2b8 & 4`) that is unpacked and not the Dutch merchant: if the
target passes vslot `+0x18` and the type's vslot `+0x10c`, it calls
`set_attacking`, pushes an `ATTACK_GROUND` order at the head holding the
target's `x`, `y`, `accuracy = (target domain == 1)` and `attack_unit = 2`,
clears the partial path, calls `update_action` and `Unit::work` (vslot
`+0x188`, so the new order runs on the same frame), and returns. `Unit::do_attack_ground@005f1410` then fires one round at the
point: `attack_unit` 2 → 1 on the shot and the order is killed on the
next ready frame, the order's `flags |= 0x80` holds it while the reload
runs, and `recharging = UnitData::recharge() + 1` (vslot `+0x134`) is the
83. The
attack order underneath is what re-pushes the next one at the target's
new point. Where the round lands and whom it hurts is §9.3 and §9.4's.
~~None of this is built. It is the next item's reading, and the harness
needs an `ATTACKGROUNDORDER` reader before any row on it can be trusted.~~
Built by item 621, with the reader (§57). Two corrections to the
paragraph above: the point is the target's own position, not its cell
(§57.1), and the order is pushed and fired on one frame with the attack
beneath untouched, which re-pushes nothing until the ground order ends
(§57.3).

### 56.4 What is not established

- **The update_los call itself.** This crate's line of sight is derived
  on every read, so `update_los` has nothing to store. A term that
  `update_los` computes and this crate does not derive would part here
  first.
- **The pack direction.** `cast_pack` shrinking the disc lights nothing
  new, and `seen2` is monotone, so nothing reads it. Not read.
- **The merchants' arm** (`TypeIndex` `0x3d`/`0x3e`/`0x190`): its own
  `set_new_location` onto the tile corner comes *before* the bit and may
  cross a half-cell. §56.1's call follows it either way.

### 56.5 Coverage

- **Diff-backed**: the whole-disc call for a human's catapult, by its
  first search after the unpack (run146's 780, the value test above), and
  the turn on 781.
- **Listing-backed**: the argument 0 (`670b82`), the order of the four
  calls, and the tail's `set_new_location` being to the unit's own point.
- **Reading only**: ~~the siege arm and `do_attack_ground` of §56.3, which
  nothing here implements~~ — diff-backed since item 621 (§57.8).

## 57. Siege fire at a unit is ground fire: `ATTACK_GROUND`, built (item 621, 2026-09-23)

Golden chapter three's restage (run146, `docs/GOLDEN.md` §7) stood at
**782** after §56. Values parted on 781 on the catapult `0/6` alone: the
original held an `ATTACKGROUNDORDER` over its attack, and this crate
carried neither the order nor a reader for it. The item named no
mechanism beyond which order appears.
`docs/journal/2026-09-23-item-621.md` has the kill conditions, written
before the build.

### 57.1 The reader, and the disk

`OrderDump` reads the record whole (`att_x att_y accuracy attack_unit`,
`AttackGroundOrder +0x4..+0x10`, `docs/ORDERS.md` §1.2). The coverage pin
now walks run146's 780–784, and with the reader off it names exactly
those four keys. Across every kept dump and every golden capture, the
order appears in **run146** (83 records) and **run44** (94), and nowhere
else. Neither long capture has one.

run146's `0/6`, the dump's side (`the_ground_order_s_life_is_the_dump_s`):

| block | orders, front first | `recharging` |
|---|---|---|
| 780 | `ATTACK` on the hoplite, `in_range 0`, `new_ord 1` | 0 |
| 781 | `ATTACK_GROUND (2472, 8136)`, `accuracy 0`, `attack_unit 1`, `flags −128`; under it the `ATTACK`, `in_range 1`, `new_ord 1` | 83 |
| 782–863 | the same two | 82 … 1 |
| 864 | none | 0 |

No other unit of the capture ever holds one. The order is **pushed and
fired on one block**. The fired bit (`0x80`) prints as a signed byte.
The attack beneath keeps `new_ord 1` for all 83 blocks.

run44 has three engines with it, the human's `0/15` and the AI's `1/6`
and `1/7`. On every push block the order's point is **the position of the
target the attack beneath names, on that block**. `1/6`'s (39421, 40762)
is `0/14`'s own position on 247 and is off every cell centre, which kills
`docs/ORDERS.md` §7.2's "at the target's cell". `0/15` pushes at `rech
83`. On 407 it is killed on its ready block and its attack runs on in the
same block. On 408 a direct strike at a building reads `rech 82`, so the
extra frame belongs to the ground order.

### 57.2 `Unit::fight@005fd4d0`'s siege arm

It sits in the attack's in-range branch (`fight:496`–`572`), after
`in_range` and `ever_in_range` are written (`:491`–`493`) and before the
strike, on the ordinary path (`iVar13` is `param_5`, the cavalry archer's
melee call, and zero). For a type that packs (`+0x2b8 & 4`) and is not
the Dutch merchant:

- **packed**: the original unpacks (`add_cast_order(0x28c, QUEUE_FIRST)`),
  or an AI-driven entrenched engine out-ranged by its target moves to a
  better spot. This crate does not carry that arm (§57.7).
- **unpacked**, the target `is_unit` (its vslot `+0x18`) and the type
  `is_siege` (vslot `+0x10c`, VISION §9.2): `set_attacking(who)`, then a
  new `AttackGroundOrder` holding the target's `x_internal`/`y_internal`,
  `accuracy = (target domain == 1)` and `attack_unit = 2`. Its flags have
  `0x80` and `0x4` cleared, it is added at the head, the head pointer is
  rotated onto it, then `clear_partial_path`, `update_action`, and
  `Unit::work` (vslot `+0x188`, the vtable export), and `return 0`.

Returning there skips the strike's tail, whose `+0x20 = 0` (`fight:847`)
is what clears `new_ord`. `Sim::siege_ground_arm` is the arm, called from
`do_attack`'s in-range branch ahead of the strike. It pushes through
`enqueue(QueuePos::First)` and re-enters `Sim::work`, as the chase tail
already does.

### 57.3 `Unit::do_attack_ground@005f1410`

`Sim::do_attack_ground`, dispatched from `work` on the new body
`Body::AttackGround`:

1. `UnitTypeData::can_attack_ground@0061db20`, from the listing
   (`61db20`–`61dc13`), else kill. It is the rush-rules age gate, a
   `max_range` (`+0x1fc`), not `unit_flags & 0x102000`, and then EXPLOSIVE,
   SIEGE, NAVAL or BOMBARD in `obj_masks` (`+0x1e4`), or the machine-gun
   lineage. With `attack_unit == 0` (a player's click) a point at peace is
   killed and an out-of-range point walked to. No command here makes one.
2. The facing: `find_angle` to the point, with a quarter turn off for
   `unit_flags & 0x40`, whichever is nearer the heading.
3. **Reloading** (`+0xae`): a fired order (the flag byte negative) returns
   and holds the unit still. Otherwise a figure off slots 0–3 rolls the
   idle.
4. **`attack_unit`**: 1 is the ready frame. It kills the order and calls
   `Unit::work` again (vslot `+0x188`, not `do_idle` as ORDERS §7.3 had
   it), so the attack beneath runs on this frame. 2 becomes 1.
5. `set_attack(−1, −1)` (`005fce70`): the aimed figures' `ox/whom` go to
   −1, and with no target no pivot is asked. A packed packer casts the
   unpack instead.
6. `unit_masks |= 0x11000`, then `set_angle` when the facing is new,
   `flags |= 0x80`, and the swing: `fight`'s own choice, `ATTACK1` with
   the third argument, or the rocking pair for `unit_flags & 0x2000000`.
   Then `Object::fire_ammo(−1, −1)` if the type has a projectile
   (`+0x2cc`), and **`recharging = recharge() + 1`**.

**On 864** the ready frame's re-entered `work` runs the attack, and
`fight` kills it. The hoplites have walked inside the three-tile minimum
(`1/11` stands about 410 units off), and an unpacked packer whose attack
was ever in range dies out of range (`fight:496`, `LAB_005fe0e5`). This
crate did not carry that kill either. It is in `do_attack` now, ahead of
the stance and the chase, and it is why both orders are gone on 864 on
both sides.

### 57.4 The round: the order's point, not the target's

The shot is the release event's (§9.0), 17 blocks after the swing: 798
in the dump. `Guy::execute_events@005d99c0`'s `0x17`/`0x18` arm gives the
event package `ox = whom = −1`, and `GraphicEvents::execute_game_events
@008e48e0`'s gate passes it on `order_type() == ATTACK_GROUND`.
`Ammo::init@0067bbf0` then reads the shooter's current order as an
attack-ground order (`local_38`, vslot `+0xd0` on `get_order`, `init:262`)
and, when it is one:

- the accuracy is taken against the plain distance to the point (the
  `vector_dist` pair is the register trap, read as the launch-to-point
  delta §9.1 already used);
- the scatter is the land-unit formula, or none when the order's
  `accuracy` is set (`init:365`);
- the landing is `att_x/att_y` plus the scatter's two draws (`init:461`–
  `484`), and `ez` is `find_data_z` at the point, clamped at 0 (`:486`).

This crate decided ground fire by the type (a siege packer at a unit) and
aimed at the target's position on the release frame, 17 blocks after the
original fixed its point. Now `Sim::fire_ammo_aim` takes an `Aim`, and
`Aim::Ground` carries the order. `guy_release_events` fires it when the
head is `ATTACK_GROUND`, and `do_attack_ground` fires it directly for a
unit whose animation launches nothing. `ez` for a ground shot is
`ground_z(point)`, not the target tile's `tile_z`.

### 57.5 What moved

| | before | after |
|---|---|---|
| the restage, run146: word / sequence / values | 782 / 782 / 783 | 782 / 782 / 783 |
| run146 widening rows on `0/6`, 781–783 | 18 | 3: 617's numbering on the attack beneath |
| first value parting, run146 | 736 | 736 |
| chapter three (run145) | 900, closed | 900, closed |

**The value diff** (`chapter_three_s_catapult_fires_on_the_ground`): on
every block from 779 to 864, `0/6`'s order list (kind, point, `accuracy`,
`attack_unit`, signed flags, or `in_range`/`new_ord` for the attack) and
its reload read the dump's. The round is on 798 on both sides, with no
object, `ez 188`, accuracy 5 and flight 33. It was made to fail with the
arm off: one order, `new_ord 0`, reload 82. The harness compares the
order's row (`order:ground.*`) and reads `flags` signed. The unit test
`an_unpacked_siege_engine_fires_on_the_ground_under_its_target` was made
to fail the same way.

### 57.6 The word did not move: the crew walks where the original's mirrors

782 is not the ground order. `RON_GOLDEN_SITES` attributes this crate's
two extra `Guy::set_anim+0x97a < Guy::inc_time+0x271` to `0/6` itself,
its two crew figures, and the `set_anim+0x104b` both sides spend to a
bird (`9/4`). On the push block the catapult's figure 0 takes
`TURN_LEFT` (21) on both sides and owes its swing (§6.2's deferral). This
crate's crew take `WALK` (8), whose three frames wrap on 782 and roll
the idle.

run44's `0/15` is the same type (265) with the figure clock printed. On
its push (324) all three figures read `cur_anim 21`, `cur_time 1`, and
the crew keep `end_time 79`, which is the mirror's copy of anim and time
and not a `set_anim`. Every block from 324 to 330 the crew stand **on
their destinations** (`x == des_x`) while those move round the turning
catapult. So the original's crew are never walking during a turn in
place, and are mirrored. This crate's move toward the rotated slot by
`follower_step` and start the walk. `Guy::move@005d9240`'s tracked arm
and `Guy::inc_time@005d9e10`'s mirror are the reading this needs, and it
is its own item. **Item 625**: "on their destinations" is the block's
end, after the step. The crew step every block onto their new
destinations (run44's `last_speed` 10 and 20), and what differs is the
slot, not the step. §58 has it.

Three residues past the word. The first two are pinned in the value test:

- **The launch**: this crate's round leaves the unit's square, at `sz
  251`. The dump's leaves the release node, (855, 7990, 496).
  `sim::launch` has no node for the catapult's piece (§22's seam).
- **The landing**: the point plus a scatter drawn on 798, after the
  stream has parted, so ours is (2655, 8142) against (2413, 8276).
- ~~**865**: this crate's catapult takes a fresh attack after the reload,
  and a chase on 866. The dump's holds nothing until 868.~~ An unpacked
  packer's search must reach what it takes (§60, item 627).

### 57.7 What is not established

- **The packed arm of `fight`'s in-range branch** (the unpack, and the
  entrenched engine's better spot). No capture on this disk reaches it
  with a human's engine. run44's AI engines do (`CASTORDER` over an
  attack on 245), and that is not a golden window.
- **A player's ground order** (`attack_unit 0`): the peace test and the
  walk into range. No command here issues one.
- **`unit_masks |= 0x11000`** is not kept by this crate, which is the
  same as its strike.
- **The rush-rules gate** in `can_attack_ground`.

### 57.8 Coverage

- **Diff-backed**: the arm's push (kind, point, `accuracy`,
  `attack_unit`, fired bit, the attack's `new_ord`) and the same-frame
  shot, the 83-block hold, the double kill on 864, and the ground
  round's frame, object, `ez`, accuracy and flight (run146, above). The
  point's source, the target's position rather than its cell, is backed
  by run44's three engines, read from the dump.
- **Listing-backed**: `can_attack_ground`'s terms (`61db20`–`61dc13`).
- **Reading only**: the vtable name of `+0x188` (`Unit::work`, from the
  export's `Unit::vftable`), the release gate's `0x17`/`0x18` arm, the
  packed arm, and the player's-order arms.

## 58. An unpacked packer's moving figure asks for no walk (item 625, 2026-09-23)

Golden chapter three's restage (run146, `docs/GOLDEN.md` §7) stood at
**782** after §57: ours 7 draws against 5 at draw 0, two `Guy::set_anim
+0x97a < Guy::inc_time+0x271` on the catapult `0/6`'s crew. The item
was booked with a hypothesis, `Guy::move`'s tracked arm and
`Guy::inc_time`'s mirror, and no mechanism.
`docs/journal/2026-09-23-item-625.md` has the kill conditions, written
before the reading.

### 58.1 The disk: the crew step, and their clock is copied

run146 prints no figure clock (`GUYS=2`), and its crew's positions agree
with ours on every block of the turn: they move, 781–785, by the same
steps. run44's human catapult `0/15` is the same type (265) turning in
place on its ground order's push (324), at `GUYS=4`
(`a_turning_catapult_s_crew_mirror_and_never_walk`):

- On 324 both crew figures **step** (`last_speed` 10 and 20) onto their
  new destinations. So they enter `Guy::move`, and its tracked arm does
  not refuse them. `x == des_x` at the block's end is the Manhattan snap.
- From 324 to 330 all three figures play `TURN_LEFT` (21), and the crew's
  `cur_time` is figure 0's. The crew's `end_time` stays 79, the idle's,
  where figure 0's is 30, and their `last_time` stays **−1** on every
  block. A `set_anim` would write both, and a step of their own clock
  would overwrite `last_time`. So their clock never steps: it is copied
  by `Guy::inc_time`'s crew arm (`docs/ANIM.md` §5).
- **Across the whole of run44**, every stepping crew figure of an
  unpacked packer (265 or 266, `unit_masks & 0x80000` clear) is on a
  turn slot: 92 records on 21, 22 on 22, none on the walk category.

The mirror was already carried here. It refuses a crew figure on the
walk category, so the question was who put ours on the walk.

### 58.2 `Guy::move@005d9240:176–181`: the walk's gate

The moving arm (the body is off its destination) chooses the walk slot
and asks for it, but not always. The listing (`5d9565`–`5d958a`):

```
if guy_flags & 0x40:                      skip          # a plane
if unit_flags2 & 4 and !(unit_masks & 0x80000):  skip   # packs, unpacked
set_anim(walk, 0, 1)
```

`0x68` on the unit is `unit_masks` and `0x2b8` on the type is
`unit_flags2`; `& 4` is the packing bit this crate already reads as
`Profile::packs`, and `unit_masks & 0x80000` is packed. An unpacked
engine cannot move. What moves is its crew, pulled round to their
rotated slots by figure 0's turn (`docs/MOVEMENT.md`, "Who writes it,
and when"). They keep the slot they had, the crew arm of
`Guy::inc_time` copies figure 0's `TURN_LEFT` and time into them on the
same frame, and no draw is spent.

`Sim::guy_follow_anim`'s moving arm asked for the walk for every figure,
gated only by the turn slots. On 781 the crew took `CHAR_WALK`, three
frames long, stepped their own clock, and wrapped on 782 into two idle
rolls. The gate is in that arm now, with the plane read as `is_plane`
(as `set_anim`'s deferral already reads `guy_flags & 0x40`).

### 58.3 What moved

| | before | after |
|---|---|---|
| the restage, run146: word / sequence / values | 782 / 782 / 783 | **792** / 792 / 793 |
| the round's landing on 798 | (2655, 8142) | (2413, 8276), the dump's |
| run146 widening rows past the numbering on the word's blocks | none | none |
| chapter three (run145) | 900, closed | 900, closed |

**The value diff** (`chapter_three_s_crew_mirror_the_turn`): on 781–785
both crew figures stand on the dump's points (`(938, 7883)` and `(660,
7965)` on 781), and play figure 0's slot, 21, on its clock. It was made
to fail with the gate off: slot 8, not mirrored, on 781. The unit test
`an_unpacked_packer_s_moving_figure_asks_for_no_walk` was made to fail
the same way. The round's scatter draws are taken on 798 inside the
stream now, so its landing agrees
(`chapter_three_s_catapult_fires_on_the_ground`); its launch is still the
unit's square (§57.6).

### 58.4 The new word is the numbering

On 792 the original spends, past the birds, one `Unit::fight+0x9b0 <
Unit::do_attack`, and this crate goes straight to the farms. On that
block the dump's arena-A hoplites `1/9`–`1/11` hold a fresh attack on
the catapult, their `idle` at 4. Ours take it on 795. `Unit::check_idle`
adds one every sixteen frames on `(frame + o) & 15 == 0`: the dump's
`1/11`, `1/10` and `1/9` reach 4 on 790, 791 and 792, and ours `1/8`,
`1/7` and `1/6` on 793, 794 and 795. Both are `frame + o = 800`. This
crate numbers the three 6–8 where the dump numbers them 9–11, which is
the `DEATH_OBJS` cull on 771 (§42.5) that parked 617 named as spending
no draw. It spends one now. The widening cannot see the rows: the
numbering leaves the three unlinked both sides, so no field of theirs is
compared.

### 58.5 What is not established

- **A packed packer's moving figure** takes the walk, as the gate says.
  No capture on this disk moves a packed catapult with its clock
  printed (run44's packers never step packed), so that half is the
  listing's alone. Merchants and fishing boats reach it on the long
  captures, and those hold.
- **The plane half** of the gate is read and carried, and no capture
  here reaches it.

### 58.6 Coverage

- **Diff-backed**: the crew's positions on the turn (run146, `g.x`/`g.y`),
  the draw stream to 792, and the round's landing.
- **Dump-backed, read off the original's own records**: the crew's
  slot, clock and `end_time`/`last_time` on a turn in place (run44
  324–330), and no walk slot on any stepping crew figure of an unpacked
  packer in run44.
- **Listing-backed**: the gate's three terms (`5d9565`–`5d9581`).

## 59. A dead number is held while its death object lives (item 617, 2026-09-23)

Golden chapter three's restage (run146, `docs/GOLDEN.md` §7) stood at
**792** after §58: ours 28 draws against 29 at draw 24. The dump's
arena-A hoplites took their attack on the catapult on 792, and ours
took it on 795. The idle is phased `(frame + o) & 15`, and ours
numbered the three 6–8 where the dump numbered them 9–11. The item was
booked with §42.5's cull as its hypothesis.
`docs/journal/2026-09-23-item-617.md` has the kill conditions, written
before the reading.

### 59.1 The floor: no cull, and the allocator parts

`chapter_three_s_restage_numbers_its_objects` walks run146 whole, both
directions. It pairs every birth on its frame, owner and point, every
death on its number, every `DEATH_OBJS` record on `(who, o,
first_frame)`, and every live round on its pool slot:

- The deaths agree: `1/6` on 680, `1/7` on 728, `1/8` on 736.
- The death-object list agrees on every block. The three records arrive
  on those blocks and **none leaves** before the capture ends, on either
  side. So nothing is culled on 771.
- On 771, arena A's three hoplites are born on the same three points.
  The dump numbers them 9–11 and ours numbered them 6–8. Player 1's uids
  run 12–14 for the dead and 15–17 for the new, so the original made no
  allocation between them.

So the original does not hand out a number whose death object still
lives, and this crate did.

### 59.2 `Objects::find_free@0065ad60`'s reuse test

`find_free` scans the band `[base, mark)` for a reusable number before
it takes the mark. A number is reusable when all of these hold:

- the object is dead (`flags & 1` clear);
- **`hold_frames` (`+0x32`) is zero**;
- it is not a unit (vslot `+0x18`), or its `o_up` (`UnitData +0x8e`)
  is negative.

This crate's `Sim::find_free` tested the first condition only.

### 59.3 Who holds it: `DeathObj::inc_time@008d5240` and `process_all`

Two writers keep a dead number's `hold_frames` above zero for as long as
its death object lives:

- **`DeathObj::inc_time`'s first statement** adds one to the dead
  object's `hold_frames` every frame. It runs from `Objects::inc_time`
  for every live record, after `Objects::process_all`. For a type whose
  `UnitTypeData::blocks_while_dead` is set (`unit_flags & 0x800000`,
  vslot `+0x120` on `ptype`, named from the PDB's field list), it also
  raises the hold to at least 30 while the record lives.
- **`Objects::process_all@0065dce0`** walks each player's objects in
  order. A dead one (`flags & 1` clear) whose hold is not zero loses one.

`Object::die` sets the hold to at least 1 (§11). So at every command
point in the frame (`NetDaemon::process_all`, `docs/ORDERS.md` §2.1),
the hold of a number whose death object lives is at least 1. Once the
record ends (`*this = 0`), nothing adds to the hold, and `process_all`
takes the rest off.

This crate carried the bump in `hold_frames_tick`, but looked its unit
up with `unit_by_o`, which finds only the living, so the bump never
landed. It had no decrement. Now:

- the bump lands on the slot's latest occupant, dead or not;
- the unit loop takes one off a dead occupant at its visit;
- `find_free` refuses a dead number whose hold is not zero.

**A third writer holds every dead number thirty frames** (item 1197,
2026-09-29). `Object::close@00647160`, at the foot of `Unit::close`,
ends each close of an active object with `Objects::remove@00658980` and
then `hold_frames = 0x1e`. Both of its arms reach that write: the one
that takes the object out of the world, and the `blocks_while_dead` arm
that leaves it there. `Object::die` calls `close` through vslot `+0x150`
first, so its `max(1, …)` is taken against 30. A death that makes no
death object therefore still holds its number thirty frames. That covers
`dtype` 0 (attrition, `disembark`'s `Object::die(boat, 0, −1, 0)`, the
upgrade's squad trim through `Unit::die`, vslot `+0x158`), an aircraft
that crashes, and a dock's gull, which `Dock::close` shuts with
`Unit::close(0, −1, 0)`. This crate held only a combat death's number,
and only at 1. Now every kill site sets `Sim::CLOSE_HOLD`, and
`hold_dead_slot` maxes the ammo terms against it.

The dump bounds it. On East Indies at Toughest the gap 7634..8513 is
undumped, but run439's first block prints player 1's units with their
`uid`s, which run in birth order. This crate's walk agrees with the
draw stream through the gap and makes the same births on the same
frames. Two transports die there putting a passenger ashore: the
Transport Barge `1/62` (a Cataphract) on frame 8195, and the Merchant
Fleet `1/59` (a Caravan) on 8411. The original's next births skip both
numbers. uid 107, born on 8210, is `1/63`, and uids 113–116, born on 8423
and 8429, are `1/68`–`1/71`. uid 111 has 62 by 8248. Inside run439, uid
118 takes 59 on 8627, and the barge `1/56`, dead on 8519, gives its
number to uid 117 on 8562. So a number is still held 15 and 18 frames
after its death, and free 43 frames after.

### 59.4 What moved

| | before | after |
|---|---|---|
| the restage, run146: word / sequence / values | 792 / 792 / 793 | **865** / 865 / 866 |
| arena A's hoplites, 771 | 6–8 | 9–11, the dump's |
| run146's numbering rows (births, deaths, `DEATH_OBJS`) | 3 | 0 |
| chapter three (run145) | 900, closed | 900, closed |

**The value diff** (`chapter_three_s_arena_a_hoplites_take_the_dump_s_numbers`):
on 790–793, `1/9`–`1/11` stand on the dump's points, with the dump's
`idle` and order list on both sides. They reach `idle 4` on 790, 791 and
792, and hold the `ATTACKORDER` on 792. The test was made to fail with
the hold test off in `find_free`. The unit test
`a_dead_unit_s_number_is_held_while_its_death_object_lives` was made to
fail the same way.

### 59.5 The ground shot's scatter has its own two sites

With the numbering fixed, the sequence word first parted on 797, with
the same values on both sides. Ours labelled the catapult's ground
round's scatter `Ammo::init+0xcd9`/`+0xd0b`, and the original's read
as a bare `67c6d8`/`67c715`. The listing puts the attack-ground arm's
own pair of `Random::get` calls at `67c6d3` and `67c710`, ahead of
`find_data_z` at `67c738`, so their return addresses are
`Ammo::init+0xae8` and `+0xb25`. The arithmetic is the target arm's,
`point − s/2 + roll % s`. Now both are named, in the trace table and in
`scatter_landing`. The values already agreed (§58.3).

### 59.6 The new word is 621's park

On 865 this crate's catapult, its ground order's reload run out, takes a
fresh attack and spends `Unit::fight+0x9b0`. The dump's holds nothing
until 868. That is §57.6's third residue, closed by §60. Below it, the widening
now links arena A's hoplites and shows what the numbering had hidden:

- on 798, the round's pool slot is 0 here and 1 in the dump, run145's
  family (§55.5);
- on 847, the scout `1/0`'s fresh `EXPLORETOORDER` move reads `facing 1`
  here and 0 in the dump. No draw is spent.

### 59.7 What is not established

- **The thirty frames' own edge** (§59.3, item 1197). The listing
  writes `0x1e` and the dump bounds the hold between 18 and 43 frames.
  No birth on disk falls on the 30th or 31st frame after a death, so
  the frame the number comes free on, against `process_all`'s visit
  order, is read and not diffed.
- **The cull itself.** Nothing ends a death object in this crate, so a
  dead number here is now held for good. The original frees it once the
  record ends. For a land unit with no corpse piece (`DeathObj +0x44 ==
  −1`), that is after `get_game_frames(cur_anim) + body_fade_end` frames
  (135). With a corpse piece it is after `+ corpse_fade_end` (627). A sea
  unit ends at its animation's end. The two constants are `.rdata`
  words (`0002:378732`, `0002:378740`). Building the cull needs the
  death piece's animation packet (`gpiece`, which this crate does not
  load) and `+0x44` from `DeathObj::init`. run146's three records stay
  live 263 blocks or more, which rules out 135 plus a short animation.
  So the hoplite has a corpse piece. run437 is the first capture on disk to
  reach a cull (item 1200): `0/7`'s record (`cur_anim` 17, first frame
  978) leaves the list on frame 1621 and `0/8`'s (`cur_anim` 18, 1068) on
  1724, 643 and 656 frames on, so over `corpse_fade_end`'s 627 the Hoplite
  death piece's two packets run 16 and 29 frames. Chapter forty-one's
  widening carries both as `death:extra`. No golden window has a birth that a
  cull would have freed a number for. The long captures' words hold under
  the grow-only hold (the item's journal has the gate).
- **`o_up`**, the third term of `find_free`'s test: a dead squad member
  still linked in its chain is not reused. This crate does not test it.
- **`blocks_while_dead`**'s floor of 30. It is not carried, because a
  live record already holds the number, and the floor only matters after
  the record ends, in the cull this crate does not have.
- The within-frame order: a unit killed before its visit loses one on
  the same frame, so a building that trains later in the same
  `process_all` could reuse it. No capture reaches that.

### 59.8 Coverage

- **Diff-backed**: the deaths, the death-object list and every birth's
  number over run146 whole, and the three hoplites' `idle` and orders on
  the word's frame.
- **Listing-backed**: the ground scatter's two sites (`67c6d3`,
  `67c710`).
- **Reading only**: `find_free`'s three terms, the bump and the
  decrement (§59.3), the cull's duration (§59.7), and `+0x120`'s name,
  which comes from the PDB's `LF_ONEMETHOD`.

## 60. An unpacked packer takes only what it can reach (item 627, 2026-09-23)

Golden chapter three's restage (run146, `docs/GOLDEN.md` §7) stood at
**865** after §59. Ours spent 5 draws against 4, parting at draw 0:
`Unit::fight+0x9b0` on the catapult `0/6`, whose ground order's reload
had just run out. The dump's catapult held nothing until 868. The item
was booked with no mechanism. `docs/journal/2026-09-23-item-627.md` has
the kill conditions, written before the reading.

### 60.1 The floor, and what it named

`chapter_three_s_catapult_after_its_reload` walks 858–880 on both sides:
`0/6`'s order list, its `idle`, reload, point and `orders_x/y`, and arena
A's three hoplites.

- Both sides drop the ground order and the attack beneath on 864, the
  ready block (§57.3).
- The hoplites `1/9`–`1/11` stand still on both sides, 339, 396 and 413
  units off. All three are inside the catapult's 570 minimum.
- `idle` reads 1 on 865 on both sides. The think's first idle frame
  (tick 864) ran on both. Ours took an attack on `1/10` there, with no
  draw, and spent the re-search on 865. The dump's took nothing.
- The dump's catapult takes an attack **only on the block after a hit**:
  868, 870 and 871, after the hits that take `damage` 12 → 16 → 20 → 24.
  Each attack reads `in_range 0` and is gone the next block, and the
  catapult never moves. On tick 868 the original spends one
  `Unit::fight+0x9b0` and nothing after it. Ours, given the same
  attack, chased away for its range.

So two questions: why the idle search finds nothing, and why the
retaliation ends without a chase.

### 60.2 `Object::find_nearby_target@00648da0`'s `local_24`

`local_24` is set, from the listing, at `64911b`–`64918d`:

- the unit's combat stance (vslot `+0xf4`) is 2, STAND_GROUND; or
- `unit_masks & 0x2000000` with `unit_masks2 & 0x20000` clear (entrenched
  without the Antipater bit); or
- the type packs (`+0x2b8 & 4`) and the unit is **unpacked**
  (`unit_masks & 0x80000` clear).

A computer's packed siege engine sets it too, and sets `local_5c` beside
it (`6491c6`–`6491cd`). A building sets it, and so does the cavalry
archer's second-weapon search.

The range gate (`6495c2`–`64963b`) reads it:

```text
if !local_24 && (!guarding || target has attack ...):
    in_range = 1                         # deemed, untested
else:
    if !is_in_range(this, candidate, own x, own y):
        if !local_5c or candidate is a unit or ...: skip
    in_range = 0
```

So **a searcher with `local_24` must reach what it takes**.
`ObjectData::is_in_range@006486b0` tests the minimum range, so an
unpacked catapult's idle search skips every hoplite inside three tiles.
This crate had the flag the other way round (§12.2, struck there). It
read these units as the ones that "consider anything", and let every
unit take anything. `compare_target`'s `/5` is never reached for a
candidate already tested in range, so the only effect is the skip.

### 60.3 `Unit::fight@005fd4d0:1051`: a packer re-searches before it chases

The chase tail opens:

```text
if (!packs || param_3 || (t = find_new_target(this, &who, 0)) >= 0 && who >= 0)
   && !param_4:
    stand-ground / entrenched hold, or find_attack_pos and the chase, on t
return 0
```

`Unit::find_new_target@005ff6a0` kills the current order (or the
group's), then runs `find_melee_target(-1, &who, 0, 1, 0)`, which adds
the attack it finds. For a packer, then, the tail goes on only with a
fresh target from the idle search. For an unpacked one, §60.2 means that
target is in range. A catapult hit from inside its minimum takes the
retaliation, drops it on its next frame, finds nothing and stays put:
run146's 868, 870 and 871. The draw that frame spends is the attack's
own re-search (`fight+0x9b0`, §8.2 step 0), ahead of both.

The unpacked packer's ever-in-range kill (§57.3, `LAB_005fe0e5`) comes
before the tail, and it still stands.

### 60.4 The build, and what moved

- `Sim::find_nearby_target`: `must_reach` is `local_24` for a unit.
  Those units test `is_in_range` and skip a miss, and they score with
  `in_range = false`.
- `Sim::do_attack`'s tail: a packer kills the attack and runs
  `find_melee_target(u, -1)`, then returns if it finds nothing. If it
  finds something, it adds the order (`QUEUE_FIRST` in DEFENSIVE, else
  `QUEUE_NEW`, `find_nearby_target`'s own rule) and chases that target.

| | before | after |
|---|---|---|
| the restage, run146: word / sequence / values | 865 / 865 / 866 | **1000 / 1000 / none**: closed |
| run146's widening rows on 865–866 | 20 on `0/6` | 0 |
| ours' second round, 960 | fired | gone |
| chapter three (run145) | 900, closed | 900, closed |

With the search gate alone the word stood at 870, where the dump's
catapult takes its third retaliation and ours was still chasing from 868.
The re-search took it to the end.

**The value diff** is `chapter_three_s_catapult_after_its_reload`. On
every block 858–880, `0/6`'s order list, `idle`, reload, point and
`orders_x/y` read the dump's on both sides, and so do the hoplites'
points. It also pins the dump's own shape: one attack each on 868, 870
and 871, on `1/9`, `1/11` and `1/10`. It was made to fail with the gate
read back: 865 parts on the order list. The unit test
`an_unpacked_packer_takes_only_what_it_can_reach` was made to fail both
ways. With the gate read back, the search names a foe inside the
minimum. With the re-search off, the retaliation keeps its order.

### 60.5 What is not established

- **`local_5c`**, a computer's packed siege engine, which keeps an
  out-of-range candidate when the candidate is not a unit. This crate
  does not set it. No capture on disk has a computer's packed engine
  searching.
- **The building's gate.** A building sets `local_24` with `local_5c`
  clear, so from the listing it skips every out-of-range candidate. This
  crate keeps an out-of-range unit or armed target for a building with
  `in_range = false` (§12.2's "or be a unit, have attack"). That reading
  is not changed here, and it wants its own measurement.
- **The guarding arm** (`local_2c`, activity 12) of the same gate.
- **The found-target branch of §60.3** for a packer, meaning the
  stand-ground hold, `find_attack_pos` and the chase on the new target.
  No capture reaches it with a packer, so it runs the arm this crate
  already had.
- `find_new_target`'s DEFENSIVE `find_def_pos` arm, its `repath`, the
  group-order kill, and the leader's `searches` count.

### 60.6 Coverage

- **Diff-backed**: the idle search's refusal on tick 864, the three
  retaliations and their one-block life, and the catapult that never
  moves (run146, 858–880 and whole to 1000).
- **Listing-backed**: `local_24`'s three terms and the gate's skip
  (`64911b`–`64918d`, `6495c2`–`64963b`).
- **Reading only**: `find_new_target`'s kill-then-search and the tail's
  packer test (`fight:1051`), and every arm §60.5 lists.

## 61. An aircraft takes no aircraft it cannot reach (item 650, 2026-09-23)

Golden chapter six (run168, `docs/GOLDEN.md` §10) stood at **616**. Ours
spent 25 draws against 24, parting at draw 18 on a `Unit::fight+0x9b0` of
the Bomber `1/6`'s. Ours' idle search had taken the Fighter `0/6` on the
Bomber's birth tick, and the dump's Bomber holds no order from birth to
899. The item was booked with no mechanism. The kill conditions for three
readings are in `docs/journal/2026-09-23-item-650.md`, written before the
build: the searcher refused, the target refused, and §60.2's reach gate.

### 61.1 The floor

`chapter_six_s_word_frame_is_widened_whole` already read every record on
616 and 617 both ways, and the coverage pin held on 614–618. No field the
dump prints was unread, so no widening came before the reading. The
aircraft's rows on the dump are `idle` 1 on 616 and 2 on 617, `orders_x`
at their own seats and `air_alt` 0, both sides. The Bomber's rules row
is `FLY_HIGH 0`, `FLY_LOW 10%`, `OBJ_MASK BSX3T` (no `6`, no `ANTI_AIR`)
and `RANGE 1-3`. The Fighter's is `FLY_HIGH 0`, `FLY_LOW 25%`, `63TG`
(`ANTI_AIR`) and `2-7`. Their seats are 1536 apart, eight tiles.

### 61.2 `ObjectData::valid_target_const@006472c0`'s air ladder

For an air target (`domain` 2), below the searcher's `max_range != 0`,
the listing `006474c3`–`0064771c` reads, for a target that is not a
helicopter (`unit_flags & 0x20`):

```text
if T.is(0x132) && S.domain != AIR && !can_carry(S, AIR):   refuse
if S.has_objmask(ANTI_AIR) && S.domain == AIR:             → missile tests
if S.fly_high == 0 && S.fly_low == 0:                      refuse
if !S.has_objmask(ANTI_AIR):                               # the target's own
    if T.fly_high == 0:
        if T.fly_low == 0 or is_flying_high(T):            refuse
    elif T.fly_low == 0 and is_flying_low(T):              refuse
if S.fly_high == 0:  if is_flying_high(T):                 refuse
elif S.fly_low == 0: if is_flying_low(T):                  refuse
missile tests: T or S obj_masks & 0x8000000 →             refuse
```

`fly_high`/`fly_low` are `ObjectTypeData +0x250/+0x254`, the `FLY_HIGH`
and `FLY_LOW` columns, read by `UnitType::init@0061ab50:628`–`635` and
`BuildType::init@00632340:319`–`326` as `get_text_num(…, -1)`. Nothing
else writes them. The searcher's type is read through `objects[who][o]`
(`0xc0ab84`) and the target's through `units[who][o]` (`0xc0aec0`). The
decompiler has both right.

**`UnitData::is_flying_low@0060a140`** is 0 unless the unit is a fixed-wing,
non-missile aircraft on the map holding an order. Then it is 1 on a
`STRAFE` or `AIR_ATTACK_GROUND` within `0x900` of its point, or on an order
whose slot `0xfc` hands back a live target within `0x900`. Slot `0xfc` is
`AirOrder::get_air_order` on `AirOrder`, `AirPatrolOrder` and
`AirAttackGroundOrder`, and `Window::get_button`, a folded `return 0`, on
every other class. **`is_flying_high@0060a310`** is the same three type
tests, on the map, and not low. So an aircraft is high unless an air order
is bringing it down to its target. This crate gives no player's unit an
air order, so every plane on the map is high.

The Bomber (`FLY_HIGH` 0, not `ANTI_AIR`) is therefore refused the Fighter
twice: on the target's own row (`T.fly_high == 0`, and high), and on its
own (`S.fly_high == 0`, and high). The Fighter, an `ANTI_AIR` aircraft,
passes straight to the missile tests.

### 61.3 `Object::poor_target@0064a270`'s plane arm

The Fighter may take the Bomber, and the original's never does. Its
periodic think runs on tick 634, `(o + frame) & 31 == 0`, and ours took
the Bomber there once §61.2 stood (the word went to 635). What refuses is
`check_target@00649e00`'s tail, which `find_nearby_target` calls with
`use_poor` 1 and which falls through to `poor_target`. Its first test is
at `0064a2c2`–`0064a2ca`:

```text
guy = units[who][o]->guys.list[0]           # Unit +0xf4: PtrArray<Guy> +0xe4, list +0x10
if guy->guy_flags & 0x40:
    if !this->has_objmask(ANTI_AIR):  return 1
    return attack_dist(this, o, who) > this->max_range() * 0xc0
... the move arm (§37.3)
```

`guy_flags & 0x40` is set in `Guy::init_real@005db6b0` on every figure of
a unit `UnitData::is_plane` accepts, an air-domain type without
`unit_flags & 0x20`. `Guy::clear` and `init_real`'s own reset zero the
word, and nothing else clears the bit. So the arm is the target
type's: **a plane is a futile target** for anything without `ANTI_AIR`,
and for an `ANTI_AIR` searcher beyond its own reach. The Fighter's reach
is seven tiles and the Bomber is eight away, so it never takes it.
~~§30.5 read this byte as `UnitTypeData +0x9a`~~: it is the first figure's
`guy_flags`, which the type record (`UnitData +0xe4 guys`) settles.

`check_target`'s tail comes before `find_nearby_target`'s `near_o`
write, so a refused plane never becomes `near_o` either.

### 61.4 The build, and what moved

- `Profile::fly_high`/`fly_low`, loaded for units and buildings with the
  `-1` default.
- `Sim::valid_target`: the ladder above, for a target `Sim::is_plane`
  accepts. `is_flying_low` is false, which is exact for the orders this
  crate carries, and `is_flying_high` is "not a missile".
- `Sim::poor_target`: the plane arm, which was a stated seam (§37.3).
- `Sim::find_nearby_target`: `poor_target` after `valid_target` and above
  `near`, as `check_target`'s tail. **This also puts §37.3's move arm into
  the ring search**, where the original has it and this crate did not.

| | before | ladder only | plane arm only | both |
|---|---|---|---|---|
| chapter six, run168 | 616 | 635 | 700 | **700** |

`bird` is `CHAPTER_DEBT` (parked 652), and 700 is its frame. There the
original's draw 0 is the bird's `Guy::init_real+0x52`, and on 701 the AI
scout `1/0`'s `think_scout` roll lands one draw early. Block 700 agrees on
every record, and every block from 606 to 700 past the aircraft's `form`.
The two long words and the closed chapters are in the item's journal.

**The value diff** is `chapter_six_s_word_frame_is_widened_whole` over
(605, 702). Neither aircraft holds an order on any block to 700 on either
side, `orders_x/y` stay at their seats, and `idle` climbs alike, 8 on
700. The unit test `an_aircraft_takes_no_aircraft_it_cannot_reach` was
made to fail both ways: with the ladder read back the Bomber's
`valid_target` is true, and with the plane arm off the Fighter's search
names the Bomber at eight tiles.

### 61.5 What is not established

- **The helicopter arm** (`006474ee`): a helicopter target refuses a
  searcher by two type vtable slots (`+0x10c`, `+0x110`), a missile, and
  `is(0x130, 0)`. This crate keeps "ranged and not a missile" for it.
- **`is(0x132, 0)`**: a non-air searcher that cannot carry aircraft is
  refused such a target. The id is 306 by the dump's type numbering
  (Fighter 289 and Bomber 304 are XML rows 239 and 254), which would be
  the Stealth Bomber. Not built.
- **An aircraft under an air order** is low, and the ladder's
  `is_flying_low` arms then matter. This crate has no player air orders.
- **The Bomber's own refusal is over-determined** on run168. The ladder
  and the plane arm each refuse it, so this capture cannot tell them
  apart.
- **`check_target`'s other gates** in the ring search are still not built:
  the region test and the DEFENSIVE-with-an-order test (§12.2).

### 61.6 Coverage

- **Diff-backed** (run168, 605–702): neither aircraft takes the other.
  For the Fighter this backs `poor_target`'s plane arm and its reach floor
  alone. For the Bomber it backs "one of the two gates".
- **Listing-backed**: the ladder (`006474c3`–`0064771c`); `is_flying_low`'s
  slot `0xfc` (`0060a27d`) and the classes that override it; the plane
  arm's read of `guys.list[0]->guy_flags` (`0064a2c2`–`0064a2ca`).
- **Reading only**: every §61.5 arm.

## 62. A captain's attack on a building re-searches every frame (item 680, 2026-09-24)

Golden chapter six-b (run175, `docs/GOLDEN.md` §10) stood at **632**. Ours
spent 34 draws against 24, parting at draw 18 on ten
`Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4` of the Fighter `0/6`'s,
at its attack point (600, 7944) under an attack on who=1's Airbase `2006`
at (2400, 6432). The original's stack is empty on block 633. Item 651
booked it as "which exit of `find_attack_pos@00601280` answers 0", after
`fight@005fd4d0:1059`, and named no mechanism. The kill conditions for
four readings are in `docs/journal/2026-09-24-item-680.md`, committed
before any function ran on the packet.

### 62.1 The packet, and what it killed

run177 is a `RON_STATE_FRAME=632` packet on run175's staging
(`docs/RUNS.md`): the state after tick 631, with the Fighter at its point
and `fight` still to run. Two calls on it under unicorn, with every block
and call recorded (the scripts are outside git, under
`~/ron-data/lab-experiments/2026-09-24-item-680/`):

- **`find_attack_pos(0/6, 2006, 1, 0, &x, &y, 0)`** through the thunk
  `@00602e60`, exactly `fight+0xcb4`'s call: it answers **1**, out (600,
  7944), with **ten** `Random::get` returns at `+0xea9`, the ring walk this
  crate spends. So no exit of it answers 0 here, and the booking's
  premise dies: the original never makes this call on 632.
- **`Unit::fight(0/6, 2006, 1, 0, 0, 0)`**, `Unit::do_attack@005f1b80`'s
  arguments (the order's `mandatory` 0): it returns 0 with **no draw**,
  never enters `find_attack_pos`, and calls `Unit::find_new_target@005ff6a0`
  once, from `fight+0xa1f`.

Reading 0 of the journal, "`find_attack_pos` is never called", survives.
Readings 1–3 (an air-domain exit, a building footprint arm, a minimum
range test) name exits of a function that is not reached, so none can fire
on this frame.

### 62.2 `fight`'s captain arm, `LAB_005fddf7`, from the listing

The executed path is `005fdd5b` … `005fe012`:

```text
if mandatory == 0 && param_4 == 0 && recharging == 0          ; 5fdd5b-5fdd76
  && is_captain()                                             ; 5fdd7c-5fdd9b, o_up >> 15
  && (cavarch || !order.is_group() || order.group_leader == me)  ; 5fdda1-5fddf1
  local_14 = 0
  if target.is_unit():                                        ; vtable +0x18, 5fde1e
      local_14 = the type test; roll; roll % 5 == 0 or flags & 0x10 → 0   ; §8.2 step 0, §47.1
  if !target.is_unit() || local_14:                           ; 5fdeb1-5fdec0 → 5fdf50
      t, who = find_new_target(this, &who, 0)                 ; 5fdee2-5fdeea (cavarch: find_melee_target)
      if (t, who) != (o, whom):
          t < 0 or who < 0          → return 0                ; LAB_005fe001
          order_type != ATTACK      → return 0
          recharging                → return 0
          (t, who) == fight's own arguments → return 0        ; local_20/local_24
          the order's target := (t, who, uid); unit_masks2 |= 0x10; return 0   ; 5fdf99-5fdfee
      t < 0 or who < 0              → return 0
      goto LAB_005fe01d            ; on to the range test and the chase, on the order the search added
  elif poor_target(o, whom):       → the same search          ; 5fded5
```

**For a building target there is no roll and no `poor_target`**: the
search runs on every frame the captain's attack runs, and it kills the
order first (`find_new_target` is `repath`, `kill_current_order`, and
`find_melee_target(-1)`, which adds what it finds; §60.3). Vtable `+0x18`
is `Buffer::is_pending_load` after COMDAT folding, a `return 1` on the
unit classes (§47.1).

**Why 632 and not before.** `find_melee_target`'s radius for a human's
ranged unit is `max((r + 1) × 0xc0 + 0x180, UNIT_RESPOND_RANGE × 0xc0)`,
twelve tiles. From the Fighter's seat (888, 7800) the Airbase is ~10.6
tiles off, and the search on its birth tick re-finds it, so the arm
changes nothing there and the walk runs as in this crate: every block to
631 agrees, the walk included. At the point it is ~12.2
tiles off, the search finds nothing, and the attack is gone with nothing
drawn. The Bomber `1/6` is AI-driven, so its search reaches 24 tiles
(`× 0x180`). It re-finds who=0's Airbase every frame from (1992, 7704)
and chases on: run175's alternating `ATTACK` and one-frame `MOVE` from 700.

### 62.3 The build, and what moved

`Sim::do_attack`, beside the one-in-five arm and under the same gate
(`!mandatory`, the captain): for an `Obj::Building` target,
`find_new_target(u, false)`. `None` returns. A different target marks
`NOT_FIRING` when the current order is an attack and returns (the search
already added the order the original rewrites). The same target re-reads
the fresh order and goes on to the range test and the chase.

| | before | after |
|---|---|---|
| chapter six-b, run175: word | 632, open | **1250**, sequence 1250, no value part: closed |
| every closed chapter | closed | closed |
| Great Lakes, East Indies long words | 12897, 13640 | 12897, 13640 |

**The value diff** is `chapter_six_b_s_word_frame_is_widened_whole`, now
over run175 whole (605 to 1249). The Fighter's 633/634 rows are gone (its
`orders.len`, stack `length`, `dest_angle` and `idle`). Past the first
block, only each aircraft's `form` and the harness's `order:target` on
its birth block part, and the scout `1/0`'s non-scoring
`order:move.facing` from 991 (parked 275). The Bomber agrees on every
block. With the arm off, 633 parts again and the draw stream with it.
The unit test `a_captains_attack_on_a_building_re_searches_every_frame`
was made to fail both ways: with the arm off, a captain sixteen tiles from
a building keeps its attack and chases; with the search's answer dropped,
one eight tiles off loses its attack.

### 62.4 What is not established

- **The retarget arm's early return** on `fight`'s entry arguments
  (`local_20`/`local_24`), which needs the target changed between entry
  and the search. Nothing in this crate changes it there.
- **The cavalry archer's variant** (`esi != 0`): `find_melee_target(-1,
  &who, cavarch, 0, 0)` in place of `find_new_target`, which neither
  kills the order nor adds one. This crate has no cavalry archer on this
  arm.
- **The unit target's `poor_target` search** (`005fded5`), and a unit
  target whose `local_14` is set, which in the original is `find_new_target`
  (a kill and a fresh order) where this crate retargets in place (§8.2).
  Both were left as they stand; the closed chapters agree on them.
- **The group test** (`vtable +0x2c`, `+0x94`), carried as this crate's
  `combat.captain == index` (§47.1).
- **Whether an aircraft on the ground strikes a building** (parked 683):
  neither aircraft is in range of one anywhere in run175.

### 62.5 Coverage

- **Diff-backed** (run175, 605–1249): the Fighter's drawless kill on 632,
  and the Bomber's every-frame re-find and stand from 700.
- **Packet-backed** (run177): `find_attack_pos` answers 1 with ten draws;
  `fight` answers 0, drawless, through `find_new_target` at `fight+0xa1f`.
- **Listing-backed**: the arm's gates and branches (`005fdd5b`–`005fe01f`).
- **Reading only**: every §62.4 arm.

## 63. A guard's attack is leashed to its post (item 707, 2026-09-24)

Golden chapter eleven (run190, `docs/GOLDEN.md` §19) stood at **1036**.
Ours spent 7 draws against 4, parting at index 0: `Unit::fight+0x9b0` and
two `Guy::set_anim+0xf2f`, all the guard `0/6`'s. On block 1037 the
original's guard holds its `GUARD` alone, `recharging 0`. This crate kept
the `ATTACK` on who=1's chariot `1/6` and fired. `1/6` had walked off on
its army's `ATTACKTOORDER` from 1021. The kill conditions for three
readings are in `docs/journal/2026-09-24-item-707.md`, written before the
build.

### 63.1 The arm in `fight`, and `check_target`'s guarding test

`Unit::fight@005fd4d0`, straight after `Object::valid_target` passes and
ahead of both captain arms (§62.2), `005fdc91`–`005fdd4f`:

```text
if param_5 == 0 && get_activity() && get_activity().type == GUARD:   ; 5fdc91-5fdca9, 0xc
    use_poor = 0
    if is_captain() && unit_masks & 0x40000:                         ; 5fdcc8-5fdce0
        use_poor = 1
        if Random::get(0, 0xffff) & 0x80000001 → drop                ; 5fdcef, fight+0x824
    if drop || !check_target(o, who, 1, NULL, 1, use_poor, 0):       ; 5fdd0c
        kill_current_order(0)                                         ; 5fdd19
        find_melee_target(-1, NULL, 0, 1, 0)                          ; 5fdd2a, adds what it finds
        order_type != ATTACK → return 0
        recharging != 0      → return 0
        unit_masks2 |= 0x10; return 0
```

`UnitData::get_activity@00608370` walks the list from the head past every
order that `is_move_attack@0047ff00` (vslot `+0x1c`, `is_move ||
is_attack`), so under a guard's `ATTACK` it lands on the `GUARD`.

**`Object::check_target@00649e00`'s guarding arm** (`param_5`,
`0064a00d`–`0064a190`, read from the listing, since the decompile lost
both `vector_dist`s' operands):

- the guard order comes off the activity (vslot `+0xec`), and needs `ox`,
  `whom` ≥ 0;
- `k` = 2, or 3 against a unit that is not a worker (`is_worker@0046fa10`,
  type `0x32`–`0x35`) when the guard has `unit_masks & 0x40000`;
- a **follower** whose captain's action is an `ATTACK` on this target is
  accepted at once, with `check_target` answering 1 (`0064a0eb`);
- `r = unit_guard_respond_range × k × 0x60`, with `constants +0x20`, 8
  tiles on both sides (`rules.xml`), so **1536** for a human;
- `vector_dist(target − post) > r` → 0, then `vector_dist(me − post) > r`
  → 0, the post being the order's `+0x1c/+0x20`, `guard_x/guard_y`.

Before it, a target in another `tregion` must be `is_in_range` (`param_3`
is 1 here, so only the region arm runs). After it comes the tail:
`poor_target` only with `use_poor`.

### 63.2 Why 1036, and not before

The guard shot `1/6` from its post on tick 1011 with `recharging` 25.
`fight`'s reload gate returns before `valid_target` (§8.2), so the leash
is first asked on **tick 1036**. By then `1/6` stands on (3356, 14040),
**≈1,780** from the post (3480, 12264). It crossed 1,536 near 1028.
`check_target` answers 0, the attack is killed with nothing drawn, and
the guard's own search is leashed too (§63.3), so it adds nothing and
writes `near` −1.

This crate's range test says the shot was still in reach, `attack_dist`
1536 against `8 × 0xc0 + 6` = 1542. The original never asks it: the leash
comes first.

### 63.3 The guard's search is leashed too

`Object::find_nearby_target@00648da0` sets `local_2c` when the searcher
is a unit, not a cavalry archer's call, and its activity is a `GUARD`
(`:176`–`184`). Then:

- **the rings are centred on the post**, the guard order's
  `+0x1c/+0x20`, not the searcher (`:240`–`250`);
- **`check_target` gets `guarding` 1** (`:332`), so every candidate out of
  the leash is refused **before `near_o` is written**;
- **an unarmed candidate must be reached** (`:346`–`352`), and under
  `unit_masks & 0x40000` so must an `ANTI_AIR` one. An armed one is deemed
  in range untested, as for any unit.

So the same leash governs `do_guard`'s idle-arm search
(`find_melee_target(−1, 0, 0, 1, 0)`, `docs/ORDERS.md` §24.3). That is
**parked 705's non-re-engagement**: `1/6` comes back to (3384, 13896),
≈1,635 from the post, and shoots the guard from there. The guard's
sixteen-frame search refuses it on the leash and finds nothing, on both
sides now.

### 63.4 A human's action order holds the retaliation

`Unit::target_opportunity@005fffc0`'s tail, `LAB_00600877`, adds the
retaliating attack only for an armed unit whose action (`update_action`)
meets one of these:

- it is null, or not flagged `ACTION` (flags & 4);
- the unit has `unit_masks & 0x40000`;
- it is an `ATTACK_TO`, a `0x15` or a patrol (vslot `+0x34`).

A player's guard, whose action is its `GUARD` (flags 4), does not answer
a hit. On 1091, `1/6`'s second hit, this crate's guard retaliated and
dropped the attack a frame later on the leash, drawlessly. The original's
never took it.

### 63.5 The build, and what moved

- `Sim::guard_activity`, `Sim::guard_leash` and `Sim::guard_check_target`
  (`crates/sim/src/fight.rs`);
- the arm in `Sim::do_attack`, with the AI's roll at `fight+0x824`
  ([`sim::fight::SITE_FIGHT_GUARD_ROLL`]);
- the leash, the post-centred rings and the guard's reach gate in
  `find_nearby_target`;
- the action gate in `target_opportunity`.

**1036 → 1133.** The value diff beside it, from the widening (every
record, both directions, and the new `near` row):

| block | field | before | after |
| --- | --- | --- | --- |
| 1037 | `0/6` orders | `ATTACK`, `GUARD` against `GUARD` | `GUARD`, both |
| 1037 | `0/6` `recharging` | 25 against 0 | 0, both |
| 1037 | `0/6` `near` | (6, 1) against (−1, −1) | (−1, −1), both |
| 1091 | `0/6` orders | (with the leash alone) `ATTACK`, `GUARD` against `GUARD` | `GUARD`, both |

From 1037 to 1133 no row parts. The word passes 1050, and **parked 705
closes with it** (§63.3): the guard never re-engages, on both sides. The
long captures' words hold at 14982 and 15782, and every closed chapter
holds.

### 63.6 What is not established, and coverage

**Diff-backed**: the arm's effect on a human guard (the 1037 rows); the
guard search's leash (`near` −1, and no re-engagement through 1133); the
retaliation gate for a human guard (1091).

**Listing-backed, never executed in a capture**:

- the AI roll at `fight+0x824` (no dump has a captain under a `GUARD`
  and `0x40000` reach an unrecharged `fight`);
- `k` = 3;
- the captain shortcut;
- the region arm;
- the guard's reach gate on an unarmed candidate;
- the retaliation gate's `ATTACK_TO`, `0x15` and patrol exemptions, whose
  ORDERS names the patrol's vslot `+0x34`.

SEAMS: `is_attack`'s overrides are folded in the export, so it is taken
as the attack and ground-attack orders. ~~`check_target`'s AI-sea region
exception~~ **carried by item 1214 (§86)**, with the coastal-refined
region; its building-cell tail is not carried. ~~The retaliation's
`on_duty` return (`00600877`'s `local_20 != NONE` above it) is not
carried.~~ **Carried by item 1040 (§70.3).**

**What stands at 1133** is not the guard's. `1/6`'s own `ATTACK` on the
guard goes on block 1134, the tick its reload opens, with no draw, so
before `fight`'s roll. The guard's `visible` bit for who=1 is 0 on every
block from 1086, so `1/6` fired only while who=1's world map saw the
guard's cell. The word's widening names no mechanism.

## 64. An attack-move passes over an unarmed building (item 997, 2026-09-27)

Item 997 was booked on Great Lakes' second word, frame 4555 of run347:
ours 7 draws against the original's 3, parting at index 1 on
`Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4` against
`Farms::inc_time+0x1ae`. No mechanism was named. The block before it
(run356, 4555) parted on `1/21`'s order alone: kind 10 and two orders
here, one `ATTACK_TO` there.

### 64.1 The frame, from the disk

- **Who and what.** `1/21`, a who=1 soldier (TY 82, `unit_masks
  262152`, stance RAID) in army 0, walks an `ATTACK_TO` to (4824, 34584)
  on both sides through 4554. The army is attacking (status 18, target
  the human's city), with `hurry 0`.
- **Ours pushed an `ATTACK`** on frame 4554 with no draw: the
  attack-move's look (`do_attack_to`, one frame in fifteen,
  `(4554 + 21) % 15 == 0`) took **`0/2001`**, the idle human's
  Woodcutter's Camp (`orig_type 418`, `myhits 800`, at (4224, 28608)),
  inside the AI radius `UNIT_RESPOND_RANGE × 0x180 = 4608`. On 4555 the
  chase asked `find_attack_pos`, whose ring walk is the four extra draws.
- **The original's `1/21` took nothing** and walked on: block 4556 has it
  at (8986, 29671) under the same single order.

### 64.2 The word, and the filter

`Unit::find_melee_target@005ff9c0` rewrites its caller's `flags` (every
caller but `do_move`'s ~~passes 0~~ and `Group::action_attack`'s retarget
passes 0; the retarget passes 1 or 2, §66.2) when the head order's index is
`ATTACK_TO` (`005ffc75`: `cmp $2`) or `order_type` answers
`GROUP_ATTACK_TO` (`005ffc81`: `cmp $0x15`). The listing,
`005ffc86`–`005ffcde`:

- the type's vslot `+0x10c` (`ObjectTypeData::is_siege`) answers →
  `0x20002`, replaced by the caller's word when `leader_flags & 4` (a
  human; `cmovne` at `005ffcab`);
- else vslot `+0x110` (`UnitTypeData::is_tank`, COLLISION's item 696)
  answers 0 → `0x20010`, else the caller's word (`cmove` at `005ffcc6`);
- then stance (vslot `+0xf4`) `== 4`, RAZE → `0x20` over either
  (`005ffcdb`).

`Object::find_nearby_target@00648da0` reads it per candidate right after
`valid_target` and above `check_target`, so above `near_o`
(`00649430`–`00649515`). The candidate's virtuals by the PDB's tables:
`+0x8` answers for a unit and 0 for a building or wall, `+0x1c` 1 for a
building or wall, `+0x20` 1 for a `Build` and 0 for a wall, `+0x2c`
`BuildData::is_wonder`, `+0x120` the attack.

- `& 1`: units only.
- else `& 2`: buildings only; with `& 0x20000`, only one that is armed,
  a wonder, or `BuildTypeData::is_military_trainer` (`+0x2c0 &
  0x40000000` on the base type).
- else `& 0x20000`: a building must be armed.

After the score (`00649766`–`006497b2`), `& 0x10` halves a candidate that
is not a unit, else `& 0x20` one that is not a `Build`, by a signed `/ 2`,
before the `score == 0 && value != 0 → 1` clamp. `Build::find_target`
passes 0 (`622c88`).

So a soldier on an attack-move passes over a Woodcutter's Camp, and would
take an armed city centre or tower at half weight against a unit.

### 64.3 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| the army is hurrying there, and `do_attack_to`'s six-cell gate skips the look | `hurry` is set only by `find_muster_spot`'s defence of a damaged own building; ours reads 0, and nothing of who=1's is damaged | not the cause |
| the radius differs | the listing's radius arms are ours (§12.4); the camp sits inside 4608 by either reading | killed |
| the attack-move's `flags` refuse an unarmed building | with the filter, 4555's draws are 3/3 and `1/21` walks on; with `search_admits` answering true (the mutation), the walk falls back to 4555 at 7 against 3 and the unit test names the camp | **held** |

### 64.4 The fix

`sim::fight`: `Sim::melee_search_flags` is the word, `Sim::search_admits`
the filter, and `find_nearby_target_with` the search with its fifth
argument; the halving sits before the clamp. `find_nearby_target` passes 0.
The unit test is `an_attack_move_passes_over_an_unarmed_building`.

### 64.5 What it moved

- **Great Lakes 4555 → 4593.** On run356, block 4555's six keys of `1/21`
  (kind 10 against 2, two orders against one, `orders_x/y` (9005, 29653)
  against (4824, 34584), `dest_angle`) all closed. Its walk agrees until
  4606. Frame 4555's draws went 7/3 → 3/3.
- **East Indies holds at 5606**, and the first pair at 24,000 on both maps.
- The new word is `docs/AI.md` §82.4's.

### 64.6 What is not established

- **The `0x20002` arm, the RAZE `0x20` arm and the `& 1` arm** have no
  capture. The siege arm's military-trainer test is this crate's
  `build_types` flag on the base type, not the dump.
- **`BuildData::attack`'s garrison arm** (§4.1) is not in
  `Sim::attack_of`, so a building armed only by its garrison would be
  refused here and taken there. No capture on disk garrisons one.
- **`do_move`'s own call** passes 1 or 2 as the caller's word (§37.2);
  this crate makes no such call.

### 64.7 Coverage

**Diff-backed**: the camp's refusal, by run356's block 4555 and frame
4555's draws, a floor that falls back under the mutation. **Listing-backed**:
`005ffc6a`–`005ffcde`, `00649430`–`00649515`, `00649766`–`006497b2`,
`622c88`. **Unit-backed only**: the siege, tank and RAZE words.

## 65. A ranged chase on a building stops at the edge of its reach (item 1002, 2026-09-27)

Item 1002 was booked on Great Lakes' second word, frame 4593 of run347:
ours 4 draws against the original's 5, parting at index 2 on
`Farms::inc_time+0x1ae` against `Guy::set_anim+0xf2f < Guy::move+0x166`.
No mechanism was named. The block before it (run356, 4592) parted on
`1/26`: an `ATTACK` there (kind 10, two orders) and a walk here (kind 1,
three orders).

### 65.1 The frame, from the disk

- **Who and what.** `1/26` is a who=1 ranged soldier (guy type 82, reach
  6, so `6 × 0xc0 + 6 = 1158`) in army 0, stance RAID. Its attack is not
  mandatory: the dump's `ATTACKORDER` reads `mandatory 0`, `ox 2000
  whom 0`, the human's city. It walks its attack position (4872, 31752)
  under that `ATTACK`, over its army's `ATTACK_TO`.
- **Both sides agree through block 4591**, at (5079, 31628). There the
  harness's reach print (`RON_DEBUG_UNIT`, new) reads `attack_dist` 1128.
  That is inside 1158 but not inside `1158 − 0x90 = 1014`.
- **The original stops there.** On block 4592 `1/26` stands at (5079,
  31628) with `stopped 1`, `orders_x/y` its own spot and one `ATTACK`. On
  4593 it shoots (`recharging 33`, `hold_attack 1`), and the shot's
  `Guy::set_anim` is the word's fifth draw.
- **Ours walked on**, to (5052, 31644), because its kill asked with the
  `0x90` margin.

### 65.2 The listing

`do_move@005f7b30`, the attack under a move. After the gate on the
attacker's `max_range` (`5f7f1a`), `5f7f27` calls the **target's** vslot
`+0x1c`: 1 for a building or a wall, 0 for a unit (§64.2's table).

- **A unit** (`je 5f7fbe`): the flank clause, then
  `is_in_range@006486b0` with the sixth argument `mandatory == 0`, then
  `find_collision`. This is the kill §35.3 measured. The captain
  retarget below it is this arm's too.
- **A building** (`5f7f34`–`5f7faa`):
  - `unit_masks & 0x40000` with the type's `is_siege` (`+0x10c`) answering
    → `5f82c1`, the planner;
  - else `is_in_range@00648d70` (`push 0` at `5f7f53`: its fourth
    argument, which it hands to `006486b0` as the sixth), so **no
    margin**;
  - then `valid_target@00648ba0` and `find_collision@0065b1b0(my spot,
    o, who, 1) == 0` → `kill_current_order(0)`;
  - every refusal → `5f82c1`. There is no retarget.

`docs/ORDERS.md` §4.4 step 4 has carried the building arm since the first
reading: "a building target in range, valid, no collision → kill". The
code asked every target the unit arm's question.

### 65.3 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| parked 1003: the attack-move's own target (`order:target`, ours none against the city) | the row stands on `1/26` from 4550 to 4604, and 4592 closes under the fix with it still standing | not the cause |
| the reach differs | both sides agree the unit is not in range on 4590 at 1176 and stop it one frame later; the reach is the type's | killed |
| a building is asked without the margin | with the building arm, 4592 and 4593 agree and 4593's draws go 5/5. With the margin put back (the mutation), the walk falls back to 4593 and the unit test fails on its first arm | **held** |

### 65.4 The fix

The fix is in `sim::orders`, in the attack block under `do_move`. A
building target takes the building arm (the siege gate and
`is_in_range` with no margin), and a unit target keeps the unit arm (the
margin and the captain retarget). The unit test is
`a_ranged_chase_on_a_building_stops_at_the_edge_of_its_reach`.

### 65.5 What it moved

- **Great Lakes 4593 → 4605.** On run356, `1/26` agrees from 4592 to
  4605. On block 4606 it first parts again, kind 2 here against 10
  there. Frame 4593's draws went 4/5 → 5/5.
- **East Indies holds at 5606.** The first pair holds at 24,000 on both
  maps.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism
  is named): frame 4605, ours 2 draws and the original 43, parting at
  index 0.
  - Ours spends `Farms::inc_time+0x1ae`.
  - The original spends 41 `Unit::find_attack_pos+0xea9`: four under
    `Group::action_attack+0x41a`, then 37 under `Unit::fight+0xcb4`.
  - Block 4606 parts on the army's members: `1/9`..`1/14` hold the
    army's `GROUP_ATTACK_TO` (kind 21) here and an `ATTACK` (kind 10)
    there.
  - Earlier rows in the window: `1/22`/`1/23`'s `order:target` on
    4598/4599 (parked 1003's shape), `1/0`'s move destination on 4599,
    `1/19`/`1/20`'s path by one unit on 4600, the human leader's `wars`
    and `active_wars` on 4601, and `1/15`..`1/17`'s group id on 4605.

### 65.6 What is not established

- **The siege gate** (`5f7f34`) has no capture: `is_siege` is this
  crate's `uflags::SIEGE` on the type, as §64.2's word reads it.
- ~~**`find_collision`** at the attacker's own spot is not modelled in
  either arm. It only makes the kill rarer.~~ **The building arm's is
  built** (item 1177, §65.8). The unit arm's (`5f7f11`) is still not
  modelled.
- **`valid_target@00648ba0`** (`5f7f71`), the building arm's third
  conjunct, is not modelled. It refuses only a capturable building the
  attacker cannot capture, which no capture on disk has shown.
- **A wall target** takes the building arm by the vslot. This crate's
  walls are buildings, so it does too, and no capture shows one.

### 65.7 Coverage

- **Diff-backed**: the building arm's kill, by run356's blocks 4592 and
  4593 and frame 4593's draws. It is a floor, and it falls back under the
  mutation. Its `find_collision` conjunct, by run426's `1/29` on blocks
  15585..15587 and the third map's long word (§65.8).
- **Listing-backed**: `5f7f1a`–`5f7faa` and `00648d70`.
- **Unit-backed only**: the siege gate, and the unit arm's margin at the
  same distance.

### 65.8 Another unit in the chaser's block holds the kill (item 1177, 2026-09-29)

Item 1177 was booked on Great Sahara's long word: frame 15586 of run383,
ours 7 draws against the original's 6, ours spending one extra
`Guy::set_anim+0xf2f < Guy::move+0x166` at index 3. No mechanism was
named.

- **The frame, from the disk.** run426's widening shows `1/29`, a who=1
  Longbowman (type record 127, `GUY.type` 177, reach 12, so `12 × 0xc0 +
  6 = 2310`), quiet on every field through block 15584. It chases the
  human's city `0/2004` with a `mandatory` `ATTACK` under a `MOVE_TO`
  (the harness's reach print, `RON_DEBUG_UNIT`). On block 15584 it stands
  at (7292, 27822) at `attack_dist` 2284, inside the reach.
- **Ours** ended the chase on frame 15584. On block 15585 it stood with
  the `ATTACK` at its head, and on frame 15586 its arrival spent the
  extra draw.
- **The original** took one more step, to (7269, 27840), and ended the
  chase on frame 15585. Its kind is 1 against ours' 10 on 15585, and its
  figure clock is 9 against ours' 13 on 15587.
- **The listing**, `5f7f69`–`5f7fa4`. After `is_in_range` the arm calls
  `valid_target@00648ba0`, then `pushl $0x1` and
  `find_collision@0065b1b0(x, y, o, who, 1)` at the unit's own position.
  Non-zero goes to `5f82c1`, the planner. With the fifth argument set,
  `find_collision` skips its land shortcut (`param_5 == 0 && type +0x218
  == 0` → `collide_here`). It walks the nine world cells' chains instead:
  every other player's (`who < 8`) live, on-map unit with a block, and
  hits when that unit is Chebyshev-within the two types' `+0x248` blocks
  in unit cells, at current positions. That is `Sim::chain_hit`, the sea
  and air arm of §5.2's `find_collision` (`docs/COLLISION.md`).
- **The collider** is `1/29`'s squad-mate `1/30` at (7176, 27720): two
  unit cells off on both axes against two blocks of one each. One step on,
  the gap is three and the kill fires.

**The fix**: `Sim::find_collision_forced` (`sim::collide`) is the forced
query, and the building arm asks it last (`sim::orders`). The unit test
is `a_unit_in_the_chaser_s_block_holds_its_chase_on_a_building`: a mate
two cells off holds the chase, three cells off ends it, and gaia is not
asked.

**What it moved**:

- **Great Sahara's long word: 15586 → 15982.** Frame 15586's draws go
  7/6 → 6/6.
- **The value diff**: run426's block 15585. `1/29`'s `order:kind` 10
  against 1 → 1, and its `pos` (7292,27822) against (7269,27840) → the
  original's. Every `1/29` row agrees across 15581..15837, and run426's
  parted keys fall 741 → 183
  (`run426_s_word_frame_is_widened_whole`).
- **The unit arm** (`5f7f11`) makes the same call, and it is left
  unmodelled here. No word on disk parts on it, and building it would
  change frames this item does not test.

## 66. An army group's attack-move looks, and hands its find to the group (item 1012, 2026-09-27)

Item 1012 was booked on Great Lakes' second word, frame 4605 of run347:
ours 2 draws against the original's 43, parting at index 0 on
`Farms::inc_time+0x1ae` against `Unit::find_attack_pos+0xea9 <
Group::action_attack+0x41a`. No mechanism was named. Block 4606 of run356
parted on army 0's members: kind 21 here, kind 10 there.

### 66.1 The frame, from the disk

- **What the original did.** On block 4606 every member of group 65
  (`1/9`–`1/26`, army 0) holds an `ATTACK` on the human's city `0/2000`
  (`mandatory 0`, `new_ord 1`). Under it, the `GROUP_ATTACK_TO` is re-issued
  with a new `id` and the old `orig` (4177, 34758). `1/9`–`1/14` hold the
  bare `ATTACK`, and `1/16` onward already walk toward their attack
  positions (kind 1).
- **The trace names the caller.** The four draws are
  `find_attack_pos < action_attack+0x41a < action_attack+0x2e5`, which is
  `Group::action_attack` recursing into itself. That is its `QUEUE_FIRST`
  arm (`00712490:168`): `set_up_insert`, `action_halt(ignore)`, recurse
  `QUEUE_NEW`, `finish_insert`. `Army::engagement` passes `QUEUE_NEW`, and
  `check_target_path` passes `mandatory 1`, so neither is the caller.
- **The looker.** `(4605 + o) % 15 == 0` puts `o` at 0, 15 and 30.
  `1/15` is a captain (`o_up −1`) walking `GROUP_ATTACK_TO`. The members
  it follows in the list were processed before the call, which is why
  `1/9`–`1/14` hold a bare `ATTACK` and the rest have already walked.
- **Why 4605 and not before.** Until block 4605, `1/21`–`1/26` had `ATTACK`
  or `MOVE` heads, so `Group::is_attacking_to` was false. By block 4605
  every member's head is `ATTACK_TO` or `GROUP_ATTACK_TO`.
- **Collisions are not the cause.** `do_group_move`'s collide arm also
  calls `Group::action_attack(…, QUEUE_FIRST)`, but no member's `collide_o`
  names an enemy on blocks 4603–4606.

### 66.2 The listing

- **`Unit::do_group_attack_to@005e74e0`** runs `do_group_move`, then
  `do_attack_to`'s own look: the head is still this order, the
  fifteen-frame phase, armed and not supply → `find_melee_target(−1,
  NULL, 0, 1, 0)`, else `do_attack_to_pause`. It has **no hurry gate**.
- **`Object::find_nearby_target@00648da0`'s add arm** (decompile lines
  545–606): a searcher whose `order_type` is `GROUP_ATTACK_TO`, whose
  group (after `normalize`) holds it active (`member(o, who, 1)`), and
  whose group `is_attacking_to`, and which is **not siege**, calls
  `Group::action_attack(o, whom, 0, QUEUE_FIRST, 4)`. Anything else calls
  `add_attack_order(…, QUEUE_FIRST, param_1, 0)` (`LAB_00649ba0`).
- **`Group::is_attacking_to@0070ea70`**: every live, on-map member with a
  head order has a head of type 2 or `0x15`.
- **The retarget's word** (`00712490:456`–`470`): `find_melee_target(u,
  min(d + 0xc0, unit_respond_range × 0x240), &whom, 0, 0, word)`, where
  `word` is **1 when the target's vslot `+0x1c` answers 0 (a unit) and 2
  when it answers 1 (a building)**. After the halt, the head is not an
  attack-move, so §64.2's rewrite leaves the word standing: ~~a member sent
  at a city searches buildings only~~. **Corrected by item 1028 (§68.2)**:
  the retarget is under the target's vslot `+0x20` answering 0, so it
  searches only for a unit or a wall target, and a city is handed to every
  member unsearched.
- **The ring's start** (`601d88`–`601f54`): the start-side switch at
  `601e3e` loads both arms' `x`/`y` slots (`601e04`–`601e1a`). For side 4
  (`601ea0`–`601ef4`) that is `x = t.x + x_size × 0x60 + stand`, `y = t.y`,
  and sides 2, 6 and 8 have the same shape. So an edge start is the face's
  **midpoint**, as §17.2 has always said. The code had started an edge at
  the corner end, which is only where an arm stepping onto an edge enters
  it.

### 66.3 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| `Army::engagement` (it attacks with every group) | it passes `QUEUE_NEW`, the army ticks at `frame ≡ 252 (mod 256)` (4604), and the members keep their `GROUP_ATTACK_TO` under the attack | killed |
| `do_group_move`'s collide arm | no member's `collide_o` names an enemy on 4603–4606 | killed |
| `do_group_attack_to`'s look, handed to the group | with it, 4605 draws 43/43; with the look skipped (the mutation) the walk falls back to 4605 and the unit test fails | **held** |
| the retarget's word | without it, `1/9`–`1/11` take the scout `0/0` (in reach through the ring rounding) where the original takes the city; with the word 0 the walk falls back to 4606 at index 0 | **held** |
| the edge start at mid-face | `1/17`, the one asker square on the east face, picks (3864, 31512) from the corner and (3912, 30840) from the midpoint, the dump's value; with the corner start the walk falls back to 4606 at index 27 | **held** |
| `find_melee_target`'s squad head at the retarget (a follower takes its captain's `ATTACK` without a search), read from the decompile's path | built, it moved the walk **back** from 4618 to 4607. `1/24` flips from the citizen `0/3` to the city on frame 4606, because the city's `targeted` (bumped only by a search's winner, §33.1) is twelve bumps short | **killed by the floor**; not built, parked |

The last row stands against the decompile. Either the followers search at
this site in the original, or another search bumps the city. A packet at
4606 that reads the city's `+0x3d` would decide between the two.
**Answered by item 1028 (§68.2)**: a city target never reaches the search,
so neither holds. The kill stood on raid-less values, and the word row
above is the wrong arm.

### 66.4 The fix

- **`sim::orders`**: the dispatch runs `do_group_attack_to_tail` after
  `do_group_move` for a `GROUP_ATTACK_TO`. `attack_move_look` is the body
  it shares with `do_attack_to_tail`, which keeps the hurry gate.
  `attack_move_add` is the add arm, and `melee_squad_head` is the old inline
  head moved into a helper.
- **`sim::group`**: `group_is_attacking_to`, `group_action_attack`'s
  `QUEUE_FIRST` insert arm, and the word at the retarget.
- **`sim::fight`**: `find_melee_target_with` and `melee_search_flags_with`,
  which carry the caller's word. A human's siege and a tank keep it.
- **`sim::attack_pos`**: `start_base`.
- **Unit tests**:
  - `an_army_group_s_look_hands_its_find_to_the_group`
  - `a_group_retarget_from_a_building_takes_a_building`
  - `an_edge_start_is_the_face_s_midpoint`
  - `a_ring_started_on_an_edge_stands_level_with_the_target`

### 66.5 What it moved

- **Great Lakes 4605 → 4618.** On run356's block 4606 every member of
  group 65 holds the city's `ATTACK` over a re-issued `GROUP_ATTACK_TO` on
  both sides, where ours had held the bare `GROUP_ATTACK_TO` (kind 21
  against 10, one order against two). `1/17` walks to (3912, 30840) on
  both. Frame 4605's draws went 2/43 → 43/43. The only rows standing on
  4606 are `order:target` on the city, and those are the harness's
  (§66.6).
- **East Indies holds at 5606.** The first pair holds at 24,000 on both
  maps.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism
  is named): frame 4618, ours 94 draws and the original 4, parting at
  index 0. Ours spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, and
  the original spends `Unit::do_non_flat_gather+0xcc3`.
  - Block 4619 parts on `1/0`'s group (66 against 67), its `idle` and its
    move, and on `1/7`'s gather `wait` (361 against 335).
  - Block 4617 parts on `1/24`'s chase spot, (4200, 31608) against
    (4104, 31512).
  - The word is inside run356's window: 69 blocks after its first and 187
    before its last.
  - **Moved to 4673 by item 1014** (`docs/SCOUT.md` §8.3). The idle
    units were the AI scout `1/0` alone, and it had taken the wrong
    explore target on 4506. `1/0`'s group and `1/7`'s `wait` no longer
    part. `1/24`'s chase spot (block 4617) now stands 56 blocks before the
    new word, which is `1/24`'s. **Moved to 4688 by item 1023** (§67): the
    review re-aims the chase on 4616.

### 66.6 What is not established

- **The siege arm of the add** (`local_2c`: an AI siege unit on a building
  whose `+8 & 0x20` is set takes it `mandatory` and writes the army's
  target) and the naval refusal. No capture reaches either.
- **The squad head at the retarget** (§66.3, last row). ~~For the city~~:
  a `Build` target never searches (§68.2). It stands for a unit or a wall
  target, with no capture.
- **Parked 1003 is the harness.** `Built::builds` links each player's
  pre-placed buildings but not the capital. who=0's table holds
  `2001`–`2006` and not `2000` (handle 0). So `build_ids` answers `None`,
  and every `ATTACK` on a city reads `order:target` ours `None`, as on
  `1/24`–`1/26` from 4550. `unit_ids` has had an `(owner, index)`
  fallback since item 462, and `build_ids` has none.

### 66.7 Coverage

- **Diff-backed**:
  - the look and the group hand-off, by run356's block 4606 and frame
    4605's draws;
  - the word, by `1/9`–`1/11`'s targets and frame 4606's draws;
  - the edge start, by `1/17`'s spot.

  Each is a floor that falls back under its mutation.
- **Listing-backed**: `601d88`–`601f54`.
- **Decompile-read**: `005e74e0`, `0070ea70`, `00648da0:545–606`,
  `00712490:168–174, 456–470`.
- **Unit-backed only**: `is_attacking_to`'s refusal. Answering it true
  does not move the walk.

## 67. A fleeing target re-aims the chase (item 1023, 2026-09-28)

Item 1023 was booked on Great Lakes' second word, frame 4673 of run347:
ours 4 draws against the original's 3, parting at index 0 on
`Unit::fight+0x9b0` (the chaser `1/24`) against `Farms::inc_time+0x1ae`.
No mechanism was named. Block 4673 of run356 parted on `1/24`'s order,
and its chase spot had parted since block 4617.

### 67.1 The frame, from the disk

- **Who and what.** `1/24` is a who=1 ranged soldier (guy type 82) in
  group 65, a captain (`o_up −1`), stance RAID. Since the army's attack
  on 4605 it has held an `ATTACK` on the human's citizen `0/3` (`ox 3
  whom 0`, `mandatory 0`) under the group's `GROUP_ATTACK_TO`, with
  `fight`'s chase in front: a `MOVE` to (4200, 31608) via (4968, 30840).
  `0/3` is gathering at `0/2002` and walking away from it, from (2754,
  31941) on block 4615 to (2718, 31907) on 4617.
- **The original re-aims on 4616 with no draw.** Block 4617 prints the
  move at (4104, 31512) via (4872, 30744): every coordinate is 96 less
  on both axes, and the path is replanned. Ours keeps (4200, 31608).
  Frame 4616's draws agree on both sides, and none of them is `1/24`'s.
  So neither `fight`, whose captain arm rolls (`+0x9b0`), nor a ring
  walk, which rolls per candidate, ran for it.
- **The clock.** `(4616 + 24) % 16 == 0`, which is `Unit::work`'s review
  phase (§36.2). No other writer of a chase spot runs on that phase
  without a draw.

### 67.2 The listing

`check_target_path@005e22d0`, for an `ATTACK` action on a unit target.
The flank triple at `5e2434`–`5e24b3` (§36.3) has three outcomes:

- **not fleeing**: `is_in_range` from where the unit stands, and `repath`
  only if the target is in reach;
- **fleeing** (the target faces within 120° of the bearing from me and
  is moving): the triple jumps to `5e24e2`, which the code had read as
  `return 0`;
- **a building target** reaches `5e24e2` too, since its vslot `+0x8`
  answers 0.

From `5e24e2`:

1. The target's vslot `+0xbc`, `UnitData::is_on_map`, answers 0 →
   `repath`, return 1.
2. `is_in_range@006486b0(o, who, this+0x70, this+0x74, ·, 0, 0)`, pushed at
   `5e254e`–`5e2561`. `+0x70`/`+0x74` are `orders_x`/`orders_y` in the
   type record, so the question is asked **from the walk spot**, with no
   margin. If it answers yes → return 0 (`5e2568` → `5e2915`, `xor eax`).
3. The order's vslot `+0x18`, `is_attack` (`5e257b`). For an attack:
   - a melee type (`max_range` `+0x1fc` is 0) with `path.length`
     (`+0xc0`) of 1 → return 0;
   - a building target (`+0x1c`) → return 0.
4. The head's vslot `+0x2c` (`5e25c6`) is 0, or `group` (`+0x80`) is
   negative → `5e26f7`. There: `repath`, then `find_attack_pos(o, who,
   1, &x, &y, 0)` through the thunk `00602e60`, from the unit's own
   position. Nothing found → return 1, with the legs gone.
5. The `DEFENSIVE` leash (`5e2719`–`5e2832`). It applies when:
   - the stance (`vt+0xf4`) is 1;
   - the attack is not `mandatory` (`+0x1c`);
   - the order is not flagged `& 4`;
   - it is `defensive` (`+0x1d`), with a post (`+0x14`/`+0x18`) that is
     non-negative.

   Then, if `vector_dist` from the post is `max(unit_defensive_respond_
   range, max_range) × 0xc0` or more → `find_new_target(this, NULL, 0)`,
   return 1.
6. Else `add_move_order(x, y, 1, 0, QUEUE_FIRST, 0, ·, −1, −1)`
   (`5e2840`–`5e2863`), return 1.

`find_attack_pos`'s third argument only picks the sweep's filter:
`(param_3 != 0) × 2 + FILTER_NOT_ME` (`00601280:266`), so `FILTER_CAN_
COLLIDE` (5) here against `fight`'s `FILTER_NOT_ME` (3), by the PDB's
`FilterIndex`. `UnitType::find_nearby_spot` sends both down the one
pairwise branch (`0061de70:72`). The unit arm's flanking projection sits
under `(unit_masks & 0x40000) == 0` (`00601280`), and `1/24` carries
`unit_masks 262158`, so the projection does not apply to it.

### 67.3 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| `do_move`'s captain retarget (`(o + frame) & 0xf`, `find_melee_target`) | it rewrites the target through `change_target`, and `1/24`'s target is `0/3` on both sides through 4688 | not the cause |
| `fight` re-ran the chase | `fight`'s captain arm rolls `+0x9b0` first, and 4616 has no draw of `1/24`'s | killed |
| the review's fleeing arm re-aims | with it, 4617's move is the original's and 4673 agrees. With the fall-through returning (the mutation), the walk falls back to 4673 at 4 against 3, and the unit test fails on its first arm | **held** |
| the range is asked from the walk spot | the walk cannot tell: the unit and its spot are both out of reach on 4616. The unit test's last arm can (a spot in reach, a chaser twelve tiles out), and it fails with `pos` in place of `orders_pos` | **held**, by the listing and the unit test |

### 67.4 The fix

The fix is `sim::orders`' `rechase_fleeing`, which `check_target_path`
calls where the flank triple holds. `fight`'s chase and the re-aim share
`find_attack_pos` and `add_move_order`. The ring's label is
`fight::SITE_ATTACK_POS_REVIEW`, named and never spent, since a unit
target's sweep draws nothing. The unit test is
`a_fleeing_target_re_aims_the_chase_on_the_review_s_phase`. It has five
arms:

- a fleeing target re-aims;
- a standing target keeps the spot;
- a target walking toward the chaser keeps the spot;
- an off-phase frame keeps the spot;
- a walk spot still in reach keeps it.

### 67.5 What moved

- **Great Lakes 4673 → 4688.** On run356's block 4617, `1/24`'s move
  reads (4104, 31512) via (4872, 30744) on both sides, where ours read
  (4200, 31608) via (4968, 30840). Its rows agree from block 4605 to
  4688. That includes the order on 4673 (kind 10 against 1) and
  `stopped` on 4674. Frame 4673's draws went 4 against 3 → agreeing.
- **East Indies holds at 5606**, and the second pair's other tests hold.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism
  is named): frame 4688, ours 35 draws and the original 35, parting at
  index 31. The count parts on 4690, 6 against 4.
  - Both sides spend `1/24`'s `Unit::fight+0x9b0`.
  - Ours then spends `Guy::set_anim+0x97a < Guy::move+0x19f`, where the
    original spends a fourth farm draw (`Farms::inc_time+0x1ae`, then
    `+0x1de`).
  - Block 4689 parts on `1/24`. The original has struck from its cell
    centre (3768, 31608): `recharging` 33, `hold_attack` 1, `in_range`
    1, target `0/3` kept. Ours has not.
- **What the disk says about the new word, and what it cannot.** The
  roll is 18653, and `18653 % 5 = 3`, so both sides run the one-in-five
  re-search. Ours scored (item 1028 corrects it, §68.1):
  - the human's scout `0/0`, 216 away, at 344 (value 1032, shaped
    distance 492);
  - `0/3` at 225 (value 1800, distance 1393).

  Ours retargets to the scout. The original keeps `0/3`, so its search
  ranked the scout lower.
  - **Killed**: the stealth arm of `UnitData::is_seen`. The Scout's type
    flags read `lmahc` in `unitrules.xml`, with no `s`, and
    `unit_masks`/`unit_masks2` print 0.
  - **The remaining input**: the scout's `targeted` (`ObjectData +0x3d`),
    which no dump prints. The scout loses at a `targeted` of 6 or more,
    where ours reads 0. A packet at 4687 would read it. That is the next
    item's, beside parked 1015's city count. **Killed by item 1028**:
    run368's packet reads `targeted` 0 on the scout and on `0/3`. The
    decider is `compare_target`'s raid arm, which values `0/3` at 9800 and
    the scout at 15. **Moved to 4690** (§68.5).

### 67.6 What is not established

- **The group arm** (`5e25d1`–`5e26e9`: `Group::is_moving_to`,
  `Group::kill_next_order`, `Group::action_attack`). It needs a reviewed
  group head, and `check_target_path_review` reviews none (§36.4's group
  conjunct).
- **A building target's fall-through.** It returns 0 at every exit but
  the building's vslot `+0xbc`, which `repath`s. This crate returns for a
  building, as before.
- **The `DEFENSIVE` leash, the `is_on_map` exit and the melee path-length
  exit** have no capture. They are built from the listing.
- ~~**`is_in_range@006486b0`'s world-cell test** (`& 0x30 == 0x30` answers
  no) is not in `Sim::is_in_range_at`.~~ Built by item 1200 (`docs/GOLDEN.md`
  §50): the tile under the point asked from, any attacker.

### 67.7 Coverage

- **Diff-backed**: the re-aim, by run356's block 4617 and frame 4673's
  draws. It is a floor, and it falls back under the mutation.
- **Listing-backed**: `5e24e2`–`5e2873`, and `find_attack_pos`'s filter
  (`00601280:266`, `0061de70:72`).
- **Unit-backed only**: the walk spot as the range's origin, and the
  standing, approaching and off-phase refusals.

## 68. A raider wants the economy, and a Build target is handed out unsearched (item 1028, 2026-09-28)

Item 1028 was booked on Great Lakes' second word, frame 4688 of run347:
35 draws a side, parting at index 31. Both sides spend `1/24`'s
`Unit::fight+0x9b0` (roll 18653, so the one-in-five re-search runs). Then
ours spends `Guy::set_anim+0x97a < Guy::move+0x19f`, where the original
spends `Farms::inc_time+0x1ae`. On block 4689 the original's `1/24` has
struck the citizen `0/3`, and ours has retargeted to the human's scout
`0/0`. The booking's hypothesis was the candidates' `targeted`
(`ObjectData +0x3d`), which no dump prints. **The packet killed it.**

### 68.1 The RAID arm of `compare_target`

- **The packet.** run368, at logger frame 4688 (after trace tick 4687), is
  the state `1/24`'s re-search reads (`docs/EMULATOR.md` §8's rule: the
  word itself, as a logger frame).
  - `targeted` is 0 on every candidate, the scout and `0/3` included, on
    both sides. Only the city differs: ours 31, the original 0 (§68.3).
  - `Unit::find_melee_target@005ff9c0`, entered directly on `1/24`
    (parked 1026), answers `0/3`.
  - Hooked at `00649701`/`00649715`/`006497b4`, every candidate's shaped
    distance equals ours. The **values** do not: `0/3`, `0/4` and `0/5` at
    9800 (ours 1800), the scout 15 (ours 1032), the city 15 (ours 1866).
- **Why.** `1/24`'s stance is 3, RAID. Every member of group 65 (`1/9`–
  `1/26`) has been RAID since before block 4550. `Object::compare_target@
  0064e5c0` has a raid arm this crate never built (§12.3 had left it as an
  input).
- **The flag** (`0064e6b6`–`0064e742`):
  - `bVar17` is a unit attacker whose type has a stance type
    (`has_stance_type`) and whose stance (`+0xb1`) is 3;
  - `bVar3` is the attacker's `unit_masks & 0x40000`, a computer's unit;
  - `bVar16` is the attacker's type `+0x218 == 1`, a ship. With both, it
    raids only without the SIEGE objmask (`has_objmask(0x40000)`).
- **An active building** (`0064ea57`–`0064ea7a`): the attack formula
  `v × attack × 100 / hits_left` becomes `v / 20` when the target is a
  `Build` (vslot `+0x20`), active (`+0x4c`) and the attacker raids. After
  the damage weight a raider jumps past the building's armed ×5 and
  siege bonus to the tail (`0064f124`).
- **A unit target**, after the combat-role ×20 (dispatch `0064edc4`):
  - a computer's land raider (`0064ef69`–`0064efbb`): a peasant
    (`ObjectData::is_peasant@0046d310`, type `0x32`/`0x33`) or a caravan
    `+900,000`; else a combat unit (`+0x2c8 & 0x10000`) `+10,000`; else
    `/10` (`0064ee6e`);
  - a human's raider (`0064efc3`–`0064eff9`): a peasant or a caravan
    `+9,000,000`; else `/10`;
  - a computer's ship (`0064edd8`–`0064ef64`): the `0x150`/`0x13d` lineage
    tests and the stealth-ship weights. **Not built**; read as the land
    arm (a SEAM, no capture).
- **Arithmetic, checked on the packet.** `0/3`: `80,000 + 900,000` →
  `ceil(/100)` 9800. The scout: `3,200 / 10 = 320` → 4 → the floor, 15. The
  city's trace on the packet: cost 20, `4 × 20 = 80`, `/20` → 4, × damage 7,
  → the floor. Out-of-range `/5` is waived for a raider, as before (§12.3).

### 68.2 A `Build` target is handed to the group without a search

With §68.1 alone the walk fell back to 4605: every member of group 65 took
the Woodcutter's Camp `0/2001` instead of the city. Under raid values every
building scores the floor, and the nearer ring wins the tie.

- **The packet.** run369, at logger frame 4605, is the state before
  `1/15`'s look. On it:
  - `find_nearby_target` with the word 2, from `1/9`, `1/13` and `1/21`,
    answers `0/2001` for all three (values 15 to 42, the first on the
    nearer ring). So the members cannot be taking a word-2 search.
  - `1/15`'s look, `find_melee_target(−1, NULL, 0, 1, 0)` entered
    directly, finds the city (flags `0x20010`, the city its only
    candidate). `Group::action_attack@00712490` is called from the add arm
    and recurses `QUEUE_NEW`. **All eighteen members get
    `add_attack_order(2000, 0, …)` from `action_attack+0xc44`, and not one
    `find_melee_target` is called between.**
- **The listing's reason.** The `mandatory == 0` retarget
  (`00712490:456`–`470`) is under `00712490:433`–`437`: the target's
  vslot `+0x20` answers 0, which is a unit or a wall. A `Build` target takes
  the other arm. So the retarget's word is 1 for a unit and 2 for a
  **wall**, never for a city. §66.2's "a member sent at a city searches
  buildings only" was the wrong arm, and the walk could not tell until the
  city's value stopped carrying it.
- **Parked 1015 is answered.** Its question was the squad head at this
  retarget, and the floor's kill of it (§66.3's last row) was a city
  target. A city target never reaches the search. The squad head stands
  unbuilt for a unit or a wall target, with no capture.

### 68.3 A building's `targeted` decays

`Wall::process@00640450`'s first statement quarters `+0x3d` (signed,
toward zero) on every nonzero frame whose low three bits are the owner's
player number, in the branch that calls `check_ever_seen`. §33's list of
`+0x3d`'s writers names it, and this crate had carried it as a seam. The
city read 31 here on logger frame 4688 against the packet's 0.

### 68.4 The fix, and its killers

- **Built**:
  - `sim::fight::compare_target`'s raid arm (§68.1);
  - `sim::group::group_action_attack`'s `Build` arm (§68.2);
  - `sim::city::process_building`'s decay (§68.3);
  - `sim::anim::is_peasant`, now `pub(crate)`.
- **Unit tests**:
  - `a_raider_weighs_a_peasant_over_a_scout_and_a_building_at_a_twentieth`
  - `a_group_sent_at_a_build_takes_it_without_a_search` (1012's
    `a_group_retarget_from_a_building_takes_a_building`, rewritten: it
    passed without exercising anything once a `Build` stopped searching)
  - `a_building_s_targeted_count_is_quartered_on_its_owner_s_eighth_frame`
- **Mutations** on `76da8ed2`, each restored from git and `touch`ed:

  | mutation | unit test | run347's walk |
  |---|---|---|
  | `raiding` false | fails | falls back to 4607 |
  | the raid unit weights off | fails | falls back to 4688, index 31 |
  | a `Build` target searched again | fails | falls back to 4605 |
  | the AI raider's weight 9,000,000 | fails | holds 4690 |
  | the attack formula in place of `/20` | fails | holds 4690 |
  | no building decay | fails | holds 4690 |

### 68.5 What moved

- **Great Lakes 4688 → 4690.** On 4688 `1/24`'s re-search reads, on both
  sides, `0/3` at value 9800 (score 1225), the scout 15 (score 5) and the
  city 15 at `targeted` 0 (score 5). It keeps `0/3` and strikes, so block
  4689's `recharging` (0 against 33) and `hold_attack` no longer part.
  Frame 4688's draws went 35 against 35, parting at 31 → agreeing. On
  run356, `1/21`'s order (first parting 4761) and `1/24`'s (4722) no
  longer part in the window. The scout's target on 4737 reads 2808 again
  (1272 after 1023), against the original's 504.
- **East Indies holds at 5606**, and the second pair's other tests hold.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism is
  named): frame 4690, ours 5 draws and the original 4, parting at index 1.
  Ours spends `Object::take_damage+0xe1`, where the original spends
  `Farms::inc_time+0x1ae`.
  - ~~Nothing parts on block 4691.~~ The human's city `0/2000` had parted
    since block 4661, healed here off `1/26`'s first strike (§69.1); a
    row that stands is not a row that parts, and the block's new rows
    were read for it. The citizen `0/3` holds `damage 0` on both
    sides from 4688 to 4693, and first parts on 4723 (3 against 2).
  - `1/24`'s walk spot parts on 4689, (3763, 31600) against the cell
    centre (3768, 31608).

### 68.6 What is not established

- **The ship raid arm** (§68.1, `0064edd8`–`0064ef64`) and the raiding
  ship's two early zeros at `0064e7af`–`0064e821`.
- **`has_stance_type`'s refusal**: a type with no stance type never raids.
  This crate sets RAID only where a stance exists.
- **A detected hidden unit's `/4`** in place of the raid weights, and the
  spellcaster and spy bonuses above them (§12.3's, unchanged).
- **The best score's seed is 0** (`0064929e`, `-0x6c`), so a candidate
  whose value is 0 is never taken. This crate takes the first of those
  when nothing else stands. No capture reaches one.
- **A wall target's retarget** (the word 2) has no capture.

### 68.7 Coverage

- **Diff-backed**:
  - the raid flag and the unit weights, by frame 4688's draws and block
    4689 (both fall back under their mutations);
  - the `Build` arm, by frame 4605 (it falls back under its mutation).
- **Packet-backed**, and unit-backed, but not a floor:
  - the AI raider's `+900,000` against the human's 9,000,000 (run368:
    9800);
  - the building's `/20` (run368: the city's `80 → 4` in the trace);
  - the decay (run368: the city at 0).
- **Listing-backed**: `0064e6b6`–`0064e742`, `0064ea57`–`0064ea7a`,
  `0064edc4`–`0064eff9`, `0064f124`.

## 69. A city under attack does not heal (item 1034, 2026-09-28)

Item 1034 was booked on Great Lakes' second word, frame 4690 of run347:
ours 5 draws against the original's 4, parting at index 1. Ours spends
`Object::take_damage+0xe1` (§7.2 step 3, a building's first wound), where
the original spends `Farms::inc_time+0x1ae`. No mechanism was named.

### 69.1 The frame, from the disk

- **Who and what.** A scratch print of the frame's hits: `1/26` (type 32)
  strikes the human's city `0/2000` for 2 5/16. On our side the city's
  `damage` was 0 before the hit, so the roll was a first wound.
- **The original's city.** On run356 its `damage` reads 2/5 from block 4658
  (the first strike, frame 4657, on both sides) to 4689, and 4/10 on 4690.
  Its second strike found `damage` 2, so it threw no roll.
- **Ours.** The city healed: `damage` 2/5 → 1/0 on block 4661 → 0/0 on
  4665, one point per four frames (the city heal, `docs/CITIES.md` §8.2).
  The widening had carried the row since 4661 (`build:damage`, ours 1
  against 2). 1028's "nothing parts on block 4691" read the block's *new*
  rows (§68.5, struck).
- **The earliest parting on the city**, block 4658: `city_flags[0x2]` ours
  0 against 1, beside `raid_stamp` 4657. The dump's word is 18449 → 18463,
  `+0xe`: bits `0x2`, `0x4` and `0x8` together. `0x2` is the heal's veto
  (`docs/CITIES.md` §1.4), and this crate had no writer of it outside
  tests. The field was already compared; its row had stood unread since
  the window opened.

### 69.2 The listing

- **The setter.** `Object::take_damage@00652020`, the Build-proper branch
  (vslot `+0x20` non-zero), under `param_5 == 0` (combat, not attrition)
  and `this->who != param_8` (another player): after the under-attack
  latch's arms (`BuildData +0x60 |= 0x30`, skipped only for a peasant or
  `0x42` attacker outside its own territory), and whichever way they went,
  a building with a city (`BuildData +0x72 >= 0`) takes
  `orw $0xe, 0x4(%eax)` at `00652561`: `CITY_UNDER_ATTACK | CITY_ATTACKING
  | CITY_EVER_ATTACKED`, the PDB's names (`docs/ARMY.md`'s enum list). An
  AI owner at difficulty above 1 then pushes an alarm group (not built;
  the human's city never takes it).
- **The decay.** `Build::process@0061edf0`, after the `is_active` return,
  on an `OBJECT_CITY` building (`flags & 0x20`) with a city: every frame
  where `(frame + o) % 200 == 0`, `0x4` is cleared if set, else `0x2`. The
  same block decrements `CityData +0x61 plundered` (not carried). It runs
  ahead of the capture re-test and the heal, so a tick that clears `0x2`
  heals on the same frame.
- **The writers, counted** (823, 869). The decompile names four:
  `take_damage` (`|= 0xe`), the decay (a computed store), `City::init`
  (zero) and `City::capture` (kept, `docs/CITIES.md` §1.4). The listing's `or`/`and`
  of 2, 4, 8 or `0xe` at a `+4` offset: every other hit is another type's
  field (the Army's own flags, `Ammo`, `Guy`, the interface).
  `CITY_EVER_ATTACKED` has no clear.
- **The heal** (`Build::process`, `0061edf0`, `& 2) == 0` before the
  `city_heal_rate` phase) and **the AI's repair order** (every 64 frames,
  past half damage, `& 2) == 0`) both read `0x2`. The second is not built
  in this crate (`docs/CITIES.md` §8.2's "an AI owner also orders").

### 69.3 The readings, and what killed each

The frame named no mechanism. The disk named the heal in one row walk; no
reading preceded it. No packet was needed: the city's damage, its flags and
the phase are all printed, and the listing gave the setter and the decay.

### 69.4 The fix, and its killers

- **Built**:
  - `sim::fight::damage_building` sets `no_heal`, `attacking` and
    `ever_attacked` on the building's city under the latch's condition;
  - `sim::city::process_building`'s 200-frame decay, before the capture
    re-test;
  - `sim::city::City` carries `0x4` and `0x8`, and `City::capture` keeps
    all three;
  - `rondata::diff::harness` compares `city_flags[0x4]` and `[0x8]`.
- **Unit test**: `a_city_hit_by_another_player_stops_healing_for_two_
  decay_ticks` (`cities_tests.rs`): the owner's hit and attrition set
  nothing; another player's sets all three; no heal until the second
  tick, and the heal on that tick's frame.
- **Mutations** on `5bca6101`, each restored from git and `touch`ed:

  | mutation | unit test | run347's walk | run356's widening |
  |---|---|---|---|
  | the setter leaves `0x2` clear | fails | falls back to 4690, index 1 | the scout's 4737 row returns |
  | `0x2` cleared on the first tick | fails | holds 4781 | `city_flags[0x2]` parts on 4801, 0 against 1 |
  | no decay | fails | holds 4781 | `city_flags[0x4]` parts on 4801, 1 against 0 |

### 69.5 What moved

- **Great Lakes 4690 → 4781.** On run356, the city's `city_flags` `0x2`,
  `0x4` and `0x8` agree on every block from 4550 to 4806. That includes
  4658's set and 4801's clear of `0x4` (`(4800 + 2000) % 200 == 0`). Its
  `damage` holds 2/5 on both sides from 4658 to 4689, where ours healed it
  to 0 by 4665. Frame 4690's draws went 5 against 4 → agreeing.
- **Exposed, not introduced**: `1/26`'s strikes on the city from the second
  on land here a frame after the original's. The original's land on 4689,
  4716, 4722, 4750 and 4782; ours on 4690, 4717, 4723, 4751 and 4783. The
  city's `damage` parts on each strike's block and agrees on the next.
  The first strike (4657) agrees. The heal had hidden this row since 4661.
  **Closed by item 1040 (§70.2)**: the stones left the Slinger's own
  square, not its bay, and flew a frame long across a boundary.
- **The scout's explore target on 4737** (2808 against 504 since item
  1028) now agrees, and `1/0`'s order agrees through the window. Which
  read of the city moved it is not established.
- **East Indies holds at 5606** (ours 4, the original 5, at index 0, as
  booked).
- **The first pair's standing `0x2` rows close**: run100's window (the
  human city's, opened with `raid_stamp` on 9451) and run294's block
  19840.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism is
  named): frame 4781, ours 5 draws and the original 4, parting at index 0.
  Ours spends `Unit::fight+0x9b0`, where the original spends
  `Farms::inc_time+0x1ae`. Block 4782 is inside run356's window (232 after
  its first block, 24 before its last).
  - On block 4780 the citizen `0/4` has been struck there
    (`damage_frame` 4779, `damage` 3/5) and not here. **Moved to 4846, and
    on to 4877, by item 1040** (§70): the Slinger's stone left the unit's square, not its
    bay, and the struck citizen, not on duty, turned on it here.
  - On 4781 `0/4`'s order parts (kind 10 against 1, `orders_x` 1882
    against 2040).
  - `1/24`'s `g.cur_time` parts from 4753 and its facing from 4787.

### 69.6 What is not established

- **`raid_stamp`** (`CityData +0x18`), written by `Object::do_damage`'s
  building arm beside `S_RAID_ATTACKED`, is still not carried; its row
  stands from 4658, asserted against this crate's 0.
- **`CityData +0x61 plundered`**'s decrement in the same 200-frame block.
- **The AI owner's alarm push** after the `0xe` (`take_damage`,
  `Group::action_alarm`), and **the AI's repair order** that reads `0x2`.
- **The latch's peasant arm**: a peasant or `0x42` attacker outside its
  own territory skips `+0x60 |= 0x30`. This crate latches every foreign
  hit. The flags are set either way.
- ~~**`1/26`'s one-frame lag** from its second strike (§69.5).~~ **Closed
  by item 1040 (§70.2).**

### 69.7 Coverage

- **Diff-backed**:
  - the setter, by run347's frame 4690 (it falls back under its
    mutation) and by run356's `city_flags[0x2]`/`[0x4]`/`[0x8]` rows;
  - the decay's two stages, by run356's block 4801: `0x4` clears there
    and `0x2` does not (the second and third mutations).
- **Unit-backed only**: `0x2`'s own clear on the second tick, and the heal
  on that frame. No block on disk reaches 200 frames past a stage-one
  clear without a new hit (the window ends on 4806).
- **Listing-backed**: `00652561`; `Build::process@0061edf0`'s
  `(frame + o) % 200` block and its heal gate.

## 70. A Slinger's stone leaves its bay, a busy citizen does not turn, and a building is struck square (item 1040, 2026-09-28)

Item 1040 was booked on Great Lakes' second word, frame 4781 of run347:
ours 5 draws against the original's 4, parting at index 0. Ours spends
`Unit::fight+0x9b0`, where the original spends `Farms::inc_time+0x1ae`.
The widening (run356, block 4780) had the citizen `0/4` struck on 4779
there and not here. No mechanism was named. Two came out of the record,
one behind the other.

### 70.1 The frame, from the disk

- **The striker.** Every order on block 4774–4782 of run356 that names
  `0/4` is `1/24`'s, a who=1 Slinger (type 82, piece 32) standing at
  (2856, 31800). Its `recharging` jumps to 33 on block 4754 on both sides,
  and `CHAR_ATTACK2` (slot 12) plays from 4755 on both.
- **The launch agrees.** The trace's `Ammo::init+0xcd9`/`+0xd0b` pairs
  fall on 4647, 4680, 4704, 4707, 4713, 4737, 4740, 4746, 4764, 4770, 4773,
  4775 and 4780. A scratch print of this crate's launches gives the same
  thirteen frames.
- **The flight does not.** `1/24`'s stone of 4775 flew 6 frames here, from
  (2856, 31800) to (1852, 31802), and landed on 4780; the original's struck
  on 4779. `1/26`'s second stone at the city (4680) flew 11 here and 10
  there; its first (4647) flew 11 on both. That is §69.5's "exposed" lag.
- **Why.** Every Slinger stone left the unit's own square. `launch::
  launch_point` had no row for piece 32, so `from` equalled the unit's
  position, which is §22.3's "a piece with no row" fallback.
- **The second parting**, once the stone flew right: on 4780 ours spends
  `0/4`'s `fight+0x9b0` first (8 draws against 7). `0/4` holds `[MOVE to
  (2040, 31992), GATHER at 2003]`, walking to its drop site, on both sides
  from before the strike. The original's stack is unchanged on 4780 and
  4781. Ours pushed an `ATTACK` on `1/24`.

### 70.2 The Slinger's three bays, from run17

No dump on this disk has `AMMO` and `GUYS=4` over a Slinger. run17 has
`AMMO=5` over a Slinger fight at `GUY` detail 1, so it prints no
`cur_anim` (§22.4). The unit record's `recharging` names the animation
anyway:

- the stone leaves on the frame the release event is crossed, which is
  `starttime − 1` frames after the block `recharging` jumps (run356's
  4754 and 4775 for `1/24`, start 22);
- piece 32's three `<RELEASEEVENT>`s are distinct: 21 on `CHAR_ATTACK1`,
  22 on `CHAR_ATTACK2`, 16 on `CHAR_ATTACK3`.

So each of run17's 27 first-printed stones names its key, and the
offsets collapse onto §22.2's three vectors:

| key | stones | headings (whole degrees) | `(right, fwd)`, thousandths | `dz` |
|---|---|---|---|---|
| `CHAR_ATTACK1`, 21 | 3 | 2 | (69,776, 67,449) | 178 |
| `CHAR_ATTACK2`, 22 | 20 | 17 | (41,209, 118,012) | 151 |
| `CHAR_ATTACK3`, 16 | 4 | 4 | (−42,148, 113,222) | 177 |

Each row is item 813's `Bay`: the centroid of the region of integer
`(right, fwd)` that reproduces every stone of its key **exactly** under
`get_position`'s whole-degree rotation (`launch::Bay::point`). `dz` is
`sz` less the figure's `z`, the same on every stone of a key. The figure
is the `GUY` record on the stone's first block; the Slingers stand while
they throw. The regions are 5,467, 823 and 658 lattice points, at steps
of 10, 5 and 10.

Piece 384's two rows (item 485, run112's slingers) have the first two's
radii to the unit (97 and 125) and bearings 1.4° and 1.1° short of them:
the same sling model under another piece number, fitted there as a planar
`Node` from three stones.

**Piece 33, the Javelineers (type 83)** (item 1194; the evidence is
`docs/journal/2026-09-29-item-1194.md`). The `Spear` leaves node 0 on
frame 12 of `CHAR_ATTACK1`, 6 of `CHAR_ATTACK2` and 27 of `CHAR_ATTACK3`.
The rows come from `GraphicPieces::get_position@0090b750(33, 0, anim,
frame, deg)`, run under unicorn on run448's packet at every whole degree.
`deg` is `whole_degrees(facing) − 180`. Each row reproduces all 360
truncated points and run448's two `AMMO` rounds exactly:

| key | `(right, fwd)`, thousandths | `dz` |
|---|---|---|
| `CHAR_ATTACK1`, 12 | (38,941, 99,188) | 163 |
| `CHAR_ATTACK2`, 6 | (1,649, 89,737) | 170 |
| `CHAR_ATTACK3`, 27 | (21,769, 74,413) | 156 |

A packet holds a piece's entries only if its game fielded the piece (item
603).

### 70.3 `on_duty`'s return, from the listing

`Unit::target_opportunity@005fffc0`, after the flee arm:

```text
600863  mov   %esi,%ecx
600865  call  on_duty@005fff70
60086a  test  %eax,%eax
60086c  jne   600877              ; on duty → the retaliation
60086e  cmp   %eax,-0x1c(%ebp)    ; local_20 = order_type(), stored at 600150
600871  jne   600b16              ; any order → return
```

- **Every way past the flee arm reaches `600863`** (`600578`, `600656`,
  `60066e`, `600689`, `600696`, `6006c6`), the front-is-a-move skip
  included. The one way round is the action-is-an-attack arm, `600516`,
  `jmp 600877`.
- **`Unit::on_duty@005fff70`**: the type's `+0x2c8 & 0x10000` (the combat
  role) and `get_activity`'s order type in {2, `0xc`, `0x16`, 5, `0x15`}:
  `ATTACK_TO`, `GUARD`, `GROUP_PATROL`, `PATROL`, `GROUP_ATTACK_TO`.
- **`UnitData::get_activity@00608370`**: the first order that is neither
  a move nor an attack (vslot `+0x1c`). When every order is one, it is the
  last, answered only if it is an `ATTACK_TO` or a `0x15`.

So a citizen with any order never answers a hit. A soldier answers one
only while it attack-moves, patrols or guards. An idle unit of any kind
still answers (`local_20` is `NONE`).

### 70.4 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| the lead on a moving target | `0/4`'s `dest` is `None` at 4775 on our side, and its position agrees to 4782 | killed |
| the launch frame | the trace's `Ammo::init` frames are ours, thirteen for thirteen | killed |
| the launch point | with the bays, `1/24`'s stone lands on 4779 and the city's strikes on the original's frames. With piece 32's rows keyed away (the mutation), the unit test fails and `0/4`'s strike row returns on 4780 | **held** |
| a busy citizen does not retaliate | with the gate, 4780 agrees and `0/4` keeps its stack. With the gate off (the mutation), the walk falls back to 4780 at index 0 on `0/4`'s `fight+0x9b0` | **held** |

### 70.5 The fix, and its killers

- **Built**:
  - `sim::launch::BAYS`: three rows for piece 32, appended;
  - `sim::fight::Sim::on_duty` and `action_is_attack`;
  - `target_opportunity`'s return after the flee arm;
  - `sim::fight::Sim::building_side`, read by `fight`'s facing (§70.6);
  - the one-in-five retarget's mark in `sim::orders` (§70.7).
- **Unit tests**:
  - `run17_s_slinger_stones_leave_from_the_measured_bays`: all 24
    distinct launch points and heights, exactly;
  - `a_busy_unit_answers_a_hit_only_on_duty`: a walker, a soldier on a
    plain move, a soldier attack-moving, a non-combat unit attack-moving,
    an idle unit.
  - `guard_on_post`'s chariot is now combat-role, as the loader gives
    the real type. 707's AI guard answers through `on_duty` (`GUARD`).
  - `a_building_is_struck_square_to_its_side`: the four faces, a corner,
    inside, an unblocked tile and a unit target;
  - `the_one_in_five_retarget_freezes_the_frame`.
- **Mutations** on `c50d00c3`, each restored from git and `touch`ed:

  | mutation | unit test | run347's walk | run356's widening |
  |---|---|---|---|
  | piece 32's rows keyed to 9032 | fails | falls back to 4810 | `0/4`'s strike row returns on 4780 |
  | the gate off | fails | falls back to 4780, index 0 | `0/4`'s `ATTACK` returns on 4780 |
  | the side arm answering `None` (on `179f2fc2`) | fails | falls back to 4846, index 2 | none then; run356's `1/22` row is asserted since |
  | the retarget's mark off (on `179f2fc2`) | fails | falls back to 4852, index 1 | `1/24`'s clock parts on 4853 (run373) |

### 70.6 A building is struck square to its side

The same item's second word, 4846 (ours 8 draws, the original 9, at index
2): the original spends `Guy::set_anim+0xf2f < Unit::set_anim+0x56 <
Unit::fight+0x19f6`, the swing, and ours does not. run373 (§70.8) is its
widening.

- **The record.** Block 4847 parts on `1/19` alone, a Hoplite (guy type
  132) of who=1's army, on its attack spot (3432, 30120) on the row above
  the human's city (centre (3168, 30816)). On 4846 the original's `1/19`
  strikes with its facing unchanged, `0x80000000`, due south: `recharging`
  32 and slot 11 on block 4847. Ours turned to `0x8ec5793b`, the centre's
  bearing. `set_anim`'s deferral (ANIM §6.2) then held the swing, and it
  struck on 4847.
- **The disk, before any reading.** Every strike on the city in run356 and
  run373 was listed with its facing and the centre's bearing. The Slingers
  and Bowmen face within a tenth of a degree of the bearing. The Hoplites on the north
  face do not: `1/22` turns from `0xc0000000` to exactly `0x80000000` on
  block 4758, where the bearing is `0x959a59e3`. That is run356's standing
  `1/22` facing row, open since 4758 and unread.
- **The listing** (`5fe8a7`–`5feb4c`). After `find_angle` to the target's
  `+0x10/+0x14`, vslot `+0xc` is asked of the target. It is
  `SubObjectData::is_active` on `Build` and `Wall` and a folded `return 0`
  on `Unit` and `Animal` (`vtables.txt`). For a building:
  - the unit's own tile covered (`WallData::covers_tile@006439b0`, tiles
    `div_3_table[v >> 6]`) keeps the bearing (`5fe8f6`);
  - else the tile at `x − 0xc0` covered **and** its world mask (`+0x138`)
    carrying `0x4000`, `BLOCKED` → `0xc0000000`;
  - else `y − 0xc0` → `0`; else `x + 0xc0` → `0x40000000`; else `y + 0xc0`
    → `0x80000000` (`cmovne` at `5feb46`);
  - anything else keeps the bearing.

  The sideways ship's quarter turn follows (`5feb51`). Ghidra prints the
  arm as a wall's, and it is every building's.
- **Two readings killed first.** A pivot (`set_attack` answering, §52):
  `set_attack@005fce70` asks the type's `max_range`, and a Hoplite's is 0.
  The swing's fifth argument (`jne 5feec6`): `5feec6` is the return address
  of the swing's own `set_anim` call, so the trace's `+0x19f6` is the
  ordinary path.

### 70.7 The one-in-five retarget freezes the frame

The third word, 4852 (ours 6 draws, the original 5, at index 1): ours
spends `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, an attack slot's wrap
to the idle. Both sides spend `1/24`'s `fight+0x9b0` first (roll 54128,
`% 5 = 3`), and on both the re-search names `0/3` in place of `0/4`.

- **The record.** Block 4853 has the original's `1/24` at `cur_time` 32
  of 33 with `last_time` 32, a clock that did not step, and `unit_masks2`
  **16**. It wraps on 4854. Ours wrapped on 4852 and rolled the idle.
- **The listing.** After `find_new_target@005ff6a0` names another valid
  target, `005fdf68`–`005fdfea` checks `order_type() == ATTACK` and
  `recharging` (`+0xae`) zero, re-points the head order, and
  `orl $0x10, 0x6c(%ebx)` before the return. That is §43.2's frozen mark,
  which `Guy::inc_time`'s step reads (`local_14 = 0`). This crate set it
  on the invalid-target and guard arms and not here.
- **SEAM**: the `jmp 005fd639` taken otherwise (a recharging unit, or the
  search naming `fight`'s entry arguments), which re-enters `fight` from
  its head.

### 70.8 What moved

- **Great Lakes 4781 → 4846** (§70.1–§70.3). On run356:
  - `1/24`'s stone of 4775 strikes `0/4` on 4779 on both sides (block
    4780: `damage_frame` 4779, `damage` 3/5, where ours read 0 and 0/0);
  - `0/4` keeps `[MOVE, GATHER]` on both, where ours pushed an `ATTACK`
    (kind 10, three orders); none of its rows parts in the window;
  - `1/26`'s strikes on the city land on 4689, 4716, 4722, 4750 and 4782
    on both sides, and `build:damage` no longer parts (§69.5's lag);
  - 122 first-parting rows close, and none opens. They include `1/1`'s
    and `1/4`'s walks, `1/15`'s path on 4792 and `1/24`'s facing on 4787.
  - Frame 4781's draws went 5 against 4 → agreeing.
- **4846 → 4852** (§70.6). On run373, `1/19` strikes on 4846 facing south
  on both sides, and its figure first parts on 4861. On run356, `1/22`'s
  facing row (from 4758) closes. Frame 4846's draws went 8 against 9 →
  agreeing.
- **4852 → 4877** (§70.7). On run373, `1/24`'s clock no longer parts on
  4853. Frame 4852's draws went 6 against 5 → agreeing.
- **East Indies holds at 5606**, ours 4 against 5 at index 0, as booked,
  at every step.
- **The new word, by frame and draw delta** (DECISIONS 42; no mechanism is
  named): frame 4877, ours 7 draws and the original 8, parting at index 1.
  **Moved to 4924 by item 1052**: block 4861 was the army's first move of
  the fight, formed a cell forward of its point here and a cell behind
  there (`docs/ARMY.md` §23).
  The original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`, a
  unit setting off. Ours spends `Guy::set_anim+0x104b`. Block 4878 is inside
  run373's window.
  - **The earliest block to part past the window's standing rows is 4861**
    (121 keys): frame 4860 is who=1's army tick (`4860 ≡ 252 mod 256`). The
    army's members (among them `1/12`–`1/19`, `1/22` and `1/24`) take new moves there
    (`order:move.dest` 1, walking), and hold the city here.
  - The window opens on a standing row: the citizens `0/2`, `0/3` and
    `0/4` have taken more damage here since run356's window closed (block
    4841: `0/4`'s `damage` 6 10/16 against 5 0/16).

### 70.9 What is not established

- **An attack action with no target** takes the `600516` arm in the
  original, whose own returns and `kill_current_order` this crate does not
  carry. It keeps the old path here (SEAM).
- **`get_activity`'s `is_move_attack`** is read as this crate's `is_move`
  plus the attack and ground-attack bodies, as `guard_activity` reads it.
- **The planar fallback for an unmeasured piece** still launches from the
  unit's square. The next ranged type a word reaches will ask the same
  question: run17's method needs only `AMMO=5` and the unit's `recharging`.
  ~~Piece 33 was the next~~: its rows are §70.2's. Other civilizations'
  javelineer pieces are still unmeasured.
- **A Slinger at a heading run17 never shows.** The bays are exact on
  twenty headings. The rotation is `get_position`'s, and it is read, not
  measured, between them.
- **Vslot `+0xc` on a building** is taken as `Sim::active`. The folded
  `SubObjectData::is_active` was not read for the flag it tests. A
  building under construction is the case that could differ.
- **The side arm's west, north and east answers** have no capture. Only
  the south face is struck on disk (§70.6).

### 70.10 Coverage

- **Diff-backed**:
  - the bays, by run356's block 4780 and the city's strike blocks, and by
    run17's 24 points (a unit test that pins the dump's columns);
  - the `on_duty` gate, by frame 4780's draws and `0/4`'s stack;
  - the side, by run373's block 4847 and run356's `1/22` from 4758;
  - the frozen mark, by run373's block 4853 and frame 4852's draws.

  Each is a floor that falls back under its mutation (§70.5).
  - piece 33's bays, by run448's two rounds
    (`run448_s_javelins_leave_from_the_measured_bays`) and run442's Farm
    on 16681. Without them, the long walk falls to 16681.
- **Listing-backed**: `600863`–`600871`, the branches into it, and
  `600516`; `5fe8a7`–`5feb51`; `5fde80`–`5fdfea`.
- **Decompile-read**: `on_duty@005fff70`, `get_activity@00608370`,
  `covers_tile@006439b0`.
- **Unit-backed only**: `on_duty`'s `PATROL`, `GROUP_PATROL` and `GUARD`
  arms for a human unit, the idle unit's answer, and the side arm's other
  three faces.

## 71. A chase in reach is not ended while its target runs from the chaser's heading (item 1061, 2026-09-28)

Great Lakes' second word, frame 4924 (DECISIONS 53, 54). The item's
booking read block 4923 as the original pushing the chase's move above
`1/11`'s `ATTACK`. The disk says the other way round: both sides carry the
chase from block 4921, and **ours ends it a frame early**.

### 71.1 The frame, from the disk

- **The whole cast first.** A probe (`RON_STACKS=1` on the second pair's
  widenings) prints every AI unit's order stack front first on both sides,
  on every block of run356 and run373. Across 4550..4922 there are 72
  chase pushes and pops, and every one is on the same block on both sides.
- **The first one-sided event is the word's.** Ours pops `1/11`'s chase
  on block 4923; the original pops it on 4924. The same early pop recurs
  on `1/26` (4938 against 4947) and `1/25` (4963 against later).
- **Block 4922 agrees whole.** `1/11` stands at (5046, 30225) with heading
  −321454080, `MOVE` over a fresh `ATTACK` on the citizen `0/1` (`in_range
  0`, `new_ord 1`) over its `ATTACK_TO`. `0/1` is at (4035, 28584), heading
  `0x40000000`, walking east at 25.
- **Neither frame is the review's.** `check_target_path`'s phase is
  `(frame + 11) mod 16`, 5 on frame 4922 and 6 on 4923. So the pop is
  `do_move`'s own kill (§35.3).

### 71.2 The listing

`Unit::do_move@005f7b30`'s kill on a ranged attacker's unit target is
`LAB_005f7faa`, reached only when all three hold:

1. **Not the flank** (`5f7fbe`–`5f7ff6`):
   - `ecx = target +0x50 − this +0x50 − 0x80000000`;
   - `cmp $0x2aaaaaaa` then `jb`: below it, go to the range test;
   - `call flanking@0092cfe0`, which reads `ecx`; zero, go to the range
     test;
   - `call *0xd8(target)`, `UnitData::is_moving`; zero, go to the range
     test;
   - otherwise `jne 5f803f`, the captain's retarget, past the kill.
2. `is_in_range@00648d70` with the sixth argument `mandatory == 0` (§35.3).
3. `Objects::find_collision(own spot, o, who, 1) == 0` (`5f7f9d`).

This crate carried the second alone. §35.3 and `orders.rs` named the first
and third as `SEAM`s "that only make it rarer". Rarer is the word's
direction: a later pop.

The flank triple is `check_target_path`'s (§67), with the angle taken
from the attacker's own heading rather than the bearing to the target. The
decompile prints `flanking`'s argument as `unaff_EDI`; the listing puts
`e` in `ecx`, and `flanking`'s first instruction is `cmp $0xd5555555,
%ecx`.

### 71.3 The values

| frame | `1/11` heading | `0/1` heading | `e` | `flanking(e)` | flank holds |
| --- | --- | --- | --- | --- | --- |
| 4922 | −321454080 | `0x40000000` | `0xd3290000` | 2 | yes: no kill |
| 4923 | −319422464 | 1480523776 | `0xeb490000` | 0 | no: kill |

`0/1` is player 0's unit, stepped before `1/11` in the frame. On frame
4923 it has already turned to the heading block 4924 prints.

### 71.4 The fix, and its killer

- **Built**: `Sim::chase_target_flees`, which `do_move`'s unit-target kill
  asks before `is_in_range_at_margin`.
- **Unit test**: `a_target_running_from_the_chaser_s_heading_keeps_the_chase`.
  A ranged chaser in reach, off the review's phase. A target walking
  along its heading keeps the chase; a standing one, or one walking toward
  it, ends it.
- **Mutation** on `1d641483`, the conjunct taken out, restored from git and
  `touch`ed:
  - run347's walk falls back to 4924, 8 against 7 at index 0;
  - `run373_s_word_frame_is_widened_whole` fails on `1/11`'s
    `order:kind` at 4923;
  - the unit test fails.

### 71.5 What moved

- **Great Lakes 4924 → 4978.**
  - On block 4923 `1/11` stands at (5032, 30200) under its chase on both
    sides.
  - On 4924 it stands there under a fresh `ATTACK` on `0/1`.
  - On 4925 it stands at (5016, 30216) on `0/2`, with `in_range 1`,
    `new_ord 0`, `recharging` 30 and `hold_attack` 1, on both sides.
  - Frame 4924 went 8 against 7 → agreeing.
  - run373's keys parted go 1,093 → 1,052.
- **East Indies holds at 5606**, ours 4 against 5 at index 0.
- **The new word** (no mechanism is named): frame 4978, ours 9 and the
  original 8, at index 1. Ours spends `Unit::close+0xcb6`, the original
  `Farms::inc_time+0x1ae`.
  - Block 4979 holds the citizen `0/2` dead on ours' side alone
    (`death:extra`, `hold_frames` 1), at 42 damage against 37.
  - The gap stands from run373's first block, 6 10/16 against 5 5/16.
  - The original's damage falls a point on 4859 and on 4904; ours' does
    not.

### 71.6 What is not established

- **`find_collision`'s conjunct** is still a `SEAM`. Block 4980 is its
  first instance on disk: `1/18` keeps its chase in the original with
  `collide 1`, `collide_o 19`, where ours ends it. That is past the word.
- **`is_moving` on a unit whose head is not a move.** This crate's
  `is_moving` reads the head order. `UnitData::is_moving`'s body was not
  read.
- ~~**The re-search's fresh order.** On every one-in-five retarget on disk
  (4753, 4853, 4898 and more), the original's head attack reads fresh for
  one block (`in_range 0`, `new_ord 1`, `ever_in_range 0`), and this
  crate's is re-pointed in place (`retarget_attack`). The values agree
  again on the next block and no draw has parted on it yet
  (`docs/GROUPS.md` §33.2).~~ Built by item 1099 (§80): the
  re-search is `find_new_target`, and the order it adds is fresh.

### 71.7 Coverage

- **Diff-backed**: the pop's frame, by run373's blocks 4923..4925 and
  frame 4924's draws; every chase event on run356 and run373 to 4979.
- **Listing-backed**: `5f7fbe`–`5f7ff6`, `92cfe0`–`92cffd`.
- **Unit-backed only**: the target walking toward the chaser. No capture
  has one in reach.

## 72. A citizen heals on its own ground (item 1072, 2026-09-28)

Great Lakes' second word, frame 4978 of run347 (DECISIONS 53, 54): ours 9
draws against the original's 8, parting at index 1. Ours spends
`Unit::close+0xcb6`, the original `Farms::inc_time+0x1ae`. On run373's
block 4979 the citizen `0/2` is dead on ours alone, 42 damage against 37.
The booking named no mechanism and called the heal a hypothesis.

### 72.1 The frame, from the disk

- **Every instance first** (1058). A scan of every unit's `damage` and
  `damage_frac` on run356 and run373, both sides, lists 22 falls with no
  hit. All are the human's citizens `0/1`..`0/5`. Each takes one point
  off and zeroes the fraction, and each sits on the frame `(o + frame) %
  45 == 0`, the block one later. Two examples: `0/3` on 4722 (3/5 → 2/0
  on block 4723) and `0/2` on 4858 (5/5 → 4/0 on 4859).
- **The field that counts it.** `healing` (`ObjectData +0x38`) reads 45
  on each such block and counts down a frame at a time. On run356 it
  reads 0 on `0/2` through 4806. On run373 it reads 18 on 4841, so it
  was set on 4813, in the gap between the two windows.
- **The gap, by arithmetic.** On 4806 `0/2` stands at 3/5 on both sides.
  The original heals it on 4813 (`(2 + 4813) % 45 == 0`) to 2/0. `1/9`'s
  hit on 4839 adds 3/5 and gives 5/5. Ours never healed, so it read 3/5
  + 3/5 = 6/10. That is run373's first-block row exactly.
- **The deaths.** This capture prints `DEATH_OBJS` only in its quit
  block (5111). There the original's `0/3` has `first_frame` 4996 and
  `0/2` has 5000. Before the heal, ours killed `0/2` on 4978, and
  `Unit::close+0xcb6`, the death draw (§42.1), was the word.

### 72.2 The listing

`Unit::process_healing@005e0670` is called by `Unit::process@00610bc0`
for every unit, after `healing` is counted down at the head of `process`.
Its head returns unless four things hold:

1. the unit is a captain;
2. it is not air (`domain` `+0x218` != 2);
3. `UnitData::has_damage@00609b50(1)` holds: the unit or a figure below
   it has `damage` or `damage_frac`;
4. `inside_up` (`+0x82`) is negative. Otherwise it takes the garrison
   branch (`docs/CITIES.md` §6.7).

On the map, a sea unit takes the ship heal and returns, and a supply unit
(`UnitData::is_supply`, type `+0x2b8 & 0x40`) returns. Six heals follow
that need a nation bonus, Versailles, a general, a CtW hero or a supply
period that ships as 0 (`docs/SUPPLY.md`, "Consumer 2"). The last one is
the civilian heal:

```
CIVILIAN_HEAL_RATE != 0 and (o + frame) % CIVILIAN_HEAL_RATE == 0
  and !(unit_masks2 & 1)
  and (is_worker or is_caravan or is_merchant or type == 0x13d)
  and (domain == SEA or the cell's owner (+0xf) >= 0 and is_ally):
    repair_damage(1, 0, 1)
    healing = max(CIVILIAN_HEAL_RATE, healing)
```

- `ObjectData::is_worker@0046fa10` is the type ids `0x32`..`0x35`,
  compared by identity. The `0x13d` test is an identity test too, not a
  lineage test.
- `rules.xml` ships `CIVILIAN_HEAL_RATE` as "45 frames (0 means don't heal
  at all)".
- `Unit::repair_damage@0060de10`:
  - first clamps `damage` to `hits(0)`;
  - then, **when `damage` is non-zero**, takes `param_1` off it (floored
    at 0) and writes `+0x3b` (`damage_frac`) to 0;
  - with `param_2` 0 it adds no sparkle, so the heal spends no draw.
- The sea arm of the territory test cannot be reached: a sea unit has
  already returned.

### 72.3 What this crate built

- **`sim::heal`**: `Sim::civilian_heal`, called in the unit's process
  step right after `garrison_heal`. It checks the head's gates, the
  supply return, the civilian test and the cell's owner through
  `is_ally`. It then heals a whole point and zeroes the fraction.
- **`Tuning::civilian_heal_rate`** is 45, checked against the install's
  constants (`CIVILIAN_HEAL_RATE`).
- **`healing` is not carried.** The aircraft heal in `Unit::process`,
  which writes it too, is still a seam (`crate::cast`). The field stays
  on the compared pin's uncompared list.
- **Unit test**: `a_citizen_heals_a_point_every_45_frames_in_its_own_territory`.
  - It checks nothing off the phase.
  - On the phase it checks a point healed and the fraction cleared.
  - It checks that a soldier (`0x60`) does not heal.
  - It checks that nothing heals on a cell nobody owns.

### 72.4 The killers

Each was run as a mutation, with `git diff --stat` non-empty: the first
on `a99cb914`, the second on `5fddc659`. Each was restored from git and
`touch`ed after.

| mutation | unit test | run347's walk | run373 | run356 | the first pair |
|---|---|---|---|---|---|
| the fraction is kept | fails | falls to 4993, 11 against 9 at index 1 (`Unit::close+0xcb6`) | fails | `0/3 damage_frac` on 4723, 5 against 0 | not run |
| the territory test dropped | fails | holds 5042 | holds | holds | run80 and run136 hold |

### 72.5 What moved

- **Great Lakes 4978 → 5042.**
  - On run373, `0/2` reads 5/5, 4/0, 3/0, 16/0 and 39/5 on blocks 4841,
    4859, 4904, 4949 and 4994, on both sides. Before, ours read 6/10 from
    4841 and died on 4978 at 42.
  - `0/1`, `0/3`, `0/4` and `0/5` close the same way.
  - Both sides now lose `0/2` on frame 5000.
  - Frame 4978's draws went 9 against 8 → agreeing.
  - run373's keys parted go 1,052 → 471. Parked 1074's first instance
    (`1/18`'s collide on 4980) went with the rest: it followed `0/2`'s
    early death.
- **run356**: `0/3`'s heals on 4722 and 4767 agree. Its row, ours 3
  against 2 from 4723, closes.
- **The first pair**: `0/5`'s four damage rows close in every Great Lakes
  window from run136 to run80. They had stood from 11400 at ours 2/2
  against the original's 0/0. Every word holds: 24,000, the chapters,
  and East Indies' 5606.
- **Great Sahara** (1066's third map) holds at 8.
- **The new word** (no mechanism is named): frame 5042, ours 8 and the
  original 6, at index 0. Ours spends `Ammo::do_damage+0xc59`, the
  original `Farms::inc_time+0x1ae`.
  - It is inside run373's window, on block 5043.
  - On block 5041 the citizen `0/1` takes 3/5 on the original's side
    alone: 2/0 → 5/5, with `damage_frame` still 5038.
  - Nothing first parts on 5042 or 5043.

### 72.6 What is not established

- **`unit_masks2 & 1`, the civilian heal's veto.**
  - `UnitData::recharge@0060fdf0` reads the same bit as "under attack".
  - `Unit::process`'s 32-frame block decays it: `0x2` first, then `0x1`.
  - Neither the decompile nor a grep of the listing's `or` into `+0x6c`
    names a setter.
  - No dumped unit on run373 holds `0x1` or `0x2`; the only values
    printed are 0 and 16.
  - This crate reads the bit and never sets it.
- **The other on-map heals**: the ship heal, Antipater/Wellington, the
  CtW hero, the Iroquois, the economic patriots and supply. None is built
  here, and no capture on disk reaches them.
- **`healing`**, the aircraft heal that also writes it, and the army
  stay-and-heal arm that reads it (`army.rs`, `think_attack_join_army`).
- **`damage_frame` on a hit that leaves it alone.** The original's `0/1`
  on 5040 and `0/2` on 4978 take damage with `damage_frame` unchanged.
  This is the new word's block and the next item's.

### 72.7 Coverage

- **Diff-backed**:
  - the phase, the point and the cleared fraction, by run373's and
    run356's citizens (22 heals) and by the first pair's `0/5`;
  - the heal drawing nothing, by run347's walk through 5042.
- **Listing- and decompile-backed**: `005e0670`'s gates, `0060de10`'s
  clamp and its `damage != 0` arm, `0046fa10`.
- **Unit-backed only**: a soldier not healing, and the territory test.
  No capture on disk has a damaged citizen off an ally's ground on its
  heal phase. The first run of the territory mutation read run80 as
  failing, but that was a floor pin still unmoved on the merged tree.
- **Not backed**: a caravan, a merchant, a fisherman, a supply unit, a
  sea unit and the veto bit.

## 73. A round with no target strikes the building it lands on (item 1077, 2026-09-28)

### 73.1 The frame

Chapter thirty-five's V2 (`docs/GOLDEN.md` §44, run371). Its round is
fired at a point, not an object: `whom −1`, `ox −1` on every block it
flies (`docs/PRODUCTION.md`, "The missile's launch and round"). It comes
down on 2820 at (13834, 14969). That is on who=1's Barracks `1/2006` at
(13824, 14976), a 5×5 footprint, with no unit of any player within two
tiles.

- **The dump**: `1/2006` prints `damage 0`, `myhits 1200` on 2819 and
  2820, and no `BUILDDATA` on 2821 or after. The round's `AMMO` record
  is on 2820 (`cur_time 119`) and gone on 2821. Nothing else on the
  frame moves but the clocks, who=1's economy and who=0's
  `leader_flags` (19 → `0x0A000013`, a field this harness does not
  compare).
- **This crate**, before item 1077: the Barracks at 400 damage on 2821
  and standing to the end of the run. `RON_DEBUG_BLAST`, a scratch
  print, showed the one hit: `do_damage` with `count` 256, **`splash`
  1**, attack 1500 (tenths), table 1075%, armor 3. So `16125 × 25 / 100
  = 4031`, and `(4031 + 5) / 10 − 3` = 400.
- Struck with `splash` 0 it is `(16125 + 5) / 10 − 3` = 1610, past the
  Barracks' 1,200.

The draw stream agrees across the frame either way: the building's first
wound draws its `% 100` on both sides (§9.5), and its fall takes no draw.

### 73.2 The rule

`Ammo::do_damage@00678060`'s splash arm gives `splash` 0 to the object
`hit_target`/`check_hit` left in `whom`/`ox`, and `splash` 1 to every
other object its walk reaches (§9.3). A round with no target fails
`hit_target`, and **`Ammo::check_hit@00678d90`** then names it (§9.4,
the listing `678d90`..`678f7f`):

1. `ObjectsData::find_unit` at the landing point, radius `0x180`, in the
   ammo's domain class (the call at `678e13`). A unit found beyond its
   own `target_size` is dropped (`678e4c`..`678e61`).
2. **Failing a unit**, and only for a landing inside the world
   (`678e7a`..`678eac`: `0 ≤ x < world+0x18 × 0xc0`, the same for `y`),
   `ObjectsData::find_building_at@0065ab40` on the landing's own tile,
   `div_3_table[c >> 6]` (the call at `678ece`). Its answer is `whom`
   (`678ed9`), and a building found returns 1 (`678ee7`).

`find_building_at(tx, ty, SEARCH_ALL, −1, FILTER_ALL)`:
- the tile inside the world and marked as a building's (`& 3 == 3`);
- the 3×3 cells round the tile's cell, in `move_x`/`move_y` order, and
  each cell's object chain;
- the first object of a player below eight, `is_active` at vslots `0xc`
  and `0x4c` (`SubObjectData::is_active`, `WallData::is_active`), whose
  footprint holds the tile: `WallData::tile_corner` ≤ tile < corner
  plus the type's `+0x234`/`+0x238` (`x_size`/`y_size`).

So **the building a round comes down on is its target**, struck whole,
whenever no unit stands within two tiles. §9.4 said so; the code did
not.

### 73.3 What this crate had

`check_hit`'s second half compared the landing's tile against the
building's point in **position units** (`|tile − pos| ≤ size × 96`). A
tile is under 256 and a point is in the thousands, so it found a
building only near the map's corner, and in no capture on disk. Every
shot on disk that missed and came down on a building went into the
ground (no splash) or struck it as a fringe (splash).

### 73.4 Built

`Sim::building_under` (`crates/sim/src/fight.rs`): the world bound, then
the first building of a player below eight, active (this crate's
`Sim::active`), whose footprint (`Sim::covers_tile`, the same
`tile_corner`) holds the landing's tile. `check_hit` falls back to it.

- **Unit test**: `fight::tests::a_round_with_no_target_strikes_the_building_it_lands_on`,
  on run371's own geometry. It fails on the old comparison: `check_hit`
  answers `None`, and the Barracks takes the fringe's damage.
- **The value diff**: `chapter_thirty_five_s_v2_blast_is_compared_field_for_field`,
  run371's blocks 2818..2822, both directions. It covers every
  building's presence and `damage`, every player's unit's presence, and
  the V2's round. 261 rows, none parting: `1/2006` at `damage 0` on
  2820 on both sides, and gone on 2821 on both.
- Chapter thirty-five's word stays at 3260. Its widening goes 43 → 42:
  the V2 row on 2821 is gone.

### 73.5 What is not established

- **A shot without splash** that misses and comes down on a building now
  strikes it where it punctured the ground before. That trades two
  puncture draws (§39) for the building's wound. No pin on disk moved
  (the gate), so no capture on disk has one. None has been seen on
  either side.
- **The walk's order** among buildings whose footprints hold one tile.
  Footprints do not overlap, so this crate takes the lowest index. A
  wall's (`WallData`) footprint was not read against it.
- **`find_building_at`'s tile mark** (`world+0x138`, `& 3 == 3`) is taken
  as "a building's footprint is on the tile". This crate reads the
  footprint, not the mark.
- **who=0's `leader_flags` gains `0x0A000000` on 2821.** Its writers of
  `0x2000000` are `Unit::init`, `Object::insert_inside` and
  `remove_from_inside`, none of them on this frame. The writer of
  `0x8000000` was not found by a grep of the decompile. No reader in
  this crate, and uncompared.
- The missile arm's other branches stand as PRODUCTION lists them: a
  nuke, leader `+0x7c0` and `S_NUKE_HIT`, which print nothing at
  `LEADERS=2`. ~~`MISSILE_DEFENSE_BONUS`~~ is built at the arm's head
  (item 1078, `Sim::land`; `docs/PRODUCTION.md` "The missile's other
  arms"): run390's V2b closed in who=1's land on 3169.

### 73.6 Coverage

- **Diff-backed**: the V2's strike on the Barracks, by run371's blocks
  2818..2822 (the value test) and the widening's run371 whole.
- **Listing-backed**: `check_hit`'s building arm, `678e7a`..`678ee7`.
- **Reading-only**: `find_building_at`'s walk and predicates
  (decompile).

## 74. A fleeing target is led by its order, along its angle (item 1081, 2026-09-28)

Great Lakes' second word, frame 5042 of run347 (DECISIONS 53, 54): ours 8
draws against the original's 6, parting at index 0. Ours spent
`Ammo::do_damage+0xc59` and `+0xc7e`, a round puncturing the ground
(§39); the original spent `Farms::inc_time+0x1ae`. On run373's block 5041
the citizen `0/1` read 5/5 on the original's side alone, `damage_frame`
still 5038. The booking named no mechanism and no direction.

### 74.1 Every instance on the disk first

- **`0/1`'s wounds, both sides, 5028..5045** (run373; no `AMMO` record
  is printed, so the rounds are this crate's side only, from a new
  `RON_DEBUG_AMMO` probe):

  | block | the original | ours (before) | the round |
  |---|---|---|---|
  | 5038 | 0/0, `damage_frame` 4890 | same | — |
  | 5039 | 3/5, `damage_frame` 5038 | same | `1/10`'s, fired 5019, hits on 5038 |
  | 5040 | 2/0 (the heal, §72, `(1 + 5039) % 45 == 0`), `healing` 45 | same | — |
  | 5041 | **5/5**, `damage_frame` 5038 | **2/0** | `1/11`'s, fired 5024, due 5040 |
  | 5045 | 8/10 | 5/5 | `1/9`'s, fired 5031, hits on 5044 |
  | 5077 | 12/4, `damage_frame` 5076 | 5/5, 5038 | — |

- **5038 is a real frame, not a window's edge** (parked 1083 b): block
  5038 prints `0/1` with `damage_frame` 4890. `1/10`'s round strikes it
  on 5038 and the dump's next block carries it.
- **`damage_frame` standing at 5038 on 5040's hit is the overkill
  window** (§7.1 step 2): `5040 − 5038 < OVERKILL_FRAMES` (30), so the
  record keeps the first striker. Both sides agree on it; it is not the
  parting.
- **The direction: ours missed a hit, the original did not add one.**
  `1/11`'s round came due on 5040 in ours, `hit_target` and `check_hit`
  found nothing, it rolled on (§42.2) and came down in the ground on 5042,
  spending the two puncture draws the word saw. The original's struck
  `0/1` on 5040 with no draw, as a hit spends none here.
- **The three shooters agree whole.** `1/9`, `1/10` and `1/11`
  (type 120) part on no row of their own on 5028..5045: position,
  `recharging`, the figure's clock, the attack row.

### 74.2 The round that missed

`1/11`'s round left (5070, 30149) on frame 5024 with a flight of 17 frames.
Its target `0/1` was fleeing: at (6222, 28816) on block 5025, heading
550174720, its figure's `avg_speed` 23. Ours landed it on (6225, 28816),
three units off the target and **with no lead**. On 5040 `0/1` stood at
(6260, 28490), 352 units away, and the hit test halves that at accuracy
300: 176, past its `target_size`.

The probe of every unit-targeted round on run347 to 5097 (29 rounds)
found exactly three where this crate's gate and the order disagreed, all
a `FLEE_TO` head with `movement.dest` empty: 4962 (`1/10` on `0/2`), 5024
(`1/11` on `0/1`) and 5064 (`1/9` on `0/1`). A fleeing citizen between
two legs of its path holds its flight order and no body destination.

### 74.3 The listing

`Ammo::init@0067bbf0`, past the lofted, ground and bomber tests:

```
67ce62: call 0x616e80        ; UnitData::order_type(target)
67ce69: call 0x46f050        ; is_move(order)
67ce70: jne  0x67ce9b        ;   a move: lead
67ce8b: call 0x616e80        ; order_type again
67ce92: call 0x46f000        ; is_air(order)
67ce99: je   0x67cf1a        ;   neither: no lead
67ceb4: mov  0xf4(%ecx),%eax ; guys.list
67ceba: mov  0x50(%ecx),%edi ; UnitData +0x50, `angle`
67cec1: mov  0x84(%eax),%esi ; guy 0's avg_speed
67cec9: call 0x92d0c0        ; cosx(angle, avg)
67ced5: call 0x92d100        ; sinx(angle, avg)
```

- `UnitData::order_type@00616e80` is the head order's `get_type`, `NONE`
  for an empty list.
- `is_move@0046f050` is `1, 2, 3, 4, 0x12, 0x13, 0x15`: this crate's
  `index::is_move_family`. `is_air@0046f000` is `0x10, 0x11, 0x18`
  (`STRAFE`, `AIR_PATROL`, `AIR_ATTACK_GROUND`): `index::is_air_family`,
  new here.
- `+0x50` is `angle` in the type record (`UnitData`, "size 0x158"), which
  is this crate's `Movement::heading` (`lib.rs`), not the figure's
  `facing` (`GuyData +0x18`). §47.4 had named `+0x50` and the code had
  read `facing`; the two agree on a unit walking straight, and a turning
  one is where they part.

~~§47.7: "The lead's gate … this crate still asks `movement.dest.is_some()`".~~
Built here.

### 74.4 What this crate built

- **`Sim::fire_ammo_aim`**'s lead asks `order_type` of `is_move_family`
  or `is_air_family`, and leads along `heading`.
- **Unit test**: `the_lead_asks_the_target_s_order_and_leads_along_its_angle`.
  A `FLEE_TO` head with no body destination and a figure facing another
  way is led by `sinx`/`cosx(heading, avg) × t`; a destination with no move
  at the head is not led. `the_lead_is_the_first_figure_s_average_speed`
  now gives its target a move order and a heading.

### 74.5 The killers

Each was run as a mutation on `a793d9ff` (after the `ccc update` onto
1077's booking), with `git diff --stat` non-empty, restored from git and
`touch`ed after. The pins a re-pin touches were green first.

| mutation | unit test | run347's walk | run373 | the rest |
|---|---|---|---|---|
| the gate back to `movement.dest.is_some()` | fails | falls to 5042 | fails on `0/1`'s rows | run356 holds |
| the angle back to `facing` | fails | holds 5066 | holds | the whole rondata suite holds (576 of 578; the two red are the commander's line and the run's own thread count) |

### 74.6 What moved

- **Great Lakes 5042 → 5066.**
  - Led, `1/11`'s round comes down on (6497, 28561) and strikes `0/1` at
    (6260, 28466) on 5040: 237 and 95 off, 255, halved 127.
  - `0/1` reads 5/5 on block 5041, 8/10 on 5045 and 12/4 on 5077
    (`damage_frame` 5076) on both sides, where ours read 2/0, 5/5 and
    5/5 (5038).
  - Frame 5042's draws went 8 against 6 → agreeing.
  - run373's keys parted go 471 → 455.
  - The other two instances (4962, 5064) are led now too, and `0/1`'s
    and `0/2`'s rows hold through the window's end.
- **The new word** (no mechanism is named): frame 5066, ours 9 draws and
  the original 10, parting at index 0. Ours spends `Guy::set_anim+0xf2f <
  Guy::move+0x166`, the original `Guy::set_anim+0x97a < Guy::move+0x19f`.
  - It sits on block 5067, inside run373's window (30 before its last).
  - The figure is `1/13`'s. On block 5066 the original's
    stands under a fresh `ATTACK` (two orders) and ours walks on under
    its move (three).
  - `1/13`'s move has parted since block 5012: ours heads for (4104,
    31992), the original for (4872, 31464), path lengths 19 against 9.
    Every key first parting on 5066 and 5067 is `1/13`'s.

### 74.7 What is not established

- **A building shooter's round is not led** in this crate
  (`process_building_combat`). Whether `Ammo::init`'s block reaches the
  lead for one (its `local_38`/`local_30` guard) is not read here, and no
  capture on disk has a building shooting a moving unit near a word.
- **An air-order target** (`is_air`) is led now and no capture on disk
  shoots at one.
- **The heading against the facing** is unit-backed only: its killer
  (74.5) moves no capture's pin. The listing and the type record name
  `+0x50`; no round on disk lands differently for it.

### 74.8 Coverage

- **Diff-backed**: the gate, by run373's `0/1` (5041, 5045, 5077) and
  frame 5042's draws on run347, and the gate's killer failing both; the
  three `FLEE_TO` instances on disk (4962, 5024, 5064).
- **Listing-backed**: `67ce62`–`67ceda`; `is_move@0046f050`,
  `is_air@0046f000`, `order_type@00616e80` (decompile).
- **Unit-backed only**: the angle (`heading`, not `facing`), and a
  destination with no move at the head going unled.
- **Not backed**: an air-order target, and a building shooter's lead.

## 76. A dead target is re-aimed at on the review (item 1086, 2026-09-28)

Great Lakes' second word, frame 5066 of run347 (DECISIONS 53, 54): ours 9
draws against the original's 10, parting at index 0. Ours spent
`Guy::set_anim+0xf2f < Guy::move+0x166`, the original `Guy::set_anim+0x97a
< Guy::move+0x19f`. The figure was `1/13`, whose move had parted since
block 5012. The booking named no mechanism and no direction.

### 76.1 Every instance on the disk first

- **5012 is the move's first parting, not a window's edge** (parked 1083
  b). `1/13` agrees whole on every block from 4841 to 5011, both sides;
  every one of its rows that parts first parts on 5012.
- **Who and what.** `1/13` (guy type 82, group 65, a captain: `up −1`,
  `down 25`) holds `MOVE, ATTACK, ATTACK_TO` from block 4999: a chase on
  the citizen `0/2` (`ox 2 whom 0 uid 9`). `0/2` dies on frame 5000 on
  both sides (§72.5) and is gone from the dump's units from block 5001.
  The `ATTACK` on it stands on both sides to 5066.
- **The one-sided writes are all the original's, and all on `1/13`'s
  review phase**, `(frame + 13) % 16 == 0`. Block by block, the original's
  move (`x`, `y`, the goal leg and the next):

  | frame | the original's move | ours (before) |
  |---|---|---|
  | 5010 | (4824, 29928) via the detour's (4104, 31992), tolerance 0, 19 legs, `coll` (3969, 32147) | same |
  | **5011** | **(4872, 29928)** via (4872, 31464), tolerance 384, 9 legs, `coll` 0 | unchanged |
  | 5027 | (4920, 29928) | unchanged |
  | 5043 | (4968, 29976) | unchanged |
  | 5059 | (4776, 29880) | unchanged |

  No other unit's move changes on these blocks. The goal moves with the
  chaser, not with the target: `find_attack_pos` runs from where `1/13`
  stands, at a corpse that does not move.
- **None of the four frames spends a unit draw** in the original's trace
  (5011's are six farm draws and three `Surf`), so neither `fight`'s
  captain arm (`+0x9b0`) nor a ring walk wrote them.
- **The word follows.** On 5065 and 5066 the original stands at (4764,
  31013), 1,133 from its goal (4776, 29880): `do_move`'s dead-target arm
  (`vector_dist ≤ 0x480` from the walk's own goal, `docs/ORDERS.md` §20)
  pops the walk, and the stop is the word's draw. Ours, walking to its
  stale (4824, 29928), was 1,335 out.

### 76.2 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| `check_target_path`'s fleeing arm (§67) on the corpse | `Unit::close@0060ee50` ends in `close_orders@005e37f0`, which kills every order whose `get_type` is non-zero, so the corpse's `is_moving@00610af0` answers 0 and the flank triple cannot hold | killed |
| the review's range arm | `is_in_range@006486b0` returns 0 for a slot whose `flags & 1` is down, so the review returns 0 there | killed |
| the suspended search (`+0x104`, `do_move`'s head) | `start_dist` is 0 on every block of `1/13` (`docs/PATHFINDER.md` §18.6) | killed |
| `do_move`'s captain retarget, `fight`'s chase | each rolls a draw, and the four frames spend none of `1/13`'s | killed |
| `check_target_path`'s `+0x8` jump | below | **held** |

### 76.3 The listing

`check_target_path@005e22d0`, for an `ATTACK` (`local_14 == 10`) on a unit
slot, after the `is_seen` test (vslot `+0x48`, `UnitData::is_seen@00607a60`,
which asks the slot's position and asks nothing of its liveness):

```
5e2418: mov  eax, [0xc0618c]        ; GameAccess::objects
5e2420: mov  eax, [edi+eax+0x14]    ; objects[whom]
5e2424: mov  ecx, [eax+4*ecx]       ; [ox]
5e2429: call [eax+0x8]              ; vslot +0x8
5e242e: je   0x5e24e8               ; zero: past the flank triple
5e2434: ...                         ; the flank triple (§36.3, §67.2)
5e24e8: ...                         ; the fall-through: is_on_map, then §67.2's steps
```

- **Vslot `+0x8` is read off the executable's own vtables**, since the
  export's names are COMDAT-folded: `Unit::vftable@00b417d0 + 8` holds
  `0x46cda0`, `movzx eax, byte [ecx+8]; and eax, 1; ret` —
  `SubObjectData::is_active`, the slot's allocation flag. `Build::vftable`
  and `Wall::vftable` hold `0x41bff0`, `xor eax, eax; ret`. So a living
  unit takes the flank triple, and a building or **a dead unit** jumps
  past it.
- **The corpse is still in its slot.** `Object::close@00647160` clears
  `flags` (`+0x8`) and sets `hold_frames` to 30, and `Objects::remove
  @00658980` only lowers the player's slot mark. Neither touches the
  pointer in `objects[whom][ox]`, the position or `inside_up`. So the
  fall-through's `is_on_map@0046ce30` (`inside_up >> 15`) answers 1 for a
  unit that died on the map, and `is_in_range` from the walk spot answers
  0 (the slot is not active). The ranged chaser then takes §67.2's steps
  4 and 6: `repath`, `find_attack_pos` from its own position, and
  `add_move_order(…, QUEUE_FIRST)` at the corpse.
- **`find_attack_pos` finds a spot only on its far arm.** Its near arm
  takes a sweep's spot only when the asker would be in range from it
  (§32), and nobody is in range of a corpse; then nothing is found, the
  legs go (§67.2 step 4) and `do_attack` takes over. `1/13` was about
  fifteen tiles out, past `(max_range + 8)`, on the far arm.

### 76.4 What this crate built

- **`Sim::check_target_path`** (`orders.rs`) asks `alive()` after
  `target_is_seen`, and a dead unit target goes to
  **`rechase_fleeing`**, which is §67's fall-through unchanged. A living
  target that is not active (inside something) still returns, as before.
  It used to return for every target that was not `active`.
- **Unit test**: `a_dead_target_is_re_aimed_at_on_the_review_s_phase`
  (`fight.rs`). A dead, standing target fifteen-plus tiles out is
  re-aimed at on the review's phase, and the new spot is nearer the
  corpse than the stale one. The same target alive and standing, and the
  dead one off the phase, keep the stale spot.

### 76.5 The killers

Run on `ed732aec` with `git diff --stat` non-empty, restored from git and
`touch`ed after. The pins the re-pin touches were green first.

| mutation | unit test | run347's walk | run373 |
|---|---|---|---|
| a dead target returns, as before | fails | falls to 5066 | fails on `1/13`'s `pos` (5012) |

### 76.6 What moved

- **Great Lakes 5066 → 5075.**
  - On block 5012 `1/13`'s move reads (4872, 29928) via (4872, 31464),
    tolerance 384 and nine legs on both sides, where ours kept (4824,
    29928) via (4104, 31992), tolerance 0 and nineteen. (4920, 29928) on
    5028, (4968, 29976) on 5044 and (4776, 29880) on 5060 on both.
  - On block 5066 both stand at (4764, 31013) under the `ATTACK`, and on
    5067 both have taken `0/4`. Ours had walked on to (4702, 31263).
  - Frame 5066's draws went 9 against 10 → agreeing.
  - run373's keys parted go 455 → 461: `1/13`'s rows from 5012 go, and
    the chase on `0/4` (from 5068) and the new word's rows arrive.
- **East Indies holds at 5606. Great Sahara holds at 8.**
- **The new word** (no mechanism is named): frame 5075, ours 11 draws
  and the original 10, parting at index 0.
  - Ours spends `1/20`'s `Guy::set_anim+0x97a < Unit::move_step+0x823`
    first. The original's first is `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`, which is ours' second.
  - It sits on block 5076, inside run373's window (21 before its last).
  - On block 5076 `1/20` reads `collide 1` on `1/18` on ours alone.
    `1/10` and `1/11` have ended their chase on ours alone (`ATTACK`
    heads, two orders, against three). Every key first parting on 5075
    and 5076 is one of those four's, or `1/0`'s figure's.
  - This is where parked 1074 said its first `collide` row now stands
    (`1/20`, 5076): `Objects::find_collision`'s conjunct of `do_move`'s
    kill, which §35.3 and §36.4 carry as a SEAM.
  - `1/13` itself first parts on 5068. Its chase on `0/4` is aimed at
    (2136, 31608), tolerance 384, ten legs, against ours (3384, 31464),
    tolerance 0, eight.

### 76.7 What is not established

- **A reused slot.** The original reads `objects[whom][ox]` with no `uid`
  test, so once `hold_frames` has run out and a new unit takes the
  corpse's `o`, the review asks the newcomer. This crate's target is the
  corpse's own entry, and it never recycles one. No capture on disk
  reaches it.
- **A dead target inside something** (`inside_up` set) would `repath`
  at the fall-through's first step. It is built as `rechase_fleeing`'s
  `on_map` test, with no capture.
- **The near arm's refusal** (a dead target within `max_range + 8` tiles)
  ends the chase with its legs gone. It is built through
  `find_attack_pos`, and it is unit-backed only.

### 76.8 Coverage

- **Diff-backed**: the re-aim, by run373's `1/13` on 5012, 5028, 5044
  and 5060, and frame 5066's draws on run347. It is a floor, and the
  killer made it fail.
- **Listing-backed**: `5e2418`–`5e242e`, and the three vtables' slot
  `+0x8` read from the PE; `Object::close`, `Objects::remove`,
  `close_orders`, `is_in_range@006486b0` (decompile).
- **Unit-backed only**: the living-and-standing and off-phase refusals.

## 77. A living target out of sight keeps the chase (item 1074, 2026-09-28)

Great Lakes' second word, frame 5075 of run347 (DECISIONS 53, 54): ours 11
draws against the original's 10, parting at index 0. Ours spent `1/20`'s
`Guy::set_anim+0x97a < Unit::move_step+0x823` first, the original
`Guy::set_anim+0x97a < Guy::inc_time+0x271`. The booking carried
`Objects::find_collision`'s conjunct of `do_move`'s kill (§71.2's third) as
the hypothesis, and named no direction (parked 1076).

### 77.1 Every instance on the disk first

- **The whole cast of block 5076** (run373), every key first parting on
  5075 and 5076, both sides: `1/10`, `1/11` and `1/18` hold `ATTACK,
  ATTACK_TO` (two orders) on ours against `MOVE, ATTACK, ATTACK_TO` (three)
  on the original's, and `1/20` reads `collide 1`, `collide_o 18` on ours
  alone. Nothing else parts there but `1/0`'s figure, which has parted
  since 5070.
- **All four chase the same unit**, the citizen `0/1` (`ox 1 whom 0 uid
  8`). So do `1/19` (below). `0/1` agrees whole on both sides: it walks
  its `FLEE_TO` head (`GATHER` under it) from (6251, 27650) to (6242,
  27627), heading −447479808 → −472055808, `visible 0`.
- **The direction is ours'** (parked 1076): ours popped three walks and
  the original kept them. `1/20`'s collide follows: it walks into the
  `1/18` ours stopped, and the stop is the word's first draw.
- **Not the review.** `(5075 + o) % 16` is 13, 14 and 5 for `1/10`,
  `1/11` and `1/18`.

### 77.2 The readings, and what killed each

| reading | killer | verdict |
|---|---|---|
| `find_collision`'s conjunct of `do_move`'s range kill (§71.2's third, the booking's) | `1/10` and `1/11` stand four and three unit cells from their nearest unit (`1/9`), past two `coll_size`s; the conjunct cannot hold for them. And none of the three is in reach of `0/1`, 1,458 to 2,500 away | killed |
| the flank clause (§71) | `e` is `0x4ee30000`, `0x697d0000` and `0x65540000` against `0/1`'s heading, `flanking` 2, 1 and 1, and `0/1`'s head is a `FLEE_TO`: the triple holds on both sides | killed |
| **`do_move`'s dead-target arm** | below | **held** |

A probe of ours' `kill_current_order` on frame 5075 named the caller for
all three: `repath`, from `do_move`, through the dead-target arm (ORDERS
§20.3). Ours' `valid_target(me, 0/1)` refused because
`target_is_seen` answered 0 for player 1. `0/1` was active, on the map and
at war with it. `1/19` and `1/20` reached the same arm on 5075, but each was
past `0x480` from its walk's goal and walked on.

### 77.3 The listing

`Unit::do_move@005f7b30`, the action block, after `update_action` (a
cursor walk that asks nothing of the target) and the order's `+0x50`
target record:

```
5f7ece: test edx, edx ; js 5f8221     ; ox < 0: the dead arm
5f7ed6: test eax, eax ; js 5f8221     ; whom < 0
5f7eed: mov  eax, [0xc0618c]          ; GameAccess::objects
5f7efc: mov  ecx, [edx+eax]           ; objects[whom][ox]
5f7eff: testb $0x1, 0x8(%ecx)         ; flags & 1
5f7f03: je   5f8221
5f7f09: movw 0x30(%ecx), %ax          ; the slot's uid
5f7f0d: cmpw 0x10(%esi), %ax          ; the order's
5f7f11: jne  5f8221
5f7f17: ...                           ; max_range, the ranged block (§35.3, §65, §71)
```

That is the whole gate. There is no diplomacy test and no `is_seen`
anywhere before `5f8221`, and `update_action@0060a870` (the decompile)
only walks the order list's cursor. So a living target the chaser's
player cannot see keeps its chase in the ranged block, and when nothing
there fires, the planner walks on to the order's goal. ORDERS §20.3
describes `5f8221` correctly. This crate reached it through `valid_target`
instead.

### 77.4 What this crate built

- **`Sim::do_move`** (`orders.rs`) enters the action block on
  `obj_alive(t)`, not `valid_target(me, t)`. This crate never recycles a
  slot, so `alive` is `flags & 1` and the `uid` together.
- **Unit test**: `a_living_target_out_of_sight_keeps_the_chase`
  (`orders.rs`, `chase_tests`). A ranged chaser under a fog grid that shows
  nobody anything walks its chase off the review's phase, within `0x480`
  of its goal, on a target out of reach. The living target keeps the walk
  and the dead one loses it.

### 77.5 The killer

Run on the committed tree with `git diff --stat` non-empty, restored from
git and `touch`ed after. The pins the re-pin touches were green first.

| mutation | unit test | run347's walk | run373 | run396 |
|---|---|---|---|---|
| the gate back on `valid_target`, on `1965ee79` | fails | falls to 5075, 11 against 10 at index 0, the old word's sites | fails on `1/19`'s facing (5085 again) | fails on `1/21`'s `order:kind` (the word's row) |

### 77.6 What moved

- **Great Lakes 5075 → 5105.**
  - On block 5076 `1/10` stands at (5043, 29856), `1/11` at (4898,
    29588) and `1/18` at (6084, 29076), each under `MOVE, ATTACK,
    ATTACK_TO` on `0/1`, on both sides. Ours read (5028, 29880), (4901,
    29616) and (6084, 29077) under two orders.
  - `1/20` stands at (5965, 29093) with `collide 0` on both, where ours
    read (5928, 29112) and `collide 1` on `1/18`.
  - Frame 5075's draws went 11 against 10 → agreeing.
  - run373's keys parted go 461 → 203. Nothing parts on 5073..5089 but
    `1/0`'s figure; `1/19`'s facing no longer parts in the window.
- **East Indies holds at 5606. Great Sahara holds at 8.**
- **The new word** (no mechanism is named): frame 5105, 11 draws on each
  side, parting at index 3.
  - The original spends `Unit::fight+0x9b0 < Unit::do_attack+0x6ba`, and
    ours `Farms::inc_time+0x1ae`. On 5106 ours spends 10 against 11.
  - It sits on block 5106, past run373's window, so **run396** widens it
    (blocks 5100..5356, `docs/RUNS.md`).
  - Its figure is `1/21`. On block 5105 the original's has ended its
    chase (`ATTACK` head, two orders) and ours walks on (`MOVE`, three).
    On 5106 the original's strikes `0/5` (`recharging` 32, `hold_attack`
    1). Every key first parting on 5105 is `1/21`'s.
  - `1/21`'s chase goal has parted since run373's block 5091: ours aims
    at (3528, 31512) with `order:flags` 16, the original at (3336, 31368)
    with 0. Its second `ATTACK` reads `new_ord` 0 against 1 on run396's
    first block, which is parked 1073's shape. How the chase is aimed is
    parked 1089's family (`1/13`'s on `0/4`, from 5068).

### 77.7 What is not established

- **`find_collision`'s conjunct** (§71.2's third) is still a `SEAM`. This
  frame was not it: the kill was never reached.
- **A target inside something.** Its `flags & 1` stays up, so the
  original enters the ranged block with it. This crate's `obj_alive` does
  too, but what `is_in_range` answers there is not read, and no capture
  holds one.
- **A target no longer at war.** It also enters the block now. No capture
  has a ceasefire in a chase.
- **The original's `is_seen` of `0/1`** is not read on either side; the
  dump prints `visible 0`, and the listing makes it moot here.

### 77.8 Coverage

- **Diff-backed**: the three walks kept on 5075, by run373's blocks 5076
  onwards and frame 5075's draws on run347. It is a floor, and the killer
  made it fail.
- **Listing-backed**: `5f7ece`–`5f7f11` (llvm-objdump), and
  `update_action@0060a870` (decompile).
- **Unit-backed only**: the dead target's arm under fog.

## 79. A follower whose target has died takes its captain's (item 1089, 2026-09-28)

Great Lakes' second word, frame 5105 of run347 (DECISIONS 53, 54): 11 draws
on each side, parting at index 3. The original spent `Unit::fight+0x9b0 <
Unit::do_attack+0x6ba`, ours `Farms::inc_time+0x1ae`. The figure was
`1/21`, whose chase goal had parted since run373's block 5091. The booking
carried two parked readings as hypotheses, 1089's `find_attack_pos`
tolerance and 1073's fresh order, and named no direction (parked 1076).
**Neither was the mechanism.** The frame was sixty-six ticks back, on
another unit.

### 79.1 Walking `1/21` back to its first parted field

- **`1/21`'s own first parted field is its attack row on 4996**:
  `ever_in_range` 1 against 0 and `new_ord` 0 against 1, which is §71.6's
  fresh-order shape (parked 1073). No draw parts on it, and the chase goal
  agrees on 5071, twenty frames after it. So 5091 is the first parting
  that reaches the draw stream, and 4996 is not its cause.
- **The re-aim on 5090** (the chase's twenty-frame repath, which pops the
  move on both sides on 5070 and 5090). `0/5` stands at (3488, 31368),
  and `1/21` at (3000, 31512). It is melee: `stand` `0x30`, one ring at
  144. `0/5` is moving, but the flank arm (§32.4) is shut. `1/21`'s
  `unit_masks` is 331790 (`0x5100E`), and `602822` jumps on `0x40000` to
  the plain sweep.
- **The sweep, candidate by candidate** (a scratch probe, not kept): the
  original's goal (3336, 31368) is ours' `+1` candidate. Ours refused it by
  `find_ordered_collision` alone: `find_collision` was clear. The refusing
  order was **`1/13`'s `orders_pos` (3384, 31464)**, a member of the same
  group 65. The original's `1/13` was aimed at (2136, 31608). That is
  parked 1089's own row.
- **`1/21`'s `order:flags` 16 against 0 follows from the goal.** Ours'
  `chase_reentry` returns early when the goal shares a coordinate with the
  chaser (y 31512 on both), so the latch was never toggled.

### 79.2 `1/13`, and the frame that parted

- **The first-parting map hid it.** On block 5067 ours' `1/13` holds
  `ATTACK 0/5` and the original's `ATTACK 0/4`. No row printed it, because
  `1/13`'s `order:target` key had first parted on 4841 (its `ATTACK_TO`)
  and the map keeps a key's first parting only. The `RON_STACKS` probe
  shows it.
- **Tick 5066.** `1/13`'s target, the citizen `0/2`, died on 5000.
  `fight`'s invalid-target arm (`LAB_005fdb9e`, §43.2) calls
  `find_new_target(this, NULL, 0)`. Both traces spend no unit draw on 5066
  beyond the figures' idles.
- **Ours searched.** Its ranking was `0/5` 888 (attack distance 1738,
  `targeted` 2, value 10664) against `0/4` 603 (2842, 0, 10261). For the
  original to rank `0/4` first, `0/5`'s `targeted` would have to be 24 or
  more.
- **run400's packet** (logger frame 5066, `docs/RUNS.md`):
  - `targeted` reads 2 on `0/5` and 0 on `0/4`, the same as ours. That
    reading is killed.
  - `find_melee_target(−1, &whom, 0, 1, 0)`, entered directly on `1/13`,
    **never calls `find_nearby_target`**. It adds `ATTACK 0/4` from
    `find_melee_target+0x1c5` and answers `o 4 whom 0`.
- **`1/13` is a follower.** On block 5066 its `o_up` is 12. `1/12`, its
  captain, holds `ATTACK 0/4`.

### 79.3 The listing's head

`Unit::find_melee_target@005ff9c0`, lines 32–91 of the decompile, for a
caller whose third argument is 0:

1. `is_captain` (vslot `+0xe8`, `o_up >> 15`). A captain goes to the
   search.
2. A follower reads its captain's action (vslot `+0xe4`, `get_action`). No
   action, or one that is not `ATTACK` (`+0x10 == 10`), returns −1: no
   search, no order.
3. With the captain's target `(o, whom)`, it takes the target when
   `valid_target` passes and either:
   - the stance is not `2` and neither of the two entrenched pairs
     (`unit_masks & 0x2000000` without `unit_masks2 & 0x20000`, on it or
     on the captain) holds; or
   - the target is in range.

   With `param_4`, it adds `add_attack_order(o, whom, pos, mandatory, 0)`,
   where `pos` is `QUEUE_FIRST` under an `ATTACK_TO`, a `GROUP_ATTACK_TO`
   or a `GUARD` in front and `QUEUE_NEW` otherwise, and `mandatory` is the
   captain's action's `+0x1c`. It returns without searching.
4. Otherwise `on_duty`, and the search.

`find_new_target@005ff6a0` kills the current order first and passes
`(−1, param_1, 0, 1, 0)`. So under `1/13`'s remaining `ATTACK_TO` the add
is `QUEUE_FIRST`, the position this arm always used.

This is not `fight`'s own mirror (`Unit::fight@005fd4d0`, before the
validity test, §43.2). That mirror hands a follower its captain's target
only in `AGGRESSIVE` or `DEFENSIVE`, or when the target is in range.
`1/13` is `RAID` and 3,000 out, so the mirror passed, and the head is
what caught it. §66.3's killed row is `Group::action_attack`'s retarget,
another caller.

### 79.4 What this crate built

- **`Sim::find_melee_target_added`** (`orders.rs`): the head
  (`melee_squad_head`, item 1012's, until now used by the attack-move's
  look alone), the add at the head's position with the captain's
  `mandatory`, and the search with its `QUEUE_FIRST` add otherwise.
  `fight`'s invalid-target arm calls it after `kill_current_order`.
- **Unit test**: `a_follower_whose_target_died_takes_its_captain_s`
  (`orders.rs`, `chase_tests`). A `RAID` follower whose target is dead
  takes its captain's target, 24 tiles off, where the search alone ranks
  a nearer foe first.

### 79.5 The killer

The arm was put back to the bare search on `4a89cba2`, with `git diff
--stat` non-empty, then restored from git and `touch`ed. The pins the
re-pin touches were green first.

| mutation | unit test | run347's walk | run373 | run396 |
|---|---|---|---|---|
| the search without the head | fails | falls to 5105 | fails on `1/13`'s move (5068) and `1/21`'s goal (5091) | fails on the old word's `order:kind` (5105) |

### 79.6 What moved

- **Great Lakes 5105 → 5161.**
  - On block 5067 `1/13` holds `ATTACK 0/4` on both sides, where ours held
    `0/5`.
  - On 5068 it stands at (4751, 31019), aimed at (2136, 31608) with
    tolerance 384, on both. Ours stood at (4749, 31041), aimed at (3384,
    31464) with tolerance 0.
  - On 5091 `1/21`'s goal is (3336, 31368) with `order:flags` 0 on both,
    where ours read (3528, 31512) and 16. On 5092 both stand at (3025,
    31501).
  - Frame 5105's draws went 11 against 11 at index 3 → agreeing.
  - run373's keys parted go 203 → 156, and run396's 1,253 → 737.
- **Chapter two's `near_o` closes** (§37.1): item 523's two bowmen on
  847, `0/7` and `0/8`, are followers whose target died. They no longer
  search, and run112 agrees on all 17,219 unit frames, against 17,113
  before.
- **East Indies holds at 5606. Great Sahara holds at 8.**
- **The new word** (no mechanism is named): frame 5161, ours 12 draws
  against the original's 13, parting at index 3. The original spends a
  second `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, and ours
  `Farms::inc_time+0x1ae`.
  - It sits on run396's block 5162, 62 after the window's first block and
    194 before its last.
  - `0/5` died on 5147 on both sides. On block 5161 `1/24` has taken the
    scout `0/0` fresh on both sides, and its follower `1/26` still names
    `0/5`.
  - On 5162 the original's `1/24` strikes `0/0` (`in_range` 1,
    `recharging` 33) and `1/26` holds `0/0`. Ours' `1/24` has re-searched
    onto the city `0/2000`, and `1/26` has followed it.
  - Every key first parting on 5162 is one of those two units'.

### 79.7 What is not established

- **The head at `find_melee_target`'s other callers with a third argument
  of 0** is still ours' bare search:
  - `Sim::find_new_target` (the collision ladder's, `stand` 1);
  - the packer's re-search;
  - the guard arm's search;
  - `think`'s idle search.

  Each is a SEAM. A captain, which the building arm's re-search requires,
  searches either way. No capture on disk has a follower reach one of them
  with a dead or refused target.
- **The fresh order** (parked 1073) still stands on `1/21`'s attack row
  from 4996 and on 5105. No draw parts on it.
- **§76.1 calls `1/13` a captain** (`up −1`, on 4999). On 5066 its `o_up`
  is 12. The chain in between was not walked here.

### 79.8 Coverage

- **Diff-backed**: the head, by run373's `1/13` on 5067–5068 and `1/21`
  on 5091–5092, run396's 5105, and frame 5105's draws on run347. It is a
  floor, and the killer made it fail.
- **Packet-backed**: `targeted` on `0/4` and `0/5`, and `find_melee_target`
  entered on `1/13` (run400).
- **Listing-backed**: `005ff9c0`'s head (decompile, lines 32–91);
  `find_new_target@005ff6a0`'s call. `find_attack_pos`'s flank gate
  `602822`–`6028ed` and `flanking@0092cfe0` were read from the listing and
  build nothing here.
- **Unit-backed only**: the `QUEUE_NEW` position and the captain's
  `mandatory`, neither of which `1/13` exercises.

## 80. The one-in-five re-search kills the attack first (item 1099, 2026-09-28)

Great Lakes' second word, frame 5161 of run347 (DECISIONS 53, 54): ours 12
draws against the original's 13, parting at index 3. The original spent a
second `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, ours
`Farms::inc_time+0x1ae`. On run396's block 5162 `1/24` and `1/26` held the
city in ours and the scout in the original. No mechanism was named, nor a
direction (parked 1076). **The word is now the game's end**, and a second
parting there, the city's capture, was built with it (`docs/CITIES.md`
§7.2).

### 80.1 Every instance on the disk first

- **`1/24` is a captain** (`o_up` −1, `o_down` 25), stationary at (3288,
  31800), stance 0, under `ATTACK` over `ATTACK_TO`. `1/26` is its
  follower. Their target `0/5` died on 5147 on both sides.
- **Block 5161 agrees**: `1/24` has taken the scout `0/0` fresh (`new_ord`
  1, `in_range` 0) on both sides, 48 from it by `attack_dist`.
- **Block 5162**: the original's `1/24` strikes `0/0` (`in_range` 1,
  `recharging` 33, `hold_attack` 1) and `1/26` holds `0/0`. Ours' `1/24`
  names the city `0/2000`; on 5163 it names `0/0` again, on 5164 the city.
  It flips every frame.
- **Every other unit that took the city in the window took it on both
  sides** (`1/23` on 5149, `1/22` 5155, `1/9` 5158, `1/10` 5159): their
  stacks agree (`RON_STACKS`). Only `1/24` and `1/26` part on 5162.
- **The draws**: both sides spend `1/24`'s `Unit::fight+0x9b0` on 5161,
  the one-in-five roll, and the stream agrees up to it. The original's
  extra idle roll is `1/24`'s strike.

### 80.2 The call chain, and the value no dump prints

A scratch probe on ours (not kept) printed the writer of `1/24`'s target:
on 5161 `retarget_attack` from `work`, the one-in-five arm, which pointed
the attack at the city; on 5162 `find_new_target` from the building arm
(§62), which killed the attack on the city and, under the attack-move,
re-found the scout. Hence the flip every frame. The search's candidates,
as ours scored them:

| search | flags | scout `0/0` | city `0/2000` |
|---|---|---|---|
| 5160, after the kill of `ATTACK 0/5` | `0x20010` | 344 | 251 (halved) |
| 5161, the one-in-five, under `ATTACK 0/0` | 0 | 344 | 669 |

The flags word is §64.1's, read off the head order: an `ATTACK` at the
head gives 0, the attack-move gives `0x20010`, which halves what is not a
unit. So the question was only where the head stands when the original
searches, and the listing answers it; no packet was needed.

### 80.3 The listing

`Unit::fight@005fd4d0`, lines 389–413 of the decompile, the captain arm
`LAB_005fddf7` (§62.2):

```text
local_14 = 0
if target.is_unit():
    local_14 = the type test (combat role, §8.2 step 0)
    roll = Random::get(0, 0xffff)                    ; fight+0x9b0
    if roll % 5 == 0 || order.flags & 0x10: local_14 = 0
if !target.is_unit() || local_14:
    t, who = find_new_target(this, &who, 0)          ; 5fdee2 (cavarch: find_melee_target)
```

**The unit arm and the building arm are the same call.**
`Unit::find_new_target@005ff6a0` is `repath`, `kill_current_order`, and
`find_melee_target(-1, &who, 0, 1, 0)`, which adds what it finds (§79.3).
So the attack is gone before the search, and the head it reads is the
attack-move's. This crate's unit arm called the search with the attack
still in front and re-pointed it in place (`retarget_attack`), which is
also why the original's head attack read fresh for a block after every
retarget (§71.6, parked 1073).

### 80.4 What this crate built

- **`Sim::do_attack`'s captain arm** (`orders.rs`) is one arm for both
  kinds of target. A building re-searches every frame; a unit after the
  draw and its suppressions. Either way `find_new_target(u, false)`:
  nothing found returns with the attack gone; another target keeps the
  search's own order and the frozen mark (§70.7); the same one goes on
  with the fresh order. `retarget_attack` is deleted.
- **Unit test**: `the_one_in_five_re_search_runs_under_the_attack_move`
  (`fight.rs`). Under the attack a search names an armed building; under
  the attack-move, the unarmed unit the captain is attacking; after the
  re-search the target is the unit.

### 80.5 The killer

The arm was put back to searching under the attack and re-pointing in
place for a unit target, on `25468c1d` with `git diff --stat` non-empty,
then restored from git and `touch`ed. The pins it touches were green
first, but for the commander's two queue lines.

| mutation | unit test | run347's walk | widenings |
|---|---|---|---|
| the re-search under the attack, in place | fails | falls to 5161; the endpoint reads 20 off | run356 (`1/24`'s spot, 4689), run373 (the attack rows, 4841 and 4853), run396 (5105's row) and run403 (601 keys on its first block against 112) fail |

**Diff-backed.**

### 80.6 What moved

- **Great Lakes 5161 → 5930, the game's end.** On block 5162 `1/24`
  strikes `0/0` and `1/26` holds it on both sides; frame 5161's draws went
  12 against 13 → agreeing, and no frame of run347's trace parts.
- **The fresh order's rows close** (parked 1073): run356's `1/24` on 4753
  (`ever_in_range`, `new_ord`) and its spot on 4689, (3763, 31600) against
  the cell centre (3768, 31608), which `fight`'s snap under the fresh order
  now matches; run373's `1/9` on 4841 and `1/24` on 4853; run396's `1/21`
  on 5105. run373's keys parted go 156 → 128, run396's 737 → 131.
- **The game's end** needed the capture tally's filter too (`docs/CITIES.md`
  §7.2): on 5930 the AI takes the human's city, and the human is defeated.
  run347's closing whole-map state, block 5931, now reads 42 units
  compared, 0 off, 0 unlinked, 0 extra, every building linked with 0
  diverged, and three cities unlinked because the closing `CITY` record
  prints no `o`.
- **East Indies holds at 5606. Great Sahara holds at 8.** Every golden
  chapter and the first pair hold (the gate).

### 80.7 What is not established

- **Value rows beneath the draws.** run396 and run403 carry standing rows
  the draw stream does not see, each standing on run396's first block
  (5100) or born in the window: the human's leader record (its `SITE`
  regions, `num_units[0]` counting five dead citizens — parked 1092 — its
  income and war flags), the human's city record, the group record (parked
  1075), the attack-move's `order:target` and the army's `form`, and
  `1/36`'s order from its birth on 5165 (`action`, `flags`, the move's
  `angle`). The AI scout `1/0`'s second figure parts on run396's 5240,
  (3081, 20587) against (3065, 20580), and agrees again by run403's 5925.
  No mechanism is named for any of them.
- **The cavalry archer's arm** (`find_melee_target` in place of
  `find_new_target`) and **the `poor_target` arm** (`005fded5`, the same
  search for a unit target the roll suppressed) are SEAMs, as is the early
  return when the search names `fight`'s own entry arguments.

### 80.8 Coverage

- **Diff-backed**: the arm, by run347's frame 5161 and every frame to
  5930, run356, run373, run396 and run403; the endpoint at 5931.
- **Listing-backed**: `005fddf7`–`005fdeea`, `005ff6a0`'s kill.
- **Unit-backed only**: nothing beyond what the captures carry.

## 81. The air line under fire: the jam roll, the flak roll, and a plane shot down (item 1102, 2026-09-28)

Golden chapter thirty-eight (`docs/GOLDEN.md` §47, run404, run405) flies
two Bombers at a Barracks past a Radar Air Defense, an Anti-Aircraft
Battery and an Infantry squad. Three mechanisms spend draws there that
this crate did not; each is read off the listing, run under the emulator
where it could be (`tools/emu/flak_arm.py`), and checked on run404.

### 81.1 The jam roll

`Unit::fight@005fd4d0`'s animation choice, past the ship's rock
(`unit_flags & 0x2000000`), the Patrol Boat's and the Immortals' arms
(`5fed6c`..`5fee2d`), asks `has_objmask(0x80000000)` of the attacker's
type (`5fee5a`, the default slot `0046cf10` forwarding to the type's
`0xf0`). An `ANTI_AIR` type draws `GameAccess::rnd(100)` (`5fee89`) and
plays animation 0 when it is under `jam_unit_radar_prob` (`Constants
+0x21c`, rules.xml's `50%`) **and** `ObjectData::is_jammed@00653660`
answers ≥ 0; otherwise `CHAR_ATTACK1` as before. `is_jammed` is −1
unless the owner's leader carries `leader_flags & 0x40000` and an enemy
special stands over the unit, so the draw is spent and never jams here.
§8 step 4's "a unit with `DETECT` … `GameAccess::rnd`, the other RNG"
was this: the mask is `ANTI_AIR`, and `rnd` draws `game_random`.
run404's Battery spends it on 742, `Unit::fight+0x9b0`'s draw first.

### 81.2 The flak roll

`Ammo::init@0067bbf0`'s air arm (`67bef8`..`67c16a`) runs for a live
fixed-wing target (`+0x218` 2, not `unit_flags & 0x20`, not a missile),
entered from `67be7e` (a shooter whose vslot `0x18` answers 0, a
building) and `67bec4` (a unit not on `0x17`/`0x18`), with `esi` 100
from `67be77`. An `ANTI_AIR` shooter of the air domain takes no roll. Any
other `ANTI_AIR` shooter draws once, at `+0x432` (return `67c022`) when
`UnitData::is_flying_low@0060a140` answers 1 and at `+0x463` when not,
and misses — `flags |= 0x10`, the same bit as `do_damage="0"`, so the
round closes without `Ammo::do_damage` — unless `r % 100` is under its
**own** `FLY_LOW` or `FLY_HIGH`. A shooter that is not `ANTI_AIR` draws
against the **target's** figure first (`+0x49f`/`+0x50f`) and, past it,
its own (`+0x4dc`/`+0x548`). Under the emulator: a Battery (50/90) hits
a low Bomber on 89 and misses on 90, a high one on 49 and 50; an
Infantry (0/33) hits a low one on 9 then 32, misses on 9 then 33 or on
10 alone.

**`is_flying_low` is a distance** (`60a140`..`60a2ff`, run whole under
the emulator): a fixed-wing unit on the map, on a `STRAFE` with a live
target or an `AIR_ATTACK_GROUND`, within `vector_dist < 0x900` of that
target's point or that order's point; failing that, its `get_air_order`
with `returning` set and its live home within `0x900`. `is_flying_high`
is "on the map and not low". run404: 43 rounds, one draw each, 31 low
and 13 high; every one of the 38 matched to its record carries the flag
its roll gives.

### 81.3 A plane shot down

`Objects::kill_guy@00659410` sends a guy whose type's `CAT` (`+0x14`) is
Air (8) and that is not a missile (`659473` → `659613`) to a free `Ammo`
and `Ammo::init_crash@0067b800`, never `add_death`. `Unit::close`'s death
animation draw (`+0xcb6`) is still spent first. The crash draws once from
the game's stream (`+0x305`, `rolling = r % 7 − 3`) and twice from a
local `Random` seeded by the guy's point (`+0x397`, `+0x3af`, the
`bank_dx`/`bank_dy` ±30), and lays `num_guys` 1, `who`/`o` the plane's,
`whom`/`ox` −1, `flags & 0xe3 | 2`, `sz` its altitude, a landing drifted
along its heading by a float fall time. run404's `0/6` on 1006 and `0/7`
on 1019: `rolling` −1 and −2, from 58907 and 56687; the crashes come
down with no draw from the game's stream.

### 81.4 What this crate built

- `sim::air`: `is_fixed_wing`, `is_flying_low`, `is_flying_high`,
  `air_round_misses`, `radar_jams`, `crashes`; the sites
  `SITE_JAM_ROLL`, `SITE_FLAK_LOW`/`HIGH`, `SITE_AIR_TARGET_*`,
  `SITE_AIR_SHOOTER_*`, `SITE_CRASH_ROLL`, and their `SITES` rows.
- `fight.rs`, three granted spots: `Sim::swing_anim`'s last arm asks
  `radar_jams`; `Sim::fire_ammo_aim`'s head asks `air_round_misses`, whose
  miss is the round's `harmless`; `Sim::close_unit` lays no death object
  when `crashes`.

### 81.5 What is not established

- **The crash's round** is not laid: its landing (a float fall time and a
  `sin_table` drift), `rolling`, `check_hit` when it comes down. No draw
  from the game's stream follows it on run404.
- **`is_flying_high` in `valid_target`'s ladder**: this crate still reads
  every plane as high there (§61). run404's Infantry never weighed a low
  Bomber; run405 enters both functions on 730, from a search whose
  answer run404 does not print.
- **Five of run404's 43 flak draws print no new round** on the next
  block (817, 857, 867, 929, 932), and 963's high site stands at 2,279
  from T; the Bomber's front order on each is not read.
- **`is_jammed`'s arm**: no capture has a jammer.

### 81.6 Coverage

- **Diff-backed**: the jam roll (run404 742 → 776); the flak roll's first
  two rounds, 753 and 755, field for field with their flags; the crash's
  draw by its site on the stream (§47's table), and by the killer.
- **Listing-backed**: every gate above; `kill_guy`'s arm.
- **Emulator-backed**: `is_flying_low` whole; the air arm's thresholds.

## 83. An anti-air unit shoots without turning; an anti-air building winds up (item 1109, 2026-09-28)

Chapter thirty-eight's word 776 (`docs/GOLDEN.md` §47) was the Battery's
swing, not the Infantry's walk: `Guy::move+0x166` is the queued attack's
`set_anim(CHAR_ATTACK1, 0, 1)`.

### 83.1 The angle kept (built)

`Unit::fight@005fd4d0`, right after `Unit::set_attack` (`5fe7f4`), whose
answer is "the pivots can bear": when the **target's** type has `+0x218 == 2`
(the air domain, `5fe81f`) and the shooter `has_objmask(0x80000000)`
(`ANTI_AIR`, `5fe82a`..`5fe855`, a `cmovne`), the answer becomes 1 whatever
the pivots said. `5febb0` then keeps `this->angle`, and the unit fires
without turning. Its guys still turn to the unit's heading, and the swing
waits in `hold_attack` until they arrive (`Guy::move`'s `+0x9e` arm).

run404, both sides: the Battery `1/9` stands at (21384, 17256) with its
heading at 1222246400 from 766, its guy turning to it at about −190,887,424
a frame. It re-attacks `0/7` on 773. Theirs keeps 1222246400 and holds
(`hold_attack` 1 from 773), the guy arrives on 776, and 777 prints
`cur_anim` 12, `stopped` 1. Ours had failed the pivot test and re-headed to
the target (−819789824), so it never swung. **Built** as
`Sim::anti_air_keeps_angle` (`crate::air`), or'd after `set_attack` in
`Sim::fight`. Tests: `an_anti_air_unit_keeps_its_angle_at_an_aircraft_only`,
and `an_anti_air_strike_at_a_plane_does_not_turn_the_unit`, which exercises
the call site. **Diff-backed** on run404's 773..777.

### 83.2 The Radar Air Defense's cycle (read here; ~~unbuilt~~ built and read from the listing in §84)

`Build::do_attack@006228f0`: a building with `ANTI_AIR` that is neither
`LOOKOUT` nor `OBSERVATIONPOST` (`TypeIndex` 521 and 522; neither is
carried here yet) skips the recharge
countdown at its head, and returns **before** `Object::fire_ammo` when its
target is in range. So a Radar Air Defense never fires from `do_attack`.
Its cycle lives in `Wall::inc_time@0063fb60`'s anti-air arm, on the same
`recharging` short (`+0x7a`, the dump's `BUILDDATA recharging`). As read
from the decompile, on its main branch: from 0 it counts up while under
`get_game_frames(8) − 1`. At the top, with a target (`+0x7c ≥ 0`), it goes
to 0 and then −1, and counts down to `−get_game_frames(0xc)`, where it goes
back to −1. With no target at the top, it is set to `get_game_frames(8) − 1`.
While it is negative, the arm builds a guy-like packet with `cur_anim` 0xc
and `cur_time = −recharging`. The other branch (`+0x34 < 0` or `+0x60 < 0`)
and the early returns above it are not read, and the dump's 0 before
acquisition says one of them holds there.

run404 prints exactly that: `1/2007` takes `attack_ox` 7 on block 778
(trace frame 777), then `recharging` 1..19 to 796, then −1..−10 repeating.
It spends no draw on 777. Ours fires on acquisition through
[`Sim::process_building_combat`]'s footprint scatter: four draws in
`buildings` on 777, where theirs has none (ours 10, theirs 6).

**Not established**: ~~where the round leaves; the two game-frame counts;
and whether the flak roll runs on the building's round the same way~~ —
all three answered in §84: the release event of slot `0xc` through
`execute_game_events`, 20 and 10 from the same-named `<UNIT>`
(`GraphicPieces::verify_load`), and the flak roll spent as a unit's.

## 84. An anti-air building's round is its animation's: `Wall::inc_time`'s cycle (item 1112, 2026-09-28)

Chapter thirty-eight's word 777 (`docs/GOLDEN.md` §47): the Radar
Air Defense `1/2007` acquires Bomber `0/7`, and ours fired at once, four
draws against none. §83.2 read the cycle from the decompile; this
section reads it from the listing, reads the loader that gives it its
numbers, and builds it.

**Every instance on the disk first.** A scan of all 299 dumps under
`~/ron-golden`, `~/ron-data` and the game's `Logs` for a `BUILDDATA`
whose `orig_type` is 521..525 (the Lookout line) finds two captures:
run404/run405 (the Radar, 1,143 blocks) and chapter thirty-one's Lookout
(608 blocks), which only counts its construction and never acquires. So
run404's Radar is the one instance of the event on disk: `recharging`
0 through block 777; 1..19 on 778..796; then −1..−10 repeating to
1020; 18 on 1021 when the target is gone, and down to 0. Its rounds are
the three draws `Ammo::init+0x432`, `+0xcd9`, `+0xd0b` under
`GraphicEvents::execute_game_events+0x40d` on every trace frame ≡ 7 mod
10 from 797 to 1017. Twenty of the twenty-three print their `AMMO`
record on the next block with `cur_time 1`. The three that do not (817,
857, 867) are the five §47 found without a round, less two.

### 84.1 The arm, from the listing

`Build::do_attack@006228f0` (the listing, not the decompile's reading):
- **The head** (`6228fe`..`622946`): the `recharging` countdown runs
  unless the building `has_objmask(0x80000000)` (`ANTI_AIR`) and is
  neither `LOOKOUT` (`0x209`) nor `OBSERVATIONPOST` (`0x20a`).
  `TypeData::is`, the type vtable's slot `0x60`, is identity.
- **In range** (`622b52`..`622ba3`): the same test returns at `622c2e`,
  before `Object::fire_ammo` at `622bb8`. Out of range (`622b4c`), the
  target is cleared at `622c1c`; ~~this crate keeps it (§84.7)~~ and so
  does this crate's since item 1131 (§8.6).

`Wall::inc_time@0063fb60`'s anti-air arm, on `BuildData::recharging`
(`+0x7a`), past `is_active` (an unfinished building returns above it),
`ANTI_AIR` (`63ff8b`), `LOOKOUT` (`63ff99`) and `OBSERVATIONPOST`
(`63ffc5`), with `W = get_game_frames(8)` (`640020`):
- `near_o` (`ObjectData +0x34`) negative or the jam bit (`build_masks &
  0x8000`, `640025`, `640033`): a negative count goes to `W − 1`, then a
  count of at least 1 goes down one. So it counts down to 0 and holds.
- otherwise: from 0 it counts **up** while under `W − 1` (`6400c6`). At
  the top, with no target (`attack_ox < 0`, `6400fa`), it holds at `W −
  1`. With one, a non-negative count goes to 0. Then, with `S =
  get_game_frames(0xc)` (`640160`), a count at or under `−S` goes to −1,
  and any other goes down one (`6402ad`).
- while the count is negative: a package with `cur_anim 0xc`, `cur_time
  = −recharging`, `last_time` one under it, `ox`/`whom` the target's, and
  `angle 0x20000000` (`64039d`), handed to `execute_game_events`
  (`640414`) unless the jam bit is set (`640409`).

**Its order** is `Objects::inc_time@0065db70`'s: per leader, each unit's
`inc_time` and `execute_events`, then each building's `+0xa0`
(`Wall::inc_time` on a `Build`), and then the ammo list. So a round
leaves between its owner's units and the next leader's, and its first
`Ammo::inc_time` is the same frame's, hence `cur_time 1` on the block.

**Writers of `recharging`** (`+0x7a` on a `Build`, by offset): the
constructor, `Build::init` and `Build::activate` (0); `Wall::do_construct`
(+1 while building); `do_attack`'s countdown and reload; `Wall::inc_time`;
and three that other types own (the Kremlin's and the Terracotta Army's
arms of `Build::process`, `Object::do_launch`'s airbase,
`Build::do_missile_launch`). `Unit::*` and `Guy::move` write `UnitData
+0x7a`, a different field.

### 84.2 The packet: the building is drawn as a unit

`GraphicPieces::verify_load@00906550` loads a build piece of type
`AIRDEFENSE` (`0x20b`), `RADAR` (`0x20c`) or `SAM` (`0x20d`) that is
among the first `0x81` build pieces through **`init_unit_data`**, not
`init_build_data`, whose packet has only the nine `BuildAnimNames`
slots (`get_game_frames` answers 3 past them). run404's Radar piece is
50804, the same base as the Airbase's 50727 (`type − 0x19e` apart), so
it is in that first slice. Its `<UNIT>` is `RADARAIRDEFENSE-DEFAULT-AGE0`
in `unit_graphics.xml`: `CHAR_WALK` (slot 8) is *RadarDefGun Unpack*, 20
frames not looping; `CHAR_ATTACK2` (slot `0xc`) is *RadarDefGun Attack1*,
10 frames; and one `<RELEASEEVENT starttime="200" anim="CHAR_ATTACK2"
type="FlakShell" node="0"/>` gives frame 2 (§50.1). **All three numbers
are run404's**: 1..19, −1..−10, and the round on `recharging −2`. The Air
Defense Gun's `<UNIT>` has the same 20, 10 and one release at 200; the
SAM's has three releases (0, 333, 666 on nodes 0..2).

### 84.3 The round

Through the unit's release path: `execute_game_events+0x40d` →
`Objects::add_ammo` → `Ammo::init`, so the flak roll (§81.2) is spent,
then the scatter's two draws. The launch is the package's `x/y/z` plus
the node vector at the package's fixed `angle`. So it does not turn with
the target: all twenty printed rounds leave from (22408, 16376, 235), the
building at (22272, 16512, 8) plus (136, −136, 227). The round's
`num_guys` is 0, the building's.

### 84.4 A building's `ATTENUATE` is signed

`BuildType::init@00632340` stores `ATTENUATE` at `+0x1f0` as read, where
`UnitType::init@0061ab50` stores its absolute value. `Ammo::init`'s
accuracy is `to_hit + (−dist / 192) × +0x1f0`, clamped at 5. So a
building's rises with distance: the Radar's first round prints `accuracy
310` (300, and −5 × −2), and this crate's loader took `abs` for both and
printed 290. Every city, tower and fort carries −3 and every Lookout-line
building −1..−8. No widening had matched a building's round until this
item (§84.5), so the field was compared nowhere.

### 84.5 What this crate built

- `sim::air::WallCycle` on the building type's `TypeDef::wall_cycle`: set
  by `rondata::load::load_tables` for every `ANTI_AIR` building that is
  not `LOOKOUT` or `OBSERVATIONPOST`, and filled by
  `rondata::artdata::wall_packets` for `AIRDEFENSE`, `RADAR` and `SAM`.
  `sim::air::WALL_LAUNCH` carries the measured vector.
- `Sim::process_building_combat` skips the countdown and the fire for
  such a building.
- `Sim::walls_inc_time` (`crate::air`), called after each leader's units
  in `Sim::guys_inc_time` (granted), runs the cycle and fires through
  `fire_ammo_pub`.
- The loader keeps a building's `ATTENUATE` signed.
- The instrument: `golden::cycle_rows` compares `recharging`,
  `attack_ox` and `attack_whom` on every cycle building (the three leave
  the coverage pin), and the golden widening matches a building's rounds,
  which it had dropped.

Tests: `an_anti_air_building_takes_its_target_and_does_not_fire_it` (the
call site), `an_anti_air_building_winds_up_and_fires_on_its_swing`.

### 84.6 What moved

**The word goes 777 → 779.** The value diff, both sides: `1/2007` on block
778 is `recharging 1`, `attack_ox 7`, `attack_whom 0` in each; ours spends
no draw on 777 (six, the farms', as theirs). The cycle rows agree from 778
to 836. They first part on 837, `attack_ox` 7 against 6, downstream of
779. The first round, block 798, agrees in launch (22408, 16376), `t 1/5`
and `angle` −1137246208.

**779** is the Battery `1/9`'s release of `CHAR_ATTACK2` frame 4 on node
0: ours fires, theirs does not. Block 780 prints the guy at `cur_anim 12`,
`cur_time 4`, `node_flags 14` (bit 0 clear), `turret_angles[0]`
1476220240 against `des_turret_angles[0]` −1978269696: the turret has not
arrived, and theirs releases node 1's frame 7 on 782. `FLAKGUN` carries a
`<RESTRICTION>` on node 4 (−180..180). This crate's gate (§55) is the one
for the event, but `node_flags` and `turret_angles` are parsed by nothing
here (the coverage pin lists them), so the turret's arrival is unwitnessed.
That is the next item's frame, and a hypothesis.

**Mutations**, each on the committed build (`08758cb9`), `git diff --stat`
non-empty first, restored from git and `touch`ed, scored against `sim`'s
two tests, the loader's `the_radar_air_defense_winds_up_on_its_unit_s_packet`
and `cargo test --release -p rondata chapter_thirty_eight`:

| mutation | sim | loader | ch38's pins |
| --- | --- | --- | --- |
| `do_attack`'s in-range return dropped | the call-site test | ok | word 777; widening 639 |
| the head's countdown kept for a cycle building | the call-site test | ok | word test (the height-read pin); widening |
| the cycle's call dropped | the cycle test | ok | word test (the height-read pin, 43 against 41); widening |
| the swing one frame longer (`<` for `<=`) | the cycle test | ok | widening 674 (word 779 still) |
| a building's `ATTENUATE` through `abs` again | ok | **fails** | both pass: no widening compares a round's `accuracy` |

### 84.7 What is not established

- **`near_o`** gates the cycle, and this crate holds it on units only; the
  gate takes the target's sign. run404's two agree in sign on all 1,157
  blocks, and a building whose search sees a candidate it does not take
  would part.
- The string `verify_load` appends (`int_str_array +0x122f0`) is read as
  `-DEFAULT-AGE0`, and only the three numbers above back it.
- The Air Defense Gun's and the SAM's launch vectors; a jammed building;
  a piece outside the first `0x81` (not reachable with the shipped art).
- ~~`do_attack`'s out-of-range arm clears the target (`622c1c`), which
  `process_building_combat` does not; nothing on disk parts on it yet.~~
  Built by item 1131 with the rest of `do_attack`'s target flow (§8.6).
- The rounds of 817, 857 and 867 that print no `AMMO` record.
- A round's `accuracy` is compared by no widening (the golden one matches
  a round by `(who, o, slot)` only), so the sign rests on one printed
  round and the loader's test; and past 779 the stream has parted, so
  the Radar's landings cannot be value-compared on run404.

### 84.8 Coverage

Diff-backed on run404: the cycle (`recharging`, `attack_ox`,
`attack_whom` on 778..836 in the widening), the round's frame (the draw
stream), its launch, time and angle (block 798), and the accuracy's sign
(one round). Read alone: `verify_load`'s type list and slice, the jam
arm, the out-of-range clear.

## 85. The release gate is the event's, not the vectors' (item 1117, 2026-09-28)

Chapter thirty-eight's word stood at 779: the Anti-Aircraft Battery
`1/9`'s `CHAR_ATTACK2` round on node 0, frame 4, which ours fired and
theirs held. Item 1112 read block 780 as the turret short of its aim
(`node_flags` 14). No reader in this crate parsed the turret, so that was
a hypothesis (DECISIONS 42). `docs/journal/2026-09-28-item-1117.md` has
the story.

### 85.1 The instrument first

`GuyData +0x20 turret_angles[4]`, `+0x30 des_turret_angles[4]`, `+0x96
node_flags` and `+0x98 des_node_flags` are parsed now (`gamelog::Guy`).
`golden::widen_turrets` compares them against `sim::anim::Turret`, both
directions, on every figure of every golden widening that walks
`widen_block` (the civilians' chapters, three, four and five). This crate
did not carry `des_node_flags`; it is `Turret::des_flags` now, cleared
with `node_flags` in `set_all_pivots`' one 32-bit store (`005d8bc0:46`)
and set per node whose range holds the bearing (`:129`).

What parts, on every golden window: **chapter thirty-eight alone**.
Chapter fifteen's pivot figure (who=1's `o 6`, 154 aimed records) agrees
on every block. On run404 the Battery's aim is 2,359,296 off theirs from
its first on 743. Both sides then turn in step, 15° a frame. The bits
first part on 801. **On block 780 both sides read `node_flags` 14,
`des_node_flags` 1.** So ours fired through a clear bit.

### 85.2 The gate, from the listing

`GraphicEvents::execute_game_events@008e48e0`, the release arm,
`008e4c28`–`008e4c4c`:

- `has_restrictions(gpiece)` (`GraphicPieces::has_restrictions@0090b640`:
  the piece's type less `0x32` indexes `pivot_restrictions`, and `+0x14`
  of the row is the count). If it is 0, the event passes.
- `movsbl 0x23(event)`, `and $3`: the **event's own node**, sign-extended,
  and its low two bits. If that is at least the count, the event passes.
- `movzwl 0x30(package)` is the package's `node_flags`
  (`Guy::execute_events@005d99c0:57` copies the figure's). With bit `node
  & 3` clear, the event is skipped to `008e4d0d`. It is not deferred: an
  event whose frame passes while the bit is clear never fires.

Nothing in it reads `get_position`'s entries. This crate held the round
only when `pivot::release` had a row for `(piece, anim, frame)`, and the
Chariot's piece 145 is the only one measured. The Battery's piece has no
row, so its node-0 events fired whatever the bits said.

**Built**: `guy_release_events` takes `k = (event node as i32) & 3`
against the type's restriction count (`art.pivots`, the same count the
launch branch uses) and the figure's `node_flags`, on every piece.
`pivot::tests::a_turret_short_of_its_aim_holds_its_release_on_any_piece`
fails with the gate keyed on `release` again.

**Who writes `+0x96` on a figure**, by offset, every spelling:
`Guy::clear@005db590` (the 32-bit zero), `Guy::set_all_pivots` (the zero,
then bit `k` within 15°), `Guy::process@005e0230:36` (each turret on its
aim), `Guy::set_pivot_angle@005d8fc0` (turret 0 at zero), and that
function's inlined copy in `Unit::move_step@005faf30:79–94`, the
cavalry-archer arm when figure 0's `des_node_flags` is 0. `+0x98`:
`set_all_pivots` and `set_pivot_angle`, and the same inline copy. This
crate has the first three. `set_pivot_angle` itself is dead in the image:
no call and no absolute reference (`blind::RESIDUE`). Its inline copy is
a SEAM, reached only by a cavalry archer none of whose nodes bears
(§85.5).

### 85.3 Every instance on disk

A pivot figure that aimed (`des_node_flags` non-zero) is in six dumps
out of 298: run404 (2,038 records), chapter fifteen (154), run147 (60),
and the islands windows run25, 26, 27 and 29 (2, 2, 14, 14). run404's
Battery fires 38 rounds, in pairs. At facing 1222246400, every node-1
round of a `CHAR_ATTACK2` pair leaves from the same point, (−105, −42),
`dz` 172 (783, 808, 858, 883, 908). The node-0 rounds move with the
turret (805, 838, 855, 880, 905, 964, 989, 1013), as §55.2's pivot
branch says.

### 85.4 What moved

**779 → 794.** The value diff on block 780, both sides: `1/9` at
`cur_anim 12`, `cur_time 4`, `node_flags 14`, `des_node_flags 1`,
`turret_angles[0]` 1476220240 against `des_turret_angles[0]` −1978269696
(theirs; ours is 2,359,296 off on both), and **no round of `1/9` in the
air** (ours had `780 1/9 ammo[0]`). The node-1 round is released on 782
on both sides (block 783, `cur_time 1`). Theirs leaves from (21279,
17214). Ours leaves from the figure's square, (21384, 17256), because
the piece's node-1 vector is unmeasured (§85.5). So ours flies 3 frames
where theirs flies 2, and `0/7`'s hit rows part on 784.

**794** is `1/7`'s walk. It parted on 765, where the army's `ATTACKTO`
point is one cell past theirs (parked 1113). Theirs stands at (20604,
16968) from 777. Ours is still walking at (21185, 17322) and spends
`Guy::set_anim+0x97a < Unit::move_step+0x823`. Ours draws 12, theirs 8.

**Mutations**, each on the committed build (`7a9702ba`), `git diff
--stat` non-empty first, restored from git and `touch`ed:

| mutation | sim | ch38's pins |
| --- | --- | --- |
| the gate keyed on `pivot::release` again | the gate's test | word test (fell to 779), word block, widening |
| `set_all_pivots`' `des_flags` write dropped | ok | word block (`des_node_flags` 0 against 1), widening |
| the golden `node_flags` row dropped (the instrument, on `d2d310ca`) | ok | widening |

### 85.5 What is not established

- **The Battery's release vectors.** Node 1 (non-pivot) and node 0
  (the pivot branch's two entries) are unmeasured for its piece. From
  784, run404's rounds leave from the figure's square in ours.
  Measuring them needs either run404's 38 rounds fitted per `(anim,
  frame, node)` as `launch::BAYS` rows (one facing for most keys), or a
  packet's `AttachPos` entries, as run147's were for the Chariot.
- **The Battery's node vector for the bearing** (`pivot::NODES`): its
  aim is 2,359,296 off from 743 for that reason.
- `set_pivot_angle`'s inline copy in `move_step`, which points turret 0
  ahead when no node of a cavalry archer bears. No capture on disk has
  shown it.
- `guy_flags & 0x100` gates `Guy::process`' turn in the original. This
  crate turns every figure of a type with restrictions.

### 85.6 Coverage

Diff-backed on run404: the four turret fields on every figure and block
of the window, and the held round on 780 (the word block test, the
widening, the draw stream to 793). On chapter fifteen: the four fields.
Read alone: the gate's `has_restrictions == 0` arm, and the writers in
§85.2 that this crate does not have.

## 86. An idle ship does not take a land building out of its range: `check_target`'s head (item 1214, 2026-09-29)

East Indies' second-pair word stood at 8907: ours 2 draws against 28,
parting at index 1, where the original spends `Unit::think_scout+0x941`
and 26 `+0xaba` under `Unit::think+0x7da < Unit::do_idle+0x94`. No
mechanism was named. `docs/journal/2026-09-29-item-1214.md` has the story.

### 86.1 The event, both sides

`1/35` is a **Caravel** (unit type 275; `ATTACK 13`, range 7), the
computer's, finishing an `EXPLORE_TO` at (7392, 480) on the sea. Both
sides spend its `Unit::do_idle` roll on 8907. Then:

- **The original** (run445, block 8908): no attack. The think goes on to
  its tail, `think_scout`'s 27 draws, and `1/35` stands in a new group 79
  on an `EXPLORE_TO` (504, 504) with an eight-node path.
- **Ours** (`RON_DEBUG_UNIT=1/35`): the auto-attack arm's
  `find_melee_target` answers the human's building `0/2004`, at (4992,
  4992) inland, and `1/35` takes an `ATTACK` on it (`order:kind` 10) and
  keeps group 65. The think ends there, so no `think_scout`.

The building is about 5,100 units from the ship, inside the computer's
search radius (`unit_respond_range × 0x180`) and far out of its range.

### 86.2 The head, from the listing

`Object::check_target@00649e00`, `649e3a`–`649fb7`, for a unit searcher
(vslot `+0x18`) and `param_7 == 0`:

```text
other = get_tregion(target's tile) != get_tregion(my tile)      # 649ec2, 649ed7
if other && (unit_masks & 0x40000)                              # 649f05
         && has_objmask(0x40000)                                # vslot +0x148, 649f17
         && type->+0x218 == 1:                                  # 649f29, cmove
    other = 0
if ((param_3 == 0 && stance() == 1 && update_order() != 0)      # vslot +0xf4, 6179d0
     || other)
   && !is_in_range(o, who, x, y, y, 0, 0):                     # 649fb0
    return 0
```

`get_tregion@006b52e0` is the coastal-refined region (`World::tregion_alt`):
a `0x100` cell answers its `region2` when the tile is ocean.

`Object::find_nearby_target@00648da0` calls it at `64955d` with `param_3 =
local_54`, which is `Unit::on_duty` for a unit searcher (`6490b6`) and 0
otherwise, and `param_7` its own cavalry-archer argument. The refusal
comes before `near_o` is written. `Unit::fight`'s guard call passes
`param_3 = 1`.

So a candidate in another region is taken only in range, unless the
searcher is a computer's `SIEGE` ship. A `DEFENSIVE` unit that is off
duty and has an order must reach anything it takes, in its own region
too.

### 86.3 The build

`Sim::check_target_reaches(u, target, duty)` (`fight.rs`) is the head.
`find_nearby_target_with` asks it for every unit searcher, after the
`flags` filter and before the guard's leash, with `duty` the searcher's
`on_duty`. `guard_check_target` asks it with `duty` set. Its region test
read `World::tregion` before, without the coastal refinement, and reads
`tregion_alt` now, as the listing does.

`fight::tests::a_candidate_in_another_region_is_taken_only_in_range`
fails when the search does not ask it.

### 86.4 What moved

**8907 → 10183.** The value diff on run445's block 8908, the word's own:
`1/35`'s `group` ours 65 against 79, `order:kind` 10 against 3,
`orders_x`/`orders_y` (7392, 480) against (504, 504), and `path:length`
0 against 8 all agree now. `1/35` parts no key through 9071. run445's
keys went 627 → 195. `1/68`'s and `1/70`'s figure clocks, which parted on
8945, agree through 9071. Frame 8907's draws went 2 against 28 →
agreeing. The new word's delta: ours 24 draws and the original 9 on
frame 10183, parting at index 0, where ours spends
`Leader::create_units+0x642` and the original `Guy::set_anim+0x97a <
do_cast`.

### 86.5 What is not established, and coverage

**Diff-backed** on run445: the region arm, for a computer's ship and a
land building. **On run484** (item 1254, `docs/GOLDEN.md` §53): the
`SIEGE` ship exception, for a computer's Bomb Vessel and a human's
Barracks, and its computer's conjunct, for a human's Bomb Vessel and a
computer's Barracks, to the chapter's word 670.

**Listing-backed, never executed in a capture**: the exception's sea
conjunct; the defensive arm (`on_duty`, `DEFENSIVE`, an order), which no
staging reaches under `!ai off` (no writer of the stance byte sets 1,
`docs/GOLDEN.md` §53); the coastal refinement's effect on the guard's
call. §86.6 has what a mutation of each did.

SEAM: `find_nearby_target`'s cavalry-archer argument (`param_4`, passed
on as `param_7`) turns the head off. This crate has no such caller.

### 86.6 The killers

Each mutation was scored by the whole `rondata` and `sim` suites:

| mutation | unit test | walk |
|---|---|---|
| the search does not ask the head | fails | run445's widening (`1/35` back on 8908) |
| the region read without the coastal refinement | passes | none |
| no `SIEGE`-ship exception | fails | ~~none~~ chapter forty-four's walk (word 611) and widening (item 1254) |
| no defensive arm | fails | none |
| `duty` false for every search | passes | chapter one's word and widening, chapter eight's widening |

So the region arm and the duty exemption are diff-held, and since item
1254 the `SIEGE` ship too: without `unit_masks & 0x40000` in it the walk
parts on 626, and the unit test asks a human's siege ship. The coastal
refinement and the defensive arm's own refusal are built on the listing
alone.
