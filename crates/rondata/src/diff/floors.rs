//! The scored floors, and the handoff's `Scoreboard:` line against them.

/// One map's headline floors: ticks and orders before divergence on its
/// score run, and its word — the frame its draw stream parts — beside
/// them. The pair is the score and the word is the instrument
/// (`docs/DECISIONS.md` 26). The scoring tests assert against these
/// fields rather than their own literals, and the queue's handoff states
/// the same six numbers on a `Scoreboard:` line that
/// `the_handoff_s_scoreboard_is_the_floors` parses — so a floor that
/// moves without the handoff, or a handoff written off a run that is not
/// the score run (item 69: two numbers describing two different
/// simulations shared one file for a week), fails somewhere instead of
/// waiting for a steering pass to notice.
pub struct MapFloors {
    pub map: &'static str,
    pub ticks: i64,
    pub orders: i64,
    pub word: i64,
}

/// East Indies first — the lower pair leads and is the headline.
pub const FLOORS: [MapFloors; 2] = [
    MapFloors {
        map: "EastIndies",
        ticks: 1851,
        orders: 1850,
        word: 1850,
    },
    MapFloors {
        map: "GreatLakes",
        ticks: 1772,
        orders: 1772,
        word: 1850,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;

    fn permitted_test_width(threads: usize, cap: Option<&str>) -> bool {
        (1..=2).contains(&threads)
            || (3..=4).contains(&threads)
                && cap
                    .and_then(|value| value.parse::<u32>().ok())
                    .is_some_and(|gib| (1..=20).contains(&gib))
    }

    #[test]
    fn wider_test_width_requires_a_bounded_monitor_contract() {
        for threads in [1, 2] {
            assert!(permitted_test_width(threads, None));
        }
        for threads in [3, 4] {
            for cap in [None, Some(""), Some("0"), Some("21"), Some("invalid")] {
                assert!(!permitted_test_width(threads, cap));
            }
            for cap in ["1", "20"] {
                assert!(permitted_test_width(threads, Some(cap)));
            }
        }
        for threads in [0, 5, 16, usize::MAX] {
            assert!(!permitted_test_width(threads, Some("20")));
        }
    }

    /// Unmonitored runs retain the conservative two-thread policy from
    /// DECISIONS 34. Three/four threads require memcap's child-only marker
    /// for a positive cap no greater than 20 GiB; the monitor owns that
    /// environment contract, not a cryptographic attestation. Measurements:
    /// docs/lab/2026-09-10-test-concurrency.md.
    #[test]
    fn the_gate_has_a_bounded_test_width() {
        if crate::testenv::install().is_none() {
            return;
        }
        let env = std::env::var("RUST_TEST_THREADS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok());
        let args: Vec<String> = std::env::args().collect();
        let flag = args.iter().enumerate().find_map(|(i, a)| {
            a.strip_prefix("--test-threads=")
                .map(str::to_string)
                .or_else(|| {
                    (a == "--test-threads")
                        .then(|| args.get(i + 1).cloned())
                        .flatten()
                })
                .and_then(|n| n.parse::<usize>().ok())
        });
        let threads = match (env, flag) {
            (_, Some(f)) => f,
            (Some(e), None) => e,
            (None, None) => panic!(
                "the diff suite is running unpinned: neither RUST_TEST_THREADS \
                 (.cargo/config.toml) nor --test-threads is set, and at this \
                 machine's default it reaches 33 GiB and is killed by memcap"
            ),
        };
        let cap = std::env::var("RON_TEST_MEMCAP_GIB").ok();
        assert!(
            permitted_test_width(threads, cap.as_deref()),
            "the diff suite is running at {threads} test threads; use at most two \
             directly, or at most four under tools/memcap.sh with a cap <= 20 GiB"
        );
    }

    #[test]
    fn the_handoff_s_scoreboard_is_the_floors() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let line = q.lines().find(|l| l.starts_with("Scoreboard:")).expect(
            "docs/QUEUE.md has no `Scoreboard:` line in the handoff; write \
             `Scoreboard: <map> <ticks>/<orders> w<word> · <map> …`, one part \
             per row of rondata::diff::FLOORS, in order",
        );
        let mut stated = Vec::new();
        for part in line.trim_start_matches("Scoreboard:").split('·') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let (ticks, orders) = t
                .get(1)
                .and_then(|p| p.split_once('/'))
                .unwrap_or_else(|| panic!("unreadable scoreboard part {part:?}"));
            let word = t
                .get(2)
                .and_then(|w| w.strip_prefix('w'))
                .unwrap_or_else(|| panic!("unreadable scoreboard part {part:?}"));
            stated.push((
                t[0].to_string(),
                ticks.parse::<i64>().expect("ticks"),
                orders.parse::<i64>().expect("orders"),
                word.parse::<i64>().expect("word"),
            ));
        }
        let pinned: Vec<_> = FLOORS
            .iter()
            .map(|f| (f.map.to_string(), f.ticks, f.orders, f.word))
            .collect();
        assert_eq!(
            stated, pinned,
            "the handoff's scoreboard is not the pinned floors: left is the \
             queue's line, right is rondata::diff::FLOORS"
        );

        // **And the line below it, which is the headline.** East Indies'
        // scored capture is closed, so the number a session is judged by
        // lives on the `Long captures:` line — and nothing checked it. Same
        // shape, one word a map, against the two long tests' own floors.
        let long = q.lines().find(|l| l.starts_with("Long captures:")).expect(
            "docs/QUEUE.md has no `Long captures:` line in the handoff; write \
             `Long captures: <map> w<word> of <length> ...`, one part per row \
             of rondata::diff::FLOORS, in order",
        );
        let mut said = Vec::new();
        for part in long.trim_start_matches("Long captures:").split('\u{b7}') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let word = t
                .get(1)
                .and_then(|w| w.strip_prefix('w'))
                .unwrap_or_else(|| panic!("unreadable long-capture part {part:?}"));
            said.push((
                t[0].to_string(),
                word.parse::<i64>().expect("the long capture's word"),
            ));
        }
        assert_eq!(
            said,
            vec![
                ("EastIndies".to_string(), LONG_WORD_EAST_INDIES),
                ("GreatLakes".to_string(), LONG_WORD_GREAT_LAKES),
            ],
            "the handoff's long-capture words are not the long tests' floors: \
             left is the queue's line, right is LONG_WORD_EAST_INDIES and \
             LONG_WORD_GREAT_LAKES"
        );
    }
}
