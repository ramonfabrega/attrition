use super::*;
use crate::ai_load::{role, uflags2};

/// `peasants_wait` is an **index**, and the default index is not the
/// default arm (item 494).
///
/// The table is `think_peasant@005f5760:16` and
/// `think_caravan@005f5650:10`, which carry it twice verbatim. What
/// the row below is worth asserting for is the one line a reader
/// gets wrong: `LeaderOptions::init` writes `peasants_wait = 2` and
/// every capture on disk prints `peasants_wait 2`, so the wait every
/// human in every run measured has had is **12** — while the
/// switch's `default:` arm, which is what an out-of-range index
/// falls to and what this crate used to return for all of them, is
/// **2**. The two numbers are the same digit in different roles.
#[test]
fn peasants_wait_is_an_index_into_five_waits() {
    let wait = |n: i32| {
        LeaderOptions {
            peasants_wait: n,
            ..LeaderOptions::default()
        }
        .idle_wait()
    };
    assert_eq!(
        (wait(1), wait(2), wait(3), wait(4), wait(5)),
        (7, 12, 17, 32, 62),
        "the five arms of `LeaderOptions +0x8`'s switch"
    );
    // Out of range, both ends, is the `default:` arm.
    assert_eq!((wait(0), wait(6), wait(-1)), (2, 2, 2));
    // And the default option is arm 2, not the default arm.
    assert_eq!(LeaderOptions::default().idle_wait(), 12);
}

/// `UnitTypeData::get_stance_type@0061d350`, arm by arm.
///
/// The two that are easy to get backwards: a **packing military** type is
/// a packer and not a combat unit, and a **non-packing military** type
/// falls out of the same arm as `Combat` (0) rather than `None` (−1) —
/// the decompiler writes that return as `-(uint)(flag) & STANCE_PACKER`,
/// which is a mask and not a conditional.
#[test]
fn get_stance_type_tests_military_before_everything_else() {
    // A citizen: `type_index` 0x32, no role bits.
    assert_eq!(stance_type(0, 0, 0x32), StanceType::Worker);
    assert_eq!(stance_type(0, 0, 0x35), StanceType::Worker);
    assert_eq!(stance_type(0, 0, 0x36), StanceType::None);
    assert_eq!(stance_type(0, 0, 0x31), StanceType::None);
    // Military wins over the worker indices, though nothing is both.
    assert_eq!(stance_type(role::MILITARY, 0, 0x32), StanceType::Combat);
    assert_eq!(
        stance_type(role::MILITARY, uflags2::PACKS, 0x60),
        StanceType::Packer
    );
    assert_eq!(stance_type(role::MILITARY, 0, 0x60), StanceType::Combat);
    // `(unit_flags2 & 6) == 2` — a caster that does not also pack. A
    // civilian type that packs is neither, because the packer arm is
    // military-only.
    assert_eq!(stance_type(0, uflags2::CASTER, 0x45), StanceType::Caster);
    assert_eq!(
        stance_type(0, uflags2::CASTER | uflags2::PACKS, 0x45),
        StanceType::None
    );
    assert_eq!(stance_type(0, uflags2::PACKS, 0x45), StanceType::None);
}

/// `LeaderOptions::init@006f1d40` — the defaults every capture runs under,
/// and what each of `Unit::init`'s five arms makes of them.
///
/// The caster/packer pair is the one worth pinning: both read the same
/// bitmask byte, both **complement** it, and `init` sets bit 3 and not
/// bit 4 — so a packer is born at 0 and a caster at 1 out of a single
/// byte's two adjacent bits.
#[test]
fn init_s_defaults_give_a_caster_one_and_a_packer_zero() {
    let o = LeaderOptions::default();
    assert_eq!(o.peasants, 0);
    assert_eq!(o.peasants_wait, 2);
    assert_eq!(o.buildings, 0);
    assert!(o.bit(1) && o.bit(3), "init sets bits 1 and 3");
    assert!(!o.bit(4), "and leaves bit 4 clear");

    assert_eq!(init_stance(StanceType::Caster, o, true, false), 1);
    assert_eq!(init_stance(StanceType::Packer, o, true, false), 0);
    assert_eq!(init_stance(StanceType::Combat, o, true, false), 0);
    assert_eq!(init_stance(StanceType::None, o, true, false), 0);
}

/// The worker arm, which is the one this crate had wrong — and it is
/// **inverted** from what the field name suggests. `leader_flags & 4` is
/// the human bit, and `Unit::init` takes `leader_options.peasants` when
/// it is *set*; the lobby's `(starting_resources == 8) + 1` is the
/// **AI's** branch.
///
/// This is the table two captures print at their first block, run68 (East
/// Indies) and run69 (Great Lakes) alike: the human's five citizens at 0,
/// the AI's five at 1, both scouts at 1, and every gaia object at 0.
#[test]
fn a_human_s_starting_citizen_is_born_at_zero_and_an_ai_s_at_one() {
    let o = LeaderOptions::default();
    assert_eq!(init_stance(StanceType::Worker, o, true, false), 0);
    assert_eq!(init_stance(StanceType::Worker, o, false, false), 1);
    // `starting_resources == 8` is the unlimited lobby; every capture so
    // far runs at 1, so this arm is a reading and no run reaches it.
    assert_eq!(init_stance(StanceType::Worker, o, false, true), 2);
    // A human's is their own option whatever the lobby says.
    assert_eq!(init_stance(StanceType::Worker, o, true, true), 0);
    let picked = LeaderOptions {
        peasants: 2,
        ..LeaderOptions::default()
    };
    assert_eq!(init_stance(StanceType::Worker, picked, true, false), 2);
}

/// `Build::init`'s worker arm against `Unit::init`'s, which is the whole
/// reason the two functions are both here.
///
/// They agree in every lobby but the unlimited one, and they disagree
/// **for the AI** — which is what makes run68's frame 6595 legible: the
/// AI's `1/1..1/5`, born at the start, carry 1; its `1/6` upward, every
/// one of them trained out of a city, carry the city's 0.
#[test]
fn a_building_s_worker_arm_is_not_a_unit_s() {
    let o = LeaderOptions::default();
    for human in [false, true] {
        assert_eq!(
            build_init_stance(StanceType::Worker, o, human, false),
            o.peasants as u8,
            "an ordinary lobby: a building takes the option either way"
        );
    }
    assert_eq!(build_init_stance(StanceType::Worker, o, true, true), 0);
    assert_eq!(build_init_stance(StanceType::Worker, o, false, true), 2);
    // The caster arm is a flat 1 here and a complemented bit there; the
    // default bitmask makes them agree, so only a set bit 4 tells them
    // apart.
    let bit4 = LeaderOptions {
        flags: LeaderOptions::default().flags | 0b1_0000,
        ..LeaderOptions::default()
    };
    assert_eq!(build_init_stance(StanceType::Caster, bit4, true, false), 1);
    assert_eq!(init_stance(StanceType::Caster, bit4, true, false), 0);
}

/// **A trained combat unit takes the trainer's byte whole** (item 3367,
/// `Build::train@0062f9b0:86–101` → `Unit::set_stance@00605310`): the
/// byte `+0xb1` is the combat stance `get_combat_stance` reads, so the
/// copy this crate keeps in `combat.stance` follows it. A computer's
/// Bark out of a Dock printed `stance 0` in run993c and fought RAID
/// here. Made to fail with the `combat.stance` write dropped from
/// `build_train` (Defensive's byte, Aggressive's stance).
#[test]
fn a_trained_combat_unit_takes_the_trainer_s_byte_as_its_combat_stance() {
    let mut s = crate::Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(40, 40),
        2,
    );
    s.leader_options = vec![
        LeaderOptions {
            buildings: 1,
            ..LeaderOptions::default()
        };
        2
    ];
    let mut t = crate::UnitType::default();
    t.cols.role = role::MILITARY;
    let ty = s.add_unit_type(t);
    assert_eq!(s.type_stance_type(ty), StanceType::Combat);
    let barracks = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Barracks,
        flags: crate::build::flags::TRAINS,
        ..crate::build::BuildType::default()
    });
    let b = s.add_building(0, crate::Pos::new(5000, 5000), 0);
    s.buildings[b].ty = Some(barracks);
    assert_eq!(s.trained_stance(ty, b), Some(s.build_stance(b)));
    let u = s.build_train(b, ty).unit;
    let byte = s.units[u].stance;
    assert_eq!(byte, s.build_stance(b), "the trainer's byte");
    assert_eq!(
        Some(s.units[u].combat.stance),
        crate::group::combat_stance_of(byte),
        "and the combat stance it reads as"
    );
}
