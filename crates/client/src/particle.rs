//! **A burning building's particles** (item 4233; `docs/RENDERER.md`
//! §3.25): each drawn burning slot ([`World::drawn_emitters`]) lays
//! `BUILDING FIRE`'s and `BUILDING SMOKE`'s plumes ([`render::particle::plume`])
//! at the game's clock, and each particle is a quad facing the camera,
//! blended as the systems' render states hold it: the smoke laid over the
//! scene by its alpha, `MODULATE2X` of the texel and its colour, the fire
//! added to it (`client::gpu`'s particle pass, after the figures as
//! draws 216–229 are).

use crate::world::World;
use render::draw::{Drawn, PlacedBuilding};
use render::particle::{Particle, System, plume, quad};
use rondata::artdata::resolve_art;
use std::sync::Arc;

/// `effects_graphics.xml`'s `<PARTICLESYSTEMS>` texture 3, which both
/// systems' `tex_id` name.
const TEXTURE: &str = ".\\art\\BuildingFX.tga";

/// The two systems a burning slot emits (`quick_particle_lookups[0x86]`
/// and `[0x85]`), in the order [`ParticleArt::systems`] holds them.
pub const SYSTEMS: [&str; 2] = ["BUILDING FIRE", "BUILDING SMOKE"];

/// The key the particles' texture takes in the GPU's cache.
pub const PARTICLE_KEY: i32 = -0x302;

/// The systems and their texture, loaded once.
pub struct ParticleArt {
    pub systems: [System; 2],
    pub texture: Arc<render::tga::Image>,
}

/// **One blend's particles**: two triangles a quad, each vertex its world
/// point, texture coordinate and colour (alpha the particle's).
#[derive(Clone, Debug)]
pub struct ParticleMesh {
    pub additive: bool,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
    pub key: i32,
    pub texture: Arc<render::tga::Image>,
}

/// The game's clock on `frame`, in milliseconds (`200/3` a frame).
pub fn game_ms(frame: i64) -> u32 {
    (frame.max(0) * 200 / 3) as u32
}

impl ParticleArt {
    /// The two systems from the install's `effects_graphics.xml`, and
    /// `BuildingFX.tga`.
    pub fn load(root: &std::path::Path) -> Option<ParticleArt> {
        let xml = std::fs::read_to_string(root.join("Data/effects_graphics.xml")).ok()?;
        let texture = resolve_art(root, TEXTURE)
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| render::tga::decode(&b).ok())?;
        let size = (texture.width, texture.height);
        let systems = [
            System::from_xml(&xml, SYSTEMS[0], size)?,
            System::from_xml(&xml, SYSTEMS[1], size)?,
        ];
        Some(ParticleArt {
            systems,
            texture: Arc::new(texture),
        })
    }
}

impl World {
    fn particle_art(&mut self) -> Option<&ParticleArt> {
        if self.particle_art.is_none() {
            self.particle_art = Some(ParticleArt::load(&self.root).map(Arc::new));
        }
        self.particle_art.as_ref()?.as_deref()
    }

    /// The ground's height under `(x, y)`: its tile's, never below zero
    /// (`TerrainOut::find_data_z` as `input::Hit::ground_z` takes it).
    fn ground_under(&self) -> impl Fn(f32, f32) -> f32 + '_ {
        let t = self.runner.terrain();
        move |x: f32, y: f32| {
            let (tile, per) = (render::terrain::UNITS_PER_TILE, 4);
            let (w, h) = (t.width * per, t.height * per);
            let (tx, ty) = ((x as i32).div_euclid(tile), (y as i32).div_euclid(tile));
            if tx < 0 || ty < 0 || tx >= w || ty >= h {
                return 0.0;
            }
            t.tile_z
                .get((ty * w + tx) as usize)
                .copied()
                .unwrap_or(0)
                .max(0) as f32
        }
    }

    /// **Every particle drawn this frame**, each with its system's index
    /// in [`SYSTEMS`]: the plumes of [`Self::drawn_emitters`], or, when a
    /// test set them, [`World::particles_given`].
    pub fn particles(
        &mut self,
        buildings: &[PlacedBuilding],
        list: &[Drawn],
    ) -> Vec<(usize, Particle)> {
        if let Some(given) = &self.particles_given {
            return given.clone();
        }
        let emitters = self.drawn_emitters(buildings, list);
        if emitters.is_empty() || self.particle_art().is_none() {
            return Vec::new();
        }
        let art = self.particle_art.clone().flatten().expect("loaded");
        let frame = self.runner.frame();
        let now = game_ms(frame);
        let ground = self.ground_under();
        let mut out = Vec::new();
        for e in &emitters {
            for (k, sys) in art.systems.iter().enumerate() {
                let key = (u32::from(e.who) << 24)
                    ^ ((e.o as u16 as u32) << 8)
                    ^ (u32::from(e.slot) << 1)
                    ^ k as u32;
                for p in plume(sys, e.pos, e.dir, key, now, &ground) {
                    out.push((k, p));
                }
            }
        }
        out
    }

    /// **The particle pass's meshes** (one a system), each particle a quad
    /// facing `camera`, the far first, as the original sorts them.
    pub fn particle_meshes(
        &mut self,
        buildings: &[PlacedBuilding],
        list: &[Drawn],
        camera: &render::Camera,
    ) -> Vec<ParticleMesh> {
        let mut particles = self.particles(buildings, list);
        if particles.is_empty() || self.particle_art().is_none() {
            return Vec::new();
        }
        let art = self.particle_art.clone().flatten().expect("loaded");
        let r = camera.render_matrix().0;
        let (right, up) = ([r[0], r[4], r[8]], [r[1], r[5], r[9]]);
        let depth = |p: &Particle| r[2] * p.pos[0] + r[6] * p.pos[1] + r[10] * p.pos[2] + r[14];
        particles.sort_by(|a, b| depth(&b.1).total_cmp(&depth(&a.1)));
        let mut out: Vec<ParticleMesh> = art
            .systems
            .iter()
            .map(|s| ParticleMesh {
                additive: s.additive,
                positions: Vec::new(),
                uvs: Vec::new(),
                colours: Vec::new(),
                key: PARTICLE_KEY,
                texture: art.texture.clone(),
            })
            .collect();
        for (k, p) in &particles {
            let sys = &art.systems[*k];
            let (_, _, alpha) = sys.look(p.time_left);
            if alpha == 0 {
                continue;
            }
            let c = sys.colour.map(|v| f32::from(v) / 255.0);
            let colour = [c[0], c[1], c[2], f32::from(alpha) / 255.0];
            let q = quad(sys, p, right, up);
            let m = &mut out[*k];
            for i in [0, 1, 2, 0, 2, 3] {
                m.positions.push(q[i].0);
                m.uvs.push(q[i].1);
                m.colours.push(colour);
            }
        }
        out.retain(|m| !m.positions.is_empty());
        out
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// One packet's dump (`particles.py` in item 4233's lab directory):
    /// its systems' lines and its live particles, each `(system, who, o,
    /// node, particle)`.
    pub(crate) struct Dump {
        pub frame: i64,
        pub wind: [f32; 3],
        pub systems: Vec<(u16, Vec<f32>)>,
        pub particles: Vec<(u16, u8, i16, u8, Particle)>,
    }

    pub(crate) fn dump(name: &str) -> Option<Dump> {
        let home = std::env::var_os("HOME")?;
        let path = std::path::PathBuf::from(home)
            .join("ron-data/lab-experiments/2026-11-14-item-4233")
            .join(format!("{name}-particles.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no particle dump at {}", path.display());
            return None;
        };
        let mut d = Dump {
            frame: 0,
            wind: [0.0; 3],
            systems: Vec::new(),
            particles: Vec::new(),
        };
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            let n = |i: usize| f[i].parse::<f32>().expect("a number");
            match f.first().copied() {
                Some("frame") => d.frame = f[1].parse().expect("a frame"),
                Some("wind") => d.wind = [n(1), n(2), n(3)],
                Some("system") => {
                    let nums = f[2..]
                        .iter()
                        .filter_map(|x| x.parse::<f32>().ok())
                        .collect();
                    d.systems.push((f[1].parse().expect("a system"), nums));
                }
                Some("particle") => d.particles.push((
                    f[1].parse().expect("a system"),
                    f[2].parse().expect("who"),
                    f[3].parse().expect("o"),
                    f[4].parse().expect("node"),
                    Particle {
                        pos: [n(5), n(6), n(7)],
                        angle: n(9) / 2.0,
                        time_left: f[11].parse().expect("time left"),
                    },
                )),
                _ => {}
            }
        }
        Some(d)
    }

    /// **Our reading of `effects_graphics.xml` against the packets'
    /// `ParticleSystem`s**, field for field: the sizes, the fades, gravity,
    /// speed, drag, texture coordinates, wind and hold alpha.
    #[test]
    fn the_systems_are_the_packets() {
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let Some(art) = ParticleArt::load(&root) else {
            println!("skipping: no install at {}", root.display());
            return;
        };
        for name in ["run1559b", "run1570"] {
            let Some(d) = dump(name) else { return };
            for (id, nums) in &d.systems {
                let s = &art.systems[usize::from(*id - 36)];
                // width 4, height 4, fade 3, gravity 3, speed, drag, uv 4,
                // min wind, wind use, freq, emit, hold alpha, rotation,
                // pos noise, vel noise (the colour is hex, skipped).
                let ours: Vec<f32> = s
                    .width
                    .iter()
                    .chain(&s.height)
                    .copied()
                    .chain(s.fade.iter().map(|&x| x as f32))
                    .chain(s.gravity)
                    .chain([s.start_speed, s.drag])
                    .chain(s.uv)
                    .chain([s.min_wind_height, s.wind_use, s.freq as f32])
                    .collect();
                let theirs = &nums[..ours.len()];
                println!("{name} system {id}: ours {ours:?}\n  theirs {theirs:?}");
                for (a, b) in ours.iter().zip(theirs) {
                    assert!((a - b).abs() <= 1e-6 * b.abs().max(1.0), "{id}: {a} {b}");
                }
                let tail = &nums[ours.len() + 1..];
                assert_eq!(
                    [s.hold_alpha, s.rotation, s.pos_noise, s.vel_noise].map(f32::from),
                    [tail[0], tail[1], tail[2], tail[3]],
                    "{id}"
                );
            }
        }
    }

    /// One packet's smoke emitters (`emitters.py` in item 4240's lab
    /// directory): each one's anchor, direction (`desc.vel`), birth count
    /// (`ParticleEmitter +0x50`), and its live particles' `(time_left,
    /// pos)`.
    pub(crate) type Emitters = Vec<([f32; 3], [f32; 3], u32, Vec<(u32, [f32; 3])>)>;

    pub(crate) fn smoke_emitters(name: &str) -> Option<Emitters> {
        let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let path = home
            .join("ron-data/lab-experiments/2026-11-14-item-4240")
            .join(format!("{name}-emitters.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no emitters at {}", path.display());
            return None;
        };
        let mut out: Emitters = Vec::new();
        let mut on = false;
        for line in text.lines() {
            let w: Vec<&str> = line.split_whitespace().collect();
            let after = |key: &str, k: usize| -> &str {
                w[w.iter().position(|x| *x == key).expect("a key") + k]
            };
            let three =
                |key: &str| [1, 2, 3].map(|k| after(key, k).parse::<f32>().expect("a number"));
            match w.first().copied() {
                Some("emitter") => {
                    on = w[2] == "37";
                    if on {
                        let counter = after("counter", 1).parse().expect("a count");
                        out.push((three("pos"), three("vel"), counter, Vec::new()));
                    }
                }
                Some("particle") if on && after("flags", 1) != "0" => {
                    let time_left = after("time_left", 1).parse().expect("time left");
                    out.last_mut()
                        .expect("its emitter")
                        .3
                        .push((time_left, three("pos")));
                }
                _ => {}
            }
        }
        Some(out)
    }

    /// **Each smoke particle's start against the packet's** (item 4240;
    /// `docs/RENDERER.md` §3.25), on run1559b's and run1570's packets. An
    /// emitter's live particles, newest first, are its counts `counter −
    /// 1` down, each born the life less its `time_left` ago. Ours for that
    /// count ([`render::particle::start`] off [`render::particle::seed`]),
    /// moved for its age with the wind always on, stands on the packet's z
    /// within half a unit: the wind turns about z only, so z does not hang
    /// on the capture's cadence. Slot 10's three oldest on run1559b lost
    /// about a second of wind to the ground and stand lower. Made to fail:
    /// the count shifted by one lands none.
    #[test]
    fn the_starts_are_the_packets() {
        use render::particle::{BUILD_RANDOM, seed, start, wind};
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let Some(art) = ParticleArt::load(&root) else {
            println!("skipping: no install at {}", root.display());
            return;
        };
        let smoke = &art.systems[1];
        for (name, frame, want) in [("run1559b", 3294, 69), ("run1570", 3331, 72)] {
            let Some(emitters) = smoke_emitters(name) else {
                return;
            };
            let (mut lands, mut shifted, mut n) = (0, 0, 0);
            for (anchor, dir, counter, mut ps) in emitters {
                ps.sort_by_key(|p| std::cmp::Reverse(p.0));
                for (rank, (time_left, theirs)) in ps.into_iter().enumerate() {
                    let age = smoke.life() - time_left;
                    let z = |c: u32| {
                        let (mut p, mut v) =
                            start(smoke, anchor, dir, seed(frame, anchor, BUILD_RANDOM, c));
                        let mut t = 0;
                        while t < age {
                            let dt = (age - t).min(67);
                            smoke.step(&mut p, &mut v, dt as f32, wind(frame), f32::MIN);
                            t += dt;
                        }
                        p[2]
                    };
                    let c = counter - 1 - rank as u32;
                    n += 1;
                    lands += usize::from((z(c) - theirs[2]).abs() < 0.5);
                    shifted += usize::from((z(c + 1) - theirs[2]).abs() < 0.5);
                }
            }
            println!("{name}: {lands} of {n} on the packet's z, shifted {shifted}");
            assert_eq!((lands, n, shifted), (want, 72, 0), "{name}");
        }
    }

    /// `rise.pdb`'s `RenderEnables` names bit 0x20000 `TL_FAKE_LIGHT`.
    pub(crate) const TL_FAKE_LIGHT: u32 = 0x2_0000;

    /// **The particles take `ps_global.hlsl`'s fake light** (item 4237;
    /// `docs/RENDERER.md` §3.25). On run1567's card trace, both passes,
    /// `D3D11GLSys` compiles the option `TL_FAKE_LIGHT` into a draw's
    /// shader exactly when its `RenderState::enables` carries the bit
    /// [`TL_FAKE_LIGHT`]: 55 draws with it, 366 without, none apart. The
    /// card holds no particle draw, so the systems' own states are read
    /// off the packets (`states.py`, `ParticleSystem +0xbc`): both carry
    /// `enables` 0x20085, so both are drawn through the branch's
    /// `pow(c, 1.9) · 5` into the HDR target.
    #[test]
    fn the_particles_take_the_fake_light() {
        use rondata::trace::card;
        let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let path = std::env::var_os("RON_CARD_TRACE_1567")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| home.join("ron-golden/run1567/map-14/rontrace.log"));
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no card trace at {}", path.display());
            return;
        };
        let draws = card::draws(&bytes);
        let with = draws.iter().filter(|d| d.enables() & TL_FAKE_LIGHT != 0);
        let apart: Vec<u32> = draws
            .iter()
            .filter(|d| (d.enables() & TL_FAKE_LIGHT != 0) != d.has("TL_FAKE_LIGHT"))
            .map(|d| d.index)
            .collect();
        println!(
            "{} draws, {} with the bit, apart {apart:?}",
            draws.len(),
            with.count()
        );
        assert_eq!(draws.len(), 421);
        assert!(apart.is_empty(), "the bit and the option part at {apart:?}");
        for name in ["run1559b", "run1570"] {
            let path = home
                .join("ron-data/lab-experiments/2026-11-14-item-4237")
                .join(format!("{name}-states.txt"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                println!("skipping: no states at {}", path.display());
                return;
            };
            // `state <system> enables <hex> zfunc z src s dst d ...`.
            let states: Vec<(u16, u32, i32)> = text
                .lines()
                .map(|l| {
                    let w: Vec<&str> = l.split_whitespace().collect();
                    let en = u32::from_str_radix(w[3].trim_start_matches("0x"), 16).unwrap();
                    (w[1].parse().unwrap(), en, w[9].parse().unwrap())
                })
                .collect();
            println!("{name}: {states:?}");
            // The fire (36) adds (`dst_blend` 2), the smoke (37) lays over (6).
            assert_eq!(states, [(36, 0x20085, 2), (37, 0x20085, 6)], "{name}");
            assert!(states.iter().all(|s| s.1 & TL_FAKE_LIGHT != 0));
        }
    }
}
