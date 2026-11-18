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
/// that part, and the first differing byte in walk order.
#[derive(Clone, Debug)]
pub(in crate::diff) struct LineOutcome {
    pub theirs: usize,
    pub ours: usize,
    pub parted: usize,
    pub first: Option<FirstDifference>,
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

/// Our walk's twelve lines, in [`STATE_LINES`]' order. Unavailable record shapes contribute
/// no ranges, so every opposing range counts as differing.
pub(in crate::diff) fn our_lines(sim: &sim::Sim, pieces: &Pieces) -> Vec<Line> {
    crate::checksum::state::ranges(sim, &pieces.look, &pieces.tribes)
        .map(Option::unwrap_or_default)
        .into()
}

/// Keep the first pair of byte strings for field decoders used by recorded-frame tests.
#[derive(Clone, Debug)]
pub(in crate::diff) struct FirstDifference {
    pub index: usize,
    pub record: i16,
    pub field: String,
    pub offset: usize,
    pub ours: Option<Vec<u8>>,
    pub theirs: Option<Vec<u8>>,
}

impl FirstDifference {
    fn at(line: &Line, stream: &StreamRows, index: usize) -> Self {
        let o = line.ranges.get(index);
        let t = stream.get(index);
        let offset = match (o, t) {
            (Some(a), Some(b)) => a
                .bytes
                .iter()
                .zip(&b.2)
                .position(|(x, y)| x != y)
                .unwrap_or_else(|| a.bytes.len().min(b.2.len())),
            _ => 0,
        };
        let field = o
            .map(|r| r.field)
            .unwrap_or_else(|| t.expect("a differing range exists on one side").0.as_str());
        Self {
            index,
            record: o.map_or(-1, |r| r.record),
            field: field.to_owned(),
            offset,
            ours: o.map(|r| r.bytes.clone()),
            theirs: t.map(|r| r.2.clone()),
        }
    }
}

impl std::fmt::Display for FirstDifference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = |range: Option<&[u8]>| match range {
            None => "absent".to_owned(),
            Some(bytes) => bytes
                .get(self.offset)
                .map_or_else(|| "end".to_owned(), |x| format!("{x:02x}")),
        };
        write!(
            f,
            "range {} record {} {} byte {}: ours {} theirs {}",
            self.index,
            self.record,
            self.field,
            self.offset,
            value(self.ours.as_deref()),
            value(self.theirs.as_deref())
        )
    }
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
            let mut differences = differing_ranges(o, t);
            let first = differences.next().map(|i| FirstDifference::at(o, t, i));
            LineOutcome {
                theirs: t.len(),
                ours: o.ranges.len(),
                parted: usize::from(first.is_some()) + differences.count(),
                first,
            }
        })
        .collect()
}

#[test]
fn packet_first_difference_distinguishes_bytes_ends_and_absent_ranges() {
    let common = vec![0x24; 80];
    let mut late = common.clone();
    late[71] = 0x52;
    for (ours, theirs, expected, count) in [
        (
            vec![common.clone(), vec![]],
            vec![late, vec![1]],
            "range 0 record 12 field byte 71: ours 24 theirs 52",
            2,
        ),
        (
            vec![vec![1]],
            vec![vec![1, 2]],
            "range 0 record 12 field byte 1: ours end theirs 02",
            1,
        ),
        (
            vec![vec![1, 2]],
            vec![vec![1]],
            "range 0 record 12 field byte 1: ours 02 theirs end",
            1,
        ),
        (
            vec![common.clone()],
            vec![common.clone(), vec![]],
            "range 1 record -1 original byte 0: ours absent theirs end",
            1,
        ),
        (
            vec![common.clone(), vec![]],
            vec![common.clone()],
            "range 1 record 12 field byte 0: ours end theirs absent",
            1,
        ),
        (vec![common.clone()], vec![common], "", 0),
    ] {
        let line = Line {
            name: "units",
            ranges: ours
                .into_iter()
                .map(|bytes| sim::checksum::Range {
                    record: 12,
                    field: "field",
                    bytes,
                })
                .collect(),
        };
        let stream: StreamRows = theirs
            .into_iter()
            .map(|bytes| ("original".into(), String::new(), bytes))
            .collect();
        let detailed = parting(&line, &stream);
        let original = std::collections::BTreeMap::from([("units".into(), stream)]);
        let result = compare(&[line], &original);
        assert_eq!((detailed.len(), result[0].parted), (count, count));
        assert_eq!(
            result[0].first.as_ref().map(ToString::to_string).as_deref(),
            (count > 0).then_some(expected)
        );
    }
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
/// `RON_PACKETS_PRINT` prints the pins and every leaf row, asserts none,
/// and prints a `MOVED` line for each pin the walk moved.
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
                eprintln!("  {n}: {} of {} ranges part; first {f}", l.parted, l.theirs);
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
            // pins are read off it, so they are not held here. It says so,
            // and names a pin the walk moved (item 4238 read a print run
            // as "nothing moved" and its lane gate went red on two rows).
            match PINNED.iter().find(|p| p.0 == w.run) {
                Some(p) if (p.1, p.2) != (w.frame, pin_of(&w.lines)) => {
                    eprintln!("MOVED\t{}\tpinned {:?}", w.run, p.2);
                }
                Some(_) => {}
                None => eprintln!("MOVED\t{}\tno pinned outcome", w.run),
            }
            eprintln!(
                "PRINT\t{}\tRON_PACKETS_PRINT asserts no pin: a MOVED line is a pin this walk moved",
                w.run
            );
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
    // Item 4159's residue: run1496, frame 1539, leader 1: 84 (was ours 96,
    // the empty Transport Freighter sunk on 818 priced at 12).
    ("score_combat", "run1496", 1),
    // run912e, frame 12753, leader 1: 672 and 549 (was ours 665 and 532,
    // stale since the Merchant Fleet's close on 12610 raised no bit).
    ("score_units", "run912e", 1),
    ("score_units_2", "run912e", 1),
    // Item 4159's ninth residue, the thirteen combat leaves (released by
    // entry 131 §1's first way: no record diff parts them; each agrees on
    // all 226 walked leaders). run1496, frame 1539, leader 1: killed 4,
    // lost 5; the averages 55 (death), 287 (damage), 131 (hit); this
    // frame's hits 1 and damage 1; the fifteen-second deaths 5, kills 4,
    // hits 132, damage 195.
    ("units_killed", "run1496", 1),
    ("units_lost", "run1496", 1),
    ("average_death_rate", "run1496", 1),
    ("average_damage_rate", "run1496", 1),
    ("average_hit_rate", "run1496", 1),
    ("hits_current_frame", "run1496", 1),
    ("damage_current_frame", "run1496", 1),
    ("deaths_fifteen_seconds", "run1496", 1),
    ("kills_fifteen_seconds", "run1496", 1),
    ("hits_fifteen_seconds", "run1496", 1),
    ("damage_fifteen_seconds", "run1496", 1),
    // run1570, frame 3331: leader 0's death this frame (1), leader 1's
    // kill (1); ten of the 226 leaders hold either word off zero.
    ("deaths_current_frame", "run1570", 0),
    ("kills_current_frame", "run1570", 1),
    // Item 4159's sixteenth residue (4159r15a), the score (released by
    // entry 131 §1's first way: no record diff parts it; it agrees on all
    // 226 walked leaders, non-zero on all 226). run1161, frame 4, leader 0:
    // 280.
    ("score", "run1161", 0),
    // Item 4159's seventeenth residue, the three victim stamps (released
    // by entry 131 §1's first way: no record diff parts a cell; each agrees
    // on all 226 walked leaders, non-zero on 95, 66 and 34). run1359,
    // frame 5619, leader 0: `attack_stamp[1]` and `raid_stamp[1]` 5568,
    // `ally_stamp[1]` 5394.
    ("attack_stamp", "run1359", 0),
    ("raid_stamp", "run1359", 0),
    ("ally_stamp", "run1359", 0),
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
pub(super) const NOT_HELD: &[(&str, i64, &str)] = &[
    (
        "run1100",
        5408,
        "run1090's lobby with `100 ally 1` and `5404 explore all` staged; no word walks it",
    ),
    (
        "run1603",
        6860,
        "the rallied file's playback (item 4256): its ammo pool is held by \
         `minted::the_rallied_game_s_round_in_flight_is_the_packet_s`; no word walks it",
    ),
    (
        "run1612",
        3294,
        "run1559b's take with `early_autumn`'s tint zeroed (item 4272): a renderer's \
         packet beside its photograph; no word walks it",
    ),
    (
        "run1608",
        2553,
        "run1075's line with a packet at 2553 (item 4159's eighth residue): read by \
         step4 for the chase's re-pushed goal; no word walks it",
    ),
    (
        "run1615",
        6865,
        "the buildmasked file's playback, take 1 (item 4277): its ammo pool empty, as \
         ours; no word walks it",
    ),
    (
        "run1615b",
        6869,
        "the buildmasked file's playback, take 2 (item 4277): its ammo pool is held by \
         `minted::the_buildmasked_game_s_round_in_flight_is_the_packet_s`, piece 384's \
         bays swept on it; no word walks it",
    ),
    (
        "run1640",
        6079,
        "run1575's lobby at DIFFICULTY 4 with no script (item 4300a): read by step4 for \
         the demand arm's war branch; our walk parts on frame 5; no word walks it",
    ),
    (
        "run1655",
        5621,
        "run1646's lobby with a packet at 5621 (item 4354): army 2/1's target, point and \
         muster read off it, the value diff behind RUN1646_WORD 5668; no word walks it",
    ),
    (
        "run1655b",
        5496,
        "run1646's lobby with a packet at 5496 (item 4354): Army::find_target stepped on \
         it, 13 of 13 draws, the French 1/2040 chosen; no word walks it",
    ),
];

/// **The takes' packets** (item 4159, step VII-b2): held by a take's walk
/// (`diff::diplomacy_take`), which stages a computer's decision off the
/// record, so no scoreboard line counts them (`docs/DECISIONS.md` entry
/// 132 §1) and [`PINNED`] does not hold them. `(run, logger frame, the
/// take, the length its staged walk is whole to)`.
pub(super) const TAKES_HELD: &[(&str, i64, &str, i64)] = &[
    ("run1575", 1090, "the diplomacy take (1099 computed)", 1150),
    ("run1629", 1190, "the attack deal's take (item 4298)", 1300),
    (
        "run1645",
        1000,
        "the reseated taunt's take (item 4313)",
        1100,
    ),
];

/// The census's counts: `(packets on disk, held by a word's walk, by a
/// chapter's, by a melee's, by the minted walk, by a take's, not held)`.
// Item 4206: (108, …, 1) → (109, …, 2), run1575's packet at 1090 named.
// Item 4159, step VII-b2: run1575's packet held by its take, (…, 1, 1).
// Item 4256: run1603's packet at 6860 named, not held, (110, …, 2).
// Item 4272: run1612's packet at 3294 named, not held, (111, …, 3).
// Item 4159's eighth residue: run1608's packet at 2553 named, not held, (112, …, 4).
// Item 4277: run1615's at 6865 and run1615b's at 6869 named, not held, (114, …, 6).
// Item 4300a: run1640's packet at 6079 named, not held, (115, …, 7).
// Item 4298: run1629's packet at 1190 held by its take, (116, …, 2, 7).
// Item 4313: run1645's packet at 1000 held by its take, (117, …, 3, 7).
// Item 4354: run1655's at 5621 and run1655b's at 5496 named, not held, (119, …, 9).
const CENSUS: (usize, usize, usize, usize, usize, usize, usize) = (119, 87, 7, 12, 1, 3, 9);

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
/// 1787's `down` ours −3 theirs −1, `down_who` 2 both). **285** at item
/// 4222, a queue's vacated slots walked as the array holds them, a removed
/// last entry's `job_counter` zeroed and the tech arm's unqueue after
/// `gain_tech` (`docs/PRODUCTION.md` "The queue record"): the builds line
/// agrees on run1247 (1692 ranges, 27 parting → 0), run893b (709, 7 → 0),
/// run958b (455, 2 → 0), run984b (789, 7 → 0) and run984c (827, 8 → 0), and
/// parts on fewer ranges on 92 more, none on more. **314** at item 4226,
/// the units and guys lines walking slots 0 to 8 as
/// `CheckSums::check_units@009371d0` and `check_guys@00937430` do, so a
/// bird (slot 9) is on neither: on 106 of the 107 the guys line's ranges
/// are now the stream's count, and it agrees whole on 28 (run1161's
/// (924, 994, 70) → (924, 924, 0) among them); the units line agrees on
/// run1161 (2760 ranges, 205 parting → 0). **318** at item 4228, a unit's
/// path stack walking the `size` it grew to (`sim::orders::PathStack`):
/// the units line agrees on run893b (2652 ranges, 6 parting → 0), run958b
/// (490, 1 → 0), run1186b (1060, 10 → 0) and run1383 (2933, 13 → 0), and
/// parts on fewer ranges on 92 more. On run1460 it parts on 2 more (1872 →
/// 1874), three ranges of the units ours walks out of step with theirs (ours
/// 2,275 ranges, theirs 2,380: our `1/10`, `2/6`, `2/8` against their `1/8`,
/// `2/3`, `2/5`) against the one aligned range now agreeing (`0/2`, 30).
/// **421** at item 4229: the cities line agrees on 77 more packets
/// (`City::new_caravan`'s bit, the partner's index in its owner's cities)
/// and the goods line on 30 more (`Animal::work`'s herd arm); the goods
/// line parts on one more range on four, run1484, run1487, run1496 and
/// run839, each a walk whose units line already parts on most of its
/// ranges (a named regression, `docs/journal/2026-11-14-item-4229.md`).
/// **422** at item 4238, an age's seating writing each figure's `last`:
/// the guys line agrees on run1236 (2070 ranges, 33 parting → 0), and
/// parts on fewer on run1238c (7 → 1) and run1264b (20 → 12); the units
/// line parts on fewer on run1236 (149 → 116) and run1238c (4079 → 4073).
/// **425** at item 4244, a mine keeping `MiningList::mtn`/`cliff`: the
/// builds line agrees on run1237 (1650 ranges, 1 parting → 0), run1383
/// (1184, 1 → 0) and run1186b (1164, 1 → 0), and parts on fewer on 78 more
/// packets and run1575's take (27 → 26), none on more: 154 ranges fewer
/// over 82 rows.
/// **433** at item 4253, a scholar's seating setting its figure's angle
/// (`Unit::go_inside@0061a2e0`'s `set_angle(0, _, 1)`): the guys line
/// agrees on eight more packets — run985b (1 → 0), run1007b (5 → 0),
/// run1001d (1 → 0), run954b (4 → 0), run991d (4 → 0), run1263 (5 → 0),
/// run1224b (7 → 0), run959 (14 → 0) — and parts on fewer on nine
/// (run1266c 21 → 9, run882b 33 → 26, run883b 31 → 24, the most); the
/// units line parts on fewer on seven (run959 3128 → 3114, the most).
/// **446** at item 4255, `equals_group`'s `get_num` zeroing an emptied last
/// slot's `ox`/`oy`: the groups line agrees on 13 more packets (run1457,
/// run1460, run1459, run1417, run1420, run1438, run1440, run1451, run1237,
/// run1263, run1361, run1224b, run1304) and parts on fewer on 27 more, none
/// on more: 55 ranges fewer over 40 rows.
/// **452** at item 4257, the garrison chain's order
/// (`Object::insert_inside@00647e90` appends at the bottom): the builds
/// line agrees on six more packets — run912e (2 → 0), run1007b (4 → 0),
/// run943c (2 → 0), run991d (2 → 0), run959 (2 → 0), run954b (2 → 0) —
/// and 40 rows fall, none rises (units on ten, run1011b 1854 → 1830 the
/// most).
/// **453** at item 4260, a trackless crew figure's `last_angle` read after
/// guy 0's turn (`Guy::do_turn@005d97a0`): the guys line agrees on run1254c
/// (2 → 0); no other row moved.
/// **455** at item 4262, a wonder's `BuildData::wonder` its entry in its
/// owner's wonder list (`Build::activate@00623e20`'s store of
/// `Wonders::init_wonder@0073c860`): the builds line agrees on run1305b
/// (2 → 0) and run1235b (1 → 0), and parts on fewer on 27 more, none on
/// more: 63 ranges fewer over 29 rows, all in the builds slot.
/// **458** at item 4298, `hire_stamp`, `hire_who` and `tributes` on the
/// line (`hire_who`'s −1 per slot, `Leader::init`'s fill, emitted): the
/// leaders line agrees on run1562 (267 ranges, 2 parting → 0), run984b
/// (271, 2 → 0) and run984c (271, 2 → 0), and parts on fewer on 78 more
/// (73 by one range, 5 by two), none on more: 89 ranges fewer over 81
/// rows, all in the leaders slot. Each of the three agrees with leaves still
/// held, at their reset on both sides.
/// **466** at item 4324, `resources_controlled` (`+0x850`) the census's
/// bit count of `rare` (`plan_strategy`'s inlined `BitMask<44>::count`;
/// it wrote 0): the leaders line agrees on run1001d, run1007b, run1011c,
/// run1011d, run1011e, run1015b, run985b and run991d (271 ranges, 1 → 0
/// each), and parts on one range fewer on 23 more, none on more: 31
/// ranges fewer over 31 rows, all in the leaders slot.
/// **467** at item 4328, `reg_unpack_merch` (`+0x5d4`) leaving a packed
/// merchant out (`plan_strategy:654`–`660`; it counted every merchant):
/// the leaders line agrees on run973g (271 ranges, 1 → 0), and parts on
/// one range fewer on 38 more, none on more: 39 ranges fewer over 39
/// rows, all in the leaders slot.
/// **482** at item 4332, `Unit::think`'s citizen bit (`leader_flags &
/// 0x80000`) emitted: the leaders line agrees on 15 more packets (run954b,
/// run1011b, run1155c, run1186b, run1220b, run1237, run1254c, run1268,
/// run1305b, run1323b, run1351, run1355, run1357, run1381, run1383), and
/// parts on fewer ranges on 32 more, none on more: 50 ranges fewer over 47
/// rows, all in the leaders slot.
/// Item 4348: lines agreeing 482 → 499 of 1,053: gaia's angles from the
/// start dump, the `guys` line whole on seventeen packets.
/// Item 4363: chapter 94 adds the occupied deaths line (16 of 16 ranges).
/// Item 4365 adds 43 whole occupied ammo lines and six whole casualty lines.
/// Item 4366 adds four whole units lines and nine whole builds lines.
/// Item 4368 adds 22 whole casualty lines.
/// Item 4369 adds six whole casualty lines.
/// Item 4370 closes the last two newly emitted casualty lines.
pub(crate) const PACKETS_SCORE: (usize, usize, usize) = (107, 592, 1053);

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
    // Item 4226, a bird walked on neither line: units (1116, 1296, 306) -> (1116, 1116, 126), guys (382, 445, 99) -> (382, 382, 36).
    // Item 4228, a path stack's `size` its high water: units (1116, 1116, 126) -> (1116, 1116, 125).
    // Item 4348, gaia's angles from the start dump: units (1116, 1116, 125) -> (1116, 1116, 89), guys (382, 382, 36) -> (382, 382, 0).
    (
        "ch94p",
        846,
        [
            (1116, 1116, 89),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (16, 16, 0),
            (538, 538, 0),
            (382, 382, 0),
            (271, 271, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 0),
            (7996, 7277, 4488),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1284, 1426, 1224) -> (1284, 1426, 1222).
    // Item 4226, a bird walked on neither line: units (1284, 1426, 1222) -> (1284, 1266, 1080), guys (417, 473, 92) -> (417, 417, 36).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1284, 1266, 1080) -> (1284, 1266, 1079).
    // Item 4348, gaia's angles from the start dump: guys (417, 417, 36) -> (417, 417, 0).
    (
        "ch95pkt",
        1117,
        [
            (1284, 1284, 90),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (10, 10, 0),
            (538, 538, 0),
            (417, 417, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2028, 2028, 35)
    // -> (2028, 2028, 5).
    // Item 4226, a bird walked on neither line: units (2445, 2672, 1966) -> (2445, 2367, 1742), guys (701, 806, 106) -> (701, 701, 1).
    // Item 4228, a path stack's `size` its high water: units (2445, 2367, 1742) -> (2445, 2367, 1733).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2445, 2367, 1733) -> (2445, 2376, 1454).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (2445, 2376, 1454) -> (2445, 2376, 1453).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2445, 2376, 1453) -> (2445, 2376, 1450).
    // Item 4244, a mine's `mtn`: builds (2028, 2028, 5) -> (2028, 2028, 3).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (701, 701, 1) -> (701, 701, 0).
    // Item 4257, the garrison chain's order: units (2445, 2376, 1450) -> (2445, 2376, 1439); builds (2028, 2028, 3) -> (2028, 2028, 1).
    // Item 4266, a guard order's walker: units (2445, 2376, 1439) -> (2445, 2421, 1413).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1001d",
        13760,
        [
            (2445, 2445, 2),
            (2028, 2028, 0),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (544, 544, 0),
            (701, 701, 0),
            (271, 271, 0),
            (30, 30, 0),
            (5, 5, 0),
            (120, 120, 0),
            (7581, 7581, 0),
        ],
    ),
    // Item 4205, the built start: units (2939, 3438, 2660) -> (2939, 3438, 2627); builds (2417,
    // 2417, 58) -> (2417, 2417, 45); world (7537, 7521, 4313) -> (7537, 7537, 12).
    // Item 4209, the box's `clear_down`: units (2939, 3438, 2627) -> (2939, 3438, 2626); items (45,
    // 110, 83) -> (45, 45, 0); world (7537, 7537, 12) -> (7537, 7537, 0).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2417, 2417, 45)
    // -> (2417, 2417, 5).
    // Item 4226, a bird walked on neither line: units (2939, 3438, 2626) -> (2939, 2828, 2137), guys (916, 1126, 215) -> (916, 916, 5).
    // Item 4228, a path stack's `size` its high water: units (2939, 2828, 2137) -> (2939, 2828, 2131).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2939, 2828, 2131) -> (2939, 2837, 1616).
    // Item 4234, a cast's three ranges walked: units (2939, 2837, 1616) -> (2939, 2840, 1600).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (2939, 2840, 1600) -> (2939, 2840, 1598).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (2939, 2840, 1598) -> (2939, 2843, 1618).
    // Item 4244, a mine's `mtn`: builds (2417, 2417, 5) -> (2417, 2417, 4).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (916, 916, 5) -> (916, 916, 0).
    // Item 4257, the garrison chain's order: units (2939, 2843, 1618) -> (2939, 2843, 1601); builds (2417, 2417, 4) -> (2417, 2417, 0).
    // Item 4266, a guard order's walker: units (2939, 2843, 1601) -> (2939, 2933, 1108).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1007b",
        15915,
        [
            (2939, 2939, 3),
            (2417, 2417, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (916, 916, 0),
            (271, 271, 0),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2060, 2060, 35)
    // -> (2060, 2060, 5).
    // Item 4226, a bird walked on neither line: units (3219, 3345, 2593) -> (3219, 3045, 2483), guys (870, 975, 105) -> (870, 870, 0).
    // Item 4228, a path stack's `size` its high water: units (3219, 3045, 2483) -> (3219, 3045, 2475).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3219, 3045, 2475) -> (3219, 3054, 1858).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3219, 3054, 1858) -> (3219, 3054, 1856).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3219, 3054, 1856) -> (3219, 3054, 1854).
    // Item 4244, a mine's `mtn`: builds (2060, 2060, 5) -> (2060, 2060, 3).
    // Item 4257, the garrison chain's order: units (3219, 3054, 1854) -> (3219, 3054, 1830); builds (2060, 2060, 3) -> (2060, 2060, 1).
    // Item 4266, a guard order's walker: units (3219, 3054, 1830) -> (3219, 3198, 1420).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1011b",
        13105,
        [
            (3219, 3219, 0),
            (2060, 2060, 0),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (538, 538, 0),
            (870, 870, 0),
            (271, 271, 0),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2080, 2080, 33)
    // -> (2080, 2080, 5).
    // Item 4226, a bird walked on neither line: units (3193, 3298, 2589) -> (3193, 2998, 2492), guys (861, 966, 105) -> (861, 861, 0).
    // Item 4228, a path stack's `size` its high water: units (3193, 2998, 2492) -> (3193, 2998, 2485).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3193, 2998, 2485) -> (3193, 3007, 1823).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3193, 3007, 1823) -> (3193, 3007, 1821).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3193, 3007, 1821) -> (3193, 3007, 1818).
    // Item 4244, a mine's `mtn`: builds (2080, 2080, 5) -> (2080, 2080, 3).
    // Item 4257, the garrison chain's order: units (3193, 3007, 1818) -> (3193, 3007, 1794); builds (2080, 2080, 3) -> (2080, 2080, 1).
    // Item 4266, a guard order's walker: units (3193, 3007, 1794) -> (3193, 3151, 1830).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1011c",
        13225,
        [
            (3193, 3193, 1),
            (2080, 2080, 0),
            (0, 0, 0),
            (6, 6, 0),
            (4, 4, 0),
            (538, 538, 0),
            (861, 861, 0),
            (271, 271, 0),
            (30, 30, 0),
            (50, 50, 0),
            (120, 120, 0),
            (7537, 7537, 0),
        ],
    ),
    // Item 4205, the built start: units (3158, 3250, 2530) -> (3158, 3250, 2498); builds (2080,
    // 2080, 47) -> (2080, 2080, 34); world (7553, 7537, 4393) -> (7553, 7553, 10).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 34) -> (2080, 2080, 33); items (50, 110,
    // 77) -> (50, 50, 0); world (7553, 7553, 10) -> (7553, 7553, 0).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2080, 2080, 33)
    // -> (2080, 2080, 5).
    // Item 4226, a bird walked on neither line: units (3158, 3250, 2498) -> (3158, 2945, 2416), guys (861, 966, 105) -> (861, 861, 0).
    // Item 4228, a path stack's `size` its high water: units (3158, 2945, 2416) -> (3158, 2945, 2409).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3158, 2945, 2409) -> (3158, 2954, 1814).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3158, 2954, 1814) -> (3158, 2954, 1812).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3158, 2954, 1812) -> (3158, 2954, 1810).
    // Item 4244, a mine's `mtn`: builds (2080, 2080, 5) -> (2080, 2080, 3).
    // Item 4257, the garrison chain's order: units (3158, 2954, 1810) -> (3158, 2954, 1786); builds (2080, 2080, 3) -> (2080, 2080, 1).
    // Item 4266, a guard order's walker: units (3158, 2954, 1786) -> (3158, 3098, 1723).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1011d",
        13277,
        [
            (3158, 3158, 2),
            (2080, 2080, 0),
            (0, 0, 0),
            (12, 12, 0),
            (4, 4, 0),
            (538, 538, 0),
            (861, 861, 0),
            (271, 271, 0),
            (30, 30, 0),
            (50, 50, 0),
            (120, 120, 0),
            (7553, 7553, 0),
        ],
    ),
    // Item 4205, the built start: units (3117, 3189, 2465) -> (3117, 3189, 2437); builds (2080,
    // 2080, 47) -> (2080, 2080, 34); world (7517, 7501, 4316) -> (7517, 7517, 11).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 34) -> (2080, 2080, 33); items (50, 110,
    // 77) -> (50, 50, 0); world (7517, 7517, 11) -> (7517, 7517, 0).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2080, 2080, 33)
    // -> (2080, 2080, 5).
    // Item 4226, a bird walked on neither line: units (3117, 3189, 2437) -> (3117, 2889, 2395), guys (854, 959, 105) -> (854, 854, 0).
    // Item 4228, a path stack's `size` its high water: units (3117, 2889, 2395) -> (3117, 2889, 2389).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3117, 2889, 2389) -> (3117, 2898, 1789).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3117, 2898, 1789) -> (3117, 2898, 1787).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3117, 2898, 1787) -> (3117, 2898, 1784).
    // Item 4244, a mine's `mtn`: builds (2080, 2080, 5) -> (2080, 2080, 3).
    // Item 4257, the garrison chain's order: units (3117, 2898, 1784) -> (3117, 2898, 1760); builds (2080, 2080, 3) -> (2080, 2080, 1).
    // Item 4266, a guard order's walker: units (3117, 2898, 1760) -> (3117, 3042, 1752).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4320, the score's dirty bits (`0x800000`, `0x1000000`) emitted, leader 0's 0x2800007 agreeing: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1011e",
        13296,
        [
            (3117, 3117, 2),
            (2080, 2080, 0),
            (0, 0, 0),
            (9, 9, 2),
            (6, 6, 0),
            (538, 538, 0),
            (854, 854, 0),
            (271, 271, 0),
            (30, 30, 0),
            (50, 50, 0),
            (120, 120, 0),
            (7517, 7517, 0),
        ],
    ),
    // Item 4205, the built start: units (3139, 3183, 2492) -> (3139, 3183, 2464); builds (2080,
    // 2080, 46) -> (2080, 2080, 33); world (7419, 7403, 4096) -> (7419, 7419, 11).
    // Item 4209, the box's `clear_down`: builds (2080, 2080, 33) -> (2080, 2080, 32); items (50, 110,
    // 77) -> (50, 50, 0); world (7419, 7419, 11) -> (7419, 7419, 0).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2080, 2080, 32)
    // -> (2080, 2080, 6).
    // Item 4226, a bird walked on neither line: units (3139, 3183, 2464) -> (3139, 2878, 2441), guys (847, 952, 105) -> (847, 847, 0).
    // Item 4228, a path stack's `size` its high water: units (3139, 2878, 2441) -> (3139, 2878, 2435).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3139, 2878, 2435) -> (3139, 2887, 1877).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3139, 2887, 1877) -> (3139, 2887, 1875).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3139, 2887, 1875) -> (3139, 2887, 1872).
    // Item 4244, a mine's `mtn`: builds (2080, 2080, 6) -> (2080, 2080, 4).
    // Item 4257, the garrison chain's order: units (3139, 2887, 1872) -> (3139, 2887, 1848); builds (2080, 2080, 4) -> (2080, 2080, 2).
    // Item 4266, a guard order's walker: units (3139, 2887, 1848) -> (3139, 3028, 1823).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1015b",
        13392,
        [
            (3139, 3139, 7),
            (2080, 2080, 0),
            (0, 0, 0),
            (27, 27, 0),
            (4, 4, 0),
            (538, 538, 0),
            (847, 847, 0),
            (271, 271, 0),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2371, 2371, 52)
    // -> (2371, 2371, 9).
    // Item 4226, a bird walked on neither line: units (4193, 4310, 3246) -> (4193, 4010, 3162), guys (1129, 1234, 106) -> (1129, 1129, 1).
    // Item 4228, a path stack's `size` its high water: units (4193, 4010, 3162) -> (4193, 4010, 3143).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4193, 4010, 3143) -> (4193, 4019, 3082).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4193, 4019, 3082) -> (4193, 4022, 3121).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4193, 4022, 3121) -> (4193, 4022, 3120).
    // Item 4244, a mine's `mtn`: builds (2371, 2371, 9) -> (2371, 2371, 7).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 2) -> (550, 550, 1).
    // Item 4257, the garrison chain's order: builds (2371, 2371, 7) -> (2371, 2371, 4).
    // Item 4266, a guard order's walker: units (4193, 4022, 3120) -> (4193, 4187, 2787).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1155c",
        3003,
        [
            (4193, 4193, 5),
            (2371, 2371, 4),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1129, 1129, 1),
            (271, 271, 0),
            (27, 27, 0),
            (75, 75, 0),
            (120, 120, 0),
            (7583, 7583, 2),
        ],
    ),
    // Item 4208, the whales' piece: units (2760, 2965, 535) -> (2760, 2965, 213); builds (520, 520,
    // 13) -> (520, 520, 0); guys (924, 994, 174) -> (924, 994, 70); world (8850, 7263, 4860) -> (8850,
    // 8850, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2760, 2965, 213) -> (2760, 2965, 205).
    // Item 4226, a bird walked on neither line: units (2760, 2965, 205) -> (2760, 2760, 0), guys (924, 994, 70) -> (924, 924, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (263, 263, 4) -> (263, 263, 2).
    (
        "run1161",
        4,
        [
            (2760, 2760, 0),
            (520, 520, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (526, 526, 0),
            (924, 924, 0),
            (263, 263, 2),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2140, 2136, 811)
    // -> (2140, 2136, 782).
    // Item 4226, a bird walked on neither line: units (6053, 5910, 5265) -> (6053, 5443, 5319), guys (1321, 1475, 158) -> (1321, 1321, 4).
    // Item 4228, a path stack's `size` its high water: units (6053, 5443, 5319) -> (6053, 5443, 5305).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 2) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6053, 5443, 5305) -> (6053, 5452, 5251).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6053, 5452, 5251) -> (6053, 5629, 5302).
    // Item 4244, a mine's `mtn`: builds (2140, 2136, 782) -> (2140, 2136, 781).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (6053, 5629, 5302) -> (6053, 5629, 5301); guys (1321, 1321, 4) -> (1321, 1321, 3).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 2) -> (550, 550, 1).
    // Item 4266, a guard order's walker: units (6053, 5629, 5301) -> (6053, 5923, 4240).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 6) -> (271, 271, 5).
    (
        "run1164",
        4528,
        [
            (6053, 5923, 4240),
            (2140, 2136, 780),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1321, 1321, 3),
            (271, 271, 5),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2267, 2266, 431)
    // -> (2267, 2266, 384).
    // Item 4226, a bird walked on neither line: units (6004, 6418, 5248) -> (6004, 5953, 4838), guys (1890, 2051, 163) -> (1890, 1890, 2).
    // Item 4228, a path stack's `size` its high water: units (6004, 5953, 4838) -> (6004, 5953, 4824).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6004, 5953, 4824) -> (6004, 5962, 4707).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6004, 5962, 4707) -> (6004, 5962, 4706).
    // Item 4159's third residue: guys (1890, 1890, 2) -> (1890, 1890, 1), a newborn ship's turret at its rest (the Battleship and Cruisers at `[0, 0, 180°]`).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6004, 5962, 4706) -> (6004, 5962, 4704).
    // Item 4244, a mine's `mtn`: builds (2267, 2266, 384) -> (2267, 2266, 383).
    // Item 4257, the garrison chain's order: builds (2267, 2266, 383) -> (2267, 2266, 382).
    // Item 4262, a wonder's entry in its wonder list: builds (2267, 2266, 382) -> (2267, 2266, 381).
    // Item 4266, a guard order's walker: units (6004, 5962, 4704) -> (6004, 6001, 4367).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1165",
        3719,
        [
            (6004, 6004, 22),
            (2267, 2266, 381),
            (0, 0, 0),
            (287, 9, 286),
            (0, 0, 0),
            (676, 676, 1),
            (1890, 1890, 1),
            (271, 271, 2),
            (30, 30, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9290, 9290, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (6004, 6310, 5126) -> (6004, 6310, 5070); builds (2247,
    // 2246, 508) -> (2247, 2246, 498); groups (688, 688, 3) -> (688, 688, 1); guys (1935, 2061, 229)
    // -> (1935, 2061, 129); world (9262, 7675, 5955) -> (9262, 9262, 17).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9262, 9262, 17) ->
    // (9262, 9262, 13).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2247, 2246, 498)
    // -> (2247, 2246, 442).
    // Item 4226, a bird walked on neither line: units (6004, 6310, 5070) -> (6004, 5950, 4771), guys (1935, 2061, 129) -> (1935, 1935, 3).
    // Item 4228, a path stack's `size` its high water: units (6004, 5950, 4771) -> (6004, 5950, 4756).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6004, 5950, 4756) -> (6004, 5959, 4748).
    // Item 4234, a cast's three ranges walked: units (6004, 5959, 4748) -> (6004, 5962, 4744).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6004, 5962, 4744) -> (6004, 5962, 4743).
    // Item 4159's third residue: guys (1935, 1935, 3) -> (1935, 1935, 2), a newborn ship's turret at its rest.
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6004, 5962, 4743) -> (6004, 5962, 4742).
    // Item 4244, a mine's `mtn`: builds (2247, 2246, 442) -> (2247, 2246, 441).
    // Item 4257, the garrison chain's order: builds (2247, 2246, 441) -> (2247, 2246, 440).
    // Item 4262, a wonder's entry in its wonder list: builds (2247, 2246, 440) -> (2247, 2246, 439).
    // Item 4266, a guard order's walker: units (6004, 5962, 4742) -> (6004, 6001, 4403).
    // Item 4159's twelfth residue, the CEO's tile step dirties the economy: leaders (271, 271, 7) -> (271, 271, 4).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 4) -> (271, 271, 3).
    (
        "run1166",
        3920,
        [
            (6004, 6004, 24),
            (2247, 2246, 438),
            (0, 0, 0),
            (0, 0, 0),
            (10, 10, 0),
            (688, 688, 1),
            (1935, 1935, 2),
            (271, 271, 3),
            (30, 30, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9262, 9262, 13),
        ],
    ),
    // Item 4208, the whales' piece: units (7749, 8143, 6789) -> (7749, 8143, 6726); builds (1813,
    // 1813, 87) -> (1813, 1813, 73); groups (628, 628, 7) -> (628, 628, 5); guys (1934, 2095, 277) ->
    // (1934, 2095, 177); world (9438, 7851, 6516) -> (9438, 9438, 4).
    // Item 4209, the box's `clear_down`: items (20, 35, 22) -> (20, 20, 0); world (9438, 9438, 4) ->
    // (9438, 9438, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (7749, 8143, 6726) -> (7749, 8143, 6718).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1813, 1813, 73)
    // -> (1813, 1813, 14).
    // Item 4226, a bird walked on neither line: units (7749, 8143, 6718) -> (7749, 7683, 6335), guys (1934, 2095, 177) -> (1934, 1934, 16).
    // Item 4228, a path stack's `size` its high water: units (7749, 7683, 6335) -> (7749, 7683, 6302).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 3) -> (27, 27, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (7749, 7683, 6302) -> (7749, 7692, 6191).
    // Item 4234, a cast's three ranges walked: units (7749, 7692, 6191) -> (7749, 7737, 5516).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (7749, 7737, 5516) -> (7749, 7737, 5515).
    // Item 4159's third residue: guys (1934, 1934, 16) -> (1934, 1934, 11), a newborn ship's turret at its rest; units (7749, 7737, 5515) -> (7749, 7737, 5513), measured on the tree merged with item 4234's.
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (7749, 7737, 5513) -> (7749, 7737, 5512).
    // Item 4244, a mine's `mtn`: builds (1813, 1813, 14) -> (1813, 1813, 13).
    // Item 4266, a guard order's walker: units (7749, 7737, 5512) -> (7749, 7740, 5515).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1169",
        3769,
        [
            (7749, 7749, 100),
            (1813, 1813, 13),
            (0, 0, 0),
            (12, 12, 0),
            (0, 0, 0),
            (628, 628, 5),
            (1934, 1934, 11),
            (271, 271, 1),
            (27, 27, 0),
            (20, 20, 0),
            (330, 330, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1911, 1911, 59)
    // -> (1911, 1911, 7).
    // Item 4226, a bird walked on neither line: units (6735, 7099, 5978) -> (6735, 6639, 5619), guys (1962, 2123, 168) -> (1962, 1962, 7).
    // Item 4228, a path stack's `size` its high water: units (6735, 6639, 5619) -> (6735, 6639, 5596).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6735, 6639, 5596) -> (6735, 6648, 5629).
    // Item 4234, a cast's three ranges walked: units (6735, 6648, 5629) -> (6735, 6651, 5597).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6735, 6651, 5597) -> (6735, 6657, 5587).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6735, 6657, 5587) -> (6735, 6657, 5586).
    // Item 4244, a mine's `mtn`: builds (1911, 1911, 7) -> (1911, 1911, 5).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (640, 640, 5) -> (640, 640, 4).
    // Item 4262, a wonder's entry in its wonder list: builds (1911, 1911, 5) -> (1911, 1911, 3).
    // Item 4266, a guard order's walker: units (6735, 6657, 5586) -> (6735, 6732, 3976).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1185",
        4582,
        [
            (6735, 6735, 67),
            (1911, 1911, 1),
            (0, 0, 0),
            (3, 3, 0),
            (4, 4, 0),
            (640, 640, 4),
            (1962, 1962, 7),
            (271, 271, 1),
            (27, 27, 0),
            (5, 5, 0),
            (330, 330, 0),
            (9268, 9268, 0),
        ],
    ),
    // Item 4205, the built start: units (1060, 1345, 381) -> (1060, 1345, 303); builds (1164,
    // 1164, 23) -> (1164, 1164, 9); world (7361, 7345, 3930) -> (7361, 7361, 1).
    // Item 4209, the box's `clear_down`: items (105, 110, 25) -> (105, 105, 0); world (7361, 7361, 1)
    // -> (7361, 7361, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1060, 1345, 303) -> (1060, 1345, 295).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1164, 1164, 9) ->
    // (1164, 1164, 1).
    // Item 4226, a bird walked on neither line: units (1060, 1345, 295) -> (1060, 1060, 10), guys (309, 407, 98) -> (309, 309, 0).
    // Item 4228, a path stack's `size` its high water: units (1060, 1060, 10) -> (1060, 1060, 0).
    // Item 4244, a mine's `mtn`: builds (1164, 1164, 1) -> (1164, 1164, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1186b",
        1377,
        [
            (1060, 1060, 0),
            (1164, 1164, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (309, 309, 0),
            (271, 271, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (3412, 3412, 124)
    // -> (3412, 3412, 24).
    // Item 4226, a bird walked on neither line: units (10607, 10674, 9057) -> (10607, 10124, 9079), guys (3247, 3436, 199) -> (3247, 3247, 10).
    // Item 4228, a path stack's `size` its high water: units (10607, 10124, 9079) -> (10607, 10124, 9057).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (56, 56, 5) -> (56, 56, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (10607, 10124, 9057) -> (10607, 10145, 9335).
    // Item 4234, a cast's three ranges walked: units (10607, 10145, 9335) -> (10607, 10154, 9295).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (10607, 10154, 9295) -> (10607, 10166, 9259).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (10607, 10166, 9259) -> (10607, 10166, 9258).
    // Item 4244, a mine's `mtn`: builds (3412, 3412, 24) -> (3412, 3412, 16).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (700, 700, 9) -> (700, 700, 8).
    // Item 4262, a wonder's entry in its wonder list: builds (3412, 3412, 16) -> (3412, 3412, 13).
    // Item 4266, a guard order's walker: units (10607, 10166, 9258) -> (10607, 10550, 8527).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1217",
        7728,
        [
            (10607, 10607, 67),
            (3412, 3412, 12),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (700, 700, 8),
            (3247, 3247, 10),
            (271, 271, 1),
            (56, 56, 0),
            (0, 0, 0),
            (330, 330, 0),
            (9486, 9486, 1),
        ],
    ),
    // Item 4205, the built start: units (4091, 4338, 3359) -> (4091, 4338, 3296); builds (1817,
    // 1817, 43) -> (1817, 1817, 28); cities (27, 27, 2) -> (27, 27, 1); world (7663, 7647,
    // 4673) -> (7663, 7663, 4).
    // Item 4209, the box's `clear_down`: items (90, 110, 44) -> (90, 90, 0); world (7663, 7663, 4) ->
    // (7663, 7663, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4091, 4338, 3296) -> (4091, 4338, 3291).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1817, 1817, 28)
    // -> (1817, 1817, 2).
    // Item 4226, a bird walked on neither line: units (4091, 4338, 3291) -> (4091, 3833, 3064), guys (1099, 1274, 175) -> (1099, 1099, 0).
    // Item 4228, a path stack's `size` its high water: units (4091, 3833, 3064) -> (4091, 3833, 3048).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4091, 3833, 3048) -> (4091, 3842, 3104).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4091, 3842, 3104) -> (4091, 3851, 3100).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4091, 3851, 3100) -> (4091, 3851, 3098).
    // Item 4244, a mine's `mtn`: builds (1817, 1817, 2) -> (1817, 1817, 1).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 2) -> (550, 550, 1).
    // Item 4266, a guard order's walker: units (4091, 3851, 3098) -> (4091, 4064, 3095).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1220b",
        2976,
        [
            (4091, 4091, 1),
            (1817, 1817, 1),
            (0, 0, 0),
            (9, 9, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1099, 1099, 0),
            (271, 271, 0),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1719, 1719, 30)
    // -> (1719, 1719, 6).
    // Item 4226, a bird walked on neither line: units (6445, 6949, 5873) -> (6445, 6304, 5383), guys (1768, 1992, 239) -> (1768, 1768, 15).
    // Item 4228, a path stack's `size` its high water: units (6445, 6304, 5383) -> (6445, 6304, 5370).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (33, 33, 3) -> (33, 33, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6445, 6304, 5370) -> (6445, 6313, 5367).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6445, 6313, 5367) -> (6445, 6349, 5430).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6445, 6349, 5430) -> (6445, 6349, 5428).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1221",
        4287,
        [
            (6445, 6445, 69),
            (1719, 1719, 6),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (592, 592, 4),
            (1768, 1768, 15),
            (271, 271, 1),
            (33, 33, 0),
            (10, 10, 0),
            (330, 330, 0),
            (9162, 9162, 1),
        ],
    ),
    // Item 4205, the built start: units (11740, 11409, 10463) -> (11740, 11409, 10402); builds
    // (4271, 4271, 126) -> (4271, 4271, 112); cities (62, 62, 7) -> (62, 62, 3); world (7873,
    // 7857, 4963) -> (7873, 7873, 17).
    // Item 4209, the box's `clear_down`: items (30, 110, 91) -> (30, 30, 0); world (7873, 7873, 17) ->
    // (7873, 7873, 1).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (4271, 4271, 112)
    // -> (4271, 4271, 30).
    // Item 4226, a bird walked on neither line: units (11740, 11409, 10402) -> (11740, 10894, 10415), guys (2735, 2910, 182) -> (2735, 2735, 7).
    // Item 4228, a path stack's `size` its high water: units (11740, 10894, 10415) -> (11740, 10894, 10396).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (62, 62, 3) -> (62, 62, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (11740, 10894, 10396) -> (11740, 10915, 10395).
    // Item 4234, a cast's three ranges walked: units (11740, 10915, 10395) -> (11740, 10927, 10351).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (11740, 10927, 10351) -> (11740, 10933, 10441).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (11740, 10933, 10441) -> (11740, 10933, 10438).
    // Item 4244, a mine's `mtn`: builds (4271, 4271, 30) -> (4271, 4271, 25).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (2735, 2735, 7) -> (2735, 2735, 0).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 1) -> (556, 556, 0).
    // Item 4257, the garrison chain's order: builds (4271, 4271, 25) -> (4271, 4271, 24).
    // Item 4262, a wonder's entry in its wonder list: builds (4271, 4271, 24) -> (4271, 4271, 21).
    // Item 4266, a guard order's walker: units (11740, 10933, 10438) -> (11740, 11677, 10203).
    // Item 4159r14, the Mongol stable arm of `get_cost`: builds (4271, 4271, 21) -> (4271, 4271, 13),
    // leaders (271, 271, 6) -> (271, 271, 4) (`encrypt.bucket[0]`/`[1]` agree).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1224b",
        9745,
        [
            (11740, 11740, 3),
            (4271, 4271, 13),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (556, 556, 0),
            (2735, 2735, 0),
            (271, 271, 1),
            (62, 62, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1733, 1733, 45)
    // -> (1733, 1733, 3).
    // Item 4226, a bird walked on neither line: units (7490, 7627, 6479) -> (7490, 7367, 6367), guys (2097, 2188, 97) -> (2097, 2097, 6).
    // Item 4228, a path stack's `size` its high water: units (7490, 7367, 6367) -> (7490, 7367, 6354).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 2) -> (30, 30, 1).
    // Item 4230, a trade order walked and a road stack's doubled size: units (7490, 7367, 6354) -> (7490, 7376, 6438).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (7490, 7376, 6438) -> (7490, 7376, 6434).
    // Item 4244, a mine's `mtn`: builds (1733, 1733, 3) -> (1733, 1733, 1).
    // Item 4262, a wonder's entry in its wonder list: builds (1733, 1733, 1) -> (1733, 1733, 0).
    // Item 4266, a guard order's walker: units (7490, 7376, 6434) -> (7490, 7412, 6594).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1235b",
        4127,
        [
            (7490, 7490, 63),
            (1733, 1733, 0),
            (0, 0, 0),
            (12, 12, 0),
            (4, 4, 0),
            (646, 646, 7),
            (2097, 2097, 6),
            (271, 271, 1),
            (30, 30, 1),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1762, 1762, 44)
    // -> (1762, 1762, 5).
    // Item 4226, a bird walked on neither line: units (6610, 7349, 5753) -> (6610, 6595, 5014), guys (2070, 2329, 292) -> (2070, 2070, 33).
    // Item 4228, a path stack's `size` its high water: units (6610, 6595, 5014) -> (6610, 6595, 4954).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (44, 44, 1) -> (44, 44, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6610, 6595, 4954) -> (6610, 6607, 3799).
    // Item 4234, a cast's three ranges walked: units (6610, 6607, 3799) -> (6610, 6610, 152).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6610, 6610, 152) -> (6610, 6610, 149).
    // Item 4238, the age's seating writes each figure's `last`: units (6610, 6610, 149) -> (6610, 6610, 116); guys (2070, 2070, 33) -> (2070, 2070, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1236",
        4179,
        [
            (6610, 6610, 116),
            (1762, 1762, 5),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (658, 658, 4),
            (2070, 2070, 0),
            (271, 271, 1),
            (44, 44, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1650, 1650, 18)
    // -> (1650, 1650, 1).
    // Item 4226, a bird walked on neither line: units (3448, 3542, 2644) -> (3448, 3262, 2574), guys (892, 990, 98) -> (892, 892, 0).
    // Item 4228, a path stack's `size` its high water: units (3448, 3262, 2574) -> (3448, 3262, 2558).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3448, 3262, 2558) -> (3448, 3271, 2559).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (3448, 3271, 2559) -> (3448, 3274, 2560).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3448, 3274, 2560) -> (3448, 3274, 2558).
    // Item 4244, a mine's `mtn`: builds (1650, 1650, 1) -> (1650, 1650, 0).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (538, 538, 1) -> (538, 538, 0).
    // Item 4266, a guard order's walker: units (3448, 3274, 2558) -> (3448, 3436, 2641).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1237",
        2674,
        [
            (3448, 3448, 3),
            (1650, 1650, 0),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (538, 538, 0),
            (892, 892, 0),
            (271, 271, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2494, 2494, 43)
    // -> (2494, 2494, 8).
    // Item 4226, a bird walked on neither line: units (6417, 7136, 5516) -> (6417, 6387, 4797), guys (2151, 2410, 266) -> (2151, 2151, 7).
    // Item 4228, a path stack's `size` its high water: units (6417, 6387, 4797) -> (6417, 6387, 4737).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (49, 49, 1) -> (49, 49, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6417, 6387, 4737) -> (6417, 6402, 4082).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6417, 6402, 4082) -> (6417, 6402, 4079).
    // Item 4238, the age's seating writes each figure's `last`: units (6417, 6402, 4079) -> (6417, 6402, 4073); guys (2151, 2151, 7) -> (2151, 2151, 1).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6417, 6402, 4073) -> (6417, 6402, 4070).
    // Item 4244, a mine's `mtn`: builds (2494, 2494, 8) -> (2494, 2494, 6).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1238c",
        5021,
        [
            (6417, 6417, 37),
            (2494, 2494, 6),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (646, 646, 4),
            (2151, 2151, 1),
            (271, 271, 1),
            (49, 49, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1692, 1692, 27)
    // -> (1692, 1692, 0).
    // Item 4226, a bird walked on neither line: units (5942, 6521, 5183) -> (5942, 5876, 4615), guys (1748, 1972, 225) -> (1748, 1748, 1).
    // Item 4228, a path stack's `size` its high water: units (5942, 5876, 4615) -> (5942, 5876, 4589).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (38, 38, 3) -> (38, 38, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5942, 5876, 4589) -> (5942, 5888, 4481).
    // Item 4234, a cast's three ranges walked: units (5942, 5888, 4481) -> (5942, 5891, 4670).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5942, 5891, 4670) -> (5942, 5897, 4766).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5942, 5897, 4766) -> (5942, 5897, 4764).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (622, 622, 6) -> (622, 622, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1247",
        4306,
        [
            (5942, 5942, 8),
            (1692, 1692, 0),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (622, 622, 5),
            (1748, 1748, 1),
            (271, 271, 1),
            (38, 38, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1674, 1674, 37)
    // -> (1674, 1674, 2).
    // Item 4226, a bird walked on neither line: units (4388, 4364, 3384) -> (4388, 4064, 3405), guys (1134, 1239, 107) -> (1134, 1134, 2).
    // Item 4228, a path stack's `size` its high water: units (4388, 4064, 3405) -> (4388, 4064, 3385).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4388, 4064, 3385) -> (4388, 4073, 3409).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4388, 4073, 3409) -> (4388, 4085, 3386).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4388, 4085, 3386) -> (4388, 4085, 3383).
    // Item 4244, a mine's `mtn`: builds (1674, 1674, 2) -> (1674, 1674, 1).
    // Item 4260, a trackless crew's `last_angle` after guy 0's turn: guys (1134, 1134, 2) -> (1134, 1134, 0).
    // Item 4266, a guard order's walker: units (4388, 4085, 3383) -> (4388, 4340, 3257).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1254c",
        3115,
        [
            (4388, 4388, 8),
            (1674, 1674, 1),
            (0, 0, 0),
            (9, 9, 0),
            (0, 0, 0),
            (550, 550, 0),
            (1134, 1134, 0),
            (271, 271, 0),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (3789, 3789, 90)
    // -> (3789, 3789, 23).
    // Item 4226, a bird walked on neither line: units (11229, 10998, 9830) -> (11229, 10383, 9863), guys (2913, 3123, 215) -> (2913, 2913, 5).
    // Item 4228, a path stack's `size` its high water: units (11229, 10383, 9863) -> (11229, 10383, 9845).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (54, 54, 5) -> (54, 54, 0), goods (120, 120, 1) -> (120, 120, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (11229, 10383, 9845) -> (11229, 10401, 9771).
    // Item 4234, a cast's three ranges walked: units (11229, 10401, 9771) -> (11229, 10407, 9827).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (11229, 10407, 9827) -> (11229, 10431, 9895).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (11229, 10431, 9895) -> (11229, 10431, 9894).
    // Item 4244, a mine's `mtn`: builds (3789, 3789, 23) -> (3789, 3789, 20).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (2913, 2913, 5) -> (2913, 2913, 0).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 1) -> (556, 556, 0).
    // Item 4257, the garrison chain's order: builds (3789, 3789, 20) -> (3789, 3789, 19).
    // Item 4262, a wonder's entry in its wonder list: builds (3789, 3789, 19) -> (3789, 3789, 16).
    // Item 4266, a guard order's walker: units (11229, 10431, 9894) -> (11229, 11202, 6921).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1263",
        8362,
        [
            (11229, 11229, 1),
            (3789, 3789, 16),
            (0, 0, 0),
            (9, 9, 0),
            (0, 0, 0),
            (556, 556, 0),
            (2913, 2913, 0),
            (271, 271, 1),
            (54, 54, 0),
            (50, 50, 0),
            (120, 120, 0),
            (7867, 7867, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (6468, 7014, 5960) -> (6468, 7014, 5902); builds (1770,
    // 1770, 57) -> (1770, 1770, 43); groups (586, 586, 8) -> (586, 586, 7); guys (1895, 2119, 344) ->
    // (1895, 2119, 244); world (9330, 7743, 6143) -> (9330, 9330, 5).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9330, 9330, 5) ->
    // (9330, 9330, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (6468, 7014, 5902) -> (6468, 7014, 5901).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1770, 1770, 43)
    // -> (1770, 1770, 4).
    // Item 4226, a bird walked on neither line: units (6468, 7014, 5901) -> (6468, 6369, 5365), guys (1895, 2119, 244) -> (1895, 1895, 20).
    // Item 4228, a path stack's `size` its high water: units (6468, 6369, 5365) -> (6468, 6369, 5344).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (43, 43, 4) -> (43, 43, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6468, 6369, 5344) -> (6468, 6384, 5412).
    // Item 4234, a cast's three ranges walked: units (6468, 6384, 5412) -> (6468, 6387, 5338).
    // Item 4238, the age's seating writes each figure's `last`: guys (1895, 1895, 20) -> (1895, 1895, 12).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6468, 6387, 5338) -> (6468, 6387, 5331).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1264b",
        4919,
        [
            (6468, 6468, 108),
            (1770, 1770, 4),
            (0, 0, 0),
            (3, 3, 0),
            (2, 2, 0),
            (586, 586, 7),
            (1895, 1895, 12),
            (271, 271, 1),
            (43, 43, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9330, 9330, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (11161, 11622, 10287) -> (11161, 11622, 10220); builds
    // (3796, 3796, 114) -> (3796, 3796, 100); groups (682, 682, 11) -> (682, 682, 10); guys (3210,
    // 3413, 324) -> (3210, 3413, 224); world (9704, 8117, 6778) -> (9704, 9704, 6).
    // Item 4209, the box's `clear_down`: items (5, 35, 31) -> (5, 5, 0); world (9704, 9704, 6) ->
    // (9704, 9704, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (11161, 11622, 10220) -> (11161, 11622, 10213).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (3796, 3796, 100)
    // -> (3796, 3796, 26).
    // Item 4226, a bird walked on neither line: units (11161, 11622, 10213) -> (11161, 10999, 9763), guys (3210, 3413, 224) -> (3210, 3210, 21).
    // Item 4228, a path stack's `size` its high water: units (11161, 10999, 9763) -> (11161, 10999, 9745).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (54, 54, 5) -> (54, 54, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (11161, 10999, 9745) -> (11161, 11017, 7744).
    // Item 4234, a cast's three ranges walked: units (11161, 11017, 7744) -> (11161, 11026, 7710).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (11161, 11026, 7710) -> (11161, 11026, 7705).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (11161, 11026, 7705) -> (11161, 11053, 7581).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (11161, 11053, 7581) -> (11161, 11053, 7561).
    // Item 4244, a mine's `mtn`: builds (3796, 3796, 26) -> (3796, 3796, 17).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (3210, 3210, 21) -> (3210, 3210, 9).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (682, 682, 10) -> (682, 682, 8).
    // Item 4257, the garrison chain's order: builds (3796, 3796, 17) -> (3796, 3796, 14).
    // Item 4262, a wonder's entry in its wonder list: builds (3796, 3796, 14) -> (3796, 3796, 11).
    // Item 4266, a guard order's walker: units (11161, 11053, 7561) -> (11161, 11131, 7339).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1266c",
        8700,
        [
            (11161, 11161, 78),
            (3796, 3796, 11),
            (0, 0, 0),
            (6, 6, 0),
            (0, 0, 0),
            (682, 682, 8),
            (3210, 3210, 9),
            (271, 271, 1),
            (54, 54, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1908, 1908, 50)
    // -> (1908, 1908, 6).
    // Item 4226, a bird walked on neither line: units (4744, 4798, 3699) -> (4744, 4393, 3674), guys (1246, 1386, 141) -> (1246, 1246, 1).
    // Item 4228, a path stack's `size` its high water: units (4744, 4393, 3674) -> (4744, 4393, 3651).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 1) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4744, 4393, 3651) -> (4744, 4402, 3711).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4744, 4402, 3711) -> (4744, 4420, 3685).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4744, 4420, 3685) -> (4744, 4420, 3684).
    // Item 4244, a mine's `mtn`: builds (1908, 1908, 6) -> (1908, 1908, 5).
    // Item 4262, a wonder's entry in its wonder list: builds (1908, 1908, 5) -> (1908, 1908, 3).
    // Item 4266, a guard order's walker: units (4744, 4420, 3684) -> (4744, 4693, 3664).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 0).
    (
        "run1268",
        3347,
        [
            (4744, 4744, 8),
            (1908, 1908, 1),
            (0, 0, 0),
            (24, 24, 0),
            (4, 4, 0),
            (544, 544, 0),
            (1246, 1246, 1),
            (271, 271, 0),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1938, 1938, 50)
    // -> (1938, 1938, 6).
    // Item 4226, a bird walked on neither line: units (9512, 9862, 8555) -> (9512, 9422, 8205), guys (2191, 2345, 165) -> (2191, 2191, 11).
    // Item 4228, a path stack's `size` its high water: units (9512, 9422, 8205) -> (9512, 9422, 8166).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (25, 25, 1) -> (25, 25, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (9512, 9422, 8166) -> (9512, 9428, 7294).
    // Item 4234, a cast's three ranges walked: units (9512, 9428, 7294) -> (9512, 9503, 7181).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (9512, 9503, 7181) -> (9512, 9503, 7180).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (9512, 9503, 7180) -> (9512, 9512, 120).
    // Item 4244, a mine's `mtn`: builds (1938, 1938, 6) -> (1938, 1938, 4).
    // Item 4262, a wonder's entry in its wonder list: builds (1938, 1938, 4) -> (1938, 1938, 2).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1293",
        3606,
        [
            (9512, 9512, 120),
            (1938, 1938, 2),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (586, 586, 4),
            (2191, 2191, 11),
            (271, 271, 1),
            (25, 25, 0),
            (20, 20, 0),
            (330, 330, 0),
            (9230, 9230, 0),
        ],
    ),
    // Item 4208, the whales' piece: units (8216, 8852, 7275) -> (8216, 8852, 7208); builds (2123,
    // 2123, 74) -> (2123, 2123, 59); groups (688, 688, 12) -> (688, 688, 11); guys (2242, 2473, 338)
    // -> (2242, 2473, 238); world (9490, 7903, 6488) -> (9490, 9490, 5).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9490, 9490, 5) ->
    // (9490, 9490, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (8216, 8852, 7208) -> (8216, 8852, 7202).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2123, 2123, 59)
    // -> (2123, 2123, 18).
    // Item 4226, a bird walked on neither line: units (8216, 8852, 7202) -> (8216, 8162, 6573), guys (2242, 2473, 238) -> (2242, 2242, 7).
    // Item 4228, a path stack's `size` its high water: units (8216, 8162, 6573) -> (8216, 8162, 6548).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 3) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (8216, 8162, 6548) -> (8216, 8171, 7049).
    // Item 4234, a cast's three ranges walked: units (8216, 8171, 7049) -> (8216, 8183, 6974).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (8216, 8183, 6974) -> (8216, 8189, 7046).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (8216, 8189, 7046) -> (8216, 8189, 7044).
    // Item 4244, a mine's `mtn`: builds (2123, 2123, 18) -> (2123, 2123, 16).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (688, 688, 11) -> (688, 688, 10).
    // Item 4258, `Group::add`'s OR of its new member's role: groups (688, 688, 10) -> (688, 688, 9).
    // Item 4262, a wonder's entry in its wonder list: builds (2123, 2123, 16) -> (2123, 2123, 14).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2123, 2123, 14) -> (2123, 2123, 4), leaders (271, 271, 8) -> (271, 271, 4).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1294",
        4473,
        [
            (8216, 8216, 68),
            (2123, 2123, 4),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (688, 688, 9),
            (2242, 2242, 7),
            (271, 271, 1),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2163, 2163, 64)
    // -> (2163, 2163, 22).
    // Item 4226, a bird walked on neither line: units (9905, 10501, 9322) -> (9905, 9836, 8734), guys (2172, 2403, 240) -> (2172, 2172, 9).
    // Item 4228, a path stack's `size` its high water: units (9905, 9836, 8734) -> (9905, 9836, 8715).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 3) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (9905, 9836, 8715) -> (9905, 9845, 7275).
    // Item 4234, a cast's three ranges walked: units (9905, 9845, 7275) -> (9905, 9854, 8666).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (9905, 9854, 8666) -> (9905, 9866, 8616).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (9905, 9866, 8616) -> (9905, 9866, 8613).
    // Item 4244, a mine's `mtn`: builds (2163, 2163, 22) -> (2163, 2163, 20).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (682, 682, 10) -> (682, 682, 9).
    // Item 4262, a wonder's entry in its wonder list: builds (2163, 2163, 20) -> (2163, 2163, 17).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2163, 2163, 17) -> (2163, 2163, 7), leaders (271, 271, 8) -> (271, 271, 4).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1296",
        4632,
        [
            (9905, 9905, 58),
            (2163, 2163, 6),
            (0, 0, 0),
            (0, 0, 0),
            (6, 6, 0),
            (682, 682, 9),
            (2172, 2172, 9),
            (271, 271, 1),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2029, 2029, 64)
    // -> (2029, 2029, 7).
    // Item 4226, a bird walked on neither line: units (5488, 5902, 4636) -> (5488, 5422, 4233), guys (1616, 1784, 169) -> (1616, 1616, 1).
    // Item 4228, a path stack's `size` its high water: units (5488, 5422, 4233) -> (5488, 5422, 4208).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (23, 23, 1) -> (23, 23, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5488, 5422, 4208) -> (5488, 5425, 4035).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5488, 5425, 4035) -> (5488, 5431, 4036).
    // Item 4244, a mine's `mtn`: builds (2029, 2029, 7) -> (2029, 2029, 4).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (616, 616, 1) -> (616, 616, 0).
    // Item 4262, a wonder's entry in its wonder list: builds (2029, 2029, 4) -> (2029, 2029, 2).
    // Item 4266, a guard order's walker: units (5488, 5431, 4036) -> (5488, 5485, 3903).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1304",
        3865,
        [
            (5488, 5488, 20),
            (2029, 2029, 2),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (616, 616, 0),
            (1616, 1616, 1),
            (271, 271, 1),
            (23, 23, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9448, 9448, 0),
        ],
    ),
    // Item 4205, the built start: units (4968, 4832, 4109) -> (4968, 4832, 4054); builds (1882,
    // 1882, 53) -> (1882, 1882, 42); cities (35, 35, 3) -> (35, 35, 1); world (7585, 7569,
    // 4445) -> (7585, 7585, 8).
    // Item 4209, the box's `clear_down`: items (70, 110, 62) -> (70, 70, 0); world (7585, 7585, 8) ->
    // (7585, 7585, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4968, 4832, 4054) -> (4968, 4832, 4048).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1882, 1882, 42)
    // -> (1882, 1882, 3).
    // Item 4226, a bird walked on neither line: units (4968, 4832, 4048) -> (4968, 4632, 4065), guys (1327, 1397, 70) -> (1327, 1327, 0).
    // Item 4228, a path stack's `size` its high water: units (4968, 4632, 4065) -> (4968, 4632, 4050).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (35, 35, 1) -> (35, 35, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4968, 4632, 4050) -> (4968, 4644, 4065).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4968, 4644, 4065) -> (4968, 4662, 4059).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4968, 4662, 4059) -> (4968, 4662, 4058).
    // Item 4244, a mine's `mtn`: builds (1882, 1882, 3) -> (1882, 1882, 2).
    // Item 4262, a wonder's entry in its wonder list: builds (1882, 1882, 2) -> (1882, 1882, 0).
    // Item 4266, a guard order's walker: units (4968, 4662, 4058) -> (4968, 4965, 3669).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1305b",
        4181,
        [
            (4968, 4968, 5),
            (1882, 1882, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 1),
            (1327, 1327, 0),
            (271, 271, 0),
            (35, 35, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2140, 2140, 67)
    // -> (2140, 2140, 13).
    // Item 4226, a bird walked on neither line: units (8490, 8757, 7384) -> (8490, 8397, 7122), guys (2197, 2323, 135) -> (2197, 2197, 9).
    // Item 4228, a path stack's `size` its high water: units (8490, 8397, 7122) -> (8490, 8397, 7078).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (330, 330, 4) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (8490, 8397, 7078) -> (8490, 8406, 7172).
    // Item 4234, a cast's three ranges walked: units (8490, 8406, 7172) -> (8490, 8445, 7099).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (8490, 8445, 7099) -> (8490, 8448, 7112).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (8490, 8448, 7112) -> (8490, 8448, 7106).
    // Item 4244, a mine's `mtn`: builds (2140, 2140, 13) -> (2140, 2140, 11).
    // Item 4258, `Group::add`'s OR of its new member's role: groups (652, 652, 5) -> (652, 652, 4).
    // Item 4262, a wonder's entry in its wonder list: builds (2140, 2140, 11) -> (2140, 2140, 9).
    // Item 4266, a guard order's walker: units (8490, 8448, 7106) -> (8490, 8451, 7004).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1320b",
        5066,
        [
            (8490, 8490, 86),
            (2140, 2140, 5),
            (0, 0, 0),
            (9, 9, 0),
            (2, 2, 0),
            (652, 652, 4),
            (2197, 2197, 9),
            (271, 271, 2),
            (39, 39, 1),
            (5, 5, 0),
            (330, 330, 0),
            (9476, 9476, 1),
        ],
    ),
    // Item 4205, the built start: units (3933, 4098, 3225) -> (3933, 4098, 3168); builds (1792,
    // 1792, 44) -> (1792, 1792, 28); cities (27, 27, 2) -> (27, 27, 1); world (7615, 7599,
    // 4535) -> (7615, 7615, 4).
    // Item 4209, the box's `clear_down`: items (90, 110, 44) -> (90, 90, 0); world (7615, 7615, 4) ->
    // (7615, 7615, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3933, 4098, 3168) -> (3933, 4098, 3164).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1792, 1792, 28)
    // -> (1792, 1792, 2).
    // Item 4226, a bird walked on neither line: units (3933, 4098, 3164) -> (3933, 3708, 3025), guys (1029, 1162, 133) -> (1029, 1029, 0).
    // Item 4228, a path stack's `size` its high water: units (3933, 3708, 3025) -> (3933, 3708, 3005).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3933, 3708, 3005) -> (3933, 3717, 2990).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (3933, 3717, 2990) -> (3933, 3720, 3013).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3933, 3720, 3013) -> (3933, 3720, 3008).
    // Item 4244, a mine's `mtn`: builds (1792, 1792, 2) -> (1792, 1792, 1).
    // Item 4266, a guard order's walker: units (3933, 3720, 3008) -> (3933, 3903, 2974).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1323b",
        3052,
        [
            (3933, 3933, 5),
            (1792, 1792, 0),
            (0, 0, 0),
            (3, 3, 0),
            (2, 2, 0),
            (544, 544, 0),
            (1029, 1029, 0),
            (271, 271, 0),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1918, 1918, 75)
    // -> (1918, 1918, 13).
    // Item 4226, a bird walked on neither line: units (7276, 7557, 6122) -> (7276, 7177, 5851), guys (2055, 2188, 142) -> (2055, 2055, 9).
    // Item 4228, a path stack's `size` its high water: units (7276, 7177, 5851) -> (7276, 7177, 5824).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0), goods (330, 330, 4) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (7276, 7177, 5824) -> (7276, 7186, 6191).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (7276, 7186, 6191) -> (7276, 7198, 6066).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (7276, 7198, 6066) -> (7276, 7198, 6063).
    // Item 4244, a mine's `mtn`: builds (1918, 1918, 13) -> (1918, 1918, 11).
    // Item 4262, a wonder's entry in its wonder list: builds (1918, 1918, 11) -> (1918, 1918, 9).
    // Item 4266, a guard order's walker: units (7276, 7198, 6063) -> (7276, 7237, 5305).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1324c",
        4904,
        [
            (7276, 7276, 94),
            (1918, 1918, 5),
            (0, 0, 0),
            (15, 15, 0),
            (2, 2, 0),
            (676, 676, 9),
            (2055, 2055, 9),
            (271, 271, 1),
            (27, 27, 0),
            (20, 20, 0),
            (330, 330, 0),
            (9466, 9466, 1),
        ],
    ),
    // Item 4208, the whales' piece: units (10367, 11043, 9354) -> (10367, 11043, 9288); builds (2521,
    // 2521, 93) -> (2521, 2521, 79); groups (682, 682, 13) -> (682, 682, 8); guys (2706, 2965, 373) ->
    // (2706, 2965, 273); world (9666, 8079, 6675) -> (9666, 9666, 6).
    // Item 4209, the box's `clear_down`: items (10, 35, 27) -> (10, 10, 0); world (9666, 9666, 6) ->
    // (9666, 9666, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (10367, 11043, 9288) -> (10367, 11043, 9283).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2521, 2521, 79)
    // -> (2521, 2521, 12).
    // Item 4226, a bird walked on neither line: units (10367, 11043, 9283) -> (10367, 10283, 8611), guys (2706, 2965, 273) -> (2706, 2706, 14).
    // Item 4228, a path stack's `size` its high water: units (10367, 10283, 8611) -> (10367, 10283, 8564).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (43, 43, 5) -> (43, 43, 2), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (10367, 10283, 8564) -> (10367, 10298, 7068).
    // Item 4234, a cast's three ranges walked: units (10367, 10298, 7068) -> (10367, 10352, 6938).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (10367, 10352, 6938) -> (10367, 10352, 6935).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (10367, 10352, 6935) -> (10367, 10352, 6933).
    // Item 4244, a mine's `mtn`: builds (2521, 2521, 12) -> (2521, 2521, 10).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (10367, 10352, 6933) -> (10367, 10352, 6932); guys (2706, 2706, 14) -> (2706, 2706, 13).
    // Item 4262, a wonder's entry in its wonder list: builds (2521, 2521, 10) -> (2521, 2521, 7).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1325c",
        5578,
        [
            (10367, 10367, 132),
            (2521, 2521, 6),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (682, 682, 8),
            (2706, 2706, 13),
            (271, 271, 1),
            (43, 43, 2),
            (10, 10, 0),
            (330, 330, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2161, 2161, 71)
    // -> (2161, 2161, 8).
    // Item 4226, a bird walked on neither line: units (6268, 6911, 5466) -> (6268, 6241, 4824), guys (1993, 2224, 241) -> (1993, 1993, 10).
    // Item 4228, a path stack's `size` its high water: units (6268, 6241, 4824) -> (6268, 6241, 4790).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 1) -> (30, 30, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6268, 6241, 4790) -> (6268, 6250, 5138).
    // Item 4234, a cast's three ranges walked: units (6268, 6250, 5138) -> (6268, 6256, 5148).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6268, 6256, 5148) -> (6268, 6256, 5147).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6268, 6256, 5147) -> (6268, 6256, 5145).
    // Item 4244, a mine's `mtn`: builds (2161, 2161, 8) -> (2161, 2161, 5).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (616, 616, 7) -> (616, 616, 4).
    // Item 4262, a wonder's entry in its wonder list: builds (2161, 2161, 5) -> (2161, 2161, 3).
    // Item 4266, a guard order's walker: units (6268, 6256, 5145) -> (6268, 6259, 5141).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1326",
        4567,
        [
            (6268, 6268, 81),
            (2161, 2161, 1),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (616, 616, 4),
            (1993, 1993, 10),
            (271, 271, 1),
            (30, 30, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9244, 9244, 0),
        ],
    ),
    // Item 4205, the built start: units (13393, 13427, 11988) -> (13393, 13427, 11927); builds
    // (4230, 4230, 113) -> (4230, 4230, 98); guys (3459, 3774, 361) -> (3459, 3774, 325); world
    // (8592, 7873, 5834) -> (8592, 8592, 16).
    // Item 4209, the box's `clear_down`: builds (4230, 4230, 98) -> (4230, 4230, 97); items (25, 110,
    // 94) -> (25, 25, 0); world (8592, 8592, 16) -> (8592, 8592, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (13393, 13427, 11927) -> (13393, 13427, 11925).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (4230, 4230, 97)
    // -> (4230, 4230, 23).
    // Item 4226, a bird walked on neither line: units (13393, 13427, 11925) -> (13393, 12502, 11980), guys (3459, 3774, 325) -> (3459, 3459, 10).
    // Item 4228, a path stack's `size` its high water: units (13393, 12502, 11980) -> (13393, 12502, 11962).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (59, 59, 6) -> (59, 59, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (13393, 12502, 11962) -> (13393, 12523, 12044).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (13393, 12523, 12044) -> (13393, 12535, 12068).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (13393, 12535, 12068) -> (13393, 12535, 12067).
    // Item 4244, a mine's `mtn`: builds (4230, 4230, 23) -> (4230, 4230, 19).
    // Item 4262, a wonder's entry in its wonder list: builds (4230, 4230, 19) -> (4230, 4230, 16).
    // Item 4266, a guard order's walker: units (13393, 12535, 12067) -> (13393, 13375, 4046).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1327",
        10930,
        [
            (13393, 13393, 10),
            (4230, 4230, 15),
            (0, 0, 0),
            (0, 0, 0),
            (4, 4, 0),
            (574, 574, 2),
            (3459, 3459, 10),
            (271, 271, 1),
            (59, 59, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2354, 2354, 58)
    // -> (2354, 2354, 7).
    // Item 4226, a bird walked on neither line: units (5746, 5891, 4373) -> (5746, 5386, 4498), guys (1463, 1638, 175) -> (1463, 1463, 0).
    // Item 4228, a path stack's `size` its high water: units (5746, 5386, 4498) -> (5746, 5386, 4477).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5746, 5386, 4477) -> (5746, 5395, 4711).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5746, 5395, 4711) -> (5746, 5407, 4697).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5746, 5407, 4697) -> (5746, 5407, 4694).
    // Item 4244, a mine's `mtn`: builds (2354, 2354, 7) -> (2354, 2354, 5).
    // Item 4266, a guard order's walker: units (5746, 5407, 4694) -> (5746, 5653, 4838).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1332a",
        3627,
        [
            (5746, 5746, 2),
            (2354, 2354, 1),
            (0, 0, 0),
            (48, 48, 0),
            (14, 14, 0),
            (580, 580, 5),
            (1463, 1463, 0),
            (271, 271, 1),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2237, 2237, 59)
    // -> (2237, 2237, 10).
    // Item 4226, a bird walked on neither line: units (4745, 4928, 3883) -> (4745, 4523, 3712), guys (1352, 1492, 143) -> (1352, 1352, 3).
    // Item 4228, a path stack's `size` its high water: units (4745, 4523, 3712) -> (4745, 4523, 3686).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0), goods (180, 180, 1) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4745, 4523, 3686) -> (4745, 4532, 3677).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4745, 4532, 3677) -> (4745, 4541, 3675).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4745, 4541, 3675) -> (4745, 4541, 3673).
    // Item 4244, a mine's `mtn`: builds (2237, 2237, 10) -> (2237, 2237, 9).
    // Item 4257, the garrison chain's order: builds (2237, 2237, 9) -> (2237, 2237, 6).
    // Item 4262, a wonder's entry in its wonder list: builds (2237, 2237, 6) -> (2237, 2237, 5).
    // Item 4266, a guard order's walker: units (4745, 4541, 3673) -> (4745, 4718, 3501).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1351",
        4303,
        [
            (4745, 4745, 11),
            (2237, 2237, 5),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (562, 562, 2),
            (1352, 1352, 3),
            (271, 271, 0),
            (30, 30, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8328, 8328, 2),
        ],
    ),
    // Item 4205, the built start: units (4512, 4732, 3134) -> (4512, 4732, 3067); builds (2143,
    // 2143, 75) -> (2143, 2143, 61); guys (1286, 1426, 176) -> (1286, 1426, 140); cities (30,
    // 30, 2) -> (30, 30, 1); world (8298, 7579, 5257) -> (8298, 8298, 7).
    // Item 4209, the box's `clear_down`: builds (2143, 2143, 61) -> (2143, 2143, 60); items (75, 110,
    // 56) -> (75, 75, 0); world (8298, 8298, 7) -> (8298, 8298, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4512, 4732, 3067) -> (4512, 4732, 3062).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2143, 2143, 60)
    // -> (2143, 2143, 7).
    // Item 4226, a bird walked on neither line: units (4512, 4732, 3062) -> (4512, 4332, 2977), guys (1286, 1426, 140) -> (1286, 1286, 0).
    // Item 4228, a path stack's `size` its high water: units (4512, 4332, 2977) -> (4512, 4332, 2952).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 1) -> (30, 30, 0), goods (180, 180, 1) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4512, 4332, 2952) -> (4512, 4341, 3401).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4512, 4341, 3401) -> (4512, 4341, 3400).
    // Item 4244, a mine's `mtn`: builds (2143, 2143, 7) -> (2143, 2143, 6).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 3) -> (556, 556, 2).
    // Item 4257, the garrison chain's order: builds (2143, 2143, 6) -> (2143, 2143, 4).
    // Item 4266, a guard order's walker: units (4512, 4341, 3400) -> (4512, 4509, 3036).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1351c",
        3521,
        [
            (4512, 4512, 11),
            (2143, 2143, 4),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (556, 556, 2),
            (1286, 1286, 0),
            (271, 271, 1),
            (30, 30, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8298, 8298, 1),
        ],
    ),
    // Item 4205, the built start: units (3610, 3841, 2961) -> (3610, 3841, 2904); builds (1836,
    // 1836, 45) -> (1836, 1836, 29); cities (27, 27, 2) -> (27, 27, 1); world (7777, 7761,
    // 4777) -> (7777, 7777, 6).
    // Item 4209, the box's `clear_down`: items (80, 110, 51) -> (80, 80, 0); world (7777, 7777, 6) ->
    // (7777, 7777, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (3610, 3841, 2904) -> (3610, 3841, 2898).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1836, 1836, 29)
    // -> (1836, 1836, 6).
    // Item 4226, a bird walked on neither line: units (3610, 3841, 2898) -> (3610, 3427, 2696), guys (902, 1042, 140) -> (902, 902, 0).
    // Item 4228, a path stack's `size` its high water: units (3610, 3427, 2696) -> (3610, 3427, 2682).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3610, 3427, 2682) -> (3610, 3436, 2699).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (3610, 3436, 2699) -> (3610, 3439, 2708).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3610, 3439, 2708) -> (3610, 3439, 2704).
    // Item 4244, a mine's `mtn`: builds (1836, 1836, 6) -> (1836, 1836, 5).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 4) -> (556, 556, 2).
    // Item 4257, the garrison chain's order: builds (1836, 1836, 5) -> (1836, 1836, 3).
    // Item 4266, a guard order's walker: units (3610, 3439, 2704) -> (3610, 3595, 2753).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1355",
        2933,
        [
            (3610, 3610, 6),
            (1836, 1836, 3),
            (0, 0, 0),
            (3, 3, 0),
            (2, 2, 0),
            (556, 556, 2),
            (902, 902, 0),
            (271, 271, 0),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1977, 1977, 43)
    // -> (1977, 1977, 3).
    // Item 4226, a bird walked on neither line: units (5707, 5937, 4766) -> (5707, 5422, 4648), guys (1427, 1602, 175) -> (1427, 1427, 0).
    // Item 4228, a path stack's `size` its high water: units (5707, 5422, 4648) -> (5707, 5422, 4629).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 1) -> (30, 30, 0), goods (180, 180, 1) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5707, 5422, 4629) -> (5707, 5431, 4576).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5707, 5431, 4576) -> (5707, 5434, 4597).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5707, 5434, 4597) -> (5707, 5434, 4595).
    // Item 4244, a mine's `mtn`: builds (1977, 1977, 3) -> (1977, 1977, 2).
    // Item 4262, a wonder's entry in its wonder list: builds (1977, 1977, 2) -> (1977, 1977, 1).
    // Item 4266, a guard order's walker: units (5707, 5434, 4595) -> (5707, 5701, 4203).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4320, the score's dirty bits (`0x800000`, `0x1000000`) emitted, leader 1's 0x800003 agreeing: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1357",
        3439,
        [
            (5707, 5707, 10),
            (1977, 1977, 1),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 2),
            (1427, 1427, 0),
            (271, 271, 0),
            (30, 30, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8528, 8528, 0),
        ],
    ),
    // Item 4205, the built start: units (6464, 6432, 5647) -> (6464, 6432, 5603); builds (2974,
    // 2974, 108) -> (2974, 2974, 94); cities (48, 48, 4) -> (48, 48, 3); world (8007, 7991,
    // 5220) -> (8007, 8007, 12).
    // Item 4209, the box's `clear_down`: items (60, 110, 69) -> (60, 60, 0); world (8007, 8007, 12) ->
    // (8007, 8007, 2).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2974, 2974, 94)
    // -> (2974, 2974, 33).
    // Item 4226, a bird walked on neither line: units (6464, 6432, 5603) -> (6464, 6023, 5643), guys (1584, 1724, 144) -> (1584, 1584, 4).
    // Item 4228, a path stack's `size` its high water: units (6464, 6023, 5643) -> (6464, 6023, 5626).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (48, 48, 3) -> (48, 48, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6464, 6023, 5626) -> (6464, 6041, 5597).
    // Item 4234, a cast's three ranges walked: units (6464, 6041, 5597) -> (6464, 6044, 5583).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6464, 6044, 5583) -> (6464, 6137, 5632).
    // Item 4244, a mine's `mtn`: builds (2974, 2974, 33) -> (2974, 2974, 31).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (6464, 6137, 5632) -> (6464, 6137, 5631); guys (1584, 1584, 4) -> (1584, 1584, 3).
    // Item 4257, the garrison chain's order: builds (2974, 2974, 31) -> (2974, 2974, 29).
    // Item 4262, a wonder's entry in its wonder list: builds (2974, 2974, 29) -> (2974, 2974, 25).
    // Item 4266, a guard order's walker: units (6464, 6137, 5631) -> (6464, 6455, 3439).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 4) -> (271, 271, 3).
    (
        "run1359",
        5619,
        [
            (6464, 6464, 9),
            (2974, 2974, 23),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 3),
            (1584, 1584, 3),
            (271, 271, 3),
            (48, 48, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2974, 2974, 96)
    // -> (2974, 2974, 33).
    // Item 4226, a bird walked on neither line: units (6408, 6424, 5541) -> (6408, 6009, 5562), guys (1598, 1738, 144) -> (1598, 1598, 4).
    // Item 4228, a path stack's `size` its high water: units (6408, 6009, 5562) -> (6408, 6009, 5546).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (48, 48, 3) -> (48, 48, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6408, 6009, 5546) -> (6408, 6027, 5594).
    // Item 4234, a cast's three ranges walked: units (6408, 6027, 5594) -> (6408, 6030, 5584).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6408, 6030, 5584) -> (6408, 6102, 5553).
    // Item 4244, a mine's `mtn`: builds (2974, 2974, 33) -> (2974, 2974, 31).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (6408, 6102, 5553) -> (6408, 6102, 5552); guys (1598, 1598, 4) -> (1598, 1598, 3).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 4) -> (556, 556, 3).
    // Item 4257, the garrison chain's order: builds (2974, 2974, 31) -> (2974, 2974, 29).
    // Item 4262, a wonder's entry in its wonder list: builds (2974, 2974, 29) -> (2974, 2974, 25).
    // Item 4266, a guard order's walker: units (6408, 6102, 5552) -> (6408, 6399, 3362).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 4) -> (271, 271, 3).
    (
        "run1359b",
        5677,
        [
            (6408, 6408, 9),
            (2974, 2974, 23),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 3),
            (1598, 1598, 3),
            (271, 271, 3),
            (48, 48, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2974, 2974, 96)
    // -> (2974, 2974, 33).
    // Item 4226, a bird walked on neither line: units (6407, 6423, 5538) -> (6407, 6008, 5559), guys (1598, 1738, 144) -> (1598, 1598, 4).
    // Item 4228, a path stack's `size` its high water: units (6407, 6008, 5559) -> (6407, 6008, 5543).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (48, 48, 3) -> (48, 48, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6407, 6008, 5543) -> (6407, 6026, 5590).
    // Item 4234, a cast's three ranges walked: units (6407, 6026, 5590) -> (6407, 6029, 5580).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6407, 6029, 5580) -> (6407, 6101, 5554).
    // Item 4244, a mine's `mtn`: builds (2974, 2974, 33) -> (2974, 2974, 31).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (6407, 6101, 5554) -> (6407, 6101, 5553); guys (1598, 1598, 4) -> (1598, 1598, 3).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 4) -> (556, 556, 3).
    // Item 4257, the garrison chain's order: builds (2974, 2974, 31) -> (2974, 2974, 29).
    // Item 4262, a wonder's entry in its wonder list: builds (2974, 2974, 29) -> (2974, 2974, 25).
    // Item 4266, a guard order's walker: units (6407, 6101, 5553) -> (6407, 6398, 3362).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 4) -> (271, 271, 3).
    (
        "run1359c",
        5678,
        [
            (6407, 6407, 9),
            (2974, 2974, 23),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (556, 556, 3),
            (1598, 1598, 3),
            (271, 271, 3),
            (48, 48, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2544, 2544, 56)
    // -> (2544, 2544, 25).
    // Item 4226, a bird walked on neither line: units (6810, 6671, 5720) -> (6810, 6366, 5744), guys (1847, 1952, 105) -> (1847, 1847, 0).
    // Item 4228, a path stack's `size` its high water: units (6810, 6366, 5744) -> (6810, 6366, 5722).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (37, 37, 2) -> (37, 37, 0), goods (180, 180, 2) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6810, 6366, 5722) -> (6810, 6381, 5712).
    // Item 4234, a cast's three ranges walked: units (6810, 6381, 5712) -> (6810, 6384, 5684).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6810, 6384, 5684) -> (6810, 6393, 5700).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6810, 6393, 5700) -> (6810, 6393, 5699).
    // Item 4244, a mine's `mtn`: builds (2544, 2544, 25) -> (2544, 2544, 22).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 1) -> (556, 556, 0).
    // Item 4262, a wonder's entry in its wonder list: builds (2544, 2544, 22) -> (2544, 2544, 19).
    // Item 4266, a guard order's walker: units (6810, 6393, 5699) -> (6810, 6783, 5665).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2544, 2544, 19) -> (2544, 2544, 3), leaders (271, 271, 8) -> (271, 271, 4).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1361",
        4840,
        [
            (6810, 6810, 15),
            (2544, 2544, 3),
            (0, 0, 0),
            (0, 0, 0),
            (4, 4, 0),
            (556, 556, 0),
            (1847, 1847, 0),
            (271, 271, 1),
            (37, 37, 0),
            (50, 50, 0),
            (180, 180, 0),
            (8476, 8476, 0),
        ],
    ),
    // Item 4205, the built start: units (5194, 5195, 3939) -> (5194, 5195, 3876); builds (2147,
    // 2147, 64) -> (2147, 2147, 49); guys (1419, 1517, 134) -> (1419, 1517, 98); cities (27,
    // 27, 3) -> (27, 27, 1); world (8334, 7615, 5331) -> (8334, 8334, 3).
    // Item 4209, the box's `clear_down`: builds (2147, 2147, 49) -> (2147, 2147, 48); items (90, 110,
    // 44) -> (90, 90, 0); world (8334, 8334, 3) -> (8334, 8334, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (5194, 5195, 3876) -> (5194, 5195, 3875).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2147, 2147, 48)
    // -> (2147, 2147, 5).
    // Item 4226, a bird walked on neither line: units (5194, 5195, 3875) -> (5194, 4915, 3902), guys (1419, 1517, 98) -> (1419, 1419, 0).
    // Item 4228, a path stack's `size` its high water: units (5194, 4915, 3902) -> (5194, 4915, 3872).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0), goods (180, 180, 1) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5194, 4915, 3872) -> (5194, 4924, 3964).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5194, 4924, 3964) -> (5194, 4942, 3888).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5194, 4942, 3888) -> (5194, 4942, 3883).
    // Item 4244, a mine's `mtn`: builds (2147, 2147, 5) -> (2147, 2147, 3).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 2) -> (556, 556, 1).
    // Item 4262, a wonder's entry in its wonder list: builds (2147, 2147, 3) -> (2147, 2147, 2).
    // Item 4266, a guard order's walker: units (5194, 4942, 3883) -> (5194, 5149, 3945).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1381",
        3262,
        [
            (5194, 5194, 9),
            (2147, 2147, 2),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (556, 556, 1),
            (1419, 1419, 0),
            (271, 271, 0),
            (27, 27, 0),
            (90, 90, 0),
            (180, 180, 0),
            (8334, 8334, 1),
        ],
    ),
    // Item 4205, the built start: units (5014, 5130, 4171) -> (5014, 5130, 4106); builds (2015,
    // 2015, 67) -> (2015, 2015, 52); guys (1349, 1482, 169) -> (1349, 1482, 133); cities (20,
    // 20, 1) -> (20, 20, 0); world (8340, 7621, 5320) -> (8340, 8340, 7).
    // Item 4209, the box's `clear_down`: items (70, 110, 61) -> (70, 70, 0); world (8340, 8340, 7) ->
    // (8340, 8340, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (5014, 5130, 4106) -> (5014, 5130, 4101).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2015, 2015, 52)
    // -> (2015, 2015, 7).
    // Item 4226, a bird walked on neither line: units (5014, 5130, 4101) -> (5014, 4750, 3998), guys (1349, 1482, 133) -> (1349, 1349, 0).
    // Item 4228, a path stack's `size` its high water: units (5014, 4750, 3998) -> (5014, 4750, 3975).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5014, 4750, 3975) -> (5014, 4753, 4000).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5014, 4753, 4000) -> (5014, 4759, 4044).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5014, 4759, 4044) -> (5014, 4759, 4043).
    // Item 4244, a mine's `mtn`: builds (2015, 2015, 7) -> (2015, 2015, 6).
    // Item 4257, the garrison chain's order: builds (2015, 2015, 6) -> (2015, 2015, 3).
    // Item 4266, a guard order's walker: units (5014, 4759, 4043) -> (5014, 5002, 3308).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1382",
        3228,
        [
            (5014, 5014, 7),
            (2015, 2015, 3),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 0),
            (1349, 1349, 0),
            (271, 271, 1),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1184, 1184, 7) ->
    // (1184, 1184, 1).
    // Item 4226, a bird walked on neither line: units (2933, 3253, 333) -> (2933, 2933, 13), guys (967, 1079, 112) -> (967, 967, 0).
    // Item 4228, a path stack's `size` its high water: units (2933, 2933, 13) -> (2933, 2933, 0).
    // Item 4244, a mine's `mtn`: builds (1184, 1184, 1) -> (1184, 1184, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run1383",
        1180,
        [
            (2933, 2933, 0),
            (1184, 1184, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 1),
            (967, 967, 0),
            (271, 271, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1774, 1774, 28)
    // -> (1774, 1774, 2).
    // Item 4226, a bird walked on neither line: units (5193, 5463, 4526) -> (5193, 5043, 4256), guys (1609, 1756, 147) -> (1609, 1609, 0).
    // Item 4228, a path stack's `size` its high water: units (5193, 5043, 4256) -> (5193, 5043, 4235).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5193, 5043, 4235) -> (5193, 5052, 4073).
    // Item 4244, a mine's `mtn`: builds (1774, 1774, 2) -> (1774, 1774, 1).
    // Item 4266, a guard order's walker: units (5193, 5052, 4073) -> (5193, 5193, 10).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1385p",
        2775,
        [
            (5193, 5193, 10),
            (1774, 1774, 1),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 1),
            (1609, 1609, 0),
            (271, 271, 1),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1830, 1830, 58)
    // -> (1830, 1830, 15).
    // Item 4226, a bird walked on neither line: units (6512, 6830, 5755) -> (6512, 6365, 5444), guys (1927, 2088, 165) -> (1927, 1927, 4).
    // Item 4228, a path stack's `size` its high water: units (6512, 6365, 5444) -> (6512, 6365, 5422).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (35, 35, 3) -> (35, 35, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6512, 6365, 5422) -> (6512, 6377, 5438).
    // Item 4242, an army's pushed group takes `add_group`'s `get_num_cap`: groups (694, 694, 14) -> (694, 694, 13).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (6512, 6377, 5438) -> (6512, 6386, 5212).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6512, 6386, 5212) -> (6512, 6386, 5209).
    // Item 4244, a mine's `mtn`: builds (1830, 1830, 15) -> (1830, 1830, 14).
    // Item 4262, a wonder's entry in its wonder list: builds (1830, 1830, 14) -> (1830, 1830, 13).
    // Item 4266, a guard order's walker: units (6512, 6386, 5209) -> (6512, 6497, 5472).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (1830, 1830, 13) -> (1830, 1830, 3), leaders (271, 271, 8) -> (271, 271, 4).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run1391",
        4174,
        [
            (6512, 6512, 38),
            (1830, 1830, 2),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (694, 694, 13),
            (1927, 1927, 4),
            (271, 271, 1),
            (35, 35, 0),
            (10, 10, 0),
            (330, 330, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2128, 2127, 669)
    // -> (2128, 2127, 613).
    // Item 4226, a bird walked on neither line: units (6953, 7259, 6066) -> (6953, 6899, 5767), guys (1829, 1955, 140) -> (1829, 1829, 14).
    // Item 4228, a path stack's `size` its high water: units (6953, 6899, 5767) -> (6953, 6899, 5737).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (33, 33, 1) -> (33, 33, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (6953, 6899, 5737) -> (6953, 6908, 5824).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (6953, 6908, 5824) -> (6953, 6908, 5823).
    // Item 4159's third residue: guys (1829, 1829, 14) -> (1829, 1829, 12), a newborn ship's turret at its rest.
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (6953, 6908, 5823) -> (6953, 6908, 5819).
    // Item 4244, a mine's `mtn`: builds (2128, 2127, 613) -> (2128, 2127, 611).
    // Item 4266, a guard order's walker: units (6953, 6908, 5819) -> (6953, 6920, 5813).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1416",
        4016,
        [
            (6953, 6953, 84),
            (2128, 2127, 609),
            (0, 0, 0),
            (18, 18, 1),
            (0, 0, 0),
            (634, 634, 9),
            (1829, 1829, 12),
            (271, 271, 2),
            (33, 33, 0),
            (5, 5, 0),
            (330, 330, 0),
            (9554, 9554, 10),
        ],
    ),
    // Item 4205, the built start: units (4518, 4705, 3767) -> (4518, 4705, 3703); builds (2321,
    // 2321, 75) -> (2321, 2321, 61); guys (1265, 1405, 178) -> (1265, 1405, 141); cities (33,
    // 33, 3) -> (33, 33, 2); world (8274, 7555, 5208) -> (8274, 8274, 8).
    // Item 4209, the box's `clear_down`: items (75, 110, 54) -> (75, 75, 0); world (8274, 8274, 8) ->
    // (8274, 8274, 1).
    // Item 4218, the idle latch (`flags & 0x08`): units (4518, 4705, 3703) -> (4518, 4705, 3700).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2321, 2321, 61)
    // -> (2321, 2321, 9).
    // Item 4226, a bird walked on neither line: units (4518, 4705, 3700) -> (4518, 4305, 3524), guys (1265, 1405, 141) -> (1265, 1265, 1).
    // Item 4228, a path stack's `size` its high water: units (4518, 4305, 3524) -> (4518, 4305, 3498).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (33, 33, 2) -> (33, 33, 0), goods (180, 180, 1) -> (180, 180, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4518, 4305, 3498) -> (4518, 4314, 3445).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4518, 4314, 3445) -> (4518, 4326, 3481).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4518, 4326, 3481) -> (4518, 4326, 3480).
    // Item 4244, a mine's `mtn`: builds (2321, 2321, 9) -> (2321, 2321, 8).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (544, 544, 1) -> (544, 544, 0).
    // Item 4257, the garrison chain's order: builds (2321, 2321, 8) -> (2321, 2321, 6).
    // Item 4262, a wonder's entry in its wonder list: builds (2321, 2321, 6) -> (2321, 2321, 5).
    // Item 4266, a guard order's walker: units (4518, 4326, 3480) -> (4518, 4515, 2748).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1417",
        4016,
        [
            (4518, 4518, 13),
            (2321, 2321, 5),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 0),
            (1265, 1265, 1),
            (271, 271, 2),
            (33, 33, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8274, 8274, 1),
        ],
    ),
    // Item 4209, the box's `clear_down`: builds (2033, 2033, 75) -> (2033, 2033, 74); items (60, 110,
    // 69) -> (60, 60, 0).
    // Item 4210, walked from the built start (4205b's words): units (4406, 4486, 3414) -> (4406,
    // 4486, 3351); builds (2033, 2033, 74) -> (2033, 2033, 56); guys (1231, 1336, 141) -> (1231,
    // 1336, 105); cities (27, 27, 2) -> (27, 27, 1); world (8434, 7715, 5388) -> (8434, 8434, 2).
    // Item 4218, the idle latch (`flags & 0x08`): units (4406, 4486, 3351) -> (4406, 4486, 3348).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2033, 2033, 56)
    // -> (2033, 2033, 6).
    // Item 4226, a bird walked on neither line: units (4406, 4486, 3348) -> (4406, 4181, 3413), guys (1231, 1336, 105) -> (1231, 1231, 0).
    // Item 4228, a path stack's `size` its high water: units (4406, 4181, 3413) -> (4406, 4181, 3395).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4406, 4181, 3395) -> (4406, 4190, 3386).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4406, 4190, 3386) -> (4406, 4196, 3407).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4406, 4196, 3407) -> (4406, 4196, 3404).
    // Item 4244, a mine's `mtn`: builds (2033, 2033, 6) -> (2033, 2033, 5).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (544, 544, 2) -> (544, 544, 1).
    // Item 4257, the garrison chain's order: builds (2033, 2033, 5) -> (2033, 2033, 4).
    // Item 4266, a guard order's walker: units (4406, 4196, 3404) -> (4406, 4391, 3423).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1419b",
        3212,
        [
            (4406, 4406, 4),
            (2033, 2033, 4),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1231, 1231, 0),
            (271, 271, 2),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2515, 2513, 737)
    // -> (2515, 2513, 688).
    // Item 4226, a bird walked on neither line: units (5583, 5614, 4589) -> (5583, 5204, 4600), guys (1327, 1467, 141) -> (1327, 1327, 1).
    // Item 4228, a path stack's `size` its high water: units (5583, 5204, 4600) -> (5583, 5204, 4576).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (38, 38, 3) -> (38, 38, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5583, 5204, 4576) -> (5583, 5216, 4625).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5583, 5216, 4625) -> (5583, 5231, 4617).
    // Item 4244, a mine's `mtn`: builds (2515, 2513, 688) -> (2515, 2513, 687).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (544, 544, 1) -> (544, 544, 0).
    // Item 4257, the garrison chain's order: builds (2515, 2513, 687) -> (2515, 2513, 686).
    // Item 4262, a wonder's entry in its wonder list: builds (2515, 2513, 686) -> (2515, 2513, 685).
    // Item 4266, a guard order's walker: units (5583, 5231, 4617) -> (5583, 5423, 1536).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1420",
        4586,
        [
            (5583, 5423, 1536),
            (2515, 2513, 684),
            (0, 0, 0),
            (0, 0, 0),
            (8, 8, 0),
            (544, 544, 0),
            (1327, 1327, 1),
            (271, 271, 2),
            (38, 38, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2025, 2025, 32)
    // -> (2025, 2025, 5).
    // Item 4226, a bird walked on neither line: units (4129, 4318, 3320) -> (4129, 3913, 3142), guys (1138, 1278, 141) -> (1138, 1138, 1).
    // Item 4228, a path stack's `size` its high water: units (4129, 3913, 3142) -> (4129, 3913, 3123).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4129, 3913, 3123) -> (4129, 3922, 3162).
    // Item 4234, a cast's three ranges walked: units (4129, 3922, 3162) -> (4129, 3925, 3156).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4129, 3925, 3156) -> (4129, 3928, 3112).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4129, 3928, 3112) -> (4129, 3928, 3110).
    // Item 4244, a mine's `mtn`: builds (2025, 2025, 5) -> (2025, 2025, 4).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 3) -> (550, 550, 2).
    // Item 4257, the garrison chain's order: builds (2025, 2025, 4) -> (2025, 2025, 3).
    // Item 4266, a guard order's walker: units (4129, 3928, 3110) -> (4129, 4102, 3132).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1424",
        3156,
        [
            (4129, 4129, 4),
            (2025, 2025, 3),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (550, 550, 2),
            (1138, 1138, 1),
            (271, 271, 2),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2266, 2265, 848)
    // -> (2266, 2265, 805).
    // Item 4226, a bird walked on neither line: units (4175, 4291, 3428) -> (4175, 3986, 3329), guys (1227, 1332, 110) -> (1227, 1227, 5).
    // Item 4228, a path stack's `size` its high water: units (4175, 3986, 3329) -> (4175, 3986, 3310).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 2) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4175, 3986, 3310) -> (4175, 3995, 2857).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4175, 3995, 2857) -> (4175, 4001, 3196).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4175, 4001, 3196) -> (4175, 4001, 3194).
    // Item 4244, a mine's `mtn`: builds (2266, 2265, 805) -> (2266, 2265, 804).
    // Item 4266, a guard order's walker: units (4175, 4001, 3194) -> (4175, 4169, 3190).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 4) -> (271, 271, 3).
    (
        "run1426b",
        3722,
        [
            (4175, 4175, 9),
            (2266, 2265, 803),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (544, 544, 1),
            (1227, 1227, 5),
            (271, 271, 3),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2154, 2151, 604)
    // -> (2154, 2151, 571).
    // Item 4226, a bird walked on neither line: units (4789, 4721, 3901) -> (4789, 4316, 3924), guys (1208, 1348, 147) -> (1208, 1208, 7).
    // Item 4228, a path stack's `size` its high water: units (4789, 4316, 3924) -> (4789, 4316, 3906).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 1) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4789, 4316, 3906) -> (4789, 4325, 3938).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4789, 4325, 3938) -> (4789, 4343, 3883).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4789, 4343, 3883) -> (4789, 4343, 3880).
    // Item 4244, a mine's `mtn`: builds (2154, 2151, 571) -> (2154, 2151, 570).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (4789, 4343, 3880) -> (4789, 4343, 3879); guys (1208, 1208, 7) -> (1208, 1208, 6).
    // Item 4266, a guard order's walker: units (4789, 4343, 3879) -> (4789, 4520, 3877).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 6) -> (271, 271, 5).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run1437",
        3894,
        [
            (4789, 4529, 2797),
            (2154, 2151, 569),
            (0, 0, 0),
            (3, 3, 0),
            (4, 4, 0),
            (538, 538, 0),
            (1208, 1208, 6),
            (271, 271, 4),
            (30, 30, 0),
            (85, 85, 0),
            (180, 180, 0),
            (8414, 8414, 1),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (2475, 2702, 2286) -> (2475, 2702, 2279).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1890, 1890, 26)
    // -> (1890, 1890, 24).
    // Item 4226, a bird walked on neither line: units (2475, 2702, 2279) -> (2475, 2382, 2059), guys (672, 784, 113) -> (672, 672, 1).
    // Item 4228, a path stack's `size` its high water: units (2475, 2382, 2059) -> (2475, 2382, 2058).
    // Item 4231, a QUEUE_FIRST move's `get_num_cap` normalizing its record: groups (574, 574, 2) -> (574, 574, 1).
    // Item 4234, a cast's three ranges walked: units (2475, 2382, 2058) -> (2475, 2385, 2066).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2475, 2385, 2066) -> (2475, 2385, 2065).
    // Item 4244, a mine's `mtn`: builds (1890, 1890, 24) -> (1890, 1890, 21).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (574, 574, 1) -> (574, 574, 0).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (399, 399, 6) -> (399, 399, 5).
    // Item 4348, gaia's angles from the start dump: guys (672, 672, 1) -> (672, 672, 0).
    (
        "run1438",
        684,
        [
            (2475, 2475, 104),
            (1890, 1890, 21),
            (0, 0, 0),
            (15, 15, 0),
            (0, 0, 0),
            (574, 574, 0),
            (672, 672, 0),
            (399, 399, 5),
            (9, 9, 0),
            (135, 135, 0),
            (165, 165, 0),
            (7499, 7483, 4263),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 28) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2308, 2539, 2166) -> (2308, 2539, 2160).
    // Item 4226, a bird walked on neither line: units (2308, 2539, 2160) -> (2308, 2194, 1945), guys (638, 757, 120) -> (638, 638, 1).
    // Item 4228, a path stack's `size` its high water: units (2308, 2194, 1945) -> (2308, 2194, 1944).
    // Item 4234, a cast's three ranges walked: units (2308, 2194, 1944) -> (2308, 2197, 1946).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2308, 2197, 1946) -> (2308, 2197, 1945).
    // Item 4244, a mine's `mtn`: builds (1968, 1968, 25) -> (1968, 1968, 22).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (568, 568, 2) -> (568, 568, 0).
    // Item 4266, a guard order's walker: units (2308, 2197, 1945) -> (2308, 2269, 1887).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (399, 399, 6) -> (399, 399, 5).
    // Item 4348, gaia's angles from the start dump: guys (638, 638, 1) -> (638, 638, 0).
    (
        "run1440",
        809,
        [
            (2308, 2308, 105),
            (1968, 1968, 22),
            (0, 0, 0),
            (18, 18, 0),
            (14, 14, 0),
            (568, 568, 0),
            (638, 638, 0),
            (399, 399, 5),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1890, 1887, 518)
    // -> (1890, 1887, 481).
    // Item 4226, a bird walked on neither line: units (4274, 4219, 3474) -> (4274, 4039, 3483), guys (1108, 1171, 64) -> (1108, 1108, 1).
    // Item 4228, a path stack's `size` its high water: units (4274, 4039, 3483) -> (4274, 4039, 3464).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 2) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4274, 4039, 3464) -> (4274, 4048, 3420).
    // Item 4244, a mine's `mtn`: builds (1890, 1887, 481) -> (1890, 1887, 480).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 3) -> (550, 550, 1).
    // Item 4266, a guard order's walker: units (4274, 4048, 3420) -> (4274, 4210, 3390).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1441",
        3294,
        [
            (4274, 4240, 907),
            (1890, 1887, 479),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (550, 550, 1),
            (1108, 1108, 1),
            (271, 271, 2),
            (30, 30, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8276, 8276, 9),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (125, 135, 37) -> (125, 125, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2022, 2281, 1870) -> (2022, 2281, 1864).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1968, 1968, 29)
    // -> (1968, 1968, 25).
    // Item 4226, a bird walked on neither line: units (2022, 2281, 1864) -> (2022, 1941, 1613), guys (604, 723, 120) -> (604, 604, 1).
    // Item 4228, a path stack's `size` its high water: units (2022, 1941, 1613) -> (2022, 1941, 1612).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2022, 1941, 1612) -> (2022, 1941, 1611).
    // Item 4244, a mine's `mtn`: builds (1968, 1968, 25) -> (1968, 1968, 22).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (568, 568, 2) -> (568, 568, 0).
    // Item 4266, a guard order's walker: units (2022, 1941, 1611) -> (2022, 1965, 1624).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (399, 399, 6) -> (399, 399, 5).
    // Item 4348, gaia's angles from the start dump: guys (604, 604, 1) -> (604, 604, 0).
    (
        "run1451",
        945,
        [
            (2022, 2022, 110),
            (1968, 1968, 22),
            (0, 0, 0),
            (18, 18, 0),
            (32, 32, 0),
            (568, 568, 0),
            (604, 604, 0),
            (399, 399, 5),
            (9, 9, 0),
            (125, 125, 0),
            (165, 165, 0),
            (7585, 7569, 4477),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2655, 2718, 2286) -> (2655, 2718, 2279).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1144, 1144, 28)
    // -> (1144, 1144, 24).
    // Item 4226, a bird walked on neither line: units (2655, 2718, 2279) -> (2655, 2538, 2239), guys (725, 788, 64) -> (725, 725, 1).
    // Item 4228, a path stack's `size` its high water: units (2655, 2538, 2239) -> (2655, 2538, 2238).
    // Item 4231, a QUEUE_FIRST move's `get_num_cap` normalizing its record: groups (574, 574, 2) -> (574, 574, 1).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2655, 2538, 2238) -> (2655, 2538, 2237).
    // Item 4244, a mine's `mtn`: builds (1144, 1144, 24) -> (1144, 1144, 23).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (574, 574, 1) -> (574, 574, 0).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (399, 399, 4) -> (399, 399, 3).
    // Item 4348, gaia's angles from the start dump: guys (725, 725, 1) -> (725, 725, 0).
    (
        "run1457",
        700,
        [
            (2655, 2655, 116),
            (1144, 1144, 23),
            (0, 0, 0),
            (21, 21, 0),
            (2, 2, 0),
            (574, 574, 0),
            (725, 725, 0),
            (399, 399, 3),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7525, 7509, 4362),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2362, 2425, 1982) -> (2362, 2425, 1974).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1316, 1316, 26)
    // -> (1316, 1316, 24).
    // Item 4226, a bird walked on neither line: units (2362, 2425, 1974) -> (2362, 2245, 1934), guys (665, 728, 64) -> (665, 665, 1).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2362, 2245, 1934) -> (2362, 2245, 1933).
    // Item 4244, a mine's `mtn`: builds (1316, 1316, 24) -> (1316, 1316, 23).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (568, 568, 2) -> (568, 568, 0).
    // Item 4266, a guard order's walker: units (2362, 2245, 1933) -> (2362, 2314, 1927).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (403, 403, 5) -> (403, 403, 3).
    // Item 4348, gaia's angles from the start dump: guys (665, 665, 1) -> (665, 665, 0).
    (
        "run1459",
        780,
        [
            (2362, 2362, 115),
            (1316, 1316, 23),
            (0, 0, 0),
            (0, 0, 0),
            (22, 22, 0),
            (568, 568, 0),
            (665, 665, 0),
            (403, 403, 3),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7535, 7519, 4363),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2380, 2455, 1949) -> (2380, 2455, 1941).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1296, 1296, 27)
    // -> (1296, 1296, 24).
    // Item 4226, a bird walked on neither line: units (2380, 2455, 1941) -> (2380, 2275, 1872), guys (686, 749, 64) -> (686, 686, 1).
    // Item 4228, a path stack's `size` its high water: units (2380, 2275, 1872) -> (2380, 2275, 1874).
    // Item 4231, a QUEUE_FIRST move's `get_num_cap` normalizing its record: groups (568, 568, 3) -> (568, 568, 2).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2380, 2275, 1874) -> (2380, 2275, 1873).
    // Item 4244, a mine's `mtn`: builds (1296, 1296, 24) -> (1296, 1296, 23).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (568, 568, 2) -> (568, 568, 0).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (403, 403, 5) -> (403, 403, 3).
    // Item 4348, gaia's angles from the start dump: guys (686, 686, 1) -> (686, 686, 0).
    (
        "run1460",
        760,
        [
            (2380, 2380, 116),
            (1296, 1296, 23),
            (0, 0, 0),
            (9, 9, 0),
            (16, 16, 0),
            (568, 568, 0),
            (686, 686, 0),
            (403, 403, 3),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7523, 7507, 4355),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 29) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1392, 1679, 1048) -> (1392, 1679, 1040).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1777, 1777, 29)
    // -> (1777, 1777, 27).
    // Item 4226, a bird walked on neither line: units (1392, 1679, 1040) -> (1392, 1374, 753), guys (413, 518, 106) -> (413, 413, 1).
    // Item 4228, a path stack's `size` its high water: units (1392, 1374, 753) -> (1392, 1374, 747).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1392, 1374, 747) -> (1392, 1374, 746).
    // Item 4244, a mine's `mtn`: builds (1777, 1777, 27) -> (1777, 1777, 25).
    // Item 4348, gaia's angles from the start dump: guys (413, 413, 1) -> (413, 413, 0).
    (
        "run1469",
        627,
        [
            (1392, 1392, 102),
            (1777, 1777, 25),
            (0, 0, 0),
            (228, 0, 228),
            (0, 0, 0),
            (556, 556, 4),
            (413, 413, 0),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 29) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (1401, 1685, 1056) -> (1401, 1685, 1049).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1777, 1777, 29)
    // -> (1777, 1777, 27).
    // Item 4226, a bird walked on neither line: units (1401, 1685, 1049) -> (1401, 1380, 767), guys (413, 518, 106) -> (413, 413, 1).
    // Item 4228, a path stack's `size` its high water: units (1401, 1380, 767) -> (1401, 1380, 761).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1401, 1380, 761) -> (1401, 1380, 760).
    // Item 4244, a mine's `mtn`: builds (1777, 1777, 27) -> (1777, 1777, 25).
    // Item 4348, gaia's angles from the start dump: guys (413, 413, 1) -> (413, 413, 0).
    (
        "run1469b",
        628,
        [
            (1401, 1401, 102),
            (1777, 1777, 25),
            (0, 0, 0),
            (190, 0, 190),
            (0, 0, 0),
            (556, 556, 4),
            (413, 413, 0),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7475, 7459, 4267),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (130, 135, 30) -> (130, 130, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2180, 2393, 2009) -> (2180, 2393, 2002).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1777, 1777, 30)
    // -> (1777, 1777, 27).
    // Item 4226, a bird walked on neither line: units (2180, 2393, 2002) -> (2180, 2093, 1793), guys (612, 717, 106) -> (612, 612, 1).
    // Item 4228, a path stack's `size` its high water: units (2180, 2093, 1793) -> (2180, 2093, 1791).
    // Item 4234, a cast's three ranges walked: units (2180, 2093, 1791) -> (2180, 2096, 1780).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2180, 2096, 1780) -> (2180, 2096, 1779).
    // Item 4244, a mine's `mtn`: builds (1777, 1777, 27) -> (1777, 1777, 25).
    // Item 4348, gaia's angles from the start dump: guys (612, 612, 1) -> (612, 612, 0).
    (
        "run1472",
        707,
        [
            (2180, 2180, 102),
            (1777, 1777, 25),
            (0, 0, 0),
            (339, 0, 339),
            (4, 4, 0),
            (568, 568, 4),
            (612, 612, 0),
            (399, 399, 6),
            (9, 9, 0),
            (130, 130, 0),
            (165, 165, 0),
            (7527, 7511, 4420),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (4145, 4206, 3495) -> (4145, 4206, 3484).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1446, 1446, 28)
    // -> (1446, 1446, 25).
    // Item 4226, a bird walked on neither line: units (4145, 4206, 3484) -> (4145, 4106, 3427), guys (1381, 1416, 159) -> (1381, 1381, 124).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (405, 405, 0) -> (405, 405, 1).
    // Item 4234, a cast's three ranges walked: units (4145, 4106, 3427) -> (4145, 4109, 3247).
    // Item 4244, a mine's `mtn`: builds (1446, 1446, 25) -> (1446, 1446, 24).
    // Item 4348, gaia's angles from the start dump: guys (1381, 1381, 124) -> (1381, 1381, 1).
    (
        "run1484",
        676,
        [
            (4145, 4145, 279),
            (1446, 1446, 24),
            (0, 0, 0),
            (304, 0, 304),
            (0, 0, 0),
            (562, 562, 2),
            (1381, 1381, 1),
            (403, 403, 6),
            (9, 9, 0),
            (25, 25, 0),
            (405, 405, 1),
            (9053, 7519, 5588),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5340, 5621, 4718) -> (5340, 5621, 4710).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2736, 2736, 66)
    // -> (2736, 2736, 31).
    // Item 4226, a bird walked on neither line: units (5340, 5621, 4710) -> (5340, 5256, 4433), guys (1718, 1844, 247) -> (1718, 1718, 121).
    // Item 4228, a path stack's `size` its high water: units (5340, 5256, 4433) -> (5340, 5256, 4421).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (405, 405, 2) -> (405, 405, 3).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5340, 5256, 4421) -> (5340, 5262, 4360).
    // Item 4234, a cast's three ranges walked: units (5340, 5262, 4360) -> (5340, 5265, 4419).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (5340, 5265, 4419) -> (5340, 5268, 4324).
    // Item 4244, a mine's `mtn`: builds (2736, 2736, 31) -> (2736, 2736, 30).
    // Item 4266, a guard order's walker: units (5340, 5268, 4324) -> (5340, 5337, 2952).
    // Item 4348, gaia's angles from the start dump: guys (1718, 1718, 121) -> (1718, 1718, 0).
    (
        "run1487",
        1980,
        [
            (5340, 5340, 287),
            (2736, 2736, 30),
            (0, 0, 0),
            (208, 0, 208),
            (8, 8, 0),
            (610, 610, 2),
            (1718, 1718, 0),
            (407, 407, 6),
            (31, 31, 0),
            (25, 25, 0),
            (405, 405, 3),
            (9267, 7733, 6310),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (25, 45, 28) -> (25, 25, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (5026, 5286, 4896) -> (5026, 5286, 4889).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2180, 2180, 47)
    // -> (2180, 2180, 30).
    // Item 4226, a bird walked on neither line: units (5026, 5286, 4889) -> (5026, 4936, 4629), guys (1577, 1696, 241) -> (1577, 1577, 122).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (405, 405, 2) -> (405, 405, 3).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5026, 4936, 4629) -> (5026, 4939, 4500).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (5026, 4939, 4500) -> (5026, 4939, 4499).
    // Item 4244, a mine's `mtn`: builds (2180, 2180, 30) -> (2180, 2180, 29).
    // Item 4257, the garrison chain's order: builds (2180, 2180, 29) -> (2180, 2180, 27).
    // Item 4266, a guard order's walker: units (5026, 4939, 4499) -> (5026, 5005, 4438).
    // Item 4348, gaia's angles from the start dump: guys (1577, 1577, 122) -> (1577, 1577, 1).
    (
        "run1496",
        1539,
        [
            (5026, 5026, 287),
            (2180, 2180, 27),
            (0, 0, 0),
            (230, 0, 230),
            (2, 2, 0),
            (586, 586, 2),
            (1577, 1577, 1),
            (407, 407, 6),
            (23, 23, 0),
            (25, 25, 0),
            (405, 405, 3),
            (9183, 7649, 6125),
        ],
    ),
    // Item 4209, the box's `clear_down`: items (75, 110, 55) -> (75, 75, 0).
    // Item 4210, walked from the built start (4205b's words): units (4274, 4219, 3527) -> (4274,
    // 4219, 3476); builds (1890, 1887, 527) -> (1890, 1887, 518); guys (1108, 1171, 100) -> (1108,
    // 1171, 64); cities (30, 30, 3) -> (30, 30, 2); world (8276, 7557, 5229) -> (8276, 8276, 9).
    // Item 4218, the idle latch (`flags & 0x08`): units (4274, 4219, 3476) -> (4274, 4219, 3474).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1890, 1887, 518)
    // -> (1890, 1887, 481).
    // Item 4226, a bird walked on neither line: units (4274, 4219, 3474) -> (4274, 4039, 3483), guys (1108, 1171, 64) -> (1108, 1108, 1).
    // Item 4228, a path stack's `size` its high water: units (4274, 4039, 3483) -> (4274, 4039, 3464).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 2) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4274, 4039, 3464) -> (4274, 4048, 3420).
    // Item 4244, a mine's `mtn`: builds (1890, 1887, 481) -> (1890, 1887, 480).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 3) -> (550, 550, 1).
    // Item 4266, a guard order's walker: units (4274, 4048, 3420) -> (4274, 4210, 3390).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1559b",
        3294,
        [
            (4274, 4240, 907),
            (1890, 1887, 479),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (550, 550, 1),
            (1108, 1108, 1),
            (271, 271, 2),
            (30, 30, 0),
            (75, 75, 0),
            (180, 180, 0),
            (8276, 8276, 9),
        ],
    ),
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (267, 267, 2) -> (267, 267, 0).
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
            (267, 267, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1890, 1887, 518)
    // -> (1890, 1887, 481).
    // Item 4226, a bird walked on neither line: units (4274, 4219, 3474) -> (4274, 4039, 3483), guys (1108, 1171, 64) -> (1108, 1108, 1).
    // Item 4228, a path stack's `size` its high water: units (4274, 4039, 3483) -> (4274, 4039, 3464).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 2) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4274, 4039, 3464) -> (4274, 4048, 3420).
    // Item 4244, a mine's `mtn`: builds (1890, 1887, 481) -> (1890, 1887, 480).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (550, 550, 3) -> (550, 550, 1).
    // Item 4266, a guard order's walker: units (4274, 4048, 3420) -> (4274, 4210, 3390).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1564",
        3294,
        [
            (4274, 4240, 907),
            (1890, 1887, 479),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (550, 550, 1),
            (1108, 1108, 1),
            (271, 271, 2),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1890, 1887, 519)
    // -> (1890, 1887, 481).
    // Item 4226, a bird walked on neither line: units (4240, 4197, 3359) -> (4240, 4017, 3395), guys (1112, 1175, 65) -> (1112, 1112, 2).
    // Item 4228, a path stack's `size` its high water: units (4240, 4017, 3395) -> (4240, 4017, 3377).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4240, 4017, 3377) -> (4240, 4026, 3398).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4240, 4026, 3398) -> (4240, 4029, 3415).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4240, 4029, 3415) -> (4240, 4029, 3414).
    // Item 4244, a mine's `mtn`: builds (1890, 1887, 481) -> (1890, 1887, 480).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (556, 556, 2) -> (556, 556, 1).
    // Item 4266, a guard order's walker: units (4240, 4029, 3414) -> (4240, 4194, 3364).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 3) -> (271, 271, 2).
    (
        "run1570",
        3331,
        [
            (4240, 4200, 432),
            (1890, 1887, 479),
            (0, 0, 0),
            (0, 0, 0),
            (4, 4, 0),
            (556, 556, 1),
            (1112, 1112, 2),
            (271, 271, 2),
            (30, 30, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1731, 1729, 958)
    // -> (1731, 1729, 941).
    // Item 4226, a bird walked on neither line: units (4969, 4983, 4140) -> (4969, 4763, 4158), guys (1429, 1506, 78) -> (1429, 1429, 1).
    // Item 4228, a path stack's `size` its high water: units (4969, 4763, 4158) -> (4969, 4763, 4142).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (47, 47, 1) -> (47, 47, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4969, 4763, 4142) -> (4969, 4784, 4300).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (640, 640, 4) -> (640, 640, 3).
    // Item 4266, a guard order's walker: units (4969, 4784, 4300) -> (4969, 4889, 2804).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (1731, 1729, 941) -> (1731, 1729, 939), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run785",
        2178,
        [
            (4969, 4889, 2804),
            (1731, 1729, 938),
            (0, 0, 0),
            (0, 0, 0),
            (28, 28, 4),
            (640, 640, 3),
            (1429, 1429, 1),
            (271, 271, 4),
            (47, 47, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9108, 9108, 10),
        ],
    ),
    // Item 4208, the whales' piece: units (5889, 5855, 5006) -> (5889, 5855, 4979); builds (2157,
    // 2155, 1285) -> (2157, 2155, 1278); guys (1765, 1842, 178) -> (1765, 1842, 78); world (9526,
    // 7939, 6468) -> (9526, 9526, 15).
    // Item 4209, the box's `clear_down`: items (15, 35, 25) -> (15, 15, 0); world (9526, 9526, 15) ->
    // (9526, 9526, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (5889, 5855, 4979) -> (5889, 5855, 4982).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2157, 2155, 1278)
    // -> (2157, 2155, 1258).
    // Item 4226, a bird walked on neither line: units (5889, 5855, 4982) -> (5889, 5635, 5015), guys (1765, 1842, 78) -> (1765, 1765, 1).
    // Item 4228, a path stack's `size` its high water: units (5889, 5635, 5015) -> (5889, 5635, 5001).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (52, 52, 5) -> (52, 52, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5889, 5635, 5001) -> (5889, 5659, 5297).
    // Item 4234, a cast's three ranges walked: units (5889, 5659, 5297) -> (5889, 5662, 5221).
    // Item 4266, a guard order's walker: units (5889, 5662, 5221) -> (5889, 5809, 3724).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2157, 2155, 1258) -> (2157, 2155, 1256), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run789",
        2799,
        [
            (5889, 5809, 3724),
            (2157, 2155, 1255),
            (0, 0, 0),
            (281, 3, 280),
            (0, 0, 0),
            (664, 664, 2),
            (1765, 1765, 1),
            (271, 271, 4),
            (52, 52, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2059, 2057, 1202)
    // -> (2059, 2057, 1181).
    // Item 4226, a bird walked on neither line: units (5860, 5823, 4862) -> (5860, 5603, 4906), guys (1704, 1781, 78) -> (1704, 1704, 1).
    // Item 4228, a path stack's `size` its high water: units (5860, 5603, 4906) -> (5860, 5603, 4890).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (52, 52, 5) -> (52, 52, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5860, 5603, 4890) -> (5860, 5627, 5173).
    // Item 4234, a cast's three ranges walked: units (5860, 5627, 5173) -> (5860, 5633, 5166).
    // Item 4266, a guard order's walker: units (5860, 5633, 5166) -> (5860, 5780, 3667).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2059, 2057, 1181) -> (2059, 2057, 1179), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run795",
        2681,
        [
            (5860, 5780, 3667),
            (2059, 2057, 1178),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (652, 652, 2),
            (1704, 1704, 1),
            (271, 271, 4),
            (52, 52, 0),
            (15, 15, 0),
            (330, 330, 0),
            (9420, 9420, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (9526, 9603, 8636) -> (9526, 9603, 8609); builds (2384,
    // 2381, 1424) -> (2384, 2381, 1417); groups (688, 688, 6) -> (688, 688, 5); guys (2341, 2425, 195)
    // -> (2341, 2425, 95); world (9442, 7855, 6428) -> (9442, 9442, 16).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9442, 9442, 16) ->
    // (9442, 9442, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (9526, 9603, 8609) -> (9526, 9603, 8610).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2384, 2381, 1417)
    // -> (2384, 2381, 1400).
    // Item 4226, a bird walked on neither line: units (9526, 9603, 8610) -> (9526, 9363, 8566), guys (2341, 2425, 95) -> (2341, 2341, 11).
    // Item 4228, a path stack's `size` its high water: units (9526, 9363, 8566) -> (9526, 9363, 8551).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (58, 58, 5) -> (58, 58, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (9526, 9363, 8551) -> (9526, 9387, 8667).
    // Item 4234, a cast's three ranges walked: units (9526, 9387, 8667) -> (9526, 9396, 8822).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (9526, 9396, 8822) -> (9526, 9402, 8740).
    // Item 4257, the garrison chain's order: builds (2384, 2381, 1400) -> (2384, 2381, 1399).
    // Item 4266, a guard order's walker: units (9526, 9402, 8740) -> (9526, 9414, 8644).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2384, 2381, 1399) -> (2384, 2381, 1398), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run810",
        3631,
        [
            (9526, 9426, 7329),
            (2384, 2381, 1397),
            (0, 0, 0),
            (156, 0, 156),
            (0, 0, 0),
            (688, 688, 5),
            (2341, 2341, 11),
            (271, 271, 4),
            (58, 58, 0),
            (10, 10, 0),
            (330, 330, 0),
            (9442, 9442, 11),
        ],
    ),
    // Item 4208, the whales' piece: units (9339, 9413, 8439) -> (9339, 9413, 8412); builds (2473,
    // 2469, 1560) -> (2473, 2469, 1553); guys (2450, 2534, 195) -> (2450, 2534, 95); world (9552,
    // 7965, 6484) -> (9552, 9552, 15).
    // Item 4209, the box's `clear_down`: items (10, 35, 28) -> (10, 10, 0); world (9552, 9552, 15) ->
    // (9552, 9552, 11).
    // Item 4218, the idle latch (`flags & 0x08`): units (9339, 9413, 8412) -> (9339, 9413, 8415).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2473, 2469, 1553)
    // -> (2473, 2469, 1532).
    // Item 4226, a bird walked on neither line: units (9339, 9413, 8415) -> (9339, 9173, 8367), guys (2450, 2534, 95) -> (2450, 2450, 11).
    // Item 4228, a path stack's `size` its high water: units (9339, 9173, 8367) -> (9339, 9173, 8350).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (58, 58, 5) -> (58, 58, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (9339, 9173, 8350) -> (9339, 9197, 8567).
    // Item 4257, the garrison chain's order: builds (2473, 2469, 1532) -> (2473, 2469, 1531).
    // Item 4266, a guard order's walker: units (9339, 9197, 8567) -> (9339, 9227, 8404).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2473, 2469, 1531) -> (2473, 2469, 1530), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run823",
        4238,
        [
            (9339, 9227, 8404),
            (2473, 2469, 1529),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (676, 676, 6),
            (2450, 2450, 11),
            (271, 271, 4),
            (58, 58, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2473, 2469, 1553)
    // -> (2473, 2469, 1532).
    // Item 4226, a bird walked on neither line: units (9362, 9436, 8379) -> (9362, 9196, 8331), guys (2471, 2555, 94) -> (2471, 2471, 10).
    // Item 4228, a path stack's `size` its high water: units (9362, 9196, 8331) -> (9362, 9196, 8314).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (58, 58, 5) -> (58, 58, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (9362, 9196, 8314) -> (9362, 9220, 8572).
    // Item 4257, the garrison chain's order: builds (2473, 2469, 1532) -> (2473, 2469, 1531).
    // Item 4266, a guard order's walker: units (9362, 9220, 8572) -> (9362, 9250, 8445).
    // Item 4159r14, the Mongol Stable arm of `get_cost` (`MONGOL_STABLE_COST`, a tenth off a Stable unit's base): builds (2473, 2469, 1531) -> (2473, 2469, 1530), leaders (271, 271, 7) -> (271, 271, 5).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 5) -> (271, 271, 4).
    (
        "run825",
        4204,
        [
            (9362, 9250, 8445),
            (2473, 2469, 1529),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (688, 688, 5),
            (2471, 2471, 10),
            (271, 271, 4),
            (58, 58, 0),
            (10, 10, 0),
            (330, 330, 0),
            (9560, 9560, 11),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1201, 1312, 1056) -> (1201, 1312, 1053).
    // Item 4226, a bird walked on neither line: units (1201, 1312, 1053) -> (1201, 1192, 944), guys (403, 445, 78) -> (403, 403, 36).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (180, 180, 0) -> (180, 180, 1).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1201, 1192, 944) -> (1201, 1192, 942).
    // Item 4348, gaia's angles from the start dump: guys (403, 403, 36) -> (403, 403, 0).
    (
        "run839",
        634,
        [
            (1201, 1201, 86),
            (479, 479, 13),
            (0, 0, 0),
            (9, 9, 0),
            (0, 0, 0),
            (526, 526, 0),
            (403, 403, 0),
            (267, 267, 3),
            (6, 6, 0),
            (110, 110, 0),
            (180, 180, 1),
            (8020, 7301, 4558),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1149, 1303, 1028) -> (1149, 1303, 1027).
    // Item 4226, a bird walked on neither line: units (1149, 1303, 1027) -> (1149, 1143, 875), guys (393, 449, 92) -> (393, 393, 36).
    // Item 4320, the score's dirty bits (`0x800000`, `0x1000000`) emitted, leader 0's 0x800007 agreeing: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4348, gaia's angles from the start dump: guys (393, 393, 36) -> (393, 393, 0).
    (
        "run842",
        777,
        [
            (1149, 1149, 88),
            (583, 583, 13),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (532, 532, 0),
            (393, 393, 0),
            (271, 271, 3),
            (9, 9, 0),
            (110, 110, 0),
            (180, 180, 0),
            (8004, 7285, 4510),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1154, 1308, 1028) -> (1154, 1308, 1027).
    // Item 4226, a bird walked on neither line: units (1154, 1308, 1027) -> (1154, 1148, 875), guys (393, 449, 92) -> (393, 393, 36).
    // Item 4320, the score's dirty bits (`0x800000`, `0x1000000`) emitted, leader 0's 0x800007 agreeing: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4348, gaia's angles from the start dump: guys (393, 393, 36) -> (393, 393, 0).
    (
        "run842b",
        776,
        [
            (1154, 1154, 88),
            (583, 583, 13),
            (0, 0, 0),
            (0, 0, 0),
            (2, 2, 0),
            (532, 532, 0),
            (393, 393, 0),
            (271, 271, 3),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2095, 2095, 61)
    // -> (2095, 2095, 15).
    // Item 4226, a bird walked on neither line: units (5770, 5833, 4618) -> (5770, 5593, 4590), guys (1794, 1878, 84) -> (1794, 1794, 0).
    // Item 4228, a path stack's `size` its high water: units (5770, 5593, 4590) -> (5770, 5593, 4579).
    // Item 4229, the herd arm and the caravan's per-leader bit: goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (5770, 5593, 4579) -> (5770, 5602, 4804).
    // Item 4234, a cast's three ranges walked: units (5770, 5602, 4804) -> (5770, 5605, 4899).
    // Item 4244, a mine's `mtn`: builds (2095, 2095, 15) -> (2095, 2095, 13).
    // Item 4257, the garrison chain's order: builds (2095, 2095, 13) -> (2095, 2095, 11).
    // Item 4266, a guard order's walker: units (5770, 5605, 4899) -> (5770, 5770, 21).
    (
        "run854",
        3301,
        [
            (5770, 5770, 21),
            (2095, 2095, 11),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (640, 640, 7),
            (1794, 1794, 0),
            (271, 271, 5),
            (36, 36, 0),
            (5, 5, 0),
            (330, 330, 0),
            (9344, 9344, 2),
        ],
    ),
    // Item 4208, the whales' piece: units (10342, 10233, 9575) -> (10342, 10233, 9554); builds (2288,
    // 2283, 1566) -> (2288, 2283, 1562); groups (640, 640, 6) -> (640, 640, 5); guys (2210, 2294, 185)
    // -> (2210, 2294, 85); cities (64, 64, 4) -> (64, 64, 1); world (9534, 7947, 6461) -> (9534, 9534,
    // 17).
    // Item 4209, the box's `clear_down`: units (10342, 10233, 9554) -> (10342, 10233, 9553); items (0,
    // 35, 35) -> (0, 0, 0); world (9534, 9534, 17) -> (9534, 9534, 12).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2288, 2283, 1562)
    // -> (2288, 2283, 1536).
    // Item 4226, a bird walked on neither line: units (10342, 10233, 9553) -> (10342, 9993, 9577), guys (2210, 2294, 85) -> (2210, 2210, 1).
    // Item 4228, a path stack's `size` its high water: units (10342, 9993, 9577) -> (10342, 9993, 9567).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (64, 64, 1) -> (64, 64, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (10342, 9993, 9567) -> (10342, 10017, 9652).
    // Item 4234, a cast's three ranges walked: units (10342, 10017, 9652) -> (10342, 10056, 9510).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (10342, 10056, 9510) -> (10342, 10179, 7746).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (640, 640, 5) -> (640, 640, 4).
    // Item 4257, the garrison chain's order: builds (2288, 2283, 1536) -> (2288, 2283, 1535).
    // Item 4258, `Group::add`'s OR of its new member's role: groups (640, 640, 4) -> (640, 640, 3).
    // Item 4266, a guard order's walker: units (10342, 10179, 7746) -> (10342, 10188, 7833).
    (
        "run865",
        4859,
        [
            (10342, 10188, 7833),
            (2288, 2283, 1534),
            (0, 0, 0),
            (0, 0, 0),
            (28, 28, 7),
            (640, 640, 3),
            (2210, 2210, 1),
            (271, 271, 5),
            (64, 64, 0),
            (0, 0, 0),
            (330, 330, 0),
            (9534, 9534, 12),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1636, 1776, 294) -> (1636, 1776, 263).
    // Item 4226, a bird walked on neither line: units (1636, 1776, 263) -> (1636, 1636, 123), guys (578, 627, 85) -> (578, 578, 36).
    // Item 4228, a path stack's `size` its high water: units (1636, 1636, 123) -> (1636, 1636, 122).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 4) -> (271, 271, 3).
    // Item 4348, gaia's angles from the start dump: units (1636, 1636, 122) -> (1636, 1636, 86), guys (578, 578, 36) -> (578, 578, 0).
    (
        "run870",
        901,
        [
            (1636, 1636, 86),
            (517, 517, 13),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (578, 578, 0),
            (271, 271, 3),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2737, 2732, 1946)
    // -> (2737, 2732, 1918).
    // Item 4226, a bird walked on neither line: units (11125, 10562, 9823) -> (11125, 10302, 9875), guys (2832, 2923, 124) -> (2832, 2832, 33).
    // Item 4228, a path stack's `size` its high water: units (11125, 10302, 9875) -> (11125, 10302, 9851).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (64, 64, 4) -> (64, 64, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (11125, 10302, 9851) -> (11125, 10326, 9965).
    // Item 4234, a cast's three ranges walked: units (11125, 10326, 9965) -> (11125, 10656, 9933).
    // Item 4159's third residue: units (11125, 10656, 9933) -> (11125, 10656, 9930), the bleed reading `anti_att` (measured on the tree merged with item 4234's).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (11125, 10656, 9930) -> (11125, 10662, 9730).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (2832, 2832, 33) -> (2832, 2832, 26).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (616, 616, 10) -> (616, 616, 7).
    // Item 4257, the garrison chain's order: builds (2737, 2732, 1918) -> (2737, 2732, 1917).
    // Item 4266, a guard order's walker: units (11125, 10662, 9730) -> (11125, 10809, 9782).
    (
        "run882b",
        6808,
        [
            (11125, 10815, 9506),
            (2737, 2732, 1916),
            (0, 0, 0),
            (15, 15, 5),
            (0, 0, 0),
            (616, 616, 7),
            (2832, 2832, 26),
            (271, 271, 5),
            (64, 64, 0),
            (0, 0, 0),
            (330, 330, 0),
            (9518, 9518, 12),
        ],
    ),
    // Item 4208, the whales' piece: units (11180, 10557, 9936) -> (11180, 10557, 9912); builds (2737,
    // 2732, 1951) -> (2737, 2732, 1947); guys (2832, 2923, 222) -> (2832, 2923, 122); cities (64, 64,
    // 7) -> (64, 64, 4); world (9536, 7949, 6589) -> (9536, 9536, 18).
    // Item 4209, the box's `clear_down`: items (0, 35, 35) -> (0, 0, 0); world (9536, 9536, 18) ->
    // (9536, 9536, 12).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2737, 2732, 1947)
    // -> (2737, 2732, 1919).
    // Item 4226, a bird walked on neither line: units (11180, 10557, 9912) -> (11180, 10297, 9964), guys (2832, 2923, 122) -> (2832, 2832, 31).
    // Item 4228, a path stack's `size` its high water: units (11180, 10297, 9964) -> (11180, 10297, 9942).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (64, 64, 4) -> (64, 64, 0), goods (330, 330, 2) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (11180, 10297, 9942) -> (11180, 10321, 10105).
    // Item 4234, a cast's three ranges walked: units (11180, 10321, 10105) -> (11180, 10654, 10052).
    // Item 4159's third residue: units (11180, 10654, 10052) -> (11180, 10654, 10050), the bleed reading `anti_att` (measured on the tree merged with item 4234's).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (11180, 10654, 10050) -> (11180, 10660, 9041).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (11180, 10660, 9041) -> (11180, 10660, 9040).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (2832, 2832, 31) -> (2832, 2832, 24).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (616, 616, 10) -> (616, 616, 7).
    // Item 4257, the garrison chain's order: builds (2737, 2732, 1919) -> (2737, 2732, 1918).
    // Item 4266, a guard order's walker: units (11180, 10660, 9040) -> (11180, 10807, 10077).
    (
        "run883b",
        6821,
        [
            (11180, 10834, 9394),
            (2737, 2732, 1917),
            (0, 0, 0),
            (36, 36, 5),
            (0, 0, 0),
            (616, 616, 7),
            (2832, 2832, 24),
            (271, 271, 5),
            (64, 64, 0),
            (0, 0, 0),
            (330, 330, 0),
            (9536, 9536, 12),
        ],
    ),
    // Item 4208, the whales' piece: units (2652, 2987, 625) -> (2652, 2987, 343); builds (709, 709,
    // 20) -> (709, 709, 7); guys (900, 1012, 212) -> (900, 1012, 112); world (8966, 7379, 5253) ->
    // (8966, 8966, 1).
    // Item 4209, the box's `clear_down`: items (30, 35, 11) -> (30, 30, 0); world (8966, 8966, 1) ->
    // (8966, 8966, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (2652, 2987, 343) -> (2652, 2987, 341).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (709, 709, 7) ->
    // (709, 709, 0).
    // Item 4226, a bird walked on neither line: units (2652, 2987, 341) -> (2652, 2652, 6), guys (900, 1012, 112) -> (900, 900, 0).
    // Item 4228, a path stack's `size` its high water: units (2652, 2652, 6) -> (2652, 2652, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 1).
    (
        "run893b",
        3975,
        [
            (2652, 2652, 0),
            (709, 709, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (900, 900, 0),
            (271, 271, 1),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1508, 1508, 26)
    // -> (1508, 1508, 5).
    // Item 4226, a bird walked on neither line: units (4112, 4387, 3620) -> (4112, 4067, 3348), guys (1352, 1464, 117) -> (1352, 1352, 5).
    // Item 4228, a path stack's `size` its high water: units (4112, 4067, 3348) -> (4112, 4067, 3338).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 3) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4112, 4067, 3338) -> (4112, 4076, 2883).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (4112, 4076, 2883) -> (4112, 4076, 2882).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (4112, 4076, 2882) -> (4112, 4091, 2837).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (4112, 4091, 2837) -> (4112, 4091, 2835).
    // Item 4244, a mine's `mtn`: builds (1508, 1508, 5) -> (1508, 1508, 2).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (1352, 1352, 5) -> (1352, 1352, 3).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (568, 568, 5) -> (568, 568, 4).
    // Item 4257, the garrison chain's order: units (4112, 4091, 2835) -> (4112, 4091, 2823); builds (1508, 1508, 2) -> (1508, 1508, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run912e",
        12753,
        [
            (4112, 4112, 18),
            (1508, 1508, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (568, 568, 4),
            (1352, 1352, 3),
            (271, 271, 1),
            (27, 27, 0),
            (5, 5, 0),
            (330, 330, 0),
            (9122, 9122, 0),
        ],
    ),
    // Item 4218, the idle latch (`flags & 0x08`): units (1118, 1235, 925) -> (1118, 1235, 924).
    // Item 4226, a bird walked on neither line: units (1118, 1235, 924) -> (1118, 1115, 808), guys (375, 417, 78) -> (375, 375, 36).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1118, 1115, 808) -> (1118, 1115, 807).
    // Item 4348, gaia's angles from the start dump: guys (375, 375, 36) -> (375, 375, 0).
    (
        "run938b",
        630,
        [
            (1118, 1118, 87),
            (479, 479, 13),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (526, 526, 0),
            (375, 375, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1580, 1580, 21)
    // -> (1580, 1580, 4).
    // Item 4226, a bird walked on neither line: units (2726, 3048, 2259) -> (2726, 2648, 1940), guys (819, 959, 140) -> (819, 819, 0).
    // Item 4228, a path stack's `size` its high water: units (2726, 2648, 1940) -> (2726, 2648, 1928).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2726, 2648, 1928) -> (2726, 2657, 1897).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (2726, 2657, 1897) -> (2726, 2657, 1896).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2726, 2657, 1896) -> (2726, 2657, 1895).
    // Item 4244, a mine's `mtn`: builds (1580, 1580, 4) -> (1580, 1580, 2).
    // Item 4257, the garrison chain's order: builds (1580, 1580, 2) -> (1580, 1580, 0).
    // Item 4266, a guard order's walker: units (2726, 2657, 1895) -> (2726, 2723, 1530).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run943c",
        10674,
        [
            (2726, 2726, 0),
            (1580, 1580, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (819, 819, 0),
            (271, 271, 1),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1580, 1580, 20)
    // -> (1580, 1580, 5).
    // Item 4226, a bird walked on neither line: units (2838, 3119, 2276) -> (2838, 2709, 2009), guys (826, 966, 140) -> (826, 826, 0).
    // Item 4228, a path stack's `size` its high water: units (2838, 2709, 2009) -> (2838, 2709, 1995).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 1) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2838, 2709, 1995) -> (2838, 2718, 1493).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (2838, 2718, 1493) -> (2838, 2718, 1492).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2838, 2718, 1492) -> (2838, 2718, 1490).
    // Item 4244, a mine's `mtn`: builds (1580, 1580, 5) -> (1580, 1580, 3).
    // Item 4257, the garrison chain's order: builds (1580, 1580, 3) -> (1580, 1580, 1).
    // Item 4266, a guard order's walker: units (2838, 2718, 1490) -> (2838, 2784, 1839).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run946",
        10842,
        [
            (2838, 2838, 2),
            (1580, 1580, 0),
            (0, 0, 0),
            (15, 15, 0),
            (0, 0, 0),
            (532, 532, 0),
            (826, 826, 0),
            (271, 271, 1),
            (27, 27, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1598, 1598, 24)
    // -> (1598, 1598, 4).
    // Item 4226, a bird walked on neither line: units (2832, 3140, 2399) -> (2832, 2730, 2097), guys (875, 1015, 144) -> (875, 875, 4).
    // Item 4228, a path stack's `size` its high water: units (2832, 2730, 2097) -> (2832, 2730, 2084).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (27, 27, 3) -> (27, 27, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2832, 2730, 2084) -> (2832, 2739, 2054).
    // Item 4234, a cast's three ranges walked: units (2832, 2739, 2054) -> (2832, 2742, 2099).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (2832, 2742, 2099) -> (2832, 2742, 2098).
    // Item 4244, a mine's `mtn`: builds (1598, 1598, 4) -> (1598, 1598, 2).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (875, 875, 4) -> (875, 875, 0).
    // Item 4257, the garrison chain's order: builds (1598, 1598, 2) -> (1598, 1598, 0).
    // Item 4266, a guard order's walker: units (2832, 2742, 2098) -> (2832, 2808, 2022).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4332, `Unit::think`'s citizen bit `0x80000` emitted: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run954b",
        12155,
        [
            (2832, 2832, 2),
            (1598, 1598, 0),
            (0, 0, 0),
            (9, 9, 0),
            (2, 2, 0),
            (532, 532, 0),
            (875, 875, 0),
            (271, 271, 0),
            (27, 27, 0),
            (35, 35, 0),
            (180, 180, 0),
            (8096, 8096, 0),
        ],
    ),
    // Item 4205, the built start: units (490, 550, 108) -> (490, 550, 62); builds (455, 455,
    // 15) -> (455, 455, 2); guys (160, 181, 23) -> (160, 181, 21); world (7291, 7275, 3748) ->
    // (7291, 7291, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (490, 550, 62) -> (490, 550, 61).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (455, 455, 2) ->
    // (455, 455, 0).
    // Item 4226, a bird walked on neither line: units (490, 550, 61) -> (490, 490, 1), guys (160, 181, 21) -> (160, 160, 0).
    // Item 4228, a path stack's `size` its high water: units (490, 490, 1) -> (490, 490, 0).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (267, 267, 3) -> (267, 267, 1).
    (
        "run958b",
        375,
        [
            (490, 490, 0),
            (455, 455, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (160, 160, 0),
            (267, 267, 1),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (1832, 1832, 35)
    // -> (1832, 1832, 4).
    // Item 4226, a bird walked on neither line: units (4466, 4844, 3924) -> (4466, 4394, 3554), guys (1514, 1668, 168) -> (1514, 1514, 14).
    // Item 4228, a path stack's `size` its high water: units (4466, 4394, 3554) -> (4466, 4394, 3543).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (17, 17, 1) -> (17, 17, 0), goods (330, 330, 1) -> (330, 330, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (4466, 4394, 3543) -> (4466, 4397, 3129).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (4466, 4397, 3129) -> (4466, 4397, 3128).
    // Item 4244, a mine's `mtn`: builds (1832, 1832, 4) -> (1832, 1832, 2).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: units (4466, 4397, 3128) -> (4466, 4397, 3114); guys (1514, 1514, 14) -> (1514, 1514, 0).
    // Item 4255, `equals_group`'s `get_num` on an emptied last slot: groups (562, 562, 5) -> (562, 562, 4).
    // Item 4257, the garrison chain's order: units (4466, 4397, 3114) -> (4466, 4397, 3090); builds (1832, 1832, 2) -> (1832, 1832, 0).
    // Item 4266, a guard order's walker: units (4466, 4397, 3090) -> (4466, 4454, 2964).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 3) -> (271, 271, 2).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 2) -> (271, 271, 1).
    (
        "run959",
        18368,
        [
            (4466, 4466, 2),
            (1832, 1832, 0),
            (0, 0, 0),
            (9, 9, 0),
            (0, 0, 0),
            (562, 562, 4),
            (1514, 1514, 0),
            (271, 271, 1),
            (17, 17, 0),
            (0, 0, 0),
            (330, 330, 0),
            (9056, 9056, 0),
        ],
    ),
    // Item 4205, the built start: units (3135, 3263, 2550) -> (3135, 3263, 2517); builds (2052,
    // 2052, 46) -> (2052, 2052, 33); cities (35, 35, 4) -> (35, 35, 3); world (7513, 7497,
    // 4325) -> (7513, 7513, 9).
    // Item 4209, the box's `clear_down`: items (60, 110, 69) -> (60, 60, 0); world (7513, 7513, 9) ->
    // (7513, 7513, 0).
    // Item 4218, the idle latch (`flags & 0x08`): units (3135, 3263, 2517) -> (3135, 3263, 2518).
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2052, 2052, 33)
    // -> (2052, 2052, 5).
    // Item 4226, a bird walked on neither line: units (3135, 3263, 2518) -> (3135, 2958, 2405), guys (941, 1046, 106) -> (941, 941, 1).
    // Item 4228, a path stack's `size` its high water: units (3135, 2958, 2405) -> (3135, 2958, 2399).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (35, 35, 3) -> (35, 35, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3135, 2958, 2399) -> (3135, 2970, 1737).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3135, 2970, 1737) -> (3135, 2970, 1734).
    // Item 4244, a mine's `mtn`: builds (2052, 2052, 5) -> (2052, 2052, 3).
    // Item 4257, the garrison chain's order: units (3135, 2970, 1734) -> (3135, 2970, 1716); builds (2052, 2052, 3) -> (2052, 2052, 1).
    // Item 4266, a guard order's walker: units (3135, 2970, 1716) -> (3135, 3126, 1560).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4328, a packed merchant out of `reg_unpack_merch`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run973g",
        12286,
        [
            (3135, 3135, 4),
            (2052, 2052, 0),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (532, 532, 0),
            (941, 941, 1),
            (271, 271, 0),
            (35, 35, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (789, 789, 7) ->
    // (789, 789, 0).
    // Item 4226, a bird walked on neither line: units (2003, 2230, 1722) -> (2003, 1910, 1500), guys (533, 638, 105) -> (533, 533, 0).
    // Item 4228, a path stack's `size` its high water: units (2003, 1910, 1500) -> (2003, 1910, 1495).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (17, 17, 1) -> (17, 17, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (2003, 1910, 1495) -> (2003, 1913, 1531).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (2003, 1913, 1531) -> (2003, 1958, 1484).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (2003, 1958, 1484) -> (2003, 1958, 1482).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 0).
    (
        "run984b",
        6011,
        [
            (2003, 2003, 0),
            (789, 789, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (538, 538, 0),
            (533, 533, 0),
            (271, 271, 0),
            (17, 17, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (827, 827, 8) ->
    // (827, 827, 0).
    // Item 4226, a bird walked on neither line: units (1903, 2165, 1593) -> (1903, 1855, 1335), guys (540, 645, 105) -> (540, 540, 0).
    // Item 4228, a path stack's `size` its high water: units (1903, 1855, 1335) -> (1903, 1855, 1329).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (17, 17, 1) -> (17, 17, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (1903, 1855, 1329) -> (1903, 1858, 1372).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (1903, 1858, 1372) -> (1903, 1858, 1369).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 0).
    (
        "run984c",
        6251,
        [
            (1903, 1903, 0),
            (827, 827, 0),
            (0, 0, 0),
            (3, 3, 0),
            (0, 0, 0),
            (538, 538, 0),
            (540, 540, 0),
            (271, 271, 0),
            (17, 17, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2783, 2783, 43)
    // -> (2783, 2783, 7).
    // Item 4226, a bird walked on neither line: units (3289, 3742, 3042) -> (3289, 3103, 2611), guys (1012, 1222, 211) -> (1012, 1012, 1).
    // Item 4228, a path stack's `size` its high water: units (3289, 3103, 2611) -> (3289, 3103, 2604).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (33, 33, 3) -> (33, 33, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3289, 3103, 2604) -> (3289, 3112, 2189).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3289, 3112, 2189) -> (3289, 3112, 2188).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3289, 3112, 2188) -> (3289, 3112, 2186).
    // Item 4244, a mine's `mtn`: builds (2783, 2783, 7) -> (2783, 2783, 4).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (1012, 1012, 1) -> (1012, 1012, 0).
    // Item 4257, the garrison chain's order: units (3289, 3112, 2186) -> (3289, 3112, 2173); builds (2783, 2783, 4) -> (2783, 2783, 2).
    // Item 4266, a guard order's walker: units (3289, 3112, 2173) -> (3289, 3214, 2139).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run985b",
        16762,
        [
            (3289, 3289, 1),
            (2783, 2783, 2),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (544, 544, 1),
            (1012, 1012, 0),
            (271, 271, 0),
            (33, 33, 0),
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
    // Item 4222, the building push's normalize and the queue's vacated slots: builds (2006, 2006, 32)
    // -> (2006, 2006, 4).
    // Item 4226, a bird walked on neither line: units (3178, 3504, 2735) -> (3178, 2989, 2426), guys (961, 1136, 179) -> (961, 961, 4).
    // Item 4228, a path stack's `size` its high water: units (3178, 2989, 2426) -> (3178, 2989, 2415).
    // Item 4229, the herd arm and the caravan's per-leader bit: cities (30, 30, 3) -> (30, 30, 0).
    // Item 4230, a trade order walked and a road stack's doubled size: units (3178, 2989, 2415) -> (3178, 2998, 2141).
    // Item 4234, a cast's three ranges walked: units (3178, 2998, 2141) -> (3178, 3004, 2150).
    // Item 4234, a caravan's `unit_masks & 0x200`: units (3178, 3004, 2150) -> (3178, 3004, 2149).
    // Item 4243, a grouped move's `GroupOrder` tail walked: units (3178, 3004, 2149) -> (3178, 3010, 2143).
    // Item 4251, `unit_masks` 0x4 (`in_danger`) emitted: units (3178, 3010, 2143) -> (3178, 3010, 2142).
    // Item 4244, a mine's `mtn`: builds (2006, 2006, 4) -> (2006, 2006, 2).
    // Item 4253, the scholar's seating `set_angle(0, _, 1)`: guys (961, 961, 4) -> (961, 961, 0).
    // Item 4257, the garrison chain's order: units (3178, 3010, 2142) -> (3178, 3010, 2130); builds (2006, 2006, 2) -> (2006, 2006, 0).
    // Item 4266, a guard order's walker: units (3178, 3010, 2130) -> (3178, 3169, 1151).
    // Item 4298, `hire_stamp`, `hire_who` and `tributes` on the line: leaders (271, 271, 2) -> (271, 271, 1).
    // Item 4324, `resources_controlled` counted off `rare`: leaders (271, 271, 1) -> (271, 271, 0).
    (
        "run991d",
        15256,
        [
            (3178, 3178, 1),
            (2006, 2006, 0),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (532, 532, 0),
            (961, 961, 0),
            (271, 271, 0),
            (30, 30, 0),
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
    ("run1161", "flags.leader_flags"),
    (
        "run1164",
        "encrypt.bucket[2] encrypt.bucket[5] explored flags.leader_flags",
    ),
    ("run1165", "flags.leader_flags"),
    (
        "run1166",
        // Item 4159's twelfth residue: `collected`, `encrypt.bucket[0]`, `[1]`, `encrypt.leftover[0]`
        // and `escrow` agree, the CEO's tile step having dirtied the economy on theirs' frame.
        "explored flags.leader_flags",
    ),
    ("run1169", "flags.leader_flags"),
    ("run1185", "flags.leader_flags"),
    ("run1217", "flags.leader_flags"),
    ("run1221", "flags.leader_flags"),
    ("run1224b", "flags.leader_flags"),
    ("run1235b", "flags.leader_flags"),
    ("run1236", "flags.leader_flags"),
    ("run1238c", "flags.leader_flags"),
    ("run1247", "flags.leader_flags"),
    ("run1263", "flags.leader_flags"),
    ("run1264b", "flags.leader_flags"),
    ("run1266c", "flags.leader_flags"),
    ("run1293", "flags.leader_flags"),
    ("run1294", "flags.leader_flags"),
    ("run1296", "flags.leader_flags"),
    ("run1304", "flags.leader_flags"),
    ("run1320b", "encrypt.bucket[2] flags.leader_flags"),
    ("run1324c", "flags.leader_flags"),
    ("run1325c", "flags.leader_flags"),
    ("run1326", "flags.leader_flags"),
    ("run1327", "flags.leader_flags"),
    ("run1332a", "flags.leader_flags"),
    ("run1351c", "flags.leader_flags"),
    (
        "run1359",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags",
    ),
    (
        "run1359b",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags",
    ),
    (
        "run1359c",
        "encrypt.bucket[0] encrypt.bucket[1] flags.leader_flags",
    ),
    ("run1361", "flags.leader_flags"),
    ("run1382", "flags.leader_flags"),
    ("run1385p", "flags.leader_flags"),
    ("run1391", "flags.leader_flags"),
    ("run1416", "flags.leader_flags"),
    ("run1417", "flags.leader_flags"),
    ("run1419b", "flags.leader_flags"),
    ("run1420", "flags.leader_flags"),
    ("run1424", "flags.leader_flags"),
    ("run1426b", "explored flags.leader_flags"),
    (
        "run1437",
        "encrypt.bucket[2] encrypt.bucket[5] flags.leader_flags",
    ),
    ("run1438", "aggression cities_built flags.leader_flags"),
    ("run1440", "aggression cities_built flags.leader_flags"),
    ("run1441", "flags.leader_flags"),
    ("run1451", "aggression cities_built flags.leader_flags"),
    ("run1457", "aggression cities_built"),
    ("run1459", "aggression cities_built"),
    ("run1460", "aggression cities_built"),
    (
        "run1469",
        "aggression cities_built flags.leader_flags gather_stamp",
    ),
    (
        "run1469b",
        "aggression cities_built flags.leader_flags gather_stamp",
    ),
    ("run1472", "aggression cities_built flags.leader_flags"),
    // Item 4159's residue: `active` and `num_units` agree on run1484, and
    // `num_units` on run1487 and run1496: the scripted Transport Freighter
    // is in neither count (`num_units[272]` ours 1 theirs 0, each computer).
    ("run1484", "aggression cities_built flags.leader_flags"),
    ("run1487", "aggression cities_built flags.leader_flags"),
    ("run1496", "aggression cities_built flags.leader_flags"),
    ("run1559b", "flags.leader_flags"),
    ("run1564", "flags.leader_flags"),
    ("run1570", "flags.leader_flags"),
    (
        "run785",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run789",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run795",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run810",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run823",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run825",
        "flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    ("run839", "aggression cities_built flags.leader_flags"),
    ("run842", "aggression cities_built flags.leader_flags"),
    ("run842b", "aggression cities_built flags.leader_flags"),
    ("run854", "diplos flags.leader_flags oil_patches.elements"),
    (
        "run865",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    ("run870", "aggression cities_built flags.leader_flags"),
    (
        "run882b",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    (
        "run883b",
        "diplos flags.leader_flags oil_patches.elements reg_gatherers",
    ),
    ("run893b", "flags.leader_flags"),
    ("run912e", "flags.leader_flags"),
    ("run938b", "aggression cities_built flags.leader_flags"),
    ("run943c", "flags.leader_flags"),
    ("run946", "flags.leader_flags"),
    ("run958b", "flags.leader_flags"),
    ("run959", "flags.leader_flags"),
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
                "  {n:8} theirs {:5} ours {:5} parted {:5} first {}",
                x.theirs,
                x.ours,
                x.parted,
                x.first
                    .as_ref()
                    .map_or_else(|| "-".into(), ToString::to_string)
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
                "  {n:8} theirs {:5} ours {:5} parted {:5} first {}",
                x.theirs,
                x.ours,
                x.parted,
                x.first
                    .as_ref()
                    .map_or_else(|| "-".into(), ToString::to_string)
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
/// Item 4222, a queue's vacated slots and the tech arm's unqueue after
/// `gain_tech`: builds (1709, 1709, 35) -> (1709, 1709, 27).
/// Step VII-d, the alliance found short: leaders (407, 407, 7) -> (407, 407,
/// 6), the human's `dip.agree` of the French (theirs 0, ours 1 before)
/// agreeing, and `counteroffer` (the French 1) on the line, agreeing.
/// Item 4226, a bird (slot 9) walked on neither line: units (3907, 4015,
/// 3077) -> (3907, 3895, 2971), guys (1289, 1331, 166) -> (1289, 1289, 124).
/// Item 4228, a path stack's `size` its high water: units (3907, 3895, 2971)
/// -> (3907, 3895, 2964).
// Item 4229, the herd arm: goods (405, 405, 1) -> (405, 405, 2), a named regression
// (the units line already parts on 2,964 of 3,907 ranges).
// Item 4244, a mine's `mtn`: builds (1709, 1709, 27) -> (1709, 1709, 26).
// Item 4266, a guard order's walker: units (3907, 3895, 2964) -> (3907, 3907, 408),
// both sides' ranges one count.
/// Item 4348, gaia's angles from the start dump: units (3907, 3907, 408) -> (3907, 3907, 286), guys (1289, 1289, 124) -> (1289, 1289, 2).
const TAKE_PINNED: Pinned = (
    "run1575",
    1090,
    [
        (3907, 3907, 286),
        (1709, 1709, 26),
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
        (556, 556, 2),
        (1289, 1289, 2),
        (407, 407, 6),
        (12, 12, 0),
        (25, 25, 0),
        (405, 405, 2),
        (8981, 7447, 5432),
    ],
);

/// The leaves of run1575's leaders line that part outside
/// [`LEADERS_HELD_OUT`] at 1090, as found: `cities_built` (theirs 1, 1, 2
/// on the three seats, ours 0, 0, 1); `diplos` (slots 3..7 read 1 there, `Leader::init`'s peace at style 0 on
/// every slot, which the take's re-read lobby writes on 0..2 only);
/// `flags.leader_flags` (high bits on every seat). Until step VII-c
/// `dip.treaty` parted too (the alliance asked on 1041, ours −1 with no
/// `action_treaty`); until step VII-d `dip.agree` (the human's record of the
/// French: theirs 0, ours 1 — the French answer to the alliance committed
/// on 1051 is found short and withdraws both agreements, which ours now
/// computes).
const TAKE_LEAF_PARTINGS: &str = "cities_built diplos flags.leader_flags";

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
            eprintln!("  {n}: {} of {} ranges part; first {f}", l.parted, l.theirs);
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

/// run1629's packet at 1190, pinned as found (item 4298): each line's (the
/// stream's ranges, ours, ranges parted), our walk the take's
/// ([`run1629_take_packet`]). The leaders line parts on 6 of 407 ranges,
/// run1575's take packet's count, none of them a hire or a tribute.
/// Item 4348, gaia's angles from the start dump: units (3911, 3911, 401) -> (3911, 3911, 279), guys (1289, 1289, 123) -> (1289, 1289, 1).
const TAKE1629_PINNED: Pinned = (
    "run1629",
    1190,
    [
        (3911, 3911, 279),
        (2015, 2015, 26),
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
        (556, 556, 2),
        (1289, 1289, 1),
        (407, 407, 6),
        (12, 12, 0),
        (25, 25, 0),
        (405, 405, 2),
        (8967, 7433, 5393),
    ],
);

/// The leaves of run1629's leaders line that part outside
/// [`LEADERS_HELD_OUT`] at 1190, as found: run1575's take packet's three
/// ([`TAKE_LEAF_PARTINGS`]) — `cities_built` (theirs 1, 1, 2, ours 0, 0,
/// 1), `diplos` on slots 3..7, `flags.leader_flags`' high bits.
const TAKE1629_LEAF_PARTINGS: &str = "cities_built diplos flags.leader_flags";

/// **The leaves the attack deal's packet releases** (item 4298;
/// `docs/DECISIONS.md` entry 131 §1): each written on 1186 by the French
/// accept our walk computes (the hire, and the human's attack price left in
/// `tributes`), off its seated image on the packet, ours equal
/// ([`hold_witnesses`]).
pub(super) const TAKE1629_RELEASED: &[(&str, &str, i16)] = &[
    // The human: `hire_stamp[2]` 1186, `hire_who[2]` 1 (the French),
    // `tributes[2]` 640.
    ("hire_stamp", "run1629", 0),
    ("hire_who", "run1629", 0),
    ("tributes", "run1629", 0),
    // The French: `hire_stamp[2]` 1186, `hire_who[2]` 0 (the human).
    ("hire_stamp", "run1629", 1),
    ("hire_who", "run1629", 1),
];

/// **run1629's packet at 1190, held by the attack deal's take** (item
/// 4298): the take's walk (`diff::diplomacy_take`: run1482's start at style
/// 0, `attack4298.cmd`, the human's window commands off the record, each
/// tribute's amount from the script) ticked to the packet's frame, its
/// twelve lines against the instrument's stream. Off the scoreboard
/// ([`TAKES_HELD`]). No decision on the way is staged: the French accept of
/// 1186 is their offers arm's, the attack terms computed.
#[test]
fn run1629_take_packet() {
    use crate::diff::diplomacy_take as take;
    let (run, frame, name, length) = TAKES_HELD[1];
    let (Some(diplomacy), Some(taunts)) = (
        take::run1629_diplomacy(),
        take::script_taunts("attack4298.cmd"),
    ) else {
        eprintln!("skipping: no {} or install", take::RUN1629.0);
        return;
    };
    let Some(walked) = walk_packets_with(name, length, &[(run, frame)], &mut |frames, probe| {
        let script = take::coverage_script("attack4298.cmd");
        let (_, mut built) = take::stand_up_sea_style_zero(take::RUN1629, script)?;
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
            eprintln!("  {n}: {} of {} ranges part; first {f}", l.parted, l.theirs);
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
        (TAKE1629_PINNED.0, TAKE1629_PINNED.1, TAKE1629_PINNED.2),
        (w.run, w.frame, pin_of(&w.lines)),
        "run1629: each line's (the stream's ranges, ours, ranges parted)"
    );
    assert_eq!(
        extra_leaves(w, &held),
        TAKE1629_LEAF_PARTINGS,
        "run1629: the leaders line's leaves parting outside LEADERS_HELD_OUT"
    );
    hold_witnesses(w, &held, TAKE1629_RELEASED);
}

/// run1645's packet at 1000, pinned as found (item 4313): each line's (the
/// stream's ranges, ours, ranges parted), our walk the reseated take's
/// ([`run1645_take_packet`]). Walked from the capture's lobby built, with
/// no start dump and no correction: the units, builds, groups, guys and
/// world lines part on 1, 4, 2, 1 and 1 ranges, and the leaders line on 6
/// of 403, each leader's long range, on the three leaves of
/// [`TAKE1645_LEAF_PARTINGS`].
const TAKE1645_PINNED: Pinned = (
    "run1645",
    1000,
    [
        (3750, 3750, 1),
        (2024, 2024, 4),
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
        (556, 556, 2),
        (1261, 1261, 1),
        (403, 403, 6),
        (9, 9, 0),
        (30, 30, 0),
        (405, 405, 0),
        (8981, 8981, 1),
    ],
);

/// The leaves of run1645's leaders line that part outside
/// [`LEADERS_HELD_OUT`] at 1000, as found: `diplos` on slots 3..7 and
/// `flags.leader_flags`' high bits, as on the two take packets before
/// ([`TAKE_LEAF_PARTINGS`]), and **`ally_mask`**, the French theirs 3 ours 1:
/// the alliance of 903 still stands at the packet, and the two bits
/// `Leader::set_diplo@006ec6a0` and `Leader::plan_strategy@006b9620:205`
/// give an allied leader under `ALLY_LOS` or `reveal_map` are not carried
/// (item 4313's journal, parked 4313a). `cities_built`, which parts on both
/// take packets before, agrees here.
const TAKE1645_LEAF_PARTINGS: &str = "ally_mask diplos flags.leader_flags";

/// **The leaf the reseated taunt's packet releases** (item 4313;
/// `docs/DECISIONS.md` entry 131 §1): `incoming_taunt_who`, written on 906
/// by the French `Leader::receive_taunt` from the human seated at who 1
/// (`--human-who 1`), off its seated image (0) on the packet, ours equal
/// ([`hold_witnesses`]).
pub(super) const TAKE1645_RELEASED: &[(&str, &str, i16)] = &[
    // The French (who 0): the ring's first cell holds the sender, 1.
    ("incoming_taunt_who", "run1645", 0),
];

/// **run1645's packet at 1000, held by the reseated taunt's take** (item
/// 4313): the take's walk (`diff::diplomacy_take::stand_up_reseated`: the
/// capture's own lobby built, the human at who 1, `taunt4313.cmd` staged
/// from the console's seat, the chat minted from it) ticked to the
/// packet's frame, its twelve lines against the instrument's stream. Off
/// the scoreboard ([`TAKES_HELD`]). Nothing is staged off the record: the
/// take has no window command, and the French answer of 966 is their
/// drain's.
#[test]
fn run1645_take_packet() {
    use crate::diff::diplomacy_take as take;
    let (run, frame, name, length) = TAKES_HELD[2];
    let Some(taunts) = take::script_taunts_from("taunt4313.cmd", take::RESEATED_HUMAN) else {
        eprintln!("skipping: no install");
        return;
    };
    let Some(walked) = walk_packets_with(name, length, &[(run, frame)], &mut |frames, probe| {
        let script = take::coverage_script("taunt4313.cmd");
        let (_, mut built) = take::stand_up_reseated(take::RUN1645, script)?;
        for &f in frames {
            while built.sim.frame < f {
                take::stage_frame(&mut built, 3, &[], &[], &taunts);
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
            eprintln!("  {n}: {} of {} ranges part; first {f}", l.parted, l.theirs);
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
        (TAKE1645_PINNED.0, TAKE1645_PINNED.1, TAKE1645_PINNED.2),
        (w.run, w.frame, pin_of(&w.lines)),
        "run1645: each line's (the stream's ranges, ours, ranges parted)"
    );
    assert_eq!(
        extra_leaves(w, &held),
        TAKE1645_LEAF_PARTINGS,
        "run1645: the leaders line's leaves parting outside LEADERS_HELD_OUT"
    );
    hold_witnesses(w, &held, TAKE1645_RELEASED);
}
