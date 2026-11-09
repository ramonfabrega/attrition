use super::*;
use crate::combat::Rng;

fn sim() -> Sim {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    s.rng = Rng::new(7);
    let t = s.add_unit_type(crate::UnitType {
        hits: 1,
        moves: 35,
        turn_speed: crate::movement::degrees_to_angle(5).0,
        kind: crate::attrition::UnitKind {
            domain: crate::attrition::Domain::Air,
            ..crate::attrition::UnitKind::default()
        },
        ..crate::UnitType::default()
    });
    // Named after the fact: the fixture's tech tree has no row 0x193.
    s.tables.unit_types[t].tree = Some(FLOCKBIRD);
    s
}

fn rolled(mut r: Rng, n: usize) -> Rng {
    for _ in 0..n {
        let _ = r.roll();
    }
    r
}

fn birds(s: &Sim) -> Vec<usize> {
    (0..s.units.len())
        .filter(|&u| s.units[u].owner == 9 && s.is_flockbird(u))
        .collect()
}

/// **Six draws for the flock and ten a bird, and each bird flies the
/// cells in the order they were listed** — the given one first, the
/// wood last (run500's proxied `do_air_physics` on 8787). Each bird is
/// born 24 on from the tile centre, on the 48-unit grid (run500's
/// `set_new_location`), and a player's second flock inside 225 frames
/// spends nothing and makes nothing.
#[test]
fn a_flock_is_six_draws_and_ten_a_bird_flown_first_cell_first() {
    let mut s = sim();
    s.frame = 1000;
    let before = s.rng;
    let (cx, cy) = (10, 12);
    // Every cell a wood but the given one, so the fourth cell — the
    // wood — is never the first.
    for y in 0..s.world.height() {
        for x in 0..s.world.width() {
            if (x, y) != (cx, cy) {
                let c = Cell::new(x, y);
                let mut d = s.world.cell_data(c);
                d.flags |= cell::FOREST;
                s.world.set_cell_data(c, d);
            }
        }
    }
    s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 3);
    assert_eq!(s.rng, rolled(before, 6 + 3 * 10), "36 draws");
    let b = birds(&s);
    assert_eq!(b.len(), 3);
    for &u in &b {
        assert_eq!(s.units[u].orders.len(), 4);
        // `(4c + 2) · 0xc0 + 0x60`, snapped: `c · 768 + 504`.
        assert_eq!(
            s.units[u].pos,
            Pos::new(cx * 768 + 504, cy * 768 + 504),
            "born on the grid's centre"
        );
        let Body::AirAttackGround(g) = s.units[u].orders[0].body else {
            panic!("an air attack on the ground");
        };
        assert_eq!(
            (g.at.x - 0x80).div_euclid(UNITS_PER_CELL),
            cx,
            "the given cell is flown first"
        );
        assert_eq!((g.at.y - 0x80).div_euclid(UNITS_PER_CELL), cy);
        let Body::AirAttackGround(w) = s.units[u].orders[3].body else {
            panic!("an air attack on the ground");
        };
        let wood = Cell::new(
            (w.at.x - 0x80).div_euclid(UNITS_PER_CELL),
            (w.at.y - 0x80).div_euclid(UNITS_PER_CELL),
        );
        assert!(
            wood != Cell::new(cx, cy) && s.world.cell_data(wood).flags & cell::FOREST != 0,
            "the wood is flown last"
        );
        assert_eq!(s.units[u].airframe.z, s.units[u].airframe.last_z + 200);
    }
    assert_eq!(s.gaia_state.flock_stamp[1], 1000);
    s.frame = 1000 + FLOCK_GAP - 1;
    let before = s.rng;
    s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 3);
    assert_eq!(s.rng, before, "a refused flock spends nothing");
    assert_eq!(birds(&s).len(), 3);
    s.frame = 1000 + FLOCK_GAP;
    s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 2);
    assert_eq!(birds(&s).len(), 5, "225 frames on, the next one flies");
}

/// **The group move's wood arm**: a slot in a `FOREST` cell within
/// `0x300` of a unit in a group rolls the flock's size and sends it up
/// (`5e83f0`–`5e84cb`); out of a group, or on open ground, it spends
/// nothing.
#[test]
fn a_slot_in_a_wood_flushes_a_flock_from_a_unit_in_a_group() {
    let mut s = sim();
    s.frame = 1000;
    let here = Pos::new(10 * 768 + 400, 12 * 768 + 400);
    let mut unit = crate::Unit::new(1, 5, here, 10);
    unit.group_ptr = Some(1);
    let u = s.add_unit(unit);
    let slot = Pos::new(10 * 768 + 600, 12 * 768 + 300);
    let before = s.rng;
    s.group_flock(u, slot);
    assert_eq!(s.rng, before, "open ground: nothing");
    let c = Cell::new(10, 12);
    let mut d = s.world.cell_data(c);
    d.flags |= cell::FOREST;
    s.world.set_cell_data(c, d);
    s.units[u].group_ptr = None;
    s.group_flock(u, slot);
    assert_eq!(s.rng, before, "in no group: nothing");
    s.units[u].group_ptr = Some(1);
    s.group_flock(u, Pos::new(slot.x + 0x300, slot.y));
    assert_eq!(s.rng, before, "0x300 away: nothing");
    s.group_flock(u, slot);
    let n = birds(&s).len();
    assert!(n == 2 || n == 3, "two or three birds, {n}");
    assert_eq!(
        s.rng,
        rolled(before, 1 + 6 + 10 * n),
        "the size, then the flock"
    );
}
