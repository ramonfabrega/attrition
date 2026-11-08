//! `CityData::get_farm_limit@00738230` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::city::farm_limit_by;

/// `CityData::get_farm_limit@00738230`, run by `tools/emu/sweep_city_data_get_farm_limit.py` under unicorn
/// (tools/emu): the city's object through the per-player list, its slot `+0x3c`'s building's type index read as
/// a level (`0x19f` 2, `0x1a0` and `0x213` 3, else 1 — 1971's `get_city_level` sweep's mapping), the leader's
/// `get_farm_limit` asked on `leaders + 0x6eec · who` and `farms_per_city_level` read off the constants. The
/// arithmetic is the port's [`farm_limit_by`], what `Sim::farm_limit` answers through.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_get_farm_limit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let w = |i: usize| i32::try_from(a[i]).expect("a 32-bit word");
        let level = match w(2) {
            0x19f => 2,
            0x1a0 | 0x213 => 3,
            _ => 1,
        };
        let ecx = (a[5] as u32).wrapping_add(w(0).wrapping_mul(0x6eec) as u32);
        assert_eq!(
            tail[0],
            format!("get_build,obj;get_farm_limit,{ecx}"),
            "get_farm_limit — the calls, row `{line}`"
        );
        assert_eq!(
            i64::from(farm_limit_by(level, w(3), w(4))),
            want,
            "get_farm_limit — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CityData::get_farm_limit@00738230: {n} rows agree");
}
