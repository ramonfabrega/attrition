//! `UnitData::needs_transport@00609920` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

use crate::transport::{DISEMBARK, EMBARK};
use crate::world::tile;

/// `UnitData::needs_transport@00609920`, run by `tools/emu/sweep_unit_data_needs_transport.py` under unicorn
/// (tools/emu): the same tile is 0 unread; otherwise each tile's surface field is ocean or not, and the answer is
/// 0 on one side of the shore, [`DISEMBARK`] from ocean to land and [`EMBARK`] from land to ocean — the port's
/// `transport.rs` `needs_transport`, held here on its own constants and [`tile::SURFACE_OCEAN`].
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_needs_transport.py") else {
        return;
    };
    let water = |m: i64| m as u16 & tile::SURFACE == tile::SURFACE_OCEAN;
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = if (a[0], a[1]) == (a[2], a[3]) {
            0
        } else {
            match (water(a[4]), water(a[5])) {
                (true, false) => DISEMBARK,
                (false, true) => EMBARK,
                _ => 0,
            }
        };
        assert_eq!(i64::from(got), want, "needs_transport — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::needs_transport@00609920: {n} rows agree");
}
