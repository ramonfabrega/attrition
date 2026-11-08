//! `WorldData::get_land@0046f9d0` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `WorldData::get_land@0046f9d0`, run by `tools/emu/sweep_world_data_get_land.py` under unicorn
/// (tools/emu): the callee is handed `ecx = this`, pointers to `table[c1>>8]` and `table[c2>>8]`, and `p3`; the answer is its own.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_get_land.py") else {
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
        let (c1, c2, p3, ans, this) = (a[0], a[1], a[2], a[3], a[4]);
        let tab = |c: i64| ((c >> 8) as u64).wrapping_mul(2_654_435_761) >> 5 & 0xffff;
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <call>`");
        let this = i64::from(this as u32 as i32);
        let want = format!("{},{},{},{}", this, tab(c1), tab(c2), p3);
        assert_eq!(seen.trim(), want, "get_land — the call, row `{line}`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            ans,
            "get_land — answer, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("WorldData::get_land@0046f9d0: {n} rows agree");
}
