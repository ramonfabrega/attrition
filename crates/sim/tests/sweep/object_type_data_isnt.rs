//! `ObjectTypeData::isnt@00661b80` under the emulator against the port (item 1851).

/// `ObjectTypeData::isnt@00661b80`, run by `tools/emu/sweep_object_type_data_isnt.py` under unicorn
/// (tools/emu): `__thiscall`, two stack ints (`ret 8`) passed on to virtual slot `+0x60` (here an argument-carrying stub answering `v + 1000*a + b`): 1 when the slot answers 0. Row `<v> <a> <b>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_isnt.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (v, x, y) = (a[0], a[1], a[2]);
        let slot = i32::try_from(v + 1000 * x + y).expect("an int");
        let got = i64::from(slot == 0);
        assert_eq!(got, want, "ObjectTypeData::isnt — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectTypeData::isnt@00661b80: {n} rows agree");
}
