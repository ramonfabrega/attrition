//! **A tracked figure's tilt to the ground** (item 4089; `docs/RENDERER.md`
//! §3.7): the world's slope normal under a point, the eased per-guy normal
//! the draw keeps, and the rotation the draw builds from it.
//!
//! Written from the listings of `WorldData::find_slope_normal@006b5130`,
//! `GuyData::find_slope_normal@005de140`, `GuyOut::draw@005dc490`
//! (`5dc786`–`5dc94c`) and `GraphicPieces::draw_unit@008e7da0`'s unparented
//! path (`8e8878`–`8e88ef`).
//!
//! - **The world's normal** samples `TerrainOut::find_data_z(·, ·, 0)` at the
//!   point, 20 along x and 20 along y, and takes the cross product of the
//!   two edges `B × A` (`A = (20, 0, z₁ − z₀)`, `B = (0, 20, z₂ − z₀)`):
//!   `(20·dz₁, 20·dz₂, −400)`, normalized unless its square length is 0 or
//!   1. It points down; the guy's function negates it.
//! - **The guy's normal** is `last_norm` (`GuyData +0xa4`), eased a quarter
//!   of the way to that target at every call, which is every draw of the
//!   guy; the unset `(0, 0, −1)` takes the target whole. The draw tilts only
//!   when the eased z is below `0x3f7fbe77` (0.999).
//! - **The rotation** is about `n × ẑ = (n.y, −n.x, 0)`, normalized, by
//!   `acos(n.z)` in degrees truncated to a whole degree and negated, set by
//!   `Quat<float>::set` into `GraphicChad::misc_rot`. The axis is carried
//!   into the figure's frame first: `Point<float>::rotate` by the facing's
//!   quaternion (`fast_angle_to_degrees(get_render_angle) + 180`, wrapped
//!   to 0..359), then its y negated.
//! - **The draw** (`draw_unit`, a figure with no parent piece) starts its
//!   transform from `misc_rot` (`Transform<float>::init`), then
//!   `rotate_z((int)angle)`, then the translation: the tilt composes before
//!   the facing, both turn about the model's origin, and the origin is the
//!   guy's point. The mirror in y is the original's model-to-world map, as
//!   [`crate::figure::guy_placement`] keeps it.

use crate::camera::{DEG, Mat4, fill, half_degree, rotate, rotate_point, table_index};
use crate::terrain::Heightfield;
use crate::trees::data_z;

/// `GuyData::last_norm`'s unset value: z exactly −1 takes the target whole.
pub const UNSET: [f32; 3] = [0.0, 0.0, -1.0];

/// `0x3f7fbe77`: the draw tilts only below this eased z.
pub const LEVEL_Z: f32 = f32::from_bits(0x3f7f_be77);

/// `WorldData::find_slope_normal@006b5130` at `(x, y)`, on `field`: the
/// three heights it reads, and its normal (pointing down) in the listing's
/// single arithmetic and order.
pub fn world_normal(field: &Heightfield<'_>, x: i32, y: i32) -> ([i32; 3], [f32; 3]) {
    let z = [
        data_z(field, x, y),
        data_z(field, x + 20, y),
        data_z(field, x, y + 20),
    ];
    (z, normal_of(x, y, z))
}

/// The world normal from the three heights `world_normal` reads.
pub fn normal_of(x: i32, y: i32, z: [i32; 3]) -> [f32; 3] {
    let (x0, y0) = (x as f32, y as f32);
    let (x1, y2) = ((x + 20) as f32, (y + 20) as f32);
    let (z0, z1, z2) = (z[0] as f32, z[1] as f32, z[2] as f32);
    // A = p1 − p0, B = p2 − p0; the original's `y0 − y0` and `x0 − x0`
    // are exactly 0.
    let (ax, ay, az) = (x1 - x0, 0.0, z1 - z0);
    let (bx, by, bz) = (0.0, y2 - y0, z2 - z0);
    let n = [by * az - bz * ay, bz * ax - bx * az, bx * ay - by * ax];
    normalized(n)
}

/// `n` scaled by `1 / sqrt(|n|²)` unless `|n|²` is 0 or 1, as both
/// functions here do it (`sqrt` the CRT's double routine).
fn normalized(n: [f32; 3]) -> [f32; 3] {
    let l = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    if l == 0.0 || l == 1.0 {
        return n;
    }
    let inv = 1.0 / ((l as f64).sqrt() as f32);
    [n[0] * inv, n[1] * inv, n[2] * inv]
}

/// The guy's target: the world normal times −1.
pub fn target(world: [f32; 3]) -> [f32; 3] {
    [-world[0], -world[1], -world[2]]
}

/// One call of `GuyData::find_slope_normal@005de140`: `last` eased a quarter
/// of the way to `target` (taken whole when `last` is [`UNSET`]); returns
/// whether the draw tilts.
pub fn ease(last: &mut [f32; 3], target: [f32; 3]) -> bool {
    if last[2] == -1.0 {
        *last = target;
    } else {
        for k in 0..3 {
            last[k] += (target[k] - last[k]) * 0.25;
        }
    }
    last[2] < LEVEL_Z
}

/// The tilt's angle as the draw takes it, `acos(n.z) / DEG` in degrees,
/// before `Quat::set` truncates it.
pub fn tilt_degrees(n: [f32; 3]) -> f32 {
    let dot = (n[1] * 0.0 + n[0] * 0.0) + n[2];
    ((dot as f64).acos() as f32) / DEG
}

/// `GraphicChad::misc_rot` for the eased normal `n` and the figure's drawn
/// facing `drawn` (`fast_angle_to_degrees(get_render_angle) + 180`), as
/// `GuyOut::draw` builds it.
pub fn misc_rot(n: [f32; 3], drawn: i32) -> [f32; 4] {
    let axis = normalized([
        n[1] - n[2] * 0.0,
        n[2] * 0.0 - n[0],
        n[0] * 0.0 - n[1] * 0.0,
    ]);
    let theta = tilt_degrees(n);
    let d = table_index(drawn);
    let (c, s) = (half_degree(d, f64::cos), half_degree(d, f64::sin));
    let a = rotate_point(axis, [s * 0.0, s * 0.0, s, c]);
    let a = [a[0], -a[1], a[2]];
    let i = table_index(-(theta as i32));
    let (c, s) = (half_degree(i, f64::cos), half_degree(i, f64::sin));
    [a[0] * s, a[1] * s, a[2] * s, c]
}

/// A figure's model-to-world transform with its tilt: `misc_rot` (or none),
/// then the turn by `drawn` whole degrees about z, scaled by `scale`, the
/// result's y negated, moved to `at`. With no tilt it is
/// [`crate::figure::guy_placement`] in the half-degree tables' arithmetic.
pub fn tilted_placement(at: [i32; 3], drawn: i32, scale: f32, misc: Option<[f32; 4]>) -> Mat4 {
    let q = rotate(misc, drawn, [0.0, 0.0, 1.0]);
    let mut m = fill(q, [0.0; 3]).0;
    for c in 0..3 {
        for r in 0..3 {
            m[r * 4 + c] *= scale;
        }
    }
    for r in 0..3 {
        m[r * 4 + 1] = -m[r * 4 + 1];
    }
    m[12] = at[0] as f32;
    m[13] = at[1] as f32;
    m[14] = at[2] as f32;
    Mat4(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::{apply, guy_placement};

    fn angle(a: [f32; 3], b: [f32; 3]) -> f32 {
        let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        ((d / (la * lb)).clamp(-1.0, 1.0)).acos().to_degrees()
    }

    /// The original's world normal on run1559b's packet at 1/27's point
    /// (`tree_points.py --slope 4551,31679`): heights 563, 559, 563 and the
    /// normal's bits `be48d2ab 00000000 bf7b0756`.
    #[test]
    fn the_world_normal_is_the_original_s_on_its_heights() {
        let n = normal_of(4551, 31679, [563, 559, 563]);
        assert_eq!(n.map(f32::to_bits), [0xbe48_d2ab, 0x0000_0000, 0xbf7b_0756]);
        // 1/15's, 1/16's and 1/28's.
        let n = normal_of(5880, 30984, [423, 420, 420]);
        assert_eq!(n.map(f32::to_bits), [0xbe16_41a6, 0xbe16_41a6, 0xbf7a_6d69]);
        let n = normal_of(6358, 30770, [370, 366, 368]);
        assert_eq!(n.map(f32::to_bits), [0xbe47_dd45, 0xbdc7_dd45, 0xbf79_d496]);
        let n = normal_of(6170, 32078, [243, 239, 242]);
        assert_eq!(n.map(f32::to_bits), [0xbe48_94fd, 0xbd48_94fd, 0xbf7a_ba3d]);
        // A check that can fail: one height a unit off moves the normal.
        assert_ne!(
            normal_of(4551, 31679, [563, 560, 563]),
            normal_of(4551, 31679, [563, 559, 563])
        );
    }

    #[test]
    fn the_ease_takes_the_unset_whole_and_a_quarter_after() {
        let t = [0.2, 0.0, 0.98];
        let mut last = UNSET;
        assert!(ease(&mut last, t));
        assert_eq!(last, t);
        let mut last = [0.0, 0.0, 1.0];
        assert!(ease(&mut last, t), "0.995 is below the level z");
        assert_eq!(last, [0.05, 0.0, 1.0 + (0.98 - 1.0) * 0.25]);
        let mut last = [0.0, 0.0, 1.0];
        assert!(!ease(&mut last, [0.0, 0.0, 0.998]), "0.9995 is level");
    }

    /// With no tilt the placement is the figure's own map, `guy_placement`.
    #[test]
    fn level_is_the_figure_s_own_placement() {
        for drawn in [180, 0, 37, 271, 359, 400] {
            let a = tilted_placement([10, 20, 30], drawn, 1.5, None).0;
            let b = guy_placement([10, 20, 30], drawn, 1.5).0;
            for k in 0..16 {
                assert!(
                    (a[k] - b[k]).abs() < 1e-5,
                    "drawn {drawn} element {k}: {} {}",
                    a[k],
                    b[k]
                );
            }
        }
    }

    /// **The reading's check**: the tilt leans the model's up the way the
    /// eased normal leans, whatever the facing, by `⌊θ⌋ + 1` degrees —
    /// `θ = acos(n.z)` of the eased vector, which the ease leaves short of
    /// unit length, truncated by `Quat::set` and wrapped one degree further
    /// by its negative arm.
    #[test]
    fn the_tilt_stands_the_model_s_up_on_the_normal() {
        let normals = [
            [0.183_931_93, 0.056_763_086, 0.980_415_2],
            [0.170_600_97, 0.108_512_71, 0.976_893_8],
            [-0.3, 0.2, 0.932_737_9],
            [0.0, -0.25, 0.968_245_8],
        ];
        for n in normals {
            let theta = tilt_degrees(n);
            for facing in (0..360).step_by(15) {
                let drawn = facing + 180;
                let m = tilted_placement([0; 3], drawn, 1.0, Some(misc_rot(n, drawn)));
                let up = apply(&m, [0.0, 0.0, 1.0]);
                let off = angle([up[0], up[1], 0.0], [n[0], n[1], 0.0]);
                let drawn_tilt = angle(up, [0.0, 0.0, 1.0]);
                assert!(
                    off < 0.05,
                    "n {n:?} facing {facing}: up {up:?} leans {off}° off n's way"
                );
                assert!(
                    (drawn_tilt - (theta.trunc() + 1.0)).abs() < 1e-2,
                    "n {n:?} facing {facing}: tilt {drawn_tilt}, θ {theta}"
                );
                // Perturbed — the facing's turn applied after the tilt the
                // other way round — the check fails.
                let wrong = Some(misc_rot(n, drawn + 90));
                let up = apply(
                    &tilted_placement([0; 3], drawn, 1.0, wrong),
                    [0.0, 0.0, 1.0],
                );
                let off = angle([up[0], up[1], 0.0], [n[0], n[1], 0.0]);
                assert!(off > 45.0, "a wrong facing still leans n's way");
            }
        }
    }
}
