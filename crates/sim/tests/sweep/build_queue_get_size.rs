//! `BuildQueueData::get_size@006306b0` under the emulator against the port (item 1723).

/// `BuildQueueData::get_size@006306b0`, run by `tools/emu/sweep_build_queue_get_size.py` under unicorn
/// (tools/emu): the int `queue_size` at `this+0`, over 32-bit edges and a seeded band, against `production::Queue::capacity` (the port's `queue_size`) for the non-negative rows, and the raw word for the rest.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_get_size.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v),
            "BuildQueueData::get_size — the raw word, row `{line}`"
        );
        if v >= 0 {
            let q = crate::production::Queue {
                capacity: usize::try_from(v).expect("non-negative"),
                ..Default::default()
            };
            assert_eq!(
                want,
                i64::try_from(q.capacity).expect("fits"),
                "queue capacity — row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildQueueData::get_size@006306b0: {n} rows agree");
}
