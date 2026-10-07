//! `BuildTypeData::get_good@0063bd50` under the emulator against the port.

use crate::build::Ident;

/// The `TypeIndex` of each identity the port's `ai_place::gather_good` names,
/// as `docs/AI.md` §99.5 and `ai_make.rs` cite them (`0x1a1` Farm, `0x1a2`
/// Woodcutter, `0x1a3` Mine, `0x1a4` University, `0x1a5` Oil Well, `0x1a6`
/// Oil Platform), with the enhancers `[this+4] − 0x1a7` (`enhancing_good`'s
/// own table) and the three cities beside them, so the neighbours of the
/// table answer `None` through real identities rather than `Other` alone.
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

/// `BuildTypeData::get_good@0063bd50`, run by
/// `tools/emu/sweep_build_type_get_good.py` under unicorn: the `TypeIndex` at
/// `this+4`, answering the good (0 food, 1 timber, 3 knowledge, 4 metal, 5 oil)
/// for `0x1a1..=0x1a6` and −1 for every other value. The port keeps the
/// mapping as `ai_place::gather_good`, keyed by [`Ident`] rather than by type
/// index (the loader assigns identities by name), so the test maps the index
/// to its identity through the table above and asserts `gather_good`'s answer
/// (`None` ↔ −1, `Some(i)` ↔ `i`) against every row.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_get_good.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let ours = crate::ai_place::gather_good(ident_of_type_index(args[0]))
            .map_or(-1, |g| i64::try_from(g).expect("a small index"));
        assert_eq!(ours, want, "get_good(type {:#x}) — row `{line}`", args[0]);
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::get_good@0063bd50: {n} rows agree");
}
