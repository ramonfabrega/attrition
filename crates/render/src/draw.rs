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
//!
//! **Buildings** (item 4035) enter as [`PlacedBuilding`], one per building
//! in use, and leave as a [`Drawn`] whose [`Pose`] is a building's. A
//! building is a sprite with a depth model, and `WallOut::draw@00641930`
//! anchors it at `world_to_screen` of its point moved back half a tile on
//! both axes when its `<BUILD>` says `halfoffset="1"`
//! ([`PlacedBuilding::anchor_point`]); it draws an unfinished building only
//! past a share of its construction ([`PlacedBuilding::drawn`]).
//!
//! **A farm's wheat field** (item 4048) is a second entry for a building
//! that carries a farm record ([`Field`]): `FarmsOut::draw@008d75f0` builds
//! the field over the footprint every frame, on its own pass, finished or
//! not, and skips the pasture. Its entry is anchored at the field's corner
//! ([`Field::anchor_point`]) and leaves the list with [`Pose::Field`].

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
    fn sees(&self, who: u8, x: i32, y: i32) -> bool {
        if who == self.viewer {
            return true;
        }
        let (fx, fy) = (x.div_euclid(self.cell), y.div_euclid(self.cell));
        fx >= 0
            && fy >= 0
            && fx < self.xs
            && fy < self.ys
            && self.plane.get((fy * self.xs + fx) as usize) == Some(&FOG_VISIBLE)
    }
}

/// One building in use, as the read door reports it
/// (`sim::observe::BuildingScene`), with what `rondata` resolves from the
/// install: its piece and its `<BUILD>`'s `halfoffset`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacedBuilding {
    pub who: u8,
    pub o: i16,
    /// The building's point and height, `SubObjectData::x_internal`,
    /// `y_internal` and `z_internal`.
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// The `TypeIndex` it is now, and the one it was placed as.
    pub type_index: i32,
    pub orig_type: i32,
    /// `WallData::gpiece`, the piece `GraphicPieces::get_build_gpiece`
    /// chose; `-1` where none resolved.
    pub piece: i32,
    /// The `<BUILD>`'s `halfoffset`, `BuildRDataStruct::build_half_tcoord`
    /// (0 or 1 in the install, and the same for every `<BUILD>` of one
    /// `GRAPH`).
    pub half_offset: i32,
    /// `WallData::is_active`, and the share built when not:
    /// `job_counter` of `BuildData::construct_time(0)`.
    pub active: bool,
    pub job_counter: i32,
    pub construct_time: i32,
    /// The farm record `FarmsOut::draw@008d75f0` draws a wheat field from,
    /// or `None` (item 4048).
    pub field: Option<Field>,
}

/// A farm's record as `FarmsOut::draw@008d75f0` reads it: the footprint of
/// the building's type (`BuildTypeData +0x234`, `+0x238`), the record's
/// `farm_type`, and its 25 corner heights (`FarmStruct::terrain_height`,
/// `x` index major).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub x_size: i32,
    pub y_size: i32,
    pub farm_type: u8,
    pub heights: [i32; 25],
}

/// `FarmType`'s pasture: `FarmsOut::draw` skips a record whose
/// `farm_type` is this (`+0xbd != 1`).
pub const PASTURE: u8 = 1;

/// A tile, in world units: `div_3_table[v >> 6]` is the tile `v` falls in.
pub const TILE: i32 = 0xc0;

impl Field {
    /// `FarmsOut::draw`'s corner of the field over a building at `(x, y)`,
    /// in world units — `WallData::tile_corner@00643440`'s tile times
    /// `0xc0`: the point snapped to its tile (to the tile's centre on an
    /// odd size), read back as a tile, less half the size (`sar`). The
    /// field's 25 heights are sampled from here, a tile apart, and its
    /// first stalk box stands half a tile in.
    pub fn corner(&self, x: i32, y: i32) -> (i32, i32) {
        let snap = |v: i32, size: i32| {
            let t = (v >> 6) / 3;
            let s = t * TILE + if size & 1 == 1 { HALF_TILE } else { 0 };
            ((s >> 6) / 3 - (size >> 1)) * TILE
        };
        (snap(x, self.x_size), snap(y, self.y_size))
    }

    /// The point the entry is anchored at: the field's corner on its own
    /// first height, `terrain_height[0][0]`.
    pub fn anchor_point(&self, x: i32, y: i32) -> (i32, i32, i32) {
        let (cx, cy) = self.corner(x, y);
        (cx, cy, self.heights[0])
    }

    /// Whether `FarmsOut::draw` draws it: every record but the pasture's.
    pub fn drawn(&self) -> bool {
        self.farm_type != PASTURE
    }
}

/// `TypeIndex` `0x1a1`, the farm: `WallOut::draw` draws it as a building
/// only while it is unfinished, the inverse of every other type.
pub const FARM: i32 = 0x1a1;

/// `0x1a6`: a type `WallOut::draw` does not draw at all until it is
/// finished.
pub const DRAWN_ONLY_FINISHED: i32 = 0x1a6;

/// `AIRDEFENSE`, `RADAR` and `SAM` (`0x20b`..`0x20d`): drawn as a model at
/// the point, with no `halfoffset`.
pub const MODELS_AT_THE_POINT: [i32; 3] = [0x20b, 0x20c, 0x20d];

/// A half tile, in world units: `WallOut::draw` moves the point back
/// `0x60 · halfoffset` on both axes.
pub const HALF_TILE: i32 = 0x60;

impl PlacedBuilding {
    /// The share built, as `WallOut::draw` reads it: 1 for a finished
    /// building, else `job_counter / construct_time(0)` in `float`.
    pub fn share(&self) -> f32 {
        if self.active {
            1.0
        } else {
            (self.job_counter as u32 as f64 as f32) / (self.construct_time as f64 as f32)
        }
    }

    /// **Whether `WallOut::draw@00641930` draws the building's piece.**
    /// Its alpha starts opaque past 0.88 of the share, fades in from 0.84,
    /// and is inverted for every type but the farm, so a building under
    /// construction is drawn past 0.84 and a farm only below 0.88; a
    /// `0x1a6` is not drawn before it is finished. `is_force_construction`
    /// (a share of 0) is not read.
    pub fn drawn(&self) -> bool {
        let share = self.share();
        let mut alpha: u8 = 0;
        if share < 0.88 {
            alpha = 0xff;
            if share > 0.84 {
                alpha = (((0.88 - share) * 25.0).min(1.0) * 255.0) as u8;
            }
        }
        if self.type_index != FARM {
            alpha = 0xff - alpha;
            if share < 1.0 && self.type_index == DRAWN_ONLY_FINISHED {
                return false;
            }
        }
        alpha != 0 && self.piece >= 0
    }

    /// The point `WallOut::draw` projects: the building's own, moved back
    /// half a tile on both axes when its `<BUILD>` says `halfoffset="1"`
    /// (`0x60 · build_half_tcoord`, the listing at `00642033`–`0064207d`).
    pub fn anchor_point(&self) -> (i32, i32, i32) {
        let back = if MODELS_AT_THE_POINT.contains(&self.type_index) {
            0
        } else {
            HALF_TILE * self.half_offset
        };
        (self.x - back, self.y - back, self.z)
    }
}

/// What a drawn entry is: a figure with its pose, or a building.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Figure {
        /// The figure's index among its object's guys.
        figure: u16,
        slot: i8,
        frame: u32,
        facing: u32,
    },
    Building {
        type_index: i32,
        orig_type: i32,
    },
    /// A farm's wheat field, `FarmsOut::draw`'s (item 4048).
    Field {
        orig_type: i32,
    },
}

/// One figure or building the viewer sees, placed on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drawn {
    pub who: u8,
    pub o: i16,
    /// The pixel `world_to_screen` gives the figure's point, or the
    /// building's [`PlacedBuilding::anchor_point`].
    pub anchor: Projected,
    /// The depth in camera space, `render_matrix`'s third column: larger
    /// is further from the eye. The list is ordered on it, furthest first,
    /// ties by identity.
    pub depth: f32,
    pub piece: i32,
    pub pose: Pose,
}

impl Drawn {
    /// The figure's index, or `None` for a building.
    pub fn figure(&self) -> Option<u16> {
        match self.pose {
            Pose::Figure { figure, .. } => Some(figure),
            Pose::Building { .. } | Pose::Field { .. } => None,
        }
    }

    fn key(&self) -> (u8, i16, i32) {
        let rank = match self.pose {
            Pose::Figure { figure, .. } => i32::from(figure),
            Pose::Building { .. } => -1,
            Pose::Field { .. } => -2,
        };
        (self.who, self.o, rank)
    }
}

/// The draw list of `placed` and `buildings` at `camera` on `screen`
/// (`size` as [`world_to_screen`] takes it), as `sight`'s viewer sees it,
/// or whole for `None`: furthest first.
pub fn draw_list(
    placed: &[Placed],
    buildings: &[PlacedBuilding],
    camera: &Camera,
    screen: &Screen,
    size: Option<(i32, i32)>,
    sight: Option<&Sight<'_>>,
) -> Vec<Drawn> {
    let to_screen = camera.to_screen_matrix();
    let r = camera.render_matrix().0;
    let depth = |x: i32, y: i32, z: i32| {
        let (x, y, z) = (x as f32, y as f32, z as f32);
        r[6] * y + r[2] * x + r[10] * z + r[14]
    };
    let mut out: Vec<Drawn> = placed
        .iter()
        .filter(|p| p.piece >= 0 && sight.is_none_or(|s| s.sees(p.who, p.x, p.y)))
        .map(|p| Drawn {
            who: p.who,
            o: p.o,
            anchor: world_to_screen(&to_screen, screen, p.x, p.y, p.z, size),
            depth: depth(p.x, p.y, p.z),
            piece: p.piece,
            pose: Pose::Figure {
                figure: p.figure,
                slot: p.slot,
                frame: p.frame,
                facing: p.facing,
            },
        })
        .collect();
    out.extend(
        buildings
            .iter()
            .filter(|b| b.drawn() && sight.is_none_or(|s| s.sees(b.who, b.x, b.y)))
            .map(|b| {
                let (x, y, z) = b.anchor_point();
                Drawn {
                    who: b.who,
                    o: b.o,
                    anchor: world_to_screen(&to_screen, screen, x, y, z, size),
                    depth: depth(b.x, b.y, b.z),
                    piece: b.piece,
                    pose: Pose::Building {
                        type_index: b.type_index,
                        orig_type: b.orig_type,
                    },
                }
            }),
    );
    out.extend(
        buildings
            .iter()
            .filter_map(|b| b.field.filter(Field::drawn).map(|f| (b, f)))
            .filter(|(b, _)| sight.is_none_or(|s| s.sees(b.who, b.x, b.y)))
            .map(|(b, f)| {
                let (x, y, z) = f.anchor_point(b.x, b.y);
                Drawn {
                    who: b.who,
                    o: b.o,
                    anchor: world_to_screen(&to_screen, screen, x, y, z, size),
                    depth: depth(x, y, z),
                    piece: -1,
                    pose: Pose::Field {
                        orig_type: b.orig_type,
                    },
                }
            }),
    );
    out.sort_by(|a, b| b.depth.total_cmp(&a.depth).then(a.key().cmp(&b.key())));
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
        let list = draw_list(&placed, &[], &camera(), &SCREEN, Some((1920, 1080)), None);
        assert_eq!(list.len(), 1);
        let a = list[0].anchor;
        assert!((a.x - 960).abs() <= 1 && (a.y - 540).abs() <= 1, "{a:?}");
    }

    #[test]
    fn the_far_figure_is_drawn_first() {
        // At `dir` 45 the eye sits toward +x +y of its target, so the
        // figure at smaller x + y is the further one — and the higher up.
        let placed = [fig(0, 0, 8000, 8000, 1), fig(0, 1, 12000, 12000, 1)];
        let list = draw_list(&placed, &[], &camera(), &SCREEN, None, None);
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
        let list = draw_list(&placed, &[], &camera(), &SCREEN, None, Some(&sight));
        let mut drawn: Vec<_> = list.iter().map(|d| d.o).collect();
        drawn.sort();
        assert_eq!(drawn, [1, 3]);
    }

    fn building(type_index: i32, active: bool, job: i32, half: i32) -> PlacedBuilding {
        PlacedBuilding {
            who: 1,
            o: 2000,
            x: 9552,
            y: 10320,
            z: 0,
            type_index,
            orig_type: type_index,
            piece: 51000,
            half_offset: half,
            active,
            job_counter: job,
            construct_time: 1000,
            field: None,
        }
    }

    #[test]
    fn a_building_anchors_half_a_tile_back_when_its_build_says_so() {
        assert_eq!(building(414, true, 0, 0).anchor_point(), (9552, 10320, 0));
        assert_eq!(building(414, true, 0, 1).anchor_point(), (9456, 10224, 0));
        // The three drawn as a model at the point take no offset.
        assert_eq!(building(0x20c, true, 0, 1).anchor_point(), (9552, 10320, 0));
        let list = draw_list(
            &[],
            &[building(414, true, 0, 1)],
            &camera(),
            &SCREEN,
            Some((1920, 1080)),
            None,
        );
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].figure(), None);
        // Back on both axes is toward the far corner: up the screen.
        assert!(list[0].anchor.y < 540, "{:?}", list[0].anchor);
    }

    #[test]
    fn an_unfinished_building_is_drawn_past_0_84_and_a_farm_only_before_0_88() {
        assert!(building(414, true, 0, 0).drawn());
        assert!(!building(414, false, 0, 0).drawn());
        assert!(!building(414, false, 840, 0).drawn());
        assert!(building(414, false, 841, 0).drawn());
        assert!(building(414, false, 900, 0).drawn());
        assert!(!building(FARM, true, 0, 0).drawn());
        assert!(building(FARM, false, 500, 0).drawn());
        assert!(!building(FARM, false, 900, 0).drawn());
        assert!(!building(DRAWN_ONLY_FINISHED, false, 990, 0).drawn());
        let mut none = building(414, true, 0, 0);
        none.piece = -1;
        assert!(!none.drawn());
    }

    #[test]
    fn a_farm_s_field_is_drawn_at_its_corner_and_the_pasture_s_is_not() {
        // A 4×4 farm at run1428's `1/2002` (3648, 29760): the tile is
        // (19, 155), an even size snaps to its corner, less two tiles.
        let mut heights = [0; 25];
        heights[0] = 184;
        heights[1] = 188;
        let field = Field {
            x_size: 4,
            y_size: 4,
            farm_type: 0,
            heights,
        };
        assert_eq!(field.corner(3648, 29760), (17 * TILE, 153 * TILE));
        assert_eq!(field.anchor_point(3648, 29760), (3264, 29376, 184));
        // An odd size snaps to the tile's centre first; 5 halves to 2.
        let odd = Field { x_size: 5, ..field };
        assert_eq!(odd.corner(3648, 29760).0, 17 * TILE);
        let mut farm = building(FARM, true, 0, 0);
        farm.x = 3648;
        farm.y = 29760;
        farm.field = Some(field);
        let list = draw_list(&[], &[farm], &camera(), &SCREEN, None, None);
        // The finished farm's sprite is not drawn; its field is.
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].pose, Pose::Field { orig_type: FARM });
        farm.field = Some(Field {
            farm_type: PASTURE,
            ..field
        });
        assert!(draw_list(&[], &[farm], &camera(), &SCREEN, None, None).is_empty());
        // Unfinished, both: the sprite under 0.88 and the field.
        farm.field = Some(field);
        farm.active = false;
        farm.job_counter = 500;
        assert_eq!(
            draw_list(&[], &[farm], &camera(), &SCREEN, None, None).len(),
            2
        );
    }
}
