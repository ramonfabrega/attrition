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
//! - **Every `name@00xxxxxx` a specification cites is that function.**
//!   Added 2026-08-31 (queue item 89b, item 72's manual sweep made a
//!   check): the address is looked up in the decompile export's
//!   `INDEX.tsv` and the exported name must end with the cited one — the
//!   class where a name sits on a different function's address has cost a
//!   wrong conclusion more than once ("36 twice over"). Skips loudly on a
//!   machine without the export; `JOURNAL.md` and `docs/audit/` are not
//!   read, because both tell stories *about* wrong citations.
//! - **A section over the ceiling may not grow.** The specifications are
//!   the handoff between sessions, and the unit a session reads is the
//!   `## ` section, not the file. Every section is held under
//!   [`SECTION_CEILING`]; the ones already over it are pinned at their size
//!   in [`OVER`] and may only shrink — the way to add a correction to one is
//!   to split it, or to move its story out to the journal first. When one
//!   drops under the ceiling, delete its row. (Until 2026-08-30 the unit was
//!   the file, at 60 KB; the journal entry of that day says why it moved.)

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
/// than this is a changelog again. Raised 180 → 200 on 2026-08-31: the
/// board holds ~20 live items whose booked shape is five or six lines
/// each, so 180 had sessions golfing lines instead of deleting stories —
/// the bound should bite on bloat, not on the working agreement's own
/// item size.
const QUEUE_LINES: usize = 200;

/// The handoff section, in lines including blanks. The file says "about
/// twenty"; this is the tolerance.
const HANDOFF_LINES: usize = 32;

/// Bytes, per `## ` section. The unit is the section because that is what
/// a session reads: an item names §6.4 and §4, never the file. A whole-file
/// ceiling (60 KB, 2026-08-27 to 08-30) taxed whoever added a finding to
/// any section of a large file, and what got cut under a 254-byte margin
/// was whatever the session personally needed least — once, nearly the
/// evidence (`docs/JOURNAL.md`, 2026-08-30, "what the byte pins are actually
/// doing"). Sixteen thousand is a section a session reads whole before
/// starting; the text before the first `## ` counts as a section too.
const SECTION_CEILING: usize = 16_000;

/// The sections over the ceiling on 2026-08-30, pinned at that day's size by
/// file and heading. Each may only shrink; a row whose section drops under
/// the ceiling, or whose heading is renamed, must be deleted. `JOURNAL.md`
/// is not measured: it is the append-only chronicle and is meant to grow.
/// A section that wants to grow past its pin splits — a `## ` heading is
/// the split, and it costs nothing a reader needs.
const OVER: &[(&str, &str, usize)] = &[
    ("AI.md", "2. The production AI — read", 71_955),
    (
        "AI.md",
        "15. The behavioural run — run18, 2026-08-25",
        27_527,
    ),
    ("CITIES.md", "3. Construction", 16_823),
    (
        "DATALAYER.md",
        "2. The loader — the tables into the sim's types",
        20_324,
    ),
    (
        "GROUPS.md",
        "6. The move — `Group::action_move_near@00704990`",
        44_907,
    ),
    ("ORDERS.md", "1. The order system", 16_545),
    ("ORDERS.md", "4. The move order", 37_806),
    (
        "ORDERS.md",
        "5. Build, repair, garrison — and what a citizen does next",
        16_431,
    ),
];

/// `(heading, bytes)` for every `## ` section of a document, the preamble
/// first under an empty heading. A section's bytes run from its heading
/// line to the next `## ` line, newlines included.
fn sections(text: &str) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = vec![(String::new(), 0)];
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            out.push((h.to_string(), 0));
        }
        out.last_mut().expect("preamble").1 += line.len() + 1;
    }
    out
}

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

/// `Class<T,U>::method` → `Class::method`: the export prints template
/// arguments, the documents cite without them.
fn strip_templates(name: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in name.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Every `name@00xxxxxx` in a document, as `(line, name, address)`. The
/// name is the run of identifier characters (plus `:` and `~`) before the
/// `@`; a citation wrapped across a line break yields the suffix that
/// survived, which the ends-with match accepts.
fn cites(text: &str) -> Vec<(usize, String, u32)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        for (at, _) in line.match_indices('@') {
            let hex: String = line[at + 1..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if hex.len() != 8 || !hex.starts_with("00") {
                continue;
            }
            let mut start = at;
            while start > 0 {
                let c = bytes[start - 1] as char;
                if c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '~' {
                    start -= 1;
                } else {
                    break;
                }
            }
            let name = line[start..at].trim_start_matches(':').to_string();
            let addr = u32::from_str_radix(&hex, 16).expect("hex");
            out.push((i + 1, name, addr));
        }
    }
    out
}

#[test]
fn every_cited_address_names_its_function() {
    let home = std::env::var("HOME").unwrap_or_default();
    let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
    let Ok(index) = std::fs::read_to_string(&index_path) else {
        eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
        return;
    };
    let mut funcs = std::collections::BTreeMap::new();
    let mut last = 0u32;
    for line in index.lines() {
        let mut it = line.split('\t');
        let (Some(addr), Some(name)) = (it.next(), it.next()) else {
            continue;
        };
        let Ok(addr) = u32::from_str_radix(addr, 16) else {
            continue;
        };
        funcs.insert(addr, strip_templates(name).to_lowercase());
        last = last.max(addr);
    }
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name == "JOURNAL.md" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        for (line, cited, addr) in cites(&text) {
            let c = cited.to_lowercase();
            match funcs.get(&addr) {
                // An empty name is a cite whose name sits behind an
                // argument list or a line wrap: the address half still
                // checks. Either side may be the more qualified — the
                // documents cite `add` for `LinkListBase<…>::add`, and
                // `OrdersMemManager::get_new_order` where the export
                // holds the bare global.
                Some(f) if c.is_empty() || f.ends_with(&c) || c.ends_with(f.as_str()) => {}
                Some(f) => failures.push(format!(
                    "docs/{name}:{line}: `{cited}@{addr:08x}` — the export names {addr:08x} `{f}`"
                )),
                // Past the last function it is data, which a function
                // index cannot check (the four `PATHDATA` tables live
                // there); inside the range an unknown address is a typo.
                None if addr > last => {}
                None => failures.push(format!(
                    "docs/{name}:{line}: `{cited}@{addr:08x}` — no function in the export starts there"
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_section_over_the_ceiling_may_not_grow() {
    let mut failures = Vec::new();
    let mut seen = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name == "JOURNAL.md" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        for (heading, size) in sections(&text) {
            let pin = OVER
                .iter()
                .find(|(n, h, _)| *n == name && *h == heading)
                .map(|&(_, _, pinned)| pinned);
            if pin.is_some() {
                seen.push((name.to_string(), heading.clone()));
            }
            let bound = pin.unwrap_or(SECTION_CEILING);
            let label = if heading.is_empty() {
                "the preamble".to_string()
            } else {
                format!("`## {heading}`")
            };
            if size > bound {
                failures.push(format!(
                    "docs/{name} {label}: {size} bytes, bound {bound} — split the section, or move its story to the journal"
                ));
            } else if let Some(pinned) = pin {
                if size <= SECTION_CEILING {
                    failures.push(format!(
                        "docs/{name} {label} is under the ceiling now ({size} ≤ {SECTION_CEILING}); delete its row from OVER"
                    ));
                } else if size < pinned {
                    // Fine — but say so, so the next session lowers the pin.
                    eprintln!("docs/{name} {label} shrank to {size}; lower its pin from {pinned}");
                }
            }
        }
    }
    for (name, heading, _) in OVER {
        if !seen.iter().any(|(n, h)| n == name && h == heading) {
            failures.push(format!(
                "docs/{name} has no `## {heading}` — the pinned section was renamed or removed; fix the row"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
