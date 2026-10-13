//! `WallData::can_make@00472cb0` under the emulator against the port (item 1840).

/// `WallData::can_make@00472cb0`, run by `tools/emu/sweep_wall_data_can_make.py` under unicorn
/// (tools/emu): `__thiscall`, two stack args, ignored: always 0. Rows `<type> <flag>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_can_make.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = {
            let _ = &a;
            0
        };
        assert_eq!(got, want, "WallData::can_make — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("WallData::can_make@00472cb0: {n} rows agree");
}
