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
    EAST_INDIES_BARK_BLOCK, EAST_INDIES_BLOCKEDWALK_BLOCK, EAST_INDIES_BLOCKWORD_BLOCK,
    EAST_INDIES_CAST_BLOCK, EAST_INDIES_COLUMNWORD_BLOCK, EAST_INDIES_EXPLORE_BLOCK,
    EAST_INDIES_FIGUREWORD_BLOCK, EAST_INDIES_GATHER_BLOCK, EAST_INDIES_GROUPWORD_BLOCK,
    EAST_INDIES_GUARDWORD_BLOCK, EAST_INDIES_IDLE_BLOCK, EAST_INDIES_LEADERWORD_BLOCK,
    EAST_INDIES_MAKE_BLOCK, EAST_INDIES_MARKETWORD_BLOCK, EAST_INDIES_MERCS_BLOCK,
    EAST_INDIES_SLOT_BLOCK, EAST_INDIES_TURNWORD_BLOCK, EAST_INDIES_WALKWORD_BLOCK,
    EAST_INDIES_WRAP_BLOCK, EAST_INDIES_WRAPWORD_BLOCK, GOLDEN_WORD_CHAPTER_SEVEN_B,
    GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL, GOLDEN_WORD_CHAPTER_SIX,
    GOLDEN_WORD_CHAPTER_THREE_RESTAGE, GOLDEN_WORD_CHAPTER_TWO, GREAT_LAKES_ATTACKED_BLOCK,
    GREAT_LAKES_BIRTH_BLOCK, GREAT_LAKES_CIVIC_BLOCK, GREAT_LAKES_COPY_BLOCK,
    GREAT_LAKES_DETOUR_BLOCK, GREAT_LAKES_ESCORT_BLOCK, GREAT_LAKES_FOREST_CELL_BLOCK,
    GREAT_LAKES_GIVEUP_BLOCK, GREAT_LAKES_MAKE_BLOCK, GREAT_LAKES_MIRROR_BLOCK,
    GREAT_LAKES_PATRIOT_BLOCK, GREAT_LAKES_PYRAMIDS_BLOCK, GREAT_LAKES_RECRUIT_BLOCK,
    GREAT_LAKES_RETRY_BLOCK, GREAT_LAKES_ROAD_BLOCK, GREAT_LAKES_SQUAD_BLOCK,
    GREAT_LAKES_STAND_BLOCK, GREAT_LAKES_UPGRADE_BLOCK, GREAT_LAKES_VALS_BLOCK,
    GREAT_LAKES_WONDER_BLOCK, WIDENING_CHAPTER_TWO,
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
    ("GAME/FRAME/ANIMALDATA", "aid ox whom"),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA",
        "air_alt cavarch_o cavarch_uid cavarch_who full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] last_angle node_flags o turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who",
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
    (
        "GAME/FRAME/BUILDDATA",
        "attack_ox attack_whom dock flags fort founder healing increment infiltrate infiltrate2 oil_well recharging stance wonder",
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
    (
        "GAME/FRAME/BUILDDATA/WALLDATA/OBJECT",
        "down down_who flags healing hold_frames increment infiltrated inside_down inside_down_who launch_frames length myhits mylos near_o near_who size uid up up_who visible",
    ),
    ("GAME/FRAME/CITIES", "increment length size"),
    (
        "GAME/FRAME/CITIES/CITY",
        "London Napata Norwich flags increment length size",
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
    // which this crate does not keep, and `reg_buildings` stay.
    (
        "GAME/FRAME/LEADERDATA",
        "(int) agendas[scan] ages_get() ages_queued aggression[scan] air_queued air_units ally_stamp[scan] anti_att att attack_stamp[scan] attrition_stamp attrition_stamp2 attrition_stamp3 average_damage_rate average_death_rate average_hit_rate average_kill_rate barracks_garr barracks_queued barracks_units base_rate[scan] best_armor best_attack best_move best_pop bit_values bits blacken bonus bonus_cap[NUM_COMMON] broke_alliance[scan] buildings_lost buildings_razed capital_stamp[scan] chat_status[scan] cities_captured cities_lost city_mark city_mine city_name combat_queued combat_units counteroffer[scan] ctw_hero_retreat_stamp ctw_hero_stamp damage_current_frame damage_fifteen_seconds deaths_current_frame deaths_fifteen_seconds defeat_stamp defeat_type defensive discovered_get() dock_mark dock_queued dock_units dow[scan] economic epochs_get() epochs_queued explored factory_queued factory_units flags flock_stamp fort_mark frame_battle gift_stamp[scan] good_deeds[scan] got_diplo_message handicap hero_mark high_buildings[scan] hire_stamp[scan] hire_who[scan] hits_current_frame hits_fifteen_seconds increment invaders[scan] kills_current_frame kills_fifteen_seconds last_spoke[scan] last_taunt[scan] length list[scan] lost_capital_modifier lost_capital_stamp lost_capital_timer lost_city_stamp made_peace[scan] misery missiles_used multi_diff nuke_stamp nukes_in_flight nukes_launched nukes_used num_bonus_cards[scan] num_buildings[scan] num_ctw_rate_bonuses[scan] oil_well_mark peasants_garr pop_cap pop_issues popwin_stamp popwin_timer raid_stamp[scan] rares_collected[scan] reg_buildings[scan][scan2] reg_terr[scan] repair_stamp retargets scholar_militia scout_garr senates_built size special_mark stable_garr stable_queued stable_units strong[scan] supply_mark support support_stamp taunt_frame[scan] team_color territory_high tribute_demanded[scan] tribute_stamp[scan] tributes[scan] units_killed units_lost victory_type village_mine weak[scan] wonderwin_stamp wonderwin_timer",
    ),
    (
        "GAME/FRAME/LEADERDATA/DIPLOMACY",
        "agree any_offer attacks[scan] offers[scan] treaty",
    ),
    (
        "GAME/FRAME/UNITDATA",
        "air_alt cavarch_o cavarch_uid cavarch_who full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] last_angle node_flags o turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who",
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
    // read off the building's `BUILDQUEUE`.
    (
        "GAME",
        "process_attack process_build process_buildmask process_eject_all process_flight process_follow process_form process_garrison process_group, process_guard process_move_to process_move_to_2 process_patrol process_queue_up process_set_transport process_spell process_swarm_around",
    ),
    // **Item 746's run223 windows are the first to print a player's air
    // order**: the `STRAFEORDER` a flight home builds and the
    // `AIRPATROLORDER` a strike on an unseen target becomes
    // (`docs/GOLDEN.md` §25). The order parser reads each `AIRORDER` row
    // and the strafe's `xx`/`yy` (`docs/ORDERS.md` §32); what stays unread
    // is the `AIRORDER`'s own `UNITORDER` copy, the same second copy as
    // the grouped moves' above, and the patrol's array allocation.
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
    // Pinned 2026-09-25, the fourteenth pass, on run202's blocks 15383..
    // 15387: 137 registrations. **A site gated on the window's content
    // registers only when it runs**, so a row here is one of two things
    // and the comment says which: a field no site compares, or a site
    // the window never reached (no death, no queued build, no patrol,
    // guard, garrison, cast or ground order, no group move, no
    // collision point on these six blocks). The second kind leaves this
    // pin the day a window that reaches it is driven here.
    //
    // `UnitDump`: `uid` is the identity behind `o` (ledger); `flags`,
    // `infiltrated` and the container pair `up`/`up_who`/`down`/
    // `down_who` no site compares.
    ("UnitDump", "down down_who flags infiltrated uid up up_who"),
    // `Guy`: `ox`/`whom` are chapter one's word and compared by its own
    // widening (item 530), `last_pos` by run86's (item 271); `kind`,
    // `guy_num` and `guy_flags` no site compares.
    ("Guy", "guy_flags guy_num kind last_pos ox whom"),
    // `OrderDump`: the move row leaves `tolerance`, `retry`, `attempts`
    // and `orig_x`/`orig_y` to run29's field-table test on purpose
    // (`compare_orders`). `orig` is carried on every move since item 738
    // (`MoveOrder::orig`, `finish_insert`'s replay point, run219), and
    // comparing it adds two rows past Great Lakes' word to run218's
    // fenced widening, `1/0`'s explore on 16470 whose point has already
    // parted: the compare waits for that widening's owner; `coll_x`/`coll_y` are compared since the window
    // moved to run211's 15609 (item 722), where a collision point stands
    // on both sides; the `GROUPORDER`, patrol, guard, garrison, cast and ground
    // rows on their orders, none of which stand on this window; `uid`,
    // `metric`, `build_type`, `non_flat_gather` and the attack order's
    // `mandatory defensive in_range ever_in_range new_ord def_x def_y`
    // no site compares. The strafe's row since item 746 — `mandatory`,
    // the `AIRORDER`'s `air_oxx air_whose cruising_alt sharp_turn air_old
    // returning` and `strafe_xx strafe_yy` — is compared on every strafe
    // (`compare_orders`, run223), and none stands on this window.
    (
        "OrderDump",
        "ag_accuracy ag_att_x ag_att_y ag_attack_unit air_old air_oxx air_whose \
         attempts build_type cast_paid cast_spell cruising_alt def_x def_y defensive \
         ever_in_range form_id garrison_search group_angle group_id in_group in_range \
         mandatory metric new_ord non_flat_gather orig_x orig_y oxx patrol_x patrol_y \
         retry returning sharp_turn strafe_xx strafe_yy tolerance uid waypoint whose",
    ),
    // `BuildDump`: `queue` registers with a non-empty queue whose depths
    // agree, and none stands on this window. **`orig_type` no site
    // compares** — the comparator's own comment
    // says `orig_type` was "parsed and neither compared" and is compared
    // now, and it is not: the position is. The pin's first catch, the
    // day it was built; parked for a widening rather than fixed here.
    // **Item 763 compares `damage` and `damage_frac`** (parked 728's
    // two of three, run223's bombed Barracks); `orig_type` remains.
    // `flags`, `max_age`, `mtn`, `cliff`, `mining_size`,
    // `construct_hits`, `ever_seen` and `ever_seen_completed` no site
    // compares. `job_counter` is compared only while **both** sides call
    // a site unfinished (the construction clock, `harness.rs`), and no
    // site stands unfinished on run226's 17098..17102, the window since
    // item 742; run218's 16459..16463 had one.
    (
        "BuildDump",
        "cliff construct_hits ever_seen ever_seen_completed flags \
         job_counter max_age mining_size mtn orig_type queue",
    ),
    // Registered with `BuildDump.queue`, on a queued build.
    ("QueueItemDump", "cost good job_counter ty"),
    // `DEATH_OBJS`: the window holds no death; the rows register on one.
    ("DeathDump", "cur_anim first_frame gpiece o valid who"),
    // The pool lists compare `id`, `who` and the members' `o`; the rest of
    // the group record is `diff::army`'s and the groups tests'.
    (
        "GroupDump",
        "army buildings disband facing form form_num new_speed num o_angle o_dist \
         order_num ox oy priority role speed stamp think_frame",
    ),
    ("GroupMemberDump", "angle curr_x curr_y off_x off_y"),
];

/// **Every field the parser carries is compared by the shared instrument
/// on the Great Lakes word's own window, or pinned above.** The window is
/// `harness::tests::great_lakes_word_window` — the word's block and two
/// on either side, replayed through the chain every widening past run196
/// replays — walked with the recorder on; a machine without the chain
/// says so. What this checks that the `UNREAD` pin cannot: that a key
/// which is *parsed* is also *compared*, per record, with both sides
/// present — the gap five landings in two tranches turned on.
#[test]
fn every_parsed_field_is_compared_by_the_instrument_or_pinned() {
    use super::compared;
    compared::start();
    let walked = super::harness::tests::great_lakes_word_window();
    let seen = compared::stop();
    let Some(w) = walked else {
        eprintln!("skipping: the Great Lakes word's chain is not all on disk");
        return;
    };
    assert!(w.blocks >= 4, "the word's window is {} blocks", w.blocks);
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
