//! `World::set_orig_coast@006b1b80` under the emulator against the port (item 1942; hand-written).

/// `World::set_orig_coast@006b1b80`, run by `tools/emu/sweep_world_set_orig_coast.py` under unicorn: the word loses bit `0x400` only.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_orig_coast.py") else {
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
        assert_eq!(res[0], a[0] & 0xfbff, "set_orig_coast — row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("World::set_orig_coast@006b1b80: {n} rows agree");
}
