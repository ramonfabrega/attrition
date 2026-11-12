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
