//! `CityData::can_upgrade@007383c0` under the emulator against the port (item 2072; hand-written on
//! tools/emu/hooks.py).

use crate::Tuning;
use crate::build::Ident;
use crate::city::upgrade_kinds;

/// `CityData::can_upgrade@007383c0`, run by `tools/emu/sweep_city_data_can_upgrade.py` under unicorn
/// (tools/emu): the type to become (the three cities, their neighbours, random), `CITY_BUILDINGS` and
/// `METRO_BUILDINGS`, and the kinds count's answer (a hooked callee) — against [`upgrade_kinds`], the count
/// `Sim::ready_to_upgrade` asks: none asks nothing and answers 1, a count is handed to the callee and its answer
/// read as a truth. The row's type becomes an identity the way the loader names the cities.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_can_upgrade.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let ident = match a[0] {
            0x19e => Ident::Village,
            0x19f => Ident::Town,
            0x1a0 => Ident::Metropolis,
            0x213 => Ident::ForbiddenCity,
            _ => Ident::Other,
        };
        let tuning = Tuning {
            city_buildings: i32::try_from(a[1]).expect("a constant"),
            metro_buildings: i32::try_from(a[2]).expect("a constant"),
            ..Tuning::RON
        };
        let (ans, calls, ecx_this, handed) = (a[3], a[4], a[5], a[6]);
        assert_eq!(
            ecx_this, 1,
            "can_upgrade — the callee is asked of this, row `{line}`"
        );
        let got = match upgrade_kinds(&tuning, ident) {
            None => {
                assert_eq!(calls, 0, "can_upgrade — nothing asked, row `{line}`");
                1
            }
            Some(needed) => {
                assert_eq!(
                    (calls, handed),
                    (1, i64::from(needed)),
                    "can_upgrade — the count, row `{line}`"
                );
                i64::from(ans != 0)
            }
        };
        assert_eq!(got, want, "can_upgrade — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("CityData::can_upgrade@007383c0: {n} rows agree");
}
