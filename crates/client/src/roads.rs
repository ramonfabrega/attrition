//! **The roads** (item 4199, `docs/RENDERER.md` §3.19): draw 91,
//! `modern_road_pieces.tga` under `RoadsOut`'s state, one piece of
//! `Data/roads.xml` laid on each road tile the read door reports
//! (`sim::observe::RoadsScene`).
//!
//! Three steps, each a function here:
//!
//! 1. **The candidate's flags** ([`candidates`]): the record's four bytes
//!    as the simulation's mesh leaves them, the connections on top, then
//!    the yellow lines, the white lines and the intersections the texture
//!    passes mark (`RoadsOut::mark_white_lines@00892c10`,
//!    `mark_intersections@00892390`; item 4690, `docs/RENDERER.md` §6.9).
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

/// **The candidate's flags on every road tile** (§3.19, §6.9): the
/// record's dword as the simulation's mesh leaves it. The connections the
/// door carries on top ([`RoadTile::dirs`]); then the yellow and white
/// lines the line passes mark, history and all ([`RoadTile::lines`]:
/// `RoadsOut::mark_white_lines@00892c10`, `mark_yellow_lines@00892550`,
/// `fixup_lines@00891210`, `delete_straglers@008910b0` and
/// `leech_codes@00890da0`'s line half; item 4687, §6.8); and the
/// intersections ([`RoadTile::inter`], item 4677, §6.5).
///
/// Item 4690: until then the client marked its own white lines once on the
/// mesh as it stood, with the rings round a joined corner added, and no
/// yellow. The mesh's own bytes carry the yellow the crossroads' pieces
/// name, and the rings from the history that holds them.
pub fn candidates(scene: &RoadsScene) -> BTreeMap<(i32, i32), u32> {
    scene.roads.iter().map(|r| ((r.x, r.y), word(r))).collect()
}

/// One record's candidate: [`RoadTile::dirs`] in the top byte, the line
/// bytes in place (`0x00ff_ff00`), [`RoadTile::inter`] in the low byte.
pub fn word(r: &RoadTile) -> u32 {
    u32::from(r.dirs) << 24 | r.lines & 0x00ff_ff00 | u32::from(r.inter)
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
            // Item 4682: the piece is picked on the intersections `find_piece`
            // last read (`RoadTile::picked`), not the ones standing now.
            let cand = cands[&(r.x, r.y)] & !0xff | u32::from(r.picked);
            let (e, rot) = find_map(p, cand, height(r.x, r.y));
            lay(&p.pieces[e], e, (r.x, r.y), rot)
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
