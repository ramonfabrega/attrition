//! `ItemTypeData::snap_center@00677df0` under the emulator against the port (item 1851).

/// `ItemTypeData::snap_center@00677df0`, run by `tools/emu/sweep_item_type_data_snap_center.py` under unicorn
/// (tools/emu): `__thiscall`, four stack words (`ret 0x10`): world tile `x`, `y`, then two output pointers; writes `x*0x300 + 0x180` and `y*0x300 + 0x180`. Row `<x> <y> <k> -> <word k written>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_item_type_data_snap_center.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (x, y) = (a[0], a[1]);
        let v = if a[2] == 0 { x } else { y };
        let got = i64::from(
            (i32::try_from(v).expect("an int"))
                .wrapping_mul(0x300)
                .wrapping_add(0x180),
        );
        assert_eq!(got, want, "ItemTypeData::snap_center — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ItemTypeData::snap_center@00677df0: {n} rows agree");
}
