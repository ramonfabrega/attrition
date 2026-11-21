//! The `leader_flags` bits item 4400 emits (`docs/CHECKSUM.md` §23.8).

use crate::Sim;
use crate::world::{Cell, Terrain, World};

fn sim() -> Sim {
    let mut w = World::new(20, 20);
    let land = w.add_region(Terrain::Land);
    for y in 0..20 {
        for x in 0..20 {
            w.set_region(Cell::new(x, y), land);
        }
    }
    let mut s = Sim::new(crate::Tuning::RON, w, 2);
    s.tables.unit_types = vec![crate::UnitType::default(); 4];
    for m in &mut s.roster.muster {
        m.by_type = vec![0; 4];
        m.queued_by_type = vec![0; 4];
    }
    s
}

fn word(s: &Sim, who: crate::Player, mask: i64) -> i64 {
    s.leader_flag_words(who).0.unwrap() & mask
}

/// `Leader::track_unit_type`'s jammer arm: a `SPECIALFORCES` (0x4b) type's
/// own count decides `0x20000`, so the last move of any such type does —
/// one of two such types gone clears it while the other still stands.
/// Another lineage moves nothing.
#[test]
fn a_special_forces_count_decides_the_jammer_bit() {
    let mut s = sim();
    s.tables.unit_types[1].tree = Some(0x4b);
    s.tables.unit_types[2].tree = Some(0x4b);
    s.tables.unit_types[3].tree = Some(0x45);
    s.track_unit_type(0, 3, 1);
    assert_eq!(word(&s, 0, 0x2_0000), 0, "a Scout is not a jammer");
    s.track_unit_type(0, 1, 1);
    assert_eq!(word(&s, 0, 0x2_0000), 0x2_0000);
    assert_eq!(word(&s, 1, 0x2_0000), 0, "its owner's alone");
    s.track_unit_type(0, 2, 1);
    s.track_unit_type(0, 2, -1);
    assert_eq!(
        word(&s, 0, 0x2_0000),
        0,
        "type 2's count reached zero, and the arm reads its own"
    );
    s.track_unit_type(0, 1, 1);
    assert_eq!(word(&s, 0, 0x2_0000), 0x2_0000, "type 1 now holds two");
    s.track_unit_type(0, 1, -2);
    assert_eq!(word(&s, 0, 0x2_0000), 0);
}

/// `Leaders::process_all`'s head: `0x40000` when a seated other slot holds
/// the jammer bit and the two are not allied both ways; the tick derives it
/// at the live slot's step.
#[test]
fn an_enemy_jammer_raises_the_threat_bit() {
    let mut s = sim();
    s.tables.unit_types[1].tree = Some(0x4b);
    s.track_unit_type(1, 1, 1);
    s.refresh_jam_threat(0);
    assert_eq!(word(&s, 0, 0x4_0000), 0x4_0000, "an enemy holds one");
    s.refresh_jam_threat(1);
    assert_eq!(word(&s, 1, 0x4_0000), 0, "its own jammer is no threat");
    s.leaders.allied[0][1] = true;
    s.refresh_jam_threat(0);
    assert_eq!(word(&s, 0, 0x4_0000), 0x4_0000, "allied one way only");
    s.leaders.allied[1][0] = true;
    s.refresh_jam_threat(0);
    assert_eq!(word(&s, 0, 0x4_0000), 0, "allied both ways");
    s.leaders.allied[1][0] = false;
    s.tick();
    assert_eq!(word(&s, 0, 0x4_0000), 0x4_0000, "the tick derives it");
}

/// `Leader::check_transport`'s block: the level's three bits on the word,
/// and the scenario lock on `leader_flags2`.
#[test]
fn the_transport_level_rides_the_flags_word() {
    let mut s = sim();
    assert_eq!(word(&s, 0, 0x700), 0);
    s.ai.transport[0].set_all();
    assert_eq!(word(&s, 0, 0x700), 0x700);
    assert_eq!(word(&s, 1, 0x700), 0, "its leader's alone");
    s.ai.transport[0].locked = true;
    assert_eq!(s.leader_flag_words(0).1.unwrap() & 0x20, 0x20);
    s.ai.transport[0].clear();
    assert_eq!(word(&s, 0, 0x700), 0);
}

/// `Leader::defeat@006ecb00` past its guards (item 4521): `leader_flags`
/// trades the live bit `2` for the defeated mark `0x40`, and
/// `leader_flags2` takes `2`; the captor is `defeated_by`, a range of its
/// own. The other leader's words are untouched.
#[test]
fn a_defeat_marks_both_flag_words() {
    let mut s = sim();
    assert_eq!(word(&s, 1, 0x42), 2, "a live leader holds the live bit");
    assert_eq!(s.leader_flag_words(1).1.unwrap() & 2, 0);
    s.defeat(1, crate::leaders::endgame::DEFEAT_CONQUEST, Some(0));
    assert_eq!(
        word(&s, 1, 0x42),
        0x40,
        "the defeated mark, not the live bit"
    );
    assert_eq!(s.leader_flag_words(1).1.unwrap() & 0x202, 2, "param_3 is 0");
    assert_eq!(s.leader_endgame(1).defeated_by, 0);
    assert_eq!(word(&s, 0, 0x42), 2, "its leader's alone");
    assert_eq!(s.leader_flag_words(0).1.unwrap() & 2, 0);
}

/// `Leaders::process_all@006ed2a0`'s slot step is behind `leader_flags &
/// 2` whole (item 4521): a defeated leader's stale wall stats
/// (`0x8000000`) are not answered by the tick, a live one's are.
#[test]
fn a_defeated_leader_s_stats_bits_stay_raised() {
    let mut s = sim();
    s.defeat(1, crate::leaders::endgame::DEFEAT_CONQUEST, Some(0));
    s.cities.wall_stats_dirty[0] = true;
    s.cities.wall_stats_dirty[1] = true;
    s.roster.unit_stats_dirty[1] = true;
    s.tick();
    assert!(!s.cities.wall_stats_dirty[0], "the live slot's is answered");
    assert!(s.cities.wall_stats_dirty[1], "the defeated slot's stays");
    assert!(s.roster.unit_stats_dirty[1]);
    assert_eq!(word(&s, 1, 0x800_0000), 0x800_0000, "and rides the word");
}

/// A Forced March's bit (`leader_flags & 0x8000`) on the word.
#[test]
fn a_forced_march_rides_the_flags_word() {
    let mut s = sim();
    s.leaders.forced_march[1] = true;
    assert_eq!(word(&s, 1, 0x8000), 0x8000);
    assert_eq!(word(&s, 0, 0x8000), 0);
}
