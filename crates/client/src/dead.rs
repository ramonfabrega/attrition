//! The corpse and the round, drawn (item 4191; `docs/RENDERER.md` §3.13).
//!
//! A corpse is each standing `DeathObj` record, drawn as
//! `DeathObjOut::draw@008d4090` draws its body: the record's piece on its
//! `cur_anim`, at its own `x`, `y`, `z` and `new_angle`
//! ([`sim::combat::Lie`], the guy's own at the kill's making). The body
//! plays its action at `age × 200 / 3` ms and holds at `(frames × 200 −
//! 200) / 3`; it is opaque to `frames + 0x69` and fades out by `frames +
//! body_fade_end` (135), after which it is not drawn. Not drawn here: the
//! fade's alpha (the body is drawn whole while any of it shows), the
//! skeleton piece `skel_gpiece` that fades in beside it, the slope tilt of
//! a type of `+0x14` 2 or 3, and `center_z` where the action moves it.

use render::draw::Placed;
use sim::observe::Scene;

/// A corpse's figure index in the draw list: past any guy's, so a dead
/// unit's record never keys as a living figure's.
pub const CORPSE_FIGURE: u16 = 0x100;

/// `body_fade_end` (`0x00b2176c`, 135): the body fades over the frames
/// from its action's end plus `0x69` to its action's end plus this.
pub const BODY_FADE_END: i64 = 135;

/// **The scene's corpses as the draw list takes them**: one [`Placed`] per
/// standing record whose body still shows, moved by `shift` world units
/// (the row's made-to-fail; `(0, 0)` to draw).
/// `frame` is the frame drawn: the scene's own, or the one before it where
/// the figures are drawn at their last points ([`crate::world::Blend::Last`]).
pub fn corpses(scene: &Scene, frame: i64, shift: (i32, i32)) -> Vec<Placed> {
    let mut out = Vec::new();
    for d in &scene.deaths {
        let age = frame - d.first_frame;
        let Ok(slot) = i8::try_from(d.cur_anim) else {
            continue;
        };
        for (k, ((&piece, lie), frames)) in
            d.pieces.iter().zip(&d.places).zip(&d.frames).enumerate()
        {
            let Some(frames) = frames.map(i64::from) else {
                continue;
            };
            if age >= frames + BODY_FADE_END {
                continue;
            }
            // The clock in game frames, as a figure's `cur_time` is
            // (`crate::world::anim_ms` takes it to milliseconds): `age`
            // while the action plays, then held at `frames − 1`, the
            // `(frames × 200 − 200) / 3` ms the draw holds at.
            let clock = if age < frames { age } else { frames - 1 };
            out.push(Placed {
                who: d.who as u8,
                o: d.o as i16,
                figure: CORPSE_FIGURE + k as u16,
                x: lie.pos.x + shift.0,
                y: lie.pos.y + shift.1,
                z: lie.z,
                piece,
                slot,
                frame: clock.max(0) as u32,
                facing: lie.angle.0 as u32,
            });
        }
    }
    out
}

/// **The frame run1570's photograph shows** (item 4191, parked 4183b):
/// 3327, four frames before the packet's 3331, clocked by its movers as
/// [`crate::silhouette::PHOTO_FRAME`] was (`RUN1570_ANCHORS`): the frame of
/// 3316..=3332 whose movers project nearest the photograph's. Not run1559b's
/// fifteen; why either trails its pause is not established (4073a).
#[cfg(test)]
pub const RUN1570_FRAME: i64 = 3327;

/// **The movers' anchors on run1570's photograph**, in its pixels (item
/// 4191), by eye on 4× crops: the runners `1/35` and `1/22` at their feet,
/// and the tank `1/16` at the point under its hull where our tank's anchor
/// stands. They are [`RUN1570_FRAME`]'s clock.
#[cfg(test)]
pub const RUN1570_ANCHORS: [((u8, i16), (i32, i32)); 3] = [
    ((1, 35), (1184, 630)),
    ((1, 22), (1389, 643)),
    ((1, 16), (1438, 675)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::tests::Probe;
    use crate::weathering::tests::run1570;
    use crate::world::World;
    use render::silhouette::chromaticity;
    use std::path::PathBuf;

    /// The miss of each frame of 3316..=3332: the summed distance from
    /// each anchor to its mover's projected point.
    #[test]
    fn run1570_s_photograph_shows_frame_3327() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = run1570().camera;
        let (m, screen) = (camera.to_screen_matrix(), crate::pinned_screen());
        let mut misses = Vec::new();
        for n in 3316..=3332 {
            world.run_to(n);
            let (figures, _, _) = world.draw_list(&camera, &screen, (1920, 1080));
            let mut miss = 0.0f64;
            for ((who, o), (px, py)) in RUN1570_ANCHORS {
                let f = figures
                    .iter()
                    .find(|f| (f.who, f.o, f.figure) == (who, o, 0))
                    .expect("each mover is on the map");
                let q =
                    render::camera::world_to_screen(&m, &screen, f.x, f.y, f.z, Some((1920, 1080)));
                miss += f64::from((q.x - px).pow(2) + (q.y - py).pow(2)).sqrt();
            }
            println!("frame {n}: miss {miss:.1}");
            misses.push((n, miss));
        }
        let best = misses
            .iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .copied()
            .expect("frames walked");
        assert_eq!(best.0, RUN1570_FRAME, "{misses:?}");
        // Every other frame misses by at least twice as much.
        for &(n, miss) in &misses {
            assert!(
                n == best.0 || miss >= 2.0 * best.1,
                "frame {n} {miss:.1} against {best:?}"
            );
        }
    }

    /// One record's window and its verdict.
    #[derive(Debug, PartialEq)]
    struct Window {
        who: i32,
        o: i32,
        /// Our record's visible pixels: where the still with the corpses
        /// differs from the still without, within [`REACH`] of the point.
        pixels: usize,
        overlap: f32,
        /// The photograph's and the still's mean chromaticity on the keys'
        /// intersection.
        theirs: [f32; 2],
        ours: [f32; 2],
        in_register: bool,
    }

    /// How far from a record's projected point its window reaches, in
    /// pixels: past any body's length, short of the other record's.
    const REACH: i32 = 80;
    /// The ring: the pixels within this many of the window, outside it,
    /// where the two stills agree.
    const RING: i32 = 6;
    /// Per pixel, a channel within 8 of the other agrees (§3.15).
    const SAME: i32 = 8;
    /// The rows' own tolerances (§3.15).
    const OVERLAP: f32 = 0.5;
    const CHROMA: f32 = 0.05;

    fn rgb(img: &[u8], w: usize, x: i32, y: i32) -> [f32; 3] {
        let i = (y as usize * w + x as usize) * 4;
        [
            f32::from(img[i]),
            f32::from(img[i + 1]),
            f32::from(img[i + 2]),
        ]
    }

    fn mean(v: &[[f32; 3]]) -> [f32; 3] {
        let n = v.len().max(1) as f32;
        let s = v
            .iter()
            .fold([0.0; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
        [s[0] / n, s[1] / n, s[2] / n]
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
    }

    /// **`RUN1570_DEAD`'s windows** (`docs/RENDERER.md` §3.13, the rule
    /// amended by item 4191 before this count). Each corpse record
    /// standing on [`RUN1570_FRAME`] is one window, at its place as the
    /// scene holds it (equal to the packet's death pool, below):
    /// - its pixels W are ours where `with` differs from `without` by more
    ///   than [`SAME`] in a channel, within [`REACH`] of its projection;
    /// - each side's key is the pixels of W whose chromaticity is more
    ///   than [`CHROMA`] from that side's ring mean (the photograph's ring
    ///   on the photograph, the still's on `with`);
    /// - in register when the keys overlap by [`OVERLAP`] (intersection
    ///   over union) and their mean chromaticities on the intersection
    ///   agree within [`CHROMA`]. A window with no pixels — the body not
    ///   drawn, or wholly hidden — is not in register, and is named.
    fn windows(p: &mut Probe, with: &[u8], without: &[u8]) -> Vec<Window> {
        let (w, h) = (p.photo.width, p.photo.height);
        let photo = &p.photo.rgba;
        let m = p.camera.to_screen_matrix();
        let screen = crate::pinned_screen();
        let scene = p.world.runner.scene(None);
        let differs = |x: i32, y: i32| {
            let (a, b) = (rgb(with, w, x, y), rgb(without, w, x, y));
            (0..3).any(|c| (a[c] - b[c]).abs() as i32 > SAME)
        };
        let mut out = Vec::new();
        for d in scene.deaths.iter().filter(|d| !d.pieces.is_empty()) {
            let lie = d.places[0];
            let q = render::camera::world_to_screen(
                &m,
                &screen,
                lie.pos.x,
                lie.pos.y,
                lie.z,
                Some((w as i32, h as i32)),
            );
            let (x0, x1) = ((q.x - REACH).max(0), (q.x + REACH).min(w as i32 - 1));
            let (y0, y1) = ((q.y - REACH).max(0), (q.y + REACH).min(h as i32 - 1));
            let inside: Vec<(i32, i32)> = (y0..=y1)
                .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
                .filter(|&(x, y)| differs(x, y))
                .collect();
            let set: std::collections::BTreeSet<(i32, i32)> = inside.iter().copied().collect();
            let mut ring = Vec::new();
            for y in (y0 - RING).max(0)..=(y1 + RING).min(h as i32 - 1) {
                for x in (x0 - RING).max(0)..=(x1 + RING).min(w as i32 - 1) {
                    if set.contains(&(x, y)) || differs(x, y) {
                        continue;
                    }
                    let near = (-RING..=RING)
                        .any(|dy| (-RING..=RING).any(|dx| set.contains(&(x + dx, y + dy))));
                    if near {
                        ring.push((x, y));
                    }
                }
            }
            let ring_of = |img: &[u8]| {
                let v: Vec<[f32; 3]> = ring.iter().map(|&(x, y)| rgb(img, w, x, y)).collect();
                chromaticity(mean(&v))
            };
            let (rt, ro) = (ring_of(photo), ring_of(with));
            let key = |img: &[u8], r: [f32; 2]| -> std::collections::BTreeSet<(i32, i32)> {
                inside
                    .iter()
                    .copied()
                    .filter(|&(x, y)| dist(chromaticity(rgb(img, w, x, y)), r) > CHROMA)
                    .collect()
            };
            let (a, b) = (key(photo, rt), key(with, ro));
            let both: Vec<(i32, i32)> = a.intersection(&b).copied().collect();
            let union = a.union(&b).count();
            let overlap = if union == 0 {
                0.0
            } else {
                both.len() as f32 / union as f32
            };
            let on = |img: &[u8]| {
                let v: Vec<[f32; 3]> = both.iter().map(|&(x, y)| rgb(img, w, x, y)).collect();
                chromaticity(mean(&v))
            };
            let (theirs, ours) = (on(photo), on(with));
            let in_register =
                !inside.is_empty() && overlap >= OVERLAP && dist(theirs, ours) <= CHROMA;
            let win = Window {
                who: d.who,
                o: d.o,
                pixels: inside.len(),
                overlap,
                theirs,
                ours,
                in_register,
            };
            println!(
                "{}/{} first {} at ({}, {}): {win:?}",
                d.who, d.o, d.first_frame, q.x, q.y
            );
            out.push(win);
        }
        out
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
            let out = PathBuf::from(dir).join(format!("still-dead-{name}.bmp"));
            std::fs::write(&out, crate::gpu::bmp(&s, size)).expect("the still writes");
        }
        s
    }

    /// **`RUN1570_DEAD`** (item 4191; §3.13): `(in register, windows)`
    /// drawn, before (no corpse drawn: the count as drawn, pinned first),
    /// and moved (every record shifted [`MOVED`] world units: the
    /// made-to-fail). **0 of 2 → 0 of 2**, each window named:
    /// - `0/7` (age 15, `man_shot1`'s 17 frames: lying) is drawn, 262
    ///   pixels, overlap 0.52, but the chromaticity parts: theirs (0.541,
    ///   0.324), ours (0.661, 0.197). Occluded: the derrick's log pile hides
    ///   most of the photograph's body, and our sprite writes one depth for
    ///   the whole building, so our body lies over the logs (4191a).
    /// - `0/11` (age 624) has no pixels: its body faded out at 23 + 135,
    ///   and the photograph shows its skeleton (piece 60113) fading at
    ///   (23 + 627 − 624) × 255 / 45 = 147, which the client does not draw
    ///   (4191b).
    ///
    /// Moved, `0/7`'s chromaticity is (0.393, 0.369) theirs, grass and logs.
    const RUN1570_DEAD: [(usize, usize); 3] = [(0, 2), (0, 2), (0, 2)];
    /// The made-to-fail's shift, about a body's length.
    const MOVED: (i32, i32) = (96, -96);

    /// The packet's death pool on 3331 (run1570, `Objects +0x14c`, slots of
    /// `0xa4`), each record's `who`, `o`, `x`, `y`, `z` and `new_angle`.
    const RUN1570_POOL: [(i32, i32, i32, i32, i32, i32); 2] = [
        (0, 7, 3972, 28574, 499, -476_512_256),
        (0, 11, 3933, 32387, 554, -1_352_335_360),
    ];

    #[test]
    fn run1570_s_corpses_lie_where_the_packet_s_do_and_are_drawn() {
        let Some(mut p) = crate::ground::tests::probe_at(run1570()) else {
            return;
        };
        // The place: every record on the frame against the packet's pool.
        let scene = p.world.runner.scene(None);
        let ours: Vec<_> = scene
            .deaths
            .iter()
            .filter(|d| !d.pieces.is_empty())
            .map(|d| {
                let l = d.places[0];
                (d.who, d.o, l.pos.x, l.pos.y, l.z, l.angle.0)
            })
            .collect();
        let mut theirs = RUN1570_POOL.to_vec();
        let mut ours_sorted = ours.clone();
        theirs.sort();
        ours_sorted.sort();
        assert_eq!(
            ours_sorted, theirs,
            "the records' places against the packet's pool"
        );

        p.world.corpses = false;
        let without = still(&mut p, "before");
        p.world.corpses = true;
        let with = still(&mut p, "drawn");
        p.world.corpse_shift = MOVED;
        let moved = still(&mut p, "moved");
        p.world.corpse_shift = (0, 0);
        let count = |v: &[Window]| (v.iter().filter(|w| w.in_register).count(), v.len());
        println!("drawn:");
        let drawn = count(&windows(&mut p, &with, &without));
        println!("before:");
        let before = count(&windows(&mut p, &without, &without));
        println!("moved:");
        let moved = count(&windows(&mut p, &moved, &without));
        assert_eq!([drawn, before, moved], RUN1570_DEAD);
    }
}
