//! **A tree's corpus is what that tree's own ledger names.**
//!
//! The kept captures live outside the repository, in archives every
//! worktree on the machine shares (`RON_GAMELOG_DIR`, `RON_GOLDEN_DIR`). A
//! test that lists such a directory and pins what it finds reads the
//! machine, not the tree: on 2026-11-13 one builder filed run1575 and
//! run1576 into the shared archives, and until its branch (with its
//! re-pins) merged, four other gates went red on two such tests
//! (`diff::checksum_lines::packets::every_packet_is_wired_to_its_game_or_named`
//! and `diff::transfer::the_held_leaves_transfer_on_the_original_s_printed_inputs`)
//! whose fix was on somebody else's branch.
//!
//! A worker that files a capture writes its `## <run> — …` section in
//! `docs/RUNS.md` on its own branch. So a disk walker that pins what it
//! finds asks [`Ledger::in_tree`] of each entry: from run
//! [`RUNS_HEADED_FROM`] on, an entry counts only when this tree's ledger
//! heads a section with its exact run token; an entry the rule leaves out
//! is skipped and named ([`listed`] prints it), never red.

use std::collections::BTreeSet;
use std::path::Path;

/// **The first run the ledger heads one section per capture from.**
/// Below it the ledger documents runs in grouped sections (`## runs
/// 23–31 — …`, `## run1510 to run1510d — …`, `## run1545 and run1545b
/// — …`), so a run there with no heading of its own is not unfiled, and
/// everything on disk counts as it did before this rule.
///
/// Measured at `331d85b01` against the shared archives (1,201
/// `gamelog-*.txt`, 1,193 run tokens; 109 packets, 107 run tokens):
/// reading every run token in a heading's subject (the text before its
/// first ` — `), as [`Ledger::parse`] does, 67 gamelog tokens and 5
/// packet tokens have no heading, the highest `run1042` (inside `##
/// run1040–run1043`); reading only a heading's leading token, 103 and 6
/// have none, the highest `run1545b` (in `## run1545 and run1545b`). No
/// token from 1546 on lacks its heading under either reading, so 1546 is
/// the first run past every grouped section.
pub const RUNS_HEADED_FROM: u32 = 1546;

/// **A disk entry's run token**: the leading `run<digits><letters>` of its
/// name, after a `gamelog-` or `rontrace-` prefix, with its number —
/// `gamelog-run1575-sea-take.txt` → `("run1575", 1575)`, a packet
/// directory `run1559b` → `("run1559b", 1559)`. `None` for a name with no
/// run token (`ch95pkt`, `gamelog-fuzz-424242-early.txt`,
/// `gamelog-scratch-…`), which the rule counts as before.
pub fn run_token(name: &str) -> Option<(&str, u32)> {
    let rest = ["gamelog-", "rontrace-"]
        .iter()
        .find_map(|p| name.strip_prefix(p))
        .unwrap_or(name);
    let after = rest.strip_prefix("run")?;
    let digits = after.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let letters = after[digits..]
        .bytes()
        .take_while(u8::is_ascii_lowercase)
        .count();
    let number = after[..digits].parse().ok()?;
    Some((&rest[..3 + digits + letters], number))
}

/// The run tokens `docs/RUNS.md` heads a section for.
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    headed: BTreeSet<String>,
}

impl Ledger {
    /// **Reads a ledger's text**: every `## ` heading's subject — the text
    /// before its first ` — `, or the whole heading without one — and every
    /// `run<digits><letters>` in it. `## run1576 — run1575's game …` heads
    /// run1576 alone; `## run1507 and run1507b — …` heads both. A range
    /// (`run1510 to run1510d`, `runs 23–31`) is not expanded: such headings
    /// are all below [`RUNS_HEADED_FROM`].
    pub fn parse(text: &str) -> Self {
        let mut headed = BTreeSet::new();
        for line in text.lines() {
            let Some(heading) = line.strip_prefix("## ") else {
                continue;
            };
            let subject = heading.split(" \u{2014} ").next().unwrap_or(heading);
            let bytes = subject.as_bytes();
            let mut at = 0;
            while let Some(i) = subject[at..].find("run") {
                let start = at + i;
                at = start + 3;
                if start > 0 && bytes[start - 1].is_ascii_alphanumeric() {
                    continue;
                }
                if let Some((token, _)) = run_token(&subject[start..]) {
                    at = start + token.len();
                    headed.insert(token.to_string());
                }
            }
        }
        Self { headed }
    }

    /// The ledger at `path`.
    pub fn read(path: &Path) -> std::io::Result<Self> {
        Ok(Self::parse(&std::fs::read_to_string(path)?))
    }

    /// **This tree's ledger**: the `docs/RUNS.md` of the checkout this
    /// crate was built from, read once.
    pub fn of_tree() -> &'static Self {
        static TREE: std::sync::OnceLock<Ledger> = std::sync::OnceLock::new();
        TREE.get_or_init(|| {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/RUNS.md");
            Self::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        })
    }

    /// Whether the ledger heads a section for exactly `token`.
    pub fn heads(&self, token: &str) -> bool {
        self.headed.contains(token)
    }

    /// **Whether a disk entry is in this tree's corpus**: a name with no run
    /// token, or a run below [`RUNS_HEADED_FROM`], always; from there on,
    /// only when the ledger heads its exact token. `name` is a file or
    /// directory name, or a bare token.
    pub fn in_tree(&self, name: &str) -> bool {
        match run_token(name) {
            None => true,
            Some((_, n)) if n < RUNS_HEADED_FROM => true,
            Some((token, _)) => self.heads(token),
        }
    }

    /// `names` split into those [`Ledger::in_tree`] keeps and the run
    /// tokens of those it leaves out (sorted, once each), the unfiled
    /// printed on stderr under `what`.
    pub fn split(&self, what: &str, names: Vec<String>) -> (Vec<String>, Vec<String>) {
        let (kept, out): (Vec<String>, Vec<String>) =
            names.into_iter().partition(|n| self.in_tree(n));
        let unfiled: BTreeSet<String> = out
            .iter()
            .filter_map(|n| run_token(n).map(|(t, _)| t.to_string()))
            .collect();
        let unfiled: Vec<String> = unfiled.into_iter().collect();
        if !unfiled.is_empty() {
            eprintln!(
                "{what}: unfiled on disk, not in this tree's docs/RUNS.md: {}",
                unfiled.join(", ")
            );
        }
        (kept, unfiled)
    }
}

/// One directory's entries a walker counts, and the run tokens it skipped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listed {
    /// The entries' names that are in the tree's corpus, sorted.
    pub kept: Vec<String>,
    /// The run tokens on disk the ledger does not head (from
    /// [`RUNS_HEADED_FROM`] on), sorted, once each; printed, never red.
    pub unfiled: Vec<String>,
}

/// **Lists `dir`** — every entry whose path `wanted` accepts — and keeps
/// those `ledger` puts in the tree's corpus, printing the rest.
pub fn listed(
    dir: &Path,
    ledger: &Ledger,
    wanted: impl Fn(&Path) -> bool,
) -> std::io::Result<Listed> {
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| wanted(p))
        .filter_map(|p| p.file_name()?.to_str().map(str::to_string))
        .collect();
    names.sort();
    let (kept, unfiled) = ledger.split(&dir.display().to_string(), names);
    Ok(Listed { kept, unfiled })
}

/// A `gamelog-*.txt`: the kept dumps' walkers' filter.
pub fn is_gamelog(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("gamelog-") && n.ends_with(".txt"))
}

/// A packet: a directory holding a `map-*/frame-snapshot.bin`
/// (`tools/explore/frame_snapshot.py`'s layout under `RON_GOLDEN_DIR`).
pub fn is_packet(p: &Path) -> bool {
    std::fs::read_dir(p).is_ok_and(|maps| {
        maps.flatten().any(|m| {
            m.file_name().to_string_lossy().starts_with("map-")
                && m.path().join("frame-snapshot.bin").is_file()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A fresh, empty directory under the system's temp directory, for a test
    /// that authors a disk; removed with its contents on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "attrition-corpus-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos())
            ));
            std::fs::create_dir_all(&dir).expect("a temp directory");
            Self(dir)
        }

        /// An empty file at `rel`, its directories made.
        fn file(&self, rel: &str) -> &Self {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&p, b"").expect("write");
            self
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const LEDGER: &str = "# Runs\n\
        ## runs 23–31 — the army's captures\n\
        ## The attrition run (run16, 2026-08-24) — the first\n\
        ## run1545 and run1545b — chapter one hundred and twenty-nine\n\
        ## run1576 — run1575's game with coverage re-armed\n\
        not a heading run1580\n";

    #[test]
    fn a_run_token_is_the_leading_run_of_a_name() {
        assert_eq!(
            run_token("gamelog-run1575-sea.txt"),
            Some(("run1575", 1575))
        );
        assert_eq!(run_token("run1559b"), Some(("run1559b", 1559)));
        assert_eq!(
            run_token("rontrace-run1077-twin.log"),
            Some(("run1077", 1077))
        );
        assert_eq!(
            run_token("gamelog-run1001c-x.txt"),
            Some(("run1001c", 1001))
        );
        assert_eq!(run_token("ch95pkt"), None);
        assert_eq!(run_token("gamelog-fuzz-424242-early.txt"), None);
        assert_eq!(run_token("gamelog-scratch-2026-08-20-2219.txt"), None);
        assert_eq!(run_token("runaway"), None);
    }

    #[test]
    fn the_ledger_heads_a_heading_s_subject_only() {
        let l = Ledger::parse(LEDGER);
        assert!(l.heads("run16") && l.heads("run1545") && l.heads("run1545b"));
        assert!(l.heads("run1576"));
        assert!(!l.heads("run1575"), "named after the dash, not headed");
        assert!(!l.heads("run1580"), "not a heading");
        assert!(!l.heads("run23"), "a range is not expanded");
    }

    #[test]
    fn from_the_threshold_on_only_a_headed_run_is_in_the_tree() {
        let l = Ledger::parse(LEDGER);
        assert!(l.in_tree("gamelog-run1576-x.txt"));
        assert!(!l.in_tree("gamelog-run1575-x.txt"));
        assert!(!l.in_tree("run1576b"), "the exact token");
        assert!(l.in_tree("gamelog-run1200-x.txt"), "below the threshold");
        assert!(l.in_tree(&format!("run{}", RUNS_HEADED_FROM - 1)));
        assert!(!l.in_tree(&format!("run{}", RUNS_HEADED_FROM)));
        assert!(l.in_tree("ch94p") && l.in_tree("gamelog-fuzz-1-x.txt"));
    }

    /// **The tree's own ledger heads every run from the threshold on that
    /// it has a section for** — read, non-empty, and holding the last
    /// capture filed before this rule (run1576, item 4206).
    #[test]
    fn the_tree_s_ledger_reads() {
        let l = Ledger::of_tree();
        assert!(l.heads("run1546") && l.heads("run1576") && l.heads("run1559b"));
        assert!(l.headed.len() > 1000, "{} headed runs", l.headed.len());
    }

    /// **A packet directory the ledger does not head is skipped and named**;
    /// with its heading it counts; a run below the threshold counts headed
    /// or not; a directory with no snapshot is not a packet.
    #[test]
    fn an_unfiled_packet_is_skipped_and_named() {
        let s = Scratch::new("packets");
        s.file("run1576/map-0/frame-snapshot.bin")
            .file("run9999/map-0/frame-snapshot.bin")
            .file("run1200/map-3/frame-snapshot.bin")
            .file("ch95pkt/map-0/frame-snapshot.bin")
            .file("run9998/map-0/other.bin");
        let got = listed(&s.0, &Ledger::parse(LEDGER), is_packet).unwrap();
        assert_eq!(got.kept, ["ch95pkt", "run1200", "run1576"]);
        assert_eq!(got.unfiled, ["run9999"]);
        let filed = format!("{LEDGER}## run9999 — x\n");
        let got = listed(&s.0, &Ledger::parse(&filed), is_packet).unwrap();
        assert_eq!(got.kept, ["ch95pkt", "run1200", "run1576", "run9999"]);
        assert!(got.unfiled.is_empty());
    }

    /// The gamelog walk the same way.
    #[test]
    fn an_unfiled_gamelog_is_skipped_and_named() {
        let s = Scratch::new("gamelogs");
        s.file("gamelog-run1576-a.txt")
            .file("gamelog-run9999-x.txt")
            .file("gamelog-run9999-y.txt")
            .file("gamelog-run1200-z.txt")
            .file("gamelog-fuzz-424242-early.txt")
            .file("rontrace-run9999.log");
        let got = listed(&s.0, &Ledger::parse(LEDGER), is_gamelog).unwrap();
        assert_eq!(
            got.kept,
            [
                "gamelog-fuzz-424242-early.txt",
                "gamelog-run1200-z.txt",
                "gamelog-run1576-a.txt"
            ]
        );
        assert_eq!(got.unfiled, ["run9999"]);
        let filed = format!("{LEDGER}## run9999 — x\n");
        let got = listed(&s.0, &Ledger::parse(&filed), is_gamelog).unwrap();
        assert_eq!(got.kept.len(), 5);
        assert!(got.unfiled.is_empty());
    }
}
