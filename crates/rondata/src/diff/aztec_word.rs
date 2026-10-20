//! **The battery's Aztec word** (DECISIONS 72, 83, 84; item 3361): the
//! held-out battery's next open game once item 3359 closed the French and
//! the Inca went to item 3360, promoted by item 3335's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Aztecs (0)** at
//! Difficulty 2, seed 12345 — minted by item 2975 (DECISIONS 77 (ii), a
//! nation no dump held), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run895 (a
//! `DUMP_ALL` start) and run895b (the draw stream to the game's end,
//! `cover=0`, its setup and closing blocks only); the game ends itself on
//! 17554 (`docs/AI.md` §282).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AZTEC: Lobby = Lobby {
    name: "GreatSaharaAztecTier2",
    map_style: 7,
    ai_tribe: 0,
    difficulty: 2,
    start: "gamelog-run895-greatsahara-aztec-diff2-start.txt",
    long: (
        "gamelog-run895b-greatsahara-aztec-diff2-24k-trace.txt",
        "rontrace-run895b.log",
    ),
    length: 17554,
    // Item 2975's lobby, held by the battery's measure to `3b6c4a2a`:
    // count 9130, sequence 9130 (ours 90 draws, theirs 87, at 23: ours
    // `Unit::think_scout+0x436`, theirs `Unit::think_scout+0x64c`). Item
    // 3361 promoted it to the AI word (`docs/AI.md` §282).
    count: 9130,
    sequence: 9130,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AZTEC_WORD: i64 = if AZTEC.count < AZTEC.sequence {
    AZTEC.count
} else {
    AZTEC.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn aztec_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AZTEC.start, AZTEC.long) else {
        return;
    };
    eprintln!(
        "AZTEC {}: count {} sequence {} of {}; {}",
        AZTEC.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, AZTEC.difficulty, "the lobby read back");
    pin_eq!(w.last, AZTEC.length, "the game's last frame");
    pin_eq!(w.count, AZTEC.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        AZTEC.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn aztec_word_is_no_other_lobby() {
    let p = &AZTEC;
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

/// run989b: the word 9130's fog, blocks 9128..9130 at `WORLD=6` (item
/// 3361). The long is `cover=0` and run895 dumps blocks 0..36, so no dump
/// on disk printed the `seen2` plane `think_scout`'s cell filter reads.
pub(crate) const RUN989B: &str =
    "gamelog-run989b-greatsahara-aztec-diff2-window-9128-9131-world6.txt";

/// run989c: the Horse Archer `1/45`'s birth, blocks 8653..8656 at
/// `UNITS=3`, `GUYS=4`, `BUILDS=7` and `WORLD=6` (item 3361): the frame
/// ours first parted the fog on (8655), walked back from the word.
pub(crate) const RUN989C: &str =
    "gamelog-run989c-greatsahara-aztec-diff2-window-8653-8657-world6.txt";

/// Ours' `seen2` against a window's `WORLD` records, block by block: the
/// half-cells that part on each, as `(fx, fy, ours, theirs)`.
fn fog_parted(window: &str) -> Option<Vec<(i64, Vec<(i32, i32, u8, u8)>)>> {
    let inst = crate::testenv::install()?;
    let (start, tr, win) = (dump(AZTEC.start)?, trace(AZTEC.long.1)?, dump(window)?);
    let loaded = crate::load::load(&inst).unwrap();
    let start_text = crate::capture::read(start);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let text = crate::capture::read(win);
    let log = Log::parse(&text);
    let mut out = Vec::new();
    for (n, block) in log.frames() {
        while built.sim.frame < n {
            built.tick();
        }
        let Some(world) = block.kid("WORLD") else {
            continue;
        };
        let theirs = crate::gamelog::world_fog(&world.fields().to_vec());
        let w = &built.sim.world;
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        assert_eq!(theirs.len(), (fw * fh) as usize, "block {n}'s fog plane");
        let parted = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .map(|(x, y)| (x, y, w.seen2(x, y).unwrap_or(0), theirs[(y * fw + x) as usize]))
            .filter(|&(_, _, a, b)| a != b)
            .collect();
        out.push((n, parted));
    }
    Some(out)
}

/// **The fog plane, walked back from the word** (item 3361): ours'
/// `seen2` against run989c's over the Horse Archer `1/45`'s birth and
/// run989b's on the word's own block, half-cell by half-cell. Before item
/// 3361 ours lit fog (117, 55) and (119, 53) for who=1 on 8655, when
/// `1/45` came out of its trainer at (44184, 19704) with no gather point —
/// a whole disc at radius 4 (`mylos` 9 on both sides) that the original's
/// `come_out` never writes — and on 9130 `think_scout`'s cell filter
/// refused cell (58, 27) as seen where theirs scored it (`+0x64c`).
#[test]
fn run989_s_fog_agrees_over_the_birth_and_on_the_word() {
    let _pins = Pins::hold();
    for window in [RUN989C, RUN989B] {
        let Some(blocks) = fog_parted(window) else {
            return;
        };
        for (n, parted) in &blocks {
            if !parted.is_empty() {
                eprintln!("{window} block {n}: {parted:?}");
            }
        }
        let counts: Vec<(i64, usize)> = blocks.iter().map(|(n, p)| (*n, p.len())).collect();
        pin_eq!(
            counts.iter().filter(|(_, c)| *c > 0).count(),
            0,
            "{window}: blocks whose fog parts ({counts:?})"
        );
    }
}
