//! Logical replay source for source-scanning guards. Read the current tree, as
//! floors' guards already do; release validation freezes that tree during a run.
use std::collections::{BTreeMap, BTreeSet};

fn expand(parent: &str, cases: &BTreeMap<String, String>) -> String {
    let mut used = BTreeSet::new();
    let lines: Vec<_> = parent.lines().collect();
    let mut out = String::new();
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        let indent = &line[..line.len() - line.trim_start().len()];
        if let Some(name) = line
            .trim_start()
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
                Some(&format!("{indent}}}").as_str()),
                "case wrapper is not one call"
            );
            let source = cases.get(module).expect("case module is not registered");
            assert!(
                used.insert((module.to_string(), name.to_string())),
                "duplicate case wrapper"
            );
            let visibility = if indent.is_empty() {
                "pub(super)"
            } else {
                "pub(in super::super)"
            };
            let head = format!("{indent}{visibility} fn {name}() {{");
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
                    .position(|l| *l == format!("{indent}}}"))
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
            l.trim_start()
                .strip_prefix("pub(super) fn ")
                .or_else(|| l.trim_start().strip_prefix("pub(in super::super) fn "))
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

fn family(name: &str, directory: &str) -> (String, Vec<String>) {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff"));
    let parent = std::fs::read_to_string(root.join(format!("{name}.rs"))).expect("replay source");
    let mut cases = BTreeMap::new();
    for module in parent.lines().filter_map(|l| {
        l.trim_start()
            .strip_prefix("mod cases_")
            .and_then(|s| s.strip_suffix(';'))
    }) {
        assert!(
            !module.is_empty()
                && module
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        );
        let module = format!("cases_{module}");
        let source = std::fs::read_to_string(root.join(directory).join(format!("{module}.rs")))
            .expect("registered case source");
        assert!(
            cases.insert(module, source).is_none(),
            "duplicate case module"
        );
    }
    assert!(!cases.is_empty(), "replay case modules disappeared");
    let paths = cases
        .keys()
        .map(|m| format!("diff/{directory}/{m}.rs"))
        .collect();
    (expand(&parent, &cases), paths)
}

pub(crate) fn golden() -> (String, Vec<String>) {
    family("golden", "golden")
}

pub(crate) fn harness() -> (String, Vec<String>) {
    family("harness", "harness/tests")
}

#[test]
fn nested_case_expansion_preserves_exact_body_and_rejects_missing_guard() {
    let parent = "    #[test]\n    fn check() {\n        cases_00::check();\n    }\n";
    let original = "    #[test]\n    fn check() {\n        let _pins = Pins::hold();\n        pin_eq!(got, want);\n    }\n";
    let body = original.replace("    #[test]\n    fn", "    pub(in super::super) fn");
    let cases = BTreeMap::from([("cases_00".to_string(), body.clone())]);
    assert_eq!(expand(parent, &cases), original);
    let changed = BTreeMap::from([(
        "cases_00".to_string(),
        body.replace("        let _pins = Pins::hold();\n", ""),
    )]);
    let expanded = expand(parent, &changed);
    assert!(!expanded.contains("Pins::hold()"));
    assert!(expanded.contains("pin_eq!(got, want)"));
    for bad in [
        parent.replace("::check()", "::wrong()"),
        parent.repeat(2),
        String::new(),
    ] {
        assert!(std::panic::catch_unwind(|| expand(&bad, &cases)).is_err());
    }
    assert!(std::panic::catch_unwind(|| expand(parent, &BTreeMap::new())).is_err());
    let duplicate = BTreeMap::from([("cases_00".to_string(), body.repeat(2))]);
    assert!(std::panic::catch_unwind(|| expand(parent, &duplicate)).is_err());
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
