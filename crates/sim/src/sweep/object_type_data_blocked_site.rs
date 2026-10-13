//! `ObjectTypeData::blocked_site@00661d70` under the emulator against the port (item 1851).

/// `ObjectTypeData::blocked_site@00661d70`, run by `tools/emu/sweep_object_type_data_blocked_site.py` under unicorn
/// (tools/emu): `__thiscall`, three stack words (`ret 0xc`), ignores them all: always 0. Row `<a> <b> <c>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_blocked_site.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(0, want, "ObjectTypeData::blocked_site — row `{line}`");
        let _ = a;
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectTypeData::blocked_site@00661d70: {n} rows agree");
}
