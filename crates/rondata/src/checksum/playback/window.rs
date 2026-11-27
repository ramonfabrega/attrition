//! Every range that differs over a window of states (the pass of
//! 2026-11-27).
//!
//! The scorer names one leaf a line: the first range that parts on the
//! line's first parted state. A landing wants the states round it — what
//! parted a tick earlier on another line, whether a record is absent on
//! one side, the whole of one record on both. Each row landing answered
//! that with a probe test cherry-picked from the landing before, pointed
//! at its row by hand and reverted before its gate; sixteen journals name
//! it. This is that printer, kept: [`print_ranges`], and the example
//! `print_ranges` over it.
use crate::checksum::CHECKSUM_LINES;
use crate::checksum::fields::{differing_fields, range_fields};
use crate::checksum::state;
use crate::trace::ranges::{OVER, Record};
use std::path::Path;

/// What to print: the states `lo..=hi`, one record's fields on both
/// sides whenever its line is read (`record`, a line's name and a
/// record), and every field of a parted range, not only the differing
/// ones (`all`).
pub struct Window<'a> {
    pub lo: i32,
    pub hi: i32,
    pub record: Option<(&'a str, i16)>,
    pub all: bool,
}

/// How far a lost pairing looks for four ranges running that agree.
const RESYNC: usize = 40;

/// One step of the pairing: a range on both sides, or one a side holds
/// alone.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Step {
    Both(usize, usize),
    Ours(usize),
    Theirs(usize),
}

/// **The two sides' ranges, paired in fold order.** Equal bytes pair, and
/// so do two ranges of one length past a byte: a record's fields part
/// where its length stands. Where the lengths part, a side holds a range
/// the other does not; the pairing skips up to [`RESYNC`] on one side to
/// where the next four agree, and names what it skipped. With no such
/// place the two are paired as they stand, and the lengths say so.
pub(crate) fn pair(ours: &[&[u8]], theirs: &[&[u8]]) -> Vec<Step> {
    let agree = |a: usize, b: usize| {
        (0..4).all(|t| a + t < ours.len() && b + t < theirs.len() && ours[a + t] == theirs[b + t])
    };
    let mut out = Vec::with_capacity(ours.len().max(theirs.len()));
    let (mut i, mut j) = (0, 0);
    while i < ours.len() && j < theirs.len() {
        let same_shape = ours[i].len() == theirs[j].len() && ours[i].len() > 1;
        if ours[i] == theirs[j] || same_shape {
            out.push(Step::Both(i, j));
            (i, j) = (i + 1, j + 1);
            continue;
        }
        let skip = (1..RESYNC)
            .flat_map(|d| [(d, 0), (0, d)])
            .find(|&(da, db)| agree(i + da, j + db));
        match skip {
            Some((da, db)) => {
                out.extend((0..da).map(|t| Step::Ours(i + t)));
                out.extend((0..db).map(|t| Step::Theirs(j + t)));
                (i, j) = (i + da, j + db);
            }
            None => {
                out.push(Step::Both(i, j));
                (i, j) = (i + 1, j + 1);
            }
        }
    }
    out.extend((i..ours.len()).map(Step::Ours));
    out.extend((j..theirs.len()).map(Step::Theirs));
    out
}

/// The bytes of two ranges that differ, where no decoder names a field.
fn bytes_apart(a: &[u8], b: &[u8]) -> String {
    let at = |s: &[u8], n: usize| {
        s.get(n)
            .map_or_else(|| "end".into(), |x| format!("{x:#04x}"))
    };
    let apart: Vec<String> = (0..a.len().max(b.len()))
        .filter(|&n| a.get(n) != b.get(n))
        .take(32)
        .map(|n| format!("{n}: {}|{}", at(a, n), at(b, n)))
        .collect();
    format!("bytes (ours|original) {}", apart.join(", "))
}

/// **Print every range that differs on the states `w.lo..=w.hi`** of
/// `recording`, walked from its lobby, against the original's ranges
/// record, each line handed to `out`. Returns how many ranges differed,
/// a side's own among them. A line the record does not keep, or that
/// ours does not emit, is said once and not counted.
pub fn print_ranges(
    install: &Path,
    recording: &Path,
    ranges: &Path,
    w: &Window,
    out: &mut dyn FnMut(String),
) -> Result<u32, String> {
    if w.lo < 0 || w.hi < w.lo {
        return Err(format!("states {}..={}: not a window", w.lo, w.hi));
    }
    let rec = crate::recgame::read(&recording.to_string_lossy()).map_err(|e| e.to_string())?;
    let mut record = Record::open(ranges).map_err(|e| format!("{}: {e}", ranges.display()))?;
    let inst = crate::Install::new(install);
    let loaded = crate::load::load(&inst).map_err(|e| e.to_string())?;
    let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
    let rules = inst.rules().map_err(|e| e.to_string())?;
    let tribes = inst.tribe_defs(&rules).map_err(|e| e.to_string())?;
    let mut built = super::recording_start(&loaded, &rec);
    let mut stream = crate::input::Stream::pump(&rec);
    let mut differing = 0u32;
    let mut said = [false; 12];
    for state in w.lo..=w.hi {
        while built.sim.frame < i64::from(state) {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        // The original tags a state's set with the frame that just ran.
        let Some(set) = record.first_of(i64::from(state) - 1).cloned() else {
            out(format!("{state}: the record holds no set"));
            continue;
        };
        if set.flags & OVER != 0 {
            out(format!("{state}: the record's buffer overflowed"));
            continue;
        }
        let all = record.ranges(&set).map_err(|e| e.to_string())?;
        let lines = state::ranges(&built.sim, &look, &tribes);
        for (li, (name, ..)) in CHECKSUM_LINES.iter().take(12).enumerate() {
            let unread = match (&lines[li], record.keeps(li)) {
                (None, _) => Some("ours emits no line"),
                (_, false) => Some("the record keeps no ranges"),
                _ => None,
            };
            if let Some(why) = unread {
                if !std::mem::replace(&mut said[li], true) {
                    out(format!("{name}: {why}"));
                }
                continue;
            }
            let ours = lines[li].as_ref().expect("read above");
            let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
            if ours.ranges.len() != theirs.len() {
                out(format!(
                    "{state} {name}: {} ranges in ours, {} in the original",
                    ours.ranges.len(),
                    theirs.len()
                ));
            }
            let a: Vec<&[u8]> = ours.ranges.iter().map(|r| r.bytes.as_slice()).collect();
            let b: Vec<&[u8]> = theirs.iter().map(|r| r.bytes.as_slice()).collect();
            // A units record opens on `who`; the ranges after it are its.
            let mut who = 0u8;
            for step in pair(&a, &b) {
                match step {
                    Step::Ours(i) => {
                        differing += 1;
                        let r = &ours.ranges[i];
                        out(format!(
                            "{state} {name} range {i}, record {}, {}: {} bytes in ours alone",
                            r.record,
                            r.field,
                            r.bytes.len()
                        ));
                    }
                    Step::Theirs(j) => {
                        differing += 1;
                        let r = theirs[j];
                        out(format!(
                            "{state} {name} original range {j}: {} bytes in the original alone, \
                             call site {:#x}, {:?}",
                            r.bytes.len(),
                            r.site,
                            &r.bytes[..r.bytes.len().min(24)]
                        ));
                    }
                    Step::Both(i, j) => {
                        let (r, t) = (&ours.ranges[i], theirs[j]);
                        if r.field == "who..y_internal" {
                            who = r.bytes[0];
                        }
                        let wanted = w.record == Some((*name, r.record));
                        let parted = r.bytes != t.bytes;
                        if wanted || (parted && w.all) {
                            for (side, bytes) in [("ours", &r.bytes), ("original", &t.bytes)] {
                                out(format!(
                                    "{state} {name} record {}, {}, {side}: {}",
                                    r.record,
                                    r.field,
                                    // A decoder reads ours' layout: bytes of another
                                    // length are the original's own, and stay bytes.
                                    Some(bytes)
                                        .filter(|b| b.len() == r.bytes.len())
                                        .and_then(|b| range_fields(name, r.field, b))
                                        .map_or_else(|| format!("{bytes:?}"), |f| format!("{f:?}"))
                                ));
                            }
                        }
                        if !parted {
                            continue;
                        }
                        differing += 1;
                        let fields = differing_fields(name, r.field, &r.bytes, &t.bytes);
                        let apart = if fields.is_empty() {
                            bytes_apart(&r.bytes, &t.bytes)
                        } else {
                            fields
                                .iter()
                                .map(|(n, x, y)| format!("{n} ours {x}, original {y}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        out(format!(
                            "{state} {name} range {i}, record {}, {}: {apart}",
                            r.record, r.field
                        ));
                        if li == 0
                            && let Some(u) = built.sim.unit_by_o(who, r.record)
                        {
                            let unit = &built.sim.units[u];
                            out(format!(
                                "  who {who}, unit slot {u}, type {}, at ({}, {})",
                                unit.type_index, unit.pos.x, unit.pos.y
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(differing)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The pairing names what a side holds alone and what parts in
    /// place.** Seen red with the resync taken out (the inserted range
    /// then pairs with its neighbour and every later one reads parted).
    #[test]
    fn the_pairing_names_a_side_s_own_range_and_a_parted_one() {
        let r = |k: u8, n: usize| vec![k; n];
        let ours: Vec<Vec<u8>> = (0..8).map(|k| r(k, 4 + k as usize % 3)).collect();
        let same: Vec<&[u8]> = ours.iter().map(Vec::as_slice).collect();
        let whole: Vec<Step> = (0..8).map(|k| Step::Both(k, k)).collect();
        assert_eq!(pair(&same, &same), whole, "equal sides pair one for one");
        // One range parted in place: paired, its bytes for the caller.
        let mut parted = ours.clone();
        parted[3][1] ^= 1;
        let b: Vec<&[u8]> = parted.iter().map(Vec::as_slice).collect();
        assert_eq!(pair(&same, &b), whole);
        // One range the original holds alone, of a length of its own.
        let mut more = ours.clone();
        more.insert(2, vec![9; 11]);
        let b: Vec<&[u8]> = more.iter().map(Vec::as_slice).collect();
        let got = pair(&same, &b);
        assert_eq!(got[2], Step::Theirs(2));
        assert_eq!(got[3], Step::Both(2, 3));
        assert_eq!(got.len(), 9);
        assert_eq!(
            got.iter().filter(|s| matches!(s, Step::Both(..))).count(),
            8
        );
        // And one ours holds alone, at the end, past any resync.
        assert_eq!(
            pair(&b, &same)[2],
            Step::Ours(2),
            "the same range, from the other side"
        );
        let short = &same[..7];
        assert_eq!(pair(&same, short).last(), Some(&Step::Ours(7)));
    }

    /// **The printer reads a state's set as the scorer does**: on the
    /// long window game, whole on every line, no range differs on three
    /// states of the record its army split was taken on. A set read one
    /// frame off parts hundreds (seen red with `state` for `state - 1`).
    #[test]
    fn a_whole_game_prints_no_differing_range() {
        let Some(root) = crate::testenv::install_root() else {
            return;
        };
        let lab = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("ron-data/lab-experiments");
        let game = crate::testenv::experiment(
            &lab.join("2026-11-17-codex-live-recording"),
            "window-long.rcx",
        );
        let ranges = crate::testenv::experiment(
            &lab.join("2026-11-17-codex-window-9211"),
            "map-14/rontrace.ranges",
        );
        let (Some(game), Some(ranges)) = (game, ranges) else {
            return;
        };
        let mut lines = Vec::new();
        let w = Window {
            lo: 9210,
            hi: 9212,
            record: Some(("units", 1)),
            all: false,
        };
        let n = print_ranges(root.as_ref(), &game, &ranges, &w, &mut |l| lines.push(l)).unwrap();
        assert_eq!(n, 0, "{lines:#?}");
        assert!(
            lines
                .iter()
                .any(|l| l.contains("units record 1,") && l.contains("ours")),
            "the asked record is printed on both sides: {lines:#?}"
        );
    }
}
