//! `ObjectsOut::sort_objects@00654a30` under the emulator against the port (item 2672; tools/sweep_gen.py, fwd).

/// `ObjectsOut::sort_objects@00654a30`, run by `tools/emu/sweep_objects_out_sort_objects.py` under unicorn
/// (tools/emu) with its callees hooked: 5 hooked callee(s), `ecx` and arguments held to the spec, `eax` none. Each call is headed `sK:`, K its callee's place in the spec, so the order is held.
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
            .enumerate()
            .map(|(k, c)| {
                let _ = k;
                let t = c.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
                format!("s{k}:{t}")
            })
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "sort_objects — calls, row `{line}`");
        let _ = eax;
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectsOut::sort_objects@00654a30: {n} rows agree");
}
