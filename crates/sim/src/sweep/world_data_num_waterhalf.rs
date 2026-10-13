//! `WorldData::num_waterhalf@006b4db0` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `WorldData::num_waterhalf@006b4db0`, run by `tools/emu/sweep_world_data_num_waterhalf.py` under unicorn
/// (tools/emu), held to `World::num_waterhalf` on a world of the row's size whose only non-zero cell and tiles
/// are the ones the row prints: the original's world is random everywhere, so a read of any other cell or tile
/// would part.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::world::{Cell, CellData, Pos, TILES_PER_CELL, World};
    let Some(rows) = super::emu_rows("sweep_world_data_num_waterhalf.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax) = super::row(line);
        let (w, h, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let mut world = World::new(w, h);
        let c = Cell::new(x, y);
        world.set_cell_data(
            c,
            CellData {
                flags: a[4] as u16,
                ..CellData::default()
            },
        );
        for (i, &m) in a[5..21].iter().enumerate() {
            let i = i as i32;
            let t = Pos::new(x * TILES_PER_CELL + (i & 3), y * TILES_PER_CELL + (i >> 2));
            world.set_tile_mask(t, m as u16);
        }
        assert_eq!(
            eax,
            i64::from(world.num_waterhalf(c)),
            "num_waterhalf — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("WorldData::num_waterhalf@006b4db0: {n} rows agree");
}
