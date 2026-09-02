//! Gaia's share of `Objects::process_all@0065dce0` — the tail after every
//! object has run: the birds' sampling every 32 frames and one herd's walk
//! every 64 (`docs/SYNC.md` §3.2). Both are on the sync stream, and on run12
//! they are 22 of frame 0's 120 draws.
//!
//! The animals themselves — the forty idle-anim draws and the wander — are
//! not here yet (`docs/SYNC.md` §6): a herd is the record its animals
//! wander around, kept so its walk draws where the original's does.

use crate::Sim;
use crate::orders::QueuePos;
use crate::world::Cell;
use crate::world::Pos;

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
    /// Every bird the sampling has hatched: the frame and the cell. Kept
    /// because no dump prints owner 9, so this is the only record a check
    /// can compare against the trace's `Animal::init` births.
    pub bird_spawns: Vec<(i64, Cell)>,
    /// Each live bird's patrol point — the `AirOrder`'s single waypoint,
    /// which `Animal::think_bird` steps and `Unit::do_air_physics` flies
    /// at. Nothing else reads it; it is here so the bird's own state is
    /// state rather than a discarded draw.
    pub bird_goals: Vec<(usize, Pos)>,
    /// Frames on which a bird's landing roll came up and its thirty-round
    /// search ran (`docs/SYNC.md` §3.9). run39's frame 576 is the first
    /// capture to reach one.
    pub bird_landings: Vec<(i64, i16)>,
    /// Each live bird's bank angle and edge turn — the two fields
    /// `Unit::do_air_physics` reads back a frame later
    /// ([`crate::air::Flight`]).
    pub bird_flight: Vec<(usize, crate::air::Flight)>,
}

/// The flag a bird's cell must carry — `WData.flags & 0x20`, a mountain
/// cell on the shipped maps (ocean cells carry 0).
pub const BIRD_CELL: u16 = 0x20;
/// The feature bits a herd's wander centre must not carry.
pub const FEATURE_MASK: u16 = 0x70;
/// `local_10 = 0x1e` — the rounds the landing search takes, a literal.
pub const BIRD_SEARCH_ROUNDS: i32 = 0x1e;

/// The draw sites, under the original's own offsets. [`Sim::mark`]
/// writes them into [`Sim::phase_marks`], so the tail's twenty-two draws
/// are compared against `rondata::trace`'s by name rather than as one
/// `gaia 22` (`docs/SYNC.md` §5).
pub const SITE_BIRD_X: &str = "Objects::process_all+0x2df";
pub const SITE_BIRD_Y: &str = "Objects::process_all+0x30b";
pub const SITE_HERD_X: &str = "Herd::process+0x17";
pub const SITE_HERD_Y: &str = "Herd::process+0x36";
/// `Animal::think_bird@005d79e0` — the patrol point's two offsets and the
/// landing roll, the three draws a live bird spends every eighth frame
/// (`docs/SYNC.md` §3.9).
pub const SITE_BIRD_WANDER_X: &str = "Animal::think_bird+0x82";
pub const SITE_BIRD_WANDER_Y: &str = "Animal::think_bird+0xa6";
pub const SITE_BIRD_LAND: &str = "Animal::think_bird+0x1f8";
/// `Animal::think_bird@005d79e0`'s landing search — thirty rounds over the
/// patrol point's region, two draws a round: the cell it samples and the
/// score it gives it (`docs/SYNC.md` §3.9).
pub const SITE_BIRD_SEARCH_CELL: &str = "Animal::think_bird+0x2aa";
pub const SITE_BIRD_SEARCH_SCORE: &str = "Animal::think_bird+0x2d3";
/// `Animal::do_idle@005d7460` — a herd animal's wander, the four draws
/// that follow its idle roll: the three-in-ten coin at `+0x83`, then, when
/// it comes up and the animal is inside `WANDER_NEAR` of its herd centre,
/// the direction and the two step counts. They are four addresses, so the
/// trace names them without a chain — unlike the idle roll itself, which
/// shares `Guy::set_anim+0x97a` with three other callers
/// (`docs/SYNC.md` §3.10).
pub const SITE_WANDER_ROLL: &str = "Animal::do_idle+0x83";
pub const SITE_WANDER_DIR: &str = "Animal::do_idle+0x1a4";
pub const SITE_WANDER_X: &str = "Animal::do_idle+0x1d4";
pub const SITE_WANDER_Y: &str = "Animal::do_idle+0x212";

/// The owner `Objects::process_all` creates a bird under —
/// `init_unit(objects, 9, BASE_GAIATYPES, …)`. Gaia's animals are owner 8;
/// its birds and a pasture's five are owner 9, and **no dump prints owner
/// 9 at all**, so the draw-site trace is the only oracle a bird has.
pub const BIRD_OWNER: crate::Player = 9;

/// `BASE_GAIATYPES` — the `TypeIndex` of the bird, the first of the twelve
/// gaia types. `think_bird` tests it by identity, and the sampling counts
/// live units of exactly this type.
pub const BIRD_TYPE_INDEX: i32 = 0x192;

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
    ///
    /// **The attempt count is `min(10, xs·ys/100)` less the birds already
    /// flying**, so the sampling shrinks as they accumulate: run14's traces
    /// ten pairs at frames 0, 32, 64 and 96, nine at 128, 160 and 192 (one
    /// bird, born at 96) and eight at 224 and 256 (two, the second born at
    /// 192). Counting the live ones wrongly is worth two draws a sampling
    /// frame, which is why the count is taken from the unit list rather
    /// than kept.
    fn sample_birds(&mut self, frame: i64) {
        let (xs, ys) = (self.world.width(), self.world.height());
        let mut n = (xs * ys / 100).min(10) - self.live_birds();
        while n > 0 {
            let x = if xs <= 1 {
                0
            } else {
                self.mark(SITE_BIRD_X);
                self.rng.roll() % xs
            };
            let y = if ys <= 1 {
                0
            } else {
                self.mark(SITE_BIRD_Y);
                self.rng.roll() % ys
            };
            let c = Cell { x, y };
            if self.world.cell_data(c).flags & BIRD_CELL != 0 {
                self.gaia.bird_spawns.push((frame, c));
                self.spawn_bird(c);
            }
            n -= 1;
        }
    }

    /// The simulation's index for [`BIRD_TYPE_INDEX`], if the data layer
    /// loaded it. A world stood up without the tables has none, and then
    /// the sampling records its spawns and creates nothing.
    pub fn bird_type(&self) -> Option<usize> {
        self.unit_types
            .iter()
            .position(|t| t.type_index == BIRD_TYPE_INDEX)
    }

    /// Live units of exactly the bird type — the original's own loop over
    /// every object testing `flags & 1`, `is_unit` and `type+4 == 0x192`.
    pub fn live_birds(&self) -> i32 {
        let ty = self.bird_type();
        self.units
            .iter()
            .filter(|u| u.alive() && u.ty == ty && ty.is_some())
            .count() as i32
    }

    /// `init_unit(objects, 9, BASE_GAIATYPES, cell centre)` and the
    /// `add_air_patrol_order` that follows it.
    ///
    /// The creation itself is on the stream: `Unit::init` builds the guy and
    /// `Guy::init_real` rolls its variant, which is run14's frame-96 draw 6
    /// — taken *inside* the sampling loop, between the third pair and the
    /// fourth, exactly where the hit fell.
    fn spawn_bird(&mut self, c: Cell) -> Option<usize> {
        let ty = self.bird_type()?;
        let pos = Pos::new(
            c.x * crate::world::UNITS_PER_CELL + crate::world::UNITS_PER_CELL / 2,
            c.y * crate::world::UNITS_PER_CELL + crate::world::UNITS_PER_CELL / 2,
        );
        let index = self
            .find_free(BIRD_OWNER, crate::UNIT_BASE, crate::BUILD_BASE)
            .unwrap_or(i16::MAX);
        let mut unit = crate::Unit::new(BIRD_OWNER, index, pos, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        // `TypeIndex::BIRD`. `set_anim` names it by identity twice — the
        // wing-beat coin and the early return it is exempt from — so a
        // bird that carried the default −1 was a bird whose animation
        // could never resolve (`docs/SYNC.md` §3.9).
        unit.type_index = self.unit_types[ty].type_index;
        // The flight reads both (`crate::air`): `MOVES` is already position
        // units a frame, and `TURN_SPEED` is what scales the bank's rate.
        unit.movement.speed = self.unit_types[ty].moves;
        unit.movement.turning = self.turning_for(ty);
        let at = self.add_unit(unit);
        self.init_guys(at, Some(ty));
        // `add_air_patrol_order` on the hatch point: the patrol point is
        // the **cell centre the bird was created on** and stays a field of
        // the order, not a reading of where the bird now is. That
        // distinction did not exist while the bird stood still, and it is
        // the whole of the first seven frames of a flight
        // (`docs/SYNC.md` §3.9).
        self.gaia.bird_goals.push((at, pos));
        Some(at)
    }

    /// `Animal::think_bird@005d79e0`, the `0x192` arm — what
    /// `Unit::do_air_patrol+0x28` calls on every one of the bird's frames.
    ///
    /// ```text
    /// spell_time < 0 → run;  else spell_time += 1, run only when frame & 7 == 0
    /// mana_burn = 0
    /// the patrol point moves:  spell_time < 1 → ±0x28 on each axis (% 0x51)
    ///                          else            → ±7    on each axis (% 0xf)
    ///     kept when both are on the map and the destination cell's region
    ///     is the point's own
    /// spell_time ≥ 0 → the order's target object is cleared
    /// spell_time > 0: spell_time += 1; when it is not 1, one more draw —
    ///     `% spell_time`, and `== 100` or `> 799` starts the landing
    ///     search ([`Sim::bird_landing_search`], thirty rounds, two draws
    ///     each). The modulus is the counter, so the search cannot fire
    ///     before the bird has been flying a hundred think-cycles.
    /// ```
    ///
    /// The counter takes **one more step than this function gives it**:
    /// `Unit::do_air_patrol`'s tail sets it to 1 when `do_air_physics`
    /// leaves it at 0, which is only ever the frame the landing search
    /// zeroed it ([`Sim::do_idle`]'s bird arm, `orders.rs`).
    ///
    /// **Three draws a think, and only the third's modulus is state** — the
    /// two offsets decide where the bird goes and nothing reads that, which
    /// is why this models the counter exactly and the flight loosely. The
    /// search's sixty are the exception: they are a tenth of run39's frame
    /// 576 and were what parted the second map's word there.
    pub(crate) fn think_bird(&mut self, u: usize, frame: i64) {
        let counter = self.units[u].spell_time;
        if counter >= 0 {
            self.units[u].spell_time = counter.saturating_add(1);
            if frame & 7 != 0 {
                return;
            }
        }
        let counter = self.units[u].spell_time;
        let span = if counter < 1 { 0x51 } else { 0xf };
        let back = if counter < 1 { 0x28 } else { 7 };
        let from = self.bird_goal(u);
        self.mark(SITE_BIRD_WANDER_X);
        let x = from.x + self.rng.roll() % span - back;
        self.mark(SITE_BIRD_WANDER_Y);
        let y = from.y + self.rng.roll() % span - back;
        let to = Pos::new(x, y);
        // The bounds are in position units — `world.xs · 0xc0` on each
        // axis — and the region test is the *point's* own, not the bird's:
        // the step is kept only when it stays inside the region the patrol
        // point already sits in.
        if x >= 0
            && y >= 0
            && x < self.world.width() * crate::world::UNITS_PER_CELL
            && y < self.world.height() * crate::world::UNITS_PER_CELL
            && self.world.region_of(to.cell()) == self.world.region_of(from.cell())
        {
            self.set_bird_goal(u, to);
        }
        if counter <= 0 {
            return;
        }
        let counter = counter.saturating_add(1);
        self.units[u].spell_time = counter;
        if counter == 1 {
            return;
        }
        self.mark(SITE_BIRD_LAND);
        let r = self.rng.roll() % i32::from(counter);
        if r == 100 || r > 799 {
            self.units[u].spell_time = 0;
            self.gaia.bird_landings.push((frame, self.units[u].index));
            self.bird_landing_search(u);
        }
    }

    /// The landing search — `think_bird`'s tail, thirty rounds over the
    /// cell list of the region the patrol point sits in, two draws a round
    /// (`docs/SYNC.md` §3.9).
    ///
    /// **The score it computes is never compared**, and that is the whole
    /// shape of it: `if (-1 < iVar7)` guards the "this one is better"
    /// assignment with a test a `% 0x32 + 1` product can never fail, so the
    /// point lands on the **thirtieth** cell sampled and the terrain
    /// multipliers — `×3` on forest, `×2` on mountain — decide nothing.
    /// Only the sixty draws are observable, and they are what the stream
    /// sees; the score is transliterated anyway, because the next reader of
    /// this function should not have to re-derive that it is inert.
    ///
    /// The region's own size is the first draw's modulus, so a region of a
    /// single cell spends thirty draws rather than sixty.
    fn bird_landing_search(&mut self, u: usize) {
        let from = self.bird_goal(u).cell();
        let reg = self.world.region_of(from).unwrap_or(0);
        let cells: Vec<Cell> = self.world.cells_in(reg).collect();
        let mut best = from;
        for _ in 0..BIRD_SEARCH_ROUNDS {
            let n = cells.len() as i32;
            let i = if n <= 1 {
                0
            } else {
                self.mark(SITE_BIRD_SEARCH_CELL);
                self.rng.roll() % n
            };
            let Some(&c) = cells.get(i as usize) else {
                continue;
            };
            self.mark(SITE_BIRD_SEARCH_SCORE);
            let mut score = self.rng.roll() % 0x32 + 1;
            let f = self.world.cell_data(c).flags;
            if f & crate::world::cell::FOREST != 0 {
                score *= 3;
            }
            if f & crate::world::cell::MOUNTAIN != 0 {
                score *= 2;
            }
            // `-1 < score`, which a score of 1..=300 always is.
            if score > -1 {
                best = c;
            }
        }
        let at = Pos::new(
            best.x * crate::world::UNITS_PER_CELL + crate::world::UNITS_PER_CELL / 2,
            best.y * crate::world::UNITS_PER_CELL + crate::world::UNITS_PER_CELL / 2,
        );
        self.set_bird_goal(u, at);
    }

    /// The patrol point of a bird, which starts on the cell it hatched in.
    pub fn bird_goal(&self, u: usize) -> Pos {
        self.gaia
            .bird_goals
            .iter()
            .find(|(b, _)| *b == u)
            .map_or(self.units[u].pos, |(_, p)| *p)
    }

    fn set_bird_goal(&mut self, u: usize, to: Pos) {
        match self.gaia.bird_goals.iter_mut().find(|(b, _)| *b == u) {
            Some((_, p)) => *p = to,
            None => self.gaia.bird_goals.push((u, to)),
        }
    }

    /// Puts one of gaia's animals back where a dump says it was, with the
    /// walk it was on — the harness's correction, and the twin of
    /// [`Sim::set_guy`].
    ///
    /// An animal wanders when its idle animation runs out and a `% 10` comes
    /// up under 3 (`Animal::do_idle`, `crate::anim`), so where it walks and
    /// when it arrives are decided by draws on the sync stream. Between two
    /// traced frames the harness's stream is **not** the original's — that
    /// is why the word is re-installed at all — so the animals drift, and
    /// their arrivals then fall on the wrong frames. An arrival costs one
    /// draw, in the middle of the unit loop, and that draw sits between one
    /// player's units and the next: run10's sheep is draw 6 of frame 101,
    /// between the AI's three farmers and the human's (`docs/SYNC.md`
    /// §4.1). Re-seating them is what keeps the players' units on the
    /// original's draws through a stretch nothing else can reach.
    ///
    /// Returns how far the animal had drifted, so a harness can report the
    /// size of the correction rather than hide it.
    pub fn reseat_animal(&mut self, u: usize, pos: Pos, goal: Option<Pos>) -> i32 {
        let drift =
            crate::world::vector_dist(self.units[u].pos.x - pos.x, self.units[u].pos.y - pos.y);
        self.set_new_location(u, pos, true);
        self.units[u].movement.body = crate::movement::Body::at(pos);
        self.units[u].movement.dest = None;
        self.units[u].orders.clear();
        if let Some(g) = goal
            && g != pos
        {
            self.add_move_order(u, g, crate::orders::MoveKind::MoveTo, QueuePos::New, false);
        }
        drift
    }

    /// `Herd::process` for herd `(frame >> 6) % max(count, 5)`: two draws
    /// put the wander centre one cell either side of the **home** cell
    /// (`−1 + % 3`), kept only if that cell is on the map, featureless and
    /// not blocked.
    ///
    /// **It reads `cx`/`cy` and writes `wx`/`wy`** — the jitter is around
    /// the home cell every time, not a random walk of the wander centre.
    /// `Herd::process@00741760` loads `field_0x0`/`field_0x4` and stores
    /// `field_0x8`/`field_0xc`, which `HerdData` names `cx, cy, wx, wy`;
    /// this crate read the destination as the source until 2026-08-31 and
    /// the two only agree until the second walk. What it costs is the
    /// **herd centre** `Animal::do_idle` measures from (`docs/ANIM.md` §7):
    /// run33's herd 0 walks on frames 0 and 832 only, and the second walk
    /// left `wy` at 34 here against the original's 33 — a quarter-tile
    /// short of nothing, but the far wander's ring is drawn about that
    /// point and `8/3` was sent to `(17688, 26616)` where the original
    /// sends it to `(17688, 26328)`.
    fn herd_walk(&mut self, frame: i64) {
        let count = self.gaia.herds.len() as i64;
        let idx = (frame >> 6) % count.max(5);
        if idx >= count || !self.gaia.herds[idx as usize].alive {
            return;
        }
        let h = idx as usize;
        self.mark(SITE_HERD_X);
        let x = self.gaia.herds[h].cx - 1 + self.rng.roll() % 3;
        self.mark(SITE_HERD_Y);
        let y = self.gaia.herds[h].cy - 1 + self.rng.roll() % 3;
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

    /// **The second walk is about the home cell, not the first walk's
    /// answer.** `Herd::process@00741760` reads `cx`/`cy` and writes
    /// `wx`/`wy`, so the wander centre is never more than one cell from
    /// home however often the herd is processed. Run12's first walk moves
    /// herd 0 to `(22, 35)`; the same two draws applied a second time
    /// return it to `(22, 35)` rather than carrying it on to `(22, 36)`.
    #[test]
    fn a_second_herd_walk_jitters_about_home_again() {
        let mut s = sim_at(stepped(0x3bd3_9ae9, 108));
        s.gaia.herds.push(Herd {
            cx: 22,
            cy: 34,
            wx: 22,
            wy: 35,
            kind: 408,
            alive: true,
        });
        s.herd_walk(0);
        assert_eq!((s.gaia.herds[0].wx, s.gaia.herds[0].wy), (22, 35));
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

    /// **`AnimalData::get_speed@005d8380`, the animal's own** — the slot
    /// `+0x17c` override that replaces `UnitData::get_speed` whole
    /// (`docs/MOVEMENT.md`, "The animal's own `get_speed`").
    ///
    /// The threshold is on the order's **goal**, not the herd centre and
    /// not the current waypoint, and it is strict: `0x180` exactly is
    /// near. run39's `8/2` crosses it mid-walk, which is what put its
    /// blocked stand a frame late (`docs/SYNC.md` §3.13).
    #[test]
    fn an_animal_beyond_0x180_of_its_order_walks_at_three_halves() {
        use crate::orders::{MoveKind, QueuePos};
        let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
        let ty = s.add_unit_type(crate::UnitType {
            hits: 1,
            moves: 19,
            ..crate::UnitType::default()
        });
        let at = Pos::new(28776, 24360);
        let mut u = crate::Unit::new(8, 2, at, 1);
        u.ty = Some(ty);
        let a = s.add_unit(u);
        s.units[a].movement.speed = 19;
        assert_eq!(s.get_speed(a), 19, "no order at all: the base, floored");

        // 432 from the goal — the distance run39 prints on the frame its
        // step is 28 long.
        let far = Pos::new(28968, 23976);
        assert_eq!(crate::world::vector_dist(192, -384), 432);
        s.add_move_order(a, far, MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(s.get_speed(a), 28, "19 * 3 / 2, truncated");

        // And the step it buys, against the record: `(12, −25)`.
        let want = crate::movement::find_angle(far.x - at.x, far.y - at.y);
        assert_eq!(
            (
                crate::movement::sin_component(want, 28),
                crate::movement::cos_component(want, 28)
            ),
            (12, 25)
        );

        // Exactly `0x180` is near — the test is `> 0x180`, not `>=`.
        s.units[a].orders.clear();
        let near = Pos::new(28776, 24360 - 0x180);
        s.add_move_order(a, near, MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(crate::world::vector_dist(0, -0x180), 0x180);
        assert_eq!(s.get_speed(a), 19, "0x180 is not more than 0x180");

        // A player's unit takes none of it, however far it is going.
        let mut p = crate::Unit::new(0, 0, at, 1);
        p.ty = Some(ty);
        let h = s.add_unit(p);
        s.units[h].movement.speed = 19;
        s.add_move_order(h, far, MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(s.get_speed(h), 19, "`UnitData::get_speed` has no hurry");

        // Nor does a bird: air returns before the hurry *and* before the
        // floor, which is the only way to see the two apart.
        s.units[a].kind.domain = crate::attrition::Domain::Air;
        s.units[a].movement.speed = 1;
        s.units[a].orders.clear();
        s.add_move_order(a, far, MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(s.get_speed(a), 1, "air: neither the 3/2 nor the floor of 3");
        s.units[a].kind.domain = crate::attrition::Domain::Land;
        assert_eq!(s.get_speed(a), 3, "and on the ground, the floor");
    }
}
