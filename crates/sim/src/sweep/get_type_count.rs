//! `get_type_count@00669130` under the emulator against the port (item 1787).

/// `get_type_count@00669130`, run by `tools/emu/sweep_get_type_count.py` under unicorn
/// (tools/emu): the byte at `0x00ed89ce + row * 0x1ed + col` over seeded rows and columns (low byte), against `flag_reads::type_count`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_get_type_count.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let table: Vec<u8> = (0..6 * 0x1edu32)
            .map(|k| ((k * 131 + (k >> 8) * 17 + 5) & 0xFF) as u8)
            .collect();
        assert_eq!(
            want,
            i64::from(crate::flag_reads::type_count(
                &table,
                a[0] as usize,
                a[1] as usize
            )),
            "get_type_count — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("get_type_count@00669130: {n} rows agree");
}
