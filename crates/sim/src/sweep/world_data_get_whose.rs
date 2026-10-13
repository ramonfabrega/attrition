//! `WorldData::get_whose@006b4d80` under the emulator against the port (item 1824).

/// `WorldData::get_whose@006b4d80`, run by `tools/emu/sweep_world_data_get_whose.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the world, two stack arguments (`ret 8`) pointers to `WCoord` words (x, then y): `wdata[y * xs + x].who`, a signed byte at `+0xf` of a 0x1c-byte record (`xs` at +0, `wdata` at `+0x134`). Rows `<xs> <x> <y> <who> <decoy>`; every other tile carries `decoy`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_get_whose.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = a[3];
        assert_eq!(got, want, "WorldData::get_whose — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WorldData::get_whose@006b4d80: {n} rows agree");
}
