//! **The minimap's pings** (item 4603; `docs/RENDERER.md` §3.34).
//!
//! `WorldMap::render_ping@008ca6d0`, the last of `on_redraw`'s draws,
//! walks the message window's `events` list (`MessageWin +0x48c`), one
//! [`Entry`] a `MessageWinEntry`. An entry whose `ping` is not 0 and whose
//! time has not run out lays its kind's [`TexPing`] at its point through
//! `add_texture_ping@008c8f70`, and `render_texture_pings@008c8dc0` draws
//! them texture by texture. Each is a square (rotated for the wonder's)
//! whose colour is the vertex colour and whose alpha is the texture's times
//! the vertex's, blended `SRCALPHA`, `INVSRCALPHA` (`tex_ping_rs`,
//! `WorldMap::init@008d2cd0`).
//!
//! The entries come from the sim's own alerts (`MessageWin::add_message`,
//! `add_event`: the attack alert in `Object::do_damage`, the wonder in
//! `Build::activate`, the rare in `World::reveal_fog`, the city's loss and
//! capture). The sim does not supply them through `observe` (parked
//! 4603a), so the client holds none ([`Live`]). [`build`] and [`draw`] are
//! the client's half, held to run1564's packet quad for quad.
//!
//! Not built: the circle (`ping` -4, `draw_minimap_circle`), the doober's
//! boxes, the selection's gather lines and type circles, the scene's own
//! pings and the splines. None is on run1564's packet.

/// One `MessageWinEntry` as `render_ping` reads it: the world point
/// (`x` `+0x4`, `y` `+0x8`), `ping` `+0x10` (the kind; 0 none), the
/// `graphic_tick` it was stamped at (`ping_stamp` `+0x18`), `ping_times`
/// `+0x1c`, `doober` `+0x20` and the colour's word (`+0x68`, red in its
/// low byte).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub x: i32,
    pub y: i32,
    pub ping: i32,
    pub stamp: i32,
    pub times: i32,
    pub doober: i32,
    pub colour: u32,
}

/// The entries the client holds and the `graphic_tick` it draws them at.
/// Empty: our sim supplies none (4603a).
#[derive(Clone, Debug, Default)]
pub struct Live {
    pub entries: Vec<Entry>,
    pub tick: i32,
}

/// One `TexPingPhase` (`0x20`): which texture, whether the alpha fades and
/// the size scales, between which phases it shows, and its colour (0: the
/// entry's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phase {
    pub texture: usize,
    pub use_alpha: bool,
    pub use_scale: bool,
    pub scale_by: f32,
    pub scale_speed: f32,
    pub start: f32,
    pub end: f32,
    pub colour: u32,
}

/// One `TexPing`: its phases, and whether it turns and how fast.
#[derive(Clone, Debug, PartialEq)]
pub struct TexPing {
    pub name: String,
    pub phases: Vec<Phase>,
    pub rotates: bool,
    pub rotate_speed: f32,
}

/// One texture, top row first.
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Texture {
    /// The texture at `(u, v)`, bilinear, its texel centres at the halves,
    /// clamped at the edges; each channel 0 to 1 (as the blips' atlas).
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

/// **The pings' definitions** (`WorldMap::init_texture_pings@008c9960`):
/// `effects_graphics.xml`'s `<MINIMAP>`, one `<PING>` a [`TexPing`], each
/// `<PHASE>`'s texture numbered by its path's first appearance (on the
/// install: the four `AttackAlertCircle`s 0 to 3, `WonderAlertCircle` 4,
/// `XResourceAlert` 5, as run1564's packet holds them).
pub struct Set {
    pub pings: Vec<TexPing>,
    pub paths: Vec<String>,
    /// The textures, by number; `None` where one does not read.
    pub textures: Vec<Option<Texture>>,
}

/// A colour attribute `0xAARRGGBB` as the packet's word, red in its low
/// byte (`String::hex`, then the `Color`'s word).
fn word(hex: &str) -> Option<u32> {
    let v = u32::from_str_radix(hex.trim().trim_start_matches("0x"), 16).ok()?;
    let (a, r, g, b) = (v >> 24, v >> 16 & 0xff, v >> 8 & 0xff, v & 0xff);
    Some(a << 24 | b << 16 | g << 8 | r)
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let i = tag.find(&key)? + key.len();
    Some(&tag[i..i + tag[i..].find('"')?])
}

/// The XML with its comments cut (the install keeps a second `attack`
/// inside one).
fn uncommented(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        match rest[i..].find("-->") {
            Some(j) => rest = &rest[i + j + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

impl Set {
    /// The definitions alone, off `effects_graphics.xml`'s text.
    pub fn parse(xml: &str) -> Option<Set> {
        let xml = uncommented(xml);
        let start = xml.find("<MINIMAP>")?;
        let end = start + xml[start..].find("</MINIMAP>")?;
        let body = &xml[start..end];
        let mut pings = Vec::new();
        let mut paths: Vec<String> = Vec::new();
        for (i, _) in body.match_indices("<PING ") {
            let close = i + body[i..].find("</PING>")?;
            let head = &body[i..i + body[i..].find('>')?];
            let f = |t: &str, k: &str| attr(t, k).and_then(|v| v.trim().parse::<f32>().ok());
            let mut phases = Vec::new();
            for (j, _) in body[i..close].match_indices("<PHASE ") {
                let t = &body[i + j..i + j + body[i + j..].find('>')?];
                let path = attr(t, "texture")?.to_string();
                let texture = match paths.iter().position(|p| *p == path) {
                    Some(n) => n,
                    None => {
                        paths.push(path);
                        paths.len() - 1
                    }
                };
                phases.push(Phase {
                    texture,
                    use_alpha: attr(t, "usealpha")? != "0",
                    use_scale: attr(t, "usescale")? != "0",
                    scale_by: f(t, "scaleby")?,
                    scale_speed: f(t, "scalespeed")?,
                    start: f(t, "start_pct")?,
                    end: f(t, "end_pct")?,
                    colour: word(attr(t, "color")?)?,
                });
            }
            pings.push(TexPing {
                name: attr(head, "name")?.to_string(),
                phases,
                rotates: attr(head, "rotates")? != "0",
                rotate_speed: f(head, "rotate_speed")?,
            });
        }
        Some(Set {
            pings,
            paths,
            textures: Vec::new(),
        })
    }

    /// `None` where `effects_graphics.xml` holds no `<MINIMAP>`.
    pub fn load(root: &std::path::Path) -> Option<Set> {
        let xml = std::fs::read_to_string(root.join("Data/effects_graphics.xml")).ok()?;
        let mut set = Set::parse(&xml)?;
        set.textures = set
            .paths
            .iter()
            .map(|p| {
                let bytes = std::fs::read(rondata::artdata::resolve_art(root, p)?).ok()?;
                let img = render::tga::decode(&bytes).ok()?;
                Some(Texture {
                    width: img.width,
                    height: img.height,
                    rgba: img.rgba,
                })
            })
            .collect();
        Some(set)
    }

    /// `render_ping`'s choice by the entry's kind: -8 the wonder's, -9 the
    /// rare's, -10 the attack's, -6 the city lost, -7 the city won, -2 and
    /// -3 the waypoint's; any other the default.
    pub fn for_kind(&self, ping: i32) -> Option<&TexPing> {
        let name = match ping {
            -8 => "wonder",
            -9 => "resource",
            -10 => "attack",
            -6 => "citylost",
            -7 => "citywon",
            -2 | -3 => "waypoint",
            _ => "default",
        };
        self.pings.iter().find(|p| p.name == name)
    }
}

/// One ping's square as `tex_ping_verts` holds it: top-left, top-right,
/// bottom-right, bottom-left (the six vertices are these, then the first
/// and the third again), its texcoords the unit square's corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub xy: [(f32, f32); 4],
    pub colour: u32,
}

/// `Texture::get_width` of the minimap's background (its slot `+0x40`),
/// 256 on the packet ([`super::BACK`]); a ping's half side is it over 16.
pub const HALF: f32 = (super::BACK >> 4) as f32;

/// **The point a ping is laid at**: `MiscAccess::world_to_console@00583840`
/// through the minimap's camera ([`super::minimap_camera`]) onto
/// `map_rect`, rounded half up, plus the window's place on the screen
/// (`Window::client_to_screen`). On run1564's packet `map_rect` is the
/// quad's bounds, 317 × 159 at (326, 910): the one rectangle of 314 to 319
/// by 156 to 161 at (324..328, 908..912) that puts all six pinging
/// entries' squares where the packet's are. Through the quad's own
/// mapping ([`super::to_screen`]) they part by a pixel, either way.
pub fn centre(size: (i32, i32), extent: (i32, i32), x: i32, y: i32) -> (i32, i32) {
    let q = super::quad(size);
    let (l, r) = (q[3].0, q[1].0);
    let (t, b) = (q[0].1, q[2].1);
    let m = super::minimap_camera(extent).to_screen_matrix();
    let p = render::camera::world_to_console(
        &m,
        &crate::pinned_screen(),
        x,
        y,
        0,
        Some((r - l, b - t)),
    );
    (l + p.x, t + p.y)
}

/// The phase `render_ping` hands `add_texture_ping` at `elapsed` ticks
/// since the stamp: 1 on the stamp, then down by an eighth a tick, round
/// and round (`(8 − elapsed % 8) · 0.125`, the remainder C's).
pub fn phase(elapsed: i32) -> f32 {
    (8.0 - (elapsed % 8) as f32) * 0.125
}

/// Whether an entry still pings `elapsed` ticks after its stamp: its eighths
/// fewer than `(k + 1) · ping_times`, `k` 1, or twice the doober.
pub fn alive(e: &Entry, elapsed: i32) -> bool {
    let k = if e.doober == 0 { 1 } else { e.doober * 2 };
    elapsed / 8 < (k + 1) * e.times
}

/// **`add_texture_ping`**: every phase of `tp` whose span holds `p`
/// (`end ≤ p ≤ start`) lays a square at `(cx, cy)` on its texture. The
/// alpha rises from 0 to 255 over the span's first half and falls over its
/// second when the phase fades (255 otherwise); the colour is the phase's,
/// or `colour` where the phase's is 0. The half side is [`HALF`], scaled
/// by `(1 − (1 − p) · scale_speed) · scale_by` when the phase scales; a
/// turning ping is turned `(1 − p) · 360 · rotate_speed` whole degrees.
pub fn add(out: &mut [Vec<Quad>], tp: &TexPing, p: f32, cx: i32, cy: i32, colour: u32) {
    for ph in &tp.phases {
        if !(p <= ph.start && ph.end <= p) {
            continue;
        }
        let t = (p - ph.start) / (ph.end - ph.start);
        let alpha = if !ph.use_alpha {
            255
        } else if t < 0.5 {
            (t * 255.0 + t * 255.0) as i32
        } else {
            let f = (1.0 - t) * 255.0;
            (f + f) as i32
        } as u32;
        let c = if ph.colour == 0 { colour } else { ph.colour };
        let c = alpha << 24 | c & 0x00ff_ffff;
        let mut half = HALF;
        if ph.use_scale {
            half *= (1.0 - (1.0 - p) * ph.scale_speed) * ph.scale_by;
        }
        let (fx, fy) = (cx as f32, cy as f32);
        let corners = [(-half, -half), (half, -half), (half, half), (-half, half)];
        let xy = if tp.rotates {
            let deg = ((1.0 - p) * 360.0 * tp.rotate_speed) as i32;
            let (s, k) = (deg as f32).to_radians().sin_cos();
            corners.map(|(dx, dy)| (fx + dx * k + dy * s, fy - dx * s + dy * k))
        } else {
            corners.map(|(dx, dy)| (fx + dx, fy + dy))
        };
        if let Some(q) = out.get_mut(ph.texture) {
            q.push(Quad { xy, colour: c });
        }
    }
}

/// **The pings laid** (`render_ping@008ca6d0`), on a `size` frame over a
/// map of `extent`, at `tick`: every entry in the list's order whose
/// `ping` is neither 0 nor -4 and which is [`alive`], its kind's
/// [`TexPing`] at its [`phase`]. One list of squares a texture, in the
/// order `render_texture_pings` draws them.
pub fn build(
    set: &Set,
    entries: &[Entry],
    tick: i32,
    size: (i32, i32),
    extent: (i32, i32),
) -> Vec<Vec<Quad>> {
    let mut out = vec![Vec::new(); set.paths.len()];
    for e in entries {
        if e.ping == 0 || e.ping == -4 {
            continue;
        }
        let elapsed = tick - e.stamp;
        if !alive(e, elapsed) {
            continue;
        }
        let Some(tp) = set.for_kind(e.ping) else {
            continue;
        };
        let (cx, cy) = centre(size, extent, e.x, e.y);
        add(&mut out, tp, phase(elapsed), cx, cy, e.colour);
    }
    out
}

/// **The pings drawn**: texture by texture, each square's pixels whose
/// centre `(i, j)` (on the integers) falls inside it, the texture sampled
/// at the pixel's place across the square; the colour the vertex's, the
/// alpha the texel's times the vertex's. One layer a square and an alpha,
/// in the order drawn, so overlapping pings blend in the original's order.
pub fn draw(quads: &[Vec<Quad>], set: &Set, size: (i32, i32)) -> Vec<crate::gpu::ScreenLayer> {
    let mut out = Vec::new();
    for (t, list) in quads.iter().enumerate() {
        let Some(Some(tex)) = set.textures.get(t) else {
            continue;
        };
        for q in list {
            let rgb = [q.colour & 0xff, q.colour >> 8 & 0xff, q.colour >> 16 & 0xff]
                .map(|c| c as f32 / 255.0);
            let va = (q.colour >> 24) as f32 / 255.0;
            let [p0, p1, _, p3] = q.xy;
            let (ex, ey) = (p1.0 - p0.0, p1.1 - p0.1);
            let (fx, fy) = (p3.0 - p0.0, p3.1 - p0.1);
            let (le, lf) = (ex * ex + ey * ey, fx * fx + fy * fy);
            if le <= 0.0 || lf <= 0.0 {
                continue;
            }
            let xs = q.xy.map(|p| p.0);
            let ys = q.xy.map(|p| p.1);
            let lo = |v: [f32; 4]| v.iter().copied().fold(f32::INFINITY, f32::min).floor() as i32;
            let hi =
                |v: [f32; 4]| v.iter().copied().fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
            let mut by: std::collections::BTreeMap<u8, crate::gpu::ScreenLayer> =
                Default::default();
            for j in lo(ys)..=hi(ys) {
                for i in lo(xs)..=hi(xs) {
                    let (dx, dy) = (i as f32 - p0.0, j as f32 - p0.1);
                    let a = (dx * ex + dy * ey) / le;
                    let b = (dx * fx + dy * fy) / lf;
                    if !(0.0..1.0).contains(&a) || !(0.0..1.0).contains(&b) {
                        continue;
                    }
                    let s = tex.sample(a, b);
                    let alpha = (s[3] * va * 255.0).round().clamp(0.0, 255.0) as u8;
                    if alpha == 0 {
                        continue;
                    }
                    let l = by.entry(alpha).or_insert_with(|| crate::gpu::ScreenLayer {
                        alpha,
                        pos: Vec::new(),
                        col: Vec::new(),
                    });
                    for p in super::pixel(size, i, j) {
                        l.pos.push(p);
                        l.col.push(rgb);
                    }
                }
            }
            out.extend(by.into_values());
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The probe's dump of run1564's packet (`tools/render/pings_packet.py`,
    /// item 4603): its `graphic_tick`, its entries, its pings' phases and,
    /// per texture, the vertices `render_texture_pings` left behind.
    /// One leftover vertex: its place, its texcoord, its colour.
    pub(crate) type Vert = ((f32, f32), (f32, f32), u32);

    pub(crate) struct Packet {
        pub tick: i32,
        /// `player_profile.prefs.stat_mode` and the game's `semaphore`
        /// bytes 0 and 1 (4601a).
        pub stat_mode: i32,
        pub semaphore: [u8; 2],
        pub entries: Vec<Entry>,
        pub phases: Vec<(String, Vec<Phase>)>,
        pub verts: Vec<Vec<Vert>>,
    }

    pub(crate) fn packet() -> Option<Packet> {
        let path = std::env::var_os("RON_PINGS_PACKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-23-item-4603/run1564-pings.txt")
            });
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no pings packet at {}", path.display());
            return None;
        };
        let mut p = Packet {
            tick: 0,
            stat_mode: -1,
            semaphore: [0; 2],
            entries: Vec::new(),
            phases: Vec::new(),
            verts: Vec::new(),
        };
        let key = |w: &[&str], k: &str| -> String {
            let i = w.iter().position(|v| *v == k).expect(k);
            w[i + 1].to_string()
        };
        for line in text.lines() {
            let w: Vec<&str> = line.split_whitespace().collect();
            let n = |k: &str| key(&w, k).parse::<i32>().expect(k);
            match w.first() {
                Some(&"frame") => p.tick = n("graphic_tick"),
                Some(&"profile") => {
                    p.stat_mode = n("stat_mode");
                    let h = |k: &str| u8::from_str_radix(&key(&w, k), 16).expect(k);
                    p.semaphore = [h("semaphore0"), h("semaphore1")];
                }
                Some(&"event") => p.entries.push(Entry {
                    x: n("x"),
                    y: n("y"),
                    ping: n("ping"),
                    stamp: n("stamp"),
                    times: n("times"),
                    doober: n("doober"),
                    colour: u32::from_str_radix(&key(&w, "color"), 16).unwrap(),
                }),
                Some(&"texping") => p.phases.push((w[1].to_string(), Vec::new())),
                Some(&"phase") => {
                    let f = |k: &str| key(&w, k).parse::<f32>().expect(k);
                    p.phases.last_mut().unwrap().1.push(Phase {
                        texture: key(&w, "tex").parse().unwrap(),
                        use_alpha: n("alpha") != 0,
                        use_scale: n("scale") != 0,
                        scale_by: f("by"),
                        scale_speed: f("speed"),
                        start: f("start"),
                        end: f("end"),
                        colour: u32::from_str_radix(&key(&w, "color"), 16).unwrap(),
                    });
                }
                Some(&"texture") => p.verts.push(Vec::new()),
                Some(&"vert") => {
                    let f = |k: usize| w[k].parse::<f32>().unwrap_or(f32::NAN);
                    p.verts.last_mut().unwrap().push((
                        (f(3), f(4)),
                        (f(8), f(9)),
                        u32::from_str_radix(w[10], 16).unwrap(),
                    ));
                }
                _ => {}
            }
        }
        Some(p)
    }

    pub(crate) fn set() -> Option<Set> {
        let root = std::path::PathBuf::from(std::env::var_os("RON_INSTALL")?);
        Set::load(&root)
    }

    /// Our squares, one texture's, against the packet's leftover vertices:
    /// the six of each square (top-left, top-right, bottom-right,
    /// bottom-left, top-left, bottom-right), each within 0.01, its
    /// texcoords the unit square's, its colour exact.
    fn parts(ours: &[Quad], theirs: &[Vert]) -> Vec<String> {
        let uv = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let mut bad = Vec::new();
        for (k, q) in ours.iter().enumerate() {
            for (v, c) in [0, 1, 2, 3, 0, 2].into_iter().enumerate() {
                let Some(&(xy, t, col)) = theirs.get(6 * k + v) else {
                    bad.push(format!("quad {k} vertex {v}: past the packet's array"));
                    continue;
                };
                let near = (q.xy[c].0 - xy.0).abs() <= 0.01 && (q.xy[c].1 - xy.1).abs() <= 0.01;
                if !near || t != uv[c] || col != q.colour {
                    bad.push(format!(
                        "quad {k} vertex {v}: ours {:?} {:?} {:08x}, theirs {xy:?} {t:?} {col:08x}",
                        q.xy[c], uv[c], q.colour
                    ));
                }
            }
        }
        bad
    }

    /// **The pings, quad for quad** (item 4603): run1564's packet's eleven
    /// entries, laid at each `graphic_tick` from the newest stamp to the
    /// packet's own, against the vertices `render_texture_pings` left in
    /// each texture's array (it zeroes the counts, not the data). Exactly
    /// one tick lays every texture's squares as the packet's leading
    /// vertices hold them: 175, the stamp of the two newest attacks (the
    /// packet reads 177; the last minimap drawn was two ticks before).
    /// 11 of 11 squares: five on texture 0, one each on 1 to 3, the three
    /// turned wonders on 4.
    #[test]
    fn the_pings_are_the_packet_s() {
        let (Some(p), Some(set)) = (packet(), set()) else {
            println!("skipping: no pings packet or no RON_INSTALL");
            return;
        };
        // The definitions are the packet's, phase for phase, in its order
        // (attack, wonder, citycap, cityloss, waypoint, normal, rare).
        for (name, phases) in &p.phases {
            let kind = match name.as_str() {
                "attack" => -10,
                "wonder" => -8,
                "citycap" => -7,
                "cityloss" => -6,
                "waypoint" => -2,
                "normal" => -1,
                "rare" => -9,
                n => panic!("{n}"),
            };
            let ours = set.for_kind(kind).expect(name);
            assert_eq!(&ours.phases, phases, "{name}");
        }
        assert_eq!(set.paths.len(), p.verts.len(), "one array a texture");
        let size = super::super::QUAD_FRAME;
        let extent = (46080, 46080);
        // A ping's stamp is a `graphic_tick`; an entry of no ping (or the
        // circle's -4) is stamped with the frame instead (`add_event`).
        let newest = p
            .entries
            .iter()
            .filter(|e| e.ping != 0 && e.ping != -4)
            .map(|e| e.stamp)
            .max()
            .unwrap_or(0);
        let mut fits = Vec::new();
        for tick in newest..=p.tick {
            let ours = build(&set, &p.entries, tick, size, extent);
            let bad: Vec<String> = ours
                .iter()
                .zip(&p.verts)
                .flat_map(|(o, t)| parts(o, t))
                .collect();
            let n: usize = ours.iter().map(Vec::len).sum();
            println!("tick {tick}: {n} squares, {} vertices part", bad.len());
            if tick == DRAWN_TICK_RUN1564 {
                for b in &bad {
                    println!("{b}");
                }
                let counts: Vec<usize> = ours.iter().map(Vec::len).collect();
                assert_eq!(counts, [5, 1, 1, 1, 3, 0], "squares a texture");
                println!("pings: {n} of {n} squares the packet's");
            }
            if bad.is_empty() {
                fits.push(tick);
            }
        }
        assert_eq!(fits, [DRAWN_TICK_RUN1564]);
    }

    /// **The `graphic_tick` the photograph shows** (item 4603): the last
    /// minimap run1564's process drew, two ticks before its packet's 177
    /// ([`the_pings_are_the_packet_s`]); the photograph's pings are this
    /// tick's to the eye.
    pub(crate) const DRAWN_TICK_RUN1564: i32 = 175;

    /// **4601a, read** (item 4603): run1564's profile holds `stat_mode` 1,
    /// [`super::super::blips::STAT_MODE`]; the game's `semaphore` byte 0 is
    /// 0x0a, so `& 0x42` is set and the console sees everything
    /// ([`super::super::blips::SEES_ALL`]), and byte 1 lacks 8, so a good
    /// still asks `type_avail`. The blink 4601 inferred is the drawn tick's
    /// bit 3: 175 & 8 is set ([`super::super::blips::tests::BLINK_RUN1564`]).
    #[test]
    fn the_packet_s_profile_is_the_blips_assumptions() {
        use super::super::blips;
        let Some(p) = packet() else {
            return;
        };
        println!(
            "stat_mode {} semaphore {:02x} {:02x}",
            p.stat_mode, p.semaphore[0], p.semaphore[1]
        );
        assert_eq!(p.stat_mode, i32::from(blips::STAT_MODE));
        assert_eq!(p.semaphore[0] & 0x42 != 0, blips::SEES_ALL);
        assert_eq!(p.semaphore[1] & 8, 0);
        assert_eq!(DRAWN_TICK_RUN1564 & 8 != 0, blips::tests::BLINK_RUN1564);
    }

    /// The phase and the life, at their edges.
    #[test]
    fn the_phase_turns_every_eight_ticks() {
        assert_eq!(phase(0), 1.0);
        assert_eq!(phase(7), 0.125);
        assert_eq!(phase(8), 1.0);
        assert_eq!(phase(-1), 1.0 + 0.125, "C's remainder: -1 % 8 is -1");
        let e = Entry {
            x: 0,
            y: 0,
            ping: -10,
            stamp: 0,
            times: 5,
            doober: 0,
            colour: 0,
        };
        assert!(alive(&e, 79));
        assert!(!alive(&e, 80));
        assert!(alive(&Entry { doober: 1, ..e }, 119));
        assert!(!alive(&Entry { doober: 1, ..e }, 120));
    }

    /// The XML's comment holds a second `attack`; the parse skips it.
    #[test]
    fn the_commented_ping_is_skipped() {
        let xml = r#"<MINIMAP>
          <PING name="attack" rotates="0" rotate_speed="0">
            <PHASE start_pct="1.0" end_pct="0.5" color="0xFFFF0000" usealpha="1" usescale="0" scaleby="0" scalespeed="0" texture="a.tga"/>
          </PING>
          <!-- <PING name="attack" rotates="1" rotate_speed="1"></PING> -->
        </MINIMAP>"#;
        let s = Set::parse(xml).expect("parses");
        assert_eq!(s.pings.len(), 1);
        assert_eq!(
            s.pings[0].phases[0].colour, 0xff00_00ff,
            "red in the low byte"
        );
        assert_eq!(s.paths, ["a.tga"]);
    }
}
