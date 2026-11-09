//! `BuildTypeData::blocked_location@006375b0` as steps over a view (item
//! 3364, `docs/CITIES.md` §2.6): may a building of this type stand at this
//! site? [`blocked_location_steps`] is the original's control flow in its
//! order, from the listing (`llvm-objdump`, `0x6375b0..0x6389c0`) rather than
//! the decompile, whose register arguments it loses; [`LocationView`] is what
//! it reads and the callees it makes, which [`SimLocation`] answers from the
//! crate's [`Sim`] and `sweep/build_type_data_blocked_location*.rs` from the
//! emulated original's windows.
//!
//! The function is one sequence of gates, each a return of a [`Blocked`]
//! number: territory and colonisation (a city or fort, or any other building
//! outside a friendly border), city spacing, fort spacing, the town the
//! building belongs to and what that town already holds, water and land,
//! the clear ring round a city or fort, the oil's fog test, and last the
//! gather survey that fills `param_7`. `who < 0` (the cursor's player-less
//! form) skips every gate up to the water test.
//!
//! **What the listing corrected in the crate's reading** (each a sim fix the
//! sweep forced, `docs/CITIES.md` §2.6): `has_tribe_bonus(0x13)` where the
//! crate read the nation flag; `get_tregion` through [`World::tregion_alt`]
//! where it read the cell's region; the razing refusal (`0x21`, a queue slot
//! of the town's building) the crate never made; the one-per-city and
//! wonder checks keyed on the town's own building (`+0x72`); the sea test of
//! the colonisation arm in the original's ring order and with its early
//! exit; spacing against `get_build`'s position of an unbuilt entry through
//! the cell's *own* region, not the tile's.

use crate::Sim;
use crate::ai_place::circle;
use crate::build::{self, BuildDomain, Ident, flags};
use crate::place::Blocked;
use crate::world::{Cell, Pos, Terrain, tile, vector_dist};

/// The constants the function reads, `constants +0x138..+0x148`.
#[derive(Clone, Copy, Debug)]
pub enum Konst {
    CitySpacing,
    RelaxCitySpacing,
    FirstCityNearCoast,
    FortSpacing,
    FortToEnemyCitySpacing,
}

/// The arguments of the call: `Coord` x and y in world units, the footprint's
/// corner tile, `who` (`-1` for none), the building to exclude (`-1` for none)
/// and whether `param_7` points anywhere.
#[derive(Clone, Copy, Debug)]
pub struct LocArgs {
    pub px: i32,
    pub py: i32,
    pub tx: i32,
    pub ty: i32,
    pub who: i32,
    pub exclude: i32,
    pub has_out: bool,
}

/// What `blocked_location` reads, and the callees it makes. Every method
/// that stands for a call of the original (a vtable slot or a direct callee)
/// takes `&mut self`, so an emulated view can record it in order; a plain
/// read takes `&self`. Players are `0..8`; `who` and `exclude` are `-1` for
/// none; positions are tiles unless a name says cells.
pub trait LocationView {
    /// What `bdata` hands back: the town building's `BuildData`.
    type B: Copy;

    // ---- this (`BuildTypeData`) ----
    /// `this +4`, the type index.
    fn type_index(&self) -> i32;
    /// `this +0x2c0`, the build flags.
    fn flags(&self) -> u32;
    /// `this +0x218`: 0 land, 1 water, anything else both.
    fn domain(&self) -> i32;
    /// `this +0x238 * this +0x234`.
    fn area(&self) -> i32;
    /// `this->vtable[0x60](id, 0)`: the lineage test on this type.
    fn is(&mut self, id: i32) -> bool;
    /// `this->vtable[0x64]` (`is_city`).
    fn is_city(&mut self) -> bool;
    /// `this->vtable[0xfc]` (`is_fort`).
    fn is_fort(&mut self) -> bool;
    /// `this->vtable[0x90]` (`is_gather_type`).
    fn gathers(&mut self) -> bool;
    /// `this->vtable[0x94]` (`is_flat`).
    fn flat(&mut self) -> bool;
    /// `this->vtable[0x1c]` (`is_wonder_type`).
    fn is_wonder(&mut self) -> bool;

    // ---- globals ----
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// `game->semaphore.ptr[1] & 8`: the editor's reveal.
    fn sem8(&self) -> bool;
    /// `constants`' field.
    fn konst(&self, k: Konst) -> i32;
    /// The world's width in cells (`worldc +0`), and the region (a short) of
    /// the cell record at `cy * width + cx`, `-1` for none.
    fn cell_region(&self, cx: i32, cy: i32) -> i32;
    /// `regions[r].size` (`+0x14`).
    fn region_size(&self, r: i32) -> i32;

    // ---- leaders ----
    /// `leaders[i].flags & 1`.
    fn alive(&self, i: i32) -> bool;
    /// `leaders[i].city_num` (`+0x3f8`).
    fn city_num(&self, i: i32) -> i32;
    /// `leaders[i] +8`: its team.
    fn team(&self, i: i32) -> i32;
    /// `leaders[i] +0x74 + 4 k`: the diplomatic row, 2 an ally.
    fn rel(&self, i: i32, k: i32) -> i32;
    /// `leaders[i].reg_cities[tr]` (a short at `+0x125e`).
    fn reg_cities(&self, i: i32, tr: i32) -> i32;
    /// `leaders[i].reg_forts[tr]` (`+0x12de`).
    fn reg_forts(&self, i: i32, tr: i32) -> i32;
    /// `leaders[i].reg_terr[tr]` (`+0x145e`), an unsigned short.
    fn reg_terr(&self, i: i32, tr: i32) -> i32;
    /// `leaders[i]`'s cities list length (`+0x408`) and forts list length
    /// (`+0x428`).
    fn city_count(&self, i: i32) -> i32;
    fn fort_count(&self, i: i32) -> i32;

    // ---- callees on the world and the leaders ----
    /// `WorldData::get_tregion(&x, &y)`.
    fn tregion(&mut self, x: i32, y: i32) -> i32;
    /// `BuildTypeData::non_friendly_territory(this, tx, ty, who)`.
    fn non_friendly(&mut self, who: i32) -> i32;
    /// `BuildTypeData::in_enemy_territory(this, tx, ty, who)`.
    fn in_enemy(&mut self, who: i32) -> bool;
    /// `LeaderData::has_tribe_bonus(bonus)` of `leaders[who]`.
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool;
    /// `LeaderData::has_preq(COLONIZE_BONUS = 0x2af)` of `leaders[who]`.
    fn has_preq(&mut self, who: i32) -> bool;
    /// `WorldData::valid(&x, &y)` on a cell coordinate.
    fn valid(&mut self, x: i32, y: i32) -> bool;
    /// `WorldData::is_ocean(&x, &y)`.
    fn is_ocean(&mut self, x: i32, y: i32) -> bool;
    /// `LeaderData::get_radius(type)` of `leaders[owner]`.
    fn get_radius(&mut self, owner: i32, type_index: i32) -> i32;
    /// `BuildTypeData::get_town(this, x, y, who)`: the town building's index
    /// in `objects[who]`, negative for none.
    fn get_town(&mut self, x: i32, y: i32, who: i32) -> i32;
    /// `BuildTypeData::count_water(this, tx, ty)`.
    fn count_water(&mut self) -> i32;
    /// `BuildTypeData::check_land_adjacent(this, tx, ty, who)`.
    fn check_land_adjacent(&mut self, who: i32) -> i32;
    /// `BuildTypeData::count_mountains_adjacent(this, tx, ty, ring)`.
    fn count_mountains(&mut self, ring: i32) -> i32;
    /// `BuildTypeData::count_trees_adjacent(this, tx, ty, ring)`.
    fn count_trees(&mut self, ring: i32) -> i32;
    /// `WorldData::was_seen(&hx, &hy, who)`, on the half-cell the footprint's
    /// position names.
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool;
    /// `BuildTypeData::calc_gather(this, …)`'s count, the mining list that
    /// of the excluded building when `md` names one.
    fn calc_gather(&mut self, who: i32, md: Option<Self::B>) -> i32;
    /// Stores `*param_7`.
    fn store_gather(&mut self, n: i32);

    // ---- the unbuilt lists (`fort` false: cities, true: forts) ----
    /// The list's length for leader `i`.
    fn unbuilt_count(&self, fort: bool, i: i32) -> i32;
    /// The `k`th entry's object index (a short, sign-extended), negative for
    /// a free slot.
    fn unbuilt_o(&self, fort: bool, i: i32, k: i32) -> i32;
    /// `get_build` on the entry's object, then `Build::vftable ? flags & 2 :
    /// vtable[0x50]()`: whether it has started.
    fn unbuilt_started(&mut self, fort: bool, i: i32, k: i32) -> bool;
    /// `get_build` on the entry's object, its `x` through `div_3_table`.
    fn unbuilt_tx(&mut self, fort: bool, i: i32, k: i32) -> i32;
    /// The same for `y`.
    fn unbuilt_ty(&mut self, fort: bool, i: i32, k: i32) -> i32;

    // ---- the built lists ----
    /// `cities[i].list[j]`: `flags & 1`, region (`+0xa`), tile `x` (`+0xc`)
    /// and `y` (`+0x10`) through `div_3_table`.
    fn city_alive(&self, i: i32, j: i32) -> bool;
    fn city_reg(&self, i: i32, j: i32) -> i32;
    fn city_tx(&self, i: i32, j: i32) -> i32;
    fn city_ty(&self, i: i32, j: i32) -> i32;
    /// The record's owner byte (`+0x5e`) and its building's index (`+8`).
    fn city_owner(&self, i: i32, j: i32) -> i32;
    fn city_bld(&self, i: i32, j: i32) -> i32;
    /// `objects[owner][idx]->+0x18->+4`: the building's type index.
    fn object_type(&self, owner: i32, idx: i32) -> i32;
    /// `CityData::count_buildings(city, this->type, 0, 0)` of `cities[i].list[j]`.
    fn count_buildings(&mut self, i: i32, j: i32) -> i32;
    /// `CityData::get_farm_limit`.
    fn farm_limit(&mut self, i: i32, j: i32) -> i32;
    /// `CityData::num_wonders(city, 1)`.
    fn num_wonders(&mut self, i: i32, j: i32) -> i32;
    /// `forts[i].list[j]`: `flags & 1`.
    fn fort_alive(&self, i: i32, j: i32) -> bool;
    /// The tile of the fort's building, `objects[i][idx]`'s position
    /// (`+0x10`, `+0x14`, xor-masked) through `div_3_table`.
    fn fort_tile(&self, i: i32, j: i32) -> (i32, i32);

    // ---- the town's building ----
    /// `objects[who][idx]->vtable[0x3c]()`: its `BuildData`.
    fn bdata(&mut self, who: i32, idx: i32) -> Self::B;
    /// `BuildData +0x82`: the queue's length.
    fn bd_queue_len(&self, b: Self::B) -> i32;
    /// `BuildData::get_queue(q)`.
    fn get_queue(&mut self, b: Self::B, q: i32) -> i32;
    /// `BuildData +0x72`: the town's index in its owner's cities.
    fn bd_city(&self, b: Self::B) -> i32;
}

/// `0x19e`: the Village, `0x1bb` the Fort, `0x216` the Red Fort, `0x1b0` the
/// Dock, `0x1a5` and `0x1a6` the oil well and platform.
const VILLAGE: i32 = 0x19e;
const FORT: i32 = 0x1bb;
const RED_FORT: i32 = 0x216;
const DOCK: i32 = 0x1b0;
const OIL_WELL: i32 = 0x1a5;
const OIL_PLATFORM: i32 = 0x1a6;

fn vd(dx: i32, dy: i32) -> i32 {
    vector_dist(dx, dy)
}

/// The original's control flow; the verdict is [`Blocked`]'s number.
pub fn blocked_location_steps<V: LocationView>(v: &mut V, a: &LocArgs) -> i32 {
    let LocArgs {
        px,
        py,
        tx: _,
        ty: _,
        who,
        exclude: _,
        has_out: _,
    } = *a;
    let (exclude, has_out) = (a.exclude, a.has_out);
    // `local_20`: the colonisation arm found the sea (`1`).
    let mut coastal = false;
    if who >= 0 {
        let sy = v.div3(py >> 6);
        let sx = v.div3(px >> 6);
        let reg = v.tregion(sx, sy);
        let cell_x = px >> 8;
        let cell_y = py >> 8;
        let site = v.is_city() || v.is_fort();
        if !site {
            if v.flags() & flags::TERRITORY_EXEMPT == 0 && v.type_index() != DOCK && !v.sem8() {
                let r = v.non_friendly(who);
                if r > 2 {
                    return Blocked::EnemyTerritory as i32;
                }
                if r == 2 {
                    return Blocked::NeutralTerritory as i32;
                }
                if r != 0 && !v.has_tribe_bonus(who, 0x13) {
                    return Blocked::NeutralTerritory as i32;
                }
            }
        } else {
            if v.in_enemy(who) {
                return if v.city_num(who) != 0 {
                    Blocked::EnemyTerritory as i32
                } else {
                    Blocked::EnemyTerritory2 as i32
                };
            }
            if v.reg_forts(who, reg) == 0
                && v.reg_cities(who, reg) == 0
                && v.city_num(who) != 0
                && v.konst(Konst::FirstCityNearCoast) != 0
            {
                if !v.has_preq(who) {
                    return Blocked::Colonize as i32;
                }
                let fcnc = v.konst(Konst::FirstCityNearCoast);
                let ridx = (fcnc + ((fcnc >> 31) & 3)) >> 2;
                let count = usize::try_from(ridx)
                    .ok()
                    .and_then(|r| circle().radius.get(r).copied())
                    .unwrap_or(0);
                let (bx, by) = (v.div3(cell_x), v.div3(cell_y));
                for k in 0..count {
                    let x = circle().x[k] + bx;
                    let y = circle().y[k] + by;
                    if v.valid(x, y)
                        && v.is_ocean(x, y)
                        && v.region_size(v.cell_region(x, y)) >= 0x19
                    {
                        coastal = true;
                        break;
                    }
                }
                if !coastal {
                    return if v.is_city() {
                        Blocked::PortCity as i32
                    } else {
                        Blocked::PortFort as i32
                    };
                }
            }
        }
        // 2.6.2 City spacing.
        if v.is_city() {
            for i in 0..8 {
                if !v.alive(i) {
                    continue;
                }
                let mut spacing = v.konst(Konst::CitySpacing);
                if v.region_size(reg) * 9 / 10 <= v.reg_terr(who, reg) {
                    spacing -= v.konst(Konst::RelaxCitySpacing);
                }
                for k in 0..v.unbuilt_count(false, i) {
                    let o = v.unbuilt_o(false, i, k);
                    if o < 0 {
                        continue;
                    }
                    if (i != who || exclude == o) && !v.unbuilt_started(false, i, k) {
                        continue;
                    }
                    let x = v.unbuilt_tx(false, i, k);
                    let y = v.unbuilt_ty(false, i, k);
                    if v.cell_region(x >> 2, y >> 2) == reg
                        && vd((x - sx).abs(), (y - sy).abs()) <= spacing
                    {
                        return Blocked::CityDistance as i32;
                    }
                }
                if i == who || v.reg_cities(i, reg) != 0 {
                    for j in 0..v.city_count(i) {
                        if v.city_alive(i, j)
                            && v.city_reg(i, j) == reg
                            && vd((v.city_tx(i, j) - sx).abs(), (v.city_ty(i, j) - sy).abs())
                                <= spacing
                        {
                            return Blocked::CityDistance as i32;
                        }
                    }
                }
            }
        }
        // 2.6.3 Fort spacing.
        if v.is_fort() {
            let fort_reg = v.cell_region(v.div3(cell_x), v.div3(cell_y));
            for i in 0..8 {
                if !v.alive(i) {
                    continue;
                }
                if i != who {
                    let team = v.team(who);
                    let spacing = if i == team || (v.rel(who, i) == 2 && v.rel(i, team) == 2) {
                        v.konst(Konst::FortSpacing)
                    } else {
                        v.konst(Konst::FortToEnemyCitySpacing)
                    };
                    for k in 0..v.unbuilt_count(false, i) {
                        if v.unbuilt_o(false, i, k) < 0 || !v.unbuilt_started(false, i, k) {
                            continue;
                        }
                        let x = v.unbuilt_tx(false, i, k);
                        let y = v.unbuilt_ty(false, i, k);
                        if v.cell_region(x >> 2, y >> 2) == fort_reg
                            && vd((x - sx).abs(), (y - sy).abs()) <= spacing
                        {
                            return Blocked::FortCityDistance as i32;
                        }
                    }
                    if v.reg_cities(i, fort_reg) != 0 {
                        for j in 0..v.city_count(i) {
                            if v.city_alive(i, j)
                                && v.city_reg(i, j) == fort_reg
                                && vd((v.city_tx(i, j) - sx).abs(), (v.city_ty(i, j) - sy).abs())
                                    <= spacing
                            {
                                return Blocked::FortCityDistance as i32;
                            }
                        }
                    }
                }
                for k in 0..v.unbuilt_count(true, i) {
                    let o = v.unbuilt_o(true, i, k);
                    if o < 0 {
                        continue;
                    }
                    if (i != who || exclude == o) && !v.unbuilt_started(true, i, k) {
                        continue;
                    }
                    let x = v.unbuilt_tx(true, i, k);
                    let y = v.unbuilt_ty(true, i, k);
                    if v.cell_region(x >> 2, y >> 2) == fort_reg
                        && vd((x - sx).abs(), (y - sy).abs()) <= v.konst(Konst::FortSpacing)
                    {
                        return Blocked::FortDistance as i32 + i32::from(who != i);
                    }
                }
                if who == i || v.reg_forts(i, fort_reg) != 0 {
                    for j in 0..v.fort_count(i) {
                        if !v.fort_alive(i, j) {
                            continue;
                        }
                        let (x, y) = v.fort_tile(i, j);
                        if vd((x - sx).abs(), (y - sy).abs()) <= v.konst(Konst::FortSpacing) {
                            return Blocked::FortDistance as i32 + i32::from(who != i);
                        }
                    }
                }
            }
        }
        // 2.6.4 Must belong to a town.
        if v.flags() & flags::NO_CITY == 0 {
            let town = v.get_town(sx, sy, who);
            if town < 0 {
                return if v.flags() & flags::NEEDS_TOWN != 0 {
                    Blocked::OutsideTown as i32
                } else {
                    Blocked::OutsideRadius as i32
                };
            }
            let bd = v.bdata(who, town);
            if v.bd_queue_len(bd) > 0 {
                let mut q = 0;
                loop {
                    let bd = v.bdata(who, town);
                    let id = v.get_queue(bd, q);
                    if id == 0x29a || id == 0x29b {
                        return Blocked::RazingTown as i32;
                    }
                    q += 1;
                    let bd = v.bdata(who, town);
                    if q >= v.bd_queue_len(bd) {
                        break;
                    }
                }
            }
            if v.flags() & flags::ONE_PER_CITY != 0 {
                let bd = v.bdata(who, town);
                let mine = v.bd_city(bd);
                if v.count_buildings(who, mine) != 0 {
                    for j in 0..v.city_count(who) {
                        if v.city_alive(who, j) && j != mine {
                            let owner = v.city_owner(who, j);
                            let bidx = v.city_bld(who, j);
                            let t = v.object_type(owner, bidx);
                            let r = v.get_radius(owner, t).min(0x40);
                            let d = vd(
                                (v.city_tx(who, j) - sx).abs(),
                                (v.city_ty(who, j) - sy).abs(),
                            );
                            if d <= r && v.count_buildings(who, j) == 0 {
                                return Blocked::OneOther as i32;
                            }
                        }
                    }
                    return Blocked::One as i32;
                }
            }
            if v.type_index() == 0x1a1 {
                let bd = v.bdata(who, town);
                let c = v.bd_city(bd);
                let n = v.count_buildings(who, c);
                if v.farm_limit(who, c) <= n {
                    return Blocked::Farm as i32;
                }
            }
            if v.is_wonder() {
                let bd = v.bdata(who, town);
                let c = v.bd_city(bd);
                if !v.is(RED_FORT) {
                    let held = i32::from(v.has_tribe_bonus(who, 7));
                    if v.num_wonders(who, c) > held {
                        return Blocked::Wonder as i32;
                    }
                }
            }
        }
    }
    // 2.6.5 Water and land.
    if !v.is(DOCK) {
        match v.domain() {
            1 => {
                if v.count_water() < v.area() {
                    return Blocked::Land as i32;
                }
            }
            0 if v.count_water() != 0 => return Blocked::Water as i32,
            _ => {}
        }
    } else {
        let w = v.count_water();
        let a3 = v.area().wrapping_mul(3);
        let a3 = a3.wrapping_add((a3 >> 31) & 3) >> 2;
        if w < a3 {
            if w != 0 && v.check_land_adjacent(who) != 0 && !v.sem8() {
                return Blocked::DockTerr as i32;
            }
            return Blocked::DockWater as i32;
        }
        match v.check_land_adjacent(who) {
            1 => {
                if !v.sem8() {
                    return Blocked::DockTerr as i32;
                }
            }
            2 => return Blocked::DockLand as i32,
            _ => {}
        }
    }
    // 2.6.6 Clear ground around a city or fort.
    if v.is_city() || v.is_fort() {
        let ring = if coastal { 1 } else { 2 };
        if v.count_mountains(ring) != 0 {
            return Blocked::Mountain as i32;
        }
        if v.count_trees(ring) != 0 {
            return Blocked::Forest as i32;
        }
    }
    // 2.6.7 The oil's fog test.
    if (v.is(OIL_WELL) || v.is(OIL_PLATFORM)) && who >= 0 {
        let hy = v.div3(py >> 7);
        let hx = v.div3(px >> 7);
        if !v.was_seen(hx, hy, who) && !v.sem8() {
            return Blocked::NoOil as i32;
        }
    }
    // 2.6.8 The gather survey.
    if v.gathers() && !v.flat() {
        let md = if exclude >= 0 && who >= 0 {
            Some(v.bdata(who, exclude))
        } else {
            None
        };
        let n = v.calc_gather(who, md);
        if has_out {
            v.store_gather(n.max(0));
        }
        let t = v.type_index();
        if n == 0 {
            return if t == 0x1a3 {
                Blocked::NoMountain as i32
            } else {
                Blocked::NoResources as i32 + i32::from(t == 0x1a2)
            };
        }
        if n < 0 {
            return i32::from(t != 0x1a2) * 2 + Blocked::ForestTaken as i32;
        }
    }
    0
}

/// [`LocationView`] on the crate's state. The object handles are global
/// building indices; the crate carries no `TypeIndex`, so [`Self::type_index`]
/// names the exact types the function tests from the building's [`Ident`],
/// and `object_type`/`get_radius` pass the crate's own type number.
pub struct SimLocation<'a> {
    pub(crate) sim: &'a Sim,
    pub(crate) ty: usize,
    pub(crate) pos: Pos,
    pub(crate) corner: Pos,
    pub(crate) out: Option<i32>,
}

impl SimLocation<'_> {
    fn bt(&self) -> &build::BuildType {
        &self.sim.build_types[self.ty]
    }
    fn player(i: i32) -> Option<crate::Player> {
        crate::Player::try_from(i).ok()
    }
    /// The indices into [`Sim::cities`] of leader `i`'s list, dead records
    /// included, in order.
    fn owner_cities(&self, i: i32) -> Vec<usize> {
        let Some(p) = Self::player(i) else {
            return Vec::new();
        };
        self.sim
            .cities
            .iter()
            .enumerate()
            .filter(|(_, c)| c.owner == p)
            .map(|(c, _)| c)
            .collect()
    }
    fn city_at(&self, i: i32, j: i32) -> usize {
        self.owner_cities(i)[usize::try_from(j).expect("a city index")]
    }
    fn unbuilt(&self, fort: bool, i: i32) -> Vec<usize> {
        let Some(p) = Self::player(i) else {
            return Vec::new();
        };
        if fort {
            self.sim.unbuilt_forts(p).collect()
        } else {
            self.sim.unbuilt_cities(p).collect()
        }
    }
    fn entry(&self, fort: bool, i: i32, k: i32) -> usize {
        self.unbuilt(fort, i)[usize::try_from(k).expect("an entry")]
    }
    fn region(tr: i32) -> Option<u16> {
        u16::try_from(tr).ok()
    }
}

impl LocationView for SimLocation<'_> {
    type B = usize;

    fn type_index(&self) -> i32 {
        let b = self.bt();
        match b.ident {
            Ident::Village => 0x19e,
            Ident::Town => 0x19f,
            Ident::Metropolis => 0x1a0,
            Ident::Farm => 0x1a1,
            Ident::Woodcutter => 0x1a2,
            Ident::Mine => 0x1a3,
            Ident::OilWell => 0x1a5,
            Ident::OilPlatform => 0x1a6,
            Ident::Dock => 0x1b0,
            Ident::Fort => 0x1bb,
            Ident::ForbiddenCity => 0x213,
            Ident::RedFort => 0x216,
            _ if b.wonder => 0x20e,
            _ => 0,
        }
    }
    fn flags(&self) -> u32 {
        self.bt().flags
    }
    fn domain(&self) -> i32 {
        match self.bt().domain() {
            BuildDomain::Land => 0,
            BuildDomain::Water => 1,
            BuildDomain::Both => 2,
        }
    }
    fn area(&self) -> i32 {
        self.bt().x_size.wrapping_mul(self.bt().y_size)
    }
    fn is(&mut self, id: i32) -> bool {
        let ident = match id {
            VILLAGE => Ident::Village,
            FORT => Ident::Fort,
            RED_FORT => Ident::RedFort,
            DOCK => Ident::Dock,
            OIL_WELL => Ident::OilWell,
            OIL_PLATFORM => Ident::OilPlatform,
            _ => return false,
        };
        build::is(&self.sim.build_types, self.ty, ident)
    }
    fn is_city(&mut self) -> bool {
        build::is_city(&self.sim.build_types, self.ty)
    }
    fn is_fort(&mut self) -> bool {
        build::is_fort(&self.sim.build_types, self.ty)
    }
    fn gathers(&mut self) -> bool {
        self.bt().has(flags::GATHER)
    }
    fn flat(&mut self) -> bool {
        self.bt().has(flags::FLAT)
    }
    fn is_wonder(&mut self) -> bool {
        self.bt().wonder
    }

    fn div3(&self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn sem8(&self) -> bool {
        // SEAM: the editor's reveal (`semaphore[1] & 8`) is not carried.
        false
    }
    fn konst(&self, k: Konst) -> i32 {
        let t = &self.sim.tuning;
        match k {
            Konst::CitySpacing => t.city_spacing,
            Konst::RelaxCitySpacing => t.relax_city_spacing,
            Konst::FirstCityNearCoast => t.first_city_near_coast,
            Konst::FortSpacing => t.fort_spacing,
            Konst::FortToEnemyCitySpacing => t.fort_to_enemy_city_spacing,
        }
    }
    fn cell_region(&self, cx: i32, cy: i32) -> i32 {
        self.sim
            .world
            .region_of(Cell::new(cx, cy))
            .map_or(-1, i32::from)
    }
    fn region_size(&self, r: i32) -> i32 {
        Self::region(r).map_or(0, |r| self.sim.world.region_size(r))
    }

    fn alive(&self, i: i32) -> bool {
        Self::player(i)
            .is_some_and(|p| usize::from(p) < self.sim.players.len() && self.sim.player_alive(p))
    }
    fn city_num(&self, i: i32) -> i32 {
        Self::player(i).map_or(0, |p| self.sim.city_num(p))
    }
    fn team(&self, i: i32) -> i32 {
        i
    }
    fn rel(&self, i: i32, k: i32) -> i32 {
        match (Self::player(i), Self::player(k)) {
            (Some(a), Some(b)) if self.sim.allied_with(a, b) => 2,
            _ => 0,
        }
    }
    fn reg_cities(&self, i: i32, tr: i32) -> i32 {
        // SEAM: a region outside `0..64` reads another field of the
        // original's leader record; the crate counts none.
        match (Self::player(i), Self::region(tr)) {
            (Some(p), Some(r)) => self.sim.reg_cities(p, r),
            _ => 0,
        }
    }
    fn reg_forts(&self, i: i32, tr: i32) -> i32 {
        match (Self::player(i), Self::region(tr)) {
            (Some(p), Some(r)) => self.sim.reg_forts(p, r),
            _ => 0,
        }
    }
    fn reg_terr(&self, i: i32, tr: i32) -> i32 {
        match (Self::player(i), Self::region(tr)) {
            (Some(p), Some(r)) => self.sim.reg_terr(p, r),
            _ => 0,
        }
    }
    fn city_count(&self, i: i32) -> i32 {
        self.owner_cities(i).len() as i32
    }
    fn fort_count(&self, i: i32) -> i32 {
        Self::player(i).map_or(0, |p| self.sim.built_forts(p).count() as i32)
    }

    fn tregion(&mut self, x: i32, y: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(x, y))
            .map_or(-1, i32::from)
    }
    fn non_friendly(&mut self, who: i32) -> i32 {
        self.sim
            .non_friendly_territory(who as crate::Player, self.ty, self.corner)
    }
    fn in_enemy(&mut self, who: i32) -> bool {
        self.sim
            .in_enemy_territory(who as crate::Player, self.ty, self.corner)
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        let n = &self.sim.nation[who as usize];
        match bonus {
            0x13 => n.lakota,
            7 => n.egyptians,
            _ => false,
        }
    }
    fn has_preq(&mut self, who: i32) -> bool {
        self.sim.colonize_preq_held(who as crate::Player)
    }
    fn valid(&mut self, x: i32, y: i32) -> bool {
        self.sim.world.contains(Cell::new(x, y))
    }
    fn is_ocean(&mut self, x: i32, y: i32) -> bool {
        self.sim
            .world
            .region_of(Cell::new(x, y))
            .is_some_and(|r| self.sim.world.terrain(r) == Terrain::Sea)
    }
    fn get_radius(&mut self, owner: i32, type_index: i32) -> i32 {
        let ty = usize::try_from(type_index).ok();
        self.sim.city_radius(owner as crate::Player, ty)
    }
    fn get_town(&mut self, _x: i32, _y: i32, who: i32) -> i32 {
        self.sim
            .get_town(who as crate::Player, self.ty, self.pos)
            .map_or(-1, |c| self.sim.cities[c].building as i32)
    }
    fn count_water(&mut self) -> i32 {
        self.sim.count_water(self.ty, self.corner)
    }
    fn check_land_adjacent(&mut self, who: i32) -> i32 {
        self.sim
            .check_land_adjacent(Self::player(who), self.ty, self.corner)
    }
    fn count_mountains(&mut self, ring: i32) -> i32 {
        self.sim
            .ring(self.ty, self.corner, ring)
            .into_iter()
            .filter(|t| self.sim.world.tile_mask(*t) & tile::OBJECT == tile::OBJECT_MOUNTAIN)
            .count() as i32
    }
    fn count_trees(&mut self, ring: i32) -> i32 {
        self.sim
            .ring(self.ty, self.corner, ring)
            .into_iter()
            .filter(|t| self.sim.world.tile_mask(*t) & tile::SURFACE == tile::SURFACE_FOREST)
            .count() as i32
    }
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.sim.was_seen_fog(hx, hy, who as crate::Player)
    }
    fn calc_gather(&mut self, who: i32, md: Option<usize>) -> i32 {
        // SEAM: the player-less form (`who = -1`) walks the survey with a
        // negative owner, which has no oracle; it answers a positive count,
        // so nothing is refused. No caller in this crate passes it.
        match Self::player(who) {
            Some(p) => self.sim.site_gather_count(self.ty, p, self.corner, md),
            None => 1,
        }
    }
    fn store_gather(&mut self, n: i32) {
        self.out = Some(n);
    }

    fn unbuilt_count(&self, fort: bool, i: i32) -> i32 {
        self.unbuilt(fort, i).len() as i32
    }
    fn unbuilt_o(&self, fort: bool, i: i32, k: i32) -> i32 {
        self.entry(fort, i, k) as i32
    }
    fn unbuilt_started(&mut self, fort: bool, i: i32, k: i32) -> bool {
        self.sim.buildings[self.entry(fort, i, k)].started
    }
    fn unbuilt_tx(&mut self, fort: bool, i: i32, k: i32) -> i32 {
        self.sim.buildings[self.entry(fort, i, k)].pos.tile().x
    }
    fn unbuilt_ty(&mut self, fort: bool, i: i32, k: i32) -> i32 {
        self.sim.buildings[self.entry(fort, i, k)].pos.tile().y
    }

    fn city_alive(&self, i: i32, j: i32) -> bool {
        self.sim.cities[self.city_at(i, j)].alive
    }
    fn city_reg(&self, i: i32, j: i32) -> i32 {
        self.sim.cities[self.city_at(i, j)]
            .reg
            .map_or(-1, i32::from)
    }
    fn city_tx(&self, i: i32, j: i32) -> i32 {
        self.sim.cities[self.city_at(i, j)].pos.tile().x
    }
    fn city_ty(&self, i: i32, j: i32) -> i32 {
        self.sim.cities[self.city_at(i, j)].pos.tile().y
    }
    fn city_owner(&self, i: i32, j: i32) -> i32 {
        i32::from(self.sim.cities[self.city_at(i, j)].owner)
    }
    fn city_bld(&self, i: i32, j: i32) -> i32 {
        self.sim.cities[self.city_at(i, j)].building as i32
    }
    fn object_type(&self, _owner: i32, idx: i32) -> i32 {
        usize::try_from(idx)
            .ok()
            .and_then(|b| self.sim.buildings.get(b))
            .and_then(|b| b.ty)
            .map_or(-1, |t| t as i32)
    }
    fn count_buildings(&mut self, i: i32, j: i32) -> i32 {
        let c = self.city_at(i, j);
        self.sim.count_buildings(c, self.bt().ident, false)
    }
    fn farm_limit(&mut self, i: i32, j: i32) -> i32 {
        self.sim.farm_limit(self.city_at(i, j))
    }
    fn num_wonders(&mut self, i: i32, j: i32) -> i32 {
        self.sim.num_wonders(self.city_at(i, j), true)
    }
    fn fort_alive(&self, _i: i32, _j: i32) -> bool {
        true
    }
    fn fort_tile(&self, i: i32, j: i32) -> (i32, i32) {
        let p = Self::player(i).expect("a leader");
        let b = self
            .sim
            .built_forts(p)
            .nth(usize::try_from(j).expect("a fort"))
            .expect("a fort");
        let t = self.sim.buildings[b].pos.tile();
        (t.x, t.y)
    }

    fn bdata(&mut self, _who: i32, idx: i32) -> usize {
        usize::try_from(idx).expect("a building")
    }
    fn bd_queue_len(&self, b: usize) -> i32 {
        self.sim.buildings[b].queue.items.len() as i32
    }
    fn get_queue(&mut self, _b: usize, _q: i32) -> i32 {
        // SEAM: the crate's queue items are crate types, not `TypeIndex`es,
        // and no order the crate queues is a razing (`0x29a`, `0x29b`).
        0
    }
    fn bd_city(&self, b: usize) -> i32 {
        let Some(c) = self
            .sim
            .cities
            .iter()
            .position(|c| c.building == b || c.members.contains(&b))
        else {
            return 0;
        };
        let owner = i32::from(self.sim.cities[c].owner);
        self.owner_cities(owner)
            .iter()
            .position(|&x| x == c)
            .map_or(0, |r| r as i32)
    }
}
