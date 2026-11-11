//! What the client draws, in world space: the free runner's game
//! (`rondata::free`), the heightfield it reports, and each drawn figure's
//! decoded mesh at its draw-list anchor and facing (`docs/RENDERER.md`
//! §3.6). Nothing here touches the GPU; `gpu` takes the triangles.

use render::bh3;
use render::draw::{Drawn, MODELS_AT_THE_POINT, Placed, PlacedBuilding, Pose, SpriteAt, sprite_at};
use render::figure::{apply, guy_placement, placement, posed, rest_pose};
use render::terrain::{Heightfield, TerrainMesh};
use rondata::artdata::{BuildLook, build_art, resolve_art, unit_art};
use rondata::free::{FreeRunner, placed, placed_buildings};
use rondata::{Install, TribeDef};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The pair the client opens by default: the Aztecs at a held Modern age,
/// Great Lakes (`rondata::diff::aztec_modern_pair`; RENDERER §5.4's M is
/// counted on it).
pub const START_DUMP: &str = "gamelog-run1428-greatlakes-aztec-modern-start.txt";
pub const LONG_DUMP: &str = "gamelog-run1429-greatlakes-aztec-modern-trace.txt";
pub const LONG_TRACE: &str = "rontrace-run1429.log";

/// Where the captures sit: `$RON_GAMELOG_DIR`, else the bottle's `Logs\`,
/// as the test environment finds them.
pub fn gamelog_dir() -> PathBuf {
    std::env::var_os("RON_GAMELOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            Path::new(&home).join("ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
        })
}

/// **`guy_scale`** (`?guy_scale@@3MA`, `0x00c06244` by the linker map),
/// read off run1441's packet at logger frame 3294: 4.8. `GuyOut::draw`
/// multiplies a figure's piece scale (`RData +0x88`, the `<UNIT>`'s
/// `scale=`) by it unless `RData +0x92` has bit 2, which only a building's
/// piece carries; run1559b's packet reads both for the drawn pieces
/// (`docs/RENDERER.md` §3.6, item 4065).
pub const GUY_SCALE: f32 = 4.8;

/// A model at rest: its vertices in model space and its triangles, and
/// the decoded model and animations a figure is posed from.
pub struct Mesh {
    pub rest: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 3]>,
    /// The `<UNIT>`'s `scale=` times [`GUY_SCALE`]; 1 for a building.
    pub scale: f32,
    pub model: bh3::Model,
    /// Each `UnitAnim` slot's decoded `.BHa`, where the entry names one
    /// that resolves and decodes; empty for a building.
    pub anims: BTreeMap<i8, std::sync::Arc<render::bha::Anim>>,
}

/// **The milliseconds a figure is drawn at** (item 4067): `GuyOut::draw
/// @005dc490` gives `cur_time · 200 / 3` — fifteen game frames a second —
/// plus the frame's `graphic_dt`, clamped to the animation's own length
/// (`AnimMgr::times`, the root's last key time). The client draws at the
/// frame's start, `graphic_dt` 0.
pub fn anim_ms(cur_time: u32, anim: &render::bha::Anim) -> u32 {
    let len = anim
        .played()
        .first()
        .and_then(|n| n.times().last().copied())
        .map_or(0, u32::from);
    (cur_time.saturating_mul(200) / 3).min(len)
}

/// A unit piece's `model=`, `scale=` and its slots' `.BHa` files.
type UnitModel = (String, String, Vec<(i8, String)>);

/// One coloured triangle soup, world space.
#[derive(Default)]
pub struct Soup {
    pub positions: Vec<[f32; 3]>,
    pub colours: Vec<[f32; 3]>,
}

/// The game and the art, stepped and drawn from.
pub struct World {
    pub runner: FreeRunner,
    look: BuildLook,
    tribes: Vec<TribeDef>,
    /// Each piece's mesh, `None` where it resolves to nothing that decodes.
    meshes: BTreeMap<i32, Option<std::sync::Arc<Mesh>>>,
    unit_models: BTreeMap<i32, UnitModel>,
    /// Each unit piece's `<UNIT>` `texture=` (item 4099), where it names one.
    unit_textures: BTreeMap<i32, String>,
    /// Each unit piece's texture, decoded once with its levels; `None`
    /// where it does not resolve or decode.
    figure_textures: BTreeMap<i32, Option<std::sync::Arc<render::silhouette::Mips>>>,
    /// Draw the figures textured through their texcoords (item 4099); off,
    /// each owner's debug tint ([`tint`]). On by default; `--tint` turns
    /// it off.
    pub textured: bool,
    root: PathBuf,
    install: Install,
    /// Pose each figure at its slot and clock (item 4067). On by default
    /// since item 4069 drew the original's facing (`guy_placement`): posed,
    /// candidate C reads 6 of 23, at rest 2 (`docs/RENDERER.md` §5.3);
    /// `--rest` turns it off.
    pub pose: bool,
    /// Draw each building as its `<BUILD>`'s `texture=` sprite (item 4076,
    /// `render::draw::SpriteAt`) rather than its `zbuffer` model. On by
    /// default; `--zbuffer` turns it off.
    pub sprites: bool,
    /// `Camera::fixed_zooms[6]` and `Game::game_sprite_scale`, the sprite
    /// scale's numerator (`render::draw::sprite_at`).
    pub sprite_zoom: (f32, f32),
    /// Each building piece's sprite, `None` where its `texture=` does not
    /// resolve or decode.
    sprite_images: BTreeMap<i32, Option<std::sync::Arc<SpriteImage>>>,
    terrain: (u32, TerrainMesh),
    /// Draw the map's trees (item 4080, `render::trees`). On by default;
    /// `--no-trees` turns it off.
    pub trees: bool,
    /// How candidate C reads a tree's pixel (`docs/RENDERER.md` §5.3).
    pub tree_pixels: TreePixels,
    /// How the trees' leaves are drawn in the key buffer (item 4082).
    pub leaves: Leaves,
    /// Whether a figure behind a tree or a sprite is drawn as its
    /// silhouette (item 4085).
    pub silhouettes: Silhouettes,
    /// The winding the trees' alpha-tested draw culls ([`TREE_CULL`]).
    pub tree_cull: render::silhouette::Cull,
    /// How a tracked figure leans to the ground (item 4089).
    pub tilt: Tilt,
    /// Which point a figure is drawn at (item 4095).
    pub blend: Blend,
    /// The figures' points before [`World::run_to`]'s last step.
    last: Vec<Placed>,
    /// Each unit type's `CAT` (`UnitTypeData +0x14`, an index into
    /// `rules.xml`'s `unit_cats`), by type index.
    cats: Vec<i32>,
    /// [`Tilt::Eased`]'s `GuyData::last_norm` per figure, eased once a
    /// frame from [`EASE_FROM`] on.
    eased: BTreeMap<(u8, i16, u16), [f32; 3]>,
    /// The lobby's tileset's trees: `tree_scale` and each model at rest,
    /// `None` where it does not decode; `None` whole where the tileset's
    /// trees do not read.
    tree_art: Option<TreeMeshes>,
}

/// The tileset's tree models, decoded once: `TileSetData::tree_scale` (the
/// `treescale` times [`render::trees::TREESCALE_FACTOR`]) and each model's
/// rest vertices and faces, in `graphics.txt`'s order.
pub struct TreeMeshes {
    pub scale: f32,
    pub models: Vec<Option<TreeModel>>,
    /// Each model's texcoords and the texture each face samples (item
    /// 4082, `render::trees::face_textures`), beside [`TreeMeshes::models`].
    pub leaves: Vec<Option<TreeLeaves>>,
    /// Each model's six node slots (`render::trees::slots`), beside
    /// [`TreeMeshes::models`].
    #[cfg_attr(not(test), allow(dead_code))]
    pub slots: Vec<Option<[Option<render::trees::Slot>; 6]>>,
    /// The seven textures (`rondata::artdata::TreeArt::textures`), `None`
    /// where one does not decode.
    pub textures: Vec<Option<render::tga::Image>>,
}

/// One tree model's texcoords, one a vertex, and the texture (0–6) each
/// face samples.
pub type TreeLeaves = (Vec<[f32; 2]>, Vec<usize>);

/// **How a tree's leaves are drawn** (item 4082, `docs/RENDERER.md` §3.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaves {
    /// Every triangle whole, item 4080's draw: the rejected reading
    /// (`docs/RENDERER.md` §5.3, "The leaves").
    #[cfg_attr(not(test), allow(dead_code))]
    Solid,
    /// The original's alpha test, the default: a fragment is kept where its
    /// texture's alpha is above `render_states[9]`'s `alpha_ref`, 0
    /// ([`TREE_ALPHA_REF`]), and a triangle wound against [`TREE_CULL`] is
    /// culled.
    Alpha,
}

/// **How a tracked figure leans to the ground** (item 4089,
/// `docs/RENDERER.md` §3.7 and §5.3, "The tilt"). `GuyOut::draw@005dc490`
/// tilts a guy whose type's `CAT` is Mechanized or Artillery to
/// `GuyData::last_norm`, which eases a quarter of the way to the negated
/// `WorldData::find_slope_normal` at every draw; the client replays no
/// draw clock.
/// **Which point a figure is drawn at** (item 4095, `docs/RENDERER.md`
/// §5.3, "The point"). The original draws a guy at `GuyData`'s point
/// blended from `last_*` by `graphic_percent`; its slot, clock and facing
/// are the frame's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    /// The default: the frame's own point.
    Current,
    /// The point the figure held a frame before ([`World::run_to`]'s last
    /// step keeps it): `graphic_percent` at 0.
    #[cfg_attr(not(test), allow(dead_code))]
    Last,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tilt {
    /// Every figure level: the draw before item 4089.
    #[cfg_attr(not(test), allow(dead_code))]
    Level,
    /// The default: the target normal at the figure's point on the frame,
    /// taken whole — the ease's fixed point.
    Target,
    /// The ease at one draw a frame, from [`EASE_FROM`] (the first frame
    /// takes the target whole, as the unset `last_norm` does).
    #[cfg_attr(not(test), allow(dead_code))]
    Eased,
}

/// The first frame [`Tilt::Eased`] keeps its normals from.
pub const EASE_FROM: i64 = 3200;

/// `rules.xml`'s `unit_cats` indices `GuyOut::draw` tilts: `Mech` and
/// `Artillery`.
const TILTED_CATS: [i32; 2] = [2, 3];

/// **Whether a land figure is drawn over what stands nearer** (item 4085,
/// `docs/RENDERER.md` §5.3, "The silhouettes").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Silhouettes {
    /// The figure is hidden where a tree or a sprite is nearer: the
    /// rejected reading, items 4080 and 4082's draw.
    #[cfg_attr(not(test), allow(dead_code))]
    None,
    /// The default: `GraphicPieces::init_land_unit_map@008fa4b0`'s method
    /// list 1 draws a land figure under `render_states[1]` before `[0]`,
    /// depth Greater and no depth write, after the trees are flushed. So
    /// where a tree or a building's sprite is nearer, the pixel is the
    /// figure's and the nearer depth stays. Figures and building meshes
    /// keep depth order.
    Drawn,
}

/// `render_states[9].alpha_ref` on run1559b's packet (item 4082,
/// `tools/render/tree_points.py`'s `rstate` row): `Scene::init_render_states`
/// leaves it at `RenderState::init`'s 0.
pub const TREE_ALPHA_REF: u8 = 0;

/// `render_states[9].cull_mode` on run1559b's packet, 2, `Counter
/// Clockwise` (item 4082; `render::silhouette::Cull`).
pub const TREE_CULL: render::silhouette::Cull = render::silhouette::Cull::CounterClockwise;

/// One tree model at rest: its vertices and its faces.
pub type TreeModel = (Vec<[f32; 3]>, Vec<[u16; 3]>);

/// **How a tree's pixel reads in candidate C's key buffer** (item 4080,
/// `docs/RENDERER.md` §5.3, the rule written before its count).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreePixels {
    /// The rule: a tree occludes what stands behind it, and its pixel is
    /// no object's — the ground's, as the terrain's is. A tree is not a
    /// draw-list entry and is never scored.
    Ground,
    /// The rejected reading: a tree's pixel is the scene's, an object of
    /// ours in every window it falls in, though never scored itself.
    #[cfg_attr(not(test), allow(dead_code))]
    Scene,
}

/// **The `who` a tree's drawing carries** (item 4080): no owner's, so no
/// draw-list entry's; its `o` is the tree's index in [`World::tree_list`].
pub const TREE_WHO: u8 = u8::MAX;

/// **The lobby's tileset**, as `rondata`'s M test reads it (item 4037):
/// the start dump's `map.cpp` 7805 seed, stepped back through the Great
/// Lakes style's `TILECHANCE`s (`mapgen::tileset_before`). The style file
/// is the pair's, named by [`START_DUMP`].
pub fn lobby_tileset(root: &Path, start: &Path) -> Option<String> {
    let head = rondata::mapgen::start_head(start).ok()??;
    let seed = rondata::mapgen::start_grid(&head).at("map.cpp", 7805)?.2;
    let chances = rondata::mapgen::tile_chances(root, "GREATLAKES.xml")?;
    rondata::mapgen::tileset_before(&chances, seed)
}

/// A building's `texture=` decoded (`render::tga`), with its
/// `(spritexoff, spriteyoff)`.
pub struct SpriteImage {
    pub image: render::tga::Image,
    pub offset: (i32, i32),
    /// Each pixel's alpha at half or more: the pixels the sprite covers.
    /// The install's building sprites are nearly two-level (item 4076).
    pub covered: Vec<bool>,
}

/// One building drawn as its sprite: the draw-list entry's owner, object
/// and piece, where the image lands, and the depth the list orders it by.
pub struct SpriteDrawing {
    pub who: u8,
    pub o: i16,
    pub piece: i32,
    pub at: SpriteAt,
    pub depth: f32,
    pub image: std::sync::Arc<SpriteImage>,
}

impl World {
    /// Open the install and the pair's captures, and stand the game up at
    /// frame zero.
    pub fn open(install_root: &Path, logs: &Path) -> Result<World, String> {
        let install = Install::new(install_root);
        if !install.looks_valid() {
            return Err(format!("no install at {}", install_root.display()));
        }
        let loaded = rondata::load::load(&install).map_err(|e| e.to_string())?;
        let (long, start) = (logs.join(LONG_DUMP), logs.join(START_DUMP));
        let trace = rondata::trace::Trace::read(&logs.join(LONG_TRACE))
            .map_err(|e| format!("{LONG_TRACE}: {e}"))?;
        let runner = FreeRunner::from_captures(
            &loaded,
            &long,
            &[start.as_path()],
            trace.as_ref(),
            sim::Tuning::RON,
        )
        .map_err(|e| format!("{}: {e}", long.display()))?;
        let rules = install.rules().map_err(|e| e.to_string())?;
        let tribes = install.tribe_defs(&rules).map_err(|e| e.to_string())?;
        let graphs: Vec<String> = install
            .units()
            .map_err(|e| e.to_string())?
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        let mut unit_models = BTreeMap::new();
        let mut unit_textures = BTreeMap::new();
        for e in unit_art(&install, &graphs) {
            for p in &e.pieces {
                if !e.texture.is_empty() {
                    unit_textures.entry(*p).or_insert_with(|| e.texture.clone());
                }
                unit_models.entry(*p).or_insert_with(|| {
                    let files = e
                        .anims
                        .iter()
                        .filter_map(|(slot, _, f)| Some((*slot, f.clone()?)))
                        .collect();
                    (e.model.clone(), e.scale.clone(), files)
                });
            }
        }
        let tree_art = lobby_tileset(install_root, &start)
            .and_then(|t| rondata::artdata::tree_art(&install, &t).ok())
            .map(|art| {
                let root = install.root().to_path_buf();
                TreeMeshes {
                    scale: art.treescale * render::trees::TREESCALE_FACTOR,
                    models: art
                        .models
                        .iter()
                        .map(|m| {
                            let bytes = std::fs::read(resolve_art(&root, m)?).ok()?;
                            let model = bh3::decode(&bytes).ok()?;
                            Some((rest_pose(&model), model.faces.clone()))
                        })
                        .collect(),
                    leaves: art
                        .models
                        .iter()
                        .map(|m| {
                            let bytes = std::fs::read(resolve_art(&root, m)?).ok()?;
                            let model = bh3::decode(&bytes).ok()?;
                            let faces = render::trees::face_textures(&model);
                            Some((model.texcoords, faces))
                        })
                        .collect(),
                    slots: art
                        .models
                        .iter()
                        .map(|m| {
                            let bytes = std::fs::read(resolve_art(&root, m)?).ok()?;
                            Some(render::trees::slots(&bh3::decode(&bytes).ok()?))
                        })
                        .collect(),
                    textures: art
                        .textures
                        .iter()
                        .map(|t| {
                            let bytes = std::fs::read(resolve_art(&root, t)?).ok()?;
                            render::tga::decode(&bytes).ok()
                        })
                        .collect(),
                }
            });
        let cats = loaded.unit_types.iter().map(|t| t.cols.cat).collect();
        Ok(World {
            look: BuildLook::new(&install).with_types(&loaded.build_types),
            tribes,
            meshes: BTreeMap::new(),
            unit_models,
            unit_textures,
            figure_textures: BTreeMap::new(),
            textured: true,
            root: install.root().to_path_buf(),
            install,
            pose: true,
            sprites: true,
            sprite_zoom: crate::PINNED_SPRITE_ZOOM,
            sprite_images: BTreeMap::new(),
            runner,
            terrain: (u32::MAX, TerrainMesh::default()),
            trees: true,
            tree_pixels: TreePixels::Ground,
            leaves: Leaves::Alpha,
            silhouettes: Silhouettes::Drawn,
            tree_cull: TREE_CULL,
            tilt: Tilt::Target,
            blend: Blend::Current,
            last: Vec::new(),
            cats,
            eased: BTreeMap::new(),
            tree_art,
        })
    }

    /// Step the game to `frame` (the dump's `FRAME frame`).
    pub fn run_to(&mut self, frame: i64) {
        while self.runner.frame() < frame {
            if self.runner.frame() + 1 == frame {
                self.last = placed(&self.runner.scene(None));
            }
            self.runner.step();
            if self.runner.frame() >= EASE_FROM {
                self.ease();
            }
        }
    }

    /// Whether a unit of type `ty` leans to the ground.
    fn tilts(&self, ty: Option<usize>) -> bool {
        ty.and_then(|t| self.cats.get(t))
            .is_some_and(|c| TILTED_CATS.contains(c))
    }

    /// One draw of `GuyData::find_slope_normal` for every tracked figure.
    fn ease(&mut self) {
        let scene = self.runner.scene(None);
        let mut eased = std::mem::take(&mut self.eased);
        for u in scene.units.iter().filter(|u| self.tilts(u.ty)) {
            for (k, f) in u.figures.iter().enumerate() {
                let (_, n) = self.slope_normal(f.pos.x, f.pos.y);
                let last = eased
                    .entry((u.id.who, u.id.o, k as u16))
                    .or_insert(render::slope::UNSET);
                render::slope::ease(last, render::slope::target(n));
            }
        }
        self.eased = eased;
    }

    /// The normal a figure is drawn tilted to under [`World::tilt`], or
    /// `None` when it is drawn level.
    fn tilt_normal(&self, key: (u8, i16, u16), at: [i32; 3]) -> Option<[f32; 3]> {
        let n = match self.tilt {
            Tilt::Level => return None,
            Tilt::Target => {
                let mut n = render::slope::UNSET;
                let (_, w) = self.slope_normal(at[0], at[1]);
                render::slope::ease(&mut n, render::slope::target(w));
                n
            }
            Tilt::Eased => *self.eased.get(&key)?,
        };
        (n[2] < render::slope::LEVEL_Z).then_some(n)
    }

    /// The heightfield's triangles, rebuilt when the door's terrain version
    /// moves.
    pub fn terrain(&mut self) -> &TerrainMesh {
        let t = self.runner.terrain();
        if t.version != self.terrain.0 || self.terrain.1.positions.is_empty() {
            let field = Heightfield {
                width: t.width,
                height: t.height,
                corners: &t.corners,
            };
            let mountains: Vec<(i32, i32)> =
                t.mountains.iter().flatten().map(|p| (p.x, p.y)).collect();
            self.terrain = (t.version, TerrainMesh::build(&field, &t.land, &mountains));
        }
        &self.terrain.1
    }

    /// `WorldData::find_slope_normal@006b5130` at `(x, y)` on our terrain
    /// as it stands now: the three heights it reads and its normal
    /// (`render::slope`, item 4089).
    pub fn slope_normal(&self, x: i32, y: i32) -> ([i32; 3], [f32; 3]) {
        let t = self.runner.terrain();
        let field = Heightfield {
            width: t.width,
            height: t.height,
            corners: &t.corners,
        };
        render::slope::world_normal(&field, x, y)
    }

    /// The tileset's tree models, where they read.
    #[cfg(test)]
    pub fn tree_meshes(&self) -> Option<&TreeMeshes> {
        self.tree_art.as_ref()
    }

    /// **Every tree of the map as it stands now** (item 4080): each forest
    /// cell of the door's `flags`, row-major, five trees a cell, placed and
    /// chosen as `ObjectsOut::sort_trees@00656c70` does
    /// (`render::trees::stand`). The original stands only the visible
    /// cells; a tree off screen draws no pixel, so the client stands them
    /// all.
    pub fn tree_list(&self) -> Vec<render::trees::Tree> {
        let Some(art) = &self.tree_art else {
            return Vec::new();
        };
        let t = self.runner.terrain();
        let field = Heightfield {
            width: t.width,
            height: t.height,
            corners: &t.corners,
        };
        let forest = |x: i32, y: i32| {
            x >= 0
                && y >= 0
                && x < t.width
                && y < t.height
                && t.flags[(y * t.width + x) as usize] & sim::world::cell::FOREST != 0
        };
        let cells = (0..t.height).flat_map(|y| (0..t.width).map(move |x| (x, y)));
        render::trees::stand(&field, forest, cells, art.models.len())
    }

    /// The trees as drawings: each model at rest under
    /// `render::trees::tree_placement`, [`TREE_WHO`] its owner. Empty with
    /// [`World::trees`] off.
    pub fn tree_drawings(&self) -> Vec<Drawing> {
        let Some(art) = self.tree_art.as_ref().filter(|_| self.trees) else {
            return Vec::new();
        };
        self.tree_list()
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let (rest, faces) = art.models.get(t.model)?.as_ref()?;
                let m = render::trees::tree_placement(t.at, art.scale);
                Some(Drawing {
                    who: TREE_WHO,
                    o: i as i16,
                    piece: t.model as i32,
                    building: false,
                    tris: faces
                        .iter()
                        .map(|f| f.map(|k| apply(&m, rest[k as usize])))
                        .collect(),
                    uvs: Vec::new(),
                })
            })
            .collect()
    }

    /// **The trees' alpha tests**, one beside each of
    /// [`World::tree_drawings`]: each face's texcoords and texture, and the
    /// reference `render_states[9]` holds ([`TREE_ALPHA_REF`]). `None` for a
    /// tree whose texcoords do not read, which is then drawn whole.
    pub fn tree_cuts(&self) -> Vec<Option<render::silhouette::Cut<'_>>> {
        let Some(art) = self.tree_art.as_ref().filter(|_| self.trees) else {
            return Vec::new();
        };
        self.tree_list()
            .iter()
            .filter_map(|t| {
                let (_, faces) = art.models.get(t.model)?.as_ref()?;
                let leaves = art.leaves.get(t.model).and_then(|l| l.as_ref());
                Some(leaves.and_then(|(uvs, textures)| {
                    let uvs = faces
                        .iter()
                        .map(|f| {
                            let at = |k: u16| uvs.get(k as usize).copied();
                            Some([at(f[0])?, at(f[1])?, at(f[2])?])
                        })
                        .collect::<Option<Vec<_>>>()?;
                    Some(render::silhouette::Cut {
                        uvs,
                        image: textures.clone(),
                        images: &art.textures,
                        alpha_ref: TREE_ALPHA_REF,
                        cull: self.tree_cull,
                    })
                }))
            })
            .collect()
    }

    /// The trees into the window's soup, flat green (no texture is bound,
    /// 4038a).
    pub fn tree_soup(&self, soup: &mut Soup) {
        for d in self.tree_drawings() {
            for tri in d.tris {
                let lit = shade(tri);
                for p in tri {
                    soup.positions.push(p);
                    soup.colours.push([0.25 * lit, 0.45 * lit, 0.2 * lit]);
                }
            }
        }
    }

    /// This frame's figures and buildings as the observer sees them: the
    /// placed input and the draw list.
    pub fn draw_list(
        &self,
        camera: &render::Camera,
        screen: &render::Screen,
        size: (i32, i32),
    ) -> (Vec<Placed>, Vec<PlacedBuilding>, Vec<Drawn>) {
        let scene = self.runner.scene(None);
        let mut figures = placed(&scene);
        if self.blend == Blend::Last {
            for f in figures.iter_mut() {
                if let Some(p) = self
                    .last
                    .iter()
                    .find(|p| (p.who, p.o, p.figure) == (f.who, f.o, f.figure))
                {
                    (f.x, f.y, f.z) = (p.x, p.y, p.z);
                }
            }
        }
        let buildings = placed_buildings(&scene, &self.look, &self.tribes);
        let list = render::draw::draw_list(
            &figures,
            &buildings,
            self.look.scaffold_timing(),
            camera,
            screen,
            Some(size),
            None,
        );
        (figures, buildings, list)
    }

    /// A unit piece's slots and the `.BHa` file each names, as the mesh
    /// loads them (item 4092).
    #[cfg(test)]
    pub(crate) fn unit_slot_files(&self, piece: i32) -> Option<Vec<(i8, String)>> {
        self.unit_models.get(&piece).map(|(_, _, f)| f.clone())
    }

    pub(crate) fn mesh(&mut self, piece: i32, building: bool) -> Option<std::sync::Arc<Mesh>> {
        if let Some(m) = self.meshes.get(&piece) {
            return m.clone();
        }
        let (model, scale, files) = if building {
            let set = std::iter::once(piece).collect();
            let art = build_art(&self.install, &set);
            let z = art.get(&piece)?.as_ref().ok()?.zbuffer.clone();
            (z.unwrap_or_default(), 1.0, Vec::new())
        } else {
            let (m, s, f) = self.unit_models.get(&piece)?.clone();
            (m, s.parse::<f32>().unwrap_or(1.0) * GUY_SCALE, f)
        };
        let root = self.root.clone();
        let read = |rel: &str| resolve_art(&root, rel).and_then(|p| std::fs::read(p).ok());
        let anims: BTreeMap<i8, std::sync::Arc<render::bha::Anim>> = files
            .iter()
            .filter_map(|(slot, f)| {
                let a = render::bha::decode(&read(f)?).ok()?;
                Some((*slot, std::sync::Arc::new(a)))
            })
            .collect();
        let decoded = read(&model).and_then(|b| bh3::decode(&b).ok()).map(|m| {
            std::sync::Arc::new(Mesh {
                rest: rest_pose(&m),
                faces: m.faces.clone(),
                scale,
                model: m,
                anims,
            })
        });
        self.meshes.insert(piece, decoded.clone());
        decoded
    }

    /// A unit piece's `texture=`, decoded once with its levels (item 4099).
    pub fn figure_texture(
        &mut self,
        piece: i32,
    ) -> Option<std::sync::Arc<render::silhouette::Mips>> {
        if let Some(t) = self.figure_textures.get(&piece) {
            return t.clone();
        }
        let decoded = self
            .unit_textures
            .get(&piece)
            .and_then(|t| resolve_art(&self.root, t))
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| render::tga::decode(&b).ok())
            .map(|i| std::sync::Arc::new(render::silhouette::Mips::new(i)));
        self.figure_textures.insert(piece, decoded.clone());
        decoded
    }

    /// A unit piece's `texture=` as the XML writes it.
    #[cfg(test)]
    pub fn figure_texture_name(&self, piece: i32) -> Option<&str> {
        self.unit_textures.get(&piece).map(String::as_str)
    }

    /// A building piece's sprite, decoded once.
    pub fn sprite_image(&mut self, piece: i32) -> Option<std::sync::Arc<SpriteImage>> {
        if let Some(s) = self.sprite_images.get(&piece) {
            return s.clone();
        }
        let set = std::iter::once(piece).collect();
        let art = build_art(&self.install, &set);
        let decoded = art.get(&piece).and_then(|a| a.as_ref().ok()).and_then(|a| {
            let bytes = std::fs::read(resolve_art(&self.root, &a.texture)?).ok()?;
            let image = render::tga::decode(&bytes).ok()?;
            let covered = image.rgba.chunks_exact(4).map(|p| p[3] >= 128).collect();
            Some(std::sync::Arc::new(SpriteImage {
                image,
                offset: a.sprite_offset,
                covered,
            }))
        });
        self.sprite_images.insert(piece, decoded.clone());
        decoded
    }

    /// Whether a building's entry is drawn as its sprite: sprites on, not a
    /// type `WallOut::draw` draws as a model at the point, and a sprite
    /// that decodes.
    fn sprite_drawn(&mut self, piece: i32, type_index: i32) -> bool {
        self.sprites
            && !MODELS_AT_THE_POINT.contains(&type_index)
            && self.sprite_image(piece).is_some()
    }

    /// Every building the list draws as its sprite, at `zoom_distance`
    /// (item 4076): the image at `render::draw::sprite_at` of the entry's
    /// anchor, at the entry's depth.
    pub fn sprite_drawings(
        &mut self,
        buildings: &[PlacedBuilding],
        list: &[Drawn],
        zoom_distance: f32,
    ) -> Vec<SpriteDrawing> {
        let at_building: BTreeMap<(u8, i16), &PlacedBuilding> =
            buildings.iter().map(|b| ((b.who, b.o), b)).collect();
        let mut out = Vec::new();
        for d in list {
            let Pose::Building { .. } = d.pose else {
                continue;
            };
            let Some(b) = at_building.get(&(d.who, d.o)) else {
                continue;
            };
            if !self.sprite_drawn(d.piece, b.type_index) {
                continue;
            }
            let Some(image) = self.sprite_image(d.piece) else {
                continue;
            };
            let at = sprite_at(
                d.anchor,
                image.offset,
                self.sprite_zoom.0,
                self.sprite_zoom.1,
                zoom_distance,
            );
            out.push(SpriteDrawing {
                who: d.who,
                o: d.o,
                piece: d.piece,
                at,
                depth: d.depth,
                image,
            });
        }
        out
    }

    /// Every drawn figure's and building's mesh, placed in world space: one
    /// entry a draw-list entry the client draws, its owner, whether it is a
    /// building, and its triangles.
    pub fn objects(
        &mut self,
        figures: &[Placed],
        buildings: &[PlacedBuilding],
        list: &[Drawn],
    ) -> Vec<Drawing> {
        let at_figure: BTreeMap<(u8, i16, u16), &Placed> = figures
            .iter()
            .map(|p| ((p.who, p.o, p.figure), p))
            .collect();
        let at_building: BTreeMap<(u8, i16), &PlacedBuilding> =
            buildings.iter().map(|b| ((b.who, b.o), b)).collect();
        let tilted: std::collections::BTreeSet<(u8, i16)> = if self.tilt == Tilt::Level {
            Default::default()
        } else {
            self.runner
                .scene(None)
                .units
                .iter()
                .filter(|u| self.tilts(u.ty))
                .map(|u| (u.id.who, u.id.o))
                .collect()
        };
        let mut out = Vec::new();
        for d in list {
            let mut figure_key = None;
            let (point, facing, building, clock) = match d.pose {
                Pose::Figure { figure, facing, .. } => {
                    let Some(p) = at_figure.get(&(d.who, d.o, figure)) else {
                        continue;
                    };
                    figure_key = Some((d.who, d.o, figure));
                    ([p.x, p.y, p.z], facing, false, Some((p.slot, p.frame)))
                }
                Pose::Building { .. } => {
                    let Some(b) = at_building.get(&(d.who, d.o)) else {
                        continue;
                    };
                    if self.sprite_drawn(d.piece, b.type_index) {
                        continue;
                    }
                    let (x, y, z) = b.anchor_point();
                    ([x, y, z], 0, true, None)
                }
                // The wheat field is a mesh `FarmsOut::draw` builds, not a
                // piece; the client does not build it yet (item 4048).
                Pose::Field { .. } => continue,
                // The foundation is a `<CONSTRUCTION>` sprite and the
                // scaffold a `<SCAFFOLD>` mesh by element, neither a
                // building piece; the client draws neither yet (item 4061).
                Pose::Foundation { .. } | Pose::Scaffold { .. } => continue,
            };
            let Some(mesh) = self.mesh(d.piece, building) else {
                continue;
            };
            let m = if building {
                placement(point, facing, mesh.scale)
            } else {
                let drawn = sim::data::pivot::release_rotation(sim::movement::Angle(facing as i32));
                let lean = figure_key
                    .filter(|k| tilted.contains(&(k.0, k.1)))
                    .and_then(|k| self.tilt_normal(k, point));
                match lean {
                    // `GuyOut::draw`'s `misc_rot`, composed before the
                    // facing (`render::slope`).
                    Some(n) => render::slope::tilted_placement(
                        point,
                        drawn,
                        mesh.scale,
                        Some(render::slope::misc_rot(n, drawn)),
                    ),
                    None => guy_placement(point, drawn, mesh.scale),
                }
            };
            let pose = clock
                .filter(|_| self.pose)
                .and_then(|(slot, t)| mesh.anims.get(&slot).map(|a| (a, t)))
                .map(|(a, t)| posed(&mesh.model, a, anim_ms(t, a)));
            let verts = pose.as_ref().unwrap_or(&mesh.rest);
            let tris = mesh
                .faces
                .iter()
                .map(|f| f.map(|i| apply(&m, verts[i as usize])))
                .collect();
            // Each corner's `id 4` texcoord, one a vertex (item 4099).
            let uvs = if building || mesh.model.texcoords.len() != mesh.model.verts.len() {
                Vec::new()
            } else {
                mesh.faces
                    .iter()
                    .map(|f| f.map(|i| mesh.model.texcoords[i as usize]))
                    .collect()
            };
            out.push(Drawing {
                who: d.who,
                o: d.o,
                piece: d.piece,
                building,
                tris,
                uvs,
            });
        }
        out
    }

    /// Every drawn figure's and building's mesh, placed: the figures
    /// textured through their texcoords where [`World::textured`] is on and
    /// the piece's `texture=` decodes (item 4099), one mesh a drawing; the
    /// rest a soup flat-shaded in the owner's debug tint.
    pub fn figure_meshes(
        &mut self,
        figures: &[Placed],
        buildings: &[PlacedBuilding],
        list: &[Drawn],
    ) -> (Soup, Vec<TexturedMesh>) {
        let mut soup = Soup::default();
        let mut textured = Vec::new();
        for d in self.objects(figures, buildings, list) {
            let mips = (self.textured && !d.uvs.is_empty())
                .then(|| self.figure_texture(d.piece))
                .flatten();
            if let Some(mips) = mips {
                textured.push(TexturedMesh {
                    piece: d.piece,
                    positions: d.tris.iter().flatten().copied().collect(),
                    uvs: d.uvs.iter().flatten().copied().collect(),
                    mips,
                });
                continue;
            }
            let tint = tint(d.who, d.building);
            for tri in d.tris {
                let lit = shade(tri);
                for p in tri {
                    soup.positions.push(p);
                    soup.colours.push(tint.map(|v| v * lit));
                }
            }
        }
        (soup, textured)
    }
}

/// One figure drawn textured (item 4099): its triangles' corners in world
/// space, three a triangle, each corner's texcoord, and the piece's texture
/// with its levels — what `client::gpu`'s textured pipeline draws, unlit,
/// and `render::silhouette::paint` samples.
pub struct TexturedMesh {
    pub piece: i32,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub mips: std::sync::Arc<render::silhouette::Mips>,
}

/// One drawn object, placed: a draw-list entry's owner, object and piece,
/// and its mesh's triangles in world space.
pub struct Drawing {
    pub who: u8,
    pub o: i16,
    pub piece: i32,
    pub building: bool,
    pub tris: Vec<[[f32; 3]; 3]>,
    /// Each triangle's texcoords, as `tris` (item 4099); empty for a
    /// building's model.
    pub uvs: Vec<[[f32; 2]; 3]>,
}

/// The debug tint an owner's meshes take: untextured, not a team colour
/// (item 4038's).
pub fn tint(who: u8, building: bool) -> [f32; 3] {
    let base = match who {
        0 => [0.62, 0.66, 0.85],
        1 => [0.85, 0.58, 0.52],
        _ => [0.80, 0.78, 0.66],
    };
    if building {
        base.map(|v| v * 0.9)
    } else {
        base
    }
}

/// A flat shade from the triangle's normal against a light from above and
/// the north-west, either face lit.
fn shade(t: [[f32; 3]; 3]) -> f32 {
    let u = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
    let v = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l == 0.0 {
        return 0.6;
    }
    let light = [-0.4f32, -0.4, 0.82];
    let ll = (light[0] * light[0] + light[1] * light[1] + light[2] * light[2]).sqrt();
    0.45 + 0.55 * ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / (l * ll)).abs()
}

#[cfg(test)]
mod tests {
    use super::{World, gamelog_dir};
    use render::figure::{apply, guy_placement, placement};
    use std::path::PathBuf;

    /// `tools/render/tree_points.py`'s output on run1559b's packet
    /// (logger frame 3294), outside the repository: its `forest` cells and
    /// its `tree` rows `(cell, mask, slot, x, y, z, model, sx, sy)`.
    type PacketTree = ((i32, i32), u8, i32, (i32, i32, i32), usize, (i32, i32));
    type PacketForest = (std::collections::BTreeSet<(i32, i32)>, Vec<PacketTree>);
    fn packet_trees() -> Option<PacketForest> {
        let path = std::env::var_os("RON_TREE_ORACLE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-10-item-4080/run1559b-trees.txt")
            });
        let text = std::fs::read_to_string(path).ok()?;
        let (mut forest, mut trees) = (std::collections::BTreeSet::new(), Vec::new());
        for l in text.lines() {
            let w: Vec<&str> = l.split(' ').filter(|t| *t != "->").collect();
            let n = |i: usize| w[i].parse::<i32>().unwrap();
            match w[0] {
                "forest" => {
                    forest.insert((n(1), n(2)));
                }
                "tree" => trees.push((
                    (n(1), n(2)),
                    n(3) as u8,
                    n(4),
                    (n(5), n(6), n(7)),
                    n(8) as usize,
                    (n(9), n(10)),
                )),
                _ => {}
            }
        }
        Some((forest, trees))
    }

    /// **The trees in register with the packet's** (item 4080,
    /// `docs/RENDERER.md` §3.7): on the photograph's frame, the runner's
    /// forest cells are the packet's 189, and every one of the 60 trees the
    /// original stands on its visible forest cells is one of ours — the
    /// same cell, mask, slot, place, z and model — and projects, under the
    /// pinned camera, to the packet's own `world_to_screen` pixel. Made to
    /// fail: the trees shifted by one cell keep none in register.
    #[test]
    fn the_trees_stand_where_the_packet_stands_them() {
        let Some((forest, packet)) = packet_trees() else {
            println!("skipping: no tree oracle on disk");
            return;
        };
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        world.run_to(crate::silhouette::PHOTO_FRAME);
        let art = world.tree_meshes().expect("the lobby's trees read");
        assert_eq!(art.scale.to_bits(), 0x4079_9999, "tree_scale");
        assert_eq!(art.models.iter().filter(|m| m.is_some()).count(), 24);
        let t = world.runner.terrain();
        let ours: std::collections::BTreeSet<(i32, i32)> = (0..t.height)
            .flat_map(|y| (0..t.width).map(move |x| (x, y)))
            .filter(|&(x, y)| t.flags[(y * t.width + x) as usize] & sim::world::cell::FOREST != 0)
            .collect();
        assert_eq!(ours, forest, "the forest cells");
        let list = world.tree_list();
        let at: std::collections::BTreeMap<((i32, i32), i32), &render::trees::Tree> =
            list.iter().map(|t| ((t.cell, t.slot), t)).collect();
        let m = crate::pinned_camera().to_screen_matrix();
        let screen = crate::pinned_screen();
        let project = |p: (i32, i32, i32)| {
            let q = render::camera::world_to_screen(&m, &screen, p.0, p.1, p.2, None);
            (q.x, q.y)
        };
        let register = |dx: i32| {
            packet
                .iter()
                .filter(|&&(cell, mask, slot, place, model, pixel)| {
                    at.get(&((cell.0 + dx, cell.1), slot)).is_some_and(|t| {
                        let p = (t.at.0 - dx * render::trees::CELL, t.at.1, t.at.2);
                        t.mask == mask && p == place && t.model == model && project(p) == pixel
                    })
                })
                .count()
        };
        for &(cell, mask, slot, place, model, pixel) in &packet {
            let t = at[&(cell, slot)];
            println!(
                "tree {cell:?} slot {slot}: mask {} / {mask}, at {:?} / {place:?}, model {} / {model}, pixel {:?} / {pixel:?}",
                t.mask,
                t.at,
                t.model,
                project(t.at)
            );
        }
        assert_eq!(register(0), 60, "trees in register");
        assert_eq!(packet.len(), 60);
        assert_eq!(register(1), 0, "shifted one cell east");
    }
    /// **The trees are the packet's own geometry** (item 4080,
    /// `docs/RENDERER.md` §3.7): for each of the 12 cells whose trees
    /// `RTreeData::tree_cache` holds assembled on run1559b's packet, the
    /// extent of the world-space vertices the original built agrees with the
    /// extent of our five models placed by `tree_placement`, to a unit, on
    /// all six bounds. Made to fail: drawn at 1.05 times `tree_scale`, no
    /// cell agrees.
    #[test]
    fn the_trees_are_the_packets_own_geometry() {
        let Some(text) = packet_trees().and(
            std::fs::read_to_string(
                std::env::var_os("RON_TREE_ORACLE")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(
                            "ron-data/lab-experiments/2026-11-10-item-4080/run1559b-trees.txt",
                        )
                    }),
            )
            .ok(),
        ) else {
            println!("skipping: no tree oracle on disk");
            return;
        };
        let cache: Vec<((i32, i32), [f32; 6])> = text
            .lines()
            .filter_map(|l| l.strip_prefix("cache "))
            .map(|l| {
                let w: Vec<&str> = l.split(' ').collect();
                let mut b = [0f32; 6];
                for (j, v) in b.iter_mut().enumerate() {
                    *v = w[3 + j].parse().unwrap();
                }
                ((w[0].parse().unwrap(), w[1].parse().unwrap()), b)
            })
            .collect();
        assert_eq!(cache.len(), 12, "the packet's cache entries");
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        world.run_to(crate::silhouette::PHOTO_FRAME);
        let art = world.tree_meshes().expect("the lobby's trees read");
        let list = world.tree_list();
        let extent = |cell: (i32, i32), scale: f32| {
            let mut b = [f32::MAX, f32::MAX, f32::MAX, f32::MIN, f32::MIN, f32::MIN];
            for t in list.iter().filter(|t| t.cell == cell) {
                let (rest, _) = art.models[t.model].as_ref().expect("decodes");
                let m = render::trees::tree_placement(t.at, scale);
                for &v in rest {
                    let p = super::apply(&m, v);
                    for j in 0..3 {
                        b[j] = b[j].min(p[j]);
                        b[j + 3] = b[j + 3].max(p[j]);
                    }
                }
            }
            b
        };
        let agree = |scale: f32| {
            cache
                .iter()
                .filter(|(cell, theirs)| {
                    let ours = extent(*cell, scale);
                    let ok = ours.iter().zip(theirs).all(|(a, b)| (a - b).abs() <= 1.0);
                    if scale == art.scale {
                        println!("cache {cell:?}: ours {ours:?} theirs {theirs:?}");
                    }
                    ok
                })
                .count()
        };
        assert_eq!(agree(art.scale), 12, "every cached cell's extent");
        assert_eq!(agree(art.scale * 1.05), 0, "at 1.05 times tree_scale");
    }

    /// **The leaves are the packet's own sections** (item 4082,
    /// `docs/RENDERER.md` §3.7): for each of the 12 cells whose trees
    /// `RTreeData::tree_cache` holds on run1559b's packet, each of the seven
    /// sections' vertex and index counts (`tools/render/tree_points.py`'s
    /// `section` rows) is the sum over the cell's five trees of the vertices
    /// `render::trees::slots` gives that texture and three indices for each
    /// face `face_textures` gives it. Made to fail: each texture shifted one
    /// on, no cell agrees.
    #[test]
    fn the_leaves_are_the_packets_own_sections() {
        let path = std::env::var_os("RON_TREE_SECTIONS")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-10-item-4082/run1559b-trees.txt")
            });
        let Ok(text) = std::fs::read_to_string(path) else {
            println!("skipping: no section oracle on disk");
            return;
        };
        let mut packet: std::collections::BTreeMap<(i32, i32), [(u32, u32); 7]> =
            std::collections::BTreeMap::new();
        for l in text.lines().filter_map(|l| l.strip_prefix("section ")) {
            let w: Vec<u32> = l.split(' ').map(|v| v.parse().unwrap()).collect();
            packet.entry((w[0] as i32, w[1] as i32)).or_default()[w[2] as usize] = (w[3], w[5]);
        }
        assert_eq!(packet.len(), 12, "the packet's cached cells");
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        world.run_to(crate::silhouette::PHOTO_FRAME);
        let art = world.tree_meshes().expect("the lobby's trees read");
        let list = world.tree_list();
        let ours = |cell: (i32, i32), shift: usize| {
            let mut out = [(0u32, 0u32); 7];
            for t in list.iter().filter(|t| t.cell == cell) {
                let slots = art.slots[t.model].as_ref().expect("decodes");
                for s in slots.iter().flatten() {
                    out[(s.texture + shift) % 7].0 += s.num;
                }
                let (_, faces) = art.leaves[t.model].as_ref().expect("decodes");
                for &f in faces {
                    out[(f + shift) % 7].1 += 3;
                }
            }
            out
        };
        let agree = |shift: usize| {
            packet
                .iter()
                .filter(|(cell, theirs)| {
                    let ours = ours(**cell, shift);
                    if shift == 0 {
                        println!("sections {cell:?}: ours {ours:?} theirs {theirs:?}");
                    }
                    ours == **theirs
                })
                .count()
        };
        assert_eq!(agree(0), 12, "every cached cell's sections");
        assert_eq!(agree(1), 0, "each texture shifted one on");
    }

    use sim::data::pivot::{Release, release_offset_at, release_rotation};
    use sim::movement::Angle;

    /// **A figure is drawn by the original's own model-to-world map**
    /// (item 4069): `guy_placement` at the degree `GuyOut::draw` hands
    /// `rotate_z` lands a model point where `get_position`'s port
    /// (`release_offset_at`, packet-backed at all 256 headings) puts it, to
    /// the unit it truncates to, at every top-byte facing. `placement`'s
    /// unmirrored turn, which the client drew figures by before, misses —
    /// the reflection in world y.
    #[test]
    fn a_figure_is_turned_as_get_position_turns_a_model_point() {
        let m = [100.0, 40.0, 30.0];
        let r = Release {
            piece: -1,
            node: 0,
            pivot: [0; 3],
            at: [100_000_000, 40_000_000, 30_000_000],
            float_lands: &[],
        };
        let mut old_misses = 0;
        for i in 0..256u32 {
            let facing = i << 24;
            let d = release_rotation(Angle(facing as i32));
            let (x, y, z) = release_offset_at(&r, d, 0);
            let p = apply(&guy_placement([0, 0, 0], d, 4.8), m);
            let near = |a: f32, b: i32| (a - b as f32).abs() < 1.0;
            assert!(
                near(p[0], x) && near(p[1], y) && near(p[2], z),
                "facing {facing:#x} at {d}°: drawn {p:?}, get_position ({x}, {y}, {z})"
            );
            let q = apply(&placement([0, 0, 0], facing, 4.8), m);
            if !(near(q[0], x) && near(q[1], y)) {
                old_misses += 1;
            }
        }
        assert!(
            old_misses > 200,
            "the unmirrored turn agrees too often: {old_misses}"
        );
    }
}
