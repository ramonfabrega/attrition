//! `WCoordData::walk_data@00681b20` under the emulator against the port (item 1911; tools/sweep_gen.py, walk).

/// `WCoordData::walk_data@00681b20`, run by `tools/emu/sweep_wcoord_data_walk_data.py` under unicorn
/// (tools/emu) with the walker's callback hooked: it is entered once, `ecx` the walker,
/// with `this` and `this+0x8` of the object, in that order.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_data_walk_data.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, walker) = (a[0], a[1]);
        let want = format!("{walker},{},{}", this, this + 0x8);
        assert_eq!(seen.trim(), want, "walk_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("WCoordData::walk_data@00681b20: {n} rows agree");
}
