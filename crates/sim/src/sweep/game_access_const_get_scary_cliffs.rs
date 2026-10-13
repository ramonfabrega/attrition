//! `GameAccessConst::get_scary_cliffs@00583370` under the emulator against the port (item 1760).

/// `GameAccessConst::get_scary_cliffs@00583370`, run by `tools/emu/sweep_game_access_const_get_scary_cliffs.py` under unicorn
/// (tools/emu): the address of a global, whatever `ecx` holds, against `flag_reads::CLIFFS_ADDR`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_cliffs.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::CLIFFS_ADDR),
            "GameAccessConst::get_scary_cliffs — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 20, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_cliffs@00583370: {n} rows agree");
}
