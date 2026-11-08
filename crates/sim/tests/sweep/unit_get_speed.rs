//! `UnitData::get_speed@006086f0` against this crate's [`Sim::get_speed`].

use crate::attrition::Domain;
use crate::orders::{AttackOrder, Body, GuardOrder, MoveKind, Order, QueuePos, flag};
use crate::world::Pos;
use crate::{Sim, Tuning, Unit, UnitType, World};

/// One order of the row's kind code: 1 a transit move, 2 a move that is
/// the action, 3 an attack, 4 a guard — the script's own coding.
fn order(s: &mut Sim, scratch: usize, kind: i64) -> Order {
    let at = Pos::new(0x2000, 0x2000);
    match kind {
        1 | 2 => {
            s.units[scratch].orders.clear();
            s.add_move_order(scratch, at, MoveKind::MoveTo, QueuePos::New, kind == 2);
            s.units[scratch].orders.pop_front().expect("the move")
        }
        3 => Order {
            flags: flag::ACTION,
            body: Body::Attack(AttackOrder {
                defensive: false,
                def: None,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        },
        4 => Order {
            flags: flag::ACTION,
            body: Body::Guard(GuardOrder {
                target: scratch,
                dx: -1,
                dy: -1,
                guard: at,
                idle: 0,
                retry: 0,
            }),
        },
        _ => unreachable!("order kind {kind}"),
    }
}

/// `UnitData::get_speed@006086f0` — the thunk that decodes the unit's own
/// point and calls vslot 0x17c, `UnitData::get_speed@00608720` — run under
/// unicorn by `tools/emu/sweep_unit_get_speed.py` and asserted against
/// [`Sim::get_speed`] row by row.
///
/// **The slice** is the non-land arm (`type +0x218 != 0`, a ship or a
/// plane) with `group = -1`: the land arm reads `worldc`, `has_general`
/// and the leaders and needs a packet, and the group cap reads the
/// `groups` global. What the rows do cover is the order scale (attack
/// 9/8, guard 9/8 or 10/8 for an AI unit), `get_action`'s walk over a
/// ring of up to three orders (transit moves skipped, the first action
/// taken), `myspeed` as a signed short, truncation toward zero, and the
/// floor of 3.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_get_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let [speed, domain, ai, o1, o2, o3, flag] = a[..] else {
            panic!("row `{line}`: seven arguments");
        };
        let domain_of = if domain == 1 {
            Domain::Sea
        } else {
            Domain::Air
        };
        let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
        let mut t = UnitType {
            hits: 1,
            moves: 34,
            ..UnitType::default()
        };
        t.combat.domain = domain_of;
        let ty = s.add_unit_type(t);
        let mk = |s: &mut Sim, who: i16| {
            let mut u = Unit::new(1, who, Pos::new(0x1000, 0x1000), 1);
            u.ty = Some(ty);
            u.kind.domain = domain_of;
            s.add_unit(u)
        };
        let scratch = mk(&mut s, 1);
        let u = mk(&mut s, 0);
        s.nation[1].human = ai == 0;
        s.units[u].movement.speed = speed as i32;
        for k in [o1, o2, o3].into_iter().filter(|&k| k != 0) {
            let o = order(&mut s, scratch, k);
            s.units[u].orders.push_back(o);
        }
        let got = i64::from(s.get_speed(u, flag as i32));
        assert_eq!(got, want, "row `{line}`: ours {got}, the original's {want}");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
}
