//! `TerrainVis::get_verts_per_bucket@008b0060` under the emulator against the port (item 1942; hand-written).

/// `TerrainVis::get_verts_per_bucket@008b0060`, run by `tools/emu/sweep_terrain_vis_get_verts_per_bucket.py` under unicorn: grade three and above answers 9, below it the table entry for the current zoom.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_vis_get_verts_per_bucket.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (res, seen) = rhs.split_once("| ").unwrap_or((rhs, ""));
        let res: Vec<i64> = res
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let _ = (&a, &res, seen);
        let (g, e) = (a[0], a[2]);
        assert_eq!(res[0], if g >= 3 { 9 } else { e }, "get_verts_per_bucket — row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("TerrainVis::get_verts_per_bucket@008b0060: {n} rows agree");
}
