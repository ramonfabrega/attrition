//! `GoodTypeData::avail_ctw@0066fd50` under the emulator against the port (item 1819).

/// `GoodTypeData::avail_ctw@0066fd50`, run by `tools/emu/sweep_good_type_data_avail_ctw.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the type; reads the `GameAccessConst::gamec` cell at 0xc061e8: with bit 0x2 of the byte at game+0x822 set, true when the dword `exclude_ctw` at +0x2f4 is 0, else true.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_data_avail_ctw.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] & 2 == 0 || a[1] == 0);
        assert_eq!(got, want, "GoodTypeData::avail_ctw — row `{line}`");
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("GoodTypeData::avail_ctw@0066fd50: {n} rows agree");
}
