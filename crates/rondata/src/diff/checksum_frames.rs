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
/// The whole game: the same file and hook to frame 24,000, trace only
/// (`docs/RUNS.md` run1578).
const RUN1578_TRACE: &str = "rontrace-run1578.log";
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
    // The whole game's take is the same trajectory on the frames both hold.
    if let Some(whole) = hook(RUN1578_TRACE) {
        let key = |s: &Set| (s.frame, s.ordinal, s.lines, s.total);
        let short: std::collections::BTreeSet<_> = trace.iter().map(key).collect();
        let common = whole.iter().filter(|s| s.frame <= 119).map(key);
        let (n, equal) = common.fold((0, 0), |(n, e), k| {
            (n + 1, e + usize::from(short.contains(&k)))
        });
        assert_eq!(
            (n, equal),
            (HOOK_TAKE.0, HOOK_TAKE.0),
            "run1578's first sets are run1577's"
        );
    }
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
///
/// Item 4222 moved two on both takes: groups 61 → 2746 (run1577 whole on
/// 120), a building push normalizing `last_group`'s slot as
/// `equals_group` does; builds 100 → 7383 (run1577 whole on 120), a
/// queue's vacated slots walked as the array holds them (100 → 159), a
/// removed last entry's `job_counter` zeroed (159 → 202), and the tech
/// arm's unqueue after `gain_tech` (202 → 7383).
///
/// Item 4226 moved two on run1578: units 129 → 189 and guys 129 → 6537,
/// both lines walking slots 0 to 8 as `check_units@009371d0` and
/// `check_guys@00937430` do, so the two Wild Birds (`TypeIndex` 402, slot
/// 9) the sampling made on 128 are on neither (run1582 on 129: units 1,147
/// ranges against 1,107 and guys 389 against 375 → the stream's counts).
///
/// Item 4229 moved two on run1578, each to whole on all 24,000: goods 3246
/// → 24,000, `Animal::work@005d7330`'s herd arm setting a seen herd's
/// good's `ever_seen` bit (run1584 on 3246: good 19 `ever_seen` ours 0x00,
/// theirs 0x02 → 0x02 on both); cities 4030 → 24,000, `City::new_caravan`'s
/// bit the partner's index in its owner's cities (run1585 on 4030: city 0
/// `traded_with[1]` ours 4, theirs 2 → 2 on both).
type FramesRow = (&'static str, i64, [(&'static str, i64, usize); 9]);
const CHECKSUM_FRAMES: &[FramesRow] = &[
    (
        RUN1577_TRACE,
        120,
        [
            ("units", 120, 0),
            ("builds", 120, 0),
            ("groups", 120, 0),
            ("guys", 120, 0),
            ("leaders", 0, 120),
            ("cities", 120, 0),
            ("items", 120, 0),
            ("goods", 120, 0),
            ("world", 120, 0),
        ],
    ),
    (
        RUN1578_TRACE,
        24000,
        [
            ("units", 189, 111),
            ("builds", 7383, 0),
            ("groups", 2746, 0),
            ("guys", 6537, 0),
            ("leaders", 0, 300),
            ("cities", 24000, 0),
            ("items", 24000, 0),
            ("goods", 24000, 0),
            ("world", 24000, 0),
        ],
    ),
];

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

/// The instrument's streams of the minted game's packets (runs 1579 to
/// 1581), outside git: `<run>/values.tsv` and `<run>/stream.tsv`.
fn minted_packets_dir() -> String {
    std::env::var("RON_MINTED_PACKETS_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/lab-experiments/2026-11-13-item-4214")
    })
}

/// **Where each first parting is**: per packet, `(run, its trace, our
/// frame, per state line that parts (line, the stream's ranges, ours,
/// ranges parted, the first parting as `range record field: ours |
/// theirs`, each side's bytes in hex cut to 32 digits))`. Each packet
/// stands on a frame [`CHECKSUM_FRAMES`] names as a line's first parting;
/// a line that agrees there is not listed.
type PacketRow = (&'static str, usize, usize, usize, &'static str);
pub(in crate::diff) const MINTED_PACKETS: &[(&str, &str, i64, &[PacketRow])] = &[
    (
        "run1579",
        "rontrace-run1579.log",
        1,
        &[(
            "leaders",
            267,
            267,
            2,
            "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|c7000000",
        )],
    ),
    (
        "run1582",
        "rontrace-run1582.log",
        129,
        &[(
            "leaders",
            267,
            267,
            2,
            "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|d9000000",
        )],
    ),
    (
        "run1583",
        "rontrace-run1583.log",
        987,
        &[
            (
                "units",
                1181,
                1181,
                4,
                "177 0 path.size..increment: path.size 10|50",
            ),
            (
                "leaders",
                267,
                267,
                2,
                "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|e7000000",
            ),
        ],
    ),
    (
        "run1584",
        "rontrace-run1584.log",
        3246,
        &[
            (
                "units",
                1389,
                1386,
                661,
                "167 0 path.size..increment: path.size 20|60",
            ),
            (
                "leaders",
                267,
                267,
                2,
                "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|fd000000",
            ),
        ],
    ),
    (
        "run1585",
        "rontrace-run1585.log",
        4030,
        &[
            (
                "units",
                1507,
                1504,
                809,
                "167 0 path.size..increment: path.size 10|60",
            ),
            (
                "leaders",
                267,
                267,
                2,
                "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|05010000",
            ),
        ],
    ),
    (
        "run1580",
        "rontrace-run1580.log",
        61,
        &[(
            "leaders",
            267,
            267,
            2,
            "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|d1000000",
        )],
    ),
    (
        "run1581",
        "rontrace-run1581.log",
        100,
        &[(
            "leaders",
            267,
            267,
            3,
            "1 0 who..ally_mask: +0x10 of 26914|26914: 00000000|db000000",
        )],
    ),
];

/// One line's first parting as [`MINTED_PACKETS`] pins it: `range record
/// field`, then the fields that part as `name ours|theirs` where the line
/// has a decoder, else the first byte that parts and four bytes from it
/// on each side, in hex.
fn first_text(line: &str, f: &(usize, i16, String, String, String)) -> String {
    let bytes = |h: &str| -> Vec<u8> {
        (0..h.len() / 2)
            .filter_map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).ok())
            .collect()
    };
    let (o, t) = (bytes(&f.3), bytes(&f.4));
    let head = format!("{} {} {}", f.0, f.1, f.2);
    let decoded = (o.len() == t.len() && !o.is_empty())
        .then(|| {
            let a = super::checksum_lines::range_fields(line, &f.2, &o)?;
            let b = super::checksum_lines::range_fields(line, &f.2, &t)?;
            Some(
                a.iter()
                    .zip(&b)
                    .filter(|(x, y)| x.1 != y.1)
                    .map(|(x, y)| format!("{} {}|{}", x.0, x.1, y.1))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .filter(|d| !d.is_empty());
    if let Some(d) = decoded {
        return format!("{head}: {}", d.join(", "));
    }
    let at = o
        .iter()
        .zip(&t)
        .position(|(x, y)| x != y)
        .unwrap_or(o.len().min(t.len()));
    let cut = |b: &[u8]| {
        b.iter()
            .skip(at)
            .take(4)
            .map(|x| format!("{x:02x}"))
            .collect::<String>()
    };
    format!(
        "{head}: +{at:#x} of {}|{}: {}|{}",
        o.len(),
        t.len(),
        cut(&o),
        cut(&t)
    )
}

/// **The packets name the partings.** Our walk under the pump to each
/// packet's frame, its twelve lines held range by range against the
/// packet's stream ([`MINTED_PACKETS`]); and the hook's set on the same
/// take is the instrument's values on that frame, 15 of 15, and so is
/// run1578's on the same frame.
#[test]
fn the_minted_packets_name_each_first_parting() {
    use super::checksum_lines::packets::{STATE_LINES, compare, our_lines, packet_stream_in};
    let dir = minted_packets_dir();
    let whole = hook(RUN1578_TRACE);
    let whole = whole.as_deref().map(per_frame);
    let mut moved = Vec::new();
    for &(run, trace, frame, pinned) in MINTED_PACKETS {
        let (Some((logger, theirs)), Some(sets)) = (packet_stream_in(&dir, run), hook(trace))
        else {
            eprintln!("skipping {run}: no stream under {dir} or no {trace}");
            continue;
        };
        assert_eq!(logger, frame, "{run}'s values file names our frame");
        let values = std::fs::read_to_string(format!("{dir}/{run}/values.tsv")).unwrap();
        let instrument = crate::checksum::instrument_values(&values);
        let set = per_frame(&sets)[&(frame - 1)];
        let agree = CHECKSUM_LINES
            .iter()
            .zip(set.lines)
            .filter(|((name, ..), v)| instrument.iter().any(|(n, iv)| n == name && *iv == *v))
            .count();
        assert_eq!(
            agree,
            15,
            "{run}: the hook's set on {} is the instrument's",
            frame - 1
        );
        if let Some(whole) = &whole {
            assert_eq!(
                whole[&(frame - 1)].lines,
                set.lines,
                "run1578's set on {} is {run}'s",
                frame - 1
            );
        }
        let (Some((_, mut built, _)), Some(pieces)) = (
            built_start(),
            super::checksum_lines::packets::Pieces::read(),
        ) else {
            eprintln!("skipping: no install or lobby dump");
            return;
        };
        let mut stream = crate::input::Stream::pump(&minted(&script()).unwrap());
        super::minted::tick_to(&mut built, Some(&mut stream), frame);
        let outcome = compare(&our_lines(&built.sim, &pieces), &theirs);
        let mut seen: Vec<(&str, usize, usize, usize, String)> = Vec::new();
        for (name, o) in STATE_LINES.iter().zip(&outcome) {
            if let Some(f) = &o.first {
                eprintln!(
                    "MINTED PACKET {run} frame {frame} {name}: theirs {} ours {} parted {}; first {} {} {}:\n  ours   {}\n  theirs {}",
                    o.theirs, o.ours, o.parted, f.0, f.1, f.2, f.3, f.4
                );
                seen.push((name, o.theirs, o.ours, o.parted, first_text(name, f)));
            }
        }
        let pinned: Vec<(&str, usize, usize, usize, String)> = pinned
            .iter()
            .map(|&(l, t, o, p, f)| (l, t, o, p, f.to_string()))
            .collect();
        if seen != pinned {
            eprintln!("MINTED_PACKETS {run} moved: {seen:#?}");
            moved.push(run);
        }
    }
    assert!(moved.is_empty(), "MINTED_PACKETS moved on {moved:?}");
}

/// run1579's dump: the minted file's playback with an `@dumpall` on frame
/// 0, whose first record (3201) is printed before frame 0's tick — the
/// hook's frame `-1`, our built start.
const RUN1579: &str = "gamelog-run1579-greatlakes-playback-of-minted-packet-1.txt";

/// **The world on the start, by section** (item 4218): record 3201 of
/// [`RUN1579`]'s section 8, `danger[0..8]`, as `(theirs, ours with every
/// row cleared)`. Theirs is not zero: `Game::run@00584590:721` calls
/// `GameDaemon::calc_danger` once past `build_game`, before the first
/// frame, and the built start runs it (`Setup::calc_danger`). Before item
/// 4218 ours was the zeros' Adler (`0x70800001`, 28,800 zero bytes) and
/// section 8 alone parted.
const WORLD_START_DANGER: (u32, u32) = (2_995_557_468, 1_887_436_801);

/// The world line on the built start, every one of the fifteen sections
/// against record 3201 printed before frame 0's tick (no packet stands
/// there; the record carries each section's own sum). Red half: the same
/// start with every danger row cleared parts on section 8 alone, at the
/// zeros' sum — the state before `Game::run`'s call.
#[test]
fn the_world_is_the_original_s_on_the_start_by_section() {
    let Some(recs) = records(RUN1579) else {
        eprintln!("skipping: {RUN1579} not filed (set RON_GAMELOG_DIR)");
        return;
    };
    let Some((_, built, _)) = built_start() else {
        eprintln!("skipping: no install or lobby dump");
        return;
    };
    assert_eq!(built.sim.frame, 0, "the built start");
    let record = recs.iter().find(|r| r.number == 3201).expect("record 3201");
    let parts = record.world_parts();
    assert_eq!(parts.len(), 16, "sections 0 to 14 and the whole");
    let parted = |sim: &sim::Sim| -> Vec<(usize, u32, u32)> {
        let image = sim.world_image();
        parts
            .iter()
            .take(15)
            .enumerate()
            .filter_map(|(s, theirs)| {
                let ours = image
                    .iter()
                    .filter(|r| usize::from(r.section) == s)
                    .fold(1, |a, r| sim::checksum::adler32(a, &r.bytes()));
                (ours != *theirs).then_some((s, ours, *theirs))
            })
            .collect()
    };
    let whole = built.sim.checksum_world().adler();
    let seen = parted(&built.sim);
    eprintln!(
        "WORLD START whole ours {whole} theirs {}; sections parted {seen:?}",
        parts[15]
    );
    assert_eq!(
        parts[8], WORLD_START_DANGER.0,
        "record 3201's danger section"
    );
    assert!(seen.is_empty(), "the world parts on the start: {seen:?}");
    assert_eq!(whole, parts[15], "the world line on the start");
    let mut cleared = built.sim.clone();
    for who in 0..8u8 {
        if !cleared.world.danger_row(who).is_empty() {
            cleared.world.clear_danger(who);
        }
    }
    assert_eq!(
        parted(&cleared),
        vec![(8, WORLD_START_DANGER.1, WORLD_START_DANGER.0)],
        "the start without Game::run's calc_danger"
    );
}
