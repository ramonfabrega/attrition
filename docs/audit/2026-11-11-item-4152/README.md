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
