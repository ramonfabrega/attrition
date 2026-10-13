//! `CityData::smelter_level@007367a0` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::holdings::smelter_level_by;

/// `CityData::smelter_level@007367a0`, run by `tools/emu/sweep_city_data_smelter_level.py` under unicorn
/// (tools/emu): the fifth good type's kind through the jump table, the enhancer type asked of `count_buildings`
/// on the city, and `get_smelter` on `leaders + 0x6eec · who` when one stands. The port's [`smelter_level_by`]
/// is handed the row's answers and records the asks, so the table, the gate, the order and the `ecx` are held to
/// the original's calls, and the level to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_smelter_level.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (kind, who) = (a[0] as i32, a[1] as i32);
        let ecx = (a[2] as u32).wrapping_add(who.wrapping_mul(0x6eec) as u32);
        let (count, smelter) = (a[3] as i32, a[4] as i32);
        let mut calls = Vec::new();
        let got = {
            let calls = std::cell::RefCell::new(&mut calls);
            smelter_level_by(
                kind,
                |t| {
                    calls.borrow_mut().push(format!("count,this,{t},0,1"));
                    count != 0
                },
                || {
                    calls.borrow_mut().push(format!("smelter,{ecx}"));
                    smelter
                },
            )
        };
        assert_eq!(tail[0], calls.join(";"), "smelter_level — the asks, row `{line}`");
        assert_eq!(i64::from(got), want, "smelter_level — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CityData::smelter_level@007367a0: {n} rows agree");
}
