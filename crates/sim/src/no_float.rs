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
//! - **Test code.** `#[cfg(test)]` modules and the two whole-file test
//!   modules `lib.rs` mounts under it. A test may use `f64` freely and
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

/// Every `.rs` file the simulation is built from, with its text.
fn sources() -> Vec<(String, String)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("the crate's own src/") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("a readable source file");
        out.push((name, text));
    }
    out.sort();
    out
}

/// The whole-file test modules `lib.rs` mounts under `#[cfg(test)]`, which a
/// per-file scan cannot recognise from their own contents.
const TEST_FILES: [&str; 3] = ["harness_tests.rs", "cities_tests.rs", "no_float.rs"];

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

/// Blanks out every `#[cfg(test)]` item: the attribute, and the braced block
/// that follows it.
pub(crate) fn without_test_modules(code: &str) -> String {
    let mut out = code.as_bytes().to_vec();
    let mut from = 0;
    while let Some(rel) = code[from..].find("#[cfg(test)]") {
        let at = from + rel;
        // The braced item that follows the attribute, from its first `{` to
        // the matching `}`.
        let mut i = at;
        while i < out.len() && out[i] != b'{' {
            i += 1;
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

    #[test]
    fn the_simulation_holds_no_floating_point() {
        let mut bad: Vec<String> = Vec::new();
        let mut scanned = 0;
        for (name, text) in sources() {
            if TEST_FILES.contains(&name.as_str()) {
                continue;
            }
            scanned += 1;
            let code = without_test_modules(&code_only(&text));
            for (line, what) in floats(&code) {
                bad.push(format!("{name}:{line}: {what}"));
            }
        }
        assert!(
            scanned >= 15,
            "only {scanned} sources scanned — has the crate moved?"
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
    }
}
