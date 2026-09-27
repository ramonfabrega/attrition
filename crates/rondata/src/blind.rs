//! The blind list — the cited functions no coverage trace has entered.
//!
//! `docs/DECISIONS.md` entry 29 names it the second counter of "sim done",
//! entry 41 makes it one of the four: **the blind list at an enumerated
//! residue**. A diff can only check what a run reaches, so a function a
//! document cites and no run of the original has ever entered is a claim
//! resting on a reading alone. `tools/trace/report.py … blind docs/` prints
//! the list; this module is the same measurement as an assertion, so the
//! list cannot grow in silence (`docs/CENSUS.md`, "The blind list, ranked").
//!
//! **What is pinned is the list, not a count.** [`NEVER`] is every
//! never-entered cited function by address, so a failure names the
//! function that moved: a new citation of a function no pinned trace
//! enters, or a pinned trace (a new capture added to [`TRACES`]) entering
//! one. A count would say only that something did. The residue DECISIONS
//! 29 asks to be "enumerated and accepted deliberately, function by
//! function" is this constant.
//!
//! **What is not pinned**: the traces themselves, which live outside the
//! repo in the capture corpus (`docs/ORACLE.md`), and the Ghidra export's
//! `INDEX.tsv`, which never enters it. A machine without either skips and
//! says so, through the same door every kept dump uses, so the release
//! gate's fixture audit sees each trace requested.
//!
//! The measurement is `report.py`'s `blind` verb exactly: a function is
//! **cited** when `docs/**/*.md` writes `name@00xxxxxx` — joined across a
//! line break after the `@` (parked 835) — and the address is a function
//! the export lists; it is **entered** when a trace carries a `HIT` record
//! for it, or, for the five trampolined functions that carry no stub, a
//! record of the kind their hook emits. The proxied call sites `report.py`
//! also credits change nothing on this corpus (item 923 measured both), so
//! they are not read here.

/// The coverage traces the list is measured against, by archive name.
///
/// **Twenty-one of the eighty-nine on disk before item 923**, and they
/// enter exactly what all eighty-nine do — 7,664 functions by `HIT`
/// record, 7,666 with the hooks — the greedy cover item 923 measured,
/// largest gain first:
/// `run29` alone enters 7,175. The other sixty-eight add nothing, so
/// reading them would cost 1.2 GB and change no line of [`NEVER`]. A
/// capture taken to shrink the list joins here in the landing that pins
/// its effect.
pub const TRACES: &[&str] = &[
    "rontrace-run29.log",
    "rontrace-run16.log",
    "rontrace-run50.log",
    "rontrace-run218.log",
    "rontrace-run49.log",
    "rontrace-run20.log",
    "rontrace-run44.log",
    "rontrace-run24.log",
    "rontrace-run47.log",
    "rontrace-run48.log",
    "rontrace-run74.log",
    "rontrace-run18a.log",
    "rontrace-run32.log",
    "rontrace-run30.log",
    "rontrace-run66.log",
    "rontrace-fuzz-424242.log",
    "rontrace-run14.log",
    "rontrace-run15.log",
    "rontrace-run22.log",
    "rontrace-run64.log",
    "rontrace-run906.log",
    // Item 923's capture: chapter twenty-three's staging under `cover=1`,
    // whose `@` issuers the coverage jmps refuse (`docs/RUNS.md` run314).
    // It enters `Object::do_launch` and `Leader::set_age`, and nothing
    // else no other trace here does.
    "rontrace-run314.log",
    // Item 935's capture: British Isles with a Large Town start, quit at
    // frame 5 (`docs/RUNS.md` run316). It enters `Map::place_start_in_region`
    // and `Setup::large_city_buildings`, the two of row 4's nine that a solo
    // lobby reaches.
    "rontrace-run316.log",
];

/// Every cited function no trace in [`TRACES`] enters, ascending — the
/// blind list itself: 228 against 1,120 cited on the corpus as it stood on
/// 2026-09-27, 227 with run314 (item 923; `Object::do_launch` off), **225**
/// with run316 (item 935; `Map::place_start_in_region` and
/// `Setup::large_city_buildings` off). `docs/CENSUS.md`'s "The blind list,
/// ranked" groups it by the staging that would enter each family.
#[rustfmt::skip]
pub const NEVER: &[u32] = &[
    0x0046_cec0, 0x0046_ed70, 0x0046_ee80, 0x0046_ef90, 0x0047_0e50, 0x0047_11e0,
    0x0047_2410, 0x0047_80c0, 0x0047_da40, 0x0047_fa40, 0x0047_fd80, 0x0047_fff0,
    0x0048_2720, 0x0048_2cf0, 0x0048_2dd0, 0x0048_41f0, 0x0048_45c0, 0x0048_46f0,
    0x0048_4820, 0x0048_5140, 0x0048_5a60, 0x0048_6ba0, 0x0048_89a0, 0x0054_cea0,
    0x0054_cf90, 0x0054_d100, 0x0058_60c0, 0x0058_6440, 0x0058_7060, 0x0059_30c0,
    0x005a_ac70, 0x005a_bc70, 0x005d_9950, 0x005e_1f20, 0x005e_2bd0, 0x005e_3310,
    0x005e_3400, 0x005e_35e0, 0x005e_3df0, 0x005e_3f60, 0x005e_4080, 0x005e_4560,
    0x005e_4c80, 0x005e_4d10, 0x005e_4ff0, 0x005e_5bf0, 0x005e_65d0, 0x005e_6b80,
    0x005e_75a0, 0x005e_8670, 0x005e_9be0, 0x005e_b960, 0x005e_d040, 0x005e_d1f0,
    0x005e_e420, 0x005f_1910, 0x005f_2480, 0x005f_79c0, 0x005f_ccc0, 0x005f_d080,
    0x0060_3470, 0x0060_4550, 0x0060_86f0, 0x0060_8850, 0x0060_a140, 0x0060_a310,
    0x0060_a600, 0x0061_a960, 0x0062_0280, 0x0062_06e0, 0x0062_2670, 0x0062_2ce0,
    0x0062_2d10, 0x0062_2e70, 0x0062_3310, 0x0062_9e70, 0x0062_d430, 0x0062_d4d0,
    0x0063_0590, 0x0063_0b10, 0x0063_3390, 0x0063_e2a0, 0x0063_e390, 0x0064_40c0,
    0x0064_5330, 0x0065_cfd0, 0x0067_04a0, 0x0067_0880, 0x0067_3a80, 0x0067_4370,
    0x0067_b800, 0x0068_3730, 0x0068_8310, 0x0068_d1a0, 0x0069_5050, 0x006b_22e0,
    0x006b_4230, 0x006b_46b0, 0x006b_81e0, 0x006b_88b0, 0x006d_0370, 0x006d_18a0,
    0x006d_5230, 0x006d_6740, 0x006d_6e80, 0x006d_a740, 0x006e_0c60, 0x006e_0f30,
    0x006e_c170, 0x006f_0230, 0x006f_2c90, 0x006f_49a0, 0x006f_4af0, 0x006f_b260,
    0x006f_c9a0, 0x006f_d510, 0x006f_e1a0, 0x0070_0010, 0x0070_0490, 0x0070_0b90,
    0x0070_20c0, 0x0070_24b0, 0x0070_30c0, 0x0070_6d90, 0x0070_7220, 0x0070_7510,
    0x0070_84c0, 0x0070_8620, 0x0070_8820, 0x0070_88e0, 0x0070_8980, 0x0070_8b10,
    0x0070_8b90, 0x0070_8c60, 0x0070_95e0, 0x0070_ad10, 0x0070_afc0, 0x0070_b060,
    0x0070_bab0, 0x0070_beb0, 0x0071_0b40, 0x0071_3390, 0x0071_37f0, 0x0071_3bb0,
    0x0071_4d00, 0x0071_c470, 0x0071_c500, 0x0071_c740, 0x0071_dfd0, 0x0072_15b0,
    0x0072_1c40, 0x0073_6820, 0x0073_c7e0, 0x0073_d070, 0x0073_e000, 0x0073_e0c0,
    0x0073_e350, 0x0082_c520, 0x008c_7050, 0x0092_fc50, 0x0093_ee70, 0x0094_1580,
    0x0094_15e0, 0x0094_16b0, 0x0094_17a0, 0x0094_1800, 0x0094_1860, 0x0094_1910,
    0x0094_1960, 0x0094_1a20, 0x0094_1a70, 0x0094_1b80, 0x0094_1be0, 0x0094_1c30,
    0x0094_1ca0, 0x0094_1d40, 0x0094_1e70, 0x0094_1ed0, 0x0094_1f80, 0x0094_2c40,
    0x0094_2c90, 0x0094_3f30, 0x0094_65d0, 0x0094_66f0, 0x0094_7680, 0x0094_78a0,
    0x0094_79c0, 0x0094_7db0, 0x0094_7fe0, 0x0094_8110, 0x0094_8230, 0x0094_8340,
    0x0094_8760, 0x0094_8cb0, 0x0094_8e00, 0x0094_8f60, 0x0094_9140, 0x0094_9380,
    0x0094_94a0, 0x0094_95c0, 0x0094_9970, 0x0094_9ae0, 0x0094_9c30, 0x0094_9d90,
    0x0094_9ed0, 0x0094_c1c0, 0x0095_2d90, 0x0099_6ac0, 0x0099_bc20, 0x009a_adc0,
    0x009b_8ac0, 0x009e_18b0, 0x009f_45e0, 0x009f_85b0, 0x009f_99e0, 0x009f_9ad0,
    0x009f_bb80, 0x009f_bd60, 0x009f_f5e0, 0x009f_f620, 0x009f_f860, 0x009f_f8e0,
    0x009f_fa10, 0x009f_fbf0, 0x00a4_69f0,
];

/// The blind list's accepted residue: functions on [`NEVER`] that no
/// capture this project can stage will enter, each with the reason — the
/// "enumerated and accepted deliberately, function by function" of
/// `docs/DECISIONS.md` entry 29. A row leaves only by a capture entering
/// it, which fails the pin, or by its citation going; either way
/// [`the_residue_is_on_the_blind_list`](tests) says so.
///
/// **"No reference in the executable"** means the image holds no `call`,
/// `jmp` or `jcc rel32` to the entry, no rel8 jump to it at an instruction
/// boundary, and no four-byte copy of its address at any offset of any
/// section, `.rdata`'s vtables included: the out-of-line copy of a
/// function the compiler inlined at every use, or one nothing calls. The
/// scan is `tools/trace/report.py <exe> refs`, run over all of [`NEVER`]
/// on 2026-09-27; the rows that say it are re-scanned against the install
/// by [`every_unreferenced_row_is_unreferenced_in_the_image`](tests).
/// Where the live copy is known, the row names it — the claim a document
/// cites at the orphan's address is then a claim about that caller.
///
/// Item 935 took seven (row 4 of `docs/CENSUS.md`'s ranked list); item
/// 940 took `docs/EMULATOR.md` §4's other twelve and ten more the scan
/// found. Eleven `CommandManager::issue_*` are unreferenced too and are
/// **not** here: the DLL's `@` issuer lines call them by address, so a
/// coverage capture can enter them (`docs/CENSUS.md`, "The blind list,
/// 2026-09-27, item 940").
pub const RESIDUE: &[(u32, &str)] = &[
    (
        0x0059_30c0,
        "Game::action_cheat_ai_toggle: no reference in the executable; \
         CommandPackage::process_cheat_ai_toggle flips ai_off inline",
    ),
    (
        0x005a_ac70,
        "Setup::init_wild_life: no reference in the executable",
    ),
    (
        0x005a_bc70,
        "Setup::build_leader: no reference in the executable",
    ),
    (
        0x0060_8850,
        "UnitData::can_gather: no reference in the executable",
    ),
    (
        0x0060_a600,
        "UnitData::turn_speed: no reference in the executable; the live copy \
         is GuyData::turn_speed",
    ),
    (
        0x0062_2ce0,
        "Build::add_attack_order: no reference in the executable",
    ),
    (
        0x0062_3310,
        "Build::update_max_gatherers: no reference in the executable",
    ),
    (
        0x0062_d430,
        "BuildData::num_scholars: no reference in the executable",
    ),
    (
        0x0063_0590,
        "BuildData::max_gatherers: no reference in the executable",
    ),
    (
        0x0063_3390,
        "BuildType::set_domain: no reference in the executable",
    ),
    (
        0x0065_cfd0,
        "ObjectsData::find_dock: no reference in the executable",
    ),
    (
        0x0068_3730,
        "PathFinder::find_wpath_army: no reference in the executable",
    ),
    (
        0x0068_8310,
        "PathFinderData::get_estimate: no reference in the executable (the rel8 \
         lookalike at 006882f9 is inside a cmpb); inlined four times in \
         PathFinder::astar_path",
    ),
    (
        0x0069_5050,
        "MapGrass::make_continents: map style 23 (BLANK_MAP), which only \
         ScenarioEditor::generate_map asks Map::new_map for; the lobby's styles stop at 22",
    ),
    (
        0x006b_22e0,
        "World::clear_danger: no reference in the executable",
    ),
    (
        0x006b_4230,
        "World::set_behind: no reference in the executable; its writers \
         (Wall::mark_behind_tiles, Mountains::add_mountain) carry it inline",
    ),
    (
        0x006b_46b0,
        "World::set_gathered_at: no reference in the executable",
    ),
    (
        0x006b_88b0,
        "Leader::process: no reference in the executable; inlined in \
         Leaders::process_all, which calls its gather and process_elimination",
    ),
    (
        0x006d_5230,
        "LeaderData::locked_transport: no reference in the executable",
    ),
    (
        0x006d_6740,
        "LeaderData::get_handicap_level: no reference in the executable",
    ),
    (
        0x006d_6e80,
        "LeaderData::get_fishermen: no reference in the executable",
    ),
    (
        0x006d_a740,
        "LeaderData::get_handicap: all four callers gate on game semaphore bit 2, \
         the multiplayer flag; the capture lanes are solo",
    ),
    (
        0x006e_c170,
        "LeaderData::is_human: no reference in the executable (inlined at every use)",
    ),
    (0x006f_0230, "Tribe::init: no reference in the executable"),
    (
        0x0071_37f0,
        "Group::leader_report_speed: no reference in the executable; its three \
         statements stand inline in Unit::do_group_move",
    ),
    (
        0x0071_3bb0,
        "Group::report_speed: no reference in the executable",
    ),
    (
        0x0092_fc50,
        "GameLog::dump_armies: no reference in the executable, and gamelog.ini \
         has no ARMY key",
    ),
    (
        0x0094_c1c0,
        "CommandPackage::clear: no reference in the executable",
    ),
    (
        0x00a4_69f0,
        "cos_table: no reference in the executable; a two-instruction thunk \
         (add ecx, 0x3fffffff; jmp sin_table) whose one use, \
         MapGrass::make_continents, adds the constant inline",
    ),
];

/// The five trampolined functions (`tools/trace/tracer.c`'s `HOOKS`), by
/// RVA, and the record kind each emits. They carry no coverage stub, so a
/// trace enters one exactly when it holds a record of its kind —
/// `report.py`'s own rule.
const HOOKS: [(u32, u32); 5] = [
    (0x0019_1ef0, 2), // `Game::do_frame`, FRAME
    (0x0063_9cf0, 1), // `Random::get()`
    (0x0063_9d70, 3), // `Random::get(a, b)`
    (0x005e_18b0, 4), // `rand_real`
    (0x0063_9d30, 6), // `Random::reseed`
];

/// The functions one trace entered, folded to the export's numbering.
/// `None` when the bytes are not a trace. Reads only the record kind and
/// the `HIT` address, so a 93 MB trace costs one pass and no allocation
/// beyond the set.
pub fn entered(bytes: &[u8]) -> Option<std::collections::BTreeSet<u32>> {
    let word = |off: usize| {
        u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
    };
    if bytes.len() < 32 || word(0) != 0x544E_4F52 {
        return None;
    }
    let base = word(8);
    let mut out = std::collections::BTreeSet::new();
    let mut kinds = [false; 16];
    for rec in bytes[32..].chunks_exact(32) {
        let kind = u32::from_le_bytes([rec[0], rec[1], rec[2], rec[3]]);
        if kind == 0 {
            let va = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]);
            out.insert(va.wrapping_sub(base).wrapping_add(crate::trace::IMAGE_BASE));
        } else if let Some(k) = kinds.get_mut(kind as usize) {
            *k = true;
        }
    }
    for (rva, kind) in HOOKS {
        if kinds[kind as usize] {
            out.insert(rva + crate::trace::IMAGE_BASE);
        }
    }
    Some(out)
}

/// Every address a document's text cites as `name@00xxxxxx`, in the
/// grammar of `report.py`'s `blind` verb: the character before the `@` is
/// a name's (`[A-Za-z0-9_:~<>]`, with a letter or `_` somewhere in the
/// run), and a line break with its indentation may stand between the `@`
/// and the address (parked 835).
pub fn cited(text: &str) -> Vec<u32> {
    let b = text.as_bytes();
    let namey = |c: u8| c.is_ascii_alphanumeric() || b"_:~<>".contains(&c);
    let mut out = Vec::new();
    for (at, _) in text.match_indices('@') {
        let mut i = at;
        let mut lead = false;
        while i > 0 && namey(b[i - 1]) {
            i -= 1;
            lead |= b[i].is_ascii_alphabetic() || b[i] == b'_';
        }
        if i == at || !lead {
            continue;
        }
        let mut j = at + 1;
        if b.get(j) == Some(&b'\n') {
            j += 1;
            while matches!(b.get(j), Some(b' ' | b'\t')) {
                j += 1;
            }
        }
        let Some(hex) = b.get(j..j + 8) else { continue };
        if hex.starts_with(b"00")
            && hex
                .iter()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
            && let Ok(a) = u32::from_str_radix(std::str::from_utf8(hex).unwrap_or(""), 16)
        {
            out.push(a);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    /// `docs/**/*.md`, each cited address with the documents that cite it.
    fn citations() -> BTreeMap<u32, BTreeSet<String>> {
        fn walk(
            dir: &std::path::Path,
            root: &std::path::Path,
            out: &mut BTreeMap<u32, BTreeSet<String>>,
        ) {
            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .expect("docs/ is readable")
                .flatten()
                .map(|e| e.path())
                .collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(&p, root, out);
                } else if p.extension().is_some_and(|x| x == "md") {
                    let text = std::fs::read_to_string(&p).unwrap_or_default();
                    let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                    for a in cited(&text) {
                        out.entry(a).or_default().insert(rel.clone());
                    }
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let mut out = BTreeMap::new();
        walk(&root, &root, &mut out);
        out
    }

    /// **The blind list is the pinned residue, function for function.**
    ///
    /// Fails in either direction and names each function that moved: one
    /// cited and entered by no pinned trace that [`NEVER`] does not list
    /// (a reading's new citation — take the capture that enters it, or
    /// add it to the residue on purpose), and one [`NEVER`] lists that is
    /// now entered or no longer cited (a capture joined [`TRACES`], or a
    /// citation went — take it off, so the list only ever shrinks by a
    /// line someone wrote).
    #[test]
    fn the_blind_list_is_the_pinned_residue() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        let Ok(index) = std::fs::read_to_string(&index_path) else {
            eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
            return;
        };
        let names: BTreeMap<u32, &str> = index
            .lines()
            .filter_map(|l| {
                let mut it = l.split('\t');
                Some((u32::from_str_radix(it.next()?, 16).ok()?, it.next()?))
            })
            .collect();
        assert!(names.len() > 40_000, "the export's index is 48k functions");

        let mut entered_any = BTreeSet::new();
        for name in TRACES {
            let Some(path) = crate::testenv::dump(name) else {
                eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
                return;
            };
            let bytes = std::fs::read(&path).expect("a kept trace is readable");
            entered_any.extend(entered(&bytes).unwrap_or_else(|| panic!("{name} is not a trace")));
        }

        let cites = citations();
        let never: BTreeSet<u32> = cites
            .keys()
            .copied()
            .filter(|a| names.contains_key(a) && !entered_any.contains(a))
            .collect();
        let pinned: BTreeSet<u32> = NEVER.iter().copied().collect();
        assert_eq!(pinned.len(), NEVER.len(), "NEVER lists an address twice");
        assert!(NEVER.is_sorted(), "NEVER is kept ascending");

        let name = |a: &u32| names.get(a).copied().unwrap_or("?");
        let mut failures = Vec::new();
        for a in never.difference(&pinned) {
            let docs: Vec<&str> = cites[a].iter().map(String::as_str).collect();
            failures.push(format!(
                "  new on the blind list: `{}@{a:08x}` is cited ({}) and no trace in TRACES enters it",
                name(a),
                docs.join(", ")
            ));
        }
        for a in pinned.difference(&never) {
            let why = if entered_any.contains(a) {
                "is now entered"
            } else {
                "is no longer cited"
            };
            failures.push(format!("  off the blind list: `{}@{a:08x}` {why}", name(a)));
        }
        assert!(
            failures.is_empty(),
            "the blind list moved ({} cited, {} never, {} pinned) — re-pin `blind::NEVER` \
             to the measurement, or take the capture a new line names:\n{}",
            cites.keys().filter(|a| names.contains_key(a)).count(),
            never.len(),
            NEVER.len(),
            failures.join("\n")
        );
    }

    /// **Every accepted residue row is still on the blind list**, once and
    /// in order — a residue row a capture entered, or whose citation went,
    /// is a row to delete, not a reason that still stands.
    #[test]
    fn the_residue_is_on_the_blind_list() {
        assert!(
            RESIDUE.is_sorted_by_key(|r| r.0),
            "RESIDUE is kept ascending"
        );
        for (a, why) in RESIDUE {
            assert!(
                NEVER.binary_search(a).is_ok(),
                "residue row `{why}` ({a:08x}) is not on NEVER — delete the row"
            );
        }
    }

    /// **Every row that says "no reference" has none**: the image holds
    /// no `call`/`jmp`/`jcc rel32` and no four-byte pointer to it
    /// (`crate::pe::Pe::references`). The controls are the two of item
    /// 935's rows that are referenced, so the scan is seen to find a caller
    /// and a vtable slot before its silence is believed.
    #[test]
    fn every_unreferenced_row_is_unreferenced_in_the_image() {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let pe = crate::pe::Pe::open(&format!("{root}/riseofnations.exe"))
            .expect("riseofnations.exe is a PE file");
        assert_eq!(
            pe.references(0x006d_a740),
            vec![0x0065_0b67, 0x0065_1780, 0x006b_107f, 0x006d_66bd],
            "LeaderData::get_handicap's four callers"
        );
        assert_eq!(
            pe.references(0x0069_5050),
            vec![0x00b4_5634],
            "MapGrass::make_continents's vtable slot"
        );
        let mut failures = Vec::new();
        for (a, why) in RESIDUE {
            if why.contains("no reference in the executable") {
                let sites = pe.references(*a);
                if !sites.is_empty() {
                    failures.push(format!("  `{why}` ({a:08x}): referenced at {sites:08x?}"));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "a residue row says unreferenced and the image says otherwise — \
             read the site (`report.py <exe> refs`) and fix the row:\n{}",
            failures.join("\n")
        );
    }

    /// The grammar: a split citation is joined, a bare address is not a
    /// citation, and a name's template brackets are part of it.
    #[test]
    fn a_citation_is_a_name_an_at_and_an_address() {
        let text = "`Unit::do_patrol@005f1910` and `Group::action_\
                    patrol@\n    007030c0`, `Foo<Bar>@00400000`, \
                    and not @00400010, 0x10@00400020, nor `x@0040003`.";
        assert_eq!(
            cited(text),
            vec![0x005f_1910, 0x0070_30c0, 0x0040_0000, 0x0040_0020]
        );
    }

    /// The scanner: a `HIT` folds to the export's base, a hook is entered
    /// by its record kind alone, and something that is not a trace is
    /// refused.
    #[test]
    fn a_trace_enters_its_hits_and_its_hooks() {
        let rec = |r: [u32; 8]| r.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>();
        let mut bytes = rec([0x544E_4F52, 2, 0x0100_0000, 0, 0, 48_233, 0, 0]);
        bytes.extend(rec([0, 0x0120_0000, 0, 0, 0, 0, 0, u32::MAX]));
        bytes.extend(rec([2, 0, 0, 0, 0, 0, 0, 0]));
        let got = entered(&bytes).expect("a trace");
        assert_eq!(
            got.into_iter().collect::<Vec<_>>(),
            vec![0x0059_1ef0, 0x0060_0000]
        );
        assert!(entered(b"not a trace, not at all, no..........").is_none());
    }
}
