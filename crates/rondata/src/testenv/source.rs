//! Logical golden source for source-scanning guards. Read the current tree, as
//! floors' guards already do; release validation freezes that tree during a run.
use std::collections::{BTreeMap, BTreeSet};

fn expand(parent: &str, cases: &BTreeMap<String, String>) -> String {
    let mut used = BTreeSet::new();
    let lines: Vec<_> = parent.lines().collect();
    let mut out = String::new();
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        if let Some(name) = line
            .strip_prefix("fn ")
            .and_then(|s| s.strip_suffix("() {"))
            && let Some(call) = lines.get(at + 1).map(|s| s.trim())
            && call.starts_with("cases_")
        {
            let (module, target) = call.split_once("::").expect("case call");
            assert_eq!(
                target,
                format!("{name}();"),
                "case wrapper changed its target"
            );
            assert_eq!(
                lines.get(at + 2),
                Some(&"}"),
                "case wrapper is not one call"
            );
            let source = cases.get(module).expect("case module is not registered");
            assert!(
                used.insert((module.to_string(), name.to_string())),
                "duplicate case wrapper"
            );
            let head = format!("pub(super) fn {name}() {{");
            let body: Vec<_> = source.lines().collect();
            let starts: Vec<_> = body
                .iter()
                .enumerate()
                .filter(|(_, l)| **l == head)
                .map(|(i, _)| i)
                .collect();
            assert_eq!(starts.len(), 1, "case body must exist exactly once");
            let start = starts[0] + 1;
            let end = start
                + body[start..]
                    .iter()
                    .position(|l| *l == "}")
                    .expect("case closing brace");
            out.push_str(line);
            out.push('\n');
            for line in &body[start..=end] {
                out.push_str(line);
                out.push('\n');
            }
            at += 3;
        } else {
            out.push_str(line);
            out.push('\n');
            at += 1;
        }
    }
    let mut defined = BTreeSet::new();
    for (module, source) in cases {
        for name in source.lines().filter_map(|l| {
            l.strip_prefix("pub(super) fn ")
                .and_then(|s| s.strip_suffix("() {"))
        }) {
            assert!(
                defined.insert((module.clone(), name.to_string())),
                "duplicate case body"
            );
        }
    }
    assert_eq!(used, defined, "unwrapped or missing case body");
    out
}

pub(crate) fn golden() -> (String, Vec<String>) {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff"));
    let parent = std::fs::read_to_string(root.join("golden.rs")).expect("golden source");
    let mut cases = BTreeMap::new();
    for module in parent.lines().filter_map(|l| {
        l.strip_prefix("mod cases_")
            .and_then(|s| s.strip_suffix(';'))
    }) {
        assert!(
            !module.is_empty()
                && module
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        );
        let module = format!("cases_{module}");
        let source = std::fs::read_to_string(root.join("golden").join(format!("{module}.rs")))
            .expect("registered case source");
        assert!(
            cases.insert(module, source).is_none(),
            "duplicate case module"
        );
    }
    assert!(!cases.is_empty(), "golden case modules disappeared");
    let paths = cases
        .keys()
        .map(|m| format!("diff/golden/{m}.rs"))
        .collect();
    (expand(&parent, &cases), paths)
}

#[test]
fn case_expansion_preserves_checks_and_rejects_missing_or_misdirected_bodies() {
    let parent = "#[test]\nfn check() {\n    cases_setup::check();\n}\n";
    let original =
        "#[test]\nfn check() {\n    let _pins = Pins::hold();\n    pin_eq!(got, want);\n}\n";
    let body = original.replace("#[test]\nfn", "pub(super) fn");
    let cases = BTreeMap::from([("cases_setup".to_string(), body)]);
    assert_eq!(expand(parent, &cases), original);
    for bad in [
        parent.replace("::check()", "::wrong()"),
        parent.repeat(2),
        String::new(),
    ] {
        assert!(std::panic::catch_unwind(|| expand(&bad, &cases)).is_err());
    }
    assert!(std::panic::catch_unwind(|| expand(parent, &BTreeMap::new())).is_err());
    let duplicate = BTreeMap::from([("cases_setup".to_string(), cases["cases_setup"].repeat(2))]);
    assert!(std::panic::catch_unwind(|| expand(parent, &duplicate)).is_err());
}
