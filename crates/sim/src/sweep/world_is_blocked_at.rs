//! `WorldData::is_blocked_at@00461340` under the emulator against the port (item 1630).

use crate::Pos;
use crate::world::{World, tile};

/// `WorldData::is_blocked_at@00461340`, run by
/// `tools/emu/sweep_world_is_blocked_at.py` under unicorn (tools/emu): `ecx`
/// the world, pointers to two `TCoord` words and a `mode`, over a `TData`
/// array laid out in mapped memory. Mode 0 answers `mask & 0x4000` raw;
/// otherwise 1 for a blocked tile whose surface field is not `0x30`. The
/// port's counterpart is `path.rs`'s private `is_blocked_at(tx, ty, mode)`,
/// which a sibling module cannot call, so each row builds a `World` of the
/// same tile extent (the queried tile carrying the row's mask, every other
/// tile the row's decoy) and asserts `World::tile_mask` with the port's
/// own predicate (`BLOCKED` set, and not `mode && SURFACE == SURFACE_FOREST`)
/// against the original, mode 0 also against the raw `0x4000` bit.
/// Left out: tiles off the map, where the original reads outside its array.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_blocked_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (txs, tys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let mask = u16::try_from(a[4]).expect("a 16-bit mask");
        let mode = a[5] != 0;
        let decoy = [0u16, 0x4000, 0x4030][a[6] as usize];
        let mut w = World::new(txs / 4, tys / 4);
        for ty in 0..tys {
            for tx in 0..txs {
                w.set_tile_mask(Pos::new(tx, ty), decoy);
            }
        }
        w.set_tile_mask(Pos::new(x, y), mask);
        let m = w.tile_mask(Pos::new(x, y));
        let ours = m & tile::BLOCKED != 0 && !(mode && m & tile::SURFACE == tile::SURFACE_FOREST);
        assert_eq!(
            i64::from(ours),
            i64::from(want != 0),
            "is_blocked_at — row `{line}`"
        );
        if !mode {
            assert_eq!(i64::from(m & 0x4000), want, "raw mask bit — row `{line}`");
        } else {
            assert_eq!(i64::from(ours), want, "mode 1 is 0 or 1 — row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_blocked_at@00461340: {n} rows agree");
}
