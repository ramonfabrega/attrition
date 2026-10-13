//! `ObjectData::is_peasant@0046d310` under the emulator against the port (item 1691).

use crate::UnitType;
use crate::orders::Worker;

/// `ObjectData::is_peasant@0046d310`, run by `tools/emu/sweep_object_is_peasant.py` under unicorn
/// (tools/emu): `type+4` one of `0x32`, `0x33`, over every index 0..=0x1ff and the edges, against `Sim::is_peasant`. The port reads `UnitType::worker`, which `rondata::load` assigns by `type_index - 0x32` (0, 1 Citizen; 2, 3 Scholar); the row builds the worker by that same rule, so what is proved is the id set and that the Scholar ids answer no.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_is_peasant.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let worker = match a[0] - 0x32 {
            0 | 1 => Worker::Citizen,
            2 | 3 => Worker::Scholar,
            _ => Worker::None,
        };
        let (s, u) = super::sim_with_unit(UnitType {
            worker,
            ..UnitType::default()
        });
        assert_eq!(s.is_peasant(u), want != 0, "is_peasant — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectData::is_peasant@0046d310: {n} rows agree");
}
