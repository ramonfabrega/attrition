//! `Balance::walk_rules_data@00582cc0` under the emulator against the port (item 2672; tools/sweep_gen.py, walk).

/// `Balance::walk_rules_data@00582cc0`, run by `tools/emu/sweep_balance_walk_rules_data.py` under unicorn
/// (tools/emu) with the walker's callback hooked: it is entered once, `ecx` the walker,
/// with `0xc12bf4` and `0xc12bf6` of the object, in that order.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_walk_rules_data.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, walker) = (a[0], a[1]);
        let _ = this;
        let pairs: Vec<(i64, i64)> = (0..0x1edi64 * 0x1ed).map(|k| (0xc12bf4 + 2 * k, 0xc12bf6 + 2 * k)).collect();
        let fmt = |p: &(i64, i64)| format!("{walker},{},{}", p.0, p.1);
        let sum = |f: fn(&(i64, i64)) -> i64| pairs.iter().map(|p| f(p) as u32 as u64).sum::<u64>() as u32;
        let want = format!(
            "{} {} {} {} {}",
            pairs.len(),
            fmt(&pairs[0]),
            fmt(pairs.last().unwrap()),
            sum(|p| p.0),
            sum(|p| p.1)
        );
        assert_eq!(seen.trim(), want, "walk_rules_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 3, "only {n} rows");
    eprintln!("Balance::walk_rules_data@00582cc0: {n} rows agree");
}
