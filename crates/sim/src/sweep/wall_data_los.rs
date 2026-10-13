//! `WallData::los@0063fa50` under the emulator against the port (item 1724).

/// `WallData::los@0063fa50`, run by `tools/emu/sweep_wall_data_los.py` under unicorn
/// (tools/emu): the signed char at `+0x3c` sign-extended, every value -128..=127.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_los.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i8::try_from(a[0]).expect("a signed char");
        assert_eq!(
            want,
            i64::from(i32::from(v)),
            "WallData::los — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("WallData::los@0063fa50: {n} rows agree");
}
