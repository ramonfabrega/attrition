//! `vector_dist@0046f020` under the emulator against the port (item 1998; hand-written on tools/emu/callfn.py).

/// `vector_dist@0046f020`, run by `tools/emu/sweep_vector_dist_ints.py` under unicorn
/// (tools/emu), the four-int overload (`x0` in `ecx`, `y0` in `edx`, `x1`, `y1` on the stack): the whole
/// octagonal distance of `(x0 - x1, y0 - y1)`, `vector_dist@0046cff0` unhooked, against
/// [`crate::world::vector_dist`]. Differences are held inside ±2^30 (the script says why).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_vector_dist_ints.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (dx, dy) = ((a[0] - a[2]) as i32, (a[1] - a[3]) as i32);
        let got = i64::from(crate::world::vector_dist(dx, dy));
        assert_eq!(got, want, "vector_dist@0046f020 — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("vector_dist@0046f020: {n} rows agree");
}
