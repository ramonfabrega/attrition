//! `World::get_wdata@0046d220` under the emulator against the port (item 1856).

/// `World::get_wdata@0046d220`, run by `tools/emu/sweep_world_get_wdata.py` under unicorn
/// (tools/emu): `__thiscall`, two stack arguments (`ret 8`): pointers to two `WCoord` words (x, y); `[this+0x134] + 28 * (y * [this] + x)`, 32-bit wrapping. Row `<xs> <x> <y> <base> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_wdata.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (a[3] as i32).wrapping_add(
            28i32.wrapping_mul(
                (a[2] as i32)
                    .wrapping_mul(a[0] as i32)
                    .wrapping_add(a[1] as i32),
            ),
        ) as i64;
        assert_eq!(got, want, "World::get_wdata — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("World::get_wdata@0046d220: {n} rows agree");
}
