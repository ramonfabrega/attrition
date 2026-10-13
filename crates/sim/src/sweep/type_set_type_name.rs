//! `Type::set_type_name@00663b70` under the emulator against the port (item 1942; hand-written).

/// `Type::set_type_name@00663b70`, run by `tools/emu/sweep_type_set_type_name.py` under unicorn: the callee is entered on `this+0xb0` with the argument, or `this+0x60` when its length word is zero.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_set_type_name.py") else {
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
        let (res, seen) = rhs.split_once("| ").unwrap_or((rhs, ""));
        let res: Vec<i64> = res
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let _ = (&a, &res, seen);
        let (ln, rel) = (a[0], a[1]);
        let arg = if ln & 0xffff == 0 { 0x60 } else { rel };
        assert_eq!(seen.trim(), format!("{},{}", 0xb0, arg), "set_type_name — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("Type::set_type_name@00663b70: {n} rows agree");
}
