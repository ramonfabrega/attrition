# Item 4152, second (blind) reading: `queued`

- Slug: `queued`
- Leaves (all `int`, `LeaderData` offsets): `scholar_militia` (+0x9f0),
  `barracks_queued` (+0xa10), `stable_queued` (+0xa14), `factory_queued`
  (+0xa18), `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued`
  (+0xa24).
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.
- Sources: the Ghidra decompile export (`~/ghidra-projects/decomp`), the PE
  listing (`llvm-objdump` of `riseofnations.exe`, pre-dumped `.text`), and
  `crates/sim`.

