# Item 4152, second (blind) reading: `kills`

- Slug: `kills`
- Leaves: `units_killed` (LeaderData +0x80c, int), `units_lost` (+0x810, int),
  `buildings_razed` (+0x818, int), `buildings_lost` (+0x81c, int)
- Reader: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp`), the PE listing
  (`aa/exe.s`, llvm-objdump of `.text`), `crates/sim`. Not opened: the first
  reading, the README, CHECKSUM §20, PARKED, LEADERS, the 41xx journals.

## 0. The nets, shared by all four leaves

**Base.** `Leader` (0x6eec) has no field list of its own in `types.txt`;
`leaders` is `S_GDATA32` at section 3 + 0x234390 = `0xe3a390`, and the
decompiler's `leaders.list[i].field_0xNNN` resolves to the absolute address
`0xe3a390 + i*0x6eec + 0xNNN` in every case checked (e.g. `Build::init`'s
`field_0x814` is `incl 0xe3aba4(%esi)` at `006297b4`, `buildings_built`). So
`Leader` = `LeaderData` at +0 with 8 trailing bytes, and a `Leader*`
displacement equals the `LeaderData` offset; no shift.

**Export net.** `->units_killed|units_lost|buildings_razed|buildings_lost`:
one hit, `LeaderData::log_data (006e5110)` (reader). `field_0x80c|810|818|81c`
on a leader: `Leader::reset_score (006e37f0)` (clear), `Object::take_damage
(00652020)`, `Object::do_damage (0064a480)`, `SpellType::cast_sabotage
(006720e0)`, `SpellType::cast_counterintel (00673740)` (writers);
`Game::determine_end_of_game_stats_and_achievements (00583b30)`,
`AchieveChartPane::set_chart_data (007f2c40)` (readers). Other
`field_0x81c` hits are other records (`MiscAccess::terrain`).

**Listing net** (`aa/exe.s`). Absolute `0xe3ab9c|0xe3aba0|0xe3aba8|0xe3abac`:
11 sites, 9 `incl`, 2 `movl`-loads (ScenarioFuncSet readers). No `leal` of
them, so no pointer-taken write. Register-relative stores to
`0x80c|0x810|0x818|0x81c(%reg)`: 133 sites mapped to functions through
`INDEX.tsv`. Only `Leader::reset_score` is a leader. The rest: a game-state
word at `+0x81c` set to 0/1/2 by `Game::run*`, `CommandManager`,
`Player::quit` etc. (not `LeaderData`); `Constants::init`, `NetDaemon`,
`TileSet::init`, `GraphicPieces`, `RTreeData`, `TerrainMap::on_redraw`
(other records); and `incl 0x80c(%edx,%eax,4)` in `Object::do_damage` at
`0064a558`, off the global pointer at `0xc061ec` (a per-player stats array
gated on that object's `+0x821 & 2`), **not** the leaders array.

**Bulk writes.** `memset(leader, 0, 0x6929)` in `Leader::init (006e3930)`
(only when its 2nd argument < 0), `Leader::close (006b7f80)`, and
`Leaders::clear (006ed5d0)` (every slot). `LeaderData::walk_data (006d6750)`
walks the raw byte range `[&who, &field_0x692a)` through the `DataWalk`,
which covers `+0x80c..+0x81c`: a load (`LoadGame`, `DataWalk::input != 0`)
writes all four; `verify_save`/`verify_load` read them.

**The writers' events, in one place.**

| site | function | leaf | leader index |
|---|---|---|---|
| `00652f5b` | `Object::take_damage` | buildings_lost +1 | victim (`this->who`) |
| `00653007` | `Object::take_damage` | units_lost +1 | victim (`this->who`) |
| `00653046` | `Object::take_damage` | units_killed +1 | `param_8` (attacker's who, `ebx`) |
| `0064ba49` | `Object::do_damage` | buildings_razed +1 | attacker (`this->who`) |
| `0067222f` | `SpellType::cast_sabotage` | units_killed +1 | caster's player (`param_2`) |
| `0067277d` | `SpellType::cast_sabotage` | buildings_razed +1 | caster's player (`param_2`) |
| `00673867` | `SpellType::cast_counterintel` | units_killed +1 | caster's player (`ecx`) |

Every step is `+1` on an `int`; nothing ever decrements any of the four
(no `decl`, `subl`, or non-zero `movl` to them anywhere).

**`take_damage`'s parameters** (`retl $0x24`, nine stack words; `ebp+8`..):
`param_1` whole hits, `param_2` sixteenths, `param_3` dtype (forwarded to
`die`), `param_4` (an object index, for the death angle), `param_5` the
attrition/no-attacker flag, `param_6`, `param_7` the attacker's object
index, `param_8` the attacker's player, `param_9`. `ebx` is loaded with
`param_8` at `00652065` and restored to it at `00652bbd`, `00652cd3`,
`00652d60` after each local reuse; every jump into the region past
`00652d60` (from `00652aba`..`00652b10`, `00652bea`, `00652d00`) comes from a
point where `ebx` already holds `param_8`. So the `units_killed` store at
`00653046` (`imul $0x6eec,%ebx`) indexes the attacker's player.

**The death gate.** `take_damage` reaches the counters only past
`if (damage(+0x24) < local_c) { … return 0 or 1 }`, i.e. when the
accumulated whole hits reach the object's hits (vslot `+0x11c`; for a unit
of `UBER_SIZE` n > 1, divided to the figure's share unless it is the lone
remaining captain). Above it, a city building (`flags & 0x20` and vslot
`+0x4c`) clamps `damage = hits` and returns 0: **a city never reaches the
counters through damage.** The counters sit after the death message/sound
block and before `die` (vslot `+0x158`, at `006533cd`).

## 1. `units_killed` (LeaderData +0x80c, int)

### 1.1 Writers

1. **`Object::take_damage (00652020)`, `incl 0xe3ab9c(%eax)` at `00653046`.**
   On the death branch, for a dead object that is **not** a building
   (vslot `+0x1c` answers 0; `Build`'s slot is `return 1`, `Unit`'s
   `return 0`), and only after `units_lost` has stepped (§2.1: captain,
   `curr_uber_size(0) == 1`, not a decoy), **and** `param_7 >= 0`
   (`cmpl $0, 0x20(%ebp); jl` at `0065300d`). Then, in order: victim's
   `+0xa58` (short) +1, `leaders[param_8]+0xa5a` (short) +1,
   `leaders[param_8].units_killed` +1. There is **no** test of `param_8`
   against the victim's player, no ally test, and no `param_8 >= 0` test.
   The ally test that follows (`LeaderData::is_ally`) only picks the sign of
   the `+0x29d8`-relative score write (`score_combat` of the attacker
   ± type value / 5); it does not gate the kill.
2. **`SpellType::cast_sabotage (006720e0)`, `incl 0xe3ab9c(%esi)` at
   `0067222f`.** Arguments (from `SpellType::cast (00676ce0)`, cases `0x280`
   and `0x281`): `param_1` caster index, `param_2` caster's player,
   `param_3` target index, `param_4` target's player. If the target's
   `flags & 1` is clear, a mana refund path and return. If the target is not
   a building (vslot `+0x1c` = 0) and is a unit (vslot `+0x18`), a captain
   (vslot `+0xe8`, inlined as `o_up < 0`), `curr_uber_size(0) == 1`, and
   `unit_masks & 1` clear: target's `+0xa58` +1, caster's `+0xa5a` +1,
   **caster's `units_killed` +1**, caster's `score_combat` += value/5. Then
   the target dies outright through vslot `+0x158` (`die(3, -1, 0)` if it
   `has_objmask(0x200000)`, else `die(2, -1, 0)`), **not** through
   `take_damage`, so the victim's `units_lost` does **not** step.
3. **`SpellType::cast_counterintel (00673740)`, `incl 0xe3ab9c(%edi)` at
   `00673867`.** A non-standard frame: `retl $0x8`, `ecx` = the caster's
   player (the caller at `00676d8c` loads `ecx` from `edi`, the cast's
   player), stack = target index, target's player. A non-captain target is
   replaced by its captain (vslot `+0xe4`). Gate: target's player ≠ caster,
   ≠ `leaders[caster].who` (+0x8), and not (`leaders[caster].diplos[target]
   == 2` and `leaders[target].diplos[leaders[caster].who] == 2`). Inside it:
   a unit, captain, `curr_uber_size(0) == 1`, decoy bit clear →
   **caster's `units_killed` +1** and `score_combat` += value/5; then
   `die(2, -1, 0)` whatever the inner test said. Again no `units_lost` for
   the victim. Spell id `0x277`, gated in `cast` by `is_valid_target`.

### 1.2 Event, step, predicate

+1 per qualifying unit death credited to a player:
- combat or any other `take_damage` death that carries an attacker index
  (`param_7 >= 0`), to `param_8`;
- a Commando/Special-Forces sabotage or sniper kill of a unit (cast ids
  `0x280`/`0x281`), to the caster's player;
- a counter-intelligence kill (cast `0x277`), to the caster's player.

The unit predicate is the same in all three: **squad head** (`o_up < 0`),
**alone** (`curr_uber_size(0) == 1`: no live `o_down` follower with
`flags & 1`), **not a decoy** (`unit_masks & 1` clear). A figure of a
multi-figure squad that dies while others live does not count; the squad
counts once, when its last figure (the captain) dies. Exempt kinds:
buildings (their own counters), decoys, non-final figures.

**A self-credit case.** A decoy (`unit_masks & 1`) that dies with whole
points past its share passes the overflow to its captain:
`take_damage(overflow, 0, dtype, param_4, param_5, param_6, captain_index,
victim's own player, 0)` at `00653407`..`00653426` (only when
`take_damage`'s local `uVar10` = the dead figure's `unit_masks & 1` is set,
`testl %esi,%esi` at `006533d3`). If that kills the captain and it
qualifies, `units_killed` steps for **the victim's own player**, since
`param_7 >= 0` and `param_8 = victim`. No exemption stops it.

### 1.3 Reset

`Leader::reset_score (006e37f0)` stores 0 (`006e3829`). Callers:
`Leader::init (006e3930)` (at `006e3b1a`, unconditionally after the setup)
and `Setup::build_game (005ac190)` (at `005ad1e7`, every slot with
`leader_flags & 1`, after `NetDaemon::process_all`). Also zeroed by the
`memset(…, 0, 0x6929)` in `Leader::init` (2nd arg < 0), `Leader::close`
and `Leaders::clear`; restored by `LeaderData::walk_data` on a load.

### 1.4 Readers

`LeaderData::log_data (006e5110)` (the leaders dump, line ~1456);
`ScenarioFuncSet::num_units_killed (009e98e0)` (`009e98ff`, trigger
condition); `Game::determine_end_of_game_stats_and_achievements
(00583b30)` (`00583ef8`, Steam stat `gStatID[4]`);
`AchieveChartPane::set_chart_data (007f2c40)` (post-game chart). No sim
decision reads it.

### 1.5 Crate site

The crate carries no `units_killed`; `crates/sim/src/checksum/leaders.rs:157`
lists the offset for the checksum. Builders would add it at:
- `Sim::take_damage_typed`, `crates/sim/src/combat/fight/delivery.rs:469`,
  in the `Taken::Died` arm (`:537`), before `close_count` (the original
  steps before `die`), crediting `self.owner_of(_by)` when `_by` is an
  object (the original's `param_7 >= 0`). The squad test needs the
  pre-death `takes_remainder_share(i)` (`units/store.rs:667`, already
  computed as `alone` at `:515`) and `!decoy` (read at `:530`). The
  decoy-overflow recursion at `:586` passes `Obj::Unit(k)` as its own
  attacker, so the self-credit case falls out if the credit reads `_by`.
- `Sim::cast_counterintel`, `crates/sim/src/spells/cast.rs:1031`; its doc
  comment at `:1028` already names the `units_killed` SEAM.
- `Sim::cast_sniper_unit`, `crates/sim/src/combat/fight/delivery.rs:806`,
  for the `0x280`/`0x281` unit arm (called from `spells/cast.rs:982`,
  `:999`); it would need the caster's player, which its signature does
  not carry today.

### 1.6 Plain game?

Yes for all three. Combat deaths are every game; Commandos/Special Forces
and Spies (counter-intelligence) are buildable in a skirmish against the
computer. `ScenarioFuncSet::set_object_health (009f5aa0)` and
`ScenarioEditor::dec_selected_health (0099ff60)` call `take_damage` with
`param_7 = -1`, so they cannot step this leaf.

### 1.7 Confidence

High on the writer set (both nets agree, 3 sites) and the predicates
(listing-read at each store). Medium on the self-credit case: the
argument order of the recursive call is read from the pushes, but whether
the captain then passes `curr_uber_size(0) == 1` depends on `die` having
cleared the decoy's `flags & 1`, which I did not trace through
`Object::close`. Not read: `Ammo::do_damage`/`Nuke::do_damage` (they reach
`take_damage` through `Object::do_damage` or not; which `this` they pass is
unverified), and whether friendly fire can reach `take_damage` with
`param_8` = the victim's own player outside the decoy case.

## 2. `units_lost` (LeaderData +0x810, int)

### 2.1 Writers

One: **`Object::take_damage (00652020)`, `incl 0xe3aba0(%eax)` at
`00653007`**, indexed by the dead object's own player (`movzbl 0x9(%edi)`,
`SubObjectData::who`). The path, from the listing `00652f46`..`00653007`:

1. on the death branch (§0), vslot `+0x1c` of the dead object answers 0
   (not a building; a building takes `buildings_lost` at `00652f5b` and
   jumps to the tail at `006533b2`);
2. vslot `+0xe8` (`UnitData::is_captain (0046ceb0)`, `o_up < 0`; inlined
   when the slot equals `0x46ceb0`, else called) answers non-zero;
3. `UnitData::curr_uber_size (0060a760)` with argument 0 returns 1: from
   the squad head, no `o_down` follower whose `flags & 1` (live) is set;
4. `unit_masks (+0x68) & 1` clear (not a decoy).

Then `units_lost` +1, **whether or not there is an attacker** (the
`param_7 >= 0` test at `0065300d` comes after and gates only the
`units_killed` / `+0xa58` / `+0xa5a` steps). `param_5` (attrition) and
`param_3` (dtype) do not enter.

No other writer: neither cast path (§1.1 items 2, 3) steps it, nor does
`die` (vslot `+0x158`) or `Object::disband (006455c0)`; the listing has no
store to `+0x810` of a leader outside `take_damage` and `reset_score`.

### 2.2 Event, step, predicate

+1 for the victim's player when a squad's last live figure (its head, not
a decoy) dies **through `take_damage`**. Callers of `take_damage` (vslot
`+0x16c`, grepped): `Object::do_damage (0064a480)` (combat),
`Unit::suffer_attrition (005e1a10)` (attrition: `param_3 = 1`,
`param_5 = 1`, `param_7 = param_8 = -1`), `take_damage` itself (decoy
overflow onto the captain), `SpellType::cast_sabotage (006720e0)` (building
targets only), `Wall::process (00640450)` (buildings),
`ScenarioFuncSet::set_object_health (009f5aa0)`,
`ScenarioFuncSet::damage_object (009f5bd0)`,
`ScenarioEditor::dec_selected_health (0099ff60)`. The listing's
`calll *0x16c(%reg)` sites, mapped to functions, are exactly these nine
plus `Window::on_mouse_wheel` (a `Window` vtable, not an object); no direct
`call 0x652020` exists. The three trigger/editor callers pass
`param_5 = 1`, `param_7 = param_8 = -1`.

So **attrition deaths count as losses** (with no killer), and **sabotage,
sniper and counter-intelligence kills do not** (they call `die` directly).
Deletion, conversion, capture-by-bribe, upgrade replacement and other
`die`/`close` paths that skip `take_damage` do not count (not each read; the
listing shows no other writer, which is the claim).

Asymmetry worth a test: summed over players, `units_lost` ≠
`units_killed`. Losses include attrition and trigger deaths; kills include
spell kills that have no loss; and the decoy-overflow case credits the
victim's own player with the kill.

### 2.3 Reset

As §1.3: `Leader::reset_score (006e37f0)` (`006e3823`), the three
`memset`s, and `LeaderData::walk_data` on a load.

### 2.4 Readers

`LeaderData::log_data (006e5110)` (~1469); `ScenarioFuncSet::num_units_lost
(009e98b0)` (`009e98cf`); `Game::determine_end_of_game_stats_and_achievements
(00583b30)` (`00583fbb`, `gStatID[7]`); `AchieveChartPane::set_chart_data
(007f2c40)`. No sim decision.

### 2.5 Crate site

No counter in the crate (`checksum/leaders.rs:158` lists the offset).
Builders would add it at:
- `Sim::take_damage_typed`, `crates/sim/src/combat/fight/delivery.rs:469`,
  `Taken::Died` arm (`:537`), before `close_count`, for the struck unit's
  owner, gated on the pre-take `alone` (`takes_remainder_share`, `:515`)
  and `!decoy` (`:530`). No attacker test.
- `Sim::die_by_attrition`, `delivery.rs:844`, called from
  `crates/sim/src/territory/upkeep.rs:307`. The crate's attrition kill
  does not go through `take_damage_typed`; the original's
  `suffer_attrition` does, so the same gate belongs here.
- Not at `cast_sniper_unit` (`:806`) or `cast_counterintel`
  (`spells/cast.rs:1031`): the original does not step it there.

### 2.6 Plain game?

Yes: combat deaths and attrition deaths both occur in a skirmish against
the computer. The trigger and editor callers are outside a plain game.

### 2.7 Confidence

High: one writer, read in the listing, with the predicate's four tests
each at a named instruction. Not read: whether any non-`take_damage` death
path (e.g. a boat's `kill_contents`, a nuke) routes through `take_damage`
for each passenger. Only the `+0x16c` callers listed above were grepped,
and the listing's `calll *0x16c(%reg)` sweep (§2.2) closes the indirect
case for that slot.

## 3. `buildings_razed` (LeaderData +0x818, int)

### 3.1 Writers

1. **`Object::do_damage (0064a480)`, `incl 0xe3aba8(%eax)` at `0064ba49`**,
   indexed by the **attacker's** player (`this` is the damaging object;
   `movzbl 0x9(%edi)`). The call before it (`0064ba18`, vslot `+0x16c` on
   the target) is `take_damage(whole, sixteenths, dtype, param_5, 0,
   param_3, this->o, this->who, param_7)`. Gate: `take_damage` returned
   non-zero (`je 0x64bbfd` at `0064ba20`) **and** the target's vslot
   `+0x1c` answers non-zero (a building; `je 0x64c86b` at `0064ba39`, which
   leaves for a unit kill). There is no owner or ally test before the step;
   the `is_ally` test after it only picks the sign of the attacker's
   `score_combat` (+0x44) change (± type value / 5, floored at 0 on the
   minus side). The Steam stat `gStatID[6]` / `STAT_BUILDINGS_DESTROYED`
   follows for the console player.
2. **`SpellType::cast_sabotage (006720e0)`, `incl 0xe3aba8(%esi)` at
   `0067277d`**, indexed by the caster's player (`param_2`). Reached only
   for a building target (vslot `+0x1c` non-zero; the unit arm returns
   earlier). Damage = max(target's vslot `+0x114` value / 2, a floor:
   `elite_min_damage` if the spell answers `is(0x4d)`, else
   `special_min_damage` if `is(0x4b)`, else `commando_min_damage`), dealt
   by `take_damage(dmg, 0, 3, -1, 1, 0x55555555, -1, -1, 0)` (no attacker,
   attrition flag set). Non-zero return → caster's `buildings_razed` +1 and
   `score_combat` += value/5 (no ally branch here).

### 3.2 Event, step, predicate

+1 for the attacker's (or caster's) player when its blow or sabotage makes
`take_damage` return non-zero on a building. `take_damage` returns non-zero
on a building in two places:

- **death** (accumulated whole hits ≥ hits): through the tail, `die`, then
  `return 1` (the dead object's decoy bit is 0 for a building, so
  `testl %esi,%esi` at `006533d3` takes the `return 1` at `00652aa7`);
- **the site-abandon arm, while alive** (`00652a1c`..`00652aa9`): a
  building (vslot `+0x1c`) whose vslot `+0x4c` answers 0 (for `Build` the
  folded `WallData::is_active (00472350)`, `flags & 4`: not yet active, a
  construction site), owned by a non-human leader (`leader_flags & 4`
  clear), with `2 * damage >= hits` (vslot `+0x11c(0)`), and
  `build_masks (+0x60) & 0x2000` set → `Object::disband(this, 0)`, return 1.
  This credits the attacker with a razed building **without** the victim's
  `buildings_lost` stepping (§4).

Exempt: a city building (clamped, returns 0; it is captured, not razed);
unit kills. No exemption for one's own or an ally's building: if the blow
lands, it counts.

### 3.3 Reset

`Leader::reset_score (006e37f0)` (`006e383b`), the three `memset`s, and
`LeaderData::walk_data` on a load (§0).

### 3.4 Readers

`LeaderData::log_data (006e5110)` (~1482); `AchieveChartPane::set_chart_data
(007f2c40)`. No trigger reader found (the `ScenarioFuncSet` hits are the
two unit counters only), and no sim decision.

### 3.5 Crate site

No counter in the crate (`checksum/leaders.rs:160`). Builders would add it
at:
- `Sim::do_damage_round`, `crates/sim/src/combat/fight/delivery.rs:70`,
  just after `take_damage_typed` (`:192`), in the `Obj::Building(b)` +
  `killed` arm (`:212`..`:221`), **before and outside** the
  plunder's air/splash/ally gate, for `self.owner_of(attacker)`. The crate's
  `Taken::Died` is the death case only.
- `Sim::cast_sabotage_building`, `delivery.rs:884` (from
  `spells/cast.rs:1000`), on a `Taken::Died` from `damage_building`; it
  would need the caster's player.
- The site-abandon arm is **not in the crate**: `damage_building`
  (`delivery.rs:957`) has no `build_masks & 0x2000` disband, and
  `crates/sim/src/ai/ai_make.rs:936` is a SEAM saying the crate does not
  keep `build_masks |= 0x2000` at all. That arm needs both before its
  razed credit can exist.

### 3.6 Plain game?

Yes: buildings are razed in any skirmish, and Commandos' sabotage is
available in the late ages. The site-abandon arm needs a computer
player's site with `build_masks & 0x2000` ("no builder could be found",
per the SEAM's comment) struck to half its hits, which a skirmish can
reach.

### 3.7 Confidence

High on the two writers and their indices (listing). Medium-high on the
site-abandon arm (read in the listing, `00652a1c`..`00652aa9`). The meaning
of `flags & 4` as "active" rests on the folded slot name and the crate's
`active` flag (`cities/city.rs:3110`), not on a type record. Not read: the
vslot `+0x114` / `+0xac` pair in sabotage's damage (it does not affect the
counter), and whether `0x4b`/`0x4d` are the two Commando types.

## 4. `buildings_lost` (LeaderData +0x81c, int)

### 4.1 Writers

One: **`Object::take_damage (00652020)`, `incl 0xe3abac(%eax)` at
`00652f5b`**, indexed by the dead building's own player
(`movzbl 0x9(%edi)`). On the death branch (§0), the first test after the
caravan-danger block is vslot `+0x1c` (`00652f4a`); non-zero → this step,
then `ScenarioData::last_razed[who] = this->o` (`movl %ecx,
0xcc2190(,%eax,4)` at `00652f69`), then a jump to the tail (`006533b2`:
`die`, return 1). No attacker test (`param_7`/`param_8` do not enter) and
no `param_5` test.

The `+0x81c` register-relative stores elsewhere (the 0/1/2 game-state
word in `Game::run*`, `CommandManager::*`, `Player::quit`,
`ScenarioFuncSet::field_battle_*` and so on, §0) are not `LeaderData`.

### 4.2 Event, step, predicate

+1 for the owner when a non-city building is destroyed **through
`take_damage`**, whatever did it: combat (`Object::do_damage`), building
attrition in enemy territory (`Wall::process (00640450)`: 8 whole hits a
call with `param_5 = 1`, for a building on a tile whose owner is another,
non-allied player, once it has started; an unstarted site there is
disbanded instead), a Commando sabotage (§3.1 item 2), or the trigger and
editor callers (§2.2).

Not counted: a city (clamped, captured instead, see the `cities_*`
leaves); a site abandoned by the take_damage disband arm (§3.2, which
still credits the attacker's `buildings_razed`); `Wall::process`'s disband
of an unstarted site; a player's own delete/disband (`Object::disband
(006455c0)` writes none of these leaves). So summed over players
`buildings_lost` ≠ `buildings_razed`: losses include attrition and trigger
deaths, a sabotage kill counts on both sides, and site-abandons appear
only as razed.

### 4.3 Reset

`Leader::reset_score (006e37f0)` (`006e3835`), the three `memset`s, and
`LeaderData::walk_data` on a load (§0).

### 4.4 Readers

`LeaderData::log_data (006e5110)` (~1495); `AchieveChartPane::set_chart_data
(007f2c40)`. No sim decision. (`ScenarioData::last_razed`, written beside
it, is the trigger layer's.)

### 4.5 Crate site

No counter in the crate (`checksum/leaders.rs:161`). The single site is
`Sim::damage_building`, `crates/sim/src/combat/fight/delivery.rs:957`, in
the `Taken::Died` arm at `:1041`, before `self.die_building(b)`, for
`self.buildings[b].owner`. That one function already serves all three
plain-game callers: combat (`take_damage_typed`, `:599`), sabotage
(`cast_sabotage_building`, `:887`), and building attrition
(`Sim::building_attrition`, `crates/sim/src/cities/city.rs:3123`, `:3128`).
The city clamp at `:1025` returns before it, as in the original.

### 4.6 Plain game?

Yes: combat and border attrition of buildings both occur in a skirmish.
`ScenarioFuncSet::set_object_health`/`damage_object` and the editor are
outside it.

### 4.7 Confidence

High: one writer, read in the listing, and the predicate is a single
vslot test on the death branch. Not read: whether `Build` (as opposed to
`Wall`) reaches `Wall::process`'s territory arm (the crate's
`building_attrition` says it does; I did not check `Build`'s vtable slot
for `process`).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `units_killed` +0x80c | `Object::take_damage (00652020)` at `00653046`; `SpellType::cast_sabotage (006720e0)` at `0067222f`; `SpellType::cast_counterintel (00673740)` at `00673867` | +1 to the attacker's / caster's player when a lone non-decoy squad head dies: through `take_damage` with `param_7 >= 0` (no owner/ally exemption; decoy overflow credits the victim's own player), or by a sabotage/sniper or counter-intel kill | `Leader::reset_score (006e37f0)` from `Leader::init (006e3930)`, `Setup::build_game (005ac190)`; `memset 0x6929` in `Leader::init`, `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`; restored by `LeaderData::walk_data (006d6750)` | `take_damage_typed` (`combat/fight/delivery.rs:469`, Died arm `:537`); `cast_counterintel` (`spells/cast.rs:1031`, SEAM at `:1028`); `cast_sniper_unit` (`delivery.rs:806`) | yes | high (self-credit case medium) |
| `units_lost` +0x810 | `Object::take_damage (00652020)` at `00653007` | +1 to the victim's player when a lone non-decoy squad head dies through `take_damage`, attacker or not (attrition counts; spell kills do not) | as above | `take_damage_typed` Died arm (`delivery.rs:537`); `die_by_attrition` (`delivery.rs:844`) | yes | high |
| `buildings_razed` +0x818 | `Object::do_damage (0064a480)` at `0064ba49`; `SpellType::cast_sabotage (006720e0)` at `0067277d` | +1 to the attacker's / caster's player when `take_damage` returns non-zero on a building: its death, or the computer site-abandon arm (not active, `2*damage >= hits`, `build_masks & 0x2000` → disband). No owner/ally exemption; cities never | as above | `do_damage_round` after `:192`, building+killed arm, outside the plunder gate; `cast_sabotage_building` (`delivery.rs:884`); site-abandon arm absent (SEAM `ai/ai_make.rs:936`) | yes | high (abandon arm medium-high) |
| `buildings_lost` +0x81c | `Object::take_damage (00652020)` at `00652f5b` | +1 to the owner when a non-city building dies through `take_damage` (combat, border attrition, sabotage, triggers) | as above | `damage_building` Died arm (`delivery.rs:1041`) | yes | high |

Note on process: a grep over `docs/*.md` for `build_masks`/`leader_flags`
surfaced four unrelated lines of `docs/PARKED.md` (mask-bit coverage rows)
in its output; no kills content was among them, and no forbidden file was
opened.
