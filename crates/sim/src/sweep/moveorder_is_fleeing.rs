//! `MoveOrder::is_fleeing@004889a0` under the emulator against the port (item 1782).

/// `MoveOrder::is_fleeing@004889a0`, run by `tools/emu/sweep_moveorder_is_fleeing.py` under unicorn
/// (tools/emu): the flag byte over all 256 values (offset 0x40), against `flag_reads::move_order_is_fleeing` on the signed byte.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_moveorder_is_fleeing.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want != 0,
            crate::flag_reads::move_order_is_fleeing(a[0] as u8 as i8),
            "MoveOrder::is_fleeing — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("MoveOrder::is_fleeing@004889a0: {n} rows agree");
}
