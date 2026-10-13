//! `WorldData::get_goods@0046ef10` under the emulator against the port (item 1834).

/// `WorldData::get_goods@0046ef10`, run by `tools/emu/sweep_world_data_get_goods.py` under unicorn
/// (tools/emu): `__thiscall`, two stack pointers to `WCoord` words (x, then y); reads the `GameAccess::world` cell at 0xc06188: the unsigned byte at `+0xd` of `wdata[y * xs + x]` (0x1c-byte records, `xs` at +0, `wdata` at `+0x134`). Rows `<xs> <x> <y> <goods> <decoy>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_get_goods.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = a[3];
        assert_eq!(got, want, "WorldData::get_goods — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WorldData::get_goods@0046ef10: {n} rows agree");
}
