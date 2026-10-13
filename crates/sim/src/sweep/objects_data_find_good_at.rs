//! `ObjectsData::find_good_at@00658160` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `ObjectsData::find_good_at@00658160`, run by `tools/emu/sweep_objects_data_find_good_at.py` under unicorn
/// (tools/emu): the `Coord` overload — `x` and `y` through `div_3_table` at `c >> 8`, handed to the `WCoord`
/// overload (hooked, so not backed here) as `(wx, wy, -1, 1, 0)` with `ecx` = `x >> 8` (the listing's own register, not
/// `this`); the callee's answer returned, the two last arguments unread.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_good_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (x, wx, wy, ans) = (a[0], a[4], a[5], a[6]);
        assert_eq!(a[7], x >> 8, "the callee's ecx — row `{line}`");
        assert_eq!(
            &a[8..13],
            &[wx, wy, -1, 1, 0],
            "the arguments — row `{line}`"
        );
        assert_eq!(ans, want, "find_good_at — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectsData::find_good_at@00658160: {n} rows agree");
}
