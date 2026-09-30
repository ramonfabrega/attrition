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

/// East Indies first — the lower pair leads and is the headline. Great
/// Sahara is the third scored map (DECISIONS 54 §3, item 1066), in the first
/// pair's lobby with the map moved; `diff::third` scores it.
pub const FLOORS: [MapFloors; 3] = [
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
    MapFloors {
        map: "GreatSahara",
        ticks: 1850,
        orders: 1850,
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

    /// **The second pair's line is its pinned words** (DECISIONS 53 §2,
    /// item 971). The first pair closed at 24,000 on both maps and its
    /// `Long captures:` line stays as the closed floor; the AI track's word
    /// is now the second pair's — the same two games at the lobby's top
    /// difficulty — and the handoff carries it on a `Second pair:` line of
    /// the same shape, `<map> w<word> of <length>`, East Indies first. **The
    /// constants are the worker's to re-pin; the line is the commander's**
    /// (`tools/release_gate.py`'s `COMMANDERS_LINES` names this test).
    /// Made to fail first on the queue with no such line, and on a line
    /// whose word is not the pin ([`ai_line_verdict`]'s own test). **The
    /// length is read too** (the nineteenth pass): the rows are
    /// [`AI_WORDS`]', and a word of its game's length is a closed game.
    #[test]
    fn the_handoff_s_second_pair_is_the_pinned_words() {
        the_handoff_carries("Second pair");
    }

    /// **The third map's line is its pinned word** (parked 1080, the
    /// nineteenth pass): item 1066 pinned `LONG_WORD_GREAT_SAHARA` and the
    /// commander wrote a `Third map:` line that no guard read. Made to
    /// fail first on the line with its word moved by one.
    #[test]
    fn the_handoff_s_third_map_is_the_pinned_word() {
        the_handoff_carries("Third map");
    }

    fn the_handoff_carries(prefix: &str) {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let head = format!("{prefix}:");
        let line = q.lines().find(|l| l.starts_with(&head)).unwrap_or_else(|| {
            panic!(
                "docs/QUEUE.md has no `{head}` line in the handoff; write \
                 `{head} <map> w<word> of <length>`, one part per row of \
                 `AI_WORDS` on that line, in order, parted by a middle dot"
            )
        });
        let rows: Vec<(&str, i64, i64)> = AI_WORDS
            .iter()
            .filter(|w| w.line == prefix)
            .map(|w| (w.map, w.word, w.length))
            .collect();
        if let Err(e) = ai_line_verdict(line, prefix, &rows) {
            panic!("{e}");
        }
    }

    /// One of the AI track's lines read part by part against its rows:
    /// `<map> w<word> of <length>`, the length with or without its comma.
    fn ai_line_verdict(line: &str, prefix: &str, rows: &[(&str, i64, i64)]) -> Result<(), String> {
        let mut said = Vec::new();
        let body = line
            .strip_prefix(prefix)
            .and_then(|l| l.strip_prefix(':'))
            .ok_or_else(|| format!("not a `{prefix}:` line: {line:?}"))?;
        for part in body.split('\u{b7}') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let word = t
                .get(1)
                .and_then(|w| w.strip_prefix('w'))
                .and_then(|w| w.parse::<i64>().ok())
                .ok_or_else(|| format!("unreadable part {part:?} on the `{prefix}:` line"))?;
            let length = (t.get(2) == Some(&"of"))
                .then(|| t.get(3))
                .flatten()
                .and_then(|n| n.replace(',', "").parse::<i64>().ok())
                .ok_or_else(|| {
                    format!("part {part:?} on the `{prefix}:` line names no `of <length>`")
                })?;
            said.push((t[0].to_string(), word, length));
        }
        let pinned: Vec<(String, i64, i64)> = rows
            .iter()
            .map(|(m, w, l)| (m.to_string(), *w, *l))
            .collect();
        if said == pinned {
            Ok(())
        } else {
            Err(format!(
                "the handoff's `{prefix}:` line is not the pins: the queue's \
                 line says {said:?}, `AI_WORDS` says {pinned:?}"
            ))
        }
    }

    #[test]
    fn an_ai_line_is_read_against_its_rows() {
        let rows = [("EastIndies", 612, 18_140), ("GreatLakes", 700, 5_930)];
        let ok = "Second pair: EastIndies w612 of 18,140 \u{b7} GreatLakes w700 of 5,930";
        assert_eq!(ai_line_verdict(ok, "Second pair", &rows), Ok(()));
        let stale = [("EastIndies", 613, 18_140), ("GreatLakes", 700, 5_930)];
        assert!(
            ai_line_verdict(ok, "Second pair", &stale).is_err(),
            "a stale East Indies word"
        );
        let stale = [("EastIndies", 612, 18_140), ("GreatLakes", 699, 5_930)];
        assert!(
            ai_line_verdict(ok, "Second pair", &stale).is_err(),
            "a stale Great Lakes word"
        );
        // The length: entry 53 wrote 24,000 for a game that ends on 5,930.
        let long = "Second pair: EastIndies w612 of 18,140 \u{b7} GreatLakes w700 of 24,000";
        assert!(
            ai_line_verdict(long, "Second pair", &rows).is_err(),
            "a length that is not the game's"
        );
        let bare = "Second pair: EastIndies w612 \u{b7} GreatLakes w700";
        assert!(
            ai_line_verdict(bare, "Second pair", &rows).is_err(),
            "no length"
        );
        let swapped = "Second pair: GreatLakes w700 of 5,930 \u{b7} EastIndies w612 of 18,140";
        assert!(
            ai_line_verdict(swapped, "Second pair", &rows).is_err(),
            "East Indies first"
        );
        assert!(ai_line_verdict("Second pair: none pinned", "Second pair", &rows).is_err());
        let third = "Third map: GreatSahara w8 of 24,000";
        assert_eq!(
            ai_line_verdict(third, "Third map", &[("GreatSahara", 8, 24_000)]),
            Ok(())
        );
        assert!(ai_line_verdict(third, "Third map", &[("GreatSahara", 9, 24_000)]).is_err());
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
    ///
    /// **A closed chapter is named closed, and the line leads with the
    /// lowest *open* word** (parked 528, the eleventh pass). A chapter
    /// closes when its word is its trace's last block — 900 in a window
    /// `[606, 901)` — and by 2026-09-23 three of four were closed, so
    /// "lowest word" read a closed chapter's 900 ahead of the live
    /// headline's 1277 and the rules track's item stood last on its own
    /// line. So the line is parsed part by part: a closed chapter is
    /// `chN closed` and never a `w`; an open one is `chN w<word> of
    /// <length>`; the first `w` is the lowest open word, which is the
    /// rules headline. A chapter that reopens — a pin under its end —
    /// fails here until the line says so as a word.
    ///
    /// **Every pinned golden word is on the list, not only `ch1`–`ch8`**
    /// (parked 638, the thirteenth pass). The first version parsed `chN`
    /// parts alone and skipped the restage and seven-b, so it passed with
    /// the queue at `restage w792` over a constant of 865 — the two words
    /// the rules lane was actually on were the two it could not see. The
    /// list below is every `GOLDEN_WORD_*` with a widening; a new one is
    /// added here in the landing that pins it. And when nothing is open
    /// the line **leads with `every chapter closed`** rather than
    /// borrowing `none pinned`, which means the opposite.
    #[test]
    fn the_handoff_s_golden_line_is_the_pinned_word() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let line = q.lines().find(|l| l.starts_with("Golden:")).expect(
            "docs/QUEUE.md has no `Golden:` line in the handoff; write \
             `Golden: chN closed · chM w<frame> of <length> · …`",
        );
        if let Err(why) = golden_line_verdict(line, &GOLDEN_WORDS) {
            panic!("{why}");
        }
    }

    /// **Every `GOLDEN_WORD_*` pinned in `testkit.rs` is on `GOLDEN_WORDS`**
    /// (parked 846, the sixteenth pass). The rule above was prose: 836
    /// pinned chapter twenty-two's word and widening and not this row, and
    /// its gate passed with the chapter open and the line reading `every
    /// chapter closed`; the commander added the row in the booking. Made to
    /// fail first by removing `ch22`.
    #[test]
    fn every_pinned_golden_word_is_on_the_line_s_list() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff/");
        let testkit = std::fs::read_to_string(format!("{dir}testkit.rs")).expect("testkit.rs");
        let floors = std::fs::read_to_string(format!("{dir}floors.rs")).expect("floors.rs");
        let tokens = |text: &str| -> Vec<String> {
            text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|t| t.starts_with("GOLDEN_WORD_"))
                .map(str::to_string)
                .collect()
        };
        let pinned: std::collections::BTreeSet<String> = testkit
            .lines()
            .filter(|l| l.contains("const GOLDEN_WORD_"))
            .flat_map(&tokens)
            .collect();
        // The array's own declaration, not this test's mention of it.
        let needle = format!("const {}: [", "GOLDEN_WORDS");
        let list = floors
            .split_once(needle.as_str())
            .and_then(|(_, rest)| rest.split_once("\n    ];"))
            .map(|(block, _)| block)
            .expect("floors.rs holds the GOLDEN_WORDS array");
        let listed: std::collections::BTreeSet<String> = tokens(list).into_iter().collect();
        let missing: Vec<&String> = pinned.difference(&listed).collect();
        assert!(
            missing.is_empty(),
            "pinned in testkit.rs and not on floors.rs's GOLDEN_WORDS, so the `Golden:` \
             line guard cannot see them — add the row in the landing that pins the word: \
             {missing:?}"
        );
        let stale: Vec<&String> = listed.difference(&pinned).collect();
        assert!(
            stale.is_empty(),
            "on GOLDEN_WORDS and pinned nowhere: {stale:?}"
        );
    }

    /// `(name on the line, word, widening window)` — closed when the word
    /// is the window's last block.
    const GOLDEN_WORDS: [(&str, i64, (i64, i64)); 48] = [
        ("ch1", GOLDEN_WORD_CHAPTER_ONE, WIDENING_CHAPTER_ONE),
        ("ch2", GOLDEN_WORD_CHAPTER_TWO, WIDENING_CHAPTER_TWO),
        ("ch3", GOLDEN_WORD_CHAPTER_THREE, WIDENING_CHAPTER_THREE),
        ("ch4", GOLDEN_WORD_CHAPTER_FOUR, WIDENING_CHAPTER_FOUR),
        ("ch5", GOLDEN_WORD_CHAPTER_FIVE, WIDENING_CHAPTER_FIVE),
        ("ch6", GOLDEN_WORD_CHAPTER_SIX, WIDENING_CHAPTER_SIX),
        ("ch6b", GOLDEN_WORD_CHAPTER_SIX_B, WIDENING_CHAPTER_SIX_B),
        ("ch7", GOLDEN_WORD_CHAPTER_SEVEN, WIDENING_CHAPTER_SEVEN),
        ("ch8", GOLDEN_WORD_CHAPTER_EIGHT, WIDENING_CHAPTER_EIGHT),
        ("ch9", GOLDEN_WORD_CHAPTER_NINE, WIDENING_CHAPTER_NINE),
        ("ch10", GOLDEN_WORD_CHAPTER_TEN, WIDENING_CHAPTER_TEN),
        ("ch11", GOLDEN_WORD_CHAPTER_ELEVEN, WIDENING_CHAPTER_ELEVEN),
        ("ch12", GOLDEN_WORD_CHAPTER_TWELVE, WIDENING_CHAPTER_TWELVE),
        (
            "ch13",
            GOLDEN_WORD_CHAPTER_THIRTEEN,
            WIDENING_CHAPTER_THIRTEEN,
        ),
        (
            "ch14",
            GOLDEN_WORD_CHAPTER_FOURTEEN,
            WIDENING_CHAPTER_FOURTEEN,
        ),
        (
            "ch15",
            GOLDEN_WORD_CHAPTER_FIFTEEN,
            WIDENING_CHAPTER_FIFTEEN,
        ),
        (
            "ch16",
            GOLDEN_WORD_CHAPTER_SIXTEEN,
            WIDENING_CHAPTER_SIXTEEN,
        ),
        (
            "ch17",
            GOLDEN_WORD_CHAPTER_SEVENTEEN,
            WIDENING_CHAPTER_SEVENTEEN,
        ),
        (
            "ch18",
            GOLDEN_WORD_CHAPTER_EIGHTEEN,
            WIDENING_CHAPTER_EIGHTEEN,
        ),
        (
            "ch19",
            GOLDEN_WORD_CHAPTER_NINETEEN,
            WIDENING_CHAPTER_NINETEEN,
        ),
        ("ch20", GOLDEN_WORD_CHAPTER_TWENTY, WIDENING_CHAPTER_TWENTY),
        (
            "ch21",
            GOLDEN_WORD_CHAPTER_TWENTY_ONE,
            WIDENING_CHAPTER_TWENTY_ONE,
        ),
        (
            "ch22",
            GOLDEN_WORD_CHAPTER_TWENTY_TWO,
            WIDENING_CHAPTER_TWENTY_TWO,
        ),
        (
            "ch23",
            GOLDEN_WORD_CHAPTER_TWENTY_THREE,
            WIDENING_CHAPTER_TWENTY_THREE,
        ),
        (
            "ch24",
            GOLDEN_WORD_CHAPTER_TWENTY_FOUR,
            WIDENING_CHAPTER_TWENTY_FOUR,
        ),
        (
            "ch25",
            GOLDEN_WORD_CHAPTER_TWENTY_FIVE,
            WIDENING_CHAPTER_TWENTY_FIVE,
        ),
        (
            "ch26",
            GOLDEN_WORD_CHAPTER_TWENTY_SIX,
            WIDENING_CHAPTER_TWENTY_SIX,
        ),
        (
            "ch27",
            GOLDEN_WORD_CHAPTER_TWENTY_SEVEN,
            WIDENING_CHAPTER_TWENTY_SEVEN,
        ),
        (
            "ch28",
            GOLDEN_WORD_CHAPTER_TWENTY_EIGHT,
            WIDENING_CHAPTER_TWENTY_EIGHT,
        ),
        (
            "ch29",
            GOLDEN_WORD_CHAPTER_TWENTY_NINE,
            WIDENING_CHAPTER_TWENTY_NINE,
        ),
        ("ch30", GOLDEN_WORD_CHAPTER_THIRTY, WIDENING_CHAPTER_THIRTY),
        (
            "ch31",
            GOLDEN_WORD_CHAPTER_THIRTY_ONE,
            WIDENING_CHAPTER_THIRTY_ONE,
        ),
        (
            "ch32",
            GOLDEN_WORD_CHAPTER_THIRTY_TWO,
            WIDENING_CHAPTER_THIRTY_TWO,
        ),
        (
            "ch33",
            GOLDEN_WORD_CHAPTER_THIRTY_THREE,
            WIDENING_CHAPTER_THIRTY_THREE,
        ),
        (
            "ch34",
            GOLDEN_WORD_CHAPTER_THIRTY_FOUR,
            WIDENING_CHAPTER_THIRTY_FOUR,
        ),
        (
            "ch35",
            GOLDEN_WORD_CHAPTER_THIRTY_FIVE,
            WIDENING_CHAPTER_THIRTY_FIVE,
        ),
        (
            "ch36",
            GOLDEN_WORD_CHAPTER_THIRTY_SIX,
            WIDENING_CHAPTER_THIRTY_SIX,
        ),
        (
            "ch37",
            GOLDEN_WORD_CHAPTER_THIRTY_SEVEN,
            WIDENING_CHAPTER_THIRTY_SEVEN,
        ),
        (
            "ch38",
            GOLDEN_WORD_CHAPTER_THIRTY_EIGHT,
            WIDENING_CHAPTER_THIRTY_EIGHT,
        ),
        (
            "ch39",
            GOLDEN_WORD_CHAPTER_THIRTY_NINE,
            WIDENING_CHAPTER_THIRTY_NINE,
        ),
        ("ch40", GOLDEN_WORD_CHAPTER_FORTY, WIDENING_CHAPTER_FORTY),
        (
            "ch41",
            GOLDEN_WORD_CHAPTER_FORTY_ONE,
            WIDENING_CHAPTER_FORTY_ONE,
        ),
        (
            "ch42",
            GOLDEN_WORD_CHAPTER_FORTY_TWO,
            WIDENING_CHAPTER_FORTY_TWO,
        ),
        (
            "ch43",
            GOLDEN_WORD_CHAPTER_FORTY_THREE,
            WIDENING_CHAPTER_FORTY_THREE,
        ),
        (
            "ch44",
            GOLDEN_WORD_CHAPTER_FORTY_FOUR,
            WIDENING_CHAPTER_FORTY_FOUR,
        ),
        (
            "restage",
            GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
            WIDENING_CHAPTER_THREE_RESTAGE,
        ),
        (
            "ch7b",
            GOLDEN_WORD_CHAPTER_SEVEN_B,
            WIDENING_CHAPTER_SEVEN_B,
        ),
        (
            "ch7b-control",
            GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
            WIDENING_CHAPTER_SEVEN_B_CONTROL,
        ),
    ];

    /// The `Golden:` line against the pinned words, as a value so the
    /// fixtures below can fail it on purpose. A part is `<name> closed`
    /// or `<name> w<word> of <length>`; a part whose first token is not a
    /// pinned name — `every chapter closed`, `651 next` — is skipped.
    fn golden_line_verdict(line: &str, words: &[(&str, i64, (i64, i64))]) -> Result<(), String> {
        let mut said_closed: Vec<&str> = Vec::new();
        let mut said_open: Vec<(&str, i64)> = Vec::new();
        for part in line.trim_start_matches("Golden:").split('\u{b7}') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let Some(name) = t.first().copied() else {
                continue;
            };
            let Some((name, _, _)) = words.iter().find(|(n, _, _)| *n == name) else {
                continue;
            };
            match t.get(1) {
                Some(&"closed") => said_closed.push(name),
                Some(w) => {
                    let word = w
                        .strip_prefix('w')
                        .and_then(|d| d.parse::<i64>().ok())
                        .ok_or_else(|| format!("unreadable golden part {part:?}"))?;
                    said_open.push((name, word));
                }
                None => return Err(format!("unreadable golden part {part:?}")),
            }
        }
        let mut open_words: Vec<i64> = Vec::new();
        for (name, w, (_, hi)) in words {
            if *w == hi - 1 {
                if !said_closed.contains(name) {
                    return Err(format!(
                        "{name} is closed (its word {w} is its trace's end) and the \
                         `Golden:` line does not say `{name} closed`: {line:?}"
                    ));
                }
                if said_open.iter().any(|(c, _)| c == name) {
                    return Err(format!(
                        "{name} is closed and the `Golden:` line still carries it as a \
                         word: {line:?}"
                    ));
                }
            } else {
                if !said_open.contains(&(name, *w)) {
                    return Err(format!(
                        "the `Golden:` line does not carry `{name} w{w}`; every open \
                         chapter's word is on it: {line:?}. The constant and its comment \
                         are the worker's to re-pin; the queue's line is the commander's \
                         to write"
                    ));
                }
                if said_closed.contains(name) {
                    return Err(format!(
                        "{name} is open at {w} and the `Golden:` line calls it closed: \
                         {line:?}"
                    ));
                }
                open_words.push(*w);
            }
        }
        let rest = line.trim_start_matches("Golden:").trim();
        match open_words.iter().min() {
            Some(lowest) => {
                let first = said_open
                    .first()
                    .map(|(_, w)| *w)
                    .ok_or_else(|| format!("the `Golden:` line names no open word: {line:?}"))?;
                if first != *lowest {
                    return Err(format!(
                        "the `Golden:` line's first word is w{first}; the lowest open \
                         chapter is {lowest}, and that is the rules headline"
                    ));
                }
            }
            None => {
                if !rest.starts_with("every chapter closed") {
                    return Err(format!(
                        "no golden word is open, and the `Golden:` line does not lead \
                         with `every chapter closed`: {line:?}"
                    ));
                }
            }
        }
        Ok(())
    }

    /// The verdict, made to fail first on the lines that fooled the old
    /// guard: the restage carried as a word under a closed constant, a
    /// seven-b part missing altogether, and an all-closed line that
    /// borrows `none pinned`.
    #[test]
    fn the_golden_line_guard_reads_the_restage_and_seven_b() {
        let words: [(&str, i64, (i64, i64)); 3] = [
            ("ch1", 900, (605, 901)),
            ("restage", 1000, (605, 1001)),
            ("ch7b", 1148, (605, 1201)),
        ];
        let sep = " \u{b7} ";
        let good =
            format!("Golden: ch7b w1148 of 1200{sep}ch1 closed{sep}restage closed{sep}651 next");
        assert_eq!(golden_line_verdict(&good, &words), Ok(()));
        // 638's case: the restage is closed at 1000 and the line still
        // says `restage w792` — the old guard skipped the part.
        let stale = format!("Golden: ch7b w1148 of 1200{sep}ch1 closed{sep}restage w792 of 1000");
        assert!(golden_line_verdict(&stale, &words).is_err(), "{stale}");
        // A pinned word missing from the line altogether.
        let missing = format!("Golden: ch1 closed{sep}restage closed");
        assert!(golden_line_verdict(&missing, &words).is_err(), "{missing}");
        // Everything closed: `none pinned` is the wrong phrase for it.
        let all: [(&str, i64, (i64, i64)); 2] =
            [("ch1", 900, (605, 901)), ("restage", 1000, (605, 1001))];
        let borrowed =
            format!("Golden: none pinned, every chapter closed{sep}ch1 closed{sep}restage closed");
        assert!(golden_line_verdict(&borrowed, &all).is_err(), "{borrowed}");
        let led =
            format!("Golden: every chapter closed{sep}ch1 closed{sep}restage closed{sep}651 next");
        assert_eq!(golden_line_verdict(&led, &all), Ok(()));
    }

    /// **The AI track's default map is the lower word's** (`docs/DECISIONS.md`
    /// 41 §1, made a guard by the eleventh pass). The queue's own preamble
    /// names the map — "lower map first — Great Lakes" — and a commander
    /// reads that line, never the constants: Great Lakes passed East
    /// Indies' 9,711 on 2026-09-21 and the line still named it through
    /// three steering passes and some forty landings, while parked 444
    /// said what the day should have brought. The phrase and the map stay
    /// on one line, so this can read them.
    #[test]
    fn the_handoff_s_default_map_is_the_lower_word() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let phrase = "lower map first \u{2014}";
        // The maintenance section quotes the template (`<map>`); the
        // preamble carries the map, on one line — its first gate run found
        // the phrase wrapped and read the template instead.
        let line = q
            .lines()
            .find(|l| l.contains(phrase) && !l.contains('<'))
            .expect("docs/QUEUE.md names no `lower map first — <map>` line, unwrapped");
        let named = line.split(phrase).nth(1).unwrap().trim_start();
        // **The newest pair's lower word** (DECISIONS 53 §2, item 971): the
        // first pair closed at 24,000 on both maps, so its words no longer
        // choose a map; the second pair's do.
        //
        // **And a closed game chooses nothing** (parked 1121, the
        // nineteenth pass): Great Lakes' 5,930 is its game's end, and when
        // East Indies passed it on 5975 this guard asked the queue to name
        // the map no item can move. The lower word is the lower *open* one.
        let pair: Vec<(&str, i64, i64)> = AI_WORDS
            .iter()
            .filter(|w| w.line == NEWEST_PAIR)
            .map(|w| (w.named, w.word, w.length))
            .collect();
        let lower = default_map(&pair);
        assert!(
            named.starts_with(lower),
            "docs/QUEUE.md says `lower map first — {named}` and the newest pair's lower \
             open word is {lower}'s ({pair:?}, as map, word, length). \
             Rewrite the line, and book that map's widening first — WIDENINGS names the \
             item that owes it"
        );
    }

    /// The lower open word's map, the first row winning a tie; `closed`
    /// when every game of the pair has reached its end, which is the day
    /// the next pair is owed (`docs/DECISIONS.md` 53 §2).
    fn default_map<'a>(pair: &[(&'a str, i64, i64)]) -> &'a str {
        pair.iter()
            .filter(|(_, word, length)| word < length)
            .min_by_key(|(_, word, _)| *word)
            .map_or("closed", |(map, _, _)| *map)
    }

    #[test]
    fn the_default_map_is_the_lower_open_word() {
        let open = [
            ("East Indies", 5_606, 18_140),
            ("Great Lakes", 4_924, 5_930),
        ];
        assert_eq!(default_map(&open), "Great Lakes");
        let tie = [("East Indies", 10, 18_140), ("Great Lakes", 10, 5_930)];
        assert_eq!(default_map(&tie), "East Indies");
        // 1121's day: the lower word is a closed game's.
        let passed = [
            ("East Indies", 5_975, 18_140),
            ("Great Lakes", 5_930, 5_930),
        ];
        assert_eq!(default_map(&passed), "East Indies");
        let done = [
            ("East Indies", 18_140, 18_140),
            ("Great Lakes", 5_930, 5_930),
        ];
        assert_eq!(default_map(&done), "closed");
    }

    /// **A closed game names the test that scored its end** (parked 1108,
    /// the nineteenth pass): the city's capture spends no draw, and
    /// run347's last event was wrong under a stream that agreed to the
    /// game's last frame; a closing whole-map state was on disk and
    /// nothing scored it until item 1099 did. And an open game names the
    /// function that walks its word's window, which the compared pin is
    /// held to below. Made to fail first with Great Lakes' endpoint taken
    /// out of its row.
    #[test]
    fn a_closed_ai_word_names_its_endpoint_and_an_open_one_its_window() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff");
        let mut source = String::new();
        for entry in std::fs::read_dir(root).expect("src/diff") {
            let path = entry.expect("entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                source.push_str(&std::fs::read_to_string(&path).expect("read"));
            }
        }
        let mut wrong = Vec::new();
        for w in AI_WORDS {
            let (owed, what) = if w.closed() {
                (w.endpoint, "is closed and names no endpoint test")
            } else {
                (w.window, "is open and names no window")
            };
            match owed {
                None => wrong.push(format!("{} w{} of {} {what}", w.map, w.word, w.length)),
                Some(f) if !source.contains(&format!("fn {f}(")) => wrong.push(format!(
                    "{} names `{f}`, and no `fn {f}(` exists under src/diff/",
                    w.map
                )),
                Some(_) => {}
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    /// **The compared pin walks every open word's window** (parked 1067
    /// and 1080, the nineteenth pass). The pin held ninety-four fields as
    /// uncompared on a window closed since item 899 while the second
    /// pair's word stood on a group's order (DECISIONS 54), and what moved
    /// it to the word was an item, held there by a comment. The pin's test
    /// calls each open row's window function and no other. Made to fail
    /// first on the third map's window, which the pin did not walk.
    #[test]
    fn the_compared_pin_walks_every_open_ai_word() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff/coverage.rs");
        let source = std::fs::read_to_string(path).expect("coverage.rs");
        let lines: Vec<&str> = source.lines().collect();
        let head = "fn every_parsed_field_is_compared_by_the_instrument_or_pinned() {";
        let at = lines
            .iter()
            .position(|l| l.trim_start() == head)
            .expect("the compared pin's test");
        let body: Vec<&str> = lines[at + 1..]
            .iter()
            .take_while(|l| **l != "}")
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect();
        let walked: Vec<&str> = AI_WORDS
            .iter()
            .filter(|w| !w.closed())
            .filter_map(|w| w.window)
            .collect();
        let mut wrong = Vec::new();
        for f in &walked {
            if !body.iter().any(|l| l.contains(&format!("{f}()"))) {
                wrong.push(format!(
                    "`{f}` is an open word's window and the compared pin does not walk it"
                ));
            }
        }
        for l in &body {
            if let Some(i) = l.find("_word_window()") {
                let name = l[..i + "_word_window".len()]
                    .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()
                    .unwrap_or("");
                if !walked.contains(&name) {
                    wrong.push(format!(
                        "the compared pin walks `{name}()`, which no open row of AI_WORDS names"
                    ));
                }
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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

    /// **A widening's test holds its pins under the guard** (parked 973,
    /// the nineteenth pass): every test [`WIDENINGS`] names takes
    /// `Pins::hold()` and pins with `pin_eq!`, `pin_ne!` and `pin!`, so one
    /// failing run prints every pin that moved. A bare `assert` in such a
    /// test stops it on the first, and the pins behind it cost a second
    /// gate (items 882, 904, 989, 1034, 1072). The test's body is read to
    /// the closing brace at its own indentation, as `rustfmt` leaves it.
    /// Made to fail first on all forty-six, before any was converted.
    #[test]
    fn a_widening_s_test_holds_its_pins_under_the_guard() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff");
        let mut bare = Vec::new();
        let mut found = 0;
        for entry in std::fs::read_dir(root).expect("src/diff") {
            let path = entry.expect("entry").path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read");
            let lines: Vec<&str> = source.lines().collect();
            for (_, _, test, _, _) in WIDENINGS {
                let Some(t) = test else { continue };
                let head = format!("fn {t}() {{");
                let Some(at) = lines.iter().position(|l| l.trim_start() == head) else {
                    continue;
                };
                found += 1;
                let indent = lines[at].len() - lines[at].trim_start().len();
                let close = format!("{}}}", " ".repeat(indent));
                let body: Vec<&str> = lines[at + 1..]
                    .iter()
                    .take_while(|l| **l != close)
                    .copied()
                    .collect();
                if !body.iter().any(|l| l.contains("Pins::hold()")) {
                    bare.push(format!("{t}: takes no `Pins::hold()`"));
                }
                for (n, l) in body.iter().enumerate() {
                    let code = l.split("//").next().unwrap_or("");
                    for mac in ["assert!(", "assert_eq!(", "assert_ne!("] {
                        let stands = code.match_indices(mac).any(|(i, _)| {
                            !code[..i]
                                .chars()
                                .next_back()
                                .is_some_and(|c| c.is_alphanumeric() || c == '_')
                        });
                        if stands {
                            bare.push(format!("{t}, line {}: a bare `{mac}`", at + 2 + n));
                        }
                    }
                }
            }
        }
        let named = WIDENINGS.iter().filter(|w| w.2.is_some()).count();
        assert_eq!(
            found, named,
            "a test WIDENINGS names was not found by its head"
        );
        assert!(
            bare.is_empty(),
            "{} of a widening's pins stand outside the guard — take `let _pins = \
             Pins::hold();` first and pin with `pin_eq!`, `pin_ne!`, `pin!` \
             (`diff::testkit::pins`):\n  {}",
            bare.len(),
            bare.join("\n  ")
        );
    }
}
