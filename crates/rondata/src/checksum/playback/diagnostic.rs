//! Compare the range bytes actually retained by the original for a scored state.
use crate::checksum::{CHECKSUM_LINES, fields::differing_fields};
use crate::trace::{checksums::Set, ranges};
use sim::checksum::Line;

pub(super) fn verify_record(
    record: &ranges::Record,
    sets: &[Set],
    trace: &[u8],
) -> Result<(), String> {
    let (due, capped, opened) = crate::trace::checksums::ranges_summary(trace)
        .ok_or("range file supplied but the checksum trace has no range summary")?;
    if record.truncated
        || !opened
        || due as usize != sets.len()
        || record.sets.len() > sets.len()
        || (!capped && record.sets.len() != sets.len())
    {
        return Err("incomplete range capture".into());
    }
    for (a, b) in record.sets.iter().zip(sets) {
        if (a.frame, a.ordinal, a.caller, a.calls, a.lines, a.total)
            != (b.frame, b.ordinal, b.caller, b.calls, b.lines, b.total)
        {
            return Err(format!(
                "range capture disagrees with the trace at frame {} ordinal {}",
                b.frame, b.ordinal
            ));
        }
    }
    Ok(())
}

/// Every byte of the parted range that differs, not only the first (item
/// 4436): a wide range, such as the world line's tile masks, can part on
/// a whole footprint, and its first byte names one tile of it. Empty
/// where the range parts on one byte alone.
fn census(ours: &[u8], original: &[u8]) -> String {
    const LISTED: usize = 32;
    let differing: Vec<usize> = (0..ours.len().max(original.len()))
        .filter(|&n| ours.get(n) != original.get(n))
        .collect();
    if differing.len() < 2 {
        return String::new();
    }
    let at = |bytes: &[u8], n: usize| {
        bytes
            .get(n)
            .map_or_else(|| "end".into(), |x| format!("{x:#04x}"))
    };
    let listed: Vec<String> = differing
        .iter()
        .take(LISTED)
        .map(|&n| format!("{n}: {}|{}", at(ours, n), at(original, n)))
        .collect();
    format!(
        "; {} bytes differ (ours|original): {}{}",
        differing.len(),
        listed.join(", "),
        if differing.len() > LISTED {
            ", …"
        } else {
            ""
        }
    )
}

pub(super) fn explain(
    i: usize,
    ours: Option<&Line>,
    hook: &Set,
    ranges: Option<&mut ranges::Record>,
) -> Result<String, String> {
    let name = CHECKSUM_LINES[i].0;
    let state = hook.frame + 1;
    let hint = || {
        let lo = (state - 1).max(0);
        let hi = state + 1;
        format!(
            "capture {name} at state {state} (trace frame {}): --end-frame {hi} --log-window {lo} {} --tracer-def RON_CHECKSUMS --tracer-def RON_RANGES --tracer-def RNG_MASK={:#x}",
            hook.frame,
            hi + 1,
            1 << i
        )
    };
    let Some(record) = ranges.filter(|r| r.keeps(i)) else {
        return Ok(hint());
    };
    let Some(set) = record.first_of(hook.frame).cloned() else {
        return Ok(hint());
    };
    if set.flags & ranges::OVER != 0 {
        return Ok(format!("range buffer overflowed; {}", hint()));
    }
    let original = record.ranges(&set).map_err(|e| e.to_string())?;
    if ranges::adler(&original, i as u8) != hook.lines[i] {
        return Err(format!(
            "original range bytes do not reproduce {name} at state {state}"
        ));
    }
    let theirs: Vec<_> = original
        .iter()
        .filter(|r| usize::from(r.line) == i)
        .collect();
    let Some(ours) = ours else {
        return Ok(format!(
            "our line is unemitted; original has {} ranges",
            theirs.len()
        ));
    };
    for n in 0..ours.ranges.len().max(theirs.len()) {
        let a = ours.ranges.get(n);
        let b = theirs.get(n);
        if a.map(|a| &a.bytes) == b.map(|b| &b.bytes) {
            continue;
        }
        let Some(a) = a else {
            return Ok(format!(
                "range {n} absent in ours, original call site {:#x}, {} bytes",
                theirs[n].site,
                theirs[n].bytes.len()
            ));
        };
        let label = format!("range {n}, record {}, {}", a.record, a.field);
        if std::env::var("RON_PROBE_4614").is_ok() {
            for m in n.saturating_sub(2)..(n + 8) {
                let oa = ours.ranges.get(m);
                let ob = theirs.get(m);
                eprintln!(
                    "PROBE4614R {name} range {m}: ours {:?} {:02x?} | theirs site {:#x} {:02x?}",
                    oa.map(|a| (a.record, a.field)),
                    oa.map(|a| &a.bytes),
                    ob.map_or(0, |b| b.site),
                    ob.map(|b| &b.bytes)
                );
            }
        }
        let Some(b) = b else {
            return Ok(format!(
                "{label}: {} bytes in ours, absent in original",
                a.bytes.len()
            ));
        };
        let byte = a
            .bytes
            .iter()
            .zip(&b.bytes)
            .position(|(x, y)| x != y)
            .unwrap_or(a.bytes.len().min(b.bytes.len()));
        let value = |bytes: &[u8]| {
            bytes
                .get(byte)
                .map_or_else(|| "end".into(), |x| format!("{x:#04x}"))
        };
        let mut detail = format!(
            "{label}, byte {byte}: ours {}, original {}",
            value(&a.bytes),
            value(&b.bytes)
        );
        for (field, ours, original) in differing_fields(name, a.field, &a.bytes, &b.bytes) {
            use std::fmt::Write;
            write!(detail, ", {field} ours {ours}, original {original}").unwrap();
        }
        detail.push_str(&census(&a.bytes, &b.bytes));
        return Ok(detail);
    }
    Err(format!(
        "unequal checksums with identical {name} range bytes at state {state}"
    ))
}
