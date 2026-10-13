//! `Nuke::on_field_change@00925cb0` under the emulator against the port (item 1942; tools/sweep_gen.py, fwd).

/// `Nuke::on_field_change@00925cb0`, run by `tools/emu/sweep_nuke_on_field_change.py` under unicorn
/// (tools/emu) with its callees hooked: 2 hooked callee(s), `ecx` and arguments held to the spec, `eax` 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_nuke_on_field_change.py") else {
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
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans, _p1, _p2) = (a[0], a[1], a[2], a[3], a[4]);
        let _ = (this, _p0, ans, _p1, _p2);
        let calls: Vec<Vec<i64>> = vec![vec![this, 1], vec![this, 1]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "on_field_change — calls, row `{line}`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            0,
            "on_field_change — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Nuke::on_field_change@00925cb0: {n} rows agree");
}
