//! `BuildTypeData::area@006389b0` under the emulator against the port (item 1793).

/// `BuildTypeData::area@006389b0`, run by `tools/emu/sweep_build_type_data_area.py` under unicorn
/// (tools/emu): the ints at `+0x234` and `+0x238`, edge pairs and two hundred random pairs, against their wrapping product.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_area.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (x, y) = (
            i32::try_from(a[0]).expect("a 32-bit integer"),
            i32::try_from(a[1]).expect("a 32-bit integer"),
        );
        assert_eq!(
            want,
            i64::from(x.wrapping_mul(y)),
            "BuildTypeData::area — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::area@006389b0: {n} rows agree");
}
