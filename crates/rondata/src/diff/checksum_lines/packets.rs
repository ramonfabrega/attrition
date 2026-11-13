//! **Step P: the twelve state lines on every packet on disk** (item 4188,
//! `docs/CHECKSUM.md` §23; `docs/DECISIONS.md` entry 129 §5). Each
//! `frame-snapshot.bin` under `~/ron-golden` is the original's memory at
//! one frame boundary of a captured game. The instrument's stream of each
//! (`tools/recomp/checksum.py --stream`, kept outside git under
//! `RON_PACKETS_DIR`) is held range by range against our walk's twelve
//! lines on the same frame of the same game: the word's own walk
//! ([`super::super::third::stand_up_from`], the state every word on the
//! scoreboard walks from) ticked to the packet's logger frame.

use super::*;
use crate::diff::third::stand_up_from;

/// The twelve state lines, in `CheckSums::check_all@00936560`'s order.
pub(super) const STATE_LINES: [&str; 12] = [
    "units", "builds", "walls", "ammo", "deaths", "groups", "guys", "leaders", "cities", "items",
    "goods", "world",
];

/// The directory the instrument's streams of the packets are kept in, one
/// `<run>/stream.tsv` and `<run>/values.tsv` a packet.
fn packets_dir() -> String {
    std::env::var("RON_PACKETS_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/lab-experiments/2026-11-12-item-4188")
    })
}

/// The instrument's twelve state lines of one packet, and the logger frame
/// its values file names. `None` on a machine without them.
fn packet_stream(run: &str) -> Option<(i64, std::collections::BTreeMap<String, StreamRows>)> {
    let dir = packets_dir();
    let values = std::fs::read_to_string(format!("{dir}/{run}/values.tsv")).ok()?;
    let frame = values
        .lines()
        .next()?
        .strip_prefix("# packet logger frame ")?
        .split(',')
        .next()?
        .trim()
        .parse()
        .ok()?;
    let text = std::fs::read_to_string(format!("{dir}/{run}/stream.tsv")).ok()?;
    Some((frame, stream_lines(&text, |l| STATE_LINES.contains(&l))))
}

/// One line's outcome on one packet: the stream's ranges, ours, the ranges
/// that part, and the first parting `(range, record, field, ours, theirs)`.
#[derive(Clone, Debug)]
pub(super) struct LineOutcome {
    pub theirs: usize,
    pub ours: usize,
    pub parted: usize,
    pub first: Option<(usize, i16, String, String, String)>,
}

impl LineOutcome {
    /// A line with a range on either side is counted (`docs/DECISIONS.md`
    /// entry 120 §2); it agrees when no range parts.
    pub fn counted(&self) -> bool {
        self.theirs > 0 || self.ours > 0
    }
    pub fn agrees(&self) -> bool {
        self.counted() && self.parted == 0
    }
}

/// The data layer the builds line reads its pieces from, read once a walk.
pub(super) struct Pieces {
    look: crate::artdata::BuildLook,
    tribes: Vec<crate::TribeDef>,
}

impl Pieces {
    pub fn read() -> Option<Pieces> {
        let inst = crate::testenv::install()?;
        let loaded = crate::load::load(&inst).unwrap();
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
        Some(Pieces { look, tribes })
    }
}

/// Our walk's twelve lines, in [`STATE_LINES`]' order. The walls, ammo and
/// deaths lines have no emitter in the crate and are empty.
pub(super) fn our_lines(sim: &sim::Sim, pieces: &Pieces) -> Vec<Line> {
    let p = build_pieces_with(sim, &pieces.look, &pieces.tribes);
    vec![
        sim.checksum_units(),
        sim.checksum_builds(&|b| p.get(&b).copied().unwrap_or(-1)),
        Line::default(),
        Line::default(),
        Line::default(),
        sim.checksum_groups(),
        sim.checksum_guys(),
        sim.checksum_leaders(),
        sim.checksum_cities(),
        sim.checksum_items(),
        sim.checksum_goods(),
        sim.checksum_world(),
    ]
}

/// The twelve lines held range by range against a packet's stream.
pub(super) fn compare(
    ours: &[Line],
    theirs: &std::collections::BTreeMap<String, StreamRows>,
) -> Vec<LineOutcome> {
    let empty = StreamRows::new();
    STATE_LINES
        .iter()
        .zip(ours)
        .map(|(name, o)| {
            let t = theirs.get(*name).unwrap_or(&empty);
            let p = parting(o, t);
            LineOutcome {
                theirs: t.len(),
                ours: o.ranges.len(),
                parted: p.len(),
                first: p.into_iter().next(),
            }
        })
        .collect()
}

/// One game our walk holds: a word on the scoreboard — its name, its start
/// sibling, its long capture and the length its word is pinned whole to.
pub(super) struct Game {
    pub name: &'static str,
    pub start: &'static str,
    pub long: (&'static str, &'static str),
    pub length: i64,
}

/// A packet's pinned outcome: `(run, logger frame, each line's (the stream's
/// ranges, ours, ranges parted) in [`STATE_LINES`]' order)`, pinned as found
/// first — a regression gate, not a target.
pub(super) type Pinned = (&'static str, i64, [(usize, usize, usize); 12]);

/// One packet walked: its run, frame, the twelve outcomes and the leaders
/// line's leaf rows.
pub(super) struct Walked {
    pub run: &'static str,
    pub frame: i64,
    pub lines: Vec<LineOutcome>,
    pub leaves: Vec<LeafRow>,
}

/// Walk `game` from its frame zero to each of `packets`' frames (ascending)
/// and hold the twelve lines there against the packet's stream. `None`,
/// saying why, on a machine without the captures, the install or a stream.
pub(super) fn walk_packets(game: &Game, packets: &[(&'static str, i64)]) -> Option<Vec<Walked>> {
    walk_packets_with(game.name, game.length, packets, &mut |frames, probe| {
        let (mut built, _) = stand_up_from(game.start, game.long)?;
        for &f in frames {
            assert!(built.sim.frame <= f, "the packets ascend");
            while built.sim.frame < f {
                built.tick();
            }
            probe(f, &built);
        }
        Some(())
    })
}

/// A walk that hands its probe the state on each of the frames asked for.
type Walker<'a> = dyn FnMut(&[i64], &mut dyn FnMut(i64, &crate::diff::Built)) -> Option<()> + 'a;

/// [`walk_packets`] on any walk: `walk` stands its game up and hands the
/// probe the state on each packet's frame, ascending.
pub(super) fn walk_packets_with(
    name: &str,
    length: i64,
    packets: &[(&'static str, i64)],
    walk: &mut Walker<'_>,
) -> Option<Vec<Walked>> {
    let mut streams = Vec::new();
    for &(run, frame) in packets {
        let Some((logged, s)) = packet_stream(run) else {
            eprintln!(
                "skipping {name}: no instrument stream for {run} ({}/{run}, RON_PACKETS_DIR)",
                packets_dir()
            );
            return None;
        };
        assert_eq!(logged, frame, "{run}: the packet's logger frame");
        assert!(
            frame <= length,
            "{run}: frame {frame} past {name}'s pinned length {length}"
        );
        streams.push((run, frame, s));
    }
    let pieces = Pieces::read()?;
    let frames: Vec<i64> = streams.iter().map(|s| s.1).collect();
    let mut out = Vec::new();
    let mut at = 0;
    walk(&frames, &mut |f, built| {
        let (run, frame, theirs) = &streams[at];
        assert_eq!(
            (f, built.sim.frame),
            (*frame, *frame),
            "{run}: the walk's frame"
        );
        at += 1;
        let lines = compare(&our_lines(&built.sim, &pieces), theirs);
        let empty = StreamRows::new();
        let leaves = leader_leaf_rows(&built.sim, theirs.get("leaders").unwrap_or(&empty));
        out.push(Walked {
            run,
            frame: *frame,
            lines,
            leaves,
        });
    })?;
    Some(out)
}

/// One chapter's packets held: `golden` the chapter's capture under
/// `RON_GOLDEN_DIR`, `stem` its script, `length` the frame its word is
/// closed to.
pub(super) fn hold_chapter(golden: &str, stem: &str, length: i64, packets: &[(&'static str, i64)]) {
    let Some(walked) = walk_packets_with(stem, length, packets, &mut |frames, probe| {
        crate::diff::golden::walk_script_to(golden, stem, frames, probe)
    }) else {
        return;
    };
    check(stem, &walked);
}

/// The twelve outcomes as a pin's array.
fn pin_of(lines: &[LineOutcome]) -> [(usize, usize, usize); 12] {
    let mut a = [(0, 0, 0); 12];
    for (slot, l) in a.iter_mut().zip(lines) {
        *slot = (l.theirs, l.ours, l.parted);
    }
    a
}

/// Hold one game's walked packets against [`PINNED`] and
/// [`PACKET_LEAF_PARTINGS`]: each line's ranges and partings, and every leaf
/// of the leaders line that parts outside [`LEADERS_HELD_OUT`].
/// `RON_PACKETS_PRINT` prints the pins and every leaf row.
pub(super) fn hold(game: &Game, packets: &[(&'static str, i64)]) {
    let Some(walked) = walk_packets(game, packets) else {
        return;
    };
    check(game.name, &walked);
}

/// The leaves of a packet's leaders line that part outside the held-out
/// list, on any slot: their held-out names, sorted, space-separated.
fn extra_leaves(w: &Walked, held: &std::collections::BTreeSet<&str>) -> String {
    let names: std::collections::BTreeSet<String> = w
        .leaves
        .iter()
        .filter(|(_, _, _, ours, t, _)| ours.as_ref() != Some(t))
        .map(|(_, field, leaf, ours, _, _)| leader_held_name(field, leaf, ours.is_none()))
        .filter(|n| !held.contains(n.as_str()))
        .collect();
    names.into_iter().collect::<Vec<_>>().join(" ")
}

/// [`hold`]'s comparison of walked packets against the pins.
pub(super) fn check(name: &str, walked: &[Walked]) {
    let print = std::env::var_os("RON_PACKETS_PRINT").is_some();
    let held: std::collections::BTreeSet<&str> = LEADERS_HELD_OUT.iter().map(|(n, _)| *n).collect();
    for w in walked {
        let agree = w.lines.iter().filter(|l| l.agrees()).count();
        let counted = w.lines.iter().filter(|l| l.counted()).count();
        eprintln!(
            "{} {} f{}: {agree} of {counted} counted lines agree",
            name, w.run, w.frame
        );
        for (n, l) in STATE_LINES.iter().zip(&w.lines) {
            if let Some(f) = &l.first {
                let cut = |s: &str| s.chars().take(64).collect::<String>();
                eprintln!(
                    "  {n}: {} of {} ranges part; first range {} record {} {}: ours {} theirs {}",
                    l.parted,
                    l.theirs,
                    f.0,
                    f.1,
                    f.2,
                    cut(&f.3),
                    cut(&f.4)
                );
            }
        }
        if print {
            eprintln!("PIN\t(\"{}\", {}, {:?}),", w.run, w.frame, pin_of(&w.lines));
            for (slot, field, leaf, ours, t, _) in &w.leaves {
                let verdict = match ours {
                    None => "unheld",
                    Some(o) if o == t => "agree",
                    Some(_) => "part",
                };
                eprintln!(
                    "LEAF\t{}\t{}\t{slot}\t{}\t{verdict}\t{}\t{}",
                    w.run,
                    w.frame,
                    leader_held_name(field, leaf, false),
                    leaf_text(t),
                    ours.as_deref().map_or("-".into(), leaf_text)
                );
            }
            eprintln!("EXTRA\t(\"{}\", \"{}\"),", w.run, extra_leaves(w, &held));
            // Printing is the measurement, every packet of the walk: the
            // pins are read off it, so they are not held here.
            continue;
        }
        let pinned = PINNED
            .iter()
            .find(|p| p.0 == w.run)
            .unwrap_or_else(|| panic!("{}: no pinned outcome", w.run));
        assert_eq!(
            (pinned.1, pinned.2),
            (w.frame, pin_of(&w.lines)),
            "{} {}: each line's (the stream's ranges, ours, ranges parted)",
            name,
            w.run
        );
        let want = PACKET_LEAF_PARTINGS
            .iter()
            .find(|p| p.0 == w.run)
            .map_or("", |p| p.1);
        assert_eq!(
            extra_leaves(w, &held),
            want,
            "{} {}: the leaders line's leaves parting outside LEADERS_HELD_OUT",
            name,
            w.run
        );
        hold_released(w, &held);
    }
}

/// A leaf row's original value is its seated image: all 0, or all −1 for
/// the two `last_*_finished` arrays (`Leader::init`'s fill).
fn at_init_image(leaf: &str, theirs: &[i64]) -> bool {
    let init = if leaf.starts_with("last_") { -1 } else { 0 };
    theirs.iter().all(|&v| v == init)
}

/// **Entry 129 §5's witnesses on one walked packet** (item 4187): each
/// [`PACKET_RELEASED`] leaf whose witness is this packet holds a value off
/// its seated image there, ours equal to it — and bent on our side, it
/// joins the leaves parting outside the list (the comparison can fail).
fn hold_released(w: &Walked, held: &std::collections::BTreeSet<&str>) {
    for &(leaf, run, slot) in PACKET_RELEASED {
        if run != w.run {
            continue;
        }
        let row = w
            .leaves
            .iter()
            .find(|r| r.0 == slot && r.1 == "who..ally_mask" && r.2 == leaf)
            .unwrap_or_else(|| panic!("{run}: slot {slot}'s {leaf}"));
        assert!(
            !at_init_image(leaf, &row.4),
            "{run}: slot {slot}'s {leaf} is its seated image, no witness"
        );
        assert_eq!(row.3.as_ref(), Some(&row.4), "{run}: slot {slot}'s {leaf}");
        let mut bent = Walked {
            run: w.run,
            frame: w.frame,
            lines: w.lines.clone(),
            leaves: w.leaves.clone(),
        };
        for r in bent.leaves.iter_mut() {
            if r.0 == slot && r.1 == "who..ally_mask" && r.2 == leaf {
                r.3.as_mut().unwrap()[0] ^= 1;
            }
        }
        assert!(
            extra_leaves(&bent, held).split(' ').any(|n| n == leaf),
            "{run}: a bent {leaf} parts"
        );
    }
}

/// **The unprinted leaves released by `docs/DECISIONS.md` entry 129 §5**
/// (item 4187, `docs/LEADERS.md` §18, §19): each agrees on every held packet
/// (it parts on none, so no [`PACKET_LEAF_PARTINGS`] entry names it), and
/// `(leaf, run, slot)` is a packet on which the original's value is off its
/// seated image — the witness [`hold_released`] holds.
pub(super) const PACKET_RELEASED: &[(&str, &str, i16)] = &[
    // run958b, frame 375, leader 1: cell 3 holds 2006, the rest −1.
    ("last_building_finished", "run958b", 1),
    // run958b, frame 375, leader 1: cell 0 (the Citizen) holds 7.
    ("last_unit_finished", "run958b", 1),
    // run1161, frame 4, leader 0: 123.
    ("score_territory", "run1161", 0),
    // run1161, frame 4, leader 0: 68.
    ("score_buildings", "run1161", 0),
    // run1161, frame 4, leader 0: 6.
    ("score_economy", "run1161", 0),
    // run938b, frame 630, leader 1: 208.
    ("score_unit_upgrades", "run938b", 1),
    // run1161, frame 4, leader 1: 56.
    ("score_research", "run1161", 1),
    // run1487, frame 1980, leader 2: 666.
    ("score_wonders", "run1487", 2),
];

/// Every [`PACKET_RELEASED`] witness is a held packet, no pinned parting
/// names a released leaf, and every released leaf is off the held-out list.
#[test]
fn the_packet_releases_are_witnessed() {
    let held: Vec<&str> = every_held().iter().map(|h| h.0).collect();
    for &(leaf, run, _) in PACKET_RELEASED {
        assert!(held.contains(&run), "{leaf}'s witness {run} is held");
        assert!(
            !PACKET_LEAF_PARTINGS
                .iter()
                .any(|p| p.1.split(' ').any(|n| n == leaf)),
            "{leaf} parts on no packet"
        );
        assert!(
            !LEADERS_HELD_OUT
                .iter()
                .any(|(n, _)| n.trim_end_matches(" (unheld)") == leaf),
            "{leaf} left the held-out list"
        );
    }
}

/// The games our walk holds and their packets, and one test a game: the
/// word's own walk ticked to each packet's frame (ascending) and held there.
macro_rules! held_packets {
    ($($test:ident: $row:path => [$(($run:literal, $frame:literal)),* $(,)?];)*) => {
        /// Every held packet: `(run, logger frame, the game's name, the
        /// length its word is pinned whole to)`.
        pub(super) const HELD: &[(&str, i64, &str, i64)] =
            &[$($(($run, $frame, $row.name, $row.length)),*,)*];
        $(
            #[test]
            fn $test() {
                let g = Game {
                    name: $row.name,
                    start: $row.start,
                    long: $row.long,
                    length: $row.length,
                };
                hold(&g, &[$(($run, $frame)),*]);
            }
        )*
    };
}

held_packets! {
    american_enlightenment_pair_packets: crate::diff::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT => [("run1325c", 5578)];
    aztec_ancient_pair_packets: crate::diff::aztec_ancient_pair::AZTEC_ANCIENT => [("run1266c", 8700)];
    bantu_ancient_pair_packets: crate::diff::bantu_ancient_pair::BANTU_ANCIENT => [("run1236", 4179), ("run1238c", 5021)];
    british_classical_pair_packets: crate::diff::british_classical_pair::BRITISH_CLASSICAL => [("run1326", 4567)];
    british_enlightenment_pair_packets: crate::diff::british_enlightenment_pair::BRITISH_ENLIGHTENMENT => [("run1185", 4582)];
    german_word_packets: crate::diff::german_word::GERMAN => [("run893b", 3975), ("run912e", 12753)];
    greek_classical_pair_packets: crate::diff::greek_classical_pair::GREEK_CLASSICAL => [("run1217", 7728)];
    indian_enlightenment_pair_packets: crate::diff::indian_enlightenment_pair::INDIAN_ENLIGHTENMENT => [("run1324c", 4904)];
    iroquois_enlightenment_pair_packets: crate::diff::iroquois_enlightenment_pair::IROQUOIS_ENLIGHTENMENT => [("run1320b", 5066)];
    japanese_ancient_pair_packets: crate::diff::japanese_ancient_pair::JAPANESE_ANCIENT => [("run1264b", 4919)];
    japanese_industrial_pair_packets: crate::diff::japanese_industrial_pair::JAPANESE_INDUSTRIAL => [("run1169", 3769)];
    korean_ancient_pair_packets: crate::diff::korean_ancient_pair::KOREAN_ANCIENT => [("run1221", 4287)];
    lakota_medieval_pair_packets: crate::diff::lakota_medieval_pair::LAKOTA_MEDIEVAL => [("run1235b", 4127)];
    lakota_pair_packets: crate::diff::lakota_pair::LAKOTA => [("run854", 3301), ("run865", 4859), ("run882b", 6808), ("run883b", 6821)];
    maya_ancient_pair_packets: crate::diff::maya_ancient_pair::MAYA_ANCIENT => [("run1247", 4306)];
    mongol_pair_packets: crate::diff::mongol_pair::MONGOL => [("run785", 2178), ("run795", 2681), ("run789", 2799), ("run810", 3631), ("run825", 4204), ("run823", 4238)];
    mongol_gunpowder_pair_packets: crate::diff::mongol_gunpowder_pair::MONGOL_GUNPOWDER => [("run1294", 4473), ("run1296", 4632)];
    mongol_medieval_pair_packets: crate::diff::mongol_medieval_pair::MONGOL_MEDIEVAL => [("run1391", 4174)];
    persian_gunpowder_pair_packets: crate::diff::persian_gunpowder_pair::PERSIAN_GUNPOWDER => [("run1383", 1180), ("run1385p", 2775)];
    roman_medieval_pair_packets: crate::diff::roman_medieval_pair::ROMAN_MEDIEVAL => [("run1293", 3606)];
    roman_word_packets: crate::diff::roman_word::ROMAN => [("run959", 18368)];
    russian_modern_pair_packets: crate::diff::russian_modern_pair::RUSSIAN_MODERN => [("run1161", 4), ("run1165", 3719), ("run1166", 3920)];
    spanish_gunpowder_pair_packets: crate::diff::spanish_gunpowder_pair::SPANISH_GUNPOWDER => [("run1304", 3865)];
    spanish_modern_pair_packets: crate::diff::spanish_modern_pair::SPANISH_MODERN => [("run1416", 4016)];
    american_information_pair_packets: crate::diff::american_information_pair::AMERICAN_INFORMATION => [("run1437", 3894)];
    aztec_modern_pair_packets: crate::diff::aztec_modern_pair::AZTEC_MODERN => [("run1441", 3294), ("run1559b", 3294), ("run1564", 3294), ("run1570", 3331)];
    chinese_enlightenment_pair_packets: crate::diff::chinese_enlightenment_pair::CHINESE_ENLIGHTENMENT => [("run1381", 3262)];
    egyptian_information_pair_packets: crate::diff::egyptian_information_pair::EGYPTIAN_INFORMATION => [("run1419b", 3212)];
    egyptian_modern_pair_packets: crate::diff::egyptian_modern_pair::EGYPTIAN_MODERN => [("run1426b", 3722)];
    german_industrial_pair_packets: crate::diff::german_industrial_pair::GERMAN_INDUSTRIAL => [("run1351c", 3521), ("run1351", 4303)];
    german_modern_pair_packets: crate::diff::german_modern_pair::GERMAN_MODERN => [("run1417", 4016), ("run1420", 4586)];
    iroquois_word_packets: crate::diff::iroquois_word::IROQUOIS => [("run943c", 10674), ("run946", 10842), ("run954b", 12155)];
    japanese_modern_pair_packets: crate::diff::japanese_modern_pair::JAPANESE_MODERN => [("run1424", 3156)];
    korean_medieval_pair_packets: crate::diff::korean_medieval_pair::KOREAN_MEDIEVAL => [("run1332a", 3627)];
    mongol_enlightenment_pair_packets: crate::diff::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT => [("run1361", 4840)];
    persian_classical_pair_packets: crate::diff::persian_classical_pair::PERSIAN_CLASSICAL => [("run1327", 10930)];
    persian_word_packets: crate::diff::persian_word::PERSIAN => [("run984b", 6011), ("run984c", 6251)];
    russian_gunpowder_pair_packets: crate::diff::russian_gunpowder_pair::RUSSIAN_GUNPOWDER => [("run1357", 3439)];
    spanish_industrial_pair_packets: crate::diff::spanish_industrial_pair::SPANISH_INDUSTRIAL => [("run1382", 3228)];
    american_industrial_pair_packets: crate::diff::american_industrial_pair::AMERICAN_INDUSTRIAL => [("run1359", 5619), ("run1359b", 5677), ("run1359c", 5678)];
    american_modern_pair_packets: crate::diff::american_modern_pair::AMERICAN_MODERN => [("run1164", 4528)];
    british_industrial_pair_packets: crate::diff::british_industrial_pair::BRITISH_INDUSTRIAL => [("run1355", 2933)];
    chinese_industrial_pair_packets: crate::diff::chinese_industrial_pair::CHINESE_INDUSTRIAL => [("run1155c", 3003)];
    chinese_medieval_pair_packets: crate::diff::chinese_medieval_pair::CHINESE_MEDIEVAL => [("run1220b", 2976)];
    egyptian_gunpowder_pair_packets: crate::diff::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER => [("run1254c", 3115)];
    french_word_packets: crate::diff::french_word::FRENCH => [("run985b", 16762)];
    inca_enlightenment_pair_packets: crate::diff::inca_enlightenment_pair::INCA_ENLIGHTENMENT => [("run1323b", 3052)];
    inca_sahara_word_packets: crate::diff::inca_sahara_word::INCA_SAHARA => [("run1011b", 13105), ("run1011c", 13225), ("run1011d", 13277), ("run1011e", 13296), ("run1015b", 13392)];
    indian_sahara_word_packets: crate::diff::indian_sahara_word::INDIAN_SAHARA => [("run1007b", 15915)];
    iroquois_sahara_word_packets: crate::diff::iroquois_sahara_word::IROQUOIS_SAHARA => [("run1001d", 13760)];
    japanese_gunpowder_pair_packets: crate::diff::japanese_gunpowder_pair::JAPANESE_GUNPOWDER => [("run1237", 2674)];
    korean_classical_pair_packets: crate::diff::korean_classical_pair::KOREAN_CLASSICAL => [("run1263", 8362)];
    korean_word_packets: crate::diff::korean_word::KOREAN => [("run991d", 15256)];
    lakota_gunpowder_pair_packets: crate::diff::lakota_gunpowder_pair::LAKOTA_GUNPOWDER => [("run1305b", 4181)];
    maya_word_packets: crate::diff::maya_word::MAYA => [("run973g", 12286)];
    mongol_classical_pair_packets: crate::diff::mongol_classical_pair::MONGOL_CLASSICAL => [("run1224b", 9745)];
    russian_medieval_pair_packets: crate::diff::russian_medieval_pair::RUSSIAN_MEDIEVAL => [("run1268", 3347)];
    spanish_word_packets: crate::diff::spanish_word::SPANISH => [("run958b", 375)];
    turkish_enlightenment_pair_packets: crate::diff::turkish_enlightenment_pair::TURKISH_ENLIGHTENMENT => [("run1186b", 1377)];
}

/// The minted game's packet, frame 91 of run 1562's playback: our walk is
/// the minted walk ([`ours_at_packet`]), the one every line's own test reads.
pub(super) const MINTED: (&str, i64) = ("run1562", 91);

/// **The minted packet among the others**: run 1562's frame 91 held by the
/// same comparison as every other packet, from the same directory.
#[test]
fn minted_packet() {
    let Some((logged, theirs)) = packet_stream(MINTED.0) else {
        eprintln!(
            "skipping: no instrument stream for {} (RON_PACKETS_DIR)",
            MINTED.0
        );
        return;
    };
    assert_eq!(logged, MINTED.1, "the packet's logger frame");
    let (Some(sim), Some(pieces)) = (ours_at_packet(), Pieces::read()) else {
        eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
        return;
    };
    let lines = compare(&our_lines(&sim, &pieces), &theirs);
    let empty = StreamRows::new();
    let leaves = leader_leaf_rows(&sim, theirs.get("leaders").unwrap_or(&empty));
    check(
        "the minted game",
        &[Walked {
            run: MINTED.0,
            frame: MINTED.1,
            lines,
            leaves,
        }],
    );
}

/// The golden chapters our walk holds and their packets, one test a
/// chapter: the chapter's own walk (`diff::golden`'s `walk_script`, the
/// Leader AI silenced and the script staged) to each packet's frame. Each
/// packet's `rontrace.cmd` is the chapter's script up to its frame.
macro_rules! held_chapters {
    ($($test:ident: $golden:literal, $stem:literal, $length:literal => [$(($run:literal, $frame:literal)),* $(,)?];)*) => {
        /// Every chapter packet: `(run, logger frame, the chapter's script,
        /// the frame its word is closed to)`.
        pub(super) const CHAPTERS_HELD: &[(&str, i64, &str, i64)] =
            &[$($(($run, $frame, $stem, $length)),*,)*];
        $(
            #[test]
            fn $test() {
                hold_chapter($golden, $stem, $length, &[$(($run, $frame)),*]);
            }
        )*
    };
}

held_chapters! {
    chapter64_packets: "ch64", "chapter64", 820 => [("run839", 634)];
    chapter65_packets: "ch65", "chapter65", 1100 => [("run842b", 776), ("run842", 777)];
    chapter75_packets: "ch75", "chapter75", 960 => [("run870", 901)];
    chapter94_packets: "ch94", "chapter94", 1200 => [("ch94p", 846)];
    chapter95_packets: "ch95", "chapter95", 1200 => [("ch95pkt", 1117)];
    chapter100_packets: "ch100", "chapter100", 2600 => [("run938b", 630)];
}

/// One melee's packets held: the word's stand-up inside the melee's staged
/// script (`gunpowder_melee::seat_walk`'s walk: the script runs through the
/// tick), to each packet's frame.
pub(super) fn hold_melee(
    name: &str,
    script: fn() -> crate::golden::Script,
    start: &str,
    long: (&str, &str),
    length: i64,
    packets: &[(&'static str, i64)],
) {
    let Some(walked) = walk_packets_with(name, length, packets, &mut |frames, probe| {
        crate::diff::setup::with_staged_script(script(), || {
            let (mut built, _) = stand_up_from(start, long)?;
            for &f in frames {
                while built.sim.frame < f {
                    built.tick();
                }
                probe(f, &built);
            }
            Some(())
        })
    }) else {
        return;
    };
    check(name, &walked);
}

/// The melees our walk holds and their packets, one test a melee. Each
/// packet's `rontrace.cmd` is the melee's script.
macro_rules! held_melees {
    ($($test:ident: $m:ident, $row:ident => [$(($run:literal, $frame:literal)),* $(,)?];)*) => {
        /// Every melee packet: `(run, logger frame, the melee, the length
        /// its word is whole to)`.
        pub(super) const MELEES_HELD: &[(&str, i64, &str, i64)] =
            &[$($(($run, $frame, stringify!($m), crate::diff::$m::$row.length)),*,)*];
        $(
            #[test]
            fn $test() {
                use crate::diff::$m as m;
                hold_melee(
                    stringify!($m),
                    m::melee_script,
                    m::MELEE_START,
                    m::MELEE_LONG,
                    m::$row.length,
                    &[$(($run, $frame)),*],
                );
            }
        )*
    };
}

held_melees! {
    gunpowder_melee_packets: gunpowder_melee, GUNPOWDER_MELEE =>
        [("run1438", 684), ("run1440", 809), ("run1451", 945)];
    classical_melee_packets: classical_melee, CLASSICAL_MELEE =>
        [("run1457", 700), ("run1460", 760), ("run1459", 780)];
    information_melee_packets: information_melee, INFORMATION_MELEE =>
        [("run1469", 627), ("run1469b", 628), ("run1472", 707)];
    sea_melee_packets: sea_melee, SEA_MELEE => [("run1484", 676), ("run1487", 1980)];
    airbase_melee_packets: airbase_melee, AIRBASE_MELEE => [("run1496", 1539)];
}

/// **Every packet our walk does not hold**, with why: `(run, logger frame,
/// why)` (`docs/CHECKSUM.md` §23.1).
pub(super) const NOT_HELD: &[(&str, i64, &str)] = &[(
    "run1100",
    5408,
    "run1090's lobby with `100 ally 1` and `5404 explore all` staged; no word walks it",
)];

/// The census's counts: `(packets on disk, held by a word's walk, by a
/// chapter's, by a melee's, by the minted walk, not held)`.
const CENSUS: (usize, usize, usize, usize, usize, usize) = (108, 87, 7, 12, 1, 1);

/// Every held packet's `(run, logger frame, game, the length its walk is
/// whole to)`: a word's, a chapter's and the minted one.
fn every_held() -> Vec<(&'static str, i64, &'static str, i64)> {
    HELD.iter()
        .copied()
        .chain(CHAPTERS_HELD.iter().copied())
        .chain(MELEES_HELD.iter().copied())
        .chain(std::iter::once((
            MINTED.0,
            MINTED.1,
            "the minted game",
            24_000,
        )))
        .collect()
}

/// **Every packet on disk is wired to its game and frame, or named**
/// (item 4188, `docs/CHECKSUM.md` §23.1): the held, the minted and the
/// not-held name each run once, a held packet's frame is inside its word's
/// pinned length, and — on a machine with them — the packets under
/// `RON_GOLDEN_DIR` are exactly these runs, each instrument values file
/// naming the frame the table does.
#[test]
fn every_packet_is_wired_to_its_game_or_named() {
    let held = every_held();
    let mut runs: Vec<&str> = held.iter().map(|h| h.0).collect();
    runs.extend(NOT_HELD.iter().map(|n| n.0));
    let mut sorted = runs.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), runs.len(), "a run named twice");
    assert_eq!(
        (
            runs.len(),
            HELD.len(),
            CHAPTERS_HELD.len(),
            MELEES_HELD.len(),
            1,
            NOT_HELD.len()
        ),
        CENSUS,
        "the census's counts"
    );
    for (run, frame, game, length) in &held {
        assert!(frame <= length, "{run}: {frame} past {game}'s {length}");
    }
    let frames: std::collections::BTreeMap<&str, i64> = held
        .iter()
        .map(|h| (h.0, h.1))
        .chain(NOT_HELD.iter().map(|n| (n.0, n.1)))
        .collect();
    let root = std::env::var("RON_GOLDEN_DIR")
        .unwrap_or_else(|_| format!("{}/ron-golden", std::env::var("HOME").unwrap_or_default()));
    let Ok(dirs) = std::fs::read_dir(&root) else {
        eprintln!("skipping the disk's half: no {root} (RON_GOLDEN_DIR)");
        return;
    };
    let mut on_disk: Vec<String> = dirs
        .flatten()
        .filter(|d| {
            std::fs::read_dir(d.path()).is_ok_and(|maps| {
                maps.flatten().any(|m| {
                    m.file_name().to_string_lossy().starts_with("map-")
                        && m.path().join("frame-snapshot.bin").is_file()
                })
            })
        })
        .map(|d| d.file_name().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();
    assert_eq!(
        on_disk.iter().map(String::as_str).collect::<Vec<_>>(),
        sorted,
        "the packets on disk are the census's"
    );
    let mut read = 0;
    for (run, frame) in &frames {
        if let Some((logged, _)) = packet_stream(run) {
            assert_eq!(logged, *frame, "{run}: the values file's logger frame");
            read += 1;
        }
    }
    eprintln!(
        "{} packets on disk, {read} with the instrument's stream",
        on_disk.len()
    );
}

/// **The score**: `(packets compared, lines agreeing, lines counted)` summed
/// over [`PINNED`], a line counted when either side walks a range of it
/// (`docs/DECISIONS.md` entry 120 §2). The queue's `Checksum:` clause.
pub(crate) const PACKETS_SCORE: (usize, usize, usize) = (107, 147, 1058);

/// The packets on disk, the census's first count: the `Checksum:` line's
/// `of` (`diff::floors::checksum_line`).
pub(crate) const PACKETS_ON_DISK: usize = CENSUS.0;

/// The pins' sum is [`PACKETS_SCORE`], and every held packet has its pin.
#[test]
fn the_packets_score_is_the_pins() {
    let mut runs: Vec<&str> = every_held().iter().map(|h| h.0).collect();
    runs.sort_unstable();
    let mut pinned: Vec<&str> = PINNED.iter().map(|p| p.0).collect();
    pinned.sort_unstable();
    assert_eq!(pinned, runs, "a pin a held packet");
    let (mut agree, mut counted) = (0, 0);
    for (_, _, lines) in PINNED {
        for &(theirs, ours, parted) in lines {
            if theirs > 0 || ours > 0 {
                counted += 1;
                agree += usize::from(parted == 0);
            }
        }
    }
    assert_eq!(
        (PINNED.len(), agree, counted),
        PACKETS_SCORE,
        "the packets' score"
    );
}

/// **The comparison can part** (item 4188): on run1161's packet (the
/// Russians at a held Modern age, East Indies, logger frame 4 — the
/// cheapest walk on disk) one bent byte of our items line parts on that
/// range alone; our walk one frame early or late is not the packet's
/// outcome and agrees on no more lines; and the same game's later packet,
/// run1165's frame 3719, against our frame 4 agrees on fewer.
#[test]
fn a_packet_s_comparison_can_part() {
    use crate::diff::russian_modern_pair::RUSSIAN_MODERN as G;
    let (Some((_, theirs)), Some((_, later))) =
        (packet_stream("run1161"), packet_stream("run1165"))
    else {
        eprintln!("skipping: no instrument stream for run1161 or run1165 (RON_PACKETS_DIR)");
        return;
    };
    let Some(pieces) = Pieces::read() else {
        return;
    };
    let Some((mut built, _)) = stand_up_from(G.start, G.long) else {
        return;
    };
    let agreeing = |o: &[LineOutcome]| o.iter().filter(|l| l.agrees()).count();
    let mut at = std::collections::BTreeMap::new();
    while built.sim.frame < 5 {
        built.tick();
        if built.sim.frame >= 3 {
            at.insert(built.sim.frame, our_lines(&built.sim, &pieces));
        }
    }
    let four = compare(&at[&4], &theirs);
    let pinned = PINNED
        .iter()
        .find(|p| p.0 == "run1161")
        .expect("run1161's pin");
    assert_eq!(pin_of(&four), pinned.2, "frame 4 is the packet's outcome");
    for f in [3, 5] {
        let off = compare(&at[&f], &theirs);
        assert_ne!(pin_of(&off), pinned.2, "frame {f} is not the packet's");
        assert!(
            agreeing(&off) <= agreeing(&four),
            "frame {f} agrees on more"
        );
    }
    let items = STATE_LINES.iter().position(|l| *l == "items").unwrap();
    assert!(four[items].agrees(), "the items line agrees on frame 4");
    let mut bent = at[&4].clone();
    bent[items].ranges[7].bytes[0] ^= 1;
    let p = parting(&bent[items], &theirs["items"]);
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        [7],
        "the bent range alone"
    );
    let wrong = compare(&at[&4], &later);
    assert!(
        agreeing(&wrong) < agreeing(&four),
        "run1165's frame 3719 against our frame 4: {} lines agree",
        agreeing(&wrong)
    );
}

/// The pinned outcomes, one a packet, pinned as found first (item 4188).
pub(super) const PINNED: &[Pinned] = &[
    (
        "ch94p",
        846,
        [
            (1116, 1296, 309),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (16, 0, 16),
            (538, 538, 0),
            (382, 445, 99),
            (271, 271, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (7996, 7277, 4488),
        ],
    ),
    (
        "ch95pkt",
        1117,
        [
            (1284, 1426, 1224),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (10, 0, 10),
            (538, 538, 0),
            (417, 473, 92),
            (271, 271, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (7994, 7275, 4482),
        ],
    ),
    (
        "run1001d",
        13760,
        [
            (2445, 2672, 1998),
            (2028, 2028, 49),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (544, 544, 0),
            (701, 806, 106),
            (271, 271, 2),
            (30, 30, 3),
            (5, 110, 107),
            (120, 120, 0),
            (7581, 7565, 4436),
        ],
    ),
    (
        "run1007b",
        15915,
        [
            (2939, 3438, 2660),
            (2417, 2417, 58),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (916, 1126, 215),
            (271, 271, 2),
            (30, 30, 3),
            (45, 110, 83),
            (120, 120, 0),
            (7537, 7521, 4313),
        ],
    ),
    (
        "run1011b",
        13105,
        [
            (3219, 3345, 2625),
            (2060, 2060, 49),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 0),
            (870, 975, 105),
            (271, 271, 3),
            (30, 30, 3),
            (50, 110, 77),
            (120, 120, 0),
            (7479, 7463, 4220),
        ],
    ),
    (
        "run1011c",
        13225,
        [
            (3193, 3298, 2619),
            (2080, 2080, 47),
            (0, 0, 0),
            (6, 0, 6),
            (4, 0, 4),
            (538, 538, 0),
            (861, 966, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 110, 77),
            (120, 120, 0),
            (7537, 7521, 4366),
        ],
    ),
    (
        "run1011d",
        13277,
        [
            (3158, 3250, 2530),
            (2080, 2080, 47),
            (0, 0, 0),
            (12, 0, 12),
            (4, 0, 4),
            (538, 538, 0),
            (861, 966, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 110, 77),
            (120, 120, 0),
            (7553, 7537, 4393),
        ],
    ),
    (
        "run1011e",
        13296,
        [
            (3117, 3189, 2465),
            (2080, 2080, 47),
            (0, 0, 0),
            (9, 0, 9),
            (6, 0, 6),
            (538, 538, 0),
            (854, 959, 105),
            (271, 271, 3),
            (30, 30, 3),
            (50, 110, 77),
            (120, 120, 0),
            (7517, 7501, 4316),
        ],
    ),
    (
        "run1015b",
        13392,
        [
            (3139, 3183, 2492),
            (2080, 2080, 46),
            (0, 0, 0),
            (27, 0, 27),
            (4, 0, 4),
            (538, 538, 0),
            (847, 952, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 110, 77),
            (120, 120, 0),
            (7419, 7403, 4096),
        ],
    ),
    (
        "run1155c",
        3003,
        [
            (4193, 4310, 3311),
            (2371, 2371, 68),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (550, 550, 2),
            (1129, 1234, 106),
            (271, 271, 3),
            (27, 27, 2),
            (75, 110, 56),
            (120, 120, 0),
            (7583, 7567, 4490),
        ],
    ),
    (
        "run1161",
        4,
        [
            (2760, 2965, 535),
            (520, 520, 13),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (526, 526, 0),
            (924, 994, 174),
            (263, 263, 4),
            (6, 6, 0),
            (35, 35, 0),
            (330, 330, 0),
            (8850, 7263, 4860),
        ],
    ),
    (
        "run1164",
        4528,
        [
            (6053, 5910, 5313),
            (2140, 2136, 820),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (550, 550, 2),
            (1321, 1475, 158),
            (271, 271, 6),
            (30, 30, 3),
            (75, 110, 56),
            (120, 120, 0),
            (7597, 7581, 4525),
        ],
    ),
    (
        "run1165",
        3719,
        [
            (6004, 6418, 5320),
            (2267, 2266, 445),
            (0, 0, 0),
            (287, 0, 287),
            (0, 0, 0),
            (676, 676, 3),
            (1890, 2051, 263),
            (271, 271, 4),
            (30, 30, 3),
            (15, 35, 25),
            (330, 330, 2),
            (9290, 7703, 6027),
        ],
    ),
    (
        "run1166",
        3920,
        [
            (6004, 6310, 5126),
            (2247, 2246, 508),
            (0, 0, 0),
            (0, 0, 0),
            (10, 0, 10),
            (688, 688, 3),
            (1935, 2061, 229),
            (271, 271, 7),
            (30, 30, 3),
            (15, 35, 25),
            (330, 330, 2),
            (9262, 7675, 5955),
        ],
    ),
    (
        "run1169",
        3769,
        [
            (7749, 8143, 6789),
            (1813, 1813, 87),
            (0, 0, 0),
            (12, 0, 12),
            (0, 0, 0),
            (628, 628, 7),
            (1934, 2095, 277),
            (271, 271, 4),
            (27, 27, 3),
            (20, 35, 22),
            (330, 330, 2),
            (9438, 7851, 6516),
        ],
    ),
    (
        "run1185",
        4582,
        [
            (6735, 7099, 6040),
            (1911, 1911, 73),
            (0, 0, 0),
            (3, 0, 3),
            (4, 0, 4),
            (640, 640, 9),
            (1962, 2123, 268),
            (271, 271, 4),
            (27, 27, 2),
            (5, 35, 31),
            (330, 330, 2),
            (9268, 7681, 6012),
        ],
    ),
    (
        "run1186b",
        1377,
        [
            (1060, 1345, 381),
            (1164, 1164, 23),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (309, 407, 98),
            (271, 271, 3),
            (9, 9, 0),
            (105, 110, 25),
            (120, 120, 0),
            (7361, 7345, 3930),
        ],
    ),
    (
        "run1217",
        7728,
        [
            (10607, 10674, 9119),
            (3412, 3412, 141),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (700, 700, 12),
            (3247, 3436, 299),
            (271, 271, 4),
            (56, 56, 5),
            (0, 35, 35),
            (330, 330, 1),
            (9486, 7899, 6514),
        ],
    ),
    (
        "run1220b",
        2976,
        [
            (4091, 4338, 3359),
            (1817, 1817, 43),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (550, 550, 2),
            (1099, 1274, 175),
            (271, 271, 3),
            (27, 27, 2),
            (90, 110, 44),
            (120, 120, 0),
            (7663, 7647, 4673),
        ],
    ),
    (
        "run1221",
        4287,
        [
            (6445, 6949, 5946),
            (1719, 1719, 44),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (592, 592, 5),
            (1768, 1992, 339),
            (271, 271, 4),
            (33, 33, 3),
            (10, 35, 29),
            (330, 330, 1),
            (9162, 7575, 5806),
        ],
    ),
    (
        "run1224b",
        9745,
        [
            (11740, 11409, 10463),
            (4271, 4271, 126),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (556, 556, 1),
            (2735, 2910, 182),
            (271, 271, 6),
            (62, 62, 7),
            (30, 110, 91),
            (120, 120, 0),
            (7873, 7857, 4963),
        ],
    ),
    (
        "run1235b",
        4127,
        [
            (7490, 7627, 6531),
            (1733, 1733, 57),
            (0, 0, 0),
            (12, 0, 12),
            (4, 0, 4),
            (646, 646, 10),
            (2097, 2188, 197),
            (271, 271, 4),
            (30, 30, 2),
            (15, 35, 26),
            (330, 330, 0),
            (9352, 7765, 6266),
        ],
    ),
    (
        "run1236",
        4179,
        [
            (6610, 7349, 5833),
            (1762, 1762, 58),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (658, 658, 7),
            (2070, 2329, 392),
            (271, 271, 4),
            (44, 44, 4),
            (15, 35, 25),
            (330, 330, 0),
            (9312, 7725, 6216),
        ],
    ),
    (
        "run1237",
        2674,
        [
            (3448, 3542, 2712),
            (1650, 1650, 35),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 1),
            (892, 990, 98),
            (271, 271, 3),
            (27, 27, 1),
            (85, 110, 48),
            (120, 120, 0),
            (7703, 7687, 4645),
        ],
    ),
    (
        "run1238c",
        5021,
        [
            (6417, 7136, 5611),
            (2494, 2494, 58),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (646, 646, 7),
            (2151, 2410, 366),
            (271, 271, 4),
            (49, 49, 4),
            (10, 35, 29),
            (330, 330, 0),
            (9318, 7731, 6179),
        ],
    ),
    (
        "run1247",
        4306,
        [
            (5942, 6521, 5260),
            (1692, 1692, 41),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (622, 622, 7),
            (1748, 1972, 325),
            (271, 271, 4),
            (38, 38, 3),
            (10, 35, 29),
            (330, 330, 0),
            (9158, 7571, 5816),
        ],
    ),
    (
        "run1254c",
        3115,
        [
            (4388, 4364, 3447),
            (1674, 1674, 54),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (550, 550, 0),
            (1134, 1239, 107),
            (271, 271, 3),
            (27, 27, 2),
            (85, 110, 49),
            (120, 120, 0),
            (7669, 7653, 4540),
        ],
    ),
    (
        "run1263",
        8362,
        [
            (11229, 10998, 9898),
            (3789, 3789, 109),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (556, 556, 1),
            (2913, 3123, 215),
            (271, 271, 4),
            (54, 54, 5),
            (50, 110, 77),
            (120, 120, 1),
            (7867, 7851, 5107),
        ],
    ),
    (
        "run1264b",
        4919,
        [
            (6468, 7014, 5960),
            (1770, 1770, 57),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (586, 586, 8),
            (1895, 2119, 344),
            (271, 271, 3),
            (43, 43, 4),
            (15, 35, 25),
            (330, 330, 1),
            (9330, 7743, 6143),
        ],
    ),
    (
        "run1266c",
        8700,
        [
            (11161, 11622, 10287),
            (3796, 3796, 114),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (682, 682, 11),
            (3210, 3413, 324),
            (271, 271, 4),
            (54, 54, 5),
            (5, 35, 31),
            (330, 330, 0),
            (9704, 8117, 6778),
        ],
    ),
    (
        "run1268",
        3347,
        [
            (4744, 4798, 3764),
            (1908, 1908, 65),
            (0, 0, 0),
            (24, 0, 24),
            (4, 0, 4),
            (544, 544, 0),
            (1246, 1386, 141),
            (271, 271, 4),
            (30, 30, 3),
            (90, 110, 42),
            (120, 120, 0),
            (7773, 7757, 4828),
        ],
    ),
    (
        "run1293",
        3606,
        [
            (9512, 9862, 8627),
            (1938, 1938, 64),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (586, 586, 4),
            (2191, 2345, 265),
            (271, 271, 4),
            (25, 25, 2),
            (20, 35, 22),
            (330, 330, 1),
            (9230, 7643, 5878),
        ],
    ),
    (
        "run1294",
        4473,
        [
            (8216, 8852, 7275),
            (2123, 2123, 74),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (688, 688, 12),
            (2242, 2473, 338),
            (271, 271, 8),
            (27, 27, 3),
            (10, 35, 28),
            (330, 330, 0),
            (9490, 7903, 6488),
        ],
    ),
    (
        "run1296",
        4632,
        [
            (9905, 10501, 9382),
            (2163, 2163, 80),
            (0, 0, 0),
            (0, 0, 0),
            (6, 0, 6),
            (682, 682, 11),
            (2172, 2403, 340),
            (271, 271, 8),
            (27, 27, 3),
            (5, 35, 32),
            (330, 330, 0),
            (9470, 7883, 6497),
        ],
    ),
    (
        "run1304",
        3865,
        [
            (5488, 5902, 4705),
            (2029, 2029, 78),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (616, 616, 4),
            (1616, 1784, 269),
            (271, 271, 4),
            (23, 23, 2),
            (15, 35, 25),
            (330, 330, 1),
            (9448, 7861, 6596),
        ],
    ),
    (
        "run1305b",
        4181,
        [
            (4968, 4832, 4109),
            (1882, 1882, 53),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1327, 1397, 70),
            (271, 271, 3),
            (35, 35, 3),
            (70, 110, 62),
            (120, 120, 0),
            (7585, 7569, 4445),
        ],
    ),
    (
        "run1320b",
        5066,
        [
            (8490, 8757, 7447),
            (2140, 2140, 78),
            (0, 0, 0),
            (9, 0, 9),
            (2, 0, 2),
            (652, 652, 10),
            (2197, 2323, 235),
            (271, 271, 5),
            (39, 39, 3),
            (5, 35, 31),
            (330, 330, 4),
            (9476, 7889, 6506),
        ],
    ),
    (
        "run1323b",
        3052,
        [
            (3933, 4098, 3225),
            (1792, 1792, 44),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (544, 544, 0),
            (1029, 1162, 133),
            (271, 271, 3),
            (27, 27, 2),
            (90, 110, 44),
            (120, 120, 0),
            (7615, 7599, 4535),
        ],
    ),
    (
        "run1324c",
        4904,
        [
            (7276, 7557, 6190),
            (1918, 1918, 87),
            (0, 0, 0),
            (15, 0, 15),
            (2, 0, 2),
            (676, 676, 9),
            (2055, 2188, 242),
            (271, 271, 4),
            (27, 27, 2),
            (20, 35, 21),
            (330, 330, 4),
            (9466, 7879, 6370),
        ],
    ),
    (
        "run1325c",
        5578,
        [
            (10367, 11043, 9354),
            (2521, 2521, 93),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (682, 682, 13),
            (2706, 2965, 373),
            (271, 271, 4),
            (43, 43, 5),
            (10, 35, 27),
            (330, 330, 1),
            (9666, 8079, 6675),
        ],
    ),
    (
        "run1326",
        4567,
        [
            (6268, 6911, 5540),
            (2161, 2161, 86),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (616, 616, 9),
            (1993, 2224, 341),
            (271, 271, 4),
            (30, 30, 2),
            (15, 35, 26),
            (330, 330, 2),
            (9244, 7657, 5995),
        ],
    ),
    (
        "run1327",
        10930,
        [
            (13393, 13427, 11988),
            (4230, 4230, 113),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (574, 574, 2),
            (3459, 3774, 361),
            (271, 271, 4),
            (59, 59, 6),
            (25, 110, 94),
            (180, 180, 0),
            (8592, 7873, 5834),
        ],
    ),
    (
        "run1332a",
        3627,
        [
            (5746, 5891, 4421),
            (2354, 2354, 75),
            (0, 0, 0),
            (48, 0, 48),
            (14, 0, 14),
            (580, 580, 5),
            (1463, 1638, 211),
            (271, 271, 3),
            (30, 30, 3),
            (80, 110, 54),
            (180, 180, 0),
            (8244, 7525, 5179),
        ],
    ),
    (
        "run1351",
        4303,
        [
            (4745, 4928, 3951),
            (2237, 2237, 74),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (562, 562, 2),
            (1352, 1492, 179),
            (271, 271, 3),
            (30, 30, 3),
            (75, 110, 56),
            (180, 180, 1),
            (8328, 7609, 5258),
        ],
    ),
    (
        "run1351c",
        3521,
        [
            (4512, 4732, 3134),
            (2143, 2143, 75),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (556, 556, 3),
            (1286, 1426, 176),
            (271, 271, 4),
            (30, 30, 2),
            (75, 110, 56),
            (180, 180, 1),
            (8298, 7579, 5257),
        ],
    ),
    (
        "run1355",
        2933,
        [
            (3610, 3841, 2961),
            (1836, 1836, 45),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (556, 556, 4),
            (902, 1042, 140),
            (271, 271, 3),
            (27, 27, 2),
            (80, 110, 51),
            (120, 120, 0),
            (7777, 7761, 4777),
        ],
    ),
    (
        "run1357",
        3439,
        [
            (5707, 5937, 4835),
            (1977, 1977, 59),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 2),
            (1427, 1602, 211),
            (271, 271, 4),
            (30, 30, 3),
            (75, 110, 59),
            (180, 180, 1),
            (8528, 7809, 5790),
        ],
    ),
    (
        "run1359",
        5619,
        [
            (6464, 6432, 5647),
            (2974, 2974, 108),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 3),
            (1584, 1724, 144),
            (271, 271, 5),
            (48, 48, 4),
            (60, 110, 69),
            (120, 120, 0),
            (8007, 7991, 5220),
        ],
    ),
    (
        "run1359b",
        5677,
        [
            (6408, 6424, 5583),
            (2974, 2974, 110),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 4),
            (1598, 1738, 144),
            (271, 271, 5),
            (48, 48, 4),
            (55, 110, 74),
            (120, 120, 0),
            (7717, 7701, 4791),
        ],
    ),
    (
        "run1359c",
        5678,
        [
            (6407, 6423, 5580),
            (2974, 2974, 110),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 4),
            (1598, 1738, 144),
            (271, 271, 5),
            (48, 48, 4),
            (55, 110, 74),
            (120, 120, 0),
            (7715, 7699, 4789),
        ],
    ),
    (
        "run1361",
        4840,
        [
            (6810, 6671, 5784),
            (2544, 2544, 72),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (556, 556, 1),
            (1847, 1952, 141),
            (271, 271, 8),
            (37, 37, 3),
            (50, 110, 78),
            (180, 180, 2),
            (8476, 7757, 5586),
        ],
    ),
    (
        "run1381",
        3262,
        [
            (5194, 5195, 3939),
            (2147, 2147, 64),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (556, 556, 2),
            (1419, 1517, 134),
            (271, 271, 3),
            (27, 27, 3),
            (90, 110, 44),
            (180, 180, 1),
            (8334, 7615, 5331),
        ],
    ),
    (
        "run1382",
        3228,
        [
            (5014, 5130, 4171),
            (2015, 2015, 67),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 0),
            (1349, 1482, 169),
            (271, 271, 4),
            (20, 20, 1),
            (70, 110, 61),
            (180, 180, 0),
            (8340, 7621, 5320),
        ],
    ),
    (
        "run1383",
        1180,
        [
            (2933, 3253, 650),
            (1184, 1184, 21),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 1),
            (967, 1079, 213),
            (271, 271, 3),
            (6, 6, 0),
            (30, 35, 11),
            (330, 330, 0),
            (8876, 7289, 4946),
        ],
    ),
    (
        "run1385p",
        2775,
        [
            (5193, 5463, 4596),
            (1774, 1774, 42),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 2),
            (1609, 1756, 247),
            (271, 271, 4),
            (27, 27, 2),
            (20, 35, 21),
            (330, 330, 0),
            (9252, 7665, 5784),
        ],
    ),
    (
        "run1391",
        4174,
        [
            (6512, 6830, 5822),
            (1830, 1830, 73),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (694, 694, 17),
            (1927, 2088, 265),
            (271, 271, 8),
            (35, 35, 3),
            (10, 35, 28),
            (330, 330, 1),
            (9408, 7821, 6277),
        ],
    ),
    (
        "run1416",
        4016,
        [
            (6953, 7259, 6119),
            (2128, 2127, 678),
            (0, 0, 0),
            (18, 0, 18),
            (0, 0, 0),
            (634, 634, 10),
            (1829, 1955, 240),
            (271, 271, 4),
            (33, 33, 2),
            (5, 35, 31),
            (330, 330, 2),
            (9554, 7967, 6698),
        ],
    ),
    (
        "run1417",
        4016,
        [
            (4518, 4705, 3767),
            (2321, 2321, 75),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1265, 1405, 178),
            (271, 271, 4),
            (33, 33, 3),
            (75, 110, 54),
            (180, 180, 1),
            (8274, 7555, 5208),
        ],
    ),
    (
        "run1419b",
        3212,
        [
            (4406, 4486, 3414),
            (2033, 2033, 75),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 2),
            (1231, 1336, 141),
            (271, 271, 4),
            (27, 27, 2),
            (60, 110, 69),
            (180, 180, 0),
            (8434, 7715, 5388),
        ],
    ),
    (
        "run1420",
        4586,
        [
            (5583, 5614, 4645),
            (2515, 2513, 751),
            (0, 0, 0),
            (0, 0, 0),
            (8, 0, 8),
            (544, 544, 1),
            (1327, 1467, 177),
            (271, 271, 4),
            (38, 38, 3),
            (70, 110, 59),
            (180, 180, 0),
            (8466, 7747, 5536),
        ],
    ),
    (
        "run1424",
        3156,
        [
            (4129, 4318, 3389),
            (2025, 2025, 48),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 3),
            (1138, 1278, 177),
            (271, 271, 4),
            (27, 27, 3),
            (90, 110, 44),
            (180, 180, 0),
            (8344, 7625, 5328),
        ],
    ),
    (
        "run1426b",
        3722,
        [
            (4175, 4291, 3481),
            (2266, 2265, 861),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (544, 544, 1),
            (1227, 1332, 146),
            (271, 271, 4),
            (27, 27, 3),
            (80, 110, 52),
            (180, 180, 0),
            (8284, 7565, 5190),
        ],
    ),
    (
        "run1437",
        3894,
        [
            (4789, 4721, 3966),
            (2154, 2151, 619),
            (0, 0, 0),
            (3, 0, 3),
            (4, 0, 4),
            (538, 538, 0),
            (1208, 1348, 183),
            (271, 271, 6),
            (30, 30, 3),
            (85, 110, 49),
            (180, 180, 0),
            (8414, 7695, 5513),
        ],
    ),
    (
        "run1438",
        684,
        [
            (2475, 2702, 2286),
            (1890, 1890, 26),
            (0, 0, 0),
            (15, 0, 15),
            (0, 0, 0),
            (574, 574, 2),
            (672, 784, 113),
            (399, 399, 6),
            (9, 9, 0),
            (135, 135, 0),
            (165, 165, 0),
            (7499, 7483, 4263),
        ],
    ),
    (
        "run1440",
        809,
        [
            (2308, 2539, 2166),
            (1968, 1968, 25),
            (0, 0, 0),
            (18, 0, 18),
            (14, 0, 14),
            (568, 568, 2),
            (638, 757, 120),
            (399, 399, 6),
            (9, 9, 0),
            (130, 135, 28),
            (165, 165, 0),
            (7513, 7497, 4347),
        ],
    ),
    (
        "run1441",
        3294,
        [
            (4274, 4219, 3527),
            (1890, 1887, 527),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 100),
            (271, 271, 4),
            (30, 30, 3),
            (75, 110, 55),
            (180, 180, 0),
            (8276, 7557, 5229),
        ],
    ),
    (
        "run1451",
        945,
        [
            (2022, 2281, 1870),
            (1968, 1968, 29),
            (0, 0, 0),
            (18, 0, 18),
            (32, 0, 32),
            (568, 568, 2),
            (604, 723, 120),
            (399, 399, 6),
            (9, 9, 0),
            (125, 135, 37),
            (165, 165, 0),
            (7585, 7569, 4477),
        ],
    ),
    (
        "run1457",
        700,
        [
            (2655, 2718, 2286),
            (1144, 1144, 28),
            (0, 0, 0),
            (21, 0, 21),
            (2, 0, 2),
            (574, 574, 2),
            (725, 788, 64),
            (399, 399, 4),
            (9, 9, 0),
            (130, 135, 30),
            (165, 165, 0),
            (7525, 7509, 4362),
        ],
    ),
    (
        "run1459",
        780,
        [
            (2362, 2425, 1982),
            (1316, 1316, 26),
            (0, 0, 0),
            (0, 0, 0),
            (22, 0, 22),
            (568, 568, 2),
            (665, 728, 64),
            (403, 403, 5),
            (9, 9, 0),
            (130, 135, 30),
            (165, 165, 0),
            (7535, 7519, 4363),
        ],
    ),
    (
        "run1460",
        760,
        [
            (2380, 2455, 1949),
            (1296, 1296, 27),
            (0, 0, 0),
            (9, 0, 9),
            (16, 0, 16),
            (568, 568, 3),
            (686, 749, 64),
            (403, 403, 5),
            (9, 9, 0),
            (130, 135, 30),
            (165, 165, 0),
            (7523, 7507, 4355),
        ],
    ),
    (
        "run1469",
        627,
        [
            (1392, 1679, 1048),
            (1777, 1777, 29),
            (0, 0, 0),
            (228, 0, 228),
            (0, 0, 0),
            (556, 556, 4),
            (413, 518, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 135, 29),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    (
        "run1469b",
        628,
        [
            (1401, 1685, 1056),
            (1777, 1777, 29),
            (0, 0, 0),
            (190, 0, 190),
            (0, 0, 0),
            (556, 556, 4),
            (413, 518, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 135, 29),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    (
        "run1472",
        707,
        [
            (2180, 2393, 2009),
            (1777, 1777, 30),
            (0, 0, 0),
            (339, 0, 339),
            (4, 0, 4),
            (568, 568, 4),
            (612, 717, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 135, 30),
            (165, 165, 0),
            (7527, 7511, 4420),
        ],
    ),
    (
        "run1484",
        676,
        [
            (4145, 4206, 3495),
            (1446, 1446, 28),
            (0, 0, 0),
            (304, 0, 304),
            (0, 0, 0),
            (562, 562, 2),
            (1381, 1416, 159),
            (403, 403, 6),
            (9, 9, 0),
            (25, 45, 28),
            (405, 405, 0),
            (9053, 7519, 5588),
        ],
    ),
    (
        "run1487",
        1980,
        [
            (5340, 5621, 4718),
            (2736, 2736, 66),
            (0, 0, 0),
            (208, 0, 208),
            (8, 0, 8),
            (610, 610, 2),
            (1718, 1844, 247),
            (407, 407, 6),
            (31, 31, 0),
            (25, 45, 28),
            (405, 405, 2),
            (9267, 7733, 6310),
        ],
    ),
    (
        "run1496",
        1539,
        [
            (5026, 5286, 4896),
            (2180, 2180, 47),
            (0, 0, 0),
            (230, 0, 230),
            (2, 0, 2),
            (586, 586, 2),
            (1577, 1696, 241),
            (407, 407, 6),
            (23, 23, 0),
            (25, 45, 28),
            (405, 405, 2),
            (9183, 7649, 6125),
        ],
    ),
    (
        "run1559b",
        3294,
        [
            (4274, 4219, 3527),
            (1890, 1887, 527),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 100),
            (271, 271, 4),
            (30, 30, 3),
            (75, 110, 55),
            (180, 180, 0),
            (8276, 7557, 5229),
        ],
    ),
    (
        "run1562",
        91,
        [
            (1084, 1084, 0),
            (519, 519, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 0),
            (368, 368, 0),
            (267, 267, 2),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (7994, 7994, 0),
        ],
    ),
    (
        "run1564",
        3294,
        [
            (4274, 4219, 3527),
            (1890, 1887, 527),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 100),
            (271, 271, 4),
            (30, 30, 3),
            (75, 110, 55),
            (180, 180, 0),
            (8276, 7557, 5229),
        ],
    ),
    (
        "run1570",
        3331,
        [
            (4240, 4197, 3409),
            (1890, 1887, 528),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (556, 556, 2),
            (1112, 1175, 101),
            (271, 271, 4),
            (30, 30, 3),
            (75, 110, 55),
            (180, 180, 0),
            (8294, 7575, 5253),
        ],
    ),
    (
        "run785",
        2178,
        [
            (4969, 4983, 4166),
            (1731, 1729, 965),
            (0, 0, 0),
            (0, 0, 0),
            (28, 0, 28),
            (640, 640, 4),
            (1429, 1506, 178),
            (271, 271, 7),
            (47, 47, 4),
            (15, 35, 25),
            (330, 330, 1),
            (9108, 7521, 5669),
        ],
    ),
    (
        "run789",
        2799,
        [
            (5889, 5855, 5006),
            (2157, 2155, 1285),
            (0, 0, 0),
            (281, 0, 281),
            (0, 0, 0),
            (664, 664, 2),
            (1765, 1842, 178),
            (271, 271, 7),
            (52, 52, 5),
            (15, 35, 25),
            (330, 330, 0),
            (9526, 7939, 6468),
        ],
    ),
    (
        "run795",
        2681,
        [
            (5860, 5823, 4887),
            (2059, 2057, 1209),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 2),
            (1704, 1781, 178),
            (271, 271, 7),
            (52, 52, 5),
            (15, 35, 25),
            (330, 330, 1),
            (9420, 7833, 6400),
        ],
    ),
    (
        "run810",
        3631,
        [
            (9526, 9603, 8636),
            (2384, 2381, 1424),
            (0, 0, 0),
            (156, 0, 156),
            (0, 0, 0),
            (688, 688, 6),
            (2341, 2425, 195),
            (271, 271, 7),
            (58, 58, 5),
            (10, 35, 28),
            (330, 330, 1),
            (9442, 7855, 6428),
        ],
    ),
    (
        "run823",
        4238,
        [
            (9339, 9413, 8439),
            (2473, 2469, 1560),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (676, 676, 6),
            (2450, 2534, 195),
            (271, 271, 7),
            (58, 58, 5),
            (10, 35, 28),
            (330, 330, 0),
            (9552, 7965, 6484),
        ],
    ),
    (
        "run825",
        4204,
        [
            (9362, 9436, 8403),
            (2473, 2469, 1560),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (688, 688, 6),
            (2471, 2555, 194),
            (271, 271, 7),
            (58, 58, 5),
            (10, 35, 28),
            (330, 330, 0),
            (9560, 7973, 6527),
        ],
    ),
    (
        "run839",
        634,
        [
            (1201, 1312, 1056),
            (479, 479, 13),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (526, 526, 0),
            (403, 445, 78),
            (267, 267, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (8020, 7301, 4558),
        ],
    ),
    (
        "run842",
        777,
        [
            (1149, 1303, 1028),
            (583, 583, 14),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (532, 532, 0),
            (393, 449, 92),
            (271, 271, 4),
            (9, 9, 0),
            (110, 110, 0),
            (180, 180, 0),
            (8004, 7285, 4510),
        ],
    ),
    (
        "run842b",
        776,
        [
            (1154, 1308, 1028),
            (583, 583, 14),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (532, 532, 0),
            (393, 449, 92),
            (271, 271, 4),
            (9, 9, 0),
            (110, 110, 0),
            (180, 180, 0),
            (8004, 7285, 4510),
        ],
    ),
    (
        "run854",
        3301,
        [
            (5770, 5833, 4677),
            (2095, 2095, 73),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (640, 640, 8),
            (1794, 1878, 184),
            (271, 271, 5),
            (36, 36, 0),
            (5, 35, 32),
            (330, 330, 2),
            (9344, 7757, 6056),
        ],
    ),
    (
        "run865",
        4859,
        [
            (10342, 10233, 9575),
            (2288, 2283, 1566),
            (0, 0, 0),
            (0, 0, 0),
            (28, 0, 28),
            (640, 640, 6),
            (2210, 2294, 185),
            (271, 271, 5),
            (64, 64, 4),
            (0, 35, 35),
            (330, 330, 2),
            (9534, 7947, 6461),
        ],
    ),
    (
        "run870",
        901,
        [
            (1636, 1776, 294),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (578, 627, 85),
            (271, 271, 4),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (8004, 7285, 4510),
        ],
    ),
    (
        "run882b",
        6808,
        [
            (11125, 10562, 9845),
            (2737, 2732, 1950),
            (0, 0, 0),
            (15, 0, 15),
            (0, 0, 0),
            (616, 616, 10),
            (2832, 2923, 224),
            (271, 271, 5),
            (64, 64, 7),
            (0, 35, 35),
            (330, 330, 2),
            (9518, 7931, 6568),
        ],
    ),
    (
        "run883b",
        6821,
        [
            (11180, 10557, 9936),
            (2737, 2732, 1951),
            (0, 0, 0),
            (36, 0, 36),
            (0, 0, 0),
            (616, 616, 10),
            (2832, 2923, 222),
            (271, 271, 5),
            (64, 64, 7),
            (0, 35, 35),
            (330, 330, 2),
            (9536, 7949, 6589),
        ],
    ),
    (
        "run893b",
        3975,
        [
            (2652, 2987, 625),
            (709, 709, 20),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (900, 1012, 212),
            (271, 271, 3),
            (9, 9, 0),
            (30, 35, 11),
            (330, 330, 0),
            (8966, 7379, 5253),
        ],
    ),
    (
        "run912e",
        12753,
        [
            (4112, 4387, 3652),
            (1508, 1508, 40),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 5),
            (1352, 1464, 217),
            (271, 271, 3),
            (27, 27, 3),
            (5, 35, 32),
            (330, 330, 0),
            (9122, 7535, 5619),
        ],
    ),
    (
        "run938b",
        630,
        [
            (1118, 1235, 925),
            (479, 479, 13),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (526, 526, 0),
            (375, 417, 78),
            (267, 267, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (7994, 7275, 4481),
        ],
    ),
    (
        "run943c",
        10674,
        [
            (2726, 3048, 2294),
            (1580, 1580, 34),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (819, 959, 176),
            (271, 271, 3),
            (27, 27, 2),
            (40, 110, 84),
            (180, 180, 0),
            (8150, 7431, 4899),
        ],
    ),
    (
        "run946",
        10842,
        [
            (2838, 3119, 2310),
            (1580, 1580, 33),
            (0, 0, 0),
            (15, 0, 15),
            (0, 0, 0),
            (532, 532, 0),
            (826, 966, 176),
            (271, 271, 3),
            (27, 27, 2),
            (40, 110, 84),
            (180, 180, 0),
            (8164, 7445, 4942),
        ],
    ),
    (
        "run954b",
        12155,
        [
            (2832, 3140, 2431),
            (1598, 1598, 36),
            (0, 0, 0),
            (9, 0, 9),
            (2, 0, 2),
            (532, 532, 0),
            (875, 1015, 180),
            (271, 271, 3),
            (27, 27, 3),
            (35, 110, 87),
            (180, 180, 0),
            (8096, 7377, 4771),
        ],
    ),
    (
        "run958b",
        375,
        [
            (490, 550, 108),
            (455, 455, 15),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (160, 181, 23),
            (267, 267, 3),
            (6, 6, 0),
            (110, 110, 0),
            (120, 120, 0),
            (7291, 7275, 3748),
        ],
    ),
    (
        "run959",
        18368,
        [
            (4466, 4844, 3954),
            (1832, 1832, 48),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (562, 562, 5),
            (1514, 1668, 268),
            (271, 271, 3),
            (17, 17, 2),
            (0, 35, 35),
            (330, 330, 1),
            (9056, 7469, 5567),
        ],
    ),
    (
        "run973g",
        12286,
        [
            (3135, 3263, 2550),
            (2052, 2052, 46),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (532, 532, 0),
            (941, 1046, 106),
            (271, 271, 2),
            (35, 35, 4),
            (60, 110, 69),
            (120, 120, 0),
            (7513, 7497, 4325),
        ],
    ),
    (
        "run984b",
        6011,
        [
            (2003, 2230, 1758),
            (789, 789, 20),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (533, 638, 141),
            (271, 271, 2),
            (17, 17, 1),
            (75, 110, 55),
            (180, 180, 0),
            (8052, 7333, 4648),
        ],
    ),
    (
        "run984c",
        6251,
        [
            (1903, 2165, 1626),
            (827, 827, 21),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 0),
            (540, 645, 141),
            (271, 271, 2),
            (17, 17, 1),
            (75, 110, 55),
            (180, 180, 0),
            (8078, 7359, 4721),
        ],
    ),
    (
        "run985b",
        16762,
        [
            (3289, 3742, 3058),
            (2783, 2783, 54),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1012, 1222, 211),
            (271, 271, 2),
            (33, 33, 3),
            (40, 110, 85),
            (120, 120, 0),
            (7443, 7427, 4174),
        ],
    ),
    (
        "run991d",
        15256,
        [
            (3178, 3504, 2770),
            (2006, 2006, 47),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (961, 1136, 179),
            (271, 271, 2),
            (30, 30, 3),
            (55, 110, 74),
            (120, 120, 0),
            (7461, 7445, 4143),
        ],
    ),
];

/// The leaves of the leaders line that part on a packet outside
/// [`LEADERS_HELD_OUT`], on any slot: `(run, the leaves)`, a packet with
/// none absent. Each is a finding (`docs/CHECKSUM.md` §23.4).
pub(super) const PACKET_LEAF_PARTINGS: &[(&str, &str)] = &[
    ("ch94p", "aggression cities_built flags.leader_flags"),
    (
        "ch95pkt",
        "aggression cities_built explored flags.leader_flags",
    ),
    ("run1001d", "aggression cities_built resources_controlled"),
    ("run1007b", "aggression cities_built resources_controlled"),
    (
        "run1011b",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    ("run1011c", "aggression cities_built resources_controlled"),
    ("run1011d", "aggression cities_built resources_controlled"),
    (
        "run1011e",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    ("run1015b", "aggression cities_built resources_controlled"),
    (
        "run1155c",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1161", "aggression cities_built flags.leader_flags"),
    (
        "run1164",
        "aggression cities_built encrypt.bucket[2] encrypt.bucket[5] explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1165",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1166",
        "aggression cities_built collected encrypt.bucket[0] encrypt.bucket[1] encrypt.leftover[0] escrow explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1169",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1185",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1186b", "aggression cities_built flags.leader_flags"),
    (
        "run1217",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1220b",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1221",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1224b",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1235b",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1236",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1237",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1238c",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1247",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1254c",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1263",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1264b",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1266c",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1268",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1293",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1294",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    (
        "run1296",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    (
        "run1304",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1305b",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1320b",
        "aggression cities_built encrypt.bucket[2] flags.leader_flags resources_controlled",
    ),
    (
        "run1323b",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1324c",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1325c",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1326",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1327",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1332a",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1351",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1351c",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1355",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1357",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359b",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359c",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1361",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1381",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1382",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1383", "aggression cities_built flags.leader_flags"),
    (
        "run1385p",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1391",
        "aggression cities_built encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    (
        "run1416",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1417",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run1419b",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1420",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1424",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1426b",
        "aggression cities_built explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1437",
        "aggression cities_built encrypt.bucket[2] encrypt.bucket[5] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1438", "aggression cities_built flags.leader_flags"),
    ("run1440", "aggression cities_built flags.leader_flags"),
    (
        "run1441",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1451", "aggression cities_built flags.leader_flags"),
    ("run1457", "aggression cities_built flags.leader_flags"),
    ("run1459", "aggression cities_built flags.leader_flags"),
    ("run1460", "aggression cities_built flags.leader_flags"),
    (
        "run1469",
        "aggression cities_built flags.leader_flags gather_stamp",
    ),
    (
        "run1469b",
        "aggression cities_built flags.leader_flags gather_stamp",
    ),
    ("run1472", "aggression cities_built flags.leader_flags"),
    (
        "run1484",
        "active aggression cities_built flags.leader_flags num_units",
    ),
    (
        "run1487",
        "aggression cities_built flags.leader_flags num_units reg_unpack_merch resources_controlled",
    ),
    (
        "run1496",
        "aggression cities_built flags.leader_flags num_units reg_unpack_merch",
    ),
    (
        "run1559b",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1564",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1570",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run785",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run789",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run795",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch",
    ),
    (
        "run810",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run823",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run825",
        "aggression cities_built encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    ("run839", "aggression cities_built flags.leader_flags"),
    ("run842", "aggression cities_built flags.leader_flags"),
    ("run842b", "aggression cities_built flags.leader_flags"),
    (
        "run854",
        "cities_built diplos flags.leader_flags oil_patches.elements reg_unpack_merch resources_controlled",
    ),
    (
        "run865",
        "cities_built diplos flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    ("run870", "aggression cities_built flags.leader_flags"),
    (
        "run882b",
        "cities_built diplos flags.leader_flags oil_patches.elements reg_gatherers resources_controlled",
    ),
    (
        "run883b",
        "cities_built diplos flags.leader_flags oil_patches.elements reg_gatherers resources_controlled",
    ),
    ("run893b", "aggression cities_built flags.leader_flags"),
    (
        "run912e",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    ("run938b", "aggression cities_built flags.leader_flags"),
    (
        "run943c",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    (
        "run946",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run954b",
        "aggression cities_built flags.leader_flags resources_controlled",
    ),
    ("run958b", "aggression cities_built flags.leader_flags"),
    (
        "run959",
        "aggression cities_built flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run973g",
        "aggression cities_built reg_unpack_merch resources_controlled",
    ),
    ("run984b", "aggression cities_built"),
    ("run984c", "aggression cities_built"),
    ("run985b", "aggression cities_built resources_controlled"),
    ("run991d", "aggression cities_built resources_controlled"),
];

/// **Probe**: walks one word (`RON_PACKET_START`, `_LONG`, `_TRACE`) to a
/// packet's frame (`RON_PACKET_RUN`) and prints the twelve outcomes on the
/// frame before, the frame and the frame after.
#[test]
#[ignore = "a probe: RON_PACKET_RUN, RON_PACKET_START, RON_PACKET_LONG, RON_PACKET_TRACE"]
fn probe_one_packet() {
    let run = std::env::var("RON_PACKET_RUN").expect("RON_PACKET_RUN");
    let start = std::env::var("RON_PACKET_START").expect("RON_PACKET_START");
    let long = std::env::var("RON_PACKET_LONG").expect("RON_PACKET_LONG");
    let tr = std::env::var("RON_PACKET_TRACE").expect("RON_PACKET_TRACE");
    let (frame, theirs) = packet_stream(&run).expect("the packet's stream");
    let pieces = Pieces::read().expect("the install");
    let t0 = std::time::Instant::now();
    let (mut built, _) = stand_up_from(&start, (&long, &tr)).expect("the captures");
    eprintln!(
        "stood up at frame {} in {:?}",
        built.sim.frame,
        t0.elapsed()
    );
    let t1 = std::time::Instant::now();
    while built.sim.frame < frame - 1 {
        built.tick();
    }
    eprintln!("ticked to {} in {:?}", built.sim.frame, t1.elapsed());
    for _ in 0..3 {
        let o = compare(&our_lines(&built.sim, &pieces), &theirs);
        let agree = o.iter().filter(|x| x.agrees()).count();
        let counted = o.iter().filter(|x| x.counted()).count();
        eprintln!("frame {}: {agree} of {counted} counted", built.sim.frame);
        for (n, x) in STATE_LINES.iter().zip(&o) {
            eprintln!(
                "  {n:8} theirs {:5} ours {:5} parted {:5} first {:?}",
                x.theirs,
                x.ours,
                x.parted,
                x.first.as_ref().map(|f| (
                    f.0,
                    f.1,
                    f.2.clone(),
                    f.3.chars().take(48).collect::<String>(),
                    f.4.chars().take(48).collect::<String>()
                ))
            );
        }
        built.tick();
    }
}
