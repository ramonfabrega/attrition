//! `UnitData::can_ever_transport@0046f290` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::transport::can_ever_transport_by;

/// `UnitData::can_ever_transport@0046f290`, run by `tools/emu/sweep_unit_data_can_ever_transport.py` under
/// unicorn (tools/emu): the domain, `this`'s slot `+0x18`, the looked-up unit's carry and `is(0x15f, 0)` — through
/// slot `+0xb8`, or the record's `+0x60` when the slot is `ObjectData::is` — each drawn per row. The port's
/// [`can_ever_transport_by`] is handed the row's answers and records the asks, so the order and the short circuit
/// are held to the original's calls (the `is` arm named by the row's mode), and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_can_ever_transport.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let calls = std::cell::RefCell::new(Vec::new());
        let got = can_ever_transport_by(
            a[1] == 0,
            || {
                calls.borrow_mut().push("s18,this".to_string());
                a[2] != 0
            },
            || a[3] != 0,
            || {
                calls.borrow_mut().push(
                    if a[0] == 1 {
                        "rec60,rec,351,0"
                    } else {
                        "is,this,351,0"
                    }
                    .to_string(),
                );
                a[4] != 0
            },
        );
        assert_eq!(
            tail[0],
            calls.borrow().join(";"),
            "can_ever_transport — the asks, row `{line}`"
        );
        assert_eq!(
            i64::from(got),
            want,
            "can_ever_transport — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitData::can_ever_transport@0046f290: {n} rows agree");
}
