//! `WallOut::get_gpiece@006424d0` under the emulator against the port (item 1724).

/// `WallOut::get_gpiece@006424d0`, run by `tools/emu/sweep_wall_out_get_gpiece.py` under unicorn
/// (tools/emu): the int `+0x68`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_out_get_gpiece.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v),
            "WallOut::get_gpiece — the raw word, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallOut::get_gpiece@006424d0: {n} rows agree");
}
