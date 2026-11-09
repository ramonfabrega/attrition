//! `WorldData::get_coll_block@006b5350` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `WorldData::get_coll_block@006b5350`, run by `tools/emu/sweep_world_data_get_coll_block.py` under unicorn
/// (tools/emu): the block word of cell `y*width + x`, or -1 when `p3 >= 0` and the cell region (signed 16-bit) differs.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_get_coll_block.py") else {
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
        let (p3, reg, blk) = (a[3], a[4], a[5]);
        let got = if p3 >= 0 && reg != p3 { -1 } else { blk };
        assert_eq!(
            got,
            rhs.parse::<i64>().unwrap(),
            "get_coll_block — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("WorldData::get_coll_block@006b5350: {n} rows agree");
}
