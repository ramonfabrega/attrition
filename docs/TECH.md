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
   `track_queued` adjusted). Then, unless `t` is a hero (`has_objmask
   0x4000000`): **every unit type `u` that is `get_graft(t.from)` or whose
   `jump` chain reaches `t` gets its `tech` bit and its `obs_flags` bit set**
   — the predecessors are owned and obsolete at once.
8. **A building type**: every live building whose type's `upgrade == t` is
   marked obsolete (`obs_flags` of its type) and converted (`set_type`), and
   `obs_flags.set(t.from)` if `from ≥ 0`. **A unit or a building type stops
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
10. **Civic epoch**: `fix_all_borders`, walls re-masked around every city.
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

## Two questions this answers for the other documents

- `docs/PRODUCTION.md` asks what writes the bit at `leader + 0x6c18` and what
  `queue_here` and `type_eligible` answer; all three are above.
- `docs/COSTS.md` assumes `type_avail` for the redirect table's "is this good
  available" test. For a good it is `has_preq` (slot 0 is the unlocking age,
  slot 1 nothing, `obs` never) and `type_eligible`, which for a good is 4 —
  so Knowledge and Metal are available from Classical and Oil from
  Industrial, and `Muster::military_level` is `epoch[0]`.

---

## What is not established

- **The lobby remaps in `get_preq`** — the Military-level rescaling for
  units, the same-line rescaling for everything else, and the University
  clause — are read and not played. A logged run starting at Gunpowder with
  `LeadersSync` on would settle every number; the check is named and not
  yet run.
- **What "Early Info Age" removes beyond the four finals.** Bit 31 of
  `info.flags` is tested in `type_eligible` for finals and in the library
  interface; nothing else read.
- **The bonus-type branch of `has_preq`** (`get_govs_taken < 2` / `< 3`,
  Monarchy/Democracy needing Socialism or Capitalism). No shipped
  prerequisite column can name a bonus type, so it is dead for the shipped
  data; it may be live for a mod.
- **The Senate placement rule in `type_avail`** is read as a capital walk and
  not reproduced; it needs cities.
- **`Unit::set_type`**, what converting a unit in place does to its health,
  orders and squad.
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
