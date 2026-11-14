use super::*;
use crate::world::{Cell, Terrain, World};

/// A two-player sim on a 60×60 all-land map (row 0 a sea, so region 1 is
/// the land): `land_size` 3540 once the holdings are counted.
fn sim() -> Sim {
    let mut w = World::new(60, 60);
    let sea = w.add_region(Terrain::Sea);
    let land = w.add_region(Terrain::Land);
    for y in 0..60 {
        for x in 0..60 {
            w.set_region(Cell::new(x, y), if y == 0 { sea } else { land });
        }
    }
    Sim::new(crate::Tuning::RON, w, 2)
}

/// Every leaf named once, each answered by [`Score::leaf`].
#[test]
fn every_leaf_is_answered_once() {
    let s = Score {
        score: 1,
        explored: 2,
        territory: 3,
        units: 4,
        units_2: 5,
        buildings: 6,
        economy: 7,
        pop: 8,
        upgrades: 9,
        research: 10,
        wonders: 11,
        combat: 12,
        dirty_units: false,
    };
    let got: Vec<i32> = SCORE_LEAVES.iter().map(|n| s.leaf(n).unwrap()).collect();
    assert_eq!(got, (1..=12).collect::<Vec<_>>());
    assert_eq!(s.leaf("tribe"), None);
}

/// **The recompute frame** (§18.2): frame 0, or `(who + frame) % 10 ==
/// 0`; any other frame changes nothing. The territory part is
/// `territory × 1000 / land_size`, truncating, and the sum takes it.
#[test]
fn the_score_recomputes_on_the_leader_s_frame_only() {
    let mut s = sim();
    s.economy.holdings[1].territory = 266;
    s.economy.holdings[1].land_size = 3060;
    s.frame = 91;
    s.compute_score(1, false);
    assert_eq!(s.leader_score(1).territory, 0, "91 is not who 1's frame");
    s.frame = 89;
    s.compute_score(1, false);
    // 266·1000/3060 = 86.9 → 86.
    assert_eq!(s.leader_score(1).territory, 86);
    assert_eq!(s.leader_score(1).score, 86);
    s.economy.holdings[1].territory = 300;
    s.frame = 90;
    s.compute_score(1, false);
    assert_eq!(s.leader_score(1).territory, 86, "90 is who 0's frame");
    s.compute_score(1, true);
    assert_eq!(s.leader_score(1).territory, 98, "forced");
}

/// **The combat arm's ally rule** (§18.2): a kill of a rival adds a fifth
/// of the kill value; a kill of an ally (the killer itself included)
/// takes it off, floored at 0. The spells add with no test.
#[test]
fn an_ally_s_kill_comes_off_floored_at_zero() {
    let mut s = sim();
    s.tables.unit_types = vec![crate::UnitType::default(); 4];
    s.tables.unit_types[1].price.base = [20, 30, 0, 0, 0, 0];
    // UCF 10: value 500, the owner holding none: kill 500, v 100.
    s.leaders.at_war[0][1] = true;
    s.leaders.at_war[1][0] = true;
    s.note_score_kill(0, Victim::Unit(1), 1, true);
    assert_eq!(s.leader_score(0).combat, 100);
    s.note_score_kill(0, Victim::Unit(1), 0, true);
    assert_eq!(s.leader_score(0).combat, 0, "its own: off");
    s.note_score_kill(0, Victim::Unit(1), 0, true);
    assert_eq!(s.leader_score(0).combat, 0, "floored at 0");
    s.note_score_kill(0, Victim::Unit(1), 0, false);
    assert_eq!(s.leader_score(0).combat, 100, "a spell adds");
}

/// **The kill is priced as the owner's next one** (§18.2): the support
/// words times the owner's count of the type, its queue included.
#[test]
fn the_kill_value_counts_the_owner_s_units_and_queue() {
    let mut s = sim();
    s.tables.unit_types = vec![crate::UnitType::default(); 4];
    s.tables.unit_types[2].price.base = [50, 0, 0, 0, 0, 0];
    s.tables.unit_types[2].price.support = [Some((crate::economy::Resource::Food, 3)), None];
    s.roster.muster[1].by_type = vec![0, 0, 4, 0];
    s.roster.muster[1].queued_by_type = vec![0, 0, 1, 0];
    // 500 + 3·5 = 515, /5 = 103.
    s.leaders.at_war[0][1] = true;
    s.leaders.at_war[1][0] = true;
    s.note_score_kill(0, Victim::Unit(2), 1, true);
    assert_eq!(s.leader_score(0).combat, 103);
}

/// **The units part waits on its bit** (§18.2): a recompute with neither
/// dirty bit keeps the last value; a unit's birth raises it, and the
/// recompute clears it. `u = (value·n + ((n−1)·sc·n)/2) / 5`.
#[test]
fn the_units_part_waits_on_its_dirty_bit() {
    let mut s = sim();
    s.tables.unit_types = vec![crate::UnitType::default(); 4];
    s.tables.unit_types[0].price.base = [50, 0, 0, 0, 0, 0];
    s.tables.unit_types[0].price.support = [Some((crate::economy::Resource::Food, 2)), None];
    s.roster.muster[0].by_type = vec![3, 0, 0, 0];
    s.roster.muster[0].queued_by_type = vec![0, 0, 0, 0];
    s.frame = 10;
    s.compute_score(0, false);
    assert_eq!(s.leader_score(0).units, 0, "no bit, no recount");
    s.score_units_dirty(0);
    s.compute_score(0, false);
    // (500·3 + (2·2·3)/2)/5 = (1500 + 6)/5 = 301.
    assert_eq!(s.leader_score(0).units, 301);
    assert!(!s.leader_score(0).dirty_units);
    s.roster.muster[0].by_type = vec![4, 0, 0, 0];
    s.frame = 20;
    s.compute_score(0, false);
    assert_eq!(s.leader_score(0).units, 301, "cleared bit: kept");
    s.tech[0].score_dirty = true;
    s.compute_score(0, false);
    // (2000 + (3·2·4)/2)/5 = 2012/5 = 402.
    assert_eq!(s.leader_score(0).units, 402, "the tech bit also recounts");
    assert!(!s.tech[0].score_dirty);
    s.score_units_dirty(8);
}

/// **A sunk transport is priced as its first passenger** (§18.2,
/// `take_damage@00652020:520`-`:528`): a victim whose type is a transport
/// (`unit_flags & 0x10`) is redirected to the head of its `inside_down`,
/// that unit's type and owner; an empty one scores nothing. Any other
/// victim is itself. run1496's airbase melee: each computer sank the
/// other's scripted, empty Transport Freighter, which the original's
/// `score_combat` never took (ours 96 and 415 against 84 and 403).
#[test]
fn a_sunk_transport_is_priced_as_its_first_passenger() {
    use crate::ai::ai_load::{UnitCols, uflags};
    let mut s = sim();
    let ty = |s: &mut Sim, unit_flags: u32, cost: i32| {
        let mut t = crate::UnitType {
            cols: UnitCols {
                unit_flags,
                ..UnitCols::default()
            },
            ..crate::UnitType::default()
        };
        t.price.base = [cost, 0, 0, 0, 0, 0];
        s.add_unit_type(t)
    };
    let freighter = ty(&mut s, uflags::FLAG_TRANSPORT, 6);
    let rifle = ty(&mut s, 0, 9);
    let at = crate::world::Pos::new(30 * 0x300, 30 * 0x300);
    let put = |s: &mut Sim, who: Player, t: usize, o: i16| {
        let mut u = crate::Unit::new(who, o, at, 100);
        u.ty = Some(t);
        u.captain = true;
        s.add_unit(u)
    };
    let boat = put(&mut s, 1, freighter, 5);
    assert_eq!(s.score_victim(boat), None, "empty: no kill on the score");
    let man = put(&mut s, 0, rifle, 6);
    assert_eq!(s.score_victim(man), Some((rifle, 0)), "a rifle is itself");
    s.units[man].inside_unit = Some(boat);
    assert_eq!(
        s.score_victim(boat),
        Some((rifle, 0)),
        "the passenger's type and owner"
    );
}
