//! **The battery's word** (DECISIONS 72, item 2904): every pair closed at
//! its game's end (the peace pair on item 2864), and the AI word's
//! successor is the held-out battery's lowest lobby. East Indies
//! (`MAP_STYLE 18`), human Nubians (4) against **the Germans (12)** at
//! Difficulty 3, seed 12345 — `docs/AI.md` §125's first lobby, scored by
//! its first try at 576 of 13,519 and promoted out of the battery here, so
//! it is a scored lobby from this item (`battery::SCORED_LOBBIES`) and no
//! longer held out. Captures: run646 (a `DUMP_ALL` start) and run643 (the
//! draw stream to the game's end, `cover=0`, its setup and closing blocks
//! only); the word's window is run886's (`docs/AI.md` §239).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GERMAN: Lobby = Lobby {
    name: "EastIndiesGermanTier3",
    map_style: 18,
    ai_tribe: 12,
    difficulty: 3,
    start: "gamelog-run646-eastindies-german-diff3-start.txt",
    long: (
        "gamelog-run643-eastindies-german-diff3-24k-trace.txt",
        "rontrace-run643.log",
    ),
    length: 13519,
    // Item 1465: the battery's first try, count and sequence both at 576
    // (ours 52 draws, theirs 36, index 4: ours `Leader::compute_sites+0x4ac`,
    // theirs `Animal::think_bird+0x82`). Item 2904 promoted it to the AI
    // word unmoved (`docs/AI.md` §239). **Item 2931 moved it 576 → 5577**
    // with the Germans' flat per-city gather (`GERMAN_CITY_GATHER`,
    // `calc_city_resources@006d5530`): who=1's income on 576 reads food
    // 880 and timber 720 on both sides (ours 800 and 640 before), its
    // timber bucket 64 on both (ours 58, two short of the Village's 60), and
    // ours' one site round buys the Village `1/2007` theirs does. At 5577
    // ours 8 draws, theirs 6, index 1: ours `Leader::use_market+0x1ed`,
    // theirs `Leader::make_stuff+0x221` (`docs/AI.md` §240). **Item 2956
    // moved it 5577 → 6585** with `blocked_site`'s unseen tally: the 3975
    // sweep's Town site at (47, 24), 29 of its 49 footprint tiles unseen,
    // is refused `0x24` on both sides now (run893b's packet), so who=1's
    // `SITE[8]` on block 3976 is (42, 53) val 32 on both (ours (47, 24) val
    // 293333 before) and the third Village offer of 5578 is gone. At 6585
    // ours 6 draws, theirs 5, index 1: ours `Leader::use_market+0x1ed`,
    // theirs `Leader::make_stuff+0x221` (`docs/AI.md` §241).
    count: 6585,
    sequence: 6585,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const GERMAN_WORD: i64 = if GERMAN.count < GERMAN.sequence {
    GERMAN.count
} else {
    GERMAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn german_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(GERMAN.start, GERMAN.long) else {
        return;
    };
    eprintln!(
        "GERMAN {}: count {} sequence {} of {}; {}",
        GERMAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, GERMAN.difficulty, "the lobby read back");
    pin_eq!(w.last, GERMAN.length, "the game's last frame");
    pin_eq!(w.count, GERMAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        GERMAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn german_word_is_no_other_lobby() {
    let p = &GERMAN;
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

/// run886: the word 576's window, blocks 570..826 at the long's detail
/// (item 2904); run643 dumps its setup and closing blocks only.
pub(crate) const RUN886: &str = "gamelog-run886-eastindies-german-diff3-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_GERMAN_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn german_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run886",
        &[(RUN886, WIDENING_GERMAN_FRAME_576.0)],
        WIDENING_GERMAN_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run886_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_frame_576_window() else {
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
    pin_eq!(w.firsts.len(), 31, "initial run886 baseline");
    // Item 2904: 262 keys part on blocks 570..826, the word's own block
    // being 577 (frame 576: ours 52 draws, theirs 36, index 4 — theirs one
    // round of `Leader::compute_sites+0x4ac`/`+0x50a` and
    // `Leader::make_stuff+0x221` ×2, ours five, then the same birds,
    // objects, herd and farms). On 577 theirs holds who=1's Village `1/2007`
    // (type 414, a foundation at (34656, 36192), `constr_time` 60000) and
    // ours none (`build:unlinked`); who=1's Citizen `1/2` holds theirs'
    // `BUILDORDER` on it (`orders.len` 1/2, `orders_x` 40632/35400,
    // `path` 0/5); who=1's `SITE[1].wx` 51/0 (theirs' site taken) and the
    // table below it shifted by one; `gatherers` 8/7. 35 stand on 570:
    // who=1's `income` and `resources` (food 800/880, timber 640/720),
    // `leftover` (food 5280/480, timber 800/3200), `bucket` (food 73/80,
    // timber 58/64), both leaders' `aggression` 0/1, and the control's
    // `SITE[n].reg`, `form` and groups' `held` (`docs/AI.md` §239).
    //
    // **Item 2931: 262 → 31**, with the Germans' flat per-city gather. The
    // word's value diff, block 577: who=1's Village `1/2007` (type 414 at
    // (34656, 36192)) is on both sides now, and the Citizen `1/2`'s
    // `BUILDORDER` on it; who=1's `income` and `resources` read food 880
    // and timber 720 on both from 570 (ours 800 and 640 before), and the
    // `leftover` and `bucket` rows are gone with them (timber bucket 64 on
    // both, ours 58 before). What stands is the control's 27 from 570 —
    // `SITE[n].reg`, `form`, groups' `held`, the pool, both leaders'
    // `aggression` 0/1 — and who=1's `SITE[1..4].reg` on 576 and 776 as
    // its sites fill. The word moved to 5577, past the window's last block
    // (826); its widening is run891's (`docs/AI.md` §240).
}

/// run891: the word 5577's window, blocks 5571..5827 at the long's detail
/// (item 2931); run886 ends on block 826 and run643 dumps its setup and
/// closing blocks only.
pub(crate) const RUN891: &str = "gamelog-run891-eastindies-german-diff3-window-5571-5828.txt";

/// The window: block 5571 through 5827; the word 5577's own block is 5578.
pub(crate) const WIDENING_GERMAN_FRAME_5577: (i64, i64) = (5571, 5827);

pub(crate) fn german_frame_5577_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run891",
        &[(RUN891, WIDENING_GERMAN_FRAME_5577.0)],
        WIDENING_GERMAN_FRAME_5577,
        1,
        &[5578],
        true,
    )
}

#[test]
fn run891_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_frame_5577_window() else {
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
    pin_eq!(w.firsts.len(), 36, "initial run891 baseline");
    // Item 2931: 302 keys part on blocks 5571..5827, the word's own block
    // being 5578 (frame 5577: ours 8 draws, theirs 6, index 1 — ours
    // `Leader::use_market+0x1ed` ×2 and `Leader::make_stuff+0x221` ×3,
    // theirs one and two). On 5578 who=1's make list holds three Village
    // offers in ours and two in theirs: `MAKE[0]` val 664398 at (47, 24)
    // against 286098 at (46, 27), `MAKE[1]` 286098 at (46, 27) against
    // 11400 at (51, 30), `MAKE[2]` 11400 at (51, 30) against empty. The
    // site table stands apart from 570's window on: on 5571 who=1's
    // `SITE[8]` is (47, 24) val 330000 in ours against (51, 30) val 1824,
    // `SITE[6]` (49, 31) reg 5 against (44, 52) reg 11; on 5576
    // `SITE[2..4]` shift by one. 46 stand on 5571: those rows, the
    // control's `SITE[n].reg`, `form`, groups' `held` and the pool, and
    // both leaders' `aggression` 0/1 (`docs/AI.md` §240).
    //
    // **Item 2956: 302 → 36**, with `blocked_site`'s unseen tally. The
    // word's value diff, block 5578: who=1's make list holds two Village
    // offers on both sides, `MAKE[0]` val 286098 at (46, 27) and `MAKE[1]`
    // 11400 at (51, 30) (ours (47, 24) val 664398 first before), and on
    // 5571 its `SITE[6..9]` agree — (44, 52) val 299, (49, 31) and (51, 30)
    // val 1824, (46, 27) val 269472 (ours (47, 24) val 330000 before). What
    // stands is the control's set on 5571 (`SITE[n].reg` 65/0 on the empty
    // slots, `form`, groups' `held`, the pool, both leaders' `aggression`
    // 0/1) and two pools' `held` rows on 5622 and 5734. The word moved to
    // 6585, past the window's last block (5827); its widening is run893c's
    // (`docs/AI.md` §241).
}

/// run893: the site table's walk-back, blocks 3770..4177 at the long's
/// detail (item 2956) — the 3775, 3975 and 4175 sweeps; run886 ends on
/// block 826 and run891 starts on 5571.
pub(crate) const RUN893: &str = "gamelog-run893-eastindies-german-diff3-window-3770-4178.txt";

/// The window: block 3770 through 4177; the sweep 3975's block is 3976.
pub(crate) const WIDENING_GERMAN_SITES_3975: (i64, i64) = (3770, 4177);

pub(crate) fn german_sites_3975_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run893",
        &[(RUN893, WIDENING_GERMAN_SITES_3975.0)],
        WIDENING_GERMAN_SITES_3975,
        1,
        &[3976],
        true,
    )
}

#[test]
fn run893_s_site_sweep_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_sites_3975_window() else {
        return;
    };
    pin_eq!(w.blocks, 408, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 37, "initial run893 baseline");
    // Item 2956: the site table's walk-back. Before the build 62 keys
    // parted, the first not standing on block 3976: who=1's `SITE[8]` (47,
    // 24) val 293333 reg 5 in ours against (42, 53) val 32 reg 11, the
    // 3975 sweep's sample that theirs refuses — `blocked_site` returns
    // `0x24` for the Town's footprint there, 29 of 49 tiles unseen (run893b's
    // packet, `compute_site_stats@006cd040` leaving at `006cd4a5`). With
    // the tally, 37: the control's standing set from 3770 (`SITE[n].reg`
    // 65/0 on the empty slots, `form`, groups' `held`, the pool, both
    // leaders' `aggression` 0/1) and who=1's `SITE[5].reg` on 3776, a
    // group's `held` on 3777 and two pools on 3795 and 3862, every one of
    // the same standing kinds (`docs/AI.md` §241).
}
