//! `BuildTypeData::gather_radius@0063bc60` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `BuildTypeData::gather_radius@0063bc60`, run by `tools/emu/sweep_build_type_data_gather_radius.py` under
/// unicorn (tools/emu), held to the radius `gather.rs`'s gather count reads inline (`WOODCUTTER_RADIUS` for a
/// woodcutter, `MINE_RADIUS` otherwise, none for a flat or a knowledge site): a type without the gather flag
/// (`build::flags::GATHER`, or its own slot `+0x90`'s word) is 0, unasked further; a flat one (`FLAT`, or slot
/// `+0x94`) is 0; else it is asked whether it is a University (`(0x1a4, 0)`, 0 if so), then a Woodcutter
/// (`(0x1a2, 0)`). The constants are laid out as the rules' radii, so a swapped offset parts. (The mark `port`
/// is `calc_gather.rs`'s unrelated `gather_radius`, a unit's ring.)
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::build::flags::{FLAT, GATHER};
    use crate::gather::{MINE_RADIUS, WOODCUTTER_RADIUS};
    use crate::nations::ty::UNIVERSITY;
    let Some(rows) = super::emu_rows("sweep_build_type_data_gather_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (flags, index, own90, own94) = (a[0] as u32, a[1] as usize, a[2] != 0, a[3] != 0);
        let (gathers, flat) = (flags & GATHER != 0, flags & FLAT != 0);
        let mut calls = Vec::new();
        if own90 {
            calls.push("gathers:1:");
        }
        if gathers && own94 {
            calls.push("flat:1:");
        }
        let asks = gathers && !flat;
        if asks {
            calls.push("is:1:420/0");
            if index != UNIVERSITY {
                calls.push("is:1:418/0");
            }
        }
        let want = match (asks, index) {
            (false, _) | (true, UNIVERSITY) => 0,
            (true, 0x1a2) => WOODCUTTER_RADIUS,
            (true, _) => MINE_RADIUS,
        };
        assert_eq!(
            tail,
            [calls.join(",").as_str()],
            "gather_radius — the calls, row `{line}`"
        );
        assert_eq!(eax, i64::from(want), "gather_radius — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("BuildTypeData::gather_radius@0063bc60: {n} rows agree");
}
