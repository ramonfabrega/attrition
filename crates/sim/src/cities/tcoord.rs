//! `BuildTypeData::blocked_tcoord@00636db0` as steps over a view (item 3346,
//! `docs/CITIES.md` §2.5): may a building of this type stand on this one
//! tile? [`blocked_tcoord_steps`] is the original's control flow in its
//! order; [`TcoordView`] is what it reads and the callees it makes, which
//! [`SimTcoord`] answers from the crate's [`Sim`] and
//! `sweep/build_type_data_blocked_tcoord.rs` from the emulated original's
//! windows. The method's account is on [`Sim::blocked_tcoord`].
//!
//! The verdict is the original's integer: the seen flag (`iVar7`, the tile
//! was seen or the editor semaphore lifts it) turns nearly every refusal
//! into `0x24` when clear, and the really-seen flag (`local_8`) gates the
//! three that read the tile's contents (a goody's ruins, a building, the
//! blocked tile's `Rare`).

use crate::build::{self, Ident, flags};
use crate::place::Blocked;
use crate::transport::TransportType;
use crate::world::{LANDS, Pos, cell, tile};
use crate::{Player, Sim};

const OFF_MAP: i32 = 0x22;
/// Returned for a tile `who` has not seen; the PDB's name for it reads
/// inverted (`Blocked::Seen`).
const NOT_SEEN: i32 = 0x24;

/// What `blocked_tcoord` reads of the type, the world and the leaders, and
/// the callees it makes. Coordinates are tiles; `who` is `-1` for none.
pub trait TcoordView {
    /// The world's width and height in tiles (`worldc +0x18`, `+0x1c`).
    fn tile_dims(&self) -> (i32, i32);
    /// `game->semaphore.ptr[1] & 8`: the editor's reveal.
    fn sem8(&self) -> bool;
    /// `WorldData::was_seen(&hx, &hy, who)`.
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool;
    /// `WorldData::was_really_seen(&hx, &hy, who)`.
    fn was_really_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool;
    /// The cell record's flags short (`WData +0`) under the tile.
    fn cell_flags(&self, x: i32, y: i32) -> u16;
    /// The tile's `TData` word.
    fn tile_mask(&self, x: i32, y: i32) -> u16;
    /// The signed byte at `WData +0xf` of the cell under the tile.
    fn cell_owner(&self, x: i32, y: i32) -> i32;
    /// `this->vtable[0x60](ident, 0)`: the lineage test on this type.
    fn this_is(&mut self, id: Ident) -> bool;
    /// `this +0x218`: 0 land, 1 water, anything else both.
    fn domain(&self) -> i32;
    /// `LeaderData::can_transport` of the leader, the enum's value.
    fn can_transport(&mut self, who: i32) -> i32;
    /// `ObjectsData::find_building_placed_at(x, y, who, exclude, who)`: the
    /// building's index, negative for none.
    fn placed_at(&mut self, x: i32, y: i32, who: i32, exclude: i32) -> i32;
    /// `this +0x2c0`: the build flags.
    fn flags(&self) -> u32;
    /// `this->vtable[0x64]` (`is_city`).
    fn is_city(&mut self) -> bool;
    /// `this->vtable[0xfc]` (`is_fort`).
    fn is_fort(&mut self) -> bool;
    /// `this->vtable[0x90]` (`is_gather_type`).
    fn gathers(&mut self) -> bool;
    /// `this->vtable[0x94]` (`is_flat`).
    fn flat(&mut self) -> bool;
    /// `LeaderData::has_tribe_bonus(bonus)` of the leader.
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool;
    /// `constants->dutch_fort_placement`.
    fn dutch_fort_placement(&self) -> i32;
    /// `WorldData::get_tregion(&x, &y)`.
    fn tregion(&mut self, x: i32, y: i32) -> i32;
    /// The leader's cities in region `tr` (the short at `+0x125e + 2 tr`).
    fn reg_cities(&self, who: i32, tr: i32) -> i32;
    /// The leader's forts in region `tr` (`+0x12de + 2 tr`).
    fn reg_forts(&self, who: i32, tr: i32) -> i32;
    /// The leader's unbuilt cities (`fort` false) or forts: the list's count.
    fn unbuilt_count(&self, fort: bool, who: i32) -> i32;
    /// The `k`th entry's building (`o`, a short), negative for a free slot.
    fn unbuilt_o(&self, fort: bool, who: i32, k: i32) -> i32;
    /// The `k`th entry's tile: `get_y` then `get_x`, each through
    /// `div_3_table[v >> 6]`.
    fn unbuilt_tile(&mut self, fort: bool, who: i32, k: i32) -> (i32, i32);
    /// `BuildTypeData::get_good(this)`.
    fn good(&mut self) -> i32;
    /// `WorldData::get_land(&x, &y, 1)`.
    fn land(&mut self, x: i32, y: i32) -> i32;
    /// `LandData::get_amount(&lands[land], good)`.
    fn land_amount(&mut self, land: i32, good: i32) -> i32;
}

/// The original's control flow; the verdict is [`Blocked`]'s number.
pub fn blocked_tcoord_steps<V: TcoordView>(
    v: &mut V,
    who: i32,
    x: i32,
    y: i32,
    exclude: i32,
) -> i32 {
    let (tw, th) = v.tile_dims();
    if x < 0 || y < 0 || x >= tw || y >= th {
        return OFF_MAP;
    }
    // `seen` is `iVar7`, `really` is `local_8`.
    let (seen, really) = if who >= 0 {
        if v.was_seen(x >> 1, y >> 1, who) || v.sem8() {
            (true, v.was_really_seen(x >> 1, y >> 1, who) || v.sem8())
        } else {
            (false, false)
        }
    } else {
        (true, true)
    };
    let code = |c: Blocked| if seen { c as i32 } else { NOT_SEEN };
    // `0x24 - (iVar7 != 0)`: where the contents-reading arms end.
    let tail = if seen {
        Blocked::Unseen as i32
    } else {
        NOT_SEEN
    };
    if v.cell_flags(x, y) & cell::GOODY != 0 {
        return if really { Blocked::Ruins as i32 } else { tail };
    }
    let mask = v.tile_mask(x, y);
    if mask & tile::OBJECT == tile::OBJECT_BUILDING {
        return if really {
            Blocked::Building as i32
        } else {
            tail
        };
    }
    if !v.this_is(Ident::Dock) {
        match v.domain() {
            1 => {
                if who >= 0 && v.can_transport(who) == TransportType::None as i32 && !v.sem8() {
                    return code(Blocked::CantTransport);
                }
                if mask & tile::SURFACE != tile::SURFACE_OCEAN {
                    return code(Blocked::Land);
                }
            }
            0 if mask & tile::SURFACE == tile::SURFACE_OCEAN => {
                return code(Blocked::Water);
            }
            _ => {}
        }
    }
    if mask & tile::BLOCKED != 0 {
        if mask & tile::SURFACE == tile::SURFACE_FOREST {
            return code(Blocked::Forest);
        }
        if mask & tile::OBJECT == tile::OBJECT_MOUNTAIN || v.cell_flags(x, y) & cell::MOUNTAIN != 0
        {
            return code(Blocked::Mountain);
        }
    } else if mask & tile::AS_BUILDING == 0 {
        let cf = v.cell_flags(x, y);
        if !v.this_is(Ident::OilWell) && !v.this_is(Ident::OilPlatform) {
            if cf & cell::ROCK != 0 {
                return code(Blocked::Rock);
            }
        } else if cf & cell::OIL == 0 {
            return code(Blocked::NoOil);
        }
        if mask & tile::PLACED != 0 && v.placed_at(x, y, who, exclude) >= 0 {
            return if really {
                Blocked::Building as i32
            } else {
                tail
            };
        }
        if v.flags() & flags::NO_CITY == 0 && mask & tile::CITY_RADIUS == 0 {
            return code(Blocked::OutsideRadius);
        }
        if (v.is_city() || v.is_fort()) && who >= 0 && !v.has_tribe_bonus(who, 0x13) {
            let dutch_anywhere =
                v.is_fort() && v.has_tribe_bonus(who, 0x16) && v.dutch_fort_placement() != 0;
            if !dutch_anywhere && v.cell_owner(x, y) < 0 {
                let tr = v.tregion(x, y);
                let mut taken = v.reg_cities(who, tr) != 0 || v.reg_forts(who, tr) != 0;
                for fort in [false, true] {
                    if taken {
                        break;
                    }
                    for k in 0..v.unbuilt_count(fort, who) {
                        let o = v.unbuilt_o(fort, who, k);
                        if o >= 0 && o != exclude {
                            let (tx, ty) = v.unbuilt_tile(fort, who, k);
                            if v.tregion(tx, ty) == tr {
                                if !seen {
                                    return NOT_SEEN;
                                }
                                taken = true;
                                break;
                            }
                        }
                    }
                }
                if taken {
                    return if v.is_fort() {
                        Blocked::NeutralTerritory as i32
                    } else {
                        Blocked::Territory as i32
                    };
                }
            }
        }
        if v.flags() & flags::NEED_WALL != 0 {
            return code(Blocked::NeedWall);
        }
        if mask & tile::RIVER != 0 {
            return code(Blocked::River);
        }
        if v.gathers() && v.flat() {
            let good = v.good();
            let land = v.land(x, y);
            if v.land_amount(land, good) == 0 {
                return code(Blocked::NoResources);
            }
        }
        return 0;
    }
    // A blocked tile that is no forest or mountain, and an as-building one.
    if really { Blocked::Rare as i32 } else { tail }
}

/// [`TcoordView`] on the crate's state.
pub struct SimTcoord<'a> {
    pub(crate) sim: &'a Sim,
    pub(crate) ty: usize,
}

impl SimTcoord<'_> {
    fn unbuilt(&self, fort: bool, who: i32) -> Vec<usize> {
        let Ok(w) = Player::try_from(who) else {
            return Vec::new();
        };
        if fort {
            self.sim.unbuilt_forts(w).collect()
        } else {
            self.sim.unbuilt_cities(w).collect()
        }
    }
}

fn region(tr: i32) -> Option<u16> {
    u16::try_from(tr).ok()
}

impl TcoordView for SimTcoord<'_> {
    fn tile_dims(&self) -> (i32, i32) {
        self.sim.world.tile_dims()
    }
    fn sem8(&self) -> bool {
        // SEAM: the editor's reveal (`semaphore[1] & 8`) is not carried.
        false
    }
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.sim.was_seen_fog(hx, hy, who as Player)
    }
    fn was_really_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.sim.was_really_seen_fog(hx, hy, who as Player)
    }
    fn cell_flags(&self, x: i32, y: i32) -> u16 {
        let cell = crate::World::cell_of_tile(Pos::new(x, y));
        self.sim.world.cell_data(cell).flags
    }
    fn tile_mask(&self, x: i32, y: i32) -> u16 {
        self.sim.world.tile_mask(Pos::new(x, y))
    }
    fn cell_owner(&self, x: i32, y: i32) -> i32 {
        let cell = crate::World::cell_of_tile(Pos::new(x, y));
        self.sim.world.owner(cell).player().map_or(-1, i32::from)
    }
    fn this_is(&mut self, id: Ident) -> bool {
        build::is(&self.sim.build_types, self.ty, id)
    }
    fn domain(&self) -> i32 {
        match self.sim.build_types[self.ty].domain() {
            build::BuildDomain::Land => 0,
            build::BuildDomain::Water => 1,
            build::BuildDomain::Both => 2,
        }
    }
    fn can_transport(&mut self, who: i32) -> i32 {
        self.sim.transport_level(who as Player) as i32
    }
    fn placed_at(&mut self, x: i32, y: i32, who: i32, exclude: i32) -> i32 {
        let t = Pos::new(x, y);
        let found = match Player::try_from(who) {
            Ok(w) => self
                .sim
                .find_building_placed_at(w, t, usize::try_from(exclude).ok()),
            // `param_3 < 0` matches every player's building, and the
            // exclusion (`param_5` is `who` too) never applies.
            Err(_) => self
                .sim
                .buildings
                .iter()
                .enumerate()
                .position(|(i, b)| b.alive && !b.started && self.sim.build_covers_tile(i, t)),
        };
        found.map_or(-1, |i| i as i32)
    }
    fn flags(&self) -> u32 {
        self.sim.build_types[self.ty].flags
    }
    fn is_city(&mut self) -> bool {
        build::is_city(&self.sim.build_types, self.ty)
    }
    fn is_fort(&mut self) -> bool {
        build::is_fort(&self.sim.build_types, self.ty)
    }
    fn gathers(&mut self) -> bool {
        self.sim.build_types[self.ty].has(flags::GATHER)
    }
    fn flat(&mut self) -> bool {
        self.sim.build_types[self.ty].has(flags::FLAT)
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        let n = &self.sim.nation[who as usize];
        match bonus {
            0x13 => n.lakota,
            0x16 => n.dutch,
            _ => false,
        }
    }
    fn dutch_fort_placement(&self) -> i32 {
        self.sim.tuning.dutch_fort_placement
    }
    fn tregion(&mut self, x: i32, y: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(x, y))
            .map_or(-1, i32::from)
    }
    fn reg_cities(&self, who: i32, tr: i32) -> i32 {
        // SEAM: a region outside `0..64` reads another field of the
        // original's leader record; the crate counts none.
        region(tr).map_or(0, |r| self.sim.reg_cities(who as Player, r))
    }
    fn reg_forts(&self, who: i32, tr: i32) -> i32 {
        region(tr).map_or(0, |r| self.sim.reg_forts(who as Player, r))
    }
    fn unbuilt_count(&self, fort: bool, who: i32) -> i32 {
        self.unbuilt(fort, who).len() as i32
    }
    fn unbuilt_o(&self, fort: bool, who: i32, k: i32) -> i32 {
        self.unbuilt(fort, who)
            .get(k as usize)
            .map_or(-1, |&i| i as i32)
    }
    fn unbuilt_tile(&mut self, fort: bool, who: i32, k: i32) -> (i32, i32) {
        let i = self.unbuilt(fort, who)[k as usize];
        let t = self.sim.buildings[i].pos.tile();
        (t.x, t.y)
    }
    fn good(&mut self) -> i32 {
        crate::ai_place::gather_good(self.sim.build_types[self.ty].ident).map_or(-1, |g| g as i32)
    }
    fn land(&mut self, x: i32, y: i32) -> i32 {
        self.sim.world.land_class_tile(Pos::new(x, y))
    }
    fn land_amount(&mut self, land: i32, good: i32) -> i32 {
        usize::try_from(land)
            .ok()
            .and_then(|i| LANDS.get(i))
            .and_then(|l| l.good.iter().position(|&g| g == good).map(|k| l.amount[k]))
            .unwrap_or(0)
    }
}

impl Blocked {
    /// The verdict `0x636db0` returns, by number.
    pub fn from_code(c: i32) -> Blocked {
        use Blocked::*;
        const ALL: [Blocked; 47] = [
            Clear,
            Building,
            Mountain,
            Rock,
            Ruins,
            NoOil,
            Forest,
            Rare,
            Nearby,
            NoResources,
            NoForest,
            ForestTaken,
            NoMountain,
            MountainTaken,
            Water,
            Land,
            DockLand,
            DockWater,
            DockTerr,
            CityRadius,
            CityDistance,
            FortDistance,
            FortCityDistance,
            Territory,
            EnemyTerritory,
            EnemyTerritory2,
            NeutralTerritory,
            PeacefulTerritory,
            Colonize,
            PortCity,
            PortFort,
            OutsideRadius,
            OutsideTown,
            RazingTown,
            OffMap,
            Unseen,
            Seen,
            Road,
            NeedRoad,
            One,
            OneOther,
            Farm,
            Wonder,
            NeedWall,
            River,
            LostCity,
            CantTransport,
        ];
        ALL[usize::try_from(c).expect("a verdict")]
    }
}
