//! `GameAccessConst::get_scary_options@00583410` under the emulator against the port (item 1866).

/// `GameAccessConst::get_scary_options@00583410`, run by `tools/emu/sweep_game_access_const_get_scary_options.py` under unicorn
/// (tools/emu): the cell's own word, as signed 32.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_options.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(a[0] as u32 as i32),
            "GameAccessConst::get_scary_options — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_options@00583410: {n} rows agree");
}
