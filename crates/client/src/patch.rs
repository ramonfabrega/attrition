//! **The transition patch's choice** (item 4134; `docs/RENDERER.md` §3.8):
//! which alpha a land-0 cell lays over its base, and at which texcoords,
//! from the cells' `(land, land_sub)`, the edge masks and the texture
//! fractal, held to run1559b's packet cell by cell. Nothing here draws.
//!
//! The stages, each diffed on its own against the packet (§3.8):
//!
//! - **the codes**: a cell's three 12-bit codes, one 3-bit group per edge
//!   (top `0xe00`, right `0x1c0`, bottom `0x38`, left `7`), read from the
//!   edge masks at the mask's three fields (block 6, connect 3, fill 0),
//!   the bottom and left groups bit-reversed;
//! - **the checks**: the block code against the four orthogonal
//!   neighbours, the connect code, the fill code, and the diagonal shapes
//!   when nothing blocks; then the open corners closed on the fill code;
//! - **the pick** ([`find_map`]): every alpha's eight orientations that
//!   the three codes admit, and the fractal choosing an alpha among the
//!   admitting ones, then an orientation among its admitted ones;
//! - **the texcoords** ([`texcoords`]): the cell's 5 × 5 grid turned by
//!   the orientation and laid in the alpha's quadrant the fractal names.
//!
//! The float arithmetic the original does for the two picks and the
//! quadrant is done in `sim::num::Single`, bit for bit.
#![cfg_attr(not(test), allow(dead_code))]

use sim::num::single::Single;

/// The groups, top, right, bottom, left, in the neighbour order they face.
const TOP: u32 = 0xe00;
const RIGHT: u32 = 0x1c0;
const BOTTOM: u32 = 0x38;
const LEFT: u32 = 7;

/// A 3-bit group read backwards.
fn rev3(v: u32) -> u32 {
    ((v & 1) << 2) | (v & 2) | ((v >> 2) & 1)
}

/// The map's cells and the tileset's two numbers the land test needs.
pub struct Ground<'a> {
    pub width: i32,
    pub height: i32,
    /// `(land, land_sub)` per cell, row-major (`WData +2`, `+3`).
    pub cells: &'a [(i8, u8)],
    /// The tileset's `COAST_BASE`.
    pub coast_base: u8,
    /// How many `BASE`s `BASELAND` has.
    pub base_subs: u8,
}

impl Ground<'_> {
    pub fn valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn at(&self, x: i32, y: i32) -> (i8, u8) {
        self.cells[(y * self.width + x) as usize]
    }

    fn at_or_zero(&self, x: i32, y: i32) -> (i8, u8) {
        if self.valid(x, y) {
            self.at(x, y)
        } else {
            (0, 0)
        }
    }

    /// The tileset's land test between a cell `a` and a cell `b`: 0 where
    /// the two may not meet, 1 where `b` sits one step above `a` (the
    /// step a patch is laid for), 2 for the other admitted meetings.
    pub fn combo(&self, a: (i8, u8), b: (i8, u8)) -> u8 {
        let ((la, sa), (lb, sb)) = ((a.0, i32::from(a.1)), (b.0, i32::from(b.1)));
        if la == lb {
            return match sb - sa {
                1 => 1,
                d if d.abs() < 2 => 2,
                _ => 0,
            };
        }
        let coast = i32::from(self.coast_base);
        let top_base = i32::from(self.base_subs) - 1;
        match (la, lb) {
            (0, 1) => u8::from(sa == coast),
            (1, 0) => {
                if sb == coast {
                    2
                } else {
                    0
                }
            }
            (_, 2) => 1,
            _ if lb < la => {
                // Only a land-0 `b` is ever asked here; its top base.
                if lb == 0 && sb == top_base && sa == 0 {
                    2
                } else {
                    0
                }
            }
            _ if sb == 0 && la == 0 => u8::from(sa == top_base),
            _ => 0,
        }
    }
}

/// The edge masks: a 9-bit word per cell edge, three 3-bit fields.
/// `x[(h + 1)·y + x]` is the vertical edge on the left of cell `(x, y)`,
/// `y[(w + 1)·x + y]` the horizontal edge on its top.
pub struct Edges {
    pub x: Vec<u32>,
    pub y: Vec<u32>,
}

impl Edges {
    /// A cell's code at one field (shift 6, 3 or 0): the top edge's group
    /// in bits 9–11, the right's in 6–8, the bottom's reversed in 3–5, the
    /// left's reversed in 0–2.
    pub fn code(&self, g: &Ground, cx: i32, cy: i32, shift: u32) -> u32 {
        let (w, h) = (g.width, g.height);
        let ex = |x: i32, y: i32| (self.x[((h + 1) * y + x) as usize] >> shift) & 7;
        let ey = |x: i32, y: i32| (self.y[((w + 1) * x + y) as usize] >> shift) & 7;
        (ey(cx, cy) << 9) | (ex(cx + 1, cy) << 6) | (rev3(ey(cx, cy + 1)) << 3) | rev3(ex(cx, cy))
    }
}

/// The orthogonal neighbours with the group that faces each.
const SIDES: [(i32, i32, u32); 4] = [(0, -1, TOP), (1, 0, RIGHT), (0, 1, BOTTOM), (-1, 0, LEFT)];

/// **Whether `mark_trans_flags` queues the cell** for the pass that lays
/// patches: some neighbour of the eight lies one step below it.
pub fn queued(g: &Ground, x: i32, y: i32) -> bool {
    let own = g.at(x, y);
    (-1..=1).any(|dy| {
        (-1..=1).any(|dx| {
            let (nx, ny) = (x + dx, y + dy);
            (dx, dy) != (0, 0) && g.valid(nx, ny) && g.combo(g.at(nx, ny), own) == 1
        })
    })
}

/// The three codes after the checks and the corner closing: `(block,
/// connect, fill)`, the arguments of [`find_map`].
pub fn checked_codes(g: &Ground, e: &Edges, x: i32, y: i32) -> (u32, u32, u32) {
    let own = g.at(x, y);
    let mut block = e.code(g, x, y, 6);
    let mut connect = e.code(g, x, y, 3);
    let mut fill = e.code(g, x, y, 0);

    // The block code: a full group the neighbour lies one step below is
    // opened; an open group whose neighbour lies one step above is closed.
    for &(dx, dy, grp) in &SIDES {
        let (nx, ny) = (x + dx, y + dy);
        if !g.valid(nx, ny) {
            continue;
        }
        let n = g.at(nx, ny);
        if block & grp == grp {
            if g.combo(own, n) == 1 {
                block &= !grp;
            }
        } else if g.combo(n, own) == 1 {
            block |= grp;
        }
    }
    // Each corner bit drags its neighbour across the corner.
    for (from, to) in [
        (0x800, 1),
        (0x200, 0x100),
        (0x100, 0x200),
        (0x40, 0x20),
        (0x20, 0x40),
        (8, 4),
        (4, 8),
        (1, 0x800),
    ] {
        if block & from != 0 {
            block |= to;
        }
    }

    // The connect code: an empty group facing a neighbour one step below
    // takes the complement of the block code's group (the whole word, as
    // the original's `not` leaves it); a set group facing any other cell
    // is cleared.
    for &(dx, dy, grp) in &SIDES {
        let (nx, ny) = (x + dx, y + dy);
        if !g.valid(nx, ny) {
            continue;
        }
        let n = g.at(nx, ny);
        if connect & grp == 0 {
            if g.combo(n, own) == 1 {
                connect |= !(block & grp);
            }
        } else if n != own {
            connect &= !grp;
        }
    }

    // The fill code: a group facing another cell is cleared, and so is
    // whatever the block code holds of it.
    for &(dx, dy, grp) in &SIDES {
        let (nx, ny) = (x + dx, y + dy);
        if !g.valid(nx, ny) {
            continue;
        }
        if fill & grp != 0 && g.at(nx, ny) != own {
            fill &= !grp;
        }
        fill &= !(block & grp);
    }

    if block == 0 {
        diagonal(g, x, y, &mut block, &mut fill);
    }

    // The corner closing: an empty side takes its middle bit; two empty
    // sides that meet take their corner.
    let any = |f: u32| f | connect | block;
    let top = any(fill) & TOP == 0;
    if top {
        fill |= 0x400;
    }
    let right = any(fill) & RIGHT == 0;
    if right {
        fill |= 0x80;
    }
    let bottom = any(fill) & BOTTOM == 0;
    if bottom {
        fill |= 0x10;
    }
    let left = any(fill) & LEFT == 0;
    if left {
        fill |= 2;
    }
    if top {
        if left {
            fill |= 0x801;
        }
        if right {
            fill |= 0x300;
        }
    }
    if bottom {
        if left {
            fill |= 9;
        }
        if right {
            fill |= 0x30;
        }
    }
    (block, connect, fill)
}

/// The diagonal shapes, asked only when nothing blocks: two orthogonal
/// neighbours that agree with each other and not with the cell (off the
/// map counts as land 0, sub 0) set a corner's block; with none, a
/// diagonal neighbour one step below sets a corner's.
fn diagonal(g: &Ground, x: i32, y: i32, block: &mut u32, fill: &mut u32) {
    let own = g.at(x, y);
    let (n, e, s, w) = (
        g.at_or_zero(x, y - 1),
        g.at_or_zero(x + 1, y),
        g.at_or_zero(x, y + 1),
        g.at_or_zero(x - 1, y),
    );
    let fill0 = *fill;
    let pair = |a: (i8, u8), b: (i8, u8)| a == b && a != own;
    let ne = pair(n, e);
    if ne {
        *block = 0xfc0;
        *fill |= 0xc;
    }
    let nw = pair(n, w);
    if nw {
        *block = 0xe07;
        *fill |= 0x60;
    }
    let se = pair(s, e);
    if se {
        *block = 0x1f8;
        *fill |= 0x801;
    }
    let sw = pair(s, w);
    if sw {
        *block = 0x3f;
        *fill |= 0x300;
    }
    if ne {
        if nw && se && sw {
            *block = 0;
            *fill = fill0;
        }
        return;
    }
    if nw || se || sw {
        return;
    }
    for (dx, dy, b, f) in [
        (-1, -1, 0x801, 0x1fc),
        (1, -1, 0x300, 0x3f),
        (-1, 1, 0xc, 0xfc0),
        (1, 1, 0x60, 0xe07),
    ] {
        let (nx, ny) = (x + dx, y + dy);
        if g.valid(nx, ny) && g.combo(g.at(nx, ny), own) == 1 {
            *block = b;
            *fill |= f;
        }
    }
}

/// An alpha's eight orientations from its file name, a 12-digit binary
/// code (top, right, bottom, left groups, high to low): the code, its
/// three turns, and the four mirror images.
pub fn bitcodes(name: &str) -> Option<[u32; 8]> {
    let stem = name.strip_suffix(".tga").unwrap_or(name);
    if stem.len() != 12 {
        return None;
    }
    let c = u32::from_str_radix(stem, 2).ok()?;
    let (a, b, cc, d) = (c & 7, (c >> 3) & 7, (c >> 6) & 7, (c >> 9) & 7);
    let pack = |p: u32, q: u32, r: u32, s: u32| (p << 9) | (q << 6) | (r << 3) | s;
    let (ra, rb, rc, rd) = (rev3(a), rev3(b), rev3(cc), rev3(d));
    Some([
        c,
        pack(a, d, cc, b),
        pack(b, a, d, cc),
        pack(cc, b, a, d),
        pack(rd, ra, rb, rc),
        pack(rc, rd, ra, rb),
        pack(rb, rc, rd, ra),
        pack(ra, rb, rc, rd),
    ])
}

/// Whether orientation `code` admits the cell's three codes.
fn admits(code: u32, block: u32, connect: u32, fill: u32) -> bool {
    if block & code != 0 || code & fill != fill {
        return false;
    }
    for grp in [LEFT, RIGHT, TOP, BOTTOM] {
        if connect & grp != 0 && code & grp == 0 {
            return false;
        }
    }
    block != 0 || connect != 0 || fill != 0 || code == 0
}

/// `⌊frac · (n / 100)⌋` in single precision, kept below `n`.
fn fractal_index(frac: u8, n: usize) -> usize {
    let nf = Single::from_i32(n as i32);
    let mut p = Single::from_i32(i32::from(frac)).mulss(nf.divss(Single::from_i32(100)));
    if !nf.subss(Single::HALF).gt(p) {
        p = nf.subss(Single::from_i32(1));
    }
    p.to_i32() as usize
}

/// **The pick**: the alpha (its index in `alphas`) and the orientation
/// (0–7) a cell's three codes and its fractal choose, or `None` where no
/// alpha admits them.
pub fn find_map(
    alphas: &[[u32; 8]],
    frac: u8,
    block: u32,
    connect: u32,
    fill: u32,
) -> Option<(usize, u8)> {
    let candidates: Vec<(usize, Vec<u8>)> = alphas
        .iter()
        .enumerate()
        .filter_map(|(k, codes)| {
            let turns: Vec<u8> = (0..8u8)
                .filter(|&r| admits(codes[usize::from(r)], block, connect, fill))
                .collect();
            (!turns.is_empty()).then_some((k, turns))
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let (k, turns) = &candidates[fractal_index(frac, candidates.len())];
    Some((*k, turns[fractal_index(frac, turns.len())]))
}

/// **The texcoords**, in eighths of the alpha, vertex `j·5 + i` (`i`
/// along world x): the cell's grid point `(i/4, j/4)` turned about the
/// centre by the orientation's quarter turns, halved, and moved into the
/// quadrant `⌊frac · 0.04⌋` names (0 top-left, 1 top-right, 2
/// bottom-right, 3 bottom-left of `(u, v)`).
pub fn texcoords(orientation: u8, frac: u8) -> [[i32; 2]; 25] {
    let turns = [2, 3, 0, 1, 2, 1, 0, 3][usize::from(orientation & 7)];
    let k = Single::from_i32(i32::from(frac))
        .mulss(Single::from_bits(0x3d23_d70a))
        .to_i32()
        .min(3);
    let (ou, ov) = (i32::from(k == 1 || k == 2) * 4, i32::from(k > 1) * 4);
    let mut out = [[0; 2]; 25];
    for j in 0..5 {
        for i in 0..5 {
            // The cell's own texcoord, flipped in v, and in u too for the
            // first four orientations; then quarter turns about (2, 2),
            // `(u, v) → (v, 4 − u)` in quarters — the direction the
            // packet's texcoords show (`Transform::rotate_z` and
            // `Point::rotate` are not read).
            let (mut u, mut v) = (if orientation < 4 { 4 - i } else { i }, 4 - j);
            for _ in 0..turns {
                (u, v) = (v, 4 - u);
            }
            out[(j * 5 + i) as usize] = [u + ou, v + ov];
        }
    }
    out
}

/// **The patch's choice on run1559b's packet, from its own edge masks**
/// (item 4134; `docs/RENDERER.md` §3.8): of the land-0 cells, `[the
/// drawn patches whose alpha is ours, of those whose texcoords are ours
/// at all 25 vertices, the emptied patches where ours is none, the
/// unpatched cells where ours is none]`, beside `[drawn, emptied, land-0
/// cells]`. New. The masks are the packet's (`mark_trans_flags` is
/// unported); the three cells that part, (41, 15), (56, 25) and (45, 28),
/// carry alpha 13, the last of three candidates the fractal does not
/// reach on these masks.
#[cfg(test)]
pub const RUN1559B_3279_PATCH_ON_ITS_MASKS: ([u32; 4], [u32; 3]) =
    ([1509, 1509, 113, 1185], [1512, 113, 2810]);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// One `patch` row of `tools/render/patches.py` (items 4132, 4134).
    #[derive(Clone, Debug)]
    pub(super) struct Row {
        pub land: i8,
        pub alpha: u8,
        pub frac: u8,
        pub uv: Vec<[f32; 2]>,
    }

    /// What the packet holds: the patches, the cells, the edge masks and
    /// the alphas' file names, in `patches.py`'s rows.
    pub(super) struct Packet {
        pub patches: BTreeMap<(i32, i32), Row>,
        pub width: i32,
        pub height: i32,
        pub cells: Vec<(i8, u8)>,
        pub edges: Edges,
        pub alpha_names: Vec<String>,
        pub alpha_codes: Vec<[u32; 8]>,
    }

    pub(super) fn packet() -> Option<Packet> {
        let path = std::env::var_os("RON_PATCHES")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-11-item-4134/run1559b-patches.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no packet patches at {}", path.display());
            return None;
        };
        let mut p = Packet {
            patches: BTreeMap::new(),
            width: 0,
            height: 0,
            cells: Vec::new(),
            edges: Edges {
                x: Vec::new(),
                y: Vec::new(),
            },
            alpha_names: Vec::new(),
            alpha_codes: Vec::new(),
        };
        let mut edges = Vec::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            let at = |k: &str| {
                let i = f.iter().position(|x| *x == k).expect("a key");
                f[i + 1]
            };
            let n = |i: usize| f[i].parse::<i32>().expect("a number");
            match f.first().copied() {
                Some("terrain") => {
                    p.width = n(1);
                    p.height = n(2);
                    p.cells = vec![(0, 0); (p.width * p.height) as usize];
                }
                Some("alpha") => {
                    p.alpha_names.push(at("file").to_string());
                    let c = f.iter().position(|x| *x == "codes").expect("codes");
                    let mut codes = [0; 8];
                    for (k, q) in f[c + 1..c + 9].iter().enumerate() {
                        codes[k] = u32::from_str_radix(q, 16).expect("hex");
                    }
                    p.alpha_codes.push(codes);
                }
                Some("patch") => {
                    let uv = f
                        .iter()
                        .position(|x| *x == "uv")
                        .map(|i| {
                            f[i + 1..]
                                .iter()
                                .filter_map(|q| {
                                    let (u, v) = q.split_once(',')?;
                                    Some([u.parse().ok()?, v.parse().ok()?])
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    p.patches.insert(
                        (n(1), n(2)),
                        Row {
                            land: at("land").parse().expect("land"),
                            alpha: at("alpha").parse().expect("alpha"),
                            frac: at("frac").parse().expect("frac"),
                            uv,
                        },
                    );
                }
                Some("edge") => {
                    let hex = |k: &str| u32::from_str_radix(at(k), 16).expect("hex");
                    edges.push((n(1), n(2), hex("x"), hex("y")));
                }
                Some("cell") => {
                    let i = (n(2) * p.width + n(1)) as usize;
                    p.cells[i] = (
                        at("land").parse().expect("land"),
                        at("sub").parse().expect("sub"),
                    );
                }
                _ => {}
            }
        }
        let (w, h) = (p.width, p.height);
        p.edges.x = vec![0; ((w + 1) * (h + 1)) as usize];
        p.edges.y = vec![0; ((w + 1) * (h + 1)) as usize];
        for (x, y, ex, ey) in edges {
            p.edges.x[((h + 1) * y + x) as usize] = ex;
            p.edges.y[((w + 1) * x + y) as usize] = ey;
        }
        Some(p)
    }

    /// The start dump's `World.seed`, which `TerrainData::texture_fractal`
    /// is built from.
    fn world_seed() -> Option<u32> {
        let start = crate::world::gamelog_dir().join(crate::world::START_DUMP);
        let Ok(Some(head)) = rondata::mapgen::start_head(&start) else {
            println!("skipping: no start dump at {}", start.display());
            return None;
        };
        Some(rondata::mapgen::start_grid(&head).seed as u32)
    }

    /// **Stage 0: the texture fractal** (item 4134): ours,
    /// `sim::mapgen::Fractal::init(61, 61, 0, 2, World.seed)`, against the
    /// packet's `frac` on every patched cell.
    #[test]
    fn stage_0_the_fractal_is_the_packet_s() {
        let (Some(p), Some(seed)) = (packet(), world_seed()) else {
            return;
        };
        let ours = sim::mapgen::Fractal::init(p.width + 1, p.height + 1, 0, 2, seed);
        let mut parting = Vec::new();
        for (&(x, y), r) in &p.patches {
            let h = ours.get_height(x, y);
            if h != r.frac {
                parting.push((x, y, h, r.frac));
            }
        }
        println!(
            "stage 0: seed {seed:#x}, {} of {} patched cells agree; first parting (x, y, ours, theirs): {:?}",
            p.patches.len() - parting.len(),
            p.patches.len(),
            &parting[..parting.len().min(12)]
        );
        assert!(parting.is_empty(), "the fractal parts");
    }

    /// The alphas' orientations from their names are the packet's table.
    #[test]
    fn the_orientations_from_the_names_are_the_packet_s() {
        let Some(p) = packet() else { return };
        assert_eq!(p.alpha_names.len(), 14);
        for (name, codes) in p.alpha_names.iter().zip(&p.alpha_codes) {
            assert_eq!(bitcodes(name).as_ref(), Some(codes), "{name}");
        }
    }

    /// A cell that parts: `(x, y, the packet's alpha, ours)`.
    type Parting = (i32, i32, Option<u8>, Option<(usize, u8)>);

    /// The pick and the texcoords scored on every land-0 cell, from the
    /// packet's own masks and cells: (drawn patches whose alpha is ours,
    /// of those whose texcoords are ours at all 25 vertices, emptied
    /// patches where ours is none, unpatched cells where ours is none),
    /// and the cells that part.
    fn score(
        p: &Packet,
        cells: &[(i8, u8)],
        frac: &sim::mapgen::Fractal,
        turn: u8,
    ) -> ([u32; 4], Vec<Parting>) {
        let g = Ground {
            width: p.width,
            height: p.height,
            cells,
            coast_base: 3,
            base_subs: 4,
        };
        let alphas: Vec<[u32; 8]> = p
            .alpha_names
            .iter()
            .map(|n| bitcodes(n).expect("a code"))
            .collect();
        let (mut n, mut parting) = ([0u32; 4], Vec::new());
        for y in 0..p.height {
            for x in 0..p.width {
                if p.cells[(y * p.width + x) as usize].0 != 0 {
                    continue;
                }
                let h = frac.get_height(x, y);
                let ours = queued(&g, x, y)
                    .then(|| {
                        let (b, c, f) = checked_codes(&g, &p.edges, x, y);
                        find_map(&alphas, h, b, c, f)
                    })
                    .flatten();
                let theirs = p.patches.get(&(x, y));
                let agrees = match (theirs, ours) {
                    (None, None) => {
                        n[3] += 1;
                        true
                    }
                    (Some(r), None) if r.uv.is_empty() => {
                        n[2] += 1;
                        true
                    }
                    (Some(r), Some((k, o))) if !r.uv.is_empty() && usize::from(r.alpha) == k => {
                        n[0] += 1;
                        let tc = texcoords(o ^ turn, h);
                        if tc.iter().zip(&r.uv).all(|(a, b)| {
                            [0, 1]
                                .iter()
                                .all(|&c| (a[c] as f32 / 8.0 - b[c]).abs() < 1e-4)
                        }) {
                            n[1] += 1;
                        }
                        true
                    }
                    _ => false,
                };
                if !agrees {
                    parting.push((x, y, theirs.map(|r| r.alpha), ours));
                }
            }
        }
        (n, parting)
    }

    /// **Stages 2 and 3, on the packet's own masks**
    /// ([`RUN1559B_3279_PATCH_ON_ITS_MASKS`]), each perturbed once.
    #[test]
    fn stages_2_and_3_the_pick_and_the_texcoords_on_the_packet_s_masks() {
        let (Some(p), Some(seed)) = (packet(), world_seed()) else {
            return;
        };
        let frac = sim::mapgen::Fractal::init(p.width + 1, p.height + 1, 0, 2, seed);
        let drawn = p
            .patches
            .iter()
            .filter(|(_, r)| r.land == 0 && !r.uv.is_empty())
            .count() as u32;
        let empty = p
            .patches
            .iter()
            .filter(|(_, r)| r.land == 0 && r.uv.is_empty())
            .count() as u32;
        let land0 = p.cells.iter().filter(|c| c.0 == 0).count() as u32;
        let (n, parting) = score(&p, &p.cells, &frac, 0);
        println!(
            "stages 2 and 3 on the packet's masks: {n:?} of [{drawn}, {drawn}, {empty}, {}]",
            land0 - drawn - empty
        );
        println!("parting (x, y, theirs, ours): {parting:?}");
        let pin = RUN1559B_3279_PATCH_ON_ITS_MASKS;
        assert_eq!(
            (n, [drawn, empty, land0]),
            pin,
            "RUN1559B_3279_PATCH_ON_ITS_MASKS"
        );

        // Perturbed once each: the fractal's seed, one neighbour's
        // land_sub, the orientation's mirror.
        let other = sim::mapgen::Fractal::init(p.width + 1, p.height + 1, 0, 2, seed + 1);
        let (m, _) = score(&p, &p.cells, &other, 0);
        println!("the seed one on: {m:?}");
        assert!(
            m[0] < n[0] && m[1] < n[1],
            "the pick and the texcoords follow the fractal"
        );
        let mut cells = p.cells.clone();
        let (&(x, y), _) = p
            .patches
            .iter()
            .find(|(_, r)| r.land == 0 && !r.uv.is_empty())
            .expect("a drawn patch");
        cells[(y * p.width + x) as usize].1 = 0;
        let (m, _) = score(&p, &cells, &frac, 0);
        println!("one land_sub one down: {m:?}");
        assert!(m != n, "the pick follows the cells");
        let (m, _) = score(&p, &p.cells, &frac, 4);
        println!("the orientation mirrored: {m:?}");
        assert!(m[1] < n[1], "the texcoords follow the orientation");
    }
}
