//! `WorldData::get_who@006b4700` under the emulator against the port (item 1721).

use crate::world::{Cell, Owner, World};

/// `WorldData::get_who@006b4700`, run by `tools/emu/sweep_world_get_who.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words, over a `WData` array laid out in mapped memory, answering the
/// cell's `who` (`WData++0xf`) sign-extended. The port reads it as `World::owner(c)`: -1 `None`, -2 `Ambiguous`, else the player.
/// Each row's port `World` has the same extent, the queried cell carrying the
/// row's value and every other cell the row's decoy, so a read of the wrong
/// cell shows. Cells off the map are left out.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_who.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let (v, decoy) = (a[4], a[5]);
        let mut w = World::new(xs, ys);
        let own = |n: i64| match n {
            -1 => Owner::None,
            -2 => Owner::Ambiguous,
            p => Owner::Player(u8::try_from(p).expect("a player")),
        };
        let dv = if decoy == 0 {
            Owner::Player(5)
        } else {
            Owner::None
        };
        for cy in 0..ys {
            for cx in 0..xs {
                w.set_owner(Cell { x: cx, y: cy }, dv, dv);
            }
        }
        let (a1, a2) = (own(v), dv);
        w.set_owner(Cell { x, y }, a1, a2);
        let ours: i64 = match w.owner(Cell { x, y }) {
            Owner::None => -1,
            Owner::Ambiguous => -2,
            Owner::Player(p) => i64::from(p),
        };
        assert_eq!(ours, want, "get_who — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("get_who@006b4700: {n} rows agree");
}
