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
use crate::testenv::{dump, install};

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
    // theirs `Leader::make_stuff+0x221` (`docs/AI.md` §241). **Item 2974
    // moved it 6585 → 12815** with the Germans' Fire Raft line
    // (`get_cost`'s `GERMAN_SUBMARINE_COST`, `train_time`'s
    // `GERMAN_SUBMARINE_SPEED`): who=1's make list on block 6581 heads the
    // Fire Raft (334) val 2000000 then the Trireme (340) on both sides (ours
    // the Trireme first, its Fire Raft at 500000 for a price of 20/41
    // against a wealth bucket of 39 — theirs 15/31), and its Dock `1/2010`
    // finishes the Fire Raft on 6718 on both. At 12815 ours 9 draws, theirs
    // 8, index 1: ours `Unit::fight+0x9b0`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` §242). **Item 2997 moved it 12815
    // → 12836** (`docs/AI.md` §244): run912 widens the gap 6837..12809 and
    // the state first parts on block 12563, who=1's thirty Citizens'
    // `myhits` 50 in ours against 40 (the Militia's pool reaches them at
    // the next leader pass); then the Scout `0/0`'s `damage` 16/0 against
    // 17/10 on 12767, the Elite Javelineers `1/42`'s javelin thrown from
    // the figure's point (6504, 8904) where theirs leaves the hand at
    // (6466, 8872), 14 frames against 13 (run912d); then the Horse Archer
    // `1/60`'s arrow, which kills the Scout on 12835 in theirs. At 12836
    // ours 7 draws, theirs 11, index 0: ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`; on block 12837 who=1's `1/31` is `collide` 1
    // (`collide_o` 68) at (6545, 12999) in theirs against 0 at (6524,
    // 12980), and `1/32`, `1/64` and `1/68` beside it. **Item 3086 moved
    // it 12836 → 13054 → 13519, the game's last frame** (`docs/AI.md`
    // §245): the four are attackers whose target, the Scout, died on
    // 12835, and the original measures no overreach to a target that
    // fails `TargetOrder::target_exists`, so they collide hard; then at
    // 13054 (ours 8, theirs 4, index 3: ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x104b`; block 13055's
    // `1/65` `idle` 1 against 0, `orders.len` 1 against 0) the Knight
    // `1/65`, orderless at `recharging` 20, steps nothing in the original
    // (`Unit::work@0060d180:79-86`) where ours idled and found it a
    // target. The draws agree to the end; run910 (to 13066) is the last
    // dump, so the close is not value-compared.
    count: 13519,
    sequence: 13519,
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

/// run893c: the word 6585's window, blocks 6579..6836 at the long's detail
/// (item 2956); run891 ends on block 5827.
pub(crate) const RUN893C: &str = "gamelog-run893c-eastindies-german-diff3-window-6579-6837.txt";

/// The window: block 6579 through 6836; the word 6585's own block is 6586.
pub(crate) const WIDENING_GERMAN_FRAME_6585: (i64, i64) = (6579, 6836);

pub(crate) fn german_frame_6585_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run893c",
        &[(RUN893C, WIDENING_GERMAN_FRAME_6585.0)],
        WIDENING_GERMAN_FRAME_6585,
        1,
        &[6586],
        true,
    )
}

#[test]
fn run893c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_frame_6585_window() else {
        return;
    };
    pin_eq!(w.blocks, 258, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 50, "initial run893c baseline");
    // Item 2956: 436 keys part on blocks 6579..6836, the word's own block
    // being 6586 (frame 6585: ours 6 draws, theirs 5, index 1 — ours
    // `Leader::use_market+0x1ed`, theirs `Leader::make_stuff+0x221`). The
    // state parts before the word: on 6581 who=1's make list heads with
    // type 340 in ours against 334 in theirs (`MAKE[1..3]` shifted by
    // one, `MAKE[3].num` 5/1); on 6583 its queue `1/2010` holds type
    // 340 against 334 (`cost` 40/15 and 42/31, `good[1]` 4/2), the
    // `num_queued` rows 290/284 follow, and the buckets part (timber 4/29,
    // wealth 39/8, metal 136/178). Standing on 6579: the control's set
    // (`SITE[n].reg` 65/0 on the empty slots, `form`, groups' `held`, the
    // pool, both leaders' `aggression` 0/1) and `g.angle[0]` on who=1's
    // `1/23`, `1/24`, `1/27`, `1/34`, `1/35` (1431655765 against 0); from
    // 6587 `0/3`'s move (`orders_x` 5688/5880) and the rest downstream
    // (`docs/AI.md` §241).
    //
    // **Item 2974: 436 → 50**, with the Germans' Fire Raft line. The value
    // diff, block 6581: who=1's `MAKE[0]` is the Fire Raft (334) val
    // 2000000 and `MAKE[1]` the Trireme (340) val 2000000 on both sides
    // (ours 340 and 209 before; ours' Fire Raft had been offered at 500000,
    // `check_income`'s 0x40 for an unaffordable 20/41 against wealth 39);
    // on 6583 the Dock `1/2010`'s `queue[0]` is type 334, `cost` 15/31, on
    // both; on 6718 its `job_counter` agrees (ours 13600 against 13533
    // with the cost arm alone) and the Fire Raft is born on 6719. What
    // stands is the control's set on 6579 (`SITE[n].reg`, `form`, groups'
    // `held`, the pool, both leaders' `aggression`), `g.angle[0]` on
    // who=1's `1/23`, `1/24`, `1/27`, `1/34`, `1/35` and from 6652 `1/37`
    // (1431655765 against 0), and two pools' `held` rows on 6606 and 6724.
    // The word moved to 12815, past the window's last block (6836); its
    // widening is run910's (`docs/AI.md` §242).
}

/// run910: the word 12815's window, blocks 12809..13066 at the long's
/// detail (item 2974); run893c ends on block 6836.
pub(crate) const RUN910: &str = "gamelog-run910-eastindies-german-diff3-window-12809-13067.txt";

/// The window: block 12809 through 13066; the word 12815's own block is
/// 12816.
pub(crate) const WIDENING_GERMAN_FRAME_12815: (i64, i64) = (12809, 13066);

pub(crate) fn german_frame_12815_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run910",
        &[(RUN910, WIDENING_GERMAN_FRAME_12815.0)],
        WIDENING_GERMAN_FRAME_12815,
        1,
        &[12816],
        true,
    )
}

#[test]
fn run910_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_frame_12815_window() else {
        return;
    };
    pin_eq!(w.blocks, 258, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 59, "initial run910 baseline");
    // Item 2974: 967 keys part on blocks 12809..13066, the word's own block
    // being 12816 (frame 12815: ours 9 draws, theirs 8, index 1 — ours
    // `Unit::fight+0x9b0`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`). **The state parts before the window**: on its
    // first block 12809 positions already differ — `0/0` at (4680, 9362) in
    // ours against (4674, 9396) (`orders_x` 4680/4632), `1/65` at (6394,
    // 11516) against (6392, 11520) (`orders_y` 9672/9720), `1/60`'s
    // heading — beside the control's standing set and `g.angle[0]`
    // 1431655765/0 on `1/83` and `1/86`. Up to the word: `1/44` and `1/47`
    // move targets on 12810, `1/28`'s on 12814, and on 12815 `1/44`'s
    // order kind 10 in ours against 1 in theirs. The first position
    // parting lies in 6837..12808, which no dump of this lobby covers
    // (`docs/AI.md` §242).
    //
    // **Item 2997: 967 → 932, and the word 12815 → 12836** (`docs/AI.md`
    // §244). Positions agree on the window's first block now: the gap's
    // three partings are built (run912). The Scout `0/0` dies on 12835 on
    // both sides (its `death:extra` row is the DEATHS record this
    // window's detail does not print). The word's own block 12837 is the
    // first that parts a position: who=1's `1/31` `collide` 0 in ours
    // against 1 (`collide_o` 68), at (6524, 12980) against (6545, 12999),
    // `half_step` 1 against 0, and `1/32`, `1/64`, `1/68` the same; the
    // count rose because the window's tail now runs past more agreeing
    // state into the new word's cast.
    //
    // **Item 3086: 932 → 77 → 59, and the word 12836 → 13054 → 13519**
    // (`docs/AI.md` §245). The attackers' overreach on the dead Scout
    // (block 12837's four collisions) takes it to 77, with block 13055's
    // `1/65` `idle` 1 against 0 and `orders.len` 1 against 0 the first
    // non-standing parting; the orderless recharging gate takes it to 59.
    // Past the standing set only `order:target` on `1/64` (12865) and
    // `1/28` (12926) part — ours' `0/2000` against theirs' dead `(0, 0)` —
    // and no position parts in the window.
}

/// The closing state's window: run643's closing block 13520 alone, the
/// game's last frame 13519 writing it (item 3086).
pub(crate) const WIDENING_GERMAN_CLOSING: (i64, i64) = (13518, 13521);

/// **The lobby's closing state** (item 3086, DECISIONS 55): the draws
/// agree to the game's last frame, 13519, and run643 — the draw stream's
/// own capture, `cover=0` — prints the closing block 13520 whole. Walked
/// to it and compared whole, as the peace pair's
/// `run883_lakota_pair_closing_state` is.
#[test]
fn run643_german_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(GERMAN.long.0),
        dump(GERMAN.start),
        trace(GERMAN.long.1),
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
        "German closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 13520, "the closing block");
    pin_eq!(e.compared, 90, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3086, the first walk: 90 units compared on block 13520, none
    // apart, no building or city field-row, and no closing field parted.
}

/// run912: the gap between run893c and run910, blocks 6837..12809 at
/// `UNITS=3,GUYS=2` only (item 2997), to find the block the state first
/// parts on before the word 12815.
pub(crate) const RUN912: &str = "gamelog-run912-eastindies-german-diff3-window-6837-12810.txt";

/// The window: block 6837 through 12809.
pub(crate) const WIDENING_GERMAN_GAP_912: (i64, i64) = (6837, 12809);

pub(crate) fn german_gap_912_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run912",
        &[(RUN912, WIDENING_GERMAN_GAP_912.0)],
        WIDENING_GERMAN_GAP_912,
        1,
        &[12816],
        true,
    )
}

#[test]
fn run912_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_gap_912_window() else {
        return;
    };
    pin_eq!(w.blocks, 5973, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 97, "initial run912 baseline");
    // Item 2997: the gap between run893c (ends 6836) and run910 (starts
    // 12809), at `UNITS=3,GUYS=2` only, so every `build:extra` row (the
    // buildings this crate holds and the dump does not print) is the
    // detail's, not a parting. Beside them only the control's standing set
    // (`form`, `group:*.held`, `pool:*`, `g.angle[0]`) parts until **block
    // 12563**, where who=1's thirty Citizens (TypeIndex 50) carry `myhits`
    // 50 in ours against 40 in theirs. The Tower `1/2014`'s Militia
    // research (TypeIndex 66) runs alike on both sides — queued on 12383
    // at 100 a frame (run912b), 18000 on 12562 and the slot empty on 12563
    // (run912c) — and the original raises the Citizens on 12564, at the
    // next `Leader::process`'s `calc_unit_stats`, where this crate raised
    // them inside `gain_tech` (`docs/AI.md` §244). The next rows are the
    // Scout `0/0`'s `damage_frac` on 12767 (ours 0, theirs 10) and its
    // `FLEETOORDER` on 12803, which theirs issues and ours does not.
    // 232 before item 2997's builds; **97** after them: the Militia's pool
    // at the next leader pass (12563), and the Javelineers' piece 34 bays
    // (12767), leave the standing set and the detail's `build:extra` rows
    // alone.
}
