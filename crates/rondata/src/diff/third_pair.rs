//! The third pair: human Nubians, French AI, seed 12345 and Toughest.
//! Each map owns a fresh full-start sibling; the second pair stays pinned.

use super::testkit::*;
use super::*;
use crate::testenv::dump;

const EAST_START: &str = "gamelog-run595-islands-french-toughest-start.txt";
const EAST_LONG: (&str, &str) = (
    "gamelog-run600-islands-french-toughest-24k-trace.txt",
    "rontrace-run600.log",
);
const LAKES_START: &str = "gamelog-run597-lakes-french-toughest-start.txt";
const LAKES_LONG: (&str, &str) = (
    "gamelog-run598-lakes-french-toughest-24k-trace.txt",
    "rontrace-run598.log",
);

#[test]
fn third_pair_starts_change_the_ai_nation_in_the_same_lobby() {
    for (start, old) in [
        (EAST_START, "gamelog-run346-islands-toughest-24k-trace.txt"),
        (
            LAKES_START,
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
        ),
    ] {
        let (Some(a), Some(b)) = (dump(start), dump(old)) else {
            continue;
        };
        let (a, b) = (crate::capture::read(&a), crate::capture::read(&b));
        let (a, b) = (Log::parse(&a), Log::parse(&b));
        let (a, b) = (a.initial().unwrap(), b.initial().unwrap());
        assert_eq!(a.game_info, b.game_info, "only the AI nation input moves");
        assert!(
            a.checksums.last().is_some(),
            "the full dump carries its setup seed"
        );
        let tribes: Vec<_> = a
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe, l.leader_flags & 4 != 0))
            .collect();
        assert_eq!(tribes, [(0, 4, true), (1, 10, false)]);
        assert!(!a.heights.is_empty() && !a.herds.is_empty() && !a.goods.is_empty());
        assert_eq!(a.players.len(), b.players.len());
        for (who, (a, b)) in a.players.iter().zip(&b.players).enumerate() {
            // The final bare line is the nation-derived display name, not a setting.
            let fields = |v: &Vec<(&str, &str)>| -> Vec<(String, String)> {
                v.iter()
                    .filter(|(k, v)| v.parse::<i64>().is_ok() && !(who == 1 && *k == "tribe"))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            };
            assert_eq!(fields(a), fields(b), "player {who}'s other settings");
        }
    }
}

#[test]
fn run600_french_east_indies_word() {
    check_long(EAST_LONG.0);
    let Some(w) = third::walk_from(EAST_START, EAST_LONG) else {
        return;
    };
    assert_eq!(w.difficulty, 5);
    assert_eq!(w.last, ai_word_length("EastIndiesFrench"));
    assert!(
        w.count >= THIRD_PAIR_WORD_EAST_INDIES && w.sequence >= THIRD_PAIR_WORD_EAST_INDIES,
        "{}",
        w.row
    );
}

#[test]
fn run598_french_great_lakes_word() {
    check_long(LAKES_LONG.0);
    let Some(w) = third::walk_from(LAKES_START, LAKES_LONG) else {
        return;
    };
    assert_eq!(w.difficulty, 5);
    assert_eq!(w.last, ai_word_length("GreatLakesFrench"));
    assert!(
        w.count >= THIRD_PAIR_WORD_GREAT_LAKES && w.sequence >= THIRD_PAIR_WORD_GREAT_LAKES,
        "{}",
        w.row
    );
}

fn incomplete_long(log: &Log<'_>) -> Vec<&'static str> {
    let init = log.initial().expect("initial state");
    let mut errors = Vec::new();
    if personality_brackets(&init.checksums).len() != 1 {
        errors.push("missing AI personality seed bracket");
    }
    // Shutdown has two documented shapes: trailing siblings or children of
    // the final FRAME. The latter is already read by frame_states.
    let closing = log.final_state().or_else(|| log.frame_states().pop());
    if closing.is_none_or(|f| f.units.is_empty()) {
        errors.push("missing closing units");
    }
    errors
}

fn check_long(name: &str) {
    let Some(path) = dump(name) else { return };
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    assert_eq!(
        incomplete_long(&log),
        Vec::<&str>::new(),
        "{name} is not scoreable"
    );
}

#[test]
fn run596_is_refused_as_a_complete_pair_capture() {
    let Some(path) = dump("gamelog-run596-islands-french-toughest-24k-trace.txt") else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        incomplete_long(&Log::parse(&text)),
        [
            "missing AI personality seed bracket",
            "missing closing units"
        ]
    );
}

#[test]
fn third_pair_openings_have_the_original_figures_and_personality() {
    for (start, base) in [(EAST_START, EAST_LONG), (LAKES_START, LAKES_LONG)] {
        let Some(w) = harness::tests::widen_on_siblings(
            &[start],
            true,
            base,
            start,
            &[(start, 1)],
            (1, 1),
            1,
            &[1],
            true,
        ) else {
            continue;
        };
        eprintln!(
            "opening {start}: {} blocks, {} differing keys; missing {:?}",
            w.blocks,
            w.firsts.len(),
            w.missing
        );
        assert!(w.missing.is_empty());
        assert!(
            w.firsts.keys().all(|(_, _, field)| field != "guys.len"
                && !field.starts_with("g.")
                && !field.starts_with("leader:PERSONALITY.")),
            "the starting cast and AI personality must agree: {:?}",
            w.firsts
        );
        assert_eq!(w.blocks, 1);
    }
}

/// run602: the third pair's East Indies word, whole records around frame 986.
pub(crate) fn french_east_indies_capacity_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run602",
        &[("gamelog-run602-islands-french-toughest-986.txt", 981)],
        WIDENING_FRENCH_CAPACITY,
        1,
        &[987],
        true,
    )
}

/// run603: the successor word after French timber capacity, frame 7356.
pub(crate) fn french_east_indies_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run603",
        &[("gamelog-run603-islands-french-toughest-7356.txt", 7351)],
        WIDENING_FRENCH_EAST_INDIES,
        1,
        &[THIRD_PAIR_WORD_EAST_INDIES + 1],
        true,
    )
}

#[test]
fn run603_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 120, "initial run603 baseline");
}

/// run601: the third pair's Great Lakes word, whole records around frame 2576.
pub(crate) fn french_great_lakes_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKES_START],
        true,
        LAKES_LONG,
        "run601",
        &[("gamelog-run601-lakes-french-toughest-2576.txt", 2571)],
        WIDENING_FRENCH_GREAT_LAKES,
        1,
        &[THIRD_PAIR_WORD_GREAT_LAKES + 1],
        true,
    )
}

#[test]
fn run602_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_capacity_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 55, "run602 after French worker capacity");
    assert!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 2010)),
        "the preserved camp agrees in every compared field"
    );
    assert!(
        w.firsts
            .keys()
            .all(|(who, _, field)| *who != 1 || !field.ends_with("gather_slots[1:timber]")),
        "the opening timber slot deficit is gone"
    );
}

#[test]
fn run601_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_great_lakes_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 81, "run601 after French worker capacity");
}

/// An empty projectile comparison still checks both sides, while a missing
/// group dumper must never masquerade as an empty original pool.
#[test]
fn third_pair_windows_have_groups_and_no_projectiles_on_either_side() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    for (start, base, capture, window) in [
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run603-islands-french-toughest-7356.txt",
            WIDENING_FRENCH_EAST_INDIES,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run602-islands-french-toughest-986.txt",
            WIDENING_FRENCH_CAPACITY,
        ),
        (
            LAKES_START,
            LAKES_LONG,
            "gamelog-run601-lakes-french-toughest-2576.txt",
            WIDENING_FRENCH_GREAT_LAKES,
        ),
    ] {
        let (Some(start), Some(base_path), Some(path)) = (dump(start), dump(base.0), dump(capture))
        else {
            continue;
        };
        let (start_text, base_text) =
            (crate::capture::read(start), crate::capture::read(base_path));
        let (start_log, base_log) = (Log::parse(&start_text), Log::parse(&base_text));
        let sibling = start_log.initial().unwrap();
        let mut init = base_log.initial().unwrap();
        assert!(
            same_start(&init, &sibling),
            "all starting building identities and positions"
        );
        assert_eq!(init.game_info, sibling.game_info, "the same lobby");
        assert_eq!(
            init.checksums.last().unwrap().seed,
            sibling.checksums.last().unwrap().seed
        );
        borrow_from_siblings(&mut init, &[&sibling]);
        if let Some(t) = trace(base.1) {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
        let mut compared = 0;
        for n in 1..=window.1 {
            built.tick();
            if n < window.0 {
                continue;
            }
            let at = ix
                .frames()
                .iter()
                .position(|f| f.number == n)
                .expect("every window block");
            let raw = ix.read_frame(at).unwrap();
            let log = Log::parse(&raw);
            let (_, block) = log.frames().into_iter().find(|(f, _)| *f == n).unwrap();
            assert!(
                !crate::gamelog::groups(block).is_empty(),
                "{capture} block {n}: group dumper missing"
            );
            assert!(
                crate::diff::ammo::blocks(&raw).is_empty(),
                "{capture} block {n}: original projectiles now present"
            );
            assert!(
                built.sim.projectiles.is_empty(),
                "{capture} block {n}: simulated projectiles now present"
            );
            compared += 1;
        }
        assert_eq!(compared, 13);
    }
}
