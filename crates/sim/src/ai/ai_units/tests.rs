use super::*;
use crate::ai::Leader;
use crate::combat::Rng;

#[test]
fn the_war_multiplier_stacks_the_region_and_the_strategy() {
    // No war anywhere: the base, unchanged.
    assert_eq!(war_multiplier(100, 0, 0, 0, 0, 0, 0, false).0, 100);
    // A war somewhere but not here: 11/10.
    assert_eq!(war_multiplier(100, 0, 0, 0, 1, 0, 0, false).0, 110);
    // A war in this region, and one active: double, and the base itself
    // doubles with it.
    let (v, base) = war_multiplier(100, 1, 0, 1, 1, 0, 0, false);
    assert_eq!((v, base), (200, 200));
    // Only one of the three: three halves.
    assert_eq!(war_multiplier(100, 0, 1, 0, 0, 0, 0, false).0, 150);
    // The strategy word: bit 2 is three halves, bit 4 (or an attacked
    // region) is double, and a city-trained type in an attacked city is
    // quadruple.
    assert_eq!(war_multiplier(100, 0, 0, 0, 0, 2, 0, false).0, 150);
    assert_eq!(war_multiplier(100, 0, 0, 0, 0, 4, 0, false).0, 200);
    assert_eq!(war_multiplier(100, 0, 0, 0, 0, 0, 1, true).0, 400);
}

#[test]
fn the_two_army_ladders_are_not_the_same() {
    // A tiny army: land opens ×6 then ×2, air ×2 then ×3/2.
    assert_eq!(army_ladder(100, 0, 100, true, false), 1200);
    assert_eq!(army_ladder(100, 0, 100, false, false), 300);
    // An army at four fifths of target clears the first rung only.
    assert_eq!(army_ladder(100, 60, 100, true, false), 200);
    assert_eq!(army_ladder(100, 60, 100, false, false), 150);
    // A big army: the last three rungs stack.
    assert_eq!(army_ladder(900, 300, 100, true, false), 900 * 2 / 3 / 2 / 2);
    // The region rung halves again.
    assert_eq!(army_ladder(100, 100, 100, true, true), 50);
}

#[test]
fn the_batch_leaves_a_gap_below_the_population_cap() {
    // Ten population's worth of a one-pop unit, clipped by the cap left.
    assert_eq!(batch_size(1, 5, 0, 100), 5);
    assert_eq!(batch_size(1, 20, 0, 100), 10);
    assert_eq!(batch_size(2, 20, 0, 100), 5);
    // A control cost above ten is one at a time.
    assert_eq!(batch_size(20, 20, 0, 100), 1);
    // Near the cap the batch walks down to leave one free.
    assert_eq!(batch_size(1, 20, 95, 100), 3);
    assert_eq!(batch_size(1, 20, 99, 100), 1);
    assert_eq!(batch_size(0, 5, 0, 100), 1);
}

/// A sim with one player on a small land world, its AI leader armed.
fn bare() -> Sim {
    let mut w = crate::world::World::new(16, 16);
    w.fill_region(
        Terrain::Land,
        crate::world::Cell::new(0, 0),
        crate::world::Cell::new(15, 15),
    );
    let mut sim = Sim::new(crate::Tuning::RON, w, 1);
    sim.ai_state.ai = vec![Leader::new()];
    sim.ai_state.ai[0]
        .census
        .resize(sim.world.region_count(), 1);
    sim.ai_state.ai[0]
        .city_ai
        .resize(4, crate::ai::CityAi::default());
    sim
}

#[test]
fn queued_units_sums_the_control_cost_of_what_is_ordered() {
    let mut sim = bare();
    let mut ty = crate::UnitType::default();
    ty.price.pop = 2;
    let a = sim.add_unit_type(ty);
    let mut ty = crate::UnitType::default();
    ty.price.pop = 3;
    let b = sim.add_unit_type(ty);
    let at = sim.add_building(0, crate::Pos::default(), 8);
    assert_eq!(sim.queued_units(0), 0);
    sim.queue_up(at, a).expect("queued");
    sim.queue_up(at, b).expect("queued");
    sim.queue_up(at, a).expect("queued");
    // Two of a two-pop type and one of a three-pop type.
    assert_eq!(sim.queued_units(0), 7);
    // A dead building's queue does not count.
    sim.buildings[at].alive = false;
    assert_eq!(sim.queued_units(0), 0);
}

#[test]
fn check_income_is_a_factor_in_two_hundred_and_fifty_sixths() {
    let mut sim = bare();
    let mut ty = crate::UnitType::default();
    ty.price.base[0] = 100;
    let a = sim.add_unit_type(ty);
    let t = {
        let mut tree = tech::TechTree::new();
        let good = tree.add(tech::TypeDef::good("Food"));
        let u = tree.add(tech::TypeDef::unit("A", tech::UnitTraits::default()));
        let _ = good;
        tree.finalize();
        sim.set_tech_tree(tree);
        u
    };
    sim.tables.unit_types[a].tree = Some(t);
    // Owned, so the price is the train arm's: an unowned type is priced
    // by its research (`docs/AI.md` §56).
    sim.tech[0].tech[t] = true;
    sim.economy.holdings[0].available = [true; RESOURCES];
    sim.economy.ledgers[0].bucket[0] = 1000;
    // No income at all: the factor is quartered once for the one good.
    sim.economy.ledgers[0].income[0] = 0;
    assert_eq!(sim.check_income(0, t, 0x400, None, false, -1, 1, 0), 0x40);
    // With an income the discount is `4q` percent of the value, `q` the
    // price in sixteenths-of-income units, capped at twenty.
    sim.economy.ledgers[0].income[0] = 10 * 16;
    let price = sim.type_price(0, t).expect("a unit has a price")[0];
    let q = (price / 10).clamp(0, 20);
    assert_eq!(
        sim.check_income(0, t, 0x400, None, false, -1, 1, 0),
        (100 - 4 * q) * 0x100 / 100
    );
    // A tenth of the price as income puts `q` under the cap.
    sim.economy.ledgers[0].income[0] = price * 16;
    assert_eq!(
        sim.check_income(0, t, 0x400, None, false, -1, 1, 0),
        (100 - 4) * 0x100 / 100
    );
    // Escrow skips the discount entirely.
    assert_eq!(sim.check_income(0, t, 0x400, None, true, -1, 1, 0), 0x100);
    // A batch it cannot pay for is worth nothing without escrow, and a
    // quarter with it.
    assert_eq!(sim.check_income(0, t, 0x400, None, false, -1, 50, 0), 0);
    assert_eq!(sim.check_income(0, t, 0x400, None, true, -1, 50, 0), 0x40);
}

/// How many draws a closure takes out of the sync stream.
fn draws(sim: &mut Sim, before: Rng) -> usize {
    let mut probe = before;
    let mut n = 0;
    while probe != sim.rng {
        probe.roll();
        n += 1;
        assert!(n < 10_000, "diverged");
    }
    n
}

/// **Great Lakes 9380's Scholar offer is a positive wrap** — item
/// 422, `docs/AI.md` §49.
///
/// `create_units` re-offers the Scholar on this frame and the number
/// it files is **45,568**, which ranks below a Citizen's 234,782 and
/// loses the purchase three frames later — the purchase that costs
/// Great Lakes its sixth scholar at 9510 (§47, §48).
///
/// 45,568 is not a valuation. It is the remainder of a 32-bit
/// overflow, and the five terms are measured off the run rather than
/// fitted:
///
/// | term | value |
/// | --- | --- |
/// | `val` | 42,000,000 |
/// | `fac` | 256 |
/// | `want` | 20 |
/// | `divisor` | 25 (`want` + 0 queued + 5 standing) |
/// | `out` | **45,568** |
///
/// `20 × 42,000,000 / 25` is 33,600,000; `256 ×` that is
/// **8,601,600,000**, which does not fit an `i32`; it wraps to
/// **11,665,408**, and `/256` is 45,568 exactly.
///
/// **And the sign is the sting.** `docs/AI.md` §45's `out < 0 →
/// 9,999,999` catches a wrap that lands negative — the ceiling that
/// section is named for. This one lands **positive**, so the guard
/// never fires and the overflow leaves behind a small, plausible
/// number instead of an obvious one. The same defect as §45, in the
/// arm next door, wearing the one disguise the guard cannot see.
///
/// The original's own offer on that frame is **5,755,741**, and
/// `5,755,741 × 256` is 1,473,469,696 — inside an `i32` with room, so
/// it does not wrap. ~~Which of its three terms differs from this
/// crate's is **not** established (§49).~~ **Closed by §53**: the term
/// is `val`, and it was wrong twice over — `k` was 7 where the
/// original's is 6 and 3 (§51's `count_gather_slots`), and the arm's
/// third multiply was read as an `else if` where the listing has an
/// independent `if`. With both corrected the arm hands the tail
/// 180,000,000 and 90,000,000, and the tail returns the original's own
/// two numbers exactly: [`the_scholar_arm_pays_great_lakes_9380_s_two_offers`].
///
/// **`VAL = 42,000,000` below is therefore historical**, the value the
/// arm produced before item 442. It is kept because the *wrap* it
/// demonstrates is unchanged and load-bearing — the corrected chain
/// wraps too, twice, and lands on the right answer.
#[test]
fn the_scholar_offer_on_great_lakes_9380_is_a_positive_wrap() {
    const VAL: i32 = 42_000_000;
    const FAC: i32 = 256;
    const WANT: i32 = 20;
    const DIVISOR: i32 = 25;
    // The offer as filed, and the arithmetic that produces it.
    assert_eq!(offer_value(FAC, WANT, VAL, DIVISOR), 45_568);
    let x = WANT * VAL / DIVISOR;
    assert_eq!(x, 33_600_000);
    // The product that does not fit, stated as the 64-bit number it
    // would be — this is the line that says "overflow" rather than
    // "small value".
    assert_eq!(i64::from(FAC) * i64::from(x), 8_601_600_000);
    assert!(i64::from(FAC) * i64::from(x) > i64::from(i32::MAX));
    // Where it lands, and that it lands **positive** — so the
    // original's own guard does not fire.
    assert_eq!(wm(FAC, x), 11_665_408);
    assert!(
        wm(FAC, x) > 0,
        "a negative wrap would be caught by the guard"
    );
    assert_eq!(wm(FAC, x) / 256, 45_568);
    // **The guard still works where the wrap is negative**, which is
    // what says this test is about the sign and not about the
    // expression being broken in general: §45's own row.
    assert_eq!(offer_value(FAC, 1, i32::MAX, 1), 9_999_999);
    // **The Citizen it has to beat, and does not.** Both sides offer
    // the Citizen at this value on the same block (run111), so the
    // ranking is what the wrap decides — asserted through
    // `offer_value` rather than between literals, which would be a
    // tautology that says nothing.
    const CITIZEN: i32 = 234_782;
    const THEIRS: i32 = 5_755_741;
    assert!(
        offer_value(FAC, WANT, VAL, DIVISOR) < CITIZEN,
        "the wrapped offer does not lose to the Citizen — then the \
             purchase at 9382 is decided by something else"
    );
    // **And the original's number does not wrap**, said through `wm`
    // rather than between constants: 5,755,741 × 256 survives the
    // same multiply that eats this crate's.
    assert_eq!(wm(FAC, THEIRS), 1_473_469_696);
    assert!(
        wm(FAC, THEIRS) > 0 && i64::from(wm(FAC, THEIRS)) == i64::from(FAC) * i64::from(THEIRS)
    );
}

/// **The scholar arm pays Great Lakes 9380's two offers, to the
/// unit** — item 442, `docs/AI.md` §53. This is the item's whole
/// claim as one assertion, and it is not a fit: both numbers were
/// read off run114's `RON_LEADER_PROBE` trace (§52.2) before the
/// arithmetic was done, and both fall out of the same two corrections.
///
/// | | city 0 | city 1 |
/// | --- | --- | --- |
/// | seated scholars (run111's `inside_down`) | 4 | 1 |
/// | `k = gfree[KNOWLEDGE] − count_queue` | **3** | **6** |
/// | `scholar_value` | 90,000,000 | 180,000,000 |
/// | `offer_value` | **4,891,136** | **5,755,741** |
/// | the original's own offer (run114) | **4,891,136** | **5,755,741** |
///
/// **Two corrections, and neither alone reaches either number.**
/// `count_gather_slots` through [`crate::Sim::num_gatherers`] turns
/// `k = 7` into 3 and 6 (§51); the third multiply as an independent
/// `if` turns ×600 into ×3000 (§53.1). With only the first, the chain
/// hands the tail 18,000,000 and 36,000,000, and **both clamp to
/// 9,999,999** — which is the 925-frame loss item 438 measured and
/// declined to land. With only the second, `k` is 7 for both cities
/// and the two offers are equal, which is the tie §50 found buying the
/// Scholar twice. The rows below assert each of those three worlds, so
/// the test says *why* the pair is a pair.
///
/// **Both corrected values still wrap.** 6's chain wraps negative at
/// `want × val` and back positive at `× fac`; 3's wraps once. The
/// ratio 5,755,741 / 4,891,136 = 1.1768 that §51.2 recorded as
/// evidence against `val ∝ k` was an artefact of exactly that — the
/// inputs *are* 2:1, and §49.2's lesson held: a number downstream of a
/// wrap has no arithmetic relationship to its inputs.
#[test]
fn the_scholar_arm_pays_great_lakes_9380_s_two_offers() {
    // The frame's shared terms, measured off the run (§49.1) and
    // unchanged by this item: `infra_mod` 256, and the tail's
    // `fac`/`want`/`divisor`.
    const INFRA: i32 = 256;
    const FAC: i32 = 256;
    const WANT: i32 = 20;
    const DIVISOR: i32 = 25;
    // The arm's two branch tests are `Leader` fields, so they are
    // leader-global: both cities take all three multiplies on this
    // frame. `filled < total * 2 / 3` is the strictest of them.
    const FILLED: i32 = 0;
    const TOTAL: i32 = 14;
    const { assert!(FILLED < TOTAL * 2 / 3 && FILLED < TOTAL) };
    // City 0: four scholars seated in its University, so `k` is 3.
    assert_eq!(scholar_value(INFRA, 3, FILLED, TOTAL), 90_000_000);
    assert_eq!(
        offer_value(FAC, WANT, scholar_value(INFRA, 3, FILLED, TOTAL), DIVISOR),
        4_891_136,
        "the original's city-0 Scholar, off run114's own trace"
    );
    // City 1: one seated, so `k` is 6.
    assert_eq!(scholar_value(INFRA, 6, FILLED, TOTAL), 180_000_000);
    assert_eq!(
        offer_value(FAC, WANT, scholar_value(INFRA, 6, FILLED, TOTAL), DIVISOR),
        5_755_741,
        "the original's city-1 Scholar, off run114's own trace"
    );
    // **The wrap is still there, and it is why the ratio was a red
    // herring.** The inputs are exactly 2:1; the outputs are 1.1768:1.
    assert_eq!(
        scholar_value(INFRA, 6, FILLED, TOTAL),
        2 * scholar_value(INFRA, 3, FILLED, TOTAL)
    );
    assert!(i64::from(WANT) * 180_000_000_i64 > i64::from(i32::MAX));
    // **Half a fix reaches neither number.** `k = 7` for both cities
    // is the tree before §51's correction; with the third multiply in
    // place it clamps, and without it the two cities tie.
    assert_eq!(
        offer_value(FAC, WANT, scholar_value(INFRA, 7, FILLED, TOTAL), DIVISOR),
        9_999_999,
        "the uncorrected `k`, with the arm right — §45's ceiling"
    );
    // The `else if` reading, which is `scholar_value` without its
    // third multiply: both corrected `k`s clamp, which is item 438's
    // 36,000,000 / 18,000,000 and its 925 lost frames.
    let without_third = |k: i32| wm(wm(wm(wm(INFRA, k), 10000) / 256, 60), 10);
    assert_eq!(without_third(6), 36_000_000);
    assert_eq!(without_third(3), 18_000_000);
    for k in [3, 6] {
        assert_eq!(
            offer_value(FAC, WANT, without_third(k), DIVISOR),
            9_999_999,
            "the `else if` reading clamps for both cities — the 925 frames"
        );
    }
    // And with neither correction, the tie §50 measured: one number
    // for two cities, below the Citizen's 234,782.
    assert_eq!(offer_value(FAC, WANT, without_third(7), DIVISOR), 45_568);
}

/// **The air arm sets the caller's `remaining` to the air cap**
/// (`create_units` 361–478: `remaining = air_cap` is the loop's own
/// variable, item 1583, `docs/AI.md` §158) — 2, 4 or 16 by difficulty,
/// not the step-5 default of 5 the Great Sahara coverage lobby's Bomber
/// was priced with. Made to fail with the assignment dropped.
#[test]
fn the_air_arm_sets_remaining_to_the_air_cap() {
    let mut sim = bare();
    let mut tree = tech::TechTree::new();
    let airbase = tree.add(tech::TypeDef::building("Airbase"));
    let unit = tree.add(tech::TypeDef::unit(
        "Fighter",
        tech::UnitTraits {
            combat: true,
            ..tech::UnitTraits::default()
        },
    ));
    tree.finalize();
    sim.set_tech_tree(tree);
    let mut ty = crate::UnitType {
        tree: Some(unit),
        ..crate::UnitType::default()
    };
    ty.price.pop = 1;
    ty.combat.combat_role = true;
    ty.combat.attack = 100;
    ty.combat.domain = crate::attrition::Domain::Air;
    ty.group = Some(0);
    sim.add_unit_type(ty);
    let rec = sim.tables.build_types.len();
    sim.tables.build_types.push(crate::build::BuildType {
        ident: Ident::Airbase,
        tree: Some(airbase),
        ..crate::build::BuildType::default()
    });
    let b = sim.add_building(0, crate::Pos::default(), 8);
    sim.buildings[b].ty = Some(rec);
    let f = sim.unit_facts(unit).expect("a unit record");
    for (difficulty, cap) in [(0, 2), (2, 4), (3, 16)] {
        sim.ai_state.lobby.difficulty = difficulty;
        let (mut remaining, mut escrow, mut num) = (5, 0, 0);
        sim.air_value(
            0,
            unit,
            &f,
            100,
            1,
            100,
            &mut remaining,
            &mut escrow,
            &mut num,
            100,
        );
        assert_eq!(remaining, cap, "difficulty {difficulty}");
    }
}

/// **`get_nukes` counts the Nuclear Missile line, not the type offered**
/// (`LeaderData::get_nukes@006ebe50`, item 1583, `docs/AI.md` §158): a
/// queued ICBM is not a nuke here, and a leader without the tech has
/// none. Made to fail with the line counted from the ICBM.
#[test]
fn the_nuke_count_is_the_nuclear_missile_line() {
    let mut sim = bare();
    let mut tree = tech::TechTree::new();
    let nuke = tree.add(tech::TypeDef::unit(
        "Nuclear Missile",
        tech::UnitTraits::default(),
    ));
    let icbm = tree.add(tech::TypeDef::unit("ICBM", tech::UnitTraits::default()));
    tree.roles.nuclearmissile = Some(nuke);
    tree.roles.icbm = Some(icbm);
    tree.finalize();
    sim.set_tech_tree(tree);
    let rec = |sim: &mut Sim, t: TypeId| {
        sim.add_unit_type(crate::UnitType {
            tree: Some(t),
            ..crate::UnitType::default()
        })
    };
    let (n, i) = (rec(&mut sim, nuke), rec(&mut sim, icbm));
    sim.roster.muster[0].by_type[n] = 1;
    sim.roster.muster[0].queued_by_type[n] = 2;
    sim.roster.muster[0].by_type[i] = 5;
    sim.roster.muster[0].queued_by_type[i] = 7;
    assert_eq!(sim.get_nukes(0), 0, "without the tech");
    sim.tech[0].tech[nuke] = true;
    assert_eq!(sim.get_nukes(0), 3, "held and queued Nuclear Missiles");
    // The census the nuke arm reads: mine against the most of the others.
    sim.territory.players = vec![Default::default(); 2];
    sim.leaders.defeated = vec![false; 2];
    sim.tech.push(sim.tech[0].clone());
    sim.roster.muster.push(sim.roster.muster[0].clone());
    sim.roster.muster[1].by_type[n] = 4;
    sim.roster.muster[1].queued_by_type[n] = 0;
    sim.tech[1].tech[nuke] = true;
    let (mine, most, vulnerable) = sim.nuke_standing(0);
    assert_eq!((mine, most, vulnerable), (3, 4, true));
}

#[test]
fn create_units_offers_nothing_without_a_tree() {
    let mut sim = bare();
    let before = sim.rng;
    sim.create_units(0);
    assert_eq!(draws(&mut sim, before), 0);
    assert_eq!(sim.ai_state.ai[0].make_list.head().t, -1);
}

/// The ids a [`barracks_sim`] hands back: the unit's tree entry, its
/// record, the barracks' record, and the barracks building.
struct Barracks {
    unit: TypeId,
    rec: usize,
    brec: usize,
    b: usize,
}

/// `find_building(city, SEARCH_FRIENDLY, who, 0x1800, …, DOCK)` — the
/// sea branch's dock is the nearest of mine around the city, whichever
/// city it belongs to, inside eight cells (`docs/AI.md` §57).
#[test]
fn the_sea_branch_s_dock_is_the_nearest_of_mine_around_the_city() {
    use crate::world::{Pos, UNITS_PER_CELL, UNITS_PER_TILE};
    let mut sim = bare();
    let dock = sim.tables.build_types.len();
    sim.tables.build_types.push(crate::build::BuildType {
        ident: Ident::Dock,
        ..crate::build::BuildType::default()
    });
    let add = |sim: &mut Sim, who: Player, tiles: (i32, i32)| {
        let b = sim.add_building(
            who,
            Pos::new(tiles.0 * UNITS_PER_TILE, tiles.1 * UNITS_PER_TILE),
            8,
        );
        sim.buildings[b].ty = Some(dock);
        b
    };
    let city = Pos::new(40 * UNITS_PER_TILE, 40 * UNITS_PER_TILE);
    assert_eq!(sim.find_dock_near(0, city), None, "no dock at all");
    // A dock of no city, twelve tiles out: found, although no city's
    // chain holds it — East Indies' `1/2010`.
    let far = add(&mut sim, 0, (52, 40));
    assert_eq!(sim.find_dock_near(0, city), Some(far));
    // A nearer one of mine wins; a nearer one of another leader's does
    // not; an unfinished one is not a candidate.
    let rival = add(&mut sim, 1, (42, 40));
    let near = add(&mut sim, 0, (46, 40));
    let site = add(&mut sim, 0, (44, 40));
    sim.buildings[site].active = false;
    assert_eq!(sim.find_dock_near(0, city), Some(near));
    let _ = rival;
    // Nine cells out is past `circle_radius[8]`: not found, however
    // near it would be in tiles to nothing else.
    for b in [far, near] {
        sim.buildings[b].alive = false;
    }
    let beyond = add(&mut sim, 0, (40 + 9 * UNITS_PER_CELL / UNITS_PER_TILE, 40));
    assert_eq!(sim.find_dock_near(0, city), None);
    sim.buildings[beyond].pos.x -= UNITS_PER_CELL;
    assert_eq!(
        sim.find_dock_near(0, city),
        Some(beyond),
        "eight cells is in"
    );
}

/// One leader with one city whose building is a Barracks — a military
/// trainer — and one available land military type made there. Enough for
/// a type to walk the whole of `create_units` and reach `make_me`.
fn barracks_sim(upgradeable: bool) -> (Sim, Barracks) {
    let mut sim = bare();
    // The tree: six goods, one unit, one building.
    let mut tree = tech::TechTree::new();
    for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        tree.add(tech::TypeDef::good(n));
    }
    let unit = tree.add(tech::TypeDef::unit(
        "Hoplites",
        tech::UnitTraits {
            combat: true,
            ..tech::UnitTraits::default()
        },
    ));
    let build = tree.add(tech::TypeDef::building("Barracks"));
    tree.types[unit].where_ = Some(build);
    tree.finalize();
    sim.set_tech_tree(tree);
    sim.tech[0].tech[unit] = !upgradeable;
    sim.tech[0].tech[build] = true;

    // The unit type: land, military, one population.
    let mut ty = crate::UnitType {
        tree: Some(unit),
        ..crate::UnitType::default()
    };
    ty.price.pop = 1;
    ty.combat.combat_role = true;
    ty.combat.attack = 100;
    ty.combat.domain = crate::attrition::Domain::Land;
    // Every shipped fighting unit belongs to a production group; the
    // muster's group counters are sized from this at registration.
    ty.group = Some(0);
    let rec = sim.add_unit_type(ty);

    // The building type: a military trainer (`BUILD_FLAGS 5`).
    let brec = sim.tables.build_types.len();
    sim.tables.build_types.push(crate::build::BuildType {
        ident: Ident::Barracks,
        tree: Some(build),
        flags: MILITARY_TRAINER,
        x_size: 2,
        y_size: 2,
        hits: 100,
        ..crate::build::BuildType::default()
    });

    let b = sim.add_building(0, crate::Pos::new(4 * 256, 4 * 256), 8);
    sim.buildings[b].ty = Some(brec);
    sim.cities_state.cities.push(crate::city::City {
        alive: true,
        owner: 0,
        race: Some(0),
        founder: 0,
        building: b,
        members: Vec::new(),
        reg: sim.world.region_of(sim.buildings[b].pos.cell()),
        pos: sim.buildings[b].pos,
        capital: true,
        founding_capital: true,
        was_founding_capital: false,
        unassimilated: false,
        no_heal: false,
        attacking: false,
        ever_attacked: false,
        alarm: false,
        no_muster: false,
        was_capital: 0,
        capture_stamp: 0,
        assimilation_timer: 0,
        attack_stamp: 0,
        reduce_stamp: 0,
        raid_stamp: 0,
        capture_strength: 0,
        pop: 1,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    });
    sim.buildings[b].city = Some(sim.cities_state.cities.len() - 1);
    sim.ai_state.ai[0]
        .city_ai
        .resize(sim.cities_state.cities.len(), crate::ai::CityAi::default());
    sim.ai_state.ai[0].mil_trainers = vec![b];
    sim.roster.muster[0].cap = 100;
    sim.ai_state.ai[0].census.pop = 10;
    for l in &mut sim.economy.ledgers {
        l.bucket = [10_000; RESOURCES];
    }
    sim.economy.holdings[0].available = [true; RESOURCES];
    (sim, Barracks { unit, rec, brec, b })
}

/// The registry `produce_unit` and `produce_tech` both read, and which
/// nothing in this crate wrote until 2026-09-04 — `docs/AI.md` §29.
#[test]
fn a_trainer_joins_the_leader_s_list_on_activation_and_leaves_on_close() {
    let (mut sim, ids) = barracks_sim(false);
    let city = sim.buildings[ids.b].city;
    // The fixture files its own barracks by hand; this is about the
    // registry the building lifecycle keeps.
    sim.ai_state.ai[0].mil_trainers.clear();
    sim.buildings[ids.b].active = false;

    // A second trainer, to make the order and the removal observable.
    let trainer = |sim: &mut Sim, x: i32| {
        let b = sim.add_building(0, crate::Pos::new(x * 256, 4 * 256), 8);
        sim.buildings[b].ty = Some(ids.brec);
        sim.buildings[b].city = city;
        b
    };
    let b2 = trainer(&mut sim, 8);
    let b3 = trainer(&mut sim, 12);
    let b4 = trainer(&mut sim, 12);

    // `Wall::increment_stats` files an *active* trainer, in activation
    // order, and `SimpleArray::add` never checks for a duplicate — but
    // `Wall::activate` only reaches it on the transition.
    sim.activate(ids.b, false, true);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b]);
    sim.activate(b2, false, true);
    sim.activate(b3, false, true);
    sim.activate(b4, false, true);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b2, b3, b4]);

    // `SimpleArray<int>::remove@00462e70` finds the first slot holding
    // the value and shifts the tail down, so closing one from the middle
    // leaves the order of the rest — it is not a swap with the last.
    sim.close_building(b2, false);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b3, b4]);
    sim.close_building(b4, false);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b3]);

    // Not a military trainer: never filed. `bare`'s type table has one
    // record with no flags at all.
    let plain = sim.add_build_type(crate::build::BuildType {
        ident: Ident::Granary,
        x_size: 2,
        y_size: 2,
        hits: 100,
        ..crate::build::BuildType::default()
    });
    let g = sim.add_building(0, crate::Pos::new(4 * 256, 12 * 256), 8);
    sim.buildings[g].ty = Some(plain);
    sim.buildings[g].city = city;
    sim.activate(g, false, true);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b3]);

    // A trainer outside a city, with no `NO_CITY` on its type, is what
    // `Wall::activate`'s own guard refuses.
    let out = trainer(&mut sim, 4);
    sim.buildings[out].city = None;
    sim.activate(out, false, true);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b3]);
    // With `NO_CITY` the same building would have joined.
    sim.tables.build_types[ids.brec].flags |= crate::build::flags::NO_CITY;
    let free = trainer(&mut sim, 6);
    sim.buildings[free].city = None;
    sim.activate(free, false, true);
    assert_eq!(sim.ai_state.ai[0].mil_trainers, vec![ids.b, b3, free]);
}

#[test]
fn the_army_size_gate_draws_once_per_military_type_and_never_on_moderate() {
    for d in [0, 1, 2, 3, 4] {
        let (mut sim, ids) = barracks_sim(false);
        sim.ai_state.lobby.difficulty = d;
        let before = sim.rng;
        sim.create_units(0);
        let n = draws(&mut sim, before);
        assert_eq!(n, usize::from(d != 2), "difficulty {d}");
        // And the type reached the make list, in the military category.
        assert_eq!(sim.ai_state.ai[0].make_list.head().t, ids.unit as i32);
        assert_eq!(sim.ai_state.ai[0].make_list.list[6].t, ids.unit as i32);
        assert_eq!(sim.ai_state.ai[0].make_list.head().city, 0);
    }
}

#[test]
fn upgrade_units_draws_once_per_eligible_type_and_never_on_moderate() {
    for d in [0, 1, 2, 3, 4] {
        // `type_avail == 2`: eligible, bit clear — an upgrade is bought
        // as research.
        let (mut sim, ids) = barracks_sim(true);
        assert_eq!(sim.type_avail(0, ids.unit), tech::RESEARCHABLE);
        sim.ai_state.lobby.difficulty = d;
        let before = sim.rng;
        sim.upgrade_units(0);
        assert_eq!(
            draws(&mut sim, before),
            usize::from(d != 2),
            "difficulty {d}"
        );
        // A military upgrade lands in category 7.
        assert_eq!(sim.ai_state.ai[0].make_list.list[7].t, ids.unit as i32);
        assert_eq!(sim.ai_state.ai[0].make_list.list[7].escrow, 1);
    }
    // A type that is already available is not an upgrade at all.
    let (mut sim, _) = barracks_sim(false);
    let before = sim.rng;
    sim.upgrade_units(0);
    assert_eq!(draws(&mut sim, before), 0);
    assert_eq!(sim.ai_state.ai[0].make_list.list[7].t, -1);
}

/// **An upgrade whose line is already in research is not offered**
/// (item 1379). `upgrade_units` asks `LeaderData::researching(t, -1, 0,
/// 0)` (`6c657e`..`6c6587`), and with `param_4` 0 the unit arm counts a
/// queued entry of any type in `t`'s lineage whose bit is clear
/// (`6db5d9`..`6db62c`): Great Sahara at Toughest's who=1 had the
/// Heavy Horse Archers (`CAVARCHERS`) in research on 10779, and the original drew for
/// four upgrades where this crate drew for the Dragoon too.
#[test]
fn an_upgrade_whose_predecessor_is_in_research_is_not_offered() {
    let (mut sim, ids) = barracks_sim(false);
    // Two rungs above the trained type: Hoplites → Phalanx → Elite.
    let mut tree = sim.tables.tech_tree.clone();
    let mid = tree.add(tech::TypeDef::unit(
        "Phalanx",
        tech::UnitTraits {
            combat: true,
            ..tech::UnitTraits::default()
        },
    ));
    // The top rung is a jump unit, as the Dragoon is: researchable
    // while the rung below it is not available.
    let top = tree.add(tech::TypeDef::unit(
        "Elite Phalanx",
        tech::UnitTraits {
            combat: true,
            jumpable: true,
            ..tech::UnitTraits::default()
        },
    ));
    let build = tree.types[ids.unit].where_;
    tree.types[mid].where_ = build;
    tree.types[mid].from = Some(ids.unit);
    tree.types[top].where_ = build;
    tree.types[top].from = Some(mid);
    tree.finalize();
    sim.set_tech_tree(tree);
    sim.tech[0].tech[ids.unit] = true;
    sim.tech[0].tech[build.expect("the barracks")] = true;
    for t in [mid, top] {
        let mut ty = sim.tables.unit_types[ids.rec].clone();
        ty.tree = Some(t);
        sim.add_unit_type(ty);
    }
    sim.roster.muster[0].by_type[ids.rec] = 1;
    sim.ai_state.lobby.difficulty = 5;
    let offered = |sim: &mut Sim| {
        sim.ai_state.ai[0].make_list.clear();
        let before = sim.rng;
        sim.upgrade_units(0);
        draws(sim, before)
    };
    assert_eq!(sim.type_avail(0, mid), tech::RESEARCHABLE);
    assert_eq!(sim.type_avail(0, top), tech::RESEARCHABLE);
    assert_eq!(offered(&mut sim), 2, "both rungs, nothing in research");
    // The first rung's research laid at the barracks.
    let mid_rec = sim.unit_record(mid).expect("registered");
    sim.queue_up(ids.b, mid_rec).expect("the research is laid");
    assert!(
        !sim.researching(0, top),
        "the tech equality does not see it"
    );
    assert_eq!(offered(&mut sim), 0, "the rung itself and the one above it");
}

/// `Build::queue_up(t, escrow)` (`docs/AI.md` §98): the affordability
/// test counts the whole stockpile (`can_pay_cost(…, 1)`, `0x6214b2`),
/// and the flag goes to `pay_cost`: an escrowed purchase draws the
/// reservation down, floored at zero; any other abandons it for a good
/// it had to reach into. East Indies' Cataphract on 7382 is the
/// receipt: the original's metal escrow 37 is 0 after it, where an
/// unflagged payment of 60 out of 209 would have left it.
#[test]
fn an_escrowed_order_draws_the_reservation_down_and_an_ordinary_one_abandons_it() {
    let (mut sim, ids) = barracks_sim(false);
    sim.tables.unit_types[ids.rec].price.base[0] = 60;
    let charges = sim.price_of(0, ids.rec);
    let g = (0..crate::economy::RESOURCES)
        .find(|&g| charges[g] > 0)
        .expect("the unit costs something");
    let price = charges[g];
    sim.economy.ledgers[0].bucket = [0; crate::economy::RESOURCES];
    sim.economy.ledgers[0].bucket[g] = price + 50;
    sim.economy.ledgers[0].escrow[g] = price + 20;
    // Only the whole bucket covers it, and the order goes through.
    sim.queue_up_with(ids.b, ids.rec, true)
        .expect("the whole bucket pays");
    assert_eq!(sim.economy.ledgers[0].bucket[g], 50);
    assert_eq!(
        sim.economy.ledgers[0].escrow[g], 20,
        "drawn down by the price"
    );
    sim.economy.ledgers[0].bucket[g] = price + 50;
    sim.economy.ledgers[0].escrow[g] = price + 20;
    sim.queue_up_with(ids.b, ids.rec, false)
        .expect("the same test");
    assert_eq!(
        sim.economy.ledgers[0].escrow[g], 0,
        "reached into: abandoned"
    );
    sim.economy.ledgers[0].bucket[g] = price + 50;
    sim.economy.ledgers[0].escrow[g] = 10;
    sim.queue_up_with(ids.b, ids.rec, false).expect("queued");
    assert_eq!(
        sim.economy.ledgers[0].escrow[g], 10,
        "not reached into: kept"
    );
}

#[test]
fn queued_units_ignores_a_type_the_tree_has_taken_away() {
    let (mut sim, ids) = barracks_sim(false);
    sim.queue_up(ids.b, ids.rec).expect("queued");
    assert_eq!(sim.queued_units(0), 1);
    sim.tech[0].tech[ids.unit] = false;
    assert_eq!(sim.queued_units(0), 0, "type_avail below 4 does not count");
}

/// **A batch's second unit pays the first one's ramp step**
/// (2026-09-06).
///
/// `LeaderData::get_support_count@006da110` answers
/// `<group>_queued + <group>_units` for a type whose `PROGRESSION` is
/// by-group — the same `queued + built` it answers by type — and
/// `Leader::produce_unit` queues its `num` units **one at a time**, so
/// the second is priced with the first already in the queue.
///
/// [`Muster::by_group`] carried only the built half, so the group ramp
/// stood still inside a batch. run76's `BUILDQUEUE` at dump-frame 6783
/// is the receipt: the AI's Barracks holds two Longbowmen at
/// `cost 31, 51` and `cost 33, 53`, and this crate charged 31/51
/// twice. Two timber and two wealth, and two hundred frames later they
/// were the difference between affording a University and not
/// (`docs/AI.md` §30).
#[test]
fn a_batch_s_second_unit_pays_the_first_one_s_group_ramp_step() {
    let (mut sim, ids) = barracks_sim(false);
    // Triangular by group, one timber a step — the shape nearly every
    // fighting unit ships with.
    {
        let price = &mut sim.tables.unit_types[ids.rec].price;
        price.progression = crate::cost::Progression::PROGRESSIVE_BY_GROUP;
        price.support = [Some((crate::economy::Resource::Timber, 1)), None];
    }
    sim.economy.ledgers[0].bucket = [10_000; crate::economy::RESOURCES];
    let timber = crate::economy::Resource::Timber.index();

    let first = sim.price_of(0, ids.rec)[timber];
    sim.queue_up(ids.b, ids.rec).expect("the first is queued");
    let second = sim.price_of(0, ids.rec)[timber];
    sim.queue_up(ids.b, ids.rec).expect("the second is queued");
    let third = sim.price_of(0, ids.rec)[timber];
    assert!(
        first < second && second < third,
        "the queue is half the count: {first}, {second}, {third}"
    );
    assert_eq!(
        sim.roster.muster[0].queued_by_group[0], 2,
        "and it is the queue"
    );

    // And it comes back off. `Build::close`'s refund path is one of the
    // six that move `queued_by_type`, and every one of them moves this
    // beside it.
    let mut ledger = crate::economy::Ledger::default();
    sim.buildings[ids.b].queue.unqueue(0, false, &mut ledger);
    sim.roster.muster[0].queued_by_type[ids.rec] -= 1;
    sim.track_queued_for_test(0, ids.rec, -1);
    assert_eq!(sim.roster.muster[0].queued_by_group[0], 1);
    assert_eq!(sim.price_of(0, ids.rec)[timber], second);
}

#[test]
fn produce_unit_queues_at_the_military_trainer_with_the_shallowest_queue() {
    let (mut sim, ids) = barracks_sim(false);
    // A second trainer, three deep; the first is empty.
    let b2 = sim.add_building(0, crate::Pos::new(8 * 256, 8 * 256), 8);
    sim.buildings[b2].ty = Some(ids.brec);
    sim.ai_state.ai[0].mil_trainers = vec![b2, ids.b];
    for _ in 0..3 {
        sim.queue_up(b2, ids.rec).expect("queued");
    }
    assert!(sim.produce_unit(0, ids.unit, None, 1, 0));
    assert_eq!(sim.buildings[ids.b].queue.items.len(), 1, "the empty one");
    assert_eq!(sim.buildings[b2].queue.items.len(), 3);
    // A trainer already four deep is not a candidate at all.
    sim.ai_state.ai[0].mil_trainers = vec![b2];
    sim.queue_up(b2, ids.rec).expect("queued");
    assert!(!sim.produce_unit(0, ids.unit, None, 1, 0));
    assert_eq!(sim.buildings[b2].queue.items.len(), 4);
}

/// **A batch goes in whole, in one call, down the city walk** — item
/// 362. The military trainer's batch has been pinned since the
/// barracks tests above; the *city-chain* walk, which is the one a
/// scholar or a citizen takes, had none — and it is the walk the
/// make list's `num` reaches. Great Lakes 8985 is what this is
/// written for: the original queues **three** scholars there in a
/// single `make_this` — three `make_stuff+0x221` draws and no
/// `+0x63d` on either side — and until item 362 every civilian
/// `make_me` carried a `num` of one, so this crate queued them one
/// production cycle apart. `docs/AI.md` §42.
///
/// Made to fail by handing `produce_unit` a `num` of 1: the queue
/// then holds one entry, which is exactly the shape run97's
/// `BUILDDATA` showed for 1,320 frames.
#[test]
fn produce_unit_queues_the_whole_batch_down_the_city_walk() {
    let (mut sim, ids) = barracks_sim(false);
    // Not a military trainer any more, so `produce_unit` takes the
    // city-chain branch; the building is still the type the unit is
    // made at, and it is the capital's own.
    sim.tables.build_types[ids.brec].flags &= !MILITARY_TRAINER;
    sim.ai_state.ai[0].mil_trainers.clear();
    assert!(sim.produce_unit(0, ids.unit, Some(0), 3, 0));
    assert_eq!(
        sim.buildings[ids.b].queue.items.len(),
        3,
        "the city walk queued {} of a batch of three",
        sim.buildings[ids.b].queue.items.len()
    );
    // A batch of one is still a batch of one: the plumbing the fix
    // added carries the number, it does not invent one.
    assert!(sim.produce_unit(0, ids.unit, Some(0), 1, 0));
    assert_eq!(sim.buildings[ids.b].queue.items.len(), 4);
}
