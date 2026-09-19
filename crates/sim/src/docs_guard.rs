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

/// The most **open items** the queue may book, and the lines one may take.
///
/// This replaced a whole-file line count on 2026-09-07 (item 262, decided
/// with Ramon and lore). The line count was measured across 221 sessions
/// and cost **43 fitting episodes and 117 USD of list price** since
/// 08-25 — 22 of the 43 needed two or more guard runs, so the fitting loop
/// was the cost rather than the edit, and the "count the lines before
/// writing" rule of `d03c279` had not ended it.
///
/// The reason it is replaced rather than raised is **not** the money. A
/// global line count is satisfied by compressing *any* item, so its remedy
/// is a retelling of entries the author has no reason to have read, and
/// every one of those edits is a chance to drop somebody else's finding.
/// That is not hypothetical: `8b37e5f` was a post-crash rewrite under this
/// exact pressure, and items 232, 233 and 234 went out with the
/// compression unlanded and unnoticed for a day
/// (`docs/audit/queue-ledger.md`). A per-item cap localises the edit to
/// the item being added, by the person who knows what is safe to cut.
///
/// Fifteen items stood on the day this landed, the longest at seven lines.
const OPEN_ITEMS: usize = 18;
const ITEM_LINES: usize = 8;

/// The handoff section, in lines including blanks. The file says "about
/// twenty"; this is the tolerance.
const HANDOFF_LINES: usize = 32;

/// The queue's **non-item mass**, pinned by section at its size on
/// 2026-09-07 and may only shrink.
///
/// An item cap bounds items and nothing else, and the queue is not only
/// items: the preamble, the standing paragraphs inside "The queue", and the
/// maintenance notes all grew under the old line count without ever being
/// an item. So each section carries the same may-only-shrink pin the
/// specifications use ([`OVER`]) — the mechanism this project already
/// trusts — in lines rather than bytes, because lines are what a queue
/// section costs a reader. Lower a pin whenever a section comes in under
/// it; that is the only direction it moves.
///
/// "Where things stand" is absent on purpose: [`HANDOFF_LINES`] is its
/// bound and two caps on one section is the double-binding that made the
/// old fitting loop.
/// The sizes are **non-item lines**: an item's own lines are bounded by
/// [`ITEM_LINES`] and are subtracted here, so booking one costs a section
/// nothing. "The queue"'s 35 is its heading, its opening instruction, the
/// widening ledger and the `(242)` cluster.
const QUEUE_SECTIONS: &[(&str, usize)] = &[
    ("", 12),
    ("The queue", 8),
    ("How to maintain this file", 38),
];

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
    ("AI.md", "2. The production AI — read", 71_928),
    (
        "AI.md",
        "15. The behavioural run — run18, 2026-08-25",
        27_421,
    ),
    ("CITIES.md", "3. Construction", 16_280),
    (
        "DATALAYER.md",
        "2. The loader — the tables into the sim's types",
        20_298,
    ),
    (
        "GROUPS.md",
        "6. The move — `Group::action_move_near@00704990`",
        44_885,
    ),
    // Lowered 37_601 -> 37_590 by item 405: §4.4 step 4 gains the ATTACK
    // arm's `max_range` gate and loses a gloss `docs/COLLISION.md` §5.1
    // already carries.
    ("ORDERS.md", "4. The move order", 37_590),
    (
        "ORDERS.md",
        "5. Build, repair, garrison — and what a citizen does next",
        16_088,
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

/// `(number, lines)` for every open item the queue books. An item opens a
/// paragraph — `N. **The claim**` after a blank line — and runs to the next
/// blank line. The blank line is what keeps prose out: the queue wraps, so
/// a sentence ending in a number leaves a bare `307.` at the head of the
/// next line and nothing else tells the two apart. The same reading as
/// `tools/queueledger.py`'s.
fn open_items(text: &str) -> Vec<(u32, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let opens = (i == 0 || lines[i - 1].trim().is_empty())
            && lines[i]
                .split_once(". ")
                .and_then(|(n, _)| n.parse::<u32>().ok())
                .is_some();
        if opens {
            let n: u32 = lines[i]
                .split_once(". ")
                .expect("checked")
                .0
                .parse()
                .expect("checked");
            let mut len = 0;
            while i + len < lines.len() && !lines[i + len].trim().is_empty() {
                len += 1;
            }
            out.push((n, len));
            i += len;
        } else {
            i += 1;
        }
    }
    out
}

/// The queue books a bounded number of open items, each of a bounded size.
///
/// **This is the whole budget** — there is no line count on the file any
/// more (see [`OPEN_ITEMS`]). The two failures it can report are the two
/// remedies that are actually local: an item over its lines is shortened by
/// whoever is touching it, and a queue over its items has one to finish or
/// park. `docs/PARKED.md` is where a parked one goes, and moving it there
/// is not a deletion — `tools/queueledger.py` reads both files.
#[test]
fn the_queue_caps_items_not_lines() {
    let items = open_items(&read("QUEUE.md"));
    let long: Vec<String> = items
        .iter()
        .filter(|(_, len)| *len > ITEM_LINES)
        .map(|(n, len)| format!("{n} ({len} lines)"))
        .collect();
    assert!(
        long.is_empty(),
        "docs/QUEUE.md items are over {ITEM_LINES} lines: {}. Shorten the item you are \
         touching — do not compress the rest of the file to make room (item 262)",
        long.join(", ")
    );
    assert!(
        items.len() <= OPEN_ITEMS,
        "docs/QUEUE.md books {} open items; the cap is {OPEN_ITEMS}. Finish one, or park \
         one in docs/PARKED.md — which is a move, not a deletion",
        items.len()
    );
}

/// Every pinned queue section is at or under its pin, and a section that
/// has come in under it says so rather than banking the slack.
#[test]
fn a_queue_section_may_only_shrink() {
    let q = read("QUEUE.md");
    // **The item lines do not count.** If they did, booking an ordinary
    // item would push its section past the pin and send the author off to
    // compress the rest of the file — the fitting loop item 262 removed,
    // rebuilt one level down. It was rebuilt, briefly, and the fixture
    // that was meant to prove the item cap caught it. So a section is
    // measured by what is *not* an item: prose, standing paragraphs, the
    // maintenance notes. Items are bounded by their own cap and by nothing
    // else.
    let lines: Vec<&str> = q.lines().collect();
    let mut item_line = vec![false; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let opens = (i == 0 || lines[i - 1].trim().is_empty())
            && lines[i]
                .split_once(". ")
                .and_then(|(n, _)| n.parse::<u32>().ok())
                .is_some();
        if opens {
            while i < lines.len() && !lines[i].trim().is_empty() {
                item_line[i] = true;
                i += 1;
            }
            // The blank line that separates it from the next entry belongs
            // to the item too. Without this an item still costs its
            // section one line, which is a fitting loop with a slower fuse.
            if i < lines.len() {
                item_line[i] = true;
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    let mut have: Vec<(String, usize)> = vec![(String::new(), 0)];
    for (i, line) in lines.iter().enumerate() {
        if let Some(h) = line.strip_prefix("## ") {
            have.push((h.to_string(), 0));
        }
        if !item_line[i] {
            have.last_mut().expect("preamble").1 += 1;
        }
    }
    for (heading, pin) in QUEUE_SECTIONS {
        let (_, size) = have.iter().find(|(h, _)| h == heading).unwrap_or_else(|| {
            panic!("docs/QUEUE.md has no section {heading:?}; QUEUE_SECTIONS names it")
        });
        let label = if heading.is_empty() {
            "the preamble"
        } else {
            heading
        };
        assert!(
            size <= pin,
            "docs/QUEUE.md's {label} is {size} lines against its pin of {pin}. \
             The pin only falls: shorten this section rather than another (item 262)"
        );
        assert!(
            *size + 4 > *pin,
            "docs/QUEUE.md's {label} is {size} lines, well under its pin of {pin}; \
             lower the pin in QUEUE_SECTIONS to bank the shrink"
        );
    }
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

/// The steering pass's backlog is the parked file's Loop section, which no
/// session reads at boot; the queue's handoff carries its count so a Fable
/// session sees it in the same breath as the score (parked item 332, ruled
/// 2026-09-18). A count nobody checks drifts — it read "twelve" over thirteen
/// items the day this was written — so the line is parsed against the section.
#[test]
fn the_handoff_counts_the_loop_backlog() {
    let q = read("QUEUE.md");
    let line = q
        .lines()
        .find(|l| l.contains("Fable backlog:"))
        .expect("docs/QUEUE.md's handoff has no `Fable backlog: N Loop items` line");
    let after = line.split("Fable backlog:").nth(1).unwrap();
    let word = after
        .trim_start_matches([' ', '*'])
        .split_whitespace()
        .next()
        .unwrap_or("");
    const WORDS: [&str; 21] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
    ];
    let said = word
        .parse::<usize>()
        .ok()
        .or_else(|| WORDS.iter().position(|w| *w == word))
        .unwrap_or_else(|| panic!("`Fable backlog:` is followed by {word:?}, not a count"));
    let parked = read("PARKED.md");
    let mut in_loop = false;
    let mut items = Vec::new();
    for l in parked.lines() {
        if l.starts_with("## ") {
            in_loop = l.starts_with("## Loop");
            continue;
        }
        // An item opens `(N) **…`; a bare `(N) ` mid-item is a reference.
        let Some((number, _)) = l.strip_prefix('(').and_then(|l| l.split_once(") **")) else {
            continue;
        };
        if let (true, Ok(n)) = (in_loop, number.parse::<u32>()) {
            items.push(n);
        }
    }
    assert!(
        !items.is_empty(),
        "docs/PARKED.md has no `## Loop` section with `(N) **` items"
    );
    assert_eq!(
        said,
        items.len(),
        "docs/QUEUE.md's handoff says the Fable backlog is {said} Loop items; docs/PARKED.md's \
         Loop section holds {}: {items:?}. Rewrite the count with the handoff",
        items.len()
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

/// The rules track has a slot that is never empty (DECISIONS 41). The
/// handoff carries a `Golden:` line beside `Scoreboard:` and `Long
/// captures:` — `none pinned` with the takes-chain until the first chapter
/// pins, then `w<frame>` — so the golden record cannot fall out of the
/// handoff the way the second map's capture once fell out of the queue.
#[test]
fn the_handoff_carries_the_golden_line() {
    let q = read("QUEUE.md");
    let line = q
        .lines()
        .find(|l| l.starts_with("Golden:"))
        .expect("docs/QUEUE.md's handoff has no `Golden:` line; write `Golden: none pinned · <takes-chain>` or `Golden: w<frame> of <length> · …`");
    let rest = line.trim_start_matches("Golden:").trim();
    let pinned = rest.split_whitespace().any(|w| {
        w.strip_prefix('w')
            .is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
    });
    assert!(
        pinned || rest.starts_with("none pinned"),
        "the `Golden:` line is {rest:?}; it says `none pinned` or names a word `w<frame>`"
    );
}

/// `docs/DECISIONS.md` is a ledger nobody reads whole, and an amended entry
/// does not say so at its own heading; its index does (DECISIONS 41). Every
/// `## N.` entry has an index row `- N <status> — <title>`, and no row names
/// an entry that does not exist.
#[test]
fn every_decision_has_an_index_row() {
    let d = std::fs::read_to_string(docs().join("DECISIONS.md")).expect("docs/DECISIONS.md");
    let entries: Vec<u32> = d
        .lines()
        .filter_map(|l| l.strip_prefix("## "))
        .filter_map(|h| h.split_once(". ").and_then(|(n, _)| n.parse().ok()))
        .collect();
    assert!(
        !entries.is_empty(),
        "docs/DECISIONS.md has no `## N.` entries"
    );
    let mut in_index = false;
    let mut rows = Vec::new();
    for l in d.lines() {
        if let Some(h) = l.strip_prefix("## ") {
            in_index = h.starts_with("Index");
            continue;
        }
        if !in_index {
            continue;
        }
        if let Some(r) = l.strip_prefix("- ") {
            let (n, rest) = r.split_once(' ').unwrap_or((r, ""));
            let n: u32 = n
                .parse()
                .unwrap_or_else(|_| panic!("index row {l:?} does not start with an entry number"));
            let status = rest.split(" — ").next().unwrap_or("");
            let ok = status == "standing"
                || status == "amended in place"
                || status.starts_with("amended by ")
                || status.starts_with("extended by ")
                || status.starts_with("superseded by ");
            assert!(
                ok,
                "index row {l:?}: the status is {status:?}, not one of standing / amended in place / amended by N / extended by N / superseded by N"
            );
            rows.push(n);
        }
    }
    assert!(
        !rows.is_empty(),
        "docs/DECISIONS.md has no `## Index` section with `- N` rows"
    );
    let missing: Vec<u32> = entries
        .iter()
        .copied()
        .filter(|n| !rows.contains(n))
        .collect();
    let extra: Vec<u32> = rows
        .iter()
        .copied()
        .filter(|n| !entries.contains(n))
        .collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "docs/DECISIONS.md's index and its entries disagree: entries with no row {missing:?}, rows with no entry {extra:?}"
    );
}
