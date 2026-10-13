//! `ObjectTypeData::has_objmask@00470860` under the emulator against the port (item 1814).

/// `ObjectTypeData::has_objmask@00470860`, run by `tools/emu/sweep_object_type_data_has_objmask.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the type record; returns `obj_masks` (+0x1e4) AND the argument — every pair of seeded words, against `masks & arg`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_has_objmask.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mask, arg) = (
            i32::try_from(a[0]).expect("int"),
            i32::try_from(a[1]).expect("int"),
        );
        let got = i64::from(mask & arg);
        assert_eq!(got, want, "has_objmask — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("ObjectTypeData::has_objmask@00470860: {n} rows agree");
}
