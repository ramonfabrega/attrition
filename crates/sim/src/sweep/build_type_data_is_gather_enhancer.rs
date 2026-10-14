//! `BuildTypeData::is_gather_enhancer@00472b50` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::build::{Ident, is_gather_enhancer_by};

/// `BuildTypeData::is_gather_enhancer@00472b50`, run by `tools/emu/sweep_build_type_data_is_gather_enhancer.py`
/// under unicorn (tools/emu): the record's slot `+0x60` asked `(0x1a7, 0)` to `(0x1aa, 0)` in turn, answered per
/// row and per type index, zero half the time. The port's [`is_gather_enhancer_by`] is handed a lineage test that
/// answers the row's word for each identity and records what it was asked, so the order of the asks and the
/// short circuit are held to the original's calls, and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_gather_enhancer.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let mut asked = Vec::new();
        let got = is_gather_enhancer_by(|i| {
            let (t, k) = match i {
                Ident::Granary => (0x1a7, 0),
                Ident::Lumbermill => (0x1a8, 1),
                Ident::Smelter => (0x1a9, 2),
                Ident::Refinery => (0x1aa, 3),
                _ => unreachable!("asked {i:?}"),
            };
            asked.push(format!("this,{t},0"));
            a[k] != 0
        });
        assert_eq!(
            tail[0],
            asked.join(";"),
            "is_gather_enhancer — the asks, row `{line}`"
        );
        assert_eq!(
            i64::from(got),
            want,
            "is_gather_enhancer — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildTypeData::is_gather_enhancer@00472b50: {n} rows agree");
}
