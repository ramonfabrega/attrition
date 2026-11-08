//! `TerrainMap::river_point@009b32f0` under the emulator against the port (item 1942; hand-written).

/// `TerrainMap::river_point@009b32f0`, run by `tools/emu/sweep_terrain_map_river_point.py` under unicorn: the callee is entered with `ecx` the current river's pointer plus 8 and both arguments, in order.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_river_point.py") else {
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
        let (cur, p0, p1) = (a[0] as i32, a[1] as i32, a[2] as i32);
        let want = format!("{},{p0},{p1}", cur.wrapping_add(8) as u32);
        assert_eq!(seen.trim(), want, "river_point — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("TerrainMap::river_point@009b32f0: {n} rows agree");
}
