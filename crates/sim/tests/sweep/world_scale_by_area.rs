//! `World::scale_by_area@006b2b10` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `World::scale_by_area@006b2b10`, run by `tools/emu/sweep_world_scale_by_area.py` under unicorn
/// (tools/emu): `(sq/2 + field*p0) / sq` with `sq = div*div`, at least 1; the divisor sits behind the `map_sizes` global.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_scale_by_area.py") else {
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
        let (f, p, d) = (a[0] as i32, a[1] as i32, a[2] as i32);
        let sq = d.wrapping_mul(d);
        let got = ((sq / 2).wrapping_add(f.wrapping_mul(p)) / sq).max(1);
        assert_eq!(
            i64::from(got),
            rhs.parse::<i64>().unwrap(),
            "scale_by_area — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("World::scale_by_area@006b2b10: {n} rows agree");
}
