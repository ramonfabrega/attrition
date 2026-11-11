## 17. The spell bits of `tech`

### 17.1 Leaves

- `tech.ptr`, bits `0x275..=0x2ab` (55 spell types), of `tech` at +0x6c0c,
  `BitMask<806>` (head `(806, 101, ptr)`, 101 bytes of bits). The spell bits
  are bytes `0x4e..0x55` of the array. `tech_at_start` (+0x6c80) never holds
  a spell bit.

### 17.2 The rule

**Writer.** One, in a plain game: the spell pass of `Leader::init
(006e3930)`, listing `6e45d9..6e461d`. It runs after the age/tech, building
and unit passes and immediately before `reset_obs_flags` (`6e4621`). It
writes `tech` only.

**Event.** `Leader::init`, once per live slot at setup. Its one plain-game
caller is `Setup::build_game (005ac190)`. `Setup::build_leader (005abc70)`
calls it too, but has no caller and no data reference (dead). The other
callers are out of plain play: `WalkDataGame::walk_data`, `MapScenario::make`,
`Leaders::init`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader` and six sites in `ScenarioRead::load_leader_chunk`.

**Predicate**, for each spell type `t` in `0x275..=0x2ab`, on the player's
`tech` as the earlier passes left it. Set the bit when `type_avail(t, 1)` is
non-zero **and** (`preq[0] != -1` or `spell_flags & 1 == 0`). For a spell,
`type_avail (006e33a0)` is non-zero exactly when both of these pass:

1. `has_preq (006db810)`: every preq slot `q` is `-1`, or `q >= 0` and held.
   A `-2` (`TYPE_ANY_BUILDING`) slot refuses, because `has_tech (006e0c80)`
   answers 1 for `-1` only.
2. `type_eligible (006dbd10)`:
   - the tribe test `tribe_can_type (006d9410)` (the `TypeIndex` overload):
     `(1 << (tribe & 31)) & tribe_mask` non-zero. A slot with tribe `-1`
     tests bit 31, which a 24-bit mask lacks, so it fails;
   - no preq slot `< -1`, so a `-2` is refused here a second time;
   - a preq that is a non-bonus, non-epoch tech whose age exceeds the game's
     `ending_technology` refuses.

   A spell then reaches the default tail and answers 4.

**With the shipped rows** (the 55 loaded `SPELLTYPE` records): every
`tribe_mask` is 24 ones (`16777215`), and no row has `spell_flags` bit 0
(letter `a`), so the second conjunct always holds. `preq` is `[-1,-1,-1]` on
46 rows, which are set. It is `[-2,-1,-1]` on eight, which are never set:
`0x276, 0x278, 0x279, 0x27e, 0x282, 0x284, 0x285, 0x287`. `RADARSPELL`
`0x299` has `[548,-1,-1]` (`INDUSTRIAL_AGE` 0x224), so it is set only when
Industrial Age is held at init and `ending_technology` admits it.

**Step.** Bit `t` of `tech` goes 0 → 1, once. `tech_at_start` is not
touched.

**Reset.** `Leader::init`'s own zeroing (its `BitMask<806>::init`, and the
memset of its `param_2 < 0` arm) precedes the pass. Beyond that, only
construction and teardown clear the bits: `LeaderData::LeaderData
(006d7540)`, `Leader::close (006b7f80)`, and the header's `Leaders::clear`
and `walk_data`. Nothing in a plain game clears the bits or re-runs the pass.
**Reaching Industrial Age by research does not set `RADARSPELL`'s bit.**
`Leader::lose_tech (006d2850)` has only unit and tech arms, so it cannot
clear a spell.

**The other setter, dead in a plain game.** `Leader::gain_tech (006dcb60)`
sets any bit unconditionally, and none of its loops reaches `0x275`. The
only plain-game route that gives it a spell is `Build::finished
(00628490)`'s fall-through when `has_spell` is 0 at the finish (`628687`,
`628548`). That needs a spell that was available when queued and
unavailable when it finished. `BuildData::can_make (0062db10)` refuses to
queue an unavailable type. Nothing in a plain game takes availability away:
the tribe and `ending_technology` are fixed, a `-1` preq is always held, and
an age is lost only through `Leader::set_age (006d25a0)`, whose callers are
the console, scenario functions and `UnitBalance::next`.

**Out of plain play:** the cheats `action_cheat_give_techs (00593180)` and
`action_cheat_zero_techs (00593120)`, `ConsoleWin::run_cmd`,
`disable_type_by_tribe (00a006a0)`, scenario, editor and CtW, and
`walk_data`. Both writer nets (the export, and every displacement or
absolute address into the eight slots' `tech` struct in the listing) found
no other writer.

**Readers.**

- `has_spell (006e0bc0)` reads the bit only for a letter-`a` row. Otherwise
  it returns `type_avail`, so with the shipped data it never reads the bit.
- `has_tech (006e0c80)` reads it raw.
- **`Build::do_queue (0061e410)` at `61e501`** reads it for any queued
  type. When the building `is(MISSILESILO 0x208)`, a missile is inside
  (`inside_down` +0x28 not negative), and the head-of-queue type's `tech`
  bit is set, `do_queue` returns without progressing the queue, unless that
  type is `DISBAND` (0x29a). `DISBAND`'s bit is one of the 46, so the named
  exemption is what lets a loaded silo disband. No other spell names the
  silo in its loaded `from`/`from2`/`where`, so today this changes nothing.
  A port of the bits must keep the exemption.
- The rest are interface (`BuildOut::make_options`, `IFace*`,
  `WallOut::draw`, `ConsoleWin::tell_tech`), `save_leaders_chunk`, CtW,
  `log_data` and the checksum. No other sim decision in a plain game reads a
  spell bit.

### 17.3 In the crate

- **No spell pass.** `TechTree::start` (`crates/sim/src/tech.rs:2235`) ends
  its building/unit loop at `:2290–:2320` and calls `self.reset_obs_flags`
  at `:2321`. The pass goes between the two, the original's order.
- **No slot to hold the bits.** `Kind` (`tech.rs:100`) ends at `Gov`
  (`:117`), so `PlayerTech::tech` (`:720`, `Vec<bool>` over tree entries)
  has no spell entry. Either add the 55 to the tree, or keep a 55-bit side
  mask on `PlayerTech`.
- **No inputs.** `orders::SpellType` (`crates/sim/src/orders/order.rs:95`,
  filled at `crates/rondata/src/load.rs:725–740`) carries `flags` but no
  `preq[0..3]` and no `tribe_mask`. Load both from the craft rows.
- **Emission.** `bits_of` (`crates/sim/src/checksum/leaders.rs:704–710`)
  maps through `tree_type_index` (`:524`) and cannot name `0x275+`. With a
  side mask, append `0x275 + i`. Feed the `tech.ptr` mask only, never
  `tech_at_start`.
- **SEAM.** `queue_disband` (`crates/sim/src/cities/queue_disband.rs:172–174`)
  stands in for `queue_here`, `type_avail` and `has_spell` of `DISBAND`, and
  answers yes. That stays correct, because `DISBAND` is among the 46.
- **Trap: keep both `-2` refusals**, or keep one deliberately. `has_preq`
  and `type_eligible` each refuse a `-2` slot, so a port that drops one
  still refuses the eight rows. The two checks are not interchangeable for
  other types.
- **Checksum.** The port removes `("tech.ptr", 2)` from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1839`). It should also turn
  agreeing the two 101-byte `tech` array ranges (one per walked leader)
  that the pinned `LEADERS` tuple (`:1746`) counts among its four parted
  ranges.

### 17.4 Reached by a plain game

Yes. A skirmish against the computer runs the pass for every live leader in
`Setup::build_game`, before the first frame, and sets 46 bits at an Ancient
start. The `RADARSPELL`-set branch needs an Industrial-or-later start (or
`all_techs`), which a lobby option reaches. The `gain_tech` route is dead in
a plain game. The cheats, the console, `disable_type_by_tribe`, scenarios,
the editor and CtW are out of plain play.

### 17.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/spells-*.md`); no diff backs it yet.

Verdicts: 16 confirmed, 3 corrected, 0 open of 19 claims, plus one added
reader (`Build::do_queue`'s silo test). The corrections:

- The tribe test is the `TypeIndex` overload `006d9410`; it computes the
  same thing for a spell, so the rule did not change.
- `Build::finished`'s fall-through is dead in a plain game. This removes
  `gain_tech` from the plain-game writers.
- `Setup::build_leader` has no caller, and `load_leader_chunk` calls
  `Leader::init` six times. This changes the caller list, not the rule.

Confidence is high. The predicate was read from the listing (deciding lines
`6e45fa`, `6e45ff`, `6e460b`), and its inputs are the loaded records of the
run.

**Witness, not a pinned test.** For run 1562's frame-91 state packet
(`~/ron-golden/run1562/map-14/frame-snapshot.bin`), the predicate, evaluated
on the run's own 55 `SPELLTYPE` records and the leaders' held ages,
predicts every spell bit in both live slots (slot 0 tribe 4, slot 1
tribe 11). 46 are set in each, and the nine clear are the eight `-2` rows
plus `RADARSPELL`. No age `0x220..0x226` is held. Bytes `0x4c..0x57` of
`tech.ptr` read `00 00 a0 bc 4b ff ff fd ff 0f 00 00` in both slots.
`tech_at_start` is all zero, and the uninited slots 2 and 3 carry no bit.
The packet was checked against the dump: its 101 bytes, printed as unpadded
decimals, reproduce the frame-91 `bit_values` strings exactly, for `tech`
and `tech_at_start`, in all four `LEADERDATA` blocks of `gamelog_p.txt`
(lines 816919, 829176, 841435, 853480). That is one run, two tribes, an
Ancient start. No test pins it. A port turns it into the leaders line's own
`tech.ptr` diff on every gate.

### 17.6 Not established

- **The `RADARSPELL`-set branch** (Industrial Age held at init). Unobserved.
  A capture with an Industrial-or-later start, diffed on `tech.ptr`, settles
  it.
- **`ending_technology`'s refusal** of a too-late tech preq. No shipped
  spell row has a tech preq other than `RADARSPELL`'s age, and no capture
  limits the ending age. A capture with a restricted ending age settles it.
- **A live slot with tribe `-1`, and a letter-`a` row.** Neither occurs with
  the shipped data. They are read from the listing only.
- **`BuildTypeData::queue_here` for the silo** was not read. "No other spell
  can be queued at a silo" rests on the loaded `from`/`from2`/`where`
  columns. Reading `queue_here` settles it.
- **A malformed lockstep queue order** that bypasses `can_make` and reaches
  `Build::finished`'s fall-through was not traced. Tracing the queue order's
  validation from receipt to `queue_up` settles it.
