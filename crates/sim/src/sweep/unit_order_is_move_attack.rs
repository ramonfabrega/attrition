//! `UnitOrder::is_move_attack@0047ff00` under the emulator against the port (item 1851).

/// `UnitOrder::is_move_attack@0047ff00`, run by `tools/emu/sweep_unit_order_is_move_attack.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: virtual slots `+0x14` then `+0x18`; 1 when either answers non-zero. Row `<s14> <s18>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_order_is_move_attack.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] != 0 || a[1] != 0);
        assert_eq!(got, want, "UnitOrder::is_move_attack — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("UnitOrder::is_move_attack@0047ff00: {n} rows agree");
}
