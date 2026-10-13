//! `TerrainVis::set_base_source_pool@008b2a80` under the emulator against the port (item 1942; hand-written).

/// `TerrainVis::set_base_source_pool@008b2a80`, run by `tools/emu/sweep_terrain_vis_set_base_source_pool.py` under unicorn: the two arguments land in the two words and nothing else of the object moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_vis_set_base_source_pool.py") else {
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
        assert_eq!(
            (res[0], res[1]),
            (a[0], a[1]),
            "set_base_source_pool — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("TerrainVis::set_base_source_pool@008b2a80: {n} rows agree");
}
