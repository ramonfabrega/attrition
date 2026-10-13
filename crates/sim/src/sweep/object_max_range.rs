//! `ObjectData::max_range@00646c00` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::combat::{Obj, Profile};

/// `ObjectData::max_range@00646c00`, run by `tools/emu/sweep_object_max_range.py` under unicorn
/// (tools/emu): the int at `type+0x1fc` through the object's type pointer, over 32-bit edges and a seeded band, against `Sim::profile(..).max_range` — the `max_range` column of the type's combat profile.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_max_range.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let (s, u) = super::sim_with_unit(UnitType {
            combat: Profile {
                max_range: v,
                ..Profile::default()
            },
            ..UnitType::default()
        });
        assert_eq!(
            i64::from(s.profile(Obj::Unit(u)).max_range),
            want,
            "max_range — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::max_range@00646c00: {n} rows agree");
}
