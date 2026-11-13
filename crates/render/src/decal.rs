//! **What stands under things** (item 4182; `docs/RENDERER.md` §3.7,
//! "The shadows"): a shadow is a decal, a flat quad in world space that
//! `DecalTransformMethod::execute@00923a70` lays from a
//! `DecalRDataStruct` (`rise.pdb`, size 0x2c) and
//! `DecalFlushMethod::execute@00923fd0` hands the card under an identity
//! transform. Read on the listing (`923af7`–`923e98`):
//!
//! - `a = cvttss2si(decal_shadow_angle)` folded into `0..360` as the
//!   listing folds it (`923bfd`–`923c30`): a negative `a` reads
//!   `359 − (−a mod 360)`, one short of the turn back;
//! - the quad's two axes `(1, 0, 0)` and `(0, 1, 0)` handed to
//!   `Point<float>::rotate@00420b10` under the quaternion `(0, 0, sin a/2,
//!   cos a/2)` (`fast_half_degree_to_cosine`/`_sine` at `a`). That is the
//!   rotation matrix times `v` (`sim::combat::quat_rotate`, diff-backed on
//!   run1471), and for this quaternion it turns by **−a**: `r1 = (cos a,
//!   −sin a)`, `r2 = (sin a, cos a)` (item 4193; 4182 read `+a`, which no
//!   tree could show, their `a` being 0);
//! - with `hw = width · 0.5`, `hh = height · 0.5` (`0xb694c0`), the four
//!   corners `x = loc.x + (±hw·r1.x ± hh·r2.x − xoffset) · xstretch`,
//!   `y = loc.y ± hw·r1.y ± hh·r2.y`, `z = loc.z` — the stretch and the
//!   offset on x alone;
//! - the texcoords the whole texture, or one quarter when
//!   `decal_shadow_corner` is 0 to 3 (the jump table at `0x923f84`).
//!
//! Two writers fill the record for frame 3279's shadows:
//! `GraphicPieces::draw_tree_shadow@00905910` (192 × 192, the tree's
//! `GraphicChad::angle`, 0 since `sort_trees` never turns one, stretch
//! 1.5, offset 192 · 0.2, corner `(int)(x + y) % 4`, `BLACK`) and
//! `GraphicPieces::draw_unit_shadow@008e8fa0` (the `UnitRDataStruct`'s
//! `shadow_width`/`shadow_height`, `unit_graphics.xml`'s `shadowwidth`
//! and `shadowheight` × 3, `init_unit_data@008f4290`; the angle `GuyOut::
//! draw_shadows@005dc070` hands it, `180 − fast_angle_to_degrees(
//! GuyData::get_render_angle)` (`5dc41b`–`5dc43c`, `0xb696c0` = 180);
//! stretch 1.25, offset 0.25, the whole texture, `BLACK` under the guy's
//! `transparent`).

/// One decal's record, `DecalRDataStruct`'s fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decal {
    pub loc: [f32; 3],
    pub width: f32,
    pub height: f32,
    /// Degrees as the writer stores them, folded where the quad is laid
    /// ([`Decal::turn`]).
    pub angle: f32,
    pub xstretch: f32,
    pub xoffset: f32,
    /// `None` the whole texture; `Some(0..=3)` one quarter.
    pub corner: Option<u8>,
    /// The vertex colour's alpha (`decal_shadow_color >> 24`).
    pub alpha: u8,
}

/// `draw_tree_shadow`'s width and height, `0x43400000`.
pub const TREE_SHADOW_SIZE: f32 = 192.0;
/// `draw_tree_shadow`'s `decal_shadow_xstretch`, `0x3fc00000`.
pub const TREE_SHADOW_XSTRETCH: f32 = 1.5;
/// `draw_tree_shadow`'s offset: the width times `0x3e4ccccd` (0.2).
pub const TREE_SHADOW_XOFFSET_SHARE: f32 = 0.2;
/// `draw_unit_shadow`'s `decal_shadow_xstretch`, `0x3fa00000`.
pub const UNIT_SHADOW_XSTRETCH: f32 = 1.25;
/// `draw_unit_shadow`'s `decal_shadow_xoffset`, `0x3e800000`.
pub const UNIT_SHADOW_XOFFSET: f32 = 0.25;
/// `init_unit_data`'s scale on `shadowwidth`/`shadowheight`,
/// `0x40400000`.
pub const UNIT_SHADOW_SCALE: f32 = 3.0;

/// The tree shadow's texture (`graphics.txt`'s tileset tree list's
/// eighth, the card's draw 119 on run1566b's pass after 3279).
pub const TREE_SHADOW_TEXTURE: &str = "tree_t_shadows.tga";

/// A tree's shadow at its placed point `at` (`ObjectsData::place_tree`).
pub fn tree_shadow(at: [i32; 3]) -> Decal {
    // `(int)(x + y) & 0x80000003` with the sign fixed up: C's `%`, whose
    // negative remainder is a `ulong` past 3, the switch's default.
    let r = (at[0] as f32 + at[1] as f32) as i32 % 4;
    Decal {
        loc: at.map(|v| v as f32),
        width: TREE_SHADOW_SIZE,
        height: TREE_SHADOW_SIZE,
        angle: 0.0,
        xstretch: TREE_SHADOW_XSTRETCH,
        xoffset: TREE_SHADOW_SIZE * TREE_SHADOW_XOFFSET_SHARE,
        corner: (r >= 0).then_some(r as u8),
        alpha: 255,
    }
}

/// A guy's shadow at `at`, its type's `shadowwidth` and `shadowheight`
/// as the XML writes them, its render angle's `fast_angle_to_degrees`
/// `degrees` (`0..360`): the decal's angle is `180 − degrees`, unfolded,
/// so negative for half the turns.
pub fn unit_shadow(at: [i32; 3], width: f32, height: f32, degrees: i32) -> Decal {
    Decal {
        loc: at.map(|v| v as f32),
        width: width * UNIT_SHADOW_SCALE,
        height: height * UNIT_SHADOW_SCALE,
        angle: 180.0 - degrees as f32,
        xstretch: UNIT_SHADOW_XSTRETCH,
        xoffset: UNIT_SHADOW_XOFFSET,
        corner: None,
        alpha: 255,
    }
}

impl Decal {
    /// The whole degree the quad turns by, `0..360`: the angle truncated
    /// (`cvttss2si`) and folded as `923bfd`–`923c30` fold it, a negative
    /// `a` to `359 − (−a mod 360)`.
    pub fn turn(&self) -> i32 {
        let a = self.angle as i32;
        if a < 0 {
            359 - (a.unsigned_abs() % 360) as i32
        } else {
            a % 360
        }
    }

    /// The four corners `Vert3Array::add` lays, in its order, each with
    /// its texcoord; faces `(0, 1, 2)` and `(2, 1, 3)`. The axes are
    /// `(1, 0)` and `(0, 1)` turned by `−turn` (module note).
    pub fn corners(&self) -> [([f32; 3], [f32; 2]); 4] {
        let a = (self.turn() as f32).to_radians();
        let (c, s) = (a.cos(), a.sin());
        let (hw, hh) = (self.width * 0.5, self.height * 0.5);
        let (ax, ay) = (hw * c, -hw * s);
        let (bx, by) = (hh * s, hh * c);
        let [x, y, z] = self.loc;
        let at = |sa: f32, sb: f32| {
            [
                x + (sa * ax + sb * bx - self.xoffset) * self.xstretch,
                y + sa * ay + sb * by,
                z,
            ]
        };
        let (u0, u1, v0, v1) = match self.corner {
            Some(0) => (0.0, 0.5, 0.0, 0.5),
            Some(1) => (0.5, 1.0, 0.0, 0.5),
            Some(2) => (0.0, 0.5, 0.5, 1.0),
            Some(3) => (0.5, 1.0, 0.5, 1.0),
            _ => (0.0, 1.0, 0.0, 1.0),
        };
        [
            (at(-1.0, -1.0), [u0, v0]),
            (at(1.0, -1.0), [u1, v0]),
            (at(-1.0, 1.0), [u0, v1]),
            (at(1.0, 1.0), [u1, v1]),
        ]
    }

    /// The quad as two triangles, `FaceArray::add`'s `(0, 1, 2)` and
    /// `(2, 1, 3)`: positions and texcoords, three a triangle.
    pub fn triangles(&self) -> ([[f32; 3]; 6], [[f32; 2]; 6]) {
        let k = self.corners();
        let order = [0, 1, 2, 2, 1, 3];
        (order.map(|i| k[i].0), order.map(|i| k[i].1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The listing's corners, worked by hand for a tree at (1000, 2000,
    /// 30): x from `1000 + (−96 − 38.4)·1.5` to `1000 + (96 − 38.4)·1.5`,
    /// y ± 96, z the point's; and `(1000 + 2000) % 4` = 0.
    #[test]
    fn a_tree_s_shadow_is_the_listing_s_quad() {
        let d = tree_shadow([1000, 2000, 30]);
        let k = d.corners();
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(&b).all(|(p, q)| (p - q).abs() < 1e-3);
        assert!(close(k[0].0, [798.4, 1904.0, 30.0]), "{:?}", k[0]);
        assert!(close(k[3].0, [1086.4, 2096.0, 30.0]), "{:?}", k[3]);
        assert_eq!((k[0].1, k[3].1), ([0.0, 0.0], [0.5, 0.5]));
        assert_eq!(tree_shadow([1001, 2000, 0]).corner, Some(1));
        assert_eq!(tree_shadow([1003, 2000, 0]).corner, Some(3));
    }

    /// A unit's shadow turned a quarter: its width lies along y.
    #[test]
    fn a_unit_s_shadow_turns_with_its_figure() {
        let d = unit_shadow([0, 0, 0], 50.0, 20.0, 90);
        assert_eq!((d.angle, d.turn()), (90.0, 90));
        let k = d.corners();
        let ys: Vec<f32> = k.iter().map(|c| c.0[1]).collect();
        let span = ys.iter().cloned().fold(f32::MIN, f32::max)
            - ys.iter().cloned().fold(f32::MAX, f32::min);
        assert!((span - 150.0).abs() < 1e-3, "{ys:?}");
    }

    /// The fold of `923bfd`–`923c30`: a negative angle reads one short of
    /// the turn back, as `missile_lead`'s fold does.
    #[test]
    fn a_negative_angle_folds_one_short() {
        let at = |angle: f32| {
            Decal {
                angle,
                ..unit_shadow([0; 3], 1.0, 1.0, 0)
            }
            .turn()
        };
        assert_eq!(
            [at(-90.0), at(-1.0), at(-360.0), at(-0.5)],
            [269, 358, 359, 0]
        );
        assert_eq!([at(0.0), at(180.0), at(359.9), at(360.0)], [0, 180, 359, 0]);
        // A figure at 200° lays its decal at 180 − 200 = −20, read 339.
        assert_eq!(unit_shadow([0; 3], 1.0, 1.0, 200).turn(), 339);
    }

    /// The turn's hand, worked by hand from `Point<float>::rotate`'s rows
    /// (`sim::combat::quat_rotate`) at a quarter: `(1, 0)` goes to `(0,
    /// −1)` and `(0, 1)` to `(1, 0)`. So the first corner, `(−hw, −hh)` in
    /// the decal's own axes, is `−hw·r1 − hh·r2 = (−hh, hw)`.
    #[test]
    fn a_turned_decal_turns_clockwise() {
        let d = Decal {
            xstretch: 1.0,
            xoffset: 0.0,
            ..unit_shadow([0; 3], 10.0, 20.0, 90)
        };
        let k = d.corners();
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(&b).all(|(p, q)| (p - q).abs() < 1e-3);
        assert!(close(k[0].0, [-30.0, 15.0, 0.0]), "{:?}", k[0]);
        assert!(close(k[3].0, [30.0, -15.0, 0.0]), "{:?}", k[3]);
    }
}
