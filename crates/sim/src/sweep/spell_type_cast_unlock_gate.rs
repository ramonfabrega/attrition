//! `SpellType::cast_unlock_gate@00671440` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `SpellType::cast_unlock_gate@00671440`, run by `tools/emu/sweep_spell_type_cast_unlock_gate.py` under unicorn
/// (tools/emu): the byte at `+0x81` of the slot's answer set to 0, reached through `objects[p1][p0]`; the script holds every other record's byte.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_unlock_gate.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let (byte, seen) = rhs.split_once(" | ").expect("`<byte> | <selected>`");
        assert_eq!(
            byte.trim(),
            "0",
            "SpellType::cast_unlock_gate — the byte, row `{line}`"
        );
        assert_eq!(
            seen.trim(),
            "True",
            "SpellType::cast_unlock_gate — the slot was entered on objects[p1][p0], row `{line}`"
        );
        assert_eq!(
            lhs.split_whitespace().count(),
            3,
            "SpellType::cast_unlock_gate — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("SpellType::cast_unlock_gate@00671440: {n} rows agree");
}
