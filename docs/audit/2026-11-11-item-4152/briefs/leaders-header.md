# Leaders: the record's held-out leaves

This document specifies the leaves of the leader record (`LeaderData`) that
the leaders line of the sync checksum is held out on
(`docs/CHECKSUM.md` §20; `LEADERS_HELD_OUT` in
`crates/rondata/src/diff/checksum_lines.rs`). Each is a counter, a stamp, a
mark or a computed value that a plain game moves and that the crate does not
hold yet. It is written for the builder who ports them. Each section names
the leaves it covers, the rule, where in `crates/sim` the event already
lives, how the rule was established and how confident it is, and what is not
established.

**Section numbers are an API.** They are the family numbers of
`docs/audit/2026-11-11-item-4152/README.md`, fixed before any reading
began, so a section keeps its number whatever order the sections land in. A
family with no section yet is one whose adjudication has not landed. `explored`
belongs to the fog and is lane 4153's port; it has no section here.

**How it was established, for every section.** Item 4152 (`docs/DECISIONS.md`
entry 124) ran three independent readings per family, all from the decompile
export and the executable's listing:

1. a first reading, `<slug>-reading.md`;
2. a blind second reading, `<slug>-second.md`. Its reader had the leaf names
   and the export, and not the first reading or §20's writer list;
3. an adjudication, `<slug>-verdicts.md`, which settles every disagreement
   back to the decompiled function or the listing, with the deciding line.

The files are under `docs/audit/2026-11-11-item-4152/`, and each section is
written from its family's verdicts alone. **No diff backs any rule here
yet.** Where a kept dump was consulted, the section says so, and treats that
value as a witness, not a pinned diff.

**Conventions.** A function is cited as `Name (00xxxxxx)`. An offset is a
`LeaderData` offset. `Leader` holds `LeaderData` at offset 0 and adds 8 bytes
at the end (the vbtable pointer at +0x6ee4), so a `Leader*`'s `field_0xNN` is
the leaf at +NN. The leaders array's slot `i` sits at `0xe3a390 + i·0x6eec`.
Every value is an integer at the original's own scale, a frame is a sim
frame, and division truncates toward zero unless a section says otherwise.

**Three facts several sections share**, each established by more than one
family's adjudication:

- **Dead bodies, live inlined copies.** Nothing calls `Leader::process
  (006b88b0)`, the four `Leader::inc_*_event`, `Leader::calc_misery
  (006cdff0)` or `Leader::process_taunts (006b8ba0)`. Their stores run as
  inlined copies in `Leaders::process_all (006ed2a0)`, `Object::do_damage
  (0064a480)`, `Object::take_damage (00652020)` and the spells. A port hooks
  the live site, never the named function.
- **The bulk writers.** `Leaders::clear (006ed5d0)`, and `Leader::init
  (006e3930)` when its tribe argument is negative (the `Leaders::init` path),
  memset `[+0, +0x6929)`. `LeaderData::walk_data (006d6750)` restores
  `[+0x8, +0x692a)` raw when a save loads. The sections list these only
  where they matter.
- **The register-relative trap below +0x200.** A displacement there cannot
  be told from another record's. Each family's writers were attributed by
  the base register's type, and the false hits are named in the readings.
