//! `BuildData::num_gather@0046f0a0` under the emulator against the port (item 1793).

/// `BuildData::num_gather@0046f0a0`, run by `tools/emu/sweep_build_data_num_gather.py` under unicorn
/// (tools/emu): the ints at `+0xc8` and `+0xcc`, against "`+0xc8` when `+0xcc` is non-zero, else 0".
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_num_gather.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (n8, nc) = (i32::try_from(a[0]).expect("a 32-bit integer"), i32::try_from(a[1]).expect("a 32-bit integer"));
        assert_eq!(want, i64::from(if nc != 0 { n8 } else { 0 }), "BuildData::num_gather — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::num_gather@0046f0a0: {n} rows agree");
}
