//! `FarmsData::get_farm_type@008d9160` under the emulator against the port (item 1819).

/// `FarmsData::get_farm_type@008d9160`, run by `tools/emu/sweep_farms_data_get_farm_type.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the farms data, one stack argument: `farm_count` at +0x18, the record array behind +0x24 (0xc0 a record); for an index in `0..farm_count` the low bit of the byte at record+0xbd, else 0 (WHEAT_FARM). The record at the index carries the byte, every other record its complement.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_data_get_farm_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (count, idx, b) = (a[0], a[1], a[2]);
        let got = if idx < count && idx >= 0 { b & 1 } else { 0 };
        assert_eq!(got, want, "FarmsData::get_farm_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("FarmsData::get_farm_type@008d9160: {n} rows agree");
}
