//! `ObjectData::is_scholar@0046d330` under the emulator against the port (item 1691).

use crate::UnitType;
use crate::orders::Worker;

/// `ObjectData::is_scholar@0046d330`, run by `tools/emu/sweep_object_is_scholar.py` under unicorn
/// (tools/emu): `type+4` one of `0x34`, `0x35`, over every index 0..=0x1ff and the edges, against `Sim::worker_of(u) == Worker::Scholar` — the test `garrison` and `lib` make inline (the port has no `is_scholar`). The worker is built from the id by `rondata::load`'s rule (`type_index - 0x32`).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_is_scholar.py") else {
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
            s.worker_of(u) == Worker::Scholar,
            want != 0,
            "is_scholar — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectData::is_scholar@0046d330: {n} rows agree");
}
