//! `ObjectTypeData::is_cargo@00661d80` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

/// `ObjectTypeData::is_cargo@00661d80`, run by `tools/emu/sweep_object_type_data_is_cargo.py` under unicorn
/// (tools/emu): a unit type — with the base slot `+0xc`, its index in `0x32..=0x19d` and no call; overridden, the
/// slot's answer non-zero — of the land domain (`+0x218` == 0). `transport.rs`'s `is_cargo` is the port's
/// object-level test, which holds the same range and domain.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_is_cargo.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mode, t, dom, ans, calls) = (a[0], a[1], a[2], a[3], a[4]);
        let unit = if mode == 1 {
            assert_eq!(calls, 0, "the base slot is never entered — row `{line}`");
            (0x32..=0x19d).contains(&t)
        } else {
            assert_eq!(calls, 1, "the slot is asked once — row `{line}`");
            ans != 0
        };
        let got = i64::from(unit && dom == 0);
        assert_eq!(got, want, "is_cargo — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectTypeData::is_cargo@00661d80: {n} rows agree");
}
