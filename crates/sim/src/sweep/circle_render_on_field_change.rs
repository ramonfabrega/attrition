//! `CircleRender::on_field_change@008c6db0` under the emulator against the port (item 1895; tools/sweep_gen.py, fwd).

/// `CircleRender::on_field_change@008c6db0`, run by `tools/emu/sweep_circle_render_on_field_change.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_circle_render_on_field_change.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans) = (a[0], a[1], a[2]);
        let _ = (this, _p0, ans);
        let calls: Vec<Vec<i64>> = vec![vec![this]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "on_field_change — calls, row `{line}`");
        assert_eq!(eax.trim().parse::<i64>().unwrap(), 0, "on_field_change — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("CircleRender::on_field_change@008c6db0: {n} rows agree");
}
