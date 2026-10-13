//! `GameAccessConst::get_scary_console_leader@005833e0` under the emulator against the port (item 1766).

/// `GameAccessConst::get_scary_console_leader@005833e0`, run by `tools/emu/sweep_game_access_const_get_scary_console_leader.py` under unicorn
/// (tools/emu): `0x00e3a390 + who * 0x6eec` over the word at `+0x298`, against `flag_reads::console_leader`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_console_leader.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let w = i32::try_from(a[0]).expect("a 32-bit word");
        let rec: Vec<i32> = (0..0x800)
            .map(|i| w.wrapping_add(0x0101_0101_i32.wrapping_mul(i)))
            .collect();
        assert_eq!(
            want,
            i64::from(crate::flag_reads::console_leader(&rec)),
            "GameAccessConst::get_scary_console_leader — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_console_leader@005833e0: {n} rows agree");
}
