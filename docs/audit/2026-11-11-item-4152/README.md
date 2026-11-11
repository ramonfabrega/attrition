# Item 4152 — the leaders line's held-out leaves, read

The reader row of `docs/DECISIONS.md` entry 124. The leaders line
(`docs/CHECKSUM.md` §20) is held out on 100 leaves (`LEADERS_HELD_OUT` in
`crates/rondata/src/diff/checksum_lines.rs`). `explored` is lane 4153's port
and is excepted here, which leaves **99 leaves in 20 families**. Each family
gets a first reading (`<slug>-reading.md`), a blind second reading
(`<slug>-second.md`) and an adjudication (`<slug>-verdicts.md`), and ends as a
section of `docs/LEADERS.md`.

**Score: families read / second-read / adjudicated, of 20.** It is kept in
the log at the foot of this file.

## The families

Offsets are `LeaderData`'s (`types.txt`, `struct /rise.pdb/LeaderData`, size
`0x6ee4`). "§20's writers" are the store sites §20 and parked 4124a name,
which come from the listing and are not each read whole. **The blind second
reader gets the leaf column alone.**

| # | slug | leaves (offset, type) | §20's writers |
|---|---|---|---|
| 1 | `city-name` | `city_name` (+0x40c int) | `City::generate_name`: a counter shared by the leaders of one tribe, where the one holding the most takes the increment |
| 2 | `kills` | `units_killed` (+0x80c), `units_lost` (+0x810), `buildings_razed` (+0x818), `buildings_lost` (+0x81c), all int | `Object::take_damage`, `Object::do_damage`, `cast_sabotage`, `cast_counterintel` |
| 3 | `combat-rates` | `frame_battle` (+0xa4c int); ushort: `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52), `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56), `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a), `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e), `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62), `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66) | `Leader::inc_*_event`, folded by `Leader::process_event_frame` |
| 4 | `victim-stamps` | `attack_stamp` (+0xf4 int[8]), `raid_stamp` (+0x114 int[8]), `ally_stamp` (+0x154 int[8]), `ctw_hero_stamp` (+0x200 int) | `Object::do_damage`; `attack_stamp` also `Leader::plan_strategy`, `end_process` |
| 5 | `attrition-stamps` | `attrition_stamp` (+0x1f4), `attrition_stamp2` (+0x1f8), `attrition_stamp3` (+0x1fc), `att` (+0x7f0), `give_att_disabled` (+0x7f8), all int | `Unit::process_attrition`, `gain_tech`; `Unit::suffer_attrition`; `Wall::process`; `Leader::calc_attrition`; `Leader::plan_strategy` |
| 6 | `anti-att` | `anti_att` (+0x7f4 float; class 3, `docs/DECISIONS.md` entry 120 §3) | out by name; a mechanic of its own |
| 7 | `diplomacy-relations` | `good_deeds` (+0xd4), `blacken` (+0x20c int), `dow` (+0x270), `broke_alliance` (+0x2b0), `made_peace` (+0x2d0), `last_spoke` (+0x2f4), `hire_stamp` (+0x1b4), `hire_who` (+0x1d4); int[8] unless named | the `Leader::action_*` family and `Leader::diplomacy`'s arms, which the crate stops at a SEAM |
| 8 | `diplomacy-tribute` | `tribute_stamp` (+0x174 int[8]), `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]), `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]), `tribute_sent` (+0x860 int), `tribute_received` (+0x864 int) | the `action_*` family, `consider_tribute` and `Leader::diplomacy`'s arms |
| 9 | `taunts` | `last_taunt` (+0x354 TauntRequest[8]), `taunt_frame` (+0x374 int[8]), `incoming_taunt` (+0x394 int[8]), `incoming_taunt_who` (+0x3b4 int[8]), `incoming_taunt_frame` (+0x3d4 int[8]), `incoming` (+0x3f4 int) | `receive_taunt`, `process_taunt`, `end_process` |
| 10 | `end-of-game` | `defeated_by` (+0x10), `wonderwin_stamp` (+0x448), `wonderwin_timer` (+0x44c), `victory_type` (+0x7d8), `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924), all int | `Leader::victory`, `defeat`, `defeat_by`; `Leader::process`; `GameDaemon::process_victory` |
| 11 | `lost-capital` | `lost_capital_stamp` (+0x414), `lost_capital_timer` (+0x418), `lost_capital_modifier` (+0x41c), all int | `Leader::lost_capital`, `recapture_capital`, `Build::activate`, `Leader::process`; the tally's `lost_capital_frame` is a partial copy of the stamp |
| 12 | `misery` | `pop_issues` (+0x7e8), `misery` (+0x7ec), int | `calc_misery`, `calc_pop_cap`, `gain_tech`; `Leader::process`, `Build::do_queue`, `Unit::process` |
| 13 | `marks-built` | `fort_mark` (+0x428), `oil_well_mark` (+0x430), `supply_mark` (+0x434), `wonders_built` (+0x854), `wonders_held` (+0x858), `senates_built` (+0x85c), `forts_built` (+0x8f0), all int | the registries' `init_*`/`close_*`; `Fort::init`; `Build::activate`; `Wonders::init_wonder` |
| 14 | `queued` | `scholar_militia` (+0x9f0); `barracks_queued` (+0xa10), `stable_queued` (+0xa14), `factory_queued` (+0xa18), `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued` (+0xa24), all int | `Leader::track_queued` (each its own zero floor) and `Unit::process`; `track_unit_type`'s militia-from-a-Scholar arm |
| 15 | `odd-counters` | `missiles_used` (+0x7c0), `repair_stamp` (+0x208), `units_bribed` (+0x8f4), `nukes_launched` (+0xa38), all int | `Ammo::do_damage`, `Unit::do_repair`, `cast_bribe`, `Unit::come_out` |
| 16 | `lobby-words` | `cannon_time` (+0x48 ulong), `handicap` (+0x4c int), `multi_diff` (+0x50 int) | below `+0x200`, unread: `Leader::init`'s lobby row and `TurnControl::start_cannon_time`; `Leader::set_diff` |
| 17 | `spells` | `tech.ptr`: `tech` (+0x6c0c `BitMask<806>`), the spell types' bits `0x275..0x2ab` (46 bits), absent from `tech_at_start` | out by name; a mechanic of its own |
| 18 | `score` | `score` (+0x18), `score_explored` (+0x1c), `score_territory` (+0x20), `score_units` (+0x24), `score_units_2` (+0x28), `score_buildings` (+0x2c), `score_economy` (+0x30), `score_pop` (+0x34), `score_unit_upgrades` (+0x38), `score_research` (+0x3c), `score_wonders` (+0x40), `score_combat` (+0x44), all int | out by name; a mechanic of its own |
| 19 | `last-finished` | `last_building_finished` (+0x6070 int[129]), `last_unit_finished` (+0x6274 int[352]) | out by name; a mechanic of its own |
| 20 | `high-buildings` | `territory_high` (+0x9dc int), `reg_terr` (+0x145e ushort[64]), `high_buildings` (+0x5660 ushort[129]) | out by name; a mechanic of its own |

Leaves: 1 + 4 + 13 + 4 + 5 + 1 + 8 + 7 + 6 + 6 + 3 + 2 + 7 + 7 + 4 + 3 + 1
+ 12 + 2 + 3 = **99**, plus `explored` makes §20's 100.

### Where this departs from the brief's list, and why

- **Diplomacy is split in two** (7, 8). Fifteen leaves over the whole
  `action_*` family (`funcs/Leader/action_*`, about a dozen) and the
  diplomacy arms would push one reader past the measured 320 k ceiling. The
  cut follows the record: standing and contact (deeds, war, peace, speech,
  hire) against goods moved (tribute, gift, counteroffer, demand). Both
  readers will read the same `action_*` bodies; that cost is accepted, and
  each family's adjudicator settles only its own leaves.
- **`scholar_militia` joins the six `*_queued`** (14). Both are kept by
  `Leader`'s tracking functions (`track_queued`, `track_unit_type`), which
  item 4124 read whole for the counters beside them; one leaf does not
  warrant a reader.
- **`anti_att` stays apart from attrition's stamps** (6 against 5). It is a
  float of class 3 and needs that class's treatment (an exact rational, or a
  pinned table), which is a different product from five integer stamps,
  though the two readers will share `Leader::calc_attrition`.
- **`explored` is out**: lane 4153 is porting it. `score_explored` (18) reads
  it; its reader states the formula in terms of `explored` and stops there.

## The log

Each hand-back as it lands: family, wave, tokens, minutes, leaves confirmed.

| family | wave | tokens | minutes | leaves (of) | note |
|---|---|---|---|---|---|
| anti-att | 1 | 154 k | 10 | 1 of 1 written | 8 stores in `Leader::calc_anti_attrition (006cdcc0)`; argues class 2 (f32 chain replayable by `Single`), not class 3 |
| last-finished | 1 | 138 k | 10 | 2 of 2 written | the object number `o` of the last finished of a type, −1 at init; `Build::activate (00623e20)` and `Build::train (0062f9b0)`; the crate's `record_last_built` hook is empty |
| city-name | 1 | 157 k | 11 | 1 of 1 written | `City::generate_name (00735c90)`: +1 to the same-tribe slot holding the most (ties low, >0, else the owner), and a second store setting the owner's 0 to 1; `Leader::init` clears |
| spells | 1 | 200 k | 12 | 1 of 1 written | one rule: `Leader::init (006e3930)`'s spell pass sets `tech` (never `tech_at_start`) for each spell type available whose `preq[0]` is not −1 or whose FLAGS lack `a`; 46 by the shipped table; the crate's `TechTree::start` stops just before it |
| lost-capital | 1 | 176 k | 12 | 3 of 3 written | `Leader::lost_capital (006dc870)`, `recapture_capital (006d1ba0)`, the Forbidden City arms of `Build::activate`/`Build::close`, and `Leaders::process_all`'s refresh (+1 a 5 frames); `Leader::process` is dead (inlined); flags `docs/CITIES.md` §8.5 |
| kills | 1 | 172 k | 13 | 4 of 4 written | `Object::take_damage (00652020)`, `do_damage (0064a480)`, `cast_sabotage`, `cast_counterintel`; a squad counts once (its captain, last figure, no decoy); attrition deaths count as lost; a hidden store printed as `local_8[0x20d].field_0x31a0` |
| taunts | 1 | 190 k | 13 | 6 of 6 written | `Leader::receive_taunt (006b8bf0)` and `process_taunt (006b8cc0)`; only a human taunting a computer ally writes them; `end_process` is not a writer (its stores are `GameInfo`'s) |
| misery | 1 | 177 k | 14 | 2 of 2 written | `pop_issues`: zeroed each frame in `Leaders::process_all`, +1 in `Unit::process`'s capped carrier and per visit in `Build::do_queue`; `misery`: every store is 0 (`calc_misery` has no caller) |
| victim-stamps | 1 | 205 k | 14 | 4 of 4 written | `Object::do_damage (0064a480)` alone in play: `leaders[victim].X[attacker] = frame` under a 300-frame window, city arms and a territory step; `plan_strategy` and `end_process` are not writers; `ctw_hero_stamp` CtW only |
| high-buildings | 1 | 204 k | 15 | 3 of 3 written | `territory_high`: max of `territory` (`World::compute_all_territory`, `check_borders`); `reg_terr`: per-region tile count in `compute_reg_territory`; `high_buildings`: `Build::activate`'s mark (crate holds it as `building_high`); no net caught a writer by offset |
| combat-rates | 1 | 194 k | 16 | 13 of 13 written | the `inc_*_event` bodies have no caller: their inlined copies in `do_damage`, `take_damage`, `cast_sabotage` write; `Leader::process_event_frame (006ec180)` folds at `frame % 50 == 0`; the fifteen-second words are game-long sums mod 2^16 |
| end-of-game | 1 | 218 k | 16 | 6 of 6 written | `Leader::defeat_by (006d1c80)`, `victory (006ec9b0)`, `defeat (006ecb00)`, `GameDaemon::process_victory (00730ef0)` and `Leaders::process_all`'s refresh; `wonderwin_timer` is a 0/1 flag; `gain_tech` and `set_diplo` call `victory`/`defeat` too |
| score | 1 | 227 k | 17 | 12 of 12 written | `Leader::compute_score (006ec560)` from `Leaders::strategy_all` at `(who+frame) % 10 == 0`, parts gated by dirty bits 23/24; `score_combat` stepped by kill value in four sites; `score_pop` and `score_explored` always 0 (not a read of `explored`); `score_units_2` not summed |
| attrition-stamps | 1 | 229 k | 18 | 5 of 5 written | `give_att_disabled`'s writers are the trigger pair, not `plan_strategy` (that store is `reg_land`); a `Game::do_frame` zeroing of `attrition_stamp`; **`att` floors at 1 even on a base of 0** under the Colosseum, the Russians, the CtW bonus or the Kremlin, against `docs/ATTRITION.md` and `territory/attrition.rs:43` (run981 cited; the line values checked, the leaders not yet) |
| diplomacy-tribute | 1 | 248 k | 18 | 7 of 7 written | of `action_*` only `action_respond (006d03c0)` writes; `consider_tribute`, `Leader::diplomacy`, `clear_agree`, `process_taunt`; reads parked 3470a both halves |
| lobby-words | 1 | 229 k | 18 | 3 of 3 written | `cannon_time`: `Leader::init`'s lobby row and `TurnControl::start_cannon_time`; `handicap`, `multi_diff`: `Game::init_handicaps (0058abf0)`, not `set_diff` (no caller); plain default skirmish: 0, 0, the lobby difficulty |
| odd-counters | 1 | 218 k | 18 | 4 of 4 written | one event writer each, as the row says; `missiles_used` feeds the V2/Cruise price; `nukes_launched` likely never moves (only `StrafeOrder` answers slot 0x100); `units_bribed` already counted as `tech.bribes` |
| queued | 1 | 234 k | 19 | 7 of 7 written | `track_queued (006e0f30)` plus six inlined live copies (`queue_up`, `unqueue`, `clean_queue`, `action_unqueue`, `Unit::process`, `action_spell`); the floor skips only a step from exactly 0; `scholar_militia` in `track_unit_type`, a possible permanent +1 from a recycled slot |
| last-finished | 2 | 158 k | 10 | 2 of 2 written | agrees on index, value (`o`), −1 init, both writers and the three (0,1,1) callers |
| anti-att | 2 | 149 k | 10 | 1 of 1 written | agrees: three writers, 256.0 start, the same four sources in order, the same one-ulp states; class 1 / class 2, never 3 |
| marks-built | 1 | 224 k | 21 | 7 of 7 written | the three registries' `init_*`/`close_*`; `wonders_built` +1 with no captured gate; `wonders_held` a high-water mark in `init_wonder`; `Setup::build_game` zeroes `supply_mark` (a store no field grep sees); `supply_mark` is the crate's `SupplyList::len()` |
| diplomacy-relations | 1 | 252 k | 21 | 8 of 8 written | only humans move `dow`, `blacken` (`action_declare`'s `free` is 1 from `Leader::diplomacy`); `made_peace` and the hire pair from `action_respond`'s execution; `last_spoke` has no writer (constant 0) |
| city-name | 2 | 163 k | 10 | 1 of 1 written | agrees on both stores, the same-tribe rule, the exemptions; adds `Leaders::clear` and `walk_data` |
| lost-capital | 2 | 173 k | 11 | 3 of 3 written | agrees on the five events and the 5-frame pay-back; states the modifier as own/other, not the loser/taker order the first gave: the adjudicator settles it |
| spells | 2 | 209 k | 12 | 1 of 1 written | agrees on the rule; checks it bit for bit against run 1562's frame-91 packet: the 9 clear bits are the 8 preq −2 spells and RADARSPELL |
| victim-stamps | 2 | 183 k | 11 | 4 of 4 written | agrees: `do_damage` alone, the same ten stores, city arms and ground arm, `ctw_hero_stamp` CtW only; one line of §20 (1710, the conquest arrays) leaked to it by a grep, recorded in its file |
| taunts | 2 | 206 k | 12 | 6 of 6 written | agrees: `receive_taunt` and `process_taunt` the only field writers, the drain in `process_all`, the HELP / +60 frame, `action_ping` the one reader; only a human's chat moves them |
| kills | 2 | ~180 k | 14 | 4 of 4 written | agrees: the same seven sites, squad test, spell kills skip `units_lost`, attrition deaths count, the self-credit tail, the site-abandon arm; four unrelated `docs/PARKED.md` lines leaked by a grep, recorded |
