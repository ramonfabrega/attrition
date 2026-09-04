//! The no-writer guard: a compared field the simulation never writes.
//!
//! [`crate::diff`] scores the port by comparing the simulation's state to the
//! original's per-frame dump, field by field, and a row that agrees is
//! evidence. **Unless the simulation never writes its side.** Then the row
//! holds a constant against a constant, agrees on every frame of every
//! capture, and is counted as evidence for a mechanic that does not exist.
//!
//! Three of those surfaced in one week, and each cost real frames before
//! anyone noticed: `LeaderData::pop`, which `create_units`' offer price is
//! computed from and which nothing in this workspace ever wrote (the word
//! stood at 6582 until it did); `mil_trainers`, read by the research AI and
//! appended to by nobody; and the nation ordering. In all three the diff was
//! green throughout. Nothing read the harness against the simulation, so
//! this does.
//!
//! # What it checks
//!
//! [`compared_fields_have_writers`] reads two sources and joins them:
//!
//! - **`diff.rs`'s labelled comparison rows** — the `("name", ours, theirs)`
//!   idiom, ~270 of them. Inside each row it collects every `.field` whose
//!   name is declared by a struct in `crates/sim`.
//! - **`crates/sim`'s own source**, with `#[cfg(test)]` and `#[test]` bodies
//!   blanked, and with the struct *definitions* blanked so a declaration
//!   (`pub pop: i32,`) is never mistaken for an initialiser (`pop: n,`).
//!
//! A field passes if the simulation writes it outside its tests: an
//! assignment or compound assignment, a `&mut` borrow, a mutating method on
//! it, or a struct-literal initialiser with a value that is not the type's
//! zero. A field that has none of those is a failure unless it is on
//! [`EXEMPT`] with a reason.
//!
//! # What it does not check
//!
//! The writer search is by **field name**, not by type: `x.pop = n` names no
//! struct, so a `pop` written on one struct satisfies a `pop` read on
//! another. Struct-literal writes *are* attributed, which is what removed
//! most of the noise (46 candidates before, 4 after), but the assignment
//! path stays name-global. It is a floor, not a proof — and the failure it
//! is built for, a field no code anywhere writes, it catches exactly.
//!
//! Rows that compare a literal (`("scouted", 0, c.scouted)`) name no
//! simulation field and so are invisible here. `diff.rs` already carries
//! those five in a comment that says what they claim and why they stay.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// A compared field the simulation deliberately never writes, and why.
///
/// A row belongs here only when the *original* does not write it either on
/// any capture on disk — which makes the comparison a live claim (a capture
/// that moved the field would fail the diff) rather than a vacuous one. The
/// reason names the evidence. When a mechanic lands that writes one of
/// these, delete its row; the guard will then check it for real.
const EXEMPT: &[(&str, &str, &str)] = &[
    (
        "navy",
        "Army::navy",
        "no capture forms a naval army: `Armies::send_navy@006f2c90` is on \
         the blind list (68 traces, never entered), the ARMY dump asserts \
         `navy` 0 for every army it prints, and run58's 5k frames print no \
         ARMY record with it set. Twenty branches of `army.rs` read it and \
         are unreached; the row is the claim that the original leaves it \
         false too.",
    ),
    (
        "upgrades",
        "Personality::upgrades",
        "deliberate, and stated where it is not written (`ai.rs`, \
         `Personality::roll`: \"`upgrades` is not rolled here\"). The \
         original's PERSONALITY block prints 0 for it on every capture, so \
         the row asserts that the twenty-four-int roll skips exactly this \
         one.",
    ),
    (
        "bordering",
        "CityAi::bordering",
        "the city AI's bit-per-leader \"an enemy borders this city\" mark. \
         Read once (`army.rs`, the invasion target sweep) and written \
         nowhere; the dump prints 0 for all 13,787 CITY records of run58 \
         and all 752 of run59, so no capture has ever set it. The row \
         falsifies the day one does.",
    ),
    (
        "ocean_filled",
        "CityAi::ocean_filled",
        "the city AI's \"the ocean side of this city's ring is full\" mark. \
         Neither written nor read in `crates/sim`; 0 for every CITY record \
         of run59. It is the census sweep's, and the sweep is not built.",
    ),
];

/// The floor on how many labelled rows the extractor finds. `diff.rs` had
/// 269 on 2026-09-05. If the idiom changes and the scanner stops seeing
/// them, the guard would pass by checking nothing — which is the failure a
/// guard is least able to report about itself.
const MIN_ROWS: usize = 200;

/// The floor on how many simulation fields those rows name — 133 on
/// 2026-09-05, for the same reason.
const MIN_FIELDS: usize = 100;

fn sim_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../sim/src"))
}

fn diff_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff.rs"))
}

/// The index of the `}` that closes the `{` at `i`.
fn close_brace(s: &[u8], i: usize) -> usize {
    let mut depth = 0usize;
    for (j, &b) in s.iter().enumerate().skip(i) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return j;
                }
            }
            _ => {}
        }
    }
    s.len() - 1
}

/// The index of the `)` that closes the `(` at `i`.
fn close_paren(s: &[u8], i: usize) -> usize {
    let mut depth = 0usize;
    for (j, &b) in s.iter().enumerate().skip(i) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return j;
                }
            }
            _ => {}
        }
    }
    s.len() - 1
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The identifier ending just before `at`, if there is one.
fn ident_before(s: &[u8], at: usize) -> &str {
    let mut end = at;
    while end > 0 && (s[end - 1] as char).is_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident(s[start - 1]) {
        start -= 1;
    }
    std::str::from_utf8(&s[start..end]).unwrap_or("")
}

/// The identifier starting at `at`, and the index just past it.
fn ident_at(s: &[u8], at: usize) -> (&str, usize) {
    let mut end = at;
    while end < s.len() && is_ident(s[end]) {
        end += 1;
    }
    (std::str::from_utf8(&s[at..end]).unwrap_or(""), end)
}

/// Overwrite `[from, to]` with spaces, keeping newlines so line numbers hold.
fn blank(s: &mut [u8], from: usize, to: usize) {
    let to = to.min(s.len() - 1);
    for b in &mut s[from..=to] {
        if *b != b'\n' {
            *b = b' ';
        }
    }
}

/// A `#[cfg(test)]` or `#[test]` body, blanked. The attribute is followed by
/// the item it applies to; the first `{` after it opens that item.
fn blank_tests(src: &mut [u8]) {
    let mut i = 0;
    while i + 8 < src.len() {
        let head = &src[i..];
        let is_attr = head.starts_with(b"#[cfg(test)]") || head.starts_with(b"#[test]");
        if !is_attr {
            i += 1;
            continue;
        }
        match src[i..].iter().position(|&b| b == b'{') {
            Some(off) => {
                let open = i + off;
                let close = close_brace(src, open);
                blank(src, i, close);
                i = close + 1;
            }
            None => break,
        }
    }
}

/// A field a struct or enum body declares, at brace depth zero of that body
/// and outside a wrapped type (`Vec<\n  T,\n>` never reaches depth zero).
fn decl_name(line: &str) -> Option<&str> {
    let t = line.trim_start();
    if t.starts_with("//") {
        return None;
    }
    let t = t.strip_prefix("pub ").unwrap_or(t);
    let t = match t.strip_prefix("pub(") {
        Some(rest) => rest.split_once(')').map(|(_, r)| r.trim_start())?,
        None => t,
    };
    let name: &str = t.split(':').next()?.trim();
    if name.is_empty() || !name.bytes().all(is_ident) {
        return None;
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
        return None;
    }
    let rest = t.strip_prefix(name)?.trim_start();
    let rest = rest.strip_prefix(':')?;
    (!rest.trim().is_empty() && !rest.starts_with(':')).then_some(name)
}

/// Every struct and enum body in `src`: `(name, open, close)` byte offsets.
fn definitions(src: &[u8]) -> Vec<(String, usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let word = if src[i..].starts_with(b"struct ") {
            7
        } else if src[i..].starts_with(b"enum ") {
            5
        } else {
            i += 1;
            continue;
        };
        if i > 0 && is_ident(src[i - 1]) {
            i += 1;
            continue;
        }
        let (name, mut j) = ident_at(src, i + word);
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            i += 1;
            continue;
        }
        // Skip generics and any `where` clause up to the body or a `;`.
        let mut depth = 0i32;
        while j < src.len() {
            match src[j] {
                b'<' => depth += 1,
                b'>' => depth -= 1,
                b'{' if depth <= 0 => break,
                b';' | b'(' if depth <= 0 => {
                    j = src.len();
                    break;
                }
                _ => {}
            }
            j += 1;
        }
        if j >= src.len() {
            i += word;
            continue;
        }
        let close = close_brace(src, j);
        out.push((name.to_string(), j, close));
        i = j + 1;
    }
    out
}

/// The names a struct-literal-shaped body initialises, with the value text.
///
/// Both `field: expr,` and the shorthand `field,` count. Only lines at the
/// body's own nesting level are read, so a nested literal's fields are not
/// attributed to the outer struct.
fn literal_fields(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    for line in body.lines() {
        if depth == 0 {
            let t = line.trim();
            if !t.starts_with("//") && !t.is_empty() {
                let head = t.trim_end_matches(',');
                if let Some((name, value)) = head.split_once(':') {
                    let name = name.trim();
                    if !name.is_empty() && name.bytes().all(is_ident) {
                        out.push((name.to_string(), value.trim().to_string()));
                    }
                } else if head.bytes().all(is_ident)
                    && !head.is_empty()
                    && t.ends_with(',')
                    && head.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                {
                    // Field-init shorthand: `World { width, height }`.
                    out.push((head.to_string(), head.to_string()));
                }
            }
        }
        for b in line.bytes() {
            match b {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                _ => {}
            }
        }
        depth = depth.max(0);
    }
    out
}

/// A value that leaves the field at its type's zero. An initialiser like
/// this is not a writer: it is the state `pop` was in for the month nothing
/// wrote it.
fn is_zero_value(v: &str) -> bool {
    matches!(
        v,
        "0" | "0.0"
            | "false"
            | "None"
            | "Vec::new()"
            | "Default::default()"
            | "[]"
            | "\"\""
            | "String::new()"
    )
}

/// The methods that mutate the receiver in place. `.push` on a field is a
/// writer of that field even though no `=` appears.
const MUTATORS: &[&str] = &[
    "push",
    "insert",
    "clear",
    "remove",
    "extend",
    "retain",
    "truncate",
    "resize",
    "fill",
    "swap",
    "sort",
    "sort_by",
    "sort_by_key",
    "sort_unstable",
    "sort_unstable_by",
    "dedup",
    "append",
    "drain",
    "push_str",
    "get_mut",
    "iter_mut",
    "last_mut",
    "first_mut",
    "entry",
    "take",
    "replace",
    "reserve",
    "splice",
    "rotate_left",
    "rotate_right",
    "swap_remove",
    "set",
    "set_len",
    "extend_from_slice",
];

/// What the simulation's own source says about its fields.
struct Sim {
    /// field name → the structs that declare it, as `Struct::field`.
    declared: BTreeMap<String, BTreeSet<String>>,
    /// `Struct::field` written by a struct literal.
    literal_writes: BTreeSet<String>,
    /// field name written by an assignment, a `&mut`, or a mutator — the
    /// receiver's type is not known, so this side is name-global.
    name_writes: BTreeSet<String>,
}

fn read_sim() -> Sim {
    let mut declared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut literal_writes = BTreeSet::new();
    let mut name_writes = BTreeSet::new();
    let mut structs: BTreeSet<String> = BTreeSet::new();
    let mut sources = Vec::new();

    for entry in std::fs::read_dir(sim_dir()).expect("crates/sim/src") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        sources.push((path, text));
    }
    sources.sort_by(|a, b| a.0.cmp(&b.0));

    // Pass one: the declarations, and the bodies to blank in pass two.
    let mut blanked = Vec::new();
    for (_, text) in &sources {
        let mut bytes: Vec<u8> = text.clone().into_bytes();
        blank_tests(&mut bytes);
        let defs = definitions(&bytes);
        for (name, open, close) in &defs {
            structs.insert(name.clone());
            let body = std::str::from_utf8(&bytes[*open..=*close]).unwrap_or("");
            let mut depth = 0i32;
            for line in body.lines() {
                if depth == 0
                    && let Some(f) = decl_name(line)
                {
                    declared
                        .entry(f.to_string())
                        .or_default()
                        .insert(format!("{name}::{f}"));
                }
                for b in line.bytes() {
                    match b {
                        b'(' | b'<' | b'[' => depth += 1,
                        b')' | b'>' | b']' => depth -= 1,
                        _ => {}
                    }
                }
                depth = depth.max(0);
            }
        }
        // Blank the definitions so a declaration never reads as an init.
        for (_, open, close) in defs {
            blank(&mut bytes, open, close);
        }
        blanked.push(bytes);
    }

    // Pass two: the writers.
    for bytes in &blanked {
        // Struct literals, attributed to their struct.
        let mut i = 0;
        let mut last_impl: Option<String> = None;
        while i < bytes.len() {
            if bytes[i..].starts_with(b"impl") && (i == 0 || !is_ident(bytes[i - 1])) {
                // `impl<..> Trait for Struct {` or `impl<..> Struct {`: the
                // last capitalised word before the `{` is the type.
                if let Some(off) = bytes[i..].iter().position(|&b| b == b'{') {
                    let head = std::str::from_utf8(&bytes[i..i + off]).unwrap_or("");
                    last_impl = head
                        .split(|c: char| !is_ident(c as u8))
                        .rfind(|w| w.starts_with(|c: char| c.is_ascii_uppercase()))
                        .map(str::to_string);
                }
            }
            if bytes[i] != b'{' {
                i += 1;
                continue;
            }
            let name = ident_before(bytes, i);
            let owner = if name == "Self" {
                last_impl.clone()
            } else if structs.contains(name) {
                // `impl Foo {`, `-> Foo {`, `struct Foo {` are not literals.
                let before = ident_before(bytes, i - name.len().min(i));
                let arrow = {
                    let mut k = i - name.len().min(i);
                    while k > 0 && (bytes[k - 1] as char).is_whitespace() {
                        k -= 1;
                    }
                    k >= 2 && &bytes[k - 2..k] == b"->"
                };
                if matches!(before, "impl" | "struct" | "enum" | "trait" | "for") || arrow {
                    None
                } else {
                    Some(name.to_string())
                }
            } else {
                None
            };
            let Some(owner) = owner else {
                i += 1;
                continue;
            };
            let close = close_brace(bytes, i);
            let body = std::str::from_utf8(&bytes[i + 1..close]).unwrap_or("");
            for (field, value) in literal_fields(body) {
                if !is_zero_value(&value) {
                    literal_writes.insert(format!("{owner}::{field}"));
                }
            }
            i += 1;
        }

        // Assignments, `&mut` borrows and mutating methods, by name.
        let text = std::str::from_utf8(bytes).unwrap_or("");
        for (at, _) in text.match_indices('.') {
            let (field, end) = ident_at(bytes, at + 1);
            if field.is_empty() || !field.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
            {
                continue;
            }
            let mut k = end;
            // An index between the field and the operator still writes it.
            if bytes.get(k) == Some(&b'[') {
                let mut depth = 0i32;
                while k < bytes.len() {
                    match bytes[k] {
                        b'[' => depth += 1,
                        b']' => {
                            depth -= 1;
                            if depth == 0 {
                                k += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                }
            }
            while k < bytes.len() && bytes[k] == b' ' {
                k += 1;
            }
            // `= x`, `+= x`, `<<= x`, but never `==`.
            let assigns = {
                let mut j = k;
                while j < bytes.len()
                    && matches!(
                        bytes[j],
                        b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'&' | b'^' | b'<' | b'>'
                    )
                {
                    j += 1;
                }
                j < bytes.len()
                    && bytes[j] == b'='
                    && bytes.get(j + 1) != Some(&b'=')
                    && (j == k || j <= k + 2)
            };
            let mutator = bytes.get(k) == Some(&b'.') && {
                let (m, _) = ident_at(bytes, k + 1);
                MUTATORS.contains(&m)
            };
            let borrowed = {
                // `&mut <path>.field`
                let mut s = at;
                while s > 0
                    && (is_ident(bytes[s - 1])
                        || matches!(bytes[s - 1], b'.' | b'[' | b']' | b'(' | b')'))
                {
                    s -= 1;
                }
                let mut t = s;
                while t > 0 && (bytes[t - 1] as char).is_whitespace() {
                    t -= 1;
                }
                t >= 4 && &bytes[t - 4..t] == b"&mut"
            };
            if assigns || mutator || borrowed {
                name_writes.insert(field.to_string());
            }
        }
    }

    Sim {
        declared,
        literal_writes,
        name_writes,
    }
}

/// Every `("label", …)` comparison row in `diff.rs`, as `(label, line, body)`.
fn comparison_rows(src: &str) -> Vec<(String, usize, String)> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'(' {
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
            j += 1;
        }
        if bytes.get(j) != Some(&b'"') {
            continue;
        }
        let start = j + 1;
        let Some(len) = bytes[start..].iter().position(|&c| c == b'"' || c == b'\n') else {
            continue;
        };
        if bytes[start + len] != b'"' {
            continue;
        }
        let label = &src[start..start + len];
        let plain = label
            .bytes()
            .all(|c| is_ident(c) || matches!(c, b'[' | b']' | b'x'))
            && label.starts_with(|c: char| c.is_ascii_lowercase() || c == '_');
        if !plain || label.is_empty() {
            continue;
        }
        // `"label",` or `"label".into(),` — anything else is not a row.
        let mut k = start + len + 1;
        if bytes[k..].starts_with(b".into()") {
            k += 7;
        }
        while k < bytes.len() && (bytes[k] as char).is_whitespace() {
            k += 1;
        }
        if bytes.get(k) != Some(&b',') {
            continue;
        }
        let close = close_paren(bytes, i);
        let line = src[..i].matches('\n').count() + 1;
        out.push((label.to_string(), line, src[i + 1..close].to_string()));
    }
    out
}

#[test]
fn compared_fields_have_writers() {
    let sim = read_sim();
    let diff = std::fs::read_to_string(diff_path()).expect("diff.rs");
    let rows = comparison_rows(&diff);
    assert!(
        rows.len() >= MIN_ROWS,
        "found {} labelled comparison rows in diff.rs, expected at least {MIN_ROWS} — \
         the `(\"name\", ours, theirs)` idiom changed and this guard is now checking \
         nothing. Fix the scanner before lowering the floor",
        rows.len()
    );

    // field name → the rows that name it
    let mut touched: BTreeMap<&str, Vec<(&str, usize)>> = BTreeMap::new();
    for (label, line, body) in &rows {
        let bytes = body.as_bytes();
        for (at, _) in body.match_indices('.') {
            let (field, _) = ident_at(bytes, at + 1);
            if field.is_empty() {
                continue;
            }
            if let Some((name, _)) = sim.declared.get_key_value(field) {
                touched
                    .entry(name.as_str())
                    .or_default()
                    .push((label.as_str(), *line));
            }
        }
    }
    assert!(
        touched.len() >= MIN_FIELDS,
        "the rows name only {} simulation fields, expected at least {MIN_FIELDS}",
        touched.len()
    );

    let mut failures = Vec::new();
    for (field, rows) in &touched {
        if sim.name_writes.contains(*field) {
            continue;
        }
        let owners = &sim.declared[*field];
        if owners.iter().any(|o| sim.literal_writes.contains(o)) {
            continue;
        }
        if EXEMPT
            .iter()
            .any(|(f, owner, _)| f == field && owners.contains(&owner.to_string()))
        {
            continue;
        }
        let (label, line) = rows[0];
        failures.push(format!(
            "`{field}` ({}) is compared by diff.rs:{line} as \"{label}\" and no code in \
             crates/sim writes it outside its tests — the row holds a constant against a \
             constant and passes whatever the original does. Implement the writer, or add \
             it to writers::EXEMPT with the capture that shows the original does not write \
             it either",
            owners.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// An exemption that no longer names a compared field is stale, and a stale
/// exemption is how the list stops being read. Every row must still be a
/// field of a simulation struct.
#[test]
fn every_exemption_still_names_a_field() {
    let sim = read_sim();
    let mut failures = Vec::new();
    for (field, owner, reason) in EXEMPT {
        match sim.declared.get(*field) {
            Some(owners) if owners.contains(&owner.to_string()) => {}
            _ => failures.push(format!(
                "writers::EXEMPT names `{owner}`, which no struct in crates/sim declares — \
                 delete the row or fix the name"
            )),
        }
        assert!(
            reason.len() > 40,
            "writers::EXEMPT `{owner}` has no real reason"
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
