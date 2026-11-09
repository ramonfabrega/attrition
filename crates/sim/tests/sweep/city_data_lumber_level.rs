//! `CityData::lumber_level@00736820` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::holdings::lumber_level_by;

/// `CityData::lumber_level@00736820`, run by `tools/emu/sweep_city_data_lumber_level.py` under unicorn
/// (tools/emu): bit 0x400 of the flags word gates `has_preq` on `leaders + 0x6eec · who` for `0x2c4`, `0x2c3`
/// and `0x2c2` from the top, answered per argument. The port's [`lumber_level_by`] is handed a `has_preq` that
/// answers the row's word and records the ask, so the gate, the order, the short circuit and the `ecx` are held
/// to the original's calls, and the level to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_lumber_level.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (flags, who) = (i32::try_from(a[0]).expect("a word"), a[1] as i32);
        let ecx = (a[2] as u32).wrapping_add(who.wrapping_mul(0x6eec) as u32);
        let mut calls = Vec::new();
        let got = lumber_level_by(flags, |i| {
            calls.push(format!("{ecx},{}", 0x2c2 + i));
            a[3 + i] != 0
        });
        assert_eq!(
            tail[0],
            calls.join(";"),
            "lumber_level — the asks, row `{line}`"
        );
        assert_eq!(i64::from(got), want, "lumber_level — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CityData::lumber_level@00736820: {n} rows agree");
}
