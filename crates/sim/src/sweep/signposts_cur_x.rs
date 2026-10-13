//! `Signposts::cur_x@00897a60` under the emulator against the port (item 1760).

/// `Signposts::cur_x@00897a60`, run by `tools/emu/sweep_signposts_cur_x.py` under unicorn
/// (tools/emu): the word at the cell `0x00eb1a8c`, against `flag_reads::global_cell`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_signposts_cur_x.py") else {
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
            "Signposts::cur_x — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("Signposts::cur_x@00897a60: {n} rows agree");
}
