use super::*;
use crate::Pos;
use crate::build::{self, Ident};
use crate::combat::Rng;
use crate::tuning::Tuning;
use crate::world::World;

/// Draws between two words of the stream, walked forward.
fn draws(from: u32, to: u32) -> usize {
    let mut r = Rng::new(from);
    for n in 0..64 {
        if r.seed == to {
            return n;
        }
        r.roll();
    }
    panic!("{to:#010x} is not within 64 draws of {from:#010x}");
}

/// The float sequences, computed in single precision — the oracle the
/// two pinned counts stand on.
#[test]
fn the_single_precision_crossings() {
    let step = 0.005f32;
    let mut x = 0.0f32;
    let mut n = 0;
    while x < 1.0 {
        x += step;
        n += 1;
    }
    assert_eq!(n, RIPE_ADDS, "adds of 0.005f to reach 1.0f");
    assert!(
        x > 1.0,
        "and the 201st overshoots, so `1.0f < x` ripens too"
    );
    let mut y = 1.0f32;
    let mut m = 0;
    while y > 0.0 {
        y -= 0.01f32;
        m += 1;
    }
    assert_eq!(m, 101, "subtractions of 0.01f from 1.0f to reach zero");
    // At m = 100 the float is still (barely) positive — the reason a cut
    // cell with `adds == 0` is not yet empty.
    let mut z = 1.0f32;
    for _ in 0..100 {
        z -= 0.01f32;
    }
    assert!(z > 0.0);
}

/// A farmed cell — `grow` and `inc_time` each frame — is ripe after
/// frame 100's `grow` (the 201st add), which is what puts the
/// original's re-target on the log's frame 102 (`docs/ORDERS.md` §6.5).
#[test]
fn a_farmed_cell_ripens_on_frame_100() {
    let mut f = Farm::default();
    for frame in 0..200 {
        f.grow(5);
        if f.state[5] == RIPE {
            assert_eq!(frame, 100);
            assert_eq!(f.adds[5], FULL);
            break;
        }
        let empty = f.advance();
        assert_eq!(empty, 15);
        assert_eq!(f.state[5], GROWING);
        assert_eq!(f.adds[5], 2 * (frame + 1));
    }
    assert_eq!(f.state[5], RIPE);
    // Cut, then 101 frames of decay to empty.
    f.snip(5);
    assert_eq!(f.state[5], CUT);
    let mut m = 0;
    while f.state[5] == CUT {
        f.advance();
        m += 1;
    }
    assert_eq!(m, 101);
    assert_eq!(f.state[5], EMPTY);
    assert_eq!(f.adds[5], 0);
}

/// A sprouted cell nobody farms grows by one add a frame and ripens on
/// its 201st.
#[test]
fn a_sprout_ripens_on_its_201st_add() {
    let mut f = Farm::default();
    f.state[0] = GROWING;
    let mut n = 0;
    while f.state[0] != RIPE {
        f.advance();
        n += 1;
    }
    assert_eq!(n, RIPE_ADDS);
}

/// A flat world with one complete farm at tile (5, 5).
fn farm_sim() -> (Sim, usize) {
    let mut s = Sim::new(Tuning::RON, World::new(16, 16), 2);
    let t = s.add_build_type(build::BuildType {
        ident: Ident::Farm,
        x_size: 3,
        y_size: 3,
        flags: build::flags::FLAT,
        job_time: 100,
        hits: 500,
        ..build::BuildType::default()
    });
    let b = s.init_build(0, t, Pos::new(5 * 192 + 96, 5 * 192 + 96), false);
    s.activate(b, false, true);
    s.buildings[b].farm.farm_type = 0;
    (s, b)
}

/// Another complete farm of the same type at a tile.
///
/// There is no city on this world, so [`Sim::farms_add`] falls to the
/// cityless coin — a pasture on an odd object index — and the farms the
/// capture pins are the original's, which are crops in a city. The type
/// is set back here rather than founding a city, so these pins keep
/// measuring `Farms::inc_time` and nothing else.
fn add_farm(s: &mut Sim, t: usize, tx: i32, ty: i32) {
    let b = s.init_build(0, t, Pos::new(tx * 192 + 96, ty * 192 + 96), false);
    s.activate(b, false, true);
    s.buildings[b].farm.farm_type = 0;
}

/// A pasture — `farm_type == 1` — and its five animals of owner 9:
/// the crop draw `Farms::inc_time` would have spent is not spent, and
/// the five each roll an idle variant in the unit loop, the first of
/// them taking `think_farm_animal`'s draw as well because its phase
/// `o · (slot + 1) + frame` is zero at frame 0. Six draws where a crop
/// farm spends one — run20's frame 0, draws 138–143 (`docs/SYNC.md`
/// §3.6).
/// **A pasture's close takes its five animals, and spends no draw**
/// (item 1591, `Build::close@00628980:273–297`): each is closed through
/// `Unit::close(0, −1, 0)` — dead, its slot held thirty frames, no
/// death roll — and a second pasture's animals stand. run710's `9/9`,
/// whose pasture `0/2002` the ICBM flattened on 1734, thought on 1747
/// in this crate. Made to fail with the walk dropped (the five alive).
#[test]
fn a_pasture_s_close_takes_its_animals_and_spends_no_draw() {
    let (mut s, b) = farm_sim();
    let chicken = s.add_unit_type(crate::UnitType {
        hits: 10,
        ..crate::UnitType::default()
    });
    s.unit_types[chicken].tree = Some(FARMCHICKEN);
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    let animals = s.farm_add_animals(b, &[]);
    let t = s.buildings[b].ty.unwrap();
    let other = s.init_build(0, t, Pos::new(10 * 192 + 96, 10 * 192 + 96), false);
    s.activate(other, false, true);
    s.buildings[other].farm.farm_type = ANIMAL_FARM;
    let others = s.farm_add_animals(other, &[]);
    assert_eq!(others.len(), FARM_ANIMALS as usize);
    let seed = s.rng.seed;
    s.die_building(b);
    assert_eq!(s.rng.seed, seed, "no draw");
    for &u in &animals {
        assert!(!s.units[u].alive(), "the pasture's animal is closed");
        assert_eq!(s.units[u].hold_frames, Sim::CLOSE_HOLD, "its slot held");
    }
    assert!(
        others.iter().all(|&u| s.units[u].alive()),
        "another pasture's animals stand"
    );
}

#[test]
fn a_pasture_spends_no_crop_draw_and_six_on_its_animals() {
    let (mut s, b) = farm_sim();
    let chicken = s.add_unit_type(crate::UnitType {
        hits: 10,
        ..crate::UnitType::default()
    });
    s.unit_types[chicken].tree = Some(FARMCHICKEN);
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    let animals = s.farm_add_animals(b, &[]);
    assert_eq!(animals.len(), FARM_ANIMALS as usize, "five animals");
    assert!(
        animals.iter().all(|&u| s.units[u].owner == 9),
        "of owner 9 — the slot no dump prints"
    );
    assert_eq!(
        animals
            .iter()
            .map(|&u| s.units[u].index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4],
        "numbered from the empty leader-9 list"
    );

    // The farm itself: no draw, where a crop farm with sixteen empty
    // cells would take one.
    let seed = s.rng.seed;
    s.farms_inc_time();
    assert_eq!(s.rng.seed, seed, "a pasture grows nothing");
    s.buildings[b].farm.farm_type = 0;
    s.farms_inc_time();
    assert_ne!(s.rng.seed, seed, "and a crop farm does draw");

    // The animals, in the unit loop: one idle roll each, and the first
    // animal's `think_farm_animal` on top.
    s.rng = Rng::new(seed);
    let mut spent = Vec::new();
    for &u in &animals {
        let before = s.rng.seed;
        s.animal_idle(u);
        spent.push(draws(before, s.rng.seed));
    }
    assert_eq!(spent, vec![2, 1, 1, 1, 1], "the six of §3.6");
}

/// **A pasture finished mid-game stocks itself, and the stream is
/// run33's frame 1372 word for word.**
///
/// `Build::activate` reaches `Farms::add_animals` for every food gather
/// building it completes, and this is what that costs: four draws an
/// animal — the species coin, the `y` offset, the `x` offset and
/// `Guy::init_real`'s variant — twenty in all, in that order.
///
/// The pin is the original's own. run33's trace has the AI's fourth farm
/// activate on frame 1372 under `Wall::do_construct+0x199`, with
/// `game_random` on `0xc91f99f2`; twenty draws later the original's
/// twenty-first draw of that frame starts from `0xafa38116`. Between
/// them the five coins are all even — a pasture's four-draw stride keeps
/// every one on the same parity of the stream — so all five are chickens,
/// and the ten offsets are the pairs below.
#[test]
fn a_finished_pasture_stocks_five_animals_for_twenty_draws() {
    let mut s = Sim::new(Tuning::RON, World::new(16, 16), 2);
    let t = s.add_build_type(build::BuildType {
        ident: Ident::Farm,
        x_size: 3,
        y_size: 3,
        flags: build::flags::FLAT | build::flags::GATHERER,
        job_time: 100,
        hits: 500,
        ..build::BuildType::default()
    });
    let chicken = s.add_unit_type(crate::UnitType {
        hits: 10,
        ..crate::UnitType::default()
    });
    s.unit_types[chicken].tree = Some(FARMCHICKEN);
    let pig = s.add_unit_type(crate::UnitType {
        hits: 10,
        ..crate::UnitType::default()
    });
    s.unit_types[pig].tree = Some(FARMPIG);

    let pos = crate::Pos::new(5 * 192 + 96, 5 * 192 + 96);
    let b = s.init_build(0, t, pos, false);
    // `Farms::add`'s pick is not what is under test — the capture's own
    // farm is a pasture because its city already held four crops — so it
    // is written here the way the dump writes it, and the frame is moved
    // off zero because zero is the setup's.
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    s.frame = 1_372;
    s.trace_phases = true;

    s.rng = Rng::new(0xc91f_99f2);
    s.activate(b, false, true);

    assert_eq!(
        draws(0xc91f_99f2, s.rng.seed),
        20,
        "four draws an animal and nothing else"
    );
    assert_eq!(s.rng.seed, 0xafa3_8116, "run33's own word twenty draws on");
    assert_eq!(
        s.phase_marks
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>(),
        [
            SITE_ANIMAL_COIN,
            SITE_ANIMAL_Y,
            SITE_ANIMAL_X,
            crate::anim::SITE_INIT_REAL
        ]
        .repeat(5),
        "the coin, the y, the x and the guy — five times over"
    );

    let animals: Vec<usize> = (0..s.units.len())
        .filter(|&u| s.units[u].farm_animal.is_some())
        .collect();
    assert_eq!(animals.len(), FARM_ANIMALS as usize, "five animals");
    assert!(animals.iter().all(|&u| s.units[u].owner == 9), "of owner 9");
    assert!(
        animals.iter().all(|&u| s.units[u].ty == Some(chicken)),
        "every coin of the five is even, so all five are chickens"
    );
    assert_eq!(
        animals.iter().map(|&u| s.units[u].pos).collect::<Vec<_>>(),
        [(-126, -77), (-154, -89), (10, -165), (110, 79), (146, -125)]
            .iter()
            .map(|&(dx, dy)| crate::orders::snapped(crate::Pos::new(pos.x + dx, pos.y + dy)))
            .collect::<Vec<_>>(),
        "each `rnd % 0x180 - 0xc0` from the farm, and snapped by `Unit::init`"
    );
    assert_eq!(
        animals
            .iter()
            .map(|&u| s.units[u].farm_animal.unwrap().slot)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4],
        "stamped with their place in the five — `think_farm_animal`'s phase"
    );

    // And a second activation of the same farm is what the *setup* path
    // must never be handed: the guard is the type, not a count, so the
    // harness's frame-0 farms are stocked from their trace instead
    // (`Sim::farm_add_animals`) and `Build::activate` is not asked twice.
    s.buildings[b].farm.farm_type = 0;
    let word = s.rng.seed;
    s.activate(b, false, true);
    assert_eq!(s.rng.seed, word, "a crop farm's activation draws nothing");
}

/// `think_farm_animal`'s two gates, each made to fail: the 128-frame
/// phase, and the farm covering the tile it measures — the farm's own
/// while nobody **has arrived** to gather there, and `gather_down`'s
/// head once somebody has.
///
/// **The middle case is the one that carried a word.** The count is
/// `BuildData::num_gatherers(this, 1, 0)`, which is `is_gathering_at`
/// with `arrived = 1` — `been_there` (`docs/ORDERS.md` §6.1) — and a
/// citizen joins the chain at *issue*, frames before it gets there.
/// Reading the chain's length instead put the reference on a citizen
/// still walking, five tiles off the farm, and dropped the draw:
/// Great Lakes' word stopped at run53's 2930 for exactly that
/// (`docs/SYNC.md` §3.6).
#[test]
fn think_farm_animal_s_phase_and_its_covers_tile() {
    let (mut s, b) = farm_sim();
    let pig = s.add_unit_type(crate::UnitType {
        hits: 10,
        ..crate::UnitType::default()
    });
    s.unit_types[pig].tree = Some(FARMPIG);
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    let animals = s.farm_add_animals(b, &[]);

    // Animal 1 (`o 1`, slot 1) is off phase at frame 0 — `1 · 2 + 0`.
    let seed = s.rng.seed;
    s.think_farm_animal(animals[1]);
    assert_eq!(s.rng.seed, seed, "off phase");
    // …and on it at frame 126, `1 · 2 + 126 == 128`.
    s.frame = 126;
    s.think_farm_animal(animals[1]);
    assert_ne!(s.rng.seed, seed, "on phase");

    // The covers test, and the arrival that decides which object it
    // measures. A citizen on the chain but still walking — far off the
    // farm, `been_there` clear — leaves the reference on the **farm**,
    // which covers its own tile, so the draw is spent.
    s.frame = 0;
    s.build_types[s.buildings[b].ty.unwrap()].flags |= build::flags::GATHERER;
    let citizen = s.add_unit_type(crate::UnitType {
        hits: 10,
        worker: crate::orders::Worker::Citizen,
        ..crate::UnitType::default()
    });
    let mut walker = crate::Unit::new(0, 99, Pos::new(0, 0), 10);
    walker.ty = Some(citizen);
    let g = s.add_unit(walker);
    s.add_gather_order(g, b, crate::orders::QueuePos::New, false);
    assert!(
        s.is_gathered_by(b, g),
        "on the chain from the moment of issue"
    );
    assert_eq!(s.num_gatherers(b, false, false), 1, "and one by length");
    assert_eq!(s.num_gatherers(b, true, false), 0, "but none arrived");
    let seed = s.rng.seed;
    s.think_farm_animal(animals[0]);
    assert_ne!(
        s.rng.seed, seed,
        "a gatherer that has not arrived leaves the reference on the farm"
    );

    // Arrived and off the farm: now it *is* the reference, the farm
    // does not cover its tile, and nothing is drawn.
    let Some(crate::orders::Body::Gather(mut go)) =
        s.action_of(g).map(|i| s.units[g].orders[i].body)
    else {
        panic!("the gather order is the action")
    };
    go.been_there = true;
    let i = s.action_of(g).unwrap();
    s.units[g].orders[i].body = crate::orders::Body::Gather(go);
    assert_eq!(s.num_gatherers(b, true, false), 1, "arrived");
    let seed = s.rng.seed;
    s.think_farm_animal(animals[0]);
    assert_eq!(s.rng.seed, seed, "the gatherer is off the farm");
    // Standing it on the farm brings the draw back.
    s.units[g].pos = s.buildings[b].pos;
    s.think_farm_animal(animals[0]);
    assert_ne!(s.rng.seed, seed, "and on it, the walk rolls");
}

/// **Where the one draw sends it.** `docs/SYNC.md` §3.11: the tile of
/// the reference object plus `move_x/move_y[dir + 1]`, then the slot's
/// own corner of that tile — `192·T + 24`, `+ 120` or `+ 168` for a
/// corner of −1, 0 or +1, which is the low edge, the middle and the
/// high side of a 192-unit tile.
///
/// **An animal is born on the snap of its two draws, not on them.**
///
/// `Unit::init@00612100:69` snaps every unit's starting position, and
/// `Objects::init_unit` hands `Farms::add_animals`' raw
/// `building ± (rnd % 0x180 − 0xc0)` straight to it. run39's first
/// animal is born at `(40536, 40392)` where its draws say
/// `(40552, 40369)`, which is 432 units of walk rather than 455 — at 25
/// a frame, eighteen steps and not nineteen, and the arrival lands on
/// the frame the word parts (`docs/SYNC.md` §3.11, the pair).
///
/// Made to fail by placing the animal on the raw point.
#[test]
fn an_animal_is_born_on_the_snap_of_its_two_draws() {
    let (mut s, b) = farm_sim();
    let chicken = s.add_unit_type(crate::UnitType {
        hits: 10,
        moves: 25,
        ..crate::UnitType::default()
    });
    s.unit_types[chicken].tree = Some(FARMCHICKEN);
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    // `+0x134` and `+0x182` are a `y` and an `x` offset in
    // `[−0xc0, 0xbf]`; neither of these lands on a cell centre.
    let seeds = [AnimalSeed {
        chicken: true,
        dy: -119,
        dx: 43,
    }; FARM_ANIMALS as usize];
    let pos = s.buildings[b].pos;
    let animals = s.farm_add_animals(b, &seeds);
    let raw = crate::Pos::new(pos.x + 43, pos.y - 119);
    let snapped = crate::orders::snapped(raw);
    assert_ne!(raw, snapped, "the draws do not name a cell centre");
    for &u in &animals {
        assert_eq!(s.units[u].pos, snapped);
        assert_eq!(s.units[u].pos.x.rem_euclid(48), 24);
        assert_eq!(s.units[u].pos.y.rem_euclid(48), 24);
    }
}

/// The corner is what this is made to fail on: five animals of one
/// pasture, all measuring the same farm and all rolling the same
/// direction, land on **three** different points, and which one is
/// [`crate::world::CORNER_X`]'s row for the slot.
#[test]
fn the_walk_lands_on_the_slot_s_corner_of_the_tile_it_picks() {
    let (mut s, b) = farm_sim();
    let chicken = s.add_unit_type(crate::UnitType {
        hits: 10,
        moves: 25,
        ..crate::UnitType::default()
    });
    s.unit_types[chicken].tree = Some(FARMCHICKEN);
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    let seeds = [AnimalSeed {
        chicken: true,
        dy: 0,
        dx: 0,
    }; FARM_ANIMALS as usize];
    let animals = s.farm_add_animals(b, &seeds);
    let t = s.buildings[b].pos.tile();

    for (slot, &u) in animals.iter().enumerate() {
        // Each animal's phase — `o · (slot + 1) + frame ≡ 0` — with
        // `o == slot` here, so the five fire on `128 − slot(slot + 1)`:
        // 128, 126, 122, 116, 108, which is §3.11's own set.
        s.frame = THINK_PERIOD - (slot * (slot + 1)) as i64;
        // The direction the draw is about to return, taken off a copy
        // of the generator so the assertion names a point rather than
        // a range.
        let dir = (crate::combat::Rng::new(s.rng.seed).roll() & 7) as usize;
        let (mx, my) = crate::world::MOVE_8[dir];
        s.think_farm_animal(u);
        let Some(crate::orders::Body::Move(mo)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("animal {slot} took no MOVE_TO");
        };
        let edge = |c: i32| match c {
            -1 => 24,
            0 => 120,
            _ => 168,
        };
        assert_eq!(
            (mo.dest.x, mo.dest.y),
            (
                192 * (t.x + mx) + edge(crate::world::CORNER_X[slot]),
                192 * (t.y + my) + edge(crate::world::CORNER_Y[slot]),
            ),
            "animal {slot}'s corner of tile ({}, {})",
            t.x + mx,
            t.y + my
        );
    }
    // The centre and the four corners really are three distinct
    // points on each axis, so the table is doing work.
    let xs: std::collections::BTreeSet<i32> = animals
        .iter()
        .map(|&u| match s.units[u].orders.front().map(|o| o.body) {
            Some(crate::orders::Body::Move(m)) => m.dest.x % 192,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(
        xs,
        [24, 120, 168].into_iter().collect(),
        "the low edge, the middle and the high side"
    );
}

/// Run12, frame 1: the stream after the frame's 48th draw, one farm
/// with its farmer's cell `(dx 2, dy 2)` growing and fifteen empty —
/// the sprout: `% 1000 = 19 < 20`, then `% 15 = 12`, the thirteenth
/// empty cell in [`Farm::nth_empty`]'s order, **memory index 7** —
/// the cell the frame-2 dump shows starting to grow (`docs/SYNC.md`
/// §4).
#[test]
fn run12_s_frame_1_sprout_lands_on_the_dump_s_cell() {
    let (mut s, b) = farm_sim();
    let mut r = Rng::new(0xb619_4ba1);
    for _ in 0..48 {
        r.roll();
    }
    s.rng = r;
    s.buildings[b].farm.state[2 * 4 + 2] = GROWING;
    s.buildings[b].farm.adds[2 * 4 + 2] = 3;
    s.farms_inc_time();
    let f = &s.buildings[b].farm;
    assert_eq!(f.state[7], GROWING, "(dx 1, dy 3) sprouted");
    assert_eq!(f.adds[7], 0);
    assert_eq!(f.state.iter().filter(|&&x| x == GROWING).count(), 2);
    let mut r2 = Rng::new(0xb619_4ba1);
    for _ in 0..50 {
        r2.roll();
    }
    assert_eq!(s.rng.seed, r2.seed, "two draws");
}

/// Run12, frame 0: six farms from the frame's 110th draw, no sprout —
/// six draws, all `% 1000 ≥ 20`.
#[test]
fn run12_s_frame_0_farms_draw_six_and_sprout_nothing() {
    let (mut s, b0) = farm_sim();
    let t = s.buildings[b0].ty.unwrap();
    for i in 1..6 {
        add_farm(&mut s, t, 5 + 4 * (i % 3), 5 + 4 * (i / 3));
    }
    let mut r = Rng::new(0x3bd3_9ae9);
    for _ in 0..110 {
        r.roll();
    }
    s.rng = r;
    s.farms_inc_time();
    for _ in 0..6 {
        r.roll();
    }
    assert_eq!(s.rng.seed, r.seed);
    assert!(s.buildings.iter().all(|b| b.farm.state == [EMPTY; 16]));
}

/// Run13, sim-frame 101 (`docs/SYNC.md` §4.1): from the frame's 14th
/// draw off the end-of-100 word `0xa45fecaf`, the six complete farms in
/// the original's `Farms` order — the AI's three, then the human's —
/// with the cells the end-of-100 pass shows (print index `p` is the
/// column-major position, so cell `(p % 4) * 4 + p / 4`; every farmer's
/// cell 10 is ripe, the rest are sprouts). Farm 1, the AI's `2003` with
/// twelve empties, sprouts at draw 15 (`% 1000 = 6`) and draw 16 picks
/// `65297 % 12 = 5`, the sixth empty column-major — the one cell that
/// appears in the next pass. Seven draws, and the stream lands on the
/// original's end-of-101 word `0x08670a66`.
#[test]
fn run13_s_frame_101_sprout_lands_on_the_ai_s_second_farm() {
    let (mut s, b0) = farm_sim();
    let t = s.buildings[b0].ty.unwrap();
    for i in 1..6 {
        add_farm(&mut s, t, 5 + 4 * (i % 3), 5 + 4 * (i / 3));
    }
    // Print-index → sim cell.
    let cell = |p: usize| (p % 4) * 4 + p / 4;
    let busy: [&[usize]; 6] = [
        &[7, 10, 11],
        &[7, 10, 14, 15],
        &[9, 10, 12, 13],
        &[2, 7, 10],
        &[9, 10, 12],
        &[10, 15],
    ];
    for (f, ps) in busy.iter().enumerate() {
        for &p in ps.iter() {
            let c = cell(p);
            if p == 10 {
                s.buildings[f].farm.state[c] = RIPE;
                s.buildings[f].farm.adds[c] = FULL;
            } else {
                s.buildings[f].farm.state[c] = GROWING;
                s.buildings[f].farm.adds[c] = 40;
            }
        }
    }
    let before: Vec<[u8; 16]> = s.buildings.iter().map(|b| b.farm.state).collect();
    let mut r = Rng::new(0xa45f_ecaf);
    for _ in 0..14 {
        r.roll();
    }
    s.rng = r;
    s.farms_inc_time();
    assert_eq!(s.rng.seed, 0x0867_0a66, "the original's end-of-101 word");
    for (f, b) in s.buildings.iter().enumerate() {
        let mut want = before[f];
        if f == 1 {
            want[cell(5)] = GROWING;
        }
        assert_eq!(b.farm.state, want, "farm {f}");
    }
}

/// A city on the flat world, so [`Sim::farms_add`] has one to count.
fn city_sim() -> (Sim, usize, usize) {
    let (mut s, first) = farm_sim();
    let t = s.buildings[first].ty.unwrap();
    let village = s.add_build_type(build::BuildType {
        ident: Ident::Village,
        x_size: 7,
        y_size: 7,
        job_time: 100,
        hits: 1000,
        ..build::BuildType::default()
    });
    // The world is 16 cells square; the farm of `farm_sim` sits at tile
    // (5, 5) and would join this city, which is not what the table below
    // counts, so it goes.
    s.buildings[first].alive = false;
    s.farm_order.clear();
    let c = s.init_build(0, village, Pos::new(20 * 192 + 96, 20 * 192 + 96), false);
    s.activate(c, false, true);
    let city = s.buildings[c].city.expect("a finished city has a record");
    (s, t, city)
}

/// `Farms::add`'s branch table and its two draw sites (`docs/SYNC.md`
/// §3.8) — the two draws run20's frame 1 spends and the sim did not.
///
/// Five placements, each naming what it costs:
///
/// 1. **No city** — the coin is the object index's own parity, and it
///    takes no draw either way.
/// 2. **The city's first farm** — nothing to copy and nothing near, so
///    `Farms::add` rolls: one draw at `+0x128`.
/// 3. **The second, beside the first** — `get_nearest_farm_type` answers
///    and there is no roll; one crop is not "more than one", so no
///    ambience. **No draw at all.**
/// 4. **The third** — the nearest still answers, and now the city has
///    two crops: the emitter's **two draws**, and the farm keeps
///    [`AMBIENCE`].
/// 5. **The fourth** — the city's emitter is taken, so the walk ends on
///    the third farm and nothing is spent.
#[test]
fn farms_add_picks_the_type_and_hands_out_one_ambience_a_city() {
    let (mut s, t, city) = city_sim();

    // 1. Cityless: far outside the city radius, so `find_city` finds
    //    none and the parity of `o` decides. Neither draws.
    let before = s.rng.seed;
    let lone = s.init_build(0, t, Pos::new(2 * 192 + 96, 2 * 192 + 96), false);
    let lone2 = s.init_build(0, t, Pos::new(2 * 192 + 96, 6 * 192 + 96), false);
    assert_eq!(s.buildings[lone].city, None, "outside every city radius");
    assert_eq!(
        s.buildings[lone].farm.farm_type,
        u8::from(s.buildings[lone].index & 1 != 0)
    );
    assert_eq!(
        s.buildings[lone2].farm.farm_type,
        u8::from(s.buildings[lone2].index & 1 != 0)
    );
    assert_ne!(
        s.buildings[lone].farm.farm_type, s.buildings[lone2].farm.farm_type,
        "consecutive object numbers, so one of the two is the pasture"
    );
    assert_eq!(s.rng.seed, before, "the cityless coin is not a draw");
    s.buildings[lone].alive = false;
    s.buildings[lone2].alive = false;

    // 2. The city's first farm: the roll.
    s.trace_phases = true;
    let f1 = s.init_build(0, t, Pos::new(18 * 192 + 96, 20 * 192 + 96), false);
    assert_eq!(s.buildings[f1].city, Some(city), "inside the city");
    assert_eq!(
        s.phase_marks
            .iter()
            .map(|m| m.0.as_str())
            .collect::<Vec<_>>(),
        vec![SITE_TYPE_COIN],
        "the first farm of a city rolls for its type"
    );
    // The seed is the harness's own, so which side the coin lands on is
    // not the assertion; that it is a legal `FarmType` is.
    assert!(s.buildings[f1].farm.farm_type <= ANIMAL_FARM);
    s.buildings[f1].farm.farm_type = 0;
    // A citizen has reached it: `get_nearest_farm_type` finds only a
    // started farm (the next test).
    s.buildings[f1].started = true;

    // 3. The second, four tiles off: copied from the first, no draw.
    s.phase_marks.clear();
    let f2 = s.init_build(0, t, Pos::new(22 * 192 + 96, 20 * 192 + 96), false);
    assert_eq!(s.buildings[f2].city, Some(city));
    assert_eq!(s.buildings[f2].farm.farm_type, 0, "the neighbour's type");
    assert!(s.phase_marks.is_empty(), "one crop is not more than one");

    // 4. The third: the ambience pair, and the bit.
    s.phase_marks.clear();
    let f3 = s.init_build(0, t, Pos::new(20 * 192 + 96, 22 * 192 + 96), false);
    assert_eq!(
        s.phase_marks
            .iter()
            .map(|m| m.0.as_str())
            .collect::<Vec<_>>(),
        vec![SITE_AMBIENCE_X, SITE_AMBIENCE_Y],
        "x then y, and no coin"
    );
    assert_eq!(s.buildings[f3].farm.farm_type, AMBIENCE);
    assert_eq!(
        s.city_count_farms(city),
        3,
        "an emitter's crop still counts"
    );

    // 5. The fourth: the emitter is taken.
    s.phase_marks.clear();
    let f4 = s.init_build(0, t, Pos::new(20 * 192 + 96, 18 * 192 + 96), false);
    assert_eq!(s.buildings[f4].farm.farm_type, 0);
    assert!(s.phase_marks.is_empty(), "one emitter a city");

    // And the list is joined at placement — these four have never been
    // activated — in placement order, the first two into the slots the
    // dead `lone` pair left (`Farms::add`'s first free record).
    assert_eq!(s.farm_order, vec![f1, f2, f3, f4]);
}

/// **A freed slot is refilled** (item 1639, `docs/AI.md` §175):
/// `Farms::add` returns the first record whose `valid` byte
/// `Farms::remove` cleared, so a farm placed after a demolition is walked
/// where the dead farm stood — ahead of the farms placed in between.
#[test]
fn a_placement_takes_the_first_freed_slot() {
    let (mut s, t, _city) = city_sim();
    let a = s.init_build(0, t, Pos::new(16 * 192 + 96, 16 * 192 + 96), false);
    let b = s.init_build(0, t, Pos::new(24 * 192 + 96, 16 * 192 + 96), false);
    let c = s.init_build(0, t, Pos::new(16 * 192 + 96, 24 * 192 + 96), false);
    assert_eq!(s.farm_order, vec![a, b, c]);
    s.buildings[a].alive = false;
    let d = s.init_build(0, t, Pos::new(24 * 192 + 96, 24 * 192 + 96), false);
    assert_eq!(
        s.farm_order,
        vec![d, b, c],
        "the dead farm's slot, not the end"
    );
    let e = s.init_build(0, t, Pos::new(8 * 192 + 96, 8 * 192 + 96), false);
    assert_eq!(s.farm_order, vec![d, b, c, e], "none free: it appends");
}

/// **A farm site nobody has started is not a neighbour** (`docs/AI.md`
/// §79), East Indies' 23182 in miniature. `find_any_building` keeps a
/// building only when `WallData::is_started` answers (`flags & 2`), so
/// with two started crops out of range, a fourth farm placed beside a
/// third that is still a bare site finds no neighbour, takes no draw,
/// and is the city's `others == crops == 3` pasture. Once the third is
/// started, the same placement copies its crop and takes the emitter.
#[test]
fn a_farm_beside_an_unstarted_site_is_the_fourth_s_pasture() {
    let place = |started: bool| {
        let (mut s, t, city) = city_sim();
        // Two crops, eight tiles apart and six-plus from the pair below.
        for x in [16, 24] {
            let f = s.init_build(0, t, Pos::new(x * 192 + 96, 16 * 192 + 96), false);
            s.buildings[f].farm.farm_type = 0;
            s.buildings[f].started = true;
        }
        // The third, a bare site; its type is the harness's to set.
        let third = s.init_build(0, t, Pos::new(19 * 192 + 96, 23 * 192 + 96), false);
        s.buildings[third].farm.farm_type = 0;
        s.buildings[third].started = started;
        assert_eq!(s.city_count_farms(city), 3);
        // The fourth, five tiles east of the third: 960, as on 23182.
        s.trace_phases = true;
        s.phase_marks.clear();
        let seed = s.rng.seed;
        let fourth = s.init_build(0, t, Pos::new(24 * 192 + 96, 23 * 192 + 96), false);
        assert_eq!(s.buildings[fourth].city, Some(city));
        let marks: Vec<String> = s.phase_marks.iter().map(|m| m.0.clone()).collect();
        (
            s.buildings[fourth].farm.farm_type & ANIMAL_FARM,
            s.rng.seed == seed,
            marks,
        )
    };
    let (ty, quiet, marks) = place(false);
    assert_eq!(
        ty, ANIMAL_FARM,
        "no started neighbour: the fourth is a pasture"
    );
    assert!(quiet && marks.is_empty(), "and it takes no draw: {marks:?}");
    // A crop with three crops beside it takes the city's emitter, as
    // `1/2031` did on the same frame; the pasture above never does.
    let (ty, _, marks) = place(true);
    assert_eq!(ty, 0, "a started neighbour's crop is copied");
    assert_eq!(marks, [SITE_AMBIENCE_X, SITE_AMBIENCE_Y], "and no coin");
}

/// A site (not yet active) and a pasture draw nothing; a farm with
/// four or fewer empty cells draws nothing.
#[test]
fn the_gates() {
    let (mut s, b) = farm_sim();
    let seed = s.rng.seed;
    s.buildings[b].active = false;
    s.farms_inc_time();
    assert_eq!(s.rng.seed, seed, "a site");
    s.buildings[b].active = true;
    s.buildings[b].farm.farm_type = ANIMAL_FARM;
    s.farms_inc_time();
    assert_eq!(s.rng.seed, seed, "a pasture");
    s.buildings[b].farm.farm_type = 0;
    for c in 0..12 {
        s.buildings[b].farm.state[c] = GROWING;
        s.buildings[b].farm.adds[c] = 1;
    }
    s.farms_inc_time();
    assert_eq!(s.rng.seed, seed, "four empty cells");
    s.buildings[b].farm.state[11] = CUT;
    s.buildings[b].farm.adds[11] = 0;
    s.farms_inc_time();
    assert_ne!(s.rng.seed, seed, "five: chance (5 − 4)·20/8 = 2");
}
