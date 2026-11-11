# combat-rates: second (blind) reading

- Slug: `combat-rates`
- Item: 4152, wave 2
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading
- Leaves (`LeaderData`, size `0x6ee4`):
  `frame_battle` (+0xa4c, int);
  ushort: `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).

Sources: the decompile export at `~/ghidra-projects/decomp`, the listing at
`tmp/r4152/aa/exe.s` (llvm-objdump of `.text`), `types.txt`.

## 0. Shared facts (every leaf below cites these)

**F1. `Leader*` is `LeaderData*`.** `Leader` (0x6eec) is `LeaderData`
(0x6ee4) at offset 0 plus a vbtable pointer at `+0x6ee4`
(`Leader::Leader (006d2bf0)` stores `&_vbtable_` at `field_0x6ee4`, then
runs `LeaderData::LeaderData` on `this` itself). So a `Leader` method's
`field_0xa58` is `LeaderData +0xa58` without a shift. No class in the
family has a vftable (`vtables.txt` has none for `Leader`/`LeaderData`).
The global slot base is `0xe3a390`, stride `0x6eec`, so the leaves sit
at `0xe3addc`..`0xe3adf7` absolute.

**F2. Search nets run.**
- Export, by name: `->frame_battle` etc. matches only
  `LeaderData::log_data (006e5110)` (13 lines, all `Log::say`).
- Export, by offset: `field_0xa4c`..`field_0xa66` in `Leader/*` and the
  combat callers: `Leader::process_event_frame (006ec180)`,
  `Leader::init (006e3930)`, `Leader::inc_{death,kill,hit,damage}_event`,
  `Object::do_damage (0064a480)`, `Object::take_damage (00652020)`,
  `SpellType::cast_sabotage (006720e0)`, `IFaceMainBase::draw_timers
  (0080b060)`. Other hits (`TerrainOut::one_time_init`, `Unit::think_carry`,
  `Army::*`, `Leader::plan_strategy` at `field_0xa68`) are other types or
  the next leaf (`strategy` at `+0xa68`).
- Listing (`exe.s`), absolute: `0xe3addc`..`0xe3adf7` occur 4 times, all
  read-modify-writes: `64a91e` (`addw` to `+0xa5e`), `64bbe3` (`+0xa58`),
  `653033` (`incw +0xa5a`), `672228` (`incw +0xa5a`).
- Listing, register-relative, both the exact offsets and the offsets
  shifted by −8/−4 (a `leader+8` base, as `process_event_frame`'s own
  scan uses): 136 stores at `0xa44`..`0xa67(%reg)`. Mapped through
  `INDEX.tsv`, the only `Leader`-typed ones are in the functions named
  above (plus `Leader::gain_tech`, which writes `+0xa48`
  `gov_hero_frame`, not a leaf). The rest are `ConquestSetupWin`,
  `SetupWin`, `Constants::init`, `GraphicPieces`, `TerrainMap`,
  `TerrainOut`; `Leader::do_gather`/`Leader::gain_tech`'s `0xa60(%reg)`
  reads are off `0xc061f0` (the constants block), not a leader.
- The four `Leader::inc_*_event` methods have **no caller** (no `call`
  to their address in `exe.s`, no slot in `vtables.txt`): `-Ob2` inlined
  them at every site, so the inlined `addw`/`incw` sites are the writers.
- Whole-record writers: `LeaderData::walk_data (006d6750)` hands the
  byte range `[+0x8 who, +0x692a)` to the `DataWalk` as one block (when
  `leader_flags & 1`), which on input (`load`) overwrites every leaf here;
  `LeaderData::LeaderData (006d7540)` touches none (static storage, zero).
  No `rep movs`/`memcpy` of `0x6ee4`/`0x1bb9` exists in `.text`.

**F3. The window step: `Leader::process_event_frame (006ec180)`.**
Called once per sim frame from `Game::do_frame (00591ef0)` (`5924ac`), in a
loop over slots 0..7 (`0xe3a390`..`<0xe71af0`) for each slot with
`leader_flags & 1`, after `Objects::process_all`, `Objects::inc_time`,
`Leaders::end_process_all` and `Achieve::capture_data`, and **before**
`game->frame += 1` (`5924bf`). Slots 8 and 9 never run it. It returns at
once unless `frame % 50 == 0` (`idiv` by `0x32`, `6ec1a2`). At 15 sim
frames a second (`docs/FORMATS.md:757`) that is every 3⅓ s. On a window
frame, in this order (listing `6ec1bb`..`6ec507`):
1. `X_fifteen += X_current` for X = deaths, damage, kills, hits
   (16-bit `addw`, wraps mod 65536).
2. `X_current *= 100`, stored back truncated to 16 bits (`imul $0x64`,
   `movw`). All four products are computed from the pre-multiply values.
3. For each pair (deaths→`average_death_rate`, kills→`average_kill_rate`,
   hits→`average_hit_rate`, damage→`average_damage_rate`, in that order):
   `v` = the truncated product. If `v == 0` (16-bit `test`):
   `avg = (avg * 7) >> 3`; else `avg = (v + avg) >> 1`, the sum in 32
   bits of two zero-extended ushorts, stored as 16 bits (cannot overflow:
   ≤ 131070 >> 1).
4. Console-only music: if `who == console->who`, the JukeBox mood is
   chosen from `average_hit_rate + average_damage_rate` (thresholds 300,
   600, 2000) and the team scores of other in-game slots whose
   `average_hit_rate` or `average_damage_rate` is non-zero. Reads only;
   writes `juke_box.next_mood` and `0xecba2c`, never a leaf.
5. The battle event (F4).
6. Clear: two 32-bit stores of 0 at `+0xa58` and `+0xa5c` (`6ec501`,
   `6ec507`) zero all four `*_current_frame` leaves. Reached on every
   window frame, whether or not F4 fired.

So the "current_frame" leaves are **counts over the 50-frame window**,
and the ×100 values of step 2 never survive the function.

**F4. The battle event** (`6ec452`..`6ec4f9`). `age` = the leader's own
age, `leaders[who].data_encrypted->+0xdc ^ 0x62766` (`0xe41248` =
`0xe3a390 + 0x6eb8`). With `D = average_death_rate`,
`K = average_kill_rate` (zero-extended), all comparisons signed 32-bit:
- gate: `D + K >= (age + 1) * 125` **and**
  (`frame_battle == 0` **or** `frame - frame_battle >= 1800`);
  1800 frames = 120 s.
- if `D >= K + 10*age + 20`: `Achieve::add_event(1 = ACHIEVEEVENT_BATTLE_DEFEAT,
  who, "")`;
- else if `K >= D + 10*age + 20`: `Achieve::add_event(0 = ACHIEVEEVENT_BATTLE_VICTORY,
  who, "")`;
- else: nothing, and nothing below runs (straight to the clear).
- after either event: one 32-bit store `0xfc18fc18` at `+0xa50`, so
  `average_death_rate = average_kill_rate = 0xfc18` (**64536** as the
  ushort the PDB declares; every later read is `movzwl`), and
  `frame_battle = game->frame` (the pre-increment frame, a multiple of 50).
- Enum values from the PDB (`LF_ENUMERATE ACHIEVEEVENT_BATTLE_VICTORY = 0`,
  `_DEFEAT = 1`, `_TIE = 2`). `Achieve::add_event (007af660)` appends to
  the per-player achievement timeline (`achieve.events[who]`).
- Consequence of the 64536 sentinel (arithmetic only, not observed): with
  no fighting both averages decay together (56469, 49410, …; 525 after
  the 36 windows the 1800-frame gate waits), so `D + K` still clears
  `(age+1)*125` for every age ≤ 7, and only the difference decides. A
  window with deaths but no kills halves `D` toward the new value while
  `K` keeps 7/8 of 64536, which opens a large gap in `K`'s favour.

**F5. The event writers** (the inlined `inc_*_event` bodies, all 16-bit
`addw`/`incw`, wrapping):
- **W1 hit/damage**, `Object::do_damage (0064a480)` at `64a90e`/`64a91e`.
  Step `s = 5` if the attacker's `ptype` answers vslot `+0x10c` non-zero
  **and** the target object answers vslot `+0x1c` non-zero, else `s = 1`.
  PDB slot names (`LF_ONEMETHOD` vftable offsets): `ObjectTypeData +0x10c`
  = `is_siege`; `SubObjectData +0x1c` = `is_wallbuild` (Build and Wall
  answer 1 through the folded `mov eax,1` stub; Unit answers 0).
  `hits_current_frame[attacker.who] += s` and
  `damage_current_frame[param_2] += s`, `param_2` the target's owner.
  Predicate: every call that passes the head's two returns
  (`param_6 < 1`, the number of guys/count; an attacker that `is_unit`
  with `unit_masks & 1`, the decoy bit) — so friendly fire, a hit on
  one's own object and a hit on gaia all count, and it runs before the
  `who != param_2` treaty marking and before `take_damage`.
- **W2 structure kill**, `Object::do_damage` at `64bbdc`/`64bbe3`. After
  vslot `+0x16c` (`Object::take_damage`) returns non-zero **and** the
  target `is_wallbuild` (else the function returns at `64c86b`):
  `kills_current_frame[attacker.who] += k`,
  `deaths_current_frame[param_2] += k`, where `k = 5` if the target, read
  **before** the take (`64a806`..`64a828`), had `flags & 1`, `is_build`
  (vslot `+0x20`) and `is_active` (vslot `+0x4c`), else `k = 2` (a wall,
  or a building not active). No ally, unit, splash or air gate. The kill
  arm also includes `take_damage`'s alive-path `return 1` (a non-active
  wall/build of a leader without flag 4, `damage*2 >= max`, type flag
  `0x2000`, disbanded).
- **W3 unit kill**, `Object::take_damage (00652020)` at `653026`/`653033`,
  on the death path (`damage >= vslot 0x11c` max hits). If the dying
  object `is_wallbuild`: the razed counter `+0x81c`, no leaf. Else, all of:
  vslot `+0xe8` non-zero (`UnitData::is_captain (0046ceb0)`, inlined when
  the slot matches: `(short)o_up < 0`; the map names no other symbol at
  that address), `UnitData::curr_uber_size(0) == 1` (`0060a760`), and
  `unit_masks & 1 == 0`; then `+0x810 += 1`; then, only if `param_7 >= 0`
  (the attacker's object index): `deaths_current_frame[this.who] += 1`,
  `kills_current_frame[param_8] += 1`, `param_8` the attacker's owner
  (`ebx` = `0x24(%ebp)`). Every other caller of `take_damage`
  (`Unit::suffer_attrition`, `Wall::process`, `SpellType::cast_sabotage`,
  `ScenarioFuncSet::set_object_health`, `ScenarioEditor::dec_selected_health`)
  passes `param_7 = -1`, so only `do_damage`'s take, and `take_damage`'s
  own decoy-overflow re-call (`param_7` = the decoy's captain, `param_8` =
  its own `who`, so a kill there credits the owner's own kills), count.
- **W4 sabotage of a unit**, `SpellType::cast_sabotage (006720e0)` at
  `67221c`/`672228`: target with `flags & 1`, not `is_wallbuild`,
  `is_unit`, `is_captain`, `curr_uber_size(0) == 1`, `unit_masks & 1 == 0`
  → `deaths_current_frame[param_4] += 1` (target owner),
  `kills_current_frame[param_2] += 1` (caster owner), then `+0x80c += 1`
  and `+0x44 += cost/5`. Its later `take_damage` passes `param_7 = -1`,
  so no double count.

**F6. Reset: `Leader::init (006e3930)`** zeroes all thirteen leaves
(32-bit stores at `+0xa50`, `+0xa54`, `+0xa58`, `+0xa5c`, `+0xa60`,
`+0xa64` from `eax = 0`, and `+0xa4c` from `ecx = 0`; `6e3b7a`..`6e3bda`).
Callers: `Leaders::init (006ed850)` (from `Game::init_rules_and_teams`),
`Setup::build_leader`, `Setup::build_game`, `MapScenario::make`,
`WalkDataGame::walk_data`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader`, `ScenarioRead::load_leader_chunk` (3×).

**F7. Readers outside the writers.** `LeaderData::log_data (006e5110)`
logs all thirteen; `LeaderData::walk_data` puts them in the walked block
(so in the sync checksum's walk and the save; `crates/sim/src/checksum/leaders.rs:264`-`:276`
lists them inside `[+0x8, +0x692a)`); `IFaceMainBase::draw_timers
(0080b060)` draws the console player's four averages and
`deaths/kills_current_frame * 100` (UI). Nothing in the sim reads the
`*_fifteen_seconds` leaves at all.

**F8. Crate state.** No leaf has a field in `crates/sim` beyond the
checksum table (`checksum/leaders.rs:264`-`:276`) and rondata's
`(unheld)` rows (`crates/rondata/src/diff/checksum_lines.rs:1762`-`:1804`).
The per-leader record a builder would extend is `crate::ai::Leader`
(`crates/sim/src/ai.rs:637`), which already holds the neighbours
`frame_attacked` (+0xa40) and `attacked_by` (+0xa44) at `ai.rs:697`, set
at `ai.rs:901`. Sites:
- W1: `Sim::do_damage_round`, `crates/sim/src/combat/fight/delivery.rs:70`,
  beside `self.treaty_on(hit_by, hit_who, 2)` (`:108`), which the original
  runs right after the increment; `ap.siege` and `Obj::Building` are at
  hand. The crate's head (`count < 1`, decoy) matches the original's two
  returns, but it also returns on `!self.struck(target)`, which the
  original's increment does not see — check before placing.
- W2: `do_damage_round`'s `if let Obj::Building(b) = target { if killed`
  (`delivery.rs:212`-`:215`), but **outside** its unit/splash/air/ally gate;
  `k` needs the target's `active` read before the take.
- W3: `Sim::take_damage_typed`, `delivery.rs:469`, the `Taken::Died` arm
  of `Obj::Unit` (`:530`ff.), gated on captain, squad size 1, not decoy,
  and `_by` being a real attacker (the attrition path never comes here).
- W4: `Sim::cast_sniper_unit`, `delivery.rs:806`, whose SEAM (`:803`)
  already names `+0xa58`, `+0xa5a`, `+0x80c`, `+0x44`.
- F3/F4: no `process_event_frame` exists. It belongs in `Sim::tick`,
  `crates/sim/src/core/frame.rs:73`, just before `self.frame += 1`
  (`:475`), for slots 0..7 with `leader_flags & 1`. `Achieve` is unported.

**F9. Plain game.** W1, W2 and W3 run on every combat blow and kill of a
skirmish against the computer. W4 needs a sabotage cast on a unit (the
crate calls this arm the Sniper's, `delivery.rs:798`); a plain game can
reach it, rarely. F3/F4 run every 50 frames for every live slot. F6 runs
at every game setup. None sits behind triggers, CtW or multiplayer.

## 1. `frame_battle` (+0xa4c, int)

1. **Writers.** `Leader::process_event_frame (006ec180)` at `6ec4f9`
   (`movl %eax, 0xa4c(%edi)`, `edi` = `this`); `Leader::init (006e3930)`
   at `6e3bda` (0); `LeaderData::walk_data (006d6750)` on load (block).
   No other store at `+0xa4c` (or `+0xa44` off a `+8` base) in a
   `Leader`-typed function; the export's other `field_0xa4c` hit,
   `TerrainOut::one_time_init`, is a `TerrainOut`.
2. **Event.** Set to the current sim frame (pre-increment, a multiple of
   50) when F4's battle event fires, i.e. a DEFEAT or VICTORY event was
   added. Predicate as F4: the leader is one of slots 0..7 with
   `leader_flags & 1`, `D + K >= (age+1)*125`, the previous event is
   ≥ 1800 frames old or none has fired, and one average leads the other
   by ≥ `10*age + 20`. Units: frames.
3. **Reset.** `Leader::init` (0); load.
4. **Readers.** `process_event_frame`'s gate (`6ec485`: zero, or
   `frame - frame_battle >= 0x708`); `log_data`; the checksum walk.
5. **Crate.** None. `crate::ai::Leader` (`ai.rs:637`), beside
   `frame_attacked`; written by a new `process_event_frame` in `Sim::tick`
   (`core/frame.rs`, before `self.frame += 1` at `:475`).
6. **Plain game.** Yes, the first time a leader's averages cross the
   threshold with a clear lead.
7. **Confidence.** High: listing read for the whole function; the age
   source read through `0xe41248` and the `0x62766` key the README names
   for `get_age`. Not read: `Achieve::add_event` past its head (it does
   not touch a leaf).

## 2. `average_death_rate` (+0xa50, ushort)

1. **Writers.** `process_event_frame` at `6ec253` (`movw %cx`) and
   `6ec4e9` (`movl $0xfc18fc18, 0xa50`); `Leader::init` at `6e3b7a`
   (32-bit 0, also clears `+0xa52`); load.
2. **Event.** Every window frame (F3): with `v = (deaths_current_frame *
   100) mod 65536`, `v == 0 → avg*7>>3`, else `(v + avg)>>1`. On a battle
   event (F4) set to 64536. Units: deaths per window ×100 (a death of a
   structure counts 2 or 5, F5 W2).
3. **Reset.** `Leader::init`; load. The F4 store is a reset to 64536,
   not to 0.
4. **Readers.** `process_event_frame` (F4's `D`); `draw_timers`
   (`80b448`); `log_data`; walk.
5. **Crate.** None; `crate::ai::Leader`, updated in the new
   `process_event_frame`.
6. **Plain game.** Yes.
7. **Confidence.** High (listing). Not established: whether the
   designers meant `-1000` (the bit pattern is `(short)-1000`) — the
   binary reads it unsigned, which is what a port must do.

## 3. `average_kill_rate` (+0xa52, ushort)

1. **Writers.** `process_event_frame` at `6ec27b` and the 32-bit
   `6ec4e9` store (upper half); `Leader::init` `6e3b7a` (upper half);
   load.
2. **Event.** As §2 with `kills_current_frame`. On a battle event: 64536.
3. **Reset.** As §2.
4. **Readers.** `process_event_frame` (F4's `K`); `draw_timers`
   (`80b4a6`); `log_data`; walk.
5. **Crate.** None; as §2.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 4. `average_damage_rate` (+0xa54, ushort)

1. **Writers.** `process_event_frame` at `6ec2d3`; `Leader::init`
   `6e3b86` (32-bit, also `+0xa56`); load.
2. **Event.** As §2 with `damage_current_frame` (blows received, step 1
   or 5, F5 W1 — not hit points). No battle-event store.
3. **Reset.** `Leader::init`; load.
4. **Readers.** `process_event_frame`'s JukeBox branch, for the console
   leader (sum with `average_hit_rate` against 300/600/2000) and for every
   other in-game slot (`cmpw $0, 0xa4c(%esi)`, `esi` = slot + 8, so
   `+0xa54`: a slot counts as fighting if this or `average_hit_rate` is
   non-zero); `draw_timers` (`80b504`); `log_data`; walk.
5. **Crate.** None; as §2.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 5. `average_hit_rate` (+0xa56, ushort)

1. **Writers.** `process_event_frame` at `6ec29f`; `Leader::init`
   `6e3b86` (upper half); load.
2. **Event.** As §2 with `hits_current_frame`. No battle-event store.
3. **Reset.** As §4.
4. **Readers.** The JukeBox branch (own, and `cmpw $0, 0xa4e(%esi)` for
   other slots); `draw_timers` (`80b5a6`); `log_data`; walk.
5. **Crate.** None; as §2.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 6. `deaths_current_frame` (+0xa58, ushort)

1. **Writers.** `Object::do_damage` `64bbe3` (W2, `+= k`, victim
   `param_2`); `Object::take_damage` `653026` (W3, `incw`, the dying
   unit's `who`); `SpellType::cast_sabotage` `67221c` (W4, `incw`, target
   owner `param_4`); `process_event_frame` `6ec1ff` (×100, transient) and
   `6ec501` (0); `Leader::init` `6e3b8c`; load. The uncalled
   `Leader::inc_death_event (006dab30)` is the inlined body.
2. **Event.** A structure of this leader destroyed by a blow (+5 an
   active building, +2 a wall or inactive building); a unit squad of this
   leader wiped by a blow whose attacker is a real object (+1, the dying
   figure a captain with squad size 1, not a decoy); a unit sabotaged
   (+1, same unit predicate). 16-bit wrap.
3. **Reset.** Every 50th frame by `process_event_frame` (after it is
   folded into `deaths_fifteen_seconds` and `average_death_rate`);
   `Leader::init`; load.
4. **Readers.** `process_event_frame` steps 1–3; `draw_timers`
   (`80b648`, ×100); `log_data`; walk.
5. **Crate.** W2 at `do_damage_round`'s building `killed` branch
   (`delivery.rs:215`), W3 in `take_damage_typed`'s unit `Died` arm
   (`delivery.rs:530`ff.), W4 at the `cast_sniper_unit` SEAM
   (`delivery.rs:803`). Field in `crate::ai::Leader`.
6. **Plain game.** W2, W3 yes; W4 only on a sabotage of a unit.
7. **Confidence.** High on the sites and steps (listing). Medium on two
   names: `is_captain`'s meaning (body `(short)o_up < 0`, PDB name) and
   `curr_uber_size(0)`, whose body I did not read; the reading "one count
   per squad, at its last figure" rests on those names. Not read: which
   calls reach `take_damage`'s alive-path `return 1` (the disband case).

## 7. `kills_current_frame` (+0xa5a, ushort)

1. **Writers.** `Object::do_damage` `64bbdc` (W2, attacker's `who`);
   `Object::take_damage` `653033` (W3, `param_8`, the attacker's owner);
   `SpellType::cast_sabotage` `672228` (W4, caster `param_2`);
   `process_event_frame` `6ec216` (×100) and `6ec501` (0, upper half);
   `Leader::init` `6e3b8c` (upper half); load.
2. **Event.** The mirror of §6, credited to the attacker. No ally or
   same-player gate anywhere: destroying one's own wall counts both a
   kill and a death to the same leader, and a decoy's overflow that kills
   its captain credits the owner (F5 W3).
3. **Reset.** As §6.
4. **Readers.** As §6 (`draw_timers` `80b6a9`, ×100).
5. **Crate.** As §6.
6. **Plain game.** As §6.
7. **Confidence.** As §6.

## 8. `hits_current_frame` (+0xa5c, ushort)

1. **Writers.** `Object::do_damage` `64a90e` (W1, attacker's `who`);
   `process_event_frame` `6ec21d` (×100) and `6ec507` (0);
   `Leader::init` `6e3b92`; load. Inlined `Leader::inc_hit_event
   (006daa30)`.
2. **Event.** Every `do_damage` blow by this leader's object that passes
   the head's returns: +5 for an `is_siege` attacker type on a wall or
   building, else +1. Counts blows, not damage; includes friendly fire
   and blows that kill.
3. **Reset.** As §6.
4. **Readers.** `process_event_frame` steps 1–3; `log_data`; walk.
5. **Crate.** `Sim::do_damage_round` (`delivery.rs:70`), beside
   `treaty_on` (`:108`). Watch the crate's extra `!self.struck(target)`
   return.
6. **Plain game.** Yes, every fight.
7. **Confidence.** High. Not read: what `is_siege` answers per type
   (`ObjectTypeData`'s body is folded; the crate's `ap.siege` reads
   `unit_flags & 0x20000`, `delivery.rs:262`).

## 9. `damage_current_frame` (+0xa5e, ushort)

1. **Writers.** `Object::do_damage` `64a91e` (W1, absolute
   `0xe3adee(%edx)`, `edx = param_2 * 0x6eec`); `process_event_frame`
   `6ec20c` (×100) and `6ec507` (0, upper half); `Leader::init`
   `6e3b92` (upper half); load. Inlined `Leader::inc_damage_event
   (006daa50)`.
2. **Event.** The same step as §8, added to the **target's** owner.
3. **Reset.** As §6.
4. **Readers.** `process_event_frame`; `log_data`; walk.
5. **Crate.** As §8.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 10. `deaths_fifteen_seconds` (+0xa60, ushort)

1. **Writers.** `process_event_frame` `6ec1c2` (`addw %si`, `si` =
   `deaths_current_frame` before ×100); `Leader::init` `6e3b98` (32-bit,
   also `+0xa62`); load.
2. **Event.** Every window frame, `+= deaths_current_frame` (the raw
   window count). 16-bit wrap.
3. **Reset.** **Only** `Leader::init` and load. Nothing clears it on a
   15-second or any other period: despite the name it is the running
   total since game start, mod 65536, as of the last 50-frame boundary.
4. **Readers.** None in the sim; `log_data` and the walk only.
5. **Crate.** None; a field in `crate::ai::Leader`, stepped in the new
   `process_event_frame`.
6. **Plain game.** Yes.
7. **Confidence.** High for "no other writer": both nets (exact offset,
   −8/−4 shifts, absolute `0xe3adf0`) and the block walkers were run.

## 11. `kills_fifteen_seconds` (+0xa62, ushort)

1. **Writers.** `process_event_frame` `6ec1e5`; `Leader::init` `6e3b98`
   (upper half); load.
2. **Event.** `+= kills_current_frame` per window frame.
3. **Reset.** As §10.
4. **Readers.** As §10.
5. **Crate.** As §10.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 12. `hits_fifteen_seconds` (+0xa64, ushort)

1. **Writers.** `process_event_frame` `6ec1ec`; `Leader::init` `6e3b9e`
   (32-bit, also `+0xa66`); load.
2. **Event.** `+= hits_current_frame` per window frame.
3. **Reset.** As §10.
4. **Readers.** As §10.
5. **Crate.** As §10.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 13. `damage_fifteen_seconds` (+0xa66, ushort)

1. **Writers.** `process_event_frame` `6ec1d0`; `Leader::init` `6e3b9e`
   (upper half); load.
2. **Event.** `+= damage_current_frame` per window frame.
3. **Reset.** As §10.
4. **Readers.** As §10.
5. **Crate.** As §10.
6. **Plain game.** Yes.
7. **Confidence.** High.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `frame_battle` +0xa4c | `Leader::process_event_frame (006ec180)`, `Leader::init (006e3930)`, load | `= frame` on a battle event (F4) | init, load | new `process_event_frame` in `Sim::tick` (`core/frame.rs:475`); field in `ai::Leader` | yes | high |
| `average_death_rate` +0xa50 | `process_event_frame`, `Leader::init`, load | every 50 frames: `v==0 ? avg*7>>3 : (v+avg)>>1`, `v` = deaths×100 mod 2^16; `= 64536` on a battle event | init, load | as above | yes | high |
| `average_kill_rate` +0xa52 | same | same with kills; `= 64536` on a battle event | init, load | as above | yes | high |
| `average_damage_rate` +0xa54 | same | same with damage | init, load | as above | yes | high |
| `average_hit_rate` +0xa56 | same | same with hits | init, load | as above | yes | high |
| `deaths_current_frame` +0xa58 | `Object::do_damage (0064a480)`, `Object::take_damage (00652020)`, `SpellType::cast_sabotage (006720e0)`, `process_event_frame`, `Leader::init`, load | +5 active building / +2 other structure destroyed; +1 squad wiped (captain, size 1, not decoy, attacker ≥ 0); +1 unit sabotaged | every 50th frame; init; load | `delivery.rs:215`, `:530`ff., `:803` | yes (W4 rare) | high (names medium) |
| `kills_current_frame` +0xa5a | same three + `process_event_frame`, `Leader::init`, load | the mirror, to the attacker; no ally gate | same | same | yes (W4 rare) | high (names medium) |
| `hits_current_frame` +0xa5c | `Object::do_damage`, `process_event_frame`, `Leader::init`, load | +5 siege type on wall/building, else +1, per blow, attacker | every 50th frame; init; load | `do_damage_round` beside `treaty_on` (`delivery.rs:108`) | yes | high |
| `damage_current_frame` +0xa5e | same | same step, to the target's owner | same | same | yes | high |
| `deaths_fifteen_seconds` +0xa60 | `process_event_frame`, `Leader::init`, load | `+= deaths_current_frame` every 50 frames; never cleared | init, load only | new `process_event_frame` | yes | high |
| `kills_fifteen_seconds` +0xa62 | same | `+= kills_current_frame` | init, load only | same | yes | high |
| `hits_fifteen_seconds` +0xa64 | same | `+= hits_current_frame` | init, load only | same | yes | high |
| `damage_fifteen_seconds` +0xa66 | same | `+= damage_current_frame` | init, load only | same | yes | high |

All writes to the ushort leaves are 16-bit and wrap. `process_event_frame`
runs for slots 0..7 only, so an object owned by slot 8 or 9, if one
exists (not checked), would accumulate `*_current_frame` there without
ever folding or clearing it.
