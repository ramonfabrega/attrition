//! **The sweep** — the original's functions run under the emulator against
//! this crate's on chosen inputs (item 1575; DECISIONS 63 (iv), the middle
//! track). One module a function: its `#[test]` names the function as
//! `Name@00xxxxxx`, which is what `tools/census.py --layers` counts as
//! backed, and runs `tools/emu/sweep_<module>.py` — a script on
//! `tools/emu/callfn.py`'s machine that prints one row per input,
//! `<args…> -> <result>` — asserting every row against the port. An input
//! where the two part is a finding: the test names it and the item parks
//! it with its frame.
//!
//! Each test skips, and says so, on a machine without the install or `uv`,
//! as `path::tests::the_emulated_original_agrees_on_every_row` does.

mod build_type_corner_tile;
mod cosx;
mod farms_grow;
mod flanking;
mod gather_point_is_inside;
mod object_is_in_range;
mod reversing;
mod unit_get_speed;
mod unit_mana_left;
mod wcoord_to_tcoord;

/// The rows `tools/emu/<script>` prints for the install's executable, or
/// `None` (said on stderr) where the install or `uv` is absent.
pub(crate) fn emu_rows(script: &str) -> Option<String> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: set RON_INSTALL (the sweep runs the install's executable)");
        return None;
    };
    let out = std::process::Command::new("uv")
        .args(["run", &format!("{root}/tools/emu/{script}")])
        .arg(format!("{install}/riseofnations.exe"))
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).expect("utf-8 rows")),
        Ok(o) => panic!("{script} failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
    }
}

/// One row, `<args…> -> <result>`, split into its integers.
pub(crate) fn row(line: &str) -> (Vec<i64>, i64) {
    let (lhs, rhs) = line
        .split_once(" -> ")
        .expect("a row is `<args…> -> <result>`");
    let args = lhs
        .split_whitespace()
        .map(|s| s.parse().expect("an integer argument"))
        .collect();
    (args, rhs.trim().parse().expect("an integer result"))
}

#[test]
fn a_row_is_its_arguments_and_its_result() {
    assert_eq!(row("3 -4 0 -> -7"), (vec![3, -4, 0], -7));
}
