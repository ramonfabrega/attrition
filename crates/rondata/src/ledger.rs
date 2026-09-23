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
///
/// **And the list is checked against the directory** (parked 517, the
/// eleventh pass): `diff/leader.rs`, the leader record's whole reader since
/// item 520, was absent from it the day this was written, which is the
/// `golden.rs` defect a second time. [`DIFF_FILES`] names each entry's
/// path in the same order, `every_differ_module_is_on_the_ledger` reads
/// `src/diff/` and fails on a file in neither it nor [`NOT_A_DIFFER`].
#[cfg(test)]
const DIFF: [&str; 19] = [
    include_str!("diff.rs"),
    include_str!("diff/ammo.rs"),
    include_str!("diff/army.rs"),
    include_str!("diff/build.rs"),
    include_str!("diff/city.rs"),
    include_str!("diff/corrections.rs"),
    include_str!("diff/endpoint.rs"),
    include_str!("diff/floors.rs"),
    include_str!("diff/golden.rs"),
    include_str!("diff/harness.rs"),
    include_str!("diff/harness/checkpoint.rs"),
    include_str!("diff/leader.rs"),
    include_str!("diff/order.rs"),
    include_str!("diff/report.rs"),
    include_str!("diff/setup.rs"),
    include_str!("diff/shutdown.rs"),
    include_str!("diff/testkit.rs"),
    include_str!("diff/unit.rs"),
    include_str!("diff/world.rs"),
];

/// [`DIFF`]'s entries by path, relative to `src/`, in the same order.
#[cfg(test)]
const DIFF_FILES: [&str; 19] = [
    "diff.rs",
    "diff/ammo.rs",
    "diff/army.rs",
    "diff/build.rs",
    "diff/city.rs",
    "diff/corrections.rs",
    "diff/endpoint.rs",
    "diff/floors.rs",
    "diff/golden.rs",
    "diff/harness.rs",
    "diff/harness/checkpoint.rs",
    "diff/leader.rs",
    "diff/order.rs",
    "diff/report.rs",
    "diff/setup.rs",
    "diff/shutdown.rs",
    "diff/testkit.rs",
    "diff/unit.rs",
    "diff/world.rs",
];

/// A file under `src/diff/` that is deliberately not on the ledger, with
/// the reason. `coverage.rs` *lists* the keys nothing reads in its
/// `UNREAD` pin; scanning it would call every one of them compared.
#[cfg(test)]
const NOT_A_DIFFER: &[(&str, &str)] = &[(
    "diff/coverage.rs",
    "its UNREAD pin names the keys nothing reads",
)];

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
    const UNCOMPARED: usize = 10;

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
    /// Lowered 14 → 10 and 39 → 36 by the eleventh pass, when the ledger's
    /// list was checked against the directory and six differs joined it
    /// (`leader.rs` among them): four fields called uncompared and three
    /// called single-capture had been compared all along.
    const SINGLE_CAPTURE: usize = 36;

    /// **Every file under `src/diff/` is on the ledger or named as not a
    /// differ** (parked 517). The ledger's scope was a hand-kept list and
    /// nothing checked the list: `golden.rs` was missing until item 510
    /// and `leader.rs` until the eleventh pass, and each absence read as
    /// health — every field those two compared counted as uncompared or
    /// as single-capture. Made to fail first on the tip of 2026-09-23,
    /// where six files were in neither list.
    #[test]
    fn every_differ_module_is_on_the_ledger() {
        assert_eq!(DIFF.len(), DIFF_FILES.len());
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut on_disk = vec!["diff.rs".to_string()];
        let mut dirs = vec![root.join("diff")];
        while let Some(d) = dirs.pop() {
            for e in std::fs::read_dir(&d).expect("src/diff") {
                let p = e.unwrap().path();
                if p.is_dir() {
                    dirs.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    let rel = p.strip_prefix(root).unwrap().to_str().unwrap().to_string();
                    on_disk.push(rel);
                }
            }
        }
        on_disk.sort();
        let listed: std::collections::BTreeSet<&str> = DIFF_FILES
            .iter()
            .copied()
            .chain(NOT_A_DIFFER.iter().map(|(p, _)| *p))
            .collect();
        let missing: Vec<&str> = on_disk
            .iter()
            .map(String::as_str)
            .filter(|p| !listed.contains(p))
            .collect();
        assert!(
            missing.is_empty(),
            "under src/diff/ and on neither DIFF_FILES nor NOT_A_DIFFER: {missing:?}. A \
             differ's fields count as uncompared until its file is on the ledger; a file \
             that is not a differ is named with the reason"
        );
        let gone: Vec<&str> = listed
            .iter()
            .copied()
            .filter(|p| !on_disk.iter().any(|d| d == p))
            .collect();
        assert!(gone.is_empty(), "on the ledger and not on disk: {gone:?}");
    }

    /// **A comparison gated on both sides says which empty side is quiet**
    /// (parked 503). `compare_orders` reported an order's target only when
    /// *both* sides named one, so this crate's empty target read as agreeing
    /// with the dump's — under the word, green, three times in one function
    /// (items 462, 496, 502; `docs/COMBAT.md` §43.3.1). Every guard before
    /// this one was aimed at an instrument that says nothing; this is aimed
    /// at one that says *yes*. The grep is mechanical: each `if let (Some(`
    /// in a differ carries, within the six lines above it, a `both
    /// sides:` comment naming what an absent side means on each side — a
    /// field this crate does not model, a detail level that does not print
    /// it, or a real disagreement, which must not be quiet. Made to fail
    /// first on eight unannotated sites.
    #[test]
    fn a_comparison_gated_on_both_sides_says_which_side_is_quiet() {
        let mut bare = Vec::new();
        for (file, src) in DIFF_FILES.iter().zip(DIFF.iter()) {
            let lines: Vec<&str> = src.lines().collect();
            for (i, l) in lines.iter().enumerate() {
                if !l.contains("if let (Some(") {
                    continue;
                }
                let above = &lines[i.saturating_sub(6)..i];
                if !above.iter().any(|a| a.contains("both sides:")) {
                    bare.push(format!("{file}:{}", i + 1));
                }
            }
        }
        assert!(
            bare.is_empty(),
            "a comparison gated on both sides being present, with no `// both sides:` \
             line above it saying what an absent side means: {bare:?}. A comparison \
             written as *compare when both carry it* reads an empty side as agreement"
        );
    }

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
