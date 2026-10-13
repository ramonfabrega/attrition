//! `WorldData::is_tocean@0046fb10` under the emulator against the port (item 1708).

use crate::world::{Pos, World};

/// `WorldData::is_tocean@0046fb10`, run by `tools/emu/sweep_world_is_tocean.py` under unicorn
/// (tools/emu): `(tdata[tile_xs * y + x].mask & 0x30) == 0x20` over a `TData` array in mapped memory, every bit of the mask alone and all-but-one, edge tiles of three widths, a decoy mask in every other tile so a wrong-tile read shows; each row's port `World` carries the same mask in the queried tile and the decoy elsewhere, against `World::is_tocean`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_tocean.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, x, y, mask, decoy) = (
            a[0] as i32,
            a[1] as i32,
            a[2] as i32,
            u16::try_from(a[3]).expect("a mask"),
            a[4],
        );
        let mut w = World::new(xs / 4, 1);
        let dm: u16 = if decoy == 0 { 0x20 } else { 0x10 };
        for ty in 0..4 {
            for tx in 0..xs {
                w.set_tile_mask(Pos::new(tx, ty), dm);
            }
        }
        w.set_tile_mask(Pos::new(x, y), mask);
        let ours = w.is_tocean(Pos::new(x, y));
        assert_eq!(i64::from(ours), want, "is_tocean — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::is_tocean@0046fb10: {n} rows agree");
}
