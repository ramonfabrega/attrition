//! **The draw list** (`docs/RENDERER.md` §3.5, item 4026): what the client
//! draws on one frame, where, and in what order — headless, before a
//! pixel exists.
//!
//! The input is [`Placed`], one figure of an object as the simulation's
//! read door reports it (`sim::observe::FigureScene` and its unit), carried
//! across as plain data: this crate depends on nothing in `sim`, and the
//! crate that holds both (`rondata`) does the copy. The output is one
//! [`Drawn`] per figure the viewer sees — its object id, its screen anchor
//! from [`crate::camera`], a depth key, and the pose the figure carries
//! (piece, animation slot, frame, facing), which the simulation already
//! agrees with the original on and this list passes through untouched.
//!
//! **What is drawn** is the figures with a piece (`-1` draws nothing), and,
//! for a viewer, those on a half-cell its plane holds in sight now or that
//! it owns. Nothing is culled to the screen: the original's clip planes
//! (`Camera::compute_clip_planes@00842950`) are not read, so the list holds
//! every figure on the map, off-screen ones at their off-screen pixel.

use crate::camera::{Camera, Projected, Screen, world_to_screen};

/// `sim::observe::fog::VISIBLE`: the half-cell is in the viewer's sight
/// now. `rondata`'s draw test holds the two equal.
pub const FOG_VISIBLE: u8 = 2;

/// One figure of an object, as the read door reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// The object's identity, `(who, o)`: its owner and its number in the
    /// owner's object array (`sim::observe::ObjId`).
    pub who: u8,
    pub o: i16,
    /// The figure's index among its object's guys.
    pub figure: u16,
    /// The figure's own point and height (`FigureScene::pos`, `z`), in the
    /// world integers `MiscAccess::world_to_screen` takes.
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// The graphic piece, `-1` for none.
    pub piece: i32,
    /// The `UnitAnim` slot and the clock into it.
    pub slot: i8,
    pub frame: u32,
    /// The facing, a full turn in `2^32`, north zero.
    pub facing: u32,
}

/// What a viewer sees: its fog plane on the half-cell grid.
#[derive(Clone, Copy, Debug)]
pub struct Sight<'a> {
    pub viewer: u8,
    /// `sim::observe::Scene::fog`, row-major over `xs × ys`.
    pub plane: &'a [u8],
    pub xs: i32,
    pub ys: i32,
    /// A half-cell in world units (`sim::UNITS_PER_FOG`).
    pub cell: i32,
}

impl Sight<'_> {
    fn sees(&self, p: &Placed) -> bool {
        if p.who == self.viewer {
            return true;
        }
        let (fx, fy) = (p.x.div_euclid(self.cell), p.y.div_euclid(self.cell));
        fx >= 0
            && fy >= 0
            && fx < self.xs
            && fy < self.ys
            && self.plane.get((fy * self.xs + fx) as usize) == Some(&FOG_VISIBLE)
    }
}

/// One figure the viewer sees, placed on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drawn {
    pub who: u8,
    pub o: i16,
    pub figure: u16,
    /// The pixel `world_to_screen` gives the figure's point.
    pub anchor: Projected,
    /// The figure's depth in camera space, `render_matrix`'s third column:
    /// larger is further from the eye. The list is ordered on it, furthest
    /// first, ties by identity.
    pub depth: f32,
    pub piece: i32,
    pub slot: i8,
    pub frame: u32,
    pub facing: u32,
}

/// The draw list of `placed` at `camera` on `screen` (`size` as
/// [`world_to_screen`] takes it), as `sight`'s viewer sees it, or whole for
/// `None`: furthest first.
pub fn draw_list(
    placed: &[Placed],
    camera: &Camera,
    screen: &Screen,
    size: Option<(i32, i32)>,
    sight: Option<&Sight<'_>>,
) -> Vec<Drawn> {
    let to_screen = camera.to_screen_matrix();
    let r = camera.render_matrix().0;
    let mut out: Vec<Drawn> = placed
        .iter()
        .filter(|p| p.piece >= 0 && sight.is_none_or(|s| s.sees(p)))
        .map(|p| {
            let (x, y, z) = (p.x as f32, p.y as f32, p.z as f32);
            Drawn {
                who: p.who,
                o: p.o,
                figure: p.figure,
                anchor: world_to_screen(&to_screen, screen, p.x, p.y, p.z, size),
                depth: r[6] * y + r[2] * x + r[10] * z + r[14],
                piece: p.piece,
                slot: p.slot,
                frame: p.frame,
                facing: p.facing,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.depth
            .total_cmp(&a.depth)
            .then((a.who, a.o, a.figure).cmp(&(b.who, b.o, b.figure)))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
            parallel_distance: 10429.4,
        }
    }

    const SCREEN: Screen = Screen {
        scaled_xs: 1920,
        scaled_ys: 1080,
        natural_ratio: 1.0,
        console_y: 0,
    };

    fn fig(who: u8, o: i16, x: i32, y: i32, piece: i32) -> Placed {
        Placed {
            who,
            o,
            figure: 0,
            x,
            y,
            z: 0,
            piece,
            slot: 0,
            frame: 0,
            facing: 0,
        }
    }

    #[test]
    fn the_target_anchors_at_the_centre_and_nothing_without_a_piece_draws() {
        let placed = [fig(0, 0, 9552, 10320, 7), fig(0, 1, 9552, 10320, -1)];
        let list = draw_list(&placed, &camera(), &SCREEN, Some((1920, 1080)), None);
        assert_eq!(list.len(), 1);
        let a = list[0].anchor;
        assert!((a.x - 960).abs() <= 1 && (a.y - 540).abs() <= 1, "{a:?}");
    }

    #[test]
    fn the_far_figure_is_drawn_first() {
        // At `dir` 45 the eye sits toward +x +y of its target, so the
        // figure at smaller x + y is the further one — and the higher up.
        let placed = [fig(0, 0, 8000, 8000, 1), fig(0, 1, 12000, 12000, 1)];
        let list = draw_list(&placed, &camera(), &SCREEN, None, None);
        assert_eq!(list.iter().map(|d| d.o).collect::<Vec<_>>(), [0, 1]);
        assert!(list[0].depth > list[1].depth);
        assert!(list[0].anchor.y < list[1].anchor.y, "further is higher up");
    }

    #[test]
    fn a_viewer_sees_its_own_and_what_is_in_sight() {
        let plane = [0, FOG_VISIBLE, 1, 0];
        let sight = Sight {
            viewer: 1,
            plane: &plane,
            xs: 2,
            ys: 2,
            cell: 384,
        };
        let placed = [
            fig(0, 0, 100, 100, 1), // unseen
            fig(0, 1, 500, 100, 1), // in sight
            fig(0, 2, 100, 500, 1), // explored only
            fig(1, 3, 500, 500, 1), // the viewer's own, in the unseen cell
        ];
        let list = draw_list(&placed, &camera(), &SCREEN, None, Some(&sight));
        let mut drawn: Vec<_> = list.iter().map(|d| d.o).collect();
        drawn.sort();
        assert_eq!(drawn, [1, 3]);
    }
}
