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
/// `Farms::add_animals@008d8f30`'s three, per animal and in this order —
/// the species coin, the `y` offset and the `x` — spent only on the
/// *drawing* path ([`Sim::farm_stock_pasture`]). The fourth of an animal's
/// four is `Guy::init_real`'s, inside `Objects::init_unit`, and is marked
/// by [`crate::units::anim::SITE_INIT_REAL`] where it is spent.
pub const SITE_ANIMAL_COIN: &str = "Farms::add_animals+0x92";
pub const SITE_ANIMAL_Y: &str = "Farms::add_animals+0x134";
pub const SITE_ANIMAL_X: &str = "Farms::add_animals+0x182";
/// The same two offsets on the coin's **odd** arm, the pig: the decompile
/// writes the loop body twice, once a species, and each copy draws at its
/// own address (`docs/AI.md` §79).
pub const SITE_PIG_Y: &str = "Farms::add_animals+0xc9";
pub const SITE_PIG_X: &str = "Farms::add_animals+0x117";

/// One animal `Farms::add_animals@008d8f30` hands `Objects::init_unit` as
/// `(9, type, x, y, -1, -1, -1)`, with the stamps it then writes through
/// the new object's slot `+0xb4`: `+0x150` the farm's index, `+0x152` its
/// player, `+0x154` the animal's place in the five.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AnimalSpawn {
    pub ty: crate::tech::TypeId,
    pub x: i32,
    pub y: i32,
    pub index: i32,
    pub player: i32,
    pub slot: u8,
}

/// What `add_animals_steps` asks of the world it runs in.
pub(crate) trait AnimalHost {
    /// `Random::get(game_random, 0, 0xffff)`, spent at `site`.
    fn draw(&mut self, site: &'static str) -> i32;
    /// `Objects::init_unit` and the three stamps; `false` ends the loop
    /// (the host has no unit to give).
    fn init_unit(&mut self, a: &AnimalSpawn) -> bool;
}

/// `Farms::add_animals@008d8f30`'s loop. `pasture` is the farm row's
/// `+0xbd == 1` and `active` the building's `flags & 4` (the vtable test
/// the original makes of the object it finds); `pos` is the building's
/// plain position (the original stores it xored with `0x63637`). Per
/// animal, in order: the coin (`& 0x80000001 == 0`, even, is the chicken),
/// the `y` draw, the `x` draw — each `% 0x180` less `0xc0` from the
/// building, `y` first — and `init_unit`.
pub(crate) fn add_animals_steps<H: AnimalHost>(
    host: &mut H,
    pasture: bool,
    active: bool,
    pos: (i32, i32),
    player: i32,
    index: i32,
) {
    if !pasture || !active {
        return;
    }
    for slot in 0..FARM_ANIMALS {
        let chicken = host.draw(SITE_ANIMAL_COIN) & (i32::MIN | 1) == 0;
        let (y_site, x_site) = if chicken {
            (SITE_ANIMAL_Y, SITE_ANIMAL_X)
        } else {
            (SITE_PIG_Y, SITE_PIG_X)
        };
        let ry = host.draw(y_site);
        let rx = host.draw(x_site);
        let at = |base: i32, r: i32| {
            base.wrapping_sub(ANIMAL_SPREAD / 2)
                .wrapping_add(r % ANIMAL_SPREAD)
        };
        let spawn = AnimalSpawn {
            ty: if chicken { FARMCHICKEN } else { FARMPIG },
            x: at(pos.0, rx),
            y: at(pos.1, ry),
            index,
            player,
            slot,
        };
        if !host.init_unit(&spawn) {
            return;
        }
    }
}

/// [`add_animals_steps`]' host in a running [`Sim`]: the stream is the
/// sim's own, and `init_unit` builds the unit and spends its guy's draw.
struct PastureHost<'a> {
    sim: &'a mut Sim,
    b: usize,
    chicken: Option<usize>,
    pig: Option<usize>,
    any: usize,
    out: Vec<usize>,
}

impl AnimalHost for PastureHost<'_> {
    fn draw(&mut self, site: &'static str) -> i32 {
        self.sim.mark(site);
        self.sim.rng.roll()
    }

    fn init_unit(&mut self, a: &AnimalSpawn) -> bool {
        let ty = if a.ty == FARMCHICKEN {
            self.chicken
        } else {
            self.pig
        }
        .unwrap_or(self.any);
        let at = crate::orders::snapped(crate::Pos::new(a.x, a.y));
        match self
            .sim
            .spawn_pasture_animal(self.b, a.slot, ty, at, true, true)
        {
            Some(u) => {
                self.out.push(u);
                true
            }
            None => false,
        }
    }
}

/// What [`farms_add_steps`] asks of the world it runs in — one method a
/// call the original makes, in the order it makes them, so a sweep's host
/// can log them and compare the sequence (item 3281).
pub(crate) trait FarmsAddHost {
    /// `ObjectData` slot `+0xac` of the new building, then `BuildData+0x72`:
    /// its city, `-1` for none. (The record's `memset` is the original's own.)
    fn city(&mut self) -> i32;
    /// `CityData::count_buildings@00739390(city, FARM, 1, 0)`, whole: the
    /// caller takes one off for the building being placed.
    fn count_buildings(&mut self, city: i32) -> i32;
    /// `CityData::count_farms@007368c0`.
    fn count_farms(&mut self, city: i32) -> i32;
    /// `FarmsData::get_nearest_farm_type@008d73a0`: `-1` for none.
    fn nearest(&mut self) -> i32;
    /// `Random::get(game_random, 0, 0xffff)`, spent at `site`.
    fn draw(&mut self, site: &'static str) -> i32;
    /// Slot `+0xb0` then `WallData::tile_corner@00643440`: the footprint's
    /// corner. Art's alone — it feeds the ambience emitter and the heights.
    fn tile_corner(&mut self) -> (i32, i32);
    /// Slot `+0xac` of the new building once more (`+0x72` again), then the
    /// first member of its city's building chain (`CityData+8`), `-1` at
    /// its end.
    fn chain_first(&mut self, city: i32) -> i32;
    /// Slot `+0xac` of member `m`, then the `Build` vtable test and the
    /// `is(FARM)` call it makes (`+0x60` of the type, `+0xb8` of the rest).
    fn member_is_farm(&mut self, m: i32) -> bool;
    /// Slot `+0xac` of member `m`, then `BuildData+0x78`: its farm slot.
    fn member_farm_slot(&mut self, m: i32) -> i32;
    /// Bit 4 of the farm record `slot`'s `farm_type`: the ambience.
    fn slot_ambience(&mut self, slot: i32) -> bool;
    /// Slot `+0xac` of member `m`, then `BuildData+0x74`: the next, `-1` at
    /// the end.
    fn member_next(&mut self, m: i32) -> i32;
    /// `GraphicEvents::add_ambience@008e3e70(1, x, y, 135.0f, who, o)`,
    /// `x` and `y` in 1/192 tiles. Art.
    fn add_ambience(&mut self, x: i32, y: i32);
    /// One of the 25 `TerrainOut::find_data_z@00866560` samples at
    /// `(x, y)`; the original stores it at `terrain_height[col][row]`.
    fn terrain_height(&mut self, row: usize, col: usize, x: i32, y: i32);
    /// `farms.count`.
    fn slot_count(&mut self) -> i32;
    /// Whether record `i` is in use (`valid != 0`).
    fn slot_valid(&mut self, i: i32) -> bool;
    /// `farms.capacity`.
    fn slot_capacity(&mut self) -> i32;
    /// `ArrayBaseSimpleCopy<FarmStruct>::increase_size@00434e10`.
    fn grow(&mut self);
    /// Copies the new record over `slot`; `slot == count` appends and takes
    /// the count a farm up.
    fn store(&mut self, slot: i32, farm_type: u8);
}

/// `Farms::add@008d8a40(who, o)`, from its first call to its return: the
/// type the new farm gets, the city's one ambience emitter, and the record
/// slot. `o` is the building's object index; the slot is the return value.
///
/// The `farm_type` word is chosen in the order the original tests it. A
/// nonzero answer — a pasture, or any nearest-farm answer but zero — jumps
/// to the store and skips the corner, the ambience and the heights; a
/// crop falls through them. `(rand & 3) > 2` is C's signed remainder: a
/// negative draw cannot make a pasture, which the game's own `0..=0xffff`
/// draws never test.
pub(crate) fn farms_add_steps<H: FarmsAddHost>(host: &mut H, o: i32) -> i32 {
    let city = host.city();
    let mut farm_type: u8 = 0;
    let mut crops = 0;
    let mut stored = false;
    if city < 0 {
        // The odd/even coin on the object's own index, not a draw.
        if o & 1 != 0 {
            farm_type = 1;
            stored = true;
        }
    } else {
        let others = host.count_buildings(city).wrapping_sub(1);
        crops = host.count_farms(city);
        if others == crops {
            // `others == 4` with `crops == 0` is dead (they are equal).
            if others == 4 {
                farm_type = 1;
                stored = true;
            } else {
                let near = host.nearest();
                if near != -1 {
                    farm_type = near as u8;
                    stored = near != 0;
                } else if others == 3 {
                    farm_type = 1;
                    stored = true;
                } else {
                    let r = host.draw(SITE_TYPE_COIN);
                    let mut u = r & (i32::MIN | 3);
                    if u < 0 {
                        u = (u.wrapping_sub(1) | -4).wrapping_add(1);
                    }
                    if u > 2 {
                        farm_type = 1;
                        stored = true;
                    }
                }
            }
        }
    }
    if !stored {
        let (x, y) = host.tile_corner();
        if crops > 1 {
            let mut m = host.chain_first(city);
            let mut taken = false;
            while m != -1 {
                if host.member_is_farm(m) {
                    let slot = host.member_farm_slot(m);
                    if slot >= 0 && host.slot_ambience(slot) {
                        taken = true;
                        break;
                    }
                }
                m = host.member_next(m);
            }
            if !taken {
                let rx = host.draw(SITE_AMBIENCE_X);
                let ry = host.draw(SITE_AMBIENCE_Y);
                let ax = x.wrapping_add(1).wrapping_add(rx % 3).wrapping_mul(0xc0);
                let ay = y.wrapping_add(1).wrapping_add(ry % 3).wrapping_mul(0xc0);
                host.add_ambience(ax, ay);
                farm_type |= AMBIENCE;
            }
        }
        for row in 0..5 {
            for col in 0..5 {
                host.terrain_height(
                    row,
                    col,
                    x.wrapping_mul(0xc0).wrapping_add(col as i32 * 0xc0),
                    y.wrapping_mul(0xc0).wrapping_add(row as i32 * 0xc0),
                );
            }
        }
    }
    let count = host.slot_count();
    for i in 0..count {
        if !host.slot_valid(i) {
            host.store(i, farm_type);
            return i;
        }
    }
    if host.slot_capacity() <= count {
        host.grow();
    }
    host.store(count, farm_type);
    count
}

/// [`farms_add_steps`]' host in a running [`Sim`]: the building being
/// placed, its city's chain, and `farm_order` as the record list. The
/// corner, the ambience emitter and the heights are art and take nothing.
struct SimFarmsHost<'a> {
    sim: &'a mut Sim,
    b: usize,
    city: Option<usize>,
    members: Vec<usize>,
    at: usize,
}

impl FarmsAddHost for SimFarmsHost<'_> {
    fn city(&mut self) -> i32 {
        self.city.map_or(-1, |c| c as i32)
    }

    fn count_buildings(&mut self, city: i32) -> i32 {
        self.sim
            .count_buildings(city as usize, crate::cities::build::Ident::Farm, false)
    }

    fn count_farms(&mut self, city: i32) -> i32 {
        self.sim.city_count_farms(city as usize)
    }

    fn nearest(&mut self) -> i32 {
        self.sim.nearest_farm_type(self.b).map_or(-1, i32::from)
    }

    fn draw(&mut self, site: &'static str) -> i32 {
        self.sim.mark(site);
        self.sim.rng.roll()
    }

    fn tile_corner(&mut self) -> (i32, i32) {
        (0, 0)
    }

    fn chain_first(&mut self, _city: i32) -> i32 {
        self.at = 0;
        if self.members.is_empty() { -1 } else { 0 }
    }

    // The chain's "member" token is its position in `members`; the cursor
    // `at` is the walk, so a building listed twice cannot loop it.
    fn member_is_farm(&mut self, m: i32) -> bool {
        let b = self.members[m as usize];
        self.sim.buildings[b].alive
            && self.sim.building_ident(b) == crate::cities::build::Ident::Farm
    }

    fn member_farm_slot(&mut self, m: i32) -> i32 {
        // A farm's record is its own building in this crate.
        let b = self.members[m as usize];
        if self.sim.buildings[b].farm.valid {
            m
        } else {
            -1
        }
    }

    fn slot_ambience(&mut self, slot: i32) -> bool {
        self.sim.buildings[self.members[slot as usize]]
            .farm
            .farm_type
            & AMBIENCE
            != 0
    }

    fn member_next(&mut self, _m: i32) -> i32 {
        self.at += 1;
        if self.at < self.members.len() {
            self.at as i32
        } else {
            -1
        }
    }

    fn add_ambience(&mut self, _x: i32, _y: i32) {}

    fn terrain_height(&mut self, _row: usize, _col: usize, _x: i32, _y: i32) {}

    fn slot_count(&mut self) -> i32 {
        self.sim.cities.farm_order.len() as i32
    }

    fn slot_valid(&mut self, i: i32) -> bool {
        self.sim.buildings[self.sim.cities.farm_order[i as usize]].alive
    }

    fn slot_capacity(&mut self) -> i32 {
        i32::MAX
    }

    fn grow(&mut self) {}

    fn store(&mut self, _slot: i32, farm_type: u8) {
        let bd = &mut self.sim.buildings[self.b];
        bd.farm.farm_type = farm_type;
        bd.farm.valid = true;
    }
}

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
        // is refilled by the original: the first record in the list whose
        // building has closed (`Build::close` → `Farms::remove` clears the
        // `valid` byte) takes the new farm, and only when none is free does
        // the list grow. Great Sahara's coverage lobby is the first capture
        // to demolish farms (the nuke kills the human's three) and then
        // place one: the Persians' 64th building takes the human's first
        // slot and is walked ahead of the farms placed between (item 1639,
        // `docs/AI.md` §175). [`farms_add_steps`] finds it; a farm already
        // in the order (`city.rs`'s activation push) keeps its place.
        let placed = self.cities.farm_order.contains(&b);
        let (o, city) = {
            let bd = &self.buildings[b];
            (i32::from(bd.index), bd.city)
        };
        // `Build::init` line 250: `BuildData+0x78 = Farms::add(...)`, *after*
        // the call. Everything below therefore counts the city's **other**
        // farms, never this one.
        debug_assert!(!self.buildings[b].farm.valid, "a farm gets one record");
        let members: Vec<usize> = city.map_or_else(Vec::new, |c| self.city_members(c).collect());
        let mut host = SimFarmsHost {
            sim: self,
            b,
            city,
            members,
            at: 0,
        };
        let slot = farms_add_steps(&mut host, o) as usize;
        if !placed {
            if slot < self.cities.farm_order.len() {
                self.cities.farm_order[slot] = b;
            } else {
                self.cities.farm_order.push(b);
            }
        }
    }

    /// A building `count_farms` and the ambience walk both count: alive, a
    /// farm, and carrying a farm record. The original's third test is
    /// `BuildData+0x78 >= 0` — the slot `Farms::add` returned — which every
    /// farm has from `Build::init` onwards and nothing else ever has.
    fn is_farm_record(&self, b: usize) -> bool {
        self.buildings[b].alive
            && self.building_ident(b) == crate::cities::build::Ident::Farm
            && self.buildings[b].farm.valid
    }

    /// The city's building chain — the head and its members, which is the
    /// order `CityData`'s walks take.
    fn city_members(&self, c: usize) -> impl Iterator<Item = usize> + '_ {
        let city = &self.cities.cities[c];
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
    /// **A site nobody has started is not a candidate.** The search passes
    /// `find_who = −1`, and `ObjectsData::find_any_building@00659ca0` keeps
    /// an object only when its vtable `+0x50` answers or its owner is
    /// `find_who`; on a building that slot is `WallData::is_started@00472360`,
    /// `flags & 2`. So a farm site placed earlier the same frame, or one no
    /// citizen has reached yet, is invisible here, and the farm placed
    /// beside it can fall through to the `others == 3` pasture
    /// (`docs/AI.md` §79).
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
            if m == b || !self.is_farm_record(m) || !bd.started {
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
        self.add_animals(b, Some(seeds))
    }

    /// The same function **drawing its own**, which is what a pasture
    /// finished mid-game gets: `Build::activate`'s `LAB_00625a36` reaches
    /// `Farms::add_animals` for every food gather building it completes, and
    /// there the five animals cost **four draws each** in this order — the
    /// coin at [`SITE_ANIMAL_COIN`], the `y` at [`SITE_ANIMAL_Y`], the `x` at
    /// [`SITE_ANIMAL_X`], and `Guy::init_real`'s variant inside
    /// `Objects::init_unit`. Twenty draws, and then the five newborns'
    /// `end_time 0` clocks wrap on the same frame's `Objects::inc_time` for
    /// five more (`docs/SYNC.md` §3.11).
    ///
    /// Nothing is borrowed here: the stream is the simulation's own, so the
    /// species and the two offsets are whatever the coin and the two rolls
    /// say. That is the difference between this and
    /// [`Sim::farm_add_animals`], and it is only the *setup*'s pastures that
    /// need the other one.
    pub(crate) fn farm_stock_pasture(&mut self, b: usize) -> Vec<usize> {
        self.add_animals(b, None)
    }

    /// `Farms::add_animals`' body. `borrowed` is `Some` on the setup path,
    /// where the five come from the capture's own trace and no draw is
    /// spent, and `None` on the drawing one.
    fn add_animals(&mut self, b: usize, borrowed: Option<&[AnimalSeed]>) -> Vec<usize> {
        // `Farms::add_animals`' own two guards: the record is a pasture, and
        // the building is active (`flags & 4`). The setup path calls this
        // having just written the type, so both hold there too.
        if self.buildings[b].farm.farm_type & ANIMAL_FARM != ANIMAL_FARM {
            return Vec::new();
        }
        let chicken = self
            .tables
            .unit_types
            .iter()
            .position(|t| t.tree == Some(FARMCHICKEN));
        let pig = self
            .tables
            .unit_types
            .iter()
            .position(|t| t.tree == Some(FARMPIG));
        let Some(any) = chicken.or(pig) else {
            return Vec::new();
        };
        let pos = self.buildings[b].pos;
        let Some(seeds) = borrowed else {
            // The drawing path is the original's loop, whole
            // ([`add_animals_steps`]); the sim is only its host.
            let owner = self.buildings[b].owner as i32;
            let mut host = PastureHost {
                sim: self,
                b,
                chicken,
                pig,
                any,
                out: Vec::with_capacity(FARM_ANIMALS as usize),
            };
            add_animals_steps(&mut host, true, true, (pos.x, pos.y), owner, b as i32);
            return host.out;
        };
        let mut out = Vec::with_capacity(FARM_ANIMALS as usize);
        for slot in 0..FARM_ANIMALS {
            let seed = seeds.get(slot as usize).copied();
            let ty = match seed {
                Some(s) if s.chicken => chicken.unwrap_or(any),
                Some(_) => pig.unwrap_or(any),
                None => any,
            };
            // **Snapped**, because `Unit::init@00612100:69` snaps every
            // unit's starting position — `div_3_table[v >> 4] · 0x30 +
            // 0x18`, the same 48-unit snap a move order's destination
            // takes — before it hands the pair to `Object::init` and to
            // `set_new_location`. `Farms::add_animals` computes
            // `building ± (rnd % 0x180 − 0xc0)` and `Objects::init_unit`
            // passes that straight through, so the two draws name a point
            // the animal is never born on: run39's first is born at
            // `(40536, 40392)` and not the `(40552, 40369)` they say, and
            // its walk is eighteen steps rather than nineteen
            // (`docs/SYNC.md` §3.11, the pair). A pasture animal is the
            // only object this crate places from a raw, unrounded point —
            // every other one comes from a dump, which prints where an
            // object *is* rather than where it was born — so the snap
            // belongs here and not in [`crate::Sim::add_unit`].
            let at = match seed {
                Some(s) => crate::orders::snapped(crate::Pos::new(pos.x + s.dx, pos.y + s.dy)),
                None => pos,
            };
            let Some(u) = self.spawn_pasture_animal(b, slot, ty, at, seed.is_some(), false) else {
                break;
            };
            out.push(u);
        }
        out
    }

    /// The tail of one `Objects::init_unit` of a pasture animal, and
    /// `Farms::add_animals`' stamp of the farm and the slot: the unit of
    /// owner 9 at `at`. `typed` gives it the table's `type_index`; `drawn`
    /// says its guy is `Unit::init`'s own, whose variant roll is the
    /// fourth draw of the animal.
    fn spawn_pasture_animal(
        &mut self,
        b: usize,
        slot: u8,
        ty: usize,
        at: crate::Pos,
        typed: bool,
        drawn: bool,
    ) -> Option<usize> {
        let index = self.find_free(9, crate::UNIT_BASE, crate::BUILD_BASE)?;
        let hits = self.tables.unit_types[ty].hits.max(1);
        let mut unit = crate::Unit::new(9, index, at, hits);
        unit.kind = self.tables.unit_types[ty].kind;
        unit.ty = Some(ty);
        // `Objects::init_unit`'s own two, without which the `MOVE_TO`
        // `think_farm_animal` hands out is an order the animal can
        // never step: it stands still and its arrival — two
        // `Animal::do_idle` draws — never comes.
        unit.movement.speed = self.type_speed(9, ty);
        unit.movement.turning = self.turning_for(ty);
        // `TypeIndex` `FARMPIG`/`FARMCHICKEN`. Without it the gaia
        // table cannot be keyed and every animation length reads
        // `anim::UNKNOWN`, so the clock steps and never wraps — three
        // of run39's six missing `Guy::inc_time` wraps on frame 29.
        // A pasture the trace does not name keeps the −1, because
        // which of the two species it is, is exactly what the coin
        // said and nothing else records.
        if typed {
            unit.type_index = self.tables.unit_types[ty].type_index;
        }
        // `Unit::init@00612100:282–309` (`crate::units::stance`); a pasture
        // animal is not a worker type and takes the `default:` 0.
        unit.stance = self.init_stance(9, ty);
        // `Unit::init`'s formation byte: not a civilian, so 0.
        unit.form = crate::init_form(self.tables.unit_types[ty].type_index);
        unit.farm_animal = Some(FarmAnimal { build: b, slot });
        let u = self.add_unit(unit);
        if drawn {
            // On the drawing path the guy is `Unit::init`'s, and its
            // variant roll is the fourth of the animal's four draws.
            self.init_guys(u, Some(ty));
        } else {
            // The guy exists so the idle roll has something to set. Its
            // piece stays −1 — no dump prints an owner-9 object, so the
            // piece pool cannot name one — and the gaia table is keyed
            // by the type instead.
            self.units[u].guys = vec![crate::units::anim::Guy::fresh(-1)];
        }
        Some(u)
    }

    /// `Animal::think_farm_animal@005d7700`: a pasture's animal, which is
    /// what a herdless animal's `do_idle` falls through to. On its own
    /// 128-frame phase — `(o · (slot + 1) + frame) % 128 == 0` — it takes
    /// **one draw** when the farm covers the tile it is measuring: the
    /// farm's own while nobody **has arrived** to gather there
    /// (`num_gatherers(1, 0)`, so a citizen still walking to the farm
    /// leaves the reference on the farm), and `gather_down`'s head once
    /// somebody has.
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
    pub fn think_farm_animal(&mut self, u: usize) {
        let Some(fa) = self.units[u].farm_animal else {
            return;
        };
        let o = i64::from(self.units[u].index);
        if (o * i64::from(fa.slot + 1) + self.frame).rem_euclid(THINK_PERIOD) != 0 {
            return;
        }
        if self.buildings.get(fa.build).is_none() {
            return;
        }
        // `BuildData::num_gatherers(this, 1, 0)` — the **arrived** count,
        // decoys included, not the chain's length. A citizen joins the
        // chain the moment `add_gather_order` issues (`docs/ORDERS.md`
        // §6.1) and only sets `been_there` when it arrives, so while it is
        // still walking the count is zero and the reference object is the
        // farm. Once it is non-zero the reference is `gather_down`'s head
        // **unfiltered** — whoever is at the front of the chain, arrived
        // or not (`docs/SYNC.md` §3.6).
        let at = if self.num_gatherers(fa.build, true, false) == 0 {
            self.buildings[fa.build].pos
        } else {
            match self.buildings[fa.build]
                .gatherers
                .first()
                .copied()
                .and_then(|g| self.units.get(g))
            {
                Some(unit) => unit.pos,
                None => return,
            }
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

    /// **A farm's close takes its pasture's animals with it**
    /// (`Build::close@00628980:273–297`, item 1591, `docs/AI.md` §164): a
    /// gather building whose good is food (`BuildTypeData::get_good` 0, the
    /// Farm) walks the animals' list (`objects +0x110`) and closes every
    /// one that is active, herdless (`UnitData +0x86 < 0`), not of the air
    /// domain (`+0x218 != 2`) and whose farm reference (`AnimalData +0x150`
    /// / `+0x152`) is this building — `Unit::close(0, −1, 0)` through vslot
    /// `+0x150`: no death draw, the slot held thirty frames. run710's
    /// pasture `0/2002`, flattened by the ICBM on 1734, leaves no chicken
    /// to think on 1747; this crate's `9/9` did, one draw the original
    /// never spent.
    pub(crate) fn close_pasture_animals(&mut self, b: usize) {
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if !unit.alive()
                || unit.herd.is_some()
                || unit.kind.domain == crate::territory::attrition::Domain::Air
                || unit.farm_animal.is_none_or(|fa| fa.build != b)
            {
                continue;
            }
            self.units[u].health = self.units[u].health.min(0);
            self.close_count(u);
            self.relink_squad(u);
            self.close_dead_orders(u);
            self.hold_dead_slot(u);
            self.forget(crate::combat::Obj::Unit(u));
        }
    }

    /// `Farms::inc_time`, from `Objects::inc_time` after the ammo — every
    /// complete, enabled farm, in building order.
    pub fn farms_inc_time(&mut self) {
        // The `Farms` list, in activation order (`Sim::farm_order`) — not
        // the buildings' — because the sprout's `% empty` draw is spent by
        // whichever farm the walk reaches first (`docs/SYNC.md` §4.1).
        let order = self.cities.farm_order.clone();
        for b in order {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.farm.farm_type == ANIMAL_FARM {
                continue;
            }
            if self.building_ident(b) != crate::cities::build::Ident::Farm {
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
mod tests;
