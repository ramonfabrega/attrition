use super::*;
use crate::Tuning;
use crate::orders::{Body, index};
use crate::world::{Cell, Good, Terrain, World};

thread_local! {
    /// `find_merchant_spot`'s calls on this test's thread.
    pub(super) static SPOT_SEARCHES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A 40 × 40 land world with a human (0) and a computer (1), and the
/// computer's Merchant — `TypeIndex` `0x3d`, packing — standing at
/// cell `(20, 20)`. Two regions: the west half is one, the east half
/// (the merchant's) another, which is what the `+100` needs.
fn merchant_sim() -> (Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 39));
    world.fill_region(Terrain::Land, Cell::new(20, 0), Cell::new(39, 39));
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities_state.nation[0].human = true;
    s.cities_state.nation[1].human = false;
    {
        let t = &mut s.tables.tech_tree;
        while t.types.len() < 0x3d {
            t.add(crate::tech::TypeDef::plain("filler", -1));
        }
        let m = t.add(crate::tech::TypeDef::unit(
            "Merchant",
            crate::tech::UnitTraits::default(),
        ));
        assert_eq!(m, 0x3d, "the fixture's ids are TypeIndex's");
    }
    s.tech = (0..2)
        .map(|_| crate::tech::PlayerTech::new(&s.tables.tech_tree))
        .collect();
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        tree: Some(0x3d),
        type_index: 0x3d,
        ..crate::UnitType::default()
    });
    s.tables.unit_types[t].combat.packs = true;
    let at = Pos::new(20 * UNITS_PER_CELL + 384, 20 * UNITS_PER_CELL + 384);
    let mut u = crate::Unit::new(1, 19, at, 20);
    u.ty = Some(t);
    u.type_index = 0x3d;
    let i = s.add_unit(u);
    (s, i)
}

/// Puts a good at a cell's centre and tells the computer it has seen
/// it. Answers the goods-list index the score reads.
fn a_good(s: &mut Sim, c: Cell) -> usize {
    let gi = s.world.add_good(Good {
        pos: Pos::new(
            c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        ),
        ty: 26,
        alive: true,
    });
    s.ai[1].new_rares.push(gi);
    gi
}

/// The craft table as `craftrules.xml` has it, for the one row these
/// tests need: 55 records, and the Merchant's `0x290` is the
/// twenty-eighth with a `JOB_TIME` of 148 (`docs/ORDERS.md` §6.9).
/// Without it the deploy's job time is zero and the first frame casts
/// outright, which hides everything that happens before the clock.
fn merchant_job_time(s: &mut Sim) {
    s.orders.spells = vec![crate::orders::SpellType::default(); 55];
    let row = usize::try_from(crate::orders::spell::UNPACK_MERCHANT - crate::orders::spell::FIRST)
        .unwrap();
    s.orders.spells[row].job_time = 148;
}

/// Re-arms the merchant for another think: packed again, and nothing
/// on its order list.
fn rearm(s: &mut Sim, u: usize) {
    s.units[u].combat.packed = true;
    s.close_orders(u);
}

/// Which good the `MOVE_TO` the think just issued points at. The order
/// carries the 48-snapped point, so the comparison is against the
/// snap of each good in turn.
fn walking_to(s: &Sim, u: usize) -> Option<usize> {
    let Some(Body::Move(m)) = s.current_order(u).map(|o| o.body) else {
        return None;
    };
    (0..s.world.goods().len()).find(|&i| crate::orders::snapped(s.world.goods()[i].pos) == m.dest)
}

/// **`do_cast`'s re-seat, on the first frame of the unpack**
/// (`docs/ORDERS.md` §6.9 step 2, `docs/ANIM.md` §4.10). The merchant
/// is standing on its own unit-cell centre, so the point does not
/// move; what the call is for is the crew, which is *put* on its
/// offset instead of walking four more frames and paying an arrival
/// stand the original never pays. Great Lakes 6151 → 6463.
#[test]
fn the_unpack_s_first_frame_seats_the_merchant_s_crew() {
    let (mut s, u) = merchant_sim();
    // **On its own unit-cell centre**, which is where a merchant that
    // has walked to a tile corner stands and is the whole point: the
    // re-seat's `set_new_location` then has `from == to`, and used to
    // return before it reached the crew. The fixture's world-cell
    // centre is a 48-grid *boundary*, not a centre.
    let at = crate::collide::ucell_centre(crate::collide::ucell(s.units[u].pos));
    assert!(s.set_new_location(u, at, true));
    assert_eq!(crate::collide::ucell_centre(crate::collide::ucell(at)), at);
    // The tile has to answer `good_merchant_spot`, whose last test is
    // `calc_gather` — a good on the tile, and the tile's own
    // `AS_BUILDING` bit.
    s.world.add_good(Good {
        pos: at,
        ty: 26,
        alive: true,
    });
    let t = at.tile();
    let m = s.world.tile_mask(t);
    s.world.set_tile_mask(t, m | tile::AS_BUILDING);

    // A crew figure with a track offset, seated and then dragged
    // behind — the state the Merchant `1/24` is in when it arrives.
    s.units[u].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
    s.roster.art.tracks.insert(2, (-48, -192));
    s.seat_guys(u);
    let seated = s.units[u].guys[1].follow.expect("a tracked crew guy").des;
    s.units[u].guys[1].follow.as_mut().unwrap().body.pos = Pos::new(seated.x - 300, seated.y - 300);
    s.units[u].guys[1].anim = crate::anim::JOG;

    // `craftrules.xml` gives the Merchant's `0x290` a `JOB_TIME` of
    // 148 (`docs/ORDERS.md` §6.9), so the first frame seats and starts
    // the clock rather than deploying outright.
    merchant_job_time(&mut s);

    s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
    let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
        panic!("a cast order")
    };
    s.do_cast(u, c);

    let f = s.units[u].guys[1].follow.expect("still tracked");
    assert_eq!(
        f.body.pos, seated,
        "the crew figure is teleported, not left to walk"
    );
    assert_eq!(
        f.facing, s.units[u].movement.facing,
        "with its driver's facing, as run75's block 6145 has it"
    );
    assert_eq!(s.units[u].pos, at, "and the merchant itself has not moved");
    assert_eq!(
        s.units[u].guys[0].anim,
        crate::anim::ANIM_UNPACK,
        "the animation the re-seat comes just ahead of"
    );
    assert_eq!(s.units[u].spell_time, 1, "and the clock has started");
}

/// **The spot's third test is the quick collision** (`Unit::find_merchant_spot
/// @00603ab0`: `detect_unit_collision(x·192, y·192, 1, 1, 0, 0, 0)` at the
/// tile's corner): a candidate whose corner cell holds another unit is
/// refused, and the walk goes on to the next tile of the ring. Great
/// Sahara's Merchant `1/38` stood on the rare `8/6` on 2205 and the
/// original took `(−1, −1)` where this crate took `(0, 0)` (item 1611).
#[test]
fn a_unit_on_a_candidate_s_corner_refuses_the_spot() {
    let (mut s, u) = merchant_sim();
    // `BLOCK_RADIUS 1`, `coll_size 1` — the profile that blocks.
    let rec = s.units[u].ty.expect("a type");
    s.tables.unit_types[rec].combat.block_radius = 48;
    s.tables.unit_types[rec].combat.big_radius = 48;
    s.tables.unit_types[rec].combat.uber_size = 1;
    // The merchant stands inside tile `t`, off its corner's unit cell.
    let t = Pos::new(82, 106);
    let corner = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
    let at = crate::collide::ucell_centre(Pos::new(
        crate::collide::ucell(corner).x + 2,
        crate::collide::ucell(corner).y + 2,
    ));
    assert!(s.set_new_location(u, at, true));
    assert_eq!(at.tile(), t);
    // A rare on the tile, so `calc_gather` answers at every ring tile
    // the walk reaches first.
    for (dx, dy) in [(0, 0), (-1, -1)] {
        let p = Pos::new((t.x + dx) * UNITS_PER_TILE, (t.y + dy) * UNITS_PER_TILE);
        s.world.add_good(Good {
            pos: p,
            ty: 26,
            alive: true,
        });
        let q = p.tile();
        let m = s.world.tile_mask(q);
        s.world.set_tile_mask(q, m | tile::AS_BUILDING);
    }
    assert_eq!(s.find_merchant_spot(u, 1), Some(t), "nothing in the way");
    // Another unit on the candidate's corner cell.
    let ty = s.units[u].ty;
    let mut b = crate::Unit::new(
        0,
        20,
        crate::collide::ucell_centre(crate::collide::ucell(corner)),
        20,
    );
    b.ty = ty;
    b.type_index = 0x3d;
    s.add_unit(b);
    assert_ne!(
        s.find_merchant_spot(u, 1),
        Some(t),
        "the quick collision refuses the corner's tile"
    );
}

/// **`cast_unpack`'s merchant arm** (`docs/MERCHANT.md` §3.2): the
/// deploy seats the trader on its tile's corner — `(−24, −24)` from the
/// unit-cell centre it walked to — blocks the two-by-two under it and
/// marks the economy; `Unit::close` gives the four tiles back. East
/// Indies' `1/20` on 7663 is the dump this is the shape of (run88).
#[test]
fn a_deployed_merchant_sits_on_its_corner_and_blocks_its_square() {
    let (mut s, u) = merchant_sim();
    let at = crate::collide::ucell_centre(crate::collide::ucell(s.units[u].pos));
    assert!(s.set_new_location(u, at, true));
    s.world.add_good(Good {
        pos: at,
        ty: 26,
        alive: true,
    });
    let t = at.tile();
    let m = s.world.tile_mask(t);
    s.world.set_tile_mask(t, m | tile::AS_BUILDING);
    let square = [(0, 0), (-1, 0), (0, -1), (-1, -1)].map(|(dx, dy)| Pos::new(t.x + dx, t.y + dy));
    assert!(
        square
            .iter()
            .all(|&p| s.world.tile_mask(p) & tile::BLOCKED == 0),
        "the square is clear before the deploy"
    );
    s.economy.ledgers[1].dirty = false;
    // No craft table: a `JOB_TIME` of 0, so the first frame casts.
    s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
    let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
        panic!("a cast order")
    };
    s.do_cast(u, c);
    assert!(!s.units[u].combat.packed, "deployed");
    let corner = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
    assert_eq!(s.units[u].pos, corner, "seated on the tile's corner");
    assert_eq!(
        (at.x - corner.x, at.y - corner.y),
        (24, 24),
        "a step of (−24, −24) from where it stood"
    );
    assert!(
        square
            .iter()
            .all(|&p| s.world.tile_mask(p) & tile::BLOCKED != 0),
        "the four tiles under it are blocked"
    );
    assert!(
        s.economy.ledgers[1].dirty,
        "and the leader's 0x2000000 is up"
    );
    // Its death gives the square back.
    s.units[u].health = 0;
    s.close_supply(u);
    assert!(
        square
            .iter()
            .all(|&p| s.world.tile_mask(p) & tile::BLOCKED == 0),
        "a dead merchant's square is clear again"
    );
}

/// The re-seat's own gate: a tile `good_merchant_spot` refuses kills
/// the order where it stands, before the clock ever starts.
#[test]
fn an_unpack_on_a_bad_spot_dies_on_its_first_frame() {
    let (mut s, u) = merchant_sim();
    merchant_job_time(&mut s);
    s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
    let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
        panic!("a cast order")
    };
    s.do_cast(u, c);
    assert!(s.units[u].orders.is_empty(), "no good under it, no deploy");
    assert_eq!(s.units[u].spell_time, 0, "and the clock never started");
    assert!(s.units[u].combat.packed, "still packed");
}

/// §2.1 — and it is a `1`, not a `0`: the deployed merchant ends the
/// think where it stands.
#[test]
fn a_deployed_merchant_ends_the_think_and_searches_nothing() {
    let (mut s, u) = merchant_sim();
    assert!(s.units[u].combat.packed, "a merchant is born packed");
    let west = a_good(&mut s, Cell::new(5, 20));
    s.units[u].combat.packed = false;
    assert!(s.think_merchant(u), "a deployed merchant answers 1");
    assert!(s.units[u].orders.is_empty(), "and issues nothing");
    assert_eq!(s.ai[1].new_rares, vec![west], "and rotates nothing");
}

/// §4, item 3329: **the Dutch merchant also thinks on the auto-attack's
/// cadence** (`think@005f6e40:145`, `is(0x3e, 1)` and packed), one
/// frame in thirty-two, where every other merchant waits for the
/// tail's one in a hundred and twenty-eight. On a frame that is the
/// first and not the second, a packed `MERCHANTDUTCH` walks at the good
/// and a plain `MERCHANT` does not (run963's `1/2` on 1566).
#[test]
fn a_dutch_merchant_searches_one_frame_in_thirty_two() {
    for (ty, walks) in [(0x3e, true), (0x3d, false)] {
        let (mut s, u) = merchant_sim();
        let t = s.units[u].ty.unwrap();
        s.tables.unit_types[t].tree = Some(ty);
        s.tables.unit_types[t].type_index = ty as i32;
        s.units[u].type_index = ty as i32;
        let good = a_good(&mut s, Cell::new(5, 20));
        s.units[u].idle = 5;
        // `o + frame` a multiple of 32 and not of 128.
        let o = i64::from(s.units[u].index);
        let frame = 32 * 5 - o;
        s.think(u, frame);
        assert_eq!(
            walking_to(&s, u),
            walks.then_some(good),
            "TypeIndex {ty:#x} on a 32-frame that is not a 128-frame"
        );
    }
}

/// The Dutch per-age terms (item 3329, `docs/COMBAT.md` §4.3):
/// `Unit::update_hits`' `+ ages × DUTCH_HP_BONUS` and
/// `ObjectData::attack`'s and `::armor`'s `ages × DUTCH_ATTACK_BONUS`
/// (× 10 on the attack, inside the base, so a merchant with none of its
/// own is armed), on a Dutch leader's merchant and nobody else's.
#[test]
fn a_dutch_merchant_gains_hits_attack_and_armour_by_age() {
    use crate::combat::Obj;
    let (mut s, u) = merchant_sim();
    let rec = s.units[u].ty.unwrap();
    s.tech[1].ages = 2;
    s.tech[1].has_city = true;
    let o = Obj::Unit(u);
    let (hits, armor) = (s.unit_hits(1, rec), s.armor_of(o));
    assert_eq!(s.attack_of(o), 0, "a merchant has no attack of its own");
    s.tech[1].power = Some(crate::ai::tribe::DUTCH);
    let (hp, at) = (
        s.tables.tuning.dutch_hp_bonus,
        s.tables.tuning.dutch_attack_bonus,
    );
    assert_eq!(s.unit_hits(1, rec), hits + 2 * hp);
    assert_eq!(s.attack_of(o), 2 * at * 10);
    assert_eq!(s.armor_of(o), armor + 2 * at);
    // The human's merchant of the same type takes none of it.
    assert_eq!(s.unit_hits(0, rec), hits);
}

/// `ObjectData::attack`'s caravan arm (item 3387): a caravan whose owner
/// holds `CARAVAN_ATTACK_BONUS`'s prerequisite has its attack
/// **replaced** by the constant (20), the Dutch term added on top; a
/// caravan without the tech, or a non-caravan with it, keeps its own.
#[test]
fn a_caravan_with_the_attack_bonus_has_its_attack_replaced() {
    use crate::combat::Obj;
    let (mut s, u) = merchant_sim();
    let rec = s.units[u].ty.unwrap();
    s.tables.unit_types[rec].combat.attack = 30;
    let o = Obj::Unit(u);
    let mut tree = s.tables.tech_tree.clone();
    let tech = tree.types.len();
    tree.types.push(crate::tech::TypeDef::new(
        "Caravan Attack",
        crate::tech::Kind::Final,
    ));
    tree.roles.caravan_attack_preq = Some(tech);
    s.set_tech_tree(tree);
    for p in &mut s.tech {
        p.tech.resize(tech + 1, false);
    }
    s.tech[1].tech[tech] = true;
    assert_eq!(s.attack_of(o), 30, "not a caravan: its own");
    s.tables.unit_types[rec].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;
    assert_eq!(
        s.attack_of(o),
        s.tables.tuning.caravan_attack_bonus,
        "replaced"
    );
    assert_eq!(s.tables.tuning.caravan_attack_bonus, 20);
    s.tech[1].tech[tech] = false;
    assert_eq!(s.attack_of(o), 30, "no tech: its own");
    s.tech[1].tech[tech] = true;
    s.tech[1].ages = 2;
    s.tech[1].has_city = true;
    s.tech[1].power = Some(crate::ai::tribe::DUTCH);
    assert_eq!(
        s.attack_of(o),
        20 + 2 * s.tables.tuning.dutch_attack_bonus * 10
    );
}

/// The caravan arm's gates, from the other side (item 3387): a tree that
/// names no prerequisite, or a player the tree has no row for, holds
/// the bonus for nobody; and the Dutch term's kinds are the merchant
/// ids and the caravan and supply flags, not a neighbouring id.
#[test]
fn the_caravan_arm_is_off_without_the_tech_and_the_dutch_kinds_are_exact() {
    use crate::combat::Obj;
    let (mut s, u) = merchant_sim();
    let rec = s.units[u].ty.unwrap();
    s.tables.unit_types[rec].combat.attack = 30;
    s.tables.unit_types[rec].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;
    let o = Obj::Unit(u);
    assert_eq!(s.tables.tech_tree.roles.caravan_attack_preq, None);
    assert_eq!(s.attack_of(o), 30, "no role named: no replacement");
    // Dutch: a neighbouring id is no merchant, a supply flag is one.
    s.tables.unit_types[rec].cols.unit_flags2 = 0;
    s.tables.unit_types[rec].type_index = 0x3c;
    s.tech[1].ages = 2;
    s.tech[1].has_city = true;
    s.tech[1].power = Some(crate::ai::tribe::DUTCH);
    assert_eq!(s.attack_of(o), 30, "0x3c is no merchant");
    s.tables.unit_types[rec].cols.unit_flags2 = crate::ai_load::uflags2::SUPPLY_OR_HERO;
    assert_eq!(
        s.attack_of(o),
        30 + 2 * s.tables.tuning.dutch_attack_bonus * 10
    );
    s.tables.unit_types[rec].cols.unit_flags2 = crate::ai_load::uflags2::FLAG2_CARAVAN;
    assert_eq!(
        s.attack_of(o),
        30 + 2 * s.tables.tuning.dutch_attack_bonus * 10
    );
}

/// §2.2's two terms that East Indies cannot separate: the good in the
/// merchant's **own** region wins from the back of the list, because
/// `+100` beats the `−10` a position costs.
#[test]
fn the_own_region_bonus_outweighs_the_list_position() {
    let (mut s, u) = merchant_sim();
    let west = a_good(&mut s, Cell::new(5, 20));
    let east = a_good(&mut s, Cell::new(30, 20));
    assert_ne!(
        s.world.region_of(Cell::new(5, 20)),
        s.world.region_of(Cell::new(30, 20)),
        "the fixture's two halves are two regions"
    );
    assert!(s.think_merchant(u), "it found one");
    assert_eq!(
        s.ai[1].new_rares,
        vec![west, east],
        "200 against 190 + 100: the winner is taken out and appended"
    );
    let dest = match s.current_order(u).map(|o| o.body) {
        Some(Body::Move(m)) => m.dest,
        other => panic!("a MOVE_TO, not {other:?}"),
    };
    assert_eq!(
        dest,
        crate::orders::snapped(s.world.goods()[east].pos),
        "the east good, 48-snapped"
    );
}

/// The same pair with the bonus taken away: position alone decides,
/// and the **head** of the list wins.
#[test]
fn with_no_bonus_the_head_of_the_list_wins() {
    let (mut s, u) = merchant_sim();
    let a = a_good(&mut s, Cell::new(5, 20));
    let b = a_good(&mut s, Cell::new(6, 20));
    assert!(s.think_merchant(u));
    assert_eq!(
        s.ai[1].new_rares,
        vec![b, a],
        "200 beats 190, and neither is in the merchant's region"
    );
}

/// And where the two terms **do** separate: eleven goods deep, the
/// `−10` a position costs has eaten the `+100` an own region pays, and
/// the head of the list wins anyway. This is the assertion the step
/// has of its own — with the `−10` dropped, the twelfth good scores
/// 300 and takes it.
#[test]
fn eleven_positions_outweigh_the_own_region_bonus() {
    let (mut s, u) = merchant_sim();
    let head = a_good(&mut s, Cell::new(5, 20));
    for y in 0..10 {
        a_good(&mut s, Cell::new(6, 20 + y));
    }
    // Index 11, in the merchant's own region: 200 − 110 + 100 = 190.
    let mine = a_good(&mut s, Cell::new(30, 20));
    assert_eq!(s.ai[1].new_rares.len(), 12);
    assert!(s.think_merchant(u));
    assert_eq!(
        walking_to(&s, u),
        Some(head),
        "200 for the head against 190 for the good in its own region"
    );
    assert_eq!(
        *s.ai[1].new_rares.last().expect("twelve"),
        head,
        "and the winner is the one appended"
    );
    assert_ne!(mine, head, "the two are different goods");
}

/// §2.2's first search: a sibling of my exact type standing within a
/// cell of the good's cell centre refuses the good outright.
#[test]
fn a_sibling_at_the_good_refuses_it() {
    let (mut s, u) = merchant_sim();
    let want = a_good(&mut s, Cell::new(30, 20));
    let other = a_good(&mut s, Cell::new(5, 20));
    let ty = s.units[u].ty;
    let mut v = crate::Unit::new(1, 20, s.world.goods()[want].pos, 20);
    v.ty = ty;
    v.type_index = 0x3d;
    let sib = s.add_unit(v);
    assert!(s.think_merchant(u), "it took the other one instead");
    assert_eq!(walking_to(&s, u), Some(other));
    s.units[sib].pos = Pos::new(0, 0);
    rearm(&mut s, u);
    assert!(s.think_merchant(u));
    assert_eq!(
        walking_to(&s, u),
        Some(want),
        "with the sibling gone the good is taken"
    );
}

/// The second search — and "ordered" is literal twice over. The
/// candidate must hold a move-family order, **and the distance is
/// measured to where it is going**: `find_unit_ordered` reads
/// `UnitData +0x70/+0x74` — `orders_x`/`orders_y` — where its sibling
/// `find_unit` reads the object's own XOR-ed position (`0065be35`
/// against `0065cd0f`, the listing; the decompiler prints both as
/// `vector_dist(unaff_EDI, unaff_ESI)`). So a merchant on the far side
/// of the map, already walking at a good, keeps every other merchant
/// off it — which is the whole of item 186 (`docs/JOURNAL.md`,
/// 2026-09-02).
#[test]
fn a_sibling_ordered_at_the_good_refuses_it_from_anywhere() {
    let (mut s, u) = merchant_sim();
    let want = a_good(&mut s, Cell::new(30, 20));
    let other = a_good(&mut s, Cell::new(5, 20));
    let ty = s.units[u].ty;
    let mut v = crate::Unit::new(1, 20, Pos::new(0, 0), 20);
    v.ty = ty;
    v.type_index = 0x3d;
    let sib = s.add_unit(v);
    assert!(s.think_merchant(u), "the sibling is nowhere near");
    assert_eq!(walking_to(&s, u), Some(want), "it took the good");

    // The sibling does not move: it is sent at the good from where it
    // stands, which is nowhere near it. The first search still passes
    // and the second is what refuses.
    let at = s.world.goods()[want].pos;
    s.add_move_order(sib, at, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(s.order_type(sib), index::MOVE_TO, "the move family");
    let stands = s.units[sib].pos;
    assert!(
        vector_dist(stands.x - at.x, stands.y - at.y) > FRIENDLY_RANGE,
        "the body is far outside the cell the search asks about"
    );
    let going = s.units[sib].orders_pos;
    assert!(
        vector_dist(going.x - at.x, going.y - at.y) <= FRIENDLY_RANGE,
        "and `orders_x/y` is the good, 48-snapped"
    );
    rearm(&mut s, u);
    assert!(s.think_merchant(u));
    assert_eq!(walking_to(&s, u), Some(other), "refused this time");

    // And it is the **order family** that arms it: with the move
    // killed, `orders_x/y` stops naming the good and the good is
    // taken again.
    s.close_orders(sib);
    s.units[sib].orders_pos = stands;
    rearm(&mut s, u);
    assert!(s.think_merchant(u));
    assert_eq!(
        walking_to(&s, u),
        Some(want),
        "nothing is ordered there now"
    );
}

/// The third: an enemy that can shoot, four cells out.
#[test]
fn an_enemy_that_can_shoot_refuses_it_at_four_cells() {
    let (mut s, u) = merchant_sim();
    let want = a_good(&mut s, Cell::new(30, 20));
    let other = a_good(&mut s, Cell::new(5, 20));
    let soldier = s.add_unit_type(crate::UnitType {
        hits: 30,
        ..crate::UnitType::default()
    });
    s.tables.unit_types[soldier].combat.attack = 5;
    // `SEARCH_ENEMY` is `LeaderData::is_enemy`, so a player merely not
    // allied is not searched at all.
    s.declare_war(0, 1);
    let mut e = crate::Unit::new(0, 1, s.world.goods()[want].pos, 30);
    e.ty = Some(soldier);
    let them = s.add_unit(e);
    assert!(s.think_merchant(u));
    assert_eq!(walking_to(&s, u), Some(other), "the good it can reach");
    s.units[them].pos = Pos::new(
        30 * UNITS_PER_CELL + UNITS_PER_CELL / 2 + ENEMY_RANGE + 1,
        30 * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    );
    rearm(&mut s, u);
    assert!(s.think_merchant(u));
    assert_eq!(
        walking_to(&s, u),
        Some(want),
        "one unit past `0xc00` and the good is acceptable again"
    );
}

/// A merchant with nothing to score answers 0 — which sends the
/// caller on to `Unit::think`'s tail — and a dead good is not a
/// destination.
#[test]
fn an_empty_list_answers_nothing() {
    let (mut s, u) = merchant_sim();
    assert!(!s.think_merchant(u), "no rares, no order");
    // A good the list names and `SubObjectData.flags & 1` denies.
    let gi = s.world.add_good(Good {
        pos: Pos::new(5 * UNITS_PER_CELL, 20 * UNITS_PER_CELL),
        ty: 26,
        alive: false,
    });
    s.ai[1].new_rares.push(gi);
    assert!(!s.think_merchant(u), "a dead good is not a destination");
    assert_eq!(s.ai[1].new_rares, vec![gi], "and nothing rotated");
}

/// §4's cadence, through `Unit::do_idle` — the first idle frame and
/// then one frame in 128, phased by `o`.
///
/// The probe is a count of `find_merchant_spot`'s calls: the spot
/// search hands `calc_gather` null out-pointers, so it leaves nothing
/// on the unit to read (item 3306; the probe was `rare`'s `−1` until
/// then, a write the original does not make).
#[test]
fn a_merchant_thinks_on_its_first_idle_frame_and_then_once_in_128() {
    let (mut s, u) = merchant_sim();
    let o = i64::from(s.units[u].index);
    let mut thought = Vec::new();
    for frame in 0..400 {
        let before = SPOT_SEARCHES.with(std::cell::Cell::get);
        s.units[u].rare = 7;
        s.do_idle(u, frame);
        assert_eq!(s.units[u].rare, 7, "the spot search writes no `rare`");
        if SPOT_SEARCHES.with(std::cell::Cell::get) != before {
            thought.push(frame);
        }
    }
    let want: Vec<i64> = std::iter::once(0)
        .chain((1..400).filter(|f| (f + o) % 128 == 0))
        .collect();
    assert_eq!(
        thought, want,
        "`idle == 1 || ((o + frame) & 127) == 0`, and the mod-32 gate \
             above it never keeps a frame this one wants"
    );
}

/// §5 — the crew figure a **trained** unit is born with sits on its
/// track offset, because `Unit::init` ends in
/// `set_new_location(·, ·, 1, 1)`.
#[test]
fn a_trained_unit_s_tracked_crew_is_seated_at_birth() {
    let (mut s, _) = merchant_sim();
    let t = s.tables.unit_types.len() - 1;
    s.tables.unit_types[t].combat.crew_size = 1;
    // `piece_of` falls back to the dump-keyed table when the install's
    // is empty, so the fixture states guy 1's piece the way a dump
    // would, and gives that piece a track.
    s.roster.art.pieces.insert((1, t, 1, 1), 4242);
    s.roster.art.tracks.insert(4242, (-48, -192));
    let mut v = crate::Unit::new(1, 21, Pos::new(9000, 9000), 20);
    v.ty = Some(t);
    v.type_index = 0x3d;
    let i = s.add_unit(v);
    s.init_guys(i, Some(t));
    assert_eq!(s.units[i].guys.len(), 2, "crew_size + squad_size");
    assert_eq!(s.units[i].guys[1].gpiece, 4242);
    assert!(
        s.units[i].guys[1].follow.is_some(),
        "a tracked crew figure is seated at birth — without it \
             `Guy::do_turn` recurses into it and it spends a draw the \
             original does not"
    );
}
