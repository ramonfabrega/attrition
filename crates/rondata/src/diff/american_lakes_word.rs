//! **The battery's American word at Great Lakes** (DECISIONS 72, 83, 84;
//! item 3384): the held-out battery's lowest open game on the measure of
//! `689ddb67` (the Indians being item 3383's), promoted by item 3335's
//! recipe. Great Lakes (`MAP_STYLE 14`), human Nubians (4) against **the
//! Americans (20)** at Difficulty 4, seed 12345 — minted by item 3380 under
//! entry 72's rule (2), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. The name
//! `american_word` is the closed East Indies game's (item 3374). Captures:
//! run1004 (a `DUMP_ALL` start) and run1004b (the draw stream to the game's
//! end, `cover=0`, its setup and closing blocks only); the game ends itself
//! on 6152 (`docs/AI.md` §292).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN_LAKES: Lobby = Lobby {
    name: "GreatLakesAmericanTier4",
    map_style: 14,
    ai_tribe: 20,
    difficulty: 4,
    start: "gamelog-run1004-greatlakes-american-diff4-start.txt",
    long: (
        "gamelog-run1004b-greatlakes-american-diff4-24k-trace.txt",
        "rontrace-run1004b.log",
    ),
    length: 6152,
    // Item 3380's lobby, its first try: count 3176, sequence 3176 (ours 31
    // draws, theirs 36, at 0: ours `Animal::think_bird+0x82`, theirs
    // `Leader::compute_sites+0x4ac`). Item 3384 promoted it to the AI word
    // and moved it 3176 -> 6152, the game's last frame, with
    // `calc_gather`'s American Barracks gather (`docs/AI.md` §292): theirs'
    // script placed its second city on 3176, City State done in time
    // because Written Word was paid on 2776, a tick before ours.
    count: 6152,
    sequence: 6152,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AMERICAN_LAKES_WORD: i64 = if AMERICAN_LAKES.count < AMERICAN_LAKES.sequence {
    AMERICAN_LAKES.count
} else {
    AMERICAN_LAKES.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn american_lakes_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AMERICAN_LAKES.start, AMERICAN_LAKES.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_LAKES {}: count {} sequence {} of {}; {}",
        AMERICAN_LAKES.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        AMERICAN_LAKES.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, AMERICAN_LAKES.length, "the game's last frame");
    pin_eq!(
        w.count,
        AMERICAN_LAKES.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        AMERICAN_LAKES.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn american_lakes_word_is_no_other_lobby() {
    let p = &AMERICAN_LAKES;
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

/// run1009: the word 3176's window, blocks 3170..3426 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3384). run1004 dumps blocks 0..36 and
/// run1004b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN1009: &str = "gamelog-run1009-greatlakes-american-diff4-window-3170-3427.txt";

/// The window: block 3170 through 3426; the word 3176's own block is 3177.
pub(crate) const WIDENING_AMERICAN_LAKES_FRAME_3176: (i64, i64) = (3170, 3426);

pub(crate) fn american_lakes_frame_3176_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_LAKES.start],
        true,
        AMERICAN_LAKES.long,
        "run1009",
        &[(RUN1009, WIDENING_AMERICAN_LAKES_FRAME_3176.0)],
        WIDENING_AMERICAN_LAKES_FRAME_3176,
        1,
        &[3177],
        true,
    )
}

#[test]
fn run1009_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_lakes_frame_3176_window() else {
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
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3384, the first walk (the Americans' Barracks gather built, the
    // word at the game's end): 34 keys, the standing family alone — on
    // 3170 the leaders' `SITE[·].reg` and `aggression`, `form` of eleven
    // units, the army pools and `group:N.held` with no `GROUPDATA` printed;
    // `pool:64`/`group:64.held` on 3194 and who=1's `SITE[3].reg` on 3376,
    // the same two families. Nothing on the word's block 3177.
    pin_eq!(w.firsts.len(), 34, "keys that part in the window");
}

/// The closing state's window: run1004b's closing block alone, the long's
/// last frame 6152 writing it (item 3384).
pub(crate) const WIDENING_AMERICAN_LAKES_CLOSING: (i64, i64) = (6151, 6154);

/// **The lobby's closing state** (item 3384, DECISIONS 55): run1004b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 6152.
#[test]
fn run1004b_american_lakes_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        crate::testenv::dump(AMERICAN_LAKES.long.0),
        crate::testenv::dump(AMERICAN_LAKES.start),
        trace(AMERICAN_LAKES.long.1),
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
        "American (Great Lakes) closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 6153, "the closing block");
    pin_eq!(e.compared, 37, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3384, the first walk (the Americans' Barracks gather, the
    // word at 6152): 37 units compared on block 6153, none apart, no
    // building or city field-row, no closing field.
}
