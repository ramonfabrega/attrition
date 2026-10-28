//! **The Germans at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3719): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Germans (12)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 12, 4)`, is new: the Germans hold this map at 5
//! (`german_industrial_pair`), East Indies at 3 (`german_word`) and 5
//! (`german_alltech_pair`), and Great Sahara at 3 (`german_sahara_word`)
//! and 5 (`german_ancient_pair`); the battery fields no Germans. Captures:
//! run1396 (a `DUMP_ALL` start) and run1397 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §366.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GERMAN_MODERN: Pair = Pair {
    name: "GreatLakesGermanModern",
    map_style: 14,
    ai_tribe: 12,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1396-greatlakes-german-modern-start.txt",
    long: (
        "gamelog-run1397-greatlakes-german-modern-trace.txt",
        "rontrace-run1397.log",
    ),
    // The game ended itself on 5304 (closing block 5305).
    length: 5304,
    // Item 3719, first try: count 4079, sequence 4077 of 5304 — on 4077
    // ours 370 draws against theirs' 370, index 0 ours' `Guy::set_anim+0x97a
    // < Unit::do_guard+0x7f4` against theirs' `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d` (`docs/AI.md` §366). Item 3738: **4299**, count
    // and sequence — pieces 208 (the Tiger Tank) and 169 (the Armored Scout
    // Car) loose from their turrets (4077 → 4167 → 4299). On 4299 ours 16
    // draws against theirs' 15, index 2 ours' `Guy::set_anim+0xf2f <
    // Unit::set_anim+0x56` against theirs' `Unit::fight+0x824`; on 4298
    // theirs' Special Forces `0/0` takes a `CASTORDER` (spell 640) on the
    // Scout Car `1/53` that ours never gives (§366.1). Item 3739:
    // **4586**, count and sequence — `target_opportunity`'s counter-craft
    // arm: an idle Commando-line unit struck by a land unit answers with
    // Sabotage (4299 → 4586). On 4586 ours 94 draws against theirs' 99,
    // index 87 ours' `Unit::think_scout+0xaba` (`1/0`) against theirs'
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` (§366.2). Item 3742:
    // **4586 → 5304, the game's last frame**, by the count and the
    // sequence — `Unit::update_speed`'s Aluminum arm: who=1's aircraft move
    // `ALUMINUM_AIR_SPEED` faster, so the computer's sortie of 4176 finds
    // Napata inside a Bomber's speed times its tank (75 × 600 against ours'
    // 60 × 600) and its Bombers fly; their sight is the fog cell (23, 28)
    // that turned `1/0`'s region scan's thirty-sixth draw away on 4586
    // (`docs/AI.md` §366.3).
    count: 5304,
    sequence: 5304,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_MODERN_PAIR_WORD: i64 = if GERMAN_MODERN.count < GERMAN_MODERN.sequence {
    GERMAN_MODERN.count
} else {
    GERMAN_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn german_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn german_modern_pair_captures_say_their_lobby() {
    let p = &GERMAN_MODERN;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the German Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3719; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Germans — and no other German lobby's: `german_industrial_pair`'s
/// `(14, 12, 5)`, this map at another difficulty, `german_word`'s
/// `(18, 12, 3)`, `german_alltech_pair`'s `(18, 12, 5)`,
/// `german_sahara_word`'s `(7, 12, 3)` and `german_ancient_pair`'s
/// `(7, 12, 5)`, other maps (item 3719).
#[test]
fn german_modern_pair_is_no_other_lobby() {
    let p = &GERMAN_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the Germans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another German lobby on this map at this difficulty"
        );
    }
    let w = &super::german_word::GERMAN;
    let s = &super::german_sahara_word::GERMAN_SAHARA;
    let a = &super::german_alltech_pair::GERMAN_ALLTECH;
    let n = &super::german_ancient_pair::GERMAN_ANCIENT;
    let i = &super::german_industrial_pair::GERMAN_INDUSTRIAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
        (n.name, (n.map_style, n.ai_tribe, n.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Germans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3719, from
/// 4077): only run1397's stride blocks are on disk before the word's
/// window, 101 through 4001.
pub(crate) const WIDENING_GERMAN_MODERN_STRIDE: (i64, i64) = (101, 4001);

pub(crate) fn german_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1397 stride",
        &[(GERMAN_MODERN.long.0, WIDENING_GERMAN_MODERN_STRIDE.0)],
        WIDENING_GERMAN_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn german_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 40, "every stride block in the window");
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
    // Item 3719: 114 keys over 40 blocks. The standing families — `form`
    // on every unit, the buildings' `build:extra` (which the stride does
    // not print) and who=1's groups with their pools — and, from block
    // 3101, `myspeed` on who=1's units, theirs a quarter above ours (75
    // against 93, 60 against 75, 115 against 143) with no `pos` parting:
    // parked 3677a's and 3705b's family. Item 3742: 114 → **101**, the
    // `myspeed` family gone with the Aluminum arm; what stands is `form`,
    // `build:extra`, and who=1's groups and pools.
    pin_eq!(w.firsts.len(), 101, "the stride's baseline");
}

/// run1397b: the word 4077's window, blocks 3995..4328 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4001 and 250 past the word's own block 4078 (item 3719).
/// run1397 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1397B: &str = "gamelog-run1397b-greatlakes-german-modern-window-3995-4328.txt";

/// The window: block 3995 through 4328.
pub(crate) const WIDENING_GERMAN_MODERN_FRAME_4077: (i64, i64) = (3995, 4328);

/// **The word 4077's window**: run1397b walked from run1396's start with
/// the recorder on and every round compared (item 3719).
pub(crate) fn german_modern_frame_4077_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1397b",
        &[(RUN1397B, WIDENING_GERMAN_MODERN_FRAME_4077.0)],
        WIDENING_GERMAN_MODERN_FRAME_4077,
        1,
        &[4078],
        true,
    )
}

#[test]
fn german_modern_pair_run1397b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_frame_4077_window() else {
        return;
    };
    pin_eq!(w.blocks, 333, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3719, the first walk: **1946** keys. Block 3995 carries the
    // standing families (both leaders' `SITE[i].reg`, `aggression[1]`,
    // `form`, groups and pools, the airports' `myspeed`). **The word**: on
    // 4017 theirs' Tiger Tank `1/38` (TI 258, piece 208, which neither
    // `launch` nor `pivot` holds) looses a round that is made and explodes
    // inside the frame and wounds the Citizen `0/12` (TI 50, hits 85 → 49);
    // ours' leaves from the guy's own square, (4392, 31416), and is still
    // in flight on 4018 (`t1/2`). Theirs' citizen flees on a `FLEETOORDER`
    // to (2136, 32616), arrives on 4077, drops the order and goes idle with
    // one `Guy::set_anim+0x97a < Unit::do_idle+0x7d` draw ours never
    // spends; the same frame theirs' Tiger `1/62` drops its attack on the
    // Citizen `0/13`, fled since its own round of 4038, and spends no
    // `Unit::fight+0x824` draw where ours does.
    // Item 3738, pieces 208 and 169's turret rows: **282** keys. The
    // Tiger's round of 4017 is made and lands inside its frame on both
    // sides; the first unit parting past block 3995's standing rows is
    // `1/49`'s `dest_angle` on 4103 (one field), then who=1's aircraft's
    // orders in their airports from 4177 (3719a's family), then the Special
    // Forces `0/0` on 4298: `order:kind` ours 4 theirs 14, `orders_x/y` ours
    // (4392, 30360) theirs (4248, 31896).
    // Item 3739, the counter-craft arm: **180** keys. Theirs' and ours'
    // `0/0` take the same `CASTORDER` (spell 640 on `1/53`) on 4298 and
    // pay 1000 mana on 4299; nothing parts on `0/0` past block 3995's
    // `form`. Past the aircraft family, the one unit row is `1/69`'s lone
    // `dest_angle` on 4320 (ours 0, theirs 1476001792).
    // Item 3742, the Aluminum arm: 180 → **61**. The aircraft fly from 4177
    // in ours too, under the same `AIRPATROLORDER` on Napata (3168, 30816);
    // what parts on them is `mirror` on each one's takeoff block (ours 0,
    // theirs 1, eight rows, 3742a). Past block 3995's 47 standing rows:
    // `1/49`'s `dest_angle` on 4103, groups 65 and 69's lists on 4171 and
    // 4180, and `1/69`'s `dest_angle` on 4320.
    pin_eq!(w.firsts.len(), 61, "run1397b baseline");
    // Every round, both directions: 137 in ours against 176, the first
    // parting the Tiger `1/38`'s on 4018, which ours holds alone. Item 3738:
    // 137 against 144 and 18 parting, the first `1/53`'s `ammo[1]` on 4301
    // (`ex, ey` (4264, 31704) against (4236, 32019)), past the word.
    // Item 3739: 137 against 137, none parting — the saboteur's target
    // `1/53` holds its ground and fires the same rounds.
    pin_eq!(w.rounds_read, (137, 137), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1418: the word 4586's window, blocks 4495..4836 at run1397b's
/// detail — six blocks before the stride's last block before the word,
/// 4501, and 250 past the word's own block 4587 (item 3739). run1397b
/// ends on 4328, and run1397's stride prints units only.
pub(crate) const RUN1418: &str = "gamelog-run1418-greatlakes-german-modern-window-4495-4836.txt";

/// The window: block 4495 through 4836.
pub(crate) const WIDENING_GERMAN_MODERN_FRAME_4586: (i64, i64) = (4495, 4836);

/// **The word 4586's window**: run1418 walked from run1396's start with
/// the recorder on and every round compared (item 3739).
pub(crate) fn german_modern_frame_4586_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1418",
        &[(RUN1418, WIDENING_GERMAN_MODERN_FRAME_4586.0)],
        WIDENING_GERMAN_MODERN_FRAME_4586,
        1,
        &[4587],
        true,
    )
}

#[test]
fn german_modern_pair_run1418_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_frame_4586_window() else {
        return;
    };
    pin_eq!(w.blocks, 341, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3739, the first walk: **877** keys. Block 4495 carries 172
    // standing rows (`form`, both leaders' `SITE[i].reg` and who=0's
    // `aggression[1]`, who=1's groups and pools, the airports' `myspeed`
    // and eight aircraft parted since 4177, 3719a's family). Past them:
    // the Bomber `1/36`'s `myspeed` on 4528 (115 against 143, the same
    // family); who=1's `leader:strong[0]` on 4545 (ours 2016, theirs
    // 1984); then **the word's block 4587**: group 65's `held` and `pool`
    // hold `[0]` in ours and nothing in theirs, where both sides' Special
    // Forces `1/0` (TI 75) leave group 68 for 65 under the same
    // `EXPLORETOORDER` to (10488, 43512) and its unit record agrees. On
    // 4586 ours spends 36 `Unit::think_scout+0xaba` draws against theirs'
    // 35 and one `Guy::set_anim+0x104b` against seven (`docs/AI.md`
    // §366.2). The first unit position parting is `1/12`'s on 4629.
    // Item 3742, the Aluminum arm: 877 → **58**, nothing past the word.
    // Block 4495's 53 standing rows (`form`, the leaders' `SITE[i].reg`,
    // who=1's groups and pools, and the eight aircraft's `mirror`, 3742a),
    // the Bomber `1/36`'s `g.z` on 4528, and groups 65 and 69's lists on
    // 4587 and 4687.
    pin_eq!(w.firsts.len(), 58, "run1418 baseline");
    // Every round, both directions: 912 against 102, 55 parting, the
    // first the Bomber `1/95`'s `ammo[0]` on 4753, which the dump holds
    // alone — the airports' family, past the word. Item 3742: (912, 912),
    // **32** parting, every one a Bomber's (`1/94`, `1/95`, `1/106`,
    // `1/107`, `1/113`) over Napata from 4770: `accuracy` ours 0 against
    // theirs 252..256, and `sy`/`ey` one unit apart (3742b).
    pin_eq!(w.rounds_read, (912, 912), "the rounds read");
    pin_eq!(w.rounds.len(), 32, "the rounds that part");
}

/// run1422: the pair's closing window, blocks 5048..5305 at the long's
/// detail with `LEADERS=9` and `AMMO=5` (item 3742). The word reached the
/// game's last frame; run1397's stride prints units every hundred frames
/// and run1418 ends on 4836, so no dump held the last blocks.
pub(crate) const RUN1422: &str =
    "gamelog-run1422-greatlakes-german-modern-closing-window-5048-5305.txt";

/// The window: block 5048 through the closing block 5305.
pub(crate) const WIDENING_GERMAN_MODERN_CLOSING: (i64, i64) = (5048, 5305);

/// **The pair's last blocks**: run1422 walked from run1396's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3742).
pub(crate) fn german_modern_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1422",
        &[(RUN1422, WIDENING_GERMAN_MODERN_CLOSING.0)],
        WIDENING_GERMAN_MODERN_CLOSING,
        1,
        &[5305],
        true,
    )
}

#[test]
fn german_modern_pair_run1422_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3742, the pair whole: **55** keys. Block 5048 carries the
    // standing families (`form` on 23 units, both leaders' `SITE[i].reg`,
    // who=0's `aggression[1]`, who=1's groups 64..67 with their pools) and
    // eight aircraft's `mirror`, ours 0 against theirs 1 (parked 3742a).
    // **One record is not a standing family**: the human's city `0/2000`
    // on 5157, `city_flags[0x4]` 0 against 1 and `reduce_stamp` 4806
    // against 5156 (parked 3742d). No unit position or order parts, no
    // draw parts to 5304, and the closing block 5305 agrees.
    pin_eq!(w.firsts.len(), 55, "run1422 baseline");
    // Every round, both directions: 228 each side, 13 rows parting. The
    // Bomber `1/36`'s `ammo[0]` on 5048, `sx`/`sy` one unit apart and
    // `angle` (3742b's family), and the Volksgrenadiers `1/48` (5291) and
    // `1/50` (5304), TI 109, each round's `sx`, `sy`, `sz`, `angle` and
    // `v1z`: a launch point off the figure's own (parked 3742e). The draws
    // never feel it.
    pin_eq!(w.rounds_read, (228, 228), "the rounds read");
    pin_eq!(w.rounds.len(), 13, "the rounds that part");
}

/// **The pair's closing state** (item 3719): run1397's closing block 5305,
/// the whole map's units, walked from run1396's start.
#[test]
fn german_modern_pair_run1397_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GERMAN_MODERN;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
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
        "German Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5305, "the closing block");
    // Item 3719, first try: 1227 frames past the word 4077 the game has
    // parted widely — 146 units compared, counts [127, 2, 3, 1, 51, 0, 55],
    // `0/13`, `1/36` and `1/111` extra in ours, `1/139` and `1/141`
    // unlinked. A floor, not a score. Item 3738 (the word at 4299): counts
    // [116, 2, 3, 2, 27, 1, 23]. Item 3739 (the word at 4586): counts
    // [112, 2, 2, 3, 27, 1, 8], `0/13` no longer extra. Item 3742, the word
    // closed at 5304: **148** units, every one on and linked, no extra; the
    // one count is a city, `1/2069`'s `assimilation_timer` 5304 against
    // 5301 (3742c).
    pin_eq!(e.compared, 148, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
