# victim-stamps — second (blind) reading

Slug: `victim-stamps`. Leaves: `attack_stamp` (LeaderData +0xf4, int[8]), `raid_stamp` (+0x114, int[8]), `ally_stamp` (+0x154, int[8]), `ctw_hero_stamp` (+0x200, int). Model: Claude Opus 5.5. Item 4152, wave 2, blind second reading; written without opening the first reading.

## Common ground: one function writes all four in the sim

**Layout.** `Leader` (0x6eec) has no fields of its own in `types.txt`; its
`this->field_0xNN` equals `LeaderData +0xNN` (`Leader::notify_deal (006d13f0)`
reads `field_0x8` as `who`; `Leader::calc_anti_attrition (006cdcc0)` writes
`field_0x7f4`, which is `anti_att`). So the prefix is 0, and the global slot
`i` is `0xe3a390 + i·0x6eec`. Absolute addresses: `attack_stamp` 0xe3a484,
`raid_stamp` 0xe3a4a4, `ally_stamp` 0xe3a4e4, `ctw_hero_stamp` 0xe3a590.

**Both nets.**
- Export: `->attack_stamp`, `->raid_stamp`, `->ally_stamp` hit only `CityData`
  (`CityData +0x14/+0x18`, a different record) and the logs;
  `->ctw_hero_stamp` hits only `LeaderData::log_data (006e5110)`.
  `field_0x(f4|114|154|200)` on a leaders expression: only
  `Object::do_damage (0064a480)` (as `leaders.list[0].field_0xNN + (who + v·0x1bbb)·4`)
  and `Leader::init (006e3930)` (`field_0x200 = 0`, and a loop over `+0xd4`).
  The `+0xf4` hits off `leaders.list[i].field_0x6eb8` are
  `LeaderDataEncrypt +0xf4`, a different record.
- Listing (`aa/exe.s`): every store to the four absolute addresses is inside
  `Object::do_damage (0064a480)`, range `0064a480..0064c880`. The only other
  absolute references are loads in `ScenarioFuncSet::was_raided (009fd140)`
  (`9fd19f`, `9fd21f`) and `ScenarioFuncSet::was_attacked (009fd290)`
  (`9fd2ef`, `9fd36f`). A scan of every instruction with displacement
  `0xf4/0x114/0x154/0x200` in functions named `Leader*`, `LeaderData*`,
  `Leaders*`, `Army*`, `Armies*`, `Diplomacy*`, `Scenario*` found one store
  to a `LeaderData`: `Leader::init (006e3930)` at `6e3dfc`
  (`movl %eax, 0x200(%ebx)`, `eax` = 0). The rest are `LeaderDataEncrypt`
  (`+0x6eb8` deref), window vtables, `ArmyData`/`LeaderData::log_data` stack
  locals, and `Army::find_muster_spot`'s `0x200(%edx)` off `GameAccess::objects`.

**The writer's arguments.** `Object::do_damage(this, target_index, victim,
…)`: `this` is the striking object, its owner byte at `+0x9` (the
**attacker** slot `a`); `param_2` (`0xc(%ebp)`) is the target's owner, the
**victim** slot `v`; the target is `objects[v].list[param_1]`. Every leaf
here is `leaders[v].<leaf>[a]` — the victim's row, the attacker's column —
built as `0xe3a4NN(,%ecx,4)` with `ecx = a + v·0x1bbb` (`0x24(%ebp)` holds
`v·0x1bbb`, set at `64a95d`). The value stored is always
`GameAccess::game->frame` (`[0xc061ec]+0x550`), the sim frame: a stamp, not
a counter.

**Preconditions on the whole block.** Returns before any of it when
`param_6 < 1` (`64a4ae`) or when the striker is a unit (`vslot +0x18`) with
`unit_masks & 1` (the decoy bit; `64a4dc`). Then at `64a932` `v == a` jumps
to `64b9df`, past all of it. Nothing else gates it: no ally test on the
attacker, no difficulty or game-mode test (except `ctw_hero_stamp`).

**Callers of `Object::do_damage (0064a480)`** (direct `call`s in the
listing): `Unit::fight` ×3 (`5fedda`, `5ff1fb`, `5ff33d`),
`Ammo::do_damage` ×3 (`678982`, `678b0a`, `678c6c`), `Nuke::do_damage` ×2,
and itself once (`64c4b7`, the spill onto builders). So every hit across
owners, melee, missile or nuke, goes through it.

**The shape of the block** (the listing, `64a8e1..64b9df`):

1. `heavy` (`-0x30(%ebp)`), set on a building path only. If the target
   has object flag `+8 & 1`, answers vslot `+0x20` (`is_build`), and
   answers `+0x4c` (`is_active`):
   - `heavy = 1` when the attacker's **type** answers vslot `+0x10c` (the
     crate and `docs/AI.md` name it `is_siege`) and the target answers
     `+0x1c` (`is_wallbuild`) (`64a8b8..64a8da`, carried in `-0x10`);
   - otherwise `ceiling = hits(0)/4`, or `hits(0)/10` when the target's
     type answers `+0x104` or the object flag `+8 & 0x20` (city) is set.
     `heavy = 1` iff `target.damage(+0x24) + this_hit (-0x24) >= ceiling`
     (`64aa13..64aa2e`).
   Then it takes the building's city (`vslot +0xac` → `BuildData +0x72`,
   `city`); `city < 0` skips to step 3.
   A unit target that is not a worker goes to the CtW hero arm (`64b3a0`).
   A worker whose action's gather order names an active building of `v`
   in a city takes the light city arm with that city (`64aa86..64ab29`).
   Any other target goes straight to step 3.
2. **The city arms** (`64ab4c`): `city_flags & 0x10` (capital, per
   `docs/CITIES.md` §1.4) picks the capital arm, else the plain arm; in
   either, `heavy == 0` goes to the light arm at `64b1f0`.
3. **The ground arm** (`64b56d..64b9df`), on **every** hit that got past
   `v != a`: the owner of the tile under the target (`world+0x134` cell
   array, stride 0x1c, signed byte `+0xf`; the cell from
   `div_3_table[x>>8]`, `div_3_table[y>>8]`) chooses raid or attack.

Since step 2's stores leave the stamp equal to `frame`, step 3's own
`frame − stamp > 300` test fails on the same hit: a hit that stamps in step
2 never stamps again in step 3.

## `attack_stamp` (LeaderData +0xf4, int[8])

### 1. Writers

All in `Object::do_damage (0064a480)`; four store instructions.

| store | arm | its gate |
|---|---|---|
| `64ace6` | capital, heavy | `frame − leaders[v].capital_stamp[a] > 300` (`64ab7e`, `0xe3a4c4`); stores `attack_stamp`, then `raid_stamp` (`64acf8`), then `capital_stamp` (`64ad0a`) |
| `64b037` | plain city, heavy | `frame − city.attack_stamp(+0x14) > 300` (`64aeb2`); stores `raid_stamp` (`64b025`) then `attack_stamp`; the city's `+0x14` and `+0x18` take `frame` after, unconditionally (`64b054`, `64b068`) |
| `64b383` | city, light (capital or not; also the worker arm) | `frame − city.raid_stamp(+0x18) > 300` (`64b201`); stores `raid_stamp` (`64b373`) then `attack_stamp`; the city's `+0x18` takes `frame` after, unconditionally (`64b398`) |
| `64b9d8` | ground arm, attack | `frame − leaders[v].attack_stamp[a] > 300` (`64b77c`) |

Export lines: 512, 623, 679, 883 of the export file of `Object::do_damage (0064a480)`.

### 2. Event, step, predicate

A hit by an object of slot `a` on an object of slot `v ≠ a`. The leaf takes
the sim frame (`leaders[v].attack_stamp[a] = frame`). Window constant
`0x12c` = 300 sim frames, strict `>`.

- **City arms** (target an active building of a city, or a worker gathering
  at one): the leaf is stamped whenever the city arm's own gate opens — the
  capital's per-attacker `capital_stamp`, or the city's attacker-blind
  `attack_stamp`/`raid_stamp`. Note the **light** arm stamps
  `attack_stamp` too, alongside `raid_stamp`, and the arms do not look at
  the leaf's own value.
- **Ground arm** (every hit, after the above; `64b56d`): let `owner` be the
  signed tile owner under the target.
  - `owner ≠ v` → attack arm.
  - `owner == v` → raid arm if (target `is_build` (`+0x20`) and not `heavy`)
    or `ObjectData::is_worker (0046fa10)` or vslot `+0xd0` (`UnitData::is_caravan`
    on a `Unit`) or `UnitData::is_merchant (0046d370)`; otherwise the attack arm
    (`64b5cc..64b62b`).
  - attack arm: if `frame − attack_stamp[a] > 300` then stamp (the notice
    and sound inside go only to `v == console->who` with
    `!is_ally(leaders[a], v)`; the store is outside that test, `64b9c6`).
- Exempt: `v == a`; a decoy striker (`unit_masks & 1`); `param_6 < 1`.
  No exemption by unit kind, ally status, or game mode.

### 3. Reset

`Leader::init (006e3930)` zeroes all 8 (`puVar23 = this+0xd4`,
`puVar23[8] = 0` per slot in the 8-pass loop, export lines 153–243;
unconditional, not under `param_2 < 0`). Callers of `Leader::init`:
`Setup::build_leader`, `Setup::build_game`, `Leaders::init`,
`MapScenario::make`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader`, `ScenarioRead::load_leader_chunk` ×6 and
`WalkDataGame::walk_data (005a2360)`. Nothing in the sim decays or clears it
mid-game. A save load restores it through the walk (`Leaders::walk_data` →
`LeaderData::walk_data (006d6750)`, whose range `[+0x8, +0x692a)` covers
it per `crates/sim/src/checksum/leaders.rs`); I did not read the walk's
read direction.

### 4. Readers

- `Object::do_damage (0064a480)` `64b77c`: its own ground-arm window.
- `ScenarioFuncSet::was_attacked (009fd290)`: trigger condition; slots are
  1-based there; 0 → false; with a positive argument `n`, true iff
  `(frame − stamp)/15 ≤ n` (so `n` is in units of 15 frames); a second form
  scans all live attackers.
- `LeaderData::log_data (006e5110)`: per slot, `this->diplos` loop,
  `piVar3[0x20]` (export ~line 287).
- The checksum walk (the crate's `LEADER_FIELDS` lists
  `("attack_stamp", 0xf4, 4, 8)`).
No AI or economic reader was found in either net.

### 5. Crate site

`crates/sim/src/combat/fight/delivery.rs`: `Sim::city_alarm` (line 280)
already ports the city arms and writes the city's `attack_stamp`/
`raid_stamp` and the victim's `capital_stamp`; its doc says the leader
`attack_stamp`/`raid_stamp`/`ally_stamp` rows are **not carried**. A
builder adds:
- the city-arm stores inside `city_alarm` (capital heavy inside the
  `frame − *stamp > 300` branch; plain heavy under `alarm`; light under a
  `frame − c.raid_stamp > 300` test it does not have today — it stamps the
  city's `raid_stamp` blind);
- the ground arm, which the crate has **nowhere**: it belongs in
  `Sim::do_damage_round` (same file, line 70) right after the
  `city_alarm` call at line 190, before `take_damage_typed`, for every hit
  with `owner(attacker) ≠ owner(target)` — including unit targets and
  buildings outside cities;
- the field: next to `capital_stamp: [i64; 8]` in `crates/sim/src/ai.rs`
  (line 704), and a `"attack_stamp" =>` arm in `Sim::leader_image`
  (`crates/sim/src/checksum/leaders.rs`, beside `"capital_stamp"` at line
  998).
Note: `city_alarm` returns early for a worker (the light arm) and leaves
the capital light hit to `c.raid_stamp = frame`, matching the listing.

### 6. Plain game?

Yes. Any skirmish hit across owners reaches `64b9d8` or one of the city
arms; nothing is behind the trigger layer, CtW or multiplayer. Only the
reader `was_attacked` is trigger-layer.

### 7. Confidence

High on the stores, the index (`a + v·0x1bbb`), the 300-frame windows and
the arm choice — all read from the listing. Medium on the names of vslots
`+0x10c` (type) and `+0x104` (target type), taken from the crate and
`docs/AI.md`; `+0xd0` named from `Unit::vftable`. Not read: whether a
gaia/neutral owner can arrive as `v` and what slot it indexes; which object
is `this` on an `Ammo::do_damage` call (shooter or projectile — matters for
`a`); the walk's read direction on load.

## `raid_stamp` (LeaderData +0x114, int[8])

### 1. Writers

All in `Object::do_damage (0064a480)`; four store instructions.

| store | arm | its gate |
|---|---|---|
| `64acf8` | capital, heavy | `frame − capital_stamp[a] > 300` (same block as `attack_stamp` `64ace6`) |
| `64b025` | plain city, heavy | `frame − city.attack_stamp > 300` |
| `64b373` | city, light (and the worker arm) | `frame − city.raid_stamp > 300` |
| `64b755` | ground arm, raid | `frame − leaders[v].raid_stamp[a] > 300` (`64b64a`) |

Export lines: 510, 621, 681, 792.

### 2. Event, step, predicate

Same event as `attack_stamp`: a hit by slot `a` on slot `v ≠ a`; the leaf
takes the sim frame; window 300 frames, strict `>`.

- City arms: stamped together with `attack_stamp` on every opening of a
  city arm's gate (heavy or light, capital or not). There is **no** city
  arm that stamps `raid_stamp` without `attack_stamp`, or the reverse.
- Ground arm, raid branch: tile under the target owned by `v`, and the
  target is a building hit lightly (`is_build` and `heavy == 0`), a worker,
  a caravan or a merchant. Stamp if `frame − raid_stamp[a] > 300`. The
  notice (`S_RAID_ATTACKED`) goes to `v == console->who` with
  `!is_ally(leaders[a], v)`; the store at `64b743..64b755` is outside it.
- Note `heavy` on a non-building target is 0 but irrelevant there: a unit
  target reaches the raid branch only by being a worker/caravan/merchant.
  A building whose `heavy` was computed (active, `+8 & 1`) keeps it even if
  it has no city (`city < 0` skips to the ground arm with `heavy` as
  computed); an inactive building reaches the ground arm with `heavy = 0`
  and so raids.
- A wall: `+0x20` is 0 on a Wall (`docs/GROUPS.md`), so a wall hit skips
  the city arms and, in its owner's ground, is not "a building hit
  lightly"; with no worker/caravan/merchant answer it takes the **attack**
  branch. (Read from the vslot values; not traced on a Wall object.)
- Exempt: as `attack_stamp`.

### 3. Reset

`Leader::init (006e3930)`, `puVar23[0x10] = 0` per slot (base `+0xd4`,
i.e. `+0x114`). Same callers and same load path as `attack_stamp`.

### 4. Readers

- `Object::do_damage (0064a480)` `64b64a`: its ground-arm window.
- `ScenarioFuncSet::was_raided (009fd140)`: same shape as `was_attacked`
  (1-based slots, `(frame − stamp)/15 ≤ n`).
- `LeaderData::log_data (006e5110)`: `piVar3[0x28]`.
- The checksum walk (`("raid_stamp", 0x114, 4, 8)` in the crate's list).

### 5. Crate site

As `attack_stamp`: the city-arm stores in `Sim::city_alarm`
(`crates/sim/src/combat/fight/delivery.rs:280`), the ground arm new in
`Sim::do_damage_round` (same file, after line 190), the field beside
`capital_stamp` in `crates/sim/src/ai.rs:704`, the image arm in
`crates/sim/src/checksum/leaders.rs` near line 998. The crate's
`is_worker`, `building_is_city`, `owner_of` already exist; a tile-owner
lookup and an is-caravan/is-merchant test on the target are needed for the
ground arm (not checked whether the crate has a merchant predicate).

### 6. Plain game?

Yes, by the same writers as `attack_stamp`. `was_raided` is trigger-layer.

### 7. Confidence

High on stores, index, windows. The ground-arm predicate is read from the
listing (`64b5cc..64b62b`) and agrees with the export's lines 743–763. Not
read: the Wall case end to end; gaia as `v`; `this` on the Ammo path.

## `ally_stamp` (LeaderData +0x154, int[8])

### 1. Writers

One store instruction, `Object::do_damage (0064a480)` `64ae8d`
(`movl %eax, 0xe3a4e4(,%ecx,4)`), reached from two paths:
- capital heavy: falls through at `64ae79` after the gate at `64ad1c`;
- plain city heavy: `64b1e4..64b1eb` jumps back to `64ae82`, after the gate
  at `64b076`.
Export lines 735 (and the `goto LAB_0064ae82` from 717).

### 2. Event, step, predicate

A **heavy** hit by slot `a ≠ v` on an active building of one of `v`'s
cities (capital or not). Not on a light hit, not on a worker, not in the
ground arm. Step: `leaders[v].ally_stamp[a] = frame`, when
`frame − ally_stamp[a] > 300` (strict). The test is independent of
whether the city arm's own gate (capital `capital_stamp`, or city
`attack_stamp`) opened on this hit — it runs after it either way
(`64ab8a jle 64ad11`; `64aeba jle 64b043` → `64b06b`).

The name is from the message it gates: the "ally under attack" notice and
`S_ALLY_UNDER_ATTACK` go to the console player only when
`v ≠ console->who` and `is_ally(leaders[v], console->who)`
(`LeaderData::is_ally (006edb50)` on the victim, `64ad46`, `64b0a0`). The
store is outside that test: every leader's row is stamped regardless of who
is local, so the leaf is deterministic. The two arms' inner notice timing
reads the leaf against 300 (capital, `64ae27`) and 400 (plain, `64b192`)
frames — UI only.

Exempt: light hits; non-city buildings (`BuildData.city < 0`); units;
inactive buildings; walls (`is_build` 0); `v == a`; decoy strikers.

### 3. Reset

`Leader::init (006e3930)`, `puVar23[0x20] = 0` per slot (`+0xd4 + 0x80`).
Same callers and load path as `attack_stamp`.

### 4. Readers

Only `Object::do_damage (0064a480)` itself (`64ad1c`, `64b076` gates;
`64ae0f`, `64b17a` notice timing) and `LeaderData::log_data (006e5110)`
(`piVar3[0x38]`), plus the checksum walk. No `ScenarioFuncSet` reader.

### 5. Crate site

`Sim::city_alarm`, `crates/sim/src/combat/fight/delivery.rs:280`: after the
`heavy` test, in both the capital branch (whether or not
`frame − capital_stamp > 300`) and the plain branch, add
`if frame − ally_stamp[vic][att] > 300 { ally_stamp[vic][att] = frame }`.
Note the capital branch there `return`s inside the capital_stamp test, so
the ally store must precede those returns. Field beside `capital_stamp` in
`crates/sim/src/ai.rs:704`; image arm in
`crates/sim/src/checksum/leaders.rs` near 998.

### 6. Plain game?

Yes: any heavy hit on a city building in a skirmish.

### 7. Confidence

High: one store, two paths, both read in the listing. Not read: the
`notify_flags` bits (UI). The heavy predicate shares the medium-confidence
vslot names given under `attack_stamp`.

## `ctw_hero_stamp` (LeaderData +0x200, int)

### 1. Writers

- `Object::do_damage (0064a480)` `64b513` (`movl %edx, 0xe3a590(%ecx)`,
  `ecx = v·0x6eec` from `-0x28(%ebp)`, `edx = frame`). Export line 382
  (`local_2c[0xfb32].field_0x40`, where `local_2c` is `v·0x6eec` typed as
  `Armies*`, size 0xe8: `0xfb32·0xe8 + 0x40 = 0xe3a590`).
- `Leader::init (006e3930)` `6e3dfc`, the clear to 0 (export line 267).
No other store in either net.

### 2. Event, step, predicate

Scalar, the victim's own (no attacker index). On a hit by `a ≠ v` on a
**unit** of `v` that is not a worker (`64aa80 je 64b3a0`), all of:
- `game->semaphore[2] & 2` (byte `+0x822` bit 1; the Conquer the World flag,
  per `docs/NATIONS.md`, `docs/ECONOMY.md`, `docs/RECGAME.md`) (`64b3a6`);
- the target's type index (`[obj+0x18]+4`) equals
  `GameAccess::conquest_game->hero_type` (`[0xc0617c]+0x118`) (`64b3be`);
- `frame − ctw_hero_stamp > 120` (`0x78`, strict) (`64b3d3`);
- `v == MiscAccess::console->who` (`[0xc06210]+0x298`), the **local**
  player only (`64b3ea`);
- `(hits(0) − damage) · 100 / hits(0) < 70` (`0x46`), integer `idiv`
  toward zero, `hits` through vslot `+0x11c` (inlined `UnitData::hits` reads
  `+0x20`), `damage` at `+0x24`, both before this hit is taken
  (`64b43e..64b44b`).
Then a message is posted, and if `frame − ctw_hero_stamp > 75` (`0x4b`)
the leaf takes `frame` (`64b508..64b513`) before the notice is drawn; the
sound (`0x83`) plays either way. Since the outer gate already demands
`> 120`, the inner `> 75` always holds: in effect the leaf is stamped every
time all five conditions hold, at most once per 121 frames.

Note: the `v == console->who` test makes this a per-machine store into a
field the checksum walk covers (`[+0x8, +0x692a)`). Whether CtW can be
multi-human, so that this could part two machines, was not read.

### 3. Reset

`Leader::init (006e3930)` only (`= 0`). Callers as under `attack_stamp`.

### 4. Readers

`Object::do_damage (0064a480)` itself (`64b3d3`, `64b508`) and
`LeaderData::log_data (006e5110)` (export line 582, `pushl 0x200(%edi)` at
`6e661c`); the checksum walk. No AI reader.

### 5. Crate site

None: the crate has no Conquer the World (cut from v1) and no hero-type of
the conquest game. In `crates/sim/src/checksum/leaders.rs` the leaf is
listed (`("ctw_hero_stamp", 0x200, 4, 1)`, line 72) and today falls to
`Sim::leader_image`'s `_ => None` (line 1305). A builder's whole change is
to add `"ctw_hero_stamp"` to the "never reached without a cut track" zero
arm (`Some(vec![0; count])`, lines ~1246–1275), with
`Object::do_damage (0064a480)` `64b513` and `Leader::init`'s clear as its
two stores.

### 6. Plain game?

No. The store sits behind `semaphore[2] & 2`, Conquer the World. In a
skirmish the field is `Leader::init`'s 0 for the whole game.

### 7. Confidence

High: one sim store, every condition read in the listing. The CtW meaning
of `semaphore[2] & 2` is taken from the other documents, not re-derived
here. Not read: who sets `conquest_game->hero_type`; the multi-human CtW
question.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `attack_stamp[a]` (+0xf4) | `Object::do_damage (0064a480)` at `64ace6` (capital heavy), `64b037` (city heavy), `64b383` (city light), `64b9d8` (ground arm, attack) | `= frame` on victim `v`'s row, attacker `a`'s column; city arms when their own 300-frame gate opens; ground arm when `frame − stamp > 300` and (tile not `v`'s, or target not light-building/worker/caravan/merchant) | `Leader::init (006e3930)` | `Sim::city_alarm` + new ground arm in `Sim::do_damage_round` (`crates/sim/src/combat/fight/delivery.rs`); field beside `capital_stamp` (`crates/sim/src/ai.rs`); `leader_image` arm | yes | high (vslot names medium) |
| `raid_stamp[a]` (+0x114) | `Object::do_damage (0064a480)` at `64acf8`, `64b025`, `64b373` (same three city arms), `64b755` (ground arm, raid) | `= frame`; city arms always together with `attack_stamp`; ground arm when `frame − stamp > 300`, tile `v`'s, and target a lightly hit building, worker, caravan or merchant | `Leader::init (006e3930)` | as `attack_stamp` | yes | high |
| `ally_stamp[a]` (+0x154) | `Object::do_damage (0064a480)` at `64ae8d` (one store, capital-heavy and city-heavy paths) | `= frame` on a heavy hit on a city building when `frame − stamp > 300`, regardless of the city arm's own gate; store not tied to the local-ally notice | `Leader::init (006e3930)` | `Sim::city_alarm`, before its capital-branch returns | yes | high |
| `ctw_hero_stamp` (+0x200) | `Object::do_damage (0064a480)` at `64b513`; `Leader::init (006e3930)` at `6e3dfc` (clear) | `= frame` when CtW, target is the conquest hero type, `v` is the local player, hp below 70 %, and `frame − stamp > 120` | `Leader::init (006e3930)` | none (CtW cut); add to `leader_image`'s cut-track zero arm in `crates/sim/src/checksum/leaders.rs` | no (CtW only) | high |

Incidental: a `grep` over `docs/*.md` for `semaphore` printed one line
from `docs/CHECKSUM.md` §20 (line 1710, the conquest arrays, not these
leaves). Nothing else from the barred files was opened.
