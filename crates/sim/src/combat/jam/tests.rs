//! Jamming (item 4541, `docs/COMBAT.md` §8.6).

use crate::combat::{self, Obj, mask, umask2};
use crate::world::Pos;
use crate::{Sim, Unit, UnitType};

fn sim() -> Sim {
    let mut s = Sim::new(
        crate::core::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        3,
    );
    for (a, b) in [(0, 1), (1, 0), (0, 2), (2, 0)] {
        s.leaders.at_war[a][b] = true;
    }
    s
}

/// An anti-air Tower of `who`'s at `at`.
fn tower(s: &mut Sim, who: crate::Player, at: Pos) -> usize {
    let b = s.add_building(who, at, 0);
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    s.buildings[b].combat = Some(combat::Profile {
        obj_masks: mask::ANTI_AIR,
        ..combat::Profile::default()
    });
    s.buildings[b].health = 336;
    b
}

/// A unit of lineage `tree`, a caster and a special (`unit_flags2 0x12`, the
/// Elite Special Forces' word), which its birth gives a specials slot
/// (`Unit::init`'s special arm, [`Sim::init_caster_slot`]).
fn special(s: &mut Sim, who: crate::Player, index: i16, at: Pos, tree: usize) -> usize {
    let mut t = UnitType {
        hits: 130,
        ..UnitType::default()
    };
    t.cols.unit_flags2 =
        crate::ai::ai_load::uflags2::CASTER | crate::ai::ai_load::uflags2::FLAG2_SCOUT;
    let ty = s.add_unit_type(t);
    let mut u = Unit::new(who, index, at, 130);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    // The lineage after the birth: the tree this fixture carries has no
    // node 0x4b for the muster's researched bit to read.
    s.tables.unit_types[ty].tree = Some(tree);
    assert!(
        s.units[u].special.is_some(),
        "the birth takes a specials slot"
    );
    u
}

/// The general radius's reach, in map units: 6 × 0x180.
const REACH: i32 = 6 * 0x180;

/// **`is_jammed`'s gates, one at a time** (`ObjectData::is_jammed@00653660`,
/// `SpecialsData::find_special@00740070`): who=2's special forces unit
/// within `GENERAL_RADIUS × 0x180` of who=1's anti-air Tower jams it while
/// who=1 carries the threat bit and who=2 the jammer bit; each gate alone
/// refuses.
#[test]
fn a_special_forces_unit_in_reach_jams_an_anti_air_tower() {
    let mut s = sim();
    let t = tower(&mut s, 1, Pos::new(12096, 34944));
    let sf = special(&mut s, 2, 0, Pos::new(12096 + REACH, 34944), 0x4b);
    s.leaders.jam_threat[1] = true;
    s.leaders.special_forces[2] = true;
    assert_eq!(
        s.is_jammed(Obj::Building(t)),
        Some(sf),
        "at the reach's edge"
    );

    s.units[sf].pos.x += 1;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "one past it");
    s.units[sf].pos.x -= 1;

    s.leaders.jam_threat[1] = false;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "no threat bit");
    s.leaders.jam_threat[1] = true;

    s.leaders.special_forces[2] = false;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "no jammer bit");
    s.leaders.special_forces[2] = true;

    s.units[sf].on_map = false;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "off the map");
    s.units[sf].on_map = true;

    s.leaders.allied[1][2] = true;
    s.leaders.allied[2][1] = true;
    s.leaders.at_war[1][2] = false;
    s.leaders.at_war[2][1] = false;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "allied both ways");
    s.leaders.allied[1][2] = false;
    s.leaders.allied[2][1] = false;

    s.buildings[t].combat.as_mut().unwrap().obj_masks = 0;
    assert_eq!(s.is_jammed(Obj::Building(t)), None, "not ANTI_AIR");
    s.buildings[t].combat.as_mut().unwrap().obj_masks = mask::ANTI_AIR;

    let scout = special(&mut s, 2, 1, Pos::new(12096, 34944), 0x45);
    s.roster.specials[2].in_use[0] = false;
    assert_eq!(
        s.is_jammed(Obj::Building(t)),
        None,
        "a special of another lineage does not jam, and a free slot is skipped"
    );
    assert_eq!(s.units[scout].special, Some(1));
}

/// **`Build::do_attack`'s jam arm marks both and holds the shot**, and a
/// call with no jammer clears the building's bit and not the unit's.
#[test]
fn a_jammed_tower_marks_the_jammer_and_holds_its_shot() {
    let mut s = sim();
    let t = tower(&mut s, 1, Pos::new(12096, 34944));
    let sf = special(&mut s, 2, 0, Pos::new(12672, 34944), 0x4b);
    s.leaders.jam_threat[1] = true;
    s.leaders.special_forces[2] = true;
    assert!(s.building_jam(t));
    assert!(s.buildings[t].jammed, "build_masks & 0x8000");
    assert_eq!(s.units[sf].unit_masks2 & umask2::JAMMER, umask2::JAMMER);

    s.units[sf].pos.x += 3 * REACH;
    assert!(!s.building_jam(t));
    assert!(!s.buildings[t].jammed, "cleared on a call that finds none");
    assert_eq!(
        s.units[sf].unit_masks2 & umask2::JAMMER,
        umask2::JAMMER,
        "the unit's mark is the caster arm's to drop"
    );
}

/// **`Unit::process`'s caster arm**: on the unit's 32-frame phase, the
/// jammer's 8 becomes `unit_masks & 0x80000000`; the next phase without
/// an 8 drops it; off the phase, and for a unit of another lineage,
/// nothing moves.
#[test]
fn the_caster_arm_trades_the_jammer_mark_for_the_shown_bit_on_its_phase() {
    let mut s = sim();
    let sf = special(&mut s, 2, 0, Pos::new(12672, 34944), 0x4b);
    s.units[sf].unit_masks2 |= umask2::JAMMER;
    s.show_jam(sf, 5599);
    assert!(!s.units[sf].jam_shown, "off its phase");
    assert_ne!(s.units[sf].unit_masks2 & umask2::JAMMER, 0);
    s.show_jam(sf, 5600);
    assert!(s.units[sf].jam_shown, "(5600 + 0) & 0x1f == 0");
    assert_eq!(s.units[sf].unit_masks2 & umask2::JAMMER, 0);
    s.show_jam(sf, 5632);
    assert!(!s.units[sf].jam_shown, "no mark on the next phase");

    let other = special(&mut s, 2, 1, Pos::new(12672, 34944), 0x45);
    s.units[other].unit_masks2 |= umask2::JAMMER;
    s.show_jam(other, 5631);
    assert_ne!(
        s.units[other].unit_masks2 & umask2::JAMMER,
        0,
        "not of the lineage"
    );
    assert!(!s.units[other].jam_shown);
}
