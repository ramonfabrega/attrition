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
  `semaphore` bit `0x200` of byte +0x821. Not LeaderData.
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

