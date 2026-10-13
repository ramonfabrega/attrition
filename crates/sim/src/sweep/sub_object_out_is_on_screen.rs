//! `SubObjectOut::is_on_screen@00662670` under the emulator against the port (item 1724).

/// `SubObjectOut::is_on_screen@00662670`, run by `tools/emu/sweep_sub_object_out_is_on_screen.py` under unicorn
/// (tools/emu): the uchar at `+0x1c` zero-extended, every value 0..=255.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sub_object_out_is_on_screen.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(want, a[0], "SubObjectOut::is_on_screen — row `{line}`");
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("SubObjectOut::is_on_screen@00662670: {n} rows agree");
}
