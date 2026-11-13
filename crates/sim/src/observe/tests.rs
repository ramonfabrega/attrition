use super::*;
use crate::soak::play_with;

/// RENDERER §2.3 rule 3: observing every tick leaves the game where not
/// observing would — the soak digest bit-identical, frame for frame.
#[test]
fn observing_every_tick_leaves_the_digest_alone() {
    for seed in [3u32, 11, 19] {
        let plain = play_with(seed, 300, |_| {});
        let watched = play_with(seed, 300, |sim| {
            let _ = sim.observe(None);
            let _ = sim.observe(Some(1));
            let _ = sim.observe_terrain();
            let _ = sim.observe_territory();
        });
        assert_eq!(plain.len(), watched.len());
        for (i, (a, b)) in plain.iter().zip(&watched).enumerate() {
            assert_eq!(a.replay, b.replay, "seed {seed}: observing moved frame {i}");
        }
    }
}

/// A scene built on a soak game agrees with the `Sim` fields it copies, on
/// every frame: one unit scene per unit on the map, in store order, and one
/// building scene per building in use.
#[test]
fn a_scene_agrees_with_the_sim_it_copies() {
    let mut frames = 0;
    let mut figures = 0;
    play_with(5, 300, |sim| {
        let scene = sim.observe(None);
        assert_eq!(scene.frame, sim.frame);
        assert_eq!(scene.fog, None);
        let on_map: Vec<_> = sim.units.iter().filter(|u| u.on_map && u.alive()).collect();
        assert_eq!(scene.units.len(), on_map.len());
        for (s, u) in scene.units.iter().zip(&on_map) {
            assert_eq!(
                s.id,
                ObjId {
                    who: u.owner,
                    o: u.index
                }
            );
            assert_eq!((s.owner, s.ty, s.pos), (u.owner, u.ty, u.pos));
            assert_eq!((s.health, s.max_health), (u.health, u.max_health));
            assert_eq!(s.facing, u.movement.facing);
            assert_eq!(s.z, sim.world.tile_z(u.pos.tile()));
            assert_eq!(s.figures.len(), u.guys.len());
            for (k, (f, g)) in s.figures.iter().zip(&u.guys).enumerate() {
                assert_eq!((f.gpiece, f.anim), (g.gpiece, g.anim));
                assert_eq!((f.cur_time, f.end_time), (g.cur_time, g.end_time));
                let own = if k == 0 { u.movement.body.pos } else { u.pos };
                assert_eq!(f.pos, g.follow.map_or(own, |b| b.body.pos));
            }
            figures += s.figures.len();
        }
        let alive: Vec<_> = sim.buildings.iter().filter(|b| b.alive).collect();
        assert_eq!(scene.buildings.len(), alive.len());
        for (s, b) in scene.buildings.iter().zip(&alive) {
            assert_eq!(
                s.id,
                ObjId {
                    who: b.owner,
                    o: b.index
                }
            );
            assert_eq!(
                (s.ty, s.pos, s.z, s.health, s.hits),
                (b.ty, b.pos, b.z, b.health, b.hits)
            );
            assert_eq!((s.active, s.constr_time), (b.active, b.constr_time));
            assert_eq!(
                (s.orig_ty, s.job_counter, s.tribe, s.age),
                (
                    b.orig_ty,
                    b.job_counter,
                    sim.tech[b.founder as usize].tribe,
                    sim.tech[b.owner as usize].ages
                )
            );
        }
        assert_eq!(scene.projectiles.len(), sim.combat.projectiles.len());
        assert_eq!(scene.deaths.len(), sim.combat.deaths.len());
        assert_eq!(scene.terrain_version, sim.world.terrain_version());
        assert_eq!(scene.territory_version, sim.world.territory_version());
        frames += 1;
    });
    assert_eq!(frames, 300);
    // The soak's units carry no figures, so the figure rows above compared
    // nothing: give one unit of a mid-game copy two, a crew body on the second.
    assert_eq!(
        figures, 0,
        "the soak game drew figures: the copy below is redundant"
    );
    let mut copy = None;
    play_with(5, 150, |sim| copy = Some(sim.clone()));
    let mut sim = copy.unwrap();
    let u = sim.units.iter().position(|u| u.on_map).unwrap();
    let mut guy = crate::units::anim::Guy::fresh(7);
    guy.anim = 3;
    guy.cur_time = 4;
    guy.end_time = 9;
    sim.set_guy(u, 0, guy);
    sim.set_guy(u, 1, guy);
    let body = Pos::new(sim.units[u].pos.x + 100, sim.units[u].pos.y);
    let mut f = sim.units[u].guys[0]
        .follow
        .unwrap_or(crate::units::anim::Follow {
            body: crate::movement::Body::at(body),
            des: body,
            facing: Angle(5),
            des_angle: Angle(5),
            track: (0, 0),
        });
    f.body.pos = body;
    sim.units[u].guys[1].follow = Some(f);
    let scene = sim.observe(None);
    let s = scene
        .units
        .iter()
        .find(|s| s.id.o == sim.units[u].index && s.id.who == sim.units[u].owner);
    let s = s.unwrap();
    assert_eq!(s.figures.len(), 2);
    assert_eq!((s.figures[0].gpiece, s.figures[0].anim), (7, 3));
    assert_eq!((s.figures[0].cur_time, s.figures[0].end_time), (4, 9));
    assert_eq!(s.figures[0].pos, sim.units[u].movement.body.pos);
    assert_eq!((s.figures[1].pos, s.figures[1].facing), (body, f.facing));
    // A push that does not move the guys (`set_new_location`'s
    // `move_guys` false, a later unit's collision) moves the unit and
    // leaves guy 0's figure on its own point until the guy next moves
    // (item 4027: run1429's `1/9` on 2501, `1/67` on 3001).
    let (was, figure_was) = (sim.units[u].pos, s.figures[0].pos);
    let to = Pos::new(was.x - 10, was.y - 11);
    sim.set_new_location(u, to, false);
    let pushed = sim.observe(None);
    let p = pushed.units.iter().find(|p| p.id == s.id).unwrap();
    assert_eq!(p.pos, to);
    assert_eq!(p.figures[0].pos, figure_was);
    sim.set_new_location(u, to, true);
    let carried = sim.observe(None);
    let c = carried.units.iter().find(|c| c.id == s.id).unwrap();
    assert_eq!(c.figures[0].pos, to, "a move that carries the guys");
    // Each figure stands on the ground under its own point (item 4026),
    // not on its unit's tile.
    if sim.unit_domain_of(u) == Domain::Land {
        for fig in &s.figures {
            assert_eq!(fig.z, sim.world.data_z(fig.pos.x, fig.pos.y).0);
        }
    }
    // And a unit at no health leaves the scene though its record stays on
    // the map: the original has taken it out of its object array.
    sim.units[u].health = 0;
    let gone = sim.observe(None);
    assert!(sim.units[u].on_map);
    assert!(!gone.units.iter().any(|g| g.id == s.id));
    assert_eq!(gone.units.len(), scene.units.len() - 1);
}

/// The grids: one entry per cell, and the territory counter moves when a
/// claimant changes and only then.
#[test]
fn the_grids_and_their_counters() {
    let mut w = crate::World::new(4, 3);
    let v = w.territory_version();
    w.set_owner(crate::Cell::new(1, 1), Owner::None, Owner::None);
    assert_eq!(
        w.territory_version(),
        v,
        "an unchanged write moved the counter"
    );
    let sim = Sim::new(crate::Tuning::RON, w, 2);
    let t = sim.observe_territory();
    assert_eq!((t.width, t.height, t.owner.len()), (4, 3, 12));
    let g = sim.observe_terrain();
    assert_eq!((g.land.len(), g.land_sub.len()), (12, 12));
    assert!(
        g.corners.is_empty(),
        "a world with no height table has no corners"
    );
    let mut sim = sim;
    let flat = vec![0i64; 17 * 13];
    let mut ramp = flat.clone();
    ramp[17 * 2 + 5] = 250_500_000;
    assert!(sim.world.set_corner_grid(ramp));
    let g = sim.observe_terrain();
    assert_eq!(g.corners.len(), 17 * 13, "(4·xs + 1) × (4·ys + 1) corners");
    assert_eq!(
        g.corners[17 * 2 + 5],
        0x437a_8000,
        "a corner is its single's bits: 250.5"
    );
    assert_eq!(
        g.corners
            .iter()
            .zip(sim.world.corner_singles())
            .filter(|(a, b)| **a != b.bits())
            .count(),
        0
    );
    let tv = sim.world.terrain_version();
    let c = crate::Cell::new(2, 1);
    let mut d = sim.world.cell_data(c);
    d.land_sub ^= 1;
    sim.world.set_cell_data(c, d);
    assert_eq!(sim.world.terrain_version(), tv + 1);
    assert_eq!(sim.observe(None).terrain_version, tv + 1);
}

/// **The door carries a round's piece and a corpse's** (item 4175,
/// `docs/RENDERER.md` §3.13): `ProjectileScene::gpiece` is the round's
/// own, and `DeathScene::pieces` lists one piece per figure record whose
/// life still stands on the frame — the record whose life has run out is
/// gone, as `DeathObj::inc_time` zeroes its `valid`.
#[test]
fn the_door_carries_the_round_s_piece_and_the_corpse_s() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(8, 8), 2);
    let mut round = crate::combat::Projectile {
        shooter: crate::combat::Obj::Unit(0),
        owner: 1,
        target: None,
        launch: Pos::new(0, 0),
        landing: Pos::new(768, 0),
        cur_time: 0,
        total_time: 10,
        accuracy: 0,
        angle: crate::movement::Angle(0),
        splash_area: 0,
        num_guys: 1,
        air: false,
        rolling: false,
        missed: false,
        harmless: false,
        sz: 0,
        ez: 0,
        v1z: crate::num::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
        gpiece: crate::combat::FIRST_AMMO_PIECE + 8,
    };
    sim.combat.projectiles.push(round);
    round.gpiece = -1;
    sim.combat.projectiles.push(round);
    let mut figures = crate::combat::Figures::one(Some(40));
    figures.n = 3;
    figures.life[1] = 20;
    figures.life[2] = i64::MAX;
    figures.gpiece[..3].copy_from_slice(&[219, 12891, 25563]);
    // Each record's own place (item 4191), distinct, so a record read from
    // its neighbour parts.
    for k in 0..3 {
        figures.place[k] = crate::combat::Lie {
            pos: crate::world::Pos {
                x: 3972 + k as i32,
                y: 28574 - k as i32,
            },
            z: 499 + k as i32,
            angle: crate::movement::Angle(-476_512_256 + k as i32),
        };
    }
    sim.combat.deaths.push(crate::combat::Death {
        who: 1,
        o: 4,
        first_frame: 0,
        cur_anim: 17,
        life: Some(40),
        figures,
    });
    let scene = sim.observe(None);
    let pieces: Vec<i32> = scene.projectiles.iter().map(|p| p.gpiece).collect();
    assert_eq!(pieces, [60157, -1], "each round's own piece");
    assert_eq!(
        scene.deaths[0].pieces,
        [219, 12891, 25563],
        "every record stands on frame 0"
    );
    assert_eq!(scene.deaths[0].cur_anim, 17);
    let xs: Vec<i32> = scene.deaths[0].places.iter().map(|l| l.pos.x).collect();
    assert_eq!(xs, [3972, 3973, 3974], "each record's own place");
    assert_eq!(
        scene.deaths[0].frames.len(),
        3,
        "one action length a record"
    );
    sim.frame = 20;
    let later = sim.observe(None);
    assert_eq!(
        later.deaths[0].pieces,
        [219, 25563],
        "the second record's life ran out on 20"
    );
    assert_eq!(
        later.deaths[0]
            .places
            .iter()
            .map(|l| l.z)
            .collect::<Vec<_>>(),
        [499, 501],
        "the places leave with their records"
    );
    sim.frame = 40;
    assert_eq!(
        sim.observe(None).deaths[0].pieces,
        [25563],
        "and the first's on 40"
    );
}

/// **The leaders behind the door** (item 4198, `docs/HUD.md` §4): one
/// [`LeaderScene`] per player slot, each a copy of the ledger, the muster
/// and the tech record it reads — on every frame of a soak game, and on a
/// sim whose every copied field holds a value of its own, so a field read
/// from its neighbour parts. Made to fail once by reading `rate` for
/// `income`.
#[test]
fn a_leader_scene_is_the_records_it_copies() {
    let check = |sim: &Sim| {
        let scene = sim.observe(None);
        assert_eq!(scene.leaders.len(), sim.tech.len());
        for (who, l) in scene.leaders.iter().enumerate() {
            let (t, e, m) = (
                &sim.tech[who],
                &sim.economy.ledgers[who],
                &sim.roster.muster[who],
            );
            assert_eq!(usize::from(l.who), who);
            assert_eq!((l.tribe, l.ages), (t.tribe, t.ages));
            assert_eq!((l.bucket, l.income), (e.bucket, e.income));
            assert_eq!((l.control, l.pop_cap), (m.control, m.cap));
            for g in 0..6 {
                assert_eq!(
                    l.over_cap[g],
                    e.over_cap[g] != crate::economy::OverCap::Under
                );
            }
        }
        scene
    };
    play_with(7, 300, |sim| {
        check(sim);
    });
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(8, 8), 2);
    for who in 0..2 {
        let k = who as i32 * 1000;
        let e = &mut sim.economy.ledgers[who];
        e.bucket = std::array::from_fn(|g| k + 10 + g as i32);
        e.income = std::array::from_fn(|g| k + 20 + g as i32);
        e.rate = std::array::from_fn(|g| k + 30 + g as i32);
        e.over_cap[2] = crate::economy::OverCap::At;
        sim.roster.muster[who].control = k + 41;
        sim.roster.muster[who].cap = k + 42;
        sim.tech[who].ages = who as i32 + 3;
    }
    let scene = check(&sim);
    assert_eq!(
        scene.leaders[1].bucket,
        [1010, 1011, 1012, 1013, 1014, 1015]
    );
    assert_eq!(scene.leaders[1].income[0], 1020);
    assert_eq!(
        scene.leaders[1].over_cap,
        [false, false, true, false, false, false]
    );
    assert_eq!(
        (
            scene.leaders[1].control,
            scene.leaders[1].pop_cap,
            scene.leaders[1].ages
        ),
        (1041, 1042, 4)
    );
}

/// **The door carries the road mesh** (item 4199, `docs/RENDERER.md`
/// §3.19): one row per element, its connections the element's top byte and
/// its compass of roads the tile masks'. Three tiles laid in a line join
/// east and west, the middle both ways, and each knows its neighbours.
#[test]
fn the_door_carries_the_road_mesh() {
    let w = crate::World::new(4, 3);
    let mut sim = Sim::new(crate::Tuning::RON, w, 2);
    assert!(sim.observe_roads().roads.is_empty());
    for x in 5..8 {
        sim.set_road_at(Pos::new(x, 6));
    }
    let r = sim.observe_roads();
    assert_eq!((r.tiles_wide, r.tiles_high), (16, 12));
    let at: Vec<(i32, i32, u8, u16)> = r.roads.iter().map(|t| (t.x, t.y, t.dirs, t.road)).collect();
    // `dir::E` is 0x10 and `dir::W` 0x01 in the top byte; the compass bit
    // `k` is `MOVE_X`/`MOVE_Y`'s neighbour `k`, 0 the tile, 4 east, 8 west.
    assert_eq!(
        at,
        vec![
            (5, 6, 0x10, 1 | 1 << 4),
            (6, 6, 0x11, 1 | 1 << 4 | 1 << 8),
            (7, 6, 0x01, 1 | 1 << 8),
        ]
    );
    assert!(r.roads.iter().all(|t| t.refs > 0 && t.footprint == 0));
}
