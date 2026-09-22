//! **The widening ledger** — which of the dump's own fields nothing in
//! `crate::diff` compares, and which rest on a single capture (queue item
//! 87).
//!
//! The project's most productive rule is "when the original dumps a record,
//! diff the whole record". It has closed or sharpened eleven queue items on
//! its own — 74, 83, 69, 113, 123, 144, 154, 169, 168/165, 179, 180 — and
//! item 186 is the twelfth: `UnitDump::orders_x` had been parsed since the
//! `UNITDATA` reader was written, and the merchant window that needed it
//! compared positions and clocks instead, so a merchant walking at the
//! wrong rare read as a merchant one frame late. A field the parser fills
//! and nothing reads is a finding waiting for somebody to notice. A *count*
//! of them is the queue of cheap widenings, and this is that count.
//!
//! Two lists, because item 87 asks for two:
//!
//! - **Uncompared** — the field's identifier appears nowhere in `diff.rs`.
//! - **Single-capture** — exactly one test function names it. That is the
//!   per-capture half, and it is the sharper of the two: a field pinned by
//!   one window and absent from the next is one capture away from an
//!   unnoticed divergence, which is precisely what happened here.
//!
//! **What "names" means, and what it does not.** A field counts as named if
//! its identifier appears as `.field` or as the string a comparison row
//! labels itself with (`row("orders_x", …)`). That is deliberately
//! generous: an appearance is not proof of a comparison, only that somebody
//! has touched the field. So the uncompared list is a **lower bound with no
//! false alarms** — everything on it is certainly uncompared, and some
//! fields off it may be too. For a queue that is the useful direction: it
//! never sends anyone chasing a field that is already pinned.

/// The parser's own source, and the differ's. Read at compile time so the
/// ledger cannot go stale against a moved file or a changed working
/// directory.
#[cfg(test)]
const GAMELOG: &str = include_str!("gamelog.rs");
/// The differ is a spine and one module per dumped record family
/// (item 228), so this is every one of them; the test concatenates them.
///
/// **`diff/golden.rs` was missing until item 510**, and it is the rules
/// track's whole walk — `chapter_two_s_word_frame_is_widened_whole` and
/// the five per-field checks beside it. Every field those compare read as
/// *uncompared* here, and every field they were the second reader of read
/// as single-capture, which is exactly the blindness this module exists
/// to count. It was found by widening chapter two's `GUY` record: the two
/// fields that landing added were compared on both headline windows and
/// the ledger still called them unread.
#[cfg(test)]
const DIFF: [&str; 13] = [
    include_str!("diff.rs"),
    include_str!("diff/army.rs"),
    include_str!("diff/build.rs"),
    include_str!("diff/city.rs"),
    include_str!("diff/floors.rs"),
    include_str!("diff/golden.rs"),
    include_str!("diff/harness.rs"),
    include_str!("diff/order.rs"),
    include_str!("diff/report.rs"),
    include_str!("diff/setup.rs"),
    include_str!("diff/testkit.rs"),
    include_str!("diff/unit.rs"),
    include_str!("diff/world.rs"),
];

/// The parser's own containers, which carry no field the original writes:
/// `Block` and `Log` are the reader's cursor and its index, `Initial` and
/// `ConstantDump` hold parsed aggregates rather than dumped scalars. They
/// are excluded by name because nothing distinguishes them structurally,
/// and a ledger that lists `Log::preamble` as a missing widening is a
/// ledger nobody reads.
#[cfg(test)]
const NOT_THE_ORIGINAL_S: &[&str] = &["Block", "Log", "Initial", "ConstantDump"];

/// One parsed record and the field names it carries.
#[derive(Debug)]
pub struct Record {
    pub name: String,
    pub fields: Vec<String>,
}

/// Every `pub struct` of the parser, with its `pub` field names. Doc
/// comments, attributes and non-`pub` fields are skipped; a lifetime or
/// generic parameter on the name is dropped, so `CellDump<'a>` is
/// `CellDump`.
pub fn records(src: &str) -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    let mut open = false;
    for line in src.lines() {
        if !open {
            let Some(rest) = line.strip_prefix("pub struct ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            // `Name<'a> {` and `Name {` have fields; `Name;` does not.
            if rest.contains('{') {
                out.push(Record {
                    name,
                    fields: Vec::new(),
                });
                open = true;
            }
            continue;
        }
        if line == "}" {
            open = false;
            continue;
        }
        let Some(rest) = line.trim_start().strip_prefix("pub ") else {
            continue;
        };
        let field: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        // A field line ends its name with a colon; `pub fn` and `pub const`
        // do not.
        if field.is_empty() || !rest[field.len()..].starts_with(':') {
            continue;
        }
        out.last_mut().expect("a struct is open").fields.push(field);
    }
    out
}

/// Every identifier `src` names, as `.ident` or as the string `"ident"`.
/// Both forms count because a comparison row labels the field with the
/// original's own name as often as it reads it.
pub fn named(src: &str) -> std::collections::HashSet<String> {
    let b = src.as_bytes();
    let mut out = std::collections::HashSet::new();
    for (i, c) in b.iter().enumerate() {
        if *c != b'.' && *c != b'"' {
            continue;
        }
        // `1.5` and `0.` are not field accesses.
        if b.get(i + 1).is_some_and(u8::is_ascii_digit) {
            continue;
        }
        let s: String = b[i + 1..]
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric() || **c == b'_')
            .map(|c| *c as char)
            .collect();
        if !s.is_empty() {
            out.insert(s);
        }
    }
    out
}

/// `diff.rs` split into its functions: `(name, body)`. A function runs from
/// its `fn name(` line to the next one, so a helper between two tests is
/// attributed to the one above it. That is coarse in the safe direction —
/// it can only make a field look *better* covered than it is, never worse.
pub fn test_bodies(src: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in src.lines() {
        if let Some(at) = line.find("fn ") {
            let after = &line[at + 3..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && after[name.len()..].starts_with('(') {
                out.push((name, String::new()));
            }
        }
        if let Some(last) = out.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fields of the original's own records that `diff.rs` never names.
    /// **23 on 2026-09-03.** The pin may only fall: a widening deletes
    /// rows, and a field the parser gains arrives here until somebody
    /// compares it, which is the point.
    /// **19 on 2026-09-06**, when item 237 gave the `GROUPORDER` row a
    /// comparison — `group_angle`, `group_id` and `in_group` left this
    /// half for the one below.
    /// **18 on 2026-09-17**, item 308: `UnitData::start_dist` is compared
    /// now (`FrameResult::search_compared`, `docs/PATHFINDER.md` §19). It
    /// had been parsed since item 301 and read only off the original's own
    /// dump, which is exactly the shape this half exists to catch.
    /// **14 on 2026-09-21**, item 478: `BuildDump::build_masks`. It was
    /// the sharpest instance this ledger has produced — the field is
    /// written at every detail level, the reader has carried it since the
    /// `BUILDDATA` block existed, and it was not merely uncompared but
    /// **unparsed**, because `build_of` took it off `BUILDDATA` where the
    /// record writes it at `WALLDATA`'s indent. Its `0x100` is the road
    /// replan flag, and it held Great Lakes' word at 10234 for four items
    /// while sitting in every block of every capture on the disk
    /// (`docs/ROADS.md` §1.2).
    const UNCOMPARED: usize = 14;

    /// Fields exactly one test function names — the per-capture half.
    /// **41 on 2026-09-04**, down from 51 when run68's window widened
    /// `idle`, `path_recursion`, `safe`, the collision block and the crew
    /// track onto a second capture, one more when item 190 gave `stance` a
    /// second reader — this ledger had named it as a single-capture field
    /// the day before run68 found it wrong — and one more when item 210
    /// gave `des_angle` a third. This one is a *ceiling on fragility*, not
    /// a target of zero: a field a single window can see is not a defect,
    /// it is a field whose next capture should be asked to carry it too.
    /// **44 on 2026-09-06**, and this is the one direction in which a
    /// rise is progress: item 237's three `GROUPORDER` fields arrived
    /// here *from* the uncompared half, which fell by three at the same
    /// time. Only run76 marches a formation, so they are single-capture
    /// until run79's squad is windowed too.
    /// **45 on 2026-09-07**, item 271: `GuyData::last_x`/`last_y` arrive as
    /// `Guy::last_pos` and only run86's window reads them. They are what
    /// tells a figure that was *placed* from one that walked, which is how
    /// `Leader::gain_tech`'s age snap is read off a dump at all
    /// (`docs/TECH.md`, "An age snaps every figure"); the same item's
    /// `BuildDump::max_age` lands with more than one reader and so does not
    /// count here.
    /// **46 on 2026-09-17**, item 308: `UnitDump::uid` arrives, named by
    /// the corpus sweep alone. It is the identity behind the per-player
    /// `o`, and it is parsed because a claim about a field nothing clears
    /// cannot be carried by a slot number — run16's `1/9` reads
    /// `start_dist` 768 under `uid 17` and 0 under `uid 25`. It stays
    /// single-capture until a window test has a reason to link on it.
    /// **41 on 2026-09-21**, item 478 — and this half did **not** move on
    /// that item. It had been 41 since some earlier landing and the
    /// ceiling stayed at 46, which the "it fell" guard could not see
    /// while the other half sat exactly on its own tolerance. Lowered to
    /// what the tree prints, so the next field that comes to rest on one
    /// capture fails this.
    /// **40 on 2026-09-22, item 484.** `UnitDump` gained `damage_frac`
    /// and the hit-point record went into [`crate::diff::compare`], so
    /// `myhits` and `damage` are named by more than one window now.
    /// Lowered to what the tree prints for item 478's reason.
    /// **39 on 2026-09-22, item 485.** `UnitDump` gained the overkill
    /// window — `damage_frame`, `damage_o`, `damage_who` — and all three
    /// went into [`crate::diff::compare`] *and* run100's own word-block
    /// walk in the same landing, so a record family that arrived with
    /// three new fields cost the ledger nothing and gave back one.
    /// Lowered again for item 478's reason.
    const SINGLE_CAPTURE: usize = 39;

    #[test]
    fn the_widening_ledger_counts_what_nothing_compares() {
        let recs: Vec<Record> = records(GAMELOG)
            .into_iter()
            .filter(|r| !NOT_THE_ORIGINAL_S.contains(&r.name.as_str()))
            .collect();
        assert!(
            recs.len() >= 18 && recs.iter().any(|r| r.name == "UnitDump"),
            "the parser's records did not parse: {} found",
            recs.len()
        );
        let diff = DIFF.concat();
        let seen = named(&diff);
        // One pass over the file, not one per record: `named` is a byte
        // scan and the differ is 800 KB.
        let per_test: Vec<std::collections::HashSet<String>> = test_bodies(&diff)
            .iter()
            .map(|(_, body)| named(body))
            .collect();
        let readers = |f: &str| per_test.iter().filter(|n| n.contains(f)).count();

        let mut uncompared = 0usize;
        let mut single = 0usize;
        let mut rows: Vec<String> = Vec::new();
        for r in &recs {
            let missing: Vec<&str> = r
                .fields
                .iter()
                .filter(|f| !seen.contains(*f))
                .map(String::as_str)
                .collect();
            let thin: Vec<&str> = r
                .fields
                .iter()
                .filter(|f| seen.contains(*f) && readers(f) == 1)
                .map(String::as_str)
                .collect();
            uncompared += missing.len();
            single += thin.len();
            if missing.is_empty() && thin.is_empty() {
                continue;
            }
            rows.push(format!(
                "  {:<16} {:>2}/{:<2} uncompared{}{}",
                r.name,
                missing.len(),
                r.fields.len(),
                if missing.is_empty() {
                    String::new()
                } else {
                    format!(": {}", missing.join(", "))
                },
                if thin.is_empty() {
                    String::new()
                } else {
                    format!("\n{:<18} one capture only: {}", "", thin.join(", "))
                }
            ));
        }
        let total: usize = recs.iter().map(|r| r.fields.len()).sum();
        eprintln!(
            "the widening ledger, over {} of the original's records and {total} fields: \
             {uncompared} that nothing names, {single} that one capture names",
            recs.len()
        );
        for r in &rows {
            eprintln!("{r}");
        }
        assert!(
            uncompared <= UNCOMPARED && single <= SINGLE_CAPTURE,
            "the ledger grew: {uncompared} uncompared (pin {UNCOMPARED}) and {single} \
             single-capture (pin {SINGLE_CAPTURE}). A parsed field arrives here until \
             it is compared — widen the record that gained it, or move the pin \
             deliberately and say so in docs/QUEUE.md"
        );
        assert!(
            uncompared + 3 >= UNCOMPARED && single + 6 >= SINGLE_CAPTURE,
            "the ledger fell to {uncompared} uncompared and {single} single-capture, \
             against pins of {UNCOMPARED} and {SINGLE_CAPTURE} — lower them, so the \
             next field that goes uncompared still fails this"
        );
    }
}
