//! **The battery's second word** (DECISIONS 72, item 3107): the first
//! battery word, `EastIndiesGermanTier3`, closed at its game's end on item
//! 3086, and its successor is the held-out battery's lowest lobby. Great
//! Lakes (`MAP_STYLE 14`), human Nubians (4) against **the Iroquois (18)**
//! at Difficulty 3, seed 12345 — `docs/AI.md` §243's third lobby, scored by
//! its first try at 0 of 12,658 and promoted out of the battery here, so it
//! is a scored lobby from this item (`battery::SCORED_LOBBIES`) and no
//! longer held out. Captures: run897 (a `DUMP_ALL` start, blocks 1, 36 and
//! 37) and run897b (the draw stream to the game's end, `cover=0`, its setup
//! and closing blocks only); the word's window is run897's own block 1
//! (`docs/AI.md` §246).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const IROQUOIS: Lobby = Lobby {
    name: "GreatLakesIroquoisTier3",
    map_style: 14,
    ai_tribe: 18,
    difficulty: 3,
    start: "gamelog-run897-greatlakes-iroquois-diff3-start.txt",
    long: (
        "gamelog-run897b-greatlakes-iroquois-diff3-24k-trace.txt",
        "rontrace-run897b.log",
    ),
    length: 12658,
    // Item 2975: the battery's first try, count and sequence both at 0
    // (ours 121 draws, theirs 122, index 30: ours `Unit::think_scout+0x436`,
    // theirs `Unit::think_scout+0x64c`). Item 3107 promoted it to the AI
    // word unmoved (`docs/AI.md` §246).
    count: 0,
    sequence: 0,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const IROQUOIS_WORD: i64 = if IROQUOIS.count < IROQUOIS.sequence {
    IROQUOIS.count
} else {
    IROQUOIS.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn iroquois_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(IROQUOIS.start, IROQUOIS.long) else {
        return;
    };
    eprintln!(
        "IROQUOIS {}: count {} sequence {} of {}; {}",
        IROQUOIS.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, IROQUOIS.difficulty, "the lobby read back");
    pin_eq!(w.last, IROQUOIS.length, "the game's last frame");
    pin_eq!(w.count, IROQUOIS.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        IROQUOIS.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn iroquois_word_is_no_other_lobby() {
    let p = &IROQUOIS;
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
        let tribes: Vec<_> = init
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe))
            .collect();
        assert_eq!(tribes, [(0, 4), (1, i64::from(p.ai_tribe))], "{name}");
    }
}

/// run897's own blocks, 0 and 1: the start dump's whole records, which is
/// where frame 0's word lives (item 3107; the coverage pair's
/// `WIDENING_COVERAGE_START` is the pattern).
pub(crate) const WIDENING_IROQUOIS_START: (i64, i64) = (0, 1);

/// The word's widening: every dumped record on block 1 (the state frame 0
/// writes), the start dump itself as the capture.
pub(crate) fn iroquois_start_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run897",
        &[(IROQUOIS.start, 0)],
        WIDENING_IROQUOIS_START,
        1,
        &[1],
        true,
    )
}

#[test]
fn run897_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_start_window() else {
        return;
    };
    pin_eq!(
        w.blocks,
        1,
        "every captured block: the start dump's block 1"
    );
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(w.firsts.len(), 45, "initial run897 baseline");
    // Item 3107, on base 8d08a485: 45 keys part on block 1 (frame 0: ours
    // 121 draws, theirs 122, index 30 — ours `Unit::think_scout+0x436`,
    // theirs `Unit::think_scout+0x64c`). The control's 31 (both leaders'
    // `SITE[n].reg` 1/0, `aggression` 0/1, and the starting units' `form`
    // -1/9) stand on every lobby's block 1. The 14 the lobby adds are all
    // who=1's Scout `1/0` and its group 65: `myhits`, `hits_left` and
    // `hits:myhits` 60 in ours against 63; its explore move's `y` 19704
    // against 12792 (`orders_y`, the path's three legs against six, its
    // `angle` 1913782272 against 120324096, `facing` 0/1), and group 65's
    // `oy` 19680/12768 and `o_angle` with it (`docs/AI.md` §246).
}

/// PROBE (item 3107): run897's block 1 fog plane against ours after frame 0.
#[test]
fn probe_iroquois_fog_block_1() {
    if std::env::var_os("RON_PROBE_3107").is_none() {
        return;
    }
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(start), Some(tr)) = (dump(IROQUOIS.start), trace(IROQUOIS.long.1)) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let start_text = crate::capture::read(start);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let w0 = &built.sim.world;
    let (fw, fh) = (w0.fog_xs(), w0.fog_ys());
    let before: Vec<u8> = (0..fh)
        .flat_map(|y| (0..fw).map(move |x| (x, y)))
        .map(|(x, y)| w0.seen2(x, y).unwrap_or(0))
        .collect();
    built.tick();
    let block = start_log
        .frames()
        .into_iter()
        .find(|(n, _)| *n == 1)
        .map(|(_, b)| b)
        .expect("block 1");
    let world = block
        .kid("FULL DUMP")
        .unwrap_or(block)
        .kid("WORLD")
        .expect("WORLD");
    let theirs = crate::gamelog::world_fog(&world.fields().to_vec());
    let w = &built.sim.world;
    eprintln!("fog {fw}x{fh}, theirs {}", theirs.len());
    let mut n = 0;
    for y in 0..fh {
        for x in 0..fw {
            let t = theirs[(y * fw + x) as usize];
            let o = w.seen2(x, y).unwrap_or(0);
            if o != t {
                n += 1;
                if n < 200 {
                    eprintln!(
                        "  ({x},{y}) cell ({},{}) ours {o:#x} theirs {t:#x} before {:#x}",
                        x / 2,
                        y / 2,
                        before[(y * fw + x) as usize]
                    );
                }
            }
        }
    }
    eprintln!("{n} half-cells part");
    for u in built.sim.units.iter().filter(|u| u.owner == 1) {
        eprintln!("  unit 1 ty {:?} pos {:?}", u.ty, u.pos);
    }
}
