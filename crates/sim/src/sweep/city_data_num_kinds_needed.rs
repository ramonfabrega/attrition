//! `CityData::num_kinds_needed@00736640` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::Tuning;
use crate::city::kinds_needed_by;

/// `CityData::num_kinds_needed@00736640`, run by `tools/emu/sweep_city_data_num_kinds_needed.py` under unicorn
/// (tools/emu): the city object's type through `objects[who][o]`, the top levels answered 0 unasked, the next
/// level asked of `type_avail(t, 1)` on `leaders + 0x6eec · who`, and the count `city_buildings + 1` or
/// `metro_buildings + 1` off the row's constants. The port's [`kinds_needed_by`] is handed the row's answer and
/// records the ask, so the gate, the next level and the `ecx` are held to the original's calls, and the count to
/// `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_num_kinds_needed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (ty, who) = (a[0] as i32, a[1] as i32);
        let ecx = (a[3] as u32).wrapping_add(who.wrapping_mul(0x6eec) as u32);
        let t = Tuning {
            city_buildings: a[5] as i32,
            metro_buildings: a[6] as i32,
            ..Tuning::RON
        };
        let mut calls = Vec::new();
        let got = kinds_needed_by(&t, ty, |next| {
            calls.push(format!("avail,{ecx},{next},1"));
            a[4] != 0
        });
        assert_eq!(
            tail[0],
            calls.join(";"),
            "num_kinds_needed — the asks, row `{line}`"
        );
        assert_eq!(i64::from(got), want, "num_kinds_needed — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CityData::num_kinds_needed@00736640: {n} rows agree");
}
