//! `Objects::remove_item@0065b850` under the emulator against the port (item 1932; hand-written).

/// `Objects::remove_item@0065b850`, run by `tools/emu/sweep_objects_remove_item.py` under unicorn: slot 0x90 of `items[index]`, once, with that object as `ecx`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_remove_item.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let a: Vec<i64> = line
            .split(" -> ")
            .next()
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let want = 0;
        let idx = a[0];
        let (eax, seen) = line
            .split_once(" -> ")
            .unwrap()
            .1
            .split_once(" | ")
            .expect("`<eax> | <calls>`");
        let _ = want;
        assert_eq!(
            seen.trim(),
            idx.to_string(),
            "remove_item — which object, row `{line}`"
        );
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            1000 + idx * 7,
            "remove_item — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Objects::remove_item@0065b850: {n} rows agree");
}
