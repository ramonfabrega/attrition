#[test]
fn a_gaia_owner_has_no_modifiers() {
    // A nuke's ring strikes the animals around a city (`docs/AI.md`
    // §158): their owner is past `mods`, which holds the players.
    let sim = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(16, 16),
        2,
    );
    assert_eq!(sim.mods_of(9).attack, 0);
    assert_eq!(sim.mods_of(9).armor, 0);
    assert_eq!(sim.mods_of(9).range, 0);
    assert_eq!(sim.mods_of(1), sim.combat.mods[1]);
}

/// **The Japanese level is the leader's own, read on the blow** (item
/// 3590, `get_damage@00644130:243`–`262`): `min(ages, epoch[0])` for an
/// owner with tribe bonus `0xf`, nothing for another nation. run1208d's
/// who=1 reads `ages_get()` 3 and `epoch_get` 3 on 2675, and its
/// Arquebusiers' blow on the Citizen `0/12` is 13 + 10/16, not 12.
#[test]
fn a_japanese_owner_s_blow_takes_the_lesser_of_its_ages_and_military_level() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(8, 8), 2);
    sim.tech[1].ages = 3;
    sim.tech[1].epoch = [2, 4, 0, 0];
    assert_eq!(sim.mods_of(1).japanese_level, None, "no bonus, no level");
    sim.tech[1].power = Some(crate::nations::power::JAPANESE);
    assert_eq!(
        sim.mods_of(1).japanese_level,
        Some(2),
        "epoch[0] the lesser"
    );
    sim.tech[1].epoch = [3, 3, 2, 2];
    sim.tech[1].ages = 3;
    assert_eq!(sim.mods_of(1).japanese_level, Some(3), "run1208d's leader");
    sim.tech[1].ages = 1;
    assert_eq!(sim.mods_of(1).japanese_level, Some(1), "ages the lesser");
    assert_eq!(sim.mods_of(0).japanese_level, None, "the other player");
    assert_eq!(sim.mods_of(9).japanese_level, None, "a gaia owner");
}

/// **`max_range`'s CEO term** (`UnitData::max_range@0060fe90`, item
/// 2825, `docs/AI.md` §236): a ranged unit within its owner's `0x165`
/// hero's radius (here 10 tiles, 1920) reaches two tiles further; one
/// a tile outside it, a siege type, another player's hero, and a hero
/// the leader does not count do not. run881's Strategic Bomber
/// `1/137` bombs on 6452 at `attack_dist` 918 with who=1's `1/161`
/// 2371 off. Made to fail with the term dropped and with the siege
/// test dropped.
#[test]
fn a_unit_near_its_owner_s_ceo_reaches_two_tiles_further() {
    use crate::ai_load::uflags2;
    use crate::{Unit, UnitType};
    let mut s = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let mut shooter = UnitType::default();
    shooter.combat.max_range = 3;
    let shooter = s.add_unit_type(shooter);
    let mut ceo = UnitType {
        type_index: 0x165,
        ..UnitType::default()
    };
    ceo.cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    let ceo = s.add_unit_type(ceo);
    let place = |s: &mut Sim, who: u8, ty: usize, x: i32| {
        let mut u = Unit::new(who, 0, Pos::new(x, 9000), 50);
        u.ty = Some(ty);
        u.type_index = s.tables.unit_types[ty].type_index;
        u.on_map = true;
        s.add_unit(u)
    };
    let bomber = place(&mut s, 1, shooter, 9000);
    let range = |s: &Sim| s.max_range_of(Obj::Unit(bomber));
    assert_eq!(range(&s), 3, "no hero");
    let hero = place(&mut s, 1, ceo, 9000 + 1920);
    assert_eq!(range(&s), 3, "a hero the leader does not count");
    s.roster.muster[1].by_type[ceo] = 1;
    assert_eq!(s.hero_radius(hero), 10, "6 × 3 / 2 + 1");
    assert_eq!(range(&s), 5, "inside the radius");
    s.units[hero].pos.x += 1;
    assert_eq!(range(&s), 3, "one past it");
    s.units[hero].pos.x -= 1;
    s.units[hero].owner = 0;
    s.roster.muster[0].by_type[ceo] = 1;
    assert_eq!(range(&s), 3, "another player's hero");
    s.units[hero].owner = 1;
    s.tables.unit_types[shooter].combat.siege = true;
    assert_eq!(range(&s), 3, "a siege type");
}

use super::*;
use crate::world::{PLAYER_SLOTS, World};

/// Two players at war, and a soldier type that can shoot.
fn at_war() -> (Sim, usize) {
    at_war_with_slots(2)
}

/// The same, with `slots` player slots — so a test can *widen* the
/// diplomacy table past the eight the original has and check that the
/// leader bound, not the table's length, is what refuses gaia.
fn at_war_with_slots(slots: usize) -> (Sim, usize) {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), slots);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    (sim, ty)
}

fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.tables.unit_types[ty].hits;
    let mut u = crate::Unit::new(who, index, p, hits);
    u.ty = Some(ty);
    u.on_map = true;
    u.kind = sim.tables.unit_types[ty].kind;
    sim.add_unit(u)
}

/// A round of `shooter`'s landing at `at`, on `target`, done.
fn round(
    sim: &Sim,
    shooter: usize,
    target: Option<Obj>,
    at: Pos,
    splash: i32,
) -> combat::Projectile {
    combat::Projectile {
        shooter: Obj::Unit(shooter),
        owner: sim.units[shooter].owner,
        target,
        launch: sim.units[shooter].pos,
        landing: at,
        cur_time: 1,
        total_time: 1,
        accuracy: 100,
        angle: Angle(0),
        splash_area: splash,
        num_guys: 1,
        air: false,
        rolling: false,
        missed: false,
        harmless: false,
        sz: 0,
        ez: 0,
        v1z: combat::arc_v1z(0, 0, 1),
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    }
}

/// **A splash round strikes its own target whoever owns it**
/// (`Ammo::do_damage`, `6787d9`..`6787e1`, item 1200): the team test
/// is asked only of an object that is not the round's target. Chapter
/// forty-one's Biplane on 858, whose round `check_hit` put on who=1's
/// own `1/2003`: the original spends that building's first-wound roll.
/// Another of the shooter's buildings in the splash is left out, and
/// so is an enemy's standing clear of it. Made to fail with the
/// target's exemption dropped.
#[test]
fn a_splash_round_strikes_its_own_target_whoever_owns_it() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
    let building = |sim: &mut Sim, who: Player, p: Pos| {
        let b = sim.add_building(who, p, 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = true;
        sim.buildings[b].combat = Some(Profile {
            x_size: 1,
            y_size: 1,
            ..Profile::default()
        });
        sim.buildings[b].health = 500;
        b
    };
    let mine = building(&mut sim, 1, Pos::new(2016, 2016));
    let other = building(&mut sim, 1, Pos::new(2208, 2016));
    let p = round(&sim, me, Some(Obj::Building(mine)), Pos::new(2016, 2016), 1);
    sim.land(p, 858);
    let struck: Vec<Obj> = sim.combat.hits.iter().map(|h| h.target).collect();
    assert_eq!(struck, vec![Obj::Building(mine)], "the target alone");
    assert!(!sim.combat.hits[0].splash, "at the full count");
    let _ = other;
}

/// **A round whose target is gone lands as a ground round** (item
/// 3829, `Ammo::do_damage@00678060`'s first test, `docs/GOLDEN.md`
/// §152.12): an air round on a plane that died in flight splashes no
/// plane beside its landing, and one whose target still stands does.
/// run1476's `0/15` round of 792 on the dead `1/17` left `1/18` whole.
/// Made to fail with the round's own `air` kept at landing.
#[test]
fn a_round_whose_target_died_splashes_no_plane() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            domain: Domain::Air,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, ty, Pos::new(1000, 1000));
    let dead = put(&mut sim, 1, ty, Pos::new(3000, 3000));
    let near = put(&mut sim, 1, ty, Pos::new(3000, 3200));
    let air_round = |sim: &Sim| combat::Projectile {
        air: true,
        ..round(sim, me, Some(Obj::Unit(dead)), Pos::new(3000, 3000), 4)
    };
    let mut live = sim.clone();
    let p = air_round(&live);
    live.land(p, 797);
    assert!(
        live.combat.hits.iter().any(|h| h.target == Obj::Unit(near)),
        "a standing target's round splashes the plane beside it"
    );
    sim.units[dead].health = 0;
    let p = air_round(&sim);
    sim.land(p, 797);
    assert!(
        sim.combat.hits.iter().all(|h| h.target != Obj::Unit(near)),
        "a dead target's round lands as ground"
    );
}

/// **A decoy target takes `get_damage`'s step 17** (item 3385): armour
/// nothing and the blow `max(attack, base) × 1000`, so any round kills
/// it. `target_side` wrote `decoy` false for every unit; run1011e's
/// packet struck the General's decoy Light Horse `1/82` for 19688 where
/// this crate dealt 6. Made to fail with the target's bit dropped.
#[test]
fn a_decoy_target_dies_to_any_round() {
    for decoy in [false, true] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
        let them = put(&mut sim, 0, ty, Pos::new(2016, 2016));
        sim.units[them].decoy = decoy;
        let p = round(&sim, me, Some(Obj::Unit(them)), Pos::new(2016, 2016), 0);
        sim.land(p, 3385);
        assert_eq!(!sim.units[them].alive(), decoy, "decoy {decoy}: dead");
    }
}

/// **A burning round kills with `dtype` 4** (item 3580, `docs/AI.md`
/// §313): `Object::do_damage@0064a480`'s `verify_ammo_flags(…, 0x10)`
/// on the round's ammo piece — `effects_graphics.xml`'s `flame="1"`,
/// the Fire Lances' `FirelanceAmmo` — overrides `get_damage`'s 2, and
/// `Unit::close` then spends its three draws (`+0xcb6`, `+0xce6`,
/// `+0xd0a`) where an ordinary death spends one: run1202b's Fire Lances
/// `1/31` on the Citizen `0/10` at 3030. Made to fail with the arm dropped.
#[test]
fn a_burning_round_kills_with_the_ammo_graphic_s_three_draws() {
    let sites = |flame: bool| {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
        let them = put(&mut sim, 0, ty, Pos::new(2016, 2016));
        sim.units[them].health = 1;
        let mut p = round(&sim, me, Some(Obj::Unit(them)), Pos::new(2016, 2016), 0);
        p.flame = flame;
        sim.trace_phases = true;
        sim.land(p, 3030);
        assert!(!sim.units[them].alive(), "flame {flame}: dead");
        let marks: Vec<String> = sim
            .phase_marks
            .iter()
            .map(|(l, _)| l.clone())
            .filter(|l| l.starts_with("Unit::close"))
            .collect();
        (marks, sim.combat.deaths.last().map(|d| d.cur_anim))
    };
    let (plain, _) = sites(false);
    assert_eq!(plain, [SITE_DEATH_ANIM]);
    let (burning, anim) = sites(true);
    assert_eq!(
        burning,
        [SITE_DEATH_ANIM, SITE_DEATH_ANIM_ALT, SITE_DEATH_ANIM_FACING]
    );
    assert!(matches!(anim, Some(0x11 | 0x12)), "{anim:?}");
}

/// **A decoy's overflow strikes its captain** (`Object::take_damage
/// @00652020:619`–`631`, item 3393): a round on the tail of a decoy
/// squad of three takes `get_damage`'s step 17, and the points past each
/// figure's share pass up the chain as `close` re-links it, so the whole
/// squad falls on the one round — run1015b's Inti Macemen `1/92`,
/// `1/90`, `1/91` on 13392. A squad that is no decoy loses no figure to
/// the one blow. Made to fail with the arm dropped.
#[test]
fn a_decoy_squad_falls_to_one_round_on_its_tail() {
    for decoy in [false, true] {
        let (mut sim, ty) = at_war();
        sim.tables.unit_types[ty].combat.uber_size = 3;
        let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
        let chain: Vec<usize> = (0..3)
            .map(|k| put(&mut sim, 0, ty, Pos::new(2016 + 48 * k, 2016)))
            .collect();
        let head = i32::from(sim.units[chain[0]].index);
        for (k, &u) in chain.iter().enumerate() {
            let un = &mut sim.units[u];
            un.decoy = decoy;
            un.squad_size = 3;
            un.captain = k == 0;
            un.combat.captain = head;
            un.o_up = (k > 0).then(|| chain[k - 1]);
            un.o_down = chain.get(k + 1).copied();
        }
        let p = round(&sim, me, Some(Obj::Unit(chain[2])), Pos::new(2112, 2016), 0);
        sim.land(p, 3393);
        let dead: Vec<bool> = chain.iter().map(|&u| !sim.units[u].alive()).collect();
        if decoy {
            assert_eq!(dead, [true, true, true], "the decoy squad falls whole");
        } else {
            assert_eq!(dead, [false, false, false], "one blow, no figure down");
        }
    }
}

/// **`UnitData::armor@00610160`'s rally arm** (item 3385): a hero's
/// own unit inside its radius, `get_radius × 0xc0` by `vector_dist`,
/// and the hero itself, add `GENERAL_RALLY_ARMOR × (general_upgrade +
/// 1)`; one tile-unit past the radius, or another player's hero, adds
/// nothing. Made to fail with the arm dropped.
#[test]
fn a_general_rallies_armour_inside_its_radius() {
    use crate::ai_load::uflags2;
    let (mut sim, ty) = at_war();
    sim.tables.unit_types[ty].combat.armor = 2;
    let mut general = crate::UnitType {
        type_index: 0x100,
        hits: 100,
        ..crate::UnitType::default()
    };
    general.cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    let general = sim.add_unit_type(general);
    let me = put(&mut sim, 1, ty, Pos::new(9000, 9000));
    let armour = |s: &Sim| s.armor_of(Obj::Unit(me));
    assert_eq!(armour(&sim), 2, "no hero");
    let hero = put(&mut sim, 1, general, Pos::new(9000, 9000));
    let reach = sim.hero_radius(hero) * 0xc0;
    sim.units[hero].pos.x += reach;
    let rally = sim.tables.tuning.general_rally_armor;
    assert_eq!(rally, 2, "the shipped GENERAL_RALLY_ARMOR");
    assert_eq!(armour(&sim), 2 + rally, "on the radius");
    assert_eq!(sim.armor_of(Obj::Unit(hero)), rally, "the hero itself");
    sim.units[hero].pos.x += 1;
    assert_eq!(armour(&sim), 2, "one past it");
    sim.units[hero].pos.x -= 1;
    sim.units[hero].owner = 0;
    assert_eq!(armour(&sim), 2, "another player's hero");
}

/// **The patriots' armour inside the rally arm** (item 3986,
/// `UnitData::armor@00610160`'s two last terms, `docs/GOLDEN.md` §179):
/// a unit in reach of one of its owner's `THECITIZEN`s (`0x164`, The
/// Comrade) adds `THECITIZEN_ARMOR_BONUS` beside the rally term; a
/// Stable-trained unit in reach of its owner's Monarch (`0x162`) adds
/// `THEMONARCH_CAV_ARMOR`, and a unit of any other trainer does not.
/// Made to fail with either term dropped.
#[test]
fn a_comrade_and_a_monarch_add_their_armour_inside_the_rally_arm() {
    use crate::ai_load::uflags2;
    use crate::build::{BuildType, Ident};
    let (mut sim, ty) = at_war();
    sim.tables.unit_types[ty].combat.armor = 1;
    let rally = sim.tables.tuning.general_rally_armor;
    assert_eq!(
        (
            sim.tables.tuning.thecitizen_armor_bonus,
            sim.tables.tuning.themonarch_cav_armor
        ),
        (2, 2),
        "the shipped THECITIZEN_ARMOR_BONUS and THEMONARCH_CAV_ARMOR"
    );
    let hero = |sim: &mut Sim, type_index: i32| {
        let mut t = crate::UnitType {
            type_index,
            hits: 100,
            ..crate::UnitType::default()
        };
        t.cols.unit_flags2 = uflags2::FLAG2_GENERAL;
        sim.add_unit_type(t)
    };
    let comrade = hero(&mut sim, 0x164);
    let monarch = hero(&mut sim, 0x162);
    // A Stable type: its tree row's `where_` is a building type whose
    // ident is the Stable.
    let mut tree = crate::tech::TechTree::new();
    let horse = tree.add(crate::tech::TypeDef::unit(
        "Heavy Knight",
        crate::tech::UnitTraits::default(),
    ));
    let stab = tree.add(crate::tech::TypeDef::building("Stable"));
    tree.types[horse].where_ = Some(stab);
    tree.finalize();
    sim.set_tech_tree(tree);
    sim.tables.build_types.push(BuildType {
        ident: Ident::Stable,
        tree: Some(stab),
        ..BuildType::default()
    });
    let knight_t = sim.add_unit_type(crate::UnitType {
        hits: 100,
        tree: Some(horse),
        combat: Profile {
            armor: 1,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    assert_eq!(sim.trainer_ident(knight_t), Some(Ident::Stable));
    let foot = put(&mut sim, 1, ty, Pos::new(9000, 9000));
    let knight = put(&mut sim, 1, knight_t, Pos::new(9000, 9000));
    let armour = |s: &Sim, u: usize| s.armor_of(Obj::Unit(u));
    let c = put(&mut sim, 1, comrade, Pos::new(9192, 9000));
    sim.units[c].type_index = 0x164;
    assert_eq!(armour(&sim, foot), 1 + rally + 2, "a Comrade beside it");
    assert_eq!(
        armour(&sim, knight),
        1 + rally + 2,
        "the knight, the Comrade's term alone"
    );
    sim.units[c].owner = 0;
    assert_eq!(armour(&sim, foot), 1, "another player's Comrade");
    let m = put(&mut sim, 1, monarch, Pos::new(9192, 9000));
    sim.units[m].type_index = 0x162;
    assert_eq!(armour(&sim, foot), 1 + rally, "a Monarch: no term on foot");
    assert_eq!(
        armour(&sim, knight),
        1 + rally + 2,
        "a Monarch beside Stable cavalry"
    );
    sim.units[m].pos.x += sim.hero_radius(m) * 0xc0;
    assert_eq!(armour(&sim, knight), 1, "the Monarch out of reach");
}

/// **A Senator armours its own buildings in reach** (item 3986,
/// `WallData::armor@0063fa60`'s Senator term, `docs/GOLDEN.md` §179):
/// one of the owner's `THESENATOR`s (`0x161`) whose radius covers the
/// building less its footprint's `(x_size + y_size) × 0x60` adds
/// `THESENATOR_BUILD_ARMOR` — and before the unfinished halving, so a
/// site holds half the sum. Made to fail with the term dropped, and
/// with it taken after the halving.
#[test]
fn a_senator_armours_its_buildings_before_the_halving() {
    use crate::ai_load::uflags2;
    let (mut sim, _) = at_war();
    assert_eq!(
        sim.tables.tuning.thesenator_build_armor, 5,
        "the shipped THESENATOR_BUILD_ARMOR"
    );
    sim.tables.build_types.push(crate::build::BuildType {
        x_size: 3,
        y_size: 3,
        ..crate::build::BuildType::default()
    });
    let bt = sim.tables.build_types.len() - 1;
    let b = sim.add_building(1, Pos::new(9000, 9000), 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].combat = Some(Profile {
        armor: 4,
        x_size: 3,
        y_size: 3,
        ..Profile::default()
    });
    let armour = |s: &Sim| s.armor_of(Obj::Building(b));
    assert_eq!(armour(&sim), 4, "no Senator");
    let mut t = crate::UnitType {
        type_index: 0x161,
        hits: 100,
        ..crate::UnitType::default()
    };
    t.cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    let senator = sim.add_unit_type(t);
    let reach = (3 + 3) * 0x60;
    let h = put(&mut sim, 1, senator, Pos::new(9000, 9000));
    sim.units[h].type_index = 0x161;
    sim.units[h].pos.x += sim.hero_radius(h) * 0xc0 + reach;
    assert_eq!(armour(&sim), 9, "on the reach, the footprint's slack");
    sim.buildings[b].active = false;
    assert_eq!(armour(&sim), 4, "a site: (4 + 5) / 2");
    sim.buildings[b].active = true;
    sim.units[h].pos.x += 1;
    assert_eq!(armour(&sim), 4, "one past it");
    sim.units[h].pos.x -= 1;
    sim.units[h].owner = 0;
    assert_eq!(armour(&sim), 4, "another player's Senator");
}

/// **A decoy's round strikes nothing** (`Object::do_damage@0064a480`,
/// item 1522): the attacker is a unit with `unit_masks & 1`, and the
/// function returns after its `count < 1` test and before `get_damage`
/// — no damage, no `build_masks` bookkeeping, and no first-wound draw.
/// run666's Bombard decoys `1/96`, `1/95` and `1/106` landed on
/// `0/2000` three times without a mark where `1/139` and `1/84`, the
/// same type with the bit clear, took 135 off it. Made to fail with the
/// gate dropped.
#[test]
fn a_decoy_s_round_strikes_nothing() {
    for decoy in [false, true] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
        sim.units[me].decoy = decoy;
        let b = sim.add_building(0, Pos::new(2016, 2016), 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = true;
        sim.buildings[b].combat = Some(Profile {
            x_size: 1,
            y_size: 1,
            ..Profile::default()
        });
        sim.buildings[b].health = 500;
        let before = sim.rng;
        let p = round(&sim, me, Some(Obj::Building(b)), Pos::new(2016, 2016), 1);
        sim.land(p, 1522);
        assert_eq!(
            sim.combat.hits.len(),
            usize::from(!decoy),
            "decoy {decoy}: a strike"
        );
        assert_eq!(
            sim.buildings[b].damage > 0,
            !decoy,
            "decoy {decoy}: the building's damage"
        );
        assert_eq!(
            sim.rng == before,
            decoy,
            "decoy {decoy}: the first-wound draw is spent by a real blow alone"
        );
    }
}

/// **A closed decoy's round is a real blow** (`Unit::close@0060ee50:603`,
/// item 3322): `close` clears `unit_masks & 1` at its tail, so a round
/// still in flight when its decoy closes lands as an ordinary one —
/// run956c's Horse Archers decoy `1/110`, closed on 18367, struck who=0's
/// Village for 1.0 on 18368 (`docs/AI.md` §265). Made to fail with
/// `clear_decoy_at_close` taken out of `close_decoy`.
#[test]
fn a_closed_decoy_s_round_in_flight_strikes() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
    sim.units[me].decoy = true;
    let b = sim.add_building(0, Pos::new(2016, 2016), 0);
    sim.buildings[b].started = true;
    sim.buildings[b].active = true;
    sim.buildings[b].combat = Some(Profile {
        x_size: 1,
        y_size: 1,
        ..Profile::default()
    });
    sim.buildings[b].health = 500;
    let p = round(&sim, me, Some(Obj::Building(b)), Pos::new(2016, 2016), 1);
    sim.close_decoy(me);
    assert!(!sim.units[me].decoy, "a closed unit is no decoy");
    sim.land(p, 3322);
    assert_eq!(sim.combat.hits.len(), 1, "the round strikes");
    assert!(sim.buildings[b].damage > 0, "the building takes it");
}

/// **An aircraft's round passes over its own side** (`Ammo::check_hit`,
/// `678db9`..`678dcc`, item 1200): an air shooter that is not a
/// missile searches `SEARCH_NON_FRIENDLY`, which leaves out its own
/// player's units and nothing else; any other shooter searches them
/// all. Chapter forty-one's Biplane on 969, whose round came down
/// beside who=1's `1/1`. Made to fail with the test dropped.
#[test]
fn an_aircraft_s_round_passes_over_its_own_side() {
    for air in [true, false] {
        let (mut sim, ty) = at_war();
        let mut t = sim.tables.unit_types[ty].clone();
        t.combat.domain = if air { Domain::Air } else { Domain::Land };
        let shooter_ty = sim.add_unit_type(t);
        let mut foot = sim.tables.unit_types[ty].clone();
        foot.combat.target_size = 300;
        let foot = sim.add_unit_type(foot);
        let me = put(&mut sim, 1, shooter_ty, Pos::new(1000, 1000));
        let own = put(&mut sim, 1, foot, Pos::new(3050, 3000));
        let foe = put(&mut sim, 0, foot, Pos::new(3100, 3000));
        let p = round(&sim, me, None, Pos::new(3000, 3000), 0);
        sim.land(p, 969);
        let want = if air { foe } else { own };
        assert_eq!(
            sim.combat.hits.last().map(|h| h.target),
            Some(Obj::Unit(want)),
            "air {air}"
        );
    }
}

/// **A ship that attacks sideways turns broadside, to the nearer side**
/// (`Unit::fight@005fd4d0:698–714`, `docs/COMBAT.md` §49). run127's
/// own numbers: who=1's trireme at (12408, 35832), heading `0x55555555`
/// (120°), attacking who=0's at (11640, 34680). The bearing is
/// `−402259968` (−33.7°); the plus side, `671481856`, is 63.7° from the
/// heading and the minus side 116.3°, so the plus side it is, which is
/// what the dump's `angle` holds on block 617. The same geometry from a
/// heading on the other side takes the minus side, a type without `g`
/// takes the bearing, and a Patrol Boat shooting at a ship does too.
///
/// Made to fail first by returning the bearing: the golden widening's
/// `1/6` angle rows on 617 are that failure.
#[test]
fn a_sideways_ship_attacks_broadside_on_the_nearer_side() {
    let (mut sim, ty) = at_war();
    let ship = |sim: &mut Sim, flags: u32, type_index: i32| {
        let mut t = sim.tables.unit_types[ty].clone();
        t.cols.unit_flags = flags;
        t.type_index = type_index;
        t.combat.domain = Domain::Sea;
        sim.add_unit_type(t)
    };
    let trireme = ship(&mut sim, uflags::SIDEWAYS, 0x154);
    let plain = ship(&mut sim, 0, 0x154);
    let patrol = ship(&mut sim, uflags::SIDEWAYS, 0x185);
    let me = put(&mut sim, 1, trireme, Pos::new(12408, 35832));
    let foe = put(&mut sim, 0, trireme, Pos::new(11640, 34680));
    let direct = find_angle(11640 - 12408, 34680 - 35832);
    assert_eq!(direct, Angle(-402_259_968), "run127's bearing");
    sim.units[me].movement.heading = Angle(0x5555_5555);
    assert_eq!(
        sim.attack_angle(me, Obj::Unit(foe), direct),
        Angle(671_481_856),
        "the plus side, as the dump's block 617 holds"
    );
    sim.units[me].movement.heading = Angle(direct.0.wrapping_sub(0x3000_0000));
    assert_eq!(
        sim.attack_angle(me, Obj::Unit(foe), direct),
        Angle(direct.0.wrapping_sub(0x4000_0000)),
        "the minus side when it is the nearer"
    );
    sim.units[me].ty = Some(plain);
    assert_eq!(sim.attack_angle(me, Obj::Unit(foe), direct), direct);
    sim.units[me].ty = Some(patrol);
    assert_eq!(
        sim.attack_angle(me, Obj::Unit(foe), direct),
        direct,
        "a Patrol Boat does not broadside a ship"
    );
}

/// **A type whose pivot bears shoots on its own heading** —
/// `Unit::set_attack@005fce70` and `Guy::set_all_pivots@005d8bc0`
/// (`docs/COMBAT.md` §52). run145's chariot `0/8`: born facing
/// `0x5555_5555` (120°), its target 36.9° off that, and the dump's
/// `angle` still `1431655765` on 634, after the shot. The same unit
/// with the target 60° off turns, a type with no `<RESTRICTION>` turns
/// at 37°, and a node whose own range does not cover the bearing turns
/// too, both for a plain range (a Katyusha's −20..20) and for a
/// wrapped one (a Dreadnought's `45..−45`, the rear arc). The crew
/// figure is never aimed.
///
/// Made to fail first: with `set_attack` answering 0, the first
/// assertion reads the bearing, `993918976` — the golden widening's
/// 634 row in shape (the capture's own bearing is `991232000`; this
/// geometry is the capture's to within a degree).
#[test]
fn a_pivot_that_bears_shoots_without_turning() {
    let (mut sim, ty) = at_war();
    const CHARIOT: i32 = 195;
    let chariot = {
        let mut t = sim.tables.unit_types[ty].clone();
        t.type_index = CHARIOT;
        t.combat.max_range = 8;
        sim.add_unit_type(t)
    };
    sim.roster
        .art
        .pivots
        .insert(CHARIOT, [(4, (-180, 180))].into_iter().collect());
    let here = Pos::new(984, 8136);
    let shoot = |sim: &mut Sim, ty: usize, at: Pos| -> (Angle, Option<Obj>) {
        let me = put(sim, 0, ty, here);
        sim.units[me].guys = vec![anim::Guy::fresh(-1), anim::Guy::fresh(-1)];
        sim.units[me].movement.heading = Angle(0x5555_5555);
        sim.units[me].movement.facing = Angle(0x5555_5555);
        let foe = put(sim, 1, ty, at);
        sim.fight_pub(me, Obj::Unit(foe), 633);
        let aim = sim.units[me].guys[0].aim;
        assert_eq!(aim, Some(Obj::Unit(foe)), "figure 0 is aimed");
        assert_eq!(sim.units[me].guys[1].aim, None, "the crew is not");
        (sim.units[me].movement.heading, aim)
    };
    // run145's `0/8` and a target on the dump's bearing, 36.9° off.
    let near = Pos::new(here.x + 1536, here.y - 192);
    let bearing = find_angle(1536, -192);
    let off = crate::movement::angle_to_degrees(Angle(bearing.0.wrapping_sub(0x5555_5555)));
    assert_eq!(off - 360, -37, "the geometry is the capture's");
    assert_eq!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
    // Due south, 60° off: past the ±45° every pivot is held to.
    let far = Pos::new(here.x, here.y + 1536);
    assert_ne!(shoot(&mut sim, chariot, far).0, Angle(0x5555_5555));
    // No `<RESTRICTION>`: the plain type turns at 37°.
    assert_ne!(shoot(&mut sim, ty, near).0, Angle(0x5555_5555));
    // A node that cannot reach 37°, plain and wrapped.
    sim.roster
        .art
        .pivots
        .insert(CHARIOT, [(4, (-20, 20))].into_iter().collect());
    assert_ne!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
    sim.roster
        .art
        .pivots
        .insert(CHARIOT, [(4, (45, -45))].into_iter().collect());
    assert_ne!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
    sim.roster
        .art
        .pivots
        .insert(CHARIOT, [(4, (-45, -30))].into_iter().collect());
    assert_eq!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
}

/// **The pivot bears from its node, not the unit's square**
/// (`docs/COMBAT.md` §54). run145's `0/8` on 684: at `984, 8136`,
/// facing 120°, its target `1/7` at `1512, 7944` bears 69.9°, 50° off
/// the heading from the square. The Chariot's figure (piece 145) has
/// its node at `(−102, −59)` from the square at that facing, and from
/// there `1/7` is 42° off, so the original shoots without turning:
/// run147's packet answers `Unit::set_attack(0/8, 7, 1)` with 1, and
/// `(6, 1)`, `1/6` at `1608, 7800` (51° off from the node), with 0.
///
/// Made to fail first: with `pivot::offset` answering `(0, 0)` the
/// first assertion reads the bearing, `834011136` — the widening's
/// 685 row.
#[test]
fn a_pivot_bears_from_its_node() {
    let (mut sim, ty) = at_war();
    const CHARIOT: i32 = 195;
    let chariot = {
        let mut t = sim.tables.unit_types[ty].clone();
        t.type_index = CHARIOT;
        t.combat.max_range = 8;
        sim.add_unit_type(t)
    };
    sim.roster
        .art
        .pivots
        .insert(CHARIOT, [(4, (-180, 180))].into_iter().collect());
    let here = Pos::new(984, 8136);
    let heading = Angle(0x5555_5555);
    let shoot = |sim: &mut Sim, piece: i32, at: Pos| -> Angle {
        let me = put(sim, 0, chariot, here);
        sim.units[me].guys = vec![anim::Guy::fresh(piece), anim::Guy::fresh(12817)];
        sim.units[me].movement.heading = heading;
        sim.units[me].movement.facing = heading;
        let foe = put(sim, 1, ty, at);
        sim.fight_pub(me, Obj::Unit(foe), 684);
        sim.units[me].movement.heading
    };
    let seven = Pos::new(1512, 7944);
    let off = |from: Pos| {
        let b = find_angle(seven.x - from.x, seven.y - from.y);
        crate::movement::angle_to_degrees(Angle(heading.0.wrapping_sub(b.0)))
    };
    assert_eq!(
        (off(here), off(Pos::new(here.x - 102, here.y - 59))),
        (50, 42),
        "the geometry is the capture's"
    );
    assert_eq!(shoot(&mut sim, 145, seven), heading, "from the node");
    assert_ne!(shoot(&mut sim, -1, seven), heading, "a piece with no node");
    assert_ne!(
        shoot(&mut sim, 145, Pos::new(1608, 7800)),
        heading,
        "1/6 is past 45° from the node too"
    );
}

/// `ObjectData::valid_target_const@006472c0`'s first line, `7 < who`,
/// which runs *before* the diplomacy question. To show that it is the
/// leader bound doing the work and not the sim's short table, the table
/// here is ten wide and gaia is declared at war: the animal is still
/// nobody's target, and the ring search — which walks the cell chains
/// with no leader bound of its own — steps over it to the enemy behind.
#[test]
fn a_gaia_animal_is_no_ones_target_even_at_war() {
    let (mut sim, ty) = at_war_with_slots(usize::from(PLAYER_SLOTS) + 2);
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let sheep = put(&mut sim, PLAYER_SLOTS, ty, Pos::new(0x1040, 0x1000));
    sim.at_war[0][usize::from(PLAYER_SLOTS)] = true;
    sim.at_war[usize::from(PLAYER_SLOTS)][0] = true;
    assert!(sim.units[sheep].is_gaia());
    assert!(sim.is_enemy(0, PLAYER_SLOTS), "the table says enemy");
    assert!(sim.valid_target(Obj::Unit(me), Obj::Unit(foe)));
    assert!(!sim.valid_target(Obj::Unit(me), Obj::Unit(sheep)));
    assert_eq!(
        sim.find_nearby_target(Obj::Unit(me), 0),
        Some(Obj::Unit(foe))
    );
}

/// **A sea-only attacker takes only a sea unit** (item 3646,
/// `ObjectData::valid_target_const@006472c0`, `6473c3`–`6473ea`): a
/// type whose `unit_flags` carries `0x2000` (`n`, the fire rafts and
/// fire ships) or `0x100000` (`u`, the submarines) refuses a land
/// unit and a building, a sea one among them, and takes a ship. A
/// Frigate beside it, with neither flag, takes all three — run1304b's
/// group 71 on 4859, where the Fire Ship `1/86` is never handed the
/// Citizen `0/12` its Frigate and Bombard Ships are. Made to fail on
/// purpose with the arm dropped.
/// **A Bomber-line searcher refuses a helicopter** (item 3800,
/// `ObjectData::valid_target_const@006472c0`, `006474ee`'s arm): a
/// helicopter target (`unit_flags & 0x20`) is refused to a searcher
/// of the Bomber line (`is(0x130)`) or a tank (`unit_flags &
/// 0x80000`), and taken by any other air unit with a range — run1468's
/// Stealth Bomber `1/17` is never handed the Attack Helicopter `2/15`
/// on 622, where the Jet Fighter `1/14` is (`docs/GOLDEN.md` §152.1).
/// Made to fail on purpose with the arm dropped.
#[test]
fn a_bomber_never_takes_a_helicopter() {
    let (mut sim, land) = at_war();
    let air_type = |sim: &mut Sim, tree: Option<crate::tech::TypeId>, flags: u32| {
        let mut t = sim.tables.unit_types[land].clone();
        t.combat.domain = Domain::Air;
        t.cols.unit_flags = flags;
        let ty = sim.add_unit_type(t);
        // The lineage after the add: the fixture keeps no tech table.
        sim.tables.unit_types[ty].tree = tree;
        ty
    };
    let bomber = air_type(&mut sim, Some(crate::air::BOMBER), 0);
    let fighter = air_type(&mut sim, None, 0);
    let heli = air_type(&mut sim, None, crate::ai_load::uflags::FLAG_HELICOPTER);
    let mut tank = sim.tables.unit_types[land].clone();
    tank.cols.unit_flags = crate::ai_load::uflags::TANK;
    let tank = sim.add_unit_type(tank);
    let at = |x| Pos::new(0x1000 + x, 0x1000);
    let b = put(&mut sim, 1, bomber, at(0));
    let f = put(&mut sim, 1, fighter, at(0x20));
    let k = put(&mut sim, 1, tank, at(0x40));
    let target = Obj::Unit(put(&mut sim, 0, heli, at(0x100)));
    assert!(!sim.valid_target_const(Obj::Unit(b), target), "the bomber");
    assert!(!sim.valid_target_const(Obj::Unit(k), target), "the tank");
    assert!(sim.valid_target_const(Obj::Unit(f), target), "the fighter");
}

/// **An air target is aimed at its figure's altitude** (item 3804,
/// `Ammo::init@0067bbf0`, `0x67c949`..: a unit of domain `+0x218 ==
/// 2` loads `guys[0] +0x14`; `docs/GOLDEN.md` §152.3): run1469's
/// Attack Helicopter `1/15` at `z` 60 is aimed at 60 and the parked
/// Jet Fighter `1/14` at 0, whatever the ground under them; a land
/// unit keeps its tile's. Made to fail on purpose with the ground
/// read back (60 against the tile's).
#[test]
fn an_air_target_is_aimed_at_its_figure_s_altitude() {
    let (mut sim, land) = at_war();
    let mut t = sim.tables.unit_types[land].clone();
    t.combat.domain = Domain::Air;
    t.cols.unit_flags = crate::ai_load::uflags::FLAG_HELICOPTER;
    let heli = sim.add_unit_type(t);
    let at = Pos::new(0x1000, 0x1000);
    let h = put(&mut sim, 0, heli, at);
    let ground = sim.ground_z(at);
    sim.units[h].airframe.z = ground + 60;
    assert_eq!(sim.aim_z(Obj::Unit(h), false), (ground + 60).max(0));
    sim.units[h].airframe.z = 0;
    assert_eq!(sim.aim_z(Obj::Unit(h), false), 0, "a parked plane");
    let c = put(&mut sim, 0, land, at);
    assert_eq!(
        sim.aim_z(Obj::Unit(c), false),
        sim.world.object_z(at.tile()).max(0),
        "a land unit"
    );
}

#[test]
fn a_fire_ship_or_a_submarine_takes_only_a_ship() {
    let (mut sim, land) = at_war();
    let sea_type = |sim: &mut Sim, flags: u32| {
        let mut t = sim.tables.unit_types[land].clone();
        t.combat.domain = Domain::Sea;
        t.cols.unit_flags = flags;
        sim.add_unit_type(t)
    };
    let fire_ship = sea_type(&mut sim, 0x2000);
    let submarine = sea_type(&mut sim, 0x10_0000);
    let frigate = sea_type(&mut sim, 0);
    let at = |x| Pos::new(0x1000 + x, 0x1000);
    let askers = [
        put(&mut sim, 1, fire_ship, at(0)),
        put(&mut sim, 1, submarine, at(0x20)),
        put(&mut sim, 1, frigate, at(0x40)),
    ];
    let citizen = Obj::Unit(put(&mut sim, 0, land, at(0x100)));
    let ship = Obj::Unit(put(&mut sim, 0, frigate, at(0x120)));
    let dock = sim.add_building(0, at(0x140), 1);
    sim.buildings[dock].hits = 100;
    sim.buildings[dock].health = 100;
    sim.buildings[dock].combat = Some(Profile {
        domain: Domain::Sea,
        ..Profile::default()
    });
    let dock = Obj::Building(dock);
    let takes = |sim: &Sim, a: usize| {
        [citizen, dock, ship].map(|t| sim.valid_target_const(Obj::Unit(a), t))
    };
    assert_eq!(
        takes(&sim, askers[0]),
        [false, false, true],
        "the fire ship"
    );
    assert_eq!(
        takes(&sim, askers[1]),
        [false, false, true],
        "the submarine"
    );
    assert_eq!(takes(&sim, askers[2]), [true, true, true], "the frigate");
}

/// **A unit killed on the map takes what it carries with it** (item
/// 3862, `docs/GOLDEN.md` §153.6): `Object::close@00647160`'s contents
/// arm is `kill_contents` for every `Unit` (vslot `+0x18` answers 1),
/// and each rider dies by `Object::die(p, 0, −1, 0)` — no death draw.
/// run1483: the Freighter `1/54` sinks and the Elite Special Forces
/// `1/0` aboard is gone from theirs' closing block; ours kept it alive
/// aboard a dead boat, and its slot, until item 3862.
///
/// Made to fail by dropping [`Sim::kill_contents`] from the kill arm:
/// the rider lives on, inside a boat that is gone.
#[test]
fn a_unit_killed_on_the_map_takes_its_rider_with_it() {
    let (mut sim, ty) = at_war();
    let boat = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let rider = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    sim.units[rider].on_map = false;
    sim.units[rider].inside_unit = Some(boat);
    let rng = sim.rng;
    sim.units[boat].health = 1;
    let taken = sim.take_damage_typed(
        Obj::Unit(boat),
        Sixteenths { whole: 5, frac: 0 },
        Obj::Unit(foe),
        10,
        1,
        false,
    );
    assert!(matches!(taken, Taken::Died { .. }), "the boat sinks");
    assert!(!sim.units[rider].alive(), "its rider goes with it");
    assert_eq!(sim.units[rider].inside_unit, None);
    assert_eq!(sim.units[rider].hold_frames, Sim::CLOSE_HOLD);
    assert!(sim.units[foe].alive());
    // The boat's own death draws (it is on the map); the rider's,
    // `dtype` 0 and off the map, draws nothing past it.
    let mut once = rng;
    let _ = once.roll();
    assert_eq!(sim.rng, once, "one death draw, the boat's");
}

/// **A hit is answered by the victim's captain, never by the figure that
/// took it.** `Unit::target_opportunity@005fffc0`'s opening `while`
/// breaks on `is_captain` and otherwise re-enters on
/// `objects[who][this->o_up]`, so every test past it — and the
/// `add_attack_order` at its foot — runs on the head of the squad. The
/// golden record measures it (`docs/COMBAT.md` §18.2): `1/6` strikes
/// `0/7` on frame 616 and it is `0/6` that carries the ATTACKORDER at
/// that frame's end, with `0/7` a frame behind.
///
/// Made to fail on purpose: answering on the victim gives
/// `(None, Some(foe))`, which is what this crate did until item 391 and
/// what put the retaliation a frame early on the golden record's 617.
#[test]
fn the_hit_is_answered_by_the_victim_s_captain() {
    let (mut sim, ty) = at_war();
    let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    sim.units[sub].captain = false;
    sim.units[sub].o_up = Some(cap);
    sim.units[cap].o_down = Some(sub);
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Unit(sub),
        crate::movement::Angle(0),
        false,
        1,
        false,
        false,
        10,
    );
    assert_eq!(
        (sim.units[cap].combat.target, sim.units[sub].combat.target),
        (Some(Obj::Unit(foe)), None),
        "the captain retaliates and the figure that was hit does not"
    );
}

/// **A figure of a squad falls at its share of the squad's hit
/// points, not at the whole of them** — `Object::take_damage`'s step
/// 8 (`docs/COMBAT.md` §7.2, §7.3, §41.4).
///
/// `update_hits` writes one number, the squad's, onto every figure;
/// the divide is on the way in. So a squad of three with `HITS` 120
/// is three figures of 40, and the dump's `myhits` is 120 on each of
/// them with `damage` counting that figure's own share of what has
/// been taken. This crate kept the squad-sized maximum (item 484's
/// row pins it on 245,679 field-frames) and used it as the
/// **threshold** too, so a figure absorbed three figures' worth:
/// run112's hoplite `1/8` reached `damage` 51 still standing where
/// the original's died on block 683 at its fortieth point, and
/// `extra 1/8` was that.
///
/// **Made to fail on purpose** by passing `u.health` again, which is
/// what stood here: the figure survives the third hit and the
/// assertion below reddens on `alive`.
///
/// `combat::share`'s captain remainder is the second half and is not
/// reachable from here — it needs the squad down to one figure — so
/// the arithmetic is tested directly in `combat`'s own tests and the
/// call site is what this pins.
#[test]
fn a_figure_falls_at_its_share_of_the_squad_s_hits() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 120,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 3,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    sim.units[me].squad_size = 3;
    sim.units[me].combat.captain = i32::from(sim.units[me].index);
    // Three fifteen-point hits: 15, 30, 45 against a share of 40.
    let hit = combat::Sixteenths { whole: 15, frac: 0 };
    for expected in [false, false, true] {
        let died = matches!(
            sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10),
            Taken::Died { .. }
        );
        assert_eq!(died, expected, "share is 120 / 3 = 40");
    }
    assert!(!sim.units[me].alive(), "a dead figure is not alive");
    // And the squad-sized maximum is untouched, because item 484's
    // row compares it against the dump's `myhits`.
    assert_eq!(sim.units[me].max_health, 120);
}

/// **A struck unit that survives raises `0x10` on its captain** (item
/// 3983, `Object::take_damage@00652020:345`–`350`, `docs/AI.md` §415):
/// the member is struck, the captain takes the bit; a blow that kills
/// the figure does not reach the write.
#[test]
fn a_survived_blow_raises_the_captain_s_cant_reach() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 120,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 3,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let mem = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let c = i32::from(sim.units[cap].index);
    for u in [cap, mem] {
        sim.units[u].combat.captain = c;
    }
    let hit = combat::Sixteenths { whole: 15, frac: 0 };
    sim.take_damage(Obj::Unit(mem), hit, Obj::Unit(foe), 10);
    assert!(sim.units[cap].cant_reach, "the captain takes it");
    assert!(!sim.units[mem].cant_reach, "the struck member does not");
    sim.units[cap].cant_reach = false;
    let kill = combat::Sixteenths { whole: 40, frac: 0 };
    sim.take_damage(Obj::Unit(mem), kill, Obj::Unit(foe), 11);
    assert!(!sim.units[cap].cant_reach, "a kill is past the write");
}

/// **The remainder share is the last figure's** (item 3618,
/// `docs/AI.md` §326.2): `Object::take_damage` gives a squad figure
/// `hits − (uber − 1) · hits / uber` only when it is a captain whose
/// live `curr_uber_size(this, 1)` is 1. A captain with two live
/// members takes `110 / 3 = 36` and falls at its thirty-sixth point;
/// standing alone it takes `110 − 73 = 37`. This crate read the stored
/// `squad_size`, 1 here, and gave the captain 37 either way.
#[test]
fn a_captain_takes_the_remainder_share_only_alone() {
    let falls_at_36 = |members: bool| {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 110,
            combat: Profile {
                attack: 15,
                max_range: 4,
                uber_size: 3,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let c = i32::from(sim.units[me].index);
        sim.units[me].combat.captain = c;
        if members {
            let a = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1030));
            let b = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1060));
            sim.units[me].o_down = Some(a);
            sim.units[a].o_up = Some(me);
            sim.units[a].o_down = Some(b);
            sim.units[b].o_up = Some(a);
            sim.units[a].combat.captain = c;
            sim.units[b].combat.captain = c;
        }
        let hit = combat::Sixteenths { whole: 36, frac: 0 };
        matches!(
            sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10),
            Taken::Died { .. }
        )
    };
    assert!(falls_at_36(true), "a whole squad's captain: share 36");
    assert!(!falls_at_36(false), "the last figure standing: share 37");
}

/// **A unit struck while moving in contact takes the blow doubled**
/// (item 3567, `docs/AI.md` §309): `Unit::target_opportunity`'s move
/// arm (`600175`..`6002c8`) raises `unit_masks & 0x10` on a victim
/// whose front order is a move and whose attacker stands inside the
/// two types' `big_radius` summed, `Object::do_damage` doubles the
/// blow under it (`0064a5dd`), `UnitData::get_speed` halves the land
/// unit's speed (`0060878c`), and the end of `Unit::work` clears it.
/// The Roman pair's 4068: the Citizen `0/13`, walking to its gather,
/// took the Light Horse `1/42`'s 7 as 14. Each arm of the predicate is
/// asked by a victim that refuses it: standing, out of reach, and
/// attacking a target near enough.
#[test]
fn a_unit_struck_while_moving_in_contact_takes_the_blow_doubled() {
    let blow = |moving: bool, gap: i32, attacking: bool| {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            moves: 40,
            combat: Profile {
                attack: 70,
                max_range: 0,
                uber_size: 1,
                big_radius: 60,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x4000 + gap, 0x4000));
        if attacking {
            sim.units[me].orders.push_front(crate::orders::Order {
                flags: 0x10,
                body: crate::orders::Body::Attack(crate::orders::AttackOrder {
                    defensive: false,
                    def: None,
                    in_range: true,
                    ever_in_range: true,
                    new_ord: false,
                }),
            });
            sim.units[me].combat.target = Some(Obj::Unit(foe));
        }
        if moving && attacking {
            // The chase's transit move in front of the attack, as
            // `1/42`'s own queue stood on 4066.
            sim.add_move_order(
                me,
                Pos::new(0x4000, 0x5000),
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::First,
                false,
            );
        } else if moving {
            sim.order_move(me, Pos::new(0x4000, 0x5000));
        }
        let taken = sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            0x100,
            false,
            false,
            10,
        );
        let Some(Taken::Alive { lost }) = taken else {
            panic!("the blow killed or missed: {taken:?}");
        };
        (lost, sim, me)
    };
    // Standing: no move, no mark — the blow as `get_damage` prices it.
    let base = blow(false, 100, false).0;
    // Moving, 100 apart against a reach of 60 + 60: doubled.
    let (lost, mut sim, me) = blow(true, 100, false);
    assert_eq!(lost, 2 * base, "the blow doubled");
    assert!(
        sim.units[me].contact,
        "the bit stands for the unit's next work"
    );
    sim.units[me].movement.speed = 40;
    let halved = sim.get_speed(me, 0);
    sim.units[me].contact = false;
    assert_eq!(
        halved,
        sim.get_speed(me, 0) / 2,
        "a land unit's speed halves"
    );
    sim.units[me].contact = true;
    sim.work(me, 11);
    assert!(!sim.units[me].contact, "Unit::work's last line clears it");
    // Out of reach: 121 against the 120 summed.
    assert_eq!(blow(true, 121, false).0, base, "outside the summed radii");
    assert_eq!(blow(true, 120, false).0, 2 * base, "the sum is inclusive");
    // Attacking a target inside twice the sum: the arm refuses.
    assert_eq!(blow(true, 100, true).0, base, "its own target is near");
}

/// **A hero or a supply unit flees on a shorter ring** (item 4005,
/// `docs/COMBAT.md` §34.2, `Unit::target_opportunity@005fffc0`,
/// `6006e2`–`600785`): `0x180` from a unit attacker with a range
/// (vslot `+0x130`, `max_range`), `0x300` from a melee one, and the
/// `0x600` of anything else; a hero in an Ambush or on a Forced March
/// (`unit_masks & 0xa000`) does not flee. Chapter 129's Senator `0/6`,
/// struck by a Hoplite on 946, flees 751 units on 947 to
/// `(6792, 28632)`, the `0x300` ring.
#[test]
fn a_hero_or_supply_unit_flees_on_the_attacker_s_ring() {
    let (mut sim, archer) = at_war();
    let melee = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 0,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let mk = |sim: &mut Sim, flags2: u32| {
        let mut t = crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 0,
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        };
        t.cols.unit_flags2 = flags2;
        sim.add_unit_type(t)
    };
    let hero = mk(&mut sim, crate::ai_load::uflags2::FLAG2_GENERAL);
    let supply = mk(&mut sim, crate::ai_load::uflags2::SUPPLY_OR_HERO);
    let plain = mk(&mut sim, 0);
    let flight = |sim: &mut Sim, ty: usize, foe_ty: usize, y: i32, ambush: bool| {
        let me = put(sim, 0, ty, Pos::new(0x4000, y));
        sim.units[me].ambushing = ambush.then_some(i64::MAX);
        let foe = put(sim, 1, foe_ty, Pos::new(0x4300, y));
        sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        match sim.units[me].orders.front().map(|o| o.body) {
            Some(crate::orders::Body::Move(m)) => {
                assert_eq!(m.kind, crate::orders::MoveKind::FleeTo);
                Some(0x4000 - m.dest.x)
            }
            _ => None,
        }
    };
    let near = |d: Option<i32>, ring: i32| d.is_some_and(|d| (d - ring).abs() <= 0x60);
    let d = flight(&mut sim, hero, melee, 0x2000, false);
    assert!(near(d, 0x300), "a hero struck by a melee unit: {d:?}");
    let d = flight(&mut sim, hero, archer, 0x3000, false);
    assert!(near(d, 0x180), "a hero struck by a unit that shoots: {d:?}");
    let d = flight(&mut sim, supply, melee, 0x4000, false);
    assert!(
        near(d, 0x300),
        "a supply unit struck by a melee unit: {d:?}"
    );
    let d = flight(&mut sim, plain, melee, 0x5000, false);
    assert!(near(d, 0x600), "anything else flees on 0x600: {d:?}");
    let d = flight(&mut sim, hero, melee, 0x6000, true);
    assert_eq!(d, None, "a hero in an Ambush does not flee");
}

/// **A hit civilian runs away from whoever hit it** —
/// `Unit::target_opportunity`'s flee arm, `docs/COMBAT.md` §34. The
/// sweep's base bearing is `find_angle(me − attacker)` and its ring
/// is `0x600`, so the spot is on the far side of the victim from the
/// attacker, and the order goes in at `QUEUE_FIRST` as a `FLEE_TO`.
///
/// **Made to fail on purpose**, both ways that matter. With the arm
/// removed the citizen **retaliates** — the `obj_masks & CIVILIAN`
/// return that used to stand in its place is gone with it, so the
/// failure is an `AttackOrder` rather than nothing, which is the
/// shape run100's `0/5` had until item 464. And with the bearing
/// taken from the attacker's side (`attacker − me`) the spot lands
/// *past* the attacker at `(17928, 16392)`, which the `x` assertion
/// below refuses. `QUEUE_FIRST` is the listing's (`60084a` pushes
/// `FLEE_TO` and `600856` calls `add_move_order`), not a diff's.
///
/// run100's own measurement is Great Lakes 10233: the human's citizen
/// `0/5` at `(2232, 31224)` is struck by `1/27` at `(4584, 29784)`
/// and the dump's block 10234 carries `FLEETOORDER x 792 y 31800` —
/// 1,551 units away, on the `0x600` ring, and this crate now lands on
/// that pair exactly.
#[test]
fn a_hit_citizen_flees_away_from_whoever_hit_it() {
    let (mut sim, soldier) = at_war();
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 40,
        // The gate's own pair: `obj_masks & CIVILIAN` with no range,
        // and an `attack` column that is **not** zero — a citizen's
        // is 40, which is why the attack half of the disjunction is
        // not what carries this.
        combat: Profile {
            attack: 40,
            max_range: 0,
            uber_size: 1,
            obj_masks: mask::CIVILIAN,
            ..Profile::default()
        },
        worker: crate::orders::Worker::Citizen,
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, citizen, Pos::new(0x4000, 0x4000));
    let foe = put(&mut sim, 1, soldier, Pos::new(0x4600, 0x4000));
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Unit(me),
        crate::movement::Angle(0),
        false,
        1,
        false,
        false,
        10,
    );
    let front = sim.units[me].orders.front().copied().expect("an order");
    let crate::orders::Body::Move(m) = front.body else {
        panic!("the hit citizen did not flee: {front:?}");
    };
    assert_eq!(m.kind, crate::orders::MoveKind::FleeTo);
    assert!(
        m.dest.x < 0x4000,
        "the flight is away from the attacker, not past it: {:?}",
        m.dest
    );
    assert_eq!(
        sim.units[me].combat.target, None,
        "a fleeing civilian takes no target"
    );
    // And it does not answer a second hit while it runs —
    // `UnitData::is_fleeing@0046efa0`, the test at `6002cf`.
    let before = sim.units[me].orders.len();
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Unit(me),
        crate::movement::Angle(0),
        false,
        1,
        false,
        false,
        10,
    );
    assert_eq!(
        sim.units[me].orders.len(),
        before,
        "a unit already fleeing answers the hit again"
    );
}

/// **A hit group member of a non-combat type neither flees nor
/// retaliates** (item 1403, `docs/GOLDEN.md` §58):
/// `Unit::target_opportunity@005fffc0`'s group arm hands the hit to
/// `Group::target_opportunity@007107d0` and returns, and that
/// function's 15-frame cooldown (`+0x38`) is set. The golden record's
/// Scout `0/7`, walked by `@move`, takes its fourth wound on 912 with
/// no order.
///
/// Made to fail first with the arm removed: the same unit fled.
#[test]
fn a_hit_group_member_of_a_non_combat_type_keeps_its_orders() {
    let (mut sim, soldier) = at_war();
    let scout = sim.add_unit_type(crate::UnitType {
        hits: 40,
        combat: Profile {
            attack: 0,
            max_range: 0,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, scout, Pos::new(0x4000, 0x4000));
    let foe = put(&mut sim, 1, soldier, Pos::new(0x4600, 0x4000));
    sim.orders.pushed.push(crate::group::Pushed {
        who: 0,
        list: vec![me],
        state: crate::group::GroupState {
            pool: Some(2),
            ..crate::group::GroupState::default()
        },
        builds: Vec::new(),
    });
    sim.units[me].group_ptr = Some(2);
    sim.frame = 912;
    sim.target_opportunity(me, Obj::Unit(foe), 912);
    assert!(sim.units[me].orders.is_empty(), "a group member fled");
    assert_eq!(sim.orders.pushed[0].state.opportunity, 912);
    // Without the seat, the same hit is the flee arm's.
    sim.units[me].group_ptr = None;
    sim.target_opportunity(me, Obj::Unit(foe), 913);
    assert_eq!(sim.units[me].orders.len(), 1, "ungrouped, it flees");
}

/// A group of a non-combat Scout (`0/0`, the one that is hit) and an
/// idle combat-role Hoplite (`0/1`), with a near and a far foe of the
/// other side — `Group::target_opportunity`'s three arms
/// (`docs/GOLDEN.md` §61).
fn group_arms_fixture() -> (Sim, usize, usize, usize, usize) {
    let (mut sim, _) = at_war();
    let scout = sim.add_unit_type(crate::UnitType {
        hits: 40,
        combat: Profile {
            attack: 0,
            max_range: 0,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let hoplite = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 0,
            uber_size: 1,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let s = put(&mut sim, 0, scout, Pos::new(0x4000, 0x4000));
    let h = put(&mut sim, 0, hoplite, Pos::new(0x4100, 0x4000));
    let near = put(&mut sim, 1, hoplite, Pos::new(0x4000 + 0xc0 * 5, 0x4000));
    let far = put(&mut sim, 1, hoplite, Pos::new(0x4000 + 0xc0 * 9, 0x4000));
    sim.orders.pushed.push(crate::group::Pushed {
        who: 0,
        list: vec![s, h],
        state: crate::group::GroupState {
            pool: Some(2),
            ..crate::group::GroupState::default()
        },
        builds: Vec::new(),
    });
    sim.units[s].group_ptr = Some(2);
    sim.units[h].group_ptr = Some(2);
    (sim, s, h, near, far)
}

/// **The cooldown refuses a second hit inside fifteen frames**
/// (`Group::target_opportunity@007107d0`, `0xe < frame − +0x38`;
/// run577's group 1 prints `think_frame` 699, 724 and 750 while the
/// Scout is wounded on 700, 705, 725, 732 and 751). Made to fail with
/// the gate dropped: the second hit moved the stamp to 920 and the
/// Hoplite answered it.
#[test]
fn a_group_answers_a_hit_once_in_fifteen_frames() {
    let (mut sim, s, h, _near, far) = group_arms_fixture();
    sim.frame = 912;
    sim.target_opportunity(s, Obj::Unit(far), 912);
    assert_eq!(sim.orders.pushed[0].state.opportunity, 912);
    assert!(!sim.units[h].orders.is_empty(), "the first hit is answered");
    sim.units[h].orders.clear();
    sim.units[h].combat.target = None;
    sim.frame = 926;
    sim.target_opportunity(s, Obj::Unit(far), 926);
    assert_eq!(
        sim.orders.pushed[0].state.opportunity, 912,
        "inside the window"
    );
    assert!(sim.units[h].orders.is_empty(), "the gate refused the hit");
    sim.frame = 927;
    sim.target_opportunity(s, Obj::Unit(far), 927);
    assert_eq!(
        sim.orders.pushed[0].state.opportunity, 927,
        "past the window"
    );
    assert!(!sim.units[h].orders.is_empty());
}

/// **A combat-role captain with no action order runs its own
/// `find_melee_target`** (`710950`..`710989`, `docs/GOLDEN.md` §61):
/// the hit comes from the far foe and the Hoplite takes the **near**
/// one, by the search's ranking, where `Unit::target_opportunity` would
/// have retaliated on the attacker. With an `ATTACK_TO` action in its
/// stack the same Hoplite takes the attacker. No walk reaches the first
/// arm: run577's `0/7` always holds the `ATTACK_TO` of its `@amove`.
#[test]
fn an_idle_group_captain_searches_for_itself_and_a_busy_one_retaliates() {
    let (mut sim, s, h, near, far) = group_arms_fixture();
    sim.frame = 912;
    sim.target_opportunity(s, Obj::Unit(far), 912);
    assert_eq!(
        sim.units[h].combat.target,
        Some(Obj::Unit(near)),
        "idle: its own search"
    );
    let (mut sim, s, h, _near, far) = group_arms_fixture();
    sim.add_move_order(
        h,
        Pos::new(0x4000 + 0xc0 * 20, 0x4000),
        crate::orders::MoveKind::AttackTo,
        crate::orders::QueuePos::New,
        true,
    );
    sim.frame = 912;
    sim.target_opportunity(s, Obj::Unit(far), 912);
    assert_eq!(
        sim.units[h].combat.target,
        Some(Obj::Unit(far)),
        "busy: Unit::target_opportunity"
    );
}

/// **The asker's own squad captain takes `Unit::target_opportunity`
/// whatever its orders** (`71092e`..`710946`, `param_4 == 0`): a Scout
/// that follows the idle Hoplite is hit by the far foe, and the Hoplite
/// — idle, so otherwise the search's arm — retaliates on the attacker.
#[test]
fn the_asker_s_own_captain_retaliates_on_the_attacker() {
    let (mut sim, s, h, _near, far) = group_arms_fixture();
    sim.units[s].captain = false;
    sim.units[s].o_up = Some(h);
    sim.frame = 912;
    sim.target_opportunity(s, Obj::Unit(far), 912);
    assert_eq!(sim.units[h].combat.target, Some(Obj::Unit(far)));
}

/// **`Unit::think`'s step 3 needs the military bit as well as the
/// attack column** — `think@005f6e40:150`'s second arm is
/// `type->attack != 0 && (type->role & 0x10000) != 0`, and until item
/// 464 this crate's target search read only the first half. A
/// citizen's `attack` is 40, so every idle citizen in run100 ran
/// `find_melee_target` and one of them — the human's `0/5` on Great
/// Lakes 10233 — took an `ATTACKORDER` where the original was still
/// standing still.
///
/// [`Sim::think_attack_join_army`] has carried the same pair since
/// item 350 (`docs/ARMY.md` §4.2's own test says so in as many
/// words); this is the other half of the arm it gates.
///
/// **Made to fail on purpose**: with `combat_role` dropped from the
/// gate the citizen below takes the same order the soldier does.
#[test]
fn an_armed_citizen_does_not_take_an_attack_order_on_its_idle_frame() {
    let armed_citizen = |military: bool| {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 40,
            combat: Profile {
                attack: 40,
                max_range: 0,
                uber_size: 1,
                obj_masks: mask::CIVILIAN,
                combat_role: military,
                ..Profile::default()
            },
            worker: crate::orders::Worker::Citizen,
            ..crate::UnitType::default()
        });
        let foe_ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 15,
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        put(&mut sim, 1, foe_ty, Pos::new(0x4060, 0x4000));
        sim.tick();
        sim.units[me].combat.target
    };
    assert_eq!(
        armed_citizen(false),
        None,
        "a type with an attack column and no `role & 0x10000` entered \
             `think_attack`"
    );
    assert!(
        armed_citizen(true).is_some(),
        "the same type with the military bit must still search"
    );
}

/// **A human's packed siege engine never searches**
/// (`Unit::think_attack@005f5a80`'s packed arm, `docs/COMBAT.md` §51):
/// it holds no order until the first auto-attack frame with `idle ≥
/// 7`, and that order is the unpack, `0x28c`, at the head. A
/// computer's packed engine is not `manual`, so it falls through the
/// arm and searches while it waits out its 21; and a human's engine
/// whose packer stance is `PACKER_NEVER` (1) neither casts nor
/// searches. Golden chapter three's word 621 was the first case: this
/// crate put run145's packed catapult into `Unit::fight` on its birth
/// block.
///
/// **Made to fail on purpose**: with the arm's call taken out of
/// `think`, the human's catapult takes the attack order on its first
/// idle frame, which is 587's word.
/// **A computer's raiding ship leaves a building alone** (item 2001,
/// `compare_target`'s `0064e7a5`..`0064e81f`, `docs/AI.md` §203):
/// run810's packet has the Aegis Cruiser `1/113` answer 0 for the
/// finished `0/2000`. A human's ship, a ship not raiding, a SIEGE
/// ship and an unfinished building each weigh something; a SIEGE
/// ship that fires on the move does not.
/// **A Bark weighs ships by their line** (item 2205, `compare_target`'s
/// `0064f04f`..`0064f0fe`, `docs/GOLDEN.md` §80): run839's packet has
/// the Bark `0/7` value the Trireme `1/6` at 168 where this crate had
/// 66,888 — a four-hundredth — and a transport takes forty times. A
/// ship not of the Bark line, and a Bark against a land unit, weigh
/// all three alike.
#[test]
fn a_bark_weighs_a_trireme_at_a_four_hundredth_and_a_transport_at_forty() {
    let weigh = |bark: bool, line: u32, flags: u32, sea: bool| {
        let (mut sim, ty) = at_war();
        let mut at = sim.tables.unit_types[ty].clone();
        at.combat.domain = Domain::Sea;
        at.kind.domain = Domain::Sea;
        if bark {
            at.combat.roles |= crate::combat::role::IS_BARK;
        }
        let at = sim.add_unit_type(at);
        let mut tt = sim.tables.unit_types[ty].clone();
        let d = if sea { Domain::Sea } else { Domain::Land };
        tt.combat.domain = d;
        tt.kind.domain = d;
        tt.combat.roles |= line;
        tt.cols.unit_flags |= flags;
        let tt = sim.add_unit_type(tt);
        let me = put(&mut sim, 0, at, Pos::new(3000, 3000));
        let t = put(&mut sim, 1, tt, Pos::new(3300, 3000));
        sim.compare_target(Obj::Unit(me), Obj::Unit(t), true, false)
    };
    let (trireme, transport) = (crate::combat::role::IS_TRIREME, uflags::FLAG_TRANSPORT);
    let plain = weigh(true, 0, 0, true);
    assert!(
        weigh(true, trireme, 0, true) < plain / 50,
        "a Trireme to a Bark"
    );
    assert!(
        weigh(true, 0, transport, true) > plain * 10,
        "a transport to a Bark"
    );
    assert_eq!(
        weigh(false, trireme, 0, true),
        weigh(false, 0, 0, true),
        "not a Bark"
    );
    assert_eq!(
        weigh(false, 0, transport, true),
        weigh(false, 0, 0, true),
        "not a Bark"
    );
    assert_eq!(
        weigh(true, trireme, 0, false),
        weigh(true, 0, 0, false),
        "a land target"
    );
}

/// **The chase's retarget takes `fight`'s Bark predicate** (item 2205,
/// `do_move@005f7b30:262`..`271`, `docs/GOLDEN.md` §80): a unit with
/// `unit_masks2 & 4` takes a non-combat candidate or an `n` one, and
/// refuses a combat role; any other unit takes a combat role alone.
/// run839's packet: `do_move+0x6e6` → `change_target(1/8 → 1/7)`.
#[test]
fn a_bark_s_chase_retargets_to_a_fishing_boat_and_not_to_a_warship() {
    let admits = |bark: bool, combat: bool, n: bool| {
        let (mut sim, ty) = at_war();
        let mut tt = sim.tables.unit_types[ty].clone();
        tt.combat.combat_role = combat;
        if n {
            tt.cols.unit_flags |= 0x2000;
        }
        let tt = sim.add_unit_type(tt);
        let me = put(&mut sim, 0, ty, Pos::new(3000, 3000));
        if bark {
            sim.units[me].unit_masks2 |= crate::combat::umask2::MASK2_BARK;
        }
        let t = put(&mut sim, 1, tt, Pos::new(3300, 3000));
        sim.retarget_admits(me, Obj::Unit(t))
    };
    assert!(admits(true, false, false), "a Bark takes the fishing boat");
    assert!(!admits(true, true, false), "a Bark leaves a warship");
    assert!(admits(true, true, true), "a Bark takes an `n` combat unit");
    assert!(admits(false, true, false), "a warship takes a warship");
    assert!(
        !admits(false, false, false),
        "a warship leaves the fishing boat"
    );
}

/// **A computer's raiding ship weighs a peasant at nothing extra**
/// (item 3662, `docs/AI.md` §342; `Object::compare_target`'s sea raid
/// arm, `0064edd8`–`0064ef64`): a transport is `+900,000`, a peasant
/// and a plain unit stand — where the land
/// arm gives a computer's raider `+900,000` for the peasant and a tenth
/// for anything else. run1326's packet scores the Citizens `0/3` and
/// `0/10` at 512 and 596 for who=1's Galley `1/21` on 4567.
///
/// Made to fail by reading the ship as the land arm (the peasant's and
/// the plain unit's rows). The Fireship and fishing-boat lineages need
/// a tech tree, and the military role moves the threat terms above the
/// arm, so neither is asserted here.
#[test]
fn a_computer_s_raiding_ship_weighs_a_peasant_at_nothing_extra() {
    let weigh = |sea: bool, raid: bool, kind: u8| {
        let (mut sim, ty) = at_war();
        sim.cities_state.nation[0].human = false;
        let mut at = sim.tables.unit_types[ty].clone();
        if sea {
            at.combat.domain = Domain::Sea;
            at.kind.domain = Domain::Sea;
        }
        let at = sim.add_unit_type(at);
        let mut tt = sim.tables.unit_types[ty].clone();
        tt.combat.cost = 5000;
        tt.combat.attack = 0;
        tt.combat.max_range = 0;
        match kind {
            1 => tt.worker = crate::orders::Worker::Citizen,
            2 => tt.cols.unit_flags |= crate::ai_load::uflags::FLAG_TRANSPORT,
            _ => {}
        }
        let tt = sim.add_unit_type(tt);
        let me = put(&mut sim, 0, at, Pos::new(3000, 3000));
        if raid {
            sim.units[me].combat.stance = Stance::Raid;
        }
        let t = put(&mut sim, 1, tt, Pos::new(3300, 3000));
        sim.compare_target(Obj::Unit(me), Obj::Unit(t), false, false)
    };
    // The base is 40,000 here; the tail is `(v + 99) / 100`.
    let plain = weigh(true, true, 0);
    assert_eq!(plain, 400, "a plain unit stands: no tenth");
    assert_eq!(weigh(true, true, 1), 400, "the peasant stands");
    assert_eq!(weigh(true, true, 2), 9400, "a transport: + 900,000");
    assert_eq!(weigh(false, true, 1), 9400, "the land arm's peasant");
    assert_eq!(weigh(false, true, 0), 40, "the land arm's tenth");
}

#[test]
fn a_computer_s_raiding_ship_weighs_a_finished_building_at_zero() {
    let weigh_at = |human: bool,
                    raid: bool,
                    siege: bool,
                    on_move: bool,
                    active: bool,
                    ident: Option<crate::build::Ident>| {
        let (mut sim, ty) = at_war();
        sim.cities_state.nation[0].human = human;
        let mut at = sim.tables.unit_types[ty].clone();
        at.combat.domain = Domain::Sea;
        at.kind.domain = Domain::Sea;
        if siege {
            at.combat.obj_masks |= crate::combat::mask::SIEGE;
        }
        if on_move {
            at.cols.unit_flags |= crate::ai_load::uflags::FIRES_ON_MOVE;
        }
        let at = sim.add_unit_type(at);
        let b = sim.add_building(1, Pos::new(3000, 3000), 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = active;
        sim.buildings[b].health = 1200;
        sim.buildings[b].combat = Some(Profile {
            cost: 200,
            x_size: 2,
            y_size: 2,
            ..Profile::default()
        });
        if let Some(ident) = ident {
            let bt = sim.add_build_type(crate::build::BuildType {
                ident,
                x_size: 2,
                y_size: 2,
                ..crate::build::BuildType::default()
            });
            sim.buildings[b].ty = Some(bt);
        }
        let me = put(&mut sim, 0, at, Pos::new(3600, 3000));
        if raid {
            sim.units[me].combat.stance = Stance::Raid;
        }
        sim.compare_target(Obj::Unit(me), Obj::Building(b), true, false)
    };
    let dock = Some(crate::build::Ident::Dock);
    let weigh =
        |human, raid, siege, on_move, active| weigh_at(human, raid, siege, on_move, active, dock);
    assert_eq!(weigh(false, true, false, false, true), 0, "the cruiser");
    assert_eq!(weigh(false, true, true, true, true), 0, "fires on the move");
    assert!(weigh(true, true, false, false, true) > 0, "a human's ship");
    assert!(weigh(false, false, false, false, true) > 0, "not raiding");
    assert!(weigh(false, true, true, false, true) > 0, "a SIEGE ship");
    assert!(weigh(false, true, false, false, false) > 0, "unfinished");
    // **Item 3642: the second test is `is_dock`.** run1303's raiding
    // Frigate `1/29` takes the human's Farm `0/2004`; a ship that
    // fires on the move still leaves any building alone.
    let farm = Some(crate::build::Ident::Farm);
    assert!(
        weigh_at(false, true, false, false, true, farm) > 0,
        "the Frigate at a Farm"
    );
    assert_eq!(
        weigh_at(false, true, false, true, true, farm),
        0,
        "a Farm, firing on the move"
    );
}

/// **An idle captain off the search's phase takes its cached nearest**
/// (`think@005f6e40:104`–`135`, `docs/AI.md` §205): `near_o` a city
/// building at its ceiling (health 0, still standing), in range, idle
/// 6, on a frame that passes the one-in-sixteen gate and is not the
/// search's one in thirty-two — the attack is added, the
/// could-not-reach bit cleared. Not on the search's own frame, not at
/// `HOLD_FIRE`, not out of range, and not when the slot is gone.
#[test]
fn an_idle_captain_off_phase_takes_its_cached_nearest() {
    let think = |frame: i64, stance: Stance, far: bool, gone: bool| {
        let (mut sim, ty) = at_war();
        let b = sim.add_building(1, Pos::new(3000, 3000), 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = true;
        sim.buildings[b].health = 0;
        sim.buildings[b].alive = !gone;
        sim.buildings[b].combat = Some(Profile {
            x_size: 2,
            y_size: 2,
            ..Profile::default()
        });
        let at = if far { 9000 } else { 3300 };
        let me = put(&mut sim, 0, ty, Pos::new(at, 3000));
        sim.units[me].captain = true;
        sim.units[me].idle = 6;
        sim.units[me].cant_reach = true;
        sim.units[me].combat.stance = stance;
        sim.units[me].near = Some(Obj::Building(b));
        sim.think(me, frame - i64::from(sim.units[me].index));
        (sim.order_type(me), sim.units[me].cant_reach)
    };
    let attack = crate::orders::index::ATTACK;
    let (ty, cant_reach) = think(16, Stance::Aggressive, false, false);
    assert_eq!(ty, attack, "off phase, in range: the cached nearest");
    assert!(!cant_reach, "the could-not-reach bit is cleared");
    assert_ne!(
        think(32, Stance::HoldFire, false, false).0,
        attack,
        "the search's frame, holding fire"
    );
    assert_ne!(
        think(16, Stance::HoldFire, false, false).0,
        attack,
        "holding fire"
    );
    assert_ne!(
        think(16, Stance::Aggressive, true, false).0,
        attack,
        "out of range"
    );
    assert_ne!(
        think(16, Stance::Aggressive, false, true).0,
        attack,
        "the slot is gone"
    );
}

/// **A hit on a building spills onto the enemy hands repairing it**
/// (`Object::do_damage@0064a480`'s tail, item 1182, `docs/GOLDEN.md`
/// §50): an eighth for a sea attacker that is not siege and a land one,
/// a quarter for a land siege type, nothing for a sea siege type (the
/// `is_siege` arm at `64c3b3`) or an aircraft, and nothing onto a
/// citizen of the building's side that is not at work on it. Made to
/// fail with the sea arm's `is_siege` test dropped, and with the
/// quarter read as an eighth.
#[test]
fn a_building_s_hit_spills_onto_its_repairers_by_the_attacker_s_domain() {
    use crate::orders::{Body, Order};
    for (domain, siege, repairing, share) in [
        (Domain::Sea, false, true, Some(8)),
        (Domain::Sea, true, true, None),
        (Domain::Land, true, true, Some(4)),
        (Domain::Land, false, true, Some(8)),
        (Domain::Air, false, true, None),
        (Domain::Land, false, false, None),
    ] {
        let (mut sim, ty) = at_war();
        let mut at = sim.tables.unit_types[ty].clone();
        at.combat.domain = domain;
        at.combat.siege = siege;
        at.kind.domain = domain;
        let at = sim.add_unit_type(at);
        let b = sim.add_building(1, Pos::new(3000, 3000), 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = true;
        sim.buildings[b].health = 1200;
        sim.buildings[b].combat = Some(Profile {
            x_size: 2,
            y_size: 2,
            ..Profile::default()
        });
        let hand = put(&mut sim, 1, ty, Pos::new(3300, 3000));
        if repairing {
            sim.units[hand].orders.push_back(Order {
                flags: 0,
                body: Body::Repair(b),
            });
        }
        let me = put(&mut sim, 0, at, Pos::new(3600, 3000));
        let count = 0x800;
        sim.do_damage(
            Obj::Unit(me),
            Obj::Building(b),
            Angle(0),
            false,
            count,
            false,
            false,
            700,
        );
        let spilt: Vec<_> = sim
            .combat
            .hits
            .iter()
            .filter(|h| h.target == Obj::Unit(hand))
            .copied()
            .collect();
        match share {
            None => assert!(spilt.is_empty(), "{domain:?} siege {siege}: a spill"),
            Some(k) => {
                assert_eq!(spilt.len(), 1, "{domain:?} siege {siege}: no spill");
                let ap = sim.profile(Obj::Unit(me));
                let want = combat::scale(
                    spilt[0].damage,
                    count / k,
                    true,
                    false,
                    ap.ammo_per_att,
                    ap.uber_size,
                );
                assert_eq!(spilt[0].dealt, want, "{domain:?} siege {siege}: the share");
            }
        }
    }
}

/// An unpacked packer (`Profile::packs`, the bit clear) standing at
/// `(0x4000, 0x4000)`, with the pack's row in the craft table at
/// `JOB_TIME` 80 — `craftrules.xml`'s `0x28b`.
fn unpacked_packer(block_radius: i32) -> (Sim, usize) {
    let (mut sim, _) = at_war();
    sim.orders.spells = vec![crate::orders::SpellType::default(); 55];
    let row = usize::try_from(crate::orders::spell::PACK - crate::orders::spell::FIRST).unwrap();
    sim.orders.spells[row].job_time = 80;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 40,
            max_range: 15,
            uber_size: 1,
            siege: true,
            packs: true,
            block_radius,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 1, ty, Pos::new(0x4000, 0x4000));
    sim.units[me].combat.packed = false;
    (sim, me)
}

/// **A packer mid-pack keeps its cast through a `QUEUE_NEW` clear**
/// (`Group::action_move_near@00704990:410`–`440`, item 3336; run967g's
/// Cannon `1/125` on 20218): the pack cast is lifted off, the rest is
/// cleared, and the cast goes back at the head. A packer whose head is
/// not a pack cast is left alone and answered false.
///
/// Made to fail by answering false always, with the Egyptian word at
/// 20235 (item 3336's M3).
#[test]
fn a_packer_mid_pack_keeps_its_cast_through_a_new_clear() {
    use crate::orders::{Body, MoveKind, QueuePos, spell};
    let (mut sim, me) = unpacked_packer(48);
    let far = Pos::new(0x4000 + 20 * 192, 0x4000);
    sim.add_move_order(me, far, MoveKind::AttackTo, QueuePos::New, true);
    assert!(
        !sim.clear_keeping_pack(me),
        "a march at the head is no pack"
    );
    assert_eq!(sim.units[me].orders.len(), 1, "and is left alone");
    let frame = sim.frame;
    sim.work(me, frame);
    assert_eq!(sim.units[me].orders.len(), 2, "the pack over the march");
    assert!(sim.clear_keeping_pack(me));
    assert_eq!(sim.units[me].orders.len(), 1, "the march is cleared");
    assert!(
        matches!(sim.units[me].orders.front().map(|o| o.body), Some(Body::Cast(c)) if c.spell == spell::PACK),
        "the pack is not at the head: {:?}",
        sim.units[me].orders
    );
    assert_eq!(sim.units[me].spell_time, 1, "the cast runs on");
}

/// **An unpacked packer told to march packs first** (`Unit::work@0060d180`'s
/// pack arm, `0060d600`, item 1370): the move stays under a `0x28b`
/// cast, and the bit goes up on the cast's eightieth frame with the
/// move at the head again. run544's Bombard `1/132` on 15868. Made to
/// fail with [`Sim::pack_before_move`] answering `Not`.
#[test]
fn an_unpacked_packer_told_to_march_packs_before_it_walks() {
    use crate::orders::{Body, MoveKind, QueuePos, spell};
    let (mut sim, me) = unpacked_packer(48);
    let far = Pos::new(0x4000 + 20 * 192, 0x4000);
    sim.add_move_order(me, far, MoveKind::AttackTo, QueuePos::New, true);
    let frame = sim.frame;
    sim.work(me, frame);
    assert!(
        matches!(sim.units[me].orders.front().map(|o| o.body), Some(Body::Cast(c)) if c.spell == spell::PACK),
        "no pack at the head: {:?}",
        sim.units[me].orders
    );
    assert_eq!(sim.units[me].orders.len(), 2, "the march stays under it");
    assert_eq!(sim.units[me].pos, Pos::new(0x4000, 0x4000), "it walked");
    assert_eq!(sim.units[me].spell_time, 1, "the cast's first frame ran");
    for _ in 1..80 {
        assert!(!sim.units[me].combat.packed, "packed before the 80th frame");
        let frame = sim.frame;
        sim.work(me, frame);
    }
    assert!(sim.units[me].combat.packed, "cast_pack never set the bit");
    assert!(
        matches!(sim.units[me].orders.front().map(|o| o.body), Some(Body::Move(m)) if m.kind == MoveKind::AttackTo),
        "the march is not back at the head: {:?}",
        sim.units[me].orders
    );
    // A packed packer is not the arm: the march runs.
    let frame = sim.frame;
    sim.work(me, frame);
    assert!(
        matches!(
            sim.units[me].orders.front().map(|o| o.body),
            Some(Body::Move(_))
        ),
        "a packed packer packed again"
    );
}

/// The pack arm's other two (`0060d45d`–`0060d5fa`): a point in the
/// unit's own 48-unit cell kills the order and `work` ends; a sole
/// action move inside the type's `block_radius` is re-aimed at the
/// unit's own position and nothing is cast. Each made to fail by
/// swapping the arm's answer.
#[test]
fn an_unpacked_packer_s_short_move_dies_or_holds() {
    use crate::orders::{Body, MoveKind, PackArm, QueuePos};
    let (mut sim, me) = unpacked_packer(48);
    sim.add_move_order(
        me,
        Pos::new(0x4000 + 20, 0x4000),
        MoveKind::MoveTo,
        QueuePos::New,
        false,
    );
    assert_eq!(sim.pack_before_move(me), PackArm::Killed);
    assert!(
        sim.units[me].orders.is_empty(),
        "the same cell: {:?}",
        sim.units[me].orders
    );

    let (mut sim, me) = unpacked_packer(192);
    let near = Pos::new(0x4000 + 100, 0x4000);
    sim.add_move_order(me, near, MoveKind::MoveTo, QueuePos::New, true);
    assert_eq!(sim.pack_before_move(me), PackArm::Held);
    let front = sim.units[me].orders.front().map(|o| o.body);
    assert!(
        matches!(front, Some(Body::Move(m)) if m.dest == Pos::new(0x4000, 0x4000)),
        "inside the block radius: {front:?}"
    );
    // Without the action bit the same short move is no hold: it packs.
    let (mut sim, me) = unpacked_packer(192);
    sim.add_move_order(me, near, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(sim.pack_before_move(me), PackArm::Packing);
}

/// A packed packer of lineage `tree` on its guard post, guarding a
/// standing unit of its own side from `(0, 372)` — the post the
/// guard's own arithmetic puts at (3768, 12072) for this target —
/// in `PACKER_AUTO` (stance 0), with the four pack and unpack rows in
/// the craft table at `craftrules.xml`'s own lengths.
fn packer_on_post(tree: Option<crate::tech::TypeId>) -> (Sim, usize) {
    let (mut sim, ty) = at_war();
    sim.orders.spells = vec![crate::orders::SpellType::default(); 55];
    for (spell, t) in [
        (crate::orders::spell::PACK, 80),
        (crate::orders::spell::UNPACK, 80),
        (crate::orders::spell::PACK_MACHINEGUN, 50),
        (crate::orders::spell::UNPACK_MACHINEGUN, 50),
    ] {
        let row = usize::try_from(spell - crate::orders::spell::FIRST).unwrap();
        sim.orders.spells[row].job_time = t;
    }
    let packer = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 8,
            uber_size: 1,
            packs: true,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    // Set after the type is added: `add_unit_type` reads a tree id
    // against the player's research, and this table has none.
    sim.tables.unit_types[packer].tree = tree;
    // `get_stance_type`'s column: a military type that packs is a
    // `STANCE_PACKER`.
    sim.tables.unit_types[packer].cols.role |= crate::ai_load::role::ROLE_MILITARY;
    let post = Pos::new(3768, 12072);
    let wagon = put(&mut sim, 1, ty, Pos::new(3456, 11904));
    let guard = put(&mut sim, 1, packer, post);
    sim.units[guard].combat.packed = true;
    sim.units[guard].stance = 0;
    sim.add_guard_order(guard, wagon, 0, 372, crate::orders::QueuePos::New);
    if let crate::orders::Body::Guard(g) = &mut sim.units[guard].orders[0].body {
        g.guard = post;
    }
    (sim, guard)
}

/// **A packed packer on its post unpacks** (`Unit::do_guard@005e5c70:
/// 298`–`:316`, item 1608, `docs/AI.md` §170): the frame the guard's
/// `idle` reaches `0x1e` for the machine gun's lineage (`is(0x7b, 0)`)
/// or `0x46` for any other, the unpack goes on at the head with the
/// guard under it — re-aimed at `0x28e` for the machine gun — and not a
/// frame before. Unpacked, or under any stance but `PACKER_AUTO`, it
/// never does. The coverage pair's Advanced Machine Gun `1/74` on 2868.
/// Made to fail with `guard_unpacks` answering false.
#[test]
fn a_packed_packer_on_its_post_unpacks_when_its_idle_runs_out() {
    use crate::orders::{Body, spell};
    let head = |sim: &Sim, u: usize| sim.units[u].orders.front().map(|o| o.body);
    for (tree, threshold, want) in [
        (Some(0x7b), 0x1e, spell::UNPACK_MACHINEGUN),
        (None, 0x46, spell::UNPACK),
    ] {
        for (idle, casts) in [(threshold - 2, false), (threshold - 1, true)] {
            let (mut sim, guard) = packer_on_post(tree);
            if let Some(Body::Guard(g)) = sim.units[guard].orders.front_mut().map(|o| &mut o.body) {
                g.idle = idle;
            }
            let f = off_phase(&sim, guard);
            sim.work(guard, f);
            let front = head(&sim, guard);
            assert_eq!(
                matches!(front, Some(Body::Cast(c)) if c.spell == want),
                casts,
                "{tree:?} at idle {idle}: {:?}",
                sim.units[guard].orders
            );
            if casts {
                assert!(
                    matches!(sim.units[guard].orders.get(1).map(|o| o.body),
                        Some(Body::Guard(g)) if g.idle == threshold),
                    "the guard stands under the cast, its idle counted"
                );
            }
        }
    }
    for (packed, stance) in [(false, 0), (true, 1)] {
        let (mut sim, guard) = packer_on_post(Some(0x7b));
        sim.units[guard].combat.packed = packed;
        sim.units[guard].stance = stance;
        if let Some(Body::Guard(g)) = sim.units[guard].orders.front_mut().map(|o| &mut o.body) {
            g.idle = 0x40;
        }
        let f = off_phase(&sim, guard);
        sim.work(guard, f);
        assert!(
            matches!(head(&sim, guard), Some(Body::Guard(_))),
            "packed {packed} stance {stance}: {:?}",
            sim.units[guard].orders
        );
    }
}

/// **`get_job_time`'s two type arms** (`00675a10`–`00675b16`, item
/// 1608): the siege pair's 80 halved for a Howitzer or a Katyusha and
/// quartered for an MLRS, each asked strictly; the machine gun's 50
/// halved for the Heavy Machine Gun's lineage. Every other caster, and
/// every other craft, takes the row. And the clock reads it: an MLRS's
/// unpack lands on its twentieth `do_cast` frame — the coverage pair's
/// `1/65`, cast on 2879 and unpacked on block 2900. Made to fail with
/// `do_cast` reading `spell_job_time`.
#[test]
fn get_job_time_shortens_the_pack_and_unpack_by_type() {
    use crate::orders::spell;
    for (tree, s, want) in [
        (Some(0x111), spell::UNPACK, 20),
        (Some(0x111), spell::PACK, 20),
        (Some(0x10f), spell::UNPACK, 40),
        (Some(0x116), spell::PACK, 40),
        (Some(0x112), spell::UNPACK, 80),
        (None, spell::UNPACK, 80),
        (Some(0x7d), spell::UNPACK_MACHINEGUN, 25),
        (Some(0x7d), spell::PACK_MACHINEGUN, 25),
        (Some(0x7b), spell::UNPACK_MACHINEGUN, 50),
        (Some(0x111), spell::UNPACK_MACHINEGUN, 50),
        (Some(0x7d), spell::UNPACK, 80),
    ] {
        let (sim, u) = packer_on_post(tree);
        assert_eq!(sim.cast_job_time(u, s), want, "{tree:?} casting {s:#x}");
    }
    let (mut sim, u) = packer_on_post(Some(0x111));
    sim.units[u].orders.clear();
    sim.add_cast_order(u, spell::UNPACK);
    for n in 1..=20 {
        assert!(sim.units[u].combat.packed, "unpacked before frame {n}");
        let Some(crate::orders::Body::Cast(c)) = sim.units[u].orders.front().map(|o| o.body) else {
            panic!("the cast left on frame {n}");
        };
        sim.do_cast(u, c);
    }
    assert!(!sim.units[u].combat.packed, "the twentieth frame unpacks");
}

/// **A packed packer in range of its attack unpacks first**
/// (`Unit::fight@005fd4d0:512`–`543`, item 1182): a human's packed
/// catapult holding an attack on a building in range is given the
/// unpack at the head and strikes nothing, where it used to fall through
/// to the strike. Made to fail with [`Sim::packed_unpacks`] answering
/// false.
#[test]
fn a_packed_packer_in_range_unpacks_before_it_strikes() {
    use crate::orders::{Body, spell};
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 40,
            max_range: 15,
            uber_size: 1,
            siege: true,
            packs: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
    assert!(sim.units[me].combat.packed);
    let b = sim.add_building(1, Pos::new(0x4000 + 6 * 192, 0x4000), 0);
    sim.buildings[b].started = true;
    sim.buildings[b].active = true;
    sim.buildings[b].health = 1200;
    sim.buildings[b].combat = Some(Profile::default());
    sim.add_attack_order(
        me,
        Obj::Building(b),
        crate::orders::QueuePos::New,
        true,
        true,
    );
    let frame = sim.frame;
    sim.work(me, frame);
    assert!(
        matches!(sim.units[me].orders.front().map(|o| o.body), Some(Body::Cast(c)) if c.spell == spell::UNPACK),
        "no unpack at the head: {:?}",
        sim.units[me].orders
    );
    assert!(sim.combat.hits.is_empty(), "a packed engine struck");
}

#[test]
fn a_human_s_packed_siege_engine_unpacks_before_it_searches() {
    use crate::orders::{Body, spell};
    let catapult = |human: bool, stance: u8| {
        let (mut sim, _) = at_war();
        sim.cities_state.nation[0].human = human;
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 15,
                uber_size: 1,
                obj_masks: mask::SIEGE,
                siege: true,
                packs: true,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        // The loader writes `role & 0x10000` twice: the profile's
        // `combat_role` and the column `get_stance_type` reads.
        sim.tables.unit_types[ty].cols.role |= crate::ai_load::role::ROLE_MILITARY;
        let foe_ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 15,
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        assert!(
            sim.units[me].combat.packed,
            "a type that packs is born packed"
        );
        sim.units[me].stance = stance;
        put(&mut sim, 1, foe_ty, Pos::new(0x4000 + 6 * 192, 0x4000));
        // Tick until the engine holds an order or 200 frames pass;
        // answer the `idle` it thought on and what it was given.
        for _ in 0..200 {
            let idle = sim.units[me].idle;
            let frame = sim.frame;
            sim.tick();
            if let Some(o) = sim.units[me].orders.front() {
                return Some((idle, frame, o.body, sim.units[me].combat.target));
            }
        }
        None
    };
    // The human's: nothing on its first idle frame, and its first
    // order is the unpack, on a thirty-two phase at `idle` 7 or more.
    let Some((idle, frame, body, target)) = catapult(true, 0) else {
        panic!("the human's packed catapult never unpacked");
    };
    assert!(
        matches!(body, Body::Cast(c) if c.spell == spell::UNPACK),
        "the human's first order is not the unpack: {body:?}"
    );
    assert_eq!(target, None, "a packed engine took a target");
    assert!(idle >= 6, "the unpack came before `idle` 7: {idle}");
    // `(frame + o) & 31 == 0` on the frame the think runs, and the
    // engine is `o` 0.
    assert_eq!(frame & 31, 0, "the unpack is on the auto-attack's phase");
    // The computer's searches while packed, on its first idle frame.
    let Some((_, _, body, target)) = catapult(false, 0) else {
        panic!("the computer's packed catapult took no order");
    };
    assert!(
        matches!(body, Body::Attack(_)) && target.is_some(),
        "a computer's packed engine is not `manual` and must search: {body:?}"
    );
    // `PACKER_NEVER`: the human's engine waits, orderless.
    assert_eq!(
        catapult(true, 1).map(|r| r.2),
        None,
        "a `PACKER_NEVER` engine cast or searched"
    );
}

/// **An unpacked siege engine shoots a unit by shooting the ground
/// under it** (`Unit::fight@005fd4d0`'s siege arm and
/// `Unit::do_attack_ground@005f1410`, `docs/COMBAT.md` §57). On the
/// frame the attack comes into range an `ATTACK_GROUND` goes on top of
/// it at the foe's own position and fires in the same frame
/// (`attack_unit` 1, the fired bit), the figure forgets its aim, the
/// attack beneath keeps `new_ord`, and the reload is one frame longer
/// than a strike's. On the ready frame the order dies and the attack
/// beneath, still in range, pushes the next.
///
/// **Made to fail on purpose**: with the arm returning `false`, the
/// engine strikes the unit directly — one order, `new_ord` cleared,
/// the plain reload.
#[test]
fn an_unpacked_siege_engine_fires_on_the_ground_under_its_target() {
    use crate::orders::{Body, flag, index};
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 40,
            max_range: 15,
            recharge: 30,
            uber_size: 1,
            obj_masks: mask::SIEGE,
            siege: true,
            packs: true,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let foe_ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, ty, Pos::new(0x4000 + 24, 0x4000 + 24));
    sim.units[me].combat.packed = false;
    let at = Pos::new(0x4000 + 6 * 192 + 24, 0x4000 + 24);
    let foe = put(&mut sim, 1, foe_ty, at);
    sim.order_attack(me, Obj::Unit(foe));
    sim.tick();
    let u = &sim.units[me];
    let kinds: Vec<u8> = u.orders.iter().map(|o| o.index()).collect();
    assert_eq!(kinds, vec![index::ATTACK_GROUND, index::ATTACK]);
    let head = u.orders[0];
    let Body::AttackGround(g) = head.body else {
        unreachable!()
    };
    assert_eq!((g.at, g.sea, g.attack_unit), (at, false, 1));
    assert!(head.has(flag::FIRED) && !head.has(flag::ACTION));
    let Body::Attack(a) = u.orders[1].body else {
        unreachable!()
    };
    assert!(a.in_range && a.ever_in_range && a.new_ord);
    let reload = sim.reload_frames(me);
    assert_eq!(u.combat.recharging, reload + 1, "the ground shot's reload");
    // Held through the reload, then gone on the ready frame, and the
    // attack beneath pushes the next at once.
    let mut pushes = 0;
    for _ in 0..=reload {
        sim.tick();
        let head = sim.units[me].orders.front().copied();
        if let Some(o) = head
            && let Body::AttackGround(g) = o.body
            && sim.units[me].combat.recharging == reload + 1
        {
            assert_eq!(g.attack_unit, 1);
            pushes += 1;
        }
    }
    assert_eq!(pushes, 1, "the ready frame re-entered work and fired again");
}

/// **A building is struck square to its side** (item 1040,
/// `docs/COMBAT.md` §70.6): `Unit::fight@005fd4d0`'s `5fe8a7`–`5feb4c`.
/// Great Lakes' Hoplites on the human's city face due south from the
/// row above its footprint, where the centre's bearing is 20° off.
/// West is asked first, then north, east and south; a covered tile that
/// is not blocked, or the unit's own tile covered, keeps the bearing.
///
/// Made to fail first with the arm answering `None`.
#[test]
fn a_building_is_struck_square_to_its_side() {
    let (mut sim, ty) = at_war();
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 3,
        y_size: 3,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let b = sim.add_building(0, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 1000;
    sim.buildings[b].health = 1000;
    sim.buildings[b].combat = Some(Profile::default());
    let corner = sim.tile_corner(bt, at);
    for t in sim.footprint(bt, corner) {
        sim.world.set_blocked_at(t, true);
    }
    let me = put(&mut sim, 1, ty, at);
    let tile = |x: i32, y: i32| Pos::new(x * 192 + 96, y * 192 + 96);
    let side = |sim: &mut Sim, p: Pos| {
        sim.units[me].pos = p;
        sim.building_side(me, Obj::Building(b)).map(|a| a.0 as u32)
    };
    let (cx, cy) = (corner.x, corner.y);
    assert_eq!(
        side(&mut sim, tile(cx, cy - 1)),
        Some(0x8000_0000),
        "above: south"
    );
    assert_eq!(
        side(&mut sim, tile(cx + 2, cy + 3)),
        Some(0),
        "below: north"
    );
    assert_eq!(
        side(&mut sim, tile(cx - 1, cy + 1)),
        Some(0x4000_0000),
        "left: east"
    );
    assert_eq!(
        side(&mut sim, tile(cx + 3, cy)),
        Some(0xc000_0000),
        "right: west"
    );
    assert_eq!(
        side(&mut sim, tile(cx - 1, cy - 1)),
        None,
        "a corner: the bearing"
    );
    assert_eq!(
        side(&mut sim, tile(cx + 1, cy + 1)),
        None,
        "inside: the bearing"
    );
    sim.world.set_blocked_at(Pos::new(cx, cy), false);
    assert_eq!(
        side(&mut sim, tile(cx, cy - 1)),
        None,
        "a covered tile that is not blocked"
    );
    let foe = put(&mut sim, 0, ty, at);
    sim.units[me].pos = tile(cx, cy - 1);
    assert_eq!(sim.building_side(me, Obj::Unit(foe)), None, "a unit target");
}

/// **The one-in-five retarget freezes the frame** (item 1040,
/// `docs/COMBAT.md` §70.7): `Unit::fight@005fd4d0`'s `005fdf68`–
/// `005fdfea`. A captain whose roll runs the re-search and finds
/// another target keeps its attack under the new target and carries
/// `unit_masks2 |= 0x10`, so its figures' clocks stand still for the
/// frame. Great Lakes' `1/24` on 4852: its Slinger's attack slot held
/// at 32 of 33 there, and wrapped to the idle here.
///
/// Made to fail first with the mark left off.
#[test]
fn the_one_in_five_retarget_freezes_the_frame() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 1, ty, Pos::new(0x4000, 0x4000));
    let far = put(&mut sim, 0, ty, Pos::new(0x4000 + 9 * 0xc0, 0x4000));
    let near = put(&mut sim, 0, ty, Pos::new(0x4000 + 2 * 0xc0, 0x4000));
    sim.add_attack_order(
        me,
        Obj::Unit(far),
        crate::orders::QueuePos::New,
        false,
        false,
    );
    assert_eq!(sim.find_melee_target(me, -1), Some(Obj::Unit(near)));
    let seed = (1u32..)
        .find(|&k| {
            let mut r = sim.rng;
            r.seed = k;
            r.roll() % 5 != 0
        })
        .unwrap();
    sim.rng.seed = seed;
    sim.units[me].unit_masks2 = 0;
    sim.work(me, 0);
    assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(near)));
    assert_ne!(
        sim.units[me].unit_masks2 & crate::combat::umask2::NOT_FIRING,
        0,
        "the retarget left the frame unfrozen"
    );
}

/// **The one-in-five re-search kills the attack first** (item 1099,
/// `docs/COMBAT.md` §80): `Unit::fight@005fd4d0:413` is
/// `find_new_target(this, &who, 0)`, so with the attack gone the head
/// is the attack-move again and the search runs under its `0x20010`,
/// which halves a building. run347's `1/24` on 5161: under the attack a
/// search names the human's city, under the attack-move the scout it
/// is already attacking, and the original strikes the scout.
///
/// Made to fail with the arm searching under the attack and
/// re-pointing it in place, as it did before: the target is the
/// building.
#[test]
fn the_one_in_five_re_search_runs_under_the_attack_move() {
    let (mut sim, ty) = at_war();
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let b = sim.add_building(0, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 800;
    sim.buildings[b].health = 800;
    // A tenth of the cost it was written with: the search here passes
    // `ai` (difficulty 0), and the class arm's `× 10` on a building
    // (item 3615) gives it back.
    sim.buildings[b].combat = Some(Profile {
        attack: 10,
        cost: 160,
        ..Profile::default()
    });
    let me = put(&mut sim, 1, ty, Pos::new(at.x + 3 * 0xc0, at.y));
    let scout = put(&mut sim, 0, ty, Pos::new(at.x + 4 * 0xc0, at.y + 0xc0));
    sim.add_move_order(
        me,
        Pos::new(at.x - 20 * 0xc0, at.y),
        crate::orders::MoveKind::AttackTo,
        crate::orders::QueuePos::New,
        false,
    );
    sim.add_attack_order(
        me,
        Obj::Unit(scout),
        crate::orders::QueuePos::First,
        false,
        false,
    );
    // Under the attack the search takes the building; under the
    // attack-move, the scout.
    assert_eq!(sim.melee_search_flags(me), 0);
    assert_eq!(
        sim.clone().find_melee_target(me, -1),
        Some(Obj::Building(b)),
        "the attack's own search"
    );
    let mut bare = sim.clone();
    bare.kill_current_order(me);
    assert_eq!(
        bare.find_melee_target(me, -1),
        Some(Obj::Unit(scout)),
        "the attack-move's search"
    );
    let seed = (1u32..)
        .find(|&k| {
            let mut r = sim.rng;
            r.seed = k;
            r.roll() % 5 != 0
        })
        .unwrap();
    sim.rng.seed = seed;
    sim.work(me, 0);
    assert_eq!(
        sim.units[me].combat.target,
        Some(Obj::Unit(scout)),
        "the re-search searched under the attack"
    );
}

/// **A captain's attack on a building re-searches every frame**
/// (`docs/COMBAT.md` §62, `Unit::fight@005fd4d0`'s `LAB_005fddf7`,
/// `005fdeb4` → `005fdf50` → `005fdeea`). Out of its search's reach the
/// attack dies where the unit stands, and nothing walks: run177's
/// Fighter on tick 632. In reach, the search re-finds the building and
/// the chase goes on under a fresh order.
///
/// Made to fail both ways: with the arm off, the far unit keeps its
/// attack and adds a chase; with the search's answer dropped, the near
/// one loses its attack.
#[test]
fn a_captains_attack_on_a_building_re_searches_every_frame() {
    let (mut sim, ty) = at_war();
    // A soldier (`role & MILITARY`): its combat stance is its own byte,
    // not `get_combat_stance`'s STAND_GROUND default (item 2265).
    sim.tables.unit_types[ty].combat.combat_role = true;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 3,
        y_size: 3,
        ..crate::build::BuildType::default()
    });
    let target_at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let b = sim.add_building(1, target_at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 1000;
    sim.buildings[b].health = 1000;
    sim.buildings[b].combat = Some(Profile::default());
    let run = |sim: &mut Sim, from: Pos| {
        let u = put(sim, 0, ty, from);
        sim.add_attack_order(
            u,
            Obj::Building(b),
            crate::orders::QueuePos::New,
            false,
            false,
        );
        let before = sim.find_melee_target(u, -1);
        assert!(sim.valid_target(Obj::Unit(u), Obj::Building(b)));
        sim.tick();
        (u, before)
    };
    // Far: sixteen tiles off, past a human's twelve-tile idle search
    // (`UNIT_RESPOND_RANGE × 0xc0`), as run175's Fighter at its attack
    // point is at about twelve.
    let mut far = sim.clone();
    let (u, before) = run(&mut far, Pos::new(target_at.x - 16 * 0xc0, target_at.y));
    assert_eq!(before, None, "the far unit's search reaches the building");
    assert!(
        far.units[u].orders.is_empty(),
        "the far unit kept an order: {:?}",
        far.units[u].orders
    );
    // Near: eight tiles off, out of range 4 and inside the search.
    let (u, before) = run(&mut sim, Pos::new(target_at.x - 8 * 0xc0, target_at.y));
    assert_eq!(
        before,
        Some(Obj::Building(b)),
        "the near unit's search misses"
    );
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(o.body, crate::orders::Body::Attack(_))),
        "the near unit lost its attack: {:?}",
        sim.units[u].orders
    );
    assert_eq!(sim.units[u].combat.target, Some(Obj::Building(b)));
}

/// **A candidate in another region is taken only in range**
/// (`Object::check_target@00649e00`'s head, `docs/COMBAT.md` §86, item
/// 1214). East Indies 8907's shape: the computer's Caravel, idle on
/// the sea, and the human's building inland. Across the regions the
/// building out of range is refused, and in range it is taken; a
/// computer's `SIEGE` ship ignores the regions. The head's other arm:
/// a `DEFENSIVE` unit off duty with an order must reach whatever it
/// takes, in its own region too.
///
/// Made to fail by the search not asking `check_target_reaches`: the
/// first assertion names the building.
#[test]
fn a_candidate_in_another_region_is_taken_only_in_range() {
    use crate::world::Terrain;
    let (mut sim, ty) = at_war();
    // A soldier (`role & MILITARY`): its combat stance is its own byte,
    // not `get_combat_stance`'s STAND_GROUND default (item 2265).
    sim.tables.unit_types[ty].combat.combat_role = true;
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let land = sim.world.add_region(Terrain::Land);
    let sea = sim.world.add_region(Terrain::Sea);
    for x in 0..60 {
        for y in 0..60 {
            let r = if x < 30 { land } else { sea };
            sim.world.set_region(crate::Cell::new(x, y), r);
        }
    }
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(29 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let site = sim.add_building(0, at, 0);
    sim.buildings[site].ty = Some(bt);
    sim.buildings[site].hits = 400;
    sim.buildings[site].health = 400;
    sim.buildings[site].combat = Some(Profile::default());
    // A ship: the domain in the type's kind, the type's combat and the
    // unit's kind.
    sim.tables.unit_types[ty].kind.domain = Domain::Sea;
    sim.tables.unit_types[ty].combat.domain = Domain::Sea;
    let far = put(&mut sim, 1, ty, Pos::new(at.x + 10 * 0xc0, at.y));
    assert_eq!(sim.units[far].kind.domain, Domain::Sea);
    assert!(!sim.is_in_range(Obj::Unit(far), Obj::Building(site)));
    assert_eq!(
        sim.clone().find_melee_target(far, -1),
        None,
        "the ship took a building in another region out of its range"
    );
    // In range, across the regions, it is taken.
    let mut near = sim.clone();
    near.units[far].pos = Pos::new(at.x + 3 * 0xc0, at.y);
    assert!(near.is_in_range(Obj::Unit(far), Obj::Building(site)));
    assert_eq!(near.find_melee_target(far, -1), Some(Obj::Building(site)));
    // A computer's siege ship does not ask the regions.
    let mut siege = sim.clone();
    siege.tables.unit_types[ty].combat.obj_masks |= mask::SIEGE;
    // A human's siege ship asks them (`unit_masks & 0x40000`, item
    // 1254: chapter forty-four's `0/6` against `1/2007`).
    let mut human = siege.clone();
    human.cities_state.nation[1].human = true;
    assert_eq!(
        human.find_melee_target(far, -1),
        None,
        "a human's siege ship took a building in another region out of its range"
    );
    assert_eq!(
        siege.find_melee_target(far, -1),
        Some(Obj::Building(site)),
        "a computer's siege ship asked the regions"
    );
    // One region: the same ship takes it out of range.
    let mut one = sim.clone();
    for x in 0..60 {
        for y in 0..60 {
            one.world.set_region(crate::Cell::new(x, y), land);
        }
    }
    assert_eq!(
        one.clone().find_melee_target(far, -1),
        Some(Obj::Building(site))
    );
    // Defensive, off duty and with an order: it must reach.
    let mut guarded = one.clone();
    guarded.units[far].combat.stance = Stance::Defensive;
    // The defensive radius is the range itself, so the search is
    // handed a wider one, as `find_melee_target`'s callers may.
    let wide = 12 * 0xc0;
    assert_eq!(
        guarded.clone().find_melee_target(far, wide),
        Some(Obj::Building(site)),
        "defensive with no order is the plain search"
    );
    guarded.add_move_order(
        far,
        Pos::new(at.x + 20 * 0xc0, at.y),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    assert_eq!(
        guarded.find_melee_target(far, wide),
        None,
        "a defensive unit with an order took what it cannot reach"
    );
}

/// **An attack-move's look passes over an unarmed building**
/// (`docs/COMBAT.md` §64, item 997). Great Lakes 4555's shape: an AI
/// soldier on an `ATTACK_TO` past the human's Woodcutter's Camp. Under
/// the attack-move's `0x20010` the camp is refused before it is
/// scored, and an armed building is taken; the same soldier with no
/// attack-move at its head searches with 0 and takes the camp. The
/// word itself is checked for each of `melee_search_flags`' arms.
///
/// Made to fail by `search_admits` answering true: the first
/// assertion names the camp.
#[test]
fn an_attack_move_passes_over_an_unarmed_building() {
    let (mut sim, ty) = at_war();
    // A soldier (`role & MILITARY`): its combat stance is its own byte,
    // not `get_combat_stance`'s STAND_GROUND default (item 2265).
    sim.tables.unit_types[ty].combat.combat_role = true;
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let camp = sim.add_building(0, at, 0);
    sim.buildings[camp].ty = Some(bt);
    sim.buildings[camp].hits = 800;
    sim.buildings[camp].health = 800;
    sim.buildings[camp].combat = Some(Profile::default());
    let me = put(&mut sim, 1, ty, Pos::new(at.x + 8 * 0xc0, at.y));
    let walk = |sim: &mut Sim| {
        sim.add_move_order(
            me,
            Pos::new(at.x - 20 * 0xc0, at.y),
            crate::orders::MoveKind::AttackTo,
            crate::orders::QueuePos::New,
            false,
        );
    };
    let mut moving = sim.clone();
    walk(&mut moving);
    assert_eq!(
        moving.melee_search_flags(me),
        search::ARMED | search::HALVE_NON_UNITS
    );
    assert_eq!(
        moving.find_melee_target(me, -1),
        None,
        "the attack-move took the unarmed camp"
    );
    // Idle, the same search takes it.
    assert_eq!(sim.melee_search_flags(me), 0);
    assert_eq!(sim.find_melee_target(me, -1), Some(Obj::Building(camp)));
    // An armed building is taken on the move.
    let mut armed = moving.clone();
    armed.buildings[camp].combat = Some(Profile {
        attack: 10,
        ..Profile::default()
    });
    assert_eq!(
        armed.find_melee_target(me, -1),
        Some(Obj::Building(camp)),
        "the attack-move refused an armed building"
    );
    // The word's other arms: a siege type (AI 0x20002, human 0), a
    // tank (0), and RAZE over any of them (0x20).
    let typed = |flags: u32, stance: Stance, human: bool| {
        let mut s = moving.clone();
        let mut t = s.tables.unit_types[ty].clone();
        t.cols.unit_flags = flags;
        let t = s.add_unit_type(t);
        s.units[me].ty = Some(t);
        s.units[me].combat.stance = stance;
        s.cities_state.nation[1].human = human;
        s.melee_search_flags(me)
    };
    assert_eq!(
        typed(uflags::FLAG_SIEGE, Stance::Aggressive, false),
        search::BUILDINGS | search::ARMED
    );
    assert_eq!(typed(uflags::FLAG_SIEGE, Stance::Aggressive, true), 0);
    assert_eq!(typed(uflags::TANK, Stance::Aggressive, false), 0);
    assert_eq!(
        typed(uflags::TANK, Stance::Raze, false),
        search::HALVE_NON_BUILDS
    );
}

/// **A ranged chase on a building stops at the edge of its reach**
/// (`do_move@005f7b30`, the listing `5f7f27`–`5f7faa`; `docs/COMBAT.md`
/// §65). The target's vslot `+0x1c` splits the kill: a unit target is
/// asked with `is_in_range`'s `0x90` margin when the attack is not
/// mandatory, a building through `is_in_range@00648d70`, whose sixth
/// argument is 0. run347's `1/26` stops on the human's city at
/// `attack_dist` 1128 against a reach of 1158 (Great Lakes 4593).
///
/// Four arms: a building inside the reach but not inside the margin
/// kills the move; a unit at the same distance does not; an AI siege
/// type is passed over at `5f7f34`; and a building past the reach
/// keeps the chase. Made to fail on purpose by asking the building
/// with the margin again: the first arm keeps its move.
#[test]
fn a_ranged_chase_on_a_building_stops_at_the_edge_of_its_reach() {
    let (mut sim, ty) = at_war();
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let city = sim.add_building(0, at, 0);
    sim.buildings[city].ty = Some(bt);
    sim.buildings[city].hits = 800;
    sim.buildings[city].health = 800;
    sim.buildings[city].combat = Some(Profile::default());
    let foe = put(&mut sim, 0, ty, at);
    let me = put(&mut sim, 1, ty, at);
    // The reach is `4 × 0xc0 + 6`; find a spot east of the target
    // inside it but not inside the margin, for each kind of target.
    let reach = sim.max_range_of(Obj::Unit(me)) * 0xc0 + 6;
    let spot = |sim: &mut Sim, t: Obj, lo: i32, hi: i32| {
        (0..0x1000)
            .map(|dx| Pos::new(at.x + dx, at.y))
            .find(|&p| {
                sim.units[me].pos = p;
                let d = sim.attack_dist(Obj::Unit(me), t);
                (lo..=hi).contains(&d)
            })
            .expect("a spot at that distance")
    };
    let inside = spot(&mut sim, Obj::Building(city), reach - 0x8f, reach);
    let past = spot(&mut sim, Obj::Building(city), reach + 1, reach + 0x40);
    let unit_inside = spot(&mut sim, Obj::Unit(foe), reach - 0x8f, reach);
    let chase = |sim: &Sim, from: Pos, t: Obj, siege: bool| {
        let mut s = sim.clone();
        if siege {
            let mut ut = s.tables.unit_types[ty].clone();
            ut.cols.unit_flags = uflags::FLAG_SIEGE;
            let ut = s.add_unit_type(ut);
            s.units[me].ty = Some(ut);
        }
        s.units[me].pos = from;
        s.add_attack_order(me, t, crate::orders::QueuePos::First, false, true);
        s.add_move_order(
            me,
            Pos::new(at.x - 0x400, at.y),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        assert!(!s.units[me].combat.mandatory);
        s.work(me, 0);
        s.units[me].orders.iter().any(crate::orders::Order::is_move)
    };
    assert!(
        !chase(&sim, inside, Obj::Building(city), false),
        "a building inside the reach kept the chase"
    );
    assert!(
        chase(&sim, unit_inside, Obj::Unit(foe), false),
        "a unit inside the reach but not the margin ended the chase"
    );
    assert!(
        chase(&sim, inside, Obj::Building(city), true),
        "an AI siege type stopped on the building"
    );
    assert!(
        chase(&sim, past, Obj::Building(city), false),
        "a building past the reach ended the chase"
    );
}

/// **A chase on a building holds while another unit stands in the
/// chaser's block** (`do_move@005f7b30`, `5f7f73`–`5f7f9d`: the
/// building arm's last conjunct is `Objects::find_collision(my spot,
/// o, who, 1) == 0`; `docs/COMBAT.md` §65.8). The fifth argument set
/// skips the land shortcut, so the query is the nine world cells'
/// chains at current positions, Chebyshev in unit cells against the
/// two `coll_size`s. run426's Longbowman `1/29` reaches the human's
/// city at `attack_dist` 2284 with its squad-mate `1/30` two unit
/// cells off, walks one more step, and ends the chase there (Great
/// Sahara 15586).
///
/// Three arms: a mate two cells off holds the chase; three cells off
/// ends it; gaia two cells off is not asked. Made to fail on purpose
/// by dropping the conjunct: the first arm ends its chase.
#[test]
fn a_unit_in_the_chaser_s_block_holds_its_chase_on_a_building() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            block_radius: 48,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let city = sim.add_building(0, at, 0);
    sim.buildings[city].ty = Some(bt);
    sim.buildings[city].hits = 800;
    sim.buildings[city].health = 800;
    sim.buildings[city].combat = Some(Profile::default());
    let me = put(&mut sim, 1, ty, at);
    // `put` chains a unit where it is born; the chaser is taken off
    // the chain while its spot is sought, and put back there.
    sim.chain_remove(me);
    let reach = sim.max_range_of(Obj::Unit(me)) * 0xc0 + 6;
    // A unit-cell centre east of the target, inside the reach.
    let inside = (0..0x1000)
        .map(|dx| Pos::new(at.x + dx * 48 + 24, at.y))
        .find(|&p| {
            sim.units[me].pos = p;
            let d = sim.attack_dist(Obj::Unit(me), Obj::Building(city));
            (reach - 0x8f..=reach).contains(&d)
        })
        .expect("a spot inside the reach");
    let chase = |sim: &Sim, who: Player, cells: i32| {
        let mut s = sim.clone();
        s.units[me].pos = inside;
        s.chain_add(me);
        put(&mut s, who, ty, Pos::new(inside.x, inside.y + cells * 48));
        s.add_attack_order(
            me,
            Obj::Building(city),
            crate::orders::QueuePos::First,
            false,
            true,
        );
        s.add_move_order(
            me,
            Pos::new(at.x - 0x400, at.y),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        s.work(me, 0);
        s.units[me].orders.iter().any(crate::orders::Order::is_move)
    };
    assert!(
        chase(&sim, 1, 2),
        "a mate two unit cells off let the chase end"
    );
    assert!(
        !chase(&sim, 1, 3),
        "a mate three unit cells off held the chase"
    );
    assert!(!chase(&sim, 8, 2), "gaia in the block held the chase");
}

/// **A chase on a unit holds while another unit stands in the chaser's
/// block, too** (`do_move@005f7b30`, `5f7f11`–`5f7f2c`: the unit arm's
/// kill asks `Objects::find_collision(my spot, o, who, 1) == 0` after
/// `is_in_range`, as the building arm does; `docs/COMBAT.md` §65.8).
/// run976's Slinger `1/22` reaches the human's `0/5` at `attack_dist`
/// 986 with a mate in its block, walks on, and the Great Lakes Russian
/// word goes 6563 → 7303 (item 3342).
///
/// Three arms, as the building's: a mate two cells off holds the
/// chase; three cells off ends it; gaia two cells off is not asked.
/// Made to fail on purpose by dropping the conjunct: the first arm
/// ends its chase.
#[test]
fn a_unit_in_the_chaser_s_block_holds_its_chase_on_a_unit() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            block_radius: 48,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let foe = put(&mut sim, 0, ty, at);
    let me = put(&mut sim, 1, ty, at);
    sim.chain_remove(me);
    let reach = sim.max_range_of(Obj::Unit(me)) * 0xc0 + 6;
    // A unit-cell centre east of the target, inside the reach less the
    // non-mandatory chase's `0x90` margin.
    let inside = (0..0x1000)
        .map(|dx| Pos::new(at.x + dx * 48 + 24, at.y))
        .find(|&p| {
            sim.units[me].pos = p;
            let d = sim.attack_dist(Obj::Unit(me), Obj::Unit(foe));
            (reach - 0x8f - 0x90..=reach - 0x90).contains(&d)
        })
        .expect("a spot inside the reach");
    let chase = |sim: &Sim, who: Player, cells: i32| {
        let mut s = sim.clone();
        s.units[me].pos = inside;
        s.chain_add(me);
        put(&mut s, who, ty, Pos::new(inside.x, inside.y + cells * 48));
        s.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        s.add_move_order(
            me,
            Pos::new(at.x - 0x400, at.y),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        s.work(me, 0);
        s.units[me].orders.iter().any(crate::orders::Order::is_move)
    };
    assert!(
        chase(&sim, 1, 2),
        "a mate two unit cells off let the chase end"
    );
    assert!(
        !chase(&sim, 1, 3),
        "a mate three unit cells off held the chase"
    );
    assert!(!chase(&sim, 8, 2), "gaia in the block held the chase");
}

/// **A fleeing target re-aims the chase** (`check_target_path@005e22d0`,
/// the listing `5e24e2`–`5e2873`; `docs/COMBAT.md` §67). On the review's
/// phase, a ranged chaser whose unit target is walking away from it asks
/// whether its **walk spot** still reaches the target; when it does not,
/// the legs go and `find_attack_pos` puts a new one in front. run347's
/// `1/24` re-aims on 4616 after the citizen `0/3` (Great Lakes 4673).
///
/// Four arms: the fleeing target re-aims; a standing target, a target
/// walking toward the chaser and an off-phase frame keep the stale spot.
/// Made to fail on purpose by returning at the fall-through, as this
/// crate did: the first arm keeps its stale spot.
#[test]
fn a_fleeing_target_re_aims_the_chase_on_the_review_s_phase() {
    let (mut sim, ty) = at_war();
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let foe = put(&mut sim, 0, ty, at);
    let me = put(&mut sim, 1, ty, Pos::new(at.x + 12 * 0xc0, at.y));
    let stale = Pos::new(at.x + 10 * 0xc0, at.y);
    let away = crate::movement::find_angle(at.x - sim.units[me].pos.x, 0);
    let toward = crate::movement::find_angle(sim.units[me].pos.x - at.x, 0);
    // The review's phase: `(frame + o) % 16 == 0`.
    let on = 16 - i64::from(sim.units[me].index);
    let chase_to = |sim: &Sim, stale: Pos, heading: Option<crate::movement::Angle>, frame: i64| {
        let mut s = sim.clone();
        if let Some(h) = heading {
            s.add_move_order(
                foe,
                Pos::new(at.x - 20 * 0xc0, at.y),
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::New,
                false,
            );
            s.units[foe].movement.heading = h;
        }
        s.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            true,
        );
        s.add_move_order(
            me,
            stale,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        let before = s.units[me].orders_pos;
        s.work(me, frame);
        let after = s.units[me].orders.iter().find_map(|o| match o.body {
            crate::orders::Body::Move(m) => Some(m.dest),
            _ => None,
        });
        (before, after, s)
    };
    let chase = |sim: &Sim, heading, frame| chase_to(sim, stale, heading, frame);
    let (before, after, s) = chase(&sim, Some(away), on);
    let after = after.expect("the chase keeps a move");
    assert_ne!(
        after, before,
        "the fleeing target's chase kept its stale spot"
    );
    // A target running straight away answers `flanking(e) == 1`, so
    // `find_attack_pos` leads it (`Sim::chase_lead`, item 3789,
    // `docs/GOLDEN.md` §150.3): the new spot is ahead of the target,
    // taken outright and not tested for range against its seat.
    assert!(
        after.x < at.x && !s.is_in_range_at(Obj::Unit(me), after, Obj::Unit(foe)),
        "the new spot leads the fleeing target: {after:?} against {at:?}"
    );
    for (heading, frame, why) in [
        (None, on, "a standing target"),
        (Some(toward), on, "a target walking toward the chaser"),
        (Some(away), on + 1, "an off-phase frame"),
    ] {
        let (before, after, _) = chase(&sim, heading, frame);
        assert_eq!(after, Some(before), "{why} re-aimed the chase");
    }
    // The range is asked from the **walk spot** (`orders_x/orders_y`,
    // `5e2553`/`5e2558`), not from where the chaser stands: a spot that
    // still reaches the target keeps the chase, though the chaser, at
    // twelve tiles, is out of reach.
    let near = Pos::new(at.x + 3 * 0xc0, at.y);
    let (before, after, _) = chase_to(&sim, near, Some(away), on);
    assert_eq!(
        after,
        Some(before),
        "a walk spot in reach re-aimed the chase"
    );
}

/// **A dead target is re-aimed at** (`check_target_path@005e22d0`,
/// `5e2429`–`5e242e`; `docs/COMBAT.md` §76). The slot's vslot `+0x8`
/// is `SubObjectData::is_active`, and a dead target's `je 5e24e8`
/// jumps the flank triple into the same fall-through a fleeing one
/// takes: its walk spot cannot reach a corpse, so on the review's phase
/// the legs go and `find_attack_pos` aims a new one at where it fell.
/// run347's `1/13` re-aims on 5011, 5027, 5043 and 5059 at `0/2`, dead
/// since 5000 (Great Lakes 5066).
///
/// Three arms: the dead target re-aims, standing still; the same target
/// alive and standing, and the dead one off the phase, keep the stale
/// spot. Made to fail on purpose by returning for a dead target, as
/// this crate did: the first arm keeps its stale spot.
#[test]
fn a_dead_target_is_re_aimed_at_on_the_review_s_phase() {
    let (mut sim, ty) = at_war();
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let foe = put(&mut sim, 0, ty, at);
    // Past `(max_range + 8)` tiles, `find_attack_pos`' far arm, which
    // takes the sweep's spot without asking the range (a corpse is in
    // nobody's): `1/13` was some fifteen tiles out.
    let me = put(&mut sim, 1, ty, Pos::new(at.x + 16 * 0xc0, at.y));
    // Ten tiles from the chaser, so `do_move`'s dead-target arm
    // (`0x480` from the walk's own goal) leaves it standing.
    let stale = Pos::new(at.x + 6 * 0xc0, at.y + 3 * 0xc0);
    // The review's phase: `(frame + o) % 16 == 0`.
    let on = 16 - i64::from(sim.units[me].index);
    let chase = |sim: &Sim, dead: bool, frame: i64| {
        let mut s = sim.clone();
        s.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            true,
        );
        s.add_move_order(
            me,
            stale,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        if dead {
            s.units[foe].health = 0;
        }
        let before = s.units[me].orders_pos;
        s.work(me, frame);
        let after = s.units[me].orders.iter().find_map(|o| match o.body {
            crate::orders::Body::Move(m) => Some(m.dest),
            _ => None,
        });
        (before, after)
    };
    let (before, after) = chase(&sim, true, on);
    let after = after.expect("the chase keeps a move");
    assert_ne!(after, before, "the dead target's chase kept its stale spot");
    assert!(
        crate::world::vector_dist((after.x - at.x).abs(), (after.y - at.y).abs())
            < crate::world::vector_dist((before.x - at.x).abs(), (before.y - at.y).abs()),
        "the new spot is not aimed at the corpse: {after:?}"
    );
    for (dead, frame, why) in [
        (false, on, "a living standing target"),
        (true, on + 1, "an off-phase frame"),
    ] {
        let (before, after) = chase(&sim, dead, frame);
        assert_eq!(after, Some(before), "{why} re-aimed the chase");
    }
}

/// **An unpacked packer takes only what it can reach, and a hit from
/// inside its minimum is dropped without a chase**
/// (`Object::find_nearby_target@00648da0`'s `local_24`, the listing
/// `64918d` and `6495c2`; `Unit::fight@005fd4d0:1051`'s packer
/// re-search; `docs/COMBAT.md` §60). run146's catapult `0/6` is the
/// diff: its idle search on 864 finds nothing inside three tiles, and
/// each hoplite that hits it after is attacked for one frame and let
/// go.
///
/// Made to fail on purpose, both ways: with the range gate read back
/// as "takes anything" the search names the foe inside the minimum;
/// with the packer re-search off the retaliation walks away to get
/// its range.
#[test]
fn an_unpacked_packer_takes_only_what_it_can_reach() {
    let (mut sim, _) = at_war();
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 40,
            max_range: 15,
            min_range: 3,
            recharge: 30,
            uber_size: 1,
            obj_masks: mask::SIEGE,
            siege: true,
            packs: true,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let foe_ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            uber_size: 1,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let here = Pos::new(0x4000 + 24, 0x4000 + 24);
    let me = put(&mut sim, 0, ty, here);
    sim.units[me].combat.packed = false;
    let near = put(
        &mut sim,
        1,
        foe_ty,
        Pos::new(0x4000 + 2 * 192 + 24, 0x4000 + 24),
    );
    assert_eq!(
        sim.find_melee_target(me, -1),
        None,
        "an unpacked packer's search took a foe inside its minimum"
    );
    // Packed, the same engine must reach too: a packer's combat stance
    // is `get_combat_stance`'s STAND_GROUND (`00610a30`), and
    // `local_24` is set by `stance == 2` before the packer arm is read
    // (item 2265, `docs/GOLDEN.md` §82).
    sim.units[me].combat.packed = true;
    assert_eq!(sim.find_melee_target(me, -1), None);
    sim.units[me].combat.packed = false;
    // The hit: an attack on the hitter, then on the engine's next
    // frame the attack is gone and the engine has not moved.
    sim.do_damage(
        Obj::Unit(near),
        Obj::Unit(me),
        crate::movement::Angle(0),
        false,
        1,
        false,
        false,
        10,
    );
    assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(near)));
    sim.tick();
    assert!(
        sim.units[me].orders.is_empty(),
        "the engine kept an order: {:?}",
        sim.units[me].orders
    );
    assert!(
        sim.units[me].path.is_empty() && sim.units[me].movement.dest.is_none(),
        "the engine walked for its range"
    );
}

/// **A computer's packed siege engine must reach what it takes**
/// (twenty-fourth pass, group 10; A6 row 14): `local_24` and `local_5c`
/// are set before `get_army`, so with the army shortcut failed the
/// ranked search still skips an out-of-range unit. ~~A human's packed
/// engine, and the EASIEST computer's, take it untested.~~ They skip it
/// too, through the stance: a packer reads `get_combat_stance`'s
/// STAND_GROUND, and `stance == 2` sets `local_24` for every packer
/// (item 2265, `docs/GOLDEN.md` §82).
#[test]
fn a_packed_computer_siege_engine_skips_what_it_cannot_reach() {
    for (case, human, difficulty, want) in [
        ("computer", false, 5, false),
        ("human", true, 5, false),
        ("easiest", false, 0, false),
    ] {
        let (mut sim, foe_ty) = at_war();
        sim.cities_state.nation[0].human = human;
        sim.lobby.difficulty = difficulty;
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 4,
                uber_size: 1,
                siege: true,
                packs: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(0x1200, 0x1200);
        let me = put(&mut sim, 0, ty, at);
        sim.units[me].combat.packed = true;
        let foe = put(&mut sim, 1, foe_ty, Pos::new(at.x + 10 * 192, at.y));
        assert!(!sim.is_in_range(Obj::Unit(me), Obj::Unit(foe)));
        assert_eq!(
            sim.find_nearby_target(Obj::Unit(me), 0).is_some(),
            want,
            "{case}"
        );
    }
}

/// **An aircraft takes no aircraft it cannot reach** (item 650,
/// `docs/COMBAT.md` §61), on run168's two types. A Bomber (`FLY_HIGH`
/// 0, no `ANTI_AIR`) is refused a plane by `valid_target`'s air ladder,
/// because a plane with no air order flies high. A Fighter (`ANTI_AIR`,
/// seven tiles) may take the Bomber, but its search refuses one eight
/// tiles off through `poor_target`'s plane arm, and takes it at three.
/// A land rifleman (`FLY_HIGH` 0) is refused a plane too, and an
/// anti-aircraft gun is not.
///
/// Made to fail on purpose, both ways. With the ladder read back the
/// Bomber's `valid_target` is true. With the plane arm off the Fighter's
/// search names the Bomber at eight tiles.
#[test]
fn an_aircraft_takes_no_aircraft_it_cannot_reach() {
    let (mut sim, _) = at_war();
    let plane = |attack, max_range, fly_low, masks| crate::UnitType {
        hits: 100,
        combat: Profile {
            attack,
            max_range,
            fly_high: 0,
            fly_low,
            uber_size: 1,
            obj_masks: mask::AIR | masks,
            domain: Domain::Air,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    };
    let fighter_ty = sim.add_unit_type(plane(450, 7, 25, mask::ANTI_AIR));
    let bomber_ty = sim.add_unit_type(plane(430, 3, 10, mask::EXPLOSIVE));
    let ground = |max_range, fly_high, fly_low, masks| crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 200,
            max_range,
            fly_high,
            fly_low,
            uber_size: 1,
            obj_masks: masks,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    };
    let rifle_ty = sim.add_unit_type(ground(8, 0, 20, mask::GUN));
    let flak_ty = sim.add_unit_type(ground(14, 50, 90, mask::ANTI_AIR));
    let y = 0x4000 + 96;
    let fighter = put(&mut sim, 0, fighter_ty, Pos::new(0x4000 + 96, y));
    let bomber = put(&mut sim, 1, bomber_ty, Pos::new(0x4000 + 96 + 8 * 192, y));
    let (f, b) = (Obj::Unit(fighter), Obj::Unit(bomber));
    assert!(!sim.valid_target(b, f), "the Bomber may take a plane");
    assert!(
        sim.valid_target(f, b),
        "the Fighter may not take the Bomber"
    );
    assert_eq!(sim.find_melee_target(bomber, -1), None);
    assert_eq!(
        sim.find_melee_target(fighter, -1),
        None,
        "the Fighter's search took a plane beyond its reach"
    );
    let near = put(&mut sim, 1, bomber_ty, Pos::new(0x4000 + 96, y - 3 * 192));
    assert_eq!(sim.find_melee_target(fighter, -1), Some(Obj::Unit(near)));
    let rifle = put(&mut sim, 0, rifle_ty, Pos::new(0x4000 + 96, y + 192));
    let flak = put(&mut sim, 0, flak_ty, Pos::new(0x4000 + 96, y + 384));
    assert!(!sim.valid_target(Obj::Unit(rifle), b));
    assert!(sim.valid_target(Obj::Unit(flak), b));
}

/// The flee arm's `else`: a **combat** unit still retaliates, which
/// is the row item 464's restructure could have swallowed. Its gate
/// is `is_worker || is_idle`, and a soldier standing idle passes the
/// second half — so what keeps it out of the flight is the first
/// clause, `(CIVILIAN && max_range == 0) || attack == 0`.
#[test]
fn a_hit_soldier_retaliates_where_a_citizen_flees() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x4100, 0x4000));
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Unit(me),
        crate::movement::Angle(0),
        false,
        1,
        false,
        false,
        10,
    );
    assert_eq!(
        sim.units[me].combat.target,
        Some(Obj::Unit(foe)),
        "an idle soldier answers the hit with an attack, not a flight"
    );
}

/// **A saboteur struck mid-cast loses half its clock to a stranger
/// only** (`Unit::target_opportunity@005fffc0`, `600534`..`60056e`,
/// item 3739, `docs/AI.md` §366.2): a unit whose action is a Sabotage
/// cast (`0x280`) halves `spell_time` when the attacker's `who` and
/// `o` both differ from the cast's target's, and otherwise answers
/// nothing. Any other craft answers nothing either way, and no cast
/// is ever answered with an attack or a flight.
///
/// Made to fail with the arm's `&&` written `||` (the stranger of the
/// target's own player kept its clock).
#[test]
fn a_saboteur_struck_mid_cast_halves_its_clock_only_for_a_stranger() {
    use crate::orders::QueuePos;
    let clock = |spell: i32, target_owner: u8, target_index: i16| {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        let mark = put(&mut sim, target_owner, ty, Pos::new(0x4400, 0x4000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x4100, 0x4000));
        sim.units[mark].index = target_index;
        sim.units[foe].index = 7;
        let at = sim.units[me].pos;
        sim.add_cast_order_on(me, spell, Some(Obj::Unit(mark)), at, QueuePos::New, true);
        sim.units[me].spell_time = 9;
        let before = sim.units[me].orders.clone();
        sim.target_opportunity(me, Obj::Unit(foe), 10);
        assert_eq!(
            sim.units[me].orders, before,
            "a cast is never answered with an order"
        );
        assert_eq!(sim.units[me].combat.target, None);
        sim.units[me].spell_time
    };
    assert_eq!(
        clock(0x280, 0, 3),
        4,
        "another player's unit, another number: halved"
    );
    assert_eq!(clock(0x280, 1, 3), 9, "the target's own player: kept");
    assert_eq!(clock(0x280, 0, 7), 9, "the attacker's own number: kept");
    assert_eq!(clock(0x281, 0, 3), 9, "the Sniper's cast: kept");
}

/// **A busy unit answers a hit only on duty** (item 1040,
/// `docs/COMBAT.md` §70): `Unit::target_opportunity`'s `600863`
/// returns for a unit that holds any order and is not
/// `Unit::on_duty@005fff70` — a combat-role type whose activity is an
/// `ATTACK_TO`, a patrol, a `GUARD` or a group attack or patrol. Great
/// Lakes' `0/4`, a citizen walking to its drop site under its
/// `GATHER`, took `1/24`'s stone on 4779 and kept walking; this crate
/// had it turn on the Slinger.
///
/// Made to fail first with the gate removed: the walker answered.
#[test]
fn a_busy_unit_answers_a_hit_only_on_duty() {
    use crate::orders::{MoveKind, QueuePos};
    let answer = |role: bool, kind: Option<MoveKind>| {
        let (mut sim, ty) = at_war();
        sim.tables.unit_types[ty].combat.combat_role = role;
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x4100, 0x4000));
        if let Some(k) = kind {
            sim.add_move_order(me, Pos::new(0x2000, 0x4000), k, QueuePos::First, false);
        }
        let before = sim.units[me].orders.clone();
        sim.target_opportunity(me, Obj::Unit(foe), 10);
        let answered = sim.units[me].combat.target == Some(Obj::Unit(foe));
        assert_eq!(
            answered,
            sim.units[me].orders != before,
            "an answer is an order, and silence leaves the list alone"
        );
        answered
    };
    assert!(!answer(false, Some(MoveKind::MoveTo)), "a walker");
    assert!(
        !answer(true, Some(MoveKind::MoveTo)),
        "a soldier on a plain move is not on duty"
    );
    assert!(
        answer(true, Some(MoveKind::AttackTo)),
        "a soldier on an attack-move is"
    );
    assert!(
        !answer(false, Some(MoveKind::AttackTo)),
        "on duty asks the combat role first"
    );
    assert!(
        answer(false, None),
        "an idle unit answers whatever its role"
    );
}

/// **A dead target outlives its order by the reload** —
/// `docs/ORDERS.md` §7.12, and the arm is `Unit::fight@005fd4d0:102`
/// returning before `:196`'s `Object::valid_target`.
/// `Unit::do_attack@005f1b80` asks nothing about the target for a
/// unit whose type has `attack` (both of its aliveness tests sit
/// under `ptype->attack == 0`), so a recharging attacker whose target
/// has just died keeps the order, and drops it on the frame the
/// reload reaches nought — never before.
///
/// Made to fail on purpose, both ways: with the validity test back in
/// front of the reload gate the order dies on the **first** frame;
/// with [`Sim::forget`] clearing `mandatory` again — which is the
/// order's own field, `TargetOrder +0x1c`, and not the dead object's
/// — a HOLD_FIRE attacker returns at `do_attack`'s stance arm and the
/// order never dies at all.
///
/// **And the target itself outlives the object too** (item 502,
/// `docs/COMBAT.md` §43.3): `Sim::forget` no longer clears it, which
/// is what run112's three bowmen say — their `ATTACKORDER` carries the
/// dead `1/8` for twelve blocks. So the first assertion below is the
/// opposite of what it was, and the reload's twelve frames are now
/// twelve frames of a **named** dead target rather than of nothing.
///
/// run100's own measurement is the six raiders of Great Lakes blocks
/// 10231–10239, each dropping on its own `recharging` clock: `1/42`
/// on 10231, `1/28` on 10233, `1/27` on 10239.
#[test]
fn a_recharging_attacker_keeps_a_dead_target_s_order_until_the_reload_ends() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    // HOLD_FIRE and mandatory, which is run100's raiders exactly: the
    // stance arm is what the cleared `mandatory` used to reach.
    sim.units[me].combat.stance = Stance::HoldFire;
    sim.add_attack_order(
        me,
        Obj::Unit(foe),
        crate::orders::QueuePos::First,
        true,
        false,
    );
    sim.units[me].combat.recharging = 3;
    sim.units[foe].health = 0;
    sim.forget(Obj::Unit(foe));
    assert_eq!(
        sim.units[me].combat.target,
        Some(Obj::Unit(foe)),
        "the dead target is kept until the next use asks about it"
    );
    assert!(
        sim.units[me].combat.mandatory,
        "the order's own `mandatory` outlives the object it named"
    );
    // `Unit::process` counts the reload down before `work`.
    for f in 0..2 {
        sim.units[me].combat.recharging -= 1;
        sim.work(me, f);
        assert_eq!(
            sim.units[me].orders.len(),
            1,
            "frame {f}: the order died while `recharging` was {}",
            sim.units[me].combat.recharging
        );
    }
    sim.units[me].combat.recharging -= 1;
    sim.work(me, 2);
    assert!(
        sim.units[me].orders.is_empty(),
        "the order outlived the reload: {:?}",
        sim.units[me].orders
    );
    // **No freeze mark**, and the reason is the condition rather than
    // the arm (§44.1). This unit reached `Unit::fight`'s
    // invalid-target branch, but it is HOLD_FIRE, so the search
    // returns nothing, so no attack order goes back in front — and
    // `unit_masks2 |= 0x10` is downstream of `order_type() == ATTACK`
    // *after* the search. A unit that finds nothing has nothing to be
    // "still ordered" under.
    assert_eq!(sim.units[me].unit_masks2, 0);
}

/// **The search budget keeps a dead target's order a frame** (item
/// 3598, `docs/AI.md` §318.3). `Unit::fight@005fd4d0`'s invalid-target
/// arm, `LAB_005fdb9e`: with the unit's `waiting` below 5 and its
/// leader's invalid targets past ten this frame, it marks the unit
/// frozen, adds 2 to `waiting` and returns, searching nothing and
/// counting nothing. run1241b's Phalanx `1/116` is the witness: who=1
/// reads `retargets 11` on block 6718, and the Phalanx `waiting 2`,
/// `unit_masks2 16` and its `ATTACKORDER` on the dead `0/12`, which it
/// drops on the next frame with `waiting` 1. At ten, the order drops at
/// once.
#[test]
fn the_search_budget_keeps_a_dead_target_s_order_a_frame() {
    for count in [10, 11] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        sim.units[foe].health = 0;
        sim.forget(Obj::Unit(foe));
        sim.combat.retargets[0] = count;
        sim.work(me, 0);
        if count == 10 {
            assert!(sim.units[me].orders.is_empty(), "at ten, no budget");
            assert_eq!(sim.units[me].waiting, 0);
            assert_eq!(sim.combat.retargets[0], 11, "the search is counted");
            continue;
        }
        assert_eq!(sim.units[me].orders.len(), 1, "past ten the order stands");
        assert_eq!(sim.units[me].waiting, 2);
        assert_eq!(sim.combat.retargets[0], 11, "nothing is counted");
        assert_ne!(
            sim.units[me].unit_masks2 & combat::umask2::NOT_FIRING,
            0,
            "the frozen mark"
        );
        // The next frame: `Unit::process` takes one off, the leader's
        // count starts again, and the order drops.
        sim.units[me].waiting -= 1;
        sim.combat.retargets[0] = 0;
        sim.work(me, 1);
        assert!(
            sim.units[me].orders.is_empty(),
            "{:?}",
            sim.units[me].orders
        );
        assert_eq!(sim.units[me].waiting, 1);
    }
}

/// **A mandatory attack on a dead figure takes its squad's captain, in
/// the same order** (item 4002, `docs/GOLDEN.md` §191; `Unit::fight@
/// 005fd4d0:300`–`:335`). The dead middle figure's `o_up` walks, after
/// the relink, to the squad's head, and the attack names it with its
/// `ever_in_range` and `new_ord` untouched and a chase over it — run1545's
/// `1/14` on 810. A plain attack drops the order to the search instead.
#[test]
fn a_mandatory_attack_on_a_dead_figure_takes_its_squad_s_captain() {
    for mandatory in [true, false] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let [a, b, c] = [0x2000, 0x2030, 0x2060].map(|x| put(&mut sim, 1, ty, Pos::new(x, 0x1000)));
        for (f, up, down) in [
            (a, None, Some(b)),
            (b, Some(a), Some(c)),
            (c, Some(b), None),
        ] {
            sim.units[f].captain = up.is_none();
            sim.units[f].o_up = up;
            sim.units[f].o_down = down;
            sim.units[f].combat.captain = i32::from(sim.units[a].index);
        }
        sim.add_attack_order(
            me,
            Obj::Unit(b),
            crate::orders::QueuePos::First,
            mandatory,
            false,
        );
        if let Some(crate::orders::Order {
            body: crate::orders::Body::Attack(x),
            ..
        }) = sim.units[me].orders.front_mut()
        {
            x.ever_in_range = true;
            x.new_ord = false;
        }
        sim.units[b].health = 0;
        sim.relink_squad(b);
        sim.forget(Obj::Unit(b));
        sim.work(me, 0);
        let attack = sim.units[me].orders.iter().find_map(|o| match o.body {
            crate::orders::Body::Attack(x) => Some(x),
            _ => None,
        });
        if !mandatory {
            // `LAB_005fdb9e`: the order goes and the search adds a
            // fresh one or none; it is never the old order retargeted.
            assert!(
                attack.is_none_or(|x| x.new_ord && !x.ever_in_range),
                "the old order kept: {attack:?}"
            );
            continue;
        }
        let attack = attack.expect("the mandatory order stands");
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(a)), "the head");
        assert!(
            attack.ever_in_range && !attack.new_ord,
            "the same order: {attack:?}"
        );
        assert!(sim.units[me].combat.mandatory, "still mandatory");
        assert!(
            matches!(
                sim.units[me].orders.front().map(|o| &o.body),
                Some(crate::orders::Body::Move(_))
            ),
            "the re-entered work's chase: {:?}",
            sim.units[me].orders
        );
    }
}

/// **An emplaced weapon drops a dead target's order at once, reload or
/// not** (item 3540, `docs/ORDERS.md` §7.12). `Unit::fight@005fd4d0:
/// 94`–`102`: the type's `unit_flags2 & 1` (the machine gun and
/// flamethrower lineages) and an invalid target set `local_1c`, which
/// walks past the reload gate to `:196`'s kill. run1184's Flamethrowers
/// `1/84` and `1/129` drop their attack on the dead building `0/2004`
/// with `recharging` 23 and 12. The same unit without the bit keeps
/// the order, as §7.12's raiders do. Made to fail on purpose by
/// clearing the latch.
#[test]
fn an_emplaced_weapon_drops_a_dead_target_s_order_while_recharging() {
    for emplaced in [true, false] {
        let (mut sim, ty) = at_war();
        if emplaced {
            sim.tables.unit_types[ty].cols.unit_flags2 |= crate::ai_load::uflags2::EMPLACED;
        }
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        sim.units[me].combat.stance = Stance::HoldFire;
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            true,
            false,
        );
        sim.units[me].combat.recharging = 12;
        sim.units[foe].health = 0;
        sim.forget(Obj::Unit(foe));
        sim.units[me].combat.recharging -= 1;
        sim.work(me, 0);
        assert_eq!(
            sim.units[me].orders.is_empty(),
            emplaced,
            "emplaced {emplaced}: {:?}",
            sim.units[me].orders
        );
    }
}

/// **A recharging melee unit with no order steps nothing**:
/// `Unit::work@0060d180:79-86`'s gate returns for a non-`ATTACK` unit
/// with `recharging` (`+0xae`) above nought and no range (`+0x1fc`)
/// when it has **no order** as well as when its order lacks the action
/// bit, so it reaches neither `do_idle` nor `check_idle` (`idle`,
/// `+0xb0`, stays 0) until the countdown ends. The battery word 13054
/// (`docs/AI.md` §245): the Knight `1/65` ends its `ATTACK_TO` at
/// `recharging` 21 and stands orderless at `idle` 0 in the original.
/// Made to fail on purpose by restoring the gate's `Some(order)` test.
#[test]
fn a_recharging_melee_unit_with_no_order_does_not_idle() {
    let (mut sim, _) = at_war();
    let melee = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 0,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let me = put(&mut sim, 0, melee, Pos::new(0x1000, 0x1000));
    let _foe = put(&mut sim, 1, melee, Pos::new(0x1100, 0x1000));
    sim.units[me].combat.recharging = 2;
    sim.work(me, 1);
    assert_eq!(
        sim.units[me].idle, 0,
        "a recharging melee unit does not idle"
    );
    assert!(sim.units[me].orders.is_empty(), "and takes no order");
    sim.units[me].combat.recharging = 0;
    sim.work(me, 2);
    assert_eq!(sim.units[me].idle, 1, "the countdown over, it idles");
}

/// **A move whose action's target has died is only re-pathed when
/// the walk is nearly over** (item 487, `docs/ORDERS.md` §20).
///
/// `do_move@005f7b30`'s `action->type == 10` block falls out of the
/// valid-target arm when the object is gone, and what it does then is
/// `005f8221`-`005f825d`:
///
/// ```text
/// vector_dist(|x − move.x|, |y − move.y|)   ; the order's own dest
/// cmp eax, 0x480 / jg   → the planner
/// test $0x10, 0x2b4(ptype) / jne → the planner   ; a sea transport
/// call Unit::repath
/// ```
///
/// `MoveOrder +0x4`/`+0x8` are `x`/`y` by the type record, so the
/// distance is to where the unit was walking. Inside `0x480` the walk
/// is pointless and the transit legs are popped; beyond it the unit
/// keeps the order and plans, dead target and all. This crate popped
/// them at any distance, which is what stood run100's `1/29` still
/// 28,000 units from home for the length of its reload.
///
/// Both arms are asserted, and the gate was made to fail on purpose
/// by taking the distance test out again — which kills the far move
/// on its first frame.
#[test]
fn a_dead_target_s_move_is_repathed_only_within_0x480_of_its_destination() {
    for (name, dest, survives) in [
        ("near", Pos::new(0x1000 + 0x400, 0x1000), false),
        ("far", Pos::new(0x1000 + 0x4000, 0x1000), true),
    ] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, dest);
        // The attack is the **action** under the move, which is the
        // only shape the block reads.
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            true,
            true,
        );
        sim.add_move_order(
            me,
            dest,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        assert_eq!(
            sim.units[me].orders.len(),
            2,
            "{name}: the move and its action"
        );
        sim.units[foe].health = 0;
        sim.forget(Obj::Unit(foe));
        sim.work(me, 0);
        assert_eq!(
            sim.units[me]
                .orders
                .iter()
                .any(crate::orders::Order::is_move),
            survives,
            "{name}: the move at {dest:?} should {} have survived: {:?}",
            if survives { "" } else { "not" },
            sim.units[me].orders
        );
    }
}

/// **A squad member never searches: it mirrors its captain.**
/// `Unit::think@005f6e40`'s first statement, above every gate in the
/// function (`docs/COMBAT.md` §21) — a non-captain reads its captain's
/// **action**, and takes that target if the action is an ATTACK order
/// (`get_type() == 10`) on something it can validly attack.
///
/// Three arms, each made to fail on purpose:
///
/// - the captain holding an ATTACK order hands it down;
/// - a captain whose action is a **move** hands nothing down — the
///   `get_type() != 10` return, and the member keeps whatever it had;
/// - a member whose `valid_target` fails on the captain's target takes
///   nothing either, which is the only one of the five returns the
///   golden record cannot see.
///
/// The captain itself is never mirrored: `captain_mirror` is a no-op on
/// it, because `Unit::think` reaches the rest of the function instead.
#[test]
fn a_squad_member_mirrors_its_captain_s_attack_order() {
    let (mut sim, ty) = at_war();
    let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    sim.units[sub].captain = false;
    sim.units[sub].o_up = Some(cap);
    sim.units[cap].o_down = Some(sub);
    // Nothing to mirror while the captain is idle.
    sim.captain_mirror(sub);
    assert_eq!(sim.units[sub].combat.target, None);
    // The captain takes an attack order; the member takes the same one.
    sim.add_attack_order(
        cap,
        Obj::Unit(foe),
        crate::orders::QueuePos::New,
        false,
        false,
    );
    sim.captain_mirror(sub);
    assert_eq!(
        sim.units[sub].combat.target,
        Some(Obj::Unit(foe)),
        "the member is handed the captain's target"
    );
    // **A captain walking to its chase point still hands the target
    // down**: `get_action` skips the leading transit moves, so the
    // ATTACK order under them is still what is read.
    sim.add_move_order(
        cap,
        Pos::new(0x1080, 0x1000),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::First,
        false,
    );
    sim.units[sub].combat.target = None;
    sim.captain_mirror(sub);
    assert_eq!(
        sim.units[sub].combat.target,
        Some(Obj::Unit(foe)),
        "the action is the attack under the transit move"
    );
    // A captain whose **action** is a move hands nothing down — the
    // `get_type() != 10` return, taken with the target still on the
    // captain, so it is the order's kind that refuses and not a
    // missing target.
    sim.add_move_order(
        cap,
        Pos::new(0x1200, 0x1000),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::First,
        true,
    );
    sim.units[sub].combat.target = None;
    sim.captain_mirror(sub);
    assert_eq!(
        (sim.units[cap].combat.target, sim.units[sub].combat.target),
        (Some(Obj::Unit(foe)), None),
        "`get_type() != 10` returns before the target is read"
    );
    // And the **member's own** `valid_target` is what is tested: an
    // ally target passes for nobody, and this is the one of the five
    // returns the golden record cannot see.
    let (mut sim, ty) = at_war();
    let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
    let friend = put(&mut sim, 0, ty, Pos::new(0x1100, 0x1000));
    sim.units[sub].captain = false;
    sim.units[sub].o_up = Some(cap);
    sim.units[cap].o_down = Some(sub);
    sim.add_attack_order(
        cap,
        Obj::Unit(friend),
        crate::orders::QueuePos::New,
        false,
        false,
    );
    assert!(!sim.valid_target(Obj::Unit(sub), Obj::Unit(friend)));
    sim.captain_mirror(sub);
    assert_eq!(sim.units[sub].combat.target, None);
    // A captain mirrors nothing — the arm is `!is_captain` only, and
    // `squad_captain` answers the unit itself for one.
    sim.captain_mirror(cap);
    assert_eq!(sim.units[cap].combat.target, Some(Obj::Unit(friend)));
}

/// And with the table the size a real lobby gives it — two — asking the
/// question at all is what took the first fuzzed seed down. Both
/// accessors are total now (`Sim::is_ally`).
#[test]
fn the_diplomacy_accessors_are_total_past_the_table() {
    let (sim, _) = at_war();
    assert_eq!(sim.at_war.len(), 2);
    assert!(!sim.is_enemy(0, PLAYER_SLOTS));
    assert!(!sim.is_ally(0, PLAYER_SLOTS));
    assert!(!sim.is_enemy(PLAYER_SLOTS, 0));
    assert!(!sim.is_ally(PLAYER_SLOTS, 0));
    // Reflexivity survives it.
    assert!(sim.is_ally(PLAYER_SLOTS, PLAYER_SLOTS));
}

/// `Ammo::check_hit@00678d90` is an `ObjectsData::find_unit`, whose
/// leader loop runs eight slots. So an arrow that comes down on top of a
/// sheep passes through it.
#[test]
fn an_arrow_never_lands_on_gaia() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let landing = Pos::new(0x1400, 0x1000);
    let sheep = put(&mut sim, PLAYER_SLOTS, ty, landing);
    let ammo = combat::Projectile {
        shooter: Obj::Unit(me),
        owner: 0,
        target: None,
        launch: sim.units[me].pos,
        landing,
        cur_time: 0,
        total_time: 1,
        accuracy: 100,
        angle: crate::movement::Angle(0),
        splash_area: 0,
        num_guys: 1,
        rolling: false,
        missed: false,
        harmless: false,
        air: false,
        sz: 0,
        ez: 0,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    };
    assert_eq!(sim.check_hit(&ammo), None);
    // The same arrow does find a player's unit standing there.
    let foe = put(&mut sim, 1, ty, landing);
    assert_eq!(sim.check_hit(&ammo), Some(Obj::Unit(foe)));
    assert!(sim.units[sheep].is_gaia());
}
/// **A round with no target that comes down on a building strikes it
/// as its own target** (item 1077, `docs/COMBAT.md` §73). run371's
/// V2: `whom −1`, landing at (13834, 14969) on who=1's Barracks
/// `1/2006` at (13824, 14976), a 5×5 footprint, with no unit within two
/// tiles. `Ammo::check_hit@00678d90` finds no unit and then
/// `find_building_at` on the landing's tile, so the splash arm's walk
/// hits the Barracks with `splash` 0: the same damage as a round aimed
/// at it, where a fringe takes `SPLASH_PERCENT` of it. A landing one
/// tile past the footprint finds nothing.
///
/// Made to fail by the tile-against-point comparison this replaced:
/// `check_hit` answers `None` and the Barracks takes the fringe's 400.
#[test]
fn a_round_with_no_target_strikes_the_building_it_lands_on() {
    let (mut sim, ty) = at_war();
    sim.tables.unit_types[ty].combat.attack = 150;
    sim.tables.unit_types[ty].combat.splash_area = 1;
    sim.tables.unit_types[ty].combat.splash_percent = 25;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 5,
        y_size: 5,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(13824, 14976);
    let b = sim.add_building(1, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 1200;
    sim.buildings[b].health = 1200;
    sim.buildings[b].combat = Some(Profile::default());
    let me = put(&mut sim, 0, ty, Pos::new(9974, 12347));
    let ammo = combat::Projectile {
        shooter: Obj::Unit(me),
        owner: 0,
        target: None,
        launch: Pos::new(9974, 12347),
        landing: Pos::new(13834, 14969),
        cur_time: 120,
        total_time: 120,
        accuracy: 228,
        angle: crate::movement::Angle(0),
        splash_area: 1,
        num_guys: 1,
        rolling: false,
        missed: false,
        harmless: false,
        air: false,
        sz: 466,
        ez: 78,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    };
    assert_eq!(sim.check_hit(&ammo), Some(Obj::Building(b)));
    // The same round aimed at the Barracks, beside it.
    let mut aimed = sim.clone();
    aimed.land(
        combat::Projectile {
            target: Some(Obj::Building(b)),
            ..ammo
        },
        2820,
    );
    sim.land(ammo, 2820);
    assert!(
        sim.buildings[b].health < 1200,
        "the Barracks was not struck"
    );
    assert_eq!(
        sim.buildings[b].health, aimed.buildings[b].health,
        "struck as a fringe, not as the round's own target"
    );
    // One tile past the footprint's east edge: nothing.
    let corner = sim.tile_corner(bt, at);
    let past = Pos::new((corner.x + 5) * 0xc0 + 0x60, 14969);
    assert_eq!(
        sim.check_hit(&combat::Projectile {
            landing: past,
            ..ammo
        }),
        None
    );
}

/// **A round that misses comes down on a city at its ceiling and
/// strikes it** (item 3539, `docs/AI.md` §303). `find_building_at`'s
/// candidate is two flag reads (vslots `0xc` and `0x4c`, flags `& 1`
/// and `& 4`) and nothing about hit points: run1171's Light Tank `1/28`,
/// whose round of 4434 lost its target in flight, comes down on 4443 on
/// the tile of Napata `0/2000`, reduced on 4341 to `health` 0, and
/// theirs strikes it and spends no puncture draws. An unfinished
/// building (`flags & 4` clear) is still refused, and the round
/// punctures the ground.
///
/// Made to fail by [`Sim::active`] in `building_under`: `check_hit`
/// answers `None` and the landing spends the puncture's two draws.
#[test]
fn a_missed_round_strikes_a_city_at_its_ceiling() {
    let (mut sim, ty) = at_war();
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 7,
        y_size: 7,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(6240, 7008);
    let b = sim.add_building(0, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 1200;
    sim.buildings[b].health = 0;
    sim.buildings[b].combat = Some(Profile::default());
    let me = put(&mut sim, 1, ty, Pos::new(7272, 10200));
    let ammo = combat::Projectile {
        shooter: Obj::Unit(me),
        owner: 1,
        target: None,
        launch: Pos::new(7218, 10078),
        landing: Pos::new(6818, 6488),
        cur_time: 10,
        total_time: 10,
        accuracy: 250,
        angle: crate::movement::Angle(0),
        splash_area: 0,
        num_guys: 1,
        rolling: true,
        missed: true,
        harmless: false,
        air: false,
        sz: 0,
        ez: 0,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    };
    assert!(
        !sim.active(Obj::Building(b)),
        "the target search refuses it"
    );
    assert_eq!(sim.check_hit(&ammo), Some(Obj::Building(b)));
    let sites = |s: &mut Sim| {
        s.trace_phases = true;
        s.phase_marks.clear();
        s.land(ammo, 4443);
        s.phase_marks
            .iter()
            .map(|(m, _)| m.clone())
            .collect::<Vec<_>>()
    };
    let struck = sites(&mut sim.clone());
    assert!(
        !struck.iter().any(|m| m == SITE_PUNCTURE_X),
        "no puncture draws: {struck:?}"
    );
    // A site not yet finished is no candidate: the round punctures.
    sim.buildings[b].active = false;
    assert_eq!(sim.check_hit(&ammo), None);
    assert_eq!(
        sites(&mut sim),
        [SITE_PUNCTURE_X, SITE_PUNCTURE_Y],
        "the puncture's two draws"
    );
}

/// **The shield closes a missile's round in its holder's land**
/// (item 1078, `Ammo::do_damage@00678060`'s `678337`..`67845d`):
/// run390's V2b came down on who=1's Barracks `1/2007` on 3169, in a
/// cell of who=1's, after `tech who=1 missile_shield on`, and the
/// Barracks stood at damage 0. The same round with the shield unheld,
/// or in a cell no one owns, strikes it. Made to fail with the close
/// dropped (the Barracks struck).
#[test]
fn the_shield_closes_a_missile_s_round_in_its_holder_s_land() {
    let (mut sim, ty) = at_war();
    sim.tables.unit_types[ty].combat.attack = 150;
    sim.tables.unit_types[ty].combat.splash_area = 1;
    sim.tables.unit_types[ty].combat.splash_percent = 25;
    sim.tables.unit_types[ty].combat.obj_masks |= mask::MISSILE;
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 5,
        y_size: 5,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(13824, 14976);
    let b = sim.add_building(1, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].hits = 1200;
    sim.buildings[b].health = 1200;
    sim.buildings[b].combat = Some(Profile::default());
    let me = put(&mut sim, 0, ty, Pos::new(9974, 12347));
    let ammo = combat::Projectile {
        shooter: Obj::Unit(me),
        owner: 0,
        target: None,
        launch: Pos::new(9974, 12347),
        landing: Pos::new(13834, 14969),
        cur_time: 120,
        total_time: 120,
        accuracy: 228,
        angle: crate::movement::Angle(0),
        splash_area: 1,
        num_guys: 1,
        rolling: false,
        missed: false,
        harmless: false,
        air: false,
        sz: 466,
        ez: 78,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    };
    let mut tree = sim.tables.tech_tree.clone();
    let shield = tree.types.len();
    tree.types.push(crate::tech::TypeDef::new(
        "Missile Shield",
        crate::tech::Kind::Final,
    ));
    tree.roles.missile_defense_preq = Some(shield);
    sim.set_tech_tree(tree);
    for p in &mut sim.tech {
        p.tech.resize(shield + 1, false);
    }
    sim.tech[1].tech[shield] = true;
    let cell = ammo.landing.cell();
    let mut unowned = sim.clone();
    unowned
        .world
        .set_owner(cell, crate::world::Owner::None, crate::world::Owner::None);
    sim.world.set_owner(
        cell,
        crate::world::Owner::Player(1),
        crate::world::Owner::None,
    );
    let mut unheld = sim.clone();
    unheld.tech[1].tech[shield] = false;
    sim.land(ammo, 3169);
    assert_eq!(sim.buildings[b].health, 1200, "closed under the shield");
    unheld.land(ammo, 3169);
    assert!(
        unheld.buildings[b].health < 1200,
        "struck with the shield unheld"
    );
    unowned.land(ammo, 3169);
    assert!(
        unowned.buildings[b].health < 1200,
        "struck in no one's land"
    );
}

/// **The tie goes to the cell's chain head, not to the lowest object
/// number** (§12.2, §32.1) — chapter two's own geometry, which is the
/// case that separates the two orders.
///
/// `0/9` at `(888, 8376)` looks at three identical hoplites on one
/// world cell. Two of them tie exactly: `attack_dist` 1459 and 1468,
/// nine units apart on a score that divides by `0xc0`, so both land
/// in the same bucket and `compare_target` returns the same value for
/// two undamaged figures of one type. The keep test is strictly
/// greater on both sides, so the winner is whichever the scan reached
/// first — and `Object::add_to_world` pushes on the **head**, so the
/// chain reaches the newest. run112's `ATTACKORDER` on all three
/// slingers reads `ox 8 whom 1`.
///
/// Made to fail on purpose by scanning `(0..units.len())` instead of
/// [`crate::Sim::cell_chain`] — the pick is then `1/6`, which is the
/// answer this crate gave for six items.
#[test]
fn a_tied_target_goes_to_the_cell_chain_s_head() {
    use crate::combat::Profile;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let mut kind = |max_range: i32| {
        sim.add_unit_type(crate::UnitType {
            hits: 120,
            combat: Profile {
                attack: 15,
                max_range,
                uber_size: 1,
                block_radius: 48,
                big_radius: 48,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        })
    };
    let slinger = kind(6);
    let hoplite = kind(0);
    let put = |sim: &mut crate::Sim, who: crate::Player, ty: usize, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 120);
        u.ty = Some(ty);
        u.on_map = true;
        let h = sim.add_unit(u);
        sim.units[h].orders_pos = p;
        h
    };
    let a9 = put(&mut sim, 0, slinger, Pos::new(888, 8376));
    // Block 622's three, in the dump's own order — so the chain, which
    // is newest first, is `1/8, 1/7, 1/6`.
    let b6 = put(&mut sim, 1, hoplite, Pos::new(2424, 7800));
    let b7 = put(&mut sim, 1, hoplite, Pos::new(2568, 7800));
    let b8 = put(&mut sim, 1, hoplite, Pos::new(2472, 7944));
    // **Anti-vacuity, and it is the whole point of the fixture.** The
    // three must sit on one cell — otherwise the ring order decides
    // and the chain never gets asked — and two of them must tie.
    let cell = sim.units[b6].pos.cell();
    assert_eq!(
        (sim.units[b7].pos.cell(), sim.units[b8].pos.cell()),
        (cell, cell),
        "the three hoplites are not on one world cell"
    );
    assert_eq!(
        sim.cell_chain(cell),
        vec![b8, b7, b6],
        "`chain_add` no longer pushes on the head"
    );
    let me = Obj::Unit(a9);
    let bucket = |sim: &crate::Sim, t: usize| (sim.attack_dist(me, Obj::Unit(t)) + 8 * 0x30) / 0xc0;
    assert_eq!(
        (bucket(&sim, b6), bucket(&sim, b8), bucket(&sim, b7)),
        (9, 9, 10),
        "`1/6` and `1/8` no longer tie, or `1/7` no longer fails to"
    );
    assert_eq!(
        sim.find_nearby_target(me, 4608),
        Some(Obj::Unit(b8)),
        "the tie went to the lowest object number instead of the \
             cell chain's head"
    );
}

/// A shot with nothing to home on, from `launch` to `landing` over
/// `total` frames, leaving at height `sz` and aimed at `ez`.
fn shot(me: usize, launch: Pos, landing: Pos, total: i32, sz: i32, ez: i32) -> combat::Projectile {
    combat::Projectile {
        shooter: Obj::Unit(me),
        owner: 0,
        target: None,
        launch,
        landing,
        cur_time: 0,
        total_time: total,
        accuracy: 100,
        angle: Angle(0),
        splash_area: 0,
        num_guys: 1,
        rolling: true,
        missed: false,
        harmless: false,
        air: false,
        sz,
        ez,
        v1z: combat::arc_v1z(sz, ez, total),
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    }
}

/// `Objects::add_ammo`'s lowest free slot, and `Objects::inc_time`'s
/// walk in slot order (`docs/COMBAT.md` §46.4): a landing frees its
/// slot without reordering the rest, and the next shot fired takes the
/// freed one and is stepped first.
#[test]
fn the_pool_takes_the_lowest_free_slot_and_steps_in_slot_order() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1400, 0x1000));
    let mut p = shot(me, a, b, 1, 0, 0);
    p.rolling = false;
    sim.add_ammo(p);
    sim.add_ammo(shot(me, a, b, 5, 0, 0));
    sim.add_ammo(shot(me, a, b, 6, 0, 0));
    let slots = |s: &Sim| {
        s.combat
            .projectiles
            .iter()
            .map(|p| (p.slot, p.total_time))
            .collect::<Vec<_>>()
    };
    assert_eq!(slots(&sim), vec![(0, 1), (1, 5), (2, 6)]);
    sim.process_projectiles(0);
    assert_eq!(
        slots(&sim),
        vec![(1, 5), (2, 6)],
        "slot 0 landed, the rest kept their order"
    );
    sim.add_ammo(shot(me, a, b, 7, 0, 0));
    assert_eq!(
        slots(&sim),
        vec![(0, 7), (1, 5), (2, 6)],
        "the freed slot, stepped first"
    );
}

/// **A `do_damage="0"` round lands and does nothing** (item 853,
/// `docs/ORDERS.md` §39.5): `Ammo::inc_time@0067d380:260` closes a
/// round with flag `0x10` instead of calling `Ammo::do_damage`. The
/// same round without the flag hits its target; with it the target
/// keeps its health, the round is gone, and a ground shot spends no
/// puncture draw. Made to fail with `process_projectiles` landing it
/// regardless.
#[test]
fn a_harmless_round_lands_without_damage() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1400, 0x1000));
    let foe = put(&mut sim, 1, ty, b);
    let aimed = |harmless: bool| {
        let mut p = shot(me, a, b, 1, 0, 0);
        (p.target, p.rolling, p.harmless) = (Some(Obj::Unit(foe)), false, harmless);
        p
    };
    let full = sim.units[foe].health;
    sim.add_ammo(aimed(true));
    sim.process_projectiles(0);
    assert!(sim.combat.projectiles.is_empty(), "the round is closed");
    assert_eq!(sim.units[foe].health, full, "and does no damage");
    let before = sim.rng;
    let mut ground = shot(me, a, Pos::new(0x1000, 0x1400), 1, 0, 0);
    (ground.rolling, ground.harmless) = (false, true);
    sim.add_ammo(ground);
    sim.process_projectiles(0);
    assert_eq!(sim.rng, before, "no puncture draw");
    sim.add_ammo(aimed(false));
    sim.process_projectiles(0);
    assert!(
        sim.units[foe].health < full,
        "the same round, harmful, hits"
    );
}

/// **A puncturing round that strikes nothing is left as a spent
/// arrow** (item 3679, `docs/COMBAT.md` §46.7): `Ammo::do_damage`'s
/// puncture arm keeps it in its slot at `flags 1`, `cur_time` 0, and
/// `Ammo::inc_time` closes it at 200 — bumping a dead shooter's
/// `hold_frames` every frame it lives, which is what held the Korean
/// Medieval pair's decoy `1/139` past the squad born on 4308. The same
/// round from a piece that does not puncture closes at once. Made to
/// fail with `stick` returning before the write, and with the walk's
/// spent arm closing at once.
#[test]
fn a_puncturing_round_into_the_ground_stays_two_hundred_frames() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1000, 0x1400));
    let ground = |punctures: bool| {
        let mut p = shot(me, a, b, 1, 0, 0);
        (p.rolling, p.punctures) = (false, punctures);
        p
    };
    sim.add_ammo(ground(false));
    sim.process_projectiles(0);
    assert!(sim.combat.projectiles.is_empty(), "no puncture: closed");
    sim.add_ammo(ground(true));
    sim.process_projectiles(0);
    let p = *sim.combat.projectiles.first().expect("the spent arrow");
    assert!(p.spent && !p.rolling && !p.missed, "flags 1");
    assert_eq!((p.slot, p.cur_time), (0, 0), "in its slot, clock reset");
    sim.units[me].health = 0;
    let held = sim.units[me].hold_frames;
    sim.hold_frames_tick();
    assert_eq!(sim.units[me].hold_frames, held + 1, "its dead shooter held");
    for f in 1..SPENT_LIFE {
        sim.process_projectiles(i64::from(f));
    }
    assert_eq!(sim.combat.projectiles.len(), 1, "199 frames on");
    assert_eq!(sim.combat.projectiles[0].cur_time, SPENT_LIFE - 1);
    sim.process_projectiles(i64::from(SPENT_LIFE));
    assert!(sim.combat.projectiles.is_empty(), "closed at 200");
}

/// **The death pool holds 100 records and evicts the oldest that does
/// not block** (item 3971, `Objects::add_death@00653b60`,
/// `docs/GOLDEN.md` §177). A hundred deaths fill it; the hundred-and-
/// first overwrites the oldest record, whose death leaves the list on
/// that frame's `DeathObj::inc_time`. A type that blocks while dead is
/// passed over for the next oldest, and of two deaths on one frame the
/// lower slot goes first.
#[test]
fn the_death_pool_evicts_the_oldest_record_that_does_not_block() {
    let (mut sim, ty) = at_war();
    let wreck = sim.add_unit_type(crate::UnitType {
        hits: 100,
        ..crate::UnitType::default()
    });
    sim.tables.unit_types[wreck].cols.unit_flags |= crate::ai_load::uflags::BLOCKS_WHILE_DEAD;
    let mut dead = Vec::new();
    // Slot 0 a wreck on frame 0; slots 1 and 2 two deaths on frame 1;
    // then one a frame to slot 99.
    let frames: Vec<i64> = [0, 1, 1].into_iter().chain(2..99).collect();
    assert_eq!(frames.len(), combat::DEATH_POOL);
    for (k, &f) in frames.iter().enumerate() {
        let t = if k == 0 { wreck } else { ty };
        let i = put(
            &mut sim,
            1,
            t,
            Pos::new(
                0x1000 + 0x200 * (k as i32 % 20),
                0x1000 + 0x200 * (k as i32 / 20),
            ),
        );
        sim.frame = f;
        sim.units[i].health = 0;
        sim.close_unit(i, 1, f);
        dead.push(sim.units[i].index);
    }
    assert_eq!(
        sim.combat.deaths.len(),
        combat::DEATH_POOL,
        "a hundred records"
    );
    let listed = |sim: &Sim| -> Vec<i32> { sim.combat.deaths.iter().map(|d| d.o).collect() };
    let kill = |sim: &mut Sim, f: i64| -> usize {
        let i = put(sim, 1, ty, Pos::new(0x3000, 0x3000));
        sim.frame = f;
        sim.units[i].health = 0;
        sim.close_unit(i, 1, f);
        sim.death_objs_inc_time();
        sim.combat
            .death_pool
            .iter()
            .position(|r| r.is_some_and(|r| r.o == i32::from(sim.units[i].index)))
            .unwrap()
    };
    // The wreck in slot 0 is the oldest and blocks: slot 1 goes.
    assert_eq!(kill(&mut sim, 200), 1, "the oldest that does not block");
    assert_eq!(sim.combat.deaths.len(), combat::DEATH_POOL);
    assert!(
        listed(&sim).contains(&i32::from(dead[0])),
        "the wreck stays"
    );
    assert!(
        !listed(&sim).contains(&i32::from(dead[1])),
        "slot 1's death leaves"
    );
    // Its twin of frame 1, slot 2, is now the oldest.
    assert_eq!(kill(&mut sim, 201), 2, "the tie's other half");
    assert!(!listed(&sim).contains(&i32::from(dead[2])));
    assert_eq!(kill(&mut sim, 202), 3, "then frame 2's");
    assert_eq!(
        sim.combat.deaths.len(),
        combat::DEATH_POOL,
        "never past a hundred"
    );
}

/// **A dead unit's number is held while its death object lives**
/// (`docs/COMBAT.md` §59). `DeathObj::inc_time` bumps the dead slot's
/// `hold_frames` every frame, `Objects::process_all` takes one off, and
/// `Objects::find_free` skips a dead number whose hold is not zero.
/// run146's arena-A hoplites are born into 9–11 for this reason, and
/// not into the dead 6–8.
#[test]
fn a_dead_unit_s_number_is_held_while_its_death_object_lives() {
    let (mut sim, ty) = at_war();
    let a = put(&mut sim, 1, ty, Pos::new(0x1000, 0x1000));
    put(&mut sim, 1, ty, Pos::new(0x1400, 0x1000));
    sim.units[a].health = 0;
    sim.hold_dead_slot(a);
    sim.combat.deaths.push(combat::Death {
        who: 1,
        o: 0,
        first_frame: 0,
        cur_anim: 17,
        life: None,
        figures: combat::Figures::one(None),
    });
    for _ in 0..40 {
        sim.tick();
        assert!(sim.units[a].hold_frames > 0, "the death object holds 1/0");
    }
    assert_eq!(
        sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
        Some(2),
        "a held number is not handed out"
    );
    // The death object's end: nothing bumps it, and `process_all`
    // takes one off a frame — from `Object::close`'s thirty, which the
    // bump and the take have held level (§59.3).
    assert_eq!(sim.units[a].hold_frames, Sim::CLOSE_HOLD);
    sim.combat.deaths.clear();
    for _ in 1..Sim::CLOSE_HOLD {
        sim.tick();
    }
    assert_eq!(sim.units[a].hold_frames, 1, "one frame short");
    assert_eq!(
        sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
        Some(2),
        "still held"
    );
    sim.tick();
    assert_eq!(sim.units[a].hold_frames, 0);
    assert_eq!(
        sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
        Some(0),
        "a number nothing holds is reused"
    );
}

/// **A death a round makes is bumped on its own frame** (item 3873,
/// `docs/GOLDEN.md` §156.1). `Objects::inc_time@0065db70` runs every
/// `Ammo::inc_time` and then every live `DeathObj::inc_time`, so the
/// record a killing round makes takes its first bump on the frame it
/// is made: `Object::close`'s thirty, plus one, held level from then
/// on by the bump and `process_all`'s take. run1492's Aircraft Carrier
/// `1/18`, sunk by a round on 1390, held 31 in theirs (run1496's packet
/// at 1539 reads `+0x32` 2), and the Jet Fighter born on tick 1539 took
/// 41. Made to fail with the death bump back ahead of the ammo walk.
#[test]
fn a_death_a_round_makes_is_bumped_on_its_own_frame() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1400, 0x1000));
    let foe = put(&mut sim, 1, ty, b);
    sim.units[foe].health = 1;
    let mut p = shot(me, a, b, 1, 0, 0);
    (p.target, p.rolling) = (Some(Obj::Unit(foe)), false);
    sim.add_ammo(p);
    sim.tick();
    assert!(!sim.units[foe].alive(), "the round kills it");
    assert_eq!(sim.combat.deaths.len(), 1, "its death object");
    assert_eq!(
        sim.units[foe].hold_frames,
        Sim::CLOSE_HOLD + 1,
        "the record's first bump is on its own frame"
    );
    sim.tick();
    assert_eq!(
        sim.units[foe].hold_frames,
        Sim::CLOSE_HOLD + 1,
        "held level"
    );
}

/// **A crew's number takes one bump per figure's record** (item 3784,
/// `docs/GOLDEN.md` §149.10). `Objects::kill_guy` makes one `DeathObj`
/// per guy and `DeathObj::inc_time@008d5240` bumps the number's
/// `hold_frames` on each, while `Objects::process_all` takes one off a
/// frame: a four-figure crew nets +3 while all four live, and the hold
/// climbs less as each record ends. run1413's Bombard `2/23` held its
/// number through 1568 this way, where one bump a frame freed it.
#[test]
fn a_crew_s_number_takes_one_bump_per_figure_s_record() {
    let (mut sim, ty) = at_war();
    let a = put(&mut sim, 1, ty, Pos::new(0x1000, 0x1000));
    put(&mut sim, 1, ty, Pos::new(0x1400, 0x1000));
    sim.units[a].health = 0;
    sim.hold_dead_slot(a);
    let mut figures = combat::Figures::one(None);
    figures.n = 4;
    figures.life[..4].copy_from_slice(&[3, 5, 8, 8]);
    sim.combat.deaths.push(combat::Death {
        who: 1,
        o: 0,
        first_frame: sim.frame,
        cur_anim: 17,
        life: Some(8),
        figures,
    });
    let mut nets = Vec::new();
    for _ in 0..10 {
        let held = sim.units[a].hold_frames;
        sim.tick();
        nets.push(sim.units[a].hold_frames - held);
    }
    assert_eq!(
        nets,
        [3, 3, 3, 2, 2, 1, 1, 1, -1, -1],
        "four records, then three, then two; then none and the take alone"
    );
    assert!(sim.combat.deaths.is_empty(), "culled with its last record");
}

/// **A rolled shot comes down on the first step its arc is not above
/// the ground** (`docs/COMBAT.md` §46.1), at the point on its line
/// that step names, and damages what stands there — not before.
#[test]
fn a_rolled_shot_comes_down_on_the_first_step_the_ground_meets_it() {
    let (mut sim, ty) = at_war();
    let side = (sim.world.corner_stride() * sim.world.corner_stride()) as usize;
    assert!(
        sim.world.set_corner_grid(vec![150_000_000; side]),
        "a square world's grid"
    );
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let (launch, landing) = (Pos::new(0x1000, 0x1000), Pos::new(0x1000 + 700, 0x1000));
    let (sz, ez, total) = (400, 200, 7);
    let v1z = combat::arc_v1z(sz, ez, total);
    let ground = crate::single::Single::from_i32(150);
    let down = (total..=3 * total)
        .find(|&t| !combat::arc_z(v1z, sz, t).gt(ground))
        .expect("the arc meets flat ground at 150 inside three flight times");
    assert!(
        down > total,
        "the shot flies past its landing point first: {down}"
    );
    let at = combat::arc_point(launch, landing, down, total);
    let foe = put(&mut sim, 1, ty, at);
    let full = sim.units[foe].health;
    sim.add_ammo(shot(me, launch, landing, total, sz, ez));
    for step in 1..=down {
        sim.process_projectiles(i64::from(step));
        let hit = sim.units[foe].health < full;
        assert_eq!(
            hit,
            step == down,
            "step {step}: the shot lands on {down} and only then"
        );
    }
    assert!(sim.combat.projectiles.is_empty());
    assert_eq!(
        sim.ground_inexact, 0,
        "a grid of whole numbers is every corner's own single"
    );
}

/// A three-figure chain, `a → b → c`, on player 0.
fn chain3(sim: &mut Sim, ty: usize) -> [usize; 3] {
    let a = put(sim, 0, ty, Pos::new(0x1000, 0x1000));
    let b = put(sim, 0, ty, Pos::new(0x1030, 0x1000));
    let c = put(sim, 0, ty, Pos::new(0x1060, 0x1000));
    for (f, up, down) in [
        (a, None, Some(b)),
        (b, Some(a), Some(c)),
        (c, Some(b), None),
    ] {
        sim.units[f].captain = up.is_none();
        sim.units[f].o_up = up;
        sim.units[f].o_down = down;
        sim.units[f].combat.captain = i32::from(sim.units[a].index);
    }
    [a, b, c]
}

fn links(sim: &Sim, f: usize) -> (Option<usize>, Option<usize>) {
    (sim.units[f].o_up, sim.units[f].o_down)
}

/// **A dying head hands the squad down, and goes to the tail**
/// (`docs/COMBAT.md` §47.2, `Unit::close@0060ee50`'s relink). run112's
/// `1/6` dies on 743 and the dump's next block carries `1/7` with
/// `o_up -1 o_down 8`, a chain of 7 → 8 → 6. The new head is a captain
/// from that frame (`UnitData::is_captain` is `o_up < 0`), and every
/// figure's captain now reads it, the dead slot's included, because
/// `get_captain` walks `o_up`.
///
/// Made to fail on purpose: with the relink skipped, the golden
/// widening's `o_up` row parts on block 744 (`1/7` ours 6, theirs −1)
/// and chapter two's word falls back to 762.
#[test]
fn a_dead_captain_hands_the_squad_to_the_figure_below() {
    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    sim.units[a].health = 0;
    sim.relink_squad(a);
    assert!(sim.units[b].captain, "the figure below is the captain now");
    assert_eq!(links(&sim, b), (None, Some(c)));
    assert_eq!(links(&sim, c), (Some(b), Some(a)));
    assert_eq!(links(&sim, a), (Some(c), None), "the dead head is the tail");
    let bi = i32::from(sim.units[b].index);
    for f in [a, b, c] {
        assert_eq!(sim.units[f].combat.captain, bi);
    }
    assert_eq!(sim.squad_captain(c), b);
}

/// **A dying tail changes nothing its captain can see.** The unlink
/// sets `above.o_down = −1` and the append puts it straight back:
/// run112's `0/10` still carries `o_down 11` on every block after
/// `0/11` dies on 729. And a dying middle figure is moved behind the
/// one below it, so the chain the living walk is unbroken.
#[test]
fn a_dead_tail_stays_linked_and_a_dead_middle_goes_behind() {
    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    sim.units[c].health = 0;
    sim.relink_squad(c);
    assert_eq!(links(&sim, b), (Some(a), Some(c)));
    assert_eq!(links(&sim, c), (Some(b), None));
    assert!(sim.units[a].captain && !sim.units[b].captain);

    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    sim.units[b].health = 0;
    sim.relink_squad(b);
    assert_eq!(links(&sim, a), (None, Some(c)));
    assert_eq!(links(&sim, c), (Some(a), Some(b)));
    assert_eq!(links(&sim, b), (Some(c), None));
    assert!(sim.units[a].captain, "a middle death moves no captaincy");
}

/// **A gatherer killed by damage leaves its building's chain on the
/// frame it dies** (`Unit::close@0060ee50`'s `close_orders(this, 1)`
/// at `60fa1c`, whose `kill_current_order` gather arm calls
/// `remove_gatherer`; item 1248, `docs/GOLDEN.md` §52). run466's who=1
/// Citizen `1/6`, walking to its tile at the head of `1/2001`'s chain,
/// dies on 1487, and block 1488 prints `gather_down` 2. The walk in
/// front of the gather order is killed with it, and a move killed by a
/// close hands no facing back. Made to fail by taking the close out of
/// the death path: the dead citizen stays at the chain's head.
#[test]
fn a_gatherer_killed_by_damage_leaves_its_building_s_chain() {
    use crate::orders::{MoveKind, QueuePos, Worker};
    let (mut sim, ty) = at_war();
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let camp = sim.add_building(0, Pos::new(0x1000, 0x1000), 1);
    sim.buildings[camp].ty = Some(sim.add_build_type(crate::build::BuildType {
        flags: crate::build::flags::GATHERER,
        ..crate::build::BuildType::default()
    }));
    let stays = put(&mut sim, 0, citizen, Pos::new(0x1100, 0x1000));
    let dies = put(&mut sim, 0, citizen, Pos::new(0x1200, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    sim.add_gather_order(stays, camp, QueuePos::New, false);
    sim.add_gather_order(dies, camp, QueuePos::New, false);
    sim.add_move_order(
        dies,
        Pos::new(0x1800, 0x1000),
        MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    assert_eq!(
        sim.buildings[camp].gatherers,
        vec![dies, stays],
        "newest first"
    );
    let hit = combat::Sixteenths { whole: 50, frac: 0 };
    let taken = sim.take_damage(Obj::Unit(dies), hit, Obj::Unit(foe), 1487);
    assert!(matches!(taken, Taken::Died { .. }));
    assert_eq!(
        sim.buildings[camp].gatherers,
        vec![stays],
        "the dead leave the chain"
    );
    assert!(
        sim.units[dies].orders.is_empty(),
        "and every order goes with them"
    );
}

/// The relink is on the death path itself: a head killed by damage
/// leaves its squad led by the next figure, with nothing called by
/// hand. A dead head over a dead figure is the whole-squad arm and
/// promotes nobody.
#[test]
fn a_captain_killed_by_damage_is_succeeded() {
    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    sim.units[a].health = 1;
    let hit = combat::Sixteenths { whole: 50, frac: 0 };
    let taken = sim.take_damage(Obj::Unit(a), hit, Obj::Unit(foe), 10);
    assert!(matches!(taken, Taken::Died { .. }));
    assert!(sim.units[b].captain);
    assert_eq!(sim.squad_captain(c), b);

    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    for f in [c, b] {
        sim.units[f].health = 0;
        sim.relink_squad(f);
    }
    sim.units[a].health = 0;
    let before = sim.units.iter().map(|u| u.captain).collect::<Vec<_>>();
    sim.relink_squad(a);
    let after = sim.units.iter().map(|u| u.captain).collect::<Vec<_>>();
    assert_eq!(
        before, after,
        "nobody alive below the head, nobody promoted"
    );
}

/// **A dead tail of a live squad holds its number; the squad's last
/// death frees the whole tail** (`Unit::close@0060ee50:292`-`316`,
/// `Objects::find_free@0065ad60`'s `+0x8e` term; `docs/AI.md` §324.2).
/// run1254b's who=1 Arquebusier `1/49` dies on 3219 and stays the tail
/// of `1/48 → 1/50 → 1/49`; the squad trained on 3268 skips it and takes
/// 46, 47 and 139, where 46 and 47 were the dead tail of `1/45`'s squad,
/// unlinked when `1/45` died last on 3254. Holds are spent here so the
/// link is the only term asked.
#[test]
fn a_dead_tail_holds_its_number_until_its_squad_is_gone() {
    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    let free = |s: &mut Sim| s.find_free(0, crate::UNIT_BASE, crate::BUILD_BASE);
    sim.units[c].health = 0;
    sim.relink_squad(c);
    sim.units[c].hold_frames = 0;
    assert_eq!(links(&sim, c), (Some(b), None), "linked under the living");
    assert_ne!(
        free(&mut sim),
        Some(sim.units[c].index),
        "a linked dead tail is no captain to the reuse test"
    );
    for f in [b, a] {
        sim.units[f].health = 0;
        sim.relink_squad(f);
        sim.units[f].hold_frames = 0;
    }
    for f in [a, b, c] {
        assert_eq!(
            links(&sim, f),
            (None, None),
            "the last death clears the tail"
        );
    }
    assert_eq!(free(&mut sim), Some(sim.units[a].index));
}

/// **A squad leaves the muster when its last live figure dies, and
/// not before** (`Unit::close@0060ee50`, `0060f3db`; `docs/AI.md`
/// §232). A head over a live figure hands the squad down and keeps the
/// count; the last head, over the dead, takes `num_units`, `control`
/// and `active` off. run867's who=1 kept three shot-down Jet Fighters
/// counted, and its next Fighter finished on 5511 against 5499.
#[test]
fn a_squad_leaves_the_muster_with_its_last_figure() {
    let (mut sim, ty) = at_war();
    sim.tables.unit_types[ty].price.pop = 1;
    let [a, b, c] = chain3(&mut sim, ty);
    sim.track_unit_type(0, ty, 1);
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let hit = combat::Sixteenths {
        whole: 500,
        frac: 0,
    };
    let count = |s: &Sim| {
        (
            s.roster.muster[0].by_type[ty],
            s.roster.muster[0].control,
            s.ai[0].census.active,
        )
    };
    for f in [a, b] {
        let taken = sim.take_damage(Obj::Unit(f), hit, Obj::Unit(foe), 10);
        assert!(matches!(taken, Taken::Died { .. }));
        assert_eq!(
            count(&sim),
            (1, 1, 1),
            "a live figure below holds the squad"
        );
    }
    let taken = sim.take_damage(Obj::Unit(c), hit, Obj::Unit(foe), 10);
    assert!(matches!(taken, Taken::Died { .. }));
    assert_eq!(
        count(&sim),
        (0, 0, 0),
        "the last figure takes the squad off"
    );

    // A lone unit of a type with no population is never counted.
    sim.tables.unit_types[ty].price.pop = 0;
    let lone = put(&mut sim, 0, ty, Pos::new(0x1200, 0x1000));
    sim.units[lone].captain = true;
    sim.take_damage(Obj::Unit(lone), hit, Obj::Unit(foe), 10);
    assert_eq!(count(&sim), (0, 0, 0));
}

/// **The lead is the target's first figure's `avg_speed`, along its
/// angle** (`docs/COMBAT.md` §47.4, `Ammo::init@0067bbf0`,
/// `0067ceb4`-`0067cec7`), and not the unit's per-frame speed. Two
/// copies of one sim fire the same shot, the draws identical, and
/// differ only in the target figure's average. The landings differ by
/// exactly `sinx`/`cosx` of that average times the flight time,
/// whatever the unit's `speed` says.
#[test]
fn the_lead_is_the_first_figure_s_average_speed() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    let facing = crate::movement::Angle(723_976_192);
    sim.add_move_order(
        foe,
        Pos::new(0x1800, 0x1000),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::First,
        false,
    );
    {
        let m = &mut sim.units[foe].movement;
        m.dest = Some(Pos::new(0x1800, 0x1000));
        m.facing = facing;
        m.heading = facing;
        m.speed = 24;
    }
    let fire = |avg: i32| {
        let mut s = sim.clone();
        s.units[foe].movement.body.avg_speed = avg;
        s.fire_ammo(
            Obj::Unit(me),
            Obj::Unit(foe),
            Angle(0),
            0,
            Pos::new(0x1000, 0x1000),
            0,
        );
        let p = *s.combat.projectiles.last().expect("a shot");
        (p.landing, p.total_time)
    };
    let (still, t) = fire(0);
    let (moving, t2) = fire(9);
    assert_eq!(t, t2, "the flight time is taken before the lead");
    assert_eq!(
        (moving.x - still.x, moving.y - still.y),
        (
            crate::movement::sin_component(facing, 9) * t,
            -crate::movement::cos_component(facing, 9) * t
        ),
    );
}

/// **A rolled round is aimed at its target's own `z`** (item 3593,
/// `docs/AI.md` §315.4). `Ammo::init` reads `ez` from the target's
/// `ObjectData +0xc` (`0x67c93a`), which every writer fills through
/// `find_tcoord_z(·, 1)`: a height under the sea is 0. The Bantu pair's
/// Citizen `0/11` stood on a shore tile at −22, so its rolled javelins
/// were aimed at 75, not 53, and rolled on a frame longer. Made to fail
/// by aiming at [`crate::World::tile_z`].
#[test]
fn a_rolled_round_is_aimed_at_its_target_s_own_clamped_z() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    let aim = |tile_z: i32| {
        let mut s = sim.clone();
        let t = s.units[foe].pos.tile();
        s.world.set_tile_z(t, tile_z);
        s.fire_ammo_pub(
            Obj::Unit(me),
            Obj::Unit(foe),
            Angle(0),
            0,
            Pos::new(0x1000, 0x1000),
            0,
            0,
            false,
            false,
        );
        let p = *s.combat.projectiles.last().expect("a shot");
        assert!(p.rolling, "a shot at a land unit rolls");
        p.ez
    };
    assert_eq!(aim(-22), 0x4b, "under the sea is 0, plus the roll's 75");
    assert_eq!(aim(17), 17 + 0x4b, "above it, the tile's own height");
}

/// **A missile's round is not led and never rolls, and flies its
/// spline's count** (item 1625, `docs/AI.md` §174). `Ammo::init`'s
/// `local_30` — the ammo's `missile="1"` — takes the round past the
/// rolling flag and the `0x4b` (`Ammo::init@0067bbf0:355`–`:360`) and
/// past the lead (`:798`), and times it by the spline
/// ([`combat::missile_flight_time`]). The same shot without the flag
/// at a fleeing land unit is led and rolls. Made to fail with each
/// `missile` conjunct dropped.
#[test]
fn a_missile_s_round_is_not_led_and_does_not_roll() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    sim.add_move_order(
        foe,
        Pos::new(0x1300, 0x0800),
        crate::orders::MoveKind::FleeTo,
        crate::orders::QueuePos::First,
        false,
    );
    sim.units[foe].movement.heading = crate::movement::Angle(117_506_048);
    let fire = |missile: bool, avg: i32| {
        let mut s = sim.clone();
        s.units[foe].movement.body.avg_speed = avg;
        s.fire_ammo_pub(
            Obj::Unit(me),
            Obj::Unit(foe),
            Angle(0),
            0,
            Pos::new(0x1000, 0x1000),
            0,
            0,
            false,
            missile,
        );
        *s.combat.projectiles.last().expect("a shot")
    };
    let (shell, missile) = (fire(false, 23), fire(true, 23));
    assert!(shell.rolling, "a shell at a land unit rolls");
    assert!(!missile.rolling, "a missile does not");
    assert_ne!(shell.landing, fire(false, 0).landing, "a shell is led");
    assert_eq!(missile.landing, fire(true, 0).landing, "a missile is not");
    let d = i64::from(sim.profile(Obj::Unit(me)).proj_speed);
    assert_eq!(
        missile.total_time,
        combat::missile_flight_time(
            Pos::new(0x1000, 0x1000),
            0,
            missile.landing,
            missile.ez,
            Some(combat::Lead {
                angle: missile.angle,
                pitch: Single::from_bits(0x4234_0000),
                r: sim.attack_dist(Obj::Unit(me), Obj::Unit(foe)) / 2,
            }),
            d
        ),
        "a land shooter's spline, half the distance out at 45° up (item 3816)"
    );
}

/// **The lead's gate is the target's order, and its angle is
/// `UnitData +0x50`** (item 1081, `docs/COMBAT.md` §74). `Ammo::init`
/// asks `UnitData::order_type` of `is_move@0046f050`, then of
/// `is_air@0046f000` (`67ce62`-`67ce99`), and leads along `angle`
/// (`67ceba`), this crate's `heading`. run373's `0/1` on 5024 fled
/// between two legs, a `FLEE_TO` head with no `movement.dest`: the
/// original led `1/11`'s round onto it and this crate, asking
/// `movement.dest`, missed. A target with a destination and no move
/// at its head is not led.
#[test]
fn the_lead_asks_the_target_s_order_and_leads_along_its_angle() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    let heading = crate::movement::Angle(117_506_048);
    let facing = crate::movement::Angle(550_174_720);
    let fire = |s: &Sim, avg: i32| {
        let mut s = s.clone();
        s.units[foe].movement.body.avg_speed = avg;
        s.fire_ammo(
            Obj::Unit(me),
            Obj::Unit(foe),
            Angle(0),
            0,
            Pos::new(0x1000, 0x1000),
            0,
        );
        let p = *s.combat.projectiles.last().expect("a shot");
        (p.landing, p.total_time)
    };
    // A flight between two legs: the head is `FLEE_TO`, the body has no
    // destination, and the figure still faces the old leg.
    let mut fleeing = sim.clone();
    fleeing.add_move_order(
        foe,
        Pos::new(0x1300, 0x0800),
        crate::orders::MoveKind::FleeTo,
        crate::orders::QueuePos::First,
        false,
    );
    {
        let m = &mut fleeing.units[foe].movement;
        m.dest = None;
        m.heading = heading;
        m.facing = facing;
    }
    let (still, t) = fire(&fleeing, 0);
    let (moving, _) = fire(&fleeing, 23);
    assert_eq!(
        (moving.x - still.x, moving.y - still.y),
        (
            crate::movement::sin_component(heading, 23) * t,
            -crate::movement::cos_component(heading, 23) * t
        ),
        "a fleeing target is led along its angle, whatever its body's \
             destination and its figure's facing"
    );
    // A destination with no move at the head: not led.
    let mut standing = sim.clone();
    {
        let m = &mut standing.units[foe].movement;
        m.dest = Some(Pos::new(0x1800, 0x1000));
        m.heading = heading;
    }
    assert_eq!(
        fire(&standing, 23).0,
        fire(&standing, 0).0,
        "no move and no air order at the head, no lead"
    );
}

/// **A landed shot strikes at its own launch-to-landing bearing**
/// (`docs/COMBAT.md` §47.3): `find_angle(ex − sx, ey − sy)`, whatever
/// the target did while the shot flew. run112's `0/9` shot `1/7` on
/// 766, `1/7` walked on, and the original struck at the shot's bearing
/// on flank level 2 where the target's current position gives 1.
#[test]
fn a_shot_strikes_at_its_own_bearing() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
    sim.fire_ammo(
        Obj::Unit(me),
        Obj::Unit(foe),
        Angle(0),
        0,
        Pos::new(0x1000, 0x1000),
        0,
    );
    let mut p = *sim.combat.projectiles.last().expect("a shot");
    p.launch = Pos::new(964, 8180);
    p.landing = Pos::new(1717, 8421);
    assert_eq!(p.bearing(), crate::movement::find_angle(753, 241));
    assert_eq!(
        combat::flank_level(crate::movement::Angle(724_631_552), p.bearing()),
        Some(2),
        "run112's own numbers: 1/7's facing on 770 and 0/9's shot"
    );
}

/// **A hit marks the whole squad in danger, and every figure's guy 0**
/// (item 530, `docs/ORDERS.md` §22). `Object::take_damage@00652020`
/// calls `Unit::set_in_danger(this, 0)` for any unit hit by anything
/// but attrition. That climbs to the captain and walks `o_down`, so
/// hitting the tail marks the head. The attacker is not the victim's
/// squad and is not marked here.
///
/// Made to fail on purpose: with the call out of `take_damage`, golden
/// chapter one's `in_danger` row parts on 617 on all six hoplites and
/// its word falls back to 774.
#[test]
fn a_hit_on_the_tail_marks_the_whole_squad_in_danger() {
    let (mut sim, ty) = at_war();
    let [a, b, c] = chain3(&mut sim, ty);
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let hit = combat::Sixteenths { whole: 5, frac: 0 };
    let taken = sim.take_damage(Obj::Unit(c), hit, Obj::Unit(foe), 10);
    assert!(matches!(taken, Taken::Alive { .. }));
    for f in [a, b, c] {
        assert!(sim.units[f].in_danger, "figure {f} was not marked");
        assert!(
            sim.units[f].guy_flag_0x20,
            "figure {f}'s guy 0 was not marked"
        );
    }
    assert!(
        !sim.units[foe].in_danger,
        "the attacker is not the victim's squad"
    );
}

/// **A hit stamps the struck object's owner's `frame_attacked`, at
/// difficulty below 2** (item 729, `docs/AI.md` §71).
/// `Object::take_damage@00652020:67-71` writes
/// `leaders[owner].frame_attacked = frame` for any combat hit, and
/// `Army::find_target`'s difficulty gate keeps a leader out for 7,200
/// frames after it. This crate wrote the field only from `find_target`,
/// so Great Lakes' human stood at 8186 where the original's stood at
/// 10233, and on 15608 the gate reached the coin the original skips.
///
/// Made to fail on purpose: with the stamp out of `take_damage`, the
/// first assertion reads 0.
#[test]
fn a_hit_stamps_the_struck_owner_s_frame_attacked_below_difficulty_two() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
    let hit = combat::Sixteenths { whole: 1, frac: 0 };
    sim.lobby.difficulty = 1;
    sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10_233);
    assert_eq!(sim.ai[0].frame_attacked, 10_233, "the struck owner's stamp");
    assert_eq!(sim.ai[1].frame_attacked, 0, "never the attacker's");
    sim.lobby.difficulty = 2;
    sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10_300);
    assert_eq!(
        sim.ai[0].frame_attacked, 10_233,
        "difficulty 2 and above: no stamp"
    );
}

/// **An attacker marks its own squad and its target's on every
/// `Unit::work`** (`0060d180:283-313`), whether or not it strikes:
/// the mark is above the dispatch, keyed on the action being an
/// `ATTACK` with a target. A target out of reach is still marked.
#[test]
fn an_attack_order_marks_both_squads_on_its_own_work() {
    let (mut sim, ty) = at_war();
    let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
    let foe = put(&mut sim, 1, ty, Pos::new(0x6000, 0x1000));
    sim.add_attack_order(
        me,
        Obj::Unit(foe),
        crate::orders::QueuePos::First,
        false,
        false,
    );
    assert!(!sim.units[me].in_danger && !sim.units[foe].in_danger);
    sim.work(me, 1);
    assert!(sim.units[me].in_danger, "the attacker marks itself");
    assert!(sim.units[foe].in_danger, "and the unit it names");
}

/// **A reloading unit handed a fresh attack does not turn: it asks for
/// the idle** (item 530, `Unit::fight@005fd4d0:100-127`). The first
/// reading had it turn toward the target and return, and golden
/// chapter one's `1/7` on 774 says otherwise: heading unchanged, and
/// one `Guy::set_anim+0x97a` under `Unit::fight+0x169`, which is the
/// chapter's word. Guy 0 already aimed at the target asks for nothing.
#[test]
fn a_reloading_unit_asks_for_the_idle_and_does_not_turn() {
    for aimed in [false, true] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x6000, 0x3000));
        let mut g = crate::anim::Guy::fresh(0);
        g.anim = crate::anim::WALK;
        g.aim = aimed.then_some(Obj::Unit(foe));
        sim.units[me].guys = vec![g];
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        sim.units[me].combat.recharging = 13;
        let heading = sim.units[me].movement.heading;
        sim.work(me, 1);
        assert_eq!(
            sim.units[me].movement.heading, heading,
            "aimed {aimed}: a reloading unit does not turn"
        );
        let asked = sim.units[me].guys[0].anim != crate::anim::WALK;
        assert_eq!(asked, !aimed, "aimed {aimed}: the idle request");
    }
}

/// **The attack-move looks around one frame in fifteen, phased by
/// `o`, and stacks what it finds `QUEUE_FIRST`** (item 530,
/// `Unit::do_attack_to@005f2320`). This crate's attack-move never
/// looked, so golden chapter one's `1/7` walked past `0/8` on 773
/// where the original's took an `ATTACK` above its march.
#[test]
fn the_attack_move_looks_around_one_frame_in_fifteen() {
    for (frame, looks) in [(15, true), (16, false)] {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1200, 0x1200));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1200));
        let dest = Pos::new(0x4000, 0x1200);
        sim.units[me].index = 0;
        sim.add_move_order(
            me,
            dest,
            crate::orders::MoveKind::AttackTo,
            crate::orders::QueuePos::New,
            true,
        );
        let dest = sim.units[me].orders[0]
            .move_dest()
            .expect("the attack-move");
        sim.do_attack_to_tail(me, frame, dest);
        let front = sim.units[me]
            .orders
            .front()
            .map(crate::orders::Order::index);
        assert_eq!(
            front == Some(crate::orders::index::ATTACK),
            looks,
            "frame {frame}: {:?}",
            sim.units[me].orders
        );
        if looks {
            assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(foe)));
            assert_eq!(
                sim.units[me].orders.back().map(crate::orders::Order::index),
                Some(crate::orders::index::ATTACK_TO),
                "the attack-move stays under the attack"
            );
        }
    }
}

#[test]
fn a_strike_reseats_tracked_crew_before_requesting_their_attack() {
    let (mut sim, ty) = at_war();
    let at = Pos::new(4632, 4632);
    let me = put(&mut sim, 0, ty, at);
    let foe = put(&mut sim, 1, ty, Pos::new(at.x + 192, at.y));
    sim.units[me].movement.heading = Angle::NORTH;
    sim.units[me].movement.facing = Angle::NORTH;
    sim.units[me].guys.push(crate::anim::Guy::fresh(-1));
    let mut crew = sim.units[me].guys[0];
    crew.follow = Some(crate::anim::Follow {
        body: crate::movement::Body::at(Pos::new(at.x + 48, at.y)),
        des: Pos::new(at.x + 48, at.y),
        facing: Angle::NORTH,
        des_angle: Angle::NORTH,
        track: (48, 0),
    });
    sim.units[me].guys.push(crew);
    sim.fight(me, Obj::Unit(foe), 0);
    let crew = sim.units[me].guys[1];
    let follow = crew.follow.unwrap();
    assert_eq!(sim.units[me].movement.heading, Angle::EAST);
    assert_eq!(sim.units[me].movement.facing, Angle::NORTH);
    assert_eq!(
        sim.units[me].guys[0].pending_attack, 1,
        "the gun still owes its turn"
    );
    assert_eq!(follow.body.pos, follow.des);
    assert_eq!(follow.facing, follow.des_angle);
    assert_eq!(crew.pending_attack, 0, "the crew can attack immediately");
    assert_eq!(crate::anim::category(crew.anim), 12);
}

#[test]
fn packed_ai_siege_takes_its_army_target_before_the_attack_move_search() {
    use crate::orders::{Body, MoveKind, QueuePos};
    for case in [
        "inside",
        "boundary",
        "human",
        "unpacked",
        "not siege",
        "easiest",
        "no target",
        "hold fire",
    ] {
        let (mut sim, foe_ty) = at_war();
        sim.cities_state.nation[0].human = case == "human";
        sim.lobby.difficulty = if case == "easiest" { 0 } else { 5 };
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 4,
                packs: true,
                siege: case != "not siege",
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(0x1200, 0x1200);
        let me = put(&mut sim, 0, ty, at);
        sim.units[me].combat.packed = case != "unpacked";
        if case == "hold fire" {
            sim.units[me].combat.stance = combat::Stance::HoldFire;
        }
        let limit = sim.tables.tuning.unit_respond_range * 0x180;
        let distance = if case == "boundary" {
            limit
        } else {
            limit - 48
        };
        let foe = put(&mut sim, 1, foe_ty, Pos::new(at.x + distance + 48, at.y));
        assert_eq!(sim.attack_dist(Obj::Unit(me), Obj::Unit(foe)), distance);
        let army = sim.init_army(0, None);
        sim.army_add_unit(0, army, me);
        sim.armies[0].list[army].target = (case != "no target").then_some(Obj::Unit(foe));
        let dest = Pos::new(0x6000, 0x1200);
        sim.add_move_order(me, dest, MoveKind::AttackTo, QueuePos::New, true);
        let dest = sim.units[me].orders[0].move_dest().unwrap();
        sim.do_attack_to_tail(me, 15, dest);
        let mandatory = matches!(sim.units[me].orders.front().map(|o| o.body),
            Some(Body::Attack(_)) if sim.units[me].combat.mandatory);
        assert_eq!(
            mandatory,
            case == "inside",
            "{case}: {:?}",
            sim.units[me].orders
        );
        if mandatory {
            assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(foe)));
            assert_eq!(
                sim.units[me].orders.back().unwrap().index(),
                crate::orders::index::ATTACK_TO
            );
            assert_eq!(
                sim.units[foe].combat.targeted, 0,
                "the normal search was bypassed"
            );
        }
    }
}

/// A computer's packed siege engine of `army`, idle at (0x1200,
/// 0x1200), and a foe `distance` east of it as the army's target.
fn idle_packed_engine(distance: i32, in_army: bool) -> (Sim, usize, usize) {
    let (mut sim, foe_ty) = at_war();
    sim.cities_state.nation[0].human = false;
    sim.lobby.difficulty = 5;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 40,
            max_range: 4,
            packs: true,
            siege: true,
            combat_role: true,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let at = Pos::new(0x1200, 0x1200);
    let me = put(&mut sim, 0, ty, at);
    sim.units[me].combat.packed = true;
    let foe = put(&mut sim, 1, foe_ty, Pos::new(at.x + distance + 48, at.y));
    if in_army {
        let army = sim.init_army(0, None);
        sim.army_add_unit(0, army, me);
        sim.armies[0].list[army].target = Some(Obj::Unit(foe));
    }
    (sim, me, foe)
}

/// **The idle think takes the army's target first** (item 2864,
/// `docs/AI.md` §238): `think_attack`'s `find_melee_target` reaches
/// `find_nearby_target@00648da0:198`'s shortcut, so a packed computer
/// siege engine of an army with a target is given it `QUEUE_FIRST`
/// and **mandatory** — the peace pair's MLRS `1/124` on 6820. Out of
/// an army the same engine searches, and a packed computer engine must
/// reach what it takes, so at this distance it takes nothing.
#[test]
fn an_idle_packed_engine_takes_its_army_target_mandatory() {
    for in_army in [true, false] {
        let limit = at_war().0.tables.tuning.unit_respond_range * 0x180;
        let (mut sim, me, foe) = idle_packed_engine(limit - 48, in_army);
        sim.units[me].idle = 1;
        sim.think(me, 1);
        let attacking = sim.order_type(me) == crate::orders::index::ATTACK
            && sim.units[me].combat.target == Some(Obj::Unit(foe));
        assert_eq!(
            attacking && sim.units[me].combat.mandatory,
            in_army,
            "in_army {in_army}: {:?}",
            sim.units[me].orders
        );
    }
}

/// **`find_new_target`'s and `Group::target_opportunity`'s searches
/// take the shortcut too** (item 3654, `docs/AI.md` §336.2): it sits
/// inside `find_nearby_target@00648da0:198`–`222`, ahead of the add
/// flag, so every `find_melee_target` caller reaches it. The Lakota
/// Gunpowder pair's decoy Bombards `1/180` and `1/185` took the army's
/// `0/2000` there `mandatory 1` on 4477, and `1/185` walked its ring.
/// Out of an army the engine searches, and at this distance takes
/// nothing.
#[test]
fn find_new_target_and_target_opportunity_take_the_army_target_mandatory() {
    for (arm, in_army) in [
        ("find_new_target", true),
        ("find_new_target", false),
        ("target_opportunity", true),
        ("target_opportunity", false),
    ] {
        let limit = at_war().0.tables.tuning.unit_respond_range * 0x180;
        let (mut sim, me, foe) = idle_packed_engine(limit - 48, in_army);
        if arm == "find_new_target" {
            sim.find_new_target(me, false);
        } else {
            sim.find_melee_target_added_in(me, limit * 2);
        }
        let attacking = sim.order_type(me) == crate::orders::index::ATTACK
            && sim.units[me].combat.target == Some(Obj::Unit(foe));
        assert_eq!(
            attacking && sim.units[me].combat.mandatory,
            in_army,
            "{arm}, in_army {in_army}: {:?}",
            sim.units[me].orders
        );
    }
}

/// **`Group::action_attack`'s member search takes the shortcut too**
/// (item 3690, `docs/AI.md` §348.2): its `find_melee_target(maxd,
/// &ww, 0, 0, word)` reaches `find_nearby_target`'s head, whose add
/// (`00649263`, theirs' `find_nearby_target+0x4c8`) is ahead of the
/// add flag. The American Industrial pair's packed Artillery `1/131`,
/// halted by its group's attack on the Citizen `0/10`, took the army's
/// `0/2000` `mandatory 1` on 5678. Out of an army the search finds
/// nothing a packed engine can reach, and it is ordered on the group's
/// own target. The member's own `QUEUE_NEW` add follows the shortcut's
/// here as in theirs (`action_attack+0xeb1`); whether `mandatory`
/// survives it is the army group's insert, which this bare fixture
/// does not stage — run1356's widening pins that value on block 5679.
#[test]
fn group_action_attack_s_member_search_takes_the_army_target() {
    for in_army in [true, false] {
        let limit = at_war().0.tables.tuning.unit_respond_range * 0x180;
        let (mut sim, me, foe) = idle_packed_engine(limit - 48, in_army);
        let foe_ty = sim.units[foe].ty.unwrap();
        let at = sim.units[me].pos;
        let blocker = put(&mut sim, 1, foe_ty, Pos::new(at.x, at.y + 600));
        let g = match sim.army_of(me) {
            Some(slot) => sim.army_group(0, slot),
            None => crate::group::Group {
                who: 0,
                army: None,
                pushed: None,
                list: vec![me],
            },
        };
        sim.group_action_attack(
            &g,
            Obj::Unit(blocker),
            false,
            crate::orders::QueuePos::First,
            0,
        );
        let want = if in_army { foe } else { blocker };
        assert_eq!(
            sim.units[me].combat.target,
            Some(Obj::Unit(want)),
            "in_army {in_army}: {:?}",
            sim.units[me].orders
        );
    }
}

/// **An idle melee-and-ranged type searches at its second range**
/// (`Unit::think_attack@005f5a80:167`–`174`, item 3383, `docs/AI.md`
/// §291): a type whose flags carry `0x400` has `max_range` 0 and its
/// range in `second_max_range`; `think_attack` swaps the second in for
/// its `find_melee_target`, whose radius at a `DEFENSIVE` stance is
/// `max(r × 0xc0, unit_defensive_respond_range × 0xc0)` — 1728 at the
/// Gun Mahout's 9 against a melee type's 768. A foe 1200 off is found
/// with the flag and not without it.
#[test]
fn an_idle_melee_and_ranged_type_searches_at_its_second_range() {
    for flagged in [true, false] {
        let (mut sim, foe_ty) = at_war();
        sim.cities_state.nation[0].human = false;
        sim.lobby.difficulty = 5;
        let mut ty = crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 0,
                second_max_range: 9,
                combat_role: true,
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        };
        if flagged {
            ty.cols.unit_flags |= crate::ai_load::uflags::MELEE_AND_RANGED;
        }
        let ty = sim.add_unit_type(ty);
        let at = Pos::new(0x1200, 0x1200);
        let me = put(&mut sim, 0, ty, at);
        let foe = put(&mut sim, 1, foe_ty, Pos::new(at.x + 1200, at.y));
        sim.units[me].combat.stance = combat::Stance::Defensive;
        sim.units[me].idle = 1;
        sim.think(me, 1);
        assert_eq!(
            sim.units[me].combat.target == Some(Obj::Unit(foe)),
            flagged,
            "flagged {flagged}: {:?}",
            sim.units[me].orders
        );
        assert_eq!(sim.combat.second_range, None);
    }
}

/// **A mandatory attack chases where a found one re-searches**
/// (`Unit::fight@005fd4d0:1051`, `5fe108`/`5fe135`; item 2864): a
/// packer's out-of-range attack carrying `mandatory` (`fight`'s third
/// argument) skips `find_new_target` and the STAND_GROUND gate and
/// walks to `find_attack_pos`'s spot — the MLRS `1/124` on 6821, whose
/// ring is theirs' six draws. Without the bit the packer's attack is
/// killed and re-found, and a packer (stance 2, off duty) stands.
#[test]
fn a_mandatory_packer_chases_without_a_re_search() {
    use crate::orders::{QueuePos, index};
    for mandatory in [true, false] {
        let (mut sim, me, foe) = idle_packed_engine(0x1800, false);
        sim.units[me].combat.stance = combat::Stance::StandGround;
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, mandatory, false);
        sim.work(me, 15);
        let kinds: Vec<u8> = sim.units[me].orders.iter().map(|o| o.index()).collect();
        if mandatory {
            assert_eq!(
                kinds,
                [index::MOVE_TO, index::ATTACK],
                "the chase stands above it"
            );
            assert!(sim.units[me].combat.mandatory);
        } else {
            assert!(
                !kinds.contains(&index::MOVE_TO),
                "a found attack re-searches and stands: {kinds:?}"
            );
        }
    }
}

/// run190's guard, in miniature: a player's chariot-ranged unit on its
/// post (3480, 12264) guarding a standing wagon, with an attack on an
/// enemy stacked above its `GUARD`.
fn guard_on_post(enemy_at: Pos) -> (Sim, usize, usize) {
    let (mut sim, ty) = at_war();
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = true;
    let mut t = sim.tables.unit_types[ty].clone();
    t.combat.max_range = 8;
    // A chariot is a combat unit (`role & 0x10000`), which is what puts
    // a guard on duty (`Unit::on_duty@005fff70`, item 1040).
    t.combat.combat_role = true;
    let chariot = sim.add_unit_type(t);
    let post = Pos::new(3480, 12264);
    let wagon = put(&mut sim, 0, chariot, Pos::new(3456, 11904));
    let guard = put(&mut sim, 0, chariot, post);
    let enemy = put(&mut sim, 1, chariot, enemy_at);
    sim.add_guard_order(guard, wagon, 0, 372, crate::orders::QueuePos::New);
    if let crate::orders::Body::Guard(g) = &mut sim.units[guard].orders[0].body {
        g.guard = post;
    }
    (sim, guard, enemy)
}

/// A frame off `do_guard`'s two sixteen-frame phases and the review's.
fn off_phase(sim: &Sim, u: usize) -> i64 {
    let o = i64::from(sim.units[u].index);
    (3 - o).rem_euclid(16) + 16
}

/// **A guard's attack is leashed to its post** (`Unit::fight@005fd4d0`
/// `005fdd0c`, `check_target@00649e00`'s guarding arm,
/// `docs/COMBAT.md` §63.1–§63.2). run190's own tick 1036: the enemy on
/// (3356, 14040) is ≈1,780 from the post, past `8 × 2 × 0x60` = 1,536,
/// so the unrecharged `fight` kills the attack with no draw, and the
/// guard's own search, leashed too, finds nothing and leaves `near`
/// empty. At 1,400 the attack stands.
///
/// Made to fail first with the arm removed from `do_attack`: the
/// attack stood and fired, the chapter's 1037 rows.
#[test]
fn a_guard_drops_an_attack_whose_target_leaves_its_leash() {
    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3356, 14040));
    sim.add_attack_order(
        guard,
        Obj::Unit(enemy),
        crate::orders::QueuePos::First,
        false,
        false,
    );
    let seed = sim.rng.seed;
    let f = off_phase(&sim, guard);
    sim.work(guard, f);
    assert_eq!(sim.order_type(guard), crate::orders::index::GUARD);
    assert_eq!(
        sim.units[guard].orders.len(),
        1,
        "{:?}",
        sim.units[guard].orders
    );
    assert_eq!(
        sim.units[guard].near, None,
        "the leashed search saw nothing"
    );
    assert_eq!(sim.rng.seed, seed, "a human's guard spends no draw");

    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3480, 13664));
    sim.add_attack_order(
        guard,
        Obj::Unit(enemy),
        crate::orders::QueuePos::First,
        false,
        false,
    );
    let f = off_phase(&sim, guard);
    sim.work(guard, f);
    assert_eq!(
        sim.units[guard].combat.target,
        Some(Obj::Unit(enemy)),
        "inside the leash the attack stands"
    );
}

/// **The guard's search is leashed and centred on its post**
/// (`find_nearby_target@00648da0`'s `local_2c`, §63.3): the enemy at
/// ≈1,635 from the post is inside the search's radius, and an idle
/// unit on the same spot takes it; a guard does not, and its `near`
/// stays empty. Parked 705's non-re-engagement, in miniature.
///
/// Made to fail first with the leash out of the candidate loop.
#[test]
fn a_guard_s_search_refuses_what_is_past_its_leash() {
    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
    assert_eq!(sim.find_melee_target(guard, -1), None);
    assert_eq!(sim.units[guard].near, None);
    sim.units[guard].orders.clear();
    assert_eq!(
        sim.find_melee_target(guard, -1),
        Some(Obj::Unit(enemy)),
        "unleashed, the same unit takes it"
    );
}

/// **A guard's attack on a building takes no chase** (`Unit::
/// find_attack_pos@00601280`'s `local_34 == 0xc`, `6015ea`, and
/// `fight`'s failure arm `5fe3f8`, `docs/AI.md` §138). A melee guard on
/// its post, its attack on an enemy building inside the leash but out
/// of reach: `find_attack_pos` reads the unit's **activity**, the
/// `GUARD` under the attack, refuses with no draw, and `fight` drops
/// the attack — the guard stands on its post. run662's Elite Pikeman
/// `1/115` on 17318, a lone `GUARD` on block 17319.
///
/// Made to fail first with the head order read in place of the
/// activity: the ring walk spent its draws and the chase went in front
/// (`[MoveTo, Attack, Guard]`).
#[test]
fn a_guard_s_attack_on_a_building_takes_no_chase() {
    let (mut sim, guard, _) = guard_on_post(Pos::new(3384, 13896));
    let ty = sim.units[guard].ty.unwrap();
    let mut melee = sim.tables.unit_types[ty].clone();
    melee.combat.max_range = 0;
    let melee = sim.add_unit_type(melee);
    sim.units[guard].ty = Some(melee);
    let b = sim.add_building(1, Pos::new(3480, 13200), 0);
    sim.buildings[b].hits = 1200;
    sim.buildings[b].health = 1200;
    assert!(!sim.is_in_range(Obj::Unit(guard), Obj::Building(b)));
    sim.add_attack_order(
        guard,
        Obj::Building(b),
        crate::orders::QueuePos::First,
        false,
        false,
    );
    assert!(sim.attack_pos_refused(guard, Obj::Building(b)));
    let seed = sim.rng.seed;
    let f = off_phase(&sim, guard);
    sim.work(guard, f);
    let kinds: Vec<u8> = sim.units[guard].orders.iter().map(|o| o.index()).collect();
    assert_eq!(kinds, [crate::orders::index::GUARD], "no chase in front");
    assert_eq!(sim.rng.seed, seed, "the refusal spends no draw");
}

/// **A human's action order holds its retaliation**
/// (`Unit::target_opportunity@005fffc0`, `LAB_00600877`, §63.4): a
/// player's guard under its `GUARD` (flags `ACTION`) does not answer a
/// hit; the same unit idle does, and so does an AI's guard
/// (`unit_masks & 0x40000`). run190's 1091.
///
/// Made to fail first with the gate removed: the human guard took the
/// attack.
#[test]
fn a_human_guard_does_not_answer_a_hit() {
    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
    sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
    assert_eq!(sim.order_type(guard), crate::orders::index::GUARD);
    assert_eq!(sim.units[guard].combat.target, None);

    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
    sim.cities_state.nation[0].human = false;
    sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
    assert_eq!(
        sim.units[guard].combat.target,
        Some(Obj::Unit(enemy)),
        "an AI's guard"
    );

    let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
    sim.units[guard].orders.clear();
    sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
    assert_eq!(
        sim.units[guard].combat.target,
        Some(Obj::Unit(enemy)),
        "idle"
    );
}
