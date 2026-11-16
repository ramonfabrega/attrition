//! `ai_toggle`'s tests (item 4283).

/// The order flips the flag, and a second order flips it back: the
/// handler toggles and never sets.
#[test]
fn the_toggle_flips_the_flag_both_ways() {
    let mut s = crate::Sim::new(
        crate::core::tuning::Tuning::RON,
        crate::world::World::new(40, 40),
        2,
    );
    assert!(!s.ai.ai_off);
    s.command().process_cheat_ai_toggle();
    assert!(s.ai.ai_off);
    assert!(s.observe(None).ai_off, "the scene reads it");
    s.command().process_cheat_ai_toggle();
    assert!(!s.ai.ai_off);
}
