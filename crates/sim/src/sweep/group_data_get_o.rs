//! `GroupData::get_o@0070f8b0` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `GroupData::get_o@0070f8b0`, run by `tools/emu/sweep_group_data_get_o.py` under unicorn
/// (tools/emu): a count gate, a buildings-byte branch answering a 16-bit word, else a tail call to `find_leader`; the out-pointer is zeroed first.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_o.py") else {
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
        let (eax, rest) = rhs.split_once(" | ").expect("`<eax> | <call> | <after>`");
        let (seen, after) = rest.split_once(" | ").expect("`<call> | <after>`");
        let (this, ptr, num, bld, word, ans) = (a[0], a[1], a[2], a[3], a[4], a[5]);
        let (want, call) = if num < 1 {
            (-1, String::new())
        } else if bld != 0 {
            (i64::from(word as i16), String::new())
        } else {
            (ans, format!("{this},{ptr}"))
        };
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            want,
            "get_o — answer, row `{line}`"
        );
        assert_eq!(seen.trim(), call, "get_o — the call, row `{line}`");
        let want_after = if ptr != 0 { 0 } else { -1 };
        assert_eq!(
            after.trim().parse::<i64>().unwrap(),
            want_after,
            "get_o — the out-pointer, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("GroupData::get_o@0070f8b0: {n} rows agree");
}
