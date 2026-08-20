use super::*;
use crate::attrition::{PlayerState, Resistance, StrengthMods, strength};
use crate::territory::{City, NationBonuses, PlayerBorders, Wonders, city_source};
use crate::world::UNITS_PER_CELL;

const T_PEACE: i32 = Tuning::RON.peace_attrition;

fn centre_of(c: Cell) -> Pos {
    Pos::new(
        c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    )
}

/// Two players on a strip of land. Player 1 holds a city at one end and enough
/// attrition tech to make its border bite; player 0 has the units.
fn skirmish(tech_steps: usize) -> Sim {
    let t = Tuning::RON;
    let mut world = World::new(24, 1);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(23, 0));

    let mut sim = Sim::new(t, world, 2);
    let borders = PlayerBorders::new(
        &t,
        0,
        1,
        1,
        &Wonders::default(),
        &NationBonuses::default(),
        0,
    );
    sim.sources.push(city_source(
        &t,
        1,
        &borders,
        &City {
            pos: centre_of(Cell::new(20, 0)),
            level: 0,
            capital: false,
            temple: false,
            owner_agrees: true,
        },
    ));
    sim.players[1] = PlayerState {
        strength: strength(&t, tech_steps, &StrengthMods::default()),
        team: 1,
        ..PlayerState::default()
    };
    sim.players[0].team = 0;
    sim.declare_war(0, 1);
    sim.recompute_territory();
    sim
}

/// Parks a supply wagon on a cell, registered in its owner's supply list the
/// way `Unit::init` registers one.
///
/// Wagons are exempt from attrition themselves and never sheltered by another
/// wagon, so one standing in a war zone is inert apart from what it supplies.
fn wagon_at(sim: &mut Sim, owner: Player, index: i16, c: Cell) -> usize {
    let mut w = Unit::new(owner, index, centre_of(c), 100);
    w.kind.supply_unit = true;
    w.kind.exempt_kind = true;
    sim.add_unit(w)
}

/// Gives a unit a Citizen's speed and a turn rate quick enough that turning is
/// not what the test is measuring.
fn make_mobile(sim: &mut Sim, unit: usize, facing: movement::Angle) {
    sim.units[unit].movement.speed = 25;
    sim.units[unit].movement.turn_rate = 0x0800_0000;
    sim.units[unit].movement.facing = facing;
}

/// Runs `frames` frames and returns every attrition tick that happened.
fn run(sim: &mut Sim, frames: i64) -> Vec<Tick> {
    let mut log = Vec::new();
    for _ in 0..frames {
        log.extend(sim.tick());
    }
    log
}

fn frames_of(ticks: &[Tick], unit: usize) -> Vec<i64> {
    ticks
        .iter()
        .filter(|t| t.unit == unit)
        .map(|t| t.frame)
        .collect()
}

#[test]
fn a_unit_outside_the_border_never_bleeds() {
    let mut sim = skirmish(1);
    // Cell 8 is 48 tiles from the city, past its 44-tile limit.
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(8, 0)), 100));
    assert_eq!(sim.world.owner_at(sim.units[0].pos), Owner::None);
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].health, 100);
}

#[test]
fn a_unit_inside_a_hostile_border_bleeds_on_schedule() {
    let mut sim = skirmish(1);
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    assert_eq!(sim.world.owner_at(sim.units[0].pos), Owner::Player(1));

    // One attrition tech, a plain unit: 48 frames a tick, 16 damage a tick, so
    // 3.2 seconds and a sixth of a full-health squaddie's life. Index 0, so
    // the phase is the raw frame and ticks land on multiples of 48 — including
    // frame 0, where the first refresh also happens.
    let ticks = run(&mut sim, 48 * 3);
    assert_eq!(frames_of(&ticks, 0), vec![0, 48, 96]);
    assert!(ticks.iter().all(|t| t.damage == 16));
    assert_eq!(sim.units[0].health, 100 - 48);
    assert_eq!(48 / FRAMES_PER_SECOND, 3);
}

#[test]
fn the_unit_index_staggers_the_schedule() {
    // The whole reason `SubObjectData::o` is in the phase: two identical units
    // standing in the same place bleed on different frames.
    let mut sim = skirmish(1);
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    sim.units
        .push(Unit::new(0, 12, centre_of(Cell::new(15, 0)), 100));
    let ticks = run(&mut sim, 100);
    assert_eq!(frames_of(&ticks, 0), vec![0, 48, 96]);
    // Unit 1 is twelve frames ahead in phase, so it ticks twelve frames
    // earlier on the same 48-frame grid.
    assert_eq!(frames_of(&ticks, 1), vec![36, 84]);
    // Same rate, different frames — which is the point.
    assert_eq!(sim.units[0].attrition, sim.units[1].attrition);
}

#[test]
fn attrition_kills() {
    let mut sim = skirmish(4);
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    // Four tech steps is strength 8, so six frames a tick: a lone figure dies
    // in seven ticks, the first of them on frame 0.
    let ticks = run(&mut sim, 600);
    assert_eq!(ticks.iter().find(|t| t.killed).map(|t| t.frame), Some(36));
    assert!(!sim.units[0].alive());
    // And nothing happens after death.
    assert_eq!(frames_of(&ticks, 0).len(), 7);
}

#[test]
fn a_bigger_squad_bleeds_slower_per_figure() {
    let mut sim = skirmish(1);
    let mut four = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    four.squad_size = 4;
    sim.units.push(four);
    let total: i32 = run(&mut sim, 48 * 4).iter().map(|t| t.damage).sum();
    // Four ticks at 4 damage rather than four at 16.
    assert_eq!(total, 16);
}

#[test]
fn leaving_the_border_stops_the_bleeding_only_at_the_next_refresh() {
    // The behaviour a countdown implementation would get wrong. The period is
    // refreshed every 32 frames, so a stale one can still fire after the unit
    // is safely outside.
    let mut sim = skirmish(1);
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    run(&mut sim, 40);
    assert_eq!(sim.units[0].attrition, 48);

    // Step outside on frame 40. The refresh at 64 clears the period, but the
    // tick at 48 lands first — while the unit is already out.
    sim.units[0].pos = centre_of(Cell::new(8, 0));
    assert_eq!(sim.world.owner_at(sim.units[0].pos), Owner::None);
    let after = run(&mut sim, 600);
    assert_eq!(frames_of(&after, 0), vec![48]);
    assert_eq!(sim.units[0].attrition, 0);
}

#[test]
fn entering_the_border_costs_nothing_until_the_next_refresh() {
    // The same fact from the other side, and the one that makes a raid work:
    // dash in, dash out, and the border may never notice.
    let mut sim = skirmish(1);
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(8, 0)), 100));
    run(&mut sim, 33);
    // Enter on frame 33 and leave on frame 63, before the refresh at 64.
    sim.units[0].pos = centre_of(Cell::new(15, 0));
    assert!(run(&mut sim, 30).is_empty());
    sim.units[0].pos = centre_of(Cell::new(8, 0));
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].health, 100);
}

#[test]
fn a_supply_radius_shelters_a_unit_from_a_war_zone() {
    let mut sim = skirmish(4);
    sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].health, 100);
    // The period is still set — the unit is in hostile territory and the
    // interface says so. It is the damage that supply prevents.
    assert!(sim.units[0].attrition > 0);
    assert!(sim.units[0].sheltered);
}

#[test]
fn a_wagon_stops_supplying_at_the_edge_of_its_radius() {
    // Fourteen tiles is three and a half cells, so a wagon three cells away
    // reaches and one four cells away does not. Cell centres are half a tile
    // off the grid in each axis, which cancels between two of them.
    let mut sim = skirmish(4);
    sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    let near = wagon_at(&mut sim, 0, 1, Cell::new(12, 0));
    assert!(sim.supplied_at(0, sim.units[0].pos));
    // Move it one cell further and the same wagon is out of reach: 16 tiles.
    sim.units[near].pos = centre_of(Cell::new(11, 0));
    assert!(!sim.supplied_at(0, sim.units[0].pos));
    assert!(!run(&mut sim, 600).is_empty());
}

#[test]
fn an_allys_wagon_supplies_nobody() {
    // Player 0 and player 1 are at war in `skirmish`; make a third player who
    // is nobody's enemy and give them the wagon. The list is per player and
    // never consulted across players, so it does not matter that they are
    // friendly — which is the point.
    let mut sim = skirmish(4);
    let third = sim.add_player();
    assert_eq!(third, 2);
    sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    wagon_at(&mut sim, 2, 1, Cell::new(15, 0));
    assert!(!sim.supplied_at(0, sim.units[0].pos));
    assert!(!run(&mut sim, 600).is_empty());
    assert!(sim.units[0].health < 100);
}

#[test]
fn a_wagon_in_a_transport_supplies_nobody() {
    let mut sim = skirmish(4);
    sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    let w = wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    sim.units[w].on_map = false;
    assert!(!sim.supplied_at(0, sim.units[0].pos));
    assert!(!run(&mut sim, 600).is_empty());
}

#[test]
fn supply_does_not_shelter_militia() {
    let mut sim = skirmish(4);
    let mut u = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    u.kind.militia = true;
    sim.add_unit(u);
    wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    assert!(!run(&mut sim, 600).is_empty());
    assert!(sim.units[0].health < 100);
}

#[test]
fn a_wagon_does_not_shelter_itself() {
    // Two wagons standing on each other still bleed: the check is on the
    // victim's own type, before any search happens. In the shipped game they
    // are exempt from attrition outright, so make this one subject to it in
    // order to see the supply refusal on its own.
    let mut sim = skirmish(4);
    let mut w = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    w.kind.supply_unit = true;
    sim.add_unit(w);
    wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    assert!(sim.supplied_at(0, sim.units[0].pos));
    assert!(!run(&mut sim, 600).is_empty());
    assert!(!sim.units[0].sheltered);
}

#[test]
fn supply_does_not_shelter_a_peacetime_border_violation() {
    let mut sim = skirmish(1);
    sim.at_war = vec![vec![false; 2]; 2];
    // Health well above what the run can remove, so the cadence is what this
    // measures rather than how fast the unit dies.
    sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 1000));
    wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    let ticks = run(&mut sim, 96);
    // The 8-frame peace period, and supply is no help against it. Six times
    // the rate of the war zone next door, through a supply wagon that would
    // have stopped the war zone entirely.
    assert_eq!(sim.units[0].attrition, T_PEACE);
    assert!(sim.units[0].ignores_supply);
    assert!(!sim.units[0].sheltered);
    assert_eq!(frames_of(&ticks, 0).len(), 96 / T_PEACE as usize);
}

#[test]
fn a_garrisoned_unit_is_not_processed_at_all() {
    let mut sim = skirmish(4);
    let mut u = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    u.on_map = false;
    sim.units.push(u);
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].attrition, 0);
}

#[test]
fn a_players_own_territory_is_safe() {
    let mut sim = skirmish(1);
    // Give the unit to the player who owns the border.
    sim.units
        .push(Unit::new(1, 0, centre_of(Cell::new(15, 0)), 100));
    assert!(run(&mut sim, 600).is_empty());
}

#[test]
fn the_statue_of_liberty_walks_through_anything() {
    let mut sim = skirmish(4);
    sim.players[0].resistance = Resistance::Immune;
    sim.units
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].health, 100);
}

#[test]
fn the_whole_run_is_deterministic() {
    // The property the entire project rests on. Two independent runs of the
    // same setup must agree frame for frame, not merely in the end state.
    let once = || {
        let mut sim = skirmish(3);
        sim.units
            .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100));
        sim.units
            .push(Unit::new(0, 7, centre_of(Cell::new(21, 0)), 100));
        let log = run(&mut sim, 500);
        (log, sim.units.clone())
    };
    let (log_a, units_a) = once();
    let (log_b, units_b) = once();
    assert_eq!(log_a, log_b);
    assert_eq!(units_a, units_b);
    assert!(!log_a.is_empty());
}

// ---------------------------------------------------------------------------
// Four mechanics at once
//
// Everything above exercises one mechanic. These run territory, movement,
// attrition and supply together over a few hundred frames, which is the only
// place an ordering mistake between them can show up.
// ---------------------------------------------------------------------------

/// The first cell `skirmish`'s city actually owns.
///
/// Worth stating rather than assuming: a plain city reaches 44 tiles, which is
/// eleven cells, so a city at cell 20 owns cell 9 and stops. A unit starting at
/// cell 8 is one cell outside the border, not seven.
const FIRST_HOSTILE_CELL: i32 = 9;

#[test]
fn the_border_starts_where_the_limit_puts_it() {
    let sim = skirmish(1);
    assert_eq!(
        sim.world.owner(Cell::new(FIRST_HOSTILE_CELL, 0)),
        Owner::Player(1)
    );
    assert_eq!(
        sim.world.owner(Cell::new(FIRST_HOSTILE_CELL - 1, 0)),
        Owner::None
    );
}

#[test]
fn a_unit_ordered_over_the_border_walks_in_and_starts_bleeding() {
    let mut sim = skirmish(1);
    let home = centre_of(Cell::new(8, 0));
    let target = centre_of(Cell::new(15, 0));
    let u = sim.add_unit(Unit::new(0, 0, home, 200));
    make_mobile(&mut sim, u, movement::Angle::EAST);
    sim.order_move(u, target);

    // It crosses at about frame 23, but the period is only refreshed every 32
    // frames and the first bleed then waits for the 48-frame grid — so nothing
    // happens for the first 40 frames even though it is already inside.
    let early = run(&mut sim, 40);
    assert!(early.is_empty(), "bled sooner than the cadence allows");
    assert_eq!(
        sim.world.owner_at(sim.units[u].pos),
        Owner::Player(1),
        "should be over the border by now"
    );
    assert_eq!(sim.units[u].health, 200);

    // Then it bleeds, and it still gets where it was going.
    let ticks = run(&mut sim, 400);
    assert_eq!(sim.units[u].pos, target, "never arrived");
    assert_eq!(
        sim.units[u].movement.dest, None,
        "order not cleared on arrival"
    );
    assert_eq!(sim.units[u].pos.y, home.y, "wandered off the strip");
    assert!(!ticks.is_empty(), "stood in hostile territory unharmed");
    assert!(sim.units[u].health < 200);
}

#[test]
fn a_wagon_covering_the_march_makes_it_free() {
    // The same march with supply in place. The wagon sits at cell 12: fourteen
    // tiles is three and a half cells, so from there it covers cell 9 — where
    // the border starts — through to the destination at cell 15. This is the
    // one test that needs territory, movement, attrition and supply to agree at
    // the same time.
    let mut sim = skirmish(1);
    let target = centre_of(Cell::new(15, 0));
    let u = sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(8, 0)), 200));
    make_mobile(&mut sim, u, movement::Angle::EAST);
    sim.order_move(u, target);
    wagon_at(&mut sim, 0, 1, Cell::new(12, 0));

    // Stopping on frame 480 is deliberate: the refresh clears the shelter flag
    // every 32 frames, and 480 is both a refresh and a bleed, so the flag is
    // set by the veto that just happened rather than by an older one.
    let ticks = run(&mut sim, 481);
    assert_eq!(sim.units[u].pos, target, "never arrived");
    assert!(ticks.is_empty(), "bled while inside a supply radius");
    assert_eq!(sim.units[u].health, 200);
    // The period is still computed and still shown. Supply vetoes the damage;
    // it does not slow the rate.
    assert!(sim.units[u].attrition > 0);
    assert!(sim.units[u].sheltered);
}

#[test]
fn walking_out_does_not_stop_the_bleeding_until_the_next_refresh() {
    // `docs/ATTRITION.md` claims a unit can take one more tick up to two
    // seconds after leaving hostile ground, because the period is stale until
    // the next refresh. With movement wired in that is finally testable end to
    // end rather than by teleporting a unit between frames.
    //
    // From the middle of cell 10, walking west: it leaves hostile ground around
    // frame 34, the refresh that would clear the period is not until 64, and
    // the 48-frame bleed grid fires at 48 — in the gap.
    let mut sim = skirmish(1);
    let safe = centre_of(Cell::new(8, 0));
    let u = sim.add_unit(Unit::new(0, 0, centre_of(Cell::new(10, 0)), 200));
    make_mobile(&mut sim, u, movement::Angle::WEST);
    sim.order_move(u, safe);

    let ticks = run(&mut sim, 56);
    assert_eq!(sim.units[u].pos, safe, "never got clear");
    assert_eq!(
        sim.world.owner_at(sim.units[u].pos),
        Owner::None,
        "not actually out of hostile territory"
    );
    let stale: Vec<_> = ticks.iter().filter(|t| t.frame == 48).collect();
    assert_eq!(
        stale.len(),
        1,
        "expected the stale tick at frame 48, got {ticks:?}"
    );
    assert!(sim.units[u].attrition > 0, "period cleared too early");

    // And the refresh at 64 does finally stop it.
    let after = run(&mut sim, 200);
    assert!(after.is_empty(), "still bleeding on safe ground: {after:?}");
    assert_eq!(sim.units[u].attrition, 0);
}

#[test]
fn the_border_that_kills_also_pays() {
    // Territory is the only thing in the game that both damages whoever stands
    // in it and earns its owner money, and the two halves read the same
    // ownership map. `skirmish` gives player 1 a city on a 24-cell strip; the
    // cells its border claims are the cells the territory tax divides by.
    let mut sim = skirmish(4);
    sim.update_territory_holdings();

    let owned = sim.holdings[1].territory;
    assert!(owned > 0 && owned < 24, "border claims {owned} of 24");
    assert_eq!(sim.holdings[1].land_size, 24);
    assert_eq!(sim.holdings[0].territory, 0, "player 0 has no city");

    // Full taxation, and nothing else at all: no cities in the economy sense,
    // no citizens, no markets. Every coin below comes from holding ground.
    sim.holdings[1].taxation = 2;
    let wealth = economy::Resource::Wealth.index();
    let period = Tuning::RON.gather_rate as i64;
    let before = sim.ledgers[1].bucket[wealth];
    run(&mut sim, period);
    let earned = sim.ledgers[1].bucket[wealth] - before;

    // A share f of the map at 100% taxation is f * 100 wealth per period.
    assert_eq!(earned, owned * 100 / 24);
    assert_eq!(
        sim.ledgers[0].bucket[wealth],
        Tuning::RON.starting_goods[wealth],
        "player 0 holds no ground and earns nothing"
    );
}

#[test]
fn a_city_pays_from_the_first_frame_of_the_game() {
    // Income runs before objects, and the reassembly is unconditional on frame
    // zero, so a player who owns something is earning on the very first tick
    // rather than waiting out the 512-frame cadence.
    let mut sim = skirmish(4);
    sim.holdings[0].cities.push(economy::City {
        market: true,
        university: true,
        sites: vec![economy::Site::new(economy::Resource::Food, 3)],
        ..economy::City::default()
    });
    sim.economy_changed(0);

    sim.tick();
    let shown = |r: economy::Resource| sim.ledgers[0].income[r.index()] / economy::RATE_SCALE;
    // Three farmers at PEASANT_RATE, plus the ten food a city is worth on its
    // own, plus the ten timber, the market's ten wealth and the university's
    // ten knowledge.
    assert_eq!(shown(economy::Resource::Food), 40);
    assert_eq!(shown(economy::Resource::Timber), 10);
    assert_eq!(shown(economy::Resource::Wealth), 10);
    assert_eq!(shown(economy::Resource::Knowledge), 10);

    // And the starting goods are there and untouched by the first frame.
    let food = economy::Resource::Food.index();
    assert_eq!(
        sim.ledgers[0].bucket[food],
        Tuning::RON.starting_goods[food]
    );
}

/// The Citizen, as `unitrules.xml` writes it: `2f`, `1f support`,
/// `PROGRESSION 0`, `POP 1`, forty hit points.
fn citizen_type() -> UnitType {
    UnitType {
        price: cost::Price {
            class: cost::RampClass::Worker,
            pop: 1,
            ..cost::Price::free()
                .with_base(economy::Resource::Food, 2)
                .with_support(economy::Resource::Food, 1)
        },
        kind: attrition::UnitKind::default(),
        group: None,
        hits: 40,
    }
}

#[test]
fn income_buys_a_citizen_and_the_next_one_costs_more() {
    // The loop closing: a player earns, the earnings buy a unit, and the
    // unit's price has moved by the time the next one is asked for. This is
    // `docs/ECONOMY.md` feeding `docs/COSTS.md`.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let at = centre_of(Cell::new(2, 0));

    assert_eq!(sim.price_of(0, citizen)[food], 20, "the first is unramped");

    let purse = sim.ledgers[0].bucket[food];
    sim.produce(0, citizen, at)
        .expect("two hundred starting food buys one");
    assert_eq!(sim.ledgers[0].bucket[food], purse - 20);
    assert_eq!(sim.muster[0].control, 1);

    // One owned, one food of ramp.
    assert_eq!(sim.price_of(0, citizen)[food], 21);
    sim.produce(0, citizen, at).unwrap();
    assert_eq!(sim.price_of(0, citizen)[food], 22);
    assert_eq!(sim.ledgers[0].bucket[food], purse - 41);

    // And they are real units, standing where they were put.
    assert_eq!(sim.units.len(), 2);
    assert!(sim.units.iter().all(|u| u.alive() && u.pos == at));
}

#[test]
fn the_population_cap_refuses_before_the_price_does() {
    // `check_population` runs before `can_pay`, so a player at the cap is
    // turned away without being charged — which is observable, because the
    // stockpile is untouched.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let at = centre_of(Cell::new(2, 0));

    // The Ancient age caps a player at twenty-five, and a citizen is one pop.
    assert_eq!(sim.muster[0].cap, 25);
    sim.ledgers[0].bucket[food] = 100_000;
    for _ in 0..25 {
        sim.produce(0, citizen, at).expect("under the cap");
    }
    assert_eq!(sim.muster[0].control, 25);

    let purse = sim.ledgers[0].bucket[food];
    assert_eq!(sim.produce(0, citizen, at), Err(Refused::Population));
    assert_eq!(sim.ledgers[0].bucket[food], purse, "refused, not charged");

    // An age raises the cap, and the same order goes through.
    sim.muster[0].age = 1;
    sim.recompute_pop_caps();
    assert_eq!(sim.muster[0].cap, 50);
    sim.produce(0, citizen, at)
        .expect("room in the Classical age");
}

#[test]
fn an_empty_purse_refuses_and_takes_nothing() {
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let at = centre_of(Cell::new(2, 0));

    sim.ledgers[0].bucket[food] = 19;
    assert_eq!(sim.produce(0, citizen, at), Err(Refused::Cost));
    assert_eq!(sim.ledgers[0].bucket[food], 19);
    assert!(sim.units.is_empty());

    sim.ledgers[0].bucket[food] = 20;
    sim.produce(0, citizen, at).expect("exactly enough");
    assert_eq!(sim.ledgers[0].bucket[food], 0);
}

#[test]
fn a_bought_unit_walks_and_bleeds_like_any_other() {
    // The whole crate in one test: a citizen is paid for, walks into a hostile
    // border, and starts taking attrition there. Six mechanics, one unit.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let start = Cell::new(2, 0);
    let unit = sim
        .produce(0, citizen, centre_of(start))
        .expect("affordable");
    make_mobile(&mut sim, unit, movement::Angle::EAST);

    let inside = (0..24)
        .map(|x| Cell::new(x, 0))
        .find(|&c| sim.world.owner(c).player() == Some(1))
        .expect("player 1's border claims something");
    sim.order_move(unit, centre_of(inside));

    let mut ticks = 0;
    for _ in 0..4000 {
        ticks += sim.tick().len();
        if ticks > 0 {
            break;
        }
    }
    assert!(ticks > 0, "a unit that walked in should be bleeding");
    assert!(sim.units[unit].health < 40);
}
