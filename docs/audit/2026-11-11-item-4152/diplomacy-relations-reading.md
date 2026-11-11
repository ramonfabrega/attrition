# Item 4152, family 7 — `diplomacy-relations`: first reading

- **Family**: 7, `diplomacy-relations`.
- **Leaves** (`LeaderData` offsets; `int[8]` unless named): `good_deeds`
  (+0xd4), `blacken` (+0x20c, `int`), `dow` (+0x270), `broke_alliance`
  (+0x2b0), `made_peace` (+0x2d0), `last_spoke` (+0x2f4), `hire_stamp`
  (+0x1b4), `hire_who` (+0x1d4).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), first reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), the listing
  of `riseofnations.exe` (`llvm-objdump`), `types.txt`. `Leader` is
  `0x6eec`, `LeaderData` `0x6ee4`.

## Ground the whole family stands on

- **`Leader` puts `LeaderData` at offset 0.** Every `Leader` method in the
  export prints these leaves at their raw `LeaderData` offsets
  (`Leader::init (006e3930)` line 153, `&this->field_0xd4`), and
  `Leaders::clear (006ed5d0)` clears from the slot base. So a store off a
  `Leader*` reads `0x<off>(%reg)`, and one off the global array reads
  `0xe3a390+off(...)`.
- **`diplos` (`+0x74`): 0 war, 1 peace, 2 alliance.** `LeaderData::is_enemy
  (006ebaa0)` is true when either side holds 0; `LeaderData::is_ally
  (006edb50)` needs 2 on both sides, or the leader's own slot.
- **`leader_flags & 4` is "human".** `LeaderData::is_human (006ec170)`
  returns exactly that bit. That matters for every leaf below:
  `action_declare`'s third argument is `~(flags >> 2) & 1` at four call
  sites, so it is **1 for a computer and 0 for a human**. (Side note for
  the adjudicator: `crates/sim/src/combat/take_damage_view.rs:32` names
  that bit `LEADER_OUT`, "the player is out". It is `is_human`. That
  file's arithmetic is unchanged by the name.)
- **`ALLY_TO_WAR_DELAY` and `ALLY_TO_WAR_GRACE` load as −1 from shipped
  data.** Both come from `Constants::get_item (0057fa60)`, which returns
  `get_attrib_num(…, -1)` when the tag is missing (`Constants::init
  (00569a90)` lines 2398–2401). The two names appear in the install only in
  `internal_strings*.xml`, never in a rules file (grep of
  `game/data/*.xml`). With a delay of −1, every
  `frame − stamp < delay` test is false. This agrees with
  `crates/sim/src/territory/attrition.rs:431–435`.
- **Method.** Absolute stores were found by scanning the full listing
  (`llvm-objdump -d` of the executable) for every displacement inside each
  leaf's `0xe3a390+off` range. Register-relative stores were found by
  scanning every function the export types with a `Leader*`, `LeaderData*`
  or `LeaderOut*` (937 functions) for displacements in the leaf's range.
  Each hit off a register was then checked against the function to see
  whether the base is a leader. The decompile was grepped for both
  `field_0x<off>` and the PDB name.

