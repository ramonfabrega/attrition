use super::*;
use crate::economy::{RATE_SCALE, Resource};
use crate::world::{
    Cell, CellData, Good, Pos, TILES_PER_CELL, Terrain, UNITS_PER_TILE, World, tile,
};
use crate::{Tuning, tech};

/// The fifty `resourcerules.xml` goods as this install writes the two
/// this mechanic reads: Fish pays Food 10 and Wealth 10, Whales pays
/// Food 10 and Metal 10.
fn good_table() -> Vec<economy::GoodType> {
    let mut v = vec![economy::GoodType::default(); 50];
    v[economy::FISH].bonus = [(Some(Resource::Food), 10), (Some(Resource::Wealth), 10)];
    v[economy::WHALES].bonus = [(Some(Resource::Food), 10), (Some(Resource::Metal), 10)];
    v
}

/// A 40 × 40 all-ocean world with a human and an AI, the fifty goods in
/// the tree, and one deployed AI Fisherman at the centre of cell
/// `(20, 20)` — `calc_gather.rs`'s fixture with the good table and a
/// naval `obj_masks` on the boat.
fn sea_sim() -> (Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
    for y in 0..40 {
        for x in 0..40 {
            world.set_cell_data(
                Cell::new(x, y),
                CellData {
                    land: 2,
                    down: -1,
                    ..CellData::default()
                },
            );
            for ty in 0..TILES_PER_CELL {
                for tx in 0..TILES_PER_CELL {
                    world.set_tile_mask(
                        Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty),
                        tile::SURFACE_OCEAN,
                    );
                }
            }
        }
    }
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.nation[0].human = true;
    s.nation[1].human = false;
    s.good_types = good_table();
    {
        let t = &mut s.tech_tree;
        for i in 0..50 {
            t.add(tech::TypeDef::good(&format!("good{i}")));
        }
        while t.types.len() < crate::fish::FISHERMEN {
            t.add(tech::TypeDef::plain("filler", -1));
        }
        let boat = t.add(tech::TypeDef::unit(
            "Fishing Boat",
            tech::UnitTraits::default(),
        ));
        assert_eq!(boat, crate::fish::FISHERMEN);
    }
    s.tech = (0..2)
        .map(|_| tech::PlayerTech::new(&s.tech_tree))
        .collect();
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        tree: Some(crate::fish::FISHERMEN),
        ..crate::UnitType::default()
    });
    s.unit_types[t].combat.packs = true;
    s.unit_types[t].combat.obj_masks |= crate::combat::mask::NAVAL;
    s.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
    let at = Pos::new(20 * 768 + 384, 20 * 768 + 384);
    let mut u = crate::Unit::new(1, 14, at, 20);
    u.ty = Some(t);
    let i = s.add_unit(u);
    // A packing type is born packed, and a packed boat is step 6's
    // very first refusal — this one has deployed.
    s.units[i].combat.packed = false;
    (s, i)
}

/// Ring 1 of the boat's own spiral, where a good the boat is "standing
/// on" actually sits (`docs/ORDERS.md` §6.10).
fn put_good_beside(s: &mut Sim, u: usize, ty: crate::tech::TypeId) {
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let t = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    s.world.add_good(Good {
        pos: Pos::new(t.x * UNITS_PER_TILE + 96, t.y * UNITS_PER_TILE + 96),
        ty,
        alive: true,
    });
    let m = s.world.tile_mask(t);
    s.world.set_tile_mask(t, m | tile::AS_BUILDING);
}

/// Puts a good on the boat's own tile and lights the tile's `0x200`,
/// which is what [`Sim::reveal_fog`] reads before it looks for one.
fn good_under(s: &mut Sim, u: usize, ty: crate::tech::TypeId) -> usize {
    let t = s.units[u].pos.tile();
    s.world.add_good(Good {
        pos: Pos::new(t.x * UNITS_PER_TILE + 96, t.y * UNITS_PER_TILE + 96),
        ty,
        alive: true,
    });
    // The footprint, not the tile: the original's gate reads the
    // **odd** tile `(2fx + 1, 2fy + 1)` of a fog cell, so a mark on one
    // even tile is a mark no fog cell can see.
    for dy in 0..2 {
        for dx in 0..2 {
            let at = Pos::new(t.x + dx, t.y + dy);
            let m = s.world.tile_mask(at);
            s.world.set_tile_mask(at, m | tile::AS_BUILDING);
        }
    }
    s.world.goods().len() - 1
}

/// The fog cell whose `(2fx + 1, 2fy + 1)` tile is one [`good_under`]
/// marked.
fn fog_of_unit(s: &Sim, u: usize) -> (i32, i32) {
    let t = s.units[u].pos.tile();
    (t.x / 2, t.y / 2)
}

/// **`FISH` and `WHALES` are not merchant rares**, and
/// `Leader::new_rare` is where they are dropped: they pay a fishing
/// boat, so a coast full of them must not buy the AI a Merchant.
/// Anything else the leader can build with is kept, once.
#[test]
fn a_fish_and_a_whale_are_never_a_seen_rare() {
    let (mut s, u) = sea_sim();
    let fish = good_under(&mut s, u, economy::FISH);
    let whale = good_under(&mut s, u, economy::WHALES);
    let citrus = good_under(&mut s, u, 26);
    s.new_rare(1, fish);
    s.new_rare(1, whale);
    assert!(s.ai[1].new_rares.is_empty(), "the fisherman's two rares");
    s.new_rare(1, citrus);
    s.new_rare(1, citrus);
    assert_eq!(
        s.ai[1].new_rares,
        vec![citrus],
        "recorded once, and once only"
    );
}

/// **A plain human records nothing** — `leader_flags & 0xc == 4` is the
/// caller's own gate and `& 4` the recipient's, so neither the human's
/// own reveals nor an ally's on its behalf reach its list.
#[test]
fn a_human_s_reveal_records_nothing_on_either_side() {
    let (mut s, u) = sea_sim();
    let citrus = good_under(&mut s, u, 26);
    s.new_rare(0, citrus);
    assert!(s.ai[0].new_rares.is_empty(), "the human's own reveal");
    assert!(s.ai[1].new_rares.is_empty(), "and it told nobody");
    s.allied[0][1] = true;
    s.allied[1][0] = true;
    s.new_rare(1, citrus);
    assert_eq!(s.ai[1].new_rares, vec![citrus]);
    assert!(
        s.ai[0].new_rares.is_empty(),
        "an ally that is a human is skipped"
    );
}

/// **`reveal_fog`'s rare arm marks the good seen** (`reveal_fog:87`–`88`,
/// item 3332): the revealer's bit of `ever_seen`, and nobody else's —
/// the byte `think_merchant` tests before it scores a good.
#[test]
fn reveal_fog_marks_the_rare_it_meets_seen() {
    let (mut s, u) = sea_sim();
    let g = good_under(&mut s, u, 26);
    assert!(!s.good_seen_bit(g, 1));
    let (fx, fy) = fog_of_unit(&s, u);
    s.reveal_fog(fx, fy, 1);
    assert_eq!(s.ai[1].new_rares, vec![g]);
    assert!(s.good_seen_bit(g, 1), "seen by its revealer");
    assert!(!s.good_seen_bit(g, 0), "and by nobody else");
}

/// **An oil patch is never seen**, because `Objects::init_good` never
/// puts one in a cell's chain — which is why East Indies' unreplayed
/// start fog costs that capture nothing: the one good under its AI at
/// frame 0 is oil.
#[test]
fn an_oil_patch_is_not_in_the_chain_to_be_seen() {
    let (mut s, u) = sea_sim();
    assert_eq!(good_under(&mut s, u, crate::world::OIL), 0);
    let (fx, fy) = fog_of_unit(&s, u);
    assert_eq!(Cell::new(fx >> 1, fy >> 1), s.units[u].pos.cell());
    s.reveal_fog(fx, fy, 1);
    assert!(s.ai[1].new_rares.is_empty(), "an oil patch is not a rare");
}

/// **The tile mark is the gate.** `reveal_fog` looks for a good only
/// where `Objects::init_good` marked the ground `0x200`; with the mark
/// off, the cell's own good is never reached.
#[test]
fn reveal_fog_looks_only_where_the_ground_is_marked() {
    let (mut s, u) = sea_sim();
    assert_eq!(good_under(&mut s, u, 26), 0);
    let (fx, fy) = fog_of_unit(&s, u);
    s.reveal_fog(fx, fy, 1);
    assert_eq!(s.ai[1].new_rares, vec![0]);

    let (mut s, u) = sea_sim();
    assert_eq!(good_under(&mut s, u, 26), 0);
    let mark = Pos::new(2 * fx + 1, 2 * fy + 1);
    let was = s.world.tile_mask(mark);
    s.world.set_tile_mask(mark, was & !tile::AS_BUILDING);
    s.reveal_fog(fx, fy, 1);
    assert!(s.ai[1].new_rares.is_empty(), "no mark, no look");
}

/// **The border pass meets a rare the reveal's tile gate missed** (item
/// 1555, `docs/AI.md` §152): `compute_reg_territory`'s per-cell goods
/// scan hands `Leader::new_rare` the first live good on a cell it gives
/// a leader, with no tile-mark test, once the cell's second half-cell
/// is seen by the leader or an ally — and sets the good's `ever_seen`
/// bit, which the Merchant's score reads.
#[test]
fn the_border_pass_meets_a_seen_rare_the_reveal_never_looked_for() {
    let (mut s, u) = sea_sim();
    let (w, h) = (s.world.width(), s.world.height());
    assert!(s.world.set_fog(vec![0; (w * h * 4) as usize]));
    let g = good_under(&mut s, u, 26);
    let c = s.units[u].pos.cell();
    s.claim_cell_goods(c, 1);
    assert!(s.ai[1].new_rares.is_empty(), "an unseen cell is not met");
    assert!(!s.good_seen_bit(g, 1));
    // The odd half-cell is the one the scan reads: a seen even one is
    // no reason to meet the good.
    s.world.set_seen(2 * c.x, 2 * c.y, 1 << 1);
    s.claim_cell_goods(c, 1);
    assert!(s.ai[1].new_rares.is_empty(), "the even half-cell is not it");
    s.world.set_seen(2 * c.x + 1, 2 * c.y + 1, 1 << 1);
    s.claim_cell_goods(c, 1);
    s.claim_cell_goods(c, 1);
    assert_eq!(s.ai[1].new_rares, vec![g], "met once");
    assert!(s.good_seen_bit(g, 1), "and seen by its owner");
    assert!(!s.good_seen_bit(g, 0), "and by nobody else");
}

/// **The two numbers run59's census measured.** A level-0 fisherman
/// standing on a fish in nobody's territory pays exactly `10 × 16` food
/// and `10 × 16` wealth — `FISHERMEN_BONUS` entry 0 is 0% and
/// `MERCHANTS_BONUS` entry 0 is 100%, so neither percentage moves
/// anything. The AI's income was 1440 against 1600 and its wealth 0
/// against 160 for exactly this reason.
#[test]
fn a_fish_pays_ten_of_each_of_its_two_goods() {
    let t = Tuning::RON;
    let g = good_table();
    let out = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        0,
        0,
        false,
        false,
        false,
    );
    assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE);
    assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE);
    assert_eq!(out[Resource::Timber.index()], 0);
}

/// **The fishermen bonus is added, and only to food.** At level 2 it is
/// 100%, so the food half doubles and the wealth half stands.
#[test]
fn the_fishermen_bonus_reaches_the_food_half_alone() {
    let t = Tuning::RON;
    let g = good_table();
    let out = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        2,
        0,
        false,
        false,
        false,
    );
    assert_eq!(t.fishermen_bonus[2], 100);
    assert_eq!(out[Resource::Food.index()], 20 * RATE_SCALE);
    assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE);
}

/// **A Japanese boat on a fish pays a quarter more of both halves**
/// (`JAPANESE_FISHING_BOATS`, `calc_rare`'s water arm, `006e0a4c`):
/// added to the food slot beside the fishermen's term, and to the wealth
/// slot without a merchant level. Two boats are block 3668's 80 in each
/// of food and wealth (run1153c, item 3521). Not on a land good.
#[test]
fn a_japanese_boat_on_a_fish_pays_a_quarter_more_of_each_half() {
    let t = Tuning::RON;
    let g = good_table();
    let out = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        0,
        0,
        false,
        false,
        true,
    );
    assert_eq!(t.japanese_fishing_boats, 25);
    assert_eq!(out[Resource::Food.index()], 200);
    assert_eq!(out[Resource::Wealth.index()], 200);
    let both = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        2,
        1,
        false,
        false,
        true,
    );
    assert_eq!(both[Resource::Food.index()], (100 + 25 + 100) * 160 / 100);
    assert_eq!(both[Resource::Wealth.index()], (120 + 25) * 160 / 100);
    let mut g = g;
    g[20].bonus = [(Some(Resource::Metal), 10), (None, 0)];
    let land = economy::calc_rare(&t, 20, &g[20], 0, 0, true, false, true);
    assert_eq!(land[Resource::Metal.index()], 160, "a land rare");
}

/// **The merchants bonus replaces the 100, and on a water good it
/// reaches everything but food.** At level 1 it is 120%.
#[test]
fn the_merchants_bonus_reaches_a_fish_s_other_half() {
    let t = Tuning::RON;
    let g = good_table();
    let out = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        0,
        1,
        false,
        false,
        false,
    );
    assert_eq!(t.merchants_bonus[1], 120);
    assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE);
    assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE * 120 / 100);
}

/// In **friendly ground** the same bonus reaches the food half too —
/// the `local_20` flag, which is the difference between "a merchant in
/// your own borders" and one abroad.
#[test]
fn friendly_ground_lets_the_merchants_bonus_reach_food() {
    let t = Tuning::RON;
    let g = good_table();
    let out = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        0,
        1,
        true,
        false,
        false,
    );
    assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE * 120 / 100);
    assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE * 120 / 100);
}

/// A good that is **not** one of the two water ones takes no fishermen
/// bonus at all, however many upgrades the player holds.
#[test]
fn a_land_rare_never_takes_the_fishermen_bonus() {
    let t = Tuning::RON;
    let mut g = good_table();
    g[20].bonus = [(Some(Resource::Metal), 10), (None, 0)];
    let out = economy::calc_rare(&t, 20, &g[20], 3, 0, false, false, false);
    assert_eq!(out[Resource::Metal.index()], 10 * RATE_SCALE);
}

/// **The Nubians take half again, on a land rare in friendly ground**
/// (`NUBIAN_RARE`, `LeaderData::calc_rare`'s `else if` arm): Dye's
/// knowledge 160 is 240 (run492, `docs/GOLDEN.md` §54). Not abroad,
/// and not on the two water goods, whose arm is the fishermen's.
#[test]
fn a_nubian_merchant_s_land_rare_pays_half_again_at_home() {
    let t = Tuning::RON;
    let mut g = good_table();
    g[10].bonus = [
        (Some(Resource::Wealth), 10),
        (Some(Resource::Knowledge), 10),
    ];
    let home = economy::calc_rare(&t, 10, &g[10], 0, 0, true, true, false);
    assert_eq!(home[Resource::Knowledge.index()], 240);
    assert_eq!(home[Resource::Wealth.index()], 240);
    let abroad = economy::calc_rare(&t, 10, &g[10], 0, 0, false, true, false);
    assert_eq!(abroad[Resource::Knowledge.index()], 160, "abroad");
    let other = economy::calc_rare(&t, 10, &g[10], 0, 0, true, false, false);
    assert_eq!(other[Resource::Knowledge.index()], 160, "not Nubian");
    let fish = economy::calc_rare(
        &t,
        economy::FISH,
        &g[economy::FISH],
        0,
        0,
        true,
        true,
        false,
    );
    assert_eq!(fish[Resource::Wealth.index()], 10 * RATE_SCALE, "a fish");
}

/// **A Nubian Merchant is born with half again** (`Unit::update_hits`'
/// `NUBIAN_HIT_POINTS` arm): run492's `0/6`, 135 on a type of 90
/// (`docs/GOLDEN.md` §54). Another type of the same leader, and the
/// same Merchant of another nation, take the type's.
#[test]
fn a_nubian_merchant_is_born_with_half_again() {
    let (mut s, _) = sea_sim();
    let merchant = s.add_unit_type(crate::UnitType {
        hits: 90,
        type_index: 0x3d,
        ..crate::UnitType::default()
    });
    let other = s.add_unit_type(crate::UnitType {
        hits: 90,
        type_index: 0x40,
        ..crate::UnitType::default()
    });
    for p in &mut s.tech {
        p.has_city = true;
    }
    s.tech[0].power = Some(crate::ai::tribe::NUBIANS);
    s.tech[1].power = Some(crate::ai::tribe::NUBIANS + 1);
    assert_eq!(s.unit_hits(0, merchant), 135);
    assert_eq!(s.unit_hits(0, other), 90, "not a trader");
    assert_eq!(s.unit_hits(1, merchant), 90, "not Nubian");
}

/// **A supply unit's hits climb with the supply upgrades its leader
/// holds** (`Unit::update_hits`' last term, `SUPPLY_HP_UPGRADE`
/// `[0, 20, 40, 60]`; `docs/AI.md` §157): the Persian wagon `1/67` is 150
/// on a type of 90 once all three steps are held, and so is a government
/// patriot, which carries the General's bit as well (§213); a General or
/// any other type takes the type's.
///
/// Made to fail once with the term dropped: the wagon stays at 90.
#[test]
fn a_supply_unit_adds_the_supply_hp_upgrade() {
    use crate::ai_load::uflags2;
    let mut s = crate::Sim::new(Tuning::RON, World::new(20, 20), 2);
    let mut tree = tech::TechTree::new();
    let steps = [
        tree.add(tech::TypeDef::plain("step one", 0)),
        tree.add(tech::TypeDef::plain("step two", 0)),
        tree.add(tech::TypeDef::plain("step three", 0)),
    ];
    tree.roles.supply_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let ty = |flags2: u32| crate::UnitType {
        hits: 90,
        cols: crate::ai_load::UnitCols {
            unit_flags2: flags2,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    };
    let wagon = s.add_unit_type(ty(uflags2::SUPPLY_OR_HERO));
    let general = s.add_unit_type(ty(uflags2::GENERAL));
    let patriot = s.add_unit_type(ty(uflags2::GENERAL | uflags2::SUPPLY_OR_HERO));
    let other = s.add_unit_type(ty(0));
    assert_eq!(s.unit_hits(0, wagon), 90);
    for (step, expected) in [(steps[0], 110), (steps[1], 130), (steps[2], 150)] {
        s.gain_tech(0, step);
        assert_eq!(s.unit_hits(0, wagon), expected);
    }
    assert_eq!(s.unit_hits(1, wagon), 90, "another leader");
    assert_eq!(s.unit_hits(0, general), 90, "a General is not a wagon");
    // A government patriot (`FROM General`, `is(BASE_GOV_HEROTYPES)`)
    // carries both bits and takes the supply term as well
    // (`docs/AI.md` §213): run820's Despot `1/191` is 659, not 599.
    assert_eq!(s.unit_hits(0, patriot), 150, "a patriot is a wagon too");
    assert_eq!(s.unit_hits(0, other), 90);
}

/// **A Spy climbs with the spy upgrades** (`Unit::update_hits@0060e930`,
/// `update_los@0060e4d0` and `update_speed@006055c0`, item 1585,
/// `docs/AI.md` §160): at level 3 the Spy of Great Sahara's `1/103` is
/// 150 hit points (the `SPY_UPGRADE_HP` entry, not the type's plus a
/// term), sees two tiles more per level and walks a quarter faster per
/// level — 15, 8 and 21 at level 0 and 150, 14 and 36 at level 3 — and
/// a Citizen takes none of it. Made to fail with each term dropped.
#[test]
fn a_spy_climbs_with_the_spy_upgrades() {
    let mut s = crate::Sim::new(Tuning::RON, World::new(20, 20), 2);
    let mut tree = tech::TechTree::new();
    let steps = [
        tree.add(tech::TypeDef::plain("step one", 0)),
        tree.add(tech::TypeDef::plain("step two", 0)),
        tree.add(tech::TypeDef::plain("step three", 0)),
    ];
    while tree.types.len() < 0x3a {
        tree.add(tech::TypeDef::building("pad"));
    }
    let spy_t = tree.add(tech::TypeDef::unit("Spy", tech::UnitTraits::default()));
    let citizen_t = tree.add(tech::TypeDef::unit("Citizen", tech::UnitTraits::default()));
    tree.roles.spy_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let ty = |t| crate::UnitType {
        tree: Some(t),
        hits: 15,
        los: 8,
        moves: 21,
        ..crate::UnitType::default()
    };
    let spy = s.add_unit_type(ty(spy_t));
    let citizen = s.add_unit_type(ty(citizen_t));
    let mut unit = crate::Unit::new(0, 0, crate::Pos::new(1000, 1000), 40);
    unit.ty = Some(spy);
    let u = s.add_unit(unit);
    assert_eq!(
        (s.unit_hits(0, spy), s.unit_los(u), s.type_speed(0, spy)),
        (15, 8, 21)
    );
    for step in steps {
        s.gain_tech(0, step);
    }
    assert_eq!(
        (s.unit_hits(0, spy), s.unit_los(u), s.type_speed(0, spy)),
        (150, 14, 36)
    );
    assert_eq!(s.unit_hits(0, citizen), 15, "a Citizen takes none of it");
    assert_eq!(s.type_speed(0, citizen), 21);
    assert_eq!(s.unit_hits(1, spy), 15, "another leader");
}

/// **A General climbs with the General upgrades** (`get_general_upgrade
/// @006e0830`, item 1614, `docs/AI.md` §172): with all three of
/// `GENERALS_UPGRADE_1..3`' prerequisites held, the coverage pair's
/// General `1/150` is born at 599 hit points on a type of 109 (`+ 3² ×
/// 109 / 2`), sees 14 on 8 (`+ 3 × GENERAL_UPGRADE_LOS`), walks 73 on 42
/// (`+ 3 × 42 / 4`) and reaches 18 tiles on `GENERAL_RADIUS` 6
/// (`× (3 + 3) / 2`) — 109, 8, 42 and 9 with none. A Citizen takes none
/// of it. Made to fail with each term dropped.
#[test]
fn a_general_climbs_with_the_general_upgrades() {
    let mut s = crate::Sim::new(Tuning::RON, World::new(20, 20), 2);
    let mut tree = tech::TechTree::new();
    let steps = [
        tree.add(tech::TypeDef::plain("step one", 0)),
        tree.add(tech::TypeDef::plain("step two", 0)),
        tree.add(tech::TypeDef::plain("step three", 0)),
    ];
    tree.roles.general_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let ty = |flags2| crate::UnitType {
        hits: 109,
        los: 8,
        moves: 42,
        cols: crate::ai_load::UnitCols {
            unit_flags2: flags2,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    };
    let general = s.add_unit_type(ty(crate::ai_load::uflags2::GENERAL));
    let citizen = s.add_unit_type(ty(0));
    let mut unit = crate::Unit::new(0, 0, crate::Pos::new(1000, 1000), 40);
    unit.ty = Some(general);
    let g = s.add_unit(unit);
    let stats = |s: &crate::Sim| {
        (
            s.unit_hits(0, general),
            s.unit_los(g),
            s.type_speed(0, general),
            s.hero_radius(g),
        )
    };
    assert_eq!(stats(&s), (109, 8, 42, 9));
    for step in steps {
        s.gain_tech(0, step);
    }
    assert_eq!(s.general_upgrade_level(0), 3);
    assert_eq!(stats(&s), (599, 14, 73, 18));
    assert_eq!(s.unit_hits(0, citizen), 109, "a Citizen takes none of it");
    assert_eq!(s.type_speed(0, citizen), 42);
    assert_eq!(s.unit_hits(1, general), 109, "another leader");
}

/// **The walk, end to end.** One idle Fisherman on a fish: the rate is
/// the good's, the mask carries the fish's bit, and nothing else does.
#[test]
fn an_idle_fisherman_on_a_fish_pays_and_claims_it() {
    let (mut s, u) = sea_sim();
    put_good_beside(&mut s, u, economy::FISH);
    let (rate, owned) = s.gather_rares(1);
    assert_eq!(rate[Resource::Food.index()], 10 * RATE_SCALE);
    assert_eq!(rate[Resource::Wealth.index()], 10 * RATE_SCALE);
    assert_eq!(owned, 1 << (economy::FISH - economy::BASE_RARE));
    assert_eq!(s.units[u].rare, i32::try_from(economy::FISH).unwrap());
}

/// **A packed boat is not in the walk at all** — `calc_gather`'s very
/// first test, and the only caller that has it. It is why the third
/// Fisherman of run63, still packed and still sailing, claims nothing
/// while the two deployed ones do.
#[test]
fn a_packed_fisherman_claims_nothing() {
    let (mut s, u) = sea_sim();
    put_good_beside(&mut s, u, economy::FISH);
    s.units[u].combat.packed = true;
    let (rate, owned) = s.gather_rares(1);
    assert_eq!(rate, [0; RESOURCES]);
    assert_eq!(owned, 0);
    assert_eq!(s.units[u].rare, -1, "and the field is cleared on the way");
}

/// **A boat with an order earns nothing.** `order_type() == NONE` is
/// step 6's whole membership test, so a fisherman between two
/// `think_fish` searches pays and one walking does not.
#[test]
fn a_fisherman_under_orders_is_not_walked() {
    let (mut s, u) = sea_sim();
    put_good_beside(&mut s, u, economy::FISH);
    let dest = Pos::new(30 * 768, 30 * 768);
    s.add_move_order(
        u,
        dest,
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    let (rate, owned) = s.gather_rares(1);
    assert_eq!(rate, [0; RESOURCES]);
    assert_eq!(owned, 0);
}

/// **The crowd divides the payout, and it divides per unit.** Three
/// boats of the same lineage close enough to share one deposit each pay
/// `160 / 3` — and the truncation is each unit's own, so the deposit
/// pays **159** rather than 160. Without the division it would pay 480,
/// which is why a second fisherman on the same fish is worth nothing.
#[test]
fn a_shared_deposit_is_divided_per_boat_and_truncated() {
    let (mut s, u) = sea_sim();
    put_good_beside(&mut s, u, economy::FISH);
    let ty = s.units[u].ty;
    let at = s.units[u].pos;
    for o in 15..17 {
        let mut other = crate::Unit::new(1, o, at, 20);
        other.ty = ty;
        let i = s.add_unit(other);
        s.units[i].combat.packed = false;
    }
    let (rate, _) = s.gather_rares(1);
    assert_eq!(rate[Resource::Food.index()], 10 * RATE_SCALE / 3 * 3);
    assert_eq!(rate[Resource::Food.index()], 159);
    assert!(s.units[u].gather_here, "and each knows it is sharing");
}

/// **The whale, and the whole of what owning one does to a unit.**
/// A naval type's cached speed takes `WHALES_SHIPS_MOVE`; a land type's
/// does not; and neither moves until the mask actually carries the bit.
#[test]
fn a_whale_speeds_the_ships_and_nothing_else() {
    let (mut s, u) = sea_sim();
    let boat_ty = s.units[u].ty.unwrap();
    let foot_ty = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 25,
        ..crate::UnitType::default()
    });
    assert_eq!(s.type_speed(1, boat_ty), 40, "no whale, no bonus");
    s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
    assert!(s.has_rare(1, economy::WHALES));
    assert_eq!(s.type_speed(1, boat_ty), 40 * 120 / 100);
    assert_eq!(s.type_speed(1, foot_ty), 25, "a foot type is not naval");
    assert_eq!(s.type_speed(0, boat_ty), 40, "and it is the owner's rare");
}

/// **Aluminum speeds the aircraft and nothing else** (item 3742): an air
/// type's cached speed takes `ALUMINUM_AIR_SPEED`, truncating — the
/// German Modern word's Bomber 60 → 75, its `ME109` 75 → 93 and its
/// Albatross 55 → 68 — a sea type's does not, and the rare is the
/// owner's.
#[test]
fn aluminum_speeds_the_aircraft_and_nothing_else() {
    let (mut s, u) = sea_sim();
    let boat_ty = s.units[u].ty.unwrap();
    let plane = |s: &mut Sim, moves: i32| {
        s.add_unit_type(crate::UnitType {
            hits: 20,
            moves,
            kind: crate::attrition::UnitKind {
                domain: crate::attrition::Domain::Air,
                ..Default::default()
            },
            ..crate::UnitType::default()
        })
    };
    let bomber = plane(&mut s, 60);
    let fighter = plane(&mut s, 75);
    let biplane = plane(&mut s, 55);
    assert_eq!(s.type_speed(1, bomber), 60, "no aluminum, no bonus");
    s.ledgers[1].rare = 1 << (economy::ALUMINUM - economy::BASE_RARE);
    assert_eq!(s.type_speed(1, bomber), 75);
    assert_eq!(s.type_speed(1, fighter), 93, "75 x 125 / 100 truncates");
    assert_eq!(s.type_speed(1, biplane), 68);
    assert_eq!(s.type_speed(1, boat_ty), 40, "a ship is not of the air");
    assert_eq!(s.type_speed(0, bomber), 60, "and it is the owner's rare");
}

/// **The gunpowder foot line, by identity and truncating** (item 1109):
/// Infantry's `MOVES` 32 is 34, as run404 prints from 621; each sibling
/// takes its own scale and no other; a whale does not reach a foot type.
#[test]
fn the_gunpowder_foot_line_takes_its_own_scale() {
    let (mut s, _) = sea_sim();
    let foot = |s: &mut Sim, type_index: i32, moves: i32| {
        s.add_unit_type(crate::UnitType {
            hits: 20,
            moves,
            type_index,
            ..crate::UnitType::default()
        })
    };
    let inf = foot(&mut s, INFANTRY, 32);
    let mech = foot(&mut s, MECH_INFANTRY, 32);
    let rifle = foot(&mut s, RIFLEMAN, 32);
    let arq = foot(&mut s, ARQUEBUSIERS, 30);
    let other = foot(&mut s, INFANTRY + 1, 32);
    assert_eq!(s.type_speed(1, inf), 34, "run404's 621: 32 x 34 / 32");
    assert_eq!(s.type_speed(1, mech), 36);
    assert_eq!(s.type_speed(1, rifle), 37, "32 x 32 / 27 truncates");
    assert_eq!(s.type_speed(1, arq), 37, "30 x 30 / 24 truncates");
    assert_eq!(s.type_speed(1, other), 32, "identity, not a lineage");
    s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
    assert_eq!(s.type_speed(1, inf), 34, "a whale is the navy's");
}

/// **The Bantu move a citizen a quarter faster, and only for a Bantu
/// leader** (item 3343, `update_speed@006055c0`'s arm after the foot
/// line): `PEASANTS` and `PEASANTSKOREAN` by identity, `MOVES` 25 → 31
/// as run978 prints the Bantu word's citizens from block 1; a type of
/// neither identity, trainer nor line takes nothing.
#[test]
fn a_bantu_citizen_moves_a_quarter_faster() {
    let (mut s, _) = sea_sim();
    let foot = |s: &mut Sim, type_index: i32| {
        s.add_unit_type(crate::UnitType {
            hits: 40,
            moves: 25,
            type_index,
            ..crate::UnitType::default()
        })
    };
    let citizen = foot(&mut s, PEASANTS);
    let korean = foot(&mut s, PEASANTS_KOREAN);
    let other = foot(&mut s, PEASANTS_KOREAN + 1);
    assert_eq!(s.type_speed(1, citizen), 25, "not a Bantu leader");
    s.nation[1].bantu = true;
    assert_eq!(s.type_speed(1, citizen), 31, "run978: 25 x 125 / 100");
    assert_eq!(s.type_speed(1, korean), 31);
    assert_eq!(s.type_speed(1, other), 25, "neither citizen");
    assert_eq!(s.type_speed(0, citizen), 25, "and it is the owner's");
}

/// **The gunpowder foot line is a lineage, not an identity** (twenty-fourth
/// pass, group 22): `Unit::update_speed`'s four tests ask the type's vslot
/// `0x60`, which is `ObjectTypeData::is`, so a national replacement that
/// the tree puts under a line — by `FROM`, by `GRAFT`, or by the
/// `is_list` — takes the line's scale, the first of the four that
/// answers wins, and a type outside every line takes none.
#[test]
fn a_national_replacement_takes_its_line_s_gunpowder_scale() {
    let (mut s, _) = sea_sim();
    let at = |s: &mut Sim, idx: usize, def: tech::TypeDef| {
        while s.tech_tree.types.len() <= idx {
            s.tech_tree.add(tech::TypeDef::plain("filler", -1));
        }
        s.tech_tree.types[idx] = def;
    };
    let traits = tech::UnitTraits::default;
    // The four lines themselves, then one replacement under each by a
    // different arm of `is`, then one under two lines at once.
    for (idx, name) in [
        (ARQUEBUSIERS, "Arquebusier"),
        (RIFLEMAN, "Rifleman"),
        (INFANTRY, "Infantry"),
        (MECH_INFANTRY, "Mechanized Infantry"),
    ] {
        at(&mut s, idx as usize, tech::TypeDef::unit(name, traits()));
    }
    let by_from = 0x70;
    let by_graft = 0x71;
    let by_list = 0x72;
    let both = 0x73;
    let stranger = 0x74;
    at(
        &mut s,
        by_from,
        tech::TypeDef::unit("Redcoat", traits()).from(INFANTRY as usize),
    );
    let mut graft = tech::TypeDef::unit("Sepoy", traits());
    graft.graft = Some(ARQUEBUSIERS as usize);
    at(&mut s, by_graft, graft);
    let mut list = tech::TypeDef::unit("Jaeger", traits());
    list.is_list.push(RIFLEMAN as usize);
    at(&mut s, by_list, list);
    let mut two = tech::TypeDef::unit("Grenzer", traits());
    two.is_list = vec![ARQUEBUSIERS as usize, RIFLEMAN as usize];
    at(&mut s, both, two);
    at(&mut s, stranger, tech::TypeDef::unit("Cow", traits()));
    let line = |s: &mut Sim, idx: usize, moves: i32| {
        s.add_unit_type(crate::UnitType {
            hits: 20,
            moves,
            type_index: idx as i32,
            ..crate::UnitType::default()
        })
    };
    let a = line(&mut s, by_from, 32);
    let b = line(&mut s, by_graft, 30);
    let c = line(&mut s, by_list, 32);
    let d = line(&mut s, both, 32);
    let e = line(&mut s, stranger, 32);
    assert_eq!(s.type_speed(1, a), 34, "FROM Infantry: 32 x 34 / 32");
    assert_eq!(s.type_speed(1, b), 37, "GRAFT Arquebusiers: 30 x 30 / 24");
    assert_eq!(s.type_speed(1, c), 37, "is_list Rifleman: 32 x 32 / 27");
    assert_eq!(
        s.type_speed(1, d),
        37,
        "Rifleman is asked before Arquebusiers"
    );
    assert_eq!(s.type_speed(1, e), 32, "no line, no scale");
}

/// **Peacocks, the one rare the population cap reads** (item 1147):
/// `calc_pop_cap` tests the mask on every call, so a recompute for any
/// cause sees it, and it is the owner's alone.
#[test]
fn peacocks_raise_the_owner_s_pop_cap_by_a_tenth() {
    let (mut s, _) = sea_sim();
    for m in &mut s.muster {
        m.military_level = 1;
    }
    s.recompute_pop_caps();
    assert_eq!((s.muster[0].cap, s.muster[1].cap), (50, 50));
    s.ledgers[1].rare = 1 << (economy::PEACOCKS - economy::BASE_RARE);
    s.recompute_pop_caps();
    assert_eq!(
        (s.muster[0].cap, s.muster[1].cap),
        (50, 55),
        "Great Sahara's 55: 50 x 110 / 100"
    );
    s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
    s.recompute_pop_caps();
    assert_eq!(s.muster[1].cap, 50, "no other rare moves it");
}

/// **And the mask's own change recomputes it** — `calc_gather`'s
/// `operator!=` arm calls `calc_pop_cap` after the assignment
/// (`006ceee0:353`), so the cap follows the Peacocks bit on the frame
/// the bit moves, with nothing else changing.
#[test]
fn the_rare_mask_s_change_recomputes_the_pop_cap() {
    let (mut s, _) = sea_sim();
    for m in &mut s.muster {
        m.military_level = 1;
    }
    s.recompute_pop_caps();
    let bit = 1u64 << (economy::PEACOCKS - economy::BASE_RARE);
    s.holdings[1].rare_owned = bit;
    s.ledgers[1].dirty = false;
    s.ledgers[1].gather_stamp = s.frame;
    assert!(!s.holdings_due(1, s.frame), "the walk does not rebuild it");
    s.tick();
    assert_eq!(s.ledgers[1].rare, bit);
    assert_eq!(s.muster[1].cap, 55, "the bit arrived and the cap moved");
    s.holdings[1].rare_owned = 0;
    s.ledgers[1].dirty = false;
    s.ledgers[1].gather_stamp = s.frame;
    s.tick();
    assert_eq!(s.muster[1].cap, 50, "and it goes when the bit goes");
}

/// The truncation is the original's: `38 × 120 / 100` is **45**, which
/// is what run63's dump prints for all three of the AI's Fishermen from
/// frame 5552 — and 25 becomes 30 for its Transport Barge.
#[test]
fn the_whales_bonus_truncates_where_the_dump_does() {
    let (mut s, u) = sea_sim();
    let boat_ty = s.units[u].ty.unwrap();
    s.unit_types[boat_ty].moves = 38;
    s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
    assert_eq!(s.type_speed(1, boat_ty), 45);
    s.unit_types[boat_ty].moves = 25;
    assert_eq!(s.type_speed(1, boat_ty), 30);
}

/// **`calc_unit_stats` reaches every live unit of the player**, which
/// is how a boat already on the water gets the new speed on the frame
/// the whale is claimed rather than the next time it is built.
#[test]
fn calc_unit_stats_re_caches_every_unit_s_speed() {
    let (mut s, u) = sea_sim();
    assert_eq!(s.units[u].movement.speed, 0, "the fixture never set one");
    s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
    s.calc_unit_stats(1);
    assert_eq!(s.units[u].movement.speed, 40 * 120 / 100);
}

#[test]
fn supply_upgrade_speed_counts_sparse_steps_and_refreshes_existing_units() {
    use crate::ai_load::uflags2;
    let mut s = crate::Sim::new(Tuning::RON, World::new(20, 20), 2);
    let mut tree = tech::TechTree::new();
    let steps = [
        tree.add(tech::TypeDef::plain("step one", 0)),
        tree.add(tech::TypeDef::plain("step two", 0)),
        tree.add(tech::TypeDef::plain("step three", 0)),
    ];
    tree.roles.supply_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let wagon = s.add_unit_type(crate::UnitType {
        moves: 25,
        hits: 90,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::SUPPLY_OR_HERO,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let hero = s.add_unit_type(crate::UnitType {
        moves: 25,
        hits: 90,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::GENERAL,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let mut unit = crate::Unit::new(0, 0, Pos::new(792, 792), 90);
    unit.ty = Some(wagon);
    let u = s.add_unit(unit);
    s.calc_unit_stats(0);
    assert_eq!(s.units[u].movement.speed, 25);
    // Grant the last step first: this is a count, not a prefix.
    for (step, expected) in [(steps[2], 31), (steps[0], 37), (steps[1], 43)] {
        let old = s.units[u].movement.speed;
        s.gain_tech(0, step);
        assert_eq!(s.type_speed(0, wagon), expected);
        assert_eq!(s.type_speed(1, wagon), 25, "other owner");
        assert_eq!(s.type_speed(0, hero), 25, "general-only bit");
        assert_eq!(
            s.units[u].movement.speed, old,
            "cached until the leader pass"
        );
        s.tick();
        assert_eq!(s.units[u].movement.speed, expected);
    }
}

/// **A fisherman going idle drops the economy's period to eight
/// frames.** `Unit::check_idle`'s latch is what puts a boat that has
/// just arrived into step 6's walk within a handful of frames instead
/// of at the next 512-frame refresh — and without it run63's whale
/// would have been claimed 130 frames late.
#[test]
fn a_fisherman_going_idle_marks_the_economy_dirty() {
    let (mut s, u) = sea_sim();
    s.ledgers[1].dirty = false;
    s.units[u].idle_latch = false;
    s.tick();
    assert!(
        s.units[u].idle_latch,
        "the latch is taken on the idle frame"
    );
    assert!(s.ledgers[1].dirty, "and it is what marks the economy");
    // And it is a **latch**: standing there does not mark it again.
    s.ledgers[1].dirty = false;
    s.tick();
    assert!(!s.ledgers[1].dirty, "one transition, one mark");
}

/// **The idle latch is `0x18`** (item 3983, `check_idle@006032c0:69`,
/// `docs/AI.md` §415): the frame it is taken also raises `0x10`, the
/// bit `think`'s gate reads as `cant_reach`.
#[test]
fn the_idle_latch_raises_cant_reach_with_it() {
    let (mut s, u) = sea_sim();
    s.units[u].idle_latch = false;
    s.units[u].cant_reach = false;
    s.units[u].captain = false;
    s.tick();
    assert!(s.units[u].idle_latch);
    assert!(s.units[u].cant_reach, "a non-captain keeps it past think");
}

/// **A unit's cached speed comes from [`Sim::type_speed`], and the
/// rule is a guard rather than a sentence.**
///
/// `Unit::update_speed`'s Whales arm has been landed since run63 and
/// was applied at every spawn site but one: `cast_transport` set the
/// boat's speed from the type's raw `MOVES`, and that is the only
/// place in this crate where a **naval** unit is born. East Indies'
/// long word sat at 7448 for three sessions on that one line, and it
/// took a 272 MB capture to name it (run86, `docs/ORACLE.md`). Every
/// new spawn site is another chance to forget, and the failure is
/// silent everywhere the owner holds no rare — so the source itself
/// is read here. Made to fail on the three gaia sites it found
/// (`farms.rs`, `gaia.rs`, the dock's gull), each of which was
/// behaviour-identical only because owner 9 holds nothing.
#[test]
fn a_unit_s_speed_is_never_set_from_the_raw_moves() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut bad: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(dir).expect("the crate's own src/") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        // `rares.rs` is where `type_speed` reads it, which is the
        // point of the rule and not a breach of it.
        if name == "rares.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a readable source file");
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("movement.speed =") && code.contains(".moves") {
                bad.push(format!("{name}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a spawn site sets `movement.speed` from the type's raw `MOVES`;              use `Sim::type_speed`, which is `Unit::update_speed`: {bad:#?}"
    );
}
