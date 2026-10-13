//! `WallData::is_defensive@00473650` under the emulator against the port (item 2083; hand-written on tools/emu/hooks.py).

use crate::build::{Ident, is_defensive_by};

/// `WallData::is_defensive@00473650`, run by `tools/emu/sweep_wall_data_is_defensive.py` under unicorn (tools/emu): the same body on the record at the wall's `+0x18`, each ask's `ecx` the record; the `is` slot
/// answered per row and per type index, zero half the time. The port's [`is_defensive_by`] is handed a lineage
/// test that answers the row's word for each identity and records what it was asked, so the order of the asks
/// and the short circuit are held to the original's calls, and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_is_defensive.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let mut asked = Vec::new();
        let got = is_defensive_by(
            |i| {
                let (t, k) = match i {
                    Ident::Tower => (0x1b7, 0),
                    Ident::Fort => (0x1bb, 1),
                    Ident::Airbase => (0x1bf, 2),
                    _ => unreachable!("asked {i:?}"),
                };
                asked.push(format!("this,{t},0"));
                a[k] != 0
            },
            i32::try_from(a[3]).expect("a 32-bit attack"),
        );
        assert_eq!(tail[0], asked.join(";"), "is_defensive — the asks, row `{line}`");
        assert_eq!(i64::from(got), want, "is_defensive — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WallData::is_defensive@00473650: {n} rows agree");
}
