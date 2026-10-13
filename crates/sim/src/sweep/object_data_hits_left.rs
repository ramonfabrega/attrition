//! `ObjectData::hits_left@006535c0` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `ObjectData::hits_left@006535c0`, run by `tools/emu/sweep_object_data_hits_left.py` under unicorn
/// (tools/emu): two calls of slot `0x11c(0)`; `min(second - damage, first)` unless either is negative, then 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_hits_left.py") else {
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
        let (first, second, damage) = (a[1] as i32, a[2] as i32, a[3] as i32);
        let left = second.wrapping_sub(damage);
        let got = if left >= 0 && first >= 0 {
            left.min(first)
        } else {
            0
        };
        assert_eq!(
            i64::from(got),
            rhs.parse::<i64>().unwrap(),
            "hits_left — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("ObjectData::hits_left@006535c0: {n} rows agree");
}
