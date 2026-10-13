//! `Game::is_sandbox@005947e0` under the emulator against the port (item 1851).

/// `Game::is_sandbox@005947e0`, run by `tools/emu/sweep_game_is_sandbox.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: through the `GameAccess::game` cell `0xc061ec`, 0 when the byte at `+0x820` has bit 2, else 1 when the int `num_nations` at `+0x6a0` is at most 1 (signed). Row `<flags> <nations>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_sandbox.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] & 4 == 0 && a[1] <= 1);
        assert_eq!(got, want, "Game::is_sandbox — row `{line}`");
        n += 1;
    }
    assert!(n >= 70, "only {n} rows");
    eprintln!("Game::is_sandbox@005947e0: {n} rows agree");
}
