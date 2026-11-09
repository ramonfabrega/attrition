//! `UnitTypeData::get_underlay_circle_radius@0061daf0` under the emulator against the port (item 1851).

/// `UnitTypeData::get_underlay_circle_radius@0061daf0`, run by `tools/emu/sweep_unit_type_data_get_underlay_circle_radius.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: the int at `+0x234` times `unit_block_radius` at `+0x10` of the `GameAccessConst::constantsc` cell `0xc061e4` (32-bit wrapping). Row `<r> <block>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_get_underlay_circle_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (r, b) = (
            i32::try_from(a[0]).expect("an int"),
            i32::try_from(a[1]).expect("an int"),
        );
        let got = i64::from(r.wrapping_mul(b));
        assert_eq!(
            got, want,
            "UnitTypeData::get_underlay_circle_radius — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("UnitTypeData::get_underlay_circle_radius@0061daf0: {n} rows agree");
}
