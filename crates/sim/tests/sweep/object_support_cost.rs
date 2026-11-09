//! `ObjectData::support_cost@006452f0` under the emulator against the port (item 1723).

/// `ObjectData::support_cost@006452f0`, run by `tools/emu/sweep_object_support_cost.py` under unicorn
/// (tools/emu): `*(int*)(type+0x270+i*4) << 8` through the type pointer at `this+0x18` with decoys in every neighbouring slot, against `wrapping_shl(8)` of the planted word (the 8.8 scale the port keeps).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_support_cost.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (v, i) = (i32::try_from(a[1]).expect("a 32-bit integer"), a[0]);
        assert!((0..48).contains(&i));
        assert_eq!(
            want,
            i64::from(v.wrapping_shl(8)),
            "ObjectData::support_cost — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::support_cost@006452f0: {n} rows agree");
}
