//! `Balance::return_modifier@00581ca0` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `Balance::return_modifier@00581ca0`, run by `tools/emu/sweep_balance_return_modifier.py` under unicorn
/// (tools/emu): a sign-extended 16-bit word at index `a*0x1ed + b`; the script fills the table with a hash of the index.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_return_modifier.py") else {
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
        let i = (a[0] * 0x1ed + a[1]) as u64;
        let w = ((i.wrapping_mul(2_654_435_761) >> 7) & 0xffff) as u16;
        assert_eq!(
            i64::from(w as i16),
            rhs.parse::<i64>().unwrap(),
            "return_modifier — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Balance::return_modifier@00581ca0: {n} rows agree");
}
