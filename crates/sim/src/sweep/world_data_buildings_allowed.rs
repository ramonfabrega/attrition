//! `WorldData::buildings_allowed@006b2340` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `WorldData::buildings_allowed@006b2340`, run by `tools/emu/sweep_world_data_buildings_allowed.py` under unicorn
/// (tools/emu): `1` when `word & 0x78 == 0`; the cell is `y*width + x` of 0x1c bytes behind the world global.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_buildings_allowed.py") else {
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
        let got = i64::from(a[3] & 0x78 == 0);
        assert_eq!(
            got,
            rhs.parse::<i64>().unwrap(),
            "buildings_allowed — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("WorldData::buildings_allowed@006b2340: {n} rows agree");
}
