//! `ObjectData::is_worker@0046fa10` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::orders::Worker;

/// `ObjectData::is_worker@0046fa10`, run by `tools/emu/sweep_object_is_worker.py` under unicorn
/// (tools/emu): `type+4` one of `0x32`..`0x35`, over every index 0..=0x1ff and the edges, against `Sim::worker_of(u) != Worker::None` — the port's loader builds the worker from the id (`type_index - 0x32`: citizens 0 and 1, scholars 2 and 3), so the row builds the same.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_is_worker.py") else {
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
        assert_eq!(
            s.worker_of(u) != Worker::None,
            want != 0,
            "is_worker — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectData::is_worker@0046fa10: {n} rows agree");
}
