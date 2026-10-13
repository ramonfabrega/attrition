//! `Guy::set_pitch@005d86a0` under the emulator against the port (item 1932; hand-written).

/// `Guy::set_pitch@005d86a0`, run by `tools/emu/sweep_guy_set_pitch.py` under unicorn: the old word moves up one slot, the float's bits land
/// in its place, and nothing else of the object moves (the script checks).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_set_pitch.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (bits, old) = (a[0] as u32, a[1] as i32 as u32);
        let (new0, new1) = (bits, old);
        let got = (i64::from(new1) << 32) | i64::from(new0);
        assert_eq!(got, want, "set_pitch — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Guy::set_pitch@005d86a0: {n} rows agree");
}
