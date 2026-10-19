//! **The battery's third word** (DECISIONS 72, 83; item 3314): the
//! held-out battery's lowest game, promoted by item 2904's and 3107's
//! recipe. East Indies (`MAP_STYLE 18`), human Nubians (4) against **the
//! Romans (6)** at Difficulty 2, seed 12345 — `docs/AI.md` §243's fifth
//! lobby, scored by its first try at 576 of 18,620 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run899 (a
//! `DUMP_ALL` start, blocks 0..36) and run899b (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the word's
//! window is run956's (`docs/AI.md` §263). `GreatSaharaSpanishTier3` parts
//! on 576 too and stays held out: it is never debugged against.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const ROMAN: Lobby = Lobby {
    name: "EastIndiesRomanTier2",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 2,
    start: "gamelog-run899-eastindies-roman-diff2-start.txt",
    long: (
        "gamelog-run899b-eastindies-roman-diff2-24k-trace.txt",
        "rontrace-run899b.log",
    ),
    length: 18620,
    // Item 2975: the battery's first try, count and sequence both at 576
    // (ours 28 draws, theirs 89, index 0: ours `Animal::think_bird+0x82`,
    // theirs `Leader::produce_building+0xc99`). Item 3314 promoted it to
    // the AI word (`docs/AI.md` §263). **Item 3314 moved it 576 →
    // 18026** with `get_cost`'s fort arm (`ROMAN_FORT_COST`, 25 % off a
    // fort or a Tower-line building for a Roman leader, `00665e55`..), which
    // this crate never carried: who=1's Tower, bought on frame 1, cost ours
    // a quarter more, so run956's block 570 had its bucket timber 46 and
    // wealth 31 against theirs' 62 and 39, and frame 576's fourth Farm (52
    // timber) was refused in ours and bought in theirs (`1/2007`, its
    // `Farms::add` draws, then four `place_city_with_cost` rounds). Both
    // read 62 and 39 now, and the Farm is on both sides on block 577. At
    // 18026 ours 199 draws, theirs 200, index 192: ours `Guy::set_anim+0x97a
    // < Guy::inc_time+0x271`, theirs `PathFinder::calc_road_cost+0x46`;
    // past run956's last block (826), its widening owed.
    count: 18026,
    sequence: 18026,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const ROMAN_WORD: i64 = if ROMAN.count < ROMAN.sequence {
    ROMAN.count
} else {
    ROMAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn roman_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(ROMAN.start, ROMAN.long) else {
        return;
    };
    eprintln!(
        "ROMAN {}: count {} sequence {} of {}; {}",
        ROMAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, ROMAN.difficulty, "the lobby read back");
    pin_eq!(w.last, ROMAN.length, "the game's last frame");
    pin_eq!(w.count, ROMAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        ROMAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn roman_word_is_no_other_lobby() {
    let p = &ROMAN;
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

/// run956: the word 576's window, blocks 570..826 at the long's detail
/// (item 3314, run886's recipe); run899 dumps blocks 0..36 and run899b its
/// setup and closing blocks only.
pub(crate) const RUN956: &str = "gamelog-run956-eastindies-roman-diff2-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_ROMAN_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn roman_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN.start],
        true,
        ROMAN.long,
        "run956",
        &[(RUN956, WIDENING_ROMAN_FRAME_576.0)],
        WIDENING_ROMAN_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run956_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_frame_576_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 29, "initial run956 baseline");
    // Item 3314: 239 keys part on blocks 570..826, the word's own block
    // being 577 (frame 576: ours 28 draws, theirs 89, index 0 — theirs one
    // `place_building_with_cost` of a Farm, `produce_building+0xc99` ×39 and
    // `+0x1805` ×4, its `Farms::add` pair, then four `place_city_with_cost`
    // rounds; ours none of them). Standing from 570: who=1's
    // `bucket[timber]` 46/62 and `bucket[wealth]` 31/39 (the Tower of frame
    // 1 priced without the Romans' quarter off); on 577 theirs holds the
    // Farm `1/2007` and ours none (`build:unlinked`), `1/2000`'s
    // `city_down` 2007, the Citizen `1/2`'s order on it, the city's
    // `filled`/`gatherers`/`space`, who=1's `script_step` 9/10; from 584
    // the timber `income`, `resources`, `leftover` and `gather_stamp`, then
    // the walkers both sides route around the Farm.
    //
    // **Item 3314: 239 → 29**, with `get_cost`'s fort arm
    // (`cost::Modifiers::fort`). The word's value diff, block 570: who=1's
    // bucket timber 62 and wealth 39 on both (ours 46 and 31 before); block
    // 577: the Farm `1/2007` on both. What stands is the control's 29 from
    // 570 — `SITE[n].reg`, `form`, the groups' `held`, the pool and both
    // leaders' `aggression` — and who=1's `SITE[2].reg` on 576 as its
    // sites fill. The word moved to 18026, past the window's last block.
}
