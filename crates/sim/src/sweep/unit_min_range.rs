//! `UnitData::min_range@006100b0` under the emulator against the port (item 1694).

use crate::UnitType;
use crate::combat::{Obj, Profile};

/// `UnitData::min_range@006100b0`, run by `tools/emu/sweep_unit_min_range.py` under unicorn
/// (tools/emu): the int at `type+0x1f8` through the unit's type pointer, over 32-bit edges and a seeded band, against `Sim::profile(..).min_range` — the port's `min_range` column of the type's combat profile (the same read every consumer makes).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_min_range.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let (s, u) = super::sim_with_unit(UnitType {
            combat: Profile {
                min_range: v,
                ..Profile::default()
            },
            ..UnitType::default()
        });
        assert_eq!(
            i64::from(s.profile(Obj::Unit(u)).min_range),
            want,
            "min_range — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::min_range@006100b0: {n} rows agree");
}
