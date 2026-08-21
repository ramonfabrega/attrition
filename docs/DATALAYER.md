# The data layer into the sim, and the gamelog diff

Phase 3's queue item 5 (2026-08-20): the shipped tables become the
simulation's typed data through one loader, the original's own per-frame
dump is read back by one reader, and a harness stands the simulation up from
the dump's initial state and steps it against the logged frames. This is the
document for all three, and for what the dump taught on the way.

**How this was established.** The loader is written from the mechanic
documents' loader passages — `docs/TECH.md` (the type space, `tech_key`, the
derived prerequisites), `docs/COMBAT.md` §2.1 (the combat columns), `docs/
CITIES.md` §1.5 (`BuildType::init`), `docs/COSTS.md` (`COST`/`SUPPORT`/
`PROGRESSION`), `docs/PRODUCTION.md` (the time columns), `docs/MOVEMENT.md`
(`MOVES`, `TURN_SPEED`) — and re-checked against the decompile wherever the
loaded result disagreed with what a reading predicted (`Types::unit_key`,
`BuildType::init`'s back-links, `XMLElement::get_text`). The reader is
written from two logged runs of this install (`docs/ORACLE.md`, last
section). The harness is the two joined.

**Confidence.** High for the reader (two dumps, 114 MB and 20 MB, parse to
the last line and round-trip through typed views). High for the loader's
arithmetic, which is the documents' arithmetic, and for the three things the
loader settled in the decompile (below). Medium for the fields no document
names a column for, which the loader leaves at their defaults and lists under
"not established". Honest and low for the diff's *score*, which is the
expected ceiling of a simulation with no AI and no order stream — and which
is not the point yet; the wiring is.

**Where the implementation is.** `crates/rondata/src/gamelog.rs` (the
reader), `dump.rs` (the constants check), `load.rs` (the loader), `diff.rs`
(the harness); `cargo run -p rondata -- <install> --gamelog <Logs/gamelog.txt>
[--diff [N]]` runs all of it, and `--types <dump>` checks the loader's
per-unit `Kind`s and the combat table it builds against a `DUMP_ALL=1`
start-of-game dump's `UNITTYPE` blocks and `COMBATTABLE`
(`crates/rondata/src/typesdump.rs`, `docs/COMBAT.md` §15.2). The install-backed tests run when
`RON_INSTALL` points at the game, or `../../game` from the crate exists;
without either they pass vacuously, so the binary is the check with teeth.

---

## 1. The reader — `Logs\gamelog.txt`

The 2003 `Log` system writes a nested text dump with no schema and no `END`
markers: `BEGIN <name>` opens a block at an indent of one space per level, a
block closes when a later `BEGIN` appears at the same or a shallower indent,
and any other line is a field — first token the key, the rest the value. Two
rules the writers force on a reader:

- **A field belongs to the innermost open block regardless of its own
  indent.** `LeaderData::log_data` writes `leader_flags` one level shallower
  than the `who`/`tribe` lines before it, inside the same `BEGIN LEADERDATA`.
- **Keys repeat.** An array constant is one line per element under one key
  (`fort_upgrade_terr[scan] 2`, `… 4`, `… 6`, `… 9`), in index order.

Everything borrows from the text; the 114 MB dump parses in about a second.
`Log::initial()` gives the start-of-game state — `GAME INFO` → `GAMEINFO` and
its `PLAYER`s, `WORLD`, `CITIES`, `CONSTANTS`, every `UNITDATA` with its
`GUY`s, every `BUILDDATA`, every `LEADERDATA` — and `Log::frame_states()` the
`FRAME n` blocks.

### What detail level 0 writes

Per unit per frame: the `Object` base — `flags`, the object number `o`, the
owner `who`, `x_internal`/`y_internal`/`z_internal` in position units (1/768
of a cell, 192 to the tile, the same unit `sim::Pos` uses). The `GUY` blocks
carry `type` (a **`TypeIndex`**: `0x32` is the Citizen, `0x45` the Scout),
position and `angle` **only in the start-of-game dump**; per frame they are
written empty. Per leader: `who`, `tribe`, `defeated_by`, `gov`, `score`,
`leader_flags`, `leader_flags2` — **no goods**. The non-player leaders are
`who 8` and `who 9`. Buildings write the same object base, under
`BUILDDATA` → `WALLDATA` → `OBJECT` → `SUBOBJECT`, with **no type**: the
starting city is the one with object flag `0x20` set. Animals are units with
`who 255`. Nothing of the terrain is written under the categories the two
runs enabled. The richer fields (`UnitData::log_data` goes on to fifty more)
wait on a detail-level argument that `DUMP_ALL=1` presumably raises; untried.

### The `CONSTANTS` block is the loaded struct, by field name

Its keys are the **`Constants` struct's field names, which are the lowercased
`rules.xml` tags** — 716 of the 723 tags match that way, one more through a
rename (`CITY_UPGRADE_TERR` is loaded into `city_level_territory_bonus`;
`Constants::init` reads it so), and the values are the **in-memory
representation**. That makes it a direct oracle for every representation
claim `sim::tuning::Slot` makes, and `rondata --gamelog` checks them all:
**231 of the 232 `Tuning::RON` slots equal the program's own loaded value**;
the 232nd, `LIBERTY_FREE_UPGRADES`, is loaded (`Constants::init` line 1442)
and not logged. Seven tags are loaded and not logged (`TAJ_CARAVAN_LIMIT`,
`LIBERTY_FREE_UPGRADES`, `EIFFEL_SIEGE_RANGE`, `SPANISH_EXTRA_SCOUT`,
`JAPANESE_AIRCRAFT_CARRIERS_SPEED`, `KOREAN_START_CITIZEN`,
`KOREAN_FREE_CITIZEN`); forty keys are struct members with no tag (camera,
editor, scenario and diplomacy state), four of them — `taj_caravan`,
`kremlin_spy_instant`, `german_light_cavalry`, `russian_uber_spies` — at `−1`,
which is what `get_item` returns for a key the file does not have.

Classifying every matched constant by which rescaling of the written value
reproduces the dumped one:

| rule | count | examples |
| --- | --- | --- |
| plain (`written_int`) | 583 | `ATTRITION 48`, `FLANK_BONUS 50%` → 50 |
| ×256, `String::fraction(s, 0x100)` | 25 | `ROCKY_MODIFIER 2/3` → 170, `PEASANT_RATE 10` → 2560, `OIL_RATE 35` → 8960, `SCHOLAR_RATE` each, `RESEARCH_PREMIUM 1/1` → 256, the patriot constants |
| ×100 | 5 | `UNIT_RATE_BASE 6/5` → 120, `UNIT_RATE_PROGRESSION 3/4` → 75, `ACCEL_TRAIN/CONSTRUCT/RESEARCH 1/1` → 100 |
| ×192, `get_fraction(s, 0xc0)` | 10 | `TARGET_RADIUS 1/2` → 96, `UNIT_FORMATION_SPACING 1/16` → 12, the train/board/disembark distances |
| ×48, a `UCoord` | 1 | `UNIT_BLOCK_RADIUS 1 UCoord` → 48 (`Constants::init`: `* 0x30`) |
| ×10, an attack | 1 | `CARAVAN_ATTACK_BONUS 2 attack` → 20 |
| ambiguous | 85 | zeros and `1/1`s the rules cannot tell apart |
| a formula | 1 | `AMERICANS_MARINE_ENTRENCH 5` → 7: `n − n/15 + 2` |

Two array facts the same pass surfaced. `scholar_rate` is `int[6]` in the
symbols and the log writes **five** of it — a logging count, not a loader
one; `Tuning::RON` keeps six and the checks compare the overlap.
`KOREAN_CITIZENS` writes nine entries into an `int[8]`; `entry8` is dead.

### Reproducibility, to the position unit

Two separate runs with `Seed (0 for random)=12345` — one logging everything
for 79 frames, one logging `UNITS`/`LEADERS`/`DEATHS`/`CHECKSUM` for 1,730 —
put the same units at the same coordinates on the same frames: the AI's
scout leaves `(42648, 18072)` for `(42651, 18106)` on frame 2 in both, the
human's citizen `o 1` leaves `(4248, 28680)` for `(4267, 28664)` on frame 4
in both. The seed fixes the game, not just the map.

---

## 2. The loader — the tables into the sim's types

`rondata::load::load(&install)` reads `rules.xml`, `unitrules.xml`,
`buildingrules.xml`, `techrules.xml`, `resourcerules.xml` and `balance.xml`
and returns a `Loaded`: 364 `sim::UnitType`, 129 `sim::build::BuildType`, a
`sim::tech::TechTree` of 628 entries, a `sim::combat::Table` over the unit
ids, the name lists, and the maps between the index spaces.
`Loaded::sim(tuning, world, players)` is a `sim::Sim` with all of it
installed and every player at the starting position.

### Index spaces

Record index is identity in each table — unit *i* is `unitrules.xml` record
*i* (352 units then 12 gaia), building *j* record *j* — and the tree is laid
out in the original's `TypeIndex` order without its gaps: goods `0..50`,
units `50..414`, buildings `414..543`, techs `543..628`, so that
`Loaded::type_index` recovers the original's id (`0x32 + i`, `0x19e + j`,
`0x220 + k`) for any entry and `unit_of_type_index` inverts it. That is what
the harness keys the dump's `GUY type` on.

### What each column becomes

| column | becomes | rule |
| --- | --- | --- |
| `NAME` | `*_names` | the interface name |
| `TYPENAME` | `*_type_names` | **what the keys match** — see below |
| `ATTACK` | `Profile::attack`, `BuildType::attack` | ×10 |
| `HITS`, `ARMOR`, `RECHARGE`, `AMMO_PER_ATT`, `SPLASH_PERCENT`, `TO_HIT` | plain | `TO_HIT` defaults −1 |
| `ATTENUATE` | plain | absolute value |
| `SPLASH` (units) / `SPLASH_AREA` (buildings) | `splash_area` | plain |
| `RANGE` | `min_range`/`max_range` | `min-max`; no `-` ⇒ `max = min`; `FLAGS` `k` moves max to `second_max_range` and zeroes `max_range` |
| `PROJ_SPEED` | plain | 200 when absent or zero on a ranged type |
| `OBJ_MASK`/`OBJ_MASKS` | `obj_masks` | `combat::mask::parse` |
| `FLAGS` | `unit_flags` | letter `c` → bit `c−'a'`, digit → `+26` |
| `BUILD_FLAGS` | `BuildType::flags` | `build::flags::parse`, digit → `+25` |
| `TARGET_SIZE`, `BLOCK_RADIUS` | `target_size`, `block_radius` | × `UNIT_BLOCK_RADIUS` (48) |
| `UBER_SIZE`, `POP`, `PROGRESSION` | plain | |
| `MOVES` | `UnitType::moves` | position units per frame, as written |
| `TURN_SPEED` | `UnitType::turn_speed` | degrees through `degrees_to_angle` |
| `JOB_TIME` | `Times::job_time`, `BuildType::job_time` | frames |
| `JOB_EXTRA_TIME` | `Times::job_extra_time` | `(num × 100) / den` |
| `RESEARCH_PREMIUM_TIME` | `Times::research_premium_time` | ×256 |
| `COST` | `Price::base` | `load_cost`: six slots by letter, unscaled |
| `SUPPORT` (units) | `Price::support` | the two slots, written order, zeros skipped |
| `SUPPORT0/SUPPORTVALUE0`, `SUPPORT1/SUPPORTVALUE1` | `Price::support` | resource as a word |
| `DOMAIN` | `Domain` | `Land`/`Sea`/`Air` |
| `X_SIZE`, `Y_SIZE`, `GARRISON_MAX`, `PLUNDER`, `BASE_ARROWS`, `MOST_SHOTS` | plain | |
| `PLUNDER_GOOD` | `plunder_good` | a resource word |
| `PREQ0..2`, `OBSOLETE`/`OBS` | `TypeDef::preq`, `obs` | `tech_key`: `none`, `disable`, or a tech by name |
| `FROM`, `JUMP`, `GRAFT`, `WHERE` | `from`, `jump`, `graft`, `where_` | `unit_key`/`build_key`, by `TYPENAME` |
| `TRIBE_MASK` | `tribe_mask` | `rondata::tribe_mask`, MSB-first |
| `AGE` | `TypeDef::age` | techs |

Derived at load, as the original derives them: the tech `Kind` from the
record's position (7 ages, 4×7 epochs in Science/Commerce/Civic/Military file
order, 4 finals, 40 plain, 6 governments in three tiers); `UnitTraits` from
`FLAGS` `h`/`j`/`y`, `OBJ_MASK` digit `1` (hero), the patriot range
`302..=307`, and `ATTACK ≠ 0`; `Kind::Building { auto }` from `BUILD_FLAGS`
`c`, `wonder` from the range `112..=128`; then `TechTree::finalize` — the
implicit Military epoch, the `upgrade` back-links, the buildings' `to`/
`upgrade`/`where`. On the sim side: the ramp class (scholar by id; citizen,
merchant and caravan as worker; `OBJ_MASK` `C` as other civilian; else
military), the attrition `UnitKind` through `kind_exempt` with the lineage
tests (`Scout` line as special, `Spy`, `Caravan`/`Merchant Fleet`, the
`Supply Wagon` line and the three patriots `302/304/306` as supply), the
combat roles by lineage and by `WHERE`, the production group (the `WHERE`
building when `PROGRESSION` bit 0 is set and `ATTACK ≠ 0`), the building
`Ident` by name, `BuildClass` (city lineage; tower/fort/Airbase/attacking as
defensive; a `WHERE` of an attacking unit as military trainer; of any unit as
training), and `balance::Kind`'s named lineages for the combat table.

### Three things the loaded result settled in the decompile

1. **The keys match `TYPENAME`, not `NAME`.** `Types::unit_key` compares the
   text against `TypeData + 0xb0`, and `UnitType::init` fills `+0xb0` from the
   third text column it reads — `TYPENAME` — falling back to the name when
   the column is empty. The shipped file relies on it: `FROM Marines`, `JUMP
   Arquebus Immortal` and `JUMP ECONQUISTADOR` name no record's `NAME`; they
   are the `TYPENAME`s of `Continental Marines`, `Arqimmortal` and `Elite
   Conquistador`. With `NAME` the loader logged three unresolved keys; with
   `TYPENAME` it logs none. `docs/TECH.md`'s "`type_name`" was this field all
   along.
2. **The `to`/`upgrade` back-links are last-writer-wins in record order.**
   `BuildType::init` writes `B[from].to = this` for every record with a
   `FROM` (not a hero), unconditionally; the Forbidden City (record 117)
   names the Small City after the Large City (record 1) does, so the
   program's Small City `to` is the Forbidden City. `UnitType::init` does the
   same for `U[from].upgrade`: the Hoplites' is whichever later record named
   them last (the Greek Mercenaries), not the Phalanx. Reproduced as read.
3. **Fields are read by tag name, from the internal string table** — see
   `docs/FORMATS.md`'s correction. The loader was right to read by name; the
   earlier "tag names are comments" was wrong at the field level.

### Fields the loader leaves at their defaults

No reading names a column for these; each is recorded as an input:
`Profile::guy_radius` and `big_radius`; `Profile::combat_role` (`role &
0x10000` — the `role` word's source is unread; the loader uses `ATTACK ≠ 0`
and not `OBJ_MASK C`); `Tribe::graft` (identity) and `Tribe::barbarian`
(false); `TechTree::free_rules` (empty — the nation and wonder free-tech
blocks); `TypeDef::is_list` and `leader_off`; `balance::Kind::age` is the
`get_age_slow` reading (first tech prerequisite's age — ~~−1 → 0~~ **an age
tech's `AGE` column plus one, any other tech's `AGE`, no tech 0**; the first
draft omitted the +1 and the dump's per-type `age` caught it,
`docs/COMBAT.md` §15.2 — checked equal for all 364 units). The
production group's identity for "factory units" (whether the Auto Plant,
Factory and Siege Factory share one count) is taken as the `WHERE` building
itself.

### The `balance.rs` domain

`rondata::balance::unit_kind` read a `TYPE` column that does not exist and
called every unit a land unit; the column is `DOMAIN`. Fixed.

---

## 3. The harness — the dump against the simulation

`rondata::diff::build_sim(&loaded, &initial, tuning)` makes the world `xs ×
ys` cells from `WORLD`, one simulation unit per logged unit of a player slot
(typed by its first `GUY`'s `TypeIndex`, squad size the number of `GUY`s,
health the type's `HITS`, speed and turn rate from the type), each player's
tribe from `LEADERDATA`, and the starting city by type at the `CITIES`
position — through `place_building`, which **refuses it (`Blocked(Ruins)`)
on a world with no terrain**, so the city is added untyped with a note.
`run` then steps the simulation to each logged frame and compares every
linked unit's position.

**The score, on both dumps: ticks before divergence 1.** The AI's scout
moves on frame 2 and the human's citizen `o 1` on frame 4 (at 25 position
units a frame — the starting citizens walk to the pre-placed sites, which is
what objects `2001–2005` beside units `1–5` are), and the simulation, with no
AI and no order stream, holds everyone still. That is the expected ceiling,
and the run still proves the wiring: every `(who, o)` in 1,730 frames links
to a simulation unit (25,943 unit-frames compared, none unlinked), every
`GUY type` maps to a unit record, and the world scale, the ids and the step
cadence agree. The leaders' logged `score` is reported and not matched; the
simulation has no score.

---

## What is not established

- **`DUMP_ALL=1`.** Presumably the detail-level-1 fields — goods per leader,
  the fifty more per unit, types on buildings. One run settles it, and until
  it is run the diff sees positions and scores only.
- **The terrain.** Nothing of the map is in the dump at level 0, so
  `place_building` cannot be exercised and territory cannot be computed from
  a dump. `TERRAIN`/`MAPMAKE` under `[Start Game]` were enabled in the full
  run and wrote nothing recognisable; whether they write at level 1 is open.
- **An order stream.** The diff's score cannot move until the simulation is
  fed what the players did. The recorded-game container (`docs/ORACLE.md`
  Part 1) is the source; it remains unread against a file.
- **The `role & 0x10000` word, `guy_radius`, `big_radius`**, the graft tables
  and barbarian flags, the free-tech rules, `is_list` — the defaults above.
- **Which duplicate a duplicated `CONSTANTS` tag resolves to** under by-name
  lookup (`get_item` → `XMLNode::get_element`, presumably the first). The
  shipped duplicates are equal, so the dump cannot tell.
- **`Constants::log_data`'s omissions** — seven loaded tags and the sixth
  `scholar_rate` — are the logger's; the loader is not affected.
