use crate::world::Owner;

/// **A citizen heals a point every 45 frames in its own territory**,
/// on the frame `(o + frame) % 45 == 0`, and the heal clears the
/// fraction (`docs/COMBAT.md` §72). Outside it, or on another frame,
/// or as a soldier, nothing heals.
#[test]
fn a_citizen_heals_a_point_every_45_frames_in_its_own_territory() {
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let ty = |sim: &mut crate::Sim, type_index: i32| {
        sim.add_unit_type(crate::UnitType {
            hits: 40,
            type_index,
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        })
    };
    let citizen = ty(&mut sim, 0x32);
    let hoplite = ty(&mut sim, 0x60);
    let put = |sim: &mut crate::Sim, t: usize, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(0, index, p, 40);
        u.ty = Some(t);
        u.on_map = true;
        u.kind = sim.tables.unit_types[t].kind;
        sim.add_unit(u)
    };
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let c = put(&mut sim, citizen, at);
    let soldier = put(&mut sim, hoplite, at);
    sim.world
        .set_owner(at.cell(), Owner::Player(0), Owner::Player(0));
    for u in [c, soldier] {
        sim.units[u].health = sim.units[u].max_health - 3;
        sim.units[u].damage_frac = 5;
    }
    let o = i64::from(sim.units[c].index);
    let due = 45 * 100 - o;
    sim.civilian_heal(c, due - 1);
    assert_eq!(
        (sim.units[c].health, sim.units[c].damage_frac),
        (sim.units[c].max_health - 3, 5),
        "off the phase, nothing"
    );
    assert_eq!(sim.units[c].healing, 0, "no heal, no clock");
    sim.civilian_heal(c, due);
    assert_eq!(
        (sim.units[c].health, sim.units[c].damage_frac),
        (sim.units[c].max_health - 2, 0),
        "on the phase, a point and the fraction"
    );
    // Item 3961: the heal sets `healing` to `max(45, healing)` —
    // run1151c's Citizen `0/8` reads 45 on 4493 and 4538.
    assert_eq!(sim.units[c].healing, 45, "the clock, at the rate");
    sim.units[c].healing = 50;
    sim.units[c].health -= 1;
    sim.civilian_heal(c, due + 45);
    assert_eq!(sim.units[c].healing, 50, "a longer clock stands");
    let os = i64::from(sim.units[soldier].index);
    sim.civilian_heal(soldier, 45 * 100 - os);
    assert_eq!(
        sim.units[soldier].health,
        sim.units[soldier].max_health - 3,
        "a soldier is not a civilian"
    );
    let cell = sim.units[c].pos.cell();
    sim.world.set_owner(cell, Owner::None, Owner::None);
    sim.civilian_heal(c, due + 45);
    assert_eq!(
        sim.units[c].health,
        sim.units[c].max_health - 2,
        "outside an ally's territory, nothing"
    );
}

/// **An Aircraft Carrier repairs a point every 20 frames, anywhere; a
/// Citrus holder's ships do too** (item 3854, `docs/GOLDEN.md` §153.2):
/// `process_healing`'s sea arm, phased on `(o + frame) %
/// UNIT_HEAL_RATE[0]`. A Destroyer heals only under Citrus, a
/// transport never, and a carrier under Citrus takes two points on its
/// phase. run1485's carriers `1/18` and `2/18` (`o` 18) healed a point
/// each on game frame 802, the cell nobody's.
#[test]
fn a_carrier_repairs_a_point_every_20_frames_and_citrus_heals_every_ship() {
    use crate::ai_load::{UnitCols, uflags};
    use crate::attrition::{Domain, UnitKind};
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let ty = |sim: &mut crate::Sim, type_index: i32, unit_flags: u32| {
        sim.add_unit_type(crate::UnitType {
            hits: 600,
            type_index,
            kind: UnitKind {
                domain: Domain::Sea,
                ..UnitKind::default()
            },
            cols: UnitCols {
                unit_flags,
                ..UnitCols::default()
            },
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        })
    };
    let carrier = ty(&mut sim, 0x15f, 0);
    let destroyer = ty(&mut sim, 328, 0);
    let freighter = ty(&mut sim, 322, uflags::FLAG_TRANSPORT);
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    // `o` 18, 38 and 58: one phase of 20 for all three.
    let put = |sim: &mut crate::Sim, t: usize, o: i16| {
        let mut u = crate::Unit::new(1, o, at, 600);
        u.ty = Some(t);
        u.type_index = sim.tables.unit_types[t].type_index;
        u.on_map = true;
        u.captain = true;
        u.kind = sim.tables.unit_types[t].kind;
        let i = sim.add_unit(u);
        sim.units[i].health = 600 - 10;
        sim.units[i].damage_frac = 7;
        i
    };
    let ships = [
        put(&mut sim, carrier, 18),
        put(&mut sim, destroyer, 38),
        put(&mut sim, freighter, 58),
    ];
    let due = 820 - 18;
    let state = |sim: &crate::Sim| ships.map(|i| (sim.units[i].health, sim.units[i].damage_frac));
    for &i in &ships {
        sim.ship_heal(i, due - 1);
    }
    assert_eq!(state(&sim), [(590, 7); 3], "off the phase, nothing");
    for &i in &ships {
        sim.ship_heal(i, due);
    }
    assert_eq!(
        state(&sim),
        [(591, 0), (590, 7), (590, 7)],
        "on the phase, the carrier alone: a point and the fraction"
    );
    sim.ledgers[1].rare |= 1 << (crate::economy::CITRUS - crate::economy::BASE_RARE);
    for &i in &ships {
        sim.ship_heal(i, due + 20);
    }
    assert_eq!(
        state(&sim),
        [(593, 0), (591, 0), (590, 7)],
        "under Citrus: the carrier two, the destroyer one, the transport none"
    );
}

/// **A CEO heals its army abroad, a point every 20 frames, down the
/// squad** (item 3686, `docs/AI.md` §348.1): `process_healing`'s third
/// economic-patriot arm runs on the captain, phased on the captain's
/// `o`, and its `repair_damage(1, …)` hands the point down `o_down`
/// past an unhurt figure to the hurt one, zeroing its fraction. The
/// cell is nobody's. Off the phase, without a CEO, or under an
/// attrition period, nothing. run1338b's `1/36` (captain `1/35`) went
/// 4 and 14/16 to 3 and 6/16 on block 3167, a point and then the
/// bleed. Made to fail with the repair stopping at the captain
/// (`tools/mutate.py`, `cur = None`).
#[test]
fn a_ceo_heals_the_hurt_figure_of_a_squad_abroad_every_20_frames() {
    use crate::ai_load::{UnitCols, uflags2};
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let rifle = sim.add_unit_type(crate::UnitType {
        hits: 645,
        type_index: 137,
        combat: crate::combat::Profile {
            uber_size: 3,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let ceo = sim.add_unit_type(crate::UnitType {
        hits: 110,
        type_index: 0x165,
        cols: UnitCols {
            unit_flags2: uflags2::FLAG2_GENERAL | uflags2::SUPPLY_OR_HERO,
            ..UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let put = |sim: &mut crate::Sim, t: usize, hits: i32| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(1, index, at, hits);
        u.ty = Some(t);
        u.type_index = sim.tables.unit_types[t].type_index;
        u.on_map = true;
        u.kind = sim.tables.unit_types[t].kind;
        sim.add_unit(u)
    };
    let squad = [
        put(&mut sim, rifle, 215),
        put(&mut sim, rifle, 215),
        put(&mut sim, rifle, 215),
    ];
    sim.units[squad[0]].captain = true;
    sim.units[squad[0]].o_down = Some(squad[1]);
    sim.units[squad[1]].captain = false;
    sim.units[squad[1]].o_up = Some(squad[0]);
    sim.units[squad[1]].o_down = Some(squad[2]);
    sim.units[squad[2]].captain = false;
    sim.units[squad[2]].o_up = Some(squad[0]);
    let hurt = squad[2];
    sim.units[hurt].health = 215 - 4;
    sim.units[hurt].damage_frac = 14;
    let c = squad[0];
    let o = i64::from(sim.units[c].index);
    let due = 20 * 160 - o;
    let state = |sim: &crate::Sim| (sim.units[hurt].health, sim.units[hurt].damage_frac);
    sim.ceo_heal(c, due);
    assert_eq!(state(&sim), (211, 14), "no CEO, nothing");
    let boss = put(&mut sim, ceo, 110);
    sim.units[boss].captain = true;
    sim.ceo_heal(c, due - 1);
    assert_eq!(state(&sim), (211, 14), "off the phase, nothing");
    sim.ceo_heal(hurt, due);
    assert_eq!(
        state(&sim),
        (211, 14),
        "a figure below the captain is not asked"
    );
    sim.ceo_heal(c, due);
    assert_eq!(
        state(&sim),
        (212, 0),
        "a point down the chain, and the fraction"
    );
    assert_eq!(
        sim.units[squad[1]].health, 215,
        "the unhurt figure passes it on"
    );
    sim.units[c].attrition = 48;
    sim.ceo_heal(c, due + 20);
    assert_eq!(state(&sim), (212, 0), "under an attrition period, nothing");
}

/// A captain of player 1 on its own cell, 40 hits, three down with a
/// fraction, in a tree whose one row is Herbal Lore, the heal ladder's
/// first rung (`TECHBONUSES` row 67).
fn iroquois_captain(type_index: i32) -> (crate::Sim, usize, usize) {
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let mut tree = crate::tech::TechTree::new();
    let herbal = tree.add(crate::tech::TypeDef::plain("Herbal Lore", 0));
    tree.roles.heal_faster_preq = [Some(herbal), None, None];
    sim.set_tech_tree(tree);
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 40,
        type_index,
        combat: crate::combat::Profile {
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let mut u = crate::Unit::new(1, 50, at, 40);
    u.ty = Some(ty);
    u.type_index = type_index;
    u.on_map = true;
    u.captain = true;
    u.kind = sim.tables.unit_types[ty].kind;
    let c = sim.add_unit(u);
    sim.units[c].health = 37;
    sim.units[c].damage_frac = 5;
    sim.world
        .set_owner(at.cell(), Owner::Player(1), Owner::Player(1));
    (sim, c, herbal)
}

fn attack_to(dest: crate::world::Pos) -> crate::orders::Order {
    crate::orders::Order {
        flags: crate::orders::flag::ACTION,
        body: crate::orders::Body::Move(crate::orders::MoveOrder {
            kind: crate::orders::MoveKind::AttackTo,
            dest,
            angle: crate::movement::Angle(0),
            facing: None,
            has_waypoint: false,
            waypoint: dest,
            last: None,
            pause: 0,
            retry: 0,
            attempts: 0,
            timer: 0,
            coll: None,
            orig: None,
            group: None,
        }),
    }
}

/// **An Iroquois captain heals standing on its own ground, at the heal
/// level's period** (item 3959, `docs/AI.md` §403.2): a point every 20
/// frames, 15 under Herbal Lore, phased on `o`; nothing under an
/// attack-move (`is_moving`), when blown, on another leader's ground,
/// or for a leader without the bonus. run1228b's `1/50` was refused
/// on 8186, 8201 and 8216 under its `ATTACKTOORDER` and healed on 8246
/// under its `GUARDORDER`.
/// **A French unit heals within its own supply source's radius, on
/// its phase, walking** (item 3978, `docs/AI.md` §410): the supply
/// heal's period is `SUPPLY_HEAL_RATE` (0) plus the French bonus's 20,
/// and `find_supply` asks only the supply list's reach — 14 tiles,
/// 2688 by `vector_dist`. run1542's Elite Special Forces `1/17` stood
/// (1709, 1865) off the Despot `1/164`, 2648, and healed on 1143
/// under an explore order; its `healing` read 13 on block 1151.
#[test]
fn a_french_unit_heals_within_its_supply_radius_every_20_frames() {
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.set_tech_tree(crate::tech::TechTree::new());
    let esf = sim.add_unit_type(crate::UnitType {
        hits: 130,
        type_index: 77,
        combat: crate::combat::Profile {
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let despot = sim.add_unit_type(crate::UnitType {
        hits: 109,
        type_index: 352,
        kind: crate::attrition::UnitKind {
            supply_unit: true,
            ..crate::attrition::UnitKind::default()
        },
        ..crate::UnitType::default()
    });
    let at = Pos::new(25157, 9797);
    let put = |sim: &mut crate::Sim, t: usize, index: i16, pos: Pos| {
        let mut u = crate::Unit::new(1, index, pos, sim.tables.unit_types[t].hits);
        u.ty = Some(t);
        u.type_index = sim.tables.unit_types[t].type_index;
        u.on_map = true;
        u.captain = true;
        u.kind = sim.tables.unit_types[t].kind;
        sim.add_unit(u)
    };
    let c = put(&mut sim, esf, 17, at);
    sim.units[c].health = 130 - 11;
    sim.units[c].damage_frac = 4;
    let state = |sim: &crate::Sim| {
        let u = &sim.units[c];
        (u.health, u.damage_frac, u.healing)
    };
    sim.supply_heal(c, 1143);
    assert_eq!(state(&sim), (119, 4, 0), "no bonus, no period");
    sim.tech[1].power = Some(crate::nations::power::FRENCH);
    sim.supply_heal(c, 1143);
    assert_eq!(state(&sim), (119, 4, 0), "no supply source, nothing");
    let d = put(&mut sim, despot, 164, Pos::new(at.x - 1709, at.y + 1865));
    assert!(
        sim.units[d].supply_slot.is_some(),
        "the Despot holds a slot"
    );
    sim.supply_heal(c, 1142);
    assert_eq!(state(&sim), (119, 4, 0), "off the phase, nothing");
    sim.units[c]
        .orders
        .push_front(attack_to(Pos::new(25848, 10488)));
    sim.supply_heal(c, 1143);
    assert_eq!(
        state(&sim),
        (120, 0, 20),
        "(17 + 1143) % 20, walking, 2648 off: a point and the fraction"
    );
    sim.units[d].pos = Pos::new(at.x - 1709 - 60, at.y + 1865);
    sim.supply_heal(c, 1163);
    assert_eq!(state(&sim), (120, 0, 20), "past 2688, nothing");
}

#[test]
fn an_iroquois_captain_heals_standing_on_its_own_ground() {
    let (mut sim, c, herbal) = iroquois_captain(167);
    let state = |sim: &crate::Sim| (sim.units[c].health, sim.units[c].damage_frac);
    sim.iroquois_heal(c, 8190);
    assert_eq!(state(&sim), (37, 5), "no bonus, nothing");
    sim.tech[1].power = Some(crate::nations::power::IROQUOIS);
    assert!(sim.iroquois(1), "the bonus holds in this setup");
    sim.iroquois_heal(c, 8189);
    assert_eq!(state(&sim), (37, 5), "off the phase, nothing");
    let at = sim.units[c].pos;
    sim.units[c].orders.push_front(attack_to(at));
    sim.iroquois_heal(c, 8190);
    assert_eq!(state(&sim), (37, 5), "an attack-move is moving");
    sim.units[c].orders.clear();
    sim.units[c].blown = true;
    sim.iroquois_heal(c, 8190);
    assert_eq!(state(&sim), (37, 5), "a blown unit, nothing");
    sim.units[c].blown = false;
    sim.iroquois_heal(c, 8190);
    assert_eq!(state(&sim), (38, 0), "standing, level 0: (50 + 8190) % 20");
    sim.tech[1].tech[herbal] = true;
    assert_eq!(sim.heal_level(1), 1, "Herbal Lore is the first rung");
    sim.iroquois_heal(c, 8210);
    assert_eq!(state(&sim), (38, 0), "level 1 is not on the 20s");
    sim.iroquois_heal(c, 8245);
    assert_eq!(state(&sim), (39, 0), "level 1: (50 + 8245) % 15");
    let cell = sim.units[c].pos.cell();
    sim.world
        .set_owner(cell, Owner::Player(0), Owner::Player(0));
    sim.iroquois_heal(c, 8260);
    assert_eq!(state(&sim), (39, 0), "another leader's ground, nothing");
}

/// **The Senator heals a walking column on allied ground, every 20
/// frames** (item 3959, `docs/AI.md` §403.3): `process_healing`'s
/// `0x161` arm asks no standing test, only an allied cell and a
/// Senator of the owner in reach. run1228b's `1/50`, walking under an
/// `ATTACKTOORDER` with `1/92` near, healed on 8191, 8211 and 8231.
#[test]
fn the_senator_heals_a_walking_column_on_allied_ground() {
    use crate::ai_load::{UnitCols, uflags2};
    let (mut sim, c, _) = iroquois_captain(167);
    let state = |sim: &crate::Sim| (sim.units[c].health, sim.units[c].damage_frac);
    let at = sim.units[c].pos;
    sim.units[c].orders.push_front(attack_to(at));
    sim.senator_heal(c, 8190);
    assert_eq!(state(&sim), (37, 5), "no Senator, nothing");
    let senator = sim.add_unit_type(crate::UnitType {
        hits: 110,
        type_index: 0x161,
        cols: UnitCols {
            unit_flags2: uflags2::FLAG2_GENERAL | uflags2::SUPPLY_OR_HERO,
            ..UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let mut u = crate::Unit::new(1, 92, sim.units[c].pos, 110);
    u.ty = Some(senator);
    u.type_index = 0x161;
    u.on_map = true;
    u.captain = true;
    u.kind = sim.tables.unit_types[senator].kind;
    sim.add_unit(u);
    sim.senator_heal(c, 8189);
    assert_eq!(state(&sim), (37, 5), "off the phase, nothing");
    sim.senator_heal(c, 8190);
    assert_eq!(state(&sim), (38, 0), "walking, a point and the fraction");
    let cell = sim.units[c].pos.cell();
    sim.world.set_owner(cell, Owner::None, Owner::None);
    sim.senator_heal(c, 8210);
    assert_eq!(state(&sim), (38, 0), "nobody's ground, nothing");
}

/// **A patriot repairs its leader's damaged building by the ground under
/// it** (item 3987, `docs/GOLDEN.md` §180; `Build::process@0061edf0`'s
/// arm to `0061f3b9`): a President's reach repairs on unowned or allied
/// ground and nowhere else — and, found, it closes the Senator's branch
/// even where that one would repair; a Senator's only on allied ground;
/// a point on the phase `(o + frame) % 20`, the fraction cleared,
/// `healing` 20; nothing without the leader's count of the type.
#[test]
fn a_patriot_repairs_its_leader_s_building_by_the_ground_under_it() {
    use crate::ai_load::{UnitCols, uflags2};
    use crate::world::{Pos, World};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.tables.build_types.push(crate::build::BuildType {
        x_size: 3,
        y_size: 3,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(9000, 9000);
    let b = sim.add_building(1, at, 0);
    sim.buildings[b].ty = Some(sim.tables.build_types.len() - 1);
    sim.buildings[b].index = 2007;
    sim.buildings[b].active = true;
    sim.buildings[b].hits = 1200;
    let hero = |sim: &mut crate::Sim, type_index: i32| {
        let t = sim.add_unit_type(crate::UnitType {
            hits: 110,
            type_index,
            cols: UnitCols {
                unit_flags2: uflags2::FLAG2_GENERAL | uflags2::SUPPLY_OR_HERO,
                ..UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let mut u = crate::Unit::new(1, 6 + type_index as i16, at, 110);
        u.ty = Some(t);
        u.type_index = type_index;
        u.on_map = true;
        u.captain = true;
        u.kind = sim.tables.unit_types[t].kind;
        (sim.add_unit(u), t)
    };
    let (senator, st) = hero(&mut sim, 0x161);
    let (president, pt) = hero(&mut sim, 0x163);
    sim.units[president].pos.x += 40 * 192;
    let n = sim.tables.unit_types.len();
    sim.muster[1].by_type.resize(n, 0);
    let cell = at.cell();
    let run = |sim: &mut crate::Sim, frame: i64, ground: Owner| {
        sim.world.set_owner(cell, ground, Owner::None);
        let bd = &mut sim.buildings[b];
        (bd.damage, bd.damage_frac, bd.healing) = (10, 5, 0);
        sim.patriot_repair(b, frame);
        let bd = &sim.buildings[b];
        (bd.damage, bd.damage_frac, bd.healing)
    };
    let own = Owner::Player(1);
    assert_eq!(run(&mut sim, 13, own), (10, 5, 0), "no patriot counted");
    sim.muster[1].by_type[st] = 1;
    assert_eq!(
        run(&mut sim, 13, own),
        (9, 0, 20),
        "a Senator on its own ground"
    );
    assert_eq!(run(&mut sim, 14, own), (10, 5, 0), "off the phase");
    assert_eq!(
        run(&mut sim, 13, Owner::None),
        (10, 5, 0),
        "a Senator on nobody's ground"
    );
    assert_eq!(
        run(&mut sim, 13, Owner::Player(0)),
        (10, 5, 0),
        "a Senator on an enemy's"
    );
    sim.muster[1].by_type[pt] = 1;
    sim.units[president].pos = at;
    assert_eq!(
        run(&mut sim, 33, Owner::None),
        (9, 0, 20),
        "a President on nobody's ground"
    );
    assert_eq!(run(&mut sim, 33, own), (9, 0, 20), "a President on its own");
    sim.units[senator].pos.x += 40 * 192;
    assert_eq!(
        run(&mut sim, 33, Owner::Player(0)),
        (10, 5, 0),
        "a President on an enemy's"
    );
    sim.units[senator].pos = at;
    assert_eq!(
        run(&mut sim, 33, Owner::Player(0)),
        (10, 5, 0),
        "a President found closes the Senator's branch"
    );
}
