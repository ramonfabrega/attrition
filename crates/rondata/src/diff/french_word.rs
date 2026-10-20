//! **The battery's French word** (DECISIONS 72, 83, 84; item 3347): the
//! held-out battery's second open game once item 3343 closed the Bantu,
//! promoted by item 3343's recipe (`diff::french_word`). Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the French (10)** at
//! Difficulty 4, seed 12345 — the lobby item 3343 minted under entry 72's
//! rule (2) (`docs/AI.md` §125, §276), scored by its first try at 13422 of
//! 16,764 and promoted out of the battery here, so it is a scored lobby
//! from this item (`battery::SCORED_LOBBIES`) and no longer held out.
//! Captures: run977 (a `DUMP_ALL` start) and run977b (the draw stream to the
//! game's end on 16764, `cover=0`, its setup and closing blocks only); the
//! word's window is `docs/AI.md` §278's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const FRENCH: Lobby = Lobby {
    name: "GreatSaharaFrenchTier4",
    map_style: 7,
    ai_tribe: 10,
    difficulty: 4,
    start: "gamelog-run977-greatsahara-french-diff4-start.txt",
    long: (
        "gamelog-run977b-greatsahara-french-diff4-24k-trace.txt",
        "rontrace-run977b.log",
    ),
    length: 16764,
    // Item 3343, the battery's first try, count and sequence both at
    // 13422: ours 14 draws, theirs 19, at 6 ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::init_real+0x52`. Item 3347
    // promoted it to the AI word. Item 3347, `train_time`'s French siege
    // arm (`FRENCH_SIEGE_SPEED`, the Siege Factory `1/2026`'s Trebuchet
    // 30000 → 24000, born on 13422): 13422 → 13843, ours 8 draws, theirs
    // 10, at 5 ours `Farms::inc_time+0x1ae`, theirs
    // `Ammo::do_damage+0xc59`. Item 3347, the Crossbowman's and the
    // Cavalry Archer's release nodes (`launch::MEASURED`, pieces 122 and
    // 138; `1/39`'s round from (9401, 30970, 842), not its guy's (9480,
    // 30936, 614), 13 frames against 14): 13843 → 14604, ours 15 draws,
    // theirs 16, at 6 ours `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`,
    // theirs `Unit::fight+0x9b0`.
    // Item 3347, `update_seen`'s own-cell arm for a decoy (the decoy
    // Hussars `1/100` lights the Citizen `0/1`'s half-cell on 14596 and
    // holds it as a valid target on 14604, where ours called it unseen and
    // searched without the captain's roll): 14604 → 16764, the game's last
    // frame.
    count: 16764,
    sequence: 16764,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const FRENCH_WORD: i64 = if FRENCH.count < FRENCH.sequence {
    FRENCH.count
} else {
    FRENCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn french_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(FRENCH.start, FRENCH.long) else {
        return;
    };
    eprintln!(
        "FRENCH {}: count {} sequence {} of {}; {}",
        FRENCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, FRENCH.difficulty, "the lobby read back");
    pin_eq!(w.last, FRENCH.length, "the game's last frame");
    pin_eq!(w.count, FRENCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        FRENCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn french_word_is_no_other_lobby() {
    let p = &FRENCH;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
    }
}

/// run982: the word 13422's window and the word 13843's, blocks 13416..14093
/// at `GUYS=2`, `AMMO=5` (item 3347); run977 dumps blocks 1, 36 and 37 and
/// run977b its setup and closing blocks only.
pub(crate) const RUN982: &str =
    "gamelog-run982-greatsahara-french-diff4-window-13416-14093-ammo.txt";

/// The window: block 13416 through 14093; the words' own blocks are 13423
/// and 13844.
pub(crate) const WIDENING_FRENCH_FRAME_13422: (i64, i64) = (13416, 14093);

pub(crate) fn french_frame_13422_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH.start],
        true,
        FRENCH.long,
        "run982",
        &[(RUN982, WIDENING_FRENCH_FRAME_13422.0)],
        WIDENING_FRENCH_FRAME_13422,
        1,
        &[13423, 13844],
        true,
    )
}

#[test]
fn run982_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_frame_13422_window() else {
        return;
    };
    pin_eq!(w.blocks, 677, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3347. 41 keys after the release nodes (`cf3e7e2c2`): 36 standing
    // on 13416, as run978's — the leader's sites and aggression, the
    // formations, who=1's groups 64/65/68 and four of its guys' waypoints;
    // who=1's group and pool 66 on 13424 (the Trebuchet `1/86`, held in a
    // group ours forms and theirs does not) and 67 on 13600; and `1/82`'s
    // `dest_angle` on 13856. `0/0`'s wound on 13870 and `0/2000`'s raid stamp
    // on 13941 left with the Cavalry Archer's and the Dragoon's nodes.
    pin_eq!(w.firsts.len(), 41, "run982 with the release nodes");
}

/// run982b: the word 14604's window, blocks 14598..14854 at `GUYS=2`,
/// `AMMO=5` (item 3347).
pub(crate) const RUN982B: &str =
    "gamelog-run982b-greatsahara-french-diff4-window-14598-14854-ammo.txt";

/// The window: block 14598 through 14854; the word 14604's own block is
/// 14605.
pub(crate) const WIDENING_FRENCH_FRAME_14604: (i64, i64) = (14598, 14854);

pub(crate) fn french_frame_14604_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH.start],
        true,
        FRENCH.long,
        "run982b",
        &[(RUN982B, WIDENING_FRENCH_FRAME_14604.0)],
        WIDENING_FRENCH_FRAME_14604,
        1,
        &[14605],
        true,
    )
}

#[test]
fn run982b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_frame_14604_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3347. The word 14604's value diff is in no dumped key: who=1's
    // decoy Hussars `1/100` (type 211, `unit_masks` 0x4000f, `mylos` 1 on
    // both sides) reaches (4776, 28344) on 14604 and its attack on the
    // Citizen `0/1` becomes current; theirs spends the captain's roll
    // (`Unit::fight+0x9b0`) and on 14605 still holds `0/1` with a chase
    // move to (4440, 28392), where ours called `0/1` unseen — `seen` at
    // (11, 73) had no who=1 bit — and retargeted to the city `0/2000`. The
    // decoy's own-cell sight (`update_seen`'s `unit_masks & 1` arm) lights
    // the cell on 14596. With it the word left for the game's end; **38**
    // keys stand, all on 14598 but four: the standing set (as run982's),
    // who=1's groups and pools 64/65/67 with the members run982 lacked,
    // `1/43`'s defensive `def_x/def_y` (ours its own post, theirs (8712,
    // 31272)), and group and pool 66 on 14612 and 68 on 14631.
    pin_eq!(
        w.firsts.len(),
        38,
        "run982b with the decoy's own-cell sight"
    );
}

/// The closing state's window: run977b's closing block alone, the long's
/// last frame 16764 writing it (item 3347).
pub(crate) const WIDENING_FRENCH_CLOSING: (i64, i64) = (16763, 16766);

/// **The lobby's closing state** (item 3347, DECISIONS 55): run977b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
#[test]
fn run977b_french_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(FRENCH.long.0),
        dump(FRENCH.start),
        trace(FRENCH.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "French closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    for b in fin.builds.iter().filter(|b| (0..8).contains(&b.who)) {
        if harness::link_building(&built.sim, b.who, b.o).is_none() {
            eprintln!(
                "  unlinked building {}/{} at ({}, {}) flags {} damage {:?} orig_type {:?}",
                b.who, b.o, b.pos.x, b.pos.y, b.flags, b.damage, b.orig_type
            );
        }
    }
    pin_eq!(e.frame, 16765, "the closing block");
    pin_eq!(e.compared, 130, "units compared");
    // Item 3347. Every unit agrees; four buildings and a city do not. On
    // the last frame theirs' who=1 captures the human's city: `0/2000`
    // closes as `1/2058` (damage 1430), and three of its buildings,
    // `0/2001`, `0/2005` and `0/2006` by position (orig types 418, 435,
    // 436), as `1/2059`..`1/2061`; ours still holds `0/2000` at damage
    // 1198, uncaptured. The city's damage parts somewhere past
    // run982b's window, with no draw (run982c, a stride over
    // 14855..16765, is the window that finds it).
    pin_eq!(e.counts(), [0, 0, 0, 4, 0, 1, 0], "the closing counts");
}

/// run982d: the game's last blocks, 16740..16764 at `GUYS=2`, `AMMO=5`
/// (item 3347) — the frames before the capture the close parts on.
pub(crate) const RUN982D: &str =
    "gamelog-run982d-greatsahara-french-diff4-window-16740-16765-ammo.txt";

/// The window: block 16740 through 16764, the last before the closing one.
pub(crate) const WIDENING_FRENCH_LAST_BLOCKS: (i64, i64) = (16740, 16764);

pub(crate) fn french_last_blocks_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH.start],
        true,
        FRENCH.long,
        "run982d",
        &[(RUN982D, WIDENING_FRENCH_LAST_BLOCKS.0)],
        WIDENING_FRENCH_LAST_BLOCKS,
        1,
        &[16764],
        true,
    )
}

#[test]
fn run982d_s_last_blocks_are_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_last_blocks_window() else {
        return;
    };
    pin_eq!(w.blocks, 25, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3347. **38** keys, every one standing on 16740: the standing set,
    // who=1's groups and pools 64..67, `1/43`'s defensive point, `1/126`'s
    // newborn `g.angle[0]`, and who=1's `leftover[2:wealth]` ours 1661
    // against theirs 1421 (parted between 14854 and 16740, unwidened).
    // **Nothing parts on `0/2000`**: its damage is 1197 of 1200 on both
    // sides through block 16764 (run982c's stride agrees from 14856 on, one
    // timing tear at 16336 aside), so the close's capture is the last
    // frame's own hits — ours +1 to 1198, theirs at least +3, through the
    // threshold.
    pin_eq!(w.firsts.len(), 38, "run982d, the last blocks");
}
