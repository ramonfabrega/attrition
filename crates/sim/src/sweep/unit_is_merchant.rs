//! `UnitData::is_merchant@0046d370` under the emulator against the port (item 1691).

use crate::UnitType;

/// `UnitData::is_merchant@0046d370`, run by `tools/emu/sweep_unit_is_merchant.py` under unicorn
/// (tools/emu): `type+4` (the type index) one of `0x3d`, `0x3e`, `400`, over every index 0..=0x1ff and the 32-bit edges, against `Sim::is_merchant` on a unit whose type tree is that id. Ids outside the port's `TypeId` are left out.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_merchant.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let Ok(id) = crate::tech::TypeId::try_from(a[0]) else {
            continue;
        };
        let (mut s, u) = super::sim_with_unit(UnitType::default());
        let t = s.units[u].ty.expect("the unit's type");
        s.unit_types[t].tree = Some(id);
        assert_eq!(s.is_merchant(u), want != 0, "is_merchant — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitData::is_merchant@0046d370: {n} rows agree");
}
