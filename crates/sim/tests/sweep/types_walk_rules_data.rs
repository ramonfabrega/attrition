//! `Types::walk_rules_data@00669800` under the emulator against the port (item 1921; hand-written, a loop of virtual calls).

/// The loop's trip count: `0xc98 / 4` entries of the `types.list` pointer array.
const ENTRIES: usize = 0xc98 / 4;

/// `Types::walk_rules_data@00669800`, run by `tools/emu/sweep_types_walk_rules_data.py` under
/// unicorn (tools/emu) with the vtable's slot `0xc4` hooked: each of the 806 entries is entered
/// once, in array order, `ecx` the entry, handed the walker alone.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_types_walk_rules_data.py") else {
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
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let first = a[0];
        let want: Vec<i64> = (0..ENTRIES as i64).map(|i| first + 16 * i).collect();
        let got: Vec<i64> = seen
            .trim()
            .split(',')
            .map(|s| s.parse().expect("an integer"))
            .collect();
        assert_eq!(got, want, "walk_rules_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 10, "only {n} rows");
    eprintln!("Types::walk_rules_data@00669800: {n} rows agree");
}
