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

/// `Farms`' per-farm record: sixteen cells in `FarmStruct`'s own memory
/// order — `uchar[4][4] status` at `+0xac` and `float[4][4] percent` at
/// `+0x8`, both indexed **`[dx][dy]`**, so a cell is `dx * 4 + dy`
/// (`docs/ORDERS.md` §6.5, and the dump prints them in this order).
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
    /// Whether this building has a record at all — `BuildData+0x78 >= 0`,
    /// the slot [`Sim::farms_add`] returns, which `Build::init` writes back
    /// only **after** `Farms::add` returns. So the farm being placed is
    /// invisible to the `count_farms` its own `Farms::add` runs, and that
    /// off-by-one is what decides the type: a city's first farm reaches the
    /// coin because `others` and `crops` are both zero.
    pub valid: bool,
}

/// `FarmType`'s pasture: no crop cells, and five animals of owner 9
/// ([`Sim::farm_add_animals`]).
pub const ANIMAL_FARM: u8 = 1;

/// The bit `Farms::add` ors into `farm_type` when it hands the farm the
/// city's one ambience emitter (`docs/SYNC.md` §3.8). It is the only thing
/// the two draws leave behind, it is what stops a second farm of the same
/// city taking another, and it is what run20's sixth farm reads `4` for.
pub const AMBIENCE: u8 = 4;

/// `FarmsData::get_nearest_farm_type`'s radius: six tiles, in world units.
pub const NEAREST_FARM_RANGE: i32 = 0x480;

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
/// The span each of `add_animals`' two offset draws is folded into —
/// `Random::get(0, 0xffff) % 0x180 − 0xc0`, so a whole tile either way from
/// the farm's centre.
pub const ANIMAL_SPREAD: i32 = 0x180;

/// What one of a pasture's five animals was created with — the three draws
/// `Farms::add_animals@008d8f30` spends on it, minus `Guy::init_real`'s.
///
/// **These are setup draws.** Every pasture any capture has ever held is a
/// *starting* farm, stood up inside `Setup::build_empire`, whose stream the
/// harness does not replay — so the simulation cannot produce them and the
/// five are borrowed from the capture's own trace instead, the way the
/// heights and the herds are borrowed from a sibling dump
/// (`rondata::trace::Trace::add_animals`, `docs/SYNC.md` §3.11). A pasture
/// with no seeds keeps the old stand-in: the farm's own centre, and no
/// `type_index` at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimalSeed {
    /// `(rnd & 1) == 0` — the chicken. A pasture's four-draw stride keeps
    /// every coin on one parity of the stream, so all five agree.
    pub chicken: bool,
    /// `+0x134`, then `+0x182`: the `y` and the `x` offset from the farm.
    pub dy: i32,
    pub dx: i32,
}

/// The draw sites this module spends, under the original's own offsets.
/// [`Sim::mark`] writes them into [`Sim::phase_marks`], so a frame's farm
/// block is compared against `rondata::trace`'s site by site rather than
/// by a count (`docs/SYNC.md` §5). The sprout's second draw is the one a
/// count cannot separate from the chance roll before it.
pub const SITE_CHANCE: &str = "Farms::inc_time+0x1ae";
pub const SITE_SPROUT: &str = "Farms::inc_time+0x1de";
pub const SITE_ANIMAL_DIR: &str = "Animal::think_farm_animal+0x142";
/// `Farms::add`'s pasture coin, and the ambience emitter's two offsets —
/// the `x` first, then the `y` (`docs/SYNC.md` §3.8).
pub const SITE_TYPE_COIN: &str = "Farms::add+0x128";
pub const SITE_AMBIENCE_X: &str = "Farms::add+0x23f";
pub const SITE_AMBIENCE_Y: &str = "Farms::add+0x25b";

impl Farm {
    /// `Farms::grow(farm, dy, dx)@008d91c0`: the farmer's add — the
    /// original takes `dy` then `dx` and writes `[dx][dy]`, so `cell` here
    /// is `dx * 4 + dy`. `1.0f < percent` after the add ripens it and
    /// clamps; the add is unconditional, even on a ripe or cut cell.
    pub fn grow(&mut self, cell: usize) {
        self.state[cell] = GROWING;
        self.adds[cell] += 1;
        if self.adds[cell] >= RIPE_ADDS {
            self.state[cell] = RIPE;
            self.adds[cell] = FULL;
        }
    }

    /// `Farms::snip(farm, dy, dx)@008d9240`: ripe → cut, same index.
    pub fn snip(&mut self, cell: usize) {
        if self.state[cell] == RIPE {
            self.state[cell] = CUT;
        }
    }

    /// The per-frame pass over one farm's cells: the adds, the ripening,
    /// the reset of anything at or under zero. Returns how many cells are
    /// empty afterwards.
    ///
    /// The walk order is `inc_time`'s own — `dy` outer, `dx` inner, so the
    /// index `dx * 4 + dy` runs `0, 4, 8, 12, 1, 5, …` — which is the
    /// order [`Farm::nth_empty`] counts the sprout's `k` in. It makes no
    /// difference here (the count is order-free) and all the difference
    /// there.
    fn advance(&mut self) -> i32 {
        let mut empty = 0;
        for dy in 0..4 {
            for dx in 0..4 {
                let c = dx * 4 + dy;
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

    /// The `k`-th empty cell in [`Farm::advance`]'s order — `dy` outer,
    /// `dx` inner, so `0, 4, 8, 12, 1, 5, …` over the memory index. This
    /// is `inc_time`'s sprout search at `008d87ba`, and it is diffed:
    /// walking the other way parts run12's frame 2 (`docs/SYNC.md` §3.3).
    fn nth_empty(&self, k: i32) -> Option<usize> {
        let mut n = k;
        for dy in 0..4 {
            for dx in 0..4 {
                let c = dx * 4 + dy;
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
    /// `Farms::add@008d8a40`, from `Build::init+0x4ea` — the farm's record
    /// is created the moment the *site* is placed, not when it finishes,
    /// and `Build::init`'s gate is `is(FARM) && !restore`
    /// (`docs/SYNC.md` §3.8).
    ///
    /// Two things happen here and nothing else the simulation can see: the
    /// `farm_type` is chosen, and the city's one ambience emitter is
    /// handed out. Both can draw.
    ///
    /// **The type** (`FarmType`: `-1` none, `0` wheat, `1` pasture). Let
    /// `others` be the city's other farm *buildings*, sites included, and
    /// `crops` the ones among them that already carry a record whose type
    /// is not the pasture:
    ///
    /// - **No city** — a pasture when the object index is odd, a crop when
    ///   it is even, and no ambience either way (`crops` is zero).
    /// - **`others != crops`** — the city already holds a pasture, so this
    ///   one is a crop, and it goes on to the ambience.
    /// - **`others == crops == 4`** — the fifth is a pasture.
    /// - Otherwise the **nearest farm within six tiles**, of any owner,
    ///   decides ([`Sim::nearest_farm_type`]); a pasture there ends it.
    /// - **`others == crops == 3`** — the fourth is a pasture.
    /// - Otherwise **one draw**, and `rand & 3 == 3` makes it a pasture.
    ///
    /// **The ambience.** A crop farm in a city with more than one crop
    /// farm, and no farm of that city already carrying [`AMBIENCE`],
    /// spends **two draws** — an `x` third of a tile and then a `y` one —
    /// on `GraphicEvents::add_ambience`, and takes the bit. The emitter
    /// itself is art; the two draws and the bit are the whole of what
    /// reaches the sync stream, and they are what run20's frame 1 spends
    /// at `+0x23f` and `+0x25b` when the AI's fourth farm is placed.
    ///
    /// The 5x5 corner heights `add` samples afterwards are floats, are the
    /// renderer's, and take no draw.
    pub(crate) fn farms_add(&mut self, b: usize) {
        // The slot. `Farms::add` returns the first record whose `valid` byte
        // is clear and appends when there is none, and that slot is the
        // order `Farms::inc_time` walks the farms in — so it is fixed here,
        // at *placement*, not at activation. A slot freed by `Farms::remove`
        // is refilled by the original and appended to here; nothing on any
        // capture has demolished a farm (`docs/SYNC.md` §3.8).
        if !self.farm_order.contains(&b) {
            self.farm_order.push(b);
        }
        let (o, city) = {
            let bd = &self.buildings[b];
            (i32::from(bd.index), bd.city)
        };
        // `Build::init` line 250: `BuildData+0x78 = Farms::add(...)`, *after*
        // the call. Everything below therefore counts the city's **other**
        // farms, never this one.
        debug_assert!(!self.buildings[b].farm.valid, "a farm gets one record");
        let Some(c) = city else {
            // `+0x72 < 0`: `Build::init` ran `find_city` and it found none.
            // The odd/even coin on the object's own index is the original's,
            // and it is not a draw.
            self.buildings[b].farm.farm_type = u8::from(o & 1 != 0);
            self.buildings[b].farm.valid = true;
            return;
        };
        let others = self.count_buildings(c, crate::build::Ident::Farm, false) - 1;
        let crops = self.city_count_farms(c);
        let farm_type = if others != crops {
            0
        } else if others == 4 {
            ANIMAL_FARM
        } else if let Some(t) = self.nearest_farm_type(b) {
            t
        } else if others == 3 {
            ANIMAL_FARM
        } else {
            self.mark(SITE_TYPE_COIN);
            u8::from(self.rng.roll() & 3 > 2)
        };
        self.buildings[b].farm.farm_type = farm_type;
        self.buildings[b].farm.valid = true;
        if farm_type != 0 {
            return;
        }
        // `1 < count_farms`, and no farm of this city holding the bit
        // already. The walk is the city's building chain and any hit ends
        // it, so the order it is walked in cannot matter.
        if crops <= 1 {
            return;
        }
        let taken = self
            .city_members(c)
            .any(|m| self.is_farm_record(m) && self.buildings[m].farm.farm_type & AMBIENCE != 0);
        if taken {
            return;
        }
        self.mark(SITE_AMBIENCE_X);
        let _x = self.rng.roll() % 3;
        self.mark(SITE_AMBIENCE_Y);
        let _y = self.rng.roll() % 3;
        self.buildings[b].farm.farm_type |= AMBIENCE;
    }

    /// A building `count_farms` and the ambience walk both count: alive, a
    /// farm, and carrying a farm record. The original's third test is
    /// `BuildData+0x78 >= 0` — the slot `Farms::add` returned — which every
    /// farm has from `Build::init` onwards and nothing else ever has.
    fn is_farm_record(&self, b: usize) -> bool {
        self.buildings[b].alive
            && self.building_ident(b) == crate::build::Ident::Farm
            && self.buildings[b].farm.valid
    }

    /// The city's building chain — the head and its members, which is the
    /// order `CityData`'s walks take.
    fn city_members(&self, c: usize) -> impl Iterator<Item = usize> + '_ {
        let city = &self.cities[c];
        std::iter::once(city.building).chain(city.members.iter().copied())
    }

    /// `CityData::count_farms@007368c0`: the city's farms that are **not**
    /// pastures — `farm_type & 1 == 0`, so an ambience-carrying crop (`4`)
    /// counts and the pasture does not. A farm whose slot is out of range
    /// counts too; here every farm has a record, so that arm is dead.
    ///
    /// It is deliberately not `count_buildings`: that one compares the type
    /// index exactly, this one asks the record.
    pub fn city_count_farms(&self, c: usize) -> i32 {
        self.city_members(c)
            .filter(|&m| self.is_farm_record(m) && self.buildings[m].farm.farm_type & 1 == 0)
            .count() as i32
    }

    /// `FarmsData::get_nearest_farm_type@008d73a0`: the nearest farm within
    /// [`NEAREST_FARM_RANGE`] of this one, **of any owner** and not itself,
    /// answers `farm_type & 1`; `None` where there is none.
    ///
    /// The original reaches it through `ObjectsData::find_any_building`,
    /// which walks the cell-circle table and keeps the running minimum with
    /// `<=`, so the *last* candidate at the winning distance wins. This
    /// walks the building list in index order and keeps the first, which
    /// parts from the original only when two farms of different types sit
    /// at exactly the same distance — `docs/SYNC.md` §3.8's open list.
    pub fn nearest_farm_type(&self, b: usize) -> Option<u8> {
        let at = self.buildings[b].pos;
        let mut best: Option<(i32, u8)> = None;
        for (m, bd) in self.buildings.iter().enumerate() {
            if m == b || !self.is_farm_record(m) {
                continue;
            }
            let d = crate::world::vector_dist(bd.pos.x - at.x, bd.pos.y - at.y);
            if d > NEAREST_FARM_RANGE {
                continue;
            }
            if best.is_none_or(|(seen, _)| d < seen) {
                best = Some((d, bd.farm.farm_type & ANIMAL_FARM));
            }
        }
        best.map(|(_, t)| t)
    }

    /// `Farms::add_animals@008d8f30`, from `Build::activate@00623e20` (line
    /// 1209) once the farm's `farm_type` is [`ANIMAL_FARM`] and its building
    /// is complete: **five animals of owner 9**, each stamped with the farm
    /// and its slot in it. Returns the units.
    ///
    /// The original spends **four draws each** — a coin (`& 1`: even the
    /// chicken, odd the pig), the `y` and then the `x` offset
    /// ([`ANIMAL_SPREAD`] from the building), and `Guy::init_real`'s variant
    /// roll inside `init_unit` — but every pasture on every capture so far
    /// is a *starting* farm, built inside `Setup::build_empire`, whose
    /// stream the harness does not replay. So none of the four is taken
    /// here; the three that leave a mark are **borrowed** from the
    /// capture's own trace as `seeds` ([`AnimalSeed`]).
    ///
    /// With seeds each animal gets its species — and with it a
    /// `type_index`, which is what lets [`crate::Sim::slot_length`] reach
    /// the install's gaia table so the idle clock wraps at all — and its
    /// own place on the ground. With none, the old stand-in: one type
    /// either way, no `type_index`, and all five on the farm's centre
    /// (`docs/SYNC.md` §3.11).
    pub fn farm_add_animals(&mut self, b: usize, seeds: &[AnimalSeed]) -> Vec<usize> {
        let chicken = self
            .unit_types
            .iter()
            .position(|t| t.tree == Some(FARMCHICKEN));
        let pig = self.unit_types.iter().position(|t| t.tree == Some(FARMPIG));
        let Some(any) = chicken.or(pig) else {
            return Vec::new();
        };
        let pos = self.buildings[b].pos;
        let mut out = Vec::with_capacity(FARM_ANIMALS as usize);
        for slot in 0..FARM_ANIMALS {
            let seed = seeds.get(slot as usize).copied();
            let ty = match seed {
                Some(s) if s.chicken => chicken.unwrap_or(any),
                Some(_) => pig.unwrap_or(any),
                None => any,
            };
            let at = match seed {
                Some(s) => crate::Pos::new(pos.x + s.dx, pos.y + s.dy),
                None => pos,
            };
            let Some(index) = self.find_free(9, crate::UNIT_BASE, crate::BUILD_BASE) else {
                break;
            };
            let hits = self.unit_types[ty].hits.max(1);
            let mut unit = crate::Unit::new(9, index, at, hits);
            unit.kind = self.unit_types[ty].kind;
            unit.ty = Some(ty);
            // `Objects::init_unit`'s own two, without which the `MOVE_TO`
            // `think_farm_animal` hands out is an order the animal can
            // never step: it stands still and its arrival — two
            // `Animal::do_idle` draws — never comes.
            unit.movement.speed = self.unit_types[ty].moves;
            unit.movement.turning = self.turning_for(ty);
            // `TypeIndex` `FARMPIG`/`FARMCHICKEN`. Without it the gaia
            // table cannot be keyed and every animation length reads
            // `anim::UNKNOWN`, so the clock steps and never wraps — three
            // of run39's six missing `Guy::inc_time` wraps on frame 29.
            // A pasture the trace does not name keeps the −1, because
            // which of the two species it is, is exactly what the coin
            // said and nothing else records.
            if seed.is_some() {
                unit.type_index = self.unit_types[ty].type_index;
            }
            unit.farm_animal = Some(FarmAnimal { build: b, slot });
            let u = self.add_unit(unit);
            // The guy exists so the idle roll has something to set. Its
            // piece stays −1 — no dump prints an owner-9 object, so the
            // piece pool cannot name one — and the gaia table is keyed by
            // the type instead.
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
    /// Then it walks, and the one draw is the whole of where to
    /// (`docs/SYNC.md` §3.11). Let `T` be the reference object's tile plus
    /// [`crate::world::MOVE_8`]`[dir]`, `C` the slot's
    /// [`crate::world::CORNER_X`]: the original scales the pair to
    /// `(3T + C) · 0x40 + 0x60`, which is `192·T + 64·C + 96` — the
    /// unsnapped point — takes the **angle** to that, and only then snaps
    /// it with `div_3_table[v >> 4] · 0x30 + 0x18`, the same 48-unit snap
    /// every `add_move_order` applies. So the destination is
    /// `add_move_facing_order`'s own of that point, and the facing is the
    /// bearing to it *before* the snap, which is why this cannot go
    /// through [`crate::Sim::add_move_order`].
    ///
    /// The order is `MOVE_TO` on an emptied list, appended after
    /// `close_orders`, `clear_partial_path` and `update_action` — the
    /// listing's own sequence at `5d78c5`–`5d79d4`.
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
        let t = at.tile();
        if !self.build_covers_tile(fa.build, t) {
            return;
        }
        self.mark(SITE_ANIMAL_DIR);
        let dir = (self.rng.roll() & 7) as usize;
        let (mx, my) = crate::world::MOVE_8[dir];
        let c = usize::from(fa.slot).min(crate::world::CORNER_X.len() - 1);
        let raw = crate::Pos::new(
            ((t.x + mx) * 3 + crate::world::CORNER_X[c]) * 0x40 + 0x60,
            ((t.y + my) * 3 + crate::world::CORNER_Y[c]) * 0x40 + 0x60,
        );
        let here = self.units[u].pos;
        let angle = crate::movement::find_angle(raw.x - here.x, raw.y - here.y);
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
        self.add_move_facing_order(
            u,
            raw,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::Last,
            false,
            angle,
            None,
            false,
        );
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
            self.mark(SITE_CHANCE);
            if self.rng.roll() % 1000 < chance {
                let k = if empty <= 1 {
                    0
                } else {
                    self.mark(SITE_SPROUT);
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
        s.buildings[b].farm.farm_type = 0;
        (s, b)
    }

    /// Another complete farm of the same type at a tile.
    ///
    /// There is no city on this world, so [`Sim::farms_add`] falls to the
    /// cityless coin — a pasture on an odd object index — and the farms the
    /// capture pins are the original's, which are crops in a city. The type
    /// is set back here rather than founding a city, so these pins keep
    /// measuring `Farms::inc_time` and nothing else.
    fn add_farm(s: &mut Sim, t: usize, tx: i32, ty: i32) {
        let b = s.init_build(0, t, Pos::new(tx * 192 + 96, ty * 192 + 96), false);
        s.activate(b, false, true);
        s.buildings[b].farm.farm_type = 0;
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
        let animals = s.farm_add_animals(b, &[]);
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
        let animals = s.farm_add_animals(b, &[]);

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

    /// **Where the one draw sends it.** `docs/SYNC.md` §3.11: the tile of
    /// the reference object plus `move_x/move_y[dir + 1]`, then the slot's
    /// own corner of that tile — `192·T + 24`, `+ 120` or `+ 168` for a
    /// corner of −1, 0 or +1, which is the low edge, the middle and the
    /// high side of a 192-unit tile.
    ///
    /// The corner is what this is made to fail on: five animals of one
    /// pasture, all measuring the same farm and all rolling the same
    /// direction, land on **three** different points, and which one is
    /// [`crate::world::CORNER_X`]'s row for the slot.
    #[test]
    fn the_walk_lands_on_the_slot_s_corner_of_the_tile_it_picks() {
        let (mut s, b) = farm_sim();
        let chicken = s.add_unit_type(crate::UnitType {
            hits: 10,
            moves: 25,
            ..crate::UnitType::default()
        });
        s.unit_types[chicken].tree = Some(FARMCHICKEN);
        s.buildings[b].farm.farm_type = ANIMAL_FARM;
        let seeds = [AnimalSeed {
            chicken: true,
            dy: 0,
            dx: 0,
        }; FARM_ANIMALS as usize];
        let animals = s.farm_add_animals(b, &seeds);
        let t = s.buildings[b].pos.tile();

        for (slot, &u) in animals.iter().enumerate() {
            // Each animal's phase — `o · (slot + 1) + frame ≡ 0` — with
            // `o == slot` here, so the five fire on `128 − slot(slot + 1)`:
            // 128, 126, 122, 116, 108, which is §3.11's own set.
            s.frame = THINK_PERIOD - (slot * (slot + 1)) as i64;
            // The direction the draw is about to return, taken off a copy
            // of the generator so the assertion names a point rather than
            // a range.
            let dir = (crate::combat::Rng::new(s.rng.seed).roll() & 7) as usize;
            let (mx, my) = crate::world::MOVE_8[dir];
            s.think_farm_animal(u);
            let Some(crate::orders::Body::Move(mo)) = s.units[u].orders.front().map(|o| o.body)
            else {
                panic!("animal {slot} took no MOVE_TO");
            };
            let edge = |c: i32| match c {
                -1 => 24,
                0 => 120,
                _ => 168,
            };
            assert_eq!(
                (mo.dest.x, mo.dest.y),
                (
                    192 * (t.x + mx) + edge(crate::world::CORNER_X[slot]),
                    192 * (t.y + my) + edge(crate::world::CORNER_Y[slot]),
                ),
                "animal {slot}'s corner of tile ({}, {})",
                t.x + mx,
                t.y + my
            );
        }
        // The centre and the four corners really are three distinct
        // points on each axis, so the table is doing work.
        let xs: std::collections::BTreeSet<i32> = animals
            .iter()
            .map(|&u| match s.units[u].orders.front().map(|o| o.body) {
                Some(crate::orders::Body::Move(m)) => m.dest.x % 192,
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            xs,
            [24, 120, 168].into_iter().collect(),
            "the low edge, the middle and the high side"
        );
    }

    /// Run12, frame 1: the stream after the frame's 48th draw, one farm
    /// with its farmer's cell `(dx 2, dy 2)` growing and fifteen empty —
    /// the sprout: `% 1000 = 19 < 20`, then `% 15 = 12`, the thirteenth
    /// empty cell in [`Farm::nth_empty`]'s order, **memory index 7** —
    /// the cell the frame-2 dump shows starting to grow (`docs/SYNC.md`
    /// §4).
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
        assert_eq!(f.state[7], GROWING, "(dx 1, dy 3) sprouted");
        assert_eq!(f.adds[7], 0);
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

    /// A city on the flat world, so [`Sim::farms_add`] has one to count.
    fn city_sim() -> (Sim, usize, usize) {
        let (mut s, first) = farm_sim();
        let t = s.buildings[first].ty.unwrap();
        let village = s.add_build_type(build::BuildType {
            ident: Ident::Village,
            x_size: 7,
            y_size: 7,
            job_time: 100,
            hits: 1000,
            ..build::BuildType::default()
        });
        // The world is 16 cells square; the farm of `farm_sim` sits at tile
        // (5, 5) and would join this city, which is not what the table below
        // counts, so it goes.
        s.buildings[first].alive = false;
        s.farm_order.clear();
        let c = s.init_build(0, village, Pos::new(20 * 192 + 96, 20 * 192 + 96), false);
        s.activate(c, false, true);
        let city = s.buildings[c].city.expect("a finished city has a record");
        (s, t, city)
    }

    /// `Farms::add`'s branch table and its two draw sites (`docs/SYNC.md`
    /// §3.8) — the two draws run20's frame 1 spends and the sim did not.
    ///
    /// Five placements, each naming what it costs:
    ///
    /// 1. **No city** — the coin is the object index's own parity, and it
    ///    takes no draw either way.
    /// 2. **The city's first farm** — nothing to copy and nothing near, so
    ///    `Farms::add` rolls: one draw at `+0x128`.
    /// 3. **The second, beside the first** — `get_nearest_farm_type` answers
    ///    and there is no roll; one crop is not "more than one", so no
    ///    ambience. **No draw at all.**
    /// 4. **The third** — the nearest still answers, and now the city has
    ///    two crops: the emitter's **two draws**, and the farm keeps
    ///    [`AMBIENCE`].
    /// 5. **The fourth** — the city's emitter is taken, so the walk ends on
    ///    the third farm and nothing is spent.
    #[test]
    fn farms_add_picks_the_type_and_hands_out_one_ambience_a_city() {
        let (mut s, t, city) = city_sim();

        // 1. Cityless: far outside the city radius, so `find_city` finds
        //    none and the parity of `o` decides. Neither draws.
        let before = s.rng.seed;
        let lone = s.init_build(0, t, Pos::new(2 * 192 + 96, 2 * 192 + 96), false);
        let lone2 = s.init_build(0, t, Pos::new(2 * 192 + 96, 6 * 192 + 96), false);
        assert_eq!(s.buildings[lone].city, None, "outside every city radius");
        assert_eq!(
            s.buildings[lone].farm.farm_type,
            u8::from(s.buildings[lone].index & 1 != 0)
        );
        assert_eq!(
            s.buildings[lone2].farm.farm_type,
            u8::from(s.buildings[lone2].index & 1 != 0)
        );
        assert_ne!(
            s.buildings[lone].farm.farm_type, s.buildings[lone2].farm.farm_type,
            "consecutive object numbers, so one of the two is the pasture"
        );
        assert_eq!(s.rng.seed, before, "the cityless coin is not a draw");
        s.buildings[lone].alive = false;
        s.buildings[lone2].alive = false;

        // 2. The city's first farm: the roll.
        s.trace_phases = true;
        let f1 = s.init_build(0, t, Pos::new(18 * 192 + 96, 20 * 192 + 96), false);
        assert_eq!(s.buildings[f1].city, Some(city), "inside the city");
        assert_eq!(
            s.phase_marks
                .iter()
                .map(|m| m.0.as_str())
                .collect::<Vec<_>>(),
            vec![SITE_TYPE_COIN],
            "the first farm of a city rolls for its type"
        );
        // The seed is the harness's own, so which side the coin lands on is
        // not the assertion; that it is a legal `FarmType` is.
        assert!(s.buildings[f1].farm.farm_type <= ANIMAL_FARM);
        s.buildings[f1].farm.farm_type = 0;

        // 3. The second, four tiles off: copied from the first, no draw.
        s.phase_marks.clear();
        let f2 = s.init_build(0, t, Pos::new(22 * 192 + 96, 20 * 192 + 96), false);
        assert_eq!(s.buildings[f2].city, Some(city));
        assert_eq!(s.buildings[f2].farm.farm_type, 0, "the neighbour's type");
        assert!(s.phase_marks.is_empty(), "one crop is not more than one");

        // 4. The third: the ambience pair, and the bit.
        s.phase_marks.clear();
        let f3 = s.init_build(0, t, Pos::new(20 * 192 + 96, 22 * 192 + 96), false);
        assert_eq!(
            s.phase_marks
                .iter()
                .map(|m| m.0.as_str())
                .collect::<Vec<_>>(),
            vec![SITE_AMBIENCE_X, SITE_AMBIENCE_Y],
            "x then y, and no coin"
        );
        assert_eq!(s.buildings[f3].farm.farm_type, AMBIENCE);
        assert_eq!(
            s.city_count_farms(city),
            3,
            "an emitter's crop still counts"
        );

        // 5. The fourth: the emitter is taken.
        s.phase_marks.clear();
        let f4 = s.init_build(0, t, Pos::new(20 * 192 + 96, 18 * 192 + 96), false);
        assert_eq!(s.buildings[f4].farm.farm_type, 0);
        assert!(s.phase_marks.is_empty(), "one emitter a city");

        // And the list is joined at placement, in placement order — these
        // four have never been activated.
        assert_eq!(s.farm_order, vec![lone, lone2, f1, f2, f3, f4]);
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
