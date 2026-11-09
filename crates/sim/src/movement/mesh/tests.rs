use super::*;
use crate::Tuning;
use crate::world::{Terrain, World};

/// Sixty cells of plain land and nothing on them. The mesh is the whole
/// subject here, so the fixture has no buildings: `set_road_at` is
/// called by hand, tile by tile, the way `place_roads` calls it.
fn bare() -> Sim {
    let mut w = World::new(60, 60);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
    Sim::new(Tuning::RON, w, 2)
}

fn is_road(sim: &Sim, x: i32, y: i32) -> bool {
    sim.world.tile_mask(Pos::new(x, y)) & tile::SURFACE == tile::SURFACE_ROAD
}

fn lay(sim: &mut Sim, tiles: &[(i32, i32)]) {
    for &(x, y) in tiles {
        sim.set_road_at(Pos::new(x, y));
    }
}

/// An elbow at `(223, 80)` — west and south — is a road with **two**
/// orthogonal connections, which is what `set_diags`' corner branch
/// wants.
fn elbow(sim: &mut Sim) {
    lay(sim, &[(222, 80), (223, 80), (223, 81)]);
    assert_eq!(
        sim.mesh.orthog_connects(&sim.world, Pos::new(223, 80)),
        2,
        "the elbow joins west and south"
    );
}

/// **run72's frame 4803, in miniature.** The road already stands at
/// `(223, 80)`; the new tile is `(224, 79)`, diagonally off it; neither
/// `(223, 79)` nor `(224, 80)` is anything. The mesh lays `(223, 79)` —
/// the seventeenth tile of a sixteen-tile ring.
#[test]
fn a_new_road_diagonally_off_a_junction_fills_the_tile_between_them() {
    let mut sim = bare();
    elbow(&mut sim);
    assert!(!is_road(&sim, 223, 79), "nothing there yet");
    sim.set_road_at(Pos::new(224, 79));
    assert!(is_road(&sim, 223, 79), "the mesh fills the corner");
    assert_eq!(sim.mesh.made(), 1, "and it is the mesh's own tile");
    let made = sim.mesh.elem(&sim.world, Pos::new(223, 79)).unwrap();
    assert!(made.made, "`is_terrain_creation` says who laid it");
    let at = sim.mesh.elem(&sim.world, Pos::new(224, 79)).unwrap();
    assert_eq!(
        at.support & (SUPPORT_MADE | SUPPORT_EAST),
        SUPPORT_MADE,
        "and the tile that laid it remembers, westward"
    );
}

/// The gate: **two orthogonal connections and the pass does nothing at
/// all**. `(224, 79)` here is the middle of a road running east–west, so
/// its own corners are never looked at.
#[test]
fn a_tile_with_two_orthogonal_connections_lays_nothing() {
    let mut sim = bare();
    elbow(&mut sim);
    lay(&mut sim, &[(225, 79), (224, 78)]);
    sim.set_road_at(Pos::new(224, 79));
    assert_eq!(
        sim.mesh.orthog_connects(&sim.world, Pos::new(224, 79)),
        2,
        "north and east"
    );
    assert!(!is_road(&sim, 223, 79), "so the corner is left open");
    assert_eq!(sim.mesh.made(), 0);
}

/// Either flank being a road is enough to stop it: there is already a
/// way round.
#[test]
fn a_flank_that_is_already_a_road_stops_the_fill() {
    let mut sim = bare();
    elbow(&mut sim);
    // `(224, 80)` is the southern flank of the south-west corner.
    lay(&mut sim, &[(224, 80)]);
    let made = sim.mesh.made();
    sim.set_road_at(Pos::new(224, 79));
    assert_eq!(sim.mesh.made(), made, "the way round is the flank itself");
}

/// And a **building footprint** on a flank stops it too — the other half
/// of `neighbor_cache`, and the reason the cache is two arrays rather
/// than one.
#[test]
fn a_footprint_on_a_flank_stops_the_fill_as_a_road_would() {
    let mut sim = bare();
    elbow(&mut sim);
    let t = Pos::new(223, 79);
    let m = sim.world.tile_mask(t);
    sim.world.set_tile_mask(t, m | tile::OBJECT_BUILDING);
    sim.set_road_at(Pos::new(224, 79));
    assert!(!is_road(&sim, 223, 79), "a footprint is not paved over");
    assert_eq!(sim.mesh.made(), 0);
}

/// A corner road that is a **dead end** — one orthogonal connection —
/// takes the other branch: the two elements are joined diagonally and no
/// tile is laid.
#[test]
fn a_dead_end_corner_is_joined_rather_than_filled() {
    let mut sim = bare();
    lay(&mut sim, &[(222, 80), (223, 80)]);
    assert_eq!(
        sim.mesh.orthog_connects(&sim.world, Pos::new(223, 80)),
        1,
        "the road ends there"
    );
    sim.set_road_at(Pos::new(224, 79));
    assert!(!is_road(&sim, 223, 79), "nothing is laid");
    let at = sim.mesh.elem(&sim.world, Pos::new(224, 79)).unwrap();
    assert_eq!(at.flags & dir::SW, dir::SW, "the tile claims its corner");
    let corner = sim.mesh.elem(&sim.world, Pos::new(223, 80)).unwrap();
    assert_eq!(corner.flags & dir::NE, dir::NE, "and the corner answers");
}

/// `mark_and_trim_directions`, straight: a road tile claims exactly the
/// cardinals that are roads, and each neighbour claims it back.
#[test]
fn a_straight_run_claims_the_cardinals_that_are_roads() {
    let mut sim = bare();
    lay(&mut sim, &[(10, 10), (10, 11), (10, 12), (11, 11)]);
    let e = |s: &Sim, x, y| s.mesh.elem(&s.world, Pos::new(x, y)).unwrap().flags;
    assert_eq!(
        e(&sim, 10, 11) & dir::ORTHOG,
        dir::N | dir::S | dir::E,
        "the middle of a T"
    );
    assert_eq!(e(&sim, 10, 10) & dir::ORTHOG, dir::S, "the top end");
    assert_eq!(e(&sim, 11, 11) & dir::ORTHOG, dir::W, "the arm");
}

/// **The removal half, driven by hand.** `clear_roads` is reached from
/// one place — a queue entry with `added == 0`, which `add_roads` only
/// writes when a tile it is laying **already had an element**. Nothing
/// in this crate creates an element on a tile that is not a road, so
/// the branch is unreachable here and the passes below would otherwise
/// never run (`docs/ROADS.md` §9.4). They are called directly instead.
///
/// What they do: a tile the mesh laid itself goes when the last
/// reference does; a tile somebody else laid stays whatever the count
/// says.
#[test]
fn a_cleared_tile_goes_only_if_the_mesh_laid_it() {
    let mut sim = bare();
    elbow(&mut sim);
    sim.set_road_at(Pos::new(224, 79));
    assert!(is_road(&sim, 223, 79), "the fill is down");

    // Somebody else's tile: the count falls, the road stays.
    let plain = Pos::new(222, 80);
    sim.clear_roads(index_of(&sim.world, plain));
    assert!(!is_road(&sim, 222, 80) || sim.mesh.elem(&sim.world, plain).is_none());

    // The mesh's own: `clear_support` on the tile that made it takes
    // both, because `is_terrain_creation` is set and the count runs out.
    let at = Pos::new(224, 79);
    sim.clear_roads(index_of(&sim.world, at));
    assert!(!is_road(&sim, 223, 79), "the fill goes with what made it");
    assert_eq!(sim.mesh.made(), 0);
}

/// Every road tile has an element and nothing else does — the invariant
/// the diffs pin on run72's mid-game grid, here on a fixture.
#[test]
fn an_element_stands_for_each_road_tile_and_no_other() {
    let mut sim = bare();
    elbow(&mut sim);
    sim.set_road_at(Pos::new(224, 79));
    let mut roads = 0;
    for y in 70..90 {
        for x in 215..235 {
            let t = Pos::new(x, y);
            let road = is_road(&sim, x, y);
            roads += i32::from(road);
            assert_eq!(
                road,
                sim.mesh.elem(&sim.world, t).is_some(),
                "tile ({x}, {y})"
            );
        }
    }
    assert_eq!(roads, 5, "three elbow tiles, the new one, and the fill");
    assert_eq!(sim.mesh.len(), 5, "and no element outside them");
}

// ------------------------------------------------------------------
// The stray-road sweep (`docs/ROADS.md` §10)
// ------------------------------------------------------------------

/// A north–south road of six tiles on open ground, `(100, 40)` to
/// `(100, 45)`.
fn lane(sim: &mut Sim) {
    let tiles: Vec<(i32, i32)> = (40..46).map(|y| (100, y)).collect();
    lay(sim, &tiles);
}

/// **A road with nothing at its ends erodes from both, a tile a visit**
/// — Great Lakes' trade road from (216, 115), a stub on 5905, to
/// (220, 98) on 14100. Each end claims one neighbour, with no building
/// and no water beside it: `scan_and_kill_straggled_tcoord` takes it,
/// and the removal trims the next tile down to one claim of its own.
///
/// Made to fail once with the straggler arm removed: the lane stood.
#[test]
fn a_lane_with_nothing_at_its_ends_erodes_a_tile_a_visit() {
    let mut sim = bare();
    lane(&mut sim);
    sim.scan_stray_tile(Pos::new(100, 44));
    assert!(is_road(&sim, 100, 44), "the middle claims two: it stands");
    sim.scan_stray_tile(Pos::new(100, 45));
    assert!(!is_road(&sim, 100, 45), "the south end is a stub");
    assert!(sim.mesh.elem(&sim.world, Pos::new(100, 45)).is_none());
    assert_eq!(
        sim.mesh
            .elem(&sim.world, Pos::new(100, 44))
            .map(|e| e.flags),
        Some(dir::N),
        "the next tile is trimmed to its one claim"
    );
    for y in (40..45).rev() {
        sim.scan_stray_tile(Pos::new(100, y));
    }
    assert!(
        (40..46).all(|y| !is_road(&sim, 100, y)),
        "and the lane is gone"
    );
}

/// **A building beside the end keeps it** — the straggler test counts
/// `(mask & 3) == 3` neighbours, so a road that reaches a footprint is
/// not a stub. Great Lakes' (216, 116) stands beside the city.
#[test]
fn a_road_that_ends_at_a_building_is_not_a_stub() {
    let mut sim = bare();
    lane(&mut sim);
    let foot = Pos::new(101, 46);
    let m = sim.world.tile_mask(foot);
    sim.world.set_tile_mask(foot, m | tile::OBJECT_BUILDING);
    sim.scan_stray_tile(Pos::new(100, 45));
    assert!(is_road(&sim, 100, 45), "a footprint beside it: it stands");
}

/// **A road tile with no element goes, and nothing is counted down** —
/// `scan_and_kill_bad_tcoord`'s first arm, `rotation == 10`.
#[test]
fn a_road_tile_the_mesh_does_not_hold_is_taken_away() {
    let mut sim = bare();
    let t = Pos::new(100, 40);
    let m = sim.world.tile_mask(t);
    sim.world.set_tile_mask(t, m | tile::SURFACE_ROAD);
    assert!(sim.mesh.elem(&sim.world, t).is_none());
    sim.scan_stray_tile(t);
    assert!(!is_road(&sim, 100, 40));
}

/// **The cursor is the frame's**: seven cells a frame on a 60 × 60
/// map, stepped before each, so the frame that has just become 14,529
/// scans cells 897 to 903 — where run189's packet holds `curscan` —
/// and not 904.
#[test]
fn the_sweep_s_cursor_is_seven_cells_a_frame_from_cell_zero() {
    let mut sim = bare();
    let at = |c: i32| Pos::new((c % 60) * 4, (c / 60) * 4);
    for c in [896, 897, 903, 904] {
        let m = sim.world.tile_mask(at(c));
        sim.world.set_tile_mask(at(c), m | tile::SURFACE_ROAD);
    }
    sim.frame = 14_529;
    sim.scan_and_kill_stray_roads();
    let road = |sim: &Sim, c: i32| is_road(sim, at(c).x, at(c).y);
    assert!(road(&sim, 896), "cell 896 was the frame before's");
    assert!(
        !road(&sim, 897) && !road(&sim, 903),
        "897..903 are this frame's"
    );
    assert!(road(&sim, 904), "904 is the next frame's");
}

/// **A tile that becomes blocked loses its road** (`World::set_blocked_at@006b4900`'s
/// `set_road_at(x, y, 0, 0, 0)`, `docs/ROADS.md` §9.5, item 1185). East
/// Indies' Temple `1/2025` started on 7479 over the caravan road's tile
/// (200, 202); the original's caravan found it plain ground on 7512 and
/// laid its road again, and this crate kept the road under the blocked
/// tile. Unblocking lays nothing back, and a neighbour keeps its road.
#[test]
fn a_tile_that_becomes_blocked_loses_its_road() {
    let mut sim = bare();
    lay(&mut sim, &[(20, 20), (21, 20), (22, 20)]);
    assert!(is_road(&sim, 21, 20));
    sim.set_blocked_at(Pos::new(21, 20), true);
    assert!(!is_road(&sim, 21, 20), "the blocked tile's road goes");
    assert!(
        sim.world.tile_mask(Pos::new(21, 20)) & tile::BLOCKED != 0,
        "and it is blocked"
    );
    assert!(
        is_road(&sim, 20, 20) && is_road(&sim, 22, 20),
        "its neighbours keep theirs"
    );
    sim.set_blocked_at(Pos::new(21, 20), false);
    assert!(!is_road(&sim, 21, 20), "unblocking lays nothing");
    // **Every call clears**, not only a tile's first blocking: the
    // original's `set_road_at` sits outside the `& 0x4000` guard that
    // keeps the counts.
    sim.set_blocked_at(Pos::new(21, 20), true);
    lay(&mut sim, &[(21, 20)]);
    assert!(is_road(&sim, 21, 20), "a road laid over a blocked tile");
    sim.set_blocked_at(Pos::new(21, 20), true);
    assert!(!is_road(&sim, 21, 20), "goes when it is blocked again");
}

/// **A road beside a footprint stays joined to its run** —
/// `RoadsOut::leech_codes@00890da0` in the redo pass (`docs/ROADS.md`
/// §11, item 1260). Great Sahara at Toughest's Farm took the west end
/// of a trade road on 6611; the tile beside it was re-derived with a
/// footprint to its west, so the trim recomputed north–south alone and
/// the tile's claim east was lost with the zeroing. The east neighbour's
/// claim west lends it back. Without it the tile claims nothing, and the
/// sweep takes it and then the rest of the run as stubs, where the
/// original kept them.
#[test]
fn a_road_beside_a_footprint_keeps_the_claim_its_neighbour_lends_it() {
    let mut sim = bare();
    lay(
        &mut sim,
        &[(20, 20), (21, 20), (22, 20), (23, 20), (24, 20)],
    );
    let end = Pos::new(20, 20);
    let m = sim.world.tile_mask(end);
    sim.world.set_tile_mask(end, m | tile::OBJECT_BUILDING);
    sim.world_set_road_at(end, false, 0, 0);
    assert!(
        !is_road(&sim, 20, 20),
        "the footprint's tile loses its road"
    );
    assert_eq!(
        sim.mesh
            .elem(&sim.world, Pos::new(21, 20))
            .map(|e| e.flags & 0xff00_0000),
        Some(dir::E),
        "the tile beside it claims east, lent by (22, 20)'s west"
    );
    // The far end, (24, 20), is a stub in open country and erodes on
    // its own visit; the tile beside the footprint is what is asked.
    sim.scan_stray_tile(Pos::new(21, 20));
    assert!(
        (21..25).all(|x| is_road(&sim, x, 20)),
        "and the sweep leaves it standing"
    );
}

/// **A road neighbour with no element ends the walk** — each arm of
/// `leech_codes` returns from inside when `get_road_data` answers
/// nothing, clearing this tile's claim toward it and leaving the
/// cardinals after it unread.
#[test]
fn leech_codes_stops_at_a_road_with_no_element() {
    let mut sim = bare();
    lay(&mut sim, &[(30, 30), (31, 30)]);
    // A road north of (30, 30) that the mesh does not hold.
    let north = Pos::new(30, 29);
    let m = sim.world.tile_mask(north);
    sim.world.set_tile_mask(north, m | tile::SURFACE_ROAD);
    let at = Pos::new(30, 30);
    sim.mesh.set_flags(&sim.world, at, dir::N);
    sim.mesh.at = at;
    sim.mesh.fill_cache(&sim.world, at);
    sim.mesh.leech_codes(&sim.world);
    assert_eq!(
        sim.mesh.elem(&sim.world, at).map(|e| e.flags & 0xff00_0000),
        Some(0),
        "north is cleared, and east is never lent"
    );
    // With the north road gone, the walk reaches east.
    sim.world.set_tile_mask(north, m);
    sim.mesh.fill_cache(&sim.world, at);
    sim.mesh.leech_codes(&sim.world);
    assert_eq!(
        sim.mesh.elem(&sim.world, at).map(|e| e.flags & 0xff00_0000),
        Some(dir::E),
        "(31, 30) claims west, so (30, 30) claims east"
    );
}

/// **And a building that starts over a road takes it** — the same arm,
/// reached the way the Temple reached it: `Wall::start` →
/// `BuildType::mask_me` → `set_blocked_at` on each template tile of a
/// type that connects to roads, where `mask_me`'s own removal arm does
/// not fire.
#[test]
fn a_building_that_starts_over_a_road_takes_it_under_its_blocked_tiles() {
    let mut sim = bare();
    let bt = sim.add_build_type(crate::build::BuildType {
        x_size: 3,
        y_size: 3,
        ..crate::build::BuildType::default()
    });
    assert!(
        crate::roads::connects_to_roads(&sim.tables.build_types, bt),
        "the type connects to roads, so mask_me's own arm lays rather than clears"
    );
    let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
    let b = sim.add_building(0, at, 0);
    sim.buildings[b].ty = Some(bt);
    sim.buildings[b].started = false;
    let corner = sim.tile_corner(bt, at);
    let under = Pos::new(corner.x + 1, corner.y + 1);
    lay(
        &mut sim,
        &[
            (under.x - 3, under.y),
            (under.x - 2, under.y),
            (under.x - 1, under.y),
            (under.x, under.y),
        ],
    );
    assert!(is_road(&sim, under.x, under.y));
    sim.start_building(b);
    assert!(
        sim.world.tile_mask(under) & tile::BLOCKED != 0,
        "the footprint's middle is blocked"
    );
    assert!(!is_road(&sim, under.x, under.y), "and its road is gone");
}
