//! `CityData::depop_can_queue@00738060` under the emulator against the port (item 2072; hand-written on
//! tools/emu/hooks.py).

/// `CityData::depop_can_queue@00738060`, run by `tools/emu/sweep_city_data_depop_can_queue.py` under unicorn
/// (tools/emu): the city's own object (through the owner's table) is asked for its data record, the record's
/// vtable slot `+0x190` is asked `(1, 0x286)`, and the city may queue a depopulation exactly when that answer is
/// 0. The port has no consumer of it (`city.rs` queues no depopulation); the row holds the two calls, their
/// objects and arguments, and the answer `v == 0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_depop_can_queue.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            &a[1..],
            &[2, 1, 1, 1, 0x286],
            "depop_can_queue — the calls, row `{line}`"
        );
        assert_eq!(i64::from(a[0] == 0), want, "depop_can_queue — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("CityData::depop_can_queue@00738060: {n} rows agree");
}
