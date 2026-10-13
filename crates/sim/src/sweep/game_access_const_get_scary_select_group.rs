//! `GameAccessConst::get_scary_select_group@00583450` under the emulator against the port (item 1787).

/// `GameAccessConst::get_scary_select_group@00583450`, run by `tools/emu/sweep_game_access_const_get_scary_select_group.py` under unicorn
/// (tools/emu): the word at `0x00e8d444` plus `0x9f0 *` the word at `+0x2a0` of the console player's record (words seeded `w + 0x01010101 * i`; the base word `~w`), against `flag_reads::select_group`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_get_scary_select_group.py") else {
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
            i64::from(crate::flag_reads::select_group(&rec, !w)),
            "GameAccessConst::get_scary_select_group — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("GameAccessConst::get_scary_select_group@00583450: {n} rows agree");
}
