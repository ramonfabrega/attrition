# diplomacy-relations: blind second reading (item 4152, wave 2)

- Slug: `diplomacy-relations`
- Leaves (LeaderData offsets): `good_deeds` +0xd4 int[8], `hire_stamp` +0x1b4 int[8], `hire_who` +0x1d4 int[8], `blacken` +0x20c int, `dow` +0x270 int[8], `broke_alliance` +0x2b0 int[8], `made_peace` +0x2d0 int[8], `last_spoke` +0x2f4 int[8]
- Reader model: Claude Opus 5.5 (claude-opus-5-5)
- Sources: decompile export `~/ghidra-projects/decomp`, PE listing (llvm-objdump of `riseofnations.exe` .text), `crates/sim`. Blind: nothing under this audit directory but this file was opened.

## Ground rules established first

- **Base register.** In the listing, a `Leader*` (`this` of `Leader::*`) points at the `LeaderData` base: `Leader::diplomacy (006bc950)` reads `hire_stamp` as `0x1b4(%ebx,%eax,4)` and `blacken` as `0x20c(%edx)`, and `Leaders::clear (006ed5d0)` memsets from `0xe3a390` per slot. So the listing displacements are the `LeaderData` offsets, unshifted; the global form is `0xe3a390 + slot*0x6eec + off`, and the pair form `slot*0x1bbb + other` indexes `[.. ,4]`. The decompile's `Leader` type is wrong in stride (it prints `*0x6ef0`, `*0x6f48`) but its `field_0xNNN` names on a `Leader*` match the `LeaderData` offsets; I used the listing for every address.
- **Frame.** `GameAccess::game` is `[0xc061ec]`, and `game->frame` is `+0x550`. `GameAccess::constants` is `[0xc061f0]`; `ally_to_war_delay` is its `+0xcf8`.
- **Diplos.** `LeaderData::is_enemy (006ebaa0)`: different slots and `diplos` 0 either way. `LeaderData::is_ally (006edb50)`: same slot, or `diplos == 2` both ways. So 0 war, 1 peace, 2 allied.
- **`leader_flags & 4` is HUMAN** (`docs/ARMY.md` line 103, the PDB enum).
- **Nets.** Export: `->leaf` greps return only `hire_who` (in `BuildOut::make_options`, a misnamed read through an unrelated pointer) and `blacken` (a read in `LeaderData::log_data`); the decompile prints every other access as `field_0xNNN`. Listing: a scan of the whole `.text` for every write-shaped instruction (`mov/add/sub/inc/dec/and/or/xor/neg/not/shift/set*/xchg/rep stos`) whose destination is one of the leaves' displacements (`off + 4k`) or an absolute in `[0xe3a390, +10*0x6eec)` that reduces to one, plus every `lea`/`mov $imm`/`push $imm` that forms a leaf's address. Each hit in a non-`Leader` function was checked for its base register; the false positives are listed under each leaf.
- **Block writers that touch every leaf** (stated once here): `Leaders::clear (006ed5d0)` memsets `LeaderData[0 .. 0x6929)` to 0 for all 10 slots (`push 0x6929; lea -0x6c14(%esi)`, `esi` starting `0xe40fa4`); `LeaderData::walk_data (006d6750)` walks the byte range `[who, field_0x692a)` as one block when `leader_flags & 1` (save/load and the sync walk); `Game::run_oos_recovery` and `Game::close` hit the scan only at displacements on other bases (not read further, see confidence).

## `action_declare`'s arguments, read at every call site

`Leader::action_declare (006dab50)` is `__thiscall (this = declarer, target, level, quiet, force)`; the listing names them `0x8(%ebp)` .. `0x14(%ebp)`. Arguments pushed right to left, read from the two to six instructions before each `call 0x6dab50` (16 sites):

| caller | target | level | quiet (`+0x10`) | force (`+0x14`) |
|---|---|---|---|---|
| `Object::take_damage` (6522ed) | `ebx` | 0 | `~(flags >> 2) & 1` of the declarer, i.e. 1 unless HUMAN | 0 (`eax` = the 0 `is_enemy` returned) |
| `Leader::diplomacy` 6be1ef, 6be210, 6bed2f, 6c04df | `esi` | 0 | 1 | 0 |
| `Leader::diplomacy` 6beceb, 6bed1c | `esi` | 1 | 1 | 0 |
| `Leader::action_respond` 6d04ed (the refuse-2 arm: the proposer breaks with a human responder) | responder | 1 | 1 | 0 |
| `Leader::action_respond` 6d1273, 6d1297 (the hire loop) | X | 0 | `~(flags >> 2) & 1` | 0 |
| `Nuke::do_damage` 92c502, 92c71e | | 0 | `~(flags >> 2) & 1` | 0 |
| `CommandPackage::process_declare` 9473aa | packet | packet | **0** | 0 |
| `DropControl::process_drop` 959761, 959850 | `edi` | 0 | 1 | **1** |
| `ScenarioFuncSet::declare_war` 9fbedb | `esi` | 0 | 1 | 0 |

`CommandPackage::process_declare` is reached only from `CommandManager::issue_declare (00942200)`, whose callers are `LeaderOut::issue_declare (006d4080)` (from `Options::exec`) and `Popup::on_modal_end` — the interface. So **quiet = 0 only for the interface's declare and for a HUMAN declarer's implicit declares** (take_damage, nuke, a hire). Every computer declaration passes quiet = 1.

The body, in my words (listing 6dab50–6dadb0):

1. `is_enemy(target)` → return 0.
2. Top of a loop: if `level == 0 && force == 0` and `is_ally(target)`: set `level = 1` and go round again (a war on an ally becomes a break to peace). If `level == 0 && force == 0` and not allied: `t = max(this.broke_alliance[target], leaders[target].broke_alliance[this])`; if `t != 0 && frame - t < ally_to_war_delay` → refuse (message to a console declarer), return 1.
3. If `quiet == 0` and `!LeaderData::afford_dow (006d5ce0)` → `LeaderOut::warn_resources`, return 1.
4. If `level == 0` and `!Game::war_allowed (00594670)`: with `force == 0` → `say_no_war`, return 1; with `force != 0` → `level = 1`, round again.
5. **If `quiet == 0`**: `Leader::pay_dow (006d2b10)(target, level)` (and a second `pay_dow(target, 0)` when allied and `level == 0`); then the two counting blocks of `dow`/`blacken` below.
6. **Whatever `quiet`**: if `is_ally(target)` still holds (before the relation changes) → `broke_alliance[target] = frame`.
7. `Leader::set_diplo (006ec6a0)(target, level)`, then the team walk (`ally_diplo` for teammates of either side), then the messages.

