//! The original's camera: its state, the matrix it builds, and the screen
//! point it gives a world position (`docs/RENDERER.md` §3.4).
//!
//! Written from reading `Camera::update_transforms@008430a0`,
//! `BaseCamera::update_transforms@00537cc0`, the graphics DLL's
//! `D3D11GLSys::set_projection` / `set_projection_parallel` (`d3dgl.dll`,
//! whose own PDB names them) and `MiscAccess::world_to_console@00583840` /
//! `world_to_screen@00583810`. The arithmetic is single precision in the
//! original's order, so a point lands on the original's pixel rather than
//! next to it; the matrices are row-vector (`p' = p · M`, element `r*4 + c`),
//! as the original keeps them.

/// The degree, as the half-degree tables' builder keeps it (`0x3c8efa35`).
pub(crate) const DEG: f32 = 0.017_453_292;

/// A 4×4 matrix, row-major, row vectors (`p' = p · M`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [f32; 16]);

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4([
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// `a · b`: apply `a`, then `b`. `Matrix<float>::operator*` is called as
    /// `b.operator*(a)` and sums each element left to right over `k`.
    pub fn then(a: &Mat4, b: &Mat4) -> Mat4 {
        let (a, b) = (&a.0, &b.0);
        let mut m = [0.0f32; 16];
        for r in 0..4 {
            for c in 0..4 {
                m[r * 4 + c] = b[c] * a[r * 4]
                    + b[4 + c] * a[r * 4 + 1]
                    + b[8 + c] * a[r * 4 + 2]
                    + b[12 + c] * a[r * 4 + 3];
            }
        }
        Mat4(m)
    }
}

/// The camera's state: the fields of the PDB's `Camera` (size `0x370`) and
/// `BaseCamera` (`0x288`) that `update_transforms` reads, by their names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// `Camera::zoom_distance` (`+0x2f0`): the eye's distance back from the
    /// look-at point along the view axis.
    pub zoom_distance: f32,
    /// The global `camera_scale`, which multiplies `zoom_distance` for the
    /// game's own camera (`MiscAccess::camera`) only; 1.0 for any other.
    pub camera_scale: f32,
    /// `tilt_angle` (`+0x2f4`), degrees; the eye rotates by `tilt + 180`
    /// about x, truncated to a whole degree.
    pub tilt_angle: f32,
    /// `dir_angle` (`+0x2f8`), degrees about z, truncated to a whole degree.
    pub dir_angle: f32,
    /// `loc_x`, `loc_y` (`+0x304`, `+0x308`): the look-at point.
    pub loc_x: f32,
    pub loc_y: f32,
    /// `loc_x_adjust`, `loc_y_adjust`, `loc_z_adjust` (`+0x30c`..`+0x314`).
    pub loc_x_adjust: f32,
    pub loc_y_adjust: f32,
    pub loc_z_adjust: f32,
    /// `BaseCamera::angle` (`+0x12c`): the field of view, degrees.
    pub angle: f32,
    /// `near_plane`, `far_plane` (`+0x130`, `+0x134`).
    pub near_plane: f32,
    pub far_plane: f32,
    /// `view_ratio` (`+0x13c`): height over width.
    pub view_ratio: f32,
    /// `parallel` (`+0x280`): the orthographic projection, which the game's
    /// camera uses.
    pub parallel: bool,
    /// `parallel_distance` (`+0x284`): the orthographic scale.
    pub parallel_distance: f32,
}

/// What `world_to_screen` reads besides the camera: `graphsys`'s
/// `scaled_screen_xs` / `scaled_screen_ys` (`+0x360`, `+0x364`) and
/// `natural_ratio` (`+0x16c`), and the console's short at `+0x12a`, which
/// it adds to the screen y.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub scaled_xs: i32,
    pub scaled_ys: i32,
    pub natural_ratio: f32,
    pub console_y: i16,
}

/// A projected point: the pixel, and `world_to_console`'s return — set only
/// when the caller passed no size and the point falls outside [-1, 1].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projected {
    pub x: i32,
    pub y: i32,
    pub off_screen: bool,
}

/// A rotation as `Transform<float>` keeps one: `None` until a rotation is
/// applied (the type byte's rotation bits clear).
pub(crate) type Quat = Option<[f32; 4]>; // x, y, z, w

/// The half-degree tables' index for a whole-degree angle, as `Quat::set`
/// (`00420c70`) reduces it: `a mod 360` for a non-negative angle, and
/// `359 − (−a mod 360)` for a negative one — one degree further round than
/// `360 − (−a mod 360)` would be.
pub(crate) fn table_index(a: i32) -> i32 {
    if a < 0 {
        359 - (a.unsigned_abs() % 360) as i32
    } else {
        a % 360
    }
}

/// `fast_half_degree_to_cosine` / `_sine`: `cosf(i · ½ · DEG)`, where the
/// original's `cosf` is the double routine between two conversions.
pub(crate) fn half_degree(i: i32, f: fn(f64) -> f64) -> f32 {
    let arg = (i as f32 * 0.5) * DEG;
    f(arg as f64) as f32
}

/// `Transform<float>::rotate_x` / `rotate_z` by a whole-degree angle about a
/// unit axis: a zero angle does nothing; a first rotation is set, a later
/// one composed as `existing ⊗ new`.
pub(crate) fn rotate(q: Quat, degrees: i32, axis: [f32; 3]) -> Quat {
    if degrees == 0 {
        return q;
    }
    let i = table_index(degrees);
    let (c, s) = (half_degree(i, f64::cos), half_degree(i, f64::sin));
    let b = [axis[0] * s, axis[1] * s, axis[2] * s, c];
    let Some(a) = q else { return Some(b) };
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    Some([
        (ay * bz - az * by) + aw * bx + ax * bw,
        (az * bx - ax * bz) + aw * by + ay * bw,
        (ax * by - ay * bx) + aw * bz + az * bw,
        aw * bw - (ax * bx + ay * by + az * bz),
    ])
}

/// `Point<float>::rotate`: `p` by `q`, as the row-vector matrix `fill`
/// builds from `q` would move it.
pub(crate) fn rotate_point(p: [f32; 3], q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = q;
    let [px, py, pz] = p;
    let a = z * z + y * y;
    let t0 = (z * x - w * y) * pz + (w * z + y * x) * py;
    let b = z * z + x * x;
    let t1 = (w * x + z * y) * pz + (y * x - w * z) * px;
    let c = y * y + x * x;
    let t2 = (z * y - w * x) * py + (w * y + z * x) * px;
    [
        (1.0 - (a + a)) * px + t0 + t0,
        (1.0 - (b + b)) * py + t1 + t1,
        (1.0 - (c + c)) * pz + t2 + t2,
    ]
}

/// `Transform<float>::fill_matrix` of a rigid transform: `Matrix::fill`'s
/// rotation (when there is one) and the translation in the last row.
pub(crate) fn fill(q: Quat, t: [f32; 3]) -> Mat4 {
    let mut m = Mat4::IDENTITY.0;
    if let Some([x, y, z, w]) = q {
        let (z2, y2) = (z + z, y + y);
        let xx2 = x * (x + x);
        let wx2 = w * (x + x);
        m[0] = 1.0 - (z * z2 + y * y2);
        m[4] = w * z2 + x * y2;
        m[1] = x * y2 - w * z2;
        m[8] = x * z2 - w * y2;
        m[2] = w * y2 + x * z2;
        m[5] = 1.0 - (z * z2 + xx2);
        m[9] = wx2 + y * z2;
        m[6] = y * z2 - wx2;
        m[10] = 1.0 - (y * y2 + xx2);
    }
    m[12] = t[0];
    m[13] = t[1];
    m[14] = t[2];
    Mat4(m)
}

/// The inverse of a rigid matrix, as `BaseCamera::update_transforms` takes
/// it in place: the rotation transposed, the translation `−t · Rᵀ`.
fn rigid_inverse(m: &Mat4) -> Mat4 {
    let o = m.0;
    let (tx, ty, tz) = (o[12], o[13], o[14]);
    let mut r = o;
    r[1] = o[4];
    r[2] = o[8];
    r[4] = o[1];
    r[8] = o[2];
    r[6] = o[9];
    r[9] = o[6];
    r[12] = -(o[1] * ty + o[0] * tx + o[2] * tz);
    r[13] = -(o[5] * ty + o[4] * tx + tz * o[6]);
    r[14] = -(o[9] * ty + o[8] * tx + o[10] * tz);
    Mat4(r)
}

/// `tan(angle / 2)` as the DLL takes it: the half angle in single
/// precision, the tangent in double, converted back.
fn tan_half(angle: f32) -> f32 {
    let half = (angle * DEG) * 0.5;
    (half as f64).tan() as f32
}

impl Camera {
    /// The look-at point the eye orbits: `loc + adjust`, at `loc_z_adjust`.
    pub fn target(&self) -> [f32; 3] {
        [
            self.loc_x_adjust + self.loc_x,
            self.loc_y_adjust + self.loc_y,
            self.loc_z_adjust,
        ]
    }

    /// The eye's transform (camera space → world), `axis_transform ∗ local`:
    /// back `zoom_distance · camera_scale` along the view axis, rotated by
    /// `tilt + 180` about x then `dir` about z, and moved to the target.
    fn eye(&self) -> (Quat, [f32; 3]) {
        let d = self.zoom_distance * self.camera_scale;
        let q = rotate(None, (self.tilt_angle + 180.0) as i32, [1.0, 0.0, 0.0]);
        let q = rotate(q, self.dir_angle as i32, [0.0, 0.0, 1.0]);
        let back = [0.0, 0.0, -d];
        let r = match q {
            Some(q) => rotate_point(back, q),
            None => back,
        };
        let t = self.target();
        (q, [t[0] + r[0], t[1] + r[1], t[2] + r[2]])
    }

    /// `render_matrix`: world → camera space.
    pub fn render_matrix(&self) -> Mat4 {
        let (q, t) = self.eye();
        rigid_inverse(&fill(q, t))
    }

    /// `projection_matrix`, from the DLL's `set_projection` (perspective) or
    /// `set_projection_parallel`. Either leaves the identity when the angle
    /// is not positive or the far plane is not beyond the near one.
    pub fn projection_matrix(&self) -> Mat4 {
        let mut m = Mat4::IDENTITY.0;
        let (near, far) = (self.near_plane, self.far_plane);
        if !(self.angle > 0.0 && far > near) {
            return Mat4(m);
        }
        let t = tan_half(self.angle);
        m[0] = 1.0 / t;
        m[5] = 1.0 / (t * self.view_ratio);
        if self.parallel {
            let pd = self.parallel_distance;
            m[10] = pd / (far - near);
            m[11] = 0.0;
            m[14] = -((near * pd) / (far - near));
            m[15] = pd;
        } else {
            let q = far / (far - near);
            m[10] = q;
            m[11] = 1.0;
            m[14] = -(q * near);
            m[15] = 0.0;
        }
        Mat4(m)
    }

    /// `to_screen_matrix`: `render_matrix · projection_matrix`.
    pub fn to_screen_matrix(&self) -> Mat4 {
        Mat4::then(&self.render_matrix(), &self.projection_matrix())
    }
}

/// `cvttss2si`: truncation, and `0x80000000` for what has no `i32`.
fn cvtt(v: f32) -> i32 {
    if v.is_nan() || !(-2_147_483_648.0..2_147_483_648.0).contains(&v) {
        i32::MIN
    } else {
        v as i32
    }
}

/// Truncate, then step up when the dropped fraction is at least a half —
/// which a negative coordinate's fraction never is.
fn round_up_half(v: f32) -> i32 {
    let i = cvtt(v);
    if v - i as f32 >= 0.5 {
        i.wrapping_add(1)
    } else {
        i
    }
}

/// `MiscAccess::world_to_console`: the integer world point through
/// `to_screen`, divided by `w` unless it is 0 or 1, onto a `size` pixel
/// screen — `None` (the original's `w < 1 || h < 1`) takes the width and
/// `height · natural_ratio` from `graphsys`.
pub fn world_to_console(
    m: &Mat4,
    screen: &Screen,
    x: i32,
    y: i32,
    z: i32,
    size: Option<(i32, i32)>,
) -> Projected {
    let m = &m.0;
    let (fx, fy, fz) = (x as f32, y as f32, z as f32);
    let mut sx = m[4] * fy + m[0] * fx + m[8] * fz + m[12];
    let mut sy = m[5] * fy + m[1] * fx + m[9] * fz + m[13];
    let w = m[7] * fy + m[3] * fx + m[11] * fz + m[15];
    if w != 0.0 && w != 1.0 {
        let r = 1.0 / w;
        sx *= r;
        sy *= r;
    }
    let size = size.filter(|&(w, h)| w >= 1 && h >= 1);
    // Four comparisons, not a range: a NaN is never off screen (`comiss`
    // leaves every `ja` untaken), where `!contains` would call it so.
    #[allow(clippy::manual_range_contains)]
    let off_screen = size.is_none() && (sx < -1.0 || sx > 1.0 || sy < -1.0 || sy > 1.0);
    let (w, h) = size.unwrap_or((
        screen.scaled_xs,
        cvtt(screen.scaled_ys as f32 * screen.natural_ratio),
    ));
    let px = (sx + 1.0) * w as f32 * 0.5;
    let py = (1.0 - sy) * h as f32 * 0.5;
    Projected {
        x: round_up_half(px),
        y: round_up_half(py),
        off_screen,
    }
}

/// `MiscAccess::world_to_screen`: `world_to_console`, then the console's
/// offset added to y.
pub fn world_to_screen(
    m: &Mat4,
    screen: &Screen,
    x: i32,
    y: i32,
    z: i32,
    size: Option<(i32, i32)>,
) -> Projected {
    let p = world_to_console(m, screen, x, y, z, size);
    Projected {
        y: p.y.wrapping_add(screen.console_y as i32),
        ..p
    }
}

#[cfg(test)]
// The packet's values as it printed them.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod tests {
    use super::*;

    /// run1484's packet (`~/ron-golden/run1484/map-18`, logger frame 676):
    /// the game's own camera, and the matrices the original built from it,
    /// read off the packet (`docs/RENDERER.md` §3.4).
    fn run1484() -> Camera {
        Camera {
            zoom_distance: 20858.805,
            camera_scale: 1.0,
            tilt_angle: 60.0,
            dir_angle: 45.0,
            loc_x: 9551.9384765625,
            loc_y: 10319.9384765625,
            loc_x_adjust: 0.0,
            loc_y_adjust: 0.0,
            loc_z_adjust: 0.0,
            angle: 45.0,
            near_plane: 6589.40234375,
            far_plane: 36262.10546875,
            view_ratio: 0.5630208253860474,
            parallel: true,
            parallel_distance: 10429.40234375,
        }
    }

    fn close(a: &Mat4, b: &[f32; 16], tol: f32) -> bool {
        a.0.iter()
            .zip(b)
            .all(|(x, y)| (x - y).abs() <= tol * (1.0 + y.abs()))
    }

    #[test]
    fn the_game_camera_builds_the_packets_matrices() {
        let c = run1484();
        let projection = [
            2.414213,
            0.0,
            0.0,
            0.0,
            0.0,
            4.287965,
            0.0,
            0.0,
            0.0,
            0.0,
            0.351481,
            0.0,
            0.0,
            0.0,
            -2316.052246,
            10429.402344,
        ];
        let render = [
            0.707107,
            -0.353553,
            -0.612373,
            0.0,
            -0.707107,
            -0.353553,
            -0.612372,
            0.0,
            0.0,
            0.866026,
            -0.5,
            0.0,
            543.062805,
            7025.768555,
            33027.796875,
            1.0,
        ];
        let to_screen = [
            1.707106,
            -1.516024,
            -0.215238,
            0.0,
            -1.707107,
            -1.516025,
            -0.215238,
            0.0,
            0.0,
            3.713487,
            -0.175741,
            0.0,
            1311.069458,
            30126.248047,
            9292.603516,
            10429.402344,
        ];
        assert!(
            close(&c.projection_matrix(), &projection, 1e-5),
            "{:?}",
            c.projection_matrix()
        );
        assert!(
            close(&c.render_matrix(), &render, 1e-4),
            "{:?}",
            c.render_matrix()
        );
        assert!(
            close(&c.to_screen_matrix(), &to_screen, 1e-4),
            "{:?}",
            c.to_screen_matrix()
        );
    }

    #[test]
    fn the_target_projects_to_the_screen_centre() {
        let c = run1484();
        let s = Screen {
            scaled_xs: 1920,
            scaled_ys: 1080,
            natural_ratio: 1.0,
            console_y: 0,
        };
        let m = c.to_screen_matrix();
        let p = world_to_screen(&m, &s, c.loc_x as i32, c.loc_y as i32, 0, None);
        assert!((p.x - 960).abs() <= 1 && (p.y - 540).abs() <= 1, "{p:?}");
        assert!(!p.off_screen);
    }

    #[test]
    fn a_negative_angle_reduces_one_degree_further_round() {
        assert_eq!(table_index(-1), 358);
        assert_eq!(table_index(-360), 359);
        assert_eq!(table_index(359), 359);
        assert_eq!(table_index(721), 1);
    }

    #[test]
    fn a_negative_fraction_never_rounds_up() {
        assert_eq!(round_up_half(2.5), 3);
        assert_eq!(round_up_half(2.49), 2);
        assert_eq!(round_up_half(-2.7), -2);
        assert_eq!(cvtt(f32::NAN), i32::MIN);
    }
}
