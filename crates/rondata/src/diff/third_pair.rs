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

#[test]
fn run598_french_great_lakes_closing_state() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(LAKES_LONG.0), dump(LAKES_START), trace(LAKES_LONG.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
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
        "French Great Lakes closing: frame {} units {} counts {:?} torn {:?} off {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off
    );
    assert_eq!(e.frame, 5639);
    eprintln!(
        "Closing city fields: {:?}",
        harness::compare(&built, &fin, 8).city_diverged
    );
    assert_eq!(e.compared, 40);
    assert!(e.torn.is_empty());
    assert_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 8]);
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

/// run612: the successor after a wonder's start became first contact,
/// frame 8236 (item 1446).
pub(crate) fn french_east_indies_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run612",
        &[("gamelog-run612-islands-french-8236.txt", 8231)],
        WIDENING_FRENCH_EAST_INDIES,
        1,
        &[8237],
        true,
    )
}

#[test]
fn run612_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(
        w.firsts.len(),
        210,
        "run612 after a wonder's start is contact"
    );
}

/// run610: the successor after the university admission limit, frame 8182.
pub(crate) fn french_east_indies_8182_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run610",
        &[("gamelog-run610-islands-french-toughest-8182.txt", 8177)],
        WIDENING_FRENCH_EAST_INDIES_8182,
        1,
        &[8183],
        true,
    )
}

#[test]
fn run610_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_8182_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(
        w.firsts.len(),
        156,
        "run610 after a wonder's start is contact"
    );
}

/// run611: the first capture of this game past first contact, blocks
/// 8030..8043 (item 1446).
pub(crate) fn french_east_indies_contact_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run611",
        &[("gamelog-run611-islands-french-contact.txt", 8030)],
        WIDENING_FRENCH_CONTACT,
        1,
        &[8034],
        true,
    )
}

/// run613: the contact frame's own window, blocks 7940..7952 (item 1446).
pub(crate) fn french_east_indies_contact_7946_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run613",
        &[("gamelog-run613-islands-french-contact-7946.txt", 7940)],
        WIDENING_FRENCH_CONTACT_7946,
        1,
        &[7946],
        true,
    )
}

#[test]
fn run613_s_contact_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_contact_7946_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(
        w.firsts.len(),
        117,
        "run613 after a wonder's start is contact"
    );
    pin!(
        !w.firsts
            .keys()
            .any(|(_, _, field)| field.starts_with("leader:treaties") || field == "leader:wars"),
        "both met bits and the AI's war census agree on every block"
    );
}

/// **The original's own record of the contact, read off run613** (item
/// 1446): the wonder `1/2022` is started on sim-frame 7941 (block 7942),
/// and on block 7946 — the owner's `check_ever_seen` frame 7945 — both it
/// and the Senate `1/2021` beside it go from `ever_seen` 2 to 255 while
/// both leaders' met bits are set, and on no earlier block. This is what
/// [`east_indies_wonder_start_is_first_contact_on_every_building`] holds
/// this crate to on the same window.
#[test]
fn run613_dates_first_contact_on_block_7946() {
    let Some(path) = dump("gamelog-run613-islands-french-contact-7946.txt") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let mut seen = Vec::new();
    for at in 0..ix.frames().len() {
        let f = ix.frame_state(at).unwrap();
        if !(7940..=7952).contains(&f.n) {
            continue;
        }
        let es = |o: i64| {
            f.builds
                .iter()
                .find(|b| (b.who, b.o) == (1, o))
                .and_then(|b| b.ever_seen)
        };
        let raw = ix.read_frame(at).unwrap();
        let log = Log::parse(&raw);
        let met = |who: i64, other: &str| {
            let block = log.leader_block(f.n, who)?;
            crate::diff::leader::theirs(&block).get(other).copied()
        };
        seen.push((
            f.n,
            es(2021),
            es(2022),
            met(1, "treaties[0]"),
            met(0, "treaties[1]"),
        ));
    }
    assert_eq!(seen.len(), 13);
    for (n, a, b, m1, m0) in seen {
        let after = n >= 7946;
        let byte = Some(if after { 255 } else { 2 });
        let bit = Some(i64::from(after));
        assert_eq!((a, b, m1, m0), (byte, byte, bit, bit), "block {n}");
    }
}

#[test]
fn run611_s_contact_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_contact_window() else {
        return;
    };
    pin_eq!(w.blocks, 14, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(
        w.firsts.len(),
        147,
        "run611 after a wonder's start is contact"
    );
}

/// run603: the successor word after French timber capacity, frame 7356.
pub(crate) fn french_east_indies_scholar_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run603",
        &[("gamelog-run603-islands-french-toughest-7356.txt", 7351)],
        WIDENING_FRENCH_SCHOLAR,
        1,
        &[7357],
        true,
    )
}

#[test]
fn run603_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_scholar_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 108, "run603 after scholar admission");
    assert!(
        !w.firsts.keys().any(|(who, o, field)| *who == 1
            && ((*o == 45 && field == "extra")
                || (*o == 2016 && field == "queue:queued")
                || (*o == -1
                    && matches!(
                        field.as_str(),
                        "leader:scholars"
                            | "leader:active"
                            | "leader:control"
                            | "leader:num_units[2]"
                            | "leader:num_queued[2]"
                    )))),
        "the surplus queue, birth and census counts agree"
    );
}

/// run601: the third pair's Great Lakes word, whole records around frame 2576.
pub(crate) fn french_great_lakes_ruins_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKES_START],
        true,
        LAKES_LONG,
        "run601",
        &[("gamelog-run601-lakes-french-toughest-2576.txt", 2571)],
        WIDENING_FRENCH_GREAT_LAKES,
        1,
        &[2577],
        true,
    )
}

pub(crate) fn french_great_lakes_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKES_START],
        true,
        LAKES_LONG,
        "run609",
        &[(
            "gamelog-run609-lakes-french-toughest-closing-window.txt",
            5633,
        )],
        WIDENING_FRENCH_LAKES_CLOSING,
        1,
        &[5639],
        true,
    )
}

#[test]
fn run609_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_great_lakes_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 7, "six running blocks and closing state");
    pin!(
        w.missing.is_empty(),
        "every record key is read: {:?}",
        w.missing
    );
    pin_eq!(
        w.firsts.len(),
        110,
        "run609 closing residue, not whole-record parity"
    );
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
    let Some(w) = french_great_lakes_ruins_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 71, "run601 after ruins placement");
    pin!(
        w.firsts.keys().all(|(who, _, field)| *who != 1
            || !(field.starts_with("leader:SITE[8].") || field.starts_with("leader:SITE[9]."))),
        "both formerly swapped site records agree throughout the window"
    );
}

/// An empty projectile comparison still checks both sides, while a missing
/// group dumper must never masquerade as an empty original pool.
#[test]
fn third_pair_windows_check_groups_and_projectiles_on_both_sides() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    for (start, base, capture, window) in [
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run612-islands-french-8236.txt",
            WIDENING_FRENCH_EAST_INDIES,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run611-islands-french-contact.txt",
            WIDENING_FRENCH_CONTACT,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run613-islands-french-contact-7946.txt",
            WIDENING_FRENCH_CONTACT_7946,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run610-islands-french-toughest-8182.txt",
            WIDENING_FRENCH_EAST_INDIES_8182,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run603-islands-french-toughest-7356.txt",
            WIDENING_FRENCH_SCHOLAR,
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
        (
            LAKES_START,
            LAKES_LONG,
            "gamelog-run609-lakes-french-toughest-closing-window.txt",
            WIDENING_FRENCH_LAKES_CLOSING,
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
        let mut ammo_counts = (0, 0, 0);
        let mut ammo_differences = std::collections::BTreeSet::new();
        let mut ammo_unmodelled = std::collections::BTreeSet::new();
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
            let mut theirs = std::collections::BTreeMap::new();
            for (a, fields) in crate::diff::ammo::blocks(&raw) {
                assert_eq!(fields, 27, "every printed ammo field");
                assert_eq!(a.flags & 3, 2, "live projectile, no unmodelled crash round");
                assert!(theirs.insert((a.who, a.o, n - a.cur_time), a).is_none());
                ammo_unmodelled.insert((
                    a.traj,
                    a.dx,
                    a.start_roll_angle,
                    a.bank_dx,
                    a.bank_dy,
                    a.gpiece,
                    a.graph_index,
                    a.flags & !0x1f,
                ));
            }
            let sim = &built.sim;
            let mut ours = std::collections::BTreeMap::new();
            for p in &sim.projectiles {
                let (who, o) = super::golden::obj_ident(sim, p.shooter);
                assert!(
                    ours.insert((who, o, n - i64::from(p.cur_time)), p)
                        .is_none()
                );
            }
            ammo_counts.0 += theirs.len();
            ammo_counts.1 += ours.len();
            let keys: std::collections::BTreeSet<_> =
                theirs.keys().chain(ours.keys()).copied().collect();
            for key in keys {
                let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                    ammo_differences.insert((
                        n,
                        key,
                        "presence",
                        i64::from(ours.contains_key(&key)),
                        i64::from(theirs.contains_key(&key)),
                    ));
                    continue;
                };
                let mut rows = super::golden::ammo_value_rows(sim, p, a);
                rows.push(("index", i64::from(p.slot), a.index));
                for (field, mine, dumped) in rows {
                    ammo_counts.2 += 1;
                    if mine != dumped {
                        ammo_differences.insert((n, key, field, mine, dumped));
                    }
                }
            }
            compared += 1;
        }
        assert_eq!(compared, window.1 - window.0 + 1);
        eprintln!(
            "{capture}: ammo counts {ammo_counts:?}, differences {ammo_differences:?}, unmodelled {ammo_unmodelled:?}"
        );
        if window == WIDENING_FRENCH_LAKES_CLOSING {
            assert_eq!(
                ammo_counts,
                (7, 7, 140),
                "every lifetime and twenty comparisons per record"
            );
            assert_eq!(
                ammo_differences,
                std::collections::BTreeSet::from([
                    (5633, (1, 22, 5625), "angle", -1111097344, -1111425024),
                    (5633, (1, 22, 5625), "sx", 4944, 4945),
                    (5633, (1, 22, 5625), "sy", 30710, 30709),
                ]),
                "three launch-geometry differences remain explicit"
            );
            assert_eq!(
                ammo_unmodelled,
                std::collections::BTreeSet::from([
                    (1, 155512146, 0, 0, 0, 60348, 90, 0),
                    (1, 161070679, 0, 0, 0, 60348, 91, 0),
                    (1, 161980988, 0, 0, 0, 60348, 92, 0),
                ]),
                "engine-only fields are evidence, not sim parity"
            );
        } else {
            assert_eq!(
                ammo_counts,
                (0, 0, 0),
                "both pools empty on earlier windows"
            );
        }
    }
}

/// `(block, who, o, field, ours, theirs)`.
type SeenRow = (i64, i64, i64, &'static str, i64, i64);

/// Every building's `ever_seen` and `ever_seen_completed` on every block of
/// an East Indies window, both directions, as `(block, who, o, field, ours,
/// theirs)`. The shared instrument leaves these two bytes uncompared
/// (`coverage::UNCOMPARED_BY_THE_INSTRUMENT`), and they are what first
/// contact hangs off (`docs/VISION.md` §6.4, item 1446); a building on one
/// side only is a `presence` row.
fn east_indies_ever_seen(
    capture: &str,
    window: (i64, i64),
) -> Option<std::collections::BTreeSet<SeenRow>> {
    let inst = crate::testenv::install()?;
    let (start, base_path, path) = (dump(EAST_START)?, dump(EAST_LONG.0)?, dump(capture)?);
    let loaded = crate::load::load(&inst).unwrap();
    let (start_text, base_text) = (crate::capture::read(start), crate::capture::read(base_path));
    let (start_log, base_log) = (Log::parse(&start_text), Log::parse(&base_text));
    let sibling = start_log.initial().unwrap();
    let mut init = base_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    if let Some(t) = trace(EAST_LONG.1) {
        borrow_pasture(&mut init, &t);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let mut rows = std::collections::BTreeSet::new();
    let mut blocks = 0;
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
        let frame = ix.frame_state(at).unwrap();
        let sim = &built.sim;
        let mut linked = std::collections::BTreeSet::new();
        for r in &frame.builds {
            let Some(b) =
                harness::link_building(sim, r.who, r.o).filter(|&b| sim.buildings[b].alive)
            else {
                rows.insert((n, r.who, r.o, "presence", 0, 1));
                continue;
            };
            linked.insert(b);
            let x = &sim.buildings[b];
            for (field, ours, theirs) in [
                ("ever_seen", x.ever_seen, r.ever_seen),
                (
                    "ever_seen_completed",
                    x.ever_seen_completed,
                    r.ever_seen_completed,
                ),
            ] {
                let theirs = theirs.expect("BUILDS prints both bytes");
                if i64::from(ours) != theirs {
                    rows.insert((n, r.who, r.o, field, i64::from(ours), theirs));
                }
            }
        }
        for (b, x) in sim.buildings.iter().enumerate() {
            if x.alive && x.owner < 8 && !linked.contains(&b) {
                rows.insert((n, i64::from(x.owner), i64::from(x.index), "presence", 1, 0));
            }
        }
        blocks += 1;
    }
    assert_eq!(
        blocks,
        window.1 - window.0 + 1,
        "{capture}: every window block"
    );
    Some(rows)
}

/// **First contact, dated by the bytes it hangs off** (item 1446,
/// `docs/VISION.md` §6.4). run611 is the first capture of this game past
/// it: on its block 8030 the original's French Senate `1/2021` and the
/// wonder `1/2022` beside it already read `ever_seen` 255 — every bit, which
/// only a wonder's `0xff` write into the *current* plane can give — and both
/// leaders' met bits are set. Until item 1446 this crate had neither: a
/// wonder's start lit nothing, so the two read 2 and the two leaders never
/// met, and the AI priced its purchases without its war on 8180. Every
/// building of all three East Indies windows now agrees in both bytes.
#[test]
fn east_indies_wonder_start_is_first_contact_on_every_building() {
    for (capture, window) in [
        (
            "gamelog-run613-islands-french-contact-7946.txt",
            WIDENING_FRENCH_CONTACT_7946,
        ),
        (
            "gamelog-run611-islands-french-contact.txt",
            WIDENING_FRENCH_CONTACT,
        ),
        (
            "gamelog-run610-islands-french-toughest-8182.txt",
            WIDENING_FRENCH_EAST_INDIES_8182,
        ),
        (
            "gamelog-run612-islands-french-8236.txt",
            WIDENING_FRENCH_EAST_INDIES,
        ),
    ] {
        let Some(rows) = east_indies_ever_seen(capture, window) else {
            continue;
        };
        eprintln!("{capture}: ever_seen rows {rows:?}");
        assert_eq!(
            rows,
            std::collections::BTreeSet::new(),
            "{capture}: every building's ever_seen bytes agree, both directions"
        );
    }
}
