//! `is_air@0046f000` under the emulator against the port (item 1691).

/// `is_air@0046f000`, run by `tools/emu/sweep_is_air.py` under unicorn
/// (tools/emu): `ecx` an `OrderIndex`, answering 1 for `0x10`, `0x11`, `0x18`. Every index 0..=40 and the 8-bit edges against `orders::index::is_air_family`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_is_air.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let ours = i64::from(crate::orders::index::is_air_family(
            u8::try_from(a[0]).expect("an index below 256"),
        ));
        assert_eq!(ours, want, "is_air_family — row `{line}`");
        n += 1;
    }
    assert!(n >= 40, "only {n} rows");
    eprintln!("is_air@0046f000: {n} rows agree");
}
