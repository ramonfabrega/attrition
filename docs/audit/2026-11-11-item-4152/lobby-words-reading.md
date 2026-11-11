# Family 16, `lobby-words` — first reading

Item 4152, wave 1. Leaves: `cannon_time` (`LeaderData`+0x48, ulong), `handicap` (+0x4c, int), `multi_diff` (+0x50, int). Reader: Claude Opus 5.5, 2026-11-11.

`Leader` is 0x6eec and `LeaderData` 0x6ee4 (`types.txt` lines 42086/42088): a `Leader*` reaches a `LeaderData` leaf at off+8 (vtable and one more word ahead of the data). The leaders array's `LeaderData` for slot i is at 0xe3a390 + i*0x6eec, so the leaves sit at 0xe3a3d8 / 0xe3a3dc / 0xe3a3e0 + i*0x6eec.

(Sections are appended per leaf as each is finished.)

