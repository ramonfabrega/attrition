//! `ObjectData::recharge@00646c30` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::combat::{Obj, Profile};

/// `ObjectData::recharge@00646c30`, run by `tools/emu/sweep_object_recharge.py` under unicorn
/// (tools/emu): the int at `type+500` (the argument ignored), over 32-bit edges and a seeded band, against `Sim::profile(..).recharge` — the type's `RECHARGE` column, the base `combat::recharge` is applied to. The base object's own method is the raw read; the supply and Bombard arms are `UnitData`'s override.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_recharge.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let (s, u) = super::sim_with_unit(UnitType {
            combat: Profile {
                recharge: v,
                ..Profile::default()
            },
            ..UnitType::default()
        });
        assert_eq!(
            i64::from(s.profile(Obj::Unit(u)).recharge),
            want,
            "recharge — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::recharge@00646c30: {n} rows agree");
}
