# The tech tree: what may be bought, and what owning it changes

`docs/COSTS.md` is what a thing is worth and `docs/PRODUCTION.md` is how it is
paid for over time. Both assume an answer to the question that comes before
either: **may this player buy this thing at all?** This document is that
answer, and the state it is asked of.

In Rise of Nations every purchasable thing — a unit, a building, a wonder, a
government, a library technology, an age — is one entry in a single table of
806 types, and every entry carries up to three prerequisites that are
themselves entries in the same table. A player's progress is one bit per
entry. "Ages" and "library levels" are not separate systems; they are rows of
that table with a few extra rules about who may take them when. The whole tree
is four predicates over those bits, one procedure that sets a bit and cascades,
and a small amount of counting.

Not in it: what a technology *does* once owned — the economy caps, the
attrition resistances, the territory bonuses — each of which is read where it
applies (`docs/ECONOMY.md`, `docs/ATTRITION.md`, `docs/SUPPLY.md`). Not in it
either: the AI's choice of what to research (`Leader::research_techs`,
`Leader::produce_tech`), which is phase 5.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied, through the export under
`~/ghidra-projects/decomp` (`tools/ghidra/`). The `TypeIndex` enum the PDB
carries was dumped in full (`tools/ghidra/scripts/DumpEnumAll.java`), so every
range test below is named rather than inferred. Two virtual calls whose
argument Ghidra dropped were settled from the raw bytes with `llvm-objdump`.
Nothing is transcribed; see `docs/DECISIONS.md` entry 7.

**Confidence.** High for the type space, the data layout, the per-player
state, `has_tech`, `has_preq`'s loop and age quota, `type_eligible`,
`type_avail`, `queue_here`, and `gain_tech`'s bookkeeping and cascades — each
read end to end at the consumer. High for the lobby remaps in `get_preq`,
which are arithmetic read line by line, **but unverified behaviourally**: no
shortened game has been played to confirm the remapped levels. Medium for the
nation-power exceptions: they are enumerated, but there are forty of them and
each was read once. Everything open is listed at the end.

A blind second reading (`docs/audit/2026-08-20-tech.md`) doubly confirmed
the type space, the loaders' columns, every predicate branch by branch,
`gain_tech`'s order and cascades, `lose_tech`, the starting position and the
lobby encodings — and overturned what the first reading had **not** read: the
loaders' derived fields (a combat unit's implicit Military requirement, the
`upgrade`/`to`/`where` back-links), `lose_tech` leaving building bits alone,
`gov` being written, and which check is the Tech Race win. Those are landed
below and marked "second reading" where they changed what an earlier draft
said.

**Where the implementation is.** `crates/sim/src/tech.rs`: the tree as data
(`TechTree`, `TypeDef`, `Kind`), one `PlayerTech` per player, and the
predicates and procedures below under their original names. The rules that
name a particular type do so through `Roles` (`docs/DECISIONS.md` entry 18).
What it leaves out, each said again at the point it applies: the Senate's
capital walk and the Town's "no city yet" count (no cities in the simulation;
the first city is taken as always eligible), the conversion of standing units
and queued entries when a unit type arrives (reported as a `Gained::UnitUpgrade`
event for the caller), the resource unlocks and fog flags a tech carries
(other mechanics'), and the per-block unit ranges of the nation free-upgrade
rules (the common shape is a `FreeRule`; the ranges are data a loader fills).
The structural claims about the shipped tables — how many techs, how the lines
are laid out, what the prerequisite vocabulary is, which constants gate the
exceptions — are re-derived from the user's own install by
`cargo run -p rondata -- <install>`, which fails if any has drifted.

---

## The type space

Every type the game can name is a `TypeIndex`, one `int` in a single range.
The engine keeps one `Type *` array (`types.list`) over the whole range and a
per-class array (`unittypes`, `buildtypes`, `techtypes`, `goodtypes`,
`bonustypes`, `spelltypes`) that shares the indices. The class of a type is a
range test, and the engine asks it through virtual predicates —
`is_unit_type`, `is_build_type`, `is_tech_type`, `is_age_type`,
`is_epoch_type`, `is_final_type`, `is_gov_type`, `is_plain_tech_type`,
`is_bonus_type` — whose bodies are those range tests and nothing else.

| range | count | what | enum base |
| --- | --- | --- | --- |
| `0x000–0x031` | 50 | goods: six basic (`FOOD TIMBER WEALTH KNOWLEDGE METAL OIL`), then rares | `BASE_GOODTYPES` |
| `0x032–0x191` | 352 | units; `0x160–0x165` are the six government patriots | `BASE_UNITTYPES`, `BASE_GOV_HEROTYPES` |
| `0x192–0x19d` | 12 | gaia | `BASE_GAIATYPES` |
| `0x19e–0x21e` | 129 | buildings; `0x20e–0x21e` are the seventeen wonders | `BASE_BUILDTYPES`, `BASE_WONDERTYPES` |
| `0x21f` | 1 | `GOODY` | `BASE_ITEMTYPES` |
| `0x220–0x274` | 85 | **technologies**, below | `BASE_TECHTYPES` |
| `0x275–0x2ab` | 55 | spells (the patriot and scenario powers) | `BASE_SPELLTYPES` |
| `0x2ac–0x325` | 122 | bonus types, `TECHBONUSES` in `rules.xml` | `BASE_BONUSTYPES` |
| `0x326` | | `NUM_TYPES` | |

Three negative values are part of the vocabulary: `TYPE_NONE = -1`,
`TYPE_ANY_BUILDING = -2` (also named `TYPE_NOT_AVAIL`), and
`TYPE_ANY_UNIT = -3` (`TYPE_NOT_SPECIFIED`).

### The 85 technologies

`0x220–0x274` is laid out in blocks, and several rules are arithmetic on the
layout:

| range | what | notes |
| --- | --- | --- |
| `0x220–0x226` | the seven **ages**, `CLASSICAL_AGE … INFORMATION_AGE` | `techtype.age` is 0–6; `BASE_AGETYPES` |
| `0x227–0x242` | the 28 library **epochs**, four lines of seven | Science `0x227` (`WRITTEN_WORD`…), Commerce `0x22e` (`BARTER`…), Civic `0x235` (`CITY_STATE`…), Military `0x23c` (`THE_ART_OF_WAR`…`SELECTIVE_SERVICE`) |
| `0x243–0x246` | the four **finals**: `MISSILE_SHIELD GLOBAL_GOVERNMENT VIRTUAL_REALITY NANOTECHNOLOGY` | `BASE_FINALTYPES`; `is_final_type` |
| `0x247–0x26e` | forty **building techs**: `TACTICS … STEEL` | the fort, temple, tower, granary, lumber mill, smelter and university lines |
| `0x26f–0x274` | six **governments** in three tiers of two: `GOV_EMPIRE GOV_REPUBLIC`, `GOV_MONARCHY GOV_DEMOCRACY`, `GOV_SOCIALISM GOV_CAPITALISM` | `BASE_GOVTYPES`; `is_gov_type` |

(`GOV_EMPIRE` is the enum's name for Despotism — the display names are loaded
from data.) `is_plain_tech_type` is "a tech that is neither an age nor an
epoch": the finals, the building techs and the governments.

**Each epoch knows its line.** `TechTypeData +0x14` (`cat`) is set by
`TechType::set_research` from the index: `(i - 0x227) / 7` is `0, 1, 2, 3` for
the Science, Commerce, Civic and Military rows, and the stored `cat` is
`3, 2, 1, 0` respectively; every non-epoch tech gets `cat = 3`. The four
counters `epoch[0..3]` are indexed by this `cat`, so **`epoch[0]` is the
Military level, `epoch[1]` Civic, `epoch[2]` Commerce, `epoch[3]` Science** —
the order `docs/COSTS.md` and `docs/ECONOMY.md` already rely on.
`LeaderData::get_epoch_base(cat)` is the inverse: `1 → 0x235`, `2 → 0x22e`,
`3 → 0x227`, anything else `→ 0x23c`.

---

## The data

### The shared record

`TypeData` (0xec bytes) is the base of every type and carries everything the
tree reads:

```
+0x04  TypeIndex     type
+0x10  int           tribe_mask     24 bits, one per nation, from TRIBE_MASK
+0x14  int           cat            the library line, above
+0x30  TypeIndex[3]  preq           PREQ0 PREQ1 PREQ2
+0x3c  TypeIndex     from           the type this one upgrades from
+0x40  TypeIndex     where          the building type that makes it
+0x44  TypeIndex     upgrade        (buildings) what this building becomes
+0x48  TypeIndex     jump           (units) the next upgrade in the line
+0x4c  TypeIndex     obs            the tech that makes this type obsolete
+0x50  TypeIndex[2]  show           display gating only
```

and `num_preq()` is virtual: **2 for a unit or a good, 3 for a building or a
tech** (the base `TypeData` answers 1) — the unit's third slot exists but is
never read.

Which column feeds which field, read at each class's `init`:

| class | file | `preq` | `from` | `where` | `jump`/`upgrade` | `obs` |
| --- | --- | --- | --- | --- | --- | --- |
| tech | `techrules.xml` | `PREQ0 PREQ1 PREQ2` | — | `WHERE` (`set_research`) | — | — |
| unit | `unitrules.xml` | `PREQ0 PREQ1` (+ the implicit slot below) | `FROM` | `WHERE` | `JUMP → jump`; `upgrade` derived | fixed `-2` |
| building | `buildingrules.xml` | `PREQ0 PREQ1 PREQ2` | `FROM` | derived: `from`, or nothing at the base of a line | `JUMP → jump`, copied to `upgrade` if ≥ 0; `to`/`upgrade` derived | `OBSOLETE` |
| good | `resourcerules.xml` | `PREQ0 PREQ1` | — | — | — | `OBS` |

**Four things the loaders derive after reading the columns** (second
reading; `crates/sim/src/tech.rs` does them in `TechTree::finalize`):

- **A combat unit that names only an age also needs that age's Military
  epoch.** `UnitType::init`: if `preq[0]` is an age, `preq[1]` is `none` and
  `attack ≠ 0`, then `preq[1] = preq[0] + 0x1c` — Classical `0x220` becomes
  `THE_ART_OF_WAR 0x23c`, Medieval `MERCENARIES`, and so on. The shipped file
  writes `none` in most military units' `PREQ1` and relies on this; the
  Phalanx needs Classical **and** The Art of War.
- **A unit's `upgrade` is a back-link.** A unit with a `FROM`, unless it is a
  hero (`OBJ_MASK 1`), writes itself into `U[from].upgrade` — when it has no
  graft, or its predecessor has one, or it is free (`h`); a grafted unit
  inherits its graft's `upgrade` if it has none. `current_upgrade` falls
  back through it.
- **A building's `to` and `upgrade`, and its `where`.** `BuildType::init`: a
  building with a `FROM`, unless a hero, is its predecessor's `to`, and its
  `upgrade` too when it carries `build_flags & 4` (the `BUILD_FLAGS` letter
  `c`); and a building's `where` is its `FROM` unless it is the base of its
  line (`basic_type() == self`), when it is nothing. So a Keep is queued at a
  Stockade.
- **`Types::finalize_grafting`** rewrites the `tribe_mask` of every unique
  unit's graft: the graft loses the unique unit's nations (or gains them
  under "No Unique Units"), and barbarian tribes always get the graft. Not
  modelled in the simulation — it needs the roster's barbarian flags and the
  lobby flag at load time.

Every prerequisite column is resolved by `Types::tech_key`, which compares the
text — case-insensitively; the shipped file writes both `none` and `None`, and
one `Information age` — against the `type_name` of each of the 85 techs and
otherwise knows two words: **`none` → `TYPE_NONE` (−1)**, **`disable` →
`TYPE_ANY_BUILDING` (−2)**; anything else is an error that resolves to `−3`. So a prerequisite is a
tech, nothing, or *impossible* — and the three behave differently, below.
`FROM`/`JUMP` go through `unit_key`/`build_key` with the same `none` word.
Unit `UPGRADE` is empty in the shipped file; the unit chain is `JUMP`.

**`type_name` is the `TYPENAME` column** (2026-08-20, `docs/DATALAYER.md`):
`TypeData + 0xb0`, which `UnitType::init` fills from `TYPENAME` and falls
back to the `NAME` when that is empty, and which `unit_key`/`build_key`/
`tech_key` search. Techs have no `TYPENAME`, so for them it is the name; units
and buildings differ in 40-odd records, and three shipped keys — `FROM
Marines`, `JUMP Arquebus Immortal`, `JUMP ECONQUISTADOR` — resolve only
through it. The back-links `Types::finalize` derives (`U[from].upgrade`,
`B[from].to`) are written unconditionally in record order, so the last
record naming a `FROM` wins; `docs/CITIES.md` has the building case.

`TechTypeData` adds `+0x1c8 age` (the `AGE` column, 0–7) and
`+0x1e2 leader_off`, an eight-bit mask of players for whom a scenario has
disabled the tech (`TechType::set_disabled`; zero at load).

### Unit flags the tree reads

`UnitTypeData +0x2b4 unit_flags` is built from the `FLAGS` column one letter at
a time: a lowercase letter `c` sets bit `c - 'a'`, a digit `d` sets bit
`d - '0' + 26`. Three bits matter here:

| bit | letter | who has it | what it means to the tree |
| --- | --- | --- | --- |
| `0x80` | `h` | citizens, scholars, caravans, scouts, and the **first unit of every military line** (Hoplites, Slingers, Bowmen, Light Horse, Catapult…) | **free with its prerequisite**: granted by the cascade the moment the tech arrives, never a research job |
| `0x200` | `j` | every researched upgrade (Phalanx, Pikemen, Arquebusiers…) | **in a jump chain**: eligibility skips the predecessor check and uses the chain rule instead |
| `0x1000000` | `y` | the nations' unique units | refused under "No Unique Units" or to a barbarian tribe |

and one `OBJ_MASK` bit: digit `1` → `0x4000000`, which every patriot and
scenario hero carries. `ObjectTypeData::has_objmask(0x4000000)` is what two
places in `gain_tech` test to keep heroes out of the unit cascades; Ghidra
drops the argument, and it was read from the bytes (`push 0x4000000` at
`0x6ddd9c` and `0x6ded3a`).

### `is`: lineage

`ObjectTypeData::is(x, strict)` is not equality. Non-strict: `type == x`, or `x`
is in the type's `is_list`, or — failing both — `ObjectTypeData::is_slow`:
`graft == x`, or recurse into `from`. (`is_list` and `is_strict_list` are
filled by `ObjectType::init_is_list` by evaluating `is_slow` over every type
once: they are a cache of the walk, not a second source.) So `is(X, 0)` reads "this type is X or
descends from X along the upgrade chain, or is X's nation variant". Strict
`is(x, 1)` uses `is_strict_list` and, for units, only `graft == x` and `x` not
unique. This is the predicate behind `queue_here`, the machine-gun rule, the
Bantu exception and the city-upgrade check, and it is why a Barracks can train
a Phalanx: the Phalanx's `WHERE` is Barracks, and a Fort, being `FROM` a
Stockade… is not.

### Nation variants: `tribe_can_type` and `get_graft`

`LeaderData::tribe_can_type(t)` is `4` if the player's tribe bit is set in
`t`'s `tribe_mask`, else `0` — with one early `0`: a unit with `unit_flags &
0x1000000` when the lobby's "No Unique Units" flag (`info.flags & 8`) is on or
the player's tribe is a barbarian tribe. `LeaderData::get_graft(t)` is `t` when
`tribe_can_type(t)`, otherwise `UnitTypeData::type_graft(who)`: the tribe's
`graft[352]` table, one replacement per unit type, which is how Hoplites become
Hypaspists for the Greeks. Every walk along `from`/`jump` in this document
grafts each step.

---

## Per-player state

`LeaderData` keeps:

```
+0x6c0c  BitMask<806>  tech           one bit per type: owned
+0x6c80  BitMask<806>  tech_at_start  the bits set by Leader::init, never cleared
+0x6cf4  BitMask<806>  obs_flags      one bit per type: obsolete for this player
```

(`+0x6c18` is `tech`'s data pointer, which is how earlier documents named the
bit.) And in `LeaderDataEncrypt`, the XOR-masked block every counter lives in:

```
+0xdc  int     ages        how many of the seven ages are owned      (^ 0x62766)
+0xe0  int     epochs      how many of the 28 epochs are owned       (^ 0x69587)
+0xe4  int     discovered  everything else gained through gain_tech  (^ 0x13985)
+0xe8  int[4]  epoch       per line, indexed by cat                  (^ 0x63187)
```

`LeaderData::get_age()` is `ages`; `epochs_get()` is `epochs`; `get_epoch(cat)`
is `epoch[cat]`; `get_highest_epoch`/`get_lowest_epoch` are the max and min of
the four. **`epoch[cat]` is not a count of owned bits.**
`LeaderData::compute_epoch(cat)` walks the line from its base and stops at the
first epoch **not** owned, so it is the *contiguous* level from the bottom; it
is what `lose_tech` writes back, while `gain_tech` simply increments. The two
agree as long as the line is only ever gained in order, which the
prerequisites enforce.

Also on the leader: `age_stamp[7]` at `+0x8f8`, the frame each age was gained
on; `gov_hero_frame` at `+0xa48`, set to 1 when a government is gained on
frame 0; and three bits of `leader_flags` (`+0x0`) — `0x1000`, `0x800`,
`0x2000` — that `fix_tech_flags` derives from owning the first prerequisite of
the `EXPLORE_MAP_BONUS`, `REVEAL_ENEMY_BONUS` (or the Space Program wonder) and
`ACTIVE_PING_BONUS` bonus types, and `ally_mask` at `+0x6929` from `ALLY_LOS`.
These are read by fog and diplomacy, not by the tree.

---

## `has_tech`: what "owned" means

`LeaderData::has_tech(t)`:

- `TYPE_NONE` → **1**. A missing prerequisite is satisfied.
- `TYPE_ANY_BUILDING` (−2, the `disable` word) → **0**. An impossible
  prerequisite is never satisfied.
- a good (`t < 0x32`) → **1**.
- a **unit**: `has_preq(t)` **and** the bit.
- a **building**: `has_preq(t)` — **the bit is not consulted.** A building type
  is "had" the moment its prerequisites are met.
- anything else (a tech): the bit.

`LeaderData::has_upgrade(t)` is the bare bit, and `get_govs_taken()` counts
owned bits over `0x26f–0x274`.

---

## `get_preq`: the three slots, with substitutions

`TypeData::get_preq(this, i, who)` is how every reader asks for a
prerequisite, and it is not a field read. Slot 0 is raw. Slots 1 and 2 are
rewritten for the lobby's starting and ending ages and for one nation power:

**Slot 1.** `p = preq[1]`, then:

1. *Iroquois governments early.* If `p` is an age, this type is a government,
   `who ≥ 0` holds nation power 18 (Iroquois) and `IROQUOIS_GOVS_EARLY` is
   non-zero: `p` becomes the previous age, or `TYPE_NONE` if there is none.
   The shipped constant is `0`, so this is off.
2. *Start age.* `start` is `starting_technology`, or in Barbarians at the
   Gates the smaller of the two starting settings. If `p ≥ 0` and either
   `start ≠ 0` or `ending_technology < 7`:
   - if `p` is an epoch, this type is a **unit**, and `p`'s line is Military:
     with `age = p.age`, if `age < start` → `TYPE_NONE`; otherwise
     `n = (28 / (ending − start + 1)) · (age − start + 1) + 3`, and the result
     is `0x23b + n/4` (signed division), clamped into `[0x23c, 0x242]` — the
     Military level a unit needs is **rescaled onto the ages actually in
     play**. This branch returns in every case.
   - else if `start ≠ 0` and `p` is an epoch: with `age = p.age`, if
     `start ≤ age` and `7 − start > 0`, the level is rescaled the same way
     onto that line, `idx = ((age − start + 1) · 7) / (7 − start) − 1 + base`,
     and `p = min(idx, preq[1])`; then if `preq[2]` is an epoch of the same
     line above `p`, fall through — otherwise return `p`. Any other path of
     this branch returns `TYPE_NONE`: a requirement from before the starting
     age vanishes.

**Slot 2.** `p = preq[2]`, then if `p ≥ 0`: if `p` is an epoch and
`get_preq(1, −1)` is an epoch of the same line at or above it → `TYPE_NONE`
(the slot is redundant); if `p` is an age, this type's `where` is the
University, and `ending_technology < p.age + 1` → `TYPE_NONE` (a university
tech keeps working past the game's last age).

`ending_technology` is **one-based**: `SetupWin::setup_game` stores the lobby
combo index plus one, so Classical is 1 and Information is 7, and the combo's
eighth entry, "Early Info Age", stores 7 and sets bit 31 of `info.flags`.
`starting_technology` is zero-based: Ancient is 0, Information 7, and 8 is the
"All Technologies" setting (`LeaderData::all_techs`). `starting_age()` clamps
to 7, and in Barbarians at the Gates gives the defending team
(`get_team() == 0`) the sum of both settings, capped by the ending age.

The lobby remaps are read line by line and nothing more; they are the part of
this document most in need of a behavioural check (below).

---

## `has_preq`: are the prerequisites met

`LeaderData::has_preq(t)`, top to bottom:

**Nation exceptions that waive the whole check** (return 1). Each is
`has_tribe_bonus(n)` — nation power `n`, which is the tribe's index in the
roster, `0` Aztecs … `23` Persians — and, where named, a `rules.xml` constant:

| type | power | constant (shipped) |
| --- | --- | --- |
| `MARKET` | 4 Nubians | — |
| `KNOWLEDGE` (the good) | 5 Greeks | `GREEK_KNOWLEDGE_EARLY` (1) |
| `UNIVERSITY` | 5 Greeks | `GREEK_UNIVERSITY_EARLY` (2) |
| `TOWER`, `FORTX` | 6 Romans | `ROMAN_FORT_EARLY` (1) |
| `GRANARY` | 7 Egyptians | `EGYPTIAN_GRANARY_EARLY` (2) |
| `LUMBERMILL` | 10 French | `FRENCH_LUMBERMILL_EARLY` (2) |
| `GRANARY`, `LUMBERMILL`, `SMELTER` | 12 Germans | `GERMAN_BUILDINGS_EARLY` (1) |
| `METAL` (the good), `MINE`, `SMELTER` | 12 Germans | `GERMAN_METAL_EARLY` (0) |
| `TEMPLE` | 16 Koreans | `KOREAN_TEMPLE_UPGRADES` (2) |
| `REFINERY` | 13 Russians | waived when `has_preq(OILWELL)` |

`has_tribe_bonus` itself is off under the lobby's "No Nation Powers" flag
(`info.flags & 4`), for a player with no city who started without a town, for
a tribe of `−1`, and — unless the power is a Conquer-the-World racial power —
when `leader_flags2 & 0x40`.

**The roster, which is the index.** `rules.xml`'s `TRIBES` block lists
twenty-four `TRIBE` records, each naming a file under `tribes/` whose root
element carries the display name `ScenarioFuncSet::find_nation` matches. The
record's position is the nation's identity everywhere: the `n` of
`has_tribe_bonus(n)`, the bit `TRIBE_MASK` sets, and the number
`LeaderData::tribe` holds and `LEADERS=9` prints.

| | | | | | |
| --- | --- | --- | --- | --- | --- |
| 0 Aztecs | 1 Maya | 2 Inca | 3 Bantu | 4 Nubians | 5 Greeks |
| 6 Romans | 7 Egyptians | 8 Turks | 9 Spanish | 10 French | 11 British |
| 12 Germans | 13 Russians | 14 Chinese | 15 Japanese | 16 Koreans | 17 Mongols |
| 18 Iroquois | 19 Lakota | 20 Americans | 21 Indians | 22 Dutch | 23 Persians |

`crates/sim/src/nations.rs` holds it as `ROSTER`, and
`cargo run -p rondata -- <install>` re-derives the whole list from the user's
own `rules.xml` and `tribes/` and exits non-zero if a name has moved — a
nation that shifted by one would silently hand its power to its neighbour.

**Where a player's nation comes from.** `Sim::set_tribe` takes the dump's own
`LeaderData::tribe` and sets both `PlayerTech::power` (the roster index, or
`None` for the `−1` a gaia leader carries) and `PlayerTech::tribe` (the index
into the loaded roster, for the graft and unique-unit tables, which stays in
range and is zero for a `−1`). It then refreshes the seventeen per-nation
booleans on `city::Nation` that the older mechanics read — each one an answer
from `has_tribe_bonus`, so the "No Nation Powers" and no-city gates above
apply to them unchanged. Before that wire existed the flags were all false on
every traced game, which is what hid the British commerce cap for a month
(`docs/ECONOMY.md`, "The commerce cap"). `leader_flags2 & 0x40` is still not
modelled.

**And what the nation's own file says.** `Tribe` is `rules.xml`'s `TRIBES`
row plus the file it points at, and two of that file's fields are loaded
here: `<TRIBE name>` and `<UNIT_CONTINENT>` (`+0x68`), the unit **art
style** — six of them, and one of the four coordinates
`GraphicPieces::get_unit_gpiece` sums, so it decides which animation packet
every unit of that nation plays and therefore how long every animation of
theirs runs (`docs/ANIM.md` §3.4). `Install::tribe_defs` reads both.
~~`graft[352]` and `barbarian` are still identity and false~~ — and both are in
every `DUMP_ALL` dump, under `Tribe::log_data@006f0d70`'s own names, beside
`build_continent` and `text_substitute`. **`graft[352]` is built since item
706** and matches run3's tables entry for entry (§"The graft table");
`barbarian` is still false.

**The four finals** (`0x243–0x246`) ignore their data prerequisites: they
require `INFORMATION_AGE`, `COMPUTERIZATION`, `GLOBALIZATION`,
`INTERNATIONAL_LAW` and `SELECTIVE_SERVICE`, and return 1 on those alone.

**The loop**, for `i` in `0..num_preq(t)`, with `p = get_preq(t, i, who)`:

- `p < 0`: `has_tech(p)` must hold — so `−1` passes and `−2` fails.
- else, unless `special_preq(t, &p)` waives it (below; it may also rewrite
  `p`): if `p` is a bonus type the government-tier rule applies (this branch
  is unreachable from the shipped tables, whose prerequisite columns only
  name techs, and is recorded in the open questions); otherwise `has_tech(p)`
  must hold.

**The age quota.** If `t` is an age: `epochs_get() ≥ techs_per_age(t)`, or 0.
`LeaderData::techs_per_age(age)`, with `start` as above (in Barbarians at the
Gates the raw `starting_technology`):

- `start == 0` and `ending > 6` (a full game): `age.age · 4 + 2` — **2, 6, 10,
  14, 18, 22, 26** library techs for Classical through Information.
- otherwise `span = ending − start + 1`; if `span == 0` → 0; else
  `n = (age.age − start + 1) · (28 / span) − 2`, except that `n == 1` for the
  first age of a game starting at Ancient is raised to 2.

**Governments pair and tier.** If `t` is a government: the first tier
(`0x26f`, `0x270`) passes; a higher tier needs **either** government of the
tier below — `t − 2`, or the other column of the tier below,
`((t − 0x26f) ^ 1) + 0x26d`. (The decompile spells the first as
`t − WEALTH`, the enum's name for 2.)

### `special_preq`: waivers and rewrites

`LeaderData::special_preq(t, &p)` returns 1 when the prerequisite is satisfied
outright, and may rewrite `p` before returning 0:

- `t` in `0x1b7–0x1be` (the Tower through Redoubt line), Romans with
  `ROMAN_FORT_EARLY`, and `p` not an age → **1**.
- `t` a unit, power 3 (Bantu, "unit upgrades do not require Military
  research"), `t` not `is(CATAPHRACT, 1)` and not `is(HORSEARCHERS, 1)`, `p` an
  epoch of the Military line → **1**.
- `t` a wonder, `p` an age, Egyptians with `EGYPTIAN_WONDERS_EARLY`: `p`
  becomes the previous age (`TYPE_NONE` below Classical).
- `t` in `AGRICULTURE–FOOD_INDUSTRY`, `CARPENTRY–PAPERMILL` or
  `METAL_ALLOYS–STEEL`, Germans with `GERMAN_INDUSTRY_EARLY`, `p` a Science
  epoch: `p` becomes the level below (`TYPE_NONE` below `WRITTEN_WORD`).

---

## `type_eligible` and `type_avail`: may it be bought

`LeaderData::type_eligible(t, strict)` — `strict` is 1 from every purchase
path and 0 only from `check_predecessor`:

1. `tribe_can_type(t) != 4` → 0.
2. `t == TOWN` and the player has no Town and none of its `to` upgrade built
   and none queued → **4**: the first city is always eligible.
3. For each `p = get_preq(t, i, who)`: `p < −1` → 0 (a `disable`
   prerequisite); `p ≥ 0`, not a plain tech, not an epoch — i.e. an age — and
   `ending_technology < p.age` → 0.
4. A **good**: 4, unless `strict` and `has_tech(obs)` → 0.
5. A **unit**: `NUCLEARMISSILE`/`ICBM` under Info Deathmatch (`game_rules ==
   11`) → 0. If `strict`: `has_tech(obs)` → 0; the player's `obs_flags` bit →
   0; then **the jump rule** — walk `u = get_graft(jump)` and onward along
   `jump`, grafting each: for any `u` with `has_preq(u)`, if `u`'s bit is set →
   0, and if `u` has flag `j`, `t`'s bit is clear and `t` lacks flag `h` → 0.
   Then the raw `preq[0..2]`: an age above `ending_technology` (counted as
   `index − 0x21f`) → 0. Then: flag `j` → **4**; else
   `check_predecessor(get_graft(from))`.
6. A **building**: 4, unless `strict` and (`has_tech(obs)` or the `obs_flags`
   bit) → 0.
7. A **tech**: `leader_off` bit for this player → 0; a final under Info
   Deathmatch or with `info.flags` bit 31 ("Early Info Age") → 0; an age with
   `ending_technology < age + 1` → 0; a government whose pair partner
   (`((t − 0x26f) ^ 1) + 0x26f`) is owned or queued (`has_tech_queued`) → 0.
   Else **4**.

`LeaderData::check_predecessor(p)`: while `p ≥ 0` and `type_avail(p, 0) < 3`:
if `p` is in the player's tribe mask → 0; else `p = get_graft(p.from)`. Return
1. So a non-`j` upgrade needs its predecessor *available* — owned, or not
for this nation at all.

**The jump rule is the upgrade mechanic.** Hoplites (`h`) jump to Phalanx
(`j`), Phalanx to Pikemen (`j`), and so on. Researching Phalanx is refused
once Pikemen's prerequisites are met — the player jumps straight to Pikemen,
whose own eligibility, being `j`, never asks for Phalanx — and refused
forever once Pikemen is owned. Free units keep their `h` so they are never
caught by it.

`LeaderData::type_avail(t, strict)` returns **0, 2 or 4** — "no", "eligible
but not yet researched", "available":

- `has_preq(t)` else 0; `type_eligible(t, strict)`, returned as is unless 4.
- A **unit**: a patriot (`0x160–0x165`) with `leader_flags2 & 0x1000` → 0.
  Bit clear → **2**. Then, for a type `is(MACHINEGUN, 0)`: 4 only if one of
  `RIFLEMAN`, `INFANTRY`, `MECHINFANTRY` (each grafted for the nation) is
  owned, else 0.
- Not a **building** (a tech, good or spell): 4 — except a government with
  `leader_flags2 & 0x800` → 0.
- A **building**: a wonder already built by anyone (`already_built`, the
  per-game table) → 0; `from ≥ 0`, `build_flags & 4`, bit clear → **2**;
  `SENATE`: 0 if one is queued, if there is no capital, or if the capital (or
  the city the capital defers to) already has one — the walk through
  `find_capital` and `count_buildings` is in the decompile and not
  reproduced here. Else 4.

`BuildTypeData::queue_here(t, …)` is the building's half of the question:
`DISBAND` → 1; if `t.preq[0] ≠ −2` and `t.where ≥ 0`: for a non-unit,
`this.is(where, is_build(t))`; for a unit, `this.is(where, 0)` or
`this.is(where.upgrade, 0)`. `BuildData::can_make` (`docs/PRODUCTION.md`)
chains `queue_here`, `type_avail(t, 1) ≠ 0`, and for a tech "not owned and not
`researching`" — `LeaderData::researching` walks the player's buildings and
their queues for the type, and for a unit type also for any queued unit in
the same line (`is(queued, 0)` with neither upgraded).

`Leader::tech_avail(t)` is the interface's wrapper: 0 if disabled for this
player, owned, or researching; for a government, 0 if the partner is owned or
researching; else `type_avail(t, 1)`.

---

## `gain_tech`: owning it

`Leader::gain_tech(t, x, y, announce, upgrade_units)`. Both
`Build::finished` (a completed research job, `docs/PRODUCTION.md`) and every
cascade below call it with `upgrade_units = 1`; the cascades pass
`announce = 0`. In order:

1. **Science re-prices the library.** If `t` is a Science epoch and the frame
   is not 0, every tech entry in the first library's queue other than `t` is
   `refund_cost`ed in place — `docs/PRODUCTION.md`, "Science re-prices the
   queue in place".
2. **Counters**, only if `has_tech(t)` was false: an age → `ages += 1`,
   `age_stamp[t − 0x220] = frame`; an epoch → `epochs += 1`,
   `epoch[cat] += 1`; anything else — a plain tech, a unit, a building —
   `discovered += 1`.
3. **The bit**: `tech.set(t)`.
4. **Resource unlocks.** For each basic good whose `preq[0] == t`, the
   player's starting stock of it (`game->starting[i]`, scaled in Barbarians /
   Conquer the World) is added to its bucket; this is how Knowledge and Metal
   arrive with Classical and Oil with Industrial. A good whose `obs == t` is
   sold down to 100. A Greek reaching Classical with `GREEK_DELAY_KNOWLEDGE`
   gets the knowledge stock then.
5. **Bonus-type flags**, the `leader_flags` bits above, and a handful of
   one-off effects keyed on a bonus type's first prerequisite: assimilation
   of foreign cities and defeat of capital-less players (`GLOBAL_GOVERNMENT_BONUS`),
   the armageddon counter (`ARMAGEDDON_BONUS`), population-cap recomputation
   (`GRANARY2–GRANARY5`), border recomputation.
6. **An age** rewrites the age byte on every building of the player.
7. **A unit type**, with `upgrade_units`: every live unit whose type is
   `get_graft(t.from)`, or whose `jump` chain reaches `t`, is converted in
   place (`Unit::set_type`, with a squad-size shrink when the new `uber_size`
   is smaller — the original only supports shrinking to 2); every queued
   entry of such a type is re-targeted to `t` (`BuildQueue::set_queue`,
   `track_queued` adjusted; "The queue loop" below, item 901). Then, unless `t` is a hero (`has_objmask
   0x4000000`): **every unit type `u` that is `get_graft(t.from)` or whose
   `jump` chain reaches `t` gets its `tech` bit and its `obs_flags` bit set**
   — the predecessors are owned and obsolete at once.
8. **A building type**: every live building whose type's `upgrade == t` is
   marked obsolete (`obs_flags` of its type) and converted (`set_type`), and
   `obs_flags.set(t.from)` if `from ≥ 0` — the loop over the leader's two
   building lists at `6dde5e`–`6ddf2b`, `Wall::set_type(t, 0)` through vslot
   `+0x84`, which moves the per-type counters and writes no hits (they
   follow at the next wall-stats pass). Only an auto-upgrade type (`build_flags
   & 4`) is any type's `upgrade`: the Tower becomes a Keep with Medieval Age
   (`Sim::upgrade_buildings_to`, item 1275, `docs/AI.md` §99.10). **A unit or a building type stops
   here** — a unit `goto`s the epilogue after step 7, and everything that is
   not a tech type returns after step 8; none of the cascades below run for
   them.
9. **A tech**: an epoch zeroes `misery` and, if Military, recomputes the
   population cap; a final, when announced, is a message; a government
   writes `LeaderData::gov = t`, and Capitalism gained after frame 0 adds
   `CAPITALISM_OIL_GIFT` (500) to the oil bucket; and **an age under the
   Tech Race victory (`victory == 9`), outside Conquer the World, with `ages`
   now equal to `ending_technology`, is `victory(3)`** — the win. (Conquer
   the World's variant is in the epoch arm: all 28 epochs owned.)
10. **Civic epoch**: `fix_all_borders`, walls re-masked around every city,
    and each city then runs `City::find_buildings` (`Sim::civic_epoch_sweep`,
    item 661, `docs/AI.md` §63).
11. **Buildings cascade.** For every building type `B` with `has_preq(B)` and
    some `get_preq(B, i, who) == t`: if `B.is(TOWN, 0)`, every city runs
    `City::check_upgrade`; and if `B.build_flags & 4` and
    `type_eligible(B, 1)`, `gain_tech(B, 0, 0, 0, 1)`.
12. **Units cascade.** For every unit type `u` with `get_preq(u, 0) == t` or
    `get_preq(u, 1) == t`, `has_preq(u)`, not `has_tech(u)`, **flag `h`**, not
    a hero, and `type_eligible(u, 1)`: `gain_tech(u, 0, 0, 0, 1)`. This is
    what hands out the first unit of every line with its age.
13. **Nation and wonder free techs.** The common shape is "for every candidate
    `c` with `has_preq(c)` and some `get_preq(c, i, who) == t` and
    `type_eligible(c, 1)`: `gain_tech(c, 0, 0, 0, 1)`", gated by a power and a
    constant. Two blocks differ (second reading): the **unit-line blocks**
    (Lakota, Iroquois, Indians, Korean militia) match "one of the candidate's
    two prerequisites is `t` and the other is owned" instead of `has_preq`;
    and the **Statue of Liberty** block compares a grafted prerequisite and
    asks `type_eligible(c, 0)`.

    | gate | constant (shipped) | candidates |
    | --- | --- | --- |
    | Chinese (14) | `CHINESE_HERBAL_LORE` (1) | `HERBAL_LORE … MEDICINE` (`0x25b–0x25c`) |
    | Red Fort | `RED_FORT_FORTIFICATION` (1) | `FORTIFICATION … STRATEGIC_RESERVES` |
    | Red Fort | `RED_FORT_TACTICS` (1) | `TACTICS … STRATEGYX` |
    | Lakota (19) | `LAKOTA_CAV_UPGRADES` (1) | `HORSEARCHERS…` and `ARMOREDCAR…` unit ranges, matched on slot 0 or 1 |
    | Iroquois (18) | `IROQUOIS_SCOUT_UPGRADES` (1) | the scout line |
    | Indians (21) | `INDIANS_ELEPHANT_UPGRADES` (1) | the elephant line |
    | Koreans (16) | `KOREAN_MILITIA_UPGRADES` (1) | the militia line |
    | Koreans | `KOREAN_TEMPLE_UPGRADES` (2) | `RELIGION … EXISTENTIALISM` |
    | Koreans | `KOREAN_TEMPLE_TAX_UPGRADES` (0) | `TAXATION … INCOME_TAX` |
    | Persians (23) | `PERSIANS_TAXATION` (1) | `TAXATION … INCOME_TAX` |
    | Mongols (17) | `MONGOL_FREE_FORAGE` (1) | `FORAGE … LOGISTICS` |
    | Russians (13) | `RUSSIAN_ATTRITION_UPGRADES` (1) | `ALLEGIANCE … NATIONALISM` |
    | Egyptians (7) | `EGYPTIAN_GRANARY_UPGRADES` (1) | `AGRICULTURE … FOOD_INDUSTRY` |
    | Romans (6) | `ROMAN_FORT_UPGRADES` (1) | every tech whose `where` is `FORTX` |
    | Hanging Gardens | `HANGING_GARDENS_UPGRADES` (0) | carpentry, metal, agriculture lines |
    | French (10) | `FRENCH_LUMBERMILL_UPGRADES` (1) | `CARPENTRY …` |
    | Chinese | `CHINESE_KNOWLEDGE_UPGRADES` (0) | `LITERACY …` |
    | Germans (12) | `GERMAN_METAL_UPGRADES` (0), `GERMAN_INDUSTRY_UPGRADES` (0) | metal / carpentry / agriculture lines |
    | Germans | `GERMAN_HEAVY_INFANTRY` (0) | Barracks units `is(SOLDURI, 0)` or `is(HOPLITES, 0)` |
    | Germans | `german_light_cavalry` — **absent from `rules.xml`**, so `get_item` yields −1 and the gate is open | Barracks units `is(HORSE, 0)`: none exist, so it does nothing |
    | British (11) | `BRITISH_ARCHER_UPGRADES` (1) | Barracks units `is(BOWMEN, 0)` |
    | Spanish (9) | `SPANISH_SCOUT_UPGRADES` (1) | units `is(SCOUT, 0)` |
    | Turks (8) | `TURK_FREE_SIEGE_UPGRADES` (1) | units `is(CATAPULT, 0)` |
    | Statue of Liberty | `LIBERTY_FREE_UPGRADES` (1) | units with no `special_upgrade` or trained at the Airbase |
    | Colosseum | `COLOSSEUM_FORT_UPGRADES` (0) | `FORTIFICATION …` |
    | Tikal | `TIKAL_TEMPLE_UPGRADES` (0) | `RELIGION …` |

    The Statue of Liberty's block also runs earlier, under the Conquer the
    World semaphore, over every unit type.
14. `leader_flags |= 0xc000000` (the interface's "rebuild" bits); the Rush
    Rules war-warning message when the age named by `rush_rules` is reached;
    an age refreshes every unit's and wall's graphics; `TerrainOil::gain_tech`.

**Research completes through this path and trains nothing**: the first of a
unit type sets the bit and upgrades the units already standing; the player
queues again for the first trained one (`docs/PRODUCTION.md`, "Completion").
`Build::finished` sends everything through `gain_tech(t, x, y, 1, 1)` except
three cases: a unit type already owned trains; a spell is cast; and **a
building type without `build_flags & 4` converts only the building that
finished it** (`set_type`), never reaching `gain_tech` — the global upgrades
(`c`) are the ones the whole nation takes at once.

### `lose_tech`

`Leader::lose_tech(t)`, the scenario and console inverse:

- A **unit** type: if `has_tech(t)`, `discovered −= 1`; clear the bit.
- **A building, good, spell or bonus: nothing — the bit is not touched**
  (second reading; the function has a unit arm and a tech arm and no other).
- A **tech**: clear the bit; then for every owned tech `y` in `0x220–0x274`:
  if `y` and `t` are both ages, lose `y` when `t.age ≤ y − 0x220`; otherwise
  lose `y` when `TechType::is_ultimate_preq(t, y)` — `t` appears somewhere in
  `y`'s prerequisite closure, walked through `get_preq` with the same
  substitutions. `fix_tech_flags`. Counters: a plain tech `discovered −= 1`;
  an epoch `epochs −= 1` and `epoch[cat] = compute_epoch(cat)`; an age
  recounts `ages` from the bits and rewrites every building's age byte.
  `TerrainOil::lose_tech`.
- Then `reset_obs_flags` and `leader_flags |= 0xc000000`.

`Leader::reset_obs_flags` rebuilds `obs_flags` from scratch: cleared; every
building type with `build_flags & 4`, `has_preq`, and a `from` marks that
`from` obsolete; every owned unit type marks obsolete each type whose grafted
`jump` is it, and its own grafted `from`.

### `set_age` and `set_epoch`

`Leader::set_age(n)`, `n` clamped to `0..7`: lose every owned age at or above
`0x220 + n` (from the top down), gain every age below it (`gain_tech(…, 0, 1)`),
then lose every owned non-age tech whose `has_preq` no longer holds, then
every owned unit and building type likewise; `reset_obs_flags`,
`calc_unit_stats`, `calc_wall_stats`. `Leader::set_epoch(cat, level)` is the
same over one line from its base. Both are reached from the console and the
scenario API (`ScenarioFuncSet::gain_next_age` is `set_age(ages + 1)`); no
game-rule path calls them.

---

## An age snaps every figure — 2026-09-07

Step 14's "an age refreshes every unit's and wall's graphics" is not only a
graphics refresh. `Leader::gain_tech@006dcb60:2366` is gated on the type
vtable's `+0x34`, `TypeData::is_age_type` (`+0x38` is `is_epoch_type` and
does **not** qualify), and for every live unit of the leader it runs

```text
Unit::update_gpiece(u);
Unit::set_new_location(u, u.x, u.y, 1, 1);
```

then the same `update_gpiece` over the player's buildings from slot 2000 and
over its walls. The graphics half is presentation. The **arguments** are not:
`Unit::set_new_location(x, y, param_3, param_4)` re-places the unit where it
already stands, and

- `param_4 != 0` calls `Guy::set_angle(guy 0, this->angle /* +0x50 */, 1)`,
  whose snap flag writes `GuyData::angle` (`+0x18`) and `last_angle`
  (`+0x1c`) outright rather than leaving them to the turn rate; and
- `param_3 != 0` calls `Guy::set_new_location(guy 0, des, 1)`, which writes
  `x`/`y` and `last_x`/`last_y`/`last_z`, and whose crew loop **puts** every
  tracked figure on its rotated offset instead of telling it to walk there.

So an age gives every one of a player's units its own heading for free. For a
unit standing still, or one already facing the way it is walking, that is no
change at all — which is why a once-per-age event was invisible for as long
as it was. For a unit **mid-turn** it is a free turn, and it costs the frame
the turn would have taken.

`Sim::tick` runs the buildings after the unit loop, so a research that
completes on frame *n* lands its snap **after** that frame's own step: the
step is taken along the old facing and the next one starts from the heading.

### What run86 proves

`gamelog-run86-eastindies-transportride.txt`, block 6937 (the end of sim
frame 6936), and it is three independent readings of the same event:

- **Every building of player 1 flips `max_age` 0 → 1** on that block, and
  building `1/2005`'s `queued` goes 1 → 0. The Classical Age completed.
- **Every one of player 1's 22 units has `guy.last_x == guy.x`** on that
  block and no other. On 6934–6936 and 6938–6941 only its *standing* units
  do — thirteen of them, the ones whose `Guy::move` wrote `last` and then did
  not move. Players 0 and 8 are unchanged across the boundary, which is what
  makes it the leader's loop rather than anything global.
- **`1/13` is the only unit of the game that is mid-turn on it.** Its
  `guy.angle` goes −178956970 → −612630528 in one frame — 36.35°, where its
  turn rate is `0x5555555`, 7.5° a frame — and lands exactly on its own
  `UnitData::angle`. It is not `Guy::move`'s arrived arm, which would have
  zeroed `last_speed`: `last_speed` is 25 and `avg_speed` holds at 22 across
  the block. It is a write *after* the step.

### What it cost, and the countdown it seeded

`1/13` is an AI Citizen walking to the wood tile `(211, 196)` at
`gather_down 12`. Without the snap this crate turned it at 7.5° a frame for
seven frames where the original was aligned after three, which cost it one
stalled frame at 6941 and left it tracking the original's own positions
**one frame delayed** from 6956 on.

The `GATHERORDER` countdown is seeded by the **arrival** frame, not by the
order: `wait` holds at its rolled 545 for all 61 blocks 6924–6984 while the
citizen walks, and takes its first decrement on the block after the one whose
position first equals `orders_x/y` — `do_non_flat_gather`'s "at the tile and
not yet in the loop" arm (`docs/ORDERS.md` §6.4). The original arrives on
6984 and counts from 6985; this crate arrived on 6985 and counted from 6986.
One frame in, `theirs + 1` for the whole 545-frame cycle, and the original's
countdown reached its end on **7529** where this crate still held 1 — which
was East Indies' long word for a day. The gather arithmetic was never wrong.

### Coverage

Diff-backed on every claim above: `run86_s_window_is_the_transport_ride`
(487 blocks, `1/13` now off the parted set), `run82_s_window…` (its shutdown
dump's `(11, 7)` gone) and `run88_s_window_is_east_indies_word_frame` (the
`wait` rows gone, and `1/13` off the parted set below the word). Not
established: whether an age reached through a **cascade** rather than
directly takes the same arm — the gate is read off the type the call was
made with, and no capture on this disk has one; and the buildings' and walls'
half is `update_gpiece` alone, so nothing in the simulation reads it.

---

## The starting position

`Leader::init` lays the bits down before frame 0:

- `ages = starting_age()`, `discovered = 0`, `epochs = 0`; `tech` and
  `tech_at_start` cleared.
- For every tech `0x220–0x274` that is not a final and not a government, if
  `all_techs()` or `techtype.age < starting_age`: an **age** is granted; a
  **plain tech** is granted unless some `get_preq` is `< −1` or is an epoch;
  an **epoch** is granted only in Barbarians at the Gates, to the defending
  team. The bit and the `tech_at_start` bit are set, and the counter stepped
  (`epochs` or `discovered`; ages were pre-counted). With `all_techs`, all of
  them. **So a Medieval start owns Classical, and every building tech of the
  Ancient and Classical ages whose prerequisites are ages — but no library
  epoch, and no tech that needs one.**
- `epoch[cat] = compute_epoch(cat)` for each line.
- Every **building** type with `type_avail(t, 1) ≠ 0` gets its bit; every
  **unit** type with `has_preq` and `tribe_can_type` gets its bit and
  `tech_at_start` bit, **except** — unless `all_techs` — one whose grafted
  `from` has an age prerequisite equal to `starting_age − 1` (`starting_age`
  itself under Deathmatch or Info Deathmatch): the last age's upgrades are
  not pre-granted; the player starts one step behind. Spells likewise.
- `reset_obs_flags`; then, outside Conquer the World, the nation starting
  techs: Dutch `DUTCH_FREE_COMMERCE` (1) the next Commerce epoch, Russians
  `RUSSIAN_FREE_CIVIC` (1) Civic, Aztecs `AZTEC_FREE_MILITARY` (1) and Romans
  `ROMAN_FREE_MILITARY` (1) Military, Americans `AMERICANS_FREE_SCIENCE` (0)
  Science, Persians `PERSIANS_DESPOTISM` (0) Despotism — each through
  `gain_tech`, which is why a Roman's first military epoch cascades into its
  first units.
- The `leader_flags` fog bits from the bonus types' prerequisites.

`Build::queue_up` has one more way to gain an age: under the Tech Race
victory, queueing the age that would be `ending_technology` while war is not
allowed grants it at once and queues nothing.

---

## The starting position is a function of the nation — 2026-09-04

The section above says every **unit** type with `has_preq` *and*
`tribe_can_type` gets its bit at `Leader::init`. That makes the opening
tech set a function of the leader's nation, and this harness computed it
before it had read one: `rondata::diff::build_sim` calls `Loaded::sim`,
which calls `Sim::start_techs` for every player, and only afterwards
walks the dump's `LEADER` records to `Sim::set_tribe`. So **every capture
ever built here opened with its leaders' unit bits laid down for
`tribe = 0`, the Aztecs**.

What that costs is visible in one line of a type dump. run53's AI is
British and its light-infantry line is the generic one:

| type | `TRIBE_MASK` | seeded for tribe 0 | correct for tribe 11 |
| --- | --- | --- | --- |
| `Slingers` (82) | `0xfffff4` | not owned | **owned** |
| `Atl-Atls` (85) | `0x1` | **owned** | not owned |

`Slingers`' mask clears bits 0 and 1 because the Aztecs and the Maya have
their own variants; `Atl-Atls` is the Aztec one. So the British AI began
the game owning a unit no British player can build and *not* owning the
one it can — which made `Slingers` `RESEARCHABLE` rather than
`AVAILABLE`, and `Javelineers` behind it likewise, and both eligible to
`Leader::upgrade_units` the moment the AI's Barracks finished. Two draws
the original does not spend, on run53's frame 6779.

The fix is the ordering, not the rule: the starting position is laid
again once the nations are known, and `scene_at` — which restores no
mid-game tech state, so its bits are `Leader::init`'s too — takes the
same. It also puts the lobby's `starting_age` in front of the seeding,
which was the second thing `Loaded::sim` could not have known.

**Diff-backed**: `diff::tests::run53_s_24000_frames_put_the_ceiling_where_
run33_did`, whose word moved 6779 → 6782 on this alone. **Not
established**: whether any *other* nation's opening set differs from the
Aztecs' in a way a capture on disk would show — every capture here is
British and Nubian, and this fix is the first thing that has ever asked.

---

## Two questions this answers for the other documents

- `docs/PRODUCTION.md` asks what writes the bit at `leader + 0x6c18` and what
  `queue_here` and `type_eligible` answer; all three are above.
- `docs/COSTS.md` assumes `type_avail` for the redirect table's "is this good
  available" test. For a good it is `has_preq` (slot 0 is the unlocking age,
  slot 1 nothing, `obs` never) and `type_eligible`, which for a good is 4 —
  so Knowledge and Metal are available from Classical and Oil from
  Industrial, and `Muster::military_level` is `epoch[0]`.

---

## The conversion, landed — 2026-09-04

Step 7's **object** half and the input it needs, both found by run53's
frame 6736. What stood there was six draws opening
`Guy::init_real+0x52 < Unit::set_type+0x40c < Leader::gain_tech+0x1071`,
and the two halves behind them had each been half-built: `free_rules` had
a shape and no table, and `Gained::UnitUpgrade` had a producer and no
consumer.

**The frame is the AI reaching the Classical Age**, and its leader is
British. §13's `BRITISH_ARCHER_UPGRADES` block hands a British player
every Barracks unit of the Bowmen line whose prerequisites the gain
completes — **Archers** — and step 7 then converts the three standing
Bowmen objects in place. Three objects, three `set_type` calls, one
`Guy::init_real` each.

### The five blocks whose candidates are a predicate

§13's table is written by name; the listing writes it by index, and these
five are the rows whose candidate set is a *predicate over unit types*
rather than a run of tech indices. `0x1ab` is the Barracks, and it is the
`where` column, not a lineage:

| block | constant | predicate |
| --- | --- | --- |
| Germans (12) | `+0x72c german_heavy_infantry` | `where == Barracks`, `is(0x99)` or `is(0x84)` (`6dfd05`, `6dfd18`) |
| Germans (12) | `+0x730 german_light_cavalry` | `where == Barracks`, `is(0xd1)` (`6dfded`) |
| British (11) | `+0x6dc british_archer_upgrades` | `where == Barracks`, `is(0xaa)` (`6dfeba`) |
| Spanish (9) | `+0x694 spanish_scout_upgrades` | `is(0x45)` (`6dff72`) |
| Turks (8) | `+0x66c turk_free_siege_upgrades` | `is(0x109)` (`6e002f`) |

`0xaa` is Bowmen, `0x45` Scout, `0x109` Catapult, `0x99`/`0x84` the two
heavy-infantry lineages and `0xd1` Light Horse. The German light-cavalry
block loads **empty**, which is what §13's row said it would — though not
for the reason it gave: the lineage is not missing, the **`where`** is,
because a Light Horse is trained at the Stable.

`crates/rondata`'s loader builds these five and no others. **The range
blocks are not loaded**: every other row of §13's table names a run of
tech indices whose endpoints are each a separate reading, and no capture
reaches any of them. `the_nation_free_upgrade_blocks_name_the_units_they
_hand_out` is the install-backed test.

### `Leader::gain_tech`'s conversion loop (`6dd9bd`–`6ddbd1`)

Gated on the gained type being a **unit** type, and on the caller's
`param_5`. It walks the player's object slots in order:

```
for o in 0 .. objects_mark[who]:
    u = units[who][o]
    if !(u.flags & 1): continue                            # active
    if types[t].is(0x134, 0) and u.is(0x15f, 0):           # the queue arm
        track_queued(current_upgrade, -u.num_queued)
        track_queued(t, +u.num_queued); continue
    ty = u.type_index
    if ty != get_graft(t.from) and not jump-chain(ty) reaches t: continue
    if uber_size(t) < uber_size(ty):                        # 6ddaf0
        if uber_size(t) == 1:
            if !u.is_captain(): u.die(); continue           # vslot +0x158
            u.damage = total_damage(u, 1, NULL)
        else: Error "No support for decreasing number of guys in a squad
                     to anything other than 2."
    u.set_type(t, 0)                                        # vslot +0x84
```

The `jump` walk is `x = get_graft(unittypes[ty].jump)` repeated until it
is negative or equal to `t` — the same chain [`TechTree::jumps_to`]
already walked for the *bits*; it decides the objects too.

### `Unit::set_type@00612fa0`, the guy loops

`set_type` does **not** rebuild the guy stack the way `Unit::init` does.
It clamps `guy_mark` to the new type's `squad_size` — written 1 for every
type — kills and recycles every slot past `crew_size + squad_size`, pops
fresh guys for any new slot, and then walks the whole stack giving each
guy the new type and a fresh `Guy::init_real(guy, 1)`. So a **kept** guy
holds its body and its place and loses only its clock, and every guy costs
one draw whether it is kept or new.

Both loops are `init_real` sites and the trace tells them apart: the
first, `0 .. guy_mark`, returns to `+0x40c`; the second,
`squad_size .. crew_size + squad_size`, to `+0x4a1`. All three of 6736's
draws are `+0x40c`, and since `guy_mark` is clamped to 1 that loop can
run at most **once** per call — which is how the frame says "three
objects converted" rather than "one unit with three figures".

The clock reset is visible on the same frame: `init_real` leaves
`end_time` at zero, so each re-typed guy wraps on its very next
`Unit::inc_time` and rolls again. 6736 is three of each.

**Damage carries.** `set_type` never writes `myhits`, and the object's
`damage` is untouched by the swap, so a unit converted at half health is
at the *new* type's hits minus the same damage.

### Coverage

Diff-backed: the whole chain, on run53 — the word ran 6736 → **6779**, and
6779 is a different mechanic (two draws inside the AI's own sweep). East
Indies is unmoved at 7448. Unit tests: the loader's five blocks against
the install, and `a_gained_unit_type_converts_the_line_below_it_and
_carries_the_damage` for the `from` match, the `jump` chain, the per-figure
draw and the damage.

Reading-only, and each is a stated seam in the code: the ~~**queue arm**
(`types[t].is(0x134, 0) && u.is(0x15f, 0)`, which re-targets a queued
entry instead of a standing unit — no capture has one)~~ **carrier arm**
(`types[t].is(0x134, 0) && u.is(0x15f, 0)`, which moves a carrier's own
`num_queued` from the old type to the new — no capture has a carrier; the
buildings' queues are a separate loop, "The queue loop" below, item 901);
the squad-size
**shrink** (`Unit::die` for a surplus member, `total_damage` onto the
captain — no capture converts to a smaller squad); and `set_type`'s tail
(`is(0x165, 1)` and `update_ceo_position`, `is(0x77, 0)`,
`update_gpiece`, the two vslots before `update_armor`/`update_speed`),
none of which has state here.

## The graft table — 2026-09-24

*Item 706. Established from `Tribe::init@006f0230`, `UnitType::init@0061ab50`
and `UnitTypeData::type_graft@0061d790`, and diff-backed by run3's
`DUMP_ALL`: all 24 nations' 352 entries, 8,448 of 8,448.*

`get_graft` (§"Nation variants") falls back to the tribe's own table when the
nation cannot make a type. Until item 706 this crate's table was the
identity, so a nation asked for a type outside its mask got that type back.
For the British, `current_upgrade(Bowmen)` walks `jump` to Archers, and
Archers is outside the British mask. The walk has to arrive at Longbowmen,
and it arrived at Archers, which the player does not own. So the Barracks'
free archers (`docs/CITIES.md` §4.3) came out Bowmen. The table is built at
load, in three steps:

1. **`Tribe::init`** writes each nation's table as the identity,
   `graft[i] = 0x32 + i` over 352 slots.
2. **`UnitType::init`'s pass 2.** Every unit record with a `GRAFT` writes
   itself into its graft's slot of **every nation in its own `TRIBE_MASK`**.
   It does not consult the graft's mask. The British Longbowmen, graft
   Archers, set `graft[Archers] = Longbowmen` for the British.
3. **Pass 3.** For each record `this` in index order, and each nation `i`
   with `graft[this] ≠ this` and `this` outside `i`'s mask: every **other**
   record `u` below `0x192` that is outside `i`'s mask and whose own `GRAFT`
   is `this` takes `graft[this]`. It is one level per record, in record
   order. Pass 3 alone accounts for 765 of the 8,448 entries: the test
   fails on exactly those with the pass out.

`Types::finalize_grafting` runs later, from `Game::init_data`, and rewrites
masks rather than tables. `rondata::load::tribe_grafts` is the builder.
`every_nation_s_graft_table_is_run3_s` is the diff, made to fail with pass 3
removed.

**What it moved.** Great Lakes' 14946 is the free train (run192). The 15 rows
of type, hits, graphic piece and `num_units` on `1/76`–`1/78` are gone, and
so is the leader's `attack` on 14976. The 70-against-88 was the two records'
`HITS`, not a multiplier. Parked 679's citizens (40 against 50 from 12564)
did not move: run192's floor under run178's tail holds at 416.

## The government patriot — 2026-09-24

*Item 706. Established from `Build::finished@00628490`,
`LeaderData::get_gov_hero@006e0600`, `LeaderData::get_gov@006d6a20`,
`Unit::init@00612100` and `Leader::gain_tech@006dcb60`. Diff-backed by
run192's block 14983 (Great Lakes) and by both long words.*

```
Build::finished, after its gain_tech:
  if the building is(SENATE) and (h = get_gov_hero()) >= 0:
    if gov_hero_frame < 0: train(h)                  -- no queue, no pop test
    else: the own FILTER_GOV_HERO unit within 0x20000 of the Senate
          is set_type(h)                             -- SEAM
get_gov_hero: none under leader_flags2 & 0x1000; else the first unit type
  below 0x192 whose unit_flags carry 0x4000000 (FLAGS digit 1, the six
  patriots) and one of whose three get_preq(i, −1) is get_gov()
get_gov: the first held of SOCIALISM_1, CAPITALISM_1, MONARCHY_1,
  DEMOCRACY_1, DESPOTISM_1, REPUBLIC_1 (has_preq on the bonus), as its
  government — not LeaderData::gov, which is the last gained
Unit::init: a gov hero born stamps gov_hero_frame = frame
gain_tech:161: a government gained at frame 0 sets gov_hero_frame = 1
```

`gov_hero_frame` is `LeaderData +0xa48`. `Leader::init` sets it to −1.
Every Senate research job reaches the tail, not only a government.
Governments are researched at the Senate, which is how the patriot arrives
with its government.

**On the maps.** On tick 14982, who=1's Senate on Great Lakes finishes
Despotism, and the original trains The Despot `1/79`: three figures, crew 2.
On tick 15782 East Indies' who=1 finishes Republic (`gov` −1 → 624 on
run78's block 15783), and its Senator is 694's `1/60`. This crate gained
both techs and trained nothing. With the arm, `1/79` is born where the
original's is, and the leader's `gov` and `gov_hero_frame` agree on 14983.
Both rows are compared since this item: `gov` had sat on the leader diff's
unmodelled list while `PlayerTech::gov` was written, and `gov_hero_frame`
on the coverage pin. Great Lakes went 14982 → 15175 and East Indies
15782 → 15985. `a_senate_that_finishes_a_government_trains_its_patriot_once`
is the test. It fails with the Senate tail removed, and again with the
stamp removed.

**Not established, and each a seam in the code:**

- **The `set_type` arm.** It turns a standing patriot into the new
  government's patriot. No capture reaches a second government.
- **The respawn.** `Unit::close@0060ee50` sets `gov_hero_frame = frame +
  15 × GOV_HERO_RESPAWN` when a patriot dies. `Build::process@0061edf0`
  trains one at a Senate on the frame before that.
- **`Build::activate`'s Senate head** (`docs/CITIES.md` §15). Nothing here
  trains a patriot from a newly built Senate.
- **The Despot's exit.** On 14983 the original's `1/79` is still inside the
  Senate (`visible 0`, `orders_x/y` its own point), and ours has come out
  onto its ring. By 14984 the two agree, and run196 shows `1/79` parting
  nowhere before 15208. ~~It is the new word's hypothesis~~: run196 killed
  it. Great Lakes' new word, 15175, is the free Longbowmen's guard posts,
  handed out the other way round on 15095 (`docs/RUNS.md` run196).

## The queue loop: a gain re-targets the line's queued entries (item 901)

*2026-09-26. Established from `Leader::gain_tech@006dcb60`'s listing,
`6ddbed`..`6ddd8e`, run under the emulator, and **diff-backed by
run300** (`docs/GOLDEN.md` §36).*

Step 7's second loop, after the object loop and under the same
`upgrade_units` flag. It walks the player's buildings from 2000 to the
building mark, each with `+0x8 & 1`, and each entry of its queue to
`queued`:

```
q = get_queue(i)
if types[q] is a unit type
   and (q == get_graft(t.from) or q's grafted jump chain reaches t):
    track_queued(<see below>, -1)
    BuildQueue::set_queue(i, t, NULL, 1)     # the type; counter and pairs kept
    track_queued(t, +1)
```

`BuildQueue::set_queue@006309f0` with a null price and a fourth argument
of 1 writes the entry's type and nothing else: **the progress and the
recorded price stay**, nothing is paid and nothing refunded. So a Slinger
queued behind the Javelineers research becomes a Javelineers train job at
the Slingers' price, and a later cancel refunds that price. An entry of
`t` itself, and one above `t` in the line, are left; so is an entry at a
building whose `+0x8 & 1` is clear. The building counter is `0x44(%ebp)`,
written 2000 at `6dd62f` on every path into the unit arm.

**The decrement names the walker.** On a match by `from` the register
still holds `q` and `track_queued(q, −1)` runs. On a match by the `jump`
chain the same register has been overwritten by the walk, and it ends
equal to `t`, so `track_queued(t, −1)` runs instead (`6ddcf4`, `push
esi`; the decompile's `TVar16` agrees). The net is +0 on `t` and nothing
off `q`, so `q`'s count stays one high. `Leader::track_queued@006e0f30`
never takes a count below zero. It moves `num_queued` (`+0x5a22`) and,
for a unit with `+0x1e8` set, the AI's per-building tallies.

**Under the emulator** (a scratch script, out of git): the arm entered at
`6dd999` with a synthesized frame, and the three loops run as shipped.
Six rows (a to f) are in `docs/GOLDEN.md` §36; row f is the `jump`-chain
decrement.

**run300 backs it.** On 922 the Javelineers research finishes, and
`0/2007`'s Slingers entry is `[83 at 0]` with its 46/46 kept. The buckets
are untouched, `0/10`..`0/12` are Javelineers, and the Hoplites beside
them are not.

**The gain comes before the unqueue** (item 1243). A unit research
finishes in `Build::do_queue@0061e410`: vslot `+0x1b0` (`Build::finished`,
whose research arm is `gain_tech`) at `61ec12`, and vslot `+0x1c8`
(`Build::unqueue`) only once it has returned. So the loop above runs
while the research entry still holds its own `num_queued[t]`. A `jump`
match's `track_queued(t, −1)` takes that count, its `+1` puts it back,
and the unqueue takes it off after. **`t` ends at the count of its other
entries less one per `jump` match, and never below zero.** Every
decrement is guarded: `Build::unqueue@006207c0` and
`Build::clean_queue@00620b60` take `num_queued` (`0xe3fdb2` plus the
stride) and each per-building tally (`+0xa10`..`+0xa24`) off only when
it is not zero, as `track_queued` does.

run462 backs it on who=1 (East Indies at Toughest). The Pikemen
research (record 84, `TypeIndex` 134) at `1/2020` finished on 9143 with
a Hoplites entry (82) behind it: Hoplites → Phalanx → Pikemen is a
`jump` match. On block 10178 the queue holds a Pikemen entry, and
`num_queued[84]` reads 0. The Hoplites' ghost, `[82]`, reads 1 on both
sides. Unqueued first, this crate's guarded `−1` found nothing and left
`[84]` one high from 9143. The ramp then priced the second Pikemen,
queued on 9385, one step dearer: `1/2020`'s `queue[0].cost` read 86/66
against 78/58. Food and metal stood 8 low,
the timber that followed stood 8 low, and the Senate was unaffordable
on 10184 (`check_income` 64 against 256). That is East Indies' 10185.
After that Pikemen trained on 10367 the dump's `[84]` stays 0,
where an unguarded unqueue would read −1.

**In the code**: `Sim::retarget_queued_to` in `crates/sim/src/lib.rs`,
called beside `Sim::upgrade_units_to` for each `Gained::UnitUpgrade`;
`Sim::advance_slot`'s research arm gains before it unqueues; and
`Sim::untrack_queued` is the guarded decrement, used at every unqueue,
cancel, close, transfer and re-target.
`a_gained_unit_type_retargets_the_queued_entries_of_its_line` and
`a_unit_research_gains_before_it_unqueues_and_no_count_goes_below_zero`
are its tests.

**What run300 cannot split, and rests on the emulator**: the progress kept
(the entry sat at 0 behind the research head). ~~and the `jump`-chain
decrement (Slingers → Javelineers is a `from` match). A mutation of
either fails the unit test and no widening.~~ The `jump`-chain decrement
is held by run462 now, together with the order above (item 1243).

## What is not established

- **The lobby remaps in `get_preq`** — the Military-level rescaling for
  units, the same-line rescaling for everything else, and the University
  clause — are read and not played. ~~A logged run starting at Gunpowder with
  `LeadersSync` on would settle every number; the check is named and not
  yet run.~~ **Run, 2026-08-20, and the mechanism is confirmed even though
  the individual numbers are not.** A Quick Battle with Game Rules = Custom,
  Start Age = Gunpowder (the lobby list is zero-based exactly as read here:
  Ancient 0 … Gunpowder 3 … Information 7, then "All Technologies") and a
  Nubian player, dumped whole (`docs/ORACLE.md`, `DUMP_ALL=1` +
  `InitialDump=1`), gives for both human and computer leaders:

  ```
  ages_get()      3          epoch_get(scan) 0    pop_cap  25
  epochs_get()    0          epoch_get(scan) 0
  discovered_get() 0         epoch_get(scan) 0
                             epoch_get(scan) 0
  ```

  — and the same values again in a later frame, so this is the settled state
  and not a half-initialised one. **A later start age grants no epochs, no
  techs and no pop cap.** It sets `LeaderData`'s age and nothing else;
  everything a Gunpowder-start player can build or research therefore comes
  from `get_preq` rescaling the prerequisite columns, which is precisely the
  mechanism this document read out of the code. That is a real constraint on
  the implementation: **`set_age` on the lobby path must not touch the four
  epochs**, and a sim that grants epochs to match the age will diverge from
  the original on the first frame.

  What the run did **not** settle is the arithmetic — which Military level a
  given unit's requirement is rescaled *to*, and the University clause. Those
  need the per-type `preq` columns compared against what the interface offers
  at each start age, which is a bigger check than one dump.
- **What "Early Info Age" removes beyond the four finals.** Bit 31 of
  `info.flags` is tested in `type_eligible` for finals and in the library
  interface; nothing else read.
- **The bonus-type branch of `has_preq`** (`get_govs_taken < 2` / `< 3`,
  Monarchy/Democracy needing Socialism or Capitalism). No shipped
  prerequisite column can name a bonus type, so it is dead for the shipped
  data; it may be live for a mod.
- **The Senate placement rule in `type_avail`** is read as a capital walk and
  not reproduced; it needs cities.
- ~~**`Unit::set_type`**, what converting a unit in place does to its health,
  orders and squad.~~ **Read and landed 2026-09-04** — "The conversion,
  landed", above. What is still unread is what it does to the unit's
  **orders**: the reading covered the guys, the counters and the health,
  and the tail's four vslots are seams.
- **`LeaderData::researching`'s same-line clause** for units is read from the
  loop and not exercised: it refuses a second unit type of the same lineage
  while one is queued anywhere.
- ~~**The nation free-upgrade blocks for unit ranges** have per-block
  differences.~~ Read by the second reading: two shapes, recorded at step 13;
  the Statue of Liberty variant is recorded and not implemented.
- **`Types::finalize_grafting`**, the unique-unit mask rewrite, is read and
  not modelled.
- **`misery`** (`LeaderData +0x7ec`), zeroed on every epoch; its reader is
  elsewhere.
- **`tech_frame` and `tech_cat_frame`** (`+0x7c4`, `+0x7c8`): named, logged
  by `LeaderData::log_data`, and written by nothing read here.
- Whether **`epoch[cat]` ever diverges from `compute_epoch`** in play, since
  `gain_tech` increments and `lose_tech` recounts.

---

## Second reading (2026-08-20) — landed

Two blind readers and an adjudication are in `docs/audit/2026-08-20-tech.md`.
Doubly confirmed: the type space, the columns, `tech_key`, `cat` and the
`epoch[]` order, the flag letters and the hero mask (both from the bytes),
`has_tech`, every `get_preq` substitution, `special_preq`, every `has_preq`
waiver and the age quota and the government tiers, every `type_eligible` and
`type_avail` branch, `queue_here`, the lobby encodings, the tribe ids (now
read from `Tribe::parse`), `gain_tech`'s order, the cascades and all 27
free-tech blocks with their bytes-read `is()` arguments, `lose_tech`'s
cascade, `reset_obs_flags`, `set_age`/`set_epoch`, the starting position, and
`Build::finished`. Corrected, each marked inline above: the loaders' derived
fields (a combat unit's implicit Military epoch, `upgrade`/`to`/`where`
back-links, `finalize_grafting`), `num_preq` for goods, `lose_tech` leaving
building bits alone, `gov` written on a government, the Tech Race win on the
ending age, units and buildings stopping before the cascades, the two other
shapes of the free-tech blocks, `Build::finished`'s local building upgrades,
and `is_list` as a cache. Implementation and tests followed in the same
commit.
