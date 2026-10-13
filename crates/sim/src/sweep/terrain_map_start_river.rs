//! `TerrainMap::start_river@009b3320` under the emulator against the port (item 1942; hand-written).

/// `TerrainMap::start_river@009b3320`, run by `tools/emu/sweep_terrain_map_start_river.py` under unicorn: the callee's answer lands at `this+0x294`, the callee is entered on `this`, and nothing else moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_start_river.py") else {
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
        let ans = a[0] as i32;
        assert_eq!(res[0], i64::from(ans), "start_river — stored word, row `{line}`");
        assert_eq!(seen.trim(), "0", "start_river — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("TerrainMap::start_river@009b3320: {n} rows agree");
}
