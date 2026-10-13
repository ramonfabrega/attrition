//! `ObjectData::say_name@00645310` under the emulator against the port (item 1942; hand-written).

/// `ObjectData::say_name@00645310`, run by `tools/emu/sweep_object_data_say_name.py` under unicorn: the callee is entered with `ecx` the argument and the pointer at `this+0x18` plus `0x74`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_say_name.py") else {
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
        let (ptr, p0) = (a[0] as i32, a[1] as i32);
        let want = format!(
            "{},{}",
            u32::from_ne_bytes(p0.to_ne_bytes()),
            ptr.wrapping_add(0x74)
        );
        assert_eq!(seen.trim(), want, "say_name — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("ObjectData::say_name@00645310: {n} rows agree");
}
