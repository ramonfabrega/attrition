//! `ObjectData::get_captain@00472400` under the emulator against the port (item 1708).

/// `ObjectData::get_captain@00472400`, run by `tools/emu/sweep_object_get_captain.py` under unicorn
/// (tools/emu): the signed short at `+0xa` (`o`), over the short's edges, every power of two and a seeded band, against `flag_reads::object_get_captain` — the reader `fight`'s attacker side takes a building's `o` through.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_get_captain.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let o = i16::try_from(a[0]).expect("a signed short");
        assert_eq!(
            i64::from(crate::flag_reads::object_get_captain(o)),
            want,
            "get_captain — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::get_captain@00472400: {n} rows agree");
}
