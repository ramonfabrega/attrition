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

use super::testkit::{GOLDEN_WORD_CHAPTER_TWO, LONG_WORD_GREAT_LAKES, WIDENING_CHAPTER_TWO};

/// Record paths read by a parser of their own, outside `Block` — the
/// module that reads each is named, and this guard leaves them alone.
const OWN_PARSER: &[(&str, &str)] = &[("GAME/FRAME/AMMO", "diff::ammo::blocks")];

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
    ("GAME/FRAME/ANIMALDATA", "aid ox whom"),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA",
        "air_alt attrition cavarch_o cavarch_uid cavarch_who full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued o_down play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/ANIMALDATA/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation *((dword*) des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] hold_attack last_angle node_flags o ox queued_attack turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who whom",
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
    (
        "GAME/FRAME/GUY",
        "(int)off_x (int)off_y (int)variation *((dword*) angle avg_speed cur_time des_angle des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] des_x des_y guy_flags guy_num hold_attack last_angle last_speed last_time last_x last_y last_z node_flags o ox queued_attack stopped track_dx track_dy turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] type who whom x y z",
    ),
    (
        "GAME/FRAME/LEADERDATA",
        "ages_get() base_rate[scan] bonus bonus_cap[NUM_COMMON] bonus_cap[scan] bucket discovered_get() econ[scan] epoch_get(scan) epochs_get() escrow[scan] escrow_rate[scan] filled_gather_slots[scan] gather_slots[scan] gather_slots_high[scan] income leftover over_cap rate resource_cap resources support tributes[scan]",
    ),
    (
        "GAME/FRAME/UNITDATA",
        "air_alt attrition cavarch_o cavarch_uid cavarch_who full gather_down good_obj guy_mark healing hero increment inside_up_who length los_x los_y mana_burn myarmor num_queued o_down play queue_time rare size special spell_time supply trench_angle waiting",
    ),
    (
        "GAME/FRAME/UNITDATA/GROUPATTACKTOORDER/GroupMoveOrder/GROUPORDER/UNITORDER",
        "flags",
    ),
    (
        "GAME/FRAME/UNITDATA/GUY",
        "(int)off_x (int)off_y (int)variation *((dword*) des_node_flags des_turret_angles[0] des_turret_angles[1] des_turret_angles[2] des_turret_angles[3] hold_attack last_angle node_flags o ox queued_attack turret_angles[0] turret_angles[1] turret_angles[2] turret_angles[3] who whom",
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
    let r100 = crate::testenv::dump("gamelog-run100-greatlakes-valuewindow2.txt");
    if ch2.is_none() && r100.is_none() {
        eprintln!("skipping: neither the ch2 golden capture nor run100 is on disk");
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
    if let Some(p) = &r100 {
        frames += drive_capture(
            p,
            LONG_WORD_GREAT_LAKES - 2,
            LONG_WORD_GREAT_LAKES + 2,
            &mut paths,
        );
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
