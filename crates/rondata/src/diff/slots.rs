//! **A dumped record's slot index is not an identity** — the guard.
//!
//! An object number is reused once `Objects::find_free` hands it on, so a
//! lookup by owner and number alone can return a closed object while a live
//! one stands where the record points. Until item 989 the harness linked a
//! dumped building to the first building of its number, dead or alive, and
//! two widenings reported this crate's dead site against the original's
//! live one as a parting (`docs/AI.md` §81.4, parked 1001). The unit
//! lookups tested `alive()` from the start; the building lookups never
//! did, and 989 left two of them unread.
//!
//! So this reads the harness's own source, as `sim::no_float` reads the
//! simulation's: every comparison of an object's `index` against a dumped
//! number, in `src/diff`, tests liveness in the same statement or goes
//! through [`super::harness::link_building`], or it is listed below with
//! the reason it need not. The eighteenth pass, 2026-09-28; its first run
//! found six, of which two were changed and four are the rows.

use std::path::{Path, PathBuf};

/// `(file, a fragment of the line, why liveness is not tested there)`.
const BY_NUMBER_ALONE: &[(&str, &str, &str)] = &[
    (
        "harness.rs",
        "let held = |x: &sim::Building|",
        "link_building's own predicate: the live holder first, then a closed one",
    ),
    (
        "harness.rs",
        "Some(n) => i64::from(x.index) == n,",
        "the debug print's filter, on an iterator already filtered on alive()",
    ),
    (
        "harness.rs",
        ".any(|x| i64::from(x.owner) == b.who && i64::from(x.index) == b.o);",
        "build:unlinked asks whether any holder exists, which is link_building's Some",
    ),
    (
        "setup.rs",
        ".position(|bd| i64::from(bd.owner) == f.who && i64::from(bd.index) == f.o)",
        "the farms' list at frame 0: nothing has closed and no number is reused",
    ),
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("src/diff") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `(line number, line)` for every comparison by number alone in `text`.
fn by_number_alone(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        if !code.contains(".index) ==") {
            continue;
        }
        // The statement: the closure's own line and the two above it,
        // where `rustfmt` puts a predicate it has had to break.
        let statement = lines[i.saturating_sub(2)..=i].join("\n");
        if statement.contains("alive") {
            continue;
        }
        out.push((i + 1, line.trim().to_string()));
    }
    out
}

#[test]
fn the_scanner_knows_a_live_lookup_from_a_lookup_by_number() {
    let live = "let u = sim.units.iter()\n    .find(|u| u.alive() && i64::from(u.index) == o);";
    assert!(by_number_alone(live).is_empty());
    let broken = "let u = sim.units.iter().position(|u| {\n    u.alive()\n        && i64::from(u.index) == o\n});";
    assert!(by_number_alone(broken).is_empty());
    let dead = "let b = sim.buildings.iter()\n    .position(|b| i64::from(b.owner) == who && i64::from(b.index) == o);";
    assert_eq!(by_number_alone(dead).len(), 1);
    let comment = "// i64::from(b.index) == o is what it used to say";
    assert!(by_number_alone(comment).is_empty());
}

#[test]
fn no_dumped_record_is_linked_by_its_number_alone() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/diff");
    let mut files = Vec::new();
    sources(&dir, &mut files);
    files.sort();
    let mut used = vec![false; BY_NUMBER_ALONE.len()];
    let mut unlisted = Vec::new();
    for path in &files {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "slots.rs" {
            continue;
        }
        let text = std::fs::read_to_string(path).expect("read");
        for (line, code) in by_number_alone(&text) {
            match BY_NUMBER_ALONE
                .iter()
                .position(|(f, fragment, _)| *f == name && code.contains(fragment))
            {
                Some(row) => used[row] = true,
                None => unlisted.push(format!("{}:{line}: {code}", path.display())),
            }
        }
    }
    let stale: Vec<&str> = BY_NUMBER_ALONE
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|((_, fragment, _), _)| *fragment)
        .collect();
    assert!(
        unlisted.is_empty() && stale.is_empty(),
        "a dumped record's slot index is not an identity (CLAUDE.md; parked 1001).\n\
         linked by number alone: {unlisted:#?}\n\
         rows of BY_NUMBER_ALONE nothing matches: {stale:#?}\n\
         Test liveness in the same statement, go through `link_building`, or add a row \
         with the reason"
    );
}
