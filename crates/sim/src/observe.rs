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
    /// Each player slot's leader record, in slot order: what the HUD's bar
    /// prints (item 4198, `docs/HUD.md` §4).
    pub leaders: Vec<LeaderScene>,
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

/// **One player's leader record, as the original's resource bar and
/// population box read it** (item 4198, `docs/HUD.md` §4). A copy: the
/// door adds no state and no rule. The fields are the ones
/// `IFaceResources::draw_resource@00818840` and
/// `IFaceMainBase::draw_pop_cap@0080ff90` read off `LeaderData`, at the
/// scale the record stores them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderScene {
    /// The player slot, the record's `who`.
    pub who: Player,
    /// `LeaderData::tribe`, an index into the tech tree's tribes.
    pub tribe: usize,
    /// That tribe's `<TRIBE name>` from the install's tables, empty when
    /// unloaded: a copy of the table's entry, so a client needs no tables
    /// of its own to name the nation.
    pub nation: String,
    /// `LeaderDataEncrypt::ages` (`+0xdc`), the dump's `ages_get()`: the
    /// ages gained past the first, 0 in the Ancient age.
    pub ages: i32,
    /// `LeaderDataEncrypt::bucket` (`+0x0`), per good in the record's
    /// order (food, timber, wealth, knowledge, metal, oil): the stockpile,
    /// in whole units, the number `draw_resource` writes.
    pub bucket: [i32; 6],
    /// `LeaderDataEncrypt::income` (`+0x94`), in sixteenths: the rate
    /// `draw_resource` writes beside the stockpile as `(income + 8) / 16`.
    pub income: [i32; 6],
    /// `LeaderDataEncrypt::over_cap` (`+0x4c`) non-zero: the good is at its
    /// commerce cap, which `draw_resource` flashes.
    pub over_cap: [bool; 6],
    /// `LeaderData::control` (`+0x940`): the population occupied, the
    /// first number of `draw_pop_cap`'s `n/cap`.
    pub control: i32,
    /// `LeaderData::pop_cap` (`+0x7e4`): the cap it is measured against.
    pub pop_cap: i32,
}

/// One unit on the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitScene {
    pub id: ObjId,
    pub owner: Player,
    /// An index into the unit types; `None` for the bare object.
    pub ty: Option<usize>,
    /// The type's `TypeIndex` (`ptype +4`), −1 when the loader named none:
    /// the key `SelectGroup::add@00716c10` sorts a selection by, and what
    /// `ObjectData::is_worker` and `UnitData::is_merchant` test (item
    /// 4180, `docs/CLIENT_INPUT.md` §7).
    pub type_index: i32,
    /// `UnitData +0xaa` and `+0xab`, [`crate::Unit::form`] and
    /// [`crate::Unit::form_width`]: what `GroupData::get_form` and
    /// `get_form_mod_option` read when a right press mints a group's move
    /// (`Console::down@007cd8f0`; item 4180).
    pub form: i8,
    pub form_width: i8,
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
    /// `WallData::is_started` (`flags & 2`, [`crate::cities::building::Building::started`]):
    /// `Wall::start` has run. `WallOut::draw_overlays@00641600` draws a
    /// building's scaffold only while it is started and not active
    /// (`docs/RENDERER.md` §3.5; item 4061).
    pub started: bool,
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
    /// A farm's record (`BuildData +0x78 >= 0`), which
    /// `FarmsOut::draw@008d75f0` draws as a wheat field; `None` for every
    /// other building (item 4048).
    pub field: Option<FieldScene>,
    /// What a right press on it asks of a gather building (item 4179,
    /// `docs/CLIENT_INPUT.md` §6): for a gather type (`build_flags & 0x40`,
    /// `BuildTypeData::is_gather_type`), the gatherers its chain holds
    /// (`BuildData::gather_down`) and its slot count (`gather_max`, `None`
    /// uncapped); `None` for every other building.
    pub gather: Option<(i32, Option<i32>)>,
}

/// A farm's `FarmStruct`, as `FarmsOut::draw@008d75f0` reads it
/// (`docs/RENDERER.md` §3.5; item 4048).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldScene {
    /// The footprint of its type, `BuildTypeData +0x234` and `+0x238`,
    /// which `FarmsOut::draw` snaps the field's corner by.
    pub x_size: i32,
    pub y_size: i32,
    /// `farm_type`: [`crate::cities::farms::ANIMAL_FARM`] is the pasture,
    /// which the field pass skips.
    pub farm_type: u8,
    /// `terrain_height[col][row]`, `heights[col * 5 + row]`
    /// ([`crate::cities::farms::Farm::heights`]).
    pub heights: [i32; 25],
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
    /// The round's graphic piece, `AmmoData::gpiece`, which
    /// `AmmoOut::draw@0067ac10` draws (`docs/RENDERER.md` §3.13, item
    /// 4175): an `<AMMO>` of `effects_graphics.xml` at
    /// [`crate::combat::FIRST_AMMO_PIECE`] plus its index, or a falling
    /// aircraft's own piece; `-1` where the shot's piece is not read.
    pub gpiece: i32,
}

/// One dead unit's fading record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeathScene {
    pub who: i32,
    pub o: i32,
    pub first_frame: i64,
    /// The `UnitAnim` slot each record plays, `DeathObj +0x8`.
    pub cur_anim: i32,
    /// **One piece per figure whose record still stands this frame**
    /// (`docs/RENDERER.md` §3.13, item 4175): `DeathObj +0x20`, each guy's
    /// own `gpiece` at the death (`DeathObj::init@008d4c60`'s vslot
    /// `+0x178`), which `DeathObjOut::draw@008d4090` sends through
    /// `GraphicPieces::draw` on slot [`Self::cur_anim`]. A record whose
    /// life has run out, or that the pool evicted, is gone from the list.
    pub pieces: Vec<i32>,
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
    /// The corner grid the tile heights are taken from
    /// (`TerrainData::master_land_heights`), row-major over `width × 4 + 1`
    /// by `height × 4 + 1` corners, each single as its 32 bits — the
    /// heightfield a client draws (item 4036); empty on a world with no
    /// height table.
    pub corners: Vec<u32>,
    /// Each cell's `WData.land` and `land_sub`, row-major.
    pub land: Vec<i8>,
    pub land_sub: Vec<u8>,
    /// Each cell's `WData.flags`, row-major — the bits [`crate::world::cell`]
    /// names. A client reads [`crate::world::cell::FOREST`] off it: the
    /// cells `ObjectsOut::sort_trees@00656c70` stands trees on (item 4080).
    pub flags: Vec<u16>,
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
                    type_index: u.type_index,
                    form: u.form,
                    form_width: u.form_width,
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
                started: b.started,
                constr_time: b.constr_time,
                construct_hits: b.construct_hits,
                orig_ty: b.orig_ty,
                job_counter: b.job_counter,
                construct_time: self.construct_time_of(i),
                tribe: self.tech.get(b.founder as usize).map_or(0, |p| p.tribe),
                age: self.tech.get(b.owner as usize).map_or(0, |p| p.ages),
                // A record in the list — `farm.valid` is `Farms::add`'s,
                // and a farm the harness stood up from a dump joins the
                // list without it.
                field: self.cities.farm_order.contains(&i).then(|| FieldScene {
                    x_size: b.ty.map_or(0, |t| self.tables.build_types[t].x_size),
                    y_size: b.ty.map_or(0, |t| self.tables.build_types[t].y_size),
                    farm_type: b.farm.farm_type,
                    heights: b.farm.heights,
                }),
                gather: b
                    .ty
                    .filter(|&t| {
                        self.tables.build_types[t].flags & crate::cities::build::flags::GATHERER
                            != 0
                    })
                    .map(|_| (b.gatherers.len() as i32, b.gather_max)),
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
                gpiece: p.gpiece,
            })
            .collect();
        let deaths = self
            .combat
            .deaths
            .iter()
            .map(|d| {
                let age = self.frame - d.first_frame;
                let n = usize::from(d.figures.n).min(crate::combat::Figures::MAX);
                DeathScene {
                    who: d.who,
                    o: d.o,
                    first_frame: d.first_frame,
                    cur_anim: d.cur_anim,
                    pieces: (0..n)
                        .filter(|&k| age < d.figures.life[k])
                        .map(|k| d.figures.gpiece[k])
                        .collect(),
                }
            })
            .collect();
        Scene {
            frame: self.frame,
            units,
            buildings,
            projectiles,
            deaths,
            leaders: self.observe_leaders(),
            fog: viewer.and_then(|p| self.fog_plane(p)),
            terrain_version: self.world.terrain_version(),
            territory_version: self.world.territory_version(),
        }
    }

    /// Every player slot's [`LeaderScene`]: a slot is one the tech, the
    /// ledgers and the muster all hold.
    fn observe_leaders(&self) -> Vec<LeaderScene> {
        let slots = self
            .tech
            .len()
            .min(self.economy.ledgers.len())
            .min(self.roster.muster.len());
        (0..slots)
            .map(|who| {
                let (t, l, m) = (
                    &self.tech[who],
                    &self.economy.ledgers[who],
                    &self.roster.muster[who],
                );
                LeaderScene {
                    who: who as Player,
                    tribe: t.tribe,
                    nation: self
                        .tables
                        .tech_tree
                        .tribes
                        .get(t.tribe)
                        .map_or_else(String::new, |n| n.name.clone()),
                    ages: t.ages,
                    bucket: l.bucket,
                    income: l.income,
                    over_cap: l.over_cap.map(|o| o != crate::economy::OverCap::Under),
                    control: m.control,
                    pop_cap: m.cap,
                }
            })
            .collect()
    }

    /// The terrain grids as they stand.
    pub fn observe_terrain(&self) -> TerrainScene {
        let w = &self.world;
        let (width, height) = (w.width(), w.height());
        let cells = (0..height).flat_map(|y| (0..width).map(move |x| crate::Cell::new(x, y)));
        let (land, land_sub): (Vec<i8>, Vec<u8>) = cells
            .clone()
            .map(|c| {
                let d = w.cell_data(c);
                (d.land, d.land_sub)
            })
            .unzip();
        let flags = cells.map(|c| w.cell_data(c).flags).collect();
        TerrainScene {
            width,
            height,
            tile_z: w.tile_z_grid().to_vec(),
            corners: w.corner_singles().iter().map(|c| c.bits()).collect(),
            land,
            land_sub,
            flags,
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
