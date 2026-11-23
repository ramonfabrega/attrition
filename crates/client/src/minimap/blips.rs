//! **The minimap's blips** (item 4601; `docs/RENDERER.md` §3.33).
//!
//! `WorldMap::render_blips@008cc700` lays every blip as a quad (or, for a
//! wonder, a triangle) in one of five layers, through
//! `add_blip@008ce500`, and draws the layers 0 to 4:
//!
//! - 0, the goody boxes, size 2, [`GOODY`];
//! - 1, the goods: oil a [`BLACK`] size 3 inside a [`WHITE`] rim, fish
//!   size 2 [`FISH`], whales size 3 [`FISH`], the rest size 3 [`RARE`];
//! - 3, the units: each captain on the map, size 2, its owner's colour;
//! - 2, the buildings (farms skipped; a tower a white size 3 under its
//!   colour at size 3; a wonder the triangle; a fort size 3; the rest
//!   size 2), then the cities, size 6;
//! - 4, the flash: not built (nothing on run1564's frame).
//!
//! A colour is the packet's word, red in its low byte; the console's own
//! unit or city in its selection is [`WHITE`]. [`build`] lays the quads
//! exactly as the packet holds them (`tools/render/blips_packet.py`), and
//! [`draw`] paints them one pixel a square, as the original's state does:
//! the atlas sampled bilinearly, times the vertex colour, opaque, the
//! pixel centres on the integers.

use sim::observe::ObjId;

/// The five layers `WorldMap +0xbf8` keeps.
pub const LAYERS: usize = 5;

/// `GOODY_COLOR`, (210, 210, 210): the packet's layer 0.
pub const GOODY: u32 = 0xffd2_d2d2;
/// A fish's (and a whale's) fill, (80, 160, 80): the packet's twelve
/// size-2 goods of layer 1, which are run1564's twelve `FISH`.
pub const FISH: u32 = 0xff50_a050;
/// Every other good's fill but oil's, (160, 160, 255): the packet's nine
/// size-3 goods of layer 1.
pub const RARE: u32 = 0xffff_a0a0;
pub const WHITE: u32 = 0xffff_ffff;
pub const BLACK: u32 = 0xff00_0000;
/// `.rdata 0xb69468`, 0.1: every texcoord of a black-filled box and of the
/// triangle, a white texel of the atlas.
pub const DOT: f32 = 0.1;

/// **The console sees everything** (item 4601): `render_blips`' arms skip
/// their `is_seen(console)` test under `semaphore[0] & 0x42` or the
/// console's player flags `0x100` or `0xd0`. Run1564's console has flags
/// 7 (its gamelog's `BEGIN PLAYER`), so the semaphore is what lets the
/// packet draw all 15 live goody boxes, all 36 goods and player 1's units
/// across the map where the console's own fog has seen none. The sim does
/// not carry the semaphore; taken as set (parked 4601a).
pub const SEES_ALL: bool = true;

/// **The minimap's filter** (item 4601): `player_profile.prefs.stat_mode`,
/// the profile's minimap mode. Under 1, the one built, `render_blips`
/// draws a unit only when its type is combat-role (`+0x2c8 & 0x10000`,
/// [`sim::combat::Profile::combat_role`]) and a building only when its type
/// attacks (`+0x1e8`) or its line's basic type trains soldiers
/// (`+0x2c0 & 0x40000000`, `MILITARY_TRAINER`). Run1564's packet is mode
/// 1's to the quad: no citizen, scout, farm, temple or city part has a
/// blip there, which is the nine units 4593c found missing. The captures'
/// profile is taken to hold 1 (parked 4601a); 0, every captain and every
/// building, is not built.
pub const STAT_MODE: u8 = 1;

/// `TypeIndex::OIL`, `BASE_RARE` (fish) and `WHALES`.
const OIL: usize = 5;
const FISH_TYPE: usize = 6;
const WHALES: usize = 0x1f;

/// One blip's quad as `Vert4Array` holds it: top-left, top-right,
/// bottom-right, bottom-left (a triangle repeats its last corner).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub xy: [(f32, f32); 4],
    pub uv: [(f32, f32); 4],
    pub colour: u32,
}

/// **The atlas** (`WorldMap::init@008d2160`): `<BLIPMAP>`'s texture and,
/// per `<SIZE>`, its box's offset.
pub struct Atlas {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
    pub sizes: Vec<(i32, i32, i32)>,
}

impl Atlas {
    /// `None` where `effects_graphics.xml` names no `<BLIPMAP>` or its
    /// texture does not read.
    pub fn load(root: &std::path::Path) -> Option<Atlas> {
        let xml = std::fs::read_to_string(root.join("Data/effects_graphics.xml")).ok()?;
        let start = xml.find("<BLIPMAP ")?;
        let end = start + xml[start..].find("</BLIPMAP>")?;
        let body = &xml[start..end];
        let head = &body[..body.find('>')?];
        let file = attr(head, "texture")?;
        let bytes = std::fs::read(rondata::artdata::resolve_art(root, file)?).ok()?;
        let img = render::tga::decode(&bytes).ok()?;
        let sizes = body
            .match_indices("<SIZE ")
            .filter_map(|(i, _)| {
                let t = &body[i..i + body[i..].find('>')?];
                let n = |k: &str| attr(t, k)?.parse::<i32>().ok();
                Some((n("size")?, n("xoffset")?, n("yoffset")?))
            })
            .collect();
        Some(Atlas {
            width: img.width,
            height: img.height,
            rgba: img.rgba,
            sizes,
        })
    }

    /// A size's box, `(u0, v0, u1, v1)`: `x/W` to `(x + s + 2)/W`, `y/H`
    /// to `(y + s + 2)/H`; a size with no entry keeps `(0, 0, 1, 1)`.
    pub fn rect(&self, size: i32) -> [f32; 4] {
        let (w, h) = (self.width as f32, self.height as f32);
        self.sizes
            .iter()
            .find(|e| e.0 == size)
            .map_or([0.0, 0.0, 1.0, 1.0], |&(s, x, y)| {
                [
                    x as f32 / w,
                    y as f32 / h,
                    (x + s + 2) as f32 / w,
                    (y + s + 2) as f32 / h,
                ]
            })
    }

    /// The texture at `(u, v)`, bilinear, its texel centres at the halves,
    /// clamped at the edges; each channel 0 to 1.
    pub fn sample(&self, u: f32, v: f32) -> [f32; 4] {
        let (w, h) = (self.width as i32, self.height as i32);
        let at = |x: i32, y: i32| {
            let (x, y) = (x.clamp(0, w - 1), y.clamp(0, h - 1));
            let i = (y * w + x) as usize * 4;
            std::array::from_fn::<f32, 4, _>(|k| f32::from(self.rgba[i + k]) / 255.0)
        };
        let (tx, ty) = (u * w as f32 - 0.5, v * h as f32 - 0.5);
        let (ix, iy) = (tx.floor() as i32, ty.floor() as i32);
        let (ax, ay) = (tx - ix as f32, ty - iy as f32);
        let (a, b, c, d) = (
            at(ix, iy),
            at(ix + 1, iy),
            at(ix, iy + 1),
            at(ix + 1, iy + 1),
        );
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * ax;
            let bot = c[k] + (d[k] - c[k]) * ax;
            top + (bot - top) * ay
        })
    }
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let i = tag.find(&key)? + key.len();
    Some(&tag[i..i + tag[i..].find('"')?])
}

/// **The window** `render_blips` hands `add_blip` (`+0x1344`..`+0x1350`):
/// half the quad's width and height, its left plus the half width, its top
/// plus the half height less one. On run1564: 158, 79, 484, 988.
pub fn window(size: (i32, i32)) -> [i32; 4] {
    let q = super::quad(size);
    let (l, r) = (q[3].0, q[1].0);
    let (t, b) = (q[0].1, q[2].1);
    let (w, h) = ((r - l) / 2, (b - t) / 2);
    [w, h, l + w, t + h - 1]
}

/// The quads, layer by layer, as they are laid.
struct Lay<'a> {
    atlas: &'a Atlas,
    m: [f32; 16],
    win: [f32; 4],
    layers: [Vec<Quad>; LAYERS],
}

impl Lay<'_> {
    /// `add_blip`'s centre: the world point through the minimap camera's
    /// `to_screen_matrix`, onto the window.
    fn centre(&self, x: i32, y: i32) -> (f32, f32) {
        let (x, y, m) = (x as f32, y as f32, &self.m);
        let w = x * m[3] + y * m[7] + m[15];
        let nx = (x * m[0] + y * m[4] + m[12]) / w;
        let ny = (x * m[1] + y * m[5] + m[13]) / w;
        let [wh, hh, xo, yo] = self.win;
        (xo + wh * nx - 0.5, yo - hh * ny - 0.5)
    }

    fn rect(&mut self, layer: usize, r: [f32; 4], uv: [f32; 4], colour: u32) {
        let [x0, y0, x1, y1] = r;
        let [u0, v0, u1, v1] = uv;
        self.layers[layer].push(Quad {
            xy: [(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
            uv: [(u0, v0), (u1, v0), (u1, v1), (u0, v1)],
            colour,
        });
    }

    /// `add_blip(x, y, fill, outline, size, size, shape, layer)`, its
    /// shape 0 (a box) or 1 (`triangle`).
    fn add(&mut self, layer: usize, at: sim::Pos, fill: u32, outline: u32, size: i32, tri: bool) {
        let (cx, cy) = self.centre(at.x, at.y);
        let h = size as f32 / 2.0;
        if tri {
            let p = [
                (cx, cy - h),
                (cx + h, cy + h),
                (cx - h, cy + h),
                (cx - h, cy + h),
            ];
            self.layers[layer].push(Quad {
                xy: p,
                uv: [(DOT, DOT); 4],
                colour: fill,
            });
            return;
        }
        let outer = [cx - h - 1.0, cy - h - 1.0, cx + h + 1.0, cy + h + 1.0];
        if fill == BLACK {
            self.rect(layer, outer, [DOT; 4], outline);
            self.rect(layer, [cx - h, cy - h, cx + h, cy + h], [DOT; 4], BLACK);
        } else {
            let uv = self.atlas.rect(size);
            self.rect(layer, outer, uv, fill);
        }
    }
}

/// A player's colour as the packet's word: `TeamColor::get_border`,
/// [`crate::borders::BORDER_COLOURS`], red in the low byte.
pub fn team(p: sim::Player) -> u32 {
    let [r, g, b] = crate::borders::BORDER_COLOURS
        .get(usize::from(p))
        .copied()
        .unwrap_or([255; 3]);
    0xff00_0000 | u32::from(b) << 16 | u32::from(g) << 8 | u32::from(r)
}

/// **The blips laid** (`render_blips@008cc700`), on a `size` frame, the
/// console [`crate::world::CONSOLE`], `selection` its selection, `blink`
/// the phase `graphic_tick & 8`.
///
/// - *Seen.* Every arm draws what the console's player sees, unless the
///   game's `semaphore[0] & 0x42` or the console's player flags `0x100` or
///   `0xd0` say it sees everything ([`SEES_ALL`]).
/// - *The players.* The units and buildings of slots 0 to 7 whose leader
///   lives; the cities of every slot.
/// - *The chain.* A captain with a unit captain above it on its cell's
///   chain of objects (`up`, `up_who`, [`sim::movement::collide::ObjectChain`])
///   has no blip of its own: one blip a pile.
pub fn build(
    world: &crate::world::World,
    selection: &[ObjId],
    size: (i32, i32),
    blink: bool,
) -> [Vec<Quad>; LAYERS] {
    let empty = || std::array::from_fn(|_| Vec::new());
    let Some(atlas) = world.blip_atlas.as_ref() else {
        return empty();
    };
    let console = crate::world::CONSOLE;
    let sim = world.runner.sim();
    let t = world.runner.terrain();
    let extent = (t.width * super::CELL, t.height * super::CELL);
    let mut lay = Lay {
        atlas,
        m: super::minimap_camera(extent).to_screen_matrix().0,
        win: window(size).map(|v| v as f32),
        layers: empty(),
    };
    let all_seen = SEES_ALL;
    let colour = |who: sim::Player, id: ObjId| {
        if who == console && selection.contains(&id) {
            WHITE
        } else {
            team(who)
        }
    };
    // Layer 0: the goody boxes.
    for g in &sim.gaia.goody_items {
        if g.alive && (all_seen || g.ever_seen >> console & 1 != 0) {
            lay.add(0, g.pos, GOODY, GOODY, 2, false);
        }
    }
    // Layer 1: the goods.
    for g in sim.observe_goods() {
        if !g.alive || !(all_seen || sim.good_ever_seen(g.pos, console)) {
            continue;
        }
        match g.ty {
            OIL => lay.add(1, g.pos, BLACK, WHITE, 3, false),
            FISH_TYPE => lay.add(1, g.pos, FISH, FISH, 2, false),
            WHALES => lay.add(1, g.pos, FISH, FISH, 3, false),
            _ => lay.add(1, g.pos, RARE, RARE, 3, false),
        }
    }
    // The slots `render_blips` walks for units and buildings: 0 to 7.
    let players: Vec<sim::Player> = (0..8)
        .filter(|&p| {
            sim.units.iter().any(|u| u.owner == p) || sim.buildings.iter().any(|b| b.owner == p)
        })
        .collect();
    // Layer 3: the units, each captain on the map the console sees and no
    // captain above it on its chain.
    let scene = world.runner.scene(Some(console));
    let seen: std::collections::HashSet<(sim::Player, i16)> = scene
        .units
        .iter()
        .filter(|u| all_seen || u.seen)
        .map(|u| (u.id.who, u.id.o))
        .collect();
    let chain = |who: i8, o: i16| -> Option<(bool, i16, i8)> {
        let who = sim::Player::try_from(who).ok()?;
        if let Some(u) = sim.unit_by_o(who, o) {
            let u = &sim.units[u];
            return Some((u.captain, u.chain.up, u.chain.up_who));
        }
        let b = &sim.buildings[sim.building_by_o(who, o)?];
        Some((false, b.chain.up, b.chain.up_who))
    };
    for &p in &players {
        for u in sim.units.iter().filter(|u| u.owner == p) {
            if !u.alive() || !u.on_map || !u.captain || !seen.contains(&(u.owner, u.index)) {
                continue;
            }
            let role =
                u.ty.is_some_and(|t| sim.tables.unit_types[t].combat.combat_role);
            if STAT_MODE == 1 && !role {
                continue;
            }
            // The blink: a unit whose current order is an attack (the
            // order's slot `+0x18`, `UnitOrder::is_attack`) has no blip
            // while `graphic_tick & 8`.
            if blink
                && matches!(
                    u.orders.front().map(|o| &o.body),
                    Some(sim::orders::Body::Attack(_))
                )
            {
                continue;
            }
            let (mut up, mut up_who) = (u.chain.up, u.chain.up_who);
            let mut covered = false;
            while up >= 0 {
                let Some((captain, next, next_who)) = chain(up_who, up) else {
                    break;
                };
                if captain {
                    covered = true;
                    break;
                }
                (up, up_who) = (next, next_who);
            }
            if covered {
                continue;
            }
            let id = ObjId {
                who: u.owner,
                o: u.index,
            };
            lay.add(3, u.pos, colour(p, id), WHITE, 2, false);
        }
    }
    // Layer 2: the buildings, then the cities.
    let types = &sim.tables.build_types;
    use sim::cities::build::{Ident, is, is_fort};
    for &p in &players {
        for (b, bd) in sim.buildings.iter().enumerate() {
            if bd.owner != p || !bd.alive || !(all_seen || sim.build_is_seen(b, console)) {
                continue;
            }
            let Some(ty) = bd.ty else { continue };
            if sim.is_gather_type(b) && sim.is_flat(b) {
                continue;
            }
            // A city's centre, active (`SubObjectData::flags & 0x20` with
            // `Build`'s `flags & 4`): drawn as its city below.
            let centre = bd
                .city
                .is_some_and(|c| sim.cities.cities.get(c).is_some_and(|c| c.building == b));
            if centre && bd.active {
                continue;
            }
            if STAT_MODE == 1 && types[ty].attack == 0 {
                let mut basic = ty;
                while let Some(f) = types[basic].from {
                    basic = f;
                }
                if !types[basic].has(sim::cities::build::flags::MILITARY_TRAINER) {
                    continue;
                }
            }
            let c = colour(
                p,
                ObjId {
                    who: bd.owner,
                    o: bd.index,
                },
            );
            if is(types, ty, Ident::Tower) {
                lay.add(2, bd.pos, WHITE, WHITE, 3, false);
                lay.add(2, bd.pos, c, WHITE, 3, false);
            } else if sim.is_wonder_building(ty) {
                lay.add(2, bd.pos, c, WHITE, if bd.active { 3 } else { 2 }, true);
            } else if is_fort(types, ty) {
                lay.add(2, bd.pos, c, WHITE, 3, false);
            } else {
                lay.add(2, bd.pos, c, WHITE, 2, false);
            }
        }
    }
    for city in &sim.cities.cities {
        let Some(bd) = sim.buildings.get(city.building) else {
            continue;
        };
        if !city.alive || !(all_seen || sim.build_is_seen(city.building, console)) {
            continue;
        }
        let c = colour(
            city.owner,
            ObjId {
                who: bd.owner,
                o: bd.index,
            },
        );
        lay.add(2, city.pos, c, WHITE, 6, false);
    }
    lay.layers
}

/// **The blips drawn**: each layer's quads in order, one square a pixel
/// whose centre `(i, j)` falls inside the quad (`x0 ≤ i < x1`, the
/// top-left rule), the atlas sampled bilinearly at the pixel's texcoord
/// times the vertex colour, opaque.
pub fn draw(
    layers: &[Vec<Quad>; LAYERS],
    atlas: &Atlas,
    size: (i32, i32),
) -> crate::gpu::ScreenLayer {
    let mut l = crate::gpu::ScreenLayer {
        alpha: 255,
        pos: Vec::new(),
        col: Vec::new(),
    };
    for q in layers.iter().flatten() {
        let fill = [q.colour & 0xff, q.colour >> 8 & 0xff, q.colour >> 16 & 0xff]
            .map(|c| c as f32 / 255.0);
        let [(x0, y0), _, (x1, y1), _] = q.xy;
        let tri = q.xy[2] == q.xy[3];
        let (left, right) = if tri {
            (q.xy[2].0, q.xy[1].0)
        } else {
            (x0, x1)
        };
        let (top, bottom) = if tri {
            (q.xy[0].1, q.xy[2].1)
        } else {
            (y0, y1)
        };
        let (i0, i1) = (left.ceil() as i32, right.ceil() as i32);
        let (j0, j1) = (top.ceil() as i32, bottom.ceil() as i32);
        for j in j0..j1 {
            for i in i0..i1 {
                let (fx, fy) = (i as f32, j as f32);
                if tri {
                    // Inside the apex's two edges.
                    let (ax, ay) = q.xy[0];
                    let s = (fy - ay) / (bottom - ay) * (right - ax);
                    if fx < ax - s || fx >= ax + s {
                        continue;
                    }
                }
                let a = (fx - left) / (right - left);
                let b = (fy - top) / (bottom - top);
                let (u0, v0) = q.uv[0];
                let (u1, v1) = q.uv[2];
                let t = atlas.sample(u0 + (u1 - u0) * a, v0 + (v1 - v0) * b);
                let c = [t[0] * fill[0], t[1] * fill[1], t[2] * fill[2]];
                for p in super::pixel(size, i, j) {
                    l.pos.push(p);
                    l.col.push(c);
                }
            }
        }
    }
    l
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The probe's dump of run1564's packet (item 4596).
    fn packet() -> Option<(Vec<i32>, [Vec<Quad>; LAYERS])> {
        let path = std::env::var_os("RON_BLIPS_PACKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-23-item-4596/run1564-blips.txt")
            });
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no blips packet at {}", path.display());
            return None;
        };
        let mut window = Vec::new();
        let mut layers: [Vec<Quad>; LAYERS] = std::array::from_fn(|_| Vec::new());
        for line in text.lines() {
            let w: Vec<&str> = line.split_whitespace().collect();
            match w.first() {
                Some(&"window") => window = w[1..].iter().map(|v| v.parse().unwrap()).collect(),
                Some(&"blip") => {
                    let f = |k: usize| f32::from_bits(u32::from_str_radix(w[k], 16).unwrap());
                    let layer: usize = w[1].parse().unwrap();
                    let c: Vec<u32> = (19..23)
                        .map(|k| u32::from_str_radix(w[k], 16).unwrap())
                        .collect();
                    assert!(c.iter().all(|&v| v == c[0]), "one colour a quad: {line}");
                    layers[layer].push(Quad {
                        xy: std::array::from_fn(|k| (f(3 + 2 * k), f(4 + 2 * k))),
                        uv: std::array::from_fn(|k| (f(11 + 2 * k), f(12 + 2 * k))),
                        colour: c[0],
                    });
                }
                _ => {}
            }
        }
        Some((window, layers))
    }

    /// **The packet, quad for quad** (item 4601): every blip run1564's
    /// packet holds at frame 3294, its rectangle within 0.01 of ours and
    /// its colour and texcoords exact, layer by layer in the order laid.
    #[test]
    fn the_blips_are_the_packet_s() {
        use crate::ground::tests::{probe_at, run1564};
        let Some((window_bits, theirs)) = packet() else {
            return;
        };
        assert_eq!(window(super::super::QUAD_FRAME).to_vec(), window_bits);
        // The world at the photograph's frame, `PHOTO_FRAME` + 1, as every
        // run1564 row reads it: the packet's units sit on its positions to
        // the 0.01 (at 3294 they have moved on).
        let Some(p) = probe_at(run1564()) else {
            return;
        };
        let sel = capital_selected(&p.world);
        let ours = build(&p.world, &sel, super::super::QUAD_FRAME, BLINK_RUN1564);
        let mut bad = Vec::new();
        for l in 0..LAYERS {
            println!(
                "layer {l}: ours {} theirs {}",
                ours[l].len(),
                theirs[l].len()
            );
            for k in 0..ours[l].len().max(theirs[l].len()) {
                let (a, b) = (ours[l].get(k), theirs[l].get(k));
                let same = match (a, b) {
                    (Some(a), Some(b)) => {
                        a.colour == b.colour
                            && a.uv == b.uv
                            && (0..4).all(|v| {
                                (a.xy[v].0 - b.xy[v].0).abs() <= 0.01
                                    && (a.xy[v].1 - b.xy[v].1).abs() <= 0.01
                            })
                    }
                    _ => false,
                };
                if !same {
                    bad.push(format!("layer {l} quad {k}: ours {a:?} theirs {b:?}"));
                }
            }
        }
        for b in &bad {
            println!("{b}");
        }
        assert!(bad.is_empty(), "{} quads part", bad.len());
    }

    /// The packet's phase: three of player 1's attackers at player 0's
    /// capital have no blip there, so `graphic_tick & 8` was set.
    pub(crate) const BLINK_RUN1564: bool = true;

    /// The console's capital, selected: the photograph's white city.
    pub(crate) fn capital_selected(world: &crate::world::World) -> Vec<ObjId> {
        let sim = world.runner.sim();
        sim.cities
            .cities
            .iter()
            .filter(|c| c.alive && c.capital && c.owner == crate::world::CONSOLE)
            .filter_map(|c| sim.buildings.get(c.building))
            .map(|b| ObjId {
                who: b.owner,
                o: b.index,
            })
            .collect()
    }
}
