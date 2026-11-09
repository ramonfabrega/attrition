//! `GameAccessConst::get_scary_console@005833c0` under the emulator against the port (item 1760).

/// `GameAccessConst::get_scary_console@005833c0`, run by `tools/emu/sweep_game_access_const_get_scary_console.py` under unicorn
/// (tools/emu): the word at the cell `0x00c06210`, against `flag_reads::global_cell`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_console.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::global_cell(
                i32::try_from(a[0]).expect("a 32-bit word")
            )),
            "GameAccessConst::get_scary_console — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_console@005833c0: {n} rows agree");
}
