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
        let on_map: Vec<_> = sim.units.iter().filter(|u| u.on_map).collect();
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
            for (f, g) in s.figures.iter().zip(&u.guys) {
                assert_eq!((f.gpiece, f.anim), (g.gpiece, g.anim));
                assert_eq!((f.cur_time, f.end_time), (g.cur_time, g.end_time));
                assert_eq!(f.pos, g.follow.map_or(u.pos, |b| b.body.pos));
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
                (s.ty, s.pos, s.health, s.hits),
                (b.ty, b.pos, b.health, b.hits)
            );
            assert_eq!((s.active, s.constr_time), (b.active, b.constr_time));
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
    assert_eq!(s.figures[0].pos, sim.units[u].pos);
    assert_eq!((s.figures[1].pos, s.figures[1].facing), (body, f.facing));
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
    let mut sim = sim;
    let tv = sim.world.terrain_version();
    let c = crate::Cell::new(2, 1);
    let mut d = sim.world.cell_data(c);
    d.land_sub ^= 1;
    sim.world.set_cell_data(c, d);
    assert_eq!(sim.world.terrain_version(), tv + 1);
    assert_eq!(sim.observe(None).terrain_version, tv + 1);
}
