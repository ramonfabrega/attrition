//! `World::scale_by_axis@006b76b0` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `World::scale_by_axis@006b76b0`, run by `tools/emu/sweep_world_scale_by_axis.py` under unicorn
/// (tools/emu): `(div/2 + width*v) / div`, at least 1; the divisor sits behind the `map_sizes` global.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_scale_by_axis.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let rhs = rhs.trim();
        let (v, w, d) = (a[0] as i32, a[1] as i32, a[2] as i32);
        let got = ((d / 2).wrapping_add(w.wrapping_mul(v)) / d).max(1);
        assert_eq!(
            i64::from(got),
            rhs.parse::<i64>().unwrap(),
            "scale_by_axis — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("World::scale_by_axis@006b76b0: {n} rows agree");
}
