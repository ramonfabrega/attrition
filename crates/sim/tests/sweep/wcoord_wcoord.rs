//! `WCoord::WCoord@00461450` under the emulator against the port (item 1819).

/// `WCoord::WCoord@00461450`, run by `tools/emu/sweep_wcoord_wcoord.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the `WCoord`, one stack argument: stores it at +0 and returns `this`; the printed result is the word stored.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_wcoord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(a[0], want, "WCoord::WCoord — row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("WCoord::WCoord@00461450: {n} rows agree");
}
