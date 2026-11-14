//! **A particle system's look and motion** (item 4233; `docs/RENDERER.md`
//! §3.25): `effects_graphics.xml`'s `<PARTICLESYSTEM>` as the loader holds it,
//! a particle's size and alpha over its life, the scene's wind, and a plume
//! laid without state from an emitter's anchor and the clock.
//!
//! Read from `Particle::draw@008da6b0`, `Particle::pass_time@008db470`,
//! `Particle::init@008db7c0`, `GraphicPieces::emit_build_particles@00909590`
//! and `Scene::render@008b3270` (the wind):
//!
//! - **The look** ([`System::look`]). A particle's life is `fade[2]`
//!   milliseconds. Before `fade[0]` it fades in, its alpha `hold_alpha · t`
//!   and its size from key 0 to key 1; until `fade[1]` it holds
//!   `hold_alpha`, its size from key 1 to key 2; after, it fades out to 0,
//!   its size from key 2 to key 3. The quad's half extent is 1.5 times the
//!   size, along the camera's right and up vectors, turned by the angle.
//! - **The motion** ([`System::step`]). Each step of `dt` ms the
//!   acceleration is gravity plus the wind times `wind_use` (the wind only at
//!   `min_wind_height` over the ground or higher); the position moves by
//!   `(v + a·dt/2)·drag·dt` and the velocity becomes `(v + a·dt)·drag`.
//! - **The start** ([`start`]). The velocity is the anchor's direction
//!   times `start_speed` plus a noise of `⌊|v|·vel_noise⌋ / 340` an axis,
//!   the point the anchor plus a noise of `pos_noise` an axis, each drawn
//!   from the engine's [`Random`] seeded by [`seed`] off the frame, the
//!   anchor and the emitter's birth count. The angle is a random degree
//!   turning at `(r − rotation/2) / 67` a millisecond. Each step takes the
//!   wind of its own frame: the wind turns through tens of degrees within
//!   a smoke particle's life.
//! - **The wind** ([`wind`]). `(4, 4, 3) / 67000`, turned about z by
//!   `−⌊22.5·(sin(t/0.8) + cos(t/1.2))⌋` degrees clamped to ±45, `t` the
//!   game's frame in radians of a degree.
//!
//! **What is the original's and what is not.** The look, the motion, the
//! wind and a particle's start for its birth count are the original's
//! arithmetic, checked on run1559b's and run1570's packets (§3.25). Two
//! things are not. The births: the original's `back_calc` births on the
//! render's cadence and carries an overshoot (`left_over`) between calls,
//! so its births and its count are the capture's, and [`plume`] births a
//! `freq` apart, its count the births since the clock's zero. The angle:
//! the original draws it from `internal_random`, which other systems
//! share, and [`plume`] draws it from a hash.

/// One `<PARTICLESYSTEM>`, in the loader's units: milliseconds, world units
/// a millisecond (squared), and texture coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct System {
    /// `start_width`, `start_hold_width`, `end_hold_width`, `end_width`.
    pub width: [f32; 4],
    pub height: [f32; 4],
    /// The fade-in's end, the hold's end and the life, in milliseconds.
    pub fade: [u32; 3],
    pub gravity: [f32; 3],
    pub start_speed: f32,
    pub drag: f32,
    /// `tu0`, `tu1`, `tv0`, `tv1`.
    pub uv: [f32; 4],
    pub min_wind_height: f32,
    pub wind_use: f32,
    /// A particle every `freq` milliseconds.
    pub freq: u32,
    pub hold_alpha: u8,
    pub rotation: u8,
    pub pos_noise: u8,
    pub vel_noise: u8,
    pub colour: [u8; 3],
    /// `blend="Fire Blend"`: the destination plus the source (`dst_blend`
    /// 2, `D3DBLEND_ONE`), where `Smoke Blend` lays the source over it
    /// (`dst_blend` 6, `INVSRCALPHA`). Read on the packets' render states.
    pub additive: bool,
    /// `MODULATE2X` (`colorop` 4) for `Smoke Blend`, `MODULATE` for `Fire
    /// Blend`, as the packets' render states hold them.
    pub modulate2x: bool,
}

/// A game frame's milliseconds: the loader scales `<PARTICLETIMING>`'s
/// frames by it, and the speeds and accelerations by its powers.
pub const FRAME_MS: f32 = 67.0;

/// One particle: where it is, its angle in degrees, and its life left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub pos: [f32; 3],
    pub angle: f32,
    pub time_left: u32,
}

/// The attributes of `name`'s `<PARTICLESYSTEM>` block in `xml`, each
/// `(tag, attribute, value)`.
fn block<'a>(xml: &'a str, name: &str) -> Option<Vec<(&'a str, &'a str, &'a str)>> {
    let open = format!("<PARTICLESYSTEM name=\"{name}\">");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find("</PARTICLESYSTEM>")?;
    let mut out = Vec::new();
    for tag in xml[start..end].split('<').skip(1) {
        let tag = tag.trim_end_matches(['/', '>', ' ', '\n', '\r', '\t']);
        let Some((tname, mut rest)) = tag.split_once(' ') else {
            continue;
        };
        if tname.starts_with('/') {
            continue;
        }
        while let Some((k, v)) = rest.split_once("=\"") {
            let Some((value, tail)) = v.split_once('"') else {
                break;
            };
            out.push((tname, k.trim(), value));
            rest = tail;
        }
    }
    Some(out)
}

impl System {
    /// **`name`'s system as the loader holds it**, from
    /// `effects_graphics.xml`'s text and the texture's size in texels: the
    /// timings in frames of [`FRAME_MS`], the speed over it, the
    /// accelerations over 100,000 times it, the drag in thousandths, the
    /// wind's use in 255ths. Each scale is checked on the packets'
    /// `ParticleSystem`s (§3.25). The first `<COLOR>` is the system's; the
    /// terrain's variants are not read.
    pub fn from_xml(xml: &str, name: &str, texture: (usize, usize)) -> Option<System> {
        let b = block(xml, name)?;
        let get = |tag: &str, k: &str| -> Option<&str> {
            b.iter().find(|(t, a, _)| *t == tag && *a == k).map(|x| x.2)
        };
        let f = |tag: &str, k: &str| -> Option<f32> { get(tag, k)?.parse::<f32>().ok() };
        let u = |tag: &str, k: &str| -> Option<u32> { get(tag, k)?.parse::<u32>().ok() };
        let size = "PARTICLESIZE";
        let timing = "PARTICLETIMING";
        let render = "PARTICLERENDER";
        let physics = "PARTICLEPHYSICS";
        let byte = |tag: &str, k: &str| -> Option<u8> { u8::try_from(u(tag, k)?).ok() };
        let accel = FRAME_MS * 100_000.0;
        let (tu, tv, tw) = (f(render, "tu")?, f(render, "tv")?, f(render, "twidth")?);
        let (w, h) = (texture.0 as f32, texture.1 as f32);
        let blend = get(render, "blend")?;
        Some(System {
            width: [
                f(size, "start_width")?,
                f(size, "start_hold_width")?,
                f(size, "end_hold_width")?,
                f(size, "end_width")?,
            ],
            height: [
                f(size, "start_height")?,
                f(size, "start_hold_height")?,
                f(size, "end_hold_height")?,
                f(size, "end_height")?,
            ],
            fade: [
                u(timing, "fade_in_time")? * FRAME_MS as u32,
                u(timing, "hold_time")? * FRAME_MS as u32,
                u(timing, "fade_out_time")? * FRAME_MS as u32,
            ],
            gravity: [
                f(physics, "gravity_x")? / accel,
                f(physics, "gravity_y")? / accel,
                f(physics, "gravity_z")? / accel,
            ],
            start_speed: f(physics, "start_speed")? / FRAME_MS / 100.0,
            drag: f(physics, "drag_coef")? / 1000.0,
            uv: [tu / w, (tu + tw) / w, tv / h, (tv + tw) / h],
            min_wind_height: f(physics, "min_wind_height")?,
            wind_use: f(physics, "uses_wind")? / 255.0,
            freq: u(timing, "freq")?,
            hold_alpha: byte(render, "hold_alpha")?,
            rotation: byte(physics, "rotation")?,
            pos_noise: byte(physics, "pos_noise")?,
            vel_noise: byte(physics, "vel_noise")?,
            colour: [
                byte("COLOR", "red")?,
                byte("COLOR", "green")?,
                byte("COLOR", "blue")?,
            ],
            additive: blend == "Fire Blend",
            modulate2x: blend == "Smoke Blend",
        })
    }

    /// The life, in milliseconds.
    pub fn life(&self) -> u32 {
        self.fade[2]
    }

    /// **A particle's half extents and alpha** at `time_left` (`Particle::
    /// draw`): `(half width, half height, alpha)`.
    pub fn look(&self, time_left: u32) -> (f32, f32, u8) {
        let [f0, f1, life] = self.fade;
        let age = life.saturating_sub(time_left);
        let hold = f32::from(self.hold_alpha);
        let (t, a, k) = if age < f0 {
            let t = age as f32 / f0 as f32;
            (t, hold * t, 0)
        } else if time_left < life - f1 {
            // The fade out runs from key 3 at its end up to key 2.
            let t = time_left as f32 / (life - f1) as f32;
            let (w, h) = (
                self.width[3] * (1.0 - t) + self.width[2] * t,
                self.height[3] * (1.0 - t) + self.height[2] * t,
            );
            return (w * 1.5, h * 1.5, (hold * t) as u8);
        } else {
            let t = (age - f0) as f32 / (f1 - f0) as f32;
            (t, hold, 1)
        };
        let w = self.width[k] * (1.0 - t) + self.width[k + 1] * t;
        let h = self.height[k] * (1.0 - t) + self.height[k + 1] * t;
        (w * 1.5, h * 1.5, a as u8)
    }

    /// **One step of `dt` milliseconds** (`Particle::pass_time`) of a
    /// particle at `pos` moving at `vel`, the ground under it at `ground`.
    pub fn step(
        &self,
        pos: &mut [f32; 3],
        vel: &mut [f32; 3],
        dt: f32,
        wind: [f32; 3],
        ground: f32,
    ) {
        let use_wind = if pos[2] < self.min_wind_height + ground {
            0.0
        } else {
            self.wind_use
        };
        let a: [f32; 3] = std::array::from_fn(|k| (self.gravity[k] + wind[k] * use_wind) * dt);
        for k in 0..3 {
            pos[k] += (a[k] * 0.5 + vel[k]) * self.drag * dt;
            vel[k] = (vel[k] + a[k]) * self.drag;
        }
    }
}

/// **The scene's wind** on game frame `frame` (`Scene::render`), in world
/// units a millisecond squared: `(4, 4, 3) / 67000` turned about z by
/// `Transform::rotate_z`'s whole degrees.
/// `graphic_dt`, the milliseconds since the frame began, is taken at 0.
pub fn wind(frame: i64) -> [f32; 3] {
    let k = 1.0 / 67_000.0f32;
    let t = (frame as f32) * std::f32::consts::PI / 180.0;
    let turn = ((t / 0.8).sin() + (t / 1.2).cos()) * 45.0 * 0.5;
    let turn = (turn as i32).clamp(-45, 45);
    let base = [4.0 * k, 4.0 * k, 3.0 * k];
    match crate::camera::rotate(None, turn, [0.0, 0.0, 1.0]) {
        Some(q) => crate::camera::rotate_point(base, q),
        None => base,
    }
}

/// **The engine's `Random`** (`Random::get@00a39d70`): a linear
/// congruence, `seed · 0x19660d + 0x3c6ef35f`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Random(pub u32);

impl Random {
    /// A draw in `lo..hi` (`hi` itself is never drawn): the new seed's low
    /// 16 bits scaled to the span. An empty span returns `lo` and leaves
    /// the seed.
    pub fn get(&mut self, lo: i32, hi: i32) -> i32 {
        if lo == hi {
            return lo;
        }
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        self.0 = self.0.wrapping_mul(0x19_660d).wrapping_add(0x3c6e_f35f);
        (((self.0 & 0xffff) as i64 * (hi - lo) as i64) >> 16) as i32 + lo
    }
}

/// **`Particle::init`'s seed** for the `counter`th birth of an emitter at
/// `at` on game frame `frame` (its listing at `0x008db840` to
/// `0x008db88e`): with `S` the anchor's `(y + x) + z` and `k` its
/// `random · counter · 47`, `cvttss2si(frame · S · k + S) + k`, all in
/// single precision. Past `i32`'s range the conversion gives `i32::MIN`:
/// a building's emitter (`random` 1) at any frame past the first few has
/// the seed `0x80000000 + 47 · counter` for every count past 0.
pub fn seed(frame: i64, at: [f32; 3], random: u8, counter: u32) -> u32 {
    let s = (at[1] + at[0]) + at[2];
    let k = (u32::from(random).wrapping_mul(counter).wrapping_mul(47)) as i32;
    let x = (frame as i32 as f32) * s * (k as f32) + s;
    let t = if x.is_nan() || x >= 2_147_483_648.0 || x < -2_147_483_648.0 {
        i32::MIN
    } else {
        x as i32
    };
    t.wrapping_add(k) as u32
}

/// `emit_build_particles`'s `desc.random`: 1.
pub const BUILD_RANDOM: u8 = 1;

/// **A particle's start** (`Particle::init`) from `seed`: its point and
/// velocity. The velocity is `dir · start_speed`, then three draws of
/// `0..2n` less `n` over 340, `n` the speed times `vel_noise` truncated;
/// the point is the anchor, then three draws of `0..2·pos_noise` less
/// `pos_noise`.
pub fn start(sys: &System, at: [f32; 3], dir: [f32; 3], seed: u32) -> ([f32; 3], [f32; 3]) {
    let mut r = Random(seed);
    let mut vel: [f32; 3] = dir.map(|d| d * sys.start_speed);
    let speed = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
    let n = (speed * f32::from(sys.vel_noise)) as i32;
    let dv: [i32; 3] = std::array::from_fn(|_| r.get(0, 2 * n));
    for k in 0..3 {
        vel[k] += (dv[k] as f32 - n as f32) * (1.0 / 340.0);
    }
    let m = i32::from(sys.pos_noise);
    let dp: [i32; 3] = std::array::from_fn(|_| r.get(0, 2 * m));
    let pos = std::array::from_fn(|k| at[k] + (dp[k] - m) as f32);
    (pos, vel)
}

/// A 32-bit mix of `x` (the finaliser of MurmurHash3).
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    x ^ (x >> 16)
}

/// A draw in `0..n` from `seed`'s `k`th, as [`Random::get`]'s span.
fn draw(seed: u32, k: u32, n: u32) -> u32 {
    mix(seed ^ mix(k.wrapping_add(0x9e37_79b9))) % n.max(1)
}

/// The game frame at `ms` milliseconds (`200/3` a frame), the clock
/// [`wind`] turns by.
pub fn frame_at(ms: u32) -> i64 {
    i64::from(ms) * 3 / 200
}

/// **The plume** of one emitter at `now` milliseconds: every particle born
/// on a multiple of `freq` within the life, the `n`th birth's start drawn
/// by [`start`] from [`seed`] with the count `n`, each moved from its
/// birth in steps of a game frame under that frame's [`wind`], as the
/// original moves each under the wind of the frame it passes. `at` is the
/// anchor, `dir` its direction (`get_position`'s second vector), `key` the
/// emitter's seed for the angle, `ground` the ground's height under a
/// point. The emitter is taken to have burned for a whole life.
pub fn plume(
    sys: &System,
    at: [f32; 3],
    dir: [f32; 3],
    key: u32,
    now: u32,
    ground: &dyn Fn(f32, f32) -> f32,
) -> Vec<Particle> {
    let life = sys.life();
    let freq = sys.freq.max(1);
    let first = now.saturating_sub(life) / freq + 1;
    let mut out = Vec::new();
    for n in first..=now / freq {
        let born = n * freq;
        let age = now - born;
        if age >= life {
            continue;
        }
        let (mut pos, mut vel) = start(sys, at, dir, seed(frame_at(born), at, BUILD_RANDOM, n));
        let key = mix(key ^ mix(n));
        let mut angle = 0.0;
        let mut turn = 0.0;
        if sys.rotation > 0 {
            let r = u32::from(sys.rotation);
            angle = draw(key, 6, 360) as f32;
            turn = (draw(key, 7, r) as f32 - (r / 2) as f32) / FRAME_MS;
        }
        let mut t = born;
        while t < now {
            let dt = (now - t).min(FRAME_MS as u32);
            let g = ground(pos[0], pos[1]);
            sys.step(&mut pos, &mut vel, dt as f32, wind(frame_at(t)), g);
            t += dt;
        }
        out.push(Particle {
            pos,
            angle: angle + turn * age as f32,
            time_left: life - age,
        });
    }
    out
}

/// **A particle's quad**: its four corners and their texture coordinates,
/// `right` and `up` the camera's axes in the world (`xvec`, `yvec`), in the
/// order `Particle::draw` lays them: `−r + u`, `+r + u`, `+r − u`, `−r − u`.
pub fn quad(
    sys: &System,
    p: &Particle,
    right: [f32; 3],
    up: [f32; 3],
) -> [([f32; 3], [f32; 2]); 4] {
    let (hw, hh, _) = sys.look(p.time_left);
    let (s, c) = p.angle.to_radians().sin_cos();
    // The axes turned about the view axis by the angle.
    let r: [f32; 3] = std::array::from_fn(|k| (right[k] * c + up[k] * s) * hw);
    let u: [f32; 3] = std::array::from_fn(|k| (up[k] * c - right[k] * s) * hh);
    let [tu0, tu1, tv0, tv1] = sys.uv;
    let at = |a: f32, b: f32| std::array::from_fn(|k| p.pos[k] + a * r[k] + b * u[k]);
    [
        (at(-1.0, 1.0), [tu0, tv0]),
        (at(1.0, 1.0), [tu1, tv0]),
        (at(1.0, -1.0), [tu1, tv1]),
        (at(-1.0, -1.0), [tu0, tv1]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smoke() -> System {
        System {
            width: [1.0, 40.0, 80.0, 110.0],
            height: [1.0, 40.0, 80.0, 110.0],
            fade: [2479, 5494, 6231],
            gravity: [0.0, 0.0, 300.0 / 6_700_000.0],
            start_speed: 100.0 / 6700.0,
            drag: 1.0,
            uv: [0.0, 0.5, 0.5, 1.0],
            min_wind_height: 100.0,
            wind_use: 115.0 / 255.0,
            freq: 400,
            hold_alpha: 255,
            rotation: 4,
            pos_noise: 50,
            vel_noise: 130,
            colour: [40, 40, 30],
            additive: false,
            modulate2x: true,
        }
    }

    /// The three phases meet: the fade in ends at key 1 and full alpha,
    /// the hold ends at key 2, the fade out ends at key 3 and alpha 0.
    #[test]
    fn the_look_runs_through_its_keys() {
        let s = smoke();
        assert_eq!(s.look(6231), (1.5, 1.5, 0));
        assert_eq!(s.look(6231 - 2479), (60.0, 60.0, 255));
        let (w, _, a) = s.look(6231 - 5494);
        assert!((w - 120.0).abs() < 1e-3 && a == 255, "{w} {a}");
        assert_eq!(s.look(0), (165.0, 165.0, 0));
    }

    /// With no drag, the steps are the closed form: a constant
    /// acceleration's `v·t + a·t²/2`, whatever the step.
    #[test]
    fn with_no_drag_the_steps_are_exact() {
        let s = smoke();
        let w = [1e-5, 2e-5, 0.0];
        let (mut p, mut v) = ([0.0, 0.0, 1000.0], [0.01, 0.0, 0.0]);
        for _ in 0..90 {
            s.step(&mut p, &mut v, 67.0, w, 0.0);
        }
        let t = 90.0 * 67.0f32;
        let a = [w[0] * s.wind_use, w[1] * s.wind_use, s.gravity[2]];
        let want = [
            0.01 * t + a[0] * t * t / 2.0,
            a[1] * t * t / 2.0,
            1000.0 + a[2] * t * t / 2.0,
        ];
        for k in 0..3 {
            assert!(
                (p[k] - want[k]).abs() < 0.05 * want[k].abs().max(1.0),
                "{p:?} {want:?}"
            );
        }
    }

    /// The wind's turn on run1570's frame 3331 is −14, which the packet's
    /// `wind_angle` holds; the vector is `(4, 4, 3) / 67000` turned by 14°.
    #[test]
    fn the_wind_on_run1570_s_frame() {
        let w = wind(3331);
        let packet = [4.221_531e-5, 7.311_911e-5, 4.477_612e-5];
        for k in 0..3 {
            assert!((w[k] - packet[k]).abs() < 2e-7, "{w:?} {packet:?}");
        }
    }

    /// **`Random::get`'s span leaves out its top**, and an empty span draws
    /// nothing; **the seed overflows** to `i32::MIN` past the first count,
    /// so a building's emitter seeds `0x80000000 + 47 · counter` past the
    /// first few frames, whatever its anchor. On run1559b's packet, slot 8's smoke births
    /// 155 and 147 start 9 and −16 units off the anchor on z, as
    /// `client::particle`'s `the_starts_are_the_packets` reads them.
    #[test]
    fn the_seed_is_init_s() {
        let mut r = Random(0);
        let draws: Vec<i32> = (0..2000).map(|_| r.get(0, 2)).collect();
        assert!(draws.iter().all(|&d| d == 0 || d == 1));
        assert!(draws.contains(&0) && draws.contains(&1));
        let before = r;
        assert_eq!(r.get(3, 3), 3);
        assert_eq!(r, before);
        let at = [3207.27, 30518.4, 726.74];
        assert_eq!(seed(3280, at, BUILD_RANDOM, 155), 0x8000_0000 + 47 * 155);
        assert_eq!(seed(9, at, BUILD_RANDOM, 155), 0x8000_0000 + 47 * 155);
        // On frame 3 the product, 7.5e8, is inside `i32` and is the seed.
        assert_eq!(seed(3, at, BUILD_RANDOM, 155), 752_999_157);
        // The first count is the anchor's sum, truncated.
        assert_eq!(seed(3280, at, BUILD_RANDOM, 0), 34452);
        let s = smoke();
        for (counter, dz) in [(155, 9.0), (147, -16.0)] {
            let (p, _) = start(&s, at, [0.0, 0.0, 1.0], seed(3280, at, 1, counter));
            assert_eq!(p[2] - at[2], dz, "{counter}");
        }
    }

    /// The plume is a whole life of births a `freq` apart, each moved for
    /// its age: none older than the life, the newest near the anchor.
    #[test]
    fn a_plume_is_a_life_of_births() {
        let s = smoke();
        let p = plume(
            &s,
            [0.0, 0.0, 700.0],
            [0.0, 0.0, 1.0],
            1,
            200_000,
            &|_, _| 500.0,
        );
        assert!((15..=16).contains(&p.len()), "{}", p.len());
        assert!(p.iter().all(|q| q.time_left > 0 && q.time_left <= 6231));
        let newest = p.iter().max_by_key(|q| q.time_left).unwrap();
        assert!(newest.pos[2] < 800.0, "{newest:?}");
        let oldest = p.iter().min_by_key(|q| q.time_left).unwrap();
        assert!(oldest.pos[2] > 1500.0, "{oldest:?}");
    }
}
