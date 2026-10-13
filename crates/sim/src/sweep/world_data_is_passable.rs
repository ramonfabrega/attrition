//! `WorldData::is_passable@006b23c0` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `WorldData::is_passable@006b23c0`, run by `tools/emu/sweep_world_data_is_passable.py` under unicorn
/// (tools/emu): `1` when `cell & 0x70 == 0`; the cell is `y*width + x` of 0x1c bytes behind the world global.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_passable.py") else {
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
        let got = i64::from(a[3] & 0x70 == 0);
        assert_eq!(
            got,
            rhs.parse::<i64>().unwrap(),
            "is_passable — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("WorldData::is_passable@006b23c0: {n} rows agree");
}
