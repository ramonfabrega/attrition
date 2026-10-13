//! `SubObject::time@0046ce20` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `SubObject::time@0046ce20`, run by `tools/emu/sweep_sub_object_time.py` under unicorn
/// (tools/emu): a tail call into slot 0x6c of the object at `+0x18`, one argument passed through.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sub_object_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <call>`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            a[2],
            "SubObject::time — answer, row `{line}`"
        );
        let want = format!("{},{}", a[0], a[1]);
        assert_eq!(
            seen.trim(),
            want,
            "SubObject::time — the call, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("SubObject::time@0046ce20: {n} rows agree");
}
