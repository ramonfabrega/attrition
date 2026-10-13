//! `CityData::upgrades_to@007364f0` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

use crate::build::{BuildType, Ident, upgrades_to};

/// `CityData::upgrades_to@007364f0`, run by `tools/emu/sweep_city_data_upgrades_to.py` under unicorn
/// (tools/emu): the building's `TypeIndex` (through `objects.list[who][o]`'s type record) mapped to the next city
/// level — against [`upgrades_to`], the port keyed by [`Ident`]: the row's index becomes an identity, the port's
/// answer a position in a list of the four city types, and the position the index the original answers.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_upgrades_to.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (ty, want_ix) = (a[2], want);
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
        let t = index.iter().position(|&i| i == ty).unwrap_or(4);
        let got = upgrades_to(&types, t).map_or(-1, |p| index[p]);
        assert_eq!(got, want_ix, "upgrades_to — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CityData::upgrades_to@007364f0: {n} rows agree");
}
