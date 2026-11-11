# Item 4152, family 14 (`queued`): first reading

- **Family**: 14, slug `queued`.
- **Leaves**: `scholar_militia` (`LeaderData` +0x9f0); `barracks_queued`
  (+0xa10), `stable_queued` (+0xa14), `factory_queued` (+0xa18),
  `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued` (+0xa24);
  all `int`.
- **Reader**: Opus 5.5 (claude-opus-5-5), wave 1 first reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), the listing
  of `riseofnations.exe` (`llvm-objdump`), and `crates/sim`.

`Leader` is 0x6eec bytes and `LeaderData` 0x6ee4 (`types.txt`), so a
`Leader*` reaches a `LeaderData` field at that offset + 8: the family sits at
`Leader` +0x9f8 (`scholar_militia`) and +0xa18 … +0xa2c (the six queues).
In the global array the slot-`i` `LeaderData` is at `0xe3a390 + i*0x6eec`.

