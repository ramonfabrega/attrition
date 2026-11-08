//! `CityData::get_level@00739340` under the emulator against the port (item 2072; hand-written on tools/emu/hooks.py).

use crate::build::{BuildType, Ident, city_level};

/// `CityData::get_level@00739340`, run by `tools/emu/sweep_city_data_get_level.py` under unicorn (tools/emu): the
/// city's own object (through the owner's table) is asked once for its data record, and its type id answers the
/// level — `0x19f` (`TOWN`) 2, `0x1a0` and `0x213` (`FORBIDDENCITY`) 3, any other 1. The port is
/// `Sim::city_level_of`'s arm, [`city_level`] floored at 1, on a list of the four city types and one other: the
/// row's id becomes an identity the way the loader names them.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_get_level.py") else {
        return;
    };
    let types: Vec<BuildType> = [
        Ident::Village,
        Ident::Town,
        Ident::Metropolis,
        Ident::ForbiddenCity,
        Ident::Other,
    ]
    .into_iter()
    .map(|ident| BuildType {
        ident,
        ..BuildType::default()
    })
    .collect();
    let index = [0x19e, 0x19f, 0x1a0, 0x213];
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            (a[1], a[2]),
            (1, 1),
            "get_level — one call of the object's slot, row `{line}`"
        );
        let t = index.iter().position(|&i| i == a[0]).unwrap_or(4);
        let got = i64::from(city_level(&types, t).max(1));
        assert_eq!(got, want, "get_level — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("CityData::get_level@00739340: {n} rows agree");
}
