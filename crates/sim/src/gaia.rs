//! Gaia's share of `Objects::process_all@0065dce0` — the tail after every
//! object has run: the birds' sampling every 32 frames and one herd's walk
//! every 64 (`docs/SYNC.md` §3.2). Both are on the sync stream, and on run12
//! they are 22 of frame 0's 120 draws.
//!
//! The animals themselves — the forty idle-anim draws and the wander — are
//! not here yet (`docs/SYNC.md` §6): a herd is the record its animals
//! wander around, kept so its walk draws where the original's does.

use crate::Sim;
use crate::world::Cell;

/// `Herd`: the herd's home (`cx, cy`) and its wander centre (`wx, wy`), in
/// cells, and the animal type it spawns. The dump's `HERDS` block prints
/// exactly these (`docs/ORACLE.md`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Herd {
    pub cx: i32,
    pub cy: i32,
    pub wx: i32,
    pub wy: i32,
    /// `t`: the animal's `TypeIndex`.
    pub kind: i32,
    /// `herd_flags & 1`.
    pub alive: bool,
}

/// Gaia's state on the sim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Gaia {
    pub herds: Vec<Herd>,
    /// Bird spawns the sampling hit and the sim does not model: the frame
    /// and the cell. Each is a divergence from there on (`Guy::init_real`
    /// draws once, then `think_bird` every eight frames).
    pub bird_spawns: Vec<(i64, Cell)>,
    /// Live birds (who 9, type `0x192`) — none until birds exist.
    pub birds: i32,
}

/// The flag a bird's cell must carry — `WData.flags & 0x20`, a mountain
/// cell on the shipped maps (ocean cells carry 0).
pub const BIRD_CELL: u16 = 0x20;
/// The feature bits a herd's wander centre must not carry.
pub const FEATURE_MASK: u16 = 0x70;

impl Sim {
    /// The tail of `Objects::process_all`, after the unit and building
    /// loops.
    pub fn process_gaia(&mut self, frame: i64) {
        if frame & 0x1f == 0 {
            self.sample_birds(frame);
        }
        if frame & 0x3f == 0 {
            self.herd_walk(frame);
        }
    }

    /// Up to ten attempts, two draws each (`% xs`, `% ys`); a `0x20` cell
    /// spawns a bird.
    fn sample_birds(&mut self, frame: i64) {
        let (xs, ys) = (self.world.width(), self.world.height());
        let mut n = (xs * ys / 100).min(10) - self.gaia.birds;
        while n > 0 {
            let x = if xs <= 1 { 0 } else { self.rng.roll() % xs };
            let y = if ys <= 1 { 0 } else { self.rng.roll() % ys };
            let c = Cell { x, y };
            if self.world.cell_data(c).flags & BIRD_CELL != 0 {
                self.gaia.bird_spawns.push((frame, c));
            }
            n -= 1;
        }
    }

    /// `Herd::process` for herd `(frame >> 6) % max(count, 5)`: two draws
    /// move the wander centre one cell in each axis (`−1 + % 3`), kept only
    /// if the cell is on the map, featureless and not blocked.
    fn herd_walk(&mut self, frame: i64) {
        let count = self.gaia.herds.len() as i64;
        let idx = (frame >> 6) % count.max(5);
        if idx >= count || !self.gaia.herds[idx as usize].alive {
            return;
        }
        let h = idx as usize;
        let x = self.gaia.herds[h].wx - 1 + self.rng.roll() % 3;
        let y = self.gaia.herds[h].wy - 1 + self.rng.roll() % 3;
        if x < 0 || y < 0 || x >= self.world.width() || y >= self.world.height() {
            return;
        }
        let d = self.world.cell_data(Cell { x, y });
        if d.flags & FEATURE_MASK == 0 && d.blocked < 8 {
            self.gaia.herds[h].wx = x;
            self.gaia.herds[h].wy = y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Rng;
    use crate::tuning::Tuning;
    use crate::world::World;

    fn sim_at(seed: u32) -> Sim {
        let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
        s.rng = Rng::new(seed);
        s
    }

    fn stepped(seed: u32, n: usize) -> u32 {
        let mut r = Rng::new(seed);
        for _ in 0..n {
            r.roll();
        }
        r.seed
    }

    /// Run12, frame 0, draws 108–109: herd 0 at `(22, 34)` walks to
    /// `(22, 35)` — `% 3 = 1, 2`.
    #[test]
    fn run12_s_herd_walk() {
        let mut s = sim_at(stepped(0x3bd3_9ae9, 108));
        s.gaia.herds.push(Herd {
            cx: 22,
            cy: 34,
            wx: 22,
            wy: 34,
            kind: 408,
            alive: true,
        });
        s.gaia.herds.push(Herd {
            cx: 44,
            cy: 21,
            wx: 44,
            wy: 21,
            kind: 411,
            alive: true,
        });
        s.herd_walk(0);
        assert_eq!((s.gaia.herds[0].wx, s.gaia.herds[0].wy), (22, 35));
        assert_eq!((s.gaia.herds[1].wx, s.gaia.herds[1].wy), (44, 21));
        assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 110));
    }

    /// Run12, frame 0, draws 88–107: ten attempts on a 60×60 map, none on a
    /// `0x20` cell — twenty draws and no bird.
    #[test]
    fn run12_s_bird_sampling_draws_twenty() {
        let mut s = sim_at(stepped(0x3bd3_9ae9, 88));
        s.sample_birds(0);
        assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 108));
        assert!(s.gaia.bird_spawns.is_empty());
    }

    /// The cadences: frame 0 takes both, frames 1–31 neither, 32 the
    /// birds, 64 both again with the next herd; a small map samples fewer.
    #[test]
    fn the_cadences() {
        let mut s = sim_at(7);
        s.gaia.herds.push(Herd {
            cx: 5,
            cy: 5,
            wx: 5,
            wy: 5,
            kind: 1,
            alive: true,
        });
        let seed = s.rng.seed;
        s.process_gaia(0);
        assert_eq!(s.rng.seed, stepped(seed, 22));
        let seed = s.rng.seed;
        for f in 1..32 {
            s.process_gaia(f);
        }
        assert_eq!(s.rng.seed, seed);
        s.process_gaia(32);
        assert_eq!(s.rng.seed, stepped(seed, 20));
        let seed = s.rng.seed;
        s.process_gaia(64);
        // Herd index (64 >> 6) % max(1, 5) = 1: there is no herd 1.
        assert_eq!(s.rng.seed, stepped(seed, 20));
        let mut small = Sim::new(Tuning::RON, World::new(20, 20), 2);
        small.rng = Rng::new(7);
        small.process_gaia(0);
        assert_eq!(small.rng.seed, stepped(7, 8), "20·20/100 = 4 attempts");
    }

    /// A hit records the spawn the sim does not model.
    #[test]
    fn a_mountain_cell_records_a_spawn() {
        let mut s = sim_at(7);
        for x in 0..60 {
            for y in 0..60 {
                let c = Cell { x, y };
                let mut d = s.world.cell_data(c);
                d.flags |= BIRD_CELL;
                s.world.set_cell_data(c, d);
            }
        }
        s.sample_birds(0);
        assert_eq!(s.gaia.bird_spawns.len(), 10);
    }
}
