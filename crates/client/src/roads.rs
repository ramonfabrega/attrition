//! **The roads** (item 4199, `docs/RENDERER.md` §3.19): draw 91,
//! `modern_road_pieces.tga` under `RoadsOut`'s state, one piece of
//! `Data/roads.xml` laid on each road tile the read door reports
//! (`sim::observe::RoadsScene`).
//!
//! Three steps, each a function here:
//!
//! 1. **The candidate's flags** ([`candidates`]): the record's four bytes,
//!    the connections the simulation keeps on top, then the yellow lines,
//!    the white lines and the intersections the texture passes mark
//!    (`RoadsOut::mark_white_lines@00892c10`, `mark_intersections@00892390`).
//! 2. **The piece** ([`find_map`]): `RoadsPieces::find_map@00897840` over
//!    every piece's eight orientations ([`rotations`], `RoadsPieces::
//!    rotate_flags@00895b20`), the texture fractal choosing among the
//!    matches.
//! 3. **The quad** ([`lay`]): the box and texcoords `RoadsOut::
//!    build_render_data@00894890` gives the piece on its tile, read off
//!    run1559b's packet (`tools/render/road_records.py`'s `piece` rows).

use rondata::artdata::{RoadArt, RoadPiece};
use sim::observe::{RoadTile, RoadsScene};
use std::collections::BTreeMap;

/// The compass's bits in a flag byte, clockwise from NW at bit 7.
pub mod bit {
    pub const NW: u8 = 0x80;
    pub const N: u8 = 0x40;
    pub const NE: u8 = 0x20;
    pub const E: u8 = 0x10;
    pub const SE: u8 = 0x08;
    pub const S: u8 = 0x04;
    pub const SW: u8 = 0x02;
    pub const W: u8 = 0x01;
}

/// The road tile's compass in `MOVE_X`/`MOVE_Y` order, as the door's
/// [`RoadTile::road`] and [`RoadTile::footprint`] index it.
const NW: usize = 1;
const N: usize = 2;
const NE: usize = 3;
const E: usize = 4;
const SE: usize = 5;
const S: usize = 6;
const SW: usize = 7;
const W: usize = 8;
const STEP: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// A byte turned `k` places clockwise, two places a quarter turn
/// (`rotate_flags`' `(b << 8 | b) >> 2k`).
fn turn(b: u8, k: u32) -> u8 {
    b.rotate_right(k)
}

/// The mirror across the north–south axis, `rotate_flags`' bit table for
/// the connections and the intersections: W↔E, SW↔SE, NW↔NE.
fn mirror(b: u8) -> u8 {
    let at = |i: u32| (b >> i) & 1;
    at(4) | at(3) << 1 | at(2) << 2 | at(1) << 3 | at(0) << 4 | at(7) << 5 | at(6) << 6 | at(5) << 7
}

/// The mirror of a line byte, `rotate_flags`' `local_c = {0, 2, 1, 3}` over
/// its pairs: each pair's two bits swapped, and the pairs taken in the
/// order 3, 0, 1, 2.
fn mirror_lines(b: u8) -> u8 {
    const SWAP: [u8; 4] = [0, 2, 1, 3];
    let p = |k: u32| SWAP[usize::from((b >> (2 * k)) & 3)];
    p(3) << 6 | p(0) << 4 | p(1) << 2 | p(2)
}

/// **`RoadsPieces::rotate_flags@00895b20`**: a piece's eight orientations,
/// `RoadElement::flags[8]`. `[0]` is the file's bytes (connections,
/// yellow, white, intersections, high to low); `[1..4]` each byte turned
/// one, two, three quarter turns; `[4]` the mirror; `[5..8]` the mirror
/// turned.
pub fn rotations(p: &RoadPiece) -> [u32; 8] {
    let word = |r: u8, y: u8, w: u8, i: u8| u32::from_be_bytes([r, y, w, i]);
    let (mr, my, mw, mi) = (
        mirror(p.road),
        mirror_lines(p.yellow),
        mirror_lines(p.white),
        mirror(p.intersect),
    );
    let mut out = [0; 8];
    for k in 0..4u32 {
        let s = 2 * k;
        out[k as usize] = word(
            turn(p.road, s),
            turn(p.yellow, s),
            turn(p.white, s),
            turn(p.intersect, s),
        );
        out[4 + k as usize] = word(turn(mr, s), turn(my, s), turn(mw, s), turn(mi, s));
    }
    out
}

/// The pieces with their orientations, and `C31`, the fallback.
pub struct Pieces {
    pub pieces: Vec<RoadPiece>,
    pub flags: Vec<[u32; 8]>,
    pub c31: usize,
}

impl Pieces {
    pub fn new(art: &RoadArt) -> Pieces {
        Pieces {
            flags: art.pieces.iter().map(rotations).collect(),
            c31: art.named("C31").unwrap_or(0),
            pieces: art.pieces.clone(),
        }
    }
}

/// **The candidate's flags on every road tile** (§3.19): the connections
/// the door carries, and the white lines and intersections the texture
/// passes mark, evaluated once on the mesh as it stands. Yellow lines are
/// never marked here (`mark_yellow_lines` is not read; 13 of run1559b's
/// 592 drawn records carry one).
///
/// The white lines, `mark_white_lines`' arms, each a half-segment of an
/// edge line where no road stands beside the tile:
/// - a tile joined north or south: with no road east, `N`'s bit toward a
///   road north (and `SE` on that neighbour), `SE` toward one south (`N`
///   on it); with no road west, `NW` and `S` the same way;
/// - a tile joined east or west: with no road north, `NE` toward a road east
///   (`W` on it) and `W` toward one west (`NE` on it); with no road south,
///   `E` and `SW`;
/// - an inside corner, joined on both of its sides with a footprint on the
///   diagonal between: both halves, and the far half on each neighbour.
///
/// A neighbour is written only where it holds a record. The intersections
/// are `mark_intersections`': a tile joined east and west takes `E` where its
/// east neighbour is not also joined both ways, and `W` likewise; north and
/// south the same.
pub fn candidates(scene: &RoadsScene) -> BTreeMap<(i32, i32), u32> {
    let rec: BTreeMap<(i32, i32), &RoadTile> =
        scene.roads.iter().map(|r| ((r.x, r.y), r)).collect();
    let mut white: BTreeMap<(i32, i32), u8> = rec.keys().map(|&k| (k, 0)).collect();
    let put = |at: (i32, i32), k: usize, b: u8, white: &mut BTreeMap<(i32, i32), u8>| {
        let t = (at.0 + STEP[k].0, at.1 + STEP[k].1);
        if let Some(w) = white.get_mut(&t) {
            *w |= b;
        }
    };
    for r in &scene.roads {
        let at = (r.x, r.y);
        let road = |k: usize| r.road >> k & 1 != 0;
        let foot = |k: usize| r.footprint >> k & 1 != 0;
        let d = r.dirs;
        let mut own = 0u8;
        if d & (bit::N | bit::S) != 0 {
            for (side, upper, lower) in [(E, bit::N, bit::SE), (W, bit::NW, bit::S)] {
                if road(side) {
                    continue;
                }
                if road(N) {
                    own |= upper;
                    put(at, N, lower, &mut white);
                }
                if road(S) {
                    own |= lower;
                    put(at, S, upper, &mut white);
                }
            }
        }
        if d & (bit::E | bit::W) != 0 {
            for (side, east, west) in [(N, bit::NE, bit::W), (S, bit::E, bit::SW)] {
                if road(side) {
                    continue;
                }
                if road(E) {
                    own |= east;
                    put(at, E, west, &mut white);
                }
                if road(W) {
                    own |= west;
                    put(at, W, east, &mut white);
                }
            }
        }
        // The inside corners: the two dirs, the footprint's diagonal, the
        // tile's two halves and each neighbour's far half.
        for (a, b, diag, mine, (na, ba), (nb, bb)) in [
            (
                bit::N,
                bit::W,
                NW,
                bit::NW | bit::W,
                (N, bit::S),
                (W, bit::NE),
            ),
            (
                bit::N,
                bit::E,
                NE,
                bit::N | bit::NE,
                (N, bit::SE),
                (E, bit::W),
            ),
            (
                bit::S,
                bit::W,
                SW,
                bit::S | bit::SW,
                (S, bit::NW),
                (W, bit::E),
            ),
            (
                bit::S,
                bit::E,
                SE,
                bit::E | bit::SE,
                (S, bit::N),
                (E, bit::SW),
            ),
        ] {
            if d & a != 0 && d & b != 0 && foot(diag) {
                own |= mine;
                put(at, na, ba, &mut white);
                put(at, nb, bb, &mut white);
            }
        }
        *white.get_mut(&at).expect("its own record") |= own;
    }
    let joined = |t: (i32, i32), both: u8| rec.get(&t).map(|r| r.dirs & both == both);
    rec.iter()
        .map(|(&at, r)| {
            let mut inter = 0u8;
            for (both, (fwd, fb), (back, bb)) in [
                (bit::E | bit::W, (E, bit::E), (W, bit::W)),
                (bit::N | bit::S, (N, bit::N), (S, bit::S)),
            ] {
                if r.dirs & both != both {
                    continue;
                }
                for (k, b) in [(fwd, fb), (back, bb)] {
                    let t = (at.0 + STEP[k].0, at.1 + STEP[k].1);
                    if joined(t, both) == Some(false) {
                        inter |= b;
                    }
                }
            }
            let word = u32::from_be_bytes([r.dirs, 0, white[&at], inter]);
            (at, word)
        })
        .collect()
}

/// **`RoadsPieces::find_map@00897840`**: every piece's every orientation
/// that matches `cand`, in piece then orientation order, and the one the
/// texture fractal's height at the tile picks, `h · n / 100` truncated and
/// held below `n`. With no match, [`Pieces::c31`] unturned.
///
/// The match: with no diagonal connection (`0xaa000000` clear), the word
/// whole; or, where an intersection is marked, the word with the yellow
/// pair each marked end names cleared, or the word with no intersections.
/// With a diagonal, the connections alone.
pub fn find_map(p: &Pieces, cand: u32, height: u8) -> (usize, u8) {
    let mut found = Vec::new();
    for (e, flags) in p.flags.iter().enumerate() {
        for (r, &f) in flags.iter().enumerate() {
            let hit = if cand & 0xaa00_0000 == 0 {
                f == cand || (cand & 0xff != 0 && (f == stripped(cand) || f == cand & 0xffff_ff00))
            } else {
                (f ^ cand) & 0xff00_0000 == 0
            };
            if hit {
                found.push((e, r as u8));
            }
        }
    }
    if found.is_empty() {
        return (p.c31, 0);
    }
    let n = found.len() as f32;
    let mut at = f32::from(height) * (n / 100.0);
    if n - 0.5 <= at {
        at = n - 1.0;
    }
    found[at as usize]
}

/// `find_map`'s yellow strip: each marked end (`N`, `E`, `S`, `W` in the
/// low byte) clears the yellow pair beside it.
fn stripped(cand: u32) -> u32 {
    let mut out = cand;
    for (end, pair) in [
        (0x40, 0x00c0_0000),
        (0x10, 0x0030_0000),
        (0x04, 0x000c_0000),
        (0x01, 0x0003_0000),
    ] {
        if cand & end != 0 {
            out &= !pair;
        }
    }
    out
}

/// One piece laid on its tile: the box in tiles, `[x0, y0, x1, y1]`, and
/// the texcoord at its NW, NE, SW and SE corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Laid {
    pub tile: (i32, i32),
    pub elem: usize,
    pub rot: u8,
    pub rect: [f32; 4],
    pub uv: [[f32; 2]; 4],
}

/// A half-tile count to the nearest, ties to even: the x87's own rounding.
fn half_grid(v: f32) -> f32 {
    (v * 2.0).round_ties_even() / 2.0
}

/// **The piece's quad** (§3.19), as run1559b's packet holds every one of
/// its 28 drawn pieces. The piece is `width × height` half tiles, its sides
/// swapped on an odd orientation (a quarter turn either way); its box
/// starts `CENTEROFFSET[rot]` half tiles above and left of the tile's
/// corner, the offset's two halves swapped likewise, on the half-tile
/// grid. The texcoords span `OFFSETX`, `OFFSETY` and the size in
/// sixteenths, each corner's fraction `(s, t)` of the box's `(X, Y)` by
/// the orientation's table: the four quarter turns clockwise, then the
/// mirror (`s` reversed) and its turns.
pub fn lay(piece: &RoadPiece, elem: usize, tile: (i32, i32), rot: u8) -> Laid {
    let odd = rot & 1 != 0;
    let (mut w, mut h) = (f32::from(piece.width) / 2.0, f32::from(piece.height) / 2.0);
    let (cx, cy) = piece.center[usize::from(rot & 7)];
    let (mut ox, mut oy) = (cx / 2.0, cy / 2.0);
    if odd {
        std::mem::swap(&mut w, &mut h);
        std::mem::swap(&mut ox, &mut oy);
    }
    let x0 = half_grid(tile.0 as f32 - ox);
    let y0 = half_grid(tile.1 as f32 - oy);
    let (u0, v0) = (
        f32::from(piece.offset.0) / 16.0,
        f32::from(piece.offset.1) / 16.0,
    );
    let (du, dv) = (
        f32::from(piece.width) / 16.0,
        f32::from(piece.height) / 16.0,
    );
    let uv = |x: f32, y: f32| {
        let (s, t) = st(rot, x, y);
        [u0 + s * du, v0 + t * dv]
    };
    Laid {
        tile,
        elem,
        rot,
        rect: [x0, y0, x0 + w, y0 + h],
        uv: [uv(0.0, 0.0), uv(1.0, 0.0), uv(0.0, 1.0), uv(1.0, 1.0)],
    }
}

/// The orientation's table: the texture's fraction `(s, t)` at the box's
/// fraction `(x, y)`.
pub fn st(rot: u8, x: f32, y: f32) -> (f32, f32) {
    match rot & 7 {
        0 => (x, y),
        1 => (y, 1.0 - x),
        2 => (1.0 - x, 1.0 - y),
        3 => (1.0 - y, x),
        4 => (1.0 - x, y),
        5 => (1.0 - y, 1.0 - x),
        6 => (x, 1.0 - y),
        _ => (y, x),
    }
}

/// **`RoadsOut::build_render_data`'s age bucket**: the owner's ages gained,
/// 0–2 → 0, 3–5 → 1, 6–7 → 2; `None` past 7, where the original lays
/// nothing.
pub fn age_bucket(ages: i32) -> Option<i32> {
    match ages {
        0..=2 => Some(0),
        3..=5 => Some(1),
        6 | 7 => Some(2),
        _ => None,
    }
}

/// **Every piece the frame lays** (§3.19): each record with a reference
/// (`ref_count`, `+0xa`), its candidate matched and laid. `height` is the
/// texture fractal's at a tile, `TerrainData::texture_fractal` asked at
/// `(x mod width, y mod height)` in cells, as `find_map` asks it.
pub fn laid(p: &Pieces, scene: &RoadsScene, height: impl Fn(i32, i32) -> u8) -> Vec<Laid> {
    let cands = candidates(scene);
    scene
        .roads
        .iter()
        .filter(|r| r.refs != 0)
        .map(|r| {
            let (e, rot) = find_map(p, cands[&(r.x, r.y)], height(r.x, r.y));
            lay(&p.pieces[e], e, (r.x, r.y), rot)
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
