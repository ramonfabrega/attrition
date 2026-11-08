//! `Caravan::distance@0073d380` under the emulator against the port (item 1966; tools/sweep_gen.py, cell).

/// `Caravan::distance@0073d380`, run by `tools/emu/sweep_caravan_distance.py` under unicorn
/// (tools/emu): the signed-32 edges and two hundred random words at the record's field, against `v as i64`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caravan_distance.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let e = i32::try_from(a[0]).expect("a 32-bit integer");
        let v = i32::try_from(a[1]).expect("a 32-bit integer");
        let got: i64 = if e < (v + ((v >> 31) & 3)) >> 2 {
            0
        } else if e < v / 2 {
            1
        } else {
            2 + i64::from((v.wrapping_shl(2)) / 5 <= e)
        };
        assert_eq!(got, want, "distance — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Caravan::distance@0073d380: {n} rows agree");
}
