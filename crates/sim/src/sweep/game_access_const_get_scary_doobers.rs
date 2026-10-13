//! `GameAccessConst::get_scary_doobers@00583420` under the emulator against the port (item 1760).

/// `GameAccessConst::get_scary_doobers@00583420`, run by `tools/emu/sweep_game_access_const_get_scary_doobers.py` under unicorn
/// (tools/emu): the address of a global, whatever `ecx` holds, against `flag_reads::DOOBERS_ADDR`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_doobers.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::DOOBERS_ADDR),
            "GameAccessConst::get_scary_doobers — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 20, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_doobers@00583420: {n} rows agree");
}
