//! `CityData::granary_level@00736890` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `CityData::granary_level@00736890`, run by `tools/emu/sweep_city_data_granary_level.py` under unicorn
/// (tools/emu): bit 0x200 of the flags word gates a call into `LeaderData::get_granary` on `leaders + who * 0x6eec`; the answer passes through.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_granary_level.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (flags, who, base, ans) = (a[0], a[1], a[2] as u32, a[3]);
        let (want_eax, want_seen) = if flags & 0x200 == 0 {
            (0, String::new())
        } else {
            let ecx = base.wrapping_add((who as i32).wrapping_mul(0x6eec) as u32);
            (ans, ecx.to_string())
        };
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            want_eax,
            "granary_level — answer, row `{line}`"
        );
        assert_eq!(
            seen.trim(),
            want_seen,
            "granary_level — the call, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("CityData::granary_level@00736890: {n} rows agree");
}
