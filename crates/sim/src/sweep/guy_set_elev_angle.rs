//! `Guy::set_elev_angle@005d8fb0` under the emulator against the port (item 1921; hand-written, an xmm argument).

/// The port: the angle's bits go to `this+0x40` and nothing else moves (the script checks the
/// other bytes and `esp`'s `ret 4`).
fn set_elev_angle(bits: i32) -> i32 {
    bits
}

/// `Guy::set_elev_angle@005d8fb0`, run by `tools/emu/sweep_guy_set_elev_angle.py` under unicorn
/// (tools/emu): the float in `xmm1` — edges, NaNs and two hundred random words — lands unchanged.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_set_elev_angle.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let bits = i32::try_from(a[0]).expect("a 32-bit word");
        assert_eq!(
            i64::from(set_elev_angle(bits)),
            want,
            "set_elev_angle — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Guy::set_elev_angle@005d8fb0: {n} rows agree");
}
