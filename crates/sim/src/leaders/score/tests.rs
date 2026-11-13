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
