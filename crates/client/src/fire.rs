//! **A burning building's fire and smoke, at the original's anchors** (item
//! 4201; `docs/RENDERER.md` §3.20): each finished building's burning slots
//! ([`render::fire::emitters`]) from its `zbuffer` model's attach slots,
//! and a stand-in drawn at each.
//!
//! **The stand-in is not the original's draw.** The original emits
//! particles there (`BUILDING FIRE`, `BUILDING SMOKE`; `Particle::batch_draw`
//! under `Fire Blend` and `Smoke Blend`) and none of their motion, timing
//! or blend is read. The client lays one sprite of `BuildingFX.tga`'s fire
//! cell (`tu 0 tv 0`, 32 texels) at the anchor and one of its smoke cell
//! (`tv 32`) tinted by the system's `COLOR` (40, 40, 30) above it, sized by
//! eye ([`CELLS`]): where the eye looks for the fire, not what it is.

use crate::world::{SpriteDrawing, SpriteImage, World};
use render::draw::{Drawn, PlacedBuilding, Pose, SpriteAt};
use render::fire::{Emitter, SLOTS, attach_slots, emitters};
use rondata::artdata::resolve_art;
use std::sync::Arc;

/// The client's keys for the two stand-ins' images (below
/// `world::SCAFFOLD_KEY`'s range).
pub const FIRE_KEY: i32 = -0x300;
pub const SMOKE_KEY: i32 = -0x301;

/// `effects_graphics.xml`'s `<PARTICLESYSTEMS>` texture 3.
const TEXTURE: &str = ".\\art\\BuildingFX.tga";

/// Each stand-in's cell's top-left texel, its width and its rise above the
/// anchor in world units, its `COLOR` and its `hold_alpha`. The widths and
/// rises are by eye on run1559b's photograph (a flame 15–20 pixels across
/// at the anchor, a plume 50–70 across above it): the systems'
/// `hold_width`s, 40 and 80, draw 9 and 18 pixels, and how the particle
/// pass scales them is not read.
const CELLS: [Cell; 2] = [
    Cell {
        texel: (0, 0),
        width: 80.0,
        rise: 0.0,
        colour: [255, 255, 255],
        alpha: 200,
    },
    Cell {
        texel: (0, 32),
        width: 240.0,
        rise: 240.0,
        colour: [40, 40, 30],
        alpha: 255,
    },
];

/// One stand-in's cell, as [`CELLS`] lists them.
struct Cell {
    texel: (usize, usize),
    width: f32,
    rise: f32,
    colour: [u8; 3],
    alpha: u8,
}
const CELL: usize = 32;

impl World {
    /// **Every burning slot this frame** (`render::fire::emitters`) of the
    /// placed `buildings`, each from its piece's `zbuffer` model.
    pub fn emitters(&mut self, buildings: &[PlacedBuilding]) -> Vec<Emitter> {
        let mut out = Vec::new();
        for b in buildings {
            if b.piece < 0 {
                continue;
            }
            if let Some(slots) = self.fire_slots_of(b.piece) {
                out.extend(emitters(b, &slots));
            }
        }
        out
    }

    fn fire_slots_of(&mut self, piece: i32) -> Option<[Option<[f32; 3]>; SLOTS]> {
        if let Some(s) = self.fire_slots.get(&piece) {
            return *s;
        }
        let slots = self.mesh(piece, true).map(|m| attach_slots(&m.model));
        self.fire_slots.insert(piece, slots);
        slots
    }

    fn fire_image(&mut self, k: usize) -> Option<Arc<SpriteImage>> {
        if self.fire_images.is_none() {
            let image = resolve_art(&self.root, TEXTURE)
                .and_then(|p| std::fs::read(p).ok())
                .and_then(|b| render::tga::decode(&b).ok());
            self.fire_images = Some(CELLS.each_ref().map(|c| {
                let ((u, v), colour, alpha) = (c.texel, c.colour, c.alpha);
                let img = image.as_ref()?;
                if u + CELL > img.width || v + CELL > img.height {
                    return None;
                }
                let mut rgba = Vec::with_capacity(CELL * CELL * 4);
                for y in v..v + CELL {
                    for x in u..u + CELL {
                        let p = &img.rgba[(y * img.width + x) * 4..][..4];
                        for c in 0..3 {
                            rgba.push((u16::from(p[c]) * u16::from(colour[c]) / 255) as u8);
                        }
                        rgba.push((u16::from(p[3]) * u16::from(alpha) / 255) as u8);
                    }
                }
                let covered = rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|p| p[3] >= 128)
                    .collect();
                Some(Arc::new(SpriteImage {
                    image: render::tga::Image {
                        width: CELL,
                        height: CELL,
                        rgba,
                        ..img.clone()
                    },
                    offset: (CELL as i32 / 2, CELL as i32 / 2),
                    covered,
                }))
            }));
        }
        self.fire_images.as_ref()?[k].clone()
    }

    /// **The burning slots the client draws**: those of each building on
    /// the list ([`render::fire::emitters`]); none with [`World::fires`]
    /// off. What [`Self::fire_drawings`] lays its stand-ins at, and what the
    /// row counts.
    pub fn drawn_emitters(&mut self, buildings: &[PlacedBuilding], list: &[Drawn]) -> Vec<Emitter> {
        if !self.fires {
            return Vec::new();
        }
        let on_list: Vec<PlacedBuilding> = buildings
            .iter()
            .filter(|b| {
                list.iter().any(|d| {
                    matches!(d.pose, Pose::Building { .. }) && (d.who, d.o) == (b.who, b.o)
                })
            })
            .copied()
            .collect();
        self.emitters(&on_list)
    }

    /// **The stand-ins** of [`Self::drawn_emitters`]: each cell's sprite
    /// centred on the projection of the anchor (raised by its rise), its
    /// width the cell's world units at the camera's pixels a unit across
    /// the screen, a little nearer than its building's sprite so that sprite
    /// does not hide it.
    pub fn fire_drawings(
        &mut self,
        buildings: &[PlacedBuilding],
        list: &[Drawn],
        camera: &render::Camera,
        screen: &render::Screen,
        size: (i32, i32),
    ) -> Vec<SpriteDrawing> {
        let m = camera.to_screen_matrix();
        let r = camera.render_matrix().0;
        let project = |p: [f32; 3]| {
            let q = render::camera::world_to_screen(
                &m,
                screen,
                p[0] as i32,
                p[1] as i32,
                p[2] as i32,
                Some(size),
            );
            (q.x as f32, q.y as f32)
        };
        let mut out = Vec::new();
        for e in self.drawn_emitters(buildings, list) {
            let Some(building_depth) = list
                .iter()
                .find(|d| matches!(d.pose, Pose::Building { .. }) && (d.who, d.o) == (e.who, e.o))
                .map(|d| d.depth)
            else {
                continue;
            };
            // Pixels a world unit across the screen: the longer of the two
            // ground diagonals' projections.
            let l = 1000.0;
            let at = project(e.pos);
            let ppu = [[l, -l], [l, l]]
                .iter()
                .map(|[dx, dy]| {
                    let q = project([e.pos[0] + dx, e.pos[1] + dy, e.pos[2]]);
                    (q.0 - at.0).hypot(q.1 - at.1) / (l * std::f32::consts::SQRT_2)
                })
                .fold(0.0f32, f32::max);
            let own = r[2] * e.pos[0] + r[6] * e.pos[1] + r[10] * e.pos[2] + r[14];
            for (k, Cell { width, rise, .. }) in CELLS.iter().enumerate() {
                let Some(image) = self.fire_image(k) else {
                    continue;
                };
                let (x, y) = project([e.pos[0], e.pos[1], e.pos[2] + rise]);
                let scale = width * ppu / CELL as f32;
                out.push(SpriteDrawing {
                    who: e.who,
                    o: e.o,
                    piece: [FIRE_KEY, SMOKE_KEY][k],
                    at: SpriteAt {
                        left: (x - CELL as f32 / 2.0 * scale) as i32,
                        top: (y - CELL as f32 / 2.0 * scale) as i32,
                        scale,
                    },
                    depth: own.min(building_depth) - 2.0,
                    image,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::tests::{Photo, probe_at, run1559b};
    use crate::weathering::tests::run1570;

    /// An anchor agrees within a tile (`render::terrain::UNITS_PER_TILE`).
    const TILE: f32 = 192.0;

    /// **The packets' emitters** (run1559b at logger frame 3294, run1570 at
    /// 3331; `GraphicPieces +0xadc`, `+0xaf8`, `+0xb14`, every valid
    /// `ParticleEmitter` of `BUILDING FIRE` (system 36) or `BUILDING SMOKE`
    /// (37)): each burning slot's `desc.who`, `o`, `node_num` and `pos`.
    /// Both systems stand at each slot's one point, and the two packets
    /// agree to the bit.
    const THEIRS: [(u8, i16, u8, [f32; 3]); 4] = [
        (0, 2000, 8, [3207.27, 30518.4, 726.74]),
        (0, 2000, 9, [2744.46, 31203.03, 694.73]),
        (0, 2000, 10, [2802.0, 30377.16, 674.0]),
        (0, 2000, 11, [3207.99, 30905.28, 682.67]),
    ];

    /// The packets' city `0/2000`: `BuildData::hits(0)` (`construct_hits`,
    /// `+0x54`) and `ObjectData::damage` (`+0x24`), on 3294 and on 3331.
    const THEIR_CITY: [(i64, i32, i32); 2] = [(3294, 1200, 1107), (3331, 1200, 1133)];

    /// One photograph's row: `(agree, theirs, ours past theirs)`, and each
    /// of theirs' pair of points and pixels.
    fn row(ours: &[Emitter], camera: &render::Camera) -> ((usize, usize, usize), Vec<String>) {
        let m = camera.to_screen_matrix();
        let px = |p: [f32; 3]| {
            let q = render::camera::world_to_screen(
                &m,
                &crate::pinned_screen(),
                p[0] as i32,
                p[1] as i32,
                p[2] as i32,
                Some((1920, 1080)),
            );
            (q.x, q.y)
        };
        let mut agree = 0;
        let mut lines = Vec::new();
        for (who, o, slot, t) in THEIRS {
            let mine = ours.iter().find(|e| (e.who, e.o, e.slot) == (who, o, slot));
            let d = mine.map(|e| {
                (0..3)
                    .map(|k| (e.pos[k] - t[k]).powi(2))
                    .sum::<f32>()
                    .sqrt()
            });
            agree += usize::from(d.is_some_and(|d| d <= TILE));
            lines.push(format!(
                "{who}/{o} slot {slot}: theirs {t:?} {:?}, ours {:?} {:?}, {d:?}",
                px(t),
                mine.map(|e| e.pos),
                mine.map(|e| px(e.pos)),
            ));
        }
        let past = ours
            .iter()
            .filter(|e| {
                !THEIRS
                    .iter()
                    .any(|t| (t.0, t.1, t.2) == (e.who, e.o, e.slot))
            })
            .count();
        ((agree, THEIRS.len(), past), lines)
    }

    /// The slots the client draws on the photograph's frame, as drawn, with
    /// the fires off (the count before this item), and unturned (each
    /// offset from the building's point turned back by `get_position`'s
    /// quarter: the made-to-fail).
    fn counts(shot: Photo) -> Option<[(usize, usize, usize); 3]> {
        let mut p = probe_at(shot)?;
        let (_, buildings, list) =
            p.world
                .draw_list(&p.camera, &crate::pinned_screen(), (1920, 1080));
        let drawn = p.world.drawn_emitters(&buildings, &list);
        let (as_drawn, lines) = row(&drawn, &p.camera);
        for l in lines {
            println!("{l}");
        }
        p.world.fires = false;
        let (before, _) = row(&p.world.drawn_emitters(&buildings, &list), &p.camera);
        p.world.fires = true;
        let unturned: Vec<Emitter> = drawn
            .iter()
            .map(|e| {
                let b = buildings
                    .iter()
                    .find(|b| (b.who, b.o) == (e.who, e.o))
                    .expect("its building");
                let d = [e.pos[0] - b.x as f32, e.pos[1] - b.y as f32];
                Emitter {
                    pos: [b.x as f32 - d[1], b.y as f32 - d[0], e.pos[2]],
                    ..*e
                }
            })
            .collect();
        let (unturned, _) = row(&unturned, &p.camera);
        // The value the level reads, ours against the packet's on its frame.
        for (frame, max, damage) in THEIR_CITY {
            if frame < p.world.runner.frame() || frame > p.world.runner.frame() + 20 {
                continue;
            }
            p.world.run_to(frame);
            let (_, b, _) = p
                .world
                .draw_list(&p.camera, &crate::pinned_screen(), (1920, 1080));
            let city = b
                .iter()
                .find(|b| (b.who, b.o) == (0, 2000))
                .expect("the city");
            println!(
                "frame {frame}: ours construct_hits {} damage {}, theirs {max} {damage}",
                city.construct_hits,
                city.hits - city.health
            );
            assert_eq!(
                (city.construct_hits, city.hits - city.health),
                (max, damage)
            );
        }
        Some([as_drawn, before, unturned])
    }

    /// **`RUN1559B_3279_FIRE`** (item 4201; §3.20, the rule committed
    /// before this count): `(agree, theirs, ours past theirs)` as drawn,
    /// before (none drawn) and unturned. **0 → 4 of 4**, each slot 0.0
    /// world units from the packet's, on its pixel; unturned 2 of 4
    /// (slots 9 and 11, near the turn's diagonal, 52 and 183 units off,
    /// inside the tile).
    const RUN1559B_3279_FIRE: [(usize, usize, usize); 3] = [(4, 4, 0), (0, 4, 0), (2, 4, 0)];
    /// **`RUN1570_FIRE`**, on run1570's frame 3327: as run1559b's.
    const RUN1570_FIRE: [(usize, usize, usize); 3] = [(4, 4, 0), (0, 4, 0), (2, 4, 0)];

    /// **The plume's window** (item 4233; §3.20): about each of
    /// [`THEIRS`]' slots' pixels on the photograph's camera, `(left, right,
    /// up, down)` pixels. The photographs' plumes rise up to 150 pixels
    /// over their slot and drift some 60 to either side.
    const PLUME_WINDOW: (i32, i32, i32, i32) = (64, 64, 192, 16);

    /// **A pixel's class against the scene** (item 4233; §3.20): its luma
    /// in hundredths of the luma of our still with the fires off, the scene
    /// without its smoke. Clear at 85 and over, haze from 35, dense under.
    /// A blended plume is haze over most of its area and dense at its core;
    /// an alpha-tested sprite is dense or clear, with no haze between. A
    /// black pixel of the scene cannot darken, and is clear.
    fn plume_class(side: u32, scene: u32) -> u8 {
        if side * 100 >= 85 * scene {
            0
        } else if side * 100 >= 35 * scene {
            1
        } else {
            2
        }
    }

    fn luma(p: &[u8]) -> u32 {
        (299 * u32::from(p[0]) + 587 * u32::from(p[1]) + 114 * u32::from(p[2])) / 1000
    }

    /// One plume row: `(agree, theirs, ours past theirs)` over the windows'
    /// union. Theirs is every pixel of the photograph not clear; a pixel
    /// agrees when ours is of its class; ours past is a pixel ours darkens
    /// that the photograph leaves clear. Beside it, ours' haze and dense
    /// pixels in the windows.
    fn plume_row(
        photo: &[u8],
        ours: &[u8],
        scene: &[u8],
        (w, h): (usize, usize),
        anchors: &[(i32, i32)],
    ) -> ((usize, usize, usize), (usize, usize)) {
        let (left, right, up, down) = PLUME_WINDOW;
        let mut inside = vec![false; w * h];
        for &(x, y) in anchors {
            for yy in (y - up).max(0)..(y + down).min(h as i32) {
                for xx in (x - left).max(0)..(x + right).min(w as i32) {
                    inside[yy as usize * w + xx as usize] = true;
                }
            }
        }
        let (mut agree, mut theirs, mut past, mut haze, mut dense) = (0, 0, 0, 0, 0);
        for (i, _) in inside.iter().enumerate().filter(|(_, on)| **on) {
            let base = luma(&scene[i * 4..]);
            let t = plume_class(luma(&photo[i * 4..]), base);
            let o = plume_class(luma(&ours[i * 4..]), base);
            theirs += usize::from(t > 0);
            agree += usize::from(t > 0 && o == t);
            past += usize::from(t == 0 && o > 0);
            haze += usize::from(o == 1);
            dense += usize::from(o == 2);
        }
        ((agree, theirs, past), (haze, dense))
    }

    /// The plume row on one photograph: as drawn, with the fires off (the
    /// count before any smoke), and the photograph taken for ours (the
    /// made-to-fail's other side: a row that cannot reach its whole is
    /// blind). Each `((agree, theirs, past), (haze, dense))`.
    fn plumes(shot: Photo, name: &str) -> Option<[((usize, usize, usize), (usize, usize)); 3]> {
        let mut p = probe_at(shot)?;
        let size = (p.photo.width, p.photo.height);
        let m = p.camera.to_screen_matrix();
        let anchors: Vec<(i32, i32)> = THEIRS
            .iter()
            .map(|t| {
                let q = render::camera::world_to_screen(
                    &m,
                    &crate::pinned_screen(),
                    t.3[0] as i32,
                    t.3[1] as i32,
                    t.3[2] as i32,
                    Some((size.0 as i32, size.1 as i32)),
                );
                (q.x, q.y)
            })
            .collect();
        println!("{name}: anchors {anchors:?}");
        p.world.fires = false;
        let scene = crate::weathering::tests::still(&mut p, &format!("{name}-fires-off"));
        p.world.fires = true;
        let drawn = crate::weathering::tests::still(&mut p, &format!("{name}-fires"));
        let photo = p.photo.rgba.clone();
        let out = [
            plume_row(&photo, &drawn, &scene, size, &anchors),
            plume_row(&photo, &scene, &scene, size, &anchors),
            plume_row(&photo, &photo, &scene, size, &anchors),
        ];
        println!("{name}: drawn, fires off, the photograph: {out:?}");
        Some(out)
    }

    /// **`RUN1559B_3279_PLUME`** (item 4233; §3.20, the rule committed
    /// before this count): `((agree, theirs, ours past), (ours' haze,
    /// dense))` as drawn, with the fires off, and with the photograph for
    /// ours. Drawn: item 4201's stand-ins, dense blocks with almost no
    /// haze.
    const RUN1559B_3279_PLUME: [((usize, usize, usize), (usize, usize)); 3] = [
        ((336, 19378, 185), (233, 964)),
        ((0, 19378, 0), (0, 0)),
        ((19378, 19378, 0), (16963, 2415)),
    ];
    /// **`RUN1570_PLUME`**, on run1570's frame 3327: as run1559b's.
    const RUN1570_PLUME: [((usize, usize, usize), (usize, usize)); 3] = [
        ((316, 18871, 254), (228, 970)),
        ((0, 18871, 0), (0, 0)),
        ((18871, 18871, 0), (16401, 2470)),
    ];

    #[test]
    fn the_plumes_over_the_burning_city_on_run1559b() {
        let Some(c) = plumes(run1559b(), "1559b") else {
            return;
        };
        assert_eq!(c, RUN1559B_3279_PLUME);
    }

    #[test]
    fn the_plumes_over_the_burning_city_on_run1570() {
        let Some(c) = plumes(run1570(), "1570") else {
            return;
        };
        assert_eq!(c, RUN1570_PLUME);
    }

    #[test]
    fn the_city_burns_where_the_packets_anchor_it_on_run1559b() {
        let Some(c) = counts(run1559b()) else { return };
        assert_eq!(c, RUN1559B_3279_FIRE);
    }

    #[test]
    fn the_city_burns_where_the_packets_anchor_it_on_run1570() {
        let Some(c) = counts(run1570()) else { return };
        assert_eq!(c, RUN1570_FIRE);
    }
}
