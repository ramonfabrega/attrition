//! **Step H: the original's fifteen checksums, every frame** (item 4214,
//! `docs/CHECKSUM.md` §24; `docs/DECISIONS.md` entry 132 §7, §10).
//!
//! `CheckSums::check_all@00936560` runs on every turn and prints only in a
//! dump window. A `RON_CHECKSUMS` build of `tools/trace/` writes its
//! fifteen lines every frame ([`crate::trace::checksums`]); this module
//! holds them two ways. First against the record the original printed on
//! the frames a dump window took — the instrument is the record, and on a
//! shifted frame it is not. Then against our walk: the minted game's built
//! start under the pump, our nine emitted lines' Adler on every frame
//! beside the hook's, and per line the first frame it parts. That frame is
//! the score `Checksum: frames`.
//!
//! The pairing is the frame label's (`tools/trace/README.md`): a set the
//! hook tags with trace frame `f` was taken after `Game::do_frame` `f`
//! and its `end_frame`, so it is logger block `f + 1`'s end record, and
//! our walk's state at `sim.frame == f + 1` before that frame's package.
//! The hook's frame `-1` is setup's `check_all`, our frame 0.

use super::minted::{built_start, minted, script};
use crate::checksum::{CHECKSUM_LINES, Record};
use crate::trace::checksums::{Set, per_frame, sets, summary};

/// The 120-frame take: the minted file's playback with the hook and an
/// `@dumpall` on 91 (`docs/RUNS.md` run1577).
const RUN1577: &str = "gamelog-run1577-greatlakes-playback-of-minted-checksums-120.txt";
const RUN1577_TRACE: &str = "rontrace-run1577.log";
/// run1562's dump: the same file and window with a packet, record 3201
/// the instrument reproduces (`docs/CHECKSUM.md` §3).
const RUN1562: &str = "gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt";

/// **The first act's score**: `(record, the hook's trace frame, lines of 15
/// that agree)`, per printed record held against the hook's first set of
/// that frame. 3201 and 3202 are run1562's; run1577 prints both again in
/// its own window. Each record against the hook one frame off is the red
/// half: 3201 against frame 89 and 91, 3202 against 90.
const HOOK_RECORDS: &[(&str, u64, i64, usize)] = &[
    ("run1562", 3201, 90, 15),
    ("run1562", 3201, 89, 9),
    ("run1562", 3201, 91, 10),
    ("run1562", 3202, 91, 15),
    ("run1562", 3202, 90, 10),
    ("run1577", 3201, 90, 15),
    ("run1577", 3202, 91, 15),
];

/// The hook's whole take: `(sets, torn, frames, the first frame, the last,
/// frames with a second set)`. Every frame of 121 has a set; the three
/// with two are the frames before an order's package (29, 59, 89): the
/// read loop's `check_all` before `process_all`, then the turn's own
/// after it.
const HOOK_TAKE: (usize, u32, usize, i64, i64, &[i64]) = (124, 0, 121, -1, 119, &[29, 59, 89]);

fn records(name: &str) -> Option<Vec<Record>> {
    let path = crate::testenv::dump(name)?;
    let file = std::fs::File::open(path).expect("a kept dump opens");
    Some(crate::checksum::records(std::io::BufReader::new(file)).expect("a kept dump reads"))
}

fn hook(name: &str) -> Option<Vec<Set>> {
    let path = crate::testenv::dump(name)?;
    let bytes = std::fs::read(path).expect("a kept trace reads");
    let (written, torn, ok) = summary(&bytes).expect("a RON_CHECKSUMS trace closes with I_CSUM");
    assert!(ok, "the hook's stub passed esp");
    let s = sets(&bytes);
    assert_eq!(s.len(), written as usize, "every set written is read");
    assert_eq!(torn, 0, "no torn set");
    Some(s)
}

/// **The hook is the record**: on the frames a dump window printed, the
/// hook's first set of the frame is the printed record line for line, and
/// one frame off it is not ([`HOOK_RECORDS`]).
#[test]
fn the_hook_s_values_are_the_printed_record_s() {
    let (Some(trace), Some(r1562), Some(r1577)) =
        (hook(RUN1577_TRACE), records(RUN1562), records(RUN1577))
    else {
        eprintln!("skipping: run1577 or run1562 not filed (set RON_GAMELOG_DIR)");
        return;
    };
    let first = per_frame(&trace);
    let frames: std::collections::BTreeMap<i64, usize> =
        trace.iter().fold(Default::default(), |mut m, s| {
            *m.entry(s.frame).or_default() += 1;
            m
        });
    let twice: Vec<i64> = frames
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(f, _)| *f)
        .collect();
    let take = (
        trace.len(),
        0,
        frames.len(),
        *frames.keys().next().unwrap(),
        *frames.keys().last().unwrap(),
        twice.as_slice(),
    );
    assert_eq!(take, HOOK_TAKE, "the hook's take");
    let mut seen = Vec::new();
    for &(run, number, frame, _) in HOOK_RECORDS {
        let recs = if run == "run1562" { &r1562 } else { &r1577 };
        let printed = recs
            .iter()
            .find(|r| r.number == number)
            .and_then(Record::fifteen)
            .expect("the record prints fifteen lines");
        let set = first[&frame];
        let agree = printed
            .iter()
            .zip(set.lines)
            .filter(|(a, b)| **a == *b)
            .count();
        for (i, (p, h)) in printed.iter().zip(set.lines).enumerate() {
            if *p != h {
                eprintln!(
                    "{run} {number} against frame {frame}: {} printed {p}, hook {h}",
                    CHECKSUM_LINES[i].0
                );
            }
        }
        let total = printed.iter().fold(0u32, |a, v| a.wrapping_add(*v));
        if agree == 15 {
            assert_eq!(set.total, total, "the hook's total is the lines' sum");
        }
        seen.push((run, number, frame, agree));
    }
    assert_eq!(seen, HOOK_RECORDS, "the hook against the records");
}

/// The lines our walk emits, by [`CHECKSUM_LINES`] index: walls, ammo and
/// deaths have no emitter, and rules and the two script lines are not the
/// simulation's (`docs/DECISIONS.md` entry 119).
const EMITTED: [usize; 9] = [0, 1, 5, 6, 7, 8, 9, 10, 11];

/// **The score, `Checksum: frames`**, per take: `(trace, frames walked,
/// per emitted line (line, the first of our walk's frames whose Adler parts
/// from the hook's — the take's length when none does — and the frames it
/// parts on inside [`DIAG`]))`. Seen red first against zeros.
const CHECKSUM_FRAMES: &[(&str, i64, [(&str, i64, usize); 9])] = &[(
    RUN1577_TRACE,
    120,
    [
        ("units", 1, 50),
        ("builds", 100, 20),
        ("groups", 61, 5),
        ("guys", 120, 0),
        ("leaders", 0, 120),
        ("cities", 120, 0),
        ("items", 120, 0),
        ("goods", 120, 0),
        ("world", 0, 1),
    ],
)];

/// The frames on which every emitted line is taken and each parting is
/// counted; past them a line is taken only until it first parts, so a
/// 24,000-frame take costs the lines still whole.
const DIAG: i64 = 300;

/// The Adler of each emitted line `want` names, on `sim`, in [`EMITTED`]'s
/// order (`None` where not wanted). The builds line needs the data layer's
/// pieces, so it takes every line through [`packets::our_lines`].
///
/// [`packets::our_lines`]: super::checksum_lines::packets::our_lines
fn our_adlers(
    sim: &sim::Sim,
    pieces: &super::checksum_lines::packets::Pieces,
    want: &[bool; 9],
) -> [Option<u32>; 9] {
    if want[1] {
        let lines = super::checksum_lines::packets::our_lines(sim, pieces);
        let mut out = [None; 9];
        for (k, &i) in EMITTED.iter().enumerate() {
            out[k] = want[k].then(|| lines[i].adler());
        }
        return out;
    }
    let mut out = [None; 9];
    for (k, &i) in EMITTED.iter().enumerate() {
        if !want[k] {
            continue;
        }
        out[k] = Some(
            match i {
                0 => sim.checksum_units(),
                5 => sim.checksum_groups(),
                6 => sim.checksum_guys(),
                7 => sim.checksum_leaders(),
                8 => sim.checksum_cities(),
                9 => sim.checksum_items(),
                10 => sim.checksum_goods(),
                11 => sim.checksum_world(),
                _ => unreachable!("builds goes through our_lines"),
            }
            .adler(),
        );
    }
    out
}

/// One emitted line's walk against a hook: the first frame it parts, both
/// sides' Adler there, and the frames it parts on inside [`DIAG`].
type LineWalk = (&'static str, i64, u32, u32, Vec<i64>);

/// The walk under the pump against a hook's sets, per emitted line.
fn first_partings(trace: &[Set], length: i64) -> Option<Vec<LineWalk>> {
    let (_, mut built, _) = built_start()?;
    let pieces = super::checksum_lines::packets::Pieces::read()?;
    let mut stream = crate::input::Stream::pump(&minted(&script())?);
    let first = per_frame(trace);
    let mut out: Vec<Option<(i64, u32, u32)>> = vec![None; EMITTED.len()];
    let mut parted: Vec<Vec<i64>> = vec![Vec::new(); EMITTED.len()];
    while built.sim.frame < length {
        let f = built.sim.frame;
        let want: [bool; 9] = std::array::from_fn(|k| f < DIAG || out[k].is_none());
        if !want.iter().any(|w| *w) {
            break;
        }
        if let Some(set) = first.get(&(f - 1)) {
            let ours = our_adlers(&built.sim, &pieces, &want);
            for (k, &i) in EMITTED.iter().enumerate() {
                let Some(o) = ours[k] else { continue };
                if o != set.lines[i] {
                    if f < DIAG {
                        parted[k].push(f);
                    }
                    if out[k].is_none() {
                        out[k] = Some((f, o, set.lines[i]));
                    }
                }
            }
        }
        stream.apply(f as i32, &mut built);
        built.tick();
    }
    Some(
        EMITTED
            .iter()
            .zip(out)
            .zip(parted)
            .map(|((&i, o), p)| {
                let (f, a, b) = o.unwrap_or((length, 0, 0));
                (CHECKSUM_LINES[i].0, f, a, b, p)
            })
            .collect(),
    )
}

/// Frames as runs, `a-b` for a run of more than one.
fn spans(frames: &[i64]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < frames.len() {
        let mut j = i;
        while j + 1 < frames.len() && frames[j + 1] == frames[j] + 1 {
            j += 1;
        }
        out.push(if i == j {
            frames[i].to_string()
        } else {
            format!("{}-{}", frames[i], frames[j])
        });
        i = j + 1;
    }
    out.join(" ")
}

/// **Our lines, every frame, against the hook's.** The score: per take
/// and emitted line, the first frame it parts and the frames it parts on
/// inside [`DIAG`] ([`CHECKSUM_FRAMES`]), each first parting printed with
/// both sides' Adler.
#[test]
fn our_lines_part_from_the_hook_s_on_their_first_frame() {
    for &(name, length, pinned) in CHECKSUM_FRAMES {
        let Some(trace) = hook(name) else {
            eprintln!("skipping: {name} not filed (set RON_GAMELOG_DIR)");
            continue;
        };
        let Some(walked) = first_partings(&trace, length) else {
            eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
            return;
        };
        for (line, f, ours, theirs, p) in &walked {
            eprintln!(
                "CHECKSUM FRAMES {name} {line}: first parts on {f} (ours {ours}, theirs {theirs}); \
                 inside {DIAG}: {} frames, {}",
                p.len(),
                spans(p)
            );
        }
        let seen: Vec<(&str, i64, usize)> = walked
            .iter()
            .map(|(l, f, _, _, p)| (*l, *f, p.len()))
            .collect();
        assert_eq!(seen, pinned, "CHECKSUM_FRAMES {name}");
    }
}
