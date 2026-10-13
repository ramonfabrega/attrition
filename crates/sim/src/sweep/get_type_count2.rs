//! `get_type_count2@00669110` under the emulator against the port (item 1782).

/// `get_type_count2@00669110`, run by `tools/emu/sweep_get_type_count2.py` under unicorn
/// (tools/emu): the byte at `0x00ed9936 + ecx` over 256 seeded entries (low byte of the result), against `flag_reads::type_count_at`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_get_type_count2.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let table: Vec<u8> = (0..256u32).map(|i| ((i * 7 + 3) & 0xFF) as u8).collect();
        assert_eq!(
            want,
            i64::from(crate::flag_reads::type_count_at(&table, a[0] as usize)),
            "get_type_count2 — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("get_type_count2@00669110: {n} rows agree");
}
