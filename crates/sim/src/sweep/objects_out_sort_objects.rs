//! `ObjectsOut::sort_objects@00654a30` under the emulator against the port (item 2644; tools/sweep_gen.py, fwd).

/// `ObjectsOut::sort_objects@00654a30`, run by `tools/emu/sweep_objects_out_sort_objects.py` under unicorn
/// (tools/emu) with its callees hooked: 5 hooked callee(s), `ecx` and arguments held to the spec, `eax` none.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_out_sort_objects.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans, _p1, _p2) = (a[0], a[1], a[2], a[3], a[4]);
        let _ = (this, _p0, ans, _p1, _p2);
        let calls: Vec<Vec<i64>> = vec![vec![this], vec![this], vec![this], vec![this], vec![this]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "sort_objects — calls, row `{line}`");
        let _ = eax;
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectsOut::sort_objects@00654a30: {n} rows agree");
}
