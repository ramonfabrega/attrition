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
written from its family's verdicts alone. **No diff backs any rule here
yet.** Where a kept dump was consulted, the section says so, and treats that
value as a witness, not a pinned diff.

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
(`docs/audit/2026-11-11-item-4152/city-name-*.md`); no diff backs it yet.

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
   heavy 0 (capital or not), or T is a worker (type 0x32..0x35) whose gather
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
frame 91 and run1100 frame 5408 (entry 120 §3.4). That reading is a
witness, not a pinned diff.

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
