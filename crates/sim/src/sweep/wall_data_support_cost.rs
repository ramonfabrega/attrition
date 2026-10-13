//! `WallData::support_cost@0063fa30` under the emulator against the port (item 1735).

/// `WallData::support_cost@0063fa30`, run by `tools/emu/sweep_wall_data_support_cost.py` under unicorn
/// (tools/emu): `*(int*)(type+0x270+i*4) << 8` through the type pointer at `this+0x18`, against `wrapping_shl(8)` of the planted word (the 8.8 scale the port keeps).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_support_cost.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[1]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v.wrapping_shl(8)),
            "WallData::support_cost — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::support_cost@0063fa30: {n} rows agree");
}
