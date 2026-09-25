//! A randomised soak: many games, each played twice, each compared frame for
//! frame.
//!
//! The determinism test in `harness_tests.rs` runs **one hand-built
//! scenario** twice. That proves the tick is a function of its state on the
//! path that scenario walks, and says nothing about any other path — which
//! is a real gap, because a lockstep simulation is only as deterministic as
//! its worst-behaved branch, and the branches this project keeps adding
//! (orders, gather chains, the swarm ring, target search) are exactly the
//! ones with iteration order in them.
//!
//! So this generates games instead: a seeded scenario, a seeded stream of
//! orders issued at seeded frames, and then the same game again from the same
//! seed. Two properties are asserted, and both are the kind that a
//! hand-written test cannot cover by construction.
//!
//! 1. **Replay equality.** The two runs must agree on a per-frame digest of
//!    selected gameplay state and the gameplay RNG state. A mismatch names
//!    the frame, which is where a desync hunt starts.
//! 2. **No panic.** The generator deliberately issues orders that are legal
//!    to *call* and unreasonable to *mean* — a move to a position off the
//!    map, a gather on a building that is not a gather type, an attack on a
//!    unit that has died, a build order on a finished building, a garrison
//!    into something with no room. Everything the simulation does with
//!    integers is under `overflow-checks` in this profile, so an arithmetic
//!    overflow is a panic and therefore a failure.
//!
//! What it is not: a proof, and not a fuzzer with coverage feedback. It is a
//! wide, cheap, reproducible sweep — and every seed that fails is a named,
//! re-runnable test case.

use super::*;
use crate::build::{BuildType, Ident, flags};
use crate::combat::Rng;
use crate::orders::QueuePos;
use crate::world::UNITS_PER_CELL;

const CELLS: i32 = 16;

/// FNV-1a, the standard 64-bit parameters — a digest, not a hash table.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn eat(&mut self, v: i64) {
        for byte in v.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x1000_0000_01b3);
        }
    }
    fn eat_bool(&mut self, v: bool) {
        self.eat(i64::from(v));
    }
}

/// Selected lockstep state, including the gameplay RNG, in one number.
///
/// Deliberately explicit rather than derived from `Debug`: what belongs in a
/// checksum is a claim about what is simulation state, and writing it out is
/// how that claim stays reviewable. Anything missing here is something a
/// divergence could hide in, so the list is meant to grow with the sim.
fn sample(sim: &Sim) -> Sample {
    let mut h = Fnv::new();
    h.eat(sim.units.len() as i64);
    for u in &sim.units {
        h.eat(i64::from(u.owner));
        h.eat(i64::from(u.index));
        h.eat(i64::from(u.pos.x));
        h.eat(i64::from(u.pos.y));
        h.eat(i64::from(u.health));
        h.eat(i64::from(u.damage_frac));
        h.eat(i64::from(u.squad_size));
        h.eat(i64::from(u.attrition));
        h.eat_bool(u.on_map);
        h.eat_bool(u.sheltered);
        h.eat(i64::from(u.idle));
        h.eat(i64::from(u.stance));
        h.eat(i64::from(u.movement.facing.0));
        h.eat(i64::from(u.movement.body.pos.x));
        h.eat(i64::from(u.movement.body.pos.y));
        h.eat(i64::from(u.movement.body.last_speed));
        h.eat(i64::from(u.movement.body.avg_speed));
        h.eat(
            u.movement
                .dest
                .map_or(-1, |d| i64::from(d.x) * 65_536 + i64::from(d.y)),
        );
        h.eat(i64::from(u.combat.recharging));
        h.eat(i64::from(u.combat.targeted));
        h.eat(u.inside.map_or(-1, |b| b as i64));
        h.eat(u.inside_unit.map_or(-1, |b| b as i64));
        // The order list, in order: kind, flags, and what each targets.
        h.eat(u.orders.len() as i64);
        for o in &u.orders {
            h.eat(i64::from(o.index()));
            h.eat(i64::from(o.flags));
            h.eat(match o.body {
                orders::Body::Build(b) | orders::Body::Repair(b) => b as i64,
                orders::Body::Garrison { building, .. } => building as i64,
                orders::Body::Gather(g) => g.building as i64,
                orders::Body::Move(m) => i64::from(m.dest.x) * 65_536 + i64::from(m.dest.y),
                orders::Body::Cast(c) => i64::from(c.spell) * 2 + i64::from(c.paid),
                orders::Body::Trade(tr) => tr.home as i64 * 256 + tr.dest.map_or(-1, |d| d as i64),
                orders::Body::Guard(g) => {
                    (g.target as i64 * 65_536 + i64::from(g.guard.x)) * 65_536
                        + i64::from(g.guard.y)
                        + i64::from(g.retry)
                }
                orders::Body::Follow(f) => f.target as i64,
                orders::Body::AttackGround(g) => {
                    (i64::from(g.at.x) * 65_536 + i64::from(g.at.y)) * 4 + i64::from(g.attack_unit)
                }
                orders::Body::Patrol(p) => {
                    (p.id * 4 + p.waypoint as i64) * 65_536 + p.leader as i64 + p.form_id as i64
                }
                orders::Body::Attack(_) | orders::Body::Think => -1,
            });
        }
        h.eat(u.path.len() as i64);
        for p in &u.path {
            h.eat(i64::from(p.to.x));
            h.eat(i64::from(p.to.y));
            h.eat(i64::from(p.tolerance));
            h.eat(i64::from(p.flags));
        }
    }
    h.eat(sim.buildings.len() as i64);
    for b in &sim.buildings {
        h.eat(i64::from(b.owner));
        h.eat(i64::from(b.index));
        h.eat(i64::from(b.pos.x));
        h.eat(i64::from(b.pos.y));
        h.eat(i64::from(b.health));
        h.eat(i64::from(b.damage_frac));
        h.eat(i64::from(b.job_counter));
        h.eat_bool(b.alive);
        h.eat_bool(b.active);
        h.eat_bool(b.started);
        h.eat(b.city.map_or(-1, |c| c as i64));
        h.eat(b.gatherers.len() as i64);
        for g in &b.gatherers {
            h.eat(*g as i64);
        }
        h.eat(b.garrison.len() as i64);
    }
    h.eat(sim.cities.len() as i64);
    for c in &sim.cities {
        h.eat(i64::from(c.owner));
        h.eat_bool(c.alive);
    }
    for l in &sim.ledgers {
        for good in l.bucket {
            h.eat(i64::from(good));
        }
    }
    // Snapshot activity before adding state that advances in a frozen world.
    let activity = h.0;
    h.eat(sim.frame);
    h.eat(i64::from(sim.rng.seed));
    Sample {
        replay: h.0,
        activity,
    }
}

/// The type table the soak plays with: enough shapes for the branches to be
/// reachable, and no more. A gather type that is flat and one that is not is
/// the important pair — they are two different machines (`docs/ORDERS.md`
/// §6.4, §6.5).
struct Types {
    village: usize,
    farm: usize,
    woodcutter: usize,
    tower: usize,
    citizen: usize,
    soldier: usize,
}

fn bt(ident: Ident, flag: &str, xs: i32, ys: i32, hits: i32, garrison: i32) -> BuildType {
    BuildType {
        ident,
        x_size: xs,
        y_size: ys,
        flags: flags::parse(flag),
        job_time: 300,
        hits,
        garrison_max: garrison,
        price: cost::Price::free(),
        ..BuildType::default()
    }
}

fn install(sim: &mut Sim) -> Types {
    let village = sim.add_build_type(bt(Ident::Village, "ean", 7, 7, 1200, 10));
    let mut farm = bt(Ident::Farm, "gda", 4, 4, 400, 0);
    farm.flags |= flags::FLAT;
    let farm = sim.add_build_type(farm);
    let woodcutter = sim.add_build_type(bt(Ident::Woodcutter, "gda", 3, 3, 400, 0));
    let mut tower = bt(Ident::Tower, "ean", 2, 2, 750, 5);
    tower.attack = 12;
    tower.combat = Some(combat::Profile {
        attack: 12,
        max_range: 10,
        recharge: 30,
        most_shots: 2,
        ..combat::Profile::default()
    });
    let tower = sim.add_build_type(tower);

    let citizen = sim.add_unit_type(UnitType {
        hits: 40,
        moves: 25,
        turn_speed: movement::degrees_to_angle(45).0,
        worker: orders::Worker::Citizen,
        ..UnitType::default()
    });
    let soldier = sim.add_unit_type(UnitType {
        hits: 100,
        moves: 30,
        turn_speed: movement::degrees_to_angle(30).0,
        combat: combat::Profile {
            attack: 10,
            max_range: 2,
            recharge: 20,
            most_shots: 1,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    });
    Types {
        village,
        farm,
        woodcutter,
        tower,
        citizen,
        soldier,
    }
}

fn centre(c: Cell) -> Pos {
    Pos::new(
        c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    )
}

/// Builds one game from a seed and returns it with its type table.
fn scenario(rng: &mut Rng) -> (Sim, Types) {
    let mut w = World::new(CELLS, CELLS);
    w.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(CELLS - 1, CELLS - 1),
    );
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.declare_war(0, 1);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    let types = install(&mut sim);

    // A city per player, at opposite corners, and a scatter of gather sites
    // and towers around them.
    for who in 0u8..2 {
        let base = if who == 0 { 3 } else { CELLS - 4 };
        let city = centre(Cell::new(base, base));
        let _ = sim.place_building(who, types.village, city);
        for _ in 0..3 {
            let ty = match rng.get(0, 2) {
                0 => types.farm,
                1 => types.woodcutter,
                _ => types.tower,
            };
            let p = Pos::new(
                city.x + rng.get(-6, 6) * UNITS_PER_CELL,
                city.y + rng.get(-6, 6) * UNITS_PER_CELL,
            );
            let _ = sim.place_building(who, ty, p);
        }
    }
    // Give every woodcutter a tile list, the way a `BUILDS=7` dump does.
    for b in 0..sim.buildings.len() {
        if sim.building_ident(b) == Ident::Woodcutter {
            let at = sim.buildings[b].pos.tile();
            sim.buildings[b].gather_from = (0..8)
                .map(|k| Pos::new(at.x + rng.get(-6, 6), at.y + rng.get(-6, 6) + k % 2))
                .collect();
        }
    }

    // The units: citizens and soldiers, both players, scattered.
    let n = rng.get(6, 14);
    for i in 0..n {
        let who = (i % 2) as Player;
        let worker = rng.get(0, 1) == 0;
        let ty = if worker { types.citizen } else { types.soldier };
        let t = &sim.unit_types[ty];
        let (hits, moves, turn) = (t.hits, t.moves, t.turn_speed);
        let pos = centre(Cell::new(rng.get(1, CELLS - 2), rng.get(1, CELLS - 2)));
        let mut u = Unit::new(who, i as i16, pos, hits);
        u.ty = Some(ty);
        u.squad_size = rng.get(1, 4);
        u.movement.speed = moves;
        u.movement.turning.type_turn_speed = turn;
        sim.add_unit(u);
    }
    sim.recompute_territory();
    (sim, types)
}

/// Issues one arbitrary order to one arbitrary unit.
///
/// Deliberately includes the unreasonable: destinations off the map, gather
/// orders on towers, attacks on the dead and on oneself, build orders on
/// finished buildings. All of them are legal calls — the indices are in
/// range — and all of them are things a real order stream will eventually
/// contain, whether from a confused AI or a player clicking during a death.
fn issue(sim: &mut Sim, rng: &mut Rng) {
    if sim.units.is_empty() {
        return;
    }
    let u = rng.get(0, sim.units.len() as i32 - 1) as usize;
    let b = if sim.buildings.is_empty() {
        None
    } else {
        Some(rng.get(0, sim.buildings.len() as i32 - 1) as usize)
    };
    let pos = Pos::new(
        rng.get(-2 * UNITS_PER_CELL, (CELLS + 2) * UNITS_PER_CELL),
        rng.get(-2 * UNITS_PER_CELL, (CELLS + 2) * UNITS_PER_CELL),
    );
    let queue = match rng.get(0, 2) {
        0 => QueuePos::New,
        1 => QueuePos::First,
        _ => QueuePos::Last,
    };
    let action = rng.get(0, 1) == 0;
    match rng.get(0, 7) {
        0 => sim.add_move_order(u, pos, orders::MoveKind::MoveTo, queue, action),
        1 => sim.add_move_order(u, pos, orders::MoveKind::ExploreTo, queue, action),
        2 => {
            if let Some(b) = b {
                sim.add_build_order(u, b, queue, action);
            }
        }
        3 => {
            if let Some(b) = b {
                // Never `QUEUE_FIRST`: `add_repair_order` has no such branch
                // in the original and asserts as much, so passing one would
                // be the generator breaking a documented precondition rather
                // than the simulation failing (`docs/audit/
                // 2026-08-21-orders.md` R3 P1). The soak found this on its
                // first run, which is the assert doing its job.
                let queue = if queue == QueuePos::First {
                    QueuePos::Last
                } else {
                    queue
                };
                sim.add_repair_order(u, b, queue, action);
            }
        }
        4 => {
            if let Some(b) = b {
                sim.add_gather_order(u, b, queue, action);
            }
        }
        5 => {
            if let Some(b) = b {
                sim.add_garrison_order(u, b, rng.get(0, 1) == 0, queue, action);
            }
        }
        6 => {
            let target = if rng.get(0, 3) == 0 && !sim.buildings.is_empty() {
                combat::Obj::Building(rng.get(0, sim.buildings.len() as i32 - 1) as usize)
            } else {
                combat::Obj::Unit(rng.get(0, sim.units.len() as i32 - 1) as usize)
            };
            sim.add_attack_order(u, target, queue, rng.get(0, 1) == 0, action);
        }
        _ => sim.clear_orders(u),
    }
}

#[derive(Clone, Copy)]
struct Sample {
    replay: u64,
    activity: u64,
}

fn digest(sim: &Sim) -> u64 {
    sample(sim).replay
}

fn distinct_activity(samples: &[Sample]) -> usize {
    let mut values: Vec<u64> = samples.iter().map(|s| s.activity).collect();
    values.sort_unstable();
    values.dedup();
    values.len()
}

fn require_activity(seed: u32, samples: &[Sample]) -> usize {
    let distinct = distinct_activity(samples);
    assert!(
        distinct > 50,
        "seed {seed}: only {distinct} distinct gameplay states — generated game is barely active"
    );
    distinct
}

/// Plays one game and returns its per-frame digests.
fn play(seed: u32, frames: usize) -> Vec<Sample> {
    let mut rng = Rng::new(seed);
    let (mut sim, _types) = scenario(&mut rng);
    let mut out = Vec::with_capacity(frames + 1);
    out.push(sample(&sim));
    for _ in 0..frames {
        // A burst of orders now and then rather than one a frame: a unit that
        // is re-ordered every frame never reaches the interesting half of any
        // order's lifetime.
        if rng.get(0, 9) == 0 {
            for _ in 0..rng.get(1, 3) {
                issue(&mut sim, &mut rng);
            }
        }
        sim.tick();
        out.push(sample(&sim));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_and_rng_only_progress_cannot_satisfy_activity_guard() {
        let (mut sim, _) = scenario(&mut Rng::new(12345));
        let initial = sample(&sim);
        sim.frame += 1;
        assert_eq!(sample(&sim).activity, initial.activity);
        assert_ne!(
            sample(&sim).replay,
            initial.replay,
            "replay still includes the clock"
        );
        sim.frame -= 1;
        let mut samples = vec![sample(&sim)];
        for _ in 0..400 {
            sim.frame += 1;
            sim.rng.roll();
            samples.push(sample(&sim));
        }
        assert!(
            samples
                .windows(2)
                .all(|pair| pair[0].replay != pair[1].replay)
        );
        assert_eq!(
            distinct_activity(&samples),
            1,
            "clock/RNG changes are not gameplay activity"
        );
        assert!(std::panic::catch_unwind(|| require_activity(12345, &samples)).is_err());
    }

    #[test]
    fn rng_only_changes_part_the_digest_before_visible_state_changes() {
        let (mut sim, _) = scenario(&mut Rng::new(12345));
        let seed = sim.rng.seed;
        let baseline = digest(&sim);
        // Only the gameplay generator changes: no tick or order is issued.
        for bit in 0..32 {
            sim.rng.seed = seed ^ (1 << bit);
            assert_ne!(
                digest(&sim),
                baseline,
                "an RNG-only seed change at bit {bit}"
            );
        }
        sim.rng.seed = seed;
        assert_eq!(digest(&sim), baseline);
        sim.rng.roll();
        assert_ne!(sim.rng.seed, seed);
        assert_ne!(
            digest(&sim),
            baseline,
            "an extra gameplay draw must be visible immediately"
        );
        sim.rng.seed = seed;
        assert_eq!(digest(&sim), baseline);
    }

    /// The hang the soak found, reduced to one order.
    ///
    /// A citizen at speed 25, told to walk 60 east and 1,600 north — two
    /// cells, so inside `find_path`'s straight-line range. `sin_component`
    /// truncates 25 × 60 / 1601 to **zero**, so the march never moves in x
    /// while the goal is 60 away, more than one step; once it has closed the
    /// y remainder exactly it stands still, and neither exit test can fire.
    /// Before the progress guard in `orders::find_path` this call never
    /// returned. It is a plain order with plain geometry, not a pathological
    /// one, which is what makes it worth a test of its own.
    #[test]
    fn a_near_axis_move_does_not_hang_the_march() {
        let mut w = World::new(16, 16);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
        let mut sim = Sim::new(Tuning::RON, w, 1);
        let mut u = Unit::new(0, 0, Pos::new(4000, 4000), 40);
        u.movement.speed = 25;
        u.movement.turning.type_turn_speed = movement::degrees_to_angle(45).0;
        u.movement.turning.instant_from_stop = true;
        let idx = sim.add_unit(u);

        // The premise, stated rather than assumed: the cross-axis component
        // really does truncate to zero here.
        let ang = movement::find_angle(60, 1600);
        assert_eq!(movement::sin_component(ang, 25), 0, "the x component");
        assert_ne!(movement::cos_component(ang, 25), 0, "the y component");

        sim.add_move_order(
            idx,
            Pos::new(4060, 5600),
            orders::MoveKind::MoveTo,
            QueuePos::New,
            true,
        );
        // It returns, and the unit gets where it was sent.
        for _ in 0..200 {
            sim.tick();
        }
        let end = sim.units[idx].pos;
        assert!(
            (end.y - 5600).abs() <= 48 && (end.x - 4060).abs() <= 48,
            "the unit did not arrive: {end:?}"
        );
    }

    /// Every generated game replays identically, frame for frame.
    #[test]
    fn a_randomised_game_replays_identically() {
        let mut moved = 0;
        for seed in 1u32..=24 {
            let a = play(seed, 400);
            let b = play(seed, 400);
            assert_eq!(a.len(), b.len());
            if let Some(frame) = (0..a.len()).find(|&i| a[i].replay != b[i].replay) {
                panic!(
                    "seed {seed}: two runs of the same game diverged at frame {frame} \
                     ({:#x} vs {:#x}). Re-run with `play({seed}, 400)`.",
                    a[frame].replay, b[frame].replay
                );
            }
            // A soak of frozen games would pass this test perfectly and
            // prove nothing, so count how much actually happened.
            let distinct = require_activity(seed, &a);
            moved += distinct;
        }
        assert!(moved > 24 * 100, "the sweep as a whole is too quiet");
    }

    /// The same sweep, wider and shorter, is also a panic and overflow
    /// hunt — every arithmetic step runs under `overflow-checks` here.
    #[test]
    fn no_generated_game_panics() {
        for seed in 100u32..=180 {
            let _ = play(seed, 120);
        }
    }

    /// The digest has to be able to tell two states apart, or the test above
    /// passes for the wrong reason.
    #[test]
    fn the_digest_notices_a_change() {
        let mut rng = Rng::new(7);
        let (mut sim, _) = scenario(&mut rng);
        let before = digest(&sim);
        let activity = sample(&sim).activity;
        sim.units[0].pos.x += 1;
        assert_ne!(activity, sample(&sim).activity, "movement is activity");
        assert_ne!(before, digest(&sim), "a moved unit");
        sim.units[0].pos.x -= 1;
        assert_eq!(before, digest(&sim), "and back again");
        assert_eq!(activity, sample(&sim).activity, "restored position");
        sim.units[0].health -= 1;
        assert_ne!(activity, sample(&sim).activity, "damage is activity");
        assert_ne!(before, digest(&sim), "a wounded unit");
    }
}
