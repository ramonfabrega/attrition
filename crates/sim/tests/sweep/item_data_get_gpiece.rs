//! `ItemData::get_gpiece@00677970` under the emulator against the port (item 1851).

/// `ItemData::get_gpiece@00677970`, run by `tools/emu/sweep_item_data_get_gpiece.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `[this+0x18]` holds a record whose `+4` is `i`; the `MiscAccess::graphic_pieces` cell `0xc06214` points at a table whose `+0xd60` is `first_item_piece`: `first - 0x21f + i`. Row `<first> <i>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_item_data_get_gpiece.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (first, i) = (
            i32::try_from(a[0]).expect("an int"),
            i32::try_from(a[1]).expect("an int"),
        );
        let got = i64::from(first.wrapping_sub(0x21f).wrapping_add(i));
        assert_eq!(got, want, "ItemData::get_gpiece — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ItemData::get_gpiece@00677970: {n} rows agree");
}
