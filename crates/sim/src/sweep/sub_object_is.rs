//! `SubObject::is@00476e50` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `SubObject::is@00476e50`, run by `tools/emu/sweep_sub_object_is.py` under unicorn
/// (tools/emu): a tail call into slot 0x60 of the object at `+0x18`, two arguments passed through.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sub_object_is.py") else {
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
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <call>`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            a[3],
            "SubObject::is — answer, row `{line}`"
        );
        let want = format!("{},{},{}", a[0], a[1], a[2]);
        assert_eq!(seen.trim(), want, "SubObject::is — the call, row `{line}`");
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("SubObject::is@00476e50: {n} rows agree");
}
