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

    /// **The rules track's headline has the guard the AI track's has** —
    /// the handoff's `Golden:` line against the pinned chapters.
    /// `docs_guard::the_handoff_carries_the_golden_line` only checks the
    /// line names *a* word; on 2026-09-19 it read `w624` over a pin of 621
    /// for a whole item (parked 406, the sixth pass). Same shape as the
    /// two above. **The constant is the worker's to re-pin; the line is the
    /// commander's** — a worker that lands with this red has done its half.
    ///
    /// With two chapters pinned the line **composes lowest chapter first**
    /// (parked 417, ruled by the seventh pass; `docs/GOLDEN.md` §1's own
    /// recommendation and the AI track's lower-map rule one level across):
    /// the first `w<frame>` is the lowest pinned word, and every pinned
    /// chapter's word appears on the line.
    #[test]
    fn the_handoff_s_golden_line_is_the_pinned_word() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let line = q.lines().find(|l| l.starts_with("Golden:")).expect(
            "docs/QUEUE.md has no `Golden:` line in the handoff; write \
             `Golden: w<frame> of <length> (chN) · …`",
        );
        let words: Vec<i64> = line
            .trim_start_matches("Golden:")
            .split_whitespace()
            .filter_map(|w| w.strip_prefix('w').and_then(|d| d.parse::<i64>().ok()))
            .collect();
        let chapters = [GOLDEN_WORD_CHAPTER_ONE, GOLDEN_WORD_CHAPTER_TWO];
        let lowest = *chapters.iter().min().unwrap();
        let said = *words
            .first()
            .unwrap_or_else(|| panic!("the `Golden:` line names no `w<frame>`: {line:?}"));
        assert_eq!(
            said, lowest,
            "the handoff's `Golden:` line leads with w{said}; the lowest pinned chapter \
             is {lowest} (chapters {chapters:?}). The constant and its comment are the \
             worker's to re-pin; the queue's line is the commander's to write"
        );
        for c in chapters {
            assert!(
                words.contains(&c),
                "the `Golden:` line names no w{c}; every pinned chapter's word is on it: \
                 {line:?}"
            );
        }
    }

    /// **A word is pinned with its widening** (`docs/DECISIONS.md` 43):
    /// [`WIDENINGS`] names, for every pinned word, the test that compared
    /// every dumped record on the word's own frame — or the open item that
    /// owes it. A named test must exist as a `fn` under `src/diff/`; an
    /// owed one must be an item the queue books or the parked file holds.
    /// Made to fail first on a misspelt test name and on an item number
    /// nothing carries.
    ///
    /// **And the word sits inside the test's own window** (parked 449,
    /// the eighth pass): a named test declares the block window it walks
    /// and the word must be strictly inside it, because a widening whose
    /// window the word has walked out of passes by saying nothing — which
    /// is how run100's 9382 test read as Great Lakes' widening for 651
    /// frames. Made to fail first with chapter two's window moved to
    /// `(630, 640)` over a word of 624.
    #[test]
    fn the_widening_behind_each_pinned_word_exists() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff");
        let mut source = String::new();
        for entry in std::fs::read_dir(root).expect("src/diff") {
            let path = entry.expect("entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                source.push_str(&std::fs::read_to_string(&path).expect("read"));
            }
        }
        let docs = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs");
        let queue = std::fs::read_to_string(format!("{docs}/QUEUE.md")).expect("QUEUE.md");
        let parked = std::fs::read_to_string(format!("{docs}/PARKED.md")).expect("PARKED.md");
        for (name, word, test, item, window) in WIDENINGS {
            match test {
                Some(t) => {
                    assert!(
                        source.contains(&format!("fn {t}(")),
                        "{name} = {word} names widening test `{t}`, and no `fn {t}(` exists \
                         under crates/rondata/src/diff/"
                    );
                    let (lo, hi) = window.unwrap_or_else(|| {
                        panic!("{name} = {word} names widening test `{t}` and no window; declare the block window the test walks")
                    });
                    assert!(
                        lo < *word && *word < hi,
                        "{name} = {word} is not inside its widening's window [{lo}, {hi}) — \
                         `{t}` widens a frame the word has left. Move the window with the \
                         word (item 456's lesson) rather than leaving the row reading as pinned"
                    );
                }
                None => assert!(
                    queue.contains(&format!("\n{item}. "))
                        || parked.contains(&format!("({item}) **")),
                    "{name} = {word} has no widening on file and names item {item}, which \
                     neither docs/QUEUE.md books (`{item}. `) nor docs/PARKED.md holds \
                     (`({item}) **`). Widen the word's frame whole, or book it"
                ),
            }
        }
    }
}
