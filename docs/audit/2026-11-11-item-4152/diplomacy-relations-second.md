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

