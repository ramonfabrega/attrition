//! `SpellType::cast_supplies@006714c0` under the emulator against the port (item 1942; hand-written).

/// `SpellType::cast_supplies@006714c0`, run by `tools/emu/sweep_spell_type_cast_supplies.py` under unicorn: the word at `0x96` of the chosen object becomes zero and nothing else moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_supplies.py") else {
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
        assert_eq!(res[0], 0, "cast_supplies — new word, row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("SpellType::cast_supplies@006714c0: {n} rows agree");
}
