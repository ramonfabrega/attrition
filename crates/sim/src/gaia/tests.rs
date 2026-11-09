use super::*;
use crate::combat::Rng;
use crate::tuning::Tuning;
use crate::world::World;

fn sim_at(seed: u32) -> Sim {
    let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
    s.rng = Rng::new(seed);
    s
}

fn stepped(seed: u32, n: usize) -> u32 {
    let mut r = Rng::new(seed);
    for _ in 0..n {
        r.roll();
    }
    r.seed
}

/// Run12, frame 0, draws 108–109: herd 0 at `(22, 34)` walks to
/// `(22, 35)` — `% 3 = 1, 2`.
#[test]
fn run12_s_herd_walk() {
    let mut s = sim_at(stepped(0x3bd3_9ae9, 108));
    s.gaia.gaia.herds.push(Herd {
        cx: 22,
        cy: 34,
        wx: 22,
        wy: 34,
        kind: 408,
        alive: true,
    });
    s.gaia.gaia.herds.push(Herd {
        cx: 44,
        cy: 21,
        wx: 44,
        wy: 21,
        kind: 411,
        alive: true,
    });
    s.herd_walk(0);
    assert_eq!((s.gaia.gaia.herds[0].wx, s.gaia.gaia.herds[0].wy), (22, 35));
    assert_eq!((s.gaia.gaia.herds[1].wx, s.gaia.gaia.herds[1].wy), (44, 21));
    assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 110));
}

/// **The second walk is about the home cell, not the first walk's
/// answer.** `Herd::process@00741760` reads `cx`/`cy` and writes
/// `wx`/`wy`, so the wander centre is never more than one cell from
/// home however often the herd is processed. Run12's first walk moves
/// herd 0 to `(22, 35)`; the same two draws applied a second time
/// return it to `(22, 35)` rather than carrying it on to `(22, 36)`.
#[test]
fn a_second_herd_walk_jitters_about_home_again() {
    let mut s = sim_at(stepped(0x3bd3_9ae9, 108));
    s.gaia.gaia.herds.push(Herd {
        cx: 22,
        cy: 34,
        wx: 22,
        wy: 35,
        kind: 408,
        alive: true,
    });
    s.herd_walk(0);
    assert_eq!((s.gaia.gaia.herds[0].wx, s.gaia.gaia.herds[0].wy), (22, 35));
}

/// Run12, frame 0, draws 88–107: ten attempts on a 60×60 map, none on a
/// `0x20` cell — twenty draws and no bird.
#[test]
fn run12_s_bird_sampling_draws_twenty() {
    let mut s = sim_at(stepped(0x3bd3_9ae9, 88));
    s.sample_birds(0);
    assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 108));
    assert!(s.gaia.gaia.bird_spawns.is_empty());
}

/// The cadences: frame 0 takes both, frames 1–31 neither, 32 the
/// birds, 64 both again with the next herd; a small map samples fewer.
#[test]
fn the_cadences() {
    let mut s = sim_at(7);
    s.gaia.gaia.herds.push(Herd {
        cx: 5,
        cy: 5,
        wx: 5,
        wy: 5,
        kind: 1,
        alive: true,
    });
    let seed = s.rng.seed;
    s.process_gaia(0);
    assert_eq!(s.rng.seed, stepped(seed, 22));
    let seed = s.rng.seed;
    for f in 1..32 {
        s.process_gaia(f);
    }
    assert_eq!(s.rng.seed, seed);
    s.process_gaia(32);
    assert_eq!(s.rng.seed, stepped(seed, 20));
    let seed = s.rng.seed;
    s.process_gaia(64);
    // Herd index (64 >> 6) % max(1, 5) = 1: there is no herd 1.
    assert_eq!(s.rng.seed, stepped(seed, 20));
    let mut small = Sim::new(Tuning::RON, World::new(20, 20), 2);
    small.rng = Rng::new(7);
    small.process_gaia(0);
    assert_eq!(small.rng.seed, stepped(7, 8), "20·20/100 = 4 attempts");
}

/// A hit records the spawn the sim does not model.
#[test]
fn a_mountain_cell_records_a_spawn() {
    let mut s = sim_at(7);
    for x in 0..60 {
        for y in 0..60 {
            let c = Cell { x, y };
            let mut d = s.world.cell_data(c);
            d.flags |= BIRD_CELL;
            s.world.set_cell_data(c, d);
        }
    }
    s.sample_birds(0);
    assert_eq!(s.gaia.gaia.bird_spawns.len(), 10);
}

/// **The console's `bird` and the sampling are one pair of calls**,
/// differing only in the point (`docs/GOLDEN.md` §10, item 652).
/// `Unit::init`'s snap seats the bird on its 48-unit tile's centre and
/// the patrol order keeps the point it was handed, so a cursor of
/// `(0, 6)` seats it where `(0, 0)` would and patrols six units apart.
/// That is the whole of what run169's packet had to measure.
#[test]
fn a_staged_bird_is_seated_on_its_tile_and_patrols_the_raw_point() {
    let mut s = sim_at(7);
    s.add_unit_type(crate::UnitType {
        hits: 1,
        type_index: BIRD_TYPE_INDEX,
        ..crate::UnitType::default()
    });
    let staged = s.spawn_bird_at(Pos::new(0, 6)).expect("a bird type");
    assert_eq!(s.units[staged].owner, BIRD_OWNER);
    assert_eq!(s.units[staged].pos, Pos::new(24, 24));
    assert_eq!(s.bird_goal(staged), Pos::new(0, 6));
    assert_eq!(s.live_birds(), 1);
    // The sampling's own bird, on a cell centre, the same way.
    let hatched = s.spawn_bird(Cell { x: 2, y: 3 }).expect("a bird type");
    let centre = |c: i32| c * crate::world::UNITS_PER_CELL + crate::world::UNITS_PER_CELL / 2;
    assert_eq!(s.bird_goal(hatched), Pos::new(centre(2), centre(3)));
    assert_eq!(
        s.units[hatched].pos,
        Pos::new(centre(2) + 24, centre(3) + 24)
    );
    assert_eq!(s.live_birds(), 2);
}

/// **`AnimalData::get_speed@005d8380`, the animal's own** — the slot
/// `+0x17c` override that replaces `UnitData::get_speed` whole
/// (`docs/MOVEMENT.md`, "The animal's own `get_speed`").
///
/// The threshold is on the order's **goal**, not the herd centre and
/// not the current waypoint, and it is strict: `0x180` exactly is
/// near. run39's `8/2` crosses it mid-walk, which is what put its
/// blocked stand a frame late (`docs/SYNC.md` §3.13).
#[test]
fn an_animal_beyond_0x180_of_its_order_walks_at_three_halves() {
    use crate::orders::{MoveKind, QueuePos};
    let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
    let ty = s.add_unit_type(crate::UnitType {
        hits: 1,
        moves: 19,
        ..crate::UnitType::default()
    });
    let at = Pos::new(28776, 24360);
    let mut u = crate::Unit::new(8, 2, at, 1);
    u.ty = Some(ty);
    let a = s.add_unit(u);
    s.units[a].movement.speed = 19;
    assert_eq!(s.get_speed(a, 0), 19, "no order at all: the base, floored");

    // 432 from the goal — the distance run39 prints on the frame its
    // step is 28 long.
    let far = Pos::new(28968, 23976);
    assert_eq!(crate::world::vector_dist(192, -384), 432);
    s.add_move_order(a, far, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(s.get_speed(a, 0), 28, "19 * 3 / 2, truncated");

    // And the step it buys, against the record: `(12, −25)`.
    let want = crate::movement::find_angle(far.x - at.x, far.y - at.y);
    assert_eq!(
        (
            crate::movement::sin_component(want, 28),
            crate::movement::cos_component(want, 28)
        ),
        (12, 25)
    );

    // Exactly `0x180` is near — the test is `> 0x180`, not `>=`.
    s.units[a].orders.clear();
    let near = Pos::new(28776, 24360 - 0x180);
    s.add_move_order(a, near, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(crate::world::vector_dist(0, -0x180), 0x180);
    assert_eq!(s.get_speed(a, 0), 19, "0x180 is not more than 0x180");

    // A player's unit takes none of it, however far it is going.
    let mut p = crate::Unit::new(0, 0, at, 1);
    p.ty = Some(ty);
    let h = s.add_unit(p);
    s.units[h].movement.speed = 19;
    s.add_move_order(h, far, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(s.get_speed(h, 0), 19, "`UnitData::get_speed` has no hurry");

    // Nor does a bird: air returns before the hurry *and* before the
    // floor, which is the only way to see the two apart.
    s.units[a].kind.domain = crate::attrition::Domain::Air;
    s.units[a].movement.speed = 1;
    s.units[a].orders.clear();
    s.add_move_order(a, far, MoveKind::MoveTo, QueuePos::New, false);
    assert_eq!(
        s.get_speed(a, 0),
        1,
        "air: neither the 3/2 nor the floor of 3"
    );
    s.units[a].kind.domain = crate::attrition::Domain::Land;
    assert_eq!(s.get_speed(a, 0), 3, "and on the ground, the floor");
}
