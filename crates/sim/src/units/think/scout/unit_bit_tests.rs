/// **A defeated leader's units lose the computer-control bit** (group 6;
/// A4 row 19): the leader-level answer stays.
#[test]
fn a_defeat_clears_the_units_computer_bit() {
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    s.cities_state.nation[1].human = false;
    assert!(s.unit_ai_bit(1));
    s.leaders.defeated[1] = true;
    assert!(!s.unit_ai_bit(1), "the units' bit is cleared");
    assert!(s.ai_driven(1), "the leader is still the computer's");
}
