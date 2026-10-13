//! `GoodTypeData::avail_drop@0066fd40` under the emulator against the port (item 1723).

/// `GoodTypeData::avail_drop@0066fd40`, run by `tools/emu/sweep_good_type_avail_drop.py` under unicorn
/// (tools/emu): the int `random_rare_drop` at `this+0x2f8`, over 32-bit edges and a seeded band: the raw word, the port has no consumer of the field.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_avail_drop.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v),
            "GoodTypeData::avail_drop — the raw word, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GoodTypeData::avail_drop@0066fd40: {n} rows agree");
}
