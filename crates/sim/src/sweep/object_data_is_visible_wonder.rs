//! `ObjectData::is_visible_wonder@00471130` under the emulator against the port (item 1834).

/// `ObjectData::is_visible_wonder@00471130`, run by `tools/emu/sweep_object_data_is_visible_wonder.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the object: virtual slot `+0x2c` (`a`), the flag dword at `+8` (`f`), and slot `+0xfc` of the type record's vtable (`c`): 1 only when `a != 0`, `f & 0x20 == 0` and `c == 0`. Rows are the 128-point grid of the three.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_visible_wonder.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[0] != 0 && a[1] & 0x20 == 0 && a[2] == 0);
        assert_eq!(got, want, "ObjectData::is_visible_wonder — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectData::is_visible_wonder@00471130: {n} rows agree");
}
