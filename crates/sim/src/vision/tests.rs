use super::*;
use crate::Tuning;
use crate::ai_place::circle;
use crate::movement::Angle;
use crate::world::{Cell, Terrain, World};

/// A 40 × 40 land world under an all-dark fog grid, one player, one
/// unit of a type whose `LOS` the caller picks.
fn fog_sim(los: i32, science_los: i32) -> (crate::Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    assert!(world.set_fog(vec![0; 80 * 80]));
    let mut s = crate::Sim::new(Tuning::RON, world, 2);
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        los,
        science_los,
        ..crate::UnitType::default()
    });
    let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
    let mut u = crate::Unit::new(0, 0, at, 20);
    u.ty = Some(t);
    let u = s.add_unit(u);
    (s, u)
}

/// How many fog cells this player has ever seen.
fn seen(s: &crate::Sim, who: u8) -> usize {
    let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
    (0..fh)
        .flat_map(|y| (0..fw).map(move |x| (x, y)))
        .filter(|&(x, y)| s.world.seen2(x, y).is_some_and(|v| v & (1 << who) != 0))
        .count()
}

/// The same world with one **building** of the caller's `LOS`,
/// `SCIENCE_LOS` and footprint, finished and on a cell centre.
fn fog_build(los: i32, science_los: i32, x_size: i32) -> (crate::Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    assert!(world.set_fog(vec![0; 80 * 80]));
    let mut s = crate::Sim::new(Tuning::RON, world, 2);
    let t = s.add_build_type(crate::build::BuildType {
        los,
        science_los,
        x_size,
        y_size: x_size,
        ..crate::build::BuildType::default()
    });
    let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
    let b = s.add_building(0, at, 0);
    s.buildings[b].ty = Some(t);
    // `Wall::init`'s `update_los`: the cache the sweep reads.
    s.update_build_los(b);
    (s, b)
}

/// **`UnitData::los@006100c0`'s CEO term** (item 3524): a non-siege
/// unit whose leader counts a `0x165` and stands within the hero's
/// radius sees `THECEO_UNIT_LOS` (2) past its cached `mylos`; out of
/// reach, a siege unit, or a leader whose muster counts no CEO, it sees
/// the cache bare. Made to fail with the term dropped.
#[test]
fn a_unit_beside_its_own_ceo_sees_two_tiles_further() {
    let (mut s, u) = fog_sim(8, 0);
    s.units[u].mylos = 8;
    let ceo = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        los: 8,
        type_index: 0x165,
        ..crate::UnitType::default()
    });
    s.tables.unit_types[ceo].cols.unit_flags2 |= crate::ai_load::uflags2::FLAG2_GENERAL;
    let at = s.units[u].pos;
    let mut h = crate::Unit::new(0, 1, Pos::new(at.x + 0x300, at.y), 20);
    h.ty = Some(ceo);
    h.type_index = 0x165;
    let h = s.add_unit(h);
    assert_eq!(s.unit_data_los(u), 8, "the muster counts no CEO");
    s.roster.muster[0].by_type[ceo] = 1;
    assert_eq!(s.unit_data_los(u), 10, "beside the CEO");
    assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(5));
    let far = s.hero_radius(h) * 0xc0 + 0x300;
    s.units[h].pos = Pos::new(at.x + far, at.y);
    assert_eq!(s.unit_data_los(u), 8, "out of the hero's radius");
    s.units[h].pos = Pos::new(at.x + 0x300, at.y);
    let t = s.units[u].ty.unwrap();
    s.tables.unit_types[t].combat.siege = true;
    assert_eq!(s.unit_data_los(u), 8, "a siege unit");
}

/// §2.1: a building's `mylos` is its type's `LOS`, plus the science
/// term, plus **half its footprint** — the tail
/// `Wall::update_los@0063eeb0` adds last and the unit's own
/// `update_los` has no counterpart to.
///
/// The three run39 opens with are the oracle: a Small City
/// (`LOS 12`, `X_SIZE 7`) at `mylos 15`, a Woodcutter's Camp
/// (`6`, `2`) at `7`, a Farm (`6`, `4`) at `8` — and the city at
/// **17** once its Science epoch is 1, `SCIENCE_LOS` being 2.
#[test]
fn a_buildings_los_adds_half_its_footprint_and_the_science_term() {
    for (los, x, want) in [(12, 7, 15), (6, 2, 7), (6, 4, 8), (8, 4, 10)] {
        let (s, b) = fog_build(los, 0, x);
        assert_eq!(s.build_los(b), want, "LOS {los}, X_SIZE {x}");
    }
    let (mut s, b) = fog_build(12, 2, 7);
    assert_eq!(s.build_los(b), 15);
    s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
    assert_eq!(s.build_los(b), 17, "the science term");
    // The sweep reads the cache, which the epoch has not refreshed
    // (parked 3965a, item 3970): the old disc until `update_los`.
    assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(7));
    s.update_build_los(b);
    assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(8));
}

/// §6.2: **first contact is a building of one leader entering the
/// other's current line of sight**, and nothing else here reaches it.
///
/// The met bit is `treaties[other] & 1`, [`Sim::treaty_on`] writes it
/// on both sides, and `Wall::check_ever_seen@0063ce70`'s tail is the
/// only live caller of `Leader::meet`. So: a player-0 building
/// standing in the dark, a player-1 unit that has not looked at it,
/// and the bit stays clear however often the building checks — then
/// the unit's disc lands on the footprint and the **next** check sets
/// it, on both leaders at once.
///
/// The order matters and is asserted: the bit follows `ever_seen`
/// rather than the fog. A leader whose unit is standing on the
/// building's own ground has still not met its owner until the
/// building's own eighth-frame check runs.
#[test]
fn a_building_seen_by_the_enemy_is_first_contact() {
    let (mut s, b) = fog_build(6, 0, 4);
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        los: 16,
        ..crate::UnitType::default()
    });
    // Nothing has looked at anything yet — asserted **before** the
    // scout exists, because a unit born on the map lights its own disc
    // (`Object::add_to_world`, item 447).
    s.check_ever_seen(b, false);
    assert_eq!(
        s.buildings[b].ever_seen, 0,
        "nobody's line of sight is on the footprint yet — not even the \
             owner's, since the building has not lit itself"
    );
    assert!(
        !s.has_met(0, 1) && !s.has_met(1, 0),
        "nobody has met anybody"
    );
    assert_eq!(s.treaties[0][1], 0);

    // Player 1's scout, three cells east of player 0's building. Its
    // disc lands as it is born, and the fog now carries player 1's bit
    // over the footprint — which alone is **not** contact.
    let at = Pos::new(23 * 0x300 + 0x180, 20 * 0x300 + 0x180);
    let mut u = crate::Unit::new(1, 0, at, 20);
    u.ty = Some(t);
    let u = s.add_unit(u);
    assert_eq!(
        s.update_seen(u, false),
        0,
        "the scout's disc landed at its birth, so nothing is new"
    );
    assert!(
        !s.has_met(1, 0),
        "the fog is lit but the building has not checked"
    );

    // The building checks, and the two leaders have met — both ways,
    // which is `treaty_on`'s own double write.
    s.check_ever_seen(b, false);
    assert_eq!(
        s.buildings[b].ever_seen, 0b10,
        "player 1 has looked at it, and only player 1"
    );
    assert!(s.has_met(0, 1) && s.has_met(1, 0), "first contact");
    assert_eq!((s.treaties[0][1], s.treaties[1][0]), (1, 1));

    // And it is never cleared: `Leader::treaty_off` has no caller in a
    // game. A second check changes nothing.
    s.check_ever_seen(b, false);
    assert_eq!((s.treaties[0][1], s.treaties[1][0]), (1, 1));
}

/// A 4 × 4 site of player 1's, placed and not started, in a two-player
/// fogged world; `wonder` picks `BuildData::is_wonder`.
fn site(wonder: bool) -> (crate::Sim, usize) {
    let (mut s, b) = fog_build(6, 0, 4);
    let t = s.buildings[b].ty.unwrap();
    s.tables.build_types[t].wonder = wonder;
    let bd = &mut s.buildings[b];
    bd.owner = 1;
    (bd.started, bd.active) = (false, false);
    (s, b)
}

/// How many fog cells carry every bit in the **current** plane.
fn lit_for_all(s: &crate::Sim) -> usize {
    let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
    (0..fh)
        .flat_map(|y| (0..fw).map(move |x| (x, y)))
        .filter(|&(x, y)| s.world.seen(x, y) == Some(0xff))
        .count()
}

/// **§6.4: a wonder's start is first contact** (item 1446).
/// `Build::start@006273a0` ends with vslot `+0x164` for a wonder, whose
/// write is `0xff` into `seen` as well as `seen2`; `Wall::start`'s own
/// `check_ever_seen` has already run, so the bits are taken at the
/// owner's next check — and with them player 0, never having looked,
/// has met player 1. run613 dates it in the original: the wonder starts
/// on 7941 and both met bits are set on block 7946.
///
/// Made to fail on purpose (item 1446): with the `+0x164` call removed
/// from `start_building`, or with the wonder's write sent to `seen2`
/// only, the current plane is dark over the footprint (0 of 16).
#[test]
fn a_wonder_s_start_is_first_contact() {
    let (mut s, b) = site(true);
    s.start_building(b);
    // The grown rectangle, 6 × 6 tiles from an even corner, is 4 × 4
    // half-cells: tiles c − 1 ..= c + 4 map to c/2 − 1 ..= c/2 + 2.
    assert_eq!(lit_for_all(&s), 16, "the grown footprint, every player");
    assert_eq!(
        s.buildings[b].ever_seen, 0,
        "Wall::start's own check ran before the reveal"
    );
    assert!(!s.has_met(0, 1));
    s.check_ever_seen(b, false);
    assert_eq!(s.buildings[b].ever_seen, 0xff, "every bit, as run613 reads");
    assert!(s.has_met(0, 1) && s.has_met(1, 0), "first contact");

    // An ordinary site starts dark for everyone else.
    let (mut s, b) = site(false);
    s.start_building(b);
    assert_eq!(lit_for_all(&s), 0);
    s.check_ever_seen(b, false);
    assert!(!s.has_met(0, 1), "no contact from an ordinary start");
}

/// **A wonder with the city flag is not a visible wonder** (twenty-fourth
/// pass, group 5; A8 row 4): the Forbidden City is in the wonder range and
/// `FROM Small City`, so `is(VILLAGE)` holds and `update_local_seen`
/// writes it as an ordinary building — `seen2` only, never `0xff`. A fort
/// type takes the same branch.
#[test]
fn a_city_flagged_wonder_is_not_first_contact() {
    for as_fort in [false, true] {
        let (mut s, b) = site(true);
        let root = s.add_build_type(crate::build::BuildType {
            ident: if as_fort {
                crate::build::Ident::Fort
            } else {
                crate::build::Ident::Village
            },
            ..crate::build::BuildType::default()
        });
        let t = s.buildings[b].ty.unwrap();
        s.tables.build_types[t].from = Some(root);
        s.start_building(b);
        assert_eq!(lit_for_all(&s), 0, "no seen-plane write, fort={as_fort}");
        s.check_ever_seen(b, false);
        assert!(!s.has_met(0, 1), "no first contact, fort={as_fort}");
    }
}

/// **The meet loop gates on a slot in use, not on "not defeated"**
/// (group 6; A8 row 15): a leader that has been defeated is met by a
/// building it sees.
#[test]
fn a_defeated_leader_is_still_met() {
    let (mut s, b) = site(true);
    s.defeated[0] = true;
    s.start_building(b);
    s.check_ever_seen(b, false);
    assert!(s.has_met(0, 1), "the defeated bit is not leader_flags & 1");
}

/// **A finished wonder is relit after every clear** (group 19; A8 row
/// 10): the whole-disc call makes its own `+0x164` first for a started
/// visible wonder, so its grown footprint carries every player's bit.
#[test]
fn a_finished_wonder_is_relit_after_the_clear() {
    let (mut s, b) = site(true);
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    s.update_all_seen();
    assert_eq!(lit_for_all(&s), 16, "the finished wonder's footprint");
    let (mut s, b) = site(false);
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    s.update_all_seen();
    assert_eq!(lit_for_all(&s), 0, "an ordinary building is not");
}

/// **§6.4: the resync relights a wonder under construction** and lights
/// no disc for an unfinished ordinary site (`GameDaemon::update_all_seen
/// @00732840`'s not-active arm; `Wall::update_los` is 0 for such a site
/// in any case). Made to fail on purpose (item 1446): the old
/// every-building disc fails the wonder's relight (0 of 16), and a disc
/// for an unfinished ordinary site fails the second half.
#[test]
fn the_resync_relights_a_started_wonder_and_no_unfinished_site() {
    let (mut s, b) = site(true);
    s.buildings[b].started = true;
    s.update_all_seen();
    assert_eq!(
        lit_for_all(&s),
        16,
        "the wonder is lit again after the clear"
    );

    let (mut s, b) = site(false);
    s.buildings[b].started = true;
    s.update_all_seen();
    let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
    let any = (0..fh)
        .flat_map(|y| (0..fw).map(move |x| (x, y)))
        .any(|(x, y)| s.world.seen(x, y).unwrap_or(0) != 0);
    assert!(!any, "an unfinished ordinary site lights nothing");
}

/// §6.2's other half: `meet` fires on the **new** bit only, so a
/// leader already met does not meet again — and a building nobody but
/// its owner has seen never fires it at all.
///
/// Made to fail on purpose by dropping the `old & m == 0` test, which
/// makes every later check a fresh contact; the assertion that catches
/// it is the census's, not the bit's, so what is checked here is the
/// thing a bit cannot show: `ever_seen`'s own monotonicity, and that
/// an unstarted building filters the mask down to its owner's allies
/// and therefore cannot be the thing that introduces two enemies.
#[test]
fn an_unstarted_building_cannot_introduce_two_enemies() {
    let (mut s, b) = fog_build(6, 0, 4);
    s.buildings[b].started = false;
    s.buildings[b].active = false;
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        los: 16,
        ..crate::UnitType::default()
    });
    let at = Pos::new(23 * 0x300 + 0x180, 20 * 0x300 + 0x180);
    let mut u = crate::Unit::new(1, 0, at, 20);
    u.ty = Some(t);
    // The disc lands at birth (item 447), so the look has happened by
    // the time the building checks.
    let u = s.add_unit(u);
    assert!(seen(&s, 1) > 0, "the scout's disc is on the grid");
    assert_eq!(s.update_seen(u, false), 0, "and nothing is new");
    s.check_ever_seen(b, false);
    assert_eq!(
        s.buildings[b].ever_seen, 0,
        "an unstarted building takes the owner's ally mask only, and the \
             enemy's bit is not in it"
    );
    assert!(!s.has_met(1, 0), "a foundation is not a sighting");
    // Started, the same look lands.
    s.buildings[b].started = true;
    s.check_ever_seen(b, false);
    assert!(s.has_met(1, 0), "a started building is");
}

/// §2.1's tower term (item 3543): a finished Tower-line building that
/// is not a fort sees `TOWER_FORT_LOS[get_tower_fort_los − 1]` past
/// `LOS + x_size / 2`, and `get_tower_fort_los` is the highest of the
/// three unit bits held. The Turks' tower `1/2012` (`LOS 14`, `X_SIZE
/// 2`) is 19 in run1186 with the second bit, 15 here without the term.
/// A fort takes none of it (its own term is not carried), nor does an
/// unfinished tower.
#[test]
fn a_tower_sees_further_with_each_tower_fort_los_bit() {
    use crate::build::Ident;
    use crate::tech::{TechTree, TypeDef, UnitTraits};
    let (mut s, b) = fog_build(14, 0, 2);
    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    let bits = ["Militia", "Minuteman", "Partisan"]
        .map(|n| tree.add(TypeDef::unit(n, UnitTraits::default())));
    tree.roles.tower_fort_los_units = bits.map(Some);
    s.set_tech_tree(tree);
    let t = s.buildings[b].ty.expect("typed");
    assert_eq!(s.build_los(b), 15, "not a tower");
    s.tables.build_types[t].ident = Ident::Tower;
    assert_eq!(s.build_los(b), 15, "no bit: get_tower_fort_los 1, entry 0");
    for (k, want) in [(0, 17), (1, 19), (2, 21)] {
        s.tech[0].tech[bits[k]] = true;
        assert_eq!(s.build_los(b), want, "bit {k}");
    }
    s.tech[0].tech[bits[2]] = false;
    assert_eq!(s.build_los(b), 19, "the Turks' 1/2012");
    assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(7), "the cache");
    s.update_build_los(b);
    assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(9));
    s.buildings[b].active = false;
    assert_eq!(s.build_los(b), 0, "an unfinished tower");
    s.buildings[b].active = true;
    s.tables.build_types[t].ident = Ident::Fort;
    assert_eq!(s.build_los(b), 15, "a fort");
}

/// §2.1's fort arm (item 3631): a finished Fort-line building adds
/// `FORT_UPGRADE_LOS[get_fort_los − 1]` and never the tower term;
/// `get_fort_los` is `1` plus the highest `FORTLOS2..4` held. run1277's
/// Roman Fort `1/2013` (`LOS 12`, `X_SIZE 5`) prints `mylos 16` at level
/// 2, a radius of eight fog cells.
#[test]
fn a_fort_sees_further_with_each_fort_los_level() {
    use crate::build::Ident;
    use crate::tech::{TechTree, TypeDef};
    let (mut s, b) = fog_build(12, 0, 5);
    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    let levels = ["Fort LOS 2", "Fort LOS 3", "Fort LOS 4"].map(|n| tree.add(TypeDef::plain(n, 0)));
    tree.roles.fort_los_preq = levels.map(Some);
    s.set_tech_tree(tree);
    let t = s.buildings[b].ty.expect("typed");
    s.tables.build_types[t].ident = Ident::Fort;
    assert_eq!(s.build_los(b), 14, "level 1: entry 0");
    for (k, want) in [(0, 16), (1, 18), (2, 20)] {
        s.tech[0].tech[levels[k]] = true;
        assert_eq!(s.build_los(b), want, "level {}", k + 2);
    }
    s.tech[0].tech[levels[1]] = false;
    s.tech[0].tech[levels[2]] = false;
    assert_eq!(s.build_los(b), 16, "the Romans' 1/2013");
    s.update_build_los(b);
    assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(8));
    s.tables.build_types[t].ident = Ident::Tower;
    assert_eq!(s.build_los(b), 14, "a tower takes no fort term");
}

/// §2.1's head: an unstarted building sees nothing, an unfinished
/// ordinary one nothing, and an unfinished wonder `1 + x_size / 2` —
/// never its type's `LOS`.
#[test]
fn an_unfinished_building_sees_one_plus_half_its_footprint() {
    let (mut s, b) = fog_build(12, 0, 7);
    s.buildings[b].active = false;
    assert_eq!(s.build_los(b), 0, "an ordinary site: `is_wonder` is 0");
    let t = s.buildings[b].ty.expect("typed");
    s.tables.build_types[t].wonder = true;
    assert_eq!(s.build_los(b), 4);
    s.buildings[b].started = false;
    assert_eq!(s.build_los(b), 0);
    s.update_build_los(b);
    assert!(s.build_sweep(b).is_none());
}

/// §2.1: a building's `mylos` is a **cache** (item 3965). A wonder
/// started after its placement keeps the `0` `Wall::init` left until
/// its owner's next `calc_wall_stats`, and a Science epoch reaches a
/// finished building only there — run718's wonder sites `1/2055` and
/// `1/2057` print 0 from their start to 2368, and the city `1/2018`
/// goes 29 → 31 on 2430, the leader pass after the gain.
#[test]
fn a_buildings_mylos_is_refreshed_only_where_the_original_calls_update_los() {
    let (mut s, b) = fog_build(12, 0, 7);
    let t = s.buildings[b].ty.expect("typed");
    s.buildings[b].active = false;
    s.buildings[b].started = false;
    s.tables.build_types[t].wonder = true;
    s.update_build_los(b);
    assert_eq!(s.buildings[b].mylos, 0, "unstarted at `Wall::init`");
    s.buildings[b].started = true;
    assert_eq!(s.build_los(b), 4, "the derivation moves at once");
    assert_eq!(s.buildings[b].mylos, 0, "the cache does not");
    s.calc_wall_stats(0);
    assert_eq!(s.buildings[b].mylos, 4, "`calc_wall_stats` refreshes it");
    s.buildings[b].active = true;
    s.update_build_los(b);
    assert_eq!(s.buildings[b].mylos, 15);
    s.tables.build_types[t].science_los = 2;
    s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
    assert_eq!(s.buildings[b].mylos, 15, "the gain alone moves nothing");
    s.calc_wall_stats(0);
    assert_eq!(s.buildings[b].mylos, 17);
}

/// §6: the disc a finished building throws is the **whole** circle at
/// its own half-cell — never §3's projected centre, which is an
/// `is_unit()` case — and `update_all_seen` throws it again.
#[test]
fn a_finished_building_lights_its_whole_disc_from_its_own_half_cell() {
    let (mut s, b) = fog_build(6, 0, 4);
    let own = (fog_of(s.buildings[b].pos.x), fog_of(s.buildings[b].pos.y));
    let sw = s.build_sweep(b).expect("a finished building sees");
    assert_eq!(
        (sw.centre, sw.radius, sw.ring, sw.start),
        (own, 4, false, 0)
    );
    assert_eq!(sw.end, circle().radius[4]);
    assert_eq!(s.update_seen_build(b), sw.end, "every point is new");
    assert_eq!(seen(&s, 0), sw.end);
    // The half-cell four east is inside the disc and the one six south
    // of that is not — the octagonal radius, not a square.
    assert!(s.world.seen2(own.0 + 4, own.1).is_some_and(|v| v & 1 != 0));
    assert!(
        s.world
            .seen2(own.0 + 4, own.1 + 6)
            .is_some_and(|v| v & 1 == 0)
    );
    // Idempotent, and the hundred-frame pass runs it for buildings too.
    s.update_all_seen();
    assert_eq!(seen(&s, 0), sw.end);
}

/// §3: `LOS` is in tiles and the fog radius is half of it, truncating.
/// The five the Ancient age actually fields, and the cap.
#[test]
fn the_radius_is_half_the_los_in_tiles() {
    for (los, want) in [
        (0, None),
        (2, Some(1)),
        (4, Some(2)),
        (6, Some(3)),
        (8, Some(4)),
        (11, Some(5)),
        (200, Some(0x40)),
    ] {
        let (s, u) = fog_sim(los, 0);
        assert_eq!(
            s.seen_sweep(u, false).map(|w| w.radius),
            want,
            "LOS {los} tiles"
        );
    }
}

/// §2 term 4: the science line multiplies `SCIENCE_LOS`, and it is
/// `epoch[3]` — a Scout's `4 + 1 × 2 = 6`, which run10's frame 202
/// confirms against the original's own `mylos`.
#[test]
fn a_science_level_adds_science_los_and_the_radius_follows() {
    let (mut s, u) = fog_sim(4, 2);
    assert_eq!(s.unit_los(u), 4);
    assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(2));
    s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
    assert_eq!(s.unit_los(u), 6);
    // §2: `mylos` is a cache. The sweep reads what `update_los` last
    // wrote, so the level reaches the fog only at the next refresh.
    assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(2));
    s.update_los(u);
    assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(3));
}

/// Term 6's fixture: a Barracks and a Market, a unit trained at each
/// (`LOS 6`), and Herbal Lore as `TROOPS_LOS_1`. Player 0 is the AI.
fn troops_sim() -> (crate::Sim, usize, usize, crate::tech::TypeId) {
    use crate::build::{BuildType, Ident};
    use crate::tech::{TechTree, TypeDef};
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    assert!(world.set_fog(vec![0; 80 * 80]));
    let mut s = crate::Sim::new(Tuning::RON, world, 2);
    let barracks = s.add_build_type(BuildType {
        ident: Ident::Barracks,
        ..BuildType::default()
    });
    let market = s.add_build_type(BuildType {
        ident: Ident::Market,
        ..BuildType::default()
    });
    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    let barracks_id = tree.add(TypeDef::building("Barracks"));
    let market_id = tree.add(TypeDef::building("Market"));
    let herbal = tree.add(TypeDef::plain("Herbal Lore", 0));
    let explorer_id =
        tree.add(TypeDef::unit("Explorer", crate::tech::UnitTraits::default()).at(barracks_id));
    let merchant_id =
        tree.add(TypeDef::unit("Merchant", crate::tech::UnitTraits::default()).at(market_id));
    tree.roles.troops_los_preq = [Some(herbal), None, None];
    s.set_tech_tree(tree);
    s.tables.build_types[barracks].tree = Some(barracks_id);
    s.tables.build_types[market].tree = Some(market_id);
    let mut add = |tree_id| {
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            los: 6,
            tree: Some(tree_id),
            ..crate::UnitType::default()
        });
        let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(0, 0, at, 20);
        u.ty = Some(t);
        s.add_unit(u)
    };
    let (explorer, merchant) = (add(explorer_id), add(merchant_id));
    (s, explorer, merchant, herbal)
}

/// §2 term 6: `TROOPS_UPGRADE_LOS` per `TROOPS_LOS_n` held, for a unit
/// whose trainer is the Barracks, the Stable or the Auto Plant — and
/// for nothing else. run529's who=1 Explorer is 12 and then 14, and
/// its Market-trained Merchant stays where it was.
#[test]
fn a_troops_los_level_adds_two_tiles_to_a_barracks_unit_only() {
    let (mut s, explorer, merchant, herbal) = troops_sim();
    assert_eq!((s.unit_los(explorer), s.unit_los(merchant)), (6, 6));
    s.tech[0].tech[herbal] = true;
    assert_eq!(s.troops_los_level(0), 1);
    assert_eq!(
        (s.unit_los(explorer), s.unit_los(merchant)),
        (6 + s.tables.tuning.troops_upgrade_los, 6)
    );
    assert_eq!(s.tables.tuning.troops_upgrade_los, 2);
}

/// §2: the tech reaches a unit's `mylos` through `gain_tech`'s tail —
/// `leader_flags |= 0xc000000` — and the next `Leader::process`'s
/// `calc_unit_stats`, not on the frame it is gained. run529: Herbal
/// Lore on 9782, every Barracks and Stable unit two tiles more from
/// block 9784.
#[test]
fn a_gained_tech_reaches_the_cache_at_the_next_leader_pass() {
    let (mut s, explorer, _, herbal) = troops_sim();
    assert_eq!(s.units[explorer].mylos, 6);
    s.gain_tech(0, herbal);
    assert_eq!(s.unit_los(explorer), 8, "the derivation sees it at once");
    assert_eq!(s.units[explorer].mylos, 6, "the cache does not");
    assert!(s.roster.unit_stats_dirty[0], "the tail raised 0x4000000");
    s.tick();
    assert_eq!(s.units[explorer].mylos, 8, "the next leader pass wrote it");
    assert!(!s.roster.unit_stats_dirty[0]);
}

/// §3: a small land unit sees from a half-cell in front of its nose,
/// not from where it stands. Heading east and heading west put the
/// centre on opposite sides of the unit's own fog cell.
#[test]
fn a_small_land_unit_sees_from_a_half_cell_ahead_of_its_heading() {
    let (mut s, u) = fog_sim(4, 0);
    let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    // `Angle` runs clockwise from north through the full `u32`; a
    // quarter turn is east, three quarters west (`docs/MOVEMENT.md`).
    s.units[u].movement.set_facing(Angle(0x4000_0000));
    let east = s.seen_sweep(u, false).unwrap().centre;
    s.units[u].movement.set_facing(Angle(-0x4000_0000));
    let west = s.seen_sweep(u, false).unwrap().centre;
    assert_eq!(east, (own.0 + 1, own.1), "heading east, one fog cell east");
    assert_eq!(west, (own.0 - 1, own.1), "heading west, one fog cell west");
}

/// §3, and the distinction item 79 turned on: the angle the disc is
/// thrown along is **`UnitData::angle`** — [`Movement::heading`] — not
/// the guy's eased [`Movement::facing`]. `set_facing` writes all three
/// together, so a test that only ever uses it cannot tell them apart;
/// this one sets them a half-turn apart, which is what a unit
/// mid-turn looks like.
#[test]
fn the_projection_follows_the_unit_s_heading_not_its_body_s_facing() {
    let (mut s, u) = fog_sim(4, 0);
    let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    s.units[u].movement.facing = Angle(-0x4000_0000);
    s.units[u].movement.heading = Angle(0x4000_0000);
    assert_eq!(
        s.seen_sweep(u, false).unwrap().centre,
        (own.0 + 1, own.1),
        "heading east while the body still points west"
    );
}

/// §3: **a decoy does not project** (`unit_masks & 1`, `00651cdc`): it
/// takes the own-cell arm, and its step's ring pass at radius 0 is the
/// whole one-cell disc — its own cell — where the projected arm's is
/// empty. The battery's French decoy Hussars `1/100`, `mylos` 1, lights
/// the Citizen `0/1`'s half-cell this way (item 3347).
#[test]
fn a_decoy_sees_from_where_it_stands_and_its_step_lights_its_own_cell() {
    let (mut s, u) = fog_sim(1, 0);
    let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    s.units[u].movement.set_facing(Angle(0x4000_0000));
    let projected = s.seen_sweep(u, true).unwrap();
    assert_eq!(projected.centre, (own.0 + 1, own.1));
    assert_eq!(
        projected.start, projected.end,
        "the projected ring pass is empty"
    );
    s.units[u].decoy = true;
    let decoy = s.seen_sweep(u, true).unwrap();
    assert_eq!(decoy.centre, own);
    assert_eq!((decoy.start, decoy.end), (0, 1), "its own cell");
}

/// §3: and a unit with radius four or more does not project — its
/// centre is its own fog cell whichever way it faces.
#[test]
fn a_unit_with_radius_four_or_more_sees_from_where_it_stands() {
    let (mut s, u) = fog_sim(8, 0);
    let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    s.units[u].movement.set_facing(Angle(0x4000_0000));
    assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
    s.units[u].movement.set_facing(Angle(-0x4000_0000));
    assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
}

/// §4: the moving pass walks the **ring** table only from radius four
/// up. Below it — every unit an Ancient game starts with — it is the
/// whole disc bar its centre, which is why a wrong ring table would not
/// be caught by any capture on disk.
#[test]
fn the_ring_table_is_reached_only_from_radius_four() {
    for (los, want_ring) in [(2, false), (4, false), (6, false), (8, true), (11, true)] {
        let (s, u) = fog_sim(los, 0);
        let sw = s.seen_sweep(u, true).unwrap();
        assert_eq!(sw.ring, want_ring, "LOS {los}");
        if want_ring {
            // The thickened annulus, out of the ring table's own
            // cumulative counts — not the circle's, which are smaller.
            let g = ring();
            let r = sw.radius as usize;
            assert_eq!((sw.start, sw.end), (g.radius[r - 1], g.radius[r]));
            assert_ne!(
                (sw.start, sw.end),
                (circle().radius[r - 1], circle().radius[r]),
                "LOS {los}: the ring table's slice is the circle's"
            );
        } else {
            assert_eq!(sw.start, 1, "LOS {los}: the disc bar its centre");
            assert_eq!(sw.end, circle().radius[sw.radius as usize]);
        }
    }
    // And the standing pass is the whole disc, centre included.
    let (s, u) = fog_sim(8, 0);
    let sw = s.seen_sweep(u, false).unwrap();
    assert!(!sw.ring);
    assert_eq!((sw.start, sw.end), (0, circle().radius[4]));
}

/// §5: the disc is written into `seen2`, and `update_seen` answers with
/// the count of cells newly revealed — the original's `reveal_fog`
/// calls. A second pass from the same spot reveals nothing.
///
/// **And the first pass is the unit's birth.** `Object::add_to_world`
/// reaches `update_seen(0)` and [`Sim::add_unit`] makes that call since
/// item 447, so `fog_sim` hands back a grid with the disc already on
/// it; before then a unit born on the map lit nothing at all until it
/// crossed a half-cell (`docs/COMBAT.md` §31.4). That is asserted here
/// rather than worked around: the count is the same disc either way,
/// and the difference is whose call made it.
#[test]
fn the_disc_lands_in_the_fog_and_only_new_cells_count() {
    let (mut s, u) = fog_sim(4, 0);
    let born = seen(&s, 0);
    assert_eq!(
        born,
        circle().radius[2],
        "the whole disc of radius two, lit at the unit's birth"
    );
    assert_eq!(s.update_seen(u, false), 0, "nothing new the second time");
    assert_eq!(seen(&s, 0), born);
}

/// §6: the trigger is a **half-cell** crossing, not any movement. A
/// step inside the unit's own fog cell writes nothing.
#[test]
fn only_a_step_that_crosses_a_half_cell_reveals() {
    let (mut s, u) = fog_sim(4, 0);
    s.update_seen(u, false);
    let base = seen(&s, 0);
    let from = s.units[u].pos;
    // Inside the same fog cell: it is `0x180` wide and the unit stands
    // at its middle.
    s.units[u].pos = Pos::new(from.x + 0x20, from.y);
    assert_eq!(
        s.moved_to(u, from, true),
        None,
        "a step inside the half-cell must not reach `update_seen` at all"
    );
    assert_eq!(seen(&s, 0), base);
    let from = s.units[u].pos;
    s.units[u].pos = Pos::new(from.x + 0x180, from.y);
    assert!(
        s.moved_to(u, from, true).is_some_and(|n| n > 0),
        "a step across a half-cell reveals"
    );
    assert!(seen(&s, 0) > base);
}

/// §6: and the hundredth-frame resync, which is what covers a unit that
/// never crosses one. Frame 33, not frame 0 and not frame 100.
#[test]
fn the_resync_runs_on_frame_thirty_three_of_each_hundred() {
    for (frame, reveals) in [
        (0, false),
        (32, false),
        (33, true),
        (100, false),
        (133, true),
    ] {
        let (mut s, u) = fog_sim(4, 0);
        // The birth disc is already on the grid (item 447), so what is
        // measured is the **delta**: the unit is teleported to ground
        // nobody has looked at — assigning `pos` reaches no reveal of
        // its own — and only the resync can light it.
        let base = seen(&s, 0);
        assert!(base > 0, "the birth disc");
        s.units[u].pos = Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180);
        s.frame = frame;
        s.tick();
        assert_eq!(
            seen(&s, 0) > base,
            reveals,
            "frame {frame}: the resync {}",
            if reveals { "should run" } else { "should not" }
        );
    }
}

/// §10: **the resync forgets what nothing sees any more.**
/// `update_all_seen` opens with `World::clear_seen@006b2250`, so the
/// current plane under ground the unit has walked off goes dark for
/// its owner, while `seen2`, the ever-seen plane, keeps it.
///
/// **Made to fail on purpose**: without the clear, the birth cell is
/// still in `seen` after the resync.
#[test]
fn the_resync_forgets_what_nothing_sees_any_more() {
    let (mut s, u) = fog_sim(4, 0);
    let born = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    assert_eq!(s.world.seen(born.0, born.1), Some(1), "the birth disc");
    s.units[u].pos = Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180);
    s.frame = 133;
    s.tick();
    assert_eq!(
        s.world.seen(born.0, born.1),
        Some(0),
        "nothing stands there now"
    );
    assert_eq!(s.world.seen2(born.0, born.1), Some(1), "ever seen");
    let now = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
    assert_eq!(s.world.seen(now.0, now.1), Some(1));
}

/// §10: **and it relights an attacker for its victims.** The whole-disc
/// `update_seen(0)` calls `Unit::update_local_seen` first when the
/// unit's `visible` byte is not 0 (`00651b80`), so a resync keeps the
/// attacker's own cell lit for the players it has attacked, and only
/// while the byte stands. Chapter eleven's guard is the diff: lit for
/// who=1 by 1033's resync, dark from 1133's (`docs/VISION.md` §10).
///
/// **Made to fail on purpose**: without the call in `update_seen`, the
/// victim's bit is gone after the resync whatever `visible` holds.
#[test]
fn the_resync_relights_an_attacker_for_its_victims() {
    for (visible, lit) in [(1u8 << 1, Some(0b11)), (0, Some(0b01))] {
        let (mut s, u) = fog_sim(4, 0);
        s.units[u].visible = visible;
        let at = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        s.frame = 133;
        s.tick();
        assert_eq!(
            s.world.seen(at.0, at.1),
            lit,
            "visible {visible:#04b}: who=1's bit on the attacker's cell"
        );
    }
}

/// §10.4: **a building target is seen through its `ever_seen` byte,
/// not the fog plane** (`BuildData::is_seen@0062e1a0` →
/// `WallData::is_seen@00642bd0`). A started building who=1 has once
/// laid eyes on stays a legal target of who=1's after the resync has
/// forgotten its cell; one it never saw is refused though the cell is
/// lit; and its owner always sees it. Great Lakes is the diff: without
/// this arm, the resync's clear dropped who=1's army's target on 8233
/// and the word fell 14982 → 9401.
///
/// **Made to fail on purpose**: through the fog plane instead, the
/// first and second rows answer the other way.
#[test]
fn a_building_target_is_seen_through_its_ever_seen_byte() {
    let (mut s, b) = fog_build(4, 0, 2);
    s.buildings[b].started = true;
    let (fx, fy) = (
        s.buildings[b].pos.x / UNITS_PER_FOG,
        s.buildings[b].pos.y / UNITS_PER_FOG,
    );
    for (ever, lit, seen) in [(0b10, 0, true), (0, 0b10, false), (0b10, 0b10, true)] {
        s.buildings[b].ever_seen = ever;
        s.world.clear_seen();
        s.world.set_seen(fx, fy, lit);
        assert_eq!(
            s.build_is_seen(b, 1),
            seen,
            "ever_seen {ever:#04b}, who=1's fog bit {lit:#04b}"
        );
    }
    s.buildings[b].ever_seen = 0;
    assert!(s.build_is_seen(b, 0), "the owner");
    s.buildings[b].started = false;
    s.buildings[b].ever_seen = 0b10;
    assert!(!s.build_is_seen(b, 1), "unstarted, and not an ally's");
}

/// **The unpack lights the whole disc** (`SpellType::cast_unpack
/// @006709c0`, `docs/COMBAT.md` §56): the packed bit clears, and the
/// next call is `update_seen(0)` at the new line of sight. A siege
/// engine born packed lights a four-tile disc; unpacked where it
/// stands, it crosses no half-cell and would keep that disc to the
/// next resync. run146's catapult is the diff: its search on the
/// frame after the unpack takes a hoplite seven tiles off.
///
/// **Made to fail on purpose**: without the call in `cast_unpack`, the
/// count after the unpack is the packed disc's.
#[test]
fn the_unpack_lights_the_whole_disc_at_the_new_line_of_sight() {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    assert!(world.set_fog(vec![0; 80 * 80]));
    let mut s = crate::Sim::new(Tuning::RON, world, 2);
    let mut ty = crate::UnitType {
        hits: 80,
        los: 10,
        ..crate::UnitType::default()
    };
    ty.combat.packs = true;
    ty.cols.unit_flags2 |= crate::ai_load::uflags2::PACKS;
    let t = s.add_unit_type(ty);
    let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
    let mut u = crate::Unit::new(0, 0, at, 80);
    u.ty = Some(t);
    let u = s.add_unit(u);
    assert!(s.units[u].combat.packed, "a type that packs is born packed");
    assert_eq!(s.unit_los(u), PACKED_LOS);
    assert_eq!(
        seen(&s, 0),
        circle().radius[2],
        "the packed disc: four tiles, radius two"
    );
    s.cast_unpack(u);
    assert_eq!(s.unit_los(u), 10);
    assert_eq!(
        seen(&s, 0),
        circle().radius[5],
        "the unpack lights the ten-tile disc, radius five, where it stands"
    );
}

/// **The Turkish siege term** (item 3380, `docs/VISION.md` §2 term 9):
/// a packed siege engine of a Turkish leader sees `TURK_SIEGE_LOS` past
/// the packed clamp — run1005's Trebuchets print `mylos 7`, not 4 — and
/// unpacked, past its own; another nation's sees neither.
#[test]
fn a_turkish_siege_engine_sees_three_tiles_past_the_packed_clamp() {
    let mut s = crate::Sim::new(Tuning::RON, World::new(40, 40), 2);
    let mut ty = crate::UnitType {
        hits: 80,
        los: 10,
        ..crate::UnitType::default()
    };
    ty.combat.packs = true;
    ty.combat.siege = true;
    ty.cols.unit_flags2 |= crate::ai_load::uflags2::PACKS;
    let t = s.add_unit_type(ty);
    let mut u = crate::Unit::new(1, 0, Pos::new(0x180, 0x180), 80);
    u.ty = Some(t);
    let u = s.add_unit(u);
    assert_eq!(s.unit_los(u), PACKED_LOS, "not Turkish");
    s.cities_state.nation[1].turks = true;
    assert_eq!(s.unit_los(u), PACKED_LOS + 3, "Turkish, packed");
    s.units[u].combat.packed = false;
    assert_eq!(s.unit_los(u), 13, "Turkish, unpacked");
}

/// A type with no `LOS` reveals nothing at all — the `mylos == 0` head,
/// which is what keeps a wall or a projectile out of the fog.
#[test]
fn a_type_with_no_los_reveals_nothing() {
    let (mut s, u) = fog_sim(0, 0);
    assert_eq!(s.seen_sweep(u, false), None);
    assert_eq!(s.update_seen(u, false), 0);
    assert_eq!(seen(&s, 0), 0);
}

/// §4: `ring_init`'s first rings against `circle_init`'s. Ring `r` is at
/// least the circle's own ring — it is copied out of it — and the patch
/// only ever adds.
#[test]
fn the_ring_table_contains_the_circles_own_ring_and_thickens_it() {
    let c = circle();
    let g = ring();
    assert_eq!((g.x[0], g.y[0]), (0, 0), "the centre is entry zero");
    assert_eq!(g.radius[0], 1);
    for r in 1..=RING_LAST as usize {
        let circle_ring = c.radius[r] - c.radius[r - 1];
        let ring_ring = g.radius[r] - g.radius[r - 1];
        assert!(
            ring_ring >= circle_ring,
            "ring {r}: {ring_ring} entries against the circle's {circle_ring}"
        );
        // Every entry of the circle's ring is at the front of the ring's.
        for k in 0..circle_ring {
            assert_eq!(
                (g.x[g.radius[r - 1] + k], g.y[g.radius[r - 1] + k]),
                (c.x[c.radius[r - 1] + k], c.y[c.radius[r - 1] + k]),
                "ring {r} entry {k} is the circle's"
            );
        }
        // And the patch lands strictly inside ring `r`.
        for k in circle_ring..ring_ring {
            let i = g.radius[r - 1] + k;
            assert!(
                vector_dist(g.x[i], g.y[i]) < r as i32,
                "ring {r}'s patch at {:?} is not inside it",
                (g.x[i], g.y[i])
            );
        }
    }
}

/// The thickening is not decorative: ring 1 is the eight neighbours and
/// gains nothing, and the first ring that gains is the first with a
/// diagonal gap.
#[test]
fn ring_one_is_the_eight_neighbours_and_the_thickening_starts_later() {
    let c = circle();
    let g = ring();
    assert_eq!(g.radius[1] - g.radius[0], 8, "ring 1 is the neighbourhood");
    assert_eq!(c.radius[1] - c.radius[0], 8);
    let grown: Vec<usize> = (1..=RING_LAST as usize)
        .filter(|&r| (g.radius[r] - g.radius[r - 1]) > (c.radius[r] - c.radius[r - 1]))
        .collect();
    assert!(
        !grown.is_empty(),
        "no ring is thickened, so `ring_init`'s patch never fires"
    );
}

/// `AnimalData::is_seen@005d8160` (NATIONS §5.1): an animal is seen
/// through the **ever-seen** plane, not the current line of sight; a
/// herdless one and a Spanish viewer's read `was_seen`, which
/// `reveal_map == 2` opens and `was_really_seen` does not.
///
/// **Made to fail on purpose**: through `UnitData::is_seen`'s current
/// fog, the first row answers false.
#[test]
fn an_animal_is_seen_through_the_ever_seen_plane() {
    let (mut s, _) = fog_sim(4, 0);
    let at = Pos::new(10 * 0x300 + 0x180, 10 * 0x300 + 0x180);
    let mut a = crate::Unit::new(8, 0, at, 20);
    a.herd = Some(0);
    let a = s.add_unit(a);
    let (fx, fy) = (at.x / UNITS_PER_FOG, at.y / UNITS_PER_FOG);
    let t = crate::combat::Obj::Unit(a);
    assert!(!s.target_seen_by(1, t), "never seen");
    s.world.set_seen2_only(fx, fy, 0b10);
    assert!(s.target_seen_by(1, t), "ever seen, no line of sight now");
    s.world.clear_seen();
    s.world.set_seen2_only(fx, fy, 0);
    s.world.set_seen(fx, fy, 0);
    // a fresh plane for the reveal rows
    let (mut s, _) = fog_sim(4, 0);
    let mut a = crate::Unit::new(8, 0, at, 20);
    a.herd = Some(0);
    let a = s.add_unit(a);
    let t = crate::combat::Obj::Unit(a);
    s.lobby.reveal_map = 2;
    assert!(
        !s.target_seen_by(1, t),
        "a herd animal reads was_really_seen"
    );
    s.units[a].herd = None;
    assert!(s.target_seen_by(1, t), "a herdless one reads was_seen");
    s.units[a].herd = Some(0);
    s.tables.setup.starting_town = true;
    s.tech[1].has_city = true;
    s.tech[1].power = Some(9);
    assert!(s.target_seen_by(1, t), "a Spanish viewer reads was_seen");
}
