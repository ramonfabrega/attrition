use super::*;
use crate::Tuning;
use crate::ai_load::uflags2;
use crate::world::{Cell, Terrain, World};

/// A computer leader's scout on open land, in no army.
fn ai_scout() -> (Sim, usize) {
    let mut world = World::new(10, 10);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(9, 9));
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities_state.nation[0].human = true;
    s.cities_state.nation[1].human = false;
    let t = s.add_unit_type(crate::UnitType::default());
    s.tables.unit_types[t].cols.unit_flags2 = uflags2::FLAG2_SCOUT | uflags2::CASTER;
    let mut u = crate::Unit::new(1, 0, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
    u.ty = Some(t);
    let u = s.add_unit(u);
    (s, u)
}

/// run346's frame 0 (`docs/AI.md` §80.5): at Easiest and Easy the arm
/// returns before any draw, from Moderate up it throws the coin once,
/// and with no army and a coin not divisible by three it returns 0.
#[test]
fn an_ai_scout_throws_the_special_coin_from_difficulty_two() {
    let (mut sim, scout) = ai_scout();
    for d in 0..=5 {
        sim.lobby.difficulty = d;
        let mut probe = sim.rng;
        let coin = probe.roll();
        let before = sim.rng;
        assert!(!sim.think_spellcaster(scout), "difficulty {d}");
        if d < 2 {
            assert_eq!(sim.rng, before, "difficulty {d}: no draw");
        } else {
            assert_eq!(sim.rng, probe, "difficulty {d}: one draw, {coin}");
        }
    }
}

/// A computer leader's General on the move at (5, 5), its type
/// `MANA 1000`, and the three crafts of the hero arm priced 500, 500
/// and 1,000. Returns the sim, the General and a seed per coin parity.
fn ai_general() -> (Sim, usize, [u32; 2]) {
    use crate::orders::{MoveKind, QueuePos, SpellType, spell};
    let (mut s, _) = ai_scout();
    s.lobby.difficulty = 5;
    let t = s.add_unit_type(crate::UnitType::default());
    s.tables.unit_types[t].cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    s.tables.unit_types[t].mana = 1000;
    // `TypeIndex` `0x36`, the General: the crafts' `FROM`.
    s.tables.unit_types[t].tree = Some(0x36);
    let mut u = crate::Unit::new(1, 1, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
    u.ty = Some(t);
    u.captain = true;
    let g = s.add_unit(u);
    s.add_move_order(
        g,
        crate::Pos::new(8 * 768, 8 * 768),
        MoveKind::MoveTo,
        QueuePos::New,
        false,
    );
    s.units[g].movement.body.avg_speed = 5;
    let mut rows = vec![SpellType::default(); 55];
    let row = |x: i32| (x - spell::FIRST) as usize;
    for (craft, mana) in [
        (spell::FORCED_MARCH, 500),
        (spell::AMBUSH, 500),
        (spell::CREATE_DECOY, 1000),
    ] {
        rows[row(craft)] = SpellType {
            mana,
            job_time: 10,
            from: [Some(0x36), None],
            ..Default::default()
        };
    }
    s.orders.spells = rows;
    let seed = |odd: i32| {
        (1..)
            .find(|&k| crate::combat::Rng::new(k).roll() & 1 == odd)
            .unwrap()
    };
    (s, g, [seed(0), seed(1)])
}

/// run470's frame 8856 (`docs/AI.md` §99.13): a moving hero throws
/// the coin at `+0x589`; an odd one lays Forced March first, an even
/// one reads the cell's owner — its own leader's, or no one's, and
/// nothing is cast; an enemy's, and Ambush is.
#[test]
fn a_moving_general_throws_the_hero_coin() {
    use crate::orders::{Body, spell};
    use crate::world::Owner;
    let cast = |sim: &Sim, g: usize| match sim.units[g].orders.front().map(|o| o.body) {
        Some(Body::Cast(c)) => Some(c.spell),
        _ => None,
    };
    let (mut sim, g, [even, odd]) = ai_general();
    sim.rng = crate::combat::Rng::new(odd);
    let mut probe = sim.rng;
    probe.roll();
    assert!(sim.think_spellcaster(g));
    assert_eq!(sim.rng, probe, "one draw");
    assert_eq!(cast(&sim, g), Some(spell::FORCED_MARCH));

    let (mut sim, g, _) = ai_general();
    sim.rng = crate::combat::Rng::new(even);
    assert!(!sim.think_spellcaster(g), "no one's cell");
    assert_ne!(
        sim.rng,
        crate::combat::Rng::new(even),
        "the coin was thrown"
    );
    sim.world
        .set_owner(Cell::new(5, 5), Owner::Player(1), Owner::None);
    sim.rng = crate::combat::Rng::new(even);
    assert!(!sim.think_spellcaster(g), "its own leader's cell");
    sim.world
        .set_owner(Cell::new(5, 5), Owner::Player(0), Owner::None);
    sim.rng = crate::combat::Rng::new(even);
    assert!(sim.think_spellcaster(g), "an enemy's cell");
    assert_eq!(cast(&sim, g), Some(spell::AMBUSH));

    // Short of the craft's mana, nothing is cast after the coin.
    let (mut sim, g, _) = ai_general();
    sim.units[g].mana_burn = 501;
    sim.rng = crate::combat::Rng::new(odd);
    assert!(!sim.think_spellcaster(g));
    assert_ne!(sim.rng, crate::combat::Rng::new(odd));
}

/// run470's 9112 and 9240: Forced March, cast, raises the leader's
/// `0x8000` and the hero's until `DURATION` runs out — no coin on the
/// turn between — and a hero's craft of its own draws no animation
/// roll (`set_anim(CHAR_ATTACK2, 0, 0)`).
#[test]
fn a_forced_march_holds_the_coin_until_it_runs_out() {
    use crate::orders::{Body, spell};
    let (mut sim, g, [_, odd]) = ai_general();
    let row = (spell::FORCED_MARCH - spell::FIRST) as usize;
    sim.orders.spells[row].job_time = 0;
    sim.orders.spells[row].duration = 150;
    sim.rng = crate::combat::Rng::new(odd);
    assert!(sim.think_spellcaster(g));
    let Some(Body::Cast(c)) = sim.units[g].orders.front().map(|o| o.body) else {
        panic!("a cast order");
    };
    let before = sim.rng;
    sim.do_cast(g, c);
    assert_eq!(sim.rng, before, "the attack slot rolls nothing");
    assert!(sim.leaders.forced_march[1]);
    assert_eq!(sim.units[g].marching, Some(sim.frame + 150));
    // The next turn: moving again, no coin while the march stands.
    let to = crate::Pos::new(8 * 768, 8 * 768);
    sim.add_move_order(
        g,
        to,
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    let before = sim.rng;
    assert!(!sim.think_spellcaster(g));
    assert_eq!(sim.rng, before, "no coin while marching");
    // Past the end frame the spell runs out and the flag with it.
    let end = sim.units[g].marching.unwrap();
    sim.process_spells(g, end);
    assert!(sim.leaders.forced_march[1], "held through its end frame");
    sim.process_spells(g, end + 1);
    assert!(!sim.leaders.forced_march[1] && sim.units[g].marching.is_none());
}

/// run511's block 9318 (`docs/AI.md` §99.15): under a Forced March
/// every land unit of the hero's leader within the hero's radius —
/// `GENERAL_RADIUS × 3 / 2` tiles at no upgrade, an economic patriot's
/// one more — walks at `FORCED_MARCH_SPEED`, 42, unless it is faster
/// already; one outside it, one of the sea, another leader's and every
/// unit once the march runs out keep their own.
#[test]
fn a_forced_march_lifts_the_units_near_its_hero() {
    let (mut sim, g, _) = ai_general();
    let row = (crate::orders::spell::FORCED_MARCH - crate::orders::spell::FIRST) as usize;
    sim.orders.spells[row].duration = 150;
    let at = sim.units[g].pos;
    let t = sim.add_unit_type(crate::UnitType::default());
    let mut o = 40;
    let mut near = |dx: i32, who: crate::Player, speed: i32| {
        o += 1;
        let mut u = crate::Unit::new(who, o, crate::Pos::new(at.x + dx, at.y), 20);
        u.ty = Some(t);
        u.movement.speed = speed;
        sim.add_unit(u)
    };
    // Nine tiles, 1,728 position units, is the General's reach.
    let (a, edge, out, fast, theirs) = (
        near(768, 1, 25),
        near(1728, 1, 25),
        near(1729, 1, 25),
        near(0, 1, 60),
        near(64, 0, 25),
    );
    assert_eq!(sim.unit_speed(a), 25, "no march yet");
    sim.cast_march(g);
    assert_eq!(sim.unit_speed(a), 42);
    assert_eq!(sim.unit_speed(edge), 42, "the radius is inclusive");
    assert_eq!(sim.unit_speed(out), 25, "one past it");
    assert_eq!(sim.unit_speed(fast), 60, "a march never slows a unit");
    assert_eq!(sim.unit_speed(theirs), 25, "another leader's");
    sim.tables.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
    sim.units[a].kind.domain = crate::attrition::Domain::Sea;
    assert_eq!(sim.unit_speed(a), 25, "a type not of the land");
    sim.tables.unit_types[t].combat.domain = crate::attrition::Domain::Land;
    sim.units[a].kind.domain = crate::attrition::Domain::Land;
    // The Senator's reach: an economic patriot's tile more.
    sim.units[g].type_index = 0x161;
    assert_eq!(sim.unit_speed(out), 42, "ten tiles for a Senator");
    let end = sim.units[g].marching.unwrap();
    sim.process_spells(g, end + 1);
    assert_eq!(sim.unit_speed(a), 25, "the march has run out");
}

/// run1013c's block 13484 (item 3390, `docs/AI.md` §294): a siege unit
/// within a Monarch's radius walks at twice its speed —
/// `UnitData::get_speed@00608720`'s `has_general(0, 0x162)` and
/// `is_siege` — and the group cap still applies after it; a unit not of
/// siege, one past the radius, another leader's and a siege unit beside
/// a hero who is no Monarch keep their own.
#[test]
fn a_siege_unit_near_its_monarch_walks_at_double() {
    let (mut sim, g, _) = ai_general();
    let at = sim.units[g].pos;
    let siege = sim.add_unit_type(crate::UnitType::default());
    sim.tables.unit_types[siege].combat.siege = true;
    let foot = sim.add_unit_type(crate::UnitType::default());
    sim.units[g].type_index = 0x162;
    let reach = sim.hero_radius(g) * 0xc0;
    let mut o = 40;
    let mut near = |dx: i32, who: crate::Player, t: usize| {
        o += 1;
        let mut u = crate::Unit::new(who, o, crate::Pos::new(at.x + dx, at.y), 20);
        u.ty = Some(t);
        u.movement.speed = 24;
        sim.add_unit(u)
    };
    let (a, edge, out, walker, theirs) = (
        near(768, 1, siege),
        near(reach, 1, siege),
        near(reach + 1, 1, siege),
        near(768, 1, foot),
        near(768, 0, siege),
    );
    assert_eq!(sim.get_speed(a, 1), 48);
    assert_eq!(sim.get_speed(edge, 1), 48, "the radius is inclusive");
    assert_eq!(sim.get_speed(out, 1), 24, "one past it");
    assert_eq!(sim.get_speed(walker, 1), 24, "not of siege");
    assert_eq!(sim.get_speed(theirs, 1), 24, "another leader's");
    sim.units[g].type_index = 0x160;
    assert_eq!(sim.get_speed(a, 1), 24, "a Despot is no Monarch");
}

/// Before the coin: a guy standing (`avg_speed` 0) returns 0 with no
/// draw, and a hero not moving takes the decoy arm — no draw, Create
/// Decoys with the mana, and nothing while one of its leader's decoy
/// captains stands.
#[test]
fn a_general_standing_draws_nothing_and_a_still_one_makes_decoys() {
    use crate::orders::{Body, spell};
    let (mut sim, g, _) = ai_general();
    sim.units[g].movement.body.avg_speed = 0;
    let before = sim.rng;
    assert!(!sim.think_spellcaster(g));
    assert_eq!(sim.rng, before, "avg_speed 0: no draw");

    let (mut sim, g, _) = ai_general();
    sim.kill_current_order(g);
    let before = sim.rng;
    assert!(sim.think_spellcaster(g));
    assert_eq!(sim.rng, before, "the decoy arm draws nothing");
    assert!(matches!(
        sim.units[g].orders.front().map(|o| o.body),
        Some(Body::Cast(c)) if c.spell == spell::CREATE_DECOY
    ));

    let (mut sim, g, _) = ai_general();
    sim.kill_current_order(g);
    let mut d = crate::Unit::new(1, 2, crate::Pos::new(2 * 768, 2 * 768), 20);
    d.captain = true;
    d.decoy = true;
    sim.add_unit(d);
    assert!(!sim.think_spellcaster(g), "a decoy captain stands");
}

/// A human's scout never reaches the computer's arm, and a decoy
/// returns before the coin.
#[test]
fn a_human_s_scout_and_a_decoy_throw_nothing() {
    let (mut sim, scout) = ai_scout();
    sim.lobby.difficulty = 5;
    sim.units[scout].decoy = true;
    let before = sim.rng;
    assert!(!sim.think_spellcaster(scout));
    assert_eq!(sim.rng, before, "a decoy");
    sim.units[scout].decoy = false;
    sim.cities_state.nation[1].human = true;
    assert!(!sim.think_spellcaster(scout));
    assert_eq!(sim.rng, before, "a human's");
}
