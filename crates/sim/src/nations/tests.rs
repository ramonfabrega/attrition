use super::*;
use crate::economy::{self, Resource};

fn sim() -> Sim {
    Sim::new(crate::Tuning::RON, crate::World::new(16, 16), 2)
}

/// The ladder the Roman and British Barracks arms share, on the
/// shipped numbers. The British one is the *count* — no multiplier, no
/// cap — which is why run53's AI is paid **one** archer at Ancient and
/// not three.
#[test]
fn the_age_ladder_is_the_british_arm_s_whole_count() {
    let t = crate::Tuning::RON;
    let british = |age| {
        free_tier(
            age,
            t.british_age_for_1_archer,
            t.british_age_for_2_archers,
            t.british_age_for_3_archers,
        )
    };
    assert_eq!(british(0), 1, "BRITISH_AGE_FOR_1_ARCHER is zero");
    assert_eq!(british(1), 1);
    assert_eq!(british(2), 2);
    assert_eq!(british(3), 3);
    assert_eq!(british(7), 3, "and it stops at three");

    // The Roman one has both a multiplier and a cap, and starts an age
    // later: nothing at Ancient.
    let roman = |age| {
        free_scaled(
            t.roman_barracks_legion,
            t.roman_max_legion,
            free_tier(
                age,
                t.roman_age_for_1_legion,
                t.roman_age_for_2_legions,
                t.roman_age_for_3_legions,
            ),
        )
    };
    assert_eq!(roman(0), 0, "ROMAN_AGE_FOR_1_LEGION is one");
    assert_eq!(roman(1), 1);
    assert_eq!(roman(3), 2);
    assert_eq!(roman(5), 3);
    assert_eq!(roman(7), 3);
}

/// `free_scaled`'s two edges, both read off the decompile rather than
/// guessed: a zero cap skips the ladder entirely and pays the plain
/// per-building figure, and a negative either side pays nothing.
#[test]
fn a_zero_cap_skips_the_ladder_and_a_negative_pays_nothing() {
    assert_eq!(
        free_scaled(2, 0, 3),
        2,
        "the whole block sits inside max != 0"
    );
    assert_eq!(free_scaled(2, 5, 3), 5);
    assert_eq!(free_scaled(2, 5, 2), 4);
    assert_eq!(free_scaled(-1, 5, 2), 0);
    assert_eq!(free_scaled(2, -1, 2), 0);
}

#[test]
fn the_roster_index_is_the_power_the_rules_name() {
    // The indices `docs/TECH.md` pins at `has_preq`, each of which is a
    // `has_tribe_bonus(n)` this crate already hardcodes.
    assert_eq!(power_of("Nubians"), Some(4));
    assert_eq!(power_of("Greeks"), Some(5));
    assert_eq!(power_of("Romans"), Some(6));
    assert_eq!(power_of("Egyptians"), Some(7));
    assert_eq!(power_of("French"), Some(10));
    assert_eq!(power_of("British"), Some(11));
    assert_eq!(power_of("Germans"), Some(12));
    assert_eq!(power_of("Russians"), Some(13));
    assert_eq!(power_of("Koreans"), Some(16));
    assert_eq!(power_of("Iroquois"), Some(18));
    assert_eq!(power_of("Lakota"), Some(0x13));
    assert_eq!(power_of("Dutch"), Some(0x16));
    assert_eq!(power_of("Persians"), Some(23));
    assert_eq!(power_of("Prussians"), None);
}

#[test]
fn the_dump_s_tribe_turns_one_flag_on_and_the_rest_off() {
    let mut s = sim();
    s.set_tribe(0, 11);
    assert_eq!(s.tech[0].power, Some(11));
    assert!(s.nation[0].british);
    assert!(!s.nation[0].french);
    assert!(!s.nation[0].nubians);

    // Nubians on the other side, from the same capture.
    s.set_tribe(1, 4);
    assert!(s.nation[1].nubians);
    assert!(!s.nation[1].british);
    // And the first player is untouched by the second's.
    assert!(s.nation[0].british);
}

#[test]
fn a_tribe_of_minus_one_is_a_leader_with_no_nation() {
    let mut s = sim();
    s.set_tribe(0, 11);
    assert!(s.nation[0].british);
    s.set_tribe(0, -1);
    assert_eq!(s.tech[0].power, None);
    assert_eq!(s.tech[0].tribe, 0, "the graft table stays in range");
    assert!(!s.nation[0].british);
    assert!(!s.nation[0].aztecs, "and −1 is not roster index 0");
}

#[test]
fn the_lobby_s_no_nation_powers_puts_every_flag_out() {
    let mut s = sim();
    s.setup.no_nation_powers = true;
    s.set_tribe(0, 11);
    assert_eq!(s.tech[0].power, Some(11), "the nation is still the nation");
    assert!(!s.nation[0].british, "but the power is off");

    // And a leader with no city takes none of it either — in a nomad
    // lobby. With a starting town the gate is `starting_town || city_num`
    // and the first half is true (group 22).
    let mut t = sim();
    t.tech[0].has_city = false;
    t.set_tribe(0, 11);
    assert!(t.nation[0].british, "a starting town opens the gate");
    let mut n = sim();
    n.setup.starting_town = false;
    n.tech[0].has_city = false;
    n.set_tribe(0, 11);
    assert!(!n.nation[0].british, "nomads have no power without a city");
    n.tech[0].has_city = true;
    n.refresh_nation_powers(0);
    assert!(n.nation[0].british, "and the first city turns it on");
}

/// The measurement the wire was worth: run40's AI is the British and
/// carries a commerce cap of 1392 where the human carries 1120.
#[test]
fn run40_s_british_cap_follows_from_the_dump_s_tribe() {
    let mut s = sim();
    s.set_tribe(0, 4); // the human, Nubians
    s.set_tribe(1, 11); // the AI, British
    s.assemble_holdings(0);
    s.assemble_holdings(1);
    let t = &s.tuning;
    let cap = |w: usize| economy::commerce_cap(t, &s.holdings[w], Resource::Food);
    assert_eq!(cap(0), 1120, "70 x 16");
    assert_eq!(cap(1), 1392, "70 x 125 / 100 = 87, then x 16");
}

// ---- the audit table (`docs/NATIONS.md`, item 2976) ---------------

/// `docs/NATIONS.md`, which the audit's guard reads.
fn audit_doc() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/NATIONS.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Every `.rs` under `crates/sim/src` by file name: the raw text and
/// the text with line comments removed (a field witness is code, an
/// address witness is a citation).
fn sim_sources() -> Vec<(String, String, String)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).expect("crates/sim/src") {
            let p = e.expect("entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")); // `src/` and `tests/` (item 4009).
    let mut files = Vec::new();
    walk(root, &mut files);
    files
        .into_iter()
        .map(|p| {
            let raw = std::fs::read_to_string(&p).expect("source");
            let code = raw
                .lines()
                .map(|l| l.find("//").map_or(l, |i| &l[..i]))
                .collect::<Vec<_>>()
                .join("\n");
            let mut name = p.file_name().expect("name").to_string_lossy().into_owned();
            // A test tail item 4010 moved from `foo.rs` to `foo/tests.rs`
            // (or `foo/<name>_tests.rs`) is still `foo.rs`'s.
            let src = root.join("src");
            if let Some(dir) = p.parent()
                && dir.parent() == Some(src.as_path())
                && !dir.ends_with("sweep")
                && p.file_stem()
                    .is_some_and(|s| s.to_string_lossy().ends_with("tests"))
            {
                name = format!("{}.rs", dir.file_name().expect("dir").to_string_lossy());
            }
            (name, raw, code)
        })
        .collect()
}

fn is_ident(w: &str) -> bool {
    !w.is_empty()
        && w.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn is_addr(w: &str) -> bool {
    w.len() == 8 && w.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `w` as a whole word (an identifier witness) or as a substring.
fn holds(text: &str, w: &str) -> bool {
    if !is_ident(w) || is_addr(w) {
        return text.contains(w);
    }
    text.match_indices(w).any(|(i, _)| {
        let b = text.as_bytes();
        let ok = |c: u8| !(c.is_ascii_alphanumeric() || c == b'_');
        (i == 0 || ok(b[i - 1])) && b.get(i + w.len()).is_none_or(|&c| ok(c))
    })
}

const STATUSES: [&str; 7] = [
    "modelled at ",
    "carried at ",
    "loaded at ",
    "carried, unread",
    "not carried",
    "not modelled",
    "not sim",
];

/// The guarded rows: `(line, status, witness)`.
fn audit_rows(doc: &str) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for (n, line) in doc.lines().enumerate() {
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if let Some(i) = cells
            .iter()
            .position(|c| STATUSES.iter().any(|s| c.starts_with(s)))
            && let Some(w) = cells.get(i + 1)
        {
            out.push((n + 1, cells[i].to_string(), w.trim_matches('`').to_string()));
        }
    }
    out
}

/// The roster table lists the 24 nations in `ROSTER`'s order, and the
/// rules key beside each is that nation's own.
#[test]
fn the_audit_table_lists_the_roster_in_order() {
    let doc = audit_doc();
    let names: Vec<String> = doc
        .lines()
        .filter(|l| {
            l.starts_with("| ")
                && l.split('|')
                    .nth(1)
                    .is_some_and(|c| c.trim().parse::<usize>().is_ok())
        })
        .map(|l| l.split('|').nth(2).expect("nation").trim().to_string())
        .collect();
    assert_eq!(
        names,
        ROSTER.to_vec(),
        "docs/NATIONS.md section 2 against nations::ROSTER"
    );
}

/// Every row's status and witness are true of this tree. This is the
/// guard that fails when a row names a module that does not hold it —
/// and, the other way, when a `not modelled` row has been built.
#[test]
fn the_audit_table_s_witnesses_hold() {
    let doc = audit_doc();
    let src = sim_sources();
    let tuning = &src
        .iter()
        .find(|s| s.0 == "tuning.rs")
        .expect("tuning.rs")
        .1;
    let rows = audit_rows(&doc);
    assert!(
        rows.len() > 340,
        "{} guarded rows; the tables are gone",
        rows.len()
    );
    let mut bad = Vec::new();
    for (line, status, w) in &rows {
        let text = |s: &(String, String, String)| {
            if is_addr(w) || w.contains('@') {
                s.1.clone()
            } else {
                s.2.clone()
            }
        };
        if status == "not sim" {
            if w.is_empty() {
                bad.push(format!("line {line}: a `not sim` row gives no reason"));
            }
        } else if let Some(f) = status
            .strip_prefix("modelled at ")
            .or_else(|| status.strip_prefix("carried at "))
        {
            match src
                .iter()
                .filter(|s| s.0 == f)
                .map(text)
                .reduce(|a, b| a + &b)
            {
                None => bad.push(format!(
                    "line {line}: `{f}` is not a module of crates/sim/src"
                )),
                Some(t) if !holds(&t, w) => {
                    bad.push(format!(
                        "line {line}: `{status}` but {f} does not hold `{w}`"
                    ));
                }
                Some(_) => {}
            }
        } else if let Some(f) = status.strip_prefix("loaded at ") {
            // The effect is a rule the loader builds (`rondata`), which
            // the sim's `TechTree` then runs; the field is read there.
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../rondata/src")
                .join(f);
            match std::fs::read_to_string(&path) {
                Err(e) => bad.push(format!("line {line}: {}: {e}", path.display())),
                Ok(t) => {
                    let code: String = t
                        .lines()
                        .filter(|l| !l.trim_start().starts_with("//"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !holds(&code, w) {
                        bad.push(format!(
                            "line {line}: `{status}` but {f} does not read `{w}`"
                        ));
                    }
                }
            }
        } else if status == "carried, unread" {
            if !tuning.contains(&format!("pub {w}:")) {
                bad.push(format!("line {line}: `{w}` is not a Tuning field"));
            }
            for s in src.iter().filter(|s| s.0 != "tuning.rs") {
                if holds(&s.2, w) {
                    bad.push(format!(
                        "line {line}: `{w}` is read in {} — mark it modelled",
                        s.0
                    ));
                }
            }
        } else if status == "not carried" {
            if tuning.contains(&format!("pub {w}:")) {
                bad.push(format!(
                    "line {line}: `{w}` is now a Tuning field — the row is stale"
                ));
            }
        } else if status == "not modelled" {
            // The dead-list pin (`docs_guard::DEAD_CITED`) holds an
            // address to say a document cites a function the game never
            // runs; it models nothing (the forty-eighth pass, `get_econ_mod`).
            for s in src.iter().filter(|s| s.0 != "docs_guard.rs") {
                if holds(&text(s), w) {
                    bad.push(format!(
                        "line {line}: `not modelled` but {} holds `{w}`",
                        s.0
                    ));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "docs/NATIONS.md rows that no longer hold:\n{}",
        bad.join("\n")
    );
}

/// A nation's constant added to `Tuning` has to be audited: every
/// `Tuning` field with a nation's prefix has a row in §4.
#[test]
fn every_nation_field_in_tuning_has_an_audit_row() {
    let doc = audit_doc();
    let src = sim_sources();
    let tuning = &src
        .iter()
        .find(|s| s.0 == "tuning.rs")
        .expect("tuning.rs")
        .2;
    let prefixes = [
        "aztec_",
        "maya_",
        "inca_",
        "bantu_",
        "nubian_",
        "greek_",
        "roman_",
        "egyptian_",
        "turk_",
        "spanish_",
        "french_",
        "british_",
        "german_",
        "russian_",
        "chinese_",
        "japanese_",
        "korean_",
        "mongol_",
        "iroquois_",
        "lakota_",
        "americans_",
        "indians_",
        "dutch_",
        "persians_",
    ];
    let missing: Vec<String> = tuning
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .filter(|f| prefixes.iter().any(|p| f.starts_with(p)) && is_ident(f))
        .filter(|f| !doc.contains(&format!("`{}`", f.to_uppercase())))
        .map(str::to_string)
        .collect();
    assert!(
        missing.is_empty(),
        "Tuning nation fields with no row in docs/NATIONS.md section 4: {missing:?}"
    );
}

/// The guard fails when it should: a witness that is false is caught
/// (the mutation the module's test of itself runs).
#[test]
fn the_audit_guard_catches_a_false_row() {
    let src = sim_sources();
    let city = src.iter().find(|s| s.0 == "city.rs").expect("city.rs");
    assert!(holds(&city.2, "chinese_large_cities"));
    // Spelled in halves, so that this file does not hold a witness the
    // table says is absent.
    assert!(!holds(&city.2, &["aztec", "_slay_bonus"].concat()));
    assert!(
        !holds("aztec_slay_bonus_x", &["aztec", "_slay_bonus"].concat()),
        "whole words only"
    );
    let rows = audit_rows(
        "| `X` | 1 | modelled at city.rs | `fn nothing_at_all` |\n| a | not sim | `why` |",
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].1, "modelled at city.rs");
    assert_eq!(rows[1].2, "why");
}
