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
//! What it does not check: that a *parsed* field is *compared*. That is
//! `crate::ledger`'s side. And a record family with a parser of its own
//! that never touches `Block` — `AMMO`, whose scanner is `diff::ammo`
//! — is named in [`OWN_PARSER`] with the module that reads it, and its
//! keys are that module's to keep.

use std::collections::{BTreeMap, BTreeSet};

use crate::gamelog::{Block, Log, reads};

use super::testkit::{
    EAST_INDIES_BARK_BLOCK, EAST_INDIES_CAST_BLOCK, EAST_INDIES_EXPLORE_BLOCK,
    EAST_INDIES_GATHER_BLOCK, EAST_INDIES_IDLE_BLOCK, EAST_INDIES_MAKE_BLOCK,
    EAST_INDIES_MERCS_BLOCK, GOLDEN_WORD_CHAPTER_SEVEN_B, GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
    GOLDEN_WORD_CHAPTER_SIX, GOLDEN_WORD_CHAPTER_THREE_RESTAGE, GOLDEN_WORD_CHAPTER_TWO,
    GREAT_LAKES_DETOUR_BLOCK, GREAT_LAKES_MAKE_BLOCK, GREAT_LAKES_UPGRADE_BLOCK,
    WIDENING_CHAPTER_TWO,
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
    ("GAME/FRAME/ANIMALDATA", "aid ox whom"),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA",
        "air_alt cavarch_o cavarch_uid cavarch_who full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation *((dword*) des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] last_angle node_flags o turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/OBJECT",
        "healing inside_down inside_down_who launch_frames near_o near_who",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/STACK<TYPE>",
        "increment length size",
    ),
    ("GAME/FRAME/BUILDDATA/BUILDQUEUE", "queue_size"),
    (
        "GAME/FRAME/BUILDDATA/WALLDATA",
        "demolition frame_started gpiece helpers job_counter_2",
    ),
    (
        "GAME/FRAME/BUILDDATA/WALLDATA/OBJECT",
        "down down_who healing hold_frames infiltrated inside_down inside_down_who launch_frames myhits mylos near_o near_who uid up up_who visible",
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
        "(int) agendas[scan] ages_get() ages_queued aggression[scan] air_queued air_units ally_stamp[scan] anti_att att attack_stamp[scan] attrition_stamp attrition_stamp2 attrition_stamp3 average_damage_rate average_death_rate average_hit_rate average_kill_rate barracks_garr barracks_queued barracks_units base_rate[scan] best_armor best_attack best_move best_pop bit_values bits blacken bonus bonus_cap[NUM_COMMON] broke_alliance[scan] buildings_lost buildings_razed capital_stamp[scan] chat_status[scan] cities_captured cities_lost city_mark city_mine city_name combat_queued combat_units counteroffer[scan] ctw_hero_retreat_stamp ctw_hero_stamp damage_current_frame damage_fifteen_seconds deaths_current_frame deaths_fifteen_seconds defeat_stamp defeat_type defensive discovered_get() dock_mark dock_queued dock_units dow[scan] economic epochs_get() epochs_queued explored factory_queued factory_units flags flock_stamp fort_mark frame_battle gift_stamp[scan] good_deeds[scan] got_diplo_message gov_hero_frame handicap hero_mark high_buildings[scan] hire_stamp[scan] hire_who[scan] hits_current_frame hits_fifteen_seconds increment invaders[scan] kills_current_frame kills_fifteen_seconds last_spoke[scan] last_taunt[scan] length list[scan] lost_capital_modifier lost_capital_stamp lost_capital_timer lost_city_stamp made_peace[scan] misery missiles_used multi_diff nuke_stamp nukes_in_flight nukes_launched nukes_used num_bonus_cards[scan] num_buildings[scan] num_ctw_rate_bonuses[scan] oil_well_mark peasants_garr pop_cap pop_issues popwin_stamp popwin_timer raid_stamp[scan] rares_collected[scan] reg_buildings[scan][scan2] reg_terr[scan] repair_stamp retargets scholar_militia scout_garr senates_built size special_mark stable_garr stable_queued stable_units strong[scan] supply_mark support support_stamp taunt_frame[scan] team_color territory_high tribute_demanded[scan] tribute_stamp[scan] tributes[scan] units_killed units_lost victory_type village_mine weak[scan] wonder_mark wonderwin_stamp wonderwin_timer",
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
        "(int)off_x (int)off_y (int)variation *((dword*) des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] last_angle node_flags o turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who",
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
    let ch8 = golden_dump("ch8");
    let r136 = crate::testenv::dump("gamelog-run136-greatlakes-detour.txt");
    let r163 = crate::testenv::dump("gamelog-run163-greatlakes-upgradeword.txt");
    let r143 = crate::testenv::dump("gamelog-run143-eastindies-bark.txt");
    let r139 = crate::testenv::dump("gamelog-run139-eastindies-makelist.txt");
    let r149 = crate::testenv::dump("gamelog-run149-eastindies-animal.txt");
    let r152 = crate::testenv::dump("gamelog-run152-eastindies-gatherbuilding.txt");
    let r155 = crate::testenv::dump("gamelog-run155-eastindies-longword.txt");
    let r159 = crate::testenv::dump("gamelog-run159-eastindies-idleword.txt");
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
    // **Chapter eight's word, on run171** (item 660): the first capture
    // with a General and a Spy in it, and the first golden capture at
    // `LEADERS=5`, whose leader record prints the diplomacy row. The
    // window is the word's block with two on either side, as six's.
    // Item 664 moved the word 617 → 659, and both windows are driven:
    // 617's carries the Spy's explore order, the first on this lobby.
    if let Some(p) = &ch8 {
        for w in [617, super::testkit::GOLDEN_WORD_CHAPTER_EIGHT] {
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
