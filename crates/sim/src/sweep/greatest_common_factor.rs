//! `greatest_common_factor@0092cee0` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `greatest_common_factor@0092cee0`, run by `tools/emu/sweep_greatest_common_factor.py` under unicorn
/// (tools/emu): the largest `d >= 2` dividing both (signed `min` start, counting down), else 1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_greatest_common_factor.py") else {
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
        let (x, y) = (a[0], a[1]);
        let mut d = x.min(y);
        let got = loop {
            if d < 2 {
                break 1;
            }
            if x % d == 0 && y % d == 0 {
                break d;
            }
            d -= 1;
        };
        assert_eq!(
            got,
            rhs.parse::<i64>().unwrap(),
            "greatest_common_factor — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("greatest_common_factor@0092cee0: {n} rows agree");
}
