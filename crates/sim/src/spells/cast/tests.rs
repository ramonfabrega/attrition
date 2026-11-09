use crate::tech::{TypeDef, UnitTraits};

/// **Parmenio's Ambush** (item 3320; `SpellType::cast_ambush@00671610`):
/// his `ActiveSpell` runs `PARMENIO_AMBUSH_TIME` (the file's `3/1`, 768
/// at 8.8) times the usual extent, shifted right by eight and rounded
/// toward zero. Made to fail with the tuning at 256.
#[test]
fn parmenio_s_ambush_is_three_times_as_long() {
    struct V(i32, bool, Option<(i32, i32)>);
    impl super::AmbushView for V {
        fn frame(&self) -> i32 {
            1000
        }
        fn spell_extent(&self) -> (i32, i32) {
            (300, 100)
        }
        fn general_upgrade(&mut self) -> i32 {
            2
        }
        fn is_parmenio(&mut self) -> bool {
            self.1
        }
        fn parmenio_ambush_time(&self) -> i32 {
            self.0
        }
        fn push_active_spell(&mut self, start: i32, end: i32) {
            self.2 = Some((start, end));
        }
        fn set_flags(&mut self) {}
        fn console_sound(&mut self) {}
    }
    assert_eq!(Tuning::RON.parmenio_ambush_time, 768);
    let run = |t: i32, parmenio: bool| {
        let mut v = V(t, parmenio, None);
        super::cast_ambush_steps(&mut v);
        v.2.expect("pushed")
    };
    assert_eq!(run(Tuning::RON.parmenio_ambush_time, false), (1000, 1500));
    assert_eq!(run(Tuning::RON.parmenio_ambush_time, true), (1000, 2500));
    // Toward zero: 500 × -255 / 256 is -498.05.
    assert_eq!(run(-255, true), (1000, 1000 - 498));
}
use crate::world::{Cell, Terrain, World};
use crate::{Pos, Sim, Tuning, Unit, UnitType};

/// **The Iroquois spear** (item 3211; `UnitData::speed@0060aae0`): a
/// unit of the line `is(0xa6, 0)` on a cell its leader is allied to
/// steps `IROQUOIS_SPEAR_SPEED[k]` more, `k` its strict `is(0xa6 +
/// k, 1)`; off allied ground, or off the line, nothing. run932's
/// `1/28`..`1/30` (TI 166). Made to fail with the term dropped and
/// with the ally test dropped.
#[test]
fn an_iroquois_spearman_steps_faster_on_allied_ground() {
    use crate::world::Owner;
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut tuning = Tuning::RON;
    tuning.iroquois_spear_speed = [1, 2, 3, 4];
    let mut s = Sim::new(tuning, world, 2);
    let mut t = crate::tech::TechTree::new();
    while t.types.len() <= 0xaa {
        t.add(TypeDef::unit("filler", UnitTraits::default()));
    }
    t.types[0xa7].from = Some(0xa6);
    s.set_tech_tree(t);
    let unit = |s: &mut Sim, row: usize| {
        let ty = s.add_unit_type(UnitType {
            tree: Some(row),
            type_index: i32::try_from(row).unwrap(),
            ..UnitType::default()
        });
        let mut u = Unit::new(1, 0, Pos::new(3 * 768 + 10, 3 * 768 + 10), 20);
        u.ty = Some(ty);
        u.movement.speed = 25;
        s.add_unit(u)
    };
    let spear = unit(&mut s, 0xa6);
    let pike = unit(&mut s, 0xa7);
    let other = unit(&mut s, 0xaa);
    let cell = Cell::new(3, 3);
    assert_eq!(s.unit_speed(spear), 25, "unowned ground");
    s.world.set_owner(cell, Owner::Player(1), Owner::None);
    assert_eq!(s.unit_speed(spear), 26, "its own ground, entry0");
    assert_eq!(s.unit_speed(pike), 27, "of the line from 0xa6, entry1");
    assert_eq!(s.unit_speed(other), 25, "off the line");
    s.world.set_owner(cell, Owner::Player(0), Owner::None);
    assert_eq!(s.unit_speed(spear), 25, "another leader's, not allied");
}

/// A tree whose rows are `TypeIndex`'s, up to Memnon, and one unit of
/// player 1 for each of the General, the Spy, Memnon and a filler row,
/// each holding a full pool (`MANA` 1000).
fn casters(tuning: Tuning) -> (Sim, [usize; 4]) {
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut s = Sim::new(tuning, world, 2);
    let mut t = crate::tech::TechTree::new();
    while t.types.len() <= super::MEMNON {
        let name = match t.types.len() {
            super::GENERAL => "General",
            super::SPY => "Spy",
            super::MEMNON => "Memnon",
            _ => "filler",
        };
        t.add(TypeDef::unit(name, UnitTraits::default()));
    }
    s.set_tech_tree(t);
    let units = [super::GENERAL, super::SPY, super::MEMNON, 0x40].map(|row| {
        let ty = s.add_unit_type(UnitType {
            mana: 1000,
            tree: Some(row),
            type_index: i32::try_from(row).unwrap(),
            ..UnitType::default()
        });
        let mut u = Unit::new(1, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = Some(ty);
        s.add_unit(u)
    });
    (s, units)
}

/// **`get_range` whole** (`SpellTypeData::get_range@00676a80`, item
/// 3985, `docs/GOLDEN.md` §178). With every spy upgrade held, a Bribe
/// and Counterintelligence reach `3 × 2 × 192` further and an Informer
/// `3 × 3 × 192`; the Informer's halving on a building is of that sum;
/// a siege unit, a supply unit or a building takes one tile for Bribe
/// whatever the upgrades; the Sniper takes one tile on a vehicle; and
/// another leader's upgrades count nothing. Made to fail with the
/// upgrade terms dropped, with the halving taken before them, and with
/// the one-tile collapse dropped.
#[test]
fn get_range_adds_the_spy_upgrades_and_collapses_on_siege_and_buildings() {
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    let at = |c: i32| usize::try_from(c - spell::FIRST).unwrap();
    s.orders.spells[at(spell::BRIBE)].range = 6 * 192;
    s.orders.spells[at(spell::COUNTERINTEL)].range = 10 * 192;
    s.orders.spells[at(spell::INFORMER)].range = 10 * 192;
    s.orders.spells[at(0x281)].range = 12 * 192;
    let mut tree = s.tables.tech_tree.clone();
    let steps = ["one", "two", "three"].map(|n| tree.add(crate::tech::TypeDef::plain(n, 0)));
    tree.roles.spy_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let who = s.units[spy].owner;
    let b = s.add_building(0, Pos::new(768, 768), 0);
    let (u, bld) = (
        crate::combat::Obj::Unit(filler),
        crate::combat::Obj::Building(b),
    );
    let row = |s: &Sim| {
        [
            s.spell_range(spell::BRIBE, who, u),
            s.spell_range(spell::COUNTERINTEL, who, u),
            s.spell_range(spell::INFORMER, who, u),
            s.spell_range(spell::INFORMER, who, bld),
            s.spell_range(spell::BRIBE, who, bld),
        ]
    };
    assert_eq!(row(&s), [1152, 1920, 1920, 960, 192], "no upgrade");
    for t in steps {
        s.gain_tech(who, t);
    }
    assert_eq!(
        row(&s),
        [2304, 3072, 3648, 1824, 192],
        "three upgrades: the Informer halved after its term"
    );
    assert_eq!(
        s.spell_range(spell::BRIBE, 1 - who, u),
        1152,
        "another leader"
    );
    let t = s.units[filler].ty.unwrap();
    s.tables.unit_types[t].combat.siege = true;
    assert_eq!(s.spell_range(spell::BRIBE, who, u), 192, "a siege unit");
    assert_eq!(
        s.spell_range(spell::INFORMER, who, u),
        3648,
        "not the Informer's"
    );
    s.tables.unit_types[t].combat.siege = false;
    s.tables.unit_types[t].cols.unit_flags2 |= crate::ai::ai_load::uflags2::SUPPLY_OR_HERO;
    assert_eq!(
        s.spell_range(spell::COUNTERINTEL, who, u),
        192,
        "a supply unit"
    );
    assert_eq!(
        s.spell_range(0x281, who, u),
        12 * 192,
        "the Sniper, no vehicle"
    );
    s.tables.unit_types[t].combat.obj_masks |= crate::combat::mask::VEHICLE;
    assert_eq!(s.spell_range(0x281, who, u), 192, "the Sniper on a vehicle");
}

/// **`get_job_time`'s spy arm** (`00675800`, item 3985, `docs/GOLDEN.md`
/// §178): with `SPIES_CRAFT_FASTER`'s prerequisite held, Bribe's and
/// the Informer's hold is halved for the caster's leader and no other,
/// and Counterintelligence's is not. Made to fail with the arm dropped
/// and with Counterintelligence let in.
#[test]
fn strategy_halves_bribe_and_the_informer_s_hold() {
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, _]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    let at = |c: i32| usize::try_from(c - spell::FIRST).unwrap();
    s.orders.spells[at(spell::BRIBE)].job_time = 100;
    s.orders.spells[at(spell::INFORMER)].job_time = 40;
    s.orders.spells[at(spell::COUNTERINTEL)].job_time = 40;
    let mut tree = s.tables.tech_tree.clone();
    let strategy = tree.add(crate::tech::TypeDef::plain("strategy", 0));
    tree.roles.spies_craft_faster_preq = Some(strategy);
    s.set_tech_tree(tree);
    let row = |s: &Sim| {
        [spell::BRIBE, spell::INFORMER, spell::COUNTERINTEL].map(|c| s.cast_job_time(spy, c))
    };
    assert_eq!(row(&s), [100, 40, 40]);
    let who = s.units[spy].owner;
    s.gain_tech(1 - who, strategy);
    assert_eq!(row(&s), [100, 40, 40], "another leader's");
    s.gain_tech(who, strategy);
    assert_eq!(row(&s), [50, 20, 40]);
}

/// **`cast_bribe`'s refusal refunds** (`00672d00`, item 3985,
/// `docs/GOLDEN.md` §178): a Bribe landing on a target of the caster's
/// own, or on one inside its owner's CEO's reach (`has_general(0,
/// 0x165)`), hands the craft's `MANA` back through `unpay_cast_costs`'
/// mana half, floored at 0, and copies no unit. Dead from a unit's
/// cast — `do_cast` asks `is_valid_target` on the land frame first —
/// and held here so the arm stays faithful. Made to fail with the
/// refund dropped and with the CEO test dropped.
#[test]
fn a_refused_bribe_hands_its_mana_back() {
    use crate::ai::ai_load::{UnitCols, uflags2};
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    let at = |c: i32| usize::try_from(c - spell::FIRST).unwrap();
    s.orders.spells[at(spell::BRIBE)].mana = 200;
    s.orders.spells[at(spell::BRIBE)].from = [Some(super::SPY), None];
    let units = s.units.len();
    let t = crate::combat::Obj::Unit(filler);
    s.units[spy].mana_burn = 500;
    s.cast_on(spy, spell::BRIBE, t);
    assert_eq!(s.units[spy].mana_burn, 300, "its own target");
    s.units[spy].mana_burn = 150;
    s.cast_on(spy, spell::BRIBE, t);
    assert_eq!(s.units[spy].mana_burn, 0, "floored at 0");
    s.units[filler].owner = 0;
    s.units[filler].type_index = 0x165;
    let ty = s.units[filler].ty.unwrap();
    s.tables.unit_types[ty].cols = UnitCols {
        unit_flags2: uflags2::FLAG2_GENERAL,
        ..UnitCols::default()
    };
    s.units[spy].mana_burn = 500;
    s.cast_on(spy, spell::BRIBE, t);
    assert_eq!(s.units[spy].mana_burn, 300, "the CEO's reach");
    assert_eq!(s.units.len(), units, "no copy");
    assert_eq!(s.units[filler].owner, 0, "not taken");
}

/// **`do_cast`'s two `0x11000` writes** (`005ec932`, `005ec9a7`; item
/// 2804, `docs/GOLDEN.md` §105): a cloaked caster of a craft without
/// `l` is blown and fought — on the first, every-frame site not a Spy
/// and not the Sniper, on the cast frame's a Spy too. A craft with `l`
/// keeps the cloak, and a stealth-only unit holds its cast order, so
/// it is not cloaked at all. Made to fail with the cloak test dropped,
/// with the `l` test dropped, and with the Spy exemption dropped.
#[test]
fn a_cloaked_caster_of_a_craft_without_l_is_blown() {
    use crate::ai::ai_load::{UnitCols, uflags};
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    let at = |c: i32| usize::try_from(c - spell::FIRST).unwrap();
    s.orders.spells[at(spell::BRIBE)].flags = 0;
    s.orders.spells[at(spell::INFORMER)].flags = super::craft::KEEP_CLOAK;
    s.orders.spells[at(0x281)].flags = 0;
    let cloak = |s: &mut Sim, u: usize, flags: u32| {
        let t = s.units[u].ty.unwrap();
        s.tables.unit_types[t].cols = UnitCols {
            unit_flags: flags,
            ..UnitCols::default()
        };
    };
    cloak(&mut s, spy, uflags::CLOAKED);
    cloak(&mut s, filler, uflags::CLOAKED);
    let reset = |s: &mut Sim, u: usize| {
        s.units[u].blown = false;
        s.units[u].fought = false;
    };
    let bits = |s: &Sim, u: usize| (s.units[u].blown, s.units[u].fought);
    // The first site: a cloaked non-Spy, the craft without `l`.
    s.cast_blow(filler, spell::BRIBE, true);
    assert_eq!(bits(&s, filler), (true, true));
    reset(&mut s, filler);
    s.cast_blow(filler, spell::INFORMER, true);
    s.cast_blow(filler, 0x281, true);
    assert_eq!(bits(&s, filler), (false, false), "`l`, and the Sniper");
    // A Spy is exempt at the first site and not at the second.
    s.cast_blow(spy, spell::BRIBE, true);
    assert_eq!(bits(&s, spy), (false, false));
    s.cast_blow(spy, spell::INFORMER, false);
    assert_eq!(bits(&s, spy), (false, false), "the Informer keeps it");
    s.cast_blow(spy, spell::BRIBE, false);
    assert_eq!(bits(&s, spy), (true, true));
    // Not cloaked: a stealth-only unit with its cast order held.
    cloak(&mut s, filler, uflags::STEALTH);
    reset(&mut s, filler);
    s.add_cast_order(filler, spell::BRIBE);
    s.cast_blow(filler, spell::BRIBE, false);
    assert_eq!(bits(&s, filler), (false, false));
    s.units[filler].orders.clear();
    assert!(s.unit_is_cloaked(filler), "idle, it is");
}

/// **A stop takes a held cast and nothing else** (item 3913,
/// `Group::action_stop_spell@006fd7a0`, `docs/GOLDEN.md` §160): a
/// member whose action is a `CASTORDER` loses its orders and its
/// `spell_time`; one whose action is a move keeps both, as run1507's
/// Spy would during its walk to the ring. Made to fail with the cast
/// test dropped, and with the `spell_time` write dropped.
#[test]
fn a_stop_takes_a_held_cast_and_nothing_else() {
    use crate::orders::{MoveKind, QueuePos, spell};
    let (mut s, [general, spy, _, _]) = casters(Tuning::RON);
    s.add_cast_order(spy, spell::INFORMER);
    s.units[spy].spell_time = 15;
    s.add_move_order(
        general,
        Pos::new(768, 768),
        MoveKind::MoveTo,
        QueuePos::New,
        true,
    );
    s.units[general].spell_time = 7;
    let mut g = crate::groups::group::Group::stack(1);
    s.group_add(&mut g, spy);
    s.group_add(&mut g, general);
    assert_eq!(s.group_action_stop_spell(&g), 1);
    assert!(s.units[spy].orders.is_empty(), "the cast is closed");
    assert_eq!(s.units[spy].spell_time, 0);
    assert_eq!(s.units[general].orders.len(), 1, "a move is not a cast");
    assert_eq!(s.units[general].spell_time, 7);
}

/// **Bribe's target arm** (`SpellTypeData::is_valid_target@006763c0`,
/// item 2812): an enemy's armed, land, non-spy, non-worker unit; a
/// building, an unarmed unit, a Spy, a worker, an ally and a peace
/// partner are refused. Made to fail with the war test and with the
/// armed test dropped.
#[test]
fn bribe_takes_only_an_enemy_s_armed_land_unit() {
    use crate::combat::Obj;
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    s.orders.spells[usize::try_from(spell::BRIBE - spell::FIRST).unwrap()].flags =
        0x2 | 0x20 | 0x80;
    let target = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[filler].ty;
        u
    });
    s.units[target].owner = 0;
    let t = Obj::Unit(target);
    s.units[spy].owner = 1;
    assert!(!s.spell_valid_target(spell::BRIBE, 1, t), "at peace");
    s.declare_war(0, 1);
    assert!(!s.spell_valid_target(spell::BRIBE, 1, t), "unarmed");
    let ty = s.units[target].ty.unwrap();
    s.tables.unit_types[ty].combat.attack = 5;
    assert!(
        s.spell_valid_target(spell::BRIBE, 1, t),
        "an enemy's soldier"
    );
    assert!(
        !s.spell_valid_target(spell::BRIBE, 1, Obj::Unit(spy)),
        "its own"
    );
}

/// **The Sniper's target arm** (`SpellTypeData::is_valid_target@006763c0`'s
/// `0x281`, item 2971): an enemy's armed land unit; refused unarmed, a
/// worker's line aside, and in the air. Made to fail with the armed
/// test dropped.
#[test]
fn sniper_takes_only_an_enemy_s_armed_land_unit() {
    use crate::combat::Obj;
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    s.orders.spells[usize::try_from(0x281 - spell::FIRST).unwrap()].flags = 0x2 | 0x20;
    let target = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[filler].ty;
        u
    });
    let t = Obj::Unit(target);
    s.declare_war(0, 1);
    assert!(!s.spell_valid_target(0x281, 1, t), "unarmed");
    let ty = s.units[target].ty.unwrap();
    s.tables.unit_types[ty].combat.attack = 5;
    assert!(s.spell_valid_target(0x281, 1, t), "an enemy's soldier");
    assert!(
        !s.spell_valid_target(0x281, 0, Obj::Unit(spy)),
        "no war arm for its own side"
    );
}

/// **Counterintelligence's target arm** (`0x277`, item 2971): one's own
/// is a target only once infiltrated; an enemy's only if it is a Spy.
/// Made to fail with the infiltrated test dropped and with the Spy
/// test dropped.
#[test]
fn counterintelligence_takes_an_infiltrated_friend_or_an_enemy_spy() {
    use crate::combat::Obj;
    use crate::orders::{SpellType, spell};
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    s.orders.spells =
        vec![SpellType::default(); usize::try_from(spell::LAST - spell::FIRST + 1).unwrap()];
    s.orders.spells[usize::try_from(spell::COUNTERINTEL - spell::FIRST).unwrap()].flags =
        0x2 | 0x10 | 0x20;
    let friend = Obj::Unit(filler);
    assert!(
        !s.spell_valid_target(spell::COUNTERINTEL, 1, friend),
        "not infiltrated"
    );
    s.units[filler].infiltrated = 1;
    assert!(
        s.spell_valid_target(spell::COUNTERINTEL, 1, friend),
        "infiltrated"
    );
    let foe_filler = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[filler].ty;
        u
    });
    let foe_spy = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[spy].ty;
        u
    });
    s.declare_war(0, 1);
    assert!(
        !s.spell_valid_target(spell::COUNTERINTEL, 1, Obj::Unit(foe_filler)),
        "an enemy soldier"
    );
    assert!(
        s.spell_valid_target(spell::COUNTERINTEL, 1, Obj::Unit(foe_spy)),
        "an enemy Spy"
    );
}

/// **Counterintelligence's body** (`SpellType::cast_counterintel@00673740`,
/// item 3920, `docs/GOLDEN.md` §167): an enemy's Spy dies with `dtype`
/// 2 and leaves a death object; one's own infiltrated unit only loses
/// its `infiltrated`. Made to fail with the kill dropped, with `dtype`
/// 0, and with the alliance test inverted.
#[test]
fn counterintelligence_kills_an_enemy_spy_and_clears_a_friend() {
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    let foe_spy = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[spy].ty;
        u
    });
    s.units[foe_spy].on_map = true;
    s.declare_war(0, 1);
    let deaths = s.combat.deaths.len();
    s.cast_counterintel(1, foe_spy);
    assert!(!s.units[foe_spy].alive(), "the enemy Spy is dead");
    assert_eq!(s.combat.deaths.len(), deaths + 1, "a death object, dtype 2");
    s.units[filler].infiltrated = 1;
    let owner = s.units[filler].owner;
    s.cast_counterintel(owner, filler);
    assert!(s.units[filler].alive(), "one's own unit lives");
    assert_eq!(s.units[filler].infiltrated, 0, "and is cleared");
}

/// **Bribe's conversion** (`SpellType::cast_bribe@00672d00`'s unit arm,
/// `Unit::swap_team@006118d0`, item 2823): a copy is born under the
/// caster's owner carrying the damage and the facing, the old unit is
/// closed without a death and held thirty frames, and the caster's
/// leader counts one. **The two angles are copied each from its own**
/// (item 3999, run1554's `1/7` → `0/10`): the figure's `angle`
/// (`facing`) and the unit's (`heading`), so a copy of a figure still
/// turning is born still turning — and with `guy_flags & 2` clear
/// (`frame_facing` on its `facing`), so its first `Guy::move` turns it.
/// Made to fail with the counter dropped, with the old unit left
/// standing, with the damage not carried, with `heading` copied into
/// both (item 3985's reading), with `facing` copied into both, and with
/// `frame_facing` left at the birth's (the seeded facing is not
/// [`crate::movement::Angle::INITIAL`], or that one passes).
#[test]
fn bribe_swaps_a_unit_s_team_and_closes_the_old_one() {
    let (mut s, [_, spy, _, filler]) = casters(Tuning::RON);
    let target = s.add_unit({
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = s.units[filler].ty;
        u
    });
    s.units[spy].owner = 1;
    s.units[target].health = 15;
    s.units[target].max_health = 20;
    s.units[target].movement.heading = crate::movement::Angle(0x1200_0000);
    s.units[target].movement.facing = crate::movement::Angle(0x6600_0000);
    let bribes = s.tech[1].bribes;
    let before = s.units.len();
    let n = s.cast_bribe_unit(spy, target).expect("a copy is born");
    assert!(s.units.len() > before || n < before, "a unit is made");
    assert_eq!(s.units[n].owner, 1, "under the caster's owner");
    assert_eq!(
        s.units[n].max_health - s.units[n].health,
        5,
        "damage carried"
    );
    assert_eq!(
        s.units[n].movement.facing,
        crate::movement::Angle(0x6600_0000),
        "the figure's angle (`GuyData +0x18`)"
    );
    assert_eq!(
        s.units[n].movement.heading,
        crate::movement::Angle(0x1200_0000),
        "the unit's angle (`UnitData +0x50`)"
    );
    assert_eq!(
        s.units[n].movement.frame_facing, s.units[n].movement.facing,
        "the copy is no turn: `guy_flags & 2` clear"
    );
    assert!(!s.units[target].alive(), "the old one is closed");
    assert_eq!(s.units[target].hold_frames, Sim::CLOSE_HOLD);
    assert_eq!(s.tech[1].bribes, bribes + 1, "the leader's +0x8f4");
    assert!(s.cast_bribe_unit(spy, n).is_none(), "its own: nothing");
}

/// What one frame of each parity takes back from a `mana_burn` of 100.
fn steps(s: &mut Sim, u: usize) -> [i16; 2] {
    [10, 11].map(|frame| {
        s.units[u].mana_burn = 100;
        s.recover_mana(u, frame);
        100 - s.units[u].mana_burn
    })
}

/// **`SPIES_GENERALS_RECOVER_CRAFT` doubles a General's and a Spy's
/// rate** (`Unit::process@00610bc0:610f2d`, item 1673): the Persian
/// General `1/150` of the coverage pair runs `mana_burn` 12, 10, ... 0
/// on 3838..3844 under All Technologies, two a frame, and so ours' 480
/// on 3864 kept it from the Create Decoys cast the original laid. The
/// gate is the bonus's prerequisite tech held; the doubling is `r *=
/// 2` on a General or a Spy, not on any other caster.
///
/// Made to fail with the arm dropped and with the gate read as held.
#[test]
fn the_recover_craft_bonus_doubles_a_generals_and_a_spys_rate() {
    let (mut s, [general, spy, _, filler]) = casters(Tuning::RON);
    s.tables.tech_tree.roles.spy_general_recover_preq = Some(0x40);
    s.tech[1].tech.resize(0x41, false);
    assert_eq!(steps(&mut s, general), [1, 1], "the prerequisite not held");
    s.tech[1].tech[0x40] = true;
    assert_eq!(steps(&mut s, general), [2, 2], "held: the General");
    assert_eq!(steps(&mut s, spy), [2, 2], "held: the Spy");
    assert_eq!(steps(&mut s, filler), [1, 1], "held: a caster of no line");
}

/// **A supplier's pool is `MANA × (get_supply_upgrade + 1)`**
/// (`UnitData::mana@00609a50`, `609a8f`..`609aac`; item 3630,
/// `docs/AI.md` §331): run1292's Monarch `1/17` (TypeIndex 354, a
/// supplier) holds 2000 under one supply upgrade on 1946, so the hero
/// arm's Create Decoy is afforded at `mana_burn` 767 where 1000 was
/// not. The steps are a count, not a prefix; a General (no
/// `SUPPLY_OR_HERO`) and another leader's supplier keep `MANA`.
///
/// Made to fail with the multiple dropped and with it on every unit.
#[test]
fn a_suppliers_pool_grows_with_the_supply_upgrade() {
    use crate::ai::ai_load::{UnitCols, uflags2};
    let mut s = Sim::new(Tuning::RON, World::new(8, 8), 2);
    let mut tree = crate::tech::TechTree::new();
    let steps = [
        tree.add(TypeDef::plain("step one", 0)),
        tree.add(TypeDef::plain("step two", 0)),
        tree.add(TypeDef::plain("step three", 0)),
    ];
    tree.roles.supply_upgrade_preq = steps.map(Some);
    s.set_tech_tree(tree);
    let ty = |s: &mut Sim, flags2: u32| {
        s.add_unit_type(UnitType {
            mana: 1000,
            cols: UnitCols {
                unit_flags2: flags2,
                ..UnitCols::default()
            },
            ..UnitType::default()
        })
    };
    let monarch = ty(&mut s, uflags2::SUPPLY_OR_HERO);
    let general = ty(&mut s, uflags2::FLAG2_GENERAL);
    let unit = |s: &mut Sim, who: u8, t: usize| {
        let mut u = Unit::new(who, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.ty = Some(t);
        s.add_unit(u)
    };
    let mine = unit(&mut s, 1, monarch);
    let theirs = unit(&mut s, 0, monarch);
    let hero = unit(&mut s, 1, general);
    assert_eq!(s.unit_mana(mine), 1000, "no upgrade");
    s.gain_tech(1, steps[2]);
    assert_eq!(s.unit_mana(mine), 2000, "one step, the last");
    assert_eq!(s.unit_mana(theirs), 1000, "another leader's supplier");
    assert_eq!(s.unit_mana(hero), 1000, "a General is no supplier");
    s.units[mine].mana_burn = 767;
    assert_eq!(s.mana_left(mine), 1233, "Create Decoy's 1000 afforded");
    s.gain_tech(1, steps[0]);
    s.gain_tech(1, steps[1]);
    assert_eq!(s.unit_mana(mine), 4000, "all three");
}

/// **A French General takes back two a frame** (`docs/AI.md` §119,
/// item 1470): `Unit::process@00610bc0`'s `r` is 4 under
/// `has_tribe_bonus(10)` and `is(GENERAL, 1)`, and `((frame & 1) + r)
/// / 2` is 2 on either parity; anyone else's `r = 2` is 1 on either.
/// run632's `1/79`, a French General, 102 on 11376 and 64 on 11395.
/// Memnon's `MEMNON_REGEN_RATE` 2/1 (512) doubles his `r` to 4, and a
/// modded 3/2 (384) takes it to 3: one and two by parity. A French
/// General's pool is `(FRENCH_SPECIAL_CRAFT + 100) × MANA / 100`.
///
/// Made to fail with the craft rate back at 2 for everyone, and with
/// the bonus on the Spy (`is(GENERAL, 1)` alone gates it).
#[test]
fn a_french_general_recovers_craft_twice_as_fast() {
    let (mut s, [general, spy, memnon, filler]) = casters(Tuning::RON);
    assert_eq!(steps(&mut s, general), [1, 1], "no nation power");
    s.tech[1].power = Some(crate::cities::nations::power::FRENCH);
    assert_eq!(steps(&mut s, general), [2, 2], "the French General");
    assert_eq!(steps(&mut s, spy), [1, 1], "a French Spy");
    assert_eq!(steps(&mut s, filler), [1, 1], "a French caster of no line");
    assert_eq!(steps(&mut s, memnon), [2, 2], "Memnon, 512 × 2 >> 8");
    assert_eq!(s.unit_mana(general), 1000, "the bonus ships as 0%");

    let mut modded = Tuning::RON;
    modded.memnon_regen_rate = 384;
    modded.french_special_craft = 20;
    let (mut s, [general, _, memnon, filler]) = casters(modded);
    assert_eq!(steps(&mut s, memnon), [1, 2], "Memnon at 3/2: r = 3");
    assert_eq!(s.unit_mana(general), 1000, "not French");
    s.tech[1].power = Some(crate::cities::nations::power::FRENCH);
    assert_eq!(s.unit_mana(general), 1200, "(20 + 100) × 1000 / 100");
    assert_eq!(s.unit_mana(filler), 1000, "only a General's pool");
}
