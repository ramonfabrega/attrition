use super::*;
use crate::build::{BuildType, Ident, flags};
use crate::tuning::Tuning;
use crate::world::{Terrain, UNITS_PER_TILE, World};
use crate::{UnitType, cost};

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(
        tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

/// A 12×8 world: land on the left eight cells, sea on the right four,
/// the sea tiles surfaced as ocean, the two shore columns marked the way
/// the map maker marks them (`land` SANDY/OCEAN on the water, the land
/// cells flagged coastal with the sea as `region2`).
struct Fix {
    sim: Sim,
    dock: usize,
    citizen: usize,
    scout: usize,
    ship: usize,
    land: u16,
    sea: u16,
}

fn fix() -> Fix {
    let mut w = World::new(12, 8);
    let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let sea = w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(11, 7));
    for y in 0..8 {
        for x in 0..12 {
            let c = Cell::new(x, y);
            let mut d = w.cell_data(c);
            if x >= 8 {
                d.land = 2;
                for tx in x * 4..x * 4 + 4 {
                    for ty in y * 4..y * 4 + 4 {
                        w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
                    }
                }
            } else if x == 7 {
                d.flags |= 0x100;
                d.region2 = Some(sea);
            }
            w.set_cell_data(c, d);
        }
    }
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.nation[1].human = false;
    let dock = sim.add_build_type(BuildType {
        ident: Ident::Dock,
        x_size: 4,
        y_size: 4,
        flags: flags::parse("eab"),
        job_time: 100,
        hits: 500,
        price: cost::Price {
            kind: cost::Kind::Building,
            ..cost::Price::free()
        },
        ..BuildType::default()
    });
    let citizen = sim.add_unit_type(UnitType {
        worker: Worker::Citizen,
        hits: 40,
        ..UnitType::default()
    });
    // No tree in this fixture: a `TypeIndex` is set after registration
    // so the lineage test short-circuits on equality.
    let scout = sim.add_unit_type(UnitType {
        hits: 40,
        ..UnitType::default()
    });
    sim.tables.unit_types[scout].tree = Some(ty::SCOUT);
    let mut ship_t = UnitType {
        hits: 100,
        ..UnitType::default()
    };
    ship_t.combat.domain = Domain::Sea;
    let ship = sim.add_unit_type(ship_t);
    Fix {
        sim,
        dock,
        citizen,
        scout,
        ship,
        land,
        sea,
    }
}

fn unit(sim: &mut Sim, who: Player, t: usize, at: Pos) -> usize {
    let index = sim.find_free(who, UNIT_BASE, BUILD_BASE).unwrap();
    let mut u = Unit::new(who, index, at, sim.tables.unit_types[t].hits);
    u.ty = Some(t);
    u.kind.domain = sim.tables.unit_types[t].combat.domain;
    sim.add_unit(u)
}

fn place_dock(sim: &mut Sim, who: Player, dock: usize, at: Pos) -> usize {
    let b = sim.add_building(who, at, 1);
    sim.buildings[b].ty = Some(dock);
    sim.buildings[b].orig_ty = Some(dock);
    sim.buildings[b].started = true;
    b
}

#[test]
fn transport_types_follow_the_kind_ladder() {
    let mut f = fix();
    let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
    let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
    let b = unit(&mut f.sim, 1, f.ship, tile_pos(36, 4));
    assert_eq!(f.sim.unit_transport_type(c), TransportType::Civilian);
    assert_eq!(f.sim.unit_transport_type(s), TransportType::Scout);
    assert_eq!(f.sim.unit_transport_type(b), TransportType::Military);
    // A land unit can ever transport; a warship without `carry` cannot.
    assert!(f.sim.unit_can_ever_transport(c));
    assert!(!f.sim.unit_can_ever_transport(b));
    f.sim.tables.unit_types[f.ship].cols.carry = 4;
    assert!(f.sim.unit_can_ever_transport(b));
}

#[test]
fn the_level_is_granted_by_a_dock_and_revoked_with_it() {
    let mut f = fix();
    let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
    let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
    // Born at level 0: no bit.
    assert!(!f.sim.units[c].auto_transport);
    assert_eq!(f.sim.transport_level(1), TransportType::None);
    let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 16));
    f.sim.activate(b, false, true);
    assert_eq!(f.sim.transport_level(1), TransportType::Civilian);
    assert!(f.sim.units[c].auto_transport);
    assert!(f.sim.units[s].auto_transport, "scouts under the option");
    assert!(f.sim.unit_can_transport(c));
    // A unit born now starts with the bit.
    let c2 = unit(&mut f.sim, 1, f.citizen, tile_pos(5, 5));
    assert!(f.sim.units[c2].auto_transport);
    // The other player has no dock and no level.
    let h = unit(&mut f.sim, 0, f.citizen, tile_pos(2, 2));
    assert!(!f.sim.units[h].auto_transport);
    // Losing the last dock revokes it for every unit at once.
    f.sim.close_building(b, false);
    assert_eq!(f.sim.transport_level(1), TransportType::None);
    assert!(!f.sim.units[c].auto_transport);
    assert!(!f.sim.units[c2].auto_transport);
    assert!(!f.sim.unit_can_transport(c));
}

#[test]
fn scouts_stay_off_without_the_option_and_the_veto_holds() {
    let mut f = fix();
    f.sim.transport[1].scouts_option = false;
    let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
    let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
    let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 16));
    f.sim.activate(b, false, true);
    assert!(!f.sim.units[s].auto_transport);
    assert!(f.sim.units[c].auto_transport);
    // The scenario veto beats the bit; the type flag beats the veto.
    f.sim.units[c].never_transport = true;
    assert!(!f.sim.unit_can_transport(c));
    f.sim.tables.unit_types[f.citizen].cols.unit_flags |= 0x10;
    assert!(f.sim.unit_can_transport(c));
    // The lock freezes the level.
    f.sim.transport[1].locked = true;
    f.sim.close_building(b, false);
    assert_eq!(f.sim.transport_level(1), TransportType::Civilian);
}

#[test]
fn the_registry_keeps_reg_docks_and_the_slot_rule() {
    let mut f = fix();
    let a = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
    f.sim.activate(a, false, true);
    let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 20));
    f.sim.activate(b, false, true);
    assert_eq!(f.sim.buildings[a].dock_slot, Some(0));
    assert_eq!(f.sim.buildings[b].dock_slot, Some(1));
    assert_eq!(f.sim.docks[1].mark, 2);
    assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 2);
    assert_eq!(f.sim.docks[1].slots[0].reg, Some(f.land));
    // Close the first: its slot frees, the mark stays at 2 (slot 1 is
    // still active); a third dock reuses slot 0.
    f.sim.close_building(a, false);
    assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 1);
    assert!(!f.sim.docks[1].slots[0].active());
    assert_eq!(f.sim.docks[1].mark, 2);
    let c = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
    f.sim.activate(c, false, true);
    assert_eq!(f.sim.buildings[c].dock_slot, Some(0));
    // Close both: the mark walks down to 0.
    f.sim.close_building(b, false);
    f.sim.close_building(c, false);
    assert_eq!(f.sim.docks[1].mark, 0);
    assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 0);
    assert_eq!(f.sim.transport_level(1), TransportType::None);
}

#[test]
fn a_dock_with_a_gull_type_draws_twice() {
    let mut f = fix();
    let gull = f.sim.add_unit_type(UnitType {
        hits: 10,
        gaia: true,
        ..UnitType::default()
    });
    f.sim.tables.unit_types[gull].tree = Some(ty::GULLBIRD);
    let seed0 = f.sim.rng.seed;
    let a = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
    f.sim.activate(a, false, true);
    let mut probe = f.sim.rng;
    probe.seed = seed0;
    // Two draws: the gull's creation (`Guy::init_real`) and its angle.
    probe.roll();
    probe.roll();
    assert_eq!(f.sim.rng.seed, probe.seed);
    let g = f.sim.docks[1].slots[0].gull.expect("a gull");
    assert_eq!(f.sim.units[g].owner, 9);
    assert_eq!(f.sim.units[g].ty, Some(gull));
    let dock_pos = f.sim.buildings[a].pos;
    // `Unit::init`'s tile snap on the `− 0xc0` point (item 1453).
    assert_eq!(
        f.sim.units[g].pos,
        Pos::new(
            crate::gaia::init_snap(dock_pos.x - GULL_OFFSET),
            crate::gaia::init_snap(dock_pos.y - GULL_OFFSET)
        )
    );
    // Closing the dock closes the gull, and the slot keeps its number
    // (`Dock::close` does not clear `gull_o`).
    f.sim.close_building(a, false);
    assert!(!f.sim.units[g].alive());
    assert!(!f.sim.docks[1].slots[0].active());
    assert_eq!(f.sim.docks[1].slots[0].gull, Some(g));
}

/// **A transport that reaches its oil platform dies there, and its
/// passenger is the platform's** (item 1594, `docs/AI.md` §165;
/// `Unit::do_gather`'s `can_carry(GROUND)` arm, `5efa39`): the boat goes
/// inside, its order is killed, and `Object::die`'s `remove_from_inside`
/// splices the citizen into its place. run711 block 1954: `1/26`
/// `inside_up 2031`, and the Freighter `1/37` gone. A sea unit that
/// carries nothing is not a transport and takes no part of the arm.
///
/// Made to fail by leaving the arm out of `do_gather`: the boat stands
/// in the garrison with the citizen still aboard.
#[test]
fn a_transport_at_its_oil_platform_dies_and_hands_in_its_passenger() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[b].cols.carry = 1;
    let platform = f.sim.add_build_type(BuildType {
        ident: Ident::OilPlatform,
        x_size: 4,
        y_size: 4,
        flags: flags::parse("igbe"),
        hits: 500,
        ..BuildType::default()
    });
    let site = place_dock(&mut f.sim, 1, platform, tile_pos(40, 16));
    f.sim.activate(site, false, true);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let boat = unit(&mut f.sim, 1, b, tile_pos(38, 16));
    f.sim.board(rider, boat);
    f.sim
        .add_gather_order(boat, site, crate::orders::QueuePos::New, false);
    // Its own step: `Unit::process` → `do_gather`'s arrival.
    assert!(
        f.sim.adjacent_to(boat, site),
        "the boat stands at the platform"
    );
    f.sim.work(boat, 0);
    assert!(!f.sim.units[boat].alive(), "the boat dies");
    assert_eq!(f.sim.units[boat].hold_frames, Sim::CLOSE_HOLD);
    assert!(f.sim.units[boat].orders.is_empty(), "its gather is killed");
    assert_eq!(f.sim.units[rider].inside, Some(site));
    assert_eq!(f.sim.units[rider].inside_unit, None);
    assert_eq!(f.sim.buildings[site].garrison, vec![rider]);
    // A ship with no `carry` is no transport: the arm passes it by.
    let ship = unit(&mut f.sim, 1, f.ship, tile_pos(38, 20));
    assert!(!f.sim.can_carry_ground(ship));
    assert!(f.sim.units[ship].alive());
}

/// A barge type, so `cast_transport` has something to build.
fn barge(sim: &mut Sim) -> usize {
    let mut t = UnitType {
        hits: 50,
        moves: 25,
        ..UnitType::default()
    };
    t.combat.domain = Domain::Sea;
    t.combat.block_radius = 48;
    let b = sim.add_unit_type(t);
    sim.tables.unit_types[b].tree = Some(ty::TRANSPORTBARGE);
    sim.tables.unit_types[b].type_index = ty::TRANSPORTBARGE as i32;
    b
}

/// **`action_move_near`'s different-region arm sweeps as the leader's
/// barge** (`705c0f`–`705c9b`, item 3522, `docs/AI.md` §304): a land
/// member whose slot 0 lies on the sea, under a leader that can
/// transport, asks `current_upgrade(TRANSPORTBARGE)` for a spot round
/// slot 0, and the barge's ring stands on slot 0's own point. Without
/// transport, or with slot 0 on land, the member's own type sweeps.
#[test]
fn a_land_member_whose_slot_zero_is_at_sea_sweeps_as_the_leader_s_barge() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let mut tank = UnitType {
        hits: 200,
        ..UnitType::default()
    };
    tank.combat.big_radius = 96;
    tank.combat.block_radius = 96;
    let tank = f.sim.add_unit_type(tank);
    let u = unit(&mut f.sim, 1, tank, tile_pos(4, 4));
    let sea = tile_pos(40, 16);
    let land = tile_pos(20, 16);
    assert_eq!(
        f.sim.region_spot_type(u, sea),
        Some(tank),
        "no transport: the member's own type"
    );
    f.sim.transport[1].military = true;
    assert_eq!(
        f.sim.region_spot_type(u, sea),
        Some(b),
        "slot 0 at sea under transport: the barge"
    );
    assert_eq!(
        f.sim.region_spot_type(u, land),
        Some(tank),
        "slot 0 on land holds the member"
    );
    let snapped = Pos::new(
        sea.x.div_euclid(0x30) * 0x30 + 0x18,
        sea.y.div_euclid(0x30) * 0x30 + 0x18,
    );
    assert_eq!(
        f.sim
            .find_nearby_spot_type(b, sea, 0, -1, 0, crate::movement::Angle(0x5555_5555)),
        Some(snapped),
        "the barge's first candidate is slot 0's own point"
    );
    assert_ne!(
        f.sim
            .find_nearby_spot_type(tank, sea, 0, -1, 0, crate::movement::Angle(0x5555_5555)),
        Some(snapped),
        "the tank's own ring cannot stand at sea"
    );
}

/// §9.3 and `num_waterhalf`: the shore column is the land side of the
/// waterline, and the sea column is the water side.
#[test]
fn coast_here_names_the_shore_and_waterhalf_counts_the_water() {
    let f = fix();
    let w = &f.sim.world;
    // Cell (7, 3) is land, flagged `HALFLAND`, with the sea next door.
    assert_ne!(w.coast_here(f.land, f.sea, Cell::new(7, 3)), 0);
    // Read from the sea side the answer is the same pair, and non-zero.
    assert_ne!(w.coast_here(f.land, f.sea, Cell::new(8, 3)), 0);
    // A cell in neither, and a cell with no shore next to it.
    assert_eq!(w.coast_here(f.land, f.sea, Cell::new(3, 3)), 0);
    // `num_waterhalf` is 0 off a `HALFLAND` cell whatever its tiles
    // say, and counts the sixteen otherwise. The fixture's shore cells
    // are dry land flagged coastal, so the count is 0 — which is what
    // `think_civilian_transport` wants.
    assert_eq!(w.num_waterhalf(Cell::new(8, 3)), 0, "not HALFLAND");
    assert_eq!(w.num_waterhalf(Cell::new(7, 3)), 0, "no water tile in it");
}

/// `Group::action_swarm_around@0070fbe0`'s barge arm (`docs/AI.md`
/// §151): a citizen swarming an Oil Platform at sea searches the ring as
/// the leader's barge once the leader transports civilians, so its
/// approach is a point on the water; before that its own (land) type
/// finds nothing and **nothing is queued** — the member arm's move and
/// order both sit inside `find_nearby_spot == 0` (item 3622,
/// `docs/AI.md` §330). A water Dock is excepted.
#[test]
fn a_citizen_swarms_a_sea_site_as_the_barge_once_its_leader_transports() {
    use crate::orders::{Body, MoveKind};
    let mut f = fix();
    barge(&mut f.sim);
    let platform = f.sim.add_build_type(BuildType {
        ident: Ident::OilPlatform,
        x_size: 4,
        y_size: 4,
        flags: flags::parse("igbe"),
        hits: 500,
        ..BuildType::default()
    });
    let water_dock = f.sim.add_build_type(BuildType {
        ident: Ident::Dock,
        x_size: 4,
        y_size: 4,
        flags: flags::parse("ebn"),
        hits: 500,
        ..BuildType::default()
    });
    let site = place_dock(&mut f.sim, 1, platform, tile_pos(40, 16));
    let c = unit(&mut f.sim, 1, f.citizen, tile_pos(20, 16));
    let ocean =
        |sim: &Sim, p: Pos| sim.world.tile_mask(p.tile()) & tile::SURFACE == tile::SURFACE_OCEAN;

    // No transport level: the land ring finds no land, and neither
    // the approach nor the build goes in, nor the builder's mark.
    f.sim.swarm_around(c, site, Body::Build(site), true);
    assert!(
        f.sim.units[c].orders.is_empty(),
        "{:?}",
        f.sim.units[c].orders
    );
    assert!(!f.sim.units[c].was_builder, "no order, no mark");

    // A finished dock grants the civilian level; the ring is now the
    // barge's, and its first answer is water.
    let d = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 28));
    f.sim.activate(d, false, true);
    assert_eq!(f.sim.transport_level(1), TransportType::Civilian);
    f.sim.clear_orders(c);
    f.sim.swarm_around(c, site, Body::Build(site), true);
    assert_eq!(f.sim.units[c].orders.len(), 2);
    let Body::Move(m) = f.sim.units[c].orders[0].body else {
        panic!("an approach first: {:?}", f.sim.units[c].orders);
    };
    assert_eq!(m.kind, MoveKind::ExploreTo, "a computer's builder explores");
    assert!(ocean(&f.sim, m.dest), "the approach stands on the water");
    assert!(matches!(f.sim.units[c].orders[1].body, Body::Build(b) if b == site));

    // The Dock lineage is excepted (`is(DOCK)`): its ring is the
    // citizen's own, and nothing on it is land.
    let dock_site = place_dock(&mut f.sim, 1, water_dock, tile_pos(44, 8));
    f.sim.clear_orders(c);
    f.sim
        .swarm_around(c, dock_site, Body::Build(dock_site), true);
    assert!(
        f.sim.units[c].orders.is_empty(),
        "no water approach to a dock, and so no build"
    );
}

/// §6.1 and §6.2 end to end: a land unit with the bit that steps onto
/// an ocean tile is not moved — it queues the transport spell, and the
/// next frame's `do_cast` converts it.
#[test]
fn a_step_into_the_sea_becomes_a_boat_with_the_walker_inside() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let u = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    f.sim.init_guys(u, Some(f.citizen));
    f.sim.units[u].auto_transport = true;
    // A queued move, so the boat has something to inherit, and a path
    // whose top is the water it is about to step onto.
    f.sim.add_move_order(
        u,
        tile_pos(40, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    let top = crate::orders::PathData {
        to: tile_pos(33, 14),
        tolerance: 0,
        flags: crate::orders::path_flag::BOARDS_TRANSPORT,
    };
    f.sim.units[u].path.push(top);

    // The step itself: refused, and a cast is in front of the move.
    let before = f.sim.rng.seed;
    assert!(
        !f.sim.set_new_location(u, tile_pos(33, 14), false),
        "a land unit does not walk onto the sea"
    );
    assert_eq!(f.sim.units[u].pos, tile_pos(30, 14), "and it does not move");
    assert_eq!(f.sim.rng.seed, before, "the conversion spends no draw");
    assert!(matches!(
        f.sim.units[u].orders.front().map(|o| o.body),
        Some(crate::orders::Body::Cast(c)) if c.spell == crate::orders::spell::TRANSPORT
    ));
    assert_eq!(
        f.sim.units[u].orders.len(),
        2,
        "the move is still behind it"
    );

    // The cast: one `set_anim` draw a figure, then the boat's own.
    let units_before = f.sim.units.len();
    f.sim.work(u, 1);
    assert_eq!(f.sim.units.len(), units_before + 1, "a boat");
    let boat = units_before;
    assert_eq!(f.sim.units[boat].ty, Some(b));
    assert_eq!(f.sim.units[boat].owner, 1);
    assert!(
        f.sim.world.tile_mask(f.sim.units[boat].pos.tile()) & tile::SURFACE == tile::SURFACE_OCEAN,
        "the boat is born on the water"
    );
    assert!(f.sim.units[boat].auto_transport, "`unit_masks |= 0x800000`");
    // The order list moved and the cast at its head was thrown away.
    assert!(f.sim.units[u].orders.is_empty());
    assert_eq!(f.sim.units[boat].orders.len(), 1);
    assert!(f.sim.units[boat].orders[0].is_move());
    // The path moved whole, and the top's embark flag is gone: the
    // boat is on the sea side already.
    assert!(f.sim.units[u].path.is_empty());
    assert_eq!(
        f.sim.units[boat].path.last().map(|p| p.flags),
        Some(0),
        "the top is re-pushed with `flags & 4` cleared"
    );
    // And the walker is cargo: off the map, its clock stopped.
    assert_eq!(f.sim.units[u].inside_unit, Some(boat));
    assert!(!f.sim.units[u].on_map);
}

/// **A member waits a frame on its captain's transport**
/// (`do_cast@005ebfe0`, `005ecd9c`–`005ecdf0`; `docs/AI.md` §207): a
/// figure that is not a captain, holding a transport cast whose clock
/// is due, returns paid with the clock back where it was while its
/// captain's own head order is a cast. East Indies' `1/24` on 3835,
/// behind `1/23`, which took its transport on its own step.
///
/// Made to fail by dropping the check: the member builds the boat at
/// once.
#[test]
fn a_squad_member_waits_on_its_captain_s_transport_cast() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let captain = unit(&mut f.sim, 1, f.citizen, tile_pos(29, 14));
    let member = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    for &m in &[captain, member] {
        f.sim.init_guys(m, Some(f.citizen));
        f.sim.units[m].auto_transport = true;
    }
    f.sim.units[captain].o_down = Some(member);
    f.sim.units[member].o_up = Some(captain);
    f.sim.units[member].captain = false;
    f.sim.add_cast_order_at(
        member,
        crate::orders::spell::TRANSPORT,
        crate::orders::QueuePos::New,
    );
    f.sim.add_cast_order_at(
        captain,
        crate::orders::spell::TRANSPORT,
        crate::orders::QueuePos::New,
    );

    let units_before = f.sim.units.len();
    f.sim.work(member, 1);
    assert_eq!(
        f.sim.units.len(),
        units_before,
        "no boat while the captain casts"
    );
    assert_eq!(
        f.sim.units[member].spell_time, 0,
        "the clock is handed back"
    );
    assert!(
        matches!(
            f.sim.units[member].orders.front().map(|o| o.body),
            Some(crate::orders::Body::Cast(c)) if c.paid
        ),
        "the cast stays at the head, paid"
    );
    assert_eq!(f.sim.units[member].inside_unit, None);

    // The captain's order is no longer a cast: the member casts.
    f.sim.units[captain].orders.clear();
    f.sim.work(member, 1);
    assert_eq!(f.sim.units.len(), units_before + 1, "a boat");
    assert_eq!(f.sim.units[units_before].ty, Some(b));
    assert_eq!(f.sim.units[member].inside_unit, Some(units_before));
}

/// §6.2 line 3 and `docs/VISION.md` §11: the boat is born on the
/// caster's point, which `add_to_world` lights, and `set_new_location
/// (boat, spot, 1, 1)` then crosses a half-cell onto the water and
/// lights the **whole disc at the spot**. A fog cell three half-cells
/// seaward of the spot is out of the birth disc's reach and inside the
/// spot's: seen only when the second reveal is thrown.
#[test]
fn a_boat_born_on_the_shore_lights_its_disc_on_the_water() {
    let mut f = fix();
    assert!(f.sim.world.set_fog(vec![0; 24 * 16]));
    let b = barge(&mut f.sim);
    // `LOS 6`: a fog radius of 3.
    f.sim.tables.unit_types[b].los = 6;
    // A sea unit sees from its own half-cell (`docs/VISION.md` §3):
    // no forward projection to carry the birth disc onto the water.
    f.sim.tables.unit_types[b].kind.domain = Domain::Sea;
    let u = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    f.sim.init_guys(u, Some(f.citizen));
    f.sim.units[u].auto_transport = true;
    assert!(!f.sim.set_new_location(u, tile_pos(33, 14), false));
    let born = f.sim.units[u].pos;
    let boat = f.sim.units.len();
    f.sim.work(u, 1);
    assert_eq!(f.sim.units.len(), boat + 1, "a boat");
    let spot = f.sim.units[boat].pos;
    let half = |p: i32| p / 0x180;
    assert_ne!(
        (half(spot.x), half(spot.y)),
        (half(born.x), half(born.y)),
        "the spot is another half-cell"
    );
    let (fx, fy) = (half(spot.x) + 3, half(spot.y));
    assert!(
        (fx - half(born.x)).abs() > 3,
        "the probe is beyond the birth disc"
    );
    assert!(
        f.sim.world.seen2(fx, fy).is_some_and(|v| v & 2 != 0),
        "the spot's disc is lit for its owner"
    );
}

/// **The boat in the muster** (item 1228, §16): `Objects::init_unit`'s
/// `Unit::set_type` counts a boat whose type has population, and
/// `Unit::close` takes it out again. A caravan's Merchant Fleet, of
/// `POP` 1, moves `num_units` and `control` by one while it is at sea;
/// a citizen's Transport Barge, of `POP` 0, never does. run462's block 10178
/// holds one of each, and the original's `num_units` has the fleet
/// alone.
///
/// Made to fail by dropping either call: without the birth's the fleet
/// is never counted, without the close's it stays counted ashore.
#[test]
fn a_merchant_fleet_is_counted_at_sea_and_a_barge_never_is() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[b].price.pop = 0;
    let mut fleet_t = UnitType {
        hits: 50,
        moves: 25,
        ..UnitType::default()
    };
    fleet_t.combat.domain = Domain::Sea;
    fleet_t.combat.block_radius = 48;
    fleet_t.price.pop = 1;
    fleet_t.cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;
    let fleet = f.sim.add_unit_type(fleet_t);
    f.sim.tables.unit_types[fleet].tree = Some(ty::MERCHANTFLEET);
    f.sim.tables.unit_types[fleet].type_index = ty::MERCHANTFLEET as i32;
    let caravan = f.sim.add_unit_type(UnitType {
        hits: 40,
        ..UnitType::default()
    });
    f.sim.tables.unit_types[caravan].tree = Some(ty::CARAVAN);
    f.sim.tables.unit_types[caravan].price.pop = 1;
    f.sim.tables.unit_types[caravan].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;

    for (walker_t, boat_t, counted, sweep) in [
        (caravan, fleet, 1, false),
        (caravan, fleet, 1, true),
        (f.citizen, b, 0, false),
    ] {
        let u = unit(&mut f.sim, 1, walker_t, tile_pos(30, 14));
        f.sim.init_guys(u, Some(walker_t));
        f.sim.units[u].auto_transport = true;
        let control = f.sim.roster.muster[1].control;
        let caras = f.sim.ai[1].census.caras;
        assert!(!f.sim.set_new_location(u, tile_pos(33, 14), false));
        let boat = f.sim.units.len();
        f.sim.work(u, 1);
        assert_eq!(f.sim.units[boat].ty, Some(boat_t), "the boat");
        assert_eq!(
            f.sim.roster.muster[1].by_type[boat_t], counted,
            "`set_type`'s `+1`, for a type with population"
        );
        assert_eq!(f.sim.roster.muster[1].control, control + counted);
        assert_eq!(f.sim.ai[1].census.caras, caras + counted);
        // Item 1647: a census drops fleets from caras, but closing
        // one afterward still subtracts. Preserve that asymmetry.
        if sweep {
            f.sim.census(1);
            let land_caravans = f
                .sim
                .units
                .iter()
                .filter(|u| u.alive() && u.owner == 1 && u.ty == Some(caravan))
                .count() as i32;
            assert_eq!(f.sim.ai[1].census.caras, land_caravans);
        }
        let before_close = f.sim.ai[1].census.caras;
        // Ashore again: the boat puts its passenger out and closes.
        assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
        assert!(!f.sim.units[boat].alive());
        assert_eq!(
            f.sim.roster.muster[1].by_type[boat_t], 0,
            "`Unit::close`'s `−1` at `0060f3db`"
        );
        assert_eq!(f.sim.roster.muster[1].control, control);
        assert_eq!(f.sim.ai[1].census.caras, before_close - counted);
    }
}

/// **A caravan's route slot goes with it onto the Merchant Fleet and
/// back** (item 3370, `docs/AI.md` §286): `cast_transport@00670db0`
/// gives the boat the caravan's `+0x86` and names the boat in the
/// route's `o`; `eject_contents@0064cd20` gives both back and clears the
/// boat's, so the boat's close closes no route. East Indies' fleet
/// `1/79` prints `special 2` — its route's slot — and verifies that
/// route's road on 14942.
///
/// Made to fail by stubbing `SimTransport::set_caravan_slot` (the boat
/// owns no route), or by dropping `disembark`'s hand-back (the boat's
/// close kills the route).
#[test]
fn a_caravan_s_route_slot_goes_with_it_onto_the_fleet_and_back() {
    let mut f = fix();
    let mut fleet_t = UnitType {
        hits: 50,
        moves: 25,
        ..UnitType::default()
    };
    fleet_t.combat.domain = Domain::Sea;
    fleet_t.combat.block_radius = 48;
    fleet_t.cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;
    let fleet = f.sim.add_unit_type(fleet_t);
    f.sim.tables.unit_types[fleet].tree = Some(ty::MERCHANTFLEET);
    f.sim.tables.unit_types[fleet].type_index = ty::MERCHANTFLEET as i32;
    let caravan = f.sim.add_unit_type(UnitType {
        hits: 40,
        ..UnitType::default()
    });
    f.sim.tables.unit_types[caravan].tree = Some(ty::CARAVAN);
    f.sim.tables.unit_types[caravan].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_CARAVAN;

    // A route below it, so the walker's slot is not 0.
    let other = unit(&mut f.sim, 1, caravan, tile_pos(10, 10));
    let u = unit(&mut f.sim, 1, caravan, tile_pos(30, 14));
    f.sim.init_guys(u, Some(caravan));
    f.sim.units[u].auto_transport = true;
    assert_eq!(f.sim.units[other].caravan, Some(0));
    assert_eq!(f.sim.units[u].caravan, Some(1));
    assert!(!f.sim.set_new_location(u, tile_pos(33, 14), false));
    let boat = f.sim.units.len();
    f.sim.work(u, 1);
    assert_eq!(f.sim.units[boat].ty, Some(fleet), "the fleet");
    assert_eq!(
        f.sim.units[boat].caravan,
        Some(1),
        "the boat takes the slot"
    );
    assert_eq!(
        f.sim.caravans[1].slots[1].unit,
        Some(boat),
        "the route names the boat"
    );

    assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
    assert!(!f.sim.units[boat].alive());
    assert_eq!(f.sim.units[boat].caravan, None, "the boat's is cleared");
    assert_eq!(f.sim.units[u].caravan, Some(1), "the caravan takes it back");
    assert_eq!(
        f.sim.caravans[1].slots[1].unit,
        Some(u),
        "the route names it again"
    );
    assert!(
        f.sim.caravans[1].slots[1].alive,
        "the boat's close closed no route"
    );
}

/// **A passenger that lands as its group's leader toggles the group's
/// mirror** (item 2193, `docs/AI.md` §214): `come_out`'s turn to the
/// boat's heading is `Unit::set_angle@00605400`, whose reversing arm
/// flips `GroupData::facing` when `find_leader` names the unit. The
/// successor pair's BMP `1/28` landed on 4363 as army 1's leader, and
/// the original's flag was 1 from there (run841). Made to fail by
/// dropping the toggle from `land_tail`: the flag stays clear.
#[test]
fn a_passenger_that_lands_as_its_group_s_leader_toggles_the_group_s_mirror() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let slot = f.sim.init_army(1, None);
    f.sim.army_add_unit(1, slot, rider);
    f.sim.units[rider]
        .movement
        .set_facing(crate::movement::Angle::NORTH);
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    f.sim.units[boat].auto_transport = true;
    f.sim.board(rider, boat);
    f.sim.units[boat]
        .movement
        .set_facing(crate::movement::Angle::SOUTH);
    assert!(!f.sim.armies[1].list[slot].group.facing);
    f.sim.disembark(boat);
    assert!(f.sim.units[rider].on_map, "it is ashore");
    assert_eq!(
        f.sim.units[rider].movement.heading,
        crate::movement::Angle::SOUTH,
        "turned to the boat's heading"
    );
    assert!(
        f.sim.armies[1].list[slot].group.facing,
        "a reversing turn by the leader toggles the army's flag"
    );
}

/// **The boat's orders queue behind what the passenger's tail gave it**
/// (item 3550, `docs/AI.md` §308): run1189's Explorer `1/0` lands on
/// 2172 with the `ATTACKTOORDER` its `come_out` tail's `add_to_army`
/// pushed still current, the barge's `EXPLORETOORDER` behind it, and a
/// 17-node path — the attack's 15 under the barge's 2, the top
/// (34296, 32760) with its transport bit clear. Made to fail by
/// replacing the passenger's list and path with the boat's: the
/// explore order is current and the path is the boat's two.
#[test]
fn the_boat_s_orders_queue_behind_what_the_passenger_s_tail_gave_it() {
    use crate::orders::{MoveKind, PathData, QueuePos, path_flag};
    let mut f = fix();
    let b = barge(&mut f.sim);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    let attack = tile_pos(20, 20);
    let explore = tile_pos(28, 10);
    f.sim
        .add_move_order(rider, attack, MoveKind::AttackTo, QueuePos::New, false);
    f.sim
        .add_move_order(boat, explore, MoveKind::ExploreTo, QueuePos::New, false);
    let node = |p: Pos, flags| PathData {
        to: p,
        tolerance: 0,
        flags,
    };
    f.sim.units[rider].path = vec![node(attack, 1), node(tile_pos(25, 17), 0)];
    f.sim.units[boat].path = vec![
        node(explore, 1),
        node(tile_pos(29, 12), path_flag::BOARDS_TRANSPORT),
    ];
    f.sim.hand_over_orders(boat, rider);
    let kinds: Vec<_> = f.sim.units[rider]
        .orders
        .iter()
        .map(|o| match &o.body {
            crate::orders::Body::Move(m) => Some(m.kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds,
        [Some(MoveKind::AttackTo), Some(MoveKind::ExploreTo)],
        "the attack stays current, the boat's order behind it"
    );
    assert_eq!(
        f.sim.units[rider].path,
        [
            node(attack, 1),
            node(tile_pos(25, 17), 0),
            node(explore, 1),
            node(tile_pos(29, 12), 0),
        ],
        "the boat's path on top of the attack's, its top's transport bit clear"
    );
    assert!(f.sim.units[boat].orders.is_empty() && f.sim.units[boat].path.is_empty());
}

/// The other arm of the same test: a boat that steps off the water
/// puts its passenger out and dies.
#[test]
fn a_boat_that_steps_ashore_ejects_and_dies() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    f.sim.units[boat].auto_transport = true;
    f.sim.board(rider, boat);
    assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
    assert!(!f.sim.units[boat].alive(), "an empty boat on land dies");
    assert_eq!(f.sim.units[rider].inside_unit, None);
    assert!(f.sim.units[rider].on_map);
}

/// **A boat that steps ashore in its own work still takes its figures'
/// `Guy::move`** (item 1191, `docs/TRANSPORT.md` §6.4):
/// `Unit::process@00610bc0` runs the `+0x188` think and then
/// `Guy::process` over the guy array whatever the think did, and the
/// boat's `Object::die(0)` → `Unit::close` leaves that array alone. A
/// boat standing on its `des` on the walk with `stopped` set pays the
/// arrival stand on the frame it dies — East Indies' `1/56` on 8519.
///
/// Made to fail by returning from `process_unit` on the dead boat before
/// `process_movement`: the figure keeps the walk.
#[test]
fn a_boat_that_steps_ashore_in_its_own_work_still_pays_its_arrival() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    // A step short of the shore, on the water side of the line.
    let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
    let boat = unit(&mut f.sim, 1, b, at);
    f.sim.units[boat].auto_transport = true;
    f.sim.init_guys(boat, Some(b));
    f.sim.board(rider, boat);
    f.sim.add_move_order(
        boat,
        tile_pos(30, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    // The state `1/56` stood in on block 8519: the body on its unit,
    // facing its waypoint, at its base speed on average, the figure on
    // the walk with `stopped` set.
    let bearing = crate::movement::Angle(-1_130_299_392);
    let u = &mut f.sim.units[boat];
    u.movement.speed = 25;
    u.movement.turning.type_turn_speed = i32::MAX;
    u.movement.facing = bearing;
    u.movement.heading = bearing;
    u.movement.body.pos = at;
    u.movement.body.avg_speed = 25;
    u.guys[0].anim = crate::anim::WALK;
    u.guys[0].stopped = true;
    f.sim.process_unit(boat, 1, &mut Vec::new());
    assert!(
        !f.sim.units[boat].alive(),
        "the boat stepped ashore and died"
    );
    assert!(f.sim.units[rider].on_map, "its passenger is out");
    assert_eq!(
        f.sim.units[boat].guys[0].anim,
        crate::anim::DEFAULT,
        "and its figure took the arrival stand on the way out"
    );
}

/// A barge a step off the shore with `rider` aboard and a move for the
/// land, standing — its body on its unit, facing its waypoint, its
/// figure's running average `avg`.
fn landing(f: &mut Fix, rider: usize, avg: i32) -> (usize, Pos) {
    let b = barge(&mut f.sim);
    let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
    let boat = unit(&mut f.sim, 1, b, at);
    f.sim.units[boat].auto_transport = true;
    f.sim.init_guys(boat, Some(b));
    f.sim.board(rider, boat);
    f.sim.add_move_order(
        boat,
        tile_pos(30, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    let bearing = crate::movement::Angle(-1_130_299_392);
    let u = &mut f.sim.units[boat];
    u.movement.speed = 25;
    u.movement.turning.type_turn_speed = i32::MAX;
    u.movement.facing = bearing;
    u.movement.heading = bearing;
    u.movement.body.pos = at;
    u.movement.body.avg_speed = avg;
    (boat, at)
}

/// **A boat that dies stepping ashore still moves its disc with its
/// figure** (item 2088, `docs/AI.md` §208). run825's packet on East
/// Indies' tick 4204: `Object::close` → `remove_from_world@00647970`
/// clears the Freighter `1/191`'s disc round its figure, and then the
/// boat's own `Guy::move+0x4a5` → `Guy::set_new_location` →
/// `CollCheck::move_unit` steps the figure a unit cell on, whose set
/// pass paints the new disc's leading row on no unit.
///
/// Made to fail by dropping the `coll_move` after the dead boat's
/// `process_movement`: the leading row is clear.
#[test]
fn a_boat_that_dies_stepping_ashore_paints_its_figure_s_step() {
    let mut f = fix();
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let (boat, at) = landing(&mut f, rider, 25);
    // The figure a unit cell behind its unit, and painted there —
    // the disc `unit` painted at the unit's own point cleared first,
    // or its leading row stands in for the step's. South of it, so the step's leading row lies in the boat's own
    // sea cell: a column west of it would be the shore's, which the
    // region gate refuses.
    let body = Pos::new(
        at.x,
        (crate::collide::ucell(at).y + 1) * crate::collide::UNITS_PER_UCELL + 1,
    );
    f.sim.coll_remove(boat);
    f.sim.units[boat].movement.body.pos = body;
    f.sim.units[boat].pos = body;
    f.sim.coll_add(boat);
    f.sim.units[boat].pos = at;
    let size = f.sim.coll_size(boat);
    assert!(size > 0, "the barge blocks");
    let from = crate::collide::ucell(body);
    // Off every repaint phase.
    let o = i64::from(f.sim.units[boat].index);
    f.sim.frame = 64 * 100 - o + 7;
    f.sim.process_unit(boat, f.sim.frame, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    let to = crate::collide::ucell(f.sim.units[boat].movement.body.pos);
    assert!(
        to.y < from.y,
        "its figure stepped north a cell: {from:?} -> {to:?}"
    );
    assert!(
        f.sim.movement.coll.get(to.x, to.y - size),
        "the step's set pass painted the leading row"
    );
    assert!(
        !f.sim.movement.coll.get(from.x, from.y + size),
        "and the close cleared the trailing one"
    );
}

/// **A boat that dies stepping ashore repaints its disc on its
/// sixty-fourth frame** (item 1223, `docs/GOLDEN.md` §52):
/// `Guy::process@005e0230`'s tail runs on the dead boat's figure, and
/// with `avg_speed` 0 on `(frame + o) % 64 == 0` it re-marks the cells
/// `Object::close` has just cleared. Standing in the open sea, the
/// barge's cell is its own region's, so the mark lands.
///
/// Made to fail by leaving `coll_repaint`'s guard at alive and on the
/// map: the cell is clear after the death.
#[test]
fn a_boat_that_dies_standing_on_its_phase_frame_leaves_its_disc_marked() {
    let mut f = fix();
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let (boat, at) = landing(&mut f, rider, 0);
    f.sim.coll_add(boat);
    let o = i64::from(f.sim.units[boat].index);
    f.sim.frame = 64 * 100 - o;
    f.sim.process_unit(boat, f.sim.frame, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    let c = crate::collide::ucell(at);
    assert!(
        f.sim.movement.coll.get(c.x, c.y),
        "the dead figure's repaint marked its own cell"
    );
}

/// And off its phase frame the death leaves the cell clear: the mark is
/// the repaint's, not a close that forgot to clear it.
#[test]
fn a_boat_that_dies_off_its_phase_frame_leaves_its_cell_clear() {
    let mut f = fix();
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let (boat, at) = landing(&mut f, rider, 0);
    f.sim.coll_add(boat);
    let o = i64::from(f.sim.units[boat].index);
    f.sim.frame = 64 * 100 - o + 1;
    f.sim.process_unit(boat, f.sim.frame, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    let c = crate::collide::ucell(at);
    assert!(
        !f.sim.movement.coll.get(c.x, c.y),
        "the close cleared the cell"
    );
}

/// **A squad's passenger takes the boat's orders through a group**
/// (item 1223, `docs/TRANSPORT.md` §6.4): `eject_contents`'
/// `uber_size > 1` arm resets the boat's move orders to their own point,
/// pushes a group of the passenger (`push_group(who, g, 1)`) and replays
/// the boat's action orders onto it with `finish_insert` — a group move
/// to where the boat was sailing, where the one-man arm hands the list
/// over whole.
///
/// Made to fail by dropping [`Sim::disembark_squad`]'s call: the
/// passenger comes ashore with no order and in no group.
#[test]
fn a_squad_s_passenger_comes_ashore_with_the_boat_s_move_as_a_group() {
    let mut f = fix();
    let mut t = f.sim.tables.unit_types[f.citizen].clone();
    t.combat.uber_size = 3;
    let squad = f.sim.add_unit_type(t);
    let rider = unit(&mut f.sim, 1, squad, tile_pos(30, 14));
    let (boat, _) = landing(&mut f, rider, 25);
    // A player's move carries the action bit (run466's barges print
    // `flags 5`), and `set_up_insert` copies only what does.
    f.sim.units[boat].orders[0].flags |= crate::orders::flag::ACTION;
    let crate::orders::Body::Move(sailing) = f.sim.units[boat].orders[0].body else {
        panic!("the boat's move");
    };
    f.sim.process_unit(boat, 1, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    assert!(f.sim.units[rider].on_map, "its passenger is out");
    let front = f.sim.units[rider].orders.front().expect("an order");
    let crate::orders::Body::Move(m) = front.body else {
        panic!("a move, not {:?}", front.body);
    };
    assert_eq!(
        m.group.map_or(m.dest, |g| g.orig),
        sailing.dest,
        "to the boat's point"
    );
    assert!(
        f.sim
            .orders
            .pushed
            .iter()
            .any(|p| p.who == 1 && p.list == vec![rider]),
        "in a pushed group of its own"
    );
}

/// **A squad comes ashore whole on its captain's pass** (item 1291,
/// `docs/TRANSPORT.md` §17, `docs/GOLDEN.md` §55): a captain's
/// `come_out` calls itself down `o_down` with each member's host its
/// captain, and `eject_contents` re-reads the boat's chain after it, so
/// the squad arm runs once — one group move on each figure. And the
/// members are turned to the captain's own angle, which its tail sets
/// to the boat's only after they are out. run496 block 1902: `0/7`,
/// `0/8` and `0/9` hold one `GroupMoveOrder` each, and `0/8`'s and
/// `0/9`'s angle is 1084948480, `0/7`'s on 1901.
///
/// Made to fail by running the eject's arm on every rider: the captain
/// holds three moves, the first member two, and both members face the
/// boat's way.
#[test]
fn a_landed_squad_takes_one_group_move_each_and_its_captain_s_angle() {
    let mut f = fix();
    let mut t = f.sim.tables.unit_types[f.citizen].clone();
    t.combat.uber_size = 3;
    let hoplite = f.sim.add_unit_type(t);
    let squad: Vec<usize> = (0..3)
        .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28, 13 + k)))
        .collect();
    for w in squad.windows(2) {
        f.sim.units[w[0]].o_down = Some(w[1]);
        f.sim.units[w[1]].o_up = Some(w[0]);
        f.sim.units[w[1]].captain = false;
    }
    let boarded = crate::movement::Angle(1_084_948_480);
    f.sim.units[squad[0]].movement.set_facing(boarded);
    let (boat, _) = landing(&mut f, squad[0], 25);
    f.sim.units[boat].orders[0].flags |= crate::orders::flag::ACTION;
    let bearing = f.sim.units[boat].movement.heading;
    assert_ne!(bearing, boarded);
    f.sim.process_unit(boat, 1, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    for &m in &squad {
        let u = &f.sim.units[m];
        assert!(u.on_map && u.inside_unit.is_none(), "figure {m} is out");
        assert_eq!(u.orders.len(), 1, "figure {m} holds one group move");
    }
    assert_eq!(f.sim.units[squad[0]].movement.heading, bearing);
    for &m in &squad[1..] {
        assert_eq!(
            f.sim.units[m].movement.heading, boarded,
            "member {m} faces its captain's boarding angle"
        );
    }
}

/// **A captain's boarding takes its whole squad** (item 1235,
/// `docs/TRANSPORT.md` §17): `Unit::go_inside@0061a2e0` climbs `o_up`
/// to the captain, inserts it, and calls itself down `o_down` — and
/// each figure of a type whose `uber_size` is over 1 has its orders
/// closed aboard. run466 block 1357: the Hoplites `0/8` and `0/9` are
/// inside their captain's barge `0/10`, their group move gone.
///
/// Made to fail by boarding the unit alone: the members stay ashore
/// holding their moves.
#[test]
fn a_captain_s_boarding_takes_its_squad_aboard_and_closes_its_orders() {
    let mut f = fix();
    let mut t = f.sim.tables.unit_types[f.citizen].clone();
    t.combat.uber_size = 3;
    let hoplite = f.sim.add_unit_type(t);
    let b = barge(&mut f.sim);
    let squad: Vec<usize> = (0..3)
        .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28 + k, 14)))
        .collect();
    for w in squad.windows(2) {
        f.sim.units[w[0]].o_down = Some(w[1]);
        f.sim.units[w[1]].o_up = Some(w[0]);
        f.sim.units[w[1]].captain = false;
    }
    for &m in &squad {
        f.sim.add_move_order(
            m,
            tile_pos(20, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
    }
    let boat = unit(&mut f.sim, 1, b, tile_pos(32, 14));
    // From the last figure: `param_3 == 0` climbs to the head first.
    f.sim.board(squad[2], boat);
    for &m in &squad {
        let u = &f.sim.units[m];
        assert_eq!(u.inside_unit, Some(boat), "figure {m} is aboard");
        assert!(!u.on_map, "figure {m} is off the map");
        assert!(u.orders.is_empty(), "figure {m}'s move is closed aboard");
    }
}

/// **The chain stops at a dead figure** (`61a497`'s `flags & 1`): what
/// hangs below it is not walked, and stays ashore.
#[test]
fn a_squad_s_boarding_stops_at_a_dead_figure_down_the_chain() {
    let mut f = fix();
    let mut t = f.sim.tables.unit_types[f.citizen].clone();
    t.combat.uber_size = 3;
    let hoplite = f.sim.add_unit_type(t);
    let b = barge(&mut f.sim);
    let squad: Vec<usize> = (0..3)
        .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28 + k, 14)))
        .collect();
    for w in squad.windows(2) {
        f.sim.units[w[0]].o_down = Some(w[1]);
        f.sim.units[w[1]].o_up = Some(w[0]);
        f.sim.units[w[1]].captain = false;
    }
    f.sim.units[squad[1]].health = 0;
    let boat = unit(&mut f.sim, 1, b, tile_pos(32, 14));
    f.sim.board(squad[0], boat);
    assert_eq!(f.sim.units[squad[0]].inside_unit, Some(boat));
    assert_eq!(f.sim.units[squad[1]].inside_unit, None);
    assert_eq!(f.sim.units[squad[2]].inside_unit, None);
}

/// **The boat's number is held thirty frames** (`docs/COMBAT.md`
/// §59.3): `Object::die(boat, 0, −1, 0)` makes no death object, but
/// `close` ends in `Object::close`'s `hold_frames = 0x1e`, and
/// `Objects::find_free` will not hand the number out until
/// `process_all` has taken it to zero. East Indies' barge `1/62` died
/// on 8195, and the original numbered its owner's next birth, on 8210,
/// past it; this crate gave that birth the barge's number.
///
/// Made to fail by leaving [`Sim::disembark`]'s boat with no hold.
#[test]
fn a_boat_that_puts_its_passenger_ashore_holds_its_number_thirty_frames() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
    let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
    let boat = unit(&mut f.sim, 1, b, at);
    f.sim.units[boat].auto_transport = true;
    f.sim.init_guys(boat, Some(b));
    f.sim.board(rider, boat);
    f.sim.add_move_order(
        boat,
        tile_pos(30, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    // 1191's state: the body on its unit, facing its waypoint.
    let bearing = crate::movement::Angle(-1_130_299_392);
    let u = &mut f.sim.units[boat];
    u.movement.speed = 25;
    u.movement.turning.type_turn_speed = i32::MAX;
    u.movement.facing = bearing;
    u.movement.heading = bearing;
    u.movement.body.pos = at;
    u.movement.body.avg_speed = 25;
    f.sim.process_unit(boat, 1, &mut Vec::new());
    assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
    assert_eq!(f.sim.units[boat].hold_frames, Sim::CLOSE_HOLD);
    let number = f.sim.units[boat].index;
    for _ in 1..Sim::CLOSE_HOLD {
        f.sim.tick();
        assert_ne!(
            f.sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
            Some(number),
            "the dead boat's number is held"
        );
    }
    f.sim.tick();
    assert_eq!(f.sim.units[boat].hold_frames, 0);
    assert_eq!(
        f.sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
        Some(number),
        "and handed out once the hold is spent"
    );
}

/// §6.4 whole: the passenger comes out on `come_out`'s **host** ring —
/// the boat's `block_radius` out to `+ UNIT_DISEMBARK_DISTANCE`, swept
/// from the boat's own angle — keeps its speed, throws the scout arm's
/// army coin, and takes the boat's order list and inverted path stack
/// back with the top's embark flag cleared.
///
/// It fails on the code this replaced at four separate lines, which is
/// why it is one test: the passenger landed on the boat's own point,
/// with no orders, at zero speed, and spending no draw.
#[test]
fn a_passenger_put_ashore_takes_the_boat_s_ring_and_the_boat_s_orders() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[f.scout].combat.uber_size = 1;
    f.sim.tables.unit_types[f.scout].combat.block_radius = 48;
    // `is_special` is `is(SCOUT)` folded into `unit_flags2` by the
    // loader, and it is what puts this unit on the coin's `% 2` arm.
    f.sim.tables.unit_types[f.scout].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_SCOUT;
    let rider = unit(&mut f.sim, 1, f.scout, tile_pos(30, 14));
    f.sim.units[rider].movement.speed = 34;
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    f.sim.units[boat].auto_transport = true;
    // The boat is heading due west, at the land: the sweep's first
    // bearing is that, and the ring starts at the **boat's** 48.
    let west = crate::movement::Angle::WEST;
    f.sim.units[boat].movement.set_facing(west);
    f.sim.add_move_order(
        boat,
        tile_pos(20, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    f.sim.units[boat].path.push(crate::orders::PathData {
        to: tile_pos(20, 14),
        tolerance: 0,
        flags: 1,
    });
    f.sim.units[boat].path.push(crate::orders::PathData {
        to: tile_pos(30, 14),
        tolerance: 0,
        flags: crate::orders::path_flag::BOARDS_TRANSPORT,
    });
    f.sim.units[rider]
        .movement
        .set_facing(crate::movement::Angle::INITIAL);
    let boarded = f.sim.units[rider].pos;
    f.sim.board(rider, boat);

    let before = f.sim.rng.seed;
    assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));

    // The coin: an AI-driven scout takes the `% 2` arm and spends one
    // draw (`docs/ARMY.md` §4.1).
    assert_ne!(f.sim.rng.seed, before, "the scout arm's army coin");

    // The ring: on the boat's own heading, at least the boat's
    // `block_radius` away and no further than that plus the disembark
    // distance — never the boat's own point.
    let (at, spot) = (f.sim.units[boat].pos, f.sim.units[rider].pos);
    assert_ne!(spot, at, "not the boat's own point");
    assert!(
        (spot.y - at.y).abs() <= 48,
        "due west of it, on the boat's heading, within the quarter-tile snap"
    );
    let d = at.x - spot.x;
    assert!(
        (48..=48 + f.sim.tables.tuning.unit_disembark_distance).contains(&d),
        "the ring is the boat's `[48, 624]`, and this is {d}"
    );

    // The speed survives, or the passenger stands there for ever.
    assert_eq!(f.sim.units[rider].movement.speed, 34);
    // And the whole order list and path came back, top flag cleared.
    assert!(f.sim.units[boat].orders.is_empty());
    assert_eq!(f.sim.units[rider].orders.len(), 1);
    assert!(f.sim.units[rider].orders[0].is_move());
    assert_eq!(
        f.sim.units[rider]
            .path
            .iter()
            .map(|p| (p.to, p.flags))
            .collect::<Vec<_>>(),
        vec![(tile_pos(20, 14), 1), (tile_pos(30, 14), 0)],
        "the boat's stack, in order, with the top's `flags & 4` gone"
    );
    assert!(!f.sim.units[boat].alive());
    // `update_action` is step 1's (item 803) and, for a computer
    // player's unit, `come_out`'s tail's again from the spot (item
    // 1164): `orders_x/y` are the landing point, not the boarding one,
    // and `dest_angle` the heading it boarded on while `come_out` snaps
    // its facing west.
    assert_ne!(spot, boarded);
    assert_eq!(f.sim.units[rider].orders_pos, spot);
    assert_eq!(f.sim.units[rider].movement.heading, west);
    assert_eq!(
        f.sim.units[rider].movement.des_angle,
        crate::movement::Angle::INITIAL
    );
}

/// A passenger put ashore on `come_out`'s ring, for a human or a
/// computer player: the rider's `orders_x/y` and its guys' speeds.
fn ashore(human: bool, avg: i32) -> (Fix, usize, Pos) {
    let mut f = fix();
    f.sim.nation[1].human = human;
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[f.scout].combat.uber_size = 1;
    f.sim.tables.unit_types[f.scout].combat.block_radius = 48;
    let rider = unit(&mut f.sim, 1, f.scout, tile_pos(30, 14));
    f.sim.units[rider].movement.speed = 34;
    f.sim.units[rider].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
    f.sim.roster.art.tracks.insert(2, (-48, -192));
    f.sim.seat_guys(rider);
    f.sim.units[rider].movement.body.avg_speed = avg;
    f.sim.units[rider].guys[1]
        .follow
        .as_mut()
        .unwrap()
        .body
        .avg_speed = avg;
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    f.sim.units[boat].auto_transport = true;
    f.sim.units[boat]
        .movement
        .set_facing(crate::movement::Angle::WEST);
    let boarded = f.sim.units[rider].pos;
    f.sim.board(rider, boat);
    assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
    assert_eq!(f.sim.units[rider].inside_unit, None, "ashore");
    (f, rider, boarded)
}

/// **`come_out`'s tail, for a computer player's unit** (item 1164,
/// §6.4): past `set_new_location`, `6187c8` skips a human's unit and
/// `618813`..`618836` run `update_action` on everyone else's from the
/// spot. run420's AI merchant `1/33` prints its landing point
/// (38952, 24840) as `orders_x/y` on 6735; a human's keeps the point it
/// boarded at, step 1's (run249's `0/6`, item 803).
#[test]
fn a_computer_player_s_passenger_is_ordered_where_it_lands_and_a_human_s_where_it_boarded() {
    let (ai, r, boarded) = ashore(false, 0);
    let spot = ai.sim.units[r].pos;
    assert_ne!(spot, boarded);
    assert_eq!(
        ai.sim.units[r].orders_pos, spot,
        "the AI's: the landing point"
    );
    let (me, r, boarded) = ashore(true, 0);
    assert_eq!(
        me.sim.units[r].orders_pos, boarded,
        "a human's: the boarding point"
    );
}

/// **A passenger comes ashore at the average it boarded with** (item
/// 1164, §6.4): `Guy::last_speed`/`avg_speed` (`+0x80`/`+0x84`) are
/// written by `Guy::move`, `Guy::clear` and `Guy::init_real` alone,
/// and neither `set_new_location` is one. So the first turn ashore is
/// [`crate::movement::turn_speed`]'s divided one: run420's `1/33` lands
/// at 12 and turns a quarter, a third, a half … of its rate, facing its
/// path on 6745 where a zeroed average faced it on 6743.
#[test]
fn a_passenger_comes_ashore_at_the_average_it_boarded_with() {
    let (f, r, _) = ashore(false, 12);
    let u = &f.sim.units[r];
    assert_eq!(u.movement.body.avg_speed, 12, "guy 0's body");
    assert_eq!(
        u.guys[1].follow.map(|g| g.body.avg_speed),
        Some(12),
        "and the tracked crew guy's, seated on its offset again"
    );
    assert_eq!(
        u.guys[1].follow.map(|g| g.body.pos),
        u.guys[1].follow.map(|g| g.des),
        "seated on its offset"
    );
}

/// **A passenger put ashore clears where it boarded** (item 1598,
/// `docs/COLLISION.md` §25): `Unit::come_out`'s `remove_from_inside`
/// paints the disc at guy 0's retained point and `set_new_location`'s
/// `move_unit` moves it to the spot, so every cell of the boarding disc
/// farther than `coll_size` from the spot ends clear — a bit another
/// unit painted there while the passenger was aboard included. East
/// Indies' `1/12`, ashore on tick 1113, clears cell (890, 771) of
/// Freighter `1/27`'s birth disc in run713's watch.
///
/// Made to fail by painting the spot fresh (`coll_add`), the code this
/// replaced: the other unit's bit at the boarding point survives.
#[test]
fn a_passenger_put_ashore_clears_the_disc_it_boarded_from() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[f.scout].combat.uber_size = 1;
    f.sim.tables.unit_types[f.scout].combat.block_radius = 48;
    let rider = unit(&mut f.sim, 1, f.scout, tile_pos(30, 14));
    f.sim.units[rider].movement.speed = 34;
    let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
    f.sim.units[boat].auto_transport = true;
    f.sim.units[boat]
        .movement
        .set_facing(crate::movement::Angle::WEST);
    let boarded = crate::collide::ucell(f.sim.units[rider].pos);
    f.sim.board(rider, boat);
    // Another unit's bit, inside the boarding disc and on its far side
    // from the ring the boat lands it on.
    let ghost = Pos::new(boarded.x - 1, boarded.y);
    f.sim.movement.coll.set(ghost.x, ghost.y, true);
    assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
    assert_eq!(f.sim.units[rider].inside_unit, None, "ashore");
    let spot = crate::collide::ucell(f.sim.units[rider].pos);
    assert!(
        (spot.x - ghost.x).abs() > 1,
        "the ghost is outside the new disc"
    );
    assert!(
        !f.sim.movement.coll.get(ghost.x, ghost.y),
        "the boarding disc is cleared, the other unit's bit with it"
    );
    assert!(
        f.sim.movement.coll.get(spot.x, spot.y),
        "the spot's disc is set"
    );
    assert_eq!(f.sim.units[rider].coll_at, Some(f.sim.units[rider].pos));
}

/// §6.4's way in, and `docs/ORDERS.md` §4.4's region check (item 1143):
/// a barge that **takes** a final waypoint on land — another tile
/// region than its own — re-pushes it with `flags | 4` and tolerance 0,
/// so `find_path`'s `invalid_loc` takes the land tile and the barge
/// keeps its goal. Without the check the pull-back walks the goal back
/// to the water and the barge stops a tile short of the shore with its
/// passenger aboard, East Indies' `1/42` on 6321.
#[test]
fn a_barge_taking_a_goal_ashore_flags_it_and_keeps_it() {
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[b].kind.domain = Domain::Sea;
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(40, 14));
    let boat = unit(&mut f.sim, 1, b, tile_pos(40, 14));
    f.sim.units[boat].auto_transport = true;
    f.sim.init_guys(boat, Some(b));
    f.sim.board(rider, boat);
    let goal = tile_pos(28, 14);
    f.sim.add_move_order(
        boat,
        goal,
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    f.sim.units[boat].orders[0].flags |= crate::orders::flag::PATHED;
    f.sim.units[boat].path.push(crate::orders::PathData {
        to: goal,
        tolerance: 96,
        flags: crate::orders::path_flag::FINAL,
    });
    assert_ne!(
        f.sim.world.tregion_alt(goal.tile()),
        f.sim.world.tregion_alt(f.sim.units[boat].pos.tile()),
        "the goal is in another tile region"
    );
    f.sim.work(boat, 1);
    assert_eq!(
        f.sim.units[boat]
            .path
            .last()
            .map(|p| (p.to, p.tolerance, p.flags)),
        Some((goal, 0, 1 | crate::orders::path_flag::BOARDS_TRANSPORT)),
        "the final leg is flagged, and its point is the land tile still"
    );
    assert_eq!(
        f.sim.current_move(boat).map(|m| m.waypoint),
        Some(goal),
        "and the barge walks at it"
    );
}

/// **The region check pushes the stack's own top**, after `do_move`'s
/// road check has stripped it (item 3370, `docs/AI.md` §286):
/// `do_move@005f7b30:496` pops the entry the strip just wrote and
/// pushes it back `| 4`, so a road-flagged waypoint over a built tile in
/// another region comes back flagged 4 alone. East Indies' fleet `1/79`
/// on 14942: run993d's block 14943 prints `path[19].flags` 4, where
/// this crate pushed the copy it read before the strip, 36.
///
/// Made to fail by pushing the pre-strip copy: the waypoint keeps
/// `ROAD`.
#[test]
fn a_trade_walker_s_region_push_carries_the_stripped_road_flag() {
    use crate::orders::{Body, Order, PathData, TradeOrder, path_flag};
    use crate::world::tile;
    let mut f = fix();
    let b = barge(&mut f.sim);
    f.sim.tables.unit_types[b].kind.domain = Domain::Sea;
    let boat = unit(&mut f.sim, 1, b, tile_pos(40, 14));
    f.sim.init_guys(boat, Some(b));
    let goal = tile_pos(28, 14);
    let m = f.sim.world.tile_mask(goal.tile());
    f.sim.world.set_tile_mask(goal.tile(), m | tile::PLACED);
    f.sim.add_move_order(
        boat,
        goal,
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    f.sim.units[boat].orders[0].flags |= crate::orders::flag::PATHED;
    f.sim.units[boat].orders.push_back(Order {
        flags: crate::orders::flag::ACTION,
        body: Body::Trade(TradeOrder {
            home: 0,
            dest: None,
            started: true,
            loaded: false,
        }),
    });
    f.sim.units[boat].path = vec![
        PathData {
            to: tile_pos(20, 14),
            tolerance: 96,
            flags: path_flag::FINAL | path_flag::ROAD_LAID,
        },
        PathData {
            to: goal,
            tolerance: 96,
            flags: path_flag::ROAD_LAID,
        },
    ];
    assert_ne!(
        f.sim.world.tregion_alt(goal.tile()),
        f.sim.world.tregion_alt(f.sim.units[boat].pos.tile()),
        "the waypoint is in another tile region"
    );
    f.sim.work(boat, 1);
    assert_eq!(
        f.sim.units[boat]
            .path
            .iter()
            .map(|p| p.flags)
            .collect::<Vec<_>>(),
        vec![path_flag::FINAL, path_flag::BOARDS_TRANSPORT],
        "the strip reaches every entry, the pushed top's too"
    );
}

/// §7: the island search picks a coastal cell of another region that
/// a sea region coasts with mine, and issues the move.
#[test]
fn the_island_search_sends_a_scout_to_another_region_s_shore() {
    let mut f = fix();
    barge(&mut f.sim);
    // A second island on the far side of the sea, flagged as a
    // resource region so `go_here` answers bit 1.
    let far = f
        .sim
        .world
        .fill_region(Terrain::Land, Cell::new(11, 0), Cell::new(11, 7));
    for y in 0..8 {
        let c = Cell::new(11, y);
        let mut d = f.sim.world.cell_data(c);
        d.land = 0;
        d.flags |= 0x100;
        d.region2 = Some(f.sea);
        f.sim.world.set_cell_data(c, d);
        for tx in 44..48 {
            for ty in y * 4..y * 4 + 4 {
                f.sim
                    .world
                    .set_tile_field(Pos::new(tx, ty), tile::SURFACE, 0);
            }
        }
    }
    f.sim.world.rebuild_coasts();
    f.sim.world.set_region_flags(far, 8);
    let u = unit(&mut f.sim, 1, f.scout, tile_pos(10, 14));
    f.sim.units[u].auto_transport = true;
    f.sim.units[u].type_index = ty::SCOUT as i32;
    f.sim.transport.resize_with(2, LeaderTransport::default);
    f.sim.transport[1].set_all();
    assert!(
        f.sim.think_civilian_transport(u, false),
        "a resource region across the water, and nothing scouted"
    );
    let dest = f.sim.units[u].orders.front().and_then(|o| o.move_dest());
    let cell = dest.expect("a move order").cell();
    assert_eq!(cell.x, 11, "the far island's own column");
    // And once its own region is marked, the same region is still
    // fair game — the mark is on *mine*, not on the candidate.
    f.sim.world.mark_region_scouted(far, 1);
    f.sim.units[u].orders.clear();
    assert!(
        !f.sim.think_civilian_transport(u, false),
        "a region this leader has scouted is not a candidate"
    );
}

/// `Unit::think_carry` (`docs/AI.md` §195): a boat with a worker aboard
/// walks to the shore of a free land region; with nothing aboard it
/// does nothing here.
#[test]
fn a_boat_with_a_worker_aboard_walks_to_a_free_shore() {
    let mut f = fix();
    f.sim.world.set_region_flags(f.land, 8);
    let boat = unit(&mut f.sim, 1, f.ship, tile_pos(36, 12));
    f.sim.units[boat].on_map = true;
    assert!(
        !f.sim.think_carry(boat),
        "nothing aboard: the seam answers no"
    );
    assert!(f.sim.units[boat].orders.is_empty());
    let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(36, 12));
    f.sim.units[rider].type_index = 0x32;
    f.sim.units[rider].inside_unit = Some(boat);
    f.sim.units[rider].on_map = false;
    assert!(f.sim.think_carry(boat), "a worker aboard and a free shore");
    let dest = f.sim.units[boat]
        .orders
        .front()
        .and_then(|o| o.move_dest())
        .expect("a move order");
    assert!(dest.cell().x >= 7, "the land's own shore column: {dest:?}");
}

/// **The boat's region is `get_tregion`'s** (`think_carry_civilian@005f37c0`,
/// `local_64`; `docs/AI.md` §274): a boat on the ocean tile of a coastal
/// land cell stands in the cell's `region2`, the sea, so the land it
/// lies against coasts it. Read as the cell's plain region it is in the
/// land, which no land region coasts, and the search finds nothing —
/// run970e's barge `1/77` on 14595, left idle where the original sails
/// it to (21912, 32664).
#[test]
fn a_boat_on_a_coastal_cell_s_water_searches_from_the_sea() {
    let mut f = fix();
    // The only region the leader wants is an island across the sea:
    // the boat's own land answers `is_coast` of itself, so the island
    // is what tells the two regions apart.
    let far = f
        .sim
        .world
        .fill_region(Terrain::Land, Cell::new(11, 0), Cell::new(11, 7));
    for y in 0..8 {
        let c = Cell::new(11, y);
        let mut d = f.sim.world.cell_data(c);
        d.land = 0;
        d.flags |= 0x100;
        d.region2 = Some(f.sea);
        f.sim.world.set_cell_data(c, d);
        for tx in 44..48 {
            for ty in y * 4..y * 4 + 4 {
                f.sim
                    .world
                    .set_tile_field(Pos::new(tx, ty), tile::SURFACE, 0);
            }
        }
    }
    f.sim.world.rebuild_coasts();
    f.sim.world.set_region_flags(far, 8);
    let at = tile_pos(31, 13);
    f.sim
        .world
        .set_tile_field(Pos::new(31, 13), tile::SURFACE, tile::SURFACE_OCEAN);
    assert_eq!(f.sim.world.tregion(Pos::new(31, 13)), Some(f.land));
    assert_eq!(f.sim.world.tregion_alt(Pos::new(31, 13)), Some(f.sea));
    let boat = unit(&mut f.sim, 1, f.ship, at);
    f.sim.units[boat].on_map = true;
    let rider = unit(&mut f.sim, 1, f.citizen, at);
    f.sim.units[rider].type_index = 0x32;
    f.sim.units[rider].inside_unit = Some(boat);
    f.sim.units[rider].on_map = false;
    assert!(
        f.sim.think_carry(boat),
        "the island's coast is found from the coastal cell's water"
    );
    let dest = f.sim.units[boat]
        .orders
        .front()
        .and_then(|o| o.move_dest())
        .expect("a move order");
    assert!(dest.cell().x >= 10, "towards the island: {dest:?}");
}

/// The shore arm of `think_carry_civilian` (§195): a boat beside the land
/// scores its own cell's sixteen quarter-cells against the first
/// neighbour in the region (`MOVE[1]`, cell (7, 2)) and walks to the
/// nearest — quarter (32, 12), centre (6240, 2400) before the snap.
#[test]
fn a_boat_beside_the_land_walks_to_its_nearest_quarter_cell() {
    let mut f = fix();
    f.sim.world.set_region_flags(f.land, 8);
    let boat = unit(
        &mut f.sim,
        1,
        f.ship,
        Pos::new(8 * 0x300 + 700, 3 * 0x300 + 0x180),
    );
    f.sim.units[boat].on_map = true;
    let rider = unit(
        &mut f.sim,
        1,
        f.citizen,
        Pos::new(8 * 0x300 + 700, 3 * 0x300 + 0x180),
    );
    f.sim.units[rider].type_index = 0x32;
    f.sim.units[rider].inside_unit = Some(boat);
    f.sim.units[rider].on_map = false;
    assert!(f.sim.think_carry(boat));
    let dest = f.sim.units[boat]
        .orders
        .front()
        .and_then(|o| o.move_dest())
        .expect("a move order");
    assert_eq!((dest.x, dest.y), (6264, 2424));
}

/// `think_carry`'s attack arm (`docs/AI.md` §235): a boat carrying a
/// captain that attacks sails it to the centre of its army's target's
/// cell, snapped to 48 — run879's `1/225`, (6240, 7008) to (6552,
/// 7320); here (6240, 2000) in cell (8, 2) to (6552, 1944). No group,
/// no army or a dead target answer no; a boat beside the army's region
/// answers yes, puts its rider ashore and, emptied, dies (item 3640,
/// `think_carry@005f2ee0:108`–`115`; run1301's Galleon `1/68` on 2163).
#[test]
fn a_boat_carrying_a_soldier_sails_to_its_army_s_target() {
    let mut f = fix();
    let soldier = f.sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: crate::combat::Profile {
            attack: 10,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let boat = unit(&mut f.sim, 1, f.ship, tile_pos(36, 12));
    f.sim.units[boat].on_map = true;
    let rider = unit(&mut f.sim, 1, soldier, tile_pos(36, 12));
    f.sim.units[rider].captain = true;
    f.sim.units[rider].inside_unit = Some(boat);
    f.sim.units[rider].on_map = false;
    assert!(!f.sim.think_carry(boat), "a soldier with no group: no");
    let slot = f.sim.init_army(1, None);
    let target = unit(&mut f.sim, 0, f.citizen, Pos::new(6240, 2000));
    {
        let a = &mut f.sim.armies[1].list[slot];
        a.group.pool = Some(3);
        a.target = Some(Obj::Unit(target));
        a.reg = None;
    }
    f.sim.units[rider].group_ptr = Some(3);
    assert!(
        f.sim.think_carry(boat),
        "a soldier of an army with a target"
    );
    let dest = f.sim.units[boat]
        .orders
        .front()
        .and_then(|o| o.move_dest())
        .expect("a move order");
    assert_eq!((dest.x, dest.y), (6552, 1944));
    // Out at sea with the target dead: the region arm's seam.
    f.sim.units[boat].orders.clear();
    f.sim.units[target].health = 0;
    assert!(!f.sim.obj_alive(Obj::Unit(target)), "the target is dead");
    assert!(!f.sim.think_carry(boat), "no live target: no");
    assert!(f.sim.units[boat].orders.is_empty());
    // Beside the army's region: `eject_contents(0, -1, 0, 1)`, the
    // rider ashore with its own empty list, and the emptied boat dead.
    f.sim.units[boat].pos = Pos::new(8 * 0x300 + 700, 3 * 0x300 + 0x180);
    f.sim.armies[1].list[slot].reg = Some(f.land);
    assert!(f.sim.think_carry(boat), "ashore beside its army's region");
    assert_eq!(f.sim.units[rider].inside_unit, None, "the rider is out");
    assert!(f.sim.units[rider].on_map, "and on the map");
    assert!(f.sim.units[rider].orders.is_empty(), "with its own list");
    assert!(!f.sim.units[boat].alive(), "the emptied boat dies");
}

#[test]
fn needs_transport_reads_the_shore() {
    let f = fix();
    let land = Pos::new(20, 10);
    let land2 = Pos::new(21, 10);
    let water = Pos::new(40, 10);
    assert_eq!(f.sim.needs_transport(land, land2), 0);
    assert_eq!(f.sim.needs_transport(land, water), NEEDS_EMBARK);
    assert_eq!(f.sim.needs_transport(water, land), NEEDS_DISEMBARK);
    assert_eq!(f.sim.needs_transport(water, water), 0);
}

/// **`snap_center`'s dock arm** (item 803, `docs/GOLDEN.md` §28): a
/// Dock whose centred tile `blocked_site` refuses takes the first tile
/// of the `move_x/move_y` spiral that it clears, in the spiral's own
/// order; a clear tile, or one with nothing clear within four, stays;
/// and nobody's shore clears nothing.
#[test]
fn a_dock_asked_off_its_shore_moves_along_the_spiral() {
    let mut f = fix();
    let placed = |f: &Fix, tx: i32| {
        f.sim
            .snap_center_placed(f.dock, tile_pos(tx, 15), Some(0))
            .tile()
    };
    // Unowned ground: every site is refused, so every Dock stays.
    assert_eq!(placed(&f, 32), Pos::new(32, 15));
    for y in 0..8 {
        for x in 0..12 {
            f.sim.world.set_owner(
                Cell::new(x, y),
                crate::world::Owner::Player(0),
                crate::world::Owner::None,
            );
        }
    }
    // Clear where asked.
    assert_eq!(
        f.sim.blocked_site(Some(0), f.dock, tile_pos(33, 15), None),
        crate::place::Blocked::Clear
    );
    assert_eq!(placed(&f, 33), Pos::new(33, 15));
    // Too little water: north-east, index 3, before east, index 4.
    assert_eq!(placed(&f, 32), Pos::new(33, 14));
    // Too much: west and south, index 18, before west and north.
    assert_eq!(placed(&f, 36), Pos::new(34, 16));
    // Four tiles out, the radius-4 ring's index 57.
    assert_eq!(placed(&f, 29), Pos::new(33, 11));
    // Five tiles out: nothing in reach, so it stays.
    assert_eq!(placed(&f, 28), Pos::new(28, 15));
    // The plain snap never moves.
    assert_eq!(
        f.sim.snap_center(f.dock, tile_pos(32, 15)).tile(),
        Pos::new(32, 15)
    );
}

#[test]
fn dock_tiles_are_water_cells_on_a_free_shore() {
    let mut f = fix();
    // The water cell next to the coastal column: its west neighbour is
    // the shore, whose last tile column is free.
    assert!(f.sim.is_dock_tile(Cell::new(8, 3)));
    // Open water two cells out: every neighbour is water or the map's
    // edge.
    assert!(!f.sim.is_dock_tile(Cell::new(10, 3)));
    // A land cell is never one; nor is a coastal-flagged cell.
    assert!(!f.sim.is_dock_tile(Cell::new(7, 3)));
    // A footprint on the shore's edge tiles blocks that direction.
    for ty in 12..16 {
        f.sim
            .world
            .set_tile_field(Pos::new(31, ty), tile::OBJECT, tile::OBJECT_BUILDING);
    }
    assert!(!f.sim.is_dock_tile(Cell::new(8, 3)));
    assert!(f.sim.is_dock_tile(Cell::new(8, 4)));
    // A forest on the first shore tile blocks it too.
    f.sim
        .world
        .set_tile_field(Pos::new(31, 16), tile::SURFACE, tile::SURFACE_FOREST);
    assert!(!f.sim.is_dock_tile(Cell::new(8, 4)));
}

#[test]
fn coasts_are_computed_from_the_cells() {
    let f = fix();
    assert!(f.sim.world.is_coast(f.land, f.sea));
    assert!(f.sim.world.is_coast(f.sea, f.land));
    assert!(f.sim.world.is_coast(f.land, f.land));
    assert_eq!(f.sim.world.num_coasts(f.land), 1);
    // A sea region counts itself, and nothing else (`Region::num_coasts`).
    assert_eq!(f.sim.world.num_coasts(f.sea), 1);
}
