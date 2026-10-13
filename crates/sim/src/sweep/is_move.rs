//! `is_move@0046f050` under the emulator against the port (item 1691).

/// `is_move@0046f050`, run by `tools/emu/sweep_is_move.py` under unicorn
/// (tools/emu): `ecx` an `OrderIndex`, answering 1 for 1–4, `0x12`, `0x13`, `0x15`. Every index 0..=40 and the 8-bit edges against `orders::index::is_move_family`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_is_move.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let ours = i64::from(crate::orders::index::is_move_family(
            u8::try_from(a[0]).expect("an index below 256"),
        ));
        assert_eq!(ours, want, "is_move_family — row `{line}`");
        n += 1;
    }
    assert!(n >= 40, "only {n} rows");
    eprintln!("is_move@0046f050: {n} rows agree");
}
