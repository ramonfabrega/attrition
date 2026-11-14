//! **Step P: the twelve state lines on every packet on disk** (item 4188,
//! `docs/CHECKSUM.md` §23; `docs/DECISIONS.md` entry 129 §5). Each
//! `frame-snapshot.bin` under `~/ron-golden` is the original's memory at
//! one frame boundary of a captured game. The instrument's stream of each
//! (`tools/recomp/checksum.py --stream`, kept outside git under
//! `RON_PACKETS_DIR`) is held range by range against our walk's twelve
//! lines on the same frame of the same game: the word's game ticked to the
//! packet's logger frame. Since item 4205 (§23.7) a word's game stands up
//! from its lobby's **built start** ([`super::super::third::stand_up_built`],
//! `docs/START.md` §7) where the draws hold lockstep from it to every
//! packet, and from the dump's seat ([`super::super::third::stand_up_from`],
//! the state the word's own floor walks from) where they do not
//! ([`SEATED_WORDS`]).

use super::*;
use crate::diff::third::stand_up_from;

/// The twelve state lines, in `CheckSums::check_all@00936560`'s order.
pub(in crate::diff) const STATE_LINES: [&str; 12] = [
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
    packet_stream_in(&packets_dir(), run)
}

/// [`packet_stream`] from another directory of `<run>/values.tsv` and
/// `<run>/stream.tsv` (item 4214's minted packets).
pub(in crate::diff) fn packet_stream_in(
    dir: &str,
    run: &str,
) -> Option<(i64, std::collections::BTreeMap<String, StreamRows>)> {
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
pub(in crate::diff) struct LineOutcome {
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
pub(in crate::diff) struct Pieces {
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
pub(in crate::diff) fn our_lines(sim: &sim::Sim, pieces: &Pieces) -> Vec<Line> {
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
pub(in crate::diff) fn compare(
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
        let (built, _) = stand_up_from(game.start, game.long)?;
        // Each run-up compares nothing: in a group's process it rides the
        // game's trunk (`harness::trunk`, item 4220).
        let mut rider = crate::diff::harness::trunk::Rider::new(built);
        for &f in frames {
            assert!(rider.state().sim.frame <= f, "the packets ascend");
            rider.advance(f);
            probe(f, rider.state());
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
    if std::env::var_os("RON_PACKETS_BUILT").is_some() {
        let Some((walked, parts)) = walk_packets_built(game, packets) else {
            return;
        };
        for w in &walked {
            let agree = w.lines.iter().filter(|l| l.agrees()).count();
            let counted = w.lines.iter().filter(|l| l.counted()).count();
            let pinned = PINNED.iter().find(|p| p.0 == w.run).map(|p| {
                let c = p.2.iter().filter(|l| l.0 > 0 || l.1 > 0).count();
                let a =
                    p.2.iter()
                        .filter(|l| (l.0 > 0 || l.1 > 0) && l.2 == 0)
                        .count();
                (a, c)
            });
            let held: std::collections::BTreeSet<&str> =
                LEADERS_HELD_OUT.iter().map(|(n, _)| *n).collect();
            eprintln!(
                "BUILT\t{}\t{}\t{}\tlockstep parts {:?}\tbuilt {agree} of {counted}\tseated {:?}\tpin {:?}\textra {}",
                game.name,
                w.run,
                w.frame,
                parts,
                pinned,
                pin_of(&w.lines),
                extra_leaves(w, &held)
            );
        }
        return;
    }
    if let Some((_, why)) = SEATED_WORDS.iter().find(|(n, _)| *n == game.name) {
        eprintln!("{}: dump-seated ({why})", game.name);
        let Some(walked) = walk_packets(game, packets) else {
            return;
        };
        check(game.name, &walked);
        return;
    }
    let Some((walked, parts)) = walk_packets_built(game, packets) else {
        return;
    };
    assert_eq!(
        parts, None,
        "{}: the built start's draws part from the long trace before the last packet \
         (a word that does not hold lockstep from the built start is named in SEATED_WORDS)",
        game.name
    );
    check(game.name, &walked);
}

/// **The words whose packets stay dump-seated** (item 4205,
/// `docs/CHECKSUM.md` §23.7): walked from the lobby's built start their
/// draw stream parts from the long trace before their first packet, so
/// the lines would be read off a walk out of step. `(the word, why)`, the
/// why the journal's parked finding. **None is left since item 4210**:
/// every word walks from the built start ([`walk_packets_built`]) and
/// holds the draws to every packet's frame. The 24 East Indies words of
/// **4205a** left at item 4208: their whales had no piece, so each kept an
/// idle variant its packet lacks and wrapped on frame 2 (`Sim::gaia_piece`,
/// `docs/START.md` §6). The five Great Lakes Modern and Information words
/// of **4205b** left at item 4210. They parted on frame 0
/// (`Unit::think_scout+0x64c` against `+0x436`) because a starting
/// citizen's step off its building lit no disc (`Sim::come_out_own_point`,
/// `docs/START.md` §7.2): run1429's four `seen2` cells, fog (101, 49),
/// (102, 50), (103, 51) and (104, 51), built 0 against the dump's 2.
pub(super) const SEATED_WORDS: &[(&str, &str)] = &[];

/// The word packets by their walk: `((words, packets) from the built
/// start, (words, packets) dump-seated)` (item 4205). ((30, 41), (29, 46))
/// at item 4205; ((54, 79), (5, 8)) at item 4208, the 24 East Indies
/// words and their 38 packets walked from the built start; **((59, 87),
/// (0, 0))** at item 4210, 4205b's five words and their 8 packets.
const BY_START: ((usize, usize), (usize, usize)) = ((59, 87), (0, 0));

/// Where a built start's draws first part from `tr`'s by frame `to`:
/// `(frame, ours, theirs, the draw, our site, theirs)`.
type BuiltParting = (i64, usize, usize, usize, String, String);

/// **The seat is named, and the built start's lockstep can fail** (item
/// 4205): every [`SEATED_WORDS`] row is a held word, the counts are
/// [`BY_START`]'s. run1149's game, which parted on frame 2 (ours 7 draws
/// theirs 4) until item 4208 gave the built start's whales their piece,
/// holds to frame 4; so does run1429's, which parted on frame 0 (ours 142
/// theirs 141, at draw 37, `Unit::think_scout+0x64c` against `+0x436`)
/// until item 4210 lit a starting citizen's disc where it steps off its
/// building. With that reveal removed the case goes red on that frame and
/// draw again, so [`hold`]'s lockstep assertion is sighted.
#[test]
fn the_seated_words_part_from_the_built_start() {
    let words: std::collections::BTreeMap<&str, usize> =
        HELD.iter().fold(Default::default(), |mut m, h| {
            *m.entry(h.2).or_default() += 1;
            m
        });
    for (w, why) in SEATED_WORDS {
        assert!(words.contains_key(w), "{w} is a held word");
        assert_eq!(*why, "4205b", "{w}");
    }
    let seated = |w: &str| SEATED_WORDS.iter().any(|(n, _)| *n == w);
    let count = |pick: bool| {
        let ws: Vec<usize> = words
            .iter()
            .filter(|(w, _)| seated(w) == pick)
            .map(|(_, n)| *n)
            .collect();
        (ws.len(), ws.iter().sum::<usize>())
    };
    assert_eq!(
        (count(false), count(true)),
        BY_START,
        "the word packets by their walk"
    );
    type Want = (i64, usize, usize, usize, &'static str, &'static str);
    let cases: [((&str, &str), Option<Want>); 2] = [
        (crate::diff::russian_modern_pair::RUSSIAN_MODERN.long, None),
        (crate::diff::aztec_modern_pair::AZTEC_MODERN.long, None),
    ];
    for (long, want) in cases {
        let Some(got) = built_parting(long, 4) else {
            return;
        };
        let got = got
            .as_ref()
            .map(|g| (g.0, g.1, g.2, g.3, g.4.as_str(), g.5.as_str()));
        assert_eq!(got, want, "{}: where the built start's draws part", long.0);
    }
}

/// A word's built start ticked to frame `to` at most, and where its draw
/// count first parts from the long trace's (`Some(None)`: the draws hold
/// to `to`). `None` on a machine without the captures.
fn built_parting(long: (&str, &str), to: i64) -> Option<Option<BuiltParting>> {
    let (mut built, tr) = crate::diff::third::stand_up_built(long)?;
    built.sim.trace.trace_phases = true;
    while built.sim.frame < to {
        built.tick();
        let (at, ours) = built.frame_sites.pop().expect("the frame's draws");
        let theirs = tr.labels(at);
        if ours.len() != theirs.len() {
            let i = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            let name = |v: &[String]| v.get(i).cloned().unwrap_or_default();
            return Some(Some((
                at,
                ours.len(),
                theirs.len(),
                i,
                name(&ours),
                name(&theirs),
            )));
        }
    }
    Some(None)
}

/// [`walk_packets`] from the lobby's **built start**
/// ([`crate::diff::third::stand_up_built`], item 4205) in place of the
/// dump's seat, and the first frame to the last packet's whose draw count
/// parts from the long trace's (`None`: the draw stream holds lockstep to
/// every packet).
pub(super) fn walk_packets_built(
    game: &Game,
    packets: &[(&'static str, i64)],
) -> Option<(Vec<Walked>, Option<i64>)> {
    let mut parts = None;
    let walked = walk_packets_with(game.name, game.length, packets, &mut |frames, probe| {
        let (mut built, tr) = crate::diff::third::stand_up_built(game.long)?;
        built.sim.trace.trace_phases = true;
        for &f in frames {
            assert!(built.sim.frame <= f, "the packets ascend");
            while built.sim.frame < f {
                built.tick();
                if let Some((at, ours)) = built.frame_sites.pop()
                    && parts.is_none()
                    && ours.len() != tr.labels(at).len()
                {
                    let theirs = tr.labels(at);
                    let i = (0..ours.len().max(theirs.len()))
                        .find(|&i| ours.get(i) != theirs.get(i))
                        .unwrap_or(0);
                    eprintln!(
                        "BUILT-PARTS\t{}\tframe {at}: ours {} theirs {} at {i}, ours {:?} theirs {:?}",
                        game.name,
                        ours.len(),
                        theirs.len(),
                        ours.get(i),
                        theirs.get(i)
                    );
                    parts = Some(at);
                }
            }
            probe(f, &built);
        }
        Some(())
    })?;
    Some((walked, parts))
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
    hold_witnesses(w, held, PACKET_RELEASED);
}

/// [`hold_released`] over any list of `(leaf, run, slot)` witnesses.
fn hold_witnesses(
    w: &Walked,
    held: &std::collections::BTreeSet<&str>,
    witnesses: &[(&str, &str, i16)],
) {
    for &(leaf, run, slot) in witnesses {
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

/// **The takes' packets** (item 4159, step VII-b2): held by a take's walk
/// (`diff::diplomacy_take`), which stages a computer's decision off the
/// record, so no scoreboard line counts them (`docs/DECISIONS.md` entry
/// 132 §1) and [`PINNED`] does not hold them. `(run, logger frame, the
/// take, the length its staged walk is whole to)`.
pub(super) const TAKES_HELD: &[(&str, i64, &str, i64)] =
    &[("run1575", 1090, "the diplomacy take (1099 computed)", 1150)];

/// The census's counts: `(packets on disk, held by a word's walk, by a
/// chapter's, by a melee's, by the minted walk, by a take's, not held)`.
// Item 4206: (108, …, 1) → (109, …, 2), run1575's packet at 1090 named.
// Item 4159, step VII-b2: run1575's packet held by its take, (…, 1, 1).
const CENSUS: (usize, usize, usize, usize, usize, usize, usize) = (109, 87, 7, 12, 1, 1, 1);

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
    runs.extend(TAKES_HELD.iter().map(|t| t.0));
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
            TAKES_HELD.len(),
            NOT_HELD.len()
        ),
        CENSUS,
        "the census's counts"
    );
    for (run, frame, game, length) in held.iter().chain(TAKES_HELD) {
        assert!(frame <= length, "{run}: {frame} past {game}'s {length}");
    }
    let frames: std::collections::BTreeMap<&str, i64> = held
        .iter()
        .map(|h| (h.0, h.1))
        .chain(TAKES_HELD.iter().map(|t| (t.0, t.1)))
        .chain(NOT_HELD.iter().map(|n| (n.0, n.1)))
        .collect();
    let root = std::env::var("RON_GOLDEN_DIR")
        .unwrap_or_else(|_| format!("{}/ron-golden", std::env::var("HOME").unwrap_or_default()));
    // The tree's packets, not the machine's: a packet another worktree
    // filed and this tree's `docs/RUNS.md` does not head is skipped and
    // named (`crate::corpus`, the pass of 2026-11-13).
    let ledger = crate::corpus::Ledger::of_tree();
    let Ok(listed) = crate::corpus::listed(root.as_ref(), ledger, crate::corpus::is_packet) else {
        eprintln!("skipping the disk's half: no {root} (RON_GOLDEN_DIR)");
        return;
    };
    let on_disk = listed.kept;
    // Step H's packets of the minted game (item 4214) are held by
    // `diff::checksum_frames`, each on a line's first parting, and are not
    // step P's: the disk is the census's and theirs, each named once.
    let minted: std::collections::BTreeSet<&str> = super::super::checksum_frames::MINTED_PACKETS
        .iter()
        .map(|p| p.0)
        .collect();
    for run in &minted {
        assert!(
            on_disk.iter().any(|d| d == run),
            "{run}: a minted packet checksum_frames holds is on disk"
        );
    }
    assert_eq!(
        on_disk
            .iter()
            .map(String::as_str)
            .filter(|d| !minted.contains(d))
            .collect::<Vec<_>>(),
        sorted,
        "the packets on disk are the census's and checksum_frames'"
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
/// 147 at item 4188; **150** at item 4205, the 41 packets of the 30 words
/// that hold lockstep from the built start walked from it: run958b's world
/// line (7,291 ranges, 3,748 parting → 0), and run1237's and run1382's
/// cities lines (27 ranges, 1 → 0; 20 ranges, 1 → 0). **153** at item
/// 4208, the 38 packets of the 24 East Indies words walked from the built
/// start once its whales carry their piece: run1161's builds line (520
/// ranges, 13 parting → 0) and world line (8,850 ranges, 4,860 → 0), and
/// run1385p's cities line (27 ranges, 2 → 0). **280 of 1,053** at item
/// 4209, the taken box's `Item::close` (`docs/GOODY.md` §4): the items line
/// agrees on 102 of 102 packets, 11 before (on run1186b (105, 110, 25) →
/// (105, 105, 0): box 2, taken on frame 1134, is walked no more), and the
/// five where theirs walks no box leave the count (run1217, run865, run882b,
/// run883b and run959, (0, 35, 35) → (0, 0, 0)); the world line agrees on
/// 39 of 107, 3 before (run1186b (7361, 7361, 1) → 0, its one range record
/// 1787's `down` ours −3 theirs −1, `down_who` 2 both).
pub(crate) const PACKETS_SCORE: (usize, usize, usize) = (107, 280, 1053);

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
    // The packet's own walk: the word holds from the built start since item
    // 4208, so its pin is the built start's.
    let Some((mut built, _)) = crate::diff::third::stand_up_built(G.long) else {
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
    // Item 4218, the idle latch (`flags & 0x08`): units (1116, 1296, 309) -> (1116, 1296, 306).
    (
        "ch94p",
        846,
        [
            (1116, 1296, 306),
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
    // Item 4218, the idle latch (`flags & 0x08`): units (1284, 1426, 1224) -> (1284, 1426, 1222).
    (
        "ch95pkt",
        1117,
        [
            (1284, 1426, 1222),
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
    // Item 4205, the built start: units (2445, 2672, 1998) -> (2445, 2672, 1966); builds (2028,
    // 2028, 49) -> (2028, 2028, 36); world (7581, 7565, 4436) -> (7581, 7581, 20).
    // Item 4209, the box's `clear_down`: builds (2028, 2028, 36) -> (2028, 2028, 35); items (5, 110,
    // 107) -> (5, 5, 0); world (7581, 7581, 20) -> (7581, 7581, 0).
    (
        "run1001d",
        13760,
        [
            (2445, 2672, 1966),
            (2028, 2028, 35),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (544, 544, 0),
            (701, 806, 106),
            (271, 271, 2),
            (30, 30, 3),
            (5, 5, 0),
            (120, 120, 0),
            (7581, 7581, 0),
        ],
    ),
    // Item 4205, the built start: units (2939, 3438, 2660) -> (2939, 3438, 2627); builds (2417,
    // 2417, 58) -> (2417, 2417, 45); world (7537, 7521, 4313) -> (7537, 7537, 12).
    // Item 4209, the box's `clear_down`: units (2939, 3438, 2627) -> (2939, 3438, 2626); items (45,
    // 110, 83) -> (45, 45, 0); world (7537, 7537, 12) -> (7537, 7537, 0).
    (
        "run1007b",
        15915,
        [
            (2939, 3438, 2626),
            (2417, 2417, 45),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (916, 1126, 215),
            (271, 271, 2),
            (30, 30, 3),
            (45, 45, 0),
            (120, 120, 0),
            (7537, 7537, 0),
        ],
    ),
    // Item 4205, the built start: units (3219, 3345, 2625) -> (3219, 3345, 2592); builds (2060,
    // 2060, 49) -> (2060, 2060, 36); world (7479, 7463, 4220) -> (7479, 7479, 11).
    // Item 4209, the box's `clear_down`: builds (2060, 2060, 36) -> (2060, 2060, 35); items (50, 110,
    // 77) -> (50, 50, 0); world (7479, 7479, 11) -> (7479, 7479, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3219, 3345, 2592) -> (3219, 3345, 2593).
    (
        "run1011b",
        13105,
        [
            (3219, 3345, 2593),
            (2060, 2060, 35),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 0),
            (870, 975, 105),
            (271, 271, 3),
            (30, 30, 3),
            (50, 50, 0),
            (120, 120, 0),
            (7479, 7479, 0),
        ],
    ),
    // Item 4205, the built start: units (3193, 3298, 2619) -> (3193, 3298, 2588); builds (2080,
    // 2080, 47) -> (2080, 2080, 34); world (7537, 7521, 4366) -> (7537, 7537, 11).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 34) -> (2080, 2080, 33); items (50, 110,
    // 77) -> (50, 50, 0); world (7537, 7537, 11) -> (7537, 7537, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3193, 3298, 2588) -> (3193, 3298, 2589).
    (
        "run1011c",
        13225,
        [
            (3193, 3298, 2589),
            (2080, 2080, 33),
            (0, 0, 0),
            (6, 0, 6),
            (4, 0, 4),
            (538, 538, 0),
            (861, 966, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 50, 0),
            (120, 120, 0),
            (7537, 7537, 0),
        ],
    ),
    // Item 4205, the built start: units (3158, 3250, 2530) -> (3158, 3250, 2498); builds (2080,
    // 2080, 47) -> (2080, 2080, 34); world (7553, 7537, 4393) -> (7553, 7553, 10).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 34) -> (2080, 2080, 33); items (50, 110,
    // 77) -> (50, 50, 0); world (7553, 7553, 10) -> (7553, 7553, 0).
    (
        "run1011d",
        13277,
        [
            (3158, 3250, 2498),
            (2080, 2080, 33),
            (0, 0, 0),
            (12, 0, 12),
            (4, 0, 4),
            (538, 538, 0),
            (861, 966, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 50, 0),
            (120, 120, 0),
            (7553, 7553, 0),
        ],
    ),
    // Item 4205, the built start: units (3117, 3189, 2465) -> (3117, 3189, 2437); builds (2080,
    // 2080, 47) -> (2080, 2080, 34); world (7517, 7501, 4316) -> (7517, 7517, 11).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 34) -> (2080, 2080, 33); items (50, 110,
    // 77) -> (50, 50, 0); world (7517, 7517, 11) -> (7517, 7517, 0).
    (
        "run1011e",
        13296,
        [
            (3117, 3189, 2437),
            (2080, 2080, 33),
            (0, 0, 0),
            (9, 0, 9),
            (6, 0, 6),
            (538, 538, 0),
            (854, 959, 105),
            (271, 271, 3),
            (30, 30, 3),
            (50, 50, 0),
            (120, 120, 0),
            (7517, 7517, 0),
        ],
    ),
    // Item 4205, the built start: units (3139, 3183, 2492) -> (3139, 3183, 2464); builds (2080,
    // 2080, 46) -> (2080, 2080, 33); world (7419, 7403, 4096) -> (7419, 7419, 11).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 33) -> (2080, 2080, 32); items (50, 110,
    // 77) -> (50, 50, 0); world (7419, 7419, 11) -> (7419, 7419, 0).
    (
        "run1015b",
        13392,
        [
            (3139, 3183, 2464),
            (2080, 2080, 32),
            (0, 0, 0),
            (27, 0, 27),
            (4, 0, 4),
            (538, 538, 0),
            (847, 952, 105),
            (271, 271, 2),
            (30, 30, 3),
            (50, 50, 0),
            (120, 120, 0),
            (7419, 7419, 0),
        ],
    ),
    // Item 4205, the built start: units (4193, 4310, 3311) -> (4193, 4310, 3249); builds (2371,
    // 2371, 68) -> (2371, 2371, 52); cities (27, 27, 2) -> (27, 27, 1); world (7583, 7567,
    // 4490) -> (7583, 7583, 8).
    // Item 4209, the box's `clear_down`: items (75, 110, 56) -> (75, 75, 0); world (7583, 7583, 8) ->
    // (7583, 7583, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (4193, 4310, 3249) -> (4193, 4310, 3246).
    (
        "run1155c",
        3003,
        [
            (4193, 4310, 3246),
            (2371, 2371, 52),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (550, 550, 2),
            (1129, 1234, 106),
            (271, 271, 3),
            (27, 27, 1),
            (75, 75, 0),
            (120, 120, 0),
            (7583, 7583, 2),
        ],
    ),
    // Item 4208, the whales' piece: units (2760, 2965, 535) -> (2760, 2965, 213); builds (520, 520,
    // 13) -> (520, 520, 0); guys (924, 994, 174) -> (924, 994, 70); world (8850, 7263, 4860) -> (8850,
    // 8850, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2760, 2965, 213) -> (2760, 2965, 205).
    (
        "run1161",
        4,
        [
            (2760, 2965, 205),
            (520, 520, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (526, 526, 0),
            (924, 994, 70),
            (263, 263, 4),
            (6, 6, 0),
            (35, 35, 0),
            (330, 330, 0),
            (8850, 8850, 0),
        ],
    ),
    // Item 4205, the built start: units (6053, 5910, 5313) -> (6053, 5910, 5268); builds (2140,
    // 2136, 820) -> (2140, 2136, 811); cities (30, 30, 3) -> (30, 30, 2); world (7597, 7581,
    // 4525) -> (7597, 7597, 22).
    // Item 4209, the box's `clear_down`: items (75, 110, 56) -> (75, 75, 0); world (7597, 7597, 22) ->
    // (7597, 7597, 15).
    // Item 4218, the idle latch (`flags & 0x08`): units (6053, 5910, 5268) -> (6053, 5910, 5265).
    (
        "run1164",
        4528,
        [
            (6053, 5910, 5265),
            (2140, 2136, 811),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (550, 550, 2),
            (1321, 1475, 158),
            (271, 271, 6),
            (30, 30, 2),
            (75, 75, 0),
            (120, 120, 0),
            (7597, 7597, 15),
        ],
    ),
    // Item 4208, the whales' piece: units (6004, 6418, 5320) -> (6004, 6418, 5252); builds (2267,
    // 2266, 445) -> (2267, 2266, 431); groups (676, 676, 3) -> (676, 676, 1); guys (1890, 2051, 263)
    // -> (1890, 2051, 163); world (9290, 7703, 6027) -> (9290, 9290, 15).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9290, 9290, 15) ->
    // (9290, 9290, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (6004, 6418, 5252) -> (6004, 6418, 5248).
    (
        "run1165",
        3719,
        [
            (6004, 6418, 5248),
            (2267, 2266, 431),
            (0, 0, 0),
            (287, 0, 287),
            (0, 0, 0),
            (676, 676, 1),
            (1890, 2051, 163),
            (271, 271, 4),
            (30, 30, 3),
            (15, 15, 0),
            (330, 330, 2),
            (9290, 9290, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (6004, 6310, 5126) -> (6004, 6310, 5070); builds (2247,
    // 2246, 508) -> (2247, 2246, 498); groups (688, 688, 3) -> (688, 688, 1); guys (1935, 2061, 229)
    // -> (1935, 2061, 129); world (9262, 7675, 5955) -> (9262, 9262, 17).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9262, 9262, 17) ->
    // (9262, 9262, 13).
    (
        "run1166",
        3920,
        [
            (6004, 6310, 5070),
            (2247, 2246, 498),
            (0, 0, 0),
            (0, 0, 0),
            (10, 0, 10),
            (688, 688, 1),
            (1935, 2061, 129),
            (271, 271, 7),
            (30, 30, 3),
            (15, 15, 0),
            (330, 330, 2),
            (9262, 9262, 13),
        ],
    ),
    // Item 4208, the whales' piece: units (7749, 8143, 6789) -> (7749, 8143, 6726); builds (1813,
    // 1813, 87) -> (1813, 1813, 73); groups (628, 628, 7) -> (628, 628, 5); guys (1934, 2095, 277) ->
    // (1934, 2095, 177); world (9438, 7851, 6516) -> (9438, 9438, 4).
    // Item 4209, the box's `clear_down`: items (20, 35, 22) -> (20, 20, 0); world (9438, 9438, 4) ->
    // (9438, 9438, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (7749, 8143, 6726) -> (7749, 8143, 6718).
    (
        "run1169",
        3769,
        [
            (7749, 8143, 6718),
            (1813, 1813, 73),
            (0, 0, 0),
            (12, 0, 12),
            (0, 0, 0),
            (628, 628, 5),
            (1934, 2095, 177),
            (271, 271, 4),
            (27, 27, 3),
            (20, 20, 0),
            (330, 330, 2),
            (9438, 9438, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (6735, 7099, 6040) -> (6735, 7099, 5981); builds (1911,
    // 1911, 73) -> (1911, 1911, 59); groups (640, 640, 9) -> (640, 640, 5); guys (1962, 2123, 268) ->
    // (1962, 2123, 168); cities (27, 27, 2) -> (27, 27, 1); world (9268, 7681, 6012) -> (9268, 9268,
    // 5).
    // Item 4209, the box's `clear_down`: units (6735, 7099, 5981) -> (6735, 7099, 5980); items (5, 35,
    // 31) -> (5, 5, 0); world (9268, 9268, 5) -> (9268, 9268, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (6735, 7099, 5980) -> (6735, 7099, 5978).
    (
        "run1185",
        4582,
        [
            (6735, 7099, 5978),
            (1911, 1911, 59),
            (0, 0, 0),
            (3, 0, 3),
            (4, 0, 4),
            (640, 640, 5),
            (1962, 2123, 168),
            (271, 271, 4),
            (27, 27, 1),
            (5, 5, 0),
            (330, 330, 2),
            (9268, 9268, 0),
        ],
    ),
    // Item 4205, the built start: units (1060, 1345, 381) -> (1060, 1345, 303); builds (1164,
    // 1164, 23) -> (1164, 1164, 9); world (7361, 7345, 3930) -> (7361, 7361, 1).
    // Item 4209, the box's `clear_down`: items (105, 110, 25) -> (105, 105, 0); world (7361, 7361, 1)
    // -> (7361, 7361, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1060, 1345, 303) -> (1060, 1345, 295).
    (
        "run1186b",
        1377,
        [
            (1060, 1345, 295),
            (1164, 1164, 9),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (309, 407, 98),
            (271, 271, 3),
            (9, 9, 0),
            (105, 105, 0),
            (120, 120, 0),
            (7361, 7361, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (10607, 10674, 9119) -> (10607, 10674, 9057); builds (3412,
    // 3412, 141) -> (3412, 3412, 126); groups (700, 700, 12) -> (700, 700, 9); guys (3247, 3436, 299)
    // -> (3247, 3436, 199); world (9486, 7899, 6514) -> (9486, 9486, 5).
    // Item 4209, the box's `clear_down`: units (10607, 10674, 9057) -> (10607, 10674, 9056); builds
    // (3412, 3412, 126) -> (3412, 3412, 124); items (0, 35, 35) -> (0, 0, 0); world (9486, 9486, 5)
    // -> (9486, 9486, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (10607, 10674, 9056) -> (10607, 10674, 9057).
    (
        "run1217",
        7728,
        [
            (10607, 10674, 9057),
            (3412, 3412, 124),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (700, 700, 9),
            (3247, 3436, 199),
            (271, 271, 4),
            (56, 56, 5),
            (0, 0, 0),
            (330, 330, 1),
            (9486, 9486, 1),
        ],
    ),
    // Item 4205, the built start: units (4091, 4338, 3359) -> (4091, 4338, 3296); builds (1817,
    // 1817, 43) -> (1817, 1817, 28); cities (27, 27, 2) -> (27, 27, 1); world (7663, 7647,
    // 4673) -> (7663, 7663, 4).
    // Item 4209, the box's `clear_down`: items (90, 110, 44) -> (90, 90, 0); world (7663, 7663, 4) ->
    // (7663, 7663, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4091, 4338, 3296) -> (4091, 4338, 3291).
    (
        "run1220b",
        2976,
        [
            (4091, 4338, 3291),
            (1817, 1817, 28),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (550, 550, 2),
            (1099, 1274, 175),
            (271, 271, 3),
            (27, 27, 1),
            (90, 90, 0),
            (120, 120, 0),
            (7663, 7663, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (6445, 6949, 5946) -> (6445, 6949, 5874); builds (1719,
    // 1719, 44) -> (1719, 1719, 30); groups (592, 592, 5) -> (592, 592, 4); guys (1768, 1992, 339) ->
    // (1768, 1992, 239); world (9162, 7575, 5806) -> (9162, 9162, 4).
    // Item 4209, the box's `clear_down`: items (10, 35, 29) -> (10, 10, 0); world (9162, 9162, 4) ->
    // (9162, 9162, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (6445, 6949, 5874) -> (6445, 6949, 5873).
    (
        "run1221",
        4287,
        [
            (6445, 6949, 5873),
            (1719, 1719, 30),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (592, 592, 4),
            (1768, 1992, 239),
            (271, 271, 4),
            (33, 33, 3),
            (10, 10, 0),
            (330, 330, 1),
            (9162, 9162, 1),
        ],
    ),
    // Item 4205, the built start: units (11740, 11409, 10463) -> (11740, 11409, 10402); builds
    // (4271, 4271, 126) -> (4271, 4271, 112); cities (62, 62, 7) -> (62, 62, 3); world (7873,
    // 7857, 4963) -> (7873, 7873, 17).
    // Item 4209, the box's `clear_down`: items (30, 110, 91) -> (30, 30, 0); world (7873, 7873, 17) ->
    // (7873, 7873, 1).
    (
        "run1224b",
        9745,
        [
            (11740, 11409, 10402),
            (4271, 4271, 112),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (556, 556, 1),
            (2735, 2910, 182),
            (271, 271, 6),
            (62, 62, 3),
            (30, 30, 0),
            (120, 120, 0),
            (7873, 7873, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (7490, 7627, 6531) -> (7490, 7627, 6482); builds (1733,
    // 1733, 57) -> (1733, 1733, 46); groups (646, 646, 10) -> (646, 646, 7); guys (2097, 2188, 197) ->
    // (2097, 2188, 97); world (9352, 7765, 6266) -> (9352, 9352, 3).
    // Item 4209, the box's `clear_down`: builds (1733, 1733, 46) -> (1733, 1733, 45); items (15, 35,
    // 26) -> (15, 15, 0); world (9352, 9352, 3) -> (9352, 9352, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (7490, 7627, 6482) -> (7490, 7627, 6479).
    (
        "run1235b",
        4127,
        [
            (7490, 7627, 6479),
            (1733, 1733, 45),
            (0, 0, 0),
            (12, 0, 12),
            (4, 0, 4),
            (646, 646, 7),
            (2097, 2188, 97),
            (271, 271, 4),
            (30, 30, 2),
            (15, 15, 0),
            (330, 330, 0),
            (9352, 9352, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (6610, 7349, 5833) -> (6610, 7349, 5764); builds (1762,
    // 1762, 58) -> (1762, 1762, 44); groups (658, 658, 7) -> (658, 658, 4); guys (2070, 2329, 392) ->
    // (2070, 2329, 292); cities (44, 44, 4) -> (44, 44, 1); world (9312, 7725, 6216) -> (9312, 9312,
    // 5).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9312, 9312, 5) ->
    // (9312, 9312, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (6610, 7349, 5764) -> (6610, 7349, 5753).
    (
        "run1236",
        4179,
        [
            (6610, 7349, 5753),
            (1762, 1762, 44),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (658, 658, 4),
            (2070, 2329, 292),
            (271, 271, 4),
            (44, 44, 1),
            (15, 15, 0),
            (330, 330, 0),
            (9312, 9312, 1),
        ],
    ),
    // Item 4205, the built start: units (3448, 3542, 2712) -> (3448, 3542, 2651); builds (1650,
    // 1650, 35) -> (1650, 1650, 18); cities (27, 27, 1) -> (27, 27, 0); world (7703, 7687,
    // 4645) -> (7703, 7703, 5).
    // Item 4209, the box's `clear_down`: items (85, 110, 48) -> (85, 85, 0); world (7703, 7703, 5) ->
    // (7703, 7703, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3448, 3542, 2651) -> (3448, 3542, 2644).
    (
        "run1237",
        2674,
        [
            (3448, 3542, 2644),
            (1650, 1650, 18),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 1),
            (892, 990, 98),
            (271, 271, 3),
            (27, 27, 0),
            (85, 85, 0),
            (120, 120, 0),
            (7703, 7703, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (6417, 7136, 5611) -> (6417, 7136, 5546); builds (2494,
    // 2494, 58) -> (2494, 2494, 44); groups (646, 646, 7) -> (646, 646, 4); guys (2151, 2410, 366) ->
    // (2151, 2410, 266); cities (49, 49, 4) -> (49, 49, 1); world (9318, 7731, 6179) -> (9318, 9318,
    // 5).
    // Item 4209, the box's `clear_down`: builds (2494, 2494, 44) -> (2494, 2494, 43); items (10, 35,
    // 29) -> (10, 10, 0); world (9318, 9318, 5) -> (9318, 9318, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (6417, 7136, 5546) -> (6417, 7136, 5516).
    (
        "run1238c",
        5021,
        [
            (6417, 7136, 5516),
            (2494, 2494, 43),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (646, 646, 4),
            (2151, 2410, 266),
            (271, 271, 4),
            (49, 49, 1),
            (10, 10, 0),
            (330, 330, 0),
            (9318, 9318, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (5942, 6521, 5260) -> (5942, 6521, 5195); builds (1692,
    // 1692, 41) -> (1692, 1692, 27); groups (622, 622, 7) -> (622, 622, 6); guys (1748, 1972, 325) ->
    // (1748, 1972, 225); world (9158, 7571, 5816) -> (9158, 9158, 5).
    // Item 4209, the box's `clear_down`: items (10, 35, 29) -> (10, 10, 0); world (9158, 9158, 5) ->
    // (9158, 9158, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5942, 6521, 5195) -> (5942, 6521, 5183).
    (
        "run1247",
        4306,
        [
            (5942, 6521, 5183),
            (1692, 1692, 27),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (622, 622, 6),
            (1748, 1972, 225),
            (271, 271, 4),
            (38, 38, 3),
            (10, 10, 0),
            (330, 330, 0),
            (9158, 9158, 0),
        ],
    ),
    // Item 4205, the built start: units (4388, 4364, 3447) -> (4388, 4364, 3388); builds (1674,
    // 1674, 54) -> (1674, 1674, 37); cities (27, 27, 2) -> (27, 27, 1); world (7669, 7653,
    // 4540) -> (7669, 7669, 5).
    // Item 4209, the box's `clear_down`: items (85, 110, 49) -> (85, 85, 0); world (7669, 7669, 5) ->
    // (7669, 7669, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4388, 4364, 3388) -> (4388, 4364, 3384).
    (
        "run1254c",
        3115,
        [
            (4388, 4364, 3384),
            (1674, 1674, 37),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (550, 550, 0),
            (1134, 1239, 107),
            (271, 271, 3),
            (27, 27, 1),
            (85, 85, 0),
            (120, 120, 0),
            (7669, 7669, 0),
        ],
    ),
    // Item 4205, the built start: units (11229, 10998, 9898) -> (11229, 10998, 9831); builds
    // (3789, 3789, 109) -> (3789, 3789, 92); world (7867, 7851, 5107) -> (7867, 7867, 10).
    // Item 4209, the box's `clear_down`: builds (3789, 3789, 92) -> (3789, 3789, 90); items (50, 110,
    // 77) -> (50, 50, 0); world (7867, 7867, 10) -> (7867, 7867, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (11229, 10998, 9831) -> (11229, 10998, 9830).
    (
        "run1263",
        8362,
        [
            (11229, 10998, 9830),
            (3789, 3789, 90),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (556, 556, 1),
            (2913, 3123, 215),
            (271, 271, 4),
            (54, 54, 5),
            (50, 50, 0),
            (120, 120, 1),
            (7867, 7867, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (6468, 7014, 5960) -> (6468, 7014, 5902); builds (1770,
    // 1770, 57) -> (1770, 1770, 43); groups (586, 586, 8) -> (586, 586, 7); guys (1895, 2119, 344) ->
    // (1895, 2119, 244); world (9330, 7743, 6143) -> (9330, 9330, 5).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9330, 9330, 5) ->
    // (9330, 9330, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (6468, 7014, 5902) -> (6468, 7014, 5901).
    (
        "run1264b",
        4919,
        [
            (6468, 7014, 5901),
            (1770, 1770, 43),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (586, 586, 7),
            (1895, 2119, 244),
            (271, 271, 3),
            (43, 43, 4),
            (15, 15, 0),
            (330, 330, 1),
            (9330, 9330, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (11161, 11622, 10287) -> (11161, 11622, 10220); builds
    // (3796, 3796, 114) -> (3796, 3796, 100); groups (682, 682, 11) -> (682, 682, 10); guys (3210,
    // 3413, 324) -> (3210, 3413, 224); world (9704, 8117, 6778) -> (9704, 9704, 6).
    // Item 4209, the box's `clear_down`: items (5, 35, 31) -> (5, 5, 0); world (9704, 9704, 6) ->
    // (9704, 9704, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (11161, 11622, 10220) -> (11161, 11622, 10213).
    (
        "run1266c",
        8700,
        [
            (11161, 11622, 10213),
            (3796, 3796, 100),
            (0, 0, 0),
            (6, 0, 6),
            (0, 0, 0),
            (682, 682, 10),
            (3210, 3413, 224),
            (271, 271, 4),
            (54, 54, 5),
            (5, 5, 0),
            (330, 330, 0),
            (9704, 9704, 1),
        ],
    ),
    // Item 4205, the built start: units (4744, 4798, 3764) -> (4744, 4798, 3708); builds (1908,
    // 1908, 65) -> (1908, 1908, 50); cities (30, 30, 3) -> (30, 30, 1); world (7773, 7757,
    // 4828) -> (7773, 7773, 4).
    // Item 4209, the box's `clear_down`: items (90, 110, 42) -> (90, 90, 0); world (7773, 7773, 4) ->
    // (7773, 7773, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4744, 4798, 3708) -> (4744, 4798, 3699).
    (
        "run1268",
        3347,
        [
            (4744, 4798, 3699),
            (1908, 1908, 50),
            (0, 0, 0),
            (24, 0, 24),
            (4, 0, 4),
            (544, 544, 0),
            (1246, 1386, 141),
            (271, 271, 4),
            (30, 30, 1),
            (90, 90, 0),
            (120, 120, 0),
            (7773, 7773, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (9512, 9862, 8627) -> (9512, 9862, 8560); builds (1938,
    // 1938, 64) -> (1938, 1938, 50); guys (2191, 2345, 265) -> (2191, 2345, 165); cities (25, 25, 2)
    // -> (25, 25, 1); world (9230, 7643, 5878) -> (9230, 9230, 3).
    // Item 4209, the box's `clear_down`: items (20, 35, 22) -> (20, 20, 0); world (9230, 9230, 3) ->
    // (9230, 9230, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (9512, 9862, 8560) -> (9512, 9862, 8555).
    (
        "run1293",
        3606,
        [
            (9512, 9862, 8555),
            (1938, 1938, 50),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (586, 586, 4),
            (2191, 2345, 165),
            (271, 271, 4),
            (25, 25, 1),
            (20, 20, 0),
            (330, 330, 1),
            (9230, 9230, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (8216, 8852, 7275) -> (8216, 8852, 7208); builds (2123,
    // 2123, 74) -> (2123, 2123, 59); groups (688, 688, 12) -> (688, 688, 11); guys (2242, 2473, 338)
    // -> (2242, 2473, 238); world (9490, 7903, 6488) -> (9490, 9490, 5).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9490, 9490, 5) ->
    // (9490, 9490, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (8216, 8852, 7208) -> (8216, 8852, 7202).
    (
        "run1294",
        4473,
        [
            (8216, 8852, 7202),
            (2123, 2123, 59),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (688, 688, 11),
            (2242, 2473, 238),
            (271, 271, 8),
            (27, 27, 3),
            (10, 10, 0),
            (330, 330, 0),
            (9490, 9490, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (9905, 10501, 9382) -> (9905, 10501, 9324); builds (2163,
    // 2163, 80) -> (2163, 2163, 64); groups (682, 682, 11) -> (682, 682, 10); guys (2172, 2403, 340)
    // -> (2172, 2403, 240); world (9470, 7883, 6497) -> (9470, 9470, 5).
    // Item 4209, the box's `clear_down`: items (5, 35, 32) -> (5, 5, 0); world (9470, 9470, 5) ->
    // (9470, 9470, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (9905, 10501, 9324) -> (9905, 10501, 9322).
    (
        "run1296",
        4632,
        [
            (9905, 10501, 9322),
            (2163, 2163, 64),
            (0, 0, 0),
            (0, 0, 0),
            (6, 0, 6),
            (682, 682, 10),
            (2172, 2403, 240),
            (271, 271, 8),
            (27, 27, 3),
            (5, 5, 0),
            (330, 330, 0),
            (9470, 9470, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (5488, 5902, 4705) -> (5488, 5902, 4638); builds (2029,
    // 2029, 78) -> (2029, 2029, 64); groups (616, 616, 4) -> (616, 616, 1); guys (1616, 1784, 269) ->
    // (1616, 1784, 169); cities (23, 23, 2) -> (23, 23, 1); world (9448, 7861, 6596) -> (9448, 9448,
    // 4).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9448, 9448, 4) ->
    // (9448, 9448, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5488, 5902, 4638) -> (5488, 5902, 4636).
    (
        "run1304",
        3865,
        [
            (5488, 5902, 4636),
            (2029, 2029, 64),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (616, 616, 1),
            (1616, 1784, 169),
            (271, 271, 4),
            (23, 23, 1),
            (15, 15, 0),
            (330, 330, 1),
            (9448, 9448, 0),
        ],
    ),
    // Item 4205, the built start: units (4968, 4832, 4109) -> (4968, 4832, 4054); builds (1882,
    // 1882, 53) -> (1882, 1882, 42); cities (35, 35, 3) -> (35, 35, 1); world (7585, 7569,
    // 4445) -> (7585, 7585, 8).
    // Item 4209, the box's `clear_down`: items (70, 110, 62) -> (70, 70, 0); world (7585, 7585, 8) ->
    // (7585, 7585, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4968, 4832, 4054) -> (4968, 4832, 4048).
    (
        "run1305b",
        4181,
        [
            (4968, 4832, 4048),
            (1882, 1882, 42),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1327, 1397, 70),
            (271, 271, 3),
            (35, 35, 1),
            (70, 70, 0),
            (120, 120, 0),
            (7585, 7585, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (8490, 8757, 7447) -> (8490, 8757, 7392); builds (2140,
    // 2140, 78) -> (2140, 2140, 67); groups (652, 652, 10) -> (652, 652, 5); guys (2197, 2323, 235) ->
    // (2197, 2323, 135); cities (39, 39, 3) -> (39, 39, 1); world (9476, 7889, 6506) -> (9476, 9476,
    // 7).
    // Item 4209, the box's `clear_down`: items (5, 35, 31) -> (5, 5, 0); world (9476, 9476, 7) ->
    // (9476, 9476, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (8490, 8757, 7392) -> (8490, 8757, 7384).
    (
        "run1320b",
        5066,
        [
            (8490, 8757, 7384),
            (2140, 2140, 67),
            (0, 0, 0),
            (9, 0, 9),
            (2, 0, 2),
            (652, 652, 5),
            (2197, 2323, 135),
            (271, 271, 5),
            (39, 39, 1),
            (5, 5, 0),
            (330, 330, 4),
            (9476, 9476, 1),
        ],
    ),
    // Item 4205, the built start: units (3933, 4098, 3225) -> (3933, 4098, 3168); builds (1792,
    // 1792, 44) -> (1792, 1792, 28); cities (27, 27, 2) -> (27, 27, 1); world (7615, 7599,
    // 4535) -> (7615, 7615, 4).
    // Item 4209, the box's `clear_down`: items (90, 110, 44) -> (90, 90, 0); world (7615, 7615, 4) ->
    // (7615, 7615, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3933, 4098, 3168) -> (3933, 4098, 3164).
    (
        "run1323b",
        3052,
        [
            (3933, 4098, 3164),
            (1792, 1792, 28),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (544, 544, 0),
            (1029, 1162, 133),
            (271, 271, 3),
            (27, 27, 1),
            (90, 90, 0),
            (120, 120, 0),
            (7615, 7615, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (7276, 7557, 6190) -> (7276, 7557, 6127); builds (1918,
    // 1918, 87) -> (1918, 1918, 75); guys (2055, 2188, 242) -> (2055, 2188, 142); cities (27, 27, 2)
    // -> (27, 27, 1); world (9466, 7879, 6370) -> (9466, 9466, 4).
    // Item 4209, the box's `clear_down`: items (20, 35, 21) -> (20, 20, 0); world (9466, 9466, 4) ->
    // (9466, 9466, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (7276, 7557, 6127) -> (7276, 7557, 6122).
    (
        "run1324c",
        4904,
        [
            (7276, 7557, 6122),
            (1918, 1918, 75),
            (0, 0, 0),
            (15, 0, 15),
            (2, 0, 2),
            (676, 676, 9),
            (2055, 2188, 142),
            (271, 271, 4),
            (27, 27, 1),
            (20, 20, 0),
            (330, 330, 4),
            (9466, 9466, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (10367, 11043, 9354) -> (10367, 11043, 9288); builds (2521,
    // 2521, 93) -> (2521, 2521, 79); groups (682, 682, 13) -> (682, 682, 8); guys (2706, 2965, 373) ->
    // (2706, 2965, 273); world (9666, 8079, 6675) -> (9666, 9666, 6).
    // Item 4209, the box's `clear_down`: items (10, 35, 27) -> (10, 10, 0); world (9666, 9666, 6) ->
    // (9666, 9666, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (10367, 11043, 9288) -> (10367, 11043, 9283).
    (
        "run1325c",
        5578,
        [
            (10367, 11043, 9283),
            (2521, 2521, 79),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (682, 682, 8),
            (2706, 2965, 273),
            (271, 271, 4),
            (43, 43, 5),
            (10, 10, 0),
            (330, 330, 1),
            (9666, 9666, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (6268, 6911, 5540) -> (6268, 6911, 5474); builds (2161,
    // 2161, 86) -> (2161, 2161, 72); groups (616, 616, 9) -> (616, 616, 7); guys (1993, 2224, 341) ->
    // (1993, 2224, 241); cities (30, 30, 2) -> (30, 30, 1); world (9244, 7657, 5995) -> (9244, 9244,
    // 3).
    // Item 4209, the box's `clear_down`: builds (2161, 2161, 72) -> (2161, 2161, 71); items (15, 35,
    // 26) -> (15, 15, 0); world (9244, 9244, 3) -> (9244, 9244, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (6268, 6911, 5474) -> (6268, 6911, 5466).
    (
        "run1326",
        4567,
        [
            (6268, 6911, 5466),
            (2161, 2161, 71),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (616, 616, 7),
            (1993, 2224, 241),
            (271, 271, 4),
            (30, 30, 1),
            (15, 15, 0),
            (330, 330, 2),
            (9244, 9244, 0),
        ],
    ),
    // Item 4205, the built start: units (13393, 13427, 11988) -> (13393, 13427, 11927); builds
    // (4230, 4230, 113) -> (4230, 4230, 98); guys (3459, 3774, 361) -> (3459, 3774, 325); world
    // (8592, 7873, 5834) -> (8592, 8592, 16).
    // Item 4209, the box's `clear_down`: builds (4230, 4230, 98) -> (4230, 4230, 97); items (25, 110,
    // 94) -> (25, 25, 0); world (8592, 8592, 16) -> (8592, 8592, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (13393, 13427, 11927) -> (13393, 13427, 11925).
    (
        "run1327",
        10930,
        [
            (13393, 13427, 11925),
            (4230, 4230, 97),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (574, 574, 2),
            (3459, 3774, 325),
            (271, 271, 4),
            (59, 59, 6),
            (25, 25, 0),
            (180, 180, 0),
            (8592, 8592, 1),
        ],
    ),
    // Item 4205, the built start: units (5746, 5891, 4421) -> (5746, 5891, 4373); builds (2354,
    // 2354, 75) -> (2354, 2354, 59); guys (1463, 1638, 211) -> (1463, 1638, 175); world (8244,
    // 7525, 5179) -> (8244, 8244, 5).
    // Item 4209, the box's `clear_down`: builds (2354, 2354, 59) -> (2354, 2354, 58); items (80, 110,
    // 54) -> (80, 80, 0); world (8244, 8244, 5) -> (8244, 8244, 0).
    (
        "run1332a",
        3627,
        [
            (5746, 5891, 4373),
            (2354, 2354, 58),
            (0, 0, 0),
            (48, 0, 48),
            (14, 0, 14),
            (580, 580, 5),
            (1463, 1638, 175),
            (271, 271, 3),
            (30, 30, 3),
            (80, 80, 0),
            (180, 180, 0),
            (8244, 8244, 0),
        ],
    ),
    // Item 4205, the built start: units (4745, 4928, 3951) -> (4745, 4928, 3885); builds (2237,
    // 2237, 74) -> (2237, 2237, 60); guys (1352, 1492, 179) -> (1352, 1492, 143); world (8328,
    // 7609, 5258) -> (8328, 8328, 7).
    // Item 4209, the box's `clear_down`: builds (2237, 2237, 60) -> (2237, 2237, 59); items (75, 110,
    // 56) -> (75, 75, 0); world (8328, 8328, 7) -> (8328, 8328, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (4745, 4928, 3885) -> (4745, 4928, 3883).
    (
        "run1351",
        4303,
        [
            (4745, 4928, 3883),
            (2237, 2237, 59),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (562, 562, 2),
            (1352, 1492, 143),
            (271, 271, 3),
            (30, 30, 3),
            (75, 75, 0),
            (180, 180, 1),
            (8328, 8328, 2),
        ],
    ),
    // Item 4205, the built start: units (4512, 4732, 3134) -> (4512, 4732, 3067); builds (2143,
    // 2143, 75) -> (2143, 2143, 61); guys (1286, 1426, 176) -> (1286, 1426, 140); cities (30,
    // 30, 2) -> (30, 30, 1); world (8298, 7579, 5257) -> (8298, 8298, 7).
    // Item 4209, the box's `clear_down`: builds (2143, 2143, 61) -> (2143, 2143, 60); items (75, 110,
    // 56) -> (75, 75, 0); world (8298, 8298, 7) -> (8298, 8298, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4512, 4732, 3067) -> (4512, 4732, 3062).
    (
        "run1351c",
        3521,
        [
            (4512, 4732, 3062),
            (2143, 2143, 60),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (556, 556, 3),
            (1286, 1426, 140),
            (271, 271, 4),
            (30, 30, 1),
            (75, 75, 0),
            (180, 180, 1),
            (8298, 8298, 1),
        ],
    ),
    // Item 4205, the built start: units (3610, 3841, 2961) -> (3610, 3841, 2904); builds (1836,
    // 1836, 45) -> (1836, 1836, 29); cities (27, 27, 2) -> (27, 27, 1); world (7777, 7761,
    // 4777) -> (7777, 7777, 6).
    // Item 4209, the box's `clear_down`: items (80, 110, 51) -> (80, 80, 0); world (7777, 7777, 6) ->
    // (7777, 7777, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (3610, 3841, 2904) -> (3610, 3841, 2898).
    (
        "run1355",
        2933,
        [
            (3610, 3841, 2898),
            (1836, 1836, 29),
            (0, 0, 0),
            (3, 0, 3),
            (2, 0, 2),
            (556, 556, 4),
            (902, 1042, 140),
            (271, 271, 3),
            (27, 27, 1),
            (80, 80, 0),
            (120, 120, 0),
            (7777, 7777, 1),
        ],
    ),
    // Item 4205, the built start: units (5707, 5937, 4835) -> (5707, 5937, 4771); builds (1977,
    // 1977, 59) -> (1977, 1977, 43); guys (1427, 1602, 211) -> (1427, 1602, 175); cities (30,
    // 30, 3) -> (30, 30, 1); world (8528, 7809, 5790) -> (8528, 8528, 7).
    // Item 4209, the box's `clear_down`: items (75, 110, 59) -> (75, 75, 0); world (8528, 8528, 7) ->
    // (8528, 8528, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5707, 5937, 4771) -> (5707, 5937, 4766).
    (
        "run1357",
        3439,
        [
            (5707, 5937, 4766),
            (1977, 1977, 43),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 2),
            (1427, 1602, 175),
            (271, 271, 4),
            (30, 30, 1),
            (75, 75, 0),
            (180, 180, 1),
            (8528, 8528, 0),
        ],
    ),
    // Item 4205, the built start: units (6464, 6432, 5647) -> (6464, 6432, 5603); builds (2974,
    // 2974, 108) -> (2974, 2974, 94); cities (48, 48, 4) -> (48, 48, 3); world (8007, 7991,
    // 5220) -> (8007, 8007, 12).
    // Item 4209, the box's `clear_down`: items (60, 110, 69) -> (60, 60, 0); world (8007, 8007, 12) ->
    // (8007, 8007, 2).
    (
        "run1359",
        5619,
        [
            (6464, 6432, 5603),
            (2974, 2974, 94),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 3),
            (1584, 1724, 144),
            (271, 271, 5),
            (48, 48, 3),
            (60, 60, 0),
            (120, 120, 0),
            (8007, 8007, 2),
        ],
    ),
    // Item 4205, the built start: units (6408, 6424, 5583) -> (6408, 6424, 5539); builds (2974,
    // 2974, 110) -> (2974, 2974, 96); cities (48, 48, 4) -> (48, 48, 3); world (7717, 7701,
    // 4791) -> (7717, 7717, 13).
    // Item 4209, the box's `clear_down`: items (55, 110, 74) -> (55, 55, 0); world (7717, 7717, 13) ->
    // (7717, 7717, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (6408, 6424, 5539) -> (6408, 6424, 5541).
    (
        "run1359b",
        5677,
        [
            (6408, 6424, 5541),
            (2974, 2974, 96),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 4),
            (1598, 1738, 144),
            (271, 271, 5),
            (48, 48, 3),
            (55, 55, 0),
            (120, 120, 0),
            (7717, 7717, 2),
        ],
    ),
    // Item 4205, the built start: units (6407, 6423, 5580) -> (6407, 6423, 5536); builds (2974,
    // 2974, 110) -> (2974, 2974, 96); cities (48, 48, 4) -> (48, 48, 3); world (7715, 7699,
    // 4789) -> (7715, 7715, 13).
    // Item 4209, the box's `clear_down`: items (55, 110, 74) -> (55, 55, 0); world (7715, 7715, 13) ->
    // (7715, 7715, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (6407, 6423, 5536) -> (6407, 6423, 5538).
    (
        "run1359c",
        5678,
        [
            (6407, 6423, 5538),
            (2974, 2974, 96),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 4),
            (1598, 1738, 144),
            (271, 271, 5),
            (48, 48, 3),
            (55, 55, 0),
            (120, 120, 0),
            (7715, 7715, 2),
        ],
    ),
    // Item 4205, the built start: units (6810, 6671, 5784) -> (6810, 6671, 5721); builds (2544,
    // 2544, 72) -> (2544, 2544, 57); guys (1847, 1952, 141) -> (1847, 1952, 105); cities (37,
    // 37, 3) -> (37, 37, 2); world (8476, 7757, 5586) -> (8476, 8476, 10).
    // Item 4209, the box's `clear_down`: builds (2544, 2544, 57) -> (2544, 2544, 56); items (50, 110,
    // 78) -> (50, 50, 0); world (8476, 8476, 10) -> (8476, 8476, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (6810, 6671, 5721) -> (6810, 6671, 5720).
    (
        "run1361",
        4840,
        [
            (6810, 6671, 5720),
            (2544, 2544, 56),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (556, 556, 1),
            (1847, 1952, 105),
            (271, 271, 8),
            (37, 37, 2),
            (50, 50, 0),
            (180, 180, 2),
            (8476, 8476, 0),
        ],
    ),
    // Item 4205, the built start: units (5194, 5195, 3939) -> (5194, 5195, 3876); builds (2147,
    // 2147, 64) -> (2147, 2147, 49); guys (1419, 1517, 134) -> (1419, 1517, 98); cities (27,
    // 27, 3) -> (27, 27, 1); world (8334, 7615, 5331) -> (8334, 8334, 3).
    // Item 4209, the box's `clear_down`: builds (2147, 2147, 49) -> (2147, 2147, 48); items (90, 110,
    // 44) -> (90, 90, 0); world (8334, 8334, 3) -> (8334, 8334, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (5194, 5195, 3876) -> (5194, 5195, 3875).
    (
        "run1381",
        3262,
        [
            (5194, 5195, 3875),
            (2147, 2147, 48),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (556, 556, 2),
            (1419, 1517, 98),
            (271, 271, 3),
            (27, 27, 1),
            (90, 90, 0),
            (180, 180, 1),
            (8334, 8334, 1),
        ],
    ),
    // Item 4205, the built start: units (5014, 5130, 4171) -> (5014, 5130, 4106); builds (2015,
    // 2015, 67) -> (2015, 2015, 52); guys (1349, 1482, 169) -> (1349, 1482, 133); cities (20,
    // 20, 1) -> (20, 20, 0); world (8340, 7621, 5320) -> (8340, 8340, 7).
    // Item 4209, the box's `clear_down`: items (70, 110, 61) -> (70, 70, 0); world (8340, 8340, 7) ->
    // (8340, 8340, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (5014, 5130, 4106) -> (5014, 5130, 4101).
    (
        "run1382",
        3228,
        [
            (5014, 5130, 4101),
            (2015, 2015, 52),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 0),
            (1349, 1482, 133),
            (271, 271, 4),
            (20, 20, 0),
            (70, 70, 0),
            (180, 180, 0),
            (8340, 8340, 2),
        ],
    ),
    // Item 4208, the whales' piece: units (2933, 3253, 650) -> (2933, 3253, 339); builds (1184, 1184,
    // 21) -> (1184, 1184, 7); guys (967, 1079, 213) -> (967, 1079, 112); world (8876, 7289, 4946) ->
    // (8876, 8876, 1).
    // Item 4209, the box's `clear_down`: items (30, 35, 11) -> (30, 30, 0); world (8876, 8876, 1) ->
    // (8876, 8876, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2933, 3253, 339) -> (2933, 3253, 333).
    (
        "run1383",
        1180,
        [
            (2933, 3253, 333),
            (1184, 1184, 7),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 1),
            (967, 1079, 112),
            (271, 271, 3),
            (6, 6, 0),
            (30, 30, 0),
            (330, 330, 0),
            (8876, 8876, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (5193, 5463, 4596) -> (5193, 5463, 4532); builds (1774,
    // 1774, 42) -> (1774, 1774, 28); groups (652, 652, 2) -> (652, 652, 1); guys (1609, 1756, 247) ->
    // (1609, 1756, 147); cities (27, 27, 2) -> (27, 27, 0); world (9252, 7665, 5784) -> (9252, 9252,
    // 3).
    // Item 4209, the box's `clear_down`: items (20, 35, 21) -> (20, 20, 0); world (9252, 9252, 3) ->
    // (9252, 9252, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5193, 5463, 4532) -> (5193, 5463, 4526).
    (
        "run1385p",
        2775,
        [
            (5193, 5463, 4526),
            (1774, 1774, 28),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 1),
            (1609, 1756, 147),
            (271, 271, 4),
            (27, 27, 0),
            (20, 20, 0),
            (330, 330, 0),
            (9252, 9252, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (6512, 6830, 5822) -> (6512, 6830, 5761); builds (1830,
    // 1830, 73) -> (1830, 1830, 58); groups (694, 694, 17) -> (694, 694, 14); guys (1927, 2088, 265)
    // -> (1927, 2088, 165); world (9408, 7821, 6277) -> (9408, 9408, 3).
    // Item 4209, the box's `clear_down`: units (6512, 6830, 5761) -> (6512, 6830, 5759); items (10, 35,
    // 28) -> (10, 10, 0); world (9408, 9408, 3) -> (9408, 9408, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (6512, 6830, 5759) -> (6512, 6830, 5755).
    (
        "run1391",
        4174,
        [
            (6512, 6830, 5755),
            (1830, 1830, 58),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (694, 694, 14),
            (1927, 2088, 165),
            (271, 271, 8),
            (35, 35, 3),
            (10, 10, 0),
            (330, 330, 1),
            (9408, 9408, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (6953, 7259, 6119) -> (6953, 7259, 6070); builds (2128,
    // 2127, 678) -> (2128, 2127, 669); groups (634, 634, 10) -> (634, 634, 9); guys (1829, 1955, 240)
    // -> (1829, 1955, 140); cities (33, 33, 2) -> (33, 33, 1); world (9554, 7967, 6698) -> (9554,
    // 9554, 15).
    // Item 4209, the box's `clear_down`: units (6953, 7259, 6070) -> (6953, 7259, 6069); items (5, 35,
    // 31) -> (5, 5, 0); world (9554, 9554, 15) -> (9554, 9554, 10).
    // Item 4218, the idle latch (`flags & 0x08`): units (6953, 7259, 6069) -> (6953, 7259, 6066).
    (
        "run1416",
        4016,
        [
            (6953, 7259, 6066),
            (2128, 2127, 669),
            (0, 0, 0),
            (18, 0, 18),
            (0, 0, 0),
            (634, 634, 9),
            (1829, 1955, 140),
            (271, 271, 4),
            (33, 33, 1),
            (5, 5, 0),
            (330, 330, 2),
            (9554, 9554, 10),
        ],
    ),
    // Item 4205, the built start: units (4518, 4705, 3767) -> (4518, 4705, 3703); builds (2321,
    // 2321, 75) -> (2321, 2321, 61); guys (1265, 1405, 178) -> (1265, 1405, 141); cities (33,
    // 33, 3) -> (33, 33, 2); world (8274, 7555, 5208) -> (8274, 8274, 8).
    // Item 4209, the box's `clear_down`: items (75, 110, 54) -> (75, 75, 0); world (8274, 8274, 8) ->
    // (8274, 8274, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4518, 4705, 3703) -> (4518, 4705, 3700).
    (
        "run1417",
        4016,
        [
            (4518, 4705, 3700),
            (2321, 2321, 61),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1265, 1405, 141),
            (271, 271, 4),
            (33, 33, 2),
            (75, 75, 0),
            (180, 180, 1),
            (8274, 8274, 1),
        ],
    ),
    // Item 4209, the box's `clear_down`: builds (2033, 2033, 75) -> (2033, 2033, 74); items (60, 110,
    // 69) -> (60, 60, 0).
    // Item 4210, walked from the built start (4205b's words): units (4406, 4486, 3414) -> (4406,
    // 4486, 3351); builds (2033, 2033, 74) -> (2033, 2033, 56); guys (1231, 1336, 141) -> (1231,
    // 1336, 105); cities (27, 27, 2) -> (27, 27, 1); world (8434, 7715, 5388) -> (8434, 8434, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (4406, 4486, 3351) -> (4406, 4486, 3348).
    (
        "run1419b",
        3212,
        [
            (4406, 4486, 3348),
            (2033, 2033, 56),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 2),
            (1231, 1336, 105),
            (271, 271, 4),
            (27, 27, 1),
            (60, 60, 0),
            (180, 180, 0),
            (8434, 8434, 2),
        ],
    ),
    // Item 4205, the built start: units (5583, 5614, 4645) -> (5583, 5614, 4588); builds (2515,
    // 2513, 751) -> (2515, 2513, 737); guys (1327, 1467, 177) -> (1327, 1467, 141); world
    // (8466, 7747, 5536) -> (8466, 8466, 9).
    // Item 4209, the box's `clear_down`: items (70, 110, 59) -> (70, 70, 0); world (8466, 8466, 9) ->
    // (8466, 8466, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (5583, 5614, 4588) -> (5583, 5614, 4589).
    (
        "run1420",
        4586,
        [
            (5583, 5614, 4589),
            (2515, 2513, 737),
            (0, 0, 0),
            (0, 0, 0),
            (8, 0, 8),
            (544, 544, 1),
            (1327, 1467, 141),
            (271, 271, 4),
            (38, 38, 3),
            (70, 70, 0),
            (180, 180, 0),
            (8466, 8466, 1),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (90, 110, 44) -> (90, 90, 0).
    // Item 4210, walked from the built start (4205b's words): units (4129, 4318, 3389) -> (4129,
    // 4318, 3325); builds (2025, 2025, 48) -> (2025, 2025, 32); guys (1138, 1278, 177) -> (1138,
    // 1278, 141); cities (27, 27, 3) -> (27, 27, 1); world (8344, 7625, 5328) -> (8344, 8344, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (4129, 4318, 3325) -> (4129, 4318, 3320).
    (
        "run1424",
        3156,
        [
            (4129, 4318, 3320),
            (2025, 2025, 32),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 3),
            (1138, 1278, 141),
            (271, 271, 4),
            (27, 27, 1),
            (90, 90, 0),
            (180, 180, 0),
            (8344, 8344, 2),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (80, 110, 52) -> (80, 80, 0).
    // Item 4210, walked from the built start (4205b's words): units (4175, 4291, 3481) -> (4175,
    // 4291, 3429); builds (2266, 2265, 861) -> (2266, 2265, 848); guys (1227, 1332, 146) -> (1227,
    // 1332, 110); cities (27, 27, 3) -> (27, 27, 2); world (8284, 7565, 5190) -> (8284, 8284, 10).
    // Item 4218, the idle latch (`flags & 0x08`): units (4175, 4291, 3429) -> (4175, 4291, 3428).
    (
        "run1426b",
        3722,
        [
            (4175, 4291, 3428),
            (2266, 2265, 848),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (544, 544, 1),
            (1227, 1332, 110),
            (271, 271, 4),
            (27, 27, 2),
            (80, 80, 0),
            (180, 180, 0),
            (8284, 8284, 10),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (85, 110, 49) -> (85, 85, 0).
    // Item 4210, walked from the built start (4205b's words): units (4789, 4721, 3966) -> (4789,
    // 4721, 3908); builds (2154, 2151, 619) -> (2154, 2151, 604); guys (1208, 1348, 183) -> (1208,
    // 1348, 147); cities (30, 30, 3) -> (30, 30, 1); world (8414, 7695, 5513) -> (8414, 8414, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4789, 4721, 3908) -> (4789, 4721, 3901).
    (
        "run1437",
        3894,
        [
            (4789, 4721, 3901),
            (2154, 2151, 604),
            (0, 0, 0),
            (3, 0, 3),
            (4, 0, 4),
            (538, 538, 0),
            (1208, 1348, 147),
            (271, 271, 6),
            (30, 30, 1),
            (85, 85, 0),
            (180, 180, 0),
            (8414, 8414, 1),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (2475, 2702, 2286) -> (2475, 2702, 2279).
    (
        "run1438",
        684,
        [
            (2475, 2702, 2279),
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
    // Item 4209, the box's `clear_down`: items (130, 135, 28) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2308, 2539, 2166) -> (2308, 2539, 2160).
    (
        "run1440",
        809,
        [
            (2308, 2539, 2160),
            (1968, 1968, 25),
            (0, 0, 0),
            (18, 0, 18),
            (14, 0, 14),
            (568, 568, 2),
            (638, 757, 120),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7513, 7497, 4347),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0).
    // Item 4210, walked from the built start (4205b's words): units (4274, 4219, 3527) -> (4274,
    // 4219, 3476); builds (1890, 1887, 527) -> (1890, 1887, 518); guys (1108, 1171, 100) -> (1108,
    // 1171, 64); cities (30, 30, 3) -> (30, 30, 2); world (8276, 7557, 5229) -> (8276, 8276, 9).
    // Item 4218, the idle latch (`flags & 0x08`): units (4274, 4219, 3476) -> (4274, 4219, 3474).
    (
        "run1441",
        3294,
        [
            (4274, 4219, 3474),
            (1890, 1887, 518),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 64),
            (271, 271, 4),
            (30, 30, 2),
            (75, 75, 0),
            (180, 180, 0),
            (8276, 8276, 9),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (125, 135, 37) -> (125, 125, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2022, 2281, 1870) -> (2022, 2281, 1864).
    (
        "run1451",
        945,
        [
            (2022, 2281, 1864),
            (1968, 1968, 29),
            (0, 0, 0),
            (18, 0, 18),
            (32, 0, 32),
            (568, 568, 2),
            (604, 723, 120),
            (399, 399, 6),
            (9, 9, 0),
            (125, 125, 0),
            (165, 165, 0),
            (7585, 7569, 4477),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2655, 2718, 2286) -> (2655, 2718, 2279).
    (
        "run1457",
        700,
        [
            (2655, 2718, 2279),
            (1144, 1144, 28),
            (0, 0, 0),
            (21, 0, 21),
            (2, 0, 2),
            (574, 574, 2),
            (725, 788, 64),
            (399, 399, 4),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7525, 7509, 4362),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2362, 2425, 1982) -> (2362, 2425, 1974).
    (
        "run1459",
        780,
        [
            (2362, 2425, 1974),
            (1316, 1316, 26),
            (0, 0, 0),
            (0, 0, 0),
            (22, 0, 22),
            (568, 568, 2),
            (665, 728, 64),
            (403, 403, 5),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7535, 7519, 4363),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2380, 2455, 1949) -> (2380, 2455, 1941).
    (
        "run1460",
        760,
        [
            (2380, 2455, 1941),
            (1296, 1296, 27),
            (0, 0, 0),
            (9, 0, 9),
            (16, 0, 16),
            (568, 568, 3),
            (686, 749, 64),
            (403, 403, 5),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7523, 7507, 4355),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 29) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1392, 1679, 1048) -> (1392, 1679, 1040).
    (
        "run1469",
        627,
        [
            (1392, 1679, 1040),
            (1777, 1777, 29),
            (0, 0, 0),
            (228, 0, 228),
            (0, 0, 0),
            (556, 556, 4),
            (413, 518, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 29) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1401, 1685, 1056) -> (1401, 1685, 1049).
    (
        "run1469b",
        628,
        [
            (1401, 1685, 1049),
            (1777, 1777, 29),
            (0, 0, 0),
            (190, 0, 190),
            (0, 0, 0),
            (556, 556, 4),
            (413, 518, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2180, 2393, 2009) -> (2180, 2393, 2002).
    (
        "run1472",
        707,
        [
            (2180, 2393, 2002),
            (1777, 1777, 30),
            (0, 0, 0),
            (339, 0, 339),
            (4, 0, 4),
            (568, 568, 4),
            (612, 717, 106),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7527, 7511, 4420),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4145, 4206, 3495) -> (4145, 4206, 3484).
    (
        "run1484",
        676,
        [
            (4145, 4206, 3484),
            (1446, 1446, 28),
            (0, 0, 0),
            (304, 0, 304),
            (0, 0, 0),
            (562, 562, 2),
            (1381, 1416, 159),
            (403, 403, 6),
            (9, 9, 0),
            (25, 25, 0),
            (405, 405, 0),
            (9053, 7519, 5588),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5340, 5621, 4718) -> (5340, 5621, 4710).
    (
        "run1487",
        1980,
        [
            (5340, 5621, 4710),
            (2736, 2736, 66),
            (0, 0, 0),
            (208, 0, 208),
            (8, 0, 8),
            (610, 610, 2),
            (1718, 1844, 247),
            (407, 407, 6),
            (31, 31, 0),
            (25, 25, 0),
            (405, 405, 2),
            (9267, 7733, 6310),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5026, 5286, 4896) -> (5026, 5286, 4889).
    (
        "run1496",
        1539,
        [
            (5026, 5286, 4889),
            (2180, 2180, 47),
            (0, 0, 0),
            (230, 0, 230),
            (2, 0, 2),
            (586, 586, 2),
            (1577, 1696, 241),
            (407, 407, 6),
            (23, 23, 0),
            (25, 25, 0),
            (405, 405, 2),
            (9183, 7649, 6125),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0).
    // Item 4210, walked from the built start (4205b's words): units (4274, 4219, 3527) -> (4274,
    // 4219, 3476); builds (1890, 1887, 527) -> (1890, 1887, 518); guys (1108, 1171, 100) -> (1108,
    // 1171, 64); cities (30, 30, 3) -> (30, 30, 2); world (8276, 7557, 5229) -> (8276, 8276, 9).
    // Item 4218, the idle latch (`flags & 0x08`): units (4274, 4219, 3476) -> (4274, 4219, 3474).
    (
        "run1559b",
        3294,
        [
            (4274, 4219, 3474),
            (1890, 1887, 518),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 64),
            (271, 271, 4),
            (30, 30, 2),
            (75, 75, 0),
            (180, 180, 0),
            (8276, 8276, 9),
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
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0).
    // Item 4210, walked from the built start (4205b's words): units (4274, 4219, 3527) -> (4274,
    // 4219, 3476); builds (1890, 1887, 527) -> (1890, 1887, 518); guys (1108, 1171, 100) -> (1108,
    // 1171, 64); cities (30, 30, 3) -> (30, 30, 2); world (8276, 7557, 5229) -> (8276, 8276, 9).
    // Item 4218, the idle latch (`flags & 0x08`): units (4274, 4219, 3476) -> (4274, 4219, 3474).
    (
        "run1564",
        3294,
        [
            (4274, 4219, 3474),
            (1890, 1887, 518),
            (0, 0, 0),
            (0, 0, 0),
            (2, 0, 2),
            (550, 550, 3),
            (1108, 1171, 64),
            (271, 271, 4),
            (30, 30, 2),
            (75, 75, 0),
            (180, 180, 0),
            (8276, 8276, 9),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0).
    // Item 4210, walked from the built start (4205b's words): units (4240, 4197, 3409) -> (4240,
    // 4197, 3360); builds (1890, 1887, 528) -> (1890, 1887, 519); guys (1112, 1175, 101) -> (1112,
    // 1175, 65); world (8294, 7575, 5253) -> (8294, 8294, 9).
    // Item 4218, the idle latch (`flags & 0x08`): units (4240, 4197, 3360) -> (4240, 4197, 3359).
    (
        "run1570",
        3331,
        [
            (4240, 4197, 3359),
            (1890, 1887, 519),
            (0, 0, 0),
            (0, 0, 0),
            (4, 0, 4),
            (556, 556, 2),
            (1112, 1175, 65),
            (271, 271, 4),
            (30, 30, 3),
            (75, 75, 0),
            (180, 180, 0),
            (8294, 8294, 9),
        ],
    ),
    // Item 4208, the whales' piece: units (4969, 4983, 4166) -> (4969, 4983, 4139); builds (1731,
    // 1729, 965) -> (1731, 1729, 958); guys (1429, 1506, 178) -> (1429, 1506, 78); cities (47, 47, 4)
    // -> (47, 47, 1); world (9108, 7521, 5669) -> (9108, 9108, 14).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9108, 9108, 14) ->
    // (9108, 9108, 10).
    // Item 4218, the idle latch (`flags & 0x08`): units (4969, 4983, 4139) -> (4969, 4983, 4140).
    (
        "run785",
        2178,
        [
            (4969, 4983, 4140),
            (1731, 1729, 958),
            (0, 0, 0),
            (0, 0, 0),
            (28, 0, 28),
            (640, 640, 4),
            (1429, 1506, 78),
            (271, 271, 7),
            (47, 47, 1),
            (15, 15, 0),
            (330, 330, 1),
            (9108, 9108, 10),
        ],
    ),
    // Item 4208, the whales' piece: units (5889, 5855, 5006) -> (5889, 5855, 4979); builds (2157,
    // 2155, 1285) -> (2157, 2155, 1278); guys (1765, 1842, 178) -> (1765, 1842, 78); world (9526,
    // 7939, 6468) -> (9526, 9526, 15).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9526, 9526, 15) ->
    // (9526, 9526, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (5889, 5855, 4979) -> (5889, 5855, 4982).
    (
        "run789",
        2799,
        [
            (5889, 5855, 4982),
            (2157, 2155, 1278),
            (0, 0, 0),
            (281, 0, 281),
            (0, 0, 0),
            (664, 664, 2),
            (1765, 1842, 78),
            (271, 271, 7),
            (52, 52, 5),
            (15, 15, 0),
            (330, 330, 0),
            (9526, 9526, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (5860, 5823, 4887) -> (5860, 5823, 4860); builds (2059,
    // 2057, 1209) -> (2059, 2057, 1202); guys (1704, 1781, 178) -> (1704, 1781, 78); world (9420,
    // 7833, 6400) -> (9420, 9420, 15).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9420, 9420, 15) ->
    // (9420, 9420, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (5860, 5823, 4860) -> (5860, 5823, 4862).
    (
        "run795",
        2681,
        [
            (5860, 5823, 4862),
            (2059, 2057, 1202),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 2),
            (1704, 1781, 78),
            (271, 271, 7),
            (52, 52, 5),
            (15, 15, 0),
            (330, 330, 1),
            (9420, 9420, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (9526, 9603, 8636) -> (9526, 9603, 8609); builds (2384,
    // 2381, 1424) -> (2384, 2381, 1417); groups (688, 688, 6) -> (688, 688, 5); guys (2341, 2425, 195)
    // -> (2341, 2425, 95); world (9442, 7855, 6428) -> (9442, 9442, 16).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9442, 9442, 16) ->
    // (9442, 9442, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (9526, 9603, 8609) -> (9526, 9603, 8610).
    (
        "run810",
        3631,
        [
            (9526, 9603, 8610),
            (2384, 2381, 1417),
            (0, 0, 0),
            (156, 0, 156),
            (0, 0, 0),
            (688, 688, 5),
            (2341, 2425, 95),
            (271, 271, 7),
            (58, 58, 5),
            (10, 10, 0),
            (330, 330, 1),
            (9442, 9442, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (9339, 9413, 8439) -> (9339, 9413, 8412); builds (2473,
    // 2469, 1560) -> (2473, 2469, 1553); guys (2450, 2534, 195) -> (2450, 2534, 95); world (9552,
    // 7965, 6484) -> (9552, 9552, 15).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9552, 9552, 15) ->
    // (9552, 9552, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (9339, 9413, 8412) -> (9339, 9413, 8415).
    (
        "run823",
        4238,
        [
            (9339, 9413, 8415),
            (2473, 2469, 1553),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (676, 676, 6),
            (2450, 2534, 95),
            (271, 271, 7),
            (58, 58, 5),
            (10, 10, 0),
            (330, 330, 0),
            (9552, 9552, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (9362, 9436, 8403) -> (9362, 9436, 8376); builds (2473,
    // 2469, 1560) -> (2473, 2469, 1553); groups (688, 688, 6) -> (688, 688, 5); guys (2471, 2555, 194)
    // -> (2471, 2555, 94); world (9560, 7973, 6527) -> (9560, 9560, 15).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9560, 9560, 15) ->
    // (9560, 9560, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (9362, 9436, 8376) -> (9362, 9436, 8379).
    (
        "run825",
        4204,
        [
            (9362, 9436, 8379),
            (2473, 2469, 1553),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (688, 688, 5),
            (2471, 2555, 94),
            (271, 271, 7),
            (58, 58, 5),
            (10, 10, 0),
            (330, 330, 0),
            (9560, 9560, 11),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1201, 1312, 1056) -> (1201, 1312, 1053).
    (
        "run839",
        634,
        [
            (1201, 1312, 1053),
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
    // Item 4218, the idle latch (`flags & 0x08`): units (1149, 1303, 1028) -> (1149, 1303, 1027).
    (
        "run842",
        777,
        [
            (1149, 1303, 1027),
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
    // Item 4218, the idle latch (`flags & 0x08`): units (1154, 1308, 1028) -> (1154, 1308, 1027).
    (
        "run842b",
        776,
        [
            (1154, 1308, 1027),
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
    // Item 4208, the whales' piece: units (5770, 5833, 4677) -> (5770, 5833, 4618); builds (2095,
    // 2095, 73) -> (2095, 2095, 61); groups (640, 640, 8) -> (640, 640, 7); guys (1794, 1878, 184) ->
    // (1794, 1878, 84); world (9344, 7757, 6056) -> (9344, 9344, 8).
    // Item 4209, the box's `clear_down`: items (5, 35, 32) -> (5, 5, 0); world (9344, 9344, 8) ->
    // (9344, 9344, 2).
    (
        "run854",
        3301,
        [
            (5770, 5833, 4618),
            (2095, 2095, 61),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (640, 640, 7),
            (1794, 1878, 84),
            (271, 271, 5),
            (36, 36, 0),
            (5, 5, 0),
            (330, 330, 2),
            (9344, 9344, 2),
        ],
    ),
    // Item 4208, the whales' piece: units (10342, 10233, 9575) -> (10342, 10233, 9554); builds (2288,
    // 2283, 1566) -> (2288, 2283, 1562); groups (640, 640, 6) -> (640, 640, 5); guys (2210, 2294, 185)
    // -> (2210, 2294, 85); cities (64, 64, 4) -> (64, 64, 1); world (9534, 7947, 6461) -> (9534, 9534,
    // 17).
    // Item 4209, the box's `clear_down`: units (10342, 10233, 9554) -> (10342, 10233, 9553); items (0,
    // 35, 35) -> (0, 0, 0); world (9534, 9534, 17) -> (9534, 9534, 12).
    (
        "run865",
        4859,
        [
            (10342, 10233, 9553),
            (2288, 2283, 1562),
            (0, 0, 0),
            (0, 0, 0),
            (28, 0, 28),
            (640, 640, 5),
            (2210, 2294, 85),
            (271, 271, 5),
            (64, 64, 1),
            (0, 0, 0),
            (330, 330, 2),
            (9534, 9534, 12),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1636, 1776, 294) -> (1636, 1776, 263).
    (
        "run870",
        901,
        [
            (1636, 1776, 263),
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
    // Item 4208, the whales' piece: units (11125, 10562, 9845) -> (11125, 10562, 9821); builds (2737,
    // 2732, 1950) -> (2737, 2732, 1946); guys (2832, 2923, 224) -> (2832, 2923, 124); cities (64, 64,
    // 7) -> (64, 64, 4); world (9518, 7931, 6568) -> (9518, 9518, 18).
    // Item 4209, the box's `clear_down`: items (0, 35, 35) -> (0, 0, 0); world (9518, 9518, 18) ->
    // (9518, 9518, 12).
    // Item 4218, the idle latch (`flags & 0x08`): units (11125, 10562, 9821) -> (11125, 10562, 9823).
    (
        "run882b",
        6808,
        [
            (11125, 10562, 9823),
            (2737, 2732, 1946),
            (0, 0, 0),
            (15, 0, 15),
            (0, 0, 0),
            (616, 616, 10),
            (2832, 2923, 124),
            (271, 271, 5),
            (64, 64, 4),
            (0, 0, 0),
            (330, 330, 2),
            (9518, 9518, 12),
        ],
    ),
    // Item 4208, the whales' piece: units (11180, 10557, 9936) -> (11180, 10557, 9912); builds (2737,
    // 2732, 1951) -> (2737, 2732, 1947); guys (2832, 2923, 222) -> (2832, 2923, 122); cities (64, 64,
    // 7) -> (64, 64, 4); world (9536, 7949, 6589) -> (9536, 9536, 18).
    // Item 4209, the box's `clear_down`: items (0, 35, 35) -> (0, 0, 0); world (9536, 9536, 18) ->
    // (9536, 9536, 12).
    (
        "run883b",
        6821,
        [
            (11180, 10557, 9912),
            (2737, 2732, 1947),
            (0, 0, 0),
            (36, 0, 36),
            (0, 0, 0),
            (616, 616, 10),
            (2832, 2923, 122),
            (271, 271, 5),
            (64, 64, 4),
            (0, 0, 0),
            (330, 330, 2),
            (9536, 9536, 12),
        ],
    ),
    // Item 4208, the whales' piece: units (2652, 2987, 625) -> (2652, 2987, 343); builds (709, 709,
    // 20) -> (709, 709, 7); guys (900, 1012, 212) -> (900, 1012, 112); world (8966, 7379, 5253) ->
    // (8966, 8966, 1).
    // Item 4209, the box's `clear_down`: items (30, 35, 11) -> (30, 30, 0); world (8966, 8966, 1) ->
    // (8966, 8966, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2652, 2987, 343) -> (2652, 2987, 341).
    (
        "run893b",
        3975,
        [
            (2652, 2987, 341),
            (709, 709, 7),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (900, 1012, 112),
            (271, 271, 3),
            (9, 9, 0),
            (30, 30, 0),
            (330, 330, 0),
            (8966, 8966, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (4112, 4387, 3652) -> (4112, 4387, 3617); builds (1508,
    // 1508, 40) -> (1508, 1508, 26); guys (1352, 1464, 217) -> (1352, 1464, 117); world (9122, 7535,
    // 5619) -> (9122, 9122, 6).
    // Item 4209, the box's `clear_down`: items (5, 35, 32) -> (5, 5, 0); world (9122, 9122, 6) ->
    // (9122, 9122, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4112, 4387, 3617) -> (4112, 4387, 3620).
    (
        "run912e",
        12753,
        [
            (4112, 4387, 3620),
            (1508, 1508, 26),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 5),
            (1352, 1464, 117),
            (271, 271, 3),
            (27, 27, 3),
            (5, 5, 0),
            (330, 330, 0),
            (9122, 9122, 0),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1118, 1235, 925) -> (1118, 1235, 924).
    (
        "run938b",
        630,
        [
            (1118, 1235, 924),
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
    // Item 4205, the built start: units (2726, 3048, 2294) -> (2726, 3048, 2260); builds (1580,
    // 1580, 34) -> (1580, 1580, 21); guys (819, 959, 176) -> (819, 959, 140); cities (27, 27,
    // 2) -> (27, 27, 1); world (8150, 7431, 4899) -> (8150, 8150, 14).
    // Item 4209, the box's `clear_down`: items (40, 110, 84) -> (40, 40, 0); world (8150, 8150, 14) ->
    // (8150, 8150, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2726, 3048, 2260) -> (2726, 3048, 2259).
    (
        "run943c",
        10674,
        [
            (2726, 3048, 2259),
            (1580, 1580, 21),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (819, 959, 140),
            (271, 271, 3),
            (27, 27, 1),
            (40, 40, 0),
            (180, 180, 0),
            (8150, 8150, 0),
        ],
    ),
    // Item 4205, the built start: units (2838, 3119, 2310) -> (2838, 3119, 2277); builds (1580,
    // 1580, 33) -> (1580, 1580, 20); guys (826, 966, 176) -> (826, 966, 140); cities (27, 27,
    // 2) -> (27, 27, 1); world (8164, 7445, 4942) -> (8164, 8164, 14).
    // Item 4209, the box's `clear_down`: items (40, 110, 84) -> (40, 40, 0); world (8164, 8164, 14) ->
    // (8164, 8164, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2838, 3119, 2277) -> (2838, 3119, 2276).
    (
        "run946",
        10842,
        [
            (2838, 3119, 2276),
            (1580, 1580, 20),
            (0, 0, 0),
            (15, 0, 15),
            (0, 0, 0),
            (532, 532, 0),
            (826, 966, 140),
            (271, 271, 3),
            (27, 27, 1),
            (40, 40, 0),
            (180, 180, 0),
            (8164, 8164, 0),
        ],
    ),
    // Item 4205, the built start: units (2832, 3140, 2431) -> (2832, 3140, 2402); builds (1598,
    // 1598, 36) -> (1598, 1598, 24); guys (875, 1015, 180) -> (875, 1015, 144); world (8096,
    // 7377, 4771) -> (8096, 8096, 15).
    // Item 4209, the box's `clear_down`: items (35, 110, 87) -> (35, 35, 0); world (8096, 8096, 15) ->
    // (8096, 8096, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2832, 3140, 2402) -> (2832, 3140, 2399).
    (
        "run954b",
        12155,
        [
            (2832, 3140, 2399),
            (1598, 1598, 24),
            (0, 0, 0),
            (9, 0, 9),
            (2, 0, 2),
            (532, 532, 0),
            (875, 1015, 144),
            (271, 271, 3),
            (27, 27, 3),
            (35, 35, 0),
            (180, 180, 0),
            (8096, 8096, 0),
        ],
    ),
    // Item 4205, the built start: units (490, 550, 108) -> (490, 550, 62); builds (455, 455,
    // 15) -> (455, 455, 2); guys (160, 181, 23) -> (160, 181, 21); world (7291, 7275, 3748) ->
    // (7291, 7291, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (490, 550, 62) -> (490, 550, 61).
    (
        "run958b",
        375,
        [
            (490, 550, 61),
            (455, 455, 2),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (160, 181, 21),
            (267, 267, 3),
            (6, 6, 0),
            (110, 110, 0),
            (120, 120, 0),
            (7291, 7291, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (4466, 4844, 3954) -> (4466, 4844, 3921); builds (1832,
    // 1832, 48) -> (1832, 1832, 35); guys (1514, 1668, 268) -> (1514, 1668, 168); cities (17, 17, 2)
    // -> (17, 17, 1); world (9056, 7469, 5567) -> (9056, 9056, 7).
    // Item 4209, the box's `clear_down`: items (0, 35, 35) -> (0, 0, 0); world (9056, 9056, 7) ->
    // (9056, 9056, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4466, 4844, 3921) -> (4466, 4844, 3924).
    (
        "run959",
        18368,
        [
            (4466, 4844, 3924),
            (1832, 1832, 35),
            (0, 0, 0),
            (9, 0, 9),
            (0, 0, 0),
            (562, 562, 5),
            (1514, 1668, 168),
            (271, 271, 3),
            (17, 17, 1),
            (0, 0, 0),
            (330, 330, 1),
            (9056, 9056, 0),
        ],
    ),
    // Item 4205, the built start: units (3135, 3263, 2550) -> (3135, 3263, 2517); builds (2052,
    // 2052, 46) -> (2052, 2052, 33); cities (35, 35, 4) -> (35, 35, 3); world (7513, 7497,
    // 4325) -> (7513, 7513, 9).
    // Item 4209, the box's `clear_down`: items (60, 110, 69) -> (60, 60, 0); world (7513, 7513, 9) ->
    // (7513, 7513, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3135, 3263, 2517) -> (3135, 3263, 2518).
    (
        "run973g",
        12286,
        [
            (3135, 3263, 2518),
            (2052, 2052, 33),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (532, 532, 0),
            (941, 1046, 106),
            (271, 271, 2),
            (35, 35, 3),
            (60, 60, 0),
            (120, 120, 0),
            (7513, 7513, 0),
        ],
    ),
    // Item 4205, the built start: units (2003, 2230, 1758) -> (2003, 2230, 1723); builds (789,
    // 789, 20) -> (789, 789, 7); guys (533, 638, 141) -> (533, 638, 105); world (8052, 7333,
    // 4648) -> (8052, 8052, 7).
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0); world (8052, 8052, 7) ->
    // (8052, 8052, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2003, 2230, 1723) -> (2003, 2230, 1722).
    (
        "run984b",
        6011,
        [
            (2003, 2230, 1722),
            (789, 789, 7),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (533, 638, 105),
            (271, 271, 2),
            (17, 17, 1),
            (75, 75, 0),
            (180, 180, 0),
            (8052, 8052, 0),
        ],
    ),
    // Item 4205, the built start: units (1903, 2165, 1626) -> (1903, 2165, 1594); builds (827,
    // 827, 21) -> (827, 827, 8); guys (540, 645, 141) -> (540, 645, 105); world (8078, 7359,
    // 4721) -> (8078, 8078, 7).
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0); world (8078, 8078, 7) ->
    // (8078, 8078, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1903, 2165, 1594) -> (1903, 2165, 1593).
    (
        "run984c",
        6251,
        [
            (1903, 2165, 1593),
            (827, 827, 8),
            (0, 0, 0),
            (3, 0, 3),
            (0, 0, 0),
            (538, 538, 0),
            (540, 645, 105),
            (271, 271, 2),
            (17, 17, 1),
            (75, 75, 0),
            (180, 180, 0),
            (8078, 8078, 0),
        ],
    ),
    // Item 4205, the built start: units (3289, 3742, 3058) -> (3289, 3742, 3041); builds (2783,
    // 2783, 54) -> (2783, 2783, 43); world (7443, 7427, 4174) -> (7443, 7443, 15).
    // Item 4209, the box's `clear_down`: items (40, 110, 85) -> (40, 40, 0); world (7443, 7443, 15) ->
    // (7443, 7443, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (3289, 3742, 3041) -> (3289, 3742, 3042).
    (
        "run985b",
        16762,
        [
            (3289, 3742, 3042),
            (2783, 2783, 43),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1012, 1222, 211),
            (271, 271, 2),
            (33, 33, 3),
            (40, 40, 0),
            (120, 120, 0),
            (7443, 7443, 1),
        ],
    ),
    // Item 4205, the built start: units (3178, 3504, 2770) -> (3178, 3504, 2733); builds (2006,
    // 2006, 47) -> (2006, 2006, 32); world (7461, 7445, 4143) -> (7461, 7461, 11).
    // Item 4209, the box's `clear_down`: items (55, 110, 74) -> (55, 55, 0); world (7461, 7461, 11) ->
    // (7461, 7461, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3178, 3504, 2733) -> (3178, 3504, 2735).
    (
        "run991d",
        15256,
        [
            (3178, 3504, 2735),
            (2006, 2006, 32),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (961, 1136, 179),
            (271, 271, 2),
            (30, 30, 3),
            (55, 55, 0),
            (120, 120, 0),
            (7461, 7461, 0),
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
    ("run1001d", "resources_controlled"),
    ("run1007b", "resources_controlled"),
    ("run1011b", "flags.leader_flags resources_controlled"),
    ("run1011c", "resources_controlled"),
    ("run1011d", "resources_controlled"),
    ("run1011e", "flags.leader_flags resources_controlled"),
    ("run1015b", "resources_controlled"),
    (
        "run1155c",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1161", "flags.leader_flags"),
    (
        "run1164",
        "encrypt.bucket[2] encrypt.bucket[5] explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1165",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1166",
        "collected encrypt.bucket[0] encrypt.bucket[1] encrypt.leftover[0] escrow explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1169",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1185",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1186b", "flags.leader_flags"),
    ("run1217", "flags.leader_flags resources_controlled"),
    (
        "run1220b",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1221", "flags.leader_flags resources_controlled"),
    (
        "run1224b",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1235b",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1236",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1237",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1238c", "flags.leader_flags resources_controlled"),
    (
        "run1247",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1254c",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1263", "flags.leader_flags resources_controlled"),
    ("run1264b", "flags.leader_flags resources_controlled"),
    (
        "run1266c",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1268",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1293",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1294",
        "encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    (
        "run1296",
        "encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    ("run1304", "flags.leader_flags resources_controlled"),
    ("run1305b", "flags.leader_flags resources_controlled"),
    (
        "run1320b",
        "encrypt.bucket[2] flags.leader_flags resources_controlled",
    ),
    ("run1323b", "flags.leader_flags resources_controlled"),
    (
        "run1324c",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1325c", "flags.leader_flags resources_controlled"),
    (
        "run1326",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1327",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1332a",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1351", "flags.leader_flags resources_controlled"),
    ("run1351c", "flags.leader_flags resources_controlled"),
    (
        "run1355",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1357",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359b",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1359c",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1361",
        "encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1381", "flags.leader_flags resources_controlled"),
    (
        "run1382",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1383", "flags.leader_flags"),
    ("run1385p", "flags.leader_flags resources_controlled"),
    (
        "run1391",
        "encrypt.bucket[0] encrypt.bucket[1] encrypt.bucket[2] encrypt.bucket[4] flags.leader_flags resources_controlled",
    ),
    ("run1416", "flags.leader_flags resources_controlled"),
    ("run1417", "flags.leader_flags resources_controlled"),
    (
        "run1419b",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1420",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1424",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1426b",
        "explored flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1437",
        "encrypt.bucket[2] encrypt.bucket[5] flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run1438", "aggression cities_built flags.leader_flags"),
    ("run1440", "aggression cities_built flags.leader_flags"),
    (
        "run1441",
        "flags.leader_flags reg_unpack_merch resources_controlled",
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
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1564",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run1570",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    (
        "run785",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run789",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run795",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch",
    ),
    (
        "run810",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run823",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    (
        "run825",
        "encrypt.bucket[4] encrypt.bucket[5] flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    ("run839", "aggression cities_built flags.leader_flags"),
    ("run842", "aggression cities_built flags.leader_flags"),
    ("run842b", "aggression cities_built flags.leader_flags"),
    (
        "run854",
        "diplos flags.leader_flags oil_patches.elements reg_unpack_merch resources_controlled",
    ),
    (
        "run865",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers reg_unpack_merch resources_controlled",
    ),
    ("run870", "aggression cities_built flags.leader_flags"),
    (
        "run882b",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers resources_controlled",
    ),
    (
        "run883b",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers resources_controlled",
    ),
    ("run893b", "flags.leader_flags"),
    ("run912e", "flags.leader_flags resources_controlled"),
    ("run938b", "aggression cities_built flags.leader_flags"),
    ("run943c", "flags.leader_flags resources_controlled"),
    (
        "run946",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run954b", "flags.leader_flags resources_controlled"),
    ("run958b", "flags.leader_flags"),
    (
        "run959",
        "flags.leader_flags reg_unpack_merch resources_controlled",
    ),
    ("run973g", "reg_unpack_merch resources_controlled"),
    ("run985b", "resources_controlled"),
    ("run991d", "resources_controlled"),
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

/// **Probe** (item 4205): one word's packet walked from the dump-seated
/// start and from the lobby's built start, the twelve lines on the packet's
/// frame from each, and the built walk's draws against the long trace to
/// that frame. `RON_PACKET_WORD=1` walks the built start on to the trace's
/// end for its word.
#[test]
#[ignore = "a probe: RON_PACKET_RUN, RON_PACKET_START, RON_PACKET_LONG, RON_PACKET_TRACE"]
fn probe_built_packet() {
    let run = std::env::var("RON_PACKET_RUN").expect("RON_PACKET_RUN");
    let start = std::env::var("RON_PACKET_START").expect("RON_PACKET_START");
    let long = std::env::var("RON_PACKET_LONG").expect("RON_PACKET_LONG");
    let trl = std::env::var("RON_PACKET_TRACE").expect("RON_PACKET_TRACE");
    let (frame, theirs) = packet_stream(&run).expect("the packet's stream");
    let pieces = Pieces::read().expect("the install");
    let show = |side: &str, o: &[LineOutcome]| {
        let agree = o.iter().filter(|x| x.agrees()).count();
        let counted = o.iter().filter(|x| x.counted()).count();
        eprintln!(
            "{side} frame {frame}: {agree} of {counted} counted; pin {:?}",
            pin_of(o)
        );
        for (n, x) in STATE_LINES.iter().zip(o) {
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
    };
    let held: std::collections::BTreeSet<&str> = LEADERS_HELD_OUT.iter().map(|(n, _)| *n).collect();
    let (mut seated, _) = stand_up_from(&start, (&long, &trl)).expect("the captures");
    while seated.sim.frame < frame {
        seated.tick();
    }
    let s = compare(&our_lines(&seated.sim, &pieces), &theirs);
    let empty = StreamRows::new();
    let sl = leader_leaf_rows(&seated.sim, theirs.get("leaders").unwrap_or(&empty));
    show("seated", &s);
    drop(seated);
    let (mut built, tr) = crate::diff::third::stand_up_built((&long, &trl)).expect("the captures");
    let mut parts = None;
    while built.sim.frame < frame {
        built.tick();
        if parts.is_none()
            && let Some((f, ours)) = built.frame_sites.last()
            && ours.len() != tr.labels(*f).len()
        {
            parts = Some(*f);
        }
    }
    eprintln!("built: draws to frame {frame} first part (count) at {parts:?}");
    let b = compare(&our_lines(&built.sim, &pieces), &theirs);
    let bl = leader_leaf_rows(&built.sim, theirs.get("leaders").unwrap_or(&empty));
    show("built", &b);
    for (side, leaves) in [("seated", &sl), ("built", &bl)] {
        let w = Walked {
            run: "probe",
            frame,
            lines: vec![],
            leaves: leaves.clone(),
        };
        eprintln!(
            "{side} leaves parting outside the list: {}",
            extra_leaves(&w, &held)
        );
    }
    for (slot, field, leaf, ours, t, _) in &bl {
        if ["cities_built", "aggression", "diplos"].contains(&leaf.as_str()) {
            eprintln!(
                "  built slot {slot} {field} {leaf}: ours {} theirs {}",
                ours.as_deref().map_or("-".into(), leaf_text),
                leaf_text(t)
            );
        }
    }
    if std::env::var_os("RON_PACKET_WORD").is_some() {
        drop(built);
        let (fresh, _) = crate::diff::third::stand_up_built((&long, &trl)).expect("the captures");
        let w = crate::diff::third::walk_word(fresh, &tr, &long);
        eprintln!(
            "built word: count {} sequence {} of {}; {}",
            w.count, w.sequence, w.last, w.row
        );
    }
}

/// **Probe** (item 4205): one word's two frame zeros, the dump-seated and
/// the lobby-built, compared whole (`start_walk::whole_diff`, take by
/// take), and the seated start's corrections by frame. `RON_PACKET_TAKE`
/// filters the takes printed by a substring. `RON_PACKET_FOG` prints each
/// `seen2` cell the two starts differ on, and both sides' buildings and
/// units by fog cell (item 4210); `RON_PACKET_FOGALL` every cell.
#[test]
#[ignore = "a probe: RON_PACKET_START, RON_PACKET_LONG, RON_PACKET_TRACE"]
fn probe_built_start_whole() {
    let start = std::env::var("RON_PACKET_START").expect("RON_PACKET_START");
    let long = std::env::var("RON_PACKET_LONG").expect("RON_PACKET_LONG");
    let trl = std::env::var("RON_PACKET_TRACE").expect("RON_PACKET_TRACE");
    let only = std::env::var("RON_PACKET_TAKE").unwrap_or_default();
    let (seated, _) = stand_up_from(&start, (&long, &trl)).expect("the captures");
    let (built, _) = crate::diff::third::stand_up_built((&long, &trl)).expect("the captures");
    eprintln!(
        "seated corrections: seeds at {:?}; clocks at {:?}",
        seated
            .frame_seeds
            .iter()
            .map(|s| s.0)
            .take(20)
            .collect::<Vec<_>>(),
        seated
            .frame_guys
            .iter()
            .map(|g| (g.0, g.1.len()))
            .take(20)
            .collect::<Vec<_>>()
    );
    eprintln!(
        "rng: seated {:#010x} built {:#010x}",
        seated.sim.rng.seed, built.sim.rng.seed
    );
    if let Ok(prefix) = std::env::var("RON_PACKET_LEAVES") {
        use crate::diff::start_walk::{debug_leaves, keyed};
        let a = keyed(&built.sim, debug_leaves(&format!("{:#?}", built.sim)));
        let b = keyed(&seated.sim, debug_leaves(&format!("{:#?}", seated.sim)));
        let bm: std::collections::HashMap<&str, &str> =
            b.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect();
        for (p, v) in &a {
            if p.starts_with(prefix.as_str()) && bm.get(p.as_str()) != Some(&v.as_str()) {
                eprintln!("LEAF {p}: built {v} seated {:?}", bm.get(p.as_str()));
            }
        }
    }
    if std::env::var_os("RON_PACKET_FOG").is_some() {
        let w = built.sim.world.width() * 2;
        for fy in 0..w {
            for fx in 0..w {
                let (a, b) = (
                    built.sim.world.seen2(fx, fy),
                    seated.sim.world.seen2(fx, fy),
                );
                if a != b {
                    eprintln!("FOG ({fx},{fy}) built {a:?} seated {b:?}");
                }
                if std::env::var_os("RON_PACKET_FOGALL").is_some() {
                    eprintln!("CELL {fx} {fy} {} {}", a.unwrap_or(0), b.unwrap_or(0));
                }
            }
        }
        for (side, s) in [("built", &built.sim), ("seated", &seated.sim)] {
            for (i, b) in s.buildings.iter().enumerate() {
                eprintln!(
                    "BLD {side} {i} {}/{} ty {:?} {:?} fog ({},{}) mylos {} city {}",
                    b.owner,
                    b.index,
                    b.ty,
                    b.ty.map(|t| s.tables.build_types[t].ident),
                    sim::vision::fog_of(b.pos.x),
                    sim::vision::fog_of(b.pos.y),
                    b.mylos,
                    s.building_is_city(i)
                );
            }
            for (i, u) in s.units.iter().enumerate() {
                eprintln!(
                    "UNIT {side} {}/{} ty {:?} TI {:#x} {:?} fog ({},{}) los {}",
                    u.owner,
                    u.index,
                    u.ty,
                    u.type_index,
                    u.kind.domain,
                    sim::vision::fog_of(u.pos.x),
                    sim::vision::fog_of(u.pos.y),
                    s.unit_los(i)
                );
            }
        }
    }
    let d = crate::diff::start_walk::whole_diff(&built.sim, &seated.sim);
    for (take, (n, first, o, t)) in &d {
        if take.contains(only.as_str()) {
            let cut = |s: &Option<String>| {
                s.as_deref()
                    .map_or("-".to_string(), |s| s.chars().take(60).collect())
            };
            eprintln!(
                "TAKE {take}: {n}; first {first}: built {} seated {}",
                cut(o),
                cut(t)
            );
        }
    }
}

/// **Probe** (item 4208): the built and the dump-seated starts ticked to
/// frame 3, every gaia figure's guy 0 clock printed where the two differ.
#[test]
#[ignore = "a probe: RON_PACKET_START, RON_PACKET_LONG, RON_PACKET_TRACE"]
fn probe_built_clocks() {
    let start = std::env::var("RON_PACKET_START").expect("RON_PACKET_START");
    let long = std::env::var("RON_PACKET_LONG").expect("RON_PACKET_LONG");
    let trl = std::env::var("RON_PACKET_TRACE").expect("RON_PACKET_TRACE");
    let (mut seated, _) = stand_up_from(&start, (&long, &trl)).expect("the captures");
    let (mut built, _) = crate::diff::third::stand_up_built((&long, &trl)).expect("the captures");
    let row = |s: &sim::Sim, owner: u8, index: i16| {
        s.units
            .iter()
            .find(|u| u.owner == owner && u.index == index)
            .map(|u| {
                let g = &u.guys[0];
                format!(
                    "ty {} anim {} cur {} end {} last {} piece {}",
                    u.type_index, g.anim, g.cur_time, g.end_time, g.last_time, g.gpiece
                )
            })
    };
    for f in 0..=3 {
        let mut keys: Vec<(u8, i16)> = built
            .sim
            .units
            .iter()
            .chain(seated.sim.units.iter())
            .filter(|u| u.owner >= 8)
            .map(|u| (u.owner, u.index))
            .collect();
        keys.sort();
        keys.dedup();
        for (o, i) in keys {
            let (b, s) = (row(&built.sim, o, i), row(&seated.sim, o, i));
            let strip = |r: &Option<String>| {
                r.as_ref()
                    .map(|r| r.split(" piece").next().unwrap_or("").to_string())
            };
            if strip(&b) != strip(&s) || std::env::var_os("RON_PACKET_ALL").is_some() {
                eprintln!("CLOCK f{f} {o}/{i}: built {b:?}\n                seated {s:?}");
            }
        }
        built.tick();
        seated.tick();
    }
}

/// run1575's packet at 1090, pinned as found (item 4159, step VII-b2):
/// each line's (the stream's ranges, ours, ranges parted), our walk the
/// take's ([`run1575_take_packet`]). One line agrees, the cities' (12
/// ranges). Item 4218, the idle latch (`flags & 0x08`): units (3907, 4015,
/// 3084) -> (3907, 4015, 3077). Item 4159's step VII-c, `action_treaty` and
/// the computed accept: leaders (407, 407, 8) -> (407, 407, 7), the deal
/// records' `treaty` (2 on both seats' records, ours −1 before) agreeing.
const TAKE_PINNED: Pinned = (
    "run1575",
    1090,
    [
        (3907, 4015, 3077),
        (1709, 1709, 35),
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
        (556, 556, 2),
        (1289, 1331, 166),
        (407, 407, 7),
        (12, 12, 0),
        (25, 25, 0),
        (405, 405, 1),
        (8981, 7447, 5432),
    ],
);

/// The leaves of run1575's leaders line that part outside
/// [`LEADERS_HELD_OUT`] at 1090, as found: `cities_built` (theirs 1, 1, 2
/// on the three seats, ours 0, 0, 1); `dip.agree` (the human's record of the
/// French: theirs 0, ours 1 — the French answer to the alliance accepted on
/// 1051 is found short and withdraws both agreements, an arm step VII-d
/// ports; ours stops before answering, so the human's commitment stands);
/// `diplos` (slots 3..7 read 1 there, `Leader::init`'s peace at style 0 on
/// every slot, which the take's re-read lobby writes on 0..2 only);
/// `flags.leader_flags` (high bits on every seat). Until step VII-c
/// `dip.treaty` parted too (the alliance asked on 1041, ours −1 with no
/// `action_treaty`).
const TAKE_LEAF_PARTINGS: &str = "cities_built dip.agree diplos flags.leader_flags";

/// **The unprinted leaves the take's packet releases** (item 4159, step
/// VII-c; `docs/DECISIONS.md` entry 131 §1): each written on 906 (the
/// ring) or 966 (the drain and the gift) and by nothing after in our walk,
/// which holds the game unstaged to the packet since the French accept of
/// 1027 is computed (entry 132 §1); the accept's exchange gives no goods.
/// `(leaf, run, slot)`, each off its seated image on the packet, ours equal
/// ([`hold_witnesses`]). Held off the line at VII-b2 (the driver's ruling)
/// while 1090 followed a staged decision.
pub(super) const TAKE_RELEASED: &[(&str, &str, i16)] = &[
    // The French ring: the cursor 1, the request 1, due 966.
    ("incoming", "run1575", 1),
    ("incoming_taunt", "run1575", 1),
    ("incoming_taunt_frame", "run1575", 1),
    // The French gave 3,290 food, the human received 1,909.
    ("tribute_sent", "run1575", 1),
    ("tribute_received", "run1575", 0),
];

/// **run1575's packet at 1090, held by the take** (item 4159, step
/// VII-b2): the take's walk (`diff::diplomacy_take`: run1482's start at
/// style 0, the script, the human's window commands off the record, the
/// chat minted) ticked to the packet's frame, its twelve lines against the
/// instrument's stream. Off the scoreboard ([`TAKES_HELD`]). Since step
/// VII-c no decision on the way is staged, and the five unprinted leaves
/// of the taunt's answer are released here ([`TAKE_RELEASED`]).
#[test]
fn run1575_take_packet() {
    use crate::diff::diplomacy_take as take;
    let (run, frame, name, length) = TAKES_HELD[0];
    let (Some(diplomacy), Some(taunts)) = (
        take::take_diplomacy(take::RUN1575.0),
        take::script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!("skipping: no {} or install", take::RUN1575.0);
        return;
    };
    let Some(walked) = walk_packets_with(name, length, &[(run, frame)], &mut |frames, probe| {
        let script = take::coverage_script("diplomacy4206.cmd");
        let (_, mut built) = take::stand_up_sea_style_zero(take::RUN1575, script)?;
        for &f in frames {
            while built.sim.frame < f {
                take::stage_frame(&mut built, 3, &[], &diplomacy, &taunts);
                built.tick();
            }
            probe(f, &built);
        }
        Some(())
    }) else {
        return;
    };
    let w = &walked[0];
    let held: std::collections::BTreeSet<&str> = LEADERS_HELD_OUT.iter().map(|(n, _)| *n).collect();
    for (n, l) in STATE_LINES.iter().zip(&w.lines) {
        if let Some(f) = &l.first {
            eprintln!(
                "  {n}: {} of {} ranges part; first range {} record {} {}",
                l.parted, l.theirs, f.0, f.1, f.2
            );
        }
    }
    for (slot, field, leaf, ours, t, _) in &w.leaves {
        if ours.as_ref() != Some(t) {
            eprintln!(
                "LEAF\t{slot}\t{}\t{}\t{}",
                leader_held_name(field, leaf, ours.is_none()),
                leaf_text(t),
                ours.as_deref().map_or("-".into(), leaf_text)
            );
        }
    }
    eprintln!("PIN\t(\"{}\", {}, {:?}),", w.run, w.frame, pin_of(&w.lines));
    eprintln!("EXTRA\t{}", extra_leaves(w, &held));
    assert_eq!(
        (TAKE_PINNED.0, TAKE_PINNED.1, TAKE_PINNED.2),
        (w.run, w.frame, pin_of(&w.lines)),
        "run1575: each line's (the stream's ranges, ours, ranges parted)"
    );
    assert_eq!(
        extra_leaves(w, &held),
        TAKE_LEAF_PARTINGS,
        "run1575: the leaders line's leaves parting outside LEADERS_HELD_OUT"
    );
    hold_witnesses(w, &held, TAKE_RELEASED);
}
