//! `Borders::get_who@00883190` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `Borders::get_who@00883190`, run by `tools/emu/sweep_borders_get_who.py` under unicorn
/// (tools/emu): the signed byte at `+0xf` of cell `y*width + x` when non-negative, else -1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_get_who.py") else {
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
        let b = a[3] as u8 as i8;
        let got = if b >= 0 { i64::from(b) } else { -1 };
        assert_eq!(got, rhs.parse::<i64>().unwrap(), "get_who — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Borders::get_who@00883190: {n} rows agree");
}
