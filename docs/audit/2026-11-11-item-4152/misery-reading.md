# Item 4152, family 12 (`misery`) — first reading

- Family: 12, slug `misery`
- Leaves: `pop_issues` (`LeaderData` +0x7e8, int), `misery` (`LeaderData` +0x7ec, int)
- Reader: Opus 5.5 (claude-opus-5-5), 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp/funcs`, `types.txt`), the listing of `riseofnations.exe` (`llvm-objdump`), `crates/sim/src`.

## Ground the offsets stand on

- `types.txt:42086` `Leader` is size `0x6eec` with no members listed; `LeaderData` (`types.txt:42088`) is `0x6ee4`, with `pop_cap` +0x7e4, `pop_issues` +0x7e8, `misery` +0x7ec, `att` +0x7f0. In every `Leader::` method the export's `this->field_0xNNN` is the `LeaderData` offset with **no prefix** (`Leader::calc_pop_cap (006dc490)` reads `this->field_0x8` as `who` and writes `field_0x7e4` as `pop_cap`; `Leader::calc_anti_attrition (006cdcc0)` writes the float `0x43800000` = 256.0 to `field_0x7f4`, `anti_att`'s type). The 8 extra bytes of `Leader` are at its end.
- The array: `Leaders::walk_data (006e38e0)` walks `LeaderData::walk_data` from `&leaders` to `0xe789dc` in steps of `0x6eec` (9 slots), so slot 0's `LeaderData` is at `0xe3a390`; `pop_issues` of slot `i` is `0xe3ab78 + i·0x6eec`, `misery` is `0xe3ab7c + i·0x6eec`.
- Nets run: `grep -- '->pop_issues\b'`, `'->misery\b'` (only `LeaderData::log_data (006e5110)`); `grep 'field_0x7e8\b\|field_0x7ec\b'`; int-index forms `[0x1f8]`..`[0x1fb]` (a `uint*` at `who` or at the base) and short-index `[0x3f4]`..`[0x3f7]` over the 2,530 export files that name a leader; byte-offset forms `+ 0x7e8)`/`+ 0x7ec)`. Listing: every operand `0x7e8(`, `0x7ec(`, `0xe3ab78`, `0xe3ab7c` (non-`ebp`/`esp`), and every *write* whose memory destination has a displacement in `0x7d8..0x7ec` (to catch a base biased by `+8`, `who`), each attributed to its function through `INDEX.tsv`. `0xe3ab7c` (absolute `misery`) occurs **nowhere** in `.text`.
- Non-leader hits, ruled out by the base register's type: `Cliffs::~Cliffs (00436120)`, `GraphicPieces::close (008feed0)`, `GraphicPieces::clear_local_data (0090c670)`, `GraphicPieces::inc_time (0090cac0)`, `GraphicPieces::~GraphicPieces (0090e720)`, `ChatSpace::ChatSpace (0093e8d0)` (each `this` of its own class); `Constants::init (00569a90)`, `Constants::log_data (00570170)`, `ObjectData::train_time (006508c0)`, `TypeData::get_cost (00664090)`, `Tribe::parse (006f0560)` (base `[0xc061e4]`, `GameAccess::constants`: `Constants +0x7e8 mongol_stable_cost`, `+0x7ec mongol_stable_speed`). `Leader::make_stuff (006c8af0)` and `Leader::compute_sites (006cc950)` write `0x7e0(this)`, which is `site_mark`, not a `who`-biased `pop_issues`.

