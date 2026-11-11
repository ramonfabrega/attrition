# Item 4152, family 2 (`kills`) — first reading

- Leaves: `units_killed` (LeaderData +0x80c), `units_lost` (+0x810),
  `buildings_razed` (+0x818), `buildings_lost` (+0x81c), all `int`.
- Reader: Claude Opus 5.5 (claude-opus-5-5), wave 1 first reading, 2026-11-11.
- Sources: the decompile export `~/ghidra-projects/decomp` and the listing of
  `riseofnations.exe` (`llvm-objdump`). Written per leaf as each finishes.

## 0. The nets, run once for all four leaves

**Base.** `Leader` (0x6eec) holds `LeaderData` (0x6ee4) at offset 0: the
extra 8 bytes are at the end. `Leader::reset_score (006e37f0)` stores
`this->field_0x18` (`score`, LeaderData +0x18) and `Leaders::clear
(006ed5d0)` memsets from `0xe40fa4 - 0x6c14 = 0xe3a390`, the slot-0
`LeaderData`. So `leaders.list[i].field_0xNNN` and `0xNNN(%Leader*)` are
LeaderData offsets with no shift. The four leaves' slot-0 addresses:
`units_killed` 0xe3ab9c, `units_lost` 0xe3aba0, `buildings_razed` 0xe3aba8,
`buildings_lost` 0xe3abac.

**Export net.** `grep '->units_killed\b'` (and the three others) finds only
readers (`LeaderData::log_data (006e5110)`; the `ScenarioData::units_killed`
hits are a different array, the scenario's per-type kill table). `grep
'field_0x80c\b|0x810|0x818|0x81c'` finds, as LeaderData: `Leader::reset_score
(006e37f0)`, `Object::take_damage (00652020)`, `Object::do_damage
(0064a480)`, `SpellType::cast_sabotage (006720e0)`, `SpellType::cast_counterintel
(00673740)` (writers), and `Game::determine_end_of_game_stats_and_achievements
(00583b30)`, `AchieveChartPane::set_chart_data (007f2c40)` (readers).
`CoordInfo::add_geometry`'s `terrain->field_0x81c` is `TerrainData`, not ours.
**The decompile hides one writer**: `take_damage`'s `units_killed` store prints
as `local_8[0x20d].field_0x31a0` with `local_8 = param_8 * 0x6eec`;
`0x20d·0x6eec + 0x31a0 = 0xe3ab9c`. The listing shows it plainly (below).

**Listing net, absolute.** Every instruction in `.text` naming 0xe3ab9c,
0xe3aba0, 0xe3aba8 or 0xe3abac — 11 hits:

| address | instruction | function | kind |
|---|---|---|---|
| 0x583ef8 | `pushl 0xe3ab9c(%ebx)` | `Game::determine_end_of_game_stats_and_achievements (00583b30)` | read |
| 0x583fbb | `pushl 0xe3aba0(%ebx)` | same | read |
| 0x64ba49 | `incl 0xe3aba8(%eax)` | `Object::do_damage (0064a480)` | **write** razed |
| 0x652f5b | `incl 0xe3abac(%eax)` | `Object::take_damage (00652020)` | **write** b. lost |
| 0x653007 | `incl 0xe3aba0(%eax)` | `Object::take_damage` | **write** u. lost |
| 0x653046 | `incl 0xe3ab9c(%eax)` | `Object::take_damage` | **write** u. killed |
| 0x67222f | `incl 0xe3ab9c(%esi)` | `SpellType::cast_sabotage (006720e0)` | **write** u. killed |
| 0x67277d | `incl 0xe3aba8(%esi)` | `SpellType::cast_sabotage` | **write** razed |
| 0x673867 | `incl 0xe3ab9c(%edi)` | `SpellType::cast_counterintel (00673740)` | **write** u. killed |
| 0x9e98cf | `movl 0xe3aba0(%eax),%eax` | `ScenarioFuncSet::num_units_lost (009e98b0)` | read |
| 0x9e98ff | `movl 0xe3ab9c(%eax),%eax` | `ScenarioFuncSet::num_units_killed (009e98e0)` | read |

Slot-biased forms (`base ± k·0x6eec`, k = 1, 2) of all four: zero hits.

**Listing net, register-relative.** Every store (`mov/inc/dec/add/sub/or/and/
xor` with the operand as destination) to `0x80c|0x810|0x818|0x81c(%reg)` —
138 hits in 60 functions. Attributed by function and base register:

- `Leader::reset_score (006e37f0)`, 4 stores off `ecx` = `Leader*`: the only
  register-relative LeaderData writers. Zeroes all four.
- `0x81c(%reg)` with `$0`/`$1`/`$2`/`%reg` in ~50 `Game::*`, `CommandManager::*`,
  `CommandPackage::*`, `Player::*`, `ScenarioEditor::*`, `ScenarioFuncSet::
  field_battle_*` …: the base is `GameAccess::game` (`0xc061ec`) and the field is
  `Game::semaphore.flags` (`field_battle_enable (009e66c0)` prints it so).
  Not LeaderData.
- `Object::do_damage (0064a480)` at 0x64a558 `incl 0x80c(%edx,%eax,4)`: `edx`
  = `GameAccess::game`, an int array of `Game` indexed by the attacker's player
  (with `addl %ecx, 0x804(%edx,%eax,4)` beside it), gated on
  `semaphore.ptr[1] & 2` (byte +0x821) and the shooter's player differing
  from `param_2`. Not LeaderData.
- `NetDaemon`, `Constants::init/log_data`, `TileSet::init`, `GraphicPieces`,
  `RTreeData`, `TerrainMap::on_redraw` (byte store), `hpPrintf*` (ebp-negative
  locals): other records.

**Wholesale clears.** `Leaders::clear (006ed5d0)` memsets [+0, +0x6929) of
every slot (called from `Game::close (00594b80)` and `Game::run_oos_recovery
(005869e0)`); `Leader::init (006e3930)` memsets [+0, +0x6929) when its tribe
argument is negative (an empty slot) and calls `reset_score` on every path;
`Setup::build_game (005ac190)` calls `reset_score` on every slot whose
`leader_flags & 1` is set, just after the `setup.cpp` checksum say (0x5ad1e7).

**Callers of `take_damage`.** No direct `call 0x652020`; all through vslot
`+0x16c`, 8 sites (listing): `Unit::suffer_attrition (005e1a10)`,
`Wall::process (00640450)`, `Object::do_damage (0064a480)`,
`Object::take_damage` itself (the decoy-overflow strike on the captain),
`SpellType::cast_sabotage (006720e0)` (building arm),
`ScenarioEditor::dec_selected_health (0099ff60)`,
`ScenarioFuncSet::set_object_health (009f5aa0)`,
`ScenarioFuncSet::damage_object (009f5bd0)` (`Window::on_mouse_wheel`'s slot
`+0x16c` is a `Window` vtable). Its `param_7`/`param_8` are the attacker's
object index and player: `do_damage` passes its own `this` (the shooter), the
recursion passes the captain's index and its owner, and every other caller
passes `-1, -1`.

**`take_damage`'s return.** 0 = alive (or a city clamped at full damage);
**1** = disbanded at half health (the `+0x2000` "AI gave up on this site"
building, decompile lines 352-360); 1 or 2 = died through vslot `+0x158`
(`die`). `do_damage` and `cast_sabotage` test only non-zero.

**Object vslots used below.** `+0x18` answers 1 on a unit (AnimalData's slot
is `Buffer::is_pending_load (0041e0e0)`, `mov eax,1; ret`), `+0x1c` answers 1
on a building (`BuildData`'s and `Wall`'s slot is the same `return 1`). The
crate names them `is_unit`/`is_build` (`take_damage_view.rs:75`). The names
are by the vtables' contents, not by `LF_ONEMETHOD`; not checked against the
PDB here. `+0xe8` is `UnitData::is_captain (0046ceb0)` (inlined:
`unit_masks` +0x8e bit 15), `+0xe4` `get_captain`, `0x60a760`
`UnitData::curr_uber_size`.

## 1. `units_killed` (+0x80c, int)

### Writers

1. **`Object::take_damage (00652020)`, store 0x653046** (decompile line ~509,
   printed as `local_8[0x20d].field_0x31a0`). Index register `eax =
   ebx·0x6eec`, `ebx = param_8` (loaded at 0x652d60, `movl 0x24(%ebp),%ebx`,
   not changed before the store).
2. **`SpellType::cast_sabotage (006720e0)`, store 0x67222f** (decompile line
   102). Index `esi = param_2·0x6eec` (0x672210, `imull $0x6eec,0xc(%ebp)`):
   the **caster's** player.
3. **`SpellType::cast_counterintel (00673740)`, store 0x673867** (decompile
   line 66). Index `edi = ecx·0x6eec` (0x6737d0), where `ecx` on entry is the
   **caster's player**, not a `SpellType*`: `SpellType::cast (00676ce0)` calls
   it at 0x676d8e with `ecx = edi = cast's param_2` and pushes only two words
   (`cast`'s `param_3`, `param_4`: the target's index and player). The
   decompile's `this`/`param_1`/`param_2` are therefore caster player, target
   index, target player.

### Event, step, predicate

Each writer adds **1** (`incl`), once per event.

- **`take_damage`, a squad killed by an attacker object.** Reached only when
  the struck object's whole points taken (`+0x24`) are at or above its
  threshold (`local_c`, the figure's share; decompile line 345 is the survive
  branch) — i.e. on the death path, before `die` (vslot `+0x158`, line 620).
  On that path, the object is not a building (vslot `+0x1c` = 0; a building
  takes branch 4.1 instead), **and** it is its squad's captain (`+0xe8`), **and**
  `curr_uber_size(0) == 1` (the last figure standing), **and** its
  `unit_masks +0x68 & 1` (decoy) is clear — then `units_lost[owner] += 1`
  (leaf 2), **and then only if `param_7 >= 0`** (an attacker object index):
  `deaths_current_frame[owner] += 1` (+0xa58), `kills_current_frame[param_8]
  += 1` (+0xa5a), `units_killed[param_8] += 1`, and then `score_combat
  [param_8]` (+0x44) gains `cost/5`, or loses it floored at 0 when the attacker
  is allied to the victim. **No alliance, ownership or `param_8 >= 0` test
  guards `units_killed`**: an ally's kill counts, and so does a player's own
  — the decoy-overflow recursion (decompile 624-630) strikes the captain with
  `param_7 = captain`, `param_8 = owner`, so a captain that dies of its own
  decoy's overflow credits its own owner one kill (and one
  `kills_current_frame`). Callers passing `param_7 = -1` (attrition, wall decay,
  sabotage's building arm, the scenario/editor health setters) never step it.
  A non-captain figure, a figure with live squadmates, or a decoy never steps
  it: the counter is per squad, at the last figure.
- **`cast_sabotage`, the unit arm** (Sniper/Commando kill). Gate: the target
  object's flags `+0x8 & 1` set — the bit `take_damage` itself demands
  first (decompile line 55) — else the spell's other arm runs and nothing is
  counted; the target not a building (`+0x1c` = 0) and a unit (`+0x18`);
  captain, `curr_uber_size(0) == 1`, decoy bit clear. Then
  `deaths_current_frame[target owner] += 1`, `kills_current_frame[caster] +=
  1`, `units_killed[caster] += 1`, `score_combat[caster] += cost/5` (cost of
  the target's type, vslot `+0x80` of its `Type` with the target's player).
  No alliance test here (the castability check upstream, `cast`'s
  `0x675bc0` call, was not read). The target then `die`s directly (vslot
  `+0x158`, dtype 3 if it carries objmask `0x200000`, else 2) — **not through
  `take_damage`, so `units_lost` of the victim is not stepped** on a sniped
  squad.
- **`cast_counterintel`** (a Spy's counter-intelligence on an enemy spy). The
  target is first replaced by its captain if it is not one (`+0xe8`, `+0xe4`).
  Gate: target player ≠ caster, target player ≠ `leaders[caster].who`, and
  not allied both ways (`diplos[...] == 2` mutual — `LeaderData::is_ally
  (006edb50)` inlined). Inside it: the target a unit, captain, `curr_uber_size
  (0) == 1`, decoy clear → `units_killed[caster] += 1` and `score_combat
  [caster] += cost/5` (the `0x66666667` multiply is a signed divide by 5).
  Then `die(2, -1, 0)` on the target whatever the inner gate said. **No
  `kills_current_frame`, no `deaths_current_frame`, no `units_lost`** here —
  three asymmetries against `take_damage`.

### Clears

`Leader::reset_score (006e37f0)` (from `Leader::init (006e3930)` and from
`Setup::build_game (005ac190)` on every live slot); `Leaders::clear
(006ed5d0)` and `Leader::init`'s empty-slot memset, both covering
[+0, +0x6929). No per-frame fold.

### Readers

`LeaderData::walk_data (006d6750)` (the checksum walk; `docs/CHECKSUM.md` §8
puts [+0x8, +0x692a) in it for the walked slots — **this is the reader that
matters for lockstep**), `LeaderData::log_data (006e5110)`,
`Game::determine_end_of_game_stats_and_achievements (00583b30)` (a Steam stat
at game end), `AchieveChartPane::set_chart_data (007f2c40)` (post-game
chart), `ScenarioFuncSet::num_units_killed (009e98e0)` (script function,
1-based player, -1 for a dead slot). The install's scripts that call it are
CtW and one custom scenario; none under `ai/`. No reader changes a decision in
a plain skirmish.

### Crate site

- take_damage's arm: `Sim::take_damage_typed`, `crates/sim/src/combat/fight/
  delivery.rs:469`, its `Obj::Unit` `Taken::Died` branch (line ~537), beside
  `unit_plunder`, whose gate (captain, last figure, no decoy) is the same
  predicate (doc at delivery.rs:613-620). The attacker is `_by`; the crate has
  no `param_7 < 0` notion there, so a builder must pass "attacker object
  present" explicitly (attrition does not come through this function in the
  crate: it is `die_by_attrition`, delivery.rs:844). The decoy-overflow
  recursion is at delivery.rs:576 and passes `Obj::Unit(k)` as its own
  attacker, matching the original's self-credit.
- cast_sabotage's unit arm: `Sim::cast_sniper_unit`, delivery.rs:806, which
  already carries `SEAM: the caster's leader counters (+0xa58, +0xa5a,
  +0x80c) and the score` (note: `+0xa58` is the **target owner's**, not the
  caster's).
- cast_counterintel: `Sim::cast_counterintel`, `crates/sim/src/spells/
  cast.rs:1030`, `SEAM: units_killed and score_combat`.
- The leaf exists only as a checksum name: `crates/sim/src/checksum/
  leaders.rs:157`; `rondata`'s `checksum_lines.rs:1847` marks it "unheld".

### Plain game?

Yes for all three. `take_damage` via `do_damage` is every combat kill.
Sabotage (Commando/Special Forces, Sniper) and counter-intelligence (Spy) are
units' spells in a standard skirmish; whether the computer casts them was not
read.

### Confidence and not read

High on the three stores and their indices (listing). High on take_damage's
gate (decompile and listing 0x652f4d-0x653046 agree). Medium on the sabotage
gate's first test (`+0x8 & 1`), named here only by `take_damage`'s use of it. Not read:
`SpellTypeData::is_castable`'s predicate (`0x675bc0`), the `+0x80` type-cost
vslot, whether `curr_uber_size(0)` counts the dying figure.

## 2. `units_lost` (+0x810, int)

### Writers

**`Object::take_damage (00652020)` only**, store 0x653007 (decompile line
504). Index `edx = this->field_0x9` (the struck object's owner) `·0x6eec`.
Plus the clears. No other store in either net.

### Event, step, predicate

+1 on the death path of `take_damage` (damage at or above the share
threshold), when the struck object is **not a building** (`+0x1c` = 0), is
its squad's **captain** (`+0xe8`; inlined as `unit_masks +0x8e` bit 15 when
the slot is `UnitData::is_captain (0046ceb0)`, else the virtual call at
0x653464), has `curr_uber_size(0) == 1`, and its decoy bit (`+0x68 & 1`) is
clear. Credited to the **victim's owner**. Stepped before, and independent
of, the `param_7 >= 0` test that gates `units_killed`: an attrition death
(`Unit::suffer_attrition (005e1a10)`, `param_7 = -1`) of a last-figure
captain counts a loss with no killer, and so do deaths by
`ScenarioFuncSet::set_object_health`/`damage_object` and the editor.

**Deaths that do not step it**, because they reach `die` without
`take_damage`: a Sniper/Sabotage kill (`cast_sabotage` unit arm calls vslot
`+0x158` directly), a counter-intelligence kill (`cast_counterintel`, same),
`Object::disband`, and any other direct `die`/`close`. Not enumerated here:
the full set of direct `die` callers (conversion, cheats, CtW).

### Clears

As leaf 1: `reset_score`, `Leaders::clear`, `Leader::init`'s empty-slot
memset.

### Readers

The checksum walk (`LeaderData::walk_data (006d6750)`), `log_data`,
`determine_end_of_game_stats_and_achievements` (Steam stat `gStatID[7]`),
`AchieveChartPane::set_chart_data`, `ScenarioFuncSet::num_units_lost
(009e98b0)` (CtW and custom-scenario scripts only). None changes a decision
in a plain skirmish.

### Crate site

Two crate functions carry the original's one site:

- combat: `Sim::take_damage_typed`, `crates/sim/src/combat/fight/
  delivery.rs:469`, `Obj::Unit` `Taken::Died` branch (~line 537), the same
  captain/last-figure/no-decoy gate as `unit_plunder`.
- attrition: `Sim::die_by_attrition`, delivery.rs:844, called from
  `crates/sim/src/territory/upkeep.rs:307` — in the original this death runs
  through `take_damage` (`suffer_attrition`'s vslot `+0x16c`), so the counter
  must be added here too, with the same gate (the crate's attrition picks the
  dying figure itself; whether it is the captain and last figure must be
  asked there).

The crate holds the leaf only as a checksum name (`checksum/leaders.rs:158`;
`checksum_lines.rs:1848` "unheld").

### Plain game?

Yes: every combat death of a squad, and every attrition death of one.

### Confidence and not read

High on the store, index and gate (listing 0x652f75-0x653007). Not read:
`curr_uber_size`'s body (whether it counts the dying figure; the threshold
`== 1` reads as "this is the last"), and whether `Unit::suffer_attrition`
can strike a non-captain figure (it calls slot `+0x16c` on `this`).

## 3. `buildings_razed` (+0x818, int)

### Writers

1. **`Object::do_damage (0064a480)`, store 0x64ba49** (decompile line 903).
   Index `eax = this->field_0x9 · 0x6eec`, `this` = the shooter
   (`do_damage`'s object; callers `Unit::fight` ×3, `Ammo::do_damage` ×3,
   `Nuke::do_damage` ×2, and itself once).
2. **`SpellType::cast_sabotage (006720e0)`, store 0x67277d** (decompile line
   283). Index `esi = param_2 · 0x6eec`, the caster's player.

`Object::take_damage` does **not** write it: the razer is credited by the
caller, after `take_damage` returns.

### Event, step, predicate

- **`do_damage`**: +1 to the shooter's owner when the `take_damage` call at
  0x64ba18 returns non-zero **and** the target answers vslot `+0x1c`
  (building). No alliance or ownership test: razing an ally's or one's own
  building counts. Then, for the console player when `Game::is_game_ready`, a
  Steam stat (`STAT_BUILDINGS_DESTROYED`) — not sim state.
  **Non-zero includes the half-health disband** (`take_damage` decompile
  352-360): a building, not a city (`+0x4c` = 0), owner's `leader_flags & 4`
  clear, `max_hits(0) <= 2·damage`, and its record's `+0x60 & 0x2000` (the
  "AI gave up on this site" bit, `docs/CITIES.md` §1.3's table, set by
  `Leader::check_orphaned_buildings`) → `Object::disband(this, 0)`, return 1.
  So a shooter that knocks an abandoned AI site past half health is credited
  a raze, while the owner's `buildings_lost` is **not** stepped (that store
  sits only on the death path).
- **`cast_sabotage`, building arm**: the damage is the target's (via vslot
  `+0xac`) max hits (vslot `+0x114`) halved, floored at `ELITE_MIN_DAMAGE`
  when the spell's type index (`+0x4`) is `0x4d`, at `SPECIAL_MIN_DAMAGE`
  when `0x4b`, else `COMMANDO_MIN_DAMAGE`; passed to `take_damage(dmg, 0, 3,
  -1, 1, 0x55555555, -1, -1, 0)` on that object. Non-zero return →
  `buildings_razed[caster] += 1`, `score_combat[caster] += cost/5` (the
  target type's cost, `/5` signed), and the Steam stat for the console. Same
  half-health-disband inclusion as above. No alliance test in this function.

### Clears

`reset_score`, `Leaders::clear`, `Leader::init`'s empty-slot memset.

### Readers

Checksum walk, `log_data`, `AchieveChartPane::set_chart_data`. Not read by
`determine_end_of_game_stats_and_achievements` (it reads +0x80c and +0x810
only) and no script function. No sim decision.

### Crate site

- `Sim::do_damage_round`, `crates/sim/src/combat/fight/delivery.rs:70`, the
  `if let Obj::Building(b) = target { … if killed {` block at ~line 212: the
  counter goes before the plunder test, unconditionally (the plunder's
  non-air / non-splash / non-ally gates do not apply to it). The crate's
  `killed` is `Taken::Died` only; the orphan-site disband is not carried by
  `damage_building` (delivery.rs:957), so that arm of the predicate has no
  crate event yet.
- `Sim::cast_sabotage_building`, delivery.rs:884, whose doc already says
  `SEAM: the caster's leader counters and score`.
- `Nuke::do_damage` callers: not checked in the crate.

### Plain game?

Yes: any building destroyed by a unit, ammo or nuke; Sabotage on a building.
The orphan-site arm needs a computer opponent that abandons a site, which a
plain skirmish against the computer can produce.

### Confidence and not read

High on both stores and their indices; high on the non-zero test (listing
0x64ba1e-0x64ba49). High that the disband path returns 1 (listing: `call
0x6455c0` `Object::disband` at 0x652aa2, then `movl $0x1,%eax` and the
epilogue); its predicate is read from the decompile only. Not read: vslot
`+0xac` on a building (which object the sabotage strikes), `Nuke::do_damage`'s
`this`.

## 4. `buildings_lost` (+0x81c, int)

### Writers

**`Object::take_damage (00652020)` only**, store 0x652f5b (decompile line
483-484). Index `eax = this->field_0x9 · 0x6eec`, the struck building's
owner. Plus `reset_score` (0x6e3835) and the memsets. The ~50 other
`0x81c(%reg)` stores are `Game::semaphore.flags` (§0).

### Event, step, predicate

+1 on `take_damage`'s death path (damage at or above the threshold, line
345 not taken) when the struck object answers vslot `+0x1c` (building).
Beside it, `ScenarioData::last_razed[owner] = this's index` (0x652f69,
`0xcc2190`), then straight to `die` (`LAB_006533b2`). **No attacker test**:
it counts whether or not `param_7 >= 0`, so it counts:

- a building destroyed in combat (`do_damage`);
- a **wall segment decaying in hostile territory**: `Wall::process
  (00640450)` calls `take_damage(8, 0, 1, -1, 1, 0x55555555, -1, -1, …)` on a
  started wall standing in a non-allied player's territory, and `Wall`'s
  vslot `+0x1c` is the same `return 1` (vtable 0x00b42cf8), so each segment
  that decays to death is a lost building for its owner;
- the building `cast_sabotage` strikes;
- the scenario/editor health setters.

**Not counted**: a city (vslot `+0x4c` with `+0x8 & 0x20`) is clamped at
full damage and returns 0 before the death path (decompile 292-302); the
half-health disband of an abandoned AI site (decompile 352-360) returns 1
without reaching this store (so `buildings_razed` can step with no matching
`buildings_lost`); any building removed by `Object::disband` or a direct
`die` (delete, captured-and-razed paths, building attrition if it does not
use slot `+0x16c` — no `Build::` function is among the eight `+0x16c` call
sites).

### Clears

`reset_score`, `Leaders::clear`, `Leader::init`'s empty-slot memset.

### Readers

Checksum walk, `log_data`, `AchieveChartPane::set_chart_data`. No script
function and no Steam stat read it. No sim decision.

### Crate site

`Sim::damage_building`, `crates/sim/src/combat/fight/delivery.rs:957`, its
`Taken::Died` branch (~line 1041, just before `self.die_building(b)`). That
branch serves all three plain-game callers: combat (`take_damage_typed` →
delivery.rs:599), Sabotage (`cast_sabotage_building` → delivery.rs:887) and
`Wall::process`'s enemy-territory hit (`crates/sim/src/cities/city.rs:3128`,
`damage_building(b, hit, None, frame, true)`). The counter must sit there and
not in `die_building` itself, which `disband.rs:360`, `city_views.rs:267`,
`city.rs:660/687` and `setup.rs:148` also reach. The crate does not carry
`last_razed` on this path (`units/disband.rs:97` carries it for disband's).

### Plain game?

Yes: combat razing, and wall decay in enemy territory whenever a player walls
into another's borders.

### Confidence and not read

High on the store, index and gate (listing 0x652f4a-0x652f70). Not read:
whether any building death outside the eight `+0x16c` callers exists that a
reader would expect to count (e.g. `City` destruction on capture, scenario
`kill_object`), and the predicate of `Wall::process`'s territory test beyond
"non-allied owner".

