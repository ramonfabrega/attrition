//! The working agreement's paperwork rules, enforced rather than trusted.
//!
//! `docs/QUEUE.md` says of itself that it *subtracts*: a finished entry
//! leaves for the journal, the handoff is rewritten from scratch and is
//! about twenty lines. Within two days of writing that it was 409 lines,
//! its struck entries had grown paragraphs, and the handoff was seventy
//! lines — because nothing failed when they did. `no_float.rs` is the
//! precedent: a rule that is only prose is a rule a tired afternoon
//! defeats. So this module reads the documents and fails.
//!
//! Three rules, each of which was made to fail before it was landed
//! (2026-08-27):
//!
//! - **The queue deletes; it does not strike.** A finished item's story is
//!   in `docs/JOURNAL.md` under its number; the queue keeps nothing of it,
//!   not even a struck line. The file is bounded, and its handoff section
//!   is bounded harder.
//! - **`CLAUDE.md` carries rules, not findings.** A blind reader inherits
//!   it, so an address or a section number in it contaminates the reading.
//! - **A document over the ceiling may not grow.** The specifications are
//!   the handoff between sessions, and a 190 KB handoff is not one. Every
//!   document is held under [`DOC_CEILING`]; the ones already over it are
//!   pinned at their size in [`OVER`] and may only shrink — the way to add a
//!   correction to one is to move its story out to the journal first. When
//!   one drops under the ceiling, delete its row.

use std::path::PathBuf;

fn docs() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs"))
}

fn read(name: &str) -> String {
    let path = docs().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The queue's whole length, in lines. Twenty for the handoff, a line or
/// two per open item, and the maintenance notes: a backlog that needs more
/// than this is a changelog again.
const QUEUE_LINES: usize = 180;

/// The handoff section, in lines including blanks. The file says "about
/// twenty"; this is the tolerance.
const HANDOFF_LINES: usize = 32;

/// Bytes. A document a fresh session can actually read before starting.
const DOC_CEILING: usize = 60_000;

/// The documents over the ceiling on 2026-08-27, pinned at that day's size.
/// Each may only shrink. `JOURNAL.md` is not here: it is the append-only
/// chronicle and is meant to grow.
const OVER: &[(&str, usize)] = &[
    ("SYNC.md", 67_737),
    ("ARMY.md", 84_781),
    ("COMBAT.md", 107_149),
    ("CITIES.md", 106_854),
    ("GROUPS.md", 127_758),
    ("ORACLE.md", 148_461),
    ("AI.md", 157_530),
    ("ORDERS.md", 191_190),
];

#[test]
fn the_queue_deletes_rather_than_strikes() {
    let q = read("QUEUE.md");
    let struck: Vec<usize> = q
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("~~"))
        .map(|(i, _)| i + 1)
        .collect();
    assert!(
        struck.is_empty(),
        "docs/QUEUE.md strikes entries at lines {struck:?}; delete them — the story is the journal's"
    );
}

#[test]
fn the_queue_is_bounded() {
    let q = read("QUEUE.md");
    let n = q.lines().count();
    assert!(
        n <= QUEUE_LINES,
        "docs/QUEUE.md is {n} lines; the bound is {QUEUE_LINES}. Move finished items and stories to docs/JOURNAL.md"
    );
}

#[test]
fn the_handoff_is_short() {
    let q = read("QUEUE.md");
    let mut in_section = false;
    let mut n = 0;
    for line in q.lines() {
        if line.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = line.starts_with("## Where things stand");
            continue;
        }
        if in_section {
            n += 1;
        }
    }
    assert!(
        n > 0,
        "docs/QUEUE.md has no '## Where things stand' section"
    );
    assert!(
        n <= HANDOFF_LINES,
        "the handoff is {n} lines; the bound is {HANDOFF_LINES}. It is rewritten from scratch, not appended to"
    );
}

#[test]
fn claude_md_carries_no_findings() {
    let text = std::fs::read_to_string(docs().join("../CLAUDE.md")).expect("CLAUDE.md");
    let bad: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            // A function address, an offset, or a section citation is a
            // finding; a blind reader must not inherit one.
            let hex = l.match_indices("0x").any(|(i, _)| {
                l[i + 2..]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .count()
                    >= 4
            });
            // `§` followed by a digit is a citation; the bare sign is prose.
            let cite = l.match_indices('§').any(|(i, _)| {
                l[i + '§'.len_utf8()..]
                    .trim_start()
                    .starts_with(|c: char| c.is_ascii_digit())
            });
            hex || l.contains("@00") || cite
        })
        .map(|(i, l)| (i + 1, l))
        .collect();
    assert!(
        bad.is_empty(),
        "CLAUDE.md names findings a reader is meant to re-derive: {bad:#?}"
    );
}

#[test]
fn a_document_over_the_ceiling_may_not_grow() {
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name == "JOURNAL.md" {
            continue;
        }
        let size = std::fs::metadata(&path).expect("metadata").len() as usize;
        let bound = OVER
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(DOC_CEILING, |&(_, pinned)| pinned);
        if size > bound {
            failures.push(format!(
                "docs/{name}: {size} bytes, bound {bound} — move its story to the journal, keep the specification"
            ));
        }
    }
    for (name, pinned) in OVER {
        let size = std::fs::metadata(docs().join(name)).map_or(0, |m| m.len() as usize);
        if size <= DOC_CEILING {
            failures.push(format!(
                "docs/{name} is under the ceiling now ({size} ≤ {DOC_CEILING}); delete its row from OVER"
            ));
        } else if size < *pinned {
            // Fine — but say so, so the next session lowers the pin.
            eprintln!("docs/{name} shrank to {size}; lower its pin from {pinned}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
