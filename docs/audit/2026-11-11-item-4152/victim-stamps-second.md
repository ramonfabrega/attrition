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

Export lines: 512, 623, 679, 883 of `do_damage@0064a480.c`.

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

