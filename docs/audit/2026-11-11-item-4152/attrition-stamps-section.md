## 5. The attrition stamps, `att` and `give_att_disabled`

### 5.1 Leaves

- `attrition_stamp`, +0x1f4, `int`, a sim frame.
- `attrition_stamp2`, +0x1f8, `int`, a sim frame.
- `attrition_stamp3`, +0x1fc, `int`, a sim frame.
- `att`, +0x7f0, `int`, the attrition strength, unscaled.
- `give_att_disabled`, +0x7f8, `int`, a 0/1 flag.

`Leader::init (006e3930)` zeroes all five (the stamps at 6e3dda, 6e3de6 and
6e3df1, the flag at 6e498a, `att` at 6e4984 before its recompute at 6e49bc).
The bulk writers apply as in the header. No constant-slot or absolute store
exists for any of them beyond those named below.

### 5.2 The rule

**`attrition_stamp`.** Writers: `Unit::process_attrition (005e11a0)`
(5e1880 peace arm, 5e1868 grace sub-arm, 5e1586 assassin arm), on the
**unit owner's** slot; zeroings in `Game::do_frame (00591ef0)` 59232b and
`Leader::gain_tech (006dcb60)` 6e038a.

- **Event**: a unit's 32-frame refresh in a rival's land, past every
  exemption before the arms (`docs/ATTRITION.md`'s chain).
- **Arms**: peace, when `!is_enemy(owner, tile)` (both `diplos` non-zero) or
  under CtW (`semaphore[2] & 2`) while `!war_allowed`; assassin, when
  `is_enemy`, `team_style == 2` and `LeaderData::get_target(owner)
  (006da000)` is not the tile owner. Both:

      if stamp[owner] == 0 || frame − stamp[owner] > 900: stamp[owner] = frame

  Strict: exactly 900 does not re-arm. The store is for every slot; only the
  message and sound are the console's.
- **The grace sub-arm is dead with shipped data.** It would store `max(1,
  ally_to_war_grace + frame − 900)` after an alliance broken within
  `ALLY_TO_WAR_DELAY`; both constants are absent from `rules.xml` and load
  as −1, so its test never holds. A mod could open it.
- **Zeroings**, `= 0` for every slot with `leader_flags & 1`:
  `Game::do_frame` on the frame where `rush_rules > 8`, `frame ==
  rush_rules[r].minutes·900` (row +0x3c, stride 0x58) and `team_style ∉ {0,
  8, 11}` (after `Leaders::process_all`, before `Objects::process_all`; it
  then declares war); `gain_tech` when a tech reaches the age-based rush end
  (rules 1..8) while war is barred and `Game::teams_locked`.
- **Readers**: its own gate and `say_no_war`'s `stamp == 0` (5e14f9,
  5e1710), `log_data`. No decision.

**`attrition_stamp2`.** `Unit::suffer_attrition (005e1a10)` 5e1b8a stores
`frame` when the owner's stamp is 0 (5e1a25), for every slot; the console
gates only the message. Its two callers are in `Unit::process (00610bc0)`
(610d35, 6117fe): the period tick, and the detected decoy. It stores before
the damage, so a killing tick stamps. Nothing re-arms it (the rush zeroings
write +0x1f4 only): it is the slot's first attrition tick. Readers: its own
gate, `log_data`.

**`attrition_stamp3`.** `Wall::process (00640450)` 640bb7, on the building
owner's slot; `Build::process (0061edf0)` is its one direct caller.

- **Predicate**: `(frame + id) & 15 == 0`; owner's
  `disable_building_attrition` (+0x804) is 0; also `& 31 == 0` unless rush
  rules still bar war; the land owner `t` (the adjacent-ring owner when the
  type's vslot +0x108 answers non-zero) is `≥ 0`, not the owner, not allied.
- **Unstarted site** (`byte 8 & 2` clear): `disband(0)`, then the store with
  **no `stamp3 == 0` test**, so every ghost site removed re-stamps.
- **Started**: `take_damage(8, …)` via vslot +0x16c; non-zero (it died)
  returns with no stamp; else `if stamp3[owner] == 0: stamp3[owner] =
  frame`.

**`att`.** One store, `Leader::calc_attrition (006cdea0)` 6cdfdc:

    if give_att_disabled != 0: att = 0; return
    n = leading run of ATTRITION1..4 (0x2dd..0x2e0) has_preq holds
    x = n > 0 ? attrition_improved[n − 1] : 0      // C+0x1d8; 1, 2, 4, 8
    for each arm that applies, in order:
        x = (100 + pct)·x / 100                    // signed, truncating
        if x == 0: x = 1
    att = x

Arms: Colosseum `has_wonder(0x212)` (C+0x470, 50); Russians
`has_tribe_bonus(13)` (C+0x74c, 100); CtW, `semaphore[2] & 2` and leader
byte +0x6900 non-zero (C+0xa0c); Kremlin `has_wonder(0x21a)` (C+0x510, 100).

- **The floor fires on a base of 0**: `att` is 1 with no step and any
  applying arm, 0 only with neither or under the flag. Port `== 0 → 1`, not
  `.max(1)` (they differ only for `pct < −100`).
- The decompile's `attrition_upgrade[n + 3]` is the same address; the
  0x2ad test in the loop compares the index (733..736) and is dead.
- `has_tribe_bonus` answers 0 with no starting town and `city_num == 0`, so
  a nomad Russian is `att 0` until its first city.
- **When**: in `Leader::calc_unit_stats (006cf970)`, its first call, on the
  `0x4000000` fold of the next `Leaders::process_all`. The bit's setters:
  `gain_tech`'s tail (every tech but two non-tech exits) and 6dd993,
  `Build::activate`, `Build::close`, `Leader::gather`,
  `Leader::calc_gather`, `Leader::lose_tech`. Also at `Leader::init`, and
  eagerly in `gain_tech` (6dd3d6) **only** in the narrow arm (vslot +0x38
  true, techtype +0x14 == 1, civic level == `ctw_wonder_min_civics`).
- **Timing**: a tech a building completes on frame `f` leaves `att` old for
  the rest of `f`, the end-of-frame dump included; the new value appears in
  `Leaders::process_all` of `f + 1`, before any unit reads it.
- **Reader**: `UnitData::get_attrition (00608fd0)` 608fe6, the only indexed
  read, on the **territory owner's** slot: 0 means no attrition, else the
  divisor after the aged-up scaling (`docs/ATTRITION.md`). So Russian land
  bleeds a hostile unit from frame 0. `LeaderData::get_attrition
  (006e0b60)` has no caller.

**`give_att_disabled`.** `ScenarioFuncSet::enable_give_attrition (00a02900)`
(a02926, 0) and `disable_give_attrition (00a02940)` (a02966, 1), for a
1-based slot with `arg − 1 < 8`, `leader_flags & 1` and `& 2`, else −1 and
no store. Registered only in `ScenarioFuncSet::init_funcs (009c7570)`;
neither recomputes `att`. `Leader::plan_strategy (006b9620)` 6b9bff is a
16-bit store into `reg_land`, **not** a writer (the README's row 5 and
`docs/CHECKSUM.md:1760` are wrong). Readers: `process_attrition` 5e132c on
the tile owner's slot (non-zero exempts every unit on that land at once)
and `calc_attrition` 6cdea6 (`att = 0`). `log_data` does not print it.

### 5.3 In the crate

- **`att`** is `territory.players[w].strength`, written by
  `Sim::calc_attrition` (`crates/sim/src/territory/upkeep.rs:24`) through
  `attrition::strength` (`territory/attrition.rs:42`). Four faults:
  1. **Trap**: `if tech_steps == 0 { return 0; }` (`attrition.rs:43–45`) is
     wrong, as is `docs/ATTRITION.md` §"1. Strength" step 1. Every Russian
     with a city reads 0 here and 1 in the original.
  2. `StrengthMods` sets `russian` only (`upkeep.rs:43–46`). Colosseum and
     Kremlin can use `tech.rs:600`/`:611` and `Holdings::wonders_held`
     (`economy/holdings.rs:572`); the doc comment at `:20–22` is stale.
  3. **Trap, one frame early**: `Sim::apply_gained` (`tech/gain.rs:601`)
     calls it eagerly at `:608`; `Sim::calc_unit_stats`
     (`economy/rares.rs:679`) does not, so a wonder never recomputes it.
     Drop `:608`; call it first in `calc_unit_stats`, on the fold the crate
     already raises for a wonder (`cities/city.rs:2491`, `:2887`); keep an
     init call.
  4. No `give_disabled` short-circuit.
- **`give_att_disabled`** is `PlayerState::give_disabled`
  (`attrition.rs:257`, default `:272`), read in the original's order at
  `:498`. Nothing writes it; the writer is the trigger track's.
- **`attrition_stamp`**: no store. It goes beside the `Situation` in
  `Sim::process_attrition` (`upkeep.rs:239`), on the `peace_attrition` /
  `assassin_attrition` outcome; `Situation.assassin` is hard-coded `false`
  (`:331`). The timed zeroing is `Sim::end_the_peace`
  (`leaders/diplomacy.rs:74`). **Trap**: its comment at `:87–88` names the
  store `update_leader_list` (that is `ScenarioEditor +0x1f4`); the base is
  the leader slot, so it is this leaf. `docs/AI.md` repeats the mislabel.
  The age zeroing is new in the `gain_tech` port.
- **`attrition_stamp2`**: `Sim::suffer_attrition` (`upkeep.rs:286`) already
  runs `warned_at(who) == 0 → stamp_warned(who)` in place; the port fills
  the stubs (`territory/suffer_view.rs:152–155`) with a per-leader word.
- **`attrition_stamp3`**: both arms of the enemy-territory block in
  `Sim::process_building` (`cities/city.rs:2947`, ~2997–3010). The crate
  runs the 32-frame cadence only (`cities/build.rs:1083`).
- **Checksum** (from the code, not the verdicts): leaves at
  `crates/sim/src/checksum/leaders.rs:69–71, 150, 152`, no value arm; each
  `(unheld)`, count 2, in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1757, 1759–1761, 1791`); each
  port removes its entry.

### 5.4 Reached by a plain game

- `att`: yes, from frame 0 for a Russian with a city; Colosseum, Kremlin
  and the steps when reached. The CtW arm is not.
- `attrition_stamp2`: yes, any slot's first attrition tick.
- `attrition_stamp3`: yes, a started building on ground a rival's border
  took; a ghost site needs a border over an unstarted foundation.
- `attrition_stamp`: the peace arm and timed zeroing need two leaders at
  peace (the "peace pair", `RUSH_RULES 10`, `TEAM_STYLE 1`); the age
  zeroing `RUSH_RULES` 1..8; the assassin arm `TEAM_STYLE 2`; the pre-war
  arm CtW; the grace sub-arm a mod.
- `give_att_disabled`: no, triggers only; 0 for a plain game.

### 5.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/attrition-stamps-*.md`); no diff backs it
yet. Both writer nets were re-run for all five leaves and found no writer
either reading missed.

| leaf | confirmed | corrected | open |
|---|---|---|---|
| `attrition_stamp` | 13 | 1 | 0 |
| `attrition_stamp2` | 6 | 1 | 0 |
| `attrition_stamp3` | 7 | 0 | 0 |
| `att` | 12 | 2 | 1 |
| `give_att_disabled` | 6 | 1 | 0 |

Corrections that changed a rule: the zero-base floor; `att` recomputed on
the next fold, not on the gain; the grace sub-arm dead with shipped data;
`attrition_stamp2`'s crate site is the stubs; `give_disabled` exists;
`plan_strategy` is not a writer. Confidence is high on every leaf (stores,
gates and arithmetic read in the listing).

**Witnesses, not pinned diffs.** run981's start dump
(`~/ron-golden/run981/map-7/gamelog.txt`, Ancient start) prints the Russian
(who 1, tribe 13, `city_num 1`, `tech_frame 0`) at `att 1` (line 822197)
and the Nubian (who 0) at `att 0` (809939), alike in all five dumps to
frame 37; only `n = 0` with the Russian arm and the floor fits. Later
Russian values (2 on run1257, run1148, run1400, run1467; 4/8/16/32 after)
fit the formula. run1072's `attrition_stamp` 1392 on block 1393
(`docs/RUNS.md`) witnesses the peace arm.

### 5.6 Not established

- **`ctw_attrition`'s shipped value** (C+0xa0c; the one open verdict,
  unreachable in v1): read it from `rules.xml`.
- **`gain_tech`'s `russian_attrition_upgrades` arm** (C+0x750), not read to
  the end; it explains run1257's Russian `att 2`. Read that arm.
- **The `0x4000000` setters 5e56cc, 66aee6, 66aef6**, unattributed; they
  only make the refresh more frequent.
- **The one-frame lag in a dump.** The verdicts cite an addendum the file
  does not contain. Read `att` on `f` and `f + 1` where a building
  completes an ATTRITION step.
- **Russian land bleeding from frame 0**: no capture with a foreign unit in
  Russian land before a step was looked up.
- **The assassin arm, the age zeroing, the timed zeroing's 0**: no witness
  (frame 4500 is outside run1072's window). A peace-pair capture across
  4500 and a `TEAM_STYLE 2` capture settle them.
- **`attrition_stamp3`'s 16-frame mask**: decompile only, and unmodelled.
- **No value diff for any leaf**: a `LEADERS` capture diffed on the five
  leaves (run981 for the floor) would make them one.
