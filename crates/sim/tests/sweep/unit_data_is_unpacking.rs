//! `UnitData::is_unpacking@0060a4b0` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py and tools/emu/orderhead.py).

/// `UnitData::is_unpacking@0060a4b0`, run by `tools/emu/sweep_unit_data_is_unpacking.py` under unicorn (tools/emu):
/// `calc_gather.rs`'s `is_unpacking` — the head order is a cast whose spell unpacks (`spell::is_unpack`). An empty list is no, unasked, its cursor
/// untouched; a list's cursor is rewound to the head (`+0xd4`, `+0xcc`, `+0xd0`, no other byte), a null head
/// order is no, unasked; a real one's slot `+0xf4` (the order's `ecx`) answers its spell record, null is no,
/// else the slot is asked a second time and the type the record's `+0x20` indexes in the type list is asked
/// its slot `0x54` (the type's own `ecx`): any non-zero word is 1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_unpacking.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (has, order, spell, ans) = (a[0] != 0, a[1] != 0, a[2] != 0, a[4]);
        let want = match (has, order, spell) {
            (false, _, _) => (0, ["", "ooo"]),
            (true, false, _) => (0, ["", "nnn"]),
            (true, true, false) => (0, ["spell:1", "nnn"]),
            (true, true, true) => (i64::from(ans != 0), ["spell:1,spell:1,asks:1", "nnn"]),
        };
        assert_eq!(
            (eax, tail.as_slice()),
            (want.0, &want.1[..]),
            "is_unpacking — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_unpacking@0060a4b0: {n} rows agree");
}
