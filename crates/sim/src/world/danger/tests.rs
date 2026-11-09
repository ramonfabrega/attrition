use crate::build::{BuildType, Ident, flags};
use crate::world::{Cell, Pos, Terrain, UNITS_PER_TILE, World};
use crate::{Sim, Tuning};

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(
        tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

/// A 16 × 16 land world with two players at war, a city record and a
/// barracks record. Nothing here goes near the tech tree: the danger
/// map reads the `BuildType` row and the object, and nothing else.
fn fx() -> (Sim, usize, usize) {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let city = sim.add_build_type(BuildType {
        ident: Ident::Village,
        x_size: 7,
        y_size: 7,
        hits: 1200,
        ..BuildType::default()
    });
    let barracks = sim.add_build_type(BuildType {
        ident: Ident::Other,
        x_size: 3,
        y_size: 3,
        hits: 400,
        flags: flags::MILITARY_TRAINER,
        ..BuildType::default()
    });
    (sim, city, barracks)
}

/// A finished building of a type that needs no city and belongs to
/// none is skipped; one that belongs to a city is counted. The filter
/// is `WallData::is_active` **and** (`city >= 0` or `NO_CITY`).
#[test]
fn the_filter_wants_a_finished_building_with_a_city_or_a_type_that_needs_none() {
    let (mut sim, _city, barracks) = fx();
    let b = sim.init_build(0, barracks, tile_pos(8, 8), false);
    assert!(!sim.danger_counts_building(b), "unfinished");
    sim.buildings[b].active = true;
    assert!(!sim.danger_counts_building(b), "finished, but no city");
    sim.tables.build_types[barracks].flags |= flags::NO_CITY;
    assert!(sim.danger_counts_building(b), "NO_CITY needs none");
    sim.tables.build_types[barracks].flags &= !flags::NO_CITY;
    sim.buildings[b].city = Some(0);
    assert!(sim.danger_counts_building(b), "a member of a city");
}

/// **An upgraded building weighs its basic type's fifty**
/// (`calc_danger@00732d10:158`, `basic_type()`): the Auto Plant, a
/// Factory's upgrade that no unit names as its `WHERE`, is a trainer in
/// the danger map though its own row carries no trainer bit — Great
/// Sahara's `1/2025` over (25, 12), priced ten here and fifty there
/// (item 1584). Made to fail with the row's own flags read.
#[test]
fn an_upgrade_weighs_the_trainer_bit_of_its_basic_type() {
    let (mut sim, _city, barracks) = fx();
    let upgrade = sim.add_build_type(BuildType {
        ident: Ident::Other,
        x_size: 3,
        y_size: 3,
        hits: 400,
        from: Some(barracks),
        ..BuildType::default()
    });
    let b = sim.init_build(0, upgrade, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
    sim.buildings[b].active = true;
    assert_eq!(sim.danger_building_value(b), 50);
    sim.tables.build_types[barracks].flags &= !flags::MILITARY_TRAINER;
    assert_eq!(sim.danger_building_value(b), 10);
}

/// **Your own building makes the ground under it negative**, at its
/// own weight in its own half-cell and half that in the eight around
/// it. This is the whole reason the map is not a decoration: it is what
/// `calc_cost` reads as a discount.
#[test]
fn an_owner_s_own_building_is_a_negative_block_of_nine() {
    let (mut sim, _city, barracks) = fx();
    sim.tables.build_types[barracks].flags |= flags::NO_CITY;
    // Cell (8, 8) is half-cell (4, 4).
    let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
    sim.buildings[b].active = true;
    assert_eq!(sim.danger_building_value(b), 50, "a military trainer");
    sim.calc_danger();
    assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), -50, "the centre");
    assert_eq!(
        sim.world.danger_half(0, Cell::new(6, 8)),
        -25,
        "one half-cell west"
    );
    assert_eq!(
        sim.world.danger_half(0, Cell::new(6, 6)),
        -25,
        "and the diagonal — the spread is a 3 x 3 block"
    );
    assert_eq!(
        sim.world.danger_half(0, Cell::new(4, 8)),
        0,
        "two half-cells out is outside the block"
    );
}

/// The same building, seen by the enemy it is at war with: **positive**,
/// and halved again because the enemy has never seen it
/// (`do_danger`'s `is_seen` arm). A neutral third party at peace would
/// halve it once more.
#[test]
fn an_enemy_s_building_adds_and_the_unseen_halving_is_the_only_one_at_war() {
    let (mut sim, _city, barracks) = fx();
    sim.tables.build_types[barracks].flags |= flags::NO_CITY;
    // A fog grid nobody has lit: without one `was_seen_fog` answers
    // "everything is seen", which is the fixture's default and not
    // what a capture looks like.
    assert!(sim.world.set_fog(vec![0; 32 * 32]), "a 16 x 16 world's fog");
    let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
    sim.buildings[b].active = true;
    sim.buildings[b].started = true;
    sim.calc_danger();
    assert_eq!(
        sim.world.danger_half(1, Cell::new(8, 8)),
        25,
        "50, halved once because leader 1 has never seen it"
    );
    // Leader 1's bit in the building's own `ever_seen`, and the halving
    // goes (`WallData::is_seen@00642bd0`).
    sim.buildings[b].ever_seen |= 1 << 1;
    sim.calc_danger();
    assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), 50, "seen");
    sim.buildings[b].ever_seen = 0;
    // At peace it is halved twice; allied, it is *subtracted* at half.
    sim.at_war[0][1] = false;
    sim.at_war[1][0] = false;
    sim.calc_danger();
    assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), 12, "50/2/2");
    sim.allied[0][1] = true;
    sim.allied[1][0] = true;
    sim.calc_danger();
    assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), -25, "an ally's");
}

/// **A building is seen by its own `ever_seen`, not by the fog** (item
/// 3367, `BuildData::is_seen@0062e1a0` → `WallData::is_seen@00642bd0`):
/// a fog plane lit everywhere — what `was_seen` answers for a Spanish
/// leader, whose `leader_flags & 0x1000` calls every cell seen — leaves
/// a building the viewer has never seen halved. run993b's block 8959:
/// the human's city half-cell (4, 4) is 60 in the original's row for
/// the Spanish, and was 125 here.
#[test]
fn a_lit_fog_does_not_see_a_building_its_ever_seen_byte_has_not() {
    let (mut sim, _city, barracks) = fx();
    sim.tables.build_types[barracks].flags |= flags::NO_CITY;
    assert!(
        sim.world.set_fog(vec![0xff; 32 * 32]),
        "a 16 x 16 world's fog"
    );
    let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
    sim.buildings[b].active = true;
    sim.buildings[b].started = true;
    sim.buildings[b].ever_seen = 0;
    sim.calc_danger();
    assert_eq!(
        sim.world.danger_half(1, Cell::new(8, 8)),
        25,
        "50, halved: the fog is lit and the building is unseen"
    );
}

/// **A unit is seen by the *live* fog, not the ever-seen bit** (item
/// 2703, `UnitData::is_seen@00607a60`, run875's Biplane over
/// (48, 24): 195 where the original's is 97). A building takes
/// `ever_seen`; a unit that has left ground the viewer once lit is
/// unseen, and its `visible` byte (set by attacking the viewer) is the
/// only other way in.
#[test]
fn a_unit_is_seen_by_the_live_fog_and_not_the_ever_seen_bit() {
    use crate::UnitType;
    let (mut sim, _city, _barracks) = fx();
    let ty = sim.add_unit_type(UnitType::default());
    let u = sim.init_unit(0, ty, tile_pos(8 * 4 + 1, 8 * 4 + 1));
    assert!(sim.world.set_fog(vec![0; 32 * 32]), "a 16 x 16 world's fog");
    assert!(!sim.danger_unit_is_seen(u, 1), "never lit");
    let t = sim.units[u].pos.tile();
    // Lit now, and seen.
    sim.world.set_seen(t.x >> 1, t.y >> 1, 1 << 1);
    assert!(sim.danger_unit_is_seen(u, 1), "lit now");
    // Lit once and since left: `seen2` keeps it, `seen` does not.
    sim.world.set_fog(vec![0; 32 * 32]);
    sim.world.set_seen2_only(t.x >> 1, t.y >> 1, 1 << 1);
    assert!(!sim.danger_unit_is_seen(u, 1), "ever seen is not seen");
    sim.units[u].visible |= 1 << 1;
    assert!(sim.danger_unit_is_seen(u, 1), "it attacked leader 1");
}

/// **A stealth unit is hidden until a detector's inner disc covers it**
/// (item 2725, `UnitData::is_seen@00607a60`'s stealth arm and
/// `is_detected@0060a630`): an idle `s` unit (the Explorer) or an `o`
/// one (the Spy) answers unseen through a lit cell until a `DETECT`
/// object of the viewer's alliance has written `seen3` over it; a plain
/// unit on the same cell is the control.
#[test]
fn a_stealth_unit_is_hidden_until_a_detector_s_disc_covers_it() {
    use crate::UnitType;
    use crate::ai_load::{UnitCols, uflags};
    let (mut sim, _city, _barracks) = fx();
    let mk = |sim: &mut Sim, flags: u32, obj: u32| {
        let mut ty = UnitType {
            cols: UnitCols {
                unit_flags: flags,
                ..UnitCols::default()
            },
            ..UnitType::default()
        };
        ty.combat.obj_masks = obj;
        sim.add_unit_type(ty)
    };
    let plain = mk(&mut sim, 0, 0);
    let idle_stealth = mk(&mut sim, uflags::STEALTH, 0);
    let cloaked = mk(&mut sim, uflags::CLOAKED, 0);
    let at = tile_pos(8 * 4 + 1, 8 * 4 + 1);
    let (p, s, c) = (
        sim.init_unit(0, plain, at),
        sim.init_unit(0, idle_stealth, at),
        sim.init_unit(0, cloaked, at),
    );
    assert!(sim.world.set_fog(vec![0; 32 * 32]), "a 16 x 16 world's fog");
    let t = at.tile();
    // The cell is lit for leader 1; nothing detects.
    sim.world.set_seen(t.x >> 1, t.y >> 1, 1 << 1);
    assert!(sim.danger_unit_is_seen(p, 1), "the control");
    assert!(
        !sim.danger_unit_is_seen(s, 1),
        "idle stealth, lit, undetected"
    );
    assert!(!sim.danger_unit_is_seen(c, 1), "cloaked, lit, undetected");
    assert!(sim.danger_unit_is_seen(s, 0), "its owner sees it");
    // A detector's inner disc over the cell.
    sim.world.set_seen3(t.x >> 1, t.y >> 1, 1 << 1);
    assert!(
        sim.danger_unit_is_seen(s, 1) && sim.danger_unit_is_seen(c, 1),
        "detected"
    );
    // The hundredth-frame clear takes `seen3` with `seen`.
    sim.world.clear_seen();
    assert_eq!(sim.world.seen3(t.x >> 1, t.y >> 1), 0, "cleared with seen");
    // A detector unit lights `seen3` through `update_seen`.
    let det = mk(&mut sim, 0, crate::combat::mask::DETECT);
    let d = sim.init_unit(1, det, at);
    sim.units[d].on_map = true;
    sim.units[d].mylos = 6;
    sim.units[d].health = 10;
    sim.update_seen(d, false);
    assert_ne!(
        sim.world.seen3(t.x >> 1, t.y >> 1) & (1 << 1),
        0,
        "the scout's disc"
    );
    assert!(sim.danger_unit_is_seen(s, 1), "seen through it");
}

/// The weights: a city a flat hundred, a fort or a tower its own
/// remaining hit points halved, a military trainer fifty and anything
/// else ten.
#[test]
fn the_four_weights() {
    let (mut sim, city, barracks) = fx();
    let tower = sim.add_build_type(BuildType {
        ident: Ident::Tower,
        x_size: 2,
        y_size: 2,
        hits: 750,
        ..BuildType::default()
    });
    let farm = sim.add_build_type(BuildType {
        ident: Ident::Farm,
        x_size: 4,
        y_size: 4,
        hits: 400,
        ..BuildType::default()
    });
    let c = sim.init_build(0, city, tile_pos(8, 8), false);
    sim.activate(c, false, true);
    assert!(sim.buildings[c].city.is_some(), "activating founds it");
    assert_eq!(sim.danger_building_value(c), 100, "a city");

    let t = sim.init_build(0, tower, tile_pos(20, 8), false);
    sim.buildings[t].health = 500;
    assert_eq!(sim.danger_building_value(t), 250, "half its hit points");

    let bk = sim.init_build(0, barracks, tile_pos(30, 8), false);
    assert_eq!(sim.danger_building_value(bk), 50, "a military trainer");

    let f = sim.init_build(0, farm, tile_pos(40, 8), false);
    assert_eq!(sim.danger_building_value(f), 10, "everything else");
}

/// The cadence is `frame % 200`, and **nothing decays the map in
/// between** — a building finished on frame 201 weighs nothing until
/// 400.
#[test]
fn the_map_is_rebuilt_every_two_hundredth_frame_and_never_between() {
    let (mut sim, _city, barracks) = fx();
    sim.tables.build_types[barracks].flags |= flags::NO_CITY;
    sim.tick(); // frame 0 builds an empty map
    assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), 0);
    let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
    sim.buildings[b].active = true;
    for _ in 1..200 {
        sim.tick();
    }
    assert_eq!(
        sim.world.danger_half(0, Cell::new(8, 8)),
        0,
        "199 frames of standing there and the map has not moved"
    );
    sim.tick();
    assert_eq!(sim.frame, 201, "frame 200 is the rebuild");
    assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), -50);
}

/// The map is 8 x `reg_size` and `reg_size` is `((xs · 4) >> 3)²`, not
/// `(xs / 2)²` — an odd cell width rounds down.
#[test]
fn the_grid_is_the_tile_width_halved() {
    let w = World::new(60, 60);
    assert_eq!((w.reg_xs(), w.reg_ys(), w.reg_size()), (30, 30, 900));
    let odd = World::new(61, 61);
    assert_eq!(odd.reg_xs(), 30, "(61 * 4) >> 3 is 30, not 31");
}
