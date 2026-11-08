//! `FarmsData::get_wheat_status@008d9190` under the emulator against the port (item 1819).

/// `FarmsData::get_wheat_status@008d9190`, run by `tools/emu/sweep_farms_data_get_wheat_status.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the farms data, three stack arguments `(a, b, c)`: the byte at array+`(a*0x30 + c)*4 + b + 0xac`, the array behind +0x24 holding `(o*31+7) & 0xff` at offset `o`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_data_get_wheat_status.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let o = (a[0] * 0x30 + a[2]) * 4 + a[1] + 0xac;
        assert_eq!(
            (o * 31 + 7) & 0xff,
            want,
            "FarmsData::get_wheat_status — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 700, "only {n} rows");
    eprintln!("FarmsData::get_wheat_status@008d9190: {n} rows agree");
}
