//! `CityData::pop_cap@00737d40` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

use crate::Tuning;
use crate::cost::CityLevel;

/// `CityData::pop_cap@00737d40`, run by `tools/emu/sweep_city_data_pop_cap.py` under unicorn (tools/emu): the
/// city's own object is asked once for its data record, and its type id picks the multiple of `VILLAGE_POP` —
/// `0x19f` (`TOWN`) twice, `0x1a0` and `0x213` (`FORBIDDENCITY`) three times, any other once. The port is
/// [`CityLevel::pop_cap`] on a tuning whose `village_pop` is the row's.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_pop_cap.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            (a[2], a[3]),
            (1, 1),
            "the object asked once, as itself — row `{line}`"
        );
        let level = match a[0] {
            0x19f => CityLevel::Town,
            0x1a0 | 0x213 => CityLevel::Metropolis,
            _ => CityLevel::Village,
        };
        let t = Tuning {
            village_pop: a[1] as i32,
            ..Tuning::RON
        };
        assert_eq!(i64::from(level.pop_cap(&t)), want, "pop_cap — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CityData::pop_cap@00737d40: {n} rows agree");
}
