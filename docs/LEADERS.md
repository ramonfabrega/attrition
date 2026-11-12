# Leaders: the record's held-out leaves

This document specifies the leaves of the leader record (`LeaderData`) that
the leaders line of the sync checksum is held out on
(`docs/CHECKSUM.md` §20; `LEADERS_HELD_OUT` in
`crates/rondata/src/diff/checksum_lines.rs`). Each is a counter, a stamp, a
mark or a computed value that a plain game moves and that the crate does not
hold yet. It is written for the builder who ports them. Each section names
the leaves it covers, the rule, where in `crates/sim` the event already
lives, how the rule was established and how confident it is, and what is not
established.

**Section numbers are an API.** They are the family numbers of
`docs/audit/2026-11-11-item-4152/README.md`, fixed before any reading
began, so a section keeps its number whatever order the sections land in. A
family with no section yet is one whose adjudication has not landed. `explored`
belongs to the fog and is lane 4153's port; it has no section here.

**How it was established, for every section.** Item 4152 (`docs/DECISIONS.md`
entry 124) ran three independent readings per family, all from the decompile
export and the executable's listing:

1. a first reading, `<slug>-reading.md`;
2. a blind second reading, `<slug>-second.md`. Its reader had the leaf names
   and the export, and not the first reading or §20's writer list;
3. an adjudication, `<slug>-verdicts.md`, which settles every disagreement
   back to the decompiled function or the listing, with the deciding line.

The files are under `docs/audit/2026-11-11-item-4152/`, and each section is
written from its family's verdicts alone. ~~**No diff backs any rule here
yet.**~~ Since items 4154 and 4162 a diff backs the leaves the census below
names as compared; each section's coverage says which. Where a kept dump
was consulted otherwise, the section says so, and treats that value as a
witness, not a pinned diff.

**The census: which leaves a diff can back** (item 4162, step I of 4159).
Every leaf of `LEADERS_HELD_OUT` as it stood at 98 names, plus `att`, was
looked up in the level-9 `LEADERDATA` record of the 1,128 scored kept
dumps that print it (`~/ron-data/…/Logs`; the 20 held-out battery dumps
are not counted, DECISIONS 61 §6). The table is `diff::leader`'s
`CENSUS`, and `the_census_split_is_the_record_s_own` asserts the split on
run647's start record. Of the 98:

- **25 are not printed** by `LeaderData::log_data`, so only the checksum
  line can back them: `cannon_time`, `forts_built`, `give_att_disabled`,
  the four `incoming*`, `last_building_finished`, `last_unit_finished`,
  the eleven `score_*` parts, `tribute_received`, `tribute_sent`,
  `units_bribed`, `wonders_built`, `wonders_held`. No other printed key
  carries them.
- **73 are printed** (72 under their own name or `name[scan]`; `tech.ptr`
  as the `bits 806` mask). Before this item `diff::leader::theirs` parsed
  **none** of them: a key nothing parses cannot part. Of the 73, 56 move
  (nonzero or changing between blocks in at least one scored dump),
  `multi_diff` holds one nonzero value per dump and never changes, and 16
  are quiet everywhere — `blacken`, `broke_alliance`, `counteroffer`,
  `ctw_hero_stamp`, `good_deeds`, `handicap`, `hire_stamp`, `hire_who`,
  `last_spoke`, `made_peace`, `misery`, `nukes_launched`, `repair_stamp`,
  `scholar_militia`, `tribute_demanded`, `tributes`. A quiet leaf's
  agreement is a reading's claim the dump cannot falsify.

Item 4162 made `theirs` read `att`, `pop_issues`, `misery`, `city_name`,
`handicap`, `multi_diff` and `anti_att` (the record's one float, as `%f`
millionths), and `rows` compare them.

**Conventions.** A function is cited as `Name (00xxxxxx)`. An offset is a
`LeaderData` offset. `Leader` holds `LeaderData` at offset 0 and adds 8 bytes
at the end (the vbtable pointer at +0x6ee4), so a `Leader*`'s `field_0xNN` is
the leaf at +NN. The leaders array's slot `i` sits at `0xe3a390 + i·0x6eec`.
Every value is an integer at the original's own scale, a frame is a sim
frame, and division truncates toward zero unless a section says otherwise.

**Three facts several sections share**, each established by more than one
family's adjudication:

- **Dead bodies, live inlined copies.** Nothing calls `Leader::process
  (006b88b0)`, the four `Leader::inc_*_event`, `Leader::calc_misery
  (006cdff0)` or `Leader::process_taunts (006b8ba0)`. Their stores run as
  inlined copies in `Leaders::process_all (006ed2a0)`, `Object::do_damage
  (0064a480)`, `Object::take_damage (00652020)` and the spells. A port hooks
  the live site, never the named function.
- **The bulk writers.** `Leaders::clear (006ed5d0)`, and `Leader::init
  (006e3930)` when its tribe argument is negative (the `Leaders::init` path),
  memset `[+0, +0x6929)`. `LeaderData::walk_data (006d6750)` restores
  `[+0x8, +0x692a)` raw when a save loads. The sections list these only
  where they matter.
- **The register-relative trap below +0x200.** A displacement there cannot
  be told from another record's. Each family's writers were attributed by
  the base register's type, and the false hits are named in the readings.

## 1. The city-name counter

### 1.1 Leaves

- `city_name`, +0x40c, `int`. A plain count with no scale. Slot `i` sits at
  `0xe3a79c + i·0x6eec`.

### 1.2 The rule

**Writers.** Two value stores, both in `City::generate_name (00735c90)`: the
step `+= 1` on the winner slot (listing 73614a) and the `= 1` on the owner's
own slot (7362d2). One explicit clear to 0 in `Leader::init (006e3930)`
(6e3b14). The three bulk writers of the header: `Leader::init`'s memset when
its tribe argument is negative (redundant with the explicit clear),
`Leaders::clear (006ed5d0)` (called from `Game::close (00594b80)` and
`Game::run_oos_recovery (005869e0)`, so never a reset inside play), and
`LeaderData::walk_data (006d6750)` when a save game loads. There is no
whole-record copy, and no other writer.

**The event.** `generate_name`'s only caller is `City::init`, and only when
its fourth argument is 0. That argument comes unchanged through
`Cities::init_city` from the first argument of `Build::activate`, in its city
branch (`Build+8 & 0x20`). So the counter steps on a city's activation with
first argument 0, and never with 1. The callers of `activate` that reach a
city:

- `Setup::build_cities (005ab910)` activates the start city with `(0,0,0)`
  when the game's `starting_town` option is set and `leader_flags2 & 0x80` is
  clear. Steps.
- A city completed by construction. Steps.
- `Cities::capture_city (00733380)` pushes `(0,1,1)`: capture and
  `switch_teams`. No step.
- `Wall::do_construct (006434d0)` pushes `(0,1,1)`. No step.
- `check_capture`, `defeat_by`, `find_buildings` and `cast_bribe` reach
  `activate` only on non-city buildings; `cast_bribe`'s city arm either calls
  `capture_city` or refunds and returns.
- `Leader::produce_building` reaches `activate` only at frame 0, and its
  Setup callers push no city type (none of VILLAGE 0x19e, TOWN 0x19f,
  METROPOLIS 0x1a0, FORBIDDENCITY 0x213).

**Exemptions (no step), in the order tested:**

1. The city's building `is(FORBIDDENCITY 0x213)`.
2. The game is Conquer the World (`semaphore[2] & 2`) **and** the owner's
   tribe is not barbarian (`Tribe +0x58`). Barbarians under CtW do step.
3. The tribe's cities element is missing, or its list is empty.

**The step.** Let `own` be the city's owner and `T = tribe[own]`.

- Over slots `k` in `0..8` (never 8 or 9, unrolled), with `tribe[k] == T`,
  find the largest `city_name[k]`; the comparison is strict `>`, starting
  from -1, so a tie keeps the lowest `k`. No liveness test: the scan reads
  `tribe` (+0xc) and `city_name` alone.
- `winner = k` if that largest value is `> 0`, else `winner = own`. An unused
  slot's `city_name` stays 0, so it can never be the winner.
- `prev = city_name[own]`, read before the step.
- `city_name[winner] += 1`; `n` = the winner's new value.

**The second store.** With `L` the length of the tribe's city list:

    if n < L and (n <= 1 or prev == 0) and city_name[own] == 0:
        city_name[own] = 1

The tests are signed. When the winner is `own`, `city_name[own]` is at least
1 after the step, so the store fires only when a tribe-mate took the step and
`own` had never named a city. When `n >= L`, the name is the tribe name with
a numbered suffix and no second store runs.

**Not needed for the counter:** the LFSR (taps 0x170, signed `idivl`), the
name index loop and the numbered suffix. They choose the name and write no
`city_name`. The LFSR is the maximal 8-bit polynomial; its loop ends whenever
`M >= 2` and never at `M = 1`. That matters to a name port, not to this leaf.

**Reset.** `Leader::init` alone, at game setup. `Setup::build_game` runs it
for each live slot (`leader_flags & 1`) before it reaches `build_cities`
through `build_empire`, so the start city's step lands on a zeroed count.
Nothing decrements it, and nothing folds it per frame. `ScenarioRead::
load_leader_chunk` calls `Leader::init` on every branch and drops a saved
value; a save game restores it raw through `walk_data`.

**Readers.** `generate_name` (the step itself and the name choice);
`log_data` (key `city_name`); `save_leaders_chunk` (9a791e); and the
sync checksum and the save game through `walk_data`
(`CheckSums::check_one_leader` walks `leaders.list + slot`). No reader
outside naming changes a decision.

### 1.3 In the crate

- **No counter today.** The event is `Sim::activate`
  (`crates/sim/src/cities/city.rs:2420`) calling `Sim::init_city` (`:451`) at
  `:2509`, inside the `building_is_city` branch; `activate`'s `captured`
  argument is the original's first argument. `init_city`'s only caller is
  `:2509`. The start city enters through `crates/sim/src/setup.rs:94`. Every
  `activate(.., true, ..)` outside tests is
  `crates/sim/src/cities/capture_view.rs:236`, which must not step.
- **Trap: the FORBIDDENCITY exemption is load-bearing.** The crate's
  `building_is_city` is true for FORBIDDENCITY (`city.rs:333-336`, where
  `flags & 0x20` is `is(VILLAGE)`; `crates/sim/src/vision.rs:977-980`), so a
  completed Forbidden City runs the city branch. The port must exempt it by
  type, not trust the branch.
- **No city-name list in the crate.** Exemption 3 and the `n < L` guard need
  `L` per tribe. Until the data layer carries it, a port takes every shipped
  list as non-empty and long enough that `n < L`, and records that as an
  assumption.
- **Unrelated:** `crates/sim/src/ai/ai_host.rs:445 city_name` is a synthetic
  AI label.
- **Checksum.** The leaf is `crates/sim/src/checksum/leaders.rs:98`. The held
  arm goes beside `city_mark` (`leaders.rs:1020-1033`). The port drops
  `("city_name (unheld)", 2)` at
  `crates/rondata/src/diff/checksum_lines.rs:1772` from 2 to 0 on the run-1562
  record. The verdicts give no `LEADERS_HELD_OUT` figure.

### 1.4 Reached by a plain game

A skirmish against the computer reaches the start city (when `starting_town`
is on), every city completed by construction, and a completed Forbidden City
(the exempt path). Capture reaches `activate` but never steps. The tribe-shared
winner and the `= 1` store need two leaders of one nation. Exemption 2 sits
behind Conquer the World. Exemption 3 needs a tribe with a missing or empty
city list, and is taken as unreached under the assumption in 1.3. `Leaders::clear` runs at game close
and in multiplayer out-of-sync recovery.

### 1.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/city-name-*.md`); ~~no diff backs it
yet.~~ Since item 4162 a diff backs `city_name`: the record diff
(`diff::leader::rows` against `theirs`) compares it on every scored dump
that prints the record, and the leaders checksum line holds it on run
1562's frame 91 (1 against 1 on both leaders). The tribe-shared winner is
backed only where a kept dump seats two slots of one tribe.

Verdicts: 41 confirmed, 3 corrected, 1 open, plus one new finding (the
crate's FORBIDDENCITY exemption is load-bearing). None of the corrections
changed a rule: the count of absolute `0xe3a79c` sites (8, the same two
stores), the count of `Leader::init` call sites in `ScenarioRead::
load_leader_chunk` (6), and "unused slots have tribe -1" (the dump shows
slots 8 and 9 at tribe 0; they sit outside the scan and cannot win).

Confidence is high on the writers (both nets swept, the bulk writers read)
and high on the step and its predicate (listing 735ff3-7362d2, both readings
agreeing). Gate (a)'s reach in a plain game is medium: it rests on the
crate's ratified `is(VILLAGE)` reading for FORBIDDENCITY.

**Witness, not a pinned diff.** Run 1562's frame-91 `@dumpall`
(`gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt`) logs slot 0
(tribe 4) and slot 1 (tribe 11) at `city_name 1` (lines 817145, 829402), and
slots 8 and 9 at 0. Two distinct tribes, each with `starting_town` on, each
stepped once for its start city: the rule's distinct-tribe path, with no
frame-0 AI placement and no double step. The crate emits 0 for both. The
tribe-shared winner and the `= 1` store rest on the listing alone.

### 1.6 Not established

- **A computer player placing a city at frame 0** through `Leader::make_this
  (006c94f0)` (called only from `make_stuff` and `plan_strategy`) and
  `produce_building`. Open, bounded by run 1562, which shows none by frame
  91. A capture with a computer player whose `city_name` exceeds its city
  count at frame 1 would settle it.
- **The tribe-shared path** (winner a tribe-mate, the `= 1` store). A
  capture with two leaders of one nation, diffed on this leaf, would make it
  a diff.
- **Whether `starting_town` is on by default in the lobby.** A capture of a
  default lobby, or a reading of the option's default, settles it.
- **The type predicate that sets the city flag on FORBIDDENCITY**
  (`SubObject::init (00662300)`) was not re-derived; it is taken from the
  crate's ratified reading. A diff of a completed Forbidden City on this leaf
  would back the exemption directly.
- **`L` per tribe** is not in the crate; the `n < L` guard and exemption 3 are
  ported under the assumption above.

## 2. The kill, loss and raze counters

### 2.1 Leaves

- `units_killed`, +0x80c, int.
- `units_lost`, +0x810, int.
- `buildings_razed`, +0x818, int.
- `buildings_lost`, +0x81c, int.

### 2.2 The rule

All four step by +1, never decrement, and no sim decision reads them.
Their readers are `LeaderData::log_data (006e5110)`, the end-of-game chart
(`AchieveChartPane::set_chart_data`), the checksum walk, and for the two
unit leaves `num_units_killed` / `num_units_lost` and an end-of-game Steam
stat. **Reset**: 0 at `Leader::reset_score (006e37f0)` (every live slot at
`Setup::build_game`, and in `Leader::init`); the bulk memsets, plus
`Leader::close (006b7f80)`, which also memsets `[+0, +0x6929)`; restored by
a load.

**"A countable unit death"**, the test the unit leaves share, is
`Object::take_damage (00652020)` reaching its death branch on a struck
figure that is:

1. not a building (vslot `+0x1c` answers 0);
2. its squad's captain (vslot `+0xe8`);
3. alone: `curr_uber_size(0) == 1`, no live `o_down` follower;
4. not a decoy (`unit_masks (+0x68) & 1` clear).

A non-captain figure fails test 2 and counts nothing, so a squad counts
once, at its last figure. The unit test (vslot `+0x18`) also answers 1 on
`Animal`, and there is no animal exemption to add. No plain-game
`take_damage` strikes an animal: gaia owns them from a slot ≥ 8, and
`valid_target_const` refuses `who > 7`.

**`units_lost`.** One writer, `take_damage`: on a countable unit death,
`units_lost[victim's player] += 1`. The store comes before the attacker
test, and `param_3` and `param_5` do not enter, so a death with no killer
counts: attrition (`Unit::suffer_attrition (005e1a10)` strikes through
vslot `+0x16c`), wall decay, triggers. Spell kills, `Object::disband` and
a direct `die` do not count it.

**`units_killed`.** Three writers:

- `take_damage`, on a countable unit death, and only when the call
  carries an attacker object (`param_7 >= 0`):
  `units_killed[param_8] += 1`, where `param_8` is the attacker's player.
  There is no ally, self or ownership test before the store. The `is_ally`
  test after it picks only the sign of `score_combat` (+0x44). Beside the
  store it also steps victim `+0xa58` and `leaders[param_8] +0xa5a`.
- `SpellType::cast_sabotage (006720e0)`, unit arm (a Sniper or Commando
  kill). The target must be live (`flags(+8) & 1`; otherwise the mana is
  refunded and the cast returns), and the inner test is a unit (`+0x18`),
  not a building, captain, alone, not a decoy. If it passes:
  `units_killed[caster] += 1`, target `+0xa58` and caster `+0xa5a` step,
  and `score_combat[caster] += cost / 5`. Either way the target then dies
  through vslot `+0x158` directly (dtype 3 with objmask `0x200000`, else
  2), so the victim's `units_lost` does not step.
- `SpellType::cast_counterintel (00673740)`. A non-captain target is
  replaced by its captain (`+0xe4`). The gate: the target's player is not
  the caster, not `leaders[caster].who` (+0x8), and not mutually allied
  (`diplos (+0x74) == 2` both ways). Inside the gate, if the target is a
  countable unit (unit, captain, alone, not a decoy),
  `units_killed[caster] += 1` and `score_combat` steps. Nothing else of
  this family steps: no `+0xa58`, `+0xa5a` or `units_lost`. Then
  `die(2, -1, 0)` runs inside the gate, whatever the inner test said.
  Outside the gate there is no die.

The caster's player reaches counterintel through `SpellType::cast
(00676ce0)`'s third argument.

**`take_damage`'s return**, which the raze writers test: 0 when the
target is still alive or is a clamped city building; 1 from the
site-abandon disband; 1 after `die` (vslot `+0x158`) when the dead
object was not a decoy; 2 after the decoy-overflow arm.

**`buildings_razed`.** Two writers:

- `Object::do_damage (0064a480)`: when its `take_damage` call returns
  non-zero and the target is a building (`+0x1c`),
  `buildings_razed[this's player] += 1`. There is no owner or ally test,
  so razing an ally's or one's own building counts. The later `is_ally`
  picks the sign of `score_combat`, floored at 0 on the minus side. On a
  unit kill `do_damage` returns before this. `this` is the shooter:
  `Ammo::do_damage (00678060)` passes the ammo's recorded shooter, and
  `Nuke::do_damage (0092bc80)` passes the object the nuke effect records.
- `cast_sabotage`, building arm:
  `take_damage(dmg, 0, 3, -1, 1, 0x55555555, -1, -1, 0)` on the building
  itself (`Build`'s vslot `+0xac` returns `this`), with
  `dmg = max(+0x114 / 2, the type-indexed floor)`. A non-zero return gives
  `buildings_razed[caster] += 1`.

On a building, "non-zero" is either:

- **death**: accumulated damage ≥ hits, and not a city building. A city
  is clamped and returns 0, for every caller, because the clamp is not
  under the `param_5 == 0` block; or
- **the site-abandon disband**, which needs all four of:
  - unfinished: vslot `+0x4c` = `WallData::is_active (00472350)` =
    `flags & 4`, clear;
  - the owner is a computer player (`leader_flags & 4` clear);
  - `hits(0) ≤ 2·damage`;
  - `build_masks (+0x60) & 0x2000`, whose only writer is
    `Leader::check_orphaned_buildings (006c9f20)`.

  The disband returns 1. It credits a raze while the owner's
  `buildings_lost` does not step.

**`buildings_lost`.** One writer, `take_damage`: when a building or wall
segment (`+0x1c` non-zero) reaches the death branch,
`buildings_lost[owner] += 1`, then `ScenarioData::last_razed[who] = o`,
then straight to `die`. There is no attacker test and no `param_5` test,
so combat, sabotage, triggers and border decay all count. Border decay is
`Wall::process (00640450)` striking
`take_damage(8, 0, 1, -1, 1, 0x55555555, -1, -1, …)` on a started building
that stands on another, non-allied player's tile. It reaches every
building, not only walls: `Build::process` calls `Wall::process` directly.
Not counted:

- a finished city building (clamped);
- the site-abandon disband;
- `Wall::process`'s disband of an unstarted site;
- `Object::disband` and a direct `die`.

So, summed over players, `units_lost ≠ units_killed` (attackerless
deaths, spell kills) and `buildings_lost ≠ buildings_razed` (abandon
disbands, attackerless decay).

### 2.3 In the crate

None of the four is held. Each exists only as a checksum name
(`crates/sim/src/checksum/leaders.rs:157-161`), and each is `(unheld)`,
count 2, in `LEADERS_HELD_OUT`
(`crates/rondata/src/diff/checksum_lines.rs:1769, 1770, 1847, 1848`). A
port gives each a value arm and removes its four entries. The sites, all
in `crates/sim/src/combat/fight/delivery.rs` unless named:

- **`take_damage` unit deaths**: `Sim::take_damage_typed` (`:469`), its
  `Taken::Died` arm (`:537`), gated on `alone` (`:515`,
  `takes_remainder_share` in `units/store.rs:667`, the same captain ∧
  size-1 test) and `!decoy` (`:530`), before `close_count`.
  - `units_lost` goes to the struck unit's owner, with no attacker test.
  - `units_killed` goes to `owner_of(_by)` only when `_by` is an object.
    The crate has no `param_7 < 0` notion there, so the
    attacker-present test is the builder's to carry.
- **Attrition deaths**: `Sim::die_by_attrition` (`:844`, called from
  `territory/upkeep.rs:307`) bypasses `take_damage_typed`, so `units_lost`
  goes there too. **Trap**: read the captain/alone/decoy test *before*
  its `relink_squad`.
- **Sniper/Commando**: `Sim::cast_sniper_unit` (`:806`). It needs the
  caster's player, which its `(v, dtype)` signature lacks. It credits
  `units_killed` only, never `units_lost`.
- **Counterintel**: `Sim::cast_counterintel`
  (`crates/sim/src/spells/cast.rs:1031`, SEAM doc at `:1028`). It
  credits `units_killed` only.
- **`do_damage` razes**: `Sim::do_damage_round`'s
  `if let Obj::Building(b) = target { if killed {` block (`:212`). The
  count goes to `owner_of(attacker)`, *before and outside* the plunder's
  unit / non-splash / non-air / non-ally gate.
- **Sabotage razes**: `Sim::cast_sabotage_building` (`:884`). It needs
  the caster's player.
- **Building deaths**: `Sim::damage_building`'s `Taken::Died` arm
  (`:1041`), before `die_building(b)`, for the building's owner. Do not
  put it in `die_building`, which other paths reach. The city clamp
  (`:1025`) returns before the arm. This one function serves:
  - combat (`take_damage_typed`);
  - sabotage (`cast_sabotage_building`);
  - border decay (`cities/city.rs:3128`, from `building_attrition`,
    which rightly serves every building).
- **Absent**: the site-abandon disband. `damage_building` has no `0x2000`
  arm, and the bit itself is a SEAM (`crates/sim/src/ai/ai_make.rs:936`).
  Until both land, the crate under-counts `buildings_razed` and fires
  nothing for those sites.
- `last_razed` is not carried on this path either; it is outside this
  family.

### 2.4 Reached by a plain game

All four, in a skirmish against the computer:

- combat kills and razes;
- attrition and border-decay losses;
- the abandon arm, when a computer's orphaned unfinished site is struck
  to half its hits (computer players run `check_orphaned_buildings`).

Sniper, Commando and counterintel are skirmish units' spells; whether the
computer casts them is unread. The decoy-overflow self-credit (§2.5) needs
a scenario.

### 2.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/kills-*.md`); no diff backs it yet.
High confidence: every store and every test was settled at a listing
instruction.

The listing net found exactly 11 absolute sites (7 `incl` stores, 4
loads), no slot-biased forms, and only `reset_score` among the 140
register-relative stores to these offsets. `take_damage`'s callers through
vslot `+0x16c` are eight, and no vtable overrides that slot.

| leaf | confirmed / corrected / open |
|---|---|
| `units_killed` | 22 / 1 / 0 |
| `units_lost` | 12 / 0 / 0 |
| `buildings_razed` | 15 / 1 / 0 |
| `buildings_lost` | 13 / 0 / 0 |

Corrections that changed a rule:

- **The decoy-overflow self-credit is unreachable in a plain game.**
  After `die`, a decoy figure with whole points past its share strikes
  its captain with `param_8` = its own player. But
  `SpellType::cast_create_decoy (00674370)` sets `unit_masks |= 1` on
  every figure of the new squad, so the captain fails test 4 and nothing
  counts. The only other writer of that bit is the scenario unit loader
  (`ScenarioRead::load_unit_chunk`), so a mixed squad needs a scenario.
- **The abandon arm's vslot `+0x4c` means "finished"** (`flags & 4`), not
  "a city". The arm strikes an *unfinished* building. The city test is a
  different slot pair (`+0x20` and `flags & 0x20`).

Settled where a reading left it open:

- `do_damage`'s `this` from ammo and nukes is the shooter;
- every `Build` reaches `Wall::process`;
- `Build`'s `+0xac` is `this`.

No kept-dump value is cited.

### 2.6 Not established

- **Spell castability upstream.** When a counterintel or sabotage cast
  happens at all: `SpellType::cast`'s pre-check (the call at `0x6763c0`)
  and `is_valid_target`. This decides how often the spell writers fire,
  not what they count. A reading of those two settles it, or a capture
  where a player casts them.
- **Whether the computer casts Sniper, Commando or counterintel** in a
  skirmish. This is unread, and its AI scripts or a capture settle it.
- **No value diff.** The rules rest on readings alone until a port's
  checksum arms meet the leaders line of a kept run. Any capture with a
  kill, a raze, an attrition death and a border-decay loss on its frames
  exercises all four.

## 3. The combat rates

### 3.1 Leaves

- `frame_battle`, +0xa4c, `int`. A frame number; 0 means never.
- `average_death_rate` +0xa50, `average_kill_rate` +0xa52,
  `average_damage_rate` +0xa54, `average_hit_rate` +0xa56, `ushort`.
- `deaths_current_frame` +0xa58, `kills_current_frame` +0xa5a,
  `hits_current_frame` +0xa5c, `damage_current_frame` +0xa5e, `ushort`.
- `deaths_fifteen_seconds` +0xa60, `kills_fifteen_seconds` +0xa62,
  `hits_fifteen_seconds` +0xa64, `damage_fifteen_seconds` +0xa66, `ushort`.

Every add below is a 16-bit wrapping add. Slot `i`'s block sits at
`0xe3addc + i·0x6eec`.

### 3.2 The rule

**Writers, complete.** Four event writers feed the `_current_frame` words
and one fold writes everything else. The four `Leader::inc_*_event` have no
caller; their inlined copies are the live sites. Besides the header's bulk
writers, `Leader::init (006e3930)` zeroes all thirteen by explicit stores.
No other store exists (both offset nets and an absolute sweep).

**W1, the blow: `Object::do_damage (0064a480)`.** Exemptions: return if the
blow count argument is `< 1`, or if the attacker is a unit and a decoy
(`unit_masks & 1`). Nothing else returns before the store, and there is no
owner, ally or self test.

    s = 5 if is_siege(attacker type) and target is a building or wall else 1
    leaders[attacker.who].hits_current_frame   += s
    leaders[owner(target)].damage_current_frame += s

It counts blows, never hit points. A blow on one's own object adds to the
same leader's hits and damage.

**W2, the structure kill: `do_damage`, after the take.** Before the take,
record `active = target alive (flags & 1) and is_build (a Build, not a Wall)
and active (flags & 4)`. If the take (vslot `+0x16c`) returns non-zero and
the target is a building or wall:

    k = 5 if active else 2
    leaders[attacker.who].kills_current_frame  += k
    leaders[owner(target)].deaths_current_frame += k

No unit, splash, air or ally gate; razing one's own wall counts a kill and a
death to one leader. The take also returns 1 on a **not-dead** path, and W2
counts it: a building or wall that is not active, whose leader lacks
`leader_flags & 4`, with `hits <= 2·damage`, and whose type has flag
`0x2000` at `+0x60`, is disbanded, giving `k = 2`.

**W3, a unit's death: `Object::take_damage (00652020)`.** A dying building or
wall skips these leaves. A dying unit counts only if it is its squad's
captain, the squad's size is 1, and it is not a decoy. Then `+0x810 += 1`
unconditionally, and only if the attacker index argument `param_7 >= 0`:

    leaders[victim.who].deaths_current_frame += 1
    leaders[param_8].kills_current_frame     += 1

`do_damage` passes the attacker's own index and `who`. The other nine
object callers of the take (attrition, `Wall::process`, scenario functions)
pass `-1`, so attrition and scripts count nothing here. The take's tail
re-calls itself only when the dying unit is a **decoy** with positive
overflow `damage - hits` and a captain index `>= 0` other than itself; the
re-call passes the captain's index and the victim's own `who`. If the
overflow kills the captain, the owner gets one death and one kill of its
own.

**W4, the sniper: `SpellType::cast_sabotage (006720e0)`, unit arm.** A dead
target takes the refund and returns. For a target that is a unit, not a
building or wall, a captain, size 1 and not a decoy:

    leaders[param_4 (target owner)].deaths_current_frame += 1
    leaders[param_2 (caster)].kills_current_frame        += 1

(with `+0x80c` and `+0x44 += cost/5` on the caster). The arm then kills the
target by `Unit::die` (vslot `+0x158`), not by a take, so the credit rests
on the predicate alone, and the unit dies whether or not it qualified. The
**building arm writes no leaf of this family**: its stores are `+0x818` and
`+0x44`, and its take passes `-1`.

**W5, the fold: `Leader::process_event_frame (006ec180)`.** `Game::do_frame
(00591ef0)` calls it for slots 0..7 with `leader_flags & 1`, after
`Achieve::capture_data` and before `frame += 1`. It returns unless
`frame % 50 == 0`. Then, in order, for each of the four kinds X (deaths,
kills, hits, damage):

1. `X_fifteen_seconds += X_current_frame` (the raw count).
2. `c = (X_current_frame · 100) mod 2^16`, stored back.
3. `avg = c == 0 ? (avg · 7) >> 3 : (c + avg) >> 1`, unsigned 32-bit on
   zero-extended words, stored as `u16`; the zero test is on the 16-bit
   product. deaths→+0xa50, kills→+0xa52, hits→+0xa56, damage→+0xa54.
4. The console leader's music mood, from `average_hit_rate +
   average_damage_rate` (thresholds 300, 600, and 2000 only when
   `0xecba20 == 2`) and the rivals' non-zero rates. Writes no leaf.
5. The battle test, below.
6. All four `_current_frame` words to 0.

**The battle test.** With `age = leaders[this.who]`'s age (the encrypted
word `data_encrypted->+0xdc ^ 0x62766`, 0-based), `D =
average_death_rate`, `K = average_kill_rate`, zero-extended, all compares
signed `i32`:

    if D + K >= (age + 1)·125 and (frame_battle == 0 or frame - frame_battle >= 1800):
        if D >= K + 10·age + 20:   Achieve::add_event(1 defeat, who, "")
        elif K >= D + 10·age + 20: Achieve::add_event(0 victory, who, "")
        else: nothing (averages untouched)
        on either event: D = K = 64536 (0xfc18); frame_battle = frame

`frame_battle` is the pre-increment frame, a multiple of 50. The sentinel
reads unsigned (every read zero-extends). Over the 36 empty folds of the
lock-out it decays to 525 (`v·7 >> 3`, 36 times). A consequence of the
arithmetic only: past the lock-out, a window with deaths and no kills halves
`D` toward `c` while `K` keeps 7/8, which can record a **victory** (from 525,
three deaths give `D = 412`, `K = 459`, a lead of 47, enough through age 2).

**Resets.** `_current_frame`: every fold frame (step 6), `Leader::init`.
`_fifteen_seconds`: `Leader::init` only, so despite the name each is a
game-long total mod 2^16. Averages: the fold and the battle event.
`frame_battle`: `Leader::init`. A save-game load restores all thirteen raw.

**Readers that change a decision.** `average_death_rate` and
`average_kill_rate` (the battle test); `frame_battle` (the lock-out);
`average_hit_rate` and `average_damage_rate` (music only). `draw_timers`
displays the averages and `deaths`/`kills_current_frame` (×100), never
`hits_current_frame`. `log_data` and the checksum walk read all thirteen.
`Achieve::add_event (007af660)` writes no leaf.

### 3.3 In the crate

No field or function of the family exists in `crates/sim` beyond the
checksum table (`crates/sim/src/checksum/leaders.rs:264-276`). The fields
belong in `crate::ai::Leader` (`crates/sim/src/ai.rs:637`), beside
`frame_attacked`/`attacked_by` (`:697-698`).

- **W1**: `Sim::do_damage_round` (`crates/sim/src/combat/fight/delivery.rs:70`),
  between the decoy return (`:94-98`) and `treaty_on` (`:107-109`), where
  the original's store also sits; `hit_who` is already `owner_of(target)`
  (`:107`). **Trap**: the crate's `!self.struck(target)` return
  (`:81-83`; `combat/fight/stats.rs:92-100`) precedes the decoy test and
  refuses a non-active unit or a dead or combat-less building, which the
  original's store does not see (open, 3.6).
- **W2**: the `if let Obj::Building(b) = target { if killed` block
  (`delivery.rs:212-222`, `killed` at `:193`), outside its unit/splash/
  air/ally gate, with `active` read before the take. `Obj` has only `Unit`
  and `Building` (`combat.rs:381-384`), so the wall's home is wherever the
  crate keeps walls; unsettled. `killed` must be true on a disbanded
  inactive structure; whether `damage_building` models the disband arm was
  not checked.
- **W3**: the unit `Taken::Died` arm of `Sim::take_damage_typed`
  (`delivery.rs:469`, arm `:537`), gated on captain, squad size 1, not
  decoy, real attacker. `Sim::take_damage` (`:452`) is `#[cfg(test)]`;
  attrition is `Sim::suffer_attrition` (`territory/upkeep.rs:286`) and never
  reaches this arm, so no attrition guard is needed. The decoy tail already
  exists (`:560-596`) and passes the captain as its own attacker, so a
  counter in the `Died` arm credits the owner's own kill there with no
  extra code.
- **W4**: `Sim::cast_sniper_unit` (`delivery.rs:806`), on the
  captain/size/decoy predicate, before `die_with`. **Trap**: its SEAM
  (`:803`) lists `+0xa58` among "the caster's leader counters";
  `deaths_current_frame` goes to the target's owner. **Trap**:
  `cast_sabotage_building`'s SEAM (`:881`) names `+0xa58`/`+0xa5a`/`+0x80c`;
  the building arm writes none of them, and the SEAM should name `+0x818`.
- **W5**: a new fold in `Sim::tick` (`crates/sim/src/core/frame.rs:73`),
  before `self.frame += 1` (`:475`), for slots 0..7 with `leader_flags & 1`.
  `Achieve` is unported.
- **Checksum.** The held arms go beside `"frame_attacked"`
  (`checksum/leaders.rs:1157`). Each of the thirteen is
  `("<leaf> (unheld)", 2)` in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1762-1765, 1776-1779, 1789,
  1797-1798, 1803-1804`), counted on run 1562's frame 91; a port holding a
  leaf moves its row. The verdicts give no figure for how many would agree.

### 3.4 Reached by a plain game

A skirmish against the computer reaches W1 on every blow, W2 on every
structure razed, W3 on every squad's last captain killed in combat, the fold
every 50 frames, and the battle event. W4 needs a spy's sniping, rare but
reachable. At age 0, three unit deaths in one window (`D = 150`) pass the
battle size test. The disband arm's reach is open (3.6). Scenario functions reach the take
with `-1` and count nothing (triggers, cut from v1). `Leaders::clear` runs
at game close and in multiplayer out-of-sync recovery. Gaia slots 8 and 9
never fold.

### 3.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/combat-rates-*.md`); no diff backs it yet.

Verdicts, as the adjudication counts them: the shared writers 52 confirmed,
5 corrected, 1 open; per leaf, `hits_current_frame` 6/1/1,
`damage_current_frame` 5/0/0, `kills_current_frame` 9/1/1,
`deaths_current_frame` 5/1/0, each `_fifteen_seconds` 6/0/0, each average
6/1/0 (one shared correction), `frame_battle` 9/0/0, the gaia rows 2/0/1.
Three questions are open (3.6).

Corrections that changed a rule: the take's tail re-call is the decoy
overflow only; the sniper's unit arm kills by `Unit::die`, and the
building arm counts nothing; the crate's `take_damage` wrapper is test-only;
`draw_timers` does not read `hits_current_frame`; the decay is 525; the
take has nine object call sites. One addition: the disband arm counts in W2.

Confidence is high throughout: every step and predicate is confirmed on the
listing. The verdicts cite no kept-dump value for these leaves.

### 3.6 Not established

- **The crate's `struck` return ahead of W1.** Whether the original ever
  runs `do_damage` on a target `struck` refuses (non-active unit, dead or
  combat-less building). If it does, the original counts the blow and the
  crate as placed cannot. A per-frame `LeadersSync` diff of
  `hits_current_frame` over a fight settles it.
- **The disband arm's reach.** Which types carry flag `0x2000` and which
  leaders hold `leader_flags & 4`; and whether the crate's `damage_building`
  disbands at all. A reading of the flag's loader, then a diff of
  `kills_current_frame` on an inactive structure under attack.
- **Gaia slots 8 and 9.** The fold stops at slot 8, so any store there would
  never fold or clear; `ObjectData::valid_target_const (006472c0)` refuses
  `who > 7` and the crate's searches stop at 8 (`crates/sim/src/world.rs:40-55`),
  so no ordinary blow reaches them. A `LeadersSync` line for slot 8 or 9
  with non-zero `deaths_current_frame` would settle it.
- **The wall's home in the crate** for W2.
- **No diff yet** of "`_fifteen_seconds` is never reset" (a long-fight
  dump) or of the battle event (any capture that logs one).

## 4. The victim stamps

### 4.1 Leaves

- `attack_stamp`: +0xf4, `int[8]`, indexed by attacker.
- `raid_stamp`: +0x114, `int[8]`, indexed by attacker.
- `ally_stamp`: +0x154, `int[8]`, indexed by attacker.
- `ctw_hero_stamp`: +0x200, `int`, a scalar.

All four live on the **victim's** record and hold a sim frame.

### 4.2 The rule

**Shared by all four.**

- **Writer**: `Object::do_damage (0064a480)` alone, in game. `Leader::init
  (006e3930)` zeroes all four unconditionally (the eight-pass loop for the
  arrays, then +0x200 after it). The bulk writers apply as in the header.
  `Leader::plan_strategy` and `Leader::end_process` are not writers.
- **Event**: one blow. `this` is the striker A, and `att` is A's raw owner
  byte (+0x9), not `leaders[att].who`. `vic` is the victim argument. On the
  `Ammo::do_damage` path `this` is the object that fired, not the
  projectile. The callers are `Unit::fight` (3 sites), `Ammo::do_damage` (3),
  `Nuke::do_damage (0092bc80)` (2) and `do_damage` itself (1, the spill onto
  builders after the take). Each spilled hit runs the stamp block for its
  own target's owner.
- **Head exits** (no stamp): `count < 1`, a decoy striker, `vic == att`.
- **Timing**: the block runs before the take, so a killing blow stamps. It
  reads T's `damage` (+0x24) as it stood before this blow.
- **Cell**: `leaders[vic].X[att] = frame`, with `frame = game->frame`. Every
  window is strict: `frame − stamp > N`.

**`heavy`.** It starts at 0 and is set only for a target T with
`obj_flags & 1`, `is_build` and `is_active`. It is computed before the
city test, so a city-less active building keeps its own value:

    heavy = siege(A)  or  damage + whole >= ceiling
    ceiling = hits(0) / 10   if T is a defence type or T.obj_flags & 0x20
            = hits(0) / 4    otherwise

Here `whole` is this blow's scaled whole hits after the `>> 4` split. Both
divisions are signed and truncating. `siege(A)` strictly means A's type is
siege and T answers `is_wallbuild`. Every Build and Wall answers 1 there, so
on this arm it reduces to "A's type is siege". An inactive or flag-less
building, a unit and a wall all have heavy = 0.

**The arms, in order**, on every blow past the head exits:

1. **Capital heavy.** T is an active building of `vic`'s in a city whose
   `city_flags & 0x10` is set, and heavy. If `frame − capital_stamp[att] >
   300`: `attack_stamp[att] = raid_stamp[att] = capital_stamp[att] = frame`.
   Then the ally test runs (below), whichever way the gate went.
2. **Plain city heavy.** The same, but not the capital. If `frame −
   city.attack_stamp > 300`: `raid_stamp[att] = attack_stamp[att] = frame`.
   Then, always, `city.attack_stamp = city.raid_stamp = frame`. Then the
   ally test runs.
3. **Light city, or a worker.** Either T is a city building of `vic`'s with
   heavy 0 (an active building with `obj_flags & 1`, capital or not), or T
   is a worker (type 0x32..0x35) whose gather
   order names an object owned by `vic`, at a non-negative index, that is an
   active building in a city. Then the city's light arm runs: if `frame −
   city.raid_stamp > 300`, `raid_stamp[att] = attack_stamp[att] = frame`.
   Then, always, `city.raid_stamp = frame`.
4. **Territory step**, after 1–3, for every blow (units, walls and
   city-less buildings included). `land` is the signed `who` of T's cell.
   - **Raid branch**: `land == vic`, and T is a building with heavy 0
     (inactive or flag-less included), a worker, a caravan, or a merchant
     (type 0x3d, 0x3e or 0x190). If `frame − raid_stamp[att] > 300`, set
     `raid_stamp[att] = frame`. The attack test is then skipped.
   - **Attack branch**: otherwise. If `frame − attack_stamp[att] > 300`,
     set `attack_stamp[att] = frame`. When an arm 1–3 has just written the
     stamp, `frame − frame = 0` and nothing changes.
   - A wall answers 0 to both `is_build` and the caravan slot, so it takes
     the attack branch even on its owner's land.

A non-worker unit goes from the head to the CtW hero test (below), then to
step 4. Any other non-city target goes straight to step 4. No arm has a
console or alliance gate on a store. A city arm never writes one of
`raid_stamp`/`attack_stamp` without the other.

**`ally_stamp`** runs only after arms 1 and 2 (heavy, city ≥ 0). If
`frame − ally_stamp[att] > 300`, set `ally_stamp[att] = frame`, for every
pair of players. A test of the console against `vic` (`vic` is the console,
or `Leader::is_ally (006edb50)` on `leaders[vic]` with the console's slot),
and throttles of 300 (capital) and 400 (plain), gate only the local-ally
notice. They never gate the store. A light hit, the worker arm, a unit, an
inactive or city-less building and the territory step never write it.

**`ctw_hero_stamp`** is written only under Conquer the World. Predicates,
in order:
1. T is a unit and not a worker;
2. the `semaphore` byte +0x822 has bit 1 set (bit 17, the CtW flag);
3. T's type is `conquest_game->hero_type`;
4. `frame − ctw_hero_stamp > 120`;
5. `vic == console->who`;
6. `(hits(0) − damage)·100 / hits(0) < 70`, signed, with damage before the
   blow.

If all hold, `leaders[vic].ctw_hero_stamp = frame`. The inner `> 75`
re-test always holds, and the sound plays whatever the notice gate says. The
store is per machine (predicate 5) and lands in a checksummed field, but CtW
is cut from v1. In a plain game the leaf is 0 for the whole game.

**Reset**: none mid-game. The only stores in `.text` are `do_damage`'s.

**Readers that change a decision.**
- `do_damage` reads its own gates.
- `ScenarioFuncSet::was_attacked (009fd290)` (`attack_stamp`) and
  `ScenarioFuncSet::was_raided (009fd140)` (`raid_stamp`) share one body.
  Slots are 1-based. A victim outside 0..7, or not live (word +0 has
  neither bit 1 nor bit 2), answers −1. If the attacker slot is in 0..7
  and live: stamp 0 → 0; `n > 0` → `(frame − stamp)/15 <= n`; `n <= 0`
  → 1. Otherwise the reader scans attackers 0..7 that are live, not the
  victim and have a non-zero stamp. It answers 1 on the first that passes
  (`n == 0`, or within `n`); a negative `n` matches none.
- `ally_stamp` and `ctw_hero_stamp` have no reader beyond `do_damage`,
  `LeaderData::log_data (006e5110)` and the checksum walk. No AI or economy
  code reads any of the four.

### 4.3 In the crate

- **Arms 1–3**: `Sim::city_alarm` (`crates/sim/src/combat/fight/delivery.rs:280`),
  called from `Sim::do_damage_round` (`:70`) at `:190`, before the take at
  `:192`. It already writes the city's two stamps. The leader stores are
  missing, and so is the light arm's `frame − city.raid_stamp > 300` gate on
  them. The city's own stamp is still written either way, which is right.
- **Trap, the early returns.** The capital branch returns `true` inside its
  `> 300` test and `false` after it. The ally store must come before both.
  `city_alarm` also returns early at the worker branch, at `_ => return
  false` and on a building with no city, so the territory step cannot live
  there.
- **Step 4 is new**: in `do_damage_round`, between `:190` and `:192`. It
  covers the spill, since `spill_onto_builders` (`:380`) re-enters
  `do_damage_round` (`:435`). The predicates exist: `Sim::is_worker` (which
  `city_alarm` already calls), `is_merchant`
  (`crates/sim/src/economy/calc_gather.rs:555`, `MERCHANTS` at `:40`), and
  `is_caravan` (`crates/sim/src/units/set_type_adapter.rs:94`). The cell is
  `World::owner_at` (`crates/sim/src/world.rs:2013`), which reads the cell's
  `who`, not `who2`. Its `Owner::None` and `Ambiguous` never equal a player,
  which matches the original's signed compare.
- **Storage**: no row today. The three arrays go beside
  `AiLeader::capital_stamp` (`crates/sim/src/ai.rs:704`).
- **Checksum**: `crates/sim/src/checksum/leaders.rs` lists all four (`:61`,
  `:72`), and `leader_image` has no arm for them (`_ => None`, `:1305`).
  `ctw_hero_stamp` belongs in the cut-track zero arm (`:1246`..`:1273`,
  `Some(vec![0; count])`). In `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1755, 1758, 1775, 1819`) each
  leaf is `(unheld)`, count 2. Each port removes its entry.
- `city_alarm`'s own comment notes that the dump prints the three arrays
  only at `LEADERS=9` (`delivery.rs:279`).

### 4.4 Reached by a plain game

`attack_stamp`, `raid_stamp` and `ally_stamp`: yes. None of the four arms has
a semaphore or mode test, and any cross-owner hit reaches step 4. The
trigger readers `was_attacked` and `was_raided` are off the plain path, so
in a skirmish the three leaves matter to the checksum and the save.
`ctw_hero_stamp`: no. It sits behind the CtW bit.

### 4.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/victim-stamps-*.md`); no diff backs it
yet.

The verdicts, per claim row:

| part | confirmed | corrected | open |
|---|---|---|---|
| shared ground | 22 | 0 | 1 |
| `attack_stamp` | 21 | 1 | 0 |
| `raid_stamp` | 17 | 0 | 0 |
| `ally_stamp` | 13 | 0 | 0 |
| `ctw_hero_stamp` | 17 | 0 | 0 |

Every store, gate and exit was settled at a listing address. The writer set
rests on an absolute-address scan of `.text` for all four leaves in slots
0..8 (18 stores in `do_damage`, 4 loads in the two trigger readers). A
register-indexed net found no leader store either. Confidence is high.

The correction that changed a rule: `was_attacked` answers true for any
`n <= 0` when the stamp is non-zero, not only `n == 0`, and it has a second
form that scans every live attacker. The same holds for `was_raided`. Three
points one reading had left open were settled:
- a wall takes the attack branch;
- on the Ammo path `att` is the shooter's owner;
- the spill stamps per spilled target.

No kept-dump value is cited as a witness.

### 4.6 Not established

- **A gaia striker.** The column is the raw owner byte and the arrays are
  `int[8]`. Gaia is slot 8, and `Leaders::clear` walks 10 records. A
  gaia-owned striker would write column 8, which aliases the next leaf:
  `attack_stamp[8]` is `raid_stamp[0]`, `raid_stamp[8]` is
  `capital_stamp[0]`, and `ally_stamp[8]` is `tribute_stamp[0]`. Nothing
  shows a gaia object reaching `do_damage` in a plain game. A port takes
  `att` in 0..7 and asserts it. To settle it, grep a capture's `DeathsSync`
  or `UnitsSync` for damage dealt by `who 8`, or read whether any gaia type
  has an attack.
- **The writer net's limit.** A store through a base other than the record
  start (as `Leader::init` uses `this+0xd4`) is outside the
  register-relative net. The absolute scan has no such limit.
- **No value diff.** A capture at `LEADERS=9`, with any cross-owner hit
  after frame 300, would turn arms 1–4 and the ally test into a value diff.

## 5. The attrition stamps, `att` and `give_att_disabled`

### 5.1 Leaves

- `attrition_stamp`, +0x1f4, `int`, a sim frame.
- `attrition_stamp2`, +0x1f8, `int`, a sim frame.
- `attrition_stamp3`, +0x1fc, `int`, a sim frame.
- `att`, +0x7f0, `int`, the attrition strength, unscaled.
- `give_att_disabled`, +0x7f8, `int`, a 0/1 flag.

`Leader::init (006e3930)` zeroes all five (the stamps at 6e3dda, 6e3de6 and
6e3df1, the flag at 6e498a, `att` at 6e4984 before its recompute at 6e49bc).
The bulk writers apply as in the header. No constant-slot or absolute store
exists for any of them beyond those named below.

### 5.2 The rule

**`attrition_stamp`.** Writers: `Unit::process_attrition (005e11a0)`
(5e1880 peace arm, 5e1868 grace sub-arm, 5e1586 assassin arm), on the
**unit owner's** slot; zeroings in `Game::do_frame (00591ef0)` 59232b and
`Leader::gain_tech (006dcb60)` 6e038a.

- **Event**: a unit's 32-frame refresh in a rival's land, past every
  exemption before the arms (`docs/ATTRITION.md`'s chain).
- **Arms**: peace, when `!is_enemy(owner, tile)` (both `diplos` non-zero) or
  under CtW (`semaphore[2] & 2`) while `!war_allowed`; assassin, when
  `is_enemy`, `team_style == 2` and `LeaderData::get_target(owner)
  (006da000)` is not the tile owner. Both:

      if stamp[owner] == 0 || frame − stamp[owner] > 900: stamp[owner] = frame

  Strict: exactly 900 does not re-arm. The store is for every slot; only the
  message and sound are the console's.
- **The grace sub-arm is dead with shipped data.** It would store `max(1,
  ally_to_war_grace + frame − 900)` after an alliance broken within
  `ALLY_TO_WAR_DELAY`; both constants are absent from `rules.xml` and load
  as −1, so its test never holds. A mod could open it.
- **Zeroings**, `= 0` for every slot with `leader_flags & 1`:
  `Game::do_frame` on the frame where `rush_rules > 8`, `frame ==
  rush_rules[r].minutes·900` (row +0x3c, stride 0x58) and `team_style ∉ {0,
  8, 11}` (after `Leaders::process_all`, before `Objects::process_all`; it
  then declares war); `gain_tech` when a tech reaches the age-based rush end
  (rules 1..8) while war is barred and `Game::teams_locked`.
- **Readers**: its own gate and `say_no_war`'s `stamp == 0` (5e14f9,
  5e1710), `log_data`. No decision.

**`attrition_stamp2`.** `Unit::suffer_attrition (005e1a10)` 5e1b8a stores
`frame` when the owner's stamp is 0 (5e1a25), for every slot; the console
gates only the message. Its two callers are in `Unit::process (00610bc0)`
(610d35, 6117fe): the period tick, and the detected decoy. It stores before
the damage, so a killing tick stamps. Nothing re-arms it (the rush zeroings
write +0x1f4 only): it is the slot's first attrition tick. Readers: its own
gate, `log_data`.

**`attrition_stamp3`.** `Wall::process (00640450)` 640bb7, on the building
owner's slot; `Build::process (0061edf0)` is its one direct caller.

- **Predicate**: `(frame + id) & 15 == 0`; owner's
  `disable_building_attrition` (+0x804) is 0; also `& 31 == 0` unless rush
  rules still bar war; the land owner `t` (the adjacent-ring owner when the
  type's vslot +0x108 answers non-zero) is `≥ 0`, not the owner, not allied.
- **Unstarted site** (`byte 8 & 2` clear): `disband(0)`, then the store with
  **no `stamp3 == 0` test**, so every ghost site removed re-stamps.
- **Started**: `take_damage(8, …)` via vslot +0x16c; non-zero (it died)
  returns with no stamp; else `if stamp3[owner] == 0: stamp3[owner] =
  frame`.

**`att`.** One store, `Leader::calc_attrition (006cdea0)` 6cdfdc:

    if give_att_disabled != 0: att = 0; return
    n = leading run of ATTRITION1..4 (0x2dd..0x2e0) has_preq holds
    x = n > 0 ? attrition_improved[n − 1] : 0      // C+0x1d8; 1, 2, 4, 8
    for each arm that applies, in order:
        x = (100 + pct)·x / 100                    // signed, truncating
        if x == 0: x = 1
    att = x

Arms: Colosseum `has_wonder(0x212)` (C+0x470, 50); Russians
`has_tribe_bonus(13)` (C+0x74c, 100); CtW, `semaphore[2] & 2` and leader
byte +0x6900 non-zero (C+0xa0c); Kremlin `has_wonder(0x21a)` (C+0x510, 100).

- **The floor fires on a base of 0**: `att` is 1 with no step and any
  applying arm, 0 only with neither or under the flag. Port `== 0 → 1`, not
  `.max(1)` (they differ only for `pct < −100`).
- The decompile's `attrition_upgrade[n + 3]` is the same address; the
  0x2ad test in the loop compares the index (733..736) and is dead.
- `has_tribe_bonus` answers 0 with no starting town and `city_num == 0`, so
  a nomad Russian is `att 0` until its first city.
- **When**: in `Leader::calc_unit_stats (006cf970)`, its first call, on the
  `0x4000000` fold of the next `Leaders::process_all`. The bit's setters:
  `gain_tech`'s tail (every tech but two non-tech exits) and 6dd993,
  `Build::activate`, `Build::close`, `Leader::gather`,
  `Leader::calc_gather`, `Leader::lose_tech`. Also at `Leader::init`, and
  eagerly in `gain_tech` (6dd3d6) **only** in the narrow arm (vslot +0x38
  true, techtype +0x14 == 1, civic level == `ctw_wonder_min_civics`).
- **Timing**: a tech a building completes on frame `f` leaves `att` old for
  the rest of `f`, the end-of-frame dump included; the new value appears in
  `Leaders::process_all` of `f + 1`, before any unit reads it.
- **Reader**: `UnitData::get_attrition (00608fd0)` 608fe6, the only indexed
  read, on the **territory owner's** slot: 0 means no attrition, else the
  divisor after the aged-up scaling (`docs/ATTRITION.md`). So Russian land
  bleeds a hostile unit from frame 0. `LeaderData::get_attrition
  (006e0b60)` has no caller.

**`give_att_disabled`.** `ScenarioFuncSet::enable_give_attrition (00a02900)`
(a02926, 0) and `disable_give_attrition (00a02940)` (a02966, 1), for a
1-based slot with `arg − 1 < 8`, `leader_flags & 1` and `& 2`, else −1 and
no store. Registered only in `ScenarioFuncSet::init_funcs (009c7570)`;
neither recomputes `att`. `Leader::plan_strategy (006b9620)` 6b9bff is a
16-bit store into `reg_land`, **not** a writer (the README's row 5 and
`docs/CHECKSUM.md:1760` are wrong). Readers: `process_attrition` 5e132c on
the tile owner's slot (non-zero exempts every unit on that land at once)
and `calc_attrition` 6cdea6 (`att = 0`). `log_data` does not print it.

### 5.3 In the crate

- **`att`** is `territory.players[w].strength`, written by
  `Sim::calc_attrition` (`crates/sim/src/territory/upkeep.rs:24`) through
  `attrition::strength` (`territory/attrition.rs:42`). Four faults:
  1. ~~**Trap**: `if tech_steps == 0 { return 0; }` (`attrition.rs:43–45`) is
     wrong, as is `docs/ATTRITION.md` §"1. Strength" step 1. Every Russian
     with a city reads 0 here and 1 in the original.~~ Fixed by item 4154:
     a base of 0 and `== 0 → 1` after each arm (`attrition::strength`), and
     `docs/ATTRITION.md` §1 is amended in place.
  2. `StrengthMods` sets `russian` only (`upkeep.rs:43–46`). Colosseum and
     Kremlin can use `tech.rs:600`/`:611` and `Holdings::wonders_held`
     (`economy/holdings.rs:572`); the doc comment at `:20–22` is stale.
  3. **Trap, one frame early**: `Sim::apply_gained` (`tech/gain.rs:601`)
     calls it eagerly at `:608`; `Sim::calc_unit_stats`
     (`economy/rares.rs:679`) does not, so a wonder never recomputes it.
     Drop `:608`; call it first in `calc_unit_stats`, on the fold the crate
     already raises for a wonder (`cities/city.rs:2491`, `:2887`); keep an
     init call.
  4. No `give_disabled` short-circuit.
- **`give_att_disabled`** is `PlayerState::give_disabled`
  (`attrition.rs:257`, default `:272`), read in the original's order at
  `:498`. Nothing writes it; the writer is the trigger track's.
- **`attrition_stamp`**: no store. It goes beside the `Situation` in
  `Sim::process_attrition` (`upkeep.rs:239`), on the `peace_attrition` /
  `assassin_attrition` outcome; `Situation.assassin` is hard-coded `false`
  (`:331`). The timed zeroing is `Sim::end_the_peace`
  (`leaders/diplomacy.rs:74`). **Trap**: its comment at `:87–88` names the
  store `update_leader_list` (that is `ScenarioEditor +0x1f4`); the base is
  the leader slot, so it is this leaf. `docs/AI.md` repeats the mislabel.
  The age zeroing is new in the `gain_tech` port.
- **`attrition_stamp2`**: `Sim::suffer_attrition` (`upkeep.rs:286`) already
  runs `warned_at(who) == 0 → stamp_warned(who)` in place; the port fills
  the stubs (`territory/suffer_view.rs:152–155`) with a per-leader word.
- **`attrition_stamp3`**: both arms of the enemy-territory block in
  `Sim::process_building` (`cities/city.rs:2947`, ~2997–3010). The crate
  runs the 32-frame cadence only (`cities/build.rs:1083`).
- **Checksum** (from the code, not the verdicts): leaves at
  `crates/sim/src/checksum/leaders.rs:69–71, 150, 152`, no value arm; each
  `(unheld)`, count 2, in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1757, 1759–1761, 1791`); each
  port removes its entry.

### 5.4 Reached by a plain game

- `att`: yes, from frame 0 for a Russian with a city; Colosseum, Kremlin
  and the steps when reached. The CtW arm is not.
- `attrition_stamp2`: yes, any slot's first attrition tick.
- `attrition_stamp3`: yes, a started building on ground a rival's border
  took; a ghost site needs a border over an unstarted foundation.
- `attrition_stamp`: the peace arm and timed zeroing need two leaders at
  peace (the "peace pair", `RUSH_RULES 10`, `TEAM_STYLE 1`); the age
  zeroing `RUSH_RULES` 1..8; the assassin arm `TEAM_STYLE 2`; the pre-war
  arm CtW; the grace sub-arm a mod.
- `give_att_disabled`: no, triggers only; 0 for a plain game.

### 5.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/attrition-stamps-*.md`). ~~no diff backs it
yet.~~ Since item 4154 a diff backs `att`'s zero-base floor under the Russian
arm and its zero with no arm. `diff::russian_word::
the_russian_start_holds_att_one_with_no_attrition_tech` checks run647's
start, `FRAME 1` and the closing dump on both leaders. The leaders checksum
line checks `att` on run 1562's frame 91, 0 against 0. The other four leaves
and `att`'s other arms still rest on the readings alone. Since item 4162
the record diff compares `att` on every scored dump that prints the record
(the census at this file's head), which backs its Colosseum and Kremlin
arms and its fold timing (§5.3 faults 2 and 3, closed on run1314b, run762
and run1370b); the four stamps and `give_att_disabled` are not printed or
not compared, and still rest on the readings. Both writer nets were re-run for all five leaves and found no writer
either reading missed.

| leaf | confirmed | corrected | open |
|---|---|---|---|
| `attrition_stamp` | 13 | 1 | 0 |
| `attrition_stamp2` | 6 | 1 | 0 |
| `attrition_stamp3` | 7 | 0 | 0 |
| `att` | 12 | 2 | 1 |
| `give_att_disabled` | 6 | 1 | 0 |

Corrections that changed a rule: the zero-base floor; `att` recomputed on
the next fold, not on the gain; the grace sub-arm dead with shipped data;
`attrition_stamp2`'s crate site is the stubs; `give_disabled` exists;
`plan_strategy` is not a writer. Confidence is high on every leaf (stores,
gates and arithmetic read in the listing).

**Witnesses, not pinned diffs.** run981's start dump
(`~/ron-golden/run981/map-7/gamelog.txt`, Ancient start) prints the Russian
(who 1, tribe 13, `city_num 1`, `tech_frame 0`) at `att 1` (line 822197)
and the Nubian (who 0) at `att 0` (809939), alike in all five dumps to
frame 37; only `n = 0` with the Russian arm and the floor fits. Later
Russian values (2 on run1257, run1148, run1400, run1467; 4/8/16/32 after)
fit the formula. run1072's `attrition_stamp` 1392 on block 1393
(`docs/RUNS.md`) witnesses the peace arm.

### 5.6 Not established

- **`ctw_attrition`'s shipped value** (C+0xa0c; the one open verdict,
  unreachable in v1): read it from `rules.xml`.
- **`gain_tech`'s `russian_attrition_upgrades` arm** (C+0x750), not read to
  the end; it explains run1257's Russian `att 2`. Read that arm.
- **The `0x4000000` setters 5e56cc, 66aee6, 66aef6**, unattributed; they
  only make the refresh more frequent.
- **The one-frame lag in a dump.** The verdicts cite an addendum the file
  does not contain. Read `att` on `f` and `f + 1` where a building
  completes an ATTRITION step.
- **Russian land bleeding from frame 0**: no capture with a foreign unit in
  Russian land before a step was looked up.
- **The assassin arm, the age zeroing, the timed zeroing's 0**: no witness
  (frame 4500 is outside run1072's window). A peace-pair capture across
  4500 and a `TEAM_STYLE 2` capture settle them.
- **`attrition_stamp3`'s 16-frame mask**: decompile only, and unmodelled.
- ~~**No value diff for any leaf**: a `LEADERS` capture diffed on the five
  leaves (run981 for the floor) would make them one.~~ `att` has a value diff
  since item 4154: run647, not run981. run981 is the held-out battery's
  start, and DECISIONS 61 §6 says it is never debugged against. The other
  four leaves still have none.

## 6. Anti-attrition (`anti_att`)

### 6.1 Leaves

- `anti_att`: +0x7f4, `float` (IEEE single). Array form `0xe3ab84 + i·0x6eec`.

### 6.2 The rule

**A pure recompute, no step and no decay.** It is a float, and it is held as
a `u32` of single-precision bits per player, emitted raw at +0x7f4. Every
run starts again from 256.0, so nothing carries over between runs.

**Writers.**

- `Leader::calc_anti_attrition (006cdcc0)` is the only computing writer. It
  has eight stores, and it is the only store of `0x43800000` to +0x7f4 in
  `.text` (no inlined copy). It has three direct callers and no indirect
  one: `Leader::calc_unit_stats (006cf970)`, `Leader::gain_tech (006dcb60)`
  and `Leader::init (006e3930)`.
- `Leader::init (006e3930)` zeroes it twice: by its head memset, and by a
  store of 0 at 6e4955. It then zeroes `take_att_disabled` and calls the
  writer (6e49c3). So init's recompute never takes the disabled exit, and
  it runs after the tribe is known.
- The bulk writers zero it or restore it as the header says.
  `Leaders::clear (006ed5d0)` runs between games only (`Game::close`,
  `Game::run_oos_recovery`). `Leader::close (006b7f80)` holds the same
  memset but is dead: nothing calls it. A load restores the saved bits raw
  through `LeaderData::walk_data (006d6750)`, only when `leader_flags & 1`.

**The recompute**, for leader `w`. The predicates read `leaders.list[w]`
and the stores go to the same record.

1. `x = 0x43800000` (256.0). It is stored **before** the next test.
2. If `take_att_disabled` (+0x7fc) is non-zero: `x = 0`, stop.
3. The Foraging tier is the **highest** held of three, not cumulative:
   - 3 if the leader holds Logistics;
   - else 2 if it holds Supply;
   - else 1 if it holds Forage;
   - else none.

   These are `has_preq(FORAGING_3..1)` on bonus types `0x300..0x2fe`. For
   these three types `has_preq` reduces to `has_tech(preq0)`, and the
   shipped `TECHBONUSES` rows 84, 83, 82 give `preq0` = Logistics, Supply,
   Forage. With a tier, `p = attrition_upgrade[tier − 1]`.
4. The sources apply in this order, each one held applying its own `p`:
   1. the tier;
   2. the Statue of Liberty: `has_wonder(0x219)`, with `liberty_attrition`
      (`Constants` +0x524);
   3. the Mongol bonus: `has_tribe_bonus(17)`, with `mongol_attrition`
      (+0x7f0);
   4. titanium: `has_rare(36)` over `rare | rare_conquest` (bit 30 of
      +0x6da4 or of +0x6dcc), with `titanium_attrition` (+0x95c).

   For each held source:
   - if `p ≥ 100` (a **signed** compare), then `x = 0` and stop;
   - otherwise
     `x = Single(x).mulss(Single::from_bits(0x42c80000)).divss(Single::from_i32(100 − p))`.
     Here `0x42c80000` is 100.0, read from `.rdata` at `0xb696ac`. All of
     it is scalar SSE single, round to nearest even: the executable has no
     `ldmxcsr`, and its one `_controlfp_s` call sets only the x87
     precision control.

The order shows in the bits, not the rational: Mongol before Forage 1
gives `0x442aaaab`, the writer's order `0x442aaaac`.

**What the predicates read:**

- `has_wonder`: an entry of this leader's wonder list that is live (+0xc
  & 1), of the type, and (any type but `0x216`) whose object's city short
  +0x72 is ≥ 0; or a CtW `conquest_wonders` bit at Civic ≥
  `ctw_wonder_min_civics`.
- `has_tribe_bonus`, first match: 0 under `info.flags & 4`, or no starting
  town and `city_num == 0`, or `tribe < 0`; 1 on a CtW
  `conquest_racial_powers` bit; 0 if `leader_flags2 & 0x40`; else the
  tribe's bonus index (`tribe·0x5f0 + 0x54`) is 17.

**When it runs.**

- At `Leader::init`.
- At the head of `calc_unit_stats`, after `calc_attrition`. It is reached
  by the per-frame fold in `Leaders::process_all (006ed2a0)` when
  `leader_flags & 0x4000000`, and the fold clears that bit. Seven sites
  set it:
  - `gain_tech`, twice: its tail and its unit-type arm;
  - `lose_tech` (site 6d2afd);
  - `gather` (site 6ce3d0);
  - `calc_gather` (site 6cf5ea);
  - `Build::activate`, **for a wonder only** (types `0x20e..0x21e`; every
    other building jumps past the set);
  - `Build::close`.
- Eagerly from `gain_tech` (6dd3cf), **only** in one arm. The gained type
  must answer `TypeData::is_epoch_type`, its techtype `+0x14` must be 1,
  and the decrypted Civic level must equal `ctw_wonder_min_civics`, which
  ships as 4. That arm also runs `calc_pop_cap`, `calc_attrition` and
  `Region::fix_borders`. Its value equals the next fold's, so it moves the
  leaf early only when a source arrived after that frame's fold.
- From `set_age (006d25a0)` / `set_epoch (006d26f0)`, and from the
  `ScenarioEditor::*` / `ScenarioFuncSet::*` functions, through direct
  `calc_unit_stats` calls.

**Timing.** Research finishes in `Objects::process_all`, after
`Leaders::process_all`, so a tech gained on frame F shows from F+1.

**Readers that change a decision.**

`UnitData::get_attrition (00608fd0)` reads the unit's **owner's** leaf. Its
non-militia arm is:

```
trunc(bits_as_f32 · k / 256)
k = 256, or 512 for siege   (25600 / (100 − siege_attrition))
```

The two multiplies are exact (k and 2^-8 are powers of two). Militia
bypass it.

The idle immunity that follows is `has_preq(0x2fe)`, which means "holds
Forage". It holds at tiers 2 and 3 as well, not at tier exactly 1.

`LeaderData::get_anti_attrition (006e0b70)` has no caller. `log_data`
prints the leaf.

**Float class (`docs/DECISIONS.md` entry 120 §3)** is class 1 or class 2,
never class 3:

- **Class 1** on a frame where no walked leader holds Forage (or above),
  the Statue, the Mongol bonus or titanium. The value is the constant
  `0x43800000`, from the store at 6cdcce.
- **Class 2** in general. Every input is an integer the simulation holds:
  four predicates and four `Tuning` percentages. The chain needs only
  `Single::from_bits`, `from_i32`, `mulss` and `divss`, all already in
  `num::single::Single` and correctly rounded. Nothing joins
  `no_float.rs`'s allowances.

The chain is the port (it serves a modded `rules.xml`); the table is its
test.

**The nine bit patterns over the shipped data** (`attrition_upgrade` 25,
50, 75; Liberty 100; Mongol 50; titanium 50). M and T are the Mongol bonus
and titanium, and "one" means either of the two. Read the cells across, in
the order "neither / one / both".

| tier | M, T: neither / one / both | reader's integer, k = 256 / 512 |
|---|---|---|
| none | `0x43800000` / `0x44000000` / `0x44800000` | 256/512, 512/1024, 1024/2048 |
| 1 (Forage) | `0x43aaaaab` / **`0x442aaaac`** / **`0x44aaaaac`** | 341/682, 682/1365, 1365/2730 |
| 2 (Supply) | `0x44000000` / `0x44800000` / `0x45000000` | exact |
| 3 (Logistics) | `0x44800000` / `0x45000000` / `0x45800000` | exact |
| Liberty held, or disabled | `0x00000000` | 0 |

On the two bold cells the bits are one ulp above the nearest single to the
exact rational, which is `0x442aaaab` and `0x44aaaaab`. In all 32 states
and for both values of k, the reader's integer equals the floor of the
exact rational. So the reader may stay on the rational.

### 6.3 In the crate

- **The rule exists unwired** as an exact rational:
  `territory::attrition::resistance` with `ResistanceMods`
  (`crates/sim/src/territory/attrition.rs:102, 132`), called only from its
  tests; `PlayerState.resistance` defaults to `Resistance::BASE` (`:268`).
- **The home is `Sim::calc_unit_stats`** (`crates/sim/src/economy/rares.rs:679`).
  It is run by the fold on `roster.unit_stats_dirty`
  (`core/frame.rs:186-189`). Its module doc (`rares.rs:47-48`) names
  `calc_attrition` / `calc_anti_attrition` as calls it does not make.
  Place the recompute after `calc_attrition`, and make sure a wonder's
  activate or close raises the dirty flag.
- **The trap: do not recompute in `Sim::apply_gained`** (`tech/gain.rs:601/608`),
  except the Civic-4 epoch arm above. An eager call there is one frame
  early on every frame a tech finishes. The same holds for the crate's
  existing eager `calc_attrition` at `gain.rs:608`, which is §7's to
  settle.
- **Init.** The `Leader::init` analogue recomputes after the tribe and the
  starting techs are set.
- **Stale docs:** `attrition.rs:76-79` ("the product … is never an
  integer": 256 is); `attrition.rs:100-101` (order unobservable holds for
  the rational, not the bits); `calc_attrition`'s "holds no wonder a player
  owns" (`PlayerTech::has_wonder`, `tech.rs:814`, and `Sim::wonders_held`,
  `economy/holdings.rs:572`, exist and `player_borders` reads them).
- **`TechTree::has_tribe_bonus`** (`crates/sim/src/tech.rs:1102`) holds the
  first arms and the index compare. It has no `leader_flags2 & 0x40` arm.
- **`PlayerState.foraging_1`** (`attrition.rs:261`) must come from "holds
  Forage", not from the tier. The tier itself is
  `Sim::bonus_level` (`economy/rares.rs:84`) over `bonus_at(82..84)`, the
  same shape as `ATTRITION1..4` (`crates/rondata/src/load.rs:1932`).
- **The checksum.** The leaf is `("anti_att", 0x7f4, 4, 1)` at
  `crates/sim/src/checksum/leaders.rs:151`, with no value arm. `Leaf::bytes`
  emits 0 for it (`:401-409`), which is wrong for every walked leader not
  under Liberty or the disabled flag. A port gives it a value arm and
  removes `("anti_att (unheld)", 2)` from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1756`).

### 6.4 Reached by a plain game

**Yes.** In a skirmish against the computer, every leader is computed at
init:

- 256.0;
- or 512.0 for the Mongols, unless "No Nation Powers" is set or the start
  is nomadic. A skirmish with starting towns passes the nomad arm while
  `city_num` is 0.

After init, the plain-game sources are Forage / Supply / Logistics, the
Statue, titanium, and the Civic-4 epoch arm.

Out of reach: `take_att_disabled` (triggers only:
`ScenarioFuncSet::enable_take_attrition (00a02880)` /
`disable_take_attrition (00a028c0)`, no recompute); `rare_conquest`,
`conquest_wonders`, `conquest_racial_powers` (CtW); the editor and
`ScenarioFuncSet` callers; `Leaders::clear` (between games); the load walk
(save track).

### 6.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/anti-att-*.md`). **No diff backs the
class 2 rows yet.** The class 1 value `0x43800000` was read off run 1562
frame 91 and run1100 frame 5408 (entry 120 §3.4). ~~That reading is a
witness, not a pinned diff.~~ Since item 4162 the record diff compares
`anti_att` on every scored dump that prints the record, and the leaders
checksum line holds it on run 1562's frame 91 (`0x43800000` both sides);
a class 2 row is backed wherever a scored dump reaches one. Its reader
(`get_attrition`'s divisor) is not wired, parked 4162b.

Verdicts: 33 confirmed, 6 corrected, 3 open (§6.6 lists four). The
corrections that changed a rule: the tier is "holds Logistics / Supply /
Forage", not a tech named FORAGING_n; `Build::activate` dirties for
wonders only; no eager recompute on gain but the Civic-4 arm; nine
patterns, not eleven; class 1 or 2, not 3; the crate does hold wonders.

Confidence:

- **High** on the writer set, exits, order and arithmetic (listing read
  whole, both constants from `.rdata`, bits replayed as f64 and as exact
  rationals rounded per op, agreeing on all 32 states); on the bonus-type
  reading (decompile plus shipped rows); on the timing (complete callers).
- **Medium** on the Mongol init value: the caller's store of `tribe` was
  not traced.

### 6.6 Not established

- **Whether a nomad Mongol's first city raises `0x4000000`.** If it does
  not, `has_tribe_bonus` flips with no recompute. To settle it, grep the
  city founder (`City::init`, `Build::activate` for a city centre) for a
  leader-flags write.
- **Whether `PlayerTech.wonders` carries `has_wonder`'s `+0x72 ≥ 0`
  (captured-city) predicate** for a captured Statue. To settle it, read
  the crate's wonder bookkeeping on a city capture.
- **Whether a lobby's later starting age reaches `set_age`** (and so
  `calc_unit_stats`): read the lobby's age setup.
- **What sets `leader_flags2 & 0x40`**: grep the stores to that flag.
- **The Mongol init value of 512.0** (medium). Check it against a capture
  whose walked leader plays the Mongols.

None changes a plain game's first frames. A capture whose walked leader
researches Forage then Supply or Logistics, or plays the Mongols, would
back every row of the pattern table at once.

## 7. Diplomacy: relations, declarations and hires

### 7.1 Leaves

- `good_deeds`, +0xd4, `int[8]`, by target; the valuation's own units.
- `hire_stamp`, +0x1b4, `int[8]`, by the hired-against slot; a frame.
- `hire_who`, +0x1d4, `int[8]`, by the hired-against slot; a slot, −1 none.
- `blacken`, +0x20c, `int`; a count of repeat declarations.
- `dow`, +0x270, `int[8]`, by target; a count.
- `broke_alliance`, +0x2b0, `int[8]`, by target; a frame.
- `made_peace`, +0x2d0, `int[8]`, by the other party; a frame.
- `last_spoke`, +0x2f4, `int[8]`; always 0.

`frame` is the sim frame. `Leader::init (006e3930)` resets every leaf for
every slot (`Leaders::init` calls it for all ten, with no exit before its
per-slot loop): all to 0, except `hire_who[j] = −1`. `Leaders::clear`'s
memset leaves 0 in all, `hire_who` included, until the next `Leader::init`.
`Leader::reset_score (006e37f0)` touches none. Nothing decays.

### 7.2 The rule

Three writers own the seven live leaves: `Leader::action_declare
(006dab50)` for `dow`, `blacken` and `broke_alliance`;
`Leader::action_respond (006d03c0)` for `made_peace`, `hire_stamp` and
`hire_who`; `Leader::diplomacy (006bc950)` for `good_deeds`.
`LeaderData::log_data (006e5110)` prints all eight, so all are in the
`LEADERDATA` dump.

**`action_declare(me, t, level, quiet, force)`**, in order:

1. return if `t` is already an enemy;
2. turn an allied pair's request into level 1;
3. refuse inside `ALLY_TO_WAR_DELAY` (signed); call `afford_dow` only when
   `quiet == 0`; refuse, or force level 1, on `war_allowed`;
4. **the counting block, only when `quiet == 0`:**
   - block A: if `level == 0` and (`allied(me, t)` or `invaders[me][t] <
     5`, signed): `if dow[me][t] != 0 { blacken[me] += 1 }`, then
     `dow[me][t] += 1`;
   - block B: if `allied(me, t)` still holds: the same pair again.

   `blacken` reads `dow` before that block's own increment. A forced level 0
   on an ally would count twice; no caller reaches it, since the only
   `force = 1` caller, `DropControl::process_drop`, passes `quiet = 1`;
5. **for any `quiet`:** if the pair is still allied both ways,
   `broke_alliance[me][t] = frame`. The target's mirror slot is not written;
6. `set_diplo(me, t, level)`.

**Who passes `quiet = 0`**, from all 16 call sites: only the human's
Declare (`CommandPackage::process_declare`, 9473aa) and a human party's leg
of the hire loop below. `Object::take_damage` and `Nuke::do_damage` pass
`!(declarer & 4)`, but each skips a human declarer before the call, so they
never pass 0. `Leader::diplomacy`, `action_respond`'s treaty call
(6d04ed), `DropControl` and `ScenarioFuncSet::declare_war` pass 1. So
`dow` and `blacken` move only for a human declarer. `broke_alliance` moves
on any break through `action_declare`: `diplomacy` 6bed1c, a level-0 call
on a mutual ally, 6d04ed, the human's Declare, `DropControl`. The 6beceb
call is dead (it needs `t == me`, which the per-target loop skips). The
original's `Game::do_frame` peace-ending and the console verb call
`set_diplo` directly and must not stamp.

**`action_respond(responder, proposer, response)`**, apply arm, after the
goods exchange (§8):

- **Treaty.** `w = responder.dip[proposer].treaty` (+0x6934). If `w != −1`:
  `set_diplo(responder, proposer, max(w, diplos[responder][proposer]))`,
  then the same for `(proposer, responder)`, both signed. If `w == 1`, both
  `responder.made_peace[proposer]` and `proposer.made_peace[responder] =
  frame`, even when the `max` keeps the pair allied.
- **Hires**, reached with or without a treaty, only if `war_allowed`. For
  each `X` in 0..8, `X ∉ {responder, proposer}`, with
  `responder.dip[proposer].attacks[X] != 0` (+0x6968):

      hire_stamp[responder][X] = hire_stamp[proposer][X] = frame
      hire_who[responder][X] = proposer
      hire_who[proposer][X]  = responder

  then responder, then proposer, if not already an enemy of `X`,
  `action_declare(party, X, 0, quiet = !human(party), 0)`. The stamps do
  not depend on the declarations. `attacks[]` is written into both records
  by `Leader::action_attack (006d14e0)`.

**`good_deeds`**, in `diplomacy`'s offers arm (`me` computer, `t` other):

- The valuation's score `S` is seeded with `good_deeds[me][t]` and
  accumulates the deal (resource value × amount >> 8, flat 25/50/100 steps).
  An early-game multiplier on asked goods applies: ×4 before frame 4500,
  ×3/2 (truncating) before 9000 when the pointer at `-0x40` reads 0. The
  over-ask flag `-0x50` is set when an ask exceeds my decrypted stock.
- `need = −S`; under `treaty == 2`, `need = (my epoch + t.blacken + 1) ×
  (100, or 50 under semaphore[0] & 4) × ((*-0x48 != 0) + 1) − S'`, with
  `S' = S + epoch[t]·{0, 50, 100}` plus adjustments. Then `need −= 99` if
  `counteroffer[me][t] != 0`.
- Accept, on `need <= 0` with the over-ask flag clear: `counteroffer[t] =
  0`; **only if `any_proposals(t)`**: `action_respond(t, 1)`, `need += 50`
  if `weak[t] != 0` and `!is_tribute_period(leaders[t], me)`, then
  `good_deeds[me][t] −= need`. Without a pending proposal it returns
  before the store.
- So the balance enters twice: an accepted deal leaves `2·good_deeds +
  surplus (+99 for a pending counteroffer) (−50 when weak)`. Port it
  doubled; it is what the listing does.

**`last_spoke`** has no writer and no sim reader; carry 0.

**Readers that change a decision.**

- `dow`: own diagonal `dow[me][me]` floored at 1 divides `weak` (6bcf0a);
  `leaders[t].dow[me] < 2` gates the peace arm for a human target (6bd871);
  `< 2`/`< 3` lift the alliance valuation by +100s (6bfdcd); `!= 0` picks a
  taunt (6c00cd). 6c0966 reads the diagonal too (always true for a
  non-self target).
- `blacken` (always `leaders[t].blacken`): gates `< 4` (peace arm 6bd87e,
  demand arm 6be667), `< 2` (race coin 6bdc3b), `>= 2` (alliance logic
  6c0b1a), `== 0` (text only, 6c017e); factors `(blacken + 1)` in the demand
  size `(25·epoch + 25)·(blacken + 1)` (6be7ec) and the alliance `need`
  (6bfcf4); `(blacken != 0) + 1` in two cost terms (6bf8d5, 6bf940);
  `consider_tribute`'s `100·blacken` floor for a computer receiver (§8).
- `hire_stamp`/`hire_who`: `me.hire_stamp[t] != 0 && frame − it < 13500`
  forces `aggression[t] = 1` (6bdc8a); the mine coin, drawn first, clears
  aggression only if `hire_stamp[t] == 0` or `frame − it > 4500` (6bdce2);
  `t` my ally, `me.hire_who[l] == t`, `frame − me.hire_stamp[l] <= 9000`, I
  at war with `l` and `t` not, forces `aggression[t] = 1` (6bdd87);
  `hire_stamp[t] != 0 && frame − it <= 13500` diverts the demand arm to war
  (6be680, §8). The arm at 6bdd57 compares `leaders[l].hire_who[me]` with
  `l` itself, a test no deal can pass; port it as written.
- `broke_alliance`, `made_peace`: `frame − max(mine[t], theirs[me])` against
  `ALLY_TO_WAR_DELAY` in `action_declare`'s refusal and `Unit::process_attrition
  (005e11a0)`; also `Options::do_declare_war (007216b0)` and
  `IFaceMainBase::draw_timers (0080b060)`. `ALLY_TO_WAR_DELAY` and
  `_GRACE` load −1 (`Constants::init (00569a90)` defaults; no shipped XML
  names them), so no outcome depends on either leaf; both are dump and
  checksum words a port still writes.
- `good_deeds`: only the next valuation's seed.

### 7.3 In the crate

- **No writer exists.** No `action_declare`, `action_respond`,
  `Diplomacy` record or hire model. `Sim::set_diplo` is
  `crates/sim/src/leaders/relations.rs:116`, `Sim::declare_war` `:294`.
  The counting and `broke_alliance` go in a new `Sim::action_declare` beside
  `set_diplo`, never in `set_diplo` (whose `_` arm an accepted treaty also
  calls), and never in `end_the_peace` or the console verb.
- **SEAMs** in `crates/sim/src/leaders/diplomacy.rs`: `dow` at `:305`,
  `dow`/`blacken` at `:398`, `let blacken = 0` at `:444–445` (the race
  coin), the hire readers at `:453`, the offers arm (`good_deeds`) at
  `:490` inside `diplomacy_with` (`:174`).
- **Traps.** `diplomacy.rs:453–454` says two hire arms; three read the
  words there (a fourth, 6be680, is in the demand arm).
  `crates/rondata/src/diff/leader.rs:47` labels `blacken` "fog
  bookkeeping"; it is a human's repeat-declaration count.
  `crates/sim/src/combat/take_damage_view.rs:34` names `0x4` `LEADER_OUT`;
  it is HUMAN (its uses are right: "skip a human declarer").
- **Checksum.** Rows `crates/sim/src/checksum/leaders.rs:60, 67, 68, 75,
  79, 81, 82, 84`, answered by `leader_long`'s `_ => None` (`:1305`). Each
  is an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1767, 1768, 1784, 1792,
  1795, 1796, 1806, 1812`; lines and counts read from the code, not the
  verdicts). The answer is a constant for all eight in an all-computer
  style-1 game: 0, and `hire_who` −1 per slot after `Leader::init`.

### 7.4 Reached by a plain game

**An all-computer skirmish at team style 1 reaches none of the event
writers.** No `action_respond` is ever called there (§8.4), so
`made_peace`, `hire_stamp`, `hire_who` and `good_deeds` (which needs
`any_proposals`) stay at their reset. No computer passes `quiet = 0`, so
`dow` and `blacken` stay 0. `broke_alliance` is open (7.6).

With a human: his Declare counts `dow`, and a second counted declaration on
the same slot counts `blacken` (run1078, style 0, shows `dow[1]` 0 → 1). A
human proposal or answer reaches `made_peace` and `good_deeds`; an executed
attack deal with three or more leaders and war allowed reaches the hires,
where a human party's declaration also counts. Team styles 0/8/11 reach the
deal writers without a human. `ScenarioFuncSet::*` is triggers. No Conquer
the World or multiplayer path is needed.

### 7.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/diplomacy-relations-*.md`); no diff backs
it yet. Verdicts: 87 confirmed, 12 corrected, 2 open.

Corrections that changed a rule:

- `take_damage` and `Nuke::do_damage` never pass `quiet = 0`: only the
  Declare command and a human hire party count `dow`/`blacken`.
- 6c0966 reads the diagonal `dow[t][t]`, not `dow[t][me]`.
- `blacken`'s readers re-mapped by address: 6be667 is a `< 4` gate and
  6be7ec the demand size; the `(blacken != 0) + 1` terms are 6bf8d5/6bf940;
  6bfcf4 is the alliance `need`.
- `diplomacy` 6beceb is dead; the live explicit break is 6bed1c.
- The 6bdd57 hire arm is a test that cannot pass, not a live third-leader
  rule; 6bf7a5 reads `diplos[k]`, not `hire_who`.
- `good_deeds` is in the dump.

Confidence is **high** on every store, gate and reset, each read in the
listing. It is **medium** on the full composition of `good_deeds`'s
valuation `S`, not walked term by term.

Witness, **not a pinned diff**: run1078 (style 0, a human Declare), where
the dump's `dow[1]` moves 0 → 1 (`crates/rondata/src/blind.rs:333–343`). No
`rondata::diff` row compares `dow`.

### 7.6 Not established

- **Declare on a locked teammate.** Whether the interface offers it under
  style 1. If it does, a human's Declare on an ally passes, becomes level 1,
  counts `dow` once (block B) and stamps `broke_alliance`. Reading
  `IFaceDiplo*`'s Declare enabling, or one capture, settles it. A self-target
  would also count the diagonal; no sim path sends one.
- **`broke_alliance` at all-computer style 1.** 6bed1c is in the demand arm,
  gated off at style 1. Whether the level-0 sites 6be1ef, 6be210 (a target
  not at war, rush rules) or 6c04df can name a mutual ally was not traced.
  Reading 6be067–6be1ad's entry, or a capture with a team and rush rules,
  settles it.
- **`good_deeds`'s `S`** term by term, and the scenario fixed-answer mode
  that skips it (`semaphore[2] & 2` read; the `semaphore[1] & 0x10` half
  not); both belong to the offers-arm port.
- **Which offers carry `treaty == 1`** (`action_treaty`, `action_offer`):
  §8's port.
- `walk_data` and `load_leaders_chunk` copy `last_spoke` and the rest on a
  load; neither originates a value.

## 8. Diplomacy: tribute, gifts and counteroffers

### 8.1 Leaves

- `tribute_stamp`, +0x174, `int[8]`, receiver's record, by giver; a frame.
- `gift_stamp`, +0x194, `int[8]`, giver's record, by receiver; a frame.
- `counteroffer`, +0x314, `int[8]`, indexed by the proposer; 0/1.
- `tribute_demanded`, +0x334, `int[8]`, indexed by the target; 0/1.
- `tributes`, +0x498, `int[6]`, one per good; whole resource units.
- `tribute_sent`, +0x860, `int`; whole units, unscaled.
- `tribute_received`, +0x864, `int`; whole units, after the giver's scale.

`frame` is `Game::frame`; every comparison is signed.

### 8.2 The rule

**The deal record.** Each leader holds `dip[8]` (`Diplomacy`, at +0x692c):
`agree` +0, `any_offer` +4, `treaty` +8, `offers[6]` +0xc, `dows[6]` +0x24,
`attacks[8]` +0x3c. `Leader::action_offer (006d1780)` adds an amount to
`A.dip[B].offers[r]` and subtracts it from `B.dip[A].offers[r]`. A positive
slot in `A.dip[B]` means "A gives B". `action_tribute` and
`action_demand_tribute` route through it; none writes a leaf here.
`Leader::init` sets every record to `Diplomacy::clear_all (0047e030)`
(`treaty = -1`).

**Setup writers.** `Leader::init (006e3930)` zeroes `tribute_stamp`,
`gift_stamp`, `counteroffer`, `tribute_demanded` and `tributes` in two loops
that run whatever its tribe argument, after the memset branch. It also calls
`Leader::reset_score (006e37f0)` (also called by `Setup::build_game
(005ac190)`), which zeroes `tribute_sent` and `tribute_received`. With the
header's bulk writers, these are the only resets; none runs in a match.

**`Leader::action_respond (006d03c0)`**, `T` answering `P` with 0
(withdraw), 1 (accept) or 2 (reject), the only `action_*` storing a leaf.

1. **Entry, every code:** `T.tribute_demanded[P] = 0` and
   `T.counteroffer[P] = 0`, before any branch.
2. **Commit** (code 1, `T.dip[P].agree != 1`). For each good both leaders
   can use, `T` must hold `bucket[r] >= max(offers[r], 0) + price[r]`, or the
   accept is refused. Then, for each such good with `o = T.dip[P].offers[r]
   > 0`:

       bucket[r] = max(bucket[r] − o, 0)
       escrow[r] = max(escrow[r] − o, 0)
       tributes[r] += o            (the unclamped o)

   A second pass does the same with `price[r]`, an attack request's price
   in `T.dip[P].attacks[]` (0 unless `T` is human, `leader_flags & 4`), and
   stores `T.dip[P].dows[r] = price[r]`. Then `agree = 1`. The deal executes now if `P` had agreed, or
   if it asks nothing of `P`; otherwise it waits.
3. **Exchange** (any treaty but 2). Each side's `dows[r]` is first credited
   back to its own bucket and zeroed. Then, per good both can use, with
   `gT = max(T.dip[P].offers[r], 0)` and `gP = max(P.dip[T].offers[r], 0)`:

       T.bucket += scale(P, gP);  P.tributes[r] −= gP
       P.tribute_sent += gP;      T.tribute_received += scale(P, gP)
       consider_tribute(T, P, gP, r)
       P.bucket += scale(T, gT);  T.tributes[r] −= gT
       T.tribute_sent += gT;      P.tribute_received += scale(T, gT)
       consider_tribute(P, T, gT, r)
       T.tributes[r] = max(.., 0); P.tributes[r] = max(.., 0)

   Then `Diplomacy::clear_all` runs on both records (no ledger touched).
   **The `dows` residue:** the refund never lowers `tributes`, so a human's
   paid attack price stays in `tributes[r]` until the next `Leader::init`.
   No decision reads it, but the dump and the checksum do, so a port keeps
   it. The second `dows` zeroing hits `P.dip[P].dows[r]` (0 over 0); a port
   may zero `P.dip[T].dows[r]` instead, or nothing.
4. **`scale(G, x)`** is `LeaderData::scale_tribute (006d5240)` on the giver:
   `pct = epoch[2]·commerce_tribute + base_tribute`, floored at 1, where
   `epoch[2]` is the Commerce line. `pct >= 100` returns `x`. Otherwise it
   returns `(x·pct + c)/100`, with `c` 0 for `x < 20`, 50 for `x < 100`,
   else 99.

**`Leader::clear_agree (006d1af0)`** (me, partner `p`), called on every
offer change and by `action_respond`. When `me.dip[p].agree == 1`, per good
`min(tributes[r], dows[r])`, then (if `offers[r] > 0`) `min(tributes[r],
offers[r])`, move from `tributes` to the bucket; `escrow` is not refunded. Then `clear_dows`, and `agree = any_offer =
0`.

**`consider_tribute (006d1360)`** (receiver `r`, giver `g`, `amount` the
gross give before `scale`, `good`), called only by the exchange, after the
receiver's credit:

- If `amount <= 0`, it does nothing.
- `gift_stamp[g][r] = frame`, unconditionally.
- `tribute_stamp[r][g] = frame` if `r` is human, or if all of these hold:
  `amount >= 100·blacken[g]` (+0x20c); `r`'s post-credit `bucket[good] <
  3000`; and `good == WEALTH` or `econ[r][good] & 4`.

**`Leader::process_taunt (006b8cc0)`** (§9 has its gate) writes `gift_stamp`
too. Taunts 1..5 ask for goods 0, 1, 2, 4, 5. Both leaders must have
`type_avail(g,1) == 4`. The ask is refused within 4500 frames of the last
gift, under the lock test next. If the computer's stock is `> 149`, and
`teams_locked == 0` or it is neutral, it sets `gift_stamp[me][from] = frame`.
It then runs `action_clear_all`, `action_offer(from, g, stock/3)` and
`action_respond(me, from, 1)`: a pure gift, executed at once, stamping
again through `consider_tribute`.

**`Leader::diplomacy (006bc950)`** (computer `me`, other `k`) stores only
these two.

- **`counteroffer`.** Only when `k.dip[me].agree == 1`, the evaluation
  builds a shortfall `s` (formula not read here). When `counteroffer[k]` is
  already set, it takes 99 off `s`.
  - If `s > 0`, no good of mine in the deal is positive, and none exceeds my
    stock: `counteroffer[k] = 1`. It asks for the good with the best
    `resource_value` (at least 0x40; never KNOWLEDGE; default WEALTH),
    rounded up to 100, and calls `clear_agree` both ways.
  - `s <= 0`: accept if `any_proposals`, charge `good_deeds[k]` (+50 if
    `weak[k]` and no tribute period), `counteroffer[k] = 0`.
  - Reject: `k.got_diplo_message = frame`, `counteroffer[k] = 0`, then
    `action_respond(me, k, agree == 1 ? 0 : 2)`.
  - At style 0/8/11 (or 7's case), `num_allies == 0`, no victory timer,
    both computers, `aggression[k] == 0`: `counteroffer[k] = 0`.
- **`tribute_demanded`.** The demand arm runs at team style 0/8/11, or
  style 7 with team 8; any other style skips it. It needs peace both ways,
  `aggression[k] != 0`, the rush rules passed, `!is_tribute_period(me, k)`,
  and `frame >= (3·d + ((seed + who) & 7))·300`. Here `d` is `4 + seed%3`
  for a rusher, `8 + seed%6` for a boomer, else `6 + seed%5`. It declares war
  instead if any of these holds: the cleared-agreement flag; `k.blacken >=
  4`; local `-0x30`; `hire_stamp[k] != 0` and `frame − hire_stamp[k] <=
  13500`; or `k.bucket[WEALTH] < D`, with `D = (epoch + 1)·(k.blacken +
  1)·100`, `epoch` read from `me`'s `data_encrypted + 0xe8` xor 0x63187.
  Otherwise it composes a demand (a random start; the non-WEALTH,
  non-KNOWLEDGE goods `me` is short of; `−min(k stock, D)`; a WEALTH ask),
  calls `action_respond(me, k, 1)`, and then sets `tribute_demanded[k] = 1`.
  It stays 1 after `k` pays, until `me`'s own next `action_respond(me, k,
  *)`.

**Readers that change a decision.**

- `is_tribute_period (006d5b90)` reads `tribute_stamp`: over slots `i`
  (not `me`, `leader_flags & 1`), the first that qualifies (`i == p`;
  `is_ally(L[i], p)` after frame 0, equal lobby team `0..3` at frame 0; not
  a team-8 `i` or `p` under style 7) decides: stamp 0 → 0, `frame − stamp <
  3000` → 1. Callers: `diplomacy` ×3, `Army::do_marching (006f3df0)`,
  `Army::find_target (006f69b0)`.
- `gift_stamp`: the taunt refusal, and `diplomacy`'s ally pricing: my
  give to an ally up to `stock/3` (×4 before frame 4500, ×3/2 before 9000,
  under a flag) is free at style 0/8/11, else free unless `gift_stamp[k] !=
  0` and `frame − gift_stamp[k] < 4500`, which charges
  `resource_value·ask/256`.
- The cadence of `diplomacy` reads `counteroffer` and `tribute_demanded`.
  The period `P` starts at 0x800. When `me.dip[k].agree != 1`, it becomes
  0x2000 if `tribute_demanded[k]`, then doubles if `counteroffer[k]`. If
  `agendas[k] & 8 == 0`, it shifts right by 2 (unsigned). The pass fires
  when `(P − 1) & ((k + 8·me)·(P >> 6) + frame) == 0`. A `tribute_demanded`
  also picks the war message.
- `tributes`: `clear_agree`'s caps and the exchange clamps.
  `LeaderData::log_data` dumps it (`6e7515`).
- `tribute_sent`, `tribute_received`: only `AchieveChartPane::set_chart_data
  (007f2c40)`, no sim reader.

### 8.3 In the crate

- **No writer exists**: no `Diplomacy` record, `action_respond`,
  `clear_agree`, `consider_tribute` or `scale_tribute`, and neither tribute
  constant is loaded. The commit debits the ledger's `escrow`
  (`crates/sim/src/economy.rs:687`).
- **Reader stubs:** `tribute()` returns `false` at
  `crates/sim/src/ai/army.rs:3262` and at
  `crates/sim/src/ai/army/target_gate.rs:523` (the `find_target` gate). One
  `is_tribute_period` port replaces both.
- **SEAMs:** `Sim::diplomacy_with` (`crates/sim/src/leaders/diplomacy.rs:177-179`)
  states that `agree` is never 1 and that neither flag is set. The cadence
  at `:180` omits both terms. The style-0/8/11 return at `:470` is where
  the demand arm goes.
- **Checksum.** The leaves are `crates/sim/src/checksum/leaders.rs:65, 66,
  85, 86, 118, 178, 179`, answered by `leader_long`'s `_ => None` (`:1305`).
  Each is an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1774, 1790, 1841-1845`; count
  read from the code, not the verdicts); a port removes them. By 8.4, 0 is
  exact for a style-1 game with no human. `coverage.rs:295` lists `tributes`,
  not `tribute_sent`/`tribute_received`.
- `process_taunt`'s `gift_stamp` waits on §9's chain.

### 8.4 Reached by a plain game

**An all-computer skirmish at team style 1 reaches no event writer**, so
all seven leaves read 0 throughout and the crate's stubs are exact. Command
routes are interface code, the taunt route needs a human's chat,
`ScenarioFuncSet::*` is triggers, and `diplomacy` originates no deal at
style 1 (demand `6be556`; alliance offers `6c09c8`; peace arm `6bdf56`
with `6c0a39`, as every enemy has `aggression[k] = 1`).

The leaves move with a human (a diplomacy-window deal, or a resource taunt
to an allied computer at any style), or at team style 0/8/11, human or
not. The `dows` residue needs a human. No Conquer the World or multiplayer
path is needed.

### 8.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/diplomacy-tribute-*.md`); no diff backs it
yet. Verdicts: 63 confirmed, 4 corrected, 4 open.

Corrections that changed a rule:

- `counteroffer = 1` needs no positive give of mine and none over my stock,
  not a "clear everything" flag.
- The demand's war test and `D` use `k.blacken`, not a counter.
- `log_data` dumps `tributes`.

Confidence is **high** on every store and predicate, each read in the
listing. It is **medium** on `scale_tribute`'s constants.

Witness, **not a pinned diff**: run1086's taunt gift (`docs/audit/parked-archive.md`
3470a), where the ally's `gift_stamp[0]` reads the frame and every
`tributes` and `escrow` slot reads 0 at frame end (commit and exchange run
in one call).

### 8.6 Not established

- **The shortfall formula** (decompile `:2676`–`:2777`) is unread, and so is
  the `6c035d` arm (a positive shortfall with a positive give of mine),
  which stores no leaf here.
- **Unnamed:** the demand's local `-0x30`, the epoch word at
  `data_encrypted + 0xe8` (`epoch[0]`?), and the pricing's scaling flag.
  Reading each one's writer settles it.
- **`scale_tribute`**: the load scale of `commerce_tribute` and
  `base_tribute` (`Constants::init`), and its rounding branches, which were
  not re-listed. A deal captured above Commerce 0 pins them.
- **run1086** (the zeros; a predicted human `tribute_stamp[1] = 661`): a
  packet check makes them a diff.
- **`walk_data`** in load mode was not re-read.

## 9. The taunts

A human's taunt-order chat line reaches an allied computer through an
eight-slot ring per receiving leader, fires on a due frame, and leaves a
stamp that one later ping reads. Every leaf is indexed or written in that one
chain.

### 9.1 Leaves

- `last_taunt`, +0x354, `TauntRequest[8]`, indexed by the sender's `who`.
- `taunt_frame`, +0x374, `int[8]`, indexed by the sender's `who`; an absolute
  sim frame.
- `incoming_taunt`, +0x394, `int[8]`, the ring's request word.
- `incoming_taunt_who`, +0x3b4, `int[8]`, the ring's sender word (a leader
  `who`, not a play index).
- `incoming_taunt_frame`, +0x3d4, `int[8]`, the ring's due frame.
- `incoming`, +0x3f4, `int`, the ring's write cursor, 0..7.

`TauntRequest` runs 0..17; `HELP` is 16. Every leaf is zero at game start
and no rule here resets one inside a match.

### 9.2 The rule

**Writers.** Two functions, and the header's bulk writers. `Leader::receive_taunt
(006b8bf0)` writes the ring and the cursor; `Leader::process_taunt (006b8cc0)`
writes `last_taunt` and `taunt_frame`. `Leader::init (006e3930)` on its
negative-tribe branch also clears the three ring arrays and `incoming` in an
explicit loop after its memset. There is no other writer: both the absolute
net over every slot's family range and the register-relative net over
displacements 0x354..0x3f7 were re-run, and every other hit belongs to
another record. In particular `Leader::end_process (006b8690)` and
`Leaders::end_process_all (006ed070)` store at 0x3bc off the 0x8c-stride
`GameInfo` player record, never a leader. `Leader::process (006b88b0)`,
`Leader::process_taunts (006b8ba0)` and `Leader::close (006b7f80)` are dead.

**The producer.** No computer issues a chat command; every producer is a
window of the local human. `CommandPackage::process_chat (009454f0)` handles
a `Chat` command from player `play` (sender `S = player[play].who`, the byte
at +0x77 of the 0x8c record). If `S`'s `net_player` is reported blocked by
crossplay, nothing is delivered. Otherwise, for each valid slot `k` in 0..7,
and only when the command's `taunt >= 0`, it calls `receive_taunt` on `k`
when the delivery gate passes (`sem` is the game semaphore word):

- **Broadcast** (`bits == -1`): (`leaders[S].chat_status[k] == 0` or
  `sem & 0x40`) and (`leaders[k].chat_status[S] != 2` or `sem & 0x40`).
- **Masked**: (`bits` has bit `k`, or `sem & 0x40`). Then, outside
  multiplayer (`sem & 4` clear), a line starting with the cheat string goes
  to the console and nothing is delivered. Else (`leaders[k].chat_status[S]
  != 2` or `sem & 0x40`). The masked arm has no sender-side test.

The taunt word comes from `Taunts::check_for_order (009809d0)`: −1 when no
taunt has the id or its order string is empty, 0 when the string matches no
request name, else 1..16 (two names map to `HELP`). The cheat path pushes 0.
Both 0 and −1 deliver nothing to the ring.

**`receive_taunt(R, req, play)`.** Exemptions, all must pass, else nothing:

1. `req > 0`, signed (0 and negatives refused; no upper bound).
2. `S` active (`leaders[S].flags & 2`).
3. `R` not human (flag bit 4 clear) and active (bit 2).
4. `is_ally(R, S)` (`LeaderData::is_ally (006edb50)`: `S == R`, or
   `diplos == 2` both ways).
5. `R != S`.

Then, at slot `i = incoming`:

    incoming_taunt[i]       = req
    incoming_taunt_who[i]   = S
    incoming_taunt_frame[i] = frame        if req == 16
                              frame + 60   otherwise
    incoming                = (incoming + 1) & 7

The cursor step is C's signed `% 8`, which is `& 7` for a cursor that only
ever holds 0..7. It advances on every accepted request, `HELP` included.
There is no occupancy test: a ninth request inside one delay overwrites an
unfired slot. `frame` is the frame on which the command is applied, which is
before that frame's leader pass.

**The drain.** In `Leaders::process_all (006ed2a0)`, at the tail of each
slot's step with `flags & 2`, and only when `frame != 0`: for `k` in 0..7 in
slot order, if `incoming_taunt_frame[k] == frame`, call
`process_taunt(incoming_taunt[k], incoming_taunt_who[k])`. Nothing is
cleared. Consequences: a `HELP` fires on its receiving frame; a `HELP`
received on frame 0 never fires; the zeroed ring never fires; several slots
due on one frame fire in slot order, which after a wrap is not arrival order.

**`process_taunt(R, req, S)`.** If `leaders[R.who]` is active and not human,
`S != R`, and `leaders[R].diplos[S] == 2` and `leaders[S].diplos[R] == 2`
(re-tested now, not at receipt):

    last_taunt[S]  = req
    taunt_frame[S] = frame

The store precedes the request's range test, so any accepted `req` is
stored; 1..16 then act, above 16 does nothing. The actions themselves
(goods offers for 1..5, the production-AI weights for 7..15, nothing for
`HELP`) write other families' leaves.

**The reader that changes a decision.** `Leader::action_ping (006d18a0)`,
when a human `P` pings for itself: for each other of the eight slots `k`
that is valid, not human and `is_ally(P, k)` (a failure continues the loop),
if `leaders[k].last_taunt[P] == 16` and `frame <= leaders[k].taunt_frame[P]
+ 225`, it sends `Armies::find_useful_army` for `k` to the spot with
`send_here(MOVE_TO)` and sets that army's +0x28 to 2,700. Nothing else reads
`last_taunt` or `taunt_frame`; `incoming` is read only by `receive_taunt`;
the ring words only by the drain. `LeaderData::walk_data (006d6750)` reads
all six whole for the checksum.

### 9.3 In the crate

Nothing of the chain exists yet.

- `Command::Chat` is decoded with `taunt` and `taunt_num`
  (`crates/rondata/src/commands.rs:921`) and dropped at
  `crates/rondata/src/input.rs:1611` inside `Stream::one` (:1503). The port
  replaces that skip with a chat arm that applies the delivery gate and calls
  a new `receive_taunt` on the sim.
- The ring and stamps belong on the leaders record
  (`crates/sim/src/leaders.rs`).
- The drain belongs in `Sim::tick`'s per-player loop
  (`crates/sim/src/core/frame.rs:94`, the `Leaders::process_all` analogue),
  calling a new `process_taunt`.
- `Command::Ping` is not mapped, so there is no `action_ping`;
  `Armies::find_useful_army` exists (`crates/sim/src/ai/army.rs:2207`) with
  no caller.
- Checksum: the six leaves are listed at
  `crates/sim/src/checksum/leaders.rs:87`–92, and `leader_long` (:949)
  answers each through its catch-all `_ => None` (:1305). Each is a
  `(unheld)` entry of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs`) with count 2 (two walked
  leaders on run 1562's frame 91; read from the code, not from the
  verdicts). Six `leader_long` arms holding the leaves move those six
  entries, off the list where run 1562's values are the zero a game with no
  taunt line predicts.

### 9.4 Reached by a plain game

Only a game in which a human has a mutually allied computer (team play) and
sends a chat line whose taunt carries an order string reaches any writer. A
one-versus-one against the computer, or any game where no human sends such a
line, leaves all six leaves at zero for its whole length. No trigger, Conquer
the World or multiplayer path is needed; run1077 and run1086 are such
games.

### 9.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/taunts-*.md`); no diff backs it yet.
Counts per claim row: `last_taunt` 11/0/0, `taunt_frame` 7/0/0,
`incoming_taunt` 12/1/0, `incoming_taunt_who` 8/0/1, `incoming_taunt_frame`
11/0/0, `incoming` 8/0/0, the shared pipeline 8/2/2 (confirmed / corrected /
open).

Corrections that changed a rule:

- The delivery gate (9.2, the producer). The masked arm has no sender-side
  `chat_status` test; a masked cheat line outside multiplayer delivers
  nothing; a crossplay-blocked sender delivers nothing.
- `check_for_order` returns 0, not −1, for an order string that names no
  request. This changes no leaf.

Confidence is **high** on every store, gate, constant, cursor step and the
drain: each was read in the listing by both readers and the adjudicator.
It is **medium** on the delivery gate's `sem & 0x40` and crossplay arms,
which rest on the decompile alone; neither bit's meaning is established.

Witnesses, dump readings in `docs/RUNS.md` and **not pinned diffs**:
run1077's `HELP`, after which leader 1 holds `last_taunt[0]` 16 and
`taunt_frame[0]` 5411 on its 5412 dump. That is the zero delay, and it shows
that commands are applied before the drain on the same frame. run1086's food
and timber requests read `last_taunt[0]` 1 with `taunt_frame[0]` 661, and 2
with 681. Receipt one block before 661 matches the 60-frame delay.

### 9.6 Not established

- **The sender's range.** `S` is a byte; the ring and `last_taunt` have
  eight entries and `leaders.list` has ten. Every capture's live `who` is
  0..7, and nothing read bounds the byte. This does not matter to any plain
  game. A bound check at the producer, or a capture with a ninth or tenth
  seat sending a taunt, would settle it.
- **`Setup::build_leader` mid-match.** Seven of `Leader::init`'s ten callers
  pass a tribe word; any of them re-zeroes the family only if that word is
  negative. All are believed to run at setup, when the family is already
  zero. Reading `Setup::build_leader`'s callers would settle whether it can
  run inside a match.
- **`LeaderData::walk_data` in load mode** was not re-read; its range
  [+0x8, +0x692a) is confirmed only through `docs/CHECKSUM.md` §8.
- **The meaning of `sem & 0x40` and of crossplay's block**: a reading of
  their writers, or a capture with chat between allied seats under each.
- **The two witnesses become assertions**: a `rondata::diff` check on
  run1077 and run1086 that the leaves hold 16/5411 and 1/661, 2/681 on the
  cited frames, once the chain is ported.

## 10. The end of the game

Defeat, victory and the wonder clock. `Leader::defeat (006ecb00)` and
`Leader::victory (006ec9b0)` write four leaves; the victory pass writes the
wonder pair.

### 10.1 Leaves

- `defeated_by`, +0x10, `int`: the captor's slot, −1 when none.
- `wonderwin_stamp`, +0x448, `int`: frames; the start frame while running,
  minus the banked time while stopped.
- `wonderwin_timer`, +0x44c, `int`: a 0/1 running flag.
- `victory_type`, +0x7d8, `int`.
- `defeat_type`, +0x7dc, `int`.
- `defeat_stamp`, +0x6924, `int`: a frame.

`defeated_by` is −1 and the rest 0 from `Leader::init (006e3930)`; under
`tribe < 0` the wonder pair is zero by the memset alone (`init` returns
before its explicit stores; same value). Nothing else resets any of them.

### 10.2 The rule

**`defeat(who, type, by, instant)`**, in this order:

1. **Captor stamp, at entry** (`Leader::defeat_by (006d1c80)`, its only
   caller is `defeat`). If `defeated_by[who] >= 0` (signed), skip this
   step. Otherwise `defeat_stamp[who] = frame`; then, if `by >= 0`,
   `defeated_by[who] = by` and, unless `elimination == 3`, the captor arm
   (`capture_city`) runs. A type ≠ 5 blow-up arm precedes the stamp and
   does not touch it.
2. **Guards.** Return if `flags & 0x40` (already defeated) or
   `semaphore[1] & 2` (balance mode).
3. `flags2 & 0x200` ← `instant`; `flags = (flags & ~2) | 0x40` (bit 0
   stays); `Armies::leader_defeated`; `flags2 |= 2`; `musical_chairs =
   frame`; `defeat_type[who] = type`.

So `defeated_by` and `defeat_type` are write-once; `defeat_stamp` is
re-stamped by every later call while `defeated_by` stays −1, even on a
defeated leader, which can also still receive `defeated_by` and run the
captor arm.

**Callers:**

| caller | type | by |
|---|---|---|
| `lost_a_city`, by `elimination` (`game+0x37`): 2 → 2 when `param_1 != 0`, unless tribe bonus `0x17` and the loser still holds its own capital; 3 → 3 when `param_2 != 0`; any other mode (0, 1, …) → 0 when `flags2 & 1` clear and cities (+0x3f8) 0 | 2 / 3 / 0 | the captor (may be −1) |
| `lost_a_city` GG arm: `param_1 != 0`, captor `>= 0`, `elimination == 1`, no wonder `0x213`, and some slot with `flags & 3 == 3` that is the captor or that the captor counts as ally holds preq `0x2b9` | 1, `instant` 1 | the captor |
| `Leader::process_elimination` (elimination 1; its by-self arm is dead) | 1 | `LeaderData::find_capital (006eb930)`'s holder; **self** when no live capital-flagged city is held by anyone (razed) |
| `Leader::gain_tech (006dcb60)`, when the gained tech is the `0xae4` bonus's preq: every slot `j ≠` gainer with `LeaderData::has_capital (006d54c0)` 0 whose holder is the gainer or the gainer's ally. `has_capital` is 1 unless `elimination == 1`; a foreign holder needs `j`'s `flags & 3 == 3` | 1, `instant` 1 | the gainer |
| `leave_game` with the capital timer (+0x418) running: under elimination 1 the `find_capital` holder (may be self), else self | 1 | holder / self |
| `leave_game` otherwise | 6 resign, 7 drop | −1 |
| `Unit::close (0060ee50)`: the **last** unit's close (count decremented first), owner `< 8`, units (+0x93c) 0, cities 0, `semaphore[1] & 8` and `leader_flags2 & 1` clear; no defeated test | 0 | −1 |
| the victory cascade | 5 | −1 |
| musical chairs (`GameDaemon::process_victory (00730ef0)`) | 4 | −1 |
| armageddon (`Game::defeat_all`) | 8 | −1 |
| the scenario | 9 | −1 |

No code writes `defeat_type` 10; it reaches the field only through
`LeaderData::walk_data (006d6750)`. In multiplayer `leave_game` defeats no
one when another seated human shares the leaver's `who`.

**`victory(who, type, instant)`.** Return if `flags & 0x60` (won or
defeated). Set `0x20`, `flags2 & 0x100` ← `instant`, `victory_type[who] =
type`. Then, for every other slot with `flags & 3 == 3`: set `0x80`; if
that slot counts `who` as ally (`is_ally(that, who)`), `victory(type,
instant)` for it, else `defeat(5, −1, 0)`. A prior winner not allied to a
new winner can be defeated, since `defeat` tests only `0x40`. Pair
`victory_type` with the won flag: 0 is also conquest.

Type by caller: 0 conquest (`Game::check_victory (005926b0)`,
`Leader::set_diplo (006ec6a0)`, `TurnControl::toggle_pause (00957ab0)`),
1 wonder, 2 territory, 3 `gain_tech` under VICTORY 9, 4–6 the other arms
of `process_victory`.

**`check_victory`'s gate.** Return if `semaphore[0] & 0x40`. Count seated
humans and live humans (`flags & 0x43 == 3`). If a human lives, or there
are none, or balance mode: return unless every live leader is mutually
allied (`diplos == 2` both ways) with the first live one. Then latch
`semaphore[0] |= 0x40` and compute

    A = armageddon_per_team·num_sides + armageddon_per_nation·num_nations + armageddon

(×3 at starting resources 7, ×2 at 5, 6, 11, 12, floor 100 at 8). Only if
`game.armageddon < A`: walk slots with `flags & 3 == 3`, setting `0x80`;
the first undefeated is the candidate; if any later live, not-won slot is
not mutually allied with it, set `semaphore[2] |= 0x40` and declare no
winner; else `victory(candidate, 0, 0)`. All humans dead and the computers
unallied ends the game with no `victory_type`.

**The wonder arm**, in `process_victory`, every frame under `VICTORY` ∈
{0, 1, 6}, with `W = Game::wonder_winning (005948a0)` read once. For slots
0..7 with `flags & 3 == 3`, in order:

- **Off side** (`W < 0`, or `i ≠ W`, `who_i ≠ W` and not mutually allied
  with `W`): if the flag is set, clear it and `stamp −= frame`.
- **On side.** `gg` = some slot of all ten with `flags & 1` (bit 0 only,
  so a defeated teammate counts) that is `i`, has `who == i`, or is
  mutually allied with `i`, holds preq `0x2b9`.
  - `VICTORY != 0`, or (not multiplayer and `num_nations <= 1`), or `gg`:
    if no earlier on-side slot was handled this frame, `victory(i, 1, gg)`.
  - Else flag clear: set it, `stamp += frame`, event 9.
  - Else, if no earlier on-side slot was handled: `remaining =
    wonder_timer() − (frame − stamp)`; `victory(i, 1, 0)` iff
    `remaining == 0` **exactly** (the territory arm wins on `<= 0`; do not
    copy it).
  - Mark handled. The walk never stops early.

`Game::wonder_timer (005944f0)` = 0 if `constants+0xd08` = `T` (`WONDER_TIMER`)
`< 1`, else `max(1, (M/2 + world[0]·T) / M)`, signed, `M/2` the signed
halve, `M` the current `map_sizes` row's +0x144. At 0 the clock never wins.

**The pay-back** (`Leaders::process_all (006ed2a0)`, inside its `flags & 2`
slot gate, after `process_elimination` and the +0x414 and +0x440
pay-backs): if `TIMER_REFRESH_RATIO` (`constants+0xd00`) ≠ 0, `frame %
ratio == 0`, `wonderwin_stamp < 0` and the flag is 0: `stamp += 1`.
`Leader::process (006b88b0)`'s copy is dead.

**Readers that change a decision.** Only `wonderwin_timer`:
`Army::find_target` (×10 and `about_to_win`), `Leader::diplomacy`,
`Game::is_victory_timer (00594950)`, `Wall::process (00640450)` (a rival
not allied to the owner with the flag adds one). The rest feed UI, stats
and the save.

### 10.3 In the crate

- `Sim::defeat(who, by)` (`crates/sim/src/cities/city.rs:623`) carries no
  type or instant, and returns at once on `leaders.defeated[who]` (:624).
  The port puts step 1 ahead of that return; `defeat_type` goes after it.
- `Sim::resign` (`crates/sim/src/leaders/resign.rs:43`) is type 6. Under
  a running `lost_capital_timer` the original passes 1 with `by` = holder
  or self: a plain-game divergence. It kills only planes, and the crate
  has no `Unit::close` last-unit `defeat(0, −1, 0)`, so a resigned
  leader's re-stamp is missing.
- No `Sim::victory`. The SEAM at `crates/sim/src/territory/victory.rs:21`
  says the game never ends by territory; five captures contradict it.
  `victory(2, ·)` from `process_victory_territory` (:31) is the first port.
- The wonder arm is a new `process_victory_wonder` in `core/frame.rs`
  before `process_victory_territory` (`crates/sim/src/core/frame.rs:222`),
  using `Sim::wonder_winning` (`crates/sim/src/ai/ai_build.rs:1439`); the
  pair sits beside `popwin_*` in `crates/sim/src/ai.rs`, with
  `WONDER_TIMER`.
- **Trap:** `refresh_victory_timers` (`territory/victory.rs:88`) is called
  for every player (`core/frame.rs:94`, :194) with no alive gate, so a
  defeated leader's banked `popwin_stamp` is paid back where the original
  skips it. Add the gate for both pairs.
- **Trap:** the `w44c` stub returns 0 (`crates/sim/src/ai/army/target_gate.rs:428`);
  30 captures hold a live 1. `Wall::process`'s SEAM is
  `crates/sim/src/cities/site_recruit.rs:157`.
- `crates/rondata/src/diff/third_pair.rs:432` calls run600's 17380 a quit;
  the dump shows `defeat_type` 5, the cascade.
- Checksum: the six leaves are listed in `crates/sim/src/checksum/leaders.rs`
  (:39, :113, :114, :144, :145, :310) and `leader_long` (:949) answers them
  through its catch-all (:1305); `popwin_*` (:1060) is the model. Each is
  an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1753`; read from the code).

### 10.4 Reached by a plain game

The captures' lobby is elimination 1 and VICTORY 0 (run600's dump line
4727; the crate's default, `crates/sim/src/ai.rs:1132`). So a skirmish
against the computer reaches: `lost_a_city` type 0 and its GG arm,
`process_elimination`, the `gain_tech` GG arm, `Unit::close`, resign (6)
and its capital-timer arm (1), the victory cascade (5), victories 0, 1
and 2, the wonder clock's start, and `check_victory`. Behind lobby
options: `lost_a_city` types 2 and 3 (elimination 2, 3), armageddon (8),
VICTORY 9's type 3. Behind the scenario: type 9. Multiplayer: drop (7)
and the shared-`who` arm. Type 10: a load only.

### 10.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/end-of-game-*.md`); no diff backs it
yet. Counts (confirmed / corrected / open): `defeated_by` 13/3/1,
`defeat_stamp` 10/0/1, `victory_type` 10/1/1, `defeat_type` 9/2/1,
`wonderwin_timer` 14/2/1, `wonderwin_stamp` 8/0/1. Every store, guard and
push was read in the listing; confidence is high throughout.

Corrections that changed a rule: `process_elimination`'s `by` falls back
to self; the `gain_tech` arm exists only under elimination 1, with the
gainer's view of alliance; elimination 1 is the plain game, so the type-1
arms are plain-game; `check_victory` can end with no winner; `defeat_type`
10 has no writer; `gg` tests bit 0, counting defeated teammates.

Witnesses from an archive sweep of 3,805 dumps, **not pinned diffs**:
run600's closing frame 17380, slot 1 `victory_type` 2 and slot 0
`defeat_stamp` 17380, `defeat_type` 5, `defeated_by` −1 (the cascade);
the same shape in run972b, run981b, run1256, run1316. run1200: slot 0
`defeated_by` 1, `defeat_stamp` 5639, `defeat_type` 0. run24: slot 1
`defeated_by` 0, 3363 frames after its capital fell (the shape of
`process_elimination`). `defeat_type` 6 in the start captures, 9 in
run1113/1114. `wonderwin_timer` 1 in 30 captures (run652 … run1423c).

### 10.6 Not established

- **The stop, the banked stamp, the pay-back** (and the crate's missing
  alive gate): no capture holds a negative `wonderwin_stamp`.
- **A wonder win.** No `victory_type` 1 in the archive; nor `M`'s row
  (`0xe7fca8`) and `world[0]` (`*0xc06188`) in `wonder_timer`. A reading
  of both, or a capture that runs the clock out.
- **A re-stamp.** No file shows two `defeat_stamp` values.
- **Resign under a capital timer.** No `defeat_type` 1 in the archive; it
  would show `defeated_by` = the quitter's own slot.
- **The territory expiry's frame** for run600 (from `popwin_stamp` 14293)
  is the territory family's arithmetic; `Game::popwin_timer` for its map
  is unread.
- **`lost_a_city`'s `param_1`, `param_2` and captor** are derived by its
  callers (`Build::close`), owned by §11.
- **The pins:** run600's closing block and run1200's loss, as
  `rondata::diff` assertions once the leaves are held.

## 11. The lost capital

### Leaves

- `lost_capital_stamp`, +0x414, int, frames.
- `lost_capital_timer`, +0x418, int, a 0/1 flag (only 0 and 1 are stored).
- `lost_capital_modifier`, +0x41c, int, 8.8 (`0x100` = ×1).

The three move together. A **loss** sets the timer, writes the modifier and
turns the stamp into a start time. A **stop** clears the timer and turns the
stamp back into a debt. A per-frame **refresh** pays the debt down while no
clock runs. The **countdown** defeats a loser whose clock runs out.

### The rule

**Loss: `Leader::lost_capital (006dc870)`**, called with `O` the loser and
`A` the other leader. It does anything only when `timer == 0`. If the timer
is already 1, the call stores nothing, so a second loss while the clock runs
is a no-op for all three leaves. Live callers:

- **E1**, `Cities::capture_city`, call site 735092, with `A` the captor.
  The captured city carries flag `0x10`, `has_wonder(O, 0x213)` (the
  Forbidden City) is 0 or 2, and `find_capital(O, skipping this city)`
  finds nothing or a holder other than `O`. Together these mean `O` holds no
  live Forbidden City and no other live `0x10` city of its own.
- **E2**, `Build::close (00628980)`, call site 62912e, with `A` the holder.
  The closing building is the Forbidden City (`0x213`) and is **active**,
  meaning Build byte `+8` bit `0x4`, the test `WallData::is_active
  (00472350)` makes. The `Game` semaphore byte 1 bit `0x8` is clear,
  `elimination != 3`, and `find_capital(O, -1, -1)` returns a holder other
  than `O`. Under tribe bonus `0x17`, a second `find_capital(O, skipping
  (idx, holder))` runs, and if its hit's flag word is negative, that hit's
  holder becomes `A`. `lost_a_city(O, 1, 1, A)` follows. When the closing
  Forbidden City is not active, the arm calls
  `UnbuiltWonders::remove_unbuilt_wonder` instead.

Within the loss, in order:

1. `timer = 1`.
2. The modifier. `To = get_team_terr(O)` and `Ta = get_team_terr(A)`, where
   `get_team_terr` sums `territory` (+0x9d8) over a team. Then:
   ```
   w = num_nations == 0 ? 0 : land_size / num_nations    // World+0x78, Game+0x6a0
   m = (To + w == 0 || Ta + w == 0) ? 256
                                    : clamp((To + w)·256 / (Ta + w), 128, 512)
   if O.city_num != 0 && A.city_num != 0 && A.city_num < O.city_num:
       m = min(m·(O.city_num + 2) / (A.city_num + 2), 768)
   ```
   The city rescale follows either branch. Its result is at least `m`, so
   only the 768 ceiling binds. Both `city_num`s **count the city changing
   hands**: `Build::activate` has already added it to `A`, and `City::close
   (00737550)`, which takes it from `O`, runs after both capital calls. In
   terms of the cities each side held outside that one, the ratio is
   `(O_left + 3)/(A_before + 3)`. `To` and `Ta` are the sums from the last
   `GameDaemon::check_borders (00732060)` pass, the only in-play writer of
   +0x9d8, so they come from before the capture's border change. All
   divisions are signed and truncating.
3. `stamp = frame + (stamp·256) / m`, C truncating division, with
   `stamp ≤ 0` on entry. A first loss starts from 0 and stores `frame`
   exactly. A loss after a recapture folds the remaining debt in, rescaled
   by the new `m`.

The messages after the stores are gated on `elimination == 1`. The stores
themselves are not gated.

**Stop: two writers.** Each does anything only when `timer == 1`. Each sets
`timer = 0` and `stamp = (stamp − frame)·m / 256`, truncating toward zero:
the listing adds `0xff` to a negative product before the shift. The product
is `≤ 0`, so a floor would differ by one frame.

- **E3**, `Leader::recapture_capital (006d1ba0)`, on `A`, the taker. Its
  sole caller is `capture_city`, call site 7350e2, when the taker's new city
  has `0x10` and `find_capital(A, skipping (new city, A))` finds nothing or
  a holder other than `A`.
- **E4**, `Build::activate (00623e20)`, Forbidden City arm (625f64-625fc8),
  on the owner. It fires when the wonder type is in `0x20e..0x21e` (or
  passes the virtual test), `init_wonder` is `≥ 0` as a signed short, the
  building `is(0x213)`, and the owner's timer is set. It is **not** gated by
  `captured`, so a captured Forbidden City stops the taker's running clock.

**Refresh, inlined in `Leaders::process_all (006ed2a0)`** (store 6ed394).
`Leader::process (006b88b0)` is its dead twin. Every frame, for slots
**0..7** with `leader_flags & 2`, the refresh runs after that slot's
`process_elimination`:

```
if ratio != 0 && frame % ratio == 0 && stamp < 0 && timer == 0: stamp += 1
```

Here `ratio` is `TIMER_REFRESH_RATIO`, 5 shipped (`rules.xml` line 727,
`core/tuning.rs:1923`). The debt is paid at **one frame per five**.

**Countdown: `Leader::process_elimination (006b8a20)`.** It runs under the
outer `game` (0xc061ec) gate when `elimination == 1` and `timer != 0`. If
`limit − (frame − stamp) ≤ 0` (signed), the leader is `defeat(1, by)`. Here
`by` is the holder `find_capital(-1, -1)` returns when `gamec` (0xc061e8)
elimination is 1, and the leader itself otherwise. `limit` comes from
`Game::retake_capital (00594530)`:

```
base  = RETAKE_CAPITAL < 1 ? 0 : max(1, (xs·RETAKE_CAPITAL + S/2) / S)
limit = base·m / 256                         // truncating toward zero, who ≥ 0
```

`xs` is `World+0x0`, the map width. `S` is `+0x144` of the record at
`*(0xe7fca8)`, the Standard map size, 70 (`docs/MAPGEN.md` line 4273).
`RETAKE_CAPITAL` is 3600 (`rules.xml` line 728). `S/2` truncates toward
zero. With `Game` semaphore byte 2 bit `0x2` set, two further arms apply:
`has_wonder(who, FC) == 2` doubles the limit, and a slot-0/1 peace arm
halves it. Both are Conquer the World-shaped (`== 2` is the CtW bit alone).
A larger `m` gives the loser more time.

**Resets.** `Leader::init (006e3930)` stores 0 to the stamp and the timer at
top level on every call (6e3f89, 6e3f8f), not to the modifier. The bulk
writers (header) zero all three. `Leader::defeat` and `defeat_by` clear
nothing. **The modifier goes stale after a stop and must be kept, not
zeroed.** Every rule reader of it sits behind `timer != 0`, but the checksum
and a whole-record dump see the stale value.

**Readers that change a decision.** The countdown (timer, stamp,
modifier). The refresh (stamp, timer). The stop (modifier). `Player::leave_game
(006ee010)` (timer): on a resign or drop, a leader with `leader_flags & 3 ==
3` and the timer set is `defeat(1, holder)` under `elimination == 1`, and
`defeat(1, self)` otherwise. It chooses who is credited with the defeat and
writes none of the leaves. Its callers are `Player::resign (006edcb0)`
(from `Player::quit (006edc00)`, `CommandManager::issue_resign
(00941170)`, `CommandPackage::process_resign (009438c0)`) and `Player::drop (006ede80)`.
`IFaceMainBase::draw_timers (0080b060)` and `LeaderData::log_data (006e5110)` read the
leaves for display only.

### Where this corrects docs/CITIES.md §8.5

§8.5 says the debt "decays one frame per frame while the capital is held".
It decays **one frame per `TIMER_REFRESH_RATIO` frames, five shipped**, and
only while the timer is 0. Three further corrections to the same paragraph:

- Both `× m >> 8` steps, in the stop and in `retake_capital`, **truncate
  toward zero**. They are not arithmetic shifts.
- "How the base countdown is first seeded is not established" is
  **settled**. `Leader::init` zeroes the stamp, so a first loss stores
  `frame`. The countdown is `retake_capital` above, with `S` the Standard
  map size. This also settles `docs/CITIES.md`'s open item 6: the stamp's
  seeding, and the map-size field `retake_capital` divides by.
- The loss runs "once" only in effect. Its guard is the timer, so a loss
  after a recapture runs again.

### In the crate

- **Stamp and timer.** There is no field for either. `Tally::lost_capital_frame`
  (`cities/city.rs:122`), an `Option`, is set and cleared at
  `city.rs:3213-3224` and **never read**: it is a partial copy of the stamp
  plus timer. E1 and E3 replace those stores in `Sim::capture_city`
  (`city.rs:3169`). The new fields sit beside `popwin_stamp` (`ai.rs:705`).
- **E2** goes in `Sim::close_building` (`city.rs:2782`). **E4** goes in
  `Sim::activate` (`city.rs:2420`), beside `kremlin_activate`
  (`cities/nations.rs:467`, called at `city.rs:2475`).
- **Refresh** becomes a second clause of `refresh_victory_timers`
  (`territory/victory.rs:88`, called at `core/frame.rs:194`), which today
  pays only `popwin_stamp`. **Countdown**: a port of `process_elimination`
  goes immediately before `frame.rs:194`, with `retake_capital` beside it.
  There is no `retake_capital` and no scale-by-axis helper today (only
  `mapgen/map_lakes.rs:314 scale_by_area`). The inputs exist: `team_terr`
  (`ai/ai_census.rs:155`) and `land_size` (`victory.rs:110`).
- The AI's `capital_lost` (`ai/ai_build.rs:296`) recomputes from the cities.
  The verdicts treat this only as a pointer.
- **Checksum.** The rows are `checksum/leaders.rs:100-102`. Today
  `LEADERS_HELD_OUT` lists `lost_capital_modifier`, `_stamp` and `_timer`,
  each `(unheld)` with count 2 (`rondata/src/diff/checksum_lines.rs`
  1809-1811). Holding all three removes those three entries.

### Reached by a plain game

Yes, on any capital capture. E1 and E3 are skirmish events, and their stores
are ungated by `elimination`. E4 is reached by building a Forbidden City or
by capturing one. E2 needs an active Forbidden City to close while another
leader holds the owner's former capital. The refresh runs every frame. The
countdown needs `elimination == 1`. `leave_game` is reached by a resign
command, in single player too. Two paths are probably not plain-game:
`retake_capital`'s semaphore-gated arms (CtW-shaped), and E2's semaphore
byte 1 bit `0x8` (meaning unread).

### How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/lost-capital-*.md`); no diff backs it yet.

The verdicts: stamp 24 confirmed, 1 corrected, 1 open; timer 14 confirmed,
0 corrected, 0 open; modifier 14 confirmed, 0 corrected, 3 open. Every
store was read in the listing, and both readers agree on all arithmetic.
Confidence is high for the formulas. It is medium-high for E1's
`city_num` timing (taken from the decompile's order; `Build::swap_team` was
not read for a `city_num` store) and medium for the territory timing.

Corrections that changed a rule:

- E2's gate is `is_active` (bit `0x4`), not bit `0x1`.
- The §8.5 corrections above.
- Two inputs the readings left open are settled: both `city_num`s include
  the captured city, and `To`/`Ta` come from the last `check_borders`.
- `leave_game` is reached by a resign command, not only by a departure.

Witness, not a pinned diff: `crates/rondata/src/diff/korean_ancient_pair.rs:326`
records `lost_capital_stamp 5639` on the human's defeat on frame 5639,
consistent with a first loss storing `frame`.

### Not established

- **Whether E2 can run inside a capture.** A captured Forbidden City is
  `swap_team`ed and `activate`d, not closed (`docs/CITIES.md` §7.3). Read
  `Build::swap_team`'s body for a `close` call.
- **The `Game` semaphore bits**: byte 1 bit `0x8` (E2's gate) and byte 2
  bit `0x2` (`retake_capital`'s arms). Read their writers.
- **`num_nations`' writer** (`Game+0x6a0`): whether it counts gaia or the
  dead. Read its writers.
- **`get_team_terr`'s team-style-7 arm and its ally rule** (not this
  family's leaf). Read `LeaderData::get_team_terr (006d62e0)` whole.

## 12. `pop_issues` and `misery`

### 12.1 Leaves

- `pop_issues`, +0x7e8, `int`. A plain count with no scale: how many times
  this frame a unit's training met the population cap.
- `misery`, +0x7ec, `int`. A constant 0.

### 12.2 The rule: `pop_issues`

**Reset.** `Leaders::process_all (006ed2a0)` stores 0 at the head of each
leader's step (the inlined head of the dead `Leader::process (006b88b0)`),
for slots with `leader_flags & 2` only. That step runs before strategy,
armies and objects, so the count each frame is that frame's alone. Also 0
from `Leader::init (006e3930)` and the bulk writers of the header; the save
loader restores the saved value.

**Step, building** (`Build::do_queue (0061e410)`, called once per building
per frame, and from the debug-only `Console::hurry_action (007c3810)`):

1. Early returns, before any slot is looked at:
   - the building's queue is empty (`queued`, BuildData +0x82, is 0);
   - it is a library (`LIBRARY` 0x1b3) that is not the leader's first
     library (`Leader::get_first_library (006db6c0)`), and the entry is not
     `DISBAND` (0x29a);
   - it is a missile silo (`MISSILESILO` 0x208) holding a missile
     (`inside_down`, ObjectData +0x28, `>= 0`), the entry's tech bit is set
     and the entry is not `DISBAND`.
2. The visited slot's type `t` counts when it is not a build type
   (`0x19e ≤ t ≤ 0x21e`), not `DISBAND`, `is_unit_type` (vtable +0xc), and
   its tech bit is set in `LeaderData +0x6c18`. Then, if
   `check_population (006e1330)` is true, which is the strict test

       pop_cap < control_cost(t) + control

   the count takes +1. The test runs before the slot's progress is advanced,
   whatever the progress.
3. Progress still advances on that visit at `accel_train`, clamped to
   `train_time`. The full clock is then refused by `Build::finished`, which
   runs the same population test first and returns 0.

**At most +1 per building per frame.** Only the first library visits more
than one slot (the slot+1 recursion is gated on `LIBRARY`); its extra slots
carry techs, which fail `is_unit_type`. Any other building visits a second
slot only from slot 0, after `Build::finished` refused the head:

- `finished` returned 0 (the population test, or anything else returning
  0): the fallback is `get_next_non_unit (00630b10)`, which returns only a
  non-unit or unresearched slot, so it cannot count;
- `finished` returned negative (the caravan or aircraft limit, tested after
  the population test), the building is an airbase (`AIRBASE` 0x1bf, for
  `get_next_helicopter`) or not (`get_next_non_caravan`), and
  `queued >= 2`: that fallback slot can count.

So a counted head never leads to a second count. The order buildings are
visited in does not change the frame total, only the partial value seen by
the in-frame reader below.

**Step, carrier** (`Unit::process (00610bc0)`): a carrier with
`num_queued` (short +0xa0) ≠ 0, owner `leader_flags & 2`, and
`t = current_upgrade(FIGHTERBOMBER 0x134)`. Each frame
`queue_time += accel_train` (times `ai_speed` when that exceeds 1); if
`queue_time >= train_time(t)` and `pop_cap < control_cost(t) + control`
(strict: equal launches), the count takes +1 and `queue_time -= accel`. The
clock sits at its last value, at or past `train_time`, so the next frame's
add crosses again: **+1 every frame for as long as the cap holds**, per
capped carrier.

**Readers.**

- `Army::find_target (006f69b0)` multiplies a candidate's score ×4 when the
  army's own leader's `pop_issues` ≠ 0, the candidate is another leader,
  and either stance word between them is 0. Since the reset precedes
  `Armies::process_all`, every bit-2 leader reads 0 there. The ×4 can fire
  only from `find_target` runs inside `Objects::process_all`:
  `Armies::emergency` from `Object::do_damage (0064a480)`, and through it
  `Army::process` and `send_navy` (which `find_target` itself calls). There
  the count is partial and depends on object order.
- `Leaders::end_process_all (006ed070)`: UI only.
- The dump: `GameLog::end_frame` runs after `end_process_all`, so it prints
  the whole frame's count.

The capped visit's other effects (flag `0x800`, `pop_cap_frame`, feedback,
`S_POP_LIMIT`, the `(o + frame) % 220` bubble) are console only.

### 12.3 The rule: `misery`

Every store writes 0, and nothing that decides anything reads it.

- **Writers**, all `= 0`: `Leader::calc_pop_cap (006dc490)`'s first store,
  before every branch (17 call sites); `Leader::gain_tech (006dcb60)`'s
  epoch arm (`is_tech_type`, `0x220 ≤ t < 0x275`, and `is_epoch_type`,
  `0x227 ≤ t < 0x243`; not gated on a first gain, which is moot); `Leader::init
  (006e3930)`; `Leaders::clear (006ed5d0)`; the save loader, restoring the
  saved 0. `Leader::calc_misery (006cdff0)` is `misery = 0` and has no
  caller.
- **Readers**: `LeaderData::log_data (006e5110)`, the save and the checksum.

A port holds 0 for every leader on every frame.

### 12.4 In the crate

- **No `pop_issues` storage exists.** The reset belongs beside
  `self.combat.retargets[who] = 0` in `Sim::tick`
  (`crates/sim/src/core/frame.rs:98`). Trap: that loop runs every player,
  while the original resets only leaders with `leader_flags & 2`; gate the
  new store on the crate's image of that bit.
- **Carrier arm**: `carrier_queue_tick` (`crates/sim/src/air/carrier_queue.rs:137`).
  The capped arm at `:155` already takes back the accel and returns; the +1
  goes there.
- **Building arm: missing.** The crate tests the cap only at completion
  (`economy/cost.rs:1248` `exceeds_population`, reached through
  `Handover::Population => Advanced::Blocked` at `cities/queue.rs:1575`).
  The +1 belongs on the per-visit path, `advance_slot_with`
  (`cities/queue.rs:1488`), for a researched unit entry, and on the fallback
  slot the crate already selects at `:1370`ff, **not** on
  `Advanced::Blocked`. The silo hold is already in `slot_plan`
  (`cities/queue.rs:1441`).
- **Reader: a seam.** `ai/army/target_score.rs:45` declares `f7e8`, used at
  `:144`; the live implementation returns 0 at `:453` (and `ai/army.rs:38`
  lists `pop_issues` among the fields whose multipliers are ×1). Wire it to
  the live count.
- **Mislabel**: `crates/rondata/src/diff/leader.rs:49` calls `pop_issues`
  "unread"; `find_target` reads it. `diff/leader.rs:44` calling `misery`
  unread is right.
- **Checksum**: both are in the layout (`checksum/leaders.rs:148`, `:149`)
  and both are `(unheld)` in `LEADERS_HELD_OUT` with count 2
  (`crates/rondata/src/diff/checksum_lines.rs`). Holding `misery` as 0
  removes its entry at no risk; `pop_issues` leaves the list once both
  arms are built.
- **`misery` needs no code** beyond emitting `Some(0)` and turning
  `diff/leader.rs:44` from an unmodelled key into a compared one.
  `crates/sim/src/economy.rs:305` (which cites `calc_pop_cap`'s tail) is a
  pointer, not a store site.

### 12.5 Reached by a plain game

`pop_issues`: yes. The reset every frame; the building arm whenever a
trained unit is queued at the cap; the carrier arm in a late naval game.
The ×4 reader is plain but narrow (a damage-driven emergency retarget while
the attacker's leader is capped). `hurry_action` is debug only.

`misery`: yes, trivially; its writers run, and write 0.

### 12.6 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/misery-*.md`); ~~no diff backs it
yet.~~ Since item 4162 the record diff compares `misery` on every scored
dump that prints the record, and the checksum line holds it on run 1562's
frame 91 (0 against 0); it is quiet everywhere, so only the zero is backed.
`pop_issues` is held but not compared: it parts once, on run667's closing
frame 17380, who 1, ours 2 theirs 0 (parked 4162a), and stays unmodelled.

- `pop_issues`: 25 confirmed, 4 corrected, 1 open. High confidence on the
  writers, predicates, reset and the one-per-building bound; every one is
  listing-backed. The corrections that changed a rule: the second early
  return is the occupied silo, not an already-researched item; the slot+1
  recursion is library-only, not unconditional; a building adds at most +1
  per frame, not several; `diff/leader.rs:49`'s "unread" is wrong. The
  carrier arm repeats every frame while capped (the clock is pinned), not
  only at the frame the clock first fills.
- `misery`: 13 confirmed, 1 corrected (17 call sites of `calc_pop_cap`, by
  an exhaustive `call`/`jmp` count). High confidence.

**Witnesses, not pinned diffs.** Across the 948 kept `gamelog.txt` under
`~/ron-golden`, `pop_issues` never exceeds 9 and never shows a growing
series, which fits the per-frame reset and the one-per-building bound; and
no dump prints a nonzero `misery`.

### 12.7 Not established

- **Whether a leader without `leader_flags & 2` runs `do_queue`.**
  `do_queue` tests no leader flag. Such a leader's count is never reset, so
  it would only grow; the archive's maximum of 9 argues against it. A dump
  of a defeated slot settles it.
- **The ×4's reachability is structural.** No capture shows it changing a
  target; a capture with an emergency retarget by a capped leader would.
- **`Console::hurry_action`'s call to `do_queue(0)`** rests on one reading's
  listing citation, not re-traced. It is outside the order stream.
- **No diff yet.** A `pop_issues` diff against the dump, once the building
  arm is built, turns this section into an assertion; the dump key exists
  (`diff/leader.rs:49`). A constant-0 `misery` diff is cheaper still.
- That `misery` was a live mechanic emptied before ship is a guess from its
  shape; nothing in the sim depends on it.

## 13. The slot marks and build counters

### 13.1 Leaves

- `fort_mark` +0x428, `int`: high-water mark over the leader's fort records.
- `oil_well_mark` +0x430, `int`: the same, over oil-well and platform records.
- `supply_mark` +0x434, `int`: the same, over supply-unit records.
- `wonders_built` +0x854, `int`: wonder activations since the last reset.
- `wonders_held` +0x858, `int`: the most wonder records ever live at once.
- `senates_built` +0x85c, `int`: Senate constructions.
- `forts_built` +0x8f0, `int`: fort activations.

The leaders array is `Leader[10]` (`types.txt`, `struct Leaders`, size
`0x4554c`), so every per-slot rule below runs over ten slots, not eight.
`Leaders::clear (006ed5d0)` has two callers only, `Game::close` and
`Game::run_oos_recovery`; it zeroes all seven leaves. The
`ScenarioRead::load_leaders_chunk (009acde0)` stores at +0x428/+0x430/+0x434
are off the `Game`, not a leader.

### 13.2 The rule

#### The three marks (shared rule)

Each mark indexes a per-leader registry of small records with an in-use bit.
The registry's index for a building is stored at `Build +0x78`; for a unit at
`Unit +0x84`.

- **Take:** reuse the first free record below the mark; else the allocated
  record at the mark; else append a new one. Then
  `mark = max(mark, idx + 1)`. A take raises the mark by 0 or 1, never more.
- **Give back:** clear the record's in-use bit, then walk the mark down past
  every free record at the top. A give-back can lower the mark by several.
- **Reset:** `Leader::init (006e3930)` zeroes all three after its
  `tribe < 0` early return, so only for seated slots; `Leaders::clear`.

Per mark:

- **`fort_mark`.** Take in `Forts::init_fort (0073ed70)`, called from
  `Build::activate (00623e20)` on every activation of an `is(FORTX 0x1bb)`
  type (`BuildTypeData::is_fort`, type slot `+0xfc`), `captured` not tested.
  Give back in `Forts::close_fort (0073ec90)`, from `Build::close (00628980)`
  when `is_fort` and `+0x78 ≥ 0`. Readers that change a decision:
  `find_fort`, `compute_reg_territory`, `Army::find_target`,
  `Object::do_launch`, `Leader::produce_unit`, `blocked_location`; also
  `log_data`.
- **`oil_well_mark`.** Take in `init_oil_well (0073f6f0)`, from
  `Build::activate` on every activation of `is(OILWELL 0x1a5)` or
  `is(OILPLATFORM 0x1a6)`, `captured` not tested. Give back in
  `close_oil_well (0073f670)`, from `Build::close` only when
  `build_flags & 0x40` (`is_gather_type`, type slot `+0x90`) **and** the type
  is a well or platform **and** `+0x78 ≥ 0`. The give-back writes the record:
  `+6 &= ~1`, `+7 = 0xff`, `+2 = 0xffff`, `+4 = 0`. Reader:
  `Leader::calc_gather (006ceee0)` (loop bound; the sum is order-free) and
  `log_data`.
- **`supply_mark`.** Take in `init_supply (0073ad40)`, from `Unit::init` when
  the unit `is_supply` (`unit_flags2 & 0x40`), and from `Unit::set_type` into
  a supply type when `+0x84 < 0`. Give back in `close_supply (0073acc0)`, from
  `Unit::close` (supply, `+0x84 ≥ 0`) and from `Unit::set_type` out of a
  supply type (`+0x84 ≥ 0`). Extra reset: `Setup::build_game (005ac190)`
  zeroes it for all ten slots, unconditionally, before any unit exists.
  Reader: `find_supply (0073aba0)`, through `UnitData::in_supply`,
  decision-changing; `log_data`.

#### `wonders_built`

- **Writer:** `Build::activate`, `wonders_built[owner] += 1` on **every**
  activation of a building whose type index is in `0x20e..=0x21e` (Pyramids
  to Space Program, 17 types; the vtable fast path and
  `TypeData::is_wonder_type (00470780)` name the same range). No test of
  `captured`, `announce` or `counted`, and every path through `activate`,
  the city branch included, reaches it. Order: `remove_unbuilt_wonder`, the
  increment, then `Wonders::init_wonder (0073c860)`.
- **Capture credits the captor.** `Cities::capture_city (00733380)` converts
  a member of the fallen city only when its type lacks `build_flags &
  0x2000`, and activates an active member as `activate(1, 1, 0)`. Fifteen
  wonders carry `BUILD_FLAGS` `a` alone and are city members, so a finished
  one in a captured city steps the captor by 1. The Forbidden City (`0x213`)
  carries `0x2000` but is itself a city object: `capture_city`'s head
  activates it `(1, 1, 0)`, and the captor is credited too. The Red Fort
  (`0x216`) carries `0x2000` and `e` and is not counted on capture. An
  unfinished wonder site is closed or disbanded, not activated, so it does not
  count. `Leader::defeat_by (006d1c80)` hands over military trainers only, so
  it never moves a wonder.
- **Never decremented.** Reset to 0 by `Leader::reset_score (006e37f0)`,
  called from every `Leader::init` and from `Setup::build_game`'s loop over
  seated slots, after the per-slot `Leader::init` and before
  `build_empire`; and by `Leaders::clear`.
- **Reader that changes a decision:** `BuildData::construct_time
  (0062d5c0)`, tribe bonus `0x14`'s first-wonder arm: free only when
  `get_wonders() == 0 && wonders_built == 0` and the type is not `0x21d` or
  `0x21e`. No other reader.

#### `wonders_held`

At `init_wonder`, after the new wonder's record is taken, count the leader's
in-use wonder records in `[0, wonder_mark)` (the new one included) and store
`wonders_held = max(wonders_held, count)`. Never lowered; `close_wonder` does
not touch it. Same predicate as `wonders_built`, since `init_wonder`'s one
caller follows that increment, so capture raises it too. Reset:
`reset_score`, `Leaders::clear`. Readers: `AchieveChartPane::set_chart_data
(007f2c40)` (UI) and the checksum; `log_data` does not print it.

#### `senates_built`

`Build::activate`'s non-city arm, `senates_built[owner] += 1` when the type
is in the SENATE `0x1b6` lineage, the building's city index `+0x72 ≥ 0`, and
`captured == 0`. `announce` and `counted` are ignored. Reset:
`reset_score`, `Leaders::clear`. Reader: `log_data` and the checksum.

#### `forts_built`

`Fort::init (0073eac0)`, `forts_built[who] += 1`, unconditionally. Its one
caller is `init_fort`, whose one caller is `Build::activate`, so the rule is
+1 per activation of an `is(FORTX)` type, Fort line and Red Fort (which is
also a wonder and steps `wonders_built` too). An upgrade in place does not
count. Never decremented. `Fort::init` also steps another family's
per-region short at `+0x12de + region·2` when `region < 0x40`. Reset:
`reset_score`, `Leaders::clear`. Reader: `set_chart_data` (UI), checksum.

### 13.3 In the crate

- **Marks.** `fort_mark` and `oil_well_mark` have no registry.
  `units::slots::SlotList` (`crates/sim/src/units/slots.rs:24`) is the exact
  shape: one per leader per mark, taken in `Sim::activate` beside the
  `is_fort` test (`crates/sim/src/cities/city.rs:2533`) and given back in
  `Sim::close_building`'s fort arm (`~2905`). Wells are found by kind in
  `economy/holdings.rs` today. `supply_mark` is already modelled:
  `SupplyList` (`territory/supply.rs:171`) pops free tails, so `len()`
  (`:207`) **is** the mark; opened `units/store.rs:145`, closed
  `territory/upkeep.rs:62`; the `set_type` pair is stubbed
  (`units/set_type_adapter.rs:207–211`).
- **`wonders_built`: no counter.** The site is `Sim::activate`
  (`city.rs:2420`) beside `self.is_wonder_building(ty)` at line 2490,
  **before** the `building_is_city` split (~2506), for `captured` true and
  false alike. Not the non-city tail: the Forbidden City runs the city
  branch. The capture member loop (`cities/capture_view.rs:156–170`) already
  gates on `not_civilian` (`0x2000`) and activates with `captured = true`, so
  a counter there reproduces the capture rule with no further work.
- **Trap: two bitmasks with counter names.** `Cities::wonders_built`
  (`cities.rs:63`) and `Sim::wonders_held` (`economy/holdings.rs:572`) are
  type bitmasks, not these counters. Do not merge them.
- **Trap: the American proxy.** `american_free_wonder` (`city.rs:1827`, tail
  ~1855) reads `census.wonder_mark == 0` and no standing wonder in place of
  `wonders_built == 0`. They part when a leader has ever activated a wonder
  (built, or captured then lost) and now holds none: the census mark trims
  to 0 (`Sim::note_wonders`, `ai/ai_census.rs:1276`), the counter does not,
  so the crate grants the free arm the original refuses. Feed it from the
  counter.
- **`wonders_held`: no counter.** `Sim::note_wonders` batches every close
  before every open, which can under-count against the original's event
  order; count at `Sim::activate`'s wonder test instead.
- **`senates_built`.** `senate_arm_steps` (`cities/activate.rs:251`, call
  `:261`) reaches `SimArms::senates_built_inc` (`:394`), an empty SEAM.
- **`forts_built`.** None; increment with the `fort_mark` take.
- **Checksum.** `checksum/leaders.rs` lists all seven leaves (`:105–108`,
  `:175–177`, `:189`); `leader_long` (`:949`) has an arm for none. Each sits
  in `LEADERS_HELD_OUT` at count 2 (`crates/rondata/src/diff/
  checksum_lines.rs:1787, 1788, 1817, 1835, 1837, 1850, 1851`); a port of a
  leaf adds its arm and drops its entry. The verdicts give no figure beyond
  that.

### 13.4 Reached by a plain game

A skirmish reaches every writer: fort and oil-well takes and give-backs by
construction and destruction (oil from the Industrial age); supply takes at
supply-unit births and deaths; `wonders_built` and `wonders_held` by
construction (`Wall::do_construct`, `(0, 1, 1)`) and by capturing a city that
holds a finished wonder or is a Forbidden City; `senates_built` and
`forts_built` by construction. `Leaders::clear` runs at game close and in
multiplayer out-of-sync recovery. `LeaderData::walk_data (006d6750)`
overwrites all seven on a save load, outside a plain game. Whether the
`set_type` supply pair or a `(0, 1, 0)` Senate transfer is reached is open
(13.6).

### 13.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/marks-built-*.md`); no diff backs it yet.

Verdicts: 63 confirmed, 7 corrected, 5 open. The corrections that changed a
rule:

- the array is ten slots, and `build_game` zeroes `supply_mark` in all ten;
- `Leaders::clear` has two callers, not four;
- `oil_well_mark`'s give-back also needs `build_flags & 0x40`, and writes
  four fields of the record;
- the `wonders_built` crate site is beside line 2490, outside the city
  split, not the non-city tail;
- `defeat_by` hands over no wonder;
- a fort taken with a city is **not** recounted for the captor: forts carry
  `0x2000`, which the member loop exempts, and no capture route to a fort
  was found.

Confidence is high throughout. The `wonders_built` increment, its missing
gate and its single return are listing-read; the capture route is
listing-read at `capture_city`'s gate, its member activation and its head
activation, and the wonders' flags are the shipped `buildingrules.xml`'s,
held to the program's loaded words by an existing guard (`docs/DATALAYER.md`).
`senates_built`'s `captured` register was traced through the listing. Every
leaf's absolute-address writer net was swept, and the register-relative
hits outside a leader are named in the verdicts.

### 13.6 Not established

- **A capture route to a Red Fort**, or to any fort, for `wonders_built` and
  `forts_built`. None was found; a sweep of every caller of the `activate`
  slot (`0x1a8`), its predicate against `is_fort`, would settle it.
- **`is_wonder_building`'s equivalence to `0x20e..=0x21e`.** The crate asks
  the tech tree's `wonder` kind; a test that the tree files exactly those 17
  types settles it.
- **Whether a plain game retypes a unit across `unit_flags2 & 0x40`**, which
  is what reaches `supply_mark`'s `set_type` pair. A capture with a supply
  unit retyped, or a reading of every `set_type` caller, settles it.
- **Whether a Senate travels a `(0, 1, 0)` transfer** (a cityless foreign
  Senate adopted by `City::find_buildings`). The predicate is settled; the
  route is not.

## 14. The six `*_queued` counters and `scholar_militia`

### 14.1 Leaves

- `scholar_militia`, +0x9f0, `int`.
- `barracks_queued` +0xa10, `stable_queued` +0xa14, `factory_queued`
  +0xa18, `combat_queued` +0xa1c, `dock_queued` +0xa20, `air_queued`
  +0xa24, all `int`.

All seven are zeroed by `Leader::init (006e3930)` for a tribe `>= 0` only
(`6e4c14` and `6e4c3e..6e4c5c`), never recounted, and restored raw by a load.

### 14.2 The rule

#### `scholar_militia`

**Writer.** One live store, at the end of `Leader::track_unit_type
(006e0dd0)` (`6e0ecd`). There is no inlined copy. It is reached from six
call sites: `Unit::set_type` twice, `Unit::close`, `Objects::init_unit`
twice and `SpellType::cast_create_decoy`. Each call site guards it the same
way: `unit_masks & 1` clear, the unit its own captain (`+0x8e < 0`), and
`control_cost != 0`, or `is(FIGHTERBOMBER)`, or a government hero.

**Step**, at every `track_unit_type(type, delta, uid)` that passes:

```
if uid >= 0 && type.is(MILITIA 0x42) {
    f = units[who][uid].former_type      // +0x54, signed
    if f >= 0 && types[f].is(SCHOLARS 0x34) { scholar_militia += delta }
}
```

There is no floor and no clamp. A militia that fails the Scholar test leaves
the function; only a non-militia or `uid < 0` goes on to the SPECIALFORCES
arm.

**Events.** +1 at a Scholar's To Arms; -1 at its Back to Work and at its
death.

**`former_type` (+0x54) lifecycle.** A port that keeps the counter needs
this:

- `cast_to_arms` sets it to the current type before the retype.
- `cast_civilian` sets it to Citizen only if it is 0, and otherwise keeps it.
- `cast_immortal_hinf (00670600)` and `cast_immortal_arch (00670740)` set
  it to the current immortal type.
- Death leaves it.
- `Unit::init (00612100)` zeroes it (`6121f6`) only **after** the birth +1.

**The stale-slot +1.** A unit born through `Objects::init_unit (0065e0c0)`
reaches `Unit::init`. That calls `Object::init`, then `SubObject::init`,
then `Unit::set_type` under `SET_TYPE_INIT` (`612182`). The birth +1 runs
there against the slot's **previous** occupant's `former_type`, because the
clear at `6121f6` comes later. `Unit::close` does not clear `+0x54`, and it
clears bit 0 of `unit_masks` (`60fa35`), so the stale +1 passes the mask
guard. `Objects::find_free (0065ad60)` hands
out the lowest free slot, so slots recycle constantly. A dead Scholar
militia leaves a Scholar id behind, and so does a Scholar that ever went to
arms and back. Two plain-game routes create a militia-typed unit this way:

- `SpellType::cast_create_decoy (00674370)` copies militia (its exclusions
  are `0x32..0x35`, caravans and `is(MERCHANT)`). Its own undo
  `track_unit_type(type, -1, new)` runs after `+0x54` is 0 and misses. It
  then sets `unit_masks |= 1`, so the decoy's death never takes the +1
  back. Net +1 for the life of the game.
- `Unit::swap_team (006118d0)` (from `cast_bribe`, `cast_assimilate`,
  `Leader::defeat_by`) re-creates the unit in the new owner's slot and does
  not copy `+0x54`. A bribed militia counts +1 for its new owner exactly
  when that slot is stale with a Scholar id, and its later -1 sees 0: a
  permanent +1. A genuine Scholar militia that is bribed loses its Scholar
  status. The old owner's -1 at close is correct, and the new owner never
  counts it.

Conversions in place (`cast_to_arms`, `Object::eject_contents (0064cd20)`'s
Citizen-to-militia turn) and the barge spells are not at risk. A port needs
the lowest-free-slot reuse modelled.

The immortal spells cannot strand the count. Their -1 is for the immortal
type, which does not answer `is(MILITIA)` (unless the shipped `is_list` says
otherwise).

**Readers.** `TypeData::get_cost (00664090)` subtracts it from the
Citizen's count and adds it to the Scholar's. `LeaderData::log_data
(006e5110)` logs it.

#### The six `*_queued` counters

**The rule**, `track_queued(who, t, d)`, out of line in
`Leader::track_queued (006e0f30)`:

1. If `t < 0`, stop.
2. Step `num_queued[t]` (u16) by `d`, unless it is 0 and `d < 0`.
3. Require `0x32 <= t < 0x19e` (`TypeData::is_unit_type (004707e0)`, the
   only implementation of vtable slot `+0xc`) and `attack (+0x1e8) != 0`.
4. Choose by `w = where (+0x40)`, by equality:
   - `0x1ab`: barracks and combat;
   - `0x1ac`: stable and combat (not Auto Plant `0x1ad`);
   - `0x1ae`: factory (Siege Factory, not `FACTORY 0x1af`);
   - `0x1b0`: dock (not `0x1b1` or `0x1b2`);
   - otherwise: air if `domain (+0x218) == 2`, else nothing.
5. For each chosen counter `c`: `if !(c == 0 && d < 0) { c += d }`, each
   tested on itself. A skipped barracks or stable step still runs combat's
   own test.

**Never a clamp.** The guard skips a negative step only at exactly 0. A
`-n` from `0 < m < n` gives `m - n < 0`. Every inlined `-1` copy tests only
`== 0` (then `decl`), so a negative counter keeps falling on each `-1`. The
`+1` copies are a bare `incl`.

**Writers.** These are inlined copies of the rule, devirtualized to
`0x32 <= t < 0x19e`, and the out-of-line callers:

| site | `d` |
|---|---|
| `Build::queue_up (00620f40)`, queue append | +1 |
| `Group::action_spell (006fe1a0)`, BUILDAIR | +1 |
| `Build::unqueue (006207c0)`, completion and cancel | -1 |
| `Build::clean_queue (00620b60)`, per entry, last first | -1 |
| `Unit::action_unqueue (005e1f20)`, carrier plane cancelled | -1 |
| `Unit::process (00610bc0)`, carrier plane finished (absolute stores) | -1 |
| `Unit::close (0060ee50)` `60efed`, carrier lost with `n` queued, then its `num_queued (+0xa0) = 0` | -n on `current_upgrade(FB)` |
| `Leader::gain_tech (006dcb60)` `6dda28`/`6dda47`, FB-line upgrade, per live carrier | -a0 on old FB, +a0 on new FB |
| `Leader::gain_tech` `6ddd04`/`6ddd4a`, unit-line upgrade, per queue entry | -1 on the entry's type, +1 on the new |

On the retarget's graft-chain arm the -1 is the **new** type's, so
`num_queued[old]` stays one high. Within one bin it nets to zero on the six.
`Build::unqueue` is called from `Build::do_queue` (completion),
`Build::action_unqueue` (cancel) and three `Build::activate (00623e20)`
sites. The four `Wall::*` copies are dead: a wall's type is a build type.

**`air_queued`** is the only leaf with `-n` writers. Each carrier +1 also
stepped it, so `air_queued >=` every carrier's `num_queued` in steady
state, and going below zero needs an earlier drift not found in a plain
game.

**Readers.** `LeaderData::get_support_count (006da110)` takes five bins
(`queued + units`, gated on `+0x2f4 & 1` and attack; no combat arm) and
feeds `TypeData::get_cost`. `log_data` logs all six; it is
`combat_queued`'s only reader.

### 14.3 In the crate

- **`scholar_militia` is derived, not kept.** `Sim::worker_support`
  (`crates/sim/src/economy/pricing.rs:895`) counts live units with
  `type_index` in `0x42..=0x44`, `rare` in `{0x34, 0x35}`, `captain`,
  `!decoy`. That cannot carry the stale +1. Those ranges are stand-ins for
  two kind tests whose `is_list` data was not opened. The kept counter goes
  in `Sim::track_unit_type` (`crates/sim/src/units/store.rs:755`), with the
  unit index added, and is exposed to `Sim::leader_long`
  (`crates/sim/src/checksum/leaders.rs:949`).
- **The queued bins.** `Muster::queued_by_group` is moved by
  `Sim::track_tree_queued` (`crates/sim/src/cities/queue.rs:2017`) and
  `Sim::untrack_queued` (`:2078`), and retargeted by
  `Sim::retarget_queued_to` (`:1964`). Carriers are in
  `crates/sim/src/air/carrier_queue.rs` (`carrier_action_spell :60`,
  `carrier_queue_tick :137`, `unit_action_unqueue :167`). `untrack_queued`
  tests `!= 0` on `queued_by_type`, `tech.queued` and `queued_by_group`
  separately: the original's skip at 0, not a clamp.
- **SEAM: no `-n`.** `Unit::close`'s `-num_queued` and `gain_tech`'s
  carrier pair are missing (named in `Sim::upgrade_units_to`,
  `cities/queue.rs:1890`). Both land in `air_queued`.
- **Trap: derive from the bins, not from `queued_by_type`.** After a
  graft-chain retarget, `num_queued[old]` stays one high and a sum over
  types carries it; the bins do not.
- **Trap: bin shape.** The crate keys bins per `where_` (`load.rs`:
  `group = if attack != 0 { where_ }`). The original has exactly four
  `where` bins and an air fallthrough on `domain`. Whether they coincide on
  the shipped data is open. A separate `air_queued` counter is the safer
  port, since several `where_`s may train armed air types.
- **`combat_queued`** has no crate home. Port it as a counter with its own
  `!= 0` test at the barracks/stable sites. Or port it as
  `queued_by_group[BARRACKS] + queued_by_group[STABLE]`, which is exact
  while all three move by paired ±1 from zero. That holds in a plain game,
  because `-n` is FB-only and FB is air.
- **Checksum.** `Sim::leader_long` has no arm for the seven. Each is
  `("<leaf> (unheld)", 2)` in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs`: `scholar_militia :1822`,
  `barracks_queued :1766`, `stable_queued :1836`, `factory_queued :1786`,
  `combat_queued :1773`, `dock_queued :1783`, `air_queued :1754`). A port
  adds the arm to `leader_long` and removes the entry.

### 14.4 Reached by a plain game

Yes, for every live writer; carriers and the FB upgrade in the Modern
ages. The
stale `scholar_militia` +1 is reached through a General's decoy over a
militia, or a Spy's bribe of a militia, landing in a slot that last held a
Scholar id. Scenario and cheat creators (`ScenarioFuncSet::add_unit`,
`ScenarioEditor::add_unit`, `Game::action_cheat_init_unit`) are out of
plain play. `air_queued` below zero is legal code but, as far as read,
unreached in a plain game.

### 14.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/queued-*.md`); no diff backs it yet.

Verdicts: `scholar_militia` 13 confirmed, 1 corrected, 1 open; each queued
leaf 16 confirmed, 1 corrected, 1 open. The writer nets over the listing
found 75 stores in exactly 12 functions for the six, and 2 leader stores
for `scholar_militia`. No writer was missed. The corrections:

- **The floor.** "Never below zero" was corrected to "skip a negative step
  at exactly 0". A negative counter is not floored by the `-1` copies. This
  changes the rule.
- **The immortal spells** cannot strand `scholar_militia`. No change to the
  rule.

Confidence is high on writers, predicates and steps (from the listing:
`6e0ecd`, `6e0f97..6e0fa7`, `6209d7`), and high on the stale +1 mechanism
(each link read). It is medium on below-zero `air_queued` being unreached,
and medium on which buildings train `domain 2` types (data). No kept-dump
value is cited.

### 14.6 Not established

- **How often a plain game meets the stale `scholar_militia` +1.** A
  capture settles it: a Scholar sent to arms and killed, then a General's
  decoy over a militia, reading `scholar_militia` from the leaders line
  before and after (expect +1). The same probe settles whether the `+0x8e`
  captain guard passes on a recycled slot. `+0x8e` is not reset before
  `Object::init`, so the read predicts it passes when the previous
  occupant was its own captain.
- **`Build::activate`'s cross-queue cancel.** Does the cancelled item
  (`BuildData::find_in_queue` after `BuildData::count_queue`) ever have an
  attacking unit type? Only then does it step the six. Reading
  `Build::activate`'s item origin settles it.
- **Data not opened.** Each of these is settled by reading the shipped
  table:
  - the `is_list` of MILITIA and SCHOLARS (the crate's `0x42..=0x44` and
    `{0x34, 0x35}`);
  - whether an Immortal type's `is_list` names MILITIA;
  - whether any building trains a Militia-line type through `Build::train`;
  - the `where`/`domain` columns that say whether the crate's per-`where_`
    bins coincide with the original's four bins and air fallthrough.
- **A plain-game drift of `air_queued` below a carrier's count**, the only
  way to below zero. None was found. A capture that loses a carrier with
  planes queued, diffed on `air_queued`, would witness the `-n`.

## 15. Missiles, repairs, bribes and nukes

### 15.1 Leaves

- `repair_stamp` (+0x208), `int`: the frame a repair was last refused.
- `missiles_used` (+0x7c0), `int`: V2 and Cruise Missile rounds landed.
- `units_bribed` (+0x8f4), `int`: units taken by Bribe.
- `nukes_launched` (+0xa38), `int`: nukes out holding a strafe.

Each leaf has exactly one event writer; nets over the whole listing found
no other. All four lie inside the header's bulk writers (and `Leader::close
(006b7f80)`'s memset, which has no direct caller).

### 15.2 The rule

**`repair_stamp`.** Writer `Unit::do_repair (005ee420)`, index the
repairer's owner (`unit+0x9`, a byte). Event: a repair step that is
refused. Before charging, `do_repair` walks the six goods `g` in `0..=5`. A good
counts when it is `type_avail(g, 1)` and has a nonzero price. Each counted
good has a due of 0 or 1 this step (1 when the truncated quotient before the
step is below the one after it; the float `due` is the repair mechanic's,
item 3346). The first counted good whose stock `< due[g]` (signed) refuses
the step:

1. kill the repairer's current order;
2. if `frame − repair_stamp[owner] ≤ 150` (signed, wrapping), return;
3. else `repair_stamp[owner] = frame`;
4. then the refusal message, for the console player only.

The stamp is written for every player, AI or human, before the console
test. It is an assignment, not a step. A refused call charges nothing (the
charge loop is reached only past the goods test). Edge: because `due` may be
0, a priced good with a **negative** stock refuses even when this step owes
none of it. Keep the comparison signed and against `due`.

Reset: 0 at `Leader::init (006e3930)`, an explicit store on both arms
(the memset runs only when its second argument is negative). Nothing
mid-game resets it; `reset_score` does not. Readers: `do_repair`'s
throttle and `log_data` only.

**`missiles_used`.** Writer `Ammo::do_damage (00678060)`, index the round's
`who` (`AmmoData+0x3c`, the shooter's player at launch). The path, in order:

1. Guards: object `o ≥ 0`, `who ≥ 0`, graphic-piece type `≠ 9`.
2. Splash arm only: `splash_area ≠ 0`.
3. The shooter object (`objects[who][o]`, the missile itself) has a type
   with `obj_masks & 0x8000000`; else straight to `hit_target`, no count.
4. Note `is(0x13b, 0)` (the Nuclear Missile line, non-strict).
5. Shield: find the landing tile's owner `L` (a signed byte). On an ocean
   tile, `L` is the first owned (`≥ 0`), on-map, non-ocean cell of a
   120-entry offset ring; none found leaves −1. If `L ≥ 0`, `L` holds
   `MISSILE_DEFENSE_BONUS` (693) and `L ≠ who`, the round closes, no count.
   The test is `≠`, not "enemy": an ally's shield stops it too.
6. A Nuclear Missile line shooter takes the nuke arm and returns, no count.
7. Otherwise the `S_NUKE_HIT` sound, then `missiles_used[who] += 1`, then
   `hit_target` and the splash.

Reset: 0 at `Leader::init`; never decremented. Reader: `get_support_count
(006da110)`, feeding `TypeData::get_cost (00664090)`. On its by-type
fall-through, for exact type `0x139` (`V2ROCKET`) or `0x13a`
(`CRUISEMISSILE`), it adds `missiles_used[player]` to the count. So each
landed V2 or Cruise round makes that player's next one dearer. The term
sits after the group-count arm (types with `progression & 1` and
`+0x1e8 ≠ 0` return first). In the shipped `unitrules.xml` both missiles
have `PROGRESSION 0`, so the term is live. The Americans' bomber discount
below it does not touch them.

**`units_bribed`.** Writer `SpellType::cast_bribe (00672d00)`, index the
caster's player (`param_2`). Exemptions first, none of which counts:

- own target (`param_2 == param_4`): a separate arm;
- an ally's unit (the target is the caster's `who`, or relation 2 both
  ways), unless teams are locked (`team_style ∉ {0, 8, 11}`) **and** the
  target leader lacks `leader_flags & 2`: `unpay_cast_costs`;
- a unit with `has_general(target, 0, 0x165) ≥ 0` (CEO cover):
  `unpay_cast_costs`;
- a non-unit target: the building/city arm, which never counts.

A unit past these meets no further test. Every message branch joins at one
point: `units_bribed[caster] += 1`, then `Unit::swap_team` (vslot `0x94`).
The count precedes the swap.

Reset: `Leader::reset_score (006e37f0)` stores 0. `Leader::init` calls it
(and has no store of its own), and `Setup::build_game (005ac190)` calls it
again for every slot with `leader_flags & 1`, inside its `param_2 == 0`
block. Never decremented. Readers: `AchieveChartPane::set_chart_data
(007f2c40)` (UI) and the checksum; no sim decision, not in `log_data`.

**`nukes_launched`.** Writer `Unit::come_out (00617c10)`, index the unit's
owner (`unit+0x9`). After placement, it counts only when all of these hold:

1. the unit `is(0x13b, 0)` (Nuclear Missile line);
2. its order list head `+0xdc` and current order `+0xcc` are non-null;
3. the current order's `get_strafe_order` (`UnitOrder` vslot `0x100`) is
   non-null, which only a `StrafeOrder` answers (every other order's slot is
   the folded zero, from the PE vtable bytes and RTTI);
4. that order's `ox` (+0x8) and `whom` (+0xc) are both `≠ −1`.

Then `nukes_launched[owner] += 1`. Every caller of `come_out` is a site: the
silo launch, ejections, the contents of a killed holder.

Which nukes qualify: `Build::do_missile_launch (00622670)` calls `come_out`
for every popped missile; its order test gates only the announcement and
sound (and its `STRAFE` arm is dead, as `is_ally(leaders[p], p)` is 1). And
`add_strafe_order (005e48c0)`, the only maker of a `StrafeOrder` (plus
`copy_order (0072f900)`, which copies one that exists), turns a strafe by a
bit-27 unit at a target `Object::valid_target (00648ba0)` accepts into
`add_air_attack_ground_order`. A Nuclear Missile has bit 27. So **a nuke
counts only when it was ordered to strafe an object that
`ObjectData::valid_target_const (006472c0)` refused at order time**: a
non-enemy, a slot outside `0..7`, an inactive, off-map or wrong-domain
target. A nuke ordered at a valid enemy, or at the ground, holds
`AirAttackGroundOrder` and does not count. `do_garrison`'s strafe passes
`−1,−1` and fails step 4.

Reset: 0 at `Leader::init`; never decremented. Reader: `log_data`.

### 15.3 In the crate

- **`repair_stamp`.** The event is `do_repair_steps`
  (`crates/sim/src/orders/jobs.rs:1802`), the throttle at `:1915`
  (`frame.wrapping_sub(feedback_stamp) < 0x97`, the same test), the signed
  `due` comparison at `:1912`, owner index at `:2102`. Storage is a stub:
  `SimDoRepair::feedback_stamp`/`set_feedback_stamp` at `:2105`–`2110`.
  **Trap:** the stub returns `i32::MIN`, which makes the throttle **always
  return** before the store, not "always due" as its SEAM says. Harmless
  while both arms are empty. Port: a per-player `i32`, 0 at init, behind
  those two methods.
- **`missiles_used`.** The event is `Sim::land`
  (`crates/sim/src/combat/fight/projectiles.rs:560`): shield at
  `:608`–`614`, nuke arm at `:617`–`620`. Put the `+1` keyed `p.owner`
  after the `is_nuke` return, under `mask::MISSILE`, before the splash
  walk. Storage beside `Nukes::used` (`combat/nuke.rs:76`). The reader is
  `nuke_ramp_extra` (`economy/pricing.rs:281`–`300`), whose doc says the
  term is unmodelled: add `if t == 0x139 || t == 0x13a { extra +=
  missiles_used[who] }`, on the by-type count only, where it already feeds
  (`of_type`, `:851`–`855`). **Trap:** `crates/sim/src/ids.rs` names
  `0x139` `CRUISE`, but `0x139` is `V2ROCKET` and `0x13a` `CRUISEMISSILE`
  (`air/airbase.rs:42`/`48` have them right). Key on the numbers.
- **`units_bribed`.** Already counted: `tech[a].bribes += 1` at
  `crates/sim/src/spells/bribe.rs:42`, field `PlayerTech::bribes`
  (`tech.rs:771`), test `spells/cast/tests.rs:544`. Not wired: the value
  match in `checksum/leaders.rs` falls to `_ => None` (`:1305`). Add
  `"units_bribed" => one(i64::from(self.tech[w].bribes))`. Gap:
  `Sim::cast_on` (`spells/cast.rs`) re-tests only same owner and CEO cover,
  not the ally refusal. The order-time `bribe_arm`
  (`spells/spell_valid_target_view.rs:183`–`186`) carries the same ally
  rule, so the gap shows only when an alliance forms between order and
  cast.
- **`nukes_launched`.** The event is `Sim::come_out`
  (`crates/sim/src/units/garrison.rs:376`). Port after placement, gated on
  `air_line_is(u, NUCLEARMISSILE)` and `Body::Strafe` with `target:
  Some(_)` (`StrafeOrder::target` is `ox/whom`, `orders/order.rs:498`–`501`).
  Already faithful: `do_missile_launch` (`air.rs:1799`) calls `come_out`
  unconditionally (`:1825`), and `add_strafe_steps`
  (`orders/order_list.rs:1854`, `:1873`) converts a missile's valid strafe.
- **Checksum.** Rows `checksum/leaders.rs:74`, `:141`, `:190`, `:259`. Each
  port removes its `("<leaf> (unheld)", 2)` entry from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1814`, `:1816`, `:1821`,
  `:1846`).

### 15.4 Reached by a plain game

- `repair_stamp`: yes, any repairer short of a charged good after frame 150.
- `missiles_used`: yes, from the Industrial age with V2 or Cruise Missiles
  landed.
- `units_bribed`: yes, Bribe on an enemy unit.
- `nukes_launched`: the writer runs on every `come_out`, but the predicate
  needs a nuke holding a strafe at a refused target, which an ordinary
  launch never carries. Expect 0 in plain play until a capture says
  otherwise. `ScenarioFuncSet::add_unit` (triggers) is among
  `add_strafe_order`'s callers, out of plain play.

### 15.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/odd-counters-*.md`); no diff backs it yet.

Verdicts by leaf (confirmed / corrected / open): `repair_stamp` 15 / 1 / 0,
`missiles_used` 16 / 0 / 0, `units_bribed` 15 / 1 / 0, `nukes_launched`
10 / 2 / 1. The corrections that changed a rule:

- `do_missile_launch`'s order test gates the announcement, not the
  release; `come_out` runs for every popped missile.
- A nuke strafing a valid object does not keep a `StrafeOrder`:
  `add_strafe_order` converts it to an air-attack-ground. This is what
  makes `nukes_launched` near-dead in plain play.

The other two corrections changed no rule: `Leader::init`'s memset is
conditional (the explicit store zeroes `repair_stamp` either way), and
Bribe's own-target arm draws down the caster's `+0x96`, not the target's
(not a count). Added by the adjudication: the negative-stock edge of the
repair refusal, and the shipped `PROGRESSION 0` of both missiles, which
keeps the `missiles_used` price term live.

Confidence is high for every writer and predicate, each read from the
listing over its branch (`5ee99f`..`5eeaa4`, `678186`..`67861b`, every
route into `67306a`, `61883d`..`6188fa`), and `get_strafe_order` from the
PE vtable bytes and RTTI. No kept dump was consulted.

### 15.6 Not established

- **Whether any plain-game path gives a nuke a strafe at a refused
  target** (the console's right-click on a non-enemy, `Group::action_flight`'s
  `NUCLEARMISSILE` arm, the AI's nuke choice); not read. A capture with
  nuke launches read under `log_data`'s `nukes_launched` settles it: a
  value above 0 after launches at enemy targets refutes the reading.
- **Which types carry `obj_masks` bit 27 outside the nuke line**, and
  **whether a missile ever lands more than one round per detonation** (both
  readings assume one). Data, held in the crate's `mask::MISSILE`. A
  capture with a V2 landing and a later V2 priced settles both.
- **Whether a stock can go negative**, which makes the repair edge
  reachable. The economy's question; the charge clamps only the second
  ledger.
- **`hit_target` order.** The crate calls it at `projectiles.rs:577`,
  before the shield and nuke tests; the original only after them. It
  matters only if it has side effects on a round that then dies; not
  adjudicated.
- **Bribe's own-target arm** skips `unpay_cast_costs` while the crate's
  refuses with `unpay_cast_mana`; not adjudicated, the bribe owner's.

## 16. `cannon_time`, `handicap` and `multi_diff`

Three words a game build writes from the lobby and that nothing moves in a
plain game. None of them is 0 by rule: `multi_diff` holds the lobby's
difficulty byte.

### 16.1 Leaves

- `cannon_time`, +0x48, `ulong`, a count of uses. Slot `i` at
  `0xe3a3d8 + i·0x6eec`.
- `handicap`, +0x4c, `int`, a row 0..20. Slot `i` at `0xe3a3dc + i·0x6eec`.
- `multi_diff`, +0x50, `int`, a difficulty level. Slot `i` at
  `0xe3a3e0 + i·0x6eec`.

### 16.2 The rule

**`multi_diff`.**

- **Writer in play: `Game::init_handicaps (0058abf0)`**, once per game
  build, after the leaders are cleared, for each seated slot 0..7
  (`leader_flags & 1`). Human and computer slots alike: no test of the
  human bit sits on the path. Slots 8, 9 and unseated slots keep the
  memset's 0. The predicate, in order:
  1. a scenario (`Game::is_scenario (0043ec70)`) with `semaphore` bit 26
     clear: no store;
  2. bit 2 (multiplayer) or bit 26: `aidiff[i]` if non-negative, else 2.
     `aidiff[i]` is the largest `Player.diff` over occupied computer
     `Player` records with `who == i`, starting from −1;
  3. otherwise: the lobby's `difficulty` byte (`Game+0x2b`, that is `info`
     +0xc, the lobby union +0x18, `difficulty` +0x7).
- **Bit 26** (`Game+0x823 & 4`) means "a scenario chosen from the skirmish
  setup window". Its one setter is `SetupWin::setup_game (005b3860)`, when
  the setup combo 0x36 picks entry 10. `Game::run_scenario (005860c0)` and
  `Game::run_playback (00586ea0)` clear it. A random-map skirmish never sets
  it.
- **Writers off the plain path:** `DropControl::process_drop (00959500)`
  stores 3 when a dropped network player's slot has no other occupied
  `Player` record (any kind, person or computer, keeps it). Multiplayer
  only. `ScenarioRead::load_leader_chunk (009ac080)` copies the chunk's
  +0xe8 after a `Leader::init`. `ScenarioFuncSet::set_leader_difficulty
  (009e5350)` stores its 1..6 argument raw, on an occupied slot with
  `leader_flags & 2`. The editor stores in `IFaceEditorNations::
  on_combobox_select (009897d0)` and `ScenarioEditor::change_difficulty
  (0099bc00)` are editor only.
- **Dead or overwritten:** `Leader::set_diff (006d2470)` has no reference
  at all. `Leader::Leader (006d2bf0)` stores 2, which the memset at
  `Leaders::init` erases.
- **Step and reset:** none per frame. Reset at the next game build. The
  bulk writers apply as in the header.
- **Reader:** `LeaderData::get_diff (006ec000)` and its inlined copies, and
  `get_gather_handicap (006d66a0)`. Under bit 2, `get_diff` returns the
  leaf raw. Otherwise it returns the leaf only under `0x821 & 0x10`,
  without `0x822 & 2`, with the leaf `>= 0`, and else the lobby byte. In a
  solo random map both arms give the same byte, so the leaf never decides
  anything there. It still holds that byte.

**Port:** each seated slot 0..7 gets `multi_diff = lobby.difficulty` (0..5,
as stored). A builder must not emit 0.

**`cannon_time`.**

- **Seed: `Leader::init (006e3930)`**, on every call and both tribe arms.
  The negative-tribe test skips only the memset. The value is
  `CANNON_TIMES_ROW[lobby.cannon_times]` (`Game+0x33`, rows of stride
  0x58, field +0x3c), with the shipped `rules.xml` `cannontimes` rows
  `[0, 1, 2, 4, 8]`. The seed is not gated on seating, so it covers slots
  0..9.
- **Step: `TurnControl::start_cannon_time (00956500)`**, on an applied
  command 0x4d. Exemptions, all must pass: the sender `who >= 0`; nobody is
  already in cannon time (`cannon_time_who < 0`); `cannon_time != 0`. Then
  `cannon_time -= 1`. It also stamps the cannon-time `who`, frame, saved
  speed and tick. A refusal posts feedback only when `who` is the console's.
  `start_cannon_time` and `process_cannon_time (009464f0)` test no
  semaphore.
- **The only issuer**, `CommandManager::issue_cannon_time (00941260)`,
  refuses without `semaphore[0] & 4` (multiplayer). It then sends 0x4d only
  if the console leader's `cannon_time` is non-zero (unsigned `> 0`).
- **Not writers:** `Leader::reset_score (006e37f0)`. The store at 006e3f6a
  in `Leader::init` goes through the +0x6eb8 `LeaderDataEncrypt*`, not this
  leaf.
- **Reset:** the next game build. The bulk writers apply as in the header.
- **Readers:** the two guards above, plus `IFaceMainBase::update_visibility
  (00811d40)`, which greys the button (UI only). `log_data` does not print
  it.

**`handicap`.** Offsets in this rule are `Player`'s: `flags` +0x30, `who`
+0x33, `team` +0x34 (signed), `handicap` +0x35, `diff` +0x38.

- **Writer in play: `Game::init_handicaps (0058abf0)`**, the same call and
  seating loop as `multi_diff`, over the eight lobby `Player` records.
  1. **Pass 0:** `own[i]` is the largest `handicap` over occupied records
     with `who == i`, of any kind, starting from 0.
  2. `sum[i] = own[i]` if `Game::is_cooperative (00594810)`, else 0.
  3. **Human pass**, over records with `(flags & 5) == 5`, sender `h` its
     handicap, slot `w` its `who`, for each seated `i`:
     - `w == i`: `own[i] = max(own[i], h)`, `sum[i] += h`;
     - `w != i` and `team != get_team(i)`: `enemy[i] = max(enemy[i], h)`;
     - `w != i`, same team: `mates[i] += 1` unless `team == 8`, and
       `sum[i] += h`.
  4. `top = max(0, max over seated i of sum[i])`. The maxima of `own` and
     `enemy` are also computed and discarded.
  5. The store, every division signed and truncating toward zero:

         if mates[i] == 0: h = clamp(top − sum[i], 0, 20)
         else:             h = clamp(((top − sum[i]) / (mates[i] + 1)
                                      + max(0, enemy[i] − own[i])) / 2, 0, 20)
         handicap[i] = h / max(1, num_teams − 1)      // num_teams: Game+0x6a4

  Slots 8, 9 and unseated slots: 0. A computer record's byte enters
  through `own` (and so `enemy − own`) and, under `is_cooperative`, the
  seed of `sum`. `Player::init` zeroes every byte, so this is moot in a
  plain game.
- **Off the plain path:** `ScenarioRead::load_leader_chunk (009ac080)`
  stores the chunk's +0xe0 at three sites, each behind `semaphore[3] & 4`
  clear (bit 26).
- **Step and reset:** none per frame. Reset at the next game build.
- **Readers:** `LeaderData::get_handicap (006da740)`, called from
  `ObjectData::train_time (006508c0)` (two sites), `World::
  compute_reg_territory (006b0bb0)` and `get_gather_handicap`'s tail, every
  caller behind `semaphore[0] & 4`. `get_handicap`'s difficulty shift
  (+5/+10/+16) runs only with that bit clear, so no caller reaches it.
  `get_handicap_level (006d6740)` has no caller.

### 16.3 In the crate

- **No leaf is held.** All three are in `LEADER_FIELDS`
  (`crates/sim/src/checksum/leaders.rs:53-55`), and `Sim::leader_long`
  (`:949`) has no arm for them. Each is a `(unheld)` entry of
  `LEADERS_HELD_OUT` with count 2: `cannon_time` at
  `crates/rondata/src/diff/checksum_lines.rs:1771`, `handicap` at `:1793`,
  `multi_diff` at `:1815`. Each arm below removes its entry.
- **`multi_diff`:** add an arm returning `i64::from(self.ai.lobby.difficulty)`
  for each seated slot. The walk already skips unseated slots.
  `Sim::ai_difficulty` (`crates/sim/src/ai/ai_types.rs:137`) is `get_diff`'s
  solo arm, and `diff_of` (`crates/sim/src/ai/army/target_gate.rs:133`)
  uses it.
- **Trap: the live `lmulti` answers −1** (`target_gate.rs:397`). The
  original holds the lobby byte. Nothing reads it while `semaphore()`
  answers `[0; 3]`, but the port points `lmulti` at the same value as the
  checksum arm so the two cannot drift.
- **Trap: the default lobby is not the answer.** `impl Default for Lobby`
  (`crates/sim/src/ai.rs:1113-1119`) sets `difficulty: 0`, but the harness
  fills the lobby per capture. The pinned pairs carry 5
  (`crates/rondata/src/diff/persian_classical_pair.rs:24`), 4
  (`french_medieval_pair.rs:23`) and 2 (`aztec_word.rs:22`). A constant 0
  is wrong on all of them. `crates/rondata/src/diff/leader.rs:58` lists the
  leaf as unmodelled ("an input, not state").
- **`cannon_time`:** add `cannon_times` to `ai::Lobby`, parsed from the
  dump's `CANNON_TIMES`, and an arm returning the row's count.
  `Command::CannonTime` is decoded (`crates/rondata/src/commands.rs:996`)
  and consumed by nothing. The step needs a 0x4d arm only for a
  multiplayer stream.
- **`handicap`:** an arm returning 0 for every seated slot is right only
  while the lobby carries no per-player handicap. The full computation needs
  each `Player`'s `team`, `handicap` and `flags`, and `num_teams`, which
  `rondata::recgame::Slot` carries and `ai::Lobby` does not.
  `Holdings::handicap` (`crates/rondata/src/diff/leader.rs:55`) is a
  different quantity.

### 16.4 Reached by a plain game

- `multi_diff`: every random-map game reaches the store. It is frozen at
  the lobby byte and decides nothing.
- `cannon_time`: the seed is reached, and it is 0 under the default
  `CANNON_TIMES 0`. The step never fires solo: the issuer needs the
  multiplayer bit.
- `handicap`: reached. With one person at handicap 0, every `sum` and `top`
  is 0, so every slot is 0. No reader consumes it without the multiplayer
  bit.

Behind multiplayer: the `aidiff` arm, `process_drop`, the cannon-time step,
and every `handicap` reader. Behind triggers: `set_leader_difficulty`.
Behind scenarios and the editor: `load_leader_chunk`, bit 26, and the
editor stores. `Leaders::clear` runs at game end.

### 16.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/lobby-words-*.md`); ~~no diff backs it
yet.~~ Since item 4162 the record diff compares `multi_diff` and `handicap`
on every scored dump that prints the record, and the leaders checksum line
holds all three on run 1562's frame 91. `cannon_time` is not printed in
the record, so the checksum line (0 against 0) is its only backing;
`handicap` is quiet everywhere, so only its zero is backed.

Verdicts (confirmed / corrected / open): `multi_diff` 12 / 2 / 0,
`cannon_time` 12 / 0 / 0, `handicap` 14 / 1 / 0. The corrections that
changed a rule:

- the plain-game `multi_diff` is `lobby.difficulty`, not 0;
- `process_drop` keeps a leader human while any other occupied record
  holds the slot, person or computer (it tests no human bit);
- a computer record's `handicap` byte also seeds `sum` under
  `is_cooperative`.

The bit-26 setter and clears are new, beyond both readings.

The writer sets rest on an absolute-address net and a walking-pointer
sweep of the listing for each leaf; both match the readings' union.
Confidence is high on all three: every store, predicate and branch of the
handicap formula was read in the listing, and the offsets come from
`types.txt`.

**Witness, not a pinned diff.** Of 1,194 kept `gamelog-*.txt`, 949 print
`multi_diff` within their first 20 MB. In each, the first eight
`multi_diff` lines carry only 0 and that dump's own first `DIFFICULTY`
value: 297 / 13 / 25 / 121 / 107 / 385 at `DIFFICULTY` 0..5. One dump has
no `DIFFICULTY` line and prints 0 and 5. No dump shows a seated slot at 0
under a non-zero lobby. `CANNON_TIMES 0` appears in 1,193 of the 1,194
dumps, and every leader `handicap` line reads 0.

### 16.6 Not established

None of these changes a plain game's value.

- **Whether the solo setup window offers a non-zero `CANNON_TIMES` or a
  per-player handicap.** Either would change the frozen value, not the
  rule. A capture of a skirmish set that way settles it.
- **`is_cooperative`'s full predicate and `get_team`'s team-8 branch.**
  Both matter only with more than one person. A reading of the two
  functions settles them.
- **`ScenarioEditor::change_difficulty`'s callers** and **the 13 inlined
  `get_diff` copies** were not re-read. Neither changes the plain-game
  value.
- **`LeaderData::walk_data` on a save load** was not re-read for these
  leaves. It belongs to the save track.
- **The dump observation becomes a diff** once the three arms land: the
  `rondata::diff` leaders line then compares them on every capture.

## 17. The spell bits of `tech`

### 17.1 Leaves

- `tech.ptr`, bits `0x275..=0x2ab` (55 spell types), of `tech` at +0x6c0c,
  `BitMask<806>` (head `(806, 101, ptr)`, 101 bytes of bits). The spell bits
  are bytes `0x4e..0x55` of the array. `tech_at_start` (+0x6c80) never holds
  a spell bit.

### 17.2 The rule

**Writer.** One, in a plain game: the spell pass of `Leader::init
(006e3930)`, listing `6e45d9..6e461d`. It runs after the age/tech, building
and unit passes and immediately before `reset_obs_flags` (`6e4621`). It
writes `tech` only.

**Event.** `Leader::init`, once per live slot at setup. Its one plain-game
caller is `Setup::build_game (005ac190)`. `Setup::build_leader (005abc70)`
calls it too, but has no caller and no data reference (dead). The other
callers are out of plain play: `WalkDataGame::walk_data`, `MapScenario::make`,
`Leaders::init`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader` and six sites in `ScenarioRead::load_leader_chunk`.

**Predicate**, for each spell type `t` in `0x275..=0x2ab`, on the player's
`tech` as the earlier passes left it. Set the bit when `type_avail(t, 1)` is
non-zero **and** (`preq[0] != -1` or `spell_flags & 1 == 0`). For a spell,
`type_avail (006e33a0)` is non-zero exactly when both of these pass:

1. `has_preq (006db810)`: every preq slot `q` is `-1`, or `q >= 0` and held.
   A `-2` (`TYPE_ANY_BUILDING`) slot refuses, because `has_tech (006e0c80)`
   answers 1 for `-1` only.
2. `type_eligible (006dbd10)`:
   - the tribe test `tribe_can_type (006d9410)` (the `TypeIndex` overload):
     `(1 << (tribe & 31)) & tribe_mask` non-zero. A slot with tribe `-1`
     tests bit 31, which a 24-bit mask lacks, so it fails;
   - no preq slot `< -1`, so a `-2` is refused here a second time;
   - a preq that is a non-bonus, non-epoch tech whose age exceeds the game's
     `ending_technology` refuses.

   A spell then reaches the default tail and answers 4.

**With the shipped rows** (the 55 loaded `SPELLTYPE` records): every
`tribe_mask` is 24 ones (`16777215`), and no row has `spell_flags` bit 0
(letter `a`), so the second conjunct always holds. `preq` is `[-1,-1,-1]` on
46 rows, which are set. It is `[-2,-1,-1]` on eight, which are never set:
`0x276, 0x278, 0x279, 0x27e, 0x282, 0x284, 0x285, 0x287`. `RADARSPELL`
`0x299` has `[548,-1,-1]` (`INDUSTRIAL_AGE` 0x224), so it is set only when
Industrial Age is held at init and `ending_technology` admits it.

**Step.** Bit `t` of `tech` goes 0 → 1, once. `tech_at_start` is not
touched.

**Reset.** `Leader::init`'s own zeroing (its `BitMask<806>::init`, and the
memset of its `param_2 < 0` arm) precedes the pass. Beyond that, only
construction and teardown clear the bits: `LeaderData::LeaderData
(006d7540)`, `Leader::close (006b7f80)`, and the header's `Leaders::clear`
and `walk_data`. Nothing in a plain game clears the bits or re-runs the pass.
**Reaching Industrial Age by research does not set `RADARSPELL`'s bit.**
`Leader::lose_tech (006d2850)` has only unit and tech arms, so it cannot
clear a spell.

**The other setter, dead in a plain game.** `Leader::gain_tech (006dcb60)`
sets any bit unconditionally, and none of its loops reaches `0x275`. The
only plain-game route that gives it a spell is `Build::finished
(00628490)`'s fall-through when `has_spell` is 0 at the finish (`628687`,
`628548`). That needs a spell that was available when queued and
unavailable when it finished. `BuildData::can_make (0062db10)` refuses to
queue an unavailable type. Nothing in a plain game takes availability away:
the tribe and `ending_technology` are fixed, a `-1` preq is always held, and
an age is lost only through `Leader::set_age (006d25a0)`, whose callers are
the console, scenario functions and `UnitBalance::next`.

**Out of plain play:** the cheats `action_cheat_give_techs (00593180)` and
`action_cheat_zero_techs (00593120)`, `ConsoleWin::run_cmd`,
`disable_type_by_tribe (00a006a0)`, scenario, editor and CtW, and
`walk_data`. Both writer nets (the export, and every displacement or
absolute address into the eight slots' `tech` struct in the listing) found
no other writer.

**Readers.**

- `has_spell (006e0bc0)` reads the bit only for a letter-`a` row. Otherwise
  it returns `type_avail`, so with the shipped data it never reads the bit.
- `has_tech (006e0c80)` reads it raw.
- **`Build::do_queue (0061e410)` at `61e501`** reads it for any queued
  type. When the building `is(MISSILESILO 0x208)`, a missile is inside
  (`inside_down` +0x28 not negative), and the head-of-queue type's `tech`
  bit is set, `do_queue` returns without progressing the queue, unless that
  type is `DISBAND` (0x29a). `DISBAND`'s bit is one of the 46, so the named
  exemption is what lets a loaded silo disband. No other spell names the
  silo in its loaded `from`/`from2`/`where`, so today this changes nothing.
  A port of the bits must keep the exemption.
- The rest are interface (`BuildOut::make_options`, `IFace*`,
  `WallOut::draw`, `ConsoleWin::tell_tech`), `save_leaders_chunk`, CtW,
  `log_data` and the checksum. No other sim decision in a plain game reads a
  spell bit.

### 17.3 In the crate

- **No spell pass.** `TechTree::start` (`crates/sim/src/tech.rs:2235`) ends
  its building/unit loop at `:2290–:2320` and calls `self.reset_obs_flags`
  at `:2321`. The pass goes between the two, the original's order.
- **No slot to hold the bits.** `Kind` (`tech.rs:100`) ends at `Gov`
  (`:117`), so `PlayerTech::tech` (`:720`, `Vec<bool>` over tree entries)
  has no spell entry. Either add the 55 to the tree, or keep a 55-bit side
  mask on `PlayerTech`.
- **No inputs.** `orders::SpellType` (`crates/sim/src/orders/order.rs:95`,
  filled at `crates/rondata/src/load.rs:725–740`) carries `flags` but no
  `preq[0..3]` and no `tribe_mask`. Load both from the craft rows.
- **Emission.** `bits_of` (`crates/sim/src/checksum/leaders.rs:704–710`)
  maps through `tree_type_index` (`:524`) and cannot name `0x275+`. With a
  side mask, append `0x275 + i`. Feed the `tech.ptr` mask only, never
  `tech_at_start`.
- **SEAM.** `queue_disband` (`crates/sim/src/cities/queue_disband.rs:172–174`)
  stands in for `queue_here`, `type_avail` and `has_spell` of `DISBAND`, and
  answers yes. That stays correct, because `DISBAND` is among the 46.
- **Trap: the `-2` refusal lives in two checks.** `has_preq` (which runs
  first) and `type_eligible` each refuse a `-2` slot alone. A port may skip
  one only knowing the other still refuses the eight rows.
- **Checksum.** The port removes `("tech.ptr", 2)` from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1839`). It should also turn
  agreeing the two 101-byte `tech` array ranges (one per walked leader)
  that the pinned `LEADERS` tuple (`:1746`) counts among its four parted
  ranges.

### 17.4 Reached by a plain game

Yes. A skirmish against the computer runs the pass for every live leader in
`Setup::build_game`, before the first frame, and sets 46 bits at an Ancient
start. The `RADARSPELL`-set branch needs an Industrial-or-later start (or
`all_techs`), which a lobby option reaches. The `gain_tech` route is dead in
a plain game. The cheats, the console, `disable_type_by_tribe`, scenarios,
the editor and CtW are out of plain play.

### 17.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/spells-*.md`); no diff backs it yet.

Verdicts: 16 confirmed, 3 corrected, 0 open of 19 claims, plus one added
reader (`Build::do_queue`'s silo test). The corrections:

- The tribe test is the `TypeIndex` overload `006d9410`; it computes the
  same thing for a spell, so the rule did not change.
- `Build::finished`'s fall-through is dead in a plain game. This removes
  `gain_tech` from the plain-game writers.
- `Setup::build_leader` has no caller, and `load_leader_chunk` calls
  `Leader::init` six times. This changes the caller list, not the rule.

Confidence is high. The predicate was read from the listing (deciding lines
`6e45fa`, `6e45ff`, `6e460b`), and its inputs are the loaded records of the
run.

**Witness, not a pinned test.** For run 1562's frame-91 state packet
(`~/ron-golden/run1562/map-14/frame-snapshot.bin`), the predicate, evaluated
on the run's own 55 `SPELLTYPE` records and the leaders' held ages,
predicts every spell bit in both live slots (slot 0 tribe 4, slot 1
tribe 11). 46 are set in each, and the nine clear are the eight `-2` rows
plus `RADARSPELL`. No age `0x220..0x226` is held. Bytes `0x4c..0x57` of
`tech.ptr` read `00 00 a0 bc 4b ff ff fd ff 0f 00 00` in both slots.
`tech_at_start` is all zero, and the uninited slots 2 and 3 carry no bit.
The packet was checked against the dump: its 101 bytes, printed as unpadded
decimals, reproduce the frame-91 `bit_values` strings exactly, for `tech`
and `tech_at_start`, in all four `LEADERDATA` blocks of `gamelog_p.txt`
(lines 816919, 829176, 841435, 853480). That is one run, two tribes, an
Ancient start. No test pins it. A port turns it into the leaders line's own
`tech.ptr` diff on every gate.

### 17.6 Not established

- **The `RADARSPELL`-set branch** (Industrial Age held at init). Unobserved.
  A capture with an Industrial-or-later start, diffed on `tech.ptr`, settles
  it.
- **`ending_technology`'s refusal** of a too-late tech preq. No shipped
  spell row has a tech preq other than `RADARSPELL`'s age, and no capture
  limits the ending age. A capture with a restricted ending age settles it.
- **A live slot with tribe `-1`, and a letter-`a` row.** Neither occurs with
  the shipped data. They are read from the listing only.
- **`BuildTypeData::queue_here` for the silo** was not read. "No other spell
  can be queued at a silo" rests on the loaded `from`/`from2`/`where`
  columns. Reading `queue_here` settles it.
- **A malformed lockstep queue order** that bypasses `can_make` and reaches
  `Build::finished`'s fall-through was not traced. Tracing the queue order's
  validation from receipt to `queue_up` settles it.

## 18. The score and its eleven parts

### 18.1 Leaves

- `score` (+0x18), i32: the sum.
- `score_explored` (+0x1c), i32: always 0.
- `score_territory` (+0x20), i32.
- `score_units` (+0x24), i32.
- `score_units_2` (+0x28), i32: armed units; not summed, no reader.
- `score_buildings` (+0x2c), i32.
- `score_economy` (+0x30), i32.
- `score_pop` (+0x34), i32: always 0.
- `score_unit_upgrades` (+0x38), i32.
- `score_research` (+0x3c), i32.
- `score_wonders` (+0x40), i32.
- `score_combat` (+0x44), i32: an accumulator, never recounted.

### 18.2 The rule

**Reset**: `reset_score (006e37f0)` zeroes all twelve, called only from
`Leader::init (006e3930)` and `Setup::build_game (005ac190)` (default 0).

**When.** `Leaders::strategy_all` runs, for each of the 8 slots with
`leader_flags & 3 == 3`, **humans included**: `check_explore`,
`plan_strategy`, `compute_score(who, forced=0)`, `diplomacy`. `do_frame`
skips it when `Game+0x821 & 8`. `compute_score` recomputes when

    frame == 0 || forced || (Game+0x820 & 0x40) || (who + frame) % 10 == 0

(`frame` before `do_frame` increments it, signed remainder), else changes
nothing. Bit 6 of `Game+0x820` has four setters: `check_victory`'s game-over
latch, `RecordGame::read_package`, `TurnControl::toggle_pause`,
`MPDropWin::on_button_clicked`. `defeat_all`'s forced `compute_score(1)` runs
only after Armageddon (`process_victory` calls it only when
`get_armageddon() <= game->armageddon`), so it only zeroes.

**Dirty bits** of `leader_flags`, cleared only by `compute_score`: bit 23
(`0x800000`) set by `Unit::init (00612100)` (owner `< 8`, signed) and
`Unit::close (0060ee50)` (owner byte `< 8`); bit 24 (`0x1000000`) set by
`Leader::gain_tech (006dcb60)` at entry. The whole-word stores in
`Game::init_rules_and_teams (00589bb0)` and `Game::init_tribes (0058a600)`
are read-modify-write and keep both.

**A recompute, in order**: `score_explored = 0`; units if bit 23 or 24, then
clear 23; buildings and wonders; economy; `score_pop = 0`; upgrades then
research if bit 24, then clear 24; territory. Then, if
`game->armageddon (G+0x6e0) >= get_armageddon()`, zero every leaf **except
`score_units_2`** and return; else

    score = combat + wonders + research + upgrades + economy + buildings
          + units + territory + pop + explored        (i32 wrapping)

Each part also zeroes and returns early after Armageddon. `get_armageddon = ARMAGEDDON (C+0xd14) + ARMAGEDDON_PER_TEAM
(C+0xd1c)·num_sides (G+0x6a8) + ARMAGEDDON_PER_NATION (C+0xd18)·num_nations
(G+0x6a0)`; ×3 if `starting_resources (G+0x2d)` is 7, ×2 if 5, 6, 11 or 12,
floor 100 if 8.

**Values.** Divides truncate toward zero; multiplies wrap at i32.
`value(t, p)` is `get_score_value` (vslot `+0x7c`, one body `006673b0`),
`S = Σcosts[0..6]`:
- `p == 0`: `factor × S` (UCF for units, TCF for techs). The militia family
  is priced from type 0x32, still with UCF; range tests read the type's own
  index (`TypeData+0x4`).
- `p == 1`: `((RP × (UCF × S)) / 256 × research_premium_cost(t)) / 256`, in
  that order; no militia substitution.

`kill(t, o)` (`get_kill_value`: unit `0061d3d0`, building `00639790`, other
`00663a00`) is `value(t,0) + (sc0+sc1)·n`, times `BUILD_SUPPORT_FACTOR`
(constants `+0x37c`) on the support term for a building. `sc0`, `sc1` are the
type's `support_cost`s; `n` is owner `o`'s count of `t` **plus its
`num_queued[t]`** (the kill is priced as the owner's next one); `o < 0` gives
`value(t,0)`.

- **Territory**, every recompute:
  `territory (+0x9d8) × 1000 / land_size (WorldData+0x78)` (faults on 0).
- **Units** (`Leader::compute_unit_score (006bc500)`), when bit 23 or 24: both
  to 0; for `t` in `0x32..0x192` with `n = num_units[t−0x32] + num_queued[t] > 0`,

      u = (value(t,0)·n + ((n−1)·(sc0+sc1)·n) / 2) / 5

  to `score_units`, and to `score_units_2` too when `attack (+0x1e8) != 0`.
  No BSF.
- **Buildings, wonders** (`Leader::compute_build_score (006bc3f0)`), every
  recompute: both to 0; for `t` in `0x19e..0x21f` with
  `n = num_buildings[t−0x19e] + num_queued[t] > 0` (queued = inactive sites),

      b = value(t,0)·n + ((n−1)·((sc0+sc1)·BSF)·n) / 2

  `b / 3` to `score_wonders` when the own index is in `0x20e..0x21f`, else
  `b / 5` to `score_buildings`.
- **Economy**, every recompute: to 0; for `g in 0..6` with
  `type_avail(g, 1) != 0`, add
  `clamp((bucket[g] − starting[g]) / 20, 0, 100) + min(income[g] / 80, 200)`
  (decrypted `^ 0x8221`, `^ 0x90236`; `starting` is `Game+0x600`; income has
  **no floor**). Then `+ 25 × popcount(rare)` (`+0x6d98`). Side effect:
  `BitMask<44>::count (0047cd40)` writes `rare.flags` (`+0x6da0`) to 1 when
  the count is 0, else 0, and returns 0 at once when it is already 1.
- **Unit upgrades**, when bit 24: to 0; for `t` in `0x32..0x192` add
  `value(t,1) / 5` when (`has_tech(t)` or `researching(t, −1, 0, 1)`, an exact
  match in a queue of my active building), `tech_at_start[t]` (`+0x6c8c`) is
  clear, and `unit_flags (+0x2b4) & 0x80` is clear.
- **Research**, when bit 24: to 0; for `t` in `0x220..0x275` (ages included)
  held or queued (`researching(t, −1, 0, 0)`, exact for techs) and not
  `tech_at_start[t]`, add `(TCF × S) / 5`.
- **Explored, pop**: 0. `compute_explore_score (006bc5e0)`,
  `compute_pop_score (006bc190)` and `compute_combat_score (006bc250)` are
  dead (no call, no pointer).

**`score_combat`**: each event moves it by `v = kill(t, o) / 5`, `o` the
victim's owner. Its only stores: `Object::do_damage (0064a480)`,
`Object::take_damage (00652020)`, `SpellType::cast_sabotage (006720e0)`,
`SpellType::cast_counterintel (00673740)`, the Armageddon zero.
- **Unit killed** (`take_damage`). A building victim takes `+0x81c += 1` and
  leaves. Else it must be a captain (vslot `+0xe8`) on its last figure
  (`curr_uber_size(0) == 1`), not a decoy (`unit_masks & 1`): victim owner
  `+0x810 += 1`. Then **only if the killer id `param_7 >= 0`**: `+0xa58`,
  `+0xa5a`, `+0x80c` move; `unit_flags & 0x10` redirects the victim to
  `(+0x3e, +0x28)`; a negative id or a false vslot `+0x18` scores nothing;
  `sc[K] = is_ally(K, o) ? max(sc[K] − v, 0) : sc[K] + v`. No killer-kind or
  splash test. It runs before the victim's `die`: `n` counts the victim.
- **Building razed** (`do_damage`): type captured before `take_damage`; when
  that returns non-zero and the target is a building, `+0x818 += 1` and the
  same ally rule, credited to the attacker's owner. `die` has already run
  `Build::close (00628980)` → `Wall::decrement_stats (00642da0)`: `n`
  **excludes** the razed building (or site).
- **Sabotage**, unit arm (captain, last figure, not decoy) or building arm
  (`take_damage` non-zero, `param_7 = −1`, so no double count):
  `sc[caster] += v`, no ally test, no floor.
- **Counterintelligence**, on a target climbed to its captain whose owner is
  neither the caster nor allied both ways, single figure, not decoy:
  `sc[caster] += v`.

**Readers.** `Army::find_target (006f69b0)` takes an arm when
`((score[c] << 2) / 3) < score[army owner]` (shift wraps). `process_victory`
with `victory == 5` defeats the live leader with the strictly lowest score; a
tie at the minimum defeats nobody. `Leader::diplomacy` counts live leaders
scoring above me and finds the top scorer.

### 18.3 In the crate

- **Step site**: `Sim::strategy_all` (`crates/sim/src/ai/ai_drive.rs:73`,
  comment `:87`). **Trap**: the human arm `continue`s at `:82`–`:85`, ahead
  of the site; the score must run before it.
- **Inputs**: `Muster::by_type`/`queued_by_type` (`units/unit.rs:838`,
  `:840`); `Holdings::territory`/`land_size` (`economy.rs:554`–`555`);
  `Ledger::bucket`/`income`, `Holdings::starting`/`rare`
  (`economy.rs:679`–`699`). Building `num_queued` as `num_sites_of` is the
  crate's own claim.
- **Dirty-bit hooks**: `add_unit` (`units/store.rs:48`), `close_unit`
  (`combat/fight/delivery.rs:1245`), `tech/gain.rs:189`.
- **Kill value**: `economy::cost::kill_value_by` (`economy/cost.rs:1258`)
  ports the building form, swept in
  `crates/sim/tests/sweep/build_type_data_get_kill_value.rs`, uncalled;
  `factor = 1` with the unit arrays gives the unit form.
- **Combat**: unit arm in `take_damage_typed` (`delivery.rs:537`–`543`, by
  `unit_plunder`); building arm in `do_damage`'s `killed` arm
  (`delivery.rs:215`), which has the attacker, not `damage_building` (`:957`,
  only an optional `by`). The predicate is wider than plunder's: any
  attacker, splash included, an ally subtracts. Sabotage SEAMs
  `delivery.rs:804`, `:882`; counterintelligence `Sim::cast_counterintel`
  (`spells/cast.rs:1031`), SEAM `:1028`.
- **Readers seamed**: `leaders/diplomacy.rs:155`, `:385`;
  `ai/army/target_score.rs:457`.
- **Trap, `is_ally` past the table.** `LeaderData::is_ally (006edb50)`
  indexes `diplos int[8]`; gaia units are owner 8 (and 9; `gaia.rs:93`). A
  gaia victim with a killer object calls `is_ally(K, 8)`, reading
  `diplos[8]` = `treaties[0]` (`+0x94`) and `leaders[8].diplos[K]`. The
  crate's `Sim::is_ally` (`leaders/relations.rs:99`) assumes no such caller;
  do not answer "not allied" from its guard (18.6).
- **Checksum**: slots `checksum/leaders.rs:41`–`52`; all twelve are
  `(unheld)` in `LEADERS_HELD_OUT`, count 2 each
  (`crates/rondata/src/diff/checksum_lines.rs:1823`–`1834`). Each port removes
  its entry; explored and pop need only the hold.

### 18.4 Reached by a plain game

A skirmish against the computer reaches `compute_score` for every live slot,
every part, both combat arms, and the `victory == 5`, `find_target` and
`diplomacy` readers. Bit 6 comes from the game-over latch and from pause;
`read_package` needs a recorded-game playback, `MPDropWin` multiplayer. The
Armageddon arm and `defeat_all` need the Armageddon count reached. The spells
need a spy or commando cast; whether the computer casts them is open.

### 18.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/score-*.md`); no diff backs it yet. Every
integer formula, both `get_kill_value`s and `get_score_value` were checked
against the listing instruction by instruction; none was wrong. Confidence
high.

Confirmed / corrected / open: shared 12/2/1; `score` 6/2/0; explored 3/0/0;
territory 4/0/1; units pair 9/0/1; buildings pair 6/0/0; economy 6/0/2; pop
2/0/0; upgrades 4/0/1; research 2/0/1; combat 10/1/1.

Corrections that changed a rule: `defeat_all` follows Armageddon only (no
end-of-game sweep); `find_target`'s clause is `4·score[c]/3 < score[mine]`,
not the reverse; the razed building is out of `n` when priced (settled). The
crate has `kill_value_by` and `cast_counterintel`. New: the gaia `is_ally`
read and the `rare.flags` write. No kept dump was consulted;
`LeaderData::log_data (006e5110)` prints `score` for a later value diff.

### 18.6 Not established

- **Gaia ally read**: whether `treaties[0]` and `leaders[8].diplos[K]` are
  ever both 2. If not, every gaia kill with a killer object adds
  `kill(t, 8)/5` on slot 8's counts. A `LEADERDATA` dump across a hunt, or a
  frame-snapshot read of the two words, settles it.
- **`rare.flags`**: whether the leaders checksum walks `BitMask::flags`
  (`+0x6da0`); read `checksum/leaders.rs`'s ranges.
- **Economy**: whether the crate's `income` (documented as the interface's
  net rate; `rate` is sixteenths) is the original's word and moment;
  `type_avail` for goods (`has_preq`, `type_eligible` unread).
- **Territory**: whether the crate's `territory` is current at
  `strategy_all`.
- **Units**: whether every removal goes through `Unit::close`; the crate's
  `queued_by_type` for units.
- **Upgrades, research**: `has_tech` past its head; `researching`'s vslot
  `+0x4c` "active" test.
- **Combat**: the `unit_flags & 0x10` redirect's meaning; whether the
  computer casts sabotage or counterintelligence.
- **Rest**: `get_team_score` past its head; the three non-latch bit-6
  setters' contexts; `LeaderData::walk_data (006d6750)` on load (off the
  frame path).

## 19. The last building and unit finished

### Leaves

- `last_building_finished`: +0x6070, `int[129]`, indexed by building
  `TypeIndex − 0x19e` (`BASE_BUILDTYPES`).
- `last_unit_finished`: +0x6274, `int[352]`, indexed by unit
  `TypeIndex − 0x32` (`BASE_UNITTYPES`).

The executable folds each index bias into the displacement, so its stores
address +0x59f8 and +0x61ac (absolute `0xe3fd88` and `0xe4053c`, slot
stride 0x1bbb dwords). A search for the leaf offsets alone misses every
writer.

### The rule

**Shared by both leaves.**

- **Init**: `Leader::init (006e3930)` fills both arrays with −1 (0x81 and
  0x160 cells) for a seated leader. An unseated slot takes the bulk memset,
  so both are 0 there, and `LeaderData::walk_data (006d6750)` never walks
  them.
- **Step**: an overwrite with an object number `o`, never an increment.
- **Reset**: `Leader::init`, and the trigger reader's read-and-clear (below).
  Nothing else clears a cell: no death, capture or delete path.
- **Writer nets are complete**: the two stores below, `Leader::init`, the
  trigger reader, and the save/load restore. Every other hit in the range is
  another class or `num_queued` (+0x5a22, which ends at +0x606e).
- **Reader**: `ScenarioFuncSet::get_last_type_built (009e9910)` alone, plus
  the checksum and save/load. No sim decision reads either leaf. The reader
  takes a 1-based player and rejects `p − 1 > 7` as an unsigned test (so it
  rejects player 0 too). It also rejects an unseated leader. It picks the
  leaf by type: buildings are 0x19e..0x21e, units are `TypeData::is_unit_type
  (004707e0)`, `0x31 < t < 0x19e`. It returns the cell and sets it to −1.
  The unit arm takes in the 12 gaia types 0x192..0x19d, which index cells
  352..363, past the array into +0x67f4..+0x6823 (`ages_queued` onward). No
  writer reaches those indices, so the overrun lives in the trigger layer
  only. `get_last_unit_built (009e9a70)` reads a `ScenarioData` scalar, not
  this leaf.

**`last_building_finished`.**

- **Writer**: `Build::activate (00623e20)` with arguments
  `(captured, _, counted)`. If `captured == 0 && counted != 0`, it sets
  `last_building_finished[who][ty − 0x19e] = o`. `o` is the building's
  `SubObjectData::o`, a sign-extended `short` at +0xa. The same `o` also goes
  to `ScenarioData::last_build_built_o[who]`, which is not this leaf.
- **The type is read after the Chinese Large City branch.** The branch needs
  `flags & 0x20`, tribe bonus 0xe, `captured == 0`, `chinese_large_cities`,
  and a type other than 0x213. When it fires it calls `set_type(0x19f)`,
  and the store re-reads the type afterwards, so such a city records at
  index 1.
- **Dispatch**: vtable slot +0x1a8 is `Build::activate` only in
  `Build::vftable (00b42174)`. `Wall`, `BuildOut` and `BuildData` carry
  `Wall::activate (0063e4b0)`, so walls (a separate object kind) never write.
- **Callers that write** all pass `(0, 1, 1)`:
  - `Wall::do_construct (006434d0)`, on completion: accumulated work +0x48
    ≥ `BuildData::construct_time` (vtable slot +0x18c), as an **unsigned**
    compare;
  - `Console::hurry_action (007c3810)`;
  - `ScenarioFuncSet::create_building (009f4d30)`.
- **Callers that do not write**:
  - (0,0,0): `Setup::build_cities (005ab910)`, `ConquestGame::build_cities`,
    `ScenarioEditor::add_build` and `::activate_selected`,
    `ScenarioRead::load_city_chunk` and `::load_build_chunk`.
  - (0,1,0): `Build::check_capture (006276a0)`, `SpellType::cast_bribe
    (00672d00)`, `Leader::defeat_by`, `City::find_buildings`,
    `ConsoleWin::run_cmd`, `UnitBalance::next`.
  - (0, `param_2`, 0): `Leader::produce_building (006e1400)`.
  - (1,1,0): `Cities::capture_city (00733380)`,
    `ScenarioFuncSet::switch_teams`.

**`last_unit_finished`.**

- **Writer**: `Build::train (0062f9b0)`, given a trainer and a type `t`,
  calls `Objects::init_unit (0065e0c0)`. If the return is ≥ 0 (signed), it
  sets `last_unit_finished[trainer.who][t − 0x32] = o`, where `o` is that
  return. The same value also goes to `ScenarioData::last_unit_built_o[who]`
  (`0xcc1930`). The index is `train`'s argument `t`, not a type read back
  from the unit. Nothing inside `train` exempts a kind: every successful
  train writes.
- **A squad** records the **captain's `o`**: `init_unit` makes `UBER_SIZE`
  (+0x308) figures and returns the last figure's `get_captain()`. If
  `find_free` fails part-way, `init_unit` returns a negative code, and
  nothing is written although the earlier figures exist.
- **Callers**: 27 direct calls and no vtable entry. `Build::activate` makes
  15, `Build::process (0061edf0)` 5, `Build::finished (00628490)` 2, and
  one each come from `Leader::gain_tech (006dcb60)`, `City::assimilate
  (00738e90)`, `Setup::place_unit (005abca0)`, `ConsoleWin::run_cmd` and
  `UnitBalance::next`.
- **`Setup::place_unit` trains only as a fallback.** Each placement try that
  finds a free tile calls `init_unit` directly, which does not write.
  `Build::train` runs only after the try budget is spent (60 with a starting
  town, 500 for a nomad), and only when `param_2 ≥ 0`. In that case
  `param_2` also moves the search centre onto the building. When the budget
  is spent with `param_2 < 0`, it calls `init_unit` directly instead. So a
  normal start unit is never recorded. Neither are gaia units or units a
  trigger makes.

### In the crate

- **Building hook**: `activate_head_steps`
  (`crates/sim/src/cities/activate.rs:53`) tests the predicate at `:74` and
  calls the empty `SimHead::record_last_built` (`:139`; `:349` is the test
  view). It is reached from `Sim::activate`
  (`crates/sim/src/cities/city.rs:2420`, head at `:2454`). The head already
  orders the Chinese branch before the type read (`activate.rs:65–76`).
  - **Index**: `buildings[b].ty`, read after the head's `set_type`
    (`activate.rs:118–131`). The crate's building record is
    `TypeIndex − 0x19e` (`crates/rondata/src/load.rs:220`).
  - **Value**: `buildings[b].index`, the crate's `SubObjectData::o` in
    `[2000, 3000)` (`crates/sim/src/cities/building.rs:14–17`), **not** the
    vector position `b`.
- **Trap, the frame-0 `counted` flag.** `Leader::produce_building` passes
  counted 0. The crate's frame-0 arm calls `self.activate(o, false, true)`
  (`crates/sim/src/ai/ai_place.rs:1035`), which is counted = true. Fill the
  hook as-is and every building `Setup` produces at frame 0 would be
  recorded, where the witness below shows −1. The same flag already gates
  `claim_building_high`'s `free_units_for`
  (`crates/sim/src/cities/nations.rs:238`). So the fix needs its own check
  and a re-check of what that path pays, not a bare flip.
- **Unit hook**: `Sim::build_train` (`crates/sim/src/cities/queue.rs:1693`),
  right after `init_unit` (`crates/sim/src/units/store.rs:504`), with
  `table[who][ty] = units[unit].index as i32`.
  - **Index**: the crate's unit record `ty` already equals
    `TypeIndex − 0x32` (`load.rs:1468`, `:212–216`). `.tree` is a tech-tree
    id, so subtracting 50 from it would be wrong.
  - **Value**: `units[unit].index` is `SubObjectData::o`
    (`crates/sim/src/units/unit.rs:40–47`). `init_unit` returns the squad
    head, and its `index` is what the members take as their captain, which
    matches the original.
  - **Callers that inherit the write**: `queue.rs:1612`, `:1675`,
    `cities/activate.rs:391`, `cities/city_views.rs:591`, and
    `cities/nations.rs:432, 524, 583, 655, 686`.
- **SEAM**: `Sim::place_unit` stops at `Err(PlaceRefused::Train)`
  (`crates/sim/src/units/starting.rs:26–27, 68`). When the fallback is
  built, it must go through `build_train` so the write follows.
- **No failure path**: the crate's `init_unit` returns `usize` and cannot
  fail, so "negative return → no write" has no counterpart until
  object-table exhaustion is modelled.
- **Checksum**: both leaves are `(unheld)` in `LEADERS_HELD_OUT`, count 2
  each (`crates/rondata/src/diff/checksum_lines.rs:1805, 1808`; pile
  `out:finish` in the 4124 journal). Each port removes its entry.

### Reached by a plain game

- **`last_building_finished`**: every construction completed through
  `Wall::do_construct`. These do not record: the start city
  (`Setup::build_cities`, counted 0), frame-0 produced buildings, captures
  and bribes. Off the plain path:
  - `Console::hurry_action` needs `cheat_keys != 0` and semaphore bit 4
    clear (key case 0x45 of `Console::exec_keymap`), or comes from
    `ConsoleWin::run_cmd`;
  - `create_building` is a trigger;
  - `ConquestGame::build_cities` is Conquer the World (and does not write).
- **`last_unit_finished`**: every queued unit (`Build::finished`) and every
  free unit (`Build::activate`, `Build::process`). The Americans' bombers
  (`Leader::gain_tech`) and assimilation's hero (`City::assimilate`) are
  skirmish-reachable too. Start units record only on the placement
  fallback. The console, `UnitBalance` and triggers are off the plain path.
- **The only reader** is a trigger, so in a plain game the leaves matter to
  the checksum and the save alone.

### How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/last-finished-*.md`); no diff backs it
yet.

The verdicts were 30 confirmed, 6 corrected and 0 open (common ground 5/0/0,
buildings 14/3/0, units 11/3/0). A squad's value, which both readings had
left open, was settled as the captain's `o`. The store, predicate, index,
init, dispatch and caller sets are each settled at a listing address.
Confidence is high.

The corrections that changed a rule:
- `Setup::place_unit` records only on its fallback train, not for every
  start unit placed at a building;
- the crate's unit cell is `[ty]`, not `.tree − 50`;
- the stored values are `Building::index` and `Unit::index`, not vector
  positions;
- the crate's frame-0 activate passes counted = true where the original
  passes 0.

The building caller list was corrected to the union of both readings; no
caller in it writes.

**Witness, not a pinned diff**: the 4124 journal's checksum columns
(`docs/journal/2026-11-11-item-4124.md:159–160`). Every cell of both leaves
is −1 at the start, including the citizen's unit cell 0, as the
fallback-only rule predicts. Later, building cells hold `o`s 2007..2015 and
the unit cell 0 holds 21, then 14.

### Not established

- **The frame-0 `counted` flag** (`ai_place.rs:1035`) needs a re-gate once
  `record_last_built` is filled, together with what `claim_building_high`
  pays on that path. A port settles it against the witness (−1 at the start).
- **Whether the crate's `Building::index` matches the original's building
  `o` allocation** (`Objects::find_free (0065ad60)` over `[2000, 3000)`) is
  the crate's existing contract and was not re-read here. A pinned diff of
  `last_building_finished` would settle it.
- **The no-write-on-failure branch of `Build::train`** cannot be exercised
  until the crate models object-table exhaustion.
- **How a port treats the trigger reader's gaia overrun** into
  +0x67f4..+0x6823 is a design choice for when triggers are built (they are
  cut from v1). Nothing in the original is open.
- **Both leaves stay unpinned** until the crate holds them and the
  `out:finish` rows on a kept capture diff clean.

## 20. `high_buildings`, `territory_high` and `reg_terr`

### 20.1 Leaves

- `territory_high`: +0x9dc, `int`. Territory's high-water mark, in tiles.
- `reg_terr`: +0x145e, `ushort[64]`. Tiles held per land region, indexed by
  the original's region number 0..63 with no bias.
- `high_buildings`: +0x5660, `ushort[129]`. A building type's high-water
  count, indexed by `basic_type(T) − 0x19e` (the root of T's `from` chain).
  The executable folds the bias into the displacement, so every access is
  spelled at +0x5324 (absolute `0xe3f6b4`). The export's
  `high_buildings[x + 0x4f]` hits are `num_units` (+0x5762), not this leaf.

All three are zeroed by both bulk writers (`Leader::init (006e3930)`'s
tribe<0 memset and `Leaders::clear (006ed5d0)`). The tribe<0 arm runs for
every slot at game creation, because `Leaders::init (006ed850)` calls
`Leader::init(i, −1, i)`. `Game::run (00584590)` reaches it before
`Game::init`, which runs setup's territory recompute. `Leaders::clear`
covers all ten slots and runs from `Game::close (00594b80)` and
`Game::run_oos_recovery (005869e0)`.

### 20.2 The rule

**`reg_terr`.** Every writer is inside the border pass,
`World::compute_reg_territory (006b0bb0)`, called as `(r, reset)`:

1. If `reset`, set region r's cursor (`Region +0x2c`) to 0.
2. **Zero block.** If the cursor is 0, then for every slot 0..7 with
   `leader_flags & 1`, set `reg_terr[s][r] = 0`. The same block zeroes
   `reg_known_rares[s][r]` (+0x4d4) and the slot's live cities' `bordering`
   (+0x65). Slots without the live bit keep their last counts, and the
   checksum walks them.
3. **Early return** when the cursor ≥ the region's size, or when the
   frame's tile budget (`GameDaemon +0x24`) is above 0xff. The test is
   signed, so −1 means unlimited. It runs **after** the zero block, so a
   region the budget denies is re-zeroed every frame until its first tile.
4. **Count.** For each tile the pass assigns, if the winner is ≥ 0, then
   `reg_terr[winner][r] += 1`, wrapping as a u16. Two winners count for
   nobody: −1 (no claim) and −2. A tile is −2 when the winning city's
   `CityData +0x5f` (`race`, a `char`) is not the slot whose city list held
   that city.

`World::compute_all_territory (006b5700)`, setup's wholesale recount,
zeroes all 64 entries for every live slot 0..7. It then sets the budget to
−1 and calls `compute_reg_territory(r, 1)` for every region r with
`Region.size (+0x14) != 0`.

`Leader::init`'s ordinary path also zeroes all 64 entries.

The reader that decides is `BuildTypeData::blocked_location (006375b0)`.
City spacing relaxes by `CITY_SPACING − RELAX_CITY_SPACING` when
`size·9/10 <= reg_terr[who][r]`. The two `territory` folds below also read
it.

**`territory_high`.** It has two writers, both the same fold:

- `World::compute_all_territory (006b5700)`, at setup. Its callers are
  `Setup::build_cities` and `Setup::build_game`, which calls it twice.
- `GameDaemon::check_borders (00732060)`, every frame through
  `GameDaemon::process_all`. It folds only when both hold: some
  `compute_reg_territory` call this frame returned bit 2, and no sized
  region 0..63 still has its cursor below its size.

The predicate is slots 0..7 with `leader_flags & 1`. The step, signed:

```
territory      = Σ reg_terr[r]   over r in 0..63 with Region.size != 0
territory_high = max(territory_high, territory)
```

It is never lowered between the resets. There is no sim reader. The
readers are `AchieveChartPane::set_chart_data (007f2c40)`, a UI float, and
`LeaderData::log_data`.

**`high_buildings`.** The only writer is `Build::activate (00623e20)`,
called as `(p1, p2, p3)`. It runs after the building is counted: the base
`Wall::activate` runs `Wall::increment_stats` first.

- `n = Σ num_buildings[U]`, over T and every record on T's `to` (+0x2e0)
  chain. This is `num_buildings[T] + LeaderData::get_buildings
  (006e0680)(to(T))`, recursive while `to >= 0`.
- **Predicate:** `p1 == 0` and `n > high_buildings[root]`, as a signed
  compare. Then `high_buildings[root] = n as u16`.
- Inside the raise branch:
  - `T == 0x21a` sets `leader_flags |= 0x10000000`;
  - `p3 != 0` pays the nation's free units, subject to the semaphore gates
    `0x821 & 0x10` and `0x822 & 2` and the scenario byte `0xcb195b`.
- The reset is all 0x81 entries, by `Leader::init`'s ordinary-path loop
  (which zeroes `num_buildings` alongside) and both bulk writers.
  `Wall::decrement_stats` leaves the mark alone, so it is never lowered.

The callers and their `(p1, p2, p3)`, from all 16 listing sites:

| caller | p1, p2, p3 |
|---|---|
| `Setup::build_cities` | 0,0,0 |
| `Build::check_capture` | 0,1,0 |
| `Wall::do_construct` | 0,1,1 |
| `SpellType::cast_bribe` | 0,1,0 |
| `Leader::defeat_by` | 0,1,0 |
| `Leader::produce_building (006e1400)` | 0,p2,0 |
| `Cities::capture_city` (two sites) | 1,1,0 |
| `City::find_buildings` | 0,1,0 |
| `Console::hurry_action` | 0,1,1 |
| `ScenarioEditor::add_build`, `activate_selected` | 0,0,0 |
| `ScenarioRead::load_build_chunk` (two sites) | 0,0,0 |
| `ScenarioFuncSet::switch_teams` | 1,1,0 |
| `ScenarioFuncSet::create_building` | 0,1,1 |

`ConquestGame::build_cities (00796140)`, `ScenarioRead::load_city_chunk`,
`UnitBalance::next` and `ConsoleWin::run_cmd` also call it. So p1 = 1 only
for `capture_city` and `switch_teams`, and every other caller can raise the
mark.

The readers are `TypeData::get_cost (00664090)` and
`BuildData::construct_time (0062d5c0)`. Both run the Iroquois quick-Senate
test, `high_buildings[T − 0x19e] == 0`, indexed by T itself.

### 20.3 In the crate

- **`territory_high`: not held.** The fold belongs in
  `Sim::update_territory_holdings` (`crates/sim/src/territory/upkeep.rs:129`).
  That function is called from `start_border_pass`
  (`territory/border_pass.rs:56`, pre-play), `settle_borders` (`:170`) and
  the pass-complete tail of `advance_border_pass` (`:248`), and those are
  the two original sites. One `Sim` is one game, so zeroing at creation is
  the reset. A crate that reuses a `Sim` across games must zero the field at
  game start. The crate's `holdings.territory` counts cells over every land
  region (per its doc comment). The original sums `reg_terr` over sized
  regions 0..63, so the checksum agrees only if the two counts agree.
- **`reg_terr`: not stored.** `Sim::reg_terr` (`cities/place.rs:344`)
  recounts on demand, so it parts from the original mid-pass. The crate
  already reads the spacing rule at `cities/location.rs:305`.
  `zero_region_rares` (`border_pass.rs:125`) notes that the +0x145e write
  "has no field in this crate". A store goes in three places:
  - the cursor-0 arm of `advance_border_pass` (`:184`, arm `:205`);
  - its per-cell loop (`:222`, `who.player()`);
  - the wholesale recount in `wholesale_bordering` (`:136`),
    `settle_borders` (`:166`) and `start_border_pass` (`:48`).

  `Sim::reg_terr` should then read the store. **Trap:** `zero_region_rares`
  zeroes for every `ai` leader without a liveness test. A `reg_terr` zero
  placed there must be gated on `leader_flags & 1`, or a dead slot's row
  parts.
- **`high_buildings`: held**, as `Cities::building_high`
  (`crates/sim/src/cities.rs:51`). It is written by
  `Sim::claim_building_high` (`cities/nations.rs:219`) from `Sim::activate`
  (`cities/city.rs:2420`, call `:2578`). The guard at `nations.rs:234`,
  `building_high[w][root] >= n || captured`, is the original's predicate,
  with `captured` as p1. Only the checksum emitter is missing. It maps
  `building_high[who][rec]` to the original's `type − 0x19e`, zero-padded
  to 129, and the `"num_buildings"` arm is the model. The crate callers
  checked against the table:
  - `capture_view.rs:236` passes `(true, false)`, `city_views.rs:276`
    passes `(false, false)` and `city.rs:680` passes `(false, false)`. All
    three match.
  - `Wall::do_construct` routes through `cities/construct.rs:49`/`:108` as
    `activate(0, 1, 1)`. Not traced further.
  - **Unported:** `Build::check_capture`'s non-city swap branch.
    `SimCapture` answers `swap_team` with −1, and its `activate_new` is
    empty (`cities/capture.rs:427` ff., `:661`). The (0,1,0) raise there
    never happens. Only the `capture_city` arm is live.
  - **Unported:** a building arm of `cast_bribe`. `spells/bribe.rs` ports
    the unit arm only.
  - **Trap:** `ai/ai_place.rs:1035` passes `activate(o, false, true)` where
    `Leader::produce_building` pushes p3 = 0. The mark is unaffected,
    because p1 = 0 on both sides. But `counted` gates `free_units_for`
    (`nations.rs:238`), so the crate can pay free units on a frame-0
    building where the original pays none. Book it against the units
    score, not this leaf (§19 names the same flag).
- **Checksum.** The rows are at `crates/sim/src/checksum/leaders.rs:236`
  (`territory_high`), `:294` (`reg_terr`) and `:297` (`high_buildings`).
  Each leaf is `(unheld)` in `LEADERS_HELD_OUT` with count 2
  (`crates/rondata/src/diff/checksum_lines.rs:1794, 1820, 1840`), and each
  port removes its entry.

### 20.4 Reached by a plain game

All three leaves are reached.

- `reg_terr` and `territory_high` move at setup and on every border pass.
- `high_buildings` moves on starting buildings (`Setup::build_cities`),
  every finished construction (`Wall::do_construct`), frame-0 production,
  `City::find_buildings`, `Leader::defeat_by`, and buildings changing
  hands.
- Not yet checked: whether a plain game reaches `check_capture`'s non-city
  swap or a building bribe.
- Off the plain path: `Console::hurry_action` and `ConsoleWin::run_cmd`
  (console), the `ScenarioFuncSet` calls (triggers), the editor and
  scenario loaders, `UnitBalance::next`, and `ConquestGame::build_cities`
  (Conquer the World).

### 20.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/high-buildings-*.md`); no diff backs it
yet.

| leaf | confirmed | corrected | open |
|---|---|---|---|
| `territory_high` | 9 | 1 | 0 |
| `reg_terr` | 12 | 2 | 1 |
| `high_buildings` | 12 | 0 | 2 |

The writers, predicates, steps and resets are settled at listing
addresses. Confidence is high on all three leaves.

The corrections that changed a rule:

- **`territory_high` is reset per game.** The claim that nothing resets it
  across games in one process is wrong. The tribe<0 memset (`006e398c`, a
  `_memset` thunk per `rise_z.map`) and `Leaders::clear` (`006ed5e0`) both
  zero `[+0, +0x6929)`. Both readings' nets missed `Leaders::clear`,
  because its size is 0x6929, not the struct size.
- **The −2 arm tests `CityData::race` (+0x5f)**, not an owner byte. The
  crate's `Owner::Ambiguous` doc (`crates/sim/src/world.rs:172`) has the
  same naming gap.
- **`compute_all_territory` clears live slots only**, with eight 16-byte
  stores (64 ushorts).

No kept-dump value is cited as a witness.

### 20.6 Not established

- **What `CityData::race` holds after a capture.** It decides when a tile
  counts for nobody (−2). A capture with `reg_terr` diffed, or a
  reading of `race`'s writers, settles it. Whether the crate's recount
  already parts in a capture is booked as `docs/audit/parked-archive.md` 568.
- **`Wall::increment_stats`' predicate**, which decides that
  `num_buildings[T]` already counts the building when `n` is taken. Both
  readings read it from a folded body (`006432e2`); confidence is medium.
- **Plain-game reach of `check_capture`'s non-city swap branch and of a
  building bribe.** Both are unported in the crate. A skirmish capture that
  swaps a non-city building, diffed on `high_buildings`, settles it.
- **Cells versus tiles in `territory`.** This is a property of
  `territory`, not of this leaf. It decides whether `territory_high`'s
  checksum value agrees.
- All three leaves stay unpinned until the crate holds them and their rows
  on a kept capture diff clean.
