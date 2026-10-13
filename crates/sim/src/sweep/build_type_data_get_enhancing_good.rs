//! `BuildTypeData::get_enhancing_good@00639880` under the emulator against the port (item 2083;
//! `build_type_get_good.rs`'s shape).

use crate::build::Ident;

/// The `TypeIndex` of the four enhancers (`[this+4] − 0x1a7`, the listing's own table at `0x6398b0`) and of the
/// gatherers and cities beside them, so the table's neighbours answer `None` through real identities rather than
/// `Other` alone.
fn ident_of_type_index(t: i64) -> Ident {
    match t {
        0x19e => Ident::Village,
        0x19f => Ident::Town,
        0x1a0 => Ident::Metropolis,
        0x1a1 => Ident::Farm,
        0x1a2 => Ident::Woodcutter,
        0x1a3 => Ident::Mine,
        0x1a4 => Ident::University,
        0x1a5 => Ident::OilWell,
        0x1a6 => Ident::OilPlatform,
        0x1a7 => Ident::Granary,
        0x1a8 => Ident::Lumbermill,
        0x1a9 => Ident::Smelter,
        0x1aa => Ident::Refinery,
        _ => Ident::Other,
    }
}

/// `BuildTypeData::get_enhancing_good@00639880`, run by
/// `tools/emu/sweep_build_type_data_get_enhancing_good.py` under unicorn: the `TypeIndex` at `this+4` over every
/// value `0..0x300`, the edges and random words, the rest of the record random and held unwritten; the good
/// (0 food, 1 timber, 4 metal, 5 oil) for `0x1a7..=0x1aa` and −1 otherwise, against
/// `ai_place::enhancing_good` keyed by [`Ident`] (`None` ↔ −1).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_enhancing_good.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let ours = crate::ai_place::enhancing_good(ident_of_type_index(args[0]))
            .map_or(-1, |g| i64::try_from(g).expect("a small index"));
        assert_eq!(
            ours, want,
            "get_enhancing_good(type {:#x}) — row `{line}`",
            args[0]
        );
        n += 1;
    }
    assert!(n >= 900, "only {n} rows");
    eprintln!("BuildTypeData::get_enhancing_good@00639880: {n} rows agree");
}
