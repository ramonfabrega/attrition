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

/// A cell's centre, on a quarter-tile centre: every move destination is
/// snapped to one (`docs/ORDERS.md` §4.1), so a unit told to walk to
/// `centre_of(c)` arrives at `quarter_of(c)`; a unit that starts on one and
/// walks due east or west stays on the row.
fn quarter_of(c: Cell) -> Pos {
    let p = centre_of(c);
    Pos::new(p.x.div_euclid(48) * 48 + 24, p.y.div_euclid(48) * 48 + 24)
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
        ..PlayerState::default()
    };
    sim.declare_war(0, 1);
    sim.recompute_territory();
    sim
}

/// Parks a supply wagon on a cell, registered in its owner's supply list the
/// way `Unit::init` registers one.
///
/// A wagon in a war zone takes no attrition itself — the supply-unit
/// exemption at the computed-period step — so one standing there is inert
/// apart from what it supplies. Over a peaceful border it bleeds like anything
/// else; see `a_wagon_bleeds_over_a_peaceful_border_and_nothing_shelters_it`.
fn wagon_at(sim: &mut Sim, owner: Player, index: i16, c: Cell) -> usize {
    let mut w = Unit::new(owner, index, centre_of(c), 100);
    w.kind.supply_unit = true;
    sim.add_unit(w)
}

/// A Citizen's turning: 45° a frame, on foot, so it turns instantly from a
/// standstill and turning is not what any test here is measuring.
const CITIZEN_TURNING: movement::Turning = movement::Turning {
    type_turn_speed: movement::degrees_to_angle(45).0,
    packed: false,
    instant_from_stop: true,
    wide_limit: false,
};

/// Gives a unit a Citizen's speed and turning, facing a given way.
fn make_mobile(sim: &mut Sim, unit: usize, facing: movement::Angle) {
    let m = &mut sim.units[unit].movement;
    m.speed = 25;
    m.turning = CITIZEN_TURNING;
    m.set_facing(facing);
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

    // One attrition tech, a lone figure: 48 frames a tick, sixteen sixteenths
    // a tick — one whole hit point every 3.2 seconds, so a hundred-point unit
    // lasts over five minutes. Slow, and the right order of magnitude: a
    // campaign abroad without supply is a bleed, not a rout. Index 0, so the
    // phase is the raw frame and ticks land on multiples of 48 — including
    // frame 0, where the first refresh also happens.
    let ticks = run(&mut sim, 48 * 3);
    assert_eq!(frames_of(&ticks, 0), vec![0, 48, 96]);
    assert!(ticks.iter().all(|t| t.sixteenths == 16 && t.lost == 1));
    assert_eq!(sim.units[0].health, 100 - 3);
    assert_eq!(sim.units[0].damage_frac, 0);
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
        .push(Unit::new(0, 0, centre_of(Cell::new(15, 0)), 10));
    // Four tech steps is strength 8, so six frames a tick and one hit point
    // a tick: a ten-point lone figure dies on its tenth tick, the first of
    // them on frame 0.
    let ticks = run(&mut sim, 600);
    assert_eq!(ticks.iter().find(|t| t.killed).map(|t| t.frame), Some(54));
    assert!(!sim.units[0].alive());
    // And nothing happens after death.
    assert_eq!(frames_of(&ticks, 0).len(), 10);
}

#[test]
fn a_bigger_squad_bleeds_slower_per_figure() {
    let mut sim = skirmish(1);
    let mut four = Unit::new(0, 0, centre_of(Cell::new(15, 0)), 100);
    four.squad_size = 4;
    sim.units.push(four);
    let ticks = run(&mut sim, 48 * 4);
    // Four ticks at four sixteenths rather than four at sixteen: one whole
    // point over the run, landing on the fourth tick, nothing carried over.
    assert_eq!(ticks.iter().map(|t| t.sixteenths).sum::<i32>(), 16);
    assert_eq!(
        ticks.iter().map(|t| t.lost).collect::<Vec<_>>(),
        [0, 0, 0, 1]
    );
    assert_eq!(sim.units[0].health, 99);
    assert_eq!(sim.units[0].damage_frac, 0);
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
fn a_wagon_is_safe_in_a_war_zone() {
    // The supply-unit exemption: at war a wagon gets no period at all, not
    // even one that supply would then have to veto.
    let mut sim = skirmish(4);
    let w = wagon_at(&mut sim, 0, 0, Cell::new(15, 0));
    assert!(run(&mut sim, 600).is_empty());
    assert_eq!(sim.units[w].attrition, 0);
    assert_eq!(sim.units[w].health, 100);
}

#[test]
fn a_wagon_bleeds_over_a_peaceful_border_and_nothing_shelters_it() {
    // The same exemption is lifted by the peace flag, so over a peaceful
    // border a wagon takes the peace period like anything else — and since
    // that flag also defeats supply before the "not itself a supply unit"
    // refusal is reached, a second wagon standing on it changes nothing. The
    // self-shelter refusal is real but unobservable from attrition: the only
    // bleeds a wagon ever has are the ones that bypass supply anyway.
    let mut sim = skirmish(4);
    sim.at_war = vec![vec![false; 2]; 2];
    let w = wagon_at(&mut sim, 0, 0, Cell::new(15, 0));
    wagon_at(&mut sim, 0, 1, Cell::new(15, 0));
    assert!(sim.supplied_at(0, sim.units[w].pos));
    let ticks = run(&mut sim, 96);
    // Strength 8 computes to 6 frames, shorter than the 8-frame peace period,
    // and the shorter wins — the wagon gets both, like anything else.
    assert_eq!(sim.units[w].attrition, 6);
    assert!(sim.units[w].ignores_supply);
    assert!(!sim.units[w].sheltered);
    assert!(!ticks.is_empty());
    assert!(sim.units[w].health < 100);
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
    let home = quarter_of(Cell::new(8, 0));
    let target = quarter_of(Cell::new(15, 0));
    let u = sim.add_unit(Unit::new(0, 0, home, 200));
    make_mobile(&mut sim, u, movement::Angle::EAST);
    sim.order_move(u, target);

    // The border is 384 units east, and the unit covers 25 a frame — its
    // quoted speed, not the body's 34 — so it crosses on its sixteenth step.
    // But the period is only refreshed every 32 frames and the first bleed
    // then waits for the 48-frame grid, so nothing happens for the first 40
    // frames even though it is already inside.
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
    let target = quarter_of(Cell::new(15, 0));
    let u = sim.add_unit(Unit::new(0, 0, quarter_of(Cell::new(8, 0)), 200));
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
    // From the middle of cell 10, walking west at 25 a frame: the border is
    // 1152 units away, so it leaves hostile ground on its 47th step, the
    // refresh that would clear the period is not until 64, and the 48-frame
    // bleed grid fires at 48 — in the gap. (At the body's 34 a frame, which an
    // earlier draft used for the unit, it was out by 34.) It reaches the
    // middle of cell 8, 1536 units, on its 62nd.
    let mut sim = skirmish(1);
    let safe = quarter_of(Cell::new(8, 0));
    let u = sim.add_unit(Unit::new(0, 0, quarter_of(Cell::new(10, 0)), 200));
    make_mobile(&mut sim, u, movement::Angle::WEST);
    sim.order_move(u, safe);

    let ticks = run(&mut sim, 63);
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
fn a_citizen_reverses_on_the_spot_and_its_body_keeps_up() {
    // The two-position model end to end. A foot unit standing still turns
    // instantly, so a unit facing east and sent west covers a full 25 on the
    // first frame; its body, a step behind, closes at 34 and is back on the
    // unit by the end of the same frame, every frame, so the unit never stops
    // being "moving" until it arrives — and then its average speed decays
    // until it is stopped again and can turn instantly once more.
    let mut sim = skirmish(1);
    let start = quarter_of(Cell::new(4, 0));
    let u = sim.add_unit(Unit::new(0, 0, start, 200));
    make_mobile(&mut sim, u, movement::Angle::EAST);
    sim.order_move(u, quarter_of(Cell::new(3, 0)));

    sim.tick();
    let m = sim.units[u].movement;
    assert_eq!(m.facing, movement::Angle::WEST);
    assert_eq!(sim.units[u].pos, Pos::new(start.x - 25, start.y));
    assert_eq!(m.body.pos, sim.units[u].pos, "body fell behind");
    assert_eq!(m.body.last_speed, 25);

    // Thirty more frames is well past the 768-unit walk (31 steps in all); the
    // body is on the unit, last_speed has gone to zero and the average is
    // draining.
    run(&mut sim, 30);
    assert_eq!(sim.units[u].pos, quarter_of(Cell::new(3, 0)));
    assert_eq!(sim.units[u].movement.dest, None);
    run(&mut sim, 5);
    let m = sim.units[u].movement;
    assert_eq!(m.body.pos, sim.units[u].pos);
    assert_eq!(m.body.last_speed, 0);
    assert!(m.body.avg_speed < 25 && m.body.avg_speed > 0);
}

/// **A standing body swallows its whole owed turn in one frame**, however
/// slowly its type turns — `Guy::move@005d9240:53` writes `last_speed = 0` at
/// the head of its at-des branch, ahead of the `turn_towards` at the foot of
/// the same branch, and `GuyData::turn_speed@005de340:29` answers a zero
/// `last_speed` on a foot or mounted type (`guy_flags & 0x10`) with
/// `0x80000000`, which exceeds any turn that can be owed.
///
/// The arrival frame itself still reads the step it just took, so the snap
/// lands on the frame **after** the arrival — which is exactly the run10 rows
/// item 37 was: the AI scout at 96, 362 and 721, where the original's body had
/// already reached the order's angle and this crate's was still five degrees
/// short (`docs/MOVEMENT.md`, "The body step").
///
/// A five-degree turner is the whole of the test: at a Citizen's forty-five it
/// would land in one frame either way.
#[test]
fn a_standing_body_takes_its_whole_turn_in_one_frame() {
    const SLOW: movement::Turning = movement::Turning {
        type_turn_speed: movement::degrees_to_angle(5).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };
    let mut sim = skirmish(1);
    let start = quarter_of(Cell::new(4, 0));
    let u = sim.add_unit(Unit::new(0, 0, start, 200));
    make_mobile(&mut sim, u, movement::Angle::EAST);
    sim.units[u].movement.turning = SLOW;

    // The order's own facing, a quarter turn off the bearing the walk ends
    // on — `add_move_facing_order`'s second argument, which `arrive` hands to
    // `set_angle` and nothing snaps.
    let owed = movement::Angle::SOUTH;
    sim.add_move_facing_order(
        u,
        quarter_of(Cell::new(3, 0)),
        orders::MoveKind::MoveTo,
        orders::QueuePos::New,
        false,
        owed,
        None,
        false,
    );
    let mut arrived = None;
    for f in 0..64 {
        sim.tick();
        if sim.units[u].orders.is_empty() {
            arrived = Some(f);
            break;
        }
    }
    let arrived = arrived.expect("the walk finishes inside 64 frames");
    let m = sim.units[u].movement;
    assert_eq!(sim.units[u].pos, quarter_of(Cell::new(3, 0)));
    assert_eq!(m.heading, owed, "`arrive` set the order's angle");
    assert_eq!(
        m.facing,
        movement::Angle::WEST,
        "the arrival frame still reads the step it took, so it turns by the rate"
    );
    assert_ne!(m.body.last_speed, 0, "and that step is what it reads");

    // One frame later — and it is one, not the eighteen a five-degree turner
    // would need for a quarter turn.
    sim.tick();
    assert_eq!(sim.units[u].movement.facing, owed);
    assert_eq!(sim.units[u].movement.body.last_speed, 0);
    assert!(arrived > 0);
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
    // The level is the player's `TAX_n` ladder since item 232, so it is
    // granted rather than poked into `Holdings` — where the assembly would
    // now overwrite it.
    let mut tree = tech::TechTree::new();
    let tax = [
        Some(tree.add(tech::TypeDef::epoch("TAX_1", tech::Line::Commerce, 0))),
        Some(tree.add(tech::TypeDef::epoch("TAX_2", tech::Line::Commerce, 0))),
        None,
        None,
    ];
    tree.roles.taxation_preq = tax;
    sim.set_tech_tree(tree);
    sim.tech[1].tech[tax[1].unwrap()] = true;
    sim.assemble_holdings(1);
    assert_eq!(sim.holdings[1].taxation, 2);
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
    // rather than waiting out the 512-frame cadence. The holdings are
    // assembled from the live state (`crates/sim/src/holdings.rs`), so the
    // "something" is a real city: a finished Small City is worth
    // `CITY_GATHER` of food and timber on its own.
    let mut world = World::new(16, 16);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    sim.nation[0].human = true;
    let village = sim.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Village,
        x_size: 7,
        y_size: 7,
        flags: crate::build::flags::parse("ean"),
        job_time: 150,
        hits: 400,
        price: crate::cost::Price {
            kind: crate::cost::Kind::Building,
            ..crate::cost::Price::free()
        },
        ..crate::build::BuildType::default()
    });
    let b = sim.init_build(0, village, Pos::new(32 * 192, 32 * 192), false);
    sim.activate(b, false, false);
    assert!(
        sim.buildings[b].city.is_some(),
        "a finished city has a record"
    );

    sim.tick();
    let shown = |r: economy::Resource| sim.ledgers[0].income[r.index()] / economy::RATE_SCALE;
    let city_gather = Tuning::RON.city_gather;
    assert_eq!(shown(economy::Resource::Food), city_gather[0]);
    assert_eq!(shown(economy::Resource::Timber), city_gather[1]);
    assert_eq!(shown(economy::Resource::Wealth), 0, "no market");
    assert_eq!(shown(economy::Resource::Knowledge), 0, "no university");

    // And the starting goods are there and untouched by the first frame.
    let food = economy::Resource::Food.index();
    assert_eq!(
        sim.ledgers[0].bucket[food],
        Tuning::RON.starting_goods[food]
    );
}

/// The Citizen, as `unitrules.xml` writes it: `2f`, `1f support`,
/// `PROGRESSION 0`, `POP 1`, forty hit points, `JOB_TIME 50`,
/// `JOB_EXTRA_TIME 1/10tsx`, `RESEARCH_PREMIUM_TIME 2`.
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
        combat: combat::Profile::default(),
        times: production::Times {
            job_time: 50,
            // `2` through `String::fraction(s, 0x100)`.
            research_premium_time: 512,
            // `1/10tsx` through the unit loader's own `(1 * 100) / 10`.
            job_extra_time: 10,
        },
        tree: None,
        garrison: garrison::UnitTraits::default(),
        ..UnitType::default()
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

    // A player with no Military tech is capped at twenty-five, and a citizen
    // is one pop.
    assert_eq!(sim.muster[0].cap, 25);
    sim.ledgers[0].bucket[food] = 100_000;
    for _ in 0..25 {
        sim.produce(0, citizen, at).expect("under the cap");
    }
    assert_eq!(sim.muster[0].control, 25);

    let purse = sim.ledgers[0].bucket[food];
    assert_eq!(sim.produce(0, citizen, at), Err(Refused::Population));
    assert_eq!(sim.ledgers[0].bucket[food], purse, "refused, not charged");

    // The Art of War raises the cap, and the same order goes through.
    sim.muster[0].military_level = 1;
    sim.recompute_pop_caps();
    assert_eq!(sim.muster[0].cap, 50);
    sim.produce(0, citizen, at)
        .expect("room once the Military line has one tech in it");
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

#[test]
fn a_queued_citizen_is_paid_for_up_front_and_arrives_on_time() {
    // `docs/PRODUCTION.md` end to end. The price leaves the stockpile the
    // moment the order is given, the counter climbs one frame's worth per
    // frame, and the unit appears when it lands on the target.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);

    let purse = sim.ledgers[0].bucket[food];
    sim.queue_up(hall, citizen).expect("affordable and room");
    assert_eq!(
        sim.ledgers[0].bucket[food],
        purse - 20,
        "charged on queue, not on delivery"
    );
    assert!(sim.units.is_empty());

    // The first of a type is a research job, and a research job skips the
    // ramp entirely — `UNIT_RATE_BASE` is applied *inside* the availability
    // branch, so the `6/5` that stretches every trained unit does not touch
    // this one. Fifty frames of `JOB_TIME`, doubled by
    // `RESEARCH_PREMIUM_TIME`, is a hundred; the counter is read before it is
    // advanced, so it completes on the hundred and first.
    //
    // And completing it sets the bit and delivers nothing: in the original
    // the entry falls through `Build::finished` to `Leader::gain_tech`, which
    // is `BitMask::set` and a return of 1. (An earlier draft of this test
    // expected a unit here; the second reading corrected it — see
    // `docs/PRODUCTION.md`, "Completion".)
    let mut frames = 0;
    while !sim.muster[0].researched[citizen] {
        sim.tick();
        frames += 1;
        assert!(frames < 1000, "the queue should have completed by now");
    }
    assert_eq!(frames, 101);
    assert!(sim.units.is_empty(), "research trains nothing");
    assert_eq!(sim.muster[0].control, 0);
    assert_eq!(sim.muster[0].by_type[citizen], 0);
    assert!(sim.buildings[hall].queue.items.is_empty());

    // The second order is the first *train* job: the ordinary time stretched
    // by `UNIT_RATE_BASE`, with nothing yet owned to ramp against. Sixty
    // frames, observed on the sixty-first.
    sim.queue_up(hall, citizen).unwrap();
    let mut frames = 0;
    while sim.units.is_empty() {
        sim.tick();
        frames += 1;
        assert!(frames < 1000, "the queue should have delivered by now");
    }
    assert_eq!(frames, 61);
    assert_eq!(sim.muster[0].control, 1);

    // The third pays one ramp step for the one already standing.
    sim.queue_up(hall, citizen).unwrap();
    let mut frames = 0;
    while sim.units.len() < 2 {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    assert_eq!(frames, 69);
}

#[test]
fn the_first_of_a_type_is_researched_and_trains_nothing() {
    // D4 of `docs/audit/2026-08-20-production.md`, end to end, with both
    // orders placed up front. The head is the research entry; the one behind
    // it waits, because a barracks advances slot 0 only. On the hundred and
    // first frame the bit flips, the entry leaves, and nothing is born. The
    // second entry is then the head, reads the bit as set, and is a train job
    // from its first frame — sixty frames, landing on the hundred and
    // sixty-second.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);
    sim.queue_up(hall, citizen).unwrap();
    sim.queue_up(hall, citizen).unwrap();

    for _ in 0..100 {
        sim.tick();
    }
    assert!(!sim.muster[0].researched[citizen]);
    assert_eq!(sim.buildings[hall].queue.items.len(), 2);
    assert_eq!(
        sim.buildings[hall].queue.items[1].job_counter, 0,
        "the entry behind a live head does not move"
    );

    sim.tick();
    assert!(sim.muster[0].researched[citizen], "frame 101: the bit");
    assert!(sim.units.is_empty(), "and no unit");
    assert_eq!(sim.muster[0].control, 0);
    assert_eq!(sim.muster[0].queued_by_type[citizen], 1);
    assert_eq!(sim.buildings[hall].queue.items.len(), 1);
    assert_eq!(sim.buildings[hall].queue.items[0].job_counter, 0);

    let mut frames = 101;
    while sim.units.is_empty() {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    assert_eq!(frames, 162);
    assert_eq!(sim.muster[0].control, 1);
    assert_eq!(sim.muster[0].queued_by_type[citizen], 0);
}

#[test]
fn only_the_first_library_fans_out() {
    // D1 of `docs/audit/2026-08-20-production.md`. `do_queue` recurses into
    // slot `i + 1`, bounded by `get_building_cities`, inside its library
    // branch and nowhere else. A barracks with four library cities behind it
    // still advances one entry at a time; the player's first library advances
    // four; a second library's own queue never advances, because orders given
    // to it land on the first one anyway.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    // A second unit type, so the library holds two distinct research jobs —
    // with no technology types in the simulation, a unit type whose bit is
    // clear is what a research entry is.
    let other = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    sim.ledgers[0].bucket[food] = 100_000;
    sim.muster[0].library_cities = 4;
    sim.muster[0].researched[citizen] = true;

    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);
    let library = sim.add_library(0, centre_of(Cell::new(3, 0)), 8);
    let annex = sim.add_library(0, centre_of(Cell::new(4, 0)), 8);

    sim.queue_up(hall, citizen).unwrap();
    sim.queue_up(hall, citizen).unwrap();
    sim.queue_up(hall, citizen).unwrap();
    // One order at each library: both land on the first.
    sim.queue_up(library, citizen).unwrap();
    sim.queue_up(annex, other).unwrap();
    assert_eq!(sim.buildings[library].queue.items.len(), 2);
    assert!(sim.buildings[annex].queue.items.is_empty());

    for _ in 0..10 {
        sim.tick();
    }
    let counters = |sim: &Sim, at: usize| -> Vec<i32> {
        sim.buildings[at]
            .queue
            .items
            .iter()
            .map(|i| i.job_counter)
            .collect()
    };
    // The barracks: head only. (An earlier draft advanced all three.)
    assert_eq!(counters(&sim, hall), [1000, 0, 0]);
    // The first library: both slots, in step.
    assert_eq!(counters(&sim, library), [1000, 1000]);

    // One library city, and the same library advances one slot.
    sim.muster[0].library_cities = 1;
    for _ in 0..10 {
        sim.tick();
    }
    assert_eq!(counters(&sim, library), [2000, 1000]);
}

#[test]
fn a_stuck_head_lets_a_research_entry_behind_it_advance() {
    // D2 of `docs/audit/2026-08-20-production.md`. When slot 0 is done and
    // `finished` refuses it, `do_queue` asks `get_next_non_unit` for the first
    // research entry behind the head and advances that one instead — the
    // population cap blocks every train job and only train jobs. A second
    // train entry behind the head never moves; the research entry runs to
    // completion while the head sits at full progress.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let other = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);
    sim.ledgers[0].bucket[food] = 100_000;

    // At the Ancient cap of twenty-five, with the citizen already researched
    // and the other type not.
    sim.muster[0].control = 25;
    sim.muster[0].researched[citizen] = true;
    sim.queue_up(hall, citizen).unwrap();
    sim.queue_up(hall, citizen).unwrap();
    sim.queue_up(hall, other).unwrap();

    // Sixty frames to the head's target; nothing behind it moves yet.
    for _ in 0..60 {
        sim.tick();
    }
    assert_eq!(sim.buildings[hall].queue.items[0].job_counter, 6000);
    assert_eq!(sim.buildings[hall].queue.items[1].job_counter, 0);
    assert_eq!(sim.buildings[hall].queue.items[2].job_counter, 0);

    // Frame 61: the head is done, offered, refused — and the research entry
    // in slot 2 takes the frame. The train entry in slot 1 does not.
    sim.tick();
    assert!(sim.units.is_empty());
    assert_eq!(sim.buildings[hall].queue.items[0].job_counter, 6000);
    assert_eq!(sim.buildings[hall].queue.items[1].job_counter, 0);
    assert_eq!(sim.buildings[hall].queue.items[2].job_counter, 100);

    // A hundred frames of research at one frame per frame, observed on the
    // hundred and first — all while the head is stuck.
    for _ in 0..100 {
        sim.tick();
    }
    assert!(sim.muster[0].researched[other], "the research got through");
    assert!(sim.units.is_empty(), "and the cap still holds");
    assert_eq!(sim.buildings[hall].queue.items.len(), 2);
    assert_eq!(sim.buildings[hall].queue.items[1].job_counter, 0);

    // Room appears and the head is handed over at once.
    sim.muster[0].military_level = 1;
    sim.recompute_pop_caps();
    sim.tick();
    assert_eq!(sim.units.len(), 1);
}

#[test]
fn the_population_cap_stalls_a_queue_at_full_progress() {
    // The finding of `docs/PRODUCTION.md`: the cap does not stop the clock,
    // it stops the handover. The item sits at a hundred percent, paid for,
    // until room appears — and then comes out on the next frame.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);
    sim.ledgers[0].bucket[food] = 100_000;

    // Fill the Ancient cap of twenty-five, then order one more anyway.
    sim.muster[0].control = 25;
    sim.muster[0].researched[citizen] = true;
    sim.queue_up(hall, citizen)
        .expect("the cap does not gate a queue");

    for _ in 0..500 {
        sim.tick();
    }
    assert!(sim.units.is_empty(), "no room, so no unit");
    let counter = sim.buildings[hall].queue.items[0].job_counter;
    assert_eq!(counter, sim.queue_target(hall, 0), "but done");

    // A Military tech raises the cap and the stalled item is handed over at
    // once.
    sim.muster[0].military_level = 1;
    sim.recompute_pop_caps();
    sim.tick();
    assert_eq!(sim.units.len(), 1);
    assert!(sim.buildings[hall].queue.items.is_empty());
}

#[test]
fn cancelling_gives_back_exactly_what_that_entry_was_charged() {
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let food = economy::Resource::Food.index();
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);

    let purse = sim.ledgers[0].bucket[food];
    sim.queue_up(hall, citizen).unwrap();
    // The second order pays a ramp step, because the price counts what is
    // ordered and not only what exists.
    sim.queue_up(hall, citizen).unwrap();
    assert_eq!(sim.ledgers[0].bucket[food], purse - 41);

    // Cancelling slot 0 removes the *last* of the run of two, so the one in
    // progress keeps its progress — and the refund is the twenty-one that
    // entry was charged, not the twenty the first one was.
    //
    // The comparison is a delta across the cancel rather than against the
    // opening purse, because the ten frames in between are ten frames of
    // income: `docs/ECONOMY.md` is running too.
    for _ in 0..10 {
        sim.tick();
    }
    let progress = sim.buildings[hall].queue.items[0].job_counter;
    assert!(progress > 0);
    let before = sim.ledgers[0].bucket[food];
    let back = sim.cancel(hall, 0).expect("something to cancel");
    assert_eq!(back.job_counter, 0, "the untouched one is the one removed");
    assert_eq!(sim.ledgers[0].bucket[food] - before, 21);
    assert_eq!(sim.buildings[hall].queue.items[0].job_counter, progress);
}

#[test]
fn a_trained_citizen_walks_and_bleeds_like_any_other() {
    // The same closing loop as the instant path, one mechanic longer: income
    // pays for an order, the order takes time, and what comes out of it walks
    // into a hostile border and starts dying there. Seven mechanics, one unit.
    let mut sim = skirmish(4);
    let citizen = sim.add_unit_type(citizen_type());
    let start = Cell::new(2, 0);
    let hall = sim.add_building(0, centre_of(start), 8);
    // Two orders: the first of a type is a research job and trains nothing
    // (`docs/PRODUCTION.md`, "Completion"), so the unit this test wants is
    // the second entry's.
    sim.queue_up(hall, citizen).expect("affordable");
    sim.queue_up(hall, citizen).expect("affordable");

    let mut frames = 0;
    while sim.units.is_empty() {
        sim.tick();
        frames += 1;
        assert!(frames < 1000, "research, then a train job, then a unit");
    }
    let unit = sim.units.len() - 1;
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
    assert!(ticks > 0, "a trained unit should bleed like a bought one");
    assert!(sim.units[unit].health < 40);
}

/// **An age snaps every one of the leader's figures where it stands** —
/// `Leader::gain_tech@006dcb60:2366`'s `is_age_type` arm
/// (`docs/TECH.md`, "An age snaps every figure").
///
/// The visible half is the facing: a unit **mid-turn** takes its own
/// heading outright rather than the frame's worth of turn rate, which is
/// worth a frame to it. A unit already aligned notices nothing, which is
/// why the event hid for as long as it did — and a plain tech does not
/// take the arm at all, which is the half that can fail.
#[test]
fn an_age_snaps_the_leader_s_figures_and_a_plain_tech_does_not() {
    use crate::tech::{TechTree, TypeDef};

    let mut tree = TechTree::new();
    let classical = tree.add(TypeDef::age("Classical Age", 0));
    let library = tree.add(TypeDef::building("Library"));
    let writing = tree.add(TypeDef::plain("Writing", 0).at(library));

    let mut sim = skirmish(4);
    sim.set_tech_tree(tree);
    sim.start_techs(0);
    sim.start_techs(1);

    // One unit each, both owed a quarter turn: facing north, heading east.
    let mut turners = Vec::new();
    for who in 0..2u8 {
        let u = sim.add_unit(Unit::new(
            who,
            i16::from(who),
            centre_of(Cell::new(2 + i32::from(who), 0)),
            100,
        ));
        make_mobile(&mut sim, u, movement::Angle::NORTH);
        sim.units[u].movement.heading = movement::Angle::EAST;
        turners.push(u);
    }
    let owed = |s: &Sim, u: usize| s.units[u].movement.facing != s.units[u].movement.heading;
    assert!(owed(&sim, turners[0]) && owed(&sim, turners[1]));

    // A plain tech leaves both turns owed — the gate is the type, not the
    // gain.
    sim.gain_tech(0, writing);
    assert!(
        owed(&sim, turners[0]) && owed(&sim, turners[1]),
        "a plain tech takes no unit through `Unit::set_new_location`"
    );

    // The age settles player 0's and leaves player 1's alone: the loop is
    // over one leader's own objects.
    sim.gain_tech(0, classical);
    assert_eq!(sim.tech[0].ages, 1);
    assert!(
        !owed(&sim, turners[0]),
        "the age puts player 0's figure on its own heading"
    );
    assert!(owed(&sim, turners[1]), "and touches nothing of player 1's");
    // The body goes with it — `Guy::set_new_location(guy 0, des, 1)`.
    assert_eq!(
        sim.units[turners[0]].movement.body.pos,
        sim.units[turners[0]].pos
    );
}

/// **Gaining a unit type converts the units of the line it replaces**
/// (`docs/TECH.md` §7's object half) — the pass that turns run53's three
/// Bowmen into Archers on frame 6736.
///
/// Three things at once: the `from` match and the `jump` chain both
/// convert; every figure of a converted unit pays a fresh
/// `Guy::init_real` draw and comes out with a zeroed clock; and the
/// damage carries across the swap onto the new type's hits.
#[test]
fn a_gained_unit_type_converts_the_line_below_it_and_carries_the_damage() {
    use crate::tech::{TechTree, TypeDef, UnitTraits};

    let free = UnitTraits {
        free: true,
        ..UnitTraits::default()
    };
    let jumpable = UnitTraits {
        jumpable: true,
        ..UnitTraits::default()
    };
    let mut tree = TechTree::new();
    let classical = tree.add(TypeDef::age("Classical Age", 0));
    let barracks = tree.add(TypeDef::building("Barracks"));
    let hoplites_t = tree.add(TypeDef::unit("Hoplites", free).at(barracks));
    let phalanx_t = tree.add(
        TypeDef::unit("Phalanx", jumpable)
            .at(barracks)
            .from(hoplites_t)
            .needs(0, classical),
    );
    tree.types[hoplites_t].jump = Some(phalanx_t);

    let mut sim = skirmish(4);
    sim.set_tech_tree(tree);
    sim.start_techs(0);
    let hoplites = sim.add_unit_type(UnitType {
        tree: Some(hoplites_t),
        hits: 40,
        ..citizen_type()
    });
    let phalanx = sim.add_unit_type(UnitType {
        tree: Some(phalanx_t),
        hits: 60,
        ..citizen_type()
    });
    let a = sim.init_unit(0, hoplites, centre_of(Cell::new(3, 3)));
    let b = sim.init_unit(0, hoplites, centre_of(Cell::new(4, 3)));
    // One of them is hurt; the other is not.
    sim.units[b].health -= 9;
    let before = sim.muster[0].by_type[hoplites];
    assert_eq!(sim.units[a].ty, Some(hoplites));

    sim.trace_phases = true;
    sim.phase_marks.clear();
    sim.gain_tech(0, phalanx_t);

    assert_eq!(sim.units[a].ty, Some(phalanx), "the `from` match converts");
    assert_eq!(sim.units[b].ty, Some(phalanx));
    assert_eq!(
        sim.units[a].max_health, 60,
        "the new type's hits, not the old"
    );
    assert_eq!(sim.units[a].health, 60, "undamaged stays undamaged");
    assert_eq!(sim.units[b].health, 51, "and nine points of damage carry");
    // The counters moved from one type to the other.
    assert_eq!(sim.muster[0].by_type[hoplites], before - 2);
    assert_eq!(sim.muster[0].by_type[phalanx], 2);
    // One `Guy::init_real` draw per figure of each converted unit, and the
    // clock is back to zero so the next `inc_time` wraps.
    let figures: usize = [a, b].iter().map(|&u| sim.units[u].guys.len()).sum();
    assert!(figures >= 2);
    assert_eq!(
        sim.phase_marks
            .iter()
            .filter(|(l, _)| l == crate::anim::SITE_INIT_REAL)
            .count(),
        figures,
        "one variant roll per figure, and nothing else drew"
    );
    assert_eq!(
        sim.phase_marks.len(),
        figures,
        "the conversion pass spends no other draw"
    );
    assert!(sim.units[a].guys.iter().all(|g| g.end_time == 0));
}

#[test]
fn the_tree_gates_the_queue_and_research_cascades_through_it() {
    // `docs/TECH.md`, end to end: a tree with Classical (two library techs'
    // quota away), a Barracks, and the Hoplites → Phalanx line; the unit
    // types joined to it through `UnitType::tree`. The free Hoplites are a
    // train job from frame 0; the Phalanx is refused before the price until
    // Classical arrives, then researched, and its completion goes through
    // `gain_tech` — which owns and obsoletes the Hoplites in one move — while
    // the Military epoch on the way moved the population cap.
    use crate::tech::{Gained, Kind, Line, TechTree, TypeDef, UnitTraits};

    let free = UnitTraits {
        free: true,
        ..UnitTraits::default()
    };
    let jumpable = UnitTraits {
        jumpable: true,
        ..UnitTraits::default()
    };
    let mut tree = TechTree::new();
    let classical = tree.add(TypeDef::age("Classical Age", 0));
    let written_word = tree.add(TypeDef::epoch("Written Word", Line::Science, 0));
    let art_of_war = tree.add(TypeDef::epoch("The Art of War", Line::Military, 0));
    let barracks = tree.add(TypeDef::building("Barracks"));
    let hoplites_t = tree.add(TypeDef::unit("Hoplites", free).at(barracks));
    let phalanx_t = tree.add(
        TypeDef::unit("Phalanx", jumpable)
            .at(barracks)
            .from(hoplites_t)
            .needs(0, classical),
    );
    tree.types[hoplites_t].jump = Some(phalanx_t);
    assert!(matches!(tree.kind(barracks), Kind::Building { .. }));

    let mut sim = skirmish(4);
    sim.set_tech_tree(tree);
    sim.start_techs(0);
    let hoplites = sim.add_unit_type(UnitType {
        tree: Some(hoplites_t),
        ..citizen_type()
    });
    let phalanx = sim.add_unit_type(UnitType {
        tree: Some(phalanx_t),
        ..citizen_type()
    });
    let hall = sim.add_building(0, centre_of(Cell::new(2, 0)), 8);

    // The free unit is owned from the start; the upgrade is not for sale.
    assert!(sim.muster[0].researched[hoplites]);
    assert!(!sim.muster[0].researched[phalanx]);
    assert_eq!(
        sim.queue_up(hall, phalanx),
        Err(production::QueueFail::CantTrain)
    );
    let cap_before = sim.muster[0].cap;

    // Two library techs, then the age. The Military one moves the cap.
    sim.gain_tech(0, written_word);
    let events = sim.gain_tech(0, art_of_war);
    assert!(events.contains(&Gained::MilitaryEpoch));
    assert_eq!(sim.muster[0].military_level, 1);
    assert!(sim.muster[0].cap > cap_before, "POP_CAP[1] > POP_CAP[0]");
    assert_eq!(sim.type_avail(0, phalanx_t), tech::NOT_AVAILABLE);
    let events = sim.gain_tech(0, classical);
    assert_eq!(events, vec![Gained::Type(classical)]);
    assert_eq!(sim.tech[0].ages, 1);
    assert_eq!(sim.type_avail(0, phalanx_t), tech::RESEARCHABLE);

    // Now it queues, and completes as research: no unit, the bit, and the
    // Hoplites owned-and-obsolete behind it.
    sim.queue_up(hall, phalanx).expect("researchable");
    assert_eq!(sim.tech[0].queued[phalanx_t], 1);
    let mut frames = 0;
    while !sim.muster[0].researched[phalanx] {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    assert!(sim.units.is_empty());
    assert_eq!(sim.tech[0].queued[phalanx_t], 0);
    assert!(sim.tech[0].tech[phalanx_t]);
    assert!(sim.tech[0].obs[hoplites_t]);
    assert_eq!(sim.type_avail(0, phalanx_t), tech::AVAILABLE);
    assert_eq!(
        sim.type_avail(0, hoplites_t),
        tech::NOT_AVAILABLE,
        "obsolete"
    );
    assert_eq!(
        sim.queue_up(hall, hoplites),
        Err(production::QueueFail::CantTrain)
    );
    // The second Phalanx is a train job.
    sim.queue_up(hall, phalanx).expect("owned");
    let mut frames = 0;
    while sim.units.is_empty() {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    assert_eq!(sim.muster[0].by_type[phalanx], 1);
}

// ---------------------------------------------------------------------------
// Combat — `docs/COMBAT.md`
// ---------------------------------------------------------------------------

use crate::combat::{self, Obj, Stance, mask};
use crate::world::vector_dist;

/// An arena: two players at war on open land, nobody's territory.
fn arena() -> Sim {
    let mut world = World::new(16, 16);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    sim.declare_war(0, 1);
    sim
}

/// A melee type: 12 attack, 3 armour, 90 hits, reload 20 frames.
fn hoplite_type() -> UnitType {
    UnitType {
        hits: 90,
        combat: combat::Profile {
            attack: 120,
            armor: 3,
            recharge: 20,
            obj_masks: mask::FOOT | mask::HEAVY_INF | mask::MELEE,
            uber_size: 1,
            ammo_per_att: 1,
            block_radius: 24,
            big_radius: 48,
            target_size: 48,
            guy_radius: 24,
            combat_role: true,
            cost: 60,
            to_hit: -1,
            ..combat::Profile::default()
        },
        ..citizen_type()
    }
}

/// A ranged type: 6 attack, 1 armour, 60 hits, range 5, reload 30, 80 % to
/// hit less 1 % a tile, arrows at 200 a frame.
fn archer_type() -> UnitType {
    UnitType {
        hits: 60,
        combat: combat::Profile {
            attack: 60,
            armor: 1,
            recharge: 30,
            max_range: 5,
            to_hit: 80,
            attenuate: 1,
            proj_speed: 200,
            obj_masks: mask::FOOT | mask::FOOT_ARCHER | mask::ARCHERY,
            uber_size: 1,
            ammo_per_att: 1,
            block_radius: 24,
            big_radius: 48,
            target_size: 48,
            guy_radius: 24,
            combat_role: true,
            cost: 70,
            ..combat::Profile::default()
        },
        ..citizen_type()
    }
}

/// Places a unit of a type for a player at a position, facing a way.
fn combatant(sim: &mut Sim, owner: Player, ty: usize, at: Pos, facing: movement::Angle) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.unit_types[ty].hits;
    let mut u = Unit::new(owner, index, at, hits);
    u.ty = Some(ty);
    let i = sim.add_unit(u);
    make_mobile(sim, i, facing);
    i
}

fn hits_on(sim: &Sim, target: usize) -> Vec<&combat::Hit> {
    sim.hits
        .iter()
        .filter(|h| h.target == Obj::Unit(target))
        .collect()
}

#[test]
fn a_melee_duel_lands_on_the_attack_frame_and_every_recharge_after() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    // A stands just south of B, facing north; B faces south into it, so the
    // attack is head-on and no flank applies.
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 900),
        movement::Angle::SOUTH,
    );
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    sim.set_stance(b, Stance::HoldFire);
    sim.order_attack(a, Obj::Unit(b));
    run(&mut sim, 61);
    let frames: Vec<i64> = hits_on(&sim, b).iter().map(|h| h.frame).collect();
    assert_eq!(
        frames,
        vec![0, 20, 40, 60],
        "one attack every RECHARGE frames"
    );
    // (120 × 100 / 100 + 5) / 10 − 3 = 9, delivered whole.
    for h in hits_on(&sim, b) {
        assert_eq!(h.damage, 9);
        assert_eq!(h.dealt, combat::Sixteenths { whole: 9, frac: 0 });
        assert!(!h.killed);
    }
    assert_eq!(sim.units[b].health, 90 - 4 * 9);
    // Ten hits kill; the tenth is at frame 180.
    run(&mut sim, 120);
    assert!(!sim.units[b].alive());
    let last = hits_on(&sim, b).last().copied().unwrap();
    assert_eq!(last.frame, 180);
    assert!(last.killed);
    // The killer's order is dropped with the target.
    assert_eq!(sim.units[a].combat.target, None);
}

/// **The ATTACK action under a move is a *ranged* attacker's**
/// (`docs/ORDERS.md` §4.4, `docs/COMBAT.md` §25, item 405).
///
/// `do_move@005f7b30:212` gates the whole action block on
/// `ptype->max_range != 0` — `SubObjectData +0x18`, `ObjectTypeData
/// +0x1fc`, both by the type record — so a **melee** type walks the leg it
/// was given even once its target is inside its reach, and a **ranged**
/// one drops the chase the moment the shot is on.
///
/// This crate asked every type, and the golden record paid five frames for
/// it: `1/8` at `(1332, 8121)` is `attack_dist` 246 from `0/7`, exactly the
/// HOPLITES `0xf6`, so its chase died 171 short of the `(1176, 8088)` the
/// original's dump walks it to.
///
/// Made to fail first in the melee direction: with the gate removed the
/// melee half says "the melee chase was dropped when the target came into
/// reach".
#[test]
fn only_a_ranged_attacker_drops_its_chase_when_the_target_comes_into_reach() {
    // The pair the duel test uses, a hundred units apart and in melee
    // reach — with an ATTACK order beneath a **transit** move, which is
    // the stack `get_action` reads through (`docs/ORDERS.md` §4.3).
    let chase = |ty: fn() -> UnitType| {
        let mut sim = arena();
        let t = sim.add_unit_type(ty());
        let hop = sim.add_unit_type(hoplite_type());
        let b = combatant(
            &mut sim,
            1,
            hop,
            Pos::new(1000, 900),
            movement::Angle::SOUTH,
        );
        let a = combatant(&mut sim, 0, t, Pos::new(1000, 1000), movement::Angle::NORTH);
        sim.set_stance(b, Stance::HoldFire);
        sim.add_attack_order(a, Obj::Unit(b), crate::orders::QueuePos::New, false, false);
        sim.add_move_order(
            a,
            Pos::new(1000, 1600),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        assert!(
            sim.is_in_range(Obj::Unit(a), Obj::Unit(b)),
            "the premise: the target is already inside the attacker's reach"
        );
        assert!(
            sim.current_order(a)
                .is_some_and(crate::orders::Order::is_move),
            "the chase is the order on top"
        );
        run(&mut sim, 1);
        sim.current_order(a)
            .is_some_and(crate::orders::Order::is_move)
    };
    assert!(
        chase(hoplite_type),
        "the melee chase was dropped when the target came into reach"
    );
    assert!(
        !chase(archer_type),
        "the ranged chase outlived the shot it was walking to take"
    );
}

#[test]
fn attacking_from_behind_is_a_flank_and_from_the_side_a_bigger_one() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    // B faces north; A behind it (south), attacking northward: rear.
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 900),
        movement::Angle::NORTH,
    );
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    sim.set_stance(b, Stance::HoldFire);
    sim.order_attack(a, Obj::Unit(b));
    run(&mut sim, 1);
    // 120 × 1.5 = 180 → (180 + 5) / 10 − 3 = 15.
    assert_eq!(hits_on(&sim, b)[0].damage, 15);
    // From the side: ×2 → 24 − 3 = 21.
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let b = combatant(&mut sim, 1, hop, Pos::new(1000, 900), movement::Angle::EAST);
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    sim.set_stance(b, Stance::HoldFire);
    sim.order_attack(a, Obj::Unit(b));
    run(&mut sim, 1);
    assert_eq!(hits_on(&sim, b)[0].damage, 21);
}

#[test]
fn the_combat_table_multiplies_the_attack_before_armour() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let arc = sim.add_unit_type(archer_type());
    // Heavy infantry against foot archers: the shipped table says 86 %.
    sim.table.set(hop, arc, 86);
    let b = combatant(
        &mut sim,
        1,
        arc,
        Pos::new(1000, 900),
        movement::Angle::SOUTH,
    );
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    sim.set_stance(b, Stance::HoldFire);
    sim.order_attack(a, Obj::Unit(b));
    run(&mut sim, 1);
    // 120 × 86 / 100 = 103 → (103 + 5) / 10 − 1 = 9.
    assert_eq!(hits_on(&sim, b)[0].damage, 9);
}

#[test]
fn a_shot_flies_for_its_distance_and_lands_where_the_rng_put_it() {
    let mut sim = arena();
    let arc = sim.add_unit_type(archer_type());
    let hop = sim.add_unit_type(hoplite_type());
    // Three tiles apart on the x axis.
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::WEST,
    );
    let a = combatant(
        &mut sim,
        0,
        arc,
        Pos::new(1000 - 3 * 192, 1000),
        movement::Angle::EAST,
    );
    sim.set_stance(b, Stance::HoldFire);
    sim.rng = combat::Rng::new(7);
    // The frame's own draws (the market on frame 0, the birds' sampling)
    // come from a control that fires nothing; the shot's are on top.
    let mut control = arena();
    control.rng = combat::Rng::new(7);
    sim.order_attack(a, Obj::Unit(b));
    sim.tick();
    control.tick();
    // One projectile, launched this frame.
    assert_eq!(sim.projectiles.len(), 1);
    let p = sim.projectiles[0];
    assert_eq!(p.shooter, Obj::Unit(a));
    assert_eq!(p.target, Some(Obj::Unit(b)));
    // Accuracy: 80 − attenuate × (attack_dist / 192). attack_dist: 576 − 48
    // − 48 = 480 → two whole tiles → 78. Scatter: 96 × 100 / ((100 − 78) /
    // 5 + 78) = 117.
    assert_eq!(p.accuracy, 78);
    assert!((p.landing.x - 1000).abs() <= 60 && (p.landing.y - 1000).abs() <= 60);
    // Flight time: sqrt(dx² + dy²) / 200, truncated.
    let dx = i64::from(p.landing.x - p.launch.x);
    let dy = i64::from(p.landing.y - p.launch.y);
    assert_eq!(p.total_time, combat::flight_time(dx * dx + dy * dy, 200));
    assert!(
        p.total_time == 2 || p.total_time == 3,
        "about three tiles at 200 a frame"
    );
    // It lands on the frame its time is up and is gone: `cur_time` is
    // counted up before the test, so a time of 2 lands on the second frame
    // after the launch frame.
    for _ in 1..(p.total_time - 1) {
        sim.tick();
        control.tick();
        assert_eq!(sim.projectiles.len(), 1);
    }
    sim.tick();
    control.tick();
    assert_eq!(sim.projectiles.len(), 0);
    // Whether it hit is whether the landing point was within target_size.
    let d = vector_dist(p.landing.x - 1000, p.landing.y - 1000);
    let hit = hits_on(&sim, b);
    if d <= 48 {
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].frame, i64::from(p.total_time) - 1);
        // 60 → (65) / 10 − 3 = 3.
        assert_eq!(hit[0].damage, 3);
    } else {
        assert!(hit.is_empty(), "a miss lands on nothing here");
    }
    // Two RNG draws were taken for the scatter — and two more for where a
    // miss punctured the ground, none else beyond the frames' own.
    let mut r = control.rng;
    r.roll();
    r.roll();
    if d > 48 {
        r.roll();
        r.roll();
    }
    assert_eq!(sim.rng, r);
}

#[test]
fn the_same_seed_gives_the_same_fight() {
    let fight = |seed: u32| {
        let mut sim = arena();
        let arc = sim.add_unit_type(archer_type());
        let hop = sim.add_unit_type(hoplite_type());
        sim.rng = combat::Rng::new(seed);
        for k in 0..4 {
            let at = Pos::new(800 + k * 60, 1000);
            let i = combatant(&mut sim, 1, hop, at, movement::Angle::NORTH);
            sim.set_stance(i, Stance::HoldFire);
        }
        for k in 0..4 {
            let at = Pos::new(800 + k * 60, 1000 + 4 * 192);
            let i = combatant(&mut sim, 0, arc, at, movement::Angle::NORTH);
            sim.order_attack(i, Obj::Unit(k as usize));
        }
        run(&mut sim, 300);
        (sim.hits.clone(), sim.rng)
    };
    assert_eq!(fight(11), fight(11));
    let (h1, _) = fight(11);
    let (h2, _) = fight(12);
    assert!(!h1.is_empty());
    assert_ne!(h1, h2, "a different seed scatters differently");
}

#[test]
fn focus_fire_from_a_second_squad_is_overkill() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::SOUTH,
    );
    sim.set_stance(b, Stance::HoldFire);
    // Two archers of different squads, two tiles away, both on B, with
    // accuracy high enough that the scatter radius is under two and no draw
    // is taken.
    let mut exact = archer_type();
    exact.combat.to_hit = 400;
    exact.combat.attenuate = 0;
    let arc2 = sim.add_unit_type(exact);
    let a1 = combatant(
        &mut sim,
        0,
        arc2,
        Pos::new(1000, 1000 + 2 * 192),
        movement::Angle::NORTH,
    );
    let a2 = combatant(
        &mut sim,
        0,
        arc2,
        Pos::new(1000 + 60, 1000 + 2 * 192),
        movement::Angle::NORTH,
    );
    // Not on frame zero: a `damage_frame` of zero reads as "no record",
    // in the original as here.
    run(&mut sim, 1);
    sim.order_attack(a1, Obj::Unit(b));
    sim.order_attack(a2, Obj::Unit(b));
    run(&mut sim, 6);
    let h = hits_on(&sim, b);
    assert_eq!(h.len(), 2, "both arrows landed the same frame");
    // The first to land owns the window: full 3; the second, a different
    // captain inside 30 frames: 3 × 85 >> 8 = 0 — and then at least one.
    assert_eq!(h[0].damage, 3);
    assert_eq!(h[1].damage, 1);
    assert_eq!(h[0].attacker, Obj::Unit(a1));
    assert_eq!(h[1].attacker, Obj::Unit(a2));
}

#[test]
fn an_idle_unit_finds_a_target_on_its_cadence_and_a_hold_fire_one_never() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000 + 3 * 192, 1000),
        movement::Angle::WEST,
    );
    sim.set_stance(b, Stance::HoldFire);
    assert_eq!(sim.units[a].combat.target, None);
    // `think` runs when `(index + frame) & 0x1f == 0`: for index 0, frame 0.
    sim.tick();
    assert_eq!(sim.units[a].combat.target, Some(Obj::Unit(b)));
    assert!(!sim.units[a].combat.mandatory);
    assert_eq!(sim.units[b].combat.targeted, 1);
    // It walks over and fights; B holds fire throughout.
    run(&mut sim, 200);
    assert!(!hits_on(&sim, b).is_empty());
    assert!(hits_on(&sim, a).is_empty());
    assert!(
        sim.units[a].movement.dest.is_none(),
        "it stands still to fight"
    );
}

#[test]
fn being_hit_makes_an_idle_combat_unit_fight_back() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 900),
        movement::Angle::SOUTH,
    );
    let a = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::NORTH,
    );
    sim.order_attack(a, Obj::Unit(b));
    sim.units[b].combat.stance = Stance::Defensive;
    run(&mut sim, 1);
    assert_eq!(
        sim.units[b].combat.target,
        Some(Obj::Unit(a)),
        "retaliation"
    );
    run(&mut sim, 25);
    assert!(!hits_on(&sim, a).is_empty());
}

#[test]
fn a_tower_picks_a_target_and_reloads_by_its_arrows() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let tower = sim.add_building(0, Pos::new(1000, 1000), 0);
    sim.buildings[tower].combat = Some(combat::Profile {
        attack: 80,
        armor: 2,
        recharge: 60,
        max_range: 6,
        to_hit: 400,
        attenuate: 0,
        proj_speed: 200,
        ammo_per_att: 1,
        base_arrows: 1,
        most_shots: 4,
        x_size: 1,
        y_size: 1,
        big_radius: 96,
        ..combat::Profile::default()
    });
    sim.buildings[tower].hits = 400;
    sim.buildings[tower].health = 400;
    let b = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000 + 3 * 192, 1000),
        movement::Angle::WEST,
    );
    sim.set_stance(b, Stance::HoldFire);
    // The building thinks on `(frame + o) & 0x1f == 0`, and `o` is 2000 —
    // the first of player 0's band, and 16 mod 32 — so the first think is
    // frame **16**, not frame 0 (item 233; `docs/CITIES.md` §12.1). This
    // test wrote the rule down correctly against code that keyed on the
    // handle, and counted the frames the code's way.
    assert_eq!(sim.buildings[tower].index, 2000);
    run(&mut sim, 17);
    assert_eq!(sim.buildings[tower].target, Some(Obj::Unit(b)));
    assert_eq!(sim.projectiles.len(), 1);
    assert_eq!(
        sim.buildings[tower].recharging, 60,
        "one arrow: the full reload"
    );
    // Two archers' worth of garrison (12 tenths each = 24 → 24 / 8 = 3
    // arrows): base 1 + 3 = 4, reload 15.
    sim.buildings[tower].garrison_attack = 24;
    run(&mut sim, 61);
    assert_eq!(sim.buildings[tower].recharging, 15);
    // The arrows hurt: a hit is (80 + 5) / 10 − 3 = 5 whole.
    run(&mut sim, 30);
    let h = hits_on(&sim, b);
    assert!(!h.is_empty());
    assert!(
        h.iter()
            .all(|h| h.damage == 5 && h.attacker == Obj::Building(tower))
    );
}

#[test]
fn splash_hurts_the_neighbours_as_a_fringe_and_never_the_shooters_side() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let mut cat = archer_type();
    cat.combat.attack = 300;
    cat.combat.splash_area = 2;
    cat.combat.splash_percent = 100;
    cat.combat.to_hit = 400;
    cat.combat.attenuate = 0;
    cat.combat.obj_masks = mask::SIEGE | mask::BOMBARD;
    cat.combat.max_range = 8;
    let catapult = sim.add_unit_type(cat);
    // A line of three enemy hoplites — the second inside the splash fringe
    // (past the `0xc0 + guy_radius` the count starts falling at), the third
    // beyond it — and a friendly one standing among them.
    let t0 = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::SOUTH,
    );
    let t1 = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000 + 312, 1000),
        movement::Angle::SOUTH,
    );
    let t2 = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000 + 800, 1000),
        movement::Angle::SOUTH,
    );
    let friend = combatant(
        &mut sim,
        0,
        hop,
        Pos::new(1000 - 64, 1000),
        movement::Angle::SOUTH,
    );
    for &t in &[t0, t1, t2, friend] {
        sim.set_stance(t, Stance::HoldFire);
    }
    let c = combatant(
        &mut sim,
        0,
        catapult,
        Pos::new(1000, 1000 + 4 * 192),
        movement::Angle::NORTH,
    );
    sim.order_attack(c, Obj::Unit(t0));
    run(&mut sim, 12);
    let on = |u: usize| hits_on(&sim, u);
    assert_eq!(on(t0).len(), 1, "the target, at full count");
    assert!(!on(t0)[0].splash);
    assert_eq!(on(t1).len(), 1, "the neighbour, as a fringe");
    assert!(on(t1)[0].splash);
    // 96 units into the fringe of a two-tile splash: count 256 − 64 = 192,
    // three quarters of the hit.
    assert_eq!(on(t1)[0].damage, on(t0)[0].damage);
    assert_eq!(on(t1)[0].dealt.whole, (on(t0)[0].damage * 192) >> 8);
    assert!(on(t2).is_empty(), "four tiles off is beyond the splash");
    assert!(on(friend).is_empty(), "splash skips the shooter's own side");
}

#[test]
fn siege_fires_at_the_ground_under_a_unit_and_hits_whoever_stands_there() {
    let mut sim = arena();
    let hop = sim.add_unit_type(hoplite_type());
    let mut cat = archer_type();
    cat.combat.attack = 300;
    cat.combat.splash_area = 2;
    cat.combat.splash_percent = 100;
    cat.combat.to_hit = 400;
    cat.combat.attenuate = 0;
    cat.combat.obj_masks = mask::SIEGE | mask::BOMBARD;
    cat.combat.siege = true;
    cat.combat.packs = true;
    cat.combat.max_range = 8;
    let catapult = sim.add_unit_type(cat);
    let t0 = combatant(
        &mut sim,
        1,
        hop,
        Pos::new(1000, 1000),
        movement::Angle::SOUTH,
    );
    sim.set_stance(t0, Stance::HoldFire);
    let c = combatant(
        &mut sim,
        0,
        catapult,
        Pos::new(1000, 1000 + 4 * 192),
        movement::Angle::NORTH,
    );
    sim.order_attack(c, Obj::Unit(t0));
    sim.tick();
    // The shot has no target: it is aimed at the ground where the unit stood.
    assert_eq!(sim.projectiles.len(), 1);
    assert_eq!(sim.projectiles[0].target, None);
    // A siege shot's flight time is its range, not its distance: 8 × 192 /
    // 200 = 7.
    assert_eq!(sim.projectiles[0].total_time, 7);
    run(&mut sim, 8);
    // It found the unit standing there.
    assert_eq!(hits_on(&sim, t0).len(), 1);
}

/// **A crew guy walks a body of its own**, end to end through `Sim::tick`
/// — the mechanic item 128 is, on a scenario built here rather than
/// borrowed from a capture.
///
/// The unit gets two figures, the second with a scout dog's own track
/// offset, and is sent due east. While it walks the crew keeps up: its
/// destination is rewritten every frame from guy 0's new position, and the
/// destination moves by the unit's step where the crew may take eleven
/// eighths of one, so it lands on it every frame.
///
/// Then the unit stands and **turns**, which is the other writer: guy 0's
/// `Guy::move` reaches `turn_towards → do_turn → Guy::set_angle`, and that
/// rotates the crew's destination about a stationary leader. A half turn
/// throws it the whole width of the offset, and the crew walks there on
/// its own over several frames while the unit does not move at all.
///
/// The two figures part in `stopped` as they do it, which is what decides
/// whether the *next* frame's idle request draws.
#[test]
fn a_crew_guy_walks_on_after_its_unit_has_arrived() {
    let mut sim = skirmish(0);
    let start = Pos::new(4000, 400);
    let mut u = Unit::new(0, 0, start, 100);
    u.guys = vec![anim::Guy::fresh(1), anim::Guy::fresh(2)];
    let unit = sim.add_unit(u);
    // A scout dog's pair, as `rondata::artdata::piece_tracks` reads it.
    sim.art.tracks.insert(2, (-96, 48));
    make_mobile(&mut sim, unit, movement::Angle::EAST);
    sim.seat_guys(unit);
    let seated = sim.units[unit].guys[1].follow.expect("the crew has a body");
    assert_ne!(seated.body.pos, start, "the crew stands off its leader");

    sim.order_move(unit, Pos::new(4000 + 40 * 25, 400));
    let (mut walking, mut still) = (0, 0);
    while still < 4 {
        let before = sim.units[unit].pos;
        sim.tick();
        if sim.units[unit].pos == before {
            still += 1;
            continue;
        }
        still = 0;
        walking += 1;
        assert!(walking < 200, "the unit never arrived");
        let f = sim.units[unit].guys[1].follow.unwrap();
        assert_eq!(f.body.pos, f.des, "the crew keeps up while the unit moves");
    }
    assert!(walking > 20, "the unit walked for {walking} frames");
    let arrived = sim.units[unit].pos;

    // Now turn the unit where it stands — `Unit::set_angle(a, a, 0)`, the
    // heading without the snap, which is what every ordinary order does.
    sim.units[unit].movement.set_heading(movement::Angle::WEST);
    sim.tick();
    let lagging = sim.units[unit].guys[1].follow.unwrap();
    assert_ne!(lagging.body.pos, lagging.des, "the crew is still coming");
    // Neither figure is stopped on the frame of the turn itself — guy 0
    // takes `Guy::move`'s turn arm, which marks it unstopped and puts it
    // back on the walk. On the next frame it is settled and stops, and the
    // crew, which is still walking, does not.
    assert!(!sim.units[unit].guys[0].stopped);
    assert!(!sim.units[unit].guys[1].stopped);
    sim.tick();
    assert!(sim.units[unit].guys[0].stopped, "guy 0 has settled");
    assert!(!sim.units[unit].guys[1].stopped, "the crew has not");

    // And it gets there on its own, with the unit standing still.
    let mut frames = 0;
    while sim.units[unit].guys[1].follow.unwrap().body.pos
        != sim.units[unit].guys[1].follow.unwrap().des
    {
        sim.tick();
        frames += 1;
        assert!(frames < 30, "the crew never arrived");
    }
    assert!(
        frames >= 2,
        "the crew arrived in one frame, so nothing lagged"
    );
    assert_eq!(
        sim.units[unit].pos, arrived,
        "and the unit never moved again"
    );
    assert!(
        sim.units[unit].guys[1].follow.unwrap().facing == movement::Angle::WEST.quarter_turn()
            || sim.units[unit].guys[1].follow.unwrap().body.pos
                == sim.units[unit].guys[1].follow.unwrap().des
    );
}

/// **And a tracked crew figure pays the turning stand of its own**
/// (`docs/ANIM.md` §4.8, item 212) — the frame Great Lakes' word parted at
/// 6463, on a scenario built here rather than borrowed from a capture.
///
/// The figure above walks to its rotated offset over several frames; the
/// frame it *lands* on it is the one that costs. `Guy::move`'s standing arm
/// is per-guy, so the crew figure reaches its own
/// `turn_towards -> do_turn(..., 1)` there, and `do_turn`'s override
/// answers `guy_flags & 8` — which `Guy::init_real@005db6b0:215` sets for
/// every guy of a type that **packs**, whatever the art says. A merchant
/// has no `CHAR_TURN_RIGHT`, so the request falls to `CHAR_DEFAULT` on a
/// figure whose category is the walk, and rolls.
///
/// The control is the same scenario with the type not packing: the mark is
/// taken either way — `turn_towards` is called either way — and only the
/// packing run draws.
#[test]
fn a_crew_figure_of_a_packing_type_pays_the_turning_stand() {
    /// The frame the crew figure lands back on its offset, and what that
    /// frame's `SITE_TURN_STAND` cost.
    fn arrival_draws(packs: bool) -> u32 {
        let mut sim = skirmish(0);
        sim.trace_phases = true;
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            ..crate::UnitType::default()
        });
        sim.unit_types[ty].combat.packs = packs;
        let start = Pos::new(4000, 400);
        let mut u = Unit::new(0, 0, start, 100);
        u.ty = Some(ty);
        u.guys = vec![anim::Guy::fresh(1), anim::Guy::fresh(2)];
        let unit = sim.add_unit(u);
        sim.art.tracks.insert(2, (-96, 48));
        make_mobile(&mut sim, unit, movement::Angle::EAST);
        sim.seat_guys(unit);

        // The heading without the snap, which is what every ordinary order
        // passes: guy 0 turns where it stands and rotates the crew's
        // destination the whole width of the offset.
        sim.units[unit].movement.set_heading(movement::Angle::WEST);
        let mut frames = 0;
        loop {
            sim.tick();
            frames += 1;
            assert!(frames < 60, "the crew never arrived");
            let f = sim.units[unit].guys[1].follow.expect("the crew has a body");
            if f.body.pos == f.des && f.facing == f.des_angle {
                break;
            }
        }
        // The last frame's marks: what the stand spent, from the stream's
        // word at the mark to the word at the next one.
        let marks = &sim.phase_marks;
        let at = marks
            .iter()
            .position(|(l, _)| l == anim::SITE_TURN_STAND)
            .expect("the crew figure reached Guy::move's standing arm");
        let to = marks.get(at + 1).map_or(sim.rng.seed, |m| m.1);
        let mut r = crate::combat::Rng::new(marks[at].1);
        for n in 0..16 {
            if r.seed == to {
                return n;
            }
            r.roll();
        }
        panic!("the turning stand spent more than fifteen draws");
    }

    assert_eq!(
        arrival_draws(true),
        1,
        "a packing type's crew figure asks for a turn animation it has not \
         got and pays the idle roll"
    );
    assert_eq!(
        arrival_draws(false),
        0,
        "a type that neither packs nor names a turn asks for nothing"
    );
}

/// **Three of the six resources are not available from the start, and until
/// they are, a price written in one of them is charged somewhere else**
/// (`docs/COSTS.md`, "Three of the six resources are not available from the
/// start"; `docs/ECONOMY.md`, "Availability, and where a price lands
/// instead").
///
/// `economy::Holdings::available` had been all-true from frame 0 since it
/// existed, so the whole undiscovered-redirect layer under it — read,
/// tabled, and tested against hand-built arrays — never fired in a played
/// game. This is the wiring: [`Sim::sync_goods_available`] writes the two
/// arrays from `type_avail`, and `apply_gained` keeps them there.
///
/// Coinage is the case that found it. It costs `14k/6t`, and an Ancient-age
/// player is charged `(140 × 384) >> 8` = **210 food** and the sixty timber,
/// not a hundred and forty knowledge they have no way to hold.
#[test]
fn a_knowledge_price_lands_in_food_until_the_classical_age() {
    use crate::economy::Resource;
    use crate::tech::{self, TypeDef};

    let mut tree = tech::TechTree::new().with_tuning(&Tuning::RON);
    let ancient = tree.add(TypeDef::age("Ancient Age", 0));
    let classical = tree.add(TypeDef::age("Classical Age", 1).needs(0, ancient));
    // The six goods in `goodrules.xml` order; the last three carry the
    // prerequisite `resourcerules.xml` gives them.
    for (i, name) in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"]
        .iter()
        .enumerate()
    {
        let mut d = TypeDef::good(name);
        d.tribe_mask = u32::MAX;
        if i >= 3 {
            d = d.needs(0, classical);
        }
        tree.add(d);
    }
    let library = tree.add(TypeDef::building("Library"));
    let mut coinage = TypeDef::plain("Coinage", 1).at(library);
    coinage.cost[Resource::Knowledge.index()] = 140;
    coinage.cost[Resource::Timber.index()] = 60;
    coinage.tribe_mask = u32::MAX;
    let coinage = tree.add(coinage);

    let mut sim = skirmish(0);
    sim.set_tech_tree(tree);
    sim.start_techs(0);

    // Ancient: knowledge, metal and oil are none of the player's business.
    assert_eq!(
        sim.holdings[0].available,
        [true, true, true, false, false, false],
        "knowledge, metal and oil wait for their age"
    );
    assert_eq!(
        sim.holdings[1].available, sim.holdings[0].available,
        "and a player whose `start_techs` was never called has it from \
         `set_tech_tree`, not from a gain"
    );
    assert_eq!(
        sim.holdings[0].discovered, sim.holdings[0].available,
        "for a good the two tests are the same one, which is why the \
         obsolete table never fires in a stock game"
    );
    // Nothing lands in the knowledge slot, and the food slot carries the
    // knowledge price at three halves. The Science penalty this player takes
    // for a tech two levels ahead of its own line (`docs/COSTS.md`, "The
    // discounts") scales both sides alike, so the ratio is the assertion and
    // the two arrays are printed beside it.
    let ancient = sim.tech_price(0, coinage);
    sim.gain_tech(0, classical);
    assert_eq!(sim.holdings[0].available, [true; 6]);
    let classical_price = sim.tech_price(0, coinage);
    let k = Resource::Knowledge.index();
    let f = Resource::Food.index();
    let ti = Resource::Timber.index();
    assert_eq!(
        (ancient[k], classical_price[f]),
        (0, 0),
        "the two slots swap whole: {ancient:?} then {classical_price:?}"
    );
    assert!(classical_price[k] > 0 && ancient[f] > 0);
    assert_eq!(
        ancient[f],
        (classical_price[k] * 384) >> 8,
        "UNDISC_COST_GOOD Food at UNDISC_COST_RATE 3/2: {ancient:?} then \
         {classical_price:?}"
    );
    assert_eq!(
        (ancient[ti], classical_price[ti]),
        (classical_price[ti], ancient[ti]),
        "and the timber half of the price is untouched by either"
    );
}

#[test]
fn reduced_path_counterexample_stops_at_a_combined_final_flag() {
    // Reduced against the original executable from an injected equality-for-
    // bitmask fault. Authored semantic input; no install or capsule required.
    let previous_segment = orders::PathData {
        to: Pos::new(0, 0),
        tolerance: 0,
        flags: 0,
    };
    let current_segment = orders::PathData {
        flags: orders::path_flag::FINAL | orders::path_flag::SIDESTEP,
        ..previous_segment
    };
    let mut unit = Unit::new(0, 0, Pos::new(0, 0), 1);
    unit.path.extend([previous_segment, current_segment]);
    unit.discard_current_path_segment();
    assert_eq!(unit.path, vec![previous_segment]);
}
