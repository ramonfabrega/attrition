//! **`BuildTypeData::calc_gather@00639e40`'s steps** (item 3386): what one gather
//! building's type yields at a corner tile, as a walk over a [`TypeGatherView`].
//! The swept steps read the world, the leaders and the constants only through
//! the view, so a sweep's port is a view over the emulated original's windows
//! (`docs/ECONOMY.md`, `crates/sim/src/gather.rs`).
//!
//! The control flow is the listing's, call for call: every leaf is one direct
//! callee, one virtual call or one inline read of the constants, asked in the
//! order the listing asks it, which is what the sweep holds. The function has
//! five arms, chosen by two virtual calls: a flat type (`is_flat`, slot
//! `+0x94`: the farm, the oil well and platform) walks the footprint's tiles; a
//! university (`is(0x1a4)`, slot `+0x60`) takes `max_knowledge_gatherers`; a
//! mine (`is(0x1a3)`) takes the nearest mountain range or cliff; anything else
//! (the woodcutter's camp) walks the octagonal circle of cells.
//!
//! **Not wired.** `crates/sim/src/gather.rs` (`gather_slots_raw`, the count) and
//! `economy.rs` (`per_gatherer`, the rate) split this one function in two and
//! model neither the leader's nation and wonder terms nor the circle's own
//! scratch list in the original's shape; wiring either would move pins
//! (parked 3386). The steps are the original's, for the sweep to hold.

/// The constants the function reads, by name (`Constants` offsets in the
/// listing; the view maps each to its source).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Konst {
    /// `constants + 0x280`.
    PeasantRate,
    /// `constants + 0x29c`.
    OilRate,
    /// `constants + 0x284 + 4 * level`.
    ScholarRate(i32),
    /// `constants + 0x7ac`.
    JapaneseFishingBoats,
    /// `constants + 0x658`.
    EgyptianFarmWealth,
    /// `constants + 0xc40`.
    RiverResourceValue,
    /// `constants + 0x80c`.
    IroquoisFood,
    /// `constants + 0x6c8`.
    FrenchWoodies,
    /// `constants + 0x508`.
    TajFarms,
    /// `constants + 0x518`.
    KremlinFarms,
    /// `constants + 0x728`.
    GermanMiners,
    /// `constants + 0x59c`.
    IncaWealthPerMiner,
}

/// `calc_gather`'s arguments, by their meaning (the listing's `+0x8…+0x38`).
#[derive(Clone, Copy, Debug)]
pub struct TypeGatherArgs {
    /// `param_4`: the caller's second "do it" flag (`+0x14`).
    pub flag4: bool,
    /// `param_5`: the city, `-1` for none (`+0x18`).
    pub city: i32,
    /// `param_6`: the player, `-1` for none (`+0x1c`).
    pub who: i32,
    /// `param_7`: the leader's resource bonuses apply (`+0x20`).
    pub bonuses: bool,
    /// `param_8`: the corner tile's x (`+0x24`).
    pub tx: i32,
    /// `param_9`: the corner tile's y (`+0x28`).
    pub ty: i32,
    /// `param_10`: yield rates are wanted (`+0x2c`).
    pub rates: bool,
    /// `param_11`: the gatherers the building has (`+0x30`).
    pub gatherers: i32,
    /// `param_13 != nullptr`: the caller names the building's own
    /// `MiningList` (`+0x38`).
    pub list: bool,
}

/// What `BuildTypeData::calc_gather` asks of the type, the leaders, the world
/// and the constants. Every method is one call of the listing.
pub trait TypeGatherView {
    /// `CityData::enhancer_amount(Cities[who][city], good, 100)`: the three
    /// calls `Cities::operator[]`, `ConstArray::operator[]` and the method.
    fn enhancer_amount(&mut self, who: i32, city: i32, good: i32) -> i32;
    /// `LeaderData::calc_resource_bonuses(Leaders[who], out)`.
    fn resource_bonuses(&mut self, who: i32, out: &mut [i32; 6]);
    /// Vtable `+0x94`: `is_flat`.
    fn is_flat(&mut self) -> bool;
    /// Vtable `+0x60`: `is(ty, 0)`.
    fn is_type(&mut self, ty: i32) -> bool;
    /// `BuildTypeData::max_flat_gatherers@006365f0`.
    fn max_flat_gatherers(&mut self) -> i32;
    /// The type's footprint width in tiles (`this + 0x234`).
    fn x_size(&mut self) -> i32;
    /// The type's footprint height in tiles (`this + 0x238`).
    fn y_size(&mut self) -> i32;
    /// `WorldData::is_valid(&x, &y)` on tile coordinates.
    fn tile_valid(&mut self, x: i32, y: i32) -> bool;
    /// `WorldData::get_whose(&cx, &cy)` on cell coordinates.
    fn get_whose(&mut self, cx: i32, cy: i32) -> i32;
    /// `Leaders[who].is_ally(other)`.
    fn is_ally(&mut self, who: i32, other: i32) -> bool;
    /// `WorldData::get_land(&x, &y, 1)` on tile coordinates.
    fn get_land(&mut self, x: i32, y: i32) -> i32;
    /// `WorldData::is_river(&x, &y)` on tile coordinates.
    fn is_river(&mut self, x: i32, y: i32) -> bool;
    /// `Lands[land].good[k]` (`+4 + 4k`), one `Lands::operator[]` call.
    fn land_good(&mut self, land: i32, k: i32) -> i32;
    /// `Lands[land].amount[k]` (`+0x14 + 4k`), one `Lands::operator[]` call.
    fn land_amount(&mut self, land: i32, k: i32) -> i32;
    /// `BuildTypeData::gathers(good)`.
    fn gathers(&mut self, good: i32) -> bool;
    /// `Leaders[who].has_tribe_bonus(k)`.
    fn has_tribe_bonus(&mut self, who: i32, k: i32) -> bool;
    /// `Leaders[who].has_wonder(k)`.
    fn has_wonder(&mut self, who: i32, k: i32) -> bool;
    /// `BuildTypeData::max_knowledge_gatherers(city, who)`.
    fn max_knowledge_gatherers(&mut self, city: i32, who: i32) -> i32;
    /// `Leaders[who].get_university()`.
    fn get_university(&mut self, who: i32) -> i32;
    /// `BuildTypeData::gather_radius@0063bc60`, the first call of the mine and
    /// camp arms.
    fn gather_radius(&mut self) -> i32;
    /// A named constant.
    fn konst(&mut self, k: Konst) -> i32;

    // The camp's circle walk.

    /// The function-local static `MiningList`'s thread-safe initialisation
    /// (`_Init_thread_header`, the constructor, `atexit`, `_Init_thread_footer`
    /// on the first call; nothing after).
    fn static_init(&mut self);
    /// `MiningList::clear(&static)`.
    fn list_clear(&mut self);
    /// `corner_tile(tx, ty, &x, &y)`: the footprint's centre in world units; `site`
    /// names the call (0 the mine's, 1 the camp's).
    fn corner_tile(&mut self, tx: i32, ty: i32, site: i32) -> (i32, i32);
    /// `div_3_table[i]` (the `Coord` to `WCoord`/`TCoord` conversions).
    fn div3(&mut self, i: i32) -> i32;
    /// `circle_radius[ring]`.
    fn circle_len(&mut self, ring: i32) -> i32;
    /// `circle_x[i]`.
    fn circle_x(&mut self, i: i32) -> i32;
    /// `circle_y[i]`.
    fn circle_y(&mut self, i: i32) -> i32;
    /// `WorldData::valid(&cx, &cy)` on cell coordinates.
    fn cell_valid(&mut self, cx: i32, cy: i32) -> bool;
    /// `vector_dist(dx, dy)`, the octagonal metric.
    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32;
    /// `WorldData::get_who(&cx, &cy)` on cell coordinates.
    fn get_who(&mut self, cx: i32, cy: i32) -> i32;
    /// The "Some coder needs to pass a valid who in here." report
    /// (`String` ×2, `Error::report`, `~String` ×2), the `site`'s line number.
    fn report_no_who(&mut self, site: i32, who: i32);
    /// `WorldData::is_gathered_from(&tx, &ty)`.
    fn is_gathered_from(&mut self, tx: i32, ty: i32) -> bool;
    /// `WorldData::is_tree_at(&tx, &ty)`.
    fn is_tree_at(&mut self, tx: i32, ty: i32) -> bool;
    /// `WorldData::is_mountain_at(&tx, &ty)`.
    fn is_mountain_at(&mut self, tx: i32, ty: i32) -> bool;
    /// `WorldData::is_forest(&cx, &cy)`.
    fn is_forest(&mut self, cx: i32, cy: i32) -> bool;
    /// `WorldData::get_land(&cx, &cy, 1)` on cell coordinates.
    fn get_land_cell(&mut self, cx: i32, cy: i32) -> i32;
    /// `ArrayBase<TCoordData>::add(&static, {tx, ty})`.
    fn list_add(&mut self, tx: i32, ty: i32);
    /// `ArrayBase<TCoordData>::find(param_13, {tx, ty})`.
    fn list_find(&mut self, tx: i32, ty: i32) -> i32;
    /// `BuildTypeData::total_gather_access(list, x, y, who)`, `list` the
    /// caller's own (`param_13`) or the static scratch list.
    fn total_gather_access(&mut self, own_list: bool, x: i32, y: i32, who: i32) -> i32;

    // The mine's mountain and cliff search.

    /// `param_13->mtn`, the signed byte at `+0x1c`.
    fn list_mtn(&mut self) -> i32;
    /// `param_13->cliff`, the signed byte at `+0x1d`.
    fn list_cliff(&mut self) -> i32;
    /// `WorldData::get_region(&cx, &cy)`; `second` is the cliff search's call.
    fn get_region(&mut self, cx: i32, cy: i32, second: bool) -> i32;
    /// `MountainsData::find_nearest(ox, oy, region, &dist)`: the nearest range's index
    /// and its distance.
    fn mountains_find_nearest(&mut self, ox: i32, oy: i32, region: i32) -> (i32, i32);
    /// `get_scary_cliffs()` then `CliffsData::find_nearest(ox, oy, region, &dist)`.
    fn cliffs_find_nearest(&mut self, ox: i32, oy: i32, region: i32) -> (i32, i32);
    /// The cliff's tile count: `get_scary_cliffs()`, `ConstArray::operator[](cliff)`, `get_style`.
    fn cliff_len(&mut self, cliff: i32) -> i32;
    /// The cliff's tile `i`, in tile coordinates (two walks of the same chain, `x` then `y`).
    fn cliff_tile(&mut self, cliff: i32, i: i32) -> (i32, i32);
    /// `CliffsData::gather_size(cliff, who, list)`.
    fn cliff_gather_size(&mut self, cliff: i32, who: i32) -> i32;
    /// The mountain range `mtn`'s tile count (`get_style` of its coordinate list).
    fn range_len(&mut self, mtn: i32) -> i32;
    /// The range's offset of tile `i` from its location (`x` then `y` arrays).
    fn range_offset(&mut self, mtn: i32, i: i32) -> (i32, i32);
    /// `*MountainsData::mnt_loc_x(mtn)` (`y` false) or `_y`; `site` 0 the tile loop's call, 1 the
    /// gather size's.
    fn mnt_loc(&mut self, mtn: i32, y: bool, site: i32) -> i32;
    /// `MountainRangeData::gather_size(x, y, who, list)` of the range `mtn`.
    fn range_gather_size(&mut self, mtn: i32, x: i32, y: i32, who: i32) -> i32;
}

/// `(v + (v >> 31 & 0xff)) >> 8` — the listing's signed divide by 256.
fn div256(v: i32) -> i32 {
    v.wrapping_add((v >> 31) & 0xff) >> 8
}

/// `(v + (v >> 31 & 0xf)) >> 4` — the signed divide by 16.
fn div16(v: i32) -> i32 {
    v.wrapping_add((v >> 31) & 0xf) >> 4
}

/// The listing's `imul` by an enhancer then `idiv` by the base: `a * num / den`
/// in 32-bit wrapping arithmetic, the divide truncating toward zero.
fn scale(a: i32, num: i32, den: i32) -> i32 {
    a.wrapping_mul(num).wrapping_div(den)
}

/// The rate's tail shared by the arms: the leader's resource bonus
/// (`calc_resource_bonuses`' array, `0x100` meaning none) applied to the sum
/// and to the per-gatherer rate (`63a2d7…63a32e`).
fn apply_bonus(out: &mut i32, rate: &mut i32, bonus: i32) {
    if bonus != 0x100 {
        *out = div256(out.wrapping_mul(bonus));
        *rate = div256(rate.wrapping_mul(bonus));
    }
}

/// `WCoord::traverse_x@0046eef0`: tile `k % 4` of cell `x`, the remainder
/// taking the dividend's sign.
fn traverse_x(x: i32, k: i32) -> i32 {
    let mut r = k & i32::MIN | (k & 3);
    if r < 0 {
        r = ((r - 1) | -4) + 1;
    }
    r.wrapping_add(x.wrapping_mul(4))
}

/// `WCoord::traverse_y@0046eed0`: tile row `k / 4` of cell `y`.
fn traverse_y(y: i32, k: i32) -> i32 {
    (k.wrapping_add((k >> 31) & 3) >> 2).wrapping_add(y.wrapping_mul(4))
}

/// `WCoord::operator_TCoord@004613b0`: a cell's centre tile.
fn cell_tile(c: i32) -> i32 {
    c.wrapping_mul(4).wrapping_add(2)
}

/// `BuildTypeData::calc_gather(out, count, rate, …)`.
///
/// `out` receives the six goods' yields, `count` the gatherer slots (or `-1`),
/// `rate` the last good's per-gatherer rate.
pub fn type_calc_gather_steps<V: TypeGatherView>(
    v: &mut V,
    a: &TypeGatherArgs,
    out: &mut [i32; 6],
    count: &mut i32,
    rate: &mut i32,
) {
    let mut acc = [0i32; 6];
    let mut enh = [0i32; 6];
    let mut bonus = [0x100i32; 6];
    for i in 0..6 {
        acc[i] = 0;
        out[i] = 0;
        enh[i] = if a.city < 0 {
            100
        } else {
            v.enhancer_amount(a.who, a.city, i as i32)
        };
    }
    if !a.rates && !a.flag4 {
        *count = 1;
        *rate = 0;
        return;
    }
    if a.who >= 0 && a.bonuses {
        v.resource_bonuses(a.who, &mut bonus);
    }
    if v.is_flat() {
        flat_arm(v, a, out, count, rate, &mut acc, &enh, &bonus);
    } else if v.is_type(0x1a4) {
        knowledge_arm(v, a, out, count, rate, &enh, &bonus);
    } else if v.is_type(0x1a3) {
        mine_arm(v, a, out, count, rate, &enh, &bonus);
    } else {
        camp_arm(v, a, out, count, rate, &mut acc, &enh, &bonus);
    }
}

/// The flat arm (`639fa1…63a3ad`): the footprint's tiles, their land's goods,
/// then each good's rate.
#[allow(clippy::too_many_arguments)]
fn flat_arm<V: TypeGatherView>(
    v: &mut V,
    a: &TypeGatherArgs,
    out: &mut [i32; 6],
    count: &mut i32,
    rate: &mut i32,
    acc: &mut [i32; 6],
    enh: &[i32; 6],
    bonus: &[i32; 6],
) {
    *count = v.max_flat_gatherers();
    *rate = 0;
    let mut x = a.tx;
    while x < a.tx.wrapping_add(v.x_size()) {
        let mut y = a.ty;
        while y < a.ty.wrapping_add(v.y_size()) {
            if v.tile_valid(x, y) {
                let whose = v.get_whose(x >> 2, y >> 2);
                let mine = whose < 0 || whose == a.who || v.is_ally(a.who, whose);
                if mine {
                    let land = v.get_land(x, y);
                    let river = v.is_river(x, y);
                    for k in 0..4 {
                        let good = v.land_good(land, k);
                        let mut amount = v.land_amount(land, k);
                        if good >= 0 {
                            if river {
                                amount = amount.wrapping_mul(v.konst(Konst::RiverResourceValue));
                            }
                            acc[good as usize] = acc[good as usize].wrapping_add(amount);
                        }
                    }
                }
            }
            y += 1;
        }
        x += 1;
    }
    for g in 0..6usize {
        if !a.rates || !v.gathers(g as i32) {
            continue;
        }
        let base = if g == 5 {
            v.konst(Konst::OilRate)
        } else {
            v.konst(Konst::PeasantRate)
        };
        *rate = div256(acc[g].wrapping_mul(base)).max(0);
        if enh[g] != 100 {
            *rate = scale(*rate, enh[g], 100);
        }
        if g == 0 && a.who >= 0 && v.has_tribe_bonus(a.who, 0xf) {
            let pct = v.konst(Konst::JapaneseFishingBoats).wrapping_add(100);
            *rate = scale(*rate, pct, 100);
        }
        out[g] = out[g].wrapping_add(rate.wrapping_mul(a.gatherers));
        apply_bonus(&mut out[g], rate, bonus[g]);
    }
    if v.is_type(0x1a1)
        && a.who >= 0
        && v.has_tribe_bonus(a.who, 7)
        && v.konst(Konst::EgyptianFarmWealth) > 0
        && a.gatherers != 0
    {
        let w = v.konst(Konst::EgyptianFarmWealth);
        out[2] = out[2].wrapping_add(w.wrapping_mul(0x10));
    }
}

/// The knowledge arm (`63a3b2…63a534`): the university's scholars at the
/// leader's university level.
fn knowledge_arm<V: TypeGatherView>(
    v: &mut V,
    a: &TypeGatherArgs,
    out: &mut [i32; 6],
    count: &mut i32,
    rate: &mut i32,
    enh: &[i32; 6],
    bonus: &[i32; 6],
) {
    *count = v.max_knowledge_gatherers(a.city, a.who);
    let cap = (*count).min(a.gatherers);
    let level = v.get_university(a.who).wrapping_sub(1);
    let per = v.konst(Konst::ScholarRate(level)).wrapping_mul(0x10);
    *rate = div256(per).max(0);
    if enh[3] != 100 {
        *rate = scale(*rate, enh[3], 100);
    }
    out[3] = out[3].wrapping_add(rate.wrapping_mul(cap));
    apply_bonus(&mut out[3], rate, bonus[3]);
}

/// The mine arm (`63a555…63af96`): the nearest mountain range or cliff, its tiles' ownership, then the
/// wealth and the rate.
fn mine_arm<V: TypeGatherView>(
    v: &mut V,
    a: &TypeGatherArgs,
    out: &mut [i32; 6],
    count: &mut i32,
    rate: &mut i32,
    enh: &[i32; 6],
    bonus: &[i32; 6],
) {
    let radius6 = v.gather_radius().wrapping_mul(0xc0);
    *count = 0;
    *rate = 0;
    let (ox, oy) = v.corner_tile(a.tx, a.ty, 0);
    let (mut mtn, mut cliff, dmtn, dcliff);
    if !a.list || (v.list_mtn() < 0 && v.list_cliff() < 0) {
        let (wx, wy) = (v.div3(ox >> 8), v.div3(oy >> 8));
        let region = v.get_region(wx, wy, false);
        (mtn, dmtn) = v.mountains_find_nearest(ox, oy, region);
        let region = v.get_region(wx, wy, true);
        (cliff, dcliff) = v.cliffs_find_nearest(ox, oy, region);
    } else {
        mtn = v.list_mtn();
        cliff = v.list_cliff();
        dmtn = 0;
        dcliff = 0;
    }
    if cliff >= 0 && mtn >= 0 && dmtn > dcliff {
        mtn = -1;
    }
    if mtn < 0 {
        if cliff >= 0 {
            // A cliff past the reach is dropped (`cliff = -1`), and nothing reads it again.
            if dcliff <= radius6 {
                let mut i = 0;
                while i < v.cliff_len(cliff) {
                    let (px, py) = v.cliff_tile(cliff, i);
                    i += 1;
                    if v.is_tree_at(px, py) {
                        continue;
                    }
                    let who2 = v.get_who(px >> 2, py >> 2);
                    if a.who < 0 {
                        v.report_no_who(0xa18, a.who);
                    }
                    if who2 >= 0 && who2 != a.who && !v.is_ally(a.who, who2) {
                        continue;
                    }
                    if !a.list && v.is_gathered_from(px, py) {
                        cliff = -2;
                        *count = -1;
                        break;
                    }
                }
                if cliff >= 0 {
                    *count = v.cliff_gather_size(cliff, a.who);
                }
            }
        }
    } else if dmtn <= radius6 {
        // (A range past the reach is dropped, `mtn = -1`, and nothing reads it again.)
        let mut i = 0;
        while i < v.range_len(mtn) {
            let (ax, ay) = v.range_offset(mtn, i);
            i += 1;
            let px = ax.wrapping_add(v.mnt_loc(mtn, false, 0).wrapping_shl(2));
            let py = ay.wrapping_add(v.mnt_loc(mtn, true, 0).wrapping_shl(2));
            if v.is_tree_at(px, py) || !v.is_mountain_at(px, py) {
                continue;
            }
            let who2 = v.get_who(px >> 2, py >> 2);
            if a.who < 0 {
                v.report_no_who(0x9ea, a.who);
            }
            if who2 >= 0 && who2 != a.who && !v.is_ally(a.who, who2) {
                continue;
            }
            if !a.list && v.is_gathered_from(px, py) {
                mtn = -2;
                *count = -1;
                break;
            }
        }
        if mtn >= 0 {
            let y = v.mnt_loc(mtn, true, 1);
            let x = v.mnt_loc(mtn, false, 1);
            *count = v.range_gather_size(mtn, x, y, a.who);
        }
    }
    if *count != 0 && a.who >= 0 {
        if v.has_tribe_bonus(a.who, 0xc) {
            *count = count.wrapping_add(v.konst(Konst::GermanMiners));
        }
        if v.has_wonder(a.who, 0x21b) {
            *count = count.wrapping_add(v.konst(Konst::TajFarms));
        }
        if v.has_wonder(a.who, 0x21a) {
            *count = count.wrapping_add(v.konst(Konst::KremlinFarms));
        }
    }
    *rate = div256(v.konst(Konst::PeasantRate)) << 4;
    if enh[4] != 100 {
        *rate = scale(*rate, enh[4], 100);
    }
    let c = (*count).max(0);
    let cap = if c < a.gatherers { c } else { a.gatherers };
    if a.who >= 0 && v.has_tribe_bonus(a.who, 2) && v.konst(Konst::IncaWealthPerMiner) > 0 {
        let w = v.konst(Konst::IncaWealthPerMiner);
        out[2] = out[2].wrapping_add(cap.wrapping_mul(w).wrapping_shl(4));
    }
    out[4] = out[4].wrapping_add(rate.wrapping_mul(cap));
    apply_bonus(&mut out[4], rate, bonus[4]);
}

/// The camp arm (`63af9b…63bc23`): the circle of cells around the footprint,
/// then each good's slots and rate.
#[allow(clippy::too_many_arguments)]
fn camp_arm<V: TypeGatherView>(
    v: &mut V,
    a: &TypeGatherArgs,
    out: &mut [i32; 6],
    count: &mut i32,
    rate: &mut i32,
    acc: &mut [i32; 6],
    enh: &[i32; 6],
    bonus: &[i32; 6],
) {
    let radius = v.gather_radius();
    v.static_init();
    let mut taken = false;
    v.list_clear();
    let (ox, oy) = v.corner_tile(a.tx, a.ty, 1);
    let cx0 = v.div3(ox >> 8);
    let cy0 = v.div3(oy >> 8);
    let r3 = radius.wrapping_add(3);
    let ring = r3.wrapping_add((r3 >> 31) & 3) >> 2;
    let mut i = 0;
    while i < v.circle_len(ring) {
        let x = cx0.wrapping_add(v.circle_x(i));
        let y = cy0.wrapping_add(v.circle_y(i));
        i += 1;
        if !v.cell_valid(x, y) {
            continue;
        }
        let (ax, ay) = (v.div3(ox >> 6), v.div3(oy >> 6));
        let d = v.vector_dist(cell_tile(x).wrapping_sub(ax), cell_tile(y).wrapping_sub(ay));
        if d > radius {
            continue;
        }
        let who2 = v.get_who(x, y);
        if a.who < 0 {
            v.report_no_who(0xa82, a.who);
        }
        if who2 >= 0 && who2 != a.who && !v.is_ally(a.who, who2) {
            continue;
        }
        let (tx2, ty2) = (cell_tile(x), cell_tile(y));
        if !a.list {
            if v.is_gathered_from(tx2, ty2) {
                taken = true;
                continue;
            }
            if v.is_type(0x1a2) {
                for k in 0..16 {
                    let (px, py) = (traverse_x(x, k), traverse_y(y, k));
                    if v.is_tree_at(px, py) {
                        v.list_add(px, py);
                    }
                }
                if !v.is_tree_at(tx2, ty2) {
                    continue;
                }
            } else if v.is_type(0x1a3) {
                if v.is_forest(x, y) {
                    continue;
                }
                for k in 0..16 {
                    let (px, py) = (traverse_x(x, k), traverse_y(y, k));
                    if v.is_mountain_at(px, py) {
                        v.list_add(px, py);
                    }
                }
                if !v.is_mountain_at(tx2, ty2) {
                    continue;
                }
            }
        } else if v.list_find(tx2, ty2) < 0 {
            continue;
        }
        let land = v.get_land_cell(x, y);
        let mut slots = 0i32;
        for k in 0..4 {
            let good = v.land_good(land, k);
            let amount = v.land_amount(land, k);
            if good < 0 || !v.gathers(good) || v.land_amount(land, k) == 0 {
                continue;
            }
            for _ in 0..16 {
                acc[good as usize] = acc[good as usize].wrapping_add(amount);
                slots += 1;
            }
            // The listing leaves the land's four slots after the first one that pays (`63b721`
            // jumps to the tribe test, not to the next slot).
            break;
        }
        if v.has_tribe_bonus(a.who, 0x12) {
            acc[0] = acc[0].wrapping_add(slots);
        }
    }

    // The rivers under the footprint.
    let mut rivers = 0i32;
    let mut x = a.tx;
    while x < a.tx.wrapping_add(v.x_size()) {
        let mut y = a.ty;
        while y < a.ty.wrapping_add(v.y_size()) {
            if v.tile_valid(x, y) && v.is_river(x, y) {
                rivers += 1;
            }
            y += 1;
        }
        x += 1;
    }

    *count = 1;
    *rate = 0;
    let mut cap = a.gatherers;
    for g in 0..6usize {
        if !a.rates {
            continue;
        }
        if !v.gathers(g as i32) {
            if v.konst(Konst::IroquoisFood) != 0 {
                if g != 0 {
                    continue;
                }
                if !v.has_tribe_bonus(a.who, 0x12) {
                    continue;
                }
            } else {
                continue;
            }
        }
        acc[g] = div16(acc[g].wrapping_mul(rivers)).wrapping_add(acc[g]);
        let peasant = v.konst(Konst::PeasantRate);
        let per = div256(peasant);
        *count = div16(acc[g].wrapping_add(8));
        if *count == 0 && taken {
            *count = -1;
            break;
        }
        if *count != 0 && a.who >= 0 && g == 1 {
            if v.has_tribe_bonus(a.who, 10) {
                *count = count.wrapping_add(v.konst(Konst::FrenchWoodies));
            }
            if v.has_wonder(a.who, 0x21b) {
                *count = count.wrapping_add(v.konst(Konst::TajFarms));
            }
            if v.has_wonder(a.who, 0x21a) {
                *count = count.wrapping_add(v.konst(Konst::KremlinFarms));
            }
        }
        if g == 0 && v.has_tribe_bonus(a.who, 0x12) && v.gathers(1) {
            *rate = v
                .konst(Konst::IroquoisFood)
                .wrapping_mul(0x10)
                .wrapping_add(*rate);
        } else {
            *rate = per << 4;
        }
        if enh[g] != 100 {
            *rate = scale(*rate, enh[g], 100);
        }
        if g == 5 {
            *rate = rate.wrapping_mul(*count);
            if *count > 1 {
                *count = 1;
            }
        }
        let limit = (*count).min(cap);
        let access = v.total_gather_access(a.list, ox, oy, a.who).wrapping_shl(1);
        cap = limit.min(access);
        *count = (*count).min(access);
        acc[g] = cap.wrapping_mul(*rate);
        if bonus[g] != 0x100 {
            acc[g] = div256(acc[g].wrapping_mul(bonus[g]));
            *rate = div256(rate.wrapping_mul(bonus[g]));
        }
        out[g] = out[g].wrapping_add(acc[g]);
    }
    v.list_clear();
}
