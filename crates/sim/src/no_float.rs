//! The hard constraint, enforced rather than trusted.
//!
//! `CLAUDE.md`: *"No floating point in the sim. Ever. Not `f32`, not `f64`,
//! not 'just for this one distance check'."* Until now that was a convention,
//! and a convention is exactly what a tired afternoon defeats — the failure
//! is silent, it survives every existing test (a float agrees with itself on
//! this machine), and it only shows up years later as a lockstep desync
//! between two players on different hardware. So this module reads the
//! simulation's own source and fails if a float appears in it.
//!
//! # What it allows, and why
//!
//! - **Test code.** `#[cfg(test)]` modules, every file the module tree
//!   mounts under one (read from `lib.rs` down, recursively: [`module_tree`]),
//!   and the files only an integration target under `tests/` mounts. A
//!   test may use `f64` freely and
//!   several do, deliberately: `movement.rs` checks `find_angle` against
//!   `atan2`, and `combat.rs` checks the software float against the host's.
//!   Those are *oracles*, and an oracle written in the same arithmetic as
//!   the thing it checks would prove nothing.
//! - **Comments and string literals.** The documents these files carry talk
//!   about `f32` constantly — the original's `0.005f` accumulator, the one
//!   `f32` in the flight-time formula — and prose is not arithmetic.
//! - **Identifiers that merely contain the letters**, such as
//!   `combat::f32_sqrt`. That function is the point of the rule rather than
//!   a breach of it: it reproduces an IEEE single-precision square root in
//!   `u64` mantissa arithmetic (`combat::F32` is `{ mant: u64, exp: i32 }`),
//!   because the original computes one and we must land on the same integer
//!   without ever holding a float.
//!
//! Everything else is a failure, including a lone `1.0` literal — a float
//! literal is a float even before it is named.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// Every `.rs` file under the crate's `src/`, recursively, with whether it is
/// test code and its text: `(path relative to src/, test, text)`, sorted.
///
/// **Recursive since item 4008** (`docs/STRUCTURE.md` §5, move 1): the scan read
/// the flat `src/` alone, so `do_attack/live.rs` was never checked and every
/// file a structure landing moves into a directory would have left the
/// guard in silence.
pub(crate) fn source_files() -> Vec<(String, bool, String)> {
    let src = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
    let tests = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
    classify(src, Some(tests))
}

/// The files of `src`, each marked test or production by the module tree:
///
/// - a file the tree reaches from `lib.rs` is test when it, or a module
///   above it, is declared under `#[cfg(test)]` (an out-of-line `mod x;`
///   after the attribute, or one inside an inline `#[cfg(test)]` block);
/// - a file the tree does not reach, but which an integration target under
///   `tests` mounts with `#[path]`, is test — it is compiled only there;
/// - a file nothing reaches is production: it is scanned, never exempted.
pub(crate) fn classify(src: &Path, tests: Option<&Path>) -> Vec<(String, bool, String)> {
    let tree = module_tree(src);
    let mounted_by_tests = match tests.filter(|t| t.is_dir()) {
        Some(tests) => walk_modules(
            src,
            walk(tests)
                .into_iter()
                .map(|(file, _, _)| (tests.join(file), tests.to_path_buf(), true))
                .collect(),
        ),
        None => BTreeMap::new(),
    };
    walk(src)
        .into_iter()
        .map(|(rel, _, text)| {
            let test = match tree.get(&rel) {
                Some(&t) => t,
                None => mounted_by_tests.contains_key(&rel),
            };
            (rel, test, text)
        })
        .collect()
}

/// Every `.rs` file under `dir`, recursively, as `(relative path, false,
/// text)`, sorted; the path is `/`-separated.
fn walk(dir: &Path) -> Vec<(String, bool, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap_or_else(|e| panic!("{}: {e}", d.display())) {
            let path = entry.expect("a readable entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).expect("a readable source file");
                let rel = relative(dir, &path).expect("under its own walk");
                out.push((rel, false, text));
            }
        }
    }
    out.sort();
    out
}

/// A path's components with `.` and `..` resolved lexically.
fn lexical(path: &Path) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            other => parts.push(other.as_os_str().to_string_lossy().into_owned()),
        }
    }
    parts
}

/// `path` relative to `base`, `/`-separated; `None` when it is not under
/// `base`.
fn relative(base: &Path, path: &Path) -> Option<String> {
    let parts = lexical(path);
    parts
        .strip_prefix(lexical(base).as_slice())
        .map(|rest| rest.join("/"))
        .filter(|r| !r.is_empty())
}

/// One out-of-line `mod name;` in a source text.
pub(crate) struct ModDecl {
    pub(crate) name: String,
    /// Declared under `#[cfg(test)]`, on itself or on an inline module
    /// around it.
    pub(crate) test: bool,
    /// Its `#[path = "…"]`, when it has one.
    pub(crate) path: Option<String>,
}

/// The out-of-line module declarations of one file: `mod x;` with any
/// visibility, its attributes read from the lines above it (attributes and
/// comments, contiguous). Comments and strings do not declare.
pub(crate) fn mod_decls(text: &str) -> Vec<ModDecl> {
    let code = code_only(text);
    let live = without_test_modules(&code);
    let raw: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::new();
    let mut offset = 0;
    for (i, line) in code.split('\n').enumerate() {
        let start = offset;
        offset += line.len() + 1;
        let t = line.trim();
        let rest = ["pub(crate) ", "pub(super) ", "pub "]
            .iter()
            .find_map(|v| t.strip_prefix(v))
            .unwrap_or(t);
        let Some(name) = rest
            .strip_prefix("mod ")
            .and_then(|r| r.strip_suffix(';'))
            .map(str::trim)
            .filter(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
        else {
            continue;
        };
        // Inside a stripped `#[cfg(test)] mod x { … }`: test code.
        let mut test = live.as_bytes()[start..start + line.len()]
            .iter()
            .all(|&c| c == b' ');
        let mut path = None;
        for above in raw[..i].iter().rev().map(|l| l.trim()) {
            if above.starts_with("//") {
                continue;
            }
            if !above.starts_with("#[") {
                break;
            }
            if above == "#[cfg(test)]" {
                test = true;
            }
            if let Some(p) = above
                .strip_prefix("#[path")
                .and_then(|r| r.split('"').nth(1))
            {
                path = Some(p.to_string());
            }
        }
        out.push(ModDecl {
            name: name.to_string(),
            test,
            path,
        });
    }
    out
}

/// The crate's module tree from `src/lib.rs`: every file it reaches,
/// relative to `src`, and whether it is test code. A child of `lib.rs`, a
/// `mod.rs` or a `#[path]` file resolves beside it; a child of `foo.rs`
/// resolves in `foo/`; a `#[path]` is relative to the declaring file's
/// directory — the rules `rustc` resolves them by (inline modules that
/// declare out-of-line children are not followed; the crate has none).
pub(crate) fn module_tree(src: &Path) -> BTreeMap<String, bool> {
    walk_modules(src, vec![(src.join("lib.rs"), src.to_path_buf(), false)])
}

/// The files under `src` that the module trees rooted at `roots` reach —
/// each root `(file, the directory its children resolve in, test)` — and
/// whether each is test code. A root outside `src` (an integration target)
/// is followed but not listed.
fn walk_modules(src: &Path, roots: Vec<(PathBuf, PathBuf, bool)>) -> BTreeMap<String, bool> {
    let mut out = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut stack = roots;
    while let Some((file, children, test)) = stack.pop() {
        if !seen.insert(lexical(&file)) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if let Some(rel) = relative(src, &file) {
            out.insert(rel, test);
        }
        let dir = file.parent().unwrap_or(src).to_path_buf();
        for decl in mod_decls(&text) {
            let test = test || decl.test;
            if let Some(p) = decl.path {
                let at = dir.join(p);
                let beside = at.parent().unwrap_or(&dir).to_path_buf();
                stack.push((at, beside, test));
                continue;
            }
            let flat = children.join(format!("{}.rs", decl.name));
            let nested = children.join(&decl.name).join("mod.rs");
            if flat.exists() {
                stack.push((flat, children.join(&decl.name), test));
            } else if nested.exists() {
                stack.push((nested, children.join(&decl.name), test));
            }
        }
    }
    out
}

/// The files of the module `name` that the crate root `root` declares
/// (`mod name;`, or with a `#[path]`), relative to `under`: the module's
/// own file wherever `rustc` finds it — `name.rs` or `name/mod.rs` beside
/// the root, or its `#[path]` — and every file its subtree mounts. Empty
/// when `root` declares no such module.
///
/// **Loop 4044** (the seventy-sixth pass): seven readers knew the sweep by
/// its directory's name, `sweep/`, and none failed when item 4032 moved
/// `sweep/mod.rs` out to `sweep.rs` — the module file left every skip and
/// read as production until the lane named each reader by hand. A reader
/// that asks the module tree follows the module however its files lie.
pub(crate) fn module_files(under: &Path, root: &Path, name: &str) -> BTreeSet<String> {
    let Ok(text) = std::fs::read_to_string(root) else {
        return BTreeSet::new();
    };
    let Some(decl) = mod_decls(&text).into_iter().find(|d| d.name == name) else {
        return BTreeSet::new();
    };
    let dir = root.parent().unwrap_or(under).to_path_buf();
    let (file, children) = match decl.path {
        Some(p) => {
            let at = dir.join(p);
            let beside = at.parent().unwrap_or(&dir).to_path_buf();
            (at, beside)
        }
        None => {
            let flat = dir.join(format!("{name}.rs"));
            let file = if flat.exists() {
                flat
            } else {
                dir.join(name).join("mod.rs")
            };
            (file, dir.join(name))
        }
    };
    walk_modules(under, vec![(file, children, true)])
        .into_keys()
        .collect()
}

/// The floors on the sweep's two halves (Loop 4044): 150 files in `lib.rs`'s
/// `sweep` module and 1,309 in the integration targets' on 2026-11-09.
const SWEEP_SRC_FILES_AT_LEAST: usize = 120;
const SWEEP_TEST_FILES_AT_LEAST: usize = 1000;

/// **The sweep's files**, relative to the crate's directory: `lib.rs`'s
/// `sweep` module under `src/` (the sweeps that need crate-private APIs)
/// and each integration target's `sweep` module under `tests/` (item 4009
/// moved the standalone sweeps there). The readers that leave the sweep
/// out — the hex-constant scan, the spawn-site walk, the classification
/// check — ask for this set instead of a directory's name, and the set's
/// size is asserted here, once for all of them: a skip that skips nothing
/// is a guard reading production as test or test as production in
/// silence (Loop 4044).
pub(crate) fn sweep_files() -> BTreeSet<String> {
    let krate = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (src, tests) = (krate.join("src"), krate.join("tests"));
    let in_src: BTreeSet<String> = module_files(&src, &src.join("lib.rs"), "sweep")
        .into_iter()
        .map(|rel| format!("src/{rel}"))
        .collect();
    let mut in_tests = BTreeSet::new();
    let mut targets: Vec<PathBuf> = std::fs::read_dir(&tests)
        .map(|d| {
            d.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "rs"))
                .collect()
        })
        .unwrap_or_default();
    targets.sort();
    for target in &targets {
        in_tests.extend(
            module_files(&tests, target, "sweep")
                .into_iter()
                .map(|rel| format!("tests/{rel}")),
        );
    }
    assert!(
        in_src.len() >= SWEEP_SRC_FILES_AT_LEAST && in_tests.len() >= SWEEP_TEST_FILES_AT_LEAST,
        "the sweep's module tree holds {} files under src/ (floor {SWEEP_SRC_FILES_AT_LEAST}) \
         and {} under tests/ (floor {SWEEP_TEST_FILES_AT_LEAST}) — the `sweep` module moved \
         or was renamed, and every reader that leaves the sweep out now skips too little \
         (Loop 4044)",
        in_src.len(),
        in_tests.len()
    );
    in_src.into_iter().chain(in_tests).collect()
}

/// The simulation's production files with their text: what the float scan
/// reads.
fn sources() -> Vec<(String, String)> {
    source_files()
        .into_iter()
        .filter(|(_, test, _)| !test)
        .map(|(name, _, text)| (name, text))
        .collect()
}

/// Blanks out everything that is not code: line and block comments, string
/// literals, char literals. Byte positions are preserved so a line number
/// still means something.
pub(crate) fn code_only(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = vec![b' '; b.len()];
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\n' => {
                out[i] = b'\n';
                i += 1;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                // Rust block comments nest.
                let mut depth = 1;
                i += 2;
                while i < b.len() && depth > 0 {
                    if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if b[i] == b'*' && i + 1 < b.len() && b[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        if b[i] == b'\n' {
                            out[i] = b'\n';
                        }
                        i += 1;
                    }
                }
            }
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    if i < b.len() && b[i] == b'\n' {
                        out[i] = b'\n';
                    }
                    i += 1;
                }
                i += 1;
            }
            // A char literal, but not a lifetime (`'a`) — a lifetime has no
            // closing quote, so require one within three bytes.
            b'\'' if b.get(i + 1).is_some_and(|&c| c != b'\\') && b.get(i + 2) == Some(&b'\'') => {
                i += 3;
            }
            c => {
                out[i] = c;
                i += 1;
            }
        }
    }
    String::from_utf8(out).expect("ASCII sources stay valid")
}

/// Blanks out every `#[cfg(test)]` item: the attribute, and the item that
/// follows it — a braced block to its matching `}`, or an item that ends at
/// a `;` before any brace (`mod tests;`, `use …;`, `const …;`).
///
/// **The `;` since item 4008**: the attribute blanked to the next braced
/// block wherever it was, so `#[cfg(test)] mod tests;` hid the production
/// item after it from the scan — the shape `docs/STRUCTURE.md` §5's third
/// move writes into every file whose tail it moves out.
pub(crate) fn without_test_modules(code: &str) -> String {
    let mut out = code.as_bytes().to_vec();
    let mut from = 0;
    while let Some(rel) = code[from..].find("#[cfg(test)]") {
        let at = from + rel;
        // The item that follows the attribute: up to a `;` outside brackets
        // when one comes before the first `{`, else from that `{` to its
        // matching `}`.
        let mut i = at + "#[cfg(test)]".len();
        let mut nest = 0i32;
        while i < out.len() && out[i] != b'{' && !(out[i] == b';' && nest == 0) {
            match out[i] {
                b'(' | b'[' => nest += 1,
                b')' | b']' => nest -= 1,
                _ => {}
            }
            i += 1;
        }
        if i < out.len() && out[i] == b';' {
            for byte in &mut out[at..=i] {
                if *byte != b'\n' {
                    *byte = b' ';
                }
            }
            from = i + 1;
            continue;
        }
        let mut depth = 0;
        let mut end = out.len();
        let mut j = i;
        while j < out.len() {
            match out[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = j + 1;
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        for byte in &mut out[at..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
        from = end;
    }
    String::from_utf8(out).expect("ASCII sources stay valid")
}

/// Every float mention in `code`, as `(line, the offending token)`.
///
/// A float is either the bare type name `f32`/`f64` as a whole identifier, or
/// a literal suffix — `0f64`, `1.0f32` — or a decimal literal, `1.0`. An
/// identifier that merely contains the letters (`f32_sqrt`, `to_f64_ish`) is
/// not one, which is what `is_ident_byte` on both sides tests.
fn floats(code: &str) -> Vec<(usize, String)> {
    fn is_ident_byte(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_'
    }
    let b = code.as_bytes();
    let mut out = Vec::new();
    let line_of = |at: usize| code[..at].bytes().filter(|&c| c == b'\n').count() + 1;
    for name in ["f32", "f64"] {
        let mut from = 0;
        while let Some(rel) = code[from..].find(name) {
            let at = from + rel;
            from = at + name.len();
            let after_ok = b.get(at + name.len()).is_none_or(|&c| !is_ident_byte(c));
            if !after_ok {
                continue;
            }
            match at.checked_sub(1).map(|p| b[p]) {
                // `f32` standing alone: the type.
                None => out.push((line_of(at), name.to_string())),
                Some(c) if !is_ident_byte(c) => out.push((line_of(at), name.to_string())),
                // `0f64`, `1.5f32`: a suffixed literal.
                Some(c) if c.is_ascii_digit() => {
                    out.push((line_of(at), format!("literal suffix {name}")));
                }
                // `f32_sqrt`, `some_f64_name`: an identifier. Allowed.
                Some(_) => {}
            }
        }
    }
    // A decimal literal needs no suffix to be a float: `let x = 1.0;`.
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                i += 1;
            }
            // `1.0` is a float; `1..2` is a range and `x.0` is a field.
            if b.get(i) == Some(&b'.') && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
                let before = start.checked_sub(1).map(|p| b[p]);
                if !before.is_some_and(|c| is_ident_byte(c) || c == b'.') {
                    out.push((line_of(start), "decimal literal".to_string()));
                }
            }
            continue;
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The production files the float scan read on 2026-11-09: 243.
    const SCANNED_AT_LEAST: usize = 215;

    #[test]
    fn the_simulation_holds_no_floating_point() {
        let mut bad: Vec<String> = Vec::new();
        let mut scanned = 0;
        for (name, text) in sources() {
            scanned += 1;
            let code = without_test_modules(&code_only(&text));
            for (line, what) in floats(&code) {
                bad.push(format!("{name}:{line}: {what}"));
            }
        }
        // Loop 4041: SCANNED_AT_LEAST production files, a tenth under
        // the day's count; it was 15 against hundreds, so a scan that lost
        // nine tenths of the crate passed.
        assert!(
            scanned >= SCANNED_AT_LEAST,
            "only {scanned} sources scanned, under the floor of {SCANNED_AT_LEAST} — has \
             the crate moved?"
        );
        assert!(
            bad.is_empty(),
            "floating point in the simulation ({} places). CLAUDE.md: the sim \
             is integers at the original's own scales; if the original really \
             holds a float there, reproduce it exactly the way \
             `combat::flight_time` does — in integers — and say so in the \
             mechanic's document.\n  {}",
            bad.len(),
            bad.join("\n  ")
        );
    }

    /// **The scan reaches every file the crate is built from** (item 4008,
    /// `docs/STRUCTURE.md` §5, move 1): a module that moves into a directory
    /// must not leave the guard. Made to fail first on the flat scan,
    /// which read `src/` alone and missed `do_attack/live.rs`.
    #[test]
    fn the_scan_reaches_a_nested_module() {
        let names: Vec<String> = sources().into_iter().map(|(n, _)| n).collect();
        assert!(
            names.iter().any(|n| n == "orders/do_attack/live.rs"),
            "the float scan does not read orders/do_attack/live.rs; it read {} files",
            names.len()
        );
    }

    /// **The module tree on a synthetic nested crate** (item 4008, before
    /// item 4010 moves anything): a `mod.rs` directory, a `foo.rs` with a
    /// `foo/` child, a `#[path]` module, a `#[cfg(test)]` directory whose
    /// children inherit it, a test declared inside an inline `#[cfg(test)]`
    /// block, a file only `tests/` mounts, and an orphan, which is scanned.
    #[test]
    fn the_module_tree_reads_a_nested_crate() {
        let root = std::env::temp_dir().join(format!(
            "attrition-module-tree-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let files: &[(&str, &str)] = &[
            (
                "src/lib.rs",
                "pub mod core;\nmod units;\n#[cfg(test)]\nmod sweep;\n\
                 #[cfg(test)]\n#[path = \"support/env.rs\"]\npub(crate) mod testenv;\n\
                 // mod commented;\n#[cfg(test)]\nmod tests {\n    mod inner;\n}\n",
            ),
            (
                "src/core/mod.rs",
                "pub mod tick;\n#[cfg(test)]\nmod tests;\n",
            ),
            ("src/core/tick.rs", "pub fn t() {}\n"),
            ("src/core/tests.rs", "fn x() {}\n"),
            ("src/units.rs", "/// The think.\npub(crate) mod think;\n"),
            ("src/units/think.rs", "fn t() {}\n"),
            ("src/sweep/mod.rs", "mod one;\n"),
            ("src/sweep/one.rs", "fn s() {}\n"),
            ("src/sweep/standalone.rs", "fn s() {}\n"),
            ("src/support/env.rs", "fn e() {}\n"),
            ("src/inner.rs", "fn i() {}\n"),
            ("src/commented.rs", "fn c() {}\n"),
            (
                "tests/sweep.rs",
                "pub use sim::*;\n#[path = \"../src/sweep/standalone.rs\"]\nmod sweep;\n",
            ),
        ];
        for (path, text) in files {
            let at = root.join(path);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, text).unwrap();
        }
        let got: Vec<(String, bool)> = classify(&root.join("src"), Some(&root.join("tests")))
            .into_iter()
            .map(|(rel, test, _)| (rel, test))
            .collect();
        std::fs::remove_dir_all(&root).unwrap();
        let want: Vec<(String, bool)> = [
            // Nothing declares it (the declaration is a comment): scanned.
            ("commented.rs", false),
            ("core/mod.rs", false),
            ("core/tests.rs", true),
            ("core/tick.rs", false),
            // `mod inner;` inside an inline `#[cfg(test)]` block resolves
            // beside lib.rs here, and is test.
            ("inner.rs", true),
            ("lib.rs", false),
            ("support/env.rs", true),
            ("sweep/mod.rs", true),
            ("sweep/one.rs", true),
            ("sweep/standalone.rs", true),
            ("units.rs", false),
            ("units/think.rs", false),
        ]
        .iter()
        .map(|(r, t)| (r.to_string(), *t))
        .collect();
        assert_eq!(got, want);
    }

    /// **The sweep's set follows the module, not the directory** (Loop
    /// 4044): the same `sweep` module laid out as `sweep/mod.rs` and as
    /// `sweep.rs` beside `sweep/` yields the same files, its own file
    /// among them, and an integration target's `#[path]` module is
    /// followed into its directory. A directory-name reader read
    /// `sweep.rs` as production on the second layout.
    #[test]
    fn a_module_s_files_are_read_from_the_tree_in_either_layout() {
        let root = std::env::temp_dir().join(format!(
            "attrition-module-files-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, text: &str| {
            let at = root.join(path);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, text).unwrap();
        };
        write("src/lib.rs", "pub mod core;\n#[cfg(test)]\nmod sweep;\n");
        write("src/core.rs", "fn c() {}\n");
        write("src/sweep/mod.rs", "mod one;\n");
        write("src/sweep/one.rs", "fn s() {}\n");
        write(
            "tests/sweep_a.rs",
            "#[path = \"sweep/standalone.rs\"]\nmod sweep;\nmod other;\n",
        );
        write(
            "tests/sweep/standalone.rs",
            "#[path = \"two.rs\"]\nmod two;\n",
        );
        write("tests/sweep/two.rs", "fn t() {}\n");
        let src = root.join("src");
        let nested = module_files(&src, &src.join("lib.rs"), "sweep");
        std::fs::rename(src.join("sweep/mod.rs"), src.join("sweep.rs")).unwrap();
        let flat = module_files(&src, &src.join("lib.rs"), "sweep");
        let tests = root.join("tests");
        let target = module_files(&tests, &tests.join("sweep_a.rs"), "sweep");
        let none = module_files(&src, &src.join("lib.rs"), "absent");
        std::fs::remove_dir_all(&root).unwrap();
        let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
        assert_eq!(nested, set(&["sweep/mod.rs", "sweep/one.rs"]));
        assert_eq!(flat, set(&["sweep.rs", "sweep/one.rs"]));
        assert_eq!(target, set(&["sweep/standalone.rs", "sweep/two.rs"]));
        assert!(none.is_empty());
    }

    /// The real tree: the files a structure landing would move are read,
    /// and the test files are recognised from the tree rather than a list.
    #[test]
    fn the_real_tree_classifies_its_test_files() {
        let all = source_files();
        let test = |name: &str| {
            all.iter()
                .find(|(n, _, _)| n == name)
                .map(|(_, t, _)| *t)
                .unwrap_or_else(|| panic!("{name} is not in the walk"))
        };
        for name in [
            "harness_tests.rs",
            "cities_tests.rs",
            "no_float.rs",
            "docs_guard.rs",
            "soak.rs",
            "sweep.rs",
            "test_support/install.rs",
        ] {
            assert!(test(name), "{name} read as production");
        }
        for name in [
            "lib.rs",
            "orders.rs",
            "orders/do_attack.rs",
            "orders/do_attack/live.rs",
        ] {
            assert!(!test(name), "{name} read as test");
        }
        // The sweep by its module, not its directory (Loop 4044): this
        // read `sweep/` by name, and passed on nothing when `sweep.rs` left it.
        let set = sweep_files();
        let sweep: Vec<&(String, bool, String)> = all
            .iter()
            .filter(|(n, _, _)| set.contains(&format!("src/{n}")))
            .collect();
        assert!(
            sweep.len() >= SWEEP_SRC_FILES_AT_LEAST,
            "{} of the sweep's files are in the walk",
            sweep.len()
        );
        assert!(
            sweep
                .iter()
                .any(|(n, _, _)| n == "sweep.rs" || n == "sweep/mod.rs"),
            "the sweep's own module file is not in its set"
        );
        assert!(
            sweep.iter().all(|(_, t, _)| *t),
            "a sweep read as production"
        );
    }

    /// **No source file is an orphan**: every `.rs` under `src/` is reached
    /// by `lib.rs`'s module tree or an integration target's. A file a move
    /// left without its `mod` line is code the compiler never sees, and the
    /// float scan would read it as production forever.
    #[test]
    fn every_source_file_is_mounted() {
        let src = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let tests = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let mut reached = module_tree(src);
        reached.extend(walk_modules(
            src,
            walk(tests)
                .into_iter()
                .map(|(f, _, _)| (tests.join(f), tests.to_path_buf(), true))
                .collect(),
        ));
        let orphans: Vec<String> = walk(src)
            .into_iter()
            .map(|(rel, _, _)| rel)
            .filter(|rel| !reached.contains_key(rel))
            .collect();
        assert!(
            orphans.is_empty(),
            "sources no module tree mounts: {orphans:?}"
        );
    }

    /// The scanner itself, on text it should and should not object to.
    #[test]
    fn the_scanner_knows_a_float_from_a_name_for_one() {
        // Allowed: an identifier containing the letters, a comment, a string.
        let ok = "fn f32_sqrt(n: i64) -> u64 { /* the f32 mantissa */ let s = \"1.0f32\"; 0 }";
        assert!(floats(&code_only(ok)).is_empty(), "{:?}", floats(ok));
        // Caught: the bare type, a suffixed literal, a bare decimal.
        for src in ["let x: f64 = 0;", "let x = 3f32;", "let x = 1.5;"] {
            assert_eq!(floats(&code_only(src)).len(), 1, "{src}");
        }
        // Not floats: a range, a tuple field, a version-like path.
        for src in ["for i in 1..2 {}", "let a = t.0;", "let b = x.0.1;"] {
            assert!(floats(&code_only(src)).is_empty(), "{src}");
        }
        // A `#[cfg(test)]` module is exempt, and only up to its own brace.
        let mixed = "fn a() -> i32 { 1 }\n#[cfg(test)]\nmod t { fn b() { let x = 1.0; } }\nfn c() { let y: f64 = 0; }";
        let stripped = without_test_modules(&code_only(mixed));
        let found = floats(&stripped);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].1, "f64");
        // An item that ends at its `;` takes nothing after it along: the
        // shape a tail moved to `foo/tests.rs` leaves behind.
        for decl in [
            "#[cfg(test)]\nmod tests;",
            "#[cfg(test)]\nuse std::f64::consts;",
            "#[cfg(test)]\nconst N: [u8; 2] = [0; 2];",
        ] {
            let src = format!("{decl}\nfn c() {{ let y: f64 = 0; }}");
            let found = floats(&without_test_modules(&code_only(&src)));
            assert_eq!(found.len(), 1, "{src}: {found:?}");
        }
    }
}
