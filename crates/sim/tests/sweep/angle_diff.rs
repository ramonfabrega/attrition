//! `angle_diff@0092d0b0` under the emulator against the port (item 1814).

/// `angle_diff@0092d0b0`, run by `tools/emu/sweep_angle_diff.py` under unicorn
/// (tools/emu): `ecx`/`edx` arguments, no frame — `a - b` wrapping, complemented above 0x80000000; the edge words crossed and four hundred random pairs, against that.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_angle_diff.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (x, y) = (
            u32::try_from(a[0]).expect("u32"),
            u32::try_from(a[1]).expect("u32"),
        );
        let d = x.wrapping_sub(y);
        let got = i64::from(if d > 0x8000_0000 { !d } else { d });
        assert_eq!(got, want, "angle_diff — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("angle_diff@0092d0b0: {n} rows agree");
}
