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

/// Simulation-layer functions a coverage section's diff or a sweep backs.
pub const SIMULATION_BACKED: usize = 68;
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
