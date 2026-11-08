//! `vector_dist@0046d060` under the emulator against the port (item 1998; hand-written on tools/emu/callfn.py).

/// `vector_dist@0046d060`, run by `tools/emu/sweep_vector_dist_coords.py` under unicorn
/// (tools/emu), the four-`Coord*` overload (`&x0` in `ecx`, `&y0` in `edx`, `&x1`, `&y1` on the stack, read and
/// never written): the same distance as `vector_dist@0046f020`, against [`crate::world::vector_dist`].
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_vector_dist_coords.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (dx, dy) = ((a[0] - a[2]) as i32, (a[1] - a[3]) as i32);
        let got = i64::from(crate::world::vector_dist(dx, dy));
        assert_eq!(got, want, "vector_dist@0046d060 — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("vector_dist@0046d060: {n} rows agree");
}
