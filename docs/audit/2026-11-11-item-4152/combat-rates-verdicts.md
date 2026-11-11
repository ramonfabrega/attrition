# Item 4152, adjudication: `combat-rates`

- Slug: `combat-rates`
- Leaves (`LeaderData`): `frame_battle` (+0xa4c, int); ushort
  `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- Inputs: `combat-rates-reading.md` (first), `combat-rates-second.md` (blind
  second), the decompile export (`~/ghidra-projects/decomp`), the `.text`
  listing (`exe.s`, llvm-objdump), `crates/sim`.
- Written per claim as settled; a section without its closing rule was not
  finished. The readings share their writers (W1..W4, the fold), so the
  shared claims are settled once in section 0 and each leaf cites them.

## 0. The shared writers, settled once

### 0.1 Layout and nets

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| `Leader` holds `LeaderData` at +0, so `field_0xa4c..0xa66` on a `Leader*` is the leaf; slot `i` at `0xe3a390 + i·0x6eec`, the leaves at `0xe3addc..0xe3adf7` | made | made | agree | confirmed | `types.txt:42318-42330` (offsets and names); `Leaders::clear (006ed5d0):11-12` memsets from `field_0x6c14 - 0x1b05·4`, the record's byte 0 |
| export by name: only `LeaderData::log_data (006e5110)` | made | made | agree | confirmed | my `grep -rlE` of the 13 names over `funcs/` returns that one file |
| export by offset: `inc_{hit,damage,kill,death}_event`, `process_event_frame`, `Leader::init`, `do_damage`, `take_damage`, `cast_sabotage`, `draw_timers`, and `TerrainOut::one_time_init` (another class) | made | made | agree | confirmed | my `grep -rl field_0xa4c / a5x / a6[0-7]` returns exactly these 12 files |
| absolute `0xe3addc..0xe3adf7` occurs 4 times, all read-modify-writes | made (sites named) | made (count 4) | agree | confirmed: `64a91e`, `64bbe3`, `653033`, `672228` | `exe.s` grep |
| register-relative stores at `0xa44..0xa67` (exact and `leader+8` shifted) belong to other classes except the Leader family | made | made | agree | **confirmed**, with three sites neither named: `SetupWin::process_combo_change` `5ba4f3`, `SetupWin::exec` `5c7ba6` (both `movb $0, 0xa46(%eax)` off `0xc0622c`, `connection_data`, as the first named for `do_scen_changes`) and `SetupWin::on_combobox_select` `5c40d2` (off the window `esi`). No leader store moves. My net (every non-`cmp/test/push/lea` instruction whose last operand is `0xa44..0xa67(%reg)`, `ebp/esp` excluded) finds 96 sites in 28 functions; the only `Leader`-family ones are `do_damage` 2, `take_damage` 1, `cast_sabotage` 1, `inc_*_event` 4, `Leader::init` 9 (7 in the family plus `+0xa44`, `+0xa48`), `Leader::gain_tech` 1 (`+0xa48`, not a leaf), `process_event_frame` 15 | `exe.s` sweep |
| the four `Leader::inc_*_event` have no caller; their inlined copies are the writers | made | made | agree | confirmed | no `call 0x6daa30/0x6daa50/0x6dab10/0x6dab30` in `exe.s` (count 0) |
| `Leaders::clear (006ed5d0)` memsets `[0, 0x6929)` of every slot | made | not made | one-sided | confirmed (covers `+0xa4c..+0xa67`) | `Leaders::clear:12` |
| `LeaderData::walk_data (006d6750)` loads `[+0x8, +0x692a)` as one block when `leader_flags & 1` | not made | made | one-sided | confirmed (a save-game load restores all 13; also the sync checksum walk) | `walk_data:16-17` |
| `Leader::init`'s own memset of `[0, 0x6929)` when `param_2 < 0` | not made | not made | **neither** | a third bulk writer, redundant with init's explicit zero stores | `Leader::init:70-77` (as settled in `city-name-verdicts.md`) |
| `Leader::init (006e3930)` zeroes all 13 by six dword stores at `a50..a64` and one at `a4c` | made | made | agree | confirmed | `6e3b7a`, `6e3b86`, `6e3b8c`, `6e3b92`, `6e3b98`, `6e3b9e`, `6e3bda` |

**Writer completeness.** No writer neither reader found. The complete set
per leaf is the value writers below (W1..W5) plus the four bulk writers:
`Leader::init`'s explicit stores, its `param_2 < 0` memset, `Leaders::clear`
and `walk_data` on input.

### 0.2 W1, `Object::do_damage (0064a480)`, the blow

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| head returns: `param_6 < 1`; attacker `is_unit` (vslot `+0x18`) with `unit_masks & 1` (decoy) | made | made | agree | confirmed; both jump to the epilogue `64c86b` | `64a49e`/`64a4ae` (`cmpl $0, 0x1c(%ebp)`; `jle`); `64a4b6`..`64a4dc` (`testb $1, 0x68(%eax)`; `jne`) |
| no exit between the head and the store | made | made (implied) | agree | confirmed: every branch from `64a541` to `64a90e` targets inside that range | `exe.s` `64a541..64a934` |
| `hits_current_frame[attacker.who] += s`, `damage_current_frame[param_2] += s`, `param_2` the target's owner | made | made | agree | confirmed | `64a8e3..64a90e` (`movzbl 0x9(%edi)`, `imull $0x6eec`, `addw %ax, 0xa5c(%ecx)`); `64a915..64a91e` (`0xc(%ebp)·0x6eec`, `addw %ax, 0xe3adee(%edx)`) |
| `s = 5` when attacker type vslot `+0x10c` (`is_siege`) and target vslot `+0x1c` are both non-zero, else 1 | made | made | agree | confirmed; `s = 4·[both] + 1` | `64a8b3..64a904` (`leal 0x1(,%eax,4)`) |
| vslot `+0x1c` is "building or wall" (`is_wallbuild`) | made | made | agree | confirmed: Build and Wall answer `Buffer::is_pending_load` (returns 1), Unit and Animal answer `Window::get_button` (returns 0) | `vtables.txt` Build/Wall/Unit/Animal `+0x1c`; `Buffer/is_pending_load@0041e0e0.c:4`, `Window/get_button@0041bff0.c:4` |
| no owner, ally or self predicate on the store (it precedes the `param_2 != attacker.who` test) | made | made | agree | confirmed | `64a932`/`64a934` (`cmpl %eax,%ecx; je`) come after both stores |
| the step counts blows (events), not hit points | made | made | agree | confirmed: the added value is `s`, never the damage | `64a904` |

### 0.3 W2, `Object::do_damage`, the structure kill

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| after the take (vslot `+0x16c`) returns non-zero, return unless the target `is_wallbuild` | made ("returns at `0064ba39`") | made ("returns at `64c86b`") | agree | confirmed; `64ba39` is the `je`, `64c86b` its target | `64ba18..64ba39` |
| `kills_current_frame[attacker.who] += k`, `deaths_current_frame[param_2] += k` | made | made | agree | confirmed; no exit between `64ba3f` and the stores | `64bbb5..64bbe3` (`-0x28(%ebp)` = `param_2·0x6eec`, set at `64a929`) |
| `k = 5` if `local_20` (`-0x1c(%ebp)`) else 2; `local_20` = target `flags & 1` and vslot `+0x20` and vslot `+0x4c`, read **before** the take | made | made | agree | confirmed | `64a806..64a828`; `64bbcf..64bbd3` (`cmovnel`) |
| vslot `+0x20` is `is_build` (Build 1, Wall 0) | made | made | agree | confirmed | `vtables.txt` Build `+0x20` `Buffer::is_pending_load`, Wall `+0x20` `Window::get_button` |
| vslot `+0x4c` on a Build: body not read (first); `is_active` (second) | open | made | one-sided | **settled**: for Build and Wall it is the folded `WallData::is_active`, `flags & 4` (the byte at `+0x8`, bit 2); do_damage inlines the same test for the Build vtable | `WallData/is_active@00472350.c:4`; `64a871..64a87c` (`cmpl $0xb42174`; `andl $4`) |
| the take can return 1 on its **not-dead** path, which W2 then counts | not made (first: "a reduction counts nothing") | made (the disband arm) | one-sided | **confirmed, the second's addition**: when `damage < hits` but the object `is_wallbuild`, is not active, its leader lacks `leader_flags & 4`, `hits <= 2·damage`, and its type (vslot `+0xb0`) has flag `0x2000` at `+0x60`, the take disbands it and returns 1; do_damage then adds `k = 2` (inactive). The first's "a city at its ceiling returns 0" is also right (`take_damage:291-301`) | `take_damage:349-361` |
| no unit, splash, air or ally gate on the W2 store | made | made | agree | confirmed | `64ba3f..64bbe3` |

### 0.4 W3, `Object::take_damage (00652020)`, a unit's death

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| a dying `is_wallbuild` object bumps `+0x81c` and skips the leaves | made | made | agree | confirmed | `652f46..652f70`; `take_damage:483-489` |
| unit predicate: `is_captain` (vslot `+0xe8`, inlined `(short)o_up < 0` via `+0x8e >> 15`), `curr_uber_size(0) == 1`, `unit_masks & 1` clear | made | made | agree | confirmed | `652f75..652ffb` |
| then `+0x810 += 1` unconditionally, then only if `param_7 >= 0` the two stores | made | made | agree | confirmed | `653007` (`incl 0xe3aba0`), `65300d`/`653011` (`cmpl $0, 0x20(%ebp)`; `jl`) |
| `deaths_current_frame[this.who] += 1`, `kills_current_frame[param_8] += 1` | made | made | agree | confirmed; `ebx` = `param_8` from `652065` (`movl 0x24(%ebp), %ebx`), not reassigned before `653033` | `653026`, `653033` |
| do_damage passes `param_7` = the attacker's own index (`+0xa`, short, always >= 0 for a live object), `param_8` = attacker's `who` | made | made | agree | confirmed | `64b9fa..64ba03` |
| object call sites of vslot `+0x16c`: nine (first) / "every other caller" five plus do_damage and the tail (second) | 9 listed | 7 listed (omits `ScenarioFuncSet::damage_object` `9f5c94`) | disagree (count) | **corrected to the first's nine**. A tenth `call *0x16c` at `41c36f` is `Window::on_mouse_wheel (0041c360)`, a Window vtable, not an object. Every non-do_damage, non-tail site pushes `param_7 = param_8 = -1` | `5e1bbb/5e1bbd`, `640b0f/640b11`, `672747/672749`, `9a022b/9a0231`, `9f5b9c/9f5b9e`, `9f5c53/9f5c55` |
| the tail re-call at `653426`: overflow goes to the captain with `param_7` = captain index, `param_8` = the victim's own `who` | made: "the overflow past a dead figure's share ... when the dead figure is not its own captain" | made: "decoy-overflow re-call" | disagree (predicate) | **corrected to the second's**: the re-call runs only when the dying unit is a **decoy** (`uVar10 = unit_masks & 1`, line 452; `if (uVar10 == 0) return 1` at line 621), the overflow `damage - hits` is positive, and its captain index (vslot `+0xe4`) is `>= 0` and not itself. An ordinary non-captain figure never reaches it. If the overflow kills the captain (captain, size 1, not decoy), the owner gets one death and one kill of its own | `take_damage:449-453`, `:619-631`; `6533d3` (`testl %esi,%esi; je 652aa7`, return 1), `6533db..653426` |
| attrition, Wall::process, scenario paths count nothing | made | made | agree | confirmed (they push `-1`) | as the call-site row |

### 0.5 W4, `SpellType::cast_sabotage (006720e0)`, the unit arm

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| a dead target (`flags & 1` clear) takes the refund and returns | made | not made | one-sided | confirmed | `672123..67217f`; `cast_sabotage:61-73` |
| unit arm: target not `is_wallbuild`, `is_unit`, `is_captain`, `curr_uber_size(0) == 1`, not decoy → `deaths[param_4] += 1`, `kills[param_2] += 1` (and `+0x80c`, `+0x44 += cost/5`) | made | made | agree | confirmed | `67218f..672228`; `cast_sabotage:77-104` |
| the arm's later `take_damage` passes `param_7 = -1`, so no double count | made | made | agree on the conclusion | **corrected on the mechanism**: the unit arm never calls `take_damage`. It kills the unit by `Unit::die` (vslot `+0x158`, `(2, -1, 0)`) and returns; the `take_damage` at `67275d` is the **building** arm's. So the step is credited on the predicate alone, and the unit dies whether or not it qualified. `Unit::die` writes no leaf (not in either offset net) | `cast_sabotage:106-118` (die, return), `:275-279` (building arm's take); `vtables.txt` Unit `+0x158` `Unit::die` |

### 0.6 W5, `Leader::process_event_frame (006ec180)`, the fold

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| called from `Game::do_frame (00591ef0)` for slots 0..7 (`< 0xe71af0`) with `leader_flags & 1`, after `Achieve::capture_data`, before `frame += 1`; unconditional otherwise | made | made | agree | confirmed; (`0xe71af0 - 0xe3a390`) / `0x6eec` = 8 | `5924a0..5924bf` |
| returns unless `game->frame % 50 == 0` (signed `idiv`; frame >= 0) | made | made | agree | confirmed | `6ec199..6ec1b5` |
| step 1: `X_fifteen += X_current` (16-bit) | made | made | agree | confirmed | `6ec1c2`, `6ec1d0`, `6ec1e5`, `6ec1ec` |
| step 2: `X_current = (X_current·100) mod 2^16` | made | made | agree | confirmed | `6ec1f3..6ec21d` |
| step 3: `c == 0 ? avg·7 >> 3 : (c + avg) >> 1`, unsigned 32-bit on zero-extended words; the zero test is on the 16-bit product | made | made | agree | confirmed | `6ec224..6ec2d3` (`testw`, `leal (,%eax,8)`/`subl`/`shrl $3`, `addl`/`shrl`) |
| step 4: jukebox mood, console leader only; writes no leaf | made | made | agree | confirmed. Thresholds: `H + Dm < 600` and `>= 300` keeps; `< 300` economic; `>= 600` win/lose; `2000` only when `0xecba20 == 2` (the second's extra threshold is right). The rival filter skips a slot when both `rival.diplos[this.who]` and `leaders[this.who].diplos[rival.who]` are non-zero (the first's), and needs `average_hit_rate` or `average_damage_rate` non-zero (`0xa4e/0xa4c(%esi)`, `esi = slot + 8`) | `6ec2e0..6ec44d` |
| step 5: battle test, `age = leaders[this.who].data_encrypted->+0xdc ^ 0x62766` | made | made | agree | confirmed | `6ec452..6ec475` (`0xe41248` = slot base + `0x6eb8`) |
| `D + K >= (age+1)·125`, lock-out `frame_battle == 0 or frame - frame_battle >= 1800`, defeat `D >= K + 10·age + 20`, else victory `K >= D + 10·age + 20`, else nothing | made | made | agree | confirmed; all signed `jl` on non-negative ints | `6ec47b..6ec4d2` |
| event: `Achieve::add_event(1 defeat / 0 victory, who, "")`, then `movl $0xfc18fc18, 0xa50`, `frame_battle = frame` | made | made | agree | confirmed | `6ec4ba..6ec4f9` |
| step 6: `movl 0` at `a58` and `a5c`, on every fold frame | made | made | agree | confirmed (`eax` zeroed at `6ec4ff`) | `6ec4ff..6ec507` |

### 0.7 Crate sites, checked once

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| W1 home: `Sim::do_damage_round`, `crates/sim/src/combat/fight/delivery.rs:70`, after the decoy return | made | made (beside `treaty_on`, `:108`) | agree | confirmed; the decoy return is `delivery.rs:94-98`, `treaty_on` `:107-109`, and the original's store sits just before its treaty marking, so the counter belongs between them | `delivery.rs:70-109` |
| the crate's extra `!self.struck(target)` return precedes the decoy test, which the original's store does not see | made | made | agree | confirmed as a placement hazard: `struck` (`combat/fight/stats.rs:92-100`) refuses a non-active unit or a dead or combat-less building. **Open**: whether the original ever runs `do_damage` on such a target (if it does, the original counts the blow and the crate cannot) | `delivery.rs:81-83` |
| W2 home: `do_damage_round`'s `if let Obj::Building(b) = target { if killed`, outside its unit/splash/air/ally gate, with `active` read before the take | made (`~211`) | made (`:212-215`) | agree | confirmed; `killed` is `delivery.rs:193`, the building block `:212-222`. The crate's `Obj` has only `Unit` and `Building` (`combat.rs:381-384`), so a wall's home is wherever the crate keeps walls; not settled here | `delivery.rs:193`, `:212-222` |
| W2 needs the disband arm's return 1 as a kill | not made | made | one-sided | confirmed (0.3): the crate's `killed` must be true on a disbanded inactive structure, or `k = 2` is lost on it. Not checked whether `damage_building` models the disband arm | `take_damage:349-361` |
| W3 home: `Sim::take_damage_typed`, `delivery.rs:469`, the unit `Taken::Died` arm | made (`~532`) | made (`:530`ff.) | agree | confirmed: the arm is `delivery.rs:537`. Gate on captain, squad size 1, not decoy and a real attacker | `delivery.rs:469`, `:537` |
| the crate's `take_damage` wrapper is also the attrition path's, so the gate must tell them apart | made | contradicted ("the attrition path never comes here") | disagree | **corrected to the second's**: `Sim::take_damage` (`delivery.rs:452`) is `#[cfg(test)]` (`:451`); outside tests `take_damage_typed` has three callers, `do_damage_round` (`:192`), the test wrapper (`:463`) and the decoy tail (`:586`); attrition is `Sim::suffer_attrition` (`territory/upkeep.rs:286`), which does not call it. The gate on a real attacker still matters for the tail, whose attacker is the captain itself (as the original's `param_7` = captain index) | `delivery.rs:451-463`; grep of `take_damage_typed` |
| the decoy tail already exists at `delivery.rs:560-596`, passing the captain as its own attacker | not made | not made | neither | a counter added in the `Died` arm credits the owner's own kill on that path, as the original does, with no extra code | `delivery.rs:560-596` |
| W4 home: `Sim::cast_sniper_unit`, `delivery.rs:806`, whose SEAM (`:803`) names "the caster's leader counters (`+0xa58`, `+0xa5a`, `+0x80c`)" | made, with the correction that `+0xa58` is the target's | made (no correction) | one-sided (correction) | **confirmed, the first's correction**: `deaths_current_frame` goes to `param_4`, the target's owner; only `+0xa5a`, `+0x80c`, `+0x44` to the caster's | `67220a`/`67221c` (`esi = 0x14(%ebp)`, `param_4`); `cast_sabotage:97` |
| the crate's sniper arm kills by `die`, not by a take | not made | not made | neither | consistent with 0.5: `cast_sniper_unit` calls `die_with` (`delivery.rs:806-808`), so the counter goes in that function, on the captain/size/decoy predicate, before the die | `delivery.rs:806-830` |
| `cast_sabotage_building`'s SEAM (`delivery.rs:881`) names `+0xa58`, `+0xa5a` on the building arm | not made | not made | **neither** (finding) | **corrected**: the building arm writes no leaf of this family. Its only leader stores are `+0x818` and `+0x44` (`cast_sabotage:283-287`), and its `take_damage` passes `param_7 = -1`, so W3 does not fire either. A sabotaged building counts no kill and no death. The SEAM should drop `+0xa58`/`+0xa5a`/`+0x80c` and name `+0x818` | `cast_sabotage:279-287`; the listing's only two family stores in the function are `67221c`, `672228` |
| W5 home: a new `process_event_frame` in `Sim::tick`, `core/frame.rs`, before `self.frame += 1`, for slots 0..7 with `leader_flags & 1` | made (`~475`) | made (`:73`, `:475`) | agree | confirmed: `Sim::tick` is `core/frame.rs:73`, `self.frame += 1` is `:475`. No function or field of the family exists in `crates/sim` beyond the checksum table | `core/frame.rs:73`, `:475`; `checksum/leaders.rs:264-276` |
| field home: `crate::ai::Leader` (`ai.rs:637`), beside `frame_attacked`/`attacked_by` (`:697`) | not made | made | one-sided | confirmed | `ai.rs:637`, `:697-698` |
| rondata holds all 13 as `(unheld)` checksum rows | not made | made | one-sided | confirmed | `crates/rondata/src/diff/checksum_lines.rs:1762-1804` |

