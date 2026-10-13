//! `UnitData::is_moving@00610af0` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py and tools/emu/orderhead.py).

/// `UnitData::is_moving@00610af0`, run by `tools/emu/sweep_unit_data_is_moving.py` under unicorn (tools/emu):
/// `attack_pos.rs`'s `is_moving` asks the **head** order whether it is a move, and nothing else. An empty list
/// is no, unasked, its cursor untouched; a list's cursor is rewound to the head (`+0xd4`, `+0xcc`, `+0xd0`, no
/// other byte), a null head order is no, unasked, and a real one's slot `+0x14` (a `jmp`, the order's `ecx`)
/// answers for the function, whatever its word.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_moving.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (has, order, ans) = (a[0] != 0, a[1] != 0, a[2]);
        let want = match (has, order) {
            (false, _) => (0, ["", "ooo"]),
            (true, false) => (0, ["", "nnn"]),
            (true, true) => (ans, ["move:1", "nnn"]),
        };
        assert_eq!(
            (eax, tail.as_slice()),
            (want.0, &want.1[..]),
            "is_moving — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_moving@00610af0: {n} rows agree");
}
