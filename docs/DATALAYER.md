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
per-unit and per-building `Kind`s and the whole 493 × 493 combat table it
builds against a `DUMP_ALL=1` start-of-game dump's `UNITTYPE`/`BUILDTYPE`
blocks and `COMBATTABLE` (`crates/rondata/src/typesdump.rs`, `docs/COMBAT.md`
§15.2–§15.3). A building's `domain` is `BuildType::set_domain` from
`BUILD_FLAGS` (`b` → Sea, `a`+`b` → Air), not a column. The install-backed tests run when
`RON_INSTALL` points at the game, or `../../game` from the crate exists;
without either they pass vacuously, so the binary is the check with teeth.

---

## 1. The reader — `Logs\gamelog.txt`

The 2003 `Log` system writes a nested text dump with no schema and no `END`
markers: `BEGIN <name>` opens a block at an indent of one space per level, a
block closes when a later `BEGIN` appears at the same or a shallower indent,
and any other line is a field — first token the key, the rest the value. Two
rules the writers force on a reader:

- **A field's home is decided by indent, and one shape is genuinely
  ambiguous.** A block's fields are written one level deeper than its `BEGIN`,
  and nothing writes `END` (`Log::end` emits only for a non-empty name, and
  the order writers all pass the empty string), so indentation is the only
  thing that closes a block. But a field at *exactly* an open block's own
  indent occurs in two forms that indentation cannot tell apart:

  ```text
  BEGIN LEADERDATA   (1)      BEGIN TARGETORDER  (4)
   who 0             (2)       BEGIN UNITORDER   (5)
  leader_flags …     (1)        flags 0          (6)
                               ox 2001           (5)
  ```

  `leader_flags` belongs to the `LEADERDATA` it follows; `ox` belongs to
  `TARGETORDER`, the **parent** of the `UNITORDER` it follows. The reader
  therefore records such a field on **both** candidates — the enclosing block
  the indent gives, and the block it just closed — and a run of them stays
  with the same block until the next `BEGIN`.

  This was originally written as "a field belongs to the innermost open block
  regardless of its own indent", which the second reading flagged
  (`docs/audit/2026-08-21-orders.md` R7 L19) and which turned out to be
  **load-bearing**: under it, a unit's `length`/`type`/`metric` order-list
  lines are filed under the preceding empty `STACK<TYPE>` and every order's
  `ox`/`whom`/`uid` under `UNITORDER`, so the whole order list reads as
  absent. `docs/ORDERS.md` §11.1 has the order side.
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
tribe from `LEADERDATA`, and the starting city at the `CITIES` position.
It then places the **pre-placed buildings and the starting citizens' gather
orders**, derived from `docs/ORDERS.md` §9.2 and §9.3 rather than read out of
the log: `BUILDDATA` carries no type at any detail level, so `2001` is typed
the woodcutter and the next few farms by `Leader::produce_building`'s order,
and each citizen is assigned by §9.3's four-step rule. `run` then steps the
simulation to each logged frame and compares every linked unit's position
**and its order list**.

**The map is one land region over every cell** ~~, stated here because it is
an assumption and not a reading: the dump carries no terrain (below)~~ —
**unless the start dump carries the cells**. Corrected 2026-08-24: a dump
at `WORLD ≥ 5` under `[Start Game]` prints every cell's `WData` record
(`docs/ORACLE.md`, "The map is a dump too"), and `build_sim` then builds
the regions, the coastal `region2`, the owners, the site values and the
goods bits from it (`gamelog::world_cells`, `Built.region_map`); the flat
world is the fallback for the dumps made before that was known. The flat
fallback matters because a cell
in no region at all is `Blocked::Ruins` to `blocked_tcoord`. Leaving the
world region-less was not neutral — ~~the city is added untyped with a
note~~ **every `place_building` was refused**, so the city and every
pre-placed building came out untyped and city-less, and a farm outside a city
sends its citizen away (`do_gather`'s second gate). The order diff below is
what found it; the positions could not.

**The sync stream and the heights come from the dump too, when it has
them — or from a sibling** (2026-08-24). A dump made with
`check_all_level=14` and `[Misc Logging] CHECKSUM=2` carries the setup
path's checksum trace (`gamelog::Checksum`, `Initial.checksums`;
`docs/ORACLE.md`, "The setup path's checksum trace is the RNG state"), and
`build_sim` seeds each computer leader's personality roll from its
`Leader::init` bracket — noting whether the roll lands on the far end —
then installs the last record as the state entering frame 0. A `DUMP_ALL`
dump carries `master_land_heights` (`Initial.heights`, exact millionths),
which `build_sim` pins per tile as `find_tcoord_z` would answer it
(`World::tile_z`). Neither is in the runs made before they were known, so
`diff::run_traced` and `rondata --diff --sibling <dump>` borrow what a dump
lacks from **siblings** — other dumps of the same lobby and seed, hence the
same setup stream and the same map: run11 for the trace, run3 for the
heights. Without either the notes say so, and the stream is the sim's own.

**The buildings go in the way §9.2 says, not through `place_building`.**
`build_cities@005ab910` is `Build::init` then `activate(0, 0, 0)`, and
`produce_building` at frame 0 is `Objects::init_build` then
`activate(0, 1, 0)` — neither pays a cost and neither re-runs `blocked_site`,
because the site was chosen already. So the harness calls `init_build` and
`activate` too. `init_build` is also what runs `Wall::find_city`, which is
how a farm gets the city membership its gather path needs.

**The buildings' types and their mining lists come from the dump, not from a
derivation.** At `BUILDS=6` a building carries `orig_type` and at `BUILDS=7`
it carries `gather_from` (`docs/ORDERS.md` §11.2), so the harness reads both.
§9.2's typing rule is still *derived* alongside and any disagreement is a
note — the same derive-then-read discipline as the starting orders — but the
dump wins where it speaks. `gather_from` is an input by construction: the
original fills it from the terrain, which no dump carries.

**The score, on `gamelog-run6`: ticks before divergence 1**, and it is worth
being precise about why, because the single number hides the state of the
port. `Report::first_divergence_by_unit` gives the breakdown; over all 432
frames it is `0/2@4 0/3@103 0/4@103 0/5@103 1/0@2 1/1@2 1/2@4 1/3@103
1/4@103 1/5@103` — **and `0/1`, a woodcutter's citizen, never disagrees at
all: 432 frames of matching positions and matching order lists.** The score
is a minimum over every unit, so it is pinned by whichever unit the
simulation cannot yet drive:

- The **AI's units** (`1/0` the scout, then the rest) move because the AI
  orders them, and there is no AI. Nothing but the order stream or phase 5
  moves those.
- The **farm citizens** (`0/3`, `0/4`, `0/5` and player 1's) hold to frame
  **102**, where the original inserts a move in front of their gather order
  and ours does not — the farm re-target, and the measurement that says
  `FARM_GROWS = 200` is wrong by about a factor of two (`docs/ORDERS.md`
  §6.5). Before this run the farmers "agreed" only because both sides stood
  still.
- `0/2`, the second woodcutter's citizen, parts on frame 4 by a few position
  units and rejoins the argument only at 428 — a path difference, not an
  order one (`path-to`, below).

The harness also runs a **derive-then-read check** (`check_start_orders`):
the starting orders it derived from §9.3 against the `GATHERORDER` blocks the
original actually wrote in the first logged frame, matched by `(who, o)`. All
ten citizens agree — `0/1→2001 0/2→2001 0/3→2002 0/4→2003 0/5→2004` and the
same for player 1 — which is the real evidence that the rule is right, and
none of it is visible in a position diff. **Two simulations can agree on
every position for a whole dump and still have given every citizen the wrong
job.**

The runs also prove the wiring: every `(who, o)` links to a simulation unit
(none unlinked, on either dump), every `GUY type` maps to a unit record, and
the world scale, the ids and the step cadence agree. The leaders' logged
`score` is reported and not matched; the simulation has no score.

**One reader correction was worth the whole run.** `GameLog::end_game`'s
`full_dump` writes its records as further children of `BEGIN GAME`, *after*
the last `FRAME` — so "the object records among `GAME`'s children" is the
first frame's world merged with the last one's. `Log::initial` now cuts at
the first `FRAME` child. Before the cut, `run6` read back as 27 buildings and
27 units where the game began with 13 and 12, `gamelog-run1-fulldump` as 400
citizens a player, and the duplicate `(who, o)` links quietly took the
start-of-game assignment away from the real ones. **A position diff cannot
see this either** — the phantom units match nothing and are never compared;
it surfaced as ten starting citizens whose derived order came out `None`.

### 3.1 The order diff — the intent, every frame

`check_start_orders` makes that comparison once, at frame 0, for one order
kind. **`compare_orders` makes it every frame, for the whole list** — the
last of `docs/ORDERS.md` §13's harness bullets, and the reason the section
above could say "agreeing on standing still is not evidence" and then do
something about it.

Both sides are walked **front first**, the order being executed at slot 0.
That means reversing the log's: `OrderList::log_data` walks the ring from the
tail, so the *last* block it writes is the current order (§11.1), and
`UnitDump::orders_front_first` is the reversing iterator. The path stack
needs no reversing — the log writes it bottom first (the goal, pushed first
by `find_path`) and `Vec<PathData>` is pushed and popped at the end, so the
two already line up. Getting either backwards is a silent, total
disagreement, which is what the round-trip test asserts.

Per slot it compares the `OrderIndex`, the action bit, the whole `flags`
byte and `TargetOrder`'s `whom`/`ox`; then the path stack's depth and each
segment's goal. Two rules keep it from manufacturing disagreements out of
what it cannot see:

- **A target is compared only when both sides name one.** A move order has
  none; a simulation building the start-of-game rule did not create has no
  logged object number. An attack order is the awkward one — the original's
  `AttackOrder` *is* a `TargetOrder` and carries the target, while here the
  order wraps `combat::State`'s (`docs/ORDERS.md` §13), so the comparison
  reads the unit's combat target instead.
- **The `flags` byte is reported but does not score.** `0x8` and `0x10` have
  no established reader (§14) and `0x1` (`PATHED`) follows the path stack.
  ~~The path stack does not score either: counting it would be scoring the
  stub.~~ **The path stack scores since 2026-08-23** — the pathfinder landed
  (`docs/PATHFINDER.md`), the stack is a modelled output, and
  `order_ticks_before_divergence` counts its disagreements.

**On `gamelog-run6`, 5,184 unit-frames compared over 432 frames: not one
disagreement of `flags`, `action` or `target`.** Every field the simulation
models agrees wherever it is comparable. What is left is 768 `length` and 511
`kind` — a missing order on a unit the simulation cannot drive, and the slot
shift that follows from it — plus 750 `path-length` and 33 `path-to`, ~~which
are the pathfinder stub~~ **which the pathfinder's landing (2026-08-23)
re-attributed rather than removed**: player 0's second woodcutter, the stub's
visible gap, now matches the original's stack for 427 straight frames, and
every remaining path disagreement is on player 1's mirror units, whose
straight lines cross forest the harness's flat world does not carry
(`docs/PATHFINDER.md` §10) — a world-data gap, not a search gap. On the
shorter `gamelog-run4` (47 frames, `BUILDS=6`) the same check reports 111
`length`, 95 `kind`, 108 `path-length`. The order score
(`order_ticks_before_divergence`) is a minimum over every unit the same way
the position score is, so the AI's scout pins it at 0; the per-unit breakdown
is the number to read.

**It has now earned itself three times, and each time on something a
position diff could not show:**

- The six farm citizens holding **`THINK` where the original holds
  `GATHER`, from frame 1** — `do_gather`'s `Ident::Farm && city.is_none()`
  gate, firing because the region-less world had refused every
  `place_building` and left the farms outside any city. The citizen stands
  inside the farm's footprint either way, so both simulations agreed on its
  coordinates for all 47 frames while one of them had sent it to think about
  its life instead of working.
- The **duplicate start-of-game state** from the end-of-game dump (above),
  which showed up as ten citizens deriving `None`.
- The **farm re-target at frame 102**, against a documented assumption of
  ~200 (`docs/ORDERS.md` §6.5). The two sides' positions part on 103, but
  only after the order list had already said why.

---

## What is not established

- **`DUMP_ALL=1`.** Presumably the detail-level-1 fields — goods per leader,
  the fifty more per unit, types on buildings. One run settles it, and until
  it is run the diff sees positions and scores only.
- ~~**A `BUILDS=7` dump.** The single blocking item for the harness.~~
  **Captured 2026-08-21**: `gamelog-run6-ancient-nubian-builds7.txt`, 432
  frames, `UNITS=3 BUILDS=7 CITIES=5 GUYS=2` under **both** `[Start Game]`
  and `[End Frame]` (`docs/ORACLE.md`). With `gather_from` read into the
  harness, player 0's woodcutter citizen `0/1` matches the original for the
  whole run.
- ~~**What a `DUMP_ALL=1` dump's initial state actually contains** — 400
  citizens a player where `run4` reads 5.~~ **Answered, and it was not
  `DUMP_ALL`:** `Log::initial` was reading the end-of-game `full_dump`'s
  records as part of the start state, because they are children of `GAME`
  like the frames (above). The fulldump still carries **no order lists** (it
  is below `UNITS=3`), so nothing of §3.1 is measured on it.
- **The terrain.** Nothing of the map is in the dump at level 0, so
  `place_building` cannot be exercised and territory cannot be computed from
  a dump. `TERRAIN`/`MAPMAKE` under `[Start Game]` were enabled in the full
  run and wrote nothing recognisable; whether they write at level 1 is open.
- **An order stream.** The diff's score cannot move until the simulation is
  fed what the players did. The recorded-game container (`docs/ORACLE.md`
  Part 1) is the source; ~~it remains unread against a file~~ **the reader
  exists** (2026-08-24, `docs/RECGAME.md`, `crates/rondata/src/recgame.rs`,
  `rondata --recgame`): the heavengames sample under
  `game/external-recgames/` parses end to end — lobby, seed, embedded rules
  (its combat table equals ours cell for cell), and 21,884 command packages
  with frame/play/stamp and an opaque payload. ~~What still stands between
  the packages and the diff: the **command payload encoding**
  (`CommandManager`'s format — the next reading), and a recording made by
  *this* install of a fixed-seed gamelog run, so the order stream and the
  per-frame ground truth describe the same game.~~ **Both done, 2026-08-24**:
  the payload encoding is `docs/COMMANDS.md`, and the paired run is
  `docs/INPUT.md` — `gamelog-run7-ancient-nubian-orders.txt` and
  `Playback - 2026.08.24 10'15'53 (Mon).rcx`, 1,732 frames and 1,732
  packages of the same game, wired in through `crate::input` and run by
  `rondata --recgame <file> --gamelog <dump> --diff`. **What the stream
  cannot ever supply is the AI's half** — no AI class issues a command, so a
  recording replays the AI by re-simulating it (`docs/INPUT.md` §1), and
  this entry's opening claim needs that qualification: the score cannot move
  past player 0 until the AI is implemented.
- **The `role & 0x10000` word, `guy_radius`, `big_radius`**, the graft tables
  and barbarian flags, the free-tech rules, `is_list` — the defaults above.
- **Which duplicate a duplicated `CONSTANTS` tag resolves to** under by-name
  lookup (`get_item` → `XMLNode::get_element`, presumably the first). The
  shipped duplicates are equal, so the dump cannot tell.
- **`Constants::log_data`'s omissions** — seven loaded tags and the sixth
  `scholar_rate` — are the logger's; the loader is not affected.
