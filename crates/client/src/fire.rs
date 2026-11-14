//! **A burning building's slots, at the original's anchors** (item 4201;
//! `docs/RENDERER.md` §3.20): each finished building's burning slots
//! ([`render::fire::emitters`]) from its `zbuffer` model's attach slots.
//! What burns there is `client::particle`'s (item 4233, §3.25), which took
//! the place of 4201's stand-in sprites.

use crate::world::World;
use render::draw::{Drawn, PlacedBuilding, Pose};
use render::fire::{Emitter, SLOTS, attach_slots, emitters};

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

    /// **The burning slots the client draws**: those of each building on
    /// the list ([`render::fire::emitters`]); none with [`World::fires`]
    /// off. Where `client::particle` lays its plumes, and what the row
    /// counts.
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

    /// One photograph's plume row in its four columns, each `((agree,
    /// theirs, ours past), (ours' haze, dense))`.
    type PlumeRows = [((usize, usize, usize), (usize, usize)); 4];

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
    fn plumes(shot: Photo, name: &str, dump_name: &str) -> Option<PlumeRows> {
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
        // The packet's own particles through our pass: the draw apart from
        // the emission.
        let packet = crate::particle::tests::dump(dump_name).map(|d| {
            let given = d
                .particles
                .iter()
                .map(|&(sys, _, _, _, q)| (usize::from(sys - 36), q))
                .collect();
            p.world.particles_given = Some(given);
            let s = crate::weathering::tests::still(&mut p, &format!("{name}-packet"));
            p.world.particles_given = None;
            s
        });
        let photo = p.photo.rgba.clone();
        let out = [
            plume_row(&photo, &drawn, &scene, size, &anchors),
            plume_row(&photo, &scene, &scene, size, &anchors),
            plume_row(&photo, &photo, &scene, size, &anchors),
            plume_row(
                &photo,
                packet.as_deref().unwrap_or(&scene),
                &scene,
                size,
                &anchors,
            ),
        ];
        println!("{name}: drawn, fires off, the photograph, the packet's: {out:?}");
        Some(out)
    }

    /// **`RUN1559B_3279_PLUME`** (item 4233; §3.20, the rule committed
    /// before this count): `((agree, theirs, ours past), (ours' haze,
    /// dense))` as drawn, with the fires off, and with the photograph for
    /// ours, and with the packet's own particles through our pass (the
    /// draw apart from the emission). Seen at **336** of 19,378 with item
    /// 4201's stand-ins (dense blocks, haze 233, dense 964); made to fail,
    /// haze and dense merged into one class, those read 1,012. Item 4233:
    /// **336 → 9,138**, past 185 → 5,617, the particles drawn (§3.25); the
    /// packet's particles read 11,518, past 1,702. Ours' dense core is
    /// thin, 120 against 2,415, and the packet's through our pass 468
    /// (4233a). Item 4237: the particles through `TL_FAKE_LIGHT` and
    /// blended in HDR (§3.25): **dense 120 → 487**, the row 9,138 → 9,728,
    /// past 5,617 → 5,538; the packet's dense **468 → 1,716**, its row
    /// 11,518 → 11,806, past 1,702 → 2,042. The LDR blend is the before.
    const RUN1559B_3279_PLUME: PlumeRows = [
        ((9728, 19378, 5538), (16570, 487)),
        ((0, 19378, 0), (0, 0)),
        ((19378, 19378, 0), (16963, 2415)),
        ((11806, 19378, 2042), (14210, 1716)),
    ];
    /// **`RUN1570_PLUME`**, on run1570's frame 3327: as run1559b's. Seen
    /// at **316** of 18,871 (made to fail: 944). Item 4233: **316 →
    /// 9,232**, past 254 → 5,055; the wind taken at birth for the whole
    /// life read 5,772, past 9,742. The packet's particles read 11,847,
    /// past 1,353. Item 4237, in HDR: **dense 182 → 770**, the row 9,232 →
    /// 9,403, past 5,055 → 5,075; the packet's dense **676 → 2,099**, its
    /// row 11,847 → 12,219, past 1,353 → 1,555.
    const RUN1570_PLUME: PlumeRows = [
        ((9403, 18871, 5075), (15668, 770)),
        ((0, 18871, 0), (0, 0)),
        ((18871, 18871, 0), (16401, 2470)),
        ((12219, 18871, 1555), (13771, 2099)),
    ];

    #[test]
    fn the_plumes_over_the_burning_city_on_run1559b() {
        let Some(c) = plumes(run1559b(), "1559b", "run1559b") else {
            return;
        };
        assert_eq!(c, RUN1559B_3279_PLUME);
    }

    #[test]
    fn the_plumes_over_the_burning_city_on_run1570() {
        let Some(c) = plumes(run1570(), "1570", "run1570") else {
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
