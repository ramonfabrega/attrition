//! `WorldMapFog::get_fog_lookup@008d1830` under the emulator against the port (item 1942; hand-written).

/// `WorldMapFog::get_fog_lookup@008d1830`, run by `tools/emu/sweep_world_map_fog_get_fog_lookup.py` under unicorn: the answer is the byte at `(y+1)*(xs+2) + x + 1` of the table.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_map_fog_get_fog_lookup.py") else {
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
        let (xs, x, y) = (a[0], a[1], a[2]);
        let i = (y + 1) * (xs + 2) + x + 1;
        assert_eq!(
            res[0],
            (i * 37 + 11) & 0xff,
            "get_fog_lookup — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("WorldMapFog::get_fog_lookup@008d1830: {n} rows agree");
}
