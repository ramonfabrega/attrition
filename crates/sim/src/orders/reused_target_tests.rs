use super::*;
use crate::combat::Profile;
use crate::world::World;

/// **An attack whose target's number went to another object is cleared,
/// and `work` returns** (`Unit::work@0060d180:300`–`:340`, item 4662,
/// `docs/RECGAME.md` §14.4). The in-danger mark goes to the object now
/// holding the order's `(whom, ox)`; the target's `uid` against that
/// object's is a mismatch, so the current order keeps its `uid` with `ox`
/// and `whom` −1 and nothing is searched that frame. On the next, `fight`'s
/// `valid_target` refuses the cleared pair and the search takes the foe in
/// reach. Row 88's Trebuchet `1/61` on 2984, its Attack on `2/63` at uid 93
/// after another barge took the number: theirs prints `(−1, −1, 93)` on
/// 2985 and the new order on 2986.
///
/// Made to fail by dropping the uid test: ours searches on the first frame
/// and takes the near foe there.
#[test]
fn an_attack_on_a_reused_number_is_cleared_and_searches_a_frame_later() {
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut Sim, who: Player, index: i16, p: Pos| {
        let mut u = crate::Unit::new(who, index, p, 100);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.tables.unit_types[ty].kind;
        sim.add_unit(u)
    };
    let c = 30 * 0x300 + 0x198;
    let me = put(&mut sim, 1, 1, Pos::new(c, c));
    let old = put(&mut sim, 0, 5, Pos::new(c, c + 6 * 0xc0));
    let near = put(&mut sim, 0, 6, Pos::new(c + 3 * 0xc0, c));
    sim.units[old].uid = 93;
    sim.add_attack_order(me, Obj::Unit(old), QueuePos::First, false, true);
    sim.units[old].health = 0;
    // The number goes to a newcomer far off, with its own uid.
    let fresh = put(&mut sim, 0, 5, Pos::new(c - 20 * 0xc0, c));
    sim.units[fresh].uid = 140;
    // Off the review's phase, `(frame + o) % 16 != 0`.
    let frame = 17 - i64::from(sim.units[me].index);
    sim.work(me, frame);
    let Some(Order {
        body: Body::Attack(a),
        ..
    }) = sim.current_order(me).copied()
    else {
        panic!("the attack stands: {:?}", sim.units[me].orders);
    };
    assert_eq!(
        (a.target, a.cleared_uid),
        (None, Some(93)),
        "ox and whom are cleared and the uid is kept"
    );
    assert_eq!(sim.units[me].combat.target, None);
    assert!(
        sim.units[fresh].in_danger,
        "the object now holding the number is marked"
    );
    let target = sim
        .checksum_units()
        .ranges
        .into_iter()
        .find(|r| r.field == "order.target")
        .map(|r| r.bytes);
    assert_eq!(
        target,
        Some(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 93, 0])
    );
    sim.work(me, frame + 1);
    assert_eq!(
        sim.units[me].combat.target,
        Some(Obj::Unit(near)),
        "fight refused the cleared pair and searched"
    );
}

/// **The chase review asks the object now holding the attack's `(whom,
/// ox)`** (`Unit::check_target_path@005e22d0`, item 4732, `docs/RECGAME.md`
/// §12.3). A chase on a target that died is reviewed through the corpse's
/// arms — not active, on the map, out of reach from the walk — and re-aimed:
/// the move goes and `find_attack_pos` issues a new one. Once the number
/// has gone to a newcomer, the review reads the newcomer instead: a live
/// unit standing out of plain reach is a `return 0`, and the move stands
/// untouched. Row 83's Rifleman `2/114` on 3774, chasing the dead Light
/// Tank `1/38` after who=1's Albatross took the number on 3773: theirs
/// keeps its move to (11928, 27000), path and all.
///
/// Made to fail by reviewing the combat target (the corpse) again: the move
/// is replaced by one to a fresh spot.
#[test]
fn the_chase_review_asks_the_object_now_holding_the_number() {
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut Sim, who: Player, index: i16, p: Pos| {
        let mut u = crate::Unit::new(who, index, p, 100);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.tables.unit_types[ty].kind;
        sim.add_unit(u)
    };
    let c = 30 * 0x300 + 0x198;
    let walk = Pos::new(c + 2 * 0xc0, c + 2 * 0xc0);
    let stage = |reused: bool| {
        let mut sim = sim.clone();
        let me = put(&mut sim, 1, 1, Pos::new(c, c));
        let old = put(&mut sim, 0, 5, Pos::new(c, c + 12 * 0xc0));
        sim.units[old].uid = 93;
        sim.add_attack_order(me, Obj::Unit(old), QueuePos::First, false, true);
        sim.add_move_order(me, walk, MoveKind::MoveTo, QueuePos::First, false);
        sim.units[old].health = 0;
        if reused {
            let fresh = put(&mut sim, 0, 5, Pos::new(c - 20 * 0xc0, c));
            sim.units[fresh].uid = 140;
        }
        assert!(
            matches!(sim.current_order(me).map(|o| o.body), Some(Body::Move(_))),
            "a chase leg over the attack: {:?}",
            sim.units[me].orders
        );
        let changed = sim.check_target_path(me);
        let head = match sim.current_order(me).map(|o| o.body) {
            Some(Body::Move(m)) => Some(m.dest),
            _ => None,
        };
        (changed, head, sim.units[me].orders.len())
    };
    // The corpse's arm 8: `repath`, then `find_attack_pos` — here it finds
    // no spot, which is `return 1` with the legs gone.
    let (changed, head, _) = stage(false);
    assert!(changed, "the corpse's review re-aims");
    assert_ne!(head, Some(walk), "the leg to {walk:?} is gone");
    assert_eq!(
        stage(true),
        (false, Some(walk), 2),
        "the newcomer out of reach: the review returns 0 and the leg stands"
    );
}
