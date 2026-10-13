//! `ObjectData::has_range@00646c10` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::combat::{Obj, Profile};

/// `ObjectData::has_range@00646c10`, run by `tools/emu/sweep_object_has_range.py` under unicorn
/// (tools/emu): the same word as `max_range` (`type+0x1fc`), returned raw — not a boolean — against `Sim::profile(..).max_range` for the value, and against `Profile::is_ranged` (`max_range != 0`, the port's `has_range`) for its truth.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_has_range.py") else {
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
        let p = s.profile(Obj::Unit(u));
        assert_eq!(
            i64::from(p.max_range),
            want,
            "has_range value — row `{line}`"
        );
        assert_eq!(p.is_ranged(), want != 0, "has_range truth — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::has_range@00646c10: {n} rows agree");
}
