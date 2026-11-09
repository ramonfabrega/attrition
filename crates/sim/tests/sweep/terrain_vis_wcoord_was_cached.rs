//! `TerrainVis::wcoord_was_cached@008b13b0` under the emulator against the port (item 1932; hand-written).

/// `TerrainVis::wcoord_was_cached@008b13b0`, run by `tools/emu/sweep_terrain_vis_wcoord_was_cached.py` under unicorn: a coordinate's cached frame equals the terrain's last-moved frame.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_vis_wcoord_was_cached.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (frame, moved) = (a[1] as i32, a[2] as i32);
        let got = i64::from(frame == moved);
        assert_eq!(got, want, "wcoord_was_cached — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainVis::wcoord_was_cached@008b13b0: {n} rows agree");
}
