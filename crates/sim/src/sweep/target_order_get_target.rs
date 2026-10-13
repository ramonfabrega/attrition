//! `TargetOrder::get_target@0072ff60` under the emulator against the port (item 1851).

/// `TargetOrder::get_target@0072ff60`, run by `tools/emu/sweep_target_order_get_target.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `whom` at `+0xc`, `ox` at `+8`; the `GameAccess::objects` cell `0xc0618c` points at a table of 28-byte rows whose `+0x14` is an array of object pointers: `table[whom].array[ox]`. Row `<whom> <ox> <val>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_target_order_get_target.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = a[2];
        assert_eq!(got, want, "TargetOrder::get_target — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TargetOrder::get_target@0072ff60: {n} rows agree");
}
