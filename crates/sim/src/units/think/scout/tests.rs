use super::*;
use crate::Tuning;
use crate::world::{Cell, Terrain, World};

/// How many draws a call spent — the LCG steps once per draw.
fn draws(before: u32, after: u32) -> i32 {
    let mut r = crate::combat::Rng::new(before);
    for n in 0..64 {
        if r.seed == after {
            return n;
        }
        r.roll();
    }
    panic!("more than 64 draws, or the seed never matched");
}

/// A 40 × 40 land world, player 1 a computer with a city at cell
/// (20, 20) and a scout standing in it, player 0 a human with one.
/// `fog` installs an all-unseen grid; without one nothing is seen and
/// [`Sim::was_really_seen`] answers true everywhere, which is the flat
/// world the harness has always had.
fn scout_sim(fog: bool) -> (Sim, usize, usize) {
    let mut world = World::new(40, 40);
    let r = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    if fog {
        assert!(world.set_fog(vec![0; 80 * 80]));
    }
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities_state.nation[0].human = true;
    s.cities_state.nation[1].human = false;
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        ..crate::UnitType::default()
    });
    s.tables.unit_types[t].cols.role = role::ROLE_SCOUT;

    let centre = Pos::new(20 * 768 + 384, 20 * 768 + 384);
    let bldg = s.add_building(1, centre, 4);
    s.cities_state.cities.push(crate::city::City {
        alive: true,
        owner: 1,
        race: Some(1),
        founder: 1,
        building: bldg,
        members: Vec::new(),
        reg: Some(r),
        pos: centre,
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

    let spawn = |s: &mut Sim, who: Player, at: Pos| {
        let mut u = crate::Unit::new(who, s.units.len() as i16, at, 20);
        u.ty = Some(t);
        s.add_unit(u)
    };
    let ai = spawn(&mut s, 1, centre);
    let human = spawn(&mut s, 0, Pos::new(5 * 768 + 384, 5 * 768 + 384));
    (s, ai, human)
}

/// **`was_really_seen`'s leader exits** (`006b54f0`, `docs/AI.md`
/// §233): on an all-unseen grid a leader sees nothing until a Fouché of
/// its own stands (`num_units[0x141]`), and the other leader is not
/// lent the sight. The Space Program arm (`leader_flags & 0x800`) is
/// the same call, [`Sim::sees_every_unit`].
#[test]
fn a_fouche_sees_every_cell_to_the_scout() {
    let (mut s, ai, _) = scout_sim(true);
    let c = Cell::new(30, 30);
    assert!(!s.was_really_seen(c, 1), "the fog alone: unseen");
    s.units[ai].type_index = 0x32 + 0x141;
    assert!(s.was_really_seen(c, 1), "a Fouché standing: every cell");
    assert!(!s.was_really_seen(c, 0), "and only its owner's");
}

/// §4: `circle_init`'s first rings, and the check that the sweep order
/// is right — ring 1 is the eight neighbours and nothing else.
#[test]
fn the_circle_tables_start_with_the_eight_neighbours() {
    let t = circle::table();
    assert_eq!(
        &t.radius[..8],
        &[1, 9, 21, 45, 69, 105, 145, 185],
        "the cumulative counts of docs/SCOUT.md §4"
    );
    let (lo, hi) = t.ring(1);
    let mut ring: Vec<(i8, i8)> = (lo..hi)
        .map(|k| (t.x[k as usize], t.y[k as usize]))
        .collect();
    ring.sort_unstable();
    assert_eq!(
        ring,
        vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1)
        ],
        "ring 1 is the eight neighbours"
    );
    assert_eq!(t.x.len(), 12_873, "circle_points");
    assert_eq!(t.x.len(), t.y.len());
}

/// §6, the ring walk on its own. Nothing is unseen, so no cell is ever
/// accepted, `best_ring` never drops, and the walk runs to its bound:
/// an AI scout around its own leader's city takes rings 1, 3, 5, 7, 9
/// and 11 — six rotations — and the phase draw only from ring 4 up,
/// which is four of them. Ten draws, none of them a cell.
#[test]
fn an_ai_scout_takes_every_other_ring_and_the_phase_draw_from_ring_four() {
    let (mut s, ai, _) = scout_sim(false);
    let before = s.rng.seed;
    assert!(
        !s.think_scout(ai),
        "everything is seen, so nothing is found"
    );
    assert_eq!(
        draws(before, s.rng.seed),
        6 + 4 + 1,
        "six rings, four phases, the §11 stride"
    );
    assert!(s.units[ai].orders.is_empty(), "and no order is issued");
}

/// **`think_scout+0x7c`, the caster's turn** (`docs/AI.md` §80.5): a
/// computer's caster scout is offered `think_spellcaster` before the
/// walk, whose coin is thrown from difficulty 2 up. The same walk costs
/// one draw more at Toughest than at Easiest, and nothing more.
#[test]
fn a_computers_caster_scout_throws_the_special_coin_before_the_walk() {
    let flags = crate::ai_load::uflags2::FLAG2_SCOUT | crate::ai_load::uflags2::CASTER;
    let mut spent = Vec::new();
    for d in [0, 5] {
        let (mut s, ai, _) = scout_sim(false);
        let t = s.units[ai].ty.expect("typed");
        s.tables.unit_types[t].cols.unit_flags2 = flags;
        s.lobby.difficulty = d;
        let before = s.rng.seed;
        assert!(!s.think_scout(ai), "difficulty {d}: nothing is found");
        spent.push(draws(before, s.rng.seed));
    }
    assert_eq!(spent, [11, 12], "the coin, at Toughest alone");
}

/// §7 and §6's early exit together. With every cell unseen the eight
/// cells of ring 1 are all accepted — one draw each — which drops
/// `best_ring` to 1, and ring 3 then breaks at its first cell, after
/// its rotation draw. Two rotations and eight cells; the phase draw
/// fires at neither ring, both being under 4.
#[test]
fn an_unseen_ring_one_is_taken_whole_and_stops_the_walk_two_rings_later() {
    let (mut s, ai, _) = scout_sim(true);
    let before = s.rng.seed;
    assert!(s.think_scout(ai), "a target is found");
    assert_eq!(draws(before, s.rng.seed), 2 + 8);
    // §9: the one-member group's `EXPLORE_TO`, at the tile centre of a
    // neighbouring cell's `4c + 2`.
    let order = *s.units[ai].orders.front().expect("an order");
    let crate::orders::Body::Move(m) = order.body else {
        panic!("not a move: {order:?}");
    };
    assert_eq!(m.kind, MoveKind::ExploreTo);
    let c = m.dest.cell();
    assert!(
        (c.x - 20).abs() <= 1 && (c.y - 20).abs() <= 1 && c != Cell::new(20, 20),
        "a neighbour of the city, not the city: {c:?}"
    );
}

/// **The tail's cadence** (`docs/ORDERS.md` §2.4 step 5): an idle
/// scout thinks on its **first** idle frame and then only when
/// `(o + frame) % 32 == 0`.
///
/// `Unit::think@005f6e40` carries two gates, not one. The mod-16 gate
/// above the auto-attack decides whether the function does anything at
/// all; the second, between the human block's `unit_masks & 0x40000`
/// exit and `think_fish`, decides the whole tail —
/// `idle != 1 && ((o + frame) & 31) != 0` returns. The crate had only
/// the first, so a scout on its second idle frame — `idle == 2`, which
/// the mod-16 gate lets through — ran a whole second `think_scout`.
/// That was East Indies' word: run56's AI scout arrives on 2664 with
/// `idle == 1`, thinks, and the original then thinks on 2688, 2720,
/// 2752 … exactly, its `o` being 0.
#[test]
fn an_idle_scout_thinks_on_its_first_frame_and_then_once_in_thirty_two() {
    let (mut s, ai, _) = scout_sim(false);
    s.trace_phases = true;
    let o = i64::from(s.units[ai].index);
    // Nothing is unseen, so `think_scout` never issues an order and
    // the scout stays idle for the whole run. `do_idle` is the whole
    // of an idle unit's frame — the anim request, `check_idle`,
    // `think` — so the loop is the tick as this unit sees it.
    let mut thought = Vec::new();
    for frame in 0..96 {
        s.phase_marks.clear();
        s.do_idle(ai, frame);
        let marks = &s.phase_marks;
        let drew = marks.iter().enumerate().any(|(i, (label, from))| {
            label == SITE_ROTATION && marks.get(i + 1).map_or(s.rng.seed, |m| m.1) != *from
        });
        if drew {
            thought.push(frame);
        }
    }
    let want: Vec<i64> = std::iter::once(0)
        .chain((1..96).filter(|f| (f + o) % 32 == 0))
        .collect();
    assert_eq!(
        thought, want,
        "the first idle frame, then `(o + frame) % 32 == 0` and nothing else"
    );
}

/// §2's gate, from both sides: a human's scout never reaches
/// `think_scout` (its `unit_masks & 0x40000` is clear), and neither
/// does a unit without the scout bit.
#[test]
fn the_gate_takes_the_computers_scout_and_nothing_else() {
    let (mut s, ai, human) = scout_sim(true);
    assert!(s.scout_thinks(ai), "the computer's scout");
    assert!(!s.scout_thinks(human), "the human's does not");

    let plain = s.add_unit_type(crate::UnitType {
        hits: 20,
        ..crate::UnitType::default()
    });
    let mut u = crate::Unit::new(1, s.units.len() as i16, s.units[ai].pos, 20);
    u.ty = Some(plain);
    let other = s.add_unit(u);
    assert!(!s.scout_thinks(other), "no scout bit, no explore");
}

/// §2's gate and §3's branch on a **spy** (§15): `is(SPY, 0)` admits
/// the computer's spy to `think_scout` with no scout bit, and routes it
/// past the city loop to the region scan even with its own city in the
/// region — one `+0x941` and one `+0xaba` a cell, where a scout on the
/// same spot spends the city loop's ring draws. run171's Spy on 617.
#[test]
fn a_computers_spy_explores_by_the_region_scan() {
    let (mut s, ai, human) = scout_sim(true);
    let spy_type = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        ..crate::UnitType::default()
    });
    s.tables.unit_types[spy_type].tree = Some(SPY);
    let at = s.units[ai].pos;
    let mut u = crate::Unit::new(1, s.units.len() as i16, at, 20);
    u.ty = Some(spy_type);
    let spy = s.add_unit(u);
    s.units[human].ty = Some(spy_type);
    assert!(s.unit_is_spy(spy) && !s.unit_is_scout(spy));
    assert!(s.scout_thinks(spy), "the computer's spy");
    assert!(!s.scout_thinks(human), "a human's spy does not");

    let region = s.world.region_of(at.cell()).expect("a region");
    let stride = (s.world.region_size(region) + 99) / 100;
    let mut probe = crate::combat::Rng::new(s.rng.seed);
    let start = probe.roll() % stride;
    let walked = s.world.region_coords_strided(region, start, stride);
    let taken = walked
        .iter()
        .filter(|&&c| {
            s.invalid_loc(
                spy,
                Pos::new(c.x * 4 + 2, c.y * 4 + 2),
                false,
                false,
                false,
                false,
                false,
            ) == 0
        })
        .count();
    let before = s.rng.seed;
    assert!(s.think_scout(spy), "an unseen region is all candidates");
    let mut r = crate::combat::Rng::new(before);
    let mut spent = 0;
    while r.seed != s.rng.seed && spent < 256 {
        r.roll();
        spent += 1;
    }
    assert_eq!(spent, 1 + taken, "the region scan, not the city loop");
    let order = *s.units[spy].orders.front().expect("an explore order");
    let crate::orders::Body::Move(m) = order.body else {
        panic!("not a move: {order:?}");
    };
    assert_eq!(m.kind, MoveKind::ExploreTo);
    assert!(walked.contains(&m.dest.cell()), "{:?}", m.dest.cell());
}

/// §5: a leader with no city of its own walks nobody's rings — the
/// `city_num == 0` gate at the head of the city loop. What is left is
/// §11's region scan, whole: a region of 1,600 cells gives a stride of
/// 16, so the walk spends one `+0x941` for its starting offset and one
/// `+0xaba` for every one of the hundred cells it then strides through
/// that `invalid_loc` allows — and its winner is one of those hundred,
/// which is what says the coordinate list is being walked in
/// `Regions::rebuild_coords`' row-major order rather than some other.
#[test]
fn a_leader_with_no_city_scans_its_whole_region() {
    let (mut s, ai, _) = scout_sim(true);
    s.cities_state.cities[0].alive = false;
    let region = s
        .world
        .region_of(s.units[ai].pos.cell())
        .expect("the scout stands in the region");
    let n = s.world.region_size(region);
    assert_eq!(n, 1600, "the fixture's one region is the whole grid");
    // `(n + 99) / 100`, plus frame 0's `frame % 8`.
    let stride = (n + 99) / 100;
    assert_eq!(stride, 16);
    // The offset the scan is about to draw, on a copy of the stream.
    let mut probe = crate::combat::Rng::new(s.rng.seed);
    let start = probe.roll() % stride;
    let walked = s.world.region_coords_strided(region, start, stride);
    assert_eq!(walked.len(), 100, "a hundred of the sixteen hundred");
    assert!(
        walked
            .windows(2)
            .all(|w| (w[0].y, w[0].x) < (w[1].y, w[1].x)),
        "row-major: y then x, strictly increasing"
    );

    let before = s.rng.seed;
    assert!(s.think_scout(ai), "an unseen region is all candidates");
    // One draw for the offset, one for every cell the location test
    // allows — the city's own footprint is what it refuses.
    let taken = walked
        .iter()
        .filter(|&&c| {
            s.invalid_loc(
                ai,
                Pos::new(c.x * 4 + 2, c.y * 4 + 2),
                false,
                false,
                false,
                false,
                false,
            ) == 0
        })
        .count();
    let mut r = crate::combat::Rng::new(before);
    let mut spent = 0;
    while r.seed != s.rng.seed && spent < 256 {
        r.roll();
        spent += 1;
    }
    assert_eq!(spent, 1 + taken, "the offset draw and one a cell");

    let order = *s.units[ai].orders.front().expect("an explore order");
    let crate::orders::Body::Move(m) = order.body else {
        panic!("not a move: {order:?}");
    };
    assert_eq!(m.kind, MoveKind::ExploreTo);
    assert!(
        walked.contains(&m.dest.cell()),
        "the winner is one of the strided cells, not any old one: {:?}",
        m.dest.cell()
    );
}

/// §11.1 — the **second call site**, `Unit::think_peasant`'s AI tail.
/// An AI worker standing in a region where its leader has no city and
/// which none of the ten `Sites` claims gives up and explores; one
/// standing where a site *is* claimed waits, but only while `idle` is
/// six or under. East Indies' word at 4313 was this arm missing.
#[test]
fn an_ai_worker_explores_out_of_a_region_no_city_and_no_site_wants() {
    let citizen = |s: &mut Sim, u: usize| {
        let t = s.units[u].ty.expect("the fixture's type");
        s.tables.unit_types[t].cols.role = 0;
        s.tables.unit_types[t].worker = crate::orders::Worker::Citizen;
        s.units[u].type_index = 0x32;
        // `stance > 1` is what skips the job search, so the tail is
        // what this test reaches (`005f5920`).
        s.units[u].stance = 2;
        s.units[u].idle = 1;
    };
    let region = |s: &Sim, u: usize| {
        s.world
            .tregion_alt(s.units[u].pos.tile())
            .expect("the worker stands in the region")
    };
    let explored = |s: &Sim, u: usize| {
        s.units[u].orders.front().is_some_and(
            |o| matches!(o.body, crate::orders::Body::Move(m) if m.kind == MoveKind::ExploreTo),
        )
    };

    // No city in the region, and no site: it explores.
    let (mut s, ai, _) = scout_sim(true);
    citizen(&mut s, ai);
    let r = region(&s, ai);
    s.ai[1].census.resize(64, 8);
    s.ai[1].sites = [crate::ai::Site::default(); crate::ai::SITE_SLOTS];
    assert!(s.think_peasant(ai, false), "the tail issued the explore");
    assert!(explored(&s, ai), "and it is an EXPLORE_TO");

    // A site claims the region: it waits instead.
    let (mut s, ai, _) = scout_sim(true);
    citizen(&mut s, ai);
    s.ai[1].census.resize(64, 8);
    s.ai[1].sites = [crate::ai::Site::default(); crate::ai::SITE_SLOTS];
    s.ai[1].sites[3] = crate::ai::Site {
        val: 1,
        reg: i32::from(r),
        ..crate::ai::Site::default()
    };
    assert!(
        !s.think_peasant(ai, false),
        "a claimed region is worth waiting in"
    );
    assert!(!explored(&s, ai));

    // …for six idle frames, and no longer.
    s.units[ai].idle = 7;
    assert!(s.think_peasant(ai, false), "seven is one too many");
    assert!(explored(&s, ai));

    // A `val` of zero claims nothing, whatever its `reg` says.
    let (mut s, ai, _) = scout_sim(true);
    citizen(&mut s, ai);
    s.ai[1].census.resize(64, 8);
    s.ai[1].sites = [crate::ai::Site::default(); crate::ai::SITE_SLOTS];
    s.ai[1].sites[3] = crate::ai::Site {
        val: 0,
        reg: i32::from(r),
        ..crate::ai::Site::default()
    };
    assert!(s.think_peasant(ai, false));

    // And the leader's own city in the region ends the question before
    // the sites are read at all.
    let (mut s, ai, _) = scout_sim(true);
    citizen(&mut s, ai);
    s.ai[1].census.resize(64, 8);
    s.ai[1].census.reg_cities[r as usize] = 1;
    s.ai[1].sites = [crate::ai::Site::default(); crate::ai::SITE_SLOTS];
    assert!(!s.think_peasant(ai, false), "a city in the region: stay");
    assert!(!explored(&s, ai));
}

/// A **scholar** skips the whole tail, `unit_masks & 0x400` included —
/// the one worker that keeps "has been a builder" across a failed
/// search (`think_peasant@005f5760:111`).
#[test]
fn a_scholar_keeps_its_builder_bit_where_a_citizen_loses_it() {
    let (mut s, ai, _) = scout_sim(true);
    let t = s.units[ai].ty.expect("the fixture's type");
    s.tables.unit_types[t].cols.role = 0;
    s.tables.unit_types[t].worker = crate::orders::Worker::Scholar;
    s.units[ai].type_index = 0x34;
    s.units[ai].stance = 2;
    s.units[ai].idle = 1;
    s.units[ai].was_builder = true;
    assert!(!s.think_peasant(ai, false));
    assert!(s.units[ai].was_builder, "a scholar never reaches the clear");

    s.units[ai].type_index = 0x32;
    s.tables.unit_types[t].worker = crate::orders::Worker::Citizen;
    let r = s
        .world
        .tregion_alt(s.units[ai].pos.tile())
        .expect("the worker stands in the region");
    s.ai[1].census.resize(64, 8);
    s.ai[1].census.reg_cities[r as usize] = 1;
    assert!(!s.think_peasant(ai, false));
    assert!(!s.units[ai].was_builder, "a citizen does");
}

/// **`find_unit_ordered` is two predicates the crate did not have**
/// (`ObjectsData::find_unit_ordered@0065bc40`, read 2026-09-01): the
/// blocking unit must carry an order, that order must be in the move
/// family, and it must stand in the target cell's own region. The
/// crate rejected a candidate for any sibling of the same type within
/// two cells, standing or not — so a scout parked beside a cell
/// refused it to every other scout for the rest of the game.
#[test]
fn a_sibling_blocks_a_cell_only_while_it_is_ordered_into_that_region() {
    let (mut s, ai, _) = scout_sim(true);
    let here = s.units[ai].pos.cell();
    let target = Cell::new(here.x + 4, here.y);
    let at = Pos::new(target.x * 768 + 384, target.y * 768 + 384);
    // A sibling of the scout's own type, one cell from the candidate —
    // well inside `0x600` — and in the same region.
    let mut sib = crate::Unit::new(1, s.units.len() as i16, Pos::new(at.x - 768, at.y), 20);
    sib.ty = s.units[ai].ty;
    let sib = s.add_unit(sib);
    assert!(
        !s.scout_unit_near(ai, target),
        "a sibling with no order is not `ordered` anywhere"
    );

    s.add_gather_order(sib, 0, crate::orders::QueuePos::New, true);
    assert_eq!(s.order_type(sib), crate::orders::index::GATHER);
    assert!(
        !s.scout_unit_near(ai, target),
        "a gather is not in the move family"
    );

    s.units[sib].orders.clear();
    s.add_move_facing_order(
        sib,
        at,
        MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
        crate::movement::Angle(0),
        None,
        false,
    );
    assert!(
        s.scout_unit_near(ai, target),
        "a sibling moving within two cells of the candidate rejects it"
    );

    // And the `0x200` flag's own half: the same sibling, moved to a
    // region of its own, stops mattering.
    let elsewhere = s.world.add_region(Terrain::Land);
    s.world.set_region(s.units[sib].pos.cell(), elsewhere);
    assert!(
        !s.scout_unit_near(ai, target),
        "a sibling in another region never rejects the cell"
    );
}

/// **`find_unit_ordered` measures where the sibling is going, not where
/// it stands** (item 1156, `docs/SCOUT.md` §8.1): `0065be35` reads
/// `UnitData +0x70/+0x74` — `orders_x`/`orders_y` — against the
/// candidate's centre; the XOR-ed position is read only for the `0x200`
/// region test. East Indies' `1/28` took cell (37,32) on 6576 because
/// the citizen `1/22` was asked about by its body, more than two cells
/// off; headed to (27384,25080) it rejects the cell, and the scan takes
/// (34,29) as the original's walk does.
#[test]
fn a_sibling_blocks_a_cell_by_where_it_is_going_not_where_it_stands() {
    let (mut s, ai, _) = scout_sim(true);
    let here = s.units[ai].pos.cell();
    let target = Cell::new(here.x + 4, here.y);
    let at = Pos::new(target.x * 768 + 384, target.y * 768 + 384);
    let far = Pos::new(at.x, at.y + 6 * 768);
    let mut sib = crate::Unit::new(1, s.units.len() as i16, at, 20);
    sib.ty = s.units[ai].ty;
    let sib = s.add_unit(sib);
    let walk = |s: &mut Sim, to: Pos| {
        s.units[sib].orders.clear();
        s.add_move_facing_order(
            sib,
            to,
            MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
            crate::movement::Angle(0),
            None,
            false,
        );
    };

    // Standing on the candidate, walking six cells away from it.
    walk(&mut s, far);
    assert!(
        vector_dist(
            s.units[sib].orders_pos.x - at.x,
            s.units[sib].orders_pos.y - at.y
        ) > 0x600,
        "`orders_x/y` is six cells off"
    );
    assert!(
        !s.scout_unit_near(ai, target),
        "a sibling on the cell but headed elsewhere does not reject it"
    );

    // Six cells off, walking onto it.
    s.units[sib].pos = far;
    walk(&mut s, at);
    assert!(
        s.scout_unit_near(ai, target),
        "a sibling headed for the cell rejects it from six cells off"
    );
}

/// **"Same type" is the lineage** (item 3857, `docs/GOLDEN.md` §153.4):
/// `think_scout` hands `find_unit_ordered` `FILTER_BASE_TYPE` and its
/// type's `basic_type`, the root of the `FROM` chain. The fourth
/// melee's Fluyt (`TI` 332) and Aegis Cruiser (`TI` 330) both root at
/// the Bark: the cruiser bound for a cell refuses it to the Fluyt,
/// where an exact-type test let the Fluyt take it on 1957. A ship of
/// another lineage still does not.
#[test]
fn a_sibling_of_the_scout_s_lineage_blocks_a_cell_whatever_its_rung() {
    let (mut s, ai, _) = scout_sim(true);
    let here = s.units[ai].pos.cell();
    let target = Cell::new(here.x + 4, here.y);
    let at = Pos::new(target.x * 768 + 384, target.y * 768 + 384);
    let traits = crate::tech::UnitTraits::default();
    let mut tree = crate::tech::TechTree::default();
    let root = tree.add(crate::tech::TypeDef::unit("Bark", traits));
    let fluyt = tree.add(crate::tech::TypeDef::unit("Fluyt", traits).from(root));
    let mid = tree.add(crate::tech::TypeDef::unit("Cruiser", traits).from(root));
    let aegis = tree.add(crate::tech::TypeDef::unit("Aegis Cruiser", traits).from(mid));
    let other = tree.add(crate::tech::TypeDef::unit("Galley", traits));
    tree.finalize();
    s.set_tech_tree(tree);
    let rec = s.units[ai].ty.expect("the fixture's type");
    let typed = |s: &mut Sim, t| {
        let mut ty = s.tables.unit_types[rec].clone();
        ty.tree = Some(t);
        s.add_unit_type(ty)
    };
    let scout_rec = typed(&mut s, fluyt);
    let aegis_rec = typed(&mut s, aegis);
    let other_rec = typed(&mut s, other);
    s.units[ai].ty = Some(scout_rec);

    let mut sib = crate::Unit::new(1, s.units.len() as i16, Pos::new(at.x - 768, at.y), 20);
    sib.ty = Some(aegis_rec);
    let sib = s.add_unit(sib);
    s.add_move_facing_order(
        sib,
        at,
        MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
        crate::movement::Angle(0),
        None,
        false,
    );
    assert!(
        s.scout_unit_near(ai, target),
        "a cruiser two rungs up the Fluyt's lineage rejects the cell"
    );

    s.units[sib].ty = Some(other_rec);
    assert!(
        !s.scout_unit_near(ai, target),
        "a ship of another lineage does not"
    );
}

/// **§8.3: the rival's multiplier asks whether the two have met, not
/// whether they are at war.** `think_scout` tests `treaties[L] & 3`
/// (`LeaderData +0x94`) at `005f6688`, where this crate read `diplos`
/// (`+0x74`) through `at_war_with`. run360's packet at tick 4506 scores
/// the human's cells with no doubling while the two are at war, because
/// they met on 4456. Both leaders are at war throughout here, so the
/// old reading doubles every row and the met rows fail.
#[test]
fn a_rival_s_cell_is_doubled_until_the_two_have_met() {
    let (mut s, ai, _) = scout_sim(true);
    s.declare_war(0, 1);
    assert!(s.at_war_with(1, 0), "the two are at war throughout");
    let here = s.units[ai].pos.cell();
    let c = Cell::new(here.x + 3, here.y + 1);
    let score = |s: &mut Sim| {
        s.rng.seed = 12345;
        let mut scan = Scan {
            score: NOTHING,
            tile: Pos::default(),
            ring: 3,
            budget: 0,
            over: false,
            leader: 0,
        };
        s.scout_cell(ai, 1, c, &mut scan, 1);
        scan.score
    };
    let unmet = score(&mut s);
    s.cities_state.nation[0].human = false;
    let unmet_computer = score(&mut s);
    s.cities_state.nation[0].human = true;
    s.treaty_on(1, 0, 1);
    let met = score(&mut s);
    assert!(met > 0 && met < NOTHING, "the cell is a candidate: {met}");
    assert_eq!(unmet, 2 * met, "a human stranger's cell counts double");
    assert_eq!(unmet_computer, 4 * met, "a computer stranger's four times");
    s.cities_state.nation[0].human = false;
    assert_eq!(score(&mut s), met, "and a met computer's not at all");
}

/// The guard on the other half of §11: with the stride at 1 the
/// original skips the offset draw outright (`leal -0x1(%ecx), %eax;
/// testl; jg` at `005f6934`), so a small region spends one draw a cell
/// and no more.
/// §3's tail at `005f6db5` (item 1297): a **sea** unit whose region
/// scan finds nothing marks its region scouted and then joins an army —
/// `cmpl $1, 0x218(type)`, `call Unit::add_to_army`, `setns` — where a
/// land unit would ask `think_civilian_transport` for an island. East
/// Indies' Caravel `1/35` gave up on its sea on frame 11523 and joined
/// army 3's group 72, walking to the army's first member on an
/// `ATTACK_TO`; this crate left it standing, and it thought again on
/// 11549 with a stride that found six cells.
#[test]
fn a_sea_scout_with_nothing_to_see_joins_the_nearest_sea_army() {
    let mut world = World::new(40, 40);
    let sea = world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities_state.nation[0].human = true;
    s.cities_state.nation[1].human = false;
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        ..crate::UnitType::default()
    });
    // A ship in all three places the crate reads a domain from.
    s.tables.unit_types[t].cols.role = role::ROLE_SCOUT;
    s.tables.unit_types[t].kind.domain = Domain::Sea;
    s.tables.unit_types[t].combat.domain = Domain::Sea;
    let spawn = |s: &mut Sim, at: Pos| {
        let mut u = crate::Unit::new(1, s.units.len() as i16, at, 20);
        u.ty = Some(t);
        u.kind.domain = Domain::Sea;
        s.add_unit(u)
    };
    let first = spawn(&mut s, Pos::new(30 * 768 + 384, 20 * 768 + 384));
    let scout = spawn(&mut s, Pos::new(5 * 768 + 384, 20 * 768 + 384));
    let slot = s.init_army(1, None);
    s.armies[1].list[slot].reg = Some(sea);
    s.armies[1].list[slot].pos = s.units[first].pos;
    s.army_add_unit(1, slot, first);
    assert_eq!(s.army_of(first), Some(slot));

    // No fog grid: every cell reads as seen, so the scan takes none.
    assert!(s.think_scout(scout), "the join is the call's answer");
    assert!(s.world.region_scouted(sea, 1), "the mark comes first");
    assert_eq!(s.army_of(scout), Some(slot), "it joins the sea army");
    // `go_to_unit` ran: its squad mark is unconditional. The walk
    // itself (`ATTACK_TO` to a spot near `first`) is that function's
    // own test's; this world's sea has no water tiles to stand on.
    assert!(s.units[scout].in_danger, "`go_to_unit`'s mark");
}

#[test]
fn a_region_of_under_a_hundred_cells_draws_no_offset() {
    let (mut s, ai, _) = scout_sim(true);
    s.cities_state.cities[0].alive = false;
    // A region of ten cells around the scout, and the rest of the grid
    // moved to a second one so `region_size` sees only these.
    let small = s.world.add_region(Terrain::Land);
    let here = s.units[ai].pos.cell();
    for x in here.x..here.x + 10 {
        s.world.set_region(Cell::new(x, here.y), small);
    }
    let before = s.rng.seed;
    assert!(s.think_scout(ai));
    let mut r = crate::combat::Rng::new(before);
    let mut spent = 0;
    while r.seed != s.rng.seed && spent < 64 {
        r.roll();
        spent += 1;
    }
    assert_eq!(spent, 10, "ten cells, ten draws, and no offset");
}
