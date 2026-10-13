//! `get_unique_types@00669120` under the emulator against the port (item 1782).

/// `get_unique_types@00669120`, run by `tools/emu/sweep_get_unique_types.py` under unicorn
/// (tools/emu): the byte at the cell `0x00ed88cb` over all 256 values (low byte of the result), against `flag_reads::global_byte`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_get_unique_types.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::global_byte(a[0] as u8)),
            "get_unique_types — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("get_unique_types@00669120: {n} rows agree");
}
