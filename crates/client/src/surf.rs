//! The surf (item 4236; `docs/RENDERER.md` §3.26).
//!
//! The waves and breakers `Surf::init@008a0cf0` lays along the shore at map
//! load, each drawn by `SurfWaveSection::draw@0089e360` from its section
//! models (`render::mot`) at its timing's frame. Their phase and their
//! shape are draws on **`internal_random`** (`0x00eb697c`), the renderer's
//! own generator, which the simulation never steps.
//!
//! **Not drawn yet** (item 4236): `Surf::init` is in register with
//! run1564's packet when fed the packet's shore chains; the client's own
//! chains (`TerrainOut::sort_waterline_wcoords@00872f30`) and the draw
//! are the next word, so the binary does not call this module yet.
#![cfg_attr(not(test), allow(dead_code))]

/// `internal_random`'s draws before `Surf::init`'s first, on run1567's
/// trace (the GreatLakes lobby's map, item 4236): 45 under
/// `RiverDoobers::init_class`, 58,081 in `Terrain::init`'s
/// `random_map_pool` (`(4 · width + 1)²`, width 60), 45 under
/// `RiverDoobers::init_class` again, 360 in `TerrainOil::add_rocky_patch`
/// and 713 in `RiverSectionOut::create_doobers`.
pub const DRAWS_BEFORE_SURF_1567: u32 = 59_244;

/// `internal_random`: `Random`'s generator (`Random::get@00a39d70`), a
/// 32-bit LCG that starts at 0 (its first draw on run1567 finds 0) and that no
/// site reseeds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InternalRandom(pub u32);

impl InternalRandom {
    /// One step.
    pub fn step(&mut self) {
        self.0 = self.0.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
    }

    /// The generator after `n` draws from 0.
    pub fn after(n: u32) -> Self {
        let mut r = Self(0);
        for _ in 0..n {
            r.step();
        }
        r
    }

    /// `Random::get(lo, hi)`: `lo` when the two are equal, without a step;
    /// otherwise a step, and the low 16 bits scaled onto `[lo, hi)`.
    pub fn get(&mut self, lo: i32, hi: i32) -> i32 {
        if lo == hi {
            return lo;
        }
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        self.step();
        ((i64::from(self.0 & 0xffff) * i64::from(hi - lo)) >> 16) as i32 + lo
    }
}

/// The binary angle's helpers: a full turn is 2³², and the "fast" readers
/// are 256-entry tables on its top byte (`fast_angle_to_degrees@00a28f70`,
/// `fast_angle_to_cosine@00a28a70`, `fast_angle_to_sine@00a28b20`), each
/// filled once from `angle_to_degrees` of the entry's angle.
pub mod angle {
    /// `angle_to_degrees@00a28e00`: whole degrees, by its own staged
    /// division (90°, 45°, 15°, 5°, then the remainder rounded on 1°).
    pub fn to_degrees(a: u32) -> i32 {
        let quad = a >> 30;
        let mut r = a.wrapping_sub(quad.wrapping_mul(0x4000_0000));
        let half = r >> 29;
        r = r.wrapping_sub(half.wrapping_mul(0x2000_0000));
        let r15 = r % 0x1555_5555;
        let r5 = r15 % 0x0aaa_aaaa;
        ((r5 % 0x038e_38e3 + 0x005b_05b0) / 0x00b6_0b60
            + (r5 / 0x038e_38e3) * 5
            + ((half + quad * 2) * 3 + r15 / 0x0aaa_aaaa + (r / 0x1555_5555) * 2) * 15)
            as i32
    }

    /// `degrees_to_angle@00a28eb0`, whole degrees to the binary angle.
    pub fn from_degrees(d: i32) -> u32 {
        let r90 = d % 90;
        let r45 = r90 % 45;
        let r15 = r45 % 30;
        let r5 = r15 % 15;
        ((((r45 / 30) * 0x071c_71c7 + r5 / 5) * 3) as u32)
            .wrapping_add(((r90 / 45 + (d / 90) * 2) as u32).wrapping_mul(0x2000_0000))
            .wrapping_add(((r15 / 15) as u32).wrapping_mul(0x0aaa_aaaa))
            .wrapping_add((r5 as u32).wrapping_mul(0x00b6_0b60))
    }

    /// The tables' entry for an angle.
    fn entry(a: u32) -> i32 {
        to_degrees(((a >> 24) & 0xff) << 24)
    }

    /// `fast_angle_to_degrees`.
    pub fn fast_degrees(a: u32) -> f32 {
        entry(a) as f32
    }

    /// `fast_angle_to_cosine` and `fast_angle_to_sine`: of the entry's
    /// whole degrees, times the `.rdata` float `0x3c8efa35` (π/180).
    pub fn fast_cos_sin(a: u32) -> (f32, f32) {
        let r = entry(a) as f32 * f32::from_bits(0x3c8e_fa35);
        (r.cos(), r.sin())
    }

    /// `find_angle@0092d130(dx, dy)`: the direction of `(dx, −dy)` as a
    /// binary angle, by an integer arctangent on the smaller over the
    /// larger at 2¹⁴.
    pub fn find(dx: i32, dy: i32) -> u32 {
        let y = dy.wrapping_neg();
        if dx == 0 {
            return if y > 0 { 0 } else { 0x8000_0000 };
        }
        if y == 0 {
            return if dx > 0 { 0x4000_0000 } else { 0xc000_0000 };
        }
        let (ax, ay) = (dx.abs(), y.abs());
        let x_major = ax > ay;
        let ratio = if x_major {
            (ay << 14) / ax
        } else {
            (ax << 14) / ay
        };
        let a = (((0x2800 - (((0x1333 - ratio).abs() * 0xb00) >> 14)) * ratio) as u32
            & 0xffff_c000)
            .wrapping_mul(4);
        match (dx > 0, y > 0, x_major) {
            (true, true, false) => a,
            (true, true, true) => 0x4000_0000u32.wrapping_sub(a),
            (true, false, true) => a.wrapping_add(0x4000_0000),
            (true, false, false) => 0x8000_0000u32.wrapping_sub(a),
            (false, true, false) => a.wrapping_neg(),
            (false, true, true) => a.wrapping_add(0xc000_0000),
            (false, false, true) => 0xc000_0000u32.wrapping_sub(a),
            (false, false, false) => a.wrapping_add(0x8000_0000),
        }
    }
}

/// `Spline` as `Surf::init` drives it: a clamped uniform B-spline
/// (`Spline::clear@00914410` then type 3, degree 4, depth 1000) through
/// the chain's points (`Spline::calc_spline@00912f00`,
/// `clamp_knots@00910be0`, `generate_bspline@00911820`,
/// `calc_bspline_basis@009116e0`), its per-vertex normals and length
/// (`build_normals@00911f60`, no flags), and the walk
/// (`begin_walk_dist@00912b00`, `next_at_dist@00912b70`).
#[derive(Clone, Debug, Default)]
pub struct Spline {
    pub verts: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub length: f32,
    curr: f32,
    next: f32,
    scan: i32,
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if l2 == 0.0 || l2 == 1.0 {
        return v;
    }
    let inv = 1.0 / l2.sqrt();
    v.map(|c| c * inv)
}

fn seg_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (f64::from(d[1] * d[1] + d[0] * d[0] + d[2] * d[2])).sqrt() as f32
}

impl Spline {
    /// The curve through `control` at degree 4 and depth 1000.
    pub fn bspline(control: &[[f32; 3]]) -> Spline {
        let n = control.len();
        let mut s = Spline {
            scan: -1,
            curr: -1.0,
            next: -1.0,
            ..Default::default()
        };
        if n == 0 {
            return s;
        }
        let degree = if n <= 4 { n - 1 } else { 4 };
        let depth = 1000usize.max(degree);
        // Clamped uniform knots: degree + 1 zeros, 1 to n − degree, the
        // last repeated to n + degree + 1 in all.
        let mut knots = Vec::with_capacity(n + degree + 1);
        let mut k = 0.0f32;
        for i in 0..=n + degree {
            if i > degree && i <= n {
                k += 1.0;
            }
            knots.push(k);
        }
        let last = knots.len() - 1;
        let mut step = (knots[last] - knots[0]) / depth as f32;
        if step == 0.0 {
            step = 1.0;
        }
        s.verts.push(control[0]);
        let mut t = knots[0] + step;
        let mut span = 0usize;
        while t < knots[last] {
            while span + 1 < knots.len() && !(t <= knots[span + 1] && knots[span] < knots[span + 1])
            {
                span += 1;
            }
            if span > last {
                break;
            }
            let mut p = [0.0f32; 3];
            let lo = span.saturating_sub(degree);
            for (j, cv) in control
                .iter()
                .enumerate()
                .take(span.min(n - 1) + 1)
                .skip(lo)
            {
                let b = basis(&knots, j, degree, f64::from(t)) as f32;
                for c in 0..3 {
                    p[c] += cv[c] * b;
                }
            }
            s.verts.push(p);
            t += step;
        }
        if f64::from(knots[last] - (t - step)) > 0.001 {
            s.verts.push(control[n - 1]);
        }
        s.build_normals();
        s
    }

    fn build_normals(&mut self) {
        let up = [1.0f32, 1.0, f32::from_bits(0x3fdd_b3d7)];
        let mut prev = [0.0f32; 3];
        self.length = 0.0;
        for k in 1..self.verts.len() {
            let (a, b) = (self.verts[k - 1], self.verts[k]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            self.length += seg_len(b, a);
            let n = norm3([
                up[2] * d[1] - up[1] * d[2],
                up[0] * d[2] - up[2] * d[0],
                up[1] * d[0] - up[0] * d[1],
            ]);
            self.normals
                .push(norm3([n[0] + prev[0], n[1] + prev[1], n[2] + prev[2]]));
            prev = n;
        }
        self.normals.push(prev);
    }

    /// `begin_walk_dist(0)`.
    pub fn begin_walk(&mut self) {
        if !self.verts.is_empty() && self.length != 0.0 {
            (self.curr, self.next, self.scan) = (0.0, 0.0, 0);
        } else {
            (self.curr, self.next, self.scan) = (-1.0, -1.0, -1);
        }
    }

    /// `next_at_dist(&pos, &normal, end, dist)`: the point `dist` past the
    /// last along the curve, and the normal there; `false` once the walk
    /// is within `end · dist` of the end or past it. A walk already done
    /// returns `false` and leaves `pos` and `normal` as they were.
    pub fn next_at_dist(
        &mut self,
        pos: &mut [f32; 3],
        normal: &mut [f32; 3],
        end: f32,
        dist: f32,
    ) -> bool {
        if self.curr < 0.0 || self.scan < 0 {
            return false;
        }
        let nv = self.verts.len() as i32;
        let mut scan = self.scan + 1;
        let target = self.next + dist;
        self.next = target;
        let mut seg = -1.0f32;
        while scan < nv && self.curr < target {
            seg = seg_len(self.verts[scan as usize], self.verts[scan as usize - 1]);
            scan += 1;
            self.curr += seg;
        }
        scan -= 1;
        self.scan = scan;
        let i = scan as usize;
        if self.curr <= target || nv <= scan || self.length - target < dist * end {
            *pos = self.verts[i.min(self.verts.len() - 1)];
            *normal = self.normals[i.min(self.normals.len() - 1)];
            if nv <= scan + 1 || self.length <= self.curr {
                self.curr = self.length;
                self.scan = -1;
                return false;
            }
            return true;
        }
        if seg < 0.0 {
            seg = seg_len(self.verts[i], self.verts[i - 1]);
        }
        let f = if seg == 0.0 {
            0.0
        } else {
            (self.curr - target) / seg
        };
        let lerp = |v: &[[f32; 3]]| -> [f32; 3] {
            std::array::from_fn(|c| v[i][c] + (v[i - 1][c] - v[i][c]) * f)
        };
        *pos = lerp(&self.verts);
        *normal = norm3(lerp(&self.normals));
        true
    }
}

/// Cox–de Boor, each term only where its knot span is positive.
fn basis(k: &[f32], i: usize, p: usize, t: f64) -> f64 {
    let kk = |j: usize| f64::from(k[j]);
    if p == 0 {
        return if kk(i) <= t && t < kk(i + 1) {
            1.0
        } else {
            0.0
        };
    }
    let mut v = 0.0;
    if t - kk(i) > 0.0 && kk(i + p) - kk(i) > 0.0 {
        v += (t - kk(i)) / (kk(i + p) - kk(i)) * basis(k, i, p - 1, t);
    }
    if kk(i + p + 1) - t > 0.0 && kk(i + p + 1) - kk(i + 1) > 0.0 {
        v += (kk(i + p + 1) - t) / (kk(i + p + 1) - kk(i + 1)) * basis(k, i + 1, p - 1, t);
    }
    v
}

/// `WaveStruct` (0x2c): one wave or breaker as `Surf::add_wave@0089d5a0`
/// lays it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveStruct {
    /// The tile, `x / 768`.
    pub wx: i32,
    pub wy: i32,
    pub x: f32,
    pub y: f32,
    pub x_axis: [f32; 2],
    pub y_axis: [f32; 2],
    /// 0 a breaker, 1 a large wave: `Surf::wave_shape_index`'s row.
    pub shape: usize,
    /// The `<WAVE>` it draws.
    pub kind: usize,
    pub time: i32,
}

/// What `Surf::init` reads of the world: the width in tiles (`World`
/// +0), the wcoord grid (+0x18, +0x1c; a wcoord is 192 units) and each
/// wcoord's flags (+0x138).
#[derive(Clone, Debug)]
pub struct SurfWorld {
    pub width_tiles: i32,
    pub wcoords: (i32, i32),
    pub flags: Vec<u16>,
}

impl SurfWorld {
    /// The flags at a world point, `None` off the grid.
    fn flags_at(&self, x: f32, y: f32) -> Option<u16> {
        let (xi, yi) = (x as i32, y as i32);
        if xi < 0 || yi < 0 {
            return None;
        }
        let (cx, cy) = ((xi >> 6) / 3, (yi >> 6) / 3);
        if cx >= self.wcoords.0 || cy >= self.wcoords.1 {
            return None;
        }
        Some(self.flags[(cy * self.wcoords.0 + cx) as usize])
    }
}

/// The surf as `Surf::init` lays it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surf {
    pub waves: Vec<WaveStruct>,
    pub breakers: Vec<WaveStruct>,
    pub total: i32,
    pub stall: i32,
}

/// `location_offset` (`0x00c062ac`), on run1564's packet.
pub const LOCATION_OFFSET: i32 = 24;

impl Surf {
    /// `Surf::init@008a0cf0` over the shore's chains (`TerrainOut::
    /// sorted_waterline_tcoords`, each a list of wcoord indices), with
    /// `internal_random` as it stands at map load.
    pub fn init(
        class: &rondata::artdata::SurfClass,
        chains: &[Vec<i32>],
        world: &SurfWorld,
        rng: &mut InternalRandom,
    ) -> Surf {
        let (total, stall) = (class.wave_time, class.stall_time);
        let mut surf = Surf {
            total,
            stall,
            ..Default::default()
        };
        let from: Vec<f32> = class.types.iter().map(|t| t.0).collect();
        let along: Vec<f32> = class.types.iter().map(|t| t.1.max(192.0)).collect();
        let shapes: [Vec<usize>; 2] = std::array::from_fn(|s| {
            (0..class.waves.len())
                .filter(|&w| class.waves[w].shape == s)
                .collect()
        });
        let period = total + stall;
        let add = |surf: &mut Surf,
                   rng: &mut InternalRandom,
                   deg: i32,
                   shape: usize,
                   base: i32,
                   x: f32,
                   y: f32| {
            let list = &shapes[shape];
            if list.is_empty() {
                return;
            }
            let a = angle::from_degrees(deg).wrapping_sub(0x8000_0000);
            let (ax, ay) = angle::fast_cos_sin(a);
            let (bx, by) = angle::fast_cos_sin(a.wrapping_add(0x4000_0000));
            let kind = list[rng.get(0, list.len() as i32) as usize];
            let (xi, yi) = (x as i32, y as i32);
            let time = ((xi + yi) / LOCATION_OFFSET + base) % period - stall;
            let w = WaveStruct {
                wx: (xi >> 8) / 3,
                wy: (yi >> 8) / 3,
                x,
                y,
                x_axis: [ax, ay],
                y_axis: [bx, by],
                shape,
                kind,
                time,
            };
            if shape == 0 {
                surf.breakers.push(w);
            } else {
                surf.waves.push(w);
            }
        };
        let degrees = |n: [f32; 3]| -> i32 {
            angle::fast_degrees(angle::find((n[0] * 1000.0) as i32, (n[1] * 1000.0) as i32)) as i32
        };
        for chain in chains.iter().filter(|c| c.len() >= 16) {
            let w4 = world.width_tiles * 4;
            let control: Vec<[f32; 3]> = chain
                .iter()
                .step_by(4)
                .map(|&t| [(t % w4) as f32 * 192.0, (t / w4) as f32 * 192.0, 0.0])
                .collect();
            let mut spline = Spline::bspline(&control);
            let r = rng.get(-stall, total);
            let (mut pos, mut n) = ([0.0f32; 3], [0.0f32; 3]);
            spline.begin_walk();
            let step = along[1] * 0.5;
            loop {
                let mut more = spline.next_at_dist(&mut pos, &mut n, 0.1, step);
                let (px, py) = (pos[0] + n[0] * from[1], pos[1] + n[1] * from[1]);
                if world.flags_at(px, py).is_none_or(|f| f & 0x800 == 0) {
                    add(&mut surf, rng, degrees(n), 1, r, px, py);
                    more = spline.next_at_dist(&mut pos, &mut n, 0.1, step);
                    let (qx, qy) = (pos[0] + n[0] * from[1], pos[1] + n[1] * from[1]);
                    let base = (period / 2 + r) % period - stall;
                    add(&mut surf, rng, degrees(n), 1, base, qx, qy);
                }
                if !more {
                    break;
                }
            }
            spline.begin_walk();
            loop {
                let d = rng.get((along[0] * 0.5) as i32, along[0] as i32) as f32;
                let more = spline.next_at_dist(&mut pos, &mut n, 0.1, d);
                let (px, py) = (pos[0] + n[0] * from[0], pos[1] + n[1] * from[0]);
                if world.flags_at(px, py).is_some_and(|f| f & 0x30 == 0x20) {
                    let deg = degrees(n);
                    let base = rng.get(-stall, total);
                    add(&mut surf, rng, deg, 0, base, px, py);
                }
                if !more {
                    break;
                }
            }
        }
        surf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// `internal_random` in VA.
    const INTERNAL_RANDOM: u32 = 0x00eb_697c;

    fn index() -> Vec<(u32, String)> {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let Ok(text) = std::fs::read_to_string(home.join("ghidra-projects/decomp/INDEX.tsv"))
        else {
            return vec![];
        };
        let mut v: Vec<(u32, String)> = text
            .lines()
            .filter_map(|l| {
                let mut f = l.split('\t');
                let a = u32::from_str_radix(f.next()?, 16).ok()?;
                Some((a, f.next()?.to_string()))
            })
            .collect();
        v.sort();
        v
    }

    fn function(index: &[(u32, String)], va: u32) -> String {
        let i = index.partition_point(|e| e.0 <= va);
        i.checked_sub(1)
            .map_or_else(|| format!("{va:x}"), |i| index[i].1.clone())
    }

    /// **`internal_random` at `Surf::init`** (item 4236; `docs/RENDERER.md`
    /// §3.26): on run1567's trace (`rontrace.log`, every `Random::get` with
    /// its `Random *` and the word before the step), the generator's first
    /// draw finds it at 0, every draw follows the last by one step, and the
    /// draws before `Surf::init`'s first are 59,244, all from map-load
    /// sites, in this order. So the state `Surf::init` starts from is the
    /// map's, not the session's: nothing in the menu or the lobby draws
    /// it. After setup it is drawn each frame by `Surf::inc_time`, the
    /// river wakes, the ocean floor's clock, the particles and the ammo.
    #[test]
    fn internal_random_at_surf_init_is_the_map_s() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let path = home.join("ron-golden/run1567/map-14/rontrace.log");
        let Ok(Some(trace)) = rondata::trace::Trace::read(&path) else {
            println!("skipping: no trace at {}", path.display());
            return;
        };
        let index = index();
        if index.is_empty() {
            println!("skipping: no decompile index");
            return;
        }
        let ours: Vec<_> = trace
            .draws()
            .iter()
            .filter(|d| d.rng == INTERNAL_RANDOM && matches!(d.kind, 1 | 3))
            .collect();
        assert_eq!(ours[0].seed, 0, "the first draw finds the generator at 0");
        let mut r = InternalRandom(0);
        for d in &ours {
            assert_eq!(d.seed, r.0, "a step between two draws");
            r.step();
        }
        let first = ours
            .iter()
            .position(|d| function(&index, d.site).starts_with("Surf::"))
            .unwrap();
        assert_eq!(first as u32, DRAWS_BEFORE_SURF_1567);
        assert_eq!(
            ours[first].seed,
            InternalRandom::after(DRAWS_BEFORE_SURF_1567).0
        );
        let mut runs: Vec<(String, u32)> = vec![];
        for d in &ours[..first] {
            let caller = format!(
                "{} < {}",
                function(&index, d.site),
                function(&index, d.up[0])
            );
            match runs.last_mut() {
                Some(r) if r.0 == caller => r.1 += 1,
                _ => runs.push((caller, 1)),
            }
        }
        assert_eq!(
            runs,
            [
                ("RiverDooberType::init < RiverDoobers::init_class", 45),
                ("Terrain::init < Setup::build_game", 58_081),
                ("RiverDooberType::init < RiverDoobers::init_class", 45),
                (
                    "TerrainOil::add_rocky_patch < TerrainOil::create_doobers",
                    360
                ),
                ("RiverSectionOut::create_doobers < RiverSection::init", 713),
            ]
            .map(|(s, n)| (s.to_string(), n))
        );
        let mut after: BTreeMap<String, u32> = BTreeMap::new();
        for d in ours.iter().filter(|d| d.frame >= 0) {
            let f = function(&index, d.site);
            *after.entry(f).or_default() += 1;
        }
        let names: Vec<&str> = after.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "AmmoOut::dirt_splash",
                "AmmoOut::explode",
                "Particle::init",
                "RiverSectionOut::update_wake_polys",
                "Surf::inc_time",
                "TerrainOut::modify_ocean_floor_texture",
            ],
            "the generator's consumers after setup, frames 0 to 3293"
        );
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use std::path::PathBuf;

    /// run1564's packet as `surf_init_packet` dumps it: the chains, the
    /// world's grid and flags, and the two arrays `Surf::init` filled.
    pub(crate) struct Packet {
        pub frame: i32,
        pub chains: Vec<Vec<i32>>,
        pub world: SurfWorld,
        pub waves: Vec<WaveStruct>,
        pub breakers: Vec<WaveStruct>,
    }

    pub(crate) fn packet() -> Option<Packet> {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let path = std::env::var_os("RON_SURF_INIT_1564")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                home.join("ron-data/lab-experiments/2026-11-14-item-4236/run1564-surf-init.txt")
            });
        let text = std::fs::read_to_string(&path).ok()?;
        let mut p = Packet {
            frame: 0,
            chains: vec![],
            world: SurfWorld {
                width_tiles: 0,
                wcoords: (0, 0),
                flags: vec![],
            },
            waves: vec![],
            breakers: vec![],
        };
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            let n = |i: usize| f[i].parse::<i32>().unwrap();
            let r = |i: usize| f[i].parse::<f32>().unwrap();
            match f[0] {
                "surf" => {
                    p.frame = n(2);
                    p.world.width_tiles = n(4);
                    p.world.wcoords = (n(6), n(7));
                    assert_eq!((n(9), n(11)), (0, LOCATION_OFFSET), "river deltas, offset");
                }
                "chain" => p.chains.push((2..f.len()).map(n).collect()),
                "flags" => {
                    let hex = f[2].as_bytes();
                    for c in hex.chunks(4) {
                        let s = std::str::from_utf8(c).unwrap();
                        let le = u16::from_str_radix(s, 16).unwrap();
                        p.world.flags.push(le.swap_bytes());
                    }
                }
                "wave" | "breaker" => {
                    let w = WaveStruct {
                        wx: n(1),
                        wy: n(2),
                        x: r(3),
                        y: r(4),
                        x_axis: [r(5), r(6)],
                        y_axis: [r(7), r(8)],
                        shape: n(9) as usize,
                        kind: n(10) as usize,
                        time: n(11),
                    };
                    if f[0] == "wave" {
                        p.waves.push(w);
                    } else {
                        p.breakers.push(w);
                    }
                }
                _ => {}
            }
        }
        Some(p)
    }

    /// **`Surf::init` from the packet's shore** (item 4236;
    /// `docs/RENDERER.md` §3.26): run1564's `sorted_waterline_tcoords`,
    /// wcoord flags and grid fed to [`Surf::init`] with `internal_random`
    /// at [`DRAWS_BEFORE_SURF_1567`], against the packet's
    /// `wave_structs` and `breaker_structs`, the whole record but the
    /// shape's draw at frame 3294 (the session's; §3.26): every wave and
    /// breaker in order, its tile, its place to [`PLACE`] units, its two
    /// axes to 1e-6, its shape, and its time stepped through `inc_time`
    /// to the packet's frame. `Surf::init` takes 1,721 draws, so the
    /// generator leaves it at 60,965, the trace's setup total.
    #[test]
    fn surf_init_lays_the_packet_s_waves() {
        let Some(inst) = install() else {
            println!("skipping: no install");
            return;
        };
        let Some(p) = packet() else {
            println!("skipping: no surf init dump");
            return;
        };
        let class = rondata::artdata::surf_class(&inst).unwrap();
        let mut rng = InternalRandom::after(DRAWS_BEFORE_SURF_1567);
        let ours = Surf::init(&class, &p.chains, &p.world, &mut rng);
        assert_eq!(
            rng,
            InternalRandom::after(60_965),
            "the draws Surf::init takes"
        );
        assert_eq!(
            (ours.waves.len(), ours.breakers.len()),
            (p.waves.len(), p.breakers.len())
        );
        let mut worst = 0.0f32;
        let period = ours.total + ours.stall;
        // [place, tile, axes, shape, time] agreeing, of every record.
        let mut agree = [0usize; 5];
        let mut parted = vec![];
        for (k, (a, b)) in ours
            .waves
            .iter()
            .chain(&ours.breakers)
            .zip(p.waves.iter().chain(&p.breakers))
            .enumerate()
        {
            let d = (a.x - b.x).abs().max((a.y - b.y).abs());
            worst = worst.max(d);
            let axes = a
                .x_axis
                .iter()
                .chain(&a.y_axis)
                .zip(b.x_axis.iter().chain(&b.y_axis))
                .all(|(u, v)| (u - v).abs() < 1e-6);
            let time = (a.time + ours.stall + p.frame).rem_euclid(period) - ours.stall;
            let fields = [
                d <= PLACE,
                (a.wx, a.wy) == (b.wx, b.wy),
                axes,
                a.shape == b.shape,
                time == b.time,
            ];
            for (n, ok) in agree.iter_mut().zip(fields) {
                *n += usize::from(ok);
            }
            if fields.contains(&false) {
                parted.push(k);
                if parted.len() <= 4 {
                    println!(
                        "surf init: record {k} parts: ours {a:?} (time at {} {time}), theirs {b:?}",
                        p.frame
                    );
                }
            }
        }
        println!(
            "surf init: worst place {worst}, [place, tile, axes, shape, time] {agree:?} of 1205, parted {parted:?}"
        );
        assert_eq!(agree, SURF_INIT_AGREE);
    }

    /// [`surf_init_lays_the_packet_s_waves`]'s fields agreeing, of 1,205
    /// (item 4236): every tile, shape and time; the axes but record 240's,
    /// whose normal falls on a table step (211° against 210°); the place
    /// within [`PLACE`] on 1,193, the worst 0.16 units (a wcoord is 192).
    const SURF_INIT_AGREE: [usize; 5] = [1193, 1205, 1204, 1205, 1205];

    pub(crate) fn install() -> Option<rondata::Install> {
        let root = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let inst = rondata::Install::new(root);
        inst.looks_valid().then_some(inst)
    }

    /// How far a wave may sit from the packet's, in world units.
    const PLACE: f32 = 0.05;
}
