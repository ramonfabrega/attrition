//! `GroupMoveOrder::is_pathed@00485a20` under the emulator against the port (item 1782).

/// `GroupMoveOrder::is_pathed@00485a20`, run by `tools/emu/sweep_groupmoveorder_is_pathed.py` under unicorn
/// (tools/emu): the flag byte over all 256 values (offset 0x40), against `flag_reads::group_move_order_is_pathed` on the signed byte.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_groupmoveorder_is_pathed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want != 0,
            crate::flag_reads::group_move_order_is_pathed(a[0] as u8 as i8),
            "GroupMoveOrder::is_pathed — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("GroupMoveOrder::is_pathed@00485a20: {n} rows agree");
}
