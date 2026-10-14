//! The census by layer, and the board's number: the simulation layer's
//! backed share (item 1467; DECISIONS 63 (iv), the sweep lane's score).
//!
//! `tools/census.py --layers` files every function the Ghidra export lists
//! under a layer — simulation, AI, engine, interface or unknown — by its
//! PDB source path, and marks it **backed** when a document's coverage
//! section cites it in a diff-backed span or a sweep `#[test]` in
//! `crates/sim` (one that runs the original under the emulator) names it.
//! The rule and what it cannot count are the tool's docstring.
//!
//! This module pins the number, as [`crate::blind`] pins the blind list:
//! a document's new coverage citation, or a new sweep, moves it, and the
//! landing that moves it re-pins it here and says so; the handoff's
//! `Census:` line is read against the same two constants
//! (`diff::floors`). The export, the PDB and `llvm-pdbutil` live outside
//! the repo; a machine without them skips and says so.

/// Simulation-layer functions a coverage section's diff or a sweep backs (item 2117: 566 -> 571,
/// `docs/GOLDEN.md` §77.1 — `think_attack`, `add_to_army`, `get_unit`, `go_to_unit`, `go_to`; item
/// 2129: 571 -> 581, the sweep's fifty-first batch; item 2149: 581 -> 591, the fifty-second; item
/// 2131: 591 -> 602, `docs/GOLDEN.md` §78.1 — `get_final_loc`, `get_loc_to`, `action_move_near`,
/// `action_move_to`, `process_move_to`, `compute_form`, `add_group_move_order`, and chapter twelve's
/// follow line, `process_follow`, `action_follow`, `add_follow_order`, `do_follow`; item 2160: 602 -> 612, the sweep's
/// fifty-third batch; item 2170: 612 -> 622, the fifty-fourth; item 2181: 622 -> 632, the
/// fifty-fifth; item 2165: 632 -> 639, `docs/GOLDEN.md` §79.1 — `Unit::init`'s Bark arm,
/// `set_stance`, `fight`'s re-search, `issue_stance`, `process_stance`, `action_stance`,
/// `get_stance_option`, `get_stance_type`; item 2187: 639 -> 649, the sweep's fifty-sixth batch; item
/// 2200: 649 -> 659, a second sweep lane's ten with a port consumer; item 2211: 659 -> 669, the
/// sweep's fifty-seventh batch; item 2205: 669 -> 670, `compare_target`, by golden chapter sixty-four's
/// sea arm, `docs/GOLDEN.md` §80.1; item 2223: 670 -> 680, the sweep's fifty-eighth batch; item 2217:
/// 680 -> 690, the second sweep lane's ten with a port consumer; item 2241: 690 -> 700, the sweep's
/// fifty-ninth batch; item 2253: 700 -> 710, the sweep's sixtieth batch; item 2247: 710 -> 720, the second
/// sweep lane's next ten, the territory family through `obj --sparse`; item 2235: 720 -> 721,
/// `UnitData::get_action`, by the city alarm's worker arm, `docs/GOLDEN.md` §81.1; item 2271: 721 ->
/// 731, the sweep's sixty-first batch, the backups and restores through `obj --ptrarg-null`; item
/// 2276: 731 -> 741, the second sweep lane's ten, the squad and city chains through `obj --link`; item
/// 2281: 741 -> 751, the sweep's sixty-second batch, seven of its ten through `obj --record`; item
/// 2298: 751 -> 761, the sweep's sixty-third batch, the unbuilt lists, the speed and the spells;
/// item 2287: 761 -> 771, the second sweep lane's ten with a port consumer, five through `obj --record`;
/// item 2309: 771 -> 782, the sweep's sixty-fourth batch, the leaders-table walks through `obj --record IDX@S`;
/// item 2325: 782 -> 793, the sweep's sixty-fifth batch, eleven, `get_inside`'s out-pointer through
/// `obj --outarg` and `memcpy` through `--stub-cdecl`;
/// item 2314: 793 -> 803, the second sweep lane's ten with a port consumer, four through `obj --rows N`;
/// item 2336: 803 -> 813, the sweep's sixty-sixth batch, a `bool`'s low byte through `obj --ret al` and
/// `timeGetTime` through `--stub iat`;
/// item 2331: 813 -> 815, chapter sixty-seven's coverage, `train_time` and `get_cost` diff-backed by run845;
/// item 2346: 815 -> 825, the sweep's sixty-seventh batch, `edx` beside `ecx` through `obj --edx`, a table
/// index through `--tame`, a container emptied through `--stub-write`;
/// item 2341: 825 -> 835, the second sweep lane's next ten with a port consumer, the order adders and the
/// gaia walks, five through `obj --record-order`;
/// item 2362: 835 -> 845, the sweep's sixty-eighth batch, eight `vtordisp` thunks through `fwd`'s
/// `vtordisp` token, `set_shooter_vis` and `issue_unqueue`;
/// item 2374: 845 -> 873, the sweep's sixty-ninth batch, the twenty-eight `vtordisp` thunks left, one `fwd`
/// line each, the band empty on `census.py --unbacked`;
/// item 2379: 873 -> 883, the sweep's seventieth batch, ten `CommandManager::issue_*` whose command
/// bytes `obj --deref` reads at the call).
pub const SIMULATION_BACKED: usize = 883;
/// Simulation-layer functions: those whose PDB line record starts in a
/// `game\` file the layer rule files under simulation.
pub const SIMULATION_FUNCTIONS: usize = 3611;

#[cfg(test)]
mod tests {
    use super::*;

    /// The number the census measures from `json`'s key `"<key>": N`.
    fn field(json: &str, key: &str) -> Option<usize> {
        let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
        json[at..]
            .trim_start()
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()
    }

    /// **The simulation layer's backed share is the pinned one**, measured
    /// by `tools/census.py --layers --json` on this tree. Made to fail first
    /// with the pin moved by one.
    #[test]
    fn the_census_s_simulation_share_is_pinned() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        if !std::path::Path::new(&index).is_file() {
            eprintln!("skipping: no {index} (the Ghidra export is not on this machine)");
            return;
        }
        let Some(install) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL) for sbl/rise.pdb");
            return;
        };
        let pdb = format!("{install}/sbl/rise.pdb");
        if !std::path::Path::new(&pdb).is_file() {
            eprintln!("skipping: no {pdb}");
            return;
        }
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let out = std::process::Command::new("python3")
            .arg(format!("{root}/tools/census.py"))
            .args(["--layers", "--json", "--index", &index, "--pdb", &pdb])
            .output()
            .expect("python3 runs");
        let text = String::from_utf8_lossy(&out.stdout);
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            if err.contains("no llvm-pdbutil") {
                eprintln!("skipping: {err}");
                return;
            }
            panic!("census.py --layers failed: {err}");
        }
        let measured = (
            field(&text, "simulation_backed").expect("simulation_backed"),
            field(&text, "simulation_functions").expect("simulation_functions"),
        );
        assert_eq!(
            measured,
            (SIMULATION_BACKED, SIMULATION_FUNCTIONS),
            "the census moved: re-pin `census::SIMULATION_BACKED` and \
             `SIMULATION_FUNCTIONS` to (backed, functions) as measured, and the \
             handoff's `Census:` line with them"
        );
    }

    #[test]
    fn a_measured_field_is_read_whole() {
        let json = r#"{"layers": {}, "simulation_backed": 45, "simulation_functions": 3611}"#;
        assert_eq!(field(json, "simulation_backed"), Some(45));
        assert_eq!(field(json, "simulation_functions"), Some(3611));
        assert_eq!(field(json, "simulation"), None);
    }
}
