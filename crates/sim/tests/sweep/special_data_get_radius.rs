//! `SpecialData::get_radius@00740060` under the emulator against the port (item 1819).

/// `SpecialData::get_radius@00740060`, run by `tools/emu/sweep_special_data_get_radius.py` under unicorn
/// (tools/emu): `__thiscall` with no use of `this`; reads the `GameAccessConst::constantsc` cell at 0xc061e4 and returns twice the int `general_radius` at +0xc58, wrapping.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_special_data_get_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("int");
        assert_eq!(
            i64::from(v.wrapping_mul(2)),
            want,
            "SpecialData::get_radius — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("SpecialData::get_radius@00740060: {n} rows agree");
}
