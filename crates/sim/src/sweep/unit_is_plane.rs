//! `UnitData::is_plane@0046ce40` under the emulator against the port (item 1691).

use crate::UnitType;
use crate::ai_load::UnitCols;
use crate::attrition::Domain;

/// `UnitData::is_plane@0046ce40`, run by `tools/emu/sweep_unit_is_plane.py` under unicorn
/// (tools/emu): `type+0x218 == 2 && !(type+0x2b4 & 0x20)` — domain 2 is Air, bit `0x20` the helicopter flag — over domains −1..=4 against flag words, against `Sim::is_plane`. Domains outside the port's three (land 0, sea 1, air 2) are left out: the port's `Domain` cannot spell them.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_plane.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let flags = u32::try_from(a[1]).expect("32-bit unit flags");
        let domain = match a[0] {
            0 => Domain::Land,
            1 => Domain::Sea,
            2 => Domain::Air,
            _ => continue,
        };
        let mut t = UnitType {
            cols: UnitCols {
                unit_flags: flags,
                ..UnitCols::default()
            },
            ..UnitType::default()
        };
        t.combat.domain = domain;
        let (s, u) = super::sim_with_unit(t);
        assert_eq!(s.is_plane(u), want != 0, "is_plane — row `{line}`");
        n += 1;
    }
    assert!(n >= 40, "only {n} rows");
    eprintln!("UnitData::is_plane@0046ce40: {n} rows agree");
}
