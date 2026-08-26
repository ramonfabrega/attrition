//! The farm's crop cells — `Farms::inc_time@008d8600`, `Farms::grow@008d91c0`,
//! `Farms::snip@008d9240`; `docs/SYNC.md` §3.3 and `docs/ORDERS.md` §6.5.
//!
//! Every farm carries sixteen cells, each a state byte and a `float
//! percent`. The farmer's stand adds `0.005f` to the cell it works
//! (`grow`); every frame `inc_time` adds another `0.005f` to every growing
//! cell, takes `0.01f` from every cut one, ripens what reached `1.0f`, and
//! rolls one sync-stream draw per complete farm — two on the two per cent
//! of frames it sprouts an empty cell on its own.
//!
//! The float never enters the simulation. Its whole life is a count of
//! `0.005f` adds from zero, a clamp to exactly `1.0f`, and a count of
//! `0.01f` subtractions from that — three sequences fixed by IEEE single
//! precision and pinned here: the 201st add is the first at or past `1.0f`
//! (`0.005f` is a hair under five thousandths), the 101st subtraction the
//! first at or under zero. [`Farm::adds`] holds the count in adds; the two
//! thresholds are the pins.

use crate::Sim;

/// Adds of `0.005f` from zero that first reach `1.0f`: **201**, not 200.
pub const RIPE_ADDS: i32 = 201;
/// The count a ripe cell is clamped to — the exact `1.0f`, from which the
/// cut cell decays by two adds (`0.01f`) a frame.
pub const FULL: i32 = 200;
/// The state bytes.
pub const EMPTY: u8 = 0;
pub const GROWING: u8 = 1;
pub const RIPE: u8 = 2;
pub const CUT: u8 = 3;

/// `Farms`' per-farm record: sixteen cells, row-major `[row][col]` as the
/// farmer's stand indexes them (`dy * 4 + dx`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Farm {
    pub state: [u8; 16],
    /// The cell's `percent` as a count of `0.005f` adds.
    pub adds: [i32; 16],
    /// `FarmStruct::farm_type` — the `+0xbd` byte `inc_time` skips a farm
    /// on when it is [`ANIMAL_FARM`] (`rise.pdb`'s type record names the
    /// field; the dump prints it after `valid`). The other values are the
    /// crop's art and change nothing here: run20's six farms read
    /// `1, 0, 0, 0, 0, 4`. `docs/SYNC.md` §3.6.
    pub farm_type: u8,
}

/// `FarmType`'s pasture: no crop cells, and five animals of owner 9
/// ([`Sim::farm_add_animals`]).
pub const ANIMAL_FARM: u8 = 1;

/// What `Farms::add_animals@008d8f30` stamps on each of a pasture's five
/// animals: the farm it belongs to (`Animal+0x150` its `o`, `+0x152` its
/// `who` — kept here as the building's own index) and its place in the
/// five (`+0x154`), which is the phase of its `think_farm_animal` tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FarmAnimal {
    pub build: usize,
    pub slot: u8,
}

/// The two types `add_animals` picks between on one draw — even is the
/// chicken, odd the pig.
pub const FARMPIG: crate::tech::TypeId = 0x195;
pub const FARMCHICKEN: crate::tech::TypeId = 0x196;
/// Animals a pasture carries.
pub const FARM_ANIMALS: u8 = 5;
/// `think_farm_animal`'s period.
pub const THINK_PERIOD: i64 = 128;

impl Farm {
    /// `Farms::grow(farm, dx, dy)`: the farmer's add. The cell is growing;
    /// `1.0f < percent` after the add ripens it and clamps.
    pub fn grow(&mut self, cell: usize) {
        self.state[cell] = GROWING;
        self.adds[cell] += 1;
        if self.adds[cell] >= RIPE_ADDS {
            self.state[cell] = RIPE;
            self.adds[cell] = FULL;
        }
    }

    /// `Farms::snip(farm, dx, dy)`: ripe → cut.
    pub fn snip(&mut self, cell: usize) {
        if self.state[cell] == RIPE {
            self.state[cell] = CUT;
        }
    }

    /// The per-frame pass over one farm's cells: the adds, the ripening,
    /// the reset of anything at or under zero. Returns how many cells are
    /// empty afterwards. Column-major, as `inc_time` walks them — the order
    /// the sprout's index counts in.
    fn advance(&mut self) -> i32 {
        let mut empty = 0;
        for col in 0..4 {
            for row in 0..4 {
                let c = row * 4 + col;
                let st = self.state[c];
                if st == GROWING {
                    self.adds[c] += 1;
                }
                if st == CUT {
                    self.adds[c] -= 2;
                }
                // `0 < percent`: a growing sum is positive from its first
                // add; the decaying `1.0f − n·0.01f` is still positive at
                // n = 100 (a hair over zero) and negative at 101.
                let positive = if st == CUT {
                    self.adds[c] >= 0
                } else {
                    self.adds[c] > 0
                };
                if positive {
                    if self.adds[c] >= RIPE_ADDS || st == RIPE {
                        self.state[c] = RIPE;
                        self.adds[c] = FULL;
                    }
                } else {
                    self.state[c] = EMPTY;
                    self.adds[c] = 0;
                    empty += 1;
                }
            }
        }
        empty
    }

    /// The `k`-th empty cell in column-major order.
    fn nth_empty(&self, k: i32) -> Option<usize> {
        let mut n = k;
        for col in 0..4 {
            for row in 0..4 {
                let c = row * 4 + col;
                if self.adds[c] == 0 && self.state[c] == EMPTY {
                    if n == 0 {
                        return Some(c);
                    }
                    n -= 1;
                }
            }
        }
        None
    }
}

impl Sim {
    /// `Farms::add_animals@008d8f30`, from `Farms::add` when the new farm's
    /// `farm_type` is [`ANIMAL_FARM`]: **five animals of owner 9**, each
    /// stamped with the farm and its slot in it. Returns the units.
    ///
    /// The original spends **four draws each** — a coin (`& 1`: even the
    /// chicken, odd the pig), the `y` and then the `x` offset
    /// (`% 0x180 − 0xc0` from the building), and `Guy::init_real`'s variant
    /// roll inside `init_unit` — but every pasture on every capture so far
    /// is a *starting* farm, built inside `Setup::build_empire`, whose
    /// stream the harness does not replay. So this stands the five up the
    /// way the harness stands a dumped unit up: no draws, and the farm's own
    /// position (`docs/SYNC.md` §3.6, "What this leaves open").
    pub fn farm_add_animals(&mut self, b: usize) -> Vec<usize> {
        let ty = self
            .unit_types
            .iter()
            .position(|t| t.tree == Some(FARMCHICKEN) || t.tree == Some(FARMPIG));
        let Some(ty) = ty else {
            return Vec::new();
        };
        let pos = self.buildings[b].pos;
        let hits = self.unit_types[ty].hits.max(1);
        let kind = self.unit_types[ty].kind;
        let mut out = Vec::with_capacity(FARM_ANIMALS as usize);
        for slot in 0..FARM_ANIMALS {
            let Some(index) = self.find_free(9, crate::UNIT_BASE, crate::BUILD_BASE) else {
                break;
            };
            let mut unit = crate::Unit::new(9, index, pos, hits);
            unit.kind = kind;
            unit.ty = Some(ty);
            unit.farm_animal = Some(FarmAnimal { build: b, slot });
            let u = self.add_unit(unit);
            // The guy exists so the idle roll has something to set; its
            // piece is unknown (no dump prints owner 9), so every length
            // lookup fails and the clock never runs out.
            self.units[u].guys = vec![crate::anim::Guy::fresh(-1)];
            out.push(u);
        }
        out
    }

    /// `Animal::think_farm_animal@005d7700`: a pasture's animal, which is
    /// what a herdless animal's `do_idle` falls through to. On its own
    /// 128-frame phase — `(o · (slot + 1) + frame) % 128 == 0` — it takes
    /// **one draw** when the farm covers the tile it is measuring: the
    /// farm's own while nobody gathers there, and `gather_down`'s first
    /// gatherer's once somebody does.
    ///
    /// Where it then walks (`corner_x`/`corner_y[slot]`, thirds of a tile)
    /// is read but not issued — the animals' positions are in no dump, so a
    /// destination here would be fiction. The draw is not.
    pub(crate) fn think_farm_animal(&mut self, u: usize) {
        let Some(fa) = self.units[u].farm_animal else {
            return;
        };
        let o = i64::from(self.units[u].index);
        if (o * i64::from(fa.slot + 1) + self.frame).rem_euclid(THINK_PERIOD) != 0 {
            return;
        }
        let Some(bd) = self.buildings.get(fa.build) else {
            return;
        };
        let at = match bd.gatherers.first() {
            Some(&g) => match self.units.get(g) {
                Some(unit) => unit.pos,
                None => return,
            },
            None => bd.pos,
        };
        if !self.build_covers_tile(fa.build, at.tile()) {
            return;
        }
        let _dir = self.rng.roll() & 7;
    }

    /// `Farms::inc_time`, from `Objects::inc_time` after the ammo — every
    /// complete, enabled farm, in building order.
    pub fn farms_inc_time(&mut self) {
        // The `Farms` list, in activation order (`Sim::farm_order`) — not
        // the buildings' — because the sprout's `% empty` draw is spent by
        // whichever farm the walk reaches first (`docs/SYNC.md` §4.1).
        let order = self.farm_order.clone();
        for b in order {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.farm.farm_type == ANIMAL_FARM {
                continue;
            }
            if self.building_ident(b) != crate::build::Ident::Farm {
                continue;
            }
            let empty = self.buildings[b].farm.advance();
            let chance = if empty >= 12 {
                20
            } else if empty >= 5 {
                (empty - 4) * 20 / 8
            } else {
                continue;
            };
            if chance < 1 {
                continue;
            }
            if self.rng.roll() % 1000 < chance {
                let k = if empty <= 1 {
                    0
                } else {
                    self.rng.roll() % empty
                };
                if let Some(c) = self.buildings[b].farm.nth_empty(k) {
                    self.buildings[b].farm.state[c] = GROWING;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pos;
    use crate::build::{self, Ident};
    use crate::combat::Rng;
    use crate::tuning::Tuning;
    use crate::world::World;

    /// Draws between two words of the stream, walked forward.
    fn draws(from: u32, to: u32) -> usize {
        let mut r = Rng::new(from);
        for n in 0..64 {
            if r.seed == to {
                return n;
            }
            r.roll();
        }
        panic!("{to:#010x} is not within 64 draws of {from:#010x}");
    }

    /// The float sequences, computed in single precision — the oracle the
    /// two pinned counts stand on.
    #[test]
    fn the_single_precision_crossings() {
        let step = 0.005f32;
        let mut x = 0.0f32;
        let mut n = 0;
        while x < 1.0 {
            x += step;
            n += 1;
        }
        assert_eq!(n, RIPE_ADDS, "adds of 0.005f to reach 1.0f");
        assert!(
            x > 1.0,
            "and the 201st overshoots, so `1.0f < x` ripens too"
        );
        let mut y = 1.0f32;
        let mut m = 0;
        while y > 0.0 {
            y -= 0.01f32;
            m += 1;
        }
        assert_eq!(m, 101, "subtractions of 0.01f from 1.0f to reach zero");
        // At m = 100 the float is still (barely) positive — the reason a cut
        // cell with `adds == 0` is not yet empty.
        let mut z = 1.0f32;
        for _ in 0..100 {
            z -= 0.01f32;
        }
        assert!(z > 0.0);
    }

    /// A farmed cell — `grow` and `inc_time` each frame — is ripe after
    /// frame 100's `grow` (the 201st add), which is what puts the
    /// original's re-target on the log's frame 102 (`docs/ORDERS.md` §6.5).
    #[test]
    fn a_farmed_cell_ripens_on_frame_100() {
        let mut f = Farm::default();
        for frame in 0..200 {
            f.grow(5);
            if f.state[5] == RIPE {
                assert_eq!(frame, 100);
                assert_eq!(f.adds[5], FULL);
                break;
            }
            let empty = f.advance();
            assert_eq!(empty, 15);
            assert_eq!(f.state[5], GROWING);
            assert_eq!(f.adds[5], 2 * (frame + 1));
        }
        assert_eq!(f.state[5], RIPE);
        // Cut, then 101 frames of decay to empty.
        f.snip(5);
        assert_eq!(f.state[5], CUT);
        let mut m = 0;
        while f.state[5] == CUT {
            f.advance();
            m += 1;
        }
        assert_eq!(m, 101);
        assert_eq!(f.state[5], EMPTY);
        assert_eq!(f.adds[5], 0);
    }

    /// A sprouted cell nobody farms grows by one add a frame and ripens on
    /// its 201st.
    #[test]
    fn a_sprout_ripens_on_its_201st_add() {
        let mut f = Farm::default();
        f.state[0] = GROWING;
        let mut n = 0;
        while f.state[0] != RIPE {
            f.advance();
            n += 1;
        }
        assert_eq!(n, RIPE_ADDS);
    }

    /// A flat world with one complete farm at tile (5, 5).
    fn farm_sim() -> (Sim, usize) {
        let mut s = Sim::new(Tuning::RON, World::new(16, 16), 2);
        let t = s.add_build_type(build::BuildType {
            ident: Ident::Farm,
            x_size: 3,
            y_size: 3,
            flags: build::flags::FLAT,
            job_time: 100,
            hits: 500,
            ..build::BuildType::default()
        });
        let b = s.init_build(0, t, Pos::new(5 * 192 + 96, 5 * 192 + 96), false);
        s.activate(b, false, true);
        (s, b)
    }

    /// Another complete farm of the same type at a tile.
    fn add_farm(s: &mut Sim, t: usize, tx: i32, ty: i32) {
        let b = s.init_build(0, t, Pos::new(tx * 192 + 96, ty * 192 + 96), false);
        s.activate(b, false, true);
    }

    /// A pasture — `farm_type == 1` — and its five animals of owner 9:
    /// the crop draw `Farms::inc_time` would have spent is not spent, and
    /// the five each roll an idle variant in the unit loop, the first of
    /// them taking `think_farm_animal`'s draw as well because its phase
    /// `o · (slot + 1) + frame` is zero at frame 0. Six draws where a crop
    /// farm spends one — run20's frame 0, draws 138–143 (`docs/SYNC.md`
    /// §3.6).
    #[test]
    fn a_pasture_spends_no_crop_draw_and_six_on_its_animals() {
        let (mut s, b) = farm_sim();
        let chicken = s.add_unit_type(crate::UnitType {
            hits: 10,
            ..crate::UnitType::default()
        });
        s.unit_types[chicken].tree = Some(FARMCHICKEN);
        s.buildings[b].farm.farm_type = ANIMAL_FARM;
        let animals = s.farm_add_animals(b);
        assert_eq!(animals.len(), FARM_ANIMALS as usize, "five animals");
        assert!(
            animals.iter().all(|&u| s.units[u].owner == 9),
            "of owner 9 — the slot no dump prints"
        );
        assert_eq!(
            animals
                .iter()
                .map(|&u| s.units[u].index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4],
            "numbered from the empty leader-9 list"
        );

        // The farm itself: no draw, where a crop farm with sixteen empty
        // cells would take one.
        let seed = s.rng.seed;
        s.farms_inc_time();
        assert_eq!(s.rng.seed, seed, "a pasture grows nothing");
        s.buildings[b].farm.farm_type = 0;
        s.farms_inc_time();
        assert_ne!(s.rng.seed, seed, "and a crop farm does draw");

        // The animals, in the unit loop: one idle roll each, and the first
        // animal's `think_farm_animal` on top.
        s.rng = Rng::new(seed);
        let mut spent = Vec::new();
        for &u in &animals {
            let before = s.rng.seed;
            s.animal_idle(u);
            spent.push(draws(before, s.rng.seed));
        }
        assert_eq!(spent, vec![2, 1, 1, 1, 1], "the six of §3.6");
    }

    /// `think_farm_animal`'s two gates, each made to fail: the 128-frame
    /// phase, and the farm covering the tile it measures — the farm's own
    /// while nobody gathers there, and the first gatherer's once somebody
    /// does.
    #[test]
    fn think_farm_animal_s_phase_and_its_covers_tile() {
        let (mut s, b) = farm_sim();
        let pig = s.add_unit_type(crate::UnitType {
            hits: 10,
            ..crate::UnitType::default()
        });
        s.unit_types[pig].tree = Some(FARMPIG);
        s.buildings[b].farm.farm_type = ANIMAL_FARM;
        let animals = s.farm_add_animals(b);

        // Animal 1 (`o 1`, slot 1) is off phase at frame 0 — `1 · 2 + 0`.
        let seed = s.rng.seed;
        s.think_farm_animal(animals[1]);
        assert_eq!(s.rng.seed, seed, "off phase");
        // …and on it at frame 126, `1 · 2 + 126 == 128`.
        s.frame = 126;
        s.think_farm_animal(animals[1]);
        assert_ne!(s.rng.seed, seed, "on phase");

        // The covers test. With a gatherer registered far away the farm
        // does not cover its tile, and nothing is drawn.
        s.frame = 0;
        let far = crate::Unit::new(0, 99, Pos::new(0, 0), 10);
        let g = s.add_unit(far);
        s.buildings[b].gatherers.push(g);
        let seed = s.rng.seed;
        s.think_farm_animal(animals[0]);
        assert_eq!(s.rng.seed, seed, "the gatherer is off the farm");
        // Standing it on the farm brings the draw back.
        s.units[g].pos = s.buildings[b].pos;
        s.think_farm_animal(animals[0]);
        assert_ne!(s.rng.seed, seed, "and on it, the walk rolls");
    }

    /// Run12, frame 1: the stream after the frame's 48th draw, one farm with
    /// its farmer's cell `(row 2, col 2)` growing and fifteen empty — the
    /// sprout: `% 1000 = 19 < 20`, then `% 15 = 12`, the thirteenth empty
    /// cell in column-major order, `(row 1, col 3)` — the cell the frame-2
    /// dump shows starting to grow (`docs/SYNC.md` §4).
    #[test]
    fn run12_s_frame_1_sprout_lands_on_the_dump_s_cell() {
        let (mut s, b) = farm_sim();
        let mut r = Rng::new(0xb619_4ba1);
        for _ in 0..48 {
            r.roll();
        }
        s.rng = r;
        s.buildings[b].farm.state[2 * 4 + 2] = GROWING;
        s.buildings[b].farm.adds[2 * 4 + 2] = 3;
        s.farms_inc_time();
        let f = &s.buildings[b].farm;
        assert_eq!(f.state[4 + 3], GROWING, "row 1, col 3 sprouted");
        assert_eq!(f.adds[4 + 3], 0);
        assert_eq!(f.state.iter().filter(|&&x| x == GROWING).count(), 2);
        let mut r2 = Rng::new(0xb619_4ba1);
        for _ in 0..50 {
            r2.roll();
        }
        assert_eq!(s.rng.seed, r2.seed, "two draws");
    }

    /// Run12, frame 0: six farms from the frame's 110th draw, no sprout —
    /// six draws, all `% 1000 ≥ 20`.
    #[test]
    fn run12_s_frame_0_farms_draw_six_and_sprout_nothing() {
        let (mut s, b0) = farm_sim();
        let t = s.buildings[b0].ty.unwrap();
        for i in 1..6 {
            add_farm(&mut s, t, 5 + 4 * (i % 3), 5 + 4 * (i / 3));
        }
        let mut r = Rng::new(0x3bd3_9ae9);
        for _ in 0..110 {
            r.roll();
        }
        s.rng = r;
        s.farms_inc_time();
        for _ in 0..6 {
            r.roll();
        }
        assert_eq!(s.rng.seed, r.seed);
        assert!(s.buildings.iter().all(|b| b.farm.state == [EMPTY; 16]));
    }

    /// Run13, sim-frame 101 (`docs/SYNC.md` §4.1): from the frame's 14th
    /// draw off the end-of-100 word `0xa45fecaf`, the six complete farms in
    /// the original's `Farms` order — the AI's three, then the human's —
    /// with the cells the end-of-100 pass shows (print index `p` is the
    /// column-major position, so cell `(p % 4) * 4 + p / 4`; every farmer's
    /// cell 10 is ripe, the rest are sprouts). Farm 1, the AI's `2003` with
    /// twelve empties, sprouts at draw 15 (`% 1000 = 6`) and draw 16 picks
    /// `65297 % 12 = 5`, the sixth empty column-major — the one cell that
    /// appears in the next pass. Seven draws, and the stream lands on the
    /// original's end-of-101 word `0x08670a66`.
    #[test]
    fn run13_s_frame_101_sprout_lands_on_the_ai_s_second_farm() {
        let (mut s, b0) = farm_sim();
        let t = s.buildings[b0].ty.unwrap();
        for i in 1..6 {
            add_farm(&mut s, t, 5 + 4 * (i % 3), 5 + 4 * (i / 3));
        }
        // Print-index → sim cell.
        let cell = |p: usize| (p % 4) * 4 + p / 4;
        let busy: [&[usize]; 6] = [
            &[7, 10, 11],
            &[7, 10, 14, 15],
            &[9, 10, 12, 13],
            &[2, 7, 10],
            &[9, 10, 12],
            &[10, 15],
        ];
        for (f, ps) in busy.iter().enumerate() {
            for &p in ps.iter() {
                let c = cell(p);
                if p == 10 {
                    s.buildings[f].farm.state[c] = RIPE;
                    s.buildings[f].farm.adds[c] = FULL;
                } else {
                    s.buildings[f].farm.state[c] = GROWING;
                    s.buildings[f].farm.adds[c] = 40;
                }
            }
        }
        let before: Vec<[u8; 16]> = s.buildings.iter().map(|b| b.farm.state).collect();
        let mut r = Rng::new(0xa45f_ecaf);
        for _ in 0..14 {
            r.roll();
        }
        s.rng = r;
        s.farms_inc_time();
        assert_eq!(s.rng.seed, 0x0867_0a66, "the original's end-of-101 word");
        for (f, b) in s.buildings.iter().enumerate() {
            let mut want = before[f];
            if f == 1 {
                want[cell(5)] = GROWING;
            }
            assert_eq!(b.farm.state, want, "farm {f}");
        }
    }

    /// A site (not yet active) and a pasture draw nothing; a farm with
    /// four or fewer empty cells draws nothing.
    #[test]
    fn the_gates() {
        let (mut s, b) = farm_sim();
        let seed = s.rng.seed;
        s.buildings[b].active = false;
        s.farms_inc_time();
        assert_eq!(s.rng.seed, seed, "a site");
        s.buildings[b].active = true;
        s.buildings[b].farm.farm_type = ANIMAL_FARM;
        s.farms_inc_time();
        assert_eq!(s.rng.seed, seed, "a pasture");
        s.buildings[b].farm.farm_type = 0;
        for c in 0..12 {
            s.buildings[b].farm.state[c] = GROWING;
            s.buildings[b].farm.adds[c] = 1;
        }
        s.farms_inc_time();
        assert_eq!(s.rng.seed, seed, "four empty cells");
        s.buildings[b].farm.state[11] = CUT;
        s.buildings[b].farm.adds[11] = 0;
        s.farms_inc_time();
        assert_ne!(s.rng.seed, seed, "five: chance (5 − 4)·20/8 = 2");
    }
}
