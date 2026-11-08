//! `ObjectData::get_wonder_value@0046cf20` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `ObjectData::get_wonder_value@0046cf20`, run by `tools/emu/sweep_object_data_get_wonder_value.py` under unicorn
/// (tools/emu): a tail call into slot 0x118 of the object at `+0x18`, no argument.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_get_wonder_value.py") else {
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
            "ObjectData::get_wonder_value — answer, row `{line}`"
        );
        let want = format!("{},{}", a[0], 0);
        assert_eq!(
            seen.trim(),
            want,
            "ObjectData::get_wonder_value — the call, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("ObjectData::get_wonder_value@0046cf20: {n} rows agree");
}
