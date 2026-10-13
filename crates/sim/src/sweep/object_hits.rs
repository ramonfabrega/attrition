//! `ObjectData::hits@006450c0` under the emulator against the port (item 1723).

/// `ObjectData::hits@006450c0`, run by `tools/emu/sweep_object_hits.py` under unicorn
/// (tools/emu): the int `myhits` at `this+0x20` with the argument never read, over 32-bit edges and a seeded band: the raw word.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_hits.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v),
            "ObjectData::hits — the raw word, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::hits@006450c0: {n} rows agree");
}
