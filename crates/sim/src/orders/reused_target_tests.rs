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
