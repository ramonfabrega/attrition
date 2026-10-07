//! The nuke's blast: `Nuke::add_nuke@0092ba30` at the landing and
//! `Nuke::do_damage@0092bc80` on every frame after it (item 1091,
//! `docs/PRODUCTION.md` "The nuke (item 1091)", `docs/GOLDEN.md` §46).
//!
//! A nuke's round does not strike what it lands on. The nuke arm of
//! `Ammo::do_damage@00678060` (`67846d`..`6785db`) stamps the shooter's
//! player (`nuke_stamp`, `nukes_used`), lays one effect into `nuke_effect`
//! — the landing point, the radius pair `(100, splash_area · 192)`, the
//! frame, the shooter — calls `Nuke::do_damage` at once, and closes the
//! round. `Objects::inc_time@0065db70` calls `Nuke::do_damage` first thing
//! on every frame after, and each call grows a ring over the effect's
//! first forty frames and strikes, once each, every unit and building the
//! ring has reached.
//!
//! **The ring is SSE float32 in the original, and a rational here.**
//! `r = (int)(max · f + (1 − f) · min)`, `f` 0 to `RING_HOLD` and `(t −
//! RING_HOLD) / RING_GROW` after (`divss`, `mulss`, `subss`, `addss`,
//! `cvttss2si` at `92bf3a`..`92bf8d`). With the nuke's own pair (100,
//! 1920) that is `100 + 1820 · (t − 10) / 30`, floored, on every frame
//! the ring lives: the executable's own function under the emulator
//! (`tools/emu/nuke_blast.py`) printed the forty radii, and
//! `the_ring_is_the_emulated_original_s_on_every_frame` pins them. The
//! struck fraction is integers in the original already: `0x100 − (d <<
//! 8) / (int)max`, the float `max` truncated to 1920.

use crate::combat::Obj;
use crate::movement::find_angle;
use crate::world::{Pos, vector_dist};
use crate::{Player, Sim};

/// `Nuke::add_nuke`'s third argument, the ring's first radius (`678500`,
/// `mov [eax], 0x64`).
pub const RING_MIN: i32 = 0x64;
/// The ring's last radius per unit of the type's `splash_area`: `add_nuke`'s
/// fourth argument is `splash_area · 3 << 6` (`6784f0`..`6784fc`), 1920 for
/// the nuke's 10.
pub const RING_PER_SPLASH: i32 = 0xc0;
/// `nuke_effect +0x104`, the frames the ring holds at `RING_MIN`
/// (`Nuke::init@0092c960`).
pub const RING_HOLD: i64 = 10;
/// `nuke_effect +0x108`, the frames it then grows over (`Nuke::init`). The
/// ring lives while `t < RING_HOLD + RING_GROW`.
pub const RING_GROW: i64 = 30;
/// `nuke_effect +0x110`, the effect's own frames (`NukeOut::init@0092c9a0`):
/// the frame the Armageddon counter moves.
pub const NUKE_FRAMES: i64 = 110;
/// The frames past `NUKE_FRAMES` before the effect is dropped (`0x140`,
/// `92bd..`'s first loop).
pub const NUKE_LINGER: i64 = 0x140;

/// One effect of `nuke_effect`'s parallel arrays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blast {
    /// `+0x40`, the landing point.
    pub at: Pos,
    /// `+0x5c`, the radius pair.
    pub min: i32,
    pub max: i32,
    /// `+0x88`, the frame it was laid.
    pub start: i64,
    /// `+0xa4` and `+0xc0`, the shooter's player and object.
    pub who: Player,
    pub shooter: Obj,
    /// `+0xdc` and `+0xf8`, what it has struck (`been_damaged_before`).
    pub struck: Vec<Obj>,
}

/// The nuke's state: the live effects, and what a nuke writes on its
/// shooter's leader and silo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Nukes {
    pub blasts: Vec<Blast>,
    /// `LeaderData +0x7b4 nuke_stamp` (`6784bb`), by player; 0 unset.
    pub stamp: Vec<i64>,
    /// `LeaderData +0x7bc nukes_used` (`6784c8`), by player.
    pub used: Vec<i32>,
    /// The buildings whose `ObjectData +0x40 visible` a nuke's launch set
    /// to `0xff` (`Build::do_missile_launch@00622670`'s nuke arm).
    pub shown: Vec<usize>,
}

impl Nukes {
    /// A player's `nuke_stamp`.
    pub fn stamp_of(&self, who: Player) -> i64 {
        self.stamp.get(usize::from(who)).copied().unwrap_or(0)
    }

    /// A player's `nukes_used`.
    pub fn used_of(&self, who: Player) -> i32 {
        self.used.get(usize::from(who)).copied().unwrap_or(0)
    }

    /// A building's `visible` as the dump prints it: `-1` once a nuke has
    /// left it, else 0 (no other writer is carried here).
    pub fn visible_of(&self, b: usize) -> i64 {
        if self.shown.contains(&b) { -1 } else { 0 }
    }
}

/// The ring's radius on frame `t` of an effect, or `None` once it is spent
/// (`t < 0` or `t ≥ RING_HOLD + RING_GROW`). See the module's note: the
/// original's float32, carried as this floor, which the emulated original
/// agrees with on every frame of the nuke's pair.
pub const fn ring_radius(t: i64, min: i32, max: i32) -> Option<i32> {
    if t < 0 || t >= RING_HOLD + RING_GROW {
        return None;
    }
    if t <= RING_HOLD {
        return Some(min);
    }
    let grown = (max - min) as i64 * (t - RING_HOLD) / RING_GROW;
    Some(min + grown as i32)
}

/// The struck fraction, 8.8: `0x100 − (d << 8) / max`.
pub const fn struck_count(d: i32, max: i32) -> i32 {
    0x100 - (d << 8) / max
}

impl Sim {
    /// Whether a unit is of the nuke's line (`is(0x13b)`).
    pub(crate) fn is_nuke(&self, o: Obj) -> bool {
        matches!(o, Obj::Unit(u) if self.air_line_is(u, crate::airbase::NUCLEARMISSILE))
    }

    /// **`Ammo::do_damage@00678060`'s nuke arm** (`67846d`..`6785db`),
    /// after the shield's read: the shooter's player stamped, the effect
    /// laid (`Nuke::add_nuke@0092ba30`), and its first frame struck at once
    /// (`Nuke::do_damage`, `6785db`). No `hit_target`, no splash walk.
    ///
    /// SEAM: the achievement, the sounds and the message, `World::set_seen2`
    /// over the circle around the landing (the fog, which nothing here
    /// reads), and `TerrainOut::terraform_for_nuke` (the crater's heights,
    /// which no figure reads before the blast has killed it).
    pub(crate) fn nuke_land(&mut self, p: crate::combat::Projectile, frame: i64) {
        let who = usize::from(p.owner);
        let n = &mut self.nukes;
        if n.stamp.len() <= who {
            n.stamp.resize(who + 1, 0);
            n.used.resize(who + 1, 0);
        }
        n.stamp[who] = frame;
        n.used[who] += 1;
        let max = self.profile(p.shooter).splash_area * RING_PER_SPLASH;
        self.nukes.blasts.push(Blast {
            at: p.landing,
            min: RING_MIN,
            max,
            start: frame,
            who: p.owner,
            shooter: p.shooter,
            struck: Vec::new(),
        });
        self.nuke_do_damage(frame);
    }

    /// **`Nuke::do_damage@0092bc80`**, once a frame from the head of
    /// `Objects::inc_time@0065db70` and once from the landing:
    /// - an effect whose frame is past `NUKE_FRAMES + NUKE_LINGER` (or
    ///   before its start) is dropped, the last moved into its place;
    /// - on `t == NUKE_FRAMES` the original moves the Armageddon counter
    ///   (`Game +0x6e0`), which is not carried here: no dump prints it and
    ///   nothing built reads it but the computer's silo strike, whose gate
    ///   reads it as open (`Sim::silo_strike`'s SEAM; the game's end at
    ///   `get_armageddon` is not built);
    /// - while the ring lives ([`ring_radius`]), every unit of the eight
    ///   players (`find_units`' last argument 1: `who < 8`, item 1591)
    ///   that is active, on the map, not a nuke and not struck before by
    ///   this effect, at
    ///   `vector_dist ≤ r` from the point, takes `Object::do_damage` of
    ///   [`struck_count`] when it is above 0 — `num_guys` 1, `ammo` −1,
    ///   `splash` 1, `quiet` 0 (`92c618`..`92c650`) — and every active
    ///   building whose footprint-shrunk distance (`|dx| − x_size · 0x60`,
    ///   floored at 0, each axis) is `< r` takes it when it is not below
    ///   0, with `num_guys` 0 (`92c35f`..`92c3ab`). Each is struck once.
    ///
    /// SEAM: the order of several struck on one frame (the original's is
    /// `Objects::find_units`' — a cell circle or the players' lists, by
    /// `total_units` — and the cell ring's for the buildings; here each
    /// list's index order), a struck player at peace declaring war
    /// (`92c502`, `92c71e`), and the rush rules' early-age exemption.
    pub(crate) fn nuke_do_damage(&mut self, frame: i64) {
        let mut i = 0;
        while i < self.nukes.blasts.len() {
            let start = self.nukes.blasts[i].start;
            if frame < start || frame >= start + NUKE_FRAMES + NUKE_LINGER {
                self.nukes.blasts.swap_remove(i);
            } else {
                i += 1;
            }
        }
        for i in 0..self.nukes.blasts.len() {
            let t = frame - self.nukes.blasts[i].start;
            let (at, min, max, shooter) = {
                let b = &self.nukes.blasts[i];
                (b.at, b.min, b.max, b.shooter)
            };
            let Some(r) = ring_radius(t, min, max) else {
                continue;
            };
            for u in 0..self.units.len() {
                let o = Obj::Unit(u);
                // `Objects::find_units(…, 1)`: the last argument set walks
                // the eight players' units alone (`who < 8` on both its
                // arms, `0065a620`), so the ring never strikes an animal.
                if self.units[u].owner >= 8
                    || !self.active(o)
                    || !self.units[u].on_map
                    || self.nukes.blasts[i].struck.contains(&o)
                    || self.is_nuke(o)
                {
                    continue;
                }
                let pos = self.units[u].pos;
                let d = vector_dist(pos.x - at.x, pos.y - at.y).max(0);
                let count = struck_count(d, max);
                if d > r || count <= 0 {
                    continue;
                }
                let angle = find_angle(pos.x - at.x, pos.y - at.y);
                self.do_damage(shooter, o, angle, false, count, true, false, frame);
                self.nukes.blasts[i].struck.push(o);
            }
            for b in 0..self.buildings.len() {
                let o = Obj::Building(b);
                if !self.active(o) || self.nukes.blasts[i].struck.contains(&o) {
                    continue;
                }
                let tp = self.profile(o);
                let pos = self.buildings[b].pos;
                let dx = ((pos.x - at.x).abs() - tp.x_size * 0x60).max(0);
                let dy = ((pos.y - at.y).abs() - tp.y_size * 0x60).max(0);
                let d = vector_dist(dx, dy);
                let count = struck_count(d, max);
                if d >= r || count < 0 {
                    continue;
                }
                let angle = find_angle(pos.x - at.x, pos.y - at.y);
                self.do_damage(shooter, o, angle, false, count, true, false, frame);
                self.nukes.blasts[i].struck.push(o);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ring is the emulated original's on every frame**
    /// (`tools/emu/nuke_blast.py`: the executable's own `Nuke::do_damage`,
    /// its SSE float32 run under unicorn with the nuke's pair (100, 1920),
    /// the radius read off the `Objects::find_units` it calls). Made to
    /// fail with the grown term rounded up (`(… + 29) / 30`: 161 on t = 11).
    #[test]
    fn the_ring_is_the_emulated_original_s_on_every_frame() {
        #[rustfmt::skip]
        const EMULATED: [i32; 40] = [
            100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 160, 221, 282,
            342, 403, 464, 524, 585, 646, 706, 767, 828, 888, 949, 1010, 1070,
            1131, 1192, 1252, 1313, 1374, 1434, 1495, 1556, 1616, 1677, 1738,
            1798, 1859,
        ];
        for (t, &r) in EMULATED.iter().enumerate() {
            assert_eq!(
                ring_radius(t as i64, RING_MIN, 10 * RING_PER_SPLASH),
                Some(r),
                "t = {t}"
            );
        }
        assert_eq!(ring_radius(40, RING_MIN, 1920), None, "the ring is spent");
        assert_eq!(ring_radius(-1, RING_MIN, 1920), None);
    }

    /// **The ring strikes each unit once, on the first frame it reaches
    /// it, and never past 1859** (`Nuke::do_damage@0092bc80`; chapter
    /// thirty-seven's four probes, `docs/GOLDEN.md` §46): a nuke landing at
    /// (23040, 34560) on 3200 strikes units at 518, 1039 and 1804 on 3217,
    /// 3226 and 3239, a fifth at 1870 never, a Barracks at a building
    /// distance of 1824 on 3239, and stamps its player. Made to fail with
    /// the ring at `max` from its first grown frame (every probe on 3211,
    /// and 1870's struck), with each-once dropped, and with `Sim::land`'s
    /// nuke arm dropped (no stamp, and the splash walk).
    #[test]
    fn the_ring_strikes_each_on_the_frame_it_reaches_it() {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        let nuke = s.add_unit_type(crate::UnitType {
            hits: 2,
            combat: crate::combat::Profile {
                attack: 1200,
                splash_area: 10,
                uber_size: 1,
                domain: crate::attrition::Domain::Air,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        s.unit_types[nuke].tree = Some(crate::airbase::NUCLEARMISSILE);
        let probe = s.add_unit_type(crate::UnitType {
            hits: 130,
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let put = |s: &mut Sim, ty: usize, p: Pos| {
            let index = i16::try_from(s.units.len()).unwrap();
            let hits = s.unit_types[ty].hits;
            let mut u = crate::Unit::new(0, index, p, hits);
            u.ty = Some(ty);
            u.on_map = true;
            u.kind = s.unit_types[ty].kind;
            s.add_unit(u)
        };
        let n = put(&mut s, nuke, Pos::new(19200, 34560));
        s.units[n].on_map = false;
        let probes = [
            put(&mut s, probe, Pos::new(23544, 34680)),
            put(&mut s, probe, Pos::new(23160, 33528)),
            put(&mut s, probe, Pos::new(21240, 34680)),
            put(&mut s, probe, Pos::new(21240, 35064)),
        ];
        let bt = s.add_build_type(crate::build::BuildType {
            x_size: 4,
            y_size: 4,
            ..crate::build::BuildType::default()
        });
        let far = s.add_building(1, Pos::new(25152, 35520), 0);
        s.buildings[far].ty = Some(bt);
        s.buildings[far].hits = 1200;
        s.buildings[far].health = 1200;
        s.buildings[far].active = true;
        s.buildings[far].combat = Some(crate::combat::Profile {
            x_size: 4,
            y_size: 4,
            ..crate::combat::Profile::default()
        });
        let round = crate::combat::Projectile {
            shooter: Obj::Unit(n),
            owner: 0,
            target: None,
            launch: Pos::new(19091, 34561),
            landing: Pos::new(23040, 34560),
            cur_time: 120,
            total_time: 120,
            accuracy: 0,
            angle: crate::movement::Angle(0),
            splash_area: 10,
            num_guys: 1,
            rolling: false,
            missed: false,
            harmless: false,
            air: false,
            sz: 0,
            ez: 0,
            v1z: crate::single::Single::ZERO,
            slot: 0,
        };
        // Through the round's own landing (`Sim::land`'s nuke arm): the
        // round's last frame lays the blast, and strikes nothing where it
        // lands.
        s.projectiles.push(crate::combat::Projectile {
            cur_time: 119,
            ..round
        });
        s.process_projectiles(3200);
        assert!(s.projectiles.is_empty(), "the round is closed");
        assert_eq!((s.nukes.stamp_of(0), s.nukes.used_of(0)), (3200, 1));
        assert_eq!(s.buildings[far].health, 1200, "no splash walk");
        let mut first = [None; 4];
        let mut far_first = None;
        for f in 3200..3260 {
            if f > 3200 {
                s.nuke_do_damage(f);
            }
            for (k, &u) in probes.iter().enumerate() {
                if first[k].is_none() && s.units[u].health < 130 {
                    first[k] = Some(f);
                }
            }
            if far_first.is_none() && s.buildings[far].health < 1200 {
                far_first = Some(f);
            }
        }
        assert_eq!(first, [Some(3217), Some(3226), Some(3239), None]);
        assert_eq!(far_first, Some(3239));
        assert_eq!(s.nukes.blasts[0].struck.len(), 4, "each once");
    }

    /// **The ring never strikes an animal** (item 1591,
    /// `Nuke::do_damage@0092bc80`'s `Objects::find_units(…, 1)`, whose
    /// last argument keeps the walk to `who < 8` on both its arms): a
    /// player's unit and an animal side by side at 518 from ground zero,
    /// and only the unit is struck. run710's chicken `9/6`, beside Napata,
    /// died in this crate on 1735 with a death draw the original never
    /// spent. Made to fail with the owner gate dropped.
    #[test]
    fn the_ring_never_strikes_an_animal() {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        let probe = s.add_unit_type(crate::UnitType {
            hits: 130,
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let put = |s: &mut Sim, who: Player, p: Pos| {
            let index = i16::try_from(s.units.len()).unwrap();
            let mut u = crate::Unit::new(who, index, p, 130);
            u.ty = Some(probe);
            u.on_map = true;
            s.add_unit(u)
        };
        let unit = put(&mut s, 0, Pos::new(23544, 34680));
        let animal = put(&mut s, 9, Pos::new(23544, 34680));
        let shooter = put(&mut s, 1, Pos::new(19200, 34560));
        s.units[shooter].on_map = false;
        s.nukes.blasts.push(Blast {
            at: Pos::new(23040, 34560),
            min: RING_MIN,
            max: 10 * RING_PER_SPLASH,
            start: 3200,
            who: 1,
            shooter: Obj::Unit(shooter),
            struck: Vec::new(),
        });
        for f in 3201..3260 {
            s.nuke_do_damage(f);
        }
        assert!(s.units[unit].health < 130, "the player's unit is struck");
        assert_eq!(s.units[animal].health, 130, "the animal is not");
    }

    /// **The struck fraction is the emulated original's**: 256 at d 0, 243
    /// at 100, 187 at 518, 118 at 1039, 16 at 1804, 9 at 1859
    /// (`tools/emu/nuke_blast.py`, `Object::do_damage`'s `count`).
    #[test]
    fn the_struck_fraction_is_the_emulated_original_s() {
        for (d, c) in [
            (0, 256),
            (100, 243),
            (518, 187),
            (1039, 118),
            (1804, 16),
            (1859, 9),
        ] {
            assert_eq!(struck_count(d, 1920), c, "d = {d}");
        }
    }
}
