//! `ObjectData::is_in_range@00648d70` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

/// The key the position words at `+0x10`/`+0x14` are stored under.
const KEY: i64 = 0x63637;

/// `ObjectData::is_in_range@00648d70`, run by `tools/emu/sweep_object_data_is_in_range_margin.py` under unicorn
/// (tools/emu): a forwarder that hands the seven-argument overload (hooked, so not backed here) `this` and
/// `(p0, p1, x, y, this, p3, 0)` — the object's own position de-keyed, its third argument never read — and answers
/// what it answers. `fight.rs`'s `is_in_range_at_margin` is the port's.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_in_range_margin.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (w10, w14, p0, p1, p3, ans) = (a[0], a[1], a[2], a[3], a[5], a[6]);
        assert_eq!(a[7], 1, "the overload's ecx is this — row `{line}`");
        assert_eq!(
            &a[8..15],
            &[p0, p1, w10 ^ KEY, w14 ^ KEY, 1, p3, 0],
            "the arguments — row `{line}`"
        );
        assert_eq!(ans, want, "is_in_range — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::is_in_range@00648d70: {n} rows agree");
}
