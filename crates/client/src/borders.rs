//! **The national borders, drawn** (item 4306, `docs/RENDERER.md` §3.29).
//!
//! The edges are `Border::sort_regular@00880960`'s: every cell compares its
//! owner with its north and east neighbours (`dx`, `dy` and the corner
//! offsets of `.rdata`), and an edge between two owners that differ,
//! neither cell ocean (`WorldData::is_ocean`'s rule), is a border's. The
//! edges chain into polylines of cell corners
//! (`Border::walk_regular_strips@0087fbb0`), a straight run merged into
//! its two ends (`Border::generate_strip@0087f8c0`).

use sim::Player;
use sim::world::Owner;

/// One edge between owners: its two corners, the owner on its left and
/// the owner on its right.
pub type Edge = ((i32, i32), (i32, i32), Owner, Owner);

/// A corner-to-corner run.
type Run = ((i32, i32), (i32, i32));

/// One border's polyline: cell corners, the owner on its left (`a`, the
/// side a +90° turn of the heading faces in x-right, y-down cells) and on
/// its right (`b`), and whether it closes on itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    pub corners: Vec<(i32, i32)>,
    pub a: Owner,
    pub b: Owner,
    pub closed: bool,
}

/// **The edges between owners**, as `Border::sort_regular` adds them: for
/// each cell not ocean, its top edge against the cell north of it and its
/// right edge against the cell east of it, where both are on the map, the
/// neighbour is not ocean and the owners differ. Each edge runs from the
/// corner `k` to `k + 1` of `corner_xoffset`/`corner_yoffset`
/// (`(0,0) (1,0) (1,1) (0,1)`), so the cell's own owner is on the edge's
/// right in x-right, y-down cells, and is returned second.
pub fn edges(width: i32, height: i32, owner: &[Owner], ocean: &[bool]) -> Vec<Edge> {
    let at = |x: i32, y: i32| (y * width + x) as usize;
    let mut out = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if ocean[at(x, y)] {
                continue;
            }
            let me = owner[at(x, y)];
            // `dx`/`dy` 0 and 1: north, then east.
            for (k, (nx, ny)) in [(x, y - 1), (x + 1, y)].into_iter().enumerate() {
                if nx < 0 || ny < 0 || nx >= width || ny >= height || ocean[at(nx, ny)] {
                    continue;
                }
                let them = owner[at(nx, ny)];
                if them == me {
                    continue;
                }
                const CX: [i32; 4] = [0, 1, 1, 0];
                const CY: [i32; 4] = [0, 0, 1, 1];
                let p0 = (x + CX[k], y + CY[k]);
                let p1 = (x + CX[k + 1], y + CY[k + 1]);
                out.push((p0, p1, them, me));
            }
        }
    }
    out
}

/// A total order on owners, to key a pair of them one way round.
fn rank(o: Owner) -> i32 {
    match o {
        Owner::None => -2,
        Owner::Ambiguous => -1,
        Owner::Player(p) => i32::from(p),
    }
}

/// **The polylines**: each pair of owners' edges, turned so the lower
/// [`rank`] is on the left, chained corner to corner — open runs from the
/// corners no edge of theirs ends at, then the closed loops — with every
/// corner a straight run passes through dropped.
pub fn chains(edges: &[Edge]) -> Vec<Chain> {
    use std::collections::BTreeMap;
    let mut by_pair: BTreeMap<(i32, i32), (Owner, Owner, Vec<Run>)> = BTreeMap::new();
    for &(p0, p1, a, b) in edges {
        let (p0, p1, a, b) = if rank(a) <= rank(b) {
            (p0, p1, a, b)
        } else {
            (p1, p0, b, a)
        };
        by_pair
            .entry((rank(a), rank(b)))
            .or_insert((a, b, Vec::new()))
            .2
            .push((p0, p1));
    }
    let mut out = Vec::new();
    for (_, (a, b, list)) in by_pair {
        let mut from: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
        let mut ends: BTreeMap<(i32, i32), usize> = BTreeMap::new();
        for (i, &(p0, p1)) in list.iter().enumerate() {
            from.entry(p0).or_default().push(i);
            *ends.entry(p1).or_default() += 1;
        }
        let mut used = vec![false; list.len()];
        let walk = |start: usize, used: &mut Vec<bool>| {
            let mut pts = vec![list[start].0];
            let mut i = start;
            loop {
                used[i] = true;
                let p1 = list[i].1;
                pts.push(p1);
                match from
                    .get(&p1)
                    .and_then(|v| v.iter().copied().find(|&j| !used[j]))
                {
                    Some(j) => i = j,
                    None => break,
                }
            }
            pts
        };
        let starts: Vec<usize> = (0..list.len())
            .filter(|&i| !ends.contains_key(&list[i].0))
            .collect();
        for i in starts {
            if !used[i] {
                let pts = walk(i, &mut used);
                out.push(Chain {
                    corners: straight(pts, false),
                    a,
                    b,
                    closed: false,
                });
            }
        }
        for i in 0..list.len() {
            if !used[i] {
                let pts = walk(i, &mut used);
                let closed = pts.first() == pts.last();
                out.push(Chain {
                    corners: straight(pts, closed),
                    a,
                    b,
                    closed,
                });
            }
        }
    }
    out
}

/// Drop each corner a straight run passes through; a closed loop's last
/// corner (its first again) is dropped too.
fn straight(pts: Vec<(i32, i32)>, closed: bool) -> Vec<(i32, i32)> {
    let mut pts = pts;
    if closed {
        pts.pop();
    }
    let n = pts.len();
    if n < 3 {
        return pts;
    }
    let dir = |p: (i32, i32), q: (i32, i32)| ((q.0 - p.0).signum(), (q.1 - p.1).signum());
    (0..n)
        .filter(|&i| {
            let (prev, next) = if closed {
                ((i + n - 1) % n, (i + 1) % n)
            } else if i == 0 || i == n - 1 {
                return true;
            } else {
                (i - 1, i + 1)
            };
            dir(pts[prev], pts[i]) != dir(pts[i], pts[next])
        })
        .map(|i| pts[i])
        .collect()
}

/// **`TEAM_COLOR_BORDER_*`** (`.data`, `TeamColors::init@008c3100` lays
/// them on `team_colors.list[0..8].border`): red, blue, pink, green,
/// yellow, cyan, white, orange — the bytes of the executable; list 0 and 1
/// agree with run1559b's packet. A slot past the eight is `WHITE`, as list
/// 8 is.
pub const BORDER_COLOURS: [[u8; 3]; 8] = [
    [255, 0, 0],
    [0, 0, 255],
    [188, 0, 250],
    [0, 255, 0],
    [255, 255, 0],
    [0, 255, 255],
    [255, 255, 255],
    [255, 128, 0],
];

/// `BorderSpline::border_alpha` and `ocean_end_alpha`
/// (`BorderSpline::BorderSpline@008830b0`): a vertex's alpha, and the
/// faded one `BorderSplineRender::init_border_render@00882b90` gives an
/// end at the water or the map's edge and a vertex over a mountain tile.
pub const BORDER_ALPHA: u8 = 64;
pub const FADED_ALPHA: u8 = 31;

/// Half the band, in world units: `breadth` 0.25 of a tile's 192
/// (`Spline::make_2d_strip@00912fe0`).
pub const HALF_BAND: f32 = 48.0;

/// **The curve** (`Spline::calc_spline@00912f00`,
/// `generate_bspline@00911820`, `clamp_knots@00910be0`): a B-spline of
/// degree `min(3, n − 1)` on clamped uniform knots, sampled
/// `min(256, ⌊3.5 n⌋) + 1` times evenly in its parameter, so the first and
/// last samples are the first and last control points. Points in any unit.
pub fn bspline(ctrl: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let n = ctrl.len();
    if n < 2 {
        return ctrl.to_vec();
    }
    let p = 3.min(n - 1);
    let span = (n - p) as f32;
    let mut knots = vec![0.0f32; p + 1];
    knots.extend((1..n - p).map(|k| k as f32));
    knots.extend(std::iter::repeat_n(span, p + 1));
    let depth = 256.min((3.5 * n as f32) as usize);
    (0..=depth)
        .map(|k| {
            if k == depth {
                return ctrl[n - 1];
            }
            let t = span * k as f32 / depth as f32;
            // Cox–de Boor, degree 0 up.
            let mut b: Vec<f32> = (0..n + p)
                .map(|i| f32::from(u8::from(knots[i] <= t && t < knots[i + 1])))
                .collect();
            for d in 1..=p {
                for i in 0..n + p - d {
                    let l = knots[i + d] - knots[i];
                    let r = knots[i + d + 1] - knots[i + 1];
                    let a = if l > 0.0 {
                        (t - knots[i]) / l * b[i]
                    } else {
                        0.0
                    };
                    let c = if r > 0.0 {
                        (knots[i + d + 1] - t) / r * b[i + 1]
                    } else {
                        0.0
                    };
                    b[i] = a + c;
                }
            }
            let mut q = [0.0f32; 2];
            for (i, c) in ctrl.iter().enumerate() {
                q[0] += b[i] * c[0];
                q[1] += b[i] * c[1];
            }
            q
        })
        .collect()
}

/// **Each sample's normal** (`Spline::build_normals@00911f60`, on flat
/// ground): a segment's `(d.y, −d.x) / |d|`, a vertex's the normalised sum
/// of its two segments', an open end its one segment's; a closed curve's
/// seam averages its first and last segments.
pub fn normals(pts: &[[f32; 2]], closed: bool) -> Vec<[f32; 2]> {
    let unit = |v: [f32; 2]| {
        let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if l > 0.0 {
            [v[0] / l, v[1] / l]
        } else {
            [0.0, 0.0]
        }
    };
    let seg: Vec<[f32; 2]> = pts
        .windows(2)
        .map(|w| unit([w[1][1] - w[0][1], -(w[1][0] - w[0][0])]))
        .collect();
    let n = pts.len();
    if seg.is_empty() {
        return vec![[0.0, 0.0]; n];
    }
    (0..n)
        .map(|i| {
            let (a, b) = match (i.checked_sub(1).map(|j| seg[j]), seg.get(i).copied()) {
                (Some(a), Some(b)) => (a, b),
                (None, Some(b)) if closed => (seg[seg.len() - 1], b),
                (Some(a), None) if closed => (a, seg[0]),
                (Some(a), None) => (a, a),
                (None, Some(b)) => (b, b),
                (None, None) => unreachable!("a segment exists"),
            };
            unit([a[0] + b[0], a[1] + b[1]])
        })
        .collect()
}

/// **One band's two halves** for a chain, in world units: the curve
/// through its corners (`× 768`, a closed chain's first corner appended),
/// and for each side a player holds, the quads from the curve out to
/// [`HALF_BAND`] along the normal on that side — the `+N` side the chain's
/// `a`, the `−N` side its `b` — as `(owner, triangles of [x, y])` with each
/// vertex's alpha from `alpha(x, y, end)`, `end` true on the curve's first
/// and last samples.
pub type Half = (Player, Vec<([f32; 2], u8)>);
pub fn band(chain: &Chain, alpha: &dyn Fn(f32, f32, bool) -> u8) -> Vec<Half> {
    let cell = sim::world::UNITS_PER_CELL as f32;
    let mut ctrl: Vec<[f32; 2]> = chain
        .corners
        .iter()
        .map(|&(x, y)| [x as f32 * cell, y as f32 * cell])
        .collect();
    if chain.closed && !ctrl.is_empty() {
        ctrl.push(ctrl[0]);
    }
    let (pts, nor) = dense(&bspline(&ctrl), chain.closed);
    let last = pts.len().saturating_sub(1);
    // Each owner's side is the one its own cells lie on: `+N` for `a`.
    let sides = [(chain.a, 1.0f32), (chain.b, -1.0)];
    let lone = match (player(chain.a), player(chain.b)) {
        (Some(_), None) => Some(sides[0]),
        (None, Some(_)) => Some(sides[1]),
        _ => None,
    };
    let mut out = Vec::new();
    // A level across the band: `k` half-bands along the side's normal and
    // the texture's alpha there, of 255.
    let strip = |sign: f32, levels: &[(f32, u16)]| {
        let v = |i: usize, (k, f): (f32, u16)| {
            let q = [
                pts[i][0] + sign * k * HALF_BAND * nor[i][0],
                pts[i][1] + sign * k * HALF_BAND * nor[i][1],
            ];
            let end = !chain.closed && (i == 0 || i == last);
            (q, (u16::from(alpha(q[0], q[1], end)) * f / 255) as u8)
        };
        let mut tris = Vec::new();
        for i in 0..last {
            for w in levels.windows(2) {
                let (a0, a1, b0, b1) = (v(i, w[0]), v(i, w[1]), v(i + 1, w[0]), v(i + 1, w[1]));
                tris.extend([a0, a1, b1, a0, b1, b0]);
            }
        }
        tris
    };
    if let Some((owner, sign)) = lone {
        let who = player(owner).expect("the lone side is a player's");
        out.push((who, strip(sign, &LONE_PROFILE)));
        return out;
    }
    for (owner, sign) in sides {
        let Some(who) = player(owner) else { continue };
        out.push((who, strip(sign, &[(0.0, 255), (1.0, 255)])));
    }
    out
}

/// **A border with one owner, across its width** (item 4358,
/// `docs/RENDERER.md` §3.29, "The settled band"): `(k, alpha of 255)` from
/// the owner's edge, `k` half-bands on its own side, to the far edge.
/// `BorderSplineRender::map_texture@00882990` spans the band's width over
/// two rows of `Borders::make_texture@00882530`'s 16 × 16 texture, and
/// run1665's packet holds `v` 4 texels at the owner's edge and 2 at the
/// far one, `u` in the owner's two columns: the owner's edge between two
/// opaque rows, then an opaque row's centre a quarter across, a row
/// transparent in the owner's own columns three quarters across, and the
/// far edge between it and an opaque row. Linear between, as the card
/// filters it: the photograph's far-edge rim says so.
pub const LONE_PROFILE: [(f32, u16); 4] = [(1.0, 255), (0.5, 255), (-0.5, 0), (-1.0, 128)];

/// **The samples and their normals, cut to [`STEP`]**: each segment
/// between two samples split evenly, position and normal interpolated as
/// they stand (the normal not normalised again), so the band's two edges
/// are the original's straight lines; only the heights between follow the
/// ground (`BORDER_LIFT`'s reason, `client::world`).
fn dense(pts: &[[f32; 2]], closed: bool) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let nor = normals(pts, closed);
    let mut out = (Vec::new(), Vec::new());
    for i in 0..pts.len() {
        if i > 0 {
            let (a, b) = (pts[i - 1], pts[i]);
            let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
            let k = (len / STEP).ceil().max(1.0) as usize;
            for j in 1..k {
                let f = j as f32 / k as f32;
                let mix =
                    |u: [f32; 2], v: [f32; 2]| [u[0] + f * (v[0] - u[0]), u[1] + f * (v[1] - u[1])];
                out.0.push(mix(a, b));
                out.1.push(mix(nor[i - 1], nor[i]));
            }
        }
        out.0.push(pts[i]);
        out.1.push(nor[i]);
    }
    out
}

/// The longest cut of [`dense`], in world units: half a tile.
pub const STEP: f32 = 96.0;

/// The player an owner names, if any.
pub fn player(o: Owner) -> Option<Player> {
    match o {
        Owner::Player(p) => Some(p),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ground::tests::{PixelTiles, Probe, ground_why, probe_at, run1564, run1665};
    use std::path::PathBuf;

    /// **A border's pixel, by colour** (item 4306): the band is a team's
    /// border colour at `BorderSpline::border_alpha` 64 over the ground,
    /// so it reads bluer than the mean of red and green while red keeps
    /// pace with green — the lake fails the second, sand and grass the
    /// first. run1564's band reads (154, 148, 169) over sand (192, 181,
    /// 117).
    pub(crate) fn band(rgb: [u8; 3]) -> bool {
        let [r, g, b] = rgb.map(i32::from);
        b > (r + g) / 2 && r + 2 >= g && r > 110
    }

    /// **Where a border can be drawn**: each pixel whose cell, or one of
    /// its eight neighbours, is held by another owner than it, on the
    /// simulation's own grid (`Sim::observe_territory`), as
    /// `Border::sort_regular@00880960` adds an edge only between two
    /// cells whose owners differ.
    pub(crate) fn near_an_edge(p: &mut Probe, tiles: &PixelTiles) -> Vec<bool> {
        let t = p.world.runner.territory();
        let at = |x: i32, y: i32| {
            (x >= 0 && y >= 0 && x < t.width && y < t.height)
                .then(|| t.owner[(y * t.width + x) as usize])
        };
        tiles
            .iter()
            .map(|q| {
                let Some((tx, ty)) = *q else { return false };
                let (cx, cy) = (tx.div_euclid(4), ty.div_euclid(4));
                let Some(me) = at(cx, cy) else { return false };
                (-1..=1).any(|dy| (-1..=1).any(|dx| at(cx + dx, cy + dy).is_some_and(|o| o != me)))
            })
            .collect()
    }

    fn still(p: &mut Probe, name: &str) -> Vec<u8> {
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let s = crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = PathBuf::from(dir).join(format!("still-{name}.bmp"));
            std::fs::write(&out, crate::gpu::bmp(&s, size)).expect("the still writes");
        }
        s
    }

    /// The row on one still: `(both, theirs, ours)` — the pixels near an
    /// edge the photograph and the still both call a border's, the
    /// photograph's alone counted, and the still's alone counted.
    fn row(p: &Probe, hood: &[bool], ours: &[u8]) -> [usize; 3] {
        let px = |rgba: &[u8], i: usize| [rgba[i * 4], rgba[i * 4 + 1], rgba[i * 4 + 2]];
        let mut out = [0; 3];
        for (i, _) in hood.iter().enumerate().filter(|(_, h)| **h) {
            let (a, b) = (band(px(&p.photo.rgba, i)), band(px(ours, i)));
            out[0] += usize::from(a && b);
            out[1] += usize::from(a);
            out[2] += usize::from(b);
        }
        out
    }

    /// **`RUN1564_3279_BORDER`** (item 4306): on run1564's photograph,
    /// of the pixels within a cell of an owner's edge, `(both, theirs,
    /// ours)` as [`row`] counts them — drawn before (no border), drawn, and
    /// the made-to-fail (the owner grid read one cell east). Minted at the
    /// item's first act at `[0, 1440, 0]` (1,433 of the 1,440 east of x
    /// 1800, the photographed band). Drawn: 225 of 1,440, 615 of ours.
    /// With each half on the other side of the curve, 81 and 643
    /// (`docs/RENDERER.md` §3.29).
    /// Item 4312: 225 → 222 of 1,440, 615 → 613 of ours (a named fall from 225 and 615): the `Oil` good's billboard drawn under the water (item 4312, `docs/RENDERER.md` §3.24) moves the frame's mean, so its white.
    /// Item 4319: 222 → 225 of 1,440, 613 → 615 of ours: the land oil's `rough` and `oil` underlays drawn (item 4319, `docs/RENDERER.md` §3.24) moves the frame's mean, so its white.
    /// Item 4310: 225 → 226 of 1,440, 615 → 609 of ours: the faded land blended over black in HDR (item 4310, `docs/RENDERER.md` §3.27, "The faded land in HDR") moves the frame's mean, so its white.
    /// Item 4338: 226 → 224 (a named fall of two), 609 → 605: the surf's lists blended on the HDR target under the card's point sampler (item 4338, `docs/RENDERER.md` §3.30).
    /// Item 4344: 224 → 222 (a named fall of two), 605 → 566 of ours: the objects and the patches after the water drawn before it (item 4344, `docs/RENDERER.md` §3.30, "The fish under the water"), so the water blends over the band on its cells, as `Borders::process_all` comes before `render_water`.
    /// Item 4345: 222 holds, 566 → 567 of ours: the herd lit (item 4345, `docs/RENDERER.md` §3.30, "The fish's light") moves the frame's mean, so its white.
    /// Item 4348: 222 holds, 567 → 568 of ours: the herd turned to the dump's angle (item 4348, `docs/RENDERER.md` §3.30, "Where the herd stands") moves the frame's mean, so its white.
    /// Item 4358: 222 → 43 (a named fall), 568 → 384 of ours: this photograph's band is a `BorderMorphRender` (item 4356), and the far-side half was fit to it; the settled band (run1665, [`RUN1665_3279_BORDER`]) lies on the owner's side across the whole width, [`super::LONE_PROFILE`].
    pub const RUN1564_3279_BORDER: [[usize; 3]; 3] = [[0, 1440, 0], [43, 1440, 384], [0, 1440, 0]];

    #[test]
    fn the_border_band_on_run1564() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let (_, tiles, _) = ground_why(&mut p);
        let hood = near_an_edge(&mut p, &tiles);
        let mut got = [[0; 3]; 3];
        for (k, (on, shift, what)) in [
            (false, (0, 0), "before"),
            (true, (0, 0), "drawn"),
            (true, (1, 0), "moved"),
        ]
        .into_iter()
        .enumerate()
        {
            p.world.borders = on;
            p.world.borders_shift = shift;
            let s = still(&mut p, &format!("border-{what}"));
            got[k] = row(&p, &hood, &s);
        }
        println!("RUN1564_3279_BORDER: {got:?}");
        assert_eq!(got, RUN1564_3279_BORDER);
    }

    /// **`RUN1665_3279_BORDER`** (item 4358): the same row on run1665's
    /// photograph, run1564's take with fast-forward ended 100 frames
    /// before the pause, so the border is the settled regular strip its
    /// packet holds (`normal_border_renders` 2, no morph, `graphic_tick`
    /// 253; `docs/RENDERER.md` §3.29, "The settled band"). Minted at
    /// `[0, 262, 0]`; drawn as 4306 left it (one flat half on the far
    /// side) 7 of 262, 568 of ours; with [`super::LONE_PROFILE`] on the
    /// owner's own side, 59 of 262, 384 of ours. The made-to-fail is the
    /// owner grid one cell east.
    pub const RUN1665_3279_BORDER: [[usize; 3]; 3] = [[0, 262, 0], [59, 262, 384], [0, 262, 0]];

    #[test]
    fn the_settled_border_band_on_run1665() {
        let Some(mut p) = probe_at(run1665()) else {
            return;
        };
        let (_, tiles, _) = ground_why(&mut p);
        let hood = near_an_edge(&mut p, &tiles);
        let mut got = [[0; 3]; 3];
        for (k, (on, shift, what)) in [
            (false, (0, 0), "before"),
            (true, (0, 0), "drawn"),
            (true, (1, 0), "moved"),
        ]
        .into_iter()
        .enumerate()
        {
            p.world.borders = on;
            p.world.borders_shift = shift;
            let s = still(&mut p, &format!("settled-{what}"));
            got[k] = row(&p, &hood, &s);
        }
        println!("RUN1665_3279_BORDER: {got:?}");
        assert_eq!(got, RUN1665_3279_BORDER);
    }
}
