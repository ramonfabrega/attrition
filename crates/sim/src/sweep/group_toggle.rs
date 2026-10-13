//! `Group::toggle@007083c0` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `Group::toggle@007083c0`, run by `tools/emu/sweep_group_toggle.py` under unicorn
/// (tools/emu): `member(p0, p1, 1)` picks slot 0x10 (member) or slot 0xc (not), both handed `p0, p1, 0, 0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_toggle.py") else {
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
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, p0, p1, m) = (a[0], a[1], a[2], a[3]);
        let (slot, ans) = if m != 0 {
            ("slot10", 1010)
        } else {
            ("slot0c", 2020)
        };
        let want = format!("member:{this},{p0},{p1},1;{slot}:{this},{p0},{p1},0,0");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            ans,
            "toggle — answer, row `{line}`"
        );
        assert_eq!(seen.trim(), want, "toggle — the calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("Group::toggle@007083c0: {n} rows agree");
}
