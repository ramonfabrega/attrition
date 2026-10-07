//! **Every key the dump prints is read, or pinned as unread** — parked
//! 488, built in the tenth Fable pass (2026-09-22).
//!
//! Ten of the twenty-one landings between the ninth pass and the tenth
//! were instrument defects, and the family that hid the most was a field
//! the original printed on every frame that nothing in this crate read:
//! `damage_frac` parsed off a building's `OBJECT` and never a figure's
//! (484), `build_masks` off the wrong block (478), `recharging` and the
//! overkill window compared nowhere (485), and 373 `AMMO` records that no
//! reader had opened in the three days since the capture was taken. Each
//! read as agreement, because a field that is not read cannot part — and
//! no widening of a window can find it, however wide the window goes.
//! `CLAUDE.md`'s "diff the whole record" was a rule with nothing checking
//! it: nothing held `compare`'s field list against the dump's.
//!
//! This does. The read recorder (`gamelog::reads`) notes every key a
//! parse asks a block for, keyed on the block itself rather than its
//! name, so the same `OBJECT` under `UNITDATA` and under `WALLDATA` are
//! two rows. The guard parses the frames of the two headline windows
//! through **the readers the harness runs on a frame** — `frame_states`
//! (units, buildings, leaders, cities), `groups`, `last_group`, `farms_of`
//! — walks the tree into paths, and holds the dump's own keys on each
//! path against the keys read there. The difference is pinned, path by
//! path, and the pin is exact both ways: a key that arrives unread fails
//! until it is read or pinned, and a key that is read fails until its pin
//! is deleted — a pin that is allowed to lag is parked 449's "stale by
//! success" one level over.
//!
//! What this pin does not check is that a *parsed* field is *compared*.
//! `crate::ledger` counts that statically, as a lower bound; **the
//! compared pin at the end of this file measures it** — the shared
//! instrument registers every `Record.field` it compares with both sides
//! present (`crate::diff::compared`, parked 527) and the pin holds the
//! parser's records against what arrived on the Great Lakes word's own
//! window. And a record family with a parser of its own
//! that never touches `Block` — `AMMO`, whose scanner is `diff::ammo`
//! — is named in [`OWN_PARSER`] with the module that reads it, and its
//! keys are that module's to keep.

use std::collections::{BTreeMap, BTreeSet};

use crate::gamelog::{Block, Log, reads};

use super::testkit::{
    EAST_INDIES_ADDANIMALS_BLOCK, EAST_INDIES_BARK_BLOCK, EAST_INDIES_BLOCKEDWALK_BLOCK,
    EAST_INDIES_BLOCKWORD_BLOCK, EAST_INDIES_CAST_BLOCK, EAST_INDIES_COLUMNWORD_BLOCK,
    EAST_INDIES_EXPLORE_BLOCK, EAST_INDIES_FIGUREWORD_BLOCK, EAST_INDIES_GATHER_BLOCK,
    EAST_INDIES_GROUPWORD_BLOCK, EAST_INDIES_GUARDWORD_BLOCK, EAST_INDIES_IDLE_BLOCK,
    EAST_INDIES_LEADERWORD_BLOCK, EAST_INDIES_MAKE_BLOCK, EAST_INDIES_MARKETWORD_BLOCK,
    EAST_INDIES_MERCS_BLOCK, EAST_INDIES_SLOT_BLOCK, EAST_INDIES_TURNWORD_BLOCK,
    EAST_INDIES_USEMARKET_BLOCK, EAST_INDIES_WALKWORD_BLOCK, EAST_INDIES_WONDERPRICE_BLOCK,
    EAST_INDIES_WRAP_BLOCK, EAST_INDIES_WRAPWORD_BLOCK, GOLDEN_WORD_CHAPTER_SEVEN_B,
    GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL, GOLDEN_WORD_CHAPTER_SIX,
    GOLDEN_WORD_CHAPTER_THREE_RESTAGE, GOLDEN_WORD_CHAPTER_TWO, GREAT_LAKES_ATTACKED_BLOCK,
    GREAT_LAKES_BIRTH_BLOCK, GREAT_LAKES_CIVIC_BLOCK, GREAT_LAKES_COPY_BLOCK,
    GREAT_LAKES_DETOUR_BLOCK, GREAT_LAKES_ESCORT_BLOCK, GREAT_LAKES_FOREST_CELL_BLOCK,
    GREAT_LAKES_GIVEUP_BLOCK, GREAT_LAKES_MAKE_BLOCK, GREAT_LAKES_MIRROR_BLOCK,
    GREAT_LAKES_PATRIOT_BLOCK, GREAT_LAKES_PYRAMIDS_BLOCK, GREAT_LAKES_RECRUIT_BLOCK,
    GREAT_LAKES_RETRY_BLOCK, GREAT_LAKES_RETURN_BLOCK, GREAT_LAKES_ROAD_BLOCK,
    GREAT_LAKES_SQUAD_BLOCK, GREAT_LAKES_STAND_BLOCK, GREAT_LAKES_UPGRADE_BLOCK,
    GREAT_LAKES_VALS_BLOCK, GREAT_LAKES_WONDER_BLOCK, WIDENING_CHAPTER_TWO,
    WIDENING_EAST_INDIES_END, WIDENING_GREAT_LAKES_END,
};

/// Record paths read by a parser of their own, outside `Block` — the
/// module that reads each is named, and this guard leaves them alone.
///
/// The per-frame `GOOD` list is read by `diff::golden`'s `frame_goods`
/// (item 578): the bare name line, `ever_seen`, and the `SubObject`'s
/// `flags`, `o`, `x_internal` and `y_internal`, which chapter seven's and
/// seven-b's widenings compare on every block. Its `who` and `z_internal`
/// are printed and not read. Item 628 put the first window carrying the
/// list in this driver.
const OWN_PARSER: &[(&str, &str)] = &[
    ("GAME/FRAME/AMMO", "diff::ammo::blocks"),
    ("GAME/FRAME/GOOD", "diff::golden::frame_goods"),
    ("GAME/FRAME/GOOD/SUBOBJECT", "diff::golden::frame_goods"),
];

/// `(path, keys)` — every key the dump prints on that path that nothing
/// reads, on the day of the pin, space-separated and sorted. **Exact**:
/// a key read since must be deleted here, a key that arrives unread must
/// be read or added here with the item that owes it. A row whose every
/// key is unread is a record family nobody has opened.
const UNREAD: &[(&str, &str)] = &[
    // Pinned 2026-09-22, the tenth pass, over 87 frames: chapter two's
    // whole window and Great Lakes 10275–10279. Twenty paths, 239 keys.
    // Three rows were whole families nobody had opened: `DEATH_OBJS`,
    // the frame-level `GUY` list (only the three animation-clock keys
    // are read, by `anim_lengths`; its `x y type who` are not), and
    // `WORLD`'s per-frame resource totals.
    //
    // **Item 491 took two of the three rows its own item named**, over
    // 99 frames on the widened window: `DEATH_OBJS` whole — all six keys
    // parsed, five of them compared, `gpiece` alone left because this
    // crate loads no death piece — and `hold_frames` off both `UNITDATA`
    // `OBJECT` paths, which is now a `hits_diverged` row. Nineteen paths
    // and 232 keys stand. `WALLDATA/OBJECT` keeps its `hold_frames`: it
    // is a building's, and `BuildDump` parses none of that half.
    //
    // **Item 510 took `hold_attack` and `queued_attack` off both
    // `UNITDATA/GUY` paths** — the pair that decides whether an attack
    // asked for mid-animation is paid next frame or inside
    // `Guy::inc_time`'s own wrap loop, which is chapter two's word
    // (`docs/COMBAT.md` §45). Nineteen paths and 228 keys stand.
    // `GAME/FRAME/GUY` keeps both: that is the frame-level list, whose
    // only reader is `anim_lengths`, and no comparison opens it.
    //
    // **Item 523 took `o_down` off both `UNITDATA` paths**, beside `o_up`
    // (`docs/COMBAT.md` §47). And it is the lesson of this module's own
    // blind side: `o_up` was never on this pin, because `diff::army`
    // reads it once at stand-up to seed the captain flag, and that one
    // read made the key look covered. Nothing compared it per frame, so
    // run112's `1/7` became captain on 743 and this crate's did not, and
    // nothing said so until the missing draw on 762. A stand-up read is
    // not a comparison. The module header already says this guard stops
    // at "parsed", and here that gap cost nineteen frames. Two keys
    // leave the pin; no path does.
    //
    // **Item 530 took `ox` and `whom` off both `UNITDATA/GUY` paths**:
    // what a figure last swung at, which `Unit::fight`'s recharging arm
    // reads against the order's target, and which chapter one's word
    // turned on (`docs/ORDERS.md` §22). Four keys leave; no path does.
    //
    // **Item 552 took `attrition` off both `UNITDATA` paths**: the pending
    // tick period, which `widen_block` compares beside `unit_masks2`'s
    // supply mark since chapter four put a squad on hostile ground. Two
    // keys leave; no path does.
    //
    // **Item 759 took `*((dword*)` off both `UNITDATA/GUY` paths**: the
    // key every one of `GuyData::log_data`'s single-precision lines
    // shares, the record splitting at the first space, and the four that
    // are a plane's attitude — `bank`, `last_bank`, `pitch`, `last_pitch`
    // — are read by their tag now and compared (`docs/ORDERS.md` §33).
    // `turret_inc`, the fifth, rides the same key unread. Two keys leave;
    // no path does.
    //
    // **Item 1117 took the turret's four off both `UNITDATA/GUY` paths**
    // — `turret_angles[4]`, `des_turret_angles[4]`, `node_flags` and
    // `des_node_flags` — which the golden widening compares
    // (`golden::widen_turrets`, `docs/COMBAT.md` §55.3): a pivot piece's
    // release waits on `node_flags`, and chapter thirty-eight's word
    // turned on one (parked 1119). Ten keys leave each path; no path does.
    ("GAME/FRAME/ANIMALDATA", "aid ox whom"),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA",
        "air_alt cavarch_uid full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation last_angle o who",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/OBJECT",
        "healing inside_down inside_down_who launch_frames near_o near_who",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/STACK<TYPE>",
        "increment length size",
    ),
    // Pinned 2026-09-23, the thirteenth pass (parked 670): `build_of`
    // iterated every field for its `tx`/`ty` pairs, which the recorder
    // notes as `*`, so the whole `BUILDDATA` level read as covered while
    // fourteen of its keys had never been parsed — `city` and `city_down`
    // among them until item 661. `Block::fields_of` reads the two by name
    // now, and these are what the pin then found.
    // **Item 928 added `metric type`**: the gather point list's per-node
    // pair (`PtrLinkListAbstract<GatherPoint>::log_data`), `type` 0 and
    // `metric` the node's key, 0 on every node `add_gather_point` makes.
    // The points themselves are read (`GATHERPOINT`, `BuildDump::gather`).
    // **Item 1112 read `attack_ox attack_whom recharging`**: the golden
    // widening's anti-air cycle rows (`golden::cycle_rows`).
    (
        "GAME/FRAME/BUILDDATA",
        "dock flags fort founder healing increment infiltrate infiltrate2 metric oil_well stance type wonder",
    ),
    ("GAME/FRAME/BUILDDATA/BUILDQUEUE", "queue_size"),
    (
        "GAME/FRAME/BUILDDATA/WALLDATA",
        "demolition frame_started gpiece helpers job_counter_2",
    ),
    // **Item 836 added `flags increment length size`**: an Airbase's
    // `launching` array (`ObjectData +0x44`), which `Object::do_launch`
    // mallocs on its first launch and `kill_current_order` empties when
    // the EXIT's order dies, so it prints from run265's 778 with `length
    // 0` and nothing reads it. It reaches a comparison only when two
    // planes launch from one base in one call, which no capture holds.
    // A Missile Silo's `launching` (`ObjectData +0x44`), printed as
    // `list[scan]` while a missile waits in it, joined the row with
    // run371 (item 1019, `docs/GOLDEN.md` §44).
    (
        "GAME/FRAME/BUILDDATA/WALLDATA/OBJECT",
        "down down_who flags healing hold_frames increment infiltrated inside_down inside_down_who launch_frames length list[scan] myhits mylos near_o near_who size uid up up_who",
    ),
    ("GAME/FRAME/CITIES", "increment length size"),
    // **Item 989 added `Newcastle`**: a city's name is the record's one
    // valueless line, so each name is a key of its own. run356's and
    // run357's windows are the first driven ones to hold a third British
    // city; the name is read by nothing, as `London` and `Norwich` are not.
    // **Item 1332 added `York`**: run529's window, Great Sahara at
    // Toughest past 9574, holds a city no earlier window did.
    // Item 1326 met it too, on run523's East Indies window. **Item 1379
    // added `Edinburgh`**: run571's window, past 11177, holds a city no
    // earlier window did.
    // Items 1442/1443: Paris, Brest and Nantes are bare display-name lines, not sim fields.
    // Item 1452: Lyons, the French city founded before run622's window, likewise.
    // Item 1455: Rheims, the fifth, founded on 9777 and named in run629.
    // Item 1481: Orleans, the sixth, named in run642's window.
    // Item 1487: Amiens, the seventh, named in run655's window.
    // Item 1511: Persepolis, the Persian capital, on the coverage pair's
    // windows (run656, run660, run669), driven since that item.
    // Item 1532: Pasargadae, the Persians' second, named in run672's window.
    // Item 1546: Arak, the Persians' third, named on run678's block 1184.
    // Item 1561: Tabriz, the Persians' fourth, named in run683's window.
    // Item 1565: Khomein, the Persians' fifth, named on run683's block 1819.
    (
        "GAME/FRAME/CITIES/CITY",
        "Amiens Arak Brest Edinburgh Khomein London Lyons Nantes Napata Newcastle Norwich Orleans Paris Pasargadae Persepolis Rheims Tabriz York flags increment length size",
    ),
    // **Item 628 added four keys to the frame-level `GUY` row**:
    // `flags`, `x_internal`, `y_internal` and `z_internal` are not a
    // figure's. They are good `o 0`'s `SubObject`, which the dump prints
    // with no `BEGIN GOOD` after the last `GUY` of the list before it
    // (item 578), so they land on this path on every block of a capture
    // with `GOODS=3`. Chapter seven-b's windows are the first such here.
    // Nothing is owed on them: the widening counts that good as
    // unreadable.
    (
        "GAME/FRAME/GUY",
        "(int)off_x (int)off_y (int)variation *((dword*) angle avg_speed cur_time des_angle des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] des_x des_y flags guy_flags guy_num hold_attack last_angle last_speed last_time last_x last_y last_z node_flags o ox queued_attack stopped track_dx track_dy turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] type who whom x x_internal y y_internal z z_internal",
    ),
    // **Item 520 re-keyed the Great Lakes window onto run123 and put the
    // leader reader in the driver.** run123 is `LEADERS=9`, the whole
    // `LeaderData::log_data` record, where run100 was `LEADERS=1`, and
    // `diff::leader::theirs`, which the run117 and run123 widenings
    // compare through, was never driven here. So fifteen keys leave this
    // row as read (`bucket`, `income`, `epoch_get(scan)` and the rest of
    // the goods block), and the level-9 tail's unread keys join it. They
    // are owed by no item: nothing measured turns on them yet.
    // `known_rares`, the Merchant offer's gate, left with item 327
    // (`docs/AI.md` §55). Item 576 put run139, East Indies at `LEADERS=9`,
    // in the driver, and one key joined: `economic`, a bare line with no
    // value closing the AI leader's record — a string the record prints
    // without its name, read as a key with nothing after it. The word is
    // the leader's script's name, `economic.bhs`; which field prints it is
    // not read, and nothing is owed on it. Item 592 read the census's
    // twenty-one per-region arrays (`leader::REGION_ARRAYS`): `reg_terr`,
    // which this crate does not keep, and `reg_buildings` stay. Item 883
    // read `ages_get()`, `epochs_get()` and `discovered_get()`, the tech
    // counters (`docs/GOLDEN.md` §35), and they left the row. Item 1147
    // read `pop_cap`, which the third map's Peacocks had parted unseen.
    // Item 1209 read `agendas[scan]`, the bit `Unit::resolve_block` writes.
    // Item 1293 read `flock_stamp`, the frame `Objects::add_flock` stamps.
    (
        "GAME/FRAME/LEADERDATA",
        "(int) ages_queued aggression[scan] air_queued air_units ally_stamp[scan] anti_att att attack_stamp[scan] attrition_stamp attrition_stamp2 attrition_stamp3 average_damage_rate average_death_rate average_hit_rate average_kill_rate barracks_garr barracks_queued barracks_units base_rate[scan] best_armor best_attack best_move best_pop bit_values bits blacken bonus bonus_cap[NUM_COMMON] broke_alliance[scan] buildings_lost buildings_razed capital_stamp[scan] chat_status[scan] cities_captured cities_lost city_mark city_mine city_name combat_queued combat_units counteroffer[scan] ctw_hero_retreat_stamp ctw_hero_stamp damage_current_frame damage_fifteen_seconds deaths_current_frame deaths_fifteen_seconds defeat_stamp defeat_type defensive dock_mark dock_queued dock_units dow[scan] economic epochs_queued explored factory_queued factory_units flags fort_mark frame_battle gift_stamp[scan] good_deeds[scan] got_diplo_message handicap hero_mark high_buildings[scan] hire_stamp[scan] hire_who[scan] hits_current_frame hits_fifteen_seconds increment invaders[scan] kills_current_frame kills_fifteen_seconds last_spoke[scan] last_taunt[scan] length list[scan] lost_capital_modifier lost_capital_stamp lost_capital_timer lost_city_stamp made_peace[scan] misery missiles_used multi_diff nuke_stamp nukes_in_flight nukes_launched nukes_used num_bonus_cards[scan] num_buildings[scan] num_ctw_rate_bonuses[scan] oil_well_mark peasants_garr pop_issues raid_stamp[scan] rares_collected[scan] reg_buildings[scan][scan2] reg_terr[scan] repair_stamp retargets scholar_militia scout_garr senates_built size special_mark stable_garr stable_queued stable_units strong[scan] supply_mark support support_stamp taunt_frame[scan] team_color territory_high tribute_demanded[scan] tribute_stamp[scan] tributes[scan] units_killed units_lost victory_type village_mine weak[scan] wonderwin_stamp wonderwin_timer",
    ),
    (
        "GAME/FRAME/LEADERDATA/DIPLOMACY",
        "agree any_offer attacks[scan] offers[scan] treaty",
    ),
    (
        "GAME/FRAME/UNITDATA",
        "air_alt cavarch_uid full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation last_angle o who",
    ),
    // **Item 571's run163 window is the first to print a grouped
    // attack-move** (`GROUPATTACKTOORDER`; run136 has none). Its
    // `GroupMoveOrder` writes both bases, so `UNITORDER` prints twice: once
    // under `MOVEORDER`, which the order parser reads as `flags` and the
    // widening compares, and once under `GROUPORDER`, which nothing reads.
    // Both copies print 5 on the word's blocks. Nothing is owed on the
    // second: no measured frame turns on it.
    (
        "GAME/FRAME/UNITDATA/GROUPATTACKTOORDER/GroupMoveOrder/GROUPORDER/UNITORDER",
        "flags",
    ),
    // **Item 676's run180 window is the first to print a plain grouped
    // move** — the squad's `GroupMoveOrder` under a player's `MOVE_TO` —
    // and its second `UNITORDER`, under `GROUPORDER`, is the same unread
    // copy as the attack-move's above; both print 5 on the word's blocks.
    // Nothing is owed on it either.
    (
        "GAME/FRAME/UNITDATA/GroupMoveOrder/GROUPORDER/UNITORDER",
        "flags",
    ),
    // **Item 693's run184 windows are the first to print a patrol.**
    // `GroupPatrolOrder` writes `PATROLORDER` — the two point arrays as
    // flat `SimpleArray<Coord>` lines and `waypoint` — then `GROUPORDER`,
    // whose `UNITORDER` copy is the same unread second copy as the grouped
    // moves' above. The order parser reads the arrays by their `length`s
    // and `list[scan]` lines and the `waypoint`; each array's `size`,
    // `increment` and `flags` are its allocation, which no step reads. The
    // turn pump's text for the squad's command, printed at `GAME` level
    // after block 641, is the same kind of line as chapter nine's
    // `process_move_to`, which no window here reached; the harness takes
    // the command from the script, not from the dump.
    (
        "GAME/FRAME/UNITDATA/GroupPatrolOrder/PATROLORDER",
        "flags increment size",
    ),
    (
        "GAME/FRAME/UNITDATA/GroupPatrolOrder/GROUPORDER/UNITORDER",
        "flags",
    ),
    // **Item 696's run190 window at 622 adds `process_guard`**, the
    // guard's line after block 621: the frame alone (the `ox`/`whom`/
    // `queued` line is the sync logger's, not the dump's). The harness
    // takes the command from the script, as for the patrol. **Item 714's
    // run204 window at 622 adds `process_follow`**, the same shape, and
    // **item 718's run208 windows `process_garrison` and
    // `process_eject_all`**. **Item 723's run210 windows `process_form`**,
    // and with its 702 window the right-click's `process_move_to` and
    // `process_move_to_2` lines, which no chapter's window had held.
    // **Item 731's run215 windows `process_attack`** (`ox whom ignore
    // queued frame`), the first player's attack on disk. **Item 746's
    // run223 windows `process_flight`**, the frame alone, the same shape.
    // **Item 779's run241 windows `process_build`** (`x y x2 y2 type
    // queued frame`), the first player's build on disk. **Item 790's
    // run245 windows `process_spell`** (`type ox whom frame`), the first
    // player's craft. **Item 803's run249 windows
    // `process_set_transport`** (`frame`), the first player's transport
    // toggle; the flag it carries is read off the units' `unit_masks`.
    // **Item 813's run255 windows `process_swarm_around`** (`ox whom
    // queued orders frame`), the first player's repair; the orders it
    // lays are read off the citizens' stacks. **Item 867's run281 windows
    // `process_buildmask`** (`frame`), the first player's repeat button;
    // the mask it toggles is read off the building's `build_masks`.
    // **Item 877's run285 windows `process_queue_up`** (`type num
    // frame`), the first player's production; the entries it lays are
    // read off the building's `BUILDQUEUE`. **Item 884's run292 windows
    // `process_unqueue`** (`o p frame`), the first player's cancel; what
    // it removes and refunds is read off `BUILDQUEUE` and the buckets.
    // **Item 928's run312 windows `process_gather_point`** (`x y action
    // add_to_end frame`), the first player's rally point; what it writes is
    // read off the building's gather list. **Item 976's run358 windows
    // `process_launch_patrol`** (`x y queued shift ctrl alt`), the
    // Airbase's right-click on the ground; what it writes is read off the
    // planes' stacks. **Item 1167's run430 windows `process_alarm`**
    // (`frame`) and **`process_gather`** (`ox queued frame`), the City's
    // alarm and the first player's gather; what they lay is read off the
    // City's `city_flags` and the units' stacks. **Item 1182's run437
    // windows `console.play`**, the line `be` logs as it moves the
    // console's seat; the harness carries the seat on its script
    // (`golden::Script`), and no field of the simulation reads it.
    (
        "GAME",
        // Item 1444: run609 prints the closing marker as GameInfo closing.
        "GameInfo console.play process_alarm process_attack process_build process_buildmask process_eject_all process_flight process_follow process_form process_gather process_gather_point process_garrison process_group, process_guard process_launch_patrol process_move_to process_move_to_2 process_patrol process_queue_up process_set_transport process_spell process_swarm_around process_unqueue",
    ),
    // **Item 746's run223 windows are the first to print a player's air
    // order**: the `STRAFEORDER` a flight home builds and the
    // `AIRPATROLORDER` a strike on an unseen target becomes
    // (`docs/GOLDEN.md` §25). The order parser reads each `AIRORDER` row
    // and the strafe's `xx`/`yy` (`docs/ORDERS.md` §32); what stays unread
    // is the `AIRORDER`'s own `UNITORDER` copy, the same second copy as
    // the grouped moves' above, and the patrol's array allocation.
    // **Item 1019 owes these** (`docs/GOLDEN.md` §44, run371): the V2's
    // `AIRATTACKGROUNDORDER` — its launch point and the round's time, and
    // its `AIRORDER`'s second `UNITORDER` copy. No order parser opens the
    // air attack on the ground; the missile's flight is not carried.
    (
        "GAME/FRAME/UNITDATA/AIRATTACKGROUNDORDER",
        "sx sy total_time",
    ),
    (
        "GAME/FRAME/UNITDATA/AIRATTACKGROUNDORDER/AIRORDER/UNITORDER",
        "flags",
    ),
    (
        "GAME/FRAME/UNITDATA/AIRPATROLORDER/AIRORDER/UNITORDER",
        "flags",
    ),
    (
        "GAME/FRAME/UNITDATA/AIRPATROLORDER/PATROLORDER",
        "flags increment size",
    ),
    (
        "GAME/FRAME/UNITDATA/STRAFEORDER/AIRORDER/UNITORDER",
        "flags",
    ),
    (
        "GAME/FRAME/UNITDATA/OBJECT",
        "healing inside_down inside_down_who launch_frames near_o near_who",
    ),
    ("GAME/FRAME/UNITDATA/STACK<TYPE>", "increment length size"),
    (
        "GAME/FRAME/UNITDATA/TRADEORDER",
        "loaded oxx started uid2 whose",
    ),
    (
        "GAME/FRAME/WORLD",
        "forest_size goodies land_resources mountain_size rock_size sea_resources total_metal total_oil",
    ),
];

/// A block's path element: its name with a trailing number stripped, so
/// `FRAME 683` and `FRAME 10277` are one path.
fn element(name: &str) -> String {
    let t = name.trim_end();
    match t.rsplit_once(' ') {
        Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|c| c.is_ascii_digit()) => {
            head.to_string()
        }
        _ => t.to_string(),
    }
}

/// `path → (printed keys, read keys)`, unioned over every block on the
/// path.
type Paths = BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>;

/// The keys the text prints on each path, by the indent rule alone: a
/// field belongs to the innermost open block shallower than it. This is
/// read off the text and not the arena on purpose — the parser records a
/// field at exactly a closed block's own indent on **both** candidates
/// (`gamelog::fill`'s both-candidates rule), so the arena says `OBJECT`
/// carries every `UNITDATA` field that follows it, and a coverage read
/// off the arena would hold the parser to keys it never printed there.
fn printed(text: &str, out: &mut Paths) {
    let mut stack: Vec<(usize, String)> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start_matches(' ');
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        while stack.last().is_some_and(|(i, _)| *i >= indent) {
            stack.pop();
        }
        if let Some(name) = trimmed.strip_prefix("BEGIN ") {
            stack.push((indent, element(name)));
            out.entry(path(&stack)).or_default();
        } else if !stack.is_empty() {
            let key = trimmed.split(' ').next().unwrap_or("");
            out.entry(path(&stack))
                .or_default()
                .0
                .insert(key.to_string());
        }
    }
}

fn path(stack: &[(usize, String)]) -> String {
    stack
        .iter()
        .map(|(_, n)| n.as_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// The keys the recorder saw asked for on each path, over every node of
/// the path. Run **after** the recorder has stopped — the walk iterates
/// fields itself.
fn read(b: Block<'_>, path: &mut Vec<String>, r: &reads::Reads, out: &mut Paths) {
    path.push(element(b.name()));
    if let Some(keys) = r.get(&b.read_key()) {
        out.entry(path.join("/"))
            .or_default()
            .1
            .extend(keys.iter().cloned());
    }
    for c in b.children() {
        read(c, path, r, out);
    }
    path.pop();
}

/// Parses one frame's text through every reader the harness runs on a
/// frame block, recording, and folds the result into `out`.
fn drive(text: &str, out: &mut Paths) {
    let log = Log::parse_eager(text);
    reads::start();
    // The harness's frame decode (`IndexedCapture::frame_state`), the
    // three walks `Log::initial` folds over every frame, and the three
    // per-frame readers the widening tests call on a frame block. A
    // reader added to the harness is added here, or its keys read as
    // unread — which is the right failure.
    let _ = log.frame_states();
    let _ = log.frame_seeds();
    let _ = log.anim_lengths();
    for root in log.roots() {
        for frame in root.children() {
            let _ = crate::gamelog::observation_rows(frame, true);
            let _ = crate::gamelog::groups(frame);
            let _ = crate::gamelog::last_group(frame);
            let _ = crate::gamelog::farms_of(frame);
            // The golden widening's anti-air cycle rows (item 1112).
            let _ = super::golden::cycle_rows(frame);
            // A building's `visible` byte (item 1423).
            let _ = super::golden::visible_rows(frame);
            // The leader widening's reader (item 520): run117's and
            // run123's tests compare the whole record through it.
            // Item 592 added the census's per-region arrays beside it:
            // run143's test reads them.
            for l in frame.kids("LEADERDATA") {
                let _ = crate::diff::leader::theirs(&l);
                let _ = crate::diff::leader::region_theirs(&l);
            }
        }
    }
    let r = reads::stop();
    printed(text, out);
    let mut path = Vec::new();
    for root in log.roots() {
        read(root, &mut path, &r, out);
    }
}

/// The keys on each path that were printed and never read; `*` on a path
/// means a whole-fields iteration read all of them.
fn unread(paths: &Paths) -> BTreeMap<String, BTreeSet<String>> {
    paths
        .iter()
        .filter(|(p, _)| !OWN_PARSER.iter().any(|(own, _)| own == p))
        .map(|(p, (printed, read))| {
            let keys = if read.contains("*") {
                BTreeSet::new()
            } else {
                printed.difference(read).cloned().collect()
            };
            (p.clone(), keys)
        })
        .filter(|(_, keys)| !keys.is_empty())
        .collect()
}

fn golden_dump(run: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
    let dump = format!("{dir}/{run}/map-14/gamelog.txt");
    std::path::Path::new(&dump).is_file().then_some(dump)
}

/// The frames of a capture in `[lo, hi]`, driven one at a time.
fn drive_capture(path: &str, lo: i64, hi: i64, out: &mut Paths) -> usize {
    let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let at: Vec<usize> = ix
        .frames()
        .iter()
        .enumerate()
        .filter(|(_, f)| (lo..=hi).contains(&f.number))
        .map(|(i, _)| i)
        .collect();
    for i in &at {
        let text = ix.read_frame(*i).unwrap();
        drive(&text, out);
    }
    at.len()
}

/// The 484 shape, synthetic: the same `OBJECT` block under two parents,
/// one read and one not, and the guard names the unread one only. This
/// is the test that was made to fail first — with the recorder keyed on
/// the block's *name* both rows read as covered.
#[test]
fn the_recorder_tells_one_object_block_from_another() {
    let text = "BEGIN FRAME 7\n BEGIN UNITDATA\n  angle 1\n  BEGIN OBJECT\n   myhits 40\n   damage_frac 3\n BEGIN BUILDDATA\n  BEGIN WALLDATA\n   BEGIN OBJECT\n    myhits 90\n    damage_frac 5\n BEGIN WORLD\n  goodies 2\n";
    let log = Log::parse_eager(text);
    reads::start();
    let frame = log.roots().next().unwrap();
    let unit = frame.kid("UNITDATA").unwrap();
    let _ = unit.int("angle");
    let obj = unit.find("OBJECT").unwrap();
    let _ = obj.int("myhits");
    let _ = obj.int("damage_frac");
    let wall = frame.kid("BUILDDATA").unwrap().find("OBJECT").unwrap();
    let _ = wall.int("myhits");
    let _ = frame.kid("WORLD").unwrap().fields().count();
    let r = reads::stop();
    let mut paths = Paths::new();
    printed(text, &mut paths);
    let mut path = Vec::new();
    read(frame, &mut path, &r, &mut paths);
    let u = unread(&paths);
    let rows: Vec<(String, Vec<String>)> = u
        .iter()
        .map(|(p, k)| (p.clone(), k.iter().cloned().collect()))
        .collect();
    assert_eq!(
        rows,
        vec![(
            "FRAME/BUILDDATA/WALLDATA/OBJECT".to_string(),
            vec!["damage_frac".to_string()]
        )],
        "the figure's OBJECT is read whole, the wall's is short one key, and WORLD was iterated"
    );
    // The recorder is off now: nothing this walk asked for was noted.
    assert!(reads::stop().is_empty());
}

/// **Every key the dump prints on the two headline windows is read by a
/// reader the harness runs, or pinned in [`UNREAD`].** Chapter two's
/// whole widening window and the Great Lakes word's block with two on
/// either side; a machine without either capture says so.
#[test]
fn every_key_the_dump_prints_is_read_or_pinned() {
    let ch2 = golden_dump("ch2");
    let ch3b = golden_dump("ch3b");
    let ch7b = golden_dump("ch7b");
    let ch7bc = golden_dump("ch7bc");
    let ch6 = golden_dump("ch6");
    let ch6b = golden_dump("ch6b");
    let ch8 = golden_dump("ch8");
    let ch9 = golden_dump("ch9");
    let ch10 = golden_dump("ch10");
    let ch11 = golden_dump("ch11");
    let ch12 = golden_dump("ch12");
    let ch13 = golden_dump("ch13");
    let ch14 = golden_dump("ch14");
    let ch15 = golden_dump("ch15");
    let ch16 = golden_dump("ch16");
    let ch17 = golden_dump("ch17");
    let ch18 = golden_dump("ch18");
    let ch19 = golden_dump("ch19");
    let ch20 = golden_dump("ch20");
    let ch21 = golden_dump("ch21");
    let ch22 = golden_dump("ch22");
    let ch23 = golden_dump("ch23");
    let ch24 = golden_dump("ch24");
    let ch25 = golden_dump("ch25");
    let ch26 = golden_dump("ch26");
    let ch27 = golden_dump("ch27");
    let ch28 = golden_dump("ch28");
    let ch29 = golden_dump("ch29");
    let ch30 = golden_dump("ch30");
    let ch31 = golden_dump("ch31");
    let ch32 = golden_dump("ch32");
    let ch33 = golden_dump("ch33");
    let ch34 = golden_dump("ch34");
    let ch35 = golden_dump("ch35");
    let ch36 = golden_dump("ch36");
    let ch37 = golden_dump("ch37");
    let ch38 = golden_dump("ch38");
    let ch39 = golden_dump("ch39");
    let ch40 = golden_dump("ch40");
    let ch41 = golden_dump("ch41");
    let ch42 = golden_dump("ch42");
    let ch43 = golden_dump("ch43");
    let ch44 = golden_dump("ch44");
    let ch45 = golden_dump("ch45");
    let ch46 = golden_dump("ch46");
    let ch47 = golden_dump("ch47");
    let ch48 = golden_dump("ch48");
    let ch49 = golden_dump("ch49");
    let ch50 = golden_dump("ch50");
    let ch17a = golden_dump("ch17-ammo");
    let r136 = crate::testenv::dump("gamelog-run136-greatlakes-detour.txt");
    let r163 = crate::testenv::dump("gamelog-run163-greatlakes-upgradeword.txt");
    let r174 = crate::testenv::dump("gamelog-run174-greatlakes-civicword.txt");
    let r178 = crate::testenv::dump("gamelog-run178-greatlakes-copyword.txt");
    let r192 = crate::testenv::dump("gamelog-run192-greatlakes-birthword.txt");
    let r196 = crate::testenv::dump("gamelog-run196-greatlakes-patriotword.txt");
    let r202 = crate::testenv::dump("gamelog-run202-greatlakes-mirrorword.txt");
    let r211 = crate::testenv::dump("gamelog-run211-greatlakes-wonderword.txt");
    let r218 = crate::testenv::dump("gamelog-run218-greatlakes-escortword.txt");
    let r226 = crate::testenv::dump("gamelog-run226-greatlakes-giveupword.txt");
    let r243 = crate::testenv::dump("gamelog-run243-greatlakes-standword.txt");
    let r80 = crate::testenv::dump("gamelog-run80-greatlakes-latecensus.txt");
    let r143 = crate::testenv::dump("gamelog-run143-eastindies-bark.txt");
    let r139 = crate::testenv::dump("gamelog-run139-eastindies-makelist.txt");
    let r149 = crate::testenv::dump("gamelog-run149-eastindies-animal.txt");
    let r152 = crate::testenv::dump("gamelog-run152-eastindies-gatherbuilding.txt");
    let r155 = crate::testenv::dump("gamelog-run155-eastindies-longword.txt");
    let r159 = crate::testenv::dump("gamelog-run159-eastindies-idleword.txt");
    let r166 = crate::testenv::dump("gamelog-run166-eastindies-incword.txt");
    let r221 = crate::testenv::dump("gamelog-run221-eastindies-slotword.txt");
    let r227 = crate::testenv::dump("gamelog-run227-eastindies-wrapword.txt");
    let r233 = crate::testenv::dump("gamelog-run233-eastindies-blockword.txt");
    let r251 = crate::testenv::dump("gamelog-run251-eastindies-columnword.txt");
    let r253 = crate::testenv::dump("gamelog-run253-eastindies-marketword.txt");
    let r257 = crate::testenv::dump("gamelog-run257-eastindies-guardword.txt");
    let r269 = crate::testenv::dump("gamelog-run269-eastindies-turnword.txt");
    let r277 = crate::testenv::dump("gamelog-run277-eastindies-blockedwalk.txt");
    let r289 = crate::testenv::dump("gamelog-run289-eastindies-marketword.txt");
    let r299 = crate::testenv::dump("gamelog-run299-eastindies-wonderprice.txt");
    let r96 = crate::testenv::dump("gamelog-run96-eastindies-latecensus.txt");
    let r349 = crate::testenv::dump("gamelog-run349-islands-toughest-open.txt");
    let r350 = crate::testenv::dump("gamelog-run350-greatlakes-toughest-open.txt");
    let r352 = crate::testenv::dump("gamelog-run352-islands-toughest-1576.txt");
    let r355 = crate::testenv::dump("gamelog-run355-greatlakes-toughest-3776.txt");
    let r356 = crate::testenv::dump("gamelog-run356-greatlakes-toughest-4555.txt");
    let r357 = crate::testenv::dump("gamelog-run357-islands-toughest-5606.txt");
    let r414 = crate::testenv::dump("gamelog-run414-islands-toughest-5975.txt");
    let r419 = crate::testenv::dump("gamelog-run419-islands-toughest-6321.txt");
    let r420 = crate::testenv::dump("gamelog-run420-islands-toughest-6609.txt");
    let r425 = crate::testenv::dump("gamelog-run425-islands-toughest-7382.txt");
    let r439 = crate::testenv::dump("gamelog-run439-islands-toughest-8519.txt");
    let r445 = crate::testenv::dump("gamelog-run445-islands-toughest-8820.txt");
    let r462 = crate::testenv::dump("gamelog-run462-islands-toughest-10183.txt");
    let r480 = crate::testenv::dump("gamelog-run480-islands-toughest-10985.txt");
    let r490 = crate::testenv::dump("gamelog-run490-islands-toughest-11328.txt");
    let r506 = crate::testenv::dump("gamelog-run506-islands-toughest-11637.txt");
    let r508 = crate::testenv::dump("gamelog-run508-islands-toughest-12582.txt");
    let r523 = crate::testenv::dump("gamelog-run523-islands-toughest-13385.txt");
    let r535 = crate::testenv::dump("gamelog-run535-islands-toughest-14141.txt");
    let r544 = crate::testenv::dump("gamelog-run544-islands-toughest-15862.txt");
    let r572 = crate::testenv::dump("gamelog-run572-islands-toughest-16160.txt");
    let r579 = crate::testenv::dump("gamelog-run579-islands-toughest-16476.txt");
    let r583 = crate::testenv::dump("gamelog-run583-islands-toughest-16754.txt");
    let r594 = crate::testenv::dump("gamelog-run594-islands-toughest-18060.txt");
    let r610 = crate::testenv::dump("gamelog-run610-islands-french-toughest-8182.txt");
    let r612 = crate::testenv::dump("gamelog-run612-islands-french-8236.txt");
    let r616 = crate::testenv::dump("gamelog-run616-islands-french-8385.txt");
    let r617 = crate::testenv::dump("gamelog-run617-islands-french-8840.txt");
    let r622 = crate::testenv::dump("gamelog-run622-islands-french-9655.txt");
    let r623 = crate::testenv::dump("gamelog-run623-islands-french-9777.txt");
    let r624 = crate::testenv::dump("gamelog-run624-islands-french-10131.txt");
    let r629 = crate::testenv::dump("gamelog-run629-islands-french-10802.txt");
    let r631 = crate::testenv::dump("gamelog-run631-islands-french-11582.txt");
    let r634 = crate::testenv::dump("gamelog-run634-islands-french-12794.txt");
    let r635 = crate::testenv::dump("gamelog-run635-islands-french-12952.txt");
    let r636 = crate::testenv::dump("gamelog-run636-islands-french-14090.txt");
    let r639 = crate::testenv::dump("gamelog-run639-islands-french-14782.txt");
    let r642 = crate::testenv::dump("gamelog-run642-islands-french-15344.txt");
    let r655 = crate::testenv::dump("gamelog-run655-islands-french-16857.txt");
    let r657 = crate::testenv::dump("gamelog-run657-islands-french-17171.txt");
    let r661 = crate::testenv::dump("gamelog-run661-islands-french-17244.txt");
    let r662 = crate::testenv::dump("gamelog-run662-islands-french-17318.txt");
    let r603 = crate::testenv::dump("gamelog-run603-islands-french-toughest-7356.txt");
    let r602 = crate::testenv::dump("gamelog-run602-islands-french-toughest-986.txt");
    let r601 = crate::testenv::dump("gamelog-run601-lakes-french-toughest-2576.txt");
    let r589 = crate::testenv::dump("gamelog-run589-islands-toughest-17890.txt");
    let r588 = crate::testenv::dump("gamelog-run588-islands-toughest-17754.txt");
    let r585 = crate::testenv::dump("gamelog-run585-islands-toughest-17501.txt");
    let r373 = crate::testenv::dump("gamelog-run373-greatlakes-toughest-4846.txt");
    let r382 = crate::testenv::dump(super::third::SAHARA_SCORE.0);
    let r416 = crate::testenv::dump(super::third::SAHARA_WORD_12783);
    let r417 = crate::testenv::dump(super::third::SAHARA_WORD_13182);
    let r418 = crate::testenv::dump(super::third::SAHARA_WORD_14587);
    let r426 = crate::testenv::dump(super::third::SAHARA_WORD_15586);
    let r428 = crate::testenv::dump(super::third::SAHARA_WORD_15982);
    let r442 = crate::testenv::dump(super::third::SAHARA_WORD_16681);
    let r449 = crate::testenv::dump(super::third::SAHARA_WORD_17623);
    let r457 = crate::testenv::dump(super::third::SAHARA_GAP_17140);
    let r458 = crate::testenv::dump(super::third::SAHARA_END);
    let r471 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_5376);
    let r476 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_5782);
    let r483 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_7070);
    let r488 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_7785);
    let r491 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_8182);
    let r500 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_8786);
    let r511 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_9323);
    let r529 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_9764);
    let r547 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_10144);
    let r562 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_10779);
    let r571 = crate::testenv::dump(super::sahara_toughest::TOUGHEST_WORD_11182);
    let r44 = crate::testenv::dump("gamelog-run44-islands-turners.txt");
    if ch2.is_none() && r136.is_none() {
        eprintln!("skipping: neither the ch2 golden capture nor run136 is on disk");
        return;
    }
    let mut paths = Paths::new();
    let mut frames = 0;
    if let Some(p) = &ch2 {
        frames += drive_capture(
            p,
            WIDENING_CHAPTER_TWO.0,
            WIDENING_CHAPTER_TWO.1,
            &mut paths,
        );
    }
    // **The Great Lakes word's own blocks, on run123** (item 520). Item
    // 518 moved the word past run100's last block and left this window on
    // 10834, the last word run100 reached, until a capture reached the new
    // one. run123 does: run100's detail with `LEADERS=9`, over
    // `WIDENING_GREAT_LAKES_MARKET`, so every path run100 printed is
    // here too, and the leader record whole beside it.
    // Item 327 moved the headline to 11531, past run123's last block, and
    // item 533's run125 reaches it at run123's detail, so the window is
    // the word's own blocks again — keyed on the block run125 was taken to
    // widen, which stays on the capture whatever the headline does next.
    // Item 545 moved the headline to 11757, past run125's last block, and
    // item 554's run130 reaches it at run125's detail, so the window moved
    // to the word's own blocks again the same way. Item 557 moved it to
    // 11806, past run130's last block, and item 560's run135 reaches it at
    // run134's line — run125's detail with `GROUPS=1` and no `DEATHS` — so
    // the window is the word's own blocks on run135. Item 560 moved it to
    // 11903, past run135's last block, and item 563's run136 reaches it
    // at run135's line, so the window is the word's own blocks on run136.
    let gl = GREAT_LAKES_DETOUR_BLOCK;
    if let Some(p) = &r136 {
        frames += drive_capture(p, gl - 2, gl + 2, &mut paths);
    }
    // Item 566 moved it to 12038, past run136's last block, and item 571's
    // run163 is run136's line from 11950 to 12399, so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let gu = GREAT_LAKES_UPGRADE_BLOCK;
    if let Some(p) = &r163 {
        let n = drive_capture(p, gu - 2, gu + 2, &mut paths);
        assert_eq!(n, 5, "run163 carries the word's five blocks");
        frames += n;
        // Item 657 moved it to 12184, inside run163, so the new word's
        // own blocks are read on the same capture.
        let gm = GREAT_LAKES_MAKE_BLOCK;
        let n = drive_capture(p, gm - 2, gm + 2, &mut paths);
        assert_eq!(n, 5, "run163 carries the new word's five blocks");
        frames += n;
    }
    // Item 661 moved it to 12429, past run163's last block, and item 669's
    // run174 is run163's line from 12390 to 12899, so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let gc = GREAT_LAKES_CIVIC_BLOCK;
    if let Some(p) = &r174 {
        let n = drive_capture(p, gc - 2, gc + 2, &mut paths);
        assert_eq!(n, 5, "run174 carries the word's five blocks");
        frames += n;
        // Item 669 moved it to 12536, inside run174, and left this window
        // on 12430; item 673 reads the new word's own blocks on the same
        // capture before judging a fix on them.
        let gs = GREAT_LAKES_SQUAD_BLOCK;
        let n = drive_capture(p, gs - 2, gs + 2, &mut paths);
        assert_eq!(n, 5, "run174 carries the new word's five blocks");
        frames += n;
        // Item 673 moved it to 12897, whose block is run174's last but
        // one, so the window ends on the capture's last block.
        let gr = GREAT_LAKES_RETRY_BLOCK;
        let n = drive_capture(p, gr - 2, gr + 1, &mut paths);
        assert_eq!(n, 4, "run174 carries the new word's four blocks");
        frames += n;
    }
    // Item 678 moved it to 14382, past run174's last block, and its run178
    // is run174's line from 12894 to 14899, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    let gy = GREAT_LAKES_COPY_BLOCK;
    if let Some(p) = &r178 {
        let n = drive_capture(p, gy - 2, gy + 2, &mut paths);
        assert_eq!(n, 5, "run178 carries the word's five blocks");
        frames += n;
        // Item 688 moved it to 14529, inside run178, and reads the new
        // word's own blocks on the same capture.
        let gv = GREAT_LAKES_VALS_BLOCK;
        let n = drive_capture(p, gv - 2, gv + 2, &mut paths);
        assert_eq!(n, 5, "run178 carries the new word's five blocks");
        frames += n;
        // Item 695 moved it to 14650, inside run178 again (the caravan's
        // road, verified over a stretch the stray-road sweep took).
        let gd = GREAT_LAKES_ROAD_BLOCK;
        let n = drive_capture(p, gd - 2, gd + 2, &mut paths);
        assert_eq!(n, 5, "run178 carries item 695's word's five blocks");
        frames += n;
    }
    // Item 698 moved it to 14982, past run178's last block, and its run192
    // is run178's line from 14894 to 15039, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    let gb = GREAT_LAKES_BIRTH_BLOCK;
    if let Some(p) = &r192 {
        let n = drive_capture(p, gb - 2, gb + 2, &mut paths);
        assert_eq!(n, 5, "run192 carries the word's five blocks");
        frames += n;
    }
    // Item 706 moved it to 15175, past run192's last block, and its run196
    // is run192's line from 15034 to 15232, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    let gp = GREAT_LAKES_PATRIOT_BLOCK;
    if let Some(p) = &r196 {
        let n = drive_capture(p, gp - 2, gp + 2, &mut paths);
        assert_eq!(n, 5, "run196 carries the word's five blocks");
        frames += n;
    }
    // Item 711 moved it to 15383, past run196's last block, and its run202
    // is run196's line from 15227 to 15440, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    // Item 715 moved it to 15384, inside run202, so the window runs on to
    // two blocks past the new word's block.
    let gm = GREAT_LAKES_MIRROR_BLOCK;
    let gr = GREAT_LAKES_RECRUIT_BLOCK;
    if let Some(p) = &r202 {
        let n = drive_capture(p, gm - 2, gr + 2, &mut paths);
        assert_eq!(n, 6, "run202 carries both words' blocks");
        frames += n;
    }
    // Item 722 moved it to 15608, past run202's last block, and its run211
    // is run202's line from 15435 to 15859, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    // Item 729 moved it to 15619, inside run211, so the window runs on to
    // two blocks past the new word's block.
    let gw = GREAT_LAKES_WONDER_BLOCK;
    let ga = GREAT_LAKES_ATTACKED_BLOCK;
    if let Some(p) = &r211 {
        let n = drive_capture(p, gw - 2, ga + 2, &mut paths);
        assert_eq!(n, 16, "run211 carries both words' blocks");
        frames += n;
    }
    // Item 736 moved it to 16460, past run211's last block, and its run218
    // is run211's line from 15854 to 16711, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    let ge = GREAT_LAKES_ESCORT_BLOCK;
    if let Some(p) = &r218 {
        let n = drive_capture(p, ge - 2, ge + 2, &mut paths);
        assert_eq!(n, 5, "run218 carries the word's five blocks");
        frames += n;
    }
    // Item 742 moved it to 17099, past run218's last block, and its run226
    // is run218's line from 16706 to 17350, so the window is the word's own
    // blocks again, on the capture taken to widen it.
    let gg = GREAT_LAKES_GIVEUP_BLOCK;
    if let Some(p) = &r226 {
        let n = drive_capture(p, gg - 2, gg + 2, &mut paths);
        assert_eq!(n, 5, "run226 carries the word's five blocks");
        frames += n;
        // Item 757 moved it to 17128, still inside run226.
        let gp = GREAT_LAKES_PYRAMIDS_BLOCK;
        let n = drive_capture(p, gp - 2, gp + 2, &mut paths);
        assert_eq!(n, 5, "run226 carries the new word's five blocks");
        frames += n;
        // Item 776 moved it to 17181, still inside run226.
        let gf = GREAT_LAKES_FOREST_CELL_BLOCK;
        let n = drive_capture(p, gf - 2, gf + 2, &mut paths);
        assert_eq!(n, 5, "run226 carries item 776's word's five blocks");
        frames += n;
    }
    // Item 785 moved it to 20568, past run226: run243 carries its blocks.
    if let Some(p) = &r243 {
        let gs = GREAT_LAKES_STAND_BLOCK;
        let n = drive_capture(p, gs - 2, gs + 2, &mut paths);
        assert_eq!(n, 5, "run243 carries the word's five blocks");
        frames += n;
        // Item 795 moved it to 20800, still inside run243.
        let gr = GREAT_LAKES_RETURN_BLOCK;
        let n = drive_capture(p, gr - 2, gr + 2, &mut paths);
        assert_eq!(n, 5, "run243 carries item 795's word's five blocks");
        frames += n;
    }
    // Item 899 moved it to 24000, the capture's own end: run80 carries the
    // last blocks any dump holds. The window is 23997..23999, its last three
    // per-frame blocks. The end block 24001 is the quit dump, which adds a
    // `GAME` record and the `WORLD` territory limits no reader parses —
    // not a per-frame record, and left out of the window rather than pinned
    // (item 899's journal names it for parking).
    if let Some(p) = &r80 {
        let end = WIDENING_GREAT_LAKES_END.1;
        let n = drive_capture(p, end - 4, end - 1, &mut paths);
        assert_eq!(n, 3, "run80 carries the end's last three per-frame blocks");
        frames += n;
    }
    // **East Indies' word's own blocks, on run99** (item 573): the lower
    // map's headline since 2026-09-21 and a window this guard had never
    // read. run99 is run98's detail — `DEATHS=1` and `LEADERS=1`, where the
    // Great Lakes line has `GROUPS=1` and `LEADERS=9`.
    // Item 576 moved the word to 10232, and run99 carries its blocks too.
    // Item 579 moved it to 10398, whose block is run99's last, so the five
    // blocks ended on it rather than straddled it. Item 588's run143 is
    // run99's line at `LEADERS=9` from 10380 to 10739, so the window
    // straddles the word's block again, on the capture taken to widen it.
    let ei = EAST_INDIES_BARK_BLOCK;
    if let Some(p) = &r143 {
        let n = drive_capture(p, ei - 2, ei + 2, &mut paths);
        assert_eq!(n, 5, "run143 carries the word's five blocks");
        frames += n;
    }
    // Item 604 moved the word to 10782, past run143's last block, and item
    // 608's run149 is run143's line from 10730 to 10879, so the window is
    // the word's own blocks again, on the capture taken to widen it. The
    // same item moved the word past run149, and the window stays on the
    // block run149 was taken to widen.
    let ew = EAST_INDIES_MERCS_BLOCK;
    if let Some(p) = &r149 {
        let n = drive_capture(p, ew - 2, ew + 2, &mut paths);
        assert_eq!(n, 5, "run149 carries the word's five blocks");
        frames += n;
    }
    // Item 608 moved the word to 10982, past run149's last block, and item
    // 613's run152 is run149's line from 10870 to 11039, so the window is
    // the word's own blocks again, on the capture taken to widen it.
    let eg = EAST_INDIES_GATHER_BLOCK;
    if let Some(p) = &r152 {
        let n = drive_capture(p, eg - 2, eg + 2, &mut paths);
        assert_eq!(n, 5, "run152 carries the word's five blocks");
        frames += n;
    }
    // Item 613 moved the word to 11069, past run152's last block, and item
    // 620's run155 is run152's line from 11030 to 11279, so the window is
    // the word's own blocks again, on the capture taken to widen it.
    let ex = EAST_INDIES_EXPLORE_BLOCK;
    if let Some(p) = &r155 {
        let n = drive_capture(p, ex - 2, ex + 2, &mut paths);
        assert_eq!(n, 5, "run155 carries the word's five blocks");
        frames += n;
    }
    // Item 620 moved the word to 11590, past run155's last block, and item
    // 629's run159 is run155's line from 11270 to 11899, so the window is
    // the word's own blocks again, on the capture taken to widen it.
    let ei2 = EAST_INDIES_IDLE_BLOCK;
    if let Some(p) = &r159 {
        let n = drive_capture(p, ei2 - 2, ei2 + 2, &mut paths);
        assert_eq!(n, 5, "run159 carries the word's five blocks");
        frames += n;
    }
    // Item 629 moved the word to 11747, inside run159, so the word's own
    // blocks are read there too (item 642).
    let ec = EAST_INDIES_CAST_BLOCK;
    if let Some(p) = &r159 {
        let n = drive_capture(p, ec - 2, ec + 2, &mut paths);
        assert_eq!(n, 5, "run159 carries the cast word's five blocks");
        frames += n;
    }
    // Item 642 moved the word to 13640, past every East Indies dump, and
    // item 643's run166 is run159's line over [13580, 13700), so the window
    // is the word's own blocks again, on the capture taken to widen it.
    let ew2 = EAST_INDIES_WRAP_BLOCK;
    if let Some(p) = &r166 {
        let n = drive_capture(p, ew2 - 2, ew2 + 2, &mut paths);
        assert_eq!(n, 5, "run166 carries the word's five blocks");
        frames += n;
    }
    // Item 706 moved the word to 15985, past run78's last block, and item
    // 708's run221 is run166's line without `DEATHS` over [15894, 16237),
    // so the window is the word's own blocks again, on the capture taken
    // to widen it.
    let es = EAST_INDIES_SLOT_BLOCK;
    if let Some(p) = &r221 {
        let n = drive_capture(p, es - 2, es + 2, &mut paths);
        assert_eq!(n, 5, "run221 carries the word's five blocks");
        frames += n;
    }
    // Item 708 moved the word to 16683, past run221's last block, and item
    // 752's run227 is run221's line over [16230, 16935), so the window is
    // the word's own blocks again, on the capture taken to widen it.
    let eww = EAST_INDIES_WRAPWORD_BLOCK;
    if let Some(p) = &r227 {
        let n = drive_capture(p, eww - 2, eww + 2, &mut paths);
        assert_eq!(n, 5, "run227 carries the word's five blocks");
        frames += n;
    }
    // Item 767 moved the word to 17189, past run227's last block, and its
    // run233 is run227's line over [16929, 17441), so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let ebw = EAST_INDIES_BLOCKWORD_BLOCK;
    if let Some(p) = &r233 {
        let n = drive_capture(p, ebw - 2, ebw + 2, &mut paths);
        assert_eq!(n, 5, "run233 carries the word's five blocks");
        frames += n;
    }
    // Item 773 moved the word to 17403, still inside run233, so the new
    // word's own blocks are driven on the same capture.
    let egw = EAST_INDIES_GROUPWORD_BLOCK;
    if let Some(p) = &r233 {
        let n = drive_capture(p, egw - 2, egw + 2, &mut paths);
        assert_eq!(n, 5, "run233 carries the new word's five blocks");
        frames += n;
    }
    // Item 800 moved the word to 17501, past run233's last block, and its
    // run251 is run233's line over [17496, 17753), so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let ecw = EAST_INDIES_COLUMNWORD_BLOCK;
    if let Some(p) = &r251 {
        let n = drive_capture(p, ecw - 2, ecw + 2, &mut paths);
        assert_eq!(n, 5, "run251 carries the word's five blocks");
        frames += n;
    }
    // Item 811 moved the word to 18182, past run251's last block, and its
    // run253 is run251's line over [18177, 18434), so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let emw = EAST_INDIES_MARKETWORD_BLOCK;
    if let Some(p) = &r253 {
        let n = drive_capture(p, emw - 2, emw + 2, &mut paths);
        assert_eq!(n, 5, "run253 carries the word's five blocks");
        frames += n;
    }
    // Item 822 moved the word to 18938, past run253's last block, and its
    // run257 is run253's line over [18933, 19190), so the window is the
    // word's own blocks again, on the capture taken to widen it.
    let egw = EAST_INDIES_GUARDWORD_BLOCK;
    if let Some(p) = &r257 {
        let n = drive_capture(p, egw - 2, egw + 2, &mut paths);
        assert_eq!(n, 5, "run257 carries the word's five blocks");
        frames += n;
    }
    // Item 829 moved the word to 18999, inside run257, so its five blocks
    // are driven on the same capture.
    let efw = EAST_INDIES_FIGUREWORD_BLOCK;
    if let Some(p) = &r257 {
        let n = drive_capture(p, efw - 2, efw + 2, &mut paths);
        assert_eq!(n, 5, "run257 carries the new word's five blocks");
        frames += n;
    }
    // Item 837 moved the word to 19182, inside run257 still, six blocks
    // from its end.
    let elw = EAST_INDIES_LEADERWORD_BLOCK;
    if let Some(p) = &r257 {
        let n = drive_capture(p, elw - 2, elw + 2, &mut paths);
        assert_eq!(n, 5, "run257 carries item 837's word's five blocks");
        frames += n;
    }
    // Item 839 moved the word to 19413, past run257's end, and run269 is
    // run257's line over [19408, 19665), taken to widen it.
    let etw = EAST_INDIES_TURNWORD_BLOCK;
    if let Some(p) = &r269 {
        let n = drive_capture(p, etw - 2, etw + 2, &mut paths);
        assert_eq!(n, 5, "run269 carries the word's five blocks");
        frames += n;
    }
    // Item 850 moved the word to 19509, inside run269 still.
    let eww = EAST_INDIES_WALKWORD_BLOCK;
    if let Some(p) = &r269 {
        let n = drive_capture(p, eww - 2, eww + 2, &mut paths);
        assert_eq!(n, 5, "run269 carries item 850's word's five blocks");
        frames += n;
    }
    // Item 857 moved the word to 20007, past run269's end, and run277 is
    // run269's line over [20002, 20259), taken to widen it.
    let ebw = EAST_INDIES_BLOCKEDWALK_BLOCK;
    if let Some(p) = &r277 {
        let n = drive_capture(p, ebw - 2, ebw + 2, &mut paths);
        assert_eq!(n, 5, "run277 carries the word's five blocks");
        frames += n;
    }
    // Item 880 moved the word to 20782, past run277's end, and run289 is
    // run277's line over [20777, 21034), taken to widen it.
    let emw = EAST_INDIES_USEMARKET_BLOCK;
    if let Some(p) = &r289 {
        let n = drive_capture(p, emw - 2, emw + 2, &mut paths);
        assert_eq!(n, 5, "run289 carries the word's five blocks");
        frames += n;
    }
    // Item 890 moved the word to 23182, past run289's end, and run299 is
    // run289's line over [23177, 23434), taken to widen it.
    let ewp = EAST_INDIES_WONDERPRICE_BLOCK;
    if let Some(p) = &r299 {
        let n = drive_capture(p, ewp - 2, ewp + 2, &mut paths);
        assert_eq!(n, 5, "run299 carries the word's five blocks");
        frames += n;
        // Item 904 moved the word to 23420 inside run299's window.
        let ean = EAST_INDIES_ADDANIMALS_BLOCK;
        let n = drive_capture(p, ean - 2, ean + 2, &mut paths);
        assert_eq!(n, 5, "run299 carries the new word's five blocks");
        frames += n;
    }
    // Item 919 moved it to 24000, the capture's own end: run96 carries the
    // last blocks any dump holds, and the window is its last three
    // per-frame blocks, 23997..23999, for run80's reason (the end block is
    // the quit dump).
    if let Some(p) = &r96 {
        let end = WIDENING_EAST_INDIES_END.1;
        let n = drive_capture(p, end - 4, end - 1, &mut paths);
        assert_eq!(n, 3, "run96 carries the end's last three per-frame blocks");
        frames += n;
    }
    // **The second pair** (item 971): East Indies at Toughest parts on
    // frame 0, which writes block 1, and run349 is run346's game over
    // blocks 1..250 — the window is the word's block and the two after it.
    if let Some(p) = &r349 {
        let n = drive_capture(p, 1, 3, &mut paths);
        assert_eq!(n, 3, "run349 carries the second pair's word's three blocks");
        frames += n;
    }
    // And Great Lakes', the lower word after the coin (frame 1, block 2),
    // on run350: the block before it, the word's and the two after.
    if let Some(p) = &r350 {
        let n = drive_capture(p, 1, 4, &mut paths);
        assert_eq!(n, 4, "run350 carries the second pair's lower word's blocks");
        frames += n;
    }
    // Item 979 moved both words past those windows: East Indies' 1576
    // (block 1577) on run352 and Great Lakes' 3776 (block 3777) on run355,
    // each the block before the word's, the word's and the two after.
    if let Some(p) = &r352 {
        let n = drive_capture(p, 1_576, 1_579, &mut paths);
        assert_eq!(
            n, 4,
            "run352 carries the second pair's East Indies word's blocks"
        );
        frames += n;
    }
    if let Some(p) = &r355 {
        let n = drive_capture(p, 3_776, 3_779, &mut paths);
        assert_eq!(
            n, 4,
            "run355 carries the second pair's Great Lakes word's blocks"
        );
        frames += n;
    }
    // Item 989 moved both again: East Indies' 5606 (block 5607) on run357
    // and Great Lakes' 4555 (block 4556) on run356, the same four blocks.
    if let Some(p) = &r357 {
        let n = drive_capture(p, 5_606, 5_609, &mut paths);
        assert_eq!(
            n, 4,
            "run357 carries the second pair's East Indies word's blocks"
        );
        frames += n;
    }
    // Item 1115 moved East Indies to 5975 (block 5976), past run357: run414
    // is its widening.
    if let Some(p) = &r414 {
        let n = drive_capture(p, 5_975, 5_978, &mut paths);
        assert_eq!(
            n, 4,
            "run414 carries the second pair's East Indies word's blocks"
        );
        frames += n;
    }
    // Item 1120 moved it to 6151 (block 6152), inside run414.
    if let Some(p) = &r414 {
        let n = drive_capture(p, 6_151, 6_154, &mut paths);
        assert_eq!(
            n, 4,
            "run414 carries the second pair's East Indies word 6151's blocks"
        );
        frames += n;
    }
    // Item 1127 moved it to 6321 (block 6322), past run414: run419 is its
    // widening.
    if let Some(p) = &r419 {
        let n = drive_capture(p, 6_321, 6_324, &mut paths);
        assert_eq!(
            n, 4,
            "run419 carries the second pair's East Indies word 6321's blocks"
        );
        frames += n;
    }
    // Item 1143 moved it to 6609 (block 6610), past run419: run420 is its
    // widening. Item 1156 moved it to 6743 (block 6744), inside run420.
    if let Some(p) = &r420 {
        let n = drive_capture(p, 6_609, 6_612, &mut paths);
        assert_eq!(
            n, 4,
            "run420 carries the second pair's East Indies word 6609's blocks"
        );
        frames += n;
        let n = drive_capture(p, 6_743, 6_746, &mut paths);
        assert_eq!(
            n, 4,
            "run420 carries the second pair's East Indies word 6743's blocks"
        );
        frames += n;
    }
    // Item 1164 moved it to 7382 (block 7383), past run420: run425 is its
    // widening.
    if let Some(p) = &r425 {
        let n = drive_capture(p, 7_382, 7_385, &mut paths);
        assert_eq!(
            n, 4,
            "run425 carries the second pair's East Indies word 7382's blocks"
        );
        frames += n;
        // Item 1174 moved it to 7512 (block 7513), inside run425.
        let n = drive_capture(p, 7_512, 7_515, &mut paths);
        assert_eq!(
            n, 4,
            "run425 carries the second pair's East Indies word 7512's blocks"
        );
        frames += n;
    }
    // Item 1185 moved it to 8519 (block 8520), past run425: run439 is its
    // widening.
    if let Some(p) = &r439 {
        let n = drive_capture(p, 8_519, 8_522, &mut paths);
        assert_eq!(
            n, 4,
            "run439 carries the second pair's East Indies word 8519's blocks"
        );
        frames += n;
    }
    // Item 1191 moved it to 8820 (block 8821), past run439: run445 is its
    // widening.
    if let Some(p) = &r445 {
        let n = drive_capture(p, 8_820, 8_823, &mut paths);
        assert_eq!(
            n, 4,
            "run445 carries the second pair's East Indies word 8820's blocks"
        );
        frames += n;
        // Item 1197 moved it to 8907 (block 8908), inside run445.
        let n = drive_capture(p, 8_907, 8_910, &mut paths);
        assert_eq!(
            n, 4,
            "run445 carries the second pair's East Indies word 8907's blocks"
        );
        frames += n;
    }
    // Item 1214 moved it to 10183 (block 10184), past run445: run462 is its
    // widening.
    if let Some(p) = &r462 {
        let n = drive_capture(p, 10_183, 10_186, &mut paths);
        assert_eq!(
            n, 4,
            "run462 carries the second pair's East Indies word 10183's blocks"
        );
        frames += n;
    }
    // Item 1243 moved it to 10985 (block 10986), past run462: run480 is
    // its widening.
    if let Some(p) = &r480 {
        let n = drive_capture(p, 10_985, 10_988, &mut paths);
        assert_eq!(
            n, 4,
            "run480 carries the second pair's East Indies word 10985's blocks"
        );
        frames += n;
    }
    // Item 1264 moved it to 11328 (block 11329), past run480: run490 is
    // its widening.
    if let Some(p) = &r490 {
        let n = drive_capture(p, 11_328, 11_331, &mut paths);
        assert_eq!(
            n, 4,
            "run490 carries the second pair's East Indies word 11328's blocks"
        );
        frames += n;
        // Item 1281 moved it to 11549 (block 11550), inside run490.
        let n = drive_capture(p, 11_549, 11_552, &mut paths);
        assert_eq!(
            n, 4,
            "run490 carries the second pair's East Indies word 11549's blocks"
        );
        frames += n;
    }
    // Item 1297 moved it to 11637 (block 11638), past run490: run506 is
    // its widening.
    if let Some(p) = &r506 {
        let n = drive_capture(p, 11_637, 11_640, &mut paths);
        assert_eq!(
            n, 4,
            "run506 carries the second pair's East Indies word 11637's blocks"
        );
        frames += n;
    }
    // Item 1302 moved it to 12582 (block 12583), past run506: run508 is
    // its widening.
    if let Some(p) = &r508 {
        let n = drive_capture(p, 12_582, 12_585, &mut paths);
        assert_eq!(
            n, 4,
            "run508 carries the second pair's East Indies word 12582's blocks"
        );
        frames += n;
    }
    // Item 1326 moved it to 13385 (block 13386), past run508: run523 is
    // its widening.
    if let Some(p) = &r523 {
        let n = drive_capture(p, 13_385, 13_388, &mut paths);
        assert_eq!(
            n, 4,
            "run523 carries the second pair's East Indies word 13385's blocks"
        );
        frames += n;
    }
    // Item 1341 moved it to 14141 (block 14142), past run523: run535 is
    // its widening.
    if let Some(p) = &r535 {
        let n = drive_capture(p, 14_141, 14_144, &mut paths);
        assert_eq!(
            n, 4,
            "run535 carries the second pair's East Indies word 14141's blocks"
        );
        frames += n;
    }
    // Item 1351 moved it to 15862 (block 15863), past run535: run544 is
    // its widening.
    if let Some(p) = &r544 {
        let n = drive_capture(p, 15_862, 15_865, &mut paths);
        assert_eq!(
            n, 4,
            "run544 carries the second pair's East Indies word 15862's blocks"
        );
        frames += n;
        // Item 1362 moved it to 15883 (block 15884), inside run544.
        let n = drive_capture(p, 15_883, 15_886, &mut paths);
        assert_eq!(
            n, 4,
            "run544 carries the second pair's East Indies word 15883's blocks"
        );
        frames += n;
        // Item 1370 moved it to 15985 (block 15986), inside run544.
        let n = drive_capture(p, 15_985, 15_988, &mut paths);
        assert_eq!(
            n, 4,
            "run544 carries the second pair's East Indies word 15985's blocks"
        );
        frames += n;
        // Item 1377 moved it to 16009 (block 16010), inside run544.
        let n = drive_capture(p, 16_009, 16_012, &mut paths);
        assert_eq!(
            n, 4,
            "run544 carries the second pair's East Indies word 16009's blocks"
        );
        frames += n;
    }
    // Item 1383 moved it to 16160 (block 16161), past run544: run572 is
    // its widening.
    // Item 1401 moved it to 16179, inside run572; item 1407 to 16221; item 1410 to 16250.
    if let Some(p) = &r572 {
        let n = drive_capture(p, 16_250, 16_253, &mut paths);
        assert_eq!(
            n, 4,
            "run572 carries the second pair's East Indies word 16250's blocks"
        );
        frames += n;
    }
    // Item 1418 moved it to 16482 (block 16483), past run572: run579 is
    // its widening.
    if let Some(p) = &r579 {
        let n = drive_capture(p, 16_482, 16_485, &mut paths);
        assert_eq!(
            n, 4,
            "run579 carries the second pair's East Indies word 16482's blocks"
        );
        frames += n;
    }
    // Item 1427's measured successor, including the prior 16760 word.
    if let Some(p) = &r583 {
        let n = drive_capture(p, 16_939, 16_944, &mut paths);
        assert_eq!(n, 6, "run583 carries the previous East Indies word");
        frames += n;
    }
    if let Some(p) = &r585 {
        let n = drive_capture(p, 17_697, 17_702, &mut paths);
        assert_eq!(n, 6, "run585 carries the previous East Indies word");
        frames += n;
    }
    if let Some(p) = &r588 {
        let n = drive_capture(p, 17_784, 17_789, &mut paths);
        assert_eq!(n, 6, "run588 carries the previous East Indies word");
        frames += n;
    }
    if let Some(p) = &r589 {
        let n = drive_capture(p, 17_943, 17_948, &mut paths);
        assert_eq!(n, 6, "run589 carries the previous East Indies word");
        frames += n;
    }
    if let Some(p) = &r594 {
        let n = drive_capture(p, 18_136, 18_141, &mut paths);
        assert_eq!(n, 6, "run594 carries East Indies through its closing state");
        frames += n;
    }
    if let Some(p) = &r356 {
        let n = drive_capture(p, 4_555, 4_558, &mut paths);
        assert_eq!(
            n, 4,
            "run356 carries the second pair's Great Lakes word's blocks"
        );
        frames += n;
    }
    // Item 1040 moved Great Lakes to 4846 (block 4847), on run373.
    if let Some(p) = &r373 {
        let n = drive_capture(p, 4_846, 4_849, &mut paths);
        assert_eq!(
            n, 4,
            "run373 carries the second pair's Great Lakes word's blocks"
        );
        frames += n;
    }
    // **The third map's first word, on run382** (item 1066): Great Sahara's
    // frame 8 writes block 9, and the window is it with two either side.
    // Item 1133 moved the word past run382; the block stays driven. run10's
    // detail on a map no other window here is on.
    if let Some(p) = &r382 {
        let b = crate::diff::third::SAHARA_FIRST_WORD_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run382 carries the third map's word's blocks");
        frames += n;
    }
    // **The third map's word, on run416** (item 1133): frame 12783 writes
    // block 12784, and the window is it with two either side, at run414's
    // detail. Item 1147 moved the word to 13182, on run417, and run416's
    // window stays as the move's.
    if let Some(p) = &r416 {
        let b = super::third::SAHARA_12783_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run416 carries the old word's blocks");
        frames += n;
    }
    // Item 1163 moved the word to 14587, on run418, and run417's window
    // stays as the move's.
    if let Some(p) = &r417 {
        let b = super::third::SAHARA_13182_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run417 carries the old word's blocks");
        frames += n;
    }
    // Item 1171 moved the word to 15586, on run426, and run418's window
    // stays as the move's.
    if let Some(p) = &r418 {
        let b = super::third::SAHARA_14587_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run418 carries the old word's blocks");
        frames += n;
    }
    // Item 1177 moved it to 15982, on run428, and run426's window stays
    // as the move's.
    if let Some(p) = &r426 {
        let b = super::third::SAHARA_15586_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run426 carries the old word's blocks");
        frames += n;
    }
    // Item 1189 moved it to 16681, on run442, and run428's window stays
    // as the move's.
    if let Some(p) = &r428 {
        let b = super::third::SAHARA_15982_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run428 carries the old word's blocks");
        frames += n;
    }
    // Item 1194 moved it to 17623, on run449, and run442's window stays
    // as the move's.
    if let Some(p) = &r442 {
        let b = super::third::SAHARA_16681_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run442 carries the old word's blocks");
        frames += n;
    }
    // Item 1206 moved it to 24000, the trace's end: run449's window stays
    // as the move's, and run457's block 17493 is the move's value diff.
    if let Some(p) = &r449 {
        let b = super::third::SAHARA_17623_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run449 carries the old word's blocks");
        frames += n;
    }
    if let Some(p) = &r457 {
        let b = super::third::SAHARA_GAP_17493_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run457 carries the move's blocks");
        frames += n;
    }
    // **And the word's own, run458's last blocks** (item 1206): the word
    // 24000 writes the closing block 24001, the last any dump holds.
    if let Some(p) = &r458 {
        let b = super::testkit::LONG_WORD_GREAT_SAHARA + 1;
        let n = drive_capture(p, b - 4, b, &mut paths);
        assert!(n >= 4, "run458 carries the third map's closing blocks: {n}");
        frames += n;
    }
    // **The third map at Toughest, on run471** (item 1221, DECISIONS 56
    // §1): its first word, frame 5376, writes block 5377, and the window is
    // it with two either side, at run449's detail.
    if let Some(p) = &r471 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run471 carries the third map's word at Toughest");
        frames += n;
    }
    // **And its word 5782, on run476** (item 1241): the frame writes block
    // 5783, and the window is it with two either side.
    if let Some(p) = &r476 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_5783;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run476 carries the third map's word 5782 at Toughest");
        frames += n;
    }
    // **And its word 7070, on run483** (item 1251): the frame writes block
    // 7071, and the window is it with two either side.
    if let Some(p) = &r483 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_7071;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run483 carries the third map's word 7070 at Toughest");
        frames += n;
    }
    // **And its word 7785, on run488** (item 1260): the frame writes block
    // 7786, and the window is it with two either side.
    if let Some(p) = &r488 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_7786;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run488 carries the third map's word 7785 at Toughest");
        frames += n;
    }
    // **And its word 8377, on run491** (item 1275): the frame writes block
    // 8378, and the window is it with two either side.
    if let Some(p) = &r491 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_8378;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run491 carries the third map's word 8377 at Toughest");
        frames += n;
    }
    // **And its word 8786, on run500** (item 1286): the frame writes block
    // 8787, and the window is it with two either side.
    if let Some(p) = &r500 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_8787;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run500 carries the third map's word 8786 at Toughest");
        frames += n;
    }
    // **And its word 8856, on the same capture** (item 1293): the frame
    // writes block 8857, and the window is it with two either side.
    if let Some(p) = &r500 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_8857;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run500 carries the third map's word 8856 at Toughest");
        frames += n;
    }
    // **And its word 9323, on run511** (item 1305): the frame writes block
    // 9324, and the window is it with two either side.
    if let Some(p) = &r511 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_9324;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run511 carries the third map's word 9323 at Toughest");
        frames += n;
    }
    // **And its word 9352, on the same capture** (item 1318): the frame
    // writes block 9353, and the window is it with two either side.
    if let Some(p) = &r511 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_9353;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run511 carries the third map's word 9352 at Toughest");
        frames += n;
    }
    // **And its word 9764, on run529** (item 1332): the frame writes block
    // 9765, and the window is it with two either side.
    if let Some(p) = &r529 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_9765;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run529 carries the third map's word 9764 at Toughest");
        frames += n;
    }
    // **And its word 9982, on the same capture** (item 1338): the frame
    // writes block 9983, and the window is it with two either side.
    if let Some(p) = &r529 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_9983;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run529 carries the third map's word 9982 at Toughest");
        frames += n;
    }
    // **And its word 9999** (item 1346): the frame writes block 10000.
    if let Some(p) = &r529 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_10000;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(n, 5, "run529 carries the third map's word 9999 at Toughest");
        frames += n;
    }
    // **And its word 10144, on run547** (item 1354): the frame writes
    // block 10145, past run529's last block 10015.
    if let Some(p) = &r547 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_10145;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(
            n, 5,
            "run547 carries the third map's word 10144 at Toughest"
        );
        frames += n;
    }
    // **And its word 10391** (item 1365): the frame writes block 10392,
    // inside run547's window.
    if let Some(p) = &r547 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_10392;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(
            n, 5,
            "run547 carries the third map's word 10391 at Toughest"
        );
        frames += n;
    }
    // **And its word 10779, on run562** (item 1371): the frame writes
    // block 10780, past run547's last block 10395.
    if let Some(p) = &r562 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_10780;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(
            n, 5,
            "run562 carries the third map's word 10779 at Toughest"
        );
        frames += n;
    }
    // **And its word 11182, on run571** (item 1379): the frame writes
    // block 11183, past run562's last block 11030.
    if let Some(p) = &r571 {
        let b = super::sahara_toughest::TOUGHEST_WORD_BLOCK_11383;
        let n = drive_capture(p, b - 2, b + 2, &mut paths);
        assert_eq!(
            n, 5,
            "run571 carries the third map's word 11382 at Toughest"
        );
        frames += n;
    }
    // **And on run139** (item 576): run99's line with `LEADERS=9`, over the
    // five blocks around the make list it was taken for, so the leader
    // record's paths are on this map's window too —
    // `run139_s_word_frame_is_widened_whole` reads them.
    let em = EAST_INDIES_MAKE_BLOCK;
    if let Some(p) = &r139 {
        let n = drive_capture(p, em - 2, em + 2, &mut paths);
        assert_eq!(n, 5, "run139 carries the word's five blocks");
        frames += n;
    }
    // **The rules headline's own blocks, on run146** (item 621): the
    // restage's word, whose catapult holds the first
    // `ATTACKGROUNDORDER` on a headline frame. No window here had one,
    // so the record's four keys could arrive unread and read as quiet.
    //
    // **Item 627 closed the restage at the capture's end**, so the window
    // is the one the ground order and its aftermath print on: its last
    // hold, 863, the ready block, and the catapult's one-block
    // retaliations on 868, 870 and 871 (`docs/COMBAT.md` §60).
    assert_eq!(GOLDEN_WORD_CHAPTER_THREE_RESTAGE, 1000, "run146 reopened");
    if let Some(p) = &ch3b {
        let n = drive_capture(p, 863, 872, &mut paths);
        assert_eq!(n, 10, "run146 carries the ground order's ten blocks");
        frames += n;
    }
    // **And the clock the restage's word is read on, on run44** (item
    // 625): run146 prints no figure clock (`GUYS=2`), so the word's
    // widening reads the crew's `cur_anim`, `cur_time`, `end_time`,
    // `last_time`, `des_x` and `last_speed` off run44's `0/15`, the same
    // type turning in place on its ground order's push, 323–330
    // (`a_turning_catapult_s_crew_mirror_and_never_walk`).
    if let Some(p) = &r44 {
        let n = drive_capture(p, 323, 330, &mut paths);
        assert_eq!(n, 8, "run44 carries the turn's eight blocks");
        frames += n;
    }
    // **Chapter seven-b's two words, on run156 and run157** (item 628):
    // the computer's civilians — a citizen, a caravan, a merchant, a
    // scholar and a fur trapper — which no other window here carries.
    // Item 629 closed run156 at 1200, the capture's end, so its window
    // stays on the block run156 was taken to widen — the old word 1148,
    // the fur trapper's turn — as the East Indies windows stay on theirs.
    // Item 647 closed run157 at 1200 too, and its window stays on 1187,
    // the scout's word it was widened on.
    let _ = (
        GOLDEN_WORD_CHAPTER_SEVEN_B,
        GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
    );
    for (p, word) in [(&ch7b, 1_148), (&ch7bc, 1_187)] {
        if let Some(p) = p {
            let n = drive_capture(p, word - 2, word + 2, &mut paths);
            assert_eq!(n, 5, "chapter seven-b carries the word's five blocks");
            frames += n;
        }
    }
    // **Chapter six's word, on run168** (item 648): the first capture
    // with an aircraft in it — a Fighter and a Bomber staged outside any
    // base — whose records no other window here carries. The window is
    // the word's block with two on either side, as seven-b's.
    // Item 652 closed it at 900, the capture's end, so the window stays
    // on 700, the block it was widened on, as seven-b's stay on theirs.
    let _ = GOLDEN_WORD_CHAPTER_SIX;
    if let Some(p) = &ch6 {
        let w = 700;
        let n = drive_capture(p, w - 2, w + 2, &mut paths);
        assert_eq!(n, 5, "chapter six carries the word's five blocks");
        frames += n;
    }
    // **Chapter six-b's word, on run175** (item 651): the first capture
    // with an Airbase in it, and the first with an aircraft under an
    // attack order — each walks at the enemy Airbase — whose `BUILDS`
    // and order records no other window here carries. The window is the
    // word's block with two on either side, as six's. Item 680 closed it
    // at 1250, the capture's end, so the window stays on 632, the block
    // it was widened on, as six's stays on 700.
    if let Some(p) = &ch6b {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_SIX_B;
        let w = 632;
        let n = drive_capture(p, w - 2, w + 2, &mut paths);
        assert_eq!(n, 5, "chapter six-b carries the word's five blocks");
        frames += n;
    }
    // **Chapter nine's word, on run180** (item 676): the first capture of
    // a player's order — a `GroupMoveOrder` under a player's command, the
    // `COMMANDMANAGER` text the turn pump prints, and `play` set on each
    // commanded unit — whose records no other window here carries. The
    // window is the word's block with two on either side, as six's. The
    // same item closed it at 1100, so the window stays on 693, the block
    // it was widened on, as six's stays on 700.
    if let Some(p) = &ch9 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_NINE;
        let w = 693;
        let n = drive_capture(p, w - 2, w + 2, &mut paths);
        assert_eq!(n, 5, "chapter nine carries the word's five blocks");
        frames += n;
    }
    // **Chapter ten's word, on run184** (item 693): the first capture of a
    // patrol — `GroupPatrolOrder`'s `PATROLORDER` arrays and `waypoint`,
    // the leader's `ATTACKTOORDER` and `GroupAttackToOrder` legs, and the
    // `process_patrol` text the turn pump prints — whose records no other
    // window here carries. The word's block with two on either side, and
    // the squad's first patrol block, 642, with its neighbours.
    if let Some(p) = &ch10 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TEN;
        for w in [640, 642] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter ten carries the word's five blocks");
            frames += n;
        }
    }
    // **Chapter eleven's word, on run190** (item 696): the first capture
    // of a player's guard — a `GUARDORDER` a command gave, the
    // `process_guard` text the turn pump prints, and a wagon walking under
    // `MOVEORDER` with its escort re-posting beside it. The word's block
    // with two on either side; 724, the first pin's, which holds 722, the
    // block its value first parted on (the wagon's push); and 622, the
    // guard's first block, with the `process_guard` text before it.
    // Item 703 moved the word 734 → 1036, the guard's attack on who=1's
    // chariot ending; 734 stays, the block it was widened on. Item 707
    // moved it 1036 → 1133, `1/6`'s own attack ending; 1036 stays. Item
    // 709 moved it 1133 → 1139, the resync; 1133 stays. Item 713 closed
    // it at 1250, the trace's end; 1139 stays, the last word's block.
    if let Some(p) = &ch11 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_ELEVEN;
        for w in [622, 724, 734, 1036, 1133, 1139] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter eleven carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twelve's word, on run204** (item 714): the first
    // `FOLLOWORDER` on disk and the `process_follow` text before it. The
    // word's block with two on either side; 622, the chariot's first
    // follow block; 642, the squad's; and 711, where the value first
    // parted, the chariot's first leg.
    if let Some(p) = &ch12 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWELVE;
        for w in [622, 642, 711, 717] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twelve carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirteen's word, on run208** (item 718): the first
    // player's `GARRISONORDER` on disk, the `process_garrison` and
    // `process_eject_all` text, and a garrisoned Barracks at `BUILDS=7`.
    // 622, the chariot's first garrison block; 640, the word; 642, the
    // squad's; 699 and 761, each squad's door; 902, the eject's first
    // block.
    if let Some(p) = &ch13 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTEEN;
        for w in [622, 640, 642, 699, 761, 902] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter fourteen's word, on run210** (item 723): the first
    // player's formation command on disk, its `process_form` text, and a
    // walking group halted and replayed. 622, the standing re-form's
    // first block; 631, the word; 702, the move; 742, the replay in Line;
    // 905, the leader's arrival.
    if let Some(p) = &ch14 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FOURTEEN;
        for w in [622, 631, 702, 742, 905] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter fourteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter fifteen's word, on run215** (item 731): the first
    // player's attack command on disk, its `process_attack` text, a
    // player's attack-move, and the first golden pool, at `GUYS=4`. 622,
    // the right-click's first block; 736, the six `ATTACKORDER`s; 753,
    // the word; 809, the target's death; 862, the six
    // `GROUPATTACKTOORDER`s; 1060, their ungroup.
    if let Some(p) = &ch15 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FIFTEEN;
        for w in [622, 736, 753, 809, 862, 1060] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter fifteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter sixteen's word, on run219** (item 738): the first
    // player's explore and flee commands on disk (`process_move_to`'s
    // `orders` 3 and 4) and the first goody-box legs a player's explorer
    // takes. 622 and 642, the explores; 685, the chariot's box leg; 730,
    // its box opened; 804, the squad's box leg; 838, the word; 902 and
    // 1002, the flees.
    if let Some(p) = &ch16 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_SIXTEEN;
        for w in [622, 642, 685, 730, 804, 838, 902, 1002] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter sixteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter seventeen's word, on run223** (item 746): the first
    // player's aircraft on disk under an air order, and the first
    // `STRAFEORDER`, `AIRORDER` and player's `AIRPATROLORDER` records in
    // any capture. 622, the strike from the ground that gives no order;
    // 642, the word and the Fighter's strafe home; 662, the pair's; 666,
    // the strike turned patrol; 722, the Fighter inside its base; 778, the
    // patrol's strafe on the Barracks; 822, its first damage; 1081, the
    // Barracks gone; 1212, the first empty tank. **Item 759 moved the word
    // to 805**, the first bomb's animation, and its window joins. **Item
    // 763 moved it to 821**, the bomb's damage, inside 822's window.
    // **Item 770 closed it at 1400**, run223's last block, whose window
    // joins at 1397, the last whole five blocks.
    if let Some(p) = &ch17 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_SEVENTEEN;
        for w in [622, 642, 662, 666, 722, 778, 805, 822, 1081, 1212, 1397] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter seventeen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter eighteen's word, on run241** (item 779): the first
    // player's `BUILDORDER` on disk and the first human builders. 622,
    // `0/6`'s orders and the Barracks placed and paid; 642, the word and
    // the three's orders on the Siege Factory; 721, the Barracks' first
    // frame of construction; 948, the Factory finished; 1097, `0/8`'s
    // help; 1141, the Barracks finished.
    if let Some(p) = &ch18 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_EIGHTEEN;
        for w in [622, 642, 721, 948, 1097, 1141] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter eighteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter nineteen's word, on run245** (item 790): the first
    // player's targeted `CASTORDER` on disk. 622, the Spy's cast and move;
    // 669, the word; 756, the first frame in range; 795, the cast and the
    // Barracks `infiltrated`.
    if let Some(p) = &ch19 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_NINETEEN;
        for w in [622, 669, 756, 795] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter nineteen carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty's word, on run249** (item 803): the first player's
    // transport toggle and Transport casts on disk. 622, `0/7`'s bit off;
    // 703, `0/6`'s cast; 704, barge `0/8`; 719, `0/7` stopped at the shore;
    // 802, its bit on; 830, barge `0/9`; 1160, the disembark.
    if let Some(p) = &ch20 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY;
        for w in [622, 703, 704, 719, 802, 830, 1160] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-one's word, on run255** (item 813): the first
    // player's repair on disk. 650, the first arrow into the Barracks;
    // 782, `0/6`'s swarm; 802, the trio's; 930, the repair; 968, a late
    // order dying; 1131, `0/7`'s gather approach; 1141, the first word
    // (closed at 1300 by item 824, a human's found gather dropping its group).
    if let Some(p) = &ch21 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_ONE;
        for w in [650, 782, 802, 930, 968, 1131, 1141] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-one carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-two's word, on run265** (item 836): the first
    // plane launched out of a base on disk. 768, the strike laid inside;
    // 778, the word and the launch; 924, the Fighter's first round; 1178,
    // its empty tank; 1385, its second landing. Item 842: 798, the climb's
    // first parting, and 1497, the closed word's last blocks.
    if let Some(p) = &ch22 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_TWO;
        for w in [768, 778, 798, 924, 1178, 1385, 1497] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-two carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-three's word, on run281** (item 867): the first
    // Airbase on disk whose repeat bit goes. 1442, the toggle and the
    // building group's pool slot; 1489 and 1513, the landings off the
    // non-repeating base; 1585, `0/6`'s kill at its full tank; 1837, the
    // closed word's last blocks.
    if let Some(p) = &ch23 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_THREE;
        for w in [1442, 1489, 1513, 1585, 1837] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(
                n, 5,
                "chapter twenty-three carries the window's five blocks"
            );
            frames += n;
        }
    }
    // **Chapter twenty-four's word, on run285** (item 877): the first
    // production queue a player fills on disk. 622 and 642, the two
    // queue-ups and the building group's pool slot; 855, the word, the
    // Hoplites' finish and training. Closed at 1560: 902, the toggle;
    // 1060, the re-queue; 1272, its refusal; 1302, the second press;
    // 1557, the closed word's last blocks.
    if let Some(p) = &ch24 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_FOUR;
        for w in [622, 642, 855, 902, 1060, 1272, 1302, 1557] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-four carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-five's word, on run292** (item 884): the first
    // cancels on disk. 702, arm a's refund and `queued`; 842, arm c's
    // clear of the bit; 855, the first walk's word, the uncancelled
    // Hoplites' finish. Closed at 1466: 762, arm b; 965, the Bowmen's
    // finish with the bit gone; 982, the pool's `get_num_cap`; 1002, arm
    // d; 1216, the last Hoplites; 1463, the closed word's last blocks.
    if let Some(p) = &ch25 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_FIVE;
        for w in [702, 762, 842, 855, 965, 982, 1002, 1216, 1463] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-five carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-six's word, on run296** (item 883): the first
    // research through the player's command. 622, arm a's entry and price;
    // 652, arm c's, behind the busy Library; 822, The Art of War's finish
    // and its counters; 852, the held press; 1023, Written Word's finish
    // and the re-price; 1042, the cancel; 1062, Barter again; 1106, the
    // Hoplites; 1243, Barter's finish and the commerce cap; 1489, the last
    // blocks.
    if let Some(p) = &ch26 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_SIX;
        for w in [622, 652, 822, 852, 1023, 1042, 1062, 1106, 1243, 1489] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-six carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter twenty-seven's word, on run300** (item 901): the first unit
    // upgrade through the player's command. 622, arm a's research entry
    // and price; 642, the gate; 652, the Slingers behind it; 922, the
    // finish that converts the squad and re-targets the entry; 1002 and
    // 1012, the presses after it; 1102, the first walk's word. Closed at
    // 1560: 1111 and 1307, the two Javelineers squads; 1557, the closed
    // word's last blocks.
    if let Some(p) = &ch27 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_SEVEN;
        for w in [622, 642, 652, 922, 1002, 1012, 1102, 1111, 1307, 1557] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(
                n, 5,
                "chapter twenty-seven carries the window's five blocks"
            );
            frames += n;
        }
    }
    // **Chapter twenty-eight's word, on run304** (item 888): two
    // buildings under one command. 622, the single press; 642, the sort
    // and passes and the building group of two; 702 and 722, the
    // infinite toggle on one and on two; 742 and 762, the group in the
    // other order and again; 825 and 876, the births into the building
    // groups' slots.
    if let Some(p) = &ch28 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_EIGHT;
        for w in [622, 642, 702, 722, 742, 762, 825, 876] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(
                n, 5,
                "chapter twenty-eight carries the window's five blocks"
            );
            frames += n;
        }
    }
    // **Chapter twenty-nine's word, on run308** (item 915): the repeat
    // launch. 1542, the press; 1585, 1789 and 1813, the three kept patrols
    // relaunched under the bit; 1746, the Biplane trained into the
    // Airbase; 1828, the counter with the Biplane alone inside; 1985, the
    // relaunched tank flown out; 2067, the last blocks.
    if let Some(p) = &ch29 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_TWENTY_NINE;
        for w in [1542, 1585, 1746, 1789, 1813, 1828, 1985, 2067] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter twenty-nine carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty's word, on run312** (item 928): the gather point.
    // 618, 652 and 702, the first lists; 760, the Citizen out under the
    // City's point; 822, the word; 856, the first Hoplites; 902, the point
    // moved; 953, 2008's Hoplites kept in; 1060 and 1079, the Bowmen out
    // and garrisoned; 1102, the Clear; 1447, the last blocks.
    if let Some(p) = &ch30 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY;
        for w in [
            618, 652, 702, 760, 822, 856, 902, 953, 1060, 1079, 1102, 1447,
        ] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty-one's word, on run338** (item 955): the gather
    // point's other arms. 704, the list of two; 718, the first Citizen
    // re-seated under its ground point; 740, the word; 792, the Lookout
    // site; 824 and 938, the gather and build arms; 858, the third
    // re-seat; 953, the two points' two orders; 1397, the last blocks.
    if let Some(p) = &ch31 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_ONE;
        for w in [704, 718, 740, 792, 824, 858, 938, 953, 1397] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-one carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty-two's word, on run344** (item 947): an Airbase's
    // gather point. 1602, the planes re-ordered; 1702, the list of two;
    // 1741, the waypoint's step; 1746 and 1747, the Biplane born on the
    // patrol and out; 1751, the word; 1852, the Clear's strafes home; 2034
    // and 2106, the first and last landings; 2357, the last blocks.
    if let Some(p) = &ch32 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_TWO;
        for w in [1602, 1702, 1741, 1746, 1747, 1751, 1852, 2034, 2106, 2357] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-two carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty-three's word, on run358** (item 976): an Airbase's
    // launch issuers. 2202, the enemy Barracks; 2262, the launch strike;
    // 2282, the plain launch patrol; 2287, the word; 2297, the shift-click;
    // 2307, action 3's strikes; 2337, the append's patrols; 2409 and 2489,
    // the first and last over P3; 2737, the last blocks.
    if let Some(p) = &ch33 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_THREE;
        for w in [2202, 2262, 2282, 2287, 2297, 2307, 2337, 2409, 2489, 2737] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(
                n, 5,
                "chapter thirty-three carries the window's five blocks"
            );
            frames += n;
        }
    }
    // **Chapter thirty-four's word, on run362** (item 1009): the launch
    // commands' other arms. 2211, the second Airbase; 2262, ctrl's patrol;
    // 2277, alt's strike; 2307, the flight to `0/2008`; 2314, ctrl's
    // strike refused; 2322, the ground point and the word; 2337, A3;
    // 2352, P2; 2402, the Clear; 2474, the landing at `0/2008`; 2602,
    // alt at `0/2008`; 2847, the last blocks.
    if let Some(p) = &ch34 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_FOUR;
        for w in [
            2211, 2262, 2277, 2307, 2314, 2322, 2337, 2352, 2402, 2474, 2602, 2847,
        ] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-four carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty-five's word, on run371** (item 1019): the
    // Helicopter's and missiles' launch arms. 2221, the silo; 2227, its
    // queue of one; 2431, the V2's birth; 2446, the first Helicopter out
    // at its birth and the word; 2502, `0/2008`'s point; 2659, the second
    // Helicopter under it; 2672, the silo's strike; 2702, the V2's round;
    // 2740, the Helicopter at its point; 2802, the Clear; 3012, the
    // patrol at `0/2008`; 3257, the last blocks.
    if let Some(p) = &ch35 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_FIVE;
        for w in [
            2221, 2227, 2431, 2446, 2502, 2659, 2672, 2702, 2740, 2802, 3012, 3257,
        ] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-five carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter thirty-six's word, on run390** (item 1078): the missile's
    // other arms. 2722, the first new V2's price; 2926, V2b's birth; 3021,
    // its strike; 3031, the re-press; 3051, the launch and its round;
    // 3082, the shield at the order; 3112, the word's next block; 3170,
    // the shield at the blast; 3417, the last blocks.
    if let Some(p) = &ch36 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_SIX;
        for w in [2722, 2926, 3021, 3031, 3051, 3082, 3112, 3170, 3417] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-six carries the window's five blocks");
            frames += n;
        }
        // **Chapter thirty-seven's word, on run397** (item 1091): the nuke.
        // 617, the research's price; 2237, its end; 3022, the nuke's birth;
        // 3052, its strike; 3082, the launch and its round; 3201, the landing;
        // 3218, 3227 and 3240, the ring's three strikes; 3487, the last blocks.
        if let Some(p) = &ch37 {
            let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_SEVEN;
            for w in [617, 2237, 3022, 3052, 3082, 3201, 3218, 3227, 3240, 3487] {
                let n = drive_capture(p, w - 2, w + 2, &mut paths);
                assert_eq!(
                    n, 5,
                    "chapter thirty-seven carries the window's five blocks"
                );
                frames += n;
            }
        }
    }
    // **Chapter thirty-eight's word, on run404** (item 1102): the air line
    // under fire. 743, the Battery's jam roll; 754, its first round (a
    // miss); 777, the word's next block; 783, the first low round; 1007
    // and 1020, the two crashes; 1759, the last blocks.
    if let Some(p) = &ch38 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_EIGHT;
        for w in [743, 754, 777, 783, 1007, 1020, 1759] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(
                n, 5,
                "chapter thirty-eight carries the window's five blocks"
            );
            frames += n;
        }
    }
    // **Chapter thirty-nine's word, on run422** (item 1111): the spell
    // issuer's untargeted crafts. 626 and 646, To Arms; 666 and 706,
    // Civilian; 722, the General's order; 821, the decoys; 1097, the last
    // blocks.
    if let Some(p) = &ch39 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_THIRTY_NINE;
        for w in [626, 646, 666, 706, 722, 821, 1097] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter thirty-nine carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty's word, on run430** (item 1167): the casts' other
    // arms. 608 and 614, the two refused presses; 632, 642 and 652, the
    // Militia's Civilians ahead of a repair, a build and a gather; 806 and
    // 846, D's wounded conversions; 882 and 902, the bell and the
    // all-clear (`CITIES=5`: the City's `city_flags`); 1075, the Barracks
    // finished; 1147, the last blocks.
    if let Some(p) = &ch40 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY;
        for w in [608, 614, 632, 642, 652, 806, 846, 882, 902, 1075, 1147] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-one's word, on run437** (item 1182): the computer's
    // sortie and the build-site spill. 622, the Biplane's flight home
    // through `be 1`; 742 and 779, its landing and the sortie; 827, the
    // patrol's strafe; 840, item 1182's word's release; 859, the first
    // strafe's hits (item 1200); 970, the second pass's; 1080 and 1130,
    // the Citizen's repair and its first spill; 1109, the dry tank's
    // kill; 1461 and 1530, the trireme's short flight and the strafer's
    // round that does not roll; 1767, the last blocks.
    if let Some(p) = &ch41 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_ONE;
        for w in [
            622, 742, 779, 827, 840, 859, 970, 1080, 1109, 1130, 1461, 1530, 1767,
        ] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-one carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-two, on run460** (item 1209): `resolve_block`'s
    // arms. 622, the Knight's attack on the ring's corner; 812, the
    // peace arm's `agendas`; 819, the peace walk's held waypoint; 907,
    // the own-side Knight's move; 1157, the last blocks.
    if let Some(p) = &ch42 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_TWO;
        for w in [622, 812, 819, 907, 1157] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-two carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-three, on run466** (item 1223): 1271, the barge
    // put ashore on its phase frame; 1357, the squad's boarding (item
    // 1235); 1553, who=1's gather with its dead gatherer gone, and 1902,
    // the refused landing, the squad aboard (item 1248).
    if let Some(p) = &ch43 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_THREE;
        for w in [1271, 1357, 1553, 1902] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-three carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-four, on run484** (item 1254): 611, the computer's
    // Bomb Vessel's attack across the regions; 631, its first strike's
    // turret; 671, the word's block, the first round's landing.
    if let Some(p) = &ch44 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_FOUR;
        for w in [611, 631, 671] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-four carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-five, on run492** (item 1268): 603, the Nubian
    // Merchant's birth; 769, its Dye's first pay; 804, whose five blocks
    // hold the three research prices (802, 804, 806).
    if let Some(p) = &ch45 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_FIVE;
        for w in [603, 769, 804] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-five carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-six, on run496** (item 1278): 726, the trained
    // Citizen out under its two-point list; 1050, `all_gathering`'s prune;
    // 1902, the word's block, the squad's landing.
    if let Some(p) = &ch46 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_SIX;
        for w in [726, 1050, 1902] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-six carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-seven, on run514** (item 1310): 677, the Keep gained
    // on a standing Tower site; 1102, the squad's landing; 1111, the age's
    // snap; 839, the word's block (item 1323); 905, the word's block
    // (item 1330), kept since the chapter closed (item 1350).
    if let Some(p) = &ch47 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_SEVEN;
        for w in [677, 839, 905, 1102, 1111] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-seven carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-eight, on run551** (item 1358): 683, the site's
    // first hit on who=0's land; 935, the squad's first blow on it (the
    // chapter's first parting); 983, the blow whose sixteenths carry; 1094,
    // the first bomb; 1451, the Tower's first shot (item 1375's
    // walk-back); 1482, the word's block (item 1375).
    if let Some(p) = &ch48 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_EIGHT;
        for w in [683, 935, 983, 1094, 1451, 1482] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-eight carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter forty-nine, on run576** (item 1393): 912, the Scout's
    // fourth wound (the word's value diff: ours flees, theirs stands);
    // 934, the Tower's fifth shot, the original's alone.
    if let Some(p) = &ch49 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FORTY_NINE;
        for w in [912, 934] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter forty-nine carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter fifty, on run577** (item 1404): 686, the Stockade's first
    // shot, the original's alone.
    if let Some(p) = &ch50 {
        let _ = super::testkit::GOLDEN_WORD_CHAPTER_FIFTY;
        let n = drive_capture(p, 684, 688, &mut paths);
        assert_eq!(n, 5, "chapter fifty carries the window's five blocks");
        frames += n;
        // Item 1419: 700, the Hoplite captain's `ATTACK` on the Stockade
        // under `Group::target_opportunity` (the word 819, then closed).
        for w in [700, 819] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter fifty carries the window's five blocks");
            frames += n;
        }
    }
    // **run235** (item 770): run223's game again at `AMMO=5`, the first
    // capture on disk with a Bomber's round in it. 806, `0/8`'s first bomb
    // a frame out; 852, `0/7`'s.
    if let Some(p) = &ch17a {
        for w in [806, 852] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "run235 carries the window's five blocks");
            frames += n;
        }
    }
    // **Chapter eight's word, on run171** (item 660): the first capture
    // with a General and a Spy in it, and the first golden capture at
    // `LEADERS=5`, whose leader record prints the diplomacy row. The
    // window is the word's block with two on either side, as six's.
    // Item 664 moved the word 617 → 659, and both windows are driven:
    // 617's carries the Spy's explore order, the first on this lobby.
    // Item 668 closed it at 900, the capture's end, so the windows stay
    // on 617 and 659, the blocks it was widened on, as six's stays on 700.
    let _ = super::testkit::GOLDEN_WORD_CHAPTER_EIGHT;
    if let Some(p) = &ch8 {
        for w in [617, 659] {
            let n = drive_capture(p, w - 2, w + 2, &mut paths);
            assert_eq!(n, 5, "chapter eight carries the word's five blocks");
            frames += n;
        }
    }
    // Third pair's own word blocks, on each map's own new capture.
    for (path, word) in [
        (&r602, 986),
        (&r603, 7356),
        (&r610, 8182),
        (&r612, 8236),
        (&r616, 8385),
        (&r617, 8840),
        (&r622, 9655),
        (&r623, 9777),
        (&r624, 10131),
        (&r629, 10802),
        (&r631, 11582),
        (&r634, 12794),
        (&r635, 12952),
        (&r636, 14090),
        // Item 1481: the word left run639 for 15344; its block stays.
        (&r639, 14786),
        // Item 1487: the word left run642 for 16857; its block stays.
        (&r642, 15344),
        // Item 1500: the word left run655 for 17171; its block stays.
        (&r655, 16857),
        // Item 1508: the word left run657 for 17244; its block stays.
        (&r657, 17171),
        // Item 1514: the word left run661 for 17318; its block stays.
        (&r661, 17244),
        // Item 1519 closed the map at its end, 17379; run662's word block
        // stays.
        (&r662, 17318),
        (&r601, 2576),
    ] {
        if let Some(path) = path {
            let n = drive_capture(path, word - 1, word + 3, &mut paths);
            assert_eq!(n, 5, "the French pair carries its word window");
            frames += n;
        }
    }
    // Item 1519: French East Indies' last running blocks and closing state.
    if let Some(path) = crate::testenv::dump("gamelog-run667-islands-french-closing-window.txt") {
        let n = drive_capture(&path, 17374, 17380, &mut paths);
        assert_eq!(
            n, 7,
            "six French East Indies running blocks and closing state"
        );
        frames += n;
    }
    if let Some(path) =
        crate::testenv::dump("gamelog-run609-lakes-french-toughest-closing-window.txt")
    {
        let n = drive_capture(&path, 5633, 5639, &mut paths);
        assert_eq!(n, 7, "six French Lakes running blocks and closing state");
        frames += n;
    }
    // **The coverage pair's word blocks** (621; parked 1513, item 1511):
    // run656's frame-8 word (block 9), run660's frame-177 word (block 178),
    // run669's frame-185 word (block 186), and run672's frame-583 word
    // (block 584), frame-667 word (block 668, item 1532) and frame-727
    // word (block 728, item 1539), and run678's frame-982 word (block 983,
    // item 1544) and frame-1183 word (block 1184, item 1546), and run679's
    // frame-1277 word (block 1278, item 1552) and frame-1408 word (block
    // 1409, item 1558), each with two either side; the third map's lobby's
    // run683 frame-1582 word (block 1583, item 1561) and frame-1818 word
    // (block 1819, item 1565) and frame-1830 word (block 1831, item 1581).
    for (name, block) in [
        (super::coverage_pair::RUN656, 9),
        (super::coverage_pair::RUN660, 178),
        (super::coverage_pair::RUN669, 186),
        (super::coverage_pair::RUN672, 584),
        (super::coverage_pair::RUN672, 668),
        (super::coverage_pair::RUN672, 728),
        (super::coverage_pair::RUN678, 983),
        (super::coverage_pair::RUN678, 1184),
        (super::coverage_pair::RUN679, 1278),
        (super::coverage_pair::RUN679, 1409),
        (super::sahara_coverage::RUN677, 13),
        (super::sahara_coverage::RUN680, 721),
        (super::sahara_coverage::RUN681, 1198),
        (super::sahara_coverage::RUN681, 1251),
        (super::sahara_coverage::RUN683, 1583),
        (super::sahara_coverage::RUN683, 1819),
        (super::sahara_coverage::RUN683, 1831),
    ] {
        if let Some(path) = crate::testenv::dump(name) {
            let n = drive_capture(&path, block - 2, block + 2, &mut paths);
            assert_eq!(n, 5, "the coverage pair carries its word window");
            frames += n;
        }
    }
    assert!(frames > 0, "no frame of either window was found");
    let _ = GOLDEN_WORD_CHAPTER_TWO;
    let actual = unread(&paths);
    let pinned: BTreeMap<String, BTreeSet<String>> = UNREAD
        .iter()
        .map(|(p, keys)| {
            (
                p.to_string(),
                keys.split_whitespace().map(str::to_string).collect(),
            )
        })
        .collect();
    let mut failures = Vec::new();
    for (p, keys) in &actual {
        let empty = BTreeSet::new();
        let was = pinned.get(p).unwrap_or(&empty);
        let new: Vec<&String> = keys.difference(was).collect();
        if !new.is_empty() {
            let whole = paths
                .get(p)
                .is_some_and(|(printed, _)| printed.len() == keys.len());
            failures.push(format!(
                "`{p}` prints {} and nothing reads {} — read {}, or pin {} in UNREAD with the item that owes it{}",
                keys.len(),
                new.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(" "),
                if new.len() == 1 { "it" } else { "them" },
                if new.len() == 1 { "it" } else { "them" },
                if whole { " (nothing on this path is read at all)" } else { "" },
            ));
        }
    }
    for (p, was) in &pinned {
        let empty = BTreeSet::new();
        let now = actual.get(p).unwrap_or(&empty);
        let gone: Vec<&String> = was.difference(now).collect();
        if !gone.is_empty() {
            failures.push(format!(
                "`{p}`: {} {} read now; delete {} from UNREAD",
                gone.iter()
                    .map(|k| format!("`{k}`"))
                    .collect::<Vec<_>>()
                    .join(" "),
                if gone.len() == 1 { "is" } else { "are" },
                if gone.len() == 1 { "it" } else { "them" },
            ));
        }
        if !paths.contains_key(p) {
            failures.push(format!(
                "`{p}` is pinned and no frame of either window prints it; delete the row"
            ));
        }
    }
    if !failures.is_empty() {
        eprintln!("the pin as measured over {frames} frames, in source form:");
        for (p, keys) in &actual {
            eprintln!(
                "    ({:?}, {:?}),",
                p,
                keys.iter().cloned().collect::<Vec<_>>().join(" ")
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---------------------------------------------------------------------
// **The compared pin** — parked 527, built in the fourteenth Fable pass
// (2026-09-25). The recorder is `crate::diff::compared`; this is the pin
// that holds the parser's own records against what the shared instrument
// registered on the Great Lakes word's own window.

/// A parsed record the per-frame instrument never touches, with who does:
/// the start block's, the world's, and the parser's own containers. A
/// registration arriving for one of these fails the pin: the row is stale.
const NOT_THE_INSTRUMENT_S: &[(&str, &str)] = &[
    ("Log", "the parser's index"),
    ("Pos", "a component, compared through its record's `pos`"),
    ("Frame", "the per-frame container"),
    (
        "FrameUnit",
        "the checkpoint's own row, `harness::checkpoint`",
    ),
    ("Initial", "the start block, `diff::setup`"),
    ("ConstantDump", "the start block's constants, `diff::setup`"),
    ("Checksum", "the start block's checksums, `diff::setup`"),
    ("CellDump", "the world, `diff::world`"),
    ("RegionDump", "the world, `build_sim`"),
    ("GoodDump", "the world, `build_sim`"),
    ("HerdDump", "the world, `build_sim`"),
    ("FarmDump", "the world, `diff::world`"),
    ("MountainDump", "the world, `build_sim`"),
    (
        "MakeObjectDump",
        "the leader's make list, `diff::setup` and `diff::leader`",
    ),
    // The leader record is compared on every AI widening — 12,660 rows on
    // this window — by `diff::leader::rows` against `leader::theirs`,
    // which reads the block's keys directly and never this struct.
    (
        "LeaderDump",
        "the leader block, read by `diff::leader` off the block",
    ),
];

/// `(record, fields)` — every field of a record the instrument reads on
/// the window that it did **not** compare, on the day of the pin,
/// space-separated and sorted. **Exact**: a field compared since must be
/// deleted here, and a field that arrives uncompared must be compared or
/// added here with the item that owes it.
const UNCOMPARED_BY_THE_INSTRUMENT: &[(&str, &str)] = &[
    // **Re-pinned 2026-09-28, item 1106** (DECISIONS 54 §2), on the second
    // pair's East Indies word's window — the word's block and two either
    // side on run357, walked from run346's start with the group record and
    // the attack order's row (`second::widen_records`): 5605..5609 when the
    // item moved the walk here, 5772..5776 since it moved the word to 5773,
    // and run414's 5974..5978 since item 1115 moved it to 5975 (the pin
    // unchanged), and run414's 6150..6154 since item 1120 moved it to 6151,
    // and run419's 6320..6324 since item 1127 moved it to 6321, and run420's
    // 6608..6612 since item 1143 moved it to 6609, and run420's 6742..6746
    // since item 1156 moved it to 6743, and run425's 7381..7385 since item
    // 1164 moved it to 7382, and run425's 7511..7515 since item 1174 moved
    // it to 7512, and run439's 8518..8522 since item 1185 moved it to 8519.
    // Item 1061 first pinned it on
    // Great Lakes' word, run373's 4977..4981, and it followed that word to
    // run403's 5927..5930 until item 1099 closed the map at its end. **A
    // site gated on the window's content registers only when it runs**, so
    // a row here is one of two things and the comment says which: a field
    // no site compares, or a site the window never reached. On both
    // windows the dump holds no death and no `ATTACKORDER`; its order
    // blocks are `UNITORDER`, `MOVEORDER`, `TARGETORDER`, `GATHERORDER`,
    // `BUILDORDER`, `EXPLORETOORDER` and `TRADEORDER` only. 5772..5776
    // holds an army's group (slot 72), which 5605..5609 did not. The second
    // kind leaves this pin the day a window that reaches it is driven here.
    //
    // `UnitDump`: `uid` is the identity behind `o` (ledger); `flags`,
    // `infiltrated` and the container pair `up`/`up_who`/`down`/
    // `down_who` no site compares. `damage_o`/`damage_who` are compared
    // only where both sides hold a live wound window, and no unit is
    // wounded on these blocks (Great Lakes' war window compared them).
    ("UnitDump", "down down_who flags infiltrated uid up up_who"),
    // `Guy`: `ox`/`whom`, the figure's aim, are compared since item 1061;
    // `last_pos` is run86's widening's (item 271); `kind`, `guy_num` and
    // `guy_flags` no site compares. **The turret's four** — `node_flags`,
    // `des_node_flags`, `turret_angles`, `des_turret_angles` — are
    // compared by the golden widening since item 1117
    // (`golden::widen_turrets`), on every figure of every golden window;
    // the second pair's walk, whose window this is, does not compare them.
    (
        "Guy",
        "des_node_flags des_turret_angles guy_flags guy_num kind last_pos node_flags \
         turret_angles",
    ),
    // `OrderDump`: the move row leaves `tolerance`, `retry`, `attempts`
    // and `orig_x`/`orig_y` to run29's field-table test on purpose
    // (`compare_orders`); the `GROUPORDER`, patrol, guard, garrison, cast,
    // air, strafe and ground rows on their orders, none of which stands on
    // this window (the guard's six were compared on run202's, where one
    // stood); `uid`, `metric`, `build_type` and `non_flat_gather` no site
    // compares. **The attack order's own row** — `mandatory`, `defensive`,
    // `in_range`, `ever_in_range`, `new_ord`, `def_x`, `def_y` — is
    // compared since item 1061 on every slot both sides hold an attack,
    // and no attack stands on these blocks. `coll_x`/`coll_y` are compared
    // on a move once the original's pair has left its `(0, 0)` start: no
    // move's has on East Indies' blocks, and one had on the third map's
    // run382 window, so the pair left this pin when the nineteenth pass
    // walked that window too. Item 1133 moved the third map's window to
    // run416's 12782..12786, where none has; East Indies' word at 6321
    // (item 1127) has one, so the pair stays off.
    // **Item 1163 moved the third map's window to run418's
    // 14586..14590**, where the AI's army stands in a group with an
    // attack order on its members: the attack row, `whose`/`oxx`,
    // `form_id` and the group row (`group_id`, `group_angle`,
    // `in_group`) are compared there, and left this pin. **Item 1171 moved
    // it to run426's 15585..15589**, where a cast order stands:
    // `cast_paid` and `cast_spell` are compared there, and left it.
    // **Item 1194 moved it to run449's 17622..17626**, where neither an
    // attack nor a cast stands and a guard order does: the attack row and
    // the cast pair return to this pin, and the guard's six leave it.
    // **Item 1206 closed the third map at its trace's end**, so only East
    // Indies' run439 window is walked: it holds no group move, no guard
    // and no move whose `coll` pair has left `(0, 0)`, so the group row
    // (`group_id`, `group_angle`, `in_group`, `form_id`, `oxx`, `whose`),
    // the guard's six and `coll_x`/`coll_y` return to this pin. **Item
    // 1197 moved East Indies' window to run445's 8907**, where `1/35`'s
    // move has left `(0, 0)`: `coll_x`/`coll_y` are compared there, and
    // left this pin again at 1206's booking (the two landed apart).
    // **Item 1214 moved it to run462's 10183**, where a cast order stands
    // and no move's `coll` pair has left `(0, 0)`: `cast_paid` and
    // `cast_spell` leave this pin, and `coll_x`/`coll_y` return to it.
    // **Item 1228 moved it to 10185**, blocks 10184..10187: the Caravan
    // `1/40`'s cast was spent on 10183, so no cast stands there and
    // `cast_paid` and `cast_spell` return to this pin.
    // **Item 1221 walks the third map at Toughest beside it**, run471's
    // 5375..5378, where a move's `coll` pair has left `(0, 0)`:
    // `coll_x`/`coll_y` are compared on the union, and leave this pin.
    // **Item 1241 moved the Toughest window to run476's 5781..5784**,
    // where no move's `coll` pair has left `(0, 0)`, and
    // East Indies' 10185 window has none either (measured on the tree
    // merged with 1228's): `coll_x`/`coll_y` return to it.
    // **Item 1243 moved East Indies' window to run480's 10984..10987**,
    // where a cast order, a group move and a move whose `coll` pair has
    // left `(0, 0)` stand: `cast_paid`, `cast_spell`, the group row
    // (`group_id`, `group_angle`, `in_group`, `form_id`, `oxx`, `whose`)
    // and `coll_x`/`coll_y` are compared there, and leave this pin
    // (measured on the tree merged with 1241's and 1248's).
    // **Item 1260 moved the Toughest window to run488's 7784..7787**, where
    // a move's `coll` pair has left `(0, 0)`: `coll_x`/`coll_y` are compared
    // on the union, and stay off this pin.
    // **Item 1264 moved it to run490's 11327..11330**, where no cast order
    // stands: `cast_paid` and `cast_spell` return to this pin; the group
    // row and `coll_x`/`coll_y` are still compared there.
    // **Item 1281 moved it to run490's 11548..11551**, where a guard order
    // stands and no group move does: the six `guard_*` fields are compared
    // there and leave this pin; the group row (`form_id`, `group_angle`,
    // `group_id`, `in_group`, `oxx`, `whose`) is compared on item 1286's
    // Toughest window (8785..8788), and stays off it.
    // **Item 1293 moved the Toughest window to run500's 8855..8858**, where
    // the squad's group attack has been ungrouped since 8786 and no group
    // move stands; with 1297's East Indies window (11636) holding none
    // either, the group row returns to this pin (measured on the tree
    // merged with 1281's and 1297's).
    // **Item 1302 moved East Indies' window to run508's 12581..12584**,
    // walked beside 1305's Toughest window (run511's 9322..9325): the group
    // row (`form_id`, `group_angle`, `group_id`, `in_group`, `oxx`,
    // `whose`) is compared on the union, and leaves this pin (measured on
    // the tree merged with 1305's).
    // **Item 1326 moved East Indies' window to run523's 13384..13387**,
    // walked beside 1318's Toughest window: no group move stands on the
    // union, so the group row returns to this pin, and a cast does, so
    // `cast_paid` and `cast_spell` leave it (measured on the tree merged
    // with 1318's).
    // **Item 1341 moved East Indies' window to run535's 14140..14143**: no
    // cast stands on the union, so `cast_paid` and `cast_spell` return to
    // this pin (measured on the tree merged with 1338's).
    // **Item 1362 moved East Indies' window to run544's 15882..15885**:
    // army 0's group 71 stands moved on it, so the group row is compared
    // on the union and leaves this pin (measured on the tree merged with
    // 1354's).
    // **Item 1383 moved East Indies' window to run572's 16159..16162**: a
    // cast stands on it (`1/128` and `1/143`, the Galleon's boarders), so
    // `cast_paid` and `cast_spell` leave this pin (measured on the tree
    // after `ccc update`'s base, f0991ca8).
    // **Item 1418 moved East Indies' window to run579's 16481..16484**: no
    // group move stands on it, so the group row (`form_id`, `group_angle`,
    // `group_id`, `in_group`, `oxx`, `whose`) returns to this pin (measured
    // on the tree after `ccc update`'s base, 1765a40e).
    // Item 1427: run583's 16761..16765 carries group moves, so those
    // six fields are compared again and leave the pin.
    // Item 1434: run585's new word compares the attack-order fields;
    // no cast stands on the union, so cast_paid/cast_spell return.
    // Item 1435: the 17698 window compares casts and damage attribution.
    // Item 1436: no group move stands on the new 17785 window or the
    // Toughest union; the six group-move fields return to this pin.
    // Item 1437: run589 compares those group fields again, but carries
    // no paired cast on the word window: cast_paid/cast_spell return.
    // Item 1443: run603 has paired cast orders; both fields leave the pin.
    // Item 1439: run594 carries a paired cast; both fields leave again.
    // Item 1440: 18089 has no paired cast; both return to the pin.
    (
        "OrderDump",
        "ag_accuracy ag_att_x ag_att_y ag_attack_unit air_old air_oxx air_whose \
         attempts build_type cruising_alt \
         garrison_search \
         metric non_flat_gather \
         orig_x orig_y patrol_x patrol_y retry returning sharp_turn strafe_xx \
         strafe_yy tolerance uid waypoint ",
    ),
    // `BuildDump`: **`orig_type` no site compares** (parked 728, the
    // pin's first catch); `flags`, `max_age`, `mtn`, `cliff`,
    // `mining_size`, `construct_hits`, `ever_seen` and
    // `ever_seen_completed` no site compares. `queue` registers here: a
    // build is queued. `job_counter` is compared only on an unfinished
    // site (`!active && flags & 4 == 0`, item 1086), and one stood on
    // 5974..5978; since item 1120 none does on run414's 6150..6154. **One
    // stood on the third map's run382 window**, walked here since the
    // nineteenth pass, so `job_counter` was compared and off this pin; it had left and
    // returned with the word four times in one tranche (items 1061, 1086,
    // 1106, 1120). Item 1133 moved the third map's window to run416's
    // 12782..12786, where none is; one is on East Indies' 6321 window (item
    // 1127), so `job_counter` stays off. **Item 1185 moved East Indies'
    // window to run439's 8518..8522**, where no unfinished site stands;
    // one does on the third map's run428 window (item 1177), so
    // `job_counter` stays off. **Item 1206 closed the third map**, and
    // with only run439's window walked `job_counter` returns to this pin.
    // **Item 1221 opened the third map at Toughest**: its word's window on
    // run471, 5375..5378, holds an unfinished site, so `job_counter` is
    // compared and off this pin again. **Item 1281 moved East Indies' window
    // to run490's 11548..11551**, and with item 1275's Toughest window on
    // 8377..8380 neither held an unfinished site: `job_counter` returned.
    // Item 1286's Toughest window on 8785..8788 holds one, so on the tree
    // merged with it `job_counter` is compared and off this pin again.
    (
        "BuildDump",
        "cliff construct_hits ever_seen ever_seen_completed flags \
         max_age mining_size mtn orig_type",
    ),
    // `DEATH_OBJS`: the window holds no death; the rows register on one.
    ("DeathDump", "cur_anim first_frame gpiece o valid who"),
    // The group record is compared whole since item 1061
    // (`second::widen_records`), `think_frame` since item 1423.
    // `role` is compared on an army's group alone, and registers on
    // 5772..5776 (slot 72, ours 0 against 599056).
];

/// **Every field the parser carries is compared by the shared instrument
/// on every open word's own window, or pinned above.** The window is
/// `second::east_indies_closing_window`, retained after item 1441 closed
/// the second pair (formerly its word window since item 1106). The third map's,
/// `third::sahara_word_window`, was walked beside it from the nineteenth
/// pass until item 1206 closed that map at its end. The second pair's
/// East Indies word's block and two on either side, on its widening
/// (run414 since item 1115, at 6151 since item 1120; run419 at 6321 since
/// item 1127; run420 at 6609 since item 1143, and at 6743 since item
/// 1156; run425 at 7382 since item 1164, and at 7512 since item 1174;
/// run439 at 8519 since item 1185; run480 at 10985 since item 1243), and
/// since item 1221 the third map at
/// Toughest's, `sahara_toughest::great_sahara_toughest_15378_window` on
/// run471 walked from run470's start, and East Indies' from run346's —
/// walked with the
/// recorder on; a machine without the
/// captures says so. What this checks that the `UNREAD` pin cannot: that a key
/// which is *parsed* is also *compared*, per record, with both sides
/// present — the gap five landings in two tranches turned on.
#[test]
fn every_parsed_field_is_compared_by_the_instrument_or_pinned() {
    use super::compared;
    compared::start();
    let walked = super::second::east_indies_closing_window();
    // **Every open word's window** (parked 1067 and 1080, the nineteenth
    // pass; `floors::the_compared_pin_walks_every_open_ai_word` holds the
    // list to `AI_WORDS`): the third map's, run416's blocks around its word
    // at run414's detail since item 1133 (run382's blocks 1..259 before), is
    // walked under the same recorder, and a field is compared when a site on
    // either window compared it.
    // Item 1206 closed the third map at its trace's end, 24000, so its
    // row names its endpoint and no window; item 1221 opened the same map
    // at Toughest (DECISIONS 56 §1), whose word's blocks on run471 are
    // walked here beside East Indies'.
    // Item 1528 closed Toughest at its trace's end, 15432: run668's blocks
    // around the word 15378 stay walked, as French East Indies' do.
    let toughest = super::sahara_toughest::great_sahara_toughest_15378_window();
    // Item 1519 closed French East Indies at its trace's end, 17379: run662,
    // the word 17318's window, stays walked, and run667's last running
    // blocks and closing state are walked beside it.
    let french_east = super::third_pair::french_east_indies_17318_window();
    let french_closing = super::third_pair::french_east_indies_closing_window();
    let french_lakes = super::third_pair::french_great_lakes_closing_window();
    // Item 1445: retain run603's cast orders after moving the open window.
    // Their comparisons still run; a quiet successor must not erase coverage.
    let french_scholars = super::third_pair::french_east_indies_scholar_window();
    // Item 1446: and run610's, where the word stood before this one.
    let french_8182 = super::third_pair::french_east_indies_8182_window();
    // Item 1449: and run612's, where it stood before run616.
    let french_8236 = super::third_pair::french_east_indies_8236_window();
    // Item 1451: and run616's, where it stood before run617.
    let french_8385 = super::third_pair::french_east_indies_8385_window();
    // Item 1452: and run617's, where it stood before run622.
    let french_8840 = super::third_pair::french_east_indies_8840_window();
    // Item 1453: and run622's, where it stood before run623.
    let french_9655 = super::third_pair::french_east_indies_9655_window();
    // Item 1454: and run623's, where it stood before run624.
    let french_9777 = super::third_pair::french_east_indies_9777_window();
    // Item 1455: and run624's, where it stood before run629.
    let french_10131 = super::third_pair::french_east_indies_10131_window();
    // Item 1458: and run629's, where it stood before run631.
    let french_10802 = super::third_pair::french_east_indies_10802_window();
    // Item 1460: and run631's, where it stood before run634.
    let french_11582 = super::third_pair::french_east_indies_11582_window();
    // Item 1461: and run634's, where it stood before run635.
    let french_12794 = super::third_pair::french_east_indies_12794_window();
    // Item 1470: and run635's, where it stood before run636.
    let french_12952 = super::third_pair::french_east_indies_12952_window();
    // Item 1476: and run636's, where it stood before run639.
    let french_14090 = super::third_pair::french_east_indies_14090_window();
    // Item 1481: and run639's, where it stood before run642.
    let french_14786 = super::third_pair::french_east_indies_14786_window();
    // Item 1487: and run642's, where it stood before run655.
    let french_15344 = super::third_pair::french_east_indies_15344_window();
    // Item 1500: and run655's, where it stood before run657.
    let french_16857 = super::third_pair::french_east_indies_16857_window();
    // Item 1508: and run657's, where it stood before run661.
    let french_17171 = super::third_pair::french_east_indies_17171_window();
    // Item 1514: and run661's, where it stood before run662.
    let french_17244 = super::third_pair::french_east_indies_17244_window();
    // Item 1511: the coverage pair, the newest pair's open word; item
    // 1532: and run669's, where it stood before run672; item 1544: and
    // run672's, where it stood before run678; item 1552: and run678's,
    // where it stood before run679.
    let coverage_185 = super::coverage_pair::coverage_frame_185_window();
    let coverage_583 = super::coverage_pair::coverage_frame_583_window();
    let coverage_982 = super::coverage_pair::coverage_frame_982_window();
    let coverage = super::coverage_pair::coverage_pair_word_window();
    // Item 1549: the third map in the coverage lobby, run677's blocks around
    // the word 12 and run680's around the word 720.
    let sahara_12 = super::sahara_coverage::sahara_coverage_frame_12_window();
    let sahara_720 = super::sahara_coverage::sahara_coverage_frame_720_window();
    // Item 1555: and run681's blocks around the word 1197 and the word 1250.
    let sahara_1197 = super::sahara_coverage::sahara_coverage_frame_1250_window();
    // Item 1561: and run683's blocks around the word 1582.
    let sahara_1582 = super::sahara_coverage::sahara_coverage_word_window();
    let seen = compared::stop();
    let (
        Some(w),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
        Some(_),
    ) = (
        walked,
        toughest,
        french_east,
        french_closing,
        french_lakes,
        french_scholars,
        french_8182,
        french_8236,
        french_8385,
        french_8840,
        french_9655,
        french_9777,
        french_10131,
        french_10802,
        french_11582,
        french_12794,
        french_12952,
        french_14090,
        french_14786,
        french_15344,
        french_16857,
        french_17171,
        french_17244,
        coverage_185,
        coverage_583,
        coverage_982,
        coverage,
        sahara_12,
        sahara_720,
        sahara_1197,
        sahara_1582,
    )
    else {
        eprintln!("skipping: the open words' captures are not all on disk");
        return;
    };
    assert!(w.blocks >= 4, "the closing window is {} blocks", w.blocks);
    assert!(
        !seen.is_empty(),
        "the instrument registered nothing on {} blocks",
        w.blocks
    );
    let recs = crate::ledger::records(include_str!("../gamelog.rs"));
    let mut wrong: Vec<String> = Vec::new();
    let mut known: BTreeSet<String> = BTreeSet::new();
    for r in &recs {
        for f in &r.fields {
            known.insert(format!("{}.{}", r.name, f));
        }
        if let Some((_, who)) = NOT_THE_INSTRUMENT_S.iter().find(|(n, _)| *n == r.name) {
            let stale: Vec<&String> = seen
                .iter()
                .filter(|s| s.split('.').next() == Some(r.name.as_str()))
                .collect();
            if !stale.is_empty() {
                wrong.push(format!(
                    "{}: pinned as not the instrument's ({who}), but it registered {stale:?}",
                    r.name
                ));
            }
            continue;
        }
        let pinned: BTreeSet<&str> = UNCOMPARED_BY_THE_INSTRUMENT
            .iter()
            .find(|(n, _)| *n == r.name)
            .map(|(_, f)| f.split_whitespace().collect())
            .unwrap_or_default();
        let actual: BTreeSet<&str> = r
            .fields
            .iter()
            .filter(|f| !seen.contains(&format!("{}.{}", r.name, f)))
            .map(String::as_str)
            .collect();
        let arrived: Vec<&&str> = actual.difference(&pinned).collect();
        let gone: Vec<&&str> = pinned.difference(&actual).collect();
        if !arrived.is_empty() || !gone.is_empty() {
            wrong.push(format!(
                "{}: uncompared and not pinned {arrived:?}; compared now, delete from the pin {gone:?}",
                r.name
            ));
        }
    }
    // A registration naming no parsed field is a site whose label drifted.
    let unknown: Vec<&String> = seen.iter().filter(|s| !known.contains(*s)).collect();
    if !unknown.is_empty() {
        wrong.push(format!(
            "registered, but no record has the field: {unknown:?}"
        ));
    }
    assert!(
        wrong.is_empty(),
        "the compared pin, over {} blocks and {} registrations:\n  {}",
        w.blocks,
        seen.len(),
        wrong.join("\n  ")
    );
}
