//! `ObjectData::los@006450b0` under the emulator against the port (item 1723).

/// `ObjectData::los@006450b0`, run by `tools/emu/sweep_object_los.py` under unicorn
/// (tools/emu): the signed char `mylos` at `this+0x3c` sign-extended, every value -128..=127, against `i32::from(i8)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_los.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i8::try_from(a[0]).expect("a signed char");
        assert_eq!(
            want,
            i64::from(i32::from(v)),
            "ObjectData::los — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("ObjectData::los@006450b0: {n} rows agree");
}
