//! `World::set_down@0046ef50` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `World::set_down@0046ef50`, run by `tools/emu/sweep_world_set_down.py` under unicorn
/// (tools/emu): two 16-bit stores into cell `y*width + x` of 0x1c bytes; the script itself holds every other byte of the array.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_down.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = (i64::from(a[4] as i32 & 0xffff) << 16) | i64::from(a[3] as i32 & 0xffff);
        assert_eq!(got, want, "set_down — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_down@0046ef50: {n} rows agree");
}
