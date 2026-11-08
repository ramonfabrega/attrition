//! `GoodTypeData::is_flat@004780c0` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `GoodTypeData::is_flat@004780c0`, run by `tools/emu/sweep_good_type_data_is_flat.py` under unicorn
/// (tools/emu): its own slot `+0x60` asked with `(1, 0)`, `(4, 0)`, `(5, 0)` in that order, stopping at the first
/// non-zero answer; 1 when all three are zero. The calls made are held as well as the answer.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_data_is_flat.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let ans = &a[..3];
        let calls = ans.iter().position(|&v| v != 0).map_or(3, |k| k + 1);
        assert_eq!(a[3], calls as i64, "calls made — row `{line}`");
        assert_eq!(a[4], 1, "each call's ecx and arguments — row `{line}`");
        let got = i64::from(ans.iter().all(|&v| v == 0));
        assert_eq!(got, want, "is_flat — row `{line}`");
        n += 1;
    }
    assert!(n >= 220, "only {n} rows");
    eprintln!("GoodTypeData::is_flat@004780c0: {n} rows agree");
}
