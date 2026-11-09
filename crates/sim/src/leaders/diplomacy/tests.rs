use crate::{Sim, Tuning, World};

/// The peace pair's shape (`docs/AI.md` §220): who=1 a computer at
/// difficulty 3, who=0 a human it has met, `TEAM_STYLE 1`, five
/// minutes' peace, no shared region and no buildings.
fn peace_pair() -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = false;
    sim.ai_state.lobby.team_style = 1;
    sim.ai_state.lobby.difficulty = 3;
    sim.ai_state.lobby.rush_rules = 10;
    sim.ai_state.lobby.rush_minutes = 5;
    sim.leaders.treaties[1][0] = 1;
    sim
}

/// who=1's view of who=0 is taken on 448, 960, 1472, 1984, … — the
/// period 512 while `agendas & 8` is clear, phased by `(t + who·8)·8`
/// — and on no other frame; a target not yet met is never visited.
#[test]
fn a_pair_is_viewed_once_a_period_on_its_own_phase() {
    let mut sim = peace_pair();
    let mut fired = Vec::new();
    for f in 0..2100 {
        sim.frame = f;
        let before = sim.ai_state.ai[0].got_diplo_message;
        sim.diplomacy(1);
        if sim.ai_state.ai[0].got_diplo_message != before || sim.ai_state.ai[1].strong[0] != 0 {
            fired.push(f);
            sim.ai_state.ai[0].got_diplo_message = 0;
            sim.ai_state.ai[1].strong[0] = 0;
            sim.leaders.agendas[1][0] = 0;
        }
    }
    assert_eq!(fired, [448, 960, 1472, 1984]);
    sim.leaders.treaties[1][0] = 0;
    sim.frame = 2496;
    sim.diplomacy(1);
    assert_eq!(
        sim.ai_state.ai[1].strong[0], 0,
        "an unmet target is skipped"
    );
}

/// Run849's block 1985: `aggression` 1 (difficulty > 1, one
/// non-ally), `strong` `1 << 5` (difficulty 3 against a human, no
/// building line), and with war not yet allowed the one message:
/// `agendas |= 0x40`, the receiver stamped with the frame. A second
/// view inside 450 frames of the stamp, or with `0x40` set, sends none.
#[test]
fn the_peace_arm_sends_one_message_and_stamps_the_receiver() {
    let mut sim = peace_pair();
    sim.frame = 1984;
    assert!(!sim.war_allowed(), "1984 is inside the five minutes");
    let seed = sim.rng.seed;
    sim.diplomacy(1);
    assert_ne!(sim.rng.seed, seed, "the peace coin draws");
    assert_eq!(sim.ai_state.ai[1].aggression[0], 1);
    assert_eq!(sim.ai_state.ai[1].strong[0], 1 << 5);
    assert_eq!(sim.ai_state.ai[1].weak[0], 0);
    assert_eq!(sim.leaders.agendas[1][0], 0x40);
    assert_eq!(sim.ai_state.ai[0].got_diplo_message, 1984);
    assert_eq!(
        sim.ai_state.ai[1].got_diplo_message, 0,
        "the stamp is the target's"
    );

    // Inside the quiet window, a cleared bit sends nothing.
    sim.leaders.agendas[1][0] = 0;
    sim.frame = 1984 + 512 - 64; // not on the phase: nothing at all
    sim.diplomacy(1);
    assert_eq!(sim.leaders.agendas[1][0], 0);
    sim.ai_state.ai[0].got_diplo_message = 1984 + 100;
    sim.frame = 2496;
    sim.diplomacy(1);
    assert_eq!(
        sim.leaders.agendas[1][0], 0,
        "the receiver heard one 412 frames ago"
    );

    // At difficulty 1 nothing raises `aggression` and nothing is sent.
    let mut sim = peace_pair();
    sim.ai_state.lobby.difficulty = 1;
    sim.frame = 1984;
    sim.diplomacy(1);
    assert_eq!(
        sim.ai_state.ai[1].aggression[0], 1,
        "team style 1 still sets it"
    );
    assert_eq!(sim.ai_state.ai[1].strong[0], 0, "no human bonus below 3");
}

/// Run855's block 4538: `diplos` 0 both ways between who=0 and
/// who=1, written on 4500 and on no other frame; an ally keeps its
/// peace, a rule of 8 or below declares nothing, and team style 0
/// leaves the declaring to `Leader::diplomacy`.
#[test]
fn the_peace_s_end_declares_war_both_ways_on_its_one_frame() {
    let mut sim = peace_pair();
    for f in [4499, 4501] {
        sim.frame = f;
        sim.end_the_peace();
        assert!(
            !sim.leaders.at_war[0][1] && !sim.leaders.at_war[1][0],
            "frame {f}"
        );
    }
    sim.frame = 4500;
    sim.end_the_peace();
    assert!(sim.leaders.at_war[0][1] && sim.leaders.at_war[1][0]);
    assert_eq!((sim.diplos(0, 1), sim.diplos(1, 0)), (0, 0));

    let mut sim = peace_pair();
    sim.leaders.allied[0][1] = true;
    sim.leaders.allied[1][0] = true;
    sim.frame = 4500;
    sim.end_the_peace();
    assert!(
        !sim.leaders.at_war[0][1] && !sim.leaders.at_war[1][0],
        "allies keep it"
    );

    let mut sim = peace_pair();
    sim.leaders.allied[0][1] = true; // one way only: not `is_ally`
    sim.frame = 4500;
    sim.end_the_peace();
    assert!(sim.leaders.at_war[0][1] && sim.leaders.at_war[1][0]);
    assert_eq!(sim.diplos(1, 0), 0);

    let mut sim = peace_pair();
    sim.ai_state.lobby.rush_rules = 8;
    sim.frame = 4500;
    sim.end_the_peace();
    assert!(!sim.leaders.at_war[0][1], "an age rule ends no clock");

    let mut sim = peace_pair();
    sim.ai_state.lobby.team_style = 0;
    sim.frame = 4500;
    sim.end_the_peace();
    assert!(!sim.leaders.at_war[0][1], "style 0 declares in `diplomacy`");
}

/// The war gate: a timed rule's minutes × 900, or an age at the rule.
#[test]
fn war_is_allowed_after_the_rush_minutes() {
    let mut sim = peace_pair();
    sim.frame = 4499;
    assert!(!sim.war_allowed());
    sim.frame = 4500;
    assert!(sim.war_allowed());
    sim.ai_state.lobby.rush_rules = 0;
    sim.frame = 0;
    assert!(sim.war_allowed());
}
