# diplomacy-relations: blind second reading (item 4152, wave 2)

- Slug: `diplomacy-relations`
- Leaves (LeaderData offsets): `good_deeds` +0xd4 int[8], `hire_stamp` +0x1b4 int[8], `hire_who` +0x1d4 int[8], `blacken` +0x20c int, `dow` +0x270 int[8], `broke_alliance` +0x2b0 int[8], `made_peace` +0x2d0 int[8], `last_spoke` +0x2f4 int[8]
- Reader model: Claude Opus 5.5 (claude-opus-5-5)
- Sources: decompile export `~/ghidra-projects/decomp`, PE listing (llvm-objdump of `riseofnations.exe` .text), `crates/sim`. Blind: nothing under this audit directory but this file was opened.

