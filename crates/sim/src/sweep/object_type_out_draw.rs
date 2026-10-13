//! `ObjectTypeOut::draw@004703f0` under the emulator against the port (item 1866).

/// `ObjectTypeOut::draw@004703f0`, run by `tools/emu/sweep_object_type_out_draw.py` under unicorn
/// (tools/emu): `0`, and the stack drops exactly the six words (the script checks `esp`).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_out_draw.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "ObjectTypeOut::draw — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectTypeOut::draw@004703f0: {n} rows agree");
}
