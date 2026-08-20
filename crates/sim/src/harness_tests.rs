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
    let mut u = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    u.in_supply = true;
    sim.units.push(u);
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[0].health, 100);
    // The period is still set — the unit is in hostile territory and the
    // interface says so. It is the damage that supply prevents.
    assert!(sim.units[0].attrition > 0);
    assert!(sim.units[0].sheltered);
}

#[test]
fn supply_does_not_shelter_militia() {
    let mut sim = skirmish(4);
    let mut u = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    u.in_supply = true;
    u.kind.militia = true;
    sim.units.push(u);
    assert!(!run(&mut sim, 600).is_empty());
    assert!(sim.units[0].health < 100);
}

#[test]
fn supply_does_not_shelter_a_peacetime_border_violation() {
    let mut sim = skirmish(1);
    sim.at_war = vec![vec![false; 2]; 2];
    // Health well above what the run can remove, so the cadence is what this
    // measures rather than how fast the unit dies.
    let mut u = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 1000);
    u.in_supply = true;
    sim.units.push(u);
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
