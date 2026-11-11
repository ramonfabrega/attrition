# Item 4152, family 3 (`combat-rates`): first reading

- **Family**: 3, slug `combat-rates`.
- **Leaves** (`LeaderData` offsets; `Leader` has `LeaderData` at +0, so a
  `Leader*` `field_0xNNN` is the same offset): `frame_battle` (+0xa4c, int);
  ushort `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- **Sources**: the decompile export under `~/ghidra-projects/decomp`
  (`funcs/`, `types.txt`, `INDEX.tsv`), and the listing of
  `game/riseofnations.exe` by `llvm-objdump`. Slot `i`'s `LeaderData` is at
  `0xe3a390 + i*0x6eec`.

## 0. The sweep, and the writers it found

**Nets run.** Export: `grep -rn -- '->leaf\b'` for all 13 names finds only
`LeaderData::log_data (006e5110)` (the gamelog printer, a reader). `grep -rl
'field_0x<off>\b'` for the 13 offsets finds `Leader::inc_hit_event
(006daa30)`, `Leader::inc_damage_event (006daa50)`, `Leader::inc_kill_event
(006dab10)`, `Leader::inc_death_event (006dab30)`, `Leader::process_event_frame
(006ec180)`, `Leader::init (006e3930)`, `Object::do_damage (0064a480)`,
`Object::take_damage (00652020)`, `SpellType::cast_sabotage (006720e0)`,
`IFaceMainBase::draw_timers (0080b060)` (reader), and
`TerrainOut::one_time_init (0085c150)` (a `TerrainOut` member at its own
`+0xa4c`, ebx is the `TerrainOut`; not this record). `Leader` is `LeaderData`
at +0 plus 8 trailing bytes (`Leader::inc_hit_event` writes
`this->field_0xa5c`, which is `hits_current_frame`), so no prefix offset
applies.

Listing (`llvm-objdump` of `riseofnations.exe`, `.text`): every instruction
whose destination is `0xa4c`..`0xa67` off a register, or `0xe3addc`..`0xe3adf7`
absolute (slot base `0xe3a390` + offset), every `lea` of those, and the
`# imm` lines. After setting aside other classes by the base register
(`TerrainMap`, `ConquestSetupWin`, `GraphicPieces`, `Constants`,
`TerrainOut::one_time_init`, `SetupWin::do_scen_changes (005b8293)` whose
base is `MiscAccess::connection_data` at `0xc0622c`, and `-0xa4c(%ebp)`-style
locals in `Group::action_alarm`, `Console::execute_at_cursor`,
`Unit::think_scout`, `Options::exec`), the stores into a leader are:

| site | function | store | leaf |
|---|---|---|---|
| `0064a90e` | `Object::do_damage` | `addw ax, 0xa5c(ecx)`, ecx = `&leaders[attacker.who]` | `hits_current_frame` |
| `0064a91e` | `Object::do_damage` | `addw ax, 0xe3adee(edx)`, edx = `param_2*0x6eec` (target's owner) | `damage_current_frame` |
| `0064bbdc` | `Object::do_damage` | `addw ax, 0xa5a(ecx)`, attacker's leader | `kills_current_frame` |
| `0064bbe3` | `Object::do_damage` | `addw ax, 0xe3ade8(edx)`, target's owner | `deaths_current_frame` |
| `00653026` | `Object::take_damage` | `incw 0xa58(eax)`, the dying object's owner | `deaths_current_frame` |
| `00653033` | `Object::take_damage` | `incw 0xe3adea(eax)`, eax = `ebx*0x6eec`, ebx = `param_8` | `kills_current_frame` |
| `0067221c` | `SpellType::cast_sabotage` | `incw 0xa58(eax)`, target's owner `param_4` | `deaths_current_frame` |
| `00672228` | `SpellType::cast_sabotage` | `incw 0xe3adea(esi)`, esi = `param_2*0x6eec` (caster's owner) | `kills_current_frame` |
| `006daa37`, `006daa57`, `006dab17`, `006dab37` | `Leader::inc_{hit,damage,kill,death}_event` | `addw ax, 0xa5c/0xa5e/0xa5a/0xa58(ecx)` | the four `_current_frame` |
| `006e3b7a`..`006e3b9e`, `006e3bda` | `Leader::init` | six `movl 0` at `0xa50`,`0xa54`,`0xa58`,`0xa5c`,`0xa60`,`0xa64` (each a pair of ushorts), `movl 0` at `0xa4c` | all 13 |
| `006ec1c2`..`006ec507` | `Leader::process_event_frame` | the fold (below) | all 13 |

The four `Leader::inc_*_event` bodies are one `addw` each and **have no
direct caller** (no `call 0x6daa30`/`0x6daa50`/`0x6dab10`/`0x6dab30` in the
listing, and none in the export): every live increment is the inlined copy in
`do_damage`, `take_damage` or `cast_sabotage`. `Leaders::clear (006ed5d0)`
also `memset`s the first `0x6929` bytes of every slot, which covers
`+0xa4c..+0xa67`.

### The writers' predicates, once (each leaf points back here)

**W1. `Object::do_damage (0064a480)`, the hit** (`0064a8b3`..`0064a91e`).
`this` is the attacker, `param_1`/`param_2` the target's index and owner,
`param_6` the count. The function returns at once when `param_6 < 1`
(`0064a4ae`) and when the attacker is a unit (vslot `+0x18`) whose
`unit_masks & 1` is set (`0064a4dc`, a decoy) — the decompile drops both
returns (it prints an empty `if (param_6 < 1) {}`); the listing has them.
Between `0064a4e2` and the store there is no exit. So **every other call**
adds `s` to `leaders[attacker.who].hits_current_frame` and the same `s` to
`leaders[target.who].damage_current_frame`, **with no owner predicate**: a
blow on one's own or an ally's object counts on both sides (the store sits
before the `param_2 != attacker.who` test at `0064a934`). `s = 5` when the
attacker's type answers vslot `+0x10c` (`is_siege`, named in
`docs/VISION.md`) **and** the target answers vslot `+0x1c` (a building or
wall); otherwise `s = 1`. Units: event counts, not hit points.

**W2. `Object::do_damage`, the building kill** (`0064ba18`..`0064bbe3`).
After `take_damage` (vslot `+0x16c`, `Object::take_damage` on every object
vtable) returns non-zero (the object died), the function returns at
`0064ba39` unless the target answers vslot `+0x1c` (building or wall). On a
building or wall it bumps `buildings_razed`, does the score and plunder
work, then adds `s'` to `leaders[attacker.who].kills_current_frame` and to
`leaders[target.who].deaths_current_frame`. `s' = 5` when `local_20`
(`-0x1c(%ebp)`) is set, else `2`; `local_20` is computed **before** the
`take_damage` call, at `0064a7f0`..`0064a828`: target `flags & 1` (alive)
**and** target vslot `+0x20` (1 for a `Build`, 0 for a wall; `docs/COMBAT.md`
§12 naming) **and** target vslot `+0x4c` (`is_active`). So a finished
non-wall building is 5, a wall or an inactive building 2. No owner
predicate. A unit killed here is not counted by `do_damage` (it returned at
`0064ba39`); it is counted inside `take_damage` (W3). A city at its ceiling
(`flags & 0x20`, active) makes `take_damage` return 0 (reduced, not
destroyed), so a reduction counts nothing.

**W3. `Object::take_damage (00652020)`, a unit's death** (`00652f75`..
`00653033`). `this` is the struck object, `param_7`/`param_8` the killer's
index and owner. Reached only on the death path (cumulative `damage` at
least the hit points; the not-dead path returns 0 earlier). A building or
wall (vslot `+0x1c`) bumps `buildings_lost` and skips this. For a unit, all
of: `is_captain` (vslot `+0xe8`, inlined as `o_up < 0`, the squad's
captain), `UnitData::curr_uber_size (0060a760)` with 0 equal to 1 (the last
figure), and `unit_masks & 1` clear (not a decoy); then `units_lost++` on the
victim's leader; then **only if `param_7 >= 0`** (a killer object):
`deaths_current_frame[this.who] += 1` and `kills_current_frame[param_8] += 1`
(and `units_killed[param_8]++`). No owner predicate. Callers of vslot
`+0x16c` (all nine call sites in the listing): `do_damage` passes the
attacker's index and owner (`0064ba18`), so every combat kill counts;
`Unit::suffer_attrition (005e1bd1)`, `Wall::process (00640b22)`,
`cast_sabotage (0067275d)`, `ScenarioEditor::dec_selected_health
(009a0246)`, `ScenarioFuncSet::set_object_health (009f5bb0)` and
`ScenarioFuncSet::damage_object (009f5c94)` all push `param_7 = -1`, so
**attrition and the scripted paths count nothing**. The ninth is
`take_damage`'s own tail (`00653426`): the overflow past a dead figure's
share goes to its captain (vslot `+0xe4`, when that is not itself) with
`param_7` = the captain's index and `param_8` = **the victim's own owner**,
so if that overflow kills the captain, the owner is credited with a death
**and a kill of its own**.

**W4. `SpellType::cast_sabotage (006720e0)`, the unit arm** (`006721a8`..
`00672228`). `param_1`/`param_2` the caster's index and owner,
`param_3`/`param_4` the target's. Target alive (`flags & 1`; a dead target
takes the early `mana_burn` refund and returns), not a building or wall
(vslot `+0x1c`), a unit (vslot `+0x18`), `is_captain`, `curr_uber_size(0)
== 1`, not a decoy → `deaths_current_frame[param_4] += 1`,
`kills_current_frame[param_2] += 1` (and `units_killed`, `score_combat +=
cost/5`). The later `take_damage` call in this function passes `param_7 =
-1`, so no double count.

**W5. `Leader::process_event_frame (006ec180)`, the fold.** Called from
`Game::do_frame (00591ef0)` (line 288) for every one of the **8** slots
(`leaders` to `0xe71af0`) whose `leader_flags & 1` is set, after
`Leaders::end_process_all` and `Achieve::capture_data` and **before** `frame
+= 1`. It returns at once unless `game->frame % 50 == 0` (the pre-increment
frame; `0x32`, sim frames). On a fold frame, in this order:
1. `deaths/kills/hits/damage_fifteen_seconds += the matching _current_frame`
   (16-bit adds of the raw event count).
2. Each `_current_frame` is replaced by `(current * 100) mod 65536`
   (`imul 0x64`, stored as a word).
3. Each average, from that 16-bit value `c`: `c == 0` → `avg = (avg * 7)
   >> 3`; else `avg = (c + avg) >> 1`; 32-bit arithmetic on zero-extended
   words, stored as a word. Order: death (`a50`, from deaths), kill (`a52`,
   from kills), hit (`a56`, from hits), damage (`a54`, from damage).
4. For the console's own leader only (`who == console->who`): the jukebox
   mood (reads below; no leaf is written).
5. The battle test, for every folded leader: let `age =
   leaders[this.who].data_encrypted->ages ^ 0x62766`, `D = average_death_rate`,
   `K = average_kill_rate` (zero-extended). If `D + K >= (age + 1) * 125`
   **and** (`frame_battle == 0` **or** `frame - frame_battle >= 1800`, `0x708`
   sim frames): if `D >= K + 10*age + 20` → `Achieve::add_event(1 =
   ACHIEVEEVENT_BATTLE_DEFEAT, who, "")`; else if `K >= D + 10*age + 20` →
   `add_event(0 = ACHIEVEEVENT_BATTLE_VICTORY, who, "")`; else nothing and
   **no** reset. When an event fires: `average_death_rate =
   average_kill_rate = 0xfc18` (64,536 as the unsigned word; one `movl
   0xfc18fc18` over the pair at `006ec4e9`) and `frame_battle = frame`.
   All compares are signed `jl` on non-negative ints, so effectively
   unsigned.
6. Unconditionally on the fold frame: `movl 0` at `0xa58` and `0xa5c` —
   all four `_current_frame` words return to 0 (`006ec4ff`..`006ec507`; `eax`
   is zeroed by the `xorl` at `006ec4ff`, not the frame left in it by
   step 5).

So between folds a `_current_frame` word holds the raw events since the last
fold (its ×100 form exists only inside the fold). `Achieve::add_event
(007af660)` appends `(frame, type)` to `achieve.events[who]` and posts a
debug message; it writes no leader leaf.

## 1. `hits_current_frame` (+0xa5c, ushort)

1. **Writers.** `Object::do_damage (0064a480)` at `0064a90e` (W1); fold
   `Leader::process_event_frame (006ec180)` at `006ec21d` (×100) and
   `006ec507` (zero, as the low half of a dword); `Leader::init (006e3930)` at
   `006e3b92`; `Leaders::clear (006ed5d0)` memset. `Leader::inc_hit_event
   (006daa30)` is the uncalled out-of-line copy.
2. **Event.** Every `do_damage` call past its two head returns: `+= 5` on
   the **attacker's** leader when a siege attacker (`is_siege`, type vslot
   `+0x10c`) strikes a building or wall (target vslot `+0x1c`), else `+= 1`.
   No owner, ally or kind exemption beyond the decoy return; friendly fire
   counts. Not reached by attrition (`Unit::suffer_attrition` goes straight to
   `take_damage`).
3. **Reset.** Zeroed on every fold frame (`frame % 50 == 0`, step 6), by
   `Leader::init` and `Leaders::clear`. Between folds it is the raw count.
4. **Readers.** The fold (step 1 into `hits_fifteen_seconds`, step 3 into
   `average_hit_rate`); `IFaceMainBase::draw_timers (0080b060)` (a debug
   overlay under a scene flag, prints it ×100); `LeaderData::log_data`.
5. **Crate.** `Sim::do_damage_round`, `crates/sim/src/combat/fight/delivery.rs:70`,
   after the decoy return (line ~92). Note the crate also returns on
   `!self.struck(target)` (`combat/fight/stats.rs:92`) before that; the
   original has no target-liveness return ahead of the store, so a counter
   placed there misses any blow the crate refuses on that test. Siege test:
   the attacker profile's `siege` (`ap.siege`, already passed to
   `city_alarm`); building-or-wall: `Obj::Building(_)` (walls' home in the
   crate not checked).
6. **Plain game.** Yes: any skirmish blow.
7. **Confidence.** High: one store, read in the listing with its predicate;
   the head returns read in the listing. Not read: whether any `do_damage`
   caller can pass a building-or-wall target whose vslot `+0x1c` answers 0.

## 2. `damage_current_frame` (+0xa5e, ushort)

1. **Writers.** `Object::do_damage` at `0064a91e` (W1, absolute
   `0xe3adee + param_2*0x6eec`); fold at `006ec20c` (×100) and `006ec507`
   (zero, high half of the dword at `0xa5c`); `Leader::init` at `006e3b92`
   (high half); `Leaders::clear`. `Leader::inc_damage_event (006daa50)`
   uncalled.
2. **Event.** Same call and same `s` (1 or 5) as `hits_current_frame`, on
   the **target's owner** (`param_2`). It counts blows received, not hit
   points. No exemption; a blow on one's own object adds to the same leader's
   hits and damage.
3. **Reset.** As §1.
4. **Readers.** Fold steps 1 and 3 (into `damage_fifteen_seconds`,
   `average_damage_rate`); `draw_timers` does **not** print it (it prints
   `a50`..`a56` and `a58`, `a5a` ×100); `log_data`.
5. **Crate.** Same site as §1: `do_damage_round`, `owner_of(target)` is
   already computed there as `hit_who`.
6. **Plain game.** Yes.
7. **Confidence.** High.

## 3. `kills_current_frame` (+0xa5a, ushort)

1. **Writers.** `Object::do_damage` at `0064bbdc` (W2); `Object::take_damage
   (00652020)` at `00653033` (W3); `SpellType::cast_sabotage (006720e0)` at
   `00672228` (W4); fold at `006ec216` (×100) and `006ec501` (zero, high half
   of the dword at `0xa58`); `Leader::init` at `006e3b8c`; `Leaders::clear`.
   `Leader::inc_kill_event (006dab10)` uncalled.
2. **Event, by writer, on the killer's leader.**
   - W2: a building or wall destroyed by a `do_damage` blow: `+= 5` if it
     was alive, a `Build` (vslot `+0x20`) and `is_active` (vslot `+0x4c`)
     when the blow began, else `+= 2` (a wall, or an inactive building).
   - W3: a unit's death on `take_damage`'s death path, when it was the
     squad's captain (`o_up < 0`), the last figure (`curr_uber_size(0) ==
     1`), not a decoy, and the caller named a killer (`param_7 >= 0`): `+= 1`
     on `param_8`. Combat kills via `do_damage` qualify; attrition, `Wall::
     process`, the sabotage's own `take_damage` and the scenario paths pass
     `-1` and do not. The overflow-to-captain tail credits the victim's own
     owner.
   - W4: Spy/Sniper sabotage killing a single-figure captain that is not a
     decoy: `+= 1` on the caster's owner.
   No owner or ally exemption in any of the three.
3. **Reset.** As §1.
4. **Readers.** Fold steps 1 and 3 (`kills_fifteen_seconds`,
   `average_kill_rate`); `draw_timers` (×100); `log_data`.
5. **Crate.** W2: `do_damage_round` after `take_damage_typed`, inside `if
   let Obj::Building(b) = target { if killed { … } }`
   (`delivery.rs` ~line 211; the `s'` predicate needs the building's
   `active` read **before** the take, beside `fresh`). W3:
   `take_damage_typed` (`delivery.rs:469`), the `Obj::Unit` arm's
   `Taken::Died` branch (~line 532), gated on a real killer (`_by`) and on
   captain/last-figure/not-decoy — but the crate's `take_damage` wrapper is
   also the attrition path's, which the original passes `-1`, so the gate
   must tell those apart. W4: the SEAM already named at
   `delivery.rs:803` (`cast_sniper_unit`); that SEAM says "the caster's
   leader counters (`+0xa58`, `+0xa5a`, …)" — `+0xa58` goes to the
   **target's** leader (`param_4`), only `+0xa5a`/`+0x80c`/`+0x44` to the
   caster's.
6. **Plain game.** Yes for all three (combat kills, razing, a Spy's or
   Sniper's sabotage on a unit). W3's overflow-to-captain arm needs a squad
   whose dead figure is not its own captain.
7. **Confidence.** High for the stores and predicates as read in the
   listing. Not read: which squads reach the overflow tail in practice; the
   vslot `+0x4c` body for `Build` (the export's folded name is
   `WallData::is_active`); whether `cast_sabotage`'s unit arm can reach a
   target whose captain test fails yet still dies (then nothing counts).

## 4. `deaths_current_frame` (+0xa58, ushort)

1. **Writers.** `Object::do_damage` at `0064bbe3` (W2, absolute `0xe3ade8 +
   param_2*0x6eec`); `Object::take_damage` at `00653026` (W3, `0xa58` off the
   victim's `Leader*`); `SpellType::cast_sabotage` at `0067221c` (W4, target's
   owner `param_4`); fold at `006ec1ff` (×100) and `006ec501` (zero); `Leader::
   init` at `006e3b8c`; `Leaders::clear`. `Leader::inc_death_event (006dab30)`
   uncalled.
2. **Event.** The mirror of §3 on the **victim's** leader, the same step in
   each writer (W2: 5 or 2; W3: 1; W4: 1) and the same predicates. In W3 the
   `deaths` store is under the same `param_7 >= 0` test as the kill, so a
   unit lost to attrition increments `units_lost` but **not**
   `deaths_current_frame`.
3. **Reset.** As §1.
4. **Readers.** Fold steps 1 and 3 (`deaths_fifteen_seconds`,
   `average_death_rate`); `draw_timers` (×100); `log_data`.
5. **Crate.** As §3, the victim's side of each.
6. **Plain game.** Yes.
7. **Confidence.** High; as §3.

