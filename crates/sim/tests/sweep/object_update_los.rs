//! `Object::update_los@00646fe0` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

/// `Object::update_los@00646fe0`, run by `tools/emu/sweep_object_update_los.py` under unicorn (tools/emu): the
/// owner's leader record's bit 0 gates — clear, the object's sight byte (`+0x3c`) is zeroed and 0 answered; set,
/// the type record's signed byte at `+0x21c` is copied there and answered. No other byte of the object moves.
/// `vision.rs`'s `update_los` is the unit's override, which the port keeps; this is the object base.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_update_los.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (leader, ty, after, rest) = (a[1], a[2], a[4], a[5]);
        let los = if leader & 1 == 0 { 0 } else { ty };
        assert_eq!(rest, 1, "only the sight byte moves — row `{line}`");
        assert_eq!(after, los, "the stored sight — row `{line}`");
        assert_eq!(los, want, "update_los — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Object::update_los@00646fe0: {n} rows agree");
}
