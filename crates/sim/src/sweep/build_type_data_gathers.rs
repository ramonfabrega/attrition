//! `BuildTypeData::gathers@0063bda0` under the emulator against the port (item 1851).

/// `BuildTypeData::gathers@0063bda0`, run by `tools/emu/sweep_build_type_data_gathers.py` under unicorn
/// (tools/emu): `__thiscall`, one stack int `g` (`ret 4`): 1 when it equals `get_good@0063bd50` (the `TypeIndex` at +4: farm 0, woodcutter 1, mine 3, university 4, oil well 5, ... else -1); the real `get_good` is called. Row `<type> <g>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_gathers.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        use crate::build::Ident;
        let ident = match a[0] {
            0x1a1 => Ident::Farm,
            0x1a2 => Ident::Woodcutter,
            0x1a3 => Ident::Mine,
            0x1a4 => Ident::University,
            0x1a5 => Ident::OilWell,
            0x1a6 => Ident::OilPlatform,
            _ => Ident::Other,
        };
        let good = crate::ai_place::gather_good(ident)
            .map_or(-1, |g| i64::try_from(g).expect("a small index"));
        let got = i64::from(a[1] == good);
        assert_eq!(got, want, "BuildTypeData::gathers — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("BuildTypeData::gathers@0063bda0: {n} rows agree");
}
