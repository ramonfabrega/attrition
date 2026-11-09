/// §4.1's coin, all four outcomes: a citizen leaving a building spends
/// nothing, a **scout** spends one draw on the `% 2` arm, a **Bark**
/// one on the `% 3` arm, and a **spy** joins with no draw at all.
///
/// The whole point of the predicate is that the site is *rare*: it is
/// reached eleven times in run54's 24,000 frames, and the citizen row
/// here is why. A version that threw the coin for every AI unit passes
/// the scout and Bark rows and fails the other two.
#[test]
fn come_out_s_army_coin_is_thrown_by_the_two_scout_lineages_alone() {
    use crate::ai_load::uflags2;

    let mut sim = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    sim.world.fill_region(
        crate::world::Terrain::Land,
        crate::world::Cell::new(0, 0),
        crate::world::Cell::new(7, 7),
    );
    sim.nation[1].human = false;

    let make = |sim: &mut crate::Sim, flags2: u32, tree: Option<crate::tech::TypeId>| {
        let t = sim.add_unit_type(crate::UnitType {
            hits: 40,
            ..crate::UnitType::default()
        });
        sim.unit_types[t].cols.unit_flags2 = flags2;
        sim.unit_types[t].tree = tree;
        let index = sim
            .find_free(1, crate::UNIT_BASE, crate::BUILD_BASE)
            .unwrap();
        let mut u = crate::Unit::new(1, index, crate::world::Pos::new(384, 384), 40);
        u.ty = Some(t);
        sim.add_unit(u)
    };

    // A plain citizen: neither `is_special` nor `is(BARK)`, and no
    // draw — this is the row that keeps the site rare.
    let citizen = make(&mut sim, 0, None);
    let before = sim.rng.seed;
    sim.come_out_join_army(citizen);
    assert_eq!(sim.rng.seed, before, "a citizen throws no coin");
    assert!(sim.army_of(citizen).is_none());

    // A scout: `is_special` is `is(SCOUT)` folded into `unit_flags2`.
    let scout = make(&mut sim, uflags2::FLAG2_SCOUT, None);
    let before = sim.rng.seed;
    sim.come_out_join_army(scout);
    assert_ne!(sim.rng.seed, before, "the scout arm's `% 2`");

    // A Bark: the naval-scout lineage, the `% 3` arm, and no
    // `unit_flags2` bit of its own.
    let bark = make(&mut sim, 0, Some(0x143));
    let before = sim.rng.seed;
    sim.come_out_join_army(bark);
    assert_ne!(sim.rng.seed, before, "the Bark arm's `% 3`");

    // A spy: the draw-free arm, and the only one that always joins.
    let spy = make(&mut sim, 0, Some(0x3a));
    let before = sim.rng.seed;
    sim.come_out_join_army(spy);
    assert_eq!(sim.rng.seed, before, "the `is(SPY)` arm spends nothing");
    // It reaches `add_to_army` unconditionally; whether a slot comes
    // back is that function's own business, and this bare world has no
    // city to seed one at (§4).
    assert!(sim.army_of(spy).is_none(), "no city, so no army to seed");

    // The human's own units never reach any of it.
    let index = sim
        .find_free(0, crate::UNIT_BASE, crate::BUILD_BASE)
        .unwrap();
    let t = sim.units[scout].ty.unwrap();
    let mut h = crate::Unit::new(0, index, crate::world::Pos::new(384, 384), 40);
    h.ty = Some(t);
    let h = sim.add_unit(h);
    let before = sim.rng.seed;
    sim.come_out_join_army(h);
    assert_eq!(sim.rng.seed, before, "`unit_masks & 0x40000` gates it");
}

use super::*;

fn sim_with_city() -> (Sim, usize) {
    sim_with_city_slots(2)
}

fn sim_with_city_slots(slots: usize) -> (Sim, usize) {
    let mut sim = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        slots,
    );
    sim.nation[0].human = true;
    sim.nation[1].human = false;
    let pos = Pos::new(0x3000, 0x3000);
    let b = sim.add_building(1, pos, 1);
    let c = sim.cities.len();
    sim.cities.push(crate::city::City {
        alive: true,
        owner: 1,
        race: Some(1),
        founder: 1,
        building: b,
        members: Vec::new(),
        reg: None,
        pos,
        capital: true,
        founding_capital: true,
        was_founding_capital: false,
        unassimilated: false,
        no_heal: false,
        attacking: false,
        ever_attacked: false,
        alarm: false,
        no_muster: false,
        was_capital: 0,
        capture_stamp: 0,
        assimilation_timer: 0,
        attack_stamp: 0,
        reduce_stamp: 0,
        raid_stamp: 0,
        capture_strength: 0,
        pop: 1,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    });
    (sim, c)
}

#[test]
fn census_seeds_armies_in_region_order_not_city_creation_order() {
    let (mut sim, c) = sim_with_city();
    sim.cities[c].reg = Some(6);
    for region in [11, 5] {
        let mut city = sim.cities[c].clone();
        city.reg = Some(region);
        city.capital = false;
        sim.cities.push(city);
    }
    sim.census_seed_army(1);
    let armies: Vec<_> = sim.armies[1]
        .valid()
        .map(|(slot, a)| (slot, a.reg, a.city))
        .collect();
    assert_eq!(
        armies,
        vec![
            (0, Some(5), Some(2)),
            (1, Some(6), Some(0)),
            (2, Some(11), Some(1))
        ]
    );
    assert!(
        sim.armies[1].list[1].ticks_on(250),
        "region six gets slot one's phase, independent of city order"
    );
}

/// **The strength terms skip a decoy** (twenty-fourth pass, group 1):
/// `GroupData::count`'s `COUNT_NON_DECOY_TYPE` skips every decoy, so the
/// hoplite term does too; `COUNT_SIEGE` skips a decoy only under
/// computer control (`unit_masks & 0x40001 == 0x40001`).
#[test]
fn the_hoplite_and_siege_terms_skip_decoys() {
    for (human, siege_expected) in [(false, 2), (true, 3)] {
        let (mut sim, c) = sim_with_city();
        sim.nation[1].human = human;
        let hop = soldier_type(&mut sim);
        sim.unit_types[hop].cols.role |= role::ROLE_HOPLITE;
        let ram = soldier_type(&mut sim);
        sim.unit_types[ram].combat.siege = true;
        let slot = sim.init_army(1, Some(c));
        for i in 0..3 {
            for t in [hop, ram] {
                let u = put(&mut sim, 1, t, Pos::new(0x3000 + i * 48, 0x3000));
                sim.army_add_unit(1, slot, u);
                sim.units[u].decoy = i == 2;
            }
        }
        assert_eq!(sim.army_count_hoplites(1, slot), 2, "decoys never count");
        assert_eq!(
            sim.army_count_siege(1, slot),
            siege_expected,
            "a decoy siege engine is skipped under computer control only"
        );
    }
}

/// **A navy's city filter asks the city's `ocean` too** (twenty-fourth
/// pass, group 4; A5 #31, `6f70b5..6f70e0`): a coastal-region city with no
/// water counted by its owner's census is skipped before its draw.
#[test]
fn a_navy_skips_a_coastal_city_with_no_ocean() {
    let mut draws = Vec::new();
    for (ocean, team_style, start_list) in [
        (0, 1, [0, 1, 2, 3, 4, 5, 6, 7]),
        (3, 1, [0, 1, 2, 3, 4, 5, 6, 7]),
        // `team_style == 2`: an enemy that is neither me, an ally nor the
        // start list's target is skipped whole (group 4; A5 #30).
        (3, 2, [1; 8]),
        (3, 2, [0, 1, 2, 3, 4, 5, 6, 7]),
    ] {
        let (mut sim, c) = sim_with_city();
        sim.lobby.difficulty = 5;
        sim.lobby.team_style = team_style;
        sim.lobby.start_list = start_list;
        sim.ai[1].pers.early_army = 1;
        sim.world
            .fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
        sim.cities[c].reg = sim.world.region_of(sim.cities[c].pos.cell());
        let slot = sim.init_army(1, Some(c));
        sim.armies[1].list[slot].navy = true;
        sim.cities[c].no_muster = true;
        let fb = sim.add_building(0, Pos::new(0x6000, 0x3000), 1);
        let mut foe = sim.cities[c].clone();
        foe.owner = 0;
        foe.founder = 0;
        foe.no_muster = false;
        foe.building = fb;
        foe.pos = sim.buildings[fb].pos;
        let fc = sim.cities.len();
        sim.cities.push(foe);
        sim.buildings[fb].city = Some(fc);
        sim.ai[0]
            .city_ai
            .resize(fc + 1, crate::ai::CityAi::default());
        sim.ai[0].city_ai[fc].ocean = ocean;
        sim.declare_war(1, 0);
        let before = sim.rng.seed;
        sim.find_target(1, slot);
        draws.push(sim.rng.seed != before);
    }
    assert_eq!(
        draws,
        [false, true, false, true],
        "the city is drawn for only when its census found water, and under \
             team style 2 only when its owner is the start list's target"
    );
}

/// **An enemy city's `bordering` is its own** (item 3338, `docs/AI.md`
/// §273): `find_target@006f69b0:486`–`489` reads `CityData +0x65` off
/// the city record, which `compute_reg_territory` writes into the
/// city's owner's row. Two enemy cities: B is already this army's
/// target (×8), A borders who=1 in who=0's row (×4, and ×3 twice for
/// a quiet leader and city at a high difficulty) — A wins. Read off the
/// scoring army's own table, A is never bordering and B wins (run973b's
/// 10490: Napata scored 192 against who=1's own cities' 313..347).
#[test]
fn an_enemy_city_s_bordering_is_read_off_its_owner() {
    let (mut sim, c) = sim_with_city();
    sim.lobby.difficulty = 5;
    sim.ai[1].pers.early_army = 1;
    sim.frame = 20_000;
    sim.cities[c].no_muster = true;
    let slot = sim.init_army(1, Some(c));
    let hop = soldier_type(&mut sim);
    sim.unit_types[hop].cols.role |= role::ROLE_HOPLITE;
    for i in 0..4 {
        let u = put(&mut sim, 1, hop, Pos::new(0x3000 + i * 48, 0x3000));
        sim.army_add_unit(1, slot, u);
    }
    let mut foes = Vec::new();
    for x in [0x6000, 0x6800] {
        let fb = sim.add_building(0, Pos::new(x, 0x3000), 1);
        let mut foe = sim.cities[c].clone();
        foe.owner = 0;
        foe.founder = 0;
        foe.capital = false;
        foe.founding_capital = false;
        foe.no_muster = false;
        foe.building = fb;
        foe.pos = sim.buildings[fb].pos;
        let fc = sim.cities.len();
        sim.cities.push(foe);
        sim.buildings[fb].city = Some(fc);
        foes.push((fc, fb));
    }
    let (a, a_b) = foes[0];
    let (_, b_b) = foes[1];
    sim.ai[0]
        .city_ai
        .resize(sim.cities.len(), crate::ai::CityAi::default());
    sim.ai[0].city_ai[a].bordering = 3;
    sim.declare_war(1, 0);
    sim.armies[1].list[slot].status = status::FORMING;
    sim.armies[1].list[slot].target = Some(Obj::Building(b_b));
    sim.find_target(1, slot);
    assert_eq!(
        sim.armies[1].list[slot].target,
        Some(Obj::Building(a_b)),
        "the bordering city, read off its owner"
    );
}

/// **`team_style == 2` narrows the leaders** (group 4; A5 #30): me, the
/// start list's next live leader, or a mutual ally. Seat `k`'s next in a
/// shuffled list (`1 0 3 2 …`, a kept dump's) is not seat `k + 1`.
#[test]
fn the_start_list_s_target_is_the_next_live_leader() {
    let (mut sim, _) = sim_with_city();
    assert_eq!(sim.start_list_target(0), 1);
    assert_eq!(sim.start_list_target(1), 0, "unseated seats are skipped");
    sim.lobby.start_list = [4, 7, 6, 1, 0, 3, 2, 5];
    assert_eq!(sim.start_list_target(1), 0, "the list's order: 6 1 0");
    assert_eq!(
        sim.start_list_target(0),
        1,
        "the list's order: 0 3 2 5 4 7 6 1"
    );
    sim.defeated[1] = true;
    assert_eq!(sim.start_list_target(0), 0, "nobody left but me: me");
}

/// **A full pool's takeover leaves the old group seated** (group 4; A5
/// #10): `Army::init` touches no group, so the evicted army's members
/// still name a group the pool holds.
#[test]
fn taking_over_a_valid_slot_orphans_its_group() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let u = put(&mut sim, 1, t, Pos::new(0x3000, 0x3000));
    sim.army_add_unit(1, slot, u);
    let pool = sim.armies[1].list[slot].group.pool;
    assert!(pool.is_some(), "the unit is seated");
    sim.army_init(1, slot, Some(c));
    assert!(
        sim.pushed
            .iter()
            .any(|x| x.who == 1 && x.state.pool == pool && x.list.contains(&u)),
        "the old group stays in the pool"
    );
}

#[test]
fn target_strength_counts_living_nondecoy_cavalry_lineage() {
    use crate::tech::{TypeDef, UnitTraits};
    for (last_decoy, last_dead, expected) in [(false, false, 4), (true, false, 3), (false, true, 3)]
    {
        let (mut sim, c) = sim_with_city();
        sim.lobby.difficulty = 5;
        sim.ai[1].pers.early_army = 1;
        sim.world
            .fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
        sim.cities[c].reg = sim.world.region_of(sim.cities[c].pos.cell());
        let root = sim
            .tech_tree
            .add(TypeDef::unit("Cataphract", UnitTraits::default()));
        let upgrade = sim
            .tech_tree
            .add(TypeDef::unit("Knight", UnitTraits::default()).from(root));
        sim.tech_tree.roles.cataphract = Some(root);
        let t = soldier_type(&mut sim);
        sim.unit_types[t].tree = Some(upgrade);
        let slot = sim.init_army(1, Some(c));
        for i in 0..4 {
            let u = put(&mut sim, 1, t, Pos::new(0x3000 + i * 48, 0x3000));
            sim.army_add_unit(1, slot, u);
            if i == 3 {
                sim.units[u].decoy = last_decoy;
                if last_dead {
                    sim.units[u].health = 0;
                }
            }
        }
        assert_eq!(sim.army_count_cataphracts(1, slot), expected);
        sim.cities[c].no_muster = true;
        let fb = sim.add_building(0, Pos::new(0x6000, 0x3000), 1);
        let mut foe = sim.cities[c].clone();
        foe.owner = 0;
        foe.founder = 0;
        foe.no_muster = false;
        foe.building = fb;
        foe.pos = sim.buildings[fb].pos;
        let fc = sim.cities.len();
        sim.cities.push(foe);
        sim.buildings[fb].city = Some(fc);
        sim.declare_war(1, 0);
        sim.find_target(1, slot);
        assert_eq!(
            sim.armies[1].list[slot].target == Some(Obj::Building(fb)),
            expected == 4,
            "only four live real cavalry admit an enemy city"
        );
    }
}

/// [`sim_with_city`] at difficulty 5, at war with player 0, whose one
/// city one cell-region away is my city's clone; my capital is under
/// attack, so it scores `×10 ×10` over its draw. An army of four
/// hoplites at my city, and the enemy city's building, are returned.
fn sim_with_rival_city() -> (Sim, usize, usize) {
    let (mut sim, c) = sim_with_city();
    sim.lobby.difficulty = 5;
    sim.ai[1].pers.early_army = 1;
    sim.world
        .fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
    sim.cities[c].reg = sim.world.region_of(sim.cities[c].pos.cell());
    sim.cities[c].no_heal = true;
    let hop = soldier_type(&mut sim);
    sim.unit_types[hop].cols.role |= role::ROLE_HOPLITE;
    let slot = sim.init_army(1, Some(c));
    for i in 0..4 {
        let u = put(&mut sim, 1, hop, Pos::new(0x3000 + i * 48, 0x3000));
        sim.army_add_unit(1, slot, u);
    }
    let fb = sim.add_building(0, Pos::new(0x6000, 0x3000), 1);
    let mut foe = sim.cities[c].clone();
    foe.owner = 0;
    foe.founder = 0;
    foe.no_heal = false;
    foe.building = fb;
    foe.pos = sim.buildings[fb].pos;
    let fc = sim.cities.len();
    sim.cities.push(foe);
    sim.buildings[fb].city = Some(fc);
    sim.declare_war(1, 0);
    // `declare_war` retargets the armies; the call under test starts
    // from none.
    sim.armies[1].list[slot].target = None;
    (sim, slot, fb)
}

/// **`about_to_win`** (`find_target@006f69b0`, `local_34`, the `imul
/// 0x64` at `6f79de`): while my territory timer runs an enemy's city
/// scores a hundred times over — French East Indies' Napata on 15088,
/// 20040 against Paris's 345 on run649's packet (item 1487).
#[test]
fn my_running_territory_timer_makes_an_enemy_city_the_target() {
    for (timer, enemy_wins) in [(0, false), (1, true)] {
        let (mut sim, slot, fb) = sim_with_rival_city();
        sim.ai[1].popwin_timer = timer;
        sim.find_target(1, slot);
        assert_eq!(
            sim.armies[1].list[slot].target == Some(Obj::Building(fb)),
            enemy_wins,
            "timer {timer}: my attacked capital wins unless the timer runs"
        );
        assert_eq!(sim.ai[0].attacked_by == 1, enemy_wins, "the stamp");
    }
}

/// **A wagon-less army past Military 2 is weak** (`local_54`,
/// `6f6a1b..6f6a6b`) while its leader can have a wagon at all: it
/// scores no enemy city. French East Indies' army 0 on 14332..14844
/// carries none and leaves Napata alone under a running timer; army 6
/// carries one on 15088 and takes it (item 1487).
#[test]
fn a_wagonless_army_past_military_two_scores_no_enemy_city() {
    use crate::tech::{TechTree, TypeDef, UnitTraits};
    for (wagon, enemy_wins) in [(false, false), (true, true)] {
        let (mut sim, slot, fb) = sim_with_rival_city();
        sim.ai[1].popwin_timer = 1;
        // My capital untroubled: an enemy city one wagon short of two
        // still loses a tenth past Military 2, and wins at that.
        sim.cities[0].no_heal = false;
        let mut tree = TechTree::default();
        while tree.types.len() <= SUPPLYWAGON {
            let mut d = TypeDef::unit("Unit", UnitTraits::default());
            d.tribe_mask = u32::MAX;
            tree.add(d);
        }
        sim.set_tech_tree(tree);
        sim.tech[1].epoch[crate::tech::Line::Military.index()] = 3;
        // Past `early_army`'s Military 2 the gate wants seven standard.
        sim.armies[1].list[slot].num_standard = 7;
        if wagon {
            let t = soldier_type(&mut sim);
            sim.unit_types[t].cols.unit_flags2 |= uflags2::SUPPLY_OR_HERO;
            let u = put(&mut sim, 1, t, Pos::new(0x3000, 0x3030));
            sim.army_add_unit(1, slot, u);
        }
        sim.find_target(1, slot);
        assert_eq!(
            sim.armies[1].list[slot].target == Some(Obj::Building(fb)),
            enemy_wins,
            "wagon {wagon}: the enemy city is scored only with a wagon"
        );
    }
}

// ---- the order-issuing half (§8, §9, §11, §14; docs/GROUPS.md) ----

fn soldier_type(sim: &mut Sim) -> usize {
    sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: crate::combat::Profile {
            attack: 15,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    })
}

fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.unit_types[ty].hits;
    let mut u = crate::Unit::new(who, index, p, hits);
    u.ty = Some(ty);
    u.on_map = true;
    u.kind = sim.unit_types[ty].kind;
    u.movement.speed = 25;
    u.movement.turning = crate::movement::Turning {
        type_turn_speed: crate::movement::degrees_to_angle(45).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };
    sim.add_unit(u)
}

/// **`Army::normalize` puts the group's cap back to its leader's
/// speed** (`docs/AI.md` §161): it calls `Group::normalize@00711540`
/// on each of the army's groups, and that ends with `speed =
/// new_speed = UnitData::speed(find_leader)` whatever the members
/// reported. run679's army `1/0` walks at the slow squad's 25 until
/// its periodic normalize on 1434 and at its leader `1/28`'s 47 after.
/// Made to fail first: without the tail the cap stays 25.
#[test]
fn an_army_normalize_resets_its_group_s_cap_to_the_leader_s_speed() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let leader = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    let slow = put(&mut sim, 1, t, Pos::new(0x1200, 0x1000));
    sim.units[leader].movement.speed = 47;
    sim.army_add_unit(1, slot, leader);
    sim.army_add_unit(1, slot, slow);
    let g = sim.army_group(1, slot);
    assert_eq!(
        sim.group_find_leader(&g),
        Some(leader),
        "the fixture: a tie on `type_cat` goes to the first"
    );
    // A follower's report has driven both halves down to its 25.
    sim.armies[1].list[slot].group.speed = 25;
    sim.armies[1].list[slot].group.new_speed = 25;
    assert_eq!(sim.group_speed_of(slow), 25);
    sim.army_normalize(1, slot);
    assert_eq!(sim.group_speed_of(slow), 47, "the leader's own speed");
    assert_eq!(sim.armies[1].list[slot].group.new_speed, 47, "both halves");
}

#[test]
fn do_forming_walks_the_army_to_the_projected_muster_origin() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    let b = put(&mut sim, 1, t, Pos::new(0x1200, 0x1000));
    sim.army_add_unit(1, slot, a);
    sim.army_add_unit(1, slot, b);
    sim.armies[1].list[slot].muster = Cell { x: 20, y: 20 };
    sim.armies[1].list[slot].muster_angle = Angle(0);
    sim.do_forming(1, slot);
    // One group, so the origin is the muster cell's centre projected
    // one tile north (§8: `num_groups × 0xc0` along the muster angle,
    // which is 0 — due north, `y − 0xc0`).
    let want = step_along(cell_centre(Cell { x: 20, y: 20 }), Angle(0), 0xc0);
    for u in [a, b] {
        let o = *sim.current_order(u).expect("an order from do_forming");
        // Two of them in a land formation, so the order the original
        // hands each is a `GroupAttackToOrder` — `GROUP_ATTACK_TO`,
        // not the plain `ATTACK_TO` (item 237; §1.2).
        assert_eq!(o.index(), crate::orders::index::GROUP_ATTACK_TO);
        let crate::orders::Body::Move(m) = o.body else {
            panic!("a move");
        };
        // The destination is the origin snapped to its 48-unit centre
        // (`docs/ORDERS.md` §4.3).
        assert_eq!(m.dest.x.div_euclid(0x30), want.x.div_euclid(0x30));
        assert_eq!(m.dest.y.div_euclid(0x30), want.y.div_euclid(0x30));
    }
}

/// **The retarget forms the army itself** (§9, item 811):
/// `do_marching@006f3df0`'s retarget arm is `find_target(); if found
/// { status |= 0x10; do_forming(); return; }`, and `Army::process`'s
/// dispatch then reads `& 0x10` and forms it **again** on the same
/// tick. So on a retarget tick the army's group is moved twice, and the
/// second move's `get_loc` reads the first one's `(ox, oy)` and order
/// (`docs/ARMY.md` §21). Made to fail first: with the arm's
/// `do_forming` removed, `do_marching` leaves the members with no order.
#[test]
fn a_retarget_forms_the_army_inside_do_marching() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    // An enemy city for `find_target` to pick.
    let foe_pos = Pos::new(0x6000, 0x3000);
    let fb = sim.add_building(0, foe_pos, 1);
    let mut foe = sim.cities[c].clone();
    foe.owner = 0;
    foe.building = fb;
    foe.pos = foe_pos;
    foe.capital = true;
    let fc = sim.cities.len();
    sim.cities.push(foe);
    sim.buildings[fb].city = Some(fc);
    sim.declare_war(1, 0);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    let b = put(&mut sim, 1, t, Pos::new(0x1200, 0x1000));
    sim.army_add_unit(1, slot, a);
    sim.army_add_unit(1, slot, b);
    sim.armies[1].list[slot].target = None;
    sim.armies[1].list[slot].status = status::MARCHING;
    sim.do_marching(1, slot);
    assert!(
        sim.armies[1].list[slot].target.is_some(),
        "the fixture: `find_target` found the enemy city"
    );
    assert_ne!(sim.armies[1].list[slot].status & status::FORMING, 0);
    for u in [a, b] {
        let o = sim
            .current_order(u)
            .expect("`do_marching`'s retarget formed the army itself");
        assert_eq!(o.index(), crate::orders::index::GROUP_ATTACK_TO);
    }
}

#[test]
fn a_forming_army_actually_moves_its_units_over_the_ticks() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    sim.army_add_unit(1, slot, a);
    sim.armies[1].list[slot].muster = Cell { x: 30, y: 30 };
    let before = sim.units[a].pos;
    sim.do_forming(1, slot);
    // The order step and the body follow, as `Objects::process_all`
    // runs them; the rest of the tick needs a tech tree this fixture
    // has no use for.
    for f in 0..64 {
        sim.work(a, f);
        sim.process_movement(a);
    }
    assert_ne!(
        sim.units[a].pos, before,
        "the order the group issued is stepped by the order system"
    );
    let d = |p: Pos| {
        vector_dist(
            (p.x - cell_centre(Cell { x: 30, y: 30 }).x).abs(),
            (p.y - cell_centre(Cell { x: 30, y: 30 }).y).abs(),
        )
    };
    assert!(
        d(sim.units[a].pos) < d(before),
        "and it walks toward the muster spot"
    );
}

/// **The emergency is a city alarm, and a soldier never rings it**
/// (§15.8, `docs/COMBAT.md` §23, item 399). Made to fail first in the
/// unit direction: before the fix `do_damage` called
/// `armies_emergency` on any object of another player, so the unit hit
/// below cleared the army's target exactly as the building hit does.
#[test]
fn only_a_hit_on_a_city_s_building_rings_the_emergency() {
    // The witness is `normalize`'s recount, not the cleared target: the
    // tick that follows the clear runs `do_marching` and can find a new
    // one on the same call, exactly as the golden record's `ns 0 -> 1`
    // at 618 showed (`docs/COMBAT.md` §21.4).
    let ticked = |sim: &Sim, slot: usize| sim.armies[1].list[slot].num_standard;
    // A unit of the AI leader takes a hit: nothing happens to its army.
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let victim = put(&mut sim, 1, t, Pos::new(0x5000, 0x5000));
    let foe = put(&mut sim, 0, t, Pos::new(0x5040, 0x5000));
    sim.army_add_unit(1, slot, victim);
    let mark = Obj::Unit(foe);
    let _ = mark;
    sim.armies[1].list[slot].target = Some(mark);
    assert_eq!(
        ticked(&sim, slot),
        0,
        "the army has not been normalized yet"
    );
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Unit(victim),
        Angle(0),
        false,
        0x100,
        false,
        false,
        0,
    );
    assert_eq!(
        ticked(&sim, slot),
        0,
        "a soldier taking a hit rang the emergency: `do_damage`'s \
             `local_30` is set only by the city-alarm arms of the building \
             branch (§15.8)"
    );
    // The same leader's own city building takes one: the army drops its
    // target and ticks.
    let b = sim.cities[c].building;
    // The fixture stands the city up around the building without the
    // back-pointer `Cities::add_to_city` writes; the predicate reads
    // that field, so set it here rather than assert on the fixture.
    sim.buildings[b].city = Some(c);
    sim.buildings[b].combat = Some(crate::combat::Profile::default());
    sim.buildings[b].hits = 1000;
    sim.buildings[b].health = 1000;
    // The alarm is the heavy arm's (item 2235, `docs/GOLDEN.md` §81):
    // the building stands at `hits / 4`, the higher of the two
    // ceilings, and the leader's `capital_stamp` is 0, so the
    // hit comes past the 300-frame cooldown.
    sim.buildings[b].active = true;
    sim.buildings[b].damage = 250;
    sim.do_damage(
        Obj::Unit(foe),
        Obj::Building(b),
        Angle(0),
        false,
        0x100,
        false,
        false,
        301,
    );
    assert_eq!(
        ticked(&sim, slot),
        1,
        "a hit on the AI leader's city building did not reach \
             `Armies::emergency`"
    );
}

/// **The engaged army forms a cell behind its point** (§23, item
/// 1052): `march_to_target`'s origin is `(x, y)` stepped one cell
/// along `muster_angle − 0x80000000`, away from the target
/// (`6f4daa`–`6f4e2b`). The group's `(ox, oy)` is where the move was
/// ordered to. Great Lakes' army 0 on 4860 is the case: point
/// `(3168, 31584)`, one cell south of the human's city, and a muster
/// angle of `−165478400` pointing north at it — the origin is
/// `(3352, 32331)`, south, where stepping forward put it on the city.
/// Made to fail first: with the step along `muster_angle` the record
/// holds `(2983, 30838)`.
#[test]
fn an_engaged_army_forms_a_cell_behind_its_point() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let p = Pos::new(3168, 31584);
    let foe_pos = Pos::new(3168, 30816);
    let fb = sim.add_building(0, foe_pos, 1);
    sim.buildings[fb].hits = 1200;
    sim.buildings[fb].health = 1200;
    sim.declare_war(1, 0);
    {
        let a = &mut sim.armies[1].list[slot];
        a.pos = p;
        a.muster_angle = Angle(-165_478_400);
        a.target = Some(Obj::Building(fb));
        a.status = status::MARCHING | status::FORMING;
    }
    let mine: Vec<usize> = (0..2)
        .map(|k| put(&mut sim, 1, t, Pos::new(p.x + k * 0x40, p.y - 0x200)))
        .collect();
    for &u in &mine {
        sim.army_add_unit(1, slot, u);
        sim.add_attack_order(u, Obj::Building(fb), QueuePos::New, true, true);
    }
    assert!(
        sim.army_is_engaged(1, slot),
        "the fixture: both are fighting"
    );
    sim.march_to_target(1, slot);
    assert_ne!(
        sim.armies[1].list[slot].status & status::MARCHED,
        0,
        "the orders went out"
    );
    let g = sim.army_group(1, slot);
    let o = sim.gstate(&g).expect("the army's record").o;
    assert_eq!(
        o,
        Pos::new(3352, 32331),
        "a cell behind, away from the city"
    );
    assert_eq!(o, march_origin(p, Angle(-165_478_400)));
}

#[test]
fn engagement_points_the_whole_army_at_the_first_fighter_s_target() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let p = Pos::new(0x5000, 0x5000);
    sim.armies[1].list[slot].pos = p;
    // Three of mine at the army's point; one is already fighting, and
    // `is_engaged` wants more than a quarter (§11).
    let mine: Vec<usize> = (0..3)
        .map(|k| put(&mut sim, 1, t, Pos::new(p.x + k * 0x40, p.y)))
        .collect();
    for &u in &mine {
        sim.army_add_unit(1, slot, u);
    }
    let foe = put(&mut sim, 0, t, Pos::new(p.x + 0x100, p.y));
    sim.add_attack_order(mine[0], Obj::Unit(foe), QueuePos::New, true, true);
    assert!(
        sim.army_is_engaged(1, slot),
        "one of three is over the quarter"
    );
    sim.engagement(1, slot);
    for &u in &mine {
        assert_eq!(
            sim.units[u].combat.target,
            Some(Obj::Unit(foe)),
            "every member takes the engaged unit's target"
        );
    }
}

/// An armed enemy building at the army's point: `active`, so
/// `group_action_attack` will order against it. The earlier form of
/// the test below left it unarmed, and its "the army ignores a
/// building target" passed only because nobody could be ordered at
/// an inactive target — the opposite of §11's fallback rule.
fn armed_wall(sim: &mut Sim, p: Pos) -> usize {
    let wall = sim.add_building(0, p, 1);
    sim.buildings[wall].combat = Some(crate::combat::Profile::default());
    sim.buildings[wall].hits = 100;
    sim.buildings[wall].health = 50;
    wall
}

#[test]
fn engagement_falls_back_to_the_last_qualifier_s_building_target() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let p = Pos::new(0x5000, 0x5000);
    sim.armies[1].list[slot].pos = p;
    let a = put(&mut sim, 1, t, p);
    let b = put(&mut sim, 1, t, Pos::new(p.x + 0x40, p.y));
    sim.army_add_unit(1, slot, a);
    sim.army_add_unit(1, slot, b);
    let wall = armed_wall(&mut sim, Pos::new(p.x + 0x100, p.y));
    sim.add_attack_order(a, Obj::Building(wall), QueuePos::New, true, true);
    sim.engagement(1, slot);
    assert_eq!(
        sim.units[b].combat.target,
        Some(Obj::Building(wall)),
        "`is_map_unit` gates only the break: with no map-unit target the \
             last qualifying unit's building is the army's (§11)"
    );
}

#[test]
fn engagement_yields_nothing_when_the_last_qualifier_s_target_is_gone() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let p = Pos::new(0x5000, 0x5000);
    sim.armies[1].list[slot].pos = p;
    let a = put(&mut sim, 1, t, p);
    let b = put(&mut sim, 1, t, Pos::new(p.x + 0x40, p.y));
    let idle = put(&mut sim, 1, t, Pos::new(p.x + 0x80, p.y));
    for &u in &[a, b, idle] {
        sim.army_add_unit(1, slot, u);
    }
    let wall = armed_wall(&mut sim, Pos::new(p.x + 0x100, p.y));
    sim.add_attack_order(a, Obj::Building(wall), QueuePos::New, true, true);
    // `b` qualifies — it carries an attack action — but its target
    // order has gone negative, as an attack whose target died reads.
    sim.add_attack_order(b, Obj::Building(wall), QueuePos::New, true, true);
    sim.units[b].combat.target = None;
    sim.engagement(1, slot);
    assert_eq!(
        sim.units[idle].combat.target, None,
        "`6f5345`: the last qualifier's −1 overwrites the earlier wall, \
             and the tail at `6f5383` gives nothing"
    );
}

#[test]
fn close_halts_the_units_it_held() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    sim.army_add_unit(1, slot, a);
    sim.add_move_order(
        a,
        Pos::new(0x4000, 0x4000),
        MoveKind::MoveTo,
        QueuePos::New,
        true,
    );
    sim.close_army(1, slot);
    assert!(
        sim.current_order(a).is_none(),
        "`Army::close` is `Group::action_halt(g, 0)` per group"
    );
}

/// **`go_to_unit` marks the whole squad, and `Unit::work` unmarks it
/// 32 frames later** — `unit_masks & 4`, the bit §6.6 step 6 of
/// `docs/GROUPS.md` exempts a `GroupMoveOrder` by and the one this
/// crate's invented army gate was standing in for (item 465,
/// `docs/ORDERS.md` §16).
///
/// Two halves, and the second is the one with teeth: a latch that is
/// set and never cleared would keep every marching squad on plain
/// moves for the rest of the game and look exactly like the gate that
/// was there before. The clear is `0060d19d`'s, inside the same
/// `(frame + o) & 31 == 0` that drops [`crate::Unit::visible`].
#[test]
fn go_to_unit_marks_the_squad_and_the_32_frame_tick_clears_it() {
    let (mut sim, _) = sim_with_city();
    let t = soldier_type(&mut sim);
    let cap = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    let sub = put(&mut sim, 1, t, Pos::new(0x1050, 0x1000));
    let far = put(&mut sim, 1, t, Pos::new(0x9000, 0x9000));
    // One squad of two: the follower keys on the captain's index.
    let c = sim.units[cap].index as i32;
    sim.units[cap].combat.captain = c;
    sim.units[sub].combat.captain = c;
    sim.units[cap].captain = true;
    sim.units[sub].captain = false;
    sim.go_to_unit(sub, far);
    assert_eq!(
        (
            sim.units[cap].in_danger,
            sim.units[sub].in_danger,
            sim.units[far].in_danger
        ),
        (true, true, false),
        "the mark walks the captain's own chain and stops there"
    );
    // The captain's own tick, and only it.
    let o = i64::from(sim.units[cap].index);
    let ticks = |f: i64| (f + o).rem_euclid(32) == 0;
    while !ticks(sim.frame) {
        sim.tick();
        assert!(
            sim.units[cap].in_danger,
            "the latch survives every frame but its own"
        );
    }
    sim.tick();
    assert!(!sim.units[cap].in_danger, "and `Unit::work` drops it on it");
}

#[test]
fn send_here_moves_the_point_the_muster_cell_and_the_units() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    sim.army_add_unit(1, slot, a);
    let to = Pos::new(0x6000, 0x7000);
    sim.send_here(1, slot, to, MoveKind::MoveTo);
    let army = &sim.armies[1].list[slot];
    assert_eq!(army.pos, to);
    assert_eq!(army.muster, to.cell());
    assert_eq!(sim.order_type(a), crate::orders::index::MOVE_TO);
}

/// **`Unit::think`'s tail joins supply wagons and heroes, and nothing
/// else** (§4) — the rule item 68 was.
///
/// The listing at `005f7615` is `if (!is_supply && !is_hero) { … return
/// } add_to_army(this)`, so a fighting unit reaching the *tail* does
/// **not** join: `think_attack`'s own `add_to_army` is a different
/// site, behind that function's city search, and `think_attack` is not
/// reached by a unit that has a job. The implementation used to join
/// "an attacker that is not a scout or a caravan" here, which
/// conscripted run10's woodcutters.
/// **A computer army holding a Scout gives it the spellcaster's turn**
/// (`Army::process@006f93d0`'s `count(COUNT_NON_DECOY_TYPE, 0x45)` →
/// `Army::use_scouts@006f49a0`, item 3186, `docs/AI.md` §251): on the
/// 128-frame turn the Scout (`is_special`) is offered
/// `think_spellcaster`, whose special arm throws its coin at `+0x413` —
/// run897b's frame 5144, index 0, who=1's `1/22` (TI 79, `takohs`, of
/// the Scout's lineage) in its army 1. A decoy Scout is
/// not counted and throws nothing. Made to fail with the scouts' turn
/// not called.
#[test]
fn an_army_s_scout_throws_the_special_coin_on_the_spellcaster_turn() {
    use crate::spellcaster::SITE_SPECIAL_COIN;
    let (mut sim, c) = sim_with_city();
    sim.lobby.difficulty = 3;
    // A tree whose `0x45` is a type, so the lineage test answers.
    let mut tree = crate::tech::TechTree::new();
    for _ in 0..=SCOUT {
        tree.add(crate::tech::TypeDef::unit(
            "Scout",
            crate::tech::UnitTraits::default(),
        ));
    }
    sim.set_tech_tree(tree);
    let scout = sim.add_unit_type(crate::UnitType {
        hits: 100,
        tree: Some(SCOUT),
        type_index: 0x45,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::FLAG2_SCOUT,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let soldier = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let s = put(&mut sim, 1, scout, Pos::new(0x1000, 0x1000));
    let f = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
    sim.army_add_unit(1, slot, f);
    sim.army_add_unit(1, slot, s);
    sim.army_normalize(1, slot);
    assert_ne!(sim.armies[1].list[slot].num_captains, 0);
    let phase = sim.armies[1].list[slot].phase();
    // The 128-frame turn and not the 256-frame tick.
    let frame = (30 - phase).rem_euclid(128) + 128;
    assert_ne!((frame + phase) % 256, 0);
    sim.frame = frame;
    sim.trace_phases = true;
    let seed = sim.rng.seed;
    sim.army_process(1, slot, false);
    assert_ne!(sim.rng.seed, seed, "the coin is thrown");
    assert!(
        sim.phase_marks.iter().any(|(m, _)| m == SITE_SPECIAL_COIN),
        "at the special arm's site"
    );
    sim.units[s].decoy = true;
    let seed = sim.rng.seed;
    sim.army_process(1, slot, false);
    assert_eq!(sim.rng.seed, seed, "a decoy Scout is not counted");
}

#[test]
fn only_a_supply_wagon_or_a_hero_joins_an_army_from_the_think_tail() {
    let (mut sim, _c) = sim_with_city();
    // A plain fighting type — an attack, no lineage bit.
    let soldier = soldier_type(&mut sim);
    let a = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
    sim.think_join_army(a);
    assert_eq!(
        sim.army_of(a),
        None,
        "the tail is not `think_attack`: an attacker does not join here"
    );

    // The same type with the supply lineage bit — `unit_flags2 & 0x40`.
    let wagon = sim.add_unit_type(crate::UnitType {
        hits: 100,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::SUPPLY_OR_HERO,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let b = put(&mut sim, 1, wagon, Pos::new(0x1000, 0x1000));
    sim.think_join_army(b);
    assert!(
        sim.army_of(b).is_some(),
        "`is_supply` is the first of the tail's two ways in"
    );

    // And the hero bit — `unit_flags2 & 0x20`.
    let hero = sim.add_unit_type(crate::UnitType {
        hits: 100,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::FLAG2_GENERAL,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let h = put(&mut sim, 1, hero, Pos::new(0x1000, 0x1000));
    sim.think_join_army(h);
    assert!(sim.army_of(h).is_some(), "`is_hero` is the second");
}

/// **A government patriot is capped by the army's wagons, not its
/// heroes** (`find_local_army@006f32e0`, `docs/AI.md` §213): the arm is
/// the joiner's `is_supply` (`& 0x40`), and `is_hero` (`& 0x20`) is
/// asked only when that is clear. The Despot (`FROM General`, both
/// bits) refuses the near army's three wagons and takes the far army,
/// as run820's `1/191` does on 4344; a General (`0x20` alone) takes
/// the near one. Made to fail with the arm read as "a wagon is `0x40`
/// without `0x20`": the Despot joins the near army.
#[test]
fn a_government_patriot_is_capped_by_the_army_s_wagons() {
    let (mut sim, c) = sim_with_city();
    let near = sim.init_army(1, Some(c));
    let far = sim.init_army(1, Some(c));
    let here = sim.armies[1].list[near].pos;
    sim.armies[1].list[far].pos = Pos::new(here.x + 0x1800, here.y);
    let ty = |sim: &mut Sim, flags2: u32| {
        sim.add_unit_type(crate::UnitType {
            hits: 100,
            cols: crate::ai_load::UnitCols {
                unit_flags2: flags2,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        })
    };
    let wagon = ty(&mut sim, uflags2::SUPPLY_OR_HERO);
    let general = ty(&mut sim, uflags2::FLAG2_GENERAL);
    let despot = ty(&mut sim, uflags2::FLAG2_GENERAL | uflags2::SUPPLY_OR_HERO);
    for _ in 0..3 {
        let w = put(&mut sim, 1, wagon, here);
        sim.army_add_unit(1, near, w);
    }
    let w = put(&mut sim, 1, wagon, here);
    sim.army_add_unit(1, far, w);
    for slot in [near, far] {
        sim.armies[1].list[slot].status |= status::MUSTERING;
    }
    let d = put(&mut sim, 1, despot, here);
    assert_eq!(
        sim.find_local_army(1, here, Some(d)).map(|(s, _)| s),
        Some(far),
        "three wagons refuse a patriot"
    );
    let g = put(&mut sim, 1, general, here);
    assert_eq!(
        sim.find_local_army(1, here, Some(g)).map(|(s, _)| s),
        Some(near),
        "a General is asked about generals"
    );
    assert_eq!(
        sim.find_army(1, here, -1, Some(d)).map(|(s, _)| s),
        Some(far),
        "`find_army` has the same arm"
    );
}

/// **`think_attack`'s head is the one an AI soldier takes** (§4.2) —
/// the site the think tail's test above says is a different one.
///
/// Three gates, each made to fail: a type with an attack but no
/// `role & 0x10000` is an armed citizen and does not join (that
/// omission was run53's frame 307); a military type does; and a
/// **damaged foot** unit standing inside its own city's radius stays
/// where it is.
#[test]
fn think_attack_s_head_joins_a_military_unit_and_leaves_an_armed_citizen() {
    use crate::ai_load::role;
    let (mut sim, _c) = sim_with_city();
    // An attack, and no military bit: `determine_roles`' civilian arm.
    let armed_citizen = soldier_type(&mut sim);
    let a = put(&mut sim, 1, armed_citizen, Pos::new(0x1000, 0x1000));
    assert_eq!(
        sim.think_attack_join_army(a),
        None,
        "`Unit::think`'s step 3 needs `role & 0x10000` as well as the attack"
    );

    let soldier = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: crate::combat::Profile {
            attack: 15,
            obj_masks: crate::combat::mask::FOOT,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        cols: crate::ai_load::UnitCols {
            role: role::ROLE_MILITARY,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let b = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
    assert!(
        sim.think_attack_join_army(b).is_some(),
        "a military type with an attack joins on its first idle think"
    );

    // The stay-and-heal arm: damaged, foot, inside my own city's
    // radius. Both halves of the territory test have to hold, so the
    // tile mask and the cell's owner are written together.
    let c = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
    sim.units[c].health -= 1;
    let p = sim.units[c].pos;
    sim.world.set_owner(
        p.cell(),
        crate::world::Owner::Player(1),
        crate::world::Owner::None,
    );
    sim.world
        .set_tile_mask(p.tile(), crate::world::tile::CITY_RADIUS);
    assert_eq!(
        sim.think_attack_join_army(c),
        None,
        "a damaged foot unit at home stays there (`5f5d10`)"
    );
    // And the same unit one tile outside the radius does join.
    sim.world.set_tile_mask(p.tile(), 0);
    assert!(
        sim.think_attack_join_army(c).is_some(),
        "the stay-and-heal arm is the city radius', not the damage's alone"
    );
}

/// `think_attack`'s tail, each arm made to fail: the nearer city wins
/// on distance alone; a city's danger divides its distance, so a
/// farther city with danger 4 wins; `CITY_UNDER_ATTACK` halves it
/// again (danger 2 alone loses, danger 2 and `no_heal` wins); a unit
/// on its own territory, a human's, and one in another region, walk
/// nowhere (item 3369, `docs/AI.md` §285).
#[test]
fn think_attack_s_tail_picks_the_city_by_distance_over_danger() {
    let (mut sim, c0) = sim_with_city();
    let near = Pos::new(0x1000, 0x2000);
    let b = sim.add_building(1, near, 1);
    let mut city = sim.cities[c0].clone();
    city.building = b;
    city.pos = near;
    city.capital = false;
    city.founding_capital = false;
    sim.cities.push(city);
    let c1 = sim.cities.len() - 1;
    for c in [c0, c1] {
        sim.cities[c].reg = sim.world.tregion(sim.cities[c].pos.tile());
    }
    let ty = soldier_type(&mut sim);
    let u = put(&mut sim, 1, ty, Pos::new(0x1000, 0x1000));
    assert_eq!(
        sim.think_attack_city(u),
        Some(c1),
        "the nearer city, on distance alone"
    );

    let far = sim.cities[c0].pos.cell();
    sim.world.set_danger_at(1, far, 4);
    assert_eq!(
        sim.think_attack_city(u),
        Some(c0),
        "danger divides the distance"
    );
    sim.world.set_danger_at(1, far, 2);
    assert_eq!(
        sim.think_attack_city(u),
        Some(c1),
        "danger 2 alone is not enough"
    );
    sim.cities[c0].no_heal = true;
    assert_eq!(
        sim.think_attack_city(u),
        Some(c0),
        "`CITY_UNDER_ATTACK` halves it"
    );

    sim.cities[c0].reg = sim.cities[c0].reg.map(|r| r + 1).or(Some(1));
    assert_eq!(
        sim.think_attack_city(u),
        Some(c1),
        "a city in another region is not a candidate"
    );

    let p = sim.units[u].pos;
    sim.world.set_owner(
        p.cell(),
        crate::world::Owner::Player(1),
        crate::world::Owner::None,
    );
    sim.world
        .set_tile_mask(p.tile(), crate::world::tile::CITY_RADIUS);
    assert_eq!(
        sim.think_attack_city(u),
        None,
        "a unit on its own territory stays"
    );
    sim.world.set_tile_mask(p.tile(), 0);

    sim.nation[1].human = true;
    assert_eq!(sim.think_attack_city(u), None, "a human's unit is not sent");
}

#[test]
fn charge_drags_the_army_onto_the_attacker() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
    sim.army_add_unit(1, slot, a);
    let foe = put(&mut sim, 0, t, Pos::new(0x2000, 0x2000));
    sim.army_charge(1, slot, Obj::Unit(foe));
    assert_eq!(sim.armies[1].list[slot].target, Some(Obj::Unit(foe)));
    assert_eq!(sim.order_type(a), crate::orders::index::ATTACK_TO);
}

#[test]
fn init_army_takes_the_first_free_slot_then_evicts_the_smallest() {
    let (mut sim, c) = sim_with_city();
    for s in 0..SLOTS {
        assert_eq!(sim.init_army(1, Some(c)), s);
        sim.armies[1].list[s].num_units = (s as i32 % 5) + 1;
    }
    // Slots 0, 5, 10 and 15 hold one unit; `<=` on the scan sends the
    // tie to the later slot.
    assert_eq!(sim.init_army(1, Some(c)), 15);
    sim.armies[1].list[15].num_units = 9;
    assert_eq!(sim.init_army(1, Some(c)), 10);
}

#[test]
fn the_muster_cell_is_one_row_below_the_city_and_the_record_is_run20_s() {
    let (mut sim, c) = sim_with_city();
    sim.cities[c].pos = Pos::new(39264, 40032);
    let s = sim.init_army(1, Some(c));
    let a = &sim.armies[1].list[s];
    assert_eq!((a.pos.x, a.pos.y), (39264, 40800));
    assert_eq!((a.muster.x, a.muster.y), (51, 53));
    assert_eq!(a.status, status::MUSTERING);
    assert_eq!(a.target, None);
    assert_eq!(a.num_groups(), 0);
    assert!(a.valid);
}

#[test]
fn the_cadence_is_slot_and_owner_phased() {
    let (mut sim, c) = sim_with_city();
    let s = sim.init_army(1, Some(c));
    let a = &sim.armies[1].list[s];
    // Leader 1's army 0 ticks at frame ≡ 252 (mod 256) — run21's 252.
    assert!(a.ticks_on(252));
    assert!(!a.ticks_on(253));
    assert!(a.ticks_on(14588));
    // Its army 1 at 250 — run21's `do_transporting` at 14586.
    let s1 = sim.init_army(1, Some(c));
    assert!(sim.armies[1].list[s1].ticks_on(14586));
}

#[test]
fn close_frees_the_slot_and_keeps_the_rest_of_the_record() {
    let (mut sim, c) = sim_with_city();
    let s = sim.init_army(1, Some(c));
    sim.armies[1].list[s].rally_dist = 0x1200;
    sim.close_army(1, s);
    let a = &sim.armies[1].list[s];
    assert!(!a.valid);
    assert_eq!(a.status, 0);
    assert_eq!(
        a.rally_dist, 0x1200,
        "close clears valid, status, human_frame and the groups only"
    );
}

/// **A closed army's group stays in the pool** (§22, item 829).
/// `Army::close@006f8ea0` writes the group's `army` to −1 and halts it;
/// the record, its index and its list stay, and so does every member's
/// `+0x80`. So a new army formed from one of them takes **another**
/// slot, and `get_open_slot`'s tail clears nobody's pointer. East
/// Indies' army 1 closed on 18682, the wagon formed army 1 again on
/// 18692, and the escort's pointers at the old group are what the
/// collision on 18938 read. Made to fail first: with the group dropped
/// at close, the new army takes the old slot and the two left behind
/// point at nothing.
#[test]
fn a_closed_army_s_group_keeps_its_slot_its_list_and_its_pointers() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    let slot = sim.init_army(1, Some(c));
    let units: Vec<usize> = (0..3)
        .map(|i| put(&mut sim, 1, t, Pos::new(0x1000 + 0x100 * i, 0x1000)))
        .collect();
    for &u in &units {
        sim.army_add_unit(1, slot, u);
    }
    let pool = sim.units[units[0]].group_ptr.expect("the army's slot");
    sim.close_army(1, slot);
    assert!(!sim.armies[1].list[slot].valid);
    assert_eq!(
        sim.pool_list(1, pool),
        units
            .iter()
            .map(|&u| sim.units[u].index)
            .collect::<Vec<_>>(),
        "the group is still listed on its own index"
    );
    assert!(
        units.iter().all(|&u| sim.units[u].group_ptr == Some(pool)),
        "and every member still names it"
    );
    assert_eq!(sim.army_of(units[0]), None, "in no army");
    // The first of them forms a new army: it takes another slot.
    let s2 = sim.init_army(1, Some(c));
    sim.army_add_unit(1, s2, units[0]);
    let fresh = sim.units[units[0]].group_ptr.expect("the new army's slot");
    assert_ne!(fresh, pool, "the old slot is still held");
    assert!(
        units[1..]
            .iter()
            .all(|&u| sim.units[u].group_ptr == Some(pool)),
        "the two left behind still point at the old group"
    );
}

#[test]
fn release_mustering_wants_five_and_the_city_count_before_the_thresholds() {
    let (mut sim, c) = sim_with_city();
    let s = sim.init_army(1, Some(c));
    sim.muster[1].cap = 200;
    sim.armies[1].list[s].num_standard = 1;
    assert!(!sim.release_mustering(1, s));
    sim.armies[1].list[s].num_standard = 4;
    assert!(!sim.release_mustering(1, s), "fewer than five");
    // Five standard: `pop_cap / 8 = 25 > 5`, one mustering army → `200 / 2 = 100 > 5`,
    // difficulty 0 wants more than six, `rush 0`, `early_army 0` → 26.
    sim.armies[1].list[s].num_standard = 5;
    assert!(!sim.release_mustering(1, s));
    sim.armies[1].list[s].num_standard = 7;
    assert!(
        sim.release_mustering(1, s),
        "difficulty 0 releases at seven"
    );
    sim.lobby.difficulty = 3;
    assert!(!sim.release_mustering(1, s));
    sim.armies[1].list[s].num_standard = 26;
    assert!(sim.release_mustering(1, s));
    // My own city under attack releases at two.
    sim.armies[1].list[s].num_standard = 2;
    sim.cities[c].no_heal = true;
    assert!(sim.release_mustering(1, s));
}

/// **The rush rule reads the Military library level, not the age**
/// (`docs/ARMY.md` §20, item 657). `release_mustering`'s tail reads
/// `data_encrypted->epoch[0] ^ 0x63187` (`6f8997`, `6f89c3`); `ages`
/// is `+0xdc` and nothing in the function reads it. Great Lakes' who=1
/// on 12024 has age 1 and Military 2, and six standard stay.
#[test]
fn release_mustering_s_rush_rule_reads_the_military_level_not_the_age() {
    let (mut sim, c) = sim_with_city();
    let s = sim.init_army(1, Some(c));
    // Every threshold above the personality's is out of reach: `pop_cap
    // / 8 = 25 > 6`, `200 / 2 = 100 > 6`, difficulty 0 wants seven.
    sim.muster[1].cap = 200;
    sim.armies[1].list[s].num_standard = 6;
    sim.ai[1].pers.rush = 1;
    sim.tech[1].ages = 1;
    sim.tech[1].epoch[crate::tech::Line::Military.index()] = 2;
    assert!(
        !sim.release_mustering(1, s),
        "Military 2 wants sixteen, whatever the age"
    );
    sim.tech[1].epoch[crate::tech::Line::Military.index()] = 1;
    sim.tech[1].ages = 3;
    assert!(
        sim.release_mustering(1, s),
        "Military 1 releases at once, whatever the age"
    );
}

/// **The pre-war gate holds a ready army** (`006f87c0`; item 2388,
/// `docs/AI.md` §221): `RUSH_RULES 10`, five minutes' peace, keeps an
/// army that every threshold would release at the muster until frame
/// 4500, and lets it go on that frame.
#[test]
fn release_mustering_holds_the_army_through_the_peace() {
    let (mut sim, c) = sim_with_city();
    let s = sim.init_army(1, Some(c));
    sim.muster[1].cap = 200;
    sim.armies[1].list[s].num_standard = 6;
    sim.ai[1].pers.rush = 1;
    sim.tech[1].epoch[crate::tech::Line::Military.index()] = 1;
    assert!(sim.release_mustering(1, s), "no rush rules: released");
    sim.lobby.rush_rules = 10;
    sim.lobby.rush_minutes = 5;
    sim.frame = 4499;
    assert!(!sim.release_mustering(1, s), "inside the peace: held");
    sim.frame = 4500;
    assert!(sim.release_mustering(1, s), "the peace over: released");
}

// ---- the ring search (§13) ----

use crate::world::CellData;

/// [`sim_with_city`] with the city building wired to its city, as
/// `activate` leaves it, and one army mustering at the city: its
/// point is one cell south of the city's, cell (16, 17). A level-1
/// city's radius is 20 tiles, so `inner = 6` and `outer = 8`: the
/// search walks rings 7 and 8 — 40 and 52 entries — around (16, 17),
/// and the first entry of ring 7 is (−7, −3), the cell (9, 14).
fn sim_with_army() -> (Sim, usize, usize, usize) {
    let (mut sim, c) = sim_with_city();
    let b = sim.cities[c].building;
    sim.buildings[b].city = Some(c);
    sim.buildings[b].combat = Some(crate::combat::Profile::default());
    sim.buildings[b].hits = 1000;
    sim.buildings[b].health = 1000;
    let s = sim.init_army(1, Some(c));
    assert_eq!(sim.armies[1].list[s].pos.cell(), Cell::new(16, 17));
    assert_eq!(sim.radius_of(c) / 4 + 1, 6);
    (sim, c, b, s)
}

fn flag_cell(sim: &mut Sim, x: i32, y: i32, flags: u16) {
    let c = Cell::new(x, y);
    let mut d = sim.world.cell_data(c);
    d.flags = flags;
    sim.world.set_cell_data(c, d);
}

fn flag_all(sim: &mut Sim, flags: u16) {
    for y in 0..sim.world.height() {
        for x in 0..sim.world.width() {
            flag_cell(sim, x, y, flags);
        }
    }
}

#[test]
fn the_ring_search_takes_the_first_cell_of_the_ring_past_inner_in_walk_order() {
    let (mut sim, c, b, s) = sim_with_army();
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    let a = &sim.armies[1].list[s];
    assert_eq!(a.muster, Cell::new(9, 14), "ring 7's first entry, (−7, −3)");
    assert_eq!(
        a.muster_angle,
        find_angle(-7, -3),
        "the angle is from the army's point"
    );
    assert!(!sim.cities[c].no_muster);
    // The same cell whichever neighbourhood the flag walks: nothing is
    // out of bounds or foreign.
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
}

#[test]
fn a_forest_coast_or_mountain_cell_in_the_3x3_puts_a_candidate_out() {
    let (mut sim, _, b, s) = sim_with_army();
    // Ring 7 opens (−7, −3), (−7, −2), (−7, −1) … (−7, 3), then (−6,
    // −4), (−6, 4), … — cells (9, 14) to (9, 20), then (10, 13), (10, 21).
    // A forest on (9, 14) is in the 3 × 3 of (9, 15) too.
    flag_cell(&mut sim, 9, 14, cell::FOREST);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 16));
    flag_cell(&mut sim, 10, 16, cell::COAST);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 18));
    flag_cell(&mut sim, 8, 17, cell::MOUNTAIN);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
    // A cell two away is walked for bounds and owners only, not class:
    // the 7 × 7 walk does not read it.
    flag_cell(&mut sim, 11, 19, cell::FOREST);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
    // Rock and road are admissible.
    flag_cell(&mut sim, 9, 19, cell::ROCK | cell::ROAD);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
    // A building's cell in the 3 × 3 scores nothing: (9, 19) and (9,
    // 20) score eight neighbours, (10, 13) is out on the forest, and
    // (10, 21) is the first to score nine.
    flag_cell(&mut sim, 9, 19, cell::BUILDING);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 21));
}

#[test]
fn another_leader_s_cell_two_steps_out_puts_a_candidate_out_and_the_flag_widens_the_walk() {
    let (mut sim, _, b, s) = sim_with_army();
    // The double of (9, 14)'s north-west neighbour.
    sim.world
        .set_owner(Cell::new(7, 12), Owner::Player(0), Owner::None);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
    sim.world
        .set_owner(Cell::new(7, 12), Owner::None, Owner::None);
    // The double of (9, 14)'s (3, 0) entry — six cells east — is walked
    // only with the flag.
    sim.world
        .set_owner(Cell::new(15, 14), Owner::Player(0), Owner::None);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
    // My own cell, or an ally's, is not foreign.
    sim.world
        .set_owner(Cell::new(15, 14), Owner::Player(1), Owner::None);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
}

/// **The spacing test is `<= 4`, not `< 4`** — the listing at
/// `6f633a` is `cmp $0x4` then **`jle`** (and `cmp $0x2` / the same
/// `jle` on the enemy arm at `6f6313`), so a candidate *at* the
/// distance is dropped. This test asserted the off-by-one until item
/// 352, and it is what put Great Lakes' AI army on the wrong muster:
/// army 1/2's own spot sat exactly four from a cell it therefore kept,
/// which left army 1/1 the cell the original had already spent
/// (`docs/ARMY.md` §13,
/// `great_lakes_8442_musters_the_army_where_the_original_does`).
#[test]
fn a_candidate_within_four_cells_of_another_army_of_mine_is_dropped() {
    let (mut sim, c, b, s) = sim_with_army();
    let s2 = sim.init_army(1, Some(c));
    sim.armies[1].list[s2].muster = Cell::new(9, 14);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(
        sim.armies[1].list[s].muster,
        Cell::new(9, 19),
        "(9, 15) to (9, 18) are within four — (9, 18) is *at* four, and \
             `jle` drops it; (9, 19) is at five"
    );
    // An invalid slot does not count.
    sim.close_army(1, s2);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
}

/// The return is the last in-bounds candidate's verdict, not whether a
/// best was found (listing `6f6824`). Ring 7 and ring 8 hold 92 entries
/// and the early stop fires 0x28 past the best, so the best has to sit
/// near the end of ring 8 for the two to differ: with everything forest
/// but one 3 × 3 patch, the patch is the best, and the return says
/// whether the *last* entry, (8, 3), was admissible.
#[test]
fn the_return_is_the_last_candidate_s_verdict_not_the_best_s() {
    let (mut sim, c, b, s) = sim_with_army();
    flag_all(&mut sim, cell::FOREST);
    for y in 19..=21 {
        for x in 23..=25 {
            flag_cell(&mut sim, x, y, 0);
        }
    }
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(
        sim.armies[1].list[s].muster,
        Cell::new(24, 20),
        "(8, 3), the last entry"
    );
    assert!(!sim.cities[c].no_muster);

    let (mut sim, c, b, s) = sim_with_army();
    flag_all(&mut sim, cell::FOREST);
    for y in 18..=20 {
        for x in 23..=25 {
            flag_cell(&mut sim, x, y, 0);
        }
    }
    assert!(
        !sim.find_muster_spot(1, s, Obj::Building(b), true),
        "the last entry is out, so the search reports failure"
    );
    assert_eq!(
        sim.armies[1].list[s].muster,
        Cell::new(24, 19),
        "with the best, (8, 2), set as the muster all the same"
    );
    assert!(!sim.cities[c].no_muster, "the mark is the no-best path's");
}

#[test]
fn no_cell_at_an_untroubled_city_of_my_own_sets_the_mark_and_a_find_clears_it() {
    let (mut sim, c, b, s) = sim_with_army();
    flag_all(&mut sim, cell::FOREST);
    let before = sim.armies[1].list[s].muster;
    assert!(!sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert!(sim.cities[c].no_muster);
    assert_eq!(
        sim.armies[1].list[s].muster, before,
        "the muster is left alone"
    );
    // Under attack, no mark.
    sim.cities[c].no_muster = false;
    sim.cities[c].no_heal = true;
    assert!(!sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert!(!sim.cities[c].no_muster);
    sim.cities[c].no_heal = false;
    sim.cities[c].no_muster = true;
    flag_all(&mut sim, 0);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert!(!sim.cities[c].no_muster);
}

#[test]
fn no_cell_at_an_enemy_city_falls_back_to_the_city_s_cell_one_row_down() {
    let (mut sim, c, b, s) = sim_with_army();
    sim.cities[c].owner = 0;
    sim.buildings[b].owner = 0;
    flag_all(&mut sim, cell::FOREST);
    // Not mine: `inner` halves to 3 without siege, and the rings are
    // 4 and 5 — all forest.
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(16, 17));
    assert_eq!(sim.armies[1].list[s].muster_angle, find_angle(0, 1));
    assert!(!sim.cities[c].no_muster);
}

/// **The escort** (`Armies::send_navy@006f2c90`, `docs/TRANSPORT.md`
/// §8.3): a land army's enemy target in another region is handed to
/// every navy of more than two captains whose sea coasts both regions,
/// and that navy is processed at once — off its frames, which only
/// counts `human_frame` down. East Indies' army 3 (corvettes,
/// fireships, a frigate) took army 0's target this way on 15612 and
/// kept it on its own turn, 15862.
#[test]
fn send_navy_hands_the_target_to_the_navy_that_coasts_both_regions() {
    let (mut sim, c, b, _) = sim_with_army();
    // Land A | sea S | land B | sea T: S coasts A and B, T only B.
    let a = sim.world.add_region(Terrain::Land);
    let sea = sim.world.add_region(Terrain::Sea);
    let bb = sim.world.add_region(Terrain::Land);
    let t = sim.world.add_region(Terrain::Sea);
    for y in 0..60 {
        for x in 0..60 {
            let r = match x {
                0..20 => a,
                20..30 => sea,
                30..40 => bb,
                _ => t,
            };
            sim.world.set_region(Cell::new(x, y), r);
        }
    }
    sim.world.rebuild_coasts();
    let mut navy = |reg: u16, captains: i32| {
        let k = sim.init_army(1, Some(c));
        let n = &mut sim.armies[1].list[k];
        n.navy = true;
        n.reg = Some(reg);
        n.num_captains = captains;
        n.human_frame = 5;
        k
    };
    let escort = navy(sea, 3);
    let elsewhere = navy(t, 3);
    let small = navy(sea, 2);
    let target = Obj::Building(b);
    sim.send_navy(1, target, Some(a), Some(bb));
    let n = &sim.armies[1].list;
    assert_eq!(n[escort].target, Some(target), "S coasts A and B");
    assert_eq!(n[escort].human_frame, 4, "and is processed at once");
    assert_eq!(n[elsewhere].target, None, "T does not coast A");
    assert_eq!(n[elsewhere].human_frame, 5);
    assert_eq!(n[small].target, None, "two captains are not enough");
    // A human leader sends nothing.
    sim.nation[1].human = true;
    sim.armies[1].list[small].num_captains = 3;
    sim.send_navy(1, target, Some(a), Some(bb));
    assert_eq!(sim.armies[1].list[small].target, None);
}

#[test]
fn a_navy_musters_on_water_of_its_region_scoring_the_5x5() {
    let (mut sim, c, b, s) = sim_with_army();
    sim.cities[c].owner = 0;
    sim.buildings[b].owner = 0;
    let sea = sim.world.add_region(Terrain::Sea);
    for y in 0..60 {
        for x in 0..60 {
            let k = Cell::new(x, y);
            sim.world.set_region(k, sea);
            sim.world.set_cell_data(
                k,
                CellData {
                    land: 2,
                    ..CellData::default()
                },
            );
        }
    }
    {
        let a = &mut sim.armies[1].list[s];
        a.navy = true;
        a.reg = Some(sea);
    }
    // `inner = 5`, `outer = 13`: ring 6 opens (−6, −3) … (−6, 3), the
    // cells (10, 14) to (10, 20).
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 14));
    // A land cell puts the candidate out — a coastal cell (`HALFLAND`)
    // is land to `is_ocean`. Without the flag only the 3 × 3 is
    // walked, so (11, 15) puts out (10, 14) to (10, 16).
    flag_cell(&mut sim, 11, 15, cell::HALFLAND);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 17));
    // With it the 5 × 5 is, and (10, 17) is out as well.
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 18));
    // Another army's spacing is two cells for an enemy target, and
    // `<= 2` rather than `< 2` (`6f6313`, the same `jle` as the ally
    // arm): (10, 18) is one away and (10, 19) exactly two, so the
    // first cell left is (10, 20).
    let s2 = sim.init_army(1, Some(c));
    sim.armies[1].list[s2].muster = Cell::new(10, 17);
    assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 20));
}

// ---- gaia is outside the object searches (`world::PLAYER_SLOTS`) ----

/// A damaged friendly building with an animal standing beside it. The
/// muster search asks `find_unit(SEARCH_ENEMY)` about everything on the
/// map, and the diplomacy table a two-player lobby builds is two wide —
/// which is how the first fuzzed seed (424242) took `Sim::is_enemy`
/// down with `who 8`. The accessors are total now, so it merely
/// answers.
#[test]
fn the_muster_search_survives_a_gaia_unit_with_a_two_wide_table() {
    let (mut sim, c) = sim_with_city();
    let t = soldier_type(&mut sim);
    assert_eq!(sim.at_war.len(), 2, "the lobby's table, not a wide one");
    let b = sim.cities[c].building;
    sim.buildings[b].hits = 100;
    sim.buildings[b].health = 50;
    let slot = sim.init_army(1, Some(c));
    let sheep = put(
        &mut sim,
        crate::world::PLAYER_SLOTS,
        t,
        Pos::new(0x3100, 0x3000),
    );
    assert!(sim.units[sheep].is_gaia());
    // No panic, and the animal is not an attacker to hurry towards.
    sim.find_muster_spot(1, slot, Obj::Building(b), false);
    assert_eq!(sim.armies[1].list[slot].hurry, 0);
}

/// And it is the leader bound that refuses it, not the table's length:
/// widen the table past the eight the original has, declare war on
/// gaia, and the animal is still not the muster point.
#[test]
fn a_gaia_unit_is_never_the_muster_point_even_at_war() {
    let slots = usize::from(crate::world::PLAYER_SLOTS) + 2;
    let (mut sim, c) = sim_with_city_slots(slots);
    let t = soldier_type(&mut sim);
    let b = sim.cities[c].building;
    sim.buildings[b].hits = 100;
    sim.buildings[b].health = 50;
    let slot = sim.init_army(1, Some(c));
    let g = usize::from(crate::world::PLAYER_SLOTS);
    sim.at_war[1][g] = true;
    sim.at_war[g][1] = true;
    let sheep = put(
        &mut sim,
        crate::world::PLAYER_SLOTS,
        t,
        Pos::new(0x3100, 0x3000),
    );
    assert!(
        sim.is_enemy(1, crate::world::PLAYER_SLOTS),
        "the table says enemy"
    );
    let sheep_cell = sim.units[sheep].pos.cell();
    sim.find_muster_spot(1, slot, Obj::Building(b), false);
    assert_eq!(sim.armies[1].list[slot].hurry, 0);
    assert_ne!(sim.armies[1].list[slot].muster, sheep_cell);
    // A player's soldier in the same spot *is* the muster point, so the
    // search is otherwise working.
    let foe = put(&mut sim, 0, t, Pos::new(0x3100, 0x3000));
    sim.at_war[1][0] = true;
    sim.at_war[0][1] = true;
    assert!(sim.find_muster_spot(1, slot, Obj::Building(b), false));
    assert_eq!(sim.armies[1].list[slot].hurry, 1);
    assert_eq!(sim.armies[1].list[slot].muster, sim.units[foe].pos.cell());
}
