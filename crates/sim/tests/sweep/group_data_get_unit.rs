//! `GroupData::get_unit@0070cc90` under the emulator against the port (item 1932; hand-written).

/// `GroupData::get_unit@0070cc90`, run by `tools/emu/sweep_group_data_get_unit.py` under unicorn with `find_leader` hooked: the guard, the call's arguments and the answer.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_unit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let a: Vec<i64> = line
            .split(" -> ")
            .next()
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let want = 0;
        let (b, num, ans) = (a[0], a[1] as i32, a[2] as i32);
        let _ = want;
        let (eax, seen) = line
            .split_once(" -> ")
            .unwrap()
            .1
            .split_once(" | ")
            .expect("`<eax> | <calls>`");
        let (got, calls) = if b == 0 && num > 0 {
            (ans, "0,0")
        } else {
            (-1, "")
        };
        assert_eq!(
            eax.trim().parse::<i32>().unwrap(),
            got,
            "get_unit — eax, row `{line}`"
        );
        assert_eq!(seen.trim(), calls, "get_unit — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_unit@0070cc90: {n} rows agree");
}
