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
        matches!(o, Obj::Unit(u) if self.air_line_is(u, crate::air::airbase::NUCLEARMISSILE))
    }

    /// **`Ammo::do_damage@00678060`'s nuke arm** (`67846d`..`6785db`),
    /// after the shield's read: the shooter's player stamped, the effect
    /// laid (`Nuke::add_nuke@0092ba30`), and its first frame struck at once
    /// (`Nuke::do_damage`, `6785db`). No `hit_target`, no splash walk.
    ///
    /// SEAM: the achievement, the sounds and the message, and
    /// `TerrainOut::terraform_for_nuke` (the crater's heights,
    /// which no figure reads before the blast has killed it).
    pub(crate) fn nuke_land(&mut self, p: crate::combat::Projectile, frame: i64) {
        let who = usize::from(p.owner);
        let n = &mut self.combat.nukes;
        if n.stamp.len() <= who {
            n.stamp.resize(who + 1, 0);
            n.used.resize(who + 1, 0);
        }
        n.stamp[who] = frame;
        n.used[who] += 1;
        let max = self.profile(p.shooter).splash_area * RING_PER_SPLASH;
        self.combat.nukes.blasts.push(Blast {
            at: p.landing,
            min: RING_MIN,
            max,
            start: frame,
            who: p.owner,
            shooter: p.shooter,
            struck: Vec::new(),
        });
        self.nuke_reveal(p.landing, self.profile(p.shooter).splash_area);
        self.nuke_do_damage(frame);
    }

    /// **The blast lights the ground it lands on for everyone**
    /// (`Ammo::do_damage@00678060:268`–`:287`, after `Nuke::add_nuke` and
    /// before `Nuke::do_damage`). The landing point is taken to its tile
    /// (`div_3_table[x >> 6]`, 192 units), and the first
    /// `circle_radius[splash_area + 1]` points of the `circle_x`/`circle_y`
    /// spiral round it — offsets in tiles, the ring index clamped to
    /// 0..=0x40 — that lie on the map (`0 ≤ t < field_0x18/0x1c`, the tile
    /// counts) each call `World::set_seen2(t >> 1, t >> 1, 0xff, 0)`: all
    /// eight players' bits, in **both** planes, so the destroyed city's
    /// ground is seen to the shooter now and to its neighbours' next
    /// `check_ever_seen` (`docs/VISION.md` §6).
    ///
    /// The original's other two writes, `World +0x168` and the cell
    /// record's `+0x14`, are [`crate::world::World::set_seen`]'s own
    /// (`docs/VISION.md` §12).
    fn nuke_reveal(&mut self, landing: Pos, splash: i32) {
        if !self.world.has_fog() {
            return;
        }
        let c = crate::ai::ai_place::circle();
        let n = c.radius[(splash + 1).clamp(0, 0x40) as usize];
        let (tx, ty) = (landing.tile().x, landing.tile().y);
        let (tw, th) = (
            self.world.width() * crate::world::TILES_PER_CELL,
            self.world.height() * crate::world::TILES_PER_CELL,
        );
        for i in 0..n {
            let (x, y) = (c.x[i] + tx, c.y[i] + ty);
            if x >= 0 && y >= 0 && x < tw && y < th {
                self.world.set_seen(x >> 1, y >> 1, 0xff);
            }
        }
    }

    /// **`Nuke::do_damage@0092bc80`**, once a frame from the head of
    /// `Objects::inc_time@0065db70` and once from the landing:
    /// - an effect whose frame is past `NUKE_FRAMES + NUKE_LINGER` (or
    ///   before its start) is dropped, the last moved into its place;
    /// - on `t == NUKE_FRAMES` the original moves the Armageddon counter
    ///   (`Game +0x6e0`), which is not carried here: no dump prints it and
    ///   nothing built reads it but the computer's silo strike, whose gate
    ///   reads it as open (a gap `Sim::silo_strike` names; the game's end at
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
        while i < self.combat.nukes.blasts.len() {
            let start = self.combat.nukes.blasts[i].start;
            if frame < start || frame >= start + NUKE_FRAMES + NUKE_LINGER {
                self.combat.nukes.blasts.swap_remove(i);
            } else {
                i += 1;
            }
        }
        for i in 0..self.combat.nukes.blasts.len() {
            let t = frame - self.combat.nukes.blasts[i].start;
            let (at, min, max, shooter) = {
                let b = &self.combat.nukes.blasts[i];
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
                    || self.combat.nukes.blasts[i].struck.contains(&o)
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
                self.combat.nukes.blasts[i].struck.push(o);
            }
            for b in 0..self.buildings.len() {
                let o = Obj::Building(b);
                if !self.active(o) || self.combat.nukes.blasts[i].struck.contains(&o) {
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
                self.combat.nukes.blasts[i].struck.push(o);
            }
        }
    }
}

#[cfg(test)]
mod tests;
