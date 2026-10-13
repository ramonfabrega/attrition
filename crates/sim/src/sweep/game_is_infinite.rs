//! `Game::is_infinite@00594640` under the emulator against the port (item 1814).

/// `Game::is_infinite@00594640`, run by `tools/emu/sweep_game_is_infinite.py` under unicorn
/// (tools/emu): `__cdecl`-shaped (reads the `GameAccess::game` cell at 0xc061ec); true when the byte at game+0x2d is 8 — every byte value, against `b == 8`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_infinite.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] == 8);
        assert_eq!(got, want, "Game::is_infinite — row `{line}`");
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_infinite@00594640: {n} rows agree");
}
