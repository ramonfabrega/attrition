# Item 4152, family 16 (`lobby-words`): adjudication

- Slug: `lobby-words`. Leaves: `cannon_time` (`LeaderData`+0x48, ulong),
  `handicap` (+0x4c, int), `multi_diff` (+0x50, int).
- Readings: `lobby-words-reading.md` (first), `lobby-words-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep, and the kept
  dumps' `GAME INFO` lines. Nothing run of the original.
- The spawn's question: the readings were summarised as differing on
  `multi_diff`'s plain-game value (the lobby difficulty for every seated
  slot, against 0). Settled first, below, then leaf by leaf.

(Appended per leaf as each is settled.)
