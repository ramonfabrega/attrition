//! Candidate C's measure (`docs/RENDERER.md` §5.3): the scene's
//! silhouettes against the original's frame.
//!
//! Two halves, both headless:
//!
//! - [`rasterize`]: each drawn object's world-space triangles through the
//!   camera's `to_screen_matrix`, the divide by `w` and the viewport — the
//!   arithmetic the client's shader runs (`client::gpu`) — into a buffer of
//!   object keys, nearest first by the camera-space depth the client's
//!   depth test takes (`render_matrix`'s third column, less wins).
//! - [`footprints`]: for each object our buffer shows, a window round its
//!   pixels, and inside it two footprints: ours, every pixel an object of
//!   ours covers, and the original's, every pixel of its photograph whose
//!   colour is not one the window's surrounding ring holds — the ring being
//!   the ground round the object, which is what an object is not. Their
//!   overlap (intersection over union) is the object's score, and an object
//!   whose window touches the photograph's interface or the frame's edge is
//!   not scored.
//!
//! The footprint is the scene's inside the window, not the object's alone:
//! a crowd's members overlap on both sides, and the photograph cannot say
//! which of its pixels is which figure. What it can say is whether there is
//! a figure where we draw one, and nothing where we do not.

use crate::Camera;

/// One drawn object: its key (non-zero) and its triangles in world space.
pub struct Object {
    pub key: u32,
    pub tris: Vec<[[f32; 3]; 3]>,
}

/// The rasterized frame: `width · height` keys, 0 where no object is.
#[derive(Clone, Debug)]
pub struct Keys {
    pub width: usize,
    pub height: usize,
    pub keys: Vec<u32>,
}

impl Keys {
    pub fn at(&self, x: usize, y: usize) -> u32 {
        self.keys[y * self.width + x]
    }
}

/// A pixel rectangle, `x0..x1` by `y0..y1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect { x0, y0, x1, y1 }
    }

    fn meets(&self, o: &Rect) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        (self.x0..self.x1).contains(&x) && (self.y0..self.y1).contains(&y)
    }

    fn grown(&self, by: i32) -> Rect {
        Rect::new(self.x0 - by, self.y0 - by, self.x1 + by, self.y1 + by)
    }
}

/// Every object's triangles into a key buffer at `camera`, `size` pixels.
/// A pixel is covered when its centre is inside the projected triangle,
/// either winding.
pub fn rasterize(objects: &[Object], camera: &Camera, size: (usize, usize)) -> Keys {
    rasterize_with(objects, &[], camera, size)
}

/// One drawn sprite (item 4076): an image blitted on the screen at
/// `at`, its pixels the object's where `covered`, at one camera-space
/// `depth` — the building's anchor, the depth the draw list orders it by.
pub struct Stamp<'a> {
    pub key: u32,
    pub at: crate::draw::SpriteAt,
    pub width: usize,
    pub height: usize,
    /// `width · height`, the top row first.
    pub covered: &'a [bool],
    pub depth: f32,
}

/// [`rasterize`], with sprites beside the meshes: each screen pixel whose
/// centre falls on a covered image pixel, sampled nearest, takes the
/// sprite's key where its depth is the nearest.
pub fn rasterize_with(
    objects: &[Object],
    stamps: &[Stamp<'_>],
    camera: &Camera,
    size: (usize, usize),
) -> Keys {
    rasterize_cut(objects, &[], stamps, camera, size).keys
}

/// **A texture's alpha test on an object's triangles** (item 4082,
/// `docs/RENDERER.md` §3.7): each triangle's texcoords and the image it
/// samples. A fragment is kept only where the image's alpha, filtered
/// bilinearly on its one level with clamped addressing, is above
/// `alpha_ref` — the original's `TEST_GREATER` on `alpha_ref / 255`.
pub struct Cut<'a> {
    /// One `{u, v}` triple a triangle, as the object's `tris`.
    pub uvs: Vec<[[f32; 2]; 3]>,
    /// The image each triangle samples, an index into `images`; a triangle
    /// whose image is `None` is drawn whole.
    pub image: Vec<usize>,
    pub images: &'a [Option<crate::tga::Image>],
    pub alpha_ref: u8,
    /// The winding on the screen that is not drawn ([`Cull`]).
    pub cull: Cull,
}

/// **Which triangles the card does not draw** (item 4082): the
/// `RenderState::cull_mode` the engine names `None`, `Clockwise` and
/// `Counter Clockwise` (0, 1, 2), which `D3D11GLGraphicCard::
/// set_rasterizer_state` (DLL `0x1002b410`) maps through its table at
/// `0x100dd094` to `D3D11_CULL_NONE`, `_BACK` and `_FRONT` with
/// `FrontCounterClockwise` set: so the engine's mode names the winding,
/// on the render target (x right, y down), that is culled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cull {
    None,
    Clockwise,
    CounterClockwise,
}

impl Cull {
    /// Whether a triangle of signed screen area `area` (`(b − a) × (c − a)`,
    /// y down: positive is clockwise as the screen shows it) is culled.
    pub fn culls(self, area: f32) -> bool {
        match self {
            Cull::None => false,
            Cull::Clockwise => area > 0.0,
            Cull::CounterClockwise => area < 0.0,
        }
    }
}

/// `image`'s alpha at `(u, v)`, filtered bilinearly with clamped
/// addressing, on the 0–255 scale: `v = 0` is the top row, as
/// `crate::tga` returns it and as `ImageIO::decode_tga@00546e10` fills the
/// surface. Texel centres at `(i + ½) / width`.
pub fn alpha_bilinear(image: &crate::tga::Image, u: f32, v: f32) -> f32 {
    let (w, h) = (image.width, image.height);
    let fx = (u.clamp(0.0, 1.0) * w as f32 - 0.5).max(0.0);
    let fy = (v.clamp(0.0, 1.0) * h as f32 - 0.5).max(0.0);
    let (x0, y0) = ((fx as usize).min(w - 1), (fy as usize).min(h - 1));
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    let a = |x: usize, y: usize| image.rgba[(y * w + x) * 4 + 3] as f32;
    let top = a(x0, y0) * (1.0 - tx) + a(x1, y0) * tx;
    let bottom = a(x0, y1) * (1.0 - tx) + a(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// A key buffer and the camera-space depth each key won at, `INFINITY`
/// where no object is.
pub struct Raster {
    pub keys: Keys,
    pub depth: Vec<f32>,
}

/// [`rasterize_with`], where `cuts[i]`, when present, is the alpha test on
/// `objects[i]`'s triangles ([`Cut`]); texcoords are interpolated
/// perspective-correct. A cut fragment neither takes the key nor writes the
/// depth, as a `clip` in the pixel shader does.
pub fn rasterize_cut(
    objects: &[Object],
    cuts: &[Option<&Cut<'_>>],
    stamps: &[Stamp<'_>],
    camera: &Camera,
    size: (usize, usize),
) -> Raster {
    let (w, h) = size;
    let project = Projector::new(camera, size);
    let mut keys = vec![0u32; w * h];
    let mut depth = vec![f32::INFINITY; w * h];
    for (n, o) in objects.iter().enumerate() {
        let cut = cut_at(cuts, n);
        for (ti, t) in o.tris.iter().enumerate() {
            let test = cut.and_then(|c| {
                let image = c.images.get(*c.image.get(ti)?)?.as_ref()?;
                Some((image, c.uvs.get(ti)?, c.alpha_ref as f32))
            });
            let cull = cut.map_or(Cull::None, |c| c.cull);
            project.fragments(t, cull, |i, z, q| {
                if z >= depth[i] {
                    return;
                }
                if let Some((image, uv, reference)) = test {
                    let (u, v) = at_weights(uv, q);
                    if alpha_bilinear(image, u, v) <= reference {
                        return;
                    }
                }
                depth[i] = z;
                keys[i] = o.key;
            });
        }
    }
    for st in stamps {
        stamp_pixels(st, size, |i, _| {
            if st.depth < depth[i] {
                depth[i] = st.depth;
                keys[i] = st.key;
            }
        });
    }
    Raster {
        keys: Keys {
            width: w,
            height: h,
            keys,
        },
        depth,
    }
}

/// Every screen pixel whose centre falls on a covered pixel of `st`'s
/// image, sampled nearest: its index, and the image pixel's.
fn stamp_pixels(st: &Stamp<'_>, size: (usize, usize), mut f: impl FnMut(usize, usize)) {
    let (w, h) = size;
    let s = st.at.scale;
    if s <= 0.0 {
        return;
    }
    let x0 = st.at.left.max(0);
    let y0 = st.at.top.max(0);
    let x1 = (st.at.left + (st.width as f32 * s).ceil() as i32).min(w as i32);
    let y1 = (st.at.top + (st.height as f32 * s).ceil() as i32).min(h as i32);
    for y in y0..y1 {
        let v = ((y - st.at.top) as f32 + 0.5) / s;
        if v < 0.0 || v as usize >= st.height {
            continue;
        }
        for x in x0..x1 {
            let u = ((x - st.at.left) as f32 + 0.5) / s;
            if u < 0.0 || u as usize >= st.width {
                continue;
            }
            let j = v as usize * st.width + u as usize;
            if !st.covered[j] {
                continue;
            }
            f(y as usize * w + x as usize, j);
        }
    }
}

/// **A sprite's own colour over a window** (item 4099): [`paint`]'s, for a
/// [`Stamp`] blitted as [`rasterize_cut`] blits it; `rgba` its image.
pub fn paint_stamp(
    st: &Stamp<'_>,
    rgba: &[u8],
    size: (usize, usize),
    window: Rect,
) -> Vec<Option<Texel>> {
    let ww = (window.x1 - window.x0).max(0) as usize;
    let wh = (window.y1 - window.y0).max(0) as usize;
    let mut out = vec![None; ww * wh];
    stamp_pixels(st, size, |i, j| {
        let (x, y) = ((i % size.0) as i32, (i / size.0) as i32);
        if window.contains(x, y) {
            let p = &rgba[j * 4..j * 4 + 4];
            out[(y - window.y0) as usize * ww + (x - window.x0) as usize] = Some(Texel {
                rgb: [p[0], p[1], p[2]].map(f32::from),
                alpha: f32::from(p[3]),
                tri: u32::MAX,
            });
        }
    });
    out
}

/// **A tree's own colour over a window** (item 4103): [`paint`]'s, for an
/// object drawn under its [`Cut`] — the nearest fragment the alpha test
/// keeps, its texel's RGBA filtered as [`alpha_bilinear`] filters the alpha.
pub fn paint_cut(
    object: &Object,
    cut: &Cut<'_>,
    camera: &Camera,
    size: (usize, usize),
    window: Rect,
) -> Vec<Option<Texel>> {
    let project = Projector::new(camera, size);
    let (w, _) = size;
    let ww = (window.x1 - window.x0).max(0) as usize;
    let wh = (window.y1 - window.y0).max(0) as usize;
    let mut out: Vec<Option<Texel>> = vec![None; ww * wh];
    let mut depth = vec![f32::INFINITY; ww * wh];
    for (ti, t) in object.tris.iter().enumerate() {
        let test = (|| {
            let image = cut.images.get(*cut.image.get(ti)?)?.as_ref()?;
            Some((image, cut.uvs.get(ti)?))
        })();
        project.fragments(t, cut.cull, |i, z, q| {
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            if !window.contains(x, y) {
                return;
            }
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            if z >= depth[j] {
                return;
            }
            let (rgb, alpha) = match test {
                Some((image, uv)) => {
                    let (u, v) = at_weights(uv, q);
                    let a = alpha_bilinear(image, u, v);
                    if a <= f32::from(cut.alpha_ref) {
                        return;
                    }
                    (rgb_bilinear(image, u, v), a)
                }
                None => ([0.0; 3], 255.0),
            };
            depth[j] = z;
            out[j] = Some(Texel {
                rgb,
                alpha,
                tri: ti as u32,
            });
        });
    }
    out
}

/// [`alpha_bilinear`]'s filter on the colour channels.
fn rgb_bilinear(image: &crate::tga::Image, u: f32, v: f32) -> [f32; 3] {
    let (w, h) = (image.width, image.height);
    let fx = (u.clamp(0.0, 1.0) * w as f32 - 0.5).max(0.0);
    let fy = (v.clamp(0.0, 1.0) * h as f32 - 0.5).max(0.0);
    let (x0, y0) = ((fx as usize).min(w - 1), (fy as usize).min(h - 1));
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    [0, 1, 2].map(|c| {
        let a = |x: usize, y: usize| image.rgba[(y * w + x) * 4 + c] as f32;
        let top = a(x0, y0) * (1.0 - tx) + a(x1, y0) * tx;
        let bottom = a(x0, y1) * (1.0 - tx) + a(x1, y1) * tx;
        top * (1.0 - ty) + bottom * ty
    })
}

fn cut_at<'a>(cuts: &'a [Option<&'a Cut<'a>>], n: usize) -> Option<&'a Cut<'a>> {
    cuts.get(n).copied().flatten()
}

/// A texcoord at a fragment's perspective-correct weights.
fn at_weights(uv: &[[f32; 2]; 3], q: [f32; 3]) -> (f32, f32) {
    (
        q[0] * uv[0][0] + q[1] * uv[1][0] + q[2] * uv[2][0],
        q[0] * uv[0][1] + q[1] * uv[1][1] + q[2] * uv[2][1],
    )
}

/// **The client shader's arithmetic, once** (items 4038, 4099): a world
/// point through `to_screen_matrix`, the divide by `w` and the viewport, and
/// the camera-space depth `render_matrix`'s third column gives. The key
/// buffer ([`rasterize_cut`]) and the colour ([`paint`]) both walk their
/// fragments through it, so the two cannot disagree on a pixel.
struct Projector {
    m: [f32; 16],
    d: [f32; 4],
    w: usize,
    h: usize,
}

impl Projector {
    fn new(camera: &Camera, size: (usize, usize)) -> Projector {
        let r = camera.render_matrix().0;
        Projector {
            m: camera.to_screen_matrix().0,
            d: [r[2], r[6], r[10], r[14]],
            w: size.0,
            h: size.1,
        }
    }

    /// Pixel x, y, camera-space depth and clip `w`.
    fn project(&self, p: [f32; 3]) -> Option<[f32; 4]> {
        let v = [p[0], p[1], p[2], 1.0];
        let m = &self.m;
        let col = |c: usize| (0..4).map(|i| v[i] * m[i * 4 + c]).sum::<f32>();
        let (cx, cy, cw) = (col(0), col(1), col(3));
        if cw.abs() < 1e-6 {
            return None;
        }
        let z: f32 = (0..4).map(|i| v[i] * self.d[i]).sum();
        Some([
            (cx / cw + 1.0) * self.w as f32 * 0.5,
            (1.0 - cy / cw) * self.h as f32 * 0.5,
            z,
            cw,
        ])
    }

    /// Every pixel whose centre the projected triangle covers, either
    /// winding unless `cull` drops it: its index, its depth, and its
    /// perspective-correct weights (summing to 1).
    fn fragments(&self, t: &[[f32; 3]; 3], cull: Cull, mut f: impl FnMut(usize, f32, [f32; 3])) {
        let (w, h) = (self.w, self.h);
        let (Some(a), Some(b), Some(c)) =
            (self.project(t[0]), self.project(t[1]), self.project(t[2]))
        else {
            return;
        };
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() < 1e-9 || cull.culls(area) {
            return;
        }
        let lo = |i: usize| a[i].min(b[i]).min(c[i]);
        let hi = |i: usize| a[i].max(b[i]).max(c[i]);
        let x0 = (lo(0) - 0.5).ceil().max(0.0) as usize;
        let y0 = (lo(1) - 0.5).ceil().max(0.0) as usize;
        let x1 = ((hi(0) - 0.5).floor() + 1.0).clamp(0.0, w as f32) as usize;
        let y1 = ((hi(1) - 0.5).floor() + 1.0).clamp(0.0, h as f32) as usize;
        for y in y0..y1 {
            let py = y as f32 + 0.5;
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let e = |p: [f32; 4], q: [f32; 4]| {
                    ((q[0] - p[0]) * (py - p[1]) - (q[1] - p[1]) * (px - p[0])) / area
                };
                let (wa, wb, wc) = (e(b, c), e(c, a), e(a, b));
                if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                    continue;
                }
                let z = wa * a[2] + wb * b[2] + wc * c[2];
                let (qa, qb, qc) = (wa / a[3], wb / b[3], wc / c[3]);
                let q = qa + qb + qc;
                f(y * w + x, z, [qa / q, qb / q, qc / q]);
            }
        }
    }
}

/// **A texture and its box-filtered levels** (item 4099), level 0 the
/// image, each next level the 2×2 mean of the one before, down to 1×1.
/// That the original's card samples the figure's texture from such a chain
/// is a supposition (`docs/RENDERER.md` §5.3, "The colour reader").
#[derive(Clone, Debug)]
pub struct Mips {
    pub levels: Vec<crate::tga::Image>,
}

impl Mips {
    pub fn new(image: crate::tga::Image) -> Mips {
        let mut levels = vec![image];
        loop {
            let last = levels.last().expect("level 0");
            if last.width <= 1 && last.height <= 1 {
                break;
            }
            let (w, h) = ((last.width / 2).max(1), (last.height / 2).max(1));
            let mut rgba = vec![0u8; w * h * 4];
            for y in 0..h {
                for x in 0..w {
                    for c in 0..4 {
                        let mut sum = 0u32;
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let sx = (2 * x + dx).min(last.width - 1);
                            let sy = (2 * y + dy).min(last.height - 1);
                            sum += u32::from(last.rgba[(sy * last.width + sx) * 4 + c]);
                        }
                        rgba[(y * w + x) * 4 + c] = ((sum + 2) / 4) as u8;
                    }
                }
            }
            levels.push(crate::tga::Image {
                width: w,
                height: h,
                rgba,
                ..last.clone()
            });
        }
        Mips { levels }
    }

    /// The level a triangle samples: the texels its texcoords span over
    /// the pixels its projection covers, as a power of four.
    pub fn level(&self, texel_area: f32, pixel_area: f32) -> usize {
        if pixel_area <= 0.0 || texel_area <= pixel_area {
            return 0;
        }
        let lod = 0.5 * (texel_area / pixel_area).log2();
        (lod.floor() as usize).min(self.levels.len() - 1)
    }

    /// RGBA at `(u, v)` on `level`, bilinear, wrapped: `v = 0` the top row,
    /// as [`alpha_bilinear`] reads the trees' textures.
    pub fn sample(&self, level: usize, u: f32, v: f32) -> [f32; 4] {
        let image = &self.levels[level];
        let (w, h) = (image.width as i64, image.height as i64);
        let fx = u * w as f32 - 0.5;
        let fy = v * h as f32 - 0.5;
        let (x0, y0) = (fx.floor() as i64, fy.floor() as i64);
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let at = |x: i64, y: i64, c: usize| {
            let (x, y) = (x.rem_euclid(w) as usize, y.rem_euclid(h) as usize);
            f32::from(image.rgba[(y * image.width + x) * 4 + c])
        };
        [0, 1, 2, 3].map(|c| {
            let top = at(x0, y0, c) * (1.0 - tx) + at(x0 + 1, y0, c) * tx;
            let bottom = at(x0, y0 + 1, c) * (1.0 - tx) + at(x0 + 1, y0 + 1, c) * tx;
            top * (1.0 - ty) + bottom * ty
        })
    }
}

/// **The owners' unit colours** (item 4102; `docs/RENDERER.md` §5.3, "The
/// team colour"): `TeamColor::unit` of `team_colors.list[0..8]` as
/// run1559b's packet holds them, the figure draw's `tfactor`.
pub const TEAM_UNIT: [[u8; 3]; 8] = [
    [255, 0, 0],
    [34, 82, 255],
    [255, 0, 255],
    [0, 208, 0],
    [255, 255, 0],
    [0, 255, 255],
    [255, 255, 255],
    [255, 105, 0],
];

/// The owner's unit colour; past the eight, white (`team_colors.list[8]`).
pub fn team_unit(who: u8) -> [u8; 3] {
    TEAM_UNIT
        .get(usize::from(who))
        .copied()
        .unwrap_or([255, 255, 255])
}

/// **The team colour** (item 4102; §5.3): `TOP_BLENDTEXTUREALPHA` with
/// the texture and the factor, `t·α + c·(1 − α)`, α the texel's alpha
/// over 255 — the arithmetic `client::gpu`'s textured shader runs.
pub fn team_blend(rgb: [f32; 3], alpha: f32, owner: [u8; 3]) -> [f32; 3] {
    let a = alpha / 255.0;
    [0, 1, 2].map(|k| rgb[k] * a + f32::from(owner[k]) * (1.0 - a))
}

/// **The silhouette's alpha** (item 4103; §5.3, "The silhouette's
/// colour"): `render_states[1]`'s `tfactor` alpha on run1559b's packet,
/// the byte `GraphicPieces::draw_unit@008e7da0` lays beside the owner's
/// unit colour (0x8e84a5–0x8e84ab) before it stores both (0x8e8598).
pub const SILHOUETTE_ALPHA: u8 = 128;

/// **The silhouette's pixel** (item 4103; §5.3): `render_states[1]`'s
/// `SELECTARG2(TFACTOR)` for colour and alpha, blended source-alpha over
/// the nearer pixel `nearer`: `c·a + n·(1 − a)`, `c` the owner's unit
/// colour, `a` [`SILHOUETTE_ALPHA`] over 255 — the arithmetic
/// `client::gpu`'s silhouette pass runs.
pub fn silhouette_blend(owner: [u8; 3], nearer: [f32; 3]) -> [f32; 3] {
    let a = f32::from(SILHOUETTE_ALPHA) / 255.0;
    [0, 1, 2].map(|k| f32::from(owner[k]) * a + nearer[k] * (1.0 - a))
}

/// How an object is coloured for [`paint`].
pub enum Paint<'a> {
    /// One colour on every triangle: the debug tint.
    Flat([u8; 3]),
    /// Each triangle's texcoords into one texture, the owner's colour
    /// through its alpha ([`team_blend`]); `None` the texel alone (4099).
    Texture {
        uvs: &'a [[[f32; 2]; 3]],
        mips: &'a Mips,
        owner: Option<[u8; 3]>,
    },
}

/// One painted pixel: its colour and, when textured, the texel's alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texel {
    pub rgb: [f32; 3],
    pub alpha: f32,
    /// The triangle it is drawn from, `u32::MAX` for a sprite.
    pub tri: u32,
}

/// **An object's own colour over a window** (item 4099): `object` alone,
/// through [`rasterize_cut`]'s fragments, the nearest fragment's colour at
/// each of `window`'s pixels (row-major), `None` where it covers none.
pub fn paint(
    object: &Object,
    paint: &Paint<'_>,
    camera: &Camera,
    size: (usize, usize),
    window: Rect,
) -> Vec<Option<Texel>> {
    let project = Projector::new(camera, size);
    let (w, _) = size;
    let ww = (window.x1 - window.x0).max(0) as usize;
    let wh = (window.y1 - window.y0).max(0) as usize;
    let mut out: Vec<Option<Texel>> = vec![None; ww * wh];
    let mut depth = vec![f32::INFINITY; ww * wh];
    for (ti, t) in object.tris.iter().enumerate() {
        let level = match paint {
            Paint::Texture { uvs, mips, .. } => uvs.get(ti).map(|uv| {
                let base = &mips.levels[0];
                let du = [uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]];
                let dv = [uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]];
                let texels =
                    0.5 * (du[0] * dv[1] - du[1] * dv[0]).abs() * (base.width * base.height) as f32;
                let pa = project.project(t[0]);
                let pb = project.project(t[1]);
                let pc = project.project(t[2]);
                let pixels = match (pa, pb, pc) {
                    (Some(a), Some(b), Some(c)) => {
                        0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs()
                    }
                    _ => 0.0,
                };
                mips.level(texels, pixels)
            }),
            Paint::Flat(_) => None,
        };
        project.fragments(t, Cull::None, |i, z, q| {
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            if !window.contains(x, y) {
                return;
            }
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            if z >= depth[j] {
                return;
            }
            let texel = match paint {
                Paint::Flat(c) => Texel {
                    rgb: c.map(f32::from),
                    alpha: 255.0,
                    tri: ti as u32,
                },
                Paint::Texture { uvs, mips, owner } => {
                    let Some(uv) = uvs.get(ti) else {
                        return;
                    };
                    let (u, v) = at_weights(uv, q);
                    let s = mips.sample(level.unwrap_or(0), u, v);
                    Texel {
                        rgb: match owner {
                            Some(c) => team_blend([s[0], s[1], s[2]], s[3], *c),
                            None => [s[0], s[1], s[2]],
                        },
                        alpha: s[3],
                        tri: ti as u32,
                    }
                }
            };
            depth[j] = z;
            out[j] = Some(texel);
        });
    }
    out
}

/// **The colour reader's constants** (item 4099; `docs/RENDERER.md` §5.3,
/// "The colour reader"), pinned before its first count.
#[derive(Clone, Copy, Debug)]
pub struct Hue {
    /// A pixel whose channel sum is under this, on either side, is dark:
    /// its chromaticity is noise, and it is left out.
    pub dark: f32,
    /// Two chromaticities agree within this distance.
    pub distance: f32,
    /// An object passes when this share of its lit pixels agree...
    pub share: f32,
    /// ...and it holds at least this many.
    pub min_pixels: u32,
    /// A texel whose alpha is under this is the mask's, by supposition.
    pub mask_alpha: f32,
}

/// The reading the colour row is counted at.
pub const HUE: Hue = Hue {
    dark: 60.0,
    distance: 0.05,
    share: 0.5,
    min_pixels: 40,
    mask_alpha: 255.0,
};

/// `(r, g) / (r + g + b)`.
pub fn chromaticity(rgb: [f32; 3]) -> [f32; 2] {
    let s = rgb[0] + rgb[1] + rgb[2];
    if s <= 0.0 {
        [1.0 / 3.0, 1.0 / 3.0]
    } else {
        [rgb[0] / s, rgb[1] / s]
    }
}

/// One object's colours on the pixels both footprints hold.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Colours {
    pub key: u32,
    /// Pixels its own key holds and the photograph's footprint holds.
    pub held: u32,
    /// Of those, dark on either side.
    pub dark: u32,
    /// The rest, and how many of them agree.
    pub lit: u32,
    pub agree: u32,
    /// Lit pixels whose texel is the mask's ([`Hue::mask_alpha`]), and how
    /// many of those part.
    pub masked: u32,
    pub masked_part: u32,
    /// Lit pixels `behind` names (the silhouette's pass, item 4085), and
    /// how many of those part; counted before the mask.
    pub behind: u32,
    pub behind_part: u32,
    /// The mean distance over the lit pixels.
    pub mean: f32,
    /// Over the lit pixels neither behind nor the mask's, the sum of the
    /// photograph's chromaticity less ours, and how many: the light's tint.
    pub offset: [f32; 2],
    pub rest: u32,
}

impl Colours {
    pub fn passes(&self, hue: &Hue) -> bool {
        self.lit >= hue.min_pixels && self.agree as f32 >= hue.share * self.lit as f32
    }

    /// The lit pixels that part.
    pub fn part(&self) -> u32 {
        self.lit - self.agree
    }
}

/// **The colour reader** (item 4099; §5.3): over `window`, the pixels whose
/// key is `key` and where `theirs` (window-sized, [`traced_theirs`]) holds,
/// `painted`'s colour against the photograph's.
#[allow(clippy::too_many_arguments)]
pub fn read_colours(
    keys: &Keys,
    photo: &Photo<'_>,
    key: u32,
    window: Rect,
    theirs: &[Option<bool>],
    painted: &[Option<Texel>],
    hue: &Hue,
    behind: impl Fn(i32, i32) -> bool,
) -> Colours {
    let ww = (window.x1 - window.x0) as usize;
    let mut c = Colours {
        key,
        ..Colours::default()
    };
    let mut total = 0.0f32;
    for y in window.y0..window.y1 {
        for x in window.x0..window.x1 {
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            if keys.at(x as usize, y as usize) != key || theirs[j] != Some(true) {
                continue;
            }
            let Some(ours) = painted[j] else {
                continue;
            };
            c.held += 1;
            let i = (y as usize * photo.width + x as usize) * 4;
            let them = [0, 1, 2].map(|k| f32::from(photo.rgba[i + k]));
            let sum = |p: [f32; 3]| p[0] + p[1] + p[2];
            if sum(ours.rgb) < hue.dark || sum(them) < hue.dark {
                c.dark += 1;
                continue;
            }
            let (a, b) = (chromaticity(ours.rgb), chromaticity(them));
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
            c.lit += 1;
            total += d;
            let agrees = d <= hue.distance;
            c.agree += u32::from(agrees);
            if behind(x, y) {
                c.behind += 1;
                c.behind_part += u32::from(!agrees);
            } else if ours.alpha < hue.mask_alpha {
                c.masked += 1;
                c.masked_part += u32::from(!agrees);
            } else {
                c.rest += 1;
                c.offset[0] += b[0] - a[0];
                c.offset[1] += b[1] - a[1];
            }
        }
    }
    c.mean = if c.lit == 0 {
        0.0
    } else {
        total / c.lit as f32
    };
    c
}

/// The photograph: `width · height` RGBA8 pixels, the top row first — what
/// `crate::tga::Image` gives.
pub struct Photo<'a> {
    pub width: usize,
    pub height: usize,
    pub rgba: &'a [u8],
}

/// How a footprint is read off the photograph.
#[derive(Clone, Copy, Debug)]
pub struct Reading {
    /// Pixels the window extends past the object's own.
    pub margin: i32,
    /// The ground ring's width outside the window.
    pub ring: i32,
    /// Colours are binned at `256 >> bin_bits` levels a channel; a pixel is
    /// the ground's when a ring colour sits within `tolerance` bins of it
    /// on every channel.
    pub bin_bits: u32,
    pub tolerance: i32,
    /// An object showing fewer pixels than this is not scored.
    pub min_pixels: u32,
    /// The overlap an object must reach to count.
    pub threshold: f32,
}

/// The reading the pinned frame is scored at.
pub const READING: Reading = Reading {
    margin: 6,
    ring: 6,
    bin_bits: 3,
    tolerance: 1,
    min_pixels: 40,
    threshold: 0.5,
};

/// **Candidate C's frame drawn at rest** (item 4038; `docs/RENDERER.md`
/// §5.3): on the Aztec Modern pair's frame 3294 at run1559b's camera, the
/// objects whose footprint overlaps run1559b's photograph by
/// [`READING`]'s half or more, of the objects scored (`client::silhouette`'s
/// test holds it). Item 4069: 3 → 2, at the original's facing
/// (`render::figure::guy_placement`): 1/24 0.528 → 0.394 falls, 0/0 0.521
/// → 0.670 and 0/5 0.599 → 0.718 hold. The rest pose is no frame the
/// original draws; the row is held beside [`RUN1559B_3294_POSED`], which
/// is the client's default.
pub const RUN1559B_3294: (usize, usize) = (2, 23);

/// **Candidate C's row: the same frame with every figure posed** at its
/// animation slot and clock (item 4067, `render::figure::posed`), the
/// client's default. Item 4069: 1 → 6 of 23, drawn at the original's
/// facing, the model's y negated and turned by `GuyOut::draw`'s angle with
/// x towards y (`render::figure::guy_placement`) rather than reflected in
/// world y: 0/0 0.160 → 0.938, 0/8 0.245 → 0.565, 0/9 0.313 → 0.571, 1/21
/// 0.498 → 0.647, 1/23 0.280 → 0.616 pass, 0/5 0.566 → 0.820 holds; the
/// tanks 1/40 0.135 → 0.300 and 1/16 0.037 → 0.091 stay under, at the
/// colour reader (4065b) and ~40 px above the photograph's (4065c).
///
/// Item 4089, a figure of `CAT` Mech or Artillery tilted to its slope
/// (§5.3, "The tilt"): 6 → 8 of 23. Two riflemen, neither tilted, each in
/// a tilted tank's window, cross up: 1/24 0.492 → 0.535 and 1/22 0.480 →
/// 0.500. The tanks stay under: 1/40 0.300 → 0.317, 1/15 0.019 → 0.037,
/// 1/16 0.091 → 0.097. At rest, as before: [`RUN1559B_3294`].
pub const RUN1559B_3294_POSED: (usize, usize) = (8, 23);

/// The same frame, drawn posed, scored at run1441's camera of it, which the
/// photograph was not taken at: the comparison made to fail (item 4038).
pub const RUN1441_CAMERA_3294: (usize, usize) = (0, 25);

/// **Candidate C's row with the tanks read by the hand-traced mask** (item
/// 4071, [`Reader::Mask`], `client::silhouette::TRACED`): the default
/// drawing, the three tanks' photograph footprints read off the trace and
/// every other row by colour. 6 → 7 of 23: 1/40 theirs 1,013 → 2,720
/// against ours 2,697, overlap 0.300 → 0.710; 1/15 theirs 98 → 1,988
/// against 4,300, 0.019 → 0.237, and 1/16 474 → 2,100 against 3,849, 0.091
/// → 0.357, stay under at the ~40 px place (4065c).
///
/// Item 4089, the tilt: 7 → 9 of 23, the same two riflemen by colour (1/24,
/// 1/22, as [`RUN1559B_3294_POSED`]); by the mask 1/40 0.710 → 0.676, 1/15
/// 0.237 → 0.260 and 1/16 0.357 → 0.364 hold their sides.
pub const RUN1559B_3294_MASKED: (usize, usize) = (9, 23);

/// The mask at run1441's camera: the made-to-fail, as for colour.
pub const RUN1441_CAMERA_3294_MASKED: (usize, usize) = (0, 25);

/// **Candidate C on the frame the photograph shows** (item 4073): frame
/// 3279, `client::silhouette::PHOTO_FRAME`, the frame whose movers sit on
/// the photograph, drawn by default and the tanks read by the mask, at the
/// same camera, readers and constants as the 3294 rows. 7 of 23 at 3294 →
/// **16 of 24**: the tanks 1/40 0.710 → 0.790, 1/15 0.237 → 0.669 and 1/16
/// 0.357 → 0.781 pass; 1/35, the figure with no counterpart (4038c), is a
/// mover and passes at 0.701 by colour. The tanks were placed right: the
/// photograph is fifteen frames older than the packet.
///
/// Item 4089, the tilt: 16 → 18 of 24, and by colour alone 13 → 15. Two
/// riflemen, neither tilted, each in a tilted tank's window, cross up: 1/20
/// 0.497 → 0.507 and 1/36 0.340 → 0.622. The tanks by the mask: 1/40 0.790
/// → 0.719, 1/15 0.669 → 0.766, 1/16 0.781 → 0.872. At rest 4 → 6: 1/36
/// 0.366 → 0.661 and 1/22 0.437 → 0.501.
pub const RUN1559B_3279_MASKED: (usize, usize) = (18, 24);

/// The same frame by colour alone: 6 of 23 at 3294 → 13 of 24.
pub const RUN1559B_3279_POSED: (usize, usize) = (15, 24);

/// The same frame drawn at rest, by colour: 2 of 23 at 3294 → 4 of 24.
pub const RUN1559B_3279: (usize, usize) = (6, 24);

/// **The same frame with the buildings as the original draws them** (item
/// 4076): each `<BUILD>`'s `texture=` sprite at `render::draw::sprite_at`
/// rather than its `zbuffer` model, the client's default since. Placed on
/// the photograph's buildings within a pixel or two by eye, the sprites
/// read **14 of 23** by the mask (from 16 of 24) and **11 of 23** by colour
/// (from 13 of 24): the reader as it stands cannot score a building drawn
/// right. The capital 0/2000's sprite reaches y 573 and its ring meets the
/// city's label, so it is no longer scored; 0/2001's window grows to the
/// photograph's woodcutter, (1579, 314)..(1706, 459), 0.209 → 0.351, its
/// ring the forest's; and the figures beside it, whose rings lie on the
/// sprite and read it as ground, fall: 0/9 0.753 → 0.410, 0/1 0.846 →
/// 0.658, 0/10 0.531 → 0.354, 0/8 0.916 → 0.587, while 0/6 rises 0.280 →
/// 0.474. The rows above are kept as the `zbuffer` draw's; candidate C's
/// row on this draw is [`RUN1559B_3279_TRACED`] (item 4077).
///
/// Item 4089, the tilt: 14 → 16 by the mask, 11 → 13 by colour, the two
/// riflemen of [`RUN1559B_3279_MASKED`] (1/20 0.497 → 0.507, 1/36 0.340 →
/// 0.622).
pub const RUN1559B_3279_SPRITES: (usize, usize) = (16, 23);
pub const RUN1559B_3279_SPRITES_POSED: (usize, usize) = (13, 23);

/// **Candidate C's row** (item 4077; `docs/RENDERER.md` §5.3, "A traced
/// region"): frame 3279 at run1559b's camera on the client's default draw,
/// the buildings as their sprites, read by [`footprints_traced`] over
/// `client::silhouette::TRACED`. 14 of 23 → **27 of 33**: the interface
/// read as neither brings in ten objects the old skip left out, among them
/// the capital 0/2000 (0.872); the woodcutter 0/2001 0.351 → 0.652; the
/// figures by it, their rings and windows clear of its trace, 0/9 0.410 →
/// 0.711, 0/1 0.658 → 0.796, 0/10 0.354 → 0.383, 0/8 0.587 → 0.587; and
/// 1/33 beside the tanks 0.360 → 0.549. No row crosses down.
///
/// **27 → 29 of 33** (item 4079): the last tank 1/27 traced on the
/// photograph (`ebcbcb0f7`, before its count) reads 0.259 → 0.662, and the
/// rifleman 1/36 in its window 0.340 → 0.503. 1/31 0.429 → 0.469 and 1/20
/// 0.497 → 0.482 stay out; 1/32 0.352 → 0.283; 1/38 0.813 → 0.807 holds.
/// No row crosses down.
///
/// **29 of 33 → 25 of 29** (item 4080): the map's trees drawn, under §5.3's
/// tree rule (`be86262b5`, before its count). Their geometry is the
/// packet's (`client::world::tests::the_trees_are_the_packets_own_geometry`),
/// but they are drawn as solid cards where the original's leaves are
/// alpha-cut, and in the photograph the figures by the woodcutter stand in
/// front of the pine. Four of those figures are hidden and leave the row:
/// 0/1 (0.796), 0/2 (0.690), 0/8 (0.587) and 0/10 (0.383). 0/6 0.522 →
/// 0.348 crosses down. 0/2001 0.652 → 0.683. The rows that moved are item
/// 4080's journal. Without trees, as before: [`RUN1559B_3279_TRACED_TREELESS`].
///
/// **25 → 26 of 29** (item 4082): the leaves alpha-tested at
/// `render_states[9]`'s reference 0 and culled counter-clockwise, under
/// §5.3's leaves rule (`882ccba36`, before its count). 0/6 0.348 → 0.513
/// (ours 329 → 352, theirs 198 → 267, both 136 → 210) crosses up; 0/2001
/// 0.683 → 0.670. No other row moves: the four figures 0/1, 0/2, 0/8 and
/// 0/10 stay out, the pine nearer than each on every pixel it covers
/// (parked 4082a). Solid, as before: [`RUN1559B_3279_TRACED_LEAVES_SOLID`].
///
/// **26 of 29 → 30 of 33** (item 4085): a land figure behind a tree or a
/// sprite drawn as its silhouette, `render_states[1]`, under §5.3's
/// silhouettes rule (`42930df49`, before its count). The four figures by
/// the woodcutter come back into the row: 0/1 0.853, 0/2 0.692, 0/8 0.911,
/// 0/10 0.541. 0/6 0.513 → 0.535; 0/2001 0.670 → 0.688 (ours 5695 → 5800,
/// theirs 6808 → 6968, both 5018 → 5206). No row crosses down. Hidden, as
/// before: [`RUN1559B_3279_TRACED_NO_SILHOUETTES`].
///
/// **30 of 33 → 31 of 33** (item 4089): a figure of `CAT` Mech or
/// Artillery tilted to the negated world slope normal at its point, taken
/// whole, under §5.3's tilt rule (`207ec454b`, before its count). 1/31
/// 0.469 → 0.537 (ours 669 → 674, theirs 318 → 362, both 315 → 362) crosses
/// up; 1/20 0.482 → 0.489 and 1/32 0.283 → 0.287 stay out; the tank 1/27
/// 0.662 → 0.713. 1/15 0.669 → 0.768, 1/16 0.772 → 0.855, 1/30 0.601 →
/// 0.744, 1/36 0.503 → 0.724, 1/38 0.807 → 0.834, 1/33 0.549 → 0.546, 1/40
/// 0.788 → 0.721. No row crosses down. Level, as before:
/// [`RUN1559B_3279_TRACED_LEVEL`]; eased: [`RUN1559B_3279_TRACED_EASED`].
///
/// **31 of 33 → 33 of 33** (item 4095): the photograph is frame 3280's
/// state drawn at the points held on 3279 (`Blend::Last`, run to 3280),
/// under §5.3's point rule (`e3df24b04`, before its count). 1/20 0.489 →
/// 0.803 (ours 599 → 634, theirs 375 → 554, both 320 → 529) and 1/32 0.287
/// → 0.828 (546 → 549, 203 → 471, 167 → 462) cross up. 1/35 0.701 → 0.851,
/// 1/15 0.768 → 0.862, 1/16 0.855 → 0.895, 1/33 0.546 → 0.826, 0/10 0.541 →
/// 0.949, 0/2 0.692 → 0.942, 1/31 0.537 → 0.601, 1/27 0.713 → 0.742; 1/38
/// 0.834 → 0.773. No row crosses down. Frame 3279 whole, as before:
/// [`RUN1559B_3279_TRACED_CURRENT`]; the readings below are its own.
pub const RUN1559B_3279_TRACED: (usize, usize) = (33, 33);

/// [`RUN1559B_3279_TRACED`] on frame 3279 whole (`Blend::Current`): the
/// draw before item 4095, and the one every reading below is taken on.
pub const RUN1559B_3279_TRACED_CURRENT: (usize, usize) = (31, 33);

/// [`RUN1559B_3279_TRACED`] with every figure level (`Tilt::Level`): the
/// draw before item 4089.
pub const RUN1559B_3279_TRACED_LEVEL: (usize, usize) = (30, 33);

/// [`RUN1559B_3279_TRACED`] under the ease at one draw a frame from 3200
/// (`Tilt::Eased`), the reading reported beside the rule (item 4089):
/// 1/31 0.536, 1/20 0.488, 1/32 0.286.
pub const RUN1559B_3279_TRACED_EASED: (usize, usize) = (31, 33);
/// The silhouettes' rejected reading, `Silhouettes::None` (item 4085): a
/// figure hidden where a tree or a sprite is nearer, item 4082's draw.
/// Item 4089, the tilt: 26 → 27 of 29, 1/31 0.469 → 0.537 as in the rule.
pub const RUN1559B_3279_TRACED_NO_SILHOUETTES: (usize, usize) = (27, 29);
/// The trees' rejected reading, `TreePixels::Scene`: a tree's pixel is
/// ours in every window it falls in. 26 of 29 under both leaf readings;
/// 26 of 29 → 29 of 33 with the silhouettes (item 4085). Item 4089, the
/// tilt: → 30 of 33, 1/31 0.469 → 0.537, as are the leaves solid and the
/// treeless draw below.
pub const RUN1559B_3279_TRACED_TREES_SCENE: (usize, usize) = (30, 33);
/// The leaves' rejected reading, `Leaves::Solid` (item 4082): every
/// tree triangle whole, item 4080's draw. 25 of 29 → 29 of 33 with the
/// silhouettes (item 4085).
pub const RUN1559B_3279_TRACED_LEAVES_SOLID: (usize, usize) = (30, 33);
/// The row on the treeless draw (`--no-trees`): item 4079's 29 of 33.
pub const RUN1559B_3279_TRACED_TREELESS: (usize, usize) = (30, 33);
/// The rejected readings, reported beside the rule: a neighbour's trace as
/// the scene; the interface skipped as before (the capital out); the paved
/// plaza traced as the capital's (`client::silhouette::PLAZA`, 0/2000 0.818);
/// and tank 1/27 un-notched (`client::silhouette::TANK_1_27_WHOLE`, item
/// 4079: 1/27 0.797, 1/36 0.436, 1/31 0.293). Item 4079 moved the first
/// three 27 → 31, 19 → 20 and 27 → 29. With the trees drawn (item 4080),
/// treeless before: 31/33 → 28/29, 20/23 → 16/19, 29/33 → 25/29, 27/33 →
/// 23/29. With the leaves alpha-tested (item 4082): 28/29 held, 16/19 →
/// 17/19, 25/29 → 26/29, 23/29 → 24/29. With the silhouettes (item
/// 4085): 28/29 → 32/33, 17/19 → 21/23, 26/29 → 30/33, 24/29 → 28/33.
/// With the tilt (item 4089): 32/33 and 21/23 held; the plaza 30/33 →
/// 31/33 (1/31 0.469 → 0.537); un-notched 28/33 → 30/33, two riflemen in
/// the tank's window crossing up, 1/36 0.436 → 0.739 and 1/38 0.464 →
/// 0.750.
pub const RUN1559B_3279_TRACED_SCENE: (usize, usize) = (32, 33);
pub const RUN1559B_3279_TRACED_SKIP: (usize, usize) = (21, 23);
pub const RUN1559B_3279_TRACED_PLAZA: (usize, usize) = (31, 33);
pub const RUN1559B_3279_TRACED_WHOLE: (usize, usize) = (30, 33);
/// The made-to-fail: the same frame and traces at run1441's camera; 0 of
/// 32 → 0 of 28 with the trees drawn; → 0 of 32 with the silhouettes
/// (item 4085).
pub const RUN1441_CAMERA_3279_TRACED: (usize, usize) = (0, 32);

/// **The colour row** (item 4099; `docs/RENDERER.md` §5.3, "The colour
/// reader", committed before its count in `744093625`): on
/// [`RUN1559B_3279_TRACED`]'s draw, the objects whose colours [`HUE`]
/// passes, of the 33 it scores, the figures textured through their
/// texcoords: **11 of 33** — nine figures (1/19, 1/20, 1/24, 1/26 piece
/// 1008, 1/27, 1/30, 1/31, 1/35, 1/36) and the two buildings' sprites.
/// The figures' lit pixels agree 7,379 of 17,817; what parts is the
/// silhouette's pass (12 %), the texels under alpha 255 (22 %) and the
/// rest (66 %); the client's `the_figures_colours_on_the_photograph`.
///
/// **15 of 33 since item 4102** (from 11), the owner's unit colour through
/// the texel's alpha ([`team_blend`], §5.3 "The team colour", committed
/// before this count): 0/5, 1/21, 1/26 piece 26352 and 1/34 cross up, none
/// down; the figures' lit pixels agree 8,165 of 17,889, and of the masked
/// 2,733 part 61 % (88 % with the texel alone).
///
/// **18 of 33 since item 4103** (from 15), a pixel the silhouette's pass
/// owns drawn as the owner's unit colour at 128/255 over the nearer tree's
/// or sprite's pixel ([`silhouette_blend`], §5.3 "The silhouette's colour",
/// committed before this count in `ccef32aac`): 0/9 (agree 32 → 101 of
/// 199), 0/1 (7 → 81 of 159) and 0/6 (90 → 97 of 187) cross up, none down;
/// 0/2, 0/8 and 0/10 rise (7 → 135, 10 → 118, 5 → 133) and stay out; of the
/// figures' 1,257 behind pixels 697 part (from 1,235); the figures agree
/// 8,679 of 17,889.
pub const RUN1559B_3279_COLOUR: (usize, usize) = (18, 33);
/// [`RUN1559B_3279_COLOUR`] with the team colour and no silhouette's
/// colour, item 4102's draw.
pub const RUN1559B_3279_COLOUR_TEAM: (usize, usize) = (15, 33);
/// [`RUN1559B_3279_COLOUR`] with the texel alone, item 4099's draw.
pub const RUN1559B_3279_COLOUR_TEXEL: (usize, usize) = (11, 33);
/// [`RUN1559B_3279_COLOUR`] in the debug tint, the draw before: the two
/// sprites alone, 0 of 31 figures (69 of 18,284 lit pixels agree).
pub const RUN1559B_3279_COLOUR_TINT: (usize, usize) = (2, 33);
/// [`RUN1559B_3279_COLOUR`] at run1441's camera: 0 of 32.
pub const RUN1441_CAMERA_3279_COLOUR: (usize, usize) = (0, 32);

/// The photograph's frame at run1441's camera, by colour and by the mask:
/// the made-to-fail.
pub const RUN1441_CAMERA_3279: (usize, usize) = (0, 24);
pub const RUN1441_CAMERA_3279_MASKED: (usize, usize) = (0, 24);

/// **The first try, refused** (item 4071): [`ROBUST`] reads the default
/// frame 9 of 23 but loses 0/9 (colour 0.571, robust 0.023, its ring the
/// derrick's trees), and reads bare grass as objects: 323 of 12,000 and
/// 113 of 16,000 pixels of `client::silhouette::BARE`, each above
/// [`READING`]'s 40. It is kept as the record of that kill, never as the
/// instrument. Item 4089, the tilt: 9 → 10, 1/24 robust 0.474 → 0.501;
/// the bare grass held.
pub const ROBUST_3294: (usize, usize) = (10, 23);
pub const ROBUST_BARE: [u32; 2] = [323, 113];

/// One object's two footprints in its window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub key: u32,
    pub window: Rect,
    /// The window's pixels an object of ours covers.
    pub ours: u32,
    /// The window's pixels the photograph shows something not the ground.
    pub theirs: u32,
    pub both: u32,
}

impl Footprint {
    /// Intersection over union.
    pub fn overlap(&self) -> f32 {
        let union = self.ours + self.theirs - self.both;
        if union == 0 {
            0.0
        } else {
            self.both as f32 / union as f32
        }
    }
}

/// Each object's footprints, in key order. `interface` is the photograph's
/// windows and labels drawn over the scene: an object whose window or ring
/// touches one, or the frame's edge, is left out.
pub fn footprints(
    keys: &Keys,
    photo: &Photo<'_>,
    interface: &[Rect],
    reading: &Reading,
) -> Vec<Footprint> {
    footprints_read(keys, photo, interface, reading, Reader::Colour)
}

/// [`footprints`], the photograph's side read by `reader`; the windows and
/// which objects are scored do not depend on the reader.
pub fn footprints_read(
    keys: &Keys,
    photo: &Photo<'_>,
    interface: &[Rect],
    reading: &Reading,
    reader: Reader,
) -> Vec<Footprint> {
    assert_eq!((keys.width, keys.height), (photo.width, photo.height));
    let (w, h) = (keys.width as i32, keys.height as i32);
    let mut boxes: std::collections::BTreeMap<u32, (Rect, u32)> = Default::default();
    for y in 0..h {
        for x in 0..w {
            let k = keys.at(x as usize, y as usize);
            if k == 0 {
                continue;
            }
            let e = boxes.entry(k).or_insert((Rect::new(x, y, x + 1, y + 1), 0));
            e.0 = Rect::new(
                e.0.x0.min(x),
                e.0.y0.min(y),
                e.0.x1.max(x + 1),
                e.0.y1.max(y + 1),
            );
            e.1 += 1;
        }
    }
    let frame = Rect::new(0, 0, w, h);
    let mut out = Vec::new();
    for (&key, &(own, pixels)) in &boxes {
        if pixels < reading.min_pixels {
            continue;
        }
        let window = own.grown(reading.margin);
        let outer = window.grown(reading.ring);
        let inside = outer.x0 >= 0 && outer.y0 >= 0 && outer.x1 <= frame.x1 && outer.y1 <= frame.y1;
        if !inside || interface.iter().any(|r| r.meets(&outer)) {
            continue;
        }
        let theirs = read_window(keys, photo, window, reading, reader);
        let mut f = Footprint {
            key,
            window,
            ours: 0,
            theirs: 0,
            both: 0,
        };
        let ww = (window.x1 - window.x0) as usize;
        for y in window.y0..window.y1 {
            for x in window.x0..window.x1 {
                let o = keys.at(x as usize, y as usize) != 0;
                let t = theirs[(y - window.y0) as usize * ww + (x - window.x0) as usize];
                f.ours += u32::from(o);
                f.theirs += u32::from(t);
                f.both += u32::from(o && t);
            }
        }
        out.push(f);
    }
    out
}

/// One traced region of the photograph, and the key of the object of ours
/// it is traced as (item 4077). A key no object holds makes the region
/// every object's neighbour.
#[derive(Clone, Copy, Debug)]
pub struct Trace<'a> {
    pub key: u32,
    pub poly: &'a [(i32, i32)],
}

/// What a neighbour's traced region is to an object (item 4077;
/// `docs/RENDERER.md` §5.3, "A traced region").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neighbour {
    /// The rule: neither ground nor footprint, left out of both sides.
    Neither,
    /// The rejected reading: the photograph's scene, as colour would have
    /// it.
    Scene,
}

/// What the photograph's interface is to an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interface {
    /// The rule: its pixels are neither ground nor footprint, and an object
    /// is scored when 40 or more of its pixels lie outside it.
    Neither,
    /// Items 4038 to 4076: an object whose window or ring meets it is not
    /// scored.
    Skip,
}

/// **The traced reader** (item 4077; `docs/RENDERER.md` §5.3, "A traced
/// region"): each object's footprints, in key order, the photograph's side
/// read by kind. A pixel under the interface is neither (by `interface`);
/// one inside a trace of the object's own is its footprint; one inside a
/// neighbour's is neither (by `neighbour`); any other is read by colour
/// against the ring's pixels of that last kind where none of ours lies.
pub fn footprints_traced(
    keys: &Keys,
    photo: &Photo<'_>,
    interface: &[Rect],
    reading: &Reading,
    traces: &[Trace<'_>],
    neighbour: Neighbour,
    iface: Interface,
) -> Vec<Footprint> {
    assert_eq!((keys.width, keys.height), (photo.width, photo.height));
    let (w, h) = (keys.width as i32, keys.height as i32);
    let mut boxes: std::collections::BTreeMap<u32, (Rect, u32)> = Default::default();
    for y in 0..h {
        for x in 0..w {
            let k = keys.at(x as usize, y as usize);
            if k == 0 {
                continue;
            }
            let shown = iface == Interface::Skip || !interface.iter().any(|r| r.contains(x, y));
            let e = boxes.entry(k).or_insert((Rect::new(x, y, x + 1, y + 1), 0));
            e.0 = Rect::new(
                e.0.x0.min(x),
                e.0.y0.min(y),
                e.0.x1.max(x + 1),
                e.0.y1.max(y + 1),
            );
            e.1 += u32::from(shown);
        }
    }
    let frame = Rect::new(0, 0, w, h);
    let mut out = Vec::new();
    for (&key, &(own, pixels)) in &boxes {
        if pixels < reading.min_pixels {
            continue;
        }
        let window = own.grown(reading.margin);
        let outer = window.grown(reading.ring);
        let inside_frame =
            outer.x0 >= 0 && outer.y0 >= 0 && outer.x1 <= frame.x1 && outer.y1 <= frame.y1;
        if !inside_frame {
            continue;
        }
        if iface == Interface::Skip && interface.iter().any(|r| r.meets(&outer)) {
            continue;
        }
        out.push(read_traced(
            keys, photo, interface, reading, traces, neighbour, iface, key, window,
        ));
    }
    out
}

/// One window read by [`footprints_traced`]'s rule, as the object `key`
/// sees it: a key no object holds reads every trace as a neighbour's.
#[allow(clippy::too_many_arguments)]
pub fn read_traced(
    keys: &Keys,
    photo: &Photo<'_>,
    interface: &[Rect],
    reading: &Reading,
    traces: &[Trace<'_>],
    neighbour: Neighbour,
    iface: Interface,
    key: u32,
    window: Rect,
) -> Footprint {
    let theirs = traced_theirs(
        keys, photo, interface, reading, traces, neighbour, iface, key, window,
    );
    let ww = (window.x1 - window.x0) as usize;
    let mut f = Footprint {
        key,
        window,
        ours: 0,
        theirs: 0,
        both: 0,
    };
    for y in window.y0..window.y1 {
        for x in window.x0..window.x1 {
            let Some(t) = theirs[(y - window.y0) as usize * ww + (x - window.x0) as usize] else {
                continue;
            };
            let o = keys.at(x as usize, y as usize) != 0;
            f.ours += u32::from(o);
            f.theirs += u32::from(t);
            f.both += u32::from(o && t);
        }
    }
    f
}

/// [`read_traced`]'s photograph side, pixel by pixel over `window`
/// (row-major): `None` where the rule leaves the pixel out of both sides,
/// else whether it is the original's footprint. The colour reader
/// ([`read_colours`], item 4099) reads its pixels off this.
#[allow(clippy::too_many_arguments)]
pub fn traced_theirs(
    keys: &Keys,
    photo: &Photo<'_>,
    interface: &[Rect],
    reading: &Reading,
    traces: &[Trace<'_>],
    neighbour: Neighbour,
    iface: Interface,
    key: u32,
    window: Rect,
) -> Vec<Option<bool>> {
    let outer = window.grown(reading.ring);
    // 0 elsewhere, 1 its own trace, 2 a neighbour's, 3 the interface; its
    // own trace wins where two overlap.
    let kind = |x: i32, y: i32| -> u8 {
        if iface == Interface::Neither && interface.iter().any(|r| r.contains(x, y)) {
            return 3;
        }
        let mut k = 0;
        for t in traces {
            if inside(t.poly, x, y) {
                if t.key == key {
                    return 1;
                }
                k = 2;
            }
        }
        k
    };
    let g = ground_where(keys, photo, window, outer, reading, |x, y| kind(x, y) == 0);
    let mut out = Vec::with_capacity(((window.x1 - window.x0) * (window.y1 - window.y0)) as usize);
    for y in window.y0..window.y1 {
        for x in window.x0..window.x1 {
            out.push(match (kind(x, y), neighbour) {
                (3, _) | (2, Neighbour::Neither) => None,
                (1, _) | (2, Neighbour::Scene) => Some(true),
                _ => Some(!g.holds(photo, x, y)),
            });
        }
    }
    out
}

/// How the photograph's side of a window is read.
#[derive(Clone, Copy, Debug)]
pub enum Reader {
    /// Item 4038's: a pixel is the original's when its colour is none the
    /// ring holds where no object of ours is ([`ground`]).
    Colour,
    /// Item 4071's: a pixel is the original's when it stands out of the
    /// ring's robust statistics ([`Robust`]). It reads the photograph and
    /// the window's rectangle only, never our key buffer.
    Robust(Robust),
    /// Item 4071's second try: a pixel is the original's when its centre
    /// lies inside one of these polygons, traced by hand off the photograph
    /// in its own pixels ([`inside`]). It reads neither our key buffer nor
    /// the photograph's colours.
    Mask(&'static [&'static [(i32, i32)]]),
}

/// Whether pixel `(x, y)`'s centre lies inside `poly` (crossing number, in
/// half-pixels so the arithmetic is exact).
pub fn inside(poly: &[(i32, i32)], x: i32, y: i32) -> bool {
    let (px, py) = (i64::from(2 * x + 1), i64::from(2 * y + 1));
    let mut odd = false;
    for i in 0..poly.len() {
        let (ax, ay) = (i64::from(2 * poly[i].0), i64::from(2 * poly[i].1));
        let j = (i + 1) % poly.len();
        let (bx, by) = (i64::from(2 * poly[j].0), i64::from(2 * poly[j].1));
        if (ay > py) != (by > py) {
            // x of the edge at py, compared without dividing.
            let lhs = (px - ax) * (by - ay);
            let rhs = (bx - ax) * (py - ay);
            if (by > ay && lhs < rhs) || (by < ay && lhs > rhs) {
                odd = !odd;
            }
        }
    }
    odd
}

/// The photograph's footprint in `window`, row-major, the window's width a
/// row: `true` where the reader sees something that is not the ground.
pub fn read_window(
    keys: &Keys,
    photo: &Photo<'_>,
    window: Rect,
    reading: &Reading,
    reader: Reader,
) -> Vec<bool> {
    let outer = window.grown(reading.ring);
    match reader {
        Reader::Colour => {
            let g = ground(keys, photo, window, outer, reading);
            let mut out = Vec::new();
            for y in window.y0..window.y1 {
                for x in window.x0..window.x1 {
                    out.push(!g.holds(photo, x, y));
                }
            }
            out
        }
        Reader::Robust(r) => robust(photo, window, outer, &r),
        Reader::Mask(polys) => {
            let mut out = Vec::new();
            for y in window.y0..window.y1 {
                for x in window.x0..window.x1 {
                    out.push(polys.iter().any(|p| inside(p, x, y)));
                }
            }
            out
        }
    }
}

/// **The second reader's constants** (item 4071; `docs/RENDERER.md` §5.3),
/// written down before any count of it was read and not moved after.
///
/// The ground is the ring round the window, as for [`Reader::Colour`], but
/// summarised by statistics a minority cannot move: each channel's median
/// and median absolute deviation over the ring, every ring pixel taken,
/// none excluded by our draw. The colour reader keeps the union of the
/// ring's colours, so a handful of an object's own pixels in the ring —
/// a tank of the photograph standing past a window placed by ours — make
/// that object's colours the ground's.
#[derive(Clone, Copy, Debug)]
pub struct Robust {
    /// Every pixel is first replaced by the mean of the box `2·smooth + 1`
    /// a side round it: the grass's speckle is a texture, not an object.
    pub smooth: i32,
    /// A pixel is a candidate when one channel stands further from the
    /// ring's median than `sigmas_x10 / 10` deviations, a deviation being
    /// `1.4826 ×` the median absolute deviation (a normal's σ).
    pub sigmas_x10: i64,
    /// The median absolute deviation's floor, in channel levels, so a ring
    /// of one flat colour does not read every shade as an object.
    pub mad_floor: i32,
    /// A candidate is kept when this many of the 3×3 round it, itself
    /// among them, are candidates: a lone pixel is noise.
    pub majority: u32,
}

/// The second reader as the pinned frame is read at: a 3×3 mean, 3σ, a
/// floor of 4 levels, a majority of 5 of 9.
pub const ROBUST: Robust = Robust {
    smooth: 1,
    sigmas_x10: 30,
    mad_floor: 4,
    majority: 5,
};

fn robust(photo: &Photo<'_>, window: Rect, outer: Rect, r: &Robust) -> Vec<bool> {
    let (w, h) = (photo.width as i32, photo.height as i32);
    let mean = |x: i32, y: i32| -> [i32; 3] {
        let mut sum = [0i32; 3];
        let mut n = 0;
        for yy in (y - r.smooth).max(0)..=(y + r.smooth).min(h - 1) {
            for xx in (x - r.smooth).max(0)..=(x + r.smooth).min(w - 1) {
                let i = (yy as usize * photo.width + xx as usize) * 4;
                for (c, s) in sum.iter_mut().enumerate() {
                    *s += i32::from(photo.rgba[i + c]);
                }
                n += 1;
            }
        }
        sum.map(|s| (s + n / 2) / n)
    };
    let mut ring: [Vec<i32>; 3] = Default::default();
    for y in outer.y0..outer.y1 {
        for x in outer.x0..outer.x1 {
            if window.contains(x, y) {
                continue;
            }
            let m = mean(x, y);
            for c in 0..3 {
                ring[c].push(m[c]);
            }
        }
    }
    let median = |v: &mut Vec<i32>| {
        v.sort_unstable();
        v[v.len() / 2]
    };
    let mut centre = [0; 3];
    let mut mad = [0; 3];
    for c in 0..3 {
        centre[c] = median(&mut ring[c]);
        let mut dev: Vec<i32> = ring[c].iter().map(|v| (v - centre[c]).abs()).collect();
        mad[c] = median(&mut dev).max(r.mad_floor);
    }
    let (ww, wh) = (window.x1 - window.x0, window.y1 - window.y0);
    let mut cand = vec![false; (ww * wh) as usize];
    for y in 0..wh {
        for x in 0..ww {
            let m = mean(window.x0 + x, window.y0 + y);
            cand[(y * ww + x) as usize] = (0..3).any(|c| {
                i64::from((m[c] - centre[c]).abs()) * 100_000
                    > r.sigmas_x10 * 14_826 * i64::from(mad[c])
            });
        }
    }
    let mut out = vec![false; cand.len()];
    for y in 0..wh {
        for x in 0..ww {
            let mut n = 0;
            for yy in (y - 1).max(0)..=(y + 1).min(wh - 1) {
                for xx in (x - 1).max(0)..=(x + 1).min(ww - 1) {
                    n += u32::from(cand[(yy * ww + xx) as usize]);
                }
            }
            out[(y * ww + x) as usize] = n >= r.majority;
        }
    }
    out
}

/// The ground round a window: the colours its ring holds where no object
/// of ours is, binned and dilated by the reading's tolerance.
pub struct Ground {
    bits: u32,
    bins: i32,
    table: Vec<bool>,
}

impl Ground {
    fn bin(&self, photo: &Photo<'_>, x: i32, y: i32) -> [i32; 3] {
        let i = (y as usize * photo.width + x as usize) * 4;
        [0, 1, 2].map(|c| i32::from(photo.rgba[i + c]) >> self.bits)
    }

    /// Whether the photograph's pixel is a colour of the ground.
    pub fn holds(&self, photo: &Photo<'_>, x: i32, y: i32) -> bool {
        let c = self.bin(photo, x, y);
        self.table[((c[0] * self.bins + c[1]) * self.bins + c[2]) as usize]
    }
}

/// The [`Ground`] of `window`, read off the ring between it and `outer`.
pub fn ground(
    keys: &Keys,
    photo: &Photo<'_>,
    window: Rect,
    outer: Rect,
    reading: &Reading,
) -> Ground {
    ground_where(keys, photo, window, outer, reading, |_, _| true)
}

/// [`ground`], sampling only the ring's pixels `sampled` admits.
pub fn ground_where(
    keys: &Keys,
    photo: &Photo<'_>,
    window: Rect,
    outer: Rect,
    reading: &Reading,
    sampled: impl Fn(i32, i32) -> bool,
) -> Ground {
    let bins = 256 >> reading.bin_bits;
    let mut g = Ground {
        bits: reading.bin_bits,
        bins,
        table: vec![false; (bins * bins * bins) as usize],
    };
    let t = reading.tolerance;
    for y in outer.y0..outer.y1 {
        for x in outer.x0..outer.x1 {
            if window.contains(x, y) || keys.at(x as usize, y as usize) != 0 || !sampled(x, y) {
                continue;
            }
            let c = g.bin(photo, x, y);
            for r in (c[0] - t).max(0)..=(c[0] + t).min(bins - 1) {
                for gg in (c[1] - t).max(0)..=(c[1] + t).min(bins - 1) {
                    for b in (c[2] - t).max(0)..=(c[2] + t).min(bins - 1) {
                        g.table[((r * bins + gg) * bins + b) as usize] = true;
                    }
                }
            }
        }
    }
    g
}

/// How many footprints reach the reading's threshold, of how many scored.
pub fn score(footprints: &[Footprint], reading: &Reading) -> (usize, usize) {
    let k = footprints
        .iter()
        .filter(|f| f.overlap() >= reading.threshold)
        .count();
    (k, footprints.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Item 4103: the silhouette's pixel — the owner's colour at 128/255
    /// over the nearer pixel.
    #[test]
    fn the_silhouette_is_the_owner_s_colour_at_half_over_the_nearer_pixel() {
        // Over black: the owner's colour at 128/255.
        let red = silhouette_blend([255, 0, 0], [0.0; 3]);
        assert!((red[0] - 128.0).abs() < 1e-3 && red[1] == 0.0 && red[2] == 0.0);
        // Over the owner's own colour: unchanged.
        let blue = silhouette_blend(TEAM_UNIT[1], [34.0, 82.0, 255.0]);
        assert!((0..3).all(|k| (blue[k] - f32::from(TEAM_UNIT[1][k])).abs() < 1e-3));
        // Over a leaf: between the two, a hair nearer the owner.
        let leaf = [100.0, 120.0, 40.0];
        let p = silhouette_blend([255, 0, 0], leaf);
        assert!((p[0] - (255.0 * 128.0 + 100.0 * 127.0) / 255.0).abs() < 1e-3);
        assert!((p[1] - 120.0 * 127.0 / 255.0).abs() < 1e-3);
    }

    /// Item 4102: the team colour's ends and its middle — alpha 255 the
    /// texel, 0 the owner's colour, half between; past the eight, white.
    #[test]
    fn the_team_colour_through_the_texel_s_alpha() {
        let t = [100.0, 120.0, 140.0];
        assert_eq!(team_blend(t, 255.0, team_unit(1)), t);
        assert_eq!(team_blend(t, 0.0, team_unit(1)), [34.0, 82.0, 255.0]);
        let half = team_blend(t, 127.5, team_unit(0));
        assert_eq!(half, [177.5, 60.0, 70.0]);
        assert_eq!(team_unit(8), [255, 255, 255]);
        assert_eq!(TEAM_UNIT[1], [34, 82, 255]);
    }

    /// The draw list's test camera (`draw::tests`), closer in.
    fn camera() -> Camera {
        Camera {
            zoom_distance: 20858.805,
            camera_scale: 1.0,
            tilt_angle: 60.0,
            dir_angle: 45.0,
            loc_x: 9552.0,
            loc_y: 10320.0,
            loc_x_adjust: 0.0,
            loc_y_adjust: 0.0,
            loc_z_adjust: 0.0,
            angle: 45.0,
            near_plane: 6589.4,
            far_plane: 36262.1,
            view_ratio: 0.5625,
            parallel: true,
            parallel_distance: 1500.0,
        }
    }

    fn square(key: u32, at: [f32; 2], side: f32) -> Object {
        let [x, y] = at;
        let p = |dx: f32, dy: f32| [x + dx, y + dy, 0.0];
        Object {
            key,
            tris: vec![
                [p(0.0, 0.0), p(side, 0.0), p(side, side)],
                [p(0.0, 0.0), p(side, side), p(0.0, side)],
            ],
        }
    }

    /// **A sprite is stamped where it covers, at its scale** (item 4076): a
    /// 2×2 image with one corner uncovered, at scale 3 from (10, 20), keys
    /// 3×3 blocks and leaves the corner's empty; a mesh square nearer than
    /// the stamp's depth takes its pixels, one further loses them.
    #[test]
    fn a_sprite_keys_its_covered_pixels_at_its_scale_and_depth() {
        let cam = camera();
        let covered = [true, true, true, false];
        let stamp = |depth: f32| Stamp {
            key: 7,
            at: crate::draw::SpriteAt {
                left: 10,
                top: 20,
                scale: 3.0,
            },
            width: 2,
            height: 2,
            covered: &covered,
            depth,
        };
        let keys = rasterize_with(&[], &[stamp(0.0)], &cam, (40, 40));
        let at = |x, y| keys.at(x, y);
        assert_eq!(keys.keys.iter().filter(|k| **k == 7).count(), 27);
        assert_eq!((at(10, 20), at(15, 22), at(12, 25)), (7, 7, 7));
        assert_eq!((at(13, 23), at(15, 25)), (0, 0), "the uncovered corner");
        assert_eq!((at(9, 20), at(16, 20), at(10, 26)), (0, 0, 0));
        // Beside a mesh: the nearer wins.
        let target = cam.target();
        let big = square(1, [target[0] - 4000.0, target[1] - 4000.0], 8000.0);
        let mesh = rasterize(std::slice::from_ref(&big), &cam, (40, 40));
        assert_eq!(mesh.at(10, 20), 1, "the mesh covers the stamp's place");
        let r = cam.render_matrix().0;
        let depth = r[2] * target[0] + r[6] * target[1] + r[14];
        let near = rasterize_with(
            std::slice::from_ref(&big),
            &[stamp(depth - 1000.0)],
            &cam,
            (40, 40),
        );
        let far = rasterize_with(
            std::slice::from_ref(&big),
            &[stamp(depth + 1000.0)],
            &cam,
            (40, 40),
        );
        assert_eq!((near.at(10, 20), far.at(10, 20)), (7, 1));
    }

    /// The photograph of the scene our buffer draws, with a textured ground
    /// of two colours and the objects in a third: every footprint agrees.
    /// Then the photograph's object moved a window's width: the overlap
    /// falls to zero — the comparison can fail.
    #[test]
    fn a_silhouette_agrees_where_the_photograph_has_it_and_not_elsewhere() {
        let cam = camera();
        let size = (160, 120);
        let target = cam.target();
        let objects = vec![square(1, [target[0], target[1]], 400.0)];
        let keys = rasterize(&objects, &cam, size);
        let covered = keys.keys.iter().filter(|k| **k != 0).count();
        assert!(covered > 40, "the square covers {covered} pixels");
        let paint = |shift: i32| {
            let mut rgba = Vec::with_capacity(size.0 * size.1 * 4);
            for y in 0..size.1 {
                for x in 0..size.0 {
                    let sx = x as i32 - shift;
                    let on = sx >= 0 && (sx as usize) < size.0 && keys.at(sx as usize, y) != 0;
                    let c = if on {
                        [200, 40, 40]
                    } else if (x + y) % 2 == 0 {
                        [90, 140, 50]
                    } else {
                        [110, 150, 60]
                    };
                    rgba.extend([c[0], c[1], c[2], 255]);
                }
            }
            rgba
        };
        let same = paint(0);
        let photo = Photo {
            width: size.0,
            height: size.1,
            rgba: &same,
        };
        let f = footprints(&keys, &photo, &[], &READING);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].overlap(), 1.0, "{:?}", f[0]);
        assert_eq!(score(&f, &READING), (1, 1));

        let width = f[0].window.x1 - f[0].window.x0;
        let moved = paint(width);
        let photo = Photo {
            width: size.0,
            height: size.1,
            rgba: &moved,
        };
        let f = footprints(&keys, &photo, &[], &READING);
        assert!(f[0].overlap() < 0.05, "{:?}", f[0]);
        assert_eq!(score(&f, &READING), (0, 1));

        // An interface rectangle over the object takes it out of the score.
        let f = footprints(&keys, &photo, &[f[0].window], &READING);
        assert!(f.is_empty());
    }

    /// A traced polygon covers the pixels whose centres it holds, either
    /// winding: a 4×4 square 16, a right triangle of legs 4 the 6 centres
    /// strictly under its hypotenuse.
    /// The traced reader's four kinds (item 4077): a 40×40 green photograph
    /// holding a red square, object 1 of ours drawn on it and traced; a
    /// neighbour's trace over part of its window, and an interface band.
    #[test]
    fn the_traced_reader_reads_each_kind_of_pixel_by_its_rule() {
        let (w, h) = (40usize, 40usize);
        let mut rgba = vec![0u8; w * h * 4];
        let mut keys = vec![0u32; w * h];
        for y in 0..h {
            for x in 0..w {
                let red = (14..26).contains(&x) && (14..26).contains(&y);
                let i = (y * w + x) * 4;
                rgba[i..i + 4].copy_from_slice(&if red {
                    [200, 0, 0, 255]
                } else {
                    [0, 160, 0, 255]
                });
                if red {
                    keys[y * w + x] = 1;
                }
            }
        }
        let photo = Photo {
            width: w,
            height: h,
            rgba: &rgba,
        };
        let keys = Keys {
            width: w,
            height: h,
            keys,
        };
        let own: &[(i32, i32)] = &[(14, 14), (26, 14), (26, 26), (14, 26)];
        let theirs: &[(i32, i32)] = &[(14, 22), (26, 22), (26, 26), (14, 26)];
        let read = |traces: &[Trace<'_>], n, i, iface: &[Rect]| {
            footprints_traced(&keys, &photo, iface, &READING, traces, n, i)[0]
        };
        // Its own trace: footprint, all 144 pixels on both sides.
        let f = read(
            &[Trace { key: 1, poly: own }],
            Neighbour::Neither,
            Interface::Neither,
            &[],
        );
        assert_eq!((f.ours, f.theirs, f.both), (144, 144, 144));
        // A neighbour's over its bottom four rows: neither, or the scene.
        let both = [
            Trace { key: 1, poly: own },
            Trace {
                key: 9,
                poly: theirs,
            },
        ];
        let f = read(&both[1..], Neighbour::Neither, Interface::Neither, &[]);
        assert_eq!((f.ours, f.theirs, f.both), (96, 96, 96));
        let f = read(&both[1..], Neighbour::Scene, Interface::Neither, &[]);
        assert_eq!((f.ours, f.theirs, f.both), (144, 144, 144));
        // Its own trace wins where the two overlap.
        let f = read(&both, Neighbour::Neither, Interface::Neither, &[]);
        assert_eq!((f.ours, f.theirs, f.both), (144, 144, 144));
        // The interface over its top two rows: neither, or the object
        // skipped.
        let band = [Rect::new(0, 0, 40, 16)];
        let f = read(&[], Neighbour::Neither, Interface::Neither, &band);
        assert_eq!((f.ours, f.theirs, f.both), (120, 120, 120));
        assert!(
            footprints_traced(
                &keys,
                &photo,
                &band,
                &READING,
                &[],
                Neighbour::Neither,
                Interface::Skip
            )
            .is_empty()
        );
    }

    #[test]
    fn a_traced_polygon_holds_the_pixel_centres_inside_it() {
        let count = |poly: &[(i32, i32)]| {
            let mut n = 0;
            for y in 0..10 {
                for x in 0..10 {
                    n += u32::from(inside(poly, x, y));
                }
            }
            n
        };
        assert_eq!(count(&[(2, 2), (6, 2), (6, 6), (2, 6)]), 16);
        assert_eq!(count(&[(2, 6), (6, 6), (6, 2), (2, 2)]), 16);
        assert_eq!(count(&[(0, 0), (4, 0), (0, 4)]), 6);
        assert!(inside(&[(2, 2), (6, 2), (6, 6), (2, 6)], 2, 2));
        assert!(!inside(&[(2, 2), (6, 2), (6, 6), (2, 6)], 6, 2));
    }

    /// The nearer of two overlapping objects takes the pixel.
    #[test]
    fn the_nearer_object_takes_the_pixel() {
        let cam = camera();
        let t = cam.target();
        let mut low = square(1, [t[0], t[1]], 400.0);
        let mut high = square(2, [t[0], t[1]], 400.0);
        for tri in &mut low.tris {
            for p in tri.iter_mut() {
                p[2] = 0.0;
            }
        }
        for tri in &mut high.tris {
            for p in tri.iter_mut() {
                p[2] = 300.0;
            }
        }
        let size = (160, 120);
        let alone_low = rasterize(std::slice::from_ref(&low), &cam, size);
        let alone_high = rasterize(std::slice::from_ref(&high), &cam, size);
        let keys = rasterize(&[low, high], &cam, size);
        let shared: Vec<usize> = (0..keys.keys.len())
            .filter(|&i| alone_low.keys[i] != 0 && alone_high.keys[i] != 0)
            .collect();
        assert!(!shared.is_empty(), "the two squares overlap on screen");
        assert!(
            shared.iter().all(|&i| keys.keys[i] == 2),
            "the higher square is nearer"
        );
    }
}
