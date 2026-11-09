//! `UnitType::gather_radius@0061daa0` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

use crate::fish::FISHERMEN;

/// `UnitType::gather_radius@0061daa0`, run by `tools/emu/sweep_unit_type_gather_radius.py` under unicorn
/// (tools/emu): the three merchant ids (`0x3d`, `0x3e`, `0x190`) and anything the `is` slot says is in the
/// [`FISHERMEN`] lineage reach `level × 4 + 4`, everything else `level + 2`, `level` the type's upgrade level from
/// slot `+0x124`; the merchants are never asked `is`. `calc_gather.rs`'s `gather_radius` is the port, its
/// `MERCHANTS` the three ids.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_gather_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (id, is, level) = (a[0], a[1], a[2] as i32);
        assert_eq!(a[6], 1, "every slot's ecx is this — row `{line}`");
        let merchant = matches!(id, 0x3d | 0x3e | 0x190);
        if merchant {
            assert_eq!(a[3], 2, "a merchant asks only the level — row `{line}`");
        } else {
            assert_eq!(a[3], 12, "is, then the level — row `{line}`");
            assert_eq!(
                (a[4], a[5]),
                (FISHERMEN as i64, 0),
                "is(FISHERMEN, 0) — row `{line}`"
            );
        }
        let got = if merchant || is != 0 {
            level.wrapping_mul(4).wrapping_add(4)
        } else {
            level.wrapping_add(2)
        };
        assert_eq!(i64::from(got), want, "gather_radius — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitType::gather_radius@0061daa0: {n} rows agree");
}
