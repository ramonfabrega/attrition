//! `Signposts::get_num@00897bc0` under the emulator against the port (item 1760).

/// `Signposts::get_num@00897bc0`, run by `tools/emu/sweep_signposts_get_num.py` under unicorn
/// (tools/emu): the word at the cell `0x00eb1a80`, against `flag_reads::global_cell`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_signposts_get_num.py") else {
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
            "Signposts::get_num — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("Signposts::get_num@00897bc0: {n} rows agree");
}
