use super::*;

/// A tree with the shape of the shipped one and a handful of its rows:
/// the seven ages, four lines of seven epochs, the four finals, a few
/// building techs, the governments, a Barracks and a Library and a Town,
/// the Hoplites → Phalanx → Pikemen line, a free Citizen, the Catapult →
/// Trebuchet line, and a machine gun.
struct Fixture {
    tree: TechTree,
    knowledge: TypeId,
    ages: [TypeId; LEVELS],
    epochs: [[TypeId; LEVELS]; 4],
    finals: [TypeId; 4],
    fortification: TypeId,
    herbal_lore: TypeId,
    medicine: TypeId,
    govs: [[TypeId; 2]; 3],
    barracks: TypeId,
    library: TypeId,
    town: TypeId,
    hoplites: TypeId,
    phalanx: TypeId,
    pikemen: TypeId,
    citizen: TypeId,
    catapult: TypeId,
    trebuchet: TypeId,
    machinegun: TypeId,
    rifleman: TypeId,
}

const H: UnitTraits = UnitTraits {
    free: true,
    jumpable: false,
    unique: false,
    hero: false,
    patriot: false,
    combat: true,
};
const J: UnitTraits = UnitTraits {
    free: false,
    jumpable: true,
    unique: false,
    hero: false,
    patriot: false,
    combat: true,
};
const PLAIN: UnitTraits = UnitTraits {
    free: false,
    jumpable: false,
    unique: false,
    hero: false,
    patriot: false,
    combat: true,
};

fn fixture() -> Fixture {
    let mut t = TechTree::new();
    let _food = t.add(TypeDef::good("Food"));
    let mut ages = [0; LEVELS];
    let names = [
        "Classical Age",
        "Medieval Age",
        "Gunpowder Age",
        "Enlightenment Age",
        "Industrial Age",
        "Modern Age",
        "Information Age",
    ];
    for (i, n) in names.iter().enumerate() {
        let mut d = TypeDef::age(n, i as u8);
        if i > 0 {
            d = d.needs(0, ages[i - 1]);
        }
        ages[i] = t.add(d);
    }
    let knowledge = t.add(TypeDef::good("Knowledge").needs(0, ages[0]));
    let mut epochs = [[0; LEVELS]; 4];
    for line in Line::ALL {
        for level in 0..LEVELS {
            let mut d = TypeDef::epoch(&format!("{line:?} {level}"), line, level as u8);
            if level > 0 {
                d = d.needs(0, epochs[line.index()][level - 1]);
            }
            epochs[line.index()][level] = t.add(d);
        }
    }
    let mil = epochs[Line::Military.index()];
    let sci = epochs[Line::Science.index()];
    let com = epochs[Line::Commerce.index()];
    let civ = epochs[Line::Civic.index()];
    let mut finals = [0; 4];
    for (i, n) in [
        "Missile Shield",
        "Global Government",
        "Virtual Reality",
        "Nanotechnology",
    ]
    .iter()
    .enumerate()
    {
        finals[i] = t.add(
            TypeDef::new(n, Kind::Final)
                .with_age(7)
                .needs(0, mil[6])
                .needs(1, ages[6]),
        );
    }
    t.roles.final_needs = vec![ages[6], sci[6], com[6], civ[6], mil[6]];
    let fortification = t.add(TypeDef::plain("Fortification", 1).needs(0, mil[1]));
    let herbal_lore = t.add(TypeDef::plain("Herbal Lore", 0).needs(0, sci[0]));
    let medicine = t.add(
        TypeDef::plain("Medicine", 2)
            .needs(0, herbal_lore)
            .needs(1, sci[2]),
    );
    let mut govs = [[0; 2]; 3];
    let gov_names = [
        ["Despotism", "Republic"],
        ["Monarchy", "Democracy"],
        ["Socialism", "Capitalism"],
    ];
    for tier in 0..3 {
        for col in 0..2 {
            let age = [0, 1, 3][tier];
            govs[tier][col] = t.add(
                TypeDef::gov(gov_names[tier][col], tier as u8, col as u8)
                    .with_age(age as i32)
                    .needs(1, ages[age]),
            );
        }
    }
    let town = t.add(TypeDef::building("Town"));
    let barracks = t.add(TypeDef::building("Barracks"));
    let library = t.add(TypeDef::building("Library"));
    let citizen = t.add(TypeDef::unit("Citizen", H).at(town));
    let hoplites = t.add(TypeDef::unit("Hoplites", H).at(barracks));
    let phalanx = t.add(
        TypeDef::unit("Phalanx", J)
            .at(barracks)
            .from(hoplites)
            .needs(0, ages[0]),
    );
    let pikemen = t.add(
        TypeDef::unit("Pikemen", J)
            .at(barracks)
            .from(phalanx)
            .needs(0, ages[1]),
    );
    t.types[hoplites].jump = Some(phalanx);
    t.types[phalanx].jump = Some(pikemen);
    let catapult = t.add(
        TypeDef::unit("Catapult", H)
            .at(barracks)
            .needs(0, ages[0])
            .needs(1, mil[0]),
    );
    let trebuchet = t.add(
        TypeDef::unit("Trebuchet", PLAIN)
            .at(barracks)
            .from(catapult)
            .needs(0, ages[1])
            .needs(1, mil[1]),
    );
    t.types[catapult].jump = Some(trebuchet);
    let rifleman = t.add(TypeDef::unit("Rifleman", J).at(barracks).needs(0, ages[4]));
    let machinegun = t.add(
        TypeDef::unit("Machine Gun", H)
            .at(barracks)
            .needs(0, ages[4]),
    );
    t.roles.machinegun = Some(machinegun);
    t.roles.rifleman = Some(rifleman);
    t.roles.town = Some(town);
    t.roles.knowledge = Some(knowledge);
    Fixture {
        tree: t,
        knowledge,
        ages,
        epochs,
        finals,
        fortification,
        herbal_lore,
        medicine,
        govs,
        barracks,
        library,
        town,
        hoplites,
        phalanx,
        pikemen,
        citizen,
        catapult,
        trebuchet,
        machinegun,
        rifleman,
    }
}

fn fresh(f: &Fixture) -> PlayerTech {
    let mut p = PlayerTech::new(&f.tree);
    f.tree.start(&Setup::STANDARD, &Tuning::RON, &mut p);
    p
}

/// Gains the epochs an age asks for, then the age, up to `to`.
fn age_up(f: &Fixture, p: &mut PlayerTech, to: usize) {
    let s = Setup::STANDARD;
    for a in 0..=to {
        let need = f.tree.techs_per_age(&s, p, f.ages[a]);
        let mut level = 0;
        while p.epochs < need {
            for line in Line::ALL {
                if p.epochs < need {
                    f.tree.gain_tech(&s, p, f.epochs[line.index()][level], 1);
                }
            }
            level += 1;
        }
        assert!(f.tree.has_preq(&s, p, f.ages[a]), "age {a} preqs");
        f.tree.gain_tech(&s, p, f.ages[a], 1);
    }
}

/// Step 11's city arm (`0x6dec2f`, `docs/AI.md` §98): the age that is a
/// prerequisite of the city line reports [`Gained::CityCheck`], so every
/// city of the leader runs `check_upgrade` on the frame it lands, and
/// an age that is not reports nothing.
#[test]
fn the_age_the_large_city_needs_asks_every_city_to_level_up() {
    let mut f = fixture();
    f.tree.types[f.town] = TypeDef::building("Town").needs(0, f.ages[1]);
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 0);
    let need = f.tree.techs_per_age(&s, &p, f.ages[1]);
    let mut level = 0;
    while p.epochs < need {
        for line in Line::ALL {
            if p.epochs < need {
                let e = f
                    .tree
                    .gain_tech(&s, &mut p, f.epochs[line.index()][level], 1);
                assert!(!e.contains(&Gained::CityCheck), "an epoch is no city's");
            }
        }
        level += 1;
    }
    let events = f.tree.gain_tech(&s, &mut p, f.ages[1], 1);
    assert_eq!(
        events.iter().filter(|e| **e == Gained::CityCheck).count(),
        1,
        "the Town's prerequisite arrived: {events:?}"
    );
    let mut q = fresh(&f);
    let first = f.tree.gain_tech(&s, &mut q, f.ages[0], 1);
    assert!(!first.contains(&Gained::CityCheck), "Classical is not it");
}

#[test]
fn the_full_game_asks_for_two_then_four_more_library_techs_per_age() {
    let f = fixture();
    let p = fresh(&f);
    let s = Setup::STANDARD;
    let quotas: Vec<i32> = f
        .ages
        .iter()
        .map(|&a| f.tree.techs_per_age(&s, &p, a))
        .collect();
    assert_eq!(quotas, [2, 6, 10, 14, 18, 22, 26]);
    // And the quota is what gates the age, not its data prerequisites:
    // Classical has none, and is still refused until two epochs are owned.
    assert!(!f.tree.has_preq(&s, &p, f.ages[0]));
    let mut p = p;
    f.tree
        .gain_tech(&s, &mut p, f.epochs[Line::Science.index()][0], 1);
    assert!(!f.tree.has_preq(&s, &p, f.ages[0]));
    f.tree
        .gain_tech(&s, &mut p, f.epochs[Line::Military.index()][0], 1);
    assert!(f.tree.has_preq(&s, &p, f.ages[0]));
    assert_eq!(p.epochs, 2);
    assert_eq!(p.epoch, [1, 0, 0, 1]);
}

#[test]
fn a_shortened_game_rescales_the_quota() {
    let f = fixture();
    let p = fresh(&f);
    // Start at Medieval, end at Industrial: span 4, 28/4 = 7 per age.
    let s = Setup {
        starting_age: 2,
        ending: 5,
        ..Setup::STANDARD
    };
    let quotas: Vec<i32> = f
        .ages
        .iter()
        .map(|&a| f.tree.techs_per_age(&s, &p, a))
        .collect();
    // (age - 2 + 1) * 7 - 2: the owned ages come out negative, which the
    // original never asks about.
    assert_eq!(quotas, [-9, -2, 5, 12, 19, 26, 33]);
}

#[test]
fn has_tech_reads_none_disabled_goods_units_and_buildings_differently() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    assert!(f.tree.has_tech_p(&s, &p, Preq::None));
    assert!(!f.tree.has_tech_p(&s, &p, Preq::Disabled));
    // A good is always had, even Knowledge before Classical.
    assert!(f.tree.has_tech(&s, &p, f.knowledge));
    // A building is had on its prerequisites alone; the bit is not read.
    p.tech[f.barracks] = false;
    assert!(f.tree.has_tech(&s, &p, f.barracks));
    // A unit needs its prerequisites and the bit. Phalanx: bit clear.
    assert!(!f.tree.has_tech(&s, &p, f.phalanx));
    p.tech[f.phalanx] = true;
    assert!(!f.tree.has_tech(&s, &p, f.phalanx), "Classical not owned");
    age_up(&f, &mut p, 0);
    assert!(f.tree.has_tech(&s, &p, f.phalanx));
}

#[test]
fn the_starting_position_owns_the_free_units_and_nothing_researched() {
    let f = fixture();
    let p = fresh(&f);
    // Hoplites, Citizen: no prerequisites, free. Phalanx needs Classical.
    assert!(p.tech[f.hoplites]);
    assert!(p.tech[f.citizen]);
    assert!(!p.tech[f.phalanx]);
    assert!(p.tech_at_start[f.hoplites]);
    // Buildings with their prerequisites met get the bit.
    assert!(p.tech[f.barracks]);
    assert!(p.tech[f.library]);
    assert!(p.tech[f.town]);
    assert_eq!((p.ages, p.epochs, p.discovered), (0, 0, 0));
    // Hoplites owned marks nothing obsolete yet: Phalanx jumps from it,
    // but Hoplites' own `from` is nothing.
    assert!(!p.obs[f.hoplites]);
}

/// **The opening unit set is a function of the nation.** `Leader::init`
/// gives a unit type its bit on `has_preq` *and* `tribe_can_type`, so
/// two leaders of the same tree and the same age start owning different
/// units — the generic line for a nation with no variant, the variant
/// for the one that has it, and never both. `rondata::diff::build_sim`
/// laid the position down before it had read the dump's `tribe` until
/// 2026-09-04 (`docs/TECH.md`, "The starting position is a function of
/// the nation"), which is what this pins.
/// **A built wonder is built for everyone**: `LeaderData::type_avail`
/// answers 0 for a type `BuildTypeData::already_built` reports, after
/// the prerequisites and eligibility and before the building arm's own
/// tests. French East Indies' AI kept the Pyramids on its make list
/// after finishing them, and its wonder arm drew three pairs too many on
/// 9781 (`docs/TECH.md`, "A built wonder is built for everyone").
#[test]
fn a_built_wonder_is_not_available() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = PlayerTech::new(&f.tree);
    f.tree.start(&s, &Tuning::RON, &mut p);
    let open = f.tree.type_avail(&s, &p, f.barracks, true);
    assert_ne!(open, NOT_AVAILABLE);
    assert_eq!(
        f.tree.type_avail_built(&s, &p, f.barracks, true, false),
        open
    );
    assert_eq!(
        f.tree.type_avail_built(&s, &p, f.barracks, true, true),
        NOT_AVAILABLE
    );
}

/// **A unique unit takes its nations out of its graft's mask**
/// (`Types::finalize_grafting@00669840`, item 3706, `docs/AI.md`
/// §358.2): the Arqimmortal's `GRAFT` is the Elite Pikemen, whose
/// shipped `TRIBE_MASK` still holds the Persians (23); run1383's packet
/// reads the Pikemen's `+0x10` as `0x7b6dbf` and `tribe_can_type(135)`
/// as 0 for them. Under "No Unique Units" the nations join it instead.
#[test]
fn a_unique_unit_s_nations_leave_its_graft_s_mask() {
    let f = fixture();
    let mut tree = f.tree.clone();
    while tree.types.len() < 0x32 {
        tree.add(TypeDef::new("pad", Kind::Plain));
    }
    let unique = UnitTraits { unique: true, ..H };
    let generic = tree.add(
        TypeDef::unit("Elite Pikemen", H)
            .at(f.barracks)
            .tribes(0xfb6dbf),
    );
    let mut arq = TypeDef::unit("Arqimmortal", unique)
        .at(f.barracks)
        .tribes(1 << 23);
    arq.graft = Some(generic);
    let arq = tree.add(arq);
    while tree.tribes.len() < 24 {
        tree.add_tribe(Tribe::default());
    }
    tree.finalize();
    assert_eq!(
        tree.types[generic].tribe_mask, 0xfb6dbf,
        "the file's mask stays"
    );
    let s = Setup::STANDARD;
    assert_eq!(tree.live_tribe_mask(&s, generic), 0x7b6dbf);
    assert_eq!(tree.live_tribe_mask(&s, arq), 1 << 23);
    let mut p = PlayerTech::new(&tree);
    p.tribe = 23;
    assert_eq!(tree.tribe_can_type(&s, &p, generic), 0);
    assert_eq!(tree.tribe_can_type(&s, &p, arq), 4);
    p.tribe = 11;
    assert_eq!(tree.tribe_can_type(&s, &p, generic), 4);
    let nouu = Setup {
        no_unique_units: true,
        ..Setup::STANDARD
    };
    assert_eq!(tree.live_tribe_mask(&nouu, generic), 0xfb6dbf);
}

#[test]
fn the_starting_units_follow_the_leader_s_nation() {
    let f = fixture();
    let mut tree = f.tree.clone();
    // A generic Ancient unit whose mask clears tribe 0, and tribe 0's
    // own variant of it — `Slingers` and `Atl-Atls`, in miniature.
    let generic = tree.add(TypeDef::unit("Slingers", H).at(f.barracks).tribes(!1u32));
    let variant = tree.add(TypeDef::unit("Atl-Atls", H).at(f.barracks).tribes(1));
    let s = Setup::STANDARD;

    let lay = |tribe: usize| {
        let mut p = PlayerTech::new(&tree);
        p.tribe = tribe;
        p.power = Some(tribe);
        tree.start(&s, &Tuning::RON, &mut p);
        p
    };
    // Tribe 11 owns the generic one and not the variant…
    let british = lay(11);
    assert!(british.tech[generic] && british.tech_at_start[generic]);
    assert!(!british.tech[variant]);
    // …and tribe 0 the other way round. Both own Hoplites, which no
    // nation substitutes — so the difference is the mask and not the
    // seeding.
    let aztec = lay(0);
    assert!(!aztec.tech[generic]);
    assert!(aztec.tech[variant] && aztec.tech_at_start[variant]);
    assert!(british.tech[f.hoplites] && aztec.tech[f.hoplites]);

    // And the consequence the AI reads: what a leader does not own is
    // `RESEARCHABLE` rather than `AVAILABLE`, which is what put two
    // Barracks types in front of `Leader::upgrade_units` on run53's
    // frame 6779.
    assert_eq!(tree.type_avail(&s, &british, generic, true), AVAILABLE);
    assert_eq!(tree.type_avail(&s, &aztec, generic, true), NOT_AVAILABLE);
    assert_eq!(tree.type_avail(&s, &british, variant, true), NOT_AVAILABLE);
}

#[test]
fn a_medieval_start_owns_two_ages_and_the_age_only_techs() {
    let f = fixture();
    let s = Setup {
        starting_age: 2,
        ..Setup::STANDARD
    };
    let mut p = PlayerTech::new(&f.tree);
    f.tree.start(&s, &Tuning::RON, &mut p);
    assert!(p.tech[f.ages[0]] && p.tech[f.ages[1]] && !p.tech[f.ages[2]]);
    assert_eq!(p.ages, 2);
    // No library epoch, and nothing that needs one.
    assert_eq!(p.epochs, 0);
    assert!(!p.tech[f.herbal_lore]);
    assert!(!p.tech[f.fortification]);
    // Governments are skipped at start even when their age is owned.
    assert!(!p.tech[f.govs[0][0]]);
    // Units whose prerequisites are the owned ages are owned outright.
    assert!(p.tech[f.phalanx]);
    assert!(p.tech[f.pikemen]);
    // And the predecessors are obsolete.
    assert!(p.obs[f.hoplites] && p.obs[f.phalanx]);
}

#[test]
fn classical_age_cascades_the_free_catapult_and_leaves_phalanx_to_research() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 0);
    // Catapult needs Classical and Military 1 (gained as part of the
    // quota, so already owned); it is free: the age's cascade took it.
    assert!(p.tech[f.catapult]);
    // Phalanx is not free: it became a research job.
    assert!(!p.tech[f.phalanx]);
    assert_eq!(f.tree.type_avail(&s, &p, f.phalanx, true), RESEARCHABLE);
    assert_eq!(f.tree.type_avail(&s, &p, f.hoplites, true), AVAILABLE);
    // Pikemen: Medieval not owned.
    assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), NOT_AVAILABLE);
}

#[test]
fn the_jump_rule_skips_the_intermediate_upgrade_and_obsoletes_it() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 1);
    // Medieval owned: Pikemen's prerequisites are met, so Phalanx can no
    // longer be researched — the player jumps straight to Pikemen.
    assert_eq!(f.tree.type_eligible(&s, &p, f.phalanx, true), 0);
    assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), RESEARCHABLE);
    // Gaining Pikemen owns and obsoletes everything below it.
    let events = f.tree.gain_tech(&s, &mut p, f.pikemen, 100);
    assert!(events.contains(&Gained::UnitUpgrade { to: f.pikemen }));
    assert!(p.tech[f.phalanx] && p.obs[f.phalanx]);
    assert!(p.tech[f.hoplites] && p.obs[f.hoplites]);
    assert_eq!(f.tree.type_avail(&s, &p, f.phalanx, true), NOT_AVAILABLE);
    assert_eq!(f.tree.current_upgrade(&s, &p, f.hoplites), f.pikemen);
    // Pikemen itself is owned and not obsolete.
    assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), AVAILABLE);
}

#[test]
fn a_non_jumpable_upgrade_needs_its_predecessor_available() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 1);
    // Trebuchet (no `j`) checks its predecessor through type_avail(…, 0):
    // the Catapult is owned (free with Classical), so it passes.
    assert!(p.tech[f.catapult]);
    assert_eq!(f.tree.type_avail(&s, &p, f.trebuchet, true), RESEARCHABLE);
    // Take the Catapult away: the predecessor is merely researchable
    // (2 < 3), and it is for this nation, so the Trebuchet is refused.
    p.tech[f.catapult] = false;
    assert_eq!(f.tree.type_avail(&s, &p, f.trebuchet, true), NOT_AVAILABLE);
}

#[test]
fn governments_pair_and_tier() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 1);
    let [despotism, republic] = f.govs[0];
    let [monarchy, democracy] = f.govs[1];
    assert_eq!(f.tree.type_avail(&s, &p, despotism, true), AVAILABLE);
    assert_eq!(f.tree.type_avail(&s, &p, republic, true), AVAILABLE);
    // Tier two needs a tier-one government.
    assert_eq!(f.tree.type_avail(&s, &p, monarchy, true), NOT_AVAILABLE);
    // Queueing Republic shuts Despotism.
    p.queued[republic] = 1;
    assert_eq!(f.tree.type_avail(&s, &p, despotism, true), NOT_AVAILABLE);
    p.queued[republic] = 0;
    f.tree.gain_tech(&s, &mut p, republic, 5);
    assert_eq!(f.tree.type_avail(&s, &p, despotism, true), NOT_AVAILABLE);
    // Either column of the tier below opens both of the tier above.
    assert_eq!(f.tree.type_avail(&s, &p, monarchy, true), AVAILABLE);
    assert_eq!(f.tree.type_avail(&s, &p, democracy, true), AVAILABLE);
    assert_eq!(f.tree.govs_taken(&s, &p), 1);
    // `discovered` counts every non-age, non-epoch gain: Republic, and
    // the free Catapult that Classical cascaded.
    assert_eq!(p.discovered, 2);
}

/// **A bonus tier above the first asks `has_preq`'s own gate** (twenty-
/// fourth pass, group 21; A1 rows 5 and 6): two governments taken, three
/// for the third tier, and Socialism or Capitalism when the tier below
/// is Monarchy's or Democracy's.
#[test]
fn a_bonus_tier_above_the_first_needs_governments_taken() {
    let mut f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 1);
    let [despotism, republic] = f.govs[0];
    let [monarchy, democracy] = f.govs[1];
    let (socialism, capitalism) = (f.herbal_lore, f.medicine);
    f.tree.roles.gov_bonuses = vec![
        ([Preq::None; 3], socialism),
        ([Preq::None; 3], capitalism),
        ([Preq::None; 3], monarchy),
        ([Preq::None; 3], democracy),
    ];
    let _ = despotism;
    // The first tier asks nothing.
    assert!(f.tree.bonus_tier_extra(&s, &p, 0, Some(monarchy)));
    // One government taken: tier two is shut.
    f.tree.gain_tech(&s, &mut p, republic, 5);
    assert_eq!(f.tree.govs_taken(&s, &p), 1);
    assert!(!f.tree.bonus_tier_extra(&s, &p, 1, Some(despotism)));
    // Two taken: a Despotism tier below needs nothing else, a
    // Monarchy's needs Socialism or Capitalism.
    f.tree.gain_tech(&s, &mut p, monarchy, 5);
    assert_eq!(f.tree.govs_taken(&s, &p), 2);
    assert!(f.tree.bonus_tier_extra(&s, &p, 1, Some(despotism)));
    assert!(!f.tree.bonus_tier_extra(&s, &p, 1, Some(monarchy)));
    p.tech[capitalism] = true;
    assert!(f.tree.bonus_tier_extra(&s, &p, 1, Some(monarchy)));
    // The third tier needs three.
    assert!(!f.tree.bonus_tier_extra(&s, &p, 2, Some(despotism)));
    f.tree.gain_tech(&s, &mut p, democracy, 5);
    assert!(f.tree.bonus_tier_extra(&s, &p, 2, Some(despotism)));
}

#[test]
fn the_finals_need_five_named_techs_and_respect_early_info_age() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 6);
    // 26 epochs are owned; the finals need the top of every line.
    assert!(!f.tree.has_preq(&s, &p, f.finals[0]));
    for line in Line::ALL {
        for level in 0..LEVELS {
            f.tree
                .gain_tech(&s, &mut p, f.epochs[line.index()][level], 1);
        }
    }
    assert_eq!(p.epochs, EPOCHS);
    assert!(f.tree.has_preq(&s, &p, f.finals[0]));
    assert_eq!(f.tree.type_avail(&s, &p, f.finals[0], true), AVAILABLE);
    let early = Setup {
        no_finals: true,
        ..Setup::STANDARD
    };
    assert_eq!(
        f.tree.type_avail(&early, &p, f.finals[0], true),
        NOT_AVAILABLE
    );
    let info_dm = Setup {
        rules: Rules::InfoDeathmatch,
        ..Setup::STANDARD
    };
    assert_eq!(
        f.tree.type_avail(&info_dm, &p, f.finals[0], true),
        NOT_AVAILABLE
    );
}

#[test]
fn the_tech_race_is_won_on_the_ending_age() {
    let f = fixture();
    let race = Setup {
        tech_race: true,
        ..Setup::STANDARD
    };
    let mut p = fresh(&f);
    age_up(&f, &mut p, 5);
    for line in Line::ALL {
        for level in 0..LEVELS {
            f.tree
                .gain_tech(&race, &mut p, f.epochs[line.index()][level], 1);
        }
    }
    // Every library tech is owned and nothing is won yet: the win is the
    // ending age, under that victory setting only.
    assert_eq!(p.epochs, EPOCHS);
    assert!(f.tree.has_preq(&race, &p, f.ages[6]));
    let plain = f
        .tree
        .gain_tech(&Setup::STANDARD, &mut p.clone(), f.ages[6], 9);
    assert!(!plain.contains(&Gained::TechRaceWon));
    let won = f.tree.gain_tech(&race, &mut p, f.ages[6], 9);
    assert!(won.contains(&Gained::TechRaceWon));
    assert_eq!(p.ages, 7);
    assert_eq!(p.epoch, [7, 7, 7, 7]);
}

#[test]
fn finalize_derives_what_the_loaders_derive() {
    let f = fixture();
    let mut t = f.tree.clone();
    let mil = f.epochs[Line::Military.index()];
    // A combat unit naming only Medieval gets Military 2 as well; a
    // civilian (Citizen: not combat) does not.
    let knight = t.add(
        TypeDef::unit("Knight", J)
            .at(f.barracks)
            .from(f.hoplites)
            .needs(0, f.ages[1]),
    );
    let stable = t.add(TypeDef::building("Stable"));
    let keep = t.add(TypeDef::building("Keep").from(stable));
    t.types[keep].kind = Kind::Building {
        auto: true,
        wonder: false,
    };
    t.finalize();
    assert_eq!(
        t.types[knight].preq,
        [Preq::Of(f.ages[1]), Preq::Of(mil[1]), Preq::None]
    );
    assert_eq!(t.types[f.citizen].preq[1], Preq::None);
    assert_eq!(t.types[f.phalanx].preq[1], Preq::Of(mil[0]));
    // The predecessor's `upgrade` is the successor (the last one written).
    assert_eq!(t.types[f.hoplites].upgrade, Some(knight));
    assert_eq!(t.types[f.phalanx].upgrade, Some(f.pikemen));
    // Buildings: `to` and, for an auto-upgrade, `upgrade`; `where` is the
    // predecessor unless the building is the base of its line.
    assert_eq!(t.types[stable].to, Some(keep));
    assert_eq!(t.types[stable].upgrade, Some(keep));
    assert_eq!(t.types[keep].where_, Some(stable));
    assert_eq!(t.types[stable].where_, None);
    // And so a Keep queues at a Stable.
    assert!(t.queue_here(stable, keep));
    // Which means the implicit requirement bites: with Medieval owned but
    // Military 2 not, the Knight is not researchable.
    let s = Setup::STANDARD;
    let mut p = PlayerTech::new(&t);
    t.start(&s, &Tuning::RON, &mut p);
    let fx = Fixture {
        tree: t.clone(),
        ..f
    };
    age_up(&fx, &mut p, 1);
    assert!(p.tech[mil[0]] && p.tech[mil[1]], "the quota owns both");
    t.lose_tech(&s, &mut p, mil[1]);
    assert!(!t.has_preq(&s, &p, knight));
    t.gain_tech(&s, &mut p, mil[1], 3);
    assert!(t.has_preq(&s, &p, knight));
}

#[test]
fn losing_touches_only_unit_and_tech_bits_and_gaining_a_government_sets_it() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 0);
    let [despotism, _] = f.govs[0];
    assert_eq!(p.gov, None);
    f.tree.gain_tech(&s, &mut p, despotism, 2);
    assert_eq!(p.gov, Some(despotism));
    // A building's bit survives lose_tech; a unit's does not.
    assert!(p.tech[f.barracks]);
    f.tree.lose_tech(&s, &mut p, f.barracks);
    assert!(p.tech[f.barracks]);
    assert!(p.tech[f.catapult]);
    f.tree.lose_tech(&s, &mut p, f.catapult);
    assert!(!p.tech[f.catapult]);
}

#[test]
fn the_ending_age_caps_ages_units_and_techs() {
    let f = fixture();
    let mut p = fresh(&f);
    age_up(&f, &mut p, 2);
    // Ending at Gunpowder (3): Enlightenment is refused, and so is a unit
    // whose prerequisite is it; Gunpowder itself was allowed.
    let s = Setup {
        ending: 3,
        ..Setup::STANDARD
    };
    assert_eq!(f.tree.type_eligible(&s, &p, f.ages[3], true), 0);
    assert_eq!(f.tree.type_eligible(&s, &p, f.ages[2], true), 4);
    assert_eq!(f.tree.type_eligible(&s, &p, f.rifleman, true), 0);
    // A building tech within the end is fine.
    assert_eq!(f.tree.type_eligible(&s, &p, f.medicine, true), 4);
}

#[test]
fn machine_guns_need_a_rifle_line_unit() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 4);
    // The Machine Gun is free with Industrial, so the cascade owned it —
    // but type_avail still asks for a Rifleman.
    assert!(p.tech[f.machinegun]);
    assert_eq!(f.tree.type_avail(&s, &p, f.machinegun, true), NOT_AVAILABLE);
    f.tree.gain_tech(&s, &mut p, f.rifleman, 9);
    assert_eq!(f.tree.type_avail(&s, &p, f.machinegun, true), AVAILABLE);
}

#[test]
fn queue_here_reads_lineage_and_the_where_column() {
    let f = fixture();
    assert!(f.tree.queue_here(f.barracks, f.phalanx));
    assert!(!f.tree.queue_here(f.library, f.phalanx));
    assert!(f.tree.queue_here(f.town, f.citizen));
    // A unit with a `disable` first prerequisite can be queued nowhere.
    let mut t = f.tree.clone();
    t.types[f.phalanx].preq[0] = Preq::Disabled;
    assert!(!t.queue_here(f.barracks, f.phalanx));
}

#[test]
fn a_disabled_prerequisite_is_refused_and_a_missing_one_is_not() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 0);
    let mut t = f.tree.clone();
    t.types[f.phalanx].preq[1] = Preq::Disabled;
    assert_eq!(t.type_eligible(&s, &p, f.phalanx, true), 0);
    assert!(!t.has_preq(&s, &p, f.phalanx));
    t.types[f.phalanx].preq[1] = Preq::None;
    assert!(t.has_preq(&s, &p, f.phalanx));
}

#[test]
fn losing_an_age_takes_its_dependents_and_recounts() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    age_up(&f, &mut p, 2);
    f.tree.gain_tech(&s, &mut p, f.fortification, 3);
    f.tree.gain_tech(&s, &mut p, f.pikemen, 3);
    assert_eq!(p.ages, 3);
    // Losing Medieval loses Gunpowder (an age at or above it) and
    // everything whose closure names it; the epochs do not name ages.
    f.tree.lose_tech(&s, &mut p, f.ages[1]);
    assert!(p.tech[f.ages[0]] && !p.tech[f.ages[1]] && !p.tech[f.ages[2]]);
    assert_eq!(p.ages, 1);
    assert!(p.tech[f.fortification], "needs Military 2 only");
    // set_age drops what no longer follows: Pikemen needs Medieval.
    f.tree.set_age(&s, &mut p, 1, 4);
    assert!(!p.tech[f.pikemen]);
    assert!(p.tech[f.phalanx], "Classical still owned");
    assert_eq!(p.ages, 1);
}

#[test]
fn set_epoch_walks_a_line_and_compute_epoch_agrees() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = fresh(&f);
    f.tree.set_epoch(&s, &mut p, Line::Commerce, 4, 2);
    assert_eq!(p.epoch[Line::Commerce.index()], 4);
    assert_eq!(f.tree.compute_epoch(&s, &p, Line::Commerce), 4);
    assert_eq!(p.epochs, 4);
    f.tree.set_epoch(&s, &mut p, Line::Commerce, 2, 3);
    assert_eq!(p.epoch[Line::Commerce.index()], 2);
    assert_eq!(p.epochs, 2);
}

#[test]
fn nation_free_rules_cascade_when_gated_on() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut t = f.tree.clone();
    t.free_rules.push(FreeRule {
        gate: Gate::Power(14),
        enabled: 1,
        candidates: vec![f.herbal_lore, f.medicine],
        shape: Shape::PreqMatch,
    });
    let mut p = PlayerTech::new(&t);
    p.power = Some(14);
    t.start(&s, &Tuning::RON, &mut p);
    // Science 1 is Herbal Lore's prerequisite: gaining it hands Herbal
    // Lore over; Medicine waits for Science 3.
    t.gain_tech(&s, &mut p, f.epochs[Line::Science.index()][0], 1);
    assert!(p.tech[f.herbal_lore]);
    assert!(!p.tech[f.medicine]);
    // Without the power, nothing.
    let mut q = PlayerTech::new(&t);
    t.start(&s, &Tuning::RON, &mut q);
    t.gain_tech(&s, &mut q, f.epochs[Line::Science.index()][0], 1);
    assert!(!q.tech[f.herbal_lore]);
    // "No Nation Powers" shuts it too.
    let off = Setup {
        no_nation_powers: true,
        ..Setup::STANDARD
    };
    let mut r = PlayerTech::new(&t);
    r.power = Some(14);
    t.start(&off, &Tuning::RON, &mut r);
    t.gain_tech(&off, &mut r, f.epochs[Line::Science.index()][0], 1);
    assert!(!r.tech[f.herbal_lore]);
}

#[test]
fn a_wonder_gated_free_rule_waits_for_the_wonder() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut t = f.tree.clone();
    let wonder = f.catapult; // any type id stands for the wonder here
    t.free_rules.push(FreeRule {
        gate: Gate::Wonder(wonder),
        enabled: 1,
        candidates: vec![f.herbal_lore],
        shape: Shape::PreqMatch,
    });
    let sci = f.epochs[Line::Science.index()][0];
    let mut p = PlayerTech::new(&t);
    t.start(&s, &Tuning::RON, &mut p);
    t.gain_tech(&s, &mut p, sci, 1);
    assert!(!p.tech[f.herbal_lore], "no wonder, no block");
    let mut q = PlayerTech::new(&t);
    q.wonders.push(wonder);
    t.start(&s, &Tuning::RON, &mut q);
    t.gain_tech(&s, &mut q, sci, 1);
    assert!(q.tech[f.herbal_lore], "the held wonder opens it");
}

#[test]
fn a_free_starting_epoch_is_owned_before_frame_zero() {
    let f = fixture();
    let s = Setup::STANDARD;
    let mut p = PlayerTech::new(&f.tree);
    p.power = Some(6); // Romans: ROMAN_FREE_MILITARY
    f.tree.start(&s, &Tuning::RON, &mut p);
    assert_eq!(p.epoch[Line::Military.index()], 1);
    assert_eq!(p.epochs, 1);
    // The Catapult needs Classical too, so it is not free yet.
    assert!(!p.tech[f.catapult]);
}

#[test]
fn the_lobby_remap_rescales_a_units_military_requirement() {
    let f = fixture();
    let mut t = f.tree.clone();
    // A unit asking for Military 4 in slot 1, in a game that starts at
    // Gunpowder and ends at Industrial: span 3 ages, 28/3 = 9 per age.
    let mil = f.epochs[Line::Military.index()];
    let u = t.add(
        TypeDef::unit("Dragoons", J)
            .needs(0, f.ages[2])
            .needs(1, mil[3]),
    );
    let s = Setup {
        starting_age: 3,
        ending: 5,
        ..Setup::STANDARD
    };
    let p = PlayerTech::new(&t);
    // age 3: n = 9 * (3 - 3 + 1) + 3 = 12, 12/4 = 3, level 2.
    assert_eq!(t.get_preq(&s, Some(&p), u, 1), Preq::Of(mil[2]));
    // A requirement from before the start vanishes.
    let v = t.add(TypeDef::unit("Old", J).needs(1, mil[1]));
    assert_eq!(t.get_preq(&s, Some(&p), v, 1), Preq::None);
    // The full game leaves the column alone.
    assert_eq!(
        t.get_preq(&Setup::STANDARD, Some(&p), u, 1),
        Preq::Of(mil[3])
    );
}

#[test]
fn a_unit_s_age_is_read_through_the_lobby_remap() {
    // `get_age_slow` asks `get_preq(i, −1)`, so the age
    // `finalize_grafting` caches is the lobby's: the Trireme's slot-1
    // The Art of War (Military 0, `AGE` 0) is Conscription (Military 3,
    // `AGE` 3) in the Bantu Ancient pair's lobby, start 0 and ending 1
    // — `n = 28 / 2 * (0 - 0 + 1) + 3 = 17`, `17 / 4 = 4`, level 3 —
    // and the Galley it upgrades to, which asks for the Classical Age
    // in slot 0, stays at 1 (item 3589, `docs/AI.md` §315).
    let f = fixture();
    let mut t = f.tree.clone();
    let mil = f.epochs[Line::Military.index()];
    let trireme = t.add(TypeDef::unit("Trireme", J).needs(1, mil[0]));
    let galley = t.add(TypeDef::unit("Galley", J).needs(0, f.ages[0]));
    let s = Setup {
        starting_age: 0,
        ending: 1,
        ..Setup::STANDARD
    };
    assert_eq!(t.age_in(&s, trireme), 3);
    assert_eq!(t.age_in(&Setup::STANDARD, trireme), 0);
    assert_eq!(t.age_in(&s, galley), 1);
    assert_eq!(t.age_in(&Setup::STANDARD, galley), 1);
}

#[test]
fn a_unit_s_military_level_is_read_through_the_lobby_remap() {
    // `military_level_in` asks `get_preq(1, −1)`, which the lobby
    // rescales: the Dragoons that ask for Military 4 stand at level 3 in
    // the lobby of `the_lobby_remap_rescales_a_units_military_requirement`
    // and at 4 in the full game; a requirement from before the start
    // vanishes, so the unit stands at level 0 (floored to 1 by
    // `get_cost`) where the table's column says 2 (`docs/AI.md` §158).
    let f = fixture();
    let mut t = f.tree.clone();
    let mil = f.epochs[Line::Military.index()];
    let u = t.add(
        TypeDef::unit("Dragoons", J)
            .needs(0, f.ages[2])
            .needs(1, mil[3]),
    );
    let v = t.add(TypeDef::unit("Old", J).needs(1, mil[1]));
    let s = Setup {
        starting_age: 3,
        ending: 5,
        ..Setup::STANDARD
    };
    assert_eq!(t.military_level_in(&s, u), 3);
    assert_eq!(t.military_level_in(&s, v), 0);
    assert_eq!(t.military_level_in(&Setup::STANDARD, u), 4);
    assert_eq!(t.military_level_in(&Setup::STANDARD, v), 2);
    assert_eq!(t.military_level_of(u), 4);
}

#[test]
fn barbarians_defenders_start_with_both_settings() {
    let f = fixture();
    let s = Setup {
        starting_age: 2,
        starting_age2: 2,
        rules: Rules::BarbariansAtTheGates,
        ..Setup::STANDARD
    };
    let mut d = PlayerTech::new(&f.tree);
    d.team = 0;
    assert_eq!(f.tree.starting_age(&s, &d), 4);
    let mut a = PlayerTech::new(&f.tree);
    a.team = 1;
    assert_eq!(f.tree.starting_age(&s, &a), 2);
    // The defenders' epochs are pre-owned in this mode alone.
    f.tree.start(&s, &Tuning::RON, &mut d);
    assert!(d.epochs > 0);
    f.tree.start(&s, &Tuning::RON, &mut a);
    assert_eq!(a.epochs, 0);
}
