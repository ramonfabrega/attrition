//! `UnitData::mana_left@00609a30` against the port.

use crate::tech::{TypeDef, UnitTraits};
use crate::world::{Cell, Terrain, World};
use crate::{Pos, Sim, Tuning, Unit, UnitType};

/// `UnitData::mana_left@00609a30`, run under unicorn by
/// `tools/emu/sweep_unit_mana_left.py` (with `mana@00609a50` and the real
/// `has_tribe_bonus@006e1370` behind it, on laid-out singletons), against
/// `Sim::mana_left`: every row's `<mana> <burn> <owner> <french> <general>
/// <fsc> <nonation> <town> <city>` is set on one unit of this crate's and
/// the answers compared. The supply and space-air arms of `mana` are not
/// reached (the port's SEAMs), and the type's `is(GENERAL, 1)` is a stub
/// word in the script, a tree row in the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_mana_left.py") else {
        return;
    };
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut s = Sim::new(Tuning::RON, world, 2);
    let mut t = crate::tech::TechTree::new();
    while t.types.len() <= crate::nations::ty::GENERAL {
        t.add(TypeDef::unit("filler", UnitTraits::default()));
    }
    s.set_tech_tree(t);
    // [general?][owner] -> the unit.
    let units = [0usize, 1].map(|general| {
        let row = if general == 1 {
            crate::nations::ty::GENERAL
        } else {
            0
        };
        let ty = s.add_unit_type(UnitType {
            tree: Some(row),
            type_index: i32::try_from(row).unwrap(),
            ..UnitType::default()
        });
        [0i16, 1].map(|owner| {
            let mut u = Unit::new(owner as u8, 0, Pos::new(3 * 768, 3 * 768), 20);
            u.ty = Some(ty);
            (s.add_unit(u), ty)
        })
    });
    let mut n = 0;
    for line in rows.lines() {
        let (a, original) = super::row(line);
        let [
            mana,
            burn,
            owner,
            french,
            general,
            fsc,
            nonation,
            town,
            city,
        ] = a[..]
        else {
            panic!("nine arguments in `{line}`");
        };
        let (u, ty) = units[general as usize][owner as usize];
        s.unit_types[ty].mana = i32::try_from(mana).unwrap();
        s.units[u].mana_burn = i16::try_from(burn).unwrap();
        s.tuning.french_special_craft = i32::try_from(fsc).unwrap();
        s.setup.no_nation_powers = nonation != 0;
        s.setup.starting_town = town != 0;
        s.tech[owner as usize].has_city = city != 0;
        s.tech[owner as usize].power = (french != 0).then_some(crate::nations::power::FRENCH);
        s.tech[1 - owner as usize].power = None;
        assert_eq!(
            i64::from(s.mana_left(u)),
            original,
            "mana_left on row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
}
