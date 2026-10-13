//! `World::set_tocean@006b1c60` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `World::set_tocean@006b1c60`, run by `tools/emu/sweep_world_set_tocean.py` under unicorn
/// (tools/emu): a 16-bit read-modify-write of cell `y*width + x`; the script itself holds every other byte of the array.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_tocean.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let rhs = rhs.trim();
        let before = a[3] as u16;
        let got = i64::from((before & 0xffef) | 0x20);
        assert_eq!(
            got,
            rhs.parse::<i64>().unwrap(),
            "set_tocean — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("World::set_tocean@006b1c60: {n} rows agree");
}
