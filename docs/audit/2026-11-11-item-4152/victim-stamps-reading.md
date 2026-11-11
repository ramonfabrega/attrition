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
hit after frame 300 always passes the window; a hit before frame 301
does not (`frame - 0 > 300` fails), so in the first 300 frames this row
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

