//! `WallData::hits@00642bb0` under the emulator against the port (item 1735).

/// `WallData::hits@00642bb0`, run by `tools/emu/sweep_wall_data_hits.py` under unicorn
/// (tools/emu): the int at `+0x20` for a non-zero argument and the one at `+0x54` for zero, against `flag_reads::wall_data_hits`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_hits.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (v20, v54) = (
            i32::try_from(a[1]).expect("int"),
            i32::try_from(a[2]).expect("int"),
        );
        assert_eq!(
            want,
            i64::from(crate::flag_reads::wall_data_hits(a[0] as i32, v20, v54)),
            "WallData::hits — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("WallData::hits@00642bb0: {n} rows agree");
}
