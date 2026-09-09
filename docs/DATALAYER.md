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
§15.2–§15.3; the same comparison runs as an install-gated `cargo test`
against run3's dump, `typesdump::tests`, since 2026-08-25). A building's
`domain` is `BuildType::set_domain` from
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

Everything borrows from the text, and **the tree is an arena** (item 235,
2026-09-06). A block is a twelve-byte cursor — a `&Log` and a node index —
and every block's fields and children live in three flat chunked vectors
that the block indexes by range; a field is four `u32` offsets into the
text rather than a pair of fat pointers. The shape it replaced was a tree
of `Vec`s, two per block, and on the 793 MB run71 capture that was 2.96 M
blocks holding 4.0 M live allocations, 1.3 GB of doubling slack and 2.3 GB
of pointer pairs: **4,668 MiB resident for 793 MiB of text**, against
**2,017 MiB** now. The block and field counts are unchanged — 2,955,663 and
76,624,335 — because the grammar is unchanged; only the storage moved.

The chunk size is fixed (65,536 entries) and that is the load-bearing part
for the *suite* rather than for one parse. macOS's allocator does not return
a freed block of this size to the system, and a differently sized block from
the next capture cannot reuse it, so a suite that parses forty captures of
forty sizes **ratchets**: it reached 15,275 MiB serialized while its largest
single test held 5,425 MiB. With one fixed chunk size every chunk a log
frees fits every chunk the next log wants — a second parse after a first is
dropped costs 10 MiB where it cost 353 — and the serialized peak is
**10,075 MiB**. ~~What is left of the ratchet is the `String` each test
reads the capture into, which is the file's own size and so a different size
every time.~~ ~~**Closed by item 260** (below): the capture is a mapping now
and so is the chunk, and both are returned by `munmap` rather than left with
the allocator.~~ **Item 280 took the mappings out again** (below): the struck
sentence is true once more, and what closes it without `unsafe` is the
never-resident capture this section ends on.

`Log::initial()` gives the start-of-game state — `GAME INFO` → `GAMEINFO` and
its `PLAYER`s, `WORLD`, `CITIES`, `CONSTANTS`, every `UNITDATA` with its
`GUY`s, every `BUILDDATA`, every `LEADERDATA` — and `Log::frame_states()` the
`FRAME n` blocks.

### The parse is lazy, and the capture was mapped (260 and 280, 2026-09-07)

Item 235 made the tree an arena and its chunks one size; what it could not
fix is that **the arena is built at all**. `Log::parse` read all 76.6 M
fields of a 793 MB capture where a test reads hundreds, and the release
suite peaked at **15,791 MiB** of the 20 GiB ceiling — the number that made
the next long capture unbookable. Two things were wrong, and only the second
is the one the item named.

**Nothing was ever given back.** A probe that parsed three captures in one
process and read its own resident set after each `drop` found it had not
fallen by a byte: 5,332 MiB held with nothing alive. Both large things a
parse holds — the `String` the capture was read into, and the arena's chunks
— go to the system allocator, and macOS keeps a freed block of that size
rather than unmapping it, so a suite that parses forty captures of forty
sizes reports the **sum of everything it ever held**. ~~Both are mappings
now (`crates/rondata/src/mapped.rs`): `Text` is the capture and `Pages<T>`
the arena's chunk; `munmap` returns them where `free` did not, and the same
probe ends at 1,647 MiB instead of 5,332. On the tree this was measured on,
that alone took the suite from 14,721 MiB to 12,182.~~

**Overturned by item 280** (`docs/DECISIONS.md` 37). The mappings are gone,
`mapped.rs` is `crates/rondata/src/capture.rs`, and the crate is back under
the workspace's `forbid(unsafe_code)` — no `allow(unsafe_code)` anywhere in
`crates/`. `capture::read` is `std::fs::read_to_string` and the arena's
chunk is a `Box<[T]>`. The mapping had been bought with a hand-rolled
`unsafe extern "C"` mmap behind a **safe** `read()` and `from_utf8_unchecked`
on top, and the hazard that keeps `memmap2::Mmap::map` an `unsafe fn` is
live in this repo rather than theoretical: `tools/gamelog/runqueue.sh`
renames `gamelog.txt` over an archive name, and the capture lane runs beside
the suite by design. On this tree the release gate went from **8,358 MiB to
9,921 MiB** of the 20 GiB ceiling, 243 tests green either way and 281 s
against 274 s — both measured on this tree, the mapping in and out.

**The successor now exists for ordinary FRAME records.**
`capture::indexed::IndexedCapture` uses safe file reads and an offset cache,
wrapping each requested span in a short-lived GAME String. Its iterator yields
one owned Frame at a time, so the borrowed `Log` API does not need to change.
The frozen-clock test now uses it: isolated peak RSS fell from 1,313.4 MB to
90.7 MB, with unchanged assertions and test-body time 3.84 s versus 4.07 s.
Complete Frame equality passed on five captures (726 frames). Shutdown tails
now also stream: all 102 archives matched full-reader closing records, and the
95-capture census fell from 6.52 GB to 260 MB peak RSS at similar time. Setup
and other whole-log consumers still use `capture::read`. Boundaries and evidence:
`docs/audit/2026-09-09-streaming-captures.md` and
`docs/audit/2026-09-09-shutdown-streaming.md`.
The earlier run58 experiment still rules out an `madvise` shortcut: macOS
accepted `MADV_DONTNEED` on the private mapping but held RSS at 1,347 MiB;
`MADV_FREE_REUSABLE` rejected non-anonymous memory with `EINVAL`.

**And the frames are indexed rather than read.** A capture's frames are all
children of `GAME`, so `Log::parse` reads eagerly up to the first `FRAME`
child — which is exactly the head `Log::initial` wants — and then walks the
rest recording, per remaining child, its name and the byte range of its own
text. An indexed block is a `Node` whose `fields_len` is `u32::MAX` and
whose `fields_at`/`kids_at` hold that range instead (the node's width is
item 235's, and 28 bytes is worth keeping); `Block::ensure` reads it in the
first time anyone asks for a field or a child, and committing it writes the
real counts over the marker. `frames()`, `dumps()` and a `name()` never ask,
so picking one frame out of five thousand reads one frame.

A span runs from its own `BEGIN` line to the next line at or above the
children's indent that opens a block, which is what makes reading it in
isolation give the same answer as reading the whole file: the trailing
fields the writers put at a block's *own* indent fall inside the span they
belong to. Two shapes the index cannot model are checked for rather than
assumed — a block opened after a field has already closed the span, and a
line shallower than the children. The second happens in **every** capture,
on the last line (`GameInfo closing`, at indent 0, which the eager rules
give to `GAME` *and* to the preamble), so the index stops there and the
eager pass resumes from that line. Offering the split again at every later
`FRAME` is the file squared: a first draft did that and got 34 tests done in
the 220 s the whole suite used to take.

`gamelog::the_lazy_tree_is_the_eager_tree` is what makes this safe. It
spells both parses of the same text out block by block and field by field
and compares the strings — over the module's sample, a fixture carrying
every quirk, and the head of run87 — and asserts the lazy parse indexed
something, so it cannot pass by doing nothing. It was made to fail twice: by
dropping the parent's share of the trailing fields, and by ending a span at
its last child.

**What the laziness costs is a second pass.** Every whole-log product —
`frame_states`, `frame_seeds`, `anim_lengths`, the height grid, and the four
walks `initial` used to make separately — now goes through `scan_children`,
which reads each of `GAME`'s children into a scratch arena cleared before
the next one. What it keeps is owned, so the memory is one frame rather than
all of them; but the walk is a **re-read** where it used to cross an arena
already built, so `initial` does its four in one pass, and a test that wants
both the initial state and the frame states parses the capture twice. The
suite is **309 s against 221 s** for **8,816 MiB against 15,791** — both
ends measured on the same tree, a detached worktree at the base's tip for
the before-number, because a benchmark against a base that has moved is
worth nothing.

Two changes of behaviour ride along, both towards the type's own
documentation: `Initial`'s regions, herds, goods and height grid are taken
from the head rather than from a whole-log depth-first search that, on a
capture with no `DUMP_ALL` head, walked every frame to answer `None` — and
where it did answer, answered with a mid-game block's state as though it
were the opening one.

On run58 (1,345 MiB, 5,201 frames) what is left is: 1,347 MiB of the
capture's own text, 1,244 MiB of `Initial`'s own `frame_bodies`/`frame_guys`, and 520 MiB
of `frame_states`' `Vec<Frame>` — 3,029 MiB at the peak against 5,206
before, and **17 MiB of it is the parser**. The two big terms are now the
caller's owned data and the file itself, which is where item 260 wanted
them.

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

### The derived words no column carries

**Established 2026-08-25, and every claim below is checked against the
program's own loaded values.** `UnitType::log_data@0061c490` prints
`unit_flags`, `unit_flags2` and `role` for each of the 364 unit types,
`BuildType::log_data` prints `build_flags` for each of the 129, and
`TechType::log_data@0066d630` prints **eleven `ai[scan]` shorts** for each of
the 85 techs — so run3's `DUMP_ALL` type dump (`docs/ORACLE.md`) is a
field-by-field oracle for the whole of this section, the same way its
`COMBATTABLE` is for `docs/COMBAT.md` §15.2. `rondata --types <dump>` checks
all four: **364/364 roles, 364/364 `unit_flags2`, 129/129 `build_flags`,
85 × 11 weights, zero differences.**

These are the loader's half of the production AI's seams (`docs/AI.md` §12.1
item 3): every one of them is read by `create_units`, `create_buildings` or
`research_techs` and none of them is a column in any file.

**1. `role` (`UnitTypeData+0x2c8`), `UnitType::determine_roles@0061c320`.**
One word from five columns, computed at the end of `UnitType::init`:

- `0x200` — citizen/scholar: the four ids `0x32..0x35` **by identity**, not by
  lineage.
- by `DOMAIN`: air (`2`) `0x1000`; land (`0`) `0x40000`, `+0x10` if
  `is(SCOUT)`, `+0xc` if `CAT` is `Mounted`; sea (`1`) `0x80000`, `+0x10` if
  `is(BARK)`.
- `0x8000` if `CARRY ≠ 0`.
- **military** — `ATTACK ≠ 0`, `CAT` neither `Command` (4) nor `Civilian` (5),
  and not already `0x200`: `0x10000`, `+0x2000` sea, `+0x4000` air; land
  `Foot` also takes `0x800` and then **either** `2` (`is(HOPLITES)`) **or**
  `0x100000` (everything else — the two are exclusive, not a pair); and
  finally `0x400` when `max_range ≠ 0`, else `1`.
- everything that fails the military test takes `0x100` instead.

The `CAT` column is `Categories::find_key(unit_cats, …)` over `rules.xml`'s
`<CATEGORIES id="unit_cats">`: 0 Foot, 1 Mounted, 2 Mech, 3 Artillery, 4
Command, 5 Civilian, 6 Sail, 7 Naval, 8 Air. `max_range` is the *stored*
`+0x1fc`, i.e. after `FLAGS k` has moved it to `second_max_range` and zeroed
it. The Citizen's `0x40300` — land, citizen, non-military — is the whole
derivation in one row.

**2. `unit_flags2` (`+0x2b8`).** Zeroed in `init`; six bits by lineage in
`UnitType::init_final_flags@0061dc70`, run from
`ObjectType::finalize_init_all@0065f4a0`:

| bit | set when |
| --- | --- |
| `1` | `is(MACHINEGUN)` or `is(FLAMETHROWER)` |
| `4` | `is(CATAPULT)`, `is(FLAMINGARROW)`, `is(MACHINEGUN)`, `is(FISHERMEN)`, `is(KATYUSHA)`, **or the exact ids** `MERCHANT`/`MERCHANTDUTCH`/`FURTRAPPER` — `needs_packing`, the sim's `Profile::packs` |
| `8` | `is(CARA)` or `is(MERCHANTFLEET)` — `is_caravan` |
| `0x10` | `is(SCOUT)` |
| `0x20` | `is(GENERAL)` |
| `0x40` | `is(SUPPLYWAGON)`, `is(BASE_GOV_HEROTYPES)`, `is(THEMONARCH)`, `is(THECITIZEN)` |

and the same function ORs `0x10` into **`unit_flags`** for `is(TRANSPORTBARGE)`
or `is(MERCHANTFLEET)`. Bit `2` — `is_spellcaster`, and the `& 6 == 2` test
of `get_stance_type` — has a different provenance and cost this reading an
hour: **`SpellType::init@00674a80` marks its own owner.** Each of
`craftrules.xml`'s 55 `CRAFT` records names a `FROM` and a `FROM2`; when
either resolves to a unit type that type gets `unit_flags2 |= 2` (when it
resolves to a *building* the building gets `build_flags |= 0x20000000`, which
is the Small City's), and then `UnitType::init_spellcasters@0061aae0` walks
self → `graft` → `from` and marks anything with a marked ancestor. 14 seeds
become exactly the 77 marked types.

The records themselves are loaded too: `Loaded::spells` is each `CRAFT`'s
`JOB_TIME` and `FLAGS`, in file order — `docs/ORDERS.md` §6.9.

**3. The five derived `build_flags` bits.** No shipped `BUILD_FLAGS` string
contains a digit — the alphabet is `abcdeg ijmn` — so **every bit above 25 is
derived**, and the first reading's conclusion that they are therefore dead
(`docs/AI.md` §2.19 item 5) is wrong in both directions:

| bit | set by | meaning |
| --- | --- | --- |
| `0x0400_0000` | `Types::init@00669cc0` — for every building with a `FROM`, on **both** the child and the parent | in an upgrade line |
| `0x0800_0000` | `TechType::set_research@0066cba0` — for every building in the lineage of any tech's `WHERE` | a tech is researched here |
| `0x1000_0000` | `finalize_init_all` — `is(FARM)`, `is(OILWELL)`, `is(OILPLATFORM)` | `is_flat` |
| `0x2000_0000` | `SpellType::init` — a craft's `FROM`/`FROM2` | a craft is cast here |
| `0x8000_0000` | `UnitType::init`'s tail — any unit with `WHERE = b` and **not** `unit_flags & 0x8000` | trains something |
| `0x4000_0000` | the same line, when that unit also has `ATTACK ≠ 0`, `!(role & 0x100)` and `!b.is(VILLAGE)` | `is_military_trainer` |

So `build_flags & 0x8000000` is **live** — Granary, Lumber Mill, Smelter,
University, Library, Temple, Senate and the whole Tower and Fort lines carry
it — which makes `create_buildings`' civic block real code rather than the
dead branch §2.19 called it; and `is_military_trainer` (`0x40000000`) is live
too, on the Barracks, Stable, Siege Factory, Dock, Airbase and Missile Silo
but **not** on the Small City, which trains militia but is excluded by the
`is(VILLAGE)` clause. `0x2000_0000` was previously unnamed.

**4. `TechType::ai[11]` (`+0x1cc`), `compute_ai_values@0066cdc0`.** Zeroed by
`TechType::init`, then one pass over `0x220..0x274` **ascending** from
`Types::init:1239`. Each tech scores itself and then counts **its
dependants** — every type whose `Type::find_preq` (vslot `+0xdc`: "is `t` one
of my `get_preq(i)`?") answers yes:

- *itself*: an epoch tech scores by `cat` — 3 → `ai[4]+1, ai[10]+1`; 1 →
  `ai[5]+1`; 2 → `ai[1]+1`; 0 → `ai[0]+1`. A non-epoch tech scores by the
  building it is researched at: Temple → `ai[5]+2`; **Fort → `ai[5]+1` *and*
  `ai[0]+1`**, the second through a fall-through the first reading missed.
- *units* (`0x32..0x191` — the twelve **gaia types are outside the loop**,
  which is what makes the twelve animals' `WHERE = Large City` harmless):
  military `ai[0]+1`; citizen `ai[1]+1, ai[9]+1`; land military `ai[8]+1`;
  otherwise `carry` → `ai[3]+1` and sea → `ai[2]+1` / **air → `ai[6]+1`**
  (`ai[6]` is air alone, not "sea and air").
- *buildings*: `s = 2` when the building has no `FROM`, else 1. `is(VILLAGE)`
  → `ai[1]+1`, and the Large City exactly → `ai[5]+1, ai[1]+1` and
  `add_preq_ai(1, 1, 0)`. Rootless only: gather → `ai[1]+2s, ai[9]+2s`;
  `0x8000000` → `ai[1]+2s, ai[4]+2s` (+`ai[9]+2s` for the four enhancers);
  `is(DOCK)` → `ai[2]+1` and `add_preq_ai(2, 1, −1)`. Every dock, rootless or
  not → `ai[2]+1` and `add_preq_ai(2, 1, 2)`. Then the **fundamental** type
  (`from` walked to the root) decides one of four arms: gather → `ai[1]+s,
  ai[9]+s`; `0x8000000` → `ai[1]+s, ai[4]+s`; Tower/Fort/Airbase or
  `ATTACK ≠ 0` → `ai[0]+s`; one of the four enhancers → `ai[1]+8, ai[9]+8`.
  Finally, when the fundamental trains anything (`0x80000000`), every unit
  whose `WHERE` is **this** building scores `ai[1]+1` if
  `role & 0x180c0 == 0`, else `ai[0]+s`.
- *techs*: `ai[4]+1`, and `+1`/`+1` to `ai[4]`/`ai[0]` for an epoch dependant,
  `+2`/`+2` for an age.
- *goods*: `s = 4` for the first six, else 1 — `ai[9]+s, ai[1]+s`.
- *spells* (`0x275..0x2ab`, `craftrules.xml`): `ai[0]+4, ai[1]+2`.
- *bonuses* (`0x2ac..0x325`, `rules.xml`'s 122 `TECHBONUSES`, one `preq0`
  each): `ai[1]+1` (twice for the first), `ai[4]+2`.

`add_preq_ai(i, n, d)` adds `n` to `ai[i]` of **the computed tech's own
prerequisites** and recurses `d` levels — `0` none, `2` three levels, `−1`
until the chain ends. `ai[7]` is never written by anything.

**What this has not established.** Three things, none of them observable in
the shipped data:

- **The lobby the weights are computed under.** `find_preq` goes through
  `get_preq`, which remaps an epoch prerequisite when the game does not run
  Ancient-to-Information; the loader computes the array under
  `Setup::STANDARD`, which is what run3's lobby was. A short game would give
  a different array, and nothing here checks that.
- **`unit_flags2 & 2`, the caster bit, in a *modded* install.** The seed is
  `craftrules.xml`'s `FROM`/`FROM2` resolved by name, and a craft naming a
  type that resolves to neither a unit nor a building is only a warning.
- **Whether the name-group rule reaches the building and technology
  tables.** Their loops in `Types::init` run two passes with no `NAME`
  comparison, so it should not — but the *only* evidence is the loop's
  shape, since no shipped building or tech group disagrees with itself.

### The name group: a unit record does not always get its own columns

**Found 2026-08-25, chasing the one unit whose `unit_flags` would not
reproduce.** `Types::init@00669cc0` loads the *unit* table in **five passes**,
and the element it hands a record is not always that record's:

```
for pass in 0..5:
    for i in 0x32..0x192:
        if pass == 0 or pass == 1 or name[i] != name[leader]:
            leader, element = i, own_element(i)
        types[i].init(element, i, pass)
```

So a **run of consecutive records with the same `NAME`** — which is how the
file spells a nation's art variant — shares one element from pass 2 on, and
pass 2 is where all but eight columns are read. Passes 0 and 1 are the
record's own: `NAME`, `GRAPH`, `TYPENAME`; `WHERE`, `FROM`, `JUMP`,
`TRIBE_MASK`, `GRAFT`.

Four shipped rows say something their program never reads:

| record | column | file | loaded |
| --- | --- | --- | --- |
| 7, the German General | `FLAGS` | `lmhc` | `lmhcb` |
| 51, Riflemen | `ARMOR` | 1 | **3** |
| 88, Anti-tank Rifle | `LOS` | 12 | 11 |
| 90, Bazooka | `LOS` | 14 | 13 |
| 222, Howitzer | `SPLASH_PERCENT` | 33 | **25** |

and the dump settles it three ways over: reading each record's own `ARMOR`,
`LOS` and `SPLASH_PERCENT` disagrees with the program on 1, 2 and 1 units,
and reading the group leader's disagrees on none. `rondata --types` now
checks `armor` and `splash_percent` (the two the simulation carries)
alongside the derived words, so the rule has a guard that fails the moment it
is dropped.

Sixty-four of the 364 unit records take a leader other than themselves. The
building and technology tables get two passes with no such comparison, so
this is the unit table's rule alone.

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
`Profile::guy_radius` and `big_radius`; ~~`Profile::combat_role` (`role &
0x10000` — the `role` word's source is unread; the loader uses `ATTACK ≠ 0`
and not `OBJ_MASK C`)~~ — **closed 2026-08-25**: the word is derived above and
`combat_role` is `role & 0x10000` exactly, which moves two of the 364 (the
Armed Supply Wagon, whose `Civilian` *category* refuses it the bit its
`ATTACK` would earn, and Boadicea, which carries the `CIVILIAN` mask and is
military anyway); `Tribe::graft` (identity) and `Tribe::barbarian`
(false); `TechTree::free_rules` (empty — the nation and wonder free-tech
blocks); `TypeDef::is_list` and `leader_off`; `balance::Kind::age` is the
`get_age_slow` reading (first tech prerequisite's age — ~~−1 → 0~~ **an age
tech's `AGE` column plus one, any other tech's `AGE`, no tech 0**; the first
draft omitted the +1 and the dump's per-type `age` caught it,
`docs/COMBAT.md` §15.2 — checked equal for all 364 units). The
production group's identity for "factory units" (whether the Auto Plant,
Factory and Siege Factory share one count) is taken as the `WHERE` building
itself.

### A building's blocked tiles are art, not rules (2026-08-27)

**No column of `buildingrules.xml` says which of a footprint's tiles a
building blocks.** `BuildType::mask_me@006312a0` marks the object field
(`T |= 3`) over the whole rectangle and then consults a **per-tile
template**, `BuildType::mask`, one byte per tile: 1 calls
`World::set_blocked_at(1)` (`T |= 0x4000`, `0x2000` on the neighbours), and
anything else calls `set_blocked_at(0)` and **clears** the bit. So a
footprint's object field and its blocked bits are different shapes, and
`docs/CITIES.md` §3.6 says so.

The template comes from two files outside the rules tables:

- **`masks.txt`, at the install root** — 36 named grids, each `#<name>`,
  then `x, y`, then `y` rows of `x` comma-separated ints.
  `BuildType::init_build_mask@006310b0` reads exactly `x × y` items and
  stops, so the **second grid** several sections carry after a blank line
  (the `gather` family) is never read. `;` starts a comment; a non-numeric
  item is 0, which is what the `doobers` grids (`up, right, left, down`)
  come to.
- **`Data/building_graphics.xml`** — every `<BUILD name="GRAPH-TRIBE-AGEn"
  … mask="…">` names one of those grids, and the Farm is written as its own
  `<FARM><DEFAULT><AGE0 mask="…">` tag instead. No shipped graphic gives two
  masks to one `GRAPH`, so the age and the nation drop out: the map is a
  function of the rules row's `GRAPH` alone, and all 129 building types
  resolve.

Three consequences, and the first is the one that cost a divergence:

- **A Woodcutter's Camp blocks nothing at all** (`2x2 gather`, four zeros),
  and neither does a Mine's neighbour on the same footprint size — the Mine
  is `2x2 solid` and blocks all four. Gatherers stand *on* their camp: a
  returning citizen's `find_nearby_spot` refuses a `0x4000` tile
  (`docs/ORDERS.md` §10) and the camp's own tiles do not carry one.
- **An `extra space` mask leaves the last row and column free**, so a city's
  7×7 footprint blocks 6×6, a Granary's 5×5 blocks 4×4, and so on down.
- **`4x4 oil`** is a saltire — the four corners and the middle 2×2 — which is
  the only non-rectangular template shipped.

*How it is established.* The two files, the two decompiled loaders, and a
**differential check against the original's own map**: a `WORLD ≥ 6` or
`DUMP_ALL` start dump prints all 57,600 `tdata[scan].mask` words, the
harness loads them and then re-marks every building through `mask_me`, and
`rondata::diff`'s `every_footprint_takes_the_blocked_bits_the_original_s_map_shows`
compares the object and blocked bits of every tile on three captures. With
this crate's old rule — block every non-flat footprint whole — run10 alone
disagrees on 59 tiles.

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

**The `WORLD` block is borrowed the same way, and it has to be** (2026-08-27).
`WorldData::log_data` writes the cells and the per-tile masks only at
`WORLD ≥ 5`; a capture taken for some other category writes the block's
seventeen scalars and stops. The harness then stands up a flat, region-less,
treeless world — and every number measured on it is measured against a map
the game never had. run6 was read that way for three weeks: it and run10 are
the *same game*, and their diffs disagreed, run6 putting player 1's first
divergence at frame 2 and run10 at frame 4. The gate on the borrow is those
seventeen scalars themselves — the map seed, the extent, the generator's
eight totals and the territory limits — so a sibling whose block opens with
the same values field for field generated the same map and the rest of its
block is ours. With it the two captures agree unit for unit.

The lesson generalises past this field: **a dump that carries less is not a
dump that says less — it is a dump the harness quietly fills in**, and what
it fills in is not marked as a guess anywhere the score can see.

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
dump wins where it speaks. `gather_from` is an input: the original fills it
from the terrain, which only a `WORLD ≥ 5` dump carries — and which the
worker's tile choice then filters again on `has_gather_access`
(`docs/ORDERS.md` §6.4), so the list alone is not enough.

**The score, on `gamelog-run6`: ticks before divergence 99** (2026-08-27),
and it is worth being precise about why, because the single number hides the
state of the port. `Report::first_divergence_by_unit` gives the breakdown;
over all 432 frames it is `0/3@103 0/4@103 0/5@103 1/0@363 1/1@171 1/3@219
1/4@203 1/5@219 1/6@100 1/7@206 1/8@320` — **and both woodcutters' citizens,
`0/1` and `0/2`, never disagree at all: 432 frames of matching positions and
matching order lists.** The score is a minimum over every unit, so it is
pinned by whichever unit the simulation cannot yet drive:

- The **farm citizens** (`0/3`, `0/4`, `0/5`) hold to frame
  **102**, where the original inserts a move in front of their gather order
  ~~and ours does not — the farm re-target, and the measurement that says
  `FARM_GROWS = 200` is wrong by about a factor of two (`docs/ORDERS.md`
  §6.5)~~ — and, since 2026-08-24, so does ours: the clock's other half was
  `Farms::inc_time` (`docs/SYNC.md` §3.3, `farms.rs`). The re-target's
  *tile* is two sync-stream draws, so past the traced frames the farmers
  part on the tile, not the frame. Before this run the farmers "agreed"
  only because both sides stood still.
- The **AI's units** part one at a time from frame 100 on — `1/6` the
  trained citizen first, then the scout at 363 — where the AI orders them
  somewhere this simulation's AI does not.

The whole capture is the same game as `gamelog-run10`, which runs six times
longer; the two now report the same score and the same per-unit breakdown,
and that agreement is itself the check on the borrow above.

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

## 4. The widening ledger — what nothing compares (2026-09-03)

`crates/rondata/src/ledger.rs`, and it is a guard rather than a report: it
reads the parser's own source and the differ's, and counts the fields the
one fills that the other never names.

The rule it serves is "when the original dumps a record, diff the whole
record", which has closed or sharpened twelve queue items on its own. The
twelfth is why the ledger exists. `UnitDump::orders_x` had been parsed since
the `UNITDATA` reader was written; the merchant window that needed it
compared positions and clocks, so a merchant walking at the **wrong rare**
read as a merchant one frame late, and the item was booked as a timing bug
for a day (`docs/MERCHANT.md` §2.2.1).

Two numbers, both pinned and both allowed only to fall:

| list | meaning | 2026-09-04 |
| --- | --- | --- |
| **uncompared** | the field's identifier appears nowhere in `diff.rs` | **23** of 255 |
| **single-capture** | exactly one test function names it | **41** |

The second is the sharper one, and it is item 87's "per capture" half. A
field that only one window can see is one capture away from an unnoticed
divergence — `dest_angle`, `collide_frame`, `myhits`, `damage` and
`inside_up` are all in that state today, and so is every `CityData` stamp.

**And a field on neither list can still be uncompared, which is the
ledger's own blind spot.** It scans identifiers, so a `GuyData` field the
parser did not have at all is invisible to it: `last_speed` and `avg_speed`
were not parsed until item 210, and they are the walk slot's own input —
`Guy::set_anim` slogs below six tenths of the type's base and jogs above
eleven, and the slot is what the arrival draw tests. They are compared on
three captures now (run64, run67, run73), and the first run of the widened
comparison failed, on `des_angle`.

**It is a lower bound with no false alarms.** A field counts as "named" if
its identifier appears as `.field` or as a `row("…")` label anywhere in
`diff.rs` — an appearance is not proof of a comparison, only that somebody
touched it. So everything on the list is certainly uncompared and some
fields off it may be too. For a queue that is the useful direction: it never
sends a session chasing a field that is already pinned. Four parser
containers — `Block`, `Log`, `Initial`, `ConstantDump` — are excluded by
name, because they hold the reader's own state rather than anything the
original writes.

Read it with `cargo test -p rondata the_widening_ledger -- --nocapture`; the
per-record breakdown is the output, and it is the queue of cheap widenings.

### 4.1 The other axis: the records nothing *reads* (2026-09-07)

The ledger above counts **fields**: what the parser fills and the differ
never names. There is a second axis, and it cost twelve months of a free
oracle before anyone swept it — **records the reader cannot reach at all**.

`GameLog::end_game` writes a whole-map state on the way out of every game
that is quit. On 65 of this machine's 95 archives it lands as a *sibling* of
the last `FRAME n` block rather than a child, so the frame walk cannot see it
and `Log::dumps` catches it only under a `DUMP_ALL`'s `FULL DUMP`.
`Log::final_state` is the reader, and until 2026-09-07 exactly one capture
used it. Fourteen of those states are now diffed whole and the census is a
test (`docs/ORACLE.md`, "The shutdown dump"; `crates/rondata/src/diff/shutdown.rs`).

The lesson generalises past this one record: **the field ledger cannot see a
record the parser never visits**, the same way it cannot see a field the
parser never parses (its own blind spot, above). Both are found by walking
what the original writes, not by walking what this crate reads.

### 4.2 The third axis: the field is compared, at the wrong width (2026-09-07)

A field can be on neither list — parsed, named, compared on every frame of
every capture — and still be wrong, because this crate holds it in a type
the original does not. Both lists count *names*, and a name says nothing
about a width.

`CityData::free` is the case that found the axis (item 265). It is a `uchar`
at `+0x5a` whose two writers are a bare byte `±1` with no clamp, so the
original's counter wraps 0 → 255 where this crate's `i32` went to −1
(`docs/CITIES.md` §5.8). It had been compared on run58's 5,201 frames since
item 154 and agreed on every one of them, because no game on disk had ever
decremented it at zero until run79 did on frame 7183.

**The booked row.** Ten fields of the same record are `uchar` in the type
record and `i32` in `ai::CityAi` — `busy`, `gatherers`, `ocean`, `land`,
`filled`, `bordering`, `ocean_filled`, `dock_tile`, `space[3]`, `ter[6]` —
`pop` at `+0x5d` is an eleventh on `sim::City`, and `gatherers` has a bare
byte decrement of its own
(`Leader::produce_building@006e1400:1135`). None has been seen to wrap on
any capture on disk, so none was fixed on the strength of `free`'s row; the
falsifier for each is a capture where the sweep's count and the producers'
decrements cross zero. The same question is open one record up, for every
`char`/`uchar`/`short` of `LeaderData` this crate holds as an `i32`.

This axis has no guard. A ledger for it would have to read the PDB's type
record beside the sim's struct — `types.txt` names every width — and that is
a tool, not a grep.

### 4.3 The fourth axis: the field is a word, and one bit of it is read (2026-09-07)

The ledger scans the differ's source for the field's *name*, so a bitfield
counts as compared the moment anything mentions it — however few of its bits
that anything actually looks at.

`UnitData::unit_masks` is the case (item 267). The harness has named it
since the packed bit (`0x80000`) got a row, and named it again for the
verified line (`0x8`) inside one window's test, so it has never been on
either list. It carries at least eight bits this crate models — `1` the
placement ghost, `8` the verified line, `0x40` the collision wait, `0x400`
the builder, `0x800` the tribe-bonus road, `0x80000` packed, `0x100000` the
soft one-shot, `0x800000` the dock walker — and until item 267 exactly one
of them was compared on every capture.

The one that was missing paid for itself on its first run: `0x100000` is
§4.3-of-COLLISION's soft half-step, printed since the first `UNITDATA` ever
captured, and comparing it dated Great Lakes' run-up divergence a frame
earlier than the position diff could and named the mechanism outright
(`docs/COLLISION.md` §8.4).

**The booked row is the sweep**: `unit_masks`, `unit_masks2`, `guy_flags`,
`node_flags`, `city_flags`, `leader_flags`, `leader_flags2`, `build_flags`
and `role` are all words the dumps print whole and the harness reads a bit
or two of. The ledger cannot see this any more than it can see a width, and
for the same reason — it counts names. What would see it is a per-*bit*
census: for each dumped mask, which bits this crate models at all, and which
of those the differ compares. That is the same shape of tool §4.2 asks for
and could be the same tool.

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
