//! **The type circles** (item 4629; `docs/RENDERER.md` §3.32.4): the
//! ring the minimap draws round each of the console's selected buildings.
//!
//! `WorldMap::render_ping@008ca6d0`, after the message window's events,
//! walks the console's selection. When it is a group of buildings
//! (`GroupData::buildings`) of the console's own, each building gets
//! `draw_type_circle@008c89c0` (vslot `+0x444`) with its type and its
//! world point. For a building type whose
//! `BuildTypeData::get_building_range@00639990` (asked of the console) is
//! not 0, that draws one `draw_minimap_circle@008c84a0` (vslot `+0x448`)
//! of radius `range · 0xc0` in the owner's `TeamColor::get_text`.
//!
//! `draw_minimap_circle` lays the ring as line strips: 73 points, every
//! 5 degrees from 0 to 360, at `(cx + r cos a, cy + ½ r sin a)`, `r` the
//! world radius times the quad's width over `cells · 0x300 · 1.413998`
//! (the constant at `0x00b69550`). A point inside the quad's two
//! triangles joins the strip; one outside ends it (the strip takes the
//! point and is drawn when it held two or more) and starts the next. The
//! last point ends the last strip. Each strip is one constant colour,
//! opaque, untextured.
//!
//! Not built: the other arms of `draw_type_circle`: a building with no
//! range that the type's `is` (vslot `+0x60`) names an air base or a
//! silo, and the selected unit of type `0x15f`, whose rings are the air
//! units' reach (4629a).

use sim::observe::ObjId;

/// **`TEAM_COLOR_TEXT_*`** (`.data`, `TeamColors::init@008c3100` lays
/// them on `team_colors.list[0..8].text`): the bytes of the executable at
/// `0x00c8ce88`, `ce64`, `cfb4`, `cfc0`, `cfcc`, `ce70`, `ce7c` and `cfd8`;
/// run1814b's packet holds the same eight. A slot past the eight is
/// `WHITE`, as list 8 is. `TeamColor::get_text` returns them unless the
/// game plays with the profile's `prefs & 0x100` (the ally colours), which
/// run1814b's profile (`0x1f2e5055`) lacks: an assumption elsewhere.
pub const TEXT_COLOURS: [[u8; 3]; 8] = [
    [255, 100, 100],
    [70, 100, 255],
    [188, 0, 250],
    [0, 236, 33],
    [255, 239, 55],
    [0, 255, 255],
    [255, 255, 255],
    [255, 182, 6],
];

/// `draw_minimap_circle`'s divisor's factor (`0x00b69550`): not `√2`'s
/// `0x3fb504f3`, which would draw run1814b's ring 0.001 px smaller.
const ROOT_TWO: u32 = 0x3fb4_fdf4;

/// One ring: its strips in screen pixels and its colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    pub strips: Vec<Vec<(f32, f32)>>,
    pub colour: [u8; 3],
}

/// **`MiscAccess::world_to_console@00583840`** under the minimap's camera,
/// onto the quad's window: the world point's device coordinates, scaled
/// to the quad's width and height and rounded half up, from the quad's
/// left and top (`Window::client_to_screen`).
pub fn centre(size: (i32, i32), extent: (i32, i32), x: i32, y: i32) -> (i32, i32) {
    let m = super::minimap_camera(extent).to_screen_matrix().0;
    let (x, y) = (x as f32, y as f32);
    let w = x * m[3] + y * m[7] + m[15];
    let nx = (x * m[0] + y * m[4] + m[12]) / w;
    let ny = (x * m[1] + y * m[5] + m[13]) / w;
    let q = super::quad(size);
    let (qw, qh) = ((q[1].0 - q[3].0) as f32, (q[2].1 - q[0].1) as f32);
    let round = |v: f32| v as i32 + i32::from(v - (v as i32) as f32 >= 0.5);
    (
        q[3].0 + round((nx + 1.0) * qw * 0.5),
        q[0].1 + round((1.0 - ny) * qh * 0.5),
    )
}

/// The radius in pixels of a world radius `r` on a map `cells` wide.
pub fn radius(size: (i32, i32), cells: i32, r: i32) -> f32 {
    let q = super::quad(size);
    let span = (cells * 0x300) as f32 * f32::from_bits(ROOT_TWO);
    ((q[1].0 - q[3].0) * r) as f32 / span
}

/// Whether `(x, y)` lies in either of the quad's triangles
/// (`translated_verts` 0-1-2, then 3-4-5), edges inclusive.
fn inside(size: (i32, i32), x: f32, y: f32) -> bool {
    super::triangles(size).iter().any(|t| {
        let [(a, _), (b, _), (c, _)] = *t;
        let det = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
        let s = ((x - a.0) * (c.1 - a.1) - (c.0 - a.0) * (y - a.1)) / det;
        let t = ((b.0 - a.0) * (y - a.1) - (x - a.0) * (b.1 - a.1)) / det;
        s >= 0.0 && t >= 0.0 && s + t <= 1.0
    })
}

/// **`draw_minimap_circle`'s strips** round `c`, radius `r` pixels.
pub fn strips(size: (i32, i32), c: (i32, i32), r: f32) -> Vec<Vec<(f32, f32)>> {
    let (cx, cy) = (c.0 as f32, c.1 as f32);
    let mut out = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    for a in (0..=360).step_by(5) {
        let rad = (a as f32).to_radians();
        let p = (cx + rad.cos() * r, cy + rad.sin() * r * 0.5);
        if a != 360 && inside(size, p.0, p.1) {
            cur.push(p);
            continue;
        }
        if cur.len() > 1 {
            cur.push(p);
            out.push(std::mem::take(&mut cur));
        }
        cur.clear();
        cur.push(p);
    }
    out
}

/// **The rings laid**: one a building of the console's `selection` whose
/// `get_building_range` is not 0, in its owner's text colour.
pub fn build(world: &crate::world::World, selection: &[ObjId], size: (i32, i32)) -> Vec<Ring> {
    let console = crate::world::CONSOLE;
    let sim = world.runner.sim();
    let t = world.runner.terrain();
    let extent = (t.width * super::CELL, t.height * super::CELL);
    let mut out = Vec::new();
    for (b, bd) in sim.buildings.iter().enumerate() {
        let id = ObjId {
            who: bd.owner,
            o: bd.index,
        };
        if !bd.alive || bd.owner != console || bd.ty.is_none() || !selection.contains(&id) {
            continue;
        }
        let range = sim.building_range(b);
        if range == 0 {
            continue;
        }
        let c = centre(size, extent, bd.pos.x, bd.pos.y);
        out.push(Ring {
            strips: strips(size, c, radius(size, t.width, range * 0xc0)),
            colour: TEXT_COLOURS
                .get(usize::from(bd.owner))
                .copied()
                .unwrap_or([255; 3]),
        });
    }
    out
}

/// The pixels a line strip lights, by the card's diamond-exit rule, pixel
/// centres at integer coordinates (the pre-transformed vertices'
/// convention the photograph shows: the point (379.66, 969) lights column
/// 380). A segment lights a pixel when it passes inside the pixel's
/// diamond, `|x − X| + |y − Y| < ½`, and its end does not lie inside it,
/// so a strip's shared point is lit once. A segment whose ends round to
/// one pixel can still light it (run1814b's ring at (397, 967)).
pub fn pixels(strip: &[(f32, f32)]) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for s in strip.windows(2) {
        let ((x0, y0), (x1, y1)) = (s[0], s[1]);
        let at = |t: f32| (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
        let l1 = |p: (f32, f32), x: f32, y: f32| (p.0 - x).abs() + (p.1 - y).abs();
        let (lx, hx) = (x0.min(x1).floor() as i32 - 1, x0.max(x1).ceil() as i32 + 1);
        let (ly, hy) = (y0.min(y1).floor() as i32 - 1, y0.max(y1).ceil() as i32 + 1);
        for py in ly..=hy {
            for px in lx..=hx {
                let (x, y) = (px as f32, py as f32);
                if l1((x1, y1), x, y) < 0.5 {
                    continue;
                }
                // The L1 distance along the segment is convex and piecewise
                // linear: its least is at an end or where it crosses the
                // pixel's row or column.
                let mut ts = vec![0.0, 1.0];
                if x1 != x0 {
                    ts.push((x - x0) / (x1 - x0));
                }
                if y1 != y0 {
                    ts.push((y - y0) / (y1 - y0));
                }
                let near = ts
                    .into_iter()
                    .filter(|t| (0.0..=1.0).contains(t))
                    .map(|t| l1(at(t), x, y))
                    .fold(f32::MAX, f32::min);
                if near < 0.5 {
                    out.push((px, py));
                }
            }
        }
    }
    out
}

/// **The rings drawn**: each strip's pixels, one square each, opaque.
pub fn draw(rings: &[Ring], size: (i32, i32)) -> crate::gpu::ScreenLayer {
    let mut l = crate::gpu::ScreenLayer {
        alpha: 255,
        pos: Vec::new(),
        col: Vec::new(),
    };
    for r in rings {
        let c = r.colour.map(|v| f32::from(v) / 255.0);
        for s in &r.strips {
            for (x, y) in pixels(s) {
                for p in super::pixel(size, x, y) {
                    l.pos.push(p);
                    l.col.push(c);
                }
            }
        }
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`RUN1814B_3294_RING_POINTS`**: run1814b's packet's ring point for
    /// point (`worldmap_ping_verts`, `0x00c8c3a0`, its 73 vertices; the
    /// draw resets the count and leaves the data) against ours, `[theirs,
    /// ours, ours within 0.01 px of theirs]`. Seen red at item 4629's
    /// build: [73, 0, 0], no ring.
    pub const RUN1814B_3294_RING_POINTS: [usize; 3] = [73, 73, 73];

    /// **`RUN1814B_3294_RING_PIXELS`**: the ring's pixels, the
    /// photograph's `(255, 100, 100)` against the still's, inside the
    /// ring's box (the blip's white and black are not the colour), `[theirs,
    /// ours, both]`. Seen red at item 4629's build, no ring drawn: [40, 0,
    /// 0].
    pub const RUN1814B_3294_RING_PIXELS: [usize; 3] = [40, 40, 40];

    fn lab(rel: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("ron-data/lab-experiments")
            .join(rel)
    }

    /// The probe's dump: `<k> <x> <y> <z> <w>` for the ring's 73 points.
    fn packet_points() -> Option<Vec<(f32, f32)>> {
        let path = std::env::var_os("RON_RING_PACKET_RUN1814B")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| lab("2026-11-23-item-4629/run1814b-ring.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no ring packet at {}", path.display());
            return None;
        };
        let pts: Vec<(f32, f32)> = text
            .lines()
            .filter_map(|l| {
                let w: Vec<&str> = l.split_whitespace().collect();
                let k: usize = w.first()?.parse().ok()?;
                if w.len() != 5 || k >= 73 {
                    return None;
                }
                Some((w[1].parse().ok()?, w[2].parse().ok()?))
            })
            .collect();
        Some(pts)
    }

    /// The radius run1814b's ring holds: range 10, a map of 60 cells, the
    /// quad 317 wide.
    #[test]
    fn the_radius_is_the_packet_s() {
        let r = radius(super::super::QUAD_FRAME, 60, 10 * 0xc0);
        assert!((r - 9.3411).abs() < 1e-3, "{r}");
    }

    /// A ring wholly inside is one strip of 73 points, the first again
    /// last, as run1814b's packet holds it; one across the quad's edge is cut where it leaves.
    #[test]
    fn a_ring_is_cut_at_the_quad_s_edge() {
        let size = super::super::QUAD_FRAME;
        let q = super::super::quad(size);
        let mid = ((q[1].0 + q[3].0) / 2, (q[0].1 + q[2].1) / 2);
        let s = strips(size, mid, 9.0);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].len(), 73);
        assert!((s[0][0].0 - s[0][72].0).abs() < 1e-4 && (s[0][0].1 - s[0][72].1).abs() < 1e-4);
        let edge = strips(size, q[3], 9.0);
        assert!(
            edge.len() <= 2 && edge.iter().all(|s| s.len() < 73),
            "{edge:?}"
        );
    }

    /// **The ring round the selected capital** (item 4629): run1814b's
    /// frame 3294, the capital selected; the ring's points against the
    /// packet's and its pixels against the photograph's.
    #[test]
    fn the_ring_round_the_capital_on_run1814b() {
        let Some(theirs) = packet_points() else {
            return;
        };
        let photo_path = lab("2026-11-23-item-4624/run1814b-3294.tga");
        let Ok(bytes) = std::fs::read(&photo_path) else {
            println!("skipping: no photograph at {}", photo_path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let mut world = match crate::world::World::open_game(
            &root,
            &crate::world::gamelog_dir(),
            crate::world::Game::Lobby,
        ) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        world.run_to(3294);
        world.blend = crate::world::Blend::Last;
        world.run_to(3295);
        world.fog = Some(crate::world::CONSOLE);
        let sel = super::super::blips::tests::capital_selected(&world);
        let size = super::super::QUAD_FRAME;
        let rings = build(&world, &sel, size);
        let ours: Vec<(f32, f32)> = rings
            .iter()
            .flat_map(|r| r.strips.iter().flatten().copied())
            .take(73)
            .collect();
        let close = theirs
            .iter()
            .zip(&ours)
            .filter(|(a, b)| (a.0 - b.0).abs() <= 0.01 && (a.1 - b.1).abs() <= 0.01)
            .count();
        let points = [theirs.len(), ours.len(), close];
        println!("RUN1814B_3294_RING_POINTS: {points:?}");
        if let Some(r) = rings.first() {
            println!("ring colour {:?}, first {:?}", r.colour, r.strips[0][0]);
        }
        // The ring's box, two pixels out.
        let (x0, x1) = (
            theirs.iter().map(|p| p.0).fold(f32::MAX, f32::min) as i32 - 2,
            theirs.iter().map(|p| p.0).fold(f32::MIN, f32::max) as i32 + 3,
        );
        let (y0, y1) = (
            theirs.iter().map(|p| p.1).fold(f32::MAX, f32::min) as i32 - 2,
            theirs.iter().map(|p| p.1).fold(f32::MIN, f32::max) as i32 + 3,
        );
        let mut camera = crate::pinned_camera();
        (camera.loc_x, camera.loc_y) = super::super::fog::tests::RUN1814B_LOC;
        let wh = (photo.width as u32, photo.height as u32);
        let still = crate::still_minimap(
            &mut world,
            &camera,
            wh,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &sel,
            &|_| None,
            true,
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-run1814b-3294-ring.bmp");
            std::fs::write(&out, crate::gpu::bmp(&still, wh)).expect("the still writes");
        }
        let w = photo.width;
        let red = |img: &[u8], x: i32, y: i32| {
            let i = (y as usize * w + x as usize) * 4;
            img[i..i + 3] == [255, 100, 100]
        };
        let (mut t, mut o, mut both) = (0, 0, 0);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (a, b) = (red(&photo.rgba, x, y), red(&still, x, y));
                t += usize::from(a);
                o += usize::from(b);
                both += usize::from(a && b);
            }
        }
        let px = [t, o, both];
        println!("RUN1814B_3294_RING_PIXELS: {px:?}");
        for y in y0..=y1 {
            let row: String = (x0..=x1)
                .map(|x| match (red(&photo.rgba, x, y), red(&still, x, y)) {
                    (true, true) => '#',
                    (true, false) => 't',
                    (false, true) => 'o',
                    _ => '.',
                })
                .collect();
            println!("{y} {row}");
        }
        assert_eq!(points, RUN1814B_3294_RING_POINTS);
        assert_eq!(px, RUN1814B_3294_RING_PIXELS);
    }
}
