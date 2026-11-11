# Item 4152, family 4 (`victim-stamps`) — first reading

- Family: 4, slug `victim-stamps`.
- Leaves: `attack_stamp` (`LeaderData+0xf4`, int[8]), `raid_stamp` (`+0x114`,
  int[8]), `ally_stamp` (`+0x154`, int[8]), `ctw_hero_stamp` (`+0x200`, int).
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), first reading, 2026-11-11.
- Sources: the decompile export `~/ghidra-projects/decomp/` and the listing
  of `riseofnations.exe` (`llvm-objdump`). Nothing copied beyond addresses
  and constants.

## 0. Shared ground (read once, cited by every leaf)

**Layout.** `types.txt`: `LeaderData` (0x6ee4) has `attack_stamp` int[8] at
+0xf4, `raid_stamp` int[8] at +0x114, `capital_stamp` int[8] at +0x134
(neighbour, not this family), `ally_stamp` int[8] at +0x154,
`ctw_hero_stamp` int at +0x200. `Leader` (0x6eec) holds `LeaderData` at
+0: `Leader::Leader (006d2bf0)` calls `LeaderData::LeaderData` on `this`
unadjusted and puts the `_vbtable_` at +0x6ee4. So a `Leader*`'s
`field_0xNN` is `LeaderData +NN`; there is no second offset to grep.
Slot `i`'s record is at `0xe3a390 + i*0x6eec`; one int of an int[8] is
`0xe3a390 + off + 4*(i*0x1bbb + j)` (0x1bbb = 0x6eec/4), which is how
every site below indexes: `off(,%ecx,4)` with `ecx = victim*0x1bbb + j`.

**Both nets.**
- Export, by name: `->attack_stamp|->raid_stamp|->ally_stamp|->ctw_hero_stamp`
  hits only `CityData`'s own same-named fields (`Unit::do_trade
  (005ed270)`, `CityData::log_data (004895c0)`) and
  `LeaderData::log_data (006e5110)` (reads `ctw_hero_stamp`). By offset,
  in functions that touch a leader: `Object::do_damage (0064a480)`
  (`leaders.list[0].field_0xf4/0x114/0x154 + (who + param_8)*4`, and
  `local_2c[0xfb32].field_0x40` with `local_2c = victim*0x6eec` cast to
  `Armies*` (size 0xe8): 0xfb32*0xe8 + 0x40 = 0xe3a590 = slot 0's
  `ctw_hero_stamp`), and `Leader::init (006e3930)` (`field_0x200`, and
  the arrays through `puVar23 = &field_0xd4` stepping one int per `j`).
- Listing, absolute: every reference in `.text` to an address in
  [0xe3a394, 0xe3a5ff] was mapped to its function (INDEX.tsv). For the
  four leaves the only functions are `Object::do_damage` (stores and
  reads), `ScenarioFuncSet::was_attacked (009fd290)` and
  `ScenarioFuncSet::was_raided (009fd140)` (reads only).
- Listing, register-relative: every `0xf4(`, `0x114(`, `0x154(`, `0x200(`
  displacement in a `Leader*`/`LeaderData*` method was listed. None is a
  store to these leaves except `Leader::init`'s (`6e3c12`, `6e3c15`,
  `6e3c1b` off `esi = this+0xd4`; `6e3dfc` `0x200(%ebx)`). The other hits
  are reads of `LeaderDataEncrypt` through `+0x6eb8` (`xor 0x63187`:
  `Leader::create_buildings`, `Leader::research_techs`,
  `LeaderData::get_lowest_epoch`/`get_highest_epoch`,
  `calc_science_discount`) or vtable calls. No indexed
  `0xf4(%r,%r,4)`-shaped access exists in any leader function.
- Bulk writers: `Leaders::clear (006ed5d0)` memsets each slot's
  [+0, +0x6929) to 0 (all four leaves). `LeaderData::walk_data (006d6750)`
  walks [+0x8, +0x692a) (`docs/CHECKSUM.md` §8), so a load restores the
  leaves and the checksum covers them.

**`Object::do_damage (0064a480)` — the one game-time writer.** `this` is
the attacker object A; `A+0x9` (byte) its owner `att`; `param_1` the
target's index, `param_2` the target's owner `vic`. The stamps block runs
on every blow that passes the head returns (`count < 1`, a decoy), after
scaling and before `take_damage` (vslot +0x16c): `64a932 cmp vic, att;
je 64b9df` skips it when the owners are equal, and `64b9df` is the
take_damage setup, so every store below lands **whether or not the blow
then kills**. At `64a95d` the victim's row base `vic*0x1bbb` is saved in
`param_8`'s stack slot (`0x24(%ebp)`), and every stamp is
`leaders[vic].X[att]`: **the victim's row, the attacker's column.**
`frame` is `game->frame` (`[0xc061ec]+0x550`), sim frames. The window
constant everywhere is `0x12c` = 300 frames, compared `frame - stamp > 300`
(`cmpl $0x12c; jle skip`).

Terms (vslots named by `rise.pdb`'s `LF_ONEMETHOD` offsets in the
`SubObjectData` field list, not by `vtables.txt`'s folded names):
`is_unit` +0x18, `is_wallbuild` +0x1c, `is_build` +0x20, `is_active`
+0x4c; on the type, `is_defense` +0x104 and `is_siege` +0x10c (the
type-class field list, offsets 260 and 268); `UnitData::is_caravan`
+0xd0, `UnitData::hits` +0x11c (vtables.txt, Unit's own entries).

- `siege` = A's type `is_siege` and the target `is_wallbuild`
  (`64a8b3`..`64a8e3`).
- `heavy` (only computed on the building arm): `siege`, or
  `T.damage(+0x24) + whole >= ceiling` where `whole` is this blow's
  scaled whole hits (`-0x24(%ebp)`, after the >>4 split) and `ceiling` =
  `hits/10` (signed, `0x66666667` multiply) when T's type `is_defense` or
  `T.obj_flags(+8) & 0x20`, else `hits/4` (signed, toward zero)
  (`64a9a1`..`64aa38`; `jl` to light).
- **Building arm**: T `obj_flags & 1`, `is_build`, `is_active`
  (`64a97a`..`64a99b`), and T's `BuildData +0x72` city ≥ 0 (`64ab34`).
  City `c`; capital iff `c.city_flags(+4) & 0x10` (`64ab52`).
- **Unit arm** (T fails the building test): if T `is_unit` and
  `is_worker` (type id 0x32..0x35, `ObjectData::is_worker (0046fa10)`),
  its current action's gather order (`UnitData::get_action (00608450)`,
  order vslot +0xe4) names an active building of `vic`'s
  (`is_active_build (0046fb40)`) whose `+0x72` city ≥ 0: that building's
  city takes the **light** path (`local_34` = 0) (`64aa70`..`64ab2c`).
  If T `is_unit` and not a worker: the CtW hero test (§4). Anything else
  (not a unit, or a non-building wall etc.): straight to the territory
  step.
- **Territory step** (`64b56d`..`64b9d8`), reached by **every** cross-owner
  blow after whichever arm ran: `land = wdata[xs*div3[T.y>>8] +
  div3[T.x>>8]].who` (`WorldData +0x134 wdata`, `WData +0xf who`, signed
  char; T's x/y at +0x10/+0x14 under `xor 0x63637`). Then
  - `land == vic` and (T `is_build` and not heavy) → **raid** test.
    `heavy` (`-0x30(%ebp)`) is zeroed at `64a946` and set only on the
    building arm, after the active test and before the city test, so a
    city-less active building keeps its own heavy/light answer, and an
    inactive building (or one with `obj_flags & 1` clear) is always light
    here;
  - `land == vic` and T `is_worker`, or `is_caravan`, or
    `UnitData::is_merchant (0046d370)` (type id 0x3d, 0x3e or 0x190) →
    **raid** test;
  - otherwise (`land != vic`, or a non-worker non-caravan non-merchant
    target in the victim's land, including a heavily hit building) →
    **attack** test.

## 1. `attack_stamp` (`LeaderData +0xf4`, int[8])

**1. Writers, both nets.**

| writer | site(s) | value |
|---|---|---|
| `Object::do_damage (0064a480)` | `64ace6` (capital, heavy), `64b037` (city, heavy), `64b383` (building light / worker arm), `64b9d8` (territory attack test); read at `64b77c` | `frame` |
| `Leader::init (006e3930)` | `6e3c12` (`0x20(%esi)`, `esi = this+0xd4+4j`), loop over `j = 0..7` | 0 |
| `Leaders::clear (006ed5d0)` | memset of [+0, +0x6929) per slot | 0 |
| `LeaderData::walk_data (006d6750)` | a load restores [+0x8, +0x692a) | saved |

**The README row's lead is contradicted**: neither `Leader::plan_strategy
(006b9620)` nor `Leader::end_process (006b8690)` touches +0xf4. Checked
by name and offset in the export, by absolute address (no `0xe3a484`
reference outside `do_damage` and `was_attacked`), and by every store in
`end_process`'s listing (it stores `0x74`..`0x448(%edx)` words and
`0xe37f30(%edx)`, nothing of this record's +0xf4). `plan_strategy`'s
`+0x290` and `+0x94` row writes may be what the lead saw.

**2. Event, step, predicate.** Each write sets `leaders[vic].attack_stamp[att]
= frame` (sim frames), for a blow by A (owner `att`) on T (owner `vic`),
`att != vic`, before the take. Four arms (§0's terms):

1. **Capital, heavy** (`64ab5c`..`64ad0a`): when `frame -
   leaders[vic].capital_stamp[att] > 300`, write `attack_stamp`,
   `raid_stamp` and `capital_stamp` `[att]` = frame (and set the alarm
   `local_30`). The city's own stamps are not touched.
2. **Non-capital city building, heavy** (`64ae99`..`64b037`): when `frame -
   c.attack_stamp(+0x14) > 300`, write `raid_stamp[att]` and
   `attack_stamp[att]` = frame (and the alarm); then, unconditionally,
   `c.attack_stamp = c.raid_stamp = frame` (`64b054`, `64b068`).
3. **Light on any city building, capital or not, and the worker arm**
   (`64b1f0`..`64b398`): when `frame - c.raid_stamp(+0x18) > 300`, write
   `raid_stamp[att]` **and `attack_stamp[att]`** = frame; then,
   unconditionally, `c.raid_stamp = frame`. A light raid does stamp the
   leader's attack row.
4. **Territory, attack test** (`64b761`..`64b9d8`, every cross-owner blow
   after the arm): when the attack branch is chosen (T not on `vic`'s land;
   or on it and T is neither a light-hit building, a worker, a caravan nor
   a merchant) and `frame - attack_stamp[att] > 300`, write
   `attack_stamp[att] = frame`.

No arm is gated on the human: `console->who` only gates the messages and
sounds (`cmp 0x298(%eax)` then `jne` straight to the store). The
territory attack message additionally tests `!is_ally(leaders[att],
vic)`, `tribe_can_type`, and the target's vslot +0xb8 for its text;
none of that guards the store. Arms 1–3 and 4 can fire on the same blow;
because an arm writes `frame` first, the territory test then sees 0 < 300
and writes nothing more, so the value is the same either way.

**3. Reset.** `Leader::init` zeroes all eight entries (the loop at
`6e3c0d`..`6e3c2d` zeroes `+0xb4`, `+0xd4`, `+0xf4`, `+0x114`, `+0x134`,
`+0x154`, `+0x174`, `+0x194`, `+0x1b4` and sets `+0x1d4 hire_who` to −1,
for each `j`). `Leaders::clear` zeroes the record. No per-frame fold, no
`reset_score` path. Since 0 is the initial value, the first cross-owner
hit after frame 300 always passes the window; a hit at frame 300 or
earlier does not (if the counter starts a game at 0, not checked here) (`frame - 0 > 300` fails), so in the first 300 frames this row
is never written by arms 1–4. `Leader::init`'s callers: `Leaders::init`,
`Setup::build_game`, `Setup::build_leader`, `WalkDataGame::walk_data`,
`MapScenario::make`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader`, `ScenarioRead::load_leader_chunk` (×6).

**4. Readers.** `Object::do_damage` itself (the territory window at
`64b77c`). `ScenarioFuncSet::was_attacked (009fd290)`: the trigger
predicate, true when the stamp is non-zero and `(frame - stamp)/15 <=
seconds` (15 frames to the second; 0 seconds means "ever"). It indexes
`leaders[victim−1].attack_stamp[attacker−1]`, the same orientation.
`LeaderData::log_data (006e5110)` prints it per `j` (`piVar3[0x20]` off
`&diplos[j]`, line 288 of the export). `CheckSums::check_leaders` folds it
through `walk_data`. No AI decision reads it in the export.

**5. Crate.** No row exists. `crates/sim/src/checksum/leaders.rs:61` lists
`("attack_stamp", 0xf4, 4, 8)` with no value arm in the name match near
line 998. The event is `Sim::do_damage_round`
(`crates/sim/src/combat/fight/delivery.rs:70`), which calls
`Sim::city_alarm` (`delivery.rs:280`) before `take_damage_typed`: arms
1–3 belong inside `city_alarm` (its doc comment says the rows "are not
carried"), and arm 4 is a new step after the `city_alarm` call at
`delivery.rs:190`, using `World::owner_at` (`crates/sim/src/world.rs:2013`,
the cell's `who`) at T's position. The row would sit beside
`AiLeader::capital_stamp` (`crates/sim/src/ai.rs:704`). Note for the
builder: `city_alarm` returns early for `att == vic` and for a non-city
building, but the territory step must still run for every cross-owner
blow, city or not, unit or building.

**6. Plain game.** Yes. Arm 4 fires on any cross-owner hit after frame
300 in a skirmish against the computer; arms 1–3 on hits on city
buildings and on gathering citizens. Only the reader `was_attacked` is
the trigger layer.

**7. Confidence and not read.** High for the stores, the orientation and
the 300 window (listing read at every store). The territory branch's
`is_caravan` slot +0xd0 is named from Unit's vtable; on a building or a
wall target I assumed it answers 0 (Build's +0xd0 is a folded `return 0`;
a wall's was not checked). `is_merchant`'s three type ids are from its
listing; which units they are was not looked up. Callers of
`do_damage` (`Ammo::do_damage` ×3, `Nuke::do_damage` ×2, `Unit::fight` ×3,
itself ×1) were not followed into their own predicates.

## 2. `raid_stamp` (`LeaderData +0x114`, int[8])

**1. Writers, both nets.**

| writer | site(s) | value |
|---|---|---|
| `Object::do_damage (0064a480)` | `64acf8` (capital, heavy), `64b025` (city, heavy), `64b373` (building light / worker arm), `64b755` (territory raid test); read at `64b64a` | `frame` |
| `Leader::init (006e3930)` | `6e3c15` (`0x40(%esi)`), `j = 0..7` | 0 |
| `Leaders::clear (006ed5d0)` | memset | 0 |
| `LeaderData::walk_data (006d6750)` | load | saved |

Agrees with the README row's `Object::do_damage`. No other function
references `0xe3a4a4` but `ScenarioFuncSet::was_raided (009fd140)`
(reads).

**2. Event, step, predicate.** `leaders[vic].raid_stamp[att] = frame`, same
blow, same orientation, before the take.

1. **Capital, heavy**: with `attack_stamp` and `capital_stamp`, when
   `frame - capital_stamp[att] > 300` (§1 arm 1).
2. **Non-capital city building, heavy**: with `attack_stamp`, when
   `frame - c.attack_stamp > 300` (§1 arm 2). Note the gate is the
   **city's** `attack_stamp`, not this row.
3. **Light on a city building, and the worker arm**: with
   `attack_stamp`, when `frame - c.raid_stamp > 300` (§1 arm 3); gate is
   the city's `raid_stamp`.
4. **Territory, raid test** (`64b630`..`64b755`): T on `vic`'s land
   (`WData.who == vic`) and T a light-hit (or inactive) building, a
   worker, a caravan or a merchant; when `frame - raid_stamp[att] > 300`,
   write `raid_stamp[att] = frame`. The message (victim is the console
   player and `!is_ally(leaders[att], vic)`) does not guard the store.

The raid test and the attack test are exclusive per blow; arms 1–3 can
precede either.

**3. Reset.** As §1: `Leader::init`'s loop (`+0x114`), `Leaders::clear`.

**4. Readers.** `do_damage`'s raid window (`64b64a`).
`ScenarioFuncSet::was_raided (009fd140)` (trigger layer; same shape as
`was_attacked` by its listing's two `0xe3a4a4` reads, body not read line
by line). `LeaderData::log_data` (`piVar3[0x28]`, line 300). The
checksum walk. No AI reader.

**5. Crate.** None. `checksum/leaders.rs:62` row, no value. Same sites as
§1: `Sim::city_alarm` (`delivery.rs:280`) for arms 1–3 (it already
writes the **city's** `raid_stamp`, `crates/sim/src/cities/city.rs`
`raid_stamp`, at `delivery.rs:314`, `:337`, `:351`), and a territory step
after `delivery.rs:190` for arm 4. The crate's `is_worker`, `is_merchant`
(`crates/sim/src/economy/calc_gather.rs:555`) and caravan predicates
exist; `Sim::is_merchant` is already `UnitData::is_merchant (0046d370)`'s
port (`MERCHANTS`).

**6. Plain game.** Yes: hits on citizens, caravans and merchants in their
owner's territory, and light hits on buildings there, are ordinary
skirmish events.

**7. Confidence and not read.** High for the stores and gates.
`was_raided`'s body was read only through its two listing loads.

## 3. `ally_stamp` (`LeaderData +0x154`, int[8])

**1. Writers, both nets.**

| writer | site(s) | value |
|---|---|---|
| `Object::do_damage (0064a480)` | `64ae8d` (the one store, `LAB_0064ae82`, joined from both heavy arms); reads at `64ad1c`, `64ae0f`, `64b076`, `64b17a` | `frame` |
| `Leader::init (006e3930)` | `6e3c1b` (`0x80(%esi)`), `j = 0..7` | 0 |
| `Leaders::clear (006ed5d0)` | memset | 0 |
| `LeaderData::walk_data (006d6750)` | load | saved |

Agrees with the README row. No reference to `0xe3a4e4` outside
`do_damage`, not even a trigger reader.

**2. Event, step, predicate.** `leaders[vic].ally_stamp[att] = frame`, on
a **heavy** hit on a city building only, capital or not:

- Capital heavy (`64ad11`): after the capital test (whether or not it
  stamped), `frame - ally_stamp[att] > 300` → store.
- Non-capital heavy (`64b06b`): after the city stamps,
  `frame - ally_stamp[att] > 300` → store.

Inside the window, the "ally under attack" message and sound play when
`vic != console->who` and `is_ally(leaders[vic], console->who)`
(`LeaderData::is_ally (006edb50)`); both negative branches jump to the
store (`64ad3c je 64ae7c`, `64ad53 je 64ae79`), so the store is
unconditional once the window passes. Never on a light hit, the worker
arm, a unit, or a city-less building. The row is named for the
notification it throttles, but it is written for every player pair,
human or not.

**3. Reset.** `Leader::init`'s loop (`+0x154`), `Leaders::clear`.

**4. Readers.** Only `do_damage`'s own windows (and the message's
`do_notice` throttle: it reloads the stamp as the "last notice" time,
`400 <` for the non-capital arm, `300 <` for the capital arm, before the
store), `LeaderData::log_data` (`piVar3[0x38]`, line 324) and the
checksum walk. Nothing decides with it.

**5. Crate.** None; `checksum/leaders.rs:64` row, no value. The site is
`Sim::city_alarm` (`delivery.rs:280`), in both heavy arms, after the
capital test and after the city stamps respectively. Note that the
crate's capital arm returns from inside its `if frame - *stamp > 300`
block (`delivery.rs:341`..`:346`); the ally store must run on both
outcomes of that test, so the early `return` has to move below it.

**6. Plain game.** Yes: any heavy hit on a computer's or a human's city
building by another player, after frame 300.

**7. Confidence and not read.** High; one store, read in the listing with
both message exits. The `do_notice` throttle values (400/300) are read
from the decompile only; they affect the interface, not the stamp.

## 4. `ctw_hero_stamp` (`LeaderData +0x200`, int)

**1. Writers, both nets.**

| writer | site(s) | value |
|---|---|---|
| `Object::do_damage (0064a480)` | `64b513` (`movl %edx, 0xe3a590(%ecx)`, `ecx = vic*0x6eec` from `-0x28(%ebp)`); reads at `64b3d3`, `64b508` | `frame` |
| `Leader::init (006e3930)` | `6e3dfc` (`0x200(%ebx)`, `ebx = this`) | 0 |
| `Leaders::clear (006ed5d0)` | memset | 0 |
| `LeaderData::walk_data (006d6750)` | load | saved |

Agrees with the README row. The export prints the `do_damage` site as
`local_2c[0xfb32].field_0x40` (§0); by name it appears only in
`LeaderData::log_data`. Beside it `Leader::init` sets
`ctw_hero_retreat_stamp` (+0x204) to −1000 (`0xfffffc18`), not this leaf.

**2. Event, step, predicate.** A single int per leader, not per attacker:
`leaders[vic].ctw_hero_stamp = frame`, in the **unit arm** of a
cross-owner blow (`64b3a0`..`64b513`), when all of:

1. T `is_unit` and not `is_worker` (the worker arm is the other branch);
2. `game->semaphore` bit 17 (`0x822 & 2`) — one of the two bits
   `Game::is_scenario (0043ec70)` tests (the other is bit 12,
   `0x821 & 0x10`); `docs/ECONOMY.md` names bit 17 Conquer the World;
3. T's type id (`T.type(+0x18)->+4`) equals `conquest_game->hero_type`
   (`ConquestGame +0x118`, global `0xc0617c`);
4. `frame - leaders[vic].ctw_hero_stamp > 120` (`0x78`);
5. `vic == console->who`, the **local** player only;
6. `(hits - T.damage(+0x24)) * 100 / hits < 70` (`0x46`), signed `idiv`,
   `hits` from `UnitData::hits` (vslot +0x11c, read direct as `T+0x20`
   when the slot is `UnitData::hits (00610890)`), damage as it stood
   **before** this blow;
7. then (after the message is queued) `frame - stamp > 75` (`0x4b`),
   which item 4 already implies, so it always holds: the store follows.

The message ("your hero is under attack", `S_YOUR_CAPITAL_ATTACKED`
sound) is posted before the store; the store is inside condition 7 with
the `do_notice` call, not inside an interface-only test.

**3. Reset.** `Leader::init` (0), `Leaders::clear`. No per-frame fold.

**4. Readers.** `do_damage`'s two windows (120 and 75 frames);
`LeaderData::log_data` (line 582). The checksum walk covers +0x200. No
other reader in the export or by absolute address.

**5. Crate.** None, and none wanted for v1: Conquer the World is cut
(`CLAUDE.md` "Cut from v1"). `checksum/leaders.rs:72` row, no value; in a
plain game it stays 0, so a constant-0 value arm is exact for every
non-CtW capture. If ever built, it sits in `Sim::do_damage_round`'s unit
path before `take_damage_typed` (`delivery.rs:190`'s neighbourhood).

**6. Plain game.** **No.** Condition 2 is a CtW/scenario semaphore bit and
condition 3 a `ConquestGame` field. A skirmish never sets bit 17, so the
leaf is 0 for its whole length. Note also condition 5: the write depends
on which player is the local console, so even under CtW it is not a
lockstep-symmetric write (CtW is single-player).

**7. Confidence and not read.** High for the store and conditions 1, 3–7
(listing). Condition 2's naming as CtW leans on `docs/ECONOMY.md` and on
the `hero_type` comparison; who sets bit 17 was not traced (no direct
`or`/`mov` to `0x822` was found in the listing; it is likely set through
a `BitMask` helper).

## Summary

All three arrays are indexed `leaders[victim].X[attacker]` (row = struck
owner, column = striker's owner byte `A+0x9`), written with `game->frame`
inside `Object::do_damage (0064a480)`'s cross-owner block, before
`take_damage`, so a killing blow stamps too. Window: 300 frames,
`frame - stamp > 300`.

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `attack_stamp` +0xf4 [8] | `Object::do_damage (0064a480)` ×4 stores (`64ace6`, `64b037`, `64b383`, `64b9d8`); `Leader::init (006e3930)`; `Leaders::clear (006ed5d0)`; `LeaderData::walk_data (006d6750)` on load | `= frame`: capital heavy (gate `capital_stamp[att]`), city heavy (gate city `attack_stamp`), any city light or worker arm (gate city `raid_stamp`), territory attack test (gate this row; T off the victim's land, or on it and not a light building/worker/caravan/merchant) | `Leader::init` 0, `Leaders::clear` 0 | none; `Sim::city_alarm` (`combat/fight/delivery.rs:280`) for arms 1–3, a new territory step after `delivery.rs:190` for arm 4; row beside `AiLeader::capital_stamp` (`ai.rs:704`) | yes | high |
| `raid_stamp` +0x114 [8] | `Object::do_damage` ×4 (`64acf8`, `64b025`, `64b373`, `64b755`); `Leader::init`; `Leaders::clear`; `walk_data` | `= frame`: same arms 1–3 as `attack_stamp`; territory raid test (gate this row; T on the victim's land and a light/inactive building, worker, caravan or merchant) | as above | as above | yes | high |
| `ally_stamp` +0x154 [8] | `Object::do_damage` ×1 (`64ae8d`); `Leader::init`; `Leaders::clear`; `walk_data` | `= frame` on a heavy hit on a city building (capital or not) when `frame - ally_stamp[att] > 300`, every player pair; the console/ally test gates only the message | as above | none; both heavy arms of `Sim::city_alarm` (the capital arm's early return must move) | yes | high |
| `ctw_hero_stamp` +0x200 | `Object::do_damage` ×1 (`64b513`); `Leader::init` (0); `Leaders::clear`; `walk_data` | `= frame` when CtW (semaphore bit 17), T is the conquest hero type, not a worker, victim is the local console, `> 120` frames since, hero below 70 % hits before the blow | `Leader::init` 0, `Leaders::clear` 0 | none; CtW is cut; 0 is exact in skirmish | **no** (CtW only) | high (bit 17's name medium) |

**Against the README row:** `attack_stamp`'s extra writers
`Leader::plan_strategy (006b9620)` and `Leader::end_process (006b8690)`
are **not** writers: no store to +0xf4 of a leader record in either, by
both nets. The row also omits `Leader::init` and `Leaders::clear` (resets)
and `LeaderData::walk_data` (load). One shape the row does not show: a
**light** hit on a city building writes the victim's `attack_stamp` too
(arm 3), and every cross-owner blow, city or not, reaches the territory
test.
