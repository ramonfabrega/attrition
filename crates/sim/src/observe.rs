//! **The read door** (`docs/STRUCTURE.md` §5 move 9, `docs/RENDERER.md`
//! §2.3, item 4017): what a client may see of a running game, as an owned
//! value it can keep past the tick.
//!
//! [`Sim::observe`] copies the objects a renderer draws into a [`Scene`];
//! [`Sim::observe_terrain`] and [`Sim::observe_territory`] copy the two grids
//! that change rarely, and the scene's version counters say when to fetch
//! them again. Four rules hold here, and RENDERER §2.3 states them:
//!
//! 1. Nothing in it is a float or a renderer type.
//! 2. An object is keyed by the original's identity, `(who, o)` — never by
//!    its slot in a `Vec`.
//! 3. It is read-only by type: every function takes `&self`, draws nothing
//!    from `rng` and writes nothing. `soak`'s
//!    `observing_every_tick_leaves_the_digest_alone` is the check.
//! 4. It reports and does not interpret: no culling, no draw order, no
//!    "visible on screen".

use crate::movement::Angle;
use crate::territory::attrition::Domain;
use crate::{Owner, Player, Pos, Sim};

/// An object's identity as the original keeps it: its owner and its number
/// in the owner's object array (`SubObjectData::o`), the pair a dump prints
/// and the harness links on. Units are numbered from 0 and buildings from
/// 2000. Whether a number is reused after a death inside one game is not
/// established (`docs/RENDERER.md` §7), so a client should not assume an
/// `ObjId` names one object for the whole game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjId {
    pub who: Player,
    pub o: i16,
}

/// One tick's drawable state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    /// The frame count, [`Sim::frame`].
    pub frame: i64,
    /// Every living unit on the map, in the simulation's store order. A
    /// unit at no health keeps its record here until it is reused, but the
    /// original has taken it out of its object array — a corpse is drawn
    /// from [`Scene::deaths`] (item 4026).
    pub units: Vec<UnitScene>,
    /// Every building in use, in the store order.
    pub buildings: Vec<BuildingScene>,
    /// The projectiles in flight.
    pub projectiles: Vec<ProjectileScene>,
    /// The corpses fading.
    pub deaths: Vec<DeathScene>,
    /// The viewer's fog plane on the half-cell grid (`fog_xs × fog_ys`,
    /// row-major): [`fog::UNSEEN`], [`fog::EXPLORED`] or [`fog::VISIBLE`].
    /// `None` for an observer who sees everything, and on a world with no
    /// fog grid installed.
    pub fog: Option<Vec<u8>>,
    /// Moves when the terrain grids change: fetch
    /// [`Sim::observe_terrain`] again.
    pub terrain_version: u32,
    /// Moves when a cell's claimants change: fetch
    /// [`Sim::observe_territory`] again.
    pub territory_version: u32,
}

/// One unit on the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitScene {
    pub id: ObjId,
    pub owner: Player,
    /// An index into the unit types; `None` for the bare object.
    pub ty: Option<usize>,
    /// The unit's position, 1/768 of a cell.
    pub pos: Pos,
    /// The pinned tile height under it, [`crate::World::tile_z`].
    pub z: i32,
    /// `GuyData::angle`, guy 0's facing. North is zero.
    pub facing: Angle,
    pub health: i32,
    pub max_health: i32,
    /// Its figures, one per guy.
    pub figures: Vec<FigureScene>,
}

/// One figure of a unit: where it stands and which piece, animation and
/// frame the original would draw (`docs/ANIM.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FigureScene {
    /// The figure's own position — a crew body's, where it has one; guy
    /// 0's own body (`Movement::body`), which a push that does not move
    /// the guys leaves behind its unit's `pos` (item 4027); and otherwise
    /// its unit's.
    pub pos: Pos,
    /// The figure's own height, the `GUY` record's `z` (`GuyData +0x14`),
    /// as `Guy::update_z@005d9950` keeps it: a sea unit's figure at 0, the
    /// water's surface; an aircraft's at its own altitude (`Airframe::z`);
    /// every other on the ground under its point (`World::data_z`), which
    /// may be below zero. Not the unit's [`UnitScene::z`]: a figure stands
    /// where the renderer draws it (`docs/RENDERER.md` §2.3, item 4026).
    pub z: i32,
    /// The figure's own facing where it has one, otherwise its unit's.
    pub facing: Angle,
    /// The graphic piece.
    pub gpiece: i32,
    /// The `UnitAnim` slot.
    pub anim: i8,
    pub cur_time: u32,
    pub end_time: u32,
}

/// One building in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingScene {
    pub id: ObjId,
    pub owner: Player,
    /// An index into the building types; `None` for the bare object.
    pub ty: Option<usize>,
    pub pos: Pos,
    /// Its own height, `SubObjectData::z_internal` ([`crate::cities::building::Building::z`]):
    /// the ground under it once its own terraform has run, which a later
    /// terraform does not move (item 4035).
    pub z: i32,
    /// Hit points left and the hit points it was built with.
    pub health: i32,
    pub hits: i32,
    /// `WallData::is_active`: finished.
    pub active: bool,
    /// Construction progress as the simulation keeps it: `constr_time`
    /// against `construct_hits` (`docs/CITIES.md` §3).
    pub constr_time: i32,
    pub construct_hits: i32,
    /// The type it was placed as, `BuildData::orig_type` — the one type a
    /// dump prints, and the type the renderer's K of M counts
    /// (`docs/RENDERER.md` §5.4; item 4035).
    pub orig_ty: Option<usize>,
    /// `WallData::job_counter` and `BuildData::construct_time(0)`: the
    /// share built, which `WallOut::draw@00641930` reads to choose whether
    /// an unfinished building is drawn (`docs/RENDERER.md` §3.5).
    pub job_counter: i32,
    pub construct_time: i32,
    /// The art coordinates `GraphicPieces::get_build_gpiece` walks from
    /// (item 4035), as `Wall::update_gpiece@00643070` passes them: the
    /// nation of its **founder** (`BuildData::founder`, `+0x7f`), an index
    /// into the tech tree's tribes — a captured city keeps the art of the
    /// nation that built it — and its age, `BuildData::max_age`, which this
    /// crate does not keep: the owner's ages owned
    /// (`LeaderDataEncrypt::ages`) stand in, the value `Leader::gain_tech`
    /// writes to every building of the player.
    pub tribe: usize,
    pub age: i32,
}

/// One projectile in flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileScene {
    pub owner: Player,
    pub launch: Pos,
    pub landing: Pos,
    pub angle: Angle,
    /// `cur_time` of `total_time`.
    pub cur_time: i32,
    pub total_time: i32,
    pub air: bool,
}

/// One dead unit's fading record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeathScene {
    pub who: i32,
    pub o: i32,
    pub first_frame: i64,
    pub cur_anim: i32,
}

/// The terrain grids, fetched when [`Scene::terrain_version`] moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerrainScene {
    /// The map's size in cells.
    pub width: i32,
    pub height: i32,
    /// The tile heights, row-major over `width × 4` by `height × 4` tiles;
    /// empty on a world with no height table.
    pub tile_z: Vec<i32>,
    /// Each cell's `WData.land` and `land_sub`, row-major.
    pub land: Vec<i8>,
    pub land_sub: Vec<u8>,
    /// The placed mountain ranges: each range's tiles.
    pub mountains: Vec<Vec<Pos>>,
    pub version: u32,
}

/// Who holds each cell, fetched when [`Scene::territory_version`] moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerritoryScene {
    pub width: i32,
    pub height: i32,
    /// Each cell's owner, row-major.
    pub owner: Vec<Owner>,
    pub version: u32,
}

/// The values of [`Scene::fog`].
pub mod fog {
    /// The viewer has never seen the half-cell.
    pub const UNSEEN: u8 = 0;
    /// Seen once (`WorldData::seen2`), not in sight now.
    pub const EXPLORED: u8 = 1;
    /// In the viewer's line of sight now (`WorldData::seen`).
    pub const VISIBLE: u8 = 2;
}

impl Sim {
    /// This tick's drawable state as `viewer` sees it, or everything when
    /// `viewer` is `None`. Reads only; see the module note.
    pub fn observe(&self, viewer: Option<Player>) -> Scene {
        let units = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.on_map && u.alive())
            .map(|(i, u)| {
                let facing = u.movement.facing;
                let domain = self.unit_domain_of(i);
                UnitScene {
                    id: ObjId {
                        who: u.owner,
                        o: u.index,
                    },
                    owner: u.owner,
                    ty: u.ty,
                    pos: u.pos,
                    z: self.world.tile_z(u.pos.tile()),
                    facing,
                    health: u.health,
                    max_health: u.max_health,
                    figures: u
                        .guys
                        .iter()
                        .enumerate()
                        .map(|(k, g)| {
                            // Guy 0's own point is the unit's body
                            // (`Movement::body`, `GuyData +0xc`), which a
                            // push that does not move the guys leaves
                            // behind (item 4027).
                            let own = if k == 0 { u.movement.body.pos } else { u.pos };
                            let pos = g.follow.map_or(own, |f| f.body.pos);
                            FigureScene {
                                pos,
                                z: match domain {
                                    Domain::Sea => 0,
                                    Domain::Air => u.airframe.z,
                                    Domain::Land => self.world.data_z(pos.x, pos.y).0,
                                },
                                facing: g.follow.map_or(facing, |f| f.facing),
                                gpiece: g.gpiece,
                                anim: g.anim,
                                cur_time: g.cur_time,
                                end_time: g.end_time,
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        let buildings = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive)
            .map(|(i, b)| BuildingScene {
                id: ObjId {
                    who: b.owner,
                    o: b.index,
                },
                owner: b.owner,
                ty: b.ty,
                pos: b.pos,
                z: b.z,
                health: b.health,
                hits: b.hits,
                active: b.active,
                constr_time: b.constr_time,
                construct_hits: b.construct_hits,
                orig_ty: b.orig_ty,
                job_counter: b.job_counter,
                construct_time: self.construct_time_of(i),
                tribe: self.tech.get(b.founder as usize).map_or(0, |p| p.tribe),
                age: self.tech.get(b.owner as usize).map_or(0, |p| p.ages),
            })
            .collect();
        let projectiles = self
            .combat
            .projectiles
            .iter()
            .map(|p| ProjectileScene {
                owner: p.owner,
                launch: p.launch,
                landing: p.landing,
                angle: p.angle,
                cur_time: p.cur_time,
                total_time: p.total_time,
                air: p.air,
            })
            .collect();
        let deaths = self
            .combat
            .deaths
            .iter()
            .map(|d| DeathScene {
                who: d.who,
                o: d.o,
                first_frame: d.first_frame,
                cur_anim: d.cur_anim,
            })
            .collect();
        Scene {
            frame: self.frame,
            units,
            buildings,
            projectiles,
            deaths,
            fog: viewer.and_then(|p| self.fog_plane(p)),
            terrain_version: self.world.terrain_version(),
            territory_version: self.world.territory_version(),
        }
    }

    /// The terrain grids as they stand.
    pub fn observe_terrain(&self) -> TerrainScene {
        let w = &self.world;
        let (width, height) = (w.width(), w.height());
        let cells = (0..height).flat_map(|y| (0..width).map(move |x| crate::Cell::new(x, y)));
        let (land, land_sub) = cells
            .map(|c| {
                let d = w.cell_data(c);
                (d.land, d.land_sub)
            })
            .unzip();
        TerrainScene {
            width,
            height,
            tile_z: w.tile_z_grid().to_vec(),
            land,
            land_sub,
            mountains: self
                .terrain
                .mountains
                .iter()
                .map(|m| m.tiles.clone())
                .collect(),
            version: w.terrain_version(),
        }
    }

    /// Who holds each cell as it stands.
    pub fn observe_territory(&self) -> TerritoryScene {
        let w = &self.world;
        let (width, height) = (w.width(), w.height());
        TerritoryScene {
            width,
            height,
            owner: (0..height)
                .flat_map(|y| (0..width).map(move |x| w.owner(crate::Cell::new(x, y))))
                .collect(),
            version: w.territory_version(),
        }
    }

    /// One player's fog plane, `None` on a world with no fog grid.
    fn fog_plane(&self, p: Player) -> Option<Vec<u8>> {
        let w = &self.world;
        if !w.has_fog() {
            return None;
        }
        let bit = 1u8.checked_shl(u32::from(p)).unwrap_or(0);
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        Some(
            (0..fh)
                .flat_map(|fy| (0..fw).map(move |fx| (fx, fy)))
                .map(|(fx, fy)| {
                    if w.seen(fx, fy).unwrap_or(0) & bit != 0 {
                        fog::VISIBLE
                    } else if w.seen2(fx, fy).unwrap_or(0) & bit != 0 {
                        fog::EXPLORED
                    } else {
                        fog::UNSEEN
                    }
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests;
